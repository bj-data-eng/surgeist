//! Associations emitted by the actual selected presentation owner.

use super::{AlternativeDefinition, GeneratedSeparatorOwner, PresentationUnitKind};
use crate::Range;

/// Actual selected occurrence associations, including distinct selected empty
/// origins and source-less native controls/alternatives. No inclusive numeric
/// anchor query invents an occurrence on the adjacent line.
#[cfg(test)]
pub(crate) fn selected_origin_intersections(
    selected: &crate::protocol::selection::SelectedData,
) -> Vec<crate::protocol::paragraph::FragmentAssociation> {
    let mut fragments = Vec::new();
    visit_selected_origin_intersections(selected, |association| {
        fragments.push(association);
        Ok(())
    })
    .expect("ordinary association collection has no fallible visitor");
    fragments
}

pub(crate) fn visit_selected_origin_intersections(
    selected: &crate::protocol::selection::SelectedData,
    mut emit: impl FnMut(crate::protocol::paragraph::FragmentAssociation) -> crate::Result<()>,
) -> crate::Result<()> {
    use crate::protocol::paragraph::{FragmentAssociation, FragmentOwner};
    let paragraph = &selected.prepared.analyzed.paragraph;
    let processed = &selected.prepared.analyzed.processed;
    let mut last_owner = None;
    macro_rules! append {
        ($association:expr) => {{
            let association = $association;
            last_owner = Some(association.owner);
            emit(association)?;
        }};
    }
    for unit in &selected.presentation.units {
        match unit.kind {
            PresentationUnitKind::Cell { cell } => {
                let analyzed = &selected.prepared.analyzed;
                let output = analyzed
                    .combination
                    .as_deref()
                    .expect("selected cells retain their source operation");
                let binding = &output.bindings[selected.prepared.shaped.cells[cell].binding];
                let group = &output.plan.groups[binding.group];
                for part in &output.plan.parts[group.parts.clone()] {
                    for piece in &part.pieces {
                        let origin = paragraph
                            .origin_map
                            .view_origin(piece.origin)
                            .expect("cell pieces belong to their admitted complete output")
                            .get();
                        append!(FragmentAssociation {
                            owner: FragmentOwner::Origin(origin),
                            source: Some(crate::protocol::paragraph::OriginIntersection {
                                origin,
                                aggregate: piece.aggregate,
                                local: piece.local,
                                processed: part.processed,
                            }),
                        });
                    }
                }
                use crate::transform::relation::{ComponentOwner, RelationEvent};
                for event in &analyzed.transform.events()[binding.events.clone()] {
                    let component_owner = |owner| match owner {
                        ComponentOwner::Wbr(control) => Some(FragmentOwner::Control(control)),
                        ComponentOwner::Virtual(index) => {
                            Some(FragmentOwner::VirtualSeparator(index))
                        }
                        ComponentOwner::Original(_) => None,
                    };
                    let owner = match *event {
                        RelationEvent::Text { atom } => {
                            component_owner(analyzed.transform.component_for_atom(atom).owner)
                        }
                        RelationEvent::GeneratedSuppression { component, .. } => {
                            component_owner(analyzed.transform.components[component].owner)
                        }
                        RelationEvent::Anchor { order } => match paragraph.anchor_order[order].item
                        {
                            crate::protocol::paragraph::SemanticAnchorItem::Control(control) => {
                                Some(FragmentOwner::Control(control))
                            }
                            _ => None,
                        },
                        _ => None,
                    };
                    if let Some(owner) = owner
                        && last_owner != Some(owner)
                    {
                        append!(FragmentAssociation {
                            owner,
                            source: None
                        });
                    }
                }
            }
            PresentationUnitKind::Control { occurrence }
            | PresentationUnitKind::GeneratedSeparator {
                owner: GeneratedSeparatorOwner::Control(occurrence),
            } => append!(FragmentAssociation {
                owner: FragmentOwner::Control(occurrence),
                source: None,
            }),
            PresentationUnitKind::GeneratedSeparator {
                owner: GeneratedSeparatorOwner::Virtual(index),
            } => append!(FragmentAssociation {
                owner: FragmentOwner::VirtualSeparator(index),
                source: None,
            }),
            PresentationUnitKind::Replacement { alternative, .. } => {
                let row = &selected.prepared.tables.alternatives[alternative];
                let origin = match row.definition {
                    AlternativeDefinition::Source { origin, .. } => Some(origin),
                    AlternativeDefinition::Control { origin, .. } => origin,
                    AlternativeDefinition::Automatic { origin, .. } => Some(origin),
                };
                let source = origin.map(|origin| {
                    let row = &paragraph.origins[origin];
                    let at = unit.source.start;
                    let local = row.local.start + at - row.aggregate.start;
                    crate::protocol::paragraph::OriginIntersection {
                        origin,
                        aggregate: unit.source,
                        local: Range::new(local, local),
                        processed: unit.processed,
                    }
                });
                let association = FragmentAssociation {
                    owner: FragmentOwner::Alternative(alternative),
                    source,
                };
                if last_owner != Some(association.owner) {
                    append!(association);
                }
            }
            PresentationUnitKind::EmptyOrigin { origin } => {
                let row = &paragraph.origins[origin];
                append!(FragmentAssociation {
                    owner: FragmentOwner::Origin(origin),
                    source: Some(crate::protocol::paragraph::OriginIntersection {
                        origin,
                        aggregate: row.aggregate,
                        local: row.local,
                        processed: unit.processed,
                    }),
                });
            }
            PresentationUnitKind::Text | PresentationUnitKind::Deleted => {
                for source in paragraph
                    .origin_intersections(processed, unit.source)
                    .filter(|i| !paragraph.origins[i.origin].aggregate.is_empty())
                {
                    append!(FragmentAssociation {
                        owner: FragmentOwner::Origin(source.origin),
                        source: Some(source)
                    });
                }
            }
            PresentationUnitKind::Atomic { .. }
            | PresentationUnitKind::ComputedLineBoundary { .. } => {}
        }
    }
    Ok(())
}
