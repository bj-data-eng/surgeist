//! Allocated children remain physical; positive-over is left for the CCW line.
use super::*;
use crate::{
    AtomicBoxAllocation, AtomicFlowAllocation, AtomicLineAllocation, BaselineShift, InlineBox,
    InlineBoxKind, LineBoxAlignment,
};

const INFLOW: Id = Id::from_u64(1_579_011);
const POSITIONED: Id = Id::from_u64(1_579_012);

fn atomic_input() -> ParagraphInput {
    let mut src = source("A");
    src.inline_box(InlineBox::new(
        INFLOW,
        InlineBoxKind::InFlow,
        1,
        Size::new(6.0, 14.0),
    ));
    src.inline_box(InlineBox::new(
        POSITIONED,
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(60.0, 140.0),
    ));
    let mut value = paragraph(src, own_style(), false);
    value.anchor_order = [1, 0]
        .into_iter()
        .map(|index| crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Atomic(index),
            affinity: Affinity::Before,
        })
        .collect();
    value.boundary_policies = (0..2)
        .flat_map(|index| {
            [
                crate::ParagraphBoundaryDefinition::AtomicBefore(index),
                crate::ParagraphBoundaryDefinition::AtomicAfter(index),
            ]
        })
        .map(|definition| crate::ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    value
}
fn allocation(shift: f64) -> AtomicLineAllocation {
    AtomicLineAllocation::InFlow {
        fit: crate::AtomicFitAllocation::NativeOccupied,
        flow: AtomicFlowAllocation {
            advance: 14.0,
            before_baseline: 2.0,
            after_baseline: 4.0,
            occupied_start: 0.0,
            occupied_end: 14.0,
        },
        box_geometry: AtomicBoxAllocation {
            border: Rect::new(0.0, -2.0, 14.0, 6.0),
            content: Rect::new(2.0, -1.0, 10.0, 4.0),
            baseline: 0.0,
        },
        alignment: LineBoxAlignment::Shift(BaselineShift::try_new(shift).unwrap()),
    }
}
fn finalized(p: &PreparedText, shift: f64) -> FinalizedLine {
    let mut r = selected_request(p, start(p), terminal(p));
    r.atomics = p
        .atomic_views()
        .map(|view| AtomicAllocation {
            occurrence: view.handle(),
            allocation: if view.original_box().kind() == InlineBoxKind::OutOfFlow {
                AtomicLineAllocation::OutOfFlow
            } else {
                allocation(shift)
            },
        })
        .collect();
    p.select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}
fn atomic_placement(p: &PreparedText, line: FinalizedLine, shift: f64) -> PlacementRequest {
    let atomics = p
        .atomic_views()
        .map(|view| {
            if view.original_box().kind() == InlineBoxKind::OutOfFlow {
                AtomicPlacement {
                    occurrence: view.handle(),
                    border: Rect::new(300.0, 500.0, 60.0, 140.0),
                    content: Rect::new(305.0, 507.0, 48.0, 126.0),
                    baseline: 321.0,
                }
            } else {
                // Local allocated border(14inline,6cross) maps to an UNROTATED
                // physical child6x14. u10..24, v6-shift..12-shift; B8.
                AtomicPlacement {
                    occurrence: view.handle(),
                    border: Rect::new(98.0 - shift, -4.0, 6.0, 14.0),
                    content: Rect::new(99.0 - shift, -2.0, 4.0, 10.0),
                    baseline: 100.0 - shift,
                }
            }
        })
        .collect();
    PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: CCW,
            atomics,
        })],
    }
}

