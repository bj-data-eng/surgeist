//! Boundary trimming follows Block's actual flat collapse-contributor graph.
use super::*;
use crate::block::fragment_margins::{Contributor, MarginOwner};

pub(super) fn trimmed<Node: Copy + Eq, S: LayoutScalar, M>(
    scopes: &Scopes<Node>,
    key: &ViewKey,
    provenance: FragmentationStart,
    packet: &Packet<Node, S>,
    arena: &[Production<Node, S>],
    views: &Views,
) -> LayoutResultOf<Node, Vec<(usize, PhysicalSide)>, S, M> {
    let scope = key.scope;
    let frame = key.frame;
    let root = scopes.source.nodes[scopes.scopes[scope.0].owner].node;
    let invalid = || {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidSource),
        )
    };
    let text_error = |error| {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::Text(error),
        )
    };
    let forced = matches!(
        provenance,
        FragmentationStart::AfterBreak(FragmentationBreakReason::Forced { .. })
    );
    // Entry groups belong to the actual placement reducer. An opening principal
    // whose child timeline continues has already consumed these contributors;
    // a represented parent End is not an adjoining end of that principal.
    // Keep only temporary references/expansions of the one Block Join arena.
    let mut entries = Vec::new();
    let mut packets = vec![(None, packet, key)];
    while let Some((owning, actual, view)) = packets.pop() {
        let mut continuing = Vec::new();
        for (index, source) in scopes.source.nodes.iter().enumerate() {
            if scopes.range(view.scope, index).is_some_and(|local| {
                local.start >= view.range.start
                    && view.resumed != Some(local.start)
                    && local.start < view.range.end
                    && (local.end > view.range.end
                        || view.partial.is_some_and(|unit| local.end == unit + 1))
            }) && actual
                .entries
                .iter()
                .any(|entry| entry.node() == source.node)
            {
                continuing.push(index);
            }
        }
        for &(node, id) in &actual.projections {
            let production = arena.get(id.0).ok_or_else(invalid)?;
            let performed = views.get(production.view).ok_or_else(invalid)?;
            let (projection, pending) = production.projection.ok_or_else(invalid)?;
            if pending.node != node || performed.frame != frame {
                return Err(invalid());
            }
            if performed.opening && !projection.closing {
                continuing.push(
                    scopes
                        .source
                        .nodes
                        .iter()
                        .position(|source| source.node == node)
                        .ok_or_else(invalid)?,
                );
            }
            packets.push((Some(id), &production.packet, performed));
        }
        for mut index in continuing {
            loop {
                let source = &scopes.source.nodes[index];
                if !actual
                    .entries
                    .iter()
                    .any(|entry| entry.node() == source.node)
                {
                    break;
                }
                let contributors = actual.margins.entry(source.node).map_err(text_error)?;
                entries.extend(
                    contributors
                        .into_iter()
                        .map(|contributor| (owning, contributor)),
                );
                let Some(parent) = source.parent else { break };
                index = parent;
            }
        }
    }
    let mut pending = Vec::new();
    for (index, source) in scopes.source.nodes.iter().enumerate() {
        let Some(local) = scopes.range(scope, index) else {
            continue;
        };
        if source.atomic_owner.is_some()
            || local.start >= key.range.end
            || local.end <= key.range.start
        {
            continue;
        }
        if (key.range.end < scopes.scopes[scope.0].units.len() || key.partial.is_some())
            && (index == scopes.scopes[scope.0].owner
                || local.end > key.range.end
                || key.partial.is_some_and(|unit| local.end == unit + 1))
        {
            let contributors = packet
                .margins
                .boundary(source.node, false)
                .map_err(text_error)?;
            pending.extend(
                contributors
                    .into_iter()
                    .map(|contributor| (contributor, false, None)),
            );
        }
        if !forced
            && (key.range.start > 0 || key.resumed.is_some())
            && (index == scopes.scopes[scope.0].owner
                || local.start < key.range.start
                || key.resumed == Some(local.start))
        {
            let contributors = packet
                .margins
                .boundary(source.node, true)
                .map_err(text_error)?;
            pending.extend(
                contributors
                    .into_iter()
                    .map(|contributor| (contributor, true, None)),
            );
        }
    }
    let mut visited = Vec::new();
    let mut result = Vec::new();
    while let Some((contributor, start, owning_production)) = pending.pop() {
        if !start && entries.contains(&(owning_production, contributor)) {
            continue;
        }
        let owner_packet = owning_production
            .map_or(Some(packet), |id: ProductionId| {
                arena.get(id.0).map(|production| &production.packet)
            })
            .ok_or_else(invalid)?;
        match contributor {
            Contributor::Edge(MarginOwner { node, side }) => {
                let index = scopes
                    .source
                    .nodes
                    .iter()
                    .position(|source| source.node == node)
                    .ok_or_else(invalid)?;
                let actual = owner_packet
                    .entries
                    .iter()
                    .find(|entry| entry.node() == node)
                    .ok_or_else(invalid)?;
                if actual.output().margin.at_physical_side(side) == S::ZERO {
                    continue;
                }
                // Each edge is retained where its actual nonzero contribution
                // is truncated. Ancestors do not copy already trimmed edges.
                if !result.contains(&(index, side)) {
                    result.push((index, side));
                }
            }
            Contributor::Projection(id, owner) => {
                if visited.contains(&(id, owner, start)) {
                    continue;
                }
                visited.push((id, owner, start));
                if !owner_packet
                    .projections
                    .iter()
                    .any(|(node, child)| *node == owner.node && *child == id)
                {
                    return Err(invalid());
                }
                let production = arena.get(id.0).ok_or_else(invalid)?;
                if !views
                    .get(production.view)
                    .is_some_and(|view| view.frame == frame)
                    || !production
                        .projection
                        .is_some_and(|(_, entry)| entry.node == owner.node)
                {
                    return Err(invalid());
                }
                let contributors = production
                    .packet
                    .margins
                    .reported(owner.node, owner.side)
                    .map_err(text_error)?;
                pending.extend(
                    contributors
                        .into_iter()
                        .map(|contributor| (contributor, start, Some(id))),
                );
            }
        }
    }
    result.sort_unstable_by_key(|(index, side)| {
        (
            *index,
            match side {
                PhysicalSide::Top => 0,
                PhysicalSide::Right => 1,
                PhysicalSide::Bottom => 2,
                PhysicalSide::Left => 3,
            },
        )
    });
    Ok(result)
}

pub(super) fn owners<Node: Copy>(
    scopes: &Scopes<Node>,
    trimmed: &[(usize, PhysicalSide)],
) -> Vec<MarginOwner<Node>> {
    trimmed
        .iter()
        .map(|&(index, side)| MarginOwner {
            node: scopes.source.nodes[index].node,
            side,
        })
        .collect()
}
