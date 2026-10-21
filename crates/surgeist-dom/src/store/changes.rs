//! Canonical change comparison preserves publication ordering and exact identities.
use super::*;

fn same_slice<T: PartialEq>(left: &[T], right: &[T]) -> bool {
    std::ptr::eq(left, right) || left == right
}

fn same_optional<T: PartialEq>(left: Option<&T>, right: Option<&T>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => std::ptr::eq(left, right) || left == right,
        _ => false,
    }
}

fn same_payload<T: PartialEq>(
    left: &Arc<payload::Payload<T>>,
    right: &Arc<payload::Payload<T>>,
) -> bool {
    Arc::ptr_eq(left, right) || left.value == right.value
}

fn same_language(left: &Node, right: &Node) -> bool {
    match (left, right) {
        (Node::Document(a), Node::Document(b)) => {
            same_payload(&a.value.metadata, &b.value.metadata)
        }
        (Node::Element(a), Node::Element(b)) => same_payload(&a.value.element, &b.value.element),
        (Node::DocumentFragment(_), Node::DocumentFragment(_)) => true,
        (Node::ShadowRoot(a), Node::ShadowRoot(b)) => same_payload(&a.value.state, &b.value.state),
        (Node::Text(a), Node::Text(b)) => same_payload(&a.value.data, &b.value.data),
        (Node::Comment(a), Node::Comment(b)) => same_payload(&a.value.data, &b.value.data),
        (Node::ProcessingInstruction(a), Node::ProcessingInstruction(b)) => {
            same_payload(&a.value.data, &b.value.data)
        }
        (Node::DocumentType(a), Node::DocumentType(b)) => {
            same_payload(&a.value.data, &b.value.data)
        }
        _ => false,
    }
}

fn same_attributes(left: &DomElement, right: &DomElement) -> bool {
    if let (Some(a), Some(b)) = (left.attribute_binding(), right.attribute_binding())
        && Arc::ptr_eq(a, b)
    {
        return true;
    }
    same_slice(left.attributes(), right.attributes())
}

fn document_fields(node: &Node) -> Option<&canonical::DocumentNode<NodeId>> {
    match node {
        Node::Document(value) => Some(&value.value),
        _ => None,
    }
}

fn same_autofocus(left: &Node, right: &Node) -> bool {
    if let (Some(a), Some(b)) = (document_fields(left), document_fields(right))
        && let (canonical::Autofocus::Known(a), canonical::Autofocus::Known(b)) =
            (&a.autofocus_document, &b.autofocus_document)
        && Arc::ptr_eq(a, b)
    {
        return true;
    }
    match (left.autofocus_document(), right.autofocus_document()) {
        (None, None)
        | (
            Some(canonical::AutofocusView::Unavailable),
            Some(canonical::AutofocusView::Unavailable),
        ) => true,
        (
            Some(canonical::AutofocusView::Known {
                candidates: a,
                processed: ap,
            }),
            Some(canonical::AutofocusView::Known {
                candidates: b,
                processed: bp,
            }),
        ) => ap == bp && same_slice(a, b),
        _ => false,
    }
}

fn same_interactive(left: &Node, right: &Node) -> bool {
    if let (Some(a), Some(b)) = (document_fields(left), document_fields(right))
        && let (canonical::Interactive::Known(a), canonical::Interactive::Known(b)) =
            (&a.interactive_document, &b.interactive_document)
        && Arc::ptr_eq(a, b)
    {
        return true;
    }
    match (left.interactive_document(), right.interactive_document()) {
        (None, None)
        | (
            Some(canonical::InteractiveView::Unavailable),
            Some(canonical::InteractiveView::Unavailable),
        ) => true,
        (
            Some(canonical::InteractiveView::Known {
                open_dialogs: a,
                top_layer: at,
                pending_top_layer_removals: ap,
            }),
            Some(canonical::InteractiveView::Known {
                open_dialogs: b,
                top_layer: bt,
                pending_top_layer_removals: bp,
            }),
        ) => same_slice(a, b) && same_slice(at, bt) && same_slice(ap, bp),
        _ => false,
    }
}

