//! Real framed entry cases must use the computation's cumulative resource owner.
use super::*;
use crate::text_layout::scope_alignment::{ScopeAlignmentFacts, ScopeAlignmentResources};
use surgeist_text as text;

const CASE_LIMITS: text::ScopeAlignmentLimits =
    text::ScopeAlignmentLimits::new(100_000, 100, 100_000, 4 << 20);

fn captured_case() -> (PublicLayoutTreeOf<f64>, crate::TextParagraphBinding<u32>) {
    use text::SourceProcessingEvent as E;
    let style = crate::test_support::text::ahem_style(10.0, 10.0);
    let source = text::Source::new("X");
    let processed = std::sync::Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, 1),
                aggregate: text::Range::new(0, 1),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(CASE_LIMITS),
            scopes: vec![text::SourceProcessingScope::Block {
                topology: text::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: text::ParagraphBaseDirection::LeftToRight,
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
            }],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let mut system = text::System::default();
    system
        .register_fonts(
            include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let binding = crate::TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![10],
        origins: vec![crate::TextOriginBinding {
            origin: origin.clone(),
            node: 10,
        }],
        controls: vec![],
        atomics: vec![],
        inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
        line_strut: None,
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, (1000..1032).chain(std::iter::once(10)))
        .style(0, block(0.0, 0.0, None))
        .input(
            10,
            crate::LayoutInputOf::inline_text(crate::TextParticipation::new(origin)),
        );
    for node in 1000..1032 {
        tree = tree.style(
            node,
            crate::NodeInputOf {
                display: crate::Display::None,
                ..Default::default()
            },
        );
    }
    (tree, binding)
}

fn resources(
    owner: &mut ComputeSession<'_, PublicLayoutTreeOf<f64>>,
) -> ScopeAlignmentResources<f64> {
    owner.text_computation().unwrap().scope_resources.clone()
}

fn exhaust_work(resources: &ScopeAlignmentResources<f64>) {
    for _ in 0..=CASE_LIMITS.structural_work() {
        if let Err(error) = resources.visit(CASE_LIMITS, 1) {
            assert!(
                matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
                reason: text::ScopeAlignmentError::Limit {
                    kind: text::ScopeAlignmentLimitKind::StructuralWork, required, limit,
                },
            }) if *required == CASE_LIMITS.structural_work() + 1 && *limit == CASE_LIMITS.structural_work())
            );
            return;
        }
    }
    panic!("actual one-unit work admissions must reach the captured ceiling");
}

fn remaining_records(resources: &ScopeAlignmentResources<f64>) -> usize {
    let mut rows = Vec::<()>::new();
    for _ in 0..=CASE_LIMITS.records() {
        if let Err(error) = resources.materialize(CASE_LIMITS, 0, 1) {
            assert!(
                matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
                reason: text::ScopeAlignmentError::Limit {
                    kind: text::ScopeAlignmentLimitKind::Records, required, limit,
                },
            }) if *required == CASE_LIMITS.records() + 1 && *limit == CASE_LIMITS.records())
            );
            return rows.len();
        }
        rows.push(());
    }
    panic!("actual zero-byte logical rows must reach the captured ceiling");
}

#[test]
fn framed_case_refuses_spent_work_while_start_only_retains_its_old_footprint() {
    let (tree, binding) = captured_case();
    let capture = crate::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let mut fresh = ComputeSession::with_flow(&capture, Vec::new());
    assert!(
        matches!(super::super::super::observe(&mut fresh, query(EntryPurpose::FramedEntry)).unwrap(),
        EntryObservation::Demand(demand) if demand.kind == EntryFrontier::NativeRun)
    );

    let mut spent = ComputeSession::with_flow(&capture, Vec::new());
    exhaust_work(&resources(&mut spent));
    assert!(
        matches!(super::super::super::observe(&mut spent, query(EntryPurpose::StartOnly)).unwrap(),
        EntryObservation::Demand(demand) if demand.kind == EntryFrontier::NativeRun)
    );
    let error = super::super::super::observe(&mut spent, query(EntryPurpose::FramedEntry))
        .err()
        .expect("new framed case work must refuse before traversing an exhausted owner");
    assert!(matches!(error.kind(), crate::LayoutErrorKindOf::Text(error)
        if matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
            reason: text::ScopeAlignmentError::Limit {
                kind: text::ScopeAlignmentLimitKind::StructuralWork, required, limit,
            },
        }) if required > limit && *limit == CASE_LIMITS.structural_work())));
}

#[test]
fn discarded_framed_case_releases_live_buffers_without_refunding_materialization() {
    let (tree, binding) = captured_case();
    let capture = crate::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let mut spent = ComputeSession::with_flow(&capture, Vec::new());
    let mut fresh = ComputeSession::with_flow(&capture, Vec::new());
    let spent_resources = resources(&mut spent);
    let fresh_resources = resources(&mut fresh);
    let before = spent_resources.retention();
    let demand =
        super::super::super::observe(&mut spent, query(EntryPurpose::FramedEntry)).unwrap();
    assert!(
        matches!(&demand, EntryObservation::Demand(demand) if demand.kind == EntryFrontier::NativeRun)
    );
    drop(demand);
    assert_eq!(spent_resources.retention(), before);
    let spent_remaining = remaining_records(&spent_resources);
    let fresh_remaining = remaining_records(&fresh_resources);
    assert!(
        spent_remaining < fresh_remaining,
        "discarded framed buffers must spend real materialization: spent {spent_remaining}, fresh {fresh_remaining}"
    );
}
