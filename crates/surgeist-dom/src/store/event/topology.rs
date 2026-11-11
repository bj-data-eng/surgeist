//! One event-parent algorithm for canonical dispatch and staged SVG closure.
use super::{Owner, Processing};
use crate::event::{EventParentFacts, LOAD_EVENT, Notification};
use crate::store::source::formation::Formation;
use crate::store::{Draft, Key, check_cancel, transaction};
use crate::{
    Dom, DomError, DomEventParent, DomEventParentQuery, DomEventTarget, DomHostedError,
    DomHostedGlobalId, DomNodeKind, DomSnapshot, NodeId,
};
use std::sync::atomic::AtomicBool;
use surgeist_html::security::nonce::HtmlBrowsingContextPresence;

#[derive(Clone, Copy)]
struct Document {
    hosted: bool,
    context: bool,
}

#[derive(Clone, Copy)]
struct Facts<K> {
    parent: Option<K>,
    assigned: Option<K>,
    shadow_host: Option<K>,
    shadow: bool,
    document: Option<Document>,
}

#[derive(Clone, Copy)]
enum Parent<K> {
    None,
    Node {
        parent: K,
        assigned: bool,
        in_target_tree: bool,
    },
    Global(DomHostedGlobalId),
}

fn ordinary_root<K: Copy, E>(
    mut key: K,
    read: &mut impl FnMut(K) -> Result<Facts<K>, E>,
) -> Result<K, E> {
    loop {
        match read(key)?.parent {
            Some(parent) => key = parent,
            None => return Ok(key),
        }
    }
}

fn parent<K: Copy + Eq, E: From<DomHostedError>>(
    current: K,
    facts: Facts<K>,
    first_root: K,
    adjusted_target: K,
    event: EventParentFacts<'_>,
    read: &mut impl FnMut(K) -> Result<Facts<K>, E>,
    global: &mut impl FnMut(K) -> Result<Option<DomHostedGlobalId>, E>,
) -> Result<Parent<K>, E> {
    let assigned = facts.assigned.is_some();
    let parent = if let Some(slot) = facts.assigned {
        Some(slot)
    } else if let Some(document) = facts.document {
        if event.event_type != LOAD_EVENT && document.context {
            return Ok(Parent::Global(
                global(current)?.ok_or(DomHostedError::UnsupportedProfile)?,
            ));
        }
        return Ok(Parent::None);
    } else if facts.shadow {
        if !event.composed && current == first_root {
            None
        } else {
            facts.shadow_host
        }
    } else {
        facts.parent
    };
    let Some(parent) = parent else {
        return Ok(Parent::None);
    };
    let target_root = ordinary_root(adjusted_target, read)?;
    let mut ancestor = Some(parent);
    let mut in_target_tree = false;
    while let Some(key) = ancestor {
        let facts = read(key)?;
        if key == target_root {
            in_target_tree = true;
            break;
        }
        ancestor = facts.parent.or(facts.shadow_host);
    }
    Ok(Parent::Node {
        parent,
        assigned,
        in_target_tree,
    })
}

