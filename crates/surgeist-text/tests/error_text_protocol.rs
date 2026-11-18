#![forbid(unsafe_code)]

//! Messages of `ProtocolError` and the three enums it wraps or shares its
//! vocabulary with.
//!
//! Every variant is public, so each table constructs all of them. A row states
//! the documented meaning of one variant; the `index` functions are exhaustive
//! matches without a fallback arm, so a new variant does not compile until it
//! has a row. Three `ProtocolError` variants wrap another error and return it
//! from `source()` without repeating its text. The handle kinds, line-context
//! fields and conditional purposes that appear inside messages have no public
//! `Display`; their names are checked through the messages that print them.

use surgeist_text::{
    Affinity, ConditionalPurpose, InlineBoxEdgeError, InlineBoxFragmentationError, PhaseTwoContext,
    ProtocolContextField, ProtocolError, ProtocolHandleKind, Range, WrappingScopeQueryError,
};

mod error_support;
use error_support::{check_table, check_wrapper};

fn index_protocol(error: &ProtocolError) -> usize {
    use ProtocolError as E;
    match error {
        E::InvalidSemanticWindow => 0,
        E::InvalidSemanticAtomic { .. } => 1,
        E::InlineBoxFragmentation(_) => 2,
        E::InlineBoxEdge(_) => 3,
        E::InvalidSourceSegment { .. } => 4,
        E::MismatchedSourceSegment { .. } => 5,
        E::WrappingScopeQuery { .. } => 6,
        E::InvalidMeasurementFlow => 7,
        E::InvalidLineDirectionQuery => 8,
        E::InvalidWordPortion => 9,
        E::MeasurementNotPlaceable => 10,
        E::NonFormattedForcedLine => 11,
        E::NonFormattedInFlowContent => 12,
        E::NoFormattedLineGeometry => 13,
        E::InvalidPhrasePortion => 14,
        E::InvalidConditionalStage => 15,
        E::MissingPhaseTwoContext(_) => 16,
        E::OutsideFinalizedLine => 17,
        E::InvalidOriginSeam { .. } => 18,
        E::ForeignPreparation { .. } => 19,
        E::ForeignSelectedLine => 20,
        E::ForeignFinalizedLine { .. } => 21,
        E::InvalidOriginRange { .. } => 22,
        E::OriginTextMismatch { .. } => 23,
        E::IncompleteOriginCoverage { .. } => 24,
        E::MissingConnection { .. } => 25,
        E::InvalidConnection { .. } => 26,
        E::InvalidControl { .. } => 27,
        E::InvalidBoundaryPolicy { .. } => 28,
        E::DuplicateReplacement { .. } => 29,
        E::InvalidReplacement { .. } => 30,
        E::InvalidBoundaryRange { .. } => 31,
        E::ParticipantMismatch { .. } => 32,
        E::MissingAtomic { .. } => 33,
        E::DuplicateAtomic { .. } => 34,
        E::IllegalOpportunity { .. } => 35,
        E::InvalidConditionalQuery { .. } => 36,
        E::InvalidConditionalSelection { .. } => 37,
        E::InvalidConditionalContext { .. } => 38,
        E::InvalidConditionalReceipt { .. } => 39,
        E::RetainedConditionalPrefix { .. } => 40,
        E::UnsupportedWrappingAnalysis => 41,
        E::MissingWrappingPolicy { .. } => 42,
        E::ForeignWrappingRelation { .. } => 43,
        E::DuplicateWrappingPolicy { .. } => 44,
        E::ConflictingWrappingPolicy { .. } => 45,
        E::ForeignAlternative { .. } => 46,
        E::UnsupportedContext { .. } => 47,
        E::MismatchedContext { .. } => 48,
        E::InvalidAtomicGeometry { .. } => 49,
        E::DuplicateSpacingSlot { .. } => 50,
        E::MissingSpacingSlot { .. } => 51,
        E::IneligibleSpacingSlot { .. } => 52,
        E::UnexpectedAtomic { .. } => 53,
        E::MissingAlternative { .. } => 54,
        E::InvalidAnchorOrder { .. } => 55,
        E::MissingControl { .. } => 56,
        E::MissingControlBoundaryGeometry { .. } => 57,
        E::MissingBoundaryContext { .. } => 58,
        E::DuplicateBoundaryContext { .. } => 59,
        E::UnexpectedBoundaryContext { .. } => 60,
        E::DuplicateControl { .. } => 61,
        E::UnexpectedControl { .. } => 62,
        E::InvalidControlGeometry { .. } => 63,
        E::MissingLineMetrics => 64,
        E::InvalidLineStrut => 65,
        E::UnsupportedStandaloneAnalysis => 66,
        E::ForeignPlacement => 67,
        E::OutsidePlacement { .. } => 68,
        E::AmbiguousSourcePosition { .. } => 69,
        E::InvalidPlacedPosition { .. } => 70,
        E::EmptyPlacement => 71,
        E::IncompleteLineCoverage { .. } => 72,
        E::DuplicateLine { .. } => 73,
        E::InvalidFragment { .. } => 74,
        E::AtomicPlacementMismatch { .. } => 75,
    }
}

