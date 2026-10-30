//! Ordinary final admission validates native-only generated child proofs
//! against genuine captured empty boxes, without inventing source scope owners.
use super::*;

struct DirectPairFixture<S: layout::LayoutScalar> {
    host: PairedEarly<S>,
    binding: layout::TextParagraphBinding<u32>,
    installed: Vec<Option<layout::LayoutInputOf<S>>>,
}
fn native_order<Scope>(
    prefix: layout::ChildOrderCompleteness,
) -> Vec<layout::InlineChildOrderInput<u32, Scope>> {
    let child = |box_| layout::GeneratedChildBoxInput {
        box_,
        participation: layout::GeneratedBoxParticipation::InFlow,
    };
    let pair =
        || layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::NativePair { start: 3, end: 4 });
    vec![
        layout::InlineChildOrderInput {
            parent: layout::GeneratedBoxRef::Node(0),
            children: vec![child(layout::GeneratedBoxRef::Node(1)), child(pair())],
            prefix: layout::ChildOrderCompleteness::Complete,
            suffix: layout::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        },
        layout::InlineChildOrderInput {
            parent: pair(),
            children: vec![child(layout::GeneratedBoxRef::Node(2))],
            prefix,
            suffix: layout::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        },
    ]
}
fn native_only_input(direction: layout::Direction, empty: bool) -> text::ContextualSourceInput {
    use text::SourceProcessingEvent as E;
    let mut raw = unscoped_native_pair_input(direction);
    assert_eq!(raw.scopes.len(), 1);
    assert!(raw.computed_line_boundaries.is_empty());
    if empty {
        raw.scopes.push(text::SourceProcessingScope::Inline {
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: None,
            boundaries: None,
        });
    }
    // Literal original event construction; there is no second event parser or
    // inferred pair. The genuine native pair is still explicitly declared.
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: text::Affinity::After,
        },
        E::Connection(0),
        E::Control {
            control: 0,
            affinity: text::Affinity::After,
        },
    ];
    if empty {
        events.extend([E::EnterScope(1), E::ExitScope(1)]);
    }
    events.extend([
        E::Origin {
            origin: 1,
            affinity: text::Affinity::After,
        },
        E::Control {
            control: 1,
            affinity: text::Affinity::Before,
        },
        E::ExitParagraph(0),
        E::ExitScope(0),
    ]);
    raw.events = events;
    raw
}
fn direct_pair_fixture<S: layout::LayoutScalar>(
    direction: layout::Direction,
    empty: bool,
    prefix: layout::ChildOrderCompleteness,
) -> DirectPairFixture<S> {
    let mut host = PairedEarly::<S>::new(direction, false);
    host.declarations
        .retain(|row| matches!(&row.box_, layout::InlineBoxRef::NativePair { .. }));
    host.declarations[0].edges = layout::WrapEdges::AUTO;
    host.scopes.clear();
    host.base.orders = native_order(prefix);
    // This is direct lower Text processing: no source-wrapper receipt or
    // computed request participates in the claimed native child-order proof.
    let processed =
        Arc::new(text::process_contextual_source(native_only_input(direction, empty)).unwrap());
    assert!(processed.captured_computed_line_boundary(0).is_none());
    assert_eq!(processed.source().text(), &text::Text::from("AB"));
    assert_eq!(processed.processed_text().code_units(), [0x41, 0x42]);
    let start = processed.captured_control(0).unwrap();
    let end = processed.captured_control(1).unwrap();
    let b = processed.captured_origin(1).unwrap();
    assert_eq!(
        start.kind(),
        &text::ParagraphControlKind::InlineBoundaryStart
    );
    assert_eq!(end.kind(), &text::ParagraphControlKind::InlineBoundaryEnd);
    assert!(start.event_position() < b);
    assert!(b < end.event_position());
    assert!(processed.captured_control(2).is_none());
    assert_eq!(
        processed.captured_scope(0).unwrap().domain_topology(),
        text::SourceProcessingTopology::Complete
    );
    if empty {
        let e = processed.captured_scope(1).unwrap();
        assert_eq!(e.parent(), processed.captured_scope(0));
        assert!(e.entry_control().is_none());
        assert!(e.exit_control().is_none());
        assert!(start.event_position() < e.enter_position());
        assert!(e.enter_position() < e.exit_position());
        assert!(e.exit_position() < processed.captured_origin(1).unwrap());
        assert!(processed.captured_scope(2).is_none());
    } else {
        assert!(processed.captured_scope(1).is_none());
    }
    let prepared = normal_prepare(&processed);
    assert_eq!(prepared.control_views().count(), 2);
    assert!(
        !prepared
            .opportunity_views()
            .any(|o| o.kind() == text::PreparedBreakKind::Forced)
    );
    let mut binding = host.base.install(&prepared);
    assert_eq!(binding.nodes, [1, 3, 2, 4]);
    let mut installed = vec![None, None, None, None, None];
    for origin in &binding.origins {
        installed[origin.node as usize] = host.base.installed[origin.node as usize].clone();
    }
    for view in prepared.control_views() {
        let handle = view.handle();
        let captured = handle.captured_control().unwrap();
        let (node, input) = if captured == start {
            (3, host.start)
        } else {
            assert_eq!(captured, end);
            (4, host.end)
        };
        assert_eq!(input.metrics().baseline(), S::ZERO);
        assert_eq!(input.metrics().line_extent(), S::ZERO);
        installed[node as usize] = Some(layout::LayoutInputOf::inline_boundary(input));
        binding.controls.push(layout::TextControlBinding {
            occurrence: handle,
            node,
        });
    }
    assert_eq!(binding.controls.len(), 2);
    DirectPairFixture {
        host,
        binding,
        installed,
    }
}
fn admit_native_order<S: layout::LayoutScalar>(
    empty: bool,
    prefix: layout::ChildOrderCompleteness,
    should_admit: bool,
) {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        let fixture = direct_pair_fixture::<S>(direction, empty, prefix);
        let tree = FinalUnscopedPair {
            host: &fixture.host,
            installed: fixture.installed,
        };
        let mut wrapping = final_pair_wrapping(&fixture.host);
        assert_eq!(wrapping.scopes.len(), 1);
        assert!(matches!(
            wrapping.scopes[0],
            layout::WrappingScopeInput::NativePair { .. }
        ));
        wrapping.child_order = native_order(prefix);
        let result =
            layout::LayoutFlowInput::try_new(&tree, 0, vec![fixture.binding], wrapping, Vec::new());
        if should_admit {
            match result {
                Ok(_flow) => {}
                Err(error) => panic!(
                    "the claimed native slice and its exact side completeness are valid: {error:?}"
                ),
            }
        } else {
            let error = match result {
                Ok(_) => panic!("complete N[B] cannot omit the genuine preceding empty child E"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error.kind(),
                    layout::LayoutErrorKindOf::InvalidInput(
                        layout::LayoutInvalidInputOf::WrappingScope(
                            layout::WrappingScopeError::ChildOrder
                        )
                    )
                ),
                "{error:?}"
            );
        }
    }
}
#[test]
fn native_only_complete_child_order_admits_without_any_source_inline_scope() {
    admit_native_order::<f32>(false, layout::ChildOrderCompleteness::Complete, true);
    admit_native_order::<f64>(false, layout::ChildOrderCompleteness::Complete, true);
}
#[test]
fn native_only_complete_prefix_refuses_omitted_real_empty_captured_child() {
    admit_native_order::<f32>(true, layout::ChildOrderCompleteness::Complete, false);
    admit_native_order::<f64>(true, layout::ChildOrderCompleteness::Complete, false);
}
#[test]
fn native_only_unknown_prefix_permits_unclaimed_empty_child_with_complete_suffix() {
    admit_native_order::<f32>(true, layout::ChildOrderCompleteness::Unknown, true);
    admit_native_order::<f64>(true, layout::ChildOrderCompleteness::Unknown, true);
}
