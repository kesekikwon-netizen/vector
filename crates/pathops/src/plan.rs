//! Plan drafting geometry generators and statistical analysis (Trowel design 2.1 §3, §4-1, §7, §8, §13, §14, §15, J009).
//!
//! Provides:
//! - Section cut line generator with 1-dot chain dash, directional arrows, and terminal badges.
//! - Spot elevation marker symbol and formatted elevation text.
//! - Depth marker generator (posthole concentric circles) with depth text.
//! - Slope hachures generator emitting alternating 100% and 50% perpendicular ticks at 1.5mm paper spacing.
//! - Closed feature polygon statistical analysis: Shoelace area (32.91 ㎡ benchmark), rotating calipers MBB axes, and PCA azimuth.

use kurbo::{BezPath, PathEl, Point, Vec2};
use vectorcraft_doc::legend::MM;
use vectorcraft_doc::scale::ScaleConfig;

/// Default plan line weight: 0.2 pt (§7).
pub const DEFAULT_PLAN_WEIGHT_PT: f64 = 0.2;

/// Default slope hachure spacing on paper: 1.5 mm (§8).
pub const DEFAULT_HACHURE_SPACING_MM: f64 = 1.5;

/// Default short tick ratio for slope hachure: 0.50 (50%).
pub const DEFAULT_HACHURE_SHORT_RATIO: f64 = 0.50;

/// Default 1-dot chain dash pattern: 12pt dash, 3pt gap, 2pt dot, 3pt gap.
pub const SECTION_LINE_DASH_PATTERN: [f64; 4] = [12.0, 3.0, 2.0, 3.0];

/// Tolerance for kurbo path flattening in points.
pub const FLATTEN_TOL: f64 = 0.05;

/// Canonical Gwangnyeong-ri 1 House 1 benchmark polygon in meters (§13).
/// Yields exactly 32.91 ㎡ under Shoelace integration.
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

/// Direction of section line line-of-sight arrows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewDirection {
    #[default]
    Left,
    Right,
}

/// Specification for generating a section cut line annotation (A–A′).
#[derive(Clone, Debug, PartialEq)]
pub struct SectionLineSpec {
    pub p0: Point,
    pub p1: Point,
    pub label_start: String,
    pub label_end: String,
    pub view_direction: ViewDirection,
    pub weight_pt: f64,
    pub arrow_length_pt: f64,
    pub dash_pattern: Vec<f64>,
}

impl Default for SectionLineSpec {
    fn default() -> Self {
        Self {
            p0: Point::ZERO,
            p1: Point::new(100.0, 0.0),
            label_start: "A".to_string(),
            label_end: "A′".to_string(),
            view_direction: ViewDirection::Left,
            weight_pt: DEFAULT_PLAN_WEIGHT_PT,
            arrow_length_pt: 14.0,
            dash_pattern: SECTION_LINE_DASH_PATTERN.to_vec(),
        }
    }
}

/// Generated geometric paths and label anchors for a section cut line.
#[derive(Clone, Debug, PartialEq)]
pub struct SectionLineGeometry {
    pub cut_line: BezPath,
    pub arrows: BezPath,
    pub badges: BezPath,
    pub label_start_pos: Point,
    pub label_end_pos: Point,
    pub label_start: String,
    pub label_end: String,
    pub weight_pt: f64,
    pub dash_pattern: Vec<f64>,
}

