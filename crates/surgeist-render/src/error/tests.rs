use super::{Error, ErrorCode};
use crate::{RenderAccountingError, RenderQuantity, RenderRefusalReason, RenderWork};
use std::{error::Error as _, fmt};

#[test]
fn accounting_failure_preserves_typed_cause_and_full_human_message() {
    let diagnostic = RenderAccountingError::GenerationRefused {
        quantity: RenderQuantity::Work(RenderWork::ResourceAllocations),
        current: 3,
        amount: 1,
        reason: RenderRefusalReason::Limit { limit: 3 },
    };
    let error = Error::from(diagnostic);
    let message =
        "generation accounting refused 1 of resource allocations work at usage 3: the limit is 3";
    assert_eq!(error.code(), ErrorCode::Accounting);
    assert_eq!(error.accounting_diagnostic(), Some(diagnostic));
    assert_eq!(error.to_string(), format!("{message}"));
    assert_eq!(error.message(), message);
    assert_eq!(error.message(), message);
    assert!(error.source().is_none());
    assert!(error.glyph_preparation_diagnostic().is_none());
    assert!(error.invalid_value_diagnostic().is_none());
    assert!(error.unsupported_primitive().is_none());
    assert!(error.unresolved_resource_diagnostic().is_none());
    assert!(error.degraded_quality_diagnostic().is_none());
    assert!(error.runtime_capability_unavailable_diagnostic().is_none());
    #[cfg(feature = "render-text")]
    assert!(error.text_layout_diagnostic().is_none());
}

#[test]
fn const_accounting_failure_is_ready_for_typed_observation() {
    // Const evaluation requires neither runtime formatting nor an allocated
    // diagnostic context; the initialized error then crosses the real API.
    let error = const { Error::accounting(RenderAccountingError::GenerationRetired) };
    assert_eq!(error.code(), ErrorCode::Accounting);
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::GenerationRetired)
    );
    assert_eq!(
        error.message(),
        "the device generation is retired and cannot start new work"
    );
}

#[derive(Debug)]
struct Cause;
impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("source marker")
    }
}
impl std::error::Error for Cause {}

#[test]
fn explicit_message_and_source_enrichment_keep_the_accounting_cause() {
    let diagnostic = RenderAccountingError::Invariant;
    let mut error = Error::from(diagnostic);
    error = error.append_message(format_args!(" at command {}", 7));
    let mut error = error.with_source(Cause);
    assert_eq!(error.code(), ErrorCode::Accounting);
    assert_eq!(error.accounting_diagnostic(), Some(diagnostic));
    assert_eq!(
        error.message(),
        "an internal consistency check failed at command 7"
    );
    assert_eq!(error.source().unwrap().to_string(), "source marker");
    error = error.replace_message("replaced diagnostic prose");
    assert_eq!(error.message(), "replaced diagnostic prose");
    assert_eq!(error.accounting_diagnostic(), Some(diagnostic));
    assert_eq!(error.source().unwrap().to_string(), "source marker");
    assert_eq!(error.to_string(), "replaced diagnostic prose");
}

#[test]
fn accounting_observers_stay_lazy_until_message_or_enrichment() {
    let mut error = Error::from(RenderAccountingError::Invariant);
    assert_eq!(error.code(), ErrorCode::Accounting);
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Invariant)
    );
    assert_eq!(error.to_string(), "an internal consistency check failed");
    let super::ErrorPayload::Accounting(failure) = &error.payload else {
        panic!("typed and Display observation must retain fixed accounting data");
    };
    assert!(failure.message.get().is_none());
    error = error.append_message(" at command 7");
    assert_eq!(
        error.message(),
        "an internal consistency check failed at command 7"
    );
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Invariant)
    );
}

fn context_address(error: &Error) -> *const super::ErrorContext {
    match &error.payload {
        super::ErrorPayload::Context(context) => &**context,
        super::ErrorPayload::Accounting(_) => {
            panic!("explicit enrichment must materialize context")
        }
    }
}

fn source_address(error: &Error) -> *const std::io::Error {
    error
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap()
}

