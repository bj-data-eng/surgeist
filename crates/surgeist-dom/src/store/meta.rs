//! Native Meta source hooks against actual staged Document history.
use super::*;
use crate::DomMetaError as Failure;
use surgeist_html::{HtmlMetaPragmaState as Pragma, KnownHtmlAttribute as Attr};
mod color;
mod scheme;
#[cfg(test)]
mod tests_css;

impl Draft {
    pub(super) fn meta_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        if let Some(recording) = &self.meta_recording {
            recording.envelope.local_step(cancel)?;
        }
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
        Ok(
            matches!(&self.node(key)?.kind(), transaction::KindRead::Element(element) if element.is_meta()),
        )
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
        let transaction::KindRead::Element(element) = &self.node(key)?.kind() else {
            return Ok(None);
        };
        for entry in element.language().attributes().iter() {
            self.meta_step(cancel)?;
            if entry.name().namespace_uri().is_none()
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
            match node.parent() {
                Some(parent) => key = parent,
                None => {
                    return Ok(
                        matches!(node.kind(), transaction::KindRead::Document(_)).then_some(key)
                    );
                }
            }
        }
    }
    fn with_meta_metadata<R>(
        &mut self,
        document: Key,
        cancel: &AtomicBool,
        edit: impl FnOnce(&mut DomDocumentMetadata) -> R,
    ) -> Result<R, DomError> {
        let mut metadata = self.document_metadata_mut(document, cancel)?;
        let result = edit(&mut metadata);
        metadata.commit(cancel)?;
        Ok(result)
    }
    fn meta_pragma(
        &mut self,
        key: Key,
        document: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        #[cfg(test)]
        transaction::queues::admission_observation::before_payload(self);
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
            let Some(parent) = self.node(key)?.parent() else {
                return Ok(());
            };
            let transaction::KindRead::Element(parent) = &self.node(parent)?.kind() else {
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
                    .document_metadata(document)?
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
                #[cfg(test)]
                let mut token_writes = crate::store::CopyObservation::default();
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
                    #[cfg(test)]
                    token_writes.pushed::<u16>();
                }
                #[cfg(test)]
                transaction::queues::admission_observation::after_payload(self);
                let mut metadata = self.document_metadata_mut(document, cancel)?;
                metadata
                    .meta_state_mut()
                    .expect("checked history")
                    .pragma_language = Some(DocumentString::from_utf16(token));
                metadata.commit_formed(formation, cancel)?;
                self.units = units;
            }
            Pragma::Refresh => {
                if content.is_empty() {
                    return Ok(());
                }
                let history = self
                    .document_metadata(document)?
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
                        self.with_meta_metadata(document, cancel, |metadata| {
                            metadata
                                .meta_state_mut()
                                .expect("checked history")
                                .will_declaratively_refresh = true
                        })?;
                    }
                    Some(surgeist_html::bounded::HtmlMetaRefreshSyntaxTarget::DocumentUrl) => {
                        // This branch selects the actual current Document URL;
                        // it does not parse URL text or reconstruct its history.
                        self.with_meta_metadata(document, cancel, |metadata| {
                            metadata
                                .meta_state_mut()
                                .expect("checked history")
                                .will_declaratively_refresh = true
                        })?;
                    }
                }
            }
            Pragma::DefaultStyle if !content.is_empty() => {
                self.guard_pending_associations(document, cancel)?;
                if self.meta_recording.is_some() {
                    if !matches!(document, Key::Existing(_)) {
                        return Err(DomError::Meta(Failure::DefaultStyleUnsupported));
                    }
                    self.admit_meta_content(content, cancel)?;
                    let content = content.clone();
                    self.record_meta_preferred(document, key, content, cancel)?;
                } else {
                    return Err(DomError::Meta(Failure::DefaultStyleUnsupported));
                }
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
                .document_metadata(document)?
                .policy_container()
                .referrer_policy()
                .ok_or(DomError::Meta(Failure::HistoryUnavailable))?;
            if current != policy {
                self.with_meta_metadata(document, cancel, |metadata| {
                    metadata.policy_container_mut().referrer = Some(policy)
                })?;
            }
        }
        Ok(())
    }
    fn meta_candidates(
        &mut self,
        document: Key,
        theme: bool,
        scheme: bool,
        limits: DomLimits,
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
            let mut selected_theme = None;
            let mut selected_scheme = None;
            while let Some(key) = next {
                self.meta_step(cancel)?;
                if self.is_meta(key)?
                    && let Some(name) = self.meta_attribute(key, Attr::Name, cancel)?
                    && self.meta_ascii(name, expected, true, cancel)?
                    && let Some(content) = self.meta_attribute(key, Attr::Content, cancel)?
                {
                    if theme_pass {
                        if let Some(media) = self.meta_attribute(key, Attr::Media, cancel)? {
                            // HTML's media foundation defines empty and solely
                            // ASCII-whitespace input to match without facts.
                            // Any other input still needs its actual matcher.
                            self.meta_step(cancel)?;
                            for unit in media.code_units() {
                                self.meta_step(cancel)?;
                                if !surgeist_html::bounded::is_ascii_whitespace(*unit) {
                                    return Err(DomError::Meta(
                                        Failure::ThemeColorMediaUnavailable,
                                    ));
                                }
                            }
                        }
                        if let Some(color) = color::parse(self, content, limits, cancel)? {
                            selected_theme = Some(color);
                            break;
                        }
                    } else {
                        let candidate = scheme::parse(self, document, content, limits, cancel)?;
                        self.meta_step(cancel)?;
                        if candidate.selected.report.result().is_ok() {
                            selected_scheme = Some(candidate);
                            break;
                        }
                    }
                }
                next = crate::slot::next_admitted(self, document, key, &mut || {
                    self.meta_step(cancel)
                })?;
            }
            if theme_pass {
                // A fixed actual Color comparison has no parse/source identity.
                for _ in 0..size_of::<crate::DomMetaThemeColorState>() {
                    self.meta_step(cancel)?;
                }
                self.with_meta_metadata(document, cancel, |metadata| {
                    metadata.meta_theme_color = crate::DomMetaThemeColorState::Known(selected_theme)
                })?;
                self.document_metadata(document)?.apply_meta_theme();
            } else {
                let old = self
                    .document_metadata(document)?
                    .meta_color_scheme
                    .as_ref()
                    .and_then(Option::as_ref);
                let equivalent = match (old, selected_scheme.as_ref()) {
                    (Some(old), Some(new)) => old
                        .report
                        .equivalent_with(&new.selected.report, || self.meta_step(cancel))
                        .map_err(scheme::payload_error)?,
                    (None, None) => self
                        .document_metadata(document)?
                        .meta_color_scheme
                        .is_some(),
                    _ => false,
                };
                if !equivalent {
                    let old_bytes = old.map_or(0, |old| old.bytes);
                    let new_bytes = selected_scheme.as_ref().map_or(0, |new| new.selected.bytes);
                    let retained = self.counts.meta_color_scheme_bytes;
                    let retained = retained
                        .checked_sub(old_bytes)
                        .and_then(|n| n.checked_add(new_bytes))
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(
                        retained,
                        limits.meta_color_scheme_bytes,
                        DomResource::MetaColorSchemeRetainedBytes,
                    )?;
                    let old_units = old.map_or(0, |old| old.units);
                    let new_units = selected_scheme.as_ref().map_or(0, |new| new.selected.units);
                    let units = self
                        .units
                        .checked_sub(old_units)
                        .and_then(|units| units.checked_add(new_units))
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(units, limits.units, DomResource::StringCodeUnits)?;
                    // Move the admitted report into canonical custody before
                    // its temporary formation debit is released.
                    if let Some(candidate) = selected_scheme {
                        self.with_meta_metadata(document, cancel, |metadata| {
                            metadata.meta_color_scheme = Some(Some(candidate.selected))
                        })?;
                        self.units = units;
                        drop(candidate.formation);
                    } else {
                        self.with_meta_metadata(document, cancel, |metadata| {
                            metadata.meta_color_scheme = Some(None)
                        })?;
                        self.units = units;
                    }
                }
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
        self.meta_candidates(document, true, true, limits, cancel)
    }
    pub(super) fn meta_removed(
        &mut self,
        key: Key,
        old_document: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.is_meta(key)?
            && let Some(document) = old_document
        {
            self.meta_candidates(document, true, true, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn meta_attribute_changed(
        &mut self,
        key: Key,
        name: &crate::DomQualifiedName,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.is_meta(key)? || name.namespace_uri().is_some() {
            return Ok(());
        }
        let name_or_content = self.meta_ascii(name.local_name(), "name", false, cancel)?
            || self.meta_ascii(name.local_name(), "content", false, cancel)?;
        let media = self.meta_ascii(name.local_name(), "media", false, cancel)?;
        if name_or_content {
            self.meta_referrer(key, cancel)?;
        }
        if name_or_content || media {
            self.meta_candidates(
                self.node(key)?.document(),
                true,
                name_or_content,
                limits,
                cancel,
            )?;
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
    pub(super) fn policy() -> DomLimits {
        DomLimits::new(
            32,
            1_000_000,
            64,
            0,
            0,
            8,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
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
    pub(super) fn source_budget(draft: &mut Draft, bytes: usize, units: usize) {
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
    struct Fixture {
        draft: Draft,
        _dom: Dom,
    }
    impl std::ops::Deref for Fixture {
        type Target = Draft;
        fn deref(&self) -> &Draft {
            &self.draft
        }
    }
    impl std::ops::DerefMut for Fixture {
        fn deref_mut(&mut self) -> &mut Draft {
            &mut self.draft
        }
    }
    fn language_draft() -> (Fixture, Key, Key) {
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
            Fixture { draft, _dom: dom },
            Key::Existing(meta),
            Key::Existing(document.node()),
        )
    }
    fn language(draft: &Draft, doc: Key) -> &[u16] {
        draft
            .document_metadata(doc)
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
        // Select only the original raw token allocation. Its finite parent
        // allowance resumes after that copy for the separate field barrier.
        for resource in [
            DomResource::SourceRetainedBytes,
            DomResource::SourceRetainedUnits,
        ] {
            let (mut draft, meta, doc) = language_draft();
            source_budget(&mut draft, 1_000_000, 1_000_000);
            let boundary = transaction::queues::admission_observation::Binding::payload(
                custody::Usage::new(bytes, 4),
                resource,
                false,
            );
            token_observation::start(None);
            draft.meta_pragma(meta, doc, policy(), &cancel).unwrap();
            let copied = token_observation::finish();
            boundary.limit();
            drop(boundary);
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
    }
    #[test]
    fn source_meta_language_refuses_one_short_and_reclaims_cancelled_copy() {
        use std::sync::atomic::Ordering;
        let cancel = AtomicBool::new(false);
        let bytes = size_of::<Vec<u16>>() + 4 * size_of::<u16>();
        for resource in [
            DomResource::SourceRetainedBytes,
            DomResource::SourceRetainedUnits,
        ] {
            let (mut draft, meta, doc) = language_draft();
            source_budget(&mut draft, 1_000_000, 1_000_000);
            let boundary = transaction::queues::admission_observation::Binding::payload(
                custody::Usage::new(bytes, 4),
                resource,
                true,
            );
            let result = draft.meta_pragma(meta, doc, policy(), &cancel);
            assert_eq!(
                result,
                Err(DomError::ResourceLimit {
                    resource,
                    limit: boundary.limit()
                })
            );
            drop(boundary);
            assert_eq!(
                language(&draft, doc),
                "old".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        }
        let (mut draft, meta, doc) = language_draft();
        source_budget(&mut draft, 1_000_000, 1_000_000);
        let parent_policy = draft.source_limits;
        let boundary = transaction::queues::admission_observation::Binding::payload(
            custody::Usage::new(bytes, 4),
            DomResource::SourceRetainedBytes,
            false,
        );
        let before_work = draft.source_work.load(Ordering::Relaxed);
        token_observation::start(Some(2));
        assert!(matches!(
            draft.meta_pragma(meta, doc, policy(), &cancel),
            Err(DomError::Cancelled)
        ));
        let copied = token_observation::finish();
        boundary.limit();
        drop(boundary);
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
        draft.source_limits = parent_policy;
        let boundary = transaction::queues::admission_observation::Binding::payload(
            custody::Usage::new(bytes, 4),
            DomResource::SourceRetainedBytes,
            false,
        );
        draft.meta_pragma(meta, doc, policy(), &cancel).unwrap();
        boundary.limit();
        drop(boundary);
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
