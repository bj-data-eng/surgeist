//! Source-initial Iframe intrinsic values and exact local guards.
//! HTML 877d6146487f #the-iframe-element, #lazy-load-resumption-steps and
//! #child-navigables define these facts; resource effects remain with consumers.

use crate::{HtmlBrowsingContextPresence, HtmlLoading};

/// Source-first branch of processing iframe attributes, after child creation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIframeAttributeBranch {
    /// Present srcdoc, including empty, precedes shared src processing.
    /// Continue the actual intrinsic/lazy/srcdoc-navigation steps.
    Srcdoc,
    Shared,
}

/// Selects only the srcdoc-presence branch. The actual owner reads that
/// presence before reading src; this function performs no attribute lookup.
pub const fn iframe_attribute_branch(srcdoc_present: bool) -> HtmlIframeAttributeBranch {
    if srcdoc_present {
        HtmlIframeAttributeBranch::Srcdoc
    } else {
        HtmlIframeAttributeBranch::Shared
    }
}

/// The first guard of iframe's HTML element post-connection steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIframePostConnectionDecision {
    /// Return before sandbox parsing, child creation or attribute processing.
    Return,
    /// Continue the actual algorithm: parse a present sandbox directive, create
    /// a real child navigable, then process iframe attributes with initialInsertion
    /// true. This decision establishes no resource capability or completed effect.
    Continue,
    /// Actual browsing-context association is unavailable, not known absent.
    NeedsBrowsingContext,
}

/// The actual node-document browsing context is checked before any iframe
/// attribute read. XML/HTML class and connectedness do not establish this input.
pub const fn iframe_post_connection(
    browsing_context: HtmlBrowsingContextPresence,
) -> HtmlIframePostConnectionDecision {
    match browsing_context {
        HtmlBrowsingContextPresence::Absent => HtmlIframePostConnectionDecision::Return,
        HtmlBrowsingContextPresence::Present => HtmlIframePostConnectionDecision::Continue,
        HtmlBrowsingContextPresence::Unavailable => {
            HtmlIframePostConnectionDecision::NeedsBrowsingContext
        }
    }
}

/// Whether the semantic loading state changed to Eager (#attr-iframe-loading).
/// The caller obtains states from the shared loading interpreter, not raw-token
/// equality. On true it reads actual resumption steps: null returns; otherwise
/// those steps must be cleared before invocation. This function invokes nothing.
pub const fn iframe_loading_became_eager(previous: HtmlLoading, current: HtmlLoading) -> bool {
    matches!((previous, current), (HtmlLoading::Lazy, HtmlLoading::Eager))
}

/// The actual source-initial lazy/timing/resumption tuple of an iframe.
///
/// This proof contains no child relationship or browsing-context assumption.
/// Creating a child does not itself change these intrinsic fields. In the
/// no-srcdoc, absent-src initial path, the shared ancestor-URL guard can return
/// before any lazy/timing write, preserving this tuple with present content.
/// The owner must leave this domain before a reached intrinsic write; it cannot
/// reset processed/imported history by calling fresh(). Sandbox state and real
/// resumption/timing objects belong to their actual consuming owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlIframeInitialState {
    initial: (),
}

impl HtmlIframeInitialState {
    /// Actual fresh intrinsic initialization. Clone/import-as-clone creates a
    /// new element and fresh fields; adoption preserves the actual fields.
    /// This value performs neither algorithm and certifies no source identity.
    pub const fn fresh() -> Self {
        Self { initial: () }
    }

    /// #current-navigation-was-lazy-loaded is initially false; loading content
    /// attributes alone do not change it.
    pub const fn current_navigation_was_lazy_loaded(&self) -> bool {
        false
    }

    pub const fn pending_resource_timing_start_time_is_null(&self) -> bool {
        true
    }

    pub const fn pending_resource_timing_url_is_null(&self) -> bool {
        true
    }

    pub const fn lazy_load_resumption_is_null(&self) -> bool {
        true
    }

    /// Iframe's potential-load classification follows its current navigation's
    /// lazy flag, independently of the loading attribute's lexical state.
    pub const fn potentially_delays_load(&self) -> bool {
        !self.current_navigation_was_lazy_loaded()
    }
}
