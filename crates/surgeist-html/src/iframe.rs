//! Native unprocessed Iframe values and exact local guards.
//! HTML 877d6146487f #the-iframe-element, #lazy-load-resumption-steps and
//! #child-navigables define these facts; resource effects remain with consumers.

use crate::child_navigable::NullChildNavigable;
use crate::{HtmlBrowsingContextPresence, HtmlLoading};

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

/// A fresh native iframe in its unprocessed, null-child phase.
///
/// The caller establishes actual fresh creation in a never-browsing-context
/// document. The source post-connection guard returns before sandbox processing
/// and child creation; null-content attribute hooks preserve this phase. The
/// exact raw sandbox attribute and its independent local directive interpretation
/// are not a numeric live sandbox policy. This type exposes no such policy,
/// external-unavailable substitute, navigable, URL, timestamp or callback handle.
///
/// Current null content alone does not prove this state for previously processed
/// or imported history. Retained-state installation needs its actual cause/state
/// proof or rejection; it cannot erase established policy by calling fresh().
/// Future context/child creation must initialize real policy before consuming
/// numeric flags and leave this native domain. This is not a default-allow policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlIframeNativeState {
    content: NullChildNavigable,
}

impl HtmlIframeNativeState {
    /// Actual fresh native creation only. DOM clone/import-as-clone creates a
    /// new element then copies attributes; it does not copy child resources or
    /// processed policy. Adoption and snapshot/transaction copying preserve the
    /// existing state instead of rerunning this initializer. This value performs
    /// none of those DOM algorithms and cannot certify a retained identity.
    pub const fn fresh() -> Self {
        Self {
            content: NullChildNavigable::fresh(),
        }
    }

    pub const fn content_navigable_is_null(&self) -> bool {
        self.content.is_null()
    }

    pub const fn content_document_is_null(&self) -> bool {
        self.content.content_document_is_null()
    }

    pub const fn content_window_is_null(&self) -> bool {
        self.content.content_window_is_null()
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

    /// Potential classification alone cannot delay load without actual content.
    pub const fn delays_load(&self) -> bool {
        self.content.delays_load()
    }

    /// srcdoc set/change/removal processes attributes only with nonnull content.
    pub const fn srcdoc_change_requires_processing(&self) -> bool {
        !self.content.is_null()
    }

    /// src set/change/removal checks nonnull content before srcdoc absence.
    /// This null branch returns without reading srcdoc, including present-empty.
    pub const fn src_change_requires_processing(&self) -> bool {
        !self.content.is_null()
    }

    /// Sandbox set/change parses only with nonnull content. Local directive
    /// interpretation does not execute this live-state mutation.
    pub const fn sandbox_change_requires_processing(&self) -> bool {
        !self.content.is_null()
    }

    /// Sandbox removal clears the live set only with nonnull content.
    pub const fn sandbox_removal_requires_clearing(&self) -> bool {
        !self.content.is_null()
    }

    /// Removing invokes destroy-child, whose null branch returns before effects.
    /// Other applicable removing steps remain the consumer's responsibility.
    pub const fn removal_requires_child_destruction(&self) -> bool {
        self.content.removal_requires_child_destruction()
    }
}
