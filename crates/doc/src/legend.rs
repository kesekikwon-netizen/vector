//! Legend rows (Trowel, design 2.1 §7): a top-level layer can carry a legend row, the default look
//! of the lines drawn on it (weight in points and line type) and how the row shows in the legend
//! panel.

use serde::{Deserialize, Serialize};

use crate::{Document, Node, NodeId, NodeKind, StrokeLayer};

/// One paper millimetre in points. Line type dash lengths are set in millimetres.
pub const MM: f64 = 72.0 / 25.4;
/// How close two weights or dash lengths (points) must be to count as the same: the Select Same
/// Stroke Weight tolerance (`select.same.strokeWeight`).
pub const SAME: f64 = 1e-9;
/// The heaviest weight a row takes (points), as Stroke Weight (`stroke.set`).
pub const MAX_WEIGHT: f64 = 1000.0;
/// The most dash lengths a line type takes.
pub const MAX_DASHES: usize = 16;
/// The longest mark text and Strata key (characters).
pub const MAX_MARK: usize = 32;
pub const MAX_KEY: usize = 64;

/// A legend row: the look of its layer's lines and how the row shows. Every field is optional in
/// the file; defaults are not written.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Legend {
    /// Line weight in points; `None`: the row has no look (marks, type, underlays) and leaves its
    /// lines alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(default, skip_serializing_if = "crate::skip::is_default")]
    pub line: LineType,
    /// ★: a row used often (pinned first in its section, clay name).
    #[serde(default, skip_serializing_if = "crate::skip::is_default")]
    pub favourite: bool,
    #[serde(default, skip_serializing_if = "crate::skip::is_default")]
    pub sample: LegendSample,
    /// The Strata layer key this row matches (`feature_poly`, …; hgis
    /// `data/schemas/ka_hgis_layers.yaml`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strata: Option<String>,
    /// The legend group the row came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<LegendGroup>,
}

/// A line type. Saved by name; dash lengths are paper millimetres from Strata's line looks (hgis
/// `src/core/FeaturePresets.cpp`: 구 3;1.5, 수혈 0.6;1.2, 도랑 3;1.2;0.6;1.2).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineType {
    #[default]
    Solid,
    Dashed,
    Dotted,
    DashDot,
    /// Any other dash pattern, in points.
    Custom(Vec<f64>),
}

/// The little picture in front of a row's name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LegendSample {
    #[default]
    Line,
    Area,
    /// A mark row (△, ③, 152.5m …) shows this text.
    Mark(String),
}

/// The legend groups of design 2.1 §7.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LegendGroup {
    Plan,
    Section,
    Finds,
}

impl LegendGroup {
    pub const ALL: [LegendGroup; 3] = [LegendGroup::Plan, LegendGroup::Section, LegendGroup::Finds];
    pub fn key(self) -> &'static str {
        match self {
            LegendGroup::Plan => "plan",
            LegendGroup::Section => "section",
            LegendGroup::Finds => "finds",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.key() == s)
    }
}

fn same_lengths(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= SAME)
}

impl LineType {
    /// The dash pattern in points (`None`: solid).
    pub fn dash_pt(&self) -> Option<Vec<f64>> {
        let mm: &[f64] = match self {
            LineType::Solid => return None,
            LineType::Dashed => &[3.0, 1.5],
            LineType::Dotted => &[0.6, 1.2],
            LineType::DashDot => &[3.0, 1.2, 0.6, 1.2],
            LineType::Custom(p) => return Some(p.clone()),
        };
        Some(mm.iter().map(|v| v * MM).collect())
    }

    /// What a dash pattern (points) reads as: solid when it draws no dashes, a named type when
    /// every length is within [`SAME`] of it, else the pattern itself.
    pub fn from_dash(pattern: &[f64]) -> LineType {
        let dash = crate::Dash { pattern: pattern.to_vec(), ..crate::Dash::default() };
        if !dash.is_dashed() {
            return LineType::Solid;
        }
        [LineType::Dashed, LineType::Dotted, LineType::DashDot]
            .into_iter()
            .find(|t| t.dash_pt().is_some_and(|p| same_lengths(&p, pattern)))
            .unwrap_or(LineType::Custom(dash.pattern))
    }

