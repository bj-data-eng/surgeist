//! A before-owner force cannot consume an interior retained receipt.
use super::*;
struct RawSites<S: LayoutScalar> {
    ready: bool,
    query: Cell<Option<FragmentationQueryOf<S>>>,
}
impl<S: LayoutScalar> FragmentationProvider<S> for RawSites<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(frame(index, 50.0)))
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
        // The speculative W0 is not adopted before its raw Page control.
        // Re-preparing on frame1 gives the actual force-filled W50, followed
        // by terminal W0 and tail10 on frame2.
        let principal = if start == 0 && query.fragmentainer_index == 1 {
            50.0
        } else {
            0.0
        };
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(principal + 10.0),
            head: FragmentationMeasuredHeadOf {
                end: position(start + 1),
                block_size: number(if start == 2 { 10.0 } else { principal }),
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities: vec![FragmentationOpportunityOf {
                cut: FragmentationCutOf::Source(position(start)),
                block_offset: S::ZERO,
                kind: FragmentationOpportunityKindOf::Sibling,
                before: if start == 0 {
                    vec![FragmentationBreakControl {
                        value: FragmentationBreak::Page,
                        role: FragmentationRole::Normal,
                        source_order: 0,
                    }]
                } else {
                    Vec::new()
                },
                after: Vec::new(),
                ancestors: Vec::new(),
            }],
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn child_at<S: LayoutScalar>(start: usize, used: f64) -> Arc<CompletedExecution<Key, S>> {
    let input = Arc::new(FragmentationInputOf {
        contexts: contexts(),
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: start,
            initial_block_offset: number(used),
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
    while !execution
        .poll(&mut ZeroChildren { count: 2 }, &mut |_| Ok(()))
        .unwrap()
    {}
    Arc::new(execution.finish())
}
fn retained_force_sites<S: LayoutScalar>() {
    let mut execution = Execution::new(parent_input::<S>(50.0)).unwrap();
    let mut provider = RawSites {
        ready: false,
        query: Cell::new(None),
    };
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let old = child_at::<S>(0, 50.0);
    let ticket = execution
        .scope_ticket(provider.query.get().unwrap(), position(0), position(1))
        .unwrap();
    execution
        .register_scope(ticket, old.clone(), vec![retained(0)], Vec::new())
        .unwrap();
    provider.ready = true;
    execution
        .poll(&mut provider, &mut |event| {
            assert_eq!((event.from_frame, event.to_frame), (0, 1));
            assert!(event.pieces.is_empty());
            assert!(
                event.appearances.is_empty(),
                "raw before-owner force must not adopt the distinct interior A0 receipt"
            );
            Ok(())
        })
        .unwrap();
    assert!(!execution.scope_participated(position(0), &old));
    provider.ready = false;
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let fresh = child_at::<S>(1, 0.0);
    let ticket = execution
        .scope_ticket(provider.query.get().unwrap(), position(0), position(1))
        .unwrap();
    execution
        .register_scope(
            ticket,
            fresh.clone(),
            vec![ScopeProjectionOf {
                child_break_index: 0,
                endpoint: ScopeEndpointOf::Advance(FragmentationCutOf::Source(position(1))),
                block_offset: number(50.0),

                normal_end: number(50.0),
            }],
            Vec::new(),
        )
        .unwrap();
    provider.ready = true;
    execution
        .poll(&mut provider, &mut |event| {
            assert_eq!((event.from_frame, event.to_frame), (1, 2));
            assert_eq!(event.pieces[0].block_size, number(50.0));
            Ok(())
        })
        .unwrap();
    assert!(execution.scope_participated(position(0), &fresh));
    assert!(!execution.scope_participated(position(0), &old));
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    let report = execution.finish().into_report();
    assert_eq!(report.breaks.len(), 2);
    for transition in &report.breaks {
        assert_eq!(
            transition.reason,
            FragmentationBreakReason::Forced {
                required: FragmentationBreakKinds::PAGE,
                side: None
            }
        );
    }
    assert_eq!(report.pieces.last().unwrap().fragmentainer, 2);
    assert_eq!(report.pieces.last().unwrap().block_offset, S::ZERO);
}
#[test]
fn retained_force_before_owner_is_distinct_f32() {
    retained_force_sites::<f32>();
}
#[test]
fn retained_force_before_owner_is_distinct_f64() {
    retained_force_sites::<f64>();
}
