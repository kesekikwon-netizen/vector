//! Where Trowel's panels go (design 3.1 §13): which column holds which panel and in what order,
//! how wide each column is, how tall the panels the user sized are, which columns are folded and
//! which panels are hidden. It is kept with the UI preferences ([`crate::state::UiState`]) and in
//! workspaces, never in a drawing. Window › Reset Panel Layout and Edit › Preferences › Reset
//! Preferences bring back [`Layout::default`], the layout the window first opens with. The window
//! draws it in `trowel::layout`.

use serde::{Deserialize, Deserializer, Serialize};

/// A panel that moves between Trowel's columns. The tool column, the ribbon, the status bar and
/// the canvas stay where they are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Panel {
    /// The drawing set (and the element box with it).
    Set,
    Legend,
    Pen,
    /// The properties.
    Props,
    /// The layer strip.
    Layers,
}

impl Panel {
    pub const ALL: [Panel; 5] = [Panel::Set, Panel::Legend, Panel::Pen, Panel::Props, Panel::Layers];

    /// Its name in commands (`trowel.layout.panel`'s `panel`).
    pub fn id(self) -> &'static str {
        match self {
            Panel::Set => "set",
            Panel::Legend => "legend",
            Panel::Pen => "pen",
            Panel::Props => "props",
            Panel::Layers => "layers",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == s)
    }

    /// Whether it takes the height its column has left: the drawing set, the legend and the layer
    /// strip. The pen and the properties are as tall as what they show.
    pub fn fills(self) -> bool {
        matches!(self, Panel::Set | Panel::Legend | Panel::Layers)
    }

    /// The column it goes back to when a layout has lost it.
    fn home(self) -> Column {
        match self {
            Panel::Set => Column::Left,
            Panel::Legend | Panel::Pen | Panel::Props => Column::Right,
            Panel::Layers => Column::Edge,
        }
    }
}

/// One of the columns the panels go in: left of the canvas, right of it, and the layer strip's
/// column at the window's right edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Column {
    Left,
    Right,
    Edge,
}

impl Column {
    pub const ALL: [Column; 3] = [Column::Left, Column::Right, Column::Edge];

    /// Its name in commands (`trowel.layout.move`'s `column`).
    pub fn id(self) -> &'static str {
        match self {
            Column::Left => "left",
            Column::Right => "right",
            Column::Edge => "edge",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.id() == s)
    }

    /// The widths it can be dragged to, in points.
    pub fn range(self) -> std::ops::RangeInclusive<f32> {
        match self {
            Column::Left => 140.0..=450.0,
            Column::Right => 220.0..=560.0,
            Column::Edge => 112.0..=240.0,
        }
    }

    /// The width it first has (design 3.1 §13, at 1440 × 880: 50 · 236 · canvas · 304 · 132).
    pub fn default_width(self) -> f32 {
        match self {
            Column::Left => 236.0,
            Column::Right => 304.0,
            Column::Edge => 132.0,
        }
    }

    /// The window width under which it is folded unless the user opened it by hand: the drawer
    /// under 1200 (design 3.0 §9), the layer strip under 1400 (§13). The right column never folds
    /// by itself: the legend is the app's centre.
    pub fn folds_under(self) -> Option<f32> {
        match self {
            Column::Left => Some(1200.0),
            Column::Right => None,
            Column::Edge => Some(1400.0),
        }
    }
}

/// A folded column's width: a rail with its name.
pub const FOLDED_W: f32 = 28.0;
/// The canvas keeps at least this width: the columns that fold by themselves fold for it, the
/// layer strip first, then the drawer.
pub const CANVAS_MIN_W: f32 = 480.0;

/// A hidden panel and where it was, where showing it puts it back.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hidden {
    pub panel: Panel,
    pub column: Column,
    pub index: usize,
}

/// A height the user gave a panel by dragging the boundary under or over it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Height {
    pub panel: Panel,
    pub height: f32,
}