#[test]
fn accounting_source_enrichment_moves_cached_message_and_formats_uncached_message() {
    for cached in [false, true] {
        let error = Error::from(RenderAccountingError::GenerationRetired);
        let cached_address = cached.then(|| {
            assert_eq!(
                error.message(),
                "the device generation is retired and cannot start new work"
            );
            error.message().as_ptr()
        });
        let error = error.with_source(std::io::Error::other("original source"));
        assert_eq!(
            error.message(),
            "the device generation is retired and cannot start new work"
        );
        if let Some(address) = cached_address {
            assert_eq!(error.message().as_ptr(), address);
        }
        assert_context_enrichment(
            error,
            "the device generation is retired and cannot start new work",
            |error| {
                assert_eq!(error.code(), ErrorCode::Accounting);
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::GenerationRetired)
                );
            },
        );
    }
}

// Allocation identity is an explicit requirement of enrichment: observe the
// real constructor-owned box, message allocation, and attached source object.
fn assert_context_enrichment(mut error: Error, original_message: &str, check: impl Fn(&Error)) {
    assert_eq!(error.message(), original_message);
    check(&error);
    let context = context_address(&error);
    let message = error.message().as_ptr();
    error = error.with_source(std::io::Error::other("source marker"));
    assert_eq!(context_address(&error), context);
    assert_eq!(error.message().as_ptr(), message);
    let source = source_address(&error);
    error = error.append_message(" at command 7");
    error = error.append_message(" during admission");
    assert_eq!(
        error.message(),
        format!("{original_message} at command 7 during admission")
    );
    check(&error);
    error = error.replace_message("qualified failure");
    error = error.append_message(" after normalization");
    assert_eq!(error.message(), "qualified failure after normalization");
    assert_eq!(source_address(&error), source);
    assert_eq!(error.source().unwrap().to_string(), "source marker");
    assert_eq!(context_address(&error), context);
    check(&error);
    error = error.with_source(std::io::Error::other("replacement source"));
    assert_eq!(error.source().unwrap().to_string(), "replacement source");
    assert_eq!(error.message(), "qualified failure after normalization");
    assert_eq!(context_address(&error), context);
    check(&error);
}

#[test]
fn enrichment_preserves_constructor_qualified_semantic_and_preparation_contexts() {
    use super::*;
    assert_context_enrichment(
        Error::invalid_value("radius", -1, "must be non-negative"),
        "radius value -1 is invalid: must be non-negative",
        |error| {
            assert_eq!(error.code(), ErrorCode::InvalidInput);
            assert_eq!(
                error.invalid_value_diagnostic(),
                Some(&InvalidValue::new("radius", -1, "must be non-negative"))
            );
        },
    );
    let primitive =
        UnsupportedPrimitive::new(PrimitiveFamily::Filters, PrimitiveOperation::LayerFilter);
    assert_context_enrichment(
        Error::unsupported_render_primitive(primitive),
        "render primitive is unsupported: filters / layer filter",
        |error| {
            assert_eq!(error.code(), ErrorCode::UnsupportedPrimitive);
            assert_eq!(error.unsupported_primitive(), Some(primitive));
        },
    );
    let resource = UnresolvedResource::new(UnresolvedResourceKind::Image, "hero-image");
    assert_context_enrichment(
        Error::unresolved_resource(resource.clone()),
        "image resource hero-image could not be resolved",
        |error| {
            assert_eq!(error.code(), ErrorCode::UnresolvedResource);
            assert_eq!(error.unresolved_resource_diagnostic(), Some(&resource));
        },
    );
    let quality = DegradedQuality::new(
        DegradedQualityKind::ReducedIntermediatePrecision,
        "rgba16float unavailable",
    );
    assert_context_enrichment(
        Error::degraded_quality(quality.clone()),
        "render quality degraded: reduced intermediate precision (rgba16float unavailable)",
        |error| {
            assert_eq!(error.code(), ErrorCode::DegradedQuality);
            assert_eq!(error.degraded_quality_diagnostic(), Some(&quality));
        },
    );
    let runtime = RuntimeCapabilityUnavailable::rendering_adapter_unavailable();
    assert_context_enrichment(
        Error::runtime_capability_unavailable(runtime),
        "runtime capability is unavailable",
        |error| {
            assert_eq!(error.code(), ErrorCode::RuntimeCapabilityUnavailable);
            assert_eq!(
                error.runtime_capability_unavailable_diagnostic(),
                Some(&runtime)
            );
        },
    );
    let geometry = GeometryRealizationError::UnrepresentablePath { element_index: 7 };
    assert_context_enrichment(
        Error::from(geometry),
        "path element 7 cannot be converted or retained natively",
        |error| {
            assert_eq!(error.code(), ErrorCode::GeometryRealization);
            assert_eq!(error.geometry_realization_diagnostic(), Some(&geometry));
        },
    );
    let gradient = GradientRealizationError::UnrepresentableStop { stop_index: 2 };
    assert_context_enrichment(
        Error::from(gradient),
        "stop 2 cannot keep its source position, ordering or separation natively",
        |error| {
            assert_eq!(error.code(), ErrorCode::GradientRealization);
            assert_eq!(error.gradient_realization_diagnostic(), Some(&gradient));
        },
    );
    let hint = GradientHintError::TrailingHint { stop_index: 3 };
    assert_context_enrichment(
        Error::from(hint),
        "stop 3 is the last stop and cannot carry an outgoing hint",
        |error| {
            assert_eq!(error.code(), ErrorCode::GradientHint);
            assert_eq!(error.gradient_hint_diagnostic(), Some(&hint));
        },
    );
    let glyph = crate::GlyphPreparationError::InvalidFont;
    assert_context_enrichment(
        Error::from(glyph),
        "font, outline, color or bitmap data is invalid",
        |error| {
            assert_eq!(error.code(), ErrorCode::GlyphPreparation);
            assert_eq!(error.glyph_preparation_diagnostic(), Some(&glyph));
        },
    );
}

