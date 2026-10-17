//! Hatching geometry algorithms (Trowel, design 2.1 §4, §7, §14):
//! - Planar face adjacency graph generation.
//! - Bipartite dual-graph 2-coloring for alternating hatch (`Alternate` 45° vs 135°).
//! - Odd-cycle conflict detection (3+ mutually adjacent touching faces flag `!` warning badge).
//! - Single-click flip toggle support.
//! - Real vector line emission clipped to planar face boundaries at 1.0 mm paper spacing.

use kurbo::{BezPath, PathEl, Point, Rect, Shape as _};
use vectorcraft_doc::hatch::{HatchDirection, HatchStyle};

/// A planar face for hatching computation.
#[derive(Clone, Debug, PartialEq)]
pub struct HatchFace {
    pub id: usize,
    pub outline: BezPath,
    pub style: HatchStyle,
}

impl HatchFace {
    pub fn new(id: usize, outline: BezPath, style: HatchStyle) -> Self {
        Self { id, outline, style }
    }
}

/// The result of resolving hatching directions across a collection of faces.
#[derive(Clone, Debug, PartialEq)]
pub struct HatchFaceResult {
    pub face_index: usize,
    /// Resolved effective direction (Deg45, Deg135, or None).
    pub direction: HatchDirection,
    /// Unflipped base direction assigned by the 2-coloring solver (Deg45 or Deg135).
    pub base_direction: HatchDirection,
    /// True if an odd-cycle conflict was detected in this face's component.
    pub conflict: bool,
    /// The generated vector hatch lines for this face.
    pub lines: BezPath,
}

/// Builds an adjacency graph for closed planar faces.
/// Two faces are adjacent if their bounding boxes overlap and they share a boundary
/// segment of length >= `min_overlap` (default 0.1 pt).
pub fn build_adjacency_graph(faces: &[BezPath], min_overlap: f64) -> Vec<Vec<usize>> {
    let n = faces.len();
    let mut adj = vec![Vec::new(); n];
    if n < 2 {
        return adj;
    }

    // Extract linearized line segments for each face
    let segments_per_face: Vec<Vec<(Point, Point)>> = faces.iter().map(|f| extract_segments(f, 0.25)).collect();
    let bounds_per_face: Vec<Rect> = faces.iter().map(|f| f.bounding_box()).collect();

    for i in 0..n {
        let b_i = match bounds_per_face.get(i) {
            Some(b) => *b,
            None => continue,
        };
        for j in (i + 1)..n {
            let b_j = match bounds_per_face.get(j) {
                Some(b) => *b,
                None => continue,
            };
            // Check bounding box intersection with a small padding
            let padded_i = Rect::new(b_i.x0 - 1.0, b_i.y0 - 1.0, b_i.x1 + 1.0, b_i.y1 + 1.0);
            if padded_i.intersect(b_j).area() < 0.0 && !padded_i.overlaps(b_j) {
                continue;
            }

            let segs_i = match segments_per_face.get(i) {
                Some(s) => s.as_slice(),
                None => continue,
            };
            let segs_j = match segments_per_face.get(j) {
                Some(s) => s.as_slice(),
                None => continue,
            };

            if faces_share_boundary(segs_i, segs_j, min_overlap) {
                if let Some(list_i) = adj.get_mut(i) {
                    list_i.push(j);
                }
                if let Some(list_j) = adj.get_mut(j) {
                    list_j.push(i);
                }
            }
        }
    }

    adj
}

/// Flattens a BezPath into linear line segments, automatically closing open polygonal subpaths.
fn extract_segments(path: &BezPath, tol: f64) -> Vec<(Point, Point)> {
    let mut segs = Vec::new();
    let mut start_pt = Point::ZERO;
    let mut curr_pt = Point::ZERO;
    let mut subpath_segs = 0usize;

    kurbo::flatten(path.iter(), tol, |el| match el {
        PathEl::MoveTo(p) => {
            if subpath_segs >= 2 && curr_pt.distance(start_pt) > 1e-6 {
                segs.push((curr_pt, start_pt));
            }
            start_pt = p;
            curr_pt = p;
            subpath_segs = 0;
        }
        PathEl::LineTo(p) => {
            if curr_pt.distance(p) > 1e-6 {
                segs.push((curr_pt, p));
                subpath_segs += 1;
            }
            curr_pt = p;
        }
        PathEl::ClosePath => {
            if curr_pt.distance(start_pt) > 1e-6 {
                segs.push((curr_pt, start_pt));
            }
            curr_pt = start_pt;
            subpath_segs = 0;
        }
        _ => {}
    });

    if subpath_segs >= 2 && curr_pt.distance(start_pt) > 1e-6 {
        segs.push((curr_pt, start_pt));
    }

    segs
}

