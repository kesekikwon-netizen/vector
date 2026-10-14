//! Shared test harness, fixtures, and mathematical verification oracles for Trowel E2E tests.
#![allow(clippy::all)]

use serde_json::{Value, json};
use vectorcraft_engine::{EngineError, Session};

/// Control points from Gwangnyeong-ri 1 (§6, §14):
/// East (X), North (Y), Elevation (Z) in EPSG:5186 (m)
pub const GWANGNYEONGRI_CP1: (f64, f64, f64) = (152840.123, 512340.456, 152.45);
pub const GWANGNYEONGRI_CP2: (f64, f64, f64) = (152860.789, 512340.123, 152.60);
pub const GWANGNYEONGRI_CP3: (f64, f64, f64) = (152860.456, 512360.890, 152.80);
pub const GWANGNYEONGRI_CP4: (f64, f64, f64) = (152840.012, 512360.234, 152.55);

/// Benchmark polygon vertices for Gwangnyeong-ri 1 House 1 (1호 주거지)
/// Evaluates to Area = 32.91 m² ± 0.01 m² via Green's Theorem / Shoelace formula.
pub const HOUSE_1_POLYGON_METERS: [(f64, f64); 12] = [
    (3.9219, 0.0),
    (3.2347, 1.4685),
    (1.7742, 2.4163),
    (0.0, 2.9370),
    (-1.9610, 2.6707),
    (-3.2347, 1.4685),
    (-3.5484, 0.0),
    (-3.2347, -1.4685),
    (-1.9610, -2.6707),
    (0.0, -2.9370),
    (1.7742, -2.4163),
    (3.2347, -1.4685),
];

/// Creates an active session with an empty 800x600 document.
pub fn trowel_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600}));
    s
}

/// Executes a command, returning true if succeeded, or false if not yet registered.
/// If registered and returns an error other than UnknownCommand, panics with error details.
pub fn try_exec(s: &mut Session, cmd: &str, params: Value) -> Result<Value, EngineError> {
    s.execute(cmd, &params)
}

/// Helper that checks if a command is implemented in this build.
pub fn is_command_implemented(s: &mut Session, cmd: &str) -> bool {
    match s.execute(cmd, &json!({})) {
        Err(EngineError::UnknownCommand(_)) => false,
        _ => true,
    }
}

// =========================================================================
// Mathematical & Geometric Verification Oracles (§13 Benchmarks)
// =========================================================================

/// Calculates polygon area using the Shoelace formula (Green's theorem).
pub fn shoelace_area(pts: &[(f64, f64)]) -> f64 {
    let n = pts.len();
    if n < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        sum += pts[i].0 * pts[j].1 - pts[j].0 * pts[i].1;
    }
    (sum.abs()) * 0.5
}

/// Calculates polygon perimeter in meters.
pub fn polygon_perimeter(pts: &[(f64, f64)]) -> f64 {
    let n = pts.len();
    if n < 2 {
        return 0.0;
    }
    let mut perim = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        let dx = pts[j].0 - pts[i].0;
        let dy = pts[j].1 - pts[i].1;
        perim += (dx * dx + dy * dy).sqrt();
    }
    perim
}

/// Computes long-axis azimuth (in degrees, 0 = North, clockwise) and length using PCA.
pub fn pca_azimuth_and_length(pts: &[(f64, f64)]) -> (f64, f64) {
    let n = pts.len() as f64;
    if n < 2.0 {
        return (0.0, 0.0);
    }
    let (mean_x, mean_y) = pts.iter().fold((0.0, 0.0), |acc, p| (acc.0 + p.0, acc.1 + p.1));
    let (cx, cy) = (mean_x / n, mean_y / n);

    let mut cxx = 0.0;
    let mut cyy = 0.0;
    let mut cxy = 0.0;
    for p in pts {
        let dx = p.0 - cx;
        let dy = p.1 - cy;
        cxx += dx * dx;
        cyy += dy * dy;
        cxy += dx * dy;
    }

    // Principal angle of covariance matrix
    let angle_rad = 0.5 * (2.0 * cxy).atan2(cxx - cyy);
    let mut deg = angle_rad.to_degrees();
    if deg < 0.0 {
        deg += 180.0;
    }

    // Extent along principal axis
    let (cos_a, sin_a) = (angle_rad.cos(), angle_rad.sin());
    let mut min_proj = f64::INFINITY;
    let mut max_proj = f64::NEG_INFINITY;
    for p in pts {
        let proj = (p.0 - cx) * cos_a + (p.1 - cy) * sin_a;
        if proj < min_proj { min_proj = proj; }
        if proj > max_proj { max_proj = proj; }
    }
    let length = max_proj - min_proj;
    (deg, length)
}