/// Trowel's panel layout. Read with [`lenient`]: a layout an older version saved, or one edited by
/// hand that doesn't read, gives the default rather than losing the other preferences.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Layout {
    /// Each column's panels, top first.
    pub left: Vec<Panel>,
    pub right: Vec<Panel>,
    pub edge: Vec<Panel>,
    pub left_width: f32,
    pub right_width: f32,
    pub edge_width: f32,
    /// Folded by hand (`Some(true)`), opened by hand (`Some(false)`), or as the window's width
    /// says (`None`: [`Column::folds_under`], [`CANVAS_MIN_W`]).
    pub left_folded: Option<bool>,
    pub right_folded: Option<bool>,
    pub edge_folded: Option<bool>,
    /// Heights the user gave panels; the others are as tall as they show, or share what is left.
    pub heights: Vec<Height>,
    /// The panels hidden from the Window menu, each with where it was.
    pub hidden: Vec<Hidden>,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            left: vec![Panel::Set],
            right: vec![Panel::Legend, Panel::Pen, Panel::Props],
            edge: vec![Panel::Layers],
            left_width: Column::Left.default_width(),
            right_width: Column::Right.default_width(),
            edge_width: Column::Edge.default_width(),
            left_folded: None,
            right_folded: None,
            edge_folded: None,
            heights: vec![],
            hidden: vec![],
        }
    }
}

/// How a column shows in a window: as wide as `width`, folded to its rail or open.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shown {
    pub width: f32,
    pub folded: bool,
}

impl Layout {
    pub fn panels(&self, c: Column) -> &[Panel] {
        match c {
            Column::Left => &self.left,
            Column::Right => &self.right,
            Column::Edge => &self.edge,
        }
    }

    fn panels_mut(&mut self, c: Column) -> &mut Vec<Panel> {
        match c {
            Column::Left => &mut self.left,
            Column::Right => &mut self.right,
            Column::Edge => &mut self.edge,
        }
    }

    pub fn width(&self, c: Column) -> f32 {
        match c {
            Column::Left => self.left_width,
            Column::Right => self.right_width,
            Column::Edge => self.edge_width,
        }
    }

    /// Set column `c`'s width, kept in its [`Column::range`].
    pub fn set_width(&mut self, c: Column, w: f32) {
        let r = c.range();
        let w = if w.is_finite() { w.clamp(*r.start(), *r.end()) } else { c.default_width() };
        match c {
            Column::Left => self.left_width = w,
            Column::Right => self.right_width = w,
            Column::Edge => self.edge_width = w,
        }
    }

    /// Whether column `c` was folded or opened by hand (`None`: as the window says).
    pub fn folded_by_hand(&self, c: Column) -> Option<bool> {
        match c {
            Column::Left => self.left_folded,
            Column::Right => self.right_folded,
            Column::Edge => self.edge_folded,
        }
    }

    pub fn set_folded(&mut self, c: Column, folded: Option<bool>) {
        match c {
            Column::Left => self.left_folded = folded,
            Column::Right => self.right_folded = folded,
            Column::Edge => self.edge_folded = folded,
        }
    }

    /// Where panel `p` is: its column and place there (none while it is hidden).
    pub fn place_of(&self, p: Panel) -> Option<(Column, usize)> {
        Column::ALL.into_iter().find_map(|c| Some((c, self.panels(c).iter().position(|x| *x == p)?)))
    }

    pub fn is_shown(&self, p: Panel) -> bool {
        self.place_of(p).is_some()
    }

    /// Put panel `p` in column `to` at `index` (0: the top; past the end: the bottom), out of
    /// wherever it was. A column it goes to that was folded by hand opens.
    pub fn move_panel(&mut self, p: Panel, to: Column, index: usize) {
        let mut index = index;
        if let Some((from, at)) = self.place_of(p) {
            self.panels_mut(from).retain(|x| *x != p);
            if from == to && at < index {
                index -= 1;
            }
        }
        self.hidden.retain(|h| h.panel != p);
        let list = self.panels_mut(to);
        let index = index.min(list.len());
        list.insert(index, p);
        if self.folded_by_hand(to) == Some(true) {
            self.set_folded(to, Some(false));
        }
    }

