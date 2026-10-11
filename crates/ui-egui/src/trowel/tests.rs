//! Trowel's window drawn and clicked in headless egui frames: 1600 × 988 like the mockup
//! (`win-plan`, design 2.1 §3) unless a test says otherwise, with the app's own title bar and one
//! document open.

use egui::{Event, PointerButton, Pos2, Rect, pos2, vec2};
use serde_json::json;
use vectorcraft_doc::NodeId;
use vectorcraft_engine::Session;

use super::*;
use crate::VectorcraftApp;
use crate::theme::Tokens;

/// The ribbon's tiles, left to right.
const TILES: [&str; 23] = [
    "open",
    "save",
    "import",
    "plan",
    "section",
    "finds",
    "sheet",
    "align",
    "underlay",
    "pen",
    "pencil",
    "section-line",
    "spot-height",
    "bucket",
    "north",
    "scale-bar",
    "legend",
    "title-block",
    "grid",
    "magnet",
    "pdf",
    "dxf",
    "strata",
];

/// The tool column's slots, top to bottom, by their first tool.
const SLOTS: [&str; 14] = [
    "selection",
    "directSelection",
    "pen",
    "pencil",
    "lineSegment",
    "rectangle",
    "type",
    "rotate",
    "scale",
    "scissors",
    "livePaintBucket",
    "eyedropper",
    "hand",
    "zoom",
];

/// The archaeology tools under them.
const ARCH: [&str; 5] = ["section-line", "spot-height", "find-number", "posthole", "hachure"];

/// Text painted in the last frame: what it says, where and in what colour.
struct Painted {
    text: String,
    rect: Rect,
    color: egui::Color32,
}

struct Harness {
    app: VectorcraftApp,
    ctx: egui::Context,
    screen: egui::Vec2,
    time: f64,
    texts: Vec<Painted>,
    shapes: Vec<egui::epaint::ClippedShape>,
}

impl Harness {
    /// The first start (Trowel's window) with a document named "Site 1" open, 1600 × 988.
    fn new() -> Self {
        Self::sized(1600.0, 988.0)
    }

    /// The same in a `w` × `h` window.
    fn sized(w: f32, h: f32) -> Self {
        let mut app = VectorcraftApp::new(Session::new(), crate::Services::default());
        app.custom_titlebar = true;
        crate::workspaces::start_trowel(&mut app.ui);
        app.run("file.new", json!({"name": "Site 1", "width": 400, "height": 300})).unwrap();
        let mut h = Harness { app, ctx: egui::Context::default(), screen: vec2(w, h), time: 0.0, texts: vec![], shapes: vec![] };
        // The first frame installs the fonts, the second lays the window out, the third sees it.
        for _ in 0..3 {
            h.frame(vec![]);
        }
        h
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.time += 0.05;
        let raw =
            egui::RawInput { time: Some(self.time), screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.screen)), events, ..Default::default() };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(raw, |ui| {
            app.logic(ui.ctx());
            app.ui(ui);
        });
        out.textures_delta.clear();
        self.texts = out
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) => {
                    let color = t.galley.job.sections.first().map(|s| s.format.color).filter(|c| *c != egui::Color32::PLACEHOLDER);
                    Some(Painted { text: t.galley.text().to_string(), rect: t.visual_bounding_rect(), color: color.unwrap_or(t.fallback_color) })
                }
                _ => None,
            })
            .collect();
        self.shapes = out.shapes;
    }

    /// A few frames with nothing new, so what the last one changed is laid out and seen.
    fn settle(&mut self) {
        for _ in 0..2 {
            self.frame(vec![]);
        }
    }

    /// Where the widget `id` was in the last frame.
    fn rect(&self, id: egui::Id) -> Option<Rect> {
        self.ctx.viewport(|vp| vp.prev_pass.widgets.get(id).map(|w| w.rect))
    }

    fn panel(&self, id: &str) -> Rect {
        egui::PanelState::load(&self.ctx, egui::Id::new(id)).unwrap_or_else(|| panic!("no panel {id}")).outer_rect
    }

    fn press(&mut self, at: Pos2, button: PointerButton) {
        self.frame(vec![Event::PointerMoved(at)]);
        self.frame(vec![Event::PointerButton { pos: at, button, pressed: true, modifiers: Default::default() }]);
        self.frame(vec![Event::PointerButton { pos: at, button, pressed: false, modifiers: Default::default() }]);
        self.settle();
    }

    fn click_at(&mut self, at: Pos2) {
        self.press(at, PointerButton::Primary);
    }

    fn click(&mut self, id: egui::Id) {
        let r = self.rect(id).unwrap_or_else(|| panic!("no widget {id:?}"));
        self.click_at(r.center());
    }

    /// Let `secs` seconds pass, then draw a frame.
    fn later(&mut self, secs: f64) {
        self.time += secs;
        self.settle();
    }

    /// Turn the mouse wheel over `at` by `delta` (the way the content moves), and let the smooth
    /// scrolling run its course.
    fn wheel(&mut self, at: Pos2, delta: egui::Vec2) {
        self.frame(vec![Event::PointerMoved(at)]);
        let modifiers = Default::default();
        self.frame(vec![Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta, phase: egui::TouchPhase::Move, modifiers }]);
        for _ in 0..40 {
            self.frame(vec![]);
        }
    }

    fn hover(&mut self, id: egui::Id) {
        let at = self.rect(id).unwrap_or_else(|| panic!("no widget {id:?}")).center();
        self.frame(vec![Event::PointerMoved(at)]);
        self.settle();
    }

    /// The texts painted inside `area`, left to right.
    fn texts_in(&self, area: Rect) -> Vec<String> {
        let mut v: Vec<&Painted> = self.texts.iter().filter(|p| area.expand(0.5).contains_rect(p.rect)).collect();
        v.sort_by(|a, b| a.rect.left().total_cmp(&b.rect.left()));
        v.into_iter().map(|p| p.text.clone()).collect()
    }

    /// The colour `text` was painted in inside `area`.
    fn color_of(&self, text: &str, area: Rect) -> Option<egui::Color32> {
        self.texts.iter().find(|p| p.text == text && area.expand(0.5).contains_rect(p.rect)).map(|p| p.color)
    }

    /// Everything painted inside `area` as one string, left to right.
    fn line_in(&self, area: Rect) -> String {
        self.texts_in(area).join(" ")
    }

    /// The fill of the rectangle painted exactly at `r` (the last one, topmost).
    fn fill_at(&self, r: Rect) -> Option<egui::Color32> {
        self.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Rect(s) if (s.rect.min - r.min).length() < 0.6 && (s.rect.max - r.max).length() < 0.6 => Some(s.fill),
                _ => None,
            })
            .next_back()
    }

    fn tokens(&self) -> Tokens {
        Tokens::get(&self.ctx)
    }

    fn layer_ids(&self) -> Vec<NodeId> {
        self.app.session.active().unwrap().doc.layers.iter().map(|l| l.id).collect()
    }

    fn current_layer(&self) -> Option<NodeId> {
        self.app.session.active().unwrap().current_layer()
    }

    /// The enabled and disabled clickable widgets of the open popup, top to bottom.
    fn popup_buttons(&self) -> Vec<(Rect, bool)> {
        let mut v: Vec<(Rect, bool)> = self.ctx.viewport(|vp| {
            vp.prev_pass
                .widgets
                .layers()
                .filter(|(l, _)| l.order == egui::Order::Foreground)
                // Not the popup's own area, which takes clicks to come to the front.
                .flat_map(|(l, ws)| ws.iter().filter(move |w| w.id != l.id.with("move")))
                .filter(|w| w.sense.senses_click())
                .map(|w| (w.rect, w.enabled))
                .collect()
        });
        v.sort_by(|a, b| a.0.top().total_cmp(&b.0.top()));
        v
    }
}