/// Generates section cut line geometry with 1-dot chain line, directional sightline arrows,
/// and terminal badge circles.
pub fn generate_section_line_geometry(spec: &SectionLineSpec) -> SectionLineGeometry {
    let mut cut_line = BezPath::new();
    cut_line.move_to(spec.p0);
    cut_line.line_to(spec.p1);

    let v = spec.p1 - spec.p0;
    let dist = v.hypot();
    let u = if dist > 1e-9 { v / dist } else { Vec2::new(1.0, 0.0) };
    let n = match spec.view_direction {
        ViewDirection::Left => Vec2::new(-u.y, u.x),
        ViewDirection::Right => Vec2::new(u.y, -u.x),
    };

    let mut arrows = BezPath::new();
    let arrow_len = spec.arrow_length_pt;
    let head_size = 5.0;

    // Start terminal arrow & back tick
    let p0_head = spec.p0 + n * arrow_len;
    arrows.move_to(spec.p0 - n * (arrow_len * 0.25));
    arrows.line_to(p0_head);
    // Arrowhead pointing along normal n
    arrows.move_to(p0_head - n * head_size - u * (head_size * 0.6));
    arrows.line_to(p0_head);
    arrows.line_to(p0_head - n * head_size + u * (head_size * 0.6));

    // End terminal arrow & back tick
    let p1_head = spec.p1 + n * arrow_len;
    arrows.move_to(spec.p1 - n * (arrow_len * 0.25));
    arrows.line_to(p1_head);
    arrows.move_to(p1_head - n * head_size - u * (head_size * 0.6));
    arrows.line_to(p1_head);
    arrows.line_to(p1_head - n * head_size + u * (head_size * 0.6));

    // Terminal badges: circular halo badges enclosing the letters
    let badge_radius = 8.0;
    let badge_offset = badge_radius + 4.0;
    let label_start_pos = spec.p0 - u * badge_offset;
    let label_end_pos = spec.p1 + u * badge_offset;

    let mut badges = BezPath::new();
    add_circle(&mut badges, label_start_pos, badge_radius);
    add_circle(&mut badges, label_end_pos, badge_radius);

    SectionLineGeometry {
        cut_line,
        arrows,
        badges,
        label_start_pos,
        label_end_pos,
        label_start: spec.label_start.clone(),
        label_end: spec.label_end.clone(),
        weight_pt: spec.weight_pt,
        dash_pattern: spec.dash_pattern.clone(),
    }
}

/// Marker kind for spot elevation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpotMarkerKind {
    #[default]
    Dot,
    Triangle,
    Crosshair,
}

/// Specification for a spot elevation marker.
#[derive(Clone, Debug, PartialEq)]
pub struct SpotElevationSpec {
    pub at: Point,
    pub elevation_m: f64,
    pub marker: SpotMarkerKind,
    pub font_size_pt: f64,
    pub halo: bool,
    pub prefix_plus: bool,
    pub suffix_m: bool,
}

impl Default for SpotElevationSpec {
    fn default() -> Self {
        Self { at: Point::ZERO, elevation_m: 0.0, marker: SpotMarkerKind::Dot, font_size_pt: 7.0, halo: true, prefix_plus: true, suffix_m: true }
    }
}

/// Generated geometry for a spot elevation.
#[derive(Clone, Debug, PartialEq)]
pub struct SpotElevationGeometry {
    pub marker_path: BezPath,
    pub marker_fill: bool,
    pub label_pos: Point,
    pub text: String,
    pub font_size_pt: f64,
    pub halo: bool,
}

