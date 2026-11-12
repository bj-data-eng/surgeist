//! A construction outcome that ends a parse has one reason and one first stop,
//! whichever driver and stage raised it.
//!
//! The oracle is the parser's own stop vocabulary. An outcome that has a stop of
//! its own ends the parse as that stop: the Meta CSP service and protocol
//! outcomes, and invalid associated facts, as their producers latch them. A
//! script or completion state the closed environment does not support is that
//! unsupported state. A fact or reaction the caller's capture lacks is the
//! provenance boundary, as the effect drain and the fragment driver have always
//! reported it. A fault of the parser's own is the invariant its kind names.

use super::context::tests::request as fragment_request;
use super::head::HeadError;
use super::prologue::PrologueError;
use super::{KernelError, StandaloneError};
use crate::construction::{SessionError, fault_effect_drain, take_effect_fault};
use crate::parser::control::{HtmlParseInvariant, HtmlParseStop, HtmlParseUnsupportedState};
use crate::{
    HtmlCaptureFact, HtmlDocumentParseRequest, HtmlParseFailureReason, HtmlScriptingStatus,
    HtmlSource,
};

const SOURCE: &str = "<!doctype html><p>x</p>";

fn unsupported(state: HtmlParseUnsupportedState) -> HtmlParseStop {
    HtmlParseStop::UnsupportedStandaloneState(state)
}

/// Every outcome of the document, the supplied facts or a service, with the
/// reason and the first stop it ends a parse with.
pub(super) fn outcomes() -> [(SessionError, HtmlParseFailureReason, HtmlParseStop); 9] {
    use HtmlParseFailureReason as Reason;
    use HtmlParseUnsupportedState as State;
    let protocol = crate::construction::csp_protocol_error();
    let script = unsupported(State::ScriptDocument);
    let completion = unsupported(State::Completion);
    let provenance = unsupported(State::Provenance);
    [
        (
            SessionError::CspService,
            Reason::CspProcessing,
            HtmlParseStop::CspProcessing,
        ),
        (
            SessionError::CspProtocol(protocol),
            Reason::CspProtocol(protocol),
            HtmlParseStop::CspProtocol(protocol),
        ),
        (
            SessionError::AssociatedFacts,
            Reason::Stop(HtmlParseStop::InvalidAssociatedFacts),
            HtmlParseStop::InvalidAssociatedFacts,
        ),
        (
            SessionError::ContextFactUnavailable(HtmlCaptureFact::ImageSupport),
            Reason::MissingContextFact(HtmlCaptureFact::ImageSupport),
            provenance,
        ),
        (
            SessionError::ContextReactionUnavailable,
            Reason::UnsupportedContextReactions,
            provenance,
        ),
        (
            SessionError::ScriptMetadataUnavailable,
            Reason::Stop(script),
            script,
        ),
        (
            SessionError::SvgScriptMetadataUnavailable,
            Reason::Stop(script),
            script,
        ),
        (
            SessionError::CompletionMetadataUnavailable,
            Reason::Stop(completion),
            completion,
        ),
        (
            SessionError::CompletionIncomplete,
            Reason::Stop(completion),
            completion,
        ),
    ]
}

/// Faults of the parser's own that reach a stage unlatched, with the
/// invariant their kind names (`session_fault_invariant`).
pub(super) fn faults() -> [(SessionError, HtmlParseInvariant); 4] {
    [
        (SessionError::WrongToken, HtmlParseInvariant::RegistryRow),
        (
            SessionError::PendingIntrinsicWork,
            HtmlParseInvariant::PendingFormEffects,
        ),
        (
            SessionError::AliasedPayload,
            HtmlParseInvariant::SharedAllocationAliased,
        ),
        (
            SessionError::RootMove,
            HtmlParseInvariant::ConstructionOwnerRefusal,
        ),
    ]
}

