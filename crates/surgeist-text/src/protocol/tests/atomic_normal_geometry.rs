//! Normal canonical flow remains independent of actual visual atomic geometry.
use super::*;
use crate::{
    Affinity, AtomicBoxAllocation, AtomicFitAllocation, AtomicFlowAllocation, AtomicLineAllocation,
    AtomicPlacement, CssMarginBoxFit, ErrorDetail, Id, InlineBox, InlineBoxKind, Layout,
    LineBoxAlignment, MarginEdgeFitAuthority, Point, Range, Rect, Size, TextNumericError,
    TextNumericPhase,
};

fn fixture() -> (PreparedText, PreparedAtomic) {
    let mut source = Source::new("");
    source.inline_box(InlineBox::new(
        Id::from_u64(1_678_001),
        InlineBoxKind::InFlow,
        0,
        Size::new(10.4, 12.0),
    ));
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Unused atomic-only face"),
                ..Style::default()
            },
            paragraph_strut: ParagraphStrut::AllocatedContributions,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![crate::ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, 0),
                aggregate: Range::new(0, 0),
                kind: crate::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            anchor_order: vec![
                crate::ParagraphAnchor {
                    item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                    affinity: Affinity::Before,
                },
                crate::ParagraphAnchor {
                    item: crate::ParagraphAnchorItem::Atomic(0),
                    affinity: Affinity::After,
                },
            ],
            boundary_policies: [
                crate::ParagraphBoundaryDefinition::AtomicBefore(0),
                crate::ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| crate::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = System::default();
    assert_eq!(system.registered_font_faces().count(), 0);
    let p = system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap();
    let atomic = {
        let mut atomics = p.atomic_views();
        let atomic = atomics.next().unwrap().handle();
        assert!(atomics.next().is_none());
        atomic
    };
    assert_eq!(atomic.view().original_box().size(), Size::new(10.4, 12.0));
    (p, atomic)
}

fn allocation() -> AtomicLineAllocation {
    // The caller supplies actual normal canonical10 and independently snapped
    // visual11x13. Only normal8/4 contributes to the line's twelve-unit height.
    AtomicLineAllocation::InFlow {
        flow: AtomicFlowAllocation {
            advance: 10.0,
            normal_border_inline_size: 10.0,
            before_baseline: 8.0,
            after_baseline: 4.0,
            occupied_start: 0.0,
            occupied_end: 10.0,
        },
        fit: AtomicFitAllocation::CssMarginBox(CssMarginBoxFit::try_new(0.0, 10.0, 0.0).unwrap()),
        box_geometry: AtomicBoxAllocation {
            border: Rect::new(0.0, -8.0, 11.0, 13.0),
            content: Rect::new(1.0, -7.0, 9.0, 11.0),
            baseline: 0.0,
        },
        alignment: LineBoxAlignment::Baseline,
    }
}
fn input(
    p: &PreparedText,
    atomic: &PreparedAtomic,
    allocation: AtomicLineAllocation,
) -> LineSelectionRequest {
    let mut r = request(p);
    r.context.spacing = SpacingPolicy::None;
    r.atomics = vec![AtomicAllocation {
        occurrence: atomic.clone(),
        allocation,
    }];
    r
}
fn finish(p: &PreparedText, r: LineSelectionRequest) -> FinalizedLine {
    p.select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}
fn invalid_atomic(error: &crate::Error) {
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::InvalidAtomicGeometry { occurrence: 0 },
        })
    );
}
fn placement(p: &PreparedText, line: &FinalizedLine, actual: AtomicPlacement) -> PlacementRequest {
    PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 30.0,
                baseline: 40.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![actual],
        })],
    }
}

#[test]
fn normal_border_ten_with_visual_eleven_preserves_flow_metrics_and_exact_placement() {
    let (p, atomic) = fixture();
    let line = finish(&p, input(&p, &atomic, allocation()));
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (10.0, 8.0, 12.0)
    );
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
    let fit = line.measurements().margin_edge_fit();
    assert_eq!(fit.authority(), MarginEdgeFitAuthority::CssComposed);
    assert_eq!(fit.required_inline_size(), 10.0);
    let slot = line.atomic_slots().next().unwrap();
    assert_eq!(slot.occurrence, atomic);
    assert_eq!(slot.origin, Point::new(0.0, 8.0));
    assert_eq!(slot.allocation, allocation());
    let actual = AtomicPlacement {
        occurrence: atomic.clone(),
        border: Rect::new(30.0, 32.0, 11.0, 13.0),
        content: Rect::new(31.0, 33.0, 9.0, 11.0),
        baseline: 40.0,
    };
    let placed = Layout::from_finalized_lines(placement(&p, &line, actual.clone())).unwrap();
    let boxes = placed.inline_boxes();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].id(), Id::from_u64(1_678_001));
    assert_eq!(boxes[0].rect(), actual.border);
    // A normal-size rectangle cannot stand in for this occurrence's VISUAL box.
    let wrong = AtomicPlacement {
        border: Rect::new(30.0, 32.0, 10.0, 13.0),
        ..actual
    };
    let error = Layout::from_finalized_lines(placement(&p, &line, wrong)).unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::AtomicPlacementMismatch {
                line: 0,
                occurrence: 0
            },
        })
    );
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(
        replay.atomic_slots().next().unwrap().allocation,
        allocation()
    );
    assert_eq!(replay.feedback(), line.feedback());
    assert_eq!(placed.inline_boxes()[0].rect(), boxes[0].rect());
}

