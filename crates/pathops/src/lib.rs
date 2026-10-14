//! VectorCraft path operations: booleans (Pathfinder), shape builder regions, offset, outline stroke,
//! simplify and the other Object → Path commands.
//!
//! Booleans are curve-preserving: `linesweeper`'s robust sweep-line works directly on cubic
//! Béziers, and the pieces it splits curves into are refitted afterwards so results carry few
//! anchors. See the crate README for the API overview.
#![forbid(unsafe_code)]

mod boolean;
mod edit;
pub mod elevation;
mod fit;
pub mod hatch;
mod offset;
mod pathfinder;
pub mod plan;
mod planar;
pub mod pottery;

pub use boolean::{BoolOp, DEFAULT_PRECISION, area, boolean, boolean_n, normalize, try_boolean, try_normalize, unite_all};
pub use edit::{
    AverageAxis, SimplifyOptions, add_anchor_points, average, join, remove_anchor, remove_redundant_points, simplify, simplify_with, smooth,
    split_into_grid,
};
pub use elevation::{ElevationLabel, ElevationOutput, ElevationSegment, clip_elevation_scanline, generate_elevation};
pub use hatch::{
    DocumentHatchResolution, HatchFace, HatchFaceResult, bipartite_2color, bipartite_2color_components, build_adjacency_graph, emit_hatch_lines,
    resolve_document_hatching, solve_face_hatching,
};
pub use offset::{Cap, Join, offset_path, outline_stroke, stroke_region};
pub use pathfinder::{FaceMerger, PathfinderOp, Region, Shape, merge_regions, pathfinder, region_at, regions};
pub use plan::{
    DEFAULT_HACHURE_SHORT_RATIO, DEFAULT_HACHURE_SPACING_MM, DEFAULT_PLAN_WEIGHT_PT, DepthMarkerGeometry,
    DepthMarkerKind, DepthMarkerSpec, FeatureStats, HOUSE_1_POLYGON_METERS, SECTION_LINE_DASH_PATTERN,
    SectionLineGeometry, SectionLineSpec, SlopeHachureSpec, SpotElevationGeometry, SpotMarkerKind,
    SpotElevationSpec, ViewDirection, compute_polygon_stats, generate_depth_marker_geometry,
    generate_section_line_geometry, generate_slope_hachures, generate_spot_elevation_geometry,
};
pub use planar::{BuilderArrangement, SHAPE_BUILDER_MAX_SEGMENTS, cut_out, encloses_area, interior_point, live_paint, shape_builder};
pub use pottery::{
    build_section_polygon, compute_vessel_measurements, generate_horizontal_ridge,
    generate_section_hatch, offset_wall_profile, reflect_path_across_centerline,
};

/// Errors from fallible operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PathOpsError {
    #[error("input contains NaN or infinite coordinates")]
    NonFinite,
    #[error("input path could not be closed")]
    OpenPath,
    #[error("the shapes are too degenerate to combine")]
    Degenerate,
}
