#![forbid(unsafe_code)]

//! Messages of the source-processing, transform, text-combine, range-fragment
//! and scope-alignment error enums.
//!
//! Every variant is public, so each table constructs all of them. A row states
//! the documented meaning of one variant; the `index` functions are exhaustive
//! matches without a fallback arm, so a new variant does not compile until it
//! has a row. `SourceProcessingError` wraps three of the other enums and
//! returns each from `source()` without repeating its text.

use surgeist_text::{
    ComputedLineBoundaryError, DocumentSegmentBreakError, ProcessingCapacityError, Range,
    ScopeAlignmentError, ScopeAlignmentLimitKind, SourceContinuityOperation,
    SourceProcessingConnectionError, SourceProcessingError, SourceProcessingJointFlowError,
    SourceRangeFragmentError, TextCombineError, TextTransformError,
};

mod error_support;
use error_support::{check_table, check_wrapper};

fn index_delimiter(error: &DocumentSegmentBreakError) -> usize {
    use DocumentSegmentBreakError as E;
    match error {
        E::Empty => 0,
        E::Reversed => 1,
        E::OutOfBounds => 2,
        E::Unordered => 3,
        E::Overlap => 4,
    }
}

#[test]
fn delimiter_errors_state_which_ordering_rule_was_broken() {
    use DocumentSegmentBreakError as E;
    check_table(
        5,
        vec![
            (E::Empty, "the delimiter occurrence covers no source units"),
            (
                E::Reversed,
                "the delimiter occurrence ends before it starts",
            ),
            (
                E::OutOfBounds,
                "the delimiter occurrence extends beyond the source",
            ),
            (
                E::Unordered,
                "the delimiter occurrence starts before the previous occurrence starts",
            ),
            (
                E::Overlap,
                "the delimiter occurrence starts inside the previous occurrence",
            ),
        ],
        index_delimiter,
        |_| false,
    );
}

fn index_capacity(error: &ProcessingCapacityError) -> usize {
    match error {
        ProcessingCapacityError::Overflow => 0,
        ProcessingCapacityError::Allocation => 1,
    }
}

#[test]
fn capacity_errors_distinguish_overflow_from_refused_reservation() {
    check_table(
        2,
        vec![
            (
                ProcessingCapacityError::Overflow,
                "a source-processing count or byte size exceeded the representable range",
            ),
            (
                ProcessingCapacityError::Allocation,
                "a source-processing buffer reservation was refused",
            ),
        ],
        index_capacity,
        |_| false,
    );
}

fn index_source_processing(error: &SourceProcessingError) -> usize {
    use SourceProcessingError as E;
    match error {
        E::MissingSourceContinuity { .. } => 0,
        E::SourceConnection { .. } => 1,
        E::JointFlow { .. } => 2,
        E::ComputedLineBoundary { .. } => 3,
        E::ScopeBoundaries => 4,
        E::IncompleteExterior => 5,
        E::Nesting => 6,
        E::Occurrence => 7,
        E::OriginalCoverage => 8,
        E::OriginCorrespondence => 9,
        E::Domain => 10,
        E::DelimiterCrossing => 11,
        E::Connection => 12,
        E::BoundaryPolicy => 13,
        E::Replacement => 14,
        E::OutsideParagraph => 15,
        E::ForeignParagraph => 16,
        E::ForeignJointFlow => 17,
        E::MissingJointAnalysisAuthority { .. } => 18,
    }
}

