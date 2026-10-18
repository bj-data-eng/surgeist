//! Checked canonical domain counts. A changed record replaces its own fixed
//! contribution; unrelated records never participate in local admission.

use super::super::*;
use super::NodeRead;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::store) struct Counts {
    pub live_ranges: usize,
    pub base_observations: usize,
    pub encoding_observations: usize,
    pub meta_color_scheme_bytes: usize,
    pub fullscreen_events: usize,
    pub autofocus_candidates: usize,
    pub slot_references: usize,
    pub select_tasks: usize,
    pub details_intentions: usize,
    pub transitions: usize,
    pub records: usize,
    pub pseudos: usize,
    pub templates: usize,
    pub shadow_roots: usize,
    pub slots: usize,
}

impl Counts {
    pub fn node(node: NodeRead<'_>) -> Result<Self, DomError> {
        Ok(Self {
            base_observations: node
                .base_document
                .as_ref()
                .and_then(|state| state.facts)
                .map_or(0, |facts| facts.formation_usage().record_capacity()),
            encoding_observations: node
                .encoding_url_document
                .map_or(0, crate::hyperlink::EncodingUrlDocument::records),
            meta_color_scheme_bytes: match node.kind {
                DomNodeKind::Document(metadata) => metadata.meta_color_scheme_bytes(),
                _ => 0,
            },
            fullscreen_events: node.fullscreen_events.len(),
            autofocus_candidates: match node.autofocus_document {
                Some(super::AutofocusRead::Known { candidates, .. }) => candidates.len(),
                _ => 0,
            },
            slot_references: node
                .manual_nodes
                .len()
                .checked_add(node.assigned_nodes.len())
                .ok_or(DomError::RepresentationExhausted)?,
            templates: usize::from(
                node.template_contents.is_some()
                    || node.template_document.is_some_and(|state| {
                        matches!(state, crate::template::TemplateDocument::InertTemplate)
                            || state.inert().is_some()
                    }),
            ),
            shadow_roots: usize::from(matches!(node.kind, DomNodeKind::ShadowRoot(_))),
            slots: usize::from(crate::slot::is_slot(node.kind)),
            ..Self::default()
        })
    }

    pub fn checked_add(self, other: Self) -> Result<Self, DomError> {
        macro_rules! add {
            ($field:ident) => {
                self.$field
                    .checked_add(other.$field)
                    .ok_or(DomError::RepresentationExhausted)?
            };
        }
        Ok(Self {
            live_ranges: add!(live_ranges),
            base_observations: add!(base_observations),
            encoding_observations: add!(encoding_observations),
            meta_color_scheme_bytes: add!(meta_color_scheme_bytes),
            fullscreen_events: add!(fullscreen_events),
            autofocus_candidates: add!(autofocus_candidates),
            slot_references: add!(slot_references),
            select_tasks: add!(select_tasks),
            details_intentions: add!(details_intentions),
            transitions: add!(transitions),
            records: add!(records),
            pseudos: add!(pseudos),
            templates: add!(templates),
            shadow_roots: add!(shadow_roots),
            slots: add!(slots),
        })
    }

    pub fn checked_replace(self, old: Self, new: Self) -> Result<Self, DomError> {
        macro_rules! replace {
            ($field:ident) => {
                self.$field
                    .checked_sub(old.$field)
                    .and_then(|value| value.checked_add(new.$field))
                    .ok_or(DomError::RepresentationExhausted)?
            };
        }
        Ok(Self {
            live_ranges: replace!(live_ranges),
            base_observations: replace!(base_observations),
            encoding_observations: replace!(encoding_observations),
            meta_color_scheme_bytes: replace!(meta_color_scheme_bytes),
            fullscreen_events: replace!(fullscreen_events),
            autofocus_candidates: replace!(autofocus_candidates),
            slot_references: replace!(slot_references),
            select_tasks: replace!(select_tasks),
            details_intentions: replace!(details_intentions),
            transitions: replace!(transitions),
            records: replace!(records),
            pseudos: replace!(pseudos),
            templates: replace!(templates),
            shadow_roots: replace!(shadow_roots),
            slots: replace!(slots),
        })
    }

    pub fn admit(
        self,
        ranges: usize,
        nodes: usize,
        units: usize,
        limits: DomLimits,
    ) -> Result<(), DomError> {
        for (used, limit, resource) in [
            (ranges, limits.ranges, DomResource::Ranges),
            (nodes, limits.nodes, DomResource::ManagedNodes),
            (units, limits.units, DomResource::StringCodeUnits),
            (
                self.base_observations,
                limits.base_observations,
                DomResource::BaseObservations,
            ),
            (
                self.encoding_observations,
                limits.encoding_url_observations,
                DomResource::EncodingUrlObservations,
            ),
            (
                self.meta_color_scheme_bytes,
                limits.meta_color_scheme_bytes,
                DomResource::MetaColorSchemeRetainedBytes,
            ),
            (
                self.fullscreen_events,
                limits.fullscreen_events,
                DomResource::FullscreenEvents,
            ),
            (
                self.autofocus_candidates,
                limits.autofocus_candidates,
                DomResource::AutofocusCandidates,
            ),
            (
                self.slot_references,
                limits.slot_references,
                DomResource::SlotReferences,
            ),
            (
                self.select_tasks,
                limits.select_tasks,
                DomResource::SelectTasks,
            ),
            (
                self.details_intentions,
                limits.details_intentions,
                DomResource::DetailsToggleIntentions,
            ),
            (
                self.transitions,
                limits.transitions,
                DomResource::ViewTransitions,
            ),
            (
                self.records,
                limits.records,
                DomResource::ViewTransitionRecords,
            ),
            (
                self.pseudos,
                limits.pseudos,
                DomResource::ViewTransitionPseudos,
            ),
        ] {
            super::super::admit(used, limit, resource)?;
        }
        Ok(())
    }
}
