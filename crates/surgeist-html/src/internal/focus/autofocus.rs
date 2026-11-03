//! HTML877d autofocus insertion bookkeeping (raw60684–60707).
//!
//! Ordinary Document-tree insertion is defined at raw1479–1485. This producer
//! elects unconditional continuation past the optional user-preference return;
//! it neither observes a preference nor supplies a preference service.

use crate::internal::values::{HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, SVG_NAMESPACE_URI};
use crate::{internal::values::semantic_input::HtmlNameInput, vocabulary::KnownHtmlAttribute};

/// Recognize one actual autofocus content attribute for an Element
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
pub fn autofocus_presence(
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    attribute: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
) -> bool {
    let Some(attribute) = attribute else {
        return false;
    };
    let Some(uri) = element.namespace_uri() else {
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
    name.namespace_uri().is_none()
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
    /// Read whether the captured target Document is fully active; if it is not, the insertion steps
    /// end.
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
    /// The insertion steps have returned; no further reply is accepted.
    Complete,
}

/// Known facts and completed-effect acknowledgments for the exact pending stage.
/// Missing facts cannot be supplied as false or Applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusInsertionReply {
    /// Whether the element has the `autofocus` attribute, answering `ReadAutofocus`.
    Autofocus(bool),
    /// Whether the element is in a document tree, answering `ReadInDocumentTree`.
    InDocumentTree(bool),
    /// Whether the target Document is fully active, answering `ReadFullyActive`.
    FullyActive(bool),
    /// Whether the target Document's active sandboxing flag set has the sandboxed automatic
    /// features browsing context flag, answering `ReadSandboxAutomaticFeatures`. True ends the
    /// insertion steps.
    SandboxAutomaticFeatures(bool),
    /// Whether the allow focus steps permit focusing in the target Document, answering
    /// `ReadAllowFocus`.
    AllowFocus(bool),
    /// Whether the top Document's autofocus processed flag is set, answering
    /// `ReadAutofocusProcessed`. True ends the insertion steps (HTML877d
    /// #autofocus-processed-flag).
    AutofocusProcessed(bool),
    /// The pending capture or candidate removal or append was performed.
    Applied,
}

/// The refusal of a reply: either it does not match the pending action (`is_wrong_reply`), or the
/// cursor is already complete (`is_terminal`). The cursor is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAutofocusInsertionError {
    kind: HtmlAutofocusInsertionErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlAutofocusInsertionErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlAutofocusInsertionErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlAutofocusInsertionError {
        HtmlAutofocusInsertionError { kind: self }
    }
}

impl From<HtmlAutofocusInsertionErrorKind> for HtmlAutofocusInsertionError {
    fn from(kind: HtmlAutofocusInsertionErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlAutofocusInsertionError {
    /// Whether the cause is: reply does not match the pending autofocus insertion action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlAutofocusInsertionErrorKind::WrongReply)
    }
    /// Whether the cause is: autofocus insertion is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlAutofocusInsertionErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlAutofocusInsertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlAutofocusInsertionErrorKind::WrongReply => {
                f.write_str("reply does not match the pending autofocus insertion action")
            }
            HtmlAutofocusInsertionErrorKind::Terminal => {
                f.write_str("autofocus insertion is already complete")
            }
        }
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
    /// Starts the insertion steps, with `ReadAutofocus` pending.
    pub const fn new() -> Self {
        Self {
            pending: HtmlAutofocusInsertionAction::ReadAutofocus,
        }
    }
    /// The action the owner must perform next, or `Complete` once the insertion steps have
    /// returned.
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
            return Err(HtmlAutofocusInsertionErrorKind::Terminal.error());
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
            _ => return Err(HtmlAutofocusInsertionErrorKind::WrongReply.error()),
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

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_autofocus_insertion_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlAutofocusInsertionError) -> bool;
        let cases: Vec<(HtmlAutofocusInsertionError, &str, Predicate)> = vec![
            (
                HtmlAutofocusInsertionErrorKind::WrongReply.error(),
                "reply does not match the pending autofocus insertion action",
                HtmlAutofocusInsertionError::is_wrong_reply,
            ),
            (
                HtmlAutofocusInsertionErrorKind::Terminal.error(),
                "autofocus insertion is already complete",
                HtmlAutofocusInsertionError::is_terminal,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}
