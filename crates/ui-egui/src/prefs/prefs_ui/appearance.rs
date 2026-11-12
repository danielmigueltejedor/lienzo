use super::fields::{color_of, preference_matches};
use super::{AppearanceMode, Color32, DarkTheme, LienzoApp, LightTheme, Map, RichText, ThemeKind, Tokens, Value, json, prefs};

/// The GPU canvas colours from Preferences › Transparency & Gamut.
pub fn canvas_style(app: &LienzoApp) -> crate::gpu_canvas::CanvasStyle {
    let t = &app.session.prefs().transparency_and_gamut;
    let [l, d] = t.colors();
    let f = |c: [u8; 3]| c.map(|v| v as f32 / 255.0);
    let g = prefs::parse_hex(&t.gamut_warning_color).unwrap_or([128; 3]);
    crate::gpu_canvas::CanvasStyle {
        checker_square: t.square().unwrap_or(0.0),
        checker_light: f(l),
        checker_dark: f(d),
        gamut_color: f(g),
        gamut_opacity: t.gamut_warning_opacity as f32 / 100.0,
    }
}

/// Pasteboard colour from Preferences › Interface (`None` = the theme's default canvas).
pub fn pasteboard_color(app: &LienzoApp) -> Option<Color32> {
    use prefs::CanvasColor;
    let i = &app.session.prefs().interface;
    Some(match i.canvas_color {
        CanvasColor::Default => return None,
        CanvasColor::Black => Color32::BLACK,
        CanvasColor::DarkGray => Color32::from_gray(40),
        CanvasColor::MediumGray => Color32::from_gray(83),
        CanvasColor::LightGray => Color32::from_gray(184),
        CanvasColor::Custom => color_of(&i.canvas_custom_color),
    })
}

pub(super) fn selected_theme(interface: &prefs::Interface, system: Option<egui::Theme>) -> ThemeKind {
    let light = match interface.appearance_mode {
        AppearanceMode::Light => true,
        AppearanceMode::Dark => false,
        AppearanceMode::Auto => system == Some(egui::Theme::Light),
    };
    // Lienzo has one design language, Nodalia's: a style saved by an older version (Pro, Studio,
    // Adwaita…) resolves to it, in the mode chosen. Unit tests keep the dense Pro look their
    // geometry was written against, as `ThemeKind::default` does.
    if !cfg!(test) {
        return if light { ThemeKind::NodaliaLight } else { ThemeKind::Nodalia };
    }
    if light {
        match interface.light_theme {
            LightTheme::NodaliaLight => ThemeKind::NodaliaLight,
            LightTheme::StudioLight => ThemeKind::StudioLight,
            LightTheme::ProLightGray => ThemeKind::ProLightGray,
            LightTheme::ProLight => ThemeKind::ProLight,
            LightTheme::Classic => ThemeKind::Classic,
            LightTheme::Adwaita => ThemeKind::Adwaita,
            LightTheme::BreezeLight => ThemeKind::BreezeLight,
            LightTheme::SoftLight => ThemeKind::SoftLight,
        }
    } else {
        match interface.dark_theme {
            DarkTheme::Nodalia => ThemeKind::Nodalia,
            DarkTheme::Pro => ThemeKind::Pro,
            DarkTheme::ProMedium => ThemeKind::ProMedium,
            DarkTheme::Studio => ThemeKind::Studio,
            DarkTheme::SolarizedDark => ThemeKind::SolarizedDark,
            DarkTheme::AdwaitaDark => ThemeKind::AdwaitaDark,
            DarkTheme::BreezeDark => ThemeKind::BreezeDark,
            DarkTheme::SoftDark => ThemeKind::SoftDark,
        }
    }
}

pub(crate) fn cycle_appearance(app: &mut LienzoApp, ctx: &egui::Context) {
    let next = match app.session.prefs().interface.appearance_mode {
        AppearanceMode::Auto => AppearanceMode::Light,
        AppearanceMode::Light => AppearanceMode::Dark,
        AppearanceMode::Dark => AppearanceMode::Auto,
    };
    if let Err(e) = app.run("prefs.set", json!({"path": "interface.appearanceMode", "value": next.name()})) {
        app.ui.status = e;
        return;
    }
    let selected = selected_theme(&app.session.prefs().interface, system_theme(app, ctx));
    if app.ui.theme != selected {
        app.apply_theme(ctx, selected);
    }
}

/// Photoshop's four interface brightnesses, darkest first.
const BRIGHTNESSES: [ThemeKind; 4] = [ThemeKind::Pro, ThemeKind::ProMedium, ThemeKind::ProLightGray, ThemeKind::ProLight];

