//! Scratch accounting pairs every release with the reservation it returns. A release the session
//! never reserved is a defect in this crate: it is reported as a release underflow, never wrapped
//! into a huge count, and an enclosing controlled OptionValue execution tracks every live scratch
//! vector.
use super::*;
use crate::controlled_option_value::{Control, QueryControl};
use std::sync::atomic::AtomicBool;
use surgeist_html::{construction::HtmlDocumentMode, values::HtmlValueInterpreter};

use crate::{
    HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlOptionValueEnvelope,
    HtmlScriptingStatus,
};

fn capture() -> HtmlCapture {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_children(&d, &[]).unwrap();
    b.finish().unwrap()
}
fn context(capture: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        100,
    )
    .unwrap()
}
fn session<'a>(
    capture: &'a HtmlCapture,
    context: &'a HtmlSemanticContext,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(
        capture,
        context,
        HtmlValueInterpreter::new(100),
        HtmlSemanticLimits::new(100000, 100, 100, 100, 100000, 100),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
}

/// Nothing was reserved, so nothing can be released: the count stays zero instead of wrapping.
#[test]
fn releasing_scratch_that_was_never_reserved_is_a_release_underflow() {
    let c = capture();
    let cx = context(&c);
    let mut s = session(&c, &cx);
    s.release_scratch(&Vec::<u64>::with_capacity(5));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(
        s.failure().map(HtmlSemanticFailure::kind),
        Some(&HtmlSemanticFailureKind::Accounting(
            HtmlAccountingFault::ReleaseUnderflow
        ))
    );
    assert_eq!(
        s.poll().unwrap_err().kind(),
        &HtmlSemanticFailureKind::Accounting(HtmlAccountingFault::ReleaseUnderflow)
    );
}

/// Releasing more than was reserved leaves the reserved count intact.
#[test]
fn releasing_more_scratch_than_was_reserved_keeps_the_reserved_count() {
    let c = capture();
    let cx = context(&c);
    let mut s = session(&c, &cx);
    let mut frames: Vec<u64> = Vec::new();
    s.reserve_scratch(&mut frames, 4).unwrap();
    let reserved = frames.capacity();
    s.release_scratch(&Vec::<u64>::with_capacity(reserved + 1));
    assert_eq!(s.usage().scratch_records(), reserved);
    assert_eq!(
        s.failure().map(HtmlSemanticFailure::kind),
        Some(&HtmlSemanticFailureKind::Accounting(
            HtmlAccountingFault::ReleaseUnderflow
        ))
    );
}

/// Two scratch vectors are live at once. Releasing one returns exactly its bytes to the enclosing
/// controlled execution, and the other stays charged until it is released too.
#[test]
fn a_controlled_execution_tracks_every_live_scratch_vector() {
    let c = capture();
    let cx = context(&c);
    let mut s = session(&c, &cx);
    let control = Control::for_tests(HtmlOptionValueEnvelope::new(
        1_000_000, 1_000_000, 1_000_000,
    ));
    let cancel = AtomicBool::new(false);
    s.control_option_value(QueryControl::for_tests(&control, &cancel));
    let mut first: Vec<u64> = Vec::new();
    let mut second: Vec<u64> = Vec::new();
    s.reserve_scratch(&mut first, 4).unwrap();
    s.reserve_scratch(&mut second, 6).unwrap();
    let first_bytes = first.capacity() * size_of::<u64>();
    let second_bytes = second.capacity() * size_of::<u64>();
    assert_eq!(control.temporary_bytes(), first_bytes + second_bytes);
    s.release_scratch(&first);
    assert_eq!(control.temporary_bytes(), second_bytes);
    s.release_scratch(&second);
    assert_eq!(control.temporary_bytes(), 0);
    assert_eq!(s.usage().scratch_records(), 0);
}
