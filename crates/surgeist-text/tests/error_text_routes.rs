#![forbid(unsafe_code)]

//! Messages of the error types as a caller obtains them from public operations.
//!
//! The tables in the other `error_text_*` files construct every variant
//! directly. These tests start from an operation that fails, read the typed
//! reason back through `Error::detail()` (or from the operation's own error
//! type) and assert its text. Each route changes one fact of an otherwise valid
//! input, so the failing condition is the one the variant documents.
//!
//! Variants with no short public route are not reached here and rest on their
//! table row only: the two `ProcessingCapacityError` variants (they need an
//! arithmetic overflow or a refused allocation), the source-transform variants
//! other than the ones below, `SourceRangeFragmentError` (it needs a placed
//! layout) and the line-selection, finalization and placement variants of
//! `ProtocolError` (they need a prepared line). Existing integration tests
//! reach several of them and assert the typed reason; this file adds the text
//! for the ones that a few lines of setup reach.

use std::error::Error as StdError;

use surgeist_text as t;

mod common;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

fn detail(error: &t::Error) -> &t::ErrorDetail {
    error
        .detail()
        .expect("the operation attaches a typed reason")
}

// ---------------------------------------------------------------- bidi and supplied lines

#[test]
fn a_resolved_level_above_126_is_refused_with_its_value() {
    let error = t::BidiLevel::try_new(127).unwrap_err();
    assert_eq!(error, t::BidiLevelError::OutOfRange { level: 127 });
    assert_eq!(
        error.to_string(),
        "bidi level 127 is above the highest resolved level 126"
    );
    assert!(t::BidiLevel::try_new(126).is_ok());
}

#[test]
fn a_visual_order_buffer_of_the_wrong_length_is_refused_with_both_lengths() {
    let level = t::BidiLevel::try_new(0).unwrap();
    let participants = [
        t::SuppliedInlineParticipant::Atomic { level },
        t::SuppliedInlineParticipant::Atomic { level },
    ];
    let mut order = [0usize; 1];
    let error = t::SuppliedLine::new(&participants)
        .reorder_into(&mut order)
        .map(|_| ())
        .unwrap_err();
    assert_eq!(
        error,
        t::SuppliedLineError::OrderBufferLength {
            expected: 2,
            actual: 1
        }
    );
    assert_eq!(
        error.to_string(),
        "the visual-order buffer holds 1 slots but the line has 2 participants"
    );
}

// ---------------------------------------------------------------- document delimiters

fn delimiter_reason(ranges: Vec<t::Range>) -> (usize, t::DocumentSegmentBreakError) {
    let error = t::process_source_whitespace(
        t::Source::new("abcde"),
        t::Style::default(),
        t::DocumentSegmentBreaks::Defined(ranges),
    )
    .unwrap_err();
    match detail(&error) {
        t::ErrorDetail::InvalidDocumentSegmentBreak {
            occurrence, reason, ..
        } => (*occurrence, *reason),
        other => panic!("unexpected detail {other:?}"),
    }
}

#[test]
fn each_invalid_delimiter_occurrence_reports_its_own_rule() {
    let cases = [
        (
            vec![t::Range { start: 3, end: 2 }],
            0,
            t::DocumentSegmentBreakError::Reversed,
            "the delimiter occurrence ends before it starts",
        ),
        (
            vec![t::Range::new(2, 2)],
            0,
            t::DocumentSegmentBreakError::Empty,
            "the delimiter occurrence covers no source units",
        ),
        (
            vec![t::Range::new(2, 9)],
            0,
            t::DocumentSegmentBreakError::OutOfBounds,
            "the delimiter occurrence extends beyond the source",
        ),
        (
            vec![t::Range::new(3, 4), t::Range::new(1, 2)],
            1,
            t::DocumentSegmentBreakError::Unordered,
            "the delimiter occurrence starts before the previous occurrence starts",
        ),
        (
            vec![t::Range::new(1, 3), t::Range::new(2, 4)],
            1,
            t::DocumentSegmentBreakError::Overlap,
            "the delimiter occurrence starts inside the previous occurrence",
        ),
    ];
    for (ranges, occurrence, reason, text) in cases {
        let (found_occurrence, found) = delimiter_reason(ranges);
        assert_eq!(found_occurrence, occurrence);
        assert_eq!(found, reason);
        assert_eq!(found.to_string(), text);
    }
}

