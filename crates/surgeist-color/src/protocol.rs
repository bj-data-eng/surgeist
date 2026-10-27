//! Exclusive numerical ownership, caller-owned digit records and action layouts.
use crate::format::bounded::{FormatDigit, FormatKernel};
use crate::{ColorSpace, Transformation};

/// Exclusive anchor for native numerical continuations and their caller-owned digits.
///
/// A computed value no longer borrows this owner:
/// ```
/// use surgeist_color::{Color, Computation, NumericalRequest, OperationOwner};
/// let mut owner = OperationOwner::new();
/// let value = Color::from_srgba8([0, 0, 0, 255]);
/// let request = NumericalRequest::WithAlpha { value, alpha: None };
/// let result = Computation::new(&mut owner, request).compute();
/// let second = Computation::new(&mut owner, request);
/// assert_eq!(result.alpha(), None);
/// assert_eq!(second.into_request(), request);
/// ```
/// A live continuation prevents a second mutable borrow of its actual anchor:
/// ```compile_fail,E0499
/// use surgeist_color::{Color, Computation, NumericalRequest, OperationOwner};
/// let mut owner = OperationOwner::new();
/// let request = NumericalRequest::WithAlpha { value: Color::from_srgba8([0; 4]), alpha: None };
/// let first = Computation::new(&mut owner, request);
/// let second = Computation::new(&mut owner, request);
/// let _ = first.compute();
/// drop(second);
/// ```
pub struct OperationOwner {
    marker: u8,
    phase: usize,
}
impl Default for OperationOwner {
    fn default() -> Self {
        Self::new()
    }
}
impl OperationOwner {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            marker: 1,
            phase: 0,
        }
    }
    #[must_use]
    pub const fn layout_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    /// Starts one operation phase.
    ///
    /// # Panics
    ///
    /// If this owner has already begun `usize::MAX` operations; no input
    /// reaches that count.
    pub(crate) fn begin(&mut self) -> usize {
        let _ = self.marker;
        self.phase = self
            .phase
            .checked_add(1)
            .expect("an operation owner begins at most usize::MAX operations");
        self.phase
    }
}

/// A checked numerical digit retaining the real numerical anchor borrow.
///
/// Only a write action mints a record. Its operation later reads it back by
/// index; a record of another operation or index is a defect in the driving
/// code and panics, as slice indexing does.
pub struct DigitSlot<'owner> {
    owner: &'owner OperationOwner,
    phase: usize,
    index: usize,
    digit: FormatDigit,
}
impl<'owner> DigitSlot<'owner> {
    pub(crate) fn new(
        owner: &'owner OperationOwner,
        phase: usize,
        index: usize,
        digit: FormatDigit,
    ) -> Self {
        Self {
            owner,
            phase,
            index,
            digit,
        }
    }
    /// The digit this record holds for `owner`'s operation `phase` at `index`.
    ///
    /// # Panics
    ///
    /// If the record belongs to another owner or operation, or was minted for
    /// another index.
    #[track_caller]
    pub(crate) fn digit_for(
        &self,
        owner: &OperationOwner,
        phase: usize,
        index: usize,
    ) -> FormatDigit {
        assert!(
            std::ptr::eq(self.owner, owner) && self.phase == phase,
            "digit record belongs to another numerical operation"
        );
        assert!(
            self.index == index,
            "digit record was minted for index {}, not the requested index {index}",
            self.index
        );
        self.digit
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kernel {
    Construct,
    Convert(ColorSpace),
    Prepare(ColorSpace),
    Interpolate,
    Transform(Transformation),
    WithAlpha,
    WithColorFunction,
    ToLegacySrgb,
    BlackWhiteContrast,
    SerializationPlan,
    SerializationPiece,
    PlanFinite,
    TrimMillionths,
    ExtractDigit,
    MultiplyDigit,
    NextMultiply,
    PlanOutput,
    PlanEmission,
}
impl From<FormatKernel> for Kernel {
    fn from(k: FormatKernel) -> Self {
        match k {
            FormatKernel::PlanFinite => Self::PlanFinite,
            FormatKernel::TrimMillionths => Self::TrimMillionths,
            FormatKernel::ExtractDigit => Self::ExtractDigit,
            FormatKernel::MultiplyDigit => Self::MultiplyDigit,
            FormatKernel::NextMultiply => Self::NextMultiply,
            FormatKernel::PlanOutput => Self::PlanOutput,
            FormatKernel::PlanEmission => Self::PlanEmission,
        }
    }
}

/// Payload observations and writes of one pending action. The caller admits
/// them, and its own storage and output events, before completing the action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionLayout {
    pub read_bytes: usize,
    pub write_bytes: usize,
    pub output_bytes: usize,
}
impl ActionLayout {
    /// A fixed-register computation observes no record and emits nothing.
    pub(crate) const COMPUTE: Self = Self {
        read_bytes: 0,
        write_bytes: 0,
        output_bytes: 0,
    };
    /// A digit read observes the record and its owner qualification.
    pub(crate) const READ_DIGIT: Self = Self {
        read_bytes: std::mem::size_of::<DigitSlot<'static>>()
            + std::mem::size_of::<OperationOwner>(),
        write_bytes: 0,
        output_bytes: 0,
    };
    /// A digit write mints one record and observes the stored acknowledgement.
    pub(crate) const WRITE_DIGIT: Self = Self {
        read_bytes: Self::READ_DIGIT.read_bytes,
        write_bytes: std::mem::size_of::<DigitSlot<'static>>(),
        output_bytes: 0,
    };
    /// An emission writes one output byte.
    pub(crate) const EMIT: Self = Self {
        read_bytes: 0,
        write_bytes: 0,
        output_bytes: 1,
    };
}
