//! HTML877d standalone Img initialization and pre-resource mutation decisions.
//! Canonical identities, invocation order, waiting work and resource effects
//! remain with the consuming owner. These are not Input Image Button rules.

use crate::{
    DocumentString, HtmlCrossOriginState, HtmlLoading, HtmlReferrerPolicy, KnownHtmlAttribute,
    KnownHtmlElement, Namespace,
};
use std::{cmp::Ordering, error::Error, fmt};

/// The request state supported by fresh images before their resource continuation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImgFreshRequestState {
    Unavailable,
}

/// An actual fresh image request, before resource selection or acquisition.
/// This closed phase cannot represent retained, loaded or broken requests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlImgUnavailableRequest {
    url: DocumentString,
}
impl HtmlImgUnavailableRequest {
    pub const fn state(&self) -> HtmlImgFreshRequestState {
        HtmlImgFreshRequestState::Unavailable
    }
    pub fn current_url(&self) -> &DocumentString {
        &self.url
    }
    pub const fn current_pixel_density(&self) -> u32 {
        1
    }
    pub const fn preferred_dimensions_are_null(&self) -> bool {
        true
    }
    pub const fn image_data_is_available(&self) -> bool {
        false
    }
}

/// The closed fresh Img tuple (HTML #images-processing-model).
///
/// Creation and DOM cloning initialize this value; copying a transaction or
/// snapshot preserves it. Adoption does not reinitialize it. The caller stores
/// the actual dimension-attribute-source identity, initially this Img itself,
/// and its independent form owner/parser flag. No loaded resources, geometry,
/// request callbacks or retained-state admission are provided by this type.
/// While an update waits at the first inactive-Document guard, this tuple stays
/// unchanged; that waiting invocation is separate consumer-owned state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlImgFreshState {
    current: HtmlImgUnavailableRequest,
}
impl HtmlImgFreshState {
    pub fn fresh() -> Self {
        Self {
            current: HtmlImgUnavailableRequest { url: "".into() },
        }
    }
    pub fn current_request(&self) -> &HtmlImgUnavailableRequest {
        &self.current
    }
    pub const fn pending_request_is_null(&self) -> bool {
        true
    }
    pub const fn last_selected_source_is_null(&self) -> bool {
        true
    }
    pub const fn last_auto_sizes_width_is_null(&self) -> bool {
        true
    }
    pub const fn lazy_load_resumption_is_null(&self) -> bool {
        true
    }
    pub const fn dimension_attribute_source_is_self(&self) -> bool {
        true
    }
}

/// Exact flags of one invocation, never a union of superseded invocations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlImageUpdateFlags {
    restart_animation: bool,
    maybe_omit_events: bool,
}
impl HtmlImageUpdateFlags {
    pub const fn new(restart_animation: bool, maybe_omit_events: bool) -> Self {
        Self {
            restart_animation,
            maybe_omit_events,
        }
    }
    pub const fn restart_animation(self) -> bool {
        self.restart_animation
    }
    pub const fn maybe_omit_events(self) -> bool {
        self.maybe_omit_events
    }
}

/// Actual node-Document activity, independent of image support or scripting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageDocumentActivity {
    FullyActive,
    NotFullyActive,
    Unavailable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageUpdateStartDecision {
    /// Suspend before image support, abort/reset, source selection or fetching.
    WaitForFullyActive,
    /// Only now inspect actual image support; this performs no later step.
    CheckImageSupport,
    NeedsFullyActiveFact,
}
/// The FIRST step of HTML #updating-the-image-data. In particular an inactive
/// Document cannot jump to the later images-disabled abort/reset branch.
pub const fn image_update_start(
    activity: HtmlImageDocumentActivity,
) -> HtmlImageUpdateStartDecision {
    match activity {
        HtmlImageDocumentActivity::NotFullyActive => {
            HtmlImageUpdateStartDecision::WaitForFullyActive
        }
        HtmlImageDocumentActivity::FullyActive => HtmlImageUpdateStartDecision::CheckImageSupport,
        HtmlImageDocumentActivity::Unavailable => {
            HtmlImageUpdateStartDecision::NeedsFullyActiveFact
        }
    }
}

/// An invocation cannot have started after the owner's latest-started mark.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlImageUpdateOrderError;
impl fmt::Display for HtmlImageUpdateOrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("image invocation is later than latest-started invocation")
    }
}
impl Error for HtmlImageUpdateOrderError {}