    /// What a stroke's dashes read as ([`Self::from_dash`]; no dashes: solid).
    pub fn of(st: &StrokeLayer) -> LineType {
        st.dash.as_ref().map_or(LineType::Solid, |d| LineType::from_dash(&d.pattern))
    }

    /// The dashes it draws: `None` when solid or when its pattern draws none.
    fn drawn(&self) -> Option<Vec<f64>> {
        self.dash_pt().filter(|p| crate::Dash { pattern: p.clone(), ..crate::Dash::default() }.is_dashed())
    }

    /// The same dashes (a custom pattern equal to a named one is that type; one that draws none is
    /// solid).
    pub fn same(&self, other: &LineType) -> bool {
        match (self.drawn(), other.drawn()) {
            (None, None) => true,
            (Some(a), Some(b)) => same_lengths(&a, &b),
            _ => false,
        }
    }
}

impl Legend {
    /// The row's look (weight and line type), when it has a weight.
    pub fn look(&self) -> Option<(f64, &LineType)> {
        self.weight.map(|w| (w, &self.line))
    }

    /// Does this stroke differ from the row's look, a line set apart ("따로")? Only weight and
    /// dashes count; a row without a look has none.
    pub fn differs(&self, st: &StrokeLayer) -> bool {
        let Some((w, line)) = self.look() else { return false };
        (st.width - w).abs() > SAME || !line.same(&LineType::of(st))
    }

    /// The row as the commands take it, for one a file gave (untrusted): a weight within
    /// 0–[`MAX_WEIGHT`] (none when not a number), dashes that draw, at most [`MAX_DASHES`] (else
    /// solid; a named type's read as that type), a mark and a Strata key trimmed and cut to
    /// [`MAX_MARK`] and [`MAX_KEY`] characters (an empty mark shows a line, an empty key is none).
    pub fn tidied(mut self) -> Legend {
        self.weight = self.weight.filter(|w| w.is_finite()).map(|w| w.clamp(0.0, MAX_WEIGHT));
        if let LineType::Custom(p) = &self.line {
            self.line = if p.len() <= MAX_DASHES { LineType::from_dash(p) } else { LineType::Solid };
        }
        if let LegendSample::Mark(m) = &self.sample {
            let m = clip(m, MAX_MARK);
            self.sample = if m.is_empty() { LegendSample::Line } else { LegendSample::Mark(m) };
        }
        self.strata = self.strata.map(|k| clip(&k, MAX_KEY)).filter(|k| !k.is_empty());
        self
    }
}

/// Text trimmed and cut to `max` characters (and trimmed again where the cut lands on a space).
pub fn clip(s: &str, max: usize) -> String {
    let cut: String = s.trim().chars().take(max).collect();
    cut.trim_end().to_string()
}

/// Read a layer's legend leniently: one this version can't read (a line type from a newer version,
/// a value of another kind) leaves the layer without a legend instead of failing the whole file;
/// one it reads is [tidied](Legend::tidied).
pub(crate) fn read<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Box<Legend>>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Read {
        Legend(Legend),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Option::<Read>::deserialize(d)? {
        Some(Read::Legend(l)) => Some(Box::new(l.tidied())),
        Some(Read::Other(_)) | None => None,
    })
}

/// The painted top stroke of a line.
pub fn stroke_of(n: &Node) -> Option<&StrokeLayer> {
    n.appearance.stroke().filter(|s| !s.paint.is_none())
}

/// Is this node a line: a path (not a guide or a clipping path) or a compound path with a painted
/// top stroke?
pub fn is_line(n: &Node) -> bool {
    matches!(n.kind, NodeKind::Path { guide: false, clipping: false, .. } | NodeKind::Compound { .. }) && stroke_of(n).is_some()
}

/// The lines of a row, top first: [`is_line`] objects in the layer, its groups and sublayers. Type,
/// images, symbols, blends and other live art are not lines and are not looked into, nor are a
/// compound's own paths.
pub fn lines(layer: &Node) -> Vec<&Node> {
    fn walk<'a>(n: &'a Node, out: &mut Vec<&'a Node>) {
        let (NodeKind::Layer { children, .. } | NodeKind::Group { children, .. }) = &n.kind else { return };
        for c in children.iter().rev() {
            if matches!(c.kind, NodeKind::Layer { .. } | NodeKind::Group { .. }) {
                walk(c, out);
            } else if is_line(c) {
                out.push(c);
            }
        }
    }
    let mut out = vec![];
    walk(layer, &mut out);
    out
}

