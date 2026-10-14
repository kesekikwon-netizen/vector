//! Stratigraphy data model: strata layers, soil description tables, circled layer numbers (Trowel design 2.1 §4-2, J010).

use serde::{Deserialize, Serialize};
use vectorcraft_geom::Point;

use crate::NodeId;

/// Numbering style for archaeological stratigraphic layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NumberingStyle {
    #[default]
    Circled, // ①, ②, ③ ... ㊿
    Roman,   // Ⅰ, Ⅱ, Ⅲ ... Ⅹ
    Arabic,  // 1, 2, 3 ...
}

pub type LayerNumberingStyle = NumberingStyle;

impl NumberingStyle {
    pub fn format(self, n: u32) -> String {
        match self {
            NumberingStyle::Circled => match n {
                1..=20 => char::from_u32(0x2460 + n - 1).map(|c| c.to_string()).unwrap_or_else(|| n.to_string()),
                21..=35 => char::from_u32(0x3251 + n - 21).map(|c| c.to_string()).unwrap_or_else(|| n.to_string()),
                36..=50 => char::from_u32(0x32B1 + n - 36).map(|c| c.to_string()).unwrap_or_else(|| n.to_string()),
                _ => format!("({})", n),
            },
            NumberingStyle::Roman => match n {
                1 => "Ⅰ".into(),
                2 => "Ⅱ".into(),
                3 => "Ⅲ".into(),
                4 => "Ⅳ".into(),
                5 => "Ⅴ".into(),
                6 => "Ⅵ".into(),
                7 => "Ⅶ".into(),
                8 => "Ⅷ".into(),
                9 => "Ⅸ".into(),
                10 => "Ⅹ".into(),
                _ => format!("{n}"),
            },
            NumberingStyle::Arabic => n.to_string(),
        }
    }
}

/// Detailed soil description record (§4-2).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SoilDescription {
    pub color_name: String, // 토색 (e.g. "암갈색")
    pub munsell: String,    // 먼셀 값 (e.g. "10YR 3/4")
    pub texture: String,    // 토성 (e.g. "점질토")
    pub inclusions: String, // 포함유물 / 혼입물 (e.g. "소토 입자, 목탄 소량")
    pub notes: String,      // 비고 (e.g. "주거지 폐기 뒤 퇴적")
}

/// Pattern kinds for archaeological strata representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StratigraphyPatternKind {
    BurntSoil,   // 소토 무늬 (sburnt)
    Charcoal,    // 목탄 무늬 (schar)
    Ash,         // 재 무늬 (sash)
    Disturbance, // 교란 무늬 (sdist)
}

/// Fill style applied to a stratigraphic bounded region.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StratigraphyFillStyle {
    #[default]
    None,
    Tint {
        color_hex: String,
        opacity: f32,
    },
    Pattern {
        kind: StratigraphyPatternKind,
    },
    Hatch {
        angle_deg: f64,
        spacing_mm: f64,
        weight_pt: f64,
    },
}

/// Single stratigraphic layer row.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StratigraphyLayer {
    pub index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_badge: Option<String>,
    #[serde(default)]
    pub description: SoilDescription,
    #[serde(default)]
    pub fill_style: StratigraphyFillStyle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_node_id: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face_node_id: Option<NodeId>,
}

impl StratigraphyLayer {
    pub fn badge_text(&self, style: NumberingStyle) -> String {
        if let Some(custom) = &self.custom_badge {
            custom.clone()
        } else {
            style.format(self.index)
        }
    }
}

/// On-canvas table render format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TableRenderFormat {
    #[default]
    Grid,
    Inline,
}

/// On-canvas placement configuration for the generated vector table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StratigraphyTablePlacement {
    pub origin: Point,
    pub width_pt: f64,
    #[serde(default = "default_table_font_size")]
    pub font_size_pt: f64,
    #[serde(default)]
    pub format: TableRenderFormat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
}

fn default_table_font_size() -> f64 {
    8.0
}

