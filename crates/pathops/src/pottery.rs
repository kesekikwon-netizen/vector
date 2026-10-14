//! Geometric operations for artifact pottery drafting (Trowel design 2.1 §4-3, §14(3), §15(2), J011).

use kurbo::{BezPath, Line, Point};
use vectorcraft_doc::hatch::HatchDirection;
use vectorcraft_doc::pottery::{PotteryMeasurements, SectionSide};
use vectorcraft_geom::{Anchor, PathData, SubPath};

/// Bilateral reflection across vertical centerline ($x = x_{CL}$) with anchor and handle orientation reversal.
pub fn reflect_path_across_centerline(path: &PathData, centerline_x: f64) -> PathData {
    let mut new_subpaths = Vec::with_capacity(path.subpaths.len());
    for sp in &path.subpaths {
        let mut new_anchors = Vec::with_capacity(sp.anchors.len());
        for a in sp.anchors.iter().rev() {
            new_anchors.push(Anchor {
                p: Point::new(2.0 * centerline_x - a.p.x, a.p.y),
                h_in: Point::new(2.0 * centerline_x - a.h_out.x, a.h_out.y),
                h_out: Point::new(2.0 * centerline_x - a.h_in.x, a.h_in.y),
                kind: a.kind,
            });
        }
        new_subpaths.push(SubPath { anchors: new_anchors, closed: sp.closed });
    }
    PathData { subpaths: new_subpaths }
}

/// Computes inward wall profile offset by `thickness_pt` towards the centerline.
pub fn offset_wall_profile(profile: &PathData, thickness_pt: f64, centerline_x: f64, side: SectionSide) -> PathData {
    let mut new_subpaths = Vec::with_capacity(profile.subpaths.len());

    for sp in &profile.subpaths {
        let n = sp.anchors.len();
        if n == 0 {
            continue;
        }

        let mut new_anchors = Vec::with_capacity(n);

        for (i, a) in sp.anchors.iter().enumerate() {
            // Compute tangent direction at this anchor
            let (tan_x, tan_y) = if n == 1 {
                (0.0, 1.0)
            } else if i == 0 {
                let next = &sp.anchors[1];
                (next.p.x - a.p.x, next.p.y - a.p.y)
            } else if i == n - 1 {
                let prev = &sp.anchors[n - 2];
                (a.p.x - prev.p.x, a.p.y - prev.p.y)
            } else {
                let prev = &sp.anchors[i - 1];
                let next = &sp.anchors[i + 1];
                (next.p.x - prev.p.x, next.p.y - prev.p.y)
            };

            let tan_len = (tan_x * tan_x + tan_y * tan_y).sqrt();
            let (unit_tan_x, unit_tan_y) = if tan_len > 1e-9 { (tan_x / tan_len, tan_y / tan_len) } else { (0.0, 1.0) };

            // Normal (-tan_y, tan_x)
            let mut norm_x = -unit_tan_y;
            let mut norm_y = unit_tan_x;

            // Inward direction points towards centerline_x
            let towards_center = match side {
                SectionSide::Right => -1.0, // x > CL, so inward is negative x
                SectionSide::Left => 1.0,   // x < CL, so inward is positive x
            };

            if norm_x * towards_center < 0.0 {
                norm_x = -norm_x;
                norm_y = -norm_y;
            }

            // If normal x is nearly zero, explicitly bias towards centerline
            if norm_x.abs() < 1e-6 {
                norm_x = towards_center;
                norm_y = 0.0;
            }

            let off_x = norm_x * thickness_pt;
            let off_y = norm_y * thickness_pt;

            let mut new_p = Point::new(a.p.x + off_x, a.p.y + off_y);
            let new_h_in = Point::new(a.h_in.x + off_x, a.h_in.y + off_y);
            let new_h_out = Point::new(a.h_out.x + off_x, a.h_out.y + off_y);

            // Prevent offset anchor from crossing past centerline
            match side {
                SectionSide::Right => {
                    if new_p.x < centerline_x {
                        new_p.x = centerline_x;
                    }
                }
                SectionSide::Left => {
                    if new_p.x > centerline_x {
                        new_p.x = centerline_x;
                    }
                }
            }

            new_anchors.push(Anchor { p: new_p, h_in: new_h_in, h_out: new_h_out, kind: a.kind });
        }

        new_subpaths.push(SubPath { anchors: new_anchors, closed: sp.closed });
    }

    PathData { subpaths: new_subpaths }
}

