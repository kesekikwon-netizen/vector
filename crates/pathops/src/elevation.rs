//! Automatic horizontal elevation lines with 1D scanline void clipping (Trowel design 2.1 §4, §4-1, §14(b, c, d)).

use kurbo::{BezPath, PathEl, Point, Shape};
use vectorcraft_doc::elevation::{ElevationLabelPlacement, ElevationRangeMode, ElevationSpec};
use vectorcraft_doc::scale::ScaleConfig;

/// A single clipped horizontal elevation line segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElevationSegment {
    pub y: f64,
    pub x0: f64,
    pub x1: f64,
    pub is_major: bool,
}

/// An elevation line text label.
#[derive(Clone, Debug, PartialEq)]
pub struct ElevationLabel {
    pub text: String,
    pub at: Point,
    pub anchor_left: bool,
}

/// Generated elevation lines and text labels.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElevationOutput {
    pub segments: Vec<ElevationSegment>,
    pub labels: Vec<ElevationLabel>,
}

/// Flattens a BezPath into linear points within the given tolerance.
fn flatten_path(path: &BezPath, tol: f64) -> Vec<Point> {
    let mut points = Vec::new();
    let mut current = Point::ZERO;
    kurbo::flatten(path.iter(), tol, |el| match el {
        PathEl::MoveTo(p) => {
            current = p;
            points.push(p);
        }
        PathEl::LineTo(p) => {
            current = p;
            points.push(p);
        }
        PathEl::ClosePath => {
            if let Some(&first) = points.first()
                && ((current.x - first.x).abs() > 1e-6 || (current.y - first.y).abs() > 1e-6)
            {
                points.push(first);
                current = first;
            }
        }
        _ => {}
    });
    points
}

