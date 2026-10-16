//! Trowel's archaeology tools (design 3.0 §5 ④, D4): the four under 「고고학」 in Trowel's tool
//! column and the Section Line slot's other tool. They are not in the classic toolbar
//! ([`crate::catalog::TROWEL_TOOLS`]).
//!
//! - Section Line (`sectionCut`, key 1): two clicks, or a drag, put the A–A′ marks at the two
//!   points (`trowel.insertSectionMarker {kind: cut, p0, p1}`) with the next letter the drawing
//!   doesn't use yet. Shift: 45° steps; Esc (or Backspace) stops.
//! - Slope Hachure (`slopeHachure`, in the Section Line slot): a click on a line hachures along it
//!   (`trowel.slopeHachures {topId}`).
//! - Level Lines (`levelLines`, key 2): the ground line under the pointer is marked; a click on it
//!   asks for the height there ([`ASK_LEVEL`]: the UI's height box, which runs `trowel.levelLines`).
//! - Spot Height (`spotHeight`, key 3): a click asks for the point's height ([`ASK_SPOT`]: the box
//!   runs `trowel.spotElevation`).
//! - Strata Line (`strataLine`, key 4): each click adds a point and a drag adds freehand ones;
//!   Enter, a double-click (a click on the last point) or the pen bar's Finish Line end it
//!   (`stratigraphy.boundary`: the line, its layer number and a description line), Backspace takes
//!   the last point back, Esc stops.
//!
//! A click that finds no line asks the UI to say what to click ([`HINT`]); the marks these tools
//! put in (level lines, A–A′ …) are not lines to click ([`vectorcraft_doc::section_marks::is_mark`]).
//! Between clicks the tools aren't [`Tool::busy`] (only while the button is down), so the
//! keyboard's Backspace, Enter and Esc reach them ([`Tool::claims_key`]) and the shortcuts work.

use serde_json::json;
use vectorcraft_doc::{NodeId, NodeKind};
use vectorcraft_geom::{BezPath, Point, constrain_angle_from};

use crate::draw2::FEEDBACK;
use crate::{Action, Cursor, Mods, Overlay, PointerEvent, PointerKind, Tool, ToolContext, ToolKey};

/// The dialog a Level Lines click opens: `{profileId, at}`.
pub const ASK_LEVEL: &str = "trowel.levelLines";
/// The dialog a Spot Height click opens: `{at}`.
pub const ASK_SPOT: &str = "trowel.spotHeight";
/// A click that found nothing to work on: `{tool}`, the UI says what to click.
pub const HINT: &str = "trowel.hint";

/// Create an archaeology tool by id (None = not one of them).
pub fn create(id: &str) -> Option<Box<dyn Tool>> {
    Some(match id {
        "sectionCut" => Box::new(SectionCutTool::default()),
        "slopeHachure" => Box::new(LineClickTool { id: "slopeHachure", hover: None }),
        "levelLines" => Box::new(LineClickTool { id: "levelLines", hover: None }),
        "spotHeight" => Box::new(SpotHeightTool::default()),
        "strataLine" => Box::new(StrataLineTool::default()),
        _ => return None,
    })
}

/// How near (screen pixels) a click must be to a line to pick it, at least.
const PICK_PX: f64 = 5.0;
/// The two points of a section mark must be this far apart (screen pixels), and at least
/// [`MIN_SPAN_PT`] on paper (as `trowel.insertSectionMarker` takes them).
const MIN_SPAN_PX: f64 = 4.0;
const MIN_SPAN_PT: f64 = 1.0;
/// Freehand points of a strata line are at least this far apart (screen pixels).
const STEP_PX: f64 = 3.0;
/// A press this near the strata line's last point (screen pixels, as near as a double-click's two
/// presses) ends the line once released.
const FINISH_PX: f64 = 6.0;
/// The most points a strata line takes (`stratigraphy.boundary`'s cap).
const MAX_POINTS: usize = 100_000;

fn xy(p: Point) -> [f64; 2] {
    [p.x, p.y]
}

/// The path whose line is under `p` (not one whose inside is, nor a mark the tools put in): its
/// id.
fn path_at(cx: &ToolContext, p: Point) -> Option<NodeId> {
    let mut opt = cx.hit_options();
    opt.path_only = true;
    opt.tol = opt.tol.max(cx.tol(PICK_PX));
    let mark = |id: NodeId| cx.doc.node(id).is_some_and(vectorcraft_doc::section_marks::is_mark);
    let hit = vectorcraft_doc::hit::hit_test_skipping(cx.doc, p, opt, &mark)?;
    matches!(cx.doc.node(hit.leaf)?.kind, NodeKind::Path { .. }).then_some(hit.leaf)
}

