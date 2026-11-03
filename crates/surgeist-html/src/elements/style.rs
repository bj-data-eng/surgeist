//! Native `style` element update and notification order, without CSS or CSP execution.

pub use crate::internal::elements::style::{
    HtmlDocumentBodyRole, HtmlStyleAttributeEffect, HtmlStyleExecutionProfile,
    HtmlStyleNotificationAction, HtmlStyleNotificationCursor, HtmlStyleNotificationReply,
    HtmlStyleParserStateError, HtmlStyleProtocolError, HtmlStyleState, HtmlStyleTrigger,
    HtmlStyleUpdateAction, HtmlStyleUpdateCursor, HtmlStyleUpdateReply, blocking_tokens_with,
    document_allows_render_blocking, document_body_role_with, style_attribute_effect,
    style_is_potentially_render_blocking, style_type_allows_css_with, style_update_is_required,
};
