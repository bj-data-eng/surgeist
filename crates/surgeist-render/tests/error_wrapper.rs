//! `Error` built from each typed diagnostic prints its message.
//!
//! The message is the typed diagnostic's own text with its cause chain joined by `": "`; the typed
//! value stays reachable through its `*_diagnostic()` accessor, and `source()` keeps returning only
//! a backend source (none here), so a reporter that walks the chain does not print the diagnostic
//! twice. The classification is `code()` and is not part of the text.

use core::error::Error as StdError;
use surgeist_render::{
    ConvolutionNumericPhase, ConvolutionPreparationError, DisplacementPreparationError,
    DropShadowPreparationError, Error, ErrorCode, GaussianPreparationError,
    GeometryRealizationError, GlyphPreparationError, GradientHintError, GradientRealizationError,
    MorphologyPreparationError, OffsetParameter, OffsetPreparationError,
};

#[track_caller]
fn assert_wrapped(error: Error, code: ErrorCode, message: &str) {
    assert_eq!(error.code(), code);
    assert_eq!(error.message(), message);
    assert_eq!(error.to_string(), message);
    assert!(
        error.source().is_none(),
        "a typed diagnostic is reported through its accessor, not as a source: {error:?}"
    );
}

#[test]
fn filter_preparation_errors_are_invalid_input() {
    let convolution = ConvolutionPreparationError::Numeric {
        phase: ConvolutionNumericPhase::TapCount,
        value: 0.0,
    };
    assert_wrapped(
        Error::from(convolution),
        ErrorCode::InvalidInput,
        "convolution numeric check failed at tap count",
    );
    let offset = OffsetPreparationError::InvalidParameter {
        parameter: OffsetParameter::Dy,
        value: f64::NAN,
    };
    assert_wrapped(
        Error::from(offset),
        ErrorCode::InvalidInput,
        "offset parameter vertical displacement has the invalid value NaN",
    );
    assert_wrapped(
        Error::from(DropShadowPreparationError::EmptyPeriod),
        ErrorCode::InvalidInput,
        "a periodic source must have positive width and height",
    );
    assert_wrapped(
        Error::from(GaussianPreparationError::InfiniteSourceBackdrop),
        ErrorCode::InvalidInput,
        "a same-layer backdrop cannot be combined with a declared infinite source",
    );
    assert_wrapped(
        Error::from(DisplacementPreparationError::InvalidMapPayload {
            expected: 16,
            actual: 3,
        }),
        ErrorCode::InvalidInput,
        "displacement map has 3 bytes but its dimensions require 16",
    );
    assert_wrapped(
        Error::from(MorphologyPreparationError::EmptyPeriod),
        ErrorCode::InvalidInput,
        "a periodic source must have positive width and height",
    );
}

#[test]
fn glyph_gradient_and_geometry_errors_keep_their_own_classification() {
    assert_wrapped(
        Error::from(GlyphPreparationError::InvalidFont),
        ErrorCode::GlyphPreparation,
        "font, outline, color or bitmap data is invalid",
    );
    assert_wrapped(
        Error::from(GradientHintError::TrailingHint { stop_index: 3 }),
        ErrorCode::GradientHint,
        "stop 3 is the last stop and cannot carry an outgoing hint",
    );
    assert_wrapped(
        Error::from(GradientRealizationError::UnrepresentableStop { stop_index: 2 }),
        ErrorCode::GradientRealization,
        "stop 2 cannot keep its source position, ordering or separation natively",
    );
    assert_wrapped(
        Error::from(GeometryRealizationError::UnrepresentablePath { element_index: 7 }),
        ErrorCode::GeometryRealization,
        "path element 7 cannot be converted or retained natively",
    );
}
