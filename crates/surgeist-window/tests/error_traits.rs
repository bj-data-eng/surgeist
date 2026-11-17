//! Public error contract: message text and the standard error traits.
//!
//! Expected texts are written from the documented meaning of each code and
//! carry no `Debug` rendering of the code.

use std::error::Error as StdError;
use surgeist_window::{Error, ErrorCode, registry::Id, system_fonts::SystemFontError};

fn assert_error_traits<T: StdError + Send + Sync + 'static>() {}

#[test]
fn error_is_a_send_sync_standard_error() {
    assert_error_traits::<Error>();
}

#[test]
fn display_without_a_window_is_the_code_phrase_then_the_message() {
    let error = Error::new(ErrorCode::InvalidRequest, "window name must not be empty");

    assert_eq!(
        error.to_string(),
        "invalid request: window name must not be empty"
    );
}

#[test]
fn display_with_a_window_appends_its_identity() {
    let error =
        Error::new(ErrorCode::StaleWindow, "window generation is closed").with_id(Id::from_u64(7));

    assert_eq!(
        error.to_string(),
        "stale window: window generation is closed (window 7)"
    );
}

#[test]
fn display_does_not_render_the_debug_form_of_the_code() {
    let text = Error::new(ErrorCode::WindowCreateFailed, "backend refused").to_string();

    assert!(
        !text.contains("WindowCreateFailed"),
        "display must be a semantic phrase, got {text:?}"
    );
}

#[test]
fn source_is_the_retained_underlying_error() {
    let error = Error::new(ErrorCode::CommandBatchFailed, "backend failed")
        .with_source(std::io::Error::other("native call rejected"));

    let source = error.source().expect("the retained source");
    assert_eq!(source.to_string(), "native call rejected");
}

#[test]
fn source_is_absent_without_an_underlying_error() {
    let error = Error::new(ErrorCode::CommandFailed, "event loop is closed");

    assert!(error.source().is_none());
}

/// Each phrase is derived from the variant's documented meaning in `ErrorCode`.
const CODE_PHRASES: [(ErrorCode, &str); 20] = [
    (ErrorCode::InvalidRequest, "invalid request"),
    (
        ErrorCode::DuplicateIdentity,
        "duplicate window identity or name",
    ),
    (ErrorCode::IdentityExhausted, "window identities exhausted"),
    (ErrorCode::StaleWindow, "stale window"),
    (ErrorCode::WindowClosing, "window is closing"),
    (
        ErrorCode::EventLoopCreateFailed,
        "event loop creation failed",
    ),
    (ErrorCode::WindowCreateFailed, "window creation failed"),
    (ErrorCode::HandleUnavailable, "window handle unavailable"),
    (
        ErrorCode::ImeUnsupported,
        "input method request unsupported",
    ),
    (ErrorCode::ImeRequestFailed, "input method request failed"),
    (ErrorCode::ClipboardUnavailable, "clipboard unavailable"),
    (ErrorCode::ClipboardReadFailed, "clipboard read failed"),
    (ErrorCode::ClipboardWriteFailed, "clipboard write failed"),
    (ErrorCode::CursorRequestFailed, "cursor request failed"),
    (ErrorCode::CommandFailed, "window command failed"),
    (ErrorCode::CommandBatchFailed, "window command batch failed"),
    (
        ErrorCode::OutstandingHandle,
        "native handle outstanding at event loop exit",
    ),
    (ErrorCode::UnsupportedFeature, "unsupported feature"),
    (
        ErrorCode::AccessibilityAdapterFailed,
        "accessibility adapter failed",
    ),
    (ErrorCode::UnknownNativeError, "unknown native error"),
];

#[test]
fn error_code_display_is_the_documented_phrase_for_every_variant() {
    for (code, phrase) in CODE_PHRASES {
        assert_eq!(code.to_string(), phrase, "display of {code:?}");
    }
}

#[test]
fn error_display_prefixes_the_message_with_the_code_phrase_for_every_variant() {
    for (code, phrase) in CODE_PHRASES {
        assert_eq!(
            Error::new(code, "detail").to_string(),
            format!("{phrase}: detail"),
            "display of an error with {code:?}"
        );
    }
}

#[test]
fn system_font_error_display_describes_each_failure() {
    let cases = [
        (
            SystemFontError::UnsupportedPlatform,
            "native UI font observation is unsupported on this target",
        ),
        (
            SystemFontError::MainThreadRequired,
            "native UI font observation requires the main thread",
        ),
        (
            SystemFontError::MissingFamily,
            "a system font requires a nonempty family name",
        ),
        (
            SystemFontError::MissingFace,
            "a system font requires a nonempty face name",
        ),
        (
            SystemFontError::InvalidMetrics,
            "system font size and metrics are invalid",
        ),
        (
            SystemFontError::InvalidWeight,
            "system font weight is outside its native range",
        ),
        (
            SystemFontError::NativeQuery {
                operation: "SystemParametersInfoW",
                code: 5,
            },
            "native UI font query SystemParametersInfoW failed (Windows error 5)",
        ),
        (
            SystemFontError::InvalidNativeData,
            "native UI font data is invalid",
        ),
    ];

    for (error, text) in cases {
        assert_eq!(error.to_string(), text, "display of {error:?}");
        assert!(error.source().is_none(), "{error:?} wraps no other error");
    }
}

#[test]
fn system_font_error_is_a_send_sync_standard_error() {
    assert_error_traits::<SystemFontError>();
}
