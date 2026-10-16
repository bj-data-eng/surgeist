//! HTML877d native Template contents/appropriate owner, in this staged owner.
use super::*;
use crate::template::TemplateDocument;

impl Draft {
    pub(super) fn owned_template_contents(
        &self,
        template: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        let Some(contents) = self.node(template)?.template_contents else {
            return Ok(None);
        };
        let target = self.node(contents)?;
        Ok(
            (matches!(target.kind, DomNodeKind::DocumentFragment) && target.host == Some(template))
                .then_some(contents),
        )
    }
    pub(super) fn retire_template_contents_target(
        &mut self,
        retired: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !matches!(self.node(retired)?.kind, DomNodeKind::ShadowRoot(_)) {
            return Ok(());
        }
        let extent = self
            .existing
            .len()
            .checked_add(self.created.len())
            .ok_or(DomError::RepresentationExhausted)?;
        for index in 0..extent {
            self.source_step(cancel)?;
            let key = if index < self.existing.len() {
                Key::Existing(index)
            } else {
                Key::Created(index - self.existing.len())
            };
            if self
                .node(key)
                .is_ok_and(|node| node.template_contents == Some(retired))
            {
                self.node_mut(key).template_contents = None;
            }
        }
        Ok(())
    }
    fn appropriate_template_document(
        &mut self,
        document: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        check_cancel(cancel)?;
        let state = self
            .node(document)?
            .template_document
            .expect("Document role");
        match surgeist_html::template_contents_owner_document(
            matches!(state, TemplateDocument::InertTemplate),
            state.inert().is_some(),
        ) {
            surgeist_html::HtmlTemplateContentsOwnerDocument::CurrentDocument => Ok(document),
            surgeist_html::HtmlTemplateContentsOwnerDocument::AssociatedInertDocument => {
                Ok(state.inert().expect("actual cached Document"))
            }
            surgeist_html::HtmlTemplateContentsOwnerDocument::CreateAssociatedInertDocument => {
                let DomNodeKind::Document(source) = &self.node(document)?.kind else {
                    unreachable!("Document role requires Document")
                };
                let formation = source::formation::Formation::new(self, cancel)?;
                let metadata = DomDocumentMetadata::template_owner(
                    source.class(),
                    source.document_environment(),
                    &mut |text| source::formation::string(text, formation.as_ref(), cancel),
                )?;
                let key = Key::Created(self.created.len());
                let key = self.allocate(DomNodeKind::Document(metadata), key, limits, cancel)?;
                drop(formation);
                self.node_mut(key).template_document = Some(TemplateDocument::InertTemplate);
                self.node_mut(document).template_document =
                    Some(TemplateDocument::Ordinary { inert: Some(key) });
                check_cancel(cancel)?;
                Ok(key)
            }
        }
    }
    pub(super) fn initialize_template_contents(
        &mut self,
        node: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !matches!(&self.node(node)?.kind, DomNodeKind::Element(element)
            if element.is_template())
        {
            return Ok(());
        }
        let document =
            self.appropriate_template_document(self.node(node)?.document, limits, cancel)?;
        let contents = self.allocate(DomNodeKind::DocumentFragment, document, limits, cancel)?;
        self.node_mut(contents).host = Some(node);
        self.node_mut(node).template_contents = Some(contents);
        self.node_mut(node).template_patch = Some(crate::template::TemplatePatch::fresh());
        check_cancel(cancel)
    }
    pub(super) fn template_adoption_target(
        &mut self,
        node: Key,
        document: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Option<(Key, Key)>, DomError> {
        let Some(contents) = self.owned_template_contents(node, cancel)? else {
            return Ok(None);
        };
        let target = self.appropriate_template_document(document, limits, cancel)?;
        Ok((self.node(contents)?.document != target).then_some((contents, target)))
    }
}

impl Draft {
    pub(super) fn retire_template_patch_target(
        &mut self,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for i in 0..self.existing.len() + self.created.len() {
            self.effect_step(cancel)?;
            let key = if i < self.existing.len() {
                Key::Existing(i)
            } else {
                Key::Created(i - self.existing.len())
            };
            let present = match key {
                Key::Existing(i) => self.existing[i].is_some(),
                Key::Created(i) => self.created[i].is_some(),
            };
            if !present {
                continue;
            }
            if let Some(patch) = &mut self.node_mut(key).template_patch {
                if patch.target == Some(target) {
                    patch.target = None;
                }
                if patch.start == Some(target) {
                    patch.start = None;
                }
                if patch.end == Some(target) {
                    patch.end = None;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    fn limits() -> DomLimits {
        DomLimits::new(512, 65536, 512, 256, 128, 8, 131072).unwrap()
    }
    #[test]
    fn privately_allocated_contents_and_cache_have_no_ticket_or_publication_on_cancel_or_exhaustion()
     {
        let mut dom = Dom::new(limits()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        let d = b.create_document();
        let c = dom.apply(b, &cancel).unwrap();
        let d = c.document(&d).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.create_element_ns(d, crate::Namespace::html(), "template");
        let mut draft = Draft::new(&dom, b.owner.clone(), &cancel).unwrap();
        for command in b.commands {
            draft.apply(&command, limits(), &cancel).unwrap();
        }
        assert_eq!(draft.created.len(), 3); //Element, real inert Document, contents
        assert_eq!(draft.tickets.len(), 1); //no private allocation aliases
        assert!(
            draft
                .node(Key::Existing(d.node().slot))
                .unwrap()
                .template_document
                .unwrap()
                .inert()
                .is_some()
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            draft.finish(&dom, &cancel),
            Err(DomError::Cancelled)
        ));
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        cancel.store(false, Ordering::Release);
        dom.revision = DomRevision(u64::MAX);
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        b.create_element_ns(d, crate::Namespace::html(), "template");
        let prepared = dom.prepare(b, &cancel).unwrap();
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    #[test]
    fn prepared_deep_clone_adoption_and_owned_destruction_cancel_together() {
        let mut dom = Dom::new(limits()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        let d = b.create_document();
        let t = b.create_element_ns(&d, crate::Namespace::html(), "template");
        let c = dom.apply(b, &cancel).unwrap();
        let d = c.document(&d).unwrap();
        let t = c.created(&t).unwrap().unwrap();
        let mut parent = dom
            .snapshot()
            .node(t)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap();
        let inert = dom
            .snapshot()
            .node(parent)
            .unwrap()
            .owner_document()
            .unwrap();
        for _ in 0..64 {
            let mut b = DomBatch::new();
            let child = b.create_element_ns(inert, crate::Namespace::html(), "template");
            b.insert_before(&child, parent, None);
            let c = dom.apply(b, &cancel).unwrap();
            parent = dom
                .snapshot()
                .node(c.created(&child).unwrap().unwrap())
                .unwrap()
                .as_template()
                .unwrap()
                .contents()
                .unwrap();
        }
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        let target = b.create_document();
        let copy = b.import_node(&before, t, &target, true);
        b.adopt(t, &target);
        b.destroy(&copy);
        let prepared = dom.prepare(b, &cancel).unwrap();
        // Actual complete clone/adopt/retirement state is private, not a fake
        // owner or allocation-failure injector. Cancel at final publication.
        cancel.store(true, Ordering::Release);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.snapshot()
                .document(d)
                .unwrap()
                .associated_inert_template_document(),
            Some(inert)
        );
    }
}