impl Node {
    /// A top-level layer's legend row.
    pub fn legend(&self) -> Option<&Legend> {
        match &self.kind {
            NodeKind::Layer { legend, .. } => legend.as_deref(),
            _ => None,
        }
    }
}

impl Document {
    /// The legend row of the top-level layer `layer` (a sublayer is no row).
    pub fn legend(&self, layer: NodeId) -> Option<&Legend> {
        self.layers.iter().find(|l| l.id == layer)?.legend()
    }
}

use vectorcraft_geom::{Point, Rect};

/// A generated legend entry corresponding to an active layer/feature in the drawing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegendEntry {
    /// Layer identifier or unique key.
    pub layer_id: NodeId,
    /// Layer or feature name (e.g. "굴광선 (유구 면)").
    pub label: String,
    /// Stroke weight in points, if any.
    pub weight_pt: Option<f64>,
    /// Line type / dash pattern.
    pub line_type: LineType,
    /// Sample visual kind: Line, Area, or Mark(String).
    pub sample_kind: LegendSample,
    /// Optional hatch style, if area hatch is associated with this layer.
    pub hatch: Option<crate::hatch::HatchStyle>,
    /// Optional description or subtitle (e.g. "유구 윤곽", "0.8 pt").
    pub description: Option<String>,
    /// Total count of active stroked/filled elements found on this layer.
    pub element_count: usize,
}

/// Layout and formatting options for rendering a legend table block onto paper (§6, §7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegendLayoutOptions {
    /// Anchor position on paper/sheet (pt).
    pub origin: Point,
    /// Total width of the legend block in points (default: 148.8 pt / ~52.5 mm).
    pub table_width_pt: f64,
    /// Row height in points (default: 11.0 pt, as in design 2.1 §6).
    pub row_height_pt: f64,
    /// Width of the sample swatch line/rectangle (default: 28.0 pt / ~10 mm).
    pub swatch_width_pt: f64,
    /// Height of the sample swatch rectangle for area/pattern (default: 8.0 pt / ~2.8 mm).
    pub swatch_height_pt: f64,
    /// Gap between sample swatch and text label (default: 8.0 pt).
    pub swatch_gap_pt: f64,
    /// Font size for label text in points (default: 8.0 pt, per §7).
    pub font_size_pt: f64,
    /// Optional font family (e.g. "Noto Sans KR", "Malgun Gothic").
    pub font_family: Option<String>,
    /// Whether to render a 0.7 pt horizontal rule above the table (default: true).
    pub top_rule: bool,
    /// Whether to render a 0.7 pt horizontal rule below the table (default: true).
    pub bottom_rule: bool,
    /// Whether to draw an enclosing border box around the entire table (default: false).
    pub enclosing_box: bool,
    /// Number of columns (default: 1; >1 splits entries into multiple columns).
    pub column_count: usize,
    /// Gap between columns in points (default: 14.0 pt).
    pub column_gap_pt: f64,
    /// Hide unused layers (layers with 0 stroked/filled elements, default: true).
    pub filter_unused: bool,
}

impl Default for LegendLayoutOptions {
    fn default() -> Self {
        Self {
            origin: Point::new(0.0, 0.0),
            table_width_pt: 148.8,
            row_height_pt: 11.0,
            swatch_width_pt: 28.0,
            swatch_height_pt: 8.0,
            swatch_gap_pt: 8.0,
            font_size_pt: 8.0,
            font_family: None,
            top_rule: true,
            bottom_rule: true,
            enclosing_box: false,
            column_count: 1,
            column_gap_pt: 14.0,
            filter_unused: true,
        }
    }
}

/// Generated Auto Legend Table model.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LegendTable {
    /// Active entries extracted from the document.
    pub entries: Vec<LegendEntry>,
    /// Layout and styling options applied.
    pub options: LegendLayoutOptions,
    /// Computed bounding box of the entire table on the artboard/sheet.
    pub bounds: Rect,
}

