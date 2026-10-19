//! Shared actual nullable reflected-target writes and permitted-weak retirement.
use super::*;
use crate::reflected::ReflectedTarget;
impl Draft {
    pub(super) fn set_reflected_target(
        &mut self,
        node: Key,
        key: ReflectedTarget,
        target: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.set_reflected_target_admitted(node, key, target, limits, cancel, &mut || {
            check_cancel(cancel)
        })
    }
    fn set_reflected_target_admitted(
        &mut self,
        node: Key,
        key: ReflectedTarget,
        target: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
        admit_effect: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        use surgeist_html::{
            HtmlReflectedElementSetter as Cursor, HtmlReflectedElementSetterEffect as Effect,
        };
        if self
            .node(node)?
            .reflected_targets
            .and_then(|targets| targets.get(key))
            .is_none()
        {
            return Err(DomError::ButtonInapplicable);
        }
        if let Some(target) = target
            && !matches!(self.node(target)?.kind, DomNodeKind::Element(_))
        {
            return Err(DomError::WrongNodeKind);
        }
        let mut cursor = if target.is_some() {
            Cursor::element()
        } else {
            Cursor::null()
        };
        while let Some(effect) = cursor.next_effect() {
            admit_effect()?;
            match effect {
                Effect::ClearExplicitTarget => {
                    self.with_node_fields(node, cancel, |fields| {
                        *fields
                            .reflected_targets
                            .as_mut()
                            .expect("checked Button")
                            .slot_mut(key)
                            .expect("checked reflected slot") = None
                    })?;
                }
                Effect::RemoveContentAttribute => {
                    let operand = source::formation::Formation::new(self, cancel)?;
                    let name = source::formation::string(
                        key.attribute().local_name(),
                        operand.as_ref(),
                        cancel,
                    )?;
                    self.remove_attribute(
                        node,
                        Some(&crate::Namespace::none()),
                        &name,
                        limits,
                        cancel,
                    )?;
                }
                Effect::SetEmptyContentAttribute => {
                    let operand = source::formation::Formation::new(self, cancel)?;
                    let name = source::formation::string(
                        key.attribute().local_name(),
                        operand.as_ref(),
                        cancel,
                    )?;
                    let value = source::formation::string("", operand.as_ref(), cancel)?;
                    self.set_attribute(
                        node,
                        Some(&crate::Namespace::none()),
                        &name,
                        &value,
                        limits,
                        cancel,
                    )?;
                }
                Effect::SetExplicitTargetFromSuppliedElement => {
                    self.with_node_fields(node, cancel, |fields| {
                        *fields
                            .reflected_targets
                            .as_mut()
                            .expect("checked Button")
                            .slot_mut(key)
                            .expect("checked reflected slot") = target
                    })?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn reflected_attribute_change(
        &mut self,
        node: Key,
        changed: &crate::DomQualifiedName,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.with_node_fields(node, cancel, |fields| {
            if let Some(targets) = fields.reflected_targets.as_mut() {
                for key in [ReflectedTarget::CommandFor, ReflectedTarget::PopoverTarget] {
                    if surgeist_html::reflected_element_attribute_change(key.attribute(), changed)
                        && let Some(slot) = targets.slot_mut(key)
                    {
                        *slot = None;
                    }
                }
            }
        })
    }
    pub(super) fn retire_reflected_target(
        &mut self,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let owners = self.inbound_keys(target, cancel)?;
        for owner in owners.iter().copied() {
            self.source_step(cancel)?;
            let Some(targets) = self.node(owner)?.reflected_targets else {
                continue;
            };
            if ![ReflectedTarget::CommandFor, ReflectedTarget::PopoverTarget]
                .into_iter()
                .any(|key| targets.get(key) == Some(Some(target)))
            {
                continue;
            }
            self.with_node_fields(owner, cancel, |fields| {
                let targets = fields
                    .reflected_targets
                    .as_mut()
                    .expect("qualified reflected owner");
                for key in [ReflectedTarget::CommandFor, ReflectedTarget::PopoverTarget] {
                    if let Some(slot) = targets.slot_mut(key)
                        && *slot == Some(target)
                    {
                        *slot = None;
                    }
                }
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stopping_between_reflected_setter_effects_observes_actual_asymmetric_order() {
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                2048,
                32,
                0,
                0,
                8,
                65536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        let d = b.create_document();
        let button = b.create_element_ns(&d, crate::Namespace::html(), "button");
        let other = b.create_element(&d, "other");
        b.set_command_for_element(&button, Some((&button).into()));
        let c = dom.apply(b, &cancel).unwrap();
        let button = c.created(&button).unwrap().unwrap();
        let other = c.created(&other).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        for supplied in [None, Some(other)] {
            let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
            let key = draft.resolve(button.into()).unwrap();
            let target = supplied.map(|id| draft.resolve(id.into()).unwrap());
            let mut effects = 0;
            assert_eq!(
                draft.set_reflected_target_admitted(
                    key,
                    ReflectedTarget::CommandFor,
                    target,
                    dom.limits(),
                    &cancel,
                    &mut || {
                        effects += 1;
                        if effects == 2 {
                            Err(DomError::Cancelled)
                        } else {
                            Ok(())
                        }
                    }
                ),
                Err(DomError::Cancelled)
            );
            assert_eq!(effects, 2);
            assert_eq!(
                draft
                    .node(key)
                    .unwrap()
                    .reflected_targets
                    .unwrap()
                    .get(ReflectedTarget::CommandFor)
                    .unwrap(),
                None
            );
            let DomNodeKind::Element(element) = &draft.node(key).unwrap().kind else {
                unreachable!()
            };
            // Non-null completed empty-content write and its clearing hook;
            // null completed explicit-clear before deleting the old content.
            assert_eq!(
                element.attribute(&"commandfor".into()),
                Some(&DocumentString::from(""))
            );
            assert_eq!(*dom.state, *before.state);
            assert_eq!(dom.usage(), usage);
        }
        let mut b = DomBatch::new();
        b.set_popover_target_element(button, Some(other.into()));
        b.set_custom_validity(button, "private\rstate");
        let prepared = dom.prepare(b, &cancel).unwrap();
        cancel.store(true, Ordering::Release);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
}
