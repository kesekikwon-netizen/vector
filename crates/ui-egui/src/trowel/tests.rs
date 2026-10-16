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

/// The ribbon's tiles, left to right (design 3.0 §5 ③: nine in four groups).
const TILES: [&str; 9] = ["new", "open", "save", "underlay", "align", "scale", "legend", "plate", "export"];

/// The tool column's slots, top to bottom, by their first tool (design 3.0 §5 ④).
const SLOTS: [&str; 18] = [
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
    "sectionCut",
    "levelLines",
    "spotHeight",
    "strataLine",
    "hand",
    "zoom",
];

/// The archaeology tools' slots under 「고고학」, keys 1–4 (design 3.0 D4).
const ARCH: [&str; 4] = ["sectionCut", "levelLines", "spotHeight", "strataLine"];

/// Text painted in the last frame: what it says, where and in what colour, and the part of it
/// that shows (inside its clip).
struct Painted {
    text: String,
    rect: Rect,
    color: egui::Color32,
    font: egui::FontId,
    shown: Rect,
}

struct Harness {
    app: VectorcraftApp,
    ctx: egui::Context,
    screen: egui::Vec2,
    time: f64,
    texts: Vec<Painted>,
    shapes: Vec<egui::epaint::ClippedShape>,
    /// What went to the system clipboard, oldest first.
    copied: Vec<String>,
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
        let lang = app.ui_language();
        crate::workspaces::start_trowel(&mut app.ui, lang);
        app.run("file.new", json!({"name": "Site 1", "width": 400, "height": 300})).unwrap();
        let mut h = Harness { app, ctx: egui::Context::default(), screen: vec2(w, h), time: 0.0, texts: vec![], shapes: vec![], copied: vec![] };
        // The first frame installs the fonts, the second lays the window out, the third sees it.
        for _ in 0..3 {
            h.frame(vec![]);
        }
        h
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.time += 0.05;
        let screen_rect = Some(Rect::from_min_size(Pos2::ZERO, self.screen));
        let raw = egui::RawInput { time: Some(self.time), screen_rect, events, ..Default::default() };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(raw, |ui| {
            app.logic(ui.ctx());
            app.ui(ui);
        });
        out.textures_delta.clear();
        for c in &out.platform_output.commands {
            if let egui::OutputCommand::CopyText(t) = c {
                self.copied.push(t.clone());
            }
        }
        self.texts = out
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) => {
                    let format = t.galley.job.sections.first().map(|s| &s.format);
                    let color = format.map(|f| f.color).filter(|c| *c != egui::Color32::PLACEHOLDER);
                    let rect = t.visual_bounding_rect();
                    Some(Painted {
                        text: t.galley.text().to_string(),
                        rect,
                        color: color.unwrap_or(t.fallback_color),
                        font: format.map(|f| f.font_id.clone()).unwrap_or_default(),
                        shown: rect.intersect(c.clip_rect),
                    })
                }
                _ => None,
            })
            .collect();
        self.shapes = out.shapes;
    }

    /// Where the document point (`x`, `y`) shows on the canvas.
    fn on_screen(&self, x: f64, y: f64) -> Pos2 {
        let xf = crate::canvas::Xf::new(self.app.canvas_rect.unwrap(), self.app.view().unwrap());
        xf.to_screen(vectorcraft_geom::Point::new(x, y))
    }

    /// What the text field `id` holds.
    fn field_text(&self, id: egui::Id) -> Option<String> {
        let _ = self.rect(id)?;
        self.app.trowel_ask.as_ref().and_then(|a| a.text.clone())
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

    /// A click on the widget `id` with Alt held.
    fn alt_click(&mut self, id: egui::Id) {
        self.frame(vec![Event::ModifiersChanged(egui::Modifiers::ALT)]);
        self.click(id);
        self.frame(vec![Event::ModifiersChanged(egui::Modifiers::NONE)]);
    }

    /// Pick `label` in the legend's ⋯.
    fn legend_more(&mut self, label: &str) {
        self.click(wid("legend-more", ()));
        self.click_text(label, Rect::from_min_size(Pos2::ZERO, self.screen));
    }

    /// Weights at Once, from the legend's ⋯.
    fn open_weights(&mut self) {
        self.legend_more("Change Weights at Once…");
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

    /// Two clicks in a row on the widget `id`.
    fn double_click(&mut self, id: egui::Id) {
        let at = self.rect(id).unwrap_or_else(|| panic!("no widget {id:?}")).center();
        self.frame(vec![Event::PointerMoved(at)]);
        for _ in 0..2 {
            for pressed in [true, false] {
                self.frame(vec![Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Default::default() }]);
            }
        }
        self.settle();
    }

    /// A touchpad's tap at `at`: the press and the release in one frame.
    fn tap(&mut self, at: Pos2) {
        let (button, modifiers) = (PointerButton::Primary, egui::Modifiers::default());
        self.frame(vec![
            Event::PointerMoved(at),
            Event::PointerButton { pos: at, button, pressed: true, modifiers },
            Event::PointerButton { pos: at, button, pressed: false, modifiers },
        ]);
    }

    /// Press and let go of `key`.
    fn key(&mut self, key: egui::Key) {
        for pressed in [true, false] {
            self.frame(vec![Event::Key { key, physical_key: None, pressed, repeat: false, modifiers: Default::default() }]);
        }
        self.settle();
    }

    /// Click the text field `id`, select what it holds and type `text` over it, then Enter.
    fn type_into(&mut self, id: egui::Id, text: &str) {
        self.type_over(id, text);
        self.key(egui::Key::Enter);
    }

    /// Click the text field `id`, select what it holds and type `text` over it; it keeps the
    /// keyboard.
    fn type_over(&mut self, id: egui::Id, text: &str) {
        self.click(id);
        let all = egui::Modifiers::COMMAND;
        self.frame(vec![Event::Key { key: egui::Key::A, physical_key: None, pressed: true, repeat: false, modifiers: all }]);
        self.frame(vec![Event::Text(text.to_string())]);
    }

    fn hover(&mut self, id: egui::Id) {
        let at = self.rect(id).unwrap_or_else(|| panic!("no widget {id:?}")).center();
        self.frame(vec![Event::PointerMoved(at)]);
        self.settle();
    }

    /// Rest the pointer on `at` long enough for its hover text to show.
    fn rest_at(&mut self, at: Pos2) {
        self.frame(vec![Event::PointerMoved(at)]);
        self.later(1.0);
    }

    /// Whether the widget `id` took presses in the last frame: not when it was in a disabled part.
    fn enabled(&self, id: egui::Id) -> Option<bool> {
        self.ctx.viewport(|vp| vp.prev_pass.widgets.get(id).map(|w| w.enabled))
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

    /// Pairs of texts showing in `area` that are drawn over each other.
    fn overlapping_texts(&self, area: Rect) -> Vec<(String, String)> {
        let shown: Vec<&Painted> = self.texts.iter().filter(|p| p.shown.is_positive() && p.shown.intersects(area)).collect();
        let mut v = vec![];
        for (i, a) in shown.iter().enumerate() {
            for b in shown.iter().skip(i + 1) {
                let both = a.shown.intersect(b.shown);
                if both.width() > 1.0 && both.height() > 1.0 {
                    v.push((a.text.clone(), b.text.clone()));
                }
            }
        }
        v
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

    /// The rectangles painted inside `area`: where, their fill and their edge.
    fn rects_in(&self, area: Rect) -> Vec<(Rect, egui::Color32, egui::Stroke)> {
        self.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Rect(s) if area.expand(0.5).contains_rect(s.rect) => Some((s.rect, s.fill, s.stroke)),
                _ => None,
            })
            .collect()
    }

    /// The straight line segments painted inside `area`.
    fn segments_in(&self, area: Rect) -> Vec<([Pos2; 2], egui::Stroke)> {
        self.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::LineSegment { points, stroke } if points.iter().all(|p| area.expand(0.5).contains(*p)) => Some((*points, *stroke)),
                _ => None,
            })
            .collect()
    }

    /// Where colour gradients (a scroll area's fading edges) were painted over `area`.
    fn gradients_over(&self, area: Rect) -> Vec<Rect> {
        self.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Mesh(m) if m.vertices.len() == 4 && m.vertices.iter().any(|v| v.color != m.vertices[0].color) => {
                    Some(m.calc_bounds()).filter(|b| b.intersects(area))
                }
                _ => None,
            })
            .collect()
    }

    /// The font `text` was painted in inside `area`.
    fn font_of(&self, text: &str, area: Rect) -> Option<egui::FontId> {
        self.texts.iter().find(|p| p.text == text && area.expand(0.5).contains_rect(p.rect)).map(|p| p.font.clone())
    }

    /// What the status bar says.
    fn status_line(&self) -> String {
        self.line_in(self.panel(STATUS))
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

    /// Click the button reading `label` inside `area`, for buttons without an id of their own.
    fn click_text(&mut self, label: &str, area: Rect) {
        let at = self
            .texts
            .iter()
            .find(|p| p.text == label && area.contains(p.shown.center()))
            .map(|p| p.shown.center())
            .unwrap_or_else(|| panic!("no {label:?} in {area:?}"));
        self.click_at(at);
    }

    fn node_count(&self) -> usize {
        self.app.session.active().unwrap().doc.node_count()
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

/// The window's parts where design 3.0 puts them (§4): the ribbon 86 high with no Control bar
/// under it, the status bar 28, the tabs 34.
#[test]
fn regions_match_the_mockup_at_1600_by_988() {
    let mut h = Harness::new();
    let want = [
        (CAPTION, r(0.0, 0.0, 1600.0, 32.0)),
        (MENU, r(0.0, 32.0, 1600.0, 60.0)),
        (RIBBON, r(0.0, 60.0, 1600.0, 146.0)),
        (STATUS, r(0.0, 960.0, 1600.0, 988.0)),
        (TOOLS, r(0.0, 146.0, 50.0, 960.0)),
        (SET, r(50.0, 146.0, 266.0, 960.0)),
        (RIGHT, r(1296.0, 146.0, 1600.0, 960.0)),
    ];
    for (id, rect) in want {
        assert!(close(h.panel(id), rect), "{id}: {:?}, the mockup {rect:?}", h.panel(id));
    }
    assert!(egui::PanelState::load(&h.ctx, egui::Id::new("trowel-control")).is_none(), "no Control bar");
    // The canvas region is 1030 × 814 under the 34-point tabs (the tool column is 50, the drawing
    // set keeps its 216 until D5); the rulers take 16 points of it.
    let canvas = h.app.canvas_rect.unwrap();
    assert!(close(canvas, r(282.0, 196.0, 1296.0, 960.0)), "{canvas:?}");
    // Full screen with menu bar off: no caption, menu or ribbon.
    h.app.ui.screen_mode = 2;
    h.settle();
    assert!(close(h.app.canvas_rect.unwrap(), r(282.0, 50.0, 1296.0, 960.0)), "{:?}", h.app.canvas_rect);
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
    assert_eq!(items.len(), 10, "the Drawing menu's items: {items:?}");
    // The first 7 run (Scale and Coordinate System… since J022); the others wait for their jobs.
    assert!(items.iter().take(7).all(|(_, enabled)| *enabled), "they run");
    assert!(items.iter().skip(7).all(|(_, enabled)| !enabled), "they wait for their jobs");
    let shown: String = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("|");
    for item in ["Weights at Once…", "Make Level Lines", "Scale and Coordinate System…", "Insert Legend Table", "Strata GeoPackage"] {
        assert!(shown.contains(item), "{item} in {shown}");
    }
    assert!(!shown.contains("캔버스 범례표 삽입"), "renamed as in design 3.0 §5 ②: {shown}");
    // A second click puts the menu away; View, after it, is the app's own View menu.
    h.click_at(at(&h, "Drawing"));
    assert!(h.popup_buttons().is_empty(), "the Drawing menu closed");
    h.click_at(at(&h, "View"));
    assert!(h.popup_buttons().iter().any(|(_, enabled)| *enabled), "View's items run");
    h.key(egui::Key::Escape);
    // Scale and Coordinate System… opens the ribbon tile's dialog.
    h.click_at(at(&h, "Drawing"));
    let items = h.popup_buttons();
    h.click_at(items[2].0.center());
    assert!(h.rect(wid("scale-chip", 20u32)).is_some(), "the scale dialog");
}

/// The ribbon of design 3.0 (§4, §5 ③): nine tiles in four groups, each doing one thing to the
/// drawing as a whole.
#[test]
fn ribbon_follows_the_mockup_and_runs_commands() {
    let mut h = Harness::new();
    let ribbon = h.panel(RIBBON);
    let tiles: Vec<Rect> = TILES.iter().map(|k| h.rect(wid("tile", k)).unwrap_or_else(|| panic!("no tile {k}"))).collect();
    for (k, t) in TILES.iter().zip(&tiles) {
        assert_eq!(t.width(), 58.0, "{k}");
        assert!(ribbon.contains_rect(*t), "{k} in the ribbon");
    }
    assert!(tiles.windows(2).all(|w| w[0].right() <= w[1].left()), "left to right as in the mockup");
    // Four groups, their captions 11.5 points and not bold; the old groups and tiles are gone.
    let texts = h.texts_in(ribbon);
    for g in ["File", "Underlay", "Drawing", "Export"] {
        assert!(texts.iter().any(|s| s == g), "{g} in {texts:?}");
    }
    assert_eq!(h.font_of("Underlay", ribbon), Some(font(11.5)), "a group's caption");
    for gone in ["Draw", "Drawing Elements", "View", "Pen", "Grid", "Snap", "North Arrow"] {
        assert!(!texts.iter().any(|s| s == gone), "{gone} left the ribbon: {texts:?}");
    }
    for k in ["pen", "pencil", "section-line", "spot-height", "grid", "magnet", "north", "pdf"] {
        assert!(h.rect(wid("tile", k)).is_none(), "{k} left the ribbon");
    }
    let t = h.tokens();
    let well = |h: &Harness, k: &str| {
        let tile = h.rect(wid("tile", k)).unwrap();
        h.fill_at(Rect::from_min_size(pos2(tile.center().x - 23.0, tile.top()), vec2(46.0, 40.0)))
    };
    // A well has no ground at rest; Save turns clay while the document has changes.
    assert_eq!(well(&h, "save"), None);
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    h.settle();
    assert_eq!(well(&h, "save"), Some(t.action));
    // New Drawing: the tile, its label too, opens the New dialog; its ▾ alone the four kinds.
    h.click(wid("tile", "new"));
    assert!(h.app.ui.dialog.is_some(), "the New dialog");
    h.app.ui.dialog = None;
    h.click_text("New Drawing", ribbon);
    assert!(h.app.ui.dialog.is_some(), "the label opens the New dialog");
    assert!(!egui::Popup::is_id_open(&h.ctx, wid("tile-list", "new")), "not the list");
    h.app.ui.dialog = None;
    let more = h.rect(wid("tile-more", "new")).unwrap();
    assert!(more.width() <= 14.0, "the ▾ only: {more:?}");
    // "New Drawing ▾" is wider than its tile in English (not in Korean, 「새 도면 ▾」): its ▾'s
    // left, clear of the Open tile.
    let at = pos2(more.left() + 1.0, more.center().y);
    assert!(!h.rect(wid("tile", "open")).unwrap().contains(at));
    h.rest_at(at);
    assert!(h.texts.iter().any(|p| p.text == "Plan, section, finds or plate at once"), "the ▾ says what it lists");
    h.click_at(at);
    let items = h.popup_buttons();
    assert_eq!(items.len(), 4, "plan, section, finds, plate: {items:?}");
    assert!(items.iter().all(|(_, enabled)| *enabled));
    let docs = h.app.session.documents().len();
    h.click_at(items[1].0.center());
    assert_eq!(h.app.session.documents().len(), docs + 1, "a section drawing");
    assert_eq!(h.app.session.active().unwrap().doc.scale.as_ref().map(|s| s.scale.denom), Some(20.0));
    // Align waits for its job: no ground, a dimmed icon, a note and nothing else.
    assert_eq!(well(&h, "align"), None);
    let docs = h.app.session.documents().len();
    h.click(wid("tile", "align"));
    assert_eq!(h.app.session.documents().len(), docs);
    assert!(h.app.ui.dialog.is_none());
    assert!(h.status_line().contains("Align: coming soon (J008)"), "{}", h.status_line());
    // Scale · CRS: Trowel's own dialog on the drawing's scale and coordinate system, not the
    // generic one (no "Epsg" field).
    h.click(wid("tile", "scale"));
    assert!(h.app.ui.dialog.is_none(), "not the generic parameter dialog");
    assert!(h.rect(wid("scale-chip", 20u32)).is_some() && h.rect(wid("crs-chip", 5186u32)).is_some(), "the scale dialog");
    h.click(wid("scale-dialog", "cancel"));
    assert!(h.rect(wid("scale-chip", 20u32)).is_none(), "put away");
    // Legend Table: a table of the rows in use, near the middle of the view.
    let (before, seq) = (h.node_count(), h.app.ui.status_seq);
    h.click(wid("tile", "legend"));
    assert_eq!(h.app.ui.status_seq, seq, "{}", h.app.ui.status);
    assert!(h.node_count() > before, "a legend table");
    // Plate: the plate dialog.
    h.click(wid("tile", "plate"));
    assert_eq!(h.ctx.data(|d| d.get_temp::<bool>(wid("plate-dialog-open", ()))), Some(true));
    // Export: the whole tile lists PDF, DXF, SVG and Strata.
    h.click(wid("tile", "export"));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 4, "{items:?}");
    let shown = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("|");
    for item in ["PDF…", "DXF (keeps coordinates)…", "SVG…", "Strata Survey Data…"] {
        assert!(shown.contains(item), "{item} in {shown}");
    }
    // Strata Survey Data… opens its browser, and opens it again rather than putting it away.
    let strata = super::gpkg_browser::gpkg_dialog_open_id();
    for _ in 0..2 {
        h.click_at(items[3].0.center());
        assert_eq!(h.ctx.data(|d| d.get_temp::<bool>(strata)), Some(true));
        h.click(wid("tile", "export"));
    }
    h.key(egui::Key::Escape);
    h.ctx.data_mut(|d| d.insert_temp(strata, false));
    // Find a command opens the command palette; its shortcut shows at its right.
    let find = h.rect(wid("find", ())).unwrap();
    assert_eq!(find.size(), vec2(210.0, 32.0));
    // Its shortcut shows at its right when the label leaves room ("명령 찾기" does), and on hover.
    let sc = crate::menus::pretty_shortcut(crate::menus::shortcut_of("help.commandPalette").unwrap());
    h.rest_at(find.center());
    let tip = format!("Search commands and tools ({sc})");
    assert!(h.texts.iter().any(|p| p.text == tip), "{tip}");
    h.click(wid("find", ()));
    assert!(h.app.ui.palette_open);
}

/// The Control bar under the ribbon is gone from Trowel's window (design 3.0 §5): Window › Control
/// is off there, and run another way it leaves the classic window's setting as it is. What it held
/// has one place each: the layer and the next line's look in the legend and the drawing palette,
/// Snap in the status bar. The classic window keeps its Control bar.
#[test]
fn the_control_bar_is_gone_in_trowel() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    assert!(egui::PanelState::load(&h.ctx, egui::Id::new("trowel-control")).is_none());
    for id in ["ctl-layer", "ctl-snap", "ctl-tolerance"] {
        assert!(h.rect(wid(id, ())).is_none(), "{id}");
    }
    assert!(!crate::menus::enabled(&h.app, "window.control"));
    assert_eq!(crate::menus::checked(&h.app, "window.control", &json!({})), Some(false));
    let classic = h.app.ui.control_bar;
    crate::menus::invoke(&mut h.app, "window.control", json!({}));
    assert_eq!(h.app.run("window.control", json!({})), Ok(json!(false)), "off in Trowel");
    assert_eq!(h.app.ui.control_bar, classic, "the classic window's Control bar stays as it was");
    let snap = h.app.ui.view.snap_to_point;
    h.click(wid("status", "snap"));
    assert_eq!(h.app.ui.view.snap_to_point, !snap, "Snap in the status bar");
    h.app.ui.trowel = false;
    assert!(crate::menus::enabled(&h.app, "window.control"), "the classic window's Control bar");
    assert_eq!(crate::menus::checked(&h.app, "window.control", &json!({})), Some(h.app.ui.control_bar));
}

/// Smaller windows keep every tile and tool in reach: a 1920 × 1080 screen at 125 % less the task
/// bar, a 1440 × 900 laptop's, a 1366 × 768 one's, the smallest window. The ribbon's nine tiles fit
/// them all, Find a Command folding to its magnifier in the smallest; the tool column tightens,
/// then scrolls.
#[test]
fn small_windows_keep_every_tile_and_tool_in_reach() {
    for (w, height) in [(1536.0, 816.0), (1440.0, 900.0), (1366.0, 728.0), (800.0, 500.0)] {
        let mut h = Harness::sized(w, height);
        let (ribbon, column) = (h.panel(RIBBON), h.panel(TOOLS));
        let find = h.rect(wid("find", ())).unwrap_or_else(|| panic!("{w}: Find a Command"));
        assert!(ribbon.contains_rect(find), "{w}: {find:?} in {ribbon:?}");
        // Find a Command keeps its field while the tiles leave it room (from 866 points).
        assert_eq!(find.width(), if w >= 866.0 { 210.0 } else { 32.0 }, "{w}: Find a Command's width");
        let tile_shown = |h: &Harness, k: &str| {
            let r = h.rect(wid("tile", k)).unwrap_or_else(|| panic!("{w}: no tile {k}"));
            ribbon.contains_rect(r) && r.right() <= find.left()
        };
        let hidden: Vec<_> = TILES.iter().filter(|k| !tile_shown(&h, k)).collect();
        assert!(hidden.is_empty(), "{w}: tiles cut off {hidden:?}");
        // No fill and stroke at the column's foot (design 3.0 §5 ④: they are the pen's).
        assert!(h.rect(wid("proxy", ()).with("fill-proxy")).is_none() && h.rect(wid("proxy-none", ())).is_none(), "{w}");
        let tool_shown = |h: &Harness, id: egui::Id| column.contains_rect(h.rect(id).unwrap_or_else(|| panic!("{w}: no {id:?}")));
        let all: Vec<egui::Id> = SLOTS.iter().map(|k| wid("tool", k)).collect();
        // The mockup's 687 points fit 1440 × 900's 726. Lower, the buttons tighten only as far as
        // they must (review J024 UI 11): the 4 lines, the caption and the gaps take 111, the 18
        // buttons share the rest to whole points, 22 at the least (1366 × 728's 554: 24).
        let sizes: Vec<f32> = all.iter().filter_map(|id| h.rect(*id)).map(|r| r.height()).collect();
        let fits = ((column.height() - 111.0) / 18.0).floor().clamp(22.0, 32.0);
        assert!(sizes.iter().all(|s| *s == fits), "{w}: {sizes:?} for {fits} in {column:?}");
        assert_eq!(fits == 32.0, height >= 900.0, "{w}");
        if height == 728.0 {
            assert_eq!(fits, 24.0, "{w}");
        }
        if height >= 728.0 {
            let hidden: Vec<_> = all.iter().filter(|id| !tool_shown(&h, **id)).collect();
            assert!(hidden.is_empty(), "{w}: tools cut off {hidden:?}");
        } else {
            let last = wid("tool", "zoom");
            assert!(tool_shown(&h, wid("tool", "selection")) && !tool_shown(&h, last), "{w}: the column scrolls");
            h.wheel(h.rect(wid("tool", "selection")).unwrap().center(), vec2(0.0, -4000.0));
            assert!(tool_shown(&h, last) && tool_shown(&h, wid("tool", "strataLine")), "{w}: the wheel brings the last tools in");
        }
        // Find a Command still opens the command palette.
        h.click(wid("find", ()));
        assert!(h.app.ui.palette_open, "{w}");
    }
}

/// A window too short for the right column scrolls it (the smallest, 800 × 500): the legend at the
/// top with room for two rows, the pen and the properties stay apart, and the wheel brings the
/// properties in. A taller one shows them all as they come.
#[test]
fn a_short_window_scrolls_the_right_column_without_overlap() {
    for (w, height) in [(1600.0, 988.0), (1366.0, 728.0), (800.0, 500.0)] {
        let h = Harness::sized(w, height);
        let right = h.panel(RIGHT);
        let clash = h.overlapping_texts(right);
        assert!(clash.is_empty(), "{w} × {height}: {clash:?}");
        let head = in_right(&h, "Legend");
        assert!(head.top() < right.top() + 36.0, "{w} × {height}: the legend at the top: {head:?}");
    }
    let mut h = Harness::sized(800.0, 500.0);
    let right = h.panel(RIGHT);
    h.wheel(h.rect(wid("weight", "0.1")).unwrap().center(), vec2(0.0, -4000.0));
    // The pointer goes on to the canvas, so no tooltip of what scrolled under it floats there.
    h.frame(vec![Event::PointerMoved(pos2(380.0, 300.0))]);
    h.settle();
    let fold = h.rect(wid("props-fold", ())).expect("the properties' ▸");
    assert!(right.contains_rect(fold), "{fold:?} in {right:?}");
    let clash = h.overlapping_texts(right);
    assert!(clash.is_empty(), "scrolled to the properties: {clash:?}");
    assert!(h.rect(wid("palette-more", ())).unwrap().bottom() < fold.top(), "the pen above the properties");
    h.click(wid("props-fold", ()));
    assert!(h.texts.iter().any(|p| p.text == "Select something to see its size and where it is"), "{}", h.line_in(right));
}

/// However short the window, the legend's list keeps to its own room: no row runs over the pen
/// under it, and scrolled to its end the list shows the bottom layer whole.
#[test]
fn the_legend_list_stays_above_the_pen() {
    for (w, height) in [(1280.0, 662.0), (800.0, 500.0)] {
        let mut h = Harness::sized(w, height);
        for i in 0..12 {
            h.app.run("layer.new", json!({ "name": format!("Pit {i}") })).unwrap();
        }
        h.settle();
        let names: Vec<String> = h.app.session.active().unwrap().doc.layers.iter().map(|l| l.display_name()).collect();
        // Down the list to its end, from between its head and the pen.
        let head = in_right(&h, "Legend");
        let line = h.rect(wid("palette-more", ())).unwrap().top() - 7.0;
        h.wheel(pos2(head.center().x, (head.bottom() + line) / 2.0), vec2(0.0, -4000.0));
        let line = h.rect(wid("palette-more", ())).unwrap().top() - 7.0;
        let rows: Vec<&Painted> = h.texts.iter().filter(|p| names.contains(&p.text) && p.shown.is_positive()).collect();
        for p in &rows {
            assert!(p.shown.bottom() <= line, "{w} × {height}: {:?} over the pen: {:?}, its line at {line}", p.text, p.shown);
        }
        let bottom = rows.iter().find(|p| Some(&p.text) == names.first()).expect("the bottom layer in view");
        assert!(close(bottom.shown, bottom.rect), "{w} × {height}: the bottom layer cut: {:?} of {:?}", bottom.shown, bottom.rect);
    }
}

/// The tab strip in a narrow window, or with a long name, never draws a tab over another or over
/// ＋, and nothing at its right (design 3.0 §5 ⑥: no drawing name, no "Not saved"): the current
/// tab's name is cut short with "…" when even it doesn't fit.
#[test]
fn the_tabs_never_overlap() {
    let long = "Excavation area 3, trench 12, north section, second season";
    // The window, the drawing opened after "Site 1" (the current one), whether its name is cut.
    let cases = [(1600.0, 988.0, None, false), (1366.0, 728.0, Some(long), false), (800.0, 500.0, Some(long), true)];
    for (w, height, other, cut) in cases {
        let mut h = Harness::sized(w, height);
        if let Some(name) = other {
            h.app.run("file.new", json!({"name": name, "width": 400, "height": 300})).unwrap();
        }
        let name = other.unwrap_or("Site 1");
        h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
        h.settle();
        let (set, right, ribbon) = (h.panel(SET), h.panel(RIGHT), h.panel(RIBBON));
        let strip = r(set.right(), ribbon.bottom(), right.left(), ribbon.bottom() + 34.0);
        let clash = h.overlapping_texts(strip);
        assert!(clash.is_empty(), "{w} {name}: {clash:?}");
        let docs = h.app.session.documents().len();
        let mut tabs: Vec<Rect> = (0..docs).filter_map(|i| h.rect(wid("tab", i))).collect();
        tabs.push(h.rect(wid("tab-new", ())).expect("＋"));
        for (i, a) in tabs.iter().enumerate() {
            assert!(strip.contains_rect(*a), "{w} {name}: {a:?} in {strip:?}");
            for b in tabs.iter().skip(i + 1) {
                assert!(a.intersect(*b).width() <= 0.0, "{w} {name}: {a:?} over {b:?}");
            }
        }
        assert!(h.rect(wid("tab-home", ())).is_none(), "{w}: no Home tab");
        // The current tab shows its name, inside it, cut short only when it doesn't fit.
        let tab = h.rect(wid("tab", h.app.session.active_index().unwrap())).unwrap();
        let elided: Vec<bool> = h
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) if t.galley.text() == name && tab.expand(0.5).contains_rect(t.visual_bounding_rect()) => Some(t.galley.elided),
                _ => None,
            })
            .collect();
        assert_eq!(elided, [cut], "{w} {name}: the current tab's name");
        let shown = h.texts_in(strip);
        assert!(!shown.iter().any(|s| s == "Not saved"), "{w} {name}: {shown:?}");
        assert_eq!(shown.iter().filter(|s| *s == name).count(), 1, "{w} {name}: once, in its tab: {shown:?}");
    }
}

