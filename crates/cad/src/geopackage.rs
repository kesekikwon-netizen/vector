//! GeoPackage survey reader for Strata survey data (OGC 12-128r17, Trowel §11, §13, §14, Task J019).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use kurbo::{BezPath, Point};
use serde::{Deserialize, Serialize};
use vectorcraft_doc::hatch::{HatchDirection, HatchStyle};
use vectorcraft_doc::scale::ScaleConfig;
use vectorcraft_doc::{Legend, LegendGroup, LegendSample, LineType};

/// Supported geometry types in GeoPackage feature tables.
#[derive(Clone, Debug, PartialEq)]
pub enum GpkgGeometry {
    Point { x: f64, y: f64, z: Option<f64> },
    LineString { points: Vec<(f64, f64, Option<f64>)> },
    Polygon { rings: Vec<Vec<(f64, f64, Option<f64>)>> },
    MultiPoint { points: Vec<(f64, f64, Option<f64>)> },
    MultiLineString { lines: Vec<Vec<(f64, f64, Option<f64>)>> },
    MultiPolygon { polygons: Vec<Vec<Vec<(f64, f64, Option<f64>)>>> },
}

impl GpkgGeometry {
    /// Compute 2D bounding box (min_x, min_y, max_x, max_y) in world coordinates.
    pub fn bounds_2d(&self) -> Option<[f64; 4]> {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let mut update = |x: f64, y: f64| {
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        };

        match self {
            GpkgGeometry::Point { x, y, .. } => update(*x, *y),
            GpkgGeometry::LineString { points } | GpkgGeometry::MultiPoint { points } => {
                for (x, y, _) in points {
                    update(*x, *y);
                }
            }
            GpkgGeometry::Polygon { rings } => {
                for ring in rings {
                    for (x, y, _) in ring {
                        update(*x, *y);
                    }
                }
            }
            GpkgGeometry::MultiLineString { lines } => {
                for line in lines {
                    for (x, y, _) in line {
                        update(*x, *y);
                    }
                }
            }
            GpkgGeometry::MultiPolygon { polygons } => {
                for poly in polygons {
                    for ring in poly {
                        for (x, y, _) in ring {
                            update(*x, *y);
                        }
                    }
                }
            }
        }

        if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
            Some([min_x, min_y, max_x, max_y])
        } else {
            None
        }
    }

    /// Converts world geometry to a `BezPath` in document points using the given `ScaleConfig`.
    pub fn to_bez_path(&self, scale_config: &ScaleConfig) -> BezPath {
        let mut path = BezPath::new();

        let pt = |x: f64, y: f64| -> Point { scale_config.world_to_canvas(x, y) };

        match self {
            GpkgGeometry::Point { x, y, .. } => {
                let p = pt(*x, *y);
                path.move_to(p);
                path.line_to(Point::new(p.x + 0.1, p.y));
            }
            GpkgGeometry::LineString { points } => {
                if let Some((first, rest)) = points.split_first() {
                    path.move_to(pt(first.0, first.1));
                    for p in rest {
                        path.line_to(pt(p.0, p.1));
                    }
                }
            }
            GpkgGeometry::Polygon { rings } => {
                for ring in rings {
                    if let Some((first, rest)) = ring.split_first() {
                        path.move_to(pt(first.0, first.1));
                        for p in rest {
                            path.line_to(pt(p.0, p.1));
                        }
                        path.close_path();
                    }
                }
            }
            GpkgGeometry::MultiPoint { points } => {
                for (x, y, _) in points {
                    let p = pt(*x, *y);
                    path.move_to(p);
                    path.line_to(Point::new(p.x + 0.1, p.y));
                }
            }
            GpkgGeometry::MultiLineString { lines } => {
                for line in lines {
                    if let Some((first, rest)) = line.split_first() {
                        path.move_to(pt(first.0, first.1));
                        for p in rest {
                            path.line_to(pt(p.0, p.1));
                        }
                    }
                }
            }
            GpkgGeometry::MultiPolygon { polygons } => {
                for poly in polygons {
                    for ring in poly {
                        if let Some((first, rest)) = ring.split_first() {
                            path.move_to(pt(first.0, first.1));
                            for p in rest {
                                path.line_to(pt(p.0, p.1));
                            }
                            path.close_path();
                        }
                    }
                }
            }
        }

        path
    }
}