/// Builds a closed wall section polygon on the right side of the centerline.
pub fn build_section_polygon(outer: &PathData, inner: &PathData, centerline_x: f64, floor_thickness_pt: f64) -> PathData {
    let outer_anchors = outer.subpaths.first().map(|s| &s.anchors[..]).unwrap_or(&[]);
    if outer_anchors.is_empty() {
        return PathData::default();
    }

    // Ensure outer anchors are ordered from top (rim) to bottom (base)
    let outer_runs_down =
        if outer_anchors.len() >= 2 { outer_anchors.first().map_or(0.0, |a| a.p.y) <= outer_anchors.last().map_or(0.0, |a| a.p.y) } else { true };

    let ordered_outer: Vec<Anchor> =
        if outer_runs_down { outer_anchors.to_vec() } else { outer_anchors.iter().rev().map(|a| a.reversed()).collect() };

    let inner_computed;
    let inner_anchors = if let Some(sp) = inner.subpaths.first().filter(|s| !s.anchors.is_empty()) {
        &sp.anchors[..]
    } else {
        inner_computed = offset_wall_profile(outer, floor_thickness_pt, centerline_x, SectionSide::Right);
        inner_computed.subpaths.first().map(|s| &s.anchors[..]).unwrap_or(&[])
    };

    let inner_runs_down =
        if inner_anchors.len() >= 2 { inner_anchors.first().map_or(0.0, |a| a.p.y) <= inner_anchors.last().map_or(0.0, |a| a.p.y) } else { true };

    let ordered_inner: Vec<Anchor> =
        if inner_runs_down { inner_anchors.to_vec() } else { inner_anchors.iter().rev().map(|a| a.reversed()).collect() };

    let mut poly_anchors = Vec::new();

    // 1. Traverse outer wall from rim down to base
    poly_anchors.extend(ordered_outer.clone());

    // 2. Base bottom anchor at centerline
    let base_y = ordered_outer.last().map_or(0.0, |a| a.p.y);
    let floor_y = (base_y - floor_thickness_pt).max(ordered_outer.first().map_or(0.0, |a| a.p.y));

    poly_anchors.push(Anchor::corner(Point::new(centerline_x, base_y)));
    poly_anchors.push(Anchor::corner(Point::new(centerline_x, floor_y)));

    // 3. Traverse inner wall from base back up to rim (reversed)
    for a in ordered_inner.iter().rev() {
        poly_anchors.push(a.reversed());
    }

    // 4. Close lip back to outer rim
    let subpath = SubPath { anchors: poly_anchors, closed: true };

    PathData { subpaths: vec![subpath] }
}

/// Emits vector hatch lines at 45° within the section polygon.
pub fn generate_section_hatch(section_poly: &PathData, spacing_mm: f64, _weight_pt: f64, angle_deg: f64) -> BezPath {
    let bez = section_poly.to_bezpath();
    let spacing_pt = (spacing_mm * (72.0 / 25.4)).max(0.5);

    let dir = if (angle_deg - 135.0).abs() < 40.0 { HatchDirection::Deg135 } else { HatchDirection::Deg45 };

    crate::hatch::emit_hatch_lines(&bez, dir, spacing_pt)
}

/// Generates a horizontal ridge line connecting an inflection point to the vertical centerline.
pub fn generate_horizontal_ridge(point: Point, centerline_x: f64) -> Line {
    Line::new(Point::new(centerline_x, point.y), point)
}