/// Finds the x-intersections of the horizontal line y = Y with a list of segments using
/// the half-open interval rule [y0, y1) to prevent vertex duplication.
fn scanline_intersections(segments: &[(Point, Point)], y: f64) -> Vec<f64> {
    let mut xs = Vec::new();
    for &(p1, p2) in segments {
        let (y0, y1) = if p1.y <= p2.y { (p1.y, p2.y) } else { (p2.y, p1.y) };
        if (y1 - y0).abs() < 1e-9 {
            // Horizontal segment
            continue;
        }
        if y >= y0 && y < y1 {
            let t = (y - p1.y) / (p2.y - p1.y);
            let x = p1.x + t * (p2.x - p1.x);
            if x.is_finite() {
                xs.push(x);
            }
        }
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    xs
}

/// Subtracts an obstacle interval [u, v] from a list of disjoint intervals.
fn subtract_interval(intervals: Vec<(f64, f64)>, u: f64, v: f64) -> Vec<(f64, f64)> {
    if u >= v {
        return intervals;
    }
    const EPS: f64 = 1e-4;
    let mut out = Vec::new();
    for (a, b) in intervals {
        if b <= a + EPS {
            continue;
        }
        if v <= a + EPS || u >= b - EPS {
            // No overlap
            out.push((a, b));
        } else if u <= a + EPS && v >= b - EPS {
            // Completely occluded: drop
        } else if u > a + EPS && v < b - EPS {
            // Split into two
            out.push((a, u));
            out.push((v, b));
        } else if u <= a + EPS && v < b - EPS {
            // Trim left
            out.push((v, b));
        } else if u > a + EPS && v >= b - EPS {
            // Trim right
            out.push((a, u));
        }
    }
    out
}

/// Clips a horizontal scanline at y against the open void between the ceiling and ground profile,
/// and subtracts all interior obstacle intervals.
pub fn clip_elevation_scanline(y: f64, ground_profile: &BezPath, obstacles: &[BezPath], x_min: f64, x_max: f64, ceiling_y: f64) -> Vec<(f64, f64)> {
    let profile_pts = flatten_path(ground_profile, 0.25);
    if profile_pts.len() < 2 {
        return Vec::new();
    }

    let p_start = profile_pts[0];
    let p_end = *profile_pts.last().unwrap_or(&p_start);

    let x_min_eff = x_min.min(p_start.x.min(p_end.x));
    let x_max_eff = x_max.max(p_start.x.max(p_end.x));

    // Construct closed Void Envelope Polygon P_void
    let mut void_pts = Vec::new();
    void_pts.push(Point::new(x_min_eff, ceiling_y));
    void_pts.push(Point::new(x_max_eff, ceiling_y));

    if p_start.x <= p_end.x {
        // Profile runs left to right
        void_pts.push(Point::new(x_max_eff, p_end.y));
        if (p_end.x - x_max_eff).abs() > 1e-6 {
            void_pts.push(p_end);
        }
        // Reverse profile
        for pt in profile_pts.iter().rev() {
            void_pts.push(*pt);
        }
        if (p_start.x - x_min_eff).abs() > 1e-6 {
            void_pts.push(Point::new(x_min_eff, p_start.y));
        }
    } else {
        // Profile runs right to left
        void_pts.push(Point::new(x_max_eff, p_start.y));
        if (p_start.x - x_max_eff).abs() > 1e-6 {
            void_pts.push(p_start);
        }
        // Forward profile
        for pt in &profile_pts {
            void_pts.push(*pt);
        }
        if (p_end.x - x_min_eff).abs() > 1e-6 {
            void_pts.push(Point::new(x_min_eff, p_end.y));
        }
    }
    void_pts.push(Point::new(x_min_eff, ceiling_y));

    // Build segments of P_void
    let mut void_segs = Vec::with_capacity(void_pts.len());
    for i in 0..void_pts.len().saturating_sub(1) {
        void_segs.push((void_pts[i], void_pts[i + 1]));
    }

    let xs = scanline_intersections(&void_segs, y);
    if xs.len() < 2 {
        return Vec::new();
    }

    // Pair intersections into void intervals by even-odd parity
    let mut void_intervals = Vec::new();
    for chunk in xs.chunks_exact(2) {
        let a = chunk[0];
        let b = chunk[1];
        if b - a > 1e-4 {
            void_intervals.push((a, b));
        }
    }

    // Subtract obstacle polygon interiors
    for obs in obstacles {
        let obs_pts = flatten_path(obs, 0.25);
        if obs_pts.len() < 3 {
            continue;
        }
        let mut obs_segs = Vec::with_capacity(obs_pts.len());
        for i in 0..obs_pts.len().saturating_sub(1) {
            obs_segs.push((obs_pts[i], obs_pts[i + 1]));
        }
        // Ensure closed segment
        if let (Some(&first), Some(&last)) = (obs_pts.first(), obs_pts.last())
            && ((first.x - last.x).abs() > 1e-6 || (first.y - last.y).abs() > 1e-6)
        {
            obs_segs.push((last, first));
        }
        let obs_xs = scanline_intersections(&obs_segs, y);
        for chunk in obs_xs.chunks_exact(2) {
            void_intervals = subtract_interval(void_intervals, chunk[0], chunk[1]);
        }
    }

    // Retain intervals with at least 0.5 pt length
    void_intervals.retain(|(a, b)| *b - *a >= 0.5);
    void_intervals
}

/// Generates all elevation line segments and labels according to the given ElevationSpec.
pub fn generate_elevation(ground_profile: &BezPath, obstacles: &[BezPath], spec: &ElevationSpec, scale_cfg: Option<&ScaleConfig>) -> ElevationOutput {
    let profile_bounds = ground_profile.bounding_box();
    if profile_bounds.width() < 1e-4 || profile_bounds.height() < 1e-4 {
        return ElevationOutput::default();
    }

    let x_min = profile_bounds.x0;
    let x_max = profile_bounds.x1;

    let (ppm, canvas_origin_y, world_origin_y) = match scale_cfg {
        Some(sc) => (sc.scale.points_per_meter(), sc.canvas_origin.y, sc.world_origin.1),
        None => {
            // Default 1:20 scale (1 meter = 50mm = 141.732pt)
            let default_ppm = (72.0 / 0.0254) / 20.0;
            (default_ppm, profile_bounds.y1, spec.datum_m)
        }
    };

    if ppm <= 0.0 || !ppm.is_finite() {
        return ElevationOutput::default();
    }

    // In canvas coordinates, smaller y is higher elevation (North is up)
    let lowest_elevation_m = world_origin_y + (canvas_origin_y - profile_bounds.y1) / ppm;
    let highest_elevation_m = world_origin_y + (canvas_origin_y - profile_bounds.y0) / ppm;

    let (min_elev, max_elev) = match spec.range_mode {
        ElevationRangeMode::Auto => {
            let minor = spec.minor_interval_m.max(0.01);
            let major = spec.major_interval_m.max(0.05);
            let min_m = (lowest_elevation_m / minor).floor() * minor;
            let max_m = (highest_elevation_m / major).ceil() * major;
            (min_m, max_m)
        }
        ElevationRangeMode::Explicit { min_elevation_m, max_elevation_m } => (min_elevation_m, max_elevation_m),
    };

    if max_elev < min_elev {
        return ElevationOutput::default();
    }

    let minor_step = spec.minor_interval_m.max(0.01);
    let min_scanline_y = canvas_origin_y - (max_elev - world_origin_y) * ppm;
    let ceiling_y = min_scanline_y.min(profile_bounds.y0) - 50.0;

    let count = ((max_elev - min_elev) / minor_step).round() as i64;
    let mut segments = Vec::new();
    let mut labels = Vec::new();

    for k in 0..=count {
        let elev = min_elev + (k as f64) * minor_step;
        let y = canvas_origin_y - (elev - world_origin_y) * ppm;

        let major_mult = (elev / spec.major_interval_m).round();
        let is_major = (elev - major_mult * spec.major_interval_m).abs() < 1e-4;

        let intervals = clip_elevation_scanline(y, ground_profile, obstacles, x_min, x_max, ceiling_y);
        if intervals.is_empty() {
            continue;
        }

        for &(x0, x1) in &intervals {
            segments.push(ElevationSegment { y, x0, x1, is_major });
        }

        if is_major {
            let text = spec.label_format.format(elev);
            match spec.label_placement {
                ElevationLabelPlacement::LeftEnd => {
                    labels.push(ElevationLabel { text, at: Point::new(intervals[0].0, y), anchor_left: true });
                }
                ElevationLabelPlacement::BothEnds => {
                    labels.push(ElevationLabel { text: text.clone(), at: Point::new(intervals[0].0, y), anchor_left: true });
                    labels.push(ElevationLabel { text, at: Point::new(intervals.last().unwrap_or(&intervals[0]).1, y), anchor_left: false });
                }
            }
        }
    }

    ElevationOutput { segments, labels }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v_trench_open_void_clipping() {
        // V-shaped trench: ground at y = 100 from x=0..100, drops to y=300 at x=200, rises back to y=100 at x=300..400
        let mut profile = BezPath::new();
        profile.move_to((0.0, 100.0));
        profile.line_to((100.0, 100.0));
        profile.line_to((200.0, 300.0));
        profile.line_to((300.0, 100.0));
        profile.line_to((400.0, 100.0));

        let ceiling_y = 0.0;
        let x_min = 0.0;
        let x_max = 400.0;

        // 1. Scanline above ground at y = 50: open air spanning full width [0, 400]
        let res_50 = clip_elevation_scanline(50.0, &profile, &[], x_min, x_max, ceiling_y);
        assert_eq!(res_50.len(), 1);
        assert!((res_50[0].0 - 0.0).abs() < 1e-3);
        assert!((res_50[0].1 - 400.0).abs() < 1e-3);

        // 2. Scanline halfway in trench at y = 200:
        // Left trench wall slope from (100, 100) to (200, 300): at y=200, x = 150.
        // Right trench wall slope from (200, 300) to (300, 100): at y=200, x = 250.
        let res_200 = clip_elevation_scanline(200.0, &profile, &[], x_min, x_max, ceiling_y);
        assert_eq!(res_200.len(), 1);
        assert!((res_200[0].0 - 150.0).abs() < 1e-3, "left wall at 150: got {}", res_200[0].0);
        assert!((res_200[0].1 - 250.0).abs() < 1e-3, "right wall at 250: got {}", res_200[0].1);

        // 3. Scanline below trench bottom at y = 350: zero lines emitted
        let res_350 = clip_elevation_scanline(350.0, &profile, &[], x_min, x_max, ceiling_y);
        assert!(res_350.is_empty(), "expected no segments below trench floor");
    }

    #[test]
    fn test_obstacle_subtraction() {
        // Flat ground trench at y = 200 spanning x=100..500
        let mut profile = BezPath::new();
        profile.move_to((100.0, 200.0));
        profile.line_to((500.0, 200.0));

        // Stone obstacle rectangle at x=250..350, y=120..200
        let mut stone = BezPath::new();
        stone.move_to((250.0, 120.0));
        stone.line_to((350.0, 120.0));
        stone.line_to((350.0, 200.0));
        stone.line_to((250.0, 200.0));
        stone.close_path();

        let res = clip_elevation_scanline(150.0, &profile, &[stone], 100.0, 500.0, 50.0);
        // Expect line to split around the stone: [100, 250] and [350, 500]
        assert_eq!(res.len(), 2);
        assert!((res[0].0 - 100.0).abs() < 1e-3);
        assert!((res[0].1 - 250.0).abs() < 1e-3);
        assert!((res[1].0 - 350.0).abs() < 1e-3);
        assert!((res[1].1 - 500.0).abs() < 1e-3);
    }

    #[test]
    fn test_dual_posthole_clipping() {
        // Trench floor at y=200, with posthole 1 at x=150..170 dropping to y=250,
        // and posthole 2 at x=330..350 dropping to y=250.
        let mut profile = BezPath::new();
        profile.move_to((100.0, 200.0));
        profile.line_to((150.0, 200.0));
        profile.line_to((150.0, 250.0));
        profile.line_to((170.0, 250.0));
        profile.line_to((170.0, 200.0));
        profile.line_to((330.0, 200.0));
        profile.line_to((330.0, 250.0));
        profile.line_to((350.0, 250.0));
        profile.line_to((350.0, 200.0));
        profile.line_to((400.0, 200.0));

        // Scanline at y = 225 (inside both postholes, below trench floor)
        let res = clip_elevation_scanline(225.0, &profile, &[], 100.0, 400.0, 50.0);
        assert_eq!(res.len(), 2, "must emit exactly two segments inside postholes");
        assert!((res[0].0 - 150.0).abs() < 1e-3 && (res[0].1 - 170.0).abs() < 1e-3);
        assert!((res[1].0 - 330.0).abs() < 1e-3 && (res[1].1 - 350.0).abs() < 1e-3);
    }

    #[test]
    fn test_benchmark_section_geometry() {
        // From Trowel 디자인.html §4 SVG (lines 846, 847, 850)
        let profile_svg = "M 56.5 173 L 109 173 L 152.75 174.75 L 165 197.5 L 179 229 L 193 257 L 207 274.5 L 217.5 279.75 L 233.25 279.75 L 236.75 302.5 L 242 321.75 L 249 327 L 256 323.5 L 261.25 306 L 264.75 279.75 L 336.5 279.75 L 476.5 281.5 L 581.5 281.5 L 756.5 281.5 L 852.75 281.5 L 861.5 286.75 L 879 292 L 907 295.5 L 922.75 299 L 949 302.5 L 984 304.25 L 1019 302.5 L 1045.25 299 L 1057.5 293.75 L 1082 292 L 1103 286.75 L 1110 281.5 L 1194 281.5 L 1369 281.5 L 1486.25 281.5 L 1489.75 306 L 1496.75 323.5 L 1505.5 327 L 1514.25 323.5 L 1521.25 306 L 1524.75 281.5 L 1540.5 279.75 L 1554.5 271 L 1568.5 250 L 1582.5 225.5 L 1596.5 197.5 L 1607 178.25 L 1649 178.25 L 1698 180";
        let profile = BezPath::from_svg(profile_svg).unwrap();

        // Cut stone obstacle (line 847)
        let stone1_svg = "M 945.5 302.5 L 947.25 290.25 L 956 281.5 L 973.5 278.88 L 991 279.75 L 1003.25 286.75 L 1006.75 297.25 L 1001.5 303.72 L 975.25 304.25 L 952.5 303.55 Z";
        let stone1 = BezPath::from_svg(stone1_svg).unwrap();

        // Grinding stone obstacle (line 850)
        let stone2_svg = "M 483.5 280.62 L 485.25 274.5 L 494 269.25 L 515 266.62 L 546.5 266.62 L 567.5 269.25 L 578 274.5 L 576.25 280.62 L 546.5 281.85 L 511.5 281.85 Z";
        let stone2 = BezPath::from_svg(stone2_svg).unwrap();

        let obstacles = vec![stone1, stone2];

        // 1. y = 166.0 (in air): line 845 "M 56.5 166 H 1698"
        let res_166 = clip_elevation_scanline(166.0, &profile, &obstacles, 56.5, 1698.0, 50.0);
        assert_eq!(res_166.len(), 1);
        assert!((res_166[0].0 - 56.5).abs() < 0.5);
        assert!((res_166[0].1 - 1698.0).abs() < 0.5);

        // 2. y = 323.5 (inside postholes): lines 828-829 "M 244.45 323.5 H 255.83" & "M 1496.93 323.5 H 1514.08"
        let res_323 = clip_elevation_scanline(323.5, &profile, &obstacles, 56.5, 1698.0, 50.0);
        assert_eq!(res_323.len(), 2, "dual postholes at 323.5");
        assert!((res_323[0].0 - 244.45).abs() < 1.0);
        assert!((res_323[0].1 - 255.83).abs() < 1.0);
        assert!((res_323[1].0 - 1496.93).abs() < 1.0);
        assert!((res_323[1].1 - 1514.08).abs() < 1.0);
    }
}