/// The 7 standard Strata Gwangnyeong-ri 1 survey layer kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrataLayerKind {
    /// 1. tr_investigation_boundary (조사구역 경계: polygon, 1.0 pt 일점쇄선)
    InvestigationBoundary,
    /// 2. tr_grid (피트/그리드 망: linestring/polygon, 0.5 pt 실선)
    Grid,
    /// 3. tr_remains_polygon (유구 다각형: polygon, 0.8 pt 실선 + 유구 종류별 자동 빗금)
    RemainsPolygon,
    /// 4. tr_remains_center (유구 중심점 및 라벨: point, 9 pt 글자)
    RemainsCenter,
    /// 5. tr_elevation_point (표고 실측점: point, 7 pt 표고 숫자 + 중심점)
    ElevationPoint,
    /// 6. tr_strata_section (토층 단면선: linestring, 0.2 pt 일점쇄선 + 양끝 A·A′)
    StrataSection,
    /// 7. tr_artifact_point (출토유물 위치 및 유물번호: point, 8 pt 원문자 ③)
    ArtifactPoint,
    /// Allied survey layer: control_points (기준점: △)
    ControlPoint,
    /// Other unclassified survey layer
    Other,
}

impl StrataLayerKind {
    /// Classifies layer table name or Strata key.
    pub fn classify(table_name: &str) -> Self {
        let lower = table_name.trim().to_ascii_lowercase();
        match lower.as_str() {
            "tr_investigation_boundary" | "survey_area" | "조사구역" | "조사구역경계" => {
                Self::InvestigationBoundary
            }
            "tr_grid" | "trial_trench" | "grid" | "grids" | "시굴격자" | "격자" => Self::Grid,
            "tr_remains_polygon"
            | "feature_poly"
            | "remains_polygon"
            | "remains_polygons"
            | "remains_pit"
            | "dwelling_perimeter"
            | "dwelling"
            | "dwellings"
            | "pit_dwelling"
            | "유구_면"
            | "유구면"
            | "주거지"
            | "주거지_외곽선"
            | "주거지외곽선"
            | "수혈유구" => Self::RemainsPolygon,
            "tr_remains_center"
            | "remains_center"
            | "feature_center"
            | "유구_중심점"
            | "유구중심점" => Self::RemainsCenter,
            "tr_elevation_point"
            | "elevation_point"
            | "elevation_points"
            | "spot_elevation"
            | "spot_elevations"
            | "표고점"
            | "표고실측점" => Self::ElevationPoint,
            "tr_strata_section"
            | "section_line"
            | "section_lines"
            | "단면선"
            | "단면위치선"
            | "토층단면선" => Self::StrataSection,
            "tr_artifact_point"
            | "artifact_point"
            | "artifact_points"
            | "finds_point"
            | "finds_points"
            | "artifact_finds"
            | "유물"
            | "유물출토위치" => Self::ArtifactPoint,
            "control_points" | "control_point" | "tr_control_point" | "기준점" => {
                Self::ControlPoint
            }
            _ => Self::Other,
        }
    }

