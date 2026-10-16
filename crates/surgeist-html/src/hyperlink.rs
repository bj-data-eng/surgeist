//! HTML877d A/Area associated-URL steps, independent of URL parsing and storage.
use crate::{KnownHtmlAttribute, Namespace};
use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkSetUrlAction {
    ClearUrl,
    ReadHref,
    ParseHref,
    StoreParsedUrl,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkSetUrlReply {
    Applied,
    HrefPresent(bool),
    Parsed(bool),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlHyperlinkSetUrlError;
impl fmt::Display for HtmlHyperlinkSetUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("hyperlink set-URL reply does not match the current action")
    }
}
impl Error for HtmlHyperlinkSetUrlError {}

/// Source #concept-hyperlink-url-set, invoked at creation and actual href change.
/// The consumer initializes fresh URL storage to null, then runs this algorithm;
/// it clears before reading href, and leaves null for absence or parse failure.
/// A present empty href still parses. Complete each action before replying.
///
/// Parsing is the actual encoding-parse operation relative to the CURRENT node
/// Document, with its Document base URL and encoding. This cursor owns neither
/// identities nor a URL engine/CSP decision. Failures in observation, admission
/// or cancellation are not Parsed(false); the consumer aborts its transaction.
#[derive(Clone, Copy, Debug)]
pub struct HtmlHyperlinkSetUrlCursor {
    action: HtmlHyperlinkSetUrlAction,
}
impl Default for HtmlHyperlinkSetUrlCursor {
    fn default() -> Self {
        Self::new()
    }
}
impl HtmlHyperlinkSetUrlCursor {
    pub const fn new() -> Self {
        Self {
            action: HtmlHyperlinkSetUrlAction::ClearUrl,
        }
    }
    pub const fn action(&self) -> HtmlHyperlinkSetUrlAction {
        self.action
    }
    pub fn reply(
        &mut self,
        reply: HtmlHyperlinkSetUrlReply,
    ) -> Result<(), HtmlHyperlinkSetUrlError> {
        use HtmlHyperlinkSetUrlAction as A;
        use HtmlHyperlinkSetUrlReply as R;
        self.action = match (self.action, reply) {
            (A::ClearUrl, R::Applied) => A::ReadHref,
            (A::ReadHref, R::HrefPresent(false)) | (A::ParseHref, R::Parsed(false)) => A::Complete,
            (A::ReadHref, R::HrefPresent(true)) => A::ParseHref,
            (A::ParseHref, R::Parsed(true)) => A::StoreParsedUrl,
            (A::StoreParsedUrl, R::Applied) => A::Complete,
            _ => return Err(HtmlHyperlinkSetUrlError),
        };
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkAttributeEffect {
    None,
    SetUrlThenConsiderSpeculativeLoads,
    ConsiderSpeculativeLoads,
}
/// A/Area attribute steps (#api-for-a-and-area-elements). The caller admits
/// exact optional old/new UTF-16 equality, including absent versus empty. A
/// non-null namespace or equal values returns before URL/Blob-store work.
/// Unknown attribute names have no effect and need not call this function.
pub fn hyperlink_attribute_effect(
    namespace: &Namespace,
    attribute: KnownHtmlAttribute,
    values_equal: bool,
) -> HtmlHyperlinkAttributeEffect {
    use HtmlHyperlinkAttributeEffect as E;
    if namespace.uri().is_some() || values_equal {
        return E::None;
    }
    match attribute {
        KnownHtmlAttribute::Href => E::SetUrlThenConsiderSpeculativeLoads,
        KnownHtmlAttribute::Rel | KnownHtmlAttribute::Referrerpolicy => E::ConsiderSpeculativeLoads,
        _ => E::None,
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkTreeMutation {
    Insertion { connected: bool },
    Removal { old_ancestor_connected: bool },
    Moving,
}
/// Whether the A/Area tree hook invokes consider-speculative-loads. Removal
/// uses the actual old ancestor's Document, independently of an adopted node's
/// current Document. This does not itself queue work or establish a navigable.
pub const fn hyperlink_tree_considers_speculative_loads(change: HtmlHyperlinkTreeMutation) -> bool {
    match change {
        HtmlHyperlinkTreeMutation::Insertion { connected } => connected,
        HtmlHyperlinkTreeMutation::Removal {
            old_ancestor_connected,
        } => old_ancestor_connected,
        HtmlHyperlinkTreeMutation::Moving => true,
    }
}
