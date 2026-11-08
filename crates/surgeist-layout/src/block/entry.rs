//! Entry facts that do not require the completed descendant body.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StartFact<S: LayoutScalar> {
    pub(crate) margin: CollapsibleMarginOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Qualification<S: LayoutScalar> {
    Known(StartFact<S>),
    RequiresBody,
}

pub(super) struct StartReduction<S: LayoutScalar> {
    pub(super) offset: S,
    pub(super) exposes_child: bool,
}

/// The same first participating-child rule is used for placement and entry
/// qualification. Through-collapse and terminal margin facts are body-owned.
pub(super) fn reduce_start<S: LayoutScalar>(
    first: bool,
    collapses_start: bool,
    child_collapses: bool,
    active: CollapsibleMarginOf<S>,
    child_start: CollapsibleMarginOf<S>,
) -> StartReduction<S> {
    let exposes_child = first && collapses_start && child_collapses;
    StartReduction {
        offset: if exposes_child {
            active.resolve()
        } else {
            active.collapse_with(child_start).resolve()
        },
        exposes_child,
    }
}

pub(crate) fn qualify<'input, Tree, M>(
    tree: &mut Tree,
    mut node: Tree::Node,
    mut input: ComputeInputOf<Tree::Scalar>,
    mut reporting_side: PhysicalSide,
) -> LayoutResultOf<Tree::Node, Qualification<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut pending = Vec::new();
    let mut fact;
    loop {
        let raw = tree.node_input(node);
        let own = crate::FlowAxes::new(raw.writing_mode, raw.direction);
        if tree.has_leaf_measurement(node)
            || raw.display.inner_display() != crate::Display::Block
            || own.block_axis() != reporting_side.axis()
        {
            // These producers report no descendant block-start collapse set.
            fact = StartFact {
                margin: CollapsibleMarginOf::ZERO,
            };
            break;
        }
        let contained = containment_sizing(node, raw, input, tree.fragment_minimum(node))?
            .map_or(Size::NONE, |value| value.outer);
        let style = block_container_projection::<Tree, M>(tree, node);
        let constants =
            Constants::from_projection::<Tree, M>(tree, node, &style, input, contained)?;
        if own.block_start() != reporting_side {
            if own.block_end() == reporting_side && !constants.collapse_bottom_margin {
                fact = StartFact {
                    margin: constants.own_bottom_margin,
                };
                break;
            }
            return Ok(Qualification::RequiresBody);
        }
        if !constants.collapse_top_margin {
            fact = StartFact {
                margin: constants.own_top_margin,
            };
            break;
        }
        let mut first = None;
        for child in tree.children(node) {
            let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
            let InlineParticipantKindOf::Box(style) = projection.kind() else {
                return Err(intrinsic::profile_error(child));
            };
            if style.display == crate::Display::None || style.common.position == Position::Absolute
            {
                continue;
            }
            if !style.float.is_none() || style.display.is_inline_level() {
                return Err(intrinsic::profile_error(child));
            }
            first = Some(child);
            break;
        }
        let Some(child) = first else {
            fact = StartFact {
                margin: CollapsibleMarginOf::ZERO,
            };
            break;
        };
        let outer =
            intrinsic::accepted_inline(tree, node, &style, &constants, input, |tree, op| {
                if let Some(value) = tree.fragment_inline_content(node, input, op)? {
                    Ok(value)
                } else {
                    intrinsic::content(tree, node, &constants, input, op)
                }
            })?;
        let inner = constants.accepted_inline_inner(node, outer)?;
        let constants = constants.with_logical_node_inner_size(
            LogicalSizeOf::new(Some(inner), constants.logical_node_inner_size().block),
            false,
        );
        let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
        let InlineParticipantKindOf::Box(child_style) = projection.kind() else {
            unreachable!("the participating child was qualified as a box")
        };
        let mut first_in_flow = true;
        let prepared = in_flow::prepare_child(
            tree,
            child,
            &child_style,
            &constants,
            in_flow::ChildSizingContext {
                inner_size: constants.node_inner_size,
                request: constants.sizing_request,
                run_mode: RunMode::ComputeSize,
                axis: RequestedAxis::Both,
                first_in_flow: &mut first_in_flow,
            },
        )?;
        pending.push((
            constants
                .flow_axes
                .logical_edges(prepared.unresolved_margin)
                .block_start,
            in_flow::child_margin_can_collapse_with_parent(&child_style),
        ));
        node = child;
        input = prepared.input;
        reporting_side = constants.flow_axes.block_start();
    }
    while let Some((own, child_collapses)) = pending.pop() {
        let start = fact
            .margin
            .collapse_with_margin(in_flow::resolved_length_auto_fallback_zero(own));
        let reduction = reduce_start(
            true,
            true,
            child_collapses,
            CollapsibleMarginOf::ZERO,
            start,
        );
        fact = StartFact {
            margin: if reduction.exposes_child {
                start
            } else {
                CollapsibleMarginOf::ZERO
            },
        };
    }
    Ok(Qualification::Known(fact))
}

#[cfg(test)]
#[path = "entry_tests.rs"]
mod tests;
