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
        let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        // Every admitted creation/install/adoption target establishes this
        // actual never-navigable environment. No activation or hosted Document
        // ingress is admitted. Do not substitute the later disabled-images step.
        let activity = if !document.native_environment().has_ever_had_navigable() {
            HtmlImageDocumentActivity::NotFullyActive
        } else {
            HtmlImageDocumentActivity::Unavailable
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
        if let Key::Existing(slot) = key {
            let mut bank = self
                .cancelled_images
                .take()
                .unwrap_or_else(|| self.effect_vec());
            // Each preexisting exact node can retire once. Fixed canonical node
            // capacity therefore bounds this actual evidence bank independently
            // of how many superseded updates ran. Exact fallible growth and its
            // whole lifetime also join the existing Select scratch ledger.
            self.select_reserve(&mut bank, 1)?;
            admit(
                bank.capacity(),
                limits.nodes,
                DomResource::ImageUpdateCancellations,
            )?;
            bank.push(Cancelled {
                update: crate::DomImageUpdateId {
                    target: NodeId {
                        owner: self.store,
                        slot,
                        generation: self.generations[slot],
                    },
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
        let same_value = if attribute == KnownHtmlAttribute::Src {
            match (old, new) {
                (Some(a), Some(b)) if a.len() == b.len() => {
                    let mut equal = true;
                    for (&a, &b) in a.code_units().iter().zip(b.code_units()) {
                        self.effect_step(cancel)?;
                        if a != b {
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
        let old_attr = old.map(|v| crate::Attribute::new(name.clone(), v.clone()));
        let new_attr = new.map(|v| crate::Attribute::new(name.clone(), v.clone()));
        let bound = old
            .map_or(0, DocumentString::len)
            .max(new.map_or(0, DocumentString::len));
        for value in [old, new].into_iter().flatten() {
            for _ in value.code_units() {
                self.effect_step(cancel)?;
            }
        }
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            unreachable!("Img")
        };
        let interpreter = HtmlValueInterpreter::new(bound);
        macro_rules! states {
            ($method:ident) => {
                (
                    *interpreter
                        .$method(element.name(), old_attr.as_ref())
                        .expect("actual admitted raw extent")
                        .intrinsic()
                        .expect("actual Img applicability"),
                    *interpreter
                        .$method(element.name(), new_attr.as_ref())
                        .expect("actual admitted raw extent")
                        .intrinsic()
                        .expect("actual Img applicability"),
                )
            };
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
    #[test]
    fn checked_invocation_exhaustion_refuses_without_wrapping_or_publishing_attributes() {
        let (mut dom, img) = fixture();
        let image = Arc::get_mut(&mut dom.state).unwrap().slots[img.slot]
            .node
            .as_mut()
            .unwrap()
            .image
            .as_mut()
            .unwrap();
        image.high_water = u64::MAX;
        image.waiting.as_mut().unwrap().invocation = u64::MAX;
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
        d.apply(b.commands.into_iter().next().unwrap(), dom.limits, &cancel)
            .unwrap();
        let bank = d.cancelled_images.as_ref().unwrap();
        assert_eq!(bank.len(), 1);
        assert_eq!(bank[0].update.invocation, 4); // Source removal fanout, Img removal.
        assert_eq!(bank[0].update.target, img);
        assert_eq!(bank[0].flags, HtmlImageUpdateFlags::new(false, false));
        assert!(bank.capacity() <= dom.limits.nodes);
        let capacity_bytes = bank.capacity() * size_of::<Cancelled>();
        assert!(capacity_bytes > 0);
        assert_eq!(d.select_temporary_bytes.get(), capacity_bytes);
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