fn close(a: Rect, b: Rect) -> bool {
    (a.min - b.min).length() <= 1.0 && (a.max - b.max).length() <= 1.0
}

fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_min_max(pos2(x0, y0), pos2(x1, y1))
}

#[test]
fn regions_match_the_mockup_at_1600_by_988() {
    let mut h = Harness::new();
    let want = [
        (CAPTION, r(0.0, 0.0, 1600.0, 32.0)),
        (MENU, r(0.0, 32.0, 1600.0, 60.0)),
        (RIBBON, r(0.0, 60.0, 1600.0, 156.0)),
        (CONTROL, r(0.0, 156.0, 1600.0, 196.0)),
        (STATUS, r(0.0, 956.0, 1600.0, 988.0)),
        (TOOLS, r(0.0, 196.0, 56.0, 956.0)),
        (SET, r(56.0, 196.0, 272.0, 956.0)),
        (RIGHT, r(1296.0, 196.0, 1600.0, 956.0)),
    ];
    for (id, rect) in want {
        assert!(close(h.panel(id), rect), "{id}: {:?}, the mockup {rect:?}", h.panel(id));
    }
    // The canvas region is 1024 × 728 under the 32-point tabs; the rulers take 16 points of it.
    let canvas = h.app.canvas_rect.unwrap();
    assert!(close(canvas, r(288.0, 244.0, 1296.0, 956.0)), "{canvas:?}");
    // Full screen with menu bar off: no caption, menu, ribbon or Control bar.
    h.app.ui.screen_mode = 2;
    h.settle();
    assert!(close(h.app.canvas_rect.unwrap(), r(288.0, 48.0, 1296.0, 956.0)), "{:?}", h.app.canvas_rect);
    // Full screen: the canvas alone.
    h.app.ui.screen_mode = 3;
    h.settle();
    assert!(close(h.app.canvas_rect.unwrap(), r(0.0, 0.0, 1600.0, 988.0)), "{:?}", h.app.canvas_rect);
}

#[test]
fn caption_shows_the_document_and_unsaved_mark() {
    let mut h = Harness::new();
    let caption = h.panel(CAPTION);
    assert!(h.texts_in(caption).contains(&"Site 1 - Trowel".to_string()), "{:?}", h.texts_in(caption));
    // Trowel's mark at the left, the three window buttons at the right.
    let mark = crate::brand::texture_id(&h.ctx);
    let marks = h
        .shapes
        .iter()
        .filter(|c| matches!(&c.shape, egui::Shape::Mesh(m) if m.texture_id == mark && caption.contains_rect(m.calc_bounds())))
        .count();
    assert_eq!(marks, 1, "the mark in the caption");
    let buttons = h.ctx.viewport(|vp| {
        vp.prev_pass
            .widgets
            .layers()
            .flat_map(|(_, w)| w.iter())
            .filter(|w| w.rect.top() == 0.0 && w.rect.width() == crate::titlebar::BUTTON_WIDTH)
            .count()
    });
    assert_eq!(buttons, 3, "minimize, maximize, close");
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    h.settle();
    assert!(h.texts_in(caption).contains(&"Site 1 * - Trowel".to_string()), "{:?}", h.texts_in(caption));
}

