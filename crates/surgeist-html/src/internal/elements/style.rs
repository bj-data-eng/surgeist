//! HTML877d native Style update and notification order; no CSS/CSP execution.
use crate::{
    internal::values::DocumentString, internal::values::Namespace, vocabulary::KnownHtmlAttribute,
};
use std::{error::Error, fmt};

/// The explicit processing set of a blocking attribute. Token splitting and
/// keyword comparison use the same rules as the ordinary value interpreter.
/// Poll before every cursor step and every keyword header/code-unit read; no
/// token buffer or normalized string is allocated.
pub fn blocking_tokens_with<E>(
    value: Option<&DocumentString>,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<crate::internal::attributes::token_declarations::HtmlBlockingTokens, E> {
    let Some(value) = value else {
        return Ok(crate::internal::attributes::token_declarations::HtmlBlockingTokens::Empty);
    };
    let mut cursor =
        crate::internal::microsyntax::tokens::HtmlSpaceTokenCursor::new(value.code_units());
    loop {
        step()?;
        match cursor.step() {
            crate::internal::microsyntax::tokens::HtmlSpaceTokenStep::Continue => {}
            crate::internal::microsyntax::tokens::HtmlSpaceTokenStep::Done => {
                return Ok(
                    crate::internal::attributes::token_declarations::HtmlBlockingTokens::Empty,
                );
            }
            crate::internal::microsyntax::tokens::HtmlSpaceTokenStep::Token { start, end } => {
                if blocking_token_is_render_with(&value.code_units()[start..end], &mut step)? {
                    return Ok(
                        crate::internal::attributes::token_declarations::HtmlBlockingTokens::Render,
                    );
                }
            }
        }
    }
}
pub(crate) fn blocking_token_is_render_with<E>(
    units: &[u16],
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    step()?;
    if units.len() != 6 {
        return Ok(false);
    }
    for (actual, expected) in units.iter().zip(b"render") {
        step()?;
        if !crate::internal::microsyntax::tokens::ascii_case_insensitive_unit(*actual, *expected) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The parser relation is an actual identity join supplied by the tree owner,
/// independently of the Style's historical creation state.
pub const fn style_is_potentially_render_blocking(
    explicit: crate::internal::attributes::token_declarations::HtmlBlockingTokens,
    state: HtmlStyleState,
    created_by_current_document_parser: bool,
) -> bool {
    matches!(
        explicit,
        crate::internal::attributes::token_declarations::HtmlBlockingTokens::Render
    ) || (state.created_by_document_parser() && created_by_current_document_parser)
}

/// HTML's Document gate for adding render-blocking Elements. Neither a missing
/// browsing context nor an unavailable body observation supplies these facts.
pub const fn document_allows_render_blocking(
    content_type_is_html: bool,
    body_present: bool,
) -> bool {
    content_type_is_html && !body_present
}

/// Name roles consumed by the actual Document.body tree selection: the first
/// document Element must be HTML html, then only its direct body/frameset child
/// is eligible. This value establishes no tree relation or body absence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDocumentBodyRole {
    /// The name is the HTML-namespace `html` element, the only element Document.body searches
    /// within.
    Html,
    /// The name is an HTML-namespace `body` or `frameset` element, the elements eligible as the
    /// body.
    Body,
    /// The name is neither: another local name or a name outside the HTML namespace.
    Other,
}
/// Classifies an element's expanded name for Document.body (HTML877d #dom-document-body). Only the
/// HTML namespace can name `html`, `body` or `frameset`; any other namespace is `Other`. `admit` is
/// called before each name read and its error ends the classification.
pub fn document_body_role_with<E>(
    name: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    mut admit: impl FnMut(
        crate::internal::elements::native_initialization::HtmlElementNameRead,
    ) -> Result<(), E>,
) -> Result<HtmlDocumentBodyRole, E> {
    use crate::internal::elements::native_initialization::{local_matches, namespace_matches};
    if namespace_matches(
        name,
        crate::internal::values::HTML_NAMESPACE_URI,
        &mut admit,
    )? != Some(true)
    {
        return Ok(HtmlDocumentBodyRole::Other);
    }
    if local_matches(name, "html", &mut admit)? {
        Ok(HtmlDocumentBodyRole::Html)
    } else if local_matches(name, "body", &mut admit)?
        || local_matches(name, "frameset", &mut admit)?
    {
        Ok(HtmlDocumentBodyRole::Body)
    } else {
        Ok(HtmlDocumentBodyRole::Other)
    }
}

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
    /// Whether the Style is on the open-element stack of its document parser.
    pub const fn on_parser_stack(self) -> bool {
        matches!(
            self.creation,
            StyleCreation::DocumentParser { on_stack: true }
        )
    }
    /// Whether a document parser created the Style. Implicit render blocking also needs that
    /// parser to be the Style's node document's parser, a relation that
    /// `style_is_potentially_render_blocking` takes from the tree owner. This fact persists after
    /// the parser pops the element.
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

/// The execution projection selected for this actual Style operation.
///
/// Consumers bind this choice to their real execution owner. Neither missing
/// observations, headlessness nor empty CSS establishes a predicate result.
/// The choice is retained immutably by the update and notification cursors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleExecutionProfile {
    /// The integration supplies actual script-blocking predicates and performs
    /// their registration/removal actions when required.
    SourcePredicates,
    /// The supported owner has no executable JavaScript. Only the selected
    /// executable-script bookkeeping is excluded; other Style predicates,
    /// rendering, policy, resources and event dispatch remain required.
    NoExecutableJavaScript,
}

/// The next owner action of the *update a style block* algorithm (HTML877d #update-a-style-block).
/// The consumer performs it and acknowledges it with `HtmlStyleUpdateReply`; the order is fixed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleUpdateAction {
    /// Remove the CSS style sheet the element currently has, if any. It precedes every guard.
    RemoveAssociatedSheet,
    /// Read whether the element is connected. A disconnected element completes the update.
    ReadConnected,
    /// Read whether the `type` attribute permits CSS (see `style_type_allows_css_with`).
    ReadType,
    /// Ask Content Security Policy whether this inline style is blocked.
    CheckInlineCsp,
    /// Create the CSS style sheet for the element.
    CreateSheet,
    /// Read whether the element is script-blocking. Under `NoExecutableJavaScript` the answer is
    /// `ScriptBookkeepingExcluded` instead of a boolean.
    ReadScriptBlocking,
    /// Register the element as script-blocking.
    RegisterScriptBlocking,
    /// Read whether the element is render-blocking.
    ReadRenderBlocking,
    /// Block rendering on the element.
    BlockRendering,
    /// The update is done; no further reply is accepted.
    Complete,
}
/// The consumer's answer to a `HtmlStyleUpdateAction`. Each variant answers specific actions, and a
/// mismatch is refused with `HtmlStyleProtocolError`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleUpdateReply {
    /// The pending removal, creation, registration or blocking action was performed.
    Applied,
    /// Whether the element is connected, answering `ReadConnected`.
    Connected(bool),
    /// Whether the `type` attribute permits CSS, answering `ReadType`.
    TypeEligible(bool),
    /// Whether Content Security Policy allows the inline style, answering `CheckInlineCsp`.
    InlineAllowed(bool),
    /// The script-blocking or render-blocking predicate, answering `ReadScriptBlocking` or
    /// `ReadRenderBlocking`. Script blocking is only valid under the `SourcePredicates` profile.
    Blocking(bool),
    /// Project only executable-script bookkeeping out of the reached script
    /// phase of a NoExecutableJavaScript operation. This is no boolean result.
    ScriptBookkeepingExcluded,
}
/// The refusal of a reply that does not match the cursor's profile and pending action. The cursor
/// is left unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlStyleProtocolError;
impl fmt::Display for HtmlStyleProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Style reply does not match the operation profile and reached action")
    }
}
impl Error for HtmlStyleProtocolError {}

