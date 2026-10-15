//! Shared private native association and custom-error storage.
use crate::DocumentString;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeControlState {
    pub(crate) parser_inserted: bool,
    pub(crate) message: DocumentString,
}
impl NativeControlState {
    pub(crate) fn fresh() -> Self {
        Self {
            parser_inserted: false,
            message: "".into(),
        }
    }
}
