//! Hatching models (Trowel, design 2.1 §4, §7, §14): direction, paper spacing,
//! stroke weight, and user-flipped state.
//!
//! Alternating hatch (`HatchDirection::Alternate`) uses dual-graph 2-coloring
//! across adjacent planar faces, flagging odd-cycle conflicts with a warning badge.
//! Paper spacing defaults to 1.0 mm (72.0 / 25.4 pt), and stroke weight defaults to 0.1 pt.

use serde::{Deserialize, Serialize};

use crate::legend::MM;

/// Default hatch line spacing on paper: 1.0 mm (approx 2.8346 pt).
pub const DEFAULT_HATCH_SPACING_MM: f64 = 1.0;

/// Default hatch stroke weight on paper: 0.1 pt.
pub const DEFAULT_HATCH_WEIGHT_PT: f64 = 0.1;

/// Default cut face border stroke weight: 0.8 pt.
pub const DEFAULT_CUT_BORDER_PT: f64 = 0.8;

/// Hatch direction modes (design 2.1 §4, §7, §14(e)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HatchDirection {
    /// 45° hatching lines (╱).
    #[serde(rename = "45", alias = "deg45")]
    Deg45,
    /// 135° hatching lines (╲).
    #[serde(rename = "135", alias = "deg135")]
    Deg135,
    /// Automatically alternating hatching across adjacent planar faces (45° vs 135°).
    #[serde(rename = "alternate", alias = "alt")]
    Alternate,
    /// No hatching.
    #[default]
    #[serde(rename = "none")]
    None,
}

impl HatchDirection {
    /// Returns the angle in radians for fixed directions, or None for None / Alternate.
    pub fn angle_rad(self) -> Option<f64> {
        match self {
            HatchDirection::Deg45 => Some(std::f64::consts::FRAC_PI_4),
            HatchDirection::Deg135 => Some(3.0 * std::f64::consts::FRAC_PI_4),
            HatchDirection::Alternate | HatchDirection::None => None,
        }
    }

    /// Inverts direction between 45° and 135°.
    pub fn inverted(self) -> Self {
        match self {
            HatchDirection::Deg45 => HatchDirection::Deg135,
            HatchDirection::Deg135 => HatchDirection::Deg45,
            other => other,
        }
    }

    /// User-facing label in Korean.
    pub fn label_ko(self) -> &'static str {
        match self {
            HatchDirection::Deg45 => "45°",
            HatchDirection::Deg135 => "135°",
            HatchDirection::Alternate => "번갈아",
            HatchDirection::None => "없음",
        }
    }
}

/// The style specification for hatching a closed planar face.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HatchStyle {
    /// Hatch direction mode.
    #[serde(default)]
    pub direction: HatchDirection,
    /// Hatch line spacing in paper millimeters (default: 1.0 mm).
    #[serde(default = "default_spacing_mm")]
    pub spacing_mm: f64,
    /// Line stroke weight in points (default: 0.1 pt).
    #[serde(default = "default_weight_pt")]
    pub weight_pt: f64,
    /// User manual flip toggle (inverts 45° ↔ 135°).
    #[serde(default)]
    pub flipped: bool,
    /// Resolved base direction from 2-coloring across adjacent faces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_direction: Option<HatchDirection>,
}

impl PartialEq for HatchStyle {
    fn eq(&self, other: &Self) -> bool {
        self.direction == other.direction
            && self.spacing_mm == other.spacing_mm
            && self.weight_pt == other.weight_pt
            && self.flipped == other.flipped
    }
}

fn default_spacing_mm() -> f64 {
    DEFAULT_HATCH_SPACING_MM
}

fn default_weight_pt() -> f64 {
    DEFAULT_HATCH_WEIGHT_PT
}

impl Default for HatchStyle {
    fn default() -> Self {
        Self {
            direction: HatchDirection::None,
            spacing_mm: DEFAULT_HATCH_SPACING_MM,
            weight_pt: DEFAULT_HATCH_WEIGHT_PT,
            flipped: false,
            resolved_direction: None,
        }
    }
}

impl HatchStyle {
    /// Creates a new HatchStyle with given direction and defaults.
    pub fn new(direction: HatchDirection) -> Self {
        Self {
            direction,
            spacing_mm: DEFAULT_HATCH_SPACING_MM,
            weight_pt: DEFAULT_HATCH_WEIGHT_PT,
            flipped: false,
            resolved_direction: None,
        }
    }

    /// Returns true if this style actually generates visible hatch lines.
    pub fn is_hatched(&self) -> bool {
        self.direction != HatchDirection::None && self.spacing_mm > 0.0 && self.weight_pt > 0.0
    }

    /// Converts spacing from paper millimeters to document points.
    pub fn spacing_pt(&self) -> f64 {
        self.spacing_mm * MM
    }

    /// Returns the effective direction for single faces (defaulting Alternate to Deg45 unless flipped).
    pub fn effective_direction(&self) -> HatchDirection {
        match self.direction {
            HatchDirection::Deg45 => {
                if self.flipped {
                    HatchDirection::Deg135
                } else {
                    HatchDirection::Deg45
                }
            }
            HatchDirection::Deg135 => {
                if self.flipped {
                    HatchDirection::Deg45
                } else {
                    HatchDirection::Deg135
                }
            }
            HatchDirection::Alternate => {
                let base = self.resolved_direction.unwrap_or(HatchDirection::Deg45);
                if self.flipped {
                    base.inverted()
                } else {
                    base
                }
            }
            HatchDirection::None => HatchDirection::None,
        }
    }

    /// Creates the standard "잘린 면" (Cut Face) hatching style (Alternate, 1.0 mm, 0.1 pt).
    pub fn cut_face() -> Self {
        Self {
            direction: HatchDirection::Alternate,
            spacing_mm: DEFAULT_HATCH_SPACING_MM,
            weight_pt: DEFAULT_HATCH_WEIGHT_PT,
            flipped: false,
            resolved_direction: None,
        }
    }
}
