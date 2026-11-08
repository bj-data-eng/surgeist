use super::scheduler::ScopeRegistrationError;
use super::*;
use std::cell::Cell;

#[path = "scope_retained_tests.rs"]
mod retained_tests;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Key {
    Prefix,
    Owner,
    End,
    Sibling,
    Child(usize),
}

struct DeferralChild {
    late: bool,
    first_height: f64,
}
impl<S: LayoutScalar> FragmentationProvider<S> for DeferralChild {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(frame(
            index,
            if index == 0 { self.first_height } else { 80.0 },
        )))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let first = query.position.occurrence == 0;
        let head = if first { 150.0 } else { 30.0 };
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(head + if first && self.late { 30.0 } else { 0.0 }),
            head: FragmentationMeasuredHeadOf {
                end: position(query.position.occurrence + 1),
                block_size: number(head),
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities: vec![FragmentationOpportunityOf {
                cut: FragmentationCutOf::Source(query.position),
                block_offset: S::ZERO,
                kind: FragmentationOpportunityKindOf::Sibling,
                before: Vec::new(),
                after: Vec::new(),
                ancestors: Vec::new(),
            }],
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn deferred_child<S: LayoutScalar>(
    used: f64,
    start: usize,
    late: bool,
) -> Arc<CompletedExecution<Key, S>> {
    deferred_child_with_height(used, start, late, 50.0)
}
fn deferred_child_with_height<S: LayoutScalar>(
    used: f64,
    start: usize,
    late: bool,
    first_height: f64,
) -> Arc<CompletedExecution<Key, S>> {
    let input = Arc::new(FragmentationInputOf {
        contexts: contexts(),
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: start,
            initial_block_offset: number(used),
            occurrences: (0..if late { 2 } else { 1 })
                .map(|index| FragmentationOccurrence {
                    key: Key::Child(index),
                    source_extent: 1,
                    monolithic: true,
                })
                .collect(),
        }],
    });
    let mut execution = Execution::new(input).unwrap();
    let mut provider = DeferralChild { late, first_height };
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    Arc::new(execution.finish())
}

