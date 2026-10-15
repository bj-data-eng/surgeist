//! Nonce content and cryptographic-slot decisions, HTML 877d §2.5.6.
use crate::values::{HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, SVG_NAMESPACE_URI};
use crate::{DocumentString, QualifiedName};

/// Actual captured cryptographic slot; content attributes are independent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedNonce {
    Unavailable,
    Value(DocumentString),
}
/// Actual Document browsing-context association.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlBrowsingContextPresence {
    Unavailable,
    Absent,
    Present,
}
/// Whether the actual policy container has a header-delivered CSP.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHeaderDeliveredCsp {
    Unavailable,
    Absent,
    Present,
}
/// Independently supplied Document facts; this does not parse or enforce CSP.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNonceDocumentEnvironment {
    browsing_context: HtmlBrowsingContextPresence,
    header_delivered_csp: HtmlHeaderDeliveredCsp,
}
impl HtmlNonceDocumentEnvironment {
    pub fn new(
        browsing_context: HtmlBrowsingContextPresence,
        header_delivered_csp: HtmlHeaderDeliveredCsp,
    ) -> Self {
        Self {
            browsing_context,
            header_delivered_csp,
        }
    }
    pub fn browsing_context(self) -> HtmlBrowsingContextPresence {
        self.browsing_context
    }
    pub fn header_delivered_csp(self) -> HtmlHeaderDeliveredCsp {
        self.header_delivered_csp
    }
}
/// Current insertion eligibility, derived from the actual shadow-including root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNonceConnection {
    NotBrowsingContextConnected,
    BrowsingContextConnected,
    Unavailable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNonceMissing {
    Connection,
    HeaderDeliveredCsp,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNonceInsertionEffect {
    KeepContent,
    HideContentPreservingSlot,
    Unavailable(HtmlNonceMissing),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNonceAttributeEffect<'a> {
    Ignore,
    ClearSlot,
    CopyValue(&'a DocumentString),
}

// Length rejection precedes inspection. Even arbitrary unmatched inputs cannot
// cause more than these fixed literal lengths to be read by one decision.
pub const NONCE_INSPECTION_BOUND: usize = 3 * 40 + 5;
fn exact(value: &DocumentString, literal: &[u8]) -> bool {
    value.len() == literal.len()
        && value
            .code_units()
            .iter()
            .zip(literal)
            .all(|(&a, &b)| a == u16::from(b))
}
pub fn nonce_element(name: &QualifiedName) -> bool {
    name.namespace().uri().is_some_and(|uri| {
        [
            HTML_NAMESPACE_URI.as_bytes(),
            SVG_NAMESPACE_URI.as_bytes(),
            MATHML_NAMESPACE_URI.as_bytes(),
        ]
        .iter()
        .any(|literal| exact(uri, literal))
    })
}
/// Attribute-change decision. Prefix metadata does not change expanded identity.
pub fn nonce_attribute_change<'a>(
    element: &QualifiedName,
    attribute: &QualifiedName,
    new_value: Option<&'a DocumentString>,
) -> HtmlNonceAttributeEffect<'a> {
    if !nonce_element(element)
        || attribute.namespace().uri().is_some()
        || !exact(attribute.local_name(), b"nonce")
    {
        return HtmlNonceAttributeEffect::Ignore;
    }
    new_value.map_or(
        HtmlNonceAttributeEffect::ClearSlot,
        HtmlNonceAttributeEffect::CopyValue,
    )
}
/// Insertion-only decision. A known false conjunct discharges unrelated gaps.
pub fn nonce_insertion_effect(
    element: &QualifiedName,
    content: Option<&DocumentString>,
    connection: HtmlNonceConnection,
    policy: HtmlHeaderDeliveredCsp,
) -> HtmlNonceInsertionEffect {
    if !nonce_element(element)
        || content.is_none_or(DocumentString::is_empty)
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
