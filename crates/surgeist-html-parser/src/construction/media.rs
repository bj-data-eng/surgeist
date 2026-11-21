//! Parser-created media state at original creation, mutation and stack exits.

use super::form_effects::{FormEffectStage, FormEffects};
use super::*;
use surgeist_html::{
    elements::media::{HtmlInitialMediaState, HtmlMediaResourceEffect},
    vocabulary::{KnownHtmlAttribute, KnownHtmlElement},
};

/// The first reached media operation outside the unloaded parser profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseMediaUnavailable {
    Resource(HtmlMediaResourceEffect),
    TextTrack,
    DocumentRemoval,
}

/// Observed parser lifecycle for an original HTML media element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseMediaDisposition {
    AwaitingStackExit,
    /// Automatic selection ran over the empty track list. The pending list is
    /// empty and the original parser block has been cleared.
    Unloaded,
    Unavailable(HtmlParseMediaUnavailable),
}

/// Source-qualified state for an Audio or Video created by this parser.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlParseMedia {
    initial: HtmlInitialMediaState,
    disposition: HtmlParseMediaDisposition,
}

impl HtmlParseMedia {
    pub const fn initial(self) -> HtmlInitialMediaState {
        self.initial
    }

    pub const fn disposition(self) -> HtmlParseMediaDisposition {
        self.disposition
    }
}

#[cfg(test)]
impl ParserArenaSession {
    pub(crate) fn observed_parser_media_for_test(
        &self,
        node: &ParserNode,
    ) -> Option<HtmlParseMedia> {
        let storage = self.active().ok()?;
        let record = storage.construction.node(&node.handle).ok()?;
        let media = storage.payloads.facts.media.as_ref()?;
        let (index, _) = media.find_published(record.position());
        media.row(index?).copied()
    }
}

impl SessionStorage {
    pub(super) fn register_parser_media(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        tag: Option<KnownHtmlElement>,
    ) -> Result<Option<HtmlInitialMediaState>, SessionError> {
        let initial = match tag {
            Some(KnownHtmlElement::Audio) => HtmlInitialMediaState::Audio,
            Some(KnownHtmlElement::Video) => HtmlInitialMediaState::Video,
            _ => return Ok(None),
        };
        if self.payloads.facts.media.is_none() {
            payload_write::<Option<FamilyTable<HtmlParseMedia>>>(control, 1)?;
            self.payloads.facts.media = Some(FamilyTable::new(control)?);
        }
        let media = self
            .payloads
            .facts
            .media
            .as_mut()
            .ok_or_else(|| latch_registry_row(control))?;
        media.admit(control, 1)?;
        media
            .push(
                node,
                HtmlParseMedia {
                    initial,
                    disposition: HtmlParseMediaDisposition::AwaitingStackExit,
                },
            )
            .map_err(|_| latch_registry_row(control))?;
        Ok(Some(initial))
    }