struct PlacementProvider<S: LayoutScalar> {
    ready: bool,
    prefix: bool,
    avoid: bool,
    control: Option<FragmentationBreak>,
    last: Cell<Option<FragmentationQueryOf<S>>>,
}
impl<S: LayoutScalar> FragmentationProvider<S> for PlacementProvider<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(frame(index, if index == 0 { 50.0 } else { 80.0 })))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        self.last.set(Some(query));
        if !self.ready {
            return Err("scope required");
        }
        if query.scan_after.is_none()
            && let Some(value) = self.control
        {
            return Ok(FragmentationReplyOf::ControlAtStart(
                FragmentationBoundaryControls {
                    kind: FragmentationControlKind::Sibling,
                    before: vec![FragmentationBreakControl {
                        value,
                        role: FragmentationRole::Normal,
                        source_order: 0,
                    }],
                    after: Vec::new(),
                },
            ));
        }
        if self.control.is_some()
            && query.fragmentainer_index == 0
            && query.position == position(usize::from(self.prefix))
            && query.used_block_offset > S::ZERO
        {
            return Err("current owner body is unreachable before placement rebind");
        }
        let prefix = self.prefix && query.position == position(0);
        let owner = usize::from(self.prefix);
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: if prefix {
                FragmentationWindowEnd::PotentialControl(position(owner))
            } else {
                FragmentationWindowEnd::FlowEnd
            },
            block_size: number(if prefix { 30.0 } else { 11.0 }),
            head: FragmentationMeasuredHeadOf {
                end: position(if prefix {
                    owner
                } else {
                    query.position.occurrence + 1
                }),
                block_size: number(if prefix { 30.0 } else { 1.0 }),
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities: if prefix {
                vec![FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Source(position(owner)),
                    block_offset: number(30.0),
                    kind: FragmentationOpportunityKindOf::Sibling,
                    before: Vec::new(),
                    after: Vec::new(),
                    ancestors: if self.avoid {
                        vec![FragmentationBreakInside::AvoidPage]
                    } else {
                        Vec::new()
                    },
                }]
            } else {
                Vec::new()
            },
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn placement_input<S: LayoutScalar>(
    prefix: bool,
    occupied: bool,
) -> Arc<FragmentationInputOf<Key, S>> {
    let mut input = (*parent_input::<S>(if occupied { 30.0 } else { 0.0 })).clone();
    if prefix {
        input.flows[0].occurrences.insert(
            0,
            FragmentationOccurrence {
                key: Key::Prefix,
                source_extent: 1,
                monolithic: true,
            },
        );
    }
    Arc::new(input)
}
fn initial_deferred_placement<S: LayoutScalar>(prefix: bool, avoid: bool) {
    let mut execution = Execution::new(placement_input::<S>(prefix, !prefix)).unwrap();
    let mut provider = PlacementProvider {
        ready: false,
        prefix,
        avoid,
        control: Some(FragmentationBreak::Auto),
        last: Cell::new(None),
    };
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let owner = position(usize::from(prefix));
    let close = position(owner.occurrence + 1);
    let ticket = execution
        .scope_ticket(provider.last.get().unwrap(), owner, close)
        .unwrap();
    execution
        .register_scope(
            ticket,
            deferred_child::<S>(30.0, 0, false),
            vec![ScopeProjectionOf {
                child_break_index: 0,
                endpoint: ScopeEndpointOf::Advance(FragmentationCutOf::Source(owner)),
                block_offset: number(30.0),

                normal_end: number(30.0),
            }],
            Vec::new(),
        )
        .unwrap();
    provider.ready = true;
    assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
    assert_eq!(execution.report.breaks.len(), 1);
    assert_eq!(execution.report.breaks[0].from_fragmentainer, 0);
    assert_eq!(execution.report.breaks[0].to_fragmentainer, 1);
    assert_eq!(execution.state.as_ref().unwrap().position, owner);
    assert!(execution.scopes.is_empty());
    if prefix {
        assert_eq!(execution.report.pieces.len(), 1);
        assert_eq!(execution.report.pieces[0].coverage[0].key, Key::Prefix);
        assert_eq!(execution.report.pieces[0].block_size, number(30.0));
        assert_eq!(
            execution.report.breaks[0].reason,
            FragmentationBreakReason::Unforced
        );
        assert_eq!(
            execution.report.breaks[0].stage,
            if avoid {
                FragmentationStage::RelaxAvoidance
            } else {
                FragmentationStage::Strict
            }
        );
    } else {
        assert!(execution.report.pieces.is_empty());
        assert_eq!(
            execution.report.breaks[0].reason,
            FragmentationBreakReason::Deferred
        );
    }
    provider.ready = false;
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let ticket = execution
        .scope_ticket(provider.last.get().unwrap(), owner, close)
        .unwrap();
    execution
        .register_scope(
            ticket,
            deferred_child::<S>(0.0, 1, false),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
    provider.ready = true;
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    let report = execution.finish().into_report();
    assert_eq!(
        report.breaks.len(),
        1,
        "rebound fresh owner cannot repeatedly defer"
    );
    assert_eq!(report.pieces.last().unwrap().fragmentainer, 1);
    assert_eq!(report.pieces.last().unwrap().block_size, number(11.0));
    assert!(
        report
            .pieces
            .last()
            .unwrap()
            .coverage
            .iter()
            .any(|coverage| coverage.key == Key::Owner)
    );
}
fn later_deferral_after_cap<S: LayoutScalar>() {
    let completed_child = deferred_child::<S>(0.0, 0, true);
    assert_eq!(completed_child.report().breaks[0].position, position(1));
    let mut execution = Execution::new(placement_input::<S>(false, false)).unwrap();
    let mut provider = PlacementProvider {
        ready: false,
        prefix: false,
        avoid: false,
        control: None,
        last: Cell::new(None),
    };
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let query = provider.last.get().unwrap();
    let invalid = execution
        .scope_ticket(query, position(0), position(1))
        .unwrap();
    assert_eq!(
        execution.register_scope(
            invalid,
            completed_child.clone(),
            vec![ScopeProjectionOf {
                child_break_index: 0,
                endpoint: ScopeEndpointOf::Advance(FragmentationCutOf::Source(position(0))),
                block_offset: number(30.0),

                normal_end: number(30.0),
            }],
            Vec::new()
        ),
        Err(ScopeRegistrationError::InvalidProjection)
    );
    let invalid_empty = execution
        .scope_ticket(query, position(0), position(1))
        .unwrap();
    assert_eq!(
        execution.register_scope(
            invalid_empty,
            deferred_child::<S>(30.0, 0, false),
            vec![ScopeProjectionOf {
                child_break_index: 0,
                endpoint: ScopeEndpointOf::Advance(FragmentationCutOf::Source(position(0))),
                block_offset: S::ZERO,

                normal_end: S::ZERO,
            }],
            Vec::new()
        ),
        Err(ScopeRegistrationError::InvalidProjection)
    );
    // This one-unit principal ends before the child's later transition; Block
    // supplies no projection for that after-cap overflow transition.
    let ticket = execution
        .scope_ticket(query, position(0), position(1))
        .unwrap();
    execution
        .register_scope(ticket, completed_child, Vec::new(), Vec::new())
        .unwrap();
    provider.ready = true;
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    let report = execution.finish().into_report();
    assert!(report.breaks.is_empty());
    assert_eq!(report.pieces[0].fragmentainer, 0);
    assert_eq!(report.pieces[0].block_size, number(11.0));
}

fn raw_force_precedes_initial_placement<S: LayoutScalar>() {
    let mut execution = Execution::new(placement_input::<S>(false, true)).unwrap();
    let mut provider = PlacementProvider {
        ready: false,
        prefix: false,
        avoid: false,
        control: Some(FragmentationBreak::Page),
        last: Cell::new(None),
    };
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let ticket = execution
        .scope_ticket(provider.last.get().unwrap(), position(0), position(1))
        .unwrap();
    execution
        .register_scope(
            ticket,
            deferred_child::<S>(30.0, 0, false),
            vec![ScopeProjectionOf {
                child_break_index: 0,
                endpoint: ScopeEndpointOf::Advance(FragmentationCutOf::Source(position(0))),
                block_offset: number(30.0),

                normal_end: number(30.0),
            }],
            Vec::new(),
        )
        .unwrap();
    provider.ready = true;
    execution.poll(&mut provider, &mut |_| Ok(())).unwrap();
    assert!(execution.report.pieces.is_empty());
    assert_eq!(
        execution.report.breaks[0].reason,
        FragmentationBreakReason::Forced {
            required: FragmentationBreakKinds::PAGE,
            side: None
        }
    );
    assert_eq!(execution.report.breaks[0].to_fragmentainer, 1);
    assert!(execution.scopes.is_empty());
}

#[test]
fn raw_start_force_keeps_priority_over_owner_rebind_f32() {
    raw_force_precedes_initial_placement::<f32>();
}
#[test]
fn raw_start_force_keeps_priority_over_owner_rebind_f64() {
    raw_force_precedes_initial_placement::<f64>();
}

struct HeadProvider<S: LayoutScalar> {
    ready: bool,
    extent: f64,
    last: Cell<Option<FragmentationQueryOf<S>>>,
}
impl<S: LayoutScalar> FragmentationProvider<S> for HeadProvider<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(frame(index, if index == 0 { 100.0 } else { 80.0 })))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        self.last.set(Some(query));
        if !self.ready {
            return Err("scope required");
        }
        let consumed = query
            .graphical_progress
            .map_or(0.0, |progress| progress.consumed.to_f64());
        let remaining = if query.position == position(0) {
            self.extent - consumed
        } else {
            0.0
        };
        let opportunities = if remaining > 150.0 {
            vec![FragmentationOpportunityOf {
                cut: FragmentationCutOf::Graphical {
                    anchor: position(0),
                    progress: FragmentationGraphicalProgressOf {
                        consumed: number(150.0),
                        extent: number(self.extent),
                    },
                },
                block_offset: number(150.0),
                kind: FragmentationOpportunityKindOf::ModeSpecific,
                before: Vec::new(),
                after: Vec::new(),
                ancestors: Vec::new(),
            }]
        } else {
            Vec::new()
        };
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(remaining + 10.0),
            head: FragmentationMeasuredHeadOf {
                end: position(query.position.occurrence + 1),
                block_size: number(remaining),
            },
            fallback: FragmentationFallback::Sliceable,
            opportunities,
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: Some(FragmentationEmergencySliceOf::Graphical {
                end: number(100.0),
                extent: number(self.extent),
                block_size: query.available_block_size,
            }),
        }))
    }
}
fn head_execution<S: LayoutScalar>(
    child: Arc<CompletedExecution<Key, S>>,
    extent: f64,
    reject_first: bool,
) -> CompletedExecution<Key, S> {
    let mut execution = Execution::new(parent_input(0.0)).unwrap();
    let mut provider = HeadProvider {
        ready: false,
        extent,
        last: Cell::new(None),
    };
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let ticket = execution
        .scope_ticket(provider.last.get().unwrap(), position(0), position(1))
        .unwrap();
    let cut = if extent == 1.0 {
        FragmentationCutOf::Source(position(1))
    } else {
        FragmentationCutOf::Graphical {
            anchor: position(0),
            progress: FragmentationGraphicalProgressOf {
                consumed: number(150.0),
                extent: number(extent),
            },
        }
    };
    let offset = number(if extent == 1.0 { 1.0 } else { 150.0 });
    execution
        .register_scope(
            ticket,
            child,
            if extent == 1.0 {
                Vec::new()
            } else {
                vec![ScopeProjectionOf {
                    child_break_index: 0,
                    endpoint: ScopeEndpointOf::Advance(cut),
                    block_offset: offset,

                    normal_end: offset,
                }]
            },
            vec![ScopeHeadProjectionOf {
                child_piece_index: 0,
                cut,
                block_offset: offset,
            }],
        )
        .unwrap();
    provider.ready = true;
    if reject_first {
        assert!(matches!(
            execution.poll(&mut provider, &mut |_| Err("reject head")),
            Err(FragmentationError::Provider("reject head"))
        ));
        assert!(execution.report.pieces.is_empty());
        assert!(execution.qualified_heads.is_empty());
        assert!(!execution.scopes[0].active);
    }
    assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
    if extent == 1.0 {
        assert_eq!(execution.report.pieces[0].block_size, number(11.0));
        assert!(execution.report.breaks.is_empty());
    } else {
        assert_eq!(execution.report.pieces[0].block_size, number(150.0));
        assert_eq!(
            execution.report.pieces[0].stage,
            FragmentationStage::Emergency
        );
        assert_eq!(execution.state.as_ref().unwrap().position, position(0));
        assert_eq!(
            execution
                .state
                .as_ref()
                .unwrap()
                .graphical
                .unwrap()
                .consumed,
            number(150.0)
        );
        assert!(execution.qualified_heads[0]);
        assert!(execution.report.breaks.is_empty());
        assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
        assert_eq!(
            execution.report.breaks[0].reason,
            FragmentationBreakReason::Deferred
        );
        assert_eq!(execution.report.breaks[0].from_fragmentainer, 0);
        assert_eq!(execution.report.breaks[0].to_fragmentainer, 1);
        assert!(
            execution.scopes[0].active,
            "later deferral cannot rebind accepted opening"
        );
    }
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    let completed = execution.finish();
    if extent != 1.0 {
        assert_eq!(
            completed.report().pieces[1].block_size,
            number(extent - 150.0)
        );
        assert_eq!(completed.report().pieces[1].fragmentainer, 1);
        assert_eq!(completed.report().breaks.len(), 1);
    }
    completed
}
fn certified_oversized_head<S: LayoutScalar>() {
    let child = deferred_child_with_height::<S>(0.0, 0, true, 100.0);
    assert_eq!(
        child.report().pieces[0].kind,
        FragmentationPieceKindOf::UnsliceableOverflow
    );
    assert!(child.head_qualified(0));
    assert!(!child.head_qualified(1));
    assert!(!child.head_qualified(child.report().pieces.len()));
    assert_eq!(
        child.report().breaks[0].reason,
        FragmentationBreakReason::Deferred
    );
    let parent = head_execution(child, 180.0, true);
    assert!(parent.head_qualified(0));
    assert!(!parent.head_qualified(1));
    assert!(matches!(
        parent.report().pieces[0].kind,
        FragmentationPieceKindOf::GraphicalSlice { .. }
    ));
    let nested = head_execution(Arc::new(parent), 190.0, false);
    assert!(nested.head_qualified(0));
    assert!(!nested.head_qualified(1));
    assert!(!nested.head_qualified(usize::MAX));
}
fn certified_head_does_not_inflate_cap<S: LayoutScalar>() {
    head_execution(
        deferred_child_with_height::<S>(0.0, 0, true, 100.0),
        1.0,
        false,
    );
}
fn ordinary_child_piece_cannot_authorize_oversized_head<S: LayoutScalar>() {
    let child = child::<S>(false, 100.0, 0.0, 0);
    assert!(!child.head_qualified(0));
    let mut execution = Execution::new(parent_input(0.0)).unwrap();
    let mut provider = HeadProvider {
        ready: false,
        extent: 180.0,
        last: Cell::new(None),
    };
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let ticket = execution
        .scope_ticket(provider.last.get().unwrap(), position(0), position(1))
        .unwrap();
    execution
        .register_scope(
            ticket,
            child,
            Vec::new(),
            vec![ScopeHeadProjectionOf {
                child_piece_index: 0,
                cut: FragmentationCutOf::Graphical {
                    anchor: position(0),
                    progress: FragmentationGraphicalProgressOf {
                        consumed: number(150.0),
                        extent: number(180.0),
                    },
                },
                block_offset: number(150.0),
            }],
        )
        .unwrap();
    provider.ready = true;
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::InvalidInput(
            FragmentationInputError::InvalidSlice
        ))
    ));
    assert!(execution.report.pieces.is_empty());
    assert!(execution.qualified_heads.is_empty());
}