#[test]
fn source_processing_errors_state_the_rejected_capture_fact() {
    use SourceProcessingError as E;
    check_table(
        19,
        vec![
            (
                E::MissingSourceContinuity {
                    before_origin: 1,
                    after_origin: 2,
                    operation: SourceContinuityOperation::WhiteSpace,
                },
                "the captured source context cannot establish the required continuity between origins 1 and 2",
            ),
            (
                E::SourceConnection {
                    reason: SourceProcessingConnectionError::Endpoints { connection: 3 },
                },
                "a supplied source connection is invalid",
            ),
            (
                E::JointFlow {
                    reason: SourceProcessingJointFlowError::EmptyMembers { declaration: 0 },
                },
                "a supplied joint-flow declaration is invalid",
            ),
            (
                E::ComputedLineBoundary {
                    request: 5,
                    reason: ComputedLineBoundaryError::TargetRole,
                },
                "computed line boundary request 5 is invalid",
            ),
            (
                E::ScopeBoundaries,
                "a scope's declared boundary controls do not match its enter and exit events",
            ),
            (
                E::IncompleteExterior,
                "the producer did not capture the exterior context that trimming needs",
            ),
            (
                E::Nesting,
                "the captured scopes and paragraphs are not properly nested",
            ),
            (
                E::Occurrence,
                "a captured occurrence is out of range, repeated, missing or out of position",
            ),
            (
                E::OriginalCoverage,
                "an origin's range is invalid or the captured occurrences do not cover the original source",
            ),
            (
                E::OriginCorrespondence,
                "the captured origins do not correspond to the original source",
            ),
            (
                E::Domain,
                "the captured content domains are inconsistent with the captured scopes and occurrences",
            ),
            (
                E::DelimiterCrossing,
                "a document delimiter crosses a captured boundary, connection or content domain",
            ),
            (
                E::Connection,
                "the captured connections do not join consecutive origins",
            ),
            (
                E::BoundaryPolicy,
                "a captured boundary policy is duplicated, unused or missing",
            ),
            (
                E::Replacement,
                "a supplied replacement does not belong to any captured paragraph",
            ),
            (
                E::OutsideParagraph,
                "a source range lies outside its paragraph",
            ),
            (
                E::ForeignParagraph,
                "the paragraph view belongs to a different capture",
            ),
            (
                E::ForeignJointFlow,
                "the joint-flow handle belongs to a different capture",
            ),
            (
                E::MissingJointAnalysisAuthority {
                    first_paragraph: 2,
                    last_paragraph: 4,
                    source: Range::new(10, 20),
                },
                "the producer did not authorize joint analysis of paragraphs 2 to 4",
            ),
        ],
        index_source_processing,
        |error| {
            matches!(
                error,
                E::SourceConnection { .. } | E::JointFlow { .. } | E::ComputedLineBoundary { .. }
            )
        },
    );
}

#[test]
fn source_processing_wrappers_report_their_reason_from_source() {
    let connection = SourceProcessingConnectionError::Duplicate {
        connection: 2,
        first_connection: 1,
    };
    let outer = SourceProcessingError::SourceConnection { reason: connection };
    check_wrapper(&outer, &connection);
    assert_eq!(outer.to_string(), "a supplied source connection is invalid");
    assert_eq!(
        std::error::Error::source(&outer).unwrap().to_string(),
        "connection 2 starts at the same origin as connection 1"
    );

    let flow = SourceProcessingJointFlowError::NotDomainRoot {
        declaration: 1,
        scope: 4,
    };
    let outer = SourceProcessingError::JointFlow { reason: flow };
    check_wrapper(&outer, &flow);
    assert_eq!(
        outer.to_string(),
        "a supplied joint-flow declaration is invalid"
    );
    assert_eq!(
        std::error::Error::source(&outer).unwrap().to_string(),
        "scope 4 of joint-flow declaration 1 is not a content-domain root"
    );

    let boundary = ComputedLineBoundaryError::InsideDelimiter;
    let outer = SourceProcessingError::ComputedLineBoundary {
        request: 7,
        reason: boundary,
    };
    check_wrapper(&outer, &boundary);
    assert_eq!(
        outer.to_string(),
        "computed line boundary request 7 is invalid"
    );
    assert_eq!(
        std::error::Error::source(&outer).unwrap().to_string(),
        "the computed boundary falls inside a document delimiter"
    );
}

fn index_connection(error: &SourceProcessingConnectionError) -> usize {
    use SourceProcessingConnectionError as E;
    match error {
        E::Endpoints { .. } => 0,
        E::Domain { .. } => 1,
        E::Position { .. } => 2,
        E::Duplicate { .. } => 3,
    }
}

