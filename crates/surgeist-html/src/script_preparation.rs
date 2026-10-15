//! HTML 877d #prepare-the-script-element, through its disabled-scripting return.
//!
//! The adapter retains actual script/document identities, attributes, text,
//! connectivity, qualification and admission. These decisions neither establish
//! a scripting environment nor provide loading, compilation or execution.

use crate::HtmlScriptKind;

/// The source-defined terminal branch reached by the bounded preparation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptPreparationResult {
    AlreadyStarted,
    Empty,
    Disconnected,
    UnrecognizedType,
    ParserDocumentMismatch,
    Disabled(HtmlScriptKind),
}

/// One pending action, fulfilled by the actual qualified script owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptPreparationAction {
    ReadAlreadyStarted,
    /// Retain the actual old parser document separately, then persist null in
    /// the script slot. Reply only after that write has been admitted/applied.
    SaveAndClearParserDocument,
    ReadAsyncPresence,
    /// Apply HtmlScriptFlags::parser_document_cleared_without_async.
    SetForceAsync,
    /// Observe immediate child Text content, not descendant text or whitespace
    /// trimming. The supported prefix needs only established emptiness.
    ReadChildTextEmpty,
    /// Presence includes an empty src value; no URL or loading work is implied.
    ReadSrcPresence,
    ReadConnected,
    /// Use HtmlScriptTypeCursor::preparation on the actual type/language values.
    ClassifyType,
    StoreType(HtmlScriptKind),
    /// Restore the saved nonnull parser document and apply
    /// HtmlScriptFlags::parser_document_restored in the same source step14.
    RestoreParserDocument,
    /// Apply HtmlScriptFlags::mark_already_started, independently of step14/16.
    MarkAlreadyStarted,
    /// Read the current actual node document and persist it as preparation-time
    /// document. Do not infer identity from arena position or document class.
    StorePreparationDocument,
    CompareDocuments,
    /// The adapter must establish actual disabled scripting for this subject.
    /// Neither XML class, absent host facts nor script flags prove this fact.
    RequireDisabledScripting,
    Complete(HtmlScriptPreparationResult),
}

/// Established observations, or acknowledgement of the pending admitted write.
/// Missing evidence is explicit and never becomes false or an early return.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptPreparationReply {
    AlreadyStarted(bool),
    ParserDocumentSaved(bool),
    AsyncPresent(bool),
    ChildTextEmpty(bool),
    SrcPresent(bool),
    Connected(bool),
    /// None is a completed unrecognized classification, not incomplete work.
    Classified(Option<HtmlScriptKind>),
    DocumentsEqual(bool),
    Applied,
    Disabled,
    /// This actual environment requires an execution capability not provided
    /// by this bounded producer; it is not a successful preparation result.
    Enabled,
    Unavailable,
}

/// A refused transition; the pending action and all cursor state are preserved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptPreparationError {
    ReplyMismatch,
    EvidenceUnavailable,
    ScriptingEnabled,
}
impl std::fmt::Display for HtmlScriptPreparationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ReplyMismatch => "script preparation reply does not match its pending action",
            Self::EvidenceUnavailable => "script preparation evidence is unavailable",
            Self::ScriptingEnabled => "script preparation requires unsupported enabled scripting",
        })
    }
}
impl std::error::Error for HtmlScriptPreparationError {}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Started,
    Clear,
    Async,
    ForceAsync,
    Text,
    Src,
    Connected,
    Classify,
    Type(HtmlScriptKind),
    Restore(HtmlScriptKind),
    Mark(HtmlScriptKind),
    PreparationDocument(HtmlScriptKind),
    Compare(HtmlScriptKind),
    Disabled(HtmlScriptKind),
    Complete(HtmlScriptPreparationResult),
}

