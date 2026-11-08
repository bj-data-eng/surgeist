//! Accepted visual-inset authority for content without a principal box.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContentTrialId(pub(crate) usize);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContentInsetBinding<Node, S: LayoutScalar> {
    pub(crate) owner: Node,
    pub(crate) role: crate::BlockGeneration,
    pub(crate) target: crate::LayoutTargetBasis,
    pub(crate) placement: crate::engine::block_scope::ReceiptId,
    pub(crate) trial: ContentTrialId,
    pub(crate) frame: usize,
    pub(crate) axes: crate::FlowAxes,
    pub(crate) start: crate::FragmentationCutOf<S>,
    pub(crate) end: crate::FragmentationCutOf<S>,
}

/// Issued from successful Normal content production. Its inline extent grants
/// visual-inset authority only; it does not establish a sizing percentage basis.
pub(crate) struct AcceptedContentInsetReceipt<Node, S: LayoutScalar> {
    binding: ContentInsetBinding<Node, S>,
    inline: S,
}

fn invalid_binding<Node, S: LayoutScalar, M>(owner: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(owner),
        LayoutOperation::ValueResolution,
        LayoutErrorKindOf::BlockFragmentation(crate::BlockFragmentationFailure::InvalidSource),
    )
}

/// The successful production carries its own extent; a separately prepared
/// context cannot be paired with it to replace measured visual authority.
pub(crate) fn accepted_inline_basis<Node: Copy + Eq, S: LayoutScalar, M>(
    normal: &content::Receipt<Node, S>,
    expected: &ContentInsetBinding<Node, S>,
) -> LayoutResultOf<Node, AcceptedContentInsetReceipt<Node, S>, S, M> {
    if normal.node != expected.owner
        || normal.role != expected.role
        || normal.binding.as_ref() != Some(expected)
        || normal.axes != expected.axes
        || !normal.normal_complete
    {
        return Err(invalid_binding(expected.owner));
    }
    Ok(AcceptedContentInsetReceipt {
        binding: expected.clone(),
        inline: normal.accepted_inline_inner,
    })
}

impl<Node: Copy + Eq, S: LayoutScalar> AcceptedContentInsetReceipt<Node, S> {
    /// `selected` has already crossed the relative-position conflict reducer.
    pub(super) fn selected_basis<M>(
        &self,
        subject: Node,
        selected: Edges<LengthAutoOf<S>>,
        expected: &ContentInsetBinding<Node, S>,
    ) -> LayoutResultOf<Node, Size<Option<S>>, S, M> {
        if self.binding != *expected {
            return Err(invalid_binding(expected.owner));
        }
        let basis = expected
            .axes
            .physical_size(LogicalSizeOf::new(Some(self.inline), None));
        for (axis, values, extent) in [
            (
                PhysicalAxis::Horizontal,
                [selected.left, selected.right],
                basis.width,
            ),
            (
                PhysicalAxis::Vertical,
                [selected.top, selected.bottom],
                basis.height,
            ),
        ] {
            if extent.is_none() && values.into_iter().any(LengthAutoOf::depends_on_basis) {
                return Err(LayoutErrorOf::new(
                    LayoutErrorSiteOf::FragmentContainingBlock {
                        container: expected.owner,
                        subject,
                        fragmentainer_index: expected.frame,
                        axis,
                    },
                    LayoutOperation::ValueResolution,
                    LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis),
                ));
            }
        }
        Ok(basis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ComputeSession;
    use crate::test_support::layout_tree::PublicLayoutTreeOf;

    fn input<S: LayoutScalar>(axes: crate::FlowAxes, inline: f64) -> ComputeInputOf<S> {
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            axes.physical_size(LogicalSizeOf::new(Some(S::from_f64(inline)), None)),
            axes.physical_size(LogicalSizeOf::new(Some(S::from_f64(inline)), None)),
            ContainingLayoutContext::new(axes, ParentFormattingContext::BlockFlow),
            crate::SizingRequestOf::new(
                Size::splat(crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                )),
                crate::OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        )
    }