#[test]
fn source_connection_errors_name_the_connection() {
    use SourceProcessingConnectionError as E;
    check_table(
        4,
        vec![
            (
                E::Endpoints { connection: 3 },
                "connection 3 names origins that do not exist or are not adjacent",
            ),
            (
                E::Domain { connection: 4 },
                "connection 4 does not lie in the content domain of both its paragraphs",
            ),
            (
                E::Position { connection: 5 },
                "connection 5 is at an invalid source position",
            ),
            (
                E::Duplicate {
                    connection: 6,
                    first_connection: 2,
                },
                "connection 6 starts at the same origin as connection 2",
            ),
        ],
        index_connection,
        |_| false,
    );
}

fn index_joint_flow(error: &SourceProcessingJointFlowError) -> usize {
    use SourceProcessingJointFlowError as E;
    match error {
        E::EmptyMembers { .. } => 0,
        E::InvalidScope { .. } => 1,
        E::NotDomainRoot { .. } => 2,
        E::InvalidMember { .. } => 3,
        E::DuplicateMember { .. } => 4,
        E::MemberAlreadyAssigned { .. } => 5,
        E::WrongDomain { .. } => 6,
        E::MemberOrder { .. } => 7,
        E::SkippedMember { .. } => 8,
        E::FactsOwnerNotMember { .. } => 9,
        E::DirectionMismatch { .. } => 10,
        E::StrutMismatch { .. } => 11,
        E::MissingConnection { .. } => 12,
    }
}

#[test]
fn joint_flow_errors_name_the_declaration_and_paragraph() {
    use SourceProcessingJointFlowError as E;
    check_table(
        13,
        vec![
            (
                E::EmptyMembers { declaration: 1 },
                "joint-flow declaration 1 lists no paragraphs",
            ),
            (
                E::InvalidScope {
                    declaration: 1,
                    scope: 9,
                },
                "joint-flow declaration 1 names scope 9, which does not exist",
            ),
            (
                E::NotDomainRoot {
                    declaration: 1,
                    scope: 4,
                },
                "scope 4 of joint-flow declaration 1 is not a content-domain root",
            ),
            (
                E::InvalidMember {
                    declaration: 1,
                    paragraph: 7,
                },
                "joint-flow declaration 1 names paragraph 7, which does not exist",
            ),
            (
                E::DuplicateMember {
                    declaration: 1,
                    paragraph: 7,
                },
                "joint-flow declaration 1 lists paragraph 7 twice",
            ),
            (
                E::MemberAlreadyAssigned {
                    declaration: 2,
                    paragraph: 7,
                    first_declaration: 1,
                },
                "paragraph 7 of joint-flow declaration 2 already belongs to declaration 1",
            ),
            (
                E::WrongDomain {
                    declaration: 1,
                    paragraph: 7,
                },
                "paragraph 7 is outside the domain of joint-flow declaration 1",
            ),
            (
                E::MemberOrder {
                    declaration: 1,
                    previous: 5,
                    paragraph: 3,
                },
                "joint-flow declaration 1 lists paragraph 3 out of order after paragraph 5",
            ),
            (
                E::SkippedMember {
                    declaration: 1,
                    paragraph: 4,
                },
                "joint-flow declaration 1 skips paragraph 4 of its range",
            ),
            (
                E::FactsOwnerNotMember {
                    declaration: 1,
                    paragraph: 8,
                },
                "the facts owner, paragraph 8, of joint-flow declaration 1 is not one of its members",
            ),
            (
                E::DirectionMismatch {
                    declaration: 1,
                    paragraph: 3,
                    owner: 2,
                },
                "paragraph 3 of joint-flow declaration 1 has a base direction that differs from owner paragraph 2",
            ),
            (
                E::StrutMismatch {
                    declaration: 1,
                    paragraph: 3,
                    owner: 2,
                },
                "paragraph 3 of joint-flow declaration 1 has a strut that differs from owner paragraph 2",
            ),
            (
                E::MissingConnection {
                    declaration: 1,
                    before_origin: 4,
                    after_origin: 5,
                },
                "joint-flow declaration 1 lacks a connection between origins 4 and 5",
            ),
        ],
        index_joint_flow,
        |_| false,
    );
}

