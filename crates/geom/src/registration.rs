//! Underlay image calibration and registration algorithms (Trowel design 2.1 §1.2, §3, §5, §13, J008).
//!
//! Provides:
//! - 2-point grid scale and rotation calibration (`fit_two_point`)
//! - N-point similarity least-squares fit (`fit_similarity`, 4 DOF)
//! - N-point affine least-squares fit (`fit_affine`, 6 DOF) with collinearity detection
//! - Comprehensive error residual tracking and RMSE computation in points, meters, and millimeters.

use kurbo::{Affine, Point, Vec2};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Conversion factor: 1 mm in PostScript points (72 pt / 25.4 mm).
pub const MM: f64 = 72.0 / 25.4;

/// Errors that can occur during registration or calibration.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RegistrationError {
    #[error("insufficient points: requires at least {required}, but got {actual}")]
    InsufficientPoints { required: usize, actual: usize },
    #[error("source points are collinear (det < 1e-12); affine fit is underdetermined")]
    CollinearPoints,
    #[error("source points are coincident or degenerate")]
    DegeneratePoints,
    #[error("coordinates must be finite")]
    InvalidCoordinates,
}

/// A corresponding pair of source (underlay/image) and target (canvas/world) points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointPair {
    pub source: Point,
    pub target: Point,
}

impl PointPair {
    pub fn new(source: Point, target: Point) -> Self {
        Self { source, target }
    }
}

/// Axis alignment preference for 2-point calibration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AxisAlignment {
    /// Align the line between points horizontally (+X East).
    #[default]
    Horizontal,
    /// Align the line between points vertically (-Y North).
    Vertical,
    /// Do not apply extra rotational alignment (only rotate to match target vector if given).
    None,
}

/// Result of 2-point calibration.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TwoPointResult {
    pub scale: f64,
    pub rotation_rad: f64,
    pub rotation_deg: f64,
    pub translation: Vec2,
    pub transform: Affine,
    pub measured_dist: f64,
    pub target_dist: f64,
}

/// Result of affine or similarity least-squares registration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegistrationResult {
    /// The resulting 2D affine transformation mapping source coordinates to target coordinates.
    pub transform: Affine,
    /// Matrix coefficients [a, b, c, d, e, f] in kurbo Affine layout.
    pub affine_coeffs: [f64; 6],
    /// Approximate scale along X and Y axes.
    pub scale: (f64, f64),
    /// Rotation angle in radians and degrees.
    pub rotation_rad: f64,
    pub rotation_deg: f64,
    /// Translation (tx, ty).
    pub translation: Vec2,
    /// Residuals for each control point pair.
    pub residuals: Vec<PointResidual>,
    /// Root Mean Square Error in canvas points.
    pub rmse_pt: f64,
    /// Root Mean Square Error in meters (if points_per_meter was provided).
    pub rmse_m: Option<f64>,
    /// Root Mean Square Error in millimeters (on paper).
    pub rmse_mm: f64,
    /// Maximum residual distance in points.
    pub max_error_pt: f64,
    /// Index of the control point with the maximum error.
    pub max_error_index: usize,
}

/// Detailed residual error for an individual control point.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointResidual {
    pub index: usize,
    pub source: Point,
    pub target: Point,
    pub mapped: Point,
    pub dx: f64,
    pub dy: f64,
    pub dist_pt: f64,
    pub dist_m: Option<f64>,
    pub dist_mm: f64,
}

