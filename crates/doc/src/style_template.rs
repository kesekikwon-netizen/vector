//! Standalone style template (.trowel-style) and style diff/merge engine (Trowel design 2.1 §8, §10, J018).

use serde::{Deserialize, Serialize};

use crate::Document;
use crate::hatch::{DEFAULT_HATCH_SPACING_MM, DEFAULT_HATCH_WEIGHT_PT, HatchDirection};
use crate::legend::{LegendGroup, LegendSample, LineType, SAME};
use crate::stratigraphy::StratigraphyPatternKind;

/// File format identifier for standalone Trowel style templates.
pub const TROWEL_STYLE_FORMAT: &str = "trowel-style";
/// Current schema version.
pub const TROWEL_STYLE_VERSION: u32 = 1;
/// Standard file extension for Trowel style templates.
pub const TROWEL_STYLE_EXTENSION: &str = "trowel-style";

/// Complete standalone style template (.trowel-style, JSON format).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrowelStyleTemplate {
    /// Format identifier: must be "trowel-style".
    pub format: String,
    /// Schema version: 1.
    pub version: u32,
    /// Human-readable template name.
    pub name: String,
    /// Author / contributor name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Organization / archaeological research institute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    /// Description or scope notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Stroke weight tiers (pt).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stroke_weights: Vec<StrokeWeightTier>,
    /// Hatch presets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hatch_presets: Vec<HatchPreset>,
    /// Munsell / soil color palette definitions (clean-room: notation text values, no proprietary chips).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soil_palettes: Vec<SoilPaletteEntry>,
    /// Layer classification rules / legend row definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legend_rows: Vec<TemplateLegendRow>,
    /// Paper typography definitions (font sizes in pt).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typography: Option<TypographyStyle>,
    /// Elevation line generation rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_rules: Option<ElevationStyleRules>,
    /// Institutional title block field definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_block: Option<TitleBlockTemplate>,
}

impl Default for TrowelStyleTemplate {
    fn default() -> Self {
        Self {
            format: TROWEL_STYLE_FORMAT.to_string(),
            version: TROWEL_STYLE_VERSION,
            name: "기본 고고학 양식".to_string(),
            author: None,
            organization: None,
            description: Some("국립문화유산연구원 및 한국고고학 발굴조사 표준 도면 양식".to_string()),
            stroke_weights: default_stroke_weights(),
            hatch_presets: default_hatch_presets(),
            soil_palettes: default_soil_palettes(),
            legend_rows: default_template_legend_rows(),
            typography: Some(TypographyStyle::default()),
            elevation_rules: Some(ElevationStyleRules::default()),
            title_block: Some(TitleBlockTemplate::default()),
        }
    }
}

/// A stroke weight tier (in paper pt).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokeWeightTier {
    pub label: String,
    pub weight_pt: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

pub fn default_stroke_weights() -> Vec<StrokeWeightTier> {
    vec![
        StrokeWeightTier {
            label: "0.8 pt".to_string(),
            weight_pt: 0.8,
            description: Some("유구 윤곽선 / 굴광선".to_string()),
            role: Some("primary".to_string()),
        },
        StrokeWeightTier {
            label: "0.5 pt".to_string(),
            weight_pt: 0.5,
            description: Some("부속 시설 / 2차 굴광선 / 주혈".to_string()),
            role: Some("secondary".to_string()),
        },
        StrokeWeightTier {
            label: "0.2 pt".to_string(),
            weight_pt: 0.2,
            description: Some("바닥선 / 토층선 / 단면선".to_string()),
            role: Some("tertiary".to_string()),
        },
        StrokeWeightTier {
            label: "0.1 pt".to_string(),
            weight_pt: 0.1,
            description: Some("빗금 / 해칭선".to_string()),
            role: Some("hatch".to_string()),
        },
    ]
}

/// A hatch pattern preset (§7, §8, §14(e,f)).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HatchPreset {
    pub name: String,
    pub direction: HatchDirection,
    #[serde(default = "default_spacing_mm")]
    pub spacing_mm: f64,
    #[serde(default = "default_weight_pt")]
    pub weight_pt: f64,
    #[serde(default)]
    pub double_hatch: bool,
    #[serde(default)]
    pub align_to_paper: bool,
    #[serde(default)]
    pub inset_margin_mm: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_spacing_mm() -> f64 {
    DEFAULT_HATCH_SPACING_MM
}
fn default_weight_pt() -> f64 {
    DEFAULT_HATCH_WEIGHT_PT
}