/// Computes $D$ (rim diameter), $d$ (base diameter), $H$ (height), and $t$ (wall thickness).
pub fn compute_vessel_measurements(
    centerline_x: f64,
    outer: &PathData,
    inner: Option<&PathData>,
    rim_fit_diameter_pt: Option<f64>,
) -> PotteryMeasurements {
    let anchors: Vec<&Anchor> = outer.subpaths.iter().flat_map(|sp| &sp.anchors).collect();
    if anchors.is_empty() {
        return PotteryMeasurements::default();
    }

    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut rim_anchor_x = centerline_x;
    let mut base_anchor_x = centerline_x;
    let mut max_radius_pt: f64 = 0.0;

    for a in &anchors {
        let r = (a.p.x - centerline_x).abs();
        if r > max_radius_pt {
            max_radius_pt = r;
        }

        if a.p.y < min_y {
            min_y = a.p.y;
            rim_anchor_x = a.p.x;
        }
        if a.p.y > max_y {
            max_y = a.p.y;
            base_anchor_x = a.p.x;
        }
    }

    let height_pt = (max_y - min_y).max(0.0);
    let height_mm = height_pt * (25.4 / 72.0);

    let rim_diameter_pt = if let Some(d_fit) = rim_fit_diameter_pt { d_fit } else { 2.0 * (rim_anchor_x - centerline_x).abs() };
    let rim_diameter_mm = rim_diameter_pt * (25.4 / 72.0);

    let base_diameter_pt = 2.0 * (base_anchor_x - centerline_x).abs();
    let base_diameter_mm = base_diameter_pt * (25.4 / 72.0);

    let max_belly_diameter_mm = (2.0 * max_radius_pt) * (25.4 / 72.0);

    let wall_thickness_mm = inner.and_then(|in_p| {
        let in_anchors: Vec<&Anchor> = in_p.subpaths.iter().flat_map(|sp| &sp.anchors).collect();
        let in_rim = in_anchors.iter().min_by(|a, b| a.p.y.partial_cmp(&b.p.y).unwrap_or(std::cmp::Ordering::Equal))?;
        let t_pt = (rim_anchor_x - in_rim.p.x).abs();
        Some(t_pt * (25.4 / 72.0))
    });

    PotteryMeasurements {
        rim_diameter_mm: Some(rim_diameter_mm),
        base_diameter_mm: Some(base_diameter_mm),
        height_mm: Some(height_mm),
        max_belly_diameter_mm: Some(max_belly_diameter_mm),
        wall_thickness_mm,
        rim_preservation_percent: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflect_path_across_centerline() {
        // Outer profile with centerline at x = 300
        let a1 = Anchor::corner(Point::new(350.0, 100.0));
        let a2 = Anchor::corner(Point::new(380.0, 150.0));
        let a3 = Anchor::corner(Point::new(330.0, 200.0));

        let path = PathData { subpaths: vec![SubPath { anchors: vec![a1, a2, a3], closed: false }] };

        let reflected = reflect_path_across_centerline(&path, 300.0);
        let sp = &reflected.subpaths[0];
        assert_eq!(sp.anchors.len(), 3);
        // Reversed traversal: a3' (x = 270, y = 200), a2' (x = 220, y = 150), a1' (x = 250, y = 100)
        assert_eq!(sp.anchors[0].p, Point::new(270.0, 200.0));
        assert_eq!(sp.anchors[1].p, Point::new(220.0, 150.0));
        assert_eq!(sp.anchors[2].p, Point::new(250.0, 100.0));
    }

    #[test]
    fn test_offset_wall_profile() {
        let a1 = Anchor::corner(Point::new(350.0, 100.0));
        let a2 = Anchor::corner(Point::new(350.0, 200.0));
        let path = PathData { subpaths: vec![SubPath { anchors: vec![a1, a2], closed: false }] };

        // Offset 10pt inward towards centerline at 300
        let offset = offset_wall_profile(&path, 10.0, 300.0, SectionSide::Right);
        let sp = &offset.subpaths[0];
        assert_eq!(sp.anchors.len(), 2);
        assert!((sp.anchors[0].p.x - 340.0).abs() < 1e-6);
        assert!((sp.anchors[1].p.x - 340.0).abs() < 1e-6);
    }

    #[test]
    fn test_horizontal_ridge() {
        let pt = Point::new(360.0, 140.0);
        let line = generate_horizontal_ridge(pt, 300.0);
        assert_eq!(line.p0, Point::new(300.0, 140.0));
        assert_eq!(line.p1, pt);
    }

    #[test]
    fn test_compute_vessel_measurements_benchmark() {
        let mm_to_pt = 72.0 / 25.4;
        let rim_r_pt = 92.0 * mm_to_pt;
        let base_r_pt = 43.0 * mm_to_pt;
        let height_pt = 212.0 * mm_to_pt;
        let wall_pt = 7.0 * mm_to_pt;

        let outer = PathData {
            subpaths: vec![SubPath {
                anchors: vec![
                    Anchor::corner(Point::new(300.0 + rim_r_pt, 100.0)),
                    Anchor::corner(Point::new(300.0 + rim_r_pt * 1.1, 100.0 + height_pt * 0.5)),
                    Anchor::corner(Point::new(300.0 + base_r_pt, 100.0 + height_pt)),
                ],
                closed: false,
            }],
        };

        let inner = PathData {
            subpaths: vec![SubPath {
                anchors: vec![
                    Anchor::corner(Point::new(300.0 + rim_r_pt - wall_pt, 100.0)),
                    Anchor::corner(Point::new(300.0 + base_r_pt - wall_pt, 100.0 + height_pt - wall_pt)),
                ],
                closed: false,
            }],
        };

        let m = compute_vessel_measurements(300.0, &outer, Some(&inner), None);
        assert!((m.rim_diameter_mm.unwrap() - 184.0).abs() < 1e-4);
        assert!((m.height_mm.unwrap() - 212.0).abs() < 1e-4);
        assert!((m.base_diameter_mm.unwrap() - 86.0).abs() < 1e-4);
        assert!((m.wall_thickness_mm.unwrap() - 7.0).abs() < 1e-4);
    }
}