fn parse_with_fault(
    fragment: bool,
    position: usize,
    error: SessionError,
) -> crate::HtmlParseFailure {
    fault_effect_drain(position, error);
    let failure = if fragment {
        fragment_request("<p>x</p>", "div", HtmlScriptingStatus::Disabled)
            .parse()
            .err()
    } else {
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(SOURCE), super::tests::limits(100))
            .parse()
            .err()
    };
    let (unraised, _) = take_effect_fault();
    assert_eq!(unraised, None, "{error:?} at {position} was raised");
    failure.expect("the armed drain fails the parse")
}

/// How many effect drains a successful parse runs.
fn drains(fragment: bool) -> usize {
    fault_effect_drain(usize::MAX, SessionError::CompletionIncomplete);
    if fragment {
        fragment_request("<p>x</p>", "div", HtmlScriptingStatus::Disabled)
            .parse()
            .unwrap();
    } else {
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(SOURCE), super::tests::limits(100))
            .parse()
            .unwrap();
    }
    let (_, drains) = take_effect_fault();
    assert!(drains > 0, "a parse runs effect drains");
    drains
}

fn assert_every_drain_ends_as(fragment: bool) {
    let drains = drains(fragment);
    for (error, reason, stop) in outcomes() {
        for position in 0..drains {
            let failure = parse_with_fault(fragment, position, error);
            assert_eq!(
                failure.reason(),
                reason,
                "{error:?} at drain {position}, fragment: {fragment}"
            );
            assert_eq!(
                failure.first_stop(),
                Some(stop),
                "{error:?} at drain {position}, fragment: {fragment}"
            );
        }
    }
}

/// The ordinary driver's drains run under the prologue stage first and the
/// head stage after it; neither may report a different stop for an outcome.
#[test]
fn an_outcome_raised_by_effect_work_ends_an_ordinary_parse_as_its_reason_names() {
    assert_every_drain_ends_as(false);
}

#[test]
fn an_outcome_raised_by_effect_work_ends_a_fragment_parse_as_its_reason_names() {
    assert_every_drain_ends_as(true);
}

fn stage_errors(error: SessionError) -> [StandaloneError; 3] {
    [
        StandaloneError::Head(HeadError::Kernel(KernelError::Session(error))),
        StandaloneError::Prologue(PrologueError::Kernel(KernelError::Session(error))),
        StandaloneError::Head(HeadError::Prologue(PrologueError::Kernel(
            KernelError::Session(error),
        ))),
    ]
}

/// Whichever stage error carries an outcome, the ordinary driver latches the
/// stop for it that the parse report would name.
#[test]
fn an_outcome_in_any_ordinary_stage_error_latches_its_stop() {
    for (error, _, stop) in outcomes() {
        for stage in stage_errors(error) {
            let request = HtmlDocumentParseRequest::offline(
                HtmlSource::utf8("<p>x"),
                super::tests::limits(100),
            );
            let mut driver = super::entry::initialized_driver(&request);
            driver.retain_terminal_failure(&Err::<(), _>(stage));
            assert_eq!(driver.control.stop(), Some(stop), "{error:?}");
            driver.release().unwrap();
        }
    }
}

/// The stage that completes a document reports the same state as the stage
/// that raised the session outcome for it.
#[test]
fn an_incomplete_completion_stage_error_latches_the_completion_stop() {
    let request =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), super::tests::limits(100));
    let mut driver = super::entry::initialized_driver(&request);
    driver.retain_terminal_failure(&Err::<(), _>(StandaloneError::Head(
        HeadError::IncompleteCompletion,
    )));
    assert_eq!(
        driver.control.stop(),
        Some(unsupported(HtmlParseUnsupportedState::Completion))
    );
    driver.release().unwrap();
}

/// A fault of the parser's own is an invariant, as its reason says, not an
/// unsupported document.
#[test]
fn a_parser_fault_in_any_ordinary_stage_error_latches_its_invariant() {
    for (error, invariant) in faults() {
        for stage in stage_errors(error) {
            let request = HtmlDocumentParseRequest::offline(
                HtmlSource::utf8("<p>x"),
                super::tests::limits(100),
            );
            let mut driver = super::entry::initialized_driver(&request);
            driver.retain_terminal_failure(&Err::<(), _>(stage));
            assert_eq!(
                driver.control.stop(),
                Some(HtmlParseStop::InternalInvariant(invariant)),
                "{error:?}"
            );
            driver.release().unwrap();
        }
    }
}