#[test]
fn drawing_menu_sits_before_view_and_waits_for_its_jobs() {
    let mut h = Harness::new();
    let bar = h.panel(MENU);
    let titles = h.texts_in(bar);
    assert_eq!(titles, ["File", "Edit", "Object", "Type", "Select", "Effect", "Drawing", "View", "Window", "Help"]);
    let at = |h: &Harness, title: &str| h.texts.iter().find(|p| p.text == title && bar.contains_rect(p.rect)).unwrap().rect.center();
    h.click_at(at(&h, "Drawing"));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 9, "the Drawing menu's items: {items:?}");
    assert!(items.iter().all(|(_, enabled)| !enabled), "they wait for their jobs");
    let shown: String = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("|");
    for item in ["Weights at Once…", "Make Level Lines", "Scale and Coordinate System…", "Send to Strata…"] {
        assert!(shown.contains(item), "{item} in {shown}");
    }
    // A second click puts the menu away; View, after it, is the app's own View menu.
    h.click_at(at(&h, "Drawing"));
    assert!(h.popup_buttons().is_empty(), "the Drawing menu closed");
    h.click_at(at(&h, "View"));
    assert!(h.popup_buttons().iter().any(|(_, enabled)| *enabled), "View's items run");
}

#[test]
fn ribbon_follows_the_mockup_and_runs_commands() {
    let mut h = Harness::new();
    let keys = TILES;
    let ribbon = h.panel(RIBBON);
    let tiles: Vec<Rect> = keys.iter().map(|k| h.rect(wid("tile", k)).unwrap_or_else(|| panic!("no tile {k}"))).collect();
    for (k, t) in keys.iter().zip(&tiles) {
        assert_eq!(t.width(), 52.0, "{k}");
        assert!(ribbon.contains_rect(*t), "{k} in the ribbon");
    }
    assert!(tiles.windows(2).all(|w| w[0].right() <= w[1].left()), "left to right as in the mockup");
    let groups = h.line_in(ribbon);
    for g in ["File", "New Drawing", "Underlay", "Draw", "Drawing Elements", "View", "Export"] {
        assert!(groups.contains(g), "{g} in {groups}");
    }
    let t = h.tokens();
    let well = |h: &Harness, k: &str| {
        let tile = h.rect(wid("tile", k)).unwrap();
        h.fill_at(Rect::from_min_size(tile.min + vec2(4.0, 0.0), vec2(44.0, 44.0)))
    };
    // The Pen tile picks the Pen tool and shows it in hand.
    h.click(wid("tile", "pen"));
    assert_eq!(h.app.session.tool_id(), "pen");
    assert_eq!(well(&h, "pen"), Some(t.tool_active));
    // Grid shows the grid, and shows itself on.
    assert!(!h.app.ui.view.grid);
    h.click(wid("tile", "grid"));
    assert!(h.app.ui.view.grid);
    assert_eq!(well(&h, "grid"), Some(t.row_selected));
    // Save turns clay while the document has changes.
    assert_eq!(well(&h, "save"), Some(t.button));
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    h.settle();
    assert_eq!(well(&h, "save"), Some(t.action));
    // A tile a later job brings is dimmed and does nothing.
    assert_eq!(well(&h, "section"), Some(t.panel_darker));
    let docs = h.app.session.documents().len();
    h.click(wid("tile", "section"));
    assert_eq!((h.app.session.documents().len(), h.app.session.tool_id()), (docs, "pen"));
    assert!(h.app.ui.dialog.is_none());
    // Find a command opens the command palette.
    h.click(wid("find", ()));
    assert!(h.app.ui.palette_open);
}

#[test]
fn control_bar_shows_the_layer_and_next_weight() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    h.app.run("layer.new", json!({"name": "Pit outline"})).unwrap();
    h.app.select_tool("pen");
    h.settle();
    let bar = h.panel(CONTROL);
    let line = h.line_in(bar);
    for want in ["Pen", "Pit outline", "1 pt", "Snap", "Tolerance"] {
        assert!(line.contains(want), "{want} in {line}");
    }
    // The legend field lists the layers; picking one makes it current.
    h.click(wid("ctl-layer", ()));
    h.click(wid("ctl-layer-row", first.0));
    assert_eq!(h.current_layer(), Some(first));
    assert!(h.line_in(bar).contains("Layer 1"), "{}", h.line_in(bar));
    // The Snap toggle is View › Snap to Point.
    let snap = h.app.ui.view.snap_to_point;
    h.click(wid("ctl-snap", ()));
    assert_eq!(h.app.ui.view.snap_to_point, !snap);
    // Other tools keep the classic Control bar: with nothing selected, Document Setup. So do the
    // anchor tools, whose options are there.
    for tool in ["selection", "anchorPoint", "addAnchor", "smooth"] {
        h.app.select_tool(tool);
        h.settle();
        let line = h.line_in(bar);
        assert!(line.contains("Document Setup") && line.contains("Snap") && !line.contains("Legend"), "{tool}: {line}");
    }
}

