//! Nonce content and cryptographic-slot decisions, HTML 877d §2.5.6.
use crate::internal::values::DocumentString;
use crate::internal::values::{HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, SVG_NAMESPACE_URI};

/// Actual captured cryptographic slot; content attributes are independent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedNonce {
    /// The captured slot was not supplied, so no decision may assume a value.
    Unavailable,
    /// The captured `[[CryptographicNonce]]` value, with exact UTF-16 code units.
    Value(DocumentString),
}
/// Actual Document browsing-context association.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlBrowsingContextPresence {
    /// The caller could not establish whether the Document has a browsing context.
    Unavailable,
    /// The Document has no browsing context.
    Absent,
    /// The Document has a browsing context.
    Present,
}
/// Whether the actual policy container has a header-delivered CSP.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHeaderDeliveredCsp {
    /// The caller could not establish whether the policy container has a header-delivered CSP.
    Unavailable,
    /// The policy container has no header-delivered Content Security Policy.
    Absent,
    /// The policy container has at least one header-delivered Content Security Policy.
    Present,
}
/// Independently supplied Document facts; this does not parse or enforce CSP.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNonceDocumentEnvironment {
    browsing_context: HtmlBrowsingContextPresence,
    header_delivered_csp: HtmlHeaderDeliveredCsp,
}
impl HtmlNonceDocumentEnvironment {
    /// Pairs the two independently supplied Document facts. Neither is inferred from the other.
    pub fn new(
        browsing_context: HtmlBrowsingContextPresence,
        header_delivered_csp: HtmlHeaderDeliveredCsp,
    ) -> Self {
        Self {
            browsing_context,
            header_delivered_csp,
        }
    }
    /// The supplied browsing-context association.
    pub fn browsing_context(self) -> HtmlBrowsingContextPresence {
        self.browsing_context
    }
    /// The supplied header-delivered CSP presence.
    pub fn header_delivered_csp(self) -> HtmlHeaderDeliveredCsp {
        self.header_delivered_csp
    }
}
/// Current insertion eligibility, derived from the actual shadow-including root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNonceConnection {
    /// The element's shadow-including root is not a Document with a browsing context.
    NotBrowsingContextConnected,
    /// The element's shadow-including root is a Document with a browsing context.
    BrowsingContextConnected,
    /// The caller could not establish connectedness.
    Unavailable,
}
/// The Document fact that an `Unavailable` insertion decision still needs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNonceMissing {
    /// Whether the element is browsing-context connected.
    Connection,
    /// Whether the policy container has a header-delivered CSP.
    HeaderDeliveredCsp,
}
/// The outcome of the nonce insertion steps; see [`nonce_insertion_effect`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNonceInsertionEffect {
    /// Leave the `nonce` content attribute unchanged: a known false condition means the hiding
    /// steps do not apply.
    KeepContent,
    /// Set the `nonce` content attribute to the empty string while the cryptographic nonce slot
    /// keeps its value, so the nonce is not exposed to attribute selectors.
    HideContentPreservingSlot,
    /// The decision depends on an unsupplied fact; the carried value names which one.
    Unavailable(HtmlNonceMissing),
}
/// The outcome of the nonce attribute change steps; see [`nonce_attribute_change`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNonceAttributeEffect<'a> {
    /// Not a nonce attribute change on a nonce-capable element: leave the slot untouched.
    Ignore,
    /// The attribute was removed: set the cryptographic nonce slot to the empty string.
    ClearSlot,
    /// The attribute now has this value, given as its exact UTF-16 code units: set the
    /// cryptographic nonce slot to it.
    CopyValue(&'a [u16]),
}

// Length rejection precedes inspection. Even arbitrary unmatched inputs cannot
// cause more than these fixed literal lengths to be read by one decision.
/// The most UTF-16 code units one nonce decision can inspect: three namespace URI literals of
/// at most 40 units each, plus the five units of `nonce`. A length mismatch is rejected before
/// any unit is read.
pub const HTML_NONCE_INSPECTION_BOUND: usize = 3 * 40 + 5;
fn exact(value: &DocumentString, literal: &[u8]) -> bool {
    value.len() == literal.len()
        && value
            .code_units()
            .iter()
            .zip(literal)
            .all(|(&a, &b)| a == u16::from(b))
}
/// Whether `name` is in the HTML, SVG or MathML namespace, the elements that implement the
/// nonce mixin. Compares the exact expanded namespace URI, not the prefix.
pub fn is_nonce_element(
    name: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
) -> bool {
    name.namespace_uri().is_some_and(|uri| {
        [
            HTML_NAMESPACE_URI.as_bytes(),
            SVG_NAMESPACE_URI.as_bytes(),
            MATHML_NAMESPACE_URI.as_bytes(),
        ]
        .iter()
        .any(|literal| exact(uri, literal))
    })
}
/// Attribute-change decision. Prefix metadata does not change expanded identity. `new_value` is the
/// attribute's exact UTF-16 code units, or `None` once it is removed.
pub fn nonce_attribute_change<'a>(
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    attribute: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    new_value: Option<&'a [u16]>,
) -> HtmlNonceAttributeEffect<'a> {
    if !is_nonce_element(element)
        || attribute.namespace_uri().is_some()
        || !exact(attribute.local_name(), b"nonce")
    {
        return HtmlNonceAttributeEffect::Ignore;
    }
    new_value.map_or(
        HtmlNonceAttributeEffect::ClearSlot,
        HtmlNonceAttributeEffect::CopyValue,
    )
}
/// Insertion-only decision. A known false conjunct discharges unrelated gaps. `content` is the
/// `nonce` content attribute's exact UTF-16 code units, or `None` when it is absent.
pub fn nonce_insertion_effect(
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    content: Option<&[u16]>,
    connection: HtmlNonceConnection,
    policy: HtmlHeaderDeliveredCsp,
) -> HtmlNonceInsertionEffect {
    if !is_nonce_element(element)
        || content.is_none_or(<[u16]>::is_empty)
        || connection == HtmlNonceConnection::NotBrowsingContextConnected
        || policy == HtmlHeaderDeliveredCsp::Absent
    {
        return HtmlNonceInsertionEffect::KeepContent;
    }
    if connection == HtmlNonceConnection::Unavailable {
        return HtmlNonceInsertionEffect::Unavailable(HtmlNonceMissing::Connection);
    }
    if policy == HtmlHeaderDeliveredCsp::Unavailable {
        return HtmlNonceInsertionEffect::Unavailable(HtmlNonceMissing::HeaderDeliveredCsp);
    }
    HtmlNonceInsertionEffect::HideContentPreservingSlot
}
