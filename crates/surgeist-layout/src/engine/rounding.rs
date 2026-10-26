use super::contracts::{Round, UnroundedInlineFragmentState};
use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf,
};
use crate::scalar::round_layout_coordinate;
use crate::scroll::rounding::rebuild_rounded_canonical_scroll_geometry;
use crate::{LayoutScalar, Point, Traverse};

pub(crate) fn round_layout<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Round<M>,
{
    round_layout_inner(tree, root, Tree::Scalar::ZERO, Tree::Scalar::ZERO)
}

fn round_layout_inner<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    cumulative_x: Tree::Scalar,
    cumulative_y: Tree::Scalar,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Round<M>,
{
    let unrounded = tree.unrounded(node)?;
    if let Some(origin) = tree.canonical_text_atomic_origin(node) {
        // This actual atomic allocation was finalized against the same line
        // anchor. Reapplying independent endpoint snapping would invalidate it.
        // This owning allocation is already canonical. Only its actual CSS
        // pixel phase influences descendant/local scroll endpoint rounding;
        // retaining the outer f64 translation avoids narrowing a large origin.
        let scalar_phase = Point::new(
            Tree::Scalar::from_f64(origin.x - origin.x.floor()),
            Tree::Scalar::from_f64(origin.y - origin.y.floor()),
        );
        let mut layout = unrounded;
        layout.scroll_geometry = unrounded
            .scroll_geometry
            .map(|geometry| rebuild_rounded_canonical_scroll_geometry(geometry, scalar_phase))
            .transpose()
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::RoundingFinalization,
                    LayoutErrorKindOf::InternalInvariant(
                        LayoutInternalInvariant::InvalidRoundedScrollGeometry,
                    ),
                )
            })?;
        tree.set_final(node, layout);
        for index in 0..tree.child_count(node) {
            round_layout_inner(
                tree,
                tree.child(node, index),
                scalar_phase.x,
                scalar_phase.y,
            )?;
        }
        return Ok(());
    }
    if tree.is_prepared_text(node) {
        // The new Text path supplies geometry from one canonical line anchor.
        // Its local metrics and fragments must never be independently snapped.
        let fragments = match tree.unrounded_inline_fragment_state(node) {
            UnroundedInlineFragmentState::Present(fragments) => fragments.to_vec(),
            UnroundedInlineFragmentState::Absent => Vec::new(),
        };
        tree.set_final(node, unrounded);
        tree.set_final_inline_fragments(node, fragments.clone(), fragments);
        return Ok(());
    }
    let mut layout = unrounded;
    let cumulative_x = cumulative_x + unrounded.location.x;
    let cumulative_y = cumulative_y + unrounded.location.y;

    layout.location.x = round_layout_coordinate(unrounded.location.x);
    layout.location.y = round_layout_coordinate(unrounded.location.y);
    layout.size.width = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x);
    layout.size.height = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y);
    layout.content_size.width =
        round_layout_coordinate(cumulative_x + unrounded.content_size.width)
            - round_layout_coordinate(cumulative_x);
    layout.content_size.height =
        round_layout_coordinate(cumulative_y + unrounded.content_size.height)
            - round_layout_coordinate(cumulative_y);
    layout.border.left = round_layout_coordinate(cumulative_x + unrounded.border.left)
        - round_layout_coordinate(cumulative_x);
    layout.border.right = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x + unrounded.size.width - unrounded.border.right);
    layout.border.top = round_layout_coordinate(cumulative_y + unrounded.border.top)
        - round_layout_coordinate(cumulative_y);
    layout.border.bottom = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y + unrounded.size.height - unrounded.border.bottom);
    layout.padding.left = round_layout_coordinate(cumulative_x + unrounded.padding.left)
        - round_layout_coordinate(cumulative_x);
    layout.padding.right = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x + unrounded.size.width - unrounded.padding.right);
    layout.padding.top = round_layout_coordinate(cumulative_y + unrounded.padding.top)
        - round_layout_coordinate(cumulative_y);
    layout.padding.bottom = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y + unrounded.size.height - unrounded.padding.bottom);
    let scroll_geometry = unrounded
        .scroll_geometry
        .map(|geometry| {
            rebuild_rounded_canonical_scroll_geometry(
                geometry,
                Point::new(cumulative_x, cumulative_y),
            )
        })
        .transpose()
        .map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::InvalidRoundedScrollGeometry,
                ),
            )
        })?;
    layout = layout.with_scroll_geometry(scroll_geometry);

    let fragment_phases = match tree.unrounded_inline_fragment_state(node) {
        UnroundedInlineFragmentState::Absent => None,
        UnroundedInlineFragmentState::Present(fragments) => {
            let unrounded_fragments = fragments.to_vec();
            let final_fragments = fragments.to_vec();
            Some((unrounded_fragments, final_fragments))
        }
    };

    tree.set_final(node, layout);
    if let Some((unrounded_fragments, final_fragments)) = fragment_phases {
        tree.set_final_inline_fragments(node, unrounded_fragments, final_fragments);
    }

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        round_layout_inner(tree, child, cumulative_x, cumulative_y)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_layout_round_boundaries<S: LayoutScalar>() {
        for (value, expected) in [
            (-2.0, -2.0),
            (-1.51, -2.0),
            (-1.5, -1.0),
            (-1.49, -1.0),
            (-0.51, -1.0),
            (-0.5, 0.0),
            (-0.49, 0.0),
            (0.0, 0.0),
            (0.49, 0.0),
            (0.5, 1.0),
            (0.51, 1.0),
            (1.49, 1.0),
            (1.5, 2.0),
            (1.51, 2.0),
            (2.0, 2.0),
            (-1_048_576.5, -1_048_576.0),
            (1_048_576.5, 1_048_577.0),
        ] {
            assert_eq!(
                round_layout_coordinate(S::from_f64(value)),
                S::from_f64(expected)
            );
        }
    }

    #[test]
    fn nearest_css_pixel_scalar_boundaries_keep_half_open_ties() {
        assert_layout_round_boundaries::<f32>();
        assert_layout_round_boundaries::<f64>();
    }

    #[test]
    fn actual_text_coordinate_narrowing_refuses_overflow_and_nonzero_underflow() {
        use crate::text_layout::{fit::FitError, run::number};
        assert!(matches!(
            number::<f32>(f64::MAX),
            Err(FitError::Numeric(
                _,
                crate::LayoutTextNumericError::OutOfRange
            ))
        ));
        assert!(matches!(
            number::<f32>(f64::from_bits(1)),
            Err(FitError::Numeric(
                _,
                crate::LayoutTextNumericError::NonZeroLost
            ))
        ));
        assert_eq!(
            number::<f64>(f64::from_bits(1)).ok(),
            Some(f64::from_bits(1))
        );
    }
}