/// Smaller windows keep every tile and tool and the fill and stroke in reach: a 1920 × 1080 screen
/// at 125 % less the task bar, a 1366 × 768 laptop's, the smallest window. The ribbon folds Find a
/// Command to its magnifier, then narrows its tiles, then scrolls (the mouse wheel too); the tool
/// column tightens, then scrolls above the fill and stroke, which stay at its foot.
#[test]
fn small_windows_keep_every_tile_and_tool_in_reach() {
    for (w, height) in [(1536.0, 816.0), (1366.0, 728.0), (800.0, 500.0)] {
        let mut h = Harness::sized(w, height);
        let (ribbon, column) = (h.panel(RIBBON), h.panel(TOOLS));
        let find = h.rect(wid("find", ())).unwrap_or_else(|| panic!("{w}: Find a Command"));
        assert!(ribbon.contains_rect(find), "{w}: {find:?} in {ribbon:?}");
        let tile_shown = |h: &Harness, k: &str| {
            let r = h.rect(wid("tile", k)).unwrap_or_else(|| panic!("{w}: no tile {k}"));
            ribbon.contains_rect(r) && r.right() <= find.left()
        };
        if w >= 1366.0 {
            let hidden: Vec<_> = TILES.iter().filter(|k| !tile_shown(&h, k)).collect();
            assert!(hidden.is_empty(), "{w}: tiles cut off {hidden:?}");
        } else {
            assert!(tile_shown(&h, "open") && !tile_shown(&h, "strata"), "{w}: the ribbon scrolls");
            h.wheel(ribbon.center(), vec2(0.0, -4000.0));
            assert!(tile_shown(&h, "strata"), "{w}: the wheel brings the last tile in");
        }
        // The fill and stroke stay at the foot of the column, the tools above them.
        let fill = h.rect(wid("proxy", ()).with("fill-proxy")).unwrap_or_else(|| panic!("{w}: the fill and stroke"));
        assert!(column.contains_rect(fill), "{w}: {fill:?} in {column:?}");
        let tool_shown = |h: &Harness, id: egui::Id| {
            let r = h.rect(id).unwrap_or_else(|| panic!("{w}: no {id:?}"));
            column.contains_rect(r) && r.bottom() <= fill.top()
        };
        let all: Vec<egui::Id> = SLOTS.iter().map(|k| wid("tool", k)).chain(ARCH.iter().map(|k| wid("arch", k))).collect();
        if height >= 816.0 {
            let hidden: Vec<_> = all.iter().filter(|id| !tool_shown(&h, **id)).collect();
            assert!(hidden.is_empty(), "{w}: tools cut off {hidden:?}");
        } else {
            let last = wid("arch", "hachure");
            assert!(tool_shown(&h, wid("tool", "selection")) && !tool_shown(&h, last), "{w}: the column scrolls");
            h.wheel(h.rect(wid("tool", "selection")).unwrap().center(), vec2(0.0, -4000.0));
            assert!(tool_shown(&h, last) && tool_shown(&h, wid("tool", "zoom")), "{w}: the wheel brings the last tools in");
        }
        // Find a Command still opens the command palette.
        h.click(wid("find", ()));
        assert!(h.app.ui.palette_open, "{w}");
    }
}

#[test]
fn tool_column_has_the_mockup_groups() {
    let mut h = Harness::new();
    let primaries = SLOTS;
    let column = h.panel(TOOLS);
    let rects: Vec<Rect> = primaries.iter().map(|p| h.rect(wid("tool", p)).unwrap_or_else(|| panic!("no tool {p}"))).collect();
    for (p, rect) in primaries.iter().zip(&rects) {
        assert_eq!(rect.size(), vec2(40.0, 27.0), "{p}");
        assert!(column.contains_rect(*rect), "{p} in the column");
    }
    assert!(rects.windows(2).all(|w| w[0].bottom() <= w[1].top()), "top to bottom as in the mockup");
    // The groups part after V A, after T, after C and after I.
    let gap = |a: usize| rects[a + 1].top() - rects[a].bottom();
    for a in [1, 6, 9, 11] {
        assert!(gap(a) > 4.0, "a separator after {}", primaries[a]);
    }
    assert!(gap(0) < 2.0 && gap(3) < 2.0);
    h.click(wid("tool", "pen"));
    assert_eq!(h.app.session.tool_id(), "pen");
    // The Pen slot keeps the tool last used in it.
    h.app.select_tool("curvature");
    h.settle();
    h.click(wid("tool", "selection"));
    assert_eq!(h.app.session.tool_id(), "selection");
    h.click(wid("tool", "pen"));
    assert_eq!(h.app.session.tool_id(), "curvature");
    // A right click opens the slot's flyout.
    let at = h.rect(wid("tool", "pen")).unwrap().center();
    h.press(at, PointerButton::Secondary);
    assert!(h.app.ui.flyout.is_some(), "the Pen slot's flyout");
    // The archaeology tools wait for J009.
    for a in ARCH {
        let rect = h.rect(wid("arch", a)).unwrap_or_else(|| panic!("no {a}"));
        assert!(column.contains_rect(rect) && rect.top() > rects[13].bottom(), "{a} under the tools");
    }
    h.app.ui.flyout = None;
    h.click(wid("arch", "section-line"));
    assert_eq!(h.app.session.tool_id(), "curvature");
}

