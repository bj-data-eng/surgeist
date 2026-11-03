//! Hyperlink associated-URL steps and link-type (`rel`) interpretation.

pub use crate::internal::elements::hyperlink::{
    HtmlHyperlinkAttributeEffect, HtmlHyperlinkSetUrlAction, HtmlHyperlinkSetUrlCursor,
    HtmlHyperlinkSetUrlError, HtmlHyperlinkSetUrlReply, HtmlHyperlinkTreeMutation,
    hyperlink_attribute_effect, hyperlink_tree_considers_speculative_loads,
};
pub use crate::internal::elements::rel::{
    HtmlLinkFlags, HtmlLinkHistory, HtmlRelAddressAuthor, HtmlRelAlternate, HtmlRelBodyAllowance,
    HtmlRelEffect, HtmlRelHrefAuthor, HtmlRelKeyword, HtmlRelLinks, HtmlRelOccurrence,
    HtmlRelOccurrenceSource, HtmlRelPresenceAuthor, HtmlRelReport, HtmlRelRole, HtmlRelSpelling,
    HtmlRelTitleAuthor, HtmlRelTokenApplicability,
};