    /// Hide panel `p`, keeping where it was for [`Layout::show`].
    pub fn hide(&mut self, p: Panel) {
        if let Some((column, index)) = self.place_of(p) {
            self.panels_mut(column).retain(|x| *x != p);
            self.hidden.retain(|h| h.panel != p);
            self.hidden.push(Hidden { panel: p, column, index });
        }
    }

    /// Show panel `p` again where it was when it was hidden.
    pub fn show(&mut self, p: Panel) {
        if self.is_shown(p) {
            return;
        }
        let (column, index) = self.hidden.iter().find(|h| h.panel == p).map_or((p.home(), usize::MAX), |h| (h.column, h.index));
        self.move_panel(p, column, index);
    }

    /// The height the user gave panel `p`.
    pub fn height(&self, p: Panel) -> Option<f32> {
        self.heights.iter().find(|h| h.panel == p).map(|h| h.height)
    }

    /// Give panel `p` a height (`None`: its own again).
    pub fn set_height(&mut self, p: Panel, height: Option<f32>) {
        self.heights.retain(|h| h.panel != p);
        if let Some(height) = height.filter(|h| h.is_finite() && *h > 0.0) {
            self.heights.push(Height { panel: p, height });
        }
    }

    /// How each column shows in a window `window_w` wide whose tool column is `tools_w` wide:
    /// `None` for a column without panels. A column folds by hand, or under its
    /// [`Column::folds_under`] width. When the canvas would be narrower than [`CANVAS_MIN_W`], the
    /// open columns first get narrower, down to the least of their range (the layer strip, then
    /// the drawer, then the right column), and only then the strip, then the drawer, fold to their
    /// rail (never one opened by hand): a panel never goes from the window (the user, 2026-10-16:
    /// 「줄여도 사라지지않고 축소되게」). The widths kept stay as they are.
    pub fn shown(&self, window_w: f32, tools_w: f32) -> [Option<Shown>; 3] {
        let first = |c: Column| self.folded_by_hand(c).unwrap_or_else(|| c.folds_under().is_some_and(|w| window_w < w));
        let mut folded = Column::ALL.map(first);
        let room = window_w - tools_w - CANVAS_MIN_W;
        let fit = |folded: &[bool; 3]| {
            let mut out = Column::ALL.map(|c| {
                let i = column_index(c);
                let f = folded.get(i).copied().unwrap_or(false);
                (!self.panels(c).is_empty()).then(|| Shown { width: if f { FOLDED_W } else { self.width(c) }, folded: f })
            });
            let mut over = out.iter().flatten().map(|s| s.width).sum::<f32>() - room;
            for c in [Column::Edge, Column::Left, Column::Right] {
                if over <= 0.0 {
                    break;
                }
                if let Some(s) = out.get_mut(column_index(c)).and_then(Option::as_mut).filter(|s| !s.folded) {
                    let give = over.min(s.width - c.range().start()).max(0.0);
                    s.width -= give;
                    over -= give;
                }
            }
            (out, over)
        };
        let (mut out, mut over) = fit(&folded);
        for c in [Column::Edge, Column::Left] {
            if over <= 0.0 {
                break;
            }
            let i = column_index(c);
            if self.folded_by_hand(c).is_none() && !self.panels(c).is_empty() && folded.get(i) == Some(&false) {
                if let Some(f) = folded.get_mut(i) {
                    *f = true;
                }
                (out, over) = fit(&folded);
            }
        }
        out
    }

