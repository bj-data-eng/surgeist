use crate::internal::{
    attr::DomNamedNodeMap, element::NativeFamily, identity::NodeId, snapshot::DomNode,
    store::DomError,
};
use surgeist_html::{
    attributes::HtmlTrackKind,
    bounded::attributes::track_kind_with,
    elements::media::{HtmlTextTrackMode, HtmlTextTrackReadiness},
    values::DocumentString,
    vocabulary::KnownHtmlAttribute,
};

/// Stable identity of the one out-of-band text track associated with a
/// canonical Track element. Adoption preserves it. Cloning creates a new one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomOutOfBandTextTrackId(NodeId);

/// A Track's out-of-band text track captured at the same revision as its
/// element. This native profile retains disabled, unloaded tracks with no cues.
#[derive(Clone, Copy, Debug)]
pub struct DomOutOfBandTextTrack<'a> {
    id: DomOutOfBandTextTrackId,
    attributes: DomNamedNodeMap<'a>,
    url: Option<&'a DocumentString>,
}

impl<'a> DomNode<'a> {
    /// The associated text track, for an admitted native HTML Track element.
    pub fn out_of_band_text_track(self) -> Option<DomOutOfBandTextTrack<'a>> {
        let NativeFamily::Track(url) = self.as_element()?.native_family() else {
            return None;
        };
        Some(DomOutOfBandTextTrack {
            id: DomOutOfBandTextTrackId(self.id()),
            attributes: self.attributes()?,
            url: url.as_ref().map(|value| &value.value),
        })
    }
}

impl<'a> DomOutOfBandTextTrack<'a> {
    /// The associated identity, independent of the captured revision.
    pub fn id(self) -> DomOutOfBandTextTrackId {
        self.id
    }

    /// The mode of this admitted text track.
    pub fn mode(self) -> HtmlTextTrackMode {
        HtmlTextTrackMode::Disabled
    }

    /// The load readiness of this admitted text track.
    pub fn readiness(self) -> HtmlTextTrackReadiness {
        HtmlTextTrackReadiness::NotLoaded
    }

    /// Number of loaded cues.
    pub fn cue_count(self) -> usize {
        0
    }

    /// The serialized Track URL from the last admitted setting of src.
    /// Reading it never parses the raw src or consults the current Document.
    pub fn url(self) -> &'a [u16] {
        self.url.map_or(&[], DocumentString::code_units)
    }

    /// Select the kind from the canonical attribute using caller admission.
    pub fn kind_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlTrackKind, E> {
        let raw = self.attribute_with(KnownHtmlAttribute::Kind, &mut step)?;
        track_kind_with(raw, |_| step())
    }

    /// Read the current label, or the empty string when absent.
    pub fn label_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<&'a [u16], E> {
        self.attribute_with(KnownHtmlAttribute::Label, step)
            .map(Option::unwrap_or_default)
    }

    /// Read the current language, or the empty string when absent.
    pub fn language_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<&'a [u16], E> {
        self.attribute_with(KnownHtmlAttribute::Srclang, step)
            .map(Option::unwrap_or_default)
    }

    /// Read the current identifier, or the empty string when absent.
    pub fn identifier_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<&'a [u16], E> {
        self.attribute_with(KnownHtmlAttribute::Id, step)
            .map(Option::unwrap_or_default)
    }

    fn attribute_with<E: From<DomError>>(
        self,
        name: KnownHtmlAttribute,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<&'a [u16]>, E> {
        let result = self.attributes.find_with(step, |attribute, step| {
            let stored = attribute.name();
            let name_matches = stored.namespace().is_none();
            let actual = stored.local_name().code_units();
            let expected = name.local_name().as_bytes();
            if !name_matches || actual.len() != expected.len() {
                return Ok(false);
            }
            for (&unit, &byte) in actual.iter().zip(expected) {
                step()?;
                if unit != u16::from(byte) {
                    return Ok(false);
                }
            }
            Ok(true)
        })?;
        Ok(result.map(|attribute| attribute.value().code_units()))
    }
}