struct LaterCutProvider<S: LayoutScalar>(ParentProvider<S>, bool);
impl<S: LayoutScalar> FragmentationProvider<S> for LaterCutProvider<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        self.0.fragmentainer(context, index)
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        if self.1 && query.scan_after == Some(position(2)) {
            return Err("future sibling body was measured before selected child transition");
        }
        let mut reply = self.0.layout(query)?;
        if query.position == position(0)
            && query.graphical_progress.is_none()
            && let FragmentationReplyOf::Measured(window) = &mut reply
        {
            window.opportunities.push(FragmentationOpportunityOf {
                cut: FragmentationCutOf::Graphical {
                    anchor: position(0),
                    progress: FragmentationGraphicalProgressOf {
                        consumed: number(45.0),
                        extent: number(80.0),
                    },
                },
                block_offset: number(45.0),
                kind: FragmentationOpportunityKindOf::ModeSpecific,
                before: Vec::new(),
                after: Vec::new(),
                ancestors: Vec::new(),
            });
            if self.1 {
                window.end = FragmentationWindowEnd::PotentialControl(position(2));
                window.block_size = number(80.0);
                window.opportunities.push(FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Source(position(2)),
                    block_offset: number(80.0),
                    kind: FragmentationOpportunityKindOf::Sibling,
                    before: Vec::new(),
                    after: Vec::new(),
                    ancestors: Vec::new(),
                });
            }
        }
        Ok(reply)
    }
}
fn pending_unforced_cut_bounds_selection<S: LayoutScalar>() {
    for partial in [false, true] {
        let mut execution = Execution::new(parent_input(0.0)).unwrap();
        let mut provider = LaterCutProvider(
            ParentProvider {
                height: 50.0,
                first_extent: 30.0,
                extent: 80.0,
                closing: 0.0,
                gap: 20.0,
                ..ParentProvider::forced(0.0, false)
            },
            partial,
        );
        assert!(matches!(
            execution.poll(&mut provider, &mut |_| Ok(())),
            Err(FragmentationError::Provider("scope required"))
        ));
        let ticket = execution
            .scope_ticket(
                provider.0.last_query.get().unwrap(),
                position(0),
                position(1),
            )
            .unwrap();
        execution
            .register_scope(
                ticket,
                child::<S>(false, 50.0, 0.0, 0),
                vec![provider.0.projection(0)],
                Vec::new(),
            )
            .unwrap();
        provider.0.ready = true;
        execution
            .poll(&mut provider, &mut |_| Ok(()))
            .expect("the common selector must preserve the pending child boundary");
        assert_eq!(execution.report.pieces[0].block_size, number(30.0));
        assert_eq!(
            execution.report.pieces[0].stage,
            FragmentationStage::RelaxAvoidance
        );
        assert_eq!(
            execution
                .state
                .as_ref()
                .unwrap()
                .graphical
                .unwrap()
                .consumed,
            number(30.0)
        );
    }
}

