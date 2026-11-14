use super::tabs::{open_on_double_click, paint_dots};
use super::{LienzoApp, Rect, Sense};

pub(super) fn start_screen(app: &mut LienzoApp, ui: &mut egui::Ui) {
    let area = ui.available_rect_before_wrap();
    if let Some(look) = crate::theme::Tokens::get(ui.ctx()).creator() {
        // Nodalia: the creator suite's Home, the same screen as Pluma's.
        creator_home(app, ui, &look, area);
        return;
    }
    paint_dots(ui, area, app.session.prefs().interface.show_canvas_dots);
    open_on_double_click(app, ui, area);

    // File › Open Recent, newest first (on the web there are no paths to reopen).
    let show_recent = app.session.prefs().interface.show_home_recent_files;
    let recent: Vec<String> =
        if cfg!(target_arch = "wasm32") || !show_recent { Vec::new() } else { app.ui.recent_files.iter().take(HOME_RECENT).cloned().collect() };

    let is_narrow = area.width() < 640.0;
    let pad_x = if is_narrow { 16.0 } else { 36.0 };
    let pad_y = if is_narrow { 24.0 } else { 54.0 };
    let rail_w = 170.0;

    if is_narrow {
        let card_w = (area.width() - 32.0).clamp(0.0, 520.0);
        let card = Rect::from_center_size(area.center(), egui::vec2(card_w, (area.height() - 40.0).max(0.0)));
        ui.scope_builder(egui::UiBuilder::new().max_rect(card), |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                render_start_actions_row(app, ui, card.width());
                ui.add_space(16.0);
                render_start_content(app, ui, &recent, card.width());
            });
        });
    } else {
        let rail_rect = Rect::from_min_size(egui::pos2(area.left() + pad_x, area.top() + pad_y), egui::vec2(rail_w, area.height() - pad_y * 2.0));
        let content_x = rail_rect.right() + 36.0;
        let content_rect = Rect::from_min_max(egui::pos2(content_x, area.top() + pad_y), egui::pos2(area.right() - pad_x, area.bottom() - 16.0));

        // 1. Left Navigation Rail
        ui.scope_builder(egui::UiBuilder::new().max_rect(rail_rect), |ui| {
            render_start_rail(app, ui, rail_rect.width());
        });

        // 2. Right Content Area (scrollable)
        ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                render_start_content(app, ui, &recent, content_rect.width());
            });
        });
    }
}

fn render_start_actions_row(app: &mut LienzoApp, ui: &mut egui::Ui, avail_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let btn_w = ((avail_w - 12.0) / 2.0).clamp(0.0, 150.0);

    ui.horizontal(|ui| {
        // [ New file ]
        let new_resp = new_file_button(ui, &t, btn_w);
        if new_resp.clicked() {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        }

        ui.add_space(8.0);

        // [ Open… ]
        let open_resp = open_button(ui, &t, btn_w, 32.0);
        if open_resp.clicked() {
            let _ = app.open_dialog_file();
        }
    });
}

