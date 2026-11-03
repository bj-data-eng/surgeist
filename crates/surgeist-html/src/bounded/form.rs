//! Form-owner reset decisions and native form association over caller-supplied reads.

pub use crate::internal::elements::form_association::native_form_association_with;
pub use crate::internal::elements::form_owner::{
    HtmlFormOwnerLookup, HtmlFormOwnerResetAction, HtmlFormOwnerResetCursor,
    HtmlFormOwnerResetCursorError, HtmlFormOwnerResetDecision, HtmlFormOwnerResetReply,
    HtmlFormOwnerResetTrigger, form_owner_lookup, form_owner_reset_trigger,
};
