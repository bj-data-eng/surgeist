//! CSS guards over surgeist-color's consuming numerical continuations.
//!
//! The CSS transformation and authored serialization guards describe one
//! pending action and complete it through `&mut` access. These states hold the
//! actual Color continuation between those calls. A completion that does not
//! match the pending action is refused without effect; a digit record of
//! another operation or index panics in surgeist-color.

use super::protocol::CssColorProtocolError;
use super::{CssColorActionDescription, CssColorKernel};
use surgeist_color::{
    ActionLayout, Color, Computation, DigitReceipt, DigitSlot, Serialization, SerializationAction,
    SerializationReceipt, SerializationStep,
};

/// A transformation computation, its uncommitted result or its abandonment.
pub(super) enum NumericalComputation<'num> {
    Pending(Computation<'num>),
    Ready(Color),
    Abandoned,
}
impl<'num> NumericalComputation<'num> {
    pub(super) fn description(&self) -> CssColorActionDescription {
        match self {
            Self::Pending(computation) => {
                CssColorActionDescription::Compute(CssColorKernel::Numerical(computation.kernel()))
            }
            Self::Ready(_) => CssColorActionDescription::UsedReady,
            Self::Abandoned => CssColorActionDescription::Stopped,
        }
    }
    /// The Color layout of the pending computation; settled states observe nothing.
    pub(super) fn layout(&self) -> ActionLayout {
        match self {
            Self::Pending(computation) => computation.layout(),
            Self::Ready(_) | Self::Abandoned => ActionLayout {
                read_bytes: 0,
                write_bytes: 0,
                output_bytes: 0,
            },
        }
    }
    pub(super) fn compute(&mut self) -> Result<(), CssColorProtocolError> {
        match std::mem::replace(self, Self::Abandoned) {
            Self::Pending(computation) => {
                *self = Self::Ready(computation.compute());
                Ok(())
            }
            other => {
                *self = other;
                Err(CssColorProtocolError::WrongAction)
            }
        }
    }
    pub(super) fn abandon(&mut self) {
        *self = Self::Abandoned;
    }
    /// The computed value; an unexecuted or abandoned computation has none.
    pub(super) fn value(&self) -> Result<Color, CssColorProtocolError> {
        match self {
            Self::Ready(value) => Ok(*value),
            Self::Pending(_) | Self::Abandoned => Err(CssColorProtocolError::Terminal),
        }
    }
}