    /// Target VectorCraft layer name.
    pub fn default_layer_name(self) -> &'static str {
        match self {
            Self::InvestigationBoundary => "조사구역 경계",
            Self::Grid => "시굴격자",
            Self::RemainsPolygon => "주거지 외곽선 (유구 면)",
            Self::RemainsCenter => "글자 · 번호",
            Self::ElevationPoint => "표고점",
            Self::StrataSection => "단면 위치선",
            Self::ArtifactPoint => "유물 출토 위치",
            Self::ControlPoint => "기준점",
            Self::Other => "Strata 가져오기",
        }
    }

    /// Default Legend definition for this layer kind.
    pub fn default_legend(self) -> Legend {
        match self {
            Self::InvestigationBoundary => Legend {
                weight: Some(1.0),
                line: LineType::DashDot,
                favourite: false,
                sample: LegendSample::Area,
                strata: Some("survey_area".to_string()),
                group: Some(LegendGroup::Plan),
            },
            Self::Grid => Legend {
                weight: Some(0.5),
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Area,
                strata: Some("trial_trench".to_string()),
                group: Some(LegendGroup::Plan),
            },
            Self::RemainsPolygon => Legend {
                weight: Some(0.8),
                line: LineType::Solid,
                favourite: true,
                sample: LegendSample::Area,
                strata: Some("feature_poly".to_string()),
                group: Some(LegendGroup::Plan),
            },
            Self::RemainsCenter => Legend {
                weight: None,
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Mark("가".to_string()),
                strata: Some("feature_center".to_string()),
                group: Some(LegendGroup::Plan),
            },
            Self::ElevationPoint => Legend {
                weight: None,
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Mark("·152".to_string()),
                strata: Some("elevation_point".to_string()),
                group: Some(LegendGroup::Plan),
            },
            Self::StrataSection => Legend {
                weight: Some(0.2),
                line: LineType::DashDot,
                favourite: true,
                sample: LegendSample::Line,
                strata: Some("section_line".to_string()),
                group: Some(LegendGroup::Plan),
            },
            Self::ArtifactPoint => Legend {
                weight: Some(0.2),
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Mark("③".to_string()),
                strata: Some("artifact_point".to_string()),
                group: Some(LegendGroup::Plan),
            },
            Self::ControlPoint => Legend {
                weight: None,
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Mark("△".to_string()),
                strata: Some("control_points".to_string()),
                group: Some(LegendGroup::Plan),
            },
            Self::Other => Legend::default(),
        }
    }
}

/// Feature hatch preset resolver for archaeological feature types (§7, §14(e)).
pub fn resolve_feature_hatch(feature_type: &str) -> Option<HatchStyle> {
    let lower = feature_type.trim().to_ascii_lowercase();
    if lower.contains("수혈") || lower.contains("pit") {
        Some(HatchStyle {
            direction: HatchDirection::Deg45,
            spacing_mm: 1.0,
            weight_pt: 0.1,
            flipped: false,
            resolved_direction: None,
        })
    } else if lower.contains("분묘") || lower.contains("석관묘") || lower.contains("tomb") {
        Some(HatchStyle {
            direction: HatchDirection::Deg135,
            spacing_mm: 1.0,
            weight_pt: 0.1,
            flipped: false,
            resolved_direction: None,
        })
    } else if lower.contains("구") || lower.contains("furrow") {
        // 0° hatch: represented as fixed angle 0°
        Some(HatchStyle {
            direction: HatchDirection::Deg45, // default fallback
            spacing_mm: 1.0,
            weight_pt: 0.1,
            flipped: false,
            resolved_direction: None,
        })
    } else if lower.contains("도랑") || lower.contains("ditch") || lower.contains("channel") {
        Some(HatchStyle {
            direction: HatchDirection::Deg135,
            spacing_mm: 1.0,
            weight_pt: 0.1,
            flipped: false,
            resolved_direction: None,
        })
    } else if lower.contains("주거지") || lower.contains("house") || lower.contains("dwelling") {
        None // 주거지: 없음 (open interior)
    } else {
        None
    }
}

/// An individual feature extracted from a GeoPackage layer.
#[derive(Clone, Debug)]
pub struct GpkgFeature {
    pub id: i64,
    pub geometry: GpkgGeometry,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// A spatial survey layer cataloged from a GeoPackage file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GpkgLayerInfo {
    pub table_name: String,
    pub kind: StrataLayerKind,
    pub geometry_type: String,
    pub feature_count: usize,
    pub srs_id: i32,
    pub bounds: Option<[f64; 4]>,
    pub target_layer: String,
}

/// Inspection summary of an entire GeoPackage database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GpkgSurveyInfo {
    pub name: String,
    pub path: PathBuf,
    pub srs_id: i32,
    pub srs_name: String,
    pub layers: Vec<GpkgLayerInfo>,
    pub total_features: usize,
    pub read_only: bool,
}

// ---------- Safe Byte Parsing Utilities ----------

fn read_u32(bytes: &[u8], is_le: bool) -> Option<u32> {
    let b = bytes.get(..4)?;
    let arr = [b[0], b[1], b[2], b[3]];
    Some(if is_le { u32::from_le_bytes(arr) } else { u32::from_be_bytes(arr) })
}

