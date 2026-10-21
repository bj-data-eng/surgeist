//! Select a mutable relation owner without copying its canonical sequence.
//! The captured sequence retains the original group's control alongside its
//! backing; only its kind-sized transaction wrapper is newly allocated.
use super::super::*;
use super::{RelationField, Sequence, Undo, fields::FieldUndo, staged::*};

impl Draft {
    pub(in crate::store) fn prepare_relation_owner(
        &mut self,
        key: Key,
        field: RelationField,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let key = self.live_key(key);
        self.materialize_node(key, cancel)?;
        if self.records_for(key).relation(field).is_some() {
            return Ok(());
        }
        // Availability is a semantic fact. An absent/unavailable Document
        // group must never become a known empty owner merely to permit a write.
        match field {
            RelationField::Children => {
                if !matches!(
                    self.records_for(key),
                    DraftNode::Document(_)
                        | DraftNode::Element(_)
                        | DraftNode::DocumentFragment(_)
                        | DraftNode::ShadowRoot(_)
                ) {
                    return Err(DomError::WrongNodeKind);
                }
            }
            RelationField::ManualNodes | RelationField::AssignedNodes => {
                match self.records_for(key).associations() {
                    Some(Associations::Slot(_)) => {}
                    Some(Associations::None)
                        if self
                            .node(key)?
                            .element()
                            .is_some_and(|value| value.is_slot()) => {}
                    _ => return Err(DomError::WrongNodeKind),
                }
            }
            RelationField::RenderBlocking => {
                match self
                    .records_for(key)
                    .document_fields()
                    .map(|value| &value.base)
                {
                    Some(BaseDocumentGroup::KnownEmpty | BaseDocumentGroup::Known(_)) => {}
                    _ => return Err(DomError::WrongNodeKind),
                }
            }
            RelationField::AutofocusCandidates => {
                match self
                    .records_for(key)
                    .document_fields()
                    .map(|value| &value.autofocus)
                {
                    Some(AutofocusGroup::KnownEmpty { .. } | AutofocusGroup::Known(_)) => {}
                    Some(AutofocusGroup::Unavailable) => {
                        return Err(DomError::AutofocusDocumentStateUnavailable);
                    }
                    _ => return Err(DomError::WrongNodeKind),
                }
            }
            RelationField::OpenDialogs
            | RelationField::TopLayer
            | RelationField::PendingTopLayerRemovals => {
                match self
                    .records_for(key)
                    .document_fields()
                    .map(|value| &value.interactive)
                {
                    Some(InteractiveGroup::KnownEmpty | InteractiveGroup::Known(_)) => {}
                    Some(InteractiveGroup::Unavailable) => {
                        return Err(DomError::DialogDocumentStateUnavailable);
                    }
                    _ => return Err(DomError::WrongNodeKind),
                }
            }
        }
        let undo = self.prepare_undo(cancel)?;
        let registry = self.custody_registry().clone();
        let old = match field {
            RelationField::Children => {
                let next = boxed(
                    &registry,
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    || {
                        let values = match self.records_for(key).children() {
                            Some(Group::Canonical(value)) => Sequence::captured(
                                &value.value.value.values,
                                value.custody.clone(),
                                Key::Existing,
                            ),
                            None => Sequence::empty(Key::Existing, registry.ledger(), |usage| {
                                self.admit_custody_usage(usage, cancel)
                            })?,
                            Some(Group::Changed(_)) => unreachable!("already prepared children"),
                        };
                        Ok(Children { values })
                    },
                )?;
                FieldUndo::Children(
                    self.private_node_mut(key)
                        .children_mut()
                        .expect("validated container")
                        .replace(Group::Changed(next)),
                )
            }
            RelationField::ManualNodes | RelationField::AssignedNodes => {
                let next = boxed(
                    &registry,
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    || {
                        Ok(
                            match self
                                .records_for(key)
                                .associations()
                                .expect("validated Slot")
                            {
                                Associations::Slot(Group::Canonical(value)) => SlotFields {
                                    manual: Sequence::captured(
                                        &value.value.value.manual_nodes,
                                        value.custody.clone(),
                                        Key::Existing,
                                    ),
                                    assigned: Sequence::captured(
                                        &value.value.value.assigned_nodes,
                                        value.custody.clone(),
                                        Key::Existing,
                                    ),
                                },
                                Associations::None => SlotFields {
                                    manual: Sequence::empty(
                                        Key::Existing,
                                        registry.ledger(),
                                        |usage| self.admit_custody_usage(usage, cancel),
                                    )?,
                                    assigned: Sequence::empty(
                                        Key::Existing,
                                        registry.ledger(),
                                        |usage| self.admit_custody_usage(usage, cancel),
                                    )?,
                                },
                                _ => unreachable!("already validated Slot group"),
                            },
                        )
                    },
                )?;
                FieldUndo::Associations(std::mem::replace(
                    self.private_node_mut(key)
                        .associations_mut()
                        .expect("validated Slot"),
                    Associations::Slot(Group::Changed(next)),
                ))
            }
            RelationField::RenderBlocking => {
                let next = boxed(
                    &registry,
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    || {
                        Ok(
                            match &self
                                .records_for(key)
                                .document_fields()
                                .expect("Document")
                                .base
                            {
                                BaseDocumentGroup::Known(Group::Canonical(value)) => {
                                    BaseDocumentFields {
                                        facts: value.value.value.facts.clone(),
                                        policy_basis: value.value.value.policy_basis.clone(),
                                        metadata_credit: MetadataCredit::Captured {
                                            _custody: value.custody.clone(),
                                        },
                                        first: value.value.value.first.map(Key::Existing),
                                        render: Sequence::captured(
                                            &value.value.value.render_blocking,
                                            value.custody.clone(),
                                            Key::Existing,
                                        ),
                                    }
                                }
                                BaseDocumentGroup::KnownEmpty => BaseDocumentFields {
                                    facts: None,
                                    policy_basis: None,
                                    metadata_credit: MetadataCredit::None,
                                    first: None,
                                    render: Sequence::empty(
                                        Key::Existing,
                                        registry.ledger(),
                                        |usage| self.admit_custody_usage(usage, cancel),
                                    )?,
                                },
                                _ => unreachable!("validated known Base Document group"),
                            },
                        )
                    },
                )?;
                FieldUndo::BaseDocument(std::mem::replace(
                    &mut self
                        .private_node_mut(key)
                        .document_fields_mut()
                        .expect("Document")
                        .base,
                    BaseDocumentGroup::Known(Group::Changed(next)),
                ))
            }
            RelationField::AutofocusCandidates => {
                let next = boxed(
                    &registry,
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    || {
                        Ok(
                            match &self
                                .records_for(key)
                                .document_fields()
                                .expect("Document")
                                .autofocus
                            {
                                AutofocusGroup::Known(Group::Canonical(value)) => AutofocusFields {
                                    processed: value.value.value.processed,
                                    candidates: Sequence::captured(
                                        &value.value.value.candidates,
                                        value.custody.clone(),
                                        Key::Existing,
                                    ),
                                },
                                AutofocusGroup::KnownEmpty { processed } => AutofocusFields {
                                    processed: *processed,
                                    candidates: Sequence::empty(
                                        Key::Existing,
                                        registry.ledger(),
                                        |usage| self.admit_custody_usage(usage, cancel),
                                    )?,
                                },
                                _ => unreachable!("validated known Autofocus group"),
                            },
                        )
                    },
                )?;
                FieldUndo::Autofocus(std::mem::replace(
                    &mut self
                        .private_node_mut(key)
                        .document_fields_mut()
                        .expect("Document")
                        .autofocus,
                    AutofocusGroup::Known(Group::Changed(next)),
                ))
            }
            RelationField::OpenDialogs
            | RelationField::TopLayer
            | RelationField::PendingTopLayerRemovals => {
                let next = boxed(
                    &registry,
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    || {
                        Ok(
                            match &self
                                .records_for(key)
                                .document_fields()
                                .expect("Document")
                                .interactive
                            {
                                InteractiveGroup::Known(Group::Canonical(value)) => {
                                    InteractiveFields {
                                        open: Sequence::captured(
                                            &value.value.value.open_dialogs,
                                            value.custody.clone(),
                                            Key::Existing,
                                        ),
                                        top: Sequence::captured(
                                            &value.value.value.top_layer,
                                            value.custody.clone(),
                                            Key::Existing,
                                        ),
                                        pending: Sequence::captured(
                                            &value.value.value.pending_top_layer_removals,
                                            value.custody.clone(),
                                            Key::Existing,
                                        ),
                                    }
                                }
                                InteractiveGroup::KnownEmpty => InteractiveFields {
                                    open: Sequence::empty(
                                        Key::Existing,
                                        registry.ledger(),
                                        |usage| self.admit_custody_usage(usage, cancel),
                                    )?,
                                    top: Sequence::empty(
                                        Key::Existing,
                                        registry.ledger(),
                                        |usage| self.admit_custody_usage(usage, cancel),
                                    )?,
                                    pending: Sequence::empty(
                                        Key::Existing,
                                        registry.ledger(),
                                        |usage| self.admit_custody_usage(usage, cancel),
                                    )?,
                                },
                                _ => unreachable!("validated known interaction group"),
                            },
                        )
                    },
                )?;
                FieldUndo::Interactive(std::mem::replace(
                    &mut self
                        .private_node_mut(key)
                        .document_fields_mut()
                        .expect("Document")
                        .interactive,
                    InteractiveGroup::Known(Group::Changed(next)),
                ))
            }
        };
        self.retain_relation_owner_inverse(key, old, undo);
        Ok(())
    }

