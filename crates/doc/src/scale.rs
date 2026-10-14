//! Drawing scale and real-world TM coordinate models (Trowel design 2.1 §6, §14):
//! - Drawing scale ratios: 1:1, 1:10, 1:20, 1:50, 1:100, 1:200, 1:500, etc.
//! - High-precision f64 conversions between canvas points and real-world coordinates.
//! - TM EPSG:5186 (Korea Central Belt 2010): X = East (동), Y = North (북).
//! - Document origin offset and graticule/frame ticks calculation.

use serde::{Deserialize, Serialize};
use vectorcraft_geom::{Point, Rect};

use crate::legend::MM;

/// Default archaeological plan scale: 1:20.
pub const DEFAULT_PLAN_SCALE_DENOM: f64 = 20.0;

/// Default artifact scale: 1:1.
pub const DEFAULT_FINDS_SCALE_DENOM: f64 = 1.0;

/// Standard drawing scale ratio 1:S.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScaleRatio {
    /// Denominator S of the scale ratio 1:S (e.g. 20 for 1:20, 1 for 1:1).
    pub denom: f64,
}

impl Default for ScaleRatio {
    fn default() -> Self {
        Self { denom: DEFAULT_PLAN_SCALE_DENOM }
    }
}

impl ScaleRatio {
    pub const S1_1: Self = Self { denom: 1.0 };
    pub const S1_2: Self = Self { denom: 2.0 };
    pub const S1_5: Self = Self { denom: 5.0 };
    pub const S1_10: Self = Self { denom: 10.0 };
    pub const S1_20: Self = Self { denom: 20.0 };
    pub const S1_50: Self = Self { denom: 50.0 };
    pub const S1_100: Self = Self { denom: 100.0 };
    pub const S1_200: Self = Self { denom: 200.0 };
    pub const S1_500: Self = Self { denom: 500.0 };
    pub const S1_1000: Self = Self { denom: 1000.0 };

    pub const PRESETS: [Self; 10] =
        [Self::S1_1, Self::S1_2, Self::S1_5, Self::S1_10, Self::S1_20, Self::S1_50, Self::S1_100, Self::S1_200, Self::S1_500, Self::S1_1000];

    pub fn new(denom: f64) -> Self {
        Self { denom: denom.max(0.001) }
    }

    /// Label formatted as "1:20".
    pub fn label(self) -> String {
        if (self.denom.fract()).abs() < 1e-4 { format!("1:{:.0}", self.denom) } else { format!("1:{:.2}", self.denom) }
    }

    /// Points per world meter on the paper.
    /// In world: 1 meter = 1000 mm.
    /// On paper at 1:S: 1 meter world = (1000.0 / S) mm on paper.
    /// In points: (1000.0 / S) * MM pt.
    pub fn points_per_meter(self) -> f64 {
        (1000.0 / self.denom) * MM
    }

    /// Meters per canvas point.
    pub fn meters_per_point(self) -> f64 {
        1.0 / self.points_per_meter()
    }
}

/// Coordinate system order: X = East (동), Y = North (북) matching Strata and Korean surveying practice (§14(2)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoordOrder {
    /// X = East (동), Y = North (북) - Default for Trowel (§14(2)).
    #[default]
    EastNorth,
    /// X = North (북), Y = East (동) - Alternative CAD mode.
    NorthEast,
}

/// Coordinate Reference System definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordSystem {
    /// EPSG code (e.g. 5186 for Korea Central Belt 2010).
    pub epsg: u32,
    /// Display name (e.g. "EPSG:5186 (중부원점 2010)").
    pub name: String,
    /// Coordinate axis order (default: EastNorth).
    #[serde(default)]
    pub order: CoordOrder,
}

impl Default for CoordSystem {
    fn default() -> Self {
        Self { epsg: 5186, name: "EPSG:5186 (중부원점 2010)".to_string(), order: CoordOrder::EastNorth }
    }
}

/// Document-level scale and real-world TM coordinate configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScaleConfig {
    /// Drawing scale ratio (e.g. 1:20).
    #[serde(default)]
    pub scale: ScaleRatio,
    /// The coordinate system (default: EPSG:5186).
    #[serde(default)]
    pub crs: CoordSystem,
    /// Canvas point corresponding to the origin (in document points).
    #[serde(default)]
    pub canvas_origin: Point,
    /// Real-world TM coordinates corresponding to the origin (East/X, North/Y in meters).
    #[serde(default)]
    pub world_origin: (f64, f64),
}

impl Default for ScaleConfig {
    fn default() -> Self {
        Self { scale: ScaleRatio::default(), crs: CoordSystem::default(), canvas_origin: Point::new(0.0, 0.0), world_origin: (0.0, 0.0) }
    }
}