/// Generates spot elevation marker symbol and formatted 2-decimal-place text.
pub fn generate_spot_elevation_geometry(spec: &SpotElevationSpec) -> SpotElevationGeometry {
    let mut marker_path = BezPath::new();
    let marker_fill = match spec.marker {
        SpotMarkerKind::Dot => {
            // Solid filled circle r = 1.8 pt
            add_circle(&mut marker_path, spec.at, 1.8);
            true
        }
        SpotMarkerKind::Triangle => {
            // Equilateral triangle side = 7.0 pt with horizontal line
            let side = 7.0;
            let h = side * (3.0f64.sqrt() / 2.0);
            let p_top = Point::new(spec.at.x, spec.at.y - h * (2.0 / 3.0));
            let p_left = Point::new(spec.at.x - side * 0.5, spec.at.y + h * (1.0 / 3.0));
            let p_right = Point::new(spec.at.x + side * 0.5, spec.at.y + h * (1.0 / 3.0));
            marker_path.move_to(p_top);
            marker_path.line_to(p_left);
            marker_path.line_to(p_right);
            marker_path.close_path();
            // Horizontal bar through center
            marker_path.move_to(Point::new(spec.at.x - side * 0.4, spec.at.y));
            marker_path.line_to(Point::new(spec.at.x + side * 0.4, spec.at.y));
            false
        }
        SpotMarkerKind::Crosshair => {
            // Target circle r = 3.0 pt + crosshair ticks
            add_circle(&mut marker_path, spec.at, 3.0);
            marker_path.move_to(Point::new(spec.at.x - 5.0, spec.at.y));
            marker_path.line_to(Point::new(spec.at.x + 5.0, spec.at.y));
            marker_path.move_to(Point::new(spec.at.x, spec.at.y - 5.0));
            marker_path.line_to(Point::new(spec.at.x, spec.at.y + 5.0));
            false
        }
    };

    let sign = if spec.prefix_plus && spec.elevation_m > 0.0 { "+" } else { "" };
    let unit = if spec.suffix_m { "m" } else { "" };
    let text = format!("{}{:.2}{}", sign, spec.elevation_m, unit);
    let label_pos = Point::new(spec.at.x + 5.0, spec.at.y - 4.0);

    SpotElevationGeometry { marker_path, marker_fill, label_pos, text, font_size_pt: spec.font_size_pt, halo: spec.halo }
}

/// Kind of depth marker symbol.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DepthMarkerKind {
    #[default]
    PostholeConcentric,
    Cross,
    None,
}

/// Specification for a trench / posthole depth marker.
#[derive(Clone, Debug, PartialEq)]
pub struct DepthMarkerSpec {
    pub at: Point,
    pub depth_val: f64,
    pub is_cm: bool,
    pub symbol: DepthMarkerKind,
    pub font_size_pt: f64,
    pub outer_radius_pt: f64,
}

impl Default for DepthMarkerSpec {
    fn default() -> Self {
        Self { at: Point::ZERO, depth_val: -32.0, is_cm: true, symbol: DepthMarkerKind::PostholeConcentric, font_size_pt: 7.0, outer_radius_pt: 7.0 }
    }
}

/// Generated geometry for a depth marker.
#[derive(Clone, Debug, PartialEq)]
pub struct DepthMarkerGeometry {
    pub solid_symbol: BezPath,
    pub dashed_symbol: BezPath,
    pub label_pos: Point,
    pub text: String,
    pub font_size_pt: f64,
}

/// Generates depth marker geometry (concentric circles for posthole) and formatted depth text.
pub fn generate_depth_marker_geometry(spec: &DepthMarkerSpec) -> DepthMarkerGeometry {
    let mut solid_symbol = BezPath::new();
    let mut dashed_symbol = BezPath::new();

    match spec.symbol {
        DepthMarkerKind::PostholeConcentric => {
            // Outer solid circle r = outer_radius_pt
            add_circle(&mut solid_symbol, spec.at, spec.outer_radius_pt);
            // Inner dashed circle r = 0.5 * outer_radius_pt
            add_circle(&mut dashed_symbol, spec.at, spec.outer_radius_pt * 0.5);
        }
        DepthMarkerKind::Cross => {
            let r = spec.outer_radius_pt * 0.7;
            solid_symbol.move_to(Point::new(spec.at.x - r, spec.at.y));
            solid_symbol.line_to(Point::new(spec.at.x + r, spec.at.y));
            solid_symbol.move_to(Point::new(spec.at.x, spec.at.y - r));
            solid_symbol.line_to(Point::new(spec.at.x, spec.at.y + r));
        }
        DepthMarkerKind::None => {}
    }

    let text = if spec.is_cm { format!("{:.0}", spec.depth_val) } else { format!("{:.2}m", spec.depth_val) };

    let label_pos = match spec.symbol {
        DepthMarkerKind::PostholeConcentric | DepthMarkerKind::Cross => Point::new(spec.at.x + spec.outer_radius_pt + 3.0, spec.at.y - 2.0),
        DepthMarkerKind::None => spec.at,
    };

    DepthMarkerGeometry { solid_symbol, dashed_symbol, label_pos, text, font_size_pt: spec.font_size_pt }
}