/// Path `id`'s line, marked as the one a click picks.
fn marked(cx: &ToolContext, id: Option<NodeId>) -> Vec<Overlay> {
    let Some(NodeKind::Path { path, .. }) = id.and_then(|id| cx.doc.node(id)).map(|n| &n.kind) else { return vec![] };
    vec![Overlay::Path { path: path.to_bezpath(), color: FEEDBACK, width: 2.0, dashed: false }]
}

/// Section Line: two points, two clicks or one drag.
#[derive(Default)]
pub struct SectionCutTool {
    /// The first point, once placed.
    start: Option<Point>,
    /// The second point as it stands (the pointer).
    end: Point,
    /// The button is down on the first point: a drag places the second.
    pressed: bool,
    dragged: bool,
    /// The letter the marks will take, found once the first point is down.
    letter: String,
}

impl SectionCutTool {
    /// The second point for the pointer at `p`: Shift snaps the line to 45° steps.
    fn end_point(cx: &ToolContext, start: Point, p: Point, m: Mods) -> Point {
        if m.shift { start + constrain_angle_from(p - start, 45.0, cx.constrain_angle) } else { p }
    }
    fn far_enough(&self, cx: &ToolContext) -> bool {
        self.start.is_some_and(|a| a.distance(self.end) >= cx.tol(MIN_SPAN_PX).max(MIN_SPAN_PT))
    }
    fn reset(&mut self) {
        self.start = None;
        self.pressed = false;
        self.dragged = false;
    }
    /// The marks at the two points.
    fn finish(&mut self) -> Vec<Action> {
        let Some(a) = self.start else { return vec![] };
        let b = self.end;
        self.reset();
        vec![Action::Exec("trowel.insertSectionMarker".into(), json!({"kind": "cut", "p0": xy(a), "p1": xy(b)}))]
    }
}

impl Tool for SectionCutTool {
    fn id(&self) -> &'static str {
        "sectionCut"
    }
    fn busy(&self) -> bool {
        self.pressed
    }
    fn pointer(&mut self, cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        match (ev.kind, self.start) {
            // The second click.
            (PointerKind::Down, Some(a)) if !self.pressed => {
                self.end = Self::end_point(cx, a, ev.pos, ev.mods);
                if self.far_enough(cx) { self.finish() } else { vec![] }
            }
            (PointerKind::Down, _) => {
                self.start = Some(ev.pos);
                self.end = ev.pos;
                self.pressed = true;
                self.dragged = false;
                self.letter = vectorcraft_doc::section_marks::next_letter(cx.doc);
                vec![]
            }
            (PointerKind::Drag | PointerKind::Up, Some(a)) if self.pressed => {
                self.end = Self::end_point(cx, a, ev.pos, ev.mods);
                self.dragged |= self.far_enough(cx);
                if ev.kind == PointerKind::Up {
                    self.pressed = false;
                    // A drag places both points; a click only the first.
                    if self.dragged && self.far_enough(cx) {
                        return self.finish();
                    }
                }
                vec![]
            }
            (PointerKind::Move, Some(a)) => {
                self.end = Self::end_point(cx, a, ev.pos, ev.mods);
                vec![]
            }
            _ => vec![],
        }
    }
    /// Esc, or Backspace taking the first point back (the pen bar's Undo), drops the first point.
    fn key(&mut self, _cx: &ToolContext, key: ToolKey, _m: Mods) -> Vec<Action> {
        if matches!(key, ToolKey::Escape | ToolKey::Backspace | ToolKey::Delete) {
            self.reset();
        }
        vec![]
    }
    fn claims_key(&self, _cx: &ToolContext, key: ToolKey) -> bool {
        self.start.is_some() && matches!(key, ToolKey::Escape | ToolKey::Backspace | ToolKey::Delete)
    }
    fn deactivate(&mut self, _cx: &ToolContext) -> Vec<Action> {
        self.reset();
        vec![]
    }
    fn overlays(&self, cx: &ToolContext) -> Vec<Overlay> {
        let Some(a) = self.start else { return vec![] };
        let mut o = vec![Overlay::Anchor { p: a, color: FEEDBACK, filled: true, size: 5.0 }];
        if self.far_enough(cx) {
            let (b, letter) = (self.end, &self.letter);
            o.push(Overlay::Line { a, b, color: FEEDBACK, dashed: true });
            o.push(Overlay::Anchor { p: b, color: FEEDBACK, filled: false, size: 5.0 });
            o.push(Overlay::Label { p: b, text: format!("{letter}–{letter}′"), color: FEEDBACK });
        }
        o
    }
    fn cursor(&self, _cx: &ToolContext, _p: Point, _m: Mods) -> Cursor {
        Cursor::Crosshair
    }
}

