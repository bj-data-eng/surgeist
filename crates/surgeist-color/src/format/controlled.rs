//! Controlled finite formatting over the shared numerical digit bank.
use super::{FormatError, RoundingMode};
use crate::controlled::{Original, Stream};
use crate::{
    ActionDescription, ActionLayout, DigitSlot, Finite, OperationOwner, ProtocolError, StreamAction,
};

pub struct FiniteFormatter<'owner> {
    stream: Stream<'owner>,
}
#[derive(Debug)]
pub struct FormatSummary {
    is_zero: bool,
    emitted: usize,
}
impl FormatSummary {
    pub const fn is_zero(&self) -> bool {
        self.is_zero
    }
    pub const fn emitted(&self) -> usize {
        self.emitted
    }
}
#[derive(Debug)]
pub struct FormatRecovery {
    value: Finite,
    rounding: RoundingMode,
    length: usize,
}
impl FormatRecovery {
    pub const fn value(&self) -> Finite {
        self.value
    }
    pub const fn rounding(&self) -> RoundingMode {
        self.rounding
    }
    pub const fn length(&self) -> usize {
        self.length
    }
}
impl<'owner> FiniteFormatter<'owner> {
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<FiniteFormatAction<'_, '_>>()
    }
    pub fn new(
        owner: &'owner mut OperationOwner,
        value: Finite,
        rounding: RoundingMode,
    ) -> Result<Self, FormatError> {
        Ok(Self {
            stream: Stream::format(owner, value, rounding)?,
        })
    }
    pub fn action(&self) -> ActionDescription {
        self.stream.action()
    }
    pub fn next_action(&mut self) -> FiniteFormatAction<'_, 'owner> {
        FiniteFormatAction {
            inner: StreamAction {
                stream: &mut self.stream,
            },
        }
    }
    pub fn planned_length(&self) -> Option<usize> {
        self.stream.planned_length()
    }
    pub fn abandon(&mut self) {
        self.stream.abandon();
    }
    pub fn take(slot: &mut Option<Self>) -> Result<FormatSummary, FormatError> {
        let summary = slot
            .as_ref()
            .ok_or(FormatError::Protocol(ProtocolError::EmptySlot))?
            .stream
            .summary()?;
        let _ = slot.take().expect("shared preflight proved ready");
        Ok(FormatSummary {
            is_zero: summary.is_zero,
            emitted: summary.emitted,
        })
    }
    pub fn recover(slot: &mut Option<Self>) -> Result<FormatRecovery, ProtocolError> {
        let s = slot.take().ok_or(ProtocolError::EmptySlot)?.stream;
        let Original::Format(value, rounding) = s.original else {
            unreachable!("finite formatter owns finite original")
        };
        Ok(FormatRecovery {
            value,
            rounding,
            length: s.emitted,
        })
    }
}
pub struct FiniteFormatAction<'step, 'owner> {
    inner: StreamAction<'step, 'owner>,
}
impl<'owner> FiniteFormatAction<'_, 'owner> {
    pub fn description(&self) -> ActionDescription {
        self.inner.description()
    }
    pub fn layout(&self) -> ActionLayout {
        self.inner.layout()
    }
    pub fn compute(self) -> Result<(), FormatError> {
        self.inner.compute().map_err(FormatError::Protocol)
    }
    pub fn read_digit(self, index: usize, slot: &DigitSlot<'_>) -> Result<(), ProtocolError> {
        self.inner.read_digit(index, slot)
    }
    pub fn record(&mut self) -> Result<DigitSlot<'owner>, ProtocolError> {
        self.inner.record()
    }
    pub fn written(self, index: usize, slot: &DigitSlot<'_>) -> Result<(), ProtocolError> {
        self.inner.written(index, slot)
    }
    pub fn emitted(self, unit: u8) -> Result<(), ProtocolError> {
        self.inner.emitted(unit)
    }
}
