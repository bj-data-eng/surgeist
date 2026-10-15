//! Actual nullable reflected-target writes and permitted-weak retirement.
use super::*;
use crate::button::ButtonTarget;
impl Draft {
    pub(super) fn set_button_target(
        &mut self,
        node: Key,
        key: ButtonTarget,
        target: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.set_button_target_admitted(node, key, target, limits, cancel, &mut || {
            check_cancel(cancel)
        })
    }
    fn set_button_target_admitted(
        &mut self,
        node: Key,
        key: ButtonTarget,
        target: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
        admit_effect: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        use surgeist_html::{
            HtmlReflectedElementSetter as Cursor, HtmlReflectedElementSetterEffect as Effect,
        };
        if self.node(node)?.button_targets.is_none() {
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
                    *self
                        .node_mut(node)
                        .button_targets
                        .as_mut()
                        .expect("checked Button")
                        .slot_mut(key) = None
                }
                Effect::RemoveContentAttribute => self.remove_attribute(
                    node,
                    Some(crate::Namespace::none()),
                    key.attribute().local_name().into(),
                    limits,
                    cancel,
                )?,
                Effect::SetEmptyContentAttribute => self.set_attribute(
                    node,
                    Some(crate::Namespace::none()),
                    key.attribute().local_name().into(),
                    "".into(),
                    limits,
                    cancel,
                )?,
                Effect::SetExplicitTargetFromSuppliedElement => {
                    *self
                        .node_mut(node)
                        .button_targets
                        .as_mut()
                        .expect("checked Button")
                        .slot_mut(key) = target
                }
            }
        }
        Ok(())
    }
    pub(super) fn button_attribute_change(&mut self, node: Key, changed: &crate::QualifiedName) {
        if let Some(targets) = self.node_mut(node).button_targets.as_mut() {
            for key in [ButtonTarget::CommandFor, ButtonTarget::PopoverTarget] {
                if surgeist_html::reflected_element_attribute_change(key.attribute(), changed) {
                    *targets.slot_mut(key) = None;
                }
            }
        }
    }
    pub(super) fn retire_button_target(
        &mut self,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for node in self
            .existing
            .iter_mut()
            .chain(&mut self.created)
            .filter_map(Option::as_mut)
        {
            check_cancel(cancel)?;
            if let Some(targets) = &mut node.button_targets {
                if targets.command_for == Some(target) {
                    targets.command_for = None;
                }
                if targets.popover_target == Some(target) {
                    targets.popover_target = None;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stopping_between_reflected_setter_effects_observes_actual_asymmetric_order() {
        let mut dom = Dom::new(DomLimits::new(16, 2048, 32, 0, 0, 8, 65536).unwrap()).unwrap();
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
                draft.set_button_target_admitted(
                    key,
                    ButtonTarget::CommandFor,
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
                draft.node(key).unwrap().button_targets.unwrap().command_for,
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
