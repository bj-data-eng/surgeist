#![forbid(unsafe_code)]

//! GPU-only rendering boundary for Surgeist.
//!
//! The crate owns renderer-facing visual facts, safe WGPU resources, and two
//! observable execution routes. Effect-free scenes use one transaction-owned
//! [`RenderRoute::DirectVello`] raster pass. Resolved image alpha masks,
//! supported blend/composition, and bounded backdrop filter lists use
//! [`RenderRoute::GpuGraph`] image/composite passes. Production rendering has no
//! CPU pixel path or CPU fallback; CPU reference algorithms compile only for tests.
//!
//! Effect graphs use high precision by default. [`EffectQualityPolicy::RequireHighPrecision`]
//! rejects a device that cannot provide it, while
//! [`EffectQualityPolicy::AllowReducedPrecision`] permits reduced precision,
//! observably, only when high precision is unavailable. [`Capabilities::CURRENT`]
//! reports semantic capabilities for authored operations; [`Renderer::runtime_capabilities`]
//! reports immutable runtime capabilities for a selected device and surface.
//! Cargo features select host adapters rather than changing those meanings.
//!
//! Renderer and surface GPU operations are asynchronous and failure-atomic. A
//! failed or canceled operation publishes neither partial pixels nor partial
//! statistics, so [`Renderer::stats`] continues to describe the last successful
//! frame. CPU-visible pixels require explicit headless readback through
//! [`Renderer::read_headless`]; rendering never enters that path implicitly.
//!
//! The native window presentation path is host lifecycle owned.
//! The tracked `render_window_smoke` example exercises the public native window lifecycle
//! and presentation path. The
//! `wasm32-unknown-unknown` leaf boundary is compile-only under `render-web`;
//! real canvas execution requires a browser host and is root integration work.
//! Render owns its optional `render-text` consumer of authentic standalone Text
//! layouts. The root `surgeist` repository owns the facade, workspace wiring,
//! generated API artifacts and browser integration evidence.

mod accounting;
mod backend;
mod capability;
mod color_mapping;
mod color_reduction;
mod command;
mod convolution;
mod encode;
mod error;
mod filter;
mod filter_domains;
mod filter_model;
mod frame;
mod gaussian;
mod geometry;
mod gpu_transaction;
mod image;
mod layer;
mod native_shadow;
mod offset;
mod paint;
#[cfg(feature = "render-paint")]
mod paint_gradient;
mod pass;
mod readback;
#[cfg(test)]
mod reference;
mod renderer;
mod resource;
mod scene;
mod shader;
mod shape;
mod stats;
mod style;
mod surface;
mod text;
#[cfg(feature = "render-text")]
mod text_layout;
mod texture;
mod validation;
mod vello_engine;

pub use accounting::{
    RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
    RenderAccountingSetupError, RenderAttemptKind, RenderAttemptReport, RenderAttemptStatus,
    RenderChangeKind, RenderClock, RenderCpuEvent, RenderCpuIdentity, RenderCpuInterval,
    RenderCpuScope, RenderCpuSnapshot, RenderCpuUsage, RenderDeviceSnapshot, RenderEvent,
    RenderFrameReport, RenderFrameReportRef, RenderGenerationEvent, RenderGenerationIdentity,
    RenderGenerationInterval, RenderGenerationScope, RenderLifetimeUsage, RenderLimits,
    RenderPhase, RenderPhaseFilter, RenderProfile, RenderQuantity, RenderReadbackReport,
    RenderReadbackReportRef, RenderRefusal, RenderRefusalReason, RenderResource,
    RenderResourceTable, RenderStandaloneReport, RenderUsage, RenderWork, RenderWorkTable,
};
pub(crate) use error::BackendErrorCode;

