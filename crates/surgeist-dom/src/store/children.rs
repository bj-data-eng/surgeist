//! Actual intrinsic children-changed dispatch at each canonical source step.
use super::*;
impl Draft {
    pub(super) fn children_changed(
        &mut self,
        parent: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.textarea_children_changed(parent, limits, cancel)?;
        if self.node(parent)?.script.is_some() && self.shadow_connected(parent, cancel)? {
            self.script_post_connection(parent, cancel)?;
        }
        Ok(())
    }
}
