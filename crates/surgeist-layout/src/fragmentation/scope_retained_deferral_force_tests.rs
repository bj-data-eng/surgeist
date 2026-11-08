//! A raw force before Owner precedes a speculative retained child Deferred.
use super::*;
struct ForcedPrefix<S: LayoutScalar>(PrefixParent<S>);
impl<S: LayoutScalar> FragmentationProvider<S> for ForcedPrefix<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        flow: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        self.0.fragmentainer(flow, index)
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let mut reply = self.0.layout(query)?;
        if let FragmentationReplyOf::Measured(window) = &mut reply {
            for opportunity in &mut window.opportunities {
                if opportunity.cut == FragmentationCutOf::Source(position(1))
                    && opportunity.block_offset == number(100.0)
                {
                    opportunity.before.push(FragmentationBreakControl {
                        value: FragmentationBreak::Page,
                        role: FragmentationRole::Normal,
                        source_order: 1,
                    });
                }
            }
        }
        Ok(reply)
    }
}
fn raw_force_before_deferred<S: LayoutScalar>() {
    let mut input = (*parent_input::<S>(0.0)).clone();
    input.flows[0].occurrences.insert(
        0,
        FragmentationOccurrence {
            key: Key::Prefix,
            source_extent: 1,
            monolithic: true,
        },
    );
    let mut parent = Execution::new(Arc::new(input)).unwrap();
    let mut provider = ForcedPrefix(PrefixParent {
        ready: false,
        avoid: false,
        query: Cell::new(None),
    });
    assert!(matches!(
        parent.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let child = deferred_children::<S>(false);
    let ticket = parent
        .scope_ticket(provider.0.query.get().unwrap(), position(1), position(2))
        .unwrap();
    parent
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
    provider.0.ready = true;
    let mut appearances = 0;
    assert!(
        !parent
            .poll(&mut provider, &mut |event| {
                appearances += event.appearances.len();
                Ok(())
            })
            .unwrap()
    );
    let first = &parent.report.breaks[0];
    assert_eq!(
        first.reason,
        FragmentationBreakReason::Forced {
            required: FragmentationBreakKinds::PAGE,
            side: None
        }
    );
    assert_eq!(first.stage, FragmentationStage::Strict);
    assert_eq!((first.from_fragmentainer, first.to_fragmentainer), (0, 1));
    assert_eq!(parent.report.pieces[0].block_size, number(100.0));
    assert_eq!(appearances, 0);
    assert!(!parent.scope_participated(position(1), &child));
}
#[test]
fn raw_prefix_force_precedes_retained_deferred_f32() {
    raw_force_before_deferred::<f32>();
}
#[test]
fn raw_prefix_force_precedes_retained_deferred_f64() {
    raw_force_before_deferred::<f64>();
}
