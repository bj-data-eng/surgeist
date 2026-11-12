//! The parser-session failure vocabulary, its classification, and the latches that make a failure the controller's first stop.

use super::*;
use surgeist_html::{
    accounting::HtmlAccountingFault,
    construction::{HtmlConstructionCause, HtmlConstructionError},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionError {
    NonceMetadataUnavailable,
    CspService,
    CspProtocol(surgeist_html::security::csp::HtmlCspProtocolError),
    Stop(HtmlParseStop),
    ForeignNode,
    RootMove,
    AssociationMove,
    NotContainer,
    TemplateContentsRequired,
    WrongSibling,
    Cycle,
    Consumed,
    WrongToken,
    DuplicateAttribute,
    AliasedPayload,
    TemplateInsertionStateUnavailable,
    NotDocument,
    OriginSplitUnavailable,
    AttributeStorageUnavailable,
    ScriptMetadataUnavailable,
    SvgScriptMetadataUnavailable,
    ImageFormMetadataUnavailable,
    AssociatedFacts,
    FormAssociationMetadataUnavailable,
    TextareaMetadataUnavailable,
    OutputMetadataUnavailable,
    InputMetadataUnavailable,
    SelectMetadataUnavailable,
    NodeDocumentUnavailable,
    TemplateMetadataUnavailable,
    ProcessingInstructionMapUnavailable,
    PendingIntrinsicWork,
    CompletionMetadataUnavailable,
    CompletionIncomplete,
    ContextUnavailable,
    ContextReactionUnavailable,
    ContextFactUnavailable(crate::HtmlCaptureFact),
    CapturedDocument,
}

/// Pinned element-placement guards; generic storage moves remain independent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ElementInsertionIgnored {
    ParentAlreadyPresent,
    HostIncludingInclusiveAncestor,
    DocumentAlreadyHasElement,
}

impl SessionError {
    /// An impossible registry state with no controller in reach.
    pub(crate) fn registry_row() -> Self {
        Self::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        ))
    }
}

impl From<HtmlParseStop> for SessionError {
    fn from(value: HtmlParseStop) -> Self {
        Self::Stop(value)
    }
}

/// HTML's construction owner answered a step with an action or observation
/// that the parser's sequencing does not expect, latched as the controller's
/// first stop.
pub(super) fn latch_owner_refusal(control: &mut ParseControl) -> SessionError {
    SessionError::Stop(
        control
            .fail_invariant(crate::parser::control::HtmlParseInvariant::ConstructionOwnerRefusal),
    )
}

/// An impossible registry state, latched as the controller's first stop.
pub(super) fn latch_registry_row(control: &mut ParseControl) -> SessionError {
    SessionError::Stop(
        control.fail_invariant(crate::parser::control::HtmlParseInvariant::RegistryRow),
    )
}

/// How a parse ends on a session error that no earlier stop has settled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionClass {
    /// The error is an outcome of the document, of the facts the caller
    /// supplied or of a service. The parse ends as this stop, whichever driver
    /// and stage met the error, and the failure's reason names the same state.
    Outcome(HtmlParseStop),
    /// The error names a fault of the parser's own, met where it cannot be a
    /// document outcome. The parse ends as the invariant of its kind.
    Fault(crate::parser::control::HtmlParseInvariant),
}

impl SessionClass {
    /// An outcome that ends the parse as an unsupported state of the closed
    /// standalone environment.
    pub(crate) const fn unsupported(state: HtmlParseUnsupportedState) -> Self {
        Self::Outcome(HtmlParseStop::UnsupportedStandaloneState(state))
    }
    /// Latches the stop for this class on `control`, unless an earlier stop is
    /// already the first. Returns the first stop either way.
    pub(crate) fn latch(self, control: &mut ParseControl) -> HtmlParseStop {
        match self {
            Self::Outcome(stop) => control.stop_outcome(stop),
            Self::Fault(invariant) => control.fail_invariant(invariant),
        }
    }
}

