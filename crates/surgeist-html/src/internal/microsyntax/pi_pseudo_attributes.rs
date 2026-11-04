//! Whole-string XML PI pseudo-attributes, shared by the two concrete owners.
//! XML-stylesheet 20101028 §3 and XML 1.0 Fifth Edition productions 2–5/66.
//! The cursor owns no source, allocation, controller or retained field.

use crate::internal::values::name_index::HtmlNameIndexSlots;

/// The caller-owned field affected by a cursor action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlPiPseudoAttributeField {
    /// The pseudo-attribute name being scanned.
    Name,
    /// The quoted pseudo-attribute value being scanned, including its references.
    Value,
}
/// A name in the caller's current field or committed pair bank.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlPiPseudoAttributeName {
    /// The name of the pair still being scanned, before it is committed.
    Current,
    /// The name of the committed pair at this zero-based index.
    Pair(usize),
}
/// The next concrete read, field-bank mutation, or grammar transition.
///
/// The caller admits the action before performing it. `Read` addresses the
/// original UTF-16 input; name reads address fields previously emitted by this
/// cursor. `Commit`, `Discard`, and `Pop` update that same caller-owned bank.
/// `Ready(false)` follows cleanup of all committed pairs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlPiPseudoAttributeAction {
    /// Read the original input's UTF-16 code unit at this index; reply with `HtmlPiPseudoAttributeReply::Unit`,
    /// `None` exactly when the index equals the input length.
    Read(usize),
    /// Run the grammar transition for the scalar value already read. The reply is `HtmlPiPseudoAttributeReply::Ack`
    /// and needs no caller read.
    Step,
    /// Start a fresh caller-owned field. The reply is `HtmlPiPseudoAttributeReply::Ack` once the bank is ready.
    Begin(HtmlPiPseudoAttributeField),
    /// Append one decoded scalar value to a field of the pair being scanned.
    Emit {
        /// The field that receives the scalar value.
        field: HtmlPiPseudoAttributeField,
        /// The scalar value as UTF-16 code units; only the first `len` are meaningful.
        units: [u16; 2],
        /// How many of `units` are meaningful: one for a BMP value, two for a surrogate pair.
        len: usize,
        /// UTF-16 offset in the original input where the source text for this value begins.
        start: usize,
        /// UTF-16 offset one past the last source code unit of this value.
        end: usize,
        /// Whether the value came from a character or predefined entity reference, so
        /// `start..end` spans the whole reference rather than one literal scalar.
        reference: bool,
    },
    /// The name of the pair being scanned is complete. Find the committed pair whose name
    /// equals it, using the caller's own index of committed names (see
    /// [`HtmlNameIndex`](crate::bounded::values::HtmlNameIndex) and
    /// [`HtmlPiPseudoAttributeCursor::name_index_slots`]), and reply with
    /// `HtmlPiPseudoAttributeReply::Pair`: `Some(index)` of the committed pair, or `None` when
    /// no committed pair has that name. The
    /// caller also records the name in its index for the pair that `Commit` then adds, which
    /// takes the next index. The cursor verifies a `Some` answer with `NameLength` and
    /// `NameUnit` against that one pair only: a candidate with another name is not a duplicate,
    /// and a candidate that is not a committed pair is an integrity failure. A `None` answer
    /// is trusted and never verified, so a false `None` hides a duplicate. The index is never
    /// consulted after `Pop` or `Discard`, because those occur only on the invalid path, so it
    /// needs no removal. The requests of a scan grow with the pair count and the name units,
    /// not with their product. The hash key is random per process, so charge totals for a PI
    /// with several pairs can differ between processes, though never beyond the table bound the
    /// caller admitted; within a process the same input charges the same work.
    FindName,
    /// Report the length, in UTF-16 code units, of the name of the committed pair at this index,
    /// to compare it with the name of the pair being scanned for a duplicate. Reply with
    /// `HtmlPiPseudoAttributeReply::Length`.
    NameLength(usize),
    /// Report one UTF-16 code unit at this index of the named field, for the same duplicate
    /// comparison. Reply with `HtmlPiPseudoAttributeReply::Unit(Some(unit))`.
    NameUnit(HtmlPiPseudoAttributeName, usize),
    /// The pair is complete and its name is not a duplicate: record it as a committed pair.
    Commit,
    /// Drop the unfinished pair after an error; the scan is invalid.
    Discard,
    /// Remove the most recently committed pair; repeated until none remain after an error.
    Pop,
    /// The scan finished: `true` if the whole input is valid pseudo-attributes, `false` if it
    /// was invalid and all pairs were cleaned up.
    Ready(bool),
}
/// The actual read result, or acknowledgement after performing a pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlPiPseudoAttributeReply {
    /// Answer to `Read` or `NameUnit`: the requested code unit, or `None` at the input's end.
    Unit(Option<u16>),
    /// Answer to `NameLength`: the requested name's length in UTF-16 code units.
    Length(usize),
    /// Answer to `FindName`: the index of the committed pair whose name equals the current name,
    /// or `None`.
    Pair(Option<usize>),
    /// The pending write or transition was performed; carries no data.
    Ack,
}
/// A refused protocol transition; the cursor retains its previous state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlPiPseudoAttributeError {
    /// A reply does not match the pending action or original input extent.
    Integrity,
    /// A cursor offset, field length, or pair count cannot be represented.
    Representation,
}
impl std::fmt::Display for HtmlPiPseudoAttributeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Integrity => "HTML PI pseudo-attribute reply does not match its pending action",
            Self::Representation => "HTML PI pseudo-attribute cursor extent cannot be represented",
        })
    }
}
impl std::error::Error for HtmlPiPseudoAttributeError {}
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
/// Allocation-free grammar state; source and emitted fields stay with the caller.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct HtmlPiPseudoAttributeCursor {
    action: Option<HtmlPiPseudoAttributeAction>,
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
impl HtmlPiPseudoAttributeCursor {
    /// Starts a whole-string scan over the actual input's UTF-16 length.
    pub fn new(len: usize) -> Self {
        Self {
            action: Some(HtmlPiPseudoAttributeAction::Read(0)),
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
    /// The slot count of the index of committed names for pseudo-attribute data of `data_len`
    /// code units, at most half full. A pair needs at least four code units (a name, `=` and two
    /// quotes) and consecutive pairs are separated by at least one more, so the data holds at
    /// most `data_len / 4 + 1` pairs.
    pub fn name_index_slots(data_len: usize) -> HtmlNameIndexSlots {
        HtmlNameIndexSlots::for_entries(data_len / 4 + 1)
    }
    /// Observes the pending action without advancing it; `None` is terminal.
    pub fn action(&self) -> Option<HtmlPiPseudoAttributeAction> {
        self.action
    }
    /// Applies an actual action result, preserving the cursor on any error.
    pub fn reply(
        &mut self,
        reply: HtmlPiPseudoAttributeReply,
    ) -> Result<(), HtmlPiPseudoAttributeError> {
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }
    fn add(a: usize, b: usize) -> Result<usize, HtmlPiPseudoAttributeError> {
        a.checked_add(b)
            .ok_or(HtmlPiPseudoAttributeError::Representation)
    }
    fn read(&mut self) {
        self.action = Some(if self.scalar.is_some() {
            HtmlPiPseudoAttributeAction::Step
        } else {
            HtmlPiPseudoAttributeAction::Read(self.offset)
        });
    }
    fn consume(&mut self) -> Result<(), HtmlPiPseudoAttributeError> {
        let (_, width) = self
            .scalar
            .take()
            .ok_or(HtmlPiPseudoAttributeError::Integrity)?;
        self.offset = Self::add(self.offset, width)?;
        self.read();
        Ok(())
    }
    fn invalid(&mut self) {
        self.action = Some(if self.active {
            HtmlPiPseudoAttributeAction::Discard
        } else if self.pairs != 0 {
            HtmlPiPseudoAttributeAction::Pop
        } else {
            HtmlPiPseudoAttributeAction::Ready(false)
        });
    }
    fn emit(
        &mut self,
        field: HtmlPiPseudoAttributeField,
        cp: u32,
        start: usize,
        end: usize,
        reference: bool,
    ) {
        let (units, len) = if cp <= 0xffff {
            ([cp as u16, 0], 1)
        } else {
            let n = cp - 0x10000;
            ([0xd800 | (n >> 10) as u16, 0xdc00 | (n & 0x3ff) as u16], 2)
        };
        self.action = Some(HtmlPiPseudoAttributeAction::Emit {
            field,
            units,
            len,
            start,
            end,
            reference,
        });
    }
    fn advance(
        &mut self,
        reply: HtmlPiPseudoAttributeReply,
    ) -> Result<(), HtmlPiPseudoAttributeError> {
        match self.action.ok_or(HtmlPiPseudoAttributeError::Integrity)? {
            HtmlPiPseudoAttributeAction::Read(index) => {
                let HtmlPiPseudoAttributeReply::Unit(unit) = reply else {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                };
                if index > self.len || (index == self.len) != unit.is_none() {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
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
                        self.action = Some(HtmlPiPseudoAttributeAction::Read(Self::add(index, 1)?));
                        return Ok(());
                    }
                    self.scalar = Some((u32::from(unit), 1));
                }
                self.action = Some(HtmlPiPseudoAttributeAction::Step);
            }
            HtmlPiPseudoAttributeAction::Step => {
                if reply != HtmlPiPseudoAttributeReply::Ack {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                self.step()?;
            }
            HtmlPiPseudoAttributeAction::Begin(HtmlPiPseudoAttributeField::Name) => {
                if reply != HtmlPiPseudoAttributeReply::Ack {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                self.active = true;
                self.name_len = 0;
                self.phase = Phase::FirstName;
                self.read();
            }
            HtmlPiPseudoAttributeAction::Begin(HtmlPiPseudoAttributeField::Value) => {
                if reply != HtmlPiPseudoAttributeReply::Ack {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                self.phase = Phase::Quote;
                self.read();
            }
            HtmlPiPseudoAttributeAction::Emit {
                field,
                len,
                reference,
                ..
            } => {
                if reply != HtmlPiPseudoAttributeReply::Ack {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                if field == HtmlPiPseudoAttributeField::Name {
                    self.name_len = Self::add(self.name_len, len)?;
                    self.phase = Phase::Name;
                } else if reference {
                    self.phase = Phase::Value;
                }
                self.consume()?;
            }
            HtmlPiPseudoAttributeAction::FindName => {
                let HtmlPiPseudoAttributeReply::Pair(candidate) = reply else {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                };
                match candidate {
                    None => self.action = Some(HtmlPiPseudoAttributeAction::Commit),
                    Some(index) if index < self.pairs => {
                        self.compare = index;
                        self.action = Some(HtmlPiPseudoAttributeAction::NameLength(index));
                    }
                    Some(_) => return Err(HtmlPiPseudoAttributeError::Integrity),
                }
            }
            HtmlPiPseudoAttributeAction::NameLength(index) => {
                let HtmlPiPseudoAttributeReply::Length(len) = reply else {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                };
                if index != self.compare || index >= self.pairs {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                if len == self.name_len {
                    self.comparison_offset = 0;
                    self.action = Some(HtmlPiPseudoAttributeAction::NameUnit(
                        HtmlPiPseudoAttributeName::Current,
                        0,
                    ));
                } else {
                    // The named candidate is a different name: nothing is a duplicate.
                    self.action = Some(HtmlPiPseudoAttributeAction::Commit);
                }
            }
            HtmlPiPseudoAttributeAction::NameUnit(name, index) => {
                let HtmlPiPseudoAttributeReply::Unit(Some(unit)) = reply else {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                };
                if index != self.comparison_offset || index >= self.name_len {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                match name {
                    HtmlPiPseudoAttributeName::Current => {
                        self.current_unit = unit;
                        self.action = Some(HtmlPiPseudoAttributeAction::NameUnit(
                            HtmlPiPseudoAttributeName::Pair(self.compare),
                            index,
                        ));
                    }
                    HtmlPiPseudoAttributeName::Pair(pair) if pair == self.compare => {
                        if unit != self.current_unit {
                            self.action = Some(HtmlPiPseudoAttributeAction::Commit);
                        } else {
                            self.comparison_offset = Self::add(index, 1)?;
                            if self.comparison_offset == self.name_len {
                                self.invalid();
                            } else {
                                self.action = Some(HtmlPiPseudoAttributeAction::NameUnit(
                                    HtmlPiPseudoAttributeName::Current,
                                    self.comparison_offset,
                                ));
                            }
                        }
                    }
                    _ => return Err(HtmlPiPseudoAttributeError::Integrity),
                }
            }
            HtmlPiPseudoAttributeAction::Commit => {
                if reply != HtmlPiPseudoAttributeReply::Ack {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                self.pairs = Self::add(self.pairs, 1)?;
                self.active = false;
                self.phase = Phase::Separated;
                self.read();
            }
            HtmlPiPseudoAttributeAction::Discard => {
                if reply != HtmlPiPseudoAttributeReply::Ack {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                self.active = false;
                self.invalid();
            }
            HtmlPiPseudoAttributeAction::Pop => {
                if reply != HtmlPiPseudoAttributeReply::Ack || self.pairs == 0 {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                self.pairs -= 1;
                self.invalid();
            }
            HtmlPiPseudoAttributeAction::Ready(_) => {
                if reply != HtmlPiPseudoAttributeReply::Ack {
                    return Err(HtmlPiPseudoAttributeError::Integrity);
                }
                self.action = None;
            }
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), HtmlPiPseudoAttributeError> {
        let cp = self.scalar.map(|x| x.0);
        let width = self.scalar.map_or(0, |x| x.1);
        match self.phase {
            Phase::Leading => match cp {
                None => self.action = Some(HtmlPiPseudoAttributeAction::Ready(true)),
                Some(cp) if whitespace(cp) => self.consume()?,
                _ => {
                    self.action = Some(HtmlPiPseudoAttributeAction::Begin(
                        HtmlPiPseudoAttributeField::Name,
                    ))
                }
            },
            Phase::FirstName => match cp {
                Some(cp) if is_xml_name_start_code_point(cp) => self.emit(
                    HtmlPiPseudoAttributeField::Name,
                    cp,
                    self.offset,
                    Self::add(self.offset, width)?,
                    false,
                ),
                _ => self.invalid(),
            },
            Phase::Name => match cp {
                Some(cp) if is_xml_name_code_point(cp) => self.emit(
                    HtmlPiPseudoAttributeField::Name,
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
                _ => {
                    self.action = Some(HtmlPiPseudoAttributeAction::Begin(
                        HtmlPiPseudoAttributeField::Value,
                    ))
                }
            },
            Phase::Quote => {
                if matches!(cp, Some(34 | 39)) {
                    self.quote = cp.ok_or(HtmlPiPseudoAttributeError::Integrity)?;
                    self.phase = Phase::Value;
                    self.consume()?;
                } else {
                    self.invalid();
                }
            }
            Phase::Value => match cp {
                Some(cp) if cp == self.quote => {
                    self.consume()?;
                    self.action = Some(HtmlPiPseudoAttributeAction::FindName);
                }
                Some(38) => {
                    self.ref_start = self.offset;
                    self.phase = Phase::RefFirst;
                    self.consume()?;
                }
                Some(cp) if cp != 60 && xml_char(cp) => self.emit(
                    HtmlPiPseudoAttributeField::Value,
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
                    HtmlPiPseudoAttributeField::Value,
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
                            HtmlPiPseudoAttributeField::Value,
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
                None => self.action = Some(HtmlPiPseudoAttributeAction::Ready(true)),
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
/// XML 1.0 Fifth Edition's `NameStartChar` production.
///
/// The argument is a code point, including a possible isolated surrogate from
/// an exact UTF-16 string. Values outside the production return false.
pub fn is_xml_name_start_code_point(cp: u32) -> bool {
    matches!(cp,0x3a|0x41..=0x5a|0x5f|0x61..=0x7a|0xc0..=0xd6|0xd8..=0xf6|0xf8..=0x2ff|0x370..=0x37d|0x37f..=0x1fff|0x200c..=0x200d|0x2070..=0x218f|0x2c00..=0x2fef|0x3001..=0xd7ff|0xf900..=0xfdcf|0xfdf0..=0xfffd|0x10000..=0xeffff)
}
/// XML 1.0 Fifth Edition's `NameChar` production, including continuation-only
/// digits, punctuation and combining characters.
pub fn is_xml_name_code_point(cp: u32) -> bool {
    is_xml_name_start_code_point(cp)
        || matches!(cp,0x2d|0x2e|0x30..=0x39|0xb7|0x300..=0x36f|0x203f..=0x2040)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_replies_rollback_and_invalidation_visit_every_concrete_action() {
        let source: Vec<u16> = "name='&#88;' x=\"a\" x='b'".encode_utf16().collect();
        let mut cursor = HtmlPiPseudoAttributeCursor::new(source.len());
        let mut names: Vec<Vec<u16>> = vec![];
        let mut name = vec![];
        let mut value = vec![];
        let mut seen = [false; 13];
        while let Some(action) = cursor.action() {
            let before = cursor;
            let invalid = if matches!(
                action,
                HtmlPiPseudoAttributeAction::Read(_)
                    | HtmlPiPseudoAttributeAction::NameUnit(..)
                    | HtmlPiPseudoAttributeAction::NameLength(_)
            ) {
                HtmlPiPseudoAttributeReply::Ack
            } else {
                HtmlPiPseudoAttributeReply::Length(0)
            };
            assert_eq!(
                cursor.reply(invalid),
                Err(HtmlPiPseudoAttributeError::Integrity)
            );
            assert_eq!(cursor, before);
            let reply = match action {
                HtmlPiPseudoAttributeAction::Read(i) => {
                    seen[0] = true;
                    HtmlPiPseudoAttributeReply::Unit(source.get(i).copied())
                }
                HtmlPiPseudoAttributeAction::Step => {
                    seen[1] = true;
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::Begin(field) => {
                    if field == HtmlPiPseudoAttributeField::Name {
                        seen[2] = true;
                        name.clear()
                    } else {
                        seen[3] = true;
                        value.clear()
                    }
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::Emit {
                    field,
                    units,
                    len,
                    reference,
                    ..
                } => {
                    if field == HtmlPiPseudoAttributeField::Name {
                        seen[4] = true;
                        name.extend_from_slice(&units[..len]);
                    } else {
                        seen[5] = true;
                        value.extend_from_slice(&units[..len]);
                        if reference {
                            assert_eq!(&units[..len], [88]);
                        }
                    }
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::FindName => {
                    seen[12] = true;
                    HtmlPiPseudoAttributeReply::Pair(names.iter().position(|n| *n == name))
                }
                HtmlPiPseudoAttributeAction::NameLength(i) => {
                    seen[6] = true;
                    HtmlPiPseudoAttributeReply::Length(names[i].len())
                }
                HtmlPiPseudoAttributeAction::NameUnit(which, i) => {
                    seen[7] = true;
                    HtmlPiPseudoAttributeReply::Unit(Some(match which {
                        HtmlPiPseudoAttributeName::Current => name[i],
                        HtmlPiPseudoAttributeName::Pair(n) => names[n][i],
                    }))
                }
                HtmlPiPseudoAttributeAction::Commit => {
                    seen[8] = true;
                    names.push(name.clone());
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::Discard => {
                    seen[9] = true;
                    name.clear();
                    value.clear();
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::Pop => {
                    seen[10] = true;
                    names.pop().unwrap();
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::Ready(valid) => {
                    seen[11] = true;
                    assert!(!valid);
                    assert!(names.is_empty());
                    HtmlPiPseudoAttributeReply::Ack
                }
            };
            cursor.reply(reply).unwrap();
        }
        assert_eq!(seen, [true; 13]);
        assert_eq!(
            cursor.reply(HtmlPiPseudoAttributeReply::Ack),
            Err(HtmlPiPseudoAttributeError::Integrity)
        );
    }
    #[test]
    fn impossible_original_unit_presence_is_integrity_and_transactional() {
        for (len, reply) in [
            (0, HtmlPiPseudoAttributeReply::Unit(Some(65))),
            (1, HtmlPiPseudoAttributeReply::Unit(None)),
        ] {
            let mut c = HtmlPiPseudoAttributeCursor::new(len);
            let old = c;
            assert_eq!(c.reply(reply), Err(HtmlPiPseudoAttributeError::Integrity));
            assert_eq!(c, old);
        }
    }
    #[test]
    fn literal_d_invalid_map_has_the_exact_thirteen_closed_admitted_actions() {
        let expected = [
            HtmlPiPseudoAttributeAction::Read(0),
            HtmlPiPseudoAttributeAction::Step,
            HtmlPiPseudoAttributeAction::Begin(HtmlPiPseudoAttributeField::Name),
            HtmlPiPseudoAttributeAction::Step,
            HtmlPiPseudoAttributeAction::Emit {
                field: HtmlPiPseudoAttributeField::Name,
                units: [68, 0],
                len: 1,
                start: 0,
                end: 1,
                reference: false,
            },
            HtmlPiPseudoAttributeAction::Read(1),
            HtmlPiPseudoAttributeAction::Step,
            HtmlPiPseudoAttributeAction::Read(1),
            HtmlPiPseudoAttributeAction::Step,
            HtmlPiPseudoAttributeAction::Read(1),
            HtmlPiPseudoAttributeAction::Step,
            HtmlPiPseudoAttributeAction::Discard,
            HtmlPiPseudoAttributeAction::Ready(false),
        ];
        let mut c = HtmlPiPseudoAttributeCursor::new(1);
        for action in expected {
            assert_eq!(c.action(), Some(action));
            c.reply(match action {
                HtmlPiPseudoAttributeAction::Read(0) => HtmlPiPseudoAttributeReply::Unit(Some(68)),
                HtmlPiPseudoAttributeAction::Read(1) => HtmlPiPseudoAttributeReply::Unit(None),
                _ => HtmlPiPseudoAttributeReply::Ack,
            })
            .unwrap();
        }
        assert_eq!(c.action(), None);
    }
    /// Runs the grammar over `source`, answering `FindName` with `answer`, which receives the
    /// committed names and the current name. Returns the final validity and committed names.
    fn drive(
        source: &str,
        answer: impl Fn(&[Vec<u16>], &[u16]) -> Option<usize>,
    ) -> Result<(bool, Vec<Vec<u16>>), HtmlPiPseudoAttributeError> {
        let source: Vec<u16> = source.encode_utf16().collect();
        let mut cursor = HtmlPiPseudoAttributeCursor::new(source.len());
        let (mut names, mut name) = (Vec::<Vec<u16>>::new(), Vec::<u16>::new());
        let mut valid = false;
        while let Some(action) = cursor.action() {
            let reply = match action {
                HtmlPiPseudoAttributeAction::Read(i) => {
                    HtmlPiPseudoAttributeReply::Unit(source.get(i).copied())
                }
                HtmlPiPseudoAttributeAction::Begin(HtmlPiPseudoAttributeField::Name) => {
                    name.clear();
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::Emit {
                    field: HtmlPiPseudoAttributeField::Name,
                    units,
                    len,
                    ..
                } => {
                    name.extend_from_slice(&units[..len]);
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::FindName => {
                    HtmlPiPseudoAttributeReply::Pair(answer(&names, &name))
                }
                HtmlPiPseudoAttributeAction::NameLength(i) => {
                    HtmlPiPseudoAttributeReply::Length(names[i].len())
                }
                HtmlPiPseudoAttributeAction::NameUnit(which, i) => {
                    HtmlPiPseudoAttributeReply::Unit(Some(match which {
                        HtmlPiPseudoAttributeName::Current => name[i],
                        HtmlPiPseudoAttributeName::Pair(n) => names[n][i],
                    }))
                }
                HtmlPiPseudoAttributeAction::Commit => {
                    names.push(name.clone());
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::Pop => {
                    names.pop().unwrap();
                    HtmlPiPseudoAttributeReply::Ack
                }
                HtmlPiPseudoAttributeAction::Ready(ok) => {
                    valid = ok;
                    HtmlPiPseudoAttributeReply::Ack
                }
                _ => HtmlPiPseudoAttributeReply::Ack,
            };
            cursor.reply(reply)?;
        }
        Ok((valid, names))
    }
    fn honest(names: &[Vec<u16>], name: &[u16]) -> Option<usize> {
        names.iter().position(|n| n == name)
    }
    #[test]
    fn find_name_answers_decide_duplicates_and_a_candidate_is_verified_against_its_units() {
        let text = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
        // An exact duplicate is invalid and the map is emptied again.
        assert_eq!(drive("a='1' b='2' a='3'", honest), Ok((false, vec![])));
        // Names that differ only in case, in length or in a late unit are distinct.
        assert_eq!(
            drive("a='' A='' ab='' aB='' abc=''", honest).unwrap().1,
            vec![text("a"), text("A"), text("ab"), text("aB"), text("abc")]
        );
        // A wrong candidate, of another length or of the same length with other units, is
        // verified against the one pair it names and is not a duplicate.
        let wrong = |names: &[Vec<u16>], _: &[u16]| (!names.is_empty()).then_some(0);
        assert_eq!(
            drive("a='' bb='' cc='' dd=''", wrong).unwrap(),
            (true, vec![text("a"), text("bb"), text("cc"), text("dd")])
        );
        assert_eq!(
            drive("aa='' ab=''", wrong).unwrap(),
            (true, vec![text("aa"), text("ab")])
        );
        // A candidate that is not a committed pair is refused as an integrity failure.
        let missing = |names: &[Vec<u16>], _: &[u16]| Some(names.len());
        assert_eq!(
            drive("a=''", missing),
            Err(HtmlPiPseudoAttributeError::Integrity)
        );
    }
    #[test]
    fn name_index_slots_hold_every_pair_count_at_most_half_full() {
        // The densest string, short names with empty values, has at most len / 4 + 1 pairs.
        for pairs in 1..40_usize {
            let source: String = (0..pairs)
                .map(|n| format!("n{n}=''"))
                .collect::<Vec<_>>()
                .join(" ");
            let len = source.encode_utf16().count();
            let slots = HtmlPiPseudoAttributeCursor::name_index_slots(len);
            assert!(
                2 * pairs <= slots.get(),
                "{pairs} pairs in {} slots",
                slots.get()
            );
        }
    }
}