/// Level Lines and Slope Hachure: a click on a line.
pub struct LineClickTool {
    id: &'static str,
    /// The line under the pointer.
    hover: Option<NodeId>,
}

impl Tool for LineClickTool {
    fn id(&self) -> &'static str {
        self.id
    }
    fn pointer(&mut self, cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        match ev.kind {
            PointerKind::Move => {
                self.hover = path_at(cx, ev.pos);
                vec![]
            }
            PointerKind::Down => {
                let line = path_at(cx, ev.pos);
                self.hover = line;
                match (self.id, line) {
                    (_, None) => vec![Action::Dialog(HINT.into(), json!({"tool": self.id}))],
                    ("levelLines", Some(id)) => vec![Action::Dialog(ASK_LEVEL.into(), json!({"profileId": id.0, "at": xy(ev.pos)}))],
                    (_, Some(id)) => vec![Action::Exec("trowel.slopeHachures".into(), json!({"topId": id.0}))],
                }
            }
            _ => vec![],
        }
    }
    fn overlays(&self, cx: &ToolContext) -> Vec<Overlay> {
        marked(cx, self.hover)
    }
    fn cursor(&self, _cx: &ToolContext, _p: Point, _m: Mods) -> Cursor {
        Cursor::Crosshair
    }
}

/// Spot Height: a click on the point.
#[derive(Default)]
pub struct SpotHeightTool {
    hover: Option<Point>,
}

/// The ▽ `trowel.spotElevation {marker: triangleDown}` draws at `p`: its tip there, 7 pt wide.
fn down_triangle(p: Point) -> BezPath {
    let h = 7.0 * 3f64.sqrt() / 2.0;
    let mut bp = BezPath::new();
    bp.move_to(p);
    bp.line_to(Point::new(p.x - 3.5, p.y - h));
    bp.line_to(Point::new(p.x + 3.5, p.y - h));
    bp.close_path();
    bp
}

impl Tool for SpotHeightTool {
    fn id(&self) -> &'static str {
        "spotHeight"
    }
    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        match ev.kind {
            PointerKind::Move => {
                self.hover = Some(ev.pos);
                vec![]
            }
            PointerKind::Down => vec![Action::Dialog(ASK_SPOT.into(), json!({"at": xy(ev.pos)}))],
            _ => vec![],
        }
    }
    fn overlays(&self, _cx: &ToolContext) -> Vec<Overlay> {
        self.hover.map(|p| Overlay::Path { path: down_triangle(p), color: FEEDBACK, width: 1.0, dashed: false }).into_iter().collect()
    }
    fn cursor(&self, _cx: &ToolContext, _p: Point, _m: Mods) -> Cursor {
        Cursor::Crosshair
    }
}

/// Strata Line: a soil layer boundary, click by click or freehand.
#[derive(Default)]
pub struct StrataLineTool {
    points: Vec<Point>,
    pressed: bool,
    /// The press landed on the last point and added none (a double-click's second press, or a
    /// click on the last point): released where it is, it ends the line; dragged, it goes on
    /// freehand from there. The window's own double-clicks aren't used: two quick clicks at two
    /// points are one too, and a quick third click on the last point is a triple click.
    on_last: bool,
    /// The pointer, where the next segment goes.
    hover: Option<Point>,
}

impl StrataLineTool {
    /// Add `p` unless it is on the last point; whether it was.
    fn add(&mut self, cx: &ToolContext, p: Point) -> bool {
        let far = self.points.last().is_none_or(|l| l.distance(p) >= cx.tol(STEP_PX));
        if far && self.points.len() < MAX_POINTS {
            self.points.push(p);
        }
        far
    }
    /// The boundary through the points, two or more; fewer leave nothing.
    fn finish(&mut self) -> Vec<Action> {
        let points = std::mem::take(&mut self.points);
        self.pressed = false;
        self.on_last = false;
        if points.len() < 2 {
            return vec![];
        }
        let points: Vec<[f64; 2]> = points.into_iter().map(xy).collect();
        vec![Action::Exec("stratigraphy.boundary".into(), json!({ "points": points }))]
    }
}