#[test]
fn drawing_set_lists_open_documents_and_switches() {
    let mut h = Harness::new();
    h.app.run("file.new", json!({"name": "Site 2", "width": 200, "height": 100})).unwrap();
    h.settle();
    let set = h.panel(SET);
    let line = h.line_in(set);
    assert!(line.contains("Drawing Set") && line.contains("Site 1") && line.contains("Site 2"), "{line}");
    assert!(line.contains("Scale not set"), "{line}");
    let (a, b) = (h.rect(wid("set-row", 0usize)).unwrap(), h.rect(wid("set-row", 1usize)).unwrap());
    assert!(set.contains_rect(a) && a.bottom() <= b.top());
    h.click(wid("set-row", 0usize));
    assert_eq!(h.app.session.active_index(), Some(0));
    // From the Home screen too.
    h.app.run("app.home", json!({})).unwrap();
    h.settle();
    assert!(crate::menus::home_showing(&h.app));
    h.click(wid("set-row", 1usize));
    assert!(!crate::menus::home_showing(&h.app));
    assert_eq!(h.app.session.active_index(), Some(1));
    // ＋ is File › New.
    h.click(wid("set-new", ()));
    assert!(h.app.ui.dialog.is_some(), "the New Document dialog");
}

#[test]
fn tabs_switch_documents_and_show_unsaved() {
    let mut h = Harness::new();
    h.app.run("file.new", json!({"name": "Site 2", "width": 200, "height": 100})).unwrap();
    h.settle();
    let canvas = h.app.canvas_rect.unwrap();
    let strip = r(272.0, 196.0, 1296.0, canvas.top() - 16.0);
    let line = h.line_in(strip);
    assert!(line.contains("Home") && line.contains("Site 1") && line.contains("Site 2"), "{line}");
    assert!(!line.contains("Not saved"), "{line}");
    h.click(wid("tab", 0usize));
    assert_eq!(h.app.session.active_index(), Some(0));
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    h.settle();
    assert!(h.line_in(strip).contains("Not saved"), "{}", h.line_in(strip));
    h.click(wid("tab-home", ()));
    assert!(crate::menus::home_showing(&h.app));
    h.click(wid("tab", 1usize));
    assert_eq!(h.app.session.active_index(), Some(1));
    // The × of a tab closes it (Site 2 has no changes: no question).
    h.hover(wid("tab", 1usize));
    h.click(wid("tab-close", 1usize));
    assert_eq!(h.app.session.documents().len(), 1);
}

#[test]
fn canvas_overlays_zoom_and_name_the_tool() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let canvas = h.app.canvas_rect.unwrap();
    let line = h.line_in(canvas);
    assert!(line.contains("Pen · Layer 1"), "{line}");
    assert!(line.contains("to add a corner point"), "the Pen's hint: {line}");
    let zoom = |h: &Harness| h.app.view().unwrap().zoom;
    let z0 = zoom(&h);
    // The screen readout: a whole percentage.
    assert!(line.contains(&format!("{:.0} %", z0 * 100.0)) && !line.contains(&format!("{:.2}", z0 * 100.0)), "{line}");
    for k in ["in", "out", "fit"] {
        assert!(canvas.contains_rect(h.rect(wid("zoom", k)).unwrap()), "{k}");
    }
    // Presses on what floats over the canvas don't reach the Pen under it.
    let nodes = |h: &Harness| h.app.session.active().unwrap().doc.node_count();
    let n = nodes(&h);
    h.click(wid("zoom", "in"));
    assert!(zoom(&h) > z0);
    h.click(wid("zoom", "out"));
    h.click(wid("zoom", "out"));
    assert!(zoom(&h) < z0);
    h.click(wid("zoom", "fit"));
    assert!((zoom(&h) - z0).abs() < 1e-6, "back to the fitted view");
    h.click(wid("band", ()));
    h.click(wid("readout", ()));
    assert_eq!(nodes(&h), n, "no point drawn under the zoom buttons, the band or the readout");
}

/// The tool's hint reads plainly, as in the mockup: no bold keys, its parts apart by "·"; on the
/// canvas the first part, in the status bar all of them.
#[test]
fn hints_read_plainly_as_in_the_mockup() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let semibold = egui::FontFamily::Name(crate::theme::FONT_UI_SEMIBOLD.into());
    // The hint painted in `within`: its text and whether any of it is bold.
    let hint = |h: &Harness, within: Rect| -> Vec<(String, bool)> {
        h.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) if within.contains_rect(t.visual_bounding_rect()) && t.galley.text().contains("to add a corner point") => {
                    let job = &t.galley.job;
                    Some((job.text.clone(), job.sections.iter().any(|s| s.format.font_id.family == semibold)))
                }
                _ => None,
            })
            .collect()
    };
    let band = hint(&h, h.app.canvas_rect.unwrap());
    assert_eq!(band, [("Click to add a corner point".to_string(), false)], "the first part only");
    let status = hint(&h, h.panel(STATUS));
    assert_eq!(status.len(), 1, "{status:?}");
    let (text, bold) = &status[0];
    assert!(!bold && !text.contains('|') && text.contains("Click to add a corner point · Drag to add a smooth point"), "{status:?}");
}

