//! Actual Frame/Iframe child-navigable hooks over the canonical staged owner.
use super::*;
use surgeist_html::{HtmlFramePostConnectionDecision, HtmlIframePostConnectionDecision};

impl Draft {
    /// Per-node family decision, after the shared native post-connection list
    /// has established actual current shadow-including connectedness.
    pub(super) fn navigable_post_connection_node(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let node = self.node(key)?;
        let DomNodeKind::Element(element) = &node.kind else {
            return Ok(());
        };
        if element.child_navigable().is_none() {
            return Ok(());
        }
        let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
            unreachable!("canonical navigable-container node document")
        };
        let context = document.document_environment().browsing_context();
        let returned = if element.iframe_initial_state().is_none() {
            matches!(
                surgeist_html::frame_post_connection(context),
                HtmlFramePostConnectionDecision::Return
            )
        } else {
            matches!(
                surgeist_html::iframe_post_connection(context),
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
                    surgeist_html::KnownHtmlAttribute::Sandbox,
                    cancel,
                )? {
                    Some(raw) => surgeist_html::HtmlSandboxFlags::parse_with(raw, || {
                        self.source_step(cancel)
                    })?,
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
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Ok(());
        };
        let destroy = element.child_navigable_presence().is_some_and(
            surgeist_html::HtmlChildNavigablePresence::removal_requires_child_destruction,
        );
        if destroy {
            let navigable = self
                .element(key)?
                .child_navigable()
                .flatten()
                .ok_or(DomError::HostedSourceUnavailable)?;
            if self.headroom_child_hook(true, cancel)? {
                self.with_element_fields(key, cancel, |element| {
                    element.establish_child_navigable(None)
                })??;
                return Ok(());
            }
            if self.source_limits.is_none() {
                return Err(DomError::HostedSourceUnavailable);
            }
            self.with_element_fields(key, cancel, |element| {
                element.establish_child_navigable(None)
            })??;
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
        let mut dom = Dom::new(DomLimits::new(16, 2048, 32, 0, 0, 8, 8192).unwrap()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let frame = batch.create_element_ns(
            &document,
            crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
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
            crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
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