/// Allocation-free, constant-state decisions for one actual preparation call.
///
/// The adapter admits every actual read/write before replying. It owns the saved
/// parser document and uses existing HtmlScriptFlags transitions for flag writes.
/// Acknowledgements do not authorize callbacks or external reentry between staged
/// writes. The adapter retains coherent subject/document facts, stops on missing
/// evidence and publishes only according to its own transaction/cleanup contract.
/// Native creation, parser insertion, cloning and adoption remain distinct hooks.
#[derive(Clone, Copy, Debug)]
pub struct HtmlScriptPreparationCursor {
    phase: Phase,
    saved_parser_document: bool,
}
impl Default for HtmlScriptPreparationCursor {
    fn default() -> Self {
        Self::new()
    }
}
impl HtmlScriptPreparationCursor {
    pub fn new() -> Self {
        Self {
            phase: Phase::Started,
            saved_parser_document: false,
        }
    }
    pub fn action(&self) -> HtmlScriptPreparationAction {
        match self.phase {
            Phase::Started => HtmlScriptPreparationAction::ReadAlreadyStarted,
            Phase::Clear => HtmlScriptPreparationAction::SaveAndClearParserDocument,
            Phase::Async => HtmlScriptPreparationAction::ReadAsyncPresence,
            Phase::ForceAsync => HtmlScriptPreparationAction::SetForceAsync,
            Phase::Text => HtmlScriptPreparationAction::ReadChildTextEmpty,
            Phase::Src => HtmlScriptPreparationAction::ReadSrcPresence,
            Phase::Connected => HtmlScriptPreparationAction::ReadConnected,
            Phase::Classify => HtmlScriptPreparationAction::ClassifyType,
            Phase::Type(kind) => HtmlScriptPreparationAction::StoreType(kind),
            Phase::Restore(_) => HtmlScriptPreparationAction::RestoreParserDocument,
            Phase::Mark(_) => HtmlScriptPreparationAction::MarkAlreadyStarted,
            Phase::PreparationDocument(_) => HtmlScriptPreparationAction::StorePreparationDocument,
            Phase::Compare(_) => HtmlScriptPreparationAction::CompareDocuments,
            Phase::Disabled(_) => HtmlScriptPreparationAction::RequireDisabledScripting,
            Phase::Complete(result) => HtmlScriptPreparationAction::Complete(result),
        }
    }
    pub fn reply(
        &mut self,
        reply: HtmlScriptPreparationReply,
    ) -> Result<(), HtmlScriptPreparationError> {
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }
    fn advance(
        &mut self,
        reply: HtmlScriptPreparationReply,
    ) -> Result<(), HtmlScriptPreparationError> {
        use HtmlScriptPreparationReply as Reply;
        use HtmlScriptPreparationResult as Result;
        if reply == Reply::Unavailable
            && matches!(
                self.phase,
                Phase::Started
                    | Phase::Async
                    | Phase::Text
                    | Phase::Src
                    | Phase::Connected
                    | Phase::Classify
                    | Phase::Compare(_)
                    | Phase::Disabled(_)
            )
        {
            return Err(HtmlScriptPreparationError::EvidenceUnavailable);
        }
        self.phase = match (self.phase, reply) {
            (Phase::Started, Reply::AlreadyStarted(true)) => {
                Phase::Complete(Result::AlreadyStarted)
            }
            (Phase::Started, Reply::AlreadyStarted(false)) => Phase::Clear,
            (Phase::Clear, Reply::ParserDocumentSaved(present)) => {
                self.saved_parser_document = present;
                if present { Phase::Async } else { Phase::Text }
            }
            (Phase::Async, Reply::AsyncPresent(true)) => Phase::Text,
            (Phase::Async, Reply::AsyncPresent(false)) => Phase::ForceAsync,
            (Phase::ForceAsync, Reply::Applied) => Phase::Text,
            (Phase::Text, Reply::ChildTextEmpty(true)) => Phase::Src,
            (Phase::Text, Reply::ChildTextEmpty(false)) => Phase::Connected,
            (Phase::Src, Reply::SrcPresent(true)) => Phase::Connected,
            (Phase::Src, Reply::SrcPresent(false)) => Phase::Complete(Result::Empty),
            (Phase::Connected, Reply::Connected(true)) => Phase::Classify,
            (Phase::Connected, Reply::Connected(false)) => Phase::Complete(Result::Disconnected),
            (Phase::Classify, Reply::Classified(None)) => Phase::Complete(Result::UnrecognizedType),
            (Phase::Classify, Reply::Classified(Some(kind))) => Phase::Type(kind),
            (Phase::Type(kind), Reply::Applied) => {
                if self.saved_parser_document {
                    Phase::Restore(kind)
                } else {
                    Phase::Mark(kind)
                }
            }
            (Phase::Restore(kind), Reply::Applied) => Phase::Mark(kind),
            (Phase::Mark(kind), Reply::Applied) => Phase::PreparationDocument(kind),
            (Phase::PreparationDocument(kind), Reply::Applied) => {
                if self.saved_parser_document {
                    Phase::Compare(kind)
                } else {
                    Phase::Disabled(kind)
                }
            }
            (Phase::Compare(kind), Reply::DocumentsEqual(true)) => Phase::Disabled(kind),
            (Phase::Compare(_), Reply::DocumentsEqual(false)) => {
                Phase::Complete(Result::ParserDocumentMismatch)
            }
            (Phase::Disabled(kind), Reply::Disabled) => Phase::Complete(Result::Disabled(kind)),
            (Phase::Disabled(_), Reply::Enabled) => {
                return Err(HtmlScriptPreparationError::ScriptingEnabled);
            }
            _ => return Err(HtmlScriptPreparationError::ReplyMismatch),
        };
        Ok(())
    }
}