#[cfg(feature = "render-text")]
#[test]
fn enrichment_preserves_text_layout_cause() {
    assert_context_enrichment(
        Error::from(crate::TextLayoutError::CapacityOverflow),
        "a count calculation overflowed",
        |error| {
            assert_eq!(error.code(), ErrorCode::TextLayout);
            assert!(matches!(
                error.text_layout_diagnostic(),
                Some(crate::TextLayoutError::CapacityOverflow)
            ));
        },
    );
}

#[cfg(feature = "render-paint")]
#[test]
fn enrichment_preserves_paint_gradient_cause() {
    assert_context_enrichment(
        Error::from(crate::PaintGradientError::UnsupportedZeroLengthLinear),
        "a linear gradient of zero length cannot be prepared",
        |error| {
            assert_eq!(error.code(), ErrorCode::PaintGradient);
            assert_eq!(
                error.paint_gradient_diagnostic(),
                Some(&crate::PaintGradientError::UnsupportedZeroLengthLinear)
            );
        },
    );
}

#[test]
fn accounting_prose_enrichment_preserves_cause_with_or_without_cached_message() {
    for cached in [false, true] {
        for replace in [false, true] {
            let mut error = Error::from(RenderAccountingError::Invariant);
            if cached {
                assert_eq!(error.message(), "an internal consistency check failed");
            }
            let expected = if replace {
                error = error.replace_message("qualified accounting failure");
                "qualified accounting failure"
            } else {
                error = error.append_message(" at command 7");
                "an internal consistency check failed at command 7"
            };
            assert_eq!(error.message(), expected);
            assert_eq!(error.code(), ErrorCode::Accounting);
            assert_eq!(
                error.accounting_diagnostic(),
                Some(RenderAccountingError::Invariant)
            );
            assert!(error.source().is_none());
        }
    }
}

#[test]
fn backend_error_codes_keep_exact_mappings_and_custom_prose() {
    use super::BackendErrorCode as B;
    for (backend, public) in [
        (B::RendererCreateFailed, ErrorCode::RendererCreateFailed),
        (B::SurfaceCreateFailed, ErrorCode::SurfaceCreateFailed),
        (B::SurfaceConfigureFailed, ErrorCode::SurfaceConfigureFailed),
        (B::SurfaceOutOfMemory, ErrorCode::SurfaceOutOfMemory),
        (B::SurfaceTimeout, ErrorCode::SurfaceTimeout),
        (B::SurfaceOutdated, ErrorCode::SurfaceOutdated),
        (B::ImageUploadFailed, ErrorCode::ImageUploadFailed),
        (B::RenderFailed, ErrorCode::RenderFailed),
        (B::ReadbackFailed, ErrorCode::ReadbackFailed),
        (B::PresentFailed, ErrorCode::PresentFailed),
        (B::UnsupportedBackend, ErrorCode::UnsupportedBackend),
    ] {
        assert_context_enrichment(
            Error::new(backend, "backend failure"),
            "backend failure",
            |error| {
                assert_eq!(error.code(), public);
                assert!(error.accounting_diagnostic().is_none());
                assert!(error.runtime_capability_unavailable_diagnostic().is_none());
            },
        );
    }
}

