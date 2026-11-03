//! Linux caption buttons in the existing header. The pixels come from the shell (see the desktop
//! app's GTK helper). Without those pixels the desktop frame stays, so this module never draws a
//! second set of buttons. macOS and Windows keep their own frames.

use egui::{Color32, CursorIcon, Pos2, Rect, Sense, Vec2, pos2};

pub const RESIZE_BAND: f32 = 4.0;
pub const RESIZE_CORNER: f32 = 10.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowBackend {
    Wayland,
    X11,
    #[default]
    Other,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DesktopKind {
    Gnome,
    Kde,
    #[default]
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptionButton {
    Minimize,
    Maximize,
    Close,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ButtonLayout {
    pub left: Vec<CaptionButton>,
    pub right: Vec<CaptionButton>,
}

impl ButtonLayout {
    fn contains(&self, button: CaptionButton) -> bool {
        self.left.iter().chain(self.right.iter()).any(|b| *b == button)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameMode {
    System,
    Integrated { buttons: ButtonLayout },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FramePlan {
    pub mode: FrameMode,
}

impl FramePlan {
    pub fn draws_controls(&self) -> bool {
        matches!(self.mode, FrameMode::Integrated { .. })
    }

    pub fn hide_system_frame(&self) -> bool {
        self.draws_controls()
    }

    pub fn buttons(&self) -> ButtonLayout {
        match &self.mode {
            FrameMode::Integrated { buttons } => buttons.clone(),
            FrameMode::System => ButtonLayout::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowHost {
    pub backend: WindowBackend,
    pub buttons: Option<ButtonLayout>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DesktopSignals {
    pub xdg_current_desktop: String,
    pub xdg_session_desktop: String,
    pub desktop_session: String,
    pub kde_full_session: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveWindow {
    pub maximized: bool,
    pub fullscreen: bool,
    pub resizable: bool,
}

impl Default for LiveWindow {
    fn default() -> Self {
        Self { maximized: false, fullscreen: false, resizable: true }
    }
}

impl LiveWindow {
    pub fn from_ctx(ctx: &egui::Context) -> Self {
        ctx.input(|i| {
            let v = i.viewport();
            Self { maximized: v.maximized.unwrap_or(false), fullscreen: v.fullscreen.unwrap_or(false), resizable: true }
        })
    }

    fn showing_restore(self) -> bool {
        self.maximized || self.fullscreen
    }
}

/// Names win. A config file counts only when the session named nothing, and only for one family.
pub fn classify_desktop(signals: &DesktopSignals, kwin_config: bool, gnome_layout: bool) -> DesktopKind {
    let tokens = [signals.xdg_current_desktop.as_str(), signals.xdg_session_desktop.as_str(), signals.desktop_session.as_str()]
        .join(":")
        .split([':', ';', ','])
        .map(|t| t.trim().to_ascii_lowercase())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>();
    if signals.kde_full_session || tokens.iter().any(|t| t == "kde" || t == "plasma") {
        return DesktopKind::Kde;
    }
    if tokens.iter().any(|t| matches!(t.as_str(), "gnome" | "gnome-classic" | "ubuntu" | "ubuntu-wayland" | "budgie" | "pop")) {
        return DesktopKind::Gnome;
    }
    if !tokens.is_empty() {
        return DesktopKind::Other;
    }
    match (kwin_config, gnome_layout) {
        (true, false) => DesktopKind::Kde,
        (false, true) => DesktopKind::Gnome,
        _ => DesktopKind::Other,
    }
}

pub fn parse_gnome_layout(text: &str) -> Option<ButtonLayout> {
    let text = text.trim().trim_matches('"').trim_matches('\'').trim();
    let (left, right) = text.split_once(':')?;
    Some(ButtonLayout { left: gnome_side(left), right: gnome_side(right) })
}

fn gnome_side(side: &str) -> Vec<CaptionButton> {
    side.split(',')
        .filter_map(|name| match name.trim().to_ascii_lowercase().as_str() {
            "minimize" => Some(CaptionButton::Minimize),
            "maximize" => Some(CaptionButton::Maximize),
            "close" => Some(CaptionButton::Close),
            _ => None,
        })
        .collect()
}

/// `None` when `[org.kde.kdecoration2]` does not name either side.
pub fn parse_kwin_buttons(text: &str) -> Option<ButtonLayout> {
    let mut in_group = false;
    let mut left = None;
    let mut right = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_group = line.eq_ignore_ascii_case("[org.kde.kdecoration2]");
            continue;
        }
        if !in_group || line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        match key.trim() {
            "ButtonsOnLeft" => left = Some(kwin_side(value.trim())),
            "ButtonsOnRight" => right = Some(kwin_side(value.trim())),
            _ => {}
        }
    }
    if left.is_none() && right.is_none() {
        return None;
    }
    Some(ButtonLayout { left: left.unwrap_or_default(), right: right.unwrap_or_default() })
}

fn kwin_side(value: &str) -> Vec<CaptionButton> {
    value
        .chars()
        .filter_map(|c| match c.to_ascii_uppercase() {
            'I' => Some(CaptionButton::Minimize),
            'A' => Some(CaptionButton::Maximize),
            'X' => Some(CaptionButton::Close),
            _ => None,
        })
        .collect()
}

fn default_layout() -> ButtonLayout {
    ButtonLayout { left: Vec::new(), right: vec![CaptionButton::Minimize, CaptionButton::Maximize, CaptionButton::Close] }
}

fn ensure_close(mut layout: ButtonLayout) -> ButtonLayout {
    if !layout.contains(CaptionButton::Close) {
        layout.right.push(CaptionButton::Close);
    }
    layout
}

/// Linux integrates only when the shell buttons were captured and the window is really Wayland or X11.
pub fn plan(host: &WindowHost, captured: bool) -> FramePlan {
    let mode = if cfg!(target_os = "linux") && captured && matches!(host.backend, WindowBackend::Wayland | WindowBackend::X11) {
        FrameMode::Integrated { buttons: ensure_close(host.buttons.clone().unwrap_or_else(default_layout)) }
    } else {
        FrameMode::System
    };
    FramePlan { mode }
}

pub fn caption_command(button: CaptionButton, live: &LiveWindow) -> Option<egui::ViewportCommand> {
    match button {
        CaptionButton::Minimize => Some(egui::ViewportCommand::Minimized(true)),
        CaptionButton::Close => Some(egui::ViewportCommand::Close),
        CaptionButton::Maximize if live.fullscreen => Some(egui::ViewportCommand::Fullscreen(false)),
        CaptionButton::Maximize if live.maximized => Some(egui::ViewportCommand::Maximized(false)),
        CaptionButton::Maximize if !live.resizable => None,
        CaptionButton::Maximize => Some(egui::ViewportCommand::Maximized(true)),
    }
}

pub fn title_drag_rect(full: Rect, edge: f32) -> Rect {
    let top = (full.top() + edge.max(0.0)).min(full.bottom());
    Rect::from_min_max(pos2(full.left(), top), full.right_bottom())
}

#[derive(Clone, Debug)]
pub struct CaptionRaster {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Twelve tiles: role (minimize, maximize, restore, close) times state (rest, hover, active).
#[derive(Clone, Debug)]
pub struct GnomeCaptions {
    pub button: f32,
    pub image: f32,
    pub gap: f32,
    pub rasters: [CaptionRaster; 12],
}

impl GnomeCaptions {
    pub fn index(button: CaptionButton, restore: bool, state: u8) -> usize {
        let role = match button {
            CaptionButton::Minimize => 0,
            CaptionButton::Maximize if restore => 2,
            CaptionButton::Maximize => 1,
            CaptionButton::Close => 3,
        };
        role * 3 + (state as usize).min(2)
    }

    pub fn encode(button: u16, image: u16, gap: u16, tiles: &[(u8, u8, u16, u16, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"GCAP");
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&button.to_le_bytes());
        out.extend_from_slice(&image.to_le_bytes());
        out.extend_from_slice(&gap.to_le_bytes());
        out.extend_from_slice(&(tiles.len() as u16).to_le_bytes());
        for (role, state, w, h, rgba) in tiles {
            out.push(*role);
            out.push(*state);
            out.extend_from_slice(&w.to_le_bytes());
            out.extend_from_slice(&h.to_le_bytes());
            out.extend_from_slice(rgba);
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 14 || &bytes[..4] != b"GCAP" {
            return None;
        }
        if u16::from_le_bytes(bytes[4..6].try_into().ok()?) != 1 {
            return None;
        }
        let button = u16::from_le_bytes(bytes[6..8].try_into().ok()?) as f32;
        let image = u16::from_le_bytes(bytes[8..10].try_into().ok()?) as f32;
        let gap = u16::from_le_bytes(bytes[10..12].try_into().ok()?) as f32;
        let count = u16::from_le_bytes(bytes[12..14].try_into().ok()?) as usize;
        let mut rasters: [Option<CaptionRaster>; 12] = std::array::from_fn(|_| None);
        let mut i = 14;
        for _ in 0..count {
            if i + 6 > bytes.len() {
                return None;
            }
            let role = bytes[i];
            let state = bytes[i + 1];
            let w = u16::from_le_bytes(bytes[i + 2..i + 4].try_into().ok()?) as u32;
            let h = u16::from_le_bytes(bytes[i + 4..i + 6].try_into().ok()?) as u32;
            i += 6;
            let n = (w as usize).checked_mul(h as usize)?.checked_mul(4)?;
            let end = i.checked_add(n)?;
            let rgba = bytes.get(i..end)?.to_vec();
            i = end;
            if role > 3 || state > 2 || w == 0 || h == 0 {
                return None;
            }
            rasters[(role as usize) * 3 + state as usize] = Some(CaptionRaster { width: w, height: h, rgba });
        }
        if i != bytes.len() {
            return None;
        }
        let rasters = rasters.into_iter().collect::<Option<Vec<_>>>()?.try_into().ok()?;
        Some(Self { button, image, gap, rasters })
    }
}

pub struct GnomeCaptionTextures {
    pub handles: [egui::TextureHandle; 12],
}

impl GnomeCaptionTextures {
    pub fn load(ctx: &egui::Context, captions: &GnomeCaptions) -> Self {
        let handles = std::array::from_fn(|i| {
            let raster = &captions.rasters[i];
            let image = egui::ColorImage::from_rgba_unmultiplied([raster.width as usize, raster.height as usize], &raster.rgba);
            ctx.load_texture(format!("gnome-caption-{i}"), image, egui::TextureOptions::LINEAR)
        });
        Self { handles }
    }
}

pub(crate) fn paint_caption_buttons(
    ui: &mut egui::Ui,
    buttons: &[CaptionButton],
    live: &LiveWindow,
    right_to_left: bool,
    captions: &GnomeCaptions,
    textures: &GnomeCaptionTextures,
) {
    let n = buttons.len();
    if n == 0 {
        return;
    }
    let prev = ui.spacing().item_spacing.x;
    ui.spacing_mut().item_spacing.x = captions.gap;
    for i in 0..n {
        let idx = if right_to_left { n - 1 - i } else { i };
        let Some(button) = buttons.get(idx).copied() else { continue };
        let side = captions.button.max(1.0);
        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
        let restore = live.showing_restore();
        let enabled = !matches!(button, CaptionButton::Maximize) || live.resizable || restore;
        let state = if !enabled {
            0
        } else if resp.is_pointer_button_down_on() {
            2
        } else if resp.hovered() {
            1
        } else {
            0
        };
        let tex = &textures.handles[GnomeCaptions::index(button, restore, state)];
        let image = captions.image.min(side);
        let tint = if enabled { Color32::WHITE } else { Color32::from_white_alpha(90) };
        ui.painter().image(tex.id(), Rect::from_center_size(rect.center(), Vec2::splat(image)), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
        if resp.clicked()
            && enabled
            && let Some(cmd) = caption_command(button, live)
        {
            ui.ctx().send_viewport_cmd(cmd);
        }
    }
    ui.spacing_mut().item_spacing.x = 0.0;
    ui.add_space(10.0);
    ui.spacing_mut().item_spacing.x = prev;
}

pub fn resize_zones(screen: Rect) -> Vec<(Rect, egui::ResizeDirection, CursorIcon)> {
    let (b, c) = (RESIZE_BAND, RESIZE_CORNER);
    let (x0, y0, x1, y1) = (screen.left(), screen.top(), screen.right(), screen.bottom());
    let zone = |a: Pos2, b: Pos2| Rect::from_two_pos(a, b);
    vec![
        (zone(pos2(x0 + c, y0), pos2(x1 - c, y0 + b)), egui::ResizeDirection::North, CursorIcon::ResizeNorth),
        (zone(pos2(x0 + c, y1 - b), pos2(x1 - c, y1)), egui::ResizeDirection::South, CursorIcon::ResizeSouth),
        (zone(pos2(x0, y0 + c), pos2(x0 + b, y1 - c)), egui::ResizeDirection::West, CursorIcon::ResizeWest),
        (zone(pos2(x1 - b, y0 + c), pos2(x1, y1 - c)), egui::ResizeDirection::East, CursorIcon::ResizeEast),
        (zone(pos2(x0, y0), pos2(x0 + c, y0 + c)), egui::ResizeDirection::NorthWest, CursorIcon::ResizeNorthWest),
        (zone(pos2(x1 - c, y0), pos2(x1, y0 + c)), egui::ResizeDirection::NorthEast, CursorIcon::ResizeNorthEast),
        (zone(pos2(x0, y1 - c), pos2(x0 + c, y1)), egui::ResizeDirection::SouthWest, CursorIcon::ResizeSouthWest),
        (zone(pos2(x1 - c, y1 - c), pos2(x1, y1)), egui::ResizeDirection::SouthEast, CursorIcon::ResizeSouthEast),
    ]
}

pub(crate) fn resize_borders(ui: &mut egui::Ui, enabled: bool) {
    if !enabled {
        return;
    }
    let screen = ui.ctx().viewport_rect();
    for (i, (rect, dir, cursor)) in resize_zones(screen).into_iter().enumerate() {
        if rect.width() < 1.0 || rect.height() < 1.0 {
            continue;
        }
        let resp = ui.interact(rect, ui.id().with(("frame-resize", i)), Sense::drag()).on_hover_cursor(cursor);
        if resp.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
        }
    }
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
pub fn detect_host(backend: WindowBackend) -> WindowHost {
    let signals = DesktopSignals {
        xdg_current_desktop: std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        xdg_session_desktop: std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default(),
        desktop_session: std::env::var("DESKTOP_SESSION").unwrap_or_default(),
        kde_full_session: std::env::var("KDE_FULL_SESSION").ok().is_some_and(|v| v == "true" || v == "1"),
    };
    let kwin = read_kwin_layout();
    let prelim = classify_desktop(&signals, kwin.is_some(), false);
    let gnome = if prelim == DesktopKind::Kde { None } else { read_gnome_layout() };
    let desktop = classify_desktop(&signals, kwin.is_some(), gnome.is_some());
    let buttons = match desktop {
        DesktopKind::Kde => kwin,
        DesktopKind::Gnome => gnome,
        DesktopKind::Other => None,
    };
    WindowHost { backend, buttons }
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn read_kwin_layout() -> Option<ButtonLayout> {
    for path in config_files("kwinrc") {
        if let Some(text) = read_limited(&path)
            && let Some(layout) = parse_kwin_buttons(&text)
        {
            return Some(layout);
        }
    }
    for bin in ["kreadconfig6", "kreadconfig5"] {
        let left = run_stdout(bin, &["--file", "kwinrc", "--group", "org.kde.kdecoration2", "--key", "ButtonsOnLeft"]);
        let right = run_stdout(bin, &["--file", "kwinrc", "--group", "org.kde.kdecoration2", "--key", "ButtonsOnRight"]);
        if left.is_some() || right.is_some() {
            return Some(ButtonLayout { left: left.as_deref().map(kwin_side).unwrap_or_default(), right: right.as_deref().map(kwin_side).unwrap_or_default() });
        }
    }
    None
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn read_gnome_layout() -> Option<ButtonLayout> {
    parse_gnome_layout(&run_stdout("gsettings", &["get", "org.gnome.desktop.wm.preferences", "button-layout"])?)
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn config_files(name: &str) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").filter(|s| !s.is_empty()) {
        out.push(std::path::PathBuf::from(dir).join(name));
    } else if let Some(home) = std::env::var_os("HOME") {
        out.push(std::path::PathBuf::from(home).join(".config").join(name));
    }
    for dir in std::env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".into()).split(':').filter(|d| !d.is_empty()) {
        out.push(std::path::Path::new(dir).join(name));
    }
    out
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn read_limited(path: &std::path::Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > 1_048_576 {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn run_stdout(program: &str, args: &[&str]) -> Option<String> {
    let child = std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    let out = rx.recv_timeout(std::time::Duration::from_millis(500)).ok()?.ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim().trim_matches(['\'', '"']).trim();
    if text.is_empty() { None } else { Some(text.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signals(current: &str, session: &str, desktop: &str, kde: bool) -> DesktopSignals {
        DesktopSignals { xdg_current_desktop: current.into(), xdg_session_desktop: session.into(), desktop_session: desktop.into(), kde_full_session: kde }
    }

    fn host(backend: WindowBackend, buttons: Option<ButtonLayout>) -> WindowHost {
        WindowHost { backend, buttons }
    }

    #[test]
    fn layout_and_frame_follow_the_desktop_only_when_buttons_were_captured() {
        assert_eq!(classify_desktop(&signals("ubuntu:GNOME", "ubuntu", "ubuntu", false), false, false), DesktopKind::Gnome);
        assert_eq!(classify_desktop(&signals("KDE", "plasma", "", true), false, true), DesktopKind::Kde);
        assert_eq!(classify_desktop(&signals("Hyprland", "", "", false), true, false), DesktopKind::Other);
        assert_eq!(classify_desktop(&signals("", "", "", false), true, false), DesktopKind::Kde);
        let gnome = parse_gnome_layout(":minimize,maximize,close").unwrap();
        assert_eq!(gnome.right, vec![CaptionButton::Minimize, CaptionButton::Maximize, CaptionButton::Close]);
        assert!(parse_kwin_buttons("[org.kde.kdecoration2]\nTheme=Breeze\n").is_none());
        let kde = parse_kwin_buttons("[org.kde.kdecoration2]\nButtonsOnLeft=\nButtonsOnRight=IAX\n").unwrap();
        assert_eq!(kde.right, gnome.right);
        let wayland = host(WindowBackend::Wayland, parse_gnome_layout(":close"));
        let integrated = plan(&wayland, true);
        assert!(integrated.hide_system_frame());
        assert_eq!(integrated.buttons().right, vec![CaptionButton::Close]);
        assert!(!plan(&wayland, false).draws_controls());
        assert!(!plan(&host(WindowBackend::Other, None), true).hide_system_frame());
        let live = LiveWindow::default();
        assert_eq!(caption_command(CaptionButton::Maximize, &live), Some(egui::ViewportCommand::Maximized(true)));
        assert_eq!(caption_command(CaptionButton::Maximize, &LiveWindow { maximized: true, ..live }), Some(egui::ViewportCommand::Maximized(false)));
        assert_eq!(caption_command(CaptionButton::Maximize, &LiveWindow { fullscreen: true, ..live }), Some(egui::ViewportCommand::Fullscreen(false)));
    }

    #[test]
    fn caption_bytes_round_trip() {
        let pixel = [255u8, 255, 255, 255];
        let mut tiles = Vec::new();
        for role in 0..4u8 {
            for state in 0..3u8 {
                tiles.push((role, state, 1u16, 1u16, pixel.as_slice()));
            }
        }
        let bytes = GnomeCaptions::encode(24, 16, 3, &tiles);
        let decoded = GnomeCaptions::decode(&bytes).expect("round trip");
        assert_eq!(decoded.rasters[GnomeCaptions::index(CaptionButton::Close, false, 2)].rgba, pixel);
        assert!(GnomeCaptions::decode(&bytes[..bytes.len() - 1]).is_none());
    }
}
