//! GeoPackage survey reader for Strata survey data (OGC 12-128r17, Trowel §11, §13, §14, Task J019).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use kurbo::{BezPath, Point};
use serde::{Deserialize, Serialize};
use vectorcraft_doc::hatch::{HatchDirection, HatchStyle};
use vectorcraft_doc::scale::ScaleConfig;
use vectorcraft_doc::{Legend, LegendGroup, LegendSample, LineType};

pub type GpkgPoint3 = (f64, f64, Option<f64>);
pub type GpkgRing = Vec<GpkgPoint3>;
pub type GpkgPolygonRings = Vec<GpkgRing>;

/// Supported geometry types in GeoPackage feature tables.
#[derive(Clone, Debug, PartialEq)]
pub enum GpkgGeometry {
    Point { x: f64, y: f64, z: Option<f64> },
    LineString { points: Vec<GpkgPoint3> },
    Polygon { rings: GpkgPolygonRings },
    MultiPoint { points: Vec<GpkgPoint3> },
    MultiLineString { lines: Vec<GpkgRing> },
    MultiPolygon { polygons: Vec<GpkgPolygonRings> },
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

        if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() { Some([min_x, min_y, max_x, max_y]) } else { None }
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
            "tr_investigation_boundary" | "survey_area" | "조사구역" | "조사구역경계" => Self::InvestigationBoundary,
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
            "tr_remains_center" | "remains_center" | "feature_center" | "유구_중심점" | "유구중심점" => Self::RemainsCenter,
            "tr_elevation_point" | "elevation_point" | "elevation_points" | "spot_elevation" | "spot_elevations" | "표고점" | "표고실측점" => {
                Self::ElevationPoint
            }
            "tr_strata_section" | "section_line" | "section_lines" | "단면선" | "단면위치선" | "토층단면선" => Self::StrataSection,
            "tr_artifact_point" | "artifact_point" | "artifact_points" | "finds_point" | "finds_points" | "artifact_finds" | "유물"
            | "유물출토위치" => Self::ArtifactPoint,
            "control_points" | "control_point" | "tr_control_point" | "기준점" => Self::ControlPoint,
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