impl Dom {
    /// Observe one event-parent step on this owner's current canonical basis.
    /// The supplied work hook meters each node observation and traversal step;
    /// current hosted authority additionally uses its original Source seam.
    pub fn dispatch_event_parent<E>(
        &self,
        snapshot: &DomSnapshot,
        query: DomEventParentQuery<'_>,
        cancel: &AtomicBool,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<DomEventParent, E>
    where
        E: From<DomError> + From<DomHostedError>,
    {
        check_cancel(cancel)?;
        if snapshot.owner() != self.owner()
            || [query.current, query.first_root, query.adjusted_target]
                .iter()
                .any(|id| id.owner() != self.owner())
        {
            return Err(DomError::ForeignOwner.into());
        }
        if snapshot.revision() != self.revision() {
            return Err(DomHostedError::StaleBasis.into());
        }
        let work = self.dispatch_hosted_read_work(cancel)?;
        let tick = std::cell::RefCell::new(|| -> Result<(), E> {
            check_cancel(cancel)?;
            step()?;
            if let Some(work) = &work {
                work.step(cancel)?;
            }
            Ok(())
        });
        let mut read = |key: NodeId| -> Result<Facts<NodeId>, E> {
            (tick.borrow_mut())()?;
            let node = snapshot.state.node(snapshot.owner, key)?;
            let document = match node.kind() {
                DomNodeKind::Document(metadata) => {
                    let environment = metadata.document_environment();
                    Some(Document {
                        hosted: environment.has_ever_had_navigable(),
                        context: environment.browsing_context()
                            != HtmlBrowsingContextPresence::Absent,
                    })
                }
                _ => None,
            };
            let shadow = matches!(node.kind(), DomNodeKind::ShadowRoot(_));
            Ok(Facts {
                parent: node.parent(),
                assigned: node.assigned_slot(),
                shadow_host: if shadow { node.host() } else { None },
                shadow,
                document,
            })
        };
        // Validate every operand, including a first root that the next step
        // does not otherwise need to dereference.
        read(query.first_root)?;
        read(query.adjusted_target)?;
        let next = parent(
            query.current,
            read(query.current)?,
            query.first_root,
            query.adjusted_target,
            query.event,
            &mut read,
            &mut |document| {
                self.dispatch_relevant_global_with(document, cancel, &mut || (tick.borrow_mut())())
            },
        )?;
        check_cancel(cancel)?;
        Ok(match next {
            Parent::None => DomEventParent::None,
            Parent::Global(global) => DomEventParent::Global(global),
            Parent::Node {
                parent,
                assigned,
                in_target_tree,
            } => DomEventParent::Node {
                parent,
                via_assigned_slot: assigned,
                in_adjusted_target_tree: in_target_tree,
            },
        })
    }
}

// Brent's route-cycle check retains only exact keys, with no tree-sized scratch
// allocation or alternate canonical IDs for unpublished nodes.
struct RouteCycle<K> {
    checkpoint: K,
    power: usize,
    length: usize,
}
impl<K: Copy + Eq> RouteCycle<K> {
    fn new(first: K) -> Self {
        Self {
            checkpoint: first,
            power: 1,
            length: 0,
        }
    }
    fn advance(&mut self, next: K) -> Result<(), DomError> {
        if self.checkpoint == next {
            return Err(DomError::SvgScriptProcessingUnavailable);
        }
        self.length = self
            .length
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if self.length == self.power {
            self.checkpoint = next;
            self.power = self
                .power
                .checked_mul(2)
                .ok_or(DomError::RepresentationExhausted)?;
            self.length = 0;
        }
        Ok(())
    }
}

impl Draft {
    pub(in crate::store) fn require_svg_notification_closed(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let Some(owner) = self
            .event_recipients
            .as_ref()
            .filter(|owner| owner.has_live())
        else {
            return Ok(());
        };
        let formation = Formation::new(self, cancel)?;
        let processing = Processing::new(
            self.dom_limits.event_recipients(),
            formation.as_ref(),
            cancel,
        );
        processing.scratch(std::mem::size_of::<RouteCycle<Key>>())?;
        // Load and Error have different Document parents, but the same target
        // flags: capture everywhere and noncapture only at retargeted targets.
        for event_type in [Notification::Load, Notification::Error] {
            self.require_svg_route_closed(key, event_type, owner, &processing)?;
        }
        check_cancel(cancel)
    }

    fn event_facts(
        &self,
        key: Key,
        processing: &Processing<'_>,
    ) -> Result<Facts<Key>, DomHostedError> {
        processing.step()?;
        let node = self.node(key)?;
        let document = match node.kind() {
            transaction::KindRead::Document(metadata) => {
                let environment = metadata.document_environment();
                Some(Document {
                    hosted: environment.has_ever_had_navigable(),
                    context: environment.browsing_context() != HtmlBrowsingContextPresence::Absent,
                })
            }
            _ => None,
        };
        let shadow = matches!(node.kind(), transaction::KindRead::ShadowRoot(_));
        Ok(Facts {
            parent: node.parent().map(|key| self.canonical_key(key)),
            assigned: node.assigned_slot().map(|key| self.canonical_key(key)),
            shadow_host: if shadow {
                node.host().map(|key| self.canonical_key(key))
            } else {
                None
            },
            shadow,
            document,
        })
    }

