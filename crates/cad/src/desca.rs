//! desca 2D Section DXF convention support (Trowel design 2.1 §4, §9, §14).

use std::path::Path;
use vectorcraft_doc::{Legend, LegendGroup, LegendSample, LineType};

/// Known desca layer identifiers.
pub const SECTION_PROFILE: &str = "SECTION_PROFILE";
pub const LEVEL_10CM: &str = "LEVEL_10CM";
pub const LEVEL_50CM: &str = "LEVEL_50CM";
pub const LEVEL_1M: &str = "LEVEL_1M";
pub const LEVEL_TEXT: &str = "LEVEL_TEXT";
pub const SECTION_IMAGE: &str = "SECTION_IMAGE";
pub const SECTION_AXIS: &str = "SECTION_AXIS";
pub const SECTION_TITLE: &str = "SECTION_TITLE";
pub const SECTION_LINE_PLAN: &str = "SECTION_LINE_PLAN";

/// Semantic remapping result for a desca layer.
#[derive(Clone, Debug, PartialEq)]
pub struct DescaLayerMapping {
    pub target_layer: &'static str,
    pub legend: Option<Legend>,
    pub hidden: bool,
    pub locked: bool,
    pub printable: bool,
    pub template: bool,
}

/// Reclassifies a CAD layer name based on desca conventions (§4, §9, §14).
pub fn reclassify_layer(layer_name: &str, is_closed: bool) -> Option<DescaLayerMapping> {
    let upper = layer_name.trim().to_uppercase();
    match upper.as_str() {
        SECTION_PROFILE if is_closed => Some(DescaLayerMapping {
            target_layer: "잘린 돌·유물·기와",
            legend: Some(Legend {
                weight: Some(0.8),
                line: LineType::Solid,
                favourite: true,
                sample: LegendSample::Area,
                strata: None,
                group: Some(LegendGroup::Section),
            }),
            hidden: false,
            locked: false,
            printable: true,
            template: false,
        }),
        SECTION_PROFILE => Some(DescaLayerMapping {
            target_layer: "단면선 (잘린 땅)",
            legend: Some(Legend {
                weight: Some(0.8),
                line: LineType::Solid,
                favourite: true,
                sample: LegendSample::Line,
                strata: None,
                group: Some(LegendGroup::Section),
            }),
            hidden: false,
            locked: false,
            printable: true,
            template: false,
        }),
        LEVEL_10CM | LEVEL_50CM | LEVEL_1M => Some(DescaLayerMapping {
            target_layer: "desca 원본",
            legend: Some(Legend {
                weight: None,
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Line,
                strata: None,
                group: Some(LegendGroup::Section),
            }),
            hidden: true,
            locked: false,
            printable: false,
            template: true,
        }),
        s if s.starts_with("LEVEL_") && s != LEVEL_TEXT => Some(DescaLayerMapping {
            target_layer: "desca 원본",
            legend: Some(Legend {
                weight: None,
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Line,
                strata: None,
                group: Some(LegendGroup::Section),
            }),
            hidden: true,
            locked: false,
            printable: false,
            template: true,
        }),
        LEVEL_TEXT => Some(DescaLayerMapping {
            target_layer: "해발 숫자",
            legend: Some(Legend {
                weight: None,
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Mark("152.5m".to_string()),
                strata: None,
                group: Some(LegendGroup::Section),
            }),
            hidden: false,
            locked: false,
            printable: true,
            template: false,
        }),
        SECTION_IMAGE => Some(DescaLayerMapping {
            target_layer: "밑그림",
            legend: Some(Legend {
                weight: None,
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Area,
                strata: None,
                group: Some(LegendGroup::Section),
            }),
            hidden: false,
            locked: true,
            printable: false,
            template: true,
        }),
        SECTION_AXIS => Some(DescaLayerMapping {
            target_layer: "화면 눈금",
            legend: Some(Legend {
                weight: None,
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Line,
                strata: None,
                group: Some(LegendGroup::Section),
            }),
            hidden: false,
            locked: false,
            printable: true,
            template: false,
        }),
        _ => None,
    }
}

/// Normalizes elevation labels to the standard "{value}m" format (§14(c)).
pub fn normalize_elevation_text(text: &str) -> String {
    let trimmed = text.trim();
    let cleaned = trimmed
        .strip_prefix("EL.")
        .or_else(|| trimmed.strip_prefix("el."))
        .or_else(|| trimmed.strip_prefix("EL "))
        .or_else(|| trimmed.strip_prefix("el "))
        .unwrap_or(trimmed)
        .trim();

    if let Some(num_str) = cleaned.strip_suffix(['m', 'M']) {
        let candidate = num_str.trim();
        if candidate.parse::<f64>().is_ok() {
            return format!("{candidate}m");
        }
    }

    if cleaned.parse::<f64>().is_ok() {
        return format!("{cleaned}m");
    }

    trimmed.to_string()
}