/// Calibrates uniform scale, rotation, and translation from two known points.
///
/// If `target_p1` and `target_p2` are provided:
/// Maps `p1 -> target_p1` and aligns the vector `p2 - p1` with `target_p2 - target_p1`.
///
/// If only `target_dist` is provided:
/// Computes scale `target_dist / measured_dist` and aligns according to `align_axis`.
/// If `target_p1` is also given, anchors `p1` to `target_p1`; otherwise keeps `p1` stationary.
pub fn fit_two_point(
    p1: Point,
    p2: Point,
    target_dist: Option<f64>,
    target_p1: Option<Point>,
    target_p2: Option<Point>,
    align_axis: AxisAlignment,
) -> Result<TwoPointResult, RegistrationError> {
    if !p1.x.is_finite() || !p1.y.is_finite() || !p2.x.is_finite() || !p2.y.is_finite() {
        return Err(RegistrationError::InvalidCoordinates);
    }

    let src_vec = p2 - p1;
    let measured_dist = src_vec.hypot();
    if measured_dist < 1e-9 {
        return Err(RegistrationError::DegeneratePoints);
    }

    let src_angle = src_vec.y.atan2(src_vec.x);

    let (target_len, theta, anchor_target) = match (target_p1, target_p2) {
        (Some(q1), Some(q2)) => {
            if !q1.x.is_finite() || !q1.y.is_finite() || !q2.x.is_finite() || !q2.y.is_finite() {
                return Err(RegistrationError::InvalidCoordinates);
            }
            let tgt_vec = q2 - q1;
            let len = tgt_vec.hypot();
            if len < 1e-9 {
                return Err(RegistrationError::DegeneratePoints);
            }
            let tgt_angle = tgt_vec.y.atan2(tgt_vec.x);
            let d_theta = tgt_angle - src_angle;
            (len, d_theta, q1)
        }
        (q1_opt, _) => {
            let Some(len) = target_dist else {
                return Err(RegistrationError::InsufficientPoints { required: 1, actual: 0 });
            };
            if !len.is_finite() || len <= 0.0 {
                return Err(RegistrationError::InvalidCoordinates);
            }
            let d_theta = match align_axis {
                AxisAlignment::Horizontal => -src_angle,
                AxisAlignment::Vertical => -std::f64::consts::FRAC_PI_2 - src_angle,
                AxisAlignment::None => 0.0,
            };
            let anchor = q1_opt.unwrap_or(p1);
            (len, d_theta, anchor)
        }
    };

    let scale = target_len / measured_dist;
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    let a = scale * cos_t;
    let b = scale * sin_t;

    // Translation to map p1 to anchor_target:
    let tx = anchor_target.x - (a * p1.x - b * p1.y);
    let ty = anchor_target.y - (b * p1.x + a * p1.y);

    let xf = Affine::new([a, b, -b, a, tx, ty]);

    Ok(TwoPointResult {
        scale,
        rotation_rad: theta,
        rotation_deg: theta.to_degrees(),
        translation: Vec2::new(tx, ty),
        transform: xf,
        measured_dist,
        target_dist: target_len,
    })
}

/// N-Point Helmert 2D Similarity Registration ($N \ge 2$, 4 DOF: scale, rotation, translation).
/// Preserves isotropic aspect ratio and orthogonal angles.
pub fn fit_similarity(pairs: &[PointPair], points_per_meter: Option<f64>) -> Result<RegistrationResult, RegistrationError> {
    let n = pairs.len();
    if n < 2 {
        return Err(RegistrationError::InsufficientPoints { required: 2, actual: n });
    }

    for p in pairs {
        if !p.source.x.is_finite() || !p.source.y.is_finite() || !p.target.x.is_finite() || !p.target.y.is_finite() {
            return Err(RegistrationError::InvalidCoordinates);
        }
    }

    let nf = n as f64;
    let mut mean_u = 0.0;
    let mut mean_v = 0.0;
    let mut mean_x = 0.0;
    let mut mean_y = 0.0;

    for p in pairs {
        mean_u += p.source.x;
        mean_v += p.source.y;
        mean_x += p.target.x;
        mean_y += p.target.y;
    }
    mean_u /= nf;
    mean_v /= nf;
    mean_x /= nf;
    mean_y /= nf;

    let mut s_uu_vv = 0.0;
    let mut s_ux_vy = 0.0;
    let mut s_uy_vx = 0.0;

    for p in pairs {
        let u_prime = p.source.x - mean_u;
        let v_prime = p.source.y - mean_v;
        let x_prime = p.target.x - mean_x;
        let y_prime = p.target.y - mean_y;

        s_uu_vv += u_prime * u_prime + v_prime * v_prime;
        s_ux_vy += u_prime * x_prime + v_prime * y_prime;
        s_uy_vx += u_prime * y_prime - v_prime * x_prime;
    }

    if s_uu_vv < 1e-12 {
        return Err(RegistrationError::DegeneratePoints);
    }

    let a = s_ux_vy / s_uu_vv;
    let b = s_uy_vx / s_uu_vv;
    let tx = mean_x - (a * mean_u - b * mean_v);
    let ty = mean_y - (b * mean_u + a * mean_v);

    let xf = Affine::new([a, b, -b, a, tx, ty]);
    let scale_val = (a * a + b * b).sqrt();
    let rot_rad = b.atan2(a);

    compute_residuals(xf, pairs, (scale_val, scale_val), rot_rad, Vec2::new(tx, ty), points_per_meter)
}