// ---------------------------------------------------------------- paragraph input

fn origin(source: &t::Source, start: usize, end: usize) -> t::ParagraphOrigin {
    t::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: t::Range::new(start, end),
        aggregate: t::Range::new(start, end),
        kind: t::ParagraphOriginKind::Authored,
    }
}

fn paragraph(value: &str) -> t::ParagraphInput {
    let source = t::Source::identified(t::Text::from(value), t::Id::from_u64(65401), 1);
    let end = source.text().len();
    t::ParagraphInput {
        source: source.clone(),
        base_style: t::Style::default(),
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: t::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: t::ParagraphStrut::DefaultFont,
        terminal_edge: t::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![origin(&source, 0, end)],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn protocol_reason(input: t::ParagraphInput) -> t::ProtocolError {
    let error =
        t::AnalyzedSource::from_paragraph(input, t::AnalysisTypography::HorizontalComposition)
            .map(|_| ())
            .unwrap_err();
    assert_eq!(error.code, t::ErrorCode::InvalidTextProtocol);
    match detail(&error) {
        t::ErrorDetail::InvalidTextProtocol { reason } => reason.clone(),
        other => panic!("unexpected detail {other:?}"),
    }
}

#[test]
fn an_invalid_paragraph_reports_the_protocol_rule_it_breaks() {
    let two_origins = || {
        let mut input = paragraph("ab");
        let source = input.source.clone();
        input.origins = vec![origin(&source, 0, 1), origin(&source, 1, 2)];
        input
    };
    let mut cases: Vec<(t::ParagraphInput, t::ProtocolError, &str)> = Vec::new();

    let mut input = paragraph("ab");
    input.origins[0].local = t::Range::new(2, 1);
    cases.push((
        input,
        t::ProtocolError::InvalidOriginRange {
            origin: 0,
            local: t::Range::new(2, 1),
            aggregate: t::Range::new(0, 2),
        },
        "the range of origin 0 is invalid",
    ));

    let mut input = paragraph("ab");
    input.origins[0].local = t::Range::new(0, 1);
    input.origins[0].aggregate = t::Range::new(1, 2);
    cases.push((
        input,
        t::ProtocolError::IncompleteOriginCoverage {
            expected: 0,
            actual: 1,
        },
        "the origins leave the source incompletely covered: position 0 was expected but 1 was found",
    ));

    let mut input = paragraph("ab");
    input.origins[0].local = t::Range::new(0, 1);
    input.origins[0].aggregate = t::Range::new(0, 1);
    cases.push((
        input,
        t::ProtocolError::IncompleteOriginCoverage {
            expected: 2,
            actual: 1,
        },
        "the origins leave the source incompletely covered: position 2 was expected but 1 was found",
    ));

    let mut input = paragraph("ab");
    input.origins[0].source = t::Source::identified(t::Text::from("xb"), t::Id::from_u64(65402), 1);
    cases.push((
        input,
        t::ProtocolError::OriginTextMismatch {
            origin: 0,
            local: 0,
            aggregate: 0,
        },
        "the text of origin 0 differs between its local offset 0 and aggregate offset 0",
    ));

    cases.push((
        two_origins(),
        t::ProtocolError::MissingConnection {
            before: 0,
            after: 1,
        },
        "origins 0 and 1 have no connection",
    ));

    let mut input = two_origins();
    input.connections.push(t::ParagraphConnection {
        at: 0,
        kind: t::ParagraphConnectionKind::Disconnected,
    });
    cases.push((
        input,
        t::ProtocolError::InvalidConnection {
            connection: 0,
            at: 0,
        },
        "connection 0 at 0 is invalid",
    ));

    let mut input = paragraph("ab");
    input.controls.push(t::ParagraphControl {
        at: 9,
        kind: t::ParagraphControlKind::ForcedBreak,
    });
    cases.push((
        input,
        t::ProtocolError::InvalidControl { control: 0, at: 9 },
        "control 0 at 9 has an invalid source position",
    ));

    let mut input = paragraph("ab");
    input.boundary_policies.push(t::ParagraphBoundaryPolicy {
        definition: t::ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: t::Wrap::Word,
        following_wrap: t::Wrap::Word,
        ancestor_wrap: t::Wrap::Word,
    });
    cases.push((
        input,
        t::ProtocolError::InvalidBoundaryPolicy { policy: 0, at: 2 },
        "boundary policy 0 at 2 is invalid",
    ));

    let definition = t::ParagraphReplacementDefinition::Source {
        origin: 0,
        source: t::Range::new(0, 1),
    };
    let mut input = paragraph("ab");
    input
        .replacements
        .push(t::ParagraphReplacement::style_driven(definition, 9));
    cases.push((
        input,
        t::ProtocolError::InvalidReplacement {
            alternative: 0,
            source: t::Range::new(0, 1),
        },
        "replacement 0 does not describe a valid source range 0..1 ending at its insertion anchor",
    ));

    for (input, expected, text) in cases {
        let found = protocol_reason(input);
        assert_eq!(found, expected);
        assert_eq!(found.to_string(), text);
        assert!(found.source().is_none());
    }
}

// ---------------------------------------------------------------- contextual source capture

fn block() -> t::SourceProcessingScope {
    t::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: t::SourceProcessingTopology::Complete,
        trim: t::WhiteSpaceTrim::default(),
        base_style: t::Style::default(),
        used_direction: t::ParagraphBaseDirection::LeftToRight,
    }
}

/// One block scope holding one paragraph with one origin that covers "ab".
fn capture_input() -> t::ContextualSourceInput {
    use t::SourceProcessingEvent as E;
    let source = t::Source::new("ab");
    t::ContextualSourceInput {
        joint_flows: vec![],
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source: source.clone(),
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        envelope: t::SourceProcessingEnvelope::Complete,
        origins: vec![origin(&source, 0, 2)],
        connections: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        controls: vec![],
        scopes: vec![block()],
        paragraphs: vec![t::SourceProcessingParagraph {
            base_direction: t::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: t::ParagraphStrut::DefaultFont,
        }],
        events: vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: t::Affinity::After,
            },
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
    }
}

