//! Actual native control state, bounded reset observations and native mutation hooks.
use super::*;
use surgeist_html::KnownHtmlAttribute;
use surgeist_html::bounded::{
    HtmlFormOwnerResetDecision as Decision, HtmlFormOwnerResetTrigger as Trigger,
    form_owner_reset_trigger,
};

fn state(
    kind: &DomNodeKind,
) -> Option<(
    &crate::control::NativeControlState,
    surgeist_html::HtmlNativeFormAssociationKind,
)> {
    match kind {
        DomNodeKind::Element(element) => element.native_control(),
        _ => None,
    }
}
// Association is wider than constraint-validation/custom-error control state.
// Standalone Img is nonlisted and has a real parser flag/form owner, but cannot
// acquire a custom validity message through this join.
fn association(node: &DraftNode) -> Option<(bool, surgeist_html::HtmlNativeFormAssociationKind)> {
    state(&node.kind)
        .map(|(control, kind)| (control.parser_inserted, kind))
        .or_else(|| {
            node.image.as_ref().map(|image| {
                (
                    image.parser_inserted,
                    surgeist_html::HtmlNativeFormAssociationKind::Image,
                )
            })
        })
}
impl Draft {
    fn control_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<&mut crate::control::NativeControlState, DomError> {
        self.prepare_family_mutation(key, cancel)?;
        let DomNodeKind::Element(element) = &mut self.node_mut(key).kind else {
            unreachable!("checked native control")
        };
        Ok(element
            .native_control_mut()
            .expect("checked native control"))
    }
    pub(super) fn set_custom_validity(
        &mut self,
        key: Key,
        raw: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let old = state(&self.node(key)?.kind)
            .ok_or(DomError::CustomValidityInapplicable)?
            .0
            .message
            .len();
        let mut length = 0usize;
        let mut normalizer = surgeist_html::HtmlNewlineNormalizer::new();
        for unit in raw.code_units() {
            self.source_step(cancel)?;
            if normalizer.push(*unit).is_some() {
                length = length
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        let units = (self.units - old)
            .checked_add(length)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        #[cfg(test)]
        let growth = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::CustomValidity,
            cancel,
        )?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let mut normalized = match &formation {
            Some(formation) => {
                source::formation::utf16_buffer(length, limits.units, Some(formation), cancel)?
            }
            None => {
                let mut output = Vec::new();
                reserve(&mut output, length)?;
                output
            }
        };
        let mut normalizer = surgeist_html::HtmlNewlineNormalizer::new();
        for unit in raw.code_units() {
            self.source_step(cancel)?;
            if let Some(unit) = normalizer.push(*unit) {
                self.source_step(cancel)?;
                normalized.push(unit);
            }
        }
        self.source_step(cancel)?;
        let normalized = DocumentString::from_utf16(normalized);
        #[cfg(test)]
        source::formation::mutation_observation::grew(
            growth,
            size_of::<Vec<u16>>() + normalized.shared_code_units().capacity() * size_of::<u16>(),
        );
        self.control_mut(key, cancel)?.message = normalized;
        self.units = units;
        drop(formation);
        Ok(())
    }
    pub(super) fn control_attribute(
        &self,
        key: Key,
        name: KnownHtmlAttribute,
        cancel: &AtomicBool,
    ) -> Result<Option<&DocumentString>, DomError> {
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Ok(None);
        };
        for attribute in element.attributes().iter() {
            self.source_step(cancel)?;
            if attribute.name().namespace().uri().is_none()
                && self.source_ascii(attribute.name().local_name(), name.local_name(), cancel)?
            {
                return Ok(Some(attribute.value()));
            }
        }
        Ok(None)
    }
    fn has_control_id(&self, key: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        Ok(self
            .control_attribute(key, KnownHtmlAttribute::Id, cancel)?
            .is_some_and(|id| !id.is_empty()))
    }
    fn is_form(&self, key: Key) -> Result<bool, DomError> {
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Ok(false);
        };
        Ok(element.form_state().is_some())
    }
    fn nearest_form(&self, key: Key, cancel: &AtomicBool) -> Result<Option<Key>, DomError> {
        let mut next = self.node(key)?.parent;
        while let Some(key) = next {
            self.source_step(cancel)?;
            if self.is_form(key)? {
                return Ok(Some(key));
            }
            next = self.node(key)?.parent;
        }
        Ok(None)
    }
    pub(super) fn control_connected(
        &self,
        mut key: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        loop {
            self.source_step(cancel)?;
            match self.shadow_including_parent(key)? {
                Some(parent) => key = parent,
                None => return Ok(matches!(self.node(key)?.kind, DomNodeKind::Document(_))),
            }
        }
    }
    fn first_id_form(&self, key: Key, cancel: &AtomicBool) -> Result<Option<Key>, DomError> {
        let value = self
            .control_attribute(key, KnownHtmlAttribute::Form, cancel)?
            .expect("cursor established presence");
        let root = crate::slot::root(self, key, cancel)?;
        let mut next = Some(root);
        while let Some(candidate) = next {
            self.source_step(cancel)?;
            if let Some(id) = self.control_attribute(candidate, KnownHtmlAttribute::Id, cancel)?
                && !id.is_empty()
                && self.same_control_units(id, value, cancel)?
            {
                return Ok(self.is_form(candidate)?.then_some(candidate));
            }
            next = crate::slot::next(self, root, candidate, cancel)?;
        }
        Ok(None)
    }
    fn reset_control(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        use surgeist_html::bounded::{
            HtmlFormOwnerResetAction as A, HtmlFormOwnerResetCursor, HtmlFormOwnerResetReply as R,
        };
        let (_, kind) =
            association(self.node(key)?).expect("actual native form-associated element");
        let owner = self.node(key)?.control_owner;
        let mut former_owner = None;
        let mut found = None;
        let mut cursor = HtmlFormOwnerResetCursor::new(kind.listed(), owner.is_some());
        loop {
            self.source_step(cancel)?;
            let reply = match cursor.action() {
                A::ClearParserInserted => {
                    if let Some(image) = self.node_mut(key).image.as_mut() {
                        image.parser_inserted = false;
                    } else {
                        self.control_mut(key, cancel)?.parser_inserted = false;
                    }
                    R::Applied
                }
                A::ReadFormAttributePresence => R::FormAttributePresent(
                    self.control_attribute(key, KnownHtmlAttribute::Form, cancel)?
                        .is_some(),
                ),
                A::ReadCurrentOwnerIsNearestAncestor => {
                    let owner = owner.expect("cursor requires owner");
                    R::CurrentOwnerIsNearestAncestor(self.nearest_form(key, cancel)? == Some(owner))
                }
                A::ClearOwner => {
                    former_owner = self.node_mut(key).control_owner.take();
                    R::Applied
                }
                A::ReadConnectedness => R::Connected(self.control_connected(key, cancel)?),
                A::FindFirstIdMatch => {
                    found = self.first_id_form(key, cancel)?;
                    R::FormFound(found.is_some())
                }
                A::FindNearestAncestor => {
                    found = self.nearest_form(key, cancel)?;
                    R::FormFound(found.is_some())
                }
                A::AssociateFoundOwner => {
                    let candidate = found.expect("preceding lookup found an actual Form");
                    former_owner = self.node_mut(key).control_owner.replace(candidate);
                    R::Applied
                }
                A::ApplyOwnerChangeEffects => {
                    if let Some(former) = former_owner.take() {
                        self.remove_form_aliases(former, key, cancel)?;
                    }
                    match kind {
                        surgeist_html::HtmlNativeFormAssociationKind::Button
                        | surgeist_html::HtmlNativeFormAssociationKind::Fieldset
                        | surgeist_html::HtmlNativeFormAssociationKind::Textarea
                        | surgeist_html::HtmlNativeFormAssociationKind::Output
                        | surgeist_html::HtmlNativeFormAssociationKind::Select
                        | surgeist_html::HtmlNativeFormAssociationKind::Image => {}
                        surgeist_html::HtmlNativeFormAssociationKind::Input => self
                            .input_radio_normalize(
                                key,
                                surgeist_html::HtmlRadioTrigger::FormOwner,
                                cancel,
                            )?,
                        _ => unreachable!("actual admitted native control kinds"),
                    }
                    R::Applied
                }
                A::Complete => return Ok(()),
            };
            cursor
                .reply(reply)
                .expect("reply after actual admitted source action");
        }
    }
    fn reset_control_for(
        &mut self,
        key: Key,
        trigger: Trigger,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let reset = match form_owner_reset_trigger(trigger) {
            Decision::Skip => false,
            Decision::Reset => true,
            Decision::ResetIfFormAttributePresent => self
                .control_attribute(key, KnownHtmlAttribute::Form, cancel)?
                .is_some(),
            Decision::ResetIfOwnerInDifferentTree => {
                let owner = self
                    .node(key)?
                    .control_owner
                    .expect("trigger requires owner");
                crate::slot::root(self, key, cancel)? != crate::slot::root(self, owner, cancel)?
            }
        };
        if reset {
            self.reset_control(key, cancel)?;
        }
        Ok(())
    }
    fn id_recipients(
        &mut self,
        source: Key,
        same_tree: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let document = self.node(source)?.document;
        let root = if same_tree {
            Some(crate::slot::root(self, source, cancel)?)
        } else {
            None
        };
        // The actual ID event selects listed, form-attribute-bearing subjects;
        // this is not an end-batch repair or an unconditional control reset.
        for index in 0..self.existing.len() + self.created.len() {
            self.source_step(cancel)?;
            let key = if index < self.existing.len() {
                Key::Existing(index)
            } else {
                Key::Created(index - self.existing.len())
            };
            let Ok(node) = self.node(key) else { continue };
            let Some((_, kind)) = association(node) else {
                continue;
            };
            if node.document == document
                && (root.is_none() || Some(crate::slot::root(self, key, cancel)?) == root)
            {
                self.reset_control_for(
                    key,
                    Trigger::IdEvent {
                        listed: kind.listed(),
                    },
                    cancel,
                )?;
            }
        }
        Ok(())
    }
    /// One actual insertion extension, called in shadow-including order by
    /// the owning native insertion traversal after the Slot steps.
    pub(super) fn control_insertion_node(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some((parser_inserted, _)) = association(self.node(key)?) {
            self.reset_control_for(key, Trigger::Insertion { parser_inserted }, cancel)?;
            if self.control_connected(key, cancel)? {
                self.input_radio_normalize(
                    key,
                    surgeist_html::HtmlRadioTrigger::Connected,
                    cancel,
                )?;
            }
        }
        if self.has_control_id(key, cancel)?
            && matches!(
                self.node(crate::slot::root(self, key, cancel)?)?.kind,
                DomNodeKind::Document(_)
            )
        {
            self.id_recipients(key, false, cancel)?;
        }
        Ok(())
    }
    /// HTML removing runs after this same node's family-specific extension.
    pub(super) fn control_removing_node(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if association(self.node(key)?).is_some() {
            self.reset_control_for(
                key,
                Trigger::RemovingOrMoving {
                    has_owner: self.node(key)?.control_owner.is_some(),
                },
                cancel,
            )?;
        }
        // HTML877d1478–1485: post-removal nonmembership in a Document tree,
        // without an old-connected prerequisite.
        if self.has_control_id(key, cancel)?
            && !matches!(
                self.node(crate::slot::root(self, key, cancel)?)?.kind,
                DomNodeKind::Document(_)
            )
        {
            self.id_recipients(key, false, cancel)?;
        }
        Ok(())
    }
    pub(super) fn control_attribute_change(
        &mut self,
        key: Key,
        changed: &crate::QualifiedName,
        old: Option<&DocumentString>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if changed.namespace().uri().is_some() {
            return Ok(());
        }
        let is = |name: KnownHtmlAttribute| {
            changed
                .local_name()
                .code_units()
                .iter()
                .copied()
                .eq(name.local_name().encode_utf16())
        };
        if is(KnownHtmlAttribute::Form)
            && let Some((_, kind)) = association(self.node(key)?)
        {
            self.reset_control_for(
                key,
                Trigger::FormAttributeChanged {
                    listed: kind.listed(),
                },
                cancel,
            )?;
        }
        if is(KnownHtmlAttribute::Id) {
            let new = self.control_attribute(key, KnownHtmlAttribute::Id, cancel)?;
            let old = old.filter(|value| !value.is_empty());
            let new = new.filter(|value| !value.is_empty());
            let changed = match (old, new) {
                (None, None) => false,
                (Some(a), Some(b)) => !self.same_control_units(a, b, cancel)?,
                _ => true,
            };
            if changed {
                self.id_recipients(key, true, cancel)?;
            }
        }
        Ok(())
    }
    fn same_control_units(
        &self,
        a: &DocumentString,
        b: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        if self.source_limits.is_some() {
            self.source_step(cancel)?;
        }
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.code_units().iter().zip(b.code_units()) {
            self.source_step(cancel)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_after_prepared_message_and_reset_hooks_preserves_all_published_state() {
        let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 32, 16, 8, 65536).unwrap()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        let d = b.create_document();
        let f = b.create_element_ns(&d, crate::Namespace::html(), "fieldset");
        b.insert_before(&f, &d, None);
        let c = dom.apply(b, &cancel).unwrap();
        let f = c.created(&f).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_custom_validity(f, "prepared\rmessage");
        b.set_attribute(f, "form", "");
        b.detach(f);
        let prepared = dom.prepare(b, &cancel).unwrap();
        cancel.store(true, Ordering::Release);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}

#[cfg(test)]
pub(super) mod participant_tests;
