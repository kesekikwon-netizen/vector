//! 3-point rim circumcircle fitting for archaeological pottery drafting (Trowel design 2.1 §4-3, §13, J011).
//!
//! Computes circumcircle radius R = abc / (4A) and diameter D = 2R from 3 rim shard edge points,
//! with strict collinearity detection (|A| < 1e-9) and arc preservation percentage calculation.

use kurbo::Point;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Minimum absolute triangle area (|A| >= 1e-9) required to avoid collinear / infinite radius degeneracy.
pub const EPSILON_COLLINEAR: f64 = 1e-9;
/// Minimum distance between distinct rim points.
pub const EPSILON_DISTANCE: f64 = 1e-9;

/// Errors that can occur during 3-point circle fitting.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CircleFitError {
    #[error("points are collinear; signed area is too small: {area:e}")]
    CollinearPoints { area: f64 },
    #[error("points are degenerate or coincident: distance between points is too small")]
    DegeneratePoints,
    #[error("point coordinates must be finite")]
    InvalidCoordinates,
}

/// Result of 3-point circumcircle fitting.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Circle3Point {
    pub center: Point,
    pub radius: f64,
    pub diameter: f64,
    pub signed_area: f64,
    pub arc_span_deg: f64,
    pub rim_preservation_percent: f64,
}

impl Circle3Point {
    /// Diameter in millimeters assuming input points were in PostScript points (72 pt / inch).
    pub fn diameter_mm(&self) -> f64 {
        self.diameter * (25.4 / 72.0)
    }

    /// Diameter in centimeters assuming input points were in PostScript points.
    pub fn diameter_cm(&self) -> f64 {
        self.diameter_mm() / 10.0
    }
}

