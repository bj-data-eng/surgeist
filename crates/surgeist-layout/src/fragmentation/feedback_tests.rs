use super::scheduler::resolve_with_feedback;
use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct Provider<S: LayoutScalar> {
    committed: Rc<Cell<bool>>,
    deferred: bool,
    queried_successor: bool,
    _scalar: core::marker::PhantomData<S>,
}
impl<S: LayoutScalar> FragmentationProvider<S> for Provider<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(FragmentainerOf {
            inline_size: S::from_f64(100.0),
            block_size: S::from_f64(if index == 0 { 40.0 } else { 80.0 }),
            breaks: if index == 0 {
                FragmentationBreakKinds::NONE
            } else {
                FragmentationBreakKinds::PAGE
            },
            parent_fragmentainer: None,
            page_side: None,
        }))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        if query.fragmentainer_index == 1 {
            self.queried_successor = true;
            if !self.committed.get() {
                return Err("next query preceded acceptance feedback");
            }
        }
        if self.deferred && query.scan_after.is_none() {
            return Ok(FragmentationReplyOf::ControlAtStart(
                FragmentationBoundaryControls {
                    kind: FragmentationControlKind::Sibling,
                    before: vec![FragmentationBreakControl {
                        value: FragmentationBreak::Column,
                        role: FragmentationRole::Normal,
                        source_order: 0,
                    }],
                    after: Vec::new(),
                },
            ));
        }
        let start = query.position.occurrence;
        let count = if self.deferred { 1 } else { 2 };
        let head = if self.deferred { 50.0 } else { 30.0 };
        let opportunities = if !self.deferred && start == 0 {
            vec![FragmentationOpportunityOf {
                cut: FragmentationCutOf::Source(FragmentationPosition {
                    occurrence: 1,
                    offset: 0,
                }),
                block_offset: S::from_f64(30.0),
                kind: FragmentationOpportunityKindOf::Sibling,
                before: Vec::new(),
                after: Vec::new(),
                ancestors: Vec::new(),
            }]
        } else {
            Vec::new()
        };
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: S::from_f64(if start == 0 && count == 2 { 60.0 } else { head }),
            head: FragmentationMeasuredHeadOf {
                end: FragmentationPosition {
                    occurrence: start + 1,
                    offset: 0,
                },
                block_size: S::from_f64(head),
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities,
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn input<S: LayoutScalar>(deferred: bool) -> FragmentationInputOf<usize, S> {
    FragmentationInputOf {
        contexts: vec![FragmentationContext {
            kind: FragmentationKind::Page {
                progression: crate::PageProgression::LeftToRight,
            },
            parent: None,
            flow_axes: crate::FlowAxes::new(
                crate::WritingMode::HorizontalTb,
                crate::Direction::Ltr,
            ),
        }],
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: 0,
            initial_block_offset: if deferred { S::from_f64(10.0) } else { S::ZERO },
            occurrences: (0..if deferred { 1 } else { 2 })
                .map(|key| FragmentationOccurrence {
                    key,
                    source_extent: 1,
                    monolithic: true,
                })
                .collect(),
        }],
    }
}
fn committed_before_successor<S: LayoutScalar>() {
    for deferred in [false, true] {
        let committed = Rc::new(Cell::new(false));
        let mut provider = Provider::<S> {
            committed: committed.clone(),
            deferred,
            queried_successor: false,
            _scalar: core::marker::PhantomData,
        };
        let seen = RefCell::new(Vec::new());
        let report = resolve_with_feedback(&input(deferred), &mut provider, |event| {
            seen.borrow_mut().push((
                event.from_frame,
                event.to_frame,
                event
                    .pieces
                    .iter()
                    .map(|piece| piece.block_size)
                    .collect::<Vec<_>>(),
                event.flow_complete,
            ));
            committed.set(true);
            Ok(())
        })
        .unwrap();
        assert!(provider.queried_successor);
        assert_eq!(seen.borrow().len(), 2);
        assert_eq!(seen.borrow()[0].0, 0);
        assert_eq!(seen.borrow()[0].1, 1);
        assert!(!seen.borrow()[0].3);
        assert!(seen.borrow()[1].3);
        if deferred {
            // Ignored discovery and the occupied-frame deferral accept no
            // source content. Only the fresh frame contributes a piece.
            assert!(seen.borrow()[0].2.is_empty());
            assert_eq!(report.pieces.len(), 1);
            assert_eq!(report.pieces[0].block_size, S::from_f64(50.0));
            assert_eq!(report.breaks[0].reason, FragmentationBreakReason::Deferred);
        } else {
            assert_eq!(seen.borrow()[0].2, [S::from_f64(30.0)]);
            assert_eq!(report.pieces.len(), 2);
            assert!(
                report
                    .pieces
                    .iter()
                    .all(|piece| piece.stage == FragmentationStage::Strict)
            );
        }
    }
}
fn feedback_failure_is_atomic<S: LayoutScalar>() {
    let mut provider = Provider::<S> {
        committed: Rc::new(Cell::new(false)),
        deferred: false,
        queried_successor: false,
        _scalar: core::marker::PhantomData,
    };
    let result = resolve_with_feedback(&input(false), &mut provider, |_| {
        Err("accepted production failed")
    });
    assert_eq!(
        result,
        Err(FragmentationError::Provider("accepted production failed"))
    );
    assert!(!provider.queried_successor);
}
#[test]
fn accepted_step_precedes_next_query_f32() {
    committed_before_successor::<f32>();
}
#[test]
fn accepted_step_precedes_next_query_f64() {
    committed_before_successor::<f64>();
}
#[test]
fn accepted_step_failure_returns_no_report_f32() {
    feedback_failure_is_atomic::<f32>();
}
#[test]
fn accepted_step_failure_returns_no_report_f64() {
    feedback_failure_is_atomic::<f64>();
}