fn capture_reason(input: t::ContextualSourceInput) -> t::SourceProcessingError {
    let error = t::process_contextual_source(input).map(|_| ()).unwrap_err();
    match detail(&error) {
        t::ErrorDetail::SourceProcessing { reason } => *reason,
        other => panic!("unexpected detail {other:?}"),
    }
}

#[test]
fn a_valid_capture_is_admitted() {
    assert!(t::process_contextual_source(capture_input()).is_ok());
}

#[test]
fn an_invalid_capture_reports_the_rule_it_breaks() {
    let mut cases: Vec<(t::ContextualSourceInput, t::SourceProcessingError, &str)> = Vec::new();

    let mut input = capture_input();
    input.envelope = t::SourceProcessingEnvelope::IncompleteExterior;
    cases.push((
        input,
        t::SourceProcessingError::IncompleteExterior,
        "the producer did not capture the exterior context that trimming needs",
    ));

    let mut input = capture_input();
    input.events.swap(3, 4);
    cases.push((
        input,
        t::SourceProcessingError::Nesting,
        "the captured scopes and paragraphs are not properly nested",
    ));

    let mut input = capture_input();
    input.scopes.clear();
    cases.push((
        input,
        t::SourceProcessingError::Occurrence,
        "a captured occurrence is out of range, repeated, missing or out of position",
    ));

    let mut input = capture_input();
    input.events.remove(2);
    cases.push((
        input,
        t::SourceProcessingError::OriginalCoverage,
        "an origin's range is invalid or the captured occurrences do not cover the original source",
    ));

    let mut input = capture_input();
    input.origins[0].source = t::Source::new("xb");
    cases.push((
        input,
        t::SourceProcessingError::OriginCorrespondence,
        "the captured origins do not correspond to the original source",
    ));

    let mut input = capture_input();
    input.boundary_policies.push(t::ParagraphBoundaryPolicy {
        definition: t::ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: t::Wrap::Word,
        following_wrap: t::Wrap::Word,
        ancestor_wrap: t::Wrap::Word,
    });
    cases.push((
        input,
        t::SourceProcessingError::BoundaryPolicy,
        "a captured boundary policy is duplicated, unused or missing",
    ));

    for (input, expected, text) in cases {
        let found = capture_reason(input);
        assert_eq!(found, expected);
        assert_eq!(found.to_string(), text);
        assert!(found.source().is_none());
    }
}

