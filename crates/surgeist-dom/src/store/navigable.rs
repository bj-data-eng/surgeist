//! Actual Frame/Iframe child-navigable hooks over the canonical staged owner.
use super::*;
use surgeist_html::{HtmlFramePostConnectionDecision, HtmlIframePostConnectionDecision};

impl Draft {
    /// Per-node family decision, after the shared native post-connection list
    /// has established actual current shadow-including connectedness.
    pub(super) fn navigable_post_connection_node(
        &self,
        key: Key,
        admit: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        admit()?;
        let node = self.node(key)?;
        let DomNodeKind::Element(element) = &node.kind else {
            return Ok(());
        };
        let frame = element.frame_null_content();
        let iframe = element.iframe_native_state();
        if frame.is_none() && iframe.is_none() {
            return Ok(());
        }
        let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
            unreachable!("canonical navigable-container node document")
        };
        let context = document.native_environment().browsing_context();
        let returned = if frame.is_some() {
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
            unreachable!("all admitted Documents retain native known-null browsing context")
        }
        Ok(())
    }

    /// Both actual families invoke destroy-child after detachment. Their closed
    /// null relationship returns before any child/document/history effects.
    pub(super) fn navigable_removing_node(
        &self,
        key: Key,
        admit: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        admit()?;
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Ok(());
        };
        let destroy = if let Some(state) = element.frame_null_content() {
            state.removal_requires_child_destruction()
        } else if let Some(state) = element.iframe_native_state() {
            state.removal_requires_child_destruction()
        } else {
            false
        };
        if destroy {
            unreachable!("stored native null content cannot destroy a child navigable")
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