/// The tool column as in the mockup (design 3.0 §4 `.a-tools`, §5 ④): 50 wide, buttons 38 × 32
/// two apart, a line 4 above and under between groups, 「고고학」 over the archaeology tools 1–4,
/// the keys at the buttons' bottom right; no fill and stroke at its foot.
#[test]
fn tool_column_has_the_mockup_groups() {
    let mut h = Harness::new();
    let column = h.panel(TOOLS);
    assert_eq!(column.width(), 50.0);
    let rects: Vec<Rect> = SLOTS.iter().map(|p| h.rect(wid("tool", p)).unwrap_or_else(|| panic!("no tool {p}"))).collect();
    // The buttons' tops below the column's, measured in the mockup at 1600 × 988.
    let tops = [8.0, 42.0, 87.0, 121.0, 155.0, 189.0, 223.0, 268.0, 302.0, 336.0, 370.0, 404.0, 466.0, 500.0, 534.0, 568.0, 613.0, 647.0];
    for ((p, rect), top) in SLOTS.iter().zip(&rects).zip(tops) {
        assert_eq!(rect.size(), vec2(38.0, 32.0), "{p}");
        assert_eq!((rect.left() - column.left(), rect.top() - column.top()), (5.5, top), "{p}");
    }
    // The lines between the groups: after V A, after T, after I and after 4.
    let lines: Vec<f32> = h
        .segments_in(column)
        .iter()
        .filter(|(p, _)| (p[0].y - p[1].y).abs() < 0.1 && (p[1].x - p[0].x).abs() > 20.0)
        .map(|(p, _)| p[0].y - column.top())
        .collect();
    assert_eq!(lines, [80.5, 261.5, 442.5, 606.5], "{lines:?}");
    // 「고고학」 over the archaeology tools; their keys 1–4, as the other tools' letters.
    let caption = h.texts.iter().find(|p| p.text == "Archaeology" && column.contains_rect(p.rect)).expect("the caption");
    assert!(caption.rect.top() > rects[11].bottom() && caption.rect.bottom() < rects[12].top(), "{:?}", caption.rect);
    // 10 points; English's longer word smaller, to fit the 50 points.
    assert!(caption.font.size <= 10.0 && caption.font.size >= 7.0, "{:?}", caption.font);
    for (tool, key) in ARCH.iter().zip(["1", "2", "3", "4"]).chain([(&"pen", "P"), (&"zoom", "Z")]) {
        let r = h.rect(wid("tool", tool)).unwrap();
        assert_eq!(h.texts_in(r), [key], "{tool}");
        assert_eq!(h.font_of(key, r), Some(mono(9.0)), "{tool}");
    }
    // The Section Line's two marks in the section marks' red.
    let t = h.tokens();
    let marks = h.rects_in(rects[12]).into_iter().filter(|(_, fill, _)| *fill == t.section_mark).count();
    assert_eq!(marks, 2);
    // No fill and stroke, no Clear button (the pen has the colours).
    assert!(h.rect(wid("proxy", ()).with("fill-proxy")).is_none() && h.rect(wid("proxy-none", ())).is_none());
    assert!(h.rect(wid("arch", "spot-height")).is_none() && h.rect(wid("arch", "hachure")).is_none(), "the old buttons left");

    h.click(wid("tool", "pen"));
    assert_eq!(h.app.session.tool_id(), "pen");
    // The Pen slot keeps the tool last used in it.
    h.app.select_tool("curvature");
    h.settle();
    h.click(wid("tool", "selection"));
    assert_eq!(h.app.session.tool_id(), "selection");
    h.click(wid("tool", "pen"));
    assert_eq!(h.app.session.tool_id(), "curvature");
    // A right click opens the slot's flyout; the Section Line's has Slope Hachure.
    let at = h.rect(wid("tool", "pen")).unwrap().center();
    h.press(at, PointerButton::Secondary);
    assert!(h.app.ui.flyout.is_some(), "the Pen slot's flyout");
    h.app.ui.flyout = None;
    h.settle();
    h.press(rects[12].center(), PointerButton::Secondary);
    let flyout = h.ctx.data(|d| d.get_temp::<Vec<String>>(egui::Id::new("flyout-tools")));
    assert_eq!(flyout, Some(vec!["sectionCut".to_string(), "slopeHachure".to_string()]));
    // While it is open the slot's tooltip doesn't cover it (review J024 UI N2).
    h.rest_at(rects[12].center());
    assert!(h.app.ui.flyout.is_some());
    assert!(!h.texts.iter().any(|p| p.text.contains("Click two points")), "no tooltip over the flyout");
    h.app.ui.flyout = None;
    h.settle();
    // The archaeology tools are tools: a click picks one, and the slot shows it chosen.
    for tool in ARCH {
        h.click(wid("tool", tool));
        assert_eq!(h.app.session.tool_id(), tool);
        assert_eq!(h.fill_at(h.rect(wid("tool", tool)).unwrap()), Some(t.tool_active), "{tool}");
    }
    // Their tooltips say what they do.
    h.rest_at(rects[13].center());
    assert!(h.texts.iter().any(|p| p.text.starts_with("Level Lines (2)\nClick the ground line")), "the tooltip");
}

#[test]
fn drawing_set_lists_open_documents_and_switches() {
    let mut h = Harness::new();
    h.app.run("file.new", json!({"name": "Site 2", "width": 200, "height": 100})).unwrap();
    h.settle();
    let set = h.panel(SET);
    let line = h.line_in(set);
    assert!(line.contains("Drawing Set") && line.contains("Site 1") && line.contains("Site 2"), "{line}");
    // The scale and coordinate system are the status bar's (design 3.0 §5 ⑤): no footer here.
    assert!(!line.contains("Scale not set") && !line.contains("This drawing"), "{line}");
    assert!(h.rect(wid("set-foot", ())).is_none());
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

/// The tabs switch drawings; one with changes not saved has a clay dot by its name ("Not saved" on
/// hover), the only place that shows it besides the caption's *; × closes a tab, ＋ is a new drawing.
#[test]
fn tabs_switch_documents_and_show_unsaved() {
    let mut h = Harness::new();
    h.app.run("file.new", json!({"name": "Site 2", "width": 200, "height": 100})).unwrap();
    h.settle();
    let strip = r(272.0, 146.0, 1296.0, 180.0);
    let line = h.line_in(strip);
    assert!(line.contains("Site 1") && line.contains("Site 2") && !line.contains("Home"), "{line}");
    assert!(h.rect(wid("tab-unsaved", 0usize)).is_none() && h.rect(wid("tab-unsaved", 1usize)).is_none());
    h.click(wid("tab", 0usize));
    assert_eq!(h.app.session.active_index(), Some(0));
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    h.settle();
    let dot = h.rect(wid("tab-unsaved", 0usize)).expect("Site 1's dot");
    let tab = h.rect(wid("tab", 0usize)).unwrap();
    assert!(tab.contains_rect(dot), "{dot:?} in {tab:?}");
    let t = h.tokens();
    let dots = h.shapes.iter().filter(|c| matches!(&c.shape, egui::Shape::Circle(s) if tab.contains(s.center) && s.fill == t.action)).count();
    assert_eq!(dots, 1, "a clay dot");
    assert!(!h.line_in(strip).contains("Not saved"), "{}", h.line_in(strip));
    assert!(!h.status_line().contains("not saved"), "{}", h.status_line());
    h.click(wid("tab", 1usize));
    assert_eq!(h.app.session.active_index(), Some(1));
    // The × of a tab closes it (Site 2 has no changes: no question).
    h.hover(wid("tab", 1usize));
    h.click(wid("tab-close", 1usize));
    assert_eq!(h.app.session.documents().len(), 1);
    // ＋ is a new drawing.
    h.click(wid("tab-new", ()));
    assert!(h.app.ui.dialog.is_some(), "the New dialog");
}

/// Over the canvas float only the zoom buttons at the bottom right, 34 × 30 each, 14 points in
/// (design 3.0 §5 ⑥); the tool, layer, hint and zoom readout are the status bar's. Presses on them
/// don't reach the tool under them.
#[test]
fn canvas_overlays_only_the_zoom_buttons() {
    let mut h = Harness::new();
    h.app.select_tool("selection");
    h.settle();
    let canvas = h.app.canvas_rect.unwrap();
    assert!(h.texts_in(canvas).iter().all(|s| s == "Fit"), "only the zoom buttons: {:?}", h.texts_in(canvas));
    let zin = h.rect(wid("zoom", "in")).unwrap();
    assert_eq!(zin.size(), vec2(34.0, 30.0));
    let fit = h.rect(wid("zoom", "fit")).unwrap();
    assert!((canvas.right() - fit.right() - 14.0).abs() < 0.5 && (canvas.bottom() - fit.bottom() - 14.0).abs() < 0.5, "{fit:?} in {canvas:?}");
    for gone in ["band", "readout"] {
        assert!(h.rect(wid(gone, ())).is_none(), "{gone}");
    }
    h.app.select_tool("pen");
    h.settle();
    let line = h.line_in(canvas);
    assert!(!line.contains("Pen · Layer 1") && !line.contains("to add a corner point") && !line.contains('%'), "{line}");
    let zoom = |h: &Harness| h.app.view().unwrap().zoom;
    let z0 = zoom(&h);
    let nodes = |h: &Harness| h.app.session.active().unwrap().doc.node_count();
    let n = nodes(&h);
    h.click(wid("zoom", "in"));
    assert!(zoom(&h) > z0);
    h.click(wid("zoom", "out"));
    h.click(wid("zoom", "out"));
    assert!(zoom(&h) < z0);
    h.click(wid("zoom", "fit"));
    assert!((zoom(&h) - z0).abs() < 1e-6, "back to the fitted view");
    assert_eq!(nodes(&h), n, "no point drawn under the zoom buttons");
}

/// The tool's hint reads plainly, as in the mockup: no bold keys, its parts apart by "·", in the
/// status bar only (the band on the canvas is gone, design 3.0 §5 ⑥).
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
    assert!(hint(&h, h.app.canvas_rect.unwrap()).is_empty(), "nothing on the canvas");
    let status = hint(&h, h.panel(STATUS));
    assert_eq!(status.len(), 1, "{status:?}");
    let (text, bold) = &status[0];
    assert!(!bold, "{status:?}");
    assert_eq!(text, "Click to add a corner point · Drag to add a smooth point · Click the first point to close · Enter to finish");
}

#[test]
fn palette_weight_chip_sets_the_next_stroke() {
    let mut h = Harness::new();
    let t = h.tokens();
    let right = h.panel(RIGHT);
    for w in ["0.1", "0.15", "0.2", "0.3", "0.4", "0.5", "0.6", "0.8", "1.0"] {
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
    // A hatch on the selected rectangle leaves its weight.
    h.click(wid("hatch", "45"));
    assert_eq!(h.app.session.shown_stroke().unwrap().width, 0.8);
    assert!(seg_on(&h, "hatch", "45"), "hatch 45 is on");
    // ⋯ pops the Stroke panel out.
    h.click(wid("palette-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("stroke"));
}

#[test]
fn legend_rows_show_hide_lock_and_pick_the_layer() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let line = NodeId(h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap()["id"].as_u64().unwrap());
    let pits = NodeId(h.app.run("layer.new", json!({"name": "Pits"})).unwrap()["id"].as_u64().unwrap());
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    let right = h.panel(RIGHT);
    let (top, below) = (h.rect(wid("legend-row", pits.0)).unwrap(), h.rect(wid("legend-row", first.0)).unwrap());
    assert!(right.contains_rect(top) && top.bottom() <= below.top(), "the top layer first");
    assert_eq!(top.height(), 27.0);
    let layer = |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().clone();
    h.click(wid("legend-eye", first.0));
    assert!(!layer(&h, first).visible);
    h.click(wid("legend-eye", first.0));
    assert!(layer(&h, first).visible);
    let t = h.tokens();
    let lock_col = h.rect(wid("legend-lock", first.0)).unwrap();
    let filled = |h: &Harness| h.rects_in(lock_col).iter().any(|(_, fill, _)| *fill == t.icon);
    assert!(!filled(&h), "open: an outline padlock");
    h.click(wid("legend-lock", first.0));
    assert!(layer(&h, first).locked);
    assert!(filled(&h), "locked: a filled padlock (design 3.0 §7)");
    h.click(wid("legend-lock", first.0));
    assert!(!layer(&h, first).locked);
    // Alt and the eye: this row alone, then every row again.
    h.alt_click(wid("legend-eye", first.0));
    assert!(layer(&h, first).visible && !layer(&h, pits).visible, "Layer 1 alone");
    h.alt_click(wid("legend-eye", first.0));
    assert!(layer(&h, first).visible && layer(&h, pits).visible, "every row again");
    assert_eq!(h.current_layer(), Some(pits));
    h.click(wid("legend-row", first.0));
    assert_eq!(h.current_layer(), Some(first));
    // The count at the right selects the row's objects.
    assert_eq!(h.texts_in(h.rect(wid("legend-count", first.0)).unwrap()), ["1"]);
    h.click(wid("legend-count", first.0));
    assert_eq!(h.app.session.active().unwrap().selection.objects, [line]);
    assert!(h.status_line().contains("Selected: 1"), "{}", h.status_line());
    // ＋ adds a row and opens its editor.
    let n = h.layer_ids().len();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.click(wid("legend-new", ()));
    assert_eq!(h.layer_ids().len(), n + 1);
    assert!(h.rect(wid("row-card", ())).is_some(), "its editor opens");
    let line = h.line_in(right);
    assert!(line.contains("Legend") && line.contains("Pits"), "{line}");
}

/// A locked or hidden layer takes no new lines (the engine draws on the topmost layer that is shown
/// and unlocked instead), so when the current one is, the status bar and the legend say where lines
/// go rather than naming a layer they won't land in.
#[test]
fn a_locked_or_hidden_current_layer_shows_where_lines_go() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let pits = NodeId(h.app.run("layer.new", json!({"name": "Pits"})).unwrap()["id"].as_u64().unwrap());
    h.app.select_tool("pen");
    h.settle();
    let (bar, right) = (h.panel(STATUS), h.panel(RIGHT));
    let t = h.tokens();
    assert!(!h.line_in(bar).contains("Pits is"), "{}", h.line_in(bar));
    assert_eq!(h.color_of("Pits", right), Some(t.text));
    h.click(wid("legend-lock", pits.0));
    assert_eq!(h.current_layer(), Some(pits), "still the current layer");
    let note = "Pits is locked: drawing on Layer 1";
    assert!(h.line_in(bar).contains(note), "{}", h.line_in(bar));
    assert_eq!(h.color_of(note, bar), Some(t.clay_text));
    assert_eq!(h.color_of("Pits", right), Some(t.clay_text), "the legend marks the current row");
    let id = h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap()["id"].as_u64().unwrap();
    assert_eq!(h.app.session.active().unwrap().doc.layer_containing(NodeId(id)), Some(first), "where the status bar said");
    // Hidden likewise.
    h.app.run("layer.setCurrent", json!({ "id": pits.0 })).unwrap();
    h.click(wid("legend-lock", pits.0));
    h.click(wid("legend-eye", pits.0));
    assert_eq!(h.current_layer(), Some(pits));
    assert!(h.line_in(bar).contains("Pits is hidden: drawing on Layer 1"), "{}", h.line_in(bar));
    // With every layer locked or hidden nothing is drawn, and the status bar says so.
    h.click(wid("legend-lock", first.0));
    assert!(h.line_in(bar).contains("Pits is hidden: no layer to draw on"), "{}", h.line_in(bar));
    assert!(h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).is_err());
}

/// A legend of hundreds of rows lays out only the rows in view (the rest scroll in), so it doesn't
/// slow every frame down.
#[test]
fn a_long_legend_lays_out_only_the_rows_in_view() {
    let mut h = Harness::new();
    let ids: Vec<NodeId> =
        (0..200).map(|i| NodeId(h.app.run("layer.new", json!({ "name": format!("Pit {i}") })).unwrap()["id"].as_u64().unwrap())).collect();
    h.settle();
    let rows = |h: &Harness| ids.iter().filter(|id| h.rect(wid("legend-row", id.0)).is_some()).count();
    let laid_out = rows(&h);
    assert!((5..60).contains(&laid_out), "{laid_out} of {} rows laid out", ids.len());
    // The top one (the last made) shows; the wheel brings the bottom one (the first made).
    let right = h.panel(RIGHT);
    let (top, bottom) = (*ids.last().unwrap(), ids[0]);
    assert!(right.contains_rect(h.rect(wid("legend-row", top.0)).unwrap()));
    assert!(h.rect(wid("legend-row", bottom.0)).is_none());
    h.wheel(h.rect(wid("legend-row", top.0)).unwrap().center(), vec2(0.0, -1.0e6));
    let row = h.rect(wid("legend-row", bottom.0)).expect("the bottom row, scrolled in");
    assert!(right.contains_rect(row), "{row:?}");
    assert!((5..60).contains(&rows(&h)), "scrolled: {} laid out", rows(&h));
    // A click on it draws with it.
    h.click(wid("legend-row", bottom.0));
    assert_eq!(h.current_layer(), Some(bottom));
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
    // The same message again shows again: a second save says so too.
    h.app.status("Saved Site 1");
    h.settle();
    assert!(h.line_in(bar).contains("Saved Site 1"), "the same message again: {}", h.line_in(bar));
    h.later(7.0);
    // So does the same error twice: the status bar is where it reaches the user.
    for time in ["first", "second"] {
        assert!(h.app.run("layer.setCurrent", json!({ "id": 999_999 })).is_err());
        h.settle();
        let line = h.line_in(bar);
        assert!(!line.contains("to add a corner point"), "the error, the {time} time: {line}");
        h.later(7.0);
    }
    // Align (J008): what it is and when.
    h.click(wid("tile", "align"));
    assert!(h.line_in(bar).contains("Align: coming soon (J008)"), "{}", h.line_in(bar));
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
    // With a template layer holding a scan: the tile shows it on and hides it, then shows it again.
    let id = h.app.run("layer.new", json!({"name": "Scan"})).unwrap()["id"].as_u64().unwrap();
    draw_rect(&mut h, 10.0);
    h.app.run("layer.setProps", json!({"ids": [id], "template": true})).unwrap();
    h.settle();
    let t = h.tokens();
    let tile = h.rect(wid("tile", "underlay")).unwrap();
    let well = Rect::from_min_size(pos2(tile.center().x - 23.0, tile.top()), vec2(46.0, 40.0));
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
    h.click(wid("palette-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("stroke"));
    let right = h.panel(RIGHT);
    let flyout = h.ctx.memory(|m| m.area_rect(egui::Id::new("icon-panel"))).expect("the panel popped out");
    assert!((flyout.right() - (right.left() - 6.0)).abs() < 1.0 && (flyout.top() - (right.top() + 6.0)).abs() < 1.0, "{flyout:?} by {right:?}");
    h.click_at(pos2(right.left() + 12.0, right.top() + 8.0));
    assert!(h.app.ui.open_panel.is_none(), "put away");
}

/// Snap's ▾ in the status bar lists the tolerance, 1 to 8 px, and a pick sets it.
#[test]
fn snap_tolerance_is_picked_from_a_list() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    assert!(h.status_line().contains("Snap 2px"), "{}", h.status_line());
    let snap = h.app.ui.view.snap_to_point;
    h.click(wid("status-more", "snap"));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 8, "{items:?}");
    h.click_at(items[3].0.center());
    assert_eq!(h.app.session.prefs.snap_to_point_tolerance, 4.0);
    assert!(h.status_line().contains("Snap 4px"), "{}", h.status_line());
    assert_eq!(h.app.ui.view.snap_to_point, snap, "the ▾ doesn't turn Snap on or off");
    h.click(wid("status", "snap"));
    assert_eq!(h.app.ui.view.snap_to_point, !snap, "the rest of the cell does");
}

/// In the smallest window the status bar's cells keep apart and in reach, the hint giving way.
#[test]
fn status_bar_keeps_its_cells_in_a_small_window() {
    let mut h = Harness::sized(800.0, 500.0);
    h.app.select_tool("pen");
    h.app.run("doc.setScale", json!({"scale": 30.0, "epsg": 5186})).unwrap();
    h.settle();
    let bar = h.panel(STATUS);
    let clash = h.overlapping_texts(bar);
    assert!(clash.is_empty(), "{clash:?}");
    for (kind, key) in [("status", "grid"), ("status", "snap"), ("status", "position"), ("status", "scale"), ("status", "zoom")] {
        let c = h.rect(wid(kind, key)).unwrap_or_else(|| panic!("{key}"));
        assert!(bar.contains_rect(c), "{key}: {c:?} in {bar:?}");
    }
    let snap = h.app.ui.view.snap_to_point;
    h.click(wid("status", "snap"));
    assert_eq!(h.app.ui.view.snap_to_point, !snap);
}

/// The properties (design 3.0 §5 ⑨): folded to their head while nothing is selected, open by
/// themselves when something is, with the selection and its row in the head, its size in metres
/// at the drawing's scale (30 mm on paper at 1:30 is 0.90 m), a closed shape's area and
/// perimeter, where it is in TM metres and, with two selected, the align buttons. ▸ and ▾ fold
/// and open them.
#[test]
fn the_properties_fold_and_show_real_sizes() {
    let mut h = Harness::new();
    let right = h.panel(RIGHT);
    assert_eq!(line_of(&h, "Properties", right), ["Properties", "No selection"]);
    assert!(h.rect(wid("props-fold", ())).is_some() && !h.texts_in(right).iter().any(|s| s == "Size"), "folded");
    h.click(wid("props-fold", ()));
    assert!(h.texts_in(right).iter().any(|s| s == "Select something to see its size and where it is"), "{}", h.line_in(right));
    h.click(wid("props-fold", ()));
    assert!(!h.texts_in(right).iter().any(|s| s == "Select something to see its size and where it is"), "folded again");
    // 30 × 60 mm on paper at 1:30.
    h.app.run("doc.setScale", json!({"scale": 30.0, "epsg": 5186})).unwrap();
    let a = h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 85.03937, "height": 170.07874})).unwrap()["id"].clone();
    h.app.run("select.set", json!({ "ids": [a] })).unwrap();
    h.settle();
    let right = h.panel(RIGHT);
    assert_eq!(line_of(&h, "Properties", right), ["Properties", "1 line · Layer 1"]);
    assert_eq!(line_of(&h, "Size", right), ["Size", "0.90 × 1.80 m"]);
    assert_eq!(line_of(&h, "Area", right), ["Area", "1.62 ㎡ · perimeter 5.40 m"]);
    // Where it is: its centre in TM metres, in the drawing's axis order, grouped as the status bar.
    let tm = |h: &Harness| {
        let st = h.app.session.active().unwrap();
        let b = st.doc.bounds_of(&st.selection.objects, true).unwrap();
        st.doc.scale.as_ref().unwrap().canvas_to_world(b.center())
    };
    let (east, north) = tm(&h);
    let said = |x: f64, y: f64| format!("X {} · Y {} m", super::status::grouped(x, 2), super::status::grouped(y, 2));
    assert_eq!(line_of(&h, "Position", right), ["Position", said(east, north).as_str()]);
    h.app.run("doc.setScale", json!({"scale": 30.0, "epsg": 5186, "order": "northeast"})).unwrap();
    h.settle();
    let (east, north) = tm(&h);
    assert_eq!(line_of(&h, "Position", right), ["Position", said(north, east).as_str()], "north first, as the drawing reads");
    assert!(h.rect(wid("props-align", "left")).is_none(), "one selected: nothing to align");
    // Two selected: no area, the align buttons.
    let b = h.app.run("shape.rectangle", json!({"x": 200, "y": 40, "width": 20, "height": 20})).unwrap()["id"].clone();
    h.app.run("select.set", json!({ "ids": [a, b] })).unwrap();
    h.settle();
    assert!(!h.texts_in(right).iter().any(|s| s == "Area"), "{}", h.line_in(right));
    for key in ["left", "center", "right", "top", "middle", "bottom"] {
        assert!(right.contains_rect(h.rect(wid("props-align", key)).unwrap_or_else(|| panic!("{key}"))), "{key}");
    }
    h.click(wid("props-align", "left"));
    let left =
        |h: &Harness, id: &serde_json::Value| h.app.session.active().unwrap().doc.bounds_of(&[NodeId(id.as_u64().unwrap())], false).unwrap().x0;
    assert!((left(&h, &a) - left(&h, &b)).abs() < 1e-6, "lined up at the left");
    // ▾ folds them while the selection stays; nothing selected and then something again, they
    // follow the selection.
    h.click(wid("props-fold", ()));
    assert!(!h.texts_in(right).iter().any(|s| s == "Size"), "folded by hand");
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.app.run("select.set", json!({ "ids": [a] })).unwrap();
    h.settle();
    assert!(h.texts_in(right).iter().any(|s| s == "Size"), "open again by themselves");
}

/// The status bar of design 3.0 (§5 ⑩): at the right the grid, Snap, where the pointer is, the
/// scale and coordinate system and the zoom; no tool name and no count of drawings not saved.
/// Without a coordinate system the position is millimetres on paper; a click copies it.
#[test]
fn status_bar_shows_grid_snap_position_scale_and_zoom() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let bar = h.panel(STATUS);
    assert_eq!(bar.height(), 28.0);
    let line = h.line_in(bar);
    let zoom = super::status::percent(h.app.view().unwrap().zoom);
    for want in ["Grid", "Snap 2px", "X", "· Y", "Scale not set", zoom.as_str()] {
        assert!(line.contains(want), "{want} in {line}");
    }
    let cells: Vec<f32> = ["grid", "snap", "position", "scale", "zoom"].iter().map(|k| h.rect(wid("status", k)).unwrap().left()).collect();
    assert!(cells.windows(2).all(|w| w[0] < w[1]), "grid, snap, position, scale, zoom: {cells:?}");
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    // The pointer over the canvas: its position on paper; it stays when the pointer leaves.
    let at = h.app.canvas_rect.unwrap().center();
    h.frame(vec![Event::PointerMoved(at)]);
    h.settle();
    let p = h.app.hover_doc.expect("over the canvas");
    let mm = vectorcraft_doc::Unit::Millimeters;
    let (x, y) = (mm.from_pt(p.x), mm.from_pt(p.y));
    let line = h.line_in(bar);
    assert!(!line.contains("Pen") && !line.contains("not saved"), "{line}");
    let shown = format!("X {} · Y {} mm", super::status::grouped(x, 1), super::status::grouped(y, 1));
    assert!(line.contains(&shown), "{shown} in {line}");
    h.click(wid("status", "position"));
    assert!(h.status_line().contains("Coordinates copied"), "{}", h.status_line());
    assert_eq!(h.copied.last(), Some(&format!("{x:.1}, {y:.1}")), "the numbers, ready to paste");
}

/// The grid's and the zoom's cells (design 3.0 §5 ⑩): a click on the grid shows or hides it, its ▾
/// lists 10, 20 and 50 mm and the colours with their swatches, what the grid has now marked; the
/// zoom lists 50 to 400 % and Fit.
#[test]
fn the_grid_and_zoom_lists_show_and_set_what_they_offer() {
    let mut h = Harness::new();
    h.settle();
    let screen = Rect::from_min_size(Pos2::ZERO, h.screen);
    let grid = h.app.ui.view.grid;
    h.click(wid("status", "grid"));
    assert_eq!(h.app.ui.view.grid, !grid);
    h.click(wid("status", "grid"));
    assert_eq!(h.app.ui.view.grid, grid);
    if h.app.ui.view.grid {
        h.click(wid("status", "grid"));
    }
    // A new install's grid: every 72 points (25.4 mm) in #C8C8C8, neither on the list, so both
    // show as what it is now, not to be pressed.
    h.click(wid("status-more", "grid"));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 10, "three spacings, five colours and the two it has now: {items:?}");
    assert_eq!(items.iter().filter(|(_, enabled)| !enabled).count(), 2, "{items:?}");
    let shown = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("|");
    for want in ["10 mm", "20 mm", "50 mm", "Current 25.4 mm", "Light grey", "Ochre", "Sky", "Graph green", "Dark grey", "Current"] {
        assert!(shown.contains(want), "{want} in {shown}");
    }
    let swatches =
        h.rects_in(screen).into_iter().filter(|(r, ..)| r.width() == 12.0 && r.height() == 12.0).map(|(_, fill, _)| fill).collect::<Vec<_>>();
    for c in [0xcc_cc_cc, 0xc8_bf_a8, 0x6b_ae_d6, 0x74_c4_76, 0x77_77_77, 0xc8_c8_c8] {
        let [_, r, g, b] = u32::to_be_bytes(c);
        assert!(swatches.contains(&egui::Color32::from_rgb(r, g, b)), "{c:06x} in {swatches:?}");
    }
    // A spacing picked: set, the grid shown, and marked on the list; Current goes.
    h.click_text("20 mm", screen);
    assert!((h.app.session.prefs.gridline_every - 20.0 * 72.0 / 25.4).abs() < 1e-6, "{}", h.app.session.prefs.gridline_every);
    assert!(h.app.ui.view.grid, "a grid set up is a grid to see");
    h.click(wid("status-more", "grid"));
    assert_eq!(h.popup_buttons().len(), 9);
    h.click_text("Sky", screen);
    assert_eq!(h.app.session.prefs.grid_color.to_uppercase(), "#6BAED6");
    assert!(h.app.ui.view.grid, "still shown: a second pick doesn't hide it");
    h.click(wid("status-more", "grid"));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 8, "{items:?}");
    assert!(items.iter().all(|(_, enabled)| *enabled));
    h.key(egui::Key::Escape);
    // The zoom: 50, 100, 200 and 400 %, then Fit.
    let zoom = |h: &Harness| h.app.view().unwrap().zoom;
    for (i, z) in [0.5, 1.0, 2.0, 4.0].into_iter().enumerate() {
        h.click(wid("status", "zoom"));
        let items = h.popup_buttons();
        assert_eq!(items.len(), 5, "{items:?}");
        h.click_at(items[i].0.center());
        assert!((zoom(&h) - z).abs() < 1e-6, "{z}: {}", zoom(&h));
        assert!(h.status_line().contains(&super::status::percent(z)), "{}", h.status_line());
    }
    h.click(wid("status", "zoom"));
    let items = h.popup_buttons();
    h.click_at(items[4].0.center());
    let fitted = zoom(&h);
    h.app.run("view.fitArtboard", json!({})).unwrap();
    assert!((zoom(&h) - fitted).abs() < 1e-6 && (fitted - 4.0).abs() > 1e-6, "Fit: {fitted}");
}

