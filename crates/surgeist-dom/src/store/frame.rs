//! Actual Frame tree hooks over the canonical staged owner.
use super::*;
use surgeist_html::{HtmlFrameNullContent, HtmlFramePostConnectionDecision};

impl Draft {
    /// DOM post-connection applies only to nodes still connected after all
    /// insertion steps. A shadow root crosses its actual host for this test;
    /// its node document is deliberately not inferred from that host.
    pub(super) fn frame_post_connection_steps(
        &self,
        root: Key,
        admit: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let mut ancestor = root;
        loop {
            admit()?;
            match self.host_including_parent(ancestor)? {
                Some(parent) => ancestor = parent,
                None => break,
            }
        }
        if !matches!(self.node(ancestor)?.kind, DomNodeKind::Document(_)) {
            return Ok(());
        }
        self.for_each_frame(root, admit, |node, _| {
            let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
                unreachable!("canonical Frame node document")
            };
            match surgeist_html::frame_post_connection(
                document.native_environment().browsing_context(),
            ) {
                HtmlFramePostConnectionDecision::Return => Ok(()),
                HtmlFramePostConnectionDecision::Continue
                | HtmlFramePostConnectionDecision::NeedsBrowsingContext => {
                    unreachable!("all admitted Documents retain native known-null browsing context")
                }
            }
        })
    }

    /// Source Frame removing invokes destroy-child; its actual null relationship
    /// returns before any child/document/history work. This runs after detach,
    /// including when a containing ordinary or shadow subtree is removed.
    pub(super) fn frame_removing_node(
        &self,
        key: Key,
        admit: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        admit()?;
        if let DomNodeKind::Element(element) = &self.node(key)?.kind
            && let Some(state) = element.frame_null_content()
            && state.removal_requires_child_destruction()
        {
            unreachable!("stored Frame null content cannot destroy a child navigable")
        }
        Ok(())
    }

    fn for_each_frame(
        &self,
        root: Key,
        admit: &mut impl FnMut() -> Result<(), DomError>,
        mut frame: impl FnMut(&DraftNode, HtmlFrameNullContent) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let mut pending = Vec::new();
        reserve(&mut pending, 1)?;
        pending.push(root);
        while let Some(key) = pending.pop() {
            admit()?;
            let node = self.node(key)?;
            reserve(&mut pending, node.children.len())?;
            pending.extend(node.children.iter().rev().copied());
            if let Some(root) = node.shadow_root {
                reserve(&mut pending, 1)?;
                pending.push(root);
            }
            if let DomNodeKind::Element(element) = &node.kind
                && let Some(state) = element.frame_null_content()
            {
                frame(node, state)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_after_frame_hook_preparation_preserves_slots_tree_state_and_history() {
        let mut dom = Dom::new(DomLimits::new(16, 2048, 32, 0, 0, 8, 8192).unwrap()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let frame = batch.create_element_ns(
            &document,
            crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
            "frame",
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
            "frame",
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
