//! Equal checkpoint/geometry does not erase an offered site's policy identity.
use super::*;

struct DuplicateOffered<S: LayoutScalar> {
    parent: ParentProvider<S>,
    auto_first: bool,
}
impl<S: LayoutScalar> FragmentationProvider<S> for DuplicateOffered<S> {
    type Error = &'static str;
    fn fragmentainer(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        self.parent.fragmentainer(context, index)
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error> {
        let mut reply = self.parent.layout(query)?;
        if let FragmentationReplyOf::Measured(window) = &mut reply
            && let Some(site) = window.opportunities.first().cloned()
        {
            // The owning window contract admits equal cut/offset opportunities
            // with different facts. Both describe the actual principal-end30 cut.
            window.opportunities = if self.auto_first {
                [FragmentationBreak::Auto, FragmentationBreak::AvoidPage]
            } else {
                [FragmentationBreak::AvoidPage, FragmentationBreak::Auto]
            }
            .into_iter()
            .map(|value| {
                let mut offered = site.clone();
                offered.before = vec![FragmentationBreakControl {
                    value,
                    role: FragmentationRole::Normal,
                    source_order: 1,
                }];
                offered
            })
            .collect();
        }
        Ok(reply)
    }
}

fn offered_policy_identity<S: LayoutScalar>(auto_first: bool) {
    // Execute actual two-unit content30+30 in frames30. The all-Auto Sibling
    // at source1 is selected Unforced/Strict after accepting child0's source.
    let mut child = Execution::new(origin_input::<S>(OriginCase::Sibling, false)).unwrap();
    let mut content = OriginProvider {
        case: OriginCase::Sibling,
        parent: None,
        embedding: 0,
    };
    while !child.poll(&mut content, &mut |_| Ok(())).unwrap() {}
    let child = Arc::new(child.finish());
    assert_eq!(child.report.breaks.len(), 1);
    assert_eq!(child.report.breaks[0].position, position(1));
    assert_eq!(
        child.report.breaks[0].reason,
        FragmentationBreakReason::Unforced
    );
    assert_eq!(child.report.breaks[0].stage, FragmentationStage::Strict);
    assert_eq!(child.report.pieces[0].block_size, number(30.0));
    let source = &child.report.pieces[0].coverage[0];
    assert_eq!(source.key, Key::Child(0));
    assert_eq!((source.start, source.end), (0, 1));
    let original = child.selected_origins[0].as_ref().unwrap().origin.clone();

    let mut parent = Execution::new(parent_input::<S>(0.0)).unwrap();
    let mut provider = DuplicateOffered {
        parent: ParentProvider {
            ready: false,
            height: 30.0,
            initial_used: 0.0,
            first_extent: 30.0,
            // Principal30 ends after the first actual child frame; the
            // remaining child content has its separate accepted window.
            extent: 30.0,
            closing: 0.0,
            source_cut: true,
            gap: 0.0,
            last_query: Cell::new(None),
        },
        auto_first,
    };
    assert!(matches!(
        parent.poll(&mut provider, &mut |_| Ok(())),
        Err(FragmentationError::Provider("scope required"))
    ));
    let ticket = parent
        .scope_ticket(
            provider.parent.last_query.get().unwrap(),
            position(0),
            position(1),
        )
        .unwrap();
    parent
        .register_scope(
            ticket,
            child.clone(),
            vec![provider.parent.projection(0)],
            Vec::new(),
        )
        .unwrap();
    provider.parent.ready = true;
    assert!(
        !parent
            .poll(&mut provider, &mut |event| {
                assert_eq!(event.pieces.len(), 1);
                assert!(event.appearances.is_empty());
                Ok(())
            })
            .unwrap()
    );

    // Inspect actual adoption before the stage oracle, so an unexpected error
    // cannot be misattributed to duplicate-site policy selection.
    assert_eq!(parent.report.breaks.len(), 1);
    let selected = &parent.report.breaks[0];
    assert_eq!(selected.reason, FragmentationBreakReason::Unforced);
    assert_eq!(
        (selected.from_fragmentainer, selected.to_fragmentainer),
        (0, 1)
    );
    assert_eq!(parent.report.pieces[0].block_size, number(30.0));
    assert_eq!(
        parent.report.pieces[0].kind,
        FragmentationPieceKindOf::Content
    );
    assert_eq!(selected.position, position(1));
    assert_eq!(parent.accepted_source, vec![1]);
    assert!(parent.scope_participated(position(0), &child));
    assert_eq!(parent.scopes[0].applied, vec![true]);
    assert!(
        parent.scopes[0]
            .source_windows
            .iter()
            .any(|applied| *applied)
    );
    assert!(Arc::ptr_eq(
        &original,
        &parent.selected_origins[0].as_ref().unwrap().origin
    ));
    assert_eq!(
        selected.stage,
        FragmentationStage::Strict,
        "the legal Auto opportunity at index{} must retain Strict",
        usize::from(!auto_first)
    );
}
#[test]
fn offered_duplicate_policy_auto_first_control_f32() {
    offered_policy_identity::<f32>(true);
}
#[test]
fn offered_duplicate_policy_auto_first_control_f64() {
    offered_policy_identity::<f64>(true);
}
#[test]
fn offered_duplicate_policy_auto_second_strict_f32() {
    offered_policy_identity::<f32>(false);
}
#[test]
fn offered_duplicate_policy_auto_second_strict_f64() {
    offered_policy_identity::<f64>(false);
}