/// With a scale and coordinate system the status bar shows them ("1:20 · EPSG:5186") and TM
/// coordinates in metres, grouped in threes, in the drawing's axis order; its ▾ lists the scales
/// (the coordinate system and axis order stay) and the dialog. A click copies the two numbers.
#[test]
fn status_bar_shows_scale_and_tm_coordinates() {
    let mut h = Harness::new();
    h.settle();
    let bar = h.panel(STATUS);
    assert!(h.line_in(bar).contains("Scale not set"));

    // Set scale 1:20, north first, and bind origin to CP1 (152840.123, 512340.456)
    h.app.run("doc.setScale", json!({"scale": "1:20", "epsg": 5186, "order": "northeast"})).unwrap();
    h.app.run("doc.setOrigin", json!({ "canvas": [100.0, 100.0], "world": [152840.123, 512340.456] })).unwrap();
    h.settle();
    let line = h.line_in(bar);
    assert!(line.contains("1:20 · EPSG:5186"), "{line}");

    // Hover over canvas -> live TM coordinates readout, north first
    let at = h.app.canvas_rect.unwrap().center();
    h.frame(vec![Event::PointerMoved(at)]);
    h.settle();
    let p = h.app.hover_doc.expect("over the canvas");
    let cfg = h.app.session.active().unwrap().doc.scale.clone().unwrap();
    let (east, north) = cfg.canvas_to_world(p);
    let shown = format!("X {} · Y {} m", super::status::grouped(north, 2), super::status::grouped(east, 2));
    assert!(shown.contains("512 3") && shown.contains("152 8"), "{shown}");
    let line = h.line_in(bar);
    assert!(line.contains(&shown), "{shown} in {line}");
    h.click(wid("status", "position"));
    assert_eq!(h.copied.last(), Some(&format!("{north:.2}, {east:.2}")));

    // Pick 1:50 from the list: the coordinate system and axis order stay.
    h.click(wid("status", "scale"));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 10, "nine scales and the dialog: {items:?}");
    h.click_text("1:50", Rect::from_min_size(Pos2::ZERO, h.screen));
    let cfg = h.app.session.active().unwrap().doc.scale.clone().unwrap();
    assert_eq!((cfg.scale.denom, cfg.crs.epsg, cfg.crs.order), (50.0, 5186, vectorcraft_doc::scale::CoordOrder::NorthEast));
    assert!(h.line_in(bar).contains("1:50 · EPSG:5186"), "{}", h.line_in(bar));
    // The last item opens the dialog, on 1:50.
    h.click(wid("status", "scale"));
    let items = h.popup_buttons();
    h.click_at(items[9].0.center());
    let t = h.tokens();
    let chip = h.rect(wid("scale-chip", 50u32)).expect("the scale dialog");
    assert_eq!(h.fill_at(chip), Some(t.text), "1:50 chosen");
}

/// Trowel's scale and coordinate system dialog (design 3.0 §5 ③, §9's chips): the nine scales or
/// one's own, the TM belts; nothing changes until OK, which keeps the axis order; Cancel leaves
/// the drawing as it was. A system not on the list stays on offer. Its words are Korean.
#[test]
fn the_scale_dialog_sets_scale_and_coordinate_system() {
    use vectorcraft_doc::scale::CoordOrder;
    let mut h = Harness::new();
    h.app.run("doc.setScale", json!({"scale": 20, "epsg": 5186, "order": "northeast"})).unwrap();
    h.settle();
    h.click(wid("tile", "scale"));
    let t = h.tokens();
    let on = |h: &Harness, id: egui::Id| h.fill_at(h.rect(id).unwrap_or_else(|| panic!("{id:?}"))) == Some(t.text);
    for d in [10_u32, 20, 30, 40, 50, 60, 80, 100, 200] {
        assert_eq!(on(&h, wid("scale-chip", d)), d == 20, "1:{d}");
    }
    for c in [5186_u32, 5185, 5187] {
        assert_eq!(on(&h, wid("crs-chip", c)), c == 5186, "{c}");
    }
    let shown = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("|");
    for want in
        ["Scale and Coordinate System", "Scale", "Other", "Coordinate system", "EPSG:5186 Central Belt", "5185 West", "5187 East", "OK", "Cancel"]
    {
        assert!(shown.contains(want), "{want} in {shown}");
    }
    assert!(!shown.contains("Epsg"), "{shown}");
    let ko = crate::i18n::Lang::from_code("ko").unwrap();
    for (en, kr) in [
        ("Scale", "축척"),
        ("Other", "직접"),
        ("Coordinate system", "좌표계"),
        ("Central Belt", "중부원점"),
        ("West", "서부"),
        ("East", "동부"),
        ("Cancel", "그만두기"),
        ("OK", "확인"),
    ] {
        assert_eq!(crate::i18n::tr_ctx(ko, "trowel", en), kr);
    }
    // Chips picked: nothing set until OK.
    h.click(wid("scale-chip", 30_u32));
    h.click(wid("crs-chip", 5187_u32));
    assert!(on(&h, wid("scale-chip", 30_u32)) && on(&h, wid("crs-chip", 5187_u32)) && !on(&h, wid("scale-chip", 20_u32)));
    let cfg = |h: &Harness| h.app.session.active().unwrap().doc.scale.clone().unwrap();
    assert_eq!((cfg(&h).scale.denom, cfg(&h).crs.epsg), (20.0, 5186));
    h.click(wid("scale-dialog", "ok"));
    assert_eq!((cfg(&h).scale.denom, cfg(&h).crs.epsg, cfg(&h).crs.order), (30.0, 5187, CoordOrder::NorthEast));
    assert!(h.rect(wid("scale-chip", 30_u32)).is_none(), "put away");
    assert!(h.status_line().contains("1:30 · EPSG:5187"), "{}", h.status_line());
    // Cancel: as it was.
    h.click(wid("tile", "scale"));
    h.click(wid("scale-chip", 100_u32));
    h.click(wid("scale-dialog", "cancel"));
    assert_eq!(cfg(&h).scale.denom, 30.0);
    // One's own scale, typed: 1:25.4 is taken as 1:25.
    h.click(wid("tile", "scale"));
    let window = h.ctx.memory(|m| m.area_rect(wid("scale-dialog-window", ()))).expect("the dialog's window");
    let field = h.texts.iter().find(|p| p.text == "30" && window.contains_rect(p.rect)).map(|p| p.rect.center()).expect("the scale field");
    h.click_at(field);
    let all = egui::Modifiers::COMMAND;
    h.frame(vec![Event::Key { key: egui::Key::A, physical_key: None, pressed: true, repeat: false, modifiers: all }]);
    h.frame(vec![Event::Text("25.4".into())]);
    h.key(egui::Key::Enter);
    assert!(!on(&h, wid("scale-chip", 30_u32)), "no chip for 1:25");
    h.click(wid("scale-dialog", "ok"));
    assert_eq!(cfg(&h).scale.denom, 25.0);
    // A system not on the list (the East Sea belt) stays on offer, chosen.
    h.app.run("doc.setScale", json!({"scale": 25, "epsg": 5188, "order": "northeast"})).unwrap();
    h.click(wid("tile", "scale"));
    assert!(on(&h, wid("crs-chip", 5188_u32)), "EPSG:5188 on offer");
    // The chip that is on reads in the panel's colour on ink (in every theme).
    let window = h.ctx.memory(|m| m.area_rect(wid("scale-dialog-window", ()))).expect("the dialog's window");
    assert_eq!(h.color_of("1:25", window), None, "1:25 is typed, no chip");
    assert_eq!(h.color_of("EPSG:5188", window), Some(t.panel));
    // OK with nothing changed: no undo step.
    let undo = |h: &Harness| h.app.session.active().unwrap().history.undo.len();
    let steps = undo(&h);
    h.click(wid("scale-dialog", "ok"));
    assert_eq!((cfg(&h).crs.epsg, undo(&h)), (5188, steps));
    // A scale outside 1 to 5000 (a 1:10 000 map) stays when only the coordinate system changes.
    h.app.run("doc.setScale", json!({"scale": 10000, "epsg": 5186, "order": "northeast"})).unwrap();
    h.click(wid("tile", "scale"));
    h.click(wid("crs-chip", 5185_u32));
    h.click(wid("scale-dialog", "ok"));
    assert_eq!((cfg(&h).scale.denom, cfg(&h).crs.epsg), (10000.0, 5185));
    // Another drawing put in front closes it: OK would set that one's.
    h.click(wid("tile", "scale"));
    h.click(wid("scale-chip", 50_u32));
    h.app.run("file.new", json!({"name": "Site 2", "width": 400, "height": 300})).unwrap();
    h.settle();
    assert!(h.rect(wid("scale-dialog", "ok")).is_none(), "closed with its drawing behind");
    assert!(h.app.session.active().unwrap().doc.scale.is_none(), "Site 2 untouched");
}

/// An error shows in red for five seconds (design 3.0 §5 ⑩); another message in ink for six.
#[test]
fn the_status_bar_shows_errors_in_red_for_five_seconds() {
    let mut h = Harness::new();
    let t = h.tokens();
    let status = h.panel(STATUS);
    h.app.status_error("Could not place it");
    h.settle();
    assert_eq!(h.color_of("Could not place it", status), Some(t.error));
    h.later(4.5);
    assert!(h.status_line().contains("Could not place it"), "{}", h.status_line());
    h.later(1.0);
    assert!(!h.status_line().contains("Could not place it"), "gone after five seconds: {}", h.status_line());
    h.app.status("Lines changed: 3");
    h.settle();
    assert_eq!(h.color_of("Lines changed: 3", status), Some(t.text));
    h.later(5.5);
    assert!(h.status_line().contains("Lines changed: 3"), "{}", h.status_line());
    h.later(1.0);
    assert!(!h.status_line().contains("Lines changed: 3"), "{}", h.status_line());
    // A command that fails, from the menus or the ribbon, is an error.
    crate::menus::invoke(&mut h.app, "no.such.command", json!({}));
    assert!(h.app.ui.status_error, "{}", h.app.ui.status);
}

/// The coordinates' cell keeps its width as the pointer moves, and the last place it showed is
/// that drawing's only.
#[test]
fn the_position_cell_keeps_its_width_and_its_drawing() {
    let mut h = Harness::new();
    let canvas = h.app.canvas_rect.unwrap();
    let width = |h: &Harness| h.rect(wid("status", "position")).unwrap().width();
    let values = |h: &Harness| {
        let cell = h.rect(wid("status", "position")).unwrap();
        h.texts_in(cell)
    };
    h.frame(vec![Event::PointerMoved(canvas.left_top() + vec2(4.0, 4.0))]);
    h.settle();
    let (w1, v1) = (width(&h), values(&h));
    h.frame(vec![Event::PointerMoved(canvas.right_bottom() - vec2(40.0, 40.0))]);
    h.settle();
    let (w2, v2) = (width(&h), values(&h));
    assert_ne!(v1, v2, "two places");
    assert!(v1.iter().zip(&v2).any(|(a, b)| a.len() != b.len()), "values of other lengths: {v1:?} {v2:?}");
    assert!((w1 - w2).abs() < 0.5, "{w1} = {w2}");
    // Off the canvas it keeps the last place; another drawing in front has none.
    let off = h.panel(STATUS).center();
    h.frame(vec![Event::PointerMoved(off)]);
    h.settle();
    assert_eq!(values(&h), v2);
    h.app.run("file.new", json!({"name": "Site 2", "width": 400, "height": 300})).unwrap();
    h.settle();
    assert_eq!(values(&h).iter().filter(|s| *s == "–").count(), 2, "{:?}", values(&h));
}

/// The status bar's numbers: centimetres, a space between thousands, no "-0.00".
#[test]
fn coordinates_read_in_groups_of_three() {
    use super::status::grouped;
    assert_eq!(grouped(205_431.2, 2), "205 431.20");
    assert_eq!(grouped(451_822.746, 2), "451 822.75");
    assert_eq!(grouped(999.999, 2), "1 000.00");
    assert_eq!(grouped(12.0, 1), "12.0");
    assert_eq!(grouped(1_234.0, 0), "1 234");
    assert_eq!(grouped(-1_234.5, 1), "-1 234.5");
    assert_eq!(grouped(-0.001, 2), "0.00");
    assert_eq!(grouped(f64::NAN, 2), "–");
    assert_eq!(grouped(f64::INFINITY, 2), "–");
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
    // The scan finds the window's icons (37 before Find Number and Posthole Depth left, design 3.0
    // D1; 32 once the ribbon's 23 tiles became 9, D2).
    assert!(names.len() >= 30, "{names:?}");
    let missing: Vec<_> = names.iter().filter(|n| !crate::icons::exists(n)).collect();
    assert!(missing.is_empty(), "icons not bundled: {missing:?}");
}

// ---------- J005: the legend's rows ----------

/// A rectangle drawn on the current layer with the pen's look; its id.
fn draw_rect(h: &mut Harness, x: f64) -> NodeId {
    NodeId(h.app.run("shape.rectangle", json!({"x": x, "y": 10, "width": 20, "height": 20})).unwrap()["id"].as_u64().unwrap())
}

/// A new top layer `name` (the current one) made a legend row with `legend`, `legend.set`'s params.
fn legend_row(h: &mut Harness, name: &str, legend: serde_json::Value) -> NodeId {
    let id = NodeId(h.app.run("layer.new", json!({ "name": name })).unwrap()["id"].as_u64().unwrap());
    let mut p = legend;
    p["id"] = json!(id.0);
    h.app.run("legend.set", p).unwrap();
    id
}

fn weight_of(h: &Harness, id: NodeId) -> f64 {
    h.app.session.active().unwrap().doc.node(id).unwrap().appearance.stroke().unwrap().width
}

/// Whether the part `key` of the pen's row of choices `kind` ("dash", "hatch") shows on: oat
/// painted in it.
fn seg_on(h: &Harness, kind: &str, key: &str) -> bool {
    let t = h.tokens();
    let part = h.rect(wid(kind, key)).unwrap_or_else(|| panic!("no {kind} {key}"));
    h.rects_in(part).iter().any(|(_, fill, _)| *fill == t.row_selected)
}

/// The texts inside `area` on the line of the one reading `text`, left to right.
fn line_of(h: &Harness, text: &str, area: Rect) -> Vec<String> {
    let inside = |p: &&Painted| area.expand(0.5).contains_rect(p.rect);
    let at = h.texts.iter().filter(inside).find(|p| p.text == text).unwrap_or_else(|| panic!("no {text:?} in {}", h.line_in(area))).rect;
    let mut v: Vec<&Painted> = h.texts.iter().filter(inside).filter(|p| same_line(p.rect, at)).collect();
    v.sort_by(|a, b| a.rect.left().total_cmp(&b.rect.left()));
    v.into_iter().map(|p| p.text.clone()).collect()
}

/// The text `s` painted in the right column: where it is.
fn in_right(h: &Harness, s: &str) -> Rect {
    let right = h.panel(RIGHT);
    h.texts.iter().find(|p| p.text == s && right.contains_rect(p.rect)).map(|p| p.rect).unwrap_or_else(|| panic!("no {s:?} in the right column"))
}

/// The pen's part of the right column: from its line to the properties.
fn pen_area(h: &Harness) -> Rect {
    let right = h.panel(RIGHT);
    let top = h.rect(wid("palette-more", ())).unwrap().top() - 7.0;
    let bottom = h.rect(wid("props-fold", ())).unwrap().top() - 5.0;
    Rect::from_min_max(pos2(right.left(), top), pos2(right.right(), bottom))
}

/// The rows of `ids` laid out in the legend, top to bottom.
fn row_order(h: &Harness, ids: &[NodeId]) -> Vec<NodeId> {
    let mut v: Vec<(f32, NodeId)> = ids.iter().filter_map(|id| h.rect(wid("legend-row", id.0)).map(|r| (r.top(), *id))).collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v.into_iter().map(|(_, id)| id).collect()
}

/// A legend row (design 3.0 §7 「범례 줄 하나의 모양」): eye, padlock, its sample 34 × 10 drawn at
/// its weight, its name (「★ name」 in clay for a favourite), 「Apart N」 in white on clay when N of
/// its lines are set apart, its weight ("-" without one) and its number of objects; a mark row
/// shows its mark, and a layer without a row its colour.
#[test]
fn legend_rows_show_the_look_star_and_overrides() {
    let mut h = Harness::new();
    let t = h.tokens();
    let first = h.layer_ids()[0];
    let pits = legend_row(&mut h, "Pit outline", json!({"weight": 0.8, "sample": "area", "favourite": true}));
    // Its lines: two in its look, one set apart at 0.5.
    h.app.run("stroke.set", json!({"weight": 0.8})).unwrap();
    for x in [10.0, 40.0] {
        draw_rect(&mut h, x);
    }
    let apart = draw_rect(&mut h, 70.0);
    h.app.run("select.set", json!({ "ids": [apart.0] })).unwrap();
    h.app.run("stroke.set", json!({"weight": 0.5})).unwrap();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    let section = legend_row(&mut h, "Section line", json!({"weight": 0.2, "line": "dashdot"}));
    let datum = legend_row(&mut h, "Datum", json!({"weight": null, "sample": {"mark": "△"}}));
    h.settle();
    let row = |h: &Harness, id: NodeId| h.rect(wid("legend-row", id.0)).unwrap();
    let slot = |r: Rect| Rect::from_min_size(pos2(r.left() + 56.0, r.center().y - 5.0), vec2(34.0, 10.0));
    let pr = row(&h, pits);
    assert_eq!(h.texts_in(pr), ["★ Pit outline", "Apart 1", "0.8", "3"]);
    assert_eq!(h.color_of("★ Pit outline", pr), Some(t.clay_text), "a favourite's name is clay");
    assert_eq!(h.font_of("★ Pit outline", pr), Some(egui::FontId::proportional(12.5)));
    assert_eq!(h.color_of("Apart 1", pr), Some(t.on_action));
    assert_eq!(h.color_of("0.8", pr), Some(t.text_dim));
    assert_eq!(h.color_of("3", pr), Some(t.text_disabled));
    let badge = h.texts.iter().find(|p| p.text == "Apart 1").unwrap().rect;
    let rects = h.rects_in(pr);
    assert!(rects.iter().any(|(r, fill, _)| *fill == t.action && r.contains_rect(badge)), "the badge on clay: {rects:?}");
    // Its sample, an area row's too: a line 30 long across the slot at 0.8 pt's 2.8 points.
    let sample: Vec<egui::epaint::PathShape> = h
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Path(p) if !p.closed && p.points.iter().all(|q| slot(pr).expand(0.5).contains(*q)) => Some(p.clone()),
            _ => None,
        })
        .collect();
    let [line] = sample.as_slice() else { panic!("one line: {sample:?}") };
    let ends = (line.points.first().unwrap(), line.points.last().unwrap());
    assert!((ends.1.x - ends.0.x - 30.0).abs() < 0.5 && (ends.0.y - ends.1.y).abs() < 1e-3, "{ends:?}");
    assert_eq!((line.stroke.width, line.stroke.color.clone()), (2.8, egui::epaint::ColorMode::Solid(t.text)));
    // A dash-dot line at 0.2 pt: thin dashes, 0.9 points wide.
    let sr = row(&h, section);
    assert_eq!(h.texts_in(sr), ["Section line", "0.2", "0"]);
    let dashes: Vec<_> = h.segments_in(slot(sr)).into_iter().filter(|(_, s)| (s.width - 0.9).abs() < 1e-3 && s.color == t.text).collect();
    assert!(dashes.len() >= 3, "dashes: {:?}", h.segments_in(slot(sr)));
    // A mark row: its mark (△ drawn as a shape: [`mark_samples_are_as_large_as_the_mockup`]) and
    // no weight. It is the current row: on oat, its name as the others.
    let dr = row(&h, datum);
    assert_eq!(h.texts_in(dr), ["Datum", "-", "0"]);
    assert_eq!(h.color_of("Datum", dr), Some(t.text));
    assert_eq!(h.fill_at(dr), Some(t.row_selected));
    // A layer without a row: its colour, name and count.
    let lr = row(&h, first);
    assert_eq!(h.texts_in(lr), ["Layer 1", "-", "0"]);
    assert!(h.rects_in(lr).iter().any(|(_, _, edge)| (edge.width - 1.5).abs() < 1e-3), "its colour patch");
    // ★ and △ are in the fonts the app bundles: no missing-glyph boxes on any system.
    let ctx = egui::Context::default();
    crate::theme::install_fonts(&ctx);
    ctx.run_ui(egui::RawInput::default(), |_| {}).textures_delta.clear();
    for c in ['★', '△', '→', '×', '✓'] {
        assert!(ctx.fonts_mut(|f| f.has_glyph(&egui::FontId::proportional(11.0), c)), "{c}");
    }
}

/// In the legend the favourites come first under ★ Used Often, then the other rows in layer order,
/// top first.
#[test]
fn favourites_come_first_in_each_section() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let a = legend_row(&mut h, "A", json!({"weight": 0.5}));
    let b = legend_row(&mut h, "B", json!({"weight": 0.5, "favourite": true}));
    let c = legend_row(&mut h, "C", json!({"weight": 0.2}));
    h.settle();
    assert_eq!(row_order(&h, &[first, a, b, c]), [b, c, a, first]);
    let captions = |h: &Harness| ["★ Used Often", "Other rows"].map(|s| h.texts.iter().any(|p| p.text == s));
    assert_eq!(captions(&h), [true, true]);
    // No longer a favourite, B goes back to its place, and ★ Used Often with nothing in it goes.
    h.app.run("legend.set", json!({"id": b.0, "favourite": false})).unwrap();
    h.settle();
    assert_eq!(row_order(&h, &[first, a, b, c]), [c, b, a, first]);
    assert_eq!(captions(&h), [false, true]);
}

/// The legend of design 3.0 (§5 ⑦, §7): its head 36 tall with 「Legend」, the group shown first
/// (「Plan ▾」), ＋ and ⋯; rows 27 tall in sections, each under its caption in 11-point grey: ★ Used
/// Often, the legend groups with the one most rows come from first, Underlays (template layers)
/// and the other rows. A pick in the head's list puts that group first.
#[test]
fn the_legend_follows_the_mockup() {
    let mut h = Harness::new();
    let t = h.tokens();
    let first = h.layer_ids()[0];
    let pit = legend_row(&mut h, "Pit", json!({"weight": 0.8, "group": "plan"}));
    let cut = legend_row(&mut h, "Cut", json!({"weight": 0.8, "group": "section"}));
    let wall = legend_row(&mut h, "Wall", json!({"weight": 0.5, "group": "section", "favourite": true}));
    let sherd = legend_row(&mut h, "Sherd", json!({"weight": 0.2, "group": "finds"}));
    let scan = NodeId(h.app.run("layer.new", json!({"name": "Scan", "template": true})).unwrap()["id"].as_u64().unwrap());
    // Drawing on Layer 1: a locked current row would show its note in clay.
    h.app.run("layer.setCurrent", json!({ "id": first.0 })).unwrap();
    h.settle();
    let right = h.panel(RIGHT);
    // The head: Legend, then the group shown first: Section, which two rows come from.
    let head = Rect::from_min_size(right.min, vec2(right.width(), 36.0));
    assert_eq!(h.texts_in(head), ["Legend", "Section"]);
    assert_eq!(h.font_of("Legend", head).map(|f| f.size), Some(13.5));
    for (id, slot) in [("legend-more", 0.0), ("legend-new", 1.0)] {
        let b = h.rect(wid(id, ())).unwrap();
        // Where the mockup's ＋ and ⋯ glyphs are: the last 16 in from the right, 24 apart.
        assert!((b.center().y - head.center().y).abs() < 0.5 && (b.center().x - (right.right() - 16.0 - 24.0 * slot)).abs() < 0.5, "{id}: {b:?}");
    }
    let order = |h: &Harness, captions: [&str; 6]| -> Vec<f32> {
        captions
            .iter()
            .map(|c| {
                let p = h
                    .texts
                    .iter()
                    .find(|p| p.text == *c && right.contains_rect(p.rect) && p.rect.top() > head.bottom())
                    .unwrap_or_else(|| panic!("no caption {c}"));
                assert_eq!((p.font.size, p.color), (11.0, t.text_dim), "{c}");
                p.rect.top()
            })
            .collect()
    };
    let captions = ["★ Used Often", "Section", "Plan", "Finds", "Underlays", "Other rows"];
    let tops = order(&h, captions);
    // Each row under its caption, before the next.
    let rows = [wall, cut, pit, sherd, scan, first];
    for (i, id) in rows.iter().enumerate() {
        let r = h.rect(wid("legend-row", id.0)).unwrap();
        assert_eq!(r.height(), 27.0);
        assert!(tops[i] < r.top() && tops.get(i + 1).is_none_or(|next| r.bottom() <= *next + 0.5), "{} under {}", i, captions[i]);
    }
    // The underlay: its name in ink (as the mockup's 「실측 사진」; only a hidden row's is faint), a
    // pale box for its sample.
    let sr = h.rect(wid("legend-row", scan.0)).unwrap();
    assert_eq!(h.color_of("Scan", sr), Some(t.text));
    assert!(h.rects_in(sr).iter().any(|(_, fill, _)| *fill == t.border), "{:?}", h.rects_in(sr));
    // Finds first.
    h.click(wid("legend-group", ()));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 3, "{items:?}");
    h.click_at(items[2].0.center());
    assert_eq!(h.texts_in(head), ["Legend", "Finds"]);
    // The chosen group first, the others in their own order.
    let tops = order(&h, ["★ Used Often", "Finds", "Plan", "Section", "Underlays", "Other rows"]);
    assert!(tops.windows(2).all(|w| w[0] < w[1]), "{tops:?}");
}

/// A click on a legend row makes it the layer drawn on and gives the pen its look; a layer without a
/// row leaves the pen as it is (criterion 13).
#[test]
fn clicking_a_row_picks_it_for_the_pen() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let section = legend_row(&mut h, "Section line", json!({"weight": 0.2, "line": "dashdot"}));
    h.settle();
    h.click(wid("legend-row", first.0));
    assert_eq!(h.current_layer(), Some(first));
    assert_eq!(h.app.session.new_art_stroke().width, 1.0, "no look: the pen as it was");
    h.click(wid("legend-row", section.0));
    assert_eq!(h.current_layer(), Some(section));
    let pen = h.app.session.new_art_stroke();
    assert_eq!(pen.width, 0.2);
    assert_eq!(pen.dash.map(|d| d.pattern), vectorcraft_doc::LineType::DashDot.dash_pt());
}

/// With a selection tool in hand, a click on a row moves the selected objects into it and gives
/// their lines its look, one undo step (criterion 13).
#[test]
fn clicking_a_row_with_the_selection_tool_moves_the_selection() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let line = draw_rect(&mut h, 10.0);
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.app.select_tool("selection");
    h.settle();
    h.click(wid("legend-row", pits.0));
    let layer_of = |h: &Harness| h.app.session.active().unwrap().doc.layer_containing(line);
    assert_eq!(layer_of(&h), Some(pits));
    assert_eq!(weight_of(&h, line), 0.8);
    assert_eq!(h.current_layer(), Some(pits));
    assert!(h.status_line().contains("Moved 1 to Pits"), "{}", h.status_line());
    h.app.run("edit.undo", json!({})).unwrap();
    assert_eq!(layer_of(&h), Some(first));
    assert_eq!(weight_of(&h, line), 1.0);
}

/// With a drawing tool in hand the line just drawn stays selected; a click on another row to draw
/// the next line with leaves it where it is (criterion 13, decision 1).
#[test]
fn clicking_a_row_while_drawing_does_not_move_the_new_line() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("layer.setCurrent", json!({ "id": first.0 })).unwrap();
    h.app.select_tool("pen");
    let line = draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.settle();
    h.click(wid("legend-row", pits.0));
    assert_eq!(h.current_layer(), Some(pits));
    assert_eq!(h.app.session.active().unwrap().doc.layer_containing(line), Some(first));
    assert_eq!(weight_of(&h, line), 1.0);
}