#[test]
fn shifted_inflow_boxes_move_left_while_positioned_boxes_keep_their_actual_physical_allocation() {
    let mut system = system_with(AHEM.to_vec());
    let p = prepare(&mut system, atomic_input(), CCW);
    assert_eq!(
        p.participants()
            .filter_map(|part| match part {
                PreparedParticipant::Atomic(a) => Some(a.view().original_box().id()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [POSITIONED, INFLOW]
    );
    for shift in [-1.0, 0.0, 1.0] {
        let line = finalized(&p, shift);
        let height = 12.0 - shift;
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (24.0, 8.0, height)
        );
        for slot in line.atomic_slots() {
            let actual = slot.occurrence.view().original_box().kind();
            assert_eq!(
                slot.origin,
                Point::new(
                    10.0,
                    if actual == InlineBoxKind::InFlow {
                        8.0 - shift
                    } else {
                        8.0
                    }
                )
            );
        }
        let layout =
            crate::Layout::from_finalized_lines(atomic_placement(&p, line.clone(), shift)).unwrap();
        assert_eq!(
            layout.lines()[0].bounds(),
            Rect::new(92.0, -4.0, height, 24.0)
        );
        let boxes = layout.inline_boxes();
        assert_eq!(
            boxes.iter().find(|b| b.id() == INFLOW).unwrap().rect(),
            Rect::new(98.0 - shift, -4.0, 6.0, 14.0)
        );
        assert_eq!(
            boxes.iter().find(|b| b.id() == POSITIONED).unwrap().rect(),
            Rect::new(300.0, 500.0, 60.0, 140.0)
        );
        assert_eq!(
            layout.hit(Point::new(100.0 - shift, 0.0)),
            Hit::InlineBox(INFLOW)
        );
        assert_eq!(
            layout.hit(Point::new(310.0, 510.0)),
            Hit::InlineBox(POSITIONED)
        );
        assert_eq!(
            layout.run_views().next().unwrap().origin(),
            Point::new(100.0, 20.0)
        );
        assert_eq!(
            layout.run_views().next().unwrap().glyph_basis(),
            GlyphBasis::Counterclockwise
        );
        assert_eq!(
            line.with_spacing(SpacingAllocations::None)
                .unwrap()
                .selected_line(),
            line.selected_line()
        );
        // A missing physical occurrence refuses without replacing the old value.
        let mut missing = atomic_placement(&p, line, shift);
        let crate::PlacementEntry::Formatted(row) = &mut missing.entries[0] else {
            unreachable!()
        };
        row.atomics.pop();
        assert!(matches!(
            reason(&crate::Layout::from_finalized_lines(missing).unwrap_err()),
            ProtocolError::MissingAtomic { .. }
        ));
        assert_eq!(
            layout
                .inline_boxes()
                .iter()
                .find(|b| b.id() == INFLOW)
                .unwrap()
                .rect(),
            Rect::new(98.0 - shift, -4.0, 6.0, 14.0)
        );
    }
}

fn controls(p: &PreparedText, shift: f64) -> FinalizedLine {
    let mut r = selected_request(p, start(p), terminal(p));
    for row in &mut r.controls {
        row.boundary = Some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO);
        if *row.occurrence.view().kind() == crate::ParagraphControlKind::InlineBoundaryStart {
            row.strut = Some(crate::ControlStrut {
                baseline: 2.0,
                line_extent: 3.0,
                alignment: LineBoxAlignment::Shift(BaselineShift::try_new(shift).unwrap()),
            });
        }
    }
    while let Some(occurrence) = p.boundary_context_demand(&r).unwrap() {
        r.boundary_context
            .push(crate::InlineBoundaryContextAllocation {
                occurrence,
                boundary: crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO,
            });
    }
    p.select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

#[test]
fn native_control_shift_uses_leftward_over_even_at_asymmetric_f64_spacing_boundary() {
    let mut system = system_with(AHEM.to_vec());
    let mut value = paragraph(source("A"), own_style(), false);
    value.controls = vec![
        crate::ParagraphControl {
            at: 0,
            kind: crate::ParagraphControlKind::InlineBoundaryStart,
        },
        crate::ParagraphControl {
            at: 1,
            kind: crate::ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    value.anchor_order = (0..2)
        .map(|control| crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Control(control),
            affinity: Affinity::After,
        })
        .collect();
    let p = prepare(&mut system, value, CCW);
    let control = p
        .control_views()
        .find(|c| *c.kind() == crate::ParagraphControlKind::InlineBoundaryStart)
        .unwrap()
        .handle();
    for shift in [-1.0, 0.0, 1.0] {
        let line = controls(&p, shift);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (8.0, 10.0)
        );
        let slot = line
            .control_slots()
            .find(|slot| slot.control() == control)
            .unwrap();
        assert_eq!(slot.slot(), Point::new(0.0, 8.0 - shift));
        let layout = placed(&p, vec![(line, 100.25, -20.5)]);
        assert_eq!(layout.bounds(), Rect::new(92.25, -30.5, 10.0, 10.0));
    }
    // At2^53, x-1 is representable while x+1 is absorbed. Actual control
    // placement therefore distinguishes CCW positive-over LEFT from CW right.
    // Font bounds x-8..x+2 themselves remain exactly representable.
    let x = 9_007_199_254_740_992.0;
    let old = placed(&p, vec![(controls(&p, 1.0), x, 100.0)]);
    assert_eq!(old.lines()[0].baseline(), x);
    let error = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: entries(&p, vec![(controls(&p, -1.0), x, 100.0)]),
    })
    .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            category: crate::TextNumericError::NonZeroLost,
            ..
        })
    ));
    assert_eq!(old.lines()[0].baseline(), x);
}