    /// Use the exact attribute that the original creation just appended.
    pub(super) fn parser_media_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        initial: HtmlInitialMediaState,
        position: usize,
    ) -> Result<(), SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(latch_registry_row(control));
        };
        let attr = element
            .attributes()
            .at(control, position)?
            .ok_or_else(|| latch_registry_row(control))?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if attr.name().namespace().uri().is_some() {
            return Ok(());
        }
        let name = if exact_ascii(control, Some(attr.name().local_name()), "src")? {
            KnownHtmlAttribute::Src
        } else if initial == HtmlInitialMediaState::Video
            && exact_ascii(control, Some(attr.name().local_name()), "poster")?
        {
            KnownHtmlAttribute::Poster
        } else {
            return Ok(());
        };
        if let Some(effect) =
            initial.attribute_resource_effect(name, Some(attr.value().units(control)?))
        {
            self.mark_parser_media_unavailable(
                control,
                node,
                HtmlParseMediaUnavailable::Resource(effect),
            )?;
        }
        Ok(())
    }

    pub(super) fn mark_parser_media_unavailable(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        reason: HtmlParseMediaUnavailable,
    ) -> Result<(), SessionError> {
        let Some(index) = self.parser_media_index(control, node)? else {
            return Ok(());
        };
        self.mark_parser_media_at(control, index, reason)
    }

    fn mark_parser_media_at(
        &mut self,
        control: &mut ParseControl,
        index: usize,
        reason: HtmlParseMediaUnavailable,
    ) -> Result<(), SessionError> {
        let row = self
            .payloads
            .facts
            .media
            .as_mut()
            .and_then(|media| media.row_mut(index))
            .ok_or_else(|| latch_registry_row(control))?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if !matches!(row.disposition, HtmlParseMediaDisposition::Unavailable(_)) {
            payload_write::<HtmlParseMediaDisposition>(control, 1)?;
            row.disposition = HtmlParseMediaDisposition::Unavailable(reason);
        }
        Ok(())
    }

    pub(super) fn parser_media_stack_exit(
        &mut self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        let Some(media) = self.payloads.facts.media.as_mut() else {
            return Ok(());
        };
        let Some(index) = media.find(control, node)? else {
            return Ok(());
        };
        let row = media
            .row_mut(index)
            .ok_or_else(|| latch_registry_row(control))?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if row.disposition == HtmlParseMediaDisposition::AwaitingStackExit {
            // Empty track lists make each of the three automatic preference
            // groups and the pending-track population an actual empty check.
            control.charge(HtmlParseWorkCategory::StateStep, 4)?;
            payload_write::<HtmlParseMediaDisposition>(control, 1)?;
            row.disposition = HtmlParseMediaDisposition::Unloaded;
        }
        Ok(())
    }

    pub(super) fn has_parser_media(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<bool, SessionError> {
        Ok(self.parser_media_index(control, node)?.is_some())
    }

    /// The original Source insertion hook has already qualified its Media parent.
    pub(super) fn parser_media_source_inserted(
        &mut self,
        control: &mut ParseControl,
        parent: usize,
    ) -> Result<(), SessionError> {
        if let Some(index) = self.parser_media_index(control, parent)?
            && self.form_attribute(control, parent, "src")?.is_none()
        {
            self.mark_parser_media_at(
                control,
                index,
                HtmlParseMediaUnavailable::Resource(HtmlMediaResourceEffect::SelectResource),
            )?;
        }
        Ok(())
    }

    fn parser_media_index(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        let Some(media) = self.payloads.facts.media.as_ref() else {
            return Ok(None);
        };
        media.find(control, node)
    }

    pub(super) fn parser_media_tree_changed(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        effects: &mut FormEffects,
    ) -> Result<(), SessionError> {
        if self.payloads.facts.media.is_none() {
            return Ok(());
        }
        match effects.stage {
            FormEffectStage::Removal => {
                let Some(index) = self.parser_media_index(control, node)? else {
                    return Ok(());
                };
                let from_document = match effects.media_removal_from_document {
                    Some(from_document) => from_document,
                    None => {
                        let parent = effects
                            .old_parent
                            .ok_or_else(|| latch_registry_row(control))?;
                        let from_document = self.connected(control, parent, None)?;
                        payload_write::<Option<bool>>(control, 1)?;
                        effects.media_removal_from_document = Some(from_document);
                        from_document
                    }
                };
                if from_document {
                    self.mark_parser_media_at(
                        control,
                        index,
                        HtmlParseMediaUnavailable::DocumentRemoval,
                    )?;
                }
            }
            FormEffectStage::Insertion
                if node == effects.root && self.html_local(control, node, "track")? =>
            {
                let parent = effects
                    .new_parent
                    .ok_or_else(|| latch_registry_row(control))?;
                self.mark_parser_media_unavailable(
                    control,
                    parent,
                    HtmlParseMediaUnavailable::TextTrack,
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