pub fn default_hatch_presets() -> Vec<HatchPreset> {
    vec![
        HatchPreset {
            name: "수혈 45°".to_string(),
            direction: HatchDirection::Deg45,
            spacing_mm: 1.0,
            weight_pt: 0.1,
            double_hatch: false,
            align_to_paper: true,
            inset_margin_mm: 0.0,
            description: Some("수혈유구 단면 빗금 (45도, 1.0mm 간격, 0.1pt)".to_string()),
        },
        HatchPreset {
            name: "분묘 135°".to_string(),
            direction: HatchDirection::Deg135,
            spacing_mm: 1.0,
            weight_pt: 0.1,
            double_hatch: false,
            align_to_paper: true,
            inset_margin_mm: 0.0,
            description: Some("분묘/석관묘 단면 빗금 (135도, 1.0mm 간격, 0.1pt)".to_string()),
        },
        HatchPreset {
            name: "번갈아 빗금".to_string(),
            direction: HatchDirection::Alternate,
            spacing_mm: 1.0,
            weight_pt: 0.1,
            double_hatch: false,
            align_to_paper: true,
            inset_margin_mm: 0.0,
            description: Some("인접 구획 자동 교차 빗금 (45°/135°)".to_string()),
        },
    ]
}

/// Clean-room Munsell soil palette entry (§4-2, §8).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoilPaletteEntry {
    pub name: String,
    pub munsell: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inclusions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<StratigraphyPatternKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub srgb_hex: Option<String>,
}

pub fn default_soil_palettes() -> Vec<SoilPaletteEntry> {
    vec![
        SoilPaletteEntry {
            name: "표토층".to_string(),
            munsell: "10YR 2/2 흑갈색".to_string(),
            texture: Some("사양토".to_string()),
            inclusions: Some("초본류 뿌리 다량 함유".to_string()),
            pattern: Some(StratigraphyPatternKind::BurntSoil),
            srgb_hex: Some("#3a312a".to_string()),
        },
        SoilPaletteEntry {
            name: "유구 퇴적토".to_string(),
            munsell: "10YR 3/4 암갈색".to_string(),
            texture: Some("점질양토".to_string()),
            inclusions: Some("목탄립 미량, 소토 입자 포함".to_string()),
            pattern: Some(StratigraphyPatternKind::Charcoal),
            srgb_hex: Some("#544332".to_string()),
        },
        SoilPaletteEntry {
            name: "생토층".to_string(),
            munsell: "7.5YR 4/4 갈색".to_string(),
            texture: Some("점토".to_string()),
            inclusions: Some("풍화암반 입자 함유".to_string()),
            pattern: Some(StratigraphyPatternKind::Ash),
            srgb_hex: Some("#78573a".to_string()),
        },
        SoilPaletteEntry {
            name: "목탄 집중층".to_string(),
            munsell: "10YR 2/1 흑색".to_string(),
            texture: Some("사질양토".to_string()),
            inclusions: Some("탄화목괴 다량 집중".to_string()),
            pattern: Some(StratigraphyPatternKind::Charcoal),
            srgb_hex: Some("#2b2826".to_string()),
        },
    ]
}

/// Template legend row corresponding to a layer classification rule (§7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TemplateLegendRow {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<LegendGroup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight_pt: Option<f64>,
    #[serde(default)]
    pub line_type: LineType,
    #[serde(default)]
    pub sample: LegendSample,
    #[serde(default)]
    pub favourite: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strata_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hatch_preset_ref: Option<String>,
}

pub fn default_template_legend_rows() -> Vec<TemplateLegendRow> {
    vec![
        TemplateLegendRow {
            name: "조사구역 경계".to_string(),
            group: Some(LegendGroup::Plan),
            weight_pt: Some(1.0),
            line_type: LineType::DashDot,
            sample: LegendSample::Area,
            favourite: false,
            strata_key: Some("survey_area".to_string()),
            description: Some("발굴조사 구역 경계선".to_string()),
            hatch_preset_ref: None,
        },
        TemplateLegendRow {
            name: "시굴격자".to_string(),
            group: Some(LegendGroup::Plan),
            weight_pt: Some(0.5),
            line_type: LineType::Solid,
            sample: LegendSample::Area,
            favourite: false,
            strata_key: Some("trial_trench".to_string()),
            description: Some("피트 및 트렌치 그리드망".to_string()),
            hatch_preset_ref: None,
        },
        TemplateLegendRow {
            name: "굴광선 (유구 면)".to_string(),
            group: Some(LegendGroup::Plan),
            weight_pt: Some(0.8),
            line_type: LineType::Solid,
            sample: LegendSample::Area,
            favourite: true,
            strata_key: Some("feature_poly".to_string()),
            description: Some("유구 굴광 윤곽선".to_string()),
            hatch_preset_ref: Some("수혈 45°".to_string()),
        },
        TemplateLegendRow {
            name: "바닥선".to_string(),
            group: Some(LegendGroup::Plan),
            weight_pt: Some(0.2),
            line_type: LineType::Solid,
            sample: LegendSample::Line,
            favourite: true,
            strata_key: Some("floor_line".to_string()),
            description: Some("유구 바닥면 및 단면 바닥선".to_string()),
            hatch_preset_ref: None,
        },
        TemplateLegendRow {
            name: "단면 위치선".to_string(),
            group: Some(LegendGroup::Plan),
            weight_pt: Some(0.2),
            line_type: LineType::DashDot,
            sample: LegendSample::Line,
            favourite: true,
            strata_key: Some("section_line".to_string()),
            description: Some("토층 단면 위치선 A-A'".to_string()),
            hatch_preset_ref: None,
        },
        TemplateLegendRow {
            name: "유물 출토 위치".to_string(),
            group: Some(LegendGroup::Plan),
            weight_pt: Some(0.2),
            line_type: LineType::Solid,
            sample: LegendSample::Mark("③".to_string()),
            favourite: false,
            strata_key: Some("artifact_point".to_string()),
            description: Some("출토유물 실측 위치 및 번호".to_string()),
            hatch_preset_ref: None,
        },
        TemplateLegendRow {
            name: "표고점".to_string(),
            group: Some(LegendGroup::Plan),
            weight_pt: None,
            line_type: LineType::Solid,
            sample: LegendSample::Mark("·152".to_string()),
            favourite: false,
            strata_key: Some("elevation_point".to_string()),
            description: Some("표고 실측점 (해발 고도)".to_string()),
            hatch_preset_ref: None,
        },
    ]
}