fn same_top_layer_backing(
    left: Option<&SharedVec<NodeId>>,
    right: Option<&SharedVec<NodeId>>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.same_storage(right),
        (None, None) => true,
        (Some(value), None) | (None, Some(value)) => !value.has_backing(),
    }
}

fn same_top_layer_storage(left: &Node, right: &Node) -> bool {
    let absent = canonical::Interactive::Absent;
    let left = document_fields(left).map_or(&absent, |node| &node.interactive_document);
    let right = document_fields(right).map_or(&absent, |node| &node.interactive_document);
    match (left, right) {
        (canonical::Interactive::Absent, canonical::Interactive::Absent)
        | (canonical::Interactive::Unavailable, canonical::Interactive::Unavailable)
        | (canonical::Interactive::KnownEmpty, canonical::Interactive::KnownEmpty) => true,
        (canonical::Interactive::Known(left), canonical::Interactive::Known(right)) => {
            same_top_layer_backing(Some(&left.value.top_layer), Some(&right.value.top_layer))
        }
        (canonical::Interactive::Known(value), canonical::Interactive::KnownEmpty)
        | (canonical::Interactive::KnownEmpty, canonical::Interactive::Known(value)) => {
            same_top_layer_backing(Some(&value.value.top_layer), None)
        }
        _ => false,
    }
}

fn same_base_document(left: &Node, right: &Node) -> bool {
    if let (Some(a), Some(b)) = (document_fields(left), document_fields(right))
        && let (canonical::BaseDocumentState::Known(a), canonical::BaseDocumentState::Known(b)) =
            (&a.base_document, &b.base_document)
        && Arc::ptr_eq(a, b)
    {
        return true;
    }
    match (left.base_document(), right.base_document()) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.first == b.first
                && a.policy_basis == b.policy_basis
                && same_slice(a.render_blocking, b.render_blocking)
                && match (a.facts, b.facts) {
                    (None, None) => true,
                    (Some(a), Some(b)) => a.same_owner(b),
                    _ => false,
                }
        }
        _ => false,
    }
}

fn same_render_blocking(left: &Node, right: &Node) -> bool {
    match (left.base_document(), right.base_document()) {
        (None, None) => true,
        (Some(a), Some(b)) => same_slice(a.render_blocking, b.render_blocking),
        _ => false,
    }
}

