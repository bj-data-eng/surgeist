//! Frame admission and actual conditional geometry/selection identity.
use super::*;
use crate::{
    Alignment, ErrorDetail, LineAlignmentInsets, OverflowWrap, TextNumericError, TextNumericPhase,
};

#[test]
fn nonzero_alignment_insets_require_a_real_outer_width_but_not_a_width_bound_on_their_sum() {
    let mut system = system();
    let p = prepared(&mut system, "A", WhiteSpace::Preserve);
    let mut intrinsic = request(&p);
    intrinsic.context.available_inline = None;
    assert!(p.select(intrinsic.clone()).is_ok());
    intrinsic.context.alignment_insets = LineAlignmentInsets::try_new(1.0, 0.0).unwrap();
    let error = p.select(intrinsic).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::UnsupportedContext {
                field: ProtocolContextField::AlignmentInsets
            }
        })
    ));

    let mut overconstrained = request(&p);
    overconstrained.context.available_inline = Some(10.0);
    overconstrained.context.alignment_insets = LineAlignmentInsets::try_new(20.0, 30.0).unwrap();
    // The finite reduced frame may be negative; existing overflow-start rules,
    // rather than a new inset-sum restriction, own its actual alignment.
    assert!(
        p.select(overconstrained)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None
            })
            .is_ok()
    );
}

#[test]
fn a_finite_pair_with_unrepresentable_consumed_content_width_fails_numerically() {
    let mut system = system();
    let p = prepared(&mut system, "A", WhiteSpace::Preserve);
    let mut r = request(&p);
    r.context.available_inline = Some(1.0);
    r.context.alignment_insets = LineAlignmentInsets::try_new(f64::MAX, f64::MAX).unwrap();
    let selected = p.select(r).unwrap();
    let error = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            field: "line alignment content width",
            phase: TextNumericPhase::Finalization,
            category: TextNumericError::NonFinite,
            ..
        })
    ));
}

#[test]
fn geometry_search_identity_keeps_insets_while_conditional_selection_accepts_a_new_trial_frame() {
    let mut system = system();
    let source = Source::new("AAA");
    let analyzed = crate::AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                overflow_wrap: OverflowWrap::Anywhere,
                ..style()
            },
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            paragraph_strut: ParagraphStrut::DefaultFont,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![paragraph::ParagraphOrigin {
                text_node: None,
                source,
                local: crate::Range::new(0, 3),
                aggregate: crate::Range::new(0, 3),
                kind: paragraph::ParagraphOriginKind::Authored,
            }],
            connections: Vec::new(),
            controls: Vec::new(),
            boundary_policies: Vec::new(),
            replacements: Vec::new(),
            anchor_order: Vec::new(),
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let p = system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap();
    let mut zero = request(&p);
    zero.context.available_inline = Some(40.0);
    zero.context.alignment = Alignment::Left;
    // Actual Ahem20 gives advance60: the real40px interval overflows. A right
    // inset changes the consumed alignment frame but not overflow-Start x0.
    let mut right = zero.clone();
    right.context.alignment_insets = LineAlignmentInsets::try_new(0.0, 1.0).unwrap();
    let first = p
        .select(zero.clone())
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let second = p
        .select(right.clone())
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    for frame in [&first, &second] {
        assert_eq!(frame.measurements().alignment().advance(), 60.0);
        let xs: Vec<_> = frame
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| run.origin().x + glyph.pen() + f64::from(glyph.x()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(xs, [0.0, 20.0, 40.0]);
    }
    let PreparedTermination::Ordinary(ceiling) = &zero.termination else {
        unreachable!()
    };
    let search = |frame: &FinalizedLine| {
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &zero.start,
                None,
                ceiling,
                ConditionalPurpose::Overflow,
                &[],
                crate::ConditionalLineContext::Finalized(frame),
            )
            .unwrap()
    };
    let first_search = search(&first);
    let second_search = search(&second);
    assert_ne!(
        first_search, second_search,
        "distinct real frames cannot alias despite equal glyph positions"
    );
    assert_eq!(search(&second), second_search);
    let batch = second_search.begin(&[]).unwrap();
    let offer = batch.opportunities().first().unwrap().clone();
    let mut trial = right;
    trial.end = offer.boundary();
    let from = trial.start.view().participant_offset();
    let to = trial.end.view().participant_offset();
    trial.participants = p.participants().skip(from).take(to - from).collect();
    trial.termination = PreparedTermination::Conditional(offer.clone());
    trial.alternative = offer.alternative();
    trial.context.end = LineEnd::Soft;
    trial.context.alignment_insets = LineAlignmentInsets::try_new(2.0, 0.0).unwrap();
    let selected = p.select(trial).unwrap();
    assert_eq!(
        selected.request().context.alignment_insets,
        LineAlignmentInsets::try_new(2.0, 0.0).unwrap()
    );
    assert!(
        selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None
            })
            .is_ok()
    );
}
