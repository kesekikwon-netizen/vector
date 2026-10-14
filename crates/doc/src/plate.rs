//! Report Plate layout, viewports, title block, scale bar, and vector export models
//! (Trowel design 2.1 §6, §9, §14(a, 1), Task J012).

use serde::{Deserialize, Serialize};
use vectorcraft_geom::{Affine, Point, Rect, Vec2};

use crate::legend::MM;
use crate::scale::ScaleRatio;

/// Standard archaeological publication sheet formats.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PlateSize {
    /// A4 (210.0 × 297.0 mm) - standard report & field size.
    A4,
    /// 4×6배판 (188.0 × 257.0 mm) - Korean archaeological monograph volume standard.
    CrownQuarto4x6,
    /// A3 (297.0 × 420.0 mm) - large plate / fold-out volume / submission drawings.
    A3,
    /// Custom user-specified dimensions in millimeters.
    Custom { width_mm: f64, height_mm: f64 },
}

impl Default for PlateSize {
    fn default() -> Self {
        Self::A4
    }
}

impl PlateSize {
    /// Width and height in millimeters according to orientation.
    pub fn dimensions_mm(&self, orientation: PlateOrientation) -> (f64, f64) {
        let (w, h) = match self {
            PlateSize::A4 => (210.0, 297.0),
            PlateSize::CrownQuarto4x6 => (188.0, 257.0),
            PlateSize::A3 => (297.0, 420.0),
            PlateSize::Custom { width_mm, height_mm } => (*width_mm, *height_mm),
        };
        match orientation {
            PlateOrientation::Portrait => (w, h),
            PlateOrientation::Landscape => (h, w),
        }
    }

    /// Width and height in points (pt) according to orientation.
    pub fn dimensions_pt(&self, orientation: PlateOrientation) -> (f64, f64) {
        let (w_mm, h_mm) = self.dimensions_mm(orientation);
        (w_mm * MM, h_mm * MM)
    }

    /// Human-readable label in Korean.
    pub fn label(&self) -> &'static str {
        match self {
            PlateSize::A4 => "A4 (210×297 mm)",
            PlateSize::CrownQuarto4x6 => "4×6배판 (188×257 mm)",
            PlateSize::A3 => "A3 (297×420 mm)",
            PlateSize::Custom { .. } => "사용자 정의",
        }
    }
}

/// Paper orientation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlateOrientation {
    #[default]
    Portrait,
    Landscape,
}

/// Source reference for a viewport item.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "target", rename_all = "lowercase")]
pub enum ViewportSource {
    WholeDocument,
    Layer(crate::NodeId),
    Artboard(usize),
    ExternalFile(String),
}

impl Default for ViewportSource {
    fn default() -> Self {
        Self::WholeDocument
    }
}

/// A drawing viewport placed on a report plate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewportItem {
    pub id: String,
    pub name: String,
    pub source: ViewportSource,
    /// Position and size on the sheet in sheet points.
    pub bounds: Rect,
    /// Scale on the plate (e.g. 1:60).
    pub scale: ScaleRatio,
    /// Base scale in the source drawing (e.g. 1:20).
    pub source_scale: ScaleRatio,
    /// Pan offset within viewport (dx, dy) in sheet points.
    pub pan_offset: Vec2,
    /// Rotation angle in degrees (clockwise).
    pub rotation_deg: f64,
    /// Whether contents are clipped to the viewport bounds.
    pub clip_boundary: bool,
    pub visible: bool,
}

impl ViewportItem {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        bounds: Rect,
        scale: ScaleRatio,
        source_scale: ScaleRatio,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source: ViewportSource::default(),
            bounds,
            scale,
            source_scale,
            pan_offset: Vec2::ZERO,
            rotation_deg: 0.0,
            clip_boundary: true,
            visible: true,
        }
    }

    /// Geometric reduction/magnification factor: e.g. 20.0 / 60.0 = 0.333333.
    pub fn zoom_factor(&self) -> f64 {
        if self.scale.denom <= 1e-4 {
            1.0
        } else {
            self.source_scale.denom / self.scale.denom
        }
    }

    /// Returns the affine transform from source drawing points to plate sheet points.
    pub fn transform(&self) -> Affine {
        let center = self.bounds.center();
        let zoom = self.zoom_factor();
        let rad = self.rotation_deg.to_radians();

        Affine::translate(center.to_vec2() + self.pan_offset)
            * Affine::rotate(rad)
            * Affine::scale(zoom)
    }
}

/// Cartographic title block data (standard 160×65mm title block).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TitleBlockData {
    pub project_title: String,
    pub drawing_title: String,
    pub site_code: String,
    pub excavation_team: String,
    pub director: String,
    pub drawn_by: String,
    pub checked_by: String,
    pub date: String,
    pub scale_text: String,
    pub view_direction: String,
    pub crs_datum: String,
    pub sheet_number: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub visible: bool,
}