/// N-Point 6-Parameter Affine Registration ($N \ge 3$, 6 DOF).
/// Supports anisotropic scaling, shearing, rotation, and translation.
/// Checks for collinearity ($\det(\mathbf{C}) < 10^{-12}$).
pub fn fit_affine(pairs: &[PointPair], points_per_meter: Option<f64>) -> Result<RegistrationResult, RegistrationError> {
    let n = pairs.len();
    if n < 3 {
        return Err(RegistrationError::InsufficientPoints { required: 3, actual: n });
    }

    for p in pairs {
        if !p.source.x.is_finite() || !p.source.y.is_finite() || !p.target.x.is_finite() || !p.target.y.is_finite() {
            return Err(RegistrationError::InvalidCoordinates);
        }
    }

    let nf = n as f64;
    let mut mean_u = 0.0;
    let mut mean_v = 0.0;
    let mut mean_x = 0.0;
    let mut mean_y = 0.0;

    for p in pairs {
        mean_u += p.source.x;
        mean_v += p.source.y;
        mean_x += p.target.x;
        mean_y += p.target.y;
    }
    mean_u /= nf;
    mean_v /= nf;
    mean_x /= nf;
    mean_y /= nf;

    let mut s_uu = 0.0;
    let mut s_vv = 0.0;
    let mut s_uv = 0.0;
    let mut s_ux = 0.0;
    let mut s_vx = 0.0;
    let mut s_uy = 0.0;
    let mut s_vy = 0.0;

    for p in pairs {
        let u_prime = p.source.x - mean_u;
        let v_prime = p.source.y - mean_v;
        let x_prime = p.target.x - mean_x;
        let y_prime = p.target.y - mean_y;

        s_uu += u_prime * u_prime;
        s_vv += v_prime * v_prime;
        s_uv += u_prime * v_prime;

        s_ux += u_prime * x_prime;
        s_vx += v_prime * x_prime;
        s_uy += u_prime * y_prime;
        s_vy += v_prime * y_prime;
    }

    let det = s_uu * s_vv - s_uv * s_uv;
    if det.abs() < 1e-12 {
        return Err(RegistrationError::CollinearPoints);
    }

    let inv_det = 1.0 / det;
    let a11 = inv_det * (s_vv * s_ux - s_uv * s_vx);
    let a12 = inv_det * (-s_uv * s_ux + s_uu * s_vx);

    let a21 = inv_det * (s_vv * s_uy - s_uv * s_vy);
    let a22 = inv_det * (-s_uv * s_uy + s_uu * s_vy);

    let tx = mean_x - (a11 * mean_u + a12 * mean_v);
    let ty = mean_y - (a21 * mean_u + a22 * mean_v);

    let xf = Affine::new([a11, a21, a12, a22, tx, ty]);

    let sx = (a11 * a11 + a21 * a21).sqrt();
    let sy = (a12 * a12 + a22 * a22).sqrt();
    let rot_rad = a21.atan2(a11);

    compute_residuals(xf, pairs, (sx, sy), rot_rad, Vec2::new(tx, ty), points_per_meter)
}