fn render_start_rail(app: &mut LienzoApp, ui: &mut egui::Ui, rail_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());

    ui.add_space(8.0);

    // 1. [ New file ] button
    let new_resp = new_file_button(ui, &t, rail_w.min(130.0));
    if new_resp.clicked() {
        let fields = app.new_document_fields();
        app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
    }

    ui.add_space(8.0);

    // 2. [ Open… ] button
    let open_resp = open_button(ui, &t, rail_w.min(130.0), 30.0);
    if open_resp.clicked() {
        let _ = app.open_dialog_file();
    }

    ui.add_space(16.0);

    // Separator line
    let sep_w = 150.0f32.min(rail_w);
    let (sep_rect, _) = ui.allocate_exact_size(egui::vec2(sep_w, 1.0), Sense::hover());
    ui.painter().line_segment([sep_rect.min, egui::pos2(sep_rect.max.x, sep_rect.min.y)], egui::Stroke::new(1.0, t.separator));

    ui.add_space(14.0);

    // 3. "Home" Nav Item (Selected)
    let look = t.creator();
    let (home_rect, home_resp) = match &look {
        // Nodalia: the creator suite's Home rail, the same code as Pluma's.
        Some(look) => {
            let r = nodalia_theme::creator::home_row(ui, look, "home", tl!("Home"), true);
            (r.rect, r)
        }
        None => ui.allocate_exact_size(egui::vec2(rail_w, 32.0), Sense::click()),
    };
    home_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Home")));
    // Home hides the tab strip and options bar, so this entry is the way back to the open
    // documents (with none open there is nothing behind Home).
    if app.session.documents().is_empty() {
        // Nothing to go back to.
    } else if home_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        app.ui.chrome.home = None;
    }
    // The other themes keep a thin accent bar beside the active row.
    if look.is_none() {
        let active_bar = Rect::from_min_size(egui::pos2(home_rect.left(), home_rect.center().y - 7.0), egui::vec2(1.5, 14.0));
        ui.painter().rect_filled(active_bar, 1.0, t.accent);
        let house_box = Rect::from_center_size(egui::pos2(home_rect.left() + 20.0, home_rect.center().y), egui::vec2(16.0, 16.0));
        crate::icons::paint(ui, house_box, "house", 16.0, t.text);
        ui.painter().text(
            egui::pos2(home_rect.left() + 34.0, home_rect.center().y),
            egui::Align2::LEFT_CENTER,
            tl!("Home"),
            crate::theme::semibold(13.5),
            t.text,
        );
    }

    ui.add_space(4.0);

    // 4. "Learn" Nav Item
    if let Some(look) = &look {
        let learn = nodalia_theme::creator::home_row(ui, look, "sparkle", tl!("Learn"), false);
        learn.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Learn")));
        if learn.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            crate::links::open(app, ui.ctx(), crate::links::TUTORIALS);
        }
    } else {
        let (learn_rect, learn_resp) = ui.allocate_exact_size(egui::vec2(rail_w, 32.0), Sense::click());
        let learn_resp = learn_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
        learn_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Learn")));
        if learn_resp.hovered() {
            ui.painter().rect_filled(learn_rect, 4.0, t.hover);
        }
        let learn_color = if learn_resp.hovered() { t.text } else { t.text_dim };
        let sparkle_box = Rect::from_center_size(egui::pos2(learn_rect.left() + 20.0, learn_rect.center().y), egui::vec2(16.0, 16.0));
        crate::icons::paint(ui, sparkle_box, "sparkles", 16.0, learn_color);
        ui.painter().text(
            egui::pos2(learn_rect.left() + 34.0, learn_rect.center().y),
            egui::Align2::LEFT_CENTER,
            tl!("Learn"),
            crate::theme::semibold(13.5),
            learn_color,
        );
        if learn_resp.clicked() {
            crate::links::open(app, ui.ctx(), crate::links::TUTORIALS);
        }
    }

    // 5. Bottom pinned community links
    let bottom_h = 100.0;
    let avail_h = ui.available_height();
    let spacer = (avail_h - bottom_h - 20.0).max(24.0);
    ui.add_space(spacer);

    let (b_sep_rect, _) = ui.allocate_exact_size(egui::vec2(sep_w, 1.0), Sense::hover());
    ui.painter().line_segment([b_sep_rect.min, egui::pos2(b_sep_rect.max.x, b_sep_rect.min.y)], egui::Stroke::new(1.0, t.separator));

    ui.add_space(8.0);

    render_rail_link(app, ui, rail_w, tl!("Website"), "link", crate::links::APP_PAGE);
    render_rail_link(app, ui, rail_w, tl!("GitHub"), "link-2", crate::links::GITHUB);
}

fn render_rail_link(app: &mut LienzoApp, ui: &mut egui::Ui, rail_w: f32, label: &str, icon_name: &str, url: &str) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(rail_w, 28.0), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(url);
    if resp.hovered() {
        ui.painter().rect_filled(rect, 4.0, t.hover);
    }
    let color = if resp.hovered() { t.text } else { t.text_dim };
    let icon_box = Rect::from_center_size(egui::pos2(rect.left() + 20.0, rect.center().y), egui::vec2(14.0, 14.0));
    crate::icons::paint(ui, icon_box, icon_name, 14.0, color);
    ui.painter().text(egui::pos2(rect.left() + 34.0, rect.center().y), egui::Align2::LEFT_CENTER, label, crate::theme::medium(12.0), color);
    if resp.clicked() {
        crate::links::open(app, ui.ctx(), url);
    }
}

