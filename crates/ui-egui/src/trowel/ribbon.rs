//! The ribbon: one row of tiles in seven groups (design 2.1 §3), Find a Command at the right.
//! Each tile runs a command or picks a tool; what a later job brings is dimmed.

use serde_json::json;

use super::*;

/// Inside the ribbon: its padding, a group's caption and padding, a tile and its icon well.
const PAD: f32 = 8.0;
const CAPTION: f32 = 14.0;
const GROUP_PAD: f32 = 9.0;
const TILE_W: f32 = 52.0;
const GAP: f32 = 6.0;
const WELL: f32 = 44.0;
const FIND: egui::Vec2 = vec2(150.0, 32.0);

/// What a tile does.
#[derive(Clone, Copy)]
enum Act {
    /// Run a menu command.
    Cmd(&'static str),
    /// Pick a tool.
    Tool(&'static str),
    /// The underlay: show or hide the template layers, or place a scan as one.
    Underlay,
    /// Toggle grid and open the Grid Color adjustment popup immediately.
    Grid,
    /// Run an engine command with parameters, or open a Trowel dialog.
    Trowel(&'static str),
    /// Comes with a later job.
    Soon(&'static str),
}

struct Tile {
    key: &'static str,
    icon: &'static str,
    label: &'static str,
    act: Act,
}

/// How a tile looks: at rest, its tool in hand, its view on, Save with changes to save, or off.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    Rest,
    On,
    Shown,
    Save,
    Off,
}

fn tile(key: &'static str, icon: &'static str, label: &'static str, act: Act) -> Tile {
    Tile { key, icon, label, act }
}

/// The groups and their tiles, left to right.
fn groups() -> [(&'static str, Vec<Tile>); 7] {
    use Act::*;
    [
        (
            tw!("File"),
            vec![
                tile("open", "tw-open", tw!("Open"), Cmd("file.open")),
                tile("save", "tw-save", tw!("Save"), Cmd("file.save")),
                tile("import", "tw-import", tw!("Import"), Cmd("file.place")),
            ],
        ),
        (
            tw!("New Drawing"),
            vec![
                tile("plan", "tw-plan", tw!("Plan"), Cmd("trowel.newPlan")),
                tile("section", "tw-section", tw!("Section"), Cmd("trowel.newSection")),
                tile("finds", "tw-finds", tw!("Finds"), Cmd("trowel.newFinds")),
                tile("sheet", "tw-sheet", tw!("Plate"), Cmd("trowel.newPlate")),
            ],
        ),
        (tw!("Underlay"), vec![tile("align", "tw-align", tw!("Align"), Soon("J008")), tile("underlay", "tw-underlay", tw!("Underlay"), Underlay)]),
        (
            tw!("Draw"),
            vec![
                tile("pen", "tw-pen", tw!("Pen"), Tool("pen")),
                tile("pencil", "tw-pencil", tw!("Pencil"), Tool("pencil")),
                tile("section-line", "tw-section-line", tw!("Section Line"), Trowel("section-line")),
                tile("spot-height", "tw-spot-height", tw!("Spot Height"), Trowel("spot-height")),
                tile("bucket", "tw-bucket", tw!("Bucket"), Tool("livePaintBucket")),
            ],
        ),
        (
            tw!("Drawing Elements"),
            vec![
                tile("north", "tw-north", tw!("North Arrow"), Trowel("north")),
                tile("scale-bar", "tw-scale-bar", tw!("Scale Bar"), Trowel("scale-bar")),
                tile("legend", "tw-legend", tw!("Legend Table"), Trowel("legend")),
                tile("title-block", "tw-title-block", tw!("Title Block"), Trowel("title-block")),
            ],
        ),
        (tw!("View"), vec![tile("grid", "tw-grid", tw!("Grid"), Grid), tile("magnet", "tw-magnet", tw!("Snap"), Cmd("view.snapToPoint"))]),
        (
            tw!("Export"),
            vec![
                tile("pdf", "tw-pdf", tw!("PDF"), Cmd("file.export.pdf")),
                tile("dxf", "tw-dxf", tw!("DXF"), Cmd("ui.dxfOptionsDialog")),
                tile("strata", "tw-strata", tw!("To Strata"), Trowel("strata")),
            ],
        ),
    ]
}

fn look(app: &VectorcraftApp, act: Act) -> Look {
    match act {
        Act::Soon(_) => Look::Off,
        Act::Underlay => match templates(app) {
            (ids, true) if !ids.is_empty() => Look::Shown,
            (ids, _) if !ids.is_empty() => Look::Rest,
            _ => look(app, Act::Cmd("file.place")),
        },
        Act::Grid => {
            if app.ui.view.grid {
                Look::Shown
            } else {
                Look::Rest
            }
        }
        Act::Trowel(_) => {
            if app.session.active().is_some() {
                Look::Rest
            } else {
                Look::Off
            }
        }
        Act::Tool(id) if app.session.tool_id() == id => Look::On,
        Act::Tool(_) => Look::Rest,
        Act::Cmd(id) if !crate::menus::enabled(app, id) => Look::Off,
        Act::Cmd("file.save") if app.session.active().is_some_and(|d| d.is_dirty()) => Look::Save,
        Act::Cmd(id) if crate::menus::checked(app, id, &json!({})) == Some(true) => Look::Shown,
        Act::Cmd(_) => Look::Rest,
    }
}

/// How the tiles are laid out: their width and the gap between them.
#[derive(Clone, Copy)]
struct Fit {
    tile_w: f32,
    gap: f32,
}

/// As in the mockup.
const FULL: Fit = Fit { tile_w: TILE_W, gap: GAP };
/// For a narrower window: tiles as wide as their wells.
const NARROW: Fit = Fit { tile_w: WELL, gap: 4.0 };
/// Find a Command folded to its magnifier, for a narrower window.
const FIND_FOLDED: f32 = 32.0;

/// How wide the groups are, laid out with `fit` (the ribbon's padding at the left included).
fn tiles_width(groups: &[(&str, Vec<Tile>)], fit: Fit) -> f32 {
    PAD + groups
        .iter()
        .map(|(_, tiles)| {
            let n = tiles.len() as f32;
            GROUP_PAD * 2.0 + n * fit.tile_w + (n - 1.0).max(0.0) * fit.gap + 1.0
        })
        .sum::<f32>()
}

fn grid_popup_id() -> egui::Id {
    wid("grid-color-popup-open", ())
}

fn trowel_popup_id() -> egui::Id {
    wid("trowel-tile-popup-open", ())
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(RIBBON).exact_size(RIBBON_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let groups = groups();
        // The tiles as in the mockup with Find a Command beside them while they fit; then the
        // tiles narrow, then Find a Command folds to its magnifier, then the tiles scroll. Each
        // narrower window gives up more, so Find a Command doesn't fold and open again.
        let room = r.width();
        let (fit, find_w) = if tiles_width(&groups, FULL) + PAD + FIND.x + PAD <= room {
            (FULL, FIND.x)
        } else if tiles_width(&groups, NARROW) + PAD + FIND.x + PAD <= room {
            (NARROW, FIND.x)
        } else {
            (NARROW, FIND_FOLDED)
        };
        let well_top = r.top() + PAD + CAPTION + 5.0;
        let find = Rect::from_center_size(pos2(r.right() - PAD - find_w / 2.0, well_top + WELL / 2.0), vec2(find_w, FIND.y));
        let view = Rect::from_min_max(r.min, pos2(find.left() - PAD, r.bottom() - 1.0));
        let width = tiles_width(&groups, fit);
        let mut run = None;
        if width <= view.width() {
            run = tiles(app, ui, &groups, fit, r.left(), r.top(), r.bottom(), &t);
        } else {
            let mut strip = ui.new_child(egui::UiBuilder::new().max_rect(view).layout(egui::Layout::left_to_right(egui::Align::Min)));
            strip.set_clip_rect(view);
            // A mouse's wheel turns it too.
            strip.style_mut().always_scroll_the_only_direction = true;
            egui::ScrollArea::horizontal().id_salt(wid("ribbon-scroll", ())).auto_shrink([false, false]).show(&mut strip, |ui| {
                let (_, content) = ui.allocate_space(vec2(width, view.height()));
                run = tiles(app, ui, &groups, fit, content.left(), content.top(), r.bottom(), &t);
            });
        }
        find_button(app, ui, find, find_w < FIND.x, &t);
        match run {
            Some(Act::Cmd(id)) => crate::menus::invoke(app, id, json!({})),
            Some(Act::Tool(id)) => app.select_tool(id),
            Some(Act::Underlay) => underlay(app),
            Some(Act::Grid) => {
                crate::menus::invoke(app, "view.grid", json!({}));
                let open = ui.ctx().data(|d| d.get_temp::<bool>(grid_popup_id()).unwrap_or(false));
                ui.ctx().data_mut(|d| d.insert_temp(grid_popup_id(), !open || app.ui.view.grid));
            }
            Some(Act::Trowel(key)) => {
                if matches!(key, "section-line" | "spot-height" | "north" | "scale-bar" | "legend") {
                    let cur = ui.ctx().data(|d| d.get_temp::<&'static str>(trowel_popup_id()));
                    let next = if cur == Some(key) { None } else { Some(key) };
                    ui.ctx().data_mut(|d| {
                        if let Some(k) = next {
                            d.insert_temp(trowel_popup_id(), k);
                        } else {
                            d.remove::<&'static str>(trowel_popup_id());
                        }
                    });
                } else {
                    match key {
                        "title-block" => super::plate::toggle_dialog(ui.ctx()),
                        "strata" => super::gpkg_browser::toggle_dialog(ui.ctx()),
                        _ => {}
                    }
                }
            }
            Some(Act::Soon(_)) | None => {}
        }
        show_grid_color_popup(app, ui.ctx(), r.bottom(), &t);
        show_trowel_tile_popup(app, ui.ctx(), r.bottom(), &t);
    });
}

/// The drawing's underlays, template layers holding something (a scan placed as a template: locked
/// and dimmed; the plan legend's Underlay row, empty, is not one yet), and whether any of them
/// shows.
fn templates(app: &VectorcraftApp) -> (Vec<u64>, bool) {
    let Some(st) = app.session.active() else { return (vec![], false) };
    let layers: Vec<_> = st.doc.layers.iter().filter(|l| l.is_template() && l.children().is_some_and(|c| !c.is_empty())).collect();
    (layers.iter().map(|l| l.id.0).collect(), layers.iter().any(|l| l.visible))
}

/// The Underlay tile: with underlays ([`templates`]), hide them all or show them all; else place a
/// scan as one (File › Place with Template ticked).
fn underlay(app: &mut VectorcraftApp) {
    let (ids, shown) = templates(app);
    if ids.is_empty() {
        // Under every line, its layer named in the user's words (`file.place`'s `underlay`).
        crate::menus::invoke(app, "file.place", json!({ "template": true, "underlay": tw!("Underlay") }));
    } else if let Err(e) = app.run("layer.setProps", json!({ "ids": ids, "visible": !shown })) {
        app.status(e);
    }
}

/// The groups from `left`, their captions at `top`, the tiles down to `bottom`, a line after each
/// group. Returns what a clicked tile does.
#[allow(clippy::too_many_arguments)]
fn tiles(app: &VectorcraftApp, ui: &Ui, groups: &[(&str, Vec<Tile>)], fit: Fit, left: f32, top: f32, bottom: f32, t: &Tokens) -> Option<Act> {
    let well_top = top + PAD + CAPTION + 5.0;
    let mut run = None;
    let mut x = left + PAD;
    for (caption, tiles) in groups {
        text(ui, pos2(x + GROUP_PAD, top + PAD + CAPTION / 2.0), Align2::LEFT_CENTER, *caption, bold(11.0), t.text_dim);
        let mut tx = x + GROUP_PAD;
        for tl in tiles {
            let rect = Rect::from_min_max(pos2(tx, well_top), pos2(tx + fit.tile_w, bottom - 6.0));
            if let Some(act) = tile_ui(app, ui, tl, rect, t) {
                run = Some(act);
            }
            tx += fit.tile_w + fit.gap;
        }
        x = tx - fit.gap + GROUP_PAD;
        ui.painter().vline(x + 0.5, (top + PAD)..=(bottom - 6.0), Stroke::new(1.0, t.border));
        x += 1.0;
    }
    run
}

/// Find a Command at `find` (`folded`: its magnifier alone): it opens the command palette.
fn find_button(app: &mut VectorcraftApp, ui: &Ui, find: Rect, folded: bool, t: &Tokens) {
    let resp = ui.interact(find, wid("find", ()), Sense::click());
    let fill = if resp.hovered() { t.panel } else { t.input };
    field_box(ui, find, fill, t.input_border);
    if folded {
        crate::icons::paint(ui, "search", Rect::from_center_size(find.center(), vec2(13.0, 13.0)), t.text_dim);
    } else {
        crate::icons::paint(ui, "search", Rect::from_center_size(find.left_center() + vec2(16.0, 0.0), vec2(13.0, 13.0)), t.text_disabled);
        text(ui, find.left_center() + vec2(29.0, 0.0), Align2::LEFT_CENTER, tw!("Find a command"), font(12.0), t.text_disabled);
    }
    let label = if folded { tw!("Find a command") } else { tl!("Search commands and tools") };
    let tip = match crate::menus::shortcut_of("help.commandPalette") {
        Some(sc) => format!("{label} ({})", crate::menus::pretty_shortcut(sc)),
        None => label.to_string(),
    };
    if resp.on_hover_text(tip).clicked() {
        app.ui.palette_open = true;
        app.ui.palette_query.clear();
    }
}

/// One tile in `rect`: its well, icon and label. Returns what it does when clicked.
fn tile_ui(app: &VectorcraftApp, ui: &Ui, tile: &Tile, rect: Rect, t: &Tokens) -> Option<Act> {
    let look = look(app, tile.act);
    let id = wid("tile", tile.key);
    let resp = ui.interact(rect, id, Sense::click());
    let well = Rect::from_min_size(pos2(rect.center().x - WELL / 2.0, rect.top()), vec2(WELL, WELL));
    let fill = match look {
        Look::On => t.tool_active,
        Look::Shown => t.row_selected,
        Look::Save => t.action,
        Look::Off => t.panel_darker,
        Look::Rest if resp.hovered() => t.hover,
        Look::Rest => t.button,
    };
    let edge = if look == Look::Shown { Stroke::new(1.0, t.ring) } else { Stroke::NONE };
    ui.painter().rect(well, CornerRadius::same(12), fill, edge, StrokeKind::Inside);
    let tint = match look {
        Look::On => t.icon_on,
        Look::Save => t.on_action,
        Look::Off => t.icon.gamma_multiply(0.45),
        Look::Rest | Look::Shown => t.icon,
    };
    crate::icons::paint(ui, tile.icon, Rect::from_center_size(well.center(), vec2(24.0, 24.0)), tint);
    let (label_font, label_color) = match look {
        Look::On | Look::Shown | Look::Save => (bold(11.0), t.text),
        Look::Off => (font(11.0), t.text_dim),
        Look::Rest => (font(11.0), t.text),
    };
    let label = pos2(rect.center().x, well.bottom() + 11.0);
    ui.painter().with_clip_rect(rect.expand2(vec2(3.0, 0.0))).text(label, Align2::CENTER_CENTER, tile.label, label_font, label_color);
    let tip = match tile.act {
        Act::Soon(job) => soon(tile.label, job),
        Act::Underlay if templates(app).0.is_empty() => tw!("Place a scan under the drawing").to_string(),
        Act::Underlay => tw!("Show or hide the underlay").to_string(),
        Act::Grid => format!("{} — 클릭하면 격자 켜기 및 격자 색 조정 창을 엽니다", tile.label),
        Act::Trowel(_) => tile.label.to_string(),
        Act::Tool(id) => vectorcraft_tools::tool_info(id).map(crate::toolbar::tip).unwrap_or_default(),
        Act::Cmd(id) => match crate::menus::shortcut_of(id) {
            Some(sc) => format!("{} ({})", tile.label, crate::menus::pretty_shortcut(sc)),
            None => tile.label.to_string(),
        },
    };
    let resp = resp.on_hover_text(tip);
    match tile.act {
        // Off: it says what it waits for, and does nothing.
        Act::Soon(job) if resp.clicked() => {
            note_soon(ui.ctx(), tile.label, job);
            None
        }
        _ => (look != Look::Off && resp.clicked()).then_some(tile.act),
    }
}

fn show_grid_color_popup(app: &mut VectorcraftApp, ctx: &egui::Context, ribbon_bottom: f32, t: &Tokens) {
    let open = ctx.data(|d| d.get_temp::<bool>(grid_popup_id()).unwrap_or(false));
    if !open {
        return;
    }
    let tile_x = ctx.viewport(|vp| vp.prev_pass.widgets.get(wid("tile", "grid")).map(|w| w.rect.left())).unwrap_or(960.0);
    let mut keep_open = true;
    let resp = egui::Window::new("격자 색 · 간격 조정")
        .id(wid("grid-color-window", ()))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .fixed_pos(pos2(tile_x.max(16.0), ribbon_bottom + 4.0))
        .frame(egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.border)).corner_radius(CornerRadius::same(8)).inner_margin(10.0))
        .show(ctx, |ui| {
            ui.set_width(250.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("격자 색 · 간격 조정").font(bold(12.0)).color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        keep_open = false;
                    }
                    let mut grid_on = app.ui.view.grid;
                    if ui.checkbox(&mut grid_on, "격자 표시").changed() {
                        crate::menus::invoke(app, "view.grid", json!({}));
                    }
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("격자 색상:").font(font(11.0)).color(t.text_dim));
                let hex_str = app.session.prefs.grid_color.trim_start_matches('#');
                let r = hex_str.get(0..2).and_then(|s| u8::from_str_radix(s, 16).ok()).unwrap_or(0xC8);
                let g = hex_str.get(2..4).and_then(|s| u8::from_str_radix(s, 16).ok()).unwrap_or(0xC8);
                let b = hex_str.get(4..6).and_then(|s| u8::from_str_radix(s, 16).ok()).unwrap_or(0xC8);
                let mut rgb = [r, g, b];
                if ui.color_edit_button_srgb(&mut rgb).changed() {
                    let hex = format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                    let _ = app.run("prefs.set", json!({ "key": "gridColor", "value": hex }));
                    app.ui.view.grid = true;
                }
                ui.label(egui::RichText::new(app.session.prefs.grid_color.to_uppercase()).font(mono(11.0)).color(t.text));
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for (label, hex) in [("연회색", "#CCCCCC"), ("황토", "#C8BFA8"), ("하늘", "#6BAED6"), ("방안녹색", "#74C476"), ("진회색", "#777777")]
                {
                    if ui.small_button(label).clicked() {
                        let _ = app.run("prefs.set", json!({ "key": "gridColor", "value": hex }));
                        app.ui.view.grid = true;
                    }
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("간격:").font(font(11.0)).color(t.text_dim));
                for (label, pt_val) in [("10pt", 10.0), ("20pt", 20.0), ("50pt", 50.0), ("72pt", 72.0)] {
                    let selected = (app.session.prefs.gridline_every - pt_val).abs() < 0.5;
                    if ui.selectable_label(selected, label).clicked() {
                        let _ = app.run("prefs.set", json!({ "key": "gridlineEvery", "value": pt_val }));
                        app.ui.view.grid = true;
                    }
                }
            });
        });
    if let Some(inner) = resp {
        let grid_tile_rect = ctx.viewport(|vp| vp.prev_pass.widgets.get(wid("tile", "grid")).map(|w| w.rect));
        let clicked_outside = ctx.input(|i| {
            i.pointer.any_pressed()
                && i.pointer.interact_pos().is_some_and(|p| !inner.response.rect.contains(p) && !grid_tile_rect.is_some_and(|gr| gr.contains(p)))
        });
        // Keep open while color picker sub-popup isn't active if clicked outside
        if clicked_outside && !ctx.any_popup_open() {
            keep_open = false;
        }
    }
    if !keep_open {
        ctx.data_mut(|d| d.insert_temp(grid_popup_id(), false));
    }
}

