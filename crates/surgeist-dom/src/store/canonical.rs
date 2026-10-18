//! Immutable family state uses shared sequence backing; algorithms retain their
//! separate staged operands until canonical publication.
use super::{NodeId, SharedVec};
use surgeist_html_semantics::HtmlParseBaseFacts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Autofocus {
    Known {
        candidates: SharedVec<NodeId>,
        processed: bool,
    },
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Interactive {
    Known {
        open_dialogs: SharedVec<NodeId>,
        top_layer: SharedVec<NodeId>,
        pending_top_layer_removals: SharedVec<NodeId>,
    },
    Unavailable,
}

#[derive(Clone, Debug)]
pub(crate) struct BaseDocument {
    pub(crate) facts: Option<HtmlParseBaseFacts>,
    pub(crate) first: Option<NodeId>,
    pub(crate) render_blocking: SharedVec<NodeId>,
}
impl PartialEq for BaseDocument {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first
            && self.render_blocking == other.render_blocking
            && match (&self.facts, &other.facts) {
                (None, None) => true,
                (Some(left), Some(right)) => left.same_owner(right),
                _ => false,
            }
    }
}
impl Eq for BaseDocument {}
