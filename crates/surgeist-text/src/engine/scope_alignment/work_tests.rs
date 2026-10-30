//! Focused structural-work admission over authentic prepared source custody.
//!
//! The isolated ledger here covers only the borrowed helper invocation. It is
//! not the ledger or a resource claim for the complete preparation operation.
//! Counts follow the adopted slot/owner/containment visits, not glyph counts or
//! the number of times shaping chooses to invoke or hoist these helpers.
use super::{CustodyInput, accounting};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use crate::*;
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

struct Fixture {
    system: System,
    prepared: PreparedText,
    domain: CapturedScope,
    leaf: CapturedScope,
}

impl Fixture {
    fn custody(&self) -> CustodyInput<'_> {
        self.prepared
            .inner
            .scope_alignment
            .custody(&self.prepared.inner.analyzed)
            .expect("the real preparation contains two captured alignment targets")
    }
}

fn fixture() -> Fixture {
    let style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    };
    let reference = ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    };
    let mut source = Source::identified("XXX", Id::from_u64(1578070), 41);
    source.span(Range::new(0, 3), style.clone());
    let input = ContextualSourceInput {
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        scope_alignment_limits: Some(ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216)),
        origins: vec![
            ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(0, 2),
                aggregate: Range::new(0, 2),
                kind: ParagraphOriginKind::Authored,
            },
            ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(2, 3),
                aggregate: Range::new(2, 3),
                kind: ParagraphOriginKind::Authored,
            },
        ],
        connections: vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: ParagraphConnectionKind::Continuous,
        }],
        boundary_policies: vec![ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        }],
        controls: vec![],
        replacements: vec![],
        computed_line_boundaries: vec![],
        scopes: vec![
            SourceProcessingScope::Block {
                topology: SourceProcessingTopology::Complete,
                trim: WhiteSpaceTrim::default(),
                base_style: style.clone(),
                used_direction: ParagraphBaseDirection::LeftToRight,
                font_reference: Some(reference),
            },
            SourceProcessingScope::Inline {
                trim: WhiteSpaceTrim::default(),
                style: Some(style.clone()),
                boundaries: None,
                alignment_boundary: Some(ScopeAlignmentBoundary::Initial),
                font_reference: Some(reference),
            },
            SourceProcessingScope::Inline {
                trim: WhiteSpaceTrim::default(),
                style: Some(style),
                boundaries: None,
                alignment_boundary: Some(ScopeAlignmentBoundary::NonInitial),
                font_reference: Some(reference),
            },
        ],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::AllocatedContributions,
        }],
        events: vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::EnterScope(2),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitScope(2),
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ],
    };
    let captured = Arc::new(process_contextual_source(input).unwrap());
    let domain = captured.captured_scope(0).unwrap();
    let leaf = captured.captured_scope(2).unwrap();
    let analyzed = AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    Fixture {
        system,
        prepared,
        domain,
        leaf,
    }
}

fn ledger(work: usize) -> accounting::Ledger {
    Some(ScopeAlignmentBudget::new(ScopeAlignmentLimits::new(0, 0, work, 0), 0, 0).unwrap())
}

fn usage(ledger: &accounting::Ledger, work: usize) {
    let usage = ledger.as_ref().unwrap().usage();
    assert_eq!(usage.structural_work(), work);
    assert_eq!(usage.font_queries(), 0);
    assert_eq!(usage.records(), 0);
    assert_eq!(usage.payload_bytes(), 0);
    assert_eq!(usage.peak_payload_bytes(), 0);
}

fn work_refusal(error: Error, required: usize, limit: usize) {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    let Some(ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("expected precise scope resource refusal, got {error:?}")
    };
    assert_eq!(
        *reason,
        ScopeAlignmentError::Limit {
            kind: ScopeAlignmentLimitKind::StructuralWork,
            required,
            limit,
        }
    );
}