/// Parses strings with unit suffixes ("152.5m", "10cm", "1.0 m") into meters.
pub fn parse_meter_string(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(v) = s.strip_suffix("cm").or_else(|| s.strip_suffix("CM")) {
        return v.trim().parse::<f64>().ok().map(|cm| cm / 100.0);
    }
    if let Some(v) = s.strip_suffix('m').or_else(|| s.strip_suffix('M')) {
        return v.trim().parse::<f64>().ok();
    }
    s.parse::<f64>().ok()
}

/// Parses SECTION_TITLE text into section name and scale ratio.
pub fn parse_section_title(text: &str) -> (String, Option<f64>) {
    let t = text.trim();
    if let Some(idx) = t.rfind("1:") {
        let name = t[..idx].trim().trim_end_matches(['-', '/', ',', ':']).trim();
        let denom = t[idx + 2..].trim().parse::<f64>().ok();
        (name.to_string(), denom)
    } else {
        (t.to_string(), None)
    }
}

/// Reads companion `.sections.json` file if it exists next to the DXF.
pub fn read_companion_sections_json(dxf_path: &Path) -> Option<String> {
    let parent = dxf_path.parent()?;
    let stem = dxf_path.file_stem()?.to_str()?;
    let companion1 = parent.join(format!("{stem}.sections.json"));
    let companion2 = parent.join(".sections.json");

    let target_path = if companion1.exists() {
        companion1
    } else if companion2.exists() {
        companion2
    } else {
        return None;
    };

    let content = std::fs::read_to_string(target_path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    val.get("sections")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reclassify_section_profile() {
        let open_mapping = reclassify_layer("SECTION_PROFILE", false).unwrap();
        assert_eq!(open_mapping.target_layer, "단면선 (잘린 땅)");
        assert_eq!(open_mapping.legend.as_ref().unwrap().weight, Some(0.8));
        assert_eq!(open_mapping.legend.as_ref().unwrap().sample, LegendSample::Line);
        assert!(!open_mapping.hidden);

        let closed_mapping = reclassify_layer("SECTION_PROFILE", true).unwrap();
        assert_eq!(closed_mapping.target_layer, "잘린 돌·유물·기와");
        assert_eq!(closed_mapping.legend.as_ref().unwrap().weight, Some(0.8));
        assert_eq!(closed_mapping.legend.as_ref().unwrap().sample, LegendSample::Area);
        assert!(!closed_mapping.hidden);
    }

    #[test]
    fn test_reclassify_level_lines() {
        for name in &["LEVEL_10CM", "LEVEL_50CM", "LEVEL_1M", "LEVEL_5M"] {
            let m = reclassify_layer(name, false).unwrap();
            assert_eq!(m.target_layer, "desca 원본");
            assert!(m.hidden);
            assert!(m.template);
            assert!(!m.printable);
        }

        let txt = reclassify_layer("LEVEL_TEXT", false).unwrap();
        assert_eq!(txt.target_layer, "해발 숫자");
        assert!(!txt.hidden);
        assert_eq!(txt.legend.as_ref().unwrap().sample, LegendSample::Mark("152.5m".into()));
    }

    #[test]
    fn test_reclassify_image_and_axis() {
        let img = reclassify_layer("SECTION_IMAGE", false).unwrap();
        assert_eq!(img.target_layer, "밑그림");
        assert!(img.locked);
        assert!(img.template);

        let axis = reclassify_layer("SECTION_AXIS", false).unwrap();
        assert_eq!(axis.target_layer, "화면 눈금");
    }

    #[test]
    fn test_normalize_elevation_text() {
        assert_eq!(normalize_elevation_text("EL. 152.5"), "152.5m");
        assert_eq!(normalize_elevation_text("el. 152.5"), "152.5m");
        assert_eq!(normalize_elevation_text("EL 152.5"), "152.5m");
        assert_eq!(normalize_elevation_text("152.5"), "152.5m");
        assert_eq!(normalize_elevation_text("152.5m"), "152.5m");
        assert_eq!(normalize_elevation_text("152.5M"), "152.5m");
        assert_eq!(normalize_elevation_text("78.0"), "78.0m");
    }

    #[test]
    fn test_parse_meter_string() {
        assert_eq!(parse_meter_string("152.5m"), Some(152.5));
        assert_eq!(parse_meter_string("152.5 m"), Some(152.5));
        assert_eq!(parse_meter_string("10cm"), Some(0.1));
        assert_eq!(parse_meter_string("10 cm"), Some(0.1));
        assert_eq!(parse_meter_string("50cm"), Some(0.5));
        assert_eq!(parse_meter_string("1m"), Some(1.0));
        assert_eq!(parse_meter_string("152.5"), Some(152.5));
    }

    #[test]
    fn test_parse_section_title() {
        let (name, scale) = parse_section_title("A-A' 1:20");
        assert_eq!(name, "A-A'");
        assert_eq!(scale, Some(20.0));

        let (name2, scale2) = parse_section_title("2호 주거지 북벽 1:20");
        assert_eq!(name2, "2호 주거지 북벽");
        assert_eq!(scale2, Some(20.0));

        let (name3, scale3) = parse_section_title("트렌치 단면");
        assert_eq!(name3, "트렌치 단면");
        assert_eq!(scale3, None);
    }
}
