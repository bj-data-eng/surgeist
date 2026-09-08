use super::{CommandKind, Id};
use std::{error, fmt};

/// Result returned by public window operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Structured failure for semantic validation, host support, backend work, and callbacks.
///
/// The code and optional command/window metadata identify the contract failure.
/// `message` is diagnostic text, not a promise of platform-native wording.
/// Handler failures become terminal loop results while retaining this structured
/// error instead of being rewritten as native platform text.
#[derive(Debug)]
pub struct Error {
    /// Stable semantic classification of this failure.
    pub code: ErrorCode,
    /// Human-readable diagnostic text supplied by this crate or a host adapter.
    pub message: String,
    /// Affected window identity, when the failed operation has one.
    pub id: Option<Id>,
    /// Authored command classification, when a command caused the failure.
    pub command_kind: Option<CommandKind>,
    /// Number of successfully committed commands before a batch backend failure.
    ///
    /// The failing command is at this zero-based index; it and every later
    /// command are discarded, while the preceding prefix remains committed.
    pub completed_prefix: usize,
    /// Retained underlying error, including the backend source of a batch failure.
    pub source: Option<Box<dyn error::Error + Send + Sync>>,
}

impl Error {
    /// Creates an error without window, command, batch, or source metadata.
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            id: None,
            command_kind: None,
            completed_prefix: 0,
            source: None,
        }
    }

    /// Associates this error with the affected window identity.
    #[must_use]
    pub fn with_id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Associates this error with the authored command's stable classification.
    #[must_use]
    pub fn with_command_kind(mut self, command_kind: CommandKind) -> Self {
        self.command_kind = Some(command_kind);
        self
    }

    /// Records the committed prefix length for a failed backend command batch.
    #[must_use]
    pub const fn with_completed_prefix(mut self, completed_prefix: usize) -> Self {
        self.completed_prefix = completed_prefix;
        self
    }

    /// Retains an underlying error as this error's standard source chain.
    #[must_use]
    pub fn with_source(mut self, source: impl error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = self.id {
            write!(
                f,
                "{:?} for window {}: {}",
                self.code,
                id.as_u64(),
                self.message
            )
        } else {
            write!(f, "{:?}: {}", self.code, self.message)
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn error::Error + 'static))
    }
}

/// Stable semantic classification for a failed operation.
///
/// This enum is intentionally exhaustive. Adding a variant is a deliberate
/// breaking change so downstream exhaustive matches keep pace with the window
/// contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// Intrinsic caller input or a requested state transition is invalid.
    InvalidRequest,
    /// A requested window identity or live name collides with one already issued.
    DuplicateIdentity,
    /// No further never-reused runtime identity can be allocated.
    IdentityExhausted,
    /// A synchronous operation targeted an unknown or closed window generation.
    StaleWindow,
    /// A synchronous operation targeted a known generation that is closing.
    WindowClosing,
    /// Native event-loop construction failed before application callbacks run.
    EventLoopCreateFailed,
    /// A backend could not create the requested native window.
    WindowCreateFailed,
    /// A native raw handle is unavailable for its lease's current lifecycle state.
    HandleUnavailable,
    /// A well-formed IME request is unsupported by the current host capabilities.
    ImeUnsupported,
    /// A backend failed while applying an IME request.
    ImeRequestFailed,
    /// Clipboard access is unavailable from the current host.
    ClipboardUnavailable,
    /// The clipboard host failed while reading data.
    ClipboardReadFailed,
    /// The clipboard host failed while writing data; completed writes are not rolled back.
    ClipboardWriteFailed,
    /// A backend failed while applying a cursor request.
    CursorRequestFailed,
    /// A single command could not be queued, planned, or applied by the host.
    CommandFailed,
    /// A backend failed after committing a batch prefix; source and metadata identify the failure.
    CommandBatchFailed,
    /// The event loop ended while an external native handle lease remained.
    OutstandingHandle,
    /// A well-formed operation is unavailable for the active host or feature set.
    UnsupportedFeature,
    /// The private accessibility adapter failed while handling a typed update.
    AccessibilityAdapterFailed,
    /// A terminal native/backend failure that has no more specific public classification.
    UnknownNativeError,
}
