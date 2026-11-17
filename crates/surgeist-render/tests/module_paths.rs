//! Every public item of the crate is imported once through its module path.
//!
//! The crate root holds only `Error`, `ErrorCode` and `Result`; every other item lives in one module, and a
//! module that stops exporting an item makes this file fail to compile. The sections for `text_layout` and
//! `paint_gradient` follow the features that add those modules. This file exists only with the module paths, so
//! it has no earlier failing form.
#![allow(unused_imports)]

use surgeist_render::{Error, ErrorCode, Result};

use surgeist_render::accounting::{
    RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
    RenderAccountingSetupError, RenderAttemptKind, RenderAttemptReport, RenderAttemptStatus,
    RenderChangeKind, RenderClock, RenderCpuEvent, RenderCpuIdentity, RenderCpuInterval,
    RenderCpuScope, RenderCpuSnapshot, RenderCpuUsage, RenderDeviceSnapshot, RenderEvent,
    RenderFrameReport, RenderFrameReportRef, RenderGenerationEvent, RenderGenerationIdentity,
    RenderGenerationInterval, RenderGenerationScope, RenderLifetimeUsage, RenderLimits,
    RenderPhase, RenderPhaseFilter, RenderProfile, RenderQuantity, RenderReadbackReport,
    RenderReadbackReportRef, RenderRefusal, RenderRefusalReason, RenderResource,
    RenderResourceFields, RenderResourceTable, RenderStandaloneReport, RenderUsage, RenderWork,
    RenderWorkFields, RenderWorkTable,
};
use surgeist_render::capability::{
    AvailableRuntimeCapabilities, BackgroundAttachmentCoordinatePolicy, BoxDecorationCapabilities,
    Capabilities, CompositingCapabilities, EffectPrecisionCapabilities, FilterCapabilities,
    GeometryTargetCapabilities, HitTestOwnership, ImageColorProfilePolicy, ImageOrientationPolicy,
    ImageSamplingCapabilities, MaskClipCapabilities, OffscreenPipelineCapabilities,
    PaintSourceCapabilities, RuntimeCapabilities, ShadowCapabilities, SurfaceCapabilities,
    SymbolicColorPolicy, TransformCoordinateSpaceCapabilities,
};
use surgeist_render::convolution::{
    ConvolutionInput, ConvolutionNumericPhase, ConvolutionParameter, ConvolutionPreparationError,
    PreparedConvolution, prepare_convolution,
};
use surgeist_render::diagnostic::{
    DegradedQuality, DegradedQualityKind, DeviceLossReason, GeometryRealizationError, GpuFaultKind,
    InvalidValue, PrimitiveFamily, PrimitiveOperation, RenderSurfaceAvailability,
    RuntimeCapabilityUnavailable, RuntimeCapabilityUnavailableReason, RuntimeOperation,
    SurfaceIdentityMismatchKind, UnresolvedResource, UnresolvedResourceKind, UnsupportedPrimitive,
};
use surgeist_render::displacement::{
    DisplacementChannel, DisplacementInput, DisplacementNumericPhase, DisplacementParameter,
    DisplacementPreparationError, DisplacementSource, PreparedDisplacement,
    PreparedDisplacementMap, prepare_displacement, prepare_native_displacement_map,
};
use surgeist_render::effect::{FilterColorSpace, FilterDomains, FilterEdgeMode};
use surgeist_render::gaussian::{
    GaussianInput, GaussianNumericPhase, GaussianParameter, GaussianPreparationError,
    GaussianSource, PreparedGaussian, prepare_gaussian,
};
use surgeist_render::geometry::{
    CoordinateSpaceId, CoordinateSpaceKind, CoordinateSpaceTag, PhysicalSize, Point, Radii, Rect,
    Size, Transform,
};
use surgeist_render::image::{
    Extend, Image, ImageBuffer, ImageFit, ImageId, ImageQuality, ImageRendering,
};
use surgeist_render::morphology::{
    MorphologyInput, MorphologyNumericPhase, MorphologyOperator, MorphologyParameter,
    MorphologyPreparationError, MorphologySource, PreparedMorphology, prepare_morphology,
};
use surgeist_render::native_shadow::{
    DropShadowInput, DropShadowNumericPhase, DropShadowPreparationError, DropShadowSource,
    PreparedDropShadow, prepare_drop_shadow,
};
use surgeist_render::offset::{
    OffsetInput, OffsetNumericPhase, OffsetParameter, OffsetPreparationError, PreparedOffset,
    prepare_offset,
};
use surgeist_render::paint::{
    Color, Gradient, GradientHint, GradientHintError, GradientRealizationError, GradientStop,
    NormalizedPaintLayer, Paint, PaintColor, PaintColorSpace, RepeatingGradient,
    prepare_sdr_srgb_color,
};
#[cfg(feature = "render-paint")]
use surgeist_render::paint_gradient::{
    PaintGradientError, prepare_linear_gradient, prepare_radial_gradient,
    prepare_repeating_linear_gradient, prepare_repeating_radial_gradient,
    prepare_zero_height_gradient_color,
};
use surgeist_render::renderer::{
    Antialiasing, EffectPrecision, EffectQualityPolicy, Options, RenderRoute, Renderer,
    ResourceCacheBudget, Stats,
};
use surgeist_render::scene::{
    BlendMode, Filter, Layer, LayerComposition, MaskLuminanceSpace, PorterDuffOperator,
    ResolvedLayerAlphaMask, ResolvedLayerLuminanceMask, ResolvedMaskLayer, ResolvedMaskSource,
    Scene, SceneSummary, Shadow, ShadowKind, ShadowList,
};
use surgeist_render::shape::{
    Dash, FillRule, FilledPath, LineCap, LineJoin, Path, PathElement, Shape, Stroke, StrokeAlign,
};
use surgeist_render::style::StyleColor;
use surgeist_render::style::background::{
    BackgroundBlendList, BackgroundBlendMode, BackgroundLayer, BackgroundNormalizationInput,
    BackgroundStack, NormalizedBackgroundCommand, NormalizedBackgroundCommandKind,
    NormalizedBackgroundLayer, NormalizedBackgroundLayerSource, NormalizedBackgroundStack,
};
use surgeist_render::style::clip::{
    ClipGeometry, ClipGeometryKind, ClipInput, ClipInputKind, NormalizedClip,
};
use surgeist_render::style::decoration::{
    BorderEdges, BorderSide, BorderStyle, BoxDecorationBreak, BoxDecorationFragment,
    BoxDecorationInput, BoxSide, NormalizedBorderCommand, NormalizedBorderStyle,
    NormalizedBoxDecoration, NormalizedBoxDecorationCommand, NormalizedBoxDecorationCommandKind,
    NormalizedBoxRadii, NormalizedDoubleBorderBands, NormalizedOutlineCommand,
    NormalizedOutlineStyle, Outline, OutlineStyle,
};
use surgeist_render::style::filter::{
    BackdropCaptureBounds, BackdropFilterInput, FilterAmount, FilterAngle, FilterBlur,
    FilterDropShadow, FilterList, FilterOp, FilterOpKind, FilteredImagePaint, UnitFilterAmount,
};
use surgeist_render::style::image::{
    BackgroundAreas, BackgroundAttachment, BackgroundBox, BackgroundClipGeometry,
    BackgroundClipGeometryKind, BackgroundPosition, BackgroundRepeat, BackgroundSize,
    BackgroundSizeKind, ImageAttachmentPlan, ImagePlacementInput, ImageRepeatMode, ImageRepeatPlan,
    ImageResourceDensity, PositionComponent, PositionComponentKind, PositionEdge,
    PositionEdgeOffset, RepeatMode, ResolvedImagePlacement, ResolvedImageRepeat,
    ResolvedImageResource, SizeComponent, SizeComponentKind, StyleImageLayer, StyleImageSource,
    StyleImageSourceKind, StyleResourceRef,
};
use surgeist_render::style::mask::{
    MaskCompositeMode, MaskInput, MaskLayer, MaskLayerStack, MaskMode, MaskSource, MaskSourceKind,
};
use surgeist_render::surface::{
    Attachment, Format, Parameters, PresentMode, Surface, SurfaceOptions, SurfaceResourceState,
    SurfaceState, WebCanvas,
};
use surgeist_render::text::{
    FontData, FontId, FontRef, GlyphCapability, GlyphNumericPhase, GlyphPreparationError,
    GlyphResource, GlyphRunLimits, GlyphSynthesis, NormalizedCoordinate, PreparedGlyphRun,
    TextDecorationLine, TextDecorationLineStyle, TextGlyph, TextInstance, TextPaint, TextRun,
    TextRunBounds, TextRunBoundsKind, TextShadowRun, VariationTable,
};
#[cfg(feature = "render-text")]
use surgeist_render::text_layout::{
    LeaderPatternCause, LeaderPatternError, PreparedGlyphDraw, PreparedLeaderGlyph,
    PreparedLeaderGroup, PreparedLeaderPattern, PreparedLeaderRun, PreparedTextScene,
    TextLayoutCause, TextLayoutError, TextLayoutLimits, TextLayoutLocation, TextLayoutResource,
    prepare_leader_pattern, prepare_text_layout,
};

// The modules themselves, including the six style submodules.
#[cfg(feature = "render-paint")]
use surgeist_render::paint_gradient;
use surgeist_render::style::image as style_image;
use surgeist_render::style::{background, clip, decoration, filter, mask};
#[cfg(feature = "render-text")]
use surgeist_render::text_layout;
use surgeist_render::{
    accounting, capability, convolution, diagnostic, displacement, effect, gaussian, geometry,
    image, morphology, native_shadow, offset, paint, renderer, scene, shape, style, surface, text,
};

#[test]
fn every_public_item_is_imported_from_its_module() {
    // The imports above are the assertion.
}

// Items: 315 imported through modules, 3 at the root.
