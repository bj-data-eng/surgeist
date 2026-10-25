//! Computed mandatory causes admitted against the one original capture.
use super::*;
use crate::protocol::paragraph::ComputedBoundaryData;
use std::collections::BTreeMap;

/// The actual exterior side of a supplied box occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceBoxSide {
    Before,
    After,
}

/// A computed line force in this operation's actual input inventories.
/// These ordinals are claims checked by admission, never issued capabilities.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ComputedLineBoundaryInput {
    Scope { scope: usize, side: SourceBoxSide },
    Endpoint { control: usize },
    Atomic { atomic: usize, side: SourceBoxSide },
}

/// A computed request that cannot denote the required original hard boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComputedLineBoundaryError {
    TargetBounds,
    TargetRole,
    MissingParagraph,
    Domain,
    InsideScalar,
    InsideDelimiter,
}

#[derive(Clone, Debug)]
pub(crate) struct CapturedComputedBoundaryData {
    pub(crate) target: ComputedLineBoundaryInput,
    pub(crate) source: usize,
    pub(crate) capture_cut: usize,
    pub(crate) paragraph: usize,
    pub(crate) domain: usize,
}

fn refusal(request: usize, reason: ComputedLineBoundaryError) -> crate::Error {
    invalid(SourceProcessingError::ComputedLineBoundary { request, reason })
}