#[test]
fn protocol_errors_state_their_cause() {
    use ProtocolError as E;
    check_table(
        76,
        vec![
            (
                E::InvalidSemanticWindow,
                "the request is invalid for its issued semantic window",
            ),
            (
                E::InvalidSemanticAtomic { occurrence: 3 },
                "atomic occurrence 3 does not match the kind issued for the semantic window",
            ),
            (
                E::InlineBoxFragmentation(InlineBoxFragmentationError::InvalidIncidence),
                "the inline-box fragment request is invalid",
            ),
            (
                E::InlineBoxEdge(InlineBoxEdgeError::MissingCapture),
                "the inline-box edge operation is invalid",
            ),
            (
                E::InvalidSourceSegment { segment: 2 },
                "source segment 2 is invalid for this preparation",
            ),
            (
                E::MismatchedSourceSegment {
                    expected: 1,
                    actual: 2,
                },
                "source segments 1 and 2 do not match",
            ),
            (
                E::WrappingScopeQuery {
                    reason: WrappingScopeQueryError::NonInlineScope,
                },
                "the wrapping scope query is invalid",
            ),
            (E::InvalidMeasurementFlow, "the measurement flow is invalid"),
            (
                E::InvalidLineDirectionQuery,
                "the line direction query or the bidi body context behind it is invalid",
            ),
            (
                E::InvalidWordPortion,
                "the word portion is invalid for this preparation",
            ),
            (
                E::MeasurementNotPlaceable,
                "a measurement termination cannot be placed",
            ),
            (
                E::NonFormattedForcedLine,
                "a forced line ending always produces a formatted line",
            ),
            (
                E::NonFormattedInFlowContent,
                "retained in-flow text, atomic or boundary content cannot be suppressed",
            ),
            (
                E::NoFormattedLineGeometry,
                "complete semantic coverage has no formatted line to lend geometry",
            ),
            (
                E::InvalidPhrasePortion,
                "the phrase portion request is invalid",
            ),
            (
                E::InvalidConditionalStage,
                "the conditional stage is invalid for this query",
            ),
            (
                E::MissingPhaseTwoContext(PhaseTwoContext::FirstFormattedLine),
                "context required by CSS Text Phase II trimming and positioning is missing",
            ),
            (
                E::OutsideFinalizedLine,
                "the original seam is not represented by this finalized line",
            ),
            (
                E::InvalidOriginSeam { position: 9 },
                "original position 9 is outside the origin's range",
            ),
            (
                E::ForeignPreparation {
                    kind: ProtocolHandleKind::Control,
                    occurrence: 4,
                },
                "the control handle 4 was issued by a different preparation",
            ),
            (
                E::ForeignSelectedLine,
                "a handle belongs to a different selected line or preparation",
            ),
            (
                E::ForeignFinalizedLine { line: 2 },
                "the finalized line 2 belongs to a different preparation or placement",
            ),
            (
                E::InvalidOriginRange {
                    origin: 1,
                    local: Range::new(0, 3),
                    aggregate: Range::new(4, 9),
                },
                "the range of origin 1 is invalid",
            ),
            (
                E::OriginTextMismatch {
                    origin: 1,
                    local: 2,
                    aggregate: 12,
                },
                "the text of origin 1 differs between its local offset 2 and aggregate offset 12",
            ),
            (
                E::IncompleteOriginCoverage {
                    expected: 5,
                    actual: 7,
                },
                "the origins leave the source incompletely covered: position 5 was expected but 7 was found",
            ),
            (
                E::MissingConnection {
                    before: 1,
                    after: 2,
                },
                "origins 1 and 2 have no connection",
            ),
            (
                E::InvalidConnection {
                    connection: 1,
                    at: 6,
                },
                "connection 1 at 6 is invalid",
            ),
            (
                E::InvalidControl { control: 2, at: 6 },
                "control 2 at 6 has an invalid source position",
            ),
            (
                E::InvalidBoundaryPolicy { policy: 2, at: 6 },
                "boundary policy 2 at 6 is invalid",
            ),
            (
                E::DuplicateReplacement {
                    alternative: 3,
                    prior: 1,
                },
                "alternative 3 claims the defining occurrence that alternative 1 already claims",
            ),
            (
                E::InvalidReplacement {
                    alternative: 3,
                    source: Range::new(2, 4),
                },
                "replacement 3 does not describe a valid source range 2..4 ending at its insertion anchor",
            ),
            (
                E::InvalidBoundaryRange {
                    start: 2,
                    end: 9,
                    participants: 5,
                },
                "boundaries 2 to 9 are not a valid range in this preparation of 5 participants",
            ),
            (
                E::ParticipantMismatch { position: 4 },
                "position 4 does not match the participants issued for the request",
            ),
            (
                E::MissingAtomic { occurrence: 3 },
                "no allocation was supplied for atomic occurrence 3",
            ),
            (
                E::DuplicateAtomic { occurrence: 3 },
                "atomic occurrence 3 appears more than once",
            ),
            (
                E::IllegalOpportunity { opportunity: 7 },
                "opportunity 7 is not a legal cut for its segment or request",
            ),
            (
                E::InvalidConditionalQuery {
                    start: 2,
                    ceiling: 8,
                },
                "the conditional query from 2 with ceiling 8 is invalid",
            ),
            (
                E::InvalidConditionalSelection { start: 2, end: 8 },
                "the conditional selection from 2 to 8 is invalid",
            ),
            (
                E::InvalidConditionalContext {
                    purpose: ConditionalPurpose::MinContent,
                },
                "the line context is invalid for a min-content conditional query",
            ),
            (
                E::InvalidConditionalReceipt {
                    receipt: 1,
                    opportunity: 6,
                },
                "conditional receipt 1 is invalid for opportunity 6",
            ),
            (
                E::RetainedConditionalPrefix {
                    opportunity: 6,
                    alternative: None,
                },
                "opportunity 6 would retain a conditional prefix that a phantom disposition cannot retain",
            ),
            (
                E::UnsupportedWrappingAnalysis,
                "the analysis does not support native wrapping relations",
            ),
            (
                E::MissingWrappingPolicy { relation: 3 },
                "wrapping relation 3 has no supplied policy",
            ),
            (
                E::ForeignWrappingRelation { relation: 3 },
                "wrapping relation 3 belongs to a different analysis",
            ),
            (
                E::DuplicateWrappingPolicy { relation: 3 },
                "wrapping relation 3 has its policy supplied more than once",
            ),
            (
                E::ConflictingWrappingPolicy { relation: 3 },
                "wrapping relation 3 has conflicting policies",
            ),
            (
                E::ForeignAlternative {
                    alternative: 2,
                    opportunity: 5,
                },
                "alternative 2 does not belong to opportunity 5",
            ),
            (
                E::UnsupportedContext {
                    field: ProtocolContextField::TabStops,
                },
                "the line context field tab stops is not supported",
            ),
            (
                E::MismatchedContext {
                    field: ProtocolContextField::BaseDirection,
                },
                "the line context field base direction is inconsistent with the prepared text or the request",
            ),
            (
                E::InvalidAtomicGeometry { occurrence: 3 },
                "the allocation of atomic occurrence 3 is invalid",
            ),
            (
                E::DuplicateSpacingSlot { slot: 4 },
                "spacing slot 4 was allocated more than once",
            ),
            (
                E::MissingSpacingSlot { slot: 4 },
                "no allocation was supplied for spacing slot 4",
            ),
            (
                E::IneligibleSpacingSlot { slot: 4 },
                "spacing slot 4 is not eligible for the supplied allocation",
            ),
            (
                E::UnexpectedAtomic { occurrence: 3 },
                "atomic occurrence 3 is not part of the selected line or semantic window",
            ),
            (
                E::MissingAlternative { opportunity: 5 },
                "no alternative was supplied for opportunity 5",
            ),
            (
                E::InvalidAnchorOrder {
                    position: 2,
                    kind: ProtocolHandleKind::Atomic,
                    occurrence: 6,
                },
                "the anchor order is invalid at position 2 for atomic 6",
            ),
            (
                E::MissingControl { occurrence: 3 },
                "no allocation was supplied for control 3",
            ),
            (
                E::MissingControlBoundaryGeometry { occurrence: 3 },
                "control 3 lacks boundary geometry or carries it without being a boundary endpoint",
            ),
            (
                E::MissingBoundaryContext { occurrence: 3 },
                "no boundary context was supplied for control endpoint 3",
            ),
            (
                E::DuplicateBoundaryContext { occurrence: 3 },
                "the boundary context for control 3 duplicates an allocation of the same control",
            ),
            (
                E::UnexpectedBoundaryContext { occurrence: 3 },
                "control 3 was not demanded and cannot take a boundary context",
            ),
            (
                E::DuplicateControl { occurrence: 3 },
                "control 3 was allocated more than once",
            ),
            (
                E::UnexpectedControl { occurrence: 3 },
                "control 3 is not part of the selected line or semantic window",
            ),
            (
                E::InvalidControlGeometry { occurrence: 3 },
                "the geometry supplied for control 3 is invalid",
            ),
            (
                E::MissingLineMetrics,
                "no line metrics are available for the selected line",
            ),
            (E::InvalidLineStrut, "the line strut is invalid"),
            (
                E::UnsupportedStandaloneAnalysis,
                "the analysis is not supported by the standalone operation",
            ),
            (
                E::ForeignPlacement,
                "the position belongs to a different placement",
            ),
            (
                E::OutsidePlacement {
                    index: 9,
                    coverage: Range::new(2, 6),
                },
                "source position 9 is outside the placement coverage 2..6 or faces away from it",
            ),
            (
                E::AmbiguousSourcePosition {
                    index: 4,
                    affinity: Affinity::After,
                },
                "source index 4 is ambiguous at the given affinity",
            ),
            (
                E::InvalidPlacedPosition { position: 11 },
                "placed position 11 is invalid",
            ),
            (
                E::EmptyPlacement,
                "the placement has no lines or source segments",
            ),
            (
                E::IncompleteLineCoverage { line: 2 },
                "line coverage is incomplete at line 2",
            ),
            (
                E::DuplicateLine { line: 2 },
                "line 2 was supplied more than once",
            ),
            (
                E::InvalidFragment {
                    line: 1,
                    fragment: 3,
                },
                "fragment 3 of line 1 is invalid",
            ),
            (
                E::AtomicPlacementMismatch {
                    line: 1,
                    occurrence: 3,
                },
                "atomic occurrence 3 of line 1 does not match its placement",
            ),
        ],
        index_protocol,
        |error| {
            matches!(
                error,
                E::InlineBoxFragmentation(_) | E::InlineBoxEdge(_) | E::WrappingScopeQuery { .. }
            )
        },
    );
}