impl LegendTable {
    /// Build a LegendTable model by scanning the active document layers.
    /// Strictly filters out unused layers when `options.filter_unused` is true.
    pub fn build_from_document(doc: &Document, options: &LegendLayoutOptions) -> Self {
        let mut entries = Vec::new();

        for layer in &doc.layers {
            if !layer.visible || layer.is_template() {
                continue;
            }
            let line_nodes = lines(layer);
            let elem_count = if !line_nodes.is_empty() {
                line_nodes.len()
            } else {
                layer.children().map_or(0, |c| c.len())
            };

            if options.filter_unused && elem_count == 0 {
                continue; // "쓰지 않은 줄은 범례표에 나오지 않습니다"
            }

            let Some(legend) = layer.legend() else { continue };

            entries.push(LegendEntry {
                layer_id: layer.id,
                label: layer.display_name().trim().to_string(),
                weight_pt: legend.weight,
                line_type: legend.line.clone(),
                sample_kind: legend.sample.clone(),
                hatch: None,
                description: None,
                element_count: elem_count,
            });
        }

        // Sort: favourites (★) first, then by group or layer order
        entries.sort_by(|a, b| {
            let fav_a = doc.node(a.layer_id).and_then(Node::legend).is_some_and(|l| l.favourite);
            let fav_b = doc.node(b.layer_id).and_then(Node::legend).is_some_and(|l| l.favourite);
            fav_b.cmp(&fav_a)
        });

        let rows_per_col = if options.column_count > 1 {
            (entries.len() + options.column_count - 1) / options.column_count
        } else {
            entries.len()
        };

        let total_h = if entries.is_empty() {
            0.0
        } else {
            (rows_per_col as f64) * options.row_height_pt + 8.0
        };

        let bounds = Rect::from_origin_size(
            options.origin,
            vectorcraft_geom::Size::new(options.table_width_pt, total_h),
        );

        Self {
            entries,
            options: options.clone(),
            bounds,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use vectorcraft_color::{Color, Paint};
    use vectorcraft_geom::{Affine, PathData, Point, SubPath};

    use super::*;
    use crate::{Appearance, Dash, ImageObject, LayerColor};

    const MM: f64 = 72.0 / 25.4;

    fn stroke(w: f64, dash: Option<Vec<f64>>) -> StrokeLayer {
        let mut st = StrokeLayer::new(Paint::solid(Color::BLACK), w);
        st.dash = dash.map(|pattern| Dash { pattern, ..Dash::default() });
        st
    }

    fn line(id: u64, w: f64) -> Node {
        let path = PathData::new(vec![SubPath::polyline(&[Point::new(0.0, 0.0), Point::new(10.0, 0.0)], false)]);
        Node::path(NodeId(id), path, Appearance::basic(Paint::None, Paint::solid(Color::BLACK), w))
    }

    fn with_children(mut n: Node, kids: Vec<Node>) -> Node {
        if let Some(c) = n.children_mut() {
            c.extend(kids.into_iter().map(Arc::new));
        }
        n
    }

    #[test]
    fn dash_lengths_are_stratas_paper_mm() {
        // Strata's line looks (hgis src/core/FeaturePresets.cpp:21-23): 구 3;1.5, 수혈 0.6;1.2,
        // 도랑 3;1.2;0.6;1.2 (mm).
        assert_eq!(LineType::Solid.dash_pt(), None);
        assert_eq!(LineType::Dashed.dash_pt(), Some(vec![3.0 * MM, 1.5 * MM]));
        assert_eq!(LineType::Dotted.dash_pt(), Some(vec![0.6 * MM, 1.2 * MM]));
        assert_eq!(LineType::DashDot.dash_pt(), Some(vec![3.0 * MM, 1.2 * MM, 0.6 * MM, 1.2 * MM]));
        let dashed = LineType::Dashed.dash_pt().unwrap_or_default();
        assert!((dashed[0] - 8.503_937_007_874_016).abs() < 1e-12, "3 mm is 8.504 pt: {}", dashed[0]);
        assert_eq!(LineType::Custom(vec![2.0, 1.0]).dash_pt(), Some(vec![2.0, 1.0]));
    }

    #[test]
    fn line_type_is_read_back_from_a_stroke() {
        assert_eq!(LineType::of(&stroke(0.5, None)), LineType::Solid);
        assert_eq!(LineType::of(&stroke(0.5, Some(vec![0.0, 0.0]))), LineType::Solid, "a dash that draws nothing is solid");
        for t in [LineType::Dashed, LineType::Dotted, LineType::DashDot] {
            let p = t.dash_pt().unwrap_or_default();
            assert_eq!(LineType::of(&stroke(0.5, Some(p.clone()))), t);
            let near: Vec<f64> = p.iter().map(|v| v + 5e-10).collect();
            assert_eq!(LineType::of(&stroke(0.5, Some(near))), t, "within 1e-9 pt");
            let off: Vec<f64> = p.iter().map(|v| v + 2e-9).collect();
            assert_eq!(LineType::of(&stroke(0.5, Some(off.clone()))), LineType::Custom(off), "past 1e-9 pt");
        }
        assert_eq!(LineType::of(&stroke(0.5, Some(vec![5.0, 5.0]))), LineType::Custom(vec![5.0, 5.0]));
        let dashed = LineType::Dashed.dash_pt().unwrap_or_default();
        let doubled: Vec<f64> = dashed.iter().chain(dashed.iter()).copied().collect();
        assert_eq!(LineType::of(&stroke(0.5, Some(doubled.clone()))), LineType::Custom(doubled), "the number of lengths counts");
        assert!(LineType::Custom(dashed).same(&LineType::Dashed));
        assert!(!LineType::Solid.same(&LineType::Dashed));
        assert!(LineType::Solid.same(&LineType::Solid));
    }

    #[test]
    fn a_dash_pattern_reads_as_its_line_type() {
        for t in [LineType::Dashed, LineType::Dotted, LineType::DashDot] {
            assert_eq!(LineType::from_dash(&t.dash_pt().unwrap_or_default()), t);
        }
        assert_eq!(LineType::from_dash(&[4.0, 2.0]), LineType::Custom(vec![4.0, 2.0]));
        assert_eq!(LineType::from_dash(&[0.0, 0.0]), LineType::Solid, "dashes that draw nothing");
        assert_eq!(LineType::from_dash(&[]), LineType::Solid);
        assert_eq!(LineType::from_dash(&[-1.0, 2.0]), LineType::Solid, "a pattern that doesn't dash");
    }

    #[test]
    fn lines_are_stroked_paths_and_compounds_under_groups_and_sublayers() {
        let mut fill_only = line(2, 1.0);
        fill_only.appearance = Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 1.0);
        let mut guide = line(3, 1.0);
        if let NodeKind::Path { guide: g, .. } = &mut guide.kind {
            *g = true;
        }
        let mut clipping = line(5, 1.0);
        if let NodeKind::Path { clipping: c, .. } = &mut clipping.kind {
            *c = true;
        }
        let mut clip_group = with_children(Node::group(NodeId(6), vec![]), vec![clipping, line(7, 1.0)]);
        if let NodeKind::Group { clip, .. } = &mut clip_group.kind {
            *clip = true;
        }
        let group = with_children(Node::group(NodeId(4), vec![]), vec![line(8, 1.0), clip_group]);
        let mut compound =
            with_children(Node::new(NodeId(9), NodeKind::Compound { children: vec![], rule: Default::default() }), vec![line(10, 1.0)]);
        compound.appearance = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.5);
        let image = Node::new(
            NodeId(11),
            NodeKind::Image(ImageObject { key: "k".into(), width: 1, height: 1, xf: Affine::IDENTITY, link: None, placement: Default::default() }),
        );
        let sub = with_children(Node::layer(NodeId(12), "sub", LayerColor::Preset(0)), vec![line(13, 1.0)]);
        let layer =
            with_children(Node::layer(NodeId(1), "row", LayerColor::Preset(0)), vec![line(14, 1.0), fill_only, guide, group, compound, image, sub]);
        let ids: Vec<u64> = lines(&layer).iter().map(|n| n.id.0).collect();
        // Top first: the last child is the topmost.
        assert_eq!(ids, vec![13, 9, 7, 8, 14]);
    }

