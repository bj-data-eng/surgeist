//! Actual Frame/Iframe child-navigable hooks over the canonical staged owner.
use super::*;
use crate::internal::{
    element::HostedContent,
    hosted::{DomHostedChildInsertion, DomHostedError},
};
use surgeist_html::elements::frame::{
    HtmlFramePostConnectionDecision, HtmlIframePostConnectionDecision,
};

impl Draft {
    /// Capture the structural attach before insertion hooks can park or reenter.
    pub(super) fn record_hosted_child_insertions(
        &mut self,
        subtree: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.hosted_history.is_none() || !self.containers.has_participants() {
            return Ok(());
        }
        self.source_step(cancel)?;
        let document = self.node(subtree)?.document();
        let metadata = self.document_metadata(document)?;
        if metadata.document_environment().browsing_context()
            != surgeist_html::security::nonce::HtmlBrowsingContextPresence::Present
        {
            return Ok(());
        }
        let mut containers = self.effect_vec();
        self.containers
            .visit_insertion(document, subtree, self, cancel, |key| {
                self.select_reserve(&mut containers, 1, cancel)?;
                containers.push(key);
                Ok(())
            })?;
        if containers.is_empty() {
            return Ok(());
        }
        let document = DocumentId(self.published_id(document).ok_or(DomError::WrongDocument)?);
        let group = self.next_hosted_insertion;
        let next = group
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if self.prepare_undo(cancel)? {
            self.record_source_undo_prepared(source::ControlUndo::HostedInsertion(group));
        }
        self.next_hosted_insertion = next;
        for (preorder, &container) in containers.iter().enumerate() {
            self.source_step(cancel)?;
            let element = self
                .node(container)?
                .element()
                .ok_or(DomError::WrongNodeKind)?
                .language();
            if element.hosted_insertion().is_some() || element.child_navigable() != Some(None) {
                return Err(DomError::HostedSourceUnavailable);
            }
            let preorder =
                u64::try_from(preorder).map_err(|_| DomError::RepresentationExhausted)?;
            self.replace_hosted_content(
                container,
                Some(HostedContent {
                    insertion: DomHostedChildInsertion {
                        document,
                        group,
                        preorder,
                    },
                    navigable: None,
                }),
                cancel,
            )?;
        }
        Ok(())
    }

    pub(in crate::internal::store) fn child_insertion_for_content(
        &self,
        container: Key,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedChildInsertion, DomHostedError> {
        self.source_step(cancel)?;
        let node = self.node(container)?;
        let element = node.element().ok_or(DomError::WrongNodeKind)?.language();
        let insertion = element
            .hosted_insertion()
            .ok_or(DomHostedError::MissingPublication)?;
        if insertion.document != document
            || self.published_id(node.document()) != Some(document.node())
            || self.published_id(self.shadow_including_root(container, cancel)?)
                != Some(document.node())
            || element.child_navigable() != Some(None)
        {
            return Err(DomHostedError::StaleBasis);
        }
        Ok(insertion)
    }

    /// Per-node family decision, after the shared native post-connection list
    /// has established actual current shadow-including connectedness.
    pub(super) fn navigable_post_connection_node(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let node = self.node(key)?;
        let Some(element) = node.element() else {
            return Ok(());
        };
        if element.child_navigable().is_none() {
            return Ok(());
        }
        let document = self.document_metadata(node.document())?;
        let context = document.document_environment().browsing_context();
        let returned = if element.iframe_initial_state().is_none() {
            matches!(
                surgeist_html::elements::frame::frame_post_connection(context),
                HtmlFramePostConnectionDecision::Return
            )
        } else {
            matches!(
                surgeist_html::elements::frame::iframe_post_connection(context),
                HtmlIframePostConnectionDecision::Return
            )
        };
        if !returned {
            if self.headroom_child_hook(false, cancel)? {
                return Ok(());
            }
            if self.source_limits.is_none() {
                return Err(DomError::HostedSourceUnavailable);
            }
            // Iframe's actual post-connection sandbox step precedes the child
            // creator. It changes only this element's own flag set.
            if self.element(key)?.iframe_initial_state().is_some() {
                let sandbox = match self.control_attribute(
                    key,
                    surgeist_html::vocabulary::KnownHtmlAttribute::Sandbox,
                    cancel,
                )? {
                    Some(raw) => surgeist_html::attributes::sandbox::HtmlSandboxFlags::parse_with(
                        raw,
                        || self.source_step(cancel),
                    )?,
                    None => self
                        .element(key)?
                        .iframe_sandbox_flags()
                        .expect("actual Iframe flag set"),
                };
                self.with_element_fields(key, cancel, |element| {
                    element.establish_iframe_sandbox_flags(sandbox)
                })??;
            }
            self.hosted_pending = Some(source::operation::hosted::ChildHook::Create { key });
            return Err(DomError::HostedCompositionRequired);
        }
        Ok(())
    }

    /// Both actual families invoke destroy-child after detachment. Their closed
    /// null relationship returns before any child/document/history effects.
    pub(super) fn navigable_removing_node(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let Some(element) = self.node(key)?.element() else {
            return Ok(());
        };
        let destroy = element.child_navigable_presence().is_some_and(
            surgeist_html::elements::frame::HtmlChildNavigablePresence::removal_requires_child_destruction,
        );
        if !destroy && element.language().hosted_insertion().is_some() {
            self.establish_child_navigable(key, None, cancel)?;
        }
        if destroy {
            let navigable = self
                .element(key)?
                .child_navigable()
                .flatten()
                .ok_or(DomError::HostedSourceUnavailable)?;
            if self.headroom_child_hook(true, cancel)? {
                self.establish_child_navigable(key, None, cancel)?;
                return Ok(());
            }
            if self.source_limits.is_none() {
                return Err(DomError::HostedSourceUnavailable);
            }
            self.establish_child_navigable(key, None, cancel)?;
            self.hosted_pending =
                Some(source::operation::hosted::ChildHook::Remove { key, navigable });
            return Err(DomError::HostedCompositionRequired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_after_frame_hook_preparation_preserves_slots_tree_state_and_history() {
        cancelled_prepared_navigable("frame");
    }

    #[test]
    fn cancellation_after_iframe_hook_preparation_preserves_slots_tree_state_and_history() {
        cancelled_prepared_navigable("iframe");
    }

    fn cancelled_prepared_navigable(local_name: &str) {
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                2048,
                32,
                0,
                0,
                8,
                8192,
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let frame = batch.create_element_ns(
            &document,
            surgeist_html::values::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
            local_name,
        );
        batch.insert_before(&frame, &document, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let frame = commit.created(&frame).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        let temporary = batch.create_element_ns(
            document,
            surgeist_html::values::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
            local_name,
        );
        batch.insert_before(&temporary, frame, None);
        batch.detach(frame);
        batch.set_attribute(frame, "src", "unpublished");
        batch.insert_before(frame, document, None);
        let prepared = dom.prepare(batch, &cancel).unwrap();
        cancel.store(true, Ordering::Release);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}