fn show_trowel_tile_popup(app: &mut VectorcraftApp, ctx: &egui::Context, ribbon_bottom: f32, t: &Tokens) {
    let Some(key) = ctx.data(|d| d.get_temp::<&'static str>(trowel_popup_id())) else {
        return;
    };
    let tile_x = ctx.viewport(|vp| vp.prev_pass.widgets.get(wid("tile", key)).map(|w| w.rect.left())).unwrap_or(520.0);
    let mut keep_open = true;
    let title = match key {
        "scale-bar" => "축척 막대 원클릭 삽입 (1/10 ~ 1/200)",
        "section-line" => "단면 기호 (A–A′ · B–B′ · 붉은 기준점)",
        "spot-height" => "자동 해발선 · 단면 기준 눈금 (┣ 93.0m)",
        "north" => "방위표 삽입 및 각도 회전",
        "legend" => "현장 주기 · 토층설명(7p) · 스탬프",
        _ => "도면 요소",
    };

    let resp = egui::Window::new(title)
        .id(wid("trowel-tile-window", key))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .fixed_pos(pos2(tile_x.max(16.0), ribbon_bottom + 4.0))
        .frame(egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.border)).corner_radius(CornerRadius::same(8)).inner_margin(10.0))
        .show(ctx, |ui| {
            ui.set_width(290.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(title).font(bold(12.0)).color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        keep_open = false;
                    }
                });
            });
            ui.add_space(6.0);

            match key {
                "scale-bar" => {
                    let style_id = wid("scale-bar-style-mode", ());
                    let mut is_parallel = ui.data(|d| d.get_temp::<bool>(style_id).unwrap_or(false));
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("형태:").font(font(11.0)).color(t.text_dim));
                        if ui.selectable_label(!is_parallel, "보고서 막대형 (0—0.5—1m)").clicked() {
                            is_parallel = false;
                        }
                        if ui.selectable_label(is_parallel, "평행 눈금선형").clicked() {
                            is_parallel = true;
                        }
                    });
                    ui.data_mut(|d| d.insert_temp(style_id, is_parallel));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("축척 선택 (클릭 시 캔버스에 즉시 삽입):").font(font(11.0)).color(t.text_dim));
                    let style_str = if is_parallel { "parallel" } else { "report" };
                    ui.horizontal_wrapped(|ui| {
                        for sc in [10u32, 20, 30, 40, 50, 60, 80, 100, 200] {
                            if ui.button(format!("1/{sc}")).clicked() {
                                let at = super::place_symbol_at(app, ctx, [0.0, 90.0]);
                                let _ = app.run("doc.setScale", json!({ "scale": f64::from(sc), "epsg": 5186 }));
                                let _ = app.run("trowel.insertScaleBar", json!({ "scale": sc, "style": style_str, "at": at }));
                            }
                        }
                    });
                }
                "section-line" => {
                    ui.label(egui::RichText::new("붉은 기준점(■) + 시선 화살표(∠) 단면 기호:").font(font(11.0)).color(t.text_dim));
                    ui.horizontal_wrapped(|ui| {
                        for (lbl, orient, desc) in [
                            ("A", "horizontal", "A–A′ (가로)"),
                            ("B", "vertical", "B–B′ (세로)"),
                            ("C", "horizontal", "C–C′ (가로)"),
                            ("a", "horizontal", "a–a′ (가로)"),
                            ("b", "vertical", "b–b′ (세로)"),
                        ] {
                            if ui.button(desc).clicked() {
                                let at = super::place_symbol_at(app, ctx, [0.0, 0.0]);
                                let _ =
                                    app.run("trowel.insertSectionMarker", json!({ "label": lbl, "orientation": orient, "kind": "cut", "at": at }));
                            }
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("단면 양끝 해발 기준 붉은 눈금:").font(font(11.0)).color(t.text_dim));
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("──┃ 93.0m (가로단면)").clicked() {
                            let at = super::place_symbol_at(app, ctx, [0.0, 0.0]);
                            let _ = app.run(
                                "trowel.insertSectionMarker",
                                json!({ "kind": "datum", "orientation": "horizontal", "elevationM": 93.0, "at": at }),
                            );
                        }
                        if ui.button("┳ 93.0m (세로단면)").clicked() {
                            let at = super::place_symbol_at(app, ctx, [0.0, 0.0]);
                            let _ = app.run(
                                "trowel.insertSectionMarker",
                                json!({ "kind": "datum", "orientation": "vertical", "elevationM": 93.0, "at": at }),
                            );
                        }
                    });
                }
                "spot-height" => {
                    let elev_id = wid("spot-datum-elev-val", ());
                    let mut elev_m = ui.data(|d| d.get_temp::<f64>(elev_id).unwrap_or(93.0));
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("기준 해발(m):").font(font(11.0)).color(t.text_dim));
                        ui.add(egui::DragValue::new(&mut elev_m).speed(0.1).range(-100.0..=2000.0).suffix(" m"));
                    });
                    ui.data_mut(|d| d.insert_temp(elev_id, elev_m));
                    ui.add_space(4.0);
                    if ui.button("자동 해발선: 단면선 아래쪽 + 우측 붉은 눈금 (──┃)").clicked() {
                        app.select_tool("selection");
                        let _ = app.run(
                            "elevation.create",
                            json!({
                                "datumM": elev_m,
                                "minorWeightPt": 0.3,
                                "majorWeightPt": 0.5,
                                "labelFontSizePt": 6.0,
                                "labelPlacement": "right",
                                "fillBelowProfile": true,
                                "redDatumTick": true,
                            }),
                        );
                    }
                    if ui.button("자동 해발선: 빈 곳(위쪽) + 좌측 숫자 (기본)").clicked() {
                        app.select_tool("selection");
                        let _ = app.run(
                            "elevation.create",
                            json!({
                                "datumM": elev_m,
                                "minorWeightPt": 0.2,
                                "majorWeightPt": 0.5,
                                "labelPlacement": "left",
                                "fillBelowProfile": false,
                            }),
                        );
                    }
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(format!("붉은 눈금 ──┃ {elev_m:.1}m 찍기")).clicked() {
                            let at = super::place_symbol_at(app, ctx, [0.0, 0.0]);
                            let _ = app.run(
                                "trowel.insertSectionMarker",
                                json!({ "kind": "datum", "orientation": "horizontal", "elevationM": elev_m, "at": at }),
                            );
                        }
                        if ui.button(format!("세로 눈금 ┳ {elev_m:.1}m 찍기")).clicked() {
                            let at = super::place_symbol_at(app, ctx, [0.0, 0.0]);
                            let _ = app.run(
                                "trowel.insertSectionMarker",
                                json!({ "kind": "datum", "orientation": "vertical", "elevationM": elev_m, "at": at }),
                            );
                        }
                    });
                }
                "north" => {
                    let deg_id = wid("north-arrow-deg", ());
                    let mut deg = ui.data(|d| d.get_temp::<f64>(deg_id).unwrap_or(0.0));
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("회전 각도:").font(font(11.0)).color(t.text_dim));
                        ui.add(egui::DragValue::new(&mut deg).speed(1.0).range(-180.0..=180.0).suffix("°"));
                        if ui.button("방위표 넣기").clicked() {
                            let at = super::place_symbol_at(app, ctx, [90.0, -70.0]);
                            let _ = app.run("trowel.insertNorthArrow", json!({ "rotationDeg": deg, "at": at }));
                        }
                    });
                    ui.data_mut(|d| d.insert_temp(deg_id, deg));
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for a in [-45.0, -30.0, -15.0, 0.0, 15.0, 30.0, 45.0, 90.0] {
                            if ui.small_button(format!("{a:+.0}°")).clicked() {
                                ui.data_mut(|d| d.insert_temp(deg_id, a));
                                let at = super::place_symbol_at(app, ctx, [90.0, -70.0]);
                                let _ = app.run("trowel.insertNorthArrow", json!({ "rotationDeg": a, "at": at }));
                            }
                        }
                    });
                }
                "legend" => {
                    ui.label(egui::RichText::new("현장 도면 자주 쓰는 스탬프 · 토층설명:").font(font(11.0)).color(t.text_dim));
                    if ui.button("토층설명 7p 텍스트 넣기 (I : 흑갈색 사질점토…)").clicked() {
                        let at = super::place_symbol_at(app, ctx, [-80.0, 60.0]);
                        let _ = app.run("stratigraphy.generateTable", json!({ "at": at, "format": "inline", "fontSizePt": 7.0 }));
                    }
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("교란 (6p)").clicked() {
                            let at = super::place_symbol_at(app, ctx, [0.0, 20.0]);
                            let _ = app.run("trowel.insertFieldStamp", json!({ "kind": "disturbance", "at": at }));
                        }
                        if ui.button("조사경계 (6p+파선)").clicked() {
                            let at = super::place_symbol_at(app, ctx, [0.0, 20.0]);
                            let _ = app.run("trowel.insertFieldStamp", json!({ "kind": "excavation_limit", "at": at }));
                        }
                        if ui.button("격자 기호 (A1~C3)").clicked() {
                            let at = super::place_symbol_at(app, ctx, [0.0, 20.0]);
                            let _ = app.run("trowel.insertFieldStamp", json!({ "kind": "grid_labels", "at": at }));
                        }
                        if ui.button("현장 선굵기 규격 주기").clicked() {
                            let at = super::place_symbol_at(app, ctx, [0.0, 20.0]);
                            let _ = app.run("trowel.insertFieldStamp", json!({ "kind": "spec_note", "at": at }));
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("토층 번호 로마숫자(I~XX) 찍기:").font(font(11.0)).color(t.text_dim));
                    ui.horizontal_wrapped(|ui| {
                        for n in 1..=10u32 {
                            let rom = vectorcraft_doc::stratigraphy::to_ascii_roman(n);
                            if ui.small_button(&rom).clicked() {
                                let at = super::place_symbol_at(app, ctx, [0.0, 0.0]);
                                let _ = app
                                    .run("stratigraphy.circledNumber", json!({ "at": at, "number": n, "style": "roman_ascii", "fontSizePt": 6.0 }));
                            }
                        }
                    });
                }
                _ => {}
            }
        });

    if let Some(inner) = resp {
        let tile_rect = ctx.viewport(|vp| vp.prev_pass.widgets.get(wid("tile", key)).map(|w| w.rect));
        let clicked_outside = ctx.input(|i| {
            i.pointer.any_pressed()
                && i.pointer.interact_pos().is_some_and(|p| !inner.response.rect.contains(p) && !tile_rect.is_some_and(|tr| tr.contains(p)))
        });
        if clicked_outside && !ctx.any_popup_open() {
            keep_open = false;
        }
    }
    if !keep_open {
        ctx.data_mut(|d| d.remove::<&'static str>(trowel_popup_id()));
    }
}
