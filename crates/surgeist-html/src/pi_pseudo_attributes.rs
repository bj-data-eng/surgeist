//! Whole-string XML PI pseudo-attributes, shared by the two concrete owners.
//! XML-stylesheet 20101028 §3 and XML 1.0 Fifth Edition productions 2–5/66.
//! The cursor owns no source, allocation, controller or retained field.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Field {
    Name,
    Value,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Name {
    Current,
    Pair(usize),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    Read(usize),
    Step,
    Begin(Field),
    Emit {
        field: Field,
        units: [u16; 2],
        len: usize,
        start: usize,
        end: usize,
        reference: bool,
    },
    NameLength(usize),
    NameUnit(Name, usize),
    Commit,
    Discard,
    Pop,
    Ready(bool),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Reply {
    Unit(Option<u16>),
    Length(usize),
    Ack,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Integrity,
    Representation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Leading,
    FirstName,
    Name,
    BeforeEquals,
    Equals,
    BeforeValue,
    Quote,
    Value,
    RefFirst,
    RefNumber,
    RefDigits,
    RefNamed,
    Separated,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    action: Option<Action>,
    phase: Phase,
    len: usize,
    offset: usize,
    scalar: Option<(u32, usize)>,
    high: Option<u16>,
    quote: u32,
    pairs: usize,
    active: bool,
    name_len: usize,
    compare: usize,
    comparison_offset: usize,
    current_unit: u16,
    ref_start: usize,
    radix: u32,
    number: u32,
    digits: usize,
    named: [u8; 4],
    named_len: usize,
}
impl Cursor {
    pub(crate) fn new(len: usize) -> Self {
        Self {
            action: Some(Action::Read(0)),
            phase: Phase::Leading,
            len,
            offset: 0,
            scalar: None,
            high: None,
            quote: 0,
            pairs: 0,
            active: false,
            name_len: 0,
            compare: 0,
            comparison_offset: 0,
            current_unit: 0,
            ref_start: 0,
            radix: 10,
            number: 0,
            digits: 0,
            named: [0; 4],
            named_len: 0,
        }
    }
    pub(crate) fn action(&self) -> Option<Action> {
        self.action
    }
    pub(crate) fn reply(&mut self, reply: Reply) -> Result<(), Error> {
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }
    fn add(a: usize, b: usize) -> Result<usize, Error> {
        a.checked_add(b).ok_or(Error::Representation)
    }
    fn read(&mut self) {
        self.action = Some(if self.scalar.is_some() {
            Action::Step
        } else {
            Action::Read(self.offset)
        });
    }
    fn consume(&mut self) -> Result<(), Error> {
        let (_, width) = self.scalar.take().ok_or(Error::Integrity)?;
        self.offset = Self::add(self.offset, width)?;
        self.read();
        Ok(())
    }
    fn invalid(&mut self) {
        self.action = Some(if self.active {
            Action::Discard
        } else if self.pairs != 0 {
            Action::Pop
        } else {
            Action::Ready(false)
        });
    }
    fn emit(&mut self, field: Field, cp: u32, start: usize, end: usize, reference: bool) {
        let (units, len) = if cp <= 0xffff {
            ([cp as u16, 0], 1)
        } else {
            let n = cp - 0x10000;
            ([0xd800 | (n >> 10) as u16, 0xdc00 | (n & 0x3ff) as u16], 2)
        };
        self.action = Some(Action::Emit {
            field,
            units,
            len,
            start,
            end,
            reference,
        });
    }
    fn advance(&mut self, reply: Reply) -> Result<(), Error> {
        match self.action.ok_or(Error::Integrity)? {
            Action::Read(index) => {
                let Reply::Unit(unit) = reply else {
                    return Err(Error::Integrity);
                };
                if index > self.len || (index == self.len) != unit.is_none() {
                    return Err(Error::Integrity);
                }
                if let Some(high) = self.high.take() {
                    let cp = match unit {
                        Some(low) if (0xdc00..=0xdfff).contains(&low) => {
                            0x10000 + ((u32::from(high) - 0xd800) << 10) + u32::from(low) - 0xdc00
                        }
                        _ => u32::from(high),
                    };
                    self.scalar = Some((cp, if cp > 0xffff { 2 } else { 1 }));
                } else if let Some(unit) = unit {
                    if (0xd800..=0xdbff).contains(&unit) && index + 1 < self.len {
                        self.high = Some(unit);
                        self.action = Some(Action::Read(Self::add(index, 1)?));
                        return Ok(());
                    }
                    self.scalar = Some((u32::from(unit), 1));
                }
                self.action = Some(Action::Step);
            }
            Action::Step => {
                if reply != Reply::Ack {
                    return Err(Error::Integrity);
                }
                self.step()?;
            }
            Action::Begin(Field::Name) => {
                if reply != Reply::Ack {
                    return Err(Error::Integrity);
                }
                self.active = true;
                self.name_len = 0;
                self.phase = Phase::FirstName;
                self.read();
            }
            Action::Begin(Field::Value) => {
                if reply != Reply::Ack {
                    return Err(Error::Integrity);
                }
                self.phase = Phase::Quote;
                self.read();
            }
            Action::Emit {
                field,
                len,
                reference,
                ..
            } => {
                if reply != Reply::Ack {
                    return Err(Error::Integrity);
                }
                if field == Field::Name {
                    self.name_len = Self::add(self.name_len, len)?;
                    self.phase = Phase::Name;
                } else if reference {
                    self.phase = Phase::Value;
                }
                self.consume()?;
            }
            Action::NameLength(index) => {
                let Reply::Length(len) = reply else {
                    return Err(Error::Integrity);
                };
                if index != self.compare || index >= self.pairs {
                    return Err(Error::Integrity);
                }
                if len == self.name_len {
                    self.comparison_offset = 0;
                    self.action = Some(Action::NameUnit(Name::Current, 0));
                } else {
                    self.next_comparison()?;
                }
            }
            Action::NameUnit(name, index) => {
                let Reply::Unit(Some(unit)) = reply else {
                    return Err(Error::Integrity);
                };
                if index != self.comparison_offset || index >= self.name_len {
                    return Err(Error::Integrity);
                }
                match name {
                    Name::Current => {
                        self.current_unit = unit;
                        self.action = Some(Action::NameUnit(Name::Pair(self.compare), index));
                    }
                    Name::Pair(pair) if pair == self.compare => {
                        if unit != self.current_unit {
                            self.next_comparison()?;
                        } else {
                            self.comparison_offset = Self::add(index, 1)?;
                            if self.comparison_offset == self.name_len {
                                self.invalid();
                            } else {
                                self.action =
                                    Some(Action::NameUnit(Name::Current, self.comparison_offset));
                            }
                        }
                    }
                    _ => return Err(Error::Integrity),
                }
            }
            Action::Commit => {
                if reply != Reply::Ack {
                    return Err(Error::Integrity);
                }
                self.pairs = Self::add(self.pairs, 1)?;
                self.active = false;
                self.phase = Phase::Separated;
                self.read();
            }
            Action::Discard => {
                if reply != Reply::Ack {
                    return Err(Error::Integrity);
                }
                self.active = false;
                self.invalid();
            }
            Action::Pop => {
                if reply != Reply::Ack || self.pairs == 0 {
                    return Err(Error::Integrity);
                }
                self.pairs -= 1;
                self.invalid();
            }
            Action::Ready(_) => {
                if reply != Reply::Ack {
                    return Err(Error::Integrity);
                }
                self.action = None;
            }
        }
        Ok(())
    }
    fn next_comparison(&mut self) -> Result<(), Error> {
        self.compare = Self::add(self.compare, 1)?;
        self.action = Some(if self.compare < self.pairs {
            Action::NameLength(self.compare)
        } else {
            Action::Commit
        });
        Ok(())
    }
    fn step(&mut self) -> Result<(), Error> {
        let cp = self.scalar.map(|x| x.0);
        let width = self.scalar.map_or(0, |x| x.1);
        match self.phase {
            Phase::Leading => match cp {
                None => self.action = Some(Action::Ready(true)),
                Some(cp) if whitespace(cp) => self.consume()?,
                _ => self.action = Some(Action::Begin(Field::Name)),
            },
            Phase::FirstName => match cp {
                Some(cp) if name_start(cp) => self.emit(
                    Field::Name,
                    cp,
                    self.offset,
                    Self::add(self.offset, width)?,
                    false,
                ),
                _ => self.invalid(),
            },
            Phase::Name => match cp {
                Some(cp) if name_char(cp) => self.emit(
                    Field::Name,
                    cp,
                    self.offset,
                    Self::add(self.offset, width)?,
                    false,
                ),
                _ => {
                    self.phase = Phase::BeforeEquals;
                    self.read();
                }
            },
            Phase::BeforeEquals => match cp {
                Some(cp) if whitespace(cp) => self.consume()?,
                _ => {
                    self.phase = Phase::Equals;
                    self.read();
                }
            },
            Phase::Equals => {
                if cp == Some(61) {
                    self.phase = Phase::BeforeValue;
                    self.consume()?;
                } else {
                    self.invalid();
                }
            }
            Phase::BeforeValue => match cp {
                Some(cp) if whitespace(cp) => self.consume()?,
                _ => self.action = Some(Action::Begin(Field::Value)),
            },
            Phase::Quote => {
                if matches!(cp, Some(34 | 39)) {
                    self.quote = cp.ok_or(Error::Integrity)?;
                    self.phase = Phase::Value;
                    self.consume()?;
                } else {
                    self.invalid();
                }
            }
            Phase::Value => match cp {
                Some(cp) if cp == self.quote => {
                    self.consume()?;
                    self.compare = 0;
                    self.action = Some(if self.pairs == 0 {
                        Action::Commit
                    } else {
                        Action::NameLength(0)
                    });
                }
                Some(38) => {
                    self.ref_start = self.offset;
                    self.phase = Phase::RefFirst;
                    self.consume()?;
                }
                Some(cp) if cp != 60 && xml_char(cp) => self.emit(
                    Field::Value,
                    cp,
                    self.offset,
                    Self::add(self.offset, width)?,
                    false,
                ),
                _ => self.invalid(),
            },
            Phase::RefFirst => {
                if cp == Some(35) {
                    self.radix = 10;
                    self.number = 0;
                    self.digits = 0;
                    self.phase = Phase::RefNumber;
                    self.consume()?;
                } else {
                    self.named_len = 0;
                    self.phase = Phase::RefNamed;
                    self.read();
                }
            }
            Phase::RefNumber => {
                if cp == Some(120) {
                    self.radix = 16;
                    self.phase = Phase::RefDigits;
                    self.consume()?;
                } else {
                    self.phase = Phase::RefDigits;
                    self.read();
                }
            }
            Phase::RefDigits => match cp {
                Some(59) if self.digits != 0 && xml_char(self.number) => self.emit(
                    Field::Value,
                    self.number,
                    self.ref_start,
                    Self::add(self.offset, width)?,
                    true,
                ),
                Some(cp) => {
                    let digit = match cp {
                        48..=57 => Some(cp - 48),
                        65..=70 if self.radix == 16 => Some(cp - 65 + 10),
                        97..=102 if self.radix == 16 => Some(cp - 97 + 10),
                        _ => None,
                    };
                    if let Some(n) = digit.and_then(|d| {
                        self.number
                            .checked_mul(self.radix)
                            .and_then(|n| n.checked_add(d))
                    }) {
                        self.number = n;
                        self.digits = Self::add(self.digits, 1)?;
                        self.consume()?;
                    } else {
                        self.invalid();
                    }
                }
                None => self.invalid(),
            },
            Phase::RefNamed => match cp {
                Some(59) => {
                    let value = match &self.named[..self.named_len] {
                        b"amp" => Some(38),
                        b"lt" => Some(60),
                        b"gt" => Some(62),
                        b"quot" => Some(34),
                        b"apos" => Some(39),
                        _ => None,
                    };
                    if let Some(value) = value {
                        self.emit(
                            Field::Value,
                            value,
                            self.ref_start,
                            Self::add(self.offset, width)?,
                            true,
                        );
                    } else {
                        self.invalid();
                    }
                }
                Some(cp) if cp <= 127 && self.named_len < 4 => {
                    self.named[self.named_len] = cp as u8;
                    self.named_len += 1;
                    self.consume()?;
                }
                _ => self.invalid(),
            },
            Phase::Separated => match cp {
                None => self.action = Some(Action::Ready(true)),
                Some(cp) if whitespace(cp) => {
                    self.phase = Phase::Leading;
                    self.consume()?;
                }
                _ => self.invalid(),
            },
        }
        Ok(())
    }
}
fn xml_char(cp: u32) -> bool {
    matches!(cp,9|10|13|0x20..=0xd7ff|0xe000..=0xfffd|0x10000..=0x10ffff)
}
fn whitespace(cp: u32) -> bool {
    matches!(cp, 9 | 10 | 13 | 32)
}
fn name_start(cp: u32) -> bool {
    matches!(cp,0x3a|0x41..=0x5a|0x5f|0x61..=0x7a|0xc0..=0xd6|0xd8..=0xf6|0xf8..=0x2ff|0x370..=0x37d|0x37f..=0x1fff|0x200c..=0x200d|0x2070..=0x218f|0x2c00..=0x2fef|0x3001..=0xd7ff|0xf900..=0xfdcf|0xfdf0..=0xfffd|0x10000..=0xeffff)
}
fn name_char(cp: u32) -> bool {
    name_start(cp) || matches!(cp,0x2d|0x2e|0x30..=0x39|0xb7|0x300..=0x36f|0x203f..=0x2040)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_replies_rollback_and_invalidation_visit_every_concrete_action() {
        let source: Vec<u16> = "name='&#88;' x=\"a\" x='b'".encode_utf16().collect();
        let mut cursor = Cursor::new(source.len());
        let mut names: Vec<Vec<u16>> = vec![];
        let mut name = vec![];
        let mut value = vec![];
        let mut seen = [false; 12];
        while let Some(action) = cursor.action() {
            let before = format!("{cursor:?}");
            let invalid = if matches!(
                action,
                Action::Read(_) | Action::NameUnit(..) | Action::NameLength(_)
            ) {
                Reply::Ack
            } else {
                Reply::Length(0)
            };
            assert_eq!(cursor.reply(invalid), Err(Error::Integrity));
            assert_eq!(format!("{cursor:?}"), before);
            let reply = match action {
                Action::Read(i) => {
                    seen[0] = true;
                    Reply::Unit(source.get(i).copied())
                }
                Action::Step => {
                    seen[1] = true;
                    Reply::Ack
                }
                Action::Begin(field) => {
                    if field == Field::Name {
                        seen[2] = true;
                        name.clear()
                    } else {
                        seen[3] = true;
                        value.clear()
                    }
                    Reply::Ack
                }
                Action::Emit {
                    field,
                    units,
                    len,
                    reference,
                    ..
                } => {
                    if field == Field::Name {
                        seen[4] = true;
                        name.extend_from_slice(&units[..len]);
                    } else {
                        seen[5] = true;
                        value.extend_from_slice(&units[..len]);
                        if reference {
                            assert_eq!(&units[..len], [88]);
                        }
                    }
                    Reply::Ack
                }
                Action::NameLength(i) => {
                    seen[6] = true;
                    Reply::Length(names[i].len())
                }
                Action::NameUnit(which, i) => {
                    seen[7] = true;
                    Reply::Unit(Some(match which {
                        Name::Current => name[i],
                        Name::Pair(n) => names[n][i],
                    }))
                }
                Action::Commit => {
                    seen[8] = true;
                    names.push(name.clone());
                    Reply::Ack
                }
                Action::Discard => {
                    seen[9] = true;
                    name.clear();
                    value.clear();
                    Reply::Ack
                }
                Action::Pop => {
                    seen[10] = true;
                    names.pop().unwrap();
                    Reply::Ack
                }
                Action::Ready(valid) => {
                    seen[11] = true;
                    assert!(!valid);
                    assert!(names.is_empty());
                    Reply::Ack
                }
            };
            cursor.reply(reply).unwrap();
        }
        assert_eq!(seen, [true; 12]);
        assert_eq!(cursor.reply(Reply::Ack), Err(Error::Integrity));
    }
    #[test]
    fn impossible_original_unit_presence_is_integrity_and_transactional() {
        for (len, reply) in [(0, Reply::Unit(Some(65))), (1, Reply::Unit(None))] {
            let mut c = Cursor::new(len);
            let old = format!("{c:?}");
            assert_eq!(c.reply(reply), Err(Error::Integrity));
            assert_eq!(format!("{c:?}"), old);
        }
    }
    #[test]
    fn literal_d_invalid_map_has_the_exact_thirteen_closed_admitted_actions() {
        let expected = [
            Action::Read(0),
            Action::Step,
            Action::Begin(Field::Name),
            Action::Step,
            Action::Emit {
                field: Field::Name,
                units: [68, 0],
                len: 1,
                start: 0,
                end: 1,
                reference: false,
            },
            Action::Read(1),
            Action::Step,
            Action::Read(1),
            Action::Step,
            Action::Read(1),
            Action::Step,
            Action::Discard,
            Action::Ready(false),
        ];
        let mut c = Cursor::new(1);
        for action in expected {
            assert_eq!(c.action(), Some(action));
            c.reply(match action {
                Action::Read(0) => Reply::Unit(Some(68)),
                Action::Read(1) => Reply::Unit(None),
                _ => Reply::Ack,
            })
            .unwrap();
        }
        assert_eq!(c.action(), None);
    }
}