#[test]
fn a_refused_joint_flow_wraps_its_reason_without_repeating_it() {
    let mut input = capture_input();
    input.joint_flows.push(t::SourceProcessingJointFlow {
        scope: 0,
        paragraphs: vec![],
        facts_owner: 0,
    });
    let found = capture_reason(input);
    let reason = t::SourceProcessingJointFlowError::EmptyMembers { declaration: 0 };
    assert_eq!(found, t::SourceProcessingError::JointFlow { reason });
    assert_eq!(
        found.to_string(),
        "a supplied joint-flow declaration is invalid"
    );
    let source = found.source().expect("the reason is the source");
    assert_eq!(
        source.to_string(),
        "joint-flow declaration 0 lists no paragraphs"
    );

    let mut input = capture_input();
    input.joint_flows.push(t::SourceProcessingJointFlow {
        scope: 7,
        paragraphs: vec![0],
        facts_owner: 0,
    });
    let found = capture_reason(input);
    assert_eq!(
        found.source().unwrap().to_string(),
        "joint-flow declaration 0 names scope 7, which does not exist"
    );
}

#[test]
fn a_refused_computed_boundary_wraps_its_reason_without_repeating_it() {
    for (scope, reason, text) in [
        (
            9,
            t::ComputedLineBoundaryError::TargetBounds,
            "the computed boundary names a target that does not exist",
        ),
        (
            0,
            t::ComputedLineBoundaryError::TargetRole,
            "the computed boundary target is not an inline box edge or an in-flow atomic box",
        ),
    ] {
        let mut input = capture_input();
        input
            .computed_line_boundaries
            .push(t::ComputedLineBoundaryInput::Scope {
                scope,
                side: t::SourceBoxSide::Before,
            });
        let found = capture_reason(input);
        assert_eq!(
            found,
            t::SourceProcessingError::ComputedLineBoundary { request: 0, reason }
        );
        assert_eq!(
            found.to_string(),
            "computed line boundary request 0 is invalid"
        );
        assert_eq!(found.source().unwrap().to_string(), text);
    }
}

// ---------------------------------------------------------------- fonts

fn table_offset(bytes: &[u8], tag: &[u8; 4]) -> usize {
    let count = usize::from(u16::from_be_bytes([bytes[4], bytes[5]]));
    (0..count)
        .map(|index| 12 + index * 16)
        .find(|record| &bytes[*record..*record + 4] == tag)
        .map(|record| {
            u32::from_be_bytes(bytes[record + 8..record + 12].try_into().unwrap()) as usize
        })
        .expect("the fixture has the table")
}

/// Registers the font bytes and returns the error of its metrics or of the
/// outline of `glyph`, as the caller of the public operation sees it.
fn font_error(bytes: Vec<u8>, glyph: Option<u32>) -> t::Error {
    let mut system = t::System::default();
    system
        .register_fonts(bytes, common::FONT_LIMITS)
        .expect("the corrupted font still registers");
    let data = system.registered_font_faces().next().unwrap().to_owned();
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    match glyph {
        Some(glyph) => instance
            .glyph_outline(glyph, system.font_catalogue_identity())
            .map(|_| ())
            .unwrap_err(),
        None => instance
            .metrics(system.font_catalogue_identity())
            .map(|_| ())
            .unwrap_err(),
    }
}

fn corrupted(tag: &[u8; 4], at: usize, bytes: &[u8]) -> Vec<u8> {
    let mut font = AHEM.to_vec();
    let start = table_offset(&font, tag) + at;
    font[start..start + bytes.len()].copy_from_slice(bytes);
    font
}

fn table_reason(error: &t::Error) -> (t::FontMetricTable, t::FontMetricTableError) {
    assert_eq!(error.code, t::ErrorCode::InvalidFontMetrics);
    match detail(error) {
        t::ErrorDetail::FontMetricTable { table, reason } => (*table, *reason),
        other => panic!("unexpected detail {other:?}"),
    }
}