pub use capability::{
    AvailableRuntimeCapabilities, BackgroundAttachmentCoordinatePolicy, BoxDecorationCapabilities,
    Capabilities, CompositingCapabilities, EffectPrecisionCapabilities, FilterCapabilities,
    GeometryTargetCapabilities, HitTestOwnership, ImageColorProfilePolicy, ImageOrientationPolicy,
    ImageSamplingCapabilities, MaskClipCapabilities, OffscreenPipelineCapabilities,
    PaintSourceCapabilities, RuntimeCapabilities, ShadowCapabilities, SurfaceCapabilities,
    SymbolicColorPolicy, TransformCoordinateSpaceCapabilities,
};
pub use color_mapping::prepare_sdr_srgb_color;
pub use convolution::{
    ConvolutionInput, ConvolutionNumericPhase, ConvolutionParameter, ConvolutionPreparationError,
    PreparedConvolution, prepare_convolution,
};
pub use error::{
    DegradedQuality, DegradedQualityKind, DeviceLossReason, Error, ErrorCode, GpuFaultKind,
    InvalidValue, PrimitiveFamily, PrimitiveOperation, RenderSurfaceAvailability, Result,
    RuntimeCapabilityUnavailable, RuntimeCapabilityUnavailableReason, RuntimeOperation,
    SurfaceIdentityMismatchKind, UnresolvedResource, UnresolvedResourceKind, UnsupportedPrimitive,
};
pub use error::{GeometryRealizationError, GradientHintError, GradientRealizationError};
pub use filter_domains::FilterDomains;
pub use filter_model::{FilterColorSpace, FilterEdgeMode};
pub use gaussian::{
    GaussianInput, GaussianNumericPhase, GaussianParameter, GaussianPreparationError,
    GaussianSource, PreparedGaussian, prepare_gaussian,
};
pub use geometry::{
    CoordinateSpaceId, CoordinateSpaceKind, CoordinateSpaceTag, PhysicalSize, Point, Radii, Rect,
    Size, Transform,
};
pub use image::{Extend, Image, ImageBuffer, ImageFit, ImageId, ImageQuality, ImageRendering};
pub use layer::{
    BlendMode, Filter, Layer, LayerComposition, MaskLuminanceSpace, PorterDuffOperator,
    ResolvedLayerAlphaMask, ResolvedLayerLuminanceMask, ResolvedMaskLayer, ResolvedMaskSource,
    Shadow, ShadowKind, ShadowList,
};
pub use native_shadow::{
    DropShadowInput, DropShadowNumericPhase, DropShadowPreparationError, DropShadowSource,
    PreparedDropShadow, prepare_drop_shadow,
};
pub use offset::{
    OffsetInput, OffsetNumericPhase, OffsetParameter, OffsetPreparationError, PreparedOffset,
    prepare_offset,
};
pub use paint::{
    Color, Gradient, GradientHint, GradientStop, NormalizedPaintLayer, Paint, PaintColor,
    PaintColorSpace, RepeatingGradient,
};
#[cfg(feature = "render-paint")]
pub use paint_gradient::{
    PaintGradientError, prepare_linear_gradient, prepare_radial_gradient,
    prepare_repeating_linear_gradient, prepare_repeating_radial_gradient,
    prepare_zero_height_gradient_color,
};
pub use renderer::{Antialiasing, EffectQualityPolicy, Options, Renderer, ResourceCacheBudget};
pub use scene::{Scene, SceneSummary};
pub use shape::{
    Dash, FillRule, FilledPath, LineCap, LineJoin, Path, PathElement, Shape, Stroke, StrokeAlign,
};
pub use stats::{EffectPrecision, RenderRoute, Stats};
pub use style::{
    BackdropCaptureBounds, BackdropFilterInput, BackgroundAreas, BackgroundAttachment,
    BackgroundBlendList, BackgroundBlendMode, BackgroundBox, BackgroundClipGeometry,
    BackgroundClipGeometryKind, BackgroundLayer, BackgroundNormalizationInput, BackgroundPosition,
    BackgroundRepeat, BackgroundSize, BackgroundSizeKind, BackgroundStack, BorderEdges, BorderSide,
    BorderStyle, BoxDecorationBreak, BoxDecorationFragment, BoxDecorationInput, BoxSide,
    ClipGeometry, ClipGeometryKind, ClipInput, ClipInputKind, FilterAmount, FilterAngle,
    FilterBlur, FilterDropShadow, FilterList, FilterOp, FilterOpKind, FilteredImagePaint,
    ImageAttachmentPlan, ImagePlacementInput, ImageRepeatMode, ImageRepeatPlan,
    ImageResourceDensity, MaskCompositeMode, MaskInput, MaskLayer, MaskLayerStack, MaskMode,
    MaskSource, MaskSourceKind, NormalizedBackgroundCommand, NormalizedBackgroundCommandKind,
    NormalizedBackgroundLayer, NormalizedBackgroundLayerSource, NormalizedBackgroundStack,
    NormalizedBorderCommand, NormalizedBorderStyle, NormalizedBoxDecoration,
    NormalizedBoxDecorationCommand, NormalizedBoxDecorationCommandKind, NormalizedBoxRadii,
    NormalizedClip, NormalizedDoubleBorderBands, NormalizedOutlineCommand, NormalizedOutlineStyle,
    Outline, OutlineStyle, PositionComponent, PositionComponentKind, PositionEdge,
    PositionEdgeOffset, RepeatMode, ResolvedImagePlacement, ResolvedImageRepeat,
    ResolvedImageResource, SizeComponent, SizeComponentKind, StyleColor, StyleImageLayer,
    StyleImageSource, StyleImageSourceKind, StyleResourceRef, UnitFilterAmount,
};
pub use surface::{
    Attachment, Format, Parameters, PresentMode, Surface, SurfaceOptions, SurfaceResourceState,
    SurfaceState, WebCanvas,
};
#[cfg(feature = "render-text")]
pub use text_layout::{
    LeaderPatternCause, LeaderPatternError, PreparedGlyphDraw, PreparedLeaderGlyph,
    PreparedLeaderGroup, PreparedLeaderPattern, PreparedLeaderRun, PreparedTextScene,
    TextLayoutCause, TextLayoutError, TextLayoutLimits, TextLayoutLocation, TextLayoutResource,
    prepare_leader_pattern, prepare_text_layout,
};

pub use text::{
    FontData, FontId, FontRef, GlyphCapability, GlyphNumericPhase, GlyphPreparationError,
    GlyphResource, GlyphRunLimits, GlyphSynthesis, NormalizedCoordinate, PreparedGlyphRun,
    TextDecorationLine, TextDecorationLineStyle, TextGlyph, TextInstance, TextPaint, TextRun,
    TextRunBounds, TextRunBoundsKind, TextShadowRun, VariationTable,
};

#[cfg(test)]
mod tests;
