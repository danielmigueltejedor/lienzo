//! In-app updates: Help ▸ Check for Updates, the check at start-up, and the windows that follow.
//!
//! The flow is `nodalia_update::egui::Updater`, the one every Nodalia app shares: it asks GitHub
//! Releases, verifies the package against its published SHA-256, replaces the AppImage or the macOS
//! app and restarts. Installs that something else manages (Flatpak, `.deb`, `.rpm`, Windows
//! installers, a dev build) are pointed at the release page instead. The engine's own update state
//! (`session.updates`, read by `ui.inspect`, the CLI and MCP) mirrors what the updater is doing.

use serde_json::Value;

use crate::LienzoApp;

pub(crate) struct Runtime {
    #[cfg(not(target_arch = "wasm32"))]
    updater: nodalia_update::egui::Updater,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            updater: nodalia_update::egui::Updater::new(crate::UPDATE_APP),
        }
    }
}

/// Help ▸ Check for Updates: looks for a release now and says what it found.
#[cfg(target_arch = "wasm32")]
pub(crate) fn check(app: &mut LienzoApp, _ctx: &egui::Context, params: Value) -> Result<Value, String> {
    app.run("help.checkForUpdates", params)
}

/// Help ▸ Check for Updates: looks for a release now and says what it found.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn check(app: &mut LienzoApp, ctx: &egui::Context, params: Value) -> Result<Value, String> {
    if !params.as_object().is_some_and(|p| p.is_empty()) {
        return Err("update commands take an empty object".into());
    }
    if app.session.updates.check.is_none() {
        return Err("Update checks are unavailable in this frontend".into());
    }
    app.updates_rt.updater.check_now(ctx);
    mirror(app);
    Ok(serde_json::json!(app.session.updates.status))
}

/// Shows `version` as if a check had found it, for screenshots.
#[cfg(not(target_arch = "wasm32"))]
pub fn preview(app: &mut LienzoApp, version: &str, notes: &str) {
    let page = format!("{}/tag/v{version}", lienzo_engine::updates_cmds::RELEASES);
    app.updates_rt.updater.offer(nodalia_update::Release {
        tag: format!("v{version}"),
        version: version.to_owned(),
        notes: notes.to_owned(),
        page,
        asset: None,
    });
    mirror(app);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn tick(_app: &mut LienzoApp, _ctx: &egui::Context) {}

/// Once per launch the updater looks for a release (after a short wait, at most once a day, and only
/// where this install can replace itself) unless Preferences ▸ General turns that off.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn tick(app: &mut LienzoApp, ctx: &egui::Context) {
    if app.session.updates.check.is_some() && app.session.prefs().general.automatically_check_updates {
        app.updates_rt.updater.check_at_start(ctx);
    }
    // An install that finishes here (no window open) says so through `poll`, and its exit counts.
    let action = app.updates_rt.updater.poll();
    mirror(app);
    act(app, ctx, action);
}