#[test]
fn a_corrupted_head_table_reports_its_cause() {
    // The magic number is at byte 12 of `head`, the version at 0, units-per-em at 18.
    let (table, reason) = table_reason(&font_error(corrupted(b"head", 12, &[0; 4]), None));
    assert_eq!(table, t::FontMetricTable::Head);
    assert_eq!(reason, t::FontMetricTableError::Malformed);
    assert_eq!(reason.to_string(), "the table is truncated or malformed");

    let (_, reason) = table_reason(&font_error(corrupted(b"head", 0, &[0, 2, 0, 0]), None));
    assert_eq!(reason, t::FontMetricTableError::UnsupportedVersion);
    assert_eq!(reason.to_string(), "the table version is not supported");

    let (_, reason) = table_reason(&font_error(
        corrupted(b"head", 18, &8u16.to_be_bytes()),
        None,
    ));
    assert_eq!(
        reason,
        t::FontMetricTableError::InvalidUnitsPerEm { value: 8 }
    );
    assert_eq!(reason.to_string(), "units-per-em 8 is outside 16 to 16384");
}

#[test]
fn a_metric_with_the_wrong_sign_reports_its_requirement() {
    // The ascender is at byte 4 of `hhea`, the descender at 6.
    let error = font_error(corrupted(b"hhea", 4, &(-100i16).to_be_bytes()), None);
    let category = match detail(&error) {
        t::ErrorDetail::InvalidFontMetric { category, .. } => *category,
        other => panic!("unexpected detail {other:?}"),
    };
    assert_eq!(category, t::FontMetricNumericError::Negative);
    assert_eq!(
        category.to_string(),
        "a metric value that must not be negative is negative"
    );

    let error = font_error(corrupted(b"hhea", 6, &100i16.to_be_bytes()), None);
    let category = match detail(&error) {
        t::ErrorDetail::InvalidFontMetric { category, .. } => *category,
        other => panic!("unexpected detail {other:?}"),
    };
    assert_eq!(category, t::FontMetricNumericError::Positive);
    assert_eq!(
        category.to_string(),
        "a metric value that must not be positive is positive"
    );
}

#[test]
fn an_outline_query_reports_why_the_glyph_has_no_outline() {
    let error = font_error(AHEM.to_vec(), Some(9999));
    let reason = match detail(&error) {
        t::ErrorDetail::FontOutline { glyph, reason } => {
            assert_eq!(*glyph, 9999);
            *reason
        }
        other => panic!("unexpected detail {other:?}"),
    };
    assert_eq!(reason, t::FontOutlineError::GlyphRange { count: 281 });
    assert_eq!(
        reason.to_string(),
        "the requested glyph is outside the face's 281 glyphs"
    );

    // glyphDataFormat is at byte 52 of `head`; only format 0 is admitted.
    let error = font_error(corrupted(b"head", 52, &1i16.to_be_bytes()), Some(1));
    let reason = match detail(&error) {
        t::ErrorDetail::FontOutline { reason, .. } => *reason,
        other => panic!("unexpected detail {other:?}"),
    };
    assert_eq!(reason, t::FontOutlineError::InvalidFont);
    assert_eq!(
        reason.to_string(),
        "the font has a malformed directory, header, outline or variation source"
    );
}

// ---------------------------------------------------------------- geometry

#[test]
fn a_nonfinite_or_negative_boundary_advance_reports_its_category() {
    for (advance, category, text) in [
        (
            f64::NAN,
            t::TextNumericError::NonFinite,
            "a geometry value is not finite",
        ),
        (
            -1.0,
            t::TextNumericError::OutOfRange,
            "a geometry value is outside the domain the operation admits",
        ),
    ] {
        let error = t::InlineBoundaryAllocation::contribution(advance)
            .map(|_| ())
            .unwrap_err();
        assert_eq!(error.code, t::ErrorCode::InvalidTextNumeric);
        match detail(&error) {
            t::ErrorDetail::InvalidTextNumeric {
                field,
                category: found,
                ..
            } => {
                assert_eq!(*field, "inline boundary advance");
                assert_eq!(*found, category);
                assert_eq!(found.to_string(), text);
            }
            other => panic!("unexpected detail {other:?}"),
        }
    }
}

// ---------------------------------------------------------------- hyphenation