/// Specification for slope hachure generation (경사면 비탈면 기호).
#[derive(Clone, Debug, PartialEq)]
pub struct SlopeHachureSpec {
    pub spacing_mm: f64,
    pub short_ratio: f64,
    pub weight_pt: f64,
    pub default_width_pt: f64,
    pub range: Option<(f64, f64)>,
}

impl Default for SlopeHachureSpec {
    fn default() -> Self {
        Self {
            spacing_mm: DEFAULT_HACHURE_SPACING_MM,
            short_ratio: DEFAULT_HACHURE_SHORT_RATIO,
            weight_pt: DEFAULT_PLAN_WEIGHT_PT,
            default_width_pt: 15.0,
            range: None,
        }
    }
}

/// Generates slope hachure tick marks along an embankment/pit edge curve.
/// Alternates between long (100%) and short (50%) perpendicular ticks at 1.5mm paper spacing.
pub fn generate_slope_hachures(top_path: &BezPath, bottom_path: Option<&BezPath>, spec: &SlopeHachureSpec) -> BezPath {
    let mut out = BezPath::new();
    let top_pts = flatten_closed_path(top_path, FLATTEN_TOL);
    if top_pts.len() < 2 {
        return out;
    }

    let spacing_pt = (spec.spacing_mm * MM).max(1.0);

    let bottom_segs: Vec<(Point, Point)> = bottom_path
        .map(|bp| {
            let bpts = flatten_closed_path(bp, FLATTEN_TOL);
            bpts.windows(2).map(|w| (w[0], w[1])).collect()
        })
        .unwrap_or_default();

    let mut total_len = 0.0;
    let mut seg_lens = Vec::with_capacity(top_pts.len());
    for i in 0..(top_pts.len() - 1) {
        let d = top_pts[i].distance(top_pts[i + 1]);
        seg_lens.push(d);
        total_len += d;
    }

    if total_len < 1e-4 {
        return out;
    }

    let (t_start, t_end) = spec.range.unwrap_or((0.0, 1.0));
    let s_min = t_start.clamp(0.0, 1.0) * total_len;
    let s_max = t_end.clamp(0.0, 1.0) * total_len;

    let mut cur_s = s_min + spacing_pt * 0.5;
    let mut tick_idx = 0usize;

    while cur_s <= s_max {
        if let Some((p, norm)) = sample_curve_at_distance(&top_pts, &seg_lens, cur_s) {
            let slope_len = if !bottom_segs.is_empty() {
                find_ray_intersection(p, norm, &bottom_segs).unwrap_or(spec.default_width_pt)
            } else {
                spec.default_width_pt
            };

            let is_long = tick_idx.is_multiple_of(2);
            let tick_len = if is_long { slope_len } else { slope_len * spec.short_ratio };

            let end_p = p + norm * tick_len;
            out.move_to(p);
            out.line_to(end_p);
            tick_idx += 1;
        }
        cur_s += spacing_pt;
    }

    out
}

/// Statistical readout metrics for a closed archaeological feature polygon.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FeatureStats {
    /// Exact area in square meters (㎡).
    pub area_m2: f64,
    /// Exact boundary perimeter in meters (m).
    pub perimeter_m: f64,
    /// Minimum Bounding Box major axis length in meters.
    pub major_axis_m: f64,
    /// Minimum Bounding Box minor axis length in meters.
    pub minor_axis_m: f64,
    /// Principal azimuth angle in decimal degrees [0, 180).
    pub azimuth_deg: f64,
    /// Formatted compass quadrant string, e.g. "N 68° E".
    pub azimuth_quadrant: String,
    /// Centroid in canvas coordinates.
    pub centroid_canvas: Point,
    /// Centroid in real-world TM coordinates (East, North).
    pub centroid_world: (f64, f64),
}

