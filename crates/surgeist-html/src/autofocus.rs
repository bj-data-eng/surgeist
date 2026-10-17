//! HTML877d autofocus insertion bookkeeping (raw60684–60707).
//!
//! Ordinary Document-tree insertion is defined at raw1479–1485. This producer
//! elects unconditional continuation past the optional user-preference return;
//! it neither observes a preference nor supplies a preference service.

use crate::values::{HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, SVG_NAMESPACE_URI};
use crate::{Attribute, KnownHtmlAttribute, QualifiedName};

/// Recognize one actual autofocus insertion-trigger attribute for an Element
/// in the HTML, SVG or MathML namespace. HTML877d raw8898–8903 excludes global
/// attributes on namespaces not defined as having them; raw8160–8167 shares
/// autofocus with SVG and MathML. HTML namespace identity applies even in XML
/// Documents (raw1373–1380), independently of the Element's local name/prefix.
///
/// Only exact null-namespace `autofocus` matches; the attribute prefix is
/// irrelevant to expanded-name identity. Every supplied value counts as present,
/// including nonconforming Boolean spellings. This does not produce an authored
/// attribute applicability report. None represents absence.
///
/// This helper neither walks an attribute collection nor allocates or inspects
/// values. At most three constant URI length checks admit at most 34 UTF-16 URI
/// comparisons (the three URI lengths differ), followed by at most nine attribute
/// local-name comparisons after its length check. The adapter admits its actual
/// attribute walk and this fixed work before calling the helper.
pub fn autofocus_insertion_presence(
    element: &QualifiedName,
    attribute: Option<&Attribute>,
) -> bool {
    let Some(attribute) = attribute else {
        return false;
    };
    let Some(uri) = element.namespace().uri() else {
        return false;
    };
    let applicable_namespace = [HTML_NAMESPACE_URI, SVG_NAMESPACE_URI, MATHML_NAMESPACE_URI]
        .into_iter()
        .any(|expected| {
            uri.len() == expected.len()
                && uri.code_units().iter().copied().eq(expected.encode_utf16())
        });
    if !applicable_namespace {
        return false;
    }
    let name = attribute.name();
    let expected = KnownHtmlAttribute::Autofocus.local_name();
    name.namespace().uri().is_none()
        && name.local_name().len() == expected.len()
        && name
            .local_name()
            .code_units()
            .iter()
            .copied()
            .eq(expected.encode_utf16())
}

/// A read or effect on the original inserted Element and actual Document locals.
/// The adapter retains identities and acknowledges only fulfilled captures/effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusInsertionAction {
    /// Interpret the original Element's actual autofocus Boolean attribute.
    ReadAutofocus,
    /// Read ordinary Document-tree membership at this insertion hook, rather
    /// than shadow-including connectedness or a post-connection notification.
    ReadInDocumentTree,
    /// Retain the original Element's actual node Document as target. Later
    /// reads use that target, even if execution suspends or the Element moves.
    CaptureTargetDocument,
    ReadFullyActive,
    /// Read the captured target's actual active sandboxing flag set.
    ReadSandboxAutomaticFeatures,
    /// Fulfill allow-focus steps for the captured target Document.
    ReadAllowFocus,
    /// Retain target's node navigable's top-level traversable's active Document
    /// at this source position, after permission, as topDocument.
    CaptureTargetTopDocument,
    /// Read the retained topDocument's actual autofocus processed flag.
    ReadAutofocusProcessed,
    /// Remove the original Element from the retained topDocument's candidates,
    /// including when no existing membership is present. Do not globally dedup.
    RemoveCandidate,
    /// Append the original Element to the same retained topDocument's candidates
    /// only after actual removal. There is no focusability or same-origin test.
    AppendCandidate,
    Complete,
}

/// Known facts and completed-effect acknowledgments for the exact pending stage.
/// Missing facts cannot be supplied as false or Applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusInsertionReply {
    Autofocus(bool),
    InDocumentTree(bool),
    FullyActive(bool),
    SandboxAutomaticFeatures(bool),
    AllowFocus(bool),
    AutofocusProcessed(bool),
    Applied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusInsertionError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlAutofocusInsertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending autofocus insertion action",
            Self::Terminal => "autofocus insertion is already complete",
        })
    }
}
impl std::error::Error for HtmlAutofocusInsertionError {}

/// Identity-free, allocation-free complete insertion language protocol.
///
/// Construct this for an actual Element insertion-steps invocation. Attribute
/// mutation alone is not this trigger. Native owners retain actual Element,
/// target/topDocument identities, admission and invocation custody. This cursor
/// owns no Document state, candidate registry, retirement policy or focus effect;
/// Complete means insertion bookkeeping returned, never rendering-time flushing.
/// Cloning gives no execution or replay authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAutofocusInsertionCursor {
    pending: HtmlAutofocusInsertionAction,
}
impl HtmlAutofocusInsertionCursor {
    pub const fn new() -> Self {
        Self {
            pending: HtmlAutofocusInsertionAction::ReadAutofocus,
        }
    }
    pub const fn action(&self) -> HtmlAutofocusInsertionAction {
        self.pending
    }
    /// Rejected wrong-stage and terminal replies leave the exact cursor intact.
    pub fn reply(
        &mut self,
        reply: HtmlAutofocusInsertionReply,
    ) -> Result<(), HtmlAutofocusInsertionError> {
        use HtmlAutofocusInsertionAction as A;
        use HtmlAutofocusInsertionReply as R;
        if self.pending == A::Complete {
            return Err(HtmlAutofocusInsertionError::Terminal);
        }
        let next = match (self.pending, reply) {
            (A::ReadAutofocus, R::Autofocus(false))
            | (A::ReadInDocumentTree, R::InDocumentTree(false))
            | (A::ReadFullyActive, R::FullyActive(false))
            | (A::ReadSandboxAutomaticFeatures, R::SandboxAutomaticFeatures(true))
            | (A::ReadAllowFocus, R::AllowFocus(false))
            | (A::ReadAutofocusProcessed, R::AutofocusProcessed(true)) => A::Complete,
            (A::ReadAutofocus, R::Autofocus(true)) => A::ReadInDocumentTree,
            (A::ReadInDocumentTree, R::InDocumentTree(true)) => A::CaptureTargetDocument,
            (A::CaptureTargetDocument, R::Applied) => A::ReadFullyActive,
            (A::ReadFullyActive, R::FullyActive(true)) => A::ReadSandboxAutomaticFeatures,
            (A::ReadSandboxAutomaticFeatures, R::SandboxAutomaticFeatures(false)) => {
                A::ReadAllowFocus
            }
            (A::ReadAllowFocus, R::AllowFocus(true)) => A::CaptureTargetTopDocument,
            (A::CaptureTargetTopDocument, R::Applied) => A::ReadAutofocusProcessed,
            (A::ReadAutofocusProcessed, R::AutofocusProcessed(false)) => A::RemoveCandidate,
            (A::RemoveCandidate, R::Applied) => A::AppendCandidate,
            (A::AppendCandidate, R::Applied) => A::Complete,
            _ => return Err(HtmlAutofocusInsertionError::WrongReply),
        };
        self.pending = next;
        Ok(())
    }
}
impl Default for HtmlAutofocusInsertionCursor {
    fn default() -> Self {
        Self::new()
    }
}
