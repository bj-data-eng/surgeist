#![forbid(unsafe_code)]

//! Messages of the three error types that already had a `Display` and wrap or
//! carry another error.
//!
//! `Error` prints its message and nothing else; its classification is read
//! through `Error::code`. The two failure types that return an `Error` from
//! `source()` state only their own context, so a reporter that walks the chain
//! prints each cause once. Every expectation is written from the documented
//! meaning of the type, not from its previous output.

use std::error::Error as StdError;

use surgeist_text as t;

mod common;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

fn assert_send_sync<E: StdError + Send + Sync + 'static>() {}

fn paragraph(value: &str) -> t::paragraph::ParagraphInput {
    let source = t::source::Source::identified(
        t::source::Text::from(value),
        t::source::Id::from_u64(65301),
        1,
    );
    let end = source.text().len();
    t::paragraph::ParagraphInput {
        source: source.clone(),
        base_style: t::style::Style {
            font: t::style::Font::new().family("Ahem"),
            size: 10.0,
            line_height: t::style::LineHeight::Absolute(10.0),
            ..t::style::Style::default()
        },
        segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
        terminal_edge: t::paragraph::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![t::paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: t::source::Range::new(0, end),
            aggregate: t::source::Range::new(0, end),
            kind: t::paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

/// A preparation made from a plain source: it has no contextual capture.
fn uncaptured_preparation() -> t::prepared::PreparedText {
    let input = paragraph("ab");
    let wrap = input.base_style.wrap;
    let analyzed = t::analyzed::AnalyzedSource::from_paragraph(
        input,
        t::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::default();
    system
        .register_fonts(AHEM.to_vec(), common::FONT_LIMITS)
        .unwrap();
    system
        .prepare(&analyzed, t::prepared::PreparationOptions::default())
        .unwrap()
}

#[test]
fn an_error_prints_its_message_without_its_code() {
    assert_send_sync::<t::Error>();
    let error = t::Error::new(t::ErrorCode::InvalidRange, "range 4..2 is reversed");
    assert_eq!(error.to_string(), "range 4..2 is reversed");
    assert_eq!(error.code, t::ErrorCode::InvalidRange);
    assert!(error.source().is_none());
    let other = t::Error::new(t::ErrorCode::LayoutFailed, "range 4..2 is reversed");
    assert_eq!(
        other.to_string(),
        error.to_string(),
        "the code is not part of the text"
    );
}

#[test]
fn an_error_from_a_public_operation_prints_its_message_only() {
    // A reversed delimiter is refused by whitespace processing.
    let source = t::source::Source::identified(
        t::source::Text::from("ab\ncd"),
        t::source::Id::from_u64(65302),
        1,
    );
    let error = t::processing::process_source_whitespace(
        source,
        t::style::Style::default(),
        t::processing::DocumentSegmentBreaks::Defined(vec![t::source::Range { start: 3, end: 2 }]),
    )
    .unwrap_err();
    assert_eq!(error.code, t::ErrorCode::InvalidRange);
    assert_eq!(error.message, "invalid document segment break");
    assert_eq!(error.to_string(), "invalid document segment break");
}

#[test]
fn an_error_reports_its_source_without_repeating_it() {
    let mut error = t::Error::new(t::ErrorCode::InvalidFontMetrics, "outer message");
    error.source = Some(Box::new(t::Error::new(
        t::ErrorCode::InvalidRange,
        "inner message",
    )));
    assert_eq!(error.to_string(), "outer message");
    let source = error.source().expect("the field is the source");
    assert_eq!(source.to_string(), "inner message");
}

#[test]
fn a_failed_census_states_only_its_own_context() {
    assert_send_sync::<t::scope_alignment::ScopeAlignmentCensusFailure>();
    let prepared = uncaptured_preparation();
    // The preparation has no contextual capture, so no allowance can fund the census.
    let failure = t::scope_alignment::scope_alignment_census(
        t::scope_alignment::ScopeAlignmentCensusInput::Product(
            t::scope_alignment::ScopeAlignmentProduct::Prepared(&prepared),
        ),
        t::scope_alignment::ScopeAlignmentProducts::Mixed(&[]),
        t::scope_alignment::ScopeAlignmentAllowance::new(1_000, 1_000_000),
    )
    .unwrap_err();
    assert_eq!(failure.error().code, t::ErrorCode::ScopeAlignmentRefused);
    assert_eq!(
        failure.to_string(),
        "the scope-alignment census was refused"
    );
    let source = failure.source().expect("the refusal is the source");
    assert_eq!(source.to_string(), failure.error().to_string());
    assert_ne!(failure.to_string(), source.to_string());
    assert!(!failure.to_string().contains(&source.to_string()));
}

#[test]
fn a_failed_semantic_window_states_only_its_own_context() {
    assert_send_sync::<t::observation::SemanticWindowFailure>();
    let prepared = uncaptured_preparation();
    let segment = prepared.source_segments().next().unwrap();
    let request = t::observation::SemanticWindowRequest::new(segment);
    // The preparation has no contextual capture, so no allowance can fund the window.
    let failure = prepared
        .semantic_window_with_scope_allowance(
            &request,
            t::scope_alignment::ScopeAlignmentAllowance::new(1_000, 1_000_000),
        )
        .unwrap_err();
    assert_eq!(failure.error().code, t::ErrorCode::ScopeAlignmentRefused);
    assert_eq!(
        failure.to_string(),
        "the semantic window observation was refused"
    );
    let source = failure.source().expect("the refusal is the source");
    assert_eq!(source.to_string(), failure.error().to_string());
    assert_ne!(failure.to_string(), source.to_string());
    assert!(!failure.to_string().contains(&source.to_string()));
}