/// The wait's post-activation supersession condition. Compare the invocation
/// with the owner's latest STARTED invocation, not its latest pending one. A
/// later invocation still supersedes when it aborted or no longer waits.
///
/// The caller owns checked nonwrapping identity/order and the high-water mark.
/// This also justifies compacting a wholly inactive older wait under the
/// selected closed-wait policy; it does not claim activation, source return,
/// abort, microtask delivery or any completed resource effect.
pub const fn image_update_is_superseded(
    invocation_compared_to_latest_started: Ordering,
) -> Result<bool, HtmlImageUpdateOrderError> {
    match invocation_compared_to_latest_started {
        Ordering::Less => Ok(true),
        Ordering::Equal => Ok(false),
        Ordering::Greater => Err(HtmlImageUpdateOrderError),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageAttributeChange {
    /// Actual exact UTF-16 equality, admitted and compared by the caller.
    Set {
        same_value: bool,
    },
    Removed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageAttributeEffect {
    None,
    Update(HtmlImageUpdateFlags),
    CompareCrossOrigin,
    CompareReferrerPolicy,
    CompareLoading,
}
/// Actual Img attribute-change dispatch (#reacting-to-dom-mutations). Only an
/// equal-value src SET carries the explicit restart-animation flag. Removal
/// here means actual removal, not an absent-attribute no-op or copied history.
pub fn image_attribute_effect(
    namespace: &Namespace,
    attribute: KnownHtmlAttribute,
    change: HtmlImageAttributeChange,
) -> HtmlImageAttributeEffect {
    if namespace.uri().is_some() {
        return HtmlImageAttributeEffect::None;
    }
    match attribute {
        KnownHtmlAttribute::Src => HtmlImageAttributeEffect::Update(HtmlImageUpdateFlags::new(
            matches!(change, HtmlImageAttributeChange::Set { same_value: true }),
            false,
        )),
        KnownHtmlAttribute::Srcset | KnownHtmlAttribute::Sizes => {
            HtmlImageAttributeEffect::Update(HtmlImageUpdateFlags::new(false, false))
        }
        KnownHtmlAttribute::Crossorigin => HtmlImageAttributeEffect::CompareCrossOrigin,
        KnownHtmlAttribute::Referrerpolicy => HtmlImageAttributeEffect::CompareReferrerPolicy,
        KnownHtmlAttribute::Loading => HtmlImageAttributeEffect::CompareLoading,
        _ => HtmlImageAttributeEffect::None,
    }
}
/// Compare actual states from the existing crossorigin interpreter, not text.
pub fn image_crossorigin_changed(old: HtmlCrossOriginState, new: HtmlCrossOriginState) -> bool {
    old != new
}
/// Compare actual states from the existing referrerpolicy interpreter.
pub fn image_referrerpolicy_changed(old: HtmlReferrerPolicy, new: HtmlReferrerPolicy) -> bool {
    old != new
}
/// On true, clear actual non-null resumption steps BEFORE invoking them. Null
/// returns without work (#attr-img-loading); no callback is created here.
pub const fn image_loading_became_eager(old: HtmlLoading, new: HtmlLoading) -> bool {
    matches!((old, new), (HtmlLoading::Lazy, HtmlLoading::Eager))
}
/// Whether an actual Source attribute change affects following sibling Imgs
/// when their common parent is Picture. Topology and recipient iteration belong
/// to the caller. Width/height participate even before an image is available.
pub fn image_source_attribute_affects_following(
    namespace: &Namespace,
    attribute: KnownHtmlAttribute,
) -> bool {
    namespace.uri().is_none()
        && matches!(
            attribute,
            KnownHtmlAttribute::Srcset
                | KnownHtmlAttribute::Sizes
                | KnownHtmlAttribute::Media
                | KnownHtmlAttribute::Type
                | KnownHtmlAttribute::Width
                | KnownHtmlAttribute::Height
        )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageTreeChange {
    Inserted,
    Removed,
    Moved,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageMutationTargets {
    None,
    ThisImage,
    ParentPictureImages,
    OldParentPictureImages,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlImageTreeEffect {
    targets: HtmlImageMutationTargets,
    clear_auto_sizes_width: bool,
    check_media_parent_selection: bool,
}
impl HtmlImageTreeEffect {
    pub const fn targets(self) -> HtmlImageMutationTargets {
        self.targets
    }
    pub const fn clear_auto_sizes_width(self) -> bool {
        self.clear_auto_sizes_width
    }
    /// Source insertion/moving under media requires its actual src absence and
    /// NETWORK_EMPTY state before invoking that owner's selection algorithm.
    /// This is not evidence about those facts or a resource-execution request.
    pub const fn check_media_parent_selection(self) -> bool {
        self.check_media_parent_selection
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlImageTreeInapplicable;
impl fmt::Display for HtmlImageTreeInapplicable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("image tree hook requires Img or Source")
    }
}
impl Error for HtmlImageTreeInapplicable {}

/// Img/Source HTML insertion, removing and moving steps. Inputs are actual
/// recognized HTML identities; None covers no parent or an unrecognized parent.
/// The caller retains the actual old ancestor and executes each source step,
/// rather than inferring recipients from the final transaction topology.
/// Service a required media-parent check before the returned mutation targets.
/// For Img removal, start the relevant update before clearing auto-sizes width.
pub fn image_tree_effect(
    element: KnownHtmlElement,
    change: HtmlImageTreeChange,
    parent: Option<KnownHtmlElement>,
    old_parent: Option<KnownHtmlElement>,
    is_subtree_root: bool,
) -> Result<HtmlImageTreeEffect, HtmlImageTreeInapplicable> {
    use HtmlImageMutationTargets as T;
    use HtmlImageTreeChange as C;
    use KnownHtmlElement as E;
    let targets = match (element, change) {
        (E::Img, C::Inserted) if parent == Some(E::Picture) => T::ThisImage,
        (E::Source, C::Inserted) if parent == Some(E::Picture) => T::ParentPictureImages,
        (E::Img, C::Removed | C::Moved) if is_subtree_root && old_parent == Some(E::Picture) => {
            T::ThisImage
        }
        (E::Source, C::Removed | C::Moved) if is_subtree_root && old_parent == Some(E::Picture) => {
            T::OldParentPictureImages
        }
        (E::Img | E::Source, _) => T::None,
        _ => return Err(HtmlImageTreeInapplicable),
    };
    Ok(HtmlImageTreeEffect {
        targets,
        clear_auto_sizes_width: element == E::Img && change == C::Removed,
        check_media_parent_selection: element == E::Source
            && change != C::Removed
            && matches!(parent, Some(E::Audio | E::Video)),
    })
}
