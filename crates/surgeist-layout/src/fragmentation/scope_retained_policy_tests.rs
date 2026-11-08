//! Actual child Deferred metadata remains distinct from parent legality.
use super::*;

struct DeferredChildren {
    avoid: bool,
}
impl<S: LayoutScalar> FragmentationProvider<S> for DeferredChildren {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(frame(index, 100.0)))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let start = query.position.occurrence;
        let opportunity = |at| FragmentationOpportunityOf {
            cut: FragmentationCutOf::Source(position(at)),
            block_offset: S::ZERO,
            kind: FragmentationOpportunityKindOf::Sibling,
            before: Vec::new(),
            after: Vec::new(),
            ancestors: if self.avoid {
                vec![FragmentationBreakInside::AvoidPage]
            } else {
                Vec::new()
            },
        };
        let mut opportunities = vec![opportunity(start)];
        if start == 0 {
            opportunities.push(opportunity(1));
        }
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(150.0),
            head: FragmentationMeasuredHeadOf {
                end: position(start + 1),
                block_size: if start == 0 { S::ZERO } else { number(150.0) },
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities,
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn deferred_children<S: LayoutScalar>(avoid: bool) -> Arc<CompletedExecution<Key, S>> {
    let input = Arc::new(FragmentationInputOf {
        contexts: contexts(),
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: 0,
            initial_block_offset: number(100.0),
            occurrences: (0..2)
                .map(|index| FragmentationOccurrence {
                    key: Key::Child(index),
                    source_extent: 1,
                    monolithic: true,
                })
                .collect(),
        }],
    });
    let mut execution = Execution::new(input).unwrap();
    let mut provider = DeferredChildren { avoid };
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    let completed = Arc::new(execution.finish());
    let transition = &completed.report.breaks[0];
    assert_eq!(transition.reason, FragmentationBreakReason::Deferred);
    assert_eq!(transition.stage, FragmentationStage::Strict);
    assert_eq!(transition.position, position(1));
    assert_eq!(completed.report.pieces[0].block_size, S::ZERO);
    assert_eq!(
        (
            completed.report.pieces[0].coverage[0].start,
            completed.report.pieces[0].coverage[0].end
        ),
        (0, 1)
    );
    completed
}
struct PrefixParent<S: LayoutScalar> {
    ready: bool,
    avoid: bool,
    query: Cell<Option<FragmentationQueryOf<S>>>,
}
impl<S: LayoutScalar> FragmentationProvider<S> for PrefixParent<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(frame(index, 100.0)))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        self.query.set(Some(query));
        if !self.ready {
            return Err("scope required");
        }
        let start = query.position.occurrence;
        let (body, head, end) = match start {
            0 => (110.0, 100.0, 1),
            1 => (160.0, 150.0, 3),
            3 => (10.0, 10.0, 4),
            _ => unreachable!(),
        };
        let opportunity = |at, offset| FragmentationOpportunityOf {
            cut: FragmentationCutOf::Source(position(at)),
            block_offset: number(offset),
            kind: FragmentationOpportunityKindOf::Sibling,
            before: Vec::new(),
            after: Vec::new(),
            ancestors: if self.avoid {
                vec![FragmentationBreakInside::AvoidPage]
            } else {
                Vec::new()
            },
        };
        let mut opportunities = vec![opportunity(start, 0.0)];
        if start == 0 {
            opportunities.push(opportunity(1, 100.0));
        }
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(body),
            head: FragmentationMeasuredHeadOf {
                end: position(end),
                block_size: number(head),
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities,
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn retained_deferred_policy<S: LayoutScalar>() {
    for (child_avoid, parent_avoid) in [(false, false), (true, false), (false, true), (true, true)]
    {
        let mut input = (*parent_input::<S>(0.0)).clone();
        input.flows[0].occurrences.insert(
            0,
            FragmentationOccurrence {
                key: Key::Prefix,
                source_extent: 1,
                monolithic: true,
            },
        );
        let mut execution = Execution::new(Arc::new(input)).unwrap();
        let mut provider = PrefixParent {
            ready: false,
            avoid: parent_avoid,
            query: Cell::new(None),
        };
        assert!(matches!(
            execution.poll(&mut provider, &mut |_| Ok(())),
            Err(FragmentationError::Provider("scope required"))
        ));
        let child = deferred_children::<S>(child_avoid);
        let ticket = execution
            .scope_ticket(provider.query.get().unwrap(), position(1), position(2))
            .unwrap();
        execution
            .register_scope(
                ticket,
                child.clone(),
                vec![ScopeProjectionOf {
                    child_break_index: 0,
                    endpoint: ScopeEndpointOf::Retain(FragmentationCutOf::Source(position(1))),
                    block_offset: number(100.0),

                    normal_end: number(100.0),
                }],
                Vec::new(),
            )
            .unwrap();
        provider.ready = true;
        let mut appearances = 0;
        execution
            .poll(&mut provider, &mut |event| {
                appearances = event.appearances.len();
                Ok(())
            })
            .unwrap();
        let transition = &execution.report.breaks[0];
        if child_avoid && !parent_avoid {
            // The actual boundary before Owner is legal at Strict. It wins
            // before relaxing the illegal retained inner boundary.
            assert_eq!(transition.reason, FragmentationBreakReason::Unforced);
            assert_eq!(transition.stage, FragmentationStage::Strict);
            assert_eq!(appearances, 0);
            assert!(!execution.scope_participated(position(1), &child));
        } else {
            assert_eq!(transition.reason, FragmentationBreakReason::Deferred);
            assert_eq!(
                transition.stage,
                if parent_avoid {
                    FragmentationStage::RelaxAvoidance
                } else {
                    FragmentationStage::Strict
                }
            );
            assert_eq!(appearances, 1);
            assert!(execution.scope_participated(position(1), &child));
            while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
        }
    }
}
#[test]
fn retained_deferred_original_and_parent_legality_f32() {
    retained_deferred_policy::<f32>();
}
#[test]
fn retained_deferred_original_and_parent_legality_f64() {
    retained_deferred_policy::<f64>();
}

#[path = "scope_retained_deferral_force_tests.rs"]
mod deferral_force_tests;