impl ScaleConfig {
    pub fn new(scale_denom: f64) -> Self {
        Self { scale: ScaleRatio::new(scale_denom), crs: CoordSystem::default(), canvas_origin: Point::new(0.0, 0.0), world_origin: (0.0, 0.0) }
    }

    /// Sets the real-world control origin: maps `canvas_pt` to world `(east_m, north_m)`.
    pub fn set_origin(&mut self, canvas_pt: Point, east_m: f64, north_m: f64) {
        self.canvas_origin = canvas_pt;
        self.world_origin = (east_m, north_m);
    }

    /// Converts canvas point (x, y) to TM real-world coordinates (East, North) in meters.
    /// Note: North is UP (decreasing y in canvas coordinates), East is RIGHT (increasing x in canvas coordinates).
    pub fn canvas_to_world(&self, canvas: Point) -> (f64, f64) {
        let ppm = self.scale.points_per_meter();
        let dx = canvas.x - self.canvas_origin.x;
        let dy = canvas.y - self.canvas_origin.y;

        let east = self.world_origin.0 + dx / ppm;
        let north = self.world_origin.1 - dy / ppm;
        (east, north)
    }

    /// Converts TM real-world coordinates (East, North) in meters to canvas point (x, y).
    pub fn world_to_canvas(&self, east: f64, north: f64) -> Point {
        let ppm = self.scale.points_per_meter();
        let dx = (east - self.world_origin.0) * ppm;
        let dy = (north - self.world_origin.1) * ppm;

        Point::new(self.canvas_origin.x + dx, self.canvas_origin.y - dy)
    }

    /// Formats live coordinates according to `CoordOrder` (§14(2)):
    /// e.g. ("152840.123", "512340.456") for X (East) and Y (North).
    pub fn format_coords(&self, canvas: Point) -> (String, String) {
        let (east, north) = self.canvas_to_world(canvas);
        match self.crs.order {
            CoordOrder::EastNorth => (format!("{:.3}", east), format!("{:.3}", north)),
            CoordOrder::NorthEast => (format!("{:.3}", north), format!("{:.3}", east)),
        }
    }
}

/// A graticule tick mark along the boundary of a drawing sheet.
#[derive(Clone, Debug, PartialEq)]
pub struct GraticuleTick {
    /// Canvas position of the tick mark base.
    pub pos: Point,
    /// Normal vector pointing outward or along the tick (length in points, e.g. 4.0 pt).
    pub tick_vector: (f64, f64),
    /// Real-world coordinate value in meters (e.g. 152840.0).
    pub world_val: f64,
    /// Axis: "X" (East) or "Y" (North).
    pub axis: &'static str,
    /// Formatted label for the tick (e.g. "152840").
    pub label: String,
}

/// Computes graticule tick marks for a bounding rectangle at given meter interval (e.g. 5.0m or 10.0m).
pub fn compute_graticule_ticks(bounds: Rect, config: &ScaleConfig, interval_m: f64) -> Vec<GraticuleTick> {
    if interval_m <= 0.001 {
        return Vec::new();
    }
    let mut ticks = Vec::new();

    let (min_e, max_n) = config.canvas_to_world(Point::new(bounds.x0, bounds.y0));
    let (max_e, min_n) = config.canvas_to_world(Point::new(bounds.x1, bounds.y1));

    let (e0, e1) = if min_e < max_e { (min_e, max_e) } else { (max_e, min_e) };
    let (n0, n1) = if min_n < max_n { (min_n, max_n) } else { (max_n, min_n) };

    // Ticks along East (X): vertical grid lines intersecting top (y0) and bottom (y1)
    let start_e = (e0 / interval_m).floor() * interval_m;
    let mut cur_e = start_e;
    while cur_e <= e1 + 1e-4 {
        if cur_e >= e0 - 1e-4 {
            let pt_top = config.world_to_canvas(cur_e, max_n);
            let pt_bottom = config.world_to_canvas(cur_e, min_n);
            if pt_top.x >= bounds.x0 - 1e-4 && pt_top.x <= bounds.x1 + 1e-4 {
                ticks.push(GraticuleTick {
                    pos: Point::new(pt_top.x, bounds.y0),
                    tick_vector: (0.0, -4.0),
                    world_val: cur_e,
                    axis: "X",
                    label: format!("{:.0}", cur_e),
                });
                ticks.push(GraticuleTick {
                    pos: Point::new(pt_bottom.x, bounds.y1),
                    tick_vector: (0.0, 4.0),
                    world_val: cur_e,
                    axis: "X",
                    label: format!("{:.0}", cur_e),
                });
            }
        }
        cur_e += interval_m;
    }

    // Ticks along North (Y): horizontal grid lines intersecting left (x0) and right (x1)
    let start_n = (n0 / interval_m).floor() * interval_m;
    let mut cur_n = start_n;
    while cur_n <= n1 + 1e-4 {
        if cur_n >= n0 - 1e-4 {
            let pt_left = config.world_to_canvas(min_e, cur_n);
            let pt_right = config.world_to_canvas(max_e, cur_n);
            if pt_left.y >= bounds.y0 - 1e-4 && pt_left.y <= bounds.y1 + 1e-4 {
                ticks.push(GraticuleTick {
                    pos: Point::new(bounds.x0, pt_left.y),
                    tick_vector: (-4.0, 0.0),
                    world_val: cur_n,
                    axis: "Y",
                    label: format!("{:.0}", cur_n),
                });
                ticks.push(GraticuleTick {
                    pos: Point::new(bounds.x1, pt_right.y),
                    tick_vector: (4.0, 0.0),
                    world_val: cur_n,
                    axis: "Y",
                    label: format!("{:.0}", cur_n),
                });
            }
        }
        cur_n += interval_m;
    }

    ticks
}

