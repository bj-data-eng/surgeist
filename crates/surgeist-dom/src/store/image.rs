//! Source-time Img updates and Picture/Source fanout over actual canonical keys.
use super::*;
use surgeist_html::{
    HtmlImageAttributeChange, HtmlImageAttributeEffect, HtmlImageDocumentActivity,
    HtmlImageMutationTargets, HtmlImageTreeChange, HtmlImageUpdateFlags,
    HtmlImageUpdateStartDecision, HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement,
};

#[derive(Clone, Copy)]
pub(super) struct Cancelled {
    pub update: crate::DomImageUpdateId,
    pub flags: HtmlImageUpdateFlags,
}

impl Draft {
    pub(super) fn start_image_update(
        &mut self,
        key: Key,
        flags: HtmlImageUpdateFlags,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.effect_step(cancel)?;
        let node = self.node(key)?;
        let image = node.image.as_ref().expect("actual Img");
        let invocation = image
            .high_water
            .checked_add(1)
            .ok_or(DomError::ImageUpdateExhausted)?;
        // The actual current node Document owns activity independently of the
        // later images-disabled decision. Retired hosted and closed Documents
        // take the same original inactive wait; unavailable authority refuses.
        let activity = if self.document_fully_active(node.document, cancel)? {
            HtmlImageDocumentActivity::FullyActive
        } else {
            HtmlImageDocumentActivity::NotFullyActive
        };
        if surgeist_html::image_update_start(activity)
            != HtmlImageUpdateStartDecision::WaitForFullyActive
        {
            return Err(DomError::ImageProcessingUnavailable);
        }
        if let Some(older) = image.waiting {
            // Only wholly inactive waits are admitted: no acquired resource,
            // callback, promise, continuation or independent task identity can
            // observe the superseded invocation. The high-water survives it.
            let superseded =
                surgeist_html::image_update_is_superseded(older.invocation.cmp(&invocation))
                    .map_err(|_| DomError::ImageProcessingUnavailable)?;
            if !superseded {
                return Err(DomError::ImageProcessingUnavailable);
            }
        }
        self.effect_step(cancel)?;
        let image = self.node_mut(key).image.as_mut().expect("Img");
        image.high_water = invocation;
        image.waiting = Some(crate::image::Waiting { invocation, flags });
        Ok(())
    }
    pub(super) fn cancel_image_update(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.effect_step(cancel)?;
        let Some(waiting) = self
            .node(key)?
            .image
            .as_ref()
            .and_then(|image| image.waiting)
        else {
            return Ok(());
        };
        if let Some(target) = self.published_id(key) {
            let mut bank = self
                .cancelled_images
                .take()
                .unwrap_or_else(|| self.effect_vec());
            // Each preexisting exact node can retire once. Fixed canonical node
            // capacity therefore bounds this actual evidence bank independently
            // of how many superseded updates ran. Exact fallible growth and its
            // whole lifetime also join the existing Select scratch ledger.
            self.select_reserve(&mut bank, 1, cancel)?;
            admit(
                bank.capacity(),
                limits.nodes,
                DomResource::ImageUpdateCancellations,
            )?;
            bank.push(Cancelled {
                update: crate::DomImageUpdateId {
                    target,
                    invocation: waiting.invocation,
                },
                flags: waiting.flags,
            });
            self.cancelled_images = Some(bank);
        }
        self.node_mut(key).image.as_mut().expect("Img").waiting = None;
        Ok(())
    }
    fn image_tag(&self, key: Key) -> Result<Option<KnownHtmlElement>, DomError> {
        Ok(match &self.node(key)?.kind {
            DomNodeKind::Element(element) => element.image_tree_element(),
            _ => None,
        })
    }
    fn update_picture_images(
        &mut self,
        picture: Key,
        after: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut following = after.is_none();
        let mut index = 0;
        while let Some(&child) = self.node(picture)?.children.get(index) {
            self.effect_step(cancel)?;
            index += 1;
            if following && self.node(child)?.image.is_some() {
                self.start_image_update(child, HtmlImageUpdateFlags::new(false, false), cancel)?;
            }
            if after == Some(child) {
                following = true;
            }
        }
        Ok(())
    }
    pub(super) fn image_tree_change(
        &mut self,
        key: Key,
        change: HtmlImageTreeChange,
        root: Key,
        old_parent: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(element @ (KnownHtmlElement::Img | KnownHtmlElement::Source)) =
            self.image_tag(key)?
        else {
            return Ok(());
        };
        self.effect_step(cancel)?;
        let parent = self.node(key)?.parent;
        let parent_tag = parent.map(|p| self.image_tag(p)).transpose()?.flatten();
        let old_tag = old_parent.map(|p| self.image_tag(p)).transpose()?.flatten();
        let effect =
            surgeist_html::image_tree_effect(element, change, parent_tag, old_tag, key == root)
                .expect("actual Img/Source marker");
        if effect.check_media_parent_selection() {
            // Audio/Video are not admitted native families. A future resource
            // owner must provide its real src/networkState/selection operation.
            return Err(DomError::ImageProcessingUnavailable);
        }
        match effect.targets() {
            HtmlImageMutationTargets::None => {}
            HtmlImageMutationTargets::ThisImage => {
                self.start_image_update(key, HtmlImageUpdateFlags::new(false, false), cancel)?
            }
            HtmlImageMutationTargets::ParentPictureImages => {
                self.update_picture_images(parent.expect("actual Picture parent"), None, cancel)?
            }
            HtmlImageMutationTargets::OldParentPictureImages => self.update_picture_images(
                old_parent.expect("actual old Picture parent"),
                None,
                cancel,
            )?,
        }
        if effect.clear_auto_sizes_width() {
            // No native update has passed the inactive guard, and no geometry
            // ingress exists. The actual closed tuple is already null.
            if !self
                .node(key)?
                .image
                .as_ref()
                .expect("Img")
                .native
                .last_auto_sizes_width_is_null()
            {
                return Err(DomError::ImageProcessingUnavailable);
            }
        }
        Ok(())
    }
    fn image_attribute_name_admission(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        if self.source_limits.is_some() {
            self.source_step(cancel)?; // namespace presence
            // The original recognizer scans this exact public vocabulary; each
            // comparison reads at most its spelling and one terminal pair.
            for known in KnownHtmlAttribute::ALL {
                self.source_step(cancel)?;
                for _ in 0..=known.local_name().len() {
                    self.source_step(cancel)?;
                }
            }
        }
        Ok(())
    }
    fn image_comparison_reads(
        &self,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for value in [old, new].into_iter().flatten() {
            if self.source_limits.is_some() {
                self.source_step(cancel)?;
            }
            for _ in value.code_units() {
                self.effect_step(cancel)?;
            }
        }
        Ok(())
    }
    fn image_attribute_operand(
        &self,
        name: &crate::QualifiedName,
        value: Option<&DocumentString>,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Option<crate::Attribute>, DomError> {
        value
            .map(|value| {
                if let Some(budget) = budget {
                    budget.step(cancel)?;
                    let mut payload = source::storage::Payload::default();
                    payload.name(name)?;
                    payload.string(value)?;
                    budget.retain_payload(payload, cancel)?;
                }
                Ok(crate::Attribute::new(name.clone(), value.clone()))
            })
            .transpose()
    }
    fn image_loading_report_budget(
        &self,
        element: &crate::QualifiedName,
        raw: Option<&crate::Attribute>,
        cancel: &AtomicBool,
    ) -> Result<Option<source::formation::Formation>, DomError> {
        let budget = source::formation::Formation::new(self, cancel)?;
        if let Some(budget) = budget.as_ref() {
            budget.step(cancel)?;
            let mut payload = source::storage::Payload::default();
            payload.name(element)?;
            if let Some(raw) = raw {
                budget.step(cancel)?;
                payload.name(raw.name())?;
                payload.string(raw.value())?;
            }
            budget.retain_payload(payload, cancel)?;
            // This is the original LOADING report, as in family::loading_state:
            // expanded attribute name, two HTML namespace comparisons, closed
            // element lookup, then both public Loading keyword comparisons.
            budget.step(cancel)?;
            for _ in 0..2 {
                budget.step(cancel)?;
                for _ in 0..=element.namespace().uri().map_or(0, DocumentString::len) {
                    budget.step(cancel)?;
                }
            }
            for known in KnownHtmlElement::ALL {
                budget.step(cancel)?;
                for _ in 0..=known.local_name().len() {
                    budget.step(cancel)?;
                }
            }
            if let Some(raw) = raw {
                budget.step(cancel)?;
                for _ in 0..=raw.name().local_name().len() {
                    budget.step(cancel)?;
                }
                for keyword in [
                    surgeist_html::HtmlLoading::Lazy,
                    surgeist_html::HtmlLoading::Eager,
                ] {
                    budget.step(cancel)?;
                    // Full raw traversal is a conservative envelope; the
                    // interpreter stops at an earlier mismatch/length guard.
                    for _ in 0..=raw.value().len() {
                        budget.step(cancel)?;
                    }
                    budget.step(cancel)?;
                    for _ in 0..=keyword.canonical_keyword().len() {
                        budget.step(cancel)?;
                    }
                }
            }
        }
        Ok(budget)
    }
    pub(super) fn image_attribute_change(
        &mut self,
        key: Key,
        name: &crate::QualifiedName,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(tag @ (KnownHtmlElement::Img | KnownHtmlElement::Source)) = self.image_tag(key)?
        else {
            return Ok(());
        };
        self.effect_step(cancel)?;
        self.image_attribute_name_admission(cancel)?;
        let Some(attribute) = KnownHtmlAttribute::from_name(name) else {
            return Ok(());
        };
        if tag == KnownHtmlElement::Source {
            if surgeist_html::image_source_attribute_affects_following(name.namespace(), attribute)
                && let Some(parent) = self.node(key)?.parent
                && self.image_tag(parent)? == Some(KnownHtmlElement::Picture)
            {
                self.update_picture_images(parent, Some(key), cancel)?;
            }
            return Ok(());
        }
        if self.source_limits.is_some() {
            self.source_step(cancel)?;
        }
        let same_value = if attribute == KnownHtmlAttribute::Src {
            match (old, new) {
                (Some(a), Some(b)) if a.len() == b.len() => {
                    let mut equal = true;
                    for (a, b) in a.code_units().iter().zip(b.code_units()) {
                        self.effect_step(cancel)?;
                        if *a != *b {
                            equal = false;
                            break;
                        }
                    }
                    equal
                }
                _ => false,
            }
        } else {
            false
        };
        let change = if new.is_some() {
            HtmlImageAttributeChange::Set { same_value }
        } else {
            HtmlImageAttributeChange::Removed
        };
        let effect = surgeist_html::image_attribute_effect(name.namespace(), attribute, change);
        if let HtmlImageAttributeEffect::Update(flags) = effect {
            return self.start_image_update(key, flags, cancel);
        }
        if effect == HtmlImageAttributeEffect::None {
            return Ok(());
        }
        if self.source_limits.is_some()
            && matches!(
                effect,
                HtmlImageAttributeEffect::CompareCrossOrigin
                    | HtmlImageAttributeEffect::CompareReferrerPolicy
            )
        {
            self.image_comparison_reads(old, new, cancel)?;
            let update = match effect {
                HtmlImageAttributeEffect::CompareCrossOrigin => {
                    let old = surgeist_html::bounded::crossorigin_processing_with(old, |_| {
                        self.source_step(cancel)
                    })?;
                    let new = surgeist_html::bounded::crossorigin_processing_with(new, |_| {
                        self.source_step(cancel)
                    })?;
                    surgeist_html::image_crossorigin_changed(old, new)
                }
                HtmlImageAttributeEffect::CompareReferrerPolicy => {
                    let old = surgeist_html::bounded::referrerpolicy_processing_with(old, |_| {
                        self.source_step(cancel)
                    })?;
                    let new = surgeist_html::bounded::referrerpolicy_processing_with(new, |_| {
                        self.source_step(cancel)
                    })?;
                    surgeist_html::image_referrerpolicy_changed(old, new)
                }
                _ => unreachable!("selected borrowed comparison"),
            };
            self.effect_step(cancel)?;
            if update {
                self.start_image_update(key, HtmlImageUpdateFlags::new(false, false), cancel)?;
            }
            return Ok(());
        }
        // This lease precedes both real Attribute owners, so their graphs drop
        // before its debit on every return/error, including nested updates.
        let pair = source::formation::Formation::new(self, cancel)?;
        let old_attr = self.image_attribute_operand(name, old, pair.as_ref(), cancel)?;
        let new_attr = self.image_attribute_operand(name, new, pair.as_ref(), cancel)?;
        if self.source_limits.is_some() {
            self.source_step(cancel)?;
        }
        let bound = old
            .map_or(0, DocumentString::len)
            .max(new.map_or(0, DocumentString::len));
        self.image_comparison_reads(old, new, cancel)?;
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            unreachable!("Img")
        };
        let interpreter = HtmlValueInterpreter::new(bound);
        macro_rules! states {
            ($method:ident) => {{
                let _old_report_budget =
                    self.image_loading_report_budget(element.name(), old_attr.as_ref(), cancel)?;
                let old_report = interpreter
                    .$method(element.name(), old_attr.as_ref())
                    .expect("actual admitted raw extent");
                let old_state = *old_report.intrinsic().expect("actual Img applicability");
                let _new_report_budget =
                    self.image_loading_report_budget(element.name(), new_attr.as_ref(), cancel)?;
                let new_report = interpreter
                    .$method(element.name(), new_attr.as_ref())
                    .expect("actual admitted raw extent");
                #[cfg(test)]
                source_observation::reported(
                    self,
                    old_attr.as_ref(),
                    new_attr.as_ref(),
                    [&old_report, &new_report],
                    self.source_limits.is_some(),
                    cancel,
                )?;
                #[cfg(test)]
                report_cancellation::at_report(self, cancel);
                let new_state = *new_report.intrinsic().expect("actual Img applicability");
                (old_state, new_state)
            }};
        }
        let update = match effect {
            HtmlImageAttributeEffect::CompareCrossOrigin => {
                let (old, new) = states!(crossorigin);
                surgeist_html::image_crossorigin_changed(old, new)
            }
            HtmlImageAttributeEffect::CompareReferrerPolicy => {
                let (old, new) = states!(referrerpolicy);
                surgeist_html::image_referrerpolicy_changed(old, new)
            }
            HtmlImageAttributeEffect::CompareLoading => {
                let (old, new) = states!(loading);
                if surgeist_html::image_loading_became_eager(old, new)
                    && !self
                        .node(key)?
                        .image
                        .as_ref()
                        .expect("Img")
                        .native
                        .lazy_load_resumption_is_null()
                {
                    return Err(DomError::ImageProcessingUnavailable);
                }
                false
            }
            _ => unreachable!("handled direct update/irrelevant effects"),
        };
        self.effect_step(cancel)?;
        if update {
            self.start_image_update(key, HtmlImageUpdateFlags::new(false, false), cancel)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod source_observation {
    use super::*;
    use std::cell::Cell;
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Report {
        pub required: usize,
        pub limit: usize,
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static LAST: Cell<Option<Report>> = const { Cell::new(None) };
    }
    pub(super) fn start() {
        ACTIVE.set(true);
        LAST.set(None);
    }
    pub(super) fn finish() -> Option<Report> {
        ACTIVE.set(false);
        LAST.take()
    }
    fn string(value: &DocumentString) -> usize {
        size_of::<Vec<u16>>() + value.shared_code_units().capacity() * size_of::<u16>()
    }
    fn name(value: &crate::QualifiedName) -> usize {
        value.namespace().uri().map_or(0, string)
            + value.prefix().map_or(0, string)
            + string(value.local_name())
    }
    fn attribute(value: &crate::Attribute) -> usize {
        name(value.name()) + string(value.value())
    }
    pub(super) fn reported<T>(
        draft: &Draft,
        old: Option<&crate::Attribute>,
        new: Option<&crate::Attribute>,
        reports: [&surgeist_html::HtmlAttributeReport<T>; 2],
        registered: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !ACTIVE.get() {
            return Ok(());
        }
        let Some(limits) = draft.source_limits else {
            return Ok(());
        };
        let extra = if registered {
            0
        } else {
            old.map_or(0, attribute)
                + new.map_or(0, attribute)
                + reports
                    .into_iter()
                    .map(|report| {
                        name(report.element()) + report.raw_attribute().map_or(0, attribute)
                    })
                    .sum::<usize>()
        };
        let observed = Report {
            required: draft.source_storage_bytes(cancel)?
                + draft.source_other_bytes
                + draft.source_formation_bytes.load(Ordering::Relaxed)
                + extra,
            limit: limits.retained_bytes,
        };
        if LAST
            .get()
            .is_none_or(|old| old.required < observed.required)
        {
            LAST.set(Some(observed));
        }
        Ok(())
    }
}
#[cfg(test)]
mod report_cancellation {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static LIVE: RefCell<Option<(Arc<AtomicUsize>,Arc<AtomicUsize>)>> = const { RefCell::new(None) }; static ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
    pub(super) fn start() {
        LIVE.with_borrow_mut(|live| *live = None);
        ACTIVE.set(true);
    }
    pub(super) fn at_report(draft: &Draft, cancel: &AtomicBool) {
        if ACTIVE.replace(false) {
            LIVE.with_borrow_mut(|live| {
                *live = Some((
                    draft.source_formation_bytes.clone(),
                    draft.source_temporary_units.clone(),
                ))
            });
            cancel.store(true, Ordering::Release);
        }
    }
    pub(super) fn finish() -> (usize, usize) {
        ACTIVE.set(false);
        let (bytes, units) = LIVE
            .with_borrow_mut(Option::take)
            .expect("actual report reached");
        (bytes.load(Ordering::Relaxed), units.load(Ordering::Relaxed))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Dom, NodeId) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 8192, 128, 0, 0, 8, 131072).unwrap()).unwrap();
        let mut b = DomBatch::new();
        let doc = b.create_document();
        let img = b.create_element_ns(&doc, crate::Namespace::html(), "img");
        let c = dom.apply(b, &cancel).unwrap();
        (dom, c.created(&img).unwrap().unwrap())
    }
    fn source_report(bytes: usize, require_complete: bool) -> Option<source_observation::Report> {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(32, 1_000_000, 128, 0, 0, 8, 1_000_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let img = setup.create_element_ns(&document, crate::Namespace::html(), "img");
        let mut old = Vec::with_capacity(65_536);
        old.extend("lazy".encode_utf16());
        setup.set_attribute(&img, "loading", DocumentString::from_utf16(old));
        let commit = dom.apply(setup, &cancel).unwrap();
        let img = commit.created(&img).unwrap().unwrap();
        let before = dom.snapshot();
        let waiting = before
            .node(img)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id();
        let mut edit = DomBatch::new();
        let mut new = Vec::with_capacity(65_536);
        new.extend("eager".encode_utf16());
        edit.set_attribute(img, "loading", DocumentString::from_utf16(new));
        let limits = crate::DomSourceLimits::new(bytes, 4_000_000, 100_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(edit, limits, &cancel) else {
            return None;
        };
        source_observation::start();
        let result = lease.advance(&cancel);
        let report = source_observation::finish();
        if require_complete {
            assert!(matches!(result, Ok(crate::DomSourceProgress::Complete(_))));
            assert_eq!(
                lease
                    .snapshot()
                    .node(img)
                    .unwrap()
                    .as_img()
                    .unwrap()
                    .suspended_update()
                    .unwrap()
                    .unwrap()
                    .id(),
                waiting
            );
            assert_eq!(
                before.node(img).unwrap().attribute(&"loading".into()),
                Some(&"lazy".into())
            );
            assert_eq!(
                lease
                    .snapshot()
                    .node(img)
                    .unwrap()
                    .attribute(&"loading".into()),
                Some(&"eager".into())
            );
        } else if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        report
    }
    #[test]
    fn source_image_comparison_admits_actual_attribute_and_report_occurrences() {
        let high = source_report(16_000_000, true).expect("actual loading report");
        if let Some(actual) = source_report(high.required - 1, false) {
            assert!(
                actual.required <= actual.limit,
                "Image comparison retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
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
    #[test]
    fn source_image_loading_report_has_exact_graph_custody_and_one_short_refusal() {
        let (mut dom, img) = fixture();
        let cancel = AtomicBool::new(false);
        let mut setup = DomBatch::new();
        setup.set_attribute(img, "loading", "lazy");
        dom.apply(setup, &cancel).unwrap();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let key = Key::Existing(img.slot);
        let graph = || {
            let DomNodeKind::Element(element) = &draft.node(key).unwrap().kind else {
                unreachable!()
            };
            let raw = element
                .attributes()
                .iter()
                .find(|a| {
                    a.name().local_name().code_units()
                        == "loading".encode_utf16().collect::<Vec<_>>()
                })
                .unwrap();
            let mut bytes = 0;
            let mut units = 0;
            for name in [element.name(), raw.name()] {
                for value in [
                    name.namespace().uri(),
                    name.prefix(),
                    Some(name.local_name()),
                ]
                .into_iter()
                .flatten()
                {
                    bytes += size_of::<Vec<u16>>()
                        + value.shared_code_units().capacity() * size_of::<u16>();
                    units += value.shared_code_units().capacity();
                }
            }
            bytes += size_of::<Vec<u16>>()
                + raw.value().shared_code_units().capacity() * size_of::<u16>();
            units += raw.value().shared_code_units().capacity();
            (bytes, units)
        };
        let (bytes, units) = graph();
        source_budget(&mut draft, bytes, units);
        {
            let DomNodeKind::Element(element) = &draft.node(key).unwrap().kind else {
                unreachable!()
            };
            let raw = element.attributes().iter().next().unwrap();
            let budget = draft
                .image_loading_report_budget(element.name(), Some(raw), &cancel)
                .unwrap()
                .unwrap();
            let report = HtmlValueInterpreter::new(4)
                .loading(element.name(), Some(raw))
                .unwrap();
            assert_eq!(report.intrinsic(), Some(&surgeist_html::HtmlLoading::Lazy));
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), bytes);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), units);
            drop(report);
            drop(budget);
        }
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        for (b, u, resource) in [
            (bytes - 1, units, DomResource::SourceRetainedBytes),
            (bytes, units - 1, DomResource::SourceRetainedUnits),
        ] {
            source_budget(&mut draft, b, u);
            let DomNodeKind::Element(element) = &draft.node(key).unwrap().kind else {
                unreachable!()
            };
            let raw = element.attributes().iter().next().unwrap();
            assert!(
                matches!(draft.image_loading_report_budget(element.name(),Some(raw),&cancel),Err(DomError::ResourceLimit {resource:actual,..}) if actual==resource)
            );
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        }
    }
    #[test]
    fn source_image_comparisons_preserve_literal_states_and_ordinary_none_results() {
        for (attribute, old, new, delta) in [
            ("crossorigin", None, Some("use-credentials"), 1),
            ("crossorigin", Some("bad"), Some("ANONYMOUS"), 0),
            ("crossorigin", Some("use-credentials"), None, 1),
            ("referrerpolicy", Some("origin"), Some("NO-REFERRER"), 1),
            ("referrerpolicy", Some("bad"), Some(""), 0),
            ("loading", Some("lazy"), Some("eager"), 0),
            ("loading", Some("bad"), Some("LAZY"), 0),
            ("src", Some("same"), Some("same"), 1),
        ] {
            for source in [false, true] {
                let (mut dom, img) = fixture();
                let cancel = AtomicBool::new(false);
                if let Some(old) = old {
                    let mut setup = DomBatch::new();
                    setup.set_attribute(img, attribute, old);
                    dom.apply(setup, &cancel).unwrap();
                }
                let before = dom.snapshot();
                let invocation = before
                    .node(img)
                    .unwrap()
                    .as_img()
                    .unwrap()
                    .suspended_update()
                    .unwrap()
                    .unwrap()
                    .id()
                    .invocation;
                let mut edit = DomBatch::new();
                if let Some(new) = new {
                    edit.set_attribute(img, attribute, new);
                } else {
                    edit.remove_attribute(img, attribute);
                }
                if source {
                    let mut lease = dom
                        .begin_source_commands(
                            edit,
                            crate::DomSourceLimits::new(
                                8_000_000,
                                4_000_000,
                                100_000_000,
                                64,
                                4096,
                                4096,
                                1,
                            ),
                            &cancel,
                        )
                        .unwrap();
                    assert!(matches!(
                        lease.advance(&cancel).unwrap(),
                        crate::DomSourceProgress::Complete(_)
                    ));
                } else {
                    dom.apply(edit, &cancel).unwrap();
                }
                let after = dom.snapshot();
                assert_eq!(
                    after
                        .node(img)
                        .unwrap()
                        .as_img()
                        .unwrap()
                        .suspended_update()
                        .unwrap()
                        .unwrap()
                        .id()
                        .invocation,
                    invocation + delta,
                    "{attribute} {old:?}->{new:?} Source={source}"
                );
                assert_eq!(
                    before.node(img).unwrap().attribute(&attribute.into()),
                    old.map(DocumentString::from).as_ref()
                );
                assert_eq!(
                    after.node(img).unwrap().attribute(&attribute.into()),
                    new.map(DocumentString::from).as_ref()
                );
            }
        }
    }
    #[test]
    fn source_image_cancelled_report_unwinds_actual_graphs_without_publication_then_retries() {
        let (mut dom, img) = fixture();
        let cancel = AtomicBool::new(false);
        let mut setup = DomBatch::new();
        setup.set_attribute(img, "loading", "lazy");
        dom.apply(setup, &cancel).unwrap();
        let before = dom.snapshot();
        let edit = || {
            let mut edit = DomBatch::new();
            edit.set_attribute(img, "loading", "eager");
            edit
        };
        let limits =
            crate::DomSourceLimits::new(8_000_000, 4_000_000, 100_000_000, 64, 4096, 4096, 1);
        let mut lease = dom.begin_source_commands(edit(), limits, &cancel).unwrap();
        report_cancellation::start();
        assert!(matches!(lease.advance(&cancel), Err(DomError::Cancelled)));
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        drop(lease);
        assert_eq!(report_cancellation::finish(), (0, 0));
        assert!(dom.pending_source_edit().is_none());
        cancel.store(false, Ordering::Release);
        let mut lease = dom.begin_source_commands(edit(), limits, &cancel).unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            crate::DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert_eq!(
            before.node(img).unwrap().attribute(&"loading".into()),
            Some(&"lazy".into())
        );
        assert_eq!(
            dom.snapshot()
                .node(img)
                .unwrap()
                .attribute(&"loading".into()),
            Some(&"eager".into())
        );
    }
    #[test]
    fn checked_invocation_exhaustion_refuses_without_wrapping_or_publishing_attributes() {
        let (mut dom, img) = fixture();
        Arc::get_mut(&mut dom.state)
            .unwrap()
            .mutate_slot(img.slot, |slot| {
                let image = slot.node.as_mut().unwrap().image.as_mut().unwrap();
                image.high_water = u64::MAX;
                image.waiting.as_mut().unwrap().invocation = u64::MAX;
            });
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_attribute(img, "src", "new");
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::ImageUpdateExhausted
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
    #[test]
    fn cancellation_and_revision_refusal_keep_the_owned_latest_wait_and_retirement_evidence_atomic()
    {
        let (mut dom, img) = fixture();
        let cancel = AtomicBool::new(false);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_attribute(img, "src", "prepared");
        b.destroy(img);
        let prepared = dom.prepare(b, &cancel).unwrap();
        cancel.store(true, Ordering::Release);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        cancel.store(false, Ordering::Release);
        dom.revision = DomRevision(u64::MAX);
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        b.set_attribute(img, "src", "private");
        b.destroy(img);
        assert_eq!(
            dom.apply(b, &cancel).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
    #[test]
    fn destruction_cancels_cleanup_started_wait_and_registers_the_complete_evidence_bank_lifetime()
    {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 8192, 128, 0, 0, 8, 131072).unwrap()).unwrap();
        let mut b = DomBatch::new();
        let doc = b.create_document();
        let picture = b.create_element_ns(&doc, crate::Namespace::html(), "picture");
        let source = b.create_element_ns(&doc, crate::Namespace::html(), "source");
        let img = b.create_element_ns(&doc, crate::Namespace::html(), "img");
        b.insert_before(&source, &picture, None);
        b.insert_before(&img, &picture, None);
        let option = b.create_element_ns(&doc, crate::Namespace::html(), "option");
        let c = dom.apply(b, &cancel).unwrap();
        let picture = c.created(&picture).unwrap().unwrap();
        let img = c.created(&img).unwrap().unwrap();
        let option = c.created(&option).unwrap().unwrap();
        assert_eq!(
            dom.state.slots[img.slot]
                .node
                .as_ref()
                .unwrap()
                .image
                .as_ref()
                .unwrap()
                .high_water,
            2
        ); // creation, Picture insertion.
        let mut b = DomBatch::new();
        b.destroy(picture);
        let mut d = Draft::new(&dom, b.owner, &cancel).unwrap();
        d.apply(&b.commands.into_iter().next().unwrap(), dom.limits, &cancel)
            .unwrap();
        let bank = d.cancelled_images.as_ref().unwrap();
        assert_eq!(bank.len(), 1);
        assert_eq!(bank[0].update.invocation, 4); // Source removal fanout, Img removal.
        assert_eq!(bank[0].update.target, img);
        assert_eq!(bank[0].flags, HtmlImageUpdateFlags::new(false, false));
        assert!(bank.capacity() <= dom.limits.nodes);
        let capacity_bytes = bank.capacity() * size_of::<Cancelled>();
        assert!(capacity_bytes > 0);
        assert_eq!(
            d.select_temporary_bytes.load(Ordering::Relaxed),
            capacity_bytes
        );
        drop(d);
        // The bank remains alive through a later actual public Option setter,
        // whose Select fast path allocates no extra storage itself.
        let before = dom.snapshot();
        dom.set_limits(
            dom.limits
                .with_select_processing(0, capacity_bytes - 1, 8)
                .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        b.destroy(picture);
        b.set_option_selected(option, true);
        assert!(matches!(
            dom.apply(b, &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SelectScratch,
                ..
            })
        ));
        assert_eq!(*dom.state, *before.state);
        dom.set_limits(
            dom.limits
                .with_select_processing(0, capacity_bytes, 8)
                .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        b.destroy(picture);
        b.set_option_selected(option, true);
        let commit = dom.apply(b, &cancel).unwrap();
        assert!(commit.changes().changes().iter().any(|change| matches!(change, DomChange::ImageUpdateCancelled { update, .. } if update.target == img && update.invocation == 4)));
    }
}
