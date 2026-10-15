//! Automatic horizontal elevation lines model (Trowel design 2.1 §4-1, §14(b, c, d)).

use crate::NodeId;
use serde::{Deserialize, Serialize};

/// Default minor elevation interval: 10 cm (0.1 m).
pub const DEFAULT_MINOR_INTERVAL_M: f64 = 0.1;

/// Default major elevation interval: 50 cm (0.5 m).
pub const DEFAULT_MAJOR_INTERVAL_M: f64 = 0.5;

/// Default minor elevation stroke weight on paper: 0.2 pt.
pub const DEFAULT_MINOR_WEIGHT_PT: f64 = 0.2;

/// Default major elevation stroke weight on paper: 0.5 pt.
pub const DEFAULT_MAJOR_WEIGHT_PT: f64 = 0.5;

/// Default elevation label text font size: 8.0 pt.
pub const DEFAULT_LABEL_FONT_SIZE_PT: f64 = 8.0;

/// Formatting style for elevation labels (Trowel spec §14(c)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElevationLabelFormat {
    /// Format with 'm' suffix: e.g. "152.5m" (Trowel default §14(c)).
    #[default]
    MetersSuffix,
    /// Format with 'EL.' prefix: e.g. "EL. 152.5".
    ElPrefix,
    /// Raw number: e.g. "152.50".
    Raw,
}

impl ElevationLabelFormat {
    pub fn format(self, elevation_m: f64) -> String {
        match self {
            ElevationLabelFormat::MetersSuffix => {
                if (elevation_m * 10.0).fract().abs() < 1e-4 {
                    format!("{:.1}m", elevation_m)
                } else {
                    format!("{:.2}m", elevation_m)
                }
            }
            ElevationLabelFormat::ElPrefix => format!("EL. {:.2}", elevation_m),
            ElevationLabelFormat::Raw => format!("{:.2}", elevation_m),
        }
    }
}

/// Placement position for elevation label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElevationLabelPlacement {
    /// Left end of major lines only (Trowel default §14(c)).
    #[default]
    LeftEnd,
    /// Right end of major lines only (Korean field report standard e.g. ┣ 93.0m).
    RightEnd,
    /// Both left and right ends.
    BothEnds,
}

/// Range calculation mode for elevation lines.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum ElevationRangeMode {
    /// Auto: From the lowest point of the section up to the first 50 cm line
    /// above the highest section point (§4-1).
    #[serde(rename = "auto")]
    #[default]
    Auto,
    /// Explicit range in meters.
    #[serde(rename = "explicit")]
    Explicit { min_elevation_m: f64, max_elevation_m: f64 },
}

/// Specification for a live elevation lines group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElevationSpec {
    /// Source ground/section profile node id.
    pub profile_id: NodeId,
    /// Obstacle node ids (cut stones, visible stones, soil layers) that clip the lines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstacle_ids: Vec<NodeId>,
    /// Datum elevation corresponding to world origin (in meters, e.g. 152.0).
    #[serde(default)]
    pub datum_m: f64,
    /// Minor line interval in meters (default: 0.1 m = 10 cm).
    #[serde(default = "default_minor_interval")]
    pub minor_interval_m: f64,
    /// Major line interval in meters (default: 0.5 m = 50 cm).
    #[serde(default = "default_major_interval")]
    pub major_interval_m: f64,
    /// Minor line stroke weight in points (default: 0.2 pt).
    #[serde(default = "default_minor_weight")]
    pub minor_weight_pt: f64,
    /// Major line stroke weight in points (default: 0.5 pt).
    #[serde(default = "default_major_weight")]
    pub major_weight_pt: f64,
    /// Label formatting (default: MetersSuffix e.g. "152.5m").
    #[serde(default)]
    pub label_format: ElevationLabelFormat,
    /// Label placement (default: LeftEnd).
    #[serde(default)]
    pub label_placement: ElevationLabelPlacement,
    /// Label font size in points (default: 8.0 pt).
    #[serde(default = "default_label_font_size")]
    pub label_font_size_pt: f64,
    /// Elevation vertical range mode.
    #[serde(default)]
    pub range_mode: ElevationRangeMode,
    /// Recompute automatically when profile or obstacles change.
    #[serde(default = "default_auto_update")]
    pub auto_update: bool,
    /// When true, elevation lines are placed below the section profile (underneath ground) rather than in the open void above.
    #[serde(default)]
    pub fill_below_profile: bool,
    /// When true, draws a red datum tick (┣) next to each major elevation label.
    #[serde(default)]
    pub red_datum_tick: bool,
}

fn default_minor_interval() -> f64 {
    DEFAULT_MINOR_INTERVAL_M
}

fn default_major_interval() -> f64 {
    DEFAULT_MAJOR_INTERVAL_M
}

fn default_minor_weight() -> f64 {
    DEFAULT_MINOR_WEIGHT_PT
}

fn default_major_weight() -> f64 {
    DEFAULT_MAJOR_WEIGHT_PT
}

fn default_label_font_size() -> f64 {
    DEFAULT_LABEL_FONT_SIZE_PT
}

fn default_auto_update() -> bool {
    true
}