impl Tool for StrataLineTool {
    fn id(&self) -> &'static str {
        "strataLine"
    }
    fn busy(&self) -> bool {
        self.pressed
    }
    fn pointer(&mut self, cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        self.hover = Some(ev.pos);
        match ev.kind {
            PointerKind::Down => {
                self.pressed = true;
                self.on_last = self.points.last().is_some_and(|l| l.distance(ev.pos) < cx.tol(FINISH_PX));
                if !self.on_last {
                    self.add(cx, ev.pos);
                }
                vec![]
            }
            PointerKind::Drag if self.pressed => {
                if self.add(cx, ev.pos) {
                    self.on_last = false;
                }
                vec![]
            }
            PointerKind::Up if self.pressed => {
                self.pressed = false;
                if self.on_last { self.finish() } else { vec![] }
            }
            _ => vec![],
        }
    }
    fn key(&mut self, _cx: &ToolContext, key: ToolKey, _m: Mods) -> Vec<Action> {
        match key {
            ToolKey::Enter => self.finish(),
            ToolKey::Backspace | ToolKey::Delete => {
                self.points.pop();
                vec![]
            }
            ToolKey::Escape => {
                self.points.clear();
                self.pressed = false;
                vec![]
            }
            _ => vec![],
        }
    }
    fn claims_key(&self, _cx: &ToolContext, key: ToolKey) -> bool {
        !self.points.is_empty() && matches!(key, ToolKey::Enter | ToolKey::Escape | ToolKey::Backspace | ToolKey::Delete)
    }
    /// Switching tools keeps what was drawn, as the Pen does.
    fn deactivate(&mut self, _cx: &ToolContext) -> Vec<Action> {
        self.finish()
    }
    fn overlays(&self, _cx: &ToolContext) -> Vec<Overlay> {
        let Some((first, rest)) = self.points.split_first() else { return vec![] };
        let mut line = BezPath::new();
        line.move_to(*first);
        for p in rest {
            line.line_to(*p);
        }
        let mut o = vec![Overlay::Path { path: line, color: FEEDBACK, width: 1.5, dashed: false }];
        if let (Some(last), Some(h), false) = (self.points.last(), self.hover, self.pressed) {
            o.push(Overlay::Line { a: *last, b: h, color: FEEDBACK, dashed: true });
        }
        o.push(Overlay::Anchor { p: *first, color: FEEDBACK, filled: true, size: 5.0 });
        o
    }
    fn cursor(&self, _cx: &ToolContext, _p: Point, _m: Mods) -> Cursor {
        Cursor::Crosshair
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use vectorcraft_doc::Selection;

    fn ev(kind: PointerKind, x: f64, y: f64) -> PointerEvent {
        PointerEvent::new(kind, x, y)
    }

    fn click(t: &mut dyn Tool, cx: &ToolContext, x: f64, y: f64) -> Vec<Action> {
        let mut out = t.pointer(cx, &ev(PointerKind::Down, x, y));
        out.extend(t.pointer(cx, &ev(PointerKind::Up, x, y)));
        out
    }

    fn marker(a: [f64; 2], b: [f64; 2]) -> Action {
        Action::Exec("trowel.insertSectionMarker".into(), json!({"kind": "cut", "p0": a, "p1": b}))
    }

    #[test]
    fn section_cut_takes_two_clicks_or_a_drag() {
        let (d, _) = doc_with_rect();
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = SectionCutTool::default();
        // Two clicks: the first places the first point and the preview follows the pointer.
        assert!(click(&mut t, &cx, 50.0, 300.0).is_empty());
        // Waiting for the second point, it isn't busy (the keyboard's shortcuts work) but takes Esc.
        assert!(!t.busy() && t.claims_key(&cx, ToolKey::Escape));
        t.pointer(&cx, &ev(PointerKind::Move, 250.0, 310.0));
        let o = t.overlays(&cx);
        assert!(o.iter().any(|o| matches!(o, Overlay::Line { dashed: true, .. })), "{o:?}");
        assert!(o.iter().any(|o| matches!(o, Overlay::Label { text, .. } if text == "A–A′")), "{o:?}");
        assert_eq!(click(&mut t, &cx, 250.0, 310.0), vec![marker([50.0, 300.0], [250.0, 310.0])]);
        assert!(!t.claims_key(&cx, ToolKey::Escape) && t.overlays(&cx).is_empty());
        // One drag, busy while the button is down.
        let mut a = t.pointer(&cx, &ev(PointerKind::Down, 10.0, 10.0));
        assert!(t.busy());
        a.extend(t.pointer(&cx, &ev(PointerKind::Drag, 60.0, 40.0)));
        a.extend(t.pointer(&cx, &ev(PointerKind::Up, 110.0, 70.0)));
        assert_eq!(a, vec![marker([10.0, 10.0], [110.0, 70.0])]);
        // Shift: 45° steps.
        click(&mut t, &cx, 0.0, 0.0);
        let shift = Mods { shift: true, ..Default::default() };
        let a = t.pointer(&cx, &ev(PointerKind::Down, 100.0, 90.0).with_mods(shift));
        let [Action::Exec(_, v)] = a.as_slice() else { panic!("{a:?}") };
        let (x, y) = (v["p1"][0].as_f64().unwrap(), v["p1"][1].as_f64().unwrap());
        assert!((x - y).abs() < 1e-9 && x > 90.0, "{v}");
    }

    #[test]
    fn section_cut_waits_for_a_second_point_apart_and_esc_stops() {
        let (d, _) = doc_with_rect();
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = SectionCutTool::default();
        click(&mut t, &cx, 50.0, 50.0);
        // A second click on the first point, or a tiny drag, places nothing.
        assert!(click(&mut t, &cx, 51.0, 50.0).is_empty());
        assert!(t.claims_key(&cx, ToolKey::Escape));
        assert!(t.key(&cx, ToolKey::Escape, Mods::default()).is_empty());
        assert!(!t.busy() && !t.claims_key(&cx, ToolKey::Escape));
        let mut a = t.pointer(&cx, &ev(PointerKind::Down, 10.0, 10.0));
        a.extend(t.pointer(&cx, &ev(PointerKind::Drag, 11.0, 11.0)));
        a.extend(t.pointer(&cx, &ev(PointerKind::Up, 11.0, 11.0)));
        assert!(a.is_empty() && t.claims_key(&cx, ToolKey::Escape), "a short drag is a click");
        assert!(t.deactivate(&cx).is_empty());
        assert!(!t.claims_key(&cx, ToolKey::Escape));
        // Backspace (the pen bar's Undo) takes the first point back.
        click(&mut t, &cx, 50.0, 50.0);
        assert!(t.claims_key(&cx, ToolKey::Backspace));
        assert!(t.key(&cx, ToolKey::Backspace, Mods::default()).is_empty());
        assert!(!t.claims_key(&cx, ToolKey::Backspace) && t.overlays(&cx).is_empty());
    }

    /// Zoomed in past 400 %, two points 4 px apart are under a point apart on paper: the tool
    /// waits for one the command takes, keeping the first.
    #[test]
    fn section_cut_points_are_a_point_apart_on_paper() {
        let (d, _) = doc_with_rect();
        let (s, p) = (Selection::default(), paint());
        let mut cx = cx(&d, &s, &p);
        cx.zoom = 8.0;
        let mut t = SectionCutTool::default();
        click(&mut t, &cx, 10.0, 10.0);
        assert!(click(&mut t, &cx, 10.6, 10.0).is_empty(), "4.8 px but 0.6 pt");
        assert_eq!(click(&mut t, &cx, 11.0, 10.0), vec![marker([10.0, 10.0], [11.0, 10.0])]);
    }

    #[test]
    fn section_cut_previews_the_next_free_letter() {
        let (mut d, _) = doc_with_rect();
        let l = d.layers[0].id;
        let id = d.alloc_id();
        let mut g = vectorcraft_doc::Node::group(id, vec![]);
        g.name = Some("단면 기호 A–A′".into());
        d.insert(Some(l), usize::MAX, g).unwrap();
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = SectionCutTool::default();
        click(&mut t, &cx, 0.0, 0.0);
        t.pointer(&cx, &ev(PointerKind::Move, 100.0, 0.0));
        assert!(t.overlays(&cx).iter().any(|o| matches!(o, Overlay::Label { text, .. } if text == "B–B′")));
    }

    #[test]
    fn level_lines_mark_the_line_and_ask_for_the_height() {
        let (d, id) = doc_with_rect();
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = create("levelLines").unwrap();
        // On the rectangle's top edge: marked, and a click asks for the height there.
        t.pointer(&cx, &ev(PointerKind::Move, 150.0, 101.0));
        assert!(matches!(t.overlays(&cx).as_slice(), [Overlay::Path { width: 2.0, .. }]));
        assert_eq!(click(t.as_mut(), &cx, 150.0, 101.0), vec![Action::Dialog(ASK_LEVEL.into(), json!({"profileId": id.0, "at": [150.0, 101.0]}))]);
        // Inside it (not on its line) or on nothing: what to click.
        t.pointer(&cx, &ev(PointerKind::Move, 150.0, 150.0));
        assert!(t.overlays(&cx).is_empty());
        assert_eq!(click(t.as_mut(), &cx, 150.0, 150.0), vec![Action::Dialog(HINT.into(), json!({"tool": "levelLines"}))]);
        assert_eq!(click(t.as_mut(), &cx, 400.0, 400.0), vec![Action::Dialog(HINT.into(), json!({"tool": "levelLines"}))]);
    }

    /// The level lines a click made, a datum tick or an A–A′ mark are not ground lines: a click on
    /// one over the ground line takes the ground line, on its own nothing.
    #[test]
    fn marks_are_not_lines_to_click() {
        let (mut d, id) = doc_with_rect();
        let l = d.layers[0].id;
        for (name, y) in [("단면 해발 눈금 (32.5m)", 101.0), ("단면 기호 A–A′", 300.0)] {
            let (gid, pid) = (d.alloc_id(), d.alloc_id());
            let line = vectorcraft_doc::Node::path(
                pid,
                vectorcraft_geom::PathData::single(vectorcraft_geom::SubPath::polyline(&[Point::new(0.0, y), Point::new(400.0, y)], false)),
                vectorcraft_doc::Appearance::default_art(),
            );
            let mut g = vectorcraft_doc::Node::group(gid, vec![std::sync::Arc::new(line)]);
            g.name = Some(name.into());
            d.insert(Some(l), usize::MAX, g).unwrap();
        }
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = create("levelLines").unwrap();
        assert_eq!(click(t.as_mut(), &cx, 150.0, 101.0), vec![Action::Dialog(ASK_LEVEL.into(), json!({"profileId": id.0, "at": [150.0, 101.0]}))]);
        assert_eq!(click(t.as_mut(), &cx, 50.0, 300.0), vec![Action::Dialog(HINT.into(), json!({"tool": "levelLines"}))]);
    }

    #[test]
    fn slope_hachure_runs_along_the_clicked_line() {
        let (d, id) = doc_with_rect();
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = create("slopeHachure").unwrap();
        assert_eq!(click(t.as_mut(), &cx, 200.0, 150.0), vec![Action::Exec("trowel.slopeHachures".into(), json!({"topId": id.0}))]);
        assert_eq!(click(t.as_mut(), &cx, 400.0, 400.0), vec![Action::Dialog(HINT.into(), json!({"tool": "slopeHachure"}))]);
    }

    #[test]
    fn spot_height_asks_for_the_height_of_the_point() {
        let (d, _) = doc_with_rect();
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = create("spotHeight").unwrap();
        t.pointer(&cx, &ev(PointerKind::Move, 40.0, 60.0));
        let o = t.overlays(&cx);
        let [Overlay::Path { path, .. }] = o.as_slice() else { panic!("{o:?}") };
        let b = vectorcraft_geom::Shape::bounding_box(path);
        assert!((b.y1 - 60.0).abs() < 1e-9 && (b.width() - 7.0).abs() < 1e-9, "{b:?}");
        assert_eq!(click(t.as_mut(), &cx, 40.0, 60.0), vec![Action::Dialog(ASK_SPOT.into(), json!({"at": [40.0, 60.0]}))]);
        assert!(!t.busy());
    }

    #[test]
    fn strata_line_by_clicks_and_freehand() {
        let (d, _) = doc_with_rect();
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = StrataLineTool::default();
        assert!(click(&mut t, &cx, 10.0, 10.0).is_empty());
        // A drag adds freehand points, not one on top of the other.
        t.pointer(&cx, &ev(PointerKind::Down, 50.0, 20.0));
        for x in [51.0, 60.0, 70.0] {
            t.pointer(&cx, &ev(PointerKind::Drag, x, 20.0));
        }
        assert!(t.busy(), "while the button is down");
        t.pointer(&cx, &ev(PointerKind::Up, 70.0, 20.0));
        assert!(!t.busy() && t.claims_key(&cx, ToolKey::Enter) && t.claims_key(&cx, ToolKey::Backspace));
        t.pointer(&cx, &ev(PointerKind::Move, 90.0, 30.0));
        let o = t.overlays(&cx);
        assert!(o.iter().any(|o| matches!(o, Overlay::Line { dashed: true, .. })), "the next segment: {o:?}");
        // Backspace takes the last point back; Enter ends it.
        t.key(&cx, ToolKey::Backspace, Mods::default());
        assert_eq!(
            t.key(&cx, ToolKey::Enter, Mods::default()),
            vec![Action::Exec("stratigraphy.boundary".into(), json!({"points": [[10.0, 10.0], [50.0, 20.0], [60.0, 20.0]]}))]
        );
        assert!(!t.claims_key(&cx, ToolKey::Enter) && !t.claims_key(&cx, ToolKey::Backspace));
    }

    #[test]
    fn strata_line_double_click_finishes_and_esc_stops() {
        let (d, _) = doc_with_rect();
        let (s, p) = (Selection::default(), paint());
        let cx = cx(&d, &s, &p);
        let mut t = StrataLineTool::default();
        click(&mut t, &cx, 10.0, 10.0);
        // A double-click: its second press lands on its first and ends the line there.
        assert!(click(&mut t, &cx, 100.0, 10.0).is_empty());
        assert_eq!(
            click(&mut t, &cx, 101.0, 10.0),
            vec![Action::Exec("stratigraphy.boundary".into(), json!({"points": [[10.0, 10.0], [100.0, 10.0]]}))]
        );
        assert!(t.pointer(&cx, &ev(PointerKind::DoubleClick, 101.0, 10.0)).is_empty() && !t.claims_key(&cx, ToolKey::Enter));
        // A double-click's second press a little off (5 px) ends it too.
        click(&mut t, &cx, 10.0, 10.0);
        click(&mut t, &cx, 100.0, 10.0);
        assert_eq!(click(&mut t, &cx, 104.0, 13.0).len(), 1, "within the double-click distance");
        // Two quick clicks at two points are a double-click to the window: they add both.
        click(&mut t, &cx, 10.0, 10.0);
        click(&mut t, &cx, 60.0, 10.0);
        assert!(t.pointer(&cx, &ev(PointerKind::DoubleClick, 60.0, 10.0)).is_empty());
        assert!(t.claims_key(&cx, ToolKey::Enter));
        // A press on the last point dragged on goes on freehand.
        t.pointer(&cx, &ev(PointerKind::Down, 60.0, 10.0));
        t.pointer(&cx, &ev(PointerKind::Drag, 80.0, 10.0));
        assert!(t.pointer(&cx, &ev(PointerKind::Up, 80.0, 10.0)).is_empty() && t.claims_key(&cx, ToolKey::Enter));
        t.key(&cx, ToolKey::Escape, Mods::default());
        // One point is no line; Esc drops what there is.
        click(&mut t, &cx, 10.0, 10.0);
        assert!(t.key(&cx, ToolKey::Enter, Mods::default()).is_empty());
        click(&mut t, &cx, 10.0, 10.0);
        click(&mut t, &cx, 30.0, 10.0);
        assert!(t.key(&cx, ToolKey::Escape, Mods::default()).is_empty());
        assert!(!t.claims_key(&cx, ToolKey::Escape));
        // Switching tools keeps the line.
        click(&mut t, &cx, 10.0, 10.0);
        click(&mut t, &cx, 30.0, 10.0);
        assert!(matches!(t.deactivate(&cx).as_slice(), [Action::Exec(c, _)] if c == "stratigraphy.boundary"));
    }

    #[test]
    fn the_archaeology_tools_are_made_by_id() {
        for id in ["sectionCut", "slopeHachure", "levelLines", "spotHeight", "strataLine"] {
            assert_eq!(crate::create(id).id(), id);
            assert!(crate::tool_info(id).is_some(), "{id}");
        }
    }
}
