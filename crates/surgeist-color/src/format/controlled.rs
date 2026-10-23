//! Controlled finite formatting over the shared numerical digit bank.
//!
//! The formatter is the same typestate as `Serialization`: `step` consumes it,
//! each issued action exposes only its own completion, and dropping either
//! abandons the formatting.
use super::RoundingMode;
use crate::controlled::{Pending, Stream};
use crate::format::bounded::FormatDigit;
use crate::{ActionLayout, DigitSlot, Finite, Kernel, OperationOwner};

/// A pending finite-bit formatting over an exclusive owner.
pub struct FiniteFormatter<'owner> {
    stream: Stream<'owner>,
}
impl<'owner> FiniteFormatter<'owner> {
    /// Begins one formatting; the owner stays borrowed until it ends and every
    /// record it minted is dropped.
    ///
    /// # Panics
    ///
    /// If `owner` has already begun `usize::MAX` operations.
    pub fn new(owner: &'owner mut OperationOwner, value: Finite, rounding: RoundingMode) -> Self {
        Self {
            stream: Stream::format(owner, value, rounding),
        }
    }
    /// Actual compiled continuation layout, admitted before construction.
    #[must_use]
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    /// Actual compiled layout of one issued action, which owns the continuation.
    #[must_use]
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<FormatAction<'static>>()
    }
    /// The exact output length, known after planning and before any emission.
    #[must_use]
    pub fn planned_length(&self) -> Option<usize> {
        self.stream.planned_length()
    }
    /// Consumes the formatter and yields its next action or its summary.
    #[must_use]
    pub fn step(self) -> FormatStep<'owner> {
        let stream = self.stream;
        FormatStep::Pending(match stream.pending() {
            Pending::Compute(kernel) => FormatAction::Compute(FormatCompute { stream, kernel }),
            Pending::ReadDigit { index } => {
                FormatAction::ReadDigit(FormatReadDigit { stream, index })
            }
            Pending::AppendDigit { index, digit } => FormatAction::AppendDigit(FormatWriteDigit {
                stream,
                index,
                digit,
            }),
            Pending::ReplaceDigit { index, digit } => {
                FormatAction::ReplaceDigit(FormatWriteDigit {
                    stream,
                    index,
                    digit,
                })
            }
            Pending::Emit { unit } => FormatAction::Emit(FormatEmit { stream, unit }),
            Pending::Complete => {
                let summary = stream.summary();
                return FormatStep::Complete(FormatSummary {
                    is_zero: summary.is_zero,
                    emitted: summary.emitted,
                });
            }
        })
    }
}
impl std::fmt::Debug for FiniteFormatter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FiniteFormatter")
            .field("stream", &self.stream)
            .finish()
    }
}
/// The next action of a pending formatting, or its completed summary.
#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "A pending action owns the inline formatting continuation, whose register layout callers admit; boxing it would add an unadmitted heap allocation per step"
)]
pub enum FormatStep<'owner> {
    Pending(FormatAction<'owner>),
    Complete(FormatSummary),
}
/// Exactly one pending action; it owns the formatter until completion.
#[derive(Debug)]
pub enum FormatAction<'owner> {
    Compute(FormatCompute<'owner>),
    ReadDigit(FormatReadDigit<'owner>),
    AppendDigit(FormatWriteDigit<'owner>),
    ReplaceDigit(FormatWriteDigit<'owner>),
    Emit(FormatEmit<'owner>),
}
impl<'owner> FormatAction<'owner> {
    /// Payload observations and writes to admit before completing the action.
    #[must_use]
    pub const fn layout(&self) -> ActionLayout {
        match self {
            Self::Compute(_) => ActionLayout::COMPUTE,
            Self::ReadDigit(_) => ActionLayout::READ_DIGIT,
            Self::AppendDigit(_) | Self::ReplaceDigit(_) => ActionLayout::WRITE_DIGIT,
            Self::Emit(_) => ActionLayout::EMIT,
        }
    }
    /// Declines the action without effect; the formatter stays pending.
    #[must_use]
    pub fn into_formatter(self) -> FiniteFormatter<'owner> {
        let stream = match self {
            Self::Compute(FormatCompute { stream, .. })
            | Self::ReadDigit(FormatReadDigit { stream, .. })
            | Self::AppendDigit(FormatWriteDigit { stream, .. })
            | Self::ReplaceDigit(FormatWriteDigit { stream, .. })
            | Self::Emit(FormatEmit { stream, .. }) => stream,
        };
        FiniteFormatter { stream }
    }
}
/// One pending fixed-register formatting kernel.
pub struct FormatCompute<'owner> {
    stream: Stream<'owner>,
    kernel: Kernel,
}
impl<'owner> FormatCompute<'owner> {
    /// The fixed kernel [`compute`](Self::compute) executes.
    #[must_use]
    pub const fn kernel(&self) -> Kernel {
        self.kernel
    }
    /// Executes the kernel.
    #[must_use]
    pub fn compute(mut self) -> FiniteFormatter<'owner> {
        self.stream.compute();
        FiniteFormatter {
            stream: self.stream,
        }
    }
}
impl std::fmt::Debug for FormatCompute<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatCompute")
            .field("kernel", &self.kernel)
            .finish_non_exhaustive()
    }
}
/// One pending read of a caller-owned digit record this formatter minted.
pub struct FormatReadDigit<'owner> {
    stream: Stream<'owner>,
    index: usize,
}
impl<'owner> FormatReadDigit<'owner> {
    /// The caller-owned record index to read.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
    /// Reads the caller's record stored at [`index`](Self::index).
    ///
    /// # Panics
    ///
    /// If `slot` was minted by another operation or for another index.
    #[must_use]
    #[track_caller]
    pub fn read(mut self, slot: &DigitSlot<'_>) -> FiniteFormatter<'owner> {
        self.stream.read(self.index, slot);
        FiniteFormatter {
            stream: self.stream,
        }
    }
}
impl std::fmt::Debug for FormatReadDigit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatReadDigit")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}
/// One pending append or replacement of a caller-owned digit record.
pub struct FormatWriteDigit<'owner> {
    stream: Stream<'owner>,
    index: usize,
    digit: FormatDigit,
}
impl<'owner> FormatWriteDigit<'owner> {
    /// The caller-owned record index to write.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
    /// Mints the record to store at [`index`](Self::index). Mint it only after
    /// the storage admission; the receipt acknowledges actual storage.
    #[must_use]
    pub fn record(self) -> (DigitSlot<'owner>, FormatDigitReceipt<'owner>) {
        (
            self.stream.record(self.index, self.digit),
            FormatDigitReceipt {
                stream: self.stream,
                index: self.index,
                digit: self.digit,
            },
        )
    }
}
impl std::fmt::Debug for FormatWriteDigit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatWriteDigit")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}
/// A minted digit record awaiting its storage acknowledgement.
pub struct FormatDigitReceipt<'owner> {
    stream: Stream<'owner>,
    index: usize,
    digit: FormatDigit,
}
impl<'owner> FormatDigitReceipt<'owner> {
    /// Acknowledges that `stored` now holds the minted record.
    ///
    /// The caller must pass the record it actually stored at this index. The
    /// check covers only the record's operation, index and digit, so a
    /// substituted record with the same identity is accepted, and a later
    /// read of a different stored record yields wrong output that the fixed
    /// registers cannot detect.
    ///
    /// # Panics
    ///
    /// If `stored` is not the record minted by the matching `record`.
    #[must_use]
    #[track_caller]
    pub fn stored(mut self, stored: &DigitSlot<'_>) -> FiniteFormatter<'owner> {
        self.stream.store(self.index, self.digit, stored);
        FiniteFormatter {
            stream: self.stream,
        }
    }
    /// Storage was refused: the minted record is discarded and the formatter
    /// waits at the same write.
    #[must_use]
    pub fn into_formatter(self) -> FiniteFormatter<'owner> {
        FiniteFormatter {
            stream: self.stream,
        }
    }
}
impl std::fmt::Debug for FormatDigitReceipt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatDigitReceipt")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}
/// One pending output byte.
pub struct FormatEmit<'owner> {
    stream: Stream<'owner>,
    unit: u8,
}
impl<'owner> FormatEmit<'owner> {
    /// The ASCII byte to append to the caller's output.
    #[must_use]
    pub const fn unit(&self) -> u8 {
        self.unit
    }
    /// Acknowledges that the caller appended [`unit`](Self::unit).
    #[must_use]
    pub fn emitted(mut self) -> FiniteFormatter<'owner> {
        self.stream.emit(self.unit);
        FiniteFormatter {
            stream: self.stream,
        }
    }
}
impl std::fmt::Debug for FormatEmit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatEmit")
            .field("unit", &self.unit)
            .finish_non_exhaustive()
    }
}
/// A completed formatting. It no longer borrows the numerical owner.
#[derive(Debug)]
pub struct FormatSummary {
    is_zero: bool,
    emitted: usize,
}
impl FormatSummary {
    /// Whether the formatted coefficient rounds to zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.is_zero
    }
    /// Count of acknowledged output bytes.
    #[must_use]
    pub const fn emitted(&self) -> usize {
        self.emitted
    }
}