/// The class of a session error, by the kind of state it reports. Every
/// variant is listed, so a new one must be classified deliberately.
///
/// Outcomes, with the stop each ends a parse as:
/// - A stop is already latched by whoever raised it, and is its own.
/// - The Meta CSP service and protocol outcomes are the stops their processing
///   latches when it raises them. A processor that breaks the accounting of the
///   service it was given is a failed processing.
/// - Invalid associated facts are the associated-facts stop, as the creation
///   and Meta CSP producers latch it.
/// - Script and SVG script metadata that effect work cannot find is the
///   unsupported script document; completion metadata that is absent or
///   incomplete is the unsupported completion. The standalone environment does
///   not run scripts or document completion.
/// - A fact or reaction the caller's capture lacks is the provenance boundary:
///   the capture is the only source of the fact, so the parse cannot continue
///   from provenance it was not given.
///
/// Faults, with the invariant of their kind:
/// - An intrinsic operation that is still owed is the pending-effects stop.
/// - A uniqueness check that failed is the aliased-allocation stop.
/// - A placement the tree refuses, or a duplicated attribute, is the
///   construction owner declining a request the parser's sequencing
///   guarantees valid.
/// - A retained row that is missing, of another kind, unjoined or out of
///   place is a registry row. This is the whole of the rest of this side: every
///   remaining variant reports a lookup or a bookkeeping check that fails on a
///   handle, index or count the parser created itself (an arena node, a
///   payload row of the wrong kind, storage already taken, a reference count
///   other than one, a parent walk past its bound, a placement or association
///   the construction owner refuses to a request the parser's sequencing
///   guarantees valid). None of them reads what the caller supplied. Whatever
///   the caller's capture does not establish is
///   `ContextFactUnavailable(fact)`, a processor that breaks the accounting of
///   the service it was given is `CspService`, and the facts a request
///   supplies are `AssociatedFacts`. A raise site that would read caller
///   input must not return a fault variant.
pub(crate) fn session_class(error: &SessionError) -> SessionClass {
    use crate::parser::control::HtmlParseInvariant;
    match error {
        SessionError::Stop(stop) => SessionClass::Outcome(*stop),
        SessionError::CspService => SessionClass::Outcome(HtmlParseStop::CspProcessing),
        SessionError::CspProtocol(protocol) => {
            SessionClass::Outcome(HtmlParseStop::CspProtocol(*protocol))
        }
        SessionError::AssociatedFacts => {
            SessionClass::Outcome(HtmlParseStop::InvalidAssociatedFacts)
        }
        SessionError::ScriptMetadataUnavailable | SessionError::SvgScriptMetadataUnavailable => {
            SessionClass::unsupported(HtmlParseUnsupportedState::ScriptDocument)
        }
        SessionError::CompletionMetadataUnavailable | SessionError::CompletionIncomplete => {
            SessionClass::unsupported(HtmlParseUnsupportedState::Completion)
        }
        SessionError::ContextFactUnavailable(_) | SessionError::ContextReactionUnavailable => {
            SessionClass::unsupported(HtmlParseUnsupportedState::Provenance)
        }
        SessionError::PendingIntrinsicWork => {
            SessionClass::Fault(HtmlParseInvariant::PendingFormEffects)
        }
        SessionError::AliasedPayload => {
            SessionClass::Fault(HtmlParseInvariant::SharedAllocationAliased)
        }
        SessionError::RootMove
        | SessionError::AssociationMove
        | SessionError::NotContainer
        | SessionError::TemplateContentsRequired
        | SessionError::WrongSibling
        | SessionError::Cycle
        | SessionError::DuplicateAttribute => {
            SessionClass::Fault(HtmlParseInvariant::ConstructionOwnerRefusal)
        }
        SessionError::NonceMetadataUnavailable
        | SessionError::ForeignNode
        | SessionError::Consumed
        | SessionError::WrongToken
        | SessionError::TemplateInsertionStateUnavailable
        | SessionError::NotDocument
        | SessionError::OriginSplitUnavailable
        | SessionError::AttributeStorageUnavailable
        | SessionError::ImageFormMetadataUnavailable
        | SessionError::FormAssociationMetadataUnavailable
        | SessionError::TextareaMetadataUnavailable
        | SessionError::OutputMetadataUnavailable
        | SessionError::InputMetadataUnavailable
        | SessionError::SelectMetadataUnavailable
        | SessionError::NodeDocumentUnavailable
        | SessionError::TemplateMetadataUnavailable
        | SessionError::ProcessingInstructionMapUnavailable
        | SessionError::ContextUnavailable
        | SessionError::CapturedDocument => SessionClass::Fault(HtmlParseInvariant::RegistryRow),
    }
}

/// The invariant that a non-stop error names when it is met where it cannot be
/// a document outcome; an outcome has none, and the caller passes it through
/// unchanged.
pub(super) fn session_fault_invariant(
    error: &SessionError,
) -> Option<crate::parser::control::HtmlParseInvariant> {
    match session_class(error) {
        SessionClass::Fault(invariant) => Some(invariant),
        SessionClass::Outcome(_) => None,
    }
}