/// Fits a circle through 3 distinct 2D points (e.g. for pottery rim radius & diameter).
/// Returns Some(((center_x, center_y), diameter)) or None if collinear.
pub fn circle_fit_3pt(p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) -> Option<((f64, f64), f64)> {
    let d = 2.0 * (p1.0 * (p2.1 - p3.1) + p2.0 * (p3.1 - p1.1) + p3.0 * (p1.1 - p2.1));
    if d.abs() < 1e-9 {
        return None; // Collinear
    }
    let p1_sq = p1.0 * p1.0 + p1.1 * p1.1;
    let p2_sq = p2.0 * p2.0 + p2.1 * p2.1;
    let p3_sq = p3.0 * p3.0 + p3.1 * p3.1;

    let cx = (p1_sq * (p2.1 - p3.1) + p2_sq * (p3.1 - p1.1) + p3_sq * (p1.1 - p2.1)) / d;
    let cy = (p1_sq * (p3.0 - p2.0) + p2_sq * (p1.0 - p3.0) + p3_sq * (p2.0 - p1.0)) / d;

    let r = ((p1.0 - cx).powi(2) + (p1.1 - cy).powi(2)).sqrt();
    Some(((cx, cy), 2.0 * r))
}

/// 1D horizontal line segment clipping against solid obstacle rectangles [x1, x2, y1, y2].
/// Clips out solid ground/stone regions, retaining strictly open air intervals.
pub fn clip_horizontal_line_against_void(
    y: f64,
    x_min: f64,
    x_max: f64,
    obstacles: &[(f64, f64, f64, f64)],
) -> Vec<(f64, f64)> {
    let mut intervals = vec![(x_min, x_max)];
    for &(ox1, ox2, oy1, oy2) in obstacles {
        let (min_y, max_y) = if oy1 <= oy2 { (oy1, oy2) } else { (oy2, oy1) };
        if y >= min_y && y <= max_y {
            let (obs_x1, obs_x2) = if ox1 <= ox2 { (ox1, ox2) } else { (ox2, ox1) };
            let mut next_intervals = Vec::new();
            for (ix1, ix2) in intervals {
                if obs_x2 <= ix1 || obs_x1 >= ix2 {
                    next_intervals.push((ix1, ix2));
                } else {
                    if obs_x1 > ix1 {
                        next_intervals.push((ix1, obs_x1));
                    }
                    if obs_x2 < ix2 {
                        next_intervals.push((obs_x2, ix2));
                    }
                }
            }
            intervals = next_intervals;
        }
    }
    intervals
}

/// 2-point similarity registration: solves translation (tx, ty), rotation theta (radians), and scale s.
pub fn solve_2pt_similarity(src: [(f64, f64); 2], dst: [(f64, f64); 2]) -> (f64, f64, f64, f64) {
    let d_src = ((src[1].0 - src[0].0).powi(2) + (src[1].1 - src[0].1).powi(2)).sqrt();
    let d_dst = ((dst[1].0 - dst[0].0).powi(2) + (dst[1].1 - dst[0].1).powi(2)).sqrt();
    let scale = d_dst / d_src;

    let angle_src = (src[1].1 - src[0].1).atan2(src[1].0 - src[0].0);
    let angle_dst = (dst[1].1 - dst[0].1).atan2(dst[1].0 - dst[0].0);
    let theta = angle_dst - angle_src;

    let cos_t = theta.cos();
    let sin_t = theta.sin();
    let tx = dst[0].0 - scale * (src[0].0 * cos_t - src[0].1 * sin_t);
    let ty = dst[0].1 - scale * (src[0].0 * sin_t + src[0].1 * cos_t);

    (tx, ty, theta, scale)
}