impl FeatureStats {
    pub fn format_area(&self) -> String {
        format!("{:.2} ㎡", self.area_m2)
    }
    pub fn format_perimeter(&self) -> String {
        format!("{:.2} m", self.perimeter_m)
    }
    pub fn format_axes(&self) -> String {
        format!("{:.2} × {:.2} m", self.major_axis_m, self.minor_axis_m)
    }
    pub fn format_azimuth(&self) -> String {
        self.azimuth_quadrant.clone()
    }
}

/// Computes comprehensive statistical readouts for a closed contour path.
pub fn compute_polygon_stats(path: &BezPath, scale: &ScaleConfig) -> FeatureStats {
    let pts = flatten_closed_path(path, FLATTEN_TOL);
    let n = pts.len();
    if n < 3 {
        return FeatureStats::default();
    }

    // 1. Exact Area and Centroid via Green's Theorem / Shoelace Formula
    let mut area_sum = 0.0;
    let mut cx_sum = 0.0;
    let mut cy_sum = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        let cross = pts[i].x * pts[j].y - pts[j].x * pts[i].y;
        area_sum += cross;
        cx_sum += (pts[i].x + pts[j].x) * cross;
        cy_sum += (pts[i].y + pts[j].y) * cross;
    }
    let area_pt2 = 0.5 * area_sum.abs();
    let signed_area = 0.5 * area_sum;

    let (centroid_x, centroid_y) =
        if signed_area.abs() > 1e-9 { (cx_sum / (6.0 * signed_area), cy_sum / (6.0 * signed_area)) } else { (pts[0].x, pts[0].y) };
    let centroid_canvas = Point::new(centroid_x, centroid_y);

    // 2. Exact Perimeter
    let mut perim_pt = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        perim_pt += pts[i].distance(pts[j]);
    }

    // 3. Unit conversions (points -> meters)
    let mpp = scale.scale.meters_per_point();
    let area_m2 = area_pt2 * (mpp * mpp);
    let perimeter_m = perim_pt * mpp;
    let centroid_world = scale.canvas_to_world(centroid_canvas);

    // 4. Principal Component Analysis (PCA) for Principal Azimuth
    let nf = n as f64;
    let mut cxx = 0.0;
    let mut cyy = 0.0;
    let mut cxy = 0.0;
    for p in &pts {
        let dx = p.x - centroid_x;
        let dy = p.y - centroid_y;
        cxx += dx * dx;
        cyy += dy * dy;
        cxy += dx * dy;
    }
    cxx /= nf;
    cyy /= nf;
    cxy /= nf;

    let angle_rad = 0.5 * (2.0 * cxy).atan2(cxx - cyy);

    // In canvas coordinates: East is +x, North is -y (decreasing y).
    // The vector in world TM coordinates:
    let de = angle_rad.cos();
    let dn = -angle_rad.sin(); // invert y for North
    let mut az_deg = de.atan2(dn).to_degrees();
    while az_deg < 0.0 {
        az_deg += 180.0;
    }
    while az_deg >= 180.0 {
        az_deg -= 180.0;
    }

    let quadrant = if az_deg <= 90.0 { format!("N {:.0}° E", az_deg.round()) } else { format!("N {:.0}° W", (180.0 - az_deg).round()) };

    // 5. Minimum Bounding Box (Rotating Calipers on Convex Hull)
    let (major_axis_pt, minor_axis_pt) = compute_mbb_dimensions(&pts);
    let major_axis_m = major_axis_pt * mpp;
    let minor_axis_m = minor_axis_pt * mpp;

    FeatureStats {
        area_m2,
        perimeter_m,
        major_axis_m,
        minor_axis_m,
        azimuth_deg: az_deg,
        azimuth_quadrant: quadrant,
        centroid_canvas,
        centroid_world,
    }
}