struct FixedBoundaryProvider<S: LayoutScalar> {
    base: ParentProvider<S>,
    later_force: bool,
}
impl<S: LayoutScalar> FragmentationProvider<S> for FixedBoundaryProvider<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        self.base.fragmentainer(context, index)
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        if query.scan_after.is_some() {
            return Err("known child terminal cannot authorize later discovery");
        }
        let mut reply = self.base.layout(query)?;
        if let FragmentationReplyOf::Measured(window) = &mut reply {
            if self.later_force {
                window.opportunities.push(FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Source(position(1)),
                    block_offset: number(45.0),
                    kind: FragmentationOpportunityKindOf::Sibling,
                    before: vec![FragmentationBreakControl {
                        value: FragmentationBreak::Page,
                        role: FragmentationRole::Normal,
                        source_order: 3,
                    }],
                    after: Vec::new(),
                    ancestors: Vec::new(),
                });
            } else {
                window.end = FragmentationWindowEnd::PotentialControl(position(1));
                window.block_size = number(30.0);
                window.opportunities[0].kind = FragmentationOpportunityKindOf::Sibling;
            }
        }
        Ok(reply)
    }
}
fn fixed_child_boundary_limits_discovery_and_force<S: LayoutScalar>() {
    for later_force in [false, true] {
        let mut execution = Execution::new(parent_input(0.0)).unwrap();
        let mut provider = FixedBoundaryProvider {
            base: ParentProvider {
                height: 50.0,
                first_extent: 30.0,
                extent: if later_force { 80.0 } else { 30.0 },
                closing: 0.0,
                gap: 20.0,
                ..ParentProvider::forced(0.0, !later_force)
            },
            later_force,
        };
        assert!(matches!(
            execution.poll(&mut provider, &mut |_| Ok(())),
            Err(FragmentationError::Provider("scope required"))
        ));
        let ticket = execution
            .scope_ticket(
                provider.base.last_query.get().unwrap(),
                position(0),
                position(1),
            )
            .unwrap();
        execution
            .register_scope(
                ticket,
                child::<S>(false, 50.0, 0.0, 0),
                vec![provider.base.projection(0)],
                Vec::new(),
            )
            .unwrap();
        provider.base.ready = true;
        execution.poll(&mut provider, &mut |_| Ok(())).unwrap();
        assert_eq!(execution.report.pieces[0].block_size, number(30.0));
        assert_eq!(
            execution.report.breaks[0].reason,
            FragmentationBreakReason::Unforced
        );
        assert_eq!(
            execution.report.breaks[0].stage,
            FragmentationStage::RelaxAvoidance
        );
    }
}
#[test]
fn fixed_child_boundary_limits_discovery_and_force_f32() {
    fixed_child_boundary_limits_discovery_and_force::<f32>();
}
#[test]
fn fixed_child_boundary_limits_discovery_and_force_f64() {
    fixed_child_boundary_limits_discovery_and_force::<f64>();
}

struct EmergencyReceiptProvider<S: LayoutScalar> {
    base: ParentProvider<S>,
    initial: bool,
}
impl<S: LayoutScalar> FragmentationProvider<S> for EmergencyReceiptProvider<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        self.base.fragmentainer(context, index)
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let mut reply = self.base.layout(query)?;
        if let FragmentationReplyOf::Measured(window) = &mut reply {
            if self.initial && query.position == position(0) && query.graphical_progress.is_none() {
                window.opportunities.clear();
                window.emergency_slice = Some(FragmentationEmergencySliceOf::Graphical {
                    end: number(30.0),
                    extent: number(80.0),
                    block_size: number(30.0),
                });
            } else {
                window.emergency_slice = None;
            }
        }
        Ok(reply)
    }
}
fn emergency_receipt_preserves_policy_for_embedding<S: LayoutScalar>() {
    let original = child::<S>(false, 50.0, 0.0, 0);
    let mut completed = original.clone();
    for initial in [true, false] {
        let mut execution = Execution::new(parent_input(0.0)).unwrap();
        let mut provider = EmergencyReceiptProvider {
            base: ParentProvider {
                height: 50.0,
                first_extent: 30.0,
                extent: 80.0,
                closing: 0.0,
                gap: 0.0,
                ..ParentProvider::forced(0.0, false)
            },
            initial,
        };
        assert!(matches!(
            execution.poll(&mut provider, &mut |_| Ok(())),
            Err(FragmentationError::Provider("scope required"))
        ));
        let ticket = execution
            .scope_ticket(
                provider.base.last_query.get().unwrap(),
                position(0),
                position(1),
            )
            .unwrap();
        execution
            .register_scope(
                ticket,
                completed,
                vec![provider.base.projection(0)],
                Vec::new(),
            )
            .unwrap();
        provider.base.ready = true;
        while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
        let result = Arc::new(execution.finish());
        assert_eq!(result.report.pieces[0].block_size, number(30.0));
        assert!(matches!(
            result.report.pieces[0].kind,
            FragmentationPieceKindOf::GraphicalSlice { .. }
        ));
        assert!(
            !result.head_qualified(0),
            "ordinary graphical acceptance does not issue an indivisible head"
        );
        assert_eq!(
            result.report.breaks[0].reason,
            FragmentationBreakReason::Unforced
        );
        assert_eq!(result.report.breaks[0].stage, FragmentationStage::Emergency);
        assert!(Arc::ptr_eq(
            &original.selected_origins[0].as_ref().unwrap().origin,
            &result.selected_origins[0].as_ref().unwrap().origin,
        ));
        completed = result;
    }
}
#[test]
fn emergency_receipt_preserves_policy_for_embedding_f32() {
    emergency_receipt_preserves_policy_for_embedding::<f32>();
}
#[test]
fn emergency_receipt_preserves_policy_for_embedding_f64() {
    emergency_receipt_preserves_policy_for_embedding::<f64>();
}

fn sided_frame<S: LayoutScalar>(index: usize) -> FragmentainerOf<S> {
    FragmentainerOf {
        page_side: Some(if index.is_multiple_of(2) {
            FragmentationPhysicalPageSide::Left
        } else {
            FragmentationPhysicalPageSide::Right
        }),
        ..frame(index, 100.0)
    }
}
struct SidedChild {
    value: FragmentationBreak,
    order: usize,
}
impl<S: LayoutScalar> FragmentationProvider<S> for SidedChild {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(sided_frame(index)))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let mut reply = ChildProvider {
            forced: true,
            height: 100.0,
            count: 2,
        }
        .layout(query)?;
        if let FragmentationReplyOf::Measured(window) = &mut reply {
            for opportunity in &mut window.opportunities {
                opportunity.before = vec![FragmentationBreakControl {
                    value: self.value,
                    role: FragmentationRole::Normal,
                    source_order: self.order,
                }];
            }
        }
        Ok(reply)
    }
}
fn sided_child<S: LayoutScalar>(
    value: FragmentationBreak,
    order: usize,
) -> Arc<CompletedExecution<Key, S>> {
    let input = Arc::new(FragmentationInputOf {
        contexts: contexts(),
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: 0,
            initial_block_offset: S::ZERO,
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
    let mut provider = SidedChild { value, order };
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    Arc::new(execution.finish())
}
struct ForceCollision<S: LayoutScalar> {
    inner: ParentProvider<S>,
    value: FragmentationBreak,
    order: usize,
}
impl<S: LayoutScalar> FragmentationProvider<S> for ForceCollision<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(sided_frame(index)))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let mut reply = self.inner.layout(query)?;
        if let FragmentationReplyOf::Measured(window) = &mut reply {
            for opportunity in &mut window.opportunities {
                opportunity.kind = FragmentationOpportunityKindOf::Sibling;
                opportunity.before = vec![FragmentationBreakControl {
                    value: self.value,
                    role: FragmentationRole::Normal,
                    source_order: self.order,
                }];
            }
        }
        Ok(reply)
    }
}
fn equal_cut_forces_keep_policy_precedence<S: LayoutScalar>() {
    for (child_value, child_order, parent_value, parent_order, side, incompatible) in [
        (
            FragmentationBreak::Page,
            2,
            FragmentationBreak::Right,
            3,
            FragmentationForcedSide::Right,
            false,
        ),
        (
            FragmentationBreak::Right,
            2,
            FragmentationBreak::Page,
            3,
            FragmentationForcedSide::Right,
            false,
        ),
        (
            FragmentationBreak::Left,
            2,
            FragmentationBreak::Right,
            3,
            FragmentationForcedSide::Right,
            true,
        ),
        (
            FragmentationBreak::Right,
            2,
            FragmentationBreak::Left,
            1,
            FragmentationForcedSide::Right,
            false,
        ),
    ] {
        let mut execution = Execution::new(parent_input(0.0)).unwrap();
        let mut provider = ForceCollision {
            inner: ParentProvider::forced(0.0, true),
            value: parent_value,
            order: parent_order,
        };
        assert!(matches!(
            execution.poll(&mut provider, &mut |_| Ok(())),
            Err(FragmentationError::Provider("scope required"))
        ));
        let ticket = execution
            .scope_ticket(
                provider.inner.last_query.get().unwrap(),
                position(0),
                position(1),
            )
            .unwrap();
        execution
            .register_scope(
                ticket,
                sided_child::<S>(child_value, child_order),
                vec![provider.inner.projection(0)],
                Vec::new(),
            )
            .unwrap();
        provider.inner.ready = true;
        let result = execution.poll(&mut provider, &mut |_| Ok(()));
        if incompatible {
            assert!(matches!(
                result,
                Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidBoundary
                ))
            ));
            assert!(execution.report.pieces.is_empty());
            assert!(execution.report.breaks.is_empty());
        } else {
            result.expect("compatible parent force must not be discarded");
            assert_eq!(
                execution.report.breaks[0].reason,
                FragmentationBreakReason::Forced {
                    required: FragmentationBreakKinds::PAGE,
                    side: Some(side),
                }
            );
            assert_eq!(execution.report.breaks[0].to_fragmentainer, 1);
        }
    }
}