    /// Keep each panel once (shown or hidden), bring back the ones lost to their own column's
    /// bottom, and widths, heights and places that are numbers in range (a hand-edited file).
    pub fn sanitize(&mut self) {
        let mut seen = std::collections::BTreeSet::new();
        for c in Column::ALL {
            self.panels_mut(c).retain(|p| seen.insert(*p));
        }
        self.hidden.retain(|h| seen.insert(h.panel));
        for p in Panel::ALL {
            if !seen.contains(&p) {
                self.panels_mut(p.home()).push(p);
            }
        }
        for c in Column::ALL {
            self.set_width(c, self.width(c));
        }
        let mut sized = std::collections::BTreeSet::new();
        self.heights.retain(|h| h.height.is_finite() && h.height > 0.0 && sized.insert(h.panel));
    }
}

/// The layout commands (`trowel.layout.*`, [`crate::menus::UI_COMMANDS`]): they change only the
/// UI preferences, never a drawing, so they are no undo steps.
pub fn run_command(ui: &mut crate::state::UiState, id: &str, p: &serde_json::Value) -> Option<Result<serde_json::Value, String>> {
    use serde_json::{Value, json};
    let s = |k: &str| p.get(k).and_then(Value::as_str);
    let panel = || s("panel").and_then(Panel::parse).ok_or_else(|| crate::tw!("panel must be one of set, legend, pen, props, layers").to_string());
    let column = || s("column").and_then(Column::parse).ok_or_else(|| crate::tw!("column must be one of left, right, edge").to_string());
    let l = &mut ui.trowel_layout;
    let r = match id {
        "trowel.layout.reset" => {
            *l = Layout::default();
            Ok(Value::Null)
        }
        "trowel.layout.panel" => panel().and_then(|panel| {
            let shown = match p.get("shown") {
                None | Some(Value::Null) => !l.is_shown(panel),
                Some(Value::Bool(b)) => *b,
                Some(_) => return Err(crate::tw!("shown must be true or false").to_string()),
            };
            if shown {
                l.show(panel);
            } else {
                l.hide(panel);
            }
            Ok(json!({ "shown": shown }))
        }),
        "trowel.layout.move" => panel().and_then(|panel| {
            let column = column()?;
            let index = match p.get("index") {
                None | Some(Value::Null) => usize::MAX,
                Some(v) => {
                    v.as_u64().and_then(|i| usize::try_from(i).ok()).ok_or_else(|| crate::tw!("index must be a whole number from 0").to_string())?
                }
            };
            l.move_panel(panel, column, index);
            let (column, index) = l.place_of(panel).ok_or_else(|| crate::tw!("the panel went missing").to_string())?;
            Ok(json!({ "column": column.id(), "index": index }))
        }),
        "trowel.layout.size" => column().and_then(|column| {
            if let Some(w) = p.get("width").filter(|v| !v.is_null()) {
                l.set_width(column, w.as_f64().ok_or_else(|| crate::tw!("width must be a number").to_string())? as f32);
            }
            match p.get("folded") {
                None => {}
                Some(Value::Null) => l.set_folded(column, None),
                Some(Value::Bool(b)) => l.set_folded(column, Some(*b)),
                Some(_) => return Err(crate::tw!("folded must be true, false or null").to_string()),
            }
            Ok(json!({ "width": l.width(column), "folded": l.folded_by_hand(column) }))
        }),
        "trowel.layout.get" => serde_json::to_value(&*l).map_err(|e| e.to_string()),
        _ => return None,
    };
    Some(r)
}

/// Where column `c` is in [`Column::ALL`] (and in [`Layout::shown`]'s answer).
pub fn column_index(c: Column) -> usize {
    match c {
        Column::Left => 0,
        Column::Right => 1,
        Column::Edge => 2,
    }
}

