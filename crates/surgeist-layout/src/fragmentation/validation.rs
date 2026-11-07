use super::*;
use crate::LayoutScalar;

pub(super) fn nonnegative<S: LayoutScalar>(value: S) -> bool {
    value.is_finite() && value >= S::ZERO
}

pub(super) fn input<K: Eq, S: LayoutScalar>(
    input: &FragmentationInputOf<K, S>,
) -> Result<(), FragmentationInputError> {
    for (index, context) in input.contexts.iter().enumerate() {
        let mut parent = context.parent;
        let mut depth = 0;
        while let Some(next) = parent {
            if next >= input.contexts.len() {
                return Err(FragmentationInputError::ContextReference);
            }
            if next == index || depth >= input.contexts.len() {
                return Err(FragmentationInputError::ContextCycle);
            }
            depth += 1;
            parent = input.contexts[next].parent;
        }
    }
    for flow in &input.flows {
        if flow.context >= input.contexts.len() {
            return Err(FragmentationInputError::ContextReference);
        }
        if !nonnegative(flow.initial_block_offset) {
            return Err(FragmentationInputError::InvalidGeometry);
        }
        for (index, occurrence) in flow.occurrences.iter().enumerate() {
            if flow.occurrences[..index]
                .iter()
                .any(|prior| prior.key == occurrence.key)
            {
                return Err(FragmentationInputError::DuplicateOccurrence);
            }
        }
    }
    Ok(())
}

pub(super) fn position<K, S: LayoutScalar>(
    flow: &FragmentationFlowOf<K, S>,
    mut position: FragmentationPosition,
) -> Result<FragmentationPosition, FragmentationInputError> {
    if position.occurrence == flow.occurrences.len() && position.offset == 0 {
        return Ok(position);
    }
    let Some(occurrence) = flow.occurrences.get(position.occurrence) else {
        return Err(FragmentationInputError::InvalidPosition);
    };
    if position.offset > occurrence.source_extent {
        return Err(FragmentationInputError::InvalidPosition);
    }
    // Zero-source occurrences still have ordered identity and must be consumed.
    if position.offset != 0 && position.offset == occurrence.source_extent {
        position.occurrence += 1;
        position.offset = 0;
    }
    Ok(position)
}

pub(super) fn window<K, S: LayoutScalar>(
    flow: &FragmentationFlowOf<K, S>,
    start: FragmentationPosition,
    window: &mut FragmentationWindowOf<S>,
) -> Result<(), FragmentationInputError> {
    if ![
        window.block_size,
        window.head_block_size,
        window.cloned_block_start,
        window.cloned_block_end,
    ]
    .into_iter()
    .all(nonnegative)
        || window.head_block_size > window.block_size
    {
        return Err(FragmentationInputError::InvalidWindow);
    }
    let end = FragmentationPosition {
        occurrence: flow.occurrences.len(),
        offset: 0,
    };
    for opportunity in &mut window.opportunities {
        opportunity.position = position(flow, opportunity.position)?;
        if opportunity.position < start
            || !nonnegative(opportunity.block_offset)
            || opportunity.block_offset > window.block_size
            || (opportunity.position == start && opportunity.block_offset != S::ZERO)
            || (opportunity.position == end && opportunity.block_offset != window.block_size)
        {
            return Err(FragmentationInputError::InvalidWindow);
        }
        match opportunity.kind {
            FragmentationOpportunityKindOf::Lines {
                orphans, widows, ..
            } if orphans == 0 || widows == 0 => {
                return Err(FragmentationInputError::InvalidLineCounts);
            }
            FragmentationOpportunityKindOf::ContentEdge { gap } if !nonnegative(gap) => {
                return Err(FragmentationInputError::InvalidContentGap);
            }
            _ => {}
        }
    }
    for (index, opportunity) in window.opportunities.iter().enumerate() {
        if window.opportunities[..index].iter().any(|prior| {
            prior.position == opportunity.position && prior.block_offset != opportunity.block_offset
        }) {
            return Err(FragmentationInputError::InvalidWindow);
        }
    }
    Ok(())
}
