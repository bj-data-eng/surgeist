//! Public error contract: message text and the standard error traits.
//!
//! Expected texts are written from the documented meaning of each code and
//! carry no `Debug` rendering of the code.

use std::error::Error as StdError;
use surgeist_window::{Error, ErrorCode, Id};

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
