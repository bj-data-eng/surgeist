//! Frame's initial null content and the first post-connection decision.
//! HTML 877d6146487f #frames and #child-navigables define these local facts.

use crate::HtmlBrowsingContextPresence;
use crate::child_navigable::NullChildNavigable;

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

/// An actual frame whose content navigable remains null.
///
/// The [child-navigables rule](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigables)
/// initializes this relationship to null. This allocation-free value carries
/// no navigable, Window, document identity or resource capability. It cannot
/// represent non-null or unknown retained content. Its caller must use fresh()
/// only at actual creation, never to replace missing imported/retained facts.
///
/// A live owner must retain the actual relationship and wire post-connection,
/// src-attribute and removing steps. If a real child is installed, that owner
/// leaves this null-content domain; it cannot keep using these decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFrameNullContent {
    content: NullChildNavigable,
}

impl HtmlFrameNullContent {
    /// Actual fresh frame initialization. Element cloning creates fresh family
    /// state and then copies attributes; it does not copy a child navigable.
    /// Adoption preserves an existing actual relationship instead of invoking
    /// this initializer. This value does not perform either DOM algorithm.
    pub const fn fresh() -> Self {
        Self {
            content: NullChildNavigable::fresh(),
        }
    }

    pub const fn content_navigable_is_null(&self) -> bool {
        self.content.is_null()
    }

    /// The content-document getter returns null before origin comparison when
    /// content navigable is null (#concept-bcc-content-document).
    pub const fn content_document_is_null(&self) -> bool {
        self.content.content_document_is_null()
    }

    /// The content-window getter returns null before WindowProxy access when
    /// content navigable is null (#content-window).
    pub const fn content_window_is_null(&self) -> bool {
        self.content.content_window_is_null()
    }

    /// A frame is always an element type that potentially delays the load event
    /// (#frames). This classification alone does not establish an actual delay.
    pub const fn potentially_delays_load(&self) -> bool {
        true
    }

    /// The actual delay rule requires non-null content navigable before any
    /// child load conditions (#potentially-delays-the-load-event).
    pub const fn delays_load(&self) -> bool {
        self.content.delays_load()
    }

    /// A frame src set, change or removal processes attributes only with a
    /// non-null content navigable (#frames); its value need not be read here.
    pub const fn src_change_requires_processing(&self) -> bool {
        !self.content.is_null()
    }

    /// Frame removing steps invoke destroy-a-child-navigable. That algorithm
    /// returns on actual null content before writes, document destruction or
    /// navigation/history effects. This decision describes that null branch;
    /// it does not suppress other applicable HTML or DOM removing steps.
    pub const fn removal_requires_child_destruction(&self) -> bool {
        self.content.removal_requires_child_destruction()
    }
}