impl Default for TitleBlockData {
    fn default() -> Self {
        Self {
            project_title: "광령리 유적 발굴조사".to_string(),
            drawing_title: "1호 주거지 평·단면도".to_string(),
            site_code: "광령리1".to_string(),
            excavation_team: "제주고고학연구소".to_string(),
            director: "권영인".to_string(),
            drawn_by: "권영인".to_string(),
            checked_by: "검토 전".to_string(),
            date: "2026-10-03".to_string(),
            scale_text: "1:40 · A3 가로".to_string(),
            view_direction: "위에서 (평면)".to_string(),
            crs_datum: "EPSG:5186 중부원점 · 해발".to_string(),
            sheet_number: "도면 12".to_string(),
            width_mm: 160.0,
            height_mm: 65.0,
            visible: false,
        }
    }
}

/// Graphical scale bar style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScaleBarStyle {
    #[default]
    AlternatingBlocks,
    SingleLineTicks,
}

/// Graphical scale bar item (막대 축척).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScaleBarItem {
    pub pos: Point,
    pub scale: ScaleRatio,
    pub linked_viewport_id: Option<String>,
    pub segment_unit_m: f64,
    pub segment_count: usize,
    pub height_pt: f64,
    pub style: ScaleBarStyle,
    pub show_scale_label: bool,
    pub visible: bool,
}

impl Default for ScaleBarItem {
    fn default() -> Self {
        Self {
            pos: Point::new(50.0, 50.0),
            scale: ScaleRatio::new(60.0),
            linked_viewport_id: None,
            segment_unit_m: 1.0,
            segment_count: 2,
            height_pt: 4.0,
            style: ScaleBarStyle::AlternatingBlocks,
            show_scale_label: true,
            visible: true,
        }
    }
}

impl ScaleBarItem {
    /// Width of a single metric segment in sheet points.
    pub fn segment_width_pt(&self) -> f64 {
        if self.scale.denom <= 1e-4 {
            0.0
        } else {
            self.segment_unit_m * (1000.0 / self.scale.denom) * MM
        }
    }

    /// Total width of the scale bar in sheet points.
    pub fn total_width_pt(&self) -> f64 {
        self.segment_width_pt() * (self.segment_count as f64)
    }
}

/// Stylized archaeological North Arrow style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NorthArrowStyle {
    #[default]
    ArchaeologicalSplit,
    FourQuadrantCompass,
}

/// Dynamic North Arrow item (방위표 / 도북).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NorthArrowItem {
    pub pos: Point,
    pub rotation_deg: f64,
    pub linked_viewport_id: Option<String>,
    pub size_pt: f64,
    pub label: String,
    pub style: NorthArrowStyle,
    pub visible: bool,
}

impl Default for NorthArrowItem {
    fn default() -> Self {
        Self {
            pos: Point::new(100.0, 100.0),
            rotation_deg: 0.0,
            linked_viewport_id: None,
            size_pt: 28.0,
            label: "도북".to_string(),
            style: NorthArrowStyle::ArchaeologicalSplit,
            visible: true,
        }
    }
}

/// Outer margins and border frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BorderFrame {
    pub margin_mm: f64,
    pub double_border: bool,
    pub inner_offset_mm: f64,
    pub outer_weight_pt: f64,
    pub inner_weight_pt: f64,
    pub grid_ticks: bool,
    pub tick_interval_mm: f64,
    pub tick_length_pt: f64,
    pub visible: bool,
}

impl Default for BorderFrame {
    fn default() -> Self {
        Self {
            margin_mm: 15.0,
            double_border: true,
            inner_offset_mm: 2.0,
            outer_weight_pt: 1.2,
            inner_weight_pt: 0.5,
            grid_ticks: true,
            tick_interval_mm: 50.0,
            tick_length_pt: 3.0,
            visible: true,
        }
    }
}

/// Complete report plate sheet definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlateSheet {
    pub id: String,
    pub name: String,
    pub caption: String,
    pub page_number: String,
    pub size: PlateSize,
    pub orientation: PlateOrientation,
    pub border_frame: BorderFrame,
    pub viewports: Vec<ViewportItem>,
    pub title_block: TitleBlockData,
    pub scale_bar: ScaleBarItem,
    pub north_arrow: NorthArrowItem,
    pub show_legend_table: bool,
    pub show_stratigraphy_table: bool,
}