/// A right click on a row lists Edit Row…, Select All on This Row, ★ Used Often, Move Up and Move
/// Down within its section, and Delete Row; the count at its right selects its objects too
/// (design 3.0 §7).
#[test]
fn legend_rows_menu_and_count_work() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let a = legend_row(&mut h, "A", json!({"weight": 0.5}));
    let b = legend_row(&mut h, "B", json!({"weight": 0.5}));
    let lines = [draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0)];
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    let menu = |h: &mut Harness, id: NodeId| {
        let at = h.rect(wid("legend-row", id.0)).unwrap().center();
        h.press(at, PointerButton::Secondary);
        h.popup_buttons()
    };
    let enabled = |items: &[(Rect, bool)]| items.iter().map(|(_, e)| *e).collect::<Vec<_>>();
    let selected = |h: &Harness| {
        let mut v = h.app.session.active().unwrap().selection.objects.clone();
        v.sort();
        v
    };
    // B tops its section: Move Up waits.
    let items = menu(&mut h, b);
    assert_eq!(enabled(&items), [true, true, true, false, true, true]);
    h.click_at(items[1].0.center());
    assert_eq!(selected(&h), lines);
    assert!(h.status_line().contains("Selected: 2"), "{}", h.status_line());
    // ★ takes B up to Used Often; A tops the other rows then.
    let items = menu(&mut h, b);
    h.click_at(items[2].0.center());
    assert!(h.app.session.active().unwrap().doc.node(b).unwrap().legend().unwrap().favourite);
    let items = menu(&mut h, a);
    assert_eq!(enabled(&items), [true, true, true, false, true, true]);
    h.click_at(items[4].0.center());
    assert_eq!(row_order(&h, &[a, first]), [first, a], "A under Layer 1");
    let items = menu(&mut h, a);
    assert_eq!(enabled(&items), [true, true, true, true, false, true]);
    h.click_at(items[3].0.center());
    assert_eq!(row_order(&h, &[a, first]), [a, first], "and over it again");
    // The count.
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.click(wid("legend-count", b.0));
    assert_eq!(selected(&h), lines);
    // Edit Row… opens its editor; Delete Row takes it away.
    let items = menu(&mut h, a);
    h.click_at(items[0].0.center());
    let card = h.rect(wid("row-card", ())).expect("A's editor");
    assert!(h.texts_in(card).contains(&"A".to_string()), "{:?}", h.texts_in(card));
    h.key(egui::Key::Escape);
    let items = menu(&mut h, a);
    h.click_at(items[5].0.center());
    assert!(!h.layer_ids().contains(&a));
}

/// The legend's ⋯ (design 3.0 §5 ⑦): Edit Row… for the current row, the 12 field standard rows
/// put back, unused rows removed, Weights at Once, the Layers panel and style templates.
#[test]
fn the_legend_more_menu_runs_its_items() {
    let mut h = Harness::new();
    h.click(wid("legend-more", ()));
    let items = h.popup_buttons();
    assert_eq!(items.iter().map(|(_, e)| *e).collect::<Vec<_>>(), [true; 6]);
    h.key(egui::Key::Escape);
    let n = h.layer_ids().len();
    h.legend_more("Put the 12 Field Standard Rows Back");
    assert_eq!(h.layer_ids().len(), n + 12);
    assert!(h.status_line().contains("Legend rows added: 12"), "{}", h.status_line());
    h.legend_more("Put the 12 Field Standard Rows Back");
    assert_eq!(h.layer_ids().len(), n + 12);
    assert!(h.status_line().contains("The 12 field standard rows are all there"), "{}", h.status_line());
    // Unused: every empty row but the current one.
    h.legend_more("Remove Unused Rows");
    assert_eq!(h.layer_ids().len(), 1);
    assert!(h.status_line().contains("Unused rows removed: 12"), "{}", h.status_line());
    h.legend_more("Edit Row…");
    let card = h.rect(wid("row-card", ())).expect("the current row's editor");
    let name = h.app.session.active().unwrap().doc.layers[0].display_name();
    assert!(h.texts_in(card).contains(&name), "{name} in {:?}", h.texts_in(card));
    h.key(egui::Key::Escape);
    h.open_weights();
    assert!(h.rect(wid("weights-card", ())).is_some(), "Weights at Once");
    h.key(egui::Key::Escape);
    h.legend_more("Layers Panel");
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"));
    h.legend_more("Save or Load a Style Template…");
    assert_eq!(h.ctx.data(|d| d.get_temp::<bool>(wid("style-dialog-open", ()))), Some(true));
}

/// When the legend's rows overflow its list, the rows at its edges show as plainly as the others:
/// no fade (egui fades a scroll area's edges) that would read as a hidden layer (criterion 21, F2).
#[test]
fn the_legend_list_does_not_fade_its_last_row() {
    let mut h = Harness::new();
    for i in 0..30 {
        h.app.run("layer.new", json!({ "name": format!("Pit {i}") })).unwrap();
    }
    h.settle();
    let top_row = h.rect(wid("legend-row", h.layer_ids().last().unwrap().0)).unwrap();
    let line = h.rect(wid("palette-more", ())).unwrap().top() - 7.0;
    let list = Rect::from_min_max(pos2(top_row.left(), top_row.top() - 40.0), pos2(top_row.right(), line));
    assert!(h.gradients_over(list).is_empty(), "at the top: {:?}", h.gradients_over(list));
    h.wheel(top_row.center(), vec2(0.0, -200.0));
    assert!(h.gradients_over(list).is_empty(), "scrolled: {:?}", h.gradients_over(list));
}

// ---------- J005: the row editor and Weights at Once ----------

/// Double-clicking a legend row opens its editor, a card beside the right column level with the
/// row; ✕, Esc and a press outside it close it (criterion 17).
#[test]
fn double_clicking_a_row_opens_its_editor() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pit outline", json!({"weight": 0.8, "strata": "feature_poly"}));
    h.settle();
    let (card, row) = (wid("row-card", ()), wid("legend-row", pits.0));
    assert!(h.rect(card).is_none());
    h.double_click(row);
    let c = h.rect(card).expect("the editor");
    let (right, r, status) = (h.panel(RIGHT), h.rect(row).unwrap(), h.panel(STATUS));
    assert!((c.width() - 304.0).abs() < 1.0 && c.right() <= right.left() && right.left() - c.right() <= 12.0, "{c:?} beside {right:?}");
    // Level with the row, or pushed up to stay above the status bar.
    assert!((c.top() - r.top()).abs() < 1.0 || (c.bottom() - status.top()).abs() < 1.0, "level with the row: {c:?}, {r:?}");
    assert!(c.bottom() <= status.top() + 0.5, "{c:?} over the status bar {status:?}");
    let line = h.line_in(c);
    for want in [
        "Legend Row",
        "Pit outline",
        "Name",
        "유구_면 · feature_poly",
        "This row is the layer",
        "Weight",
        "= 0.28 mm",
        "Solid",
        "Dash-dot",
        "Often Used",
        "This row's lines",
        "Select All",
        "Back to the Legend",
        "Saved in this drawing file for now",
        "Layer Options…",
    ] {
        assert!(line.contains(want), "{want} in {line}");
    }
    h.click(wid("card-close", ()));
    assert!(h.rect(card).is_none(), "✕ closes it");
    // Right away: egui alone would take the row's second click for a third one.
    h.double_click(row);
    assert!(h.rect(card).is_some(), "it opens again right after ✕");
    h.key(egui::Key::Escape);
    assert!(h.rect(card).is_none(), "Esc closes it");
    h.double_click(row);
    assert!(h.rect(card).is_some());
    h.click_at(h.app.canvas_rect.unwrap().center());
    assert!(h.rect(card).is_none(), "a press outside closes it");
}

/// The editor changes its row: a weight chip restyles the row's lines that followed it (one set
/// apart keeps its own), and so do the number field and the line types; ★, Select All, Back to the
/// Legend, the name and No Weight work on the row (criterion 17).
#[test]
fn the_row_editor_changes_the_row_and_its_lines() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.8})).unwrap();
    let (a, b, c) = (draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0), draw_rect(&mut h, 70.0));
    h.app.run("select.set", json!({ "ids": [c.0] })).unwrap();
    h.app.run("stroke.set", json!({"weight": 0.3})).unwrap();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    let legend = |h: &Harness| h.app.session.active().unwrap().doc.node(pits).unwrap().legend().cloned().unwrap();
    h.click(wid("card-weight", "0.5"));
    assert_eq!(legend(&h).weight, Some(0.5));
    assert_eq!((weight_of(&h, a), weight_of(&h, b), weight_of(&h, c)), (0.5, 0.5, 0.3));
    assert!(h.status_line().contains("Lines changed: 2"), "{}", h.status_line());
    h.type_into(wid("card-pt", ()), "0.6");
    assert_eq!(legend(&h).weight, Some(0.6));
    assert_eq!((weight_of(&h, a), weight_of(&h, c)), (0.6, 0.3));
    assert!(h.line_in(h.rect(wid("row-card", ())).unwrap()).contains("= 0.21 mm"));
    h.click(wid("card-line", "dashed"));
    assert_eq!(legend(&h).line, vectorcraft_doc::LineType::Dashed);
    let dash =
        |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().appearance.stroke().unwrap().dash.clone().map(|d| d.pattern);
    assert_eq!(dash(&h, a), vectorcraft_doc::LineType::Dashed.dash_pt());
    h.click(wid("card-favourite", ()));
    assert!(legend(&h).favourite);
    h.click(wid("card-select", ()));
    let mut selected = h.app.session.active().unwrap().selection.objects.clone();
    selected.sort();
    assert_eq!(selected, [a, b, c]);
    h.click(wid("card-revert", ()));
    assert_eq!((weight_of(&h, c), dash(&h, c)), (0.6, vectorcraft_doc::LineType::Dashed.dash_pt()));
    h.type_into(wid("card-name", ()), "Pit outlines");
    assert_eq!(h.app.session.active().unwrap().doc.node(pits).unwrap().display_name(), "Pit outlines");
    h.click(wid("card-weight", "none"));
    assert_eq!(legend(&h).weight, None);
    assert_eq!(weight_of(&h, a), 0.6, "no look: the lines stay");
}

/// The rows Weights at Once's card shows: those of `plan`, the ones changing alike together.
fn plan_table(h: &Harness, params: serde_json::Value) -> Vec<(String, String, String)> {
    let plan = vectorcraft_engine::cmd::legend::plan_weights(&h.app.session.active().unwrap().doc, &params).unwrap();
    let mut rows: Vec<(String, String, String)> = vec![];
    for row in &plan.rows {
        let (from, to) = (super::legend::pt(row.from), super::legend::pt(row.to));
        match rows.iter_mut().find(|r| r.1 == from && r.2 == to) {
            Some(r) => r.0 = format!("{} · {}", r.0, row.name),
            None => rows.push((row.name.clone(), from, to)),
        }
    }
    rows
}

/// Weights at Once's card previews the change as the engine plans it (the rows that change alike
/// together) and Change applies it, one undo step (criterion 18).
#[test]
fn weights_card_previews_and_applies_a_ratio() {
    let mut h = Harness::sized(1600.0, 1400.0);
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    let doc = h.app.session.active().unwrap().doc.clone();
    let named = |name: &str| doc.layers.iter().find(|l| l.display_name() == name).unwrap().id;
    let pit = named("굴광선 (유구 면)");
    h.app.run("legend.pick", json!({ "id": pit.0 })).unwrap();
    let line = draw_rect(&mut h, 10.0);
    assert_eq!(weight_of(&h, line), 0.8);
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.open_weights();
    let card = h.rect(wid("weights-card", ())).expect("the card");
    let line_in = h.line_in(card);
    for want in ["Weights at Once", "This drawing", "Ratio", "Change Values", "One Step", "Lines set apart too", "Change", "Close"] {
        assert!(line_in.contains(want), "{want} in {line_in}");
    }
    assert_eq!(h.texts_in(h.rect(wid("weights-ratio", ())).unwrap()), ["×", "0.8"]);
    // The table, as the engine plans it.
    let want = plan_table(&h, json!({"how": "ratio", "ratio": 0.8}));
    assert!(want.len() >= 4, "{want:?}");
    for (names, from, to) in &want {
        let cell = h.texts.iter().find(|p| p.text == *names && card.contains_rect(p.rect)).unwrap_or_else(|| panic!("{names} in {line_in}")).rect;
        let row: Vec<String> = h.texts.iter().filter(|p| card.contains_rect(p.rect) && same_line(p.rect, cell)).map(|p| p.text.clone()).collect();
        assert!(row.contains(from) && row.contains(to), "{names}: {from} → {to} in {row:?}");
    }
    assert!(line_in.contains("Lines that change: 1. One Ctrl+Z puts them all back."), "{line_in}");
    h.click(wid("weights-apply", ()));
    let legend = |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().legend().cloned().unwrap();
    assert_eq!(legend(&h, pit).weight, Some(0.64));
    assert_eq!(weight_of(&h, line), 0.64);
    assert_eq!(legend(&h, named("조사구역 경계")).weight, Some(0.8));
    h.app.run("edit.undo", json!({})).unwrap();
    assert_eq!((legend(&h, pit).weight, weight_of(&h, line)), (Some(0.8), 0.8), "one undo step");
}

/// The card's other ways: Change Values (only rows at the first weight take the second) and One
/// Step (thinner by default, or thicker), with lines set apart left unless asked (criterion 18).
#[test]
fn weights_card_value_and_step_modes() {
    let mut h = Harness::sized(1600.0, 1400.0);
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    let doc = h.app.session.active().unwrap().doc.clone();
    let named = |name: &str| doc.layers.iter().find(|l| l.display_name() == name).unwrap().id;
    let (floor, pit, boundary) = (named("바닥선"), named("굴광선 (유구 면)"), named("조사구역 경계"));
    // A line on the floor row set apart at 0.2.
    h.app.run("legend.pick", json!({ "id": floor.0 })).unwrap();
    let apart = draw_rect(&mut h, 10.0);
    h.app.run("stroke.set", json!({"weight": 0.2})).unwrap();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    let legend = |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().legend().cloned().unwrap().weight;
    h.open_weights();
    h.click(wid("weights-how", "value"));
    h.type_into(wid("weights-from", ()), "0.5");
    h.type_into(wid("weights-to", ()), "0.4");
    let card = h.rect(wid("weights-card", ())).unwrap();
    let want = plan_table(&h, json!({"how": "value", "from": 0.5, "to": 0.4}));
    assert_eq!(want.len(), 1, "{want:?}");
    assert!(h.texts.iter().any(|p| p.text == want[0].0 && card.contains_rect(p.rect)), "{want:?} in {}", h.line_in(card));
    h.click(wid("weights-apply", ()));
    assert_eq!((legend(&h, floor), legend(&h, pit)), (Some(0.4), Some(0.8)));
    assert_eq!(weight_of(&h, apart), 0.2, "set apart: left");
    // One step: thinner by default; thicker, with the lines set apart too. Change puts the card
    // away: it opens again for each.
    h.open_weights();
    h.click(wid("weights-how", "step"));
    h.click(wid("weights-apply", ()));
    assert_eq!((legend(&h, floor), legend(&h, pit), legend(&h, boundary)), (Some(0.3), Some(0.6), Some(0.8)));
    assert_eq!(weight_of(&h, apart), 0.2, "set apart: left");
    h.open_weights();
    h.click(wid("weights-how", "step"));
    h.click(wid("weights-step", "up"));
    h.click(wid("weights-overrides", ()));
    h.click(wid("weights-apply", ()));
    assert_eq!((legend(&h, floor), legend(&h, pit), legend(&h, boundary)), (Some(0.4), Some(0.8), Some(1.0)));
    assert_eq!(weight_of(&h, apart), 0.3, "set apart, with Lines set apart too");
}

// ---------- J005: the drawing palette, the Drawing menu and the Control bar ----------

/// The pen's line types run: with nothing selected they set the next line's dashes, with lines
/// selected theirs (Solid takes them off) and the pen stays as it was; the first selected line's
/// type shows on, none when the selected lines' dashes differ.
#[test]
fn palette_line_types_set_the_dash() {
    use vectorcraft_doc::LineType;
    let mut h = Harness::new();
    let on = |h: &Harness, key: &str| seg_on(h, "dash", key);
    assert!(on(&h, "solid") && !on(&h, "dashed"));
    for (key, want) in [("dashed", LineType::Dashed), ("dotted", LineType::Dotted), ("dashdot", LineType::DashDot), ("solid", LineType::Solid)] {
        h.click(wid("dash", key));
        assert_eq!(LineType::of(&h.app.session.new_art_stroke()), want, "the next line, {key}");
        assert!(on(&h, key), "{key} shows on");
    }
    let (a, b) = (draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0));
    h.app.run("select.set", json!({ "ids": [a.0, b.0] })).unwrap();
    h.settle();
    h.click(wid("dash", "dotted"));
    let line_of = |h: &Harness, id: NodeId| LineType::of(h.app.session.active().unwrap().doc.node(id).unwrap().appearance.stroke().unwrap());
    assert_eq!((line_of(&h, a), line_of(&h, b)), (LineType::Dotted, LineType::Dotted));
    assert_eq!(LineType::of(&h.app.session.new_art_stroke()), LineType::Solid, "the pen as it was");
    assert!(on(&h, "dotted") && !on(&h, "solid"));
    // Dashes that differ: none on.
    h.app.run("select.set", json!({ "ids": [b.0] })).unwrap();
    h.app.run("stroke.set", json!({ "dash": null })).unwrap();
    h.app.run("select.set", json!({ "ids": [a.0, b.0] })).unwrap();
    h.settle();
    assert!(["solid", "dashed", "dotted", "dashdot"].iter().all(|k| !on(&h, k)), "mixed: none on");
}

/// The legend's ＋ makes a row of the pen's look (the first selected line's, when lines are
/// selected), named with the next free number, moves the selected lines into it and opens its
/// editor (design 3.0 §5 ⑦).
#[test]
fn the_legend_plus_makes_a_row_of_the_pens_look() {
    use vectorcraft_doc::LineType;
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let line = draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.app.run("stroke.set", json!({ "weight": 0.3, "dash": LineType::Dashed.dash_pt() })).unwrap();
    h.settle();
    h.click(wid("legend-new", ()));
    let row = h.current_layer().unwrap();
    assert_ne!(row, first);
    let doc = h.app.session.active().unwrap().doc.clone();
    let node = doc.node(row).unwrap();
    assert_eq!(node.display_name(), "Legend Row 1");
    let legend = node.legend().cloned().unwrap();
    assert_eq!((legend.weight, legend.line), (Some(0.3), LineType::Dashed));
    assert_eq!(doc.layer_containing(line), Some(row), "the line moves into its row");
    assert!(h.status_line().contains("Moved 1 to Legend Row 1"), "{}", h.status_line());
    let card = h.rect(wid("row-card", ())).expect("the row's editor opens");
    assert!(h.line_in(card).contains("Legend Row 1"), "{}", h.line_in(card));
    // Nothing selected: the pen's look, the next number.
    h.key(egui::Key::Escape);
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.app.run("stroke.set", json!({ "weight": 0.15, "dash": null })).unwrap();
    h.settle();
    h.click(wid("legend-new", ()));
    let node = h.app.session.active().unwrap().doc.node(h.current_layer().unwrap()).unwrap().clone();
    assert_eq!(node.display_name(), "Legend Row 2");
    assert_eq!(node.legend().map(|l| (l.weight, l.line.clone())), Some((Some(0.15), LineType::Solid)));
}

/// When the pen differs from its row, its three buttons (design 3.0 §7 「펜이 범례와 다를 때」): Just
/// This Once folds them and draws on with the look, the lines set apart (「Apart 1」 on the row);
/// they come back when the look changes again. Back to the Legend gives the pen the row's look, or
/// the selected lines with a selection; Update the Legend gives the row the look shown, the lines
/// that followed it following.
#[test]
fn the_three_buttons_do_what_they_say() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("legend.pick", json!({ "id": pits.0 })).unwrap();
    h.app.select_tool("pen");
    h.settle();
    let button = |h: &Harness, key: &str| h.rect(wid("pen-state", key));
    assert!(button(&h, "once").is_none(), "the pen as its row: no buttons");
    h.click(wid("weight", "0.6"));
    assert_eq!(h.app.session.new_art_stroke().width, 0.6);
    for key in ["once", "update", "back"] {
        assert_eq!(button(&h, key).map(|b| b.height()), Some(24.0), "{key}");
    }
    let labels: Vec<String> = ["once", "update", "back"].iter().flat_map(|k| h.texts_in(button(&h, k).unwrap())).collect();
    assert_eq!(labels, ["Just This Once", "Update the Legend", "Back to the Legend"]);
    // Just This Once: they fold; the next line keeps the look, set apart.
    h.click(wid("pen-state", "once"));
    assert!(button(&h, "once").is_none());
    let apart = draw_rect(&mut h, 10.0);
    assert_eq!(weight_of(&h, apart), 0.6);
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    assert!(button(&h, "once").is_none(), "still folded for that look");
    assert!(h.texts_in(h.rect(wid("legend-row", pits.0)).unwrap()).contains(&"Apart 1".to_string()));
    // Another look: they come back.
    h.click(wid("weight", "0.5"));
    assert!(button(&h, "back").is_some());
    // Back to the Legend, nothing selected: the pen takes the row's look.
    h.click(wid("pen-state", "back"));
    assert_eq!(h.app.session.new_art_stroke().width, 0.8);
    assert!(h.status_line().contains("The next line takes Pits's look"), "{}", h.status_line());
    assert!(button(&h, "back").is_none());
    // The line set apart, selected: Update the Legend gives the row its look, and the line that
    // followed the row follows.
    let follows = draw_rect(&mut h, 40.0);
    h.app.run("select.set", json!({ "ids": [apart.0] })).unwrap();
    h.settle();
    h.click(wid("pen-state", "update"));
    assert_eq!(row_weight(&h, pits), Some(0.6));
    assert_eq!(weight_of(&h, follows), 0.6);
    assert!(h.status_line().contains("Pits takes the pen's look · lines: 1"), "{}", h.status_line());
    // Back to the Legend with a selection: the selected lines take the row's look.
    h.app.run("stroke.set", json!({ "weight": 0.2 })).unwrap();
    h.settle();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.click(wid("weight", "1.0"));
    h.app.run("select.set", json!({ "ids": [apart.0] })).unwrap();
    h.settle();
    h.click(wid("pen-state", "back"));
    assert_eq!(weight_of(&h, apart), 0.6);
    assert!(h.status_line().contains("Lines back to the legend: 1"), "{}", h.status_line());
    // ... and the pen the current row's look (design 3.0 §7: the pen and the selected lines).
    assert_eq!(h.app.session.new_art_stroke().width, 0.6, "the pen too");
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    assert!(button(&h, "once").is_none(), "nothing differs after");
}

/// Just This Once pressed for lines picked out with the Selection tool folds the buttons too (the
/// lines keep their look, set apart) while they stay selected; those lines or others in that look,
/// picked out later, show them again. A weight chip pressed for selected lines leaves the next
/// line's weight as it was.
#[test]
fn just_this_once_folds_for_selected_lines_too() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("legend.pick", json!({ "id": pits.0 })).unwrap();
    let (a, b) = (draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0));
    h.app.select_tool("selection");
    h.app.run("select.set", json!({ "ids": [a.0] })).unwrap();
    h.settle();
    h.click(wid("weight", "0.6"));
    assert_eq!(weight_of(&h, a), 0.6);
    assert_eq!(h.app.session.new_art_stroke().width, 0.8, "the next line keeps the row's weight");
    let buttons = |h: &Harness| h.rect(wid("pen-state", "once")).is_some();
    assert!(buttons(&h), "a line not as its row");
    h.click(wid("pen-state", "once"));
    assert!(!buttons(&h), "folded");
    h.frame(vec![Event::PointerMoved(h.app.canvas_rect.unwrap().center())]);
    h.settle();
    assert!(!buttons(&h), "and stay folded");
    assert_eq!(weight_of(&h, a), 0.6, "the line keeps its look");
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    assert!(!buttons(&h), "the pen as its row");
    h.app.run("select.set", json!({ "ids": [a.0] })).unwrap();
    h.settle();
    assert!(buttons(&h), "the same line picked out again: it may be for the legend now");
    // Another line in that look, picked out: they show again.
    h.app.run("select.set", json!({ "ids": [b.0] })).unwrap();
    h.app.run("stroke.set", json!({ "weight": 0.6 })).unwrap();
    h.settle();
    assert!(buttons(&h), "other lines may be for the legend");
}

/// Drawing on a sublayer, the pen is against the row the sublayer is in: its name in the head and
/// whether the pen is as its look.
#[test]
fn the_pen_on_a_sublayer_is_against_its_row() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.6}));
    h.app.run("legend.pick", json!({ "id": pits.0 })).unwrap();
    let sub = NodeId(h.app.run("layer.newSublayer", json!({"name": "Sub"})).unwrap()["id"].as_u64().unwrap());
    h.app.run("layer.setCurrent", json!({ "id": sub.0 })).unwrap();
    h.settle();
    assert_eq!(h.current_layer(), Some(sub));
    let has = |h: &Harness, s: &str| h.texts.iter().any(|p| p.text == s && pen_area(h).contains_rect(p.rect));
    assert!(has(&h, "Pits · Next line") && has(&h, "Same as the legend"), "{}", h.line_in(pen_area(&h)));
    h.click(wid("weight", "0.8"));
    assert!(has(&h, "Not as the legend · Weight 0.6 → 0.8"), "{}", h.line_in(pen_area(&h)));
}

/// The pen's colours (design 3.0 §5 ⑧): each swatch paints its own colour, the line's on a click
/// and the fill's on Alt and a click, on the selected lines or for the next ones; black is the
/// pen's own, so a new drawing's pen shows it ringed as the mockup does.
#[test]
fn the_swatches_paint_their_colours() {
    let mut h = Harness::new();
    let t = h.tokens();
    let ring = |h: &Harness, key: &str| {
        let sw = h.rect(wid("pal-swatch", key)).unwrap();
        h.rects_in(sw.expand(4.0)).iter().any(|(r, _, stroke)| stroke.color == t.text && r.width() >= sw.width() + 3.0)
    };
    assert!(ring(&h, "black") && !ring(&h, "red"), "the pen's black ringed");
    let colour = |paint: &vectorcraft_color::Paint| match paint {
        vectorcraft_color::Paint::Solid { color, .. } => crate::panels::c32(color),
        other => panic!("{other:?}"),
    };
    let hex = |s: &str| crate::panels::c32(&vectorcraft_color::Color::from_hex(s).unwrap());
    let line = draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.settle();
    let node = |h: &Harness| h.app.session.active().unwrap().doc.node(line).unwrap().clone();
    let stroke = |h: &Harness| colour(&vectorcraft_doc::legend::stroke_of(&node(h)).unwrap().paint);
    for (key, want) in [("red", "#E8334F"), ("blue", "#2B5FB8"), ("ochre", "#B08A52"), ("white", "#FFFFFF"), ("black", "#000000")] {
        h.click(wid("pal-swatch", key));
        assert_eq!(stroke(&h), hex(want), "{key}");
    }
    h.alt_click(wid("pal-swatch", "brown"));
    let fill = node(&h).appearance.fill_paint();
    assert_eq!(colour(&fill), hex("#7A5235"), "Alt: the fill");
    assert_eq!(stroke(&h), hex("#000000"), "the line's as it was");
    // Nothing selected: the next line's.
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.click(wid("pal-swatch", "blue"));
    assert_eq!(colour(&h.app.session.new_art_stroke().paint), hex("#2B5FB8"));
    assert!(ring(&h, "blue"));
}

/// The pen's top stays put while the pointer is on it as its three buttons go too (design 3.0
/// §5 ⑧): a press on Update the Legend leaves nothing else under the pointer to press next.
#[test]
fn the_pen_stays_put_as_its_buttons_go() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.6}));
    h.app.run("legend.pick", json!({ "id": pits.0 })).unwrap();
    h.settle();
    h.click(wid("weight", "0.8"));
    // The pointer away first: the pen laid out with its buttons, up over the legend.
    h.frame(vec![Event::PointerMoved(h.app.canvas_rect.unwrap().center())]);
    h.settle();
    let top = h.rect(wid("palette-more", ())).unwrap().top();
    let at = h.rect(wid("pen-state", "update")).unwrap().center();
    h.click_at(at);
    assert_eq!(row_weight(&h, pits), Some(0.8));
    assert!(h.rect(wid("pen-state", "update")).is_none(), "the buttons went");
    assert_eq!(h.rect(wid("palette-more", ())).unwrap().top(), top, "the pen's top stays");
    for key in ["none", "45", "135", "alt"] {
        assert!(!h.rect(wid("hatch", key)).unwrap().contains(at), "{key} not under the pointer");
    }
    // The pointer away: the pen down to the properties again.
    h.frame(vec![Event::PointerMoved(h.app.canvas_rect.unwrap().center())]);
    h.settle();
    assert!((h.rect(wid("palette-more", ())).unwrap().top() - top - 40.0).abs() < 0.5, "{top}");
}

/// The selected objects with their rows, in one walk: in the selection's order, a group's line in
/// the group's row, an id not in the drawing left out.
#[test]
fn selected_objects_are_found_with_their_rows() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let a = draw_rect(&mut h, 10.0);
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("legend.pick", json!({ "id": pits.0 })).unwrap();
    let (b, c) = (draw_rect(&mut h, 40.0), draw_rect(&mut h, 70.0));
    h.app.run("select.set", json!({ "ids": [b.0, c.0] })).unwrap();
    let group = NodeId(h.app.run("object.group", json!({})).unwrap()["id"].as_u64().unwrap());
    let doc = &h.app.session.active().unwrap().doc;
    let found: Vec<(NodeId, NodeId)> = super::selected_in_rows(doc, &[c, a, NodeId(9999), group]).iter().map(|(n, row)| (n.id, row.id)).collect();
    assert_eq!(found, [(c, pits), (a, first), (group, pits)]);
}

