//! Capture the shell's real caption buttons.
//!
//! GNOME (and the desktops that ship its widgets) snapshots libadwaita's header bar: the active
//! icon theme inside libadwaita's own circular plate. Every other desktop, including KDE,
//! snapshots a plain GTK header bar, so the buttons are whatever theme that session configured
//! (Breeze, when Plasma has written it). The helper is a child process so GTK never shares a
//! display connection with winit. If it cannot run, the desktop frame stays.

#![cfg(feature = "gnome-captions")]

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use photocraft_ui_egui::window_chrome::{CaptionRaster, GnomeCaptions};

/// Render the buttons when this process was started as the helper.
pub fn render_to_stdout() -> Result<(), String> {
    if session_looks_like_gnome() {
        // libadwaita replaces the GTK theme. Only the GNOME family wants that; KDE and the others
        // must keep the theme their session already configured.
        libadwaita::init().map_err(|e| e.to_string())?;
        let window = gtk4::Window::new();
        let bar = libadwaita::HeaderBar::new();
        bar.set_show_back_button(false);
        window.set_titlebar(Some(&bar));
        write_captions(&window, &bar)
    } else {
        gtk4::init().map_err(|e| e.to_string())?;
        let window = gtk4::Window::new();
        let bar = gtk4::HeaderBar::new();
        bar.set_show_title_buttons(true);
        // Always snapshot minimize, maximize and close. The editor still places only the buttons
        // this desktop configured, in that desktop's order.
        bar.set_decoration_layout(Some(":minimize,maximize,close"));
        window.set_titlebar(Some(&bar));
        write_captions(&window, &bar)
    }
}