struct CutProvider {
    graphical_first: bool,
    raw_offset: bool,
}
impl<S: LayoutScalar> FragmentationProvider<S> for CutProvider {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(FragmentainerOf {
            inline_size: S::from_f64(100.0),
            block_size: S::from_f64(40.0),
            breaks: if index == 0 {
                FragmentationBreakKinds::NONE
            } else {
                FragmentationBreakKinds::PAGE
            },
            parent_fragmentainer: None,
            page_side: None,
        }))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let first = query.position.occurrence == 0;
        let anchor = FragmentationPosition {
            occurrence: if self.raw_offset { 0 } else { 1 },
            offset: usize::from(self.raw_offset),
        };
        let mut opportunities = if first {
            vec![
                FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Source(FragmentationPosition {
                        occurrence: 1,
                        offset: 0,
                    }),
                    block_offset: S::from_f64(40.0),
                    kind: FragmentationOpportunityKindOf::ContentEdge {
                        gap: S::from_f64(20.0),
                    },
                    before: Vec::new(),
                    after: Vec::new(),
                    ancestors: Vec::new(),
                },
                FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Graphical {
                        anchor,
                        progress: FragmentationGraphicalProgressOf {
                            consumed: S::from_f64(10.0),
                            extent: S::from_f64(20.0),
                        },
                    },
                    block_offset: S::from_f64(40.0),
                    kind: FragmentationOpportunityKindOf::ContentEdge {
                        gap: S::from_f64(20.0),
                    },
                    before: Vec::new(),
                    after: Vec::new(),
                    ancestors: Vec::new(),
                },
            ]
        } else {
            Vec::new()
        };
        if self.graphical_first {
            opportunities.reverse();
        }
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: S::from_f64(if first { 50.0 } else { 10.0 }),
            head: FragmentationMeasuredHeadOf {
                end: FragmentationPosition {
                    occurrence: if first { 1 } else { 2 },
                    offset: 0,
                },
                block_size: S::from_f64(if first { 30.0 } else { 10.0 }),
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities,
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn cut_identity<S: LayoutScalar>() {
    let input = FragmentationInputOf {
        contexts: input::<S>(false).contexts,
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: 0,
            initial_block_offset: S::ZERO,
            occurrences: vec![
                FragmentationOccurrence {
                    key: 10,
                    source_extent: 1,
                    monolithic: true,
                },
                FragmentationOccurrence {
                    key: 11,
                    source_extent: 0,
                    monolithic: false,
                },
            ],
        }],
    };
    for graphical_first in [false, true] {
        let report = resolve_fragmentation(
            &input,
            &mut CutProvider {
                graphical_first,
                raw_offset: false,
            },
        )
        .unwrap();
        assert_eq!(
            report.pieces[0].end,
            FragmentationPosition {
                occurrence: 1,
                offset: 0
            }
        );
        assert!(
            matches!(report.pieces[0].kind, FragmentationPieceKindOf::GraphicalSlice { anchor, .. }
            if anchor == FragmentationPosition { occurrence: 1, offset: 0 })
        );
        assert_eq!(
            report.pieces[0]
                .coverage
                .iter()
                .map(|item| item.key)
                .collect::<Vec<_>>(),
            [10, 11]
        );
        assert_eq!(
            report.pieces[1]
                .coverage
                .iter()
                .map(|item| item.key)
                .collect::<Vec<_>>(),
            [11]
        );
    }
    // A source endpoint that normalizes to the following zero-source box is
    // not an authorized graphical anchor for that different occurrence.
    let result = resolve_fragmentation(
        &input,
        &mut CutProvider {
            graphical_first: false,
            raw_offset: true,
        },
    );
    assert_eq!(
        result,
        Err(FragmentationError::InvalidInput(
            FragmentationInputError::InvalidPosition
        ))
    );
}
#[test]
fn graphical_cut_order_and_anchor_identity_f32() {
    cut_identity::<f32>();
}
#[test]
fn graphical_cut_order_and_anchor_identity_f64() {
    cut_identity::<f64>();
}