/// Drawing on without pressing anything counts as Just This Once (design 3.0 §7): the line drawn
/// keeps the pen's look, set apart, and the three buttons fold, for it and for the pen; another
/// look brings them back. The line picked out later with the Selection tool shows them, so its look
/// can go to the legend.
#[test]
fn drawing_on_counts_as_just_this_once() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("legend.pick", json!({ "id": pits.0 })).unwrap();
    h.app.select_tool("pen");
    h.settle();
    let buttons = |h: &Harness| h.rect(wid("pen-state", "once")).is_some();
    h.click(wid("weight", "0.6"));
    assert!(buttons(&h));
    let line = draw_rect(&mut h, 10.0);
    h.settle();
    assert_eq!(weight_of(&h, line), 0.6);
    assert!(!buttons(&h), "drawn: folded");
    assert!(h.texts_in(h.rect(wid("legend-row", pits.0)).unwrap()).contains(&"Apart 1".to_string()));
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    assert!(!buttons(&h), "and for the pen");
    h.app.select_tool("selection");
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.settle();
    assert!(buttons(&h), "picked out: they show");
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.click(wid("weight", "0.5"));
    assert!(buttons(&h), "another look");
}

/// A double click on a weight chip opens the field to type a weight in, even when its first click
/// makes the pen differ from its row and brings the three buttons: the chip stays under the
/// pointer.
#[test]
fn a_double_click_on_a_chip_types_a_weight_as_the_buttons_come() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.6}));
    h.app.run("legend.pick", json!({ "id": pits.0 })).unwrap();
    h.settle();
    h.double_click(wid("weight", "0.8"));
    assert!(h.rect(wid("pen-state", "once")).is_some(), "the buttons came");
    assert!(h.rect(wid("pen-weight-field", ())).is_some(), "the field to type in");
    // It has the keyboard: what it holds selected, typed over, then Enter.
    let all = egui::Modifiers::COMMAND;
    h.frame(vec![Event::Key { key: egui::Key::A, physical_key: None, pressed: true, repeat: false, modifiers: all }]);
    h.frame(vec![Event::Text("0.7".to_string())]);
    h.key(egui::Key::Enter);
    assert!((h.app.session.new_art_stroke().width - 0.7).abs() < 1e-9, "{}", h.app.session.new_art_stroke().width);
}

/// The pen's state line (design 3.0 §7): 「✓ Same as the legend」 with its green check while the
/// pen is as its row, 「● Not as the legend · Weight 0.6 → 0.8」 with a clay dot and what differs
/// when not, the pen growing by its three buttons; a layer without a look says so. The head names
/// the row and what the pen draws.
#[test]
fn the_pen_says_when_it_differs_from_the_legend() {
    let mut h = Harness::new();
    let t = h.tokens();
    let has = |h: &Harness, s: &str| h.texts.iter().any(|p| p.text == s && pen_area(h).contains_rect(p.rect));
    let circles = |h: &Harness, fill: egui::Color32| {
        let area = pen_area(h);
        h.shapes.iter().filter(|c| matches!(&c.shape, egui::Shape::Circle(s) if s.fill == fill && area.contains(s.center))).count()
    };
    assert!(has(&h, "The layer has no legend look"), "{}", h.line_in(pen_area(&h)));
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.6}));
    h.app.run("legend.pick", json!({ "id": pits.0 })).unwrap();
    h.settle();
    assert!(has(&h, "Pits · Next line"), "{}", h.line_in(pen_area(&h)));
    assert!(has(&h, "Same as the legend"), "{}", h.line_in(pen_area(&h)));
    assert_eq!(circles(&h, t.ok), 1, "the green check");
    let top = h.rect(wid("palette-more", ())).unwrap().top();
    let chip = h.rect(wid("weight", "0.8")).unwrap();
    h.click(wid("weight", "0.8"));
    assert!(has(&h, "Not as the legend · Weight 0.6 → 0.8"), "{}", h.line_in(pen_area(&h)));
    assert_eq!((circles(&h, t.ok), circles(&h, t.action)), (0, 1), "the clay dot");
    // The pen grows by its buttons: while the pointer is on it, downwards over the properties, so
    // the chip pressed stays under it; once it leaves, upwards over the legend.
    assert_eq!(h.rect(wid("weight", "0.8")), Some(chip), "the chip stays under the pointer");
    let right = h.panel(RIGHT);
    assert!(h.rect(wid("pen-state", "back")).is_some_and(|b| right.contains_rect(b)), "the buttons in view");
    h.frame(vec![Event::PointerMoved(h.app.canvas_rect.unwrap().center())]);
    h.settle();
    assert!((top - h.rect(wid("palette-more", ())).unwrap().top() - 40.0).abs() < 0.5, "the pen grows by its buttons");
    assert!((h.rect(wid("props-fold", ())).unwrap().center().y - (right.bottom() - 17.0)).abs() < 0.5, "the properties at the foot again");
    h.click(wid("dash", "dashed"));
    assert!(has(&h, "Not as the legend · Weight 0.6 → 0.8 · Line Solid → Dashed"), "{}", h.line_in(pen_area(&h)));
    // A selected line: its look against its row.
    h.click(wid("pen-state", "back"));
    let line = draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.settle();
    assert!(has(&h, "Pits · 1 selected line") && has(&h, "Same as the legend"), "{}", h.line_in(pen_area(&h)));
    h.app.run("stroke.set", json!({ "weight": 1.0 })).unwrap();
    h.settle();
    assert!(has(&h, "Not as the legend · Weight 0.6 → 1.0"), "{}", h.line_in(pen_area(&h)));
}

/// The pen of design 3.0 (§5 ⑧, the mockup's `.pen`): its head with 「Pen」 and ⋯; nine weight
/// chips 24 × 30, 2 apart, each its sample over ".1" … "1.0"; the line types, the eight colours and
/// ⋯, and the hatches, each in one row. Gone: the Field row, 0.4 dashed, ＋ for a weight, line or
/// fill, no colour, Cut Face, Add to Legend.
#[test]
fn the_pen_follows_the_mockup() {
    use vectorcraft_engine::cmd::legend::LADDER;
    let h = Harness::new();
    let t = h.tokens();
    let right = h.panel(RIGHT);
    let more = h.rect(wid("palette-more", ())).unwrap();
    let head = Rect::from_min_max(pos2(right.left(), more.top() - 5.0), pos2(right.right(), more.bottom() + 5.0));
    assert_eq!(h.texts_in(head), ["Pen", "Layer 1 · Next line"]);
    assert_eq!(h.font_of("Pen", head).map(|f| f.size), Some(13.5));
    let chips: Vec<Rect> = LADDER.iter().map(|w| h.rect(wid("weight", super::legend::pt(*w))).unwrap()).collect();
    let labels: Vec<String> = chips.iter().flat_map(|c| h.texts_in(*c)).collect();
    assert_eq!(labels, [".1", ".15", ".2", ".3", ".4", ".5", ".6", ".8", "1.0"]);
    for c in &chips {
        assert!((c.width() - 24.0).abs() < 0.01 && (c.height() - 30.0).abs() < 0.01, "{c:?}");
    }
    for pair in chips.windows(2) {
        assert!((pair[1].left() - pair[0].right() - 2.0).abs() < 0.01, "{pair:?}");
    }
    // Each chip's sample as thick as its weight shows.
    let sample = |c: Rect| h.rects_in(c).into_iter().filter(|(_, fill, _)| *fill == t.text).map(|(r, _, _)| r.height()).next().unwrap();
    assert!((sample(chips[0]) - 0.5).abs() < 0.01 && (sample(chips[8]) - 3.4).abs() < 0.01);
    let row =
        |kind: &str, keys: &[&str]| -> Vec<Rect> { keys.iter().map(|k| h.rect(wid(kind, *k)).unwrap_or_else(|| panic!("{kind} {k}"))).collect() };
    let dashes = row("dash", &["solid", "dashed", "dotted", "dashdot"]);
    let hatches = row("hatch", &["none", "45", "135", "alt"]);
    let swatches = row("pal-swatch", &["black", "red", "blue", "darkbrown", "brown", "ochre", "gray", "white"]);
    for parts in [&dashes, &hatches] {
        assert!(parts.iter().all(|p| (p.height() - 26.0).abs() < 0.01 && (p.top() - parts[0].top()).abs() < 0.01), "{parts:?}");
    }
    assert!(swatches.iter().all(|s| s.size() == vec2(20.0, 20.0) && (s.top() - swatches[0].top()).abs() < 0.01), "{swatches:?}");
    assert!(chips[0].bottom() < dashes[0].top() && dashes[0].bottom() < swatches[0].top() && swatches[0].bottom() < hatches[0].top());
    let labels = |parts: &[Rect]| parts.iter().flat_map(|p| h.texts_in(*p)).collect::<Vec<_>>();
    assert_eq!(labels(&dashes), ["Solid", "Dashed", "Dotted", "Dash-dot"]);
    assert_eq!(labels(&hatches), ["None", "45°", "135°", "Alternate"]);
    assert!(h.rect(wid("pal-color-more", ())).is_some());
    let pen = pen_area(&h);
    for label in ["Weight", "Line", "Colour", "Hatch"] {
        assert!(h.texts_in(pen).iter().any(|s| s == label), "{label} in {}", h.line_in(pen));
    }
    for gone in
        [wid("weight-add", ()), wid("pal-color-target", ()), wid("pal-color-none", ()), wid("palette-to-legend", ()), wid("palette-from-legend", ())]
    {
        assert!(h.rect(gone).is_none(), "{gone:?}");
    }
    assert!(h.rect(wid("weight", "0.25")).is_none());
}

/// The right column of design 3.0 (§4, §5 ⑦–⑨), 304 wide: from the top the legend, taking the
/// height left, the pen and the properties, folded to their head at the foot while nothing is
/// selected and open by themselves when something is, the pen going up over them.
#[test]
fn the_right_column_is_legend_pen_properties() {
    let mut h = Harness::new();
    let right = h.panel(RIGHT);
    assert!((right.width() - 304.0).abs() < 1.0, "{right:?}");
    let (legend, pen, props) = (in_right(&h, "Legend"), in_right(&h, "Pen"), in_right(&h, "Properties"));
    // The legend's head 36 at the top: its ＋ in the middle of it, its title beside.
    let plus = h.rect(wid("legend-new", ())).unwrap();
    assert!(
        (plus.center().y - (right.top() + 18.0)).abs() < 0.5 && (legend.center().y - plus.center().y).abs() < 3.0,
        "{legend:?} {plus:?} in {right:?}"
    );
    let fold = h.rect(wid("props-fold", ())).unwrap();
    assert!(
        (fold.center().y - (right.bottom() - 17.0)).abs() < 0.5 && (props.center().y - fold.center().y).abs() < 3.0,
        "the properties folded at the foot: {props:?} {fold:?} in {right:?}"
    );
    let pen_top = |h: &Harness| h.rect(wid("palette-more", ())).unwrap().top() - 7.0;
    assert!((pen_top(&h) - (right.bottom() - 34.0 - 192.0)).abs() < 0.5, "{}", pen_top(&h));
    assert!(legend.bottom() < pen.top() && pen.bottom() < props.top());
    assert!(!h.texts.iter().any(|p| p.text == "Legend · Layers"));
    // Something selected: the properties open (its size and where it is), the pen over them.
    draw_rect(&mut h, 10.0);
    h.settle();
    let props_top = h.rect(wid("props-fold", ())).unwrap().top() - 5.0;
    assert!((props_top - (right.bottom() - 96.0)).abs() < 0.5, "{props_top}");
    assert!((pen_top(&h) - (props_top - 192.0)).abs() < 0.5, "{}", pen_top(&h));
}

/// One list of weights everywhere (design 3.0 §2, §5 ⑧): 0.1 · 0.15 · 0.2 · 0.3 · 0.4 · 0.5 · 0.6 ·
/// 0.8 · 1.0 pt in the pen and in a row's editor, and Weights at Once's one step goes along it.
#[test]
fn one_weight_list_everywhere() {
    use vectorcraft_engine::cmd::legend::LADDER;
    assert_eq!(LADDER, [0.1, 0.15, 0.2, 0.3, 0.4, 0.5, 0.6, 0.8, 1.0]);
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.5}));
    h.settle();
    let keys: Vec<String> = LADDER.iter().map(|w| super::legend::pt(*w)).collect();
    for k in &keys {
        assert!(h.rect(wid("weight", k.as_str())).is_some(), "the pen's {k}");
    }
    h.double_click(wid("legend-row", pits.0));
    for k in &keys {
        assert!(h.rect(wid("card-weight", k.as_str())).is_some(), "the editor's {k}");
    }
    for k in ["0.25", "0.35", "0.7"] {
        assert!(h.rect(wid("weight", k)).is_none() && h.rect(wid("card-weight", k)).is_none(), "no {k}");
    }
    let step = |h: &Harness, dir: f64| {
        let doc = &h.app.session.active().unwrap().doc;
        vectorcraft_engine::cmd::legend::plan_weights(doc, &json!({"how": "step", "step": dir})).unwrap().rows[0].to
    };
    // From 0.5 the old list of six went to 0.8 and 0.3.
    assert_eq!((step(&h, 1.0), step(&h, -1.0)), (0.6, 0.4));
}

/// The Drawing menu's first two items run (criterion 20): Change Weights at Once… opens its card,
/// Revert to the Legend puts the selected lines back to their row's look, or every line set apart
/// when nothing is selected. The others still wait for their jobs.
#[test]
fn drawing_menu_opens_the_weights_card_and_reverts() {
    let mut h = Harness::new();
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.5})).unwrap();
    let (a, b) = (draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0));
    h.settle();
    let bar = h.panel(MENU);
    let open = |h: &mut Harness| {
        let at = h.texts.iter().find(|p| p.text == "Drawing" && bar.contains_rect(p.rect)).unwrap().rect.center();
        h.click_at(at);
        h.popup_buttons()
    };
    let items = open(&mut h);
    let enabled: Vec<bool> = items.iter().map(|(_, e)| *e).collect();
    // Seven run (Scale and Coordinate System… since J022), three wait for their jobs.
    assert_eq!(enabled, [true, true, true, true, true, true, true, false, false, false]);
    h.click_at(h.texts.iter().find(|p| p.text == "Drawing" && bar.contains_rect(p.rect)).unwrap().rect.center());
    h.app.run("select.set", json!({ "ids": [b.0] })).unwrap();
    h.settle();
    let items = open(&mut h);
    h.click_at(items[1].0.center());
    assert_eq!((weight_of(&h, a), weight_of(&h, b)), (0.5, 0.8), "the selected line");
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    let items = open(&mut h);
    h.click_at(items[1].0.center());
    assert_eq!(weight_of(&h, a), 0.8, "nothing selected: every line set apart");
    assert!(h.status_line().contains("Lines back to the legend: 1"), "{}", h.status_line());
    let items = open(&mut h);
    h.click_at(items[0].0.center());
    assert!(h.rect(wid("weights-card", ())).is_some(), "Weights at Once opens");
}

// ---------- J005 rework 1: what the code and UI reviews found ----------

/// The legend weight of row `id`.
fn row_weight(h: &Harness, id: NodeId) -> Option<f64> {
    h.app.session.active().unwrap().doc.node(id).unwrap().legend().and_then(|l| l.weight)
}

/// A ratio typed into Weights at Once and Change pressed straight after (no Enter) changes the
/// weights by what was typed; a ratio out of range is refused, says so and changes nothing
/// (code review C1).
#[test]
fn weights_card_changes_by_the_ratio_just_typed() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.open_weights();
    h.type_over(wid("weights-ratio", ()), "0.5");
    h.click(wid("weights-apply", ()));
    assert_eq!(row_weight(&h, pits), Some(0.4), "×0.5: {}", h.status_line());
    h.open_weights();
    h.type_over(wid("weights-ratio", ()), "20");
    h.click(wid("weights-apply", ()));
    assert_eq!(row_weight(&h, pits), Some(0.4), "refused: nothing changes");
    assert!(h.status_line().contains("The ratio is between 0.1 and 10"), "{}", h.status_line());
}

/// The texts on the line of Weights at Once's table that names `name`.
fn table_line(h: &Harness, name: &str) -> Vec<String> {
    let card = h.rect(wid("weights-card", ())).expect("the card");
    let cell = h.texts.iter().find(|p| p.text == name && card.contains_rect(p.rect)).unwrap_or_else(|| panic!("{name} in {}", h.line_in(card))).rect;
    h.texts.iter().filter(|p| card.contains_rect(p.rect) && same_line(p.rect, cell)).map(|p| p.text.clone()).collect()
}

/// Whether two texts sit on one line of a table: their glyphs overlap in height. (Not their
/// centres within 2 points: Hangul from one fallback font stands taller than digits, by up to 3.)
fn same_line(a: Rect, b: Rect) -> bool {
    a.top() < b.bottom() && b.top() < a.bottom()
}

/// The table follows a ratio being typed, before Enter: what it shows is what Change does (UI
/// review 2, the rest of code review C1).
#[test]
fn the_weights_table_follows_the_ratio_being_typed() {
    let mut h = Harness::new();
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.open_weights();
    assert!(table_line(&h, "Pits").contains(&"0.64".to_string()), "{:?}", table_line(&h, "Pits"));
    h.type_over(wid("weights-ratio", ()), "0.5");
    h.settle();
    assert!(table_line(&h, "Pits").contains(&"0.4".to_string()), "×0.5 as typed: {:?}", table_line(&h, "Pits"));
}

/// Change does the change and puts the card away, the status bar saying what changed: a second
/// press can't change the weights again unseen (UI review 2 U26).
#[test]
fn the_weights_card_closes_after_change() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.open_weights();
    h.click(wid("weights-apply", ()));
    assert_eq!(row_weight(&h, pits), Some(0.64));
    assert!(h.rect(wid("weights-card", ())).is_none(), "the card is put away");
    assert!(h.status_line().contains("Rows changed: 1 · lines: 0"), "{}", h.status_line());
}

/// Rows changing with no line following them: the card's note counts the rows rather than saying
/// that no line changes (UI review 2 U29).
#[test]
fn the_weights_note_counts_rows_when_no_line_follows() {
    let mut h = Harness::new();
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.open_weights();
    let note = h.line_in(h.rect(wid("weights-card", ())).unwrap());
    assert!(note.contains("Rows that change: 1. No line follows them. One Ctrl+Z puts them back."), "{note}");
    assert!(!note.contains("Lines that change"), "{note}");
}

/// A weight chip pressed while points are being typed in the row's editor wins, one undo step
/// (code review C6).
#[test]
fn a_weight_chip_pressed_while_typing_wins() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    h.type_over(wid("card-pt", ()), "0.4");
    h.click(wid("card-weight", "0.5"));
    assert_eq!(row_weight(&h, pits), Some(0.5));
    h.app.run("edit.undo", json!({})).unwrap();
    assert_eq!(row_weight(&h, pits), Some(0.8), "one undo step");
}

/// Esc while a card's field has the keyboard puts back what was typed and keeps the card; the next
/// Esc puts the card away (code review C7, UI review U5).
#[test]
fn esc_in_a_card_field_keeps_the_card() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    h.type_over(wid("card-name", ()), "Postholes");
    h.key(egui::Key::Escape);
    assert!(h.rect(wid("row-card", ())).is_some(), "the card stays");
    assert_eq!(h.app.session.active().unwrap().doc.node(pits).unwrap().display_name(), "Pits", "what was typed is put back");
    h.key(egui::Key::Escape);
    assert!(h.rect(wid("row-card", ())).is_none(), "the next Esc puts it away");
}

/// Weights at Once opens as it starts (by ratio, ×0.8) however it was left, even when another
/// drawing came to the front while it was open (code review C8).
#[test]
fn weights_card_opens_afresh() {
    let mut h = Harness::new();
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.open_weights();
    h.click(wid("weights-how", "value"));
    assert!(h.rect(wid("weights-from", ())).is_some());
    h.app.run("file.new", json!({"name": "Site 2", "width": 400, "height": 300})).unwrap();
    h.settle();
    assert!(h.rect(wid("weights-card", ())).is_none(), "it goes with its drawing");
    h.open_weights();
    let ratio = h.rect(wid("weights-ratio", ())).expect("by ratio");
    assert_eq!(h.texts_in(ratio), ["×", "0.8"]);
    assert!(h.rect(wid("weights-from", ())).is_none());
}

/// A press on the canvas while a card is open only puts the card away: the tool in hand doesn't
/// get it (UI review U1).
#[test]
fn a_press_on_the_canvas_only_puts_a_card_away() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.select_tool("pen");
    h.settle();
    let nodes = |h: &Harness| h.app.session.active().unwrap().doc.node_count();
    let n = nodes(&h);
    let canvas = h.app.canvas_rect.unwrap();
    h.double_click(wid("legend-row", pits.0));
    assert!(h.rect(wid("row-card", ())).is_some());
    h.click_at(canvas.center());
    assert!(h.rect(wid("row-card", ())).is_none(), "the card is put away");
    assert_eq!(nodes(&h), n, "the Pen got nothing");
    h.open_weights();
    assert!(h.rect(wid("weights-card", ())).is_some());
    h.click_at(canvas.center());
    assert!(h.rect(wid("weights-card", ())).is_none());
    assert_eq!(nodes(&h), n, "the Pen got nothing");
    h.click_at(canvas.center());
    assert!(nodes(&h) > n, "with no card the Pen draws");
}

/// The same with a touchpad's tap (a press and a release in one frame): it puts the card away and
/// the Pen gets nothing.
#[test]
fn a_tap_on_the_canvas_only_puts_a_card_away() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.select_tool("pen");
    h.settle();
    let nodes = |h: &Harness| h.app.session.active().unwrap().doc.node_count();
    let n = nodes(&h);
    let canvas = h.app.canvas_rect.unwrap();
    h.double_click(wid("legend-row", pits.0));
    assert!(h.rect(wid("row-card", ())).is_some());
    h.tap(canvas.center());
    h.settle();
    assert!(h.rect(wid("row-card", ())).is_none(), "the card is put away");
    assert_eq!(nodes(&h), n, "the Pen got nothing");
}

/// A double click on a row with a selection tool in hand and lines selected opens the row's editor
/// and leaves the lines where they were, as they were: the first click's move is taken back (code
/// review C2 option b, UI review U2).
#[test]
fn double_clicking_a_row_with_lines_selected_moves_nothing() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let line = draw_rect(&mut h, 10.0);
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.app.select_tool("selection");
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    assert!(h.rect(wid("row-card", ())).is_some(), "the editor opens");
    assert_eq!(h.app.session.active().unwrap().doc.layer_containing(line), Some(first), "the line stays");
    assert_eq!(weight_of(&h, line), 1.0, "with its own look");
    assert_eq!(h.app.session.active().unwrap().selection.objects, [line], "still selected");
    assert_eq!(h.current_layer(), Some(pits), "the row is the one drawn with");
    assert!(h.status_line().contains("The selected lines stay where they were"), "{}", h.status_line());
}

/// Two clicks on a row up to half a second apart open its editor (a system's usual double click);
/// further apart they are two picks (UI review U10).
#[test]
fn a_slow_double_click_opens_the_editor() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    let row = wid("legend-row", pits.0);
    // The clicks' releases 0.35 s apart plus the wait.
    h.click(row);
    h.later(0.1);
    h.click(row);
    assert!(h.rect(wid("row-card", ())).is_some(), "0.45 s apart: the editor");
    h.click(wid("card-close", ()));
    h.later(1.0);
    h.click(row);
    h.later(0.25);
    h.click(row);
    assert!(h.rect(wid("row-card", ())).is_none(), "0.6 s apart: two picks");
}

/// A touchpad's tap comes as a press and a release in one frame. Two taps on a row open its
/// editor, and do so again after a card was shown and put away: the press of the tap that opens a
/// card doesn't put it away.
#[test]
fn two_taps_open_the_editor_again() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    let at = h.rect(wid("legend-row", pits.0)).unwrap().center();
    for round in 0..2 {
        h.tap(at);
        h.tap(at);
        h.settle();
        assert!(h.rect(wid("row-card", ())).is_some(), "round {round}: the editor opens");
        h.click(wid("card-close", ()));
        assert!(h.rect(wid("row-card", ())).is_none(), "round {round}: ✕ puts it away");
    }
    // With the editor open, the legend's ＋ tapped opens the new row's editor.
    h.tap(at);
    h.tap(at);
    h.settle();
    h.tap(h.rect(wid("legend-new", ())).unwrap().center());
    h.settle();
    let card = h.rect(wid("row-card", ())).expect("the new row's editor");
    assert!(h.texts_in(card).iter().any(|s| s.starts_with("Legend Row ")), "{:?}", h.texts_in(card));
}

