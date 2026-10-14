//! Artifact Pottery drafting data models (Trowel design 2.1 §4-3, §5, §14(3), J011).

use serde::{Deserialize, Serialize};

use crate::NodeId;

/// PostScript points per millimeter constant (72 pt / 25.4 mm).
pub const MM: f64 = 72.0 / 25.4;

/// Side of the vessel profile on which the sliced section is placed.
///
/// Section is placed on the right side of the centerline by default per §14(3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionSide {
    #[default]
    Right,
    Left,
}

/// Sliced wall section fill mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionFillMode {
    #[default]
    None,
    #[serde(alias = "solidblack", alias = "solid")]
    Black,
    #[serde(alias = "hatch")]
    Hatch45,
}

/// Archaeological vessel taxonomy types.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VesselType {
    #[default]
    DeepBowl, // 심발 / 바리
    Bowl,     // 발
    Jar,      // 호 / 항아리
    Cup,      // 배 / 잔
    Pot,      // 옹 / 독
    Plate,    // 반 / 접시
    Lid,      // 개 / 뚜껑
    Other(String),
}

/// Physical measurements of the artifact pottery vessel.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PotteryMeasurements {
    pub rim_diameter_mm: Option<f64>,
    pub base_diameter_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub max_belly_diameter_mm: Option<f64>,
    pub wall_thickness_mm: Option<f64>,
    pub rim_preservation_percent: Option<f64>,
}

impl PotteryMeasurements {
    pub fn rim_diameter_cm(&self) -> Option<f64> {
        self.rim_diameter_mm.map(|v| v / 10.0)
    }

    pub fn base_diameter_cm(&self) -> Option<f64> {
        self.base_diameter_mm.map(|v| v / 10.0)
    }

    pub fn height_cm(&self) -> Option<f64> {
        self.height_mm.map(|v| v / 10.0)
    }

    pub fn wall_thickness_cm(&self) -> Option<f64> {
        self.wall_thickness_mm.map(|v| v / 10.0)
    }

    /// Formats the 2x2 ledger readout strings (구경, 기고, 저경, 기벽).
    pub fn format_ledger(&self) -> (String, String, String, String) {
        let fmt = |v_cm: Option<f64>| match v_cm {
            Some(v) => format!("{:.1} cm", v),
            None => "-".to_string(),
        };
        (
            format!("구경 {}", fmt(self.rim_diameter_cm())),
            format!("기고 {}", fmt(self.height_cm())),
            format!("저경 {}", fmt(self.base_diameter_cm())),
            format!("기벽 {}", fmt(self.wall_thickness_cm())),
        )
    }
}

fn default_target_scale() -> String {
    "1:3".to_string()
}

/// Artifact pottery vessel drafting session record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PotteryVessel {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub vessel_type: VesselType,
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub find_number: String,
    #[serde(default)]
    pub feature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_m: Option<f64>,
    #[serde(default)]
    pub preservation_note: String,

    pub centerline_x: f64,
    #[serde(default)]
    pub section_side: SectionSide,
    #[serde(default)]
    pub fill_mode: SectionFillMode,
    #[serde(default = "default_target_scale")]
    pub target_scale: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub centerline_node: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outer_profile_node: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_profile_node: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reflected_node: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section_polygon_node: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hatch_lines_node: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ridge_nodes: Vec<NodeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pattern_nodes: Vec<NodeId>,

    #[serde(default)]
    pub measurements: PotteryMeasurements,
}

impl Default for PotteryVessel {
    fn default() -> Self {
        Self {
            id: "vessel-1".to_string(),
            name: "심발".to_string(),
            vessel_type: VesselType::DeepBowl,
            period: "청동기시대".to_string(),
            find_number: "1".to_string(),
            feature: "1호 주거지".to_string(),
            elevation_m: None,
            preservation_note: "완형".to_string(),
            centerline_x: 300.0,
            section_side: SectionSide::Right,
            fill_mode: SectionFillMode::None,
            target_scale: "1:3".to_string(),
            centerline_node: None,
            outer_profile_node: None,
            inner_profile_node: None,
            reflected_node: None,
            section_polygon_node: None,
            hatch_lines_node: None,
            ridge_nodes: Vec::new(),
            pattern_nodes: Vec::new(),
            measurements: PotteryMeasurements::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pottery_measurements_ledger() {
        let m = PotteryMeasurements {
            rim_diameter_mm: Some(184.0),
            base_diameter_mm: Some(86.0),
            height_mm: Some(212.0),
            max_belly_diameter_mm: Some(195.0),
            wall_thickness_mm: Some(7.0),
            rim_preservation_percent: Some(66.7),
        };

        assert_eq!(m.rim_diameter_cm(), Some(18.4));
        assert_eq!(m.height_cm(), Some(21.2));
        assert_eq!(m.base_diameter_cm(), Some(8.6));
        assert_eq!(m.wall_thickness_cm(), Some(0.7));

        let (rim, h, base, t) = m.format_ledger();
        assert_eq!(rim, "구경 18.4 cm");
        assert_eq!(h, "기고 21.2 cm");
        assert_eq!(base, "저경 8.6 cm");
        assert_eq!(t, "기벽 0.7 cm");
    }

    #[test]
    fn test_section_side_defaults() {
        let vessel = PotteryVessel::default();
        assert_eq!(vessel.section_side, SectionSide::Right);
        assert_eq!(vessel.fill_mode, SectionFillMode::None);
    }
}