fn render_start_content(app: &mut LienzoApp, ui: &mut egui::Ui, recent: &[String], content_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    ui.add_space(8.0);

    // Top row: the app icon and its name
    ui.horizontal(|ui| {
        let (mark_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), Sense::hover());
        crate::panels::brand::paint_mark(ui, mark_rect);

        ui.add_space(10.0);

        let title_font = if t.islands() { crate::theme::display(22.0) } else { crate::theme::semibold(20.0) };
        let title_galley = ui.painter().layout_no_wrap("Lienzo".into(), title_font, t.text);
        let (title_r, _) = ui.allocate_exact_size(title_galley.size(), Sense::hover());
        ui.painter().galley(title_r.min, title_galley, t.text);
    });

    ui.add_space(18.0);

    // Hairline separator under header
    let (sep_r, _) = ui.allocate_exact_size(egui::vec2(content_w, 1.0), Sense::hover());
    ui.painter().line_segment([sep_r.min, egui::pos2(sep_r.max.x, sep_r.min.y)], egui::Stroke::new(1.0, t.separator));

    ui.add_space(24.0);

    if recent.is_empty() {
        render_first_run_content(app, ui, content_w);
    } else {
        crate::home_recent_ui::show_home_recent(app, ui, recent, content_w);
    }

    ui.add_space(24.0);
}

fn render_first_run_content(app: &mut LienzoApp, ui: &mut egui::Ui, content_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());

    let drop_w = 540.0f32.min(content_w - 24.0);
    let drop_h = 200.0;

    // Center vertically in available content area
    let avail_h = ui.available_height();
    let top_space = ((avail_h - drop_h) / 2.0 - 50.0).max(40.0);
    ui.add_space(top_space);

    // Center horizontally across content_w
    let pad_x = ((content_w - drop_w) / 2.0).max(0.0);
    ui.horizontal(|ui| {
        ui.add_space(pad_x);

        let (drop_r, drop_resp) = ui.allocate_exact_size(egui::vec2(drop_w, drop_h), Sense::click());
        let drop_resp = drop_resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tl!("Open…"));
        let is_hov = drop_resp.hovered();

        let border_color = if is_hov { t.text_dim } else { t.separator };
        let bg_color = if is_hov { t.hover } else { egui::Color32::TRANSPARENT };
        ui.painter().rect_filled(drop_r, 8.0, bg_color);
        ui.painter().rect_stroke(drop_r, 8.0, egui::Stroke::new(1.0, border_color), egui::StrokeKind::Inside);

        // Centered icon & message
        let icon_box = Rect::from_center_size(egui::pos2(drop_r.center().x, drop_r.center().y - 18.0), egui::vec2(40.0, 40.0));
        crate::icons::paint(ui, icon_box, "image", 34.0, if is_hov { t.text } else { t.text_dim });

        let drop_msg = start_screen_drop_hint(app);
        ui.painter().text(
            egui::pos2(drop_r.center().x, drop_r.center().y + 20.0),
            egui::Align2::CENTER_CENTER,
            drop_msg,
            crate::theme::medium(14.0),
            if is_hov { t.text } else { t.text_faint },
        );

        if drop_resp.clicked() {
            let _ = app.open_dialog_file();
        }
    });
}

/// Hint displayed on the start screen for opening files.
pub(crate) fn start_screen_drop_hint(_app: &LienzoApp) -> std::borrow::Cow<'static, str> {
    tl!("Drop an image or PSD anywhere to open it.").into()
}

/// Recent files listed on the Home screen.
const HOME_RECENT: usize = 11;

/// The view's main action: a solid pill in the text colour on Nodalia, the blue button elsewhere.
fn paint_new_button(ui: &egui::Ui, t: &crate::theme::Tokens, rect: Rect, label: &str, hovered: bool) {
    let (bg, ink, radius) = if t.islands() {
        let bg = if hovered { t.primary_bg.gamma_multiply(0.88) } else { t.primary_bg };
        (bg, t.primary_text, t.radius)
    } else {
        let bg = if hovered { egui::Color32::from_rgb(41, 131, 245) } else { egui::Color32::from_rgb(26, 115, 232) };
        (bg, egui::Color32::WHITE, 4.0)
    };
    ui.painter().rect_filled(rect, radius, bg);
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, label, crate::theme::semibold(13.0), ink);
}

