use std::{error, fmt, sync::OnceLock};

use crate::{EffectQualityPolicy, Format, PhysicalSize};

/// Result type returned by fallible rendering contracts.
pub type Result<T> = std::result::Result<T, Error>;

/// Stable render diagnostic with optional typed semantic or runtime context.
///
/// Callers should branch on [`ErrorCode`] and the matching typed payload rather
/// than parse [`Self::message`]. Frame-operation errors are failure-atomic: the
/// attempted frame is not published, and any previous complete frame remains
/// observable. Creating an accounting diagnostic requires no allocation;
/// requesting its message formats and caches that text at the observation edge.
#[derive(Debug)]
pub struct Error {
    payload: ErrorPayload,
}

#[derive(Debug)]
enum ErrorPayload {
    Accounting(AccountingFailure),
    Context(Box<ErrorContext>),
}

/// Fixed accounting data is available during cleanup without formatting or
/// allocating an error context. Only explicit message observation fills the cache.
#[derive(Debug)]
struct AccountingFailure {
    diagnostic: crate::RenderAccountingError,
    message: OnceLock<String>,
}

#[derive(Debug)]
struct ErrorContext {
    code: ErrorCode,
    message: String,
    source: Option<BackendErrorSource>,
    invalid_value: Option<InvalidValue>,
    unsupported_primitive: Option<UnsupportedPrimitive>,
    unresolved_resource: Option<UnresolvedResource>,
    degraded_quality: Option<DegradedQuality>,
    runtime_capability_unavailable: Option<RuntimeCapabilityUnavailable>,
    preparation: Option<PreparationDiagnostic>,
    #[cfg(feature = "render-text")]
    text_layout: Option<crate::TextLayoutError>,
    #[cfg(feature = "render-text")]
    leader_pattern: Option<crate::LeaderPatternError>,
}

#[derive(Debug)]
enum PreparationDiagnostic {
    Convolution(crate::ConvolutionPreparationError),
    Offset(crate::OffsetPreparationError),
    Gaussian(crate::GaussianPreparationError),
    DropShadow(crate::DropShadowPreparationError),
    Glyph(crate::GlyphPreparationError),
    Accounting(crate::RenderAccountingError),
    GeometryRealization(GeometryRealizationError),
    GradientRealization(GradientRealizationError),
    GradientHint(GradientHintError),
    #[cfg(feature = "render-paint")]
    PaintGradient(crate::PaintGradientError),
}

#[cfg(not(target_arch = "wasm32"))]
type BackendErrorSource = Box<dyn error::Error + Send + Sync + 'static>;

#[cfg(target_arch = "wasm32")]
type BackendErrorSource = Box<dyn error::Error + 'static>;

/// Private code domain accepted by backend error construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BackendErrorCode {
    RendererCreateFailed,
    SurfaceCreateFailed,
    #[cfg(any(
        test,
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    SurfaceConfigureFailed,
    SurfaceOutOfMemory,
    #[cfg(any(
        test,
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    SurfaceTimeout,
    #[cfg(any(
        test,
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    SurfaceOutdated,
    ImageUploadFailed,
    RenderFailed,
    /// A texture readback failed during copy, mapping, or row decoding.
    ReadbackFailed,
    #[cfg(any(
        test,
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    PresentFailed,
    UnsupportedBackend,
}

impl BackendErrorCode {
    const fn error_code(self) -> ErrorCode {
        match self {
            Self::RendererCreateFailed => ErrorCode::RendererCreateFailed,
            Self::SurfaceCreateFailed => ErrorCode::SurfaceCreateFailed,
            #[cfg(any(
                test,
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            Self::SurfaceConfigureFailed => ErrorCode::SurfaceConfigureFailed,
            Self::SurfaceOutOfMemory => ErrorCode::SurfaceOutOfMemory,
            #[cfg(any(
                test,
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            Self::SurfaceTimeout => ErrorCode::SurfaceTimeout,
            #[cfg(any(
                test,
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            Self::SurfaceOutdated => ErrorCode::SurfaceOutdated,
            Self::ImageUploadFailed => ErrorCode::ImageUploadFailed,
            Self::RenderFailed => ErrorCode::RenderFailed,
            Self::ReadbackFailed => ErrorCode::ReadbackFailed,
            #[cfg(any(
                test,
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            Self::PresentFailed => ErrorCode::PresentFailed,
            Self::UnsupportedBackend => ErrorCode::UnsupportedBackend,
        }
    }
}

impl ErrorPayload {
    /// Explicit prose/source enrichment is an allocating observation path.
    /// Mandatory accounting finalization uses the fixed constructor and typed data.
    fn into_context(self) -> Box<ErrorContext> {
        match self {
            Self::Context(context) => context,
            Self::Accounting(failure) => {
                let diagnostic = failure.diagnostic;
                let message = failure
                    .message
                    .into_inner()
                    .unwrap_or_else(|| format!("render accounting: {diagnostic}"));
                Box::new(ErrorContext {
                    code: ErrorCode::Accounting,
                    message,
                    source: None,
                    invalid_value: None,
                    unsupported_primitive: None,
                    unresolved_resource: None,
                    degraded_quality: None,
                    runtime_capability_unavailable: None,
                    preparation: Some(PreparationDiagnostic::Accounting(diagnostic)),
                    #[cfg(feature = "render-text")]
                    text_layout: None,
                    #[cfg(feature = "render-text")]
                    leader_pattern: None,
                })
            }
        }
    }
}

impl Error {
    fn context(context: ErrorContext) -> Self {
        Self {
            payload: ErrorPayload::Context(Box::new(context)),
        }
    }

    /// This constructor and the public From conversion retain only fixed data.
    pub(crate) const fn accounting(diagnostic: crate::RenderAccountingError) -> Self {
        Self {
            payload: ErrorPayload::Accounting(AccountingFailure {
                diagnostic,
                message: OnceLock::new(),
            }),
        }
    }

    /// General accounting setup/admission diagnostic, when present.
    pub fn accounting_diagnostic(&self) -> Option<crate::RenderAccountingError> {
        match &self.payload {
            ErrorPayload::Accounting(failure) => Some(failure.diagnostic),
            ErrorPayload::Context(context) => match context.preparation {
                Some(PreparationDiagnostic::Accounting(diagnostic)) => Some(diagnostic),
                _ => None,
            },
        }
    }

    /// Reports a standalone Text-layout admission refusal with preserved item causes.
    #[cfg(feature = "render-text")]
    #[must_use]
    pub fn text_layout_diagnostic(&self) -> Option<&crate::TextLayoutError> {
        match &self.payload {
            ErrorPayload::Accounting(_) => None,
            ErrorPayload::Context(context) => context.text_layout.as_ref(),
        }
    }

