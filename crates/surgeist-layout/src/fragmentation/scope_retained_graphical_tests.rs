//! A real zero-height child advances source between equal graphical checkpoints.
use super::*;

struct PlateauChildren;
impl<S: LayoutScalar> FragmentationProvider<S> for PlateauChildren {
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
        let heights = [30.0, 0.0, 30.0];
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: FragmentationWindowEnd::FlowEnd,
            block_size: number(heights[start..].iter().sum()),
            head: FragmentationMeasuredHeadOf {
                end: position(start + 1),
                block_size: number(heights[start]),
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities: (start + 1..3)
                .map(|index| FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Source(position(index)),
                    block_offset: number(heights[start..index].iter().sum()),
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
fn graphical_plateau<S: LayoutScalar>() {
    let input = Arc::new(FragmentationInputOf {
        contexts: contexts(),
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: 0,
            initial_block_offset: S::ZERO,
            occurrences: (0..3)
                .map(|index| FragmentationOccurrence {
                    key: Key::Child(index),
                    source_extent: 1,
                    monolithic: true,
                })
                .collect(),
        }],
    });
    let mut child = Execution::new(input).unwrap();
    while !child.poll(&mut PlateauChildren, &mut |_| Ok(())).unwrap() {}
    let child = Arc::new(child.finish());
    assert_eq!(child.report.breaks.len(), 2);
    assert_eq!(child.report.pieces[0].coverage[0].key, Key::Child(0));
    assert_eq!(child.report.pieces[1].coverage[0].key, Key::Child(1));
    assert_eq!(child.report.pieces[1].block_size, S::ZERO);
    for (index, transition) in child.report.breaks.iter().enumerate() {
        assert_eq!(
            transition.reason,
            FragmentationBreakReason::Forced {
                required: FragmentationBreakKinds::PAGE,
                side: None
            }
        );
        assert_eq!(transition.position, position(index + 1));
    }
    let mut parent = Execution::new(parent_input::<S>(0.0)).unwrap();
    let mut provider = ParentProvider {
        ready: false,
        height: 50.0,
        initial_used: 0.0,
        first_extent: 30.0,
        extent: 60.0,
        closing: 0.0,
        source_cut: false,
        gap: 0.0,
        last_query: Cell::new(None),
    };
    let ticket = prepare(&mut parent, &mut provider);
    let cut = provider.projection(0).endpoint.checkpoint();
    parent
        .register_scope(
            ticket,
            child.clone(),
            vec![
                provider.projection(0),
                ScopeProjectionOf {
                    child_break_index: 1,
                    endpoint: ScopeEndpointOf::Retain(cut),
                    block_offset: S::ZERO,

                    normal_end: S::ZERO,
                },
            ],
            Vec::new(),
        )
        .unwrap();
    provider.ready = true;
    assert!(!parent.poll(&mut provider, &mut |_| Ok(())).unwrap());
    assert_eq!(parent.report.breaks[0].to_fragmentainer, 1);
    let pieces = parent.report.pieces.len();
    let mut appearances = 0;
    assert!(
        !parent
            .poll(&mut provider, &mut |event| {
                appearances += event.appearances.len();
                Ok(())
            })
            .unwrap()
    );
    assert_eq!(parent.report.pieces.len(), pieces);
    assert_eq!(appearances, 1);
    assert_eq!(parent.report.breaks[1].to_fragmentainer, 2);
    assert_eq!(
        parent.report.breaks[1].reason,
        FragmentationBreakReason::Forced {
            required: FragmentationBreakKinds::PAGE,
            side: None
        }
    );
    assert!(parent.scope_participated(position(0), &child));
}
#[test]
fn retained_equal_graphical_actual_source_f32() {
    graphical_plateau::<f32>();
}
#[test]
fn retained_equal_graphical_actual_source_f64() {
    graphical_plateau::<f64>();
}