/// An undecorated titlebar is never allocated, so the buttons stay 0×0. A resizable window is what
/// includes maximize. Fully transparent windows lay out but snapshot empty; 1% still returns the
/// real pixels. The editor window itself stays undecorated.
fn write_captions(window: &gtk4::Window, bar: &impl IsA<gtk4::Widget>) -> Result<(), String> {
    window.set_resizable(true);
    window.set_opacity(0.01);
    window.set_default_size(280, 48);
    window.present();

    let ready = wait_until(bar, Duration::from_secs(2))?;
    let (button_px, image_px, gap, tiles) = capture(window, &ready)?;
    window.close();
    pump(Duration::from_millis(30));
    let borrowed: Vec<(u8, u8, u16, u16, &[u8])> = tiles.iter().map(|(role, state, w, h, px)| (*role, *state, *w, *h, px.as_slice())).collect();
    let bytes = GnomeCaptions::encode(button_px, image_px, gap, &borrowed);
    std::io::stdout().write_all(&bytes).map_err(|e| e.to_string())?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Spawn the helper. `None` leaves the drawn buttons in place.
pub fn capture_for_session() -> Option<GnomeCaptions> {
    let exe = std::env::current_exe().ok()?;
    let mut child = Command::new(exe).arg("--gnome-captions").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    let mut stdout = child.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    let bytes = match rx.recv_timeout(Duration::from_secs(8)) {
        Ok(bytes) => bytes,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
    };
    let status = child.wait().ok()?;
    if !status.success() {
        return None;
    }
    GnomeCaptions::decode(&bytes)
}

fn session_looks_like_gnome() -> bool {
    let blob = [std::env::var("XDG_CURRENT_DESKTOP"), std::env::var("XDG_SESSION_DESKTOP"), std::env::var("DESKTOP_SESSION")]
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
        .join(":");
    uses_libadwaita(&blob)
}

/// GNOME's own widgets. KDE, sway, Hyprland and the rest keep their GTK theme.
fn uses_libadwaita(blob: &str) -> bool {
    let blob = blob.to_ascii_lowercase();
    blob.split([':', ';', ',']).any(|token| matches!(token.trim(), "gnome" | "gnome-classic" | "ubuntu" | "ubuntu-wayland" | "budgie" | "pop"))
}

/// role, state, width, height, straight RGBA.
type CaptionTile = (u8, u8, u16, u16, Vec<u8>);

struct LaidOut {
    controls: gtk4::Widget,
    buttons: Vec<(u8, gtk4::Widget, gtk4::Image)>,
}

fn wait_until(bar: &impl IsA<gtk4::Widget>, limit: Duration) -> Result<LaidOut, String> {
    let start = Instant::now();
    while start.elapsed() < limit {
        pump(Duration::from_millis(20));
        if let Some(laid) = layout(bar) {
            return Ok(laid);
        }
    }
    Err("GNOME caption buttons were not laid out".into())
}

fn layout(root: &impl gtk4::prelude::IsA<gtk4::Widget>) -> Option<LaidOut> {
    let mut controls = None;
    let mut stack = vec![root.clone().upcast::<gtk4::Widget>()];
    while let Some(widget) = stack.pop() {
        let mut child = widget.first_child();
        let mut buttons = Vec::new();
        while let Some(next) = child {
            if let Some(role) = role_of(&next)
                && let Some(child) = next.first_child()
                && let Ok(image) = child.downcast::<gtk4::Image>()
                && next.width() > 0
                && image.width() > 0
            {
                buttons.push((role, next.clone(), image));
            }
            stack.push(next.clone());
            child = next.next_sibling();
        }
        if buttons.len() >= 3 {
            controls = Some((widget, buttons));
        }
    }
    let (controls, mut buttons) = controls?;
    buttons.sort_by_key(|(role, _, _)| *role);
    // The end-side group is the one that contains close. Prefer it over a start-side copy.
    if !buttons.iter().any(|(role, _, _)| *role == 3) {
        return None;
    }
    Some(LaidOut { controls, buttons })
}

fn role_of(widget: &gtk4::Widget) -> Option<u8> {
    widget.css_classes().iter().find_map(|class| match class.as_str() {
        "minimize" => Some(0),
        "maximize" => Some(1),
        "close" => Some(3),
        _ => None,
    })
}

fn capture(window: &gtk4::Window, laid: &LaidOut) -> Result<(u16, u16, u16, Vec<CaptionTile>), String> {
    // `width()` is the content box. The circular plate is the image's border box, and the click
    // target is the button's border box (content plus the stylesheet padding).
    let button_px = laid.buttons.iter().map(|(_, button, _)| border_box(button).0).max().unwrap_or(24).max(1) as u16;
    let image_px = laid.buttons.iter().map(|(_, _, image)| border_box(image).0).max().unwrap_or(16).max(1) as u16;
    let gap = gap_between(&laid.controls, &laid.buttons);
    let mut tiles: Vec<CaptionTile> = Vec::new();
    for (role, button, image) in &laid.buttons {
        push_states(&mut tiles, *role, window, button, image)?;
    }
    let Some((_, maximize, image)) = laid.buttons.iter().find(|(role, _, _)| *role == 1) else {
        return Err("GNOME header has no maximize button".into());
    };
    image.set_icon_name(Some("window-restore-symbolic"));
    pump(Duration::from_millis(40));
    push_states(&mut tiles, 2, window, maximize, image)?;
    Ok((button_px, image_px, gap, tiles))
}

fn push_states(tiles: &mut Vec<CaptionTile>, role: u8, window: &gtk4::Window, button: &gtk4::Widget, image: &gtk4::Image) -> Result<(), String> {
    let rest = grab(window, image).ok_or("could not snapshot a caption button")?;
    button.set_state_flags(gtk4::StateFlags::PRELIGHT, false);
    pump(Duration::from_millis(40));
    let hover = grab(window, image).unwrap_or_else(|| rest.clone());
    button.unset_state_flags(gtk4::StateFlags::PRELIGHT);
    button.set_state_flags(gtk4::StateFlags::ACTIVE, false);
    pump(Duration::from_millis(40));
    let active = grab(window, image).unwrap_or_else(|| hover.clone());
    button.unset_state_flags(gtk4::StateFlags::ACTIVE);
    for (state, raster) in [rest, hover, active].into_iter().enumerate() {
        tiles.push((role, state as u8, raster.width as u16, raster.height as u16, raster.rgba));
    }
    Ok(())
}

fn gap_between(controls: &gtk4::Widget, buttons: &[(u8, gtk4::Widget, gtk4::Image)]) -> u16 {
    let mut placed: Vec<(f32, i32)> = buttons
        .iter()
        .filter_map(|(_, button, _)| {
            let bounds = button.compute_bounds(controls)?;
            Some((bounds.x(), bounds.width().round() as i32))
        })
        .collect();
    placed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let Some((first, rest)) = placed.split_first() else {
        return 3;
    };
    let Some(second) = rest.first() else {
        return 3;
    };
    let gap = second.0 - first.0 - first.1 as f32;
    gap.round().clamp(0.0, 32.0) as u16
}

/// Border box, which includes the padding the circular plate is painted in. `width()` is only the
/// content box, so snapshotting that crops the plate down to the glyph.
fn border_box(widget: &impl gtk4::prelude::IsA<gtk4::Widget>) -> (i32, i32) {
    let widget = widget.as_ref();
    let content = (widget.width().max(1), widget.height().max(1));
    let Some(parent) = widget.parent() else {
        return content;
    };
    let Some(bounds) = widget.compute_bounds(&parent) else {
        return content;
    };
    let width = bounds.width().round() as i32;
    let height = bounds.height().round() as i32;
    if width <= 0 || height <= 0 { content } else { (width, height) }
}

fn grab(window: &gtk4::Window, widget: &impl gtk4::prelude::IsA<gtk4::Widget>) -> Option<CaptionRaster> {
    let widget = widget.as_ref();
    let (logical_w, logical_h) = border_box(widget);
    if logical_w <= 0 || logical_h <= 0 {
        return None;
    }
    let paintable = gtk4::WidgetPaintable::new(Some(widget));
    let snapshot = gtk4::Snapshot::new();
    paintable.snapshot(&snapshot, logical_w as f64, logical_h as f64);
    let node = snapshot.to_node()?;
    let renderer = window.native()?.renderer()?;
    // The node's bounds are the ink. Themes that draw no plate at rest would otherwise return a
    // tiny glyph and the header would scale it. The viewport is the widget box the glyph sits in.
    let viewport = gtk4::graphene::Rect::new(0.0, 0.0, logical_w as f32, logical_h as f32);
    let texture = renderer.render_texture(&node, Some(&viewport));
    let width = texture.width().max(0) as u32;
    let height = texture.height().max(0) as u32;
    if width == 0 || height == 0 {
        return None;
    }
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    texture.download(&mut rgba, width as usize * 4);
    straighten(&mut rgba);
    Some(CaptionRaster { width, height, rgba })
}

/// `gdk_texture_download` is premultiplied. A straight pixel can have a channel greater than its
/// alpha, which premultiplication never produces; only un-multiply in the other case.
fn straighten(rgba: &mut [u8]) {
    let (pixels, _) = rgba.as_chunks::<4>();
    let straight = pixels.iter().any(|px| px[3] < 255 && (px[0] > px[3] || px[1] > px[3] || px[2] > px[3]));
    if straight {
        return;
    }
    let (pixels, _) = rgba.as_chunks_mut::<4>();
    for px in pixels {
        let alpha = px[3] as u32;
        if alpha == 0 || alpha == 255 {
            continue;
        }
        for channel in &mut px[..3] {
            *channel = ((*channel as u32) * 255 / alpha).min(255) as u8;
        }
    }
}

fn pump(duration: Duration) {
    let end = Instant::now() + duration;
    let context = gtk4::glib::MainContext::default();
    while Instant::now() < end {
        let _ = context.iteration(false);
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
mod tests {
    use super::uses_libadwaita;

    #[test]
    fn only_the_gnome_family_snapshots_libadwaita() {
        assert!(uses_libadwaita("gnome"));
        assert!(uses_libadwaita("ubuntu:GNOME"));
        assert!(uses_libadwaita("budgie"));
        assert!(!uses_libadwaita("kde"));
        assert!(!uses_libadwaita("KDE:plasma"));
        assert!(!uses_libadwaita("sway"));
        assert!(!uses_libadwaita("Hyprland"));
        assert!(!uses_libadwaita(""));
    }
}