#[cfg(test)]
mod tests {
    use super::*;

    const GWANGNYEONGRI_CP1: (f64, f64, f64) = (152840.123, 512340.456, 152.45);
    const GWANGNYEONGRI_CP2: (f64, f64, f64) = (152860.789, 512340.123, 152.60);
    const GWANGNYEONGRI_CP3: (f64, f64, f64) = (152860.456, 512360.890, 152.80);
    const GWANGNYEONGRI_CP4: (f64, f64, f64) = (152840.012, 512360.234, 152.55);

    #[test]
    fn test_scale_ratio_points_per_meter() {
        let scale_1_20 = ScaleRatio::new(20.0);
        // 1 meter = 1000 mm. At 1:20 => 50 mm paper = 50 * (72.0 / 25.4) pt
        let expected_ppm = 50.0 * (72.0 / 25.4);
        assert!((scale_1_20.points_per_meter() - expected_ppm).abs() < 1e-9);
        assert_eq!(scale_1_20.label(), "1:20");

        let scale_1_1 = ScaleRatio::new(1.0);
        assert_eq!(scale_1_1.label(), "1:1");
        assert!((scale_1_1.points_per_meter() - 1000.0 * (72.0 / 25.4)).abs() < 1e-9);
    }

    #[test]
    fn test_gwangnyeongri_control_points_round_trip() {
        let mut cfg = ScaleConfig::new(20.0);
        // Bind CP1 to canvas (100.0, 100.0)
        let canvas_cp1 = Point::new(100.0, 100.0);
        cfg.set_origin(canvas_cp1, GWANGNYEONGRI_CP1.0, GWANGNYEONGRI_CP1.1);

        // CP1 must recover exactly
        let (e1, n1) = cfg.canvas_to_world(canvas_cp1);
        assert!((e1 - GWANGNYEONGRI_CP1.0).abs() < 1e-9);
        assert!((n1 - GWANGNYEONGRI_CP1.1).abs() < 1e-9);

        // CP2, CP3, CP4 forward and inverse
        for cp in [GWANGNYEONGRI_CP2, GWANGNYEONGRI_CP3, GWANGNYEONGRI_CP4] {
            let canvas_pt = cfg.world_to_canvas(cp.0, cp.1);
            let (recovered_e, recovered_n) = cfg.canvas_to_world(canvas_pt);
            assert!((recovered_e - cp.0).abs() < 1e-6);
            assert!((recovered_n - cp.1).abs() < 1e-6);
        }
    }

    #[test]
    fn test_coordinate_orientation_east_north() {
        let mut cfg = ScaleConfig::new(20.0);
        cfg.set_origin(Point::new(0.0, 0.0), 1000.0, 2000.0);

        // Moving right (+x) on canvas must increase East
        let (e_right, n_same) = cfg.canvas_to_world(Point::new(100.0, 0.0));
        assert!(e_right > 1000.0);
        assert!((n_same - 2000.0).abs() < 1e-9);

        // Moving up (-y) on canvas must increase North
        let (e_same, n_up) = cfg.canvas_to_world(Point::new(0.0, -100.0));
        assert!((e_same - 1000.0).abs() < 1e-9);
        assert!(n_up > 2000.0);
    }

    #[test]
    fn test_graticule_ticks_generation() {
        let mut cfg = ScaleConfig::new(20.0);
        cfg.set_origin(Point::new(0.0, 0.0), 152840.0, 512340.0);

        let bounds = Rect::new(0.0, -500.0, 500.0, 0.0);
        let ticks = compute_graticule_ticks(bounds, &cfg, 1.0);
        assert!(!ticks.is_empty());
        assert!(ticks.iter().any(|t| t.axis == "X"));
        assert!(ticks.iter().any(|t| t.axis == "Y"));
    }
}
