#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod accounting;
mod algorithm;
mod border_image;
mod gradient;
mod gradient_stops;
mod model;
mod paint_order;
mod radial_gradient;
mod zero_height_gradient;

pub use accounting::{
    AccountingFailure, BudgetRefusal, PaintChange, PaintEvent, PaintLimits, PaintOptions,
    PaintPhase, PaintReport, PhaseReport, RefusalReason, ResourceKind, WorkKind,
};
pub use algorithm::{PaintOutcome, plan_stacking};
pub use border_image::{
    BorderImageArea, BorderImageAreaOutcome, BorderImageAreaRequest, BorderImageGeometry,
    BorderImageGeometryError, BorderImageOutcome, BorderImageOutset, BorderImagePartitionRequest,
    BorderImageRect, BorderImageRegion, BorderImageRequest, BorderImageSlice, BorderImageSource,
    BorderImageWidth, resolve_border_image, resolve_border_image_area,
    resolve_border_image_in_area,
};
pub use gradient::{
    GradientGeometryError, LinearGradientDirection, LinearGradientLine, LinearGradientOutcome,
    resolve_linear_gradient,
};
pub use gradient_stops::{
    GradientPosition, GradientPositionComponent, GradientRecord, GradientStopItem,
    GradientStopPositions, GradientStops, GradientStopsError, GradientStopsOutcome,
    UsedGradientItem, UsedGradientStop, resolve_gradient_stops,
};
pub use model::{
    BoxFacts, BoxId, ChildOrder, ContextEffect, ContextView, PaintError, Participation, Position,
    StackingPlan, ZIndex,
};
pub use paint_order::{
    BlockInInlinePlacement, BoxCategoryError, BoxPaint, BoxPaintKind, BoxView, CanvasPaintKind,
    ContentEntry, DecorationKind, FloatParticipation, FragmentId, FragmentOwner, FragmentPaintKind,
    FragmentView, LineId, LineView, PaintFragment, PaintInputError, PaintLine, PaintOrder,
    PaintOrderEvent, PaintOrderOutcome, PaintPrimitive, PaintRequest, PrimitiveId, PrimitiveKind,
    PrimitiveView, SourceChild, TextId, TextSource, plan_paint_order,
};
pub use radial_gradient::{
    RadialGradientCase, RadialGradientError, RadialGradientExtent, RadialGradientGeometry,
    RadialGradientOutcome, RadialGradientSize, resolve_radial_gradient,
};
pub use surgeist_ledger::{EventFilter, ProfileMode};

pub use zero_height_gradient::{
    GradientRepeat, WeightedGradientColor, WeightedGradientRecord, ZeroHeightGradientColors,
    ZeroHeightGradientColorsOutcome, ZeroHeightGradientError, resolve_zero_height_gradient_colors,
};