    fn binding<S: LayoutScalar>(axes: crate::FlowAxes) -> ContentInsetBinding<u32, S> {
        let request = input::<S>(axes, 80.0);
        let target = crate::LayoutBoundRequestOf::continuous(
            crate::LayoutSourceBasis::fresh(),
            crate::LayoutRootRequestOf::standalone(request.sizing_request(), axes, Size::NONE),
        )
        .target_basis();
        ContentInsetBinding {
            owner: 0,
            role: crate::BlockGeneration::Principal,
            target,
            placement: crate::engine::block_scope::ReceiptId(3),
            trial: ContentTrialId(7),
            frame: 2,
            axes,
            start: crate::FragmentationCutOf::Source(crate::FragmentationPosition {
                occurrence: 0,
                offset: 0,
            }),
            end: crate::FragmentationCutOf::Source(crate::FragmentationPosition {
                occurrence: 1,
                offset: 0,
            }),
        }
    }

    fn tree<S: LayoutScalar>(
        axes: crate::FlowAxes,
        inset: Edges<LengthAutoOf<S>>,
    ) -> PublicLayoutTreeOf<S> {
        PublicLayoutTreeOf::new()
            .style(
                0,
                crate::NodeInputOf {
                    display: crate::Display::Block,
                    writing_mode: axes.writing_mode(),
                    direction: axes.direction(),
                    ..Default::default()
                },
            )
            .style(
                1,
                crate::NodeInputOf {
                    display: crate::Display::Block,
                    writing_mode: axes.writing_mode(),
                    direction: axes.direction(),
                    inset,
                    size: Size::new(
                        crate::PreferredSizeOf::px(S::from_f64(10.0)),
                        crate::PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    ..Default::default()
                },
            )
            .children(0, [1])
    }

    struct Produced<'a, S: LayoutScalar> {
        context: content::Context<S>,
        normal: content::Receipt<u32, S>,
        session: ComputeSession<'a, PublicLayoutTreeOf<S>>,
    }