/// Paper typography styles in points (§7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypographyStyle {
    pub caption_pt: f64,
    pub feature_name_pt: f64,
    pub legend_and_soil_pt: f64,
    pub elevation_pt: f64,
    pub coordinates_pt: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
}

impl Default for TypographyStyle {
    fn default() -> Self {
        Self { caption_pt: 10.0, feature_name_pt: 9.0, legend_and_soil_pt: 8.0, elevation_pt: 7.0, coordinates_pt: 7.0, font_family: None }
    }
}

/// Elevation line generation rules (§4-1, §14(b,c,d)).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElevationStyleRules {
    pub major_interval_cm: f64,
    pub major_weight_pt: f64,
    pub minor_interval_cm: f64,
    pub minor_weight_pt: f64,
    pub label_format: String,
    pub label_placement: String,
    pub clip_voids_only: bool,
}

impl Default for ElevationStyleRules {
    fn default() -> Self {
        Self {
            major_interval_cm: 50.0,
            major_weight_pt: 0.5,
            minor_interval_cm: 10.0,
            minor_weight_pt: 0.2,
            label_format: "{elevation:.1f}m".to_string(),
            label_placement: "left_end".to_string(),
            clip_voids_only: true,
        }
    }
}

/// Submission title block fields (§6, §14).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TitleBlockTemplate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drawing_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crs: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_datum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
}