    /// Preserved failure at an actual finalized leader run.
    #[cfg(feature = "render-text")]
    #[must_use]
    pub fn leader_pattern_diagnostic(&self) -> Option<&crate::LeaderPatternError> {
        match &self.payload {
            ErrorPayload::Accounting(_) => None,
            ErrorPayload::Context(context) => context.leader_pattern.as_ref(),
        }
    }

    #[must_use]
    pub(crate) fn new(code: BackendErrorCode, message: impl Into<String>) -> Self {
        Self::context(ErrorContext {
            code: code.error_code(),
            message: message.into(),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: None,
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub(crate) fn with_source(self, source: impl error::Error + Send + Sync + 'static) -> Self {
        let mut context = self.payload.into_context();
        context.source = Some(Box::new(source));
        Self {
            payload: ErrorPayload::Context(context),
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[must_use]
    pub(crate) fn with_source(self, source: impl error::Error + 'static) -> Self {
        let mut context = self.payload.into_context();
        context.source = Some(Box::new(source));
        Self {
            payload: ErrorPayload::Context(context),
        }
    }

    #[must_use]
    pub(crate) fn append_message(self, suffix: impl fmt::Display) -> Self {
        let mut context = self.payload.into_context();
        context.message.push_str(&suffix.to_string());
        Self {
            payload: ErrorPayload::Context(context),
        }
    }

    #[must_use]
    pub(crate) fn replace_message(self, message: impl Into<String>) -> Self {
        let mut context = self.payload.into_context();
        context.message = message.into();
        Self {
            payload: ErrorPayload::Context(context),
        }
    }

    pub(crate) fn invalid_input_message(message: impl Into<String>) -> Self {
        Self::from_invalid_value(InvalidValue::new(
            "input",
            "internal validation",
            "must satisfy the requested validation rule",
        ))
        .replace_message(message)
    }

    pub(crate) fn runtime_unavailable(
        diagnostic: RuntimeCapabilityUnavailable,
        message: impl Into<String>,
    ) -> Self {
        Self::runtime_capability_unavailable(diagnostic).replace_message(message)
    }

    /// Returns this diagnostic's stable classification.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match &self.payload {
            ErrorPayload::Accounting(_) => ErrorCode::Accounting,
            ErrorPayload::Context(context) => context.code,
        }
    }

    /// Returns the human-readable diagnostic message.
    ///
    /// Accounting diagnostics format and cache this text on first request. Their
    /// construction, typed access and Display formatting do not initialize it.
    #[must_use]
    pub fn message(&self) -> &str {
        match &self.payload {
            ErrorPayload::Accounting(failure) => failure
                .message
                .get_or_init(|| format!("render accounting: {}", failure.diagnostic)),
            ErrorPayload::Context(context) => &context.message,
        }
    }

    /// Creates an invalid-input diagnostic with its structured payload.
    #[must_use]
    pub fn invalid_value(
        name: impl Into<String>,
        value: impl std::fmt::Display,
        rule: &'static str,
    ) -> Self {
        Self::from_invalid_value(InvalidValue::new(name, value, rule))
    }

    /// Creates an invalid-input diagnostic whose structured payload is available through
    /// [`Self::invalid_value_diagnostic`].
    #[must_use]
    pub fn from_invalid_value(invalid_value: InvalidValue) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::InvalidInput,
            message: invalid_value.message(),
            source: None,
            invalid_value: Some(invalid_value),
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: None,
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }

    /// Creates an unsupported-primitive diagnostic whose payload is returned by
    /// [`Self::unsupported_primitive`].
    #[must_use]
    pub fn unsupported_render_primitive(primitive: UnsupportedPrimitive) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::UnsupportedPrimitive,
            message: format!(
                "render primitive is unsupported: {} / {}",
                primitive.family().label(),
                primitive.label()
            ),
            source: None,
            invalid_value: None,
            unsupported_primitive: Some(primitive),
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: None,
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }

    /// Creates an unresolved-resource diagnostic whose structured payload is available through
    /// [`Self::unresolved_resource_diagnostic`].
    #[must_use]
    pub fn unresolved_resource(resource: UnresolvedResource) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::UnresolvedResource,
            message: resource.message(),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: Some(resource),
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: None,
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }

    /// Creates a degraded-quality diagnostic whose structured payload is available through
    /// [`Self::degraded_quality_diagnostic`].
    #[must_use]
    pub fn degraded_quality(diagnostic: DegradedQuality) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::DegradedQuality,
            message: diagnostic.message(),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: Some(diagnostic),
            runtime_capability_unavailable: None,
            preparation: None,
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }

    /// Creates a runtime-capability diagnostic whose payload is available through
    /// [`Self::runtime_capability_unavailable_diagnostic`].
    #[must_use]
    pub fn runtime_capability_unavailable(value: RuntimeCapabilityUnavailable) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::RuntimeCapabilityUnavailable,
            message: "runtime capability is unavailable".into(),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: Some(value),
            preparation: None,
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }

    /// Returns a semantic headless glyph-preparation refusal, when present.
    #[must_use]
    pub const fn glyph_preparation_diagnostic(&self) -> Option<&crate::GlyphPreparationError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::Glyph(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns the typed native drop-shadow diagnostic, when present.
    #[must_use]
    pub const fn drop_shadow_diagnostic(&self) -> Option<&crate::DropShadowPreparationError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::DropShadow(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns the typed native Gaussian diagnostic, if present.
    #[must_use]
    pub const fn gaussian_diagnostic(&self) -> Option<&crate::GaussianPreparationError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::Gaussian(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns a typed resolved convolution refusal, when applicable.
    #[must_use]
    pub const fn convolution_diagnostic(&self) -> Option<&crate::ConvolutionPreparationError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::Convolution(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns a typed native offset preparation or frame qualification refusal.
    #[must_use]
    pub const fn offset_diagnostic(&self) -> Option<&crate::OffsetPreparationError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::Offset(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns an invalid outgoing-hint association or unsupported sweep hint.
    #[must_use]
    pub const fn gradient_hint_diagnostic(&self) -> Option<&GradientHintError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::GradientHint(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns a contextual native-geometry precision or representation refusal.
    #[must_use]
    pub const fn geometry_realization_diagnostic(&self) -> Option<&GeometryRealizationError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::GeometryRealization(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns the contextual gradient realization refusal, when present.
    #[must_use]
    pub const fn gradient_realization_diagnostic(&self) -> Option<&GradientRealizationError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::GradientRealization(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns a native Paint-gradient consumer diagnostic, when present.
    #[cfg(feature = "render-paint")]
    #[must_use]
    pub const fn paint_gradient_diagnostic(&self) -> Option<&crate::PaintGradientError> {
        match &self.payload {
            ErrorPayload::Context(context) => match &context.preparation {
                Some(PreparationDiagnostic::PaintGradient(error)) => Some(error),
                _ => None,
            },
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns the unsupported-primitive payload, when this diagnostic carries one.
    #[must_use]
    pub const fn unsupported_primitive(&self) -> Option<UnsupportedPrimitive> {
        match &self.payload {
            ErrorPayload::Context(context) => context.unsupported_primitive,
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns the invalid-input payload, when this diagnostic carries one.
    #[must_use]
    pub const fn invalid_value_diagnostic(&self) -> Option<&InvalidValue> {
        match &self.payload {
            ErrorPayload::Context(context) => context.invalid_value.as_ref(),
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns the unresolved-resource payload, when this diagnostic carries one.
    #[must_use]
    pub const fn unresolved_resource_diagnostic(&self) -> Option<&UnresolvedResource> {
        match &self.payload {
            ErrorPayload::Context(context) => context.unresolved_resource.as_ref(),
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns the degraded-quality payload, when this diagnostic carries one.
    #[must_use]
    pub const fn degraded_quality_diagnostic(&self) -> Option<&DegradedQuality> {
        match &self.payload {
            ErrorPayload::Context(context) => context.degraded_quality.as_ref(),
            ErrorPayload::Accounting(_) => None,
        }
    }

    /// Returns the runtime-capability payload, when this diagnostic carries one.
    #[must_use]
    pub const fn runtime_capability_unavailable_diagnostic(
        &self,
    ) -> Option<&RuntimeCapabilityUnavailable> {
        match &self.payload {
            ErrorPayload::Context(context) => context.runtime_capability_unavailable.as_ref(),
            ErrorPayload::Accounting(_) => None,
        }
    }
}

impl From<crate::RenderAccountingError> for Error {
    fn from(value: crate::RenderAccountingError) -> Self {
        Self::accounting(value)
    }
}

impl From<crate::GlyphPreparationError> for Error {
    fn from(value: crate::GlyphPreparationError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::GlyphPreparation,
            message: format!("glyph preparation refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::Glyph(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

impl From<crate::ConvolutionPreparationError> for Error {
    fn from(value: crate::ConvolutionPreparationError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::InvalidInput,
            message: format!("convolution preparation refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::Convolution(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

impl From<crate::DropShadowPreparationError> for Error {
    fn from(value: crate::DropShadowPreparationError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::InvalidInput,
            message: format!("native drop-shadow preparation refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::DropShadow(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

impl From<crate::GaussianPreparationError> for Error {
    fn from(value: crate::GaussianPreparationError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::InvalidInput,
            message: format!("Gaussian preparation refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::Gaussian(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

impl From<crate::OffsetPreparationError> for Error {
    fn from(value: crate::OffsetPreparationError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::InvalidInput,
            message: format!("offset preparation refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::Offset(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

impl From<GradientHintError> for Error {
    fn from(value: GradientHintError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::GradientHint,
            message: format!("gradient hint refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::GradientHint(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

/// An outgoing hint does not identify a supported interval of ordered colors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientHintError {
    /// The final color stop cannot carry an outgoing interval hint.
    TrailingHint { stop_index: usize },
    /// The hint lies outside the inclusive interval of its adjacent colors.
    HintOutsideInterval { stop_index: usize },
    /// Hinted full-turn sweep interpolation is not supported.
    UnsupportedSweep { stop_index: usize },
}

impl From<GeometryRealizationError> for Error {
    fn from(value: GeometryRealizationError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::GeometryRealization,
            message: format!("geometry realization refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::GeometryRealization(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

/// Valid geometry cannot qualify for its actual native representation.
///
/// These contextual refusals preserve intrinsic input validity and semantic
/// support. Original accounting, storage, font, device, and gradient errors
/// retain their existing diagnostic domains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeometryRealizationError {
    /// Required native transform coefficients cannot be represented.
    UnrepresentableTransform,
    /// A required native path conversion or segment-retention condition fails.
    UnrepresentablePath {
        /// Local index in the receiving operation's input element sequence.
        element_index: usize,
    },
    /// A required native stroke parameter cannot be represented.
    UnrepresentableStroke,
    /// A required native glyph placement cannot be represented.
    UnrepresentableGlyph {
        /// Local index in the receiving operation's input glyph sequence.
        glyph_index: usize,
    },
    /// A required native blur parameter cannot be represented.
    UnrepresentableBlur,
    /// Required rank, composition, or subnormal-flushing behavior cannot qualify.
    InsufficientNativeCapacity,
}

impl From<GradientRealizationError> for Error {
    fn from(value: GradientRealizationError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::GradientRealization,
            message: format!("gradient realization refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::GradientRealization(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

/// A valid gradient cannot be represented in the current native context.
///
/// These refusals describe field realization, not invalid authored CSS. Named
/// accounting and resource limits continue to use [`crate::RenderAccountingError`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientRealizationError {
    /// A required linear comparison or interpolation field has no direction.
    DegenerateLinearField,
    /// Required range or transformed geometry cannot be represented.
    UnrepresentableGeometry,
    /// A required native stop loses its source ordering or interval separation.
    UnrepresentableStop {
        /// Index in the owned gradient's supplied stop list.
        stop_index: usize,
    },
    /// A required native hint control or exponent cannot be represented while
    /// preserving its source equality and strict containment.
    UnrepresentableHint {
        /// Index of the outgoing interval in the owned stop list.
        stop_index: usize,
    },
    /// Native field arithmetic cannot meet the selected accuracy guarantee.
    InsufficientNativeCapacity,
}

#[cfg(feature = "render-paint")]
impl From<crate::PaintGradientError> for Error {
    fn from(value: crate::PaintGradientError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::PaintGradient,
            message: format!("Paint gradient preparation refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: Some(PreparationDiagnostic::PaintGradient(value)),
            #[cfg(feature = "render-text")]
            text_layout: None,
            #[cfg(feature = "render-text")]
            leader_pattern: None,
        })
    }
}

#[cfg(feature = "render-text")]
impl From<crate::TextLayoutError> for Error {
    fn from(value: crate::TextLayoutError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::TextLayout,
            message: format!("Text layout admission refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: None,
            text_layout: Some(value),
            leader_pattern: None,
        })
    }
}

#[cfg(feature = "render-text")]
impl From<crate::LeaderPatternError> for Error {
    fn from(value: crate::LeaderPatternError) -> Self {
        Self::context(ErrorContext {
            code: ErrorCode::LeaderPattern,
            message: format!("leader pattern admission refused: {value:?}"),
            source: None,
            invalid_value: None,
            unsupported_primitive: None,
            unresolved_resource: None,
            degraded_quality: None,
            runtime_capability_unavailable: None,
            preparation: None,
            text_layout: None,
            leader_pattern: Some(value),
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.payload {
            ErrorPayload::Accounting(failure) => {
                write!(f, "Accounting: render accounting: {}", failure.diagnostic)
            }
            ErrorPayload::Context(context) => {
                write!(f, "{:?}: {}", context.code, context.message)
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.payload {
            ErrorPayload::Accounting(_) => None,
            ErrorPayload::Context(context) => context.source.as_deref().map(|error| error as _),
        }
    }
}

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Stable classifications for render diagnostics, suitable for programmatic failure handling.
///
/// Callers first inspect [`Error::code`], then use the matching typed payload
/// accessor for invalid input, glyph preparation, unsupported primitives,
/// unresolved resources, degraded quality, or unavailable runtime capabilities. Other codes describe
/// backend-operation failures and may expose a safe source through
/// [`std::error::Error::source`].
pub enum ErrorCode {
    /// General accounting setup or named admission failure.
    Accounting,
    /// Reports contextual native-geometry representation or precision refusal.
    GeometryRealization,
    /// Reports an invalid or unsupported outgoing gradient hint association.
    GradientHint,
    /// Reports contextual gradient representation or precision refusal.
    GradientRealization,
    /// Reports atomic native Paint-gradient preparation failure.
    #[cfg(feature = "render-paint")]
    PaintGradient,
    /// Reports atomic standalone Text-layout admission failure.
    #[cfg(feature = "render-text")]
    TextLayout,
    /// Reports atomic finalized leader-pattern admission failure.
    #[cfg(feature = "render-text")]
    LeaderPattern,
    /// Reports a semantic headless glyph-preparation refusal.
    GlyphPreparation,
    /// Reports failure to create a GPU device.
    DeviceCreateFailed,
    /// Reports failure to create renderer-owned GPU state.
    RendererCreateFailed,
    /// Reports failure to create a render surface.
    SurfaceCreateFailed,
    /// Reports failure to configure a render surface.
    SurfaceConfigureFailed,
    /// Reports GPU memory exhaustion during a surface operation.
    SurfaceOutOfMemory,
    /// Reports a surface operation that timed out.
    SurfaceTimeout,
    /// Reports a surface configuration that became outdated before use.
    SurfaceOutdated,
    /// Reports caller input that violates an intrinsic validation rule.
    ///
    /// When constructed from a typed value, details are available through
    /// [`Error::invalid_value_diagnostic`].
    InvalidInput,
    /// Reports a render primitive that this renderer cannot represent.
    UnsupportedPrimitive,
    /// Reports a resource reference that could not be resolved.
    ///
    /// Details are available through [`Error::unresolved_resource_diagnostic`].
    UnresolvedResource,
    /// Reports a requested rendering quality that could not be preserved.
    ///
    /// Details are available through [`Error::degraded_quality_diagnostic`].
    DegradedQuality,
    /// Reports that a runtime GPU capability prevented a specific render operation.
    RuntimeCapabilityUnavailable,
    /// Reports failure while uploading image data to the GPU.
    ImageUploadFailed,
    /// Reports failure while validating, encoding, submitting, or executing a render.
    RenderFailed,
    /// Reports failure to copy, map, or decode pixels during GPU readback.
    ReadbackFailed,
    /// Reports failure to present a completed surface frame.
    PresentFailed,
    /// Reports that the selected platform or backend path is unsupported.
    UnsupportedBackend,
}

/// Validated runtime-phase evidence that an operation cannot use a selected GPU capability.
///
/// Each value couples one [`RuntimeOperation`] with a reason permitted for that
/// operation. Callers receive the validated pair through [`Error`]; backend
/// failures cannot manufacture an unrelated operation/reason combination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeCapabilityUnavailable {
    operation: RuntimeOperation,
    reason: RuntimeCapabilityUnavailableReason,
}

impl RuntimeCapabilityUnavailable {
    pub(crate) const fn device_lost(operation: RuntimeOperation, reason: DeviceLossReason) -> Self {
        Self {
            operation,
            reason: RuntimeCapabilityUnavailableReason::DeviceLost { reason },
        }
    }

    pub(crate) const fn device_faulted(operation: RuntimeOperation, kind: GpuFaultKind) -> Self {
        Self {
            operation,
            reason: RuntimeCapabilityUnavailableReason::DeviceFaulted { kind },
        }
    }

    pub(crate) const fn adapter_selection_unavailable() -> Self {
        Self {
            operation: RuntimeOperation::AdapterSelection,
            reason: RuntimeCapabilityUnavailableReason::AdapterUnavailable,
        }
    }

    pub(crate) const fn rendering_adapter_unavailable() -> Self {
        Self {
            operation: RuntimeOperation::SurfaceRendering,
            reason: RuntimeCapabilityUnavailableReason::AdapterUnavailable,
        }
    }

    pub(crate) const fn readback_adapter_unavailable() -> Self {
        Self {
            operation: RuntimeOperation::SurfaceReadback,
            reason: RuntimeCapabilityUnavailableReason::AdapterUnavailable,
        }
    }

    pub(crate) const fn effect_format_unavailable(policy: EffectQualityPolicy) -> Self {
        Self {
            operation: RuntimeOperation::EffectRendering,
            reason: RuntimeCapabilityUnavailableReason::EffectFormatUnavailable { policy },
        }
    }

    pub(crate) const fn filter_float32_unavailable(operation: RuntimeOperation) -> Self {
        Self {
            operation,
            reason: RuntimeCapabilityUnavailableReason::FilterFloat32Unavailable,
        }
    }

    pub(crate) const fn texture_dimension_exceeded(requested: PhysicalSize, maximum: u32) -> Self {
        Self {
            operation: RuntimeOperation::EffectTextureAllocation,
            reason: RuntimeCapabilityUnavailableReason::TextureDimensionExceeded {
                requested,
                maximum,
            },
        }
    }

    pub(crate) const fn surface_suspended(operation: SurfaceIoOperation) -> Self {
        Self {
            operation: operation.runtime_operation(),
            reason: RuntimeCapabilityUnavailableReason::SurfaceUnavailable {
                state: RenderSurfaceAvailability::Suspended,
            },
        }
    }

    pub(crate) const fn rendering_nonrenderable() -> Self {
        Self {
            operation: RuntimeOperation::SurfaceRendering,
            reason: RuntimeCapabilityUnavailableReason::SurfaceUnavailable {
                state: RenderSurfaceAvailability::NonRenderable,
            },
        }
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    pub(crate) const fn rendering_occluded() -> Self {
        Self {
            operation: RuntimeOperation::SurfaceRendering,
            reason: RuntimeCapabilityUnavailableReason::SurfaceUnavailable {
                state: RenderSurfaceAvailability::Occluded,
            },
        }
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    pub(crate) const fn rendering_lost() -> Self {
        Self {
            operation: RuntimeOperation::SurfaceRendering,
            reason: RuntimeCapabilityUnavailableReason::SurfaceUnavailable {
                state: RenderSurfaceAvailability::Lost,
            },
        }
    }

    pub(crate) const fn readback_uninitialized() -> Self {
        Self {
            operation: RuntimeOperation::SurfaceReadback,
            reason: RuntimeCapabilityUnavailableReason::SurfaceUnavailable {
                state: RenderSurfaceAvailability::Uninitialized,
            },
        }
    }

    pub(crate) const fn surface_identity_mismatch(
        operation: SurfaceDiagnosticOperation,
        kind: SurfaceIdentityMismatchKind,
    ) -> Self {
        Self {
            operation: operation.runtime_operation(),
            reason: RuntimeCapabilityUnavailableReason::SurfaceIdentityMismatch { kind },
        }
    }

    /// Returns the operation that could not use the selected runtime capability.
    #[must_use]
    pub const fn operation(self) -> RuntimeOperation {
        self.operation
    }

    /// Returns the validated runtime reason that prevented the operation.
    #[must_use]
    pub const fn reason(self) -> RuntimeCapabilityUnavailableReason {
        self.reason
    }
}

/// Operations whose surface identity is validated before backend work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SurfaceDiagnosticOperation {
    Rendering,
    Readback,
    Resume,
}

impl SurfaceDiagnosticOperation {
    pub(crate) const fn runtime_operation(self) -> RuntimeOperation {
        match self {
            Self::Rendering => RuntimeOperation::SurfaceRendering,
            Self::Readback => RuntimeOperation::SurfaceReadback,
            Self::Resume => RuntimeOperation::SurfaceResume,
        }
    }
}

/// Surface IO that can be refused while suspended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SurfaceIoOperation {
    Rendering,
    Readback,
}

impl SurfaceIoOperation {
    pub(crate) const fn runtime_operation(self) -> RuntimeOperation {
        match self {
            Self::Rendering => RuntimeOperation::SurfaceRendering,
            Self::Readback => RuntimeOperation::SurfaceReadback,
        }
    }
}

/// Runtime operation for which GPU capability availability is diagnosed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RuntimeOperation {
    /// Selecting an adapter for a presented surface.
    AdapterSelection,
    /// Rendering into a surface.
    SurfaceRendering,
    /// Reading pixels from a surface.
    SurfaceReadback,
    /// Resuming a suspended surface.
    SurfaceResume,
    /// Rendering an effect graph.
    EffectRendering,
    /// Executing signed float32 convolution accumulation.
    ConvolutionRendering,
    /// Executing native Gaussian accumulation.
    GaussianRendering,
    /// Executing native alpha-blur, displacement, flood and original-source merge.
    NativeDropShadowRendering,
    /// Allocating an effect texture.
    EffectTextureAllocation,
    /// Presenting an effect result to a surface.
    EffectPresentation,
}

/// Runtime reason that a GPU capability cannot serve an operation.
///
/// These reasons reject the owning GPU operation; there is no CPU fallback or
/// production CPU retry. Semantic unsupported-input diagnostics remain separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RuntimeCapabilityUnavailableReason {
    /// No compatible adapter is available.
    AdapterUnavailable,
    /// The surface is unavailable in the stated lifecycle condition.
    SurfaceUnavailable {
        /// Lifecycle condition that prevents the surface operation.
        state: RenderSurfaceAvailability,
    },
    /// The selected device has been lost.
    DeviceLost {
        /// Reason reported for the device loss.
        reason: DeviceLossReason,
    },
    /// The selected device has entered a terminal fault state.
    DeviceFaulted {
        /// Class of GPU fault observed for the device.
        kind: GpuFaultKind,
    },
    /// The surface belongs to a different renderer or device generation.
    SurfaceIdentityMismatch {
        /// Identity mismatch detected before the operation.
        kind: SurfaceIdentityMismatchKind,
    },
    /// No effect format satisfies the requested precision policy.
    EffectFormatUnavailable {
        /// Effect precision policy that could not be met.
        policy: EffectQualityPolicy,
    },
    /// Float32 native-filter textures lack required render/sample/copy usages.
    FilterFloat32Unavailable,
    /// The requested effect texture exceeds the selected device limit.
    TextureDimensionExceeded {
        /// Requested physical texture dimensions.
        requested: PhysicalSize,
        /// Maximum supported two-dimensional texture dimension.
        maximum: u32,
    },
    /// The surface format cannot receive the effect result.
    SurfaceFormatUnavailable {
        /// Surface format that cannot receive the effect result.
        format: Format,
    },
}

/// Lifecycle condition that makes a render surface unavailable for an operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderSurfaceAvailability {
    /// The surface is suspended.
    Suspended,
    /// The surface has no renderable extent.
    NonRenderable,
    /// The surface has no initialized readable publication.
    Uninitialized,
    /// The surface is occluded and cannot acquire a frame.
    Occluded,
    /// The surface has been lost.
    Lost,
}

/// Identity mismatch detected between a renderer operation and a surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceIdentityMismatchKind {
    /// The surface belongs to another renderer instance.
    ForeignRenderer,
    /// The surface references a stale device generation.
    StaleDeviceGeneration,
}

/// Reason reported when a selected GPU device is lost.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceLossReason {
    /// The backend did not provide a more specific loss reason.
    Unknown,
    /// The device was explicitly destroyed.
    Destroyed,
}

/// Terminal class of a GPU fault observed by the renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuFaultKind {
    /// A GPU validation failure occurred.
    Validation,
    /// GPU memory was exhausted.
    OutOfMemory,
    /// An internal GPU failure occurred.
    Internal,
}

/// Structured details for an [`ErrorCode::InvalidInput`] diagnostic.
///
/// The payload records the rejected field, its diagnostic rendering, and the
/// violated invariant. Callers obtain it from [`Error::invalid_value_diagnostic`]
/// instead of parsing [`Error::message`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidValue {
    field: String,
    value: String,
    invariant: &'static str,
}

impl InvalidValue {
    /// Records an invalid field value and the rule it violates.
    ///
    /// `value` is formatted once for diagnostics; this constructor performs no
    /// additional validation.
    #[must_use]
    pub fn new(
        field: impl Into<String>,
        value: impl std::fmt::Display,
        invariant: &'static str,
    ) -> Self {
        Self {
            field: field.into(),
            value: format!("{value}"),
            invariant,
        }
    }

    /// Returns the name of the rejected field or input.
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Returns the diagnostic rendering of the rejected value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the rule the rejected value must satisfy.
    #[must_use]
    pub const fn invariant(&self) -> &'static str {
        self.invariant
    }

    fn message(&self) -> String {
        format!(
            "{} value {} is invalid: {}",
            self.field, self.value, self.invariant
        )
    }
}

/// Typed semantic identity of an unsupported authored rendering operation.
///
/// [`Error::unsupported_render_primitive`] carries this payload with
/// [`ErrorCode::UnsupportedPrimitive`], allowing callers to match the family and
/// operation without parsing the diagnostic message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnsupportedPrimitive {
    family: PrimitiveFamily,
    operation: PrimitiveOperation,
}

impl UnsupportedPrimitive {
    /// Creates an unsupported-operation identity from its family and operation.
    #[must_use]
    pub const fn new(family: PrimitiveFamily, operation: PrimitiveOperation) -> Self {
        Self { family, operation }
    }

    /// Returns the operation's rendering family.
    #[must_use]
    pub const fn family(self) -> PrimitiveFamily {
        self.family
    }

    /// Returns the specific unsupported operation.
    #[must_use]
    pub const fn operation(self) -> PrimitiveOperation {
        self.operation
    }

    /// Returns the operation's stable human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        self.operation.label()
    }
}

/// Rendering domain that contains an unsupported authored operation.
///
/// This semantic classification is paired with [`PrimitiveOperation`] inside
/// [`UnsupportedPrimitive`]. It does not report runtime device capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveFamily {
    /// Geometry targets and geometry operations.
    GeometryTargets,
    /// Paint sources and color handling.
    PaintSources,
    /// Image sampling, placement, and conversion.
    ImageSampling,
    /// Shadow shapes and kinds.
    Shadows,
    /// Filter lists and filter execution.
    Filters,
    /// Clip and mask operations.
    MasksAndClips,
    /// Border, outline, and box-decoration operations.
    BoxDecorations,
    /// Text-decoration operations.
    TextDecorations,
    /// Opacity, blending, and compositing operations.
    Compositing,
    /// Direct and GPU-graph offscreen operations.
    OffscreenPipeline,
    /// Render-surface operations.
    Surfaces,
    /// Transform and coordinate-space operations.
    TransformsAndCoordinateSpaces,
}

impl PrimitiveFamily {
    /// Returns a stable human-readable label for the rendering family.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::GeometryTargets => "geometry targets",
            Self::PaintSources => "paint sources",
            Self::ImageSampling => "image sampling",
            Self::Shadows => "shadows",
            Self::Filters => "filters",
            Self::MasksAndClips => "masks and clips",
            Self::BoxDecorations => "box decorations",
            Self::TextDecorations => "text decorations",
            Self::Compositing => "compositing",
            Self::OffscreenPipeline => "offscreen pipeline",
            Self::Surfaces => "surfaces",
            Self::TransformsAndCoordinateSpaces => "transforms and coordinate spaces",
        }
    }
}

/// Specific authored or normalized rendering operation used in capability diagnostics.
///
/// Values are paired with a [`PrimitiveFamily`] in [`UnsupportedPrimitive`].
/// They describe semantic operations, not enabled Cargo features or facts about
/// a selected runtime device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveOperation {
    /// Applying a filter to an authored layer.
    LayerFilter,
    /// Clipping with an authored shape.
    ShapeClip,
    /// Executing a referenced clip resource.
    ClipReferenceExecution,
    /// Applying an authored layer mask.
    LayerMask,
    /// Executing an authored alpha-mask source.
    AlphaMaskSourceExecution,
    /// Executing resolved image coverage from alpha or luminance.
    ResolvedAlphaMaskExecution,
    /// Interpreting an authored mask as luminance.
    LuminanceMaskMode,
    /// Composing more than one mask layer.
    MultiLayerMaskComposition,
    /// Applying an authored mask composite mode.
    MaskCompositeMode,
    /// Painting a shadow with a non-solid paint source.
    NonSolidShadowPaint,
    /// Rendering a color that has not been symbolically resolved.
    UnresolvedSymbolicColor,
    /// Evaluating an authored color-mix function.
    ColorMixFunction,
    /// Converting or rendering an unsupported color space.
    UnsupportedColorSpace,
    /// Rendering a repeating gradient.
    RepeatingGradient,
    /// Repeating a background image with `round` sizing.
    BackgroundRepeatRound,
    /// Repeating a background image with spaced placement.
    BackgroundRepeatSpace,
    /// Applying a filter to image paint.
    FilteredImagePaint,
    /// Applying a color filter to image paint.
    ColorFilteredImagePaint,
    /// Converting image orientation at the rendering boundary.
    ImageOrientationConversion,
    /// Converting an image color profile at the rendering boundary.
    ImageColorProfileConversion,
    /// Rendering a shadow from an ellipse or arbitrary path.
    EllipsePathShadowShape,
    /// Rendering an inset box shadow.
    InsetBoxShadow,
    /// Rendering a text shadow.
    TextShadow,
    /// Stroking an arbitrary path with inside or outside alignment.
    InsideOutsidePathStrokeAlignment,
    /// Applying a boolean operation to geometry.
    GeometryBooleanOperation,
    /// Offsetting geometry.
    GeometryOffsetOperation,
    /// Constructing a web-canvas render surface.
    WebCanvasSurface,
    /// Applying a `matrix3d` transform.
    Matrix3dTransform,
    /// Applying a perspective transform.
    PerspectiveTransform,
    /// Applying a three-dimensional rotation.
    Rotate3dTransform,
    /// Translating along the z axis.
    TranslateZTransform,
    /// Scaling along the z axis.
    ScaleZTransform,
    /// Rendering a groove border.
    BorderGrooveStyle,
    /// Rendering a ridge border.
    BorderRidgeStyle,
    /// Rendering an inset border.
    BorderInsetStyle,
    /// Rendering an outset border.
    BorderOutsetStyle,
    /// Rendering a double outline.
    OutlineDoubleStyle,
    /// Rendering an automatic outline.
    OutlineAutoStyle,
    /// Rendering an authored text-decoration style.
    TextDecorationStyle,
    /// Rendering an authored layer into a general offscreen target.
    OffscreenLayerRendering,
    /// Retaining effect resources across frame transactions.
    PersistentEffectResources,
    /// Capturing a bounded Vello span into a graph resource.
    BoundedVelloCapture,
    /// Executing an image-processing GPU graph pass.
    ImagePassExecution,
    /// Executing a GPU graph composition pass.
    CompositePassExecution,
    /// Composing nested opacity through ordered passes.
    NestedOpacityComposition,
    /// Executing a general mask operation in the offscreen pipeline.
    MaskExecution,
    /// Executing a general layer filter in the offscreen pipeline.
    LayerFilterExecution,
    /// Preserving and executing an ordered filter list.
    OrderedFilterList,
    /// Executing a color-filter pass on the GPU.
    GpuColorFilterExecution,
    /// Executing a blur-filter pass on the GPU.
    GpuBlurFilterExecution,
    /// Executing a drop-shadow filter pass on the GPU.
    GpuDropShadowFilterExecution,
    /// Executing an admitted resolved convolution through the native Layer GPU receiver.
    GpuConvolutionFilterExecution,
    /// Executing an admitted native image-space offset through the Layer GPU receiver.
    GpuOffsetFilterExecution,
    /// Executing admitted native Gaussian filtering on the GPU.
    GpuGaussianFilterExecution,
    /// Executing admitted native alpha-blur, displacement, flood and merge.
    GpuNativeDropShadowFilterExecution,
    /// Planning filter execution regions and outsets.
    FilterRegionPlanning,
    /// Executing an unbounded or root/nested backdrop form.
    BroadBackdropExecution,
    /// Capturing bounded backdrop content for filtering.
    BoundedBackdropCapture,
    /// Executing the supported bounded backdrop-filter subset.
    BoundedBackdropFilterExecution,
    /// Isolating and composing a backdrop through separate phases.
    BackdropIsolationComposition,
    /// Applying root-backdrop compositing policy.
    RootBackdropPolicy,
    /// Blending an authored background layer.
    BackgroundBlendMode,
    /// Applying a Porter-Duff composite mode.
    PorterDuffCompositeMode,
    /// Applying one of the twelve basic Porter-Duff operators.
    BasicPorterDuffCompositeMode,
    /// Applying saturating PlusLighter layer composition.
    PlusLighterComposition,
    /// Executing an explicitly isolated resolved layer group.
    IsolatedLayerExecution,
}

impl PrimitiveOperation {
    /// Returns a stable human-readable label for the operation.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::LayerFilter => "layer filter",
            Self::ShapeClip => "shape clip",
            Self::ClipReferenceExecution => "clip reference execution",
            Self::LayerMask => "layer mask",
            Self::AlphaMaskSourceExecution => "alpha mask source execution",
            Self::ResolvedAlphaMaskExecution => "resolved alpha-mask execution",
            Self::LuminanceMaskMode => "luminance mask mode",
            Self::MultiLayerMaskComposition => "multi-layer mask composition",
            Self::MaskCompositeMode => "mask composite mode",
            Self::NonSolidShadowPaint => "non-solid shadow paint",
            Self::UnresolvedSymbolicColor => "unresolved symbolic color",
            Self::ColorMixFunction => "color-mix function",
            Self::UnsupportedColorSpace => "unsupported color space",
            Self::RepeatingGradient => "repeating gradient",
            Self::BackgroundRepeatRound => "background repeat round",
            Self::BackgroundRepeatSpace => "background repeat space",
            Self::FilteredImagePaint => "filtered image paint",
            Self::ColorFilteredImagePaint => "color-filtered image paint",
            Self::ImageOrientationConversion => "image orientation conversion",
            Self::ImageColorProfileConversion => "image color profile conversion",
            Self::EllipsePathShadowShape => "ellipse/path shadow shape",
            Self::InsetBoxShadow => "inset box shadow",
            Self::TextShadow => "text shadow",
            Self::InsideOutsidePathStrokeAlignment => "inside/outside path stroke alignment",
            Self::GeometryBooleanOperation => "geometry boolean operation",
            Self::GeometryOffsetOperation => "geometry offset operation",
            Self::WebCanvasSurface => "web canvas surface",
            Self::Matrix3dTransform => "matrix3d transform",
            Self::PerspectiveTransform => "perspective transform",
            Self::Rotate3dTransform => "rotate3d transform",
            Self::TranslateZTransform => "translateZ transform",
            Self::ScaleZTransform => "scaleZ transform",
            Self::BorderGrooveStyle => "border groove style",
            Self::BorderRidgeStyle => "border ridge style",
            Self::BorderInsetStyle => "border inset style",
            Self::BorderOutsetStyle => "border outset style",
            Self::OutlineDoubleStyle => "outline double style",
            Self::OutlineAutoStyle => "outline auto style",
            Self::TextDecorationStyle => "text decoration style",
            Self::OffscreenLayerRendering => "offscreen layer rendering",
            Self::PersistentEffectResources => "persistent effect resources",
            Self::BoundedVelloCapture => "bounded Vello capture",
            Self::ImagePassExecution => "image-pass execution",
            Self::CompositePassExecution => "composite-pass execution",
            Self::NestedOpacityComposition => "nested opacity composition",
            Self::MaskExecution => "mask execution",
            Self::LayerFilterExecution => "layer-filter execution",
            Self::OrderedFilterList => "ordered filter list",
            Self::GpuColorFilterExecution => "GPU color-filter execution",
            Self::GpuBlurFilterExecution => "GPU blur filter execution",
            Self::GpuDropShadowFilterExecution => "GPU drop-shadow filter execution",
            Self::GpuConvolutionFilterExecution => "GPU convolution filter execution",
            Self::GpuOffsetFilterExecution => "GPU offset filter execution",
            Self::GpuGaussianFilterExecution => "GPU Gaussian filter execution",
            Self::GpuNativeDropShadowFilterExecution => "GPU native drop-shadow filter execution",
            Self::FilterRegionPlanning => "filter-region planning",
            Self::BroadBackdropExecution => "broad backdrop execution",
            Self::BoundedBackdropCapture => "bounded backdrop capture",
            Self::BoundedBackdropFilterExecution => "bounded backdrop filter execution",
            Self::BackdropIsolationComposition => "backdrop isolation/composition",
            Self::RootBackdropPolicy => "root backdrop policy",
            Self::BackgroundBlendMode => "background blend mode",
            Self::PorterDuffCompositeMode => "Porter-Duff composite mode",
            Self::BasicPorterDuffCompositeMode => "basic Porter-Duff composite mode",
            Self::PlusLighterComposition => "plus-lighter composition",
            Self::IsolatedLayerExecution => "isolated layer execution",
        }
    }
}

/// Structured details for an [`ErrorCode::UnresolvedResource`] diagnostic.
///
/// The payload identifies the resource category and the caller-provided or
/// normalized identifier that could not be resolved. Callers obtain it through
/// [`Error::unresolved_resource_diagnostic`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnresolvedResource {
    kind: UnresolvedResourceKind,
    identifier: String,
}

impl UnresolvedResource {
    /// Records the resource kind and unresolved identifier.
    #[must_use]
    pub fn new(kind: UnresolvedResourceKind, identifier: impl Into<String>) -> Self {
        Self {
            kind,
            identifier: identifier.into(),
        }
    }

    /// Returns the unresolved resource category.
    #[must_use]
    pub const fn kind(&self) -> UnresolvedResourceKind {
        self.kind
    }

    /// Returns the identifier that could not be resolved.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    fn message(&self) -> String {
        format!(
            "{} resource {} could not be resolved",
            self.kind.label(),
            self.identifier
        )
    }
}

/// Category of resource named by an [`UnresolvedResource`] payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnresolvedResourceKind {
    /// An image resource.
    Image,
    /// A mask resource.
    Mask,
    /// A filter resource.
    Filter,
    /// A clip resource.
    Clip,
    /// Ink bounds required for a text run.
    TextRunInkBounds,
}

impl UnresolvedResourceKind {
    /// Returns a stable human-readable label for the resource category.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Mask => "mask",
            Self::Filter => "filter",
            Self::Clip => "clip",
            Self::TextRunInkBounds => "text run ink bounds",
        }
    }
}

/// Structured details for an [`ErrorCode::DegradedQuality`] diagnostic.
///
/// The payload identifies the quality limitation and the affected value. Callers
/// obtain it through [`Error::degraded_quality_diagnostic`] rather than parsing
/// [`Error::message`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DegradedQuality {
    kind: DegradedQualityKind,
    value: String,
}

impl DegradedQuality {
    /// Records a quality limitation and its diagnostic value.
    #[must_use]
    pub fn new(kind: DegradedQualityKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }

    /// Returns the class of quality limitation.
    #[must_use]
    pub const fn kind(&self) -> DegradedQualityKind {
        self.kind
    }

    /// Returns the value associated with the limitation.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    fn message(&self) -> String {
        format!(
            "render quality degraded: {} ({})",
            self.kind.label(),
            self.value
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Describes a quality limitation reported by a [`ErrorCode::DegradedQuality`] diagnostic.
pub enum DegradedQualityKind {
    /// Reports output produced with reduced intermediate precision.
    ReducedIntermediatePrecision,
    /// Reports inability to perform the requested paint-space conversion.
    UnsupportedPaintSpaceConversion,
}

impl DegradedQualityKind {
    /// Returns a stable human-readable label for this quality limitation.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ReducedIntermediatePrecision => "reduced intermediate precision",
            Self::UnsupportedPaintSpaceConversion => "unsupported paint-space conversion",
        }
    }
}

#[cfg(test)]
mod hint_error_tests {
    use super::*;

    #[test]
    fn intrinsic_hint_and_contextual_representation_errors_keep_distinct_typed_domains() {
        for diagnostic in [
            GradientHintError::TrailingHint { stop_index: 2 },
            GradientHintError::HintOutsideInterval { stop_index: 3 },
            GradientHintError::UnsupportedSweep { stop_index: 4 },
        ] {
            let error = Error::from(diagnostic);
            assert_eq!(error.code(), ErrorCode::GradientHint);
            assert_eq!(error.gradient_hint_diagnostic(), Some(&diagnostic));
            assert!(error.gradient_realization_diagnostic().is_none());
            assert!(error.accounting_diagnostic().is_none());
        }
        let diagnostic = GradientRealizationError::UnrepresentableHint { stop_index: 5 };
        let error = Error::from(diagnostic);
        assert_eq!(error.code(), ErrorCode::GradientRealization);
        assert_eq!(error.gradient_realization_diagnostic(), Some(&diagnostic));
        assert!(error.gradient_hint_diagnostic().is_none());
    }
}

#[cfg(test)]
mod geometry_error_tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn geometry_refusals_preserve_their_exact_reason_and_local_input_index() {
        for diagnostic in [
            GeometryRealizationError::UnrepresentableTransform,
            GeometryRealizationError::UnrepresentablePath { element_index: 7 },
            GeometryRealizationError::UnrepresentableStroke,
            GeometryRealizationError::UnrepresentableGlyph { glyph_index: 11 },
            GeometryRealizationError::UnrepresentableBlur,
            GeometryRealizationError::InsufficientNativeCapacity,
        ] {
            let error = Error::from(diagnostic);
            assert_eq!(error.code(), ErrorCode::GeometryRealization);
            assert_eq!(error.geometry_realization_diagnostic(), Some(&diagnostic));
            assert!(error.invalid_value_diagnostic().is_none());
            assert!(error.unsupported_primitive().is_none());
            assert!(error.degraded_quality_diagnostic().is_none());
            assert!(error.accounting_diagnostic().is_none());
            assert!(error.gradient_realization_diagnostic().is_none());
            assert!(error.gradient_hint_diagnostic().is_none());
            assert!(error.glyph_preparation_diagnostic().is_none());
            assert!(error.source().is_none());
            #[cfg(feature = "render-paint")]
            assert!(error.paint_gradient_diagnostic().is_none());
            #[cfg(feature = "render-text")]
            assert!(error.text_layout_diagnostic().is_none());
        }
        assert_ne!(
            GeometryRealizationError::UnrepresentablePath { element_index: 7 },
            GeometryRealizationError::UnrepresentablePath { element_index: 8 },
        );
        assert_ne!(
            GeometryRealizationError::UnrepresentablePath { element_index: 7 },
            GeometryRealizationError::UnrepresentableGlyph { glyph_index: 7 },
        );
    }

    #[derive(Debug)]
    struct Cause;

    impl fmt::Display for Cause {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("native receiving context")
        }
    }

    impl error::Error for Cause {}

    #[test]
    fn geometry_identity_survives_human_context_and_source_enrichment() {
        let diagnostic = GeometryRealizationError::UnrepresentablePath {
            element_index: usize::MAX,
        };
        let mut error = Error::from(diagnostic).with_source(Cause);
        error = error.append_message(format_args!(" at command {}", 3));
        assert_eq!(error.code(), ErrorCode::GeometryRealization);
        assert_eq!(error.geometry_realization_diagnostic(), Some(&diagnostic));
        assert_eq!(
            error.source().unwrap().to_string(),
            "native receiving context"
        );
        error = error.replace_message("qualified receiving failure");
        assert_eq!(error.geometry_realization_diagnostic(), Some(&diagnostic));
        assert_eq!(
            error.source().unwrap().to_string(),
            "native receiving context"
        );
        assert_eq!(
            error.to_string(),
            "GeometryRealization: qualified receiving failure"
        );
    }

    #[test]
    fn existing_gradient_accounting_input_and_device_errors_retain_their_domains() {
        let gradient = GradientRealizationError::UnrepresentableStop { stop_index: 2 };
        let error = Error::from(gradient);
        assert_eq!(error.code(), ErrorCode::GradientRealization);
        assert_eq!(error.gradient_realization_diagnostic(), Some(&gradient));
        assert!(error.geometry_realization_diagnostic().is_none());
        let accounting = crate::RenderAccountingError::Invariant;
        let error = Error::from(accounting);
        assert_eq!(error.code(), ErrorCode::Accounting);
        assert_eq!(error.accounting_diagnostic(), Some(accounting));
        assert!(error.geometry_realization_diagnostic().is_none());
        let error = Error::invalid_value("path", "nonfinite", "must be finite");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
        assert!(error.geometry_realization_diagnostic().is_none());
        let error = Error::new(BackendErrorCode::RenderFailed, "native backend failure");
        assert_eq!(error.code(), ErrorCode::RenderFailed);
        assert!(error.geometry_realization_diagnostic().is_none());
    }
}
