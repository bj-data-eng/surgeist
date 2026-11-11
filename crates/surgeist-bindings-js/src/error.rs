use std::fmt;

use crate::conversion::{CleanupFailure, LedgerError, Rule};
use crate::{HostFailure, Root};

/// Operation that produced a guest exception.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExceptionPhase {
    Compile,
    Evaluate,
    PropertyRead,
    PropertyWrite,
    DefineProperty,
    Prototype,
    Invoke,
    Coerce,
}

/// An original guest-thrown value, retained without invoking guest diagnostics.
#[derive(Clone, Debug)]
pub struct Exception {
    pub(crate) phase: ExceptionPhase,
    pub(crate) value: Root,
    pub(crate) rule: Option<Rule>,
}

impl Exception {
    /// Boundary at which this exception arose.
    pub fn phase(&self) -> ExceptionPhase {
        self.phase
    }

    /// Original thrown value; import into a scope of its owning heap.
    pub fn value(&self) -> &Root {
        &self.value
    }

    /// Specified binding validation that created this intrinsic TypeError.
    /// This annotates the direct conversion failure. Values captured again
    /// across guest execution, including guest rethrows, have no host rule.
    pub fn conversion_rule(&self) -> Option<Rule> {
        self.rule
    }
}

/// Typed refusal before invalid engine access, or a captured guest failure.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Original typed native host failure, distinct from a guest exception.
    HostFailure(HostFailure),
    /// Binding-owned conversion work or storage was refused by Ledger.
    ConversionLedger(LedgerError),
    /// Fatal cleanup failed; its selected primary, if any, is retained exactly.
    ConversionCleanup(CleanupFailure),
    /// Bootstrap is already held by another live process token.
    OwnerAlreadyActive,
    /// Only the thread that initialized the platform may acquire this backend.
    WrongThread,
    /// Realm or value belongs to a different or already destroyed heap.
    ForeignOwner,
    /// A previously valid identity is no longer available.
    ExpiredIdentity,
    /// A monotonic identity cannot advance without reuse.
    IdentityExhausted,
    /// A Rust-side allocation was refused.
    Allocation,
    /// A string exceeds the backend's supported code-unit bound.
    StringTooLong,
    /// The argument list exceeds the backend's supported call bound.
    TooManyArguments,
    /// An array exceeds the embedding policy's conservative pinned SDK capacity.
    ArrayTooLong,
    /// A dynamic property key was neither String nor Symbol.
    ExpectedPropertyKey,
    /// A descriptor accessor was neither callable nor undefined.
    ExpectedAccessor,
    /// This operation requires an object or function, respectively.
    ExpectedObject,
    /// This operation requires an exact native object of the requested type.
    ExpectedNativeObject,
    ExpectedFunction,
    /// This object has no supported associated engine creation Realm.
    CallbackRealmUnavailable,
    /// Web IDL EnforceRange rejected a nonfinite or out-of-range Number.
    /// Installed JavaScript interfaces map this category to TypeError.
    IntegerOutOfRange,
    /// Web IDL float/double conversion rejected a nonfinite value or float overflow.
    /// Installed JavaScript interfaces map this category to TypeError.
    FloatOutOfRange,
    /// Web IDL ByteString conversion encountered a code unit greater than 255.
    /// Installed JavaScript interfaces map this category to TypeError.
    ByteStringOutOfRange,
    /// Guest Set rejected the property update.
    PropertyRejected,
    /// Execution was terminated by the backend.
    Terminated,
    /// The entered realm has no installed DOMException interface.
    DomExceptionUnavailable,
    /// The backend could not complete and supplied no guest exception.
    BackendUnavailable,
    /// Internal root-registry access overlapped. The owner fails closed after
    /// this defect rather than entering the SDK with inconsistent storage.
    RegistryConflict,
    /// A native callback panicked. The owner is terminal and cannot be resumed.
    CallbackPanicked,
    /// The original guest-thrown value and operation phase.
    Exception(Exception),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::HostFailure(_) => "native host refused execution",
            Self::ConversionLedger(_) => "binding conversion admission failed",
            Self::ConversionCleanup(_) => "binding conversion cleanup failed",
            Self::OwnerAlreadyActive => "an execution owner is already active",
            Self::WrongThread => "the backend belongs to a different thread",
            Self::ForeignOwner => "the capability belongs to a different heap",
            Self::ExpiredIdentity => "the capability has expired",
            Self::IdentityExhausted => "execution identity space is exhausted",
            Self::Allocation => "binding storage allocation was refused",
            Self::StringTooLong => "the string exceeds the backend limit",
            Self::ArrayTooLong => "the array exceeds the backend element limit",
            Self::ExpectedPropertyKey => "a property key must be String or Symbol",
            Self::ExpectedAccessor => "an accessor must be callable or undefined",
            Self::TooManyArguments => "the argument list exceeds the backend limit",
            Self::ExpectedObject => "an object is required",
            Self::ExpectedNativeObject => "the exact native object is required",
            Self::ExpectedFunction => "a callable function is required",
            Self::CallbackRealmUnavailable => {
                "the callback object's associated realm is unavailable"
            }
            Self::IntegerOutOfRange => "the Number is outside the Web IDL integer range",
            Self::FloatOutOfRange => "the Number is outside the Web IDL floating point range",
            Self::ByteStringOutOfRange => "the string contains a code unit outside the byte range",
            Self::PropertyRejected => "the object rejected the property update",
            Self::Terminated => "JavaScript execution was terminated",
            Self::DomExceptionUnavailable => "DOMException is unavailable in this realm",
            Self::BackendUnavailable => "the JavaScript backend is unavailable",
            Self::CallbackPanicked => "a native callback panicked",
            Self::RegistryConflict => "binding root-registry access overlapped",
            Self::Exception(_) => "JavaScript threw an exception",
        })
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::HostFailure(failure) => Some(failure),
            _ => None,
        }
    }
}
