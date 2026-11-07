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
) -> Result<FragmentationPosition, FragmentationInputError> {
    if ![
        window.block_size,
        window.head.block_size,
        window.cloned_block_start,
        window.cloned_block_end,
    ]
    .into_iter()
    .all(nonnegative)
        || window.head.block_size > window.block_size
    {
        return Err(FragmentationInputError::InvalidWindow);
    }
    let end = FragmentationPosition {
        occurrence: flow.occurrences.len(),
        offset: 0,
    };
    let covered_end = match window.end {
        FragmentationWindowEnd::FlowEnd => end,
        FragmentationWindowEnd::PotentialControl(raw) => {
            let terminal = position(flow, raw)?;
            if terminal <= start || terminal >= end {
                return Err(FragmentationInputError::InvalidWindow);
            }
            window.end = FragmentationWindowEnd::PotentialControl(terminal);
            terminal
        }
    };
    window.head.end = position(flow, window.head.end)?;
    let head_end = FragmentationPosition {
        occurrence: start.occurrence + 1,
        offset: 0,
    }
    .min(covered_end);
    if window.head.end != head_end {
        return Err(FragmentationInputError::InvalidWindow);
    }
    for opportunity in &mut window.opportunities {
        if matches!(opportunity.cut, FragmentationCutOf::Graphical { anchor, .. } if anchor.offset != 0)
        {
            return Err(FragmentationInputError::InvalidPosition);
        }
        let anchor = position(flow, opportunity.cut.source_position())?;
        match &mut opportunity.cut {
            FragmentationCutOf::Source(position) => *position = anchor,
            FragmentationCutOf::Graphical {
                anchor: position,
                progress,
            } => {
                *position = anchor;
                if anchor.occurrence >= flow.occurrences.len() || anchor.offset != 0 {
                    return Err(FragmentationInputError::InvalidPosition);
                }
                if flow.occurrences[anchor.occurrence].monolithic
                    || !nonnegative(progress.consumed)
                    || !nonnegative(progress.extent)
                    || progress.consumed <= S::ZERO
                    || progress.consumed >= progress.extent
                    || !matches!(
                        opportunity.kind,
                        FragmentationOpportunityKindOf::ContentEdge { .. }
                            | FragmentationOpportunityKindOf::ModeSpecific
                    )
                    || !opportunity.before.is_empty()
                    || !opportunity.after.is_empty()
                {
                    return Err(FragmentationInputError::InvalidSlice);
                }
            }
        }
        let source = matches!(opportunity.cut, FragmentationCutOf::Source(_));
        if anchor < start
            || anchor > covered_end
            || !nonnegative(opportunity.block_offset)
            || opportunity.block_offset > window.block_size
            || (source && anchor == start && opportunity.block_offset != S::ZERO)
            || (source && anchor == covered_end && opportunity.block_offset != window.block_size)
            || (!source && anchor == covered_end)
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
    if matches!(window.end, FragmentationWindowEnd::PotentialControl(_))
        && !window.opportunities.iter().any(|item| {
            item.cut == FragmentationCutOf::Source(covered_end)
                && matches!(
                    item.kind,
                    FragmentationOpportunityKindOf::Sibling
                        | FragmentationOpportunityKindOf::ModeSpecific
                )
        })
    {
        return Err(FragmentationInputError::InvalidWindow);
    }
    for opportunity in &window.opportunities {
        if let FragmentationCutOf::Graphical { anchor, progress } = opportunity.cut
            && window.opportunities.iter().any(|other| match other.cut {
                FragmentationCutOf::Source(position) => {
                    position.occurrence == anchor.occurrence && position.offset != 0
                }
                FragmentationCutOf::Graphical {
                    anchor: at,
                    progress: other,
                } => at == anchor && other.extent != progress.extent,
            })
        {
            return Err(FragmentationInputError::InvalidSlice);
        }
    }
    for (index, opportunity) in window.opportunities.iter().enumerate() {
        if window.opportunities[..index].iter().any(|prior| {
            prior.cut == opportunity.cut && prior.block_offset != opportunity.block_offset
        }) {
            return Err(FragmentationInputError::InvalidWindow);
        }
    }
    Ok(covered_end)
}
