use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;

const CONTAINER: u32 = 0;
const ATOMIC: u32 = 2;
const TEXT_SLOT: u32 = 1;

fn request<S: LayoutScalar>(direction: Direction) -> LayoutRootRequestOf<S> {
    let extent = |value| NonNegativeFiniteOf::new(S::from_f64(value)).unwrap();
    LayoutRootRequestOf::positioned_root_local(
        RootLocalPositionedContextOf::horizontal_auto_insets(
            Size::new(extent(400.0), extent(300.0)),
            direction,
            S::from_f64(9.0),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Unknown,
                Size::splat(OrthogonalFallbackLimitOf::Unknown),
            ),
        )
        .unwrap(),
    )
    .unwrap()
}

fn container<S: LayoutScalar>(block_size: PreferredSizeOf<S>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        position: Position::Absolute,
        size: Size::new(PreferredSizeOf::px(S::from_f64(180.0)), block_size),
        ..NodeInputOf::default()
    }
}

fn atomic<S: LayoutScalar>() -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::InlineBlock,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(20.0)),
            PreferredSizeOf::percent(S::from_f64(0.5)),
        ),
        ..NodeInputOf::default()
    }
}

#[test]
fn definite_inline_atomic_uses_the_actual_authored_block_basis() {
    fn lane<S: LayoutScalar>() {
        for (block, expected) in [(80.0, 40.0), (60.0, 30.0), (0.0, 0.0)] {
            let tree = PublicLayoutTreeOf::new()
                .style(
                    CONTAINER,
                    container(PreferredSizeOf::px(S::from_f64(block))),
                )
                .style(ATOMIC, atomic())
                // Text has no prepared input yet. Its identity is present
                // among the actual siblings, and its input must not be read.
                .children(CONTAINER, [TEXT_SLOT, ATOMIC]);
            let size = measure_definite_inline_atomic_size(
                &tree,
                CONTAINER,
                ATOMIC,
                request(Direction::Ltr),
            )
            .unwrap();
            assert_eq!(size, Size::new(S::from_f64(20.0), S::from_f64(expected)));
        }

        let tree = PublicLayoutTreeOf::new()
            .style(CONTAINER, container(PreferredSizeOf::px(S::from_f64(80.0))))
            .style(
                ATOMIC,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::px(S::ZERO)),
                    ..atomic()
                },
            )
            .children(CONTAINER, [ATOMIC]);
        assert_eq!(
            measure_definite_inline_atomic_size(&tree, CONTAINER, ATOMIC, request(Direction::Ltr))
                .unwrap(),
            Size::ZERO
        );
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn definite_inline_atomic_uses_the_content_box_basis_and_production_child_constraints() {
    fn lane<S: LayoutScalar>() {
        let scalar = S::from_f64;
        // 80px border-box, with 4px border and 12px padding vertically,
        // supplies a 64px percentage basis. The child adds 6px padding
        // to its 32px content, or to its clamped 30px content.
        let parent = NodeInputOf {
            direction: Direction::Rtl,
            box_sizing: BoxSizing::BorderBox,
            padding: Edges::all(LengthOf::px(scalar(6.0))),
            border: Edges::all(LengthOf::px(scalar(2.0))),
            ..container(PreferredSizeOf::px(scalar(80.0)))
        };
        let child = NodeInputOf {
            direction: Direction::Rtl,
            box_sizing: BoxSizing::ContentBox,
            padding: Edges::new(
                LengthOf::px(scalar(3.0)),
                LengthOf::ZERO,
                LengthOf::px(scalar(3.0)),
                LengthOf::ZERO,
            ),
            atomic_inline_participation: Some(AtomicInlineParticipation::new(
                BidiLevel::try_new(1).unwrap(),
                InlineBreakOpportunity::Allowed,
            )),
            ..atomic()
        };
        for (maximum, border_box_height) in
            [(MaxSizeOf::NONE, 38.0), (MaxSizeOf::px(scalar(30.0)), 36.0)]
        {
            let tree = PublicLayoutTreeOf::new()
                .style(CONTAINER, parent.clone())
                .style(
                    ATOMIC,
                    NodeInputOf {
                        max_size: Size::new(MaxSizeOf::NONE, maximum),
                        ..child.clone()
                    },
                )
                .children(CONTAINER, [ATOMIC]);
            let request = request(Direction::Rtl);
            let expected = Size::new(scalar(20.0), scalar(border_box_height));
            assert_eq!(
                measure_definite_inline_atomic_size(&tree, CONTAINER, ATOMIC, request).unwrap(),
                expected
            );
            let composition = compute_layout(&tree, CONTAINER, request).unwrap();
            let child = composition
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == ATOMIC)
                .unwrap();
            assert_eq!(child.output().size, expected);
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn definite_inline_atomic_keeps_vertical_block_basis_on_the_physical_width() {
    fn lane<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let tree = PublicLayoutTreeOf::new()
            .style(
                CONTAINER,
                NodeInputOf {
                    writing_mode: WritingMode::VerticalRl,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(80.0)),
                        PreferredSizeOf::px(scalar(180.0)),
                    ),
                    ..container(PreferredSizeOf::AUTO)
                },
            )
            .style(
                ATOMIC,
                NodeInputOf {
                    writing_mode: WritingMode::VerticalRl,
                    size: Size::new(
                        PreferredSizeOf::percent(scalar(0.5)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..atomic()
                },
            )
            .children(CONTAINER, [ATOMIC]);
        assert_eq!(
            measure_definite_inline_atomic_size(&tree, CONTAINER, ATOMIC, request(Direction::Ltr))
                .unwrap(),
            Size::new(scalar(40.0), scalar(20.0))
        );
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn definite_inline_atomic_refuses_missing_frame_and_invalid_incidence() {
    fn lane<S: LayoutScalar>() {
        let request = request(Direction::Ltr);
        let definite = PreferredSizeOf::px(S::from_f64(80.0));
        let tree = PublicLayoutTreeOf::new()
            .style(CONTAINER, container(definite.clone()))
            .style(ATOMIC, atomic())
            .children(CONTAINER, [TEXT_SLOT, ATOMIC]);

        let error = measure_definite_inline_atomic_size(&tree, CONTAINER, 3, request).unwrap_err();
        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: CONTAINER,
                subject: 3
            }
        );
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineAtomicNotDirectChild)
        ));

        let duplicate = tree.clone().children(CONTAINER, [ATOMIC, ATOMIC]);
        let error = measure_definite_inline_atomic_size(&duplicate, CONTAINER, ATOMIC, request)
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineAtomicNotDirectChild)
        ));

        let cycle = tree.clone().children(CONTAINER, [ATOMIC, CONTAINER]);
        let error =
            measure_definite_inline_atomic_size(&cycle, CONTAINER, ATOMIC, request).unwrap_err();
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TreeTopologyCycle)
        ));

        for style in [
            container(PreferredSizeOf::AUTO),
            NodeInputOf {
                size: Size::new(PreferredSizeOf::AUTO, definite.clone()),
                ..container(definite)
            },
        ] {
            let indefinite = tree.clone().style(CONTAINER, style);
            let error =
                measure_definite_inline_atomic_size(&indefinite, CONTAINER, ATOMIC, request)
                    .unwrap_err();
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
            ));
        }

        let nonempty = tree.clone().children(ATOMIC, [3]);
        let unmeasured_scope = [nonempty, tree.clone().measure(ATOMIC, Size::ZERO)];
        for unsupported in unmeasured_scope {
            let error =
                measure_definite_inline_atomic_size(&unsupported, CONTAINER, ATOMIC, request)
                    .unwrap_err();
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::UnsupportedCapability(
                    LayoutUnsupportedCapability::InlineAtomicSizePreparation
                )
            ));
        }

        let standalone = LayoutRootRequestOf::standalone(
            request.sizing_request(),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::NONE,
        );
        let error =
            measure_definite_inline_atomic_size(&tree, CONTAINER, ATOMIC, standalone).unwrap_err();
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::UnsupportedCapability(
                LayoutUnsupportedCapability::InlineAtomicSizePreparation
            )
        ));
    }
    lane::<f32>();
    lane::<f64>();
}
