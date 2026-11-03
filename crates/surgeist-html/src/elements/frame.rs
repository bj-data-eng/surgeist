//! Frame and iframe classification, the first post-connection decision and child-navigable guards.

pub use crate::internal::elements::child_navigable::{
    HtmlChildNavigableAccessDecision, HtmlChildNavigableLoadDelayDecision,
    HtmlChildNavigablePresence, HtmlFrameIframeAfterAncestors, HtmlFrameIframeSrcDecision,
};
pub use crate::internal::elements::frame::{
    HtmlFramePostConnectionDecision, frame_post_connection, frame_potentially_delays_load,
};
pub use crate::internal::elements::iframe::{
    HtmlIframeAttributeBranch, HtmlIframeInitialState, HtmlIframePostConnectionDecision,
    iframe_attribute_branch, iframe_loading_became_eager, iframe_post_connection,
};