impl Default for PlateSheet {
    fn default() -> Self {
        Self {
            id: "plate-1".to_string(),
            name: "도면 12".to_string(),
            caption: "1호 주거지 평·단면도".to_string(),
            page_number: "57".to_string(),
            size: PlateSize::A4,
            orientation: PlateOrientation::Portrait,
            border_frame: BorderFrame::default(),
            viewports: Vec::new(),
            title_block: TitleBlockData::default(),
            scale_bar: ScaleBarItem::default(),
            north_arrow: NorthArrowItem::default(),
            show_legend_table: true,
            show_stratigraphy_table: true,
        }
    }
}

impl PlateSheet {
    pub fn sheet_rect_pt(&self) -> Rect {
        let (w, h) = self.size.dimensions_pt(self.orientation);
        Rect::new(0.0, 0.0, w, h)
    }

    pub fn printable_rect_pt(&self) -> Rect {
        let (w, h) = self.size.dimensions_pt(self.orientation);
        let m = self.border_frame.margin_mm * MM;
        Rect::new(m, m, (w - m).max(m), (h - m).max(m))
    }

    /// Synchronizes dependent items (North Arrow angle, Scale Bar ratio) from linked viewports.
    pub fn sync_dependencies(&mut self) {
        if let Some(ref vp_id) = self.north_arrow.linked_viewport_id {
            if let Some(vp) = self.viewports.iter().find(|v| &v.id == vp_id) {
                self.north_arrow.rotation_deg = vp.rotation_deg;
            }
        }
        if let Some(ref vp_id) = self.scale_bar.linked_viewport_id {
            if let Some(vp) = self.viewports.iter().find(|v| &v.id == vp_id) {
                self.scale_bar.scale = vp.scale;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plate_dimensions() {
        let a4_p = PlateSize::A4.dimensions_mm(PlateOrientation::Portrait);
        assert_eq!(a4_p, (210.0, 297.0));
        let a4_l = PlateSize::A4.dimensions_mm(PlateOrientation::Landscape);
        assert_eq!(a4_l, (297.0, 210.0));

        let q_p = PlateSize::CrownQuarto4x6.dimensions_mm(PlateOrientation::Portrait);
        assert_eq!(q_p, (188.0, 257.0));

        let a3_l = PlateSize::A3.dimensions_mm(PlateOrientation::Landscape);
        assert_eq!(a3_l, (420.0, 297.0));

        let (w_pt, h_pt) = PlateSize::A4.dimensions_pt(PlateOrientation::Portrait);
        assert!((w_pt - 595.27559).abs() < 0.01);
        assert!((h_pt - 841.88976).abs() < 0.01);
    }

    #[test]
    fn test_viewport_zoom_and_transform() {
        let vp = ViewportItem::new(
            "vp1",
            "Plan",
            Rect::new(50.0, 50.0, 250.0, 250.0),
            ScaleRatio::new(60.0),
            ScaleRatio::S1_20,
        );
        // 20 / 60 = 1/3
        assert!((vp.zoom_factor() - (1.0 / 3.0)).abs() < 1e-9);
        let xf = vp.transform();
        assert!(xf.as_coeffs().iter().all(|c| c.is_finite()));
    }

    #[test]
    fn test_scale_bar_calculation() {
        let mut sb = ScaleBarItem::default();
        sb.scale = ScaleRatio::new(60.0);
        sb.segment_unit_m = 1.0;
        sb.segment_count = 2;

        // 1m at 1:60 = 1000mm / 60 * (72 / 25.4) = 47.24409 pt
        let seg_pt = sb.segment_width_pt();
        assert!((seg_pt - 47.244).abs() < 0.01);
        assert!((sb.total_width_pt() - 47.244 * 2.0).abs() < 0.02);
    }

    #[test]
    fn test_plate_sheet_sync_dependencies() {
        let mut sheet = PlateSheet::default();
        let mut vp = ViewportItem::new(
            "vp-main",
            "Main Viewport",
            Rect::new(20.0, 20.0, 200.0, 200.0),
            ScaleRatio::new(40.0),
            ScaleRatio::S1_20,
        );
        vp.rotation_deg = 22.5;
        sheet.viewports.push(vp);

        sheet.north_arrow.linked_viewport_id = Some("vp-main".to_string());
        sheet.scale_bar.linked_viewport_id = Some("vp-main".to_string());

        sheet.sync_dependencies();
        assert_eq!(sheet.north_arrow.rotation_deg, 22.5);
        assert_eq!(sheet.scale_bar.scale, ScaleRatio::new(40.0));
    }

    #[test]
    fn test_plate_serialization_roundtrip() {
        let sheet = PlateSheet::default();
        let json_str = serde_json::to_string(&sheet).unwrap();
        let deserialized: PlateSheet = serde_json::from_str(&json_str).unwrap();
        assert_eq!(sheet, deserialized);
    }
}
