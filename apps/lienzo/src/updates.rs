//! The release check behind `help.checkForUpdates` for the CLI, MCP and automation, through
//! `nodalia-update`. The windows people see (and the install itself) are `lienzo_ui_egui::updates_ui`.

use lienzo_engine::updates_cmds::{CheckFn, CheckResult, Release};

pub fn service() -> CheckFn {
    Box::new(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("update check".into())
            .spawn(move || {
                let result = check();
                if let Err(e) = &result {
                    log::warn!("Update check: {e}");
                }
                let _ = tx.send(result);
            })
            .map_err(|e| format!("Could not start update check: {e}"))?;
        Ok(rx)
    })
}

fn check() -> CheckResult {
    match nodalia_update::check(&lienzo_ui_egui::UPDATE_APP).map_err(|e| e.to_string())? {
        nodalia_update::Check::UpToDate => Ok(None),
        nodalia_update::Check::Available(release) => Ok(Some(Release { version: release.version, url: release.page })),
    }
}