    #[test]
    fn an_override_differs_in_weight_or_line_type() {
        let row = Legend { weight: Some(0.8), ..Legend::default() };
        assert!(!row.differs(&stroke(0.8, None)));
        assert!(!row.differs(&stroke(0.8 + 5e-10, None)), "within 1e-9 pt");
        assert!(row.differs(&stroke(0.8 + 2e-9, None)), "past 1e-9 pt");
        assert!(row.differs(&stroke(0.5, None)));
        assert!(row.differs(&stroke(0.8, LineType::Dashed.dash_pt())));
        let dashed = Legend { weight: Some(0.5), line: LineType::Dashed, ..Legend::default() };
        let mut st = stroke(0.5, LineType::Dashed.dash_pt());
        if let Some(d) = st.dash.as_mut() {
            (d.offset, d.align_corners) = (2.0, true);
        }
        st.cap = crate::LineCap::Round;
        st.paint = Paint::solid(Color::rgb(1.0, 0.0, 0.0));
        assert!(!dashed.differs(&st), "offset, corner alignment, cap and colour are not the look");
        assert!(dashed.differs(&stroke(0.5, None)));
        let no_look = Legend { weight: None, line: LineType::Dashed, ..Legend::default() };
        assert!(!no_look.differs(&stroke(3.0, None)), "a row without a weight has no overrides");
        assert_eq!(no_look.look(), None);
        assert_eq!(dashed.look(), Some((0.5, &LineType::Dashed)));
    }

