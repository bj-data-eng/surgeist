//! Exclusive numerical ownership and finite action accounting.
use crate::format::bounded::{FormatDigit, FormatKernel, FormatProtocolError};
use crate::{ColorSpace, Transformation};

/// Exclusive anchor for native numerical continuations and their caller-owned digits.
///
/// A completed receipt no longer borrows this owner:
/// ```
/// use surgeist_color::{Color, Computation, NumericalRequest, OperationOwner};
/// let mut owner = OperationOwner::new();
/// let value = Color::from_srgba8([0, 0, 0, 255]);
/// let request = NumericalRequest::WithAlpha { value, alpha: None };
/// let mut first = Some(Computation::new(&mut owner, request).unwrap());
/// first.as_mut().unwrap().next_action().compute().unwrap();
/// let result = Computation::take(&mut first).unwrap();
/// let second = Computation::new(&mut owner, request).unwrap();
/// assert_eq!(result.alpha(), None);
/// assert_eq!(second.original(), request);
/// ```
/// A live continuation prevents a second mutable borrow of its actual anchor:
/// ```compile_fail,E0499
/// use surgeist_color::{Color, Computation, NumericalRequest, OperationOwner};
/// let mut owner = OperationOwner::new();
/// let request = NumericalRequest::WithAlpha { value: Color::from_srgba8([0; 4]), alpha: None };
/// let mut first = Computation::new(&mut owner, request).unwrap();
/// let second = Computation::new(&mut owner, request).unwrap();
/// first.next_action().compute().unwrap();
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
    pub const fn new() -> Self {
        Self {
            marker: 1,
            phase: 0,
        }
    }
    pub const fn layout_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    pub(crate) fn begin(&mut self) -> Result<usize, ProtocolError> {
        let _ = self.marker;
        let phase = self.phase.checked_add(1).ok_or(ProtocolError::Overflow)?;
        self.phase = phase;
        Ok(phase)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProtocolError {
    Overflow,
    ForeignSlot,
    WrongAction,
    WrongIndex,
    WrongDigit,
    WrongReply,
    NotReady,
    StorageNotEmpty,
    Terminated,
    EmptySlot,
}
impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "numerical color protocol: {self:?}")
    }
}
impl std::error::Error for ProtocolError {}
impl From<FormatProtocolError> for ProtocolError {
    fn from(e: FormatProtocolError) -> Self {
        match e {
            FormatProtocolError::WrongIndex => Self::WrongIndex,
            FormatProtocolError::InvalidDigit | FormatProtocolError::WrongDigit => Self::WrongDigit,
            FormatProtocolError::Terminated => Self::Terminated,
            FormatProtocolError::WrongReply | FormatProtocolError::NonFinite => Self::WrongReply,
        }
    }
}

/// A checked numerical digit retaining the real numerical anchor borrow.
/// Clearing a bank's length alone does not destroy its records or end their borrow:
/// ```compile_fail,E0499
/// use surgeist_color::{ActionDescription, Color, Computation, NumericalRequest, OperationOwner, Serialization, SerializationRequest};
/// let mut owner = OperationOwner::new();
/// let value = Color::from_srgba8([255, 0, 0, 255]);
/// let digit = {
///     let mut serializer = Serialization::new(&mut owner, value, SerializationRequest::Css).unwrap();
///     loop {
///         let mut action = serializer.next_action();
///         match action.description() {
///             ActionDescription::Compute(_) => action.compute().unwrap(),
///             ActionDescription::EmitByte { unit } => action.emitted(unit).unwrap(),
///             ActionDescription::AppendDigit { .. } => break action.record().unwrap(),
///             _ => panic!("expected a coefficient digit"),
///         }
///     }
/// };
/// let second = Computation::new(&mut owner, NumericalRequest::WithAlpha { value, alpha: None }).unwrap();
/// drop(digit);
/// drop(second);
/// ```
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
    pub(crate) fn qualify(
        &self,
        owner: &OperationOwner,
        phase: usize,
        index: usize,
    ) -> Result<FormatDigit, ProtocolError> {
        if !std::ptr::eq(self.owner, owner) || self.phase != phase || owner.phase != phase {
            return Err(ProtocolError::ForeignSlot);
        }
        if self.index != index {
            return Err(ProtocolError::WrongIndex);
        }
        Ok(self.digit)
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
    WithFamily,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionDescription {
    Compute(Kernel),
    ReadDigit { index: usize },
    AppendDigit { index: usize },
    ReplaceDigit { index: usize },
    EmitByte { unit: u8 },
    Ready,
    Failed,
    Abandoned,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionLayout {
    pub read_bytes: usize,
    pub write_bytes: usize,
    pub output_bytes: usize,
}
pub(crate) fn layout(action: ActionDescription) -> ActionLayout {
    use std::mem::size_of;
    ActionLayout {
        read_bytes: if matches!(
            action,
            ActionDescription::ReadDigit { .. }
                | ActionDescription::AppendDigit { .. }
                | ActionDescription::ReplaceDigit { .. }
        ) {
            size_of::<DigitSlot<'_>>() + size_of::<OperationOwner>()
        } else {
            0
        },
        write_bytes: if matches!(
            action,
            ActionDescription::AppendDigit { .. } | ActionDescription::ReplaceDigit { .. }
        ) {
            size_of::<DigitSlot<'_>>()
        } else {
            0
        },
        output_bytes: usize::from(matches!(action, ActionDescription::EmitByte { .. })),
    }
}