/// Checks if two sets of segments share an overlapping boundary segment of length >= min_overlap.
fn faces_share_boundary(segs1: &[(Point, Point)], segs2: &[(Point, Point)], min_overlap: f64) -> bool {
    const DIST_TOL: f64 = 0.5; // points tolerance for collinearity

    for &(p1, p2) in segs1 {
        let v1 = p2 - p1;
        let len1 = v1.hypot();
        if len1 < 1e-6 {
            continue;
        }
        let u1 = v1 / len1;

        for &(q1, q2) in segs2 {
            let v2 = q2 - q1;
            let len2 = v2.hypot();
            if len2 < 1e-6 {
                continue;
            }

            // Check if segments are roughly parallel or anti-parallel
            let dot = u1.x * (v2.x / len2) + u1.y * (v2.y / len2);
            if dot.abs() < 0.98 {
                continue;
            }

            // Check distance of q1 and q2 from line p1->p2
            let d1 = perp_dist(q1, p1, u1);
            let d2 = perp_dist(q2, p1, u1);
            if d1 > DIST_TOL || d2 > DIST_TOL {
                continue;
            }

            // Project onto p1->p2 axis
            let t_q1 = (q1 - p1).dot(u1);
            let t_q2 = (q2 - p1).dot(u1);
            let t_min_q = t_q1.min(t_q2);
            let t_max_q = t_q1.max(t_q2);

            let overlap_start = 0.0_f64.max(t_min_q);
            let overlap_end = len1.min(t_max_q);

            if overlap_end - overlap_start >= min_overlap {
                return true;
            }
        }
    }
    false
}

fn perp_dist(pt: Point, line_pt: Point, line_dir: kurbo::Vec2) -> f64 {
    let v = pt - line_pt;
    (v.x * line_dir.y - v.y * line_dir.x).abs()
}

/// Component-isolated 2-coloring of an adjacency graph for alternating hatch (§14(e)).
/// Returns `(assignments: Vec<bool>, face_conflicts: Vec<bool>)`.
/// - False = 45°, True = 135°.
/// - `face_conflicts[i]` is true if and only if face `i` belongs to a connected component
///   containing an odd cycle (non-bipartite component).
pub fn bipartite_2color_components(adj: &[Vec<usize>]) -> (Vec<bool>, Vec<bool>) {
    let num_faces = adj.len();
    let mut color = vec![None; num_faces];
    let mut face_conflicts = vec![false; num_faces];

    for start in 0..num_faces {
        if color.get(start).copied().flatten().is_some() {
            continue;
        }

        let mut comp_nodes = Vec::new();
        let mut comp_conflict = false;

        if let Some(c) = color.get_mut(start) {
            *c = Some(false);
        }
        comp_nodes.push(start);

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);

        while let Some(u) = queue.pop_front() {
            let c: bool = color.get(u).copied().flatten().unwrap_or_default();
            let neighbors = match adj.get(u) {
                Some(list) => list.clone(),
                None => Vec::new(),
            };
            for v in neighbors {
                if v >= num_faces {
                    continue;
                }
                if let Some(&Some(vc)) = color.get(v) {
                    if vc == c {
                        comp_conflict = true; // Odd cycle detected in this component
                    }
                } else if let Some(slot) = color.get_mut(v) {
                    *slot = Some(!c);
                    comp_nodes.push(v);
                    queue.push_back(v);
                }
            }
        }

        if comp_conflict {
            for node in comp_nodes {
                if let Some(slot) = face_conflicts.get_mut(node) {
                    *slot = true;
                }
            }
        }
    }

    let assignments = color.into_iter().map(|c| c.unwrap_or(false)).collect();
    (assignments, face_conflicts)
}

