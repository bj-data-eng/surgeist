//! HTML877d's Document.title setter on the original canonical Source owner.
use super::*;
use surgeist_html::values::Namespace;
use surgeist_html_semantics::HtmlDocumentTitleKind;

/// The insertion's original mandatory text-replacement tail. Keeping the actual
/// title key avoids reselecting after native effects or reentrant work.
#[derive(Clone)]
pub(in crate::internal::store) struct Frame {
    title: Key,
    value: DocumentString,
}
impl Frame {
    pub(in crate::internal::store) fn retained_payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        step()?;
        let mut payload = source::storage::Payload::default();
        payload.string(&self.value)?;
        Ok(payload)
    }
}

impl Draft {
    pub(super) fn set_document_title(
        &mut self,
        document: Key,
        value: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.document_metadata(document)?;
        let Some(root) = self
            .role_facts
            .document_element(self.live_key(document), &mut || self.source_step(cancel))?
        else {
            return Ok(());
        };
        let root_element = self.element(root)?;
        let name = root_element.name();
        let kind = HtmlDocumentTitleKind::for_root(name, || self.source_step(cancel))?;
        if kind == HtmlDocumentTitleKind::Html
            && !crate::internal::names::in_html_namespace_with(name, || self.source_step(cancel))?
        {
            return Ok(());
        }
        if let Some(title) = self.document_title_target(root, kind, cancel)? {
            let (value, payload) = source::formation::shared_string(self, value, cancel)?;
            return self.string_replace_all(title, value, payload, limits, cancel);
        }
        let (parent, before) = if kind == HtmlDocumentTitleKind::Svg {
            (root, self.node(root)?.children().first())
        } else {
            // The head accessor requires the exact HTML html document element.
            if !self.source_ascii(self.element(root)?.name().local_name(), "html", cancel)? {
                return Ok(());
            }
            let mut head = None;
            for child in self.node(root)?.children().iter() {
                self.source_step(cancel)?;
                if let transaction::KindRead::Element(element) = self.node(child)?.kind()
                    && crate::internal::names::in_html_namespace_with(element.name(), || {
                        self.source_step(cancel)
                    })?
                    && self.source_ascii(element.name().local_name(), "head", cancel)?
                {
                    head = Some(child);
                    break;
                }
            }
            let Some(head) = head else { return Ok(()) };
            (head, None)
        };
        let operands = source::formation::Formation::new(self, cancel)?;
        let uri = source::formation::string(
            if kind == HtmlDocumentTitleKind::Svg {
                "http://www.w3.org/2000/svg"
            } else {
                "http://www.w3.org/1999/xhtml"
            },
            operands.as_ref(),
            cancel,
        )?;
        let local = source::formation::string("title", operands.as_ref(), cancel)?;
        let name = crate::internal::source::DomElementCreationName::Qualified(
            Namespace::from_uri(uri),
            local,
        );
        let formation = source::formation::Formation::new(self, cancel)?;
        let element = self.native_element(
            &name,
            &crate::internal::source::DomElementCreationOptions::default(),
            self.document_metadata(document)?,
            formation.as_ref(),
            cancel,
        )?;
        drop(name);
        drop(operands);
        let title = self.allocate_element_formed(element, document, limits, cancel, formation)?;
        self.initialize_native_element(title, None, limits, cancel)?;
        let (value, payload) = source::formation::shared_string(self, value, cancel)?;
        self.push_effect_retained(
            select::Effect::DocumentTitle(Frame { title, value }),
            self.effect_scoped,
            payload,
            cancel,
        )?;
        self.pre_insert(title, parent, before, limits, cancel)
    }

    fn document_title_target(
        &self,
        root: Key,
        kind: HtmlDocumentTitleKind,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        if kind == HtmlDocumentTitleKind::Svg {
            for key in self.node(root)?.children().iter() {
                self.source_step(cancel)?;
                if let transaction::KindRead::Element(element) = self.node(key)?.kind()
                    && kind.matches(element.name(), || self.source_step(cancel))?
                {
                    return Ok(Some(key));
                }
            }
        } else {
            let mut current = Some(root);
            while let Some(key) = current {
                self.source_step(cancel)?;
                if let transaction::KindRead::Element(element) = self.node(key)?.kind()
                    && kind.matches(element.name(), || self.source_step(cancel))?
                {
                    return Ok(Some(key));
                }
                current = crate::internal::slot::next(self, root, key, cancel)?;
            }
        }
        Ok(None)
    }

    pub(in crate::internal::store) fn document_title_effect(
        &mut self,
        frame: Frame,
        payload: Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.string_replace_all(frame.title, frame.value, payload, limits, cancel)
    }
}
