//! Every public item is reachable at the path of the module that owns its domain.
//!
//! The paths are written out in full, so a module that stops exporting an item
//! or a root re-export that returns fails to compile here. The old flat root
//! paths are covered by the `compile_fail` example in the crate documentation.

use std::mem::size_of;

macro_rules! sized {
    ($($path:ty),+ $(,)?) => {
        $( assert!(size_of::<$path>() < usize::MAX, stringify!($path)); )+
    };
}

#[test]
fn the_crate_root_holds_only_the_shared_error() {
    sized!(surgeist_paint::PaintError);
}

#[test]
fn stacking_exports_context_planning() {
    use surgeist_paint::stacking as m;
    sized!(
        m::BoxId,
        m::BoxFacts,
        m::ChildOrder,
        m::Participation,
        m::Position,
        m::ZIndex,
        m::ContextEffect,
        m::StackingPlan<'static>,
        m::ContextView<'static>,
        m::StackingOutcome<'static>,
    );
    let _ = m::plan_stacking;
}

#[test]
fn paint_order_exports_request_order_and_views() {
    use surgeist_paint::paint_order as m;
    sized!(
        m::PaintOrderOutcome<'static>,
        m::PaintOrder<'static>,
        m::PaintOrderEvent<'static>,
        m::PaintRequest<'static>,
        m::BoxPaint,
        m::BoxPaintKind,
        m::FloatParticipation,
        m::TextSource,
        m::PaintLine,
        m::PaintFragment,
        m::FragmentOwner,
        m::BlockInInlinePlacement,
        m::PaintPrimitive,
        m::PrimitiveKind,
        m::CanvasPaintKind,
        m::FragmentPaintKind,
        m::DecorationKind,
        m::TextId,
        m::LineId,
        m::FragmentId,
        m::PrimitiveId,
        m::SourceChild,
        m::ContentEntry,
        m::BoxView<'static>,
        m::LineView<'static>,
        m::FragmentView<'static>,
        m::PrimitiveView<'static>,
        m::PaintInputError,
        m::BoxCategoryError,
    );
    let _ = m::plan_paint_order;
}

#[test]
fn accounting_exports_limits_options_and_reports() {
    use surgeist_paint::accounting as m;
    sized!(
        m::PaintOptions,
        m::PaintLimits,
        m::WorkKind,
        m::ResourceKind,
        m::PaintPhase,
        m::PaintReport,
        m::PhaseReport<'static>,
        m::PaintEvent,
        m::PaintChange,
        m::BudgetRefusal,
        m::AccountingFailure,
    );
}

#[test]
fn linear_gradient_exports_line_geometry() {
    use surgeist_paint::linear_gradient as m;
    sized!(
        m::LinearGradientDirection,
        m::LinearGradientLine,
        m::LinearGradientOutcome,
        m::GradientGeometryError,
    );
    let _ = m::resolve_linear_gradient;
}

#[test]
fn gradient_stops_exports_used_positions() {
    use surgeist_paint::gradient_stops as m;
    sized!(
        m::GradientStopItem<()>,
        m::GradientStopPositions,
        m::GradientPosition,
        m::GradientPositionComponent,
        m::GradientStops<'static, ()>,
        m::GradientStopsOutcome<'static, ()>,
        m::GradientRecord<'static, ()>,
        m::UsedGradientItem<'static, ()>,
        m::UsedGradientStop<'static, ()>,
        m::GradientStopsError,
    );
    let _ = m::resolve_gradient_stops::<()>;
}

#[test]
fn radial_gradient_exports_size_resolution() {
    use surgeist_paint::radial_gradient as m;
    sized!(
        m::RadialGradientSize,
        m::RadialGradientExtent,
        m::RadialGradientGeometry,
        m::RadialGradientCase,
        m::RadialGradientOutcome,
        m::RadialGradientError,
    );
    let _ = m::resolve_radial_gradient;
}

#[test]
fn zero_height_gradient_exports_the_color_measure() {
    use surgeist_paint::zero_height_gradient as m;
    sized!(
        m::GradientRepeat,
        m::WeightedGradientColor<'static, ()>,
        m::WeightedGradientRecord<'static, ()>,
        m::ZeroHeightGradientColors<'static, ()>,
        m::ZeroHeightGradientColorsOutcome<'static, ()>,
        m::ZeroHeightGradientError,
    );
    let _ = m::resolve_zero_height_gradient_colors::<()>;
}

#[test]
fn border_image_exports_areas_partitions_and_tiles() {
    use surgeist_paint::border_image as m;
    sized!(
        m::BorderImageRequest,
        m::BorderImageAreaRequest,
        m::BorderImagePartitionRequest,
        m::BorderImageSource,
        m::BorderImageSlice,
        m::BorderImageWidth,
        m::BorderImageOutset,
        m::BorderImageRepeat,
        m::BorderImageArea,
        m::BorderImageRect,
        m::BorderImageRegion,
        m::BorderImageGeometry,
        m::BorderImageTiling,
        m::BorderImageTileRegion,
        m::BorderImageAxisPattern,
        m::BorderImageAxisPlacement,
        m::BorderImageAreaOutcome,
        m::BorderImageOutcome,
        m::BorderImageTilingOutcome,
        m::MaskBorderPlan,
        m::MaskBorderOutcome,
        m::BorderImageGeometryError,
    );
    let _ = m::resolve_border_image;
    let _ = m::resolve_border_image_area;
    let _ = m::resolve_border_image_in_area;
    let _ = m::resolve_border_image_tiling;
    let _ = m::resolve_mask_border_in_area;
}