/// New file: the creator suite's primary button in Nodalia (the same code as Pluma's), the
/// painted pill elsewhere.
fn new_file_button(ui: &mut egui::Ui, t: &crate::theme::Tokens, width: f32) -> egui::Response {
    let label = tl!("New file");
    if let Some(look) = t.creator() {
        return nodalia_theme::creator::primary_button_min(ui, &look, label, true, width).on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 32.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    paint_new_button(ui, t, rect, label, resp.hovered());
    resp
}

/// Open…: the creator suite's secondary button in Nodalia, an outlined button elsewhere.
fn open_button(ui: &mut egui::Ui, t: &crate::theme::Tokens, width: f32, height: f32) -> egui::Response {
    let label = tl!("Open…");
    if let Some(look) = t.creator() {
        return nodalia_theme::creator::secondary_button_min(ui, &look, label, true, width).on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if resp.hovered() {
        ui.painter().rect_filled(rect, 4.0, t.hover);
    }
    ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, label, crate::theme::semibold(13.0), t.text);
    resp
}

/// The creator suite's Home (`nodalia_theme::creator::home`), drawn with the same code as Pluma's:
/// the rail, Lienzo's mark and name, and Recent Files with their thumbnails.
fn creator_home(app: &mut LienzoApp, ui: &mut egui::Ui, look: &nodalia_theme::creator::Look, area: Rect) {
    use crate::home_recent_ui::{RecentAction, act, get_or_load_thumbnail};
    use nodalia_theme::creator::home;
    open_on_double_click(app, ui, area);
    let show_recent = app.session.prefs().interface.show_home_recent_files;
    let paths: Vec<String> =
        if cfg!(target_arch = "wasm32") || !show_recent { Vec::new() } else { app.ui.recent_files.iter().take(HOME_RECENT).cloned().collect() };
    let names: Vec<(String, String)> = paths
        .iter()
        .map(|path| {
            let folder = std::path::Path::new(path).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
            (crate::file_open::display_name(path), folder)
        })
        .collect();
    let links = [
        home::Link { icon: "globe", label: tl!("Website"), url: crate::links::APP_PAGE },
        home::Link { icon: "code", label: tl!("GitHub"), url: crate::links::GITHUB },
    ];
    let rail = home::Rail { new_file: tl!("New file"), open: tl!("Open…"), home: tl!("Home"), learn: Some(tl!("Learn")), links: &links };
    let labels = home::RecentLabels {
        heading: tl!("Recent Files"),
        grid: tl!("Grid view"),
        list: tl!("List view"),
        remove: tl!("Remove from Recent"),
        clear: tl!("Clear Recent Files"),
        no_folder: tl!("On your computer"),
        drop: &start_screen_drop_hint(app),
        drop_icon: "image",
    };
    let mark = |p: &egui::Painter, r: Rect| crate::panels::brand::paint_mark_on(p, r);
    let mut recent_pick = None;
    let picked = home::view(ui, look, area, "Lienzo", &mark, &rail, |ui| {
        let textures: Vec<_> = paths.iter().map(|p| get_or_load_thumbnail(app, ui.ctx(), p)).collect();
        let list: Vec<home::Recent> = paths
            .iter()
            .zip(&names)
            .zip(&textures)
            .map(|((path, (name, folder)), tex)| home::Recent { name, folder, path, thumb: tex.as_ref().map(|t| (t.id(), t.size_vec2())) })
            .collect();
        let mut grid = app.ui.chrome.home_recent_grid;
        recent_pick = home::recents(ui, look, &labels, &list, &mut grid);
        app.ui.chrome.home_recent_grid = grid;
    });
    match picked {
        Some(home::Pick::NewFile) => {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        }
        Some(home::Pick::Open) => {
            let _ = app.open_dialog_file();
        }
        // Home hides the tab strip, so this row is the way back to the open documents.
        Some(home::Pick::Home) if !app.session.documents().is_empty() => app.ui.chrome.home = None,
        Some(home::Pick::Learn) => {
            crate::links::open(app, ui.ctx(), crate::links::TUTORIALS);
        }
        Some(home::Pick::Link(i)) => {
            if let Some(link) = links.get(i) {
                crate::links::open(app, ui.ctx(), link.url);
            }
        }
        _ => {}
    }
    let path = |i: usize| paths.get(i).cloned();
    let action = match recent_pick {
        Some(home::RecentPick::Open(i)) => path(i).map(RecentAction::Open),
        Some(home::RecentPick::Remove(i)) => path(i).map(RecentAction::Remove),
        Some(home::RecentPick::Clear) => Some(RecentAction::Clear),
        Some(home::RecentPick::OpenDialog) => Some(RecentAction::OpenDialog),
        None => None,
    };
    if let Some(action) = action {
        act(app, action);
    }
}