#[test]
fn scheduler_review_pending_unforced_cut_keeps_common_selector_f32() {
    pending_unforced_cut_bounds_selection::<f32>();
}
#[test]
fn scheduler_review_pending_unforced_cut_keeps_common_selector_f64() {
    pending_unforced_cut_bounds_selection::<f64>();
}
#[test]
fn scheduler_review_equal_cut_force_preserves_existing_policy_f32() {
    equal_cut_forces_keep_policy_precedence::<f32>();
}
#[test]
fn scheduler_review_equal_cut_force_preserves_existing_policy_f64() {
    equal_cut_forces_keep_policy_precedence::<f64>();
}

#[test]
fn certified_oversized_head_keeps_active_deferred_timeline_and_nests_f32() {
    certified_oversized_head::<f32>();
}
#[test]
fn certified_oversized_head_keeps_active_deferred_timeline_and_nests_f64() {
    certified_oversized_head::<f64>();
}
#[test]
fn capped_head_uses_projected_one_not_child_one_fifty_f32() {
    certified_head_does_not_inflate_cap::<f32>();
}
#[test]
fn capped_head_uses_projected_one_not_child_one_fifty_f64() {
    certified_head_does_not_inflate_cap::<f64>();
}
#[test]
fn unqualified_child_piece_cannot_invent_an_oversized_head_f32() {
    ordinary_child_piece_cannot_authorize_oversized_head::<f32>();
}
#[test]
fn unqualified_child_piece_cannot_invent_an_oversized_head_f64() {
    ordinary_child_piece_cannot_authorize_oversized_head::<f64>();
}

#[test]
fn initial_deferral_keeps_unaccepted_prefix_in_its_frame_f32() {
    initial_deferred_placement::<f32>(true, false);
}
#[test]
fn initial_deferral_keeps_unaccepted_prefix_in_its_frame_f64() {
    initial_deferred_placement::<f64>(true, false);
}
#[test]
fn initial_deferral_uses_occupied_query_ordinary_placement_f32() {
    initial_deferred_placement::<f32>(false, false);
}
#[test]
fn initial_deferral_uses_occupied_query_ordinary_placement_f64() {
    initial_deferred_placement::<f64>(false, false);
}
#[test]
fn initial_deferral_keeps_parent_avoidance_stage_f32() {
    initial_deferred_placement::<f32>(true, true);
}
#[test]
fn initial_deferral_keeps_parent_avoidance_stage_f64() {
    initial_deferred_placement::<f64>(true, true);
}
#[test]
fn after_cap_deferral_is_not_whole_owner_rebinding_f32() {
    later_deferral_after_cap::<f32>();
}
#[test]
fn after_cap_deferral_is_not_whole_owner_rebinding_f64() {
    later_deferral_after_cap::<f64>();
}

fn number<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn position(occurrence: usize) -> FragmentationPosition {
    FragmentationPosition {
        occurrence,
        offset: 0,
    }
}
fn contexts() -> Vec<FragmentationContext> {
    vec![FragmentationContext {
        kind: FragmentationKind::Page {
            progression: crate::PageProgression::LeftToRight,
        },
        parent: None,
        flow_axes: crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
    }]
}
fn parent_input<S: LayoutScalar>(used: f64) -> Arc<FragmentationInputOf<Key, S>> {
    Arc::new(FragmentationInputOf {
        contexts: contexts(),
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: 0,
            initial_block_offset: number(used),
            occurrences: [Key::Owner, Key::End, Key::Sibling]
                .into_iter()
                .map(|key| {
                    let source_extent = usize::from(key == Key::Sibling);
                    FragmentationOccurrence {
                        key,
                        source_extent,
                        monolithic: source_extent != 0,
                    }
                })
                .collect(),
        }],
    })
}
fn frame<S: LayoutScalar>(index: usize, height: f64) -> FragmentainerOf<S> {
    FragmentainerOf {
        inline_size: number(if index == 0 { 100.0 } else { 60.0 }),
        block_size: number(height),
        breaks: if index == 0 {
            FragmentationBreakKinds::NONE
        } else {
            FragmentationBreakKinds::PAGE
        },
        parent_fragmentainer: None,
        page_side: None,
    }
}

struct ChildProvider {
    forced: bool,
    height: f64,
    count: usize,
}
impl<S: LayoutScalar> FragmentationProvider<S> for ChildProvider {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(frame(index, self.height)))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let start = query.position.occurrence;
        let opportunities = (start + 1..self.count)
            .map(|index| FragmentationOpportunityOf {
                cut: FragmentationCutOf::Source(position(index)),
                block_offset: number(30.0 * (index - start) as f64),
                kind: FragmentationOpportunityKindOf::Sibling,
                before: if self.forced {
                    vec![FragmentationBreakControl {
                        value: FragmentationBreak::Page,
                        role: FragmentationRole::Normal,
                        source_order: index,
                    }]
                } else {
                    Vec::new()
                },
                after: Vec::new(),
                ancestors: if self.forced {
                    Vec::new()
                } else {
                    vec![FragmentationBreakInside::AvoidPage]
                },
            })
            .collect();
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(30.0 * (self.count - start) as f64),
            head: FragmentationMeasuredHeadOf {
                end: position(start + 1),
                block_size: number(30.0),
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities,
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: None,
        }))
    }
}
fn child<S: LayoutScalar>(
    forced: bool,
    height: f64,
    used: f64,
    start: usize,
) -> Arc<CompletedExecution<Key, S>> {
    let count = if forced { 3 } else { 2 };
    let input = Arc::new(FragmentationInputOf {
        contexts: contexts(),
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: start,
            initial_block_offset: number(used),
            occurrences: (0..count)
                .map(|index| FragmentationOccurrence {
                    key: Key::Child(index),
                    source_extent: 1,
                    monolithic: true,
                })
                .collect(),
        }],
    });
    let mut execution = Execution::new(input).unwrap();
    let mut provider = ChildProvider {
        forced,
        height,
        count,
    };
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    Arc::new(execution.finish())
}