    fn event_global(
        &self,
        document: Key,
        processing: &Processing<'_>,
    ) -> Result<Option<DomHostedGlobalId>, DomHostedError> {
        processing.step()?;
        let global = self.node(document)?.relevant_global();
        let Some(global) = global else {
            return Ok(None);
        };
        processing.step()?;
        if global.source.owner != self.store {
            return Err(DomHostedError::ForeignSource);
        }
        let associated = self
            .hosted_globals
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?
            .document_with(global, &mut || {
                processing.step().map_err(DomHostedError::from)
            })?;
        processing.step()?;
        let environment = self
            .document_metadata(Key::Existing(associated.node()))?
            .document_environment();
        let association = environment
            .hosted
            .as_ref()
            .filter(|value| value.global == global)
            .ok_or(DomHostedError::StaleIdentity)?;
        association.validate_live()?;
        Ok(Some(global))
    }

    fn event_recipient_active(
        &self,
        target: DomEventTarget,
        processing: &Processing<'_>,
    ) -> Result<bool, DomError> {
        processing.step()?;
        let document = match target {
            DomEventTarget::Node(node) => self.node(Key::Existing(node))?.document(),
            DomEventTarget::Global(global) => {
                let document = self
                    .hosted_globals
                    .as_ref()
                    .ok_or(DomError::SvgScriptProcessingUnavailable)?
                    .document_with(global, &mut || {
                        processing.step().map_err(DomHostedError::from)
                    })
                    .map_err(svg_error)?;
                Key::Existing(document.node())
            }
        };
        processing.step()?;
        let environment = self.document_metadata(document)?.document_environment();
        if !environment.has_ever_had_navigable() {
            return Ok(true);
        }
        // An actual canonical inactive/retired association proves inapplicability.
        // Unknown current authority instead refuses closed processing.
        processing.step()?;
        if !self.document_fully_active_with(document, || processing.step())? {
            return Ok(false);
        }
        let global = match target {
            DomEventTarget::Node(_) => self
                .event_global(document, processing)
                .map_err(svg_error)?
                .ok_or(DomError::SvgScriptProcessingUnavailable)?,
            DomEventTarget::Global(global) => global,
        };
        let associated = self
            .hosted_globals
            .as_ref()
            .ok_or(DomError::SvgScriptProcessingUnavailable)?
            .document_with(global, &mut || {
                processing.step().map_err(DomHostedError::from)
            })
            .map_err(svg_error)?;
        if self.published_id(document) != Some(associated.node()) {
            return Err(DomError::SvgScriptProcessingUnavailable);
        }
        if self
            .event_global(Key::Existing(associated.node()), processing)
            .map_err(svg_error)?
            != Some(global)
        {
            return Err(DomError::SvgScriptProcessingUnavailable);
        }
        Ok(true)
    }