fn index_computed(error: &ComputedLineBoundaryError) -> usize {
    use ComputedLineBoundaryError as E;
    match error {
        E::TargetBounds => 0,
        E::TargetRole => 1,
        E::MissingParagraph => 2,
        E::Domain => 3,
        E::InsideScalar => 4,
        E::InsideDelimiter => 5,
    }
}

#[test]
fn computed_boundary_errors_state_which_requirement_failed() {
    use ComputedLineBoundaryError as E;
    check_table(
        6,
        vec![
            (
                E::TargetBounds,
                "the computed boundary names a target that does not exist",
            ),
            (
                E::TargetRole,
                "the computed boundary target is not an inline box edge or an in-flow atomic box",
            ),
            (
                E::MissingParagraph,
                "the computed boundary has no containing paragraph",
            ),
            (
                E::Domain,
                "the computed boundary target and its paragraph belong to different content domains",
            ),
            (
                E::InsideScalar,
                "the computed boundary falls inside a Unicode scalar value",
            ),
            (
                E::InsideDelimiter,
                "the computed boundary falls inside a document delimiter",
            ),
        ],
        index_computed,
        |_| false,
    );
}

fn index_transform(error: &TextTransformError) -> usize {
    use TextTransformError as E;
    match error {
        E::IndivisibleCoverage { .. } => 0,
        E::InvalidPhraseData => 1,
        E::MissingPhraseTopology { .. } => 2,
        E::MissingPhraseScopeStyle { .. } => 3,
        E::MissingSeparatorBoundaryContext { .. } => 4,
        E::MissingOriginalSeparatorBoundaryContext { .. } => 5,
        E::UnrepresentableSeparatorSeam { .. } => 6,
        E::InvalidTextNodeRange { .. } => 7,
        E::TextNodeMismatch { .. } => 8,
        E::MissingTextNodeContext { .. } => 9,
        E::CasingDataUnavailable => 10,
        E::InvalidCasingData => 11,
        E::InvalidContentLocale => 12,
        E::Capacity => 13,
    }
}

#[test]
fn transform_errors_state_their_cause() {
    use TextTransformError as E;
    check_table(
        14,
        vec![
            (
                E::IndivisibleCoverage {
                    source: Range::new(3, 5),
                },
                "the transform cannot keep the indivisible original unit 3..5 whole",
            ),
            (
                E::InvalidPhraseData,
                "the retained phrase model data is invalid",
            ),
            (
                E::MissingPhraseTopology { at: 4 },
                "phrase separator insertion at 4 needs the captured scope topology",
            ),
            (
                E::MissingPhraseScopeStyle { scope: 2, at: 4 },
                "phrase separator insertion at 4 needs the style of scope 2",
            ),
            (
                E::MissingSeparatorBoundaryContext {
                    control: 1,
                    at: 4,
                    connection: 2,
                },
                "separator control 1 at 4 lacks the boundary event order of connection 2",
            ),
            (
                E::MissingOriginalSeparatorBoundaryContext {
                    source: Range::new(3, 4),
                    at: 3,
                    connection: 0,
                },
                "the original separator 3..4 depends on an uncaptured side of connection 0 at 3",
            ),
            (
                E::UnrepresentableSeparatorSeam {
                    control: 1,
                    at: 4,
                    retained: Range::new(3, 6),
                },
                "separator control 1 at 4 lies inside the retained unit 3..6 and has no exact scalar seam",
            ),
            (
                E::InvalidTextNodeRange {
                    range: Range::new(5, 9),
                    text_len: 7,
                },
                "text-node range 5..9 is reversed or extends beyond the node's UTF-16 length 7",
            ),
            (
                E::TextNodeMismatch { origin: 2 },
                "the text-node content for origin 2 differs from its original occurrence",
            ),
            (
                E::MissingTextNodeContext { origin: 2 },
                "the transform requires complete original text-node context for origin 2",
            ),
            (E::CasingDataUnavailable, "the casing data is unavailable"),
            (E::InvalidCasingData, "the casing data is invalid"),
            (
                E::InvalidContentLocale,
                "the content locale is not a valid language tag",
            ),
            (
                E::Capacity,
                "the transform could not reserve storage for its result",
            ),
        ],
        index_transform,
        |_| false,
    );
}

