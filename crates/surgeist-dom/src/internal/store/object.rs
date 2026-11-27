//! Reached Object representation occurrences in an unpublished Draft.

use super::source::operation::object_queue::DomObjectRepresentationTask;
use super::*;
use crate::internal::{details::DomElementTaskAssociation, name::DomQualifiedName};
use surgeist_html::{
    elements::object::{
        HtmlObjectResourceAttribute as Attribute, HtmlObjectResourcePrecedence as Precedence,
    },
    vocabulary::KnownHtmlAttribute,
};

#[derive(Clone, Copy)]
pub(super) struct Pending {
    pub(super) target: Key,
    pub(super) association: DomElementTaskAssociation,
    pub(super) node_document: Key,
}

pub(super) fn resource_attribute_with(
    name: &DomQualifiedName,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Option<Attribute>, DomError> {
    if name.namespace_uri().is_some() {
        return Ok(None);
    }
    let local = name.local_name().code_units();
    for (spelling, attr) in [
        ("classid", Attribute::ClassId),
        ("data", Attribute::Data),
        ("type", Attribute::Type),
    ] {
        step()?;
        if local.len() != spelling.len() {
            continue;
        }
        let mut matches = true;
        for (&unit, byte) in local.iter().zip(spelling.bytes()) {
            step()?;
            if unit != u16::from(byte) {
                matches = false;
                break;
            }
        }
        if matches {
            return Ok(Some(attr));
        }
    }
    Ok(None)
}

impl Dom {
    /// Run the admitted fallback body of an original queued Object task.
    /// The caller's task-bank work allowance is debited for actual Source
    /// work even on refusal. The task permit is consumed only on success.
    pub fn run_object_representation_task(
        &mut self,
        task: &mut Option<DomObjectRepresentationTask>,
        mut limits: crate::internal::source::DomSourceLimits,
        bank_work: &mut u64,
        cancel: &AtomicBool,
    ) -> Result<DomCommit, DomError> {
        limits.work = limits.work.min(*bank_work);
        let initial = source::initial::Initial::new(limits);
        let formed = (|| {
            initial.step(cancel)?;
            self.require_sync_source_idle()?;
            let permit = task.as_ref().ok_or(DomError::InvalidNativeTask)?;
            let binding = permit
                .binding
                .upgrade()
                .ok_or(DomError::InvalidNativeTask)?;
            if !self
                .element_tasks
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .is_some_and(|actual| Arc::ptr_eq(&actual, &binding))
            {
                return Err(DomError::InvalidNativeTask);
            }
            initial.retain(
                source::storage::Payload {
                    bytes: size_of::<BatchIdentity>(),
                    units: 0,
                },
                cancel,
            )?;
            let draft = Draft::new_formed(
                self,
                source::formation::allocated_arc(BatchIdentity),
                Some(&initial),
                cancel,
            )?;
            Ok((draft, permit.info()))
        })();
        let (mut draft, info) = match formed {
            Ok(formed) => formed,
            Err(error) => {
                *bank_work -= initial.spent_work();
                return Err(error);
            }
        };
        let result = (|| {
            let key = Key::Existing(info.id().target());
            draft.source_step(cancel)?;
            let element = draft.element(key)?;
            let representation = element
                .language()
                .object_representation()
                .ok_or(DomError::InvalidNativeTask)?;
            if draft.element_task_association(key, cancel)? != Some(info.queue_association()) {
                return Err(DomError::InvalidNativeTask);
            }
            // A detached or closed tree decides fallback before resource access.
            draft.object_in_closed_document(key, cancel)?;
            if representation
                != surgeist_html::elements::object::HtmlObjectRepresentation::FallbackContent
            {
                {
                    let mut fields = draft.element_fields_mut(key, cancel)?;
                    fields.show_object_fallback()?;
                    fields.commit(cancel)?;
                }
                draft.object_changes = true;
                let mut step = || draft.source_step(cancel);
                let mut member = crate::internal::slot::next_admitted(&draft, key, key, &mut step)?;
                while let Some(node) = member {
                    step()?;
                    if let transaction::KindRead::Element(element) = draft.node(node)?.kind() {
                        if element.object_representation().is_some() {
                            return Err(DomError::ObjectHostedTransitionUnavailable);
                        }
                        if element.is_inactive_embed() {
                            draft.check_embed_setup(node, cancel)?;
                        }
                    }
                    member = crate::internal::slot::next_admitted(&draft, key, node, &mut step)?;
                }
            }
            let prepared = draft.finish(self, cancel)?;
            if limits.phases == 0 && prepared.changed_from(&self.state) {
                return Err(DomError::SourcePublicationLimit);
            }
            if limits.outputs == 0 {
                return Err(DomError::SourceOutputLimit);
            }
            self.publish(prepared, cancel)
        })();
        *bank_work -= draft.source_work.load(Ordering::Relaxed);
        let commit = result?;
        task.take();
        Ok(commit)
    }
}

pub(super) fn object_context_with<'a, S>(
    mut ancestor: Option<Key>,
    step: &mut S,
    mut read: impl FnMut(Key, &mut S) -> Result<transaction::NodeRead<'a>, DomError>,
) -> Result<bool, DomError>
where
    S: FnMut() -> Result<(), DomError>,
{
    use surgeist_html::elements::object::HtmlObjectRepresentation;
    let mut shadow = false;
    while let Some(parent) = ancestor {
        step()?;
        let node = read(parent, step)?;
        match node.kind() {
            transaction::KindRead::Document(metadata) => {
                let context = metadata.document_environment().browsing_context();
                if context == surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent {
                    return Ok(!shadow);
                }
                return Err(DomError::ObjectHostedTransitionUnavailable);
            }
            transaction::KindRead::Element(element)
                if !shadow
                    && element.object_representation()
                        == Some(HtmlObjectRepresentation::Undetermined) =>
            {
                return Err(DomError::ObjectHostedTransitionUnavailable);
            }
            _ => {}
        }
        ancestor = match node.kind() {
            transaction::KindRead::ShadowRoot(_) => {
                shadow = true;
                node.host()
            }
            _ => node.parent(),
        };
    }
    Ok(false)
}