/// The engine's update state follows the updater's, so `ui.inspect`, the CLI and MCP see the same.
#[cfg(not(target_arch = "wasm32"))]
fn mirror(app: &mut LienzoApp) {
    use lienzo_engine::updates_cmds::{Release, Status};
    use nodalia_update::egui::Status as Flow;
    let updater = &app.updates_rt.updater;
    let found = |release: &nodalia_update::Release| Status::Available { release: Release { version: release.version.clone(), url: release.page.clone() } };
    let status = match updater.status() {
        Flow::Checking { .. } => Some(Status::Checking),
        Flow::UpToDate => Some(Status::UpToDate),
        Flow::CheckFailed(message) | Flow::InstallFailed(message) => Some(Status::Failed { message: message.clone() }),
        Flow::Offer(release) | Flow::Downloading { release, .. } | Flow::Ready { release, .. } | Flow::Installing(release) => Some(found(release)),
        // A check that found nothing says nothing: the engine stops showing it as running.
        Flow::Idle if app.session.updates.status == Status::Checking => Some(Status::Idle),
        Flow::Idle | Flow::LastFailed(_) => None,
    };
    if let Some(status) = status {
        app.session.updates.status = status;
    }
    app.ui.updates_open = updater.is_open();
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn show(_app: &mut LienzoApp, _ctx: &egui::Context) {}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn show(app: &mut LienzoApp, ctx: &egui::Context) {
    let language = nodalia_update::Language::from_code(crate::i18n::current().code());
    // An update installs, and a downloaded one restarts the app, only while nothing is unsaved.
    let safe_to_close = !app.has_unsaved_work();
    let action = app.updates_rt.updater.show(ctx, language, safe_to_close);
    mirror(app);
    act(app, ctx, action);
}

#[cfg(not(target_arch = "wasm32"))]
fn act(app: &mut LienzoApp, ctx: &egui::Context, action: nodalia_update::egui::Action) {
    use nodalia_update::egui::Action;
    match action {
        Action::None => {}
        Action::OpenUrl(url) => {
            crate::links::open(app, ctx, &url);
        }
        // The new version is starting (or a helper waits for this process to end): leave now, after
        // the same last steps a normal close takes.
        Action::Exit => {
            eframe::App::on_exit(app);
            std::process::exit(0);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::{Services, menus};
    use lienzo_engine::{Session, updates_cmds::Status};
    use nodalia_update::{Check, Release, egui::Updater};
    use std::sync::Arc;
    use std::time::Duration;

    fn app_with(check: Option<lienzo_engine::updates_cmds::CheckFn>) -> LienzoApp {
        let mut app = LienzoApp::new(Session::new(), Services { check_updates: check, ..Default::default() });
        // A stand-in for GitHub, and no stamp file: a test never reads or writes the update folder.
        app.updates_rt.updater =
            Updater::new(crate::UPDATE_APP).with_stamp(None).with_startup_delay(Duration::ZERO).with_source(Arc::new(|| Ok(Check::UpToDate)));
        app
    }

    fn enabled() -> Option<lienzo_engine::updates_cmds::CheckFn> {
        Some(Box::new(|| Err("the engine's own transport is not used by the window".into())))
    }

    fn release() -> Release {
        Release { tag: "v9.9.9".into(), version: "9.9.9".into(), notes: "Notes".into(), page: "https://example.test/r".into(), asset: None }
    }

    fn frame(app: &mut LienzoApp, ctx: &egui::Context, time: f64) {
        let mut output = ctx.run_ui(egui::RawInput { time: Some(time), ..Default::default() }, |ui| {
            tick(app, ui.ctx());
            show(app, ui.ctx());
        });
        output.textures_delta.clear();
    }

    fn wait_for(app: &mut LienzoApp, ctx: &egui::Context, done: impl Fn(&LienzoApp) -> bool) {
        for step in 0..200 {
            frame(app, ctx, f64::from(step));
            if done(app) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(done(app), "the update check did not finish");
    }

    #[test]
    fn manual_check_opens_the_window_and_reports_up_to_date() {
        let mut app = app_with(enabled());
        let ctx = egui::Context::default();
        app.session.edit_prefs(|p| p.general.automatically_check_updates = false);
        assert!(menus::is_enabled(&app, "help.checkForUpdates"));
        menus::invoke(&mut app, &ctx, "help.checkForUpdates", serde_json::json!({})).unwrap();
        assert!(app.ui.updates_open);
        assert_eq!(app.session.updates.status, Status::Checking);
        wait_for(&mut app, &ctx, |app| app.session.updates.status == Status::UpToDate);
        assert!(app.ui.updates_open, "a manual check always says what it found");
    }

    #[test]
    fn automatic_check_stays_quiet_when_nothing_is_new_and_obeys_the_preference() {
        let ctx = egui::Context::default();
        let mut on = app_with(enabled());
        wait_for(&mut on, &ctx, |app| {
            !matches!(app.updates_rt.updater.status(), nodalia_update::egui::Status::Checking { .. }) && app.session.updates.status == Status::Idle
        });
        assert!(!on.ui.updates_open);

        let mut off = app_with(enabled());
        off.session.edit_prefs(|p| p.general.automatically_check_updates = false);
        frame(&mut off, &ctx, 0.0);
        assert_eq!(off.updates_rt.updater.status(), &nodalia_update::egui::Status::Idle, "opt-out is honoured");
    }

    #[test]
    fn an_available_release_opens_the_window_and_is_mirrored_to_the_engine() {
        let mut app = app_with(enabled());
        let ctx = egui::Context::default();
        // The check at start would otherwise replace the offer with its own wait.
        app.session.edit_prefs(|p| p.general.automatically_check_updates = false);
        app.updates_rt.updater.offer(release());
        frame(&mut app, &ctx, 0.0);
        assert!(app.ui.updates_open);
        assert!(matches!(&app.session.updates.status, Status::Available { release } if release.version == "9.9.9"));
        app.updates_rt.updater.set_status(nodalia_update::egui::Status::Idle);
        frame(&mut app, &ctx, 1.0);
        assert!(!app.ui.updates_open);
    }

    #[test]
    fn bad_params_and_unsupported_frontends_are_refused() {
        let ctx = egui::Context::default();
        let mut app = app_with(enabled());
        assert!(menus::invoke(&mut app, &ctx, "help.checkForUpdates", serde_json::json!({"x": 1})).is_err());
        assert!(!app.ui.updates_open);
        let mut none = app_with(None);
        assert!(!menus::is_enabled(&none, "help.checkForUpdates"));
        assert!(check(&mut none, &ctx, serde_json::json!({})).is_err());
        assert!(!none.ui.updates_open);
    }
}