pub(super) fn admit(
    input: &ContextualSourceInput,
    topology: &mut ScopeCapture,
    captures: &mut [Capture],
    context: &mut ProcessingContext,
) -> Result<()> {
    if input.computed_line_boundaries.is_empty() {
        return Ok(());
    }
    // Original flow incidence includes real empty boxes/occurrences. Connections
    // and opaque child-domain scheduling cannot manufacture a second boundary.
    let mut orders: Vec<Vec<usize>> = (0..context.domains.len()).map(|_| Vec::new()).collect();
    for (index, event) in topology.events.iter().enumerate() {
        let domain = match event.event {
            SourceProcessingEvent::EnterScope(i) | SourceProcessingEvent::ExitScope(i) => {
                Some(topology.scopes[i].domain)
            }
            SourceProcessingEvent::Connection(_) => None,
            _ => event.paragraph.map(|p| topology.paragraphs[p].domain),
        };
        if let Some(domain) = domain {
            super::super::reserve(&mut orders[domain], 1)?;
            orders[domain].push(index);
        }
    }
    let mut endpoint_scopes = Vec::new();
    super::super::reserve(&mut endpoint_scopes, topology.controls.len())?;
    endpoint_scopes.resize(topology.controls.len(), None);
    for (i, scope) in topology.scopes.iter().enumerate() {
        if let Some(pair) = scope.boundaries {
            endpoint_scopes[pair.start] = Some((i, SourceBoxSide::Before));
            endpoint_scopes[pair.end] = Some((i, SourceBoxSide::After));
        }
    }
    let mut domain_paragraphs: Vec<Vec<(usize, usize, usize)>> =
        (0..context.domains.len()).map(|_| Vec::new()).collect();
    for (index, paragraph) in topology.paragraphs.iter().enumerate() {
        super::super::reserve(&mut domain_paragraphs[paragraph.domain], 1)?;
        domain_paragraphs[paragraph.domain].push((paragraph.enter, paragraph.exit, index));
    }
    for paragraphs in &mut domain_paragraphs {
        paragraphs.sort_unstable_by_key(|p| p.0);
    }
    let mut groups: BTreeMap<(usize, usize), (usize, Vec<usize>)> = BTreeMap::new();
    let mut identities = BTreeMap::new();
    for (request, &target) in input.computed_line_boundaries.iter().enumerate() {
        if let Some(&identity) = identities.get(&target) {
            super::super::reserve(&mut topology.computed_input_rows, 1)?;
            topology.computed_input_rows.push(identity);
            continue;
        }
        let err = |reason| refusal(request, reason);
        let (cut, source, domain, paragraph) = match target {
            ComputedLineBoundaryInput::Scope { scope, side } => {
                let fact = topology
                    .scopes
                    .get(scope)
                    .ok_or_else(|| err(ComputedLineBoundaryError::TargetBounds))?;
                if fact.definition.is_some() {
                    return Err(err(ComputedLineBoundaryError::TargetRole));
                }
                let event = match side {
                    SourceBoxSide::Before => fact.enter,
                    SourceBoxSide::After => fact.exit,
                };
                let paragraph = topology.events[event]
                    .paragraph
                    .or_else(|| {
                        let paragraphs = &domain_paragraphs[fact.domain];
                        let candidate = match side {
                            SourceBoxSide::Before => {
                                paragraphs.get(paragraphs.partition_point(|p| p.0 <= fact.enter))
                            }
                            SourceBoxSide::After => paragraphs
                                .partition_point(|p| p.1 < fact.exit)
                                .checked_sub(1)
                                .and_then(|i| paragraphs.get(i)),
                        }?;
                        (fact.enter < candidate.0 && candidate.1 < fact.exit).then_some(candidate.2)
                    })
                    .ok_or_else(|| err(ComputedLineBoundaryError::MissingParagraph))?;
                let p = topology.paragraphs[paragraph];
                if topology.events[event].paragraph.is_none()
                    && topology.events[event].at
                        != topology.events[match side {
                            SourceBoxSide::Before => p.enter,
                            SourceBoxSide::After => p.exit,
                        }]
                        .at
                {
                    return Err(err(ComputedLineBoundaryError::MissingParagraph));
                }
                (
                    event + usize::from(side == SourceBoxSide::After),
                    topology.events[event].at,
                    fact.domain,
                    paragraph,
                )
            }
            ComputedLineBoundaryInput::Endpoint { control } => {
                let fact = topology
                    .controls
                    .get(control)
                    .ok_or_else(|| err(ComputedLineBoundaryError::TargetBounds))?;
                let after = match fact.fact.kind {
                    ParagraphControlKind::InlineBoundaryStart => false,
                    ParagraphControlKind::InlineBoundaryEnd => true,
                    _ => return Err(err(ComputedLineBoundaryError::TargetRole)),
                };
                let cut = endpoint_scopes[control].map_or(
                    fact.order + usize::from(after),
                    |(i, side)| {
                        let scope = &topology.scopes[i];
                        match side {
                            SourceBoxSide::Before => scope.enter,
                            SourceBoxSide::After => scope.exit + 1,
                        }
                    },
                );
                (
                    cut,
                    fact.fact.at,
                    topology.paragraphs[fact.paragraph].domain,
                    fact.paragraph,
                )
            }
            ComputedLineBoundaryInput::Atomic { atomic, side } => {
                let fact = input
                    .source
                    .boxes()
                    .get(atomic)
                    .ok_or_else(|| err(ComputedLineBoundaryError::TargetBounds))?;
                if fact.kind() != InlineBoxKind::InFlow {
                    return Err(err(ComputedLineBoundaryError::TargetRole));
                }
                let order = topology.atomic_orders[atomic];
                let paragraph = topology.events[order]
                    .paragraph
                    .ok_or_else(|| err(ComputedLineBoundaryError::MissingParagraph))?;
                (
                    order + usize::from(side == SourceBoxSide::After),
                    fact.index(),
                    topology.paragraphs[paragraph].domain,
                    paragraph,
                )
            }
        };
        if captures[paragraph].domain != domain {
            return Err(err(ComputedLineBoundaryError::Domain));
        }
        if !input.source.text().is_scalar_boundary(source) {
            return Err(err(ComputedLineBoundaryError::InsideScalar));
        }
        if super::super::delimiter_crosses(&input.segment_breaks, source) {
            return Err(err(ComputedLineBoundaryError::InsideDelimiter));
        }
        super::super::reserve(&mut topology.computed, 1)?;
        let identity = topology.computed.len();
        identities.insert(target, identity);
        super::super::reserve(&mut topology.computed_input_rows, 1)?;
        topology.computed_input_rows.push(identity);
        topology.computed.push(CapturedComputedBoundaryData {
            target,
            source,
            capture_cut: cut,
            paragraph,
            domain,
        });
        let rank = orders[domain].partition_point(|&event| event < cut);
        let group = groups
            .entry((paragraph, rank))
            .or_insert_with(|| (cut, Vec::new()));
        group.0 = group.0.min(cut);
        super::super::reserve(&mut group.1, 1)?;
        group.1.push(identity);
    }
    for ((paragraph, _), (cut, causes)) in groups {
        let source = topology
            .events
            .get(cut)
            .map_or(input.source.text().len(), |event| event.at);
        let capture = &mut captures[paragraph];
        super::super::reserve(&mut capture.computed_boundaries, 1)?;
        capture.computed_boundaries.push(ComputedBoundaryData {
            at: source,
            capture_cut: cut,
            causes,
        });
        super::super::reserve(&mut context.barriers, 1)?;
        context.barriers.push(ProcessingBarrier {
            at: source,
            domain: capture.domain,
            order: cut,
            plaintext_reset: true,
            atomic: None,
            scope: None,
        });
    }
    // Capture and group order are genuine event order. Merge once, placing a
    // computed cut before an original anchor at that event and after anchors
    // whose event has already been consumed. Numeric offsets play no role.
    for capture in captures
        .iter_mut()
        .filter(|capture| !capture.computed_boundaries.is_empty())
    {
        let original = std::mem::take(&mut capture.anchors);
        let count = original
            .len()
            .checked_add(capture.computed_boundaries.len())
            .ok_or_else(super::super::overflow)?;
        let mut merged = Vec::new();
        super::super::reserve(&mut merged, count)?;
        let mut computed = 0;
        for anchor in original {
            let order = match anchor.item {
                SemanticAnchorItem::EmptyOrigin(i) => capture.origin_orders[i],
                SemanticAnchorItem::Control(i) => topology.controls[capture.controls[i]].order,
                SemanticAnchorItem::Atomic(i) => topology.atomic_orders[i],
                SemanticAnchorItem::ComputedLineBoundary(i) => {
                    capture.computed_boundaries[i].capture_cut
                }
            };
            while computed < capture.computed_boundaries.len()
                && capture.computed_boundaries[computed].capture_cut <= order
            {
                merged.push(SemanticAnchor {
                    item: SemanticAnchorItem::ComputedLineBoundary(computed),
                    affinity: Affinity::After,
                });
                computed += 1;
            }
            merged.push(anchor);
        }
        while computed < capture.computed_boundaries.len() {
            merged.push(SemanticAnchor {
                item: SemanticAnchorItem::ComputedLineBoundary(computed),
                affinity: Affinity::After,
            });
            computed += 1;
        }
        capture.anchors = merged;
        // Translate each original connection position by precisely the groups
        // that precede its actual event. Connections and groups are ordered.
        let mut inserted = 0;
        for (position, &order) in capture
            .connection_anchor_cuts
            .iter_mut()
            .zip(&capture.connection_orders)
        {
            while inserted < capture.computed_boundaries.len()
                && capture.computed_boundaries[inserted].capture_cut <= order
            {
                inserted += 1;
            }
            *position += inserted;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
