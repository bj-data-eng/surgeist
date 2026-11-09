use super::*;

fn ten<S: LayoutScalar>(_: crate::LeafMeasureInputOf<S>) -> Size<S> {
    leaf(10.0)
}

fn growing_source<S: LayoutScalar>() {
    for count in [1_u32, 2, 4] {
        let terminal = 3 + count;
        let mut tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(2, 3..=terminal)
            .style(0, block(0.0, 0.0, None))
            .style(
                1,
                crate::NodeInputOf {
                    float: crate::Float::Left,
                    size: Size::new(
                        crate::PreferredSizeOf::px(S::from_f64(20.0)),
                        crate::PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    ..block(0.0, 0.0, None)
                },
            )
            .style(2, block(0.0, 0.0, None))
            .style(terminal, block(0.0, 0.0, None))
            .measure_with(terminal, ten::<S>);
        for node in 3..terminal {
            tree = tree.style(
                node,
                crate::NodeInputOf {
                    clear: crate::Clear::Left,
                    ..block(0.0, 0.0, None)
                },
            );
        }

        let mut owner = ComputeSession::new(&tree, Vec::new());
        let input = query(EntryPurpose::FramedEntry).input;
        let parent = block_container_projection::<_, core::convert::Infallible>(&owner, 0);
        let constants = Constants::from_projection::<_, core::convert::Infallible>(
            &owner,
            0,
            &parent,
            input,
            Size::NONE,
        )
        .unwrap();
        let axes = constants.flow_axes;
        let mut ledger = FloatExclusions::new(
            axes,
            Size::splat(S::from_f64(100.0)),
            S::from_f64(100.0),
            LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
        );
        let style = BlockChildProjection::from_node(owner.node_input(1));
        let mut first = true;
        let prepared = in_flow::prepare_child(
            &owner,
            1,
            &style,
            &constants,
            in_flow::ChildSizingContext {
                inner_size: constants.node_inner_size,
                request: constants.sizing_request,
                run_mode: RunMode::ComputeSize,
                axis: RequestedAxis::Both,
                first_in_flow: &mut first,
            },
        )
        .unwrap();
        let output = owner.compute_child(1, prepared.input).unwrap();
        assert_eq!(output.size, Size::splat(S::from_f64(20.0)));
        let pending = PendingFloat {
            placement: super::super::super::super::floats::PendingPlacement::Unresolved,
            node: 1,
            source_index: 0,
            side: style.float,
            clear: style.clear,
            block_start: S::ZERO,
            size: output.size,
            named_baselines: output.named_baselines,
            content_size: output.content_size,
            border: prepared.border,
            padding: prepared.padding,
            margin: Edges::all(S::ZERO),
            float_exclusion: style.float_exclusion,
            child_compute_geometry: output.scroll_geometry,
        };
        let placement = ledger
            .place_float_for_line::<core::convert::Infallible>(0, &pending, None)
            .unwrap();
        assert_eq!(placement.logical.block, S::ZERO);
        let frame =
            std::sync::Arc::new(ledger.for_ordinary_child(LogicalPointOf::new(S::ZERO, S::ZERO)));
        CALLS.with(|calls| calls.set(0));
        for _ in 0..2 {
            let mut request = query(EntryPurpose::FramedEntry);
            request.node = 2;
            request.purpose = EntryPurpose::Component;
            request.incoming = Some(float_context::Incoming {
                node: 2,
                request: input,
                frame: frame.clone(),
                cut: None,
            });
            request.position = Some(super::super::super::canonical::EntryPosition {
                cursor: S::ZERO,
                active: CollapsibleMarginOf::ZERO,
                start_open: false,
                collapses_start: false,
                child_adjoins_parent: true,
                own_start: CollapsibleMarginOf::ZERO,
                inline_offset: S::ZERO,
            });
            let EntryObservation::Summary(summary) = owner.qualify_entry(request).unwrap() else {
                panic!("finite source offers and the measured terminal complete entry");
            };
            let cut = summary.separation.unwrap();
            assert_eq!((cut.container, cut.node, cut.index), (2, 3, 0));
            assert_eq!(cut.hypothetical.get(), S::ZERO);
            assert_eq!(cut.relevant_bottom.get(), S::from_f64(20.0));
            // These calls are the actual host measurements. Repeating the same
            // complete entry request cannot resample its completed opaque demand.
            CALLS.with(|calls| assert_eq!(calls.get(), 1, "offers {count}"));
            assert!(owner.staged_unrounded(2).is_err());
            assert!(owner.staged_unrounded(terminal).is_err());
        }
        drop(owner);

        let width = crate::NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            axes,
            Size::new(Some(width), None),
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::new(
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Definite(width),
                crate::SizingOperation::Normal,
            ),
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::Normal,
            ),
        ));
        let batch = crate::compute_layout(&tree, 0, request).unwrap();
        let output = |node| {
            batch
                .node_layout(node)
                .unwrap()
                .single_box()
                .unwrap()
                .unrounded_output()
        };
        assert_eq!(output(0).size.height, S::from_f64(30.0));
        assert_eq!(output(1).location.y, S::ZERO);
        assert_eq!(output(1).size.height, S::from_f64(20.0));
        assert_eq!(output(2).location.y, S::ZERO);
        assert_eq!(output(2).size.height, S::from_f64(30.0));
        for node in 3..=terminal {
            assert_eq!(output(node).location.y, S::from_f64(20.0));
            assert_eq!(
                output(node).size.height,
                if node == terminal {
                    S::from_f64(10.0)
                } else {
                    S::ZERO
                }
            );
        }
    }
}

#[test]
fn growing_clearance_source_retains_geometry_and_opaque_samples_f32() {
    growing_source::<f32>();
}

#[test]
fn growing_clearance_source_retains_geometry_and_opaque_samples_f64() {
    growing_source::<f64>();
}