struct ParentProvider<S: LayoutScalar> {
    ready: bool,
    height: f64,
    initial_used: f64,
    first_extent: f64,
    extent: f64,
    closing: f64,
    source_cut: bool,
    gap: f64,
    last_query: Cell<Option<FragmentationQueryOf<S>>>,
}
impl<S: LayoutScalar> ParentProvider<S> {
    fn forced(used: f64, source_cut: bool) -> Self {
        Self {
            ready: false,
            height: 100.0,
            initial_used: used,
            first_extent: 100.0 - used,
            extent: if source_cut {
                100.0 - used
            } else {
                130.0 - used
            },
            closing: 10.0,
            source_cut,
            gap: 0.0,
            last_query: Cell::new(None),
        }
    }
    fn projection(&self, child_break_index: usize) -> ScopeProjectionOf<S> {
        ScopeProjectionOf {
            child_break_index,
            endpoint: ScopeEndpointOf::Advance(if self.source_cut {
                FragmentationCutOf::Source(position(1))
            } else {
                FragmentationCutOf::Graphical {
                    anchor: position(0),
                    progress: FragmentationGraphicalProgressOf {
                        consumed: number(self.first_extent),
                        extent: number(self.extent),
                    },
                }
            }),
            block_offset: number(self.initial_used + self.first_extent),

            normal_end: number(self.initial_used + self.first_extent),
        }
    }
}
impl<S: LayoutScalar> FragmentationProvider<S> for ParentProvider<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(Some(frame(index, self.height)))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        self.last_query.set(Some(query));
        if !self.ready {
            return Err("scope required");
        }
        let (body, head, head_end, opportunities) = match query.position.occurrence {
            0 => {
                let consumed = query
                    .graphical_progress
                    .map_or(0.0, |progress| progress.consumed.to_f64());
                let remaining = self.extent - consumed;
                let opportunities = if consumed == 0.0 {
                    vec![FragmentationOpportunityOf {
                        cut: self.projection(0).endpoint.checkpoint(),
                        block_offset: number(self.first_extent),
                        kind: FragmentationOpportunityKindOf::ContentEdge {
                            gap: number(self.gap),
                        },
                        before: Vec::new(),
                        after: Vec::new(),
                        ancestors: Vec::new(),
                    }]
                } else {
                    Vec::new()
                };
                (remaining + self.closing + 10.0, remaining, 1, opportunities)
            }
            1 => (self.closing + 10.0, self.closing, 2, Vec::new()),
            2 => (10.0, 10.0, 3, Vec::new()),
            _ => unreachable!("the engine never queries a finished flow"),
        };
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(body),
            head: FragmentationMeasuredHeadOf {
                end: position(head_end),
                block_size: number(head),
            },
            fallback: FragmentationFallback::Sliceable,
            opportunities,
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            emergency_slice: Some(FragmentationEmergencySliceOf::Graphical {
                end: number(self.first_extent),
                extent: number(self.extent),
                block_size: query.available_block_size,
            }),
        }))
    }
}

