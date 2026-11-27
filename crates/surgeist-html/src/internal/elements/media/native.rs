//! HTML media-element initialization and resource entry conditions.

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

/// HTML mode of a text track, independent of its load readiness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTextTrackMode {
    /// The track is disabled.
    Disabled,
    /// The track is active without showing its cues.
    Hidden,
    /// The track is active and its cues can be shown.
    Showing,
}

/// HTML readiness of an out-of-band text track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTextTrackReadiness {
    /// The track has not started loading.
    NotLoaded,
    /// The track is loading.
    Loading,
    /// The track has loaded successfully.
    Loaded,
    /// The track failed to load.
    FailedToLoad,
}

/// Current state of an admitted media element before loading or playback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlUnloadedMediaState {
    initial: HtmlInitialMediaState,
    can_autoplay: bool,
}

impl HtmlUnloadedMediaState {
    /// Initialize an unloaded Audio or Video with its fresh eligibility.
    pub const fn new(initial: HtmlInitialMediaState) -> Self {
        Self {
            initial,
            can_autoplay: initial.can_autoplay(),
        }
    }

    /// The original unloaded media family and initialization.
    pub const fn initial(self) -> HtmlInitialMediaState {
        self.initial
    }

    /// The current eligibility flag, independent of the autoplay attribute.
    pub const fn can_autoplay(self) -> bool {
        self.can_autoplay
    }

    /// Apply the internal pause steps supported for unloaded media.
    pub const fn pause(&mut self) {
        // This flag changes even when paused is already true. This phase
        // has no pending play promises or playable-data event to settle.
        self.can_autoplay = false;
    }
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

    /// Intrinsic dimensions in CSS pixels for a Video in the unloaded phase.
    /// An unloaded Audio element has no Video dimensions.
    pub const fn video_intrinsic_dimensions(self) -> Option<[u32; 2]> {
        match self {
            Self::Audio => None,
            Self::Video => Some([0, 0]),
        }
    }

    /// Initial paused flag.
    pub const fn paused(self) -> bool {
        true
    }

    /// The initial autoplay eligibility flag.
    pub const fn can_autoplay(self) -> bool {
        true
    }

    /// Whether the initial media state delays the Document's load event.
    pub const fn delays_load_event(self) -> bool {
        false
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
        value: Option<&[u16]>,
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
