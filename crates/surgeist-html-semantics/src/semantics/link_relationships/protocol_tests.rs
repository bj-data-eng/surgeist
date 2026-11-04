//! A rel owner that refuses a step the session took out of its protocol is a defect in this
//! crate, and it is reported as a protocol breach rather than as an unrepresentable size.
use super::*;
use surgeist_html::{construction::HtmlDocumentMode, values::HtmlValueInterpreter};

use crate::{HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlScriptingStatus};

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

/// An owner that has not interpreted yet refuses a refinement: the step is out of order.
#[test]
fn a_rel_owner_asked_out_of_order_is_a_protocol_violation_not_exhaustion() {
    let c = capture();
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        100,
    )
    .unwrap();
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        HtmlSemanticLimits::new(100000, 100, 100, 100, 100000, 100),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let mut owner = HtmlRelInterpretation::new(HtmlRelInputs::new(Some(HtmlRelRole::Link)));
    let early_refinement = owner.begin_refinement().unwrap_err();
    assert!(early_refinement.is_wrong_phase());
    let failure = session.rel_action_failure(HtmlRelActionError::Protocol(early_refinement));
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::ProtocolViolated);
}

/// Every cause a driver can provoke by taking a step out of the owner's protocol maps to the
/// protocol breach, whether the owner refuses a step or refuses to finish.
#[test]
fn every_out_of_order_rel_step_maps_to_a_protocol_violation() {
    let new = || HtmlRelInterpretation::new(HtmlRelInputs::new(Some(HtmlRelRole::Link)));
    let mut owner = new();
    let wrong_phase = owner.begin_refinement().unwrap_err();
    assert!(wrong_phase.is_wrong_phase());
    let not_ready = new().finish().unwrap_err();
    assert!(not_ready.cause().is_not_ready());
    let wrong_phase_read = new().occurrences().unwrap_err();
    for error in [wrong_phase, *not_ready.cause(), wrong_phase_read] {
        assert_eq!(
            rel_failure_kind(error),
            HtmlSemanticFailureKind::ProtocolViolated
        );
    }
}