    #[test]
    fn a_pattern_that_draws_nothing_is_solid() {
        // A file can hold dashes that draw nothing: they look like a solid line.
        let nothing = LineType::Custom(vec![0.0, 0.0]);
        assert!(nothing.same(&LineType::Solid));
        assert!(LineType::Solid.same(&nothing));
        assert!(!nothing.same(&LineType::Dashed));
        let row = Legend { weight: Some(0.5), line: nothing, ..Legend::default() };
        assert!(!row.differs(&stroke(0.5, None)), "a solid line follows the row");
        assert!(!row.differs(&stroke(0.5, Some(vec![0.0]))));
    }

    #[test]
    fn only_top_level_layers_are_legend_rows() {
        let row = Legend { weight: Some(0.5), ..Legend::default() };
        let with_row = |mut n: Node| {
            if let NodeKind::Layer { legend, .. } = &mut n.kind {
                *legend = Some(Box::new(row.clone()));
            }
            n
        };
        let sub = with_row(Node::layer(NodeId(3), "sub", LayerColor::Preset(1)));
        let top = with_children(with_row(Node::layer(NodeId(2), "top", LayerColor::Preset(0))), vec![sub]);
        let mut d = Document::new(100.0, 100.0);
        d.layers = vec![Arc::new(top)];
        assert_eq!(d.legend(NodeId(2)), Some(&row));
        assert_eq!(d.legend(NodeId(3)), None, "a sublayer is not a row");
    }

    #[test]
    fn odd_values_are_tidied_as_the_commands_take_them() {
        let dashed = LineType::Dashed.dash_pt().unwrap_or_default();
        let cases = [
            (
                Legend { weight: Some(-3.0), line: LineType::Custom(vec![0.0, 0.0]), sample: LegendSample::Mark("  ".into()), ..Legend::default() },
                Legend { weight: Some(0.0), ..Legend::default() },
            ),
            (
                Legend {
                    weight: Some(5000.0),
                    line: LineType::Custom(vec![1.0; 17]),
                    strata: Some(format!(" {} ", "k".repeat(100))),
                    ..Legend::default()
                },
                Legend { weight: Some(1000.0), strata: Some("k".repeat(64)), ..Legend::default() },
            ),
            (
                Legend { weight: Some(f64::NAN), line: LineType::Custom(vec![1.0, f64::INFINITY]), strata: Some("  ".into()), ..Legend::default() },
                Legend::default(),
            ),
            (
                Legend {
                    weight: Some(0.5),
                    line: LineType::Custom(dashed),
                    sample: LegendSample::Mark(format!(" {} ", "가".repeat(40))),
                    ..Legend::default()
                },
                Legend { weight: Some(0.5), line: LineType::Dashed, sample: LegendSample::Mark("가".repeat(32)), ..Legend::default() },
            ),
        ];
        for (odd, tidy) in cases {
            assert_eq!(odd.clone().tidied(), tidy, "{odd:?}");
            assert_eq!(tidy.clone().tidied(), tidy, "tidy rows stay as they are");
        }
        let fine = Legend {
            weight: Some(0.35),
            line: LineType::Custom(vec![2.5, 1.25]),
            favourite: true,
            sample: LegendSample::Mark("③".into()),
            strata: Some("artifact_point".into()),
            group: Some(LegendGroup::Finds),
        };
        assert_eq!(fine.clone().tidied(), fine);
    }

