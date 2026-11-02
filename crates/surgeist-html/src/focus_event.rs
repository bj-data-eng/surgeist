//! Complete fire-a-focus-event protocol, WHATWG HTML revision
//! 877d6146487f15b15bc8dd0e3c625063a37b456a, retained raw60300–60311.
//!
//! The actual original event name, target and nullable related target remain
//! adapter-owned. The source accepts an arbitrary event name; the Focus/Blur
//! vocabulary below is the focus-update caller's bounded name selection.

/// Event names selected by focus-update, rather than a restriction on the
/// original event-name operand of fire-a-focus-event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusEventName {
    Blur,
    Focus,
}
impl HtmlFocusEventName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blur => "blur",
            Self::Focus => "focus",
        }
    }
}

/// Actual classification of the source's Element-or-Window target. This does
/// not classify a focusable area's anchor in place of that event target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusEventTargetKind {
    Element,
    Window,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusEventView {
    TargetWindow,
    ElementDocumentGlobal,
}
/// Required interface of the source event constructor, distinct from Event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusEventInterface {
    FocusEvent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusEventAction {
    /// Retain the actual original Window target itself as view.
    CaptureWindowView,
    /// At this source position retain the actual Element target's current node
    /// Document's relevant Global as view. Do not use an earlier queue Document,
    /// captured area anchor, saved designation Document or desired Window.
    CaptureElementDocumentGlobalView,
    /// Fire the original name at the original target using FocusEvent, original
    /// nullable relatedTarget and retained actual view. Fulfill authentic dispatch
    /// and its callbacks before replying Dispatched. Flags are source values,
    /// including the false bubbles/cancelable defaults.
    Dispatch {
        interface: HtmlFocusEventInterface,
        view: HtmlFocusEventView,
        bubbles: bool,
        cancelable: bool,
        composed: bool,
    },
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusEventReply {
    /// Only the pending view capture actually completed.
    Applied,
    /// The exact actual event dispatch and its incurred work completed.
    Dispatched,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFocusEventError {
    kind: HtmlFocusEventErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlFocusEventErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlFocusEventErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlFocusEventError {
        HtmlFocusEventError { kind: self }
    }
}

impl From<HtmlFocusEventErrorKind> for HtmlFocusEventError {
    fn from(kind: HtmlFocusEventErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlFocusEventError {
    /// Whether the cause is: reply does not match the pending fire-focus-event action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlFocusEventErrorKind::WrongReply)
    }
    /// Whether the cause is: fire-focus-event is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlFocusEventErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlFocusEventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlFocusEventErrorKind::WrongReply => {
                f.write_str("reply does not match the pending fire-focus-event action")
            }
            HtmlFocusEventErrorKind::Terminal => {
                f.write_str("fire-focus-event is already complete")
            }
        }
    }
}

impl std::error::Error for HtmlFocusEventError {}

/// Constant-state, non-Copy source protocol with no identity, dispatch or host
/// ownership. Actual operands, view, admission and authentic invocation custody
/// belong to the original adapter. No callback interface or event service is
/// supplied here. Unknown/unrepresentable views make no reply.
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlFocusEventCursor {
    pending: HtmlFocusEventAction,
}
impl HtmlFocusEventCursor {
    pub const fn new(target_kind: HtmlFocusEventTargetKind) -> Self {
        Self {
            pending: match target_kind {
                HtmlFocusEventTargetKind::Window => HtmlFocusEventAction::CaptureWindowView,
                HtmlFocusEventTargetKind::Element => {
                    HtmlFocusEventAction::CaptureElementDocumentGlobalView
                }
            },
        }
    }
    pub const fn action(&self) -> HtmlFocusEventAction {
        self.pending
    }
    /// Wrong-stage and terminal replies leave every cursor field unchanged.
    pub fn reply(&mut self, reply: HtmlFocusEventReply) -> Result<(), HtmlFocusEventError> {
        use HtmlFocusEventAction as A;
        use HtmlFocusEventReply as R;
        if self.pending == A::Complete {
            return Err(HtmlFocusEventErrorKind::Terminal.error());
        }
        let next = match (self.pending, reply) {
            (A::CaptureWindowView, R::Applied) => A::Dispatch {
                interface: HtmlFocusEventInterface::FocusEvent,
                view: HtmlFocusEventView::TargetWindow,
                bubbles: false,
                cancelable: false,
                composed: true,
            },
            (A::CaptureElementDocumentGlobalView, R::Applied) => A::Dispatch {
                interface: HtmlFocusEventInterface::FocusEvent,
                view: HtmlFocusEventView::ElementDocumentGlobal,
                bubbles: false,
                cancelable: false,
                composed: true,
            },
            (A::Dispatch { .. }, R::Dispatched) => A::Complete,
            _ => return Err(HtmlFocusEventErrorKind::WrongReply.error()),
        };
        self.pending = next;
        Ok(())
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_focus_event_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlFocusEventError) -> bool;
        let cases: Vec<(HtmlFocusEventError, &str, Predicate)> = vec![
            (
                HtmlFocusEventErrorKind::WrongReply.error(),
                "reply does not match the pending fire-focus-event action",
                HtmlFocusEventError::is_wrong_reply,
            ),
            (
                HtmlFocusEventErrorKind::Terminal.error(),
                "fire-focus-event is already complete",
                HtmlFocusEventError::is_terminal,
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
