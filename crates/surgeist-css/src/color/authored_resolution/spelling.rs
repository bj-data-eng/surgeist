//! Original typed number/name units into the canonical Algorithm records.
//!
//! Each byte observation and each record append is a separate admitted action.
//! Number syntax is the existing number owner's cursor. No typed value is
//! formatted, rounded, or routed through the source-color lexer.

use super::super::protocol::{
    AuthoredName, AuthoredNumber, CssColorBank, CssColorOperationOwner, CssColorProtocolError,
    CssColorSlot, Data, RecordRange,
};
use crate::tokenization::number::{Layout, NumberCursor, Part, Step};
use crate::{CssNumericTokenRef, CssNumericUnit};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Action {
    ReadByte { offset: usize },
    Compute,
    Append { index: usize },
    Done { record: usize },
}
#[derive(Clone, Copy)]
enum NumberState {
    Fill,
    Compute,
    Digit { exponent: bool, value: u8 },
    Header(Layout),
    Done { record: usize },
}
pub(super) struct NumberSpelling<'css> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    numeric: CssNumericTokenRef<'css>,
    representation: &'css str,
    unit: Option<CssNumericUnit>,
    percentage: bool,
    site: usize,
    cursor: NumberCursor,
    offset: usize,
    lookahead: [Option<char>; 3],
    filled: usize,
    start: usize,
    end: usize,
    state: NumberState,
}
impl<'css> NumberSpelling<'css> {
    /// The original numeric token header, including its representation length,
    /// is admitted by the invoking scalar/literal observation.
    pub(super) fn start(
        owner: &'css CssColorOperationOwner,
        phase: usize,
        numeric: CssNumericTokenRef<'css>,
        unit: Option<CssNumericUnit>,
        percentage: bool,
        site: usize,
        algorithm_end: usize,
    ) -> Self {
        Self {
            owner,
            phase,
            numeric,
            representation: numeric.representation(),
            unit,
            percentage,
            site,
            cursor: NumberCursor::new(),
            offset: 0,
            lookahead: [None; 3],
            filled: 0,
            start: algorithm_end,
            end: algorithm_end,
            state: NumberState::Fill,
        }
    }
    pub(super) fn action(&self) -> Action {
        match self.state {
            NumberState::Fill
                if self.filled < 3 && self.offset + self.filled < self.representation.len() =>
            {
                Action::ReadByte {
                    offset: self.offset + self.filled,
                }
            }
            NumberState::Fill | NumberState::Compute => Action::Compute,
            NumberState::Digit { .. } | NumberState::Header(_) => {
                Action::Append { index: self.end }
            }
            NumberState::Done { record } => Action::Done { record },
        }
    }
    pub(super) fn read_byte(&mut self) -> Result<(), CssColorProtocolError> {
        let Action::ReadByte { offset } = self.action() else {
            return Err(CssColorProtocolError::WrongAction);
        };
        let byte = *self
            .representation
            .as_bytes()
            .get(offset)
            .ok_or(CssColorProtocolError::WrongIndex)?;
        if !byte.is_ascii() {
            return Err(CssColorProtocolError::Input);
        }
        self.lookahead[self.filled] = Some(char::from(byte));
        self.filled += 1;
        Ok(())
    }
    pub(super) fn compute(&mut self) -> Result<(), CssColorProtocolError> {
        if self.action() != Action::Compute {
            return Err(CssColorProtocolError::WrongAction);
        }
        self.end
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        match self
            .cursor
            .step(self.lookahead)
            .map_err(|_| CssColorProtocolError::Overflow)?
        {
            Step::Transition => self.state = NumberState::Compute,
            Step::Consume(part) => {
                let next = self
                    .offset
                    .checked_add(1)
                    .ok_or(CssColorProtocolError::Overflow)?;
                if next > self.representation.len() || self.filled == 0 {
                    return Err(CssColorProtocolError::WrongIndex);
                }
                self.offset = next;
                self.lookahead = [self.lookahead[1], self.lookahead[2], None];
                self.filled -= 1;
                self.state = match part {
                    Part::Mantissa(value) => NumberState::Digit {
                        exponent: false,
                        value,
                    },
                    Part::ExponentDigit(value) => NumberState::Digit {
                        exponent: true,
                        value,
                    },
                    _ => NumberState::Fill,
                };
            }
            Step::Done(layout) => {
                if self.offset != self.representation.len() || layout.mantissa_digits == 0 {
                    return Err(CssColorProtocolError::Input);
                }
                self.state = NumberState::Header(layout);
            }
        }
        Ok(())
    }
    fn data(&self) -> Result<Data<'css>, CssColorProtocolError> {
        match self.state {
            NumberState::Digit { exponent, value } => Ok(Data::NumberDigit { exponent, value }),
            NumberState::Header(layout) => Ok(Data::AuthoredNumber(AuthoredNumber {
                numeric: self.numeric,
                layout,
                digits: RecordRange {
                    start: self.start,
                    end: self.end,
                },
                unit: self.unit,
                percentage: self.percentage,
                site: self.site,
            })),
            _ => Err(CssColorProtocolError::WrongAction),
        }
    }
    pub(super) fn record(&self, index: usize) -> Result<CssColorSlot<'css>, CssColorProtocolError> {
        if self.action() != (Action::Append { index }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        self.end
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        Ok(CssColorSlot::new(
            self.owner,
            self.phase,
            index,
            self.data()?,
        ))
    }
    pub(super) fn written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        if self.action() != (Action::Append { index }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        slot.qualify(self.owner, self.phase, index, CssColorBank::Algorithm)?;
        let same = match (self.data()?, slot.data) {
            (
                Data::NumberDigit {
                    exponent: a,
                    value: b,
                },
                Data::NumberDigit {
                    exponent: c,
                    value: d,
                },
            ) => a == c && b == d,
            (Data::AuthoredNumber(a), Data::AuthoredNumber(b)) => {
                a.layout == b.layout
                    && a.digits == b.digits
                    && a.unit == b.unit
                    && a.percentage == b.percentage
                    && a.site == b.site
                    && a.numeric.same_original(b.numeric)
            }
            _ => false,
        };
        if !same {
            return Err(CssColorProtocolError::WrongSlot);
        }
        self.end = self
            .end
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        self.state = match self.state {
            NumberState::Digit { .. } => NumberState::Fill,
            NumberState::Header(_) => NumberState::Done { record: index },
            _ => unreachable!("pending write was checked above"),
        };
        Ok(())
    }
    pub(super) const fn algorithm_end(&self) -> usize {
        self.end
    }
}

