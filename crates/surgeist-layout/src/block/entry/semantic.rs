//! Consume Text's issued semantic source-window evidence at Block entry.
use super::*;
use crate::text_layout::scope_alignment::{EntryFunding, SemanticWindowProof};
use surgeist_text as text;

/// The normal fitter and this adapter share endpoint allocation and the final
/// participation predicate. This observes no strut, font, line or float band.
pub(super) fn observe<'input, Tree, M>(
    tree: &mut Tree,
    container: Tree::Node,
    window: &EntryWindow,
    nodes: &[Tree::Node],
    constants: &Constants<Tree::Scalar>,
    funding: Option<&EntryFunding<Tree::Scalar>>,
) -> LayoutResultOf<Tree::Node, SemanticWindowProof, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let segment = window
        .segment
        .as_ref()
        .ok_or_else(|| float_context::missing(container))?;
    let error =
        |error: text::Error| super::super::inline_run::text_run_error(container, error.into());
    let (run, associations, resources) = {
        let state = tree
            .text_computation()
            .ok_or_else(|| float_context::missing(container))?;
        let run = if window.nodes.is_empty() {
            state.entry_semantic_run(container, segment, funding)
        } else {
            state.entry_run(container, nodes, funding)
        }
        .map_err(error)?
        .ok_or_else(|| float_context::missing(container))?;
        (
            run,
            state.associations.clone(),
            state.scope_resources.clone(),
        )
    };
    let prepared = &run.binding.prepared;
    let inspect = || -> text::Result<()> {
        if let Some(funding) = funding {
            funding.visit(1)?;
        }
        Ok(())
    };
    let start = segment.start().view().participant_offset();
    let end = segment.end().view().participant_offset();
    let selected = || prepared.participants().skip(start).take(end - start);
    let mut atomics = 0usize;
    let mut controls = 0usize;
    for participant in selected() {
        inspect().map_err(error)?;
        match participant {
            text::PreparedParticipant::Atomic(_) => atomics += 1,
            text::PreparedParticipant::Control(_) => controls += 1,
            _ => {}
        }
    }
    // Endpoint contexts describe real counterparts outside this selection.
    // Their allocations are source/box facts, not invented line contexts.
    let mut outside = 0usize;
    for binding in &run.binding.controls {
        inspect().map_err(error)?;
        if !matches!(
            tree.layout_input(binding.node),
            crate::LayoutInputViewOf::InlineBoundary(_)
        ) {
            continue;
        }
        let mut inside = false;
        for participant in selected() {
            inspect().map_err(error)?;
            if matches!(participant, text::PreparedParticipant::Control(ref value) if *value == binding.occurrence)
            {
                inside = true;
                break;
            }
        }
        if !inside {
            outside += 1;
        }
    }
    let mut template = text::SemanticWindowRequest::new(segment.clone());
    template.inline_boxes = run.inline_boxes.family().cloned();
    template.formatted_line =
        text::FormattedLineContext::new(Some(constants.first_formatted), None);
    let mut request = resources
        .fund_semantic_request(template, atomics, controls, outside)
        .map_err(error)?;
    let orientation =
        crate::text_layout::run::text_orientation(constants.flow_axes.writing_mode(), prepared)
            .map_err(|error| super::super::inline_run::text_run_error(container, error))?;
    let endpoints = super::super::inline_run::BoundaryAllocationContext {
        container,
        prepared,
        associations: &associations,
        resources: &resources,
        orientation,
        axes: constants.flow_axes,
    };
    for participant in selected() {
        inspect().map_err(error)?;
        match participant {
            text::PreparedParticipant::Atomic(occurrence) => {
                request
                    .push_atomic(text::SemanticAtomicAllocation {
                        kind: occurrence.view().original_box().kind(),
                        occurrence,
                    })
                    .map_err(error)?;
            }
            text::PreparedParticipant::Control(occurrence) => {
                let mut node = None;
                for binding in &run.binding.controls {
                    inspect().map_err(error)?;
                    if binding.occurrence == occurrence {
                        node = Some(binding.node);
                        break;
                    }
                }
                let node = node.ok_or_else(|| float_context::missing(container))?;
                let boundary = match tree.layout_input(node) {
                    crate::LayoutInputViewOf::InlineBoundary(input) => Some(endpoints.allocate(
                        tree,
                        node,
                        &occurrence,
                        crate::inline::InlineEndpointOf::from_input(input),
                    )?),
                    _ => None,
                };
                request
                    .push_control(text::SemanticControlAllocation {
                        occurrence,
                        boundary,
                    })
                    .map_err(error)?;
            }
            _ => {}
        }
    }
    for binding in &run.binding.controls {
        inspect().map_err(error)?;
        let crate::LayoutInputViewOf::InlineBoundary(input) = tree.layout_input(binding.node)
        else {
            continue;
        };
        let mut inside = false;
        for participant in selected() {
            inspect().map_err(error)?;
            if matches!(participant, text::PreparedParticipant::Control(ref value) if *value == binding.occurrence)
            {
                inside = true;
                break;
            }
        }
        if inside {
            continue;
        }
        let boundary = endpoints.allocate(
            tree,
            binding.node,
            &binding.occurrence,
            crate::inline::InlineEndpointOf::from_input(input),
        )?;
        request
            .push_boundary_context(text::InlineBoundaryContextAllocation {
                occurrence: binding.occurrence.clone(),
                boundary,
            })
            .map_err(error)?;
    }
    resources.observe_semantic(request).map_err(error)
}

pub(super) fn separates<S: LayoutScalar>(
    proof: &SemanticWindowProof,
    funding: Option<&EntryFunding<S>>,
) -> text::Result<bool> {
    let evidence = proof.evidence();
    let mut edge = false;
    for control in &evidence.request().controls {
        if let Some(funding) = funding {
            funding.visit(1)?;
        }
        if control
            .boundary
            .as_ref()
            .is_some_and(|boundary| boundary.participates())
        {
            edge = true;
            break;
        }
    }
    for partition in evidence.partitions() {
        if let Some(funding) = funding {
            funding.visit(1)?;
        }
        let end = if partition.terminal().view().kind() == text::PreparedBreakKind::Forced {
            text::LineEnd::Forced
        } else {
            text::LineEnd::Paragraph
        };
        if crate::text_layout::fit::LineDisposition::participates(
            end,
            partition.has_in_flow_content(),
            edge,
        ) {
            return Ok(true);
        }
    }
    Ok(false)
}
