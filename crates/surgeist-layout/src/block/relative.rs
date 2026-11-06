use super::*;
use crate::output::{GeometryStateOf, MeasuredInsetBasisOf};
use crate::sizing::resolve::select_relative_insets;

/// Measurement may issue provisional visual allocations. A canonical frame
/// requires its actual accepted measured content basis, including a true zero.
#[derive(Clone, Copy)]
pub(super) enum InsetReference<S: LayoutScalar> {
    Measurement(Size<Option<S>>),
    Accepted(Option<MeasuredInsetBasisOf<S>>),
}

fn depends_on_basis<S: LayoutScalar>(inset: Edges<LengthAutoOf<S>>) -> bool {
    [inset.top, inset.right, inset.bottom, inset.left]
        .into_iter()
        .any(|value| value.depends_on_basis())
}

pub(super) fn needs_basis<S: LayoutScalar>(
    inset: Edges<LengthAutoOf<S>>,
    axes: crate::FlowAxes,
) -> bool {
    depends_on_basis(select_relative_insets(inset, axes))
}

/// Only roles whose relative visual placement is owned by the Block pass.
pub(super) fn has_pending_children<'input, Tree, M>(
    tree: &Tree,
    children: &[Tree::Node],
    axes: crate::FlowAxes,
) -> bool
where
    Tree: Compute<'input, M>,
{
    children.iter().any(|child| {
        let crate::LayoutInputViewOf::Box(input) = tree.layout_input(*child) else {
            return false;
        };
        input.display != crate::Display::None
            && input.position != Position::Absolute
            && input.float.is_none()
            && needs_basis(input.inset, axes)
    })
}

pub(super) fn resolve<'input, Tree, S, M>(
    tree: &Tree,
    child: Tree::Node,
    inset: Edges<LengthAutoOf<S>>,
    reference: InsetReference<S>,
    axes: crate::FlowAxes,
) -> LayoutResultOf<Tree::Node, Point<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let inset = select_relative_insets(inset, axes);
    let size = match reference {
        InsetReference::Measurement(size) => size,
        InsetReference::Accepted(Some(basis)) => basis.size().map(Some),
        InsetReference::Accepted(None) if depends_on_basis(inset) => {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(child),
                LayoutOperation::ValueResolution,
                LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis),
            ));
        }
        InsetReference::Accepted(None) => Size::NONE,
    };
    Ok(super::in_flow::relative_inset_offset(
        inset
            .zip_size(size, resolve_auto_optional)
            .transpose_with_node(tree, child)?,
        axes,
    ))
}

/// Settle only actual native percentage operands after the performed normal
/// result is complete. Their visual contributions have not yet been unioned.
/// Captured atomics are already sealed by the Text converter and are untouched.
pub(super) fn settle_children<'input, Tree, S, M>(
    tree: &mut Tree,
    container: Tree::Node,
    children: &[Tree::Node],
    basis: MeasuredInsetBasisOf<S>,
    axes: crate::FlowAxes,
    contributions: &mut scroll::BlockScrollContributions<S>,
) -> LayoutResultOf<Tree::Node, (), S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    for &child in children {
        let crate::LayoutInputViewOf::Box(input) = tree.layout_input(child) else {
            continue;
        };
        if input.display == crate::Display::None
            || input.position == Position::Absolute
            || !input.float.is_none()
            || !needs_basis(input.inset, axes)
        {
            continue;
        }
        let entry = tree.staged_unrounded(child)?;
        let measured_normal = match entry.geometry_state() {
            GeometryStateOf::OrdinaryUnrounded { measured_normal } => measured_normal,
            GeometryStateOf::CanonicalAtomic { .. } => continue,
            _ => {
                return Err(LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(child),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::InternalInvariant(
                        crate::LayoutInternalInvariant::InvalidOutputGeometryPhase,
                    ),
                ));
            }
        };
        let offset = resolve(
            tree,
            child,
            input.inset,
            InsetReference::Accepted(Some(basis)),
            axes,
        )?;
        let mut visual = entry.output();
        visual.location = Point::new(
            crate::group_alignment::add::<S>(
                measured_normal.location.x.to_f64(),
                offset.x.to_f64(),
            )
            .map(S::from_f64)
            .map_err(|error| error.layout::<Tree::Node, S, M>(child))?,
            crate::group_alignment::add::<S>(
                measured_normal.location.y.to_f64(),
                offset.y.to_f64(),
            )
            .map(S::from_f64)
            .map_err(|error| error.layout::<Tree::Node, S, M>(child))?,
        );
        let geometry = visual.scroll_geometry.ok_or_else(|| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::ContainerSubject {
                    container,
                    subject: child,
                },
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InternalInvariant(
                    crate::LayoutInternalInvariant::MissingStagedUnroundedOutput,
                ),
            )
        })?;
        contributions
            .include_in_flow_geometry(visual.location, visual.margin, geometry)
            .map_err(|error| layout_child_geometry_error(container, child, error))?;
        tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
            child,
            measured_normal,
            visual,
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "relative/percentage_basis_refusal_tests.rs"]
mod percentage_basis_refusal_tests;