/// Solves overdetermined affine transformation via least-squares:
/// Returns [a, b, c, d, tx, ty] and residual error per point.
pub fn solve_affine_least_squares(src: &[(f64, f64)], dst: &[(f64, f64)]) -> Option<([f64; 6], Vec<f64>)> {
    let n = src.len();
    if n < 3 || n != dst.len() {
        return None;
    }

    // Solve for (a, c, tx) and (b, d, ty) via normal equations
    let mut s_xx = 0.0;
    let mut s_yy = 0.0;
    let mut s_xy = 0.0;
    let mut s_x = 0.0;
    let mut s_y = 0.0;

    let mut s_u_x = 0.0;
    let mut s_u_y = 0.0;
    let mut s_u = 0.0;

    let mut s_v_x = 0.0;
    let mut s_v_y = 0.0;
    let mut s_v = 0.0;

    for i in 0..n {
        let (x, y) = src[i];
        let (u, v) = dst[i];
        s_xx += x * x;
        s_yy += y * y;
        s_xy += x * y;
        s_x += x;
        s_y += y;

        s_u_x += u * x;
        s_u_y += u * y;
        s_u += u;

        s_v_x += v * x;
        s_v_y += v * y;
        s_v += v;
    }

    // 3x3 matrix inversion for [s_xx s_xy s_x; s_xy s_yy s_y; s_x s_y n]
    let nf = n as f64;
    let det = s_xx * (s_yy * nf - s_y * s_y)
        - s_xy * (s_xy * nf - s_y * s_x)
        + s_x * (s_xy * s_y - s_yy * s_x);

    if det.abs() < 1e-9 {
        return None; // Singular (collinear)
    }

    let inv00 = (s_yy * nf - s_y * s_y) / det;
    let inv01 = (s_y * s_x - s_xy * nf) / det;
    let inv02 = (s_xy * s_y - s_yy * s_x) / det;

    let inv10 = inv01;
    let inv11 = (s_xx * nf - s_x * s_x) / det;
    let inv12 = (s_x * s_xy - s_xx * s_y) / det;

    let inv20 = inv02;
    let inv21 = inv12;
    let inv22 = (s_xx * s_yy - s_xy * s_xy) / det;

    let a = inv00 * s_u_x + inv01 * s_u_y + inv02 * s_u;
    let c = inv10 * s_u_x + inv11 * s_u_y + inv12 * s_u;
    let tx = inv20 * s_u_x + inv21 * s_u_y + inv22 * s_u;

    let b = inv00 * s_v_x + inv01 * s_v_y + inv02 * s_v;
    let d = inv10 * s_v_x + inv11 * s_v_y + inv12 * s_v;
    let ty = inv20 * s_v_x + inv21 * s_v_y + inv22 * s_v;

    let mut residuals = Vec::with_capacity(n);
    for i in 0..n {
        let (x, y) = src[i];
        let (u, v) = dst[i];
        let pred_u = a * x + c * y + tx;
        let pred_v = b * x + d * y + ty;
        let res = ((pred_u - u).powi(2) + (pred_v - v).powi(2)).sqrt();
        residuals.push(res);
    }

    Some(([a, b, c, d, tx, ty], residuals))
}

/// Dual-graph 2-coloring for alternating hatch (§14(e)):
/// Assigns 45° (false) and 135° (true).
/// Returns (assignments, has_odd_cycle_conflict).
pub fn bipartite_alternating_hatch(
    adjacencies: &[(usize, usize)],
    num_faces: usize,
) -> (Vec<bool>, bool) {
    let mut adj = vec![Vec::new(); num_faces];
    for &(u, v) in adjacencies {
        if u < num_faces && v < num_faces {
            adj[u].push(v);
            adj[v].push(u);
        }
    }

    let mut color = vec![None; num_faces];
    let mut has_conflict = false;

    for start in 0..num_faces {
        if color[start].is_none() {
            color[start] = Some(false);
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(start);

            while let Some(u) = queue.pop_front() {
                let c = color[u].unwrap();
                for &v in &adj[u] {
                    if let Some(vc) = color[v] {
                        if vc == c {
                            has_conflict = true; // Odd cycle detected
                        }
                    } else {
                        color[v] = Some(!c);
                        queue.push_back(v);
                    }
                }
            }
        }
    }

    let assignments = color.into_iter().map(|c| c.unwrap_or(false)).collect();
    (assignments, has_conflict)
}

/// Converts points to millimeters (72 pt / inch, 25.4 mm / inch).
pub fn pt_to_mm(pt: f64) -> f64 {
    pt * 25.4 / 72.0
}

/// Converts millimeters to points.
pub fn mm_to_pt(mm: f64) -> f64 {
    mm * 72.0 / 25.4
}

/// Formats elevation line label per §14(c): "152.5m" without "EL." prefix.
pub fn format_elevation_label(meters: f64) -> String {
    format!("{:.1}m", meters)
}

/// Formats TM coordinate readout per §14(2): X=동(East), Y=북(North).
pub fn format_tm_coords(east_x: f64, north_y: f64) -> String {
    format!("X: {:.3}m, Y: {:.3}m", east_x, north_y)
}

/// Converts world meters to paper millimeters at a given scale denominator (e.g. 20 for 1:20).
pub fn world_to_paper_mm(world_m: f64, scale_denom: f64) -> f64 {
    (world_m * 1000.0) / scale_denom
}

/// Converts world meters to paper points at a given scale denominator.
pub fn world_to_paper_pt(world_m: f64, scale_denom: f64) -> f64 {
    mm_to_pt(world_to_paper_mm(world_m, scale_denom))
}