fn index_combine(error: &TextCombineError) -> usize {
    use TextCombineError as E;
    match error {
        E::MissingCapture => 0,
        E::MissingMode { .. } => 1,
        E::MissingWritingAxis { .. } => 2,
        E::MissingInnerDirection { .. } => 3,
        E::MissingScopeContext { .. } => 4,
        E::UnqualifiedWidthObservation { .. } => 5,
        E::UnsupportedCellPreparation => 6,
    }
}

#[test]
fn text_combine_errors_name_the_missing_fact() {
    use TextCombineError as E;
    check_table(
        7,
        vec![
            (
                E::MissingCapture,
                "the preparation has no contextual capture for text-combine-upright",
            ),
            (
                E::MissingMode { scope: 3 },
                "scope 3 has no captured text-combine-upright mode",
            ),
            (
                E::MissingWritingAxis { domain: 1 },
                "domain 1 has no captured writing axis",
            ),
            (
                E::MissingInnerDirection { scope: 3 },
                "scope 3 has no captured horizontal inner direction",
            ),
            (
                E::MissingScopeContext { scope: 3 },
                "the captured context around scope 3 is incomplete",
            ),
            (
                E::UnqualifiedWidthObservation {
                    scope: 3,
                    source: Range::new(2, 4),
                    feature: surgeist_shaper::WidthFeature::HalfWidth,
                    path: surgeist_shaper::WidthObservationPath::ValueRecord,
                },
                "the width-feature path cannot establish complete evidence for scope 3",
            ),
            (
                E::UnsupportedCellPreparation,
                "the requested cell behavior is outside native preparation capability",
            ),
        ],
        index_combine,
        |_| false,
    );
}

fn index_fragments(error: &SourceRangeFragmentError) -> usize {
    use SourceRangeFragmentError as E;
    match error {
        E::EmptyRange => 0,
        E::OutsideOrigin { .. } => 1,
        E::OutsidePlacement { .. } => 2,
        E::MissingOriginalMapping { .. } => 3,
        E::AmbiguousOriginalMapping { .. } => 4,
        E::MissingUnitGeometry { .. } => 5,
        E::UnsupportedPartition { .. } => 6,
        E::MissingFontMetrics { .. } => 7,
        E::MissingCellReference { .. } => 8,
        E::UnsupportedCellPartition { .. } => 9,
        E::AmbiguousCellUnit { .. } => 10,
    }
}

#[test]
fn range_fragment_errors_name_the_range_or_unit() {
    use SourceRangeFragmentError as E;
    check_table(
        11,
        vec![
            (E::EmptyRange, "the source range is empty"),
            (
                E::OutsideOrigin {
                    origin: 2,
                    local: Range::new(4, 9),
                    captured: Range::new(0, 6),
                },
                "source range 4..9 lies outside the captured range 0..6 of origin 2",
            ),
            (
                E::OutsidePlacement {
                    aggregate: Range::new(10, 12),
                },
                "aggregate range 10..12 lies outside the source the placement covers",
            ),
            (
                E::MissingOriginalMapping {
                    aggregate: Range::new(3, 5),
                },
                "the placement has no original mapping for aggregate range 3..5",
            ),
            (
                E::AmbiguousOriginalMapping {
                    aggregate: Range::new(3, 5),
                },
                "the original mappings of aggregate range 3..5 overlap one another",
            ),
            (
                E::MissingUnitGeometry { line: 1, unit: 2 },
                "grapheme group 2 of line 1 has no geometry for the requested range",
            ),
            (
                E::UnsupportedPartition { line: 1, unit: 2 },
                "grapheme group 2 of line 1 cannot be partitioned for the requested range",
            ),
            (
                E::MissingFontMetrics { line: 1, unit: 2 },
                "no font metrics are available for grapheme group 2 of line 1",
            ),
            (
                E::MissingCellReference {
                    line: 1,
                    cell: 3,
                    unit: 2,
                },
                "inner unit 2 of cell 3 on line 1 has no retained font or layout reference",
            ),
            (
                E::UnsupportedCellPartition {
                    line: 1,
                    cell: 3,
                    unit: 2,
                },
                "the provider members of cell 3 on line 1 cannot be partitioned into inner unit 2",
            ),
            (
                E::AmbiguousCellUnit {
                    line: 1,
                    cell: 3,
                    selection_unit: 5,
                },
                "source unit 5 of cell 3 on line 1 has no unique inner-unit closure",
            ),
        ],
        index_fragments,
        |_| false,
    );
}