/// After the plan legend is added its Underlay row is there but empty: the Underlay tile still
/// places a scan (UI review U3, J003's tile).
#[test]
fn the_underlay_tile_places_a_scan_beside_the_empty_underlay_row() {
    let mut h = Harness::new();
    let dir = std::env::temp_dir().join(format!("trowel-underlay-row-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let scan = dir.join("scan.svg");
    std::fs::write(&scan, r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30"/></svg>"#).unwrap();
    let path = scan.to_string_lossy().to_string();
    h.app.services.pick_open = Some(Box::new(move |_| Some(path.clone())));
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    h.settle();
    let t = h.tokens();
    let tile = h.rect(wid("tile", "underlay")).unwrap();
    let well = Rect::from_min_size(pos2(tile.center().x - 22.0, tile.top()), vec2(44.0, 44.0));
    assert_ne!(h.fill_at(well), Some(t.row_selected), "no scan to show");
    h.click(wid("tile", "underlay"));
    let d = h.app.ui.dialog.as_ref().expect("the Place dialog");
    assert_eq!(d.kind, crate::dialogs::place::KIND);
    assert_eq!(d.fields.get("template"), Some(&json!(true)), "Template ticked");
    std::fs::remove_dir_all(&dir).ok();
}

/// The Underlay tile's scan goes under every line: on a locked template layer at the bottom of the
/// stack, named after the tile and the file, while the row drawn with stays (UI review 2 U24).
#[test]
fn the_underlay_tiles_scan_goes_under_every_line() {
    let mut h = Harness::new();
    let dir = std::env::temp_dir().join(format!("trowel-underlay-bottom-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let scan = dir.join("scan.svg");
    std::fs::write(&scan, r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30"/></svg>"#).unwrap();
    let path = scan.to_string_lossy().to_string();
    h.app.services.pick_open = Some(Box::new(move |_| Some(path.clone())));
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    let pit = h.app.session.active().unwrap().doc.layers.iter().find(|l| l.display_name() == "굴광선 (유구 면)").unwrap().id;
    h.app.run("legend.pick", json!({ "id": pit.0 })).unwrap();
    h.settle();
    h.click(wid("tile", "underlay"));
    crate::dialogs::confirm(&mut h.app).unwrap();
    h.settle();
    let doc = h.app.session.active().unwrap().doc.clone();
    let bottom = doc.layers.first().unwrap();
    assert_eq!(bottom.display_name(), "Underlay · scan.svg");
    assert!(bottom.is_template() && bottom.locked && bottom.children().is_some_and(|c| !c.is_empty()), "the scan, locked, on a template layer");
    assert_eq!(h.current_layer(), Some(pit), "the row drawn with stays");
    std::fs::remove_dir_all(&dir).ok();
}

/// The closed outlines painted inside `area` with three corners: where they are and their edge.
fn triangles_in(h: &Harness, area: Rect) -> Vec<(Rect, egui::epaint::PathStroke)> {
    h.shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Path(p) if p.closed && p.points.len() == 3 && p.points.iter().all(|q| area.contains(*q)) => {
                Some((Rect::from_points(&p.points), p.stroke.clone()))
            }
            _ => None,
        })
        .collect()
}

/// The mark samples are as large as the mockup's: △ is its triangle (10 × 9 inside 12 × 11, one
/// point of ink), other marks are 12 points (UI review U4).
#[test]
fn mark_samples_are_as_large_as_the_mockup() {
    let mut h = Harness::new();
    let t = h.tokens();
    let datum = legend_row(&mut h, "Datum", json!({"weight": null, "sample": {"mark": "△"}}));
    let finds = legend_row(&mut h, "Finds", json!({"weight": 0.2, "sample": {"mark": "③"}}));
    h.settle();
    let dr = h.rect(wid("legend-row", datum.0)).unwrap();
    let slot = Rect::from_min_size(pos2(dr.left() + 56.0, dr.center().y - 5.0), vec2(34.0, 10.0));
    let found = triangles_in(&h, slot.expand(1.0));
    let [(b, stroke)] = found.as_slice() else { panic!("one triangle: {found:?}") };
    assert!((b.width() - 10.0).abs() < 0.6 && (b.height() - 9.0).abs() < 0.6, "{b:?}");
    assert!((b.center().x - slot.center().x).abs() < 0.6, "centred: {b:?} in {slot:?}");
    assert_eq!((stroke.width, stroke.color.clone()), (1.0, egui::epaint::ColorMode::Solid(t.text)));
    let fr = h.rect(wid("legend-row", finds.0)).unwrap();
    assert_eq!(h.font_of("③", fr).map(|f| f.size), Some(12.0));
}

/// What is cut short says itself in full on hover, and 「Apart N」 says what it means: a long row
/// name, the badge, a long card head (cut with "…"), a long line of Weights at Once's table (UI
/// review U6, U11).
#[test]
fn cut_names_and_apart_badges_say_themselves_on_hover() {
    let mut h = Harness::sized(1600.0, 1400.0);
    let long = "Pit outline of the northern dwelling, second phase";
    let pits = legend_row(&mut h, long, json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.5})).unwrap();
    draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    let row = h.rect(wid("legend-row", pits.0)).unwrap();
    let tip = |h: &Harness, s: &str| h.texts.iter().any(|p| p.text == s);
    h.rest_at(pos2(row.left() + 140.0, row.center().y));
    assert!(tip(&h, &format!("{long}\nDraw with this row · double-click to change it")), "the name in full");
    let badge = h.texts.iter().find(|p| p.text == "Apart 1" && row.contains_rect(p.rect)).unwrap().rect;
    h.rest_at(badge.center());
    assert!(tip(&h, "Lines of this row with a weight or line type of their own: 1"), "what Apart means");
    h.double_click(wid("legend-row", pits.0));
    let (card, close) = (h.rect(wid("row-card", ())).unwrap(), h.rect(wid("card-close", ())).unwrap());
    let head = h.texts.iter().find(|p| p.text == long && card.contains(p.rect.left_center()) && p.rect.top() < close.bottom()).unwrap().rect;
    assert!(head.right() <= close.left(), "the card's head cut short with …: {head:?}, ✕ {close:?}");
    h.click(wid("card-close", ()));
    // Weights at Once's table: the longest line of rows changing alike.
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    h.settle();
    h.open_weights();
    let card = h.rect(wid("weights-card", ())).unwrap();
    let longest = plan_table(&h, json!({"how": "ratio", "ratio": 0.8})).into_iter().map(|r| r.0).max_by_key(|n| n.chars().count()).unwrap();
    let cell = h.texts.iter().find(|p| p.text == longest && card.contains_rect(p.rect)).unwrap().rect;
    h.rest_at(pos2(cell.left() + 20.0, cell.center().y));
    assert!(h.texts.iter().filter(|p| p.text == longest).count() >= 2, "{longest} on hover");
}

/// With no drawing open the legend's ⋯ only opens the Layers panel: there is nothing else to work
/// on (UI review U7).
#[test]
fn the_legend_more_items_wait_for_a_drawing() {
    let mut h = Harness::new();
    h.app.run("file.close", json!({})).unwrap();
    h.settle();
    assert!(h.app.session.active().is_none());
    h.click(wid("legend-more", ()));
    let items = h.popup_buttons();
    assert_eq!(items.iter().map(|(_, e)| *e).collect::<Vec<_>>(), [false, false, false, false, false, true]);
}

/// Dimmed controls say on hover why they wait: a weightless row's line types for a weight (UI
/// review 2 U28).
#[test]
fn dimmed_controls_say_why_on_hover() {
    let mut h = Harness::new();
    let datum = legend_row(&mut h, "Datum", json!({"weight": null, "sample": {"mark": "△"}}));
    h.settle();
    h.double_click(wid("legend-row", datum.0));
    h.rest_at(h.rect(wid("card-line", "dashed")).unwrap().center());
    let shown: Vec<&str> = h.texts.iter().map(|p| p.text.as_str()).collect();
    assert!(shown.iter().any(|s| s.contains("A row without a weight has no line type")), "{shown:?}");
}

/// A click on a locked or hidden row says why it isn't drawn with and leaves the current layer as
/// it was (UI review U8).
#[test]
fn a_locked_or_hidden_row_is_not_picked() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("layer.setCurrent", json!({ "id": first.0 })).unwrap();
    h.app.run("layer.setProps", json!({ "ids": [pits.0], "locked": true })).unwrap();
    h.settle();
    h.click(wid("legend-row", pits.0));
    assert_eq!(h.current_layer(), Some(first));
    assert!(h.status_line().contains("Pits is locked: unlock it to draw with it"), "{}", h.status_line());
    h.app.run("layer.setProps", json!({ "ids": [pits.0], "locked": false, "visible": false })).unwrap();
    h.later(1.0);
    h.click(wid("legend-row", pits.0));
    assert_eq!(h.current_layer(), Some(first));
    assert!(h.status_line().contains("Pits is hidden: show it to draw with it"), "{}", h.status_line());
}

/// A row without a weight has no look: its line types are dimmed and change nothing (UI review U9).
#[test]
fn a_row_without_a_weight_has_its_line_types_dimmed() {
    let mut h = Harness::new();
    let datum = legend_row(&mut h, "Datum", json!({"weight": null, "sample": {"mark": "△"}}));
    h.settle();
    h.double_click(wid("legend-row", datum.0));
    assert_eq!(h.enabled(wid("card-line", "dashed")), Some(false));
    h.click(wid("card-line", "dashed"));
    let line = h.app.session.active().unwrap().doc.node(datum).unwrap().legend().unwrap().line.clone();
    assert_eq!(line, vectorcraft_doc::LineType::Solid);
    // With a weight they run.
    h.click(wid("card-weight", "0.5"));
    assert_eq!(h.enabled(wid("card-line", "dashed")), Some(true));
}

/// The cards' sizes and type as in the mockup (figures ② and ③, UI review nits U12–U17): chip
/// rows 24, the line under the Strata field 18, "= mm" in
/// 12 points, "pt" right after the number, the row's line count in mono, ×0.8 in mono ink, the
/// table's lines 34 and its head 30.
#[test]
fn cards_match_the_mockup_sizes() {
    let mut h = Harness::sized(1600.0, 1400.0);
    let t = h.tokens();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.8})).unwrap();
    draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    let card = h.rect(wid("row-card", ())).unwrap();
    assert_eq!(h.rect(wid("card-weight", "0.5")).unwrap().height(), 24.0);
    assert_eq!(h.rect(wid("card-line", "solid")).unwrap().height(), 24.0);
    let strata = h.rect(wid("card-strata", ())).unwrap();
    let note = h.texts.iter().find(|p| p.text == "This row is the layer" && card.contains_rect(p.rect)).unwrap().rect;
    let chip = h.rect(wid("card-weight", "0.5")).unwrap();
    assert!((chip.top() - strata.bottom() - (5.0 + 18.0 + 5.0)).abs() < 0.6, "18 for the line between: {strata:?} {note:?} {chip:?}");
    assert_eq!(h.font_of("= 0.28 mm", card).map(|f| f.size), Some(12.0));
    let field = h.rect(wid("card-pt", ())).unwrap();
    let number = h.texts.iter().find(|p| p.text == "0.8" && field.contains_rect(p.rect)).unwrap().rect;
    let unit = h.texts.iter().find(|p| p.text == "pt" && field.contains_rect(p.rect)).unwrap().rect;
    assert!(unit.left() - number.right() < 8.0, "pt right after the number: {number:?} {unit:?}");
    assert_eq!(h.font_of("Lines: 1", card).map(|f| f.family), Some(crate::theme::mono(12.0).family));
    h.click(wid("card-close", ()));
    h.open_weights();
    let card = h.rect(wid("weights-card", ())).unwrap();
    assert_eq!(h.rect(wid("weights-how", "ratio")).unwrap().height(), 24.0);
    let ratio = h.rect(wid("weights-ratio", ())).unwrap();
    assert_eq!(h.font_of("×", ratio).map(|f| f.family), Some(crate::theme::mono(11.0).family));
    assert_eq!(h.color_of("×", ratio), Some(t.text));
    let head = h.texts.iter().find(|p| p.text == "Legend Row" && card.contains_rect(p.rect)).unwrap().rect;
    let first = h.texts.iter().find(|p| p.text == "Pits" && card.contains_rect(p.rect)).unwrap().rect;
    assert!((first.center().y - head.center().y - (30.0 / 2.0 + 34.0 / 2.0)).abs() < 1.0, "head 30, lines 34: {head:?} {first:?}");
}

/// In a small window a card pushed up to clear the status bar stops under the tab row and scrolls
/// instead (UI review U23).
#[test]
fn a_card_stays_under_the_tab_row() {
    let mut h = Harness::sized(1366.0, 728.0);
    let first = h.layer_ids()[0];
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.double_click(wid("legend-row", first.0));
    let c = h.rect(wid("row-card", ())).expect("the editor");
    let canvas = h.app.canvas_rect.unwrap();
    assert!(c.top() >= canvas.top() - 0.5, "{c:?} under the tab row, canvas {canvas:?}");
    assert!(c.bottom() <= h.panel(STATUS).top() + 0.5, "{c:?}");
}

/// A press on something a later job brings that opens a dialog names it without its "…" (code
/// review C14: nothing on screen reaches this now that Change Weights at Once… runs).
#[test]
fn a_soon_note_drops_a_dialogs_ellipsis() {
    let ctx = egui::Context::default();
    super::note_soon(&ctx, "Change All Weights…", "J005");
    assert_eq!(super::soon_note(&ctx).as_deref(), Some("Change All Weights: coming soon (J005)"));
}

// ==================== Task J004 Tests ====================

