//! HTML877d Image Button observation/history and pre-resource guards.
//! Consumers qualify source storage and the explicit disabled-images premise.
use crate::internal::elements::input::HtmlInputImageCoordinate;

/// Actual last Image-state source. S belongs to the consumer's storage phase:
/// e.g. its qualified retained-string slot or admitted immutable string value.
/// NeverEntered is distinct from having entered Image without a src attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputImageHistory<S> {
    /// The control has never been in the Image Button state, so it has observed no source.
    NeverEntered,
    /// The control has been in the Image Button state and last observed the given source, with the
    /// selected coordinate retained from then.
    Last {
        /// The `src` attribute value last observed, where `None` means the attribute was absent.
        /// The consumer decides what `S` stores.
        source: Option<S>,
        /// The selected coordinate retained with that observation.
        coordinate: HtmlInputImageCoordinate,
    },
}
impl<S> HtmlInputImageHistory<S> {
    /// The last observed source. The outer `None` means the control never entered Image;
    /// `Some(None)` means it entered with no `src` attribute, which is distinct.
    pub fn observed_source(&self) -> Option<Option<&S>> {
        match self {
            Self::NeverEntered => None,
            Self::Last { source, .. } => Some(source.as_ref()),
        }
    }
    /// The retained selected coordinate, which is the initial coordinate until the control has
    /// entered the Image Button state.
    pub fn coordinate(&self) -> HtmlInputImageCoordinate {
        match self {
            Self::NeverEntered => HtmlInputImageCoordinate::initial(),
            Self::Last { coordinate, .. } => *coordinate,
        }
    }
    /// First entry always observes the actual optional source. Later re-entry
    /// observes it only if presence or exact value differs. The consuming owner
    /// performs its admitted exact UTF-16 comparison; it is called only for the
    /// two present sources and its cancellation/admission error propagates.
    ///
    /// This is not the direct-src-set rule. An applicable src set/change while
    /// already Image runs its source guard and records even an equal value.
    pub fn entering_requires_observation<E>(
        &self,
        current: Option<&S>,
        equal: impl FnOnce(&S, &S) -> Result<bool, E>,
    ) -> Result<bool, E> {
        match self {
            Self::NeverEntered => Ok(true),
            Self::Last { source, .. } => match (source.as_ref(), current) {
                (None, None) => Ok(false),
                (Some(previous), Some(current)) => equal(previous, current).map(|same| !same),
                _ => Ok(true),
            },
        }
    }
    /// Record only after the source update/guard succeeds and the owner has
    /// admitted/copied its retained source. Preserve coordinates, including on
    /// absent/empty src changes. No string copy/allocation is hidden here.
    pub fn with_observed_source<T>(&self, source: Option<T>) -> HtmlInputImageHistory<T> {
        HtmlInputImageHistory::Last {
            source,
            coordinate: self.coordinate(),
        }
    }
}

/// The precise source premise “support for images has been disabled”.
/// This value alone does not qualify a document/environment or establish this
/// policy: the consuming boundary must supply actual cause-qualified evidence.
/// It is independent of no-BC, inactivity, scripting or missing loader support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputImagesDisabled {
    /// Support for images is explicitly disabled, which is the source premise the guard needs to
    /// decide `ImagesDisabled`.
    ExplicitlyDisabled,
}

/// The selected prefix of the Image source update, before URL/request/fetch.
/// An environment request is not a fetch instruction or an enabled-image API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputImageUpdateGuard {
    /// There is no `src` attribute, so the source update stops before any request.
    NoSource,
    /// The `src` attribute is present and is the empty string, so the update stops before any
    /// request.
    EmptySource,
    /// A nonempty `src` is present, so the update needs the image environment's facts to continue;
    /// no decision is possible yet.
    NeedsImageEnvironment,
    /// A nonempty `src` is present and images are explicitly disabled, so the update stops before
    /// any request.
    ImagesDisabled,
}
impl HtmlInputImageUpdateGuard {
    /// Observe presence/exact emptiness of the `src` attribute's UTF-16 code units before
    /// requiring any environment fact.
    /// The owner admits the borrowed source/header read before calling.
    pub fn for_source(source: Option<&[u16]>) -> Self {
        match source {
            None => Self::NoSource,
            Some([]) => Self::EmptySource,
            Some(_) => Self::NeedsImageEnvironment,
        }
    }
    /// Discharge the environment guard only with the established disabled fact.
    /// Existing source/empty returns remain prior independent reasons.
    pub fn with_disabled_images(self, _: HtmlInputImagesDisabled) -> Self {
        match self {
            Self::NeedsImageEnvironment => Self::ImagesDisabled,
            stopped => stopped,
        }
    }
}
