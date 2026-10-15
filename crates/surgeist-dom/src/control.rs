//! Shared private native association and custom-error storage.
use crate::{DocumentString, NodeId};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeControlState {
    pub(crate) owner: Option<NodeId>,
    pub(crate) parser_inserted: bool,
    pub(crate) message: DocumentString,
}
impl NativeControlState {
    pub(crate) fn fresh() -> Self {
        Self {
            owner: None,
            parser_inserted: false,
            message: "".into(),
        }
    }
}