    fn produce<'a, S: LayoutScalar>(
        tree: &'a PublicLayoutTreeOf<S>,
        key: &ContentInsetBinding<u32, S>,
        inline: f64,
    ) -> Produced<'a, S> {
        let mut session = ComputeSession::new(tree, Vec::new());
        let context = content::prepare(
            &mut session,
            0,
            input(key.axes, inline),
            crate::LayoutTree::node_input(tree, 0),
        )
        .unwrap();
        let normal = content::produce(
            &mut session,
            0,
            &context,
            content::Projection {
                block_reference: S::ZERO,
                content_origin: S::ZERO,
            },
            Some(key.clone()),
        )
        .unwrap();
        Produced {
            context,
            normal,
            session,
        }
    }

    fn horizontal<S: LayoutScalar>() {
        let axes = crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr);
        let key = binding::<S>(axes);
        let inset = Edges {
            left: LengthAutoOf::percent(S::from_f64(0.1)),
            ..Edges::all(LengthAutoOf::AUTO)
        };
        let tree = tree(axes, inset);
        let mut produced = produce(&tree, &key, 80.0);
        let basis =
            accepted_inline_basis::<_, _, core::convert::Infallible>(&produced.normal, &key)
                .unwrap();
        relative::settle_content_children(
            &mut produced.session,
            0,
            &[1],
            &basis,
            &key,
            &mut produced.normal.flow.contributions,
        )
        .unwrap();
        let child = produced.session.staged_unrounded(1).unwrap();
        assert_eq!(
            child.output().location,
            Point::new(S::from_f64(8.0), S::ZERO)
        );
        let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
            child.geometry_state()
        else {
            panic!("settlement retains normal geometry");
        };
        assert_eq!(measured_normal.location, Point::ZERO);
        let bounds = produced
            .normal
            .flow
            .contributions
            .geometry
            .descendant_bounds()
            .unwrap()
            .unwrap();
        assert_eq!(bounds.origin(), Point::new(S::from_f64(8.0), S::ZERO));
        assert_eq!(bounds.size(), Size::splat(S::from_f64(10.0)));

        for value in [
            LengthAutoOf::percent(S::from_f64(0.1)),
            LengthAutoOf::value(
                crate::LengthPercentageOf::from_coefficients(S::from_f64(2.0), S::from_f64(0.1))
                    .unwrap(),
            ),
        ] {
            let inset = Edges {
                top: value,
                left: LengthAutoOf::percent(S::from_f64(0.1)),
                ..Edges::all(LengthAutoOf::AUTO)
            };
            let error =
                relative::resolve_content(&produced.session, 1, inset, &basis, &key).unwrap_err();
            assert_eq!(
                error.site(),
                LayoutErrorSiteOf::FragmentContainingBlock {
                    container: 0,
                    subject: 1,
                    fragmentainer_index: 2,
                    axis: PhysicalAxis::Vertical
                }
            );
            assert_eq!(error.operation(), LayoutOperation::ValueResolution);
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis)
            ));
        }
        for (inset, expected) in [
            (
                Edges {
                    top: LengthAutoOf::percent(S::ZERO),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                Point::ZERO,
            ),
            (
                Edges {
                    top: LengthAutoOf::px(S::from_f64(2.0)),
                    bottom: LengthAutoOf::percent(S::from_f64(0.1)),
                    left: LengthAutoOf::px(S::from_f64(3.0)),
                    right: LengthAutoOf::percent(S::from_f64(0.1)),
                },
                Point::new(S::from_f64(3.0), S::from_f64(2.0)),
            ),
            (
                Edges {
                    top: LengthAutoOf::value(
                        crate::LengthPercentageOf::from_coefficients(S::from_f64(4.0), S::ZERO)
                            .unwrap(),
                    ),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                Point::new(S::ZERO, S::from_f64(4.0)),
            ),
        ] {
            assert_eq!(
                relative::resolve_content(&produced.session, 1, inset, &basis, &key).unwrap(),
                expected
            );
        }
        for stale in [
            ContentInsetBinding {
                frame: key.frame + 1,
                ..key.clone()
            },
            ContentInsetBinding {
                trial: ContentTrialId(key.trial.0 + 1),
                ..key.clone()
            },
            ContentInsetBinding {
                placement: crate::engine::block_scope::ReceiptId(99),
                ..key.clone()
            },
            ContentInsetBinding {
                target: binding::<S>(axes).target,
                ..key.clone()
            },
            ContentInsetBinding {
                role: crate::BlockGeneration::Anonymous,
                ..key.clone()
            },
            ContentInsetBinding {
                axes: crate::FlowAxes::new(crate::WritingMode::VerticalRl, Direction::Ltr),
                ..key.clone()
            },
            ContentInsetBinding {
                end: crate::FragmentationCutOf::Source(crate::FragmentationPosition {
                    occurrence: 2,
                    offset: 0,
                }),
                ..key.clone()
            },
        ] {
            let error =
                relative::resolve_content(&produced.session, 1, inset, &basis, &stale).unwrap_err();
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::BlockFragmentation(
                    crate::BlockFragmentationFailure::InvalidSource
                )
            ));
            assert!(
                accepted_inline_basis::<_, _, core::convert::Infallible>(&produced.normal, &stale)
                    .is_err()
            );
        }
        produced.normal.binding = None;
        assert!(
            accepted_inline_basis::<_, _, core::convert::Infallible>(&produced.normal, &key)
                .is_err()
        );
        produced.normal.binding = Some(key.clone());
        produced.normal.normal_complete = false;
        assert!(
            accepted_inline_basis::<_, _, core::convert::Infallible>(&produced.normal, &key)
                .is_err()
        );

        let zero = produce(&tree, &key, 0.0);
        let zero_basis =
            accepted_inline_basis::<_, _, core::convert::Infallible>(&zero.normal, &key).unwrap();
        assert_eq!(
            relative::resolve_content(&zero.session, 1, inset, &zero_basis, &key).unwrap(),
            Point::ZERO
        );
    }

    fn vertical<S: LayoutScalar>() {
        for mode in [
            crate::WritingMode::VerticalRl,
            crate::WritingMode::VerticalLr,
        ] {
            let axes = crate::FlowAxes::new(mode, Direction::Ltr);
            let key = binding::<S>(axes);
            let inset = Edges {
                top: LengthAutoOf::percent(S::from_f64(0.1)),
                ..Edges::all(LengthAutoOf::AUTO)
            };
            let tree = tree(axes, inset);
            let produced = produce(&tree, &key, 80.0);
            let basis =
                accepted_inline_basis::<_, _, core::convert::Infallible>(&produced.normal, &key)
                    .unwrap();
            assert_eq!(
                relative::resolve_content(&produced.session, 1, inset, &basis, &key).unwrap(),
                Point::new(S::ZERO, S::from_f64(8.0))
            );
            let inset = Edges {
                left: LengthAutoOf::percent(S::from_f64(0.1)),
                ..Edges::all(LengthAutoOf::AUTO)
            };
            let error =
                relative::resolve_content(&produced.session, 1, inset, &basis, &key).unwrap_err();
            assert_eq!(
                error.site(),
                LayoutErrorSiteOf::FragmentContainingBlock {
                    container: 0,
                    subject: 1,
                    fragmentainer_index: 2,
                    axis: PhysicalAxis::Horizontal
                }
            );
        }
    }

    fn ordinary_nested<S: LayoutScalar>() {
        let axes = crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr);
        let box_style = |height, percentage| crate::NodeInputOf {
            display: crate::Display::Block,
            size: Size::new(
                crate::PreferredSizeOf::px(S::from_f64(20.0)),
                crate::PreferredSizeOf::px(S::from_f64(height)),
            ),
            inset: Edges {
                top: if percentage {
                    LengthAutoOf::percent(S::from_f64(0.1))
                } else {
                    LengthAutoOf::AUTO
                },
                ..Edges::all(LengthAutoOf::AUTO)
            },
            ..Default::default()
        };
        let tree = PublicLayoutTreeOf::new()
            .style(0, box_style(40.0, false))
            .style(1, box_style(20.0, true))
            .style(2, box_style(10.0, true))
            .children(0, [1])
            .children(1, [2]);
        let mut session = ComputeSession::new(&tree, Vec::new());
        let output =
            super::super::compute_block_scoped(&mut session, 0, input(axes, 80.0)).unwrap();
        assert_eq!(output.size.height, S::from_f64(40.0));
        assert_eq!(
            session.staged_unrounded(1).unwrap().output().location.y,
            S::from_f64(4.0)
        );
        assert_eq!(
            session.staged_unrounded(2).unwrap().output().location.y,
            S::from_f64(2.0)
        );
    }

    fn production_owned_basis<S: LayoutScalar>() {
        let axes = crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr);
        let key = binding::<S>(axes);
        let inset = Edges {
            left: LengthAutoOf::percent(S::from_f64(0.1)),
            ..Edges::all(LengthAutoOf::AUTO)
        };
        let tree = tree(axes, inset);
        let narrow = produce(&tree, &key, 80.0);
        let wide = produce(&tree, &key, 160.0);
        assert_eq!(narrow.context.inline_outer, S::from_f64(80.0));
        assert_eq!(wide.context.inline_outer, S::from_f64(160.0));
        // A later same-owner, same-key context cannot supply the earlier body's
        // basis: issuance accepts only the completed Normal receipt itself.
        let narrow_basis =
            accepted_inline_basis::<_, _, core::convert::Infallible>(&narrow.normal, &key).unwrap();
        let wide_basis =
            accepted_inline_basis::<_, _, core::convert::Infallible>(&wide.normal, &key).unwrap();
        assert_eq!(
            relative::resolve_content(&narrow.session, 1, inset, &narrow_basis, &key).unwrap(),
            Point::new(S::from_f64(8.0), S::ZERO),
        );
        assert_eq!(
            relative::resolve_content(&wide.session, 1, inset, &wide_basis, &key).unwrap(),
            Point::new(S::from_f64(16.0), S::ZERO),
        );
    }

    #[test]
    fn accepted_content_insets_use_only_bound_inline_extent_f32() {
        horizontal::<f32>();
    }
    #[test]
    fn accepted_content_insets_use_only_bound_inline_extent_f64() {
        horizontal::<f64>();
    }
    #[test]
    fn accepted_content_insets_follow_vertical_containing_axes_f32() {
        vertical::<f32>();
    }
    #[test]
    fn accepted_content_insets_follow_vertical_containing_axes_f64() {
        vertical::<f64>();
    }
    #[test]
    fn ordinary_and_nested_principal_insets_keep_measured_block_basis_f32() {
        ordinary_nested::<f32>();
    }
    #[test]
    fn ordinary_and_nested_principal_insets_keep_measured_block_basis_f64() {
        ordinary_nested::<f64>();
    }
    #[test]
    fn accepted_inline_basis_belongs_to_actual_normal_production_f32() {
        production_owned_basis::<f32>();
    }
    #[test]
    fn accepted_inline_basis_belongs_to_actual_normal_production_f64() {
        production_owned_basis::<f64>();
    }
}
