//! HTML877d A/Area associated-URL steps, independent of URL parsing and storage.
use crate::{internal::values::Namespace, vocabulary::KnownHtmlAttribute};
use std::{error::Error, fmt};

/// The next owner action of the set the URL algorithm (HTML877d #concept-hyperlink-url-set). The
/// consumer performs it and acknowledges it with `HtmlHyperlinkSetUrlReply`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkSetUrlAction {
    /// Set the element's URL to null. It precedes the `href` read.
    ClearUrl,
    /// Read whether the element has an `href` attribute; a present empty value counts as present.
    ReadHref,
    /// Parse the `href` value against the element's current node document and report whether
    /// parsing succeeded.
    ParseHref,
    /// Store the parsed URL as the element's URL.
    StoreParsedUrl,
    /// The algorithm is done; no further reply is accepted.
    Complete,
}
/// The consumer's answer to a `HtmlHyperlinkSetUrlAction`. A mismatched pairing is refused with
/// `HtmlHyperlinkSetUrlError`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkSetUrlReply {
    /// The pending clear or store action was performed.
    Applied,
    /// Whether the `href` attribute is present, answering `ReadHref`.
    HrefPresent(bool),
    /// Whether parsing the `href` value succeeded, answering `ParseHref`.
    Parsed(bool),
}
/// The refusal of a reply that does not match the pending action, including a reply after
/// `Complete`. The cursor is left unchanged.
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
    /// Starts the algorithm, with `ClearUrl` pending.
    pub const fn new() -> Self {
        Self {
            action: HtmlHyperlinkSetUrlAction::ClearUrl,
        }
    }
    /// The action the consumer must perform next, or `Complete` once the algorithm is done.
    pub const fn action(&self) -> HtmlHyperlinkSetUrlAction {
        self.action
    }
    /// Acknowledges the pending action. A false `href` presence or a failed parse completes the
    /// algorithm with the URL left null. A mismatched reply fails with `HtmlHyperlinkSetUrlError`
    /// and leaves the cursor unchanged.
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

/// What a changed attribute of an `a` or `area` element requires (HTML877d
/// #api-for-a-and-area-elements).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkAttributeEffect {
    /// The change requires nothing.
    None,
    /// Run the set the URL algorithm, then consider speculative loads. This is the `href` change.
    SetUrlThenConsiderSpeculativeLoads,
    /// Consider speculative loads without setting the URL. This is the `rel` or `referrerpolicy`
    /// change.
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
/// A change of an `a` or `area` element's position in the tree. `Insertion` and `Removal` carry the
/// connectedness fact the speculative-load hook reads; `Moving` carries none.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkTreeMutation {
    /// The element was inserted.
    Insertion {
        /// Whether the element is now connected, which is what considering speculative loads needs.
        connected: bool,
    },
    /// The element was removed.
    Removal {
        /// Whether the old ancestor chain was connected, read from the old ancestor's document
        /// rather than the removed node's current one.
        old_ancestor_connected: bool,
    },
    /// The element was moved, which always considers speculative loads.
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