fn same_encoding_url(left: &Node, right: &Node) -> bool {
    if let (Some(a), Some(b)) = (document_fields(left), document_fields(right))
        && let (canonical::EncodingUrl::Known(a), canonical::EncodingUrl::Known(b)) =
            (&a.encoding_url_document, &b.encoding_url_document)
        && Arc::ptr_eq(a, b)
    {
        return true;
    }
    match (left.encoding_url_document(), right.encoding_url_document()) {
        (None, None) => true,
        (Some(a), Some(b)) => match (a.facts, b.facts) {
            (None, None) => true,
            (Some(a), Some(b)) => a.same_owner(b),
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn sparse_difference(
    owner: DomOwnerId,
    old: &State,
    new: &State,
    budget: Option<&source::formation::Formation>,
    cancelled: &AtomicBool,
    node_indices: &bank::Bank<bool>,
    range_indices: impl IntoIterator<Item = usize>,
) -> Result<Vec<DomChange>, DomError> {
    let mut changes = Vec::new();
    #[cfg(test)]
    let mut written = transaction::CopyObservation::default();
    for index in range_indices {
        check_cancel(cancelled)?;
        if let Some(budget) = budget {
            budget.retain_bytes(0, cancelled)?;
        }
        let before = old
            .ranges
            .get(index)
            .and_then(|slot| slot.record.as_ref().map(|record| (slot.generation, record)));
        let after = new
            .ranges
            .get(index)
            .and_then(|slot| slot.record.as_ref().map(|record| (slot.generation, record)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, record)) = before {
            let id = crate::DomRangeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                if record != after.expect("same Range").1 {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::RangeBoundary(id));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
            } else {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::RangeRetired(id));
                #[cfg(test)]
                written.pushed::<DomChange>();
            }
        }
        if !same && let Some((generation, _)) = after {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::RangeCreated(crate::DomRangeId {
                owner,
                slot: index,
                generation,
            }));
            #[cfg(test)]
            written.pushed::<DomChange>();
        }
    }
    // Fullscreen facts depend on top-layer membership and flags of actual
    // members. Scalar Document writes do not enter that dependency domain.
    let scratch = budget.map(source::formation::Formation::fork);
    let mut fullscreen_documents = Vec::new();
    let mut selected_nodes = Vec::new();
    for (index, _) in node_indices.iter() {
        source::formation::reserve(&mut selected_nodes, 1, scratch.as_ref(), cancelled)?;
        selected_nodes.push(index);
        #[cfg(test)]
        written.pushed::<usize>();
        let before = old.slots.get(index).and_then(|s| s.node.as_ref());
        let after = new.slots.get(index).and_then(|s| s.node.as_ref());
        let flags = |node: Option<&Node>| {
            node.and_then(|n| match n.kind() {
                DomNodeKind::Element(e) => {
                    Some((n.document(), e.fullscreen_flag(), e.keyboard_lock_flag()))
                }
                _ => None,
            })
        };
        let before = flags(before);
        let after = flags(after);
        if before != after && (before.is_some_and(|v| v.1) || after.is_some_and(|v| v.1)) {
            for doc in [before.map(|v| v.0), after.map(|v| v.0)]
                .into_iter()
                .flatten()
            {
                source::formation::reserve(
                    &mut fullscreen_documents,
                    1,
                    scratch.as_ref(),
                    cancelled,
                )?;
                fullscreen_documents.push(doc);
                #[cfg(test)]
                written.pushed::<DocumentId>();
                source::formation::reserve(&mut selected_nodes, 1, scratch.as_ref(), cancelled)?;
                selected_nodes.push(doc.node().slot);
                #[cfg(test)]
                written.pushed::<usize>();
            }
        }
    }
    fullscreen_documents.sort_unstable_by_key(|id| (id.node().slot, id.node().generation));
    fullscreen_documents.dedup();
    selected_nodes.sort_unstable();
    selected_nodes.dedup();
    for index in selected_nodes {
        let before = old
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let after = new
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, node)) = before {
            let id = NodeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                let current = after.expect("same live identity").1;
                if node.active_range() != current.active_range() {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentActiveRange(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if let (DomNodeKind::Document(before), DomNodeKind::Document(after)) =
                    (node.kind(), current.kind())
                    && before.design_mode_enabled() != after.design_mode_enabled()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentDesignMode(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (node.kind(), current.kind())
                    && before.captured_in_view_transition() != after.captured_in_view_transition()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::ViewTransitionCaptured(
                        DomViewTransitionSubject::Element(id),
                    ));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                let intrinsic_changed = match (node.kind(), current.kind()) {
                    (DomNodeKind::Element(before), DomNodeKind::Element(after)) => {
                        !same_attributes(before, after)
                    }
                    (DomNodeKind::Document(before), DomNodeKind::Document(after)) => {
                        !std::ptr::eq(before, after) && !before.same_nondesign_mode_state(after)
                    }
                    _ => !same_language(node, current),
                };
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (node.kind(), current.kind())
                    && ((!std::ptr::eq(before, after) && !before.same_nonattribute_state(after))
                        || node.reflected_targets() != current.reflected_targets()
                        || node.control_owner() != current.control_owner()
                        || !same_optional(node.form_past_names(), current.form_past_names())
                        || !same_optional(node.base(), current.base())
                        || !same_optional(node.link(), current.link())
                        || !same_optional(node.image(), current.image())
                        || !same_optional(node.hyperlink(), current.hyperlink())
                        || !same_optional(node.script(), current.script())
                        || node.style_parser_document() != current.style_parser_document()
                        || node.option_select() != current.option_select()
                        || node.template_patch() != current.template_patch())
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::ElementState(id));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if !same_optional(node.base(), current.base())
                    && let Some(frozen) = current.base().and_then(|b| b.frozen.as_ref())
                {
                    let change = DomChange::DocumentBase(DocumentId(frozen.document));
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(change);
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if !same_autofocus(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentAutofocusState(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if !same_interactive(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentInteractiveState(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if !same_slice(node.fullscreen_events(), current.fullscreen_events()) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentFullscreenEvents(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                let same_top_layer = same_top_layer_storage(node, current);
                let fullscreen_dependency = fullscreen_documents
                    .binary_search_by_key(&(id.slot, id.generation), |doc| {
                        (doc.node().slot, doc.node().generation)
                    })
                    .is_ok()
                    || !same_top_layer;
                if matches!(node.kind(), DomNodeKind::Document(_))
                    && fullscreen_dependency
                    && fullscreen::captured_facts(old, node, owner, budget, cancelled)?
                        != fullscreen::captured_facts(new, current, owner, budget, cancelled)?
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentFullscreenState(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if !same_base_document(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentBase(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if !same_render_blocking(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentRenderBlocking(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if let (DomNodeKind::Document(before), DomNodeKind::Document(after)) =
                    (node.kind(), current.kind())
                    && (before.meta_state() != after.meta_state()
                        || before.meta_theme_color() != after.meta_theme_color()
                        || before.meta_color_scheme != after.meta_color_scheme
                        || before.policy_container() != after.policy_container())
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentMeta(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if !same_encoding_url(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentEncodingUrlFacts(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                let kind_change = if intrinsic_changed {
                    Some(
                        if matches!(
                            (node.kind(), current.kind()),
                            (DomNodeKind::Element(_), DomNodeKind::Element(_))
                        ) {
                            DomChange::Attributes(id)
                        } else {
                            DomChange::Data(id)
                        },
                    )
                } else {
                    None
                };
                for change in [
                    kind_change,
                    (!same_slice(node.manual_nodes(), current.manual_nodes()))
                        .then_some(DomChange::SlotManual(id)),
                    (!same_slice(node.assigned_nodes(), current.assigned_nodes()))
                        .then_some(DomChange::SlotAssignedNodes(id)),
                    (node.manual_slot() != current.manual_slot())
                        .then_some(DomChange::ManualAssignment(id)),
                    (node.assigned_slot() != current.assigned_slot())
                        .then_some(DomChange::StoredAssignedSlot(id)),
                    (node.shadow_root() != current.shadow_root())
                        .then_some(DomChange::ShadowRoot(id)),
                    (node.document() != current.document()).then_some(DomChange::NodeDocument(id)),
                    (node.parent() != current.parent()).then_some(DomChange::Parent(id)),
                    (!same_slice(node.children(), current.children()))
                        .then_some(DomChange::Children(id)),
                ]
                .into_iter()
                .flatten()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(change);
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
            } else {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::Destroyed(id));
                #[cfg(test)]
                written.pushed::<DomChange>();
            }
        }
        if let Some((generation, _)) = after
            && !same
        {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::Created(NodeId {
                owner,
                slot: index,
                generation,
            }));
            #[cfg(test)]
            written.pushed::<DomChange>();
        }
    }
    let has_template = |state: &State| state.counts.templates != 0;
    let cache_changed = node_indices.iter().any(|(index, _)| {
        old.slots
            .get(index)
            .and_then(|slot| slot.node.as_ref())
            .zip(new.slots.get(index).and_then(|slot| slot.node.as_ref()))
            .is_some_and(|(a, b)| {
                a.template_document() != b.template_document()
                    || a.template_contents() != b.template_contents()
                    || a.template_patch() != b.template_patch()
            })
    });
    if cache_changed
        || ((has_template(old) || has_template(new))
            && changes.iter().any(|change| {
                matches!(
                    change,
                    DomChange::Created(_)
                        | DomChange::Destroyed(_)
                        | DomChange::Parent(_)
                        | DomChange::Children(_)
                        | DomChange::NodeDocument(_)
                )
            }))
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::TemplateRelations);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    // All derived shadow relations are invalidated conservatively whenever a
    // structural edit can affect a managed shadow tree. Ordinary-only stores
    // retain their existing precise change contract.
    let has_shadow = |state: &State| state.counts.shadow_roots != 0;
    if (has_shadow(old) || has_shadow(new))
        && changes.iter().any(|change| {
            matches!(
                change,
                DomChange::Created(_)
                    | DomChange::Destroyed(_)
                    | DomChange::Parent(_)
                    | DomChange::Children(_)
                    | DomChange::NodeDocument(_)
                    | DomChange::ShadowRoot(_)
            )
        })
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::ShadowRelations);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    if old.details_intentions != new.details_intentions
        || old.next_details_intention != new.next_details_intention
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::DetailsToggleIntentions);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    if old.signal_slots != new.signal_slots
        || old.notification_queued != new.notification_queued
        || old.native_tasks != new.native_tasks
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::IntrinsicTasks);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    if !changes.is_empty() && (old.counts.slots != 0 || new.counts.slots != 0) {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::SlotQueries);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    Ok(changes)
}

#[cfg(test)]
pub(super) fn full_difference(
    owner: DomOwnerId,
    old: &State,
    new: &State,
    cancelled: &AtomicBool,
) -> Result<Vec<DomChange>, DomError> {
    let budget: Option<&source::formation::Formation> = None;
    let mut changes = Vec::new();
    for index in 0..old.ranges.len().max(new.ranges.len()) {
        check_cancel(cancelled)?;
        if let Some(budget) = budget {
            budget.retain_bytes(0, cancelled)?;
        }
        let before = old
            .ranges
            .get(index)
            .and_then(|slot| slot.record.as_ref().map(|record| (slot.generation, record)));
        let after = new
            .ranges
            .get(index)
            .and_then(|slot| slot.record.as_ref().map(|record| (slot.generation, record)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, record)) = before {
            let id = crate::DomRangeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                if record != after.expect("same Range").1 {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::RangeBoundary(id));
                }
            } else {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::RangeRetired(id));
            }
        }
        if !same && let Some((generation, _)) = after {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::RangeCreated(crate::DomRangeId {
                owner,
                slot: index,
                generation,
            }));
        }
    }
    for index in 0..old.slots.len().max(new.slots.len()) {
        let before = old
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let after = new
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, node)) = before {
            let id = NodeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                let current = after.expect("same live identity").1;
                if node.active_range() != current.active_range() {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentActiveRange(DocumentId(id)));
                }
                if let (DomNodeKind::Document(before), DomNodeKind::Document(after)) =
                    (node.kind(), current.kind())
                    && before.design_mode_enabled() != after.design_mode_enabled()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentDesignMode(DocumentId(id)));
                }
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (node.kind(), current.kind())
                    && before.captured_in_view_transition() != after.captured_in_view_transition()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::ViewTransitionCaptured(
                        DomViewTransitionSubject::Element(id),
                    ));
                }
                let intrinsic_changed = match (node.kind(), current.kind()) {
                    (DomNodeKind::Element(before), DomNodeKind::Element(after)) => {
                        !same_attributes(before, after)
                    }
                    (DomNodeKind::Document(before), DomNodeKind::Document(after)) => {
                        !std::ptr::eq(before, after) && !before.same_nondesign_mode_state(after)
                    }
                    _ => !same_language(node, current),
                };
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (node.kind(), current.kind())
                    && ((!std::ptr::eq(before, after) && !before.same_nonattribute_state(after))
                        || node.reflected_targets() != current.reflected_targets()
                        || node.control_owner() != current.control_owner()
                        || !same_optional(node.form_past_names(), current.form_past_names())
                        || !same_optional(node.base(), current.base())
                        || !same_optional(node.link(), current.link())
                        || !same_optional(node.image(), current.image())
                        || !same_optional(node.hyperlink(), current.hyperlink())
                        || !same_optional(node.script(), current.script())
                        || node.style_parser_document() != current.style_parser_document()
                        || node.option_select() != current.option_select()
                        || node.template_patch() != current.template_patch())
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::ElementState(id));
                }
                if !same_optional(node.base(), current.base())
                    && let Some(frozen) = current.base().and_then(|b| b.frozen.as_ref())
                {
                    let change = DomChange::DocumentBase(DocumentId(frozen.document));
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(change);
                }
                if !same_autofocus(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentAutofocusState(DocumentId(id)));
                }
                if !same_interactive(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentInteractiveState(DocumentId(id)));
                }
                if !same_slice(node.fullscreen_events(), current.fullscreen_events()) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentFullscreenEvents(DocumentId(id)));
                }
                if matches!(node.kind(), DomNodeKind::Document(_))
                    && fullscreen::captured_facts(old, node, owner, budget, cancelled)?
                        != fullscreen::captured_facts(new, current, owner, budget, cancelled)?
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentFullscreenState(DocumentId(id)));
                }
                if !same_base_document(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentBase(DocumentId(id)));
                }
                if !same_render_blocking(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentRenderBlocking(DocumentId(id)));
                }
                if let (DomNodeKind::Document(before), DomNodeKind::Document(after)) =
                    (node.kind(), current.kind())
                    && (before.meta_state() != after.meta_state()
                        || before.meta_theme_color() != after.meta_theme_color()
                        || before.meta_color_scheme != after.meta_color_scheme
                        || before.policy_container() != after.policy_container())
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentMeta(DocumentId(id)));
                }
                if !same_encoding_url(node, current) {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentEncodingUrlFacts(DocumentId(id)));
                }
                let kind_change = if intrinsic_changed {
                    Some(
                        if matches!(
                            (node.kind(), current.kind()),
                            (DomNodeKind::Element(_), DomNodeKind::Element(_))
                        ) {
                            DomChange::Attributes(id)
                        } else {
                            DomChange::Data(id)
                        },
                    )
                } else {
                    None
                };
                for change in [
                    kind_change,
                    (!same_slice(node.manual_nodes(), current.manual_nodes()))
                        .then_some(DomChange::SlotManual(id)),
                    (!same_slice(node.assigned_nodes(), current.assigned_nodes()))
                        .then_some(DomChange::SlotAssignedNodes(id)),
                    (node.manual_slot() != current.manual_slot())
                        .then_some(DomChange::ManualAssignment(id)),
                    (node.assigned_slot() != current.assigned_slot())
                        .then_some(DomChange::StoredAssignedSlot(id)),
                    (node.shadow_root() != current.shadow_root())
                        .then_some(DomChange::ShadowRoot(id)),
                    (node.document() != current.document()).then_some(DomChange::NodeDocument(id)),
                    (node.parent() != current.parent()).then_some(DomChange::Parent(id)),
                    (!same_slice(node.children(), current.children()))
                        .then_some(DomChange::Children(id)),
                ]
                .into_iter()
                .flatten()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(change);
                }
            } else {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::Destroyed(id));
            }
        }
        if let Some((generation, _)) = after
            && !same
        {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::Created(NodeId {
                owner,
                slot: index,
                generation,
            }));
        }
    }
    let has_template = |state: &State| {
        state.slots.values().any(|slot| {
            slot.node.as_ref().is_some_and(|node| {
                node.template_contents().is_some()
                    || node.template_document().is_some_and(|state| {
                        matches!(state, crate::template::TemplateDocument::InertTemplate)
                            || state.inert().is_some()
                    })
            })
        })
    };
    let cache_changed = old.slots.values().zip(&new.slots).any(|(a, b)| {
        a.node.as_ref().zip(b.node.as_ref()).is_some_and(|(a, b)| {
            a.template_document() != b.template_document()
                || a.template_contents() != b.template_contents()
                || a.template_patch() != b.template_patch()
        })
    });
    if cache_changed
        || ((has_template(old) || has_template(new))
            && changes.iter().any(|change| {
                matches!(
                    change,
                    DomChange::Created(_)
                        | DomChange::Destroyed(_)
                        | DomChange::Parent(_)
                        | DomChange::Children(_)
                        | DomChange::NodeDocument(_)
                )
            }))
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::TemplateRelations);
    }
    // All derived shadow relations are invalidated conservatively whenever a
    // structural edit can affect a managed shadow tree. Ordinary-only stores
    // retain their existing precise change contract.
    let has_shadow = |state: &State| {
        state.slots.values().any(|slot| {
            slot.node
                .as_ref()
                .is_some_and(|node| matches!(node.kind(), DomNodeKind::ShadowRoot(_)))
        })
    };
    if (has_shadow(old) || has_shadow(new))
        && changes.iter().any(|change| {
            matches!(
                change,
                DomChange::Created(_)
                    | DomChange::Destroyed(_)
                    | DomChange::Parent(_)
                    | DomChange::Children(_)
                    | DomChange::NodeDocument(_)
                    | DomChange::ShadowRoot(_)
            )
        })
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::ShadowRelations);
    }
    if old.details_intentions != new.details_intentions
        || old.next_details_intention != new.next_details_intention
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::DetailsToggleIntentions);
    }
    if old.signal_slots != new.signal_slots
        || old.notification_queued != new.notification_queued
        || old.native_tasks != new.native_tasks
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::IntrinsicTasks);
    }
    if !changes.is_empty()
        && [old, new].iter().any(|state| {
            state.slots.values().any(|s| {
                s.node
                    .as_ref()
                    .is_some_and(|n| crate::slot::is_slot(n.kind().into()))
            })
        })
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::SlotQueries);
    }
    new.view_transitions
        .changes(&old.view_transitions, &mut changes, budget, cancelled)?;
    // Complete pseudo origins derive from ordinary Document children. Report
    // that domain when a valid generic edit changes or removes their origin.
    for transition in new.view_transitions.transitions.values() {
        if old
            .view_transitions
            .transitions
            .values()
            .any(|t| t.id == transition.id)
        {
            let origin = |state: &State| {
                state
                    .node(owner, transition.document.node())
                    .ok()
                    .and_then(|document| {
                        document.children().iter().copied().find(|id| {
                            matches!(
                                state.node(owner, *id).map(Node::kind),
                                Ok(DomNodeKind::Element(_))
                            )
                        })
                    })
            };
            let change = DomChange::ViewTransition(transition.id);
            if origin(old) != origin(new) && !changes.contains(&change) {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(change);
            }
        }
    }
    Ok(changes)
}

