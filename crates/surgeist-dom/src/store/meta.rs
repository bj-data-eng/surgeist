//! Native Meta source hooks against actual staged Document history.
use super::*;
use crate::DomMetaError as Failure;
use surgeist_html::{HtmlMetaPragmaState as Pragma, KnownHtmlAttribute as Attr};

impl Draft {
    pub(super) fn meta_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let next = self
            .meta_work
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.meta_work_limit {
            return Err(DomError::Meta(Failure::WorkLimit {
                limit: self.meta_work_limit,
            }));
        }
        self.meta_work.set(next);
        Ok(())
    }
    fn is_meta(&self, key: Key) -> Result<bool, DomError> {
        Ok(matches!(&self.node(key)?.kind, DomNodeKind::Element(element) if element.is_meta()))
    }
    fn meta_ascii(
        &self,
        value: &DocumentString,
        expected: &str,
        insensitive: bool,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.meta_step(cancel)?;
        if value.len() != expected.len() {
            return Ok(false);
        }
        for (unit, byte) in value.code_units().iter().zip(expected.bytes()) {
            self.meta_step(cancel)?;
            let matches = if insensitive {
                u8::try_from(*unit).is_ok_and(|u| u.eq_ignore_ascii_case(&byte))
            } else {
                *unit == u16::from(byte)
            };
            if !matches {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn meta_attribute(
        &self,
        key: Key,
        attribute: Attr,
        cancel: &AtomicBool,
    ) -> Result<Option<&DocumentString>, DomError> {
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Ok(None);
        };
        for entry in element.attributes().iter() {
            self.meta_step(cancel)?;
            if entry.name().namespace().uri().is_none()
                && self.meta_ascii(
                    entry.name().local_name(),
                    attribute.local_name(),
                    false,
                    cancel,
                )?
            {
                #[cfg(test)]
                source_observation::retained(self, entry.value(), false, cancel)?;
                return Ok(Some(entry.value()));
            }
        }
        Ok(None)
    }
    fn meta_document_tree(
        &self,
        mut key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        loop {
            self.meta_step(cancel)?;
            let node = self.node(key)?;
            match node.parent {
                Some(parent) => key = parent,
                None => return Ok(matches!(node.kind, DomNodeKind::Document(_)).then_some(key)),
            }
        }
    }
    fn meta_metadata(&self, document: Key) -> Result<&DomDocumentMetadata, DomError> {
        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        Ok(metadata)
    }
    fn meta_metadata_mut(&mut self, document: Key) -> &mut DomDocumentMetadata {
        let DomNodeKind::Document(metadata) = &mut self.node_mut(document).kind else {
            unreachable!("checked Document");
        };
        metadata
    }
    fn meta_pragma(
        &mut self,
        key: Key,
        document: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(raw) = self.meta_attribute(key, Attr::HttpEquiv, cancel)? else {
            return Ok(());
        };
        let state = surgeist_html::bounded::meta_pragma_state_with(raw, || self.meta_step(cancel))?;
        let Some(state) = state else {
            return Ok(());
        };
        if matches!(
            state,
            Pragma::EncodingDeclaration | Pragma::SetCookie | Pragma::XUaCompatible
        ) {
            return Ok(());
        }
        // CSP's actual Head guard precedes even the content-attribute read.
        if state == Pragma::ContentSecurityPolicy {
            self.meta_step(cancel)?;
            let Some(parent) = self.node(key)?.parent else {
                return Ok(());
            };
            let DomNodeKind::Element(parent) = &self.node(parent)?.kind else {
                return Ok(());
            };
            if parent.common_state().html().is_none()
                || !self.meta_ascii(parent.name().local_name(), "head", false, cancel)?
            {
                return Ok(());
            }
        }
        let Some(content) = self.meta_attribute(key, Attr::Content, cancel)? else {
            return Ok(());
        };
        match state {
            Pragma::ContentLanguage => {
                let Some(range) =
                    surgeist_html::bounded::meta_language_token_with(content, || {
                        self.meta_step(cancel)
                    })?
                else {
                    return Ok(());
                };
                let old = self
                    .meta_metadata(document)?
                    .meta_state()
                    .ok_or(DomError::Meta(Failure::HistoryUnavailable))?
                    .pragma_language()
                    .map_or(0, |value| value.shared_code_units().capacity());
                let required = self
                    .units
                    .checked_sub(old)
                    .and_then(|units| units.checked_add(range.len()))
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(required, limits.units, DomResource::StringCodeUnits)?;
                let requested = range
                    .len()
                    .checked_mul(size_of::<u16>())
                    .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                    .ok_or(DomError::RepresentationExhausted)?;
                self.source_admit_units(range.len(), cancel)?;
                self.source_admit(requested, cancel)?;
                let formation = source::formation::Formation::new(self, cancel)?;
                let mut token = source::formation::utf16_buffer(
                    range.len(),
                    limits.units,
                    formation.as_ref(),
                    cancel,
                )?;
                let units = self
                    .units
                    .checked_sub(old)
                    .and_then(|n| n.checked_add(token.capacity()))
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(units, limits.units, DomResource::StringCodeUnits)?;
                for unit in &content.code_units()[range] {
                    self.meta_step(cancel)?;
                    #[cfg(test)]
                    token_observation::copy(self, token.capacity(), cancel);
                    token.push(*unit);
                }
                self.meta_metadata_mut(document)
                    .meta_state_mut()
                    .expect("checked history")
                    .pragma_language = Some(DocumentString::from_utf16(token));
                self.units = units;
                // The actual buffer has moved into canonical metadata. No
                // fallible action intervenes before releasing its local lease.
                drop(formation);
            }
            Pragma::Refresh => {
                if content.is_empty() {
                    return Ok(());
                }
                let history = self
                    .meta_metadata(document)?
                    .meta_state()
                    .ok_or(DomError::Meta(Failure::HistoryUnavailable))?;
                if history.will_declaratively_refresh() {
                    return Ok(());
                }
                let target = surgeist_html::bounded::meta_refresh_syntax_with(content, |_| {
                    self.meta_step(cancel)
                })?;
                match target {
                    None => (),
                    Some(surgeist_html::bounded::HtmlMetaRefreshSyntaxTarget::UrlString(range)) => {
                        let surgeist_html_semantics::HtmlEncodingUrlParseOutcome::Parsed(url) =
                            self.resolve_encoding_url_units(
                                document,
                                &content.code_units()[range],
                                cancel,
                            )?
                        else {
                            return Ok(());
                        };
                        if url.scheme() == surgeist_html_semantics::HtmlBaseUrlScheme::Javascript {
                            return Ok(());
                        }
                        self.meta_metadata_mut(document)
                            .meta_state_mut()
                            .expect("checked history")
                            .will_declaratively_refresh = true;
                    }
                    Some(surgeist_html::bounded::HtmlMetaRefreshSyntaxTarget::DocumentUrl) => {
                        // This branch selects the actual current Document URL;
                        // it does not parse URL text or reconstruct its history.
                        self.meta_metadata_mut(document)
                            .meta_state_mut()
                            .expect("checked history")
                            .will_declaratively_refresh = true;
                    }
                }
            }
            Pragma::DefaultStyle if !content.is_empty() => {
                return Err(DomError::Meta(Failure::DefaultStyleUnsupported));
            }
            Pragma::ContentSecurityPolicy if !content.is_empty() => {
                return Err(DomError::Meta(Failure::ContentSecurityPolicyUnsupported));
            }
            _ => (),
        }
        Ok(())
    }
    fn meta_referrer(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        let Some(document) = self.meta_document_tree(key, cancel)? else {
            return Ok(());
        };
        let Some(name) = self.meta_attribute(key, Attr::Name, cancel)? else {
            return Ok(());
        };
        if !self.meta_ascii(name, "referrer", true, cancel)? {
            return Ok(());
        }
        let Some(content) = self.meta_attribute(key, Attr::Content, cancel)? else {
            return Ok(());
        };
        if content.is_empty() {
            return Ok(());
        }
        if let Some(policy) =
            surgeist_html::bounded::meta_referrer_policy_with(content, || self.meta_step(cancel))?
        {
            let current = self
                .meta_metadata(document)?
                .policy_container()
                .referrer_policy()
                .ok_or(DomError::Meta(Failure::HistoryUnavailable))?;
            if current != policy {
                self.meta_metadata_mut(document)
                    .policy_container_mut()
                    .referrer = Some(policy);
            }
        }
        Ok(())
    }
    fn meta_candidates(
        &self,
        document: Key,
        theme: bool,
        scheme: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // These are two source algorithms, not an interleaved candidate query.
        // Complete theme selection/application before starting color-scheme.
        for theme_pass in [true, false] {
            if (theme_pass && !theme) || (!theme_pass && !scheme) {
                continue;
            }
            let expected = if theme_pass {
                "theme-color"
            } else {
                "color-scheme"
            };
            let mut next = Some(document);
            while let Some(key) = next {
                self.meta_step(cancel)?;
                if self.is_meta(key)?
                    && let Some(name) = self.meta_attribute(key, Attr::Name, cancel)?
                    && self.meta_ascii(name, expected, true, cancel)?
                    && self.meta_attribute(key, Attr::Content, cancel)?.is_some()
                {
                    return Err(DomError::Meta(if theme_pass {
                        Failure::ThemeColorUnsupported
                    } else {
                        Failure::ColorSchemeUnsupported
                    }));
                }
                next = crate::slot::next_admitted(self, document, key, &mut || {
                    self.meta_step(cancel)
                })?;
            }
            if theme_pass {
                self.meta_metadata(document)?.apply_no_meta_theme();
            }
        }
        Ok(())
    }
    pub(super) fn meta_inserted(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.is_meta(key)? {
            return Ok(());
        }
        let Some(document) = self.meta_document_tree(key, cancel)? else {
            return Ok(());
        };
        self.meta_pragma(key, document, limits, cancel)?;
        self.meta_referrer(key, cancel)?;
        self.meta_candidates(document, true, true, cancel)
    }
    pub(super) fn meta_removed(
        &self,
        key: Key,
        old_document: Option<Key>,
        _limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.is_meta(key)?
            && let Some(document) = old_document
        {
            self.meta_candidates(document, true, true, cancel)?;
        }
        Ok(())
    }
    pub(super) fn meta_attribute_changed(
        &mut self,
        key: Key,
        name: &crate::QualifiedName,
        _limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.is_meta(key)? || name.namespace().uri().is_some() {
            return Ok(());
        }
        let name_or_content = self.meta_ascii(name.local_name(), "name", false, cancel)?
            || self.meta_ascii(name.local_name(), "content", false, cancel)?;
        let media = self.meta_ascii(name.local_name(), "media", false, cancel)?;
        if name_or_content {
            self.meta_referrer(key, cancel)?;
        }
        if name_or_content || media {
            self.meta_candidates(self.node(key)?.document, true, name_or_content, cancel)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod source_observation {
    use super::*;
    use std::{cell::Cell, sync::atomic::Ordering};
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Read {
        pub required: usize,
        pub limit: usize,
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static LAST: Cell<Option<Read>> = const { Cell::new(None) };
    }
    pub(super) fn start() {
        ACTIVE.set(true);
        LAST.set(None);
    }
    pub(super) fn finish() -> Option<Read> {
        ACTIVE.set(false);
        LAST.take()
    }
    pub(super) fn retained(
        draft: &Draft,
        value: &DocumentString,
        owned: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !ACTIVE.get() {
            return Ok(());
        }
        let Some(limits) = draft.source_limits else {
            return Ok(());
        };
        let extra = if owned {
            size_of::<Vec<u16>>() + value.shared_code_units().capacity() * size_of::<u16>()
        } else {
            0
        };
        let read = Read {
            required: draft.source_storage_bytes(cancel)?
                + draft.source_other_bytes
                + draft.source_formation_bytes.load(Ordering::Relaxed)
                + extra,
            limit: limits.retained_bytes,
        };
        if LAST.get().is_none_or(|old| old.required < read.required) {
            LAST.set(Some(read));
        }
        Ok(())
    }
}
#[cfg(test)]
mod token_observation {
    use super::*;
    use std::{cell::Cell, sync::atomic::Ordering};
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Copy {
        pub count: usize,
        pub bytes: usize,
        pub units: usize,
        pub capacity: usize,
        cancel_after: Option<usize>,
    }
    thread_local! { static ACTIVE: Cell<Option<Copy>> = const { Cell::new(None) }; }
    pub(super) fn start(cancel_after: Option<usize>) {
        ACTIVE.set(Some(Copy {
            count: 0,
            bytes: 0,
            units: 0,
            capacity: 0,
            cancel_after,
        }));
    }
    pub(super) fn finish() -> Copy {
        ACTIVE.take().expect("started actual token copy")
    }
    pub(super) fn copy(draft: &Draft, capacity: usize, cancel: &AtomicBool) {
        if let Some(mut copied) = ACTIVE.get() {
            copied.count += 1;
            copied.bytes = draft.source_formation_bytes.load(Ordering::Relaxed);
            copied.units = draft.source_temporary_units.load(Ordering::Relaxed);
            copied.capacity = capacity;
            if copied.cancel_after == Some(copied.count) {
                cancel.store(true, Ordering::Release);
            }
            ACTIVE.set(Some(copied));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Namespace;
    fn policy() -> DomLimits {
        DomLimits::new(32, 1_000_000, 64, 0, 0, 8, 1_000_000)
            .unwrap()
            .with_meta_processing_work(1_000_000)
    }
    fn actual_source_meta_read(
        bytes: usize,
        require_complete: bool,
    ) -> Option<source_observation::Read> {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(policy()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let root = setup.create_element_ns(&document, Namespace::html(), "head");
        let meta = setup.create_element_ns(&document, Namespace::html(), "meta");
        let mut raw = Vec::with_capacity(65_536);
        raw.extend("referrer".encode_utf16());
        setup.set_attribute(&meta, "name", DocumentString::from_utf16(raw));
        setup.set_attribute(&meta, "content", "no-referrer");
        setup.insert_before(&root, &document, None);
        let commit = dom.apply(setup, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let root = commit.created(&root).unwrap().unwrap();
        let meta = commit.created(&meta).unwrap().unwrap();
        let before = dom.snapshot();
        let mut edit = DomBatch::new();
        edit.insert_before(meta, root, None);
        let limits = crate::DomSourceLimits::new(bytes, 4_000_000, 100_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(edit, limits, &cancel) else {
            return None;
        };
        source_observation::start();
        let result = lease.advance(&cancel);
        let read = source_observation::finish();
        if require_complete {
            assert!(matches!(result, Ok(crate::DomSourceProgress::Complete(_))));
            assert_eq!(
                lease
                    .snapshot()
                    .document(document)
                    .unwrap()
                    .metadata()
                    .policy_container()
                    .referrer_policy(),
                Some(surgeist_html::HtmlReferrerPolicy::NoReferrer)
            );
            assert_eq!(
                before
                    .document(document)
                    .unwrap()
                    .metadata()
                    .policy_container()
                    .referrer_policy(),
                Some(surgeist_html::HtmlReferrerPolicy::StrictOriginWhenCrossOrigin)
            );
        } else if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        read
    }
    fn source_budget(draft: &mut Draft, bytes: usize, units: usize) {
        let cancel = AtomicBool::new(false);
        draft.source_limits = Some(crate::DomSourceLimits::new(
            usize::MAX,
            usize::MAX,
            100_000_000,
            64,
            4096,
            4096,
            1,
        ));
        let base = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
        let base_units = draft.source_storage_units(&cancel).unwrap() + draft.source_other_units;
        draft.source_limits = Some(crate::DomSourceLimits::new(
            base + bytes,
            base_units + units,
            100_000_000,
            64,
            4096,
            4096,
            1,
        ));
    }
    fn language_draft() -> (Draft, Key, Key) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(policy()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let root = setup.create_element_ns(&document, Namespace::html(), "head");
        let meta = setup.create_element_ns(&document, Namespace::html(), "meta");
        setup.set_attribute(&meta, "http-equiv", "content-language");
        setup.set_attribute(&meta, "content", "old");
        setup.insert_before(&root, &document, None);
        setup.insert_before(&meta, &root, None);
        let commit = dom.apply(setup, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let meta = commit.created(&meta).unwrap().unwrap();
        let mut edit = DomBatch::new();
        edit.detach(meta);
        edit.set_attribute(
            meta,
            "content",
            DocumentString::from_utf16(vec![0x45, 0x4e, 0x2d, 0xd800]),
        );
        dom.apply(edit, &cancel).unwrap();
        let draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        (
            draft,
            Key::Existing(meta.slot),
            Key::Existing(document.node().slot),
        )
    }
    fn language(draft: &Draft, doc: Key) -> &[u16] {
        draft
            .meta_metadata(doc)
            .unwrap()
            .meta_state()
            .unwrap()
            .pragma_language()
            .unwrap()
            .code_units()
    }
    #[test]
    fn source_meta_language_keeps_exact_buffer_custody_through_copy_and_transfer() {
        use std::sync::atomic::Ordering;
        let cancel = AtomicBool::new(false);
        let bytes = size_of::<Vec<u16>>() + 4 * size_of::<u16>();
        let (mut draft, meta, doc) = language_draft();
        source_budget(&mut draft, bytes, 4);
        token_observation::start(None);
        draft.meta_pragma(meta, doc, policy(), &cancel).unwrap();
        let copied = token_observation::finish();
        assert_eq!(
            (copied.count, copied.capacity, copied.bytes, copied.units),
            (4, 4, bytes, 4)
        );
        assert_eq!(language(&draft, doc), [0x45, 0x4e, 0x2d, 0xd800]);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        // The exact same original ordinary branch keeps its values and tariff.
        let (mut ordinary, meta, doc) = language_draft();
        let initial_work = ordinary.meta_work.get();
        ordinary.meta_pragma(meta, doc, policy(), &cancel).unwrap();
        assert_eq!(language(&ordinary, doc), [0x45, 0x4e, 0x2d, 0xd800]);
        assert_eq!(
            ordinary.meta_work.get() - initial_work,
            draft.meta_work.get()
        );
    }
    #[test]
    fn source_meta_language_refuses_one_short_and_reclaims_cancelled_copy() {
        use std::sync::atomic::Ordering;
        let cancel = AtomicBool::new(false);
        let bytes = size_of::<Vec<u16>>() + 4 * size_of::<u16>();
        for (byte_extra, unit_extra, resource) in [
            (bytes - 1, 4, DomResource::SourceRetainedBytes),
            (bytes, 3, DomResource::SourceRetainedUnits),
        ] {
            let (mut draft, meta, doc) = language_draft();
            source_budget(&mut draft, byte_extra, unit_extra);
            assert!(
                matches!(draft.meta_pragma(meta, doc, policy(), &cancel), Err(DomError::ResourceLimit { resource: actual, .. }) if actual == resource)
            );
            assert_eq!(
                language(&draft, doc),
                "old".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        }
        let (mut draft, meta, doc) = language_draft();
        source_budget(&mut draft, bytes, 4);
        let before_work = draft.source_work.load(Ordering::Relaxed);
        token_observation::start(Some(2));
        assert!(matches!(
            draft.meta_pragma(meta, doc, policy(), &cancel),
            Err(DomError::Cancelled)
        ));
        let copied = token_observation::finish();
        assert_eq!((copied.count, copied.bytes, copied.units), (2, bytes, 4));
        assert_eq!(
            language(&draft, doc),
            "old".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        let spent = draft.source_work.load(Ordering::Relaxed);
        assert!(spent > before_work);
        cancel.store(false, Ordering::Release);
        draft.meta_pragma(meta, doc, policy(), &cancel).unwrap();
        assert_eq!(language(&draft, doc), [0x45, 0x4e, 0x2d, 0xd800]);
        assert!(draft.source_work.load(Ordering::Relaxed) > spent);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    }
    #[test]
    fn source_meta_read_admits_actual_padded_raw_occurrence_before_retaining() {
        let high = actual_source_meta_read(16_000_000, true).expect("actual Meta raw read");
        if let Some(actual) = actual_source_meta_read(high.required - 1, false) {
            assert!(
                actual.required <= actual.limit,
                "Meta raw local retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
}