    fn require_svg_route_closed(
        &self,
        original: Key,
        event_type: Notification,
        owner: &Owner,
        processing: &Processing<'_>,
    ) -> Result<(), DomError> {
        let original = self.canonical_key(original);
        let mut read = |key| self.event_facts(key, processing);
        let first_root = ordinary_root(original, &mut read).map_err(svg_error)?;
        let mut current = original;
        let mut adjusted_target = original;
        let mut at_target = true;
        let mut cycle = RouteCycle::new(original);
        loop {
            processing.step()?;
            // Only a real published identity can have an indexed registration.
            // New staged keys still participate in every topology decision.
            if let Some(node) = self.published_id(current) {
                let target = DomEventTarget::Node(node);
                if owner.has_recipient(target, event_type, at_target, processing)?
                    && self.event_recipient_active(target, processing)?
                {
                    return Err(DomError::SvgScriptProcessingUnavailable);
                }
            }
            let facts = read(current).map_err(svg_error)?;
            // An SVG no-recipient proof also needs live issuer authority on a
            // hosted Document where the standard event path stops.
            let global =
                svg_document_global(facts.document, || self.event_global(current, processing))
                    .map_err(svg_error)?;
            let next = parent(
                current,
                facts,
                first_root,
                adjusted_target,
                EventParentFacts {
                    event_type: event_type.event_type(),
                    composed: false,
                },
                &mut read,
                &mut |_| Ok(global),
            )
            .map_err(svg_error)?;
            match next {
                Parent::None => return Ok(()),
                Parent::Global(global) => {
                    let target = DomEventTarget::Global(global);
                    if owner.has_recipient(target, event_type, false, processing)?
                        && self.event_recipient_active(target, processing)?
                    {
                        return Err(DomError::SvgScriptProcessingUnavailable);
                    }
                    return Ok(());
                }
                Parent::Node {
                    parent: node,
                    in_target_tree,
                    ..
                } => {
                    processing.step()?;
                    cycle.advance(node)?;
                    at_target = !in_target_tree;
                    if at_target {
                        adjusted_target = node;
                    }
                    current = node;
                }
            }
        }
    }
}

fn svg_document_global(
    document: Option<Document>,
    resolve: impl FnOnce() -> Result<Option<DomHostedGlobalId>, DomHostedError>,
) -> Result<Option<DomHostedGlobalId>, DomHostedError> {
    if document.is_some_and(|document| document.hosted) {
        return resolve()?
            .ok_or(DomHostedError::UnsupportedProfile)
            .map(Some);
    }
    Ok(None)
}

fn svg_error(error: DomHostedError) -> DomError {
    match error {
        DomHostedError::Dom(error) => error,
        _ => DomError::SvgScriptProcessingUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noncomposed(notification: Notification) -> EventParentFacts<'static> {
        EventParentFacts {
            event_type: notification.event_type(),
            composed: false,
        }
    }

    fn node(parent: Option<usize>) -> Facts<usize> {
        Facts {
            parent,
            assigned: None,
            shadow_host: None,
            shadow: false,
            document: None,
        }
    }

    #[test]
    fn assigned_slot_parent_uses_shadow_including_target_tree_membership() {
        // 0 Document <- 1 host <- 2 light target; 3 shadow root <- 4 slot.
        let mut nodes = [
            node(None),
            node(Some(0)),
            node(Some(1)),
            node(None),
            node(Some(3)),
        ];
        nodes[2].assigned = Some(4);
        nodes[3].shadow = true;
        nodes[3].shadow_host = Some(1);
        let next = parent(
            2,
            nodes[2],
            0,
            2,
            noncomposed(Notification::Load),
            &mut |key| Ok::<_, DomHostedError>(nodes[key]),
            &mut |_| Ok(None),
        )
        .unwrap();
        assert!(matches!(
            next,
            Parent::Node {
                parent: 4,
                assigned: true,
                in_target_tree: true
            }
        ));
    }

    #[test]
    fn initial_shadow_root_stops_but_a_different_root_uses_its_actual_host() {
        let mut nodes = [node(None), node(Some(0)), node(None)];
        nodes[2].shadow = true;
        nodes[2].shadow_host = Some(1);
        let stopped = parent(
            2,
            nodes[2],
            2,
            2,
            noncomposed(Notification::Error),
            &mut |key| Ok::<_, DomHostedError>(nodes[key]),
            &mut |_| Ok(None),
        )
        .unwrap();
        assert!(matches!(stopped, Parent::None));
        let crossed = parent(
            2,
            nodes[2],
            0,
            2,
            noncomposed(Notification::Error),
            &mut |key| Ok::<_, DomHostedError>(nodes[key]),
            &mut |_| Ok(None),
        )
        .unwrap();
        // Root 2 is not an inclusive shadow ancestor of host 1: Runtime must
        // retarget to the host and invoke its noncapture at-target listeners.
        assert!(matches!(
            crossed,
            Parent::Node {
                parent: 1,
                in_target_tree: false,
                ..
            }
        ));
    }

    #[test]
    fn svg_document_proof_requires_current_authority_even_when_load_stops() {
        let document = Some(Document {
            hosted: true,
            context: true,
        });
        let result = svg_document_global(document, || Err(DomHostedError::SourceExpired));
        assert!(matches!(result, Err(DomHostedError::SourceExpired)));
    }
}