/// Automatically fits affine transform if possible (N >= 3 and not collinear);
/// otherwise falls back to similarity transform.
pub fn fit_auto(pairs: &[PointPair], points_per_meter: Option<f64>) -> Result<RegistrationResult, RegistrationError> {
    if pairs.len() >= 3 {
        match fit_affine(pairs, points_per_meter) {
            Ok(res) => return Ok(res),
            Err(RegistrationError::CollinearPoints) => {}
            Err(e) => return Err(e),
        }
    }
    fit_similarity(pairs, points_per_meter)
}

/// Evaluates per-point residuals and RMSE given an existing transform matrix and point pairs.
pub fn evaluate_residuals(transform: Affine, pairs: &[PointPair], points_per_meter: Option<f64>) -> Result<RegistrationResult, RegistrationError> {
    if pairs.is_empty() {
        return Err(RegistrationError::InsufficientPoints { required: 1, actual: 0 });
    }
    let coeffs = transform.as_coeffs();
    let sx = (coeffs[0] * coeffs[0] + coeffs[1] * coeffs[1]).sqrt();
    let sy = (coeffs[2] * coeffs[2] + coeffs[3] * coeffs[3]).sqrt();
    let rot_rad = coeffs[1].atan2(coeffs[0]);
    let translation = Vec2::new(coeffs[4], coeffs[5]);
    compute_residuals(transform, pairs, (sx, sy), rot_rad, translation, points_per_meter)
}