/// Ends the session on `error` where it is met: latches the stop its class
/// names now. If that stop is the first one, an outcome is returned as itself,
/// which keeps its own reason, and a fault as the invariant it latched. If
/// another stop was already first, the first stop is returned for either, so
/// the error's reason and the failure's first stop cannot disagree.
pub(super) fn end_session_on(control: &mut ParseControl, error: SessionError) -> SessionError {
    let class = session_class(&error);
    let first = class.latch(control);
    match class {
        SessionClass::Outcome(own) if own == first => error,
        SessionClass::Outcome(_) | SessionClass::Fault(_) => SessionError::Stop(first),
    }
}

/// The result of a non-stop error that effect work met, classified by
/// [`session_fault_invariant`]. An outcome the driver classifies itself passes
/// through unchanged and latches nothing.
pub(super) fn latch_effect_fault(control: &mut ParseControl, error: SessionError) -> SessionError {
    match session_fault_invariant(&error) {
        Some(invariant) => SessionError::Stop(control.fail_invariant(invariant)),
        None => error,
    }
}

/// What a non-stop error that ended a creation after its association metadata
/// was registered becomes. A fault of the creation's own is a stop labelled by
/// what it reports, as in effect work.
///
/// An outcome the driver classifies itself passes through unchanged, as the
/// unavailable image-support fact does: the creation runs image update,
/// associated-fact (`start_img_update` and `set_hyperlink`), nonce and
/// context-fact work, whose outcomes come from the facts the caller supplied
/// and so must keep the classification the driver gives them.
///
/// The outcomes of script, SVG-wrapper and completion work are different. The
/// callees of the creation (`allocate_ordered_payload`,
/// `allocate_html_payload_prepared`, `mark_document_parser_style` and
/// `normalize_radio`, and what they call for attribute, input, select,
/// details, nonce, meta, dialog, style and associated-state set-up) were read
/// for calls into script preparation, SVG-wrapper or completion code, and
/// none was found; the form-effect drain they reach only steps the id-change
/// stage of the creation's own element. Such an outcome met here is a creation
/// that returned something that belongs to another stage, which is what the
/// creation label names.
pub(super) fn latch_creation_fault(
    control: &mut ParseControl,
    error: SessionError,
) -> SessionError {
    use crate::parser::control::HtmlParseInvariant;
    if let Some(invariant) = session_fault_invariant(&error) {
        return SessionError::Stop(control.fail_invariant(invariant));
    }
    match error {
        SessionError::ScriptMetadataUnavailable
        | SessionError::SvgScriptMetadataUnavailable
        | SessionError::CompletionMetadataUnavailable
        | SessionError::CompletionIncomplete => SessionError::Stop(
            control.fail_invariant(HtmlParseInvariant::IncompleteFormAssociationCreation),
        ),
        other => other,
    }
}

/// The position after `cursor`. Every caller reaches it only after the work
/// list yielded an entry at `cursor`, so `cursor` is below an in-memory length
/// (at most `isize::MAX`) and the successor cannot overflow.
pub(super) fn next_cursor(cursor: usize) -> usize {
    cursor.saturating_add(1)
}

/// The successor of a record count or position, or the registry-row stop when
/// it would overflow.
pub(super) fn successor(control: &mut ParseControl, value: usize) -> Result<usize, SessionError> {
    value
        .checked_add(1)
        .ok_or_else(|| latch_registry_row(control))
}

/// How many of the flags are set; at most `N`.
pub(super) fn count_set<const N: usize>(flags: [bool; N]) -> usize {
    flags.into_iter().filter(|flag| *flag).count()
}

/// How a cause of the owner's error is reported to the session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Disposition {
    /// A document-shaped error that `session_class` classifies; nothing is
    /// latched here.
    Session(SessionError),
    /// The class a stage error of the same kind would carry, latched through it.
    Class(SessionClass),
    /// A count that cannot be represented.
    Representation,
    /// A failure of the ledger scope that is not a refused limit, latched as the parse's
    /// accounting stop.
    Accounting(HtmlAccountingFault),
    /// An allocation whose layout is unknown here.
    AllocationFailed,
}