/// Every session error. The match below names every variant, so adding one
/// stops compiling until it is listed here, and the classification test then
/// holds it to the same rules.
pub(super) fn every_session_error() -> Vec<SessionError> {
    let protocol = crate::construction::csp_protocol_error();
    let all = vec![
        SessionError::NonceMetadataUnavailable,
        SessionError::CspService,
        SessionError::CspProtocol(protocol),
        SessionError::Stop(HtmlParseStop::Cancelled),
        SessionError::ForeignNode,
        SessionError::RootMove,
        SessionError::AssociationMove,
        SessionError::NotContainer,
        SessionError::TemplateContentsRequired,
        SessionError::WrongSibling,
        SessionError::Cycle,
        SessionError::Consumed,
        SessionError::WrongToken,
        SessionError::DuplicateAttribute,
        SessionError::AliasedPayload,
        SessionError::TemplateInsertionStateUnavailable,
        SessionError::NotDocument,
        SessionError::OriginSplitUnavailable,
        SessionError::ScriptMetadataUnavailable,
        SessionError::SvgScriptMetadataUnavailable,
        SessionError::ImageFormMetadataUnavailable,
        SessionError::AssociatedFacts,
        SessionError::FormAssociationMetadataUnavailable,
        SessionError::TextareaMetadataUnavailable,
        SessionError::OutputMetadataUnavailable,
        SessionError::InputMetadataUnavailable,
        SessionError::SelectMetadataUnavailable,
        SessionError::NodeDocumentUnavailable,
        SessionError::TemplateMetadataUnavailable,
        SessionError::ProcessingInstructionMapUnavailable,
        SessionError::PendingIntrinsicWork,
        SessionError::CompletionMetadataUnavailable,
        SessionError::CompletionIncomplete,
        SessionError::ContextUnavailable,
        SessionError::ContextReactionUnavailable,
        SessionError::ContextFactUnavailable(HtmlCaptureFact::ImageSupport),
        SessionError::CapturedDocument,
    ];
    for error in &all {
        match error {
            SessionError::NonceMetadataUnavailable
            | SessionError::CspService
            | SessionError::CspProtocol(_)
            | SessionError::Stop(_)
            | SessionError::ForeignNode
            | SessionError::RootMove
            | SessionError::AssociationMove
            | SessionError::NotContainer
            | SessionError::TemplateContentsRequired
            | SessionError::WrongSibling
            | SessionError::Cycle
            | SessionError::Consumed
            | SessionError::WrongToken
            | SessionError::DuplicateAttribute
            | SessionError::AliasedPayload
            | SessionError::TemplateInsertionStateUnavailable
            | SessionError::NotDocument
            | SessionError::OriginSplitUnavailable
            | SessionError::ScriptMetadataUnavailable
            | SessionError::SvgScriptMetadataUnavailable
            | SessionError::ImageFormMetadataUnavailable
            | SessionError::AssociatedFacts
            | SessionError::FormAssociationMetadataUnavailable
            | SessionError::TextareaMetadataUnavailable
            | SessionError::OutputMetadataUnavailable
            | SessionError::InputMetadataUnavailable
            | SessionError::SelectMetadataUnavailable
            | SessionError::NodeDocumentUnavailable
            | SessionError::TemplateMetadataUnavailable
            | SessionError::ProcessingInstructionMapUnavailable
            | SessionError::PendingIntrinsicWork
            | SessionError::CompletionMetadataUnavailable
            | SessionError::CompletionIncomplete
            | SessionError::ContextUnavailable
            | SessionError::ContextReactionUnavailable
            | SessionError::ContextFactUnavailable(_)
            | SessionError::CapturedDocument => {}
        }
    }
    all
}