#[derive(Clone, Copy)]
enum NameState {
    Byte,
    Unit(char),
    Header,
    Done { record: usize },
}
pub(super) struct NameSpelling<'css> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    name: &'css str,
    site: usize,
    offset: usize,
    encoded: [u8; 4],
    filled: usize,
    start: usize,
    end: usize,
    state: NameState,
}
impl<'css> NameSpelling<'css> {
    pub(super) const fn units_start(&self) -> usize {
        self.start
    }

    pub(super) fn start(
        owner: &'css CssColorOperationOwner,
        phase: usize,
        name: &'css str,
        site: usize,
        algorithm_end: usize,
    ) -> Self {
        Self {
            owner,
            phase,
            name,
            site,
            offset: 0,
            encoded: [0; 4],
            filled: 0,
            start: algorithm_end,
            end: algorithm_end,
            state: NameState::Byte,
        }
    }
    pub(super) fn action(&self) -> Action {
        match self.state {
            NameState::Byte if self.offset < self.name.len() => Action::ReadByte {
                offset: self.offset,
            },
            NameState::Byte => Action::Compute,
            NameState::Unit(_) | NameState::Header => Action::Append { index: self.end },
            NameState::Done { record } => Action::Done { record },
        }
    }
    pub(super) fn read_byte(&mut self) -> Result<(), CssColorProtocolError> {
        let Action::ReadByte { offset } = self.action() else {
            return Err(CssColorProtocolError::WrongAction);
        };
        if self.filled == self.encoded.len() {
            return Err(CssColorProtocolError::Input);
        }
        let byte = *self
            .name
            .as_bytes()
            .get(offset)
            .ok_or(CssColorProtocolError::WrongIndex)?;
        let next = self
            .offset
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        self.encoded[self.filled] = byte;
        self.filled += 1;
        self.offset = next;
        match std::str::from_utf8(&self.encoded[..self.filled]) {
            Ok(text) => {
                let value = text.chars().next().ok_or(CssColorProtocolError::Input)?;
                self.state = NameState::Unit(value);
            }
            Err(error) if error.error_len().is_none() => {}
            Err(_) => return Err(CssColorProtocolError::Input),
        }
        Ok(())
    }
    pub(super) fn compute(&mut self) -> Result<(), CssColorProtocolError> {
        if self.action() != Action::Compute {
            return Err(CssColorProtocolError::WrongAction);
        }
        if self.filled != 0 {
            return Err(CssColorProtocolError::Input);
        }
        self.state = NameState::Header;
        Ok(())
    }
    fn data(&self) -> Result<Data<'css>, CssColorProtocolError> {
        match self.state {
            NameState::Unit(value) => Ok(Data::NameUnit(value)),
            NameState::Header => Ok(Data::AuthoredName(AuthoredName {
                name: self.name,
                units: RecordRange {
                    start: self.start,
                    end: self.end,
                },
                site: self.site,
            })),
            _ => Err(CssColorProtocolError::WrongAction),
        }
    }
    pub(super) fn record(&self, index: usize) -> Result<CssColorSlot<'css>, CssColorProtocolError> {
        if self.action() != (Action::Append { index }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        self.end
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        Ok(CssColorSlot::new(
            self.owner,
            self.phase,
            index,
            self.data()?,
        ))
    }
    pub(super) fn written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        if self.action() != (Action::Append { index }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        slot.qualify(self.owner, self.phase, index, CssColorBank::Algorithm)?;
        let same = match (self.data()?, slot.data) {
            (Data::NameUnit(a), Data::NameUnit(b)) => a == b,
            (Data::AuthoredName(a), Data::AuthoredName(b)) => {
                a.units == b.units && a.site == b.site && std::ptr::eq(a.name, b.name)
            }
            _ => false,
        };
        if !same {
            return Err(CssColorProtocolError::WrongSlot);
        }
        self.end = self
            .end
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        self.state = match self.state {
            NameState::Unit(_) => {
                self.filled = 0;
                NameState::Byte
            }
            NameState::Header => NameState::Done { record: index },
            _ => unreachable!("pending write was checked above"),
        };
        Ok(())
    }
    pub(super) const fn algorithm_end(&self) -> usize {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::literal::{LiteralAction, LiteralConversion};
    use crate::color::protocol::RecordTable;
    use crate::{CssComponentValue, CssComponentValueRef, CssValueTokenRef};

    #[test]
    fn original_authored_numbers_reach_exact_conversion_without_color_tokens() {
        for (spelling, unit, expected) in [
            // Authored zero is unsigned; a nonzero value that underflows keeps its sign.
            ("-0", None, 0.0_f64),
            ("-1e-9999", None, -0.0),
            (".5", None, 0.5),
            ("9007199254740993", None, 9007199254740992.0),
            ("1e-9999", None, 0.0),
            ("1e9999", None, f64::INFINITY),
            (
                "1.9e308",
                Some(CssNumericUnit::Angle(crate::CssAngleUnit::Gradians)),
                1.71e308,
            ),
        ] {
            let component = CssComponentValue::try_number(spelling).unwrap();
            let CssComponentValueRef::Token(CssValueTokenRef::Number(numeric)) = component.view()
            else {
                panic!("number");
            };
            let owner = CssColorOperationOwner::new();
            let phase = owner.current_phase();
            let next = owner.prepare_advance(phase).unwrap();
            owner.commit_phase(phase, next);
            let mut slots = Vec::new();
            let mut writer = NumberSpelling::start(&owner, next, numeric, unit, false, 17, 0);
            let header = loop {
                match writer.action() {
                    Action::ReadByte { .. } => writer.read_byte().unwrap(),
                    Action::Compute => writer.compute().unwrap(),
                    Action::Append { index } => {
                        assert_eq!(index, slots.len());
                        slots.push(writer.record(index).unwrap());
                        writer.written(index, &slots[index]).unwrap();
                    }
                    Action::Done { record } => break record,
                }
            };
            assert_eq!(writer.algorithm_end(), slots.len());
            assert!(
                slots
                    .iter()
                    .all(|slot| !matches!(slot.data, Data::Token(_)))
            );
            let Data::AuthoredNumber(number) = slots[header].data else {
                panic!("original header");
            };
            assert!(std::ptr::eq(
                number.numeric.representation(),
                numeric.representation()
            ));
            let table = RecordTable {
                owner: &owner,
                phase: next,
                length: slots.len(),
            };
            let mut conversion =
                LiteralConversion::start_authored(&table, header, &slots[header], slots.len())
                    .unwrap();
            let actual = loop {
                match conversion.action() {
                    LiteralAction::ReadSource { index }
                    | LiteralAction::ReadScaled { index }
                    | LiteralAction::ReadScratch { index }
                    | LiteralAction::ReadHeader { index } => {
                        conversion.read(index, &slots[index]).unwrap()
                    }
                    LiteralAction::Append { index } => {
                        assert_eq!(index, slots.len());
                        slots.push(conversion.record(index).unwrap());
                        conversion.written(index, &slots[index]).unwrap();
                    }
                    LiteralAction::Replace { index } => {
                        slots[index] = conversion.record(index).unwrap();
                        conversion.written(index, &slots[index]).unwrap();
                    }
                    LiteralAction::Compute => conversion.compute().unwrap(),
                    LiteralAction::Done(value) => break value,
                    LiteralAction::Stopped => panic!("unexpected stop"),
                }
            };
            assert_eq!(actual.value.to_bits(), expected.to_bits(), "{spelling}");
        }
    }
    #[test]
    fn original_decoded_unicode_name_is_read_once_into_real_name_units() {
        let name = crate::CssColorProfileName::try_new("--é🍕").unwrap();
        let owner = CssColorOperationOwner::new();
        let phase = owner.current_phase();
        let next = owner.prepare_advance(phase).unwrap();
        owner.commit_phase(phase, next);
        let mut writer = NameSpelling::start(&owner, next, name.as_str(), 4, 0);
        let mut slots = Vec::new();
        let mut bytes = 0;
        let header = loop {
            match writer.action() {
                Action::ReadByte { offset } => {
                    assert_eq!(offset, bytes);
                    bytes += 1;
                    writer.read_byte().unwrap();
                }
                Action::Compute => writer.compute().unwrap(),
                Action::Append { index } => {
                    slots.push(writer.record(index).unwrap());
                    writer.written(index, &slots[index]).unwrap();
                }
                Action::Done { record } => break record,
            }
        };
        assert_eq!(bytes, 8);
        assert_eq!(header, 4);
        assert_eq!(writer.algorithm_end(), 5);
        let actual: Vec<_> = slots[..header]
            .iter()
            .map(|slot| match slot.data {
                Data::NameUnit(value) => value,
                _ => panic!("name unit"),
            })
            .collect();
        assert_eq!(actual, ['-', '-', 'é', '🍕']);
        let Data::AuthoredName(original) = slots[header].data else {
            panic!("name header");
        };
        assert!(std::ptr::eq(original.name, name.as_str()));
    }
}
