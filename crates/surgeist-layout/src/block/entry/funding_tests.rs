//! Real framed entry cases must use the computation's cumulative resource owner.
use super::*;
use crate::text_layout::scope_alignment::{ScopeAlignmentFacts, ScopeAlignmentResources};
use surgeist_text as text;

const CASE_LIMITS: text::scope_alignment::ScopeAlignmentLimits =
    text::scope_alignment::ScopeAlignmentLimits::new(100_000, 100, 100_000, 4 << 20);

fn captured_case() -> (
    PublicLayoutTreeOf<f64>,
    crate::text_layout::TextParagraphBinding<u32>,
) {
    use text::capture::SourceProcessingEvent as E;
    let style = crate::test_support::text::ahem_style(10.0, 10.0);
    let source = text::source::Source::new("X");
    let processed = std::sync::Arc::new(
        text::capture::process_contextual_source(text::capture::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::capture::SourceProcessingEnvelope::Complete,
            origins: vec![text::paragraph::ParagraphOrigin {
                text_node: None,
                source,
                local: text::source::Range::new(0, 1),
                aggregate: text::source::Range::new(0, 1),
                kind: text::paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(CASE_LIMITS),
            scopes: vec![text::capture::SourceProcessingScope::Block {
                topology: text::capture::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
            }],
            paragraphs: vec![text::capture::SourceProcessingParagraph {
                base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: text::layout::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = text::analyzed::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed =
        crate::test_support::text::in_flat_wrapping_context(analyzed, text::style::Wrap::Word);
    let mut system = text::System::default();
    system
        .register_fonts(
            include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
    let prepared = system
        .prepare(&analyzed, text::prepared::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let binding = crate::text_layout::TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![10],
        origins: vec![crate::text_layout::TextOriginBinding {
            origin: origin.clone(),
            node: 10,
        }],
        controls: vec![],
        atomics: vec![],
        inline_box_fragments: crate::text_layout::InlineBoxFragmentationDemand::NotRequested,
        line_strut: None,
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, (1000..1032).chain(std::iter::once(10)))
        .style(0, block(0.0, 0.0, None))
        .input(
            10,
            crate::node_input::LayoutInputOf::inline_text(
                crate::text_layout::TextParticipation::new(origin),
            ),
        );
    for node in 1000..1032 {
        tree = tree.style(
            node,
            crate::node_input::NodeInputOf {
                display: crate::node_input::Display::None,
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
                reason: text::scope_alignment::ScopeAlignmentError::Limit {
                    kind: text::scope_alignment::ScopeAlignmentLimitKind::StructuralWork, required, limit,
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
                reason: text::scope_alignment::ScopeAlignmentError::Limit {
                    kind: text::scope_alignment::ScopeAlignmentLimitKind::Records, required, limit,
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
    let capture = crate::text_layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::inline_wrapping::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let mut fresh = ComputeSession::with_flow_unobserved(&capture, Vec::new());
    assert!(
        matches!(super::super::super::observe(&mut fresh, query(EntryPurpose::FramedEntry)).unwrap(),
        EntryObservation::Demand(demand) if demand.kind == EntryFrontier::NativeRun)
    );

    let mut spent = ComputeSession::with_flow_unobserved(&capture, Vec::new());
    exhaust_work(&resources(&mut spent));
    assert!(
        matches!(super::super::super::observe(&mut spent, query(EntryPurpose::StartOnly)).unwrap(),
        EntryObservation::Demand(demand) if demand.kind == EntryFrontier::NativeRun)
    );
    let error = super::super::super::observe(&mut spent, query(EntryPurpose::FramedEntry))
        .err()
        .expect("new framed case work must refuse before traversing an exhausted owner");
    assert!(
        matches!(error.kind(), crate::error::LayoutErrorKindOf::ResourceLimit(error)
        if matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
            reason: text::scope_alignment::ScopeAlignmentError::Limit {
                kind: text::scope_alignment::ScopeAlignmentLimitKind::StructuralWork, required, limit,
            },
        }) if required > limit && *limit == CASE_LIMITS.structural_work()))
    );
}

#[test]
fn discarded_framed_case_releases_live_buffers_without_refunding_materialization() {
    let (tree, binding) = captured_case();
    let capture = crate::text_layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::inline_wrapping::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let mut spent = ComputeSession::with_flow_unobserved(&capture, Vec::new());
    let mut fresh = ComputeSession::with_flow_unobserved(&capture, Vec::new());
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