/// The stage states the closed environment cannot continue from, with the
/// unsupported state each ends a parse as: a script that asks the parser to
/// pause, a blocking script it would have to wait for, and a head owner that
/// holds no proof of the construction capability.
pub(super) fn stage_states() -> [(fn() -> HeadError, HtmlParseUnsupportedState); 3] {
    [
        (
            || HeadError::UnsupportedScriptPause,
            HtmlParseUnsupportedState::ScriptPause,
        ),
        (
            || HeadError::UnsupportedPendingScript,
            HtmlParseUnsupportedState::PendingScript,
        ),
        (
            || HeadError::ConstructionCapabilityUnavailable,
            HtmlParseUnsupportedState::Provenance,
        ),
    ]
}

/// The ordinary driver ends on each of these states as the state it names.
#[test]
fn an_unsupported_stage_state_latches_its_stop_in_the_ordinary_driver() {
    for (error, state) in stage_states() {
        let request =
            HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), super::tests::limits(100));
        let mut driver = super::entry::initialized_driver(&request);
        driver.retain_terminal_failure(&Err::<(), _>(StandaloneError::Head(error())));
        assert_eq!(driver.control.stop(), Some(unsupported(state)));
        driver.release().unwrap();
    }
}

/// A named constructor of a stage error.
pub(super) type StructureCase = (&'static str, fn() -> HeadError);

/// Errors of the tree builder's own structures that no outcome of the document
/// or of supplied facts names but that input may reach: the stack of open
/// elements or the tokenizer handoff is in a shape the standalone environment
/// does not continue from. They are the unsupported nesting state, in both
/// drivers and in the failure's reason.
pub(super) fn structure_errors() -> Vec<StructureCase> {
    vec![
        ("EmptyStack", || HeadError::Kernel(KernelError::EmptyStack)),
        ("NotElement", || HeadError::Kernel(KernelError::NotElement)),
        ("FirstOpenElementNotHtml", || {
            HeadError::Kernel(KernelError::FirstOpenElementNotHtml)
        }),
        ("DuplicateOpenElement", || {
            HeadError::Kernel(KernelError::DuplicateOpenElement)
        }),
        ("ForeignSource", || {
            HeadError::Handoff(crate::parser::tokenizer::TokenizerHandoffError::ForeignSource)
        }),
        ("NotData", || {
            HeadError::Handoff(crate::parser::tokenizer::TokenizerHandoffError::NotData)
        }),
        ("PendingEvents", || {
            HeadError::Handoff(crate::parser::tokenizer::TokenizerHandoffError::PendingEvents)
        }),
        ("ActiveDraft", || {
            HeadError::Handoff(crate::parser::tokenizer::TokenizerHandoffError::ActiveDraft)
        }),
    ]
}

#[test]
fn a_structure_error_latches_the_unsupported_nesting_stop_in_the_ordinary_driver() {
    for (name, error) in structure_errors() {
        let request =
            HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), super::tests::limits(100));
        let mut driver = super::entry::initialized_driver(&request);
        driver.retain_terminal_failure(&Err::<(), _>(StandaloneError::Head(error())));
        assert_eq!(
            driver.control.stop(),
            Some(unsupported(HtmlParseUnsupportedState::InvalidNesting)),
            "{name}"
        );
        driver.release().unwrap();
    }
}

/// A document initialization or execution profile the standalone environment
/// was given no proof of is refused at the provenance boundary, at the stage
/// and at the seal alike. The reason is checked beside the reason mappings.
#[test]
fn an_unsupported_request_shape_latches_the_provenance_stop() {
    for (name, error) in [
        (
            "UnsupportedDocumentInitialization",
            (|| StandaloneError::UnsupportedDocumentInitialization) as fn() -> StandaloneError,
        ),
        ("UnsupportedExecutionProfile", || {
            StandaloneError::UnsupportedExecutionProfile
        }),
    ] {
        let request =
            HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), super::tests::limits(100));
        let mut driver = super::entry::initialized_driver(&request);
        driver.retain_terminal_failure(&Err::<(), _>(error()));
        assert_eq!(
            driver.control.stop(),
            Some(unsupported(HtmlParseUnsupportedState::Provenance)),
            "{name}"
        );
        driver.release().unwrap();
    }
}