/// Complete each actual owner action before acknowledging. Removal precedes ALL
/// guards. CSP receives exact direct-child Text, current Document and nonce/policy
/// context; neither no browsing context nor absent loader proves permission.
/// Fixed-size and allocation-free; it holds no consumer references, so every effect still needs
/// the consumer's own operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlStyleUpdateCursor {
    action: HtmlStyleUpdateAction,
    profile: HtmlStyleExecutionProfile,
}
impl HtmlStyleUpdateCursor {
    /// Start one operation with the consumer's actual execution profile.
    pub const fn new(profile: HtmlStyleExecutionProfile) -> Self {
        Self {
            action: HtmlStyleUpdateAction::RemoveAssociatedSheet,
            profile,
        }
    }
    /// Immutable execution qualification retained across every transition.
    pub const fn profile(self) -> HtmlStyleExecutionProfile {
        self.profile
    }
    /// The action the consumer must perform next, or `Complete` once the update is done.
    pub const fn action(self) -> HtmlStyleUpdateAction {
        self.action
    }
    /// Acknowledges the pending action. A mismatched reply, including one after `Complete`, or a
    /// script-blocking reply that does not fit the execution profile, fails with
    /// `HtmlStyleProtocolError` and leaves the cursor unchanged. A false connected, type or CSP
    /// answer completes the update.
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
            (A::ReadScriptBlocking, R::Blocking(true))
                if self.profile == HtmlStyleExecutionProfile::SourcePredicates =>
            {
                A::RegisterScriptBlocking
            }
            (A::ReadScriptBlocking, R::Blocking(false))
                if self.profile == HtmlStyleExecutionProfile::SourcePredicates =>
            {
                A::ReadRenderBlocking
            }
            (A::ReadScriptBlocking, R::ScriptBookkeepingExcluded)
                if self.profile == HtmlStyleExecutionProfile::NoExecutableJavaScript =>
            {
                A::ReadRenderBlocking
            }
            (A::RegisterScriptBlocking, R::Applied) => A::ReadRenderBlocking,
            (A::ReadRenderBlocking, R::Blocking(true)) => A::BlockRendering,
            (A::ReadRenderBlocking, R::Blocking(false)) | (A::BlockRendering, R::Applied) => {
                A::Complete
            }
            _ => return Err(HtmlStyleProtocolError),
        };
        Ok(())
    }
}
/// Exact absent/empty or ASCII-insensitive text/css; parameters are ineligible. `value` is the
/// `type` attribute's exact UTF-16 code units.
/// Each read code unit invokes the consumer's actual cancellation/admission hook.
pub fn style_type_allows_css_with<E>(
    value: Option<&[u16]>,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    let Some(value) = value else { return Ok(true) };
    if value.is_empty() {
        return Ok(true);
    }
    if value.len() != 8 {
        return Ok(false);
    }
    for (&actual, expected) in value.iter().zip("text/css".bytes()) {
        step()?;
        if !crate::internal::microsyntax::tokens::ascii_case_insensitive_unit(actual, expected) {
            return Ok(false);
        }
    }
    Ok(true)
}
/// What an attribute change on a Style element requires of its sheet. Only the null-namespace
/// `media` and `title` attributes affect it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleAttributeEffect {
    /// The attribute change does not affect the sheet.
    None,
    /// The attribute is the `media` or `title` attribute, which the associated sheet reads from its
    /// owner element; CSSOM defines what happens.
    OwnerAttributes,
}
/// Classifies an attribute change by the attribute's `namespace` and known name. The null-namespace
/// `media` and `title` attributes return `OwnerAttributes`; everything else returns `None`.
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
/// An event that may require a run of the *update a style block* algorithm (HTML877d
/// #update-a-style-block). `Insertion`, `Removal` and `ChildrenChanged` carry whether the element
/// is on a parser's open-element stack, which suppresses the update. `ParserPopped` carries
/// nothing and always requires the update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleTrigger {
    /// The element was inserted.
    Insertion {
        /// Whether the element became connected.
        connected: bool,
        /// Whether the element is on a parser's open-element stack.
        on_parser_stack: bool,
    },
    /// The element was removed.
    Removal {
        /// Whether the element became disconnected.
        disconnected: bool,
        /// Whether the element is on a parser's open-element stack.
        on_parser_stack: bool,
    },
    /// The element's children changed.
    ChildrenChanged {
        /// Whether the element is on a parser's open-element stack.
        on_parser_stack: bool,
    },
    /// A parser popped the element from its stack of open elements.
    ParserPopped,
}
/// Whether the trigger requires a run of the *update a style block* algorithm: a pop always does;
/// the others do only when the element is off a parser's stack, and insertion and removal also
/// require the element to have become connected or disconnected.
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
    /// Dispatch the `load` event at the element.
    DispatchLoad,
    /// Dispatch the `error` event at the element.
    DispatchError,
    /// Read whether the element is script-blocking, after the event handlers have run.
    ReadScriptBlocking,
    /// Remove the element from the script-blocking set.
    RemoveScriptBlocking,
    /// Unblock rendering on the element.
    UnblockRendering,
    /// The notification is done; no further reply is accepted.
    Complete,
}
/// The consumer's answer to a `HtmlStyleNotificationAction`. A mismatch is refused with
/// `HtmlStyleProtocolError`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlStyleNotificationReply {
    /// The pending dispatch, removal or unblocking action was performed.
    Applied,
    /// Whether the element is script-blocking, answering `ReadScriptBlocking`.
    ScriptBlocking(bool),
    /// The actual NoExecutableJavaScript operation excluded script-set
    /// bookkeeping. This neither observes nor removes membership.
    ScriptBookkeepingExcluded,
}
/// The event action must complete before reading current predicates. The
/// consumer permits event-handler reentry and then reads the live element's
/// current Document; queue-time owner or blocking observations are insufficient.
/// Fixed-size and allocation-free; it holds no consumer references, so every effect still needs
/// the consumer's own operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlStyleNotificationCursor {
    action: HtmlStyleNotificationAction,
    profile: HtmlStyleExecutionProfile,
}
impl HtmlStyleNotificationCursor {
    /// Queue-time success selects the real event; the execution profile only
    /// qualifies later script bookkeeping and cannot waive that dispatch.
    pub const fn new(profile: HtmlStyleExecutionProfile, success: bool) -> Self {
        Self {
            profile,
            action: if success {
                HtmlStyleNotificationAction::DispatchLoad
            } else {
                HtmlStyleNotificationAction::DispatchError
            },
        }
    }
    /// Immutable execution qualification of this notification operation.
    pub const fn profile(self) -> HtmlStyleExecutionProfile {
        self.profile
    }
    /// The action the consumer must perform next, or `Complete` once the notification is done.
    pub const fn action(self) -> HtmlStyleNotificationAction {
        self.action
    }
    /// Acknowledges the pending action. A mismatched reply, including one after `Complete`, or a
    /// script-blocking reply that does not fit the execution profile, fails with
    /// `HtmlStyleProtocolError` and leaves the cursor unchanged.
    pub fn reply(
        &mut self,
        reply: HtmlStyleNotificationReply,
    ) -> Result<(), HtmlStyleProtocolError> {
        use HtmlStyleNotificationAction as A;
        use HtmlStyleNotificationReply as R;
        self.action = match (self.action, reply) {
            (A::DispatchLoad | A::DispatchError, R::Applied) => A::ReadScriptBlocking,
            (A::ReadScriptBlocking, R::ScriptBlocking(true))
                if self.profile == HtmlStyleExecutionProfile::SourcePredicates =>
            {
                A::RemoveScriptBlocking
            }
            (A::ReadScriptBlocking, R::ScriptBlocking(false))
                if self.profile == HtmlStyleExecutionProfile::SourcePredicates =>
            {
                A::UnblockRendering
            }
            (A::ReadScriptBlocking, R::ScriptBookkeepingExcluded)
                if self.profile == HtmlStyleExecutionProfile::NoExecutableJavaScript =>
            {
                A::UnblockRendering
            }
            (A::RemoveScriptBlocking, R::Applied) => A::UnblockRendering,
            (A::UnblockRendering, R::Applied) => A::Complete,
            _ => return Err(HtmlStyleProtocolError),
        };
        Ok(())
    }
}