/// Darken Interface (⇧⌘F1, `lighter` false) and Lighten Interface (⇧⌘F2): the next of
/// Photoshop's four brightnesses from the theme on screen, stopping at either end, as in
/// Photoshop. Another theme steps from the nearest brightness: Medium Gray for a dark one, Light
/// Gray for a light one. The choice is saved like picking it in Preferences › Interface.
pub(crate) fn step_brightness(app: &mut LienzoApp, ctx: &egui::Context, lighter: bool) -> Result<(), String> {
    let current = app.ui.theme;
    let at = BRIGHTNESSES.iter().position(|k| *k == current).unwrap_or(if crate::theme::Tokens::for_kind(current).dark() { 1 } else { 2 });
    let next = if lighter { (at + 1).min(BRIGHTNESSES.len() - 1) } else { at.saturating_sub(1) };
    let Some(&kind) = BRIGHTNESSES.get(next) else { return Ok(()) };
    if kind == current {
        return Ok(());
    }
    app.run("prefs.set", json!({"path": "interface.theme", "value": kind.id()}))?;
    sync_appearance(app, ctx);
    Ok(())
}

pub(super) fn system_theme(app: &LienzoApp, ctx: &egui::Context) -> Option<egui::Theme> {
    app.services.system_theme.as_ref().and_then(|read| read(ctx)).or_else(|| ctx.system_theme())
}

/// Resolve an absolute pixels-per-point preference without multiplying the OS DPI twice.
/// Monitor dimensions are physical pixels, independent of window size and UI zoom.
pub(super) fn display_scale(pref: prefs::UiScale, native: Option<f32>) -> f32 {
    let native = native.filter(|v| v.is_finite() && *v > 0.0).unwrap_or(1.0);
    match pref {
        // The system's scale, fractional ones included (125%, 150%, 175%: #1072), and 100% on a
        // large 4K display run unscaled (#1278); 200% there is a fixed choice.
        prefs::UiScale::Auto => native,
        fixed => fixed.name().parse::<f32>().map_or(1.0, |pct| pct / 100.0),
    }
}

pub(super) fn sync_display_scale(app: &LienzoApp, ctx: &egui::Context) {
    let native = ctx.native_pixels_per_point();
    let native_scale = native.filter(|v| v.is_finite() && *v > 0.0).unwrap_or(1.0);
    let scale = display_scale(app.session.prefs().interface.ui_scale, native);
    ctx.set_zoom_factor(scale / native_scale);
}

/// Interface › Show Tooltips and Tools › Show Tooltips (either one off hides them): egui never
/// shows a tooltip whose delay is infinite. Re-checked every frame because a theme change
/// rebuilds the style; that's one style read, and a write only when it differs.
pub(super) fn sync_tooltips(app: &LienzoApp, ctx: &egui::Context) {
    let p = app.session.prefs();
    let delay = if p.interface.show_tooltips && p.tools.show_tooltips { crate::theme::TOOLTIP_DELAY } else { f32::INFINITY };
    if ctx.global_style().interaction.tooltip_delay != delay {
        ctx.global_style_mut(|s| s.interaction.tooltip_delay = delay);
    }
}

/// Resolve the saved appearance mode without changing it or either theme slot.
pub(crate) fn sync_appearance(app: &mut LienzoApp, ctx: &egui::Context) {
    // A restored or embedding-supplied pin must not suppress native appearance events.
    ctx.set_theme(egui::ThemePreference::System);
    // The native service also covers Wayland sessions where winit has no system theme.
    let selected = selected_theme(&app.session.prefs().interface, system_theme(app, ctx));
    if app.ui.theme != selected {
        app.apply_theme(ctx, selected);
    }
}

/// Whether the Appearance Mode row shows for a search. Light and dark are the only choices: the
/// interface is always Nodalia's, so there are no theme swatches to pick from.
#[derive(Clone, Copy)]
pub(super) struct AppearanceRows {
    mode: bool,
}

impl AppearanceRows {
    /// `query` is lowercased; empty shows every row.
    pub(super) fn matching(query: &str) -> Self {
        Self { mode: query.is_empty() || preference_matches("interface", "appearanceMode", query) }
    }

    pub(super) fn any(self) -> bool {
        self.mode
    }
}

/// Appearance Mode, as a row of the Appearance grid.
pub(super) fn appearance_rows(ui: &mut egui::Ui, obj: &mut Map<String, Value>, rows: AppearanceRows) {
    let t = Tokens::get(ui.ctx());
    let mut mode = obj.get("appearanceMode").and_then(Value::as_str).unwrap_or("auto").to_string();
    if rows.mode {
        ui.label(RichText::new(tl!("Appearance Mode")).color(t.text_dim));
        let choices: [(String, &str); 3] = [("auto".into(), tl!("Sync with system")), ("dark".into(), tl!("Dark")), ("light".into(), tl!("Light"))];
        let pairs: Vec<(String, &str)> = choices.iter().map(|(id, label)| (id.clone(), *label)).collect();
        crate::widgets::dropdown(ui, "appearance-mode", tl!("Appearance Mode"), &mut mode, &pairs, 220.0);
        ui.end_row();
    }
    obj.insert("appearanceMode".into(), json!(mode));
}
