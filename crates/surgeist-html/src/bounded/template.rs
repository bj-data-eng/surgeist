//! Template marker scanning and slot assignment over caller-owned identity-bearing storage.

pub use crate::internal::elements::slot_assign::{
    HtmlSlotAssignAction, HtmlSlotAssignCursor, HtmlSlotAssignCursorError, HtmlSlotAssignReply,
};
pub use crate::internal::elements::template_markers::{
    HtmlTemplateMarkerFirst, HtmlTemplateMarkerNestingOverflow, HtmlTemplateMarkerSiblings,
    HtmlTemplateMarkerTarget, template_marker_first,
};
