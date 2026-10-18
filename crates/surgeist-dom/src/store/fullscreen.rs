//! Fullscreen7c38 model transitions through the original canonical owner.
use super::*;
use crate::DomFullscreenModelStep as Step;
use crate::dialog::DocumentInteractiveState;
pub(in crate::store) mod exit;

impl Draft {
    pub(in crate::store) fn fullscreen_raw_members(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<&[Key], DomError> {
        self.source_step(cancel)?;
        let node = self.node(document)?;
        if !matches!(node.kind, DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        match &node.interactive_document {
            Some(DocumentInteractiveState::Known { top_layer, .. }) => Ok(top_layer),
            _ => Err(DomError::FullscreenModelUnavailable {
                step: Step::ReadTopLayer,
            }),
        }
    }
    pub(in crate::store) fn fullscreen_model_target(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        for &key in self.fullscreen_raw_members(document, cancel)?.iter().rev() {
            self.source_step(cancel)?;
            if self.element(key)?.fullscreen_flag() {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }
    pub(super) fn fullscreen_element_model(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.element(node)?;
        let document = self.node(node)?.document;
        if !self.known_empty_popover_lists(document, cancel)? {
            return Err(DomError::FullscreenModelUnavailable {
                step: Step::ReadTopLayer,
            });
        }
        // Actual empty showing lists supply topmost=null and zero hide iterations.
        self.source_step(cancel)?;
        self.element_mut(node).set_fullscreen_flag();
        self.remove_top_layer_immediately(node, cancel)?;
        self.add_top_layer(node, cancel)
    }
    pub(super) fn unfullscreen_element_model(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.element(node)?;
        self.fullscreen_raw_members(self.node(node)?.document, cancel)?;
        self.element_mut(node).clear_fullscreen_flags();
        self.remove_top_layer_immediately(node, cancel)
    }
    pub(super) fn unfullscreen_document_model(
        &mut self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut flagged = self.effect_vec();
        for &key in self.fullscreen_raw_members(document, cancel)? {
            self.source_step(cancel)?;
            if self.element(key)?.fullscreen_flag() {
                self.select_reserve(&mut flagged, 1, cancel)?;
                flagged.push(key);
            }
        }
        for &key in flagged.iter() {
            self.unfullscreen_element_model(key, cancel)?;
        }
        Ok(())
    }
    pub(super) fn fullscreen_removing_members(
        &mut self,
        root: Key,
        cancel: &AtomicBool,
    ) -> Result<select::ScratchVec<Key>, DomError> {
        let mut pending = self.effect_vec();
        let mut flagged = self.effect_vec();
        self.select_reserve(&mut pending, 1, cancel)?;
        pending.push(root);
        while let Some(key) = pending.pop() {
            self.source_step(cancel)?;
            let node = self.node(key)?;
            if matches!(&node.kind, DomNodeKind::Element(element) if element.fullscreen_flag()) {
                self.select_reserve(&mut flagged, 1, cancel)?;
                flagged.push(key);
            }
            self.select_reserve(&mut pending, node.children.len(), cancel)?;
            for &child in node.children.iter().rev() {
                self.source_step(cancel)?;
                pending.push(child);
            }
            if let Some(shadow) = node.shadow_root {
                self.select_reserve(&mut pending, 1, cancel)?;
                pending.push(shadow);
            }
        }
        Ok(flagged)
    }
    pub(super) fn retire_fullscreen_membership(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if matches!(self.node(key)?.kind, DomNodeKind::Element(_)) {
            let document = self.node(key)?.document;
            if matches!(
                self.node(document)?.interactive_document,
                Some(DocumentInteractiveState::Known { .. })
            ) {
                self.remove_top_layer_immediately(key, cancel)?;
            }
        }
        Ok(())
    }
    /// Complete actual adoption closure qualification before any node-document
    /// write. Early scans skip only nodes whose real removing walk will run.
    pub(super) fn preflight_top_layer_adoption(
        &self,
        root: Key,
        document: Key,
        early: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        // Only the actual adoption root has this whole-call fast return.
        // Matching intermediate node-documents do not prune its descendants.
        if self.node(root)?.document == document {
            return Ok(());
        }
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1, cancel)?;
        pending.push((
            root,
            Some(document),
            early && self.node(root)?.parent.is_some(),
        ));
        while let Some((key, target, removal_covers)) = pending.pop() {
            self.source_step(cancel)?;
            let node = self.node(key)?;
            if target != Some(node.document)
                && !removal_covers
                && matches!(node.kind, DomNodeKind::Element(_))
                && let Some(DocumentInteractiveState::Known {
                    top_layer,
                    pending_top_layer_removals,
                    ..
                }) = &self.node(node.document)?.interactive_document
            {
                for &member in top_layer.iter().chain(pending_top_layer_removals) {
                    self.source_step(cancel)?;
                    if member == key {
                        return Err(DomError::TopLayerAdoptionUnavailable {
                            subject: self.published_id(key),
                            old_document: self.published_id(node.document).map(DocumentId),
                        });
                    }
                }
            }
            self.select_reserve(&mut pending, node.children.len(), cancel)?;
            for &child in node.children.iter().rev() {
                self.source_step(cancel)?;
                pending.push((child, target, removal_covers));
            }
            if let Some(shadow) = node.shadow_root {
                self.select_reserve(&mut pending, 1, cancel)?;
                pending.push((shadow, target, removal_covers));
            }
            if let Some(contents) = self.owned_template_contents(key, cancel)? {
                // None means an actual future fresh inert owner, not unknown
                // history. Its identity cannot equal a current node-document.
                let target = target
                    .map(|target| self.template_adoption_owner(target, cancel))
                    .transpose()?
                    .flatten();
                // The actual Template adapter separately adopts contents only
                // when its selected appropriate owner differs. Unlike an
                // intermediate ordinary/shadow node, a matching contents root
                // therefore excludes this entire separate adoption call.
                if target != Some(self.node(contents)?.document) {
                    self.select_reserve(&mut pending, 1, cancel)?;
                    pending.push((contents, target, false));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

/// Derived Document facts use the same raw membership as the public model
/// getter. This comparison adds explicit Document invalidation for flag-only
/// writes without storing a second canonical designation.
pub(super) fn captured_facts(
    state: &State,
    document: &Node,
    owner: DomOwnerId,
    budget: Option<&source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<Option<(Option<NodeId>, bool, bool)>, DomError> {
    let Some(DocumentInteractiveState::Known { top_layer, .. }) = &document.interactive_document
    else {
        return Ok(None);
    };
    let mut target = None;
    let mut count = 0u8;
    let mut locked = false;
    for &id in top_layer {
        source::formation::step(budget, cancel)?;
        let DomNodeKind::Element(element) = &state.node(owner, id)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        if element.fullscreen_flag() {
            target = Some(id);
            count = (count + 1).min(2);
            locked = element.keyboard_lock_flag();
        }
    }
    Ok(Some((target, count == 1, locked)))
}