/// Computes MBB dimensions using rotating calipers on the convex hull.
fn compute_mbb_dimensions(pts: &[Point]) -> (f64, f64) {
    let hull = convex_hull(pts);
    let hn = hull.len();
    if hn < 2 {
        return (0.0, 0.0);
    }
    if hn == 2 {
        return (hull[0].distance(hull[1]), 0.0);
    }

    let mut min_area = f64::INFINITY;
    let mut best_major = 0.0;
    let mut best_minor = 0.0;

    for i in 0..hn {
        let j = (i + 1) % hn;
        let v = hull[j] - hull[i];
        let len = v.hypot();
        if len < 1e-9 {
            continue;
        }
        let u = v / len;
        let n = Vec2::new(-u.y, u.x);

        let mut min_u = f64::INFINITY;
        let mut max_u = f64::NEG_INFINITY;
        let mut min_n = f64::INFINITY;
        let mut max_n = f64::NEG_INFINITY;

        for p in &hull {
            let pu = (p.x * u.x) + (p.y * u.y);
            let pn = (p.x * n.x) + (p.y * n.y);
            if pu < min_u {
                min_u = pu;
            }
            if pu > max_u {
                max_u = pu;
            }
            if pn < min_n {
                min_n = pn;
            }
            if pn > max_n {
                max_n = pn;
            }
        }

        let w = max_u - min_u;
        let h = max_n - min_n;
        let area = w * h;
        if area < min_area {
            min_area = area;
            best_major = w.max(h);
            best_minor = w.min(h);
        }
    }
    (best_major, best_minor)
}

/// 2D Monotone chain convex hull algorithm.
fn convex_hull(pts: &[Point]) -> Vec<Point> {
    let mut sorted = pts.to_vec();
    sorted.sort_by(|a, b| {
        a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });
    sorted.dedup_by(|a, b| a.distance(*b) < 1e-9);

    let n = sorted.len();
    if n <= 2 {
        return sorted;
    }

    let cross = |o: Point, a: Point, b: Point| -> f64 { (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x) };

    let mut lower = Vec::new();
    for &p in &sorted {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }

    let mut upper = Vec::new();
    for &p in sorted.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Flattens a BezPath to discrete points and ensures loop closure.
pub fn flatten_closed_path(path: &BezPath, tol: f64) -> Vec<Point> {
    let mut pts = Vec::new();
    let mut cur = Point::ZERO;
    kurbo::flatten(path.iter(), tol, |el| match el {
        PathEl::MoveTo(p) => {
            cur = p;
            pts.push(p);
        }
        PathEl::LineTo(p) => {
            cur = p;
            pts.push(p);
        }
        PathEl::ClosePath => {
            if let Some(&first) = pts.first()
                && cur.distance(first) > 1e-6
            {
                pts.push(first);
                cur = first;
            }
        }
        _ => {}
    });
    if let (Some(&first), Some(&last)) = (pts.first(), pts.last())
        && first.distance(last) < 1e-6
        && pts.len() > 1
    {
        pts.pop();
    }
    pts
}

fn sample_curve_at_distance(pts: &[Point], lens: &[f64], target_s: f64) -> Option<(Point, Vec2)> {
    let mut accum = 0.0;
    for (i, &d) in lens.iter().enumerate() {
        if target_s <= accum + d + 1e-9 {
            let t = if d > 1e-9 { (target_s - accum) / d } else { 0.0 };
            let p0 = pts[i];
            let p1 = pts[i + 1];
            let p = Point::new(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y));
            let dir = p1 - p0;
            let h = dir.hypot();
            if h > 1e-9 {
                let u = dir / h;
                let norm = Vec2::new(-u.y, u.x); // perpendicular normal
                return Some((p, norm));
            }
        }
        accum += d;
    }
    None
}