/// Maps every cause of the owner's error to its report. The cause enum is exhaustive, so a cause
/// that the owner adds needs a row here to compile.
fn disposition(cause: HtmlConstructionCause) -> Disposition {
    use crate::parser::control::HtmlParseInvariant;
    use Disposition::{AllocationFailed, Class, Representation, Session};
    use HtmlConstructionCause as Cause;
    match cause {
        Cause::ForeignBuilder | Cause::UnknownNode => Session(SessionError::ForeignNode),
        Cause::RootMove => Session(SessionError::RootMove),
        Cause::AssociationMove => Session(SessionError::AssociationMove),
        Cause::ParentCannotHaveChildren => Session(SessionError::NotContainer),
        Cause::TemplateContentsRequired => Session(SessionError::TemplateContentsRequired),
        Cause::WrongSibling | Cause::ParentChildConflict => Session(SessionError::WrongSibling),
        Cause::Cycle => Session(SessionError::Cycle),
        Cause::SharedAllocationAliased => Class(SessionClass::Fault(
            HtmlParseInvariant::SharedAllocationAliased,
        )),
        Cause::RepresentationExhausted => Representation,
        Cause::CapacityMismatch | Cause::LimitExceeded { .. } => {
            Class(SessionClass::Fault(HtmlParseInvariant::CapacityMismatch))
        }
        Cause::ReleaseUnderflow => Class(SessionClass::Fault(HtmlParseInvariant::ReleaseUnderflow)),
        Cause::Accounting(fault) => Disposition::Accounting(fault),
        // `layout_bytes: 0` means the layout is unknown here, not that none was
        // requested. Every bounded producer of this error reports it with its
        // real layout before it gets this far: the record and child reserves
        // in the record operations of HTML's construction storage, the attribute-list
        // reserves in the attribute admission, and the text reserves in
        // the text admission. This is only a failure a caller without a layout
        // still has to report.
        Cause::AllocationFailed => AllocationFailed,
        Cause::ShadowHostMustBeElement
        | Cause::ShadowAlreadyAttached
        | Cause::WrongNodeKind
        | Cause::AssociationAlreadyOwned
        | Cause::InvalidTemplateAssociation
        | Cause::DuplicateAttribute { .. }
        | Cause::InvalidShadowAssociation
        | Cause::InvalidContentsReference
        | Cause::InvalidOutputRoot
        | Cause::OperandMissing
        | Cause::InvalidTextReference
        | Cause::NotPublished
        | Cause::IndexUnavailable => Class(SessionClass::Fault(
            HtmlParseInvariant::ConstructionOwnerRefusal,
        )),
    }
}

/// The session error for a refusal from HTML's construction owner, with every
/// cause of its error reported by what it violates.
///
/// There are two tables, and the second is built on the first.
/// [`session_class`] classifies every session error; [`disposition`] maps the
/// construction owner's causes, which are not session errors until this
/// function converts them. A document-shaped refusal converts to the
/// [`SessionError`] that `session_class` already classifies, and nothing is
/// latched here. Every other cause converts to the [`SessionClass`] that a
/// stage error would carry and is latched through [`SessionClass::latch`], so
/// the stop and the failure's reason derive from the same class (`StopCause`).
/// Three stops are latched directly, as their controller owners latch them: a
/// count that cannot be represented, an allocation that failed and a failure of
/// the ledger scope. A test checks that no document-shaped disposition
/// disagrees with `session_class`.
///
/// - Handle, placement and cycle refusals are errors about the document's own
///   shape; their callers classify them and nothing is latched here.
/// - A payload that was aliased and a count that cannot be represented keep
///   their own stops.
/// - A ticket, layout or limit that disagrees with what the controller
///   charged, and a release beyond what was held, are accounting faults. The
///   parse controller admits every resource before the owner is asked, so a
///   limit the owner reports is the owner's accounting disagreeing with it.
/// - A failure of the ledger scope that is not a refused limit stops the parse
///   with the same accounting fault as every other ledger fault of the parser.
/// - An allocation that failed is the environment refusing, a reached stop
///   that never names an invariant. The sites that know the layout classify
///   it themselves first, so none is known here.
/// - Every other refusal is the owner declining a request that the parser's
///   sequencing guarantees valid.
///
/// The owner's cause enum is exhaustive, so [`disposition`] has no fallback: a
/// cause that the owner adds does not compile here until it has a row, and
/// cannot become a parser fault or a document outcome by default. The tests
/// cover the report of every cause, and the errors the owner produces for the
/// document-shaped, refusal and limit causes.
pub(super) fn construction_error(
    control: &mut ParseControl,
    error: HtmlConstructionError,
) -> SessionError {
    report(control, disposition(error.cause()))
}

/// Reports a disposition to the session, latching a stop where it names one.
fn report(control: &mut ParseControl, disposition: Disposition) -> SessionError {
    match disposition {
        Disposition::Session(error) => error,
        Disposition::Class(class) => SessionError::Stop(class.latch(control)),
        Disposition::Accounting(fault) => SessionError::Stop(control.stop_accounting(fault)),
        Disposition::Representation => {
            SessionError::Stop(control.stop_representation(HtmlParseResource::ManagedCapacityBytes))
        }
        Disposition::AllocationFailed => {
            SessionError::Stop(control.construction_allocation_failed(0))
        }
    }
}

#[cfg(test)]
mod tests;