pub(super) fn same_auxiliary<T: PartialEq>(left: &Option<Arc<T>>, right: &Option<Arc<T>>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b) || a == b,
        _ => false,
    }
}

#[cfg(test)]
mod top_layer_storage_tests {
    use super::*;

    #[test]
    fn empty_top_layer_targets_preserve_identity_and_absent_backing() {
        let absent = SharedVec::<NodeId>::default();
        let first = SharedVec::from_shared(Arc::new(Vec::new()));
        let alias = first.clone();
        let second = SharedVec::from_shared(Arc::new(Vec::new()));
        assert!(same_top_layer_backing(None, Some(&absent)));
        assert!(same_top_layer_backing(Some(&first), Some(&alias)));
        assert!(!same_top_layer_backing(None, Some(&first)));
        assert!(!same_top_layer_backing(Some(&first), Some(&second)));

        let reserved = SharedVec::from_shared(Arc::new(Vec::with_capacity(4)));
        let other_reserved = SharedVec::from_shared(Arc::new(Vec::with_capacity(4)));
        let reserved_alias = reserved.clone();
        assert!(same_top_layer_backing(
            Some(&reserved),
            Some(&reserved_alias)
        ));
        assert!(!same_top_layer_backing(None, Some(&reserved)));
        assert!(!same_top_layer_backing(
            Some(&reserved),
            Some(&other_reserved)
        ));
    }
}
