//! HTML media-element initialization and resource entry conditions.

use crate::internal::values::DocumentString;
use crate::vocabulary::{KnownHtmlAttribute, KnownHtmlElement};

/// The actual unloaded state of a freshly created media element.
///
/// This state applies before loading or playback can alter the media tuple.
/// Both positions and the default start position are zero, the resource
/// address and tracks are empty, and no poster request or frame exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInitialMediaState {
    /// Fresh Audio state.
    Audio,
    /// Fresh Video state.
    Video,
}

/// A source-defined media resource operation reached by a native change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaResourceEffect {
    /// Run the media element load algorithm after setting src.
    Load,
    /// Run resource selection after insertion of a Source child.
    SelectResource,
    /// Determine the Video poster frame from the specified resource.
    DeterminePoster,
}

impl HtmlInitialMediaState {
    /// The exact built-in family that owns this state.
    pub const fn element(self) -> KnownHtmlElement {
        match self {
            Self::Audio => KnownHtmlElement::Audio,
            Self::Video => KnownHtmlElement::Video,
        }
    }

    /// Initial NETWORK_EMPTY state.
    pub const fn network_is_empty(self) -> bool {
        true
    }

    /// Initial HAVE_NOTHING state.
    pub const fn has_media_data(self) -> bool {
        false
    }

    /// Initial paused flag.
    pub const fn paused(self) -> bool {
        true
    }

    /// The initial show-poster flag. Its rendering applies to Video.
    pub const fn show_poster(self) -> bool {
        true
    }

    /// Initial currentTime in seconds.
    pub const fn current_time(self) -> f64 {
        0.0
    }

    /// Initial currentSrc in UTF-16.
    pub const fn current_src(self) -> &'static [u16] {
        &[]
    }

    /// Resource work required by the actual null-namespace attribute change.
    /// A missing new value represents removal. An empty Video poster leaves
    /// the original null resumption and absent frame in the initial state.
    pub fn attribute_resource_effect(
        self,
        attribute: KnownHtmlAttribute,
        value: Option<&DocumentString>,
    ) -> Option<HtmlMediaResourceEffect> {
        match (self, attribute, value) {
            (_, KnownHtmlAttribute::Src, Some(_)) => Some(HtmlMediaResourceEffect::Load),
            (Self::Video, KnownHtmlAttribute::Poster, Some(value)) if !value.is_empty() => {
                Some(HtmlMediaResourceEffect::DeterminePoster)
            }
            _ => None,
        }
    }
}
