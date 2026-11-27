//! Reached native Embed and out-of-band Track effects.

use super::*;
use crate::internal::{
    element::{DomElement, NativeFamily, TrackUrlChange},
    name::DomQualifiedName,
};
use surgeist_html::security::nonce::HtmlBrowsingContextPresence;
use surgeist_html::vocabulary::KnownHtmlAttribute;
use surgeist_html_semantics::HtmlEncodingUrlParseOutcome;

impl Draft {
    pub(super) fn embedded_attribute_change(
        &self,
        key: Key,
        name: &DomQualifiedName,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.element(key)?.language().is_inactive_embed()
            && name.namespace_uri().is_none()
            && (self.source_ascii(name.local_name(), "src", cancel)?
                || self.source_ascii(name.local_name(), "type", cancel)?)
        {
            self.check_embed_setup(key, cancel)?;
        }
        Ok(())
    }

    /// Resolve the source effect before projecting the Attr and native slot's
    /// combined logical units. Their overlapping backing has separate custody.
    pub(super) fn track_url_for_attribute_set<'a>(
        &'a self,
        key: Key,
        element: &DomElement,
        name: &DomQualifiedName,
        value: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<TrackUrlChange<'a>, DomError> {
        let NativeFamily::Track(previous) = element.native_family() else {
            return Ok(TrackUrlChange::Unchanged);
        };
        if name.namespace_uri().is_some() || !self.source_ascii(name.local_name(), "src", cancel)? {
            return Ok(TrackUrlChange::Unchanged);
        }
        self.effect_step(cancel)?;
        // The admitted Track is disabled. Its loading algorithm returns
        // before membership or fetch, including after a source change.
        let url = if value.is_empty() {
            None
        } else {
            let document = self.node(key)?.document();
            match self.resolve_encoding_url_units(document, value.code_units(), cancel)? {
                HtmlEncodingUrlParseOutcome::Failure => None,
                HtmlEncodingUrlParseOutcome::Parsed(url) => Some(url.serialization()),
            }
        };
        let same = match (previous.as_deref(), url) {
            (None, None) => true,
            (Some(previous), Some(url)) => {
                crate::internal::names::same_string(&previous.value, url, || {
                    self.effect_step(cancel)
                })?
            }
            _ => false,
        };
        Ok(if same {
            TrackUrlChange::Unchanged
        } else {
            TrackUrlChange::Set(url)
        })
    }

    pub(super) fn check_embed_setup(&self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        self.effect_step(cancel)?;
        let NativeFamily::Embed {
            in_document_history,
        } = self.element(key)?.language().native_family()
        else {
            return Err(DomError::WrongNodeKind);
        };
        let document = self.node(key)?.document();
        if self
            .document_metadata(document)?
            .document_environment()
            .browsing_context()
            == HtmlBrowsingContextPresence::Absent
        {
            return Ok(());
        }
        // The pinned ordinary Document condition also looks back one loop
        // step. Prior published membership can matter after detachment.
        if !*in_document_history {
            let tree =
                crate::internal::slot::root_admitted(self, key, &mut || self.effect_step(cancel))?;
            if !matches!(self.node(tree)?.kind(), transaction::KindRead::Document(_)) {
                return Ok(());
            }
        }
        let src = self.control_attribute(key, KnownHtmlAttribute::Src, cancel)?;
        if src.is_some_and(DocumentString::is_empty)
            || (src.is_none()
                && self
                    .control_attribute(key, KnownHtmlAttribute::Type, cancel)?
                    .is_none())
        {
            return Ok(());
        }
        let mut ancestor = self.node(key)?.parent();
        while let Some(parent) = ancestor {
            self.effect_step(cancel)?;
            let node = self.node(parent)?;
            if matches!(node.kind(), transaction::KindRead::Element(element)
                if element.initial_media_state().is_some()
                    || element.object_representation().is_some_and(|state|
                        state != surgeist_html::elements::object::HtmlObjectRepresentation::FallbackContent))
            {
                return Ok(());
            }
            ancestor = node.parent();
        }
        Err(DomError::EmbedSetupUnavailable)
    }

    pub(super) fn embedded_tree_change(
        &mut self,
        key: Key,
        root: Key,
        old_parent: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let transaction::KindRead::Element(element) = self.node(key)?.kind() else {
            return Ok(());
        };
        match element.native_family() {
            NativeFamily::Embed {
                in_document_history,
            } => {
                if !*in_document_history
                    && old_parent.is_some()
                    && self.embed_was_published_in_document(key, cancel)?
                {
                    let mut fields = self.element_fields_mut(key, cancel)?;
                    fields.note_embed_document_membership();
                    fields.commit(cancel)?;
                }
                self.check_embed_setup(key, cancel)
            }
            NativeFamily::Track(_) if key == root => {
                self.effect_step(cancel)?;
                let parent = self.node(key)?.parent();
                if old_parent == parent {
                    return Ok(());
                }
                for parent in [old_parent, parent].into_iter().flatten() {
                    self.effect_step(cancel)?;
                    if matches!(self.node(parent)?.kind(), transaction::KindRead::Element(element)
                        if element.initial_media_state().is_some())
                    {
                        return Err(DomError::TrackMembershipUnavailable);
                    }
                }
                Ok(())
            }
            NativeFamily::Object(_) => self.object_tree_change(key, root, old_parent, cancel),
            _ => Ok(()),
        }
    }

    fn embed_was_published_in_document(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        // A private insert/detach pair cannot have reached an event-loop turn.
        let Some(mut id) = self.published_id(key).filter(|_| !self.is_private(key)) else {
            return Ok(false);
        };
        loop {
            let node = self
                .base
                .node_with(self.store, id, &mut || self.effect_step(cancel))?;
            match node.parent() {
                Some(parent) => id = parent,
                None => return Ok(matches!(node.kind(), DomNodeKind::Document(_))),
            }
        }
    }

    pub(super) fn embedded_adoption(&self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        if let transaction::KindRead::Element(element) = self.node(key)?.kind()
            && element.is_inactive_embed()
        {
            self.check_embed_setup(key, cancel)?;
        }
        Ok(())
    }
}
