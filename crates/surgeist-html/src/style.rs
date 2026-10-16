//! HTML877d native Style update and notification order; no CSS/CSP execution.
use crate::{DocumentString, KnownHtmlAttribute, Namespace};
use std::{error::Error, fmt};

/// Actual intrinsic creation and parser-stack state. This value carries no
/// parser Document identity or associated CSSOM-sheet readiness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlStyleState {
    creation: StyleCreation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StyleCreation {
    Other,
    DocumentParser { on_stack: bool },
}
impl HtmlStyleState {
    /// Script/native creation and general cloning have no parser history.
    pub const fn fresh() -> Self {
        Self {
            creation: StyleCreation::Other,
        }
    }
    /// Actual parser allocation precedes any push onto its open-element stack.
    /// The parser owner separately retains its real parser Document relation.
    pub const fn document_parser_created() -> Self {
        Self {
            creation: StyleCreation::DocumentParser { on_stack: false },
        }
    }
    pub const fn on_parser_stack(self) -> bool {
        matches!(
            self.creation,
            StyleCreation::DocumentParser { on_stack: true }
        )
    }
    pub const fn created_by_document_parser(self) -> bool {
        matches!(self.creation, StyleCreation::DocumentParser { .. })
    }
    /// Apply an actual parser push. Other creation causes cannot enter this state.
    pub fn parser_stack_added(&mut self) -> Result<(), HtmlStyleParserStateError> {
        let StyleCreation::DocumentParser { on_stack } = &mut self.creation else {
            return Err(HtmlStyleParserStateError);
        };
        *on_stack = true;
        Ok(())
    }
    /// Apply an actual parser pop, independently of stylesheet processing.
    pub fn parser_stack_removed(&mut self) -> Result<(), HtmlStyleParserStateError> {
        let StyleCreation::DocumentParser { on_stack } = &mut self.creation else {
            return Err(HtmlStyleParserStateError);
        };
        *on_stack = false;
        Ok(())
    }
}
impl Default for HtmlStyleState {
    fn default() -> Self {
        Self::fresh()
    }
}
/// A non-parser Style cannot be placed on a parser's open-element stack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlStyleParserStateError;
impl fmt::Display for HtmlStyleParserStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Style was not created by a document parser")
    }
}
impl Error for HtmlStyleParserStateError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleUpdateAction {
    RemoveAssociatedSheet,
    ReadConnected,
    ReadType,
    CheckInlineCsp,
    CreateSheet,
    ReadScriptBlocking,
    RegisterScriptBlocking,
    ReadRenderBlocking,
    BlockRendering,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleUpdateReply {
    Applied,
    Connected(bool),
    TypeEligible(bool),
    InlineAllowed(bool),
    Blocking(bool),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlStyleProtocolError;
impl fmt::Display for HtmlStyleProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Style reply does not match the reached source action")
    }
}
impl Error for HtmlStyleProtocolError {}