    /// A legend this version can't read leaves its layer without one (the file still opens), and
    /// one it reads is tidied.
    #[test]
    fn layers_read_their_legend_leniently() {
        let read = |legend: serde_json::Value| -> Option<Legend> {
            let mut v = serde_json::to_value(Node::layer(NodeId(1), "row", LayerColor::Preset(0))).unwrap_or_default();
            v["kind"]["legend"] = legend;
            let n: Node = serde_json::from_value(v).unwrap_or_else(|e| panic!("the layer opens: {e}"));
            n.legend().cloned()
        };
        assert_eq!(read(serde_json::json!({"weight": 0.5, "line": "wavy"})), None, "a line type from a newer version");
        assert_eq!(read(serde_json::json!({"weight": "thick"})), None);
        assert_eq!(read(serde_json::json!([1, 2])), None);
        assert_eq!(read(serde_json::Value::Null), None);
        assert_eq!(
            read(serde_json::json!({"weight": 0.8, "group": "plan"})),
            Some(Legend { weight: Some(0.8), group: Some(LegendGroup::Plan), ..Legend::default() })
        );
        assert_eq!(
            read(serde_json::json!({"weight": -1, "line": {"custom": [0, 0]}})),
            Some(Legend { weight: Some(0.0), ..Legend::default() }),
            "tidied"
        );
    }

    #[test]
    fn legends_save_by_name() {
        let row = Legend {
            weight: Some(0.2),
            line: LineType::DashDot,
            favourite: true,
            sample: LegendSample::Mark("③".into()),
            strata: Some("artifact_point".into()),
            group: Some(LegendGroup::Plan),
        };
        let v = serde_json::to_value(&row).unwrap_or_default();
        assert_eq!(
            v,
            serde_json::json!({"weight": 0.2, "line": "dashdot", "favourite": true, "sample": {"mark": "③"}, "strata": "artifact_point", "group": "plan"})
        );
        let back: Legend = serde_json::from_value(v).unwrap_or_default();
        assert_eq!(back, row);
        assert_eq!(serde_json::to_value(Legend::default()).unwrap_or_default(), serde_json::json!({}), "defaults are not written");
        let custom = Legend { line: LineType::Custom(vec![1.0, 2.0]), sample: LegendSample::Area, ..Legend::default() };
        assert_eq!(serde_json::to_value(&custom).unwrap_or_default(), serde_json::json!({"line": {"custom": [1.0, 2.0]}, "sample": "area"}));
    }

    #[test]
    fn test_legend_table_filters_unused_layers() {
        let mut doc = Document::new(500.0, 500.0);
        let id1 = doc.alloc_id();
        let mut l1 = Node::layer(id1, "굴광선 (유구 면)", LayerColor::Preset(0));
        let p = line(doc.alloc_id().0, 0.8);
        if let Some(c) = l1.children_mut() {
            c.push(Arc::new(p));
        }
        if let NodeKind::Layer { legend, .. } = &mut l1.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.8),
                line: LineType::Solid,
                favourite: true,
                sample: LegendSample::Area,
                strata: Some("feature_poly".into()),
                group: Some(LegendGroup::Plan),
            }));
        }

        let id2 = doc.alloc_id();
        let mut l2 = Node::layer(id2, "시굴격자", LayerColor::Preset(1));
        // l2 has no child lines (unused!)
        if let NodeKind::Layer { legend, .. } = &mut l2.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.5),
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Area,
                strata: Some("trial_trench".into()),
                group: Some(LegendGroup::Plan),
            }));
        }

        doc.layers = vec![Arc::new(l1), Arc::new(l2)];

        let opts = LegendLayoutOptions::default();
        let table = LegendTable::build_from_document(&doc, &opts);
        // Only l1 should be present because l2 has 0 elements and filter_unused is true!
        assert_eq!(table.entries.len(), 1);
        assert_eq!(table.entries[0].label, "굴광선 (유구 면)");

        // If filter_unused is false, both appear
        let mut opts_all = opts.clone();
        opts_all.filter_unused = false;
        let table_all = LegendTable::build_from_document(&doc, &opts_all);
        assert_eq!(table_all.entries.len(), 2);
    }
}

