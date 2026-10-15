//! HTML877d Image Button observation/history and pre-resource guards.
//! Consumers qualify source storage and the explicit disabled-images premise.
use crate::{DocumentString, HtmlInputImageCoordinate};

/// Actual last Image-state source. S belongs to the consumer's storage phase:
/// e.g. its qualified retained-string slot or admitted immutable string value.
/// NeverEntered is distinct from having entered Image without a src attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputImageHistory<S> {
    NeverEntered,
    Last {
        source: Option<S>,
        coordinate: HtmlInputImageCoordinate,
    },
}
impl<S> HtmlInputImageHistory<S> {
    pub fn observed_source(&self) -> Option<Option<&S>> {
        match self {
            Self::NeverEntered => None,
            Self::Last { source, .. } => Some(source.as_ref()),
        }
    }
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
    ExplicitlyDisabled,
}

/// The selected prefix of the Image source update, before URL/request/fetch.
/// An environment request is not a fetch instruction or an enabled-image API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputImageUpdateGuard {
    NoSource,
    EmptySource,
    NeedsImageEnvironment,
    ImagesDisabled,
}
impl HtmlInputImageUpdateGuard {
    /// Observe presence/exact emptiness before requiring any environment fact.
    /// The owner admits the borrowed source/header read before calling.
    pub fn for_source(source: Option<&DocumentString>) -> Self {
        match source {
            None => Self::NoSource,
            Some(source) if source.is_empty() => Self::EmptySource,
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