    pub(in crate::store) fn prepare_fullscreen_owner(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let key = self.live_key(key);
        self.materialize_node(key, cancel)?;
        let document = self
            .records_for(key)
            .document_fields()
            .ok_or(DomError::WrongNodeKind)?;
        if matches!(document.fullscreen, Some(Group::Changed(_))) {
            return Ok(());
        }
        let undo = self.prepare_undo(cancel)?;
        let registry = self.custody_registry().clone();
        let next = boxed(
            &registry,
            &mut |usage| self.admit_custody_usage(usage, cancel),
            || {
                Ok(
                    match &self
                        .records_for(key)
                        .document_fields()
                        .expect("Document")
                        .fullscreen
                    {
                        Some(Group::Canonical(value)) => Sequence::captured(
                            &value.value.value.values,
                            value.custody.clone(),
                            std::convert::identity,
                        ),
                        None => {
                            Sequence::empty(std::convert::identity, registry.ledger(), |usage| {
                                self.admit_custody_usage(usage, cancel)
                            })?
                        }
                        Some(Group::Changed(_)) => {
                            unreachable!("already prepared fullscreen queue")
                        }
                    },
                )
            },
        )?;
        let old = FieldUndo::FullscreenGroup(
            self.private_node_mut(key)
                .document_fields_mut()
                .expect("Document")
                .fullscreen
                .replace(Group::Changed(next)),
        );
        self.retain_relation_owner_inverse(key, old, undo);
        Ok(())
    }

    fn retain_relation_owner_inverse(&mut self, key: Key, old: FieldUndo, undo: bool) {
        if undo {
            // Structural capture changes no semantic values, counts, or units.
            // Sequence edits append their own inverses after this group inverse.
            self.journal.push_prepared(Undo::Selected {
                key,
                old,
                counts: self.counts,
                units: self.units,
                private_units: self.private_units,
                private_counts: self.private_counts,
            });
        }
    }
}