/// Read a saved layout, or the default when it doesn't read (an older or hand-edited file): the
/// rest of the preferences still load.
pub fn lenient<'de, D: Deserializer<'de>>(d: D) -> Result<Layout, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    let mut layout: Layout = serde_json::from_value(value).unwrap_or_default();
    layout.sanitize();
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_layout_is_the_mockups() {
        let l = Layout::default();
        assert_eq!(l.left, [Panel::Set]);
        assert_eq!(l.right, [Panel::Legend, Panel::Pen, Panel::Props]);
        assert_eq!(l.edge, [Panel::Layers]);
        // 1440 × 880: 50 · 236 · 718 · 304 · 132.
        let s = l.shown(1440.0, 50.0);
        assert_eq!(s.map(|s| s.map(|s| (s.width, s.folded))), [Some((236.0, false)), Some((304.0, false)), Some((132.0, false))]);
        // Under 1400 the strip folds; under 1200 the drawer too.
        assert_eq!(l.shown(1366.0, 50.0)[2], Some(Shown { width: FOLDED_W, folded: true }));
        assert!(!l.shown(1366.0, 50.0)[0].is_some_and(|s| s.folded));
        assert!(l.shown(1100.0, 50.0)[0].is_some_and(|s| s.folded));
    }

    #[test]
    fn a_narrow_canvas_narrows_the_columns_then_folds_the_strip_then_the_drawer() {
        let mut l = Layout::default();
        l.set_width(Column::Right, 560.0);
        l.set_width(Column::Left, 450.0);
        l.set_width(Column::Edge, 240.0);
        let widths = |s: [Option<Shown>; 3]| s.map(|s| s.map(|s| (s.width, s.folded)));
        // 1600 - 50 - 480 = 1070 for 450 + 560 + 240 = 1250: the strip gives 128 (to 112), the
        // drawer 52; nothing folds.
        assert_eq!(widths(l.shown(1600.0, 50.0)), [Some((398.0, false)), Some((560.0, false)), Some((112.0, false))]);
        // 1500: 970 for them: the strip gives 128, the drawer 152.
        assert_eq!(widths(l.shown(1500.0, 50.0)), [Some((298.0, false)), Some((560.0, false)), Some((112.0, false))]);
        // 1300, the strip folded under 1400 by itself: 770 for 450 + 560 + 28; the drawer gives
        // 268 to 182.
        assert_eq!(widths(l.shown(1300.0, 50.0)), [Some((182.0, false)), Some((560.0, false)), Some((FOLDED_W, true))]);
        // Opened by hand at 1300 it stays open, at its least, and the drawer gives the rest.
        l.set_folded(Column::Edge, Some(false));
        assert_eq!(widths(l.shown(1300.0, 50.0)), [Some((140.0, false)), Some((518.0, false)), Some((112.0, false))]);
        l.set_folded(Column::Edge, None);
        // 1250 with the right column at its least: 450 + 220 + 28 = 698 fits 720.
        l.set_width(Column::Right, 220.0);
        assert_eq!(widths(l.shown(1250.0, 50.0)), [Some((450.0, false)), Some((220.0, false)), Some((FOLDED_W, true))]);
        l.set_width(Column::Right, 560.0);
        // Too little room even with every column at its least (a 600-point tool column): the strip
        // folds first, then the drawer, and the right column narrows.
        assert_eq!(widths(l.shown(1450.0, 600.0)), [Some((FOLDED_W, true)), Some((314.0, false)), Some((FOLDED_W, true))]);
        // 800, under every rule: rails, and the right column at its least.
        assert_eq!(widths(l.shown(800.0, 50.0)), [Some((FOLDED_W, true)), Some((220.0, false)), Some((FOLDED_W, true))]);
        // The widths kept are the user's.
        assert_eq!((l.left_width, l.right_width, l.edge_width), (450.0, 560.0, 240.0));
    }

    #[test]
    fn panels_move_hide_and_come_back_where_they_were() {
        let mut l = Layout::default();
        l.move_panel(Panel::Pen, Column::Left, 0);
        assert_eq!(l.left, [Panel::Pen, Panel::Set]);
        assert_eq!(l.right, [Panel::Legend, Panel::Props]);
        // Down in its own column: past the props.
        l.move_panel(Panel::Legend, Column::Right, 2);
        assert_eq!(l.right, [Panel::Props, Panel::Legend]);
        l.hide(Panel::Props);
        assert_eq!(l.right, [Panel::Legend]);
        assert!(!l.is_shown(Panel::Props));
        l.show(Panel::Props);
        assert_eq!(l.right, [Panel::Props, Panel::Legend]);
        // An emptied column has no panels; one folded by hand opens when a panel goes in.
        l.move_panel(Panel::Layers, Column::Right, 9);
        assert!(l.edge.is_empty() && l.shown(1600.0, 50.0)[2].is_none());
        l.set_folded(Column::Edge, Some(true));
        l.move_panel(Panel::Layers, Column::Edge, 0);
        assert_eq!(l.edge_folded, Some(false));
    }

    #[test]
    fn a_saved_layout_reads_leniently() {
        #[derive(Deserialize)]
        struct Holder {
            #[serde(default, deserialize_with = "lenient")]
            layout: Layout,
        }
        let read = |s: &str| serde_json::from_str::<Holder>(s).map(|h| h.layout);
        // None saved, nonsense, an unknown panel: the default.
        assert_eq!(read("{}").ok(), Some(Layout::default()));
        assert_eq!(read(r#"{"layout": 7}"#).ok(), Some(Layout::default()));
        assert_eq!(read(r#"{"layout": {"left": ["toolbox"]}}"#).ok(), Some(Layout::default()));
        // Twice the same panel, lost panels, a width out of range.
        let l = read(r#"{"layout": {"left": ["pen", "pen"], "right": ["legend"], "edge": [], "rightWidth": 9000, "heights": [{"panel": "pen", "height": -3}]}}"#)
            .ok()
            .unwrap_or_default();
        assert_eq!(l.left, [Panel::Pen, Panel::Set]);
        assert_eq!(l.right, [Panel::Legend, Panel::Props]);
        assert_eq!(l.edge, [Panel::Layers]);
        assert_eq!(l.right_width, 560.0);
        assert!(l.heights.is_empty());
        // Round trip.
        let mut moved = Layout::default();
        moved.move_panel(Panel::Pen, Column::Left, 1);
        moved.set_height(Panel::Pen, Some(150.0));
        moved.set_folded(Column::Edge, Some(true));
        let json = serde_json::to_string(&moved).unwrap_or_default();
        let back: Layout = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(back, moved);
    }

    /// The layout is saved with the UI preferences and the workspaces, never with a drawing: an
    /// older preferences file without it, or with a broken one, opens on the first layout and keeps
    /// the rest of what it saved.
    #[test]
    fn the_layout_is_kept_with_the_preferences() {
        use crate::state::UiState;
        let mut ui = UiState { toolbar: false, ..UiState::default() };
        ui.trowel_layout.move_panel(Panel::Pen, Column::Left, 1);
        ui.trowel_layout.set_width(Column::Right, 400.0);
        let saved = serde_json::to_value(&ui).unwrap();
        let back: UiState = serde_json::from_value(saved.clone()).unwrap();
        assert_eq!(back.trowel_layout, ui.trowel_layout);
        for old in [None, Some(serde_json::json!("broken")), Some(serde_json::json!({ "left": 3 }))] {
            let mut file = saved.clone();
            let fields = file.as_object_mut().unwrap();
            assert!(fields.remove("trowel_layout").is_some(), "saved under its name");
            if let Some(v) = old.clone() {
                fields.insert("trowel_layout".into(), v);
            }
            let back: UiState = serde_json::from_value(file).unwrap();
            assert_eq!(back.trowel_layout, Layout::default(), "{old:?}");
            assert!(!back.toolbar, "{old:?}: the rest as saved");
        }
        // A workspace keeps it too, and Reset Essentials brings back the first one.
        let w = crate::workspaces::capture(&ui, "Mine");
        let mut other = UiState::default();
        crate::workspaces::apply(&mut other, &w);
        assert_eq!(other.trowel_layout, ui.trowel_layout);
        crate::workspaces::apply(&mut other, &crate::workspaces::Workspace::default());
        assert_eq!(other.trowel_layout, Layout::default());
    }
}