#[test]
fn test_drawing_book_path_resolution() {
    use super::book::ResolvedDrawingBook;

    let base_dir = std::env::temp_dir()
        .join(format!("trowel_book_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let survey_dir = base_dir.join("광령리1");
    let book_dir = survey_dir.join("도면");
    let _ = std::fs::create_dir_all(&book_dir);

    let gpkg_path = survey_dir.join("광령리1.gpkg");
    let _ = std::fs::write(&gpkg_path, b"mock gpkg file");

    // 1. Resolve from explicit .gpkg file
    let b1 = ResolvedDrawingBook::resolve(&gpkg_path);
    assert_eq!(b1.name, "광령리1");
    assert_eq!(b1.survey_dir, survey_dir);
    assert_eq!(b1.drawing_book_dir, book_dir);
    assert!(b1.strata_status.is_connected());

    // 2. Resolve from survey directory
    let b2 = ResolvedDrawingBook::resolve(&survey_dir);
    assert_eq!(b2.name, "광령리1");
    assert_eq!(b2.survey_dir, survey_dir);
    assert_eq!(b2.drawing_book_dir, book_dir);
    assert!(b2.strata_status.is_connected());

    // 3. Resolve from <survey_dir>/도면
    let b3 = ResolvedDrawingBook::resolve(&book_dir);
    assert_eq!(b3.name, "광령리1");
    assert_eq!(b3.survey_dir, survey_dir);
    assert_eq!(b3.drawing_book_dir, book_dir);
    assert!(b3.strata_status.is_connected());

    // 4. Resolve from a drawing file inside 도면
    let dwg_path = book_dir.join("1호 주거지 평면.vectorcraft");
    let _ = std::fs::write(&dwg_path, b"{}");
    let b4 = ResolvedDrawingBook::resolve(&dwg_path);
    assert_eq!(b4.drawing_book_dir, book_dir);

    // 5. Drawing categorizer test
    let _ = std::fs::write(book_dir.join("A-A 단면.vectorcraft"), b"{}");
    let _ = std::fs::write(book_dir.join("토층도.drawcraft"), b"{}");
    let _ = std::fs::write(book_dir.join("1호 토기 유물.trowel"), b"{}");
    let _ = std::fs::write(book_dir.join("보고서 도판.vectorcraft"), b"{}");

    let (plan, section, finds, plate) = ResolvedDrawingBook::scan_drawings(&book_dir);
    assert_eq!(plan, 1); // 1호 주거지 평면
    assert_eq!(section, 2); // A-A 단면, 토층도
    assert_eq!(finds, 1); // 1호 토기 유물
    assert_eq!(plate, 1); // 보고서 도판

    // Clean up
    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_recent_drawing_books_operations() {
    use super::book::{RecentDrawingBooks, ResolvedDrawingBook, StrataGpkgStatus};
    use std::path::PathBuf;

    let mut recents = RecentDrawingBooks::new();
    assert_eq!(recents.books.len(), 3);
    assert_eq!(recents.books[0].name, "광령리1");

    // Remove at index
    recents.remove_at(1);
    assert_eq!(recents.books.len(), 2);
    assert_eq!(recents.books[1].name, "하가리 시굴");

    // Add new
    let new_book = ResolvedDrawingBook {
        name: "서귀포 유적".to_string(),
        survey_dir: PathBuf::from("D:\\발굴\\서귀포"),
        drawing_book_dir: PathBuf::from("D:\\발굴\\서귀포\\도면"),
        strata_status: StrataGpkgStatus::Disconnected,
        plan_count: 5,
        section_count: 2,
        finds_count: 3,
        plate_count: 1,
        crs: "5186 중부원점".to_string(),
        last_opened: "방금 전".to_string(),
    };
    recents.add(new_book);
    assert_eq!(recents.books.len(), 3);
    assert_eq!(recents.books[0].name, "서귀포 유적");

    // Remove by path
    recents.remove_by_path(&PathBuf::from("D:\\발굴\\서귀포\\도면"));
    assert_eq!(recents.books.len(), 2);
    assert_eq!(recents.books[0].name, "광령리1");
}

#[test]
fn test_headless_start_screen_frame_rendering() {
    let mut h = Harness::sized(1600.0, 1061.0);

    // Switch to Home screen
    h.app.run("app.home", json!({})).unwrap();
    h.settle();

    assert!(crate::menus::home_showing(&h.app), "Home screen must be showing");

    // Verify key text elements from §2
    let text_dump: String = h.texts.iter().map(|t| t.text.as_str()).collect::<Vec<_>>().join(" ");

    // Brand & Title
    assert!(text_dump.contains("Trowel"), "Must contain brand 'Trowel'");
    assert!(text_dump.contains("발굴 도면"), "Must contain title '발굴 도면'");
    assert!(text_dump.contains("도면집을 이어서 열거나, 새 도면을 시작하세요."), "Must contain subtitle");

    // Quick Actions
    assert!(text_dump.contains("＋ 새 도면"), "Must contain '＋ 새 도면'");
    assert!(text_dump.contains("도면집 열기"), "Must contain '도면집 열기'");
    assert!(text_dump.contains("Strata 조사에서 시작"), "Must contain 'Strata 조사에서 시작'");

    // Resume Work Card
    assert!(text_dump.contains("이어서 작업"), "Must contain '이어서 작업'");
    assert!(text_dump.contains("1호 주거지 평면"), "Must contain '1호 주거지 평면'");
    assert!(text_dump.contains("이어서 그리기"), "Must contain '이어서 그리기'");
    assert!(text_dump.contains("도면집 보기"), "Must contain '도면집 보기'");

    // Recent Books Table
    assert!(text_dump.contains("최근 도면집"), "Must contain '최근 도면집'");
    assert!(text_dump.contains("오른쪽 클릭: 목록에서 빼기"), "Must contain context menu hint");
    assert!(text_dump.contains("광령리1"), "Must contain '광령리1'");
    assert!(text_dump.contains("중문동 2지점"), "Must contain '중문동 2지점'");
    assert!(text_dump.contains("하가리 시굴"), "Must contain '하가리 시굴'");
    assert!(text_dump.contains("5186 중부원점"), "Must contain CRS '5186 중부원점'");

    // Workflow Rail
    assert!(text_dump.contains("작업 순서"), "Must contain '작업 순서'");
    assert!(text_dump.contains("1. 자료 받기"), "Must contain '1. 자료 받기'");
    assert!(text_dump.contains("2. 밑그림 맞추기"), "Must contain '2. 밑그림 맞추기'");
    assert!(text_dump.contains("3. 평면 그리기"), "Must contain '3. 평면 그리기'");
    assert!(text_dump.contains("다음"), "Must contain '다음' tag");
    assert!(text_dump.contains("4. 단면·토층"), "Must contain '4. 단면·토층'");
    assert!(text_dump.contains("5. 유물 실측"), "Must contain '5. 유물 실측'");
    assert!(text_dump.contains("6. 도판 만들기"), "Must contain '6. 도판 만들기'");
    assert!(text_dump.contains("7. 내보내기"), "Must contain '7. 내보내기'");

    // Archaeological Templates
    assert!(text_dump.contains("새 도면 견본"), "Must contain '새 도면 견본'");
    assert!(text_dump.contains("개별유구 평·단면도"), "Must contain '개별유구 평·단면도'");
    assert!(text_dump.contains("토층 단면도"), "Must contain '토층 단면도'");
    assert!(text_dump.contains("유물 실측도"), "Must contain '유물 실측도'");
    assert!(text_dump.contains("유구배치도"), "Must contain '유구배치도'");
    assert!(text_dump.contains("보고서 도판"), "Must contain '보고서 도판'");

    // Connection Summary Bar
    assert!(text_dump.contains("Strata 조사 파일"), "Must contain 'Strata 조사 파일'");
    assert!(text_dump.contains("desca 단면"), "Must contain 'desca 단면'");
    assert!(text_dump.contains("측점"), "Must contain '측점'");
    assert!(text_dump.contains("밑그림"), "Must contain '밑그림'");
    assert!(text_dump.contains("보고서 글꼴"), "Must contain '보고서 글꼴'");

    // Status Bar Readout
    assert!(text_dump.contains("「이어서 그리기」를 누르면 하던 도면이 그대로 열립니다."), "Must contain status bar message");
    // No made-up count of drawing sets in the status bar (design 3.0 §5 ⑩).
    assert!(!h.line_in(h.panel(STATUS)).contains("도면집"), "{}", h.line_in(h.panel(STATUS)));
    assert!(text_dump.contains("Trowel 0.2"), "Must contain status bar version");
}

#[test]
fn test_template_card_click_creates_standard_drawing() {
    let mut h = Harness::sized(1600.0, 1061.0);
    h.app.run("app.home", json!({})).unwrap();
    h.settle();
    assert!(crate::menus::home_showing(&h.app));

    // Dispatch finds template creation command
    h.app.run("trowel.newFinds", json!({})).unwrap();
    h.app.ui.home = None;
    h.settle();

    // Verifies Home screen closes
    assert!(!crate::menus::home_showing(&h.app), "Home screen must close after creating drawing");

    // Verifies document properties
    let doc = h.app.session.active().expect("Document must be active");
    assert_eq!(doc.doc.units, vectorcraft_doc::Unit::Millimeters, "Units must be Millimeters");
    let scale = doc.doc.scale.as_ref().expect("Scale must be configured");
    assert_eq!(scale.scale.denom, 1.0, "Finds scale must be 1:1");

    // Verifies initial finds working layer exists
    let layers: Vec<String> = doc.doc.layers.iter().map(|l| l.display_name().to_string()).collect();
    assert!(layers.iter().any(|n| n.contains("외형선")), "Must have '외형선' initial layer for finds template");
}

#[test]
fn test_ribbon_new_drawing_tiles_dispatch() {
    let mut h = Harness::sized(1600.0, 988.0);
    // Click ribbon tiles
    h.app.run("trowel.newSection", json!({})).unwrap();
    h.settle();

    let doc = h.app.session.active().expect("Section document must be active");
    assert_eq!(doc.doc.units, vectorcraft_doc::Unit::Millimeters);
    let scale = doc.doc.scale.as_ref().expect("Scale must be configured");
    assert_eq!(scale.scale.denom, 20.0, "Section scale must be 1:20");
    let layers: Vec<String> = doc.doc.layers.iter().map(|l| l.display_name().to_string()).collect();
    assert!(layers.iter().any(|n| n.contains("단면선") || n.contains("레벨선") || n.contains("토층선")), "Must contain section legend layers");
}

#[test]
fn test_adversarial_drawing_book_resolve() {
    use super::book::{ResolvedDrawingBook, StrataGpkgStatus};

    let base_dir = std::env::temp_dir()
        .join(format!("trowel_adv_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&base_dir);

    // 1. Target is a survey folder containing .gpkg -> confirms 도면 subfolder is used and .gpkg status is Connected
    let survey_dir = base_dir.join("광령리1");
    let book_dir = survey_dir.join("도면");
    let _ = std::fs::create_dir_all(&book_dir);
    let gpkg_path = survey_dir.join("광령리1.gpkg");
    let _ = std::fs::write(&gpkg_path, b"mock gpkg");

    let b1 = ResolvedDrawingBook::resolve(&survey_dir);
    assert_eq!(b1.survey_dir, survey_dir);
    assert_eq!(b1.drawing_book_dir, book_dir);
    assert!(b1.strata_status.is_connected(), "Survey folder with .gpkg must have Connected status");
    if let StrataGpkgStatus::Connected { filename, path } = &b1.strata_status {
        assert_eq!(filename, "광령리1.gpkg");
        assert_eq!(path, &gpkg_path);
    }

    // 2. Target is already inside 도면 subfolder -> confirms parent survey folder is located and .gpkg status is Connected
    let b2 = ResolvedDrawingBook::resolve(&book_dir);
    assert_eq!(b2.survey_dir, survey_dir);
    assert_eq!(b2.drawing_book_dir, book_dir);
    assert!(b2.strata_status.is_connected(), "도면 subfolder target must locate parent and have Connected status");

    // 3. Target folder has NO .gpkg -> what is the status?
    let no_gpkg_dir = base_dir.join("중문동");
    let no_gpkg_book_dir = no_gpkg_dir.join("도면");
    let _ = std::fs::create_dir_all(&no_gpkg_book_dir);
    let b3 = ResolvedDrawingBook::resolve(&no_gpkg_dir);
    assert_eq!(b3.drawing_book_dir, no_gpkg_book_dir);
    // CHALLENGE: Check if it's Missing vs Disconnected
    eprintln!("[CHALLENGE] Target folder with NO .gpkg status: {:?}", b3.strata_status);

    // 4. Target is a standalone .drawcraft or .vectorcraft file outside 도면 -> confirms Disconnected status
    let standalone_file = base_dir.join("standalone_drawing.vectorcraft");
    let _ = std::fs::write(&standalone_file, b"{}");
    let b4 = ResolvedDrawingBook::resolve(&standalone_file);
    eprintln!("[CHALLENGE] Standalone drawing file status: {:?}", b4.strata_status);
    assert_eq!(b4.strata_status, StrataGpkgStatus::Disconnected, "Standalone drawing file must have Disconnected status");

    // 5. Category scanning (scan_drawings): verify correct classification
    let cat_test_dir = base_dir.join("cat_test_도면");
    let _ = std::fs::create_dir_all(&cat_test_dir);
    let _ = std::fs::write(cat_test_dir.join("1호 주거지 평면.vectorcraft"), b"{}");
    let _ = std::fs::write(cat_test_dir.join("전체배치.drawcraft"), b"{}"); // fallback to plan
    let _ = std::fs::write(cat_test_dir.join("A-A 단면.vectorcraft"), b"{}");
    let _ = std::fs::write(cat_test_dir.join("트렌치 토층도.drawcraft"), b"{}");
    let _ = std::fs::write(cat_test_dir.join("1호 토기 유물.trowel"), b"{}");
    let _ = std::fs::write(cat_test_dir.join("완형토기 실측.vectorcraft"), b"{}");
    let _ = std::fs::write(cat_test_dir.join("보고서 도판 1.vectorcraft"), b"{}");
    // Non-drawing files should NOT be counted:
    let _ = std::fs::write(cat_test_dir.join("notes.txt"), b"some text");
    let _ = std::fs::write(cat_test_dir.join("survey.gpkg"), b"binary");
    let _ = std::fs::write(cat_test_dir.join("photo.png"), b"image");

    let (plan, section, finds, plate) = ResolvedDrawingBook::scan_drawings(&cat_test_dir);
    assert_eq!(plan, 2, "plan count mismatch");
    assert_eq!(section, 2, "section count mismatch");
    assert_eq!(finds, 2, "finds count mismatch");
    assert_eq!(plate, 1, "plate count mismatch");

    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_adversarial_recent_drawing_books_operations() {
    use super::book::{RecentDrawingBooks, ResolvedDrawingBook, StrataGpkgStatus};
    use std::path::PathBuf;

    let mut recents = RecentDrawingBooks::new();
    assert_eq!(recents.books.len(), 3);

    // 1. remove_at out-of-bounds does not panic
    recents.remove_at(100);
    assert_eq!(recents.books.len(), 3, "out-of-bounds remove_at must not change length or panic");

    // 2. remove_at valid
    recents.remove_at(0);
    assert_eq!(recents.books.len(), 2);
    assert_eq!(recents.books[0].name, "중문동 2지점");

    // 3. add deduplicates if drawing_book_dir is the same
    let duplicate = ResolvedDrawingBook {
        name: "중문동 2지점 신규".to_string(),
        survey_dir: PathBuf::from("D:\\발굴\\중문동"),
        drawing_book_dir: PathBuf::from("D:\\발굴\\중문동\\도면"),
        strata_status: StrataGpkgStatus::Disconnected,
        plan_count: 10,
        section_count: 5,
        finds_count: 2,
        plate_count: 1,
        crs: "5186 중부원점".to_string(),
        last_opened: "방금".to_string(),
    };
    recents.add(duplicate);
    assert_eq!(recents.books.len(), 2, "add with duplicate drawing_book_dir must replace/deduplicate");
    assert_eq!(recents.books[0].name, "중문동 2지점 신규");

    // 4. remove_by_path using survey_dir
    recents.remove_by_path(&PathBuf::from("D:\\발굴\\중문동"));
    assert_eq!(recents.books.len(), 1);
    assert_eq!(recents.books[0].name, "하가리 시굴");

    // 5. remove_by_path non-existent path
    recents.remove_by_path(&PathBuf::from("D:\\발굴\\없는경로"));
    assert_eq!(recents.books.len(), 1);
}

#[test]
fn test_adversarial_recent_books_json_roundtrip() {
    use super::book::{RecentDrawingBooks, StrataGpkgStatus};
    use std::path::PathBuf;

    // 1. Verify default recent books serialization containing multiple books & statuses:
    //    - Connected: 광령리1
    //    - Disconnected: 중문동 2지점
    //    - Missing: 하가리 시굴
    let recents = RecentDrawingBooks::new();
    assert_eq!(recents.books.len(), 3);
    assert!(recents.books[0].strata_status.is_connected());
    assert_eq!(recents.books[1].strata_status, StrataGpkgStatus::Disconnected);
    assert!(recents.books[2].strata_status.is_missing());

    // Serialize using serde_json directly
    let json_str = serde_json::to_string(&recents).expect("serialization failed");
    assert!(!json_str.is_empty(), "serialized JSON string must not be empty");
    assert!(json_str.contains("광령리1"), "JSON must preserve book names");
    assert!(json_str.contains("하가리 시굴"), "JSON must preserve missing book name");
    assert!(json_str.contains("Connected"), "JSON must serialize Connected status");
    assert!(json_str.contains("Disconnected"), "JSON must serialize Disconnected status");
    assert!(json_str.contains("Missing"), "JSON must serialize Missing status");

    // Deserialize and verify structural equality
    let roundtripped: RecentDrawingBooks = serde_json::from_str(&json_str).expect("deserialization failed");
    assert_eq!(recents, roundtripped, "Deserialized RecentDrawingBooks must be identical to original");

    // Detailed verification of all 3 books and statuses after roundtrip
    assert_eq!(roundtripped.books.len(), 3);

    // Book 0: Connected
    assert_eq!(roundtripped.books[0].name, "광령리1");
    match &roundtripped.books[0].strata_status {
        StrataGpkgStatus::Connected { filename, path } => {
            assert_eq!(filename, "광령리1.gpkg");
            assert_eq!(path, &PathBuf::from("D:\\발굴\\광령리1\\광령리1.gpkg"));
        }
        other => panic!("Expected Connected status for book 0, got {:?}", other),
    }
    assert_eq!(roundtripped.books[0].plan_count, 4);
    assert_eq!(roundtripped.books[0].section_count, 3);
    assert_eq!(roundtripped.books[0].finds_count, 8);
    assert_eq!(roundtripped.books[0].plate_count, 2);
    assert_eq!(roundtripped.books[0].crs, "5186 중부원점");

    // Book 1: Disconnected
    assert_eq!(roundtripped.books[1].name, "중문동 2지점");
    assert_eq!(roundtripped.books[1].strata_status, StrataGpkgStatus::Disconnected);
    assert!(!roundtripped.books[1].strata_status.is_connected());
    assert!(!roundtripped.books[1].strata_status.is_missing());

    // Book 2: Missing
    assert_eq!(roundtripped.books[2].name, "하가리 시굴");
    match &roundtripped.books[2].strata_status {
        StrataGpkgStatus::Missing { filename, expected_path } => {
            assert_eq!(filename, "하가리.gpkg");
            assert_eq!(expected_path, &PathBuf::from("D:\\발굴\\하가리\\하가리.gpkg"));
        }
        other => panic!("Expected Missing status for book 2, got {:?}", other),
    }

    // 2. Roundtrip empty RecentDrawingBooks
    let empty = RecentDrawingBooks { books: vec![] };
    let empty_json = serde_json::to_string(&empty).expect("empty serialization failed");
    let roundtripped_empty: RecentDrawingBooks = serde_json::from_str(&empty_json).expect("empty deserialization failed");
    assert_eq!(empty, roundtripped_empty);
    assert!(roundtripped_empty.books.is_empty());

    // 3. Roundtrip pretty-printed JSON
    let pretty_json = serde_json::to_string_pretty(&recents).expect("pretty serialization failed");
    let roundtripped_pretty: RecentDrawingBooks = serde_json::from_str(&pretty_json).expect("pretty deserialization failed");
    assert_eq!(recents, roundtripped_pretty);

    // 4. Roundtrip via helper methods on RecentDrawingBooks
    let helper_json = recents.to_json().expect("to_json helper failed");
    let roundtripped_helper = RecentDrawingBooks::from_json(&helper_json).expect("from_json helper failed");
    assert_eq!(recents, roundtripped_helper);

    // 5. Adversarial Stress Test: Deserialization of minimal JSON "{}" verifying #[serde(default)]
    let minimal_json = "{}";
    let from_minimal: RecentDrawingBooks =
        serde_json::from_str(minimal_json).expect("deserialization of minimal '{}' must succeed via #[serde(default)]");
    assert!(from_minimal.books.is_empty(), "deserializing '{{}}' must yield empty books");
    let from_minimal_helper = RecentDrawingBooks::from_json(minimal_json).expect("from_json of minimal '{}' must succeed");
    assert!(from_minimal_helper.books.is_empty());

    // 6. Adversarial Stress Test: Malformed JSON syntax with RecentDrawingBooks::from_json -> returns Err gracefully without panic
    let malformed_cases = [
        "",
        "{",
        "{ \"books\": ",
        "{\"books\": [ { \"name\": 123 } ]}",
        "not a json string at all",
        "{\"books\": \"invalid_type_not_a_vec\"}",
        "{\"books\": [ { \"strata_status\": \"InvalidStatusEnumVariant\" } ]}",
        "null",
        "12345",
        "\"a string\"",
    ];
    for bad_json in malformed_cases {
        let res = RecentDrawingBooks::from_json(bad_json);
        assert!(res.is_err(), "malformed or invalid JSON '{bad_json}' must return Err, but got Ok({:?})", res.ok());
    }

    // 7. Adversarial Stress Test: Round-trip with multiple books containing complex Korean strings, Windows path separators (\\ vs /), and all 3 statuses
    let complex_recents = RecentDrawingBooks {
        books: vec![
            super::book::ResolvedDrawingBook {
                name: "제주 용담동 유적 (제1차 발굴·조사) [특수문자: ★/①/㎡]".to_string(),
                survey_dir: PathBuf::from("D:/발굴_2026/용담동_특수/조사_1"),
                drawing_book_dir: PathBuf::from("D:/발굴_2026/용담동_특수/조사_1/도면"),
                strata_status: StrataGpkgStatus::Connected {
                    filename: "용담동_조사_1.gpkg".to_string(),
                    path: PathBuf::from("D:/발굴_2026/용담동_특수/조사_1/용담동_조사_1.gpkg"),
                },
                plan_count: 10,
                section_count: 20,
                finds_count: 30,
                plate_count: 4,
                crs: "5186 중부원점 (EPSG:5186)".to_string(),
                last_opened: "2026-10-14 18:00:00 (KST)".to_string(),
            },
            super::book::ResolvedDrawingBook {
                name: "강릉 초당동 유구\\하부층 2호 주거지".to_string(),
                survey_dir: PathBuf::from("C:\\Survey\\Gangneung\\Chodang"),
                drawing_book_dir: PathBuf::from("C:\\Survey\\Gangneung\\Chodang\\도면"),
                strata_status: StrataGpkgStatus::Disconnected,
                plan_count: 0,
                section_count: 0,
                finds_count: 0,
                plate_count: 0,
                crs: "5187 동부원점".to_string(),
                last_opened: "방금 전".to_string(),
            },
            super::book::ResolvedDrawingBook {
                name: "경주 황남동 120호분 [누락 파일 테스트 / Missing GPKG]".to_string(),
                survey_dir: PathBuf::from("E:\\Excavation\\Gyeongju\\Hwangnam_120"),
                drawing_book_dir: PathBuf::from("E:\\Excavation\\Gyeongju\\Hwangnam_120\\도면"),
                strata_status: StrataGpkgStatus::Missing {
                    filename: "hwangnam_missing.gpkg".to_string(),
                    expected_path: PathBuf::from("E:\\Excavation\\Gyeongju\\Hwangnam_120\\hwangnam_missing.gpkg"),
                },
                plan_count: 5,
                section_count: 7,
                finds_count: 12,
                plate_count: 3,
                crs: "5186 중부원점".to_string(),
                last_opened: "1개월 전".to_string(),
            },
        ],
    };

    let complex_json = complex_recents.to_json().expect("complex serialization failed");
    let roundtripped_complex = RecentDrawingBooks::from_json(&complex_json).expect("complex deserialization failed");
    assert_eq!(complex_recents, roundtripped_complex, "Complex Korean & path roundtrip failed");
    assert_eq!(roundtripped_complex.books.len(), 3);
    assert!(roundtripped_complex.books[0].strata_status.is_connected());
    assert_eq!(roundtripped_complex.books[1].strata_status, StrataGpkgStatus::Disconnected);
    assert!(roundtripped_complex.books[2].strata_status.is_missing());
}

#[test]
fn test_adversarial_drawing_book_resolve_missing_status() {
    use super::book::{ResolvedDrawingBook, StrataGpkgStatus};

    // 1. Non-existent .gpkg file path -> StrataGpkgStatus::Missing. A path in this system's own
    // form: Linux and macOS read `D:\발굴\…` as one file name (the Windows form is checked below
    // on Windows).
    let survey_missing = std::env::temp_dir().join("trowel_no_such_survey").join("하가리");
    let missing_gpkg = survey_missing.join("하가리.gpkg");
    let book_missing = ResolvedDrawingBook::resolve(&missing_gpkg);
    assert!(book_missing.strata_status.is_missing(), "Expected Missing status for non-existent .gpkg path, got {:?}", book_missing.strata_status);
    match &book_missing.strata_status {
        StrataGpkgStatus::Missing { filename, expected_path } => {
            assert_eq!(filename, "하가리.gpkg");
            assert_eq!(expected_path, &missing_gpkg);
        }
        other => panic!("Expected StrataGpkgStatus::Missing, got {:?}", other),
    }
    assert_eq!(book_missing.name, "하가리");
    assert_eq!(book_missing.survey_dir, survey_missing);
    assert_eq!(book_missing.drawing_book_dir, survey_missing.join("도면"));
    #[cfg(windows)]
    {
        use std::path::{Path, PathBuf};
        let missing_gpkg = Path::new("D:\\발굴\\하가리\\하가리.gpkg");
        let book_missing = ResolvedDrawingBook::resolve(missing_gpkg);
        assert!(book_missing.strata_status.is_missing(), "{:?}", book_missing.strata_status);
        assert_eq!(book_missing.name, "하가리");
        assert_eq!(book_missing.survey_dir, PathBuf::from("D:\\발굴\\하가리"));
        assert_eq!(book_missing.drawing_book_dir, PathBuf::from("D:\\발굴\\하가리\\도면"));
    }

    // 2. Existing directory lacking any .gpkg -> StrataGpkgStatus::Disconnected
    let base_dir = std::env::temp_dir()
        .join(format!("trowel_book_missing_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let survey_no_gpkg = base_dir.join("중문동");
    let _ = std::fs::create_dir_all(&survey_no_gpkg);

    let book_disconnected = ResolvedDrawingBook::resolve(&survey_no_gpkg);
    assert_eq!(
        book_disconnected.strata_status,
        StrataGpkgStatus::Disconnected,
        "Directory without .gpkg must return Disconnected, got {:?}",
        book_disconnected.strata_status
    );
    assert!(!book_disconnected.strata_status.is_connected());
    assert!(!book_disconnected.strata_status.is_missing());
    assert_eq!(book_disconnected.name, "중문동");
    assert_eq!(book_disconnected.survey_dir, survey_no_gpkg);
    assert_eq!(book_disconnected.drawing_book_dir, survey_no_gpkg.join("도면"));

    // 3. Subdirectory <survey_dir>/도면 when parent survey lacks .gpkg -> StrataGpkgStatus::Disconnected
    let book_dir_no_gpkg = survey_no_gpkg.join("도면");
    let _ = std::fs::create_dir_all(&book_dir_no_gpkg);
    let book_sub_disconnected = ResolvedDrawingBook::resolve(&book_dir_no_gpkg);
    assert_eq!(book_sub_disconnected.strata_status, StrataGpkgStatus::Disconnected, "도면 folder of survey without .gpkg must return Disconnected");
    assert_eq!(book_sub_disconnected.survey_dir, survey_no_gpkg);
    assert_eq!(book_sub_disconnected.drawing_book_dir, book_dir_no_gpkg);
    assert_eq!(
        book_sub_disconnected.drawing_book_dir,
        book_sub_disconnected.survey_dir.join("도면"),
        "Drawing book must be <survey_dir>/도면 per §14(1)"
    );

    // 4. Target is an existing .gpkg file -> StrataGpkgStatus::Connected
    let survey_with_gpkg = base_dir.join("광령리1");
    let _ = std::fs::create_dir_all(&survey_with_gpkg);
    let existing_gpkg = survey_with_gpkg.join("광령리1.gpkg");
    let _ = std::fs::write(&existing_gpkg, b"sqlite gpkg header mock");
    let book_connected = ResolvedDrawingBook::resolve(&existing_gpkg);
    assert!(book_connected.strata_status.is_connected(), "Existing .gpkg file must return Connected");
    match &book_connected.strata_status {
        StrataGpkgStatus::Connected { filename, path } => {
            assert_eq!(filename, "광령리1.gpkg");
            assert_eq!(path, &existing_gpkg);
        }
        other => panic!("Expected StrataGpkgStatus::Connected, got {:?}", other),
    }
    assert_eq!(book_connected.name, "광령리1");
    assert_eq!(book_connected.survey_dir, survey_with_gpkg);
    assert_eq!(book_connected.drawing_book_dir, survey_with_gpkg.join("도면"), "Drawing book must be <survey_dir>/도면 per §14(1)");

    // 5. Target is a non-gpkg drawing file (.vectorcraft, .drawcraft)
    // 5a. .vectorcraft file inside <survey_dir>/도면 when survey lacks .gpkg -> Disconnected
    let vec_file = book_dir_no_gpkg.join("1호주거지.vectorcraft");
    let _ = std::fs::write(&vec_file, b"{}");
    let book_vec = ResolvedDrawingBook::resolve(&vec_file);
    assert_eq!(book_vec.strata_status, StrataGpkgStatus::Disconnected, "Drawing file without GPKG must return Disconnected");
    assert_eq!(book_vec.survey_dir, survey_no_gpkg);
    assert_eq!(book_vec.drawing_book_dir, book_dir_no_gpkg);
    assert_eq!(book_vec.drawing_book_dir, book_vec.survey_dir.join("도면"), "Drawing book must be <survey_dir>/도면 per §14(1)");

    // 5b. .drawcraft file inside <survey_dir>/도면 when survey lacks .gpkg -> Disconnected
    let draw_file = book_dir_no_gpkg.join("2호주거지.drawcraft");
    let _ = std::fs::write(&draw_file, b"{}");
    let book_draw = ResolvedDrawingBook::resolve(&draw_file);
    assert_eq!(book_draw.strata_status, StrataGpkgStatus::Disconnected, "Drawing file without GPKG must return Disconnected");
    assert_eq!(book_draw.survey_dir, survey_no_gpkg);
    assert_eq!(book_draw.drawing_book_dir, book_dir_no_gpkg);
    assert_eq!(book_draw.drawing_book_dir, book_draw.survey_dir.join("도면"), "Drawing book must be <survey_dir>/도면 per §14(1)");

    // 5c. Standalone drawing file directly under survey directory (not inside 도면) without .gpkg -> Disconnected
    let standalone_vec = survey_no_gpkg.join("단독도면.vectorcraft");
    let _ = std::fs::write(&standalone_vec, b"{}");
    let book_standalone = ResolvedDrawingBook::resolve(&standalone_vec);
    assert_eq!(book_standalone.strata_status, StrataGpkgStatus::Disconnected, "Standalone drawing without GPKG must return Disconnected");
    assert_eq!(book_standalone.survey_dir, survey_no_gpkg);
    assert_eq!(book_standalone.drawing_book_dir, survey_no_gpkg.join("도면"));
    assert_eq!(book_standalone.drawing_book_dir, book_standalone.survey_dir.join("도면"), "Drawing book must be <survey_dir>/도면 per §14(1)");

    // 6. Case sensitivity: Uppercase .GPKG missing file -> StrataGpkgStatus::Missing
    let uppercase_missing = survey_missing.join("하가리.GPKG");
    let book_upper_missing = ResolvedDrawingBook::resolve(&uppercase_missing);
    assert!(book_upper_missing.strata_status.is_missing(), "Uppercase .GPKG path must return Missing status");
    if let StrataGpkgStatus::Missing { filename, expected_path } = &book_upper_missing.strata_status {
        assert_eq!(filename, "하가리.GPKG");
        assert_eq!(expected_path, &uppercase_missing);
    }
    assert_eq!(book_upper_missing.drawing_book_dir, book_upper_missing.survey_dir.join("도면"), "Drawing book must be <survey_dir>/도면 per §14(1)");

    // Clean up
    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_adversarial_template_creation_all_variants() {
    let mut h = Harness::sized(1600.0, 988.0);

    let cases = [
        ("trowel.newPlan", "개별유구 평·단면도", 20.0, 420.0, 297.0, vec!["굴광선 (유구 면)"]),
        ("trowel.newSection", "토층 단면도", 20.0, 420.0, 297.0, vec!["단면선 (잘린 땅)"]),
        ("trowel.newFinds", "유물 실측도", 1.0, 210.0, 297.0, vec!["외형선"]),
        ("trowel.newSitePlan", "유구배치도", 100.0, 420.0, 297.0, vec!["굴광선 (유구 면)"]),
        ("trowel.newPlate", "보고서 도판", 1.0, 210.0, 297.0, vec![]),
    ];

    for (cmd, _expected_name, expected_scale, expected_w_mm, expected_h_mm, expected_layers) in cases {
        h.app.run(cmd, json!({})).unwrap();
        h.settle();

        let doc = h.app.session.active().unwrap_or_else(|| panic!("failed to activate document for {cmd}"));

        // 1. Check units == Millimeters
        assert_eq!(doc.doc.units, vectorcraft_doc::Unit::Millimeters, "{cmd}: units must be Millimeters");

        // 2. Check scale config
        let scale = doc.doc.scale.as_ref().unwrap_or_else(|| panic!("{cmd}: scale must be set"));
        assert_eq!(scale.scale.denom, expected_scale, "{cmd}: scale denominator mismatch");

        // 3. Check canvas size in points (mm * 72 / 25.4)
        assert!(!doc.doc.artboards.is_empty(), "{cmd}: document must have at least one artboard");
        let rect = doc.doc.artboards[0].rect;
        let expected_w_pt = expected_w_mm * 72.0 / 25.4;
        let expected_h_pt = expected_h_mm * 72.0 / 25.4;
        assert!((rect.width() - expected_w_pt).abs() < 1.0, "{cmd}: width mismatch: got {}, expected {}", rect.width(), expected_w_pt);
        assert!((rect.height() - expected_h_pt).abs() < 1.0, "{cmd}: height mismatch: got {}, expected {}", rect.height(), expected_h_pt);

        // 4. Check expected default layers
        let layer_names: Vec<String> = doc.doc.layers.iter().map(|l| l.display_name().to_string()).collect();
        for expected_layer in expected_layers {
            assert!(
                layer_names.iter().any(|n| n.contains(expected_layer)),
                "{cmd}: missing expected layer containing '{expected_layer}', layers present: {:?}",
                layer_names
            );
        }
    }
}

#[test]
fn test_canvas_penbar_adversarial_suite() {
    let mut h = Harness::sized(1600.0, 988.0);

    // 1. Tool visibility check for active drawing tools
    let active_drawing_tools = ["pen", "pencil", "paintbrush", "curvature", "lineSegment"];
    for t in active_drawing_tools {
        h.app.select_tool(t);
        assert!(canvas_penbar::is_visible(&h.app), "Tool '{t}' must show CanvasPenBar");
    }

    let non_drawing_tools = ["selection", "directSelection", "rectangle", "type", "zoom", "hand"];
    for t in non_drawing_tools {
        h.app.select_tool(t);
        assert!(!canvas_penbar::is_visible(&h.app), "Tool '{t}' must NOT show CanvasPenBar");
    }

    // 2. Render frame with CanvasPenBar and check 0.5s double-click configuration
    h.app.select_tool("pen");
    let tokens = Tokens::get(&h.ctx);
    let canvas_rect = Rect::from_min_size(pos2(100.0, 100.0), vec2(1000.0, 700.0));

    let mut out = h.ctx.run_ui(egui::RawInput::default(), |ui| {
        canvas_penbar::show(&mut h.app, ui, canvas_rect, &tokens);
    });
    out.textures_delta.clear();

    let delay = h.ctx.options(|o| o.input_options.max_double_click_delay);
    assert_eq!(delay, 0.5, "CanvasPenBar must configure egui max_double_click_delay to 0.5s");

    // 3. Three buttons only (design 3.0 §5 ⑥, §9): Undo, Finish Line (Enter) and Close; redo and
    // the zoom buttons are the menu's and the canvas corner's. Words only, as in the mockup.
    let shown: Vec<String> = out
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_string()),
            _ => None,
        })
        .collect();
    for want in ["Undo", "Finish Line", "Enter", "Close"] {
        assert!(shown.iter().any(|s| s == want), "{want} in {shown:?}");
    }
    for gone in ["Redo", "Zoom In", "Zoom Out", "Fit"] {
        assert!(!shown.iter().any(|s| s == gone), "{gone} left the pen bar: {shown:?}");
    }
    assert!(!out.shapes.iter().any(|c| matches!(c.shape, egui::Shape::LineSegment { .. })), "no drawn icons");
    // Korean reads 되돌리기 (design 3.0 §9), not the menu's 실행 취소.
    let ko = crate::i18n::Lang::from_code("ko").unwrap();
    assert_eq!(crate::i18n::tr_ctx(ko, "trowel", "Undo"), "되돌리기");
    crate::menus::invoke(&mut h.app, "edit.undo", json!({}));
}

/// The pen bar in the window: centred at the foot of the canvas as in the mockup (clay Finish Line,
/// the others on the tab wash), clear of the zoom buttons; in the smallest window above them, every
/// button whole.
#[test]
fn the_pen_bar_keeps_clear_of_the_zoom_buttons() {
    for (w, height) in [(1600.0, 988.0), (1366.0, 728.0), (800.0, 500.0)] {
        let mut h = Harness::sized(w, height);
        h.app.select_tool("pen");
        h.settle();
        let t = h.tokens();
        let canvas = h.app.canvas_rect.unwrap();
        let bar = h.rect(wid("penbar-shield", ())).unwrap_or_else(|| panic!("{w}: the pen bar"));
        let zoom = h.rect(wid("zoom", "in")).unwrap().union(h.rect(wid("zoom", "fit")).unwrap());
        assert!(!bar.expand(7.5).intersects(zoom), "{w}: {bar:?} 8 points clear of {zoom:?}");
        assert!(canvas.contains_rect(bar), "{w}: {bar:?} in {canvas:?}");
        for k in ["undo", "finish", "close"] {
            let b = h.rect(wid("penbar", k)).unwrap_or_else(|| panic!("{w}: {k}"));
            assert!(bar.contains_rect(b), "{w}: {k} {b:?} in {bar:?}");
        }
        let fill = |h: &Harness, k: &str| h.fill_at(h.rect(wid("penbar", k)).unwrap());
        assert_eq!(fill(&h, "finish"), Some(t.action), "{w}");
        assert_eq!(fill(&h, "undo"), Some(t.button), "{w}");
        assert_eq!(fill(&h, "close"), Some(t.button), "{w}: close whole, not cut off");
        assert_eq!(h.texts_in(bar).iter().any(|s| s == "Enter"), w >= 1366.0, "{w}: the key shown where there is room");
        if w >= 1366.0 {
            assert!((bar.center().x - canvas.center().x).abs() < 0.5, "{w}: centred, {bar:?} in {canvas:?}");
            assert!((canvas.bottom() - 14.0 - bar.bottom()).abs() < 0.5, "{w}: at the foot, beside the zoom buttons");
        } else {
            // 208 points of canvas: above the zoom buttons, the key left out.
            assert!(bar.bottom() <= zoom.top() - 7.5, "{w}: {bar:?} above {zoom:?}");
        }
    }
}

#[test]
fn test_style_dialog_dynamic_diff_and_toggle() {
    use std::collections::HashMap;
    use vectorcraft_doc::style_template::MergeAction;
    use vectorcraft_doc::{LayerColor, Legend, LineType, Node, NodeKind};

    let mut h = Harness::new();

    // 1. Open style dialog
    style_dialog::toggle_dialog(&h.ctx);
    h.settle();

    // 2. Verify window title and content appear
    let win_text = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text.contains("도면 양식 템플릿 차이 분석 및 병합"), "Style dialog must open: {win_text}");
    assert!(win_text.contains("템플릿 차이 분석 결과 (Diff)"), "Diff summary header must appear");
    assert!(win_text.contains("동일: 0"), "Initial identical count must be 0");
    assert!(win_text.contains("추가: 7"), "Initial added count must be 7 for default template");

    // 3. Stress-test Dynamic Diff Computation reflects document state changes across frames:
    // Add a layer in the document matching template's "시굴격자" row (weight: 0.5, Line: Solid, Group: Plan)
    h.app
        .session
        .edit("Add trench grid layer", |doc, _| {
            let mut l = Node::layer(doc.alloc_id(), "시굴격자", LayerColor::Preset(0));
            if let NodeKind::Layer { ref mut legend, .. } = l.kind {
                *legend = Some(Box::new(Legend {
                    weight: Some(0.5),
                    line: LineType::Solid,
                    favourite: false,
                    sample: vectorcraft_doc::legend::LegendSample::Area,
                    strata: Some("trial_trench".to_string()),
                    group: Some(vectorcraft_doc::legend::LegendGroup::Plan),
                }));
            }
            doc.layers.push(std::sync::Arc::new(l));
            Ok(())
        })
        .unwrap();
    h.settle();

    // Verify dynamic diff recomputation in subsequent frame: identical count is now 1, added is 6!
    let win_text2 = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text2.contains("동일: 1"), "Dynamic diff must update across frames to reflect document changes (동일: 1)");
    assert!(win_text2.contains("추가: 6"), "Dynamic diff must update added count to 6");

    // 4. Stress-test Batch Action Buttons updating resolutions map:
    // 4a. "모두 덮어쓰기 (Overwrite)"
    let overwrite_btn = h.texts.iter().find(|p| p.text.contains("모두 덮어쓰기")).expect("overwrite button").rect.center();
    h.click_at(overwrite_btn);
    let res_overwrite: HashMap<String, MergeAction> = h.ctx.data(|d| d.get_temp(wid("style-merge-resolutions-map", ()))).unwrap_or_default();
    assert!(!res_overwrite.is_empty(), "Resolutions map must be populated on batch overwrite");
    assert!(res_overwrite.values().all(|&a| a == MergeAction::Overwrite), "All resolutions must be Overwrite");

    // 4b. "모두 기존 유지 (Keep)"
    let keep_btn = h.texts.iter().find(|p| p.text.contains("모두 기존 유지")).expect("keep button").rect.center();
    h.click_at(keep_btn);
    let res_keep: HashMap<String, MergeAction> = h.ctx.data(|d| d.get_temp(wid("style-merge-resolutions-map", ()))).unwrap_or_default();
    assert!(res_keep.values().all(|&a| a == MergeAction::KeepExisting), "All resolutions must be KeepExisting");

    // 4c. "모두 새 사본 (New Copy)"
    let copy_btn = h.texts.iter().find(|p| p.text.contains("모두 새 사본")).expect("copy button").rect.center();
    h.click_at(copy_btn);
    let res_copy: HashMap<String, MergeAction> = h.ctx.data(|d| d.get_temp(wid("style-merge-resolutions-map", ()))).unwrap_or_default();
    assert!(res_copy.values().all(|&a| a == MergeAction::ImportAsNewCopy), "All resolutions must be ImportAsNewCopy");

    // 4d. "권장값 복원"
    let reset_btn = h.texts.iter().find(|p| p.text.contains("권장값 복원")).expect("reset button").rect.center();
    h.click_at(reset_btn);
    let res_reset: HashMap<String, MergeAction> = h.ctx.data(|d| d.get_temp(wid("style-merge-resolutions-map", ()))).unwrap_or_default();
    assert_eq!(res_reset.get("시굴격자"), Some(&MergeAction::KeepExisting), "Identical item must reset to KeepExisting");

    // 5. Stress-test per-item ComboBox / resolution modifications:
    h.ctx.data_mut(|d| {
        let mut map: HashMap<String, MergeAction> = d.get_temp(wid("style-merge-resolutions-map", ())).unwrap_or_default();
        map.insert("조사구역 경계".to_string(), MergeAction::ImportAsNewCopy);
        d.insert_temp(wid("style-merge-resolutions-map", ()), map);
    });
    h.settle();
    let res_custom: HashMap<String, MergeAction> = h.ctx.data(|d| d.get_temp(wid("style-merge-resolutions-map", ()))).unwrap_or_default();
    assert_eq!(res_custom.get("조사구역 경계"), Some(&MergeAction::ImportAsNewCopy));

    // 6. Stress-test applying resolutions with style.applyTemplate and verifying single-step undo:
    let apply_btn = h.texts.iter().find(|p| p.text.contains("선택한 조치로 양식 적용")).expect("apply button").rect.center();
    h.click_at(apply_btn);

    // Dialog must dismiss automatically upon apply
    let win_text_applied = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(!win_text_applied.contains("도면 양식 템플릿 차이 분석 및 병합"), "Style dialog must close after apply");

    // Verify styles were added into document
    let doc_layers = &h.app.session.active().unwrap().doc.layers;
    assert!(doc_layers.iter().any(|l| l.display_name().contains("조사구역 경계")), "Template layer must be applied to document");

    // Verify single-step undo cleanly removes applied layers
    h.app.run("edit.undo", json!({})).unwrap();
    h.settle();
    let doc_layers_undone = &h.app.session.active().unwrap().doc.layers;
    assert!(
        !doc_layers_undone.iter().any(|l| l.display_name().contains("조사구역 경계")),
        "Undo must roll back template application in a single step"
    );

    // 7. Verify dialog close button dismisses cleanly:
    style_dialog::toggle_dialog(&h.ctx);
    h.settle();
    let win_text_reopened = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text_reopened.contains("도면 양식 템플릿 차이 분석 및 병합"));

    let close_btn = h.texts.iter().find(|p| p.text == "닫기" && p.rect.top() > 500.0).expect("close button").rect.center();
    h.click_at(close_btn);

    let win_closed = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(!win_closed.contains("도면 양식 템플릿 차이 분석 및 병합"), "Style dialog must dismiss cleanly on 닫기 button");
}

#[test]
fn test_gpkg_browser_checkbox_and_toggle() {
    use std::collections::BTreeSet;

    let mut h = Harness::new();

    // 1. Open GPKG browser dialog
    gpkg_browser::toggle_dialog(&h.ctx);
    h.settle();

    // 2. Verify dialog content appears with initial 7 default layers selected
    let win_text = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text.contains("Strata OGC GeoPackage 현장 측량 데이터 연동"), "GPKG browser must open: {win_text}");
    assert!(win_text.contains("광령리 1호 유적 표준 레이어"), "Layers header must appear");
    assert!(win_text.contains("(7/7 선택)"), "Initial selection count must be (7/7 선택)");

    // 3. Stress-test persistent checkbox state across frames, Deselect All and Select All:
    // 3a. Click "전체 해제"
    let deselect_btn = h.texts.iter().find(|p| p.text == "전체 해제").expect("전체 해제 button").rect.center();
    h.click_at(deselect_btn);
    let win_text_desel = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text_desel.contains("(0/7 선택)"), "Count must update to (0/7 선택) after 전체 해제");
    let sel_empty: BTreeSet<String> = h.ctx.data(|d| d.get_temp(gpkg_browser::gpkg_selected_layers_id())).unwrap_or_default();
    assert!(sel_empty.is_empty(), "All layers must be deselected in ctx.data");

    // 3b. Verify persistent state across frames (no reset to default)
    h.settle();
    let win_text_persist = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text_persist.contains("(0/7 선택)"), "Empty selection must persist across subsequent frames");

    // 3c. Click "전체 선택"
    let select_all_btn = h.texts.iter().find(|p| p.text == "전체 선택").expect("전체 선택 button").rect.center();
    h.click_at(select_all_btn);
    let win_text_sel = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text_sel.contains("(7/7 선택)"), "Count must update to (7/7 선택) after 전체 선택");
    let sel_full: BTreeSet<String> = h.ctx.data(|d| d.get_temp(gpkg_browser::gpkg_selected_layers_id())).unwrap_or_default();
    assert_eq!(sel_full.len(), 7, "All 7 layers must be selected in ctx.data");

    // 3d. Subset selection (e.g. exactly 2 layers selected)
    h.ctx.data_mut(|d| {
        let mut subset = BTreeSet::new();
        subset.insert("remains_pit".to_string());
        subset.insert("stone_wall".to_string());
        d.insert_temp(gpkg_browser::gpkg_selected_layers_id(), subset);
    });
    h.settle();
    let win_text_subset = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text_subset.contains("(2/7 선택)"), "Subset count must display (2/7 선택)");

    // 4. Stress-test import button passing 'layers': selected_vec into gpkg.importLayers:
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let temp_gpkg = std::env::temp_dir().join(format!("test_ui_gpkg_{now}.gpkg"));
    vectorcraft_cad::create_7layers_sample_gpkg(&temp_gpkg).unwrap();
    let gpkg_path_str = temp_gpkg.to_str().unwrap().to_string();

    h.ctx.data_mut(|d| {
        d.insert_temp(gpkg_browser::gpkg_path_input_id(), gpkg_path_str.clone());
    });
    h.settle();

    // 4a. Click "조사 데이터 분석 (Inspect)" to analyze GPKG layers
    let inspect_btn = h.texts.iter().find(|p| p.text.contains("조사 데이터 분석")).expect("inspect button").rect.center();
    h.click_at(inspect_btn);

    let win_text_inspected = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text_inspected.contains("tr_remains_polygon"), "Inspected layers must show in browser: {win_text_inspected}");

    // 4b. Select ONLY 'tr_remains_polygon' (subset selection)
    h.ctx.data_mut(|d| {
        let mut selected = BTreeSet::new();
        selected.insert("tr_remains_polygon".to_string());
        d.insert_temp(gpkg_browser::gpkg_selected_layers_id(), selected);
    });
    h.settle();

    let win_text_subset_gpkg = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(win_text_subset_gpkg.contains("(1/7 선택)"), "Must display (1/7 선택)");

    // 4c. Click "선택한 레이어 도면으로 가져오기 (Import)"
    let import_btn = h.texts.iter().find(|p| p.text.contains("선택한 레이어 도면으로 가져오기")).expect("import button").rect.center();
    h.click_at(import_btn);

    // Verify ONLY the selected layer was imported into the document
    let doc = &h.app.session.active().unwrap().doc;
    let imported_layer_names: Vec<String> = doc.layers.iter().map(|l| l.display_name().to_string()).collect();
    assert!(imported_layer_names.iter().any(|n| n.contains("주거지")), "Selected layer (주거지) must be imported");
    assert!(!imported_layer_names.iter().any(|n| n.contains("시굴격자")), "Unselected layer 시굴격자 must NOT be imported");
    assert!(!imported_layer_names.iter().any(|n| n.contains("표고점")), "Unselected layer 표고점 must NOT be imported");
    std::fs::remove_file(&temp_gpkg).ok();

    // 5. Verify dialog close button dismisses cleanly
    let close_btn = h.texts.iter().find(|p| p.text == "닫기" && p.rect.top() > 400.0).expect("close button").rect.center();
    h.click_at(close_btn);

    let win_closed = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(!win_closed.contains("Strata OGC GeoPackage 현장 측량 데이터 연동"), "GPKG browser must close after clicking 닫기");
}