#[test]
fn an_unusable_hyphenation_convention_reports_its_cause() {
    let error = t::NativeHyphenationConvention::declared_script(
        "Zzzz",
        t::Text::from("-"),
        t::HyphenationPlacement::LogicalBefore,
    )
    .map(|_| ())
    .unwrap_err();
    match detail(&error) {
        t::ErrorDetail::Hyphenation { reason } => {
            assert_eq!(*reason, t::HyphenationError::InvalidConvention);
            assert_eq!(
                reason.to_string(),
                "the native hyphenation convention has an invalid or inconsistent locale or script"
            );
        }
        other => panic!("unexpected detail {other:?}"),
    }
}

#[test]
fn a_native_replacement_without_alternatives_reports_its_cause() {
    let definition = t::ParagraphReplacementDefinition::Source {
        origin: 0,
        source: t::Range::new(0, 1),
    };
    let error = t::ParagraphReplacement::resolved_native_logical_before(definition, 1, vec![])
        .map(|_| ())
        .unwrap_err();
    match detail(&error) {
        t::ErrorDetail::Hyphenation { reason } => {
            assert_eq!(*reason, t::HyphenationError::EmptyNativeAlternatives);
            assert_eq!(
                reason.to_string(),
                "the native logical-before replacement lists no alternatives"
            );
        }
        other => panic!("unexpected detail {other:?}"),
    }
}

// ---------------------------------------------------------------- captured scopes

fn style() -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        ..t::Style::default()
    }
}

fn uncaptured_preparation() -> t::PreparedText {
    let analyzed = t::AnalyzedSource::new(
        t::Source::new("AB"),
        style(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = t::System::default();
    system
        .register_fonts(AHEM.to_vec(), common::FONT_LIMITS)
        .unwrap();
    system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap()
}

#[test]
fn source_text_combination_without_a_capture_reports_the_missing_capture() {
    let combining = t::Style {
        text_combine_upright: t::TextCombineUpright::All,
        ..style()
    };
    let error = t::AnalyzedSource::new(
        t::Source::new("ab"),
        combining,
        t::AnalysisTypography::HorizontalComposition,
    )
    .map(|_| ())
    .unwrap_err();
    match detail(&error) {
        t::ErrorDetail::TextCombine { reason } => {
            assert_eq!(*reason, t::TextCombineError::MissingCapture);
            assert_eq!(
                reason.to_string(),
                "the preparation has no contextual capture for text-combine-upright"
            );
        }
        other => panic!("unexpected detail {other:?}"),
    }
}

#[test]
fn a_scope_query_on_an_uncaptured_preparation_wraps_its_reason() {
    let prepared = uncaptured_preparation();
    let segment = prepared.source_segments().next().unwrap();
    let end = segment.end_opportunity().view().boundary();
    let error = prepared
        .boundary_incidences(&segment, &end)
        .map(|_| ())
        .unwrap_err();
    let reason = match detail(&error) {
        t::ErrorDetail::InvalidTextProtocol { reason } => reason.clone(),
        other => panic!("unexpected detail {other:?}"),
    };
    assert_eq!(
        reason,
        t::ProtocolError::WrappingScopeQuery {
            reason: t::WrappingScopeQueryError::MissingCapture
        }
    );
    assert_eq!(reason.to_string(), "the wrapping scope query is invalid");
    assert_eq!(
        reason.source().unwrap().to_string(),
        "the prepared paragraph has no contextual capture"
    );
}

#[test]
fn a_scope_alignment_refusal_prints_its_cause_apart_from_the_failure() {
    let prepared = uncaptured_preparation();
    let allowance = t::ScopeAlignmentAllowance::new(1_000, 100_000);
    let census = t::scope_alignment_census(
        t::ScopeAlignmentCensusInput::Product(t::ScopeAlignmentProduct::Prepared(&prepared)),
        t::ScopeAlignmentProducts::Mixed(&[]),
        allowance,
    )
    .unwrap_err();
    let window = prepared
        .semantic_window_with_scope_allowance(
            &t::SemanticWindowRequest::new(prepared.source_segments().next().unwrap()),
            allowance,
        )
        .unwrap_err();
    for error in [census.error(), window.error()] {
        match detail(error) {
            t::ErrorDetail::ScopeAlignment { reason } => {
                assert_eq!(*reason, t::ScopeAlignmentError::MissingCapture);
                assert_eq!(
                    reason.to_string(),
                    "the operation has no contextual capture"
                );
            }
            other => panic!("unexpected detail {other:?}"),
        }
    }
}