// ---------- Diff & Merge Types ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleCategory {
    LegendRow,
    StrokeWeight,
    HatchPreset,
    SoilPalette,
    Typography,
    ElevationRules,
    TitleBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffStatus {
    Identical,
    Added,
    Modified,
    Missing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeAction {
    KeepExisting,
    Overwrite,
    ImportAsNewCopy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyDiff {
    pub property_name: String,
    pub current_value: String,
    pub incoming_value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemDiff {
    pub category: StyleCategory,
    pub key: String,
    pub label: String,
    pub status: DiffStatus,
    pub property_diffs: Vec<PropertyDiff>,
    pub recommended_action: MergeAction,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StyleDiff {
    pub items: Vec<ItemDiff>,
    pub identical_count: usize,
    pub added_count: usize,
    pub modified_count: usize,
    pub missing_count: usize,
}

impl StyleDiff {
    pub fn summary_ko(&self) -> String {
        format!("새 항목 {}개 · 바뀐 항목 {}개 · 같은 항목 {}개", self.added_count, self.modified_count, self.identical_count)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleMergeResolution {
    pub category: StyleCategory,
    pub key: String,
    pub action: MergeAction,
}

/// Compares incoming template against active document state.
pub fn diff_template(doc: &Document, template: &TrowelStyleTemplate) -> StyleDiff {
    let mut items = Vec::new();
    let mut identical_count = 0;
    let mut added_count = 0;
    let mut modified_count = 0;
    let mut missing_count = 0;

    // 1. Legend Rows
    for row in &template.legend_rows {
        // Find matching layer in doc by name or strata key
        let existing = doc.layers.iter().find(|l| {
            l.display_name().trim() == row.name.trim()
                || (row.strata_key.is_some() && l.legend().and_then(|leg| leg.strata.as_deref()) == row.strata_key.as_deref())
        });

        if let Some(layer) = existing {
            let leg = layer.legend();
            let mut prop_diffs = Vec::new();

            // Compare weight
            let cur_w = leg.and_then(|l| l.weight);
            let inc_w = row.weight_pt;
            match (cur_w, inc_w) {
                (Some(cw), Some(iw)) if (cw - iw).abs() > SAME => {
                    prop_diffs.push(PropertyDiff {
                        property_name: "선 굵기".to_string(),
                        current_value: format!("{cw:.2} pt"),
                        incoming_value: format!("{iw:.2} pt"),
                    });
                }
                (None, Some(iw)) => {
                    prop_diffs.push(PropertyDiff {
                        property_name: "선 굵기".to_string(),
                        current_value: "없음".to_string(),
                        incoming_value: format!("{iw:.2} pt"),
                    });
                }
                (Some(cw), None) => {
                    prop_diffs.push(PropertyDiff {
                        property_name: "선 굵기".to_string(),
                        current_value: format!("{cw:.2} pt"),
                        incoming_value: "없음".to_string(),
                    });
                }
                _ => {}
            }

            // Compare line type
            let cur_lt = leg.map(|l| &l.line).unwrap_or(&LineType::Solid);
            if !cur_lt.same(&row.line_type) {
                prop_diffs.push(PropertyDiff {
                    property_name: "선 종류".to_string(),
                    current_value: format!("{cur_lt:?}"),
                    incoming_value: format!("{:?}", row.line_type),
                });
            }

            if prop_diffs.is_empty() {
                identical_count += 1;
                items.push(ItemDiff {
                    category: StyleCategory::LegendRow,
                    key: row.name.clone(),
                    label: row.name.clone(),
                    status: DiffStatus::Identical,
                    property_diffs: vec![],
                    recommended_action: MergeAction::KeepExisting,
                });
            } else {
                modified_count += 1;
                items.push(ItemDiff {
                    category: StyleCategory::LegendRow,
                    key: row.name.clone(),
                    label: row.name.clone(),
                    status: DiffStatus::Modified,
                    property_diffs: prop_diffs,
                    recommended_action: MergeAction::Overwrite,
                });
            }
        } else {
            added_count += 1;
            items.push(ItemDiff {
                category: StyleCategory::LegendRow,
                key: row.name.clone(),
                label: row.name.clone(),
                status: DiffStatus::Added,
                property_diffs: vec![PropertyDiff {
                    property_name: "새 레이어".to_string(),
                    current_value: "미존재".to_string(),
                    incoming_value: format!("{:?}", row.weight_pt),
                }],
                recommended_action: MergeAction::Overwrite,
            });
        }
    }

    // Check for doc layers missing in template
    for layer in &doc.layers {
        if layer.legend().is_some() {
            let layer_name = layer.display_name();
            let name = layer_name.trim();
            if !template.legend_rows.iter().any(|r| r.name.trim() == name) {
                missing_count += 1;
                items.push(ItemDiff {
                    category: StyleCategory::LegendRow,
                    key: name.to_string(),
                    label: name.to_string(),
                    status: DiffStatus::Missing,
                    property_diffs: vec![],
                    recommended_action: MergeAction::KeepExisting,
                });
            }
        }
    }

    StyleDiff { items, identical_count, added_count, modified_count, missing_count }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_roundtrip_equality() {
        let tmpl = TrowelStyleTemplate::default();
        let json_str = serde_json::to_string_pretty(&tmpl).unwrap();
        let back: TrowelStyleTemplate = serde_json::from_str(&json_str).unwrap();
        assert_eq!(tmpl, back);
        assert_eq!(back.format, "trowel-style");
        assert_eq!(back.version, 1);
    }

    #[test]
    fn test_diff_engine_identifies_added_and_modified() {
        let mut doc = Document::new(500.0, 500.0);
        // Add a layer "굴광선 (유구 면)" with weight 0.8
        let id = doc.alloc_id();
        let mut layer = crate::Node::layer(id, "굴광선 (유구 면)", crate::LayerColor::Preset(0));
        if let crate::NodeKind::Layer { legend, .. } = &mut layer.kind {
            *legend = Some(Box::new(crate::Legend {
                weight: Some(0.8),
                line: LineType::Solid,
                favourite: true,
                sample: LegendSample::Area,
                strata: Some("feature_poly".to_string()),
                group: Some(LegendGroup::Plan),
            }));
        }
        doc.layers.push(std::sync::Arc::new(layer));

        let mut tmpl = TrowelStyleTemplate::default();
        // Modify 굴광선 weight to 1.0 pt
        if let Some(r) = tmpl.legend_rows.iter_mut().find(|r| r.name == "굴광선 (유구 면)") {
            r.weight_pt = Some(1.0);
        }

        let diff = diff_template(&doc, &tmpl);
        assert_eq!(diff.modified_count, 1);
        assert!(diff.added_count >= 1);
    }
}
