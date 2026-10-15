//! Qualified native facts and source-time first-Base changes, without a URL engine.
use super::*;
use crate::base::{DomBaseFactsError as Failure, Frozen, Response};
use surgeist_html::{HtmlScriptResult, KnownHtmlAttribute};
use surgeist_html_semantics::{
    HtmlAboutBaseUrl, HtmlBaseMissing, HtmlBaseUrlScheme, HtmlParseBaseFacts,
};

fn failure(error: Failure) -> DomError {
    DomError::BaseFacts(error)
}
impl Draft {
    fn base_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let next = self
            .base_work
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.base_work_limit {
            return Err(DomError::BaseWorkLimit {
                limit: self.base_work_limit,
            });
        }
        self.base_work.set(next);
        Ok(())
    }
    fn base_equal(
        &self,
        a: &DocumentString,
        b: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.base_step(cancel)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.code_units().iter().zip(b.code_units()) {
            self.base_step(cancel)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn base_ascii(
        &self,
        a: &DocumentString,
        b: &str,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.base_step(cancel)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.code_units().iter().zip(b.bytes()) {
            self.base_step(cancel)?;
            if *a != u16::from(b) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn base_attribute(
        &self,
        key: Key,
        name: KnownHtmlAttribute,
        cancel: &AtomicBool,
    ) -> Result<Option<DocumentString>, DomError> {
        let DomNodeKind::Element(e) = &self.node(key)?.kind else {
            return Ok(None);
        };
        for attr in e.attributes().iter() {
            self.base_step(cancel)?;
            if attr.name().namespace().uri().is_none()
                && self.base_ascii(attr.name().local_name(), name.local_name(), cancel)?
            {
                return Ok(Some(attr.value().clone()));
            }
        }
        Ok(None)
    }
    pub(super) fn bind_base_facts(
        &mut self,
        basis: crate::DomDocumentStamp,
        facts: HtmlParseBaseFacts,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if basis.store().owner() != self.store {
            return Err(DomError::ForeignOwner);
        }
        if basis.store().revision() != self.basis.revision() {
            return Err(failure(Failure::BasisRevisionMismatch));
        }
        let key = self.resolve(basis.document().into())?;
        let DomNodeKind::Document(metadata) = &self.node(key)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        self.base_step(cancel)?;
        let env = facts.document();
        let url = env
            .document_url()
            .ok_or_else(|| failure(Failure::Missing(HtmlBaseMissing::DocumentUrl)))?;
        let encoding = env
            .encoding()
            .ok_or_else(|| failure(Failure::Missing(HtmlBaseMissing::CharacterEncoding)))?;
        if env.iframe_srcdoc()
            || !matches!(env.about_base_url(), HtmlAboutBaseUrl::KnownNull)
            || metadata.native_environment().has_ever_had_navigable()
            || !self.base_ascii(metadata.encoding(), "UTF-8", cancel)?
            || !self.base_ascii(metadata.url(), "about:blank", cancel)?
            || !url.matches_about_blank()
            || url.scheme() != HtmlBaseUrlScheme::Other
            || !self.base_equal(metadata.url(), url.serialization(), cancel)?
        {
            return Err(failure(Failure::EnvironmentMismatch));
        }
        let fallback = env.fallback().map_err(|e| failure(Failure::Missing(e)))?;
        if let Some(previous) = self
            .node(key)?
            .base_document
            .as_ref()
            .and_then(|d| d.facts.as_ref())
        {
            self.base_step(cancel)?;
            if previous.document().revision().producer() != env.revision().producer() {
                return Err(failure(Failure::ForeignProducer));
            }
            if previous.document().revision() == env.revision()
                && (previous.document().encoding() != Some(encoding)
                    || previous.document().fallback().ok() != Some(fallback))
            {
                return Err(failure(Failure::InputsMismatch));
            }
        }
        for observation in facts.observations() {
            self.base_step(cancel)?;
            if !observation.inputs_match(fallback, encoding, env.revision()) {
                return Err(failure(Failure::InputsMismatch));
            }
        }
        let mut records = facts.formation_usage().record_capacity();
        for (i, node) in self.existing.iter().enumerate() {
            self.base_step(cancel)?;
            if key != Key::Existing(i)
                && let Some(document) = node.as_ref().and_then(|n| n.base_document.as_ref())
            {
                records = records
                    .checked_add(document.records())
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        for (i, node) in self.created.iter().enumerate() {
            self.base_step(cancel)?;
            if key != Key::Created(i)
                && let Some(document) = node.as_ref().and_then(|n| n.base_document.as_ref())
            {
                records = records
                    .checked_add(document.records())
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        admit(
            records,
            limits.base_observations,
            DomResource::BaseObservations,
        )?;
        let old = self
            .node(key)?
            .base_document
            .as_ref()
            .expect("native Document base state")
            .units();
        let units = (self.units - old)
            .checked_add(facts.formation_usage().utf16_extent())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        self.node_mut(key)
            .base_document
            .as_mut()
            .expect("Document")
            .facts = Some(facts);
        self.units = units;
        Ok(())
    }
    fn first_base(&self, document: Key, cancel: &AtomicBool) -> Result<Option<Key>, DomError> {
        let mut next = Some(document);
        while let Some(key) = next {
            self.base_step(cancel)?;
            if self.node(key)?.base.is_some()
                && self
                    .base_attribute(key, KnownHtmlAttribute::Href, cancel)?
                    .is_some()
            {
                return Ok(Some(key));
            }
            next = crate::slot::next_admitted(self, document, key, &mut || self.base_step(cancel))?;
        }
        Ok(None)
    }
    fn base_root(&self, mut key: Key, cancel: &AtomicBool) -> Result<Key, DomError> {
        loop {
            self.base_step(cancel)?;
            match self.node(key)?.parent {
                Some(p) => key = p,
                None => return Ok(key),
            }
        }
    }
    pub(super) fn base_tree_change(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.native_link_guard(key, cancel)?;
        if self.node(key)?.base.is_none() {
            return Ok(());
        }
        let document = self.node(key)?.document;
        self.update_first_base(document, None, limits, cancel)
    }
    pub(super) fn base_attribute_change(
        &mut self,
        key: Key,
        name: &crate::QualifiedName,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.native_link_guard(key, cancel)?;
        // HTML877d18848 id/anchor-name hooks iterate the actual render-blocking
        // set, not all Links. Native Documents start empty and no admitted front
        // enters it. A resource-bearing retained set cannot use this proof.
        if name.namespace().uri().is_none()
            && (name.local_name().code_units() == [105, 100]
                || (name.local_name().code_units() == [110, 97, 109, 101]
                    && matches!(&self.node(key)?.kind,DomNodeKind::Element(e) if e.common_state().html().is_some() && e.name().local_name().code_units()==[97])))
        {
            let doc = self
                .node(self.node(key)?.document)?
                .base_document
                .as_ref()
                .ok_or(DomError::WrongNodeKind)?;
            if !doc.render_blocking.is_empty() {
                return Err(DomError::LinkProcessingUnavailable);
            }
        }
        if let Some(link) = &mut self.node_mut(key).link
            && name.local_name().code_units() == [100, 105, 115, 97, 98, 108, 101, 100]
        {
            link.flags
                .disabled_attribute_changed(name.namespace(), old, new);
        }
        if self.node(key)?.base.is_some()
            && name.namespace().uri().is_none()
            && name.local_name().code_units() == [104, 114, 101, 102]
        {
            let document = self.node(key)?.document;
            if self.base_root(key, cancel)? == document {
                self.update_first_base(document, Some(key), limits, cancel)?;
            }
        }
        Ok(())
    }
    fn native_link_guard(&self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        let Some(link) = self.node(key)?.link.as_ref() else {
            return Ok(());
        };
        check_cancel(cancel)?;
        let doc = self.node(self.node(key)?.document)?;
        let DomNodeKind::Document(metadata) = &doc.kind else {
            return Err(DomError::WrongNodeKind);
        };
        // Native creation, clone/import and native authoring installation all
        // establish fresh-null associated CSS sheet. No command/host observation
        // installs one; VT CSSOM identities establish no Link sheet association.
        // Thus disabled-set has no associated sheet to disable (HTML877d19799).
        if link.sheet.is_some()
            || metadata.native_environment().browsing_context()
                != surgeist_html::HtmlBrowsingContextPresence::Absent
        {
            return Err(DomError::LinkProcessingUnavailable);
        }
        // Stylesheet/modulepreload/preload/prefetch/preconnect/dns-prefetch and
        // expect creation/change/insertion triggers require BC-connectedness.
        // Native absence discharges those exact guards, not all Link models.
        // Icon/manifest/feed/search are qualified optional services; none is an
        // admitted native recipient. Pingback's external processing is not
        // implemented or advertised as a supported token by this canonical owner.
        Ok(())
    }
    fn update_first_base(
        &mut self,
        document: Key,
        changed_href: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let first = self.first_base(document, cancel)?;
        let state = self
            .node(document)?
            .base_document
            .as_ref()
            .ok_or(DomError::WrongNodeKind)?;
        if first == state.first && !(first.is_some() && first == changed_href) {
            return Ok(());
        }
        let Some(first) = first else {
            self.node_mut(document)
                .base_document
                .as_mut()
                .expect("Document")
                .first = None;
            return Ok(());
        };
        let facts = state
            .facts
            .clone()
            .ok_or_else(|| failure(Failure::Missing(HtmlBaseMissing::DocumentFacts)))?;
        let env = facts.document();
        let fallback = env.fallback().map_err(|e| failure(Failure::Missing(e)))?;
        let encoding = env
            .encoding()
            .ok_or_else(|| failure(Failure::Missing(HtmlBaseMissing::CharacterEncoding)))?;
        let raw = self
            .base_attribute(first, KnownHtmlAttribute::Href, cancel)?
            .expect("selected href");
        let mut outcome = None;
        for observation in facts.observations() {
            self.base_step(cancel)?;
            if observation.inputs_match(fallback, encoding, env.revision())
                && self.base_equal(observation.raw(), &raw, cancel)?
            {
                outcome = Some(observation.outcome());
                break;
            }
        }
        let outcome =
            outcome.ok_or_else(|| failure(Failure::Missing(HtmlBaseMissing::ParseObservation)))?;
        self.base_step(cancel)?;
        let (value, respond) = surgeist_html_semantics::select_frozen_base(fallback, outcome)
            .map_err(|e| failure(Failure::Missing(e)))?;
        let old = self.node(first)?.base.as_ref().expect("Base").units();
        let units = (self.units - old)
            .checked_add(value.serialization().len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        self.node_mut(first).base.as_mut().expect("Base").frozen = Some(Frozen {
            value: value.clone(),
            document,
            revision: env.revision().clone(),
            response: if respond {
                Response::Pending
            } else {
                Response::Returned
            },
        });
        self.node_mut(document)
            .base_document
            .as_mut()
            .expect("Document")
            .first = Some(first);
        self.units = units;
        // Real assignment precedes response. Any cancellation/refusal below
        // leaves this private candidate unpublished.
        if respond {
            self.respond_to_base_change(document, cancel)?;
            self.node_mut(first)
                .base
                .as_mut()
                .expect("Base")
                .frozen
                .as_mut()
                .expect("assigned")
                .response = Response::NativeFactsPublished;
        }
        Ok(())
    }
    /// Stackless shadow-inclusive order, without Template ownership traversal.
    fn base_next_shadow(
        &self,
        boundary: Key,
        mut key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.base_step(cancel)?;
        let node = self.node(key)?;
        if let Some(shadow) = node.shadow_root {
            return Ok(Some(shadow));
        }
        if let Some(child) = node.children.first() {
            return Ok(Some(*child));
        }
        loop {
            self.base_step(cancel)?;
            if key == boundary {
                return Ok(None);
            }
            let node = self.node(key)?;
            if let Some(parent) = node.parent {
                for (i, sibling) in self.node(parent)?.children.iter().enumerate() {
                    self.base_step(cancel)?;
                    if *sibling == key {
                        if let Some(next) = self.node(parent)?.children.get(i + 1) {
                            return Ok(Some(*next));
                        }
                        break;
                    }
                }
                key = parent;
            } else if let Some(host) = node.host {
                if let Some(child) = self.node(host)?.children.first() {
                    return Ok(Some(*child));
                }
                key = host;
            } else {
                return Err(failure(Failure::Missing(HtmlBaseMissing::ResponseDomain)));
            }
        }
    }
    fn respond_to_base_change(&self, document: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        self.base_step(cancel)?;
        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        // Supported commands/host captures admit canonical values and selection
        // observations only: no UI recipient, selector/style executor or result
        // callback is installed. Canonical DocumentBase invalidation is the
        // actual downstream response publication, not delivery certification.
        let mut next = Some(document);
        while let Some(key) = next {
            self.base_step(cancel)?;
            if let Some(script) = self.node(key)?.script {
                match script.result {
                    HtmlScriptResult::Uninitialized => {}
                }
            }
            next = self.base_next_shadow(document, key, cancel)?;
        }
        // HTML877d76574 returns before flag/task access when node navigable is
        // not top-level. This actual native environment has never been an
        // active Document of any navigable; its node navigable is null.
        if metadata.native_environment().has_ever_had_navigable() {
            return Err(failure(Failure::Missing(HtmlBaseMissing::ResponseDomain)));
        }
        check_cancel(cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Namespace;
    use surgeist_html_semantics::*;
    fn policy(work: u64) -> DomLimits {
        DomLimits::new(64, 32768, 256, 128, 64, 8, 131072)
            .unwrap()
            .with_base_processing(64, work)
            .unwrap()
    }
    fn fixture() -> (Dom, DocumentId, NodeId) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(policy(1_000_000)).unwrap();
        let mut b = DomBatch::new();
        let doc = b.create_document();
        let root = b.create_element_ns(&doc, Namespace::html(), "div");
        b.insert_before(&root, &doc, None);
        let base = b.create_element_ns(&doc, Namespace::html(), "base");
        let c = dom.apply(b, &cancel).unwrap();
        let doc = c.document(&doc).unwrap();
        let root = c.created(&root).unwrap().unwrap();
        let base = c.created(&base).unwrap().unwrap();
        let r = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let encoding = HtmlBaseEncoding::fresh();
        let fallback = HtmlBaseUrlRecord::observed(
            r.clone(),
            "about:blank".into(),
            HtmlBaseUrlScheme::Other,
            true,
        )
        .unwrap();
        let env = HtmlBaseDocumentFacts::new(
            r.clone(),
            Some(fallback.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap();
        let mut observations = Vec::new();
        for raw in ["target", "next"] {
            let record = HtmlBaseUrlRecord::observed(
                r.clone(),
                DocumentString::from(format!("https://example.test/{raw}")),
                HtmlBaseUrlScheme::Other,
                false,
            )
            .unwrap();
            observations.push(
                HtmlBaseParseObservation::new(
                    raw.into(),
                    fallback.clone(),
                    encoding.clone(),
                    r.clone(),
                    HtmlBaseParseOutcome::Parsed {
                        record,
                        base_allowed: HtmlBaseAllowed::Allowed,
                    },
                )
                .unwrap(),
            );
        }
        let facts =
            HtmlParseBaseFacts::new(env, observations, HtmlBaseFactsLimits::new(8, 4096, 10000))
                .unwrap();
        let mut b = DomBatch::new();
        b.bind_native_base_facts(dom.snapshot().document(doc).unwrap().stamp(), &facts);
        b.set_attribute(base, "href", "target");
        b.insert_before(base, root, None);
        dom.apply(b, &cancel).unwrap();
        (dom, doc, base)
    }
    fn edit(base: NodeId, repeats: usize, value: &str) -> DomBatch {
        let mut b = DomBatch::new();
        for _ in 0..repeats {
            b.set_attribute(base, "href", value);
        }
        b
    }
    fn usage(dom: &Dom, b: DomBatch) -> u64 {
        let cancel = AtomicBool::new(false);
        let mut d = Draft::new(dom, b.owner, &cancel).unwrap();
        for cmd in b.commands {
            d.apply(cmd, dom.limits, &cancel).unwrap();
        }
        d.base_work.get()
    }
    #[test]
    fn exact_native_work_is_cumulative_across_repeated_public_refreezes() {
        let (mut dom, _, base) = fixture();
        let cancel = AtomicBool::new(false);
        let one = usage(&dom, edit(base, 1, "target"));
        let two = usage(&dom, edit(base, 2, "target"));
        assert_eq!(two, one * 2);
        assert!(one > 0);
        dom.set_limits(policy(one)).unwrap();
        let before = dom.snapshot();
        dom.apply(edit(base, 1, "target"), &cancel).unwrap();
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.apply(edit(base, 2, "target"), &cancel).unwrap_err(),
            DomError::BaseWorkLimit { limit: one }
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        dom.set_limits(policy(two)).unwrap();
        dom.apply(edit(base, 2, "target"), &cancel).unwrap();
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    #[test]
    fn response_one_short_keeps_real_assignment_private_and_pending_cannot_publish() {
        let (mut dom, doc, base) = fixture();
        let cancel = AtomicBool::new(false);
        let exact = usage(&dom, edit(base, 1, "next"));
        dom.set_limits(policy(exact - 1)).unwrap();
        let before = dom.snapshot();
        let b = edit(base, 1, "next");
        let mut d = Draft::new(&dom, b.owner, &cancel).unwrap();
        assert_eq!(
            d.apply(b.commands.into_iter().next().unwrap(), dom.limits, &cancel)
                .unwrap_err(),
            DomError::BaseWorkLimit { limit: exact - 1 }
        );
        let key = d.resolve(base.into()).unwrap();
        let frozen = d
            .node(key)
            .unwrap()
            .base
            .as_ref()
            .unwrap()
            .frozen
            .as_ref()
            .unwrap();
        assert_eq!(frozen.response, Response::Pending);
        assert_eq!(frozen.document, d.resolve(doc.into()).unwrap());
        assert_eq!(
            d.finish(&dom, &cancel).err().unwrap(),
            failure(Failure::Missing(HtmlBaseMissing::ResponseDomain))
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        dom.set_limits(policy(exact)).unwrap();
        let prepared = dom.prepare(edit(base, 1, "next"), &cancel).unwrap();
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    #[test]
    fn revision_refusal_preserves_frozen_history_and_current_binding() {
        let (mut dom, doc, base) = fixture();
        let cancel = AtomicBool::new(false);
        dom.revision = DomRevision(u64::MAX);
        let before = dom.snapshot();
        assert_eq!(
            dom.apply(edit(base, 1, "next"), &cancel).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.snapshot().document(doc).unwrap().first_base(),
            Some(base)
        );
        assert_eq!(
            dom.snapshot()
                .node(base)
                .unwrap()
                .as_base()
                .unwrap()
                .frozen()
                .unwrap()
                .value(),
            before
                .node(base)
                .unwrap()
                .as_base()
                .unwrap()
                .frozen()
                .unwrap()
                .value()
        );
    }
    #[test]
    fn native_link_and_id_hooks_refuse_resource_bearing_state_instead_of_closing_it() {
        let (dom, doc, base) = fixture();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        b.set_attribute(base, "id", "target");
        let mut d = Draft::new(&dom, b.owner, &cancel).unwrap();
        let document = d.resolve(doc.into()).unwrap();
        let key = d.resolve(base.into()).unwrap();
        d.node_mut(document)
            .base_document
            .as_mut()
            .unwrap()
            .render_blocking
            .push(key);
        assert_eq!(
            d.apply(b.commands.into_iter().next().unwrap(), dom.limits, &cancel)
                .unwrap_err(),
            DomError::LinkProcessingUnavailable
        );
        // This is an owner-private unavailable-phase guard, not an admitted
        // resource installation or a fabricated public resource fixture.
    }
    #[test]
    fn private_positive_freeze_and_retirement_can_finish_as_an_exact_canonical_noop() {
        let (mut dom, doc, base) = fixture();
        let cancel = AtomicBool::new(false);
        let parent = dom.snapshot().node(base).unwrap().parent().unwrap();
        let before = dom.snapshot();
        let units = dom.usage().string_code_units();
        let mut b = DomBatch::new();
        let private = b.create_element_ns(doc, Namespace::html(), "base");
        b.set_attribute(&private, "href", "next");
        b.insert_before(&private, parent, Some(base.into()));
        b.detach(&private);
        b.destroy(&private);
        let c = dom.apply(b, &cancel).unwrap();
        assert_eq!(c.created(&private).unwrap(), None);
        assert!(c.changes().changes().is_empty());
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage().string_code_units(), units);
        assert_eq!(
            dom.snapshot().document(doc).unwrap().first_base(),
            Some(base)
        );
    }
}