#[test]
fn css_receipt_cannot_match_visual_width_instead_of_normal_border() {
    let (p, atomic) = fixture();
    let mut wrong = allocation();
    let AtomicLineAllocation::InFlow { flow, fit, .. } = &mut wrong else {
        unreachable!()
    };
    // Every old equality now agrees with the11-wide visual box; ONLY the real
    // normal border10 disagrees with this receipt's claimed normal border11.
    *fit = AtomicFitAllocation::CssMarginBox(CssMarginBoxFit::try_new(0.0, 11.0, 0.0).unwrap());
    flow.advance = 11.0;
    flow.occupied_end = 11.0;
    invalid_atomic(&p.select(input(&p, &atomic, wrong)).unwrap_err());
    assert_eq!(
        finish(&p, input(&p, &atomic, allocation()))
            .feedback()
            .advance,
        10.0
    );
}

#[test]
fn normal_border_requires_finite_nonnegative_extent_and_visual_checks_remain_active() {
    let (p, atomic) = fixture();
    let prior = finish(&p, input(&p, &atomic, allocation()));
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut wrong = allocation();
        let AtomicLineAllocation::InFlow { flow, .. } = &mut wrong else {
            unreachable!()
        };
        flow.normal_border_inline_size = value;
        let error = p.select(input(&p, &atomic, wrong)).unwrap_err();
        assert!(matches!(
            error.detail(),
            Some(ErrorDetail::InvalidTextNumeric {
                field: "atomic normal border inline size",
                phase: TextNumericPhase::Selection,
                category: TextNumericError::NonFinite,
                ..
            })
        ));
    }
    let mut negative = allocation();
    let AtomicLineAllocation::InFlow { flow, .. } = &mut negative else {
        unreachable!()
    };
    flow.normal_border_inline_size = -1.0;
    invalid_atomic(&p.select(input(&p, &atomic, negative)).unwrap_err());
    for content in [false, true] {
        let mut wrong = allocation();
        let AtomicLineAllocation::InFlow { box_geometry, .. } = &mut wrong else {
            unreachable!()
        };
        if content {
            box_geometry.content.size.width = -1.0;
        } else {
            box_geometry.border.size.width = -1.0;
        }
        invalid_atomic(&p.select(input(&p, &atomic, wrong)).unwrap_err());
    }
    let mut baseline = allocation();
    let AtomicLineAllocation::InFlow { box_geometry, .. } = &mut baseline else {
        unreachable!()
    };
    box_geometry.baseline = f64::NAN;
    assert!(matches!(
        p.select(input(&p, &atomic, baseline)).unwrap_err().detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            field: "atomic baseline",
            category: TextNumericError::NonFinite,
            ..
        })
    ));
    assert_eq!(
        (prior.feedback().advance, prior.feedback().line_height),
        (10.0, 12.0)
    );
}

#[test]
fn normal_border_is_retained_in_native_request_identity_without_acquiring_css_authority() {
    let (p, atomic) = fixture();
    let mut left = allocation();
    let AtomicLineAllocation::InFlow { fit, flow, .. } = &mut left else {
        unreachable!()
    };
    *fit = AtomicFitAllocation::NativeOccupied;
    flow.occupied_end = 11.0; // Actual opaque occupied authority, independent of border/advance.
    let mut right = left;
    let AtomicLineAllocation::InFlow { flow, .. } = &mut right else {
        unreachable!()
    };
    flow.normal_border_inline_size = 11.0;
    let a = input(&p, &atomic, left);
    let b = input(&p, &atomic, right);
    let key_a = crate::protocol::conditional::request_context((&a).into());
    let key_b = crate::protocol::conditional::request_context((&b).into());
    assert_ne!(
        key_a, key_b,
        "the real normal fact participates in conditional context"
    );
    assert_ne!(a, b);
    for (r, expected) in [(a, left), (b, right)] {
        let line = finish(&p, r);
        assert_eq!(line.atomic_slots().next().unwrap().allocation, expected);
        let fit = line.measurements().margin_edge_fit();
        assert_eq!(fit.authority(), MarginEdgeFitAuthority::NativeOnly);
        assert_eq!(fit.required_inline_size(), 11.0);
    }
}

#[test]
fn signed_css_margins_keep_normal_border_distinct_from_advance_and_visual_width() {
    let (p, atomic) = fixture();
    let mut actual = allocation();
    let AtomicLineAllocation::InFlow {
        flow,
        fit,
        box_geometry,
        ..
    } = &mut actual
    else {
        unreachable!()
    };
    *fit = AtomicFitAllocation::CssMarginBox(CssMarginBoxFit::try_new(3.0, 10.0, -5.0).unwrap());
    flow.advance = 8.0;
    flow.occupied_end = 13.0;
    box_geometry.border.origin.x = 3.0;
    box_geometry.content.origin.x = 4.0;
    let line = finish(&p, input(&p, &atomic, actual));
    assert_eq!(line.feedback().advance, 8.0);
    assert_eq!(line.feedback().line_height, 12.0);
    assert_eq!(
        line.measurements().margin_edge_fit().required_inline_size(),
        8.0
    );
    assert_eq!(line.atomic_slots().next().unwrap().allocation, actual);
}