/// Fit a circle through three distinct points p1, p2, p3 along a circular arc.
///
/// Uses the circumcircle formula:
/// - $R = \frac{a \cdot b \cdot c}{4 |A|}$
/// - $D = 2R = \frac{a \cdot b \cdot c}{2 |A|}$
///
/// where $a, b, c$ are side lengths and $A$ is the triangle area.
///
/// Returns `Err(CircleFitError::CollinearPoints)` if $|A| < 10^{-9}$.
pub fn fit_circle_3point(p1: Point, p2: Point, p3: Point) -> Result<Circle3Point, CircleFitError> {
    if !p1.x.is_finite() || !p1.y.is_finite() || !p2.x.is_finite() || !p2.y.is_finite() || !p3.x.is_finite() || !p3.y.is_finite() {
        return Err(CircleFitError::InvalidCoordinates);
    }

    let v12_x = p2.x - p1.x;
    let v12_y = p2.y - p1.y;
    let v13_x = p3.x - p1.x;
    let v13_y = p3.y - p1.y;
    let v23_x = p3.x - p2.x;
    let v23_y = p3.y - p2.y;

    let c = (v12_x * v12_x + v12_y * v12_y).sqrt();
    let b = (v13_x * v13_x + v13_y * v13_y).sqrt();
    let a = (v23_x * v23_x + v23_y * v23_y).sqrt();

    if c < EPSILON_DISTANCE || b < EPSILON_DISTANCE || a < EPSILON_DISTANCE {
        return Err(CircleFitError::DegeneratePoints);
    }

    // Signed cross product (twice signed area: 2A)
    let two_a = v12_x * v13_y - v12_y * v13_x;
    let area = 0.5 * two_a.abs();

    if area < EPSILON_COLLINEAR {
        return Err(CircleFitError::CollinearPoints { area });
    }

    let radius = (a * b * c) / (4.0 * area);
    let diameter = 2.0 * radius;

    // Circumcenter (cx, cy) using perpendicular bisector intersection
    let d1 = p1.x * p1.x + p1.y * p1.y;
    let d2 = p2.x * p2.x + p2.y * p2.y;
    let d3 = p3.x * p3.x + p3.y * p3.y;
    let delta = 2.0 * two_a;

    let cx = (d1 * (p2.y - p3.y) + d2 * (p3.y - p1.y) + d3 * (p1.y - p2.y)) / delta;
    let cy = (d1 * (p3.x - p2.x) + d2 * (p1.x - p3.x) + d3 * (p2.x - p1.x)) / delta;
    let center = Point::new(cx, cy);

    // Compute angular span passing from p1 through p2 to p3
    let theta1 = (p1.y - cy).atan2(p1.x - cx);
    let theta2 = (p2.y - cy).atan2(p2.x - cx);
    let theta3 = (p3.y - cy).atan2(p3.x - cx);

    let norm_angle = |mut ang: f64| -> f64 {
        const TWO_PI: f64 = std::f64::consts::TAU;
        ang %= TWO_PI;
        if ang < 0.0 {
            ang += TWO_PI;
        }
        ang
    };

    // Determine direction from p1 to p2
    let ccw_12 = norm_angle(theta2 - theta1);
    let ccw_23 = norm_angle(theta3 - theta2);
    let total_ccw = ccw_12 + ccw_23;

    let arc_span_rad = if total_ccw < std::f64::consts::TAU {
        total_ccw
    } else {
        // Going clockwise
        let cw_12 = norm_angle(theta1 - theta2);
        let cw_23 = norm_angle(theta2 - theta3);
        cw_12 + cw_23
    };

    let arc_span_deg = arc_span_rad.to_degrees().min(360.0);
    let rim_preservation_percent = (arc_span_deg / 360.0) * 100.0;

    Ok(Circle3Point { center, radius, diameter, signed_area: 0.5 * two_a, arc_span_deg, rim_preservation_percent })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_circle_points() {
        let p1 = Point::new(1.0, 0.0);
        let p2 = Point::new(0.0, 1.0);
        let p3 = Point::new(-1.0, 0.0);
        let fit = fit_circle_3point(p1, p2, p3).unwrap();
        assert!((fit.center.x).abs() < 1e-9);
        assert!((fit.center.y).abs() < 1e-9);
        assert!((fit.radius - 1.0).abs() < 1e-9);
        assert!((fit.diameter - 2.0).abs() < 1e-9);
        assert!((fit.arc_span_deg - 180.0).abs() < 1e-6);
        assert!((fit.rim_preservation_percent - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_translated_scaled_circle() {
        // Center (100, 200), radius 50, diameter 100
        let c = Point::new(100.0, 200.0);
        let r = 50.0;
        let p1 = Point::new(c.x + r, c.y);
        let p2 = Point::new(c.x, c.y + r);
        let p3 = Point::new(c.x - r, c.y);
        let fit = fit_circle_3point(p1, p2, p3).unwrap();
        assert!((fit.center.x - 100.0).abs() < 1e-9);
        assert!((fit.center.y - 200.0).abs() < 1e-9);
        assert!((fit.radius - 50.0).abs() < 1e-9);
        assert!((fit.diameter - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_spec_rim_diameter_benchmark() {
        // Pottery spec: rim diameter 18.4 cm = 184.0 mm
        let r_mm = 92.0;
        let ang1 = 20.0_f64.to_radians();
        let ang2 = 45.0_f64.to_radians();
        let ang3 = 70.0_f64.to_radians();
        let p1 = Point::new(r_mm * ang1.cos(), r_mm * ang1.sin());
        let p2 = Point::new(r_mm * ang2.cos(), r_mm * ang2.sin());
        let p3 = Point::new(r_mm * ang3.cos(), r_mm * ang3.sin());
        let fit = fit_circle_3point(p1, p2, p3).unwrap();
        assert!((fit.diameter - 184.0).abs() < 1e-6);
        assert!((fit.arc_span_deg - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_collinear_returns_error() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(5.0, 5.0);
        let p3 = Point::new(10.0, 10.0);
        let err = fit_circle_3point(p1, p2, p3).unwrap_err();
        match err {
            CircleFitError::CollinearPoints { area } => assert!(area < 1e-9),
            other => panic!("expected CollinearPoints, got {:?}", other),
        }
    }

    #[test]
    fn test_degenerate_points() {
        let p1 = Point::new(1.0, 1.0);
        let p2 = Point::new(1.0, 1.0);
        let p3 = Point::new(2.0, 2.0);
        assert_eq!(fit_circle_3point(p1, p2, p3), Err(CircleFitError::DegeneratePoints));
    }

    #[test]
    fn test_non_finite_points() {
        let p1 = Point::new(f64::NAN, 1.0);
        let p2 = Point::new(1.0, 2.0);
        let p3 = Point::new(2.0, 3.0);
        assert_eq!(fit_circle_3point(p1, p2, p3), Err(CircleFitError::InvalidCoordinates));
    }
}