/// 2-coloring of an adjacency graph for alternating hatch (§14(e)).
/// Returns (assignments: Vec<bool>, has_conflict: bool) where has_conflict indicates
/// if any connected component in the graph contains an odd cycle.
/// False = 45°, True = 135°.
/// Maintained as a backwards-compatible wrapper around `bipartite_2color_components`.
pub fn bipartite_2color(adj: &[Vec<usize>]) -> (Vec<bool>, bool) {
    let (assignments, conflicts) = bipartite_2color_components(adj);
    let has_conflict = conflicts.into_iter().any(|c| c);
    (assignments, has_conflict)
}

/// Resolves effective hatch directions across faces and emits vector hatch lines.
pub fn solve_face_hatching(faces: &[BezPath], styles: &[HatchStyle], adjacencies: Option<&[Vec<usize>]>) -> Vec<HatchFaceResult> {
    let n = faces.len();
    let computed_adj;
    let adj = match adjacencies {
        Some(a) => a,
        None => {
            computed_adj = build_adjacency_graph(faces, 0.1);
            &computed_adj
        }
    };

    let (coloring, face_conflicts) = bipartite_2color_components(adj);
    let mut results = Vec::with_capacity(n);

    for i in 0..n {
        let face = match faces.get(i) {
            Some(f) => f,
            None => continue,
        };
        let style = styles.get(i).cloned().unwrap_or_default();

        let (effective_dir, base_dir) = match style.direction {
            HatchDirection::Deg0 => {
                let base = HatchDirection::Deg0;
                let eff = if style.flipped { HatchDirection::Deg90 } else { HatchDirection::Deg0 };
                (eff, base)
            }
            HatchDirection::Deg45 => {
                let base = HatchDirection::Deg45;
                let eff = if style.flipped { HatchDirection::Deg135 } else { HatchDirection::Deg45 };
                (eff, base)
            }
            HatchDirection::Deg90 => {
                let base = HatchDirection::Deg90;
                let eff = if style.flipped { HatchDirection::Deg0 } else { HatchDirection::Deg90 };
                (eff, base)
            }
            HatchDirection::Deg135 => {
                let base = HatchDirection::Deg135;
                let eff = if style.flipped { HatchDirection::Deg45 } else { HatchDirection::Deg135 };
                (eff, base)
            }
            HatchDirection::Cross => (HatchDirection::Cross, HatchDirection::Cross),
            HatchDirection::DiagonalCross => (HatchDirection::DiagonalCross, HatchDirection::DiagonalCross),
            HatchDirection::Alternate => {
                let is_135 = coloring.get(i).copied().unwrap_or(false);
                let base = if is_135 { HatchDirection::Deg135 } else { HatchDirection::Deg45 };
                let eff = if style.flipped { base.inverted() } else { base };
                (eff, base)
            }
            HatchDirection::None => (HatchDirection::None, HatchDirection::None),
        };

        let lines = if effective_dir != HatchDirection::None && style.is_hatched() {
            emit_hatch_lines(face, effective_dir, style.spacing_pt())
        } else {
            BezPath::new()
        };

        let is_conflict = face_conflicts.get(i).copied().unwrap_or(false);
        results.push(HatchFaceResult {
            face_index: i,
            direction: effective_dir,
            base_direction: base_dir,
            conflict: is_conflict && style.direction == HatchDirection::Alternate,
            lines,
        });
    }

    results
}

/// Document-level hatching resolution results across all Alternate-hatched faces.
#[derive(Clone, Debug, Default)]
pub struct DocumentHatchResolution {
    /// Map from NodeId to effective hatch direction (Deg45 or Deg135), including flip.
    pub effective_dirs: std::collections::HashMap<vectorcraft_doc::NodeId, HatchDirection>,
    /// Map from NodeId to base dual-coloring direction (Deg45 or Deg135), excluding flip.
    pub base_dirs: std::collections::HashMap<vectorcraft_doc::NodeId, HatchDirection>,
    /// List of NodeIds in odd-cycle conflict components.
    pub conflicts: Vec<vectorcraft_doc::NodeId>,
}