#[test]
fn palette_weight_chip_sets_the_next_stroke() {
    let mut h = Harness::new();
    let t = h.tokens();
    let right = h.panel(RIGHT);
    for w in ["0.1", "0.2", "0.3", "0.5", "0.8", "1.0"] {
        assert!(right.contains_rect(h.rect(wid("weight", w)).unwrap()), "{w}");
    }
    // Nothing selected: the weight of the next line drawn.
    h.click(wid("weight", "0.5"));
    assert_eq!(h.app.session.paint.stroke_width, 0.5);
    assert_eq!(h.fill_at(h.rect(wid("weight", "0.5")).unwrap()), Some(t.row_selected), "the chip shows on");
    // With a line selected: its weight.
    let id = h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap()["id"].clone();
    h.app.run("select.set", json!({ "ids": [id] })).unwrap();
    h.app.run("stroke.set", json!({"dash": [4, 2]})).unwrap();
    h.settle();
    h.click(wid("weight", "0.8"));
    let stroke = h.app.session.shown_stroke().unwrap();
    assert_eq!(stroke.width, 0.8);
    // Solid clears the dash.
    assert!(stroke.dash.is_some());
    h.click(wid("dash", "solid"));
    assert!(h.app.session.shown_stroke().unwrap().dash.is_none());
    // Hatches wait for J014.
    h.click(wid("hatch", "45"));
    assert_eq!(h.app.session.shown_stroke().unwrap().width, 0.8);
    assert_eq!(h.fill_at(h.rect(wid("hatch", "none")).unwrap()), Some(t.row_selected), "no hatch yet");
    // ⋯ pops the Stroke panel out.
    h.click(wid("palette-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("stroke"));
}

#[test]
fn legend_rows_show_hide_lock_and_pick_the_layer() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    let pits = NodeId(h.app.run("layer.new", json!({"name": "Pits"})).unwrap()["id"].as_u64().unwrap());
    h.settle();
    let right = h.panel(RIGHT);
    let (top, below) = (h.rect(wid("legend-row", pits.0)).unwrap(), h.rect(wid("legend-row", first.0)).unwrap());
    assert!(right.contains_rect(top) && top.bottom() <= below.top(), "the top layer first");
    assert_eq!(top.height(), 24.0);
    let layer = |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().clone();
    h.click(wid("legend-eye", first.0));
    assert!(!layer(&h, first).visible);
    h.click(wid("legend-eye", first.0));
    assert!(layer(&h, first).visible);
    h.click(wid("legend-lock", first.0));
    assert!(layer(&h, first).locked);
    h.click(wid("legend-lock", first.0));
    assert!(!layer(&h, first).locked);
    assert_eq!(h.current_layer(), Some(pits));
    h.click(wid("legend-row", first.0));
    assert_eq!(h.current_layer(), Some(first));
    // ▸ lists the layer's objects.
    h.click(wid("legend-open", first.0));
    let objects = h.app.session.active().unwrap().doc.node(first).unwrap().children().unwrap().clone();
    let obj = h.rect(wid("legend-obj", objects[0].id.0)).expect("the rectangle's row");
    assert_eq!(obj.height(), 20.0);
    // ＋ adds a layer.
    let n = h.layer_ids().len();
    h.click(wid("legend-new", ()));
    assert_eq!(h.layer_ids().len(), n + 1);
    let line = h.line_in(right);
    assert!(line.contains("Legend · Layers") && line.contains("Pits"), "{line}");
}

/// A locked or hidden layer takes no new lines (the engine draws on the topmost layer that is shown
/// and unlocked instead), so when the current one is, the band, the Control bar and the legend say
/// where lines go rather than naming a layer they won't land in.
#[test]
fn a_locked_or_hidden_current_layer_shows_where_lines_go() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let pits = NodeId(h.app.run("layer.new", json!({"name": "Pits"})).unwrap()["id"].as_u64().unwrap());
    h.app.select_tool("pen");
    h.settle();
    let (canvas, bar, right) = (h.app.canvas_rect.unwrap(), h.panel(CONTROL), h.panel(RIGHT));
    let t = h.tokens();
    assert!(h.line_in(canvas).contains("Pen · Pits"), "{}", h.line_in(canvas));
    assert_eq!(h.color_of("Pits", right), Some(t.text));
    h.click(wid("legend-lock", pits.0));
    assert_eq!(h.current_layer(), Some(pits), "still the current layer");
    let line = h.line_in(canvas);
    assert!(line.contains("Pen · Layer 1") && line.contains("Pits is locked: drawing on Layer 1"), "{line}");
    assert!(h.line_in(bar).contains("Layer 1") && !h.line_in(bar).contains("Pits"), "{}", h.line_in(bar));
    assert_eq!(h.color_of("Pits", right), Some(t.clay_text), "the legend marks the current row");
    let id = h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap()["id"].as_u64().unwrap();
    assert_eq!(h.app.session.active().unwrap().doc.layer_containing(NodeId(id)), Some(first), "where the band said");
    // Hidden likewise.
    h.app.run("layer.setCurrent", json!({ "id": pits.0 })).unwrap();
    h.click(wid("legend-lock", pits.0));
    h.click(wid("legend-eye", pits.0));
    assert_eq!(h.current_layer(), Some(pits));
    assert!(h.line_in(canvas).contains("Pits is hidden: drawing on Layer 1"), "{}", h.line_in(canvas));
    // With every layer locked or hidden nothing is drawn, and the band says so.
    h.click(wid("legend-lock", first.0));
    let line = h.line_in(canvas);
    assert!(line.contains("Pits is hidden: no layer to draw on") && !line.contains("Pen · Layer 1"), "{line}");
    assert!(h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).is_err());
}

