//! Frame's local classification and first post-connection decision.
//! HTML 877d6146487f #frames and #child-navigables define these local facts.

use crate::HtmlBrowsingContextPresence;

/// The first decision of frame's HTML element post-connection steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFramePostConnectionDecision {
    /// The actual node document's browsing context is null. Return before
    /// creating a child navigable or processing any frame attributes.
    Return,
    /// Continue the source algorithm: create a real child navigable, then
    /// process frame attributes with initialInsertion true. The consuming
    /// adapter must establish the required capabilities and own those effects.
    Continue,
    /// The actual browsing-context association is unavailable. Neither source
    /// branch is established; this is not an absent browsing context.
    NeedsBrowsingContext,
}

/// The exact first guard of [frame post-connection](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#frames).
///
/// The input is the actual node document's association, independently of its
/// XML/HTML class or the element's attributes. No attribute, topology or resource
/// is read, and Continue does not itself authorize or execute child creation.
pub const fn frame_post_connection(
    browsing_context: HtmlBrowsingContextPresence,
) -> HtmlFramePostConnectionDecision {
    match browsing_context {
        HtmlBrowsingContextPresence::Absent => HtmlFramePostConnectionDecision::Return,
        HtmlBrowsingContextPresence::Present => HtmlFramePostConnectionDecision::Continue,
        HtmlBrowsingContextPresence::Unavailable => {
            HtmlFramePostConnectionDecision::NeedsBrowsingContext
        }
    }
}

/// Frame's source-defined potential classification, independent of actual
/// content or child load state. It does not establish an actual load delay.
pub const fn frame_potentially_delays_load() -> bool {
    true
}
