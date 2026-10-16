//! Actual intrinsic children-changed dispatch at each canonical source step.
use super::*;
impl Draft {
    pub(super) fn children_changed(
        &mut self,
        parent: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.push_effect(
            select::Effect::ChildrenChanged { parent, stage: 0 },
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    pub(super) fn children_changed_effect(
        &mut self,
        parent: Key,
        stage: u8,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        match stage {
            0 => {
                self.push_effect(
                    select::Effect::ChildrenChanged { parent, stage: 1 },
                    self.effect_scoped,
                    cancel,
                )?;
                self.textarea_children_changed(parent, limits, cancel)?;
            }
            1 => {
                self.push_effect(
                    select::Effect::ChildrenChanged { parent, stage: 2 },
                    self.effect_scoped,
                    cancel,
                )?;
                if self.node(parent)?.script.is_some() && self.shadow_connected(parent, cancel)? {
                    self.script_post_connection(parent, cancel)?;
                }
            }
            2 => self.style_update(parent, limits, cancel)?,
            _ => unreachable!("finite children-changed dispatch"),
        }
        Ok(())
    }
}