/// A layer of thousands of objects opened in the legend lays out only the rows in view (the rest
/// scroll in), so it doesn't slow every frame down.
#[test]
fn a_big_open_layer_lays_out_only_the_rows_in_view() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let ids: Vec<u64> = (0..1500)
        .map(|i| h.app.run("shape.rectangle", json!({"x": i % 300, "y": 10, "width": 5, "height": 5})).unwrap()["id"].as_u64().unwrap())
        .collect();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.click(wid("legend-open", first.0));
    let rows = |h: &Harness| ids.iter().filter(|id| h.rect(wid("legend-obj", **id)).is_some()).count();
    let laid_out = rows(&h);
    assert!((5..60).contains(&laid_out), "{laid_out} of {} rows laid out", ids.len());
    // The top one (the last drawn) shows; the wheel brings the bottom one (the first drawn).
    let right = h.panel(RIGHT);
    let (top, bottom) = (*ids.last().unwrap(), ids[0]);
    assert!(right.contains_rect(h.rect(wid("legend-obj", top)).unwrap()));
    assert!(h.rect(wid("legend-obj", bottom)).is_none());
    let list = h.rect(wid("legend-obj", top)).unwrap().center();
    h.wheel(list, vec2(0.0, -1.0e6));
    let row = h.rect(wid("legend-obj", bottom)).expect("the bottom row, scrolled in");
    assert!(right.contains_rect(row), "{row:?}");
    // A click on it selects it.
    h.click(wid("legend-obj", bottom));
    assert_eq!(h.app.session.active().unwrap().selection.objects, [NodeId(bottom)]);
}

/// A message shows in the status bar for a while, then the hint again; a press on something a
/// later job brings says so there.
#[test]
fn status_bar_shows_messages_for_a_while() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let bar = h.panel(STATUS);
    assert!(h.line_in(bar).contains("to add a corner point"), "{}", h.line_in(bar));
    h.app.status("Saved Site 1");
    h.settle();
    let line = h.line_in(bar);
    assert!(line.contains("Saved Site 1") && !line.contains("to add a corner point"), "{line}");
    h.later(7.0);
    let line = h.line_in(bar);
    assert!(!line.contains("Saved Site 1") && line.contains("to add a corner point"), "{line}");
    // Section (J004) and Change All Weights (J005): what they are and when.
    h.click(wid("tile", "section"));
    assert!(h.line_in(bar).contains("Section: coming soon (J004)"), "{}", h.line_in(bar));
    h.click(wid("legend-weights", ()));
    assert!(h.line_in(bar).contains("Change All Weights…: coming soon (J005)"), "{}", h.line_in(bar));
    h.later(5.0);
    assert!(!h.line_in(bar).contains("coming soon"), "{}", h.line_in(bar));
}