/// Resolves 2-coloring for all Alternate hatch nodes across the entire document.
pub fn resolve_document_hatching(doc: &vectorcraft_doc::Document) -> DocumentHatchResolution {
    let mut node_ids = Vec::new();
    let mut faces = Vec::new();
    let mut styles = Vec::new();

    doc.walk(|n| {
        let Some(hatch) = &n.hatch else { return };
        if hatch.direction != HatchDirection::Alternate {
            return;
        }
        let outline = match &n.kind {
            vectorcraft_doc::NodeKind::Path { path, .. } => path.to_bezpath(),
            vectorcraft_doc::NodeKind::Compound { children, .. } => {
                let mut bp = BezPath::new();
                for c in children {
                    if let Some(pd) = c.path_data() {
                        bp.extend(pd.to_bezpath());
                    }
                }
                bp
            }
            _ => return,
        };
        if !outline.elements().is_empty() {
            faces.push(outline);
            styles.push(hatch.clone());
            node_ids.push(n.id);
        }
    });

    if faces.is_empty() {
        return DocumentHatchResolution::default();
    }

    let results = solve_face_hatching(&faces, &styles, None);
    let mut effective_dirs = std::collections::HashMap::with_capacity(results.len());
    let mut base_dirs = std::collections::HashMap::with_capacity(results.len());
    let mut conflicts = Vec::new();

    for r in results {
        if let Some(&id) = node_ids.get(r.face_index) {
            effective_dirs.insert(id, r.direction);
            base_dirs.insert(id, r.base_direction);
            if r.conflict {
                conflicts.push(id);
            }
        }
    }

    DocumentHatchResolution { effective_dirs, base_dirs, conflicts }
}

/// Helper to emit single-angle parallel scanlines clipped to polygon boundary.
fn emit_single_angle_hatch_lines(boundary: &BezPath, angle_rad: f64, spacing_pt: f64) -> BezPath {
    let mut result = BezPath::new();
    if spacing_pt <= 0.001 {
        return result;
    }

    let segments = extract_segments(boundary, 0.1);
    if segments.is_empty() {
        return result;
    }

    let cos_a = (-angle_rad).cos();
    let sin_a = (-angle_rad).sin();
    let rot_forward_cos = angle_rad.cos();
    let rot_forward_sin = angle_rad.sin();

    let rotate_neg = |p: Point| Point::new(p.x * cos_a - p.y * sin_a, p.x * sin_a + p.y * cos_a);
    let rotate_pos = |p: Point| Point::new(p.x * rot_forward_cos - p.y * rot_forward_sin, p.x * rot_forward_sin + p.y * rot_forward_cos);

    let rot_segments: Vec<(Point, Point)> = segments.iter().map(|&(p1, p2)| (rotate_neg(p1), rotate_neg(p2))).collect();

    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for &(p1, p2) in &rot_segments {
        y_min = y_min.min(p1.y).min(p2.y);
        y_max = y_max.max(p1.y).max(p2.y);
    }

    if !y_min.is_finite() || !y_max.is_finite() || y_max <= y_min {
        return result;
    }

    let start_idx = (y_min / spacing_pt).floor() as i64 - 1;
    let end_idx = (y_max / spacing_pt).ceil() as i64 + 1;

    for k in start_idx..=end_idx {
        let scan_y = k as f64 * spacing_pt;
        if scan_y < y_min - 1e-4 || scan_y > y_max + 1e-4 {
            continue;
        }

        let mut x_intersections = Vec::new();
        for &(p1, p2) in &rot_segments {
            let (min_y, max_y) = if p1.y <= p2.y { (p1.y, p2.y) } else { (p2.y, p1.y) };
            if (max_y - min_y).abs() < 1e-9 {
                continue;
            }
            if scan_y >= min_y && scan_y < max_y {
                let t = (scan_y - p1.y) / (p2.y - p1.y);
                let x = p1.x + t * (p2.x - p1.x);
                x_intersections.push(x);
            }
        }

        x_intersections.sort_by(|a, b| a.total_cmp(b));

        let mut idx = 0;
        while idx + 1 < x_intersections.len() {
            let x1 = match x_intersections.get(idx) {
                Some(v) => *v,
                None => break,
            };
            let x2 = match x_intersections.get(idx + 1) {
                Some(v) => *v,
                None => break,
            };
            idx += 2;

            if x2 - x1 > 1e-6 {
                let doc_p1 = rotate_pos(Point::new(x1, scan_y));
                let doc_p2 = rotate_pos(Point::new(x2, scan_y));
                result.move_to(doc_p1);
                result.line_to(doc_p2);
            }
        }
    }

    result
}