/// A serialization's pending action, a minted record awaiting its storage
/// acknowledgement, its receipt or its abandonment.
pub(super) enum NumericalSerialization<'num> {
    Pending(SerializationAction<'num>),
    Recorded {
        receipt: DigitReceipt<'num>,
        description: CssColorActionDescription,
        layout: ActionLayout,
    },
    Complete(SerializationReceipt),
    Abandoned,
}
impl<'num> NumericalSerialization<'num> {
    pub(super) fn new(serialization: Serialization<'num>) -> Self {
        match serialization.step() {
            SerializationStep::Pending(action) => Self::Pending(action),
            SerializationStep::Complete(receipt) => Self::Complete(receipt),
        }
    }
    pub(super) fn description(&self) -> CssColorActionDescription {
        match self {
            Self::Pending(action) => match action {
                SerializationAction::Compute(compute) => {
                    CssColorActionDescription::Compute(CssColorKernel::Numerical(compute.kernel()))
                }
                SerializationAction::ReadDigit(read) => {
                    CssColorActionDescription::ReadNumericalDigit {
                        index: read.index(),
                    }
                }
                SerializationAction::AppendDigit(write) => {
                    CssColorActionDescription::AppendNumericalDigit {
                        index: write.index(),
                    }
                }
                SerializationAction::ReplaceDigit(write) => {
                    CssColorActionDescription::UpdateNumericalDigit {
                        index: write.index(),
                    }
                }
                SerializationAction::Emit(emit) => {
                    CssColorActionDescription::EmitUnit { unit: emit.unit() }
                }
            },
            Self::Recorded { description, .. } => *description,
            Self::Complete(_) => CssColorActionDescription::Serialized,
            Self::Abandoned => CssColorActionDescription::Stopped,
        }
    }
    /// The Color layout of the pending action; settled states observe nothing.
    pub(super) fn layout(&self) -> ActionLayout {
        match self {
            Self::Pending(action) => action.layout(),
            Self::Recorded { layout, .. } => *layout,
            Self::Complete(_) | Self::Abandoned => ActionLayout {
                read_bytes: 0,
                write_bytes: 0,
                output_bytes: 0,
            },
        }
    }
    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Abandoned)
    }
    pub(super) fn compute(&mut self) -> Result<(), CssColorProtocolError> {
        match self.take() {
            Self::Pending(SerializationAction::Compute(compute)) => {
                *self = Self::new(compute.compute());
                Ok(())
            }
            other => {
                *self = other;
                Err(CssColorProtocolError::WrongAction)
            }
        }
    }
    /// # Panics
    ///
    /// If `slot` is a record of another operation or index.
    #[track_caller]
    pub(super) fn read(
        &mut self,
        index: usize,
        slot: &DigitSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        match self.take() {
            Self::Pending(SerializationAction::ReadDigit(read)) if read.index() == index => {
                *self = Self::new(read.read(slot));
                Ok(())
            }
            other => {
                *self = other;
                Err(CssColorProtocolError::WrongAction)
            }
        }
    }
    /// Mints the pending write's record. Minting again before the storage
    /// acknowledgement returns the same record for the same write.
    pub(super) fn record(&mut self) -> Result<DigitSlot<'num>, CssColorProtocolError> {
        let pending = match self.take() {
            Self::Recorded { receipt, .. } => Self::new(receipt.into_serialization()),
            other => other,
        };
        let description = pending.description();
        let layout = pending.layout();
        match pending {
            Self::Pending(
                SerializationAction::AppendDigit(write) | SerializationAction::ReplaceDigit(write),
            ) => {
                let (record, receipt) = write.record();
                *self = Self::Recorded {
                    receipt,
                    description,
                    layout,
                };
                Ok(record)
            }
            other => {
                *self = other;
                Err(CssColorProtocolError::WrongAction)
            }
        }
    }
    /// # Panics
    ///
    /// If `slot` is not the record minted for the pending write.
    #[track_caller]
    pub(super) fn written(
        &mut self,
        index: usize,
        slot: &DigitSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        match self.take() {
            Self::Recorded {
                receipt,
                description:
                    CssColorActionDescription::AppendNumericalDigit { index: pending }
                    | CssColorActionDescription::UpdateNumericalDigit { index: pending },
                ..
            } if pending == index => {
                *self = Self::new(receipt.stored(slot));
                Ok(())
            }
            other @ Self::Recorded { .. } => {
                *self = other;
                Err(CssColorProtocolError::WrongIndex)
            }
            other => {
                *self = other;
                Err(CssColorProtocolError::WrongAction)
            }
        }
    }
    pub(super) fn emitted(&mut self, unit: u8) -> Result<(), CssColorProtocolError> {
        match self.take() {
            Self::Pending(SerializationAction::Emit(emit)) if emit.unit() == unit => {
                *self = Self::new(emit.emitted());
                Ok(())
            }
            other => {
                *self = other;
                Err(CssColorProtocolError::WrongAction)
            }
        }
    }
    pub(super) fn abandon(&mut self) {
        *self = Self::Abandoned;
    }
    /// The acknowledged output length of a completed serialization.
    pub(super) fn length(&self) -> Result<usize, CssColorProtocolError> {
        match self {
            Self::Complete(receipt) => Ok(receipt.length()),
            _ => Err(CssColorProtocolError::Terminal),
        }
    }
}