impl Draft {
    fn object_in_closed_document(&self, key: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        object_context_with(
            self.node(key)?.parent(),
            &mut || self.effect_step(cancel),
            |key, _| self.node(key),
        )
    }

    pub(super) fn object_tree_change(
        &mut self,
        key: Key,
        root: Key,
        old_parent: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let in_closed_document = self.object_in_closed_document(key, cancel)?;
        let reached = match old_parent {
            Some(parent) => self
                .former_ordinary_document(key, root, parent, &mut || self.effect_step(cancel))?
                .is_some(),
            None => in_closed_document,
        };
        if reached {
            self.queue_object_representation(key, cancel)?;
        }
        self.object_changes = true;
        Ok(())
    }

    fn require_object_document_open(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use crate::internal::hosted::{DocumentPhase, DomHostedError};
        let environment = self.document_metadata(document)?.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(());
        };
        let DocumentPhase::InitialEntry { navigable, .. } = association.phase else {
            return Ok(());
        };
        let graph = self
            .hosted_history
            .as_deref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let mut current = Some(navigable);
        while let Some(id) = current {
            let row = graph
                .navigable_with(id, || {
                    self.source_step(cancel).map_err(DomHostedError::from)
                })
                .map_err(|error| match error {
                    DomHostedError::Dom(error) => error,
                    _ => DomError::HostedSourceUnavailable,
                })?;
            if row.retiring && !row.retired {
                return Err(DomError::ObjectHostedTransitionUnavailable);
            }
            current = row.parent;
        }
        Ok(())
    }

    fn adjust_native_objects(
        &mut self,
        document: Key,
        add: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut count = self.native_object_count_mut(document, cancel)?;
        let next = if add {
            count.checked_add(1)
        } else {
            count.checked_sub(1)
        };
        *count = next.ok_or(DomError::RepresentationExhausted)?;
        count.commit(cancel)
    }

    pub(super) fn object_created(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        let node = self.node(key)?;
        if matches!(node.kind(), transaction::KindRead::Element(element)
            if element.object_representation().is_some())
        {
            let document = node.document();
            self.require_object_document_open(document, cancel)?;
            self.adjust_native_objects(document, true, cancel)?;
            self.queue_object_representation(key, cancel)?;
        }
        Ok(())
    }

    pub(super) fn object_retiring(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = self.node(key)?;
        if matches!(node.kind(), transaction::KindRead::Element(element)
            if element.object_representation().is_some())
        {
            self.adjust_native_objects(node.document(), false, cancel)?;
            self.object_changes = true;
        }
        Ok(())
    }

    pub(super) fn object_attribute_change(
        &mut self,
        key: Key,
        changed: &DomQualifiedName,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let element = self.element(key)?;
        if element.language().object_representation().is_none() {
            return Ok(());
        }
        let Some(attribute) = resource_attribute_with(changed, || self.source_step(cancel))? else {
            return Ok(());
        };
        if attribute == Attribute::ClassId {
            return self.queue_object_representation(key, cancel);
        }
        let mut precedence = Precedence::Neither;
        for attribute in element.attributes().iter() {
            self.source_step(cancel)?;
            if attribute.name().namespace_uri().is_none()
                && self.source_ascii(attribute.name().local_name(), "classid", cancel)?
            {
                precedence = Precedence::ClassId;
                break;
            }
        }
        if precedence == Precedence::Neither
            && self
                .control_attribute(key, KnownHtmlAttribute::Data, cancel)?
                .is_some()
        {
            precedence = Precedence::Data;
        }
        if attribute.queues_representation(precedence) {
            self.queue_object_representation(key, cancel)?;
        }
        self.object_changes = true;
        Ok(())
    }

    pub(super) fn object_adopted(
        &mut self,
        key: Key,
        previous_document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = self.node(key)?;
        if !matches!(node.kind(), transaction::KindRead::Element(element)
            if element.object_representation().is_some())
        {
            return Ok(());
        }
        let document = node.document();
        self.require_object_document_open(document, cancel)?;
        self.adjust_native_objects(previous_document, false, cancel)?;
        self.adjust_native_objects(document, true, cancel)?;
        if self.document_fully_active(previous_document, cancel)?
            != self.document_fully_active(document, cancel)?
        {
            self.queue_object_representation(key, cancel)?;
        }
        self.object_changes = true;
        Ok(())
    }

    fn queue_object_representation(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.source_limits.is_none() {
            return Err(DomError::ObjectTaskSourceUnavailable);
        }
        self.effect_step(cancel)?;
        let association = self
            .element_task_association(key, cancel)?
            .ok_or(DomError::ObjectTaskSourceUnavailable)?;
        let node_document = self.node(key)?.document();
        // The original relevant Global can differ from the node Document
        // after adoption. The admitted body must qualify for fallback before any fetch.
        self.object_in_closed_document(key, cancel)?;
        if self.published_id(node_document).is_none() {
            return Err(DomError::ObjectHostedTransitionUnavailable);
        }
        let mut tasks = self
            .object_tasks
            .take()
            .unwrap_or_else(|| self.effect_vec());
        let admission = self.select_reserve(&mut tasks, 1, cancel);
        if admission.is_ok() {
            tasks.push(Pending {
                target: key,
                association,
                node_document,
            });
        }
        self.object_tasks = Some(tasks);
        admission?;
        self.object_changes = true;
        Ok(())
    }
}