/// Complete stratigraphic table for a section.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StratigraphyTable {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section_line_id: Option<NodeId>,
    #[serde(default)]
    pub numbering_style: NumberingStyle,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<StratigraphyLayer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<StratigraphyTablePlacement>,
}

impl StratigraphyTable {
    /// Formats the table as TSV for clipboard copy into HWP / Excel.
    pub fn to_tsv(&self) -> String {
        let mut out = String::from("층위\t토색\t먼셀\t토성\t포함유물\t비고\n");
        for layer in &self.layers {
            let badge = layer.badge_text(self.numbering_style);
            let d = &layer.description;
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\n",
                badge, d.color_name, d.munsell, d.texture, d.inclusions, d.notes
            ));
        }
        out
    }
}

fn default_badge_font_size() -> f64 {
    8.0
}

fn default_badge_radius() -> f64 {
    7.0
}

fn default_halo() -> bool {
    true
}

/// Circled layer number badge node specification.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CircledNumberSpec {
    pub number: u32,
    pub style: NumberingStyle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_text: Option<String>,
    pub center: Point,
    #[serde(default = "default_badge_font_size")]
    pub font_size_pt: f64,
    #[serde(default = "default_badge_radius")]
    pub radius_pt: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_to: Option<Point>,
    #[serde(default = "default_halo")]
    pub halo: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numbering_styles() {
        assert_eq!(NumberingStyle::Circled.format(1), "①");
        assert_eq!(NumberingStyle::Circled.format(20), "⑳");
        assert_eq!(NumberingStyle::Circled.format(21), "㉑");
        assert_eq!(NumberingStyle::Circled.format(35), "㉟");
        assert_eq!(NumberingStyle::Circled.format(36), "㊱");
        assert_eq!(NumberingStyle::Circled.format(50), "㊿");
        assert_eq!(NumberingStyle::Circled.format(51), "(51)");

        assert_eq!(NumberingStyle::Roman.format(1), "Ⅰ");
        assert_eq!(NumberingStyle::Roman.format(5), "Ⅴ");
        assert_eq!(NumberingStyle::Roman.format(10), "Ⅹ");
        assert_eq!(NumberingStyle::Roman.format(11), "11");

        assert_eq!(NumberingStyle::Arabic.format(1), "1");
        assert_eq!(NumberingStyle::Arabic.format(42), "42");
    }

    #[test]
    fn test_tsv_export() {
        let mut table = StratigraphyTable {
            id: "sec-aa".into(),
            title: "A–A′ 토층 설명표".into(),
            section_line_id: None,
            numbering_style: NumberingStyle::Circled,
            layers: Vec::new(),
            placement: None,
        };

        table.layers.push(StratigraphyLayer {
            index: 1,
            custom_badge: None,
            description: SoilDescription {
                color_name: "흑갈색".into(),
                munsell: "10YR 2/2".into(),
                texture: "사질점토".into(),
                inclusions: "목탄 소량".into(),
                notes: "표토층".into(),
            },
            fill_style: StratigraphyFillStyle::None,
            badge_node_id: None,
            face_node_id: None,
        });

        table.layers.push(StratigraphyLayer {
            index: 2,
            custom_badge: None,
            description: SoilDescription {
                color_name: "암갈색".into(),
                munsell: "10YR 3/4".into(),
                texture: "점질토".into(),
                inclusions: "소토 입자".into(),
                notes: "주거지 폐기 뒤 퇴적".into(),
            },
            fill_style: StratigraphyFillStyle::None,
            badge_node_id: None,
            face_node_id: None,
        });

        let tsv = table.to_tsv();
        let expected = "층위\t토색\t먼셀\t토성\t포함유물\t비고\n\
①\t흑갈색\t10YR 2/2\t사질점토\t목탄 소량\t표토층\n\
②\t암갈색\t10YR 3/4\t점질토\t소토 입자\t주거지 폐기 뒤 퇴적\n";
        assert_eq!(tsv, expected);
    }
}