#[test]
fn test_always_visible_palettes_work_without_popups() {
    let mut h = Harness::new();
    let (set, right) = (h.panel(SET), h.panel(RIGHT));

    // 1. The pen: field weights (0.15, 0.4, 0.6) and colour swatches are immediately visible.
    for w in ["0.15", "0.4", "0.6"] {
        let r = h.rect(wid("weight", w)).unwrap_or_else(|| panic!("field weight {w} must be visible from start"));
        assert!(right.contains_rect(r), "{w} inside RIGHT panel");
    }
    h.click(wid("weight", "0.15"));
    assert!((h.app.session.paint.stroke_width - 0.15).abs() < 1e-6);
    h.click(wid("weight", "0.4"));
    assert!((h.app.session.paint.stroke_width - 0.4).abs() < 1e-6);

    // Colour swatches are immediately visible in the pen.
    assert!(right.contains_rect(h.rect(wid("pal-swatch", "red")).unwrap()));
    h.click(wid("pal-swatch", "red"));
    let stroke_col = h.app.session.new_art_stroke().paint;
    assert!(matches!(stroke_col, vectorcraft_color::Paint::Solid { .. }));

    // 2. Left Scrapboard Palette: 12 field rules, scale bars, section markers, north arrow, stratigraphy, and stamps are immediately visible.
    assert!(set.contains_rect(h.rect(wid("scrap-preset", "all12")).unwrap()));
    assert!(set.contains_rect(h.rect(wid("scrap-preset", "p1")).unwrap()));
    assert!(set.contains_rect(h.rect(wid("scrap-scale", 20_u32)).unwrap()));
    assert!(set.contains_rect(h.rect(wid("scrap-sec", "ah")).unwrap()));
    assert!(set.contains_rect(h.rect(wid("scrap-north", "n0")).unwrap()));
    assert!(set.contains_rect(h.rect(wid("scrap-strat", "roman")).unwrap()));
    assert!(set.contains_rect(h.rect(wid("scrap-stamp", "disturb")).unwrap()));

    // Clicking a field rule preset in the Scrapboard Palette adds and selects that rule immediately.
    h.click(wid("scrap-preset", "p1"));
    let cur_name = h.current_layer().and_then(|id| h.app.session.active().unwrap().doc.node(id).map(|n| n.display_name()));
    assert_eq!(cur_name.as_deref(), Some("평면외곽선"));

    // Clicking a scale bar button places the scale bar and updates document scale.
    h.click(wid("scrap-scale", 20_u32));
    assert_eq!(h.app.session.active().unwrap().doc.scale.as_ref().unwrap().scale.denom, 20.0);
}

#[test]
fn test_trowel_ux_dedup_units_and_one_click_workflows() {
    let mut h = Harness::new();
    h.app.run("document.setUnits", json!({"units": "Millimeters"})).unwrap();
    h.app.run("shape.rectangle", json!({"x": 0.0, "y": 0.0, "width": 28.346456692913385, "height": 56.69291338582677})).unwrap();
    h.settle();
    let right = h.panel(RIGHT);
    let props_line = h.line_in(right);
    assert!(props_line.contains("10 × 20 mm"), "Millimeters size should show a single trailing 'mm': {props_line}");
    assert!(!props_line.contains("10 mm × 20 mm"), "Must not duplicate 'mm' before '×': {props_line}");

    // Clicking hatch chip with no selection sets default_hatch on Session
    h.app.run("select.none", json!({})).unwrap();
    h.settle();
    h.click(wid("hatch", "135"));
    assert_eq!(h.app.session.default_hatch, vectorcraft_doc::HatchDirection::Deg135);

    // Clicking a 1-click symbol button while using Pen tool places symbol and switches to Selection tool
    h.app.select_tool("pen");
    assert_eq!(h.app.session.tool_id(), "pen");
    h.click(wid("scrap-sec", "ah"));
    assert_eq!(h.app.session.tool_id(), "selection", "1-click symbol insertion auto-switches to selection tool");
}

/// Design 3.0 D1: every one-click button of the drawing set and the tool column draws something and
/// reports no error (the ribbon's element popups left with D2). (Five of the set's standard lines sent names
/// the engine doesn't know and did nothing; Spot Height sent `x`, `y` without `at`.)
#[test]
fn every_one_click_button_draws_without_error() {
    // Tall enough for the whole drawing set, unscrolled.
    let mut h = Harness::sized(1600.0, 1700.0);
    let check = |h: &mut Harness, what: &str, before: usize, seq: u64| {
        assert_eq!(h.app.ui.status_seq, seq, "{what}: {}", h.app.ui.status);
        assert!(h.node_count() > before, "{what} drew nothing");
    };
    // The drawing set's buttons.
    h.click(wid("scrap-preset-fold", ()));
    let mut set: Vec<(&str, egui::Id)> = vec![];
    for d in [10_u32, 20, 30, 40, 50, 60, 80, 100, 200] {
        set.push(("scale bar", wid("scrap-scale", d)));
    }
    for k in ["ah", "bh", "ch", "av", "dh", "dv"] {
        set.push(("section mark", wid("scrap-sec", k)));
    }
    for k in ["below", "open"] {
        set.push(("level lines", wid("scrap-elev", k)));
    }
    for k in ["n0", "n30", "nm30"] {
        set.push(("north arrow", wid("scrap-north", k)));
    }
    for k in ["roman", "circled", "table7p"] {
        set.push(("layer number", wid("scrap-strat", k)));
    }
    for k in ["disturb", "limit", "grid_lbl"] {
        set.push(("stamp", wid("scrap-stamp", k)));
    }
    // A drawing in the east belt, north first: the scale bars set the scale and keep those.
    h.app.run("doc.setScale", json!({"scale": 20, "epsg": 5187, "order": "northeast"})).unwrap();
    for (what, id) in set {
        let (before, seq) = (h.node_count(), h.app.ui.status_seq);
        h.click(id);
        check(&mut h, &format!("{what} {id:?}"), before, seq);
    }
    let cfg = h.app.session.active().unwrap().doc.scale.clone().unwrap();
    assert_eq!((cfg.scale.denom, cfg.crs.epsg, cfg.crs.order), (200.0, 5187, vectorcraft_doc::scale::CoordOrder::NorthEast));
    // Each standard line by name: added, then drawn with. The twelve buttons name the engine's
    // twelve rows, no more and no fewer.
    let buttons = [
        ("p1", "평면외곽선"),
        ("p2", "내선(바닥)"),
        ("p3", "이중굴광시-가운데선"),
        ("p4", "단면선"),
        ("p5", "석곽-평면외곽선"),
        ("p6", "석곽-내선"),
        ("p7", "석곽-유물 토기외곽·금속"),
        ("p8", "석곽-유물 내선"),
        ("p9", "석곽-유물 깨진면"),
        ("p10", "단면빗금"),
        ("p11", "해발선"),
        ("p12", "해발선 0.5단위"),
    ];
    let mut engine: Vec<&str> = vectorcraft_engine::cmd::legend::field_preset_rows().iter().map(|(n, _)| *n).collect();
    let mut named: Vec<&str> = buttons.iter().map(|b| b.1).collect();
    engine.sort_unstable();
    named.sort_unstable();
    assert_eq!(named, engine);
    for (key, name) in buttons {
        let (before, seq) = (h.node_count(), h.app.ui.status_seq);
        h.click(wid("scrap-preset", key));
        check(&mut h, name, before, seq);
        let cur = h.current_layer().and_then(|id| h.app.session.active().unwrap().doc.node(id).map(|n| n.display_name()));
        assert_eq!(cur.as_deref(), Some(name), "{key}");
    }
    // All twelve again: there already, so nothing to add and nothing wrong.
    let seq = h.app.ui.status_seq;
    h.click(wid("scrap-preset", "all12"));
    assert_eq!(h.app.ui.status_seq, seq, "{}", h.app.ui.status);

    // The datum the set's level-line and section buttons use is typed in its own field, and they
    // draw at it.
    let set = h.panel(SET);
    assert!(h.texts_in(set).iter().any(|s| s == "Datum elevation"), "the drawing set's Datum elevation");
    h.ctx.data_mut(|d| d.insert_temp(super::datum_elevation_id(), 95.5));
    h.settle();
    assert!(h.texts_in(set).iter().any(|s| s == "해발 95.5m"), "the datum buttons show it: {:?}", h.texts_in(set));
    let picked = |h: &Harness| {
        let st = h.app.session.active().unwrap();
        st.selection.objects.first().and_then(|id| st.doc.node(*id)).map(|n| n.display_name())
    };
    for k in ["dh", "dv"] {
        h.click(wid("scrap-sec", k));
        assert!(picked(&h).is_some_and(|n| n.contains("95.5m")), "{k}: {:?}", picked(&h));
    }
    h.click(wid("scrap-elev", "below"));
    let mut datums = vec![];
    h.app.session.active().unwrap().doc.walk(|n| datums.extend(n.elevation.as_ref().map(|e| e.datum_m)));
    assert!(datums.contains(&95.5), "level lines at the datum: {datums:?}");

    // The tool column's Spot Height and Slope Hachure are tools now (design 3.0 D4): a click on the
    // canvas, its height box starting at the elevation the drawing set's field set; a click on a
    // line.
    h.click(wid("tool", "spotHeight"));
    h.click_at(h.on_screen(200.0, 60.0));
    assert_eq!(h.field_text(wid("ask-field", ())), Some("95.5".to_string()), "the last height");
    let (before, seq) = (h.node_count(), h.app.ui.status_seq);
    h.key(egui::Key::Enter);
    check(&mut h, "spot height", before, seq);
    let st = h.app.session.active().unwrap();
    let spot = st.selection.objects.first().and_then(|id| st.doc.node(*id)).map(|n| n.display_name());
    assert_eq!(spot.as_deref(), Some("표고 95.5m"));
    let top = h.app.run("shape.line", json!({ "x1": 50.0, "y1": 50.0, "x2": 250.0, "y2": 80.0 })).unwrap();
    h.app.run("select.none", json!({})).unwrap();
    h.app.select_tool("slopeHachure");
    h.settle();
    let (before, seq) = (h.node_count(), h.app.ui.status_seq);
    h.click_at(h.on_screen(150.0, 65.0));
    check(&mut h, "slope hachure", before, seq);
    // Along the clicked line (50–250 across), not some other one.
    let st = h.app.session.active().unwrap();
    let line = st.doc.node(vectorcraft_doc::NodeId(top["id"].as_u64().unwrap())).and_then(|n| n.geometric_bounds()).unwrap();
    let mut hachures = None;
    st.doc.walk(|n| {
        if n.name.as_deref() == Some("비탈면 빗금") {
            hachures = n.geometric_bounds();
        }
    });
    let b = hachures.expect("the hachures");
    assert!(b.x0 >= line.x0 - 1.0 && b.x1 <= line.x1 + 1.0, "along the line {line:?}: {b:?}");
}

/// The names the drawing's objects have, in document order.
fn object_names(h: &Harness) -> Vec<String> {
    let mut v = vec![];
    h.app.session.active().unwrap().doc.walk(|n| v.extend(n.name.clone()));
    v
}

/// Keys 1–4 pick the archaeology tools in Trowel's window and the classic window's digits pick
/// none (design 3.0 §5 ④, D4 criterion 6); the pen bar shows for the Section Line and the Strata
/// Line as for the Pen.
#[test]
fn digits_pick_the_archaeology_tools_in_trowel_only() {
    let mut h = Harness::new();
    let digit = |h: &mut Harness, d: &str, key: egui::Key| {
        let modifiers = egui::Modifiers::default();
        h.frame(vec![Event::Key { key, physical_key: Some(key), pressed: true, repeat: false, modifiers }, Event::Text(d.into())]);
        h.frame(vec![Event::Key { key, physical_key: Some(key), pressed: false, repeat: false, modifiers }]);
        h.settle();
    };
    use egui::Key::{Num1, Num2, Num3, Num4};
    for (d, key, tool, penbar) in
        [("1", Num1, "sectionCut", true), ("2", Num2, "levelLines", false), ("3", Num3, "spotHeight", false), ("4", Num4, "strataLine", true)]
    {
        digit(&mut h, d, key);
        assert_eq!(h.app.session.tool_id(), tool, "{d}");
        assert_eq!(canvas_penbar::is_visible(&h.app), penbar, "{tool}");
        // The status bar says how to use it, not where to search commands.
        let hint: String = crate::chrome::current_hint(&h.app).into_iter().map(|(s, _)| s).collect();
        assert!(hint.starts_with("Click ") && !hint.contains("search every command"), "{tool}: {hint}");
    }
    // The classic window: a digit is no tool's key.
    h.app.ui.trowel = false;
    h.app.select_tool("selection");
    h.settle();
    digit(&mut h, "1", Num1);
    assert_eq!(h.app.session.tool_id(), "selection");
}

/// The keyboard reaches the Strata Line while it is drawn (review J024 1): Backspace takes the last
/// point back, Esc drops the line, and between clicks the shortcuts work (Z, the Zoom tool).
#[test]
fn the_keyboard_reaches_the_strata_line_while_it_is_drawn() {
    let mut h = Harness::new();
    h.click(wid("tool", "strataLine"));
    for (x, y) in [(20.0, 215.0), (200.0, 240.0), (380.0, 215.0)] {
        h.click_at(h.on_screen(x, y));
    }
    h.key(egui::Key::Backspace);
    h.key(egui::Key::Enter);
    let st = h.app.session.active().unwrap();
    let mut anchors = None;
    st.doc.walk(|n| {
        if n.name.as_deref() == Some("토층선 Ⅰ")
            && let vectorcraft_doc::NodeKind::Path { path, .. } = &n.kind
        {
            anchors = path.subpaths.first().map(|sp| sp.anchors.len());
        }
    });
    assert_eq!(anchors, Some(2), "the third point taken back");
    // Ctrl+Z while a line is drawn takes its last point back too, not the line before (review
    // J024 N2): Enter then makes 「토층선 Ⅱ」 of two points, and 「토층선 Ⅰ」 is still there.
    for (x, y) in [(20.0, 240.0), (200.0, 250.0), (380.0, 240.0)] {
        h.click_at(h.on_screen(x, y));
    }
    let undo =
        |pressed| Event::Key { key: egui::Key::Z, physical_key: Some(egui::Key::Z), pressed, repeat: false, modifiers: egui::Modifiers::COMMAND };
    h.frame(vec![undo(true)]);
    h.frame(vec![undo(false)]);
    h.settle();
    assert!(object_names(&h).iter().any(|n| n == "토층선 Ⅰ"), "{:?}", object_names(&h));
    h.key(egui::Key::Enter);
    let points = |h: &Harness, name: &str| {
        let mut anchors = None;
        h.app.session.active().unwrap().doc.walk(|n| {
            if n.name.as_deref() == Some(name)
                && let vectorcraft_doc::NodeKind::Path { path, .. } = &n.kind
            {
                anchors = path.subpaths.first().map(|sp| sp.anchors.len());
            }
        });
        anchors
    };
    assert_eq!((points(&h, "토층선 Ⅰ"), points(&h, "토층선 Ⅱ")), (Some(2), Some(2)));
    h.app.run("edit.undo", json!({})).unwrap();
    // Esc drops a line being drawn: nothing made.
    for (x, y) in [(20.0, 260.0), (200.0, 270.0)] {
        h.click_at(h.on_screen(x, y));
    }
    let before = h.node_count();
    h.key(egui::Key::Escape);
    h.key(egui::Key::Enter);
    assert_eq!(h.node_count(), before);
    // A shortcut between clicks: the line so far is kept, as switching tools does.
    for (x, y) in [(20.0, 300.0), (200.0, 310.0)] {
        h.click_at(h.on_screen(x, y));
    }
    h.frame(vec![
        Event::Key { key: egui::Key::Z, physical_key: Some(egui::Key::Z), pressed: true, repeat: false, modifiers: Default::default() },
        Event::Text("z".into()),
    ]);
    h.frame(vec![Event::Key { key: egui::Key::Z, physical_key: Some(egui::Key::Z), pressed: false, repeat: false, modifiers: Default::default() }]);
    h.settle();
    assert_eq!(h.app.session.tool_id(), "zoom");
    assert!(object_names(&h).iter().any(|n| n == "토층선 Ⅱ"), "{:?}", object_names(&h));
}

/// The pen bar has what each tool can do (review J024 UI 3 and 15): a strata line nothing to
/// close, a section line nothing to finish either, its Cancel and Undo dropping the first point.
#[test]
fn the_pen_bar_has_what_each_tool_can_do() {
    let mut h = Harness::new();
    let has = |h: &Harness, k: &str| h.rect(wid("penbar", k)).is_some();
    for (tool, keys) in [("pen", ["undo", "finish", "close"].as_slice()), ("strataLine", &["undo", "finish"]), ("sectionCut", &["undo", "cancel"])] {
        h.app.select_tool(tool);
        h.settle();
        let shown: Vec<&str> = ["undo", "finish", "close", "cancel"].into_iter().filter(|k| has(&h, k)).collect();
        assert_eq!(shown, keys, "{tool}");
    }
    let waiting = |h: &mut Harness| {
        let view = h.app.view_info();
        h.app.session.tool_claims_key(vectorcraft_tools::ToolKey::Escape, view)
    };
    for k in ["cancel", "undo"] {
        h.click_at(h.on_screen(60.0, 120.0));
        assert!(waiting(&mut h), "{k}: the first point");
        h.click(wid("penbar", k));
        assert!(!waiting(&mut h), "{k} drops it");
    }
    let before = h.node_count();
    h.click_at(h.on_screen(60.0, 120.0));
    h.click_at(h.on_screen(300.0, 120.0));
    assert!(object_names(&h).iter().any(|n| n == "단면 기호 A–A′"), "{:?}", object_names(&h));
    assert!(h.node_count() > before);
}

/// A soil layer's description line takes typing at a double-click with the Selection tool, not
/// after isolating its block first (review J024 UI 4, design 3.0 §8 step 8).
#[test]
fn a_description_line_takes_typing_at_a_double_click() {
    let mut h = Harness::new();
    h.app.run("stratigraphy.boundary", json!({ "points": [[40, 100], [200, 120], [360, 100]] })).unwrap();
    h.app.run("select.none", json!({})).unwrap();
    h.app.select_tool("selection");
    let st = h.app.session.active().unwrap();
    let mut line = None;
    st.doc.walk(|n| {
        if n.name.as_deref() == Some("토층설명 Ⅰ") {
            line = n.geometric_bounds().map(|b| (n.id, b));
        }
    });
    let (id, b) = line.expect("the description line");
    let at = h.on_screen(b.x1 - 1.0, b.center().y);
    h.frame(vec![Event::PointerMoved(at)]);
    for _ in 0..2 {
        for pressed in [true, false] {
            h.frame(vec![Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Default::default() }]);
        }
    }
    h.settle();
    assert_eq!(h.app.session.tool_id(), "type");
    assert_eq!(h.app.session.tool_options()["editing"].as_u64(), Some(id.0), "typing in the line");
    assert!(h.app.session.active().unwrap().isolation.is_none(), "not isolated");
    h.frame(vec![Event::Text("회황갈색 사질점토".into())]);
    h.key(egui::Key::Escape);
    let st = h.app.session.active().unwrap();
    let Some(vectorcraft_doc::NodeKind::Text(t)) = st.doc.node(id).map(|n| &n.kind) else { panic!("the line") };
    assert!(t.plain_text().contains("회황갈색 사질점토"), "{}", t.plain_text());
}

/// What to click next belongs to the tool it was said for: picking another tool takes
/// 「Click the ground line」 away, and so does finding the line (review J024 UI 7).
#[test]
fn the_level_lines_hint_goes_with_the_tool() {
    let mut h = Harness::new();
    let anchors = json!([{"x": 20, "y": 200}, {"x": 200, "y": 260}, {"x": 380, "y": 200}]);
    h.app.run("path.create", json!({ "anchors": anchors })).unwrap();
    h.app.run("select.none", json!({})).unwrap();
    h.click(wid("tool", "levelLines"));
    h.click_at(h.on_screen(200.0, 100.0));
    // The message, not the tool's hint (「Click the ground line, then type the height there」).
    let said = |h: &Harness| h.status_line().contains("Click the ground line") && !h.status_line().contains("then type");
    assert!(said(&h), "{}", h.status_line());
    h.click(wid("tool", "strataLine"));
    assert!(!said(&h) && h.status_line().contains("to add a point"), "the Strata Line's hint: {}", h.status_line());
    h.click(wid("tool", "levelLines"));
    h.click_at(h.on_screen(200.0, 100.0));
    assert!(said(&h), "{}", h.status_line());
    h.click_at(h.on_screen(110.0, 230.0));
    assert!(h.app.trowel_ask.is_some(), "the height box");
    assert!(!said(&h) && h.status_line().contains("then type the height there"), "the hint back: {}", h.status_line());
}

/// A plan with map coordinates (a TM origin) is told at the click, before a height is typed
/// (review J024 10): no height box, the refusal in the status bar.
#[test]
fn level_lines_on_a_map_drawing_are_refused_at_the_click() {
    let mut h = Harness::new();
    let anchors = json!([{"x": 20, "y": 200}, {"x": 200, "y": 260}, {"x": 380, "y": 200}]);
    h.app.run("path.create", json!({ "anchors": anchors })).unwrap();
    h.app.run("select.none", json!({})).unwrap();
    h.app.run("doc.setOrigin", json!({"canvas": [0, 0], "world": [200000.0, 450000.0]})).unwrap();
    h.click(wid("tool", "levelLines"));
    let before = h.node_count();
    h.click_at(h.on_screen(110.0, 230.0));
    assert!(h.app.trowel_ask.is_none(), "no height box");
    assert_eq!(h.app.ui.status, "This drawing has map coordinates: draw level lines in a section drawing");
    assert_eq!(h.node_count(), before);
}

/// Design 3.0 §8 steps 5, 7 and 8 with the tool column alone (D4 criterion 7): A–A′ across the
/// plan with the Section Line, level lines from a click on the ground line at 32.5 m with their
/// datum tick, three layer boundaries ended with Enter, a double-click and the pen bar, with their
/// numbers and description lines, and a spot height. Each is one undo step.
#[test]
fn the_archaeology_tools_draw_the_eighth_chapter() {
    let mut h = Harness::new();
    // 5: two clicks, the preview with the letter between them.
    h.click(wid("tool", "sectionCut"));
    h.click_at(h.on_screen(40.0, 150.0));
    h.frame(vec![Event::PointerMoved(h.on_screen(360.0, 150.0))]);
    h.settle();
    assert!(h.texts.iter().any(|p| p.text == "A–A′"), "the preview's letter");
    h.click_at(h.on_screen(360.0, 150.0));
    assert!(object_names(&h).iter().any(|n| n == "단면 기호 A–A′"), "{:?}", object_names(&h));

    // 7: the ground line (drawn with the legend's section line row), then Level Lines on it.
    let anchors = json!([{"x": 20, "y": 200}, {"x": 120, "y": 200}, {"x": 200, "y": 260}, {"x": 280, "y": 200}, {"x": 380, "y": 200}]);
    h.app.run("path.create", json!({ "anchors": anchors })).unwrap();
    h.app.run("select.none", json!({})).unwrap();
    h.click(wid("tool", "levelLines"));
    h.click_at(h.on_screen(200.0, 100.0));
    assert_eq!((h.app.ui.status.as_str(), h.app.trowel_ask.is_none()), ("Click the ground line", true), "a click on nothing");
    h.click_at(h.on_screen(160.0, 230.0));
    let (field, ok) = (h.rect(wid("ask-field", ())).expect("the height box"), h.rect(wid("ask", "ok")).expect("넣기"));
    assert!(ok.right() - field.left() <= 212.0, "the box keeps to its own width: {field:?} {ok:?}");
    assert!(h.texts.iter().any(|p| p.text == "Height here"), "its title");
    let before = h.node_count();
    h.type_into(wid("ask-field", ()), "32.5");
    assert!(h.app.trowel_ask.is_none(), "{}", h.app.ui.status);
    assert!(object_names(&h).iter().any(|n| n == "단면 해발 눈금 (32.5m)"), "the datum tick: {:?}", object_names(&h));
    let mut levels = vec![];
    h.app.session.active().unwrap().doc.walk(|n| levels.extend(n.elevation.as_ref().map(|e| e.datum_m)));
    assert_eq!(levels.len(), 1, "level lines");
    h.app.run("edit.undo", json!({})).unwrap();
    assert_eq!(h.node_count(), before, "one undo step");
    h.app.run("edit.redo", json!({})).unwrap();

    // 8: three boundaries.
    h.click(wid("tool", "strataLine"));
    for (x, y) in [(20.0, 215.0), (200.0, 240.0), (380.0, 215.0)] {
        h.click_at(h.on_screen(x, y));
    }
    h.key(egui::Key::Enter);
    h.click_at(h.on_screen(60.0, 230.0));
    let end = h.on_screen(340.0, 230.0);
    h.click_at(end);
    h.click_at(end);
    h.click_at(h.on_screen(100.0, 245.0));
    h.click_at(h.on_screen(300.0, 245.0));
    h.click(wid("penbar", "finish"));
    let names = object_names(&h);
    for n in ["토층선 Ⅰ", "토층선 Ⅱ", "토층선 Ⅲ", "층위 번호 Ⅰ", "층위 번호 Ⅲ", "토층설명 7p"] {
        assert!(names.iter().any(|m| m == n), "{n}: {names:?}");
    }
    let st = h.app.session.active().unwrap();
    let mut notes = vec![];
    st.doc.walk(|n| {
        if let (Some(name), vectorcraft_doc::NodeKind::Text(t)) = (&n.name, &n.kind)
            && name.starts_with("토층설명 ")
        {
            notes.push(t.plain_text());
        }
    });
    assert_eq!(notes, ["Ⅰ층 : ", "Ⅱ층 : ", "Ⅲ층 : "]);
    assert_eq!(st.doc.stratigraphy_tables.first().map(|t| t.layers.len()), Some(3));

    // 3: a spot height, its box starting at the last height.
    h.click(wid("tool", "spotHeight"));
    h.click_at(h.on_screen(300.0, 120.0));
    assert_eq!(h.field_text(wid("ask-field", ())), Some("32.5".to_string()));
    h.key(egui::Key::Enter);
    let mut spot = None;
    h.app.session.active().unwrap().doc.walk(|n| {
        if let vectorcraft_doc::NodeKind::Text(t) = &n.kind
            && t.plain_text() == "32.5m"
        {
            spot = Some(n.id);
        }
    });
    assert!(spot.is_some(), "▽ 32.5m: {}", h.app.ui.status);
    // Undo: the spot height, then the third boundary with its number, line and row.
    h.app.run("edit.undo", json!({})).unwrap();
    h.app.run("edit.undo", json!({})).unwrap();
    let names = object_names(&h);
    assert!(names.iter().any(|m| m == "토층선 Ⅱ") && !names.iter().any(|m| m.ends_with('Ⅲ')), "{names:?}");
    assert_eq!(h.app.session.active().unwrap().doc.stratigraphy_tables.first().map(|t| t.layers.len()), Some(2));
}