fn find_ray_intersection(origin: Point, dir: Vec2, segs: &[(Point, Point)]) -> Option<f64> {
    let mut closest_t = f64::INFINITY;
    for &(a, b) in segs {
        let v = b - a;
        let denom = dir.x * v.y - dir.y * v.x;
        if denom.abs() < 1e-9 {
            continue;
        }
        let dp = a - origin;
        let t = (dp.x * v.y - dp.y * v.x) / denom;
        let u = (dp.x * dir.y - dp.y * dir.x) / denom;
        if t > 0.5 && (0.0..=1.0).contains(&u) && t < closest_t {
            closest_t = t;
        }
    }
    if closest_t.is_finite() { Some(closest_t) } else { None }
}

fn add_circle(path: &mut BezPath, c: Point, r: f64) {
    let k = 0.5522847498307935 * r;
    path.move_to(Point::new(c.x + r, c.y));
    path.curve_to(Point::new(c.x + r, c.y + k), Point::new(c.x + k, c.y + r), Point::new(c.x, c.y + r));
    path.curve_to(Point::new(c.x - k, c.y + r), Point::new(c.x - r, c.y + k), Point::new(c.x - r, c.y));
    path.curve_to(Point::new(c.x - r, c.y - k), Point::new(c.x - k, c.y - r), Point::new(c.x, c.y - r));
    path.curve_to(Point::new(c.x + k, c.y - r), Point::new(c.x + r, c.y - k), Point::new(c.x + r, c.y));
    path.close_path();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_house_1_polygon_area_32_91() {
        let scale = ScaleConfig::new(20.0); // 1:20 scale
        let ppm = scale.scale.points_per_meter();

        let mut path = BezPath::new();
        let (first_x, first_y) = HOUSE_1_POLYGON_METERS[0];
        path.move_to(Point::new(first_x * ppm, -first_y * ppm));

        for &(x, y) in &HOUSE_1_POLYGON_METERS[1..] {
            path.line_to(Point::new(x * ppm, -y * ppm));
        }
        path.close_path();

        let stats = compute_polygon_stats(&path, &scale);
        assert_eq!(stats.format_area(), "32.91 ㎡");
        assert!((stats.area_m2 - 32.91).abs() < 0.01);
    }

    #[test]
    fn test_section_line_geometry() {
        let spec = SectionLineSpec { p0: Point::new(10.0, 50.0), p1: Point::new(210.0, 50.0), ..Default::default() };
        let geom = generate_section_line_geometry(&spec);
        assert!(!geom.cut_line.is_empty());
        assert!(!geom.arrows.is_empty());
        assert!(!geom.badges.is_empty());
        assert_eq!(geom.label_start, "A");
        assert_eq!(geom.label_end, "A′");
        assert_eq!(geom.dash_pattern, vec![12.0, 3.0, 2.0, 3.0]);
    }

    #[test]
    fn test_spot_elevation_formatting() {
        let spec = SpotElevationSpec { at: Point::new(100.0, 200.0), elevation_m: 152.35, marker: SpotMarkerKind::Dot, ..Default::default() };
        let geom = generate_spot_elevation_geometry(&spec);
        assert_eq!(geom.text, "+152.35m");
        assert!(geom.marker_fill);
    }

    #[test]
    fn test_slope_hachures_alternation() {
        let mut top = BezPath::new();
        top.move_to(Point::new(0.0, 0.0));
        top.line_to(Point::new(100.0, 0.0));

        let spec = SlopeHachureSpec { spacing_mm: 1.5, short_ratio: 0.5, default_width_pt: 10.0, ..Default::default() };

        let hachures = generate_slope_hachures(&top, None, &spec);
        assert!(!hachures.is_empty());
    }
}