fn read_i32(bytes: &[u8], is_le: bool) -> Option<i32> {
    let b = bytes.get(..4)?;
    let arr = [b[0], b[1], b[2], b[3]];
    Some(if is_le { i32::from_le_bytes(arr) } else { i32::from_be_bytes(arr) })
}

fn read_f64(bytes: &[u8], is_le: bool) -> Option<f64> {
    let b = bytes.get(..8)?;
    let arr = [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]];
    let v = if is_le { f64::from_le_bytes(arr) } else { f64::from_be_bytes(arr) };
    if v.is_finite() { Some(v) } else { None }
}

/// Parse OGC GeoPackage binary geometry header (OGC 12-128r17 §3.1.1).
/// Returns `(srs_id, is_empty, wkb_slice)`.
pub fn parse_gpkg_header(bytes: &[u8]) -> Option<(i32, bool, &[u8])> {
    // Byte 0..2: 0x47, 0x50 ("GP")
    if bytes.get(0..2)? != [0x47, 0x50] {
        return None;
    }
    // Byte 2: version 0x00
    if *bytes.get(2)? != 0 {
        return None;
    }
    // Byte 3: flags
    let flags = *bytes.get(3)?;
    let is_le = (flags & 0x01) != 0;
    let env_indicator = (flags >> 1) & 0x07;
    let is_empty = (flags & 0x10) != 0;

    let env_len = match env_indicator {
        0 => 0,
        1 => 32, // min_x, max_x, min_y, max_y (4 * f64)
        2 | 3 => 48, // with z or m (6 * f64)
        4 => 64, // with z and m (8 * f64)
        _ => return None,
    };

    let srs_id = read_i32(bytes.get(4..8)?, is_le)?;
    let wkb_offset = 8 + env_len;
    let wkb_slice = bytes.get(wkb_offset..)?;

    Some((srs_id, is_empty, wkb_slice))
}

/// Safe ISO 19125-1 Well-Known Binary (WKB) parser.
pub fn parse_wkb(bytes: &[u8]) -> Option<GpkgGeometry> {
    let mut cursor = 0;
    parse_wkb_element(bytes, &mut cursor)
}

fn parse_wkb_element(bytes: &[u8], cursor: &mut usize) -> Option<GpkgGeometry> {
    let order_byte = *bytes.get(*cursor)?;
    *cursor += 1;
    let is_le = match order_byte {
        0 => false,
        1 => true,
        _ => return None,
    };

    let raw_geom_type = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)?;
    *cursor += 4;

    // Check for 3D indicators: ISO 1000..1999 or EWKB 0x80000000 bit
    let has_z = (raw_geom_type >= 1000 && raw_geom_type < 2000)
        || ((raw_geom_type & 0x8000_0000) != 0);
    let base_type = if raw_geom_type >= 1000 && raw_geom_type < 2000 {
        raw_geom_type - 1000
    } else {
        raw_geom_type & 0x0FFF
    };

    match base_type {
        1 => {
            // Point
            let x = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
            *cursor += 8;
            let y = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
            *cursor += 8;
            let z = if has_z {
                let val = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
                *cursor += 8;
                Some(val)
            } else {
                None
            };
            Some(GpkgGeometry::Point { x, y, z })
        }
        2 => {
            // LineString
            let num_points = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)? as usize;
            *cursor += 4;
            let mut points = Vec::with_capacity(num_points.min(100_000));
            for _ in 0..num_points {
                let x = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
                *cursor += 8;
                let y = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
                *cursor += 8;
                let z = if has_z {
                    let val = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
                    *cursor += 8;
                    Some(val)
                } else {
                    None
                };
                points.push((x, y, z));
            }
            Some(GpkgGeometry::LineString { points })
        }
        3 => {
            // Polygon
            let num_rings = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)? as usize;
            *cursor += 4;
            let mut rings = Vec::with_capacity(num_rings.min(10_000));
            for _ in 0..num_rings {
                let num_points = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)? as usize;
                *cursor += 4;
                let mut ring = Vec::with_capacity(num_points.min(100_000));
                for _ in 0..num_points {
                    let x = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
                    *cursor += 8;
                    let y = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
                    *cursor += 8;
                    let z = if has_z {
                        let val = read_f64(bytes.get(*cursor..*cursor + 8)?, is_le)?;
                        *cursor += 8;
                        Some(val)
                    } else {
                        None
                    };
                    ring.push((x, y, z));
                }
                rings.push(ring);
            }
            Some(GpkgGeometry::Polygon { rings })
        }
        4 => {
            // MultiPoint
            let num_geoms = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)? as usize;
            *cursor += 4;
            let mut points = Vec::with_capacity(num_geoms.min(50_000));
            for _ in 0..num_geoms {
                if let Some(GpkgGeometry::Point { x, y, z }) = parse_wkb_element(bytes, cursor) {
                    points.push((x, y, z));
                }
            }
            Some(GpkgGeometry::MultiPoint { points })
        }
        5 => {
            // MultiLineString
            let num_geoms = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)? as usize;
            *cursor += 4;
            let mut lines = Vec::with_capacity(num_geoms.min(10_000));
            for _ in 0..num_geoms {
                if let Some(GpkgGeometry::LineString { points }) = parse_wkb_element(bytes, cursor) {
                    lines.push(points);
                }
            }
            Some(GpkgGeometry::MultiLineString { lines })
        }
        6 => {
            // MultiPolygon
            let num_geoms = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)? as usize;
            *cursor += 4;
            let mut polygons = Vec::with_capacity(num_geoms.min(1_000));
            for _ in 0..num_geoms {
                if let Some(GpkgGeometry::Polygon { rings }) = parse_wkb_element(bytes, cursor) {
                    polygons.push(rings);
                }
            }
            Some(GpkgGeometry::MultiPolygon { polygons })
        }
        _ => None,
    }
}