fn compute_residuals(
    xf: Affine,
    pairs: &[PointPair],
    scale: (f64, f64),
    rot_rad: f64,
    translation: Vec2,
    points_per_meter: Option<f64>,
) -> Result<RegistrationResult, RegistrationError> {
    let mut residuals = Vec::with_capacity(pairs.len());
    let mut sum_sq_err = 0.0;
    let mut max_err_pt = -1.0;
    let mut max_err_idx = 0;

    for (i, p) in pairs.iter().enumerate() {
        let mapped = xf * p.source;
        let dx = mapped.x - p.target.x;
        let dy = mapped.y - p.target.y;
        let dist_pt = dx.hypot(dy);

        sum_sq_err += dist_pt * dist_pt;
        if dist_pt > max_err_pt {
            max_err_pt = dist_pt;
            max_err_idx = i;
        }

        let dist_m = points_per_meter.map(|ppm| dist_pt / ppm);
        let dist_mm = dist_pt / MM;

        residuals.push(PointResidual { index: i, source: p.source, target: p.target, mapped, dx, dy, dist_pt, dist_m, dist_mm });
    }

    let n = pairs.len() as f64;
    let rmse_pt = (sum_sq_err / n).sqrt();
    let rmse_m = points_per_meter.map(|ppm| rmse_pt / ppm);
    let rmse_mm = rmse_pt / MM;

    let coeffs = xf.as_coeffs();

    Ok(RegistrationResult {
        transform: xf,
        affine_coeffs: coeffs,
        scale,
        rotation_rad: rot_rad,
        rotation_deg: rot_rad.to_degrees(),
        translation,
        residuals,
        rmse_pt,
        rmse_m,
        rmse_mm,
        max_error_pt: max_err_pt.max(0.0),
        max_error_index: max_err_idx,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_two_point_exact() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(110.0, 20.0); // dist = 100
        let target_dist = 200.0; // scale = 2.0

        let res = fit_two_point(p1, p2, Some(target_dist), None, None, AxisAlignment::Horizontal).expect("two point fit");
        assert!((res.scale - 2.0).abs() < 1e-9);
        assert!(res.rotation_deg.abs() < 1e-9);

        let mapped_p1 = res.transform * p1;
        let mapped_p2 = res.transform * p2;
        assert!((mapped_p1.distance(p1)).abs() < 1e-9);
        assert!((mapped_p1.distance(mapped_p2) - 200.0).abs() < 1e-9);
    }

    #[test]
    fn test_fit_two_point_with_targets() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(100.0, 0.0);
        let q1 = Point::new(50.0, 50.0);
        let q2 = Point::new(50.0, 150.0); // rotated 90 deg, len 100

        let res = fit_two_point(p1, p2, None, Some(q1), Some(q2), AxisAlignment::None).expect("two point with targets");
        assert!((res.scale - 1.0).abs() < 1e-9);
        assert!((res.rotation_deg - 90.0).abs() < 1e-9);

        assert!((res.transform * p1).distance(q1) < 1e-9);
        assert!((res.transform * p2).distance(q2) < 1e-9);
    }

    #[test]
    fn test_fit_similarity_synthetic() {
        // True transform: scale=1.5, rot=30 deg, tx=100, ty=-50
        let theta = 30.0f64.to_radians();
        let s = 1.5;
        let ground_truth = Affine::new([s * theta.cos(), s * theta.sin(), -s * theta.sin(), s * theta.cos(), 100.0, -50.0]);

        let pts = [Point::new(10.0, 10.0), Point::new(80.0, 20.0), Point::new(40.0, 90.0), Point::new(120.0, 110.0)];

        let pairs: Vec<PointPair> = pts.iter().map(|&p| PointPair::new(p, ground_truth * p)).collect();

        let res = fit_similarity(&pairs, Some(100.0)).expect("similarity fit");
        assert!(res.rmse_pt < 1e-9);
        assert!((res.scale.0 - 1.5).abs() < 1e-9);
        assert!((res.rotation_deg - 30.0).abs() < 1e-9);
        assert!((res.translation.x - 100.0).abs() < 1e-9);
        assert!((res.translation.y - (-50.0)).abs() < 1e-9);
    }

    #[test]
    fn test_fit_affine_synthetic() {
        // Non-uniform scaling + shear + rotation + translation
        let ground_truth = Affine::new([1.050, 0.080, -0.060, 0.980, 150.0, -80.0]);

        // Gwangnyeong-ri 1 control point layout (in canvas space)
        let pts = [Point::new(100.0, 100.0), Point::new(500.0, 120.0), Point::new(490.0, 480.0), Point::new(110.0, 470.0)];

        let pairs: Vec<PointPair> = pts.iter().map(|&p| PointPair::new(p, ground_truth * p)).collect();

        let res = fit_affine(&pairs, Some(141.732)).expect("affine fit");
        assert!(res.rmse_pt < 1e-9);
        assert!(res.max_error_pt < 1e-9);
        for i in 0..6 {
            assert!((res.affine_coeffs[i] - ground_truth.as_coeffs()[i]).abs() < 1e-9);
        }
    }

    #[test]
    fn test_collinear_error() {
        // Collinear source points
        let pairs = vec![
            PointPair::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)),
            PointPair::new(Point::new(10.0, 10.0), Point::new(20.0, 20.0)),
            PointPair::new(Point::new(20.0, 20.0), Point::new(30.0, 30.0)),
        ];

        let err = fit_affine(&pairs, None).unwrap_err();
        assert_eq!(err, RegistrationError::CollinearPoints);

        // But fit_auto falls back gracefully to similarity!
        let auto_res = fit_auto(&pairs, None).expect("fit_auto fallback");
        assert!(auto_res.rmse_pt < 1e-9);
    }

    #[test]
    fn test_degenerate_points() {
        let pairs = vec![PointPair::new(Point::new(5.0, 5.0), Point::new(10.0, 10.0)), PointPair::new(Point::new(5.0, 5.0), Point::new(20.0, 20.0))];
        let err = fit_similarity(&pairs, None).unwrap_err();
        assert_eq!(err, RegistrationError::DegeneratePoints);
    }
}