#[test]
fn every_backend_code_prints_exactly_its_message_without_the_classification() {
    use super::BackendErrorCode as B;
    for (backend, public, phrase) in [
        (
            B::RendererCreateFailed,
            ErrorCode::RendererCreateFailed,
            "renderer creation failed",
        ),
        (
            B::SurfaceCreateFailed,
            ErrorCode::SurfaceCreateFailed,
            "surface setup failed",
        ),
        (
            B::SurfaceConfigureFailed,
            ErrorCode::SurfaceConfigureFailed,
            "surface configuration unavailable",
        ),
        (
            B::SurfaceOutOfMemory,
            ErrorCode::SurfaceOutOfMemory,
            "gpu memory exhausted",
        ),
        (
            B::SurfaceTimeout,
            ErrorCode::SurfaceTimeout,
            "surface operation timed out",
        ),
        (
            B::SurfaceOutdated,
            ErrorCode::SurfaceOutdated,
            "surface configuration outdated",
        ),
        (
            B::ImageUploadFailed,
            ErrorCode::ImageUploadFailed,
            "image data refused",
        ),
        (B::RenderFailed, ErrorCode::RenderFailed, "render failed"),
        (
            B::ReadbackFailed,
            ErrorCode::ReadbackFailed,
            "readback failed",
        ),
        (
            B::PresentFailed,
            ErrorCode::PresentFailed,
            "presented output failed",
        ),
        (
            B::UnsupportedBackend,
            ErrorCode::UnsupportedBackend,
            "unsupported backend",
        ),
    ] {
        let error = Error::new(backend, "the stage reported a fault");
        assert_eq!(error.code(), public);
        // `Display` is the message alone: the phrase belongs to the code, whose text is separate.
        assert_eq!(error.to_string(), "the stage reported a fault");
        assert_eq!(error.code().to_string(), phrase);
        // A source is reported through `source()` and is not folded into the text.
        let with_source = error.with_source(std::io::Error::other("the device said no"));
        assert_eq!(with_source.to_string(), "the stage reported a fault");
        assert_eq!(
            with_source.source().unwrap().to_string(),
            "the device said no"
        );
        // Enrichment changes the message and `Display` follows it.
        let enriched = with_source.append_message(" at command 3");
        assert_eq!(
            enriched.to_string(),
            "the stage reported a fault at command 3"
        );
        let replaced = enriched.replace_message("replaced");
        assert_eq!(replaced.to_string(), "replaced");
        assert_eq!(replaced.message(), "replaced");
    }
}

#[test]
fn the_runtime_capability_constructor_prints_its_fixed_sentence() {
    let error = Error::runtime_capability_unavailable(
        super::RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
    );
    assert_eq!(error.code(), ErrorCode::RuntimeCapabilityUnavailable);
    assert_eq!(error.to_string(), "runtime capability is unavailable");
    assert_eq!(error.code().to_string(), "runtime capability unavailable");
    // The variant that adds prose keeps the typed payload and prints the prose.
    let described = Error::runtime_unavailable(
        super::RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
        "no adapter can render",
    );
    assert_eq!(described.to_string(), "no adapter can render");
    assert!(
        described
            .runtime_capability_unavailable_diagnostic()
            .is_some()
    );
}

#[test]
fn an_accounting_error_prints_its_chain_and_not_its_classification() {
    let error = Error::from(RenderAccountingError::Setup(
        crate::RenderAccountingSetupError::Allocation,
    ));
    assert_eq!(error.code(), ErrorCode::Accounting);
    assert_eq!(
        error.to_string(),
        "accounting setup failed: accounting could not allocate its setup or report storage"
    );
    assert_eq!(error.code().to_string(), "accounting failed");
    // Printing does not initialize the cached message.
    let super::ErrorPayload::Accounting(failure) = &error.payload else {
        panic!("an accounting diagnostic keeps its fixed payload");
    };
    assert!(failure.message.get().is_none());
}