/// Emits vector hatch lines clipped to a closed boundary polygon at the given spacing.
/// Rotates the boundary by -theta, slices with horizontal scanlines, pairs up even-odd
/// intersections, and rotates back by +theta.
pub fn emit_hatch_lines(boundary: &BezPath, direction: HatchDirection, spacing_pt: f64) -> BezPath {
    match direction {
        HatchDirection::Deg0 => emit_single_angle_hatch_lines(boundary, 0.0, spacing_pt),
        HatchDirection::Deg45 => emit_single_angle_hatch_lines(boundary, std::f64::consts::FRAC_PI_4, spacing_pt),
        HatchDirection::Deg90 => emit_single_angle_hatch_lines(boundary, std::f64::consts::FRAC_PI_2, spacing_pt),
        HatchDirection::Deg135 => emit_single_angle_hatch_lines(boundary, 3.0 * std::f64::consts::FRAC_PI_4, spacing_pt),
        HatchDirection::Cross => {
            let mut lines = emit_single_angle_hatch_lines(boundary, 0.0, spacing_pt);
            lines.extend(emit_single_angle_hatch_lines(boundary, std::f64::consts::FRAC_PI_2, spacing_pt));
            lines
        }
        HatchDirection::DiagonalCross => {
            let mut lines = emit_single_angle_hatch_lines(boundary, std::f64::consts::FRAC_PI_4, spacing_pt);
            lines.extend(emit_single_angle_hatch_lines(boundary, 3.0 * std::f64::consts::FRAC_PI_4, spacing_pt));
            lines
        }
        HatchDirection::Alternate | HatchDirection::None => BezPath::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_adjacent_regions_alternating() {
        let mut r1 = BezPath::new();
        r1.move_to((0.0, 0.0));
        r1.line_to((10.0, 0.0));
        r1.line_to((10.0, 10.0));
        r1.line_to((0.0, 10.0));
        r1.close_path();

        let mut r2 = BezPath::new();
        r2.move_to((10.0, 0.0));
        r2.line_to((20.0, 0.0));
        r2.line_to((20.0, 10.0));
        r2.line_to((10.0, 10.0));
        r2.close_path();

        let faces = vec![r1, r2];
        let styles = vec![HatchStyle::cut_face(), HatchStyle::cut_face()];

        let res = solve_face_hatching(&faces, &styles, None);
        assert_eq!(res.len(), 2);
        assert!(!res[0].conflict);
        assert!(!res[1].conflict);
        // They must alternate: one 45°, one 135°
        assert_ne!(res[0].direction, res[1].direction);
        assert!(matches!(res[0].direction, HatchDirection::Deg45 | HatchDirection::Deg135));
        assert!(matches!(res[1].direction, HatchDirection::Deg45 | HatchDirection::Deg135));
    }

    #[test]
    fn test_three_mutually_touching_regions_odd_cycle_conflict() {
        // K3 triangle: 3 mutually adjacent touching faces
        let mut r1 = BezPath::new();
        r1.move_to((0.0, 0.0));
        r1.line_to((10.0, 0.0));
        r1.line_to((5.0, 10.0));
        r1.close_path();

        let mut r2 = BezPath::new();
        r2.move_to((10.0, 0.0));
        r2.line_to((20.0, 0.0));
        r2.line_to((15.0, 10.0));
        r2.close_path();

        // r3 touches both r1 and r2 along edges
        let mut r3 = BezPath::new();
        r3.move_to((5.0, 10.0));
        r3.line_to((10.0, 0.0));
        r3.line_to((15.0, 10.0));
        r3.close_path();

        let faces = vec![r1, r2, r3];
        let styles = vec![HatchStyle::cut_face(), HatchStyle::cut_face(), HatchStyle::cut_face()];

        // Explicit adjacency for K3: 0-1, 1-2, 2-0
        let adj = vec![vec![1, 2], vec![0, 2], vec![0, 1]];
        let res = solve_face_hatching(&faces, &styles, Some(&adj));
        assert_eq!(res.len(), 3);
        // Odd cycle conflict MUST be detected
        assert!(res.iter().any(|r| r.conflict), "K3 odd-cycle conflict must flag conflict badge");
    }

    #[test]
    fn test_flip_toggle() {
        let mut r = BezPath::new();
        r.move_to((0.0, 0.0));
        r.line_to((10.0, 0.0));
        r.line_to((10.0, 10.0));
        r.line_to((0.0, 10.0));
        r.close_path();

        let mut s_normal = HatchStyle::cut_face();
        s_normal.direction = HatchDirection::Deg45;
        s_normal.flipped = false;

        let mut s_flipped = s_normal.clone();
        s_flipped.flipped = true;

        let res1 = solve_face_hatching(&[r.clone()], &[s_normal], None);
        let res2 = solve_face_hatching(&[r], &[s_flipped], None);

        assert_eq!(res1[0].direction, HatchDirection::Deg45);
        assert_eq!(res2[0].direction, HatchDirection::Deg135);
    }

    #[test]
    fn test_vector_line_emission_spacing() {
        let mut r = BezPath::new();
        r.move_to((0.0, 0.0));
        r.line_to((100.0, 0.0));
        r.line_to((100.0, 100.0));
        r.line_to((0.0, 100.0));
        r.close_path();

        let style = HatchStyle::cut_face();
        assert!((style.spacing_mm - 1.0).abs() < 1e-6);
        let spacing_pt = style.spacing_pt();
        // 1.0 mm = 72 / 25.4 pt approx 2.8346 pt
        assert!((spacing_pt - 2.83464567).abs() < 1e-4);

        let lines = emit_hatch_lines(&r, HatchDirection::Deg45, spacing_pt);
        assert!(!lines.elements().is_empty());

        // Count segments: in 100x100 square, diagonal extent is 100 * sqrt(2) approx 141.4 pt
        // With spacing 2.8346 pt, expected lines is approx 141.4 / 2.8346 approx 50 lines
        let move_count = lines.elements().iter().filter(|el| matches!(el, PathEl::MoveTo(_))).count();
        assert!((40..=60).contains(&move_count), "Expected approx 50 hatch lines, got {move_count}");
    }

    #[test]
    fn test_bipartite_2color_components_disconnected_isolation() {
        // Component 1: K3 (0-1-2) with odd cycle
        // Component 2: P2 (3-4) bipartite path
        let adj = vec![
            vec![1, 2], // 0
            vec![0, 2], // 1
            vec![0, 1], // 2
            vec![4],    // 3
            vec![3],    // 4
        ];
        let (assignments, conflicts) = bipartite_2color_components(&adj);
        assert_eq!(assignments.len(), 5);
        assert_eq!(conflicts.len(), 5);

        // Component 1 MUST have conflict
        assert!(conflicts[0]);
        assert!(conflicts[1]);
        assert!(conflicts[2]);

        // Component 2 MUST NOT have conflict
        assert!(!conflicts[3]);
        assert!(!conflicts[4]);

        // Component 2 must alternate colors
        assert_ne!(assignments[3], assignments[4]);

        // Legacy wrapper must report global conflict
        let (_legacy_assign, legacy_conflict) = bipartite_2color(&adj);
        assert!(legacy_conflict);
    }

    #[test]
    fn test_emit_hatch_lines_extended_directions() {
        let mut r = BezPath::new();
        r.move_to((0.0, 0.0));
        r.line_to((100.0, 0.0));
        r.line_to((100.0, 100.0));
        r.line_to((0.0, 100.0));
        r.close_path();

        let spacing_pt = 10.0;

        let lines_0 = emit_hatch_lines(&r, HatchDirection::Deg0, spacing_pt);
        assert!(!lines_0.elements().is_empty());
        let count_0 = lines_0.elements().iter().filter(|el| matches!(el, PathEl::MoveTo(_))).count();
        assert!((8..=12).contains(&count_0), "Deg0 lines count: {count_0}");

        let lines_90 = emit_hatch_lines(&r, HatchDirection::Deg90, spacing_pt);
        assert!(!lines_90.elements().is_empty());
        let count_90 = lines_90.elements().iter().filter(|el| matches!(el, PathEl::MoveTo(_))).count();
        assert!((8..=12).contains(&count_90), "Deg90 lines count: {count_90}");

        let lines_cross = emit_hatch_lines(&r, HatchDirection::Cross, spacing_pt);
        let count_cross = lines_cross.elements().iter().filter(|el| matches!(el, PathEl::MoveTo(_))).count();
        assert_eq!(count_cross, count_0 + count_90, "Cross must equal Deg0 + Deg90 lines");

        let lines_diag = emit_hatch_lines(&r, HatchDirection::DiagonalCross, spacing_pt);
        assert!(!lines_diag.elements().is_empty());
    }
}