#[test]
fn a_retained_prefix_names_its_alternative_when_it_has_one() {
    let error = ProtocolError::RetainedConditionalPrefix {
        opportunity: 6,
        alternative: Some(2),
    };
    assert_eq!(
        error.to_string(),
        "alternative 2 of opportunity 6 would retain a conditional prefix that a phantom disposition cannot retain"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn protocol_wrappers_report_their_reason_from_source() {
    let fragmentation = InlineBoxFragmentationError::DifferentBody;
    let outer = ProtocolError::InlineBoxFragmentation(fragmentation);
    check_wrapper(&outer, &fragmentation);
    assert_eq!(
        outer.to_string(),
        "the inline-box fragment request is invalid"
    );
    assert_eq!(
        std::error::Error::source(&outer).unwrap().to_string(),
        "the fragments belong to different bodies"
    );

    let edge = InlineBoxEdgeError::InvalidPair;
    let outer = ProtocolError::InlineBoxEdge(edge);
    check_wrapper(&outer, &edge);
    assert_eq!(
        outer.to_string(),
        "the inline-box edge operation is invalid"
    );
    assert_eq!(
        std::error::Error::source(&outer).unwrap().to_string(),
        "the box-edge endpoints are not a valid, properly nested pair of boundary controls"
    );

    let scope = WrappingScopeQueryError::ForeignCapture;
    let outer = ProtocolError::WrappingScopeQuery { reason: scope };
    check_wrapper(&outer, &scope);
    assert_eq!(outer.to_string(), "the wrapping scope query is invalid");
    assert_eq!(
        std::error::Error::source(&outer).unwrap().to_string(),
        "the scope belongs to a different capture"
    );
}

#[test]
fn every_handle_kind_has_its_own_name_in_a_message() {
    use ProtocolHandleKind as K;
    let names = [
        (K::SourceSegment, "source segment"),
        (K::WordPortion, "word portion"),
        (K::Origin, "origin"),
        (K::Unit, "unit"),
        (K::Cell, "cell"),
        (K::Atomic, "atomic"),
        (K::Control, "control"),
        (K::ComputedLineBoundary, "computed line boundary"),
        (K::Boundary, "boundary"),
        (K::BoundaryIncidence, "boundary incidence"),
        (K::Opportunity, "opportunity"),
        (K::ConditionalOpportunity, "conditional opportunity"),
        (K::Alternative, "alternative"),
        (K::Fragment, "fragment"),
    ];
    // An exhaustive match keeps this list complete when a kind is added.
    for (kind, _) in &names {
        match kind {
            K::SourceSegment
            | K::WordPortion
            | K::Origin
            | K::Unit
            | K::Cell
            | K::Atomic
            | K::Control
            | K::ComputedLineBoundary
            | K::Boundary
            | K::BoundaryIncidence
            | K::Opportunity
            | K::ConditionalOpportunity
            | K::Alternative
            | K::Fragment => {}
        }
    }
    assert_eq!(names.len(), 14);
    for (kind, name) in names {
        let error = ProtocolError::ForeignPreparation {
            kind,
            occurrence: 5,
        };
        assert_eq!(
            error.to_string(),
            format!("the {name} handle 5 was issued by a different preparation")
        );
    }
}

#[test]
fn every_context_field_has_its_own_name_in_a_message() {
    use ProtocolContextField as F;
    let names = [
        (F::BaseDirection, "base direction"),
        (F::Orientation, "orientation"),
        (F::BaselineReference, "baseline reference"),
        (F::LineStart, "line start"),
        (F::LineEnd, "line end"),
        (F::TabStops, "tab stops"),
        (F::TabPreparation, "tab preparation"),
        (F::TabReference, "tab reference"),
        (F::TabMetrics, "tab metrics"),
        (F::CssParagraphBoundary, "CSS paragraph boundary"),
        (F::AlignmentInsets, "alignment insets"),
        (F::Spacing, "spacing"),
    ];
    for (field, _) in &names {
        match field {
            F::BaseDirection
            | F::Orientation
            | F::BaselineReference
            | F::LineStart
            | F::LineEnd
            | F::TabStops
            | F::TabPreparation
            | F::TabReference
            | F::TabMetrics
            | F::CssParagraphBoundary
            | F::AlignmentInsets
            | F::Spacing => {}
        }
    }
    assert_eq!(names.len(), 12);
    for (field, name) in names {
        assert_eq!(
            ProtocolError::UnsupportedContext { field }.to_string(),
            format!("the line context field {name} is not supported")
        );
        assert_eq!(
            ProtocolError::MismatchedContext { field }.to_string(),
            format!(
                "the line context field {name} is inconsistent with the prepared text or the request"
            )
        );
    }
}

#[test]
fn both_conditional_purposes_have_their_own_name_in_a_message() {
    for (purpose, name) in [
        (ConditionalPurpose::Overflow, "overflow"),
        (ConditionalPurpose::MinContent, "min-content"),
    ] {
        match purpose {
            ConditionalPurpose::Overflow | ConditionalPurpose::MinContent => {}
        }
        assert_eq!(
            ProtocolError::InvalidConditionalContext { purpose }.to_string(),
            format!("the line context is invalid for a {name} conditional query")
        );
    }
}

fn index_edge(error: &InlineBoxEdgeError) -> usize {
    use InlineBoxEdgeError as E;
    match error {
        E::MissingCapture => 0,
        E::ForeignCapture => 1,
        E::WrongDomain => 2,
        E::InvalidOwner => 3,
        E::InvalidPair => 4,
        E::InvalidParent => 5,
        E::InconsistentContext => 6,
        E::UnrepresentedIncidence => 7,
    }
}

#[test]
fn inline_box_edge_errors_state_their_cause() {
    use InlineBoxEdgeError as E;
    check_table(
        8,
        vec![
            (
                E::MissingCapture,
                "the box-edge operation has no contextual capture",
            ),
            (
                E::ForeignCapture,
                "the box-edge handles belong to a different contextual capture",
            ),
            (
                E::WrongDomain,
                "the box-edge handles belong to different content domains or to a domain other than the prepared paragraph's",
            ),
            (
                E::InvalidOwner,
                "the declared box-edge owner is not a usable inline box",
            ),
            (
                E::InvalidPair,
                "the box-edge endpoints are not a valid, properly nested pair of boundary controls",
            ),
            (
                E::InvalidParent,
                "the declared box parent does not immediately contain the box",
            ),
            (
                E::InconsistentContext,
                "a box has inconsistent edge contexts",
            ),
            (
                E::UnrepresentedIncidence,
                "the requested box incidence is not represented",
            ),
        ],
        index_edge,
        |_| false,
    );
}

fn index_fragmentation(error: &InlineBoxFragmentationError) -> usize {
    use InlineBoxFragmentationError as E;
    match error {
        E::IncompleteTopology { .. } => 0,
        E::IncompleteBodyCoverage { .. } => 1,
        E::ForeignFamily => 2,
        E::FamilyContextMismatch { .. } => 3,
        E::DifferentBody => 4,
        E::InvalidIncidence => 5,
    }
}

#[test]
fn inline_box_fragmentation_errors_state_their_cause() {
    use InlineBoxFragmentationError as E;
    check_table(
        6,
        vec![
            (
                E::IncompleteTopology { domain: 2 },
                "domain 2 was not fully captured",
            ),
            (
                E::IncompleteBodyCoverage {
                    section: 1,
                    paragraph: 3,
                },
                "paragraph 3 of section 1 is not covered by the captured body",
            ),
            (
                E::ForeignFamily,
                "the handle belongs to a different fragment family or preparation",
            ),
            (
                E::FamilyContextMismatch { control: 4 },
                "the context of control 4 differs from the fragment family's",
            ),
            (E::DifferentBody, "the fragments belong to different bodies"),
            (
                E::InvalidIncidence,
                "the inline box incidence is invalid for fragmentation",
            ),
        ],
        index_fragmentation,
        |_| false,
    );
}

fn index_wrapping_scope(error: &WrappingScopeQueryError) -> usize {
    use WrappingScopeQueryError as E;
    match error {
        E::MissingCapture => 0,
        E::ForeignCapture => 1,
        E::NonInlineScope => 2,
        E::WrongDomain => 3,
        E::UnrepresentedIncidence => 4,
    }
}

#[test]
fn wrapping_scope_query_errors_state_their_cause() {
    use WrappingScopeQueryError as E;
    check_table(
        5,
        vec![
            (
                E::MissingCapture,
                "the prepared paragraph has no contextual capture",
            ),
            (
                E::ForeignCapture,
                "the scope belongs to a different capture",
            ),
            (
                E::NonInlineScope,
                "the scope is a content domain, not an inline box",
            ),
            (
                E::WrongDomain,
                "the scope belongs to a different content domain than the prepared paragraph",
            ),
            (
                E::UnrepresentedIncidence,
                "the prepared paragraph does not represent the requested scope incidence",
            ),
        ],
        index_wrapping_scope,
        |_| false,
    );
}