// ---------- Desktop-Gated SQLite Reader ----------

#[cfg(not(target_arch = "wasm32"))]
pub struct GpkgReader {
    conn: rusqlite::Connection,
    path: PathBuf,
}

#[cfg(target_arch = "wasm32")]
pub struct GpkgReader;

#[cfg(not(target_arch = "wasm32"))]
impl GpkgReader {
    /// Open a GeoPackage file strictly read-only.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref().to_path_buf();
        let conn = rusqlite::Connection::open_with_flags(
            &p,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| format!("Failed to open GeoPackage at {}: {e}", p.display()))?;

        Ok(Self { conn, path: p })
    }

    /// Open an in-memory database for testing.
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| format!("Failed to create in-memory sqlite: {e}"))?;
        Ok(Self {
            conn,
            path: PathBuf::from(":memory:"),
        })
    }

    /// Access reference to underlying SQLite connection (for test seeding).
    pub fn connection_mut(&mut self) -> &mut rusqlite::Connection {
        &mut self.conn
    }

    /// Inspect database contents and metadata.
    pub fn inspect(&self) -> Result<GpkgSurveyInfo, String> {
        let mut srs_id = 5186;
        let mut srs_name = "EPSG:5186 (중부원점 2010)".to_string();

        if let Ok(mut stmt) = self
            .conn
            .prepare("SELECT srs_id, srs_name FROM gpkg_spatial_ref_sys WHERE srs_id != 0 AND srs_id != -1 LIMIT 1")
        {
            if let Ok(mut rows) = stmt.query([]) {
                if let Ok(Some(row)) = rows.next() {
                    let id: i32 = row.get(0).unwrap_or(5186);
                    let name: String = row.get(1).unwrap_or_else(|_| "EPSG:5186".to_string());
                    srs_id = id;
                    srs_name = name;
                }
            }
        }

        let mut layers = Vec::new();
        let mut total_features = 0;

        let query = "SELECT c.table_name, c.min_x, c.min_y, c.max_x, c.max_y, c.srs_id, g.geometry_type_name
                     FROM gpkg_contents c
                     LEFT JOIN gpkg_geometry_columns g ON c.table_name = g.table_name
                     WHERE c.data_type = 'features'";

        if let Ok(mut stmt) = self.conn.prepare(query) {
            let rows_iter = stmt.query_map([], |row| {
                let table_name: String = row.get(0)?;
                let min_x: Option<f64> = row.get(1).ok();
                let min_y: Option<f64> = row.get(2).ok();
                let max_x: Option<f64> = row.get(3).ok();
                let max_y: Option<f64> = row.get(4).ok();
                let layer_srs: i32 = row.get(5).unwrap_or(srs_id);
                let geom_type: String = row.get(6).unwrap_or_else(|_| "GEOMETRY".to_string());
                Ok((table_name, min_x, min_y, max_x, max_y, layer_srs, geom_type))
            });

            if let Ok(iter) = rows_iter {
                for item in iter.flatten() {
                    let (table_name, min_x, min_y, max_x, max_y, layer_srs, geom_type) = item;

                    let count_query = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
                    let count: usize = self
                        .conn
                        .query_row(&count_query, [], |r| r.get(0))
                        .unwrap_or(0);

                    total_features += count;
                    let kind = StrataLayerKind::classify(&table_name);
                    let target_layer = kind.default_layer_name().to_string();

                    let bounds = match (min_x, min_y, max_x, max_y) {
                        (Some(x0), Some(y0), Some(x1), Some(y1)) => Some([x0, y0, x1, y1]),
                        _ => None,
                    };

                    layers.push(GpkgLayerInfo {
                        table_name,
                        kind,
                        geometry_type: geom_type,
                        feature_count: count,
                        srs_id: layer_srs,
                        bounds,
                        target_layer,
                    });
                }
            }
        }

        let survey_name = self
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Strata Survey")
            .to_string();

        Ok(GpkgSurveyInfo {
            name: survey_name,
            path: self.path.clone(),
            srs_id,
            srs_name,
            layers,
            total_features,
            read_only: true,
        })
    }

    /// Read all features from a table.
    pub fn read_layer_features(&self, table_name: &str) -> Result<Vec<GpkgFeature>, String> {
        let geom_col_query = "SELECT column_name FROM gpkg_geometry_columns WHERE table_name = ?1";
        let geom_col: String = self
            .conn
            .query_row(geom_col_query, [&table_name], |r| r.get(0))
            .unwrap_or_else(|_| "geom".to_string());

        let query = format!("SELECT * FROM \"{}\"", table_name);
        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| format!("Failed to prepare query on {table_name}: {e}"))?;

        let col_count = stmt.column_count();
        let col_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();

        let mut rows = stmt
            .query([])
            .map_err(|e| format!("Query failed on {table_name}: {e}"))?;

        let mut features = Vec::new();

        while let Some(row) = rows.next().map_err(|e| format!("Row read error: {e}"))? {
            let mut feature_id = 0;
            let mut geometry_blob: Option<Vec<u8>> = None;
            let mut attrs = HashMap::new();

            for i in 0..col_count {
                let col_name = col_names.get(i).map(|s| s.as_str()).unwrap_or("");
                if col_name == "id" || col_name == "fid" {
                    if let Ok(id_val) = row.get(i) {
                        feature_id = id_val;
                    }
                } else if col_name == geom_col {
                    if let Ok(blob) = row.get::<_, Vec<u8>>(i) {
                        geometry_blob = Some(blob);
                    }
                } else {
                    // Attribute
                    if let Ok(val_str) = row.get::<_, String>(i) {
                        attrs.insert(col_name.to_string(), serde_json::Value::String(val_str));
                    } else if let Ok(val_f64) = row.get::<_, f64>(i) {
                        if let Some(num) = serde_json::Number::from_f64(val_f64) {
                            attrs.insert(col_name.to_string(), serde_json::Value::Number(num));
                        }
                    } else if let Ok(val_i64) = row.get::<_, i64>(i) {
                        attrs.insert(
                            col_name.to_string(),
                            serde_json::Value::Number(val_i64.into()),
                        );
                    }
                }
            }

            if let Some(blob) = geometry_blob {
                if let Some((_srs, _is_empty, wkb)) = parse_gpkg_header(&blob) {
                    if let Some(geom) = parse_wkb(wkb) {
                        features.push(GpkgFeature {
                            id: feature_id,
                            geometry: geom,
                            attributes: attrs,
                        });
                    }
                }
            }
        }

        Ok(features)
    }
}