    // 주거지 (dwelling/house): 없음 (open interior)
    if lower.contains("주거지") || lower.contains("house") || lower.contains("dwelling") {
        None
    } else if lower.contains("수혈") || lower.contains("pit") {
        // 수혈: ╱ 45°
        Some(HatchStyle { direction: HatchDirection::Deg45, spacing_mm: 1.0, weight_pt: 0.1, flipped: false, resolved_direction: None })
    } else if lower.contains("분묘")
        || lower.contains("석관묘")
        || lower.contains("목관묘")
        || lower.contains("고분")
        || lower.contains("tomb")
        || lower.contains("grave")
    {
        // 분묘: ╲ 135°
        Some(HatchStyle { direction: HatchDirection::Deg135, spacing_mm: 1.0, weight_pt: 0.1, flipped: false, resolved_direction: None })
    } else if (lower.contains("구") && !lower.contains("유구")) || lower.contains("furrow") {
        // 구: ─ 0° (guard against generic '유구')
        Some(HatchStyle { direction: HatchDirection::Deg0, spacing_mm: 1.0, weight_pt: 0.1, flipped: false, resolved_direction: None })
    } else if lower.contains("도랑") || lower.contains("ditch") || lower.contains("channel") {
        // 도랑: │ 90°
        Some(HatchStyle { direction: HatchDirection::Deg90, spacing_mm: 1.0, weight_pt: 0.1, flipped: false, resolved_direction: None })
    } else if lower.contains("가마") || lower.contains("kiln") || lower.contains("furnace") {
        // 가마: ┼
        Some(HatchStyle { direction: HatchDirection::Cross, spacing_mm: 1.0, weight_pt: 0.1, flipped: false, resolved_direction: None })
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
        1 => 32,     // min_x, max_x, min_y, max_y (4 * f64)
        2 | 3 => 48, // with z or m (6 * f64)
        4 => 64,     // with z and m (8 * f64)
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
    let has_z = (1000..2000).contains(&raw_geom_type) || ((raw_geom_type & 0x8000_0000) != 0);
    let base_type = if (1000..2000).contains(&raw_geom_type) { raw_geom_type - 1000 } else { raw_geom_type & 0x0FFF };

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
                let Some(GpkgGeometry::Point { x, y, z }) = parse_wkb_element(bytes, cursor) else {
                    return None;
                };
                points.push((x, y, z));
            }
            Some(GpkgGeometry::MultiPoint { points })
        }
        5 => {
            // MultiLineString
            let num_geoms = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)? as usize;
            *cursor += 4;
            let mut lines = Vec::with_capacity(num_geoms.min(10_000));
            for _ in 0..num_geoms {
                let Some(GpkgGeometry::LineString { points }) = parse_wkb_element(bytes, cursor) else {
                    return None;
                };
                lines.push(points);
            }
            Some(GpkgGeometry::MultiLineString { lines })
        }
        6 => {
            // MultiPolygon
            let num_geoms = read_u32(bytes.get(*cursor..*cursor + 4)?, is_le)? as usize;
            *cursor += 4;
            let mut polygons = Vec::with_capacity(num_geoms.min(1_000));
            for _ in 0..num_geoms {
                let Some(GpkgGeometry::Polygon { rings }) = parse_wkb_element(bytes, cursor) else {
                    return None;
                };
                polygons.push(rings);
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
        let conn = rusqlite::Connection::open_with_flags(&p, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| format!("Failed to open GeoPackage at {}: {e}", p.display()))?;

        Ok(Self { conn, path: p })
    }

    /// Open an in-memory database for testing.
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = rusqlite::Connection::open_in_memory().map_err(|e| format!("Failed to create in-memory sqlite: {e}"))?;
        Ok(Self { conn, path: PathBuf::from(":memory:") })
    }

    /// Access reference to underlying SQLite connection (for test seeding).
    pub fn connection_mut(&mut self) -> &mut rusqlite::Connection {
        &mut self.conn
    }

    /// Inspect database contents and metadata.
    pub fn inspect(&self) -> Result<GpkgSurveyInfo, String> {
        let mut srs_id = 5186;
        let mut srs_name = "EPSG:5186 (중부원점 2010)".to_string();

        if let Ok(mut stmt) = self.conn.prepare("SELECT srs_id, srs_name FROM gpkg_spatial_ref_sys WHERE srs_id != 0 AND srs_id != -1 LIMIT 1")
            && let Ok(mut rows) = stmt.query([])
            && let Ok(Some(row)) = rows.next()
        {
            let id: i32 = row.get(0).unwrap_or(5186);
            let name: String = row.get(1).unwrap_or_else(|_| "EPSG:5186".to_string());
            srs_id = id;
            srs_name = name;
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
                    let count: usize = self.conn.query_row(&count_query, [], |r| r.get(0)).unwrap_or(0);

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

        let survey_name = self.path.file_stem().and_then(|s| s.to_str()).unwrap_or("Strata Survey").to_string();

        Ok(GpkgSurveyInfo { name: survey_name, path: self.path.clone(), srs_id, srs_name, layers, total_features, read_only: true })
    }

    /// Read all features from a table.
    pub fn read_layer_features(&self, table_name: &str) -> Result<Vec<GpkgFeature>, String> {
        let geom_col_query = "SELECT column_name FROM gpkg_geometry_columns WHERE table_name = ?1";
        let geom_col: String = self.conn.query_row(geom_col_query, [&table_name], |r| r.get(0)).unwrap_or_else(|_| "geom".to_string());

        let query = format!("SELECT * FROM \"{}\"", table_name);
        let mut stmt = self.conn.prepare(&query).map_err(|e| format!("Failed to prepare query on {table_name}: {e}"))?;

        let col_count = stmt.column_count();
        let col_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();

        let mut rows = stmt.query([]).map_err(|e| format!("Query failed on {table_name}: {e}"))?;

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
                        attrs.insert(col_name.to_string(), serde_json::Value::Number(val_i64.into()));
                    }
                }
            }

            if let Some(blob) = geometry_blob
                && let Some((_srs, _is_empty, wkb)) = parse_gpkg_header(&blob)
                && let Some(geom) = parse_wkb(wkb)
            {
                features.push(GpkgFeature { id: feature_id, geometry: geom, attributes: attrs });
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

    conn.execute("INSERT INTO dwelling_perimeter (geom, feature_name) VALUES (?1, ?2)", rusqlite::params![poly_blob, "1호 주거지"])
        .map_err(|e| e.to_string())?;

    let mut pt_blob = vec![0x47, 0x50, 0x00, 0x01];
    pt_blob.extend_from_slice(&5186i32.to_le_bytes());
    pt_blob.push(0x01);
    pt_blob.extend_from_slice(&1u32.to_le_bytes());
    pt_blob.extend_from_slice(&105.0f64.to_le_bytes());
    pt_blob.extend_from_slice(&205.0f64.to_le_bytes());

    conn.execute("INSERT INTO elevation_points (geom, elevation) VALUES (?1, ?2)", rusqlite::params![pt_blob, 152.45f64])
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Helper function to create a 7-layer Strata survey GeoPackage test database on disk.
#[cfg(not(target_arch = "wasm32"))]
pub fn create_7layers_sample_gpkg(path: &Path) -> Result<(), String> {
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
            min_x REAL, min_y REAL, max_x REAL, max_y REAL,
            srs_id INTEGER
        );

        CREATE TABLE gpkg_geometry_columns (
            table_name TEXT NOT NULL,
            column_name TEXT NOT NULL,
            geometry_type_name TEXT NOT NULL,
            srs_id INTEGER NOT NULL,
            z TINYINT NOT NULL,
            m TINYINT NOT NULL,
            PRIMARY KEY (table_name, column_name)
        );

        -- 1. Investigation Boundary
        INSERT INTO gpkg_contents VALUES ('tr_investigation_boundary', 'features', '조사구역', '', '', 0.0, 0.0, 100.0, 100.0, 5186);
        INSERT INTO gpkg_geometry_columns VALUES ('tr_investigation_boundary', 'geom', 'POLYGON', 5186, 0, 0);
        CREATE TABLE tr_investigation_boundary (id INTEGER PRIMARY KEY, geom BLOB, name TEXT);

        -- 2. Grid
        INSERT INTO gpkg_contents VALUES ('tr_grid', 'features', '그리드', '', '', 0.0, 0.0, 100.0, 100.0, 5186);
        INSERT INTO gpkg_geometry_columns VALUES ('tr_grid', 'geom', 'LINESTRING', 5186, 0, 0);
        CREATE TABLE tr_grid (id INTEGER PRIMARY KEY, geom BLOB, grid_id TEXT);

        -- 3. Remains Polygon
        INSERT INTO gpkg_contents VALUES ('tr_remains_polygon', 'features', '유구면', '', '', 10.0, 10.0, 50.0, 50.0, 5186);
        INSERT INTO gpkg_geometry_columns VALUES ('tr_remains_polygon', 'geom', 'POLYGON', 5186, 0, 0);
        CREATE TABLE tr_remains_polygon (id INTEGER PRIMARY KEY, geom BLOB, feature_name TEXT, feature_type TEXT);

        -- 4. Remains Center
        INSERT INTO gpkg_contents VALUES ('tr_remains_center', 'features', '유구중심점', '', '', 20.0, 20.0, 20.0, 20.0, 5186);
        INSERT INTO gpkg_geometry_columns VALUES ('tr_remains_center', 'geom', 'POINT', 5186, 0, 0);
        CREATE TABLE tr_remains_center (id INTEGER PRIMARY KEY, geom BLOB, feature_name TEXT);

        -- 5. Elevation Point
        INSERT INTO gpkg_contents VALUES ('tr_elevation_point', 'features', '표고점', '', '', 25.0, 25.0, 25.0, 25.0, 5186);
        INSERT INTO gpkg_geometry_columns VALUES ('tr_elevation_point', 'geom', 'POINT', 5186, 0, 0);
        CREATE TABLE tr_elevation_point (id INTEGER PRIMARY KEY, geom BLOB, elevation REAL);

        -- 6. Strata Section
        INSERT INTO gpkg_contents VALUES ('tr_strata_section', 'features', '단면선', '', '', 10.0, 30.0, 40.0, 30.0, 5186);
        INSERT INTO gpkg_geometry_columns VALUES ('tr_strata_section', 'geom', 'LINESTRING', 5186, 0, 0);
        CREATE TABLE tr_strata_section (id INTEGER PRIMARY KEY, geom BLOB, section_name TEXT);

        -- 7. Artifact Point
        INSERT INTO gpkg_contents VALUES ('tr_artifact_point', 'features', '유물점', '', '', 22.0, 22.0, 22.0, 22.0, 5186);
        INSERT INTO gpkg_geometry_columns VALUES ('tr_artifact_point', 'geom', 'POINT', 5186, 0, 0);
        CREATE TABLE tr_artifact_point (id INTEGER PRIMARY KEY, geom BLOB, find_number TEXT);
        ",
    )
    .map_err(|e| e.to_string())?;

    // Insert a pit polygon ("1호 수혈") into tr_remains_polygon
    let mut pit_blob = vec![0x47, 0x50, 0x00, 0x01];
    pit_blob.extend_from_slice(&5186i32.to_le_bytes());
    pit_blob.push(0x01); // LE WKB
    pit_blob.extend_from_slice(&3u32.to_le_bytes()); // Polygon
    pit_blob.extend_from_slice(&1u32.to_le_bytes()); // 1 ring
    pit_blob.extend_from_slice(&5u32.to_le_bytes()); // 5 points
    for (x, y) in [(10.0_f64, 10.0_f64), (15.0, 10.0), (15.0, 15.0), (10.0, 15.0), (10.0, 10.0)] {
        pit_blob.extend_from_slice(&x.to_le_bytes());
        pit_blob.extend_from_slice(&y.to_le_bytes());
    }
    conn.execute(
        "INSERT INTO tr_remains_polygon (geom, feature_name, feature_type) VALUES (?1, ?2, ?3)",
        rusqlite::params![pit_blob, "1호 수혈", "수혈"],
    )
    .map_err(|e| e.to_string())?;

    // Insert a dwelling polygon ("1호 주거지") into tr_remains_polygon
    let mut house_blob = vec![0x47, 0x50, 0x00, 0x01];
    house_blob.extend_from_slice(&5186i32.to_le_bytes());
    house_blob.push(0x01);
    house_blob.extend_from_slice(&3u32.to_le_bytes());
    house_blob.extend_from_slice(&1u32.to_le_bytes());
    house_blob.extend_from_slice(&5u32.to_le_bytes());
    for (x, y) in [(20.0_f64, 20.0_f64), (30.0, 20.0), (30.0, 30.0), (20.0, 30.0), (20.0, 20.0)] {
        house_blob.extend_from_slice(&x.to_le_bytes());
        house_blob.extend_from_slice(&y.to_le_bytes());
    }
    conn.execute(
        "INSERT INTO tr_remains_polygon (geom, feature_name, feature_type) VALUES (?1, ?2, ?3)",
        rusqlite::params![house_blob, "1호 주거지", "주거지"],
    )
    .map_err(|e| e.to_string())?;

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
        assert_eq!(StrataLayerKind::classify("tr_remains_polygon"), StrataLayerKind::RemainsPolygon);
        assert_eq!(StrataLayerKind::classify("tr_investigation_boundary"), StrataLayerKind::InvestigationBoundary);
        assert_eq!(StrataLayerKind::classify("tr_strata_section"), StrataLayerKind::StrataSection);
        assert_eq!(StrataLayerKind::classify("tr_artifact_point"), StrataLayerKind::ArtifactPoint);
        assert_eq!(StrataLayerKind::classify("tr_elevation_point"), StrataLayerKind::ElevationPoint);
        assert_eq!(StrataLayerKind::classify("dwelling_perimeter"), StrataLayerKind::RemainsPolygon);
        assert_eq!(StrataLayerKind::classify("elevation_points"), StrataLayerKind::ElevationPoint);
    }

    #[test]
    fn test_stress_ogc_header_parser() {
        // Valid Little-Endian header with no envelope (env_indicator 0)
        let mut buf_le = vec![0x47, 0x50, 0x00, 0x01]; // GP, ver 0, flags (LE, no env)
        buf_le.extend_from_slice(&5186i32.to_le_bytes());
        buf_le.extend_from_slice(&[0x01, 0x01, 0x00, 0x00, 0x00]); // Point LE type
        let (srs, empty, wkb) = parse_gpkg_header(&buf_le).expect("Valid LE header must parse");
        assert_eq!(srs, 5186);
        assert!(!empty);
        assert_eq!(wkb.len(), 5);

        // Valid Big-Endian header (flags bit 0 = 0)
        let mut buf_be = vec![0x47, 0x50, 0x00, 0x00]; // GP, ver 0, flags (BE, no env)
        buf_be.extend_from_slice(&5186i32.to_be_bytes());
        buf_be.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x01]); // Point BE type
        let (srs_be, empty_be, wkb_be) = parse_gpkg_header(&buf_be).expect("Valid BE header must parse");
        assert_eq!(srs_be, 5186);
        assert!(!empty_be);
        assert_eq!(wkb_be.len(), 5);

        // Envelope variants: 1 (32 bytes), 2 (48 bytes), 3 (48 bytes), 4 (64 bytes)
        for (indicator, env_len) in [(1, 32), (2, 48), (3, 48), (4, 64)] {
            let mut buf_env = vec![0x47, 0x50, 0x00, 0x01 | (indicator << 1)];
            buf_env.extend_from_slice(&5186i32.to_le_bytes());
            buf_env.extend_from_slice(&vec![0u8; env_len]);
            buf_env.push(0xAA); // 1 byte wkb payload
            let (srs_e, _, wkb_e) = parse_gpkg_header(&buf_env).expect("Valid envelope must parse");
            assert_eq!(srs_e, 5186);
            assert_eq!(wkb_e, &[0xAA]);

            // Truncated envelope: buffer shorter than 8 + env_len
            let truncated_env = &buf_env[..8 + env_len - 1];
            assert!(parse_gpkg_header(truncated_env).is_none(), "Truncated envelope must return None");
        }

        // Malformed headers: truncated < 8 bytes
        for len in 0..8 {
            let slice = &buf_le[..len.min(buf_le.len())];
            assert!(parse_gpkg_header(slice).is_none(), "Truncated header < 8 bytes must return None");
        }

        // Malformed magic
        let bad_magics = [[0x00, 0x00], [0x47, 0x00], [0x00, 0x50], [0xFF, 0xFF]];
        for magic in bad_magics {
            let mut bad_buf = buf_le.clone();
            bad_buf[0] = magic[0];
            bad_buf[1] = magic[1];
            assert!(parse_gpkg_header(&bad_buf).is_none(), "Invalid magic must return None");
        }

        // Malformed version != 0
        let mut bad_ver = buf_le.clone();
        bad_ver[2] = 1;
        assert!(parse_gpkg_header(&bad_ver).is_none(), "Non-zero version must return None");

        // Unsupported flags: envelope indicator 5, 6, 7
        for ind in 5..=7 {
            let mut bad_flag = buf_le.clone();
            bad_flag[3] = (ind << 1) | 0x01;
            assert!(parse_gpkg_header(&bad_flag).is_none(), "Unsupported envelope indicator must return None");
        }
    }

    #[test]
    fn test_stress_wkb_parser_types_and_endianness() {
        // 1. Point Big-Endian and Little-Endian
        let mut pt_le = vec![0x01];
        pt_le.extend_from_slice(&1u32.to_le_bytes());
        pt_le.extend_from_slice(&123.456_f64.to_le_bytes());
        pt_le.extend_from_slice(&789.012_f64.to_le_bytes());
        assert_eq!(parse_wkb(&pt_le), Some(GpkgGeometry::Point { x: 123.456, y: 789.012, z: None }));

        let mut pt_be = vec![0x00];
        pt_be.extend_from_slice(&1u32.to_be_bytes());
        pt_be.extend_from_slice(&123.456_f64.to_be_bytes());
        pt_be.extend_from_slice(&789.012_f64.to_be_bytes());
        assert_eq!(parse_wkb(&pt_be), Some(GpkgGeometry::Point { x: 123.456, y: 789.012, z: None }));

        // 2. LineString Big-Endian and Little-Endian
        let mut line_be = vec![0x00];
        line_be.extend_from_slice(&2u32.to_be_bytes()); // LineString
        line_be.extend_from_slice(&2u32.to_be_bytes()); // 2 points
        line_be.extend_from_slice(&10.0_f64.to_be_bytes());
        line_be.extend_from_slice(&20.0_f64.to_be_bytes());
        line_be.extend_from_slice(&30.0_f64.to_be_bytes());
        line_be.extend_from_slice(&40.0_f64.to_be_bytes());
        assert_eq!(parse_wkb(&line_be), Some(GpkgGeometry::LineString { points: vec![(10.0, 20.0, None), (30.0, 40.0, None)] }));

        // 3. Polygon with Outer and Inner Rings (Big-Endian & Little-Endian)
        let mut poly_le = vec![0x01];
        poly_le.extend_from_slice(&3u32.to_le_bytes()); // Polygon
        poly_le.extend_from_slice(&2u32.to_le_bytes()); // 2 rings (exterior + 1 interior hole)

        // Exterior ring: (0,0), (10,0), (10,10), (0,10), (0,0)
        poly_le.extend_from_slice(&5u32.to_le_bytes());
        for (x, y) in [(0.0_f64, 0.0_f64), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0), (0.0, 0.0)] {
            poly_le.extend_from_slice(&x.to_le_bytes());
            poly_le.extend_from_slice(&y.to_le_bytes());
        }
        // Interior ring: (2,2), (8,2), (8,8), (2,8), (2,2)
        poly_le.extend_from_slice(&5u32.to_le_bytes());
        for (x, y) in [(2.0_f64, 2.0_f64), (8.0, 2.0), (8.0, 8.0), (2.0, 8.0), (2.0, 2.0)] {
            poly_le.extend_from_slice(&x.to_le_bytes());
            poly_le.extend_from_slice(&y.to_le_bytes());
        }

        let parsed_poly = parse_wkb(&poly_le).expect("Polygon with hole must parse");
        match parsed_poly {
            GpkgGeometry::Polygon { rings } => {
                assert_eq!(rings.len(), 2);
                assert_eq!(rings[0].len(), 5);
                assert_eq!(rings[1].len(), 5);
                assert_eq!(rings[0][2], (10.0, 10.0, None));
                assert_eq!(rings[1][2], (8.0, 8.0, None));
            }
            _ => panic!("Expected polygon"),
        }

        // 4. MultiPoint
        let mut mp_le = vec![0x01];
        mp_le.extend_from_slice(&4u32.to_le_bytes()); // MultiPoint
        mp_le.extend_from_slice(&2u32.to_le_bytes()); // 2 points
        mp_le.extend_from_slice(&pt_le);
        mp_le.extend_from_slice(&pt_le);
        let parsed_mp = parse_wkb(&mp_le).expect("MultiPoint must parse");
        match parsed_mp {
            GpkgGeometry::MultiPoint { points } => assert_eq!(points.len(), 2),
            _ => panic!("Expected MultiPoint"),
        }

        // 5. MultiLineString
        let mut mls_le = vec![0x01];
        mls_le.extend_from_slice(&5u32.to_le_bytes()); // MultiLineString
        mls_le.extend_from_slice(&1u32.to_le_bytes()); // 1 line
        let mut line_le = vec![0x01];
        line_le.extend_from_slice(&2u32.to_le_bytes());
        line_le.extend_from_slice(&2u32.to_le_bytes());
        line_le.extend_from_slice(&1.0_f64.to_le_bytes());
        line_le.extend_from_slice(&2.0_f64.to_le_bytes());
        line_le.extend_from_slice(&3.0_f64.to_le_bytes());
        line_le.extend_from_slice(&4.0_f64.to_le_bytes());
        mls_le.extend_from_slice(&line_le);
        let parsed_mls = parse_wkb(&mls_le).expect("MultiLineString must parse");
        match parsed_mls {
            GpkgGeometry::MultiLineString { lines } => assert_eq!(lines.len(), 1),
            _ => panic!("Expected MultiLineString"),
        }

        // 6. MultiPolygon
        let mut mpoly_le = vec![0x01];
        mpoly_le.extend_from_slice(&6u32.to_le_bytes()); // MultiPolygon
        mpoly_le.extend_from_slice(&1u32.to_le_bytes()); // 1 polygon
        mpoly_le.extend_from_slice(&poly_le);
        let parsed_mpoly = parse_wkb(&mpoly_le).expect("MultiPolygon must parse");
        match parsed_mpoly {
            GpkgGeometry::MultiPolygon { polygons } => assert_eq!(polygons.len(), 1),
            _ => panic!("Expected MultiPolygon"),
        }
    }

    #[test]
    fn test_stress_wkb_parser_nan_inf_and_truncation() {
        // NaN in Point
        let mut nan_pt = vec![0x01];
        nan_pt.extend_from_slice(&1u32.to_le_bytes());
        nan_pt.extend_from_slice(&f64::NAN.to_le_bytes());
        nan_pt.extend_from_slice(&100.0_f64.to_le_bytes());
        assert!(parse_wkb(&nan_pt).is_none(), "Point with NaN must return None");

        // Inf in Point
        let mut inf_pt = vec![0x01];
        inf_pt.extend_from_slice(&1u32.to_le_bytes());
        inf_pt.extend_from_slice(&100.0_f64.to_le_bytes());
        inf_pt.extend_from_slice(&f64::INFINITY.to_le_bytes());
        assert!(parse_wkb(&inf_pt).is_none(), "Point with Inf must return None");

        // Neg Inf in Point
        let mut neg_inf_pt = vec![0x01];
        neg_inf_pt.extend_from_slice(&1u32.to_le_bytes());
        neg_inf_pt.extend_from_slice(&f64::NEG_INFINITY.to_le_bytes());
        neg_inf_pt.extend_from_slice(&100.0_f64.to_le_bytes());
        assert!(parse_wkb(&neg_inf_pt).is_none(), "Point with -Inf must return None");

        // Truncated coordinate buffers across all byte offsets: verify zero panic
        let mut valid_poly = vec![0x01];
        valid_poly.extend_from_slice(&3u32.to_le_bytes());
        valid_poly.extend_from_slice(&1u32.to_le_bytes());
        valid_poly.extend_from_slice(&4u32.to_le_bytes());
        for (x, y) in [(0.0_f64, 0.0_f64), (10.0, 0.0), (10.0, 10.0), (0.0, 0.0)] {
            valid_poly.extend_from_slice(&x.to_le_bytes());
            valid_poly.extend_from_slice(&y.to_le_bytes());
        }

        for i in 0..valid_poly.len() {
            let truncated = &valid_poly[..i];
            let res = parse_wkb(truncated);
            assert!(res.is_none(), "Truncated slice at {i} must return None without panic");
        }

        // MultiPoint with claimed count > 0 but truncated payload
        let mut mp_truncated = vec![0x01];
        mp_truncated.extend_from_slice(&4u32.to_le_bytes()); // MultiPoint
        mp_truncated.extend_from_slice(&10u32.to_le_bytes()); // num_geoms = 10
        let res_mp = parse_wkb(&mp_truncated);
        assert!(res_mp.is_none(), "Truncated MultiPoint must fail-fast and return None");

        let mut mls_truncated = vec![0x01];
        mls_truncated.extend_from_slice(&5u32.to_le_bytes()); // MultiLineString
        mls_truncated.extend_from_slice(&10u32.to_le_bytes());
        assert!(parse_wkb(&mls_truncated).is_none(), "Truncated MultiLineString must return None");

        let mut mpoly_truncated = vec![0x01];
        mpoly_truncated.extend_from_slice(&6u32.to_le_bytes()); // MultiPolygon
        mpoly_truncated.extend_from_slice(&10u32.to_le_bytes());
        assert!(parse_wkb(&mpoly_truncated).is_none(), "Truncated MultiPolygon must return None");
    }

    #[test]
    fn test_stress_house1_polygon_area_benchmark() {
        // Gwangnyeong-ri 1 House 1 benchmark (32.91 ㎡ ± 0.01 ㎡)
        let pts = vectorcraft_pathops::plan::HOUSE_1_POLYGON_METERS;
        let mut poly_blob = vec![0x01]; // LE
        poly_blob.extend_from_slice(&3u32.to_le_bytes()); // Polygon
        poly_blob.extend_from_slice(&1u32.to_le_bytes()); // 1 ring
        poly_blob.extend_from_slice(&((pts.len() + 1) as u32).to_le_bytes()); // +1 for closing point
        for (x, y) in pts {
            poly_blob.extend_from_slice(&x.to_le_bytes());
            poly_blob.extend_from_slice(&y.to_le_bytes());
        }
        // Closing vertex matching first vertex
        poly_blob.extend_from_slice(&pts[0].0.to_le_bytes());
        poly_blob.extend_from_slice(&pts[0].1.to_le_bytes());

        let geom = parse_wkb(&poly_blob).expect("House 1 WKB polygon must parse");
        let area = match geom {
            GpkgGeometry::Polygon { rings } => {
                let ring = &rings[0];
                let mut sum = 0.0;
                for i in 0..ring.len() - 1 {
                    sum += ring[i].0 * ring[i + 1].1 - ring[i + 1].0 * ring[i].1;
                }
                (sum / 2.0).abs()
            }
            _ => panic!("Expected Polygon"),
        };

        assert!((area - 32.91).abs() < 0.01, "House 1 12-vertex area must equal 32.91 ㎡ ± 0.01 ㎡, got {area:.4} ㎡");
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_stress_readonly_sqlite_guarantees() {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let db_path = std::env::temp_dir().join(format!("test_readonly_gpkg_{}.gpkg", now));
        create_sample_gpkg(&db_path).unwrap();

        let reader = GpkgReader::open(&db_path).unwrap();
        // 1. Verify inspect and feature reading work
        let info = reader.inspect().unwrap();
        assert!(info.read_only);
        assert_eq!(info.srs_id, 5186);

        // 2. Attempt mutation: open separate connection or try to mutate
        // With SQLITE_OPEN_READ_ONLY, trying to execute write statements fails
        let write_res = reader.conn.execute("INSERT INTO dwelling_perimeter (feature_name) VALUES ('mutation_attempt')", []);
        assert!(write_res.is_err(), "Write on read-only connection must fail");
        let err_msg = write_res.unwrap_err().to_string();
        assert!(err_msg.contains("readonly") || err_msg.contains("read-only"), "Error must specify readonly database, got: {err_msg}");

        // 3. Opening non-existent file in read-only mode must fail and NOT create a new file
        let bogus_path = std::env::temp_dir().join(format!("non_existent_{}.gpkg", now));
        assert!(!bogus_path.exists());
        let bogus_res = GpkgReader::open(&bogus_path);
        assert!(bogus_res.is_err(), "Opening non-existent file must fail");
        assert!(!bogus_path.exists(), "Read-only open must never create a file");

        // Clean up
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_stress_hatch_mapping_benchmarks() {
        // Benchmark feature types: 수혈 45°, 분묘 135°, 구 0°, 도랑 90°, 가마 ┼, 주거지 없음
        let h_pit = resolve_feature_hatch("수혈");
        assert!(h_pit.is_some(), "수혈 must have hatch");
        assert_eq!(h_pit.unwrap().direction, HatchDirection::Deg45, "수혈 must be 45°");

        let h_tomb = resolve_feature_hatch("분묘");
        assert!(h_tomb.is_some(), "분묘 must have hatch");
        assert_eq!(h_tomb.unwrap().direction, HatchDirection::Deg135, "분묘 must be 135°");

        let h_house = resolve_feature_hatch("주거지");
        assert!(h_house.is_none(), "주거지 must have no hatch (open interior)");

        let h_furrow = resolve_feature_hatch("구");
        assert!(h_furrow.is_some(), "구 must have hatch");
        assert_eq!(h_furrow.unwrap().direction, HatchDirection::Deg0, "구 must be 0°");

        let h_ditch = resolve_feature_hatch("도랑");
        assert!(h_ditch.is_some(), "도랑 must have hatch");
        assert_eq!(h_ditch.unwrap().direction, HatchDirection::Deg90, "도랑 must be 90°");

        let h_kiln = resolve_feature_hatch("가마");
        assert!(h_kiln.is_some(), "가마 must have hatch");
        assert_eq!(h_kiln.unwrap().direction, HatchDirection::Cross, "가마 must be Cross (┼)");
    }
}
