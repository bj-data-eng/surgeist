//! Freezing and releasing the active session storage.

use super::*;
use surgeist_html::construction::ConstructionNodeKind;

impl SessionStorage {
    pub(super) fn prepare_freeze(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<usize>, SessionError> {
        control.require_identity(&self.control)?;
        self.effects.require_freezable(control)?;
        self.require(control)?;
        self.validate_template_facts(control)?;
        self.validate_svg_facts(control)?;
        self.validate_completion_facts(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let document = matches!(
            self.construction
                .node_at(0)
                .ok_or(SessionError::ForeignNode)?
                .kind(),
            ConstructionNodeKind::Document(_)
        );
        if let Some(fragment) = &self.fragment {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let output = fragment.output.ok_or(SessionError::ContextUnavailable)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if !matches!(
                self.construction
                    .node_at(output)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                ConstructionNodeKind::Fragment
            ) {
                return Err(SessionError::WrongToken);
            }
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<Option<usize>>() as u64,
            )?;
            Ok(Some(output))
        } else {
            Ok(if document { None } else { Some(0) })
        }
    }

    /// Called only after the owning transition verifies cleanup identity. This
    /// consumes actual storage; exact private tickets prove the release invariant.
    pub(super) fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        let Self {
            csp_slot: _,
            control: owner,
            effects,
            evidence,
            fragment,
            #[cfg(test)]
                cancel_after_radio_peer: _,
            #[cfg(test)]
                cancel_after_removal: _,
            #[cfg(test)]
                cancel_after_textarea_reset: _,
            #[cfg(test)]
                cancel_after_output_reset: _,
            #[cfg(test)]
                cancel_after_input_reset: _,
            #[cfg(test)]
                lose_association_row: _,
            #[cfg(test)]
                cancel_after_range_append: _,
            #[cfg(test)]
                cancel_after_range_ingress: _,
            standalone_ineligibility: _,
            construction,
            holds,
            payloads,
            appended_units: _,
            style_order_floor: _,
            token_end: _,
            handler_starts: _,
            main_tree_styleless: _,
            style_subtree: _,
            detached_order: _,
            meta_batches: _,
        } = self;
        // Every owner is released even after a refusal; the first one is
        // returned at the end.
        let mut refused = None;
        if let Some(fragment) = fragment {
            keep_first(&mut refused, fragment.release(control, &owner));
        }
        if let Some(evidence) = evidence {
            keep_first(&mut refused, evidence.release(control));
        }
        keep_first(&mut refused, effects.release(control));
        keep_first(
            &mut refused,
            release_parts(
                control,
                &owner,
                ReleasedStorage {
                    construction,
                    holds,
                    payloads,
                },
            ),
        );
        refused.map_or(Ok(()), Err)
    }
}