fn index_alignment(error: &ScopeAlignmentError) -> usize {
    use ScopeAlignmentError as E;
    match error {
        E::LimitsRequired => 0,
        E::Limit { .. } => 1,
        E::Overflow { .. } => 2,
        E::Allocation { .. } => 3,
        E::MissingCapture => 4,
        E::ForeignCapture => 5,
        E::WrongDomain => 6,
        E::MissingBoundary { .. } => 7,
        E::MissingStyle { .. } => 8,
        E::MissingFontReference { .. } => 9,
        E::MissingMetrics { .. } => 10,
        E::UnsupportedReference { .. } => 11,
        E::UnrepresentedOwnership { .. } => 12,
        E::UnpartitionableUnit { .. } => 13,
        E::ForeignSelectedScope => 14,
        E::DuplicateAllocation { .. } => 15,
        E::MissingAllocation { .. } => 16,
        E::IncompatibleFrames { .. } => 17,
        E::BoundaryMismatch { .. } => 18,
    }
}

#[test]
fn scope_alignment_errors_state_their_cause() {
    use ScopeAlignmentError as E;
    check_table(
        19,
        vec![
            (
                E::LimitsRequired,
                "scope alignment needs explicit resource limits, but none were supplied",
            ),
            (
                E::Limit {
                    kind: ScopeAlignmentLimitKind::Records,
                    required: 50,
                    limit: 40,
                },
                "the scope-alignment limit on records (40) cannot admit the required 50",
            ),
            (
                E::Overflow {
                    kind: ScopeAlignmentLimitKind::PayloadBytes,
                },
                "scope-alignment count or byte arithmetic overflowed or underflowed",
            ),
            (
                E::Allocation {
                    kind: ScopeAlignmentLimitKind::Records,
                },
                "reserving scope-alignment storage was refused",
            ),
            (
                E::MissingCapture,
                "a contextual capture, parent scope or scope-alignment custody that the operation requires is missing or unavailable",
            ),
            (
                E::ForeignCapture,
                "a supplied scope, line or product belongs to a different capture or preparation",
            ),
            (
                E::WrongDomain,
                "a scope, computed cause or scope tree does not fit the content domain the operation requires",
            ),
            (
                E::MissingBoundary { scope: 3 },
                "scope 3 lacks its early formatting classification",
            ),
            (
                E::MissingStyle { scope: 3 },
                "scope 3 has no supplied effective style",
            ),
            (
                E::MissingFontReference { scope: 3 },
                "scope 3 lacks its supplied script, axis and baseline facts",
            ),
            (
                E::MissingMetrics { scope: 3 },
                "scope 3 lacks the metrics the operation demands",
            ),
            (
                E::UnsupportedReference { scope: 3 },
                "scope 3 has a font reference, orientation or baseline that the operation's frame cannot use",
            ),
            (
                E::UnrepresentedOwnership { scope: 3 },
                "content custody cannot represent the requested frame (scope 3; 0 when no scope applies)",
            ),
            (
                E::UnpartitionableUnit { scope: 3 },
                "a boundary of scope 3 divides an indivisible unit",
            ),
            (
                E::ForeignSelectedScope,
                "a scope handle belongs to a different selected presentation",
            ),
            (
                E::DuplicateAllocation { scope: 3 },
                "scope 3 received more than one allocation",
            ),
            (
                E::MissingAllocation { scope: 3 },
                "no allocation or allocation limits were supplied for scope 3",
            ),
            (
                E::IncompatibleFrames {
                    first: 3,
                    second: 4,
                },
                "scopes 3 and 4 jointly own output but were assigned different frames",
            ),
            (
                E::BoundaryMismatch { scope: 3 },
                "what was supplied for scope 3 disagrees with its captured formatting or owner",
            ),
        ],
        index_alignment,
        |_| false,
    );
}