fn prepare<S: LayoutScalar>(
    execution: &mut Execution<Key, S>,
    provider: &mut ParentProvider<S>,
) -> ScopeTicket<S> {
    assert!(matches!(
        execution.poll(provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    execution
        .scope_ticket(provider.last_query.get().unwrap(), position(0), position(1))
        .unwrap()
}

fn forced_scope<S: LayoutScalar>(source_cut: bool, used: f64) {
    let child = child::<S>(true, 100.0, used, 0);
    assert_eq!(child.report().breaks.len(), 2);
    let mut execution = Execution::new(parent_input(used)).unwrap();
    let mut provider = ParentProvider::forced(used, source_cut);
    let ticket = prepare(&mut execution, &mut provider);
    execution
        .register_scope(
            ticket,
            child.clone(),
            vec![provider.projection(0)],
            Vec::new(),
        )
        .unwrap();
    provider.ready = true;
    assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
    assert!(execution.scopes[0].active);
    assert!(!execution.scopes[0].retired);
    let first = &execution.report.pieces[0];
    assert_eq!(first.block_offset, number(used));
    assert_eq!(first.block_size, number(100.0 - used));
    assert_eq!(first.stage, FragmentationStage::Strict);
    assert_eq!(
        execution.report.breaks[0].reason,
        child.report().breaks[0].reason
    );
    assert_eq!(execution.report.breaks[0].to_fragmentainer, 1);
    if !source_cut {
        assert_eq!(execution.state.as_ref().unwrap().position, position(0));
        assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
        assert_eq!(execution.state.as_ref().unwrap().position, position(1));
        assert!(execution.scopes[0].active);
        assert!(!execution.scopes[0].retired);
    }
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    assert!(execution.scopes.is_empty());
    assert_eq!(
        Arc::strong_count(&child),
        1,
        "closed owner releases the scheduler's child receipt"
    );
    let completed = execution.finish();
    let report = completed.report();
    assert_eq!(
        report.breaks.len(),
        1,
        "the later overflow force stays in the child"
    );
    assert_eq!(report.pieces.last().unwrap().fragmentainer, 1);
    assert!(
        report
            .pieces
            .iter()
            .flat_map(|piece| &piece.coverage)
            .all(|coverage| !matches!(coverage.key, Key::Child(_)))
    );
    for key in [Key::Child(0), Key::Child(1), Key::Child(2)] {
        assert_eq!(
            child
                .report()
                .pieces
                .iter()
                .flat_map(|piece| &piece.coverage)
                .filter(|coverage| coverage.key == key)
                .count(),
            1
        );
    }
}
fn projected_stage_floor<S: LayoutScalar>() {
    let child = child::<S>(false, 50.0, 0.0, 0);
    assert_eq!(
        child.report().breaks[0].stage,
        FragmentationStage::RelaxAvoidance
    );
    let mut execution = Execution::new(parent_input(0.0)).unwrap();
    let mut provider = ParentProvider {
        height: 50.0,
        first_extent: 50.0,
        extent: 80.0,
        closing: 0.0,
        gap: 20.0,
        ..ParentProvider::forced(0.0, false)
    };
    let ticket = prepare(&mut execution, &mut provider);
    execution
        .register_scope(ticket, child, vec![provider.projection(0)], Vec::new())
        .unwrap();
    provider.ready = true;
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    let report = execution.finish().into_report();
    assert_eq!(report.pieces[0].stage, FragmentationStage::RelaxAvoidance);
    assert_eq!(report.breaks[0].stage, FragmentationStage::RelaxAvoidance);
    assert_eq!(report.breaks[0].reason, FragmentationBreakReason::Unforced);
}

#[derive(Clone, Copy)]
enum OriginCase {
    Sibling,
    ParentAvoid,
    InapplicableControl,
    ConditionalSibling,
    ConditionalLines,
    IntermediateWaiver,
    Lines,
}
impl OriginCase {
    fn nested(self) -> bool {
        matches!(
            self,
            Self::ConditionalSibling | Self::ConditionalLines | Self::IntermediateWaiver
        )
    }
    fn expected(self) -> FragmentationStage {
        match self {
            Self::Sibling | Self::ConditionalSibling | Self::IntermediateWaiver => {
                FragmentationStage::Strict
            }
            Self::Lines => FragmentationStage::RelaxLineCounts,
            _ => FragmentationStage::RelaxAvoidance,
        }
    }
}

struct OriginProvider<S: LayoutScalar> {
    case: OriginCase,
    parent: Option<ParentProvider<S>>,
    embedding: usize,
}
impl<S: LayoutScalar> FragmentationProvider<S> for OriginProvider<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        if self.case.nested() && context == 0 {
            return Ok(Some(frame(index, 1000.0)));
        }
        let mut result = frame(index, 30.0);
        if self.case.nested() {
            result.breaks = if index == 0 {
                FragmentationBreakKinds::NONE
            } else {
                FragmentationBreakKinds::COLUMN
            };
            result.parent_fragmentainer = Some(0);
        }
        Ok(Some(result))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let parent = self.parent.is_some();
        let mut reply = if let Some(provider) = &mut self.parent {
            provider.layout(query)?
        } else {
            ChildProvider {
                forced: false,
                height: 30.0,
                count: 2,
            }
            .layout(query)?
        };
        if let FragmentationReplyOf::Measured(window) = &mut reply {
            for opportunity in &mut window.opportunities {
                opportunity.ancestors.clear();
                if parent {
                    if matches!(self.case, OriginCase::IntermediateWaiver) {
                        opportunity
                            .ancestors
                            .push(FragmentationBreakInside::AvoidColumn);
                        if self.embedding == 0 {
                            opportunity.before.push(FragmentationBreakControl {
                                value: FragmentationBreak::AvoidPage,
                                role: FragmentationRole::Normal,
                                source_order: 2,
                            });
                        }
                        continue;
                    }
                    if matches!(self.case, OriginCase::ParentAvoid) || self.case.nested() {
                        opportunity.ancestors.push(FragmentationBreakInside::Avoid);
                    }
                    continue;
                }
                opportunity.kind =
                    if matches!(self.case, OriginCase::Lines | OriginCase::ConditionalLines) {
                        FragmentationOpportunityKindOf::Lines {
                            before: 1,
                            after: 3,
                            orphans: 2,
                            widows: 2,
                        }
                    } else {
                        FragmentationOpportunityKindOf::Sibling
                    };
                if (self.case.nested() && !matches!(self.case, OriginCase::IntermediateWaiver))
                    || matches!(self.case, OriginCase::InapplicableControl)
                {
                    opportunity.before.push(FragmentationBreakControl {
                        value: if self.case.nested() {
                            FragmentationBreak::AvoidPage
                        } else {
                            FragmentationBreak::AvoidColumn
                        },
                        role: FragmentationRole::Normal,
                        source_order: 1,
                    });
                    opportunity.ancestors.push(FragmentationBreakInside::Avoid);
                }
            }
        }
        Ok(reply)
    }
}

fn origin_input<S: LayoutScalar>(
    case: OriginCase,
    parent: bool,
) -> Arc<FragmentationInputOf<Key, S>> {
    let mut input = if parent {
        (*parent_input::<S>(0.0)).clone()
    } else {
        FragmentationInputOf {
            contexts: contexts(),
            flows: vec![FragmentationFlowOf {
                context: 0,
                start_fragmentainer: 0,
                initial_block_offset: S::ZERO,
                occurrences: (0..2)
                    .map(|index| FragmentationOccurrence {
                        key: Key::Child(index),
                        source_extent: 1,
                        monolithic: true,
                    })
                    .collect(),
            }],
        }
    };
    if case.nested() {
        input.contexts.push(FragmentationContext {
            kind: FragmentationKind::Column,
            parent: Some(0),
            flow_axes: input.contexts[0].flow_axes,
        });
        input.flows[0].context = 1;
    }
    Arc::new(input)
}

fn selected_opportunity_policy_survives_embedding<S: LayoutScalar>() {
    for case in [
        OriginCase::Sibling,
        OriginCase::ParentAvoid,
        OriginCase::InapplicableControl,
        OriginCase::ConditionalSibling,
        OriginCase::ConditionalLines,
        OriginCase::IntermediateWaiver,
        OriginCase::Lines,
    ] {
        let mut child = Execution::new(origin_input::<S>(case, false)).unwrap();
        let mut provider = OriginProvider {
            case,
            parent: None,
            embedding: 0,
        };
        while !child.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
        let child = Arc::new(child.finish());
        assert_eq!(child.report.breaks[0].position, position(1));
        let original = &child.selected_origins[0].as_ref().unwrap().origin;

        // Repeated duplicate parent facts and the intermediate Rule2 waiver
        // retain the original receipt without copying ancestor vectors.
        let mut completed = child.clone();
        for embedding in 0..3 {
            let mut execution = Execution::new(origin_input::<S>(case, true)).unwrap();
            let mut provider = OriginProvider {
                case,
                embedding,
                parent: Some(ParentProvider {
                    height: 30.0,
                    first_extent: 30.0,
                    extent: if matches!(case, OriginCase::IntermediateWaiver) {
                        30.0
                    } else {
                        60.0
                    },
                    closing: 0.0,
                    gap: 0.0,
                    ..ParentProvider::forced(0.0, matches!(case, OriginCase::IntermediateWaiver))
                }),
            };
            assert!(matches!(
                execution.poll(&mut provider, &mut |_| Ok(())),
                Err(FragmentationError::Provider("scope required"))
            ));
            let query = provider.parent.as_ref().unwrap().last_query.get().unwrap();
            let ticket = execution
                .scope_ticket(query, position(0), position(1))
                .unwrap();
            let projection = provider.parent.as_ref().unwrap().projection(0);
            execution
                .register_scope(ticket, completed, vec![projection], Vec::new())
                .unwrap();
            provider.parent.as_mut().unwrap().ready = true;
            while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
            let result = Arc::new(execution.finish());
            assert_eq!(result.report.pieces[0].block_size, number(30.0));
            assert_eq!(result.report.breaks[0].stage, case.expected());
            assert_eq!(
                result.report.breaks[0].reason,
                FragmentationBreakReason::Unforced
            );
            assert!(Arc::ptr_eq(
                original,
                &result.selected_origins[0].as_ref().unwrap().origin
            ));
            completed = result;
        }
    }
}

#[test]
fn selected_opportunity_policy_survives_embedding_f32() {
    selected_opportunity_policy_survives_embedding::<f32>();
}
#[test]
fn selected_opportunity_policy_survives_embedding_f64() {
    selected_opportunity_policy_survives_embedding::<f64>();
}

fn deferred_candidate<S: LayoutScalar>() {
    let completed_child = child::<S>(true, 100.0, 80.0, 0);
    assert_eq!(
        completed_child.report().breaks[0].reason,
        FragmentationBreakReason::Deferred
    );
    let mut execution = Execution::new(parent_input(80.0)).unwrap();
    let mut provider = ParentProvider::forced(80.0, false);
    provider.first_extent = 50.0;
    provider.extent = 130.0;
    let ticket = prepare(&mut execution, &mut provider);
    let query = provider.last_query.get().unwrap();
    let stale = execution
        .scope_ticket(query, position(0), position(1))
        .unwrap();
    assert_eq!(
        execution.register_scope(
            ticket,
            completed_child.clone(),
            vec![provider.projection(0)],
            Vec::new()
        ),
        Err(ScopeRegistrationError::InvalidProjection)
    );
    let ticket = execution
        .scope_ticket(query, position(0), position(1))
        .unwrap();
    execution
        .register_scope(ticket, completed_child.clone(), Vec::new(), Vec::new())
        .unwrap();
    provider.ready = true;
    assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
    assert!(execution.report.pieces.is_empty());
    assert_eq!(
        execution.report.breaks[0].reason,
        FragmentationBreakReason::Deferred
    );
    assert!(execution.scopes.is_empty());
    assert_eq!(
        Arc::strong_count(&completed_child),
        1,
        "deferred candidate releases its child receipt"
    );
    assert_eq!(
        execution.register_scope(stale, completed_child, Vec::new(), Vec::new()),
        Err(ScopeRegistrationError::StalePlacement)
    );
    provider.ready = false;
    let fresh = prepare(&mut execution, &mut provider);
    assert_eq!(provider.last_query.get().unwrap().fragmentainer_index, 1);
    execution
        .register_scope(
            fresh,
            child::<S>(true, 100.0, 0.0, 1),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
}

fn feedback_rollback<S: LayoutScalar>() {
    let mut execution = Execution::new(parent_input(0.0)).unwrap();
    let mut provider = ParentProvider::forced(0.0, false);
    let ticket = prepare(&mut execution, &mut provider);
    execution
        .register_scope(
            ticket,
            child::<S>(true, 100.0, 0.0, 0),
            vec![provider.projection(0)],
            Vec::new(),
        )
        .unwrap();
    provider.ready = true;
    assert!(matches!(
        execution.poll(&mut provider, &mut |_| Err("reject adoption")),
        Err(FragmentationError::Provider("reject adoption"))
    ));
    assert!(execution.report.pieces.is_empty());
    assert!(execution.report.breaks.is_empty());
    assert!(!execution.scopes[0].active);
    assert!(!execution.scopes[0].applied[0]);
    assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
    assert_eq!(execution.report.breaks.len(), 1);
}

fn inactive_replacement<S: LayoutScalar>() {
    for replace in [false, true] {
        let original = child::<S>(true, 50.0, 0.0, 0);
        let replacement = child::<S>(false, 50.0, 0.0, 0);
        let mut execution = Execution::new(parent_input(0.0)).unwrap();
        let mut provider = ParentProvider {
            height: 50.0,
            first_extent: 50.0,
            extent: 80.0,
            closing: 0.0,
            gap: 20.0,
            ..ParentProvider::forced(0.0, false)
        };
        let ticket = prepare(&mut execution, &mut provider);
        execution
            .register_scope(ticket, original, vec![provider.projection(0)], Vec::new())
            .unwrap();
        let query = provider.last_query.get().unwrap();
        let invalid = execution
            .scope_ticket(query, position(0), position(1))
            .unwrap();
        assert_eq!(
            execution.register_scope(
                invalid,
                replacement.clone(),
                vec![provider.projection(usize::MAX)],
                Vec::new()
            ),
            Err(ScopeRegistrationError::InvalidProjection)
        );
        if replace {
            let valid = execution
                .scope_ticket(query, position(0), position(1))
                .unwrap();
            execution
                .register_scope(valid, replacement, vec![provider.projection(0)], Vec::new())
                .unwrap();
        }
        provider.ready = true;
        while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
        let report = execution.finish().into_report();
        assert_eq!(report.breaks.len(), 1);
        assert_eq!(
            report.breaks[0].reason,
            if replace {
                FragmentationBreakReason::Unforced
            } else {
                FragmentationBreakReason::Forced {
                    required: FragmentationBreakKinds::PAGE,
                    side: None,
                }
            }
        );
        assert_eq!(
            report.breaks[0].stage,
            if replace {
                FragmentationStage::RelaxAvoidance
            } else {
                FragmentationStage::Strict
            }
        );
    }
}

fn active_replacement_refused<S: LayoutScalar>() {
    let mut execution = Execution::new(parent_input(0.0)).unwrap();
    let mut provider = ParentProvider::forced(0.0, false);
    let ticket = prepare(&mut execution, &mut provider);
    execution
        .register_scope(
            ticket,
            child::<S>(true, 100.0, 0.0, 0),
            vec![provider.projection(0)],
            Vec::new(),
        )
        .unwrap();
    provider.ready = true;
    assert!(!execution.poll(&mut provider, &mut |_| Ok(())).unwrap());
    provider.ready = false;
    let ticket = prepare(&mut execution, &mut provider);
    assert_eq!(
        execution.register_scope(
            ticket,
            child::<S>(true, 100.0, 0.0, 1),
            Vec::new(),
            Vec::new()
        ),
        Err(ScopeRegistrationError::InvalidOwner)
    );
    provider.ready = true;
    while !execution.poll(&mut provider, &mut |_| Ok(())).unwrap() {}
    let report = execution.finish().into_report();
    assert_eq!(report.breaks.len(), 1);
    assert!(
        matches!(report.breaks[0].reason, FragmentationBreakReason::Forced { required, side: None }
        if required == FragmentationBreakKinds::PAGE)
    );
}

#[test]
fn contextual_controls_use_existing_applicability() {
    let control = |value, role| FragmentationBreakControl {
        value,
        role,
        source_order: 0,
    };
    let page = control(FragmentationBreak::Page, FragmentationRole::Normal);
    let column = control(FragmentationBreak::Column, FragmentationRole::Normal);
    let absolute = control(FragmentationBreak::Page, FragmentationRole::Absolute);
    assert!(controls_require_break(
        &[page],
        &[],
        FragmentationBreakKinds::PAGE
    ));
    assert!(controls_require_break(
        &[],
        &[page],
        FragmentationBreakKinds::PAGE
    ));
    assert!(!controls_require_break(
        &[column],
        &[],
        FragmentationBreakKinds::PAGE
    ));
    assert!(!controls_require_break(
        &[absolute],
        &[],
        FragmentationBreakKinds::PAGE
    ));
    assert!(!controls_require_break(
        &[page],
        &[],
        FragmentationBreakKinds::COLUMN
    ));
}

#[test]
fn inactive_replacement_validates_before_discarding_prior_selection_f32() {
    inactive_replacement::<f32>();
}
#[test]
fn inactive_replacement_validates_before_discarding_prior_selection_f64() {
    inactive_replacement::<f64>();
}
#[test]
fn accepted_owner_refuses_replacement_and_keeps_selected_force_f32() {
    active_replacement_refused::<f32>();
}
#[test]
fn accepted_owner_refuses_replacement_and_keeps_selected_force_f64() {
    active_replacement_refused::<f64>();
}

#[test]
fn certified_zero_gap_force_keeps_later_overflow_independent_f32() {
    forced_scope::<f32>(false, 0.0);
}
#[test]
fn certified_zero_gap_force_keeps_later_overflow_independent_f64() {
    forced_scope::<f64>(false, 0.0);
}
#[test]
fn certified_principal_end_force_precedes_closing_f32() {
    forced_scope::<f32>(true, 0.0);
}
#[test]
fn certified_principal_end_force_precedes_closing_f64() {
    forced_scope::<f64>(true, 0.0);
}
#[test]
fn certified_offset_is_fragmentainer_relative_f32() {
    forced_scope::<f32>(false, 10.0);
}
#[test]
fn certified_offset_is_fragmentainer_relative_f64() {
    forced_scope::<f64>(false, 10.0);
}
#[test]
fn projected_gap_preserves_child_relaxation_floor_f32() {
    projected_stage_floor::<f32>();
}
#[test]
fn projected_gap_preserves_child_relaxation_floor_f64() {
    projected_stage_floor::<f64>();
}
#[test]
fn pre_owner_deferral_discards_ticket_and_allows_rebinding_f32() {
    deferred_candidate::<f32>();
}
#[test]
fn pre_owner_deferral_discards_ticket_and_allows_rebinding_f64() {
    deferred_candidate::<f64>();
}
#[test]
fn failed_adoption_replays_without_consuming_scope_force_f32() {
    feedback_rollback::<f32>();
}
#[test]
fn failed_adoption_replays_without_consuming_scope_force_f64() {
    feedback_rollback::<f64>();
}

#[path = "scope_offered_policy_tests.rs"]
mod offered_policy_tests;