#[test]
fn outside_source_scan_refuses_without_each_slot_owner_and_comparison_visit() {
    let fixture = fixture();
    let stats = fixture.system.stats();
    let prepared_usage = fixture.prepared.scope_alignment_usage();
    let custody = fixture.custody();
    // Each outside X has one authentic domain owner. Neither of the two inline
    // targets contains that owner: 2 * (slot + owner + 2 comparisons) = 8.
    // A refused increment leaves the already admitted usage unchanged.
    for limit in [0, 3, 7] {
        let mut budget = ledger(limit);
        work_refusal(
            custody.observes(0..2, &mut budget).unwrap_err(),
            limit + 1,
            limit,
        );
        usage(&budget, limit);
    }
    let mut budget = ledger(8);
    assert!(!custody.observes(0..2, &mut budget).unwrap());
    usage(&budget, 8);
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(fixture.prepared.scope_alignment_usage(), prepared_usage);
}

#[test]
fn observed_source_stops_at_first_matching_target_and_empty_range_visits_nothing() {
    let fixture = fixture();
    let stats = fixture.system.stats();
    let prepared_usage = fixture.prepared.scope_alignment_usage();
    let custody = fixture.custody();
    // The third X belongs to the inner scope. Both real inline targets contain
    // it, so the first comparison succeeds irrespective of their ordering.
    let mut budget = ledger(2);
    work_refusal(custody.observes(2..3, &mut budget).unwrap_err(), 3, 2);
    usage(&budget, 2);
    let mut budget = ledger(3);
    assert!(custody.observes(2..3, &mut budget).unwrap());
    usage(&budget, 3);
    let mut budget = ledger(0);
    assert!(!custody.observes(2..2, &mut budget).unwrap());
    usage(&budget, 0);
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(fixture.prepared.scope_alignment_usage(), prepared_usage);
}

#[test]
fn target_lookup_requires_all_candidates_to_choose_the_actual_inner_scope() {
    let fixture = fixture();
    let stats = fixture.system.stats();
    let prepared_usage = fixture.prepared.scope_alignment_usage();
    let custody = fixture.custody();
    // The supplied owner is an authentic captured scope, not an injected index
    // or a fabricated owner/target vector. Both targets match; the later-entered
    // inner scope is the nearest target. A negative lookup also visits both.
    let inner = fixture.leaf.capture_ordinal();
    for limit in [0, 1] {
        let mut budget = ledger(limit);
        work_refusal(
            custody.target(inner, &mut budget).unwrap_err(),
            limit + 1,
            limit,
        );
        usage(&budget, limit);
    }
    let mut budget = ledger(2);
    assert_eq!(custody.target(inner, &mut budget).unwrap(), Some(inner));
    usage(&budget, 2);
    let mut budget = ledger(2);
    assert_eq!(
        custody
            .target(fixture.domain.capture_ordinal(), &mut budget)
            .unwrap(),
        None
    );
    usage(&budget, 2);
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(fixture.prepared.scope_alignment_usage(), prepared_usage);
}

#[test]
fn authentic_owner_slot_access_requires_one_work_unit() {
    let fixture = fixture();
    let stats = fixture.system.stats();
    let prepared_usage = fixture.prepared.scope_alignment_usage();
    let custody = fixture.custody();
    let mut budget = ledger(0);
    work_refusal(custody.input_owners(2, &mut budget).unwrap_err(), 1, 0);
    usage(&budget, 0);
    let mut budget = ledger(1);
    assert_eq!(
        custody.input_owners(2, &mut budget).unwrap(),
        [fixture.leaf.capture_ordinal()]
    );
    usage(&budget, 1);
    let mut budget = ledger(1);
    assert_eq!(
        custody.input_owners(0, &mut budget).unwrap(),
        [fixture.domain.capture_ordinal()]
    );
    usage(&budget, 1);
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(fixture.prepared.scope_alignment_usage(), prepared_usage);
}