/// Complete each actual owner action before acknowledging. Removal precedes ALL
/// guards. CSP receives exact direct-child Text, current Document and nonce/policy
/// context; neither no browsing context nor absent loader proves permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlStyleUpdateCursor {
    action: HtmlStyleUpdateAction,
}
impl Default for HtmlStyleUpdateCursor {
    fn default() -> Self {
        Self::new()
    }
}
impl HtmlStyleUpdateCursor {
    pub const fn new() -> Self {
        Self {
            action: HtmlStyleUpdateAction::RemoveAssociatedSheet,
        }
    }
    pub const fn action(self) -> HtmlStyleUpdateAction {
        self.action
    }
    pub fn reply(&mut self, reply: HtmlStyleUpdateReply) -> Result<(), HtmlStyleProtocolError> {
        use HtmlStyleUpdateAction as A;
        use HtmlStyleUpdateReply as R;
        self.action = match (self.action, reply) {
            (A::RemoveAssociatedSheet, R::Applied) => A::ReadConnected,
            (A::ReadConnected, R::Connected(true)) => A::ReadType,
            (A::ReadConnected, R::Connected(false)) => A::Complete,
            (A::ReadType, R::TypeEligible(true)) => A::CheckInlineCsp,
            (A::ReadType, R::TypeEligible(false)) => A::Complete,
            (A::CheckInlineCsp, R::InlineAllowed(true)) => A::CreateSheet,
            (A::CheckInlineCsp, R::InlineAllowed(false)) => A::Complete,
            (A::CreateSheet, R::Applied) => A::ReadScriptBlocking,
            (A::ReadScriptBlocking, R::Blocking(true)) => A::RegisterScriptBlocking,
            (A::ReadScriptBlocking, R::Blocking(false))
            | (A::RegisterScriptBlocking, R::Applied) => A::ReadRenderBlocking,
            (A::ReadRenderBlocking, R::Blocking(true)) => A::BlockRendering,
            (A::ReadRenderBlocking, R::Blocking(false)) | (A::BlockRendering, R::Applied) => {
                A::Complete
            }
            _ => return Err(HtmlStyleProtocolError),
        };
        Ok(())
    }
}
/// Exact absent/empty or ASCII-insensitive text/css; parameters are ineligible.
/// Each read code unit invokes the consumer's actual cancellation/admission hook.
pub fn style_type_allows_css_with<E>(
    value: Option<&DocumentString>,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    let Some(value) = value else { return Ok(true) };
    if value.is_empty() {
        return Ok(true);
    }
    if value.len() != 8 {
        return Ok(false);
    }
    for (&actual, expected) in value.code_units().iter().zip("text/css".encode_utf16()) {
        step()?;
        let actual = if (65..=90).contains(&actual) {
            actual + 32
        } else {
            actual
        };
        if actual != expected {
            return Ok(false);
        }
    }
    Ok(true)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleAttributeEffect {
    None,
    OwnerAttributes,
}
pub fn style_attribute_effect(
    namespace: &Namespace,
    attribute: KnownHtmlAttribute,
) -> HtmlStyleAttributeEffect {
    if namespace.uri().is_none()
        && matches!(
            attribute,
            KnownHtmlAttribute::Media | KnownHtmlAttribute::Title
        )
    {
        HtmlStyleAttributeEffect::OwnerAttributes
    } else {
        HtmlStyleAttributeEffect::None
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleTrigger {
    Insertion {
        connected: bool,
        on_parser_stack: bool,
    },
    Removal {
        disconnected: bool,
        on_parser_stack: bool,
    },
    ChildrenChanged {
        on_parser_stack: bool,
    },
    ParserPopped,
}
pub const fn style_update_is_required(trigger: HtmlStyleTrigger) -> bool {
    match trigger {
        HtmlStyleTrigger::Insertion {
            connected,
            on_parser_stack,
        } => connected && !on_parser_stack,
        HtmlStyleTrigger::Removal {
            disconnected,
            on_parser_stack,
        } => disconnected && !on_parser_stack,
        HtmlStyleTrigger::ChildrenChanged { on_parser_stack } => !on_parser_stack,
        HtmlStyleTrigger::ParserPopped => true,
    }
}
/// Body of the actual queued networking task, not permission to queue it before
/// real parsing/processing and critical-resource outcome proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleNotificationAction {
    DispatchLoad,
    DispatchError,
    ReadScriptBlocking,
    RemoveScriptBlocking,
    UnblockRendering,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleNotificationReply {
    Applied,
    ScriptBlocking(bool),
}
/// The event action must complete before reading current predicates. The
/// consumer permits event-handler reentry and then reads the live element's
/// current Document; queue-time owner or blocking observations are insufficient.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlStyleNotificationCursor {
    action: HtmlStyleNotificationAction,
}
impl HtmlStyleNotificationCursor {
    pub const fn new(success: bool) -> Self {
        Self {
            action: if success {
                HtmlStyleNotificationAction::DispatchLoad
            } else {
                HtmlStyleNotificationAction::DispatchError
            },
        }
    }
    pub const fn action(self) -> HtmlStyleNotificationAction {
        self.action
    }
    pub fn reply(
        &mut self,
        reply: HtmlStyleNotificationReply,
    ) -> Result<(), HtmlStyleProtocolError> {
        use HtmlStyleNotificationAction as A;
        use HtmlStyleNotificationReply as R;
        self.action = match (self.action, reply) {
            (A::DispatchLoad | A::DispatchError, R::Applied) => A::ReadScriptBlocking,
            (A::ReadScriptBlocking, R::ScriptBlocking(true)) => A::RemoveScriptBlocking,
            (A::ReadScriptBlocking, R::ScriptBlocking(false))
            | (A::RemoveScriptBlocking, R::Applied) => A::UnblockRendering,
            (A::UnblockRendering, R::Applied) => A::Complete,
            _ => return Err(HtmlStyleProtocolError),
        };
        Ok(())
    }
}
