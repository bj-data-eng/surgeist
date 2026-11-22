//! Incremental construction from a bounded decoded source into a hosted tree.
//!
//! The host owns the canonical tree. This profile admits a no-quirks HTML
//! doctype, explicit HTML, head and body, ordinary `div` and `span`, `id` and
//! `class`, text and comments. Other reached construction work is reported
//! without inventing effect, scripting or load-delay observations. An opt-in
//! classic-script profile additionally admits implied structure, UTF-8 meta,
//! title, and blocking classic scripts with inline text or a local path.

pub use crate::parser::tree_builder::live::{
    HtmlLiveAttribute, HtmlLiveAttributes, HtmlLiveElement, HtmlLiveParseError,
    HtmlLiveParseProgress, HtmlLiveParser, HtmlLiveStopParsing, HtmlLiveTree, HtmlLiveUnsupported,
};
