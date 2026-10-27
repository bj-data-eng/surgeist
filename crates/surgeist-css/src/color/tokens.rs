//! Finite qualified token/header and decoded-name observations for color grammar.

use super::keywords::{Word, word};
use crate::color::controlled::CssColorSlot;
use crate::color::controlled::protocol::{ColorProtocolError, LexicalRecord, LexicalTable};
use surgeist_css_parser::lexer::protocol::{RawToken, RawTokenKind, Span};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ClassifiedToken {
    pub(crate) index: usize,
    pub(crate) token: RawToken,
    pub(crate) word: Word,
    pub(crate) dashed: bool,
    pub(crate) unit: Option<crate::numeric::CssNumericUnit>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenAction {
    ReadRecord { index: usize },
    ReadName { index: usize },
    Classify,
    Yield,
    End,
    Stopped,
}

enum State {
    Records,
    Name,
    Yield,
    End,
    Stopped,
}

pub(crate) struct TokenReader<'owner> {
    table: LexicalTable<'owner>,
    next: usize,
    token: Option<RawToken>,
    token_index: usize,
    name: Span,
    name_index: usize,
    ascii: [u8; 24],
    ascii_length: usize,
    word_possible: bool,
    dashed: bool,
    word: Word,
    unit: Option<crate::numeric::CssNumericUnit>,
    state: State,
}
impl<'owner> TokenReader<'owner> {
    pub(crate) fn start(table: &LexicalTable<'owner>) -> Self {
        Self {
            table: *table,
            next: table.first_token_record(),
            token: None,
            token_index: 0,
            name: Span { start: 0, end: 0 },
            name_index: 0,
            ascii: [0; 24],
            ascii_length: 0,
            word_possible: true,
            dashed: false,
            word: Word::Unknown,
            unit: None,
            state: State::Records,
        }
    }
    /// Begins with a real lexical header read, rather than trusting a supplied
    /// classification or scanning for an index in another bank.
    pub(crate) fn start_at(
        table: &LexicalTable<'owner>,
        index: usize,
    ) -> Result<Self, ColorProtocolError> {
        if index < table.first_token_record() || index >= table.length() {
            return Err(ColorProtocolError::WrongIndex);
        }
        let mut reader = Self::start(table);
        reader.next = index;
        Ok(reader)
    }
    pub(crate) fn start_after(
        table: &LexicalTable<'owner>,
        token: ClassifiedToken,
        header: &CssColorSlot<'_>,
    ) -> Result<Self, ColorProtocolError> {
        let LexicalRecord::Token(actual) = table.observe(token.index, header)? else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if actual != token.token {
            return Err(ColorProtocolError::WrongSlot);
        }
        let next = token
            .index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        if next > table.length() {
            return Err(ColorProtocolError::WrongIndex);
        }
        let mut reader = Self::start(table);
        reader.next = next;
        Ok(reader)
    }
    pub(crate) fn action(&self) -> TokenAction {
        match self.state {
            State::Records if self.next == self.table.length() => TokenAction::End,
            State::Records => TokenAction::ReadRecord { index: self.next },
            State::Name if self.name_index == self.name.end => TokenAction::Classify,
            State::Name => TokenAction::ReadName {
                index: self.name_index,
            },
            State::Yield => TokenAction::Yield,
            State::End => TokenAction::End,
            State::Stopped => TokenAction::Stopped,
        }
    }
    pub(crate) fn read(
        &mut self,
        index: usize,
        record: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        match self.action() {
            TokenAction::ReadRecord { index: expected } if index == expected => {
                let observed = self.table.observe(index, record)?;
                let next = self
                    .next
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                if let LexicalRecord::Token(token) = observed {
                    let name = match token.kind {
                        RawTokenKind::Name { units, .. } => Some(units),
                        RawTokenKind::Number { unit, .. } => Some(unit),
                        RawTokenKind::Whitespace
                        | RawTokenKind::Comment { .. }
                        | RawTokenKind::String { .. }
                        | RawTokenKind::BadString
                        | RawTokenKind::Url { .. }
                        | RawTokenKind::BadUrl { .. }
                        | RawTokenKind::Delim(_)
                        | RawTokenKind::Cdo
                        | RawTokenKind::Cdc => None,
                    };
                    if name
                        .is_some_and(|name| name.end > self.table.length() || name.start > name.end)
                    {
                        return Err(ColorProtocolError::WrongIndex);
                    }
                    self.token = Some(token);
                    self.token_index = index;
                    self.word = Word::Unknown;
                    self.dashed = false;
                    self.unit = None;
                    if let Some(name) = name {
                        self.name = name;
                        self.name_index = name.start;
                        self.ascii_length = 0;
                        self.word_possible = true;
                        self.state = State::Name;
                    } else {
                        self.state = State::Yield;
                    }
                }
                self.next = next;
            }
            TokenAction::ReadName { index: expected } if index == expected => {
                let LexicalRecord::NameUnit(unit) = self.table.observe(index, record)? else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                let position = index - self.name.start;
                if position == 0 {
                    self.dashed = unit == '-';
                }
                if position == 1 {
                    self.dashed &= unit == '-';
                }
                if self.word_possible && unit.is_ascii() && self.ascii_length < self.ascii.len() {
                    self.ascii[self.ascii_length] = unit as u8;
                    self.ascii_length += 1;
                } else {
                    self.word_possible = false;
                }
                self.name_index += 1;
            }
            _ => return Err(ColorProtocolError::WrongAction),
        }
        Ok(())
    }
    pub(crate) fn classify(&mut self) -> Result<(), ColorProtocolError> {
        if self.action() != TokenAction::Classify {
            return Err(ColorProtocolError::WrongAction);
        }
        self.dashed &= self.name.end - self.name.start >= 2;
        // The fixed cache holds only ASCII, and its maximum24 is independent of
        // authored name size. Long/non-ASCII profile names retain their span.
        let name = self.word_possible.then(|| {
            std::str::from_utf8(&self.ascii[..self.ascii_length]).expect("admitted ASCII")
        });
        self.word = match (name, self.token.map(|token| token.kind)) {
            (Some(name), Some(RawTokenKind::Name { kind, .. })) => word(name, kind),
            _ => Word::Unknown,
        };
        self.unit = if matches!(
            self.token.map(|token| token.kind),
            Some(RawTokenKind::Number {
                kind: surgeist_css_parser::lexer::protocol::NumberKind::Dimension,
                ..
            })
        ) {
            name.and_then(crate::numeric::CssNumericUnit::parse)
        } else {
            None
        };
        self.state = State::Yield;
        Ok(())
    }
    pub(crate) fn current(&self) -> Result<ClassifiedToken, ColorProtocolError> {
        if !matches!(self.state, State::Yield) {
            return Err(ColorProtocolError::WrongAction);
        }
        Ok(ClassifiedToken {
            index: self.token_index,
            token: self.token.ok_or(ColorProtocolError::WrongSlot)?,
            word: self.word,
            dashed: self.dashed,
            unit: self.unit,
        })
    }
    /// Reuses the ordinary environment/name owner over the already admitted
    /// fixed ASCII cache. Long/non-ASCII identifiers can only be custom names.
    pub(crate) fn relative_channel(
        &self,
        environment: crate::color::CssRelativeColorEnvironment,
    ) -> Result<Option<crate::color::CssRelativeColorChannel>, ColorProtocolError> {
        if !matches!(self.state, State::Yield) {
            return Err(ColorProtocolError::WrongAction);
        }
        Ok(if self.word_possible {
            crate::color::syntax::recognition::numeric_relative_channel(
                environment,
                std::str::from_utf8(&self.ascii[..self.ascii_length]).expect("actual ASCII cache"),
            )
            .map(|(channel, _)| channel)
        } else {
            None
        })
    }
    pub(crate) fn consumed(&mut self) -> Result<(), ColorProtocolError> {
        if !matches!(self.state, State::Yield) {
            return Err(ColorProtocolError::WrongAction);
        }
        self.token = None;
        self.state = if self.next == self.table.length() {
            State::End
        } else {
            State::Records
        };
        Ok(())
    }
    pub(crate) fn abandon(&mut self) {
        self.state = State::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::controlled::CssColorOperationOwner;
    use crate::color::controlled::protocol::{ActionDescription, LexicalOperation};
    use surgeist_css_parser::input::InputKind;
    use surgeist_css_parser::lexer::protocol::NameKind;
    #[test]
    fn escaped_keywords_and_long_profile_spans_use_actual_flat_records() {
        let input = "\\72 GB( --évery-long-profile-name ) CanvasText lightgoldenrodyellow";
        let mut owner = CssColorOperationOwner::new();
        let mut lexer = LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
        let mut slots = Vec::new();
        loop {
            let mut action = lexer.next_action();
            match action.description() {
                ActionDescription::ReadInput { offset } => action
                    .read_input(offset, u16::from(input.as_bytes()[offset]))
                    .unwrap(),
                ActionDescription::ReadSlot { index, .. } => {
                    action.read_slot(index, &slots[index]).unwrap()
                }
                ActionDescription::AppendSlot { index, .. } => {
                    slots.push(action.append_record().unwrap());
                    action.appended(index).unwrap();
                }
                ActionDescription::ComputeInput | ActionDescription::ComputeToken => {
                    action.compute().unwrap()
                }
                ActionDescription::Done => break,
                ActionDescription::Stopped => panic!("stopped"),
            }
        }
        let table = lexer.finish().unwrap();
        let mut reader = TokenReader::start(&table);
        let mut observed = Vec::new();
        loop {
            match reader.action() {
                TokenAction::ReadName { index } | TokenAction::ReadRecord { index } => {
                    reader.read(index, &slots[index]).unwrap()
                }
                TokenAction::Classify => reader.classify().unwrap(),
                TokenAction::Yield => {
                    let token = reader.current().unwrap();
                    if !matches!(token.token.kind, RawTokenKind::Whitespace) {
                        observed.push(token);
                    }
                    reader.consumed().unwrap();
                }
                TokenAction::End => break,
                TokenAction::Stopped => panic!("stop"),
            }
        }
        assert_eq!(
            observed[0].word,
            Word::Function(super::super::keywords::ColorFunction::Rgb)
        );
        assert!(matches!(
            observed[0].token.kind,
            RawTokenKind::Name {
                kind: NameKind::Function,
                ..
            }
        ));
        assert!(observed[1].dashed);
        assert_eq!(observed[1].word, Word::Unknown);
        assert_eq!(
            observed[3].word,
            Word::System(crate::color::CssSystemColor::CanvasText)
        );
        assert_eq!(observed[4].word, Word::Named([250, 250, 210]));
        reader.abandon();
        assert_eq!(reader.current(), Err(ColorProtocolError::WrongAction));
    }
}
