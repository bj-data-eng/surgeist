//! `Display` and `Error` of the listener callback and host failure codes.
//!
//! Both types carry a caller-defined diagnostic code, so their messages state
//! the documented meaning and print the code as a number. Neither wraps an
//! error. The dispatch error that carries the host refusal reports it from
//! `source()` without repeating its text.
#![forbid(unsafe_code)]

use std::error::Error;

use surgeist_runtime::event::{DomEventCallbackError, DomEventDispatchError, DomEventHostError};

fn assert_send_sync<E: Error + Send + Sync + 'static>() {}

#[test]
fn a_callback_failure_code_is_reported_as_a_number() {
    assert_send_sync::<DomEventCallbackError>();
    let error = DomEventCallbackError { code: 7 };
    assert_eq!(
        error.to_string(),
        "the listener callback reported failure code 7"
    );
    assert!(error.source().is_none());
    assert_eq!(
        DomEventCallbackError { code: u64::MAX }.to_string(),
        "the listener callback reported failure code 18446744073709551615"
    );
    assert_ne!(error.to_string(), format!("{error:?}"));
}

#[test]
fn a_host_refusal_code_is_reported_as_a_number() {
    assert_send_sync::<DomEventHostError>();
    let error = DomEventHostError { code: 0 };
    assert_eq!(
        error.to_string(),
        "the host refused to run the listener, code 0"
    );
    assert!(error.source().is_none());
    assert_ne!(error.to_string(), format!("{error:?}"));
}

#[test]
fn a_dispatch_error_reports_the_host_refusal_from_its_source() {
    let outer = DomEventDispatchError::HostRefused(DomEventHostError { code: 9 });
    assert_eq!(outer.to_string(), "native DOM event host refused execution");
    let source = outer.source().expect("the host refusal is the source");
    assert_eq!(
        source.to_string(),
        "the host refused to run the listener, code 9"
    );
    assert!(!outer.to_string().contains("code 9"));
}
