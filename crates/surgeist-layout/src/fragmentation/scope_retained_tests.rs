//! Functional tests for private retained source participation and rollback.
use super::scheduler::ScopeRegistrationError;
use super::*;

#[path = "scope_retained_policy_tests.rs"]
mod policy_tests;

#[path = "scope_retained_force_sites_tests.rs"]
mod force_sites_tests;

struct ZeroChildren {
    count: usize,
}
impl<S: LayoutScalar> FragmentationProvider<S> for ZeroChildren {
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
        let start = query.position.occurrence;
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: S::ZERO,
            head: FragmentationMeasuredHeadOf {
                end: position(start + 1),
                block_size: S::ZERO,
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities: (start + 1..self.count)
                .map(|index| FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Source(position(index)),
                    block_offset: S::ZERO,
                    kind: FragmentationOpportunityKindOf::Sibling,
                    before: vec![FragmentationBreakControl {
                        value: FragmentationBreak::Page,
                        role: FragmentationRole::Normal,
                        source_order: index,
                    }],
                    after: Vec::new(),
                    ancestors: Vec::new(),
                })
                .collect(),
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn zero_children<S: LayoutScalar>(
    count: usize,
    actual_source: bool,
) -> Arc<CompletedExecution<Key, S>> {
    let input = Arc::new(FragmentationInputOf {
        contexts: contexts(),
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: 0,
            initial_block_offset: number(50.0),
            occurrences: (0..count)
                .map(|index| FragmentationOccurrence {
                    key: Key::Child(index),
                    source_extent: usize::from(actual_source),
                    monolithic: actual_source,
                })
                .collect(),
        }],
    });
    let mut execution = Execution::new(input).unwrap();
    while !execution
        .poll(&mut ZeroChildren { count }, &mut |_| Ok(()))
        .unwrap()
    {}
    Arc::new(execution.finish())
}
struct ZeroParent<S: LayoutScalar> {
    ready: bool,
    force_before_tail: bool,
    query: Cell<Option<FragmentationQueryOf<S>>>,
}
impl<S: LayoutScalar> FragmentationProvider<S> for ZeroParent<S> {
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
        let mut opportunities = vec![FragmentationOpportunityOf {
            cut: FragmentationCutOf::Source(position(start)),
            block_offset: S::ZERO,
            kind: FragmentationOpportunityKindOf::Sibling,
            before: Vec::new(),
            after: Vec::new(),
            ancestors: Vec::new(),
        }];
        if self.force_before_tail && start < 2 {
            opportunities.push(FragmentationOpportunityOf {
                cut: FragmentationCutOf::Source(position(2)),
                block_offset: S::ZERO,
                kind: FragmentationOpportunityKindOf::Sibling,
                before: vec![FragmentationBreakControl {
                    value: FragmentationBreak::Page,
                    role: FragmentationRole::Normal,
                    source_order: 2,
                }],
                after: Vec::new(),
                ancestors: Vec::new(),
            });
        }
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(10.0),
            head: FragmentationMeasuredHeadOf {
                end: position(start + 1),
                block_size: if start == 2 { number(10.0) } else { S::ZERO },
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities,
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn pending_ticket<S: LayoutScalar>(
    execution: &mut Execution<Key, S>,
    provider: &mut ZeroParent<S>,
) -> ScopeTicket<S> {
    assert!(matches!(
        execution.poll(provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    execution
        .scope_ticket(provider.query.get().unwrap(), position(0), position(1))
        .unwrap()
}
fn retained<S: LayoutScalar>(index: usize) -> ScopeProjectionOf<S> {
    ScopeProjectionOf {
        child_break_index: index,
        endpoint: ScopeEndpointOf::Retain(FragmentationCutOf::Source(position(0))),
        block_offset: number(50.0),

        normal_end: number(50.0),
    }
}
fn retained_receipts<S: LayoutScalar>() {
    let mut execution = Execution::new(parent_input::<S>(50.0)).unwrap();
    let mut provider = ZeroParent {
        ready: false,
        force_before_tail: false,
        query: Cell::new(None),
    };
    let child = zero_children::<S>(3, true);
    for projections in [
        vec![retained(0), retained(0)],
        vec![retained(1), retained(0)],
        vec![ScopeProjectionOf {
            normal_end: number(f64::INFINITY),
            ..retained(0)
        }],
        vec![ScopeProjectionOf {
            normal_end: number(51.0),
            ..retained(0)
        }],
    ] {
        let ticket = pending_ticket(&mut execution, &mut provider);
        assert_eq!(
            execution.register_scope(ticket, child.clone(), projections, Vec::new()),
            Err(ScopeRegistrationError::InvalidProjection)
        );
    }
    // Zero-source control tokens alone cannot authenticate retained content.
    let ticket = pending_ticket(&mut execution, &mut provider);
    assert_eq!(
        execution.register_scope(
            ticket,
            zero_children(3, false),
            vec![retained(0)],
            Vec::new()
        ),
        Err(ScopeRegistrationError::InvalidProjection)
    );
    let ticket = pending_ticket(&mut execution, &mut provider);
    let mut second = retained(1);
    second.block_offset = S::ZERO;
    second.normal_end = S::ZERO;
    execution
        .register_scope(ticket, child.clone(), vec![retained(0), second], Vec::new())
        .unwrap();
    provider.ready = true;
    assert!(matches!(
        execution.poll(&mut provider, &mut |event| {
            assert!(event.pieces.is_empty());
            assert_eq!(event.appearances.len(), 1);
            assert_eq!(event.position, position(0));
            assert_eq!((event.from_frame, event.to_frame), (0, 1));
            Err("feedback rollback")
        }),
        Err(FragmentationError::Provider("feedback rollback"))
    ));
    assert!(execution.report.pieces.is_empty());
    assert!(execution.report.breaks.is_empty());
    assert!(!execution.scope_participated(position(0), &child));
    assert!(execution.scopes[0].applied.iter().all(|applied| !*applied));
    assert!(
        execution.scopes[0]
            .source_windows
            .iter()
            .all(|applied| !*applied)
    );
    for from in 0..2 {
        assert!(
            !execution
                .poll(&mut provider, &mut |event| {
                    assert!(event.pieces.is_empty());
                    assert_eq!(event.appearances.len(), 1);
                    assert_eq!((event.from_frame, event.to_frame), (from, from + 1));
                    Ok(())
                })
                .unwrap()
        );
        assert!(execution.scope_participated(position(0), &child));
        assert!(!execution.scope_participated(position(0), &zero_children::<S>(3, true)));
        assert_eq!(execution.report.breaks.len(), from + 1);
    }
    // A terminal actual source window still participates while PrincipalEnd
    // is consumed. Failed feedback must not consume its witness.
    assert!(matches!(
        execution.poll(&mut provider, &mut |event| {
            assert!(event.flow_complete);
            assert_eq!(event.from_frame, 2);
            Err("terminal rollback")
        }),
        Err(FragmentationError::Provider("terminal rollback"))
    ));
    assert!(!execution.scopes[0].source_windows[2]);
    assert!(execution.scope_participated(position(0), &child));
    assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
    assert!(execution.scope_participated(position(0), &child));
    assert!(execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
    let report = execution.finish().into_report();
    assert_eq!(report.breaks.len(), 2);
    assert_eq!(report.pieces.len(), 1);
    assert_eq!(report.pieces[0].fragmentainer, 2);
    assert_eq!(report.pieces[0].block_size, number(10.0));
}
fn nested_whole_fit<S: LayoutScalar>() {
    let mut outer = Execution::new(parent_input::<S>(50.0)).unwrap();
    let mut provider = ZeroParent {
        ready: false,
        force_before_tail: true,
        query: Cell::new(None),
    };
    let ticket = pending_ticket(&mut outer, &mut provider);
    outer
        .register_scope(ticket, zero_children(1, true), Vec::new(), Vec::new())
        .unwrap();
    provider.ready = true;
    while !outer.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    let completed = Arc::new(outer.finish());
    // The first accepted piece contains only zero-source Owner/End tokens;
    // its source witness must therefore come from the completed nested child.
    assert!(
        completed.report.pieces[0]
            .coverage
            .iter()
            .all(|coverage| coverage.start == coverage.end)
    );
    assert!(completed.accepted_source[0] > 0);
    let mut parent = Execution::new(parent_input::<S>(50.0)).unwrap();
    let mut provider = ZeroParent {
        ready: false,
        force_before_tail: false,
        query: Cell::new(None),
    };
    let ticket = pending_ticket(&mut parent, &mut provider);
    parent
        .register_scope(ticket, completed.clone(), vec![retained(0)], Vec::new())
        .unwrap();
    provider.ready = true;
    parent
        .poll(&mut provider, &mut |event| {
            assert!(event.pieces.is_empty());
            assert_eq!(event.appearances.len(), 1);
            Ok(())
        })
        .unwrap();
    assert!(parent.scope_participated(position(0), &completed));
}
#[test]
fn retained_source_receipts_and_rollback_f32() {
    retained_receipts::<f32>();
}
#[test]
fn retained_source_receipts_and_rollback_f64() {
    retained_receipts::<f64>();
}
#[test]
fn retained_source_inherits_nested_whole_fit_f32() {
    nested_whole_fit::<f32>();
}
#[test]
fn retained_source_inherits_nested_whole_fit_f64() {
    nested_whole_fit::<f64>();
}

#[path = "scope_retained_graphical_tests.rs"]
mod graphical_tests;