/// The Underlay tile places a scan as a template (a locked, dimmed layer under the drawing); with
/// one placed it shows and hides the template layers.
#[test]
fn underlay_tile_places_a_template_then_shows_and_hides_it() {
    let mut h = Harness::new();
    let dir = std::env::temp_dir().join(format!("trowel-underlay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let scan = dir.join("scan.svg");
    std::fs::write(&scan, r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30"/></svg>"#).unwrap();
    let path = scan.to_string_lossy().to_string();
    h.app.services.pick_open = Some(Box::new(move |_| Some(path.clone())));
    h.click(wid("tile", "underlay"));
    let d = h.app.ui.dialog.as_ref().expect("the Place dialog");
    assert_eq!(d.kind, crate::dialogs::place::KIND);
    assert_eq!(d.fields.get("template"), Some(&json!(true)), "Template ticked");
    h.app.ui.dialog = None;
    // With a template layer: the tile shows it on and hides it, then shows it again.
    let id = h.app.run("layer.new", json!({"name": "Scan"})).unwrap()["id"].as_u64().unwrap();
    h.app.run("layer.setProps", json!({"ids": [id], "template": true})).unwrap();
    h.settle();
    let t = h.tokens();
    let tile = h.rect(wid("tile", "underlay")).unwrap();
    let well = Rect::from_min_size(pos2(tile.center().x - 22.0, tile.top()), vec2(44.0, 44.0));
    assert_eq!(h.fill_at(well), Some(t.row_selected), "shown");
    let visible = |h: &Harness| h.app.session.active().unwrap().doc.node(NodeId(id)).unwrap().visible;
    h.click(wid("tile", "underlay"));
    assert!(!visible(&h) && h.app.ui.dialog.is_none());
    h.click(wid("tile", "underlay"));
    assert!(visible(&h));
    std::fs::remove_dir_all(&dir).ok();
}

/// The panels ⋯ opens pop out beside the right column, just under its top, however the classic
/// dock was left; with Auto-Collapse Iconic Panels on, a press on the right column puts them away.
#[test]
fn panels_pop_out_beside_the_right_column() {
    let mut h = Harness::new();
    h.app.ui.dock_collapsed = false;
    h.app.session.prefs.auto_collapse_icon_panels = true;
    h.click(wid("props-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("properties"));
    let right = h.panel(RIGHT);
    let flyout = h.ctx.memory(|m| m.area_rect(egui::Id::new("icon-panel"))).expect("the panel popped out");
    assert!((flyout.right() - (right.left() - 6.0)).abs() < 1.0 && (flyout.top() - (right.top() + 6.0)).abs() < 1.0, "{flyout:?} by {right:?}");
    h.click_at(pos2(right.left() + 12.0, right.top() + 8.0));
    assert!(h.app.ui.open_panel.is_none(), "put away");
}

/// The snapping tolerance is a field like the mockup's: a click lists 1 to 8 px, and a pick sets it.
#[test]
fn snap_tolerance_is_picked_from_a_list() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let field = h.rect(wid("ctl-tolerance", ())).unwrap();
    assert_eq!(h.texts_in(field), ["2", "px"]);
    h.click(wid("ctl-tolerance", ()));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 8, "{items:?}");
    h.click_at(items[3].0.center());
    assert_eq!(h.app.session.prefs.snap_to_point_tolerance, 4.0);
    let field = h.rect(wid("ctl-tolerance", ())).unwrap();
    assert_eq!(h.texts_in(field), ["4", "px"]);
}

/// In a small window the drawing tools' Control bar is cut at Snap rather than running under it.
#[test]
fn control_bar_keeps_snap_in_reach_in_a_small_window() {
    let mut h = Harness::sized(800.0, 500.0);
    h.app.select_tool("pen");
    h.settle();
    // Nothing else shows over Snap and its tolerance…
    let toggle = h.rect(wid("ctl-snap", ())).unwrap();
    let bar = h.panel(CONTROL);
    let snap_area = Rect::from_min_max(pos2(toggle.left(), bar.top()), bar.max);
    let under: Vec<String> = h
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Text(t) if !["Snap", "Tolerance", "2", "px"].contains(&t.galley.text()) => {
                let shown = c.clip_rect.intersect(t.visual_bounding_rect());
                (shown.is_positive() && shown.intersects(snap_area)).then(|| t.galley.text().to_string())
            }
            _ => None,
        })
        .collect();
    assert!(under.is_empty(), "over Snap: {under:?}");
    // …and a click on it reaches it.
    let snap = h.app.ui.view.snap_to_point;
    h.click(wid("ctl-snap", ()));
    assert_eq!(h.app.ui.view.snap_to_point, !snap);
}

#[test]
fn properties_summarise_the_selection() {
    let mut h = Harness::new();
    let right = h.panel(RIGHT);
    assert!(h.line_in(right).contains("No selection"), "{}", h.line_in(right));
    let id = h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 100, "height": 50})).unwrap()["id"].clone();
    h.app.run("select.set", json!({ "ids": [id] })).unwrap();
    h.settle();
    let u = h.app.session.general_unit();
    let size = format!("{} × {}", u.format(100.0), u.format(50.0));
    assert!(h.line_in(right).contains(&size), "{size} in {}", h.line_in(right));
    h.click(wid("props-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("properties"));
}

#[test]
fn status_bar_shows_tool_snap_unsaved_and_position() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let bar = h.panel(STATUS);
    let line = h.line_in(bar);
    let snap = if h.app.ui.view.snap_to_point { "Snap on" } else { "Snap off" };
    assert!(line.contains("Pen") && line.contains(snap) && line.contains("Scale and coordinate system not set"), "{line}");
    assert!(!line.contains("not saved"), "{line}");
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    // The pointer over the canvas: its position in the document.
    let at = h.app.canvas_rect.unwrap().center();
    h.frame(vec![Event::PointerMoved(at)]);
    h.settle();
    let p = h.app.hover_doc.expect("over the canvas");
    let u = h.app.session.general_unit();
    let line = h.line_in(bar);
    assert!(line.contains("1 not saved"), "{line}");
    assert!(line.contains(&u.readout(p.x)) && line.contains(&u.readout(p.y)), "{line}");
}

/// Trowel's colours come from the theme tokens, as everywhere in the app.
#[test]
fn trowel_code_takes_colours_from_tokens() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/trowel");
    let mut found = vec![];
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "tests.rs" {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).unwrap();
        for bad in ["Color32::from_", "Color32::WHITE", "Color32::BLACK", "Color32::GRAY", "Color32::LIGHT", "Color32::DARK", "Rgba::"] {
            if text.contains(bad) {
                found.push(format!("{name}: {bad}"));
            }
        }
    }
    assert!(found.is_empty(), "colours outside theme::Tokens: {found:?}");
}

/// Every Trowel icon the window names is bundled.
#[test]
fn every_trowel_icon_is_bundled() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/trowel");
    let mut names = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        if entry.file_name() == "tests.rs" {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).unwrap();
        for (at, _) in text.match_indices("\"tw-") {
            let rest = &text[at + 1..];
            names.insert(rest[..rest.find('"').unwrap()].to_string());
        }
    }
    assert!(names.len() >= 37, "{names:?}");
    let missing: Vec<_> = names.iter().filter(|n| !crate::icons::exists(n)).collect();
    assert!(missing.is_empty(), "icons not bundled: {missing:?}");
}