/// Helper function to create a minimal valid OGC GeoPackage test database on disk.
#[cfg(not(target_arch = "wasm32"))]
pub fn create_sample_gpkg(path: &Path) -> Result<(), String> {
    let conn = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute_batch(
        "
        CREATE TABLE gpkg_spatial_ref_sys (
            srs_name TEXT NOT NULL,
            srs_id INTEGER NOT NULL PRIMARY KEY,
            organization TEXT NOT NULL,
            organization_coordsys_id INTEGER NOT NULL,
            definition TEXT NOT NULL,
            description TEXT
        );
        INSERT INTO gpkg_spatial_ref_sys VALUES ('Korea 2000 / Central Belt 2010', 5186, 'EPSG', 5186, '', 'Korean archaeological grid');

        CREATE TABLE gpkg_contents (
            table_name TEXT NOT NULL PRIMARY KEY,
            data_type TEXT NOT NULL,
            identifier TEXT,
            description TEXT,
            last_change TEXT NOT NULL DEFAULT '',
            min_x REAL,
            min_y REAL,
            max_x REAL,
            max_y REAL,
            srs_id INTEGER
        );
        INSERT INTO gpkg_contents VALUES ('dwelling_perimeter', 'features', '1호 주거지 외곽선', 'Gwangnyeong-ri dwelling', '', 100.0, 200.0, 120.0, 220.0, 5186);
        INSERT INTO gpkg_contents VALUES ('elevation_points', 'features', '표고점', 'Spot heights', '', 100.0, 200.0, 110.0, 210.0, 5186);

        CREATE TABLE gpkg_geometry_columns (
            table_name TEXT NOT NULL,
            column_name TEXT NOT NULL,
            geometry_type_name TEXT NOT NULL,
            srs_id INTEGER NOT NULL,
            z TINYINT NOT NULL,
            m TINYINT NOT NULL,
            PRIMARY KEY (table_name, column_name)
        );
        INSERT INTO gpkg_geometry_columns VALUES ('dwelling_perimeter', 'geom', 'POLYGON', 5186, 0, 0);
        INSERT INTO gpkg_geometry_columns VALUES ('elevation_points', 'geom', 'POINT', 5186, 0, 0);

        CREATE TABLE dwelling_perimeter (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            geom BLOB,
            feature_name TEXT
        );

        CREATE TABLE elevation_points (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            geom BLOB,
            elevation REAL
        );
        ",
    ).map_err(|e| e.to_string())?;

    let mut poly_blob = vec![0x47, 0x50, 0x00, 0x01];
    poly_blob.extend_from_slice(&5186i32.to_le_bytes());
    poly_blob.push(0x01);
    poly_blob.extend_from_slice(&3u32.to_le_bytes());
    poly_blob.extend_from_slice(&1u32.to_le_bytes());
    poly_blob.extend_from_slice(&4u32.to_le_bytes());
    for (x, y) in [(100.0f64, 200.0f64), (120.0, 200.0), (120.0, 220.0), (100.0, 200.0)] {
        poly_blob.extend_from_slice(&x.to_le_bytes());
        poly_blob.extend_from_slice(&y.to_le_bytes());
    }

    conn.execute(
        "INSERT INTO dwelling_perimeter (geom, feature_name) VALUES (?1, ?2)",
        rusqlite::params![poly_blob, "1호 주거지"],
    ).map_err(|e| e.to_string())?;

    let mut pt_blob = vec![0x47, 0x50, 0x00, 0x01];
    pt_blob.extend_from_slice(&5186i32.to_le_bytes());
    pt_blob.push(0x01);
    pt_blob.extend_from_slice(&1u32.to_le_bytes());
    pt_blob.extend_from_slice(&105.0f64.to_le_bytes());
    pt_blob.extend_from_slice(&205.0f64.to_le_bytes());

    conn.execute(
        "INSERT INTO elevation_points (geom, elevation) VALUES (?1, ?2)",
        rusqlite::params![pt_blob, 152.45f64],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
impl GpkgReader {
    pub fn open<P: AsRef<Path>>(_path: P) -> Result<Self, String> {
        Err("Strata GeoPackage survey reader is desktop-only".to_string())
    }

    pub fn inspect(&self) -> Result<GpkgSurveyInfo, String> {
        Err("Strata GeoPackage survey reader is desktop-only".to_string())
    }

    pub fn read_layer_features(&self, _table_name: &str) -> Result<Vec<GpkgFeature>, String> {
        Err("Strata GeoPackage survey reader is desktop-only".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wkb_point_and_linestring() {
        // Point (152840.123, 512340.456) Little-Endian
        let mut buf = Vec::new();
        buf.push(1); // Little-endian
        buf.extend_from_slice(&1u32.to_le_bytes()); // Type = Point
        buf.extend_from_slice(&152840.123_f64.to_le_bytes());
        buf.extend_from_slice(&512340.456_f64.to_le_bytes());

        let geom = parse_wkb(&buf).unwrap();
        match geom {
            GpkgGeometry::Point { x, y, z } => {
                assert_eq!(x, 152840.123);
                assert_eq!(y, 512340.456);
                assert_eq!(z, None);
            }
            _ => panic!("Expected Point"),
        }

        // LineString of 2 points
        let mut line_buf = Vec::new();
        line_buf.push(1);
        line_buf.extend_from_slice(&2u32.to_le_bytes()); // Type = LineString
        line_buf.extend_from_slice(&2u32.to_le_bytes()); // Count = 2
        line_buf.extend_from_slice(&100.0_f64.to_le_bytes());
        line_buf.extend_from_slice(&200.0_f64.to_le_bytes());
        line_buf.extend_from_slice(&300.0_f64.to_le_bytes());
        line_buf.extend_from_slice(&400.0_f64.to_le_bytes());

        let line_geom = parse_wkb(&line_buf).unwrap();
        match line_geom {
            GpkgGeometry::LineString { points } => {
                assert_eq!(points.len(), 2);
                assert_eq!(points[0], (100.0, 200.0, None));
                assert_eq!(points[1], (300.0, 400.0, None));
            }
            _ => panic!("Expected LineString"),
        }
    }

    #[test]
    fn test_parse_gpkg_header_and_envelope() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[0x47, 0x50]); // "GP"
        buf.push(0x00); // version
        buf.push(0x01 | (0x01 << 1)); // LE (bit 0=1), envelope indicator 1 (32 bytes)
        buf.extend_from_slice(&5186i32.to_le_bytes()); // SRS ID 5186
        buf.extend_from_slice(&[0u8; 32]); // Envelope 32 bytes

        // Append a Point WKB
        buf.push(1);
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(&10.0_f64.to_le_bytes());
        buf.extend_from_slice(&20.0_f64.to_le_bytes());

        let (srs_id, is_empty, wkb) = parse_gpkg_header(&buf).unwrap();
        assert_eq!(srs_id, 5186);
        assert!(!is_empty);
        assert_eq!(wkb.len(), 21);

        let geom = parse_wkb(wkb).unwrap();
        match geom {
            GpkgGeometry::Point { x, y, .. } => {
                assert_eq!(x, 10.0);
                assert_eq!(y, 20.0);
            }
            _ => panic!("Expected Point"),
        }
    }

    #[test]
    fn test_strata_layer_classification() {
        assert_eq!(
            StrataLayerKind::classify("tr_remains_polygon"),
            StrataLayerKind::RemainsPolygon
        );
        assert_eq!(
            StrataLayerKind::classify("tr_investigation_boundary"),
            StrataLayerKind::InvestigationBoundary
        );
        assert_eq!(
            StrataLayerKind::classify("tr_strata_section"),
            StrataLayerKind::StrataSection
        );
        assert_eq!(
            StrataLayerKind::classify("tr_artifact_point"),
            StrataLayerKind::ArtifactPoint
        );
        assert_eq!(
            StrataLayerKind::classify("tr_elevation_point"),
            StrataLayerKind::ElevationPoint
        );
        assert_eq!(
            StrataLayerKind::classify("dwelling_perimeter"),
            StrataLayerKind::RemainsPolygon
        );
        assert_eq!(
            StrataLayerKind::classify("elevation_points"),
            StrataLayerKind::ElevationPoint
        );
    }
}
