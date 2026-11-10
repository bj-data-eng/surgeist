//! Exact reached profile-key matching over real decoded/fact units.

use crate::color::controlled::CssColorSlot;
use crate::color::controlled::protocol::{
    ColorProtocolError, LexicalRecord, RecordRange, RecordTable,
};
use crate::color::{CssColorFactIdentity, CssColorFacts, CssColorProfileAvailability};
use crate::rule::color_profile::CssColorProfileRenderingIntent;
#[cfg(test)]
use surgeist_css_parser::lexer::protocol::{NameKind, RawTokenKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProfileError {
    Protocol(ColorProtocolError),
    ForeignFacts,
    InvalidName,
    InvalidMetadata,
    Missing,
    Duplicate { first: usize, second: usize },
    Stopped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProfileAction {
    ReadPrefix { index: usize },
    ReadLength,
    ReadHeader { binding: usize },
    ReadSource { index: usize },
    ReadFactByte { binding: usize, index: usize },
    FinishCandidate,
    ReadSelectedHeader { binding: usize },
    Done,
    Stopped,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProfileSelection {
    pub(crate) binding: usize,
    pub(crate) facts: CssColorFactIdentity,
    pub(crate) resource: CssColorFactIdentity,
    pub(crate) availability: CssColorProfileAvailability,
    pub(crate) intent: CssColorProfileRenderingIntent,
    pub(crate) parameter_count: usize,
    pub(crate) named_count: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ComparisonAction {
    ReadSource { index: usize },
    ReadFactByte { index: usize },
    Finish,
}
/// The same exact decoded-name comparison is used for profile names and named
/// profile channels. Each action observes one source scalar or one fact byte.
struct NameComparison {
    source: usize,
    end: usize,
    fact_byte: usize,
    fact_length: usize,
    encoded: [u8; 4],
    encoded_length: usize,
    encoded_index: usize,
    fact_pending: bool,
    matches: bool,
}
impl NameComparison {
    const fn start(source: RecordRange, fact_length: usize) -> Self {
        Self {
            source: source.start,
            end: source.end,
            fact_byte: 0,
            fact_length,
            encoded: [0; 4],
            encoded_length: 0,
            encoded_index: 0,
            fact_pending: false,
            matches: true,
        }
    }
    const fn action(&self) -> ComparisonAction {
        if !self.matches || self.source == self.end && !self.fact_pending {
            ComparisonAction::Finish
        } else if self.fact_pending {
            ComparisonAction::ReadFactByte {
                index: self.fact_byte,
            }
        } else {
            ComparisonAction::ReadSource { index: self.source }
        }
    }
    fn source(&mut self, index: usize, value: char) -> Result<(), ColorProtocolError> {
        if self.action() != (ComparisonAction::ReadSource { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        let next = self
            .source
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        self.encoded_length = value.encode_utf8(&mut self.encoded).len();
        self.encoded_index = 0;
        self.source = next;
        if self.fact_byte == self.fact_length {
            self.matches = false;
        } else {
            self.fact_pending = true;
        }
        Ok(())
    }
    fn fact(&mut self, index: usize, value: u8) -> Result<(), ColorProtocolError> {
        if self.action() != (ComparisonAction::ReadFactByte { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        if value != self.encoded[self.encoded_index] {
            self.matches = false;
            self.fact_pending = false;
        } else {
            let next = self
                .fact_byte
                .checked_add(1)
                .ok_or(ColorProtocolError::Overflow)?;
            self.fact_byte = next;
            self.encoded_index += 1;
            if self.encoded_index == self.encoded_length {
                self.fact_pending = false;
            } else if self.fact_byte == self.fact_length {
                self.matches = false;
                self.fact_pending = false;
            }
        }
        Ok(())
    }
    const fn equal(&self) -> bool {
        self.matches
            && self.source == self.end
            && self.fact_byte == self.fact_length
            && !self.fact_pending
    }
}
enum State {
    Prefix,
    Length,
    Header,
    Source,
    FactByte,
    Finish,
    Selected,
    Done,
    Stopped,
}
pub(crate) struct ProfileLookup<'owner, 'facts> {
    table: RecordTable<'owner>,
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    name: RecordRange,
    prefix: usize,
    binding: usize,
    bindings: usize,
    comparison: Option<NameComparison>,
    selected: Option<usize>,
    result: Option<ProfileSelection>,
    state: State,
}
impl<'owner, 'facts> ProfileLookup<'owner, 'facts> {
    pub(crate) fn start(
        table: &RecordTable<'owner>,
        token_index: usize,
        token_slot: &CssColorSlot<'_>,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
    ) -> Result<Self, ProfileError> {
        let Some(name) = table
            .name_header(token_index, token_slot)
            .map_err(ProfileError::Protocol)?
        else {
            return Err(ProfileError::InvalidName);
        };
        if name.end > table.length || name.start.checked_add(2).is_none_or(|end| end > name.end) {
            return Err(ProfileError::InvalidName);
        }
        Ok(Self {
            table: *table,
            facts,
            expected,
            name,
            prefix: 0,
            binding: 0,
            bindings: 0,
            comparison: None,
            selected: None,
            result: None,
            state: State::Prefix,
        })
    }
    pub(crate) fn action(&self) -> ProfileAction {
        match self.state {
            State::Prefix => ProfileAction::ReadPrefix {
                index: self.name.start + self.prefix,
            },
            State::Length => ProfileAction::ReadLength,
            State::Header => ProfileAction::ReadHeader {
                binding: self.binding,
            },
            State::Source | State::FactByte => match self
                .comparison
                .as_ref()
                .expect("actual compared name")
                .action()
            {
                ComparisonAction::ReadSource { index } => ProfileAction::ReadSource { index },
                ComparisonAction::ReadFactByte { index } => ProfileAction::ReadFactByte {
                    binding: self.binding,
                    index,
                },
                ComparisonAction::Finish => ProfileAction::FinishCandidate,
            },
            State::Finish => ProfileAction::FinishCandidate,
            State::Selected => ProfileAction::ReadSelectedHeader {
                binding: self.selected.expect("selected actual index"),
            },
            State::Done => ProfileAction::Done,
            State::Stopped => ProfileAction::Stopped,
        }
    }
    fn qualification(&self) -> Result<(), ProfileError> {
        if self.facts.identity != self.expected {
            Err(ProfileError::ForeignFacts)
        } else {
            Ok(())
        }
    }
    pub(crate) fn read_source(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ProfileError> {
        self.qualification()?;
        if !matches!(self.action(), ProfileAction::ReadPrefix { index: expected } | ProfileAction::ReadSource { index: expected } if expected == index)
        {
            return Err(ProfileError::Protocol(ColorProtocolError::WrongAction));
        }
        let LexicalRecord::NameUnit(unit) = self
            .table
            .observe(index, slot)
            .map_err(ProfileError::Protocol)?
        else {
            return Err(ProfileError::Protocol(ColorProtocolError::WrongSlot));
        };
        if matches!(self.state, State::Prefix) {
            if unit != '-' {
                self.state = State::Stopped;
                return Err(ProfileError::InvalidName);
            }
            self.prefix += 1;
            if self.prefix == 2 {
                self.state = State::Length;
            }
        } else {
            self.comparison
                .as_mut()
                .ok_or(ProfileError::Protocol(ColorProtocolError::WrongAction))?
                .source(index, unit)
                .map_err(ProfileError::Protocol)?;
            self.state = State::FactByte;
        }
        Ok(())
    }
    /// This method observes one real borrowed fact header/unit after admission.
    /// It never clones a decoded name or invokes a whole-string equality scan.
    pub(crate) fn step(&mut self, action: ProfileAction) -> Result<(), ProfileError> {
        self.qualification()?;
        if action != self.action() {
            return Err(ProfileError::Protocol(ColorProtocolError::WrongAction));
        }
        match action {
            ProfileAction::ReadLength => {
                self.bindings = self.facts.profiles.len();
                self.state = if self.bindings == 0 {
                    State::Finish
                } else {
                    State::Header
                };
            }
            ProfileAction::ReadHeader { binding } => {
                let header = self
                    .facts
                    .profiles
                    .get(binding)
                    .ok_or(ProfileError::Protocol(ColorProtocolError::WrongIndex))?;
                self.comparison =
                    Some(NameComparison::start(self.name, header.name.as_str().len()));
                self.state = State::Source;
            }
            ProfileAction::ReadFactByte { binding, index } => {
                let header = self
                    .facts
                    .profiles
                    .get(binding)
                    .ok_or(ProfileError::Protocol(ColorProtocolError::WrongIndex))?;
                let byte = *header
                    .name
                    .as_str()
                    .as_bytes()
                    .get(index)
                    .ok_or(ProfileError::Protocol(ColorProtocolError::WrongIndex))?;
                self.comparison
                    .as_mut()
                    .ok_or(ProfileError::Protocol(ColorProtocolError::WrongAction))?
                    .fact(index, byte)
                    .map_err(ProfileError::Protocol)?;
                self.state = State::Source;
            }
            ProfileAction::FinishCandidate => {
                if self.binding < self.bindings
                    && self.comparison.as_ref().is_some_and(NameComparison::equal)
                {
                    if let Some(first) = self.selected {
                        self.state = State::Stopped;
                        return Err(ProfileError::Duplicate {
                            first,
                            second: self.binding,
                        });
                    }
                    self.selected = Some(self.binding);
                }
                if self.binding < self.bindings {
                    self.binding += 1;
                }
                if self.binding < self.bindings {
                    self.state = State::Header;
                } else if self.selected.is_some() {
                    self.state = State::Selected;
                } else {
                    self.state = State::Stopped;
                    return Err(ProfileError::Missing);
                }
            }
            ProfileAction::ReadSelectedHeader { binding } => {
                let header = self
                    .facts
                    .profiles
                    .get(binding)
                    .ok_or(ProfileError::Protocol(ColorProtocolError::WrongIndex))?;
                if header.availability == CssColorProfileAvailability::Loaded
                    && header.parameter_count == 0
                {
                    self.state = State::Stopped;
                    return Err(ProfileError::InvalidMetadata);
                }
                self.result = Some(ProfileSelection {
                    binding,
                    facts: self.expected,
                    resource: header.resource,
                    availability: header.availability,
                    intent: header.intent,
                    parameter_count: header.parameter_count,
                    named_count: header.components.len(),
                });
                self.state = State::Done;
            }
            _ => return Err(ProfileError::Protocol(ColorProtocolError::WrongAction)),
        }
        Ok(())
    }
    pub(crate) fn result(&self) -> Result<ProfileSelection, ProfileError> {
        if !matches!(self.state, State::Done) {
            return Err(ProfileError::Stopped);
        }
        self.result.ok_or(ProfileError::Stopped)
    }
    pub(crate) fn abandon(&mut self) {
        self.state = State::Stopped;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentNameAction {
    ReadProfileHeader,
    ReadNameHeader { ordinal: usize },
    ReadSource { index: usize },
    ReadNameByte { ordinal: usize, index: usize },
    FinishCandidate,
    Done,
    Stopped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentNameError {
    Protocol(ColorProtocolError),
    ForeignFacts,
    InvalidMetadata,
    Unavailable,
    InvalidProfile,
    UnknownChannel,
    Stopped,
}
enum ComponentState {
    Header,
    Name,
    Compare,
    Finish,
    Done,
    Stopped,
}
/// Exact first-name/actual-ordinal binding. Duplicate descriptor names do not
/// shift coordinates, and unnamed actual channels do not gain invented names.
pub(crate) struct ProfileComponentLookup<'owner, 'facts> {
    table: RecordTable<'owner>,
    facts: &'facts CssColorFacts<'facts>,
    selection: ProfileSelection,
    name: RecordRange,
    ordinal: usize,
    comparison: Option<NameComparison>,
    state: ComponentState,
}
impl<'owner, 'facts> ProfileComponentLookup<'owner, 'facts> {
    pub(crate) fn start(
        table: &RecordTable<'owner>,
        token: usize,
        slot: &CssColorSlot<'_>,
        facts: &'facts CssColorFacts<'facts>,
        selection: ProfileSelection,
    ) -> Result<Self, ComponentNameError> {
        let Some(name) = table
            .name_header(token, slot)
            .map_err(ComponentNameError::Protocol)?
        else {
            return Err(ComponentNameError::UnknownChannel);
        };
        if name.start > name.end || name.end > table.length {
            return Err(ComponentNameError::InvalidMetadata);
        }
        Ok(Self {
            table: *table,
            facts,
            selection,
            name,
            ordinal: 0,
            comparison: None,
            state: ComponentState::Header,
        })
    }
    pub(crate) fn action(&self) -> ComponentNameAction {
        match self.state {
            ComponentState::Header => ComponentNameAction::ReadProfileHeader,
            ComponentState::Name => ComponentNameAction::ReadNameHeader {
                ordinal: self.ordinal,
            },
            ComponentState::Compare => match self
                .comparison
                .as_ref()
                .expect("actual channel-name comparison")
                .action()
            {
                ComparisonAction::ReadSource { index } => ComponentNameAction::ReadSource { index },
                ComparisonAction::ReadFactByte { index } => ComponentNameAction::ReadNameByte {
                    ordinal: self.ordinal,
                    index,
                },
                ComparisonAction::Finish => ComponentNameAction::FinishCandidate,
            },
            ComponentState::Finish => ComponentNameAction::FinishCandidate,
            ComponentState::Done => ComponentNameAction::Done,
            ComponentState::Stopped => ComponentNameAction::Stopped,
        }
    }
    fn qualification(&self) -> Result<(), ComponentNameError> {
        if self.facts.identity != self.selection.facts {
            Err(ComponentNameError::ForeignFacts)
        } else {
            Ok(())
        }
    }
    fn fail(&mut self, error: ComponentNameError) -> Result<(), ComponentNameError> {
        self.state = ComponentState::Stopped;
        Err(error)
    }
    pub(crate) fn read_source(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ComponentNameError> {
        self.qualification()?;
        if self.action() != (ComponentNameAction::ReadSource { index }) {
            return Err(ComponentNameError::Protocol(
                ColorProtocolError::WrongAction,
            ));
        }
        let LexicalRecord::NameUnit(value) = self
            .table
            .observe(index, slot)
            .map_err(ComponentNameError::Protocol)?
        else {
            return Err(ComponentNameError::Protocol(ColorProtocolError::WrongSlot));
        };
        self.comparison
            .as_mut()
            .ok_or(ComponentNameError::Protocol(
                ColorProtocolError::WrongAction,
            ))?
            .source(index, value)
            .map_err(ComponentNameError::Protocol)
    }
    pub(crate) fn step(&mut self, action: ComponentNameAction) -> Result<(), ComponentNameError> {
        self.qualification()?;
        if action != self.action() {
            return Err(ComponentNameError::Protocol(
                ColorProtocolError::WrongAction,
            ));
        }
        match action {
            ComponentNameAction::ReadProfileHeader => {
                let Some(header) = self.facts.profiles.get(self.selection.binding) else {
                    return self.fail(ComponentNameError::InvalidMetadata);
                };
                if header.resource != self.selection.resource
                    || header.intent != self.selection.intent
                    || header.availability != self.selection.availability
                    || header.parameter_count != self.selection.parameter_count
                    || header.components.len() != self.selection.named_count
                {
                    return self.fail(ComponentNameError::InvalidMetadata);
                }
                match header.availability {
                    CssColorProfileAvailability::Unknown => {
                        return self.fail(ComponentNameError::Unavailable);
                    }
                    CssColorProfileAvailability::Loaded if header.parameter_count != 0 => {}
                    CssColorProfileAvailability::Loaded => {
                        return self.fail(ComponentNameError::InvalidMetadata);
                    }
                    _ => return self.fail(ComponentNameError::InvalidProfile),
                }
                self.state = if self.selection.named_count == 0 {
                    ComponentState::Finish
                } else {
                    ComponentState::Name
                };
            }
            ComponentNameAction::ReadNameHeader { ordinal } => {
                let Some(name) = self
                    .facts
                    .profiles
                    .get(self.selection.binding)
                    .and_then(|header| header.components.get(ordinal))
                else {
                    return self.fail(ComponentNameError::InvalidMetadata);
                };
                self.comparison = Some(NameComparison::start(self.name, name.as_str().len()));
                self.state = ComponentState::Compare;
            }
            ComponentNameAction::ReadNameByte { ordinal, index } => {
                let Some(value) = self
                    .facts
                    .profiles
                    .get(self.selection.binding)
                    .and_then(|header| header.components.get(ordinal))
                    .and_then(|name| name.as_str().as_bytes().get(index))
                    .copied()
                else {
                    return self.fail(ComponentNameError::InvalidMetadata);
                };
                self.comparison
                    .as_mut()
                    .ok_or(ComponentNameError::Protocol(
                        ColorProtocolError::WrongAction,
                    ))?
                    .fact(index, value)
                    .map_err(ComponentNameError::Protocol)?;
            }
            ComponentNameAction::FinishCandidate => {
                if self.comparison.as_ref().is_some_and(NameComparison::equal) {
                    if self.ordinal >= self.selection.parameter_count {
                        return self.fail(ComponentNameError::UnknownChannel);
                    }
                    self.state = ComponentState::Done;
                } else {
                    if self.ordinal >= self.selection.named_count {
                        return self.fail(ComponentNameError::UnknownChannel);
                    }
                    let Some(next) = self.ordinal.checked_add(1) else {
                        return self
                            .fail(ComponentNameError::Protocol(ColorProtocolError::Overflow));
                    };
                    if next == self.selection.named_count {
                        return self.fail(ComponentNameError::UnknownChannel);
                    }
                    self.ordinal = next;
                    self.state = ComponentState::Name;
                }
            }
            _ => {
                return Err(ComponentNameError::Protocol(
                    ColorProtocolError::WrongAction,
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn result(&self) -> Result<usize, ComponentNameError> {
        if matches!(self.state, ComponentState::Done) {
            Ok(self.ordinal)
        } else {
            Err(ComponentNameError::Stopped)
        }
    }
    pub(crate) fn abandon(&mut self) {
        self.state = ComponentState::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::CssColorProfileName;
    use crate::color::controlled::CssColorOperationOwner;
    use crate::color::controlled::protocol::{ActionDescription, LexicalOperation};
    use crate::color::{
        CssColorProfileBinding, CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact,
    };
    use surgeist_css_parser::input::InputKind;
    fn lex<'a>(operation: &mut LexicalOperation<'a>, input: &str) -> Vec<CssColorSlot<'a>> {
        let mut slots = Vec::new();
        loop {
            let mut action = operation.next_action();
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
                ActionDescription::Stopped => panic!("stop"),
            }
        }
        slots
    }
    fn finish(
        lookup: &mut ProfileLookup<'_, '_>,
        slots: &[CssColorSlot<'_>],
    ) -> Result<ProfileSelection, ProfileError> {
        loop {
            match lookup.action() {
                ProfileAction::ReadPrefix { index } | ProfileAction::ReadSource { index } => {
                    lookup.read_source(index, &slots[index])?
                }
                ProfileAction::Done => return lookup.result(),
                action => lookup.step(action)?,
            }
        }
    }
    #[test]
    fn decoded_unicode_case_sensitive_name_keeps_availability_and_resource() {
        let input = "--\\e9  X"; // Escaped é plus an insignificant escape terminator, then X.
        let mut owner = CssColorOperationOwner::new();
        let mut operation =
            LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
        let slots = lex(&mut operation, input);
        let table = operation.finish().unwrap();
        // The extra space terminates the identifier; use the first token, not
        // an assumed source-character or node ordinal.
        let token = (table.first_token_record()..table.length()).find(|index| matches!(table.observe(*index, &slots[*index]), Ok(LexicalRecord::Token(token)) if matches!(token.kind, RawTokenKind::Name { kind: NameKind::Ident, .. }))).unwrap();
        let matching = CssColorProfileName::try_new("--é").unwrap();
        let wrong_case = CssColorProfileName::try_new("--É").unwrap();
        let id = CssColorFactIdentity::new(1, 2);
        let resource = CssColorFactIdentity::new(3, 4);
        let bindings = [
            CssColorProfileBinding::new(
                &wrong_case,
                id,
                CssColorProfileAvailability::Loaded,
                CssColorProfileRenderingIntent::Perceptual,
                4,
                &[],
            ),
            CssColorProfileBinding::new(
                &matching,
                resource,
                CssColorProfileAvailability::Absent,
                CssColorProfileRenderingIntent::RelativeColorimetric,
                0,
                &[],
            ),
        ];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &bindings,
            &[],
            CssDeviceColorFact::Unknown,
        );
        let mut lookup =
            ProfileLookup::start(&table.records(), token, &slots[token], &facts, id).unwrap();
        let selected = finish(&mut lookup, &slots).unwrap();
        assert_eq!(selected.binding, 1);
        assert_eq!(selected.availability, CssColorProfileAvailability::Absent);
        assert_eq!(selected.resource, resource);
        lookup.abandon();
        assert_eq!(lookup.result(), Err(ProfileError::Stopped));
    }
    #[test]
    fn absent_registry_record_is_missing_not_known_absent_and_duplicate_requested_key_refuses() {
        let input = "--p";
        let mut owner = CssColorOperationOwner::new();
        let mut operation =
            LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
        let slots = lex(&mut operation, input);
        let table = operation.finish().unwrap();
        let token = table.length() - 1;
        let id = CssColorFactIdentity::new(4, 5);
        let name = CssColorProfileName::try_new(input).unwrap();
        let absent = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        );
        assert_eq!(
            finish(
                &mut ProfileLookup::start(&table.records(), token, &slots[token], &absent, id)
                    .unwrap(),
                &slots
            ),
            Err(ProfileError::Missing)
        );
        let bindings = [CssColorProfileBinding::new(
            &name,
            id,
            CssColorProfileAvailability::Unknown,
            CssColorProfileRenderingIntent::Perceptual,
            0,
            &[],
        ); 2];
        let duplicated = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &bindings,
            &[],
            CssDeviceColorFact::Unknown,
        );
        assert_eq!(
            finish(
                &mut ProfileLookup::start(&table.records(), token, &slots[token], &duplicated, id)
                    .unwrap(),
                &slots
            ),
            Err(ProfileError::Duplicate {
                first: 0,
                second: 1
            })
        );
        let mut foreign = ProfileLookup::start(
            &table.records(),
            token,
            &slots[token],
            &absent,
            CssColorFactIdentity::new(4, 6),
        )
        .unwrap();
        let action = foreign.action();
        let ProfileAction::ReadPrefix { index } = action else {
            panic!("prefix")
        };
        assert_eq!(
            foreign.read_source(index, &slots[index]),
            Err(ProfileError::ForeignFacts)
        );
        assert_eq!(foreign.action(), action);
    }
    fn named_finish(
        lookup: &mut ProfileComponentLookup<'_, '_>,
        slots: &[CssColorSlot<'_>],
    ) -> Result<usize, ComponentNameError> {
        loop {
            match lookup.action() {
                ComponentNameAction::ReadSource { index } => {
                    lookup.read_source(index, &slots[index])?
                }
                ComponentNameAction::Done => return lookup.result(),
                action => lookup.step(action)?,
            }
        }
    }
    #[test]
    fn actual_parameter_count_and_first_exact_named_ordinal_are_independent() {
        let input = "--p é É ghost";
        let mut owner = CssColorOperationOwner::new();
        let mut operation =
            LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
        let slots = lex(&mut operation, input);
        let table = operation.finish().unwrap();
        let tokens: Vec<_> = (table.first_token_record()..table.length()).filter(|index| matches!(table.observe(*index, &slots[*index]), Ok(LexicalRecord::Token(token)) if matches!(token.kind, RawTokenKind::Name { kind: NameKind::Ident, .. }))).collect();
        assert_eq!(tokens.len(), 4);
        let name = CssColorProfileName::try_new("--p").unwrap();
        let names = [
            crate::color::CssColorProfileComponentName::try_new("é").unwrap(),
            crate::color::CssColorProfileComponentName::try_new("é").unwrap(),
            crate::color::CssColorProfileComponentName::try_new("É").unwrap(),
        ];
        let id = CssColorFactIdentity::new(1, 1);
        let resource = CssColorFactIdentity::new(2, 3);
        let bindings = [CssColorProfileBinding::new(
            &name,
            resource,
            CssColorProfileAvailability::Loaded,
            CssColorProfileRenderingIntent::RelativeColorimetric,
            4,
            &names,
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &bindings,
            &[],
            CssDeviceColorFact::Unknown,
        );
        let selection = finish(
            &mut ProfileLookup::start(&table.records(), tokens[0], &slots[tokens[0]], &facts, id)
                .unwrap(),
            &slots,
        )
        .unwrap();
        assert_eq!(selection.parameter_count, 4);
        assert_eq!(selection.named_count, 3);
        assert_eq!(
            named_finish(
                &mut ProfileComponentLookup::start(
                    &table.records(),
                    tokens[1],
                    &slots[tokens[1]],
                    &facts,
                    selection
                )
                .unwrap(),
                &slots
            ),
            Ok(0)
        );
        assert_eq!(
            named_finish(
                &mut ProfileComponentLookup::start(
                    &table.records(),
                    tokens[2],
                    &slots[tokens[2]],
                    &facts,
                    selection
                )
                .unwrap(),
                &slots
            ),
            Ok(2)
        );
        assert_eq!(
            named_finish(
                &mut ProfileComponentLookup::start(
                    &table.records(),
                    tokens[3],
                    &slots[tokens[3]],
                    &facts,
                    selection
                )
                .unwrap(),
                &slots
            ),
            Err(ComponentNameError::UnknownChannel)
        );
        // The first exact name retains its actual ordinal even when later
        // descriptor names are beyond the profile's real parameter count.
        let fewer = [CssColorProfileBinding::new(
            &name,
            resource,
            CssColorProfileAvailability::Loaded,
            CssColorProfileRenderingIntent::RelativeColorimetric,
            1,
            &names,
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &fewer,
            &[],
            CssDeviceColorFact::Unknown,
        );
        let selection = finish(
            &mut ProfileLookup::start(&table.records(), tokens[0], &slots[tokens[0]], &facts, id)
                .unwrap(),
            &slots,
        )
        .unwrap();
        assert_eq!(
            named_finish(
                &mut ProfileComponentLookup::start(
                    &table.records(),
                    tokens[1],
                    &slots[tokens[1]],
                    &facts,
                    selection
                )
                .unwrap(),
                &slots
            ),
            Ok(0)
        );
        assert_eq!(
            named_finish(
                &mut ProfileComponentLookup::start(
                    &table.records(),
                    tokens[2],
                    &slots[tokens[2]],
                    &facts,
                    selection
                )
                .unwrap(),
                &slots
            ),
            Err(ComponentNameError::UnknownChannel)
        );
    }
    #[test]
    fn unnamed_loaded_profile_and_malformed_zero_are_not_inferred_from_descriptor() {
        let input = "--p channel";
        let mut owner = CssColorOperationOwner::new();
        let mut operation =
            LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
        let slots = lex(&mut operation, input);
        let table = operation.finish().unwrap();
        let tokens: Vec<_> = (table.first_token_record()..table.length()).filter(|index| matches!(table.observe(*index, &slots[*index]), Ok(LexicalRecord::Token(token)) if matches!(token.kind, RawTokenKind::Name { kind: NameKind::Ident, .. }))).collect();
        let name = CssColorProfileName::try_new("--p").unwrap();
        let id = CssColorFactIdentity::new(5, 6);
        let bindings = [CssColorProfileBinding::new(
            &name,
            id,
            CssColorProfileAvailability::Loaded,
            CssColorProfileRenderingIntent::Perceptual,
            4,
            &[],
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &bindings,
            &[],
            CssDeviceColorFact::Unknown,
        );
        let selection = finish(
            &mut ProfileLookup::start(&table.records(), tokens[0], &slots[tokens[0]], &facts, id)
                .unwrap(),
            &slots,
        )
        .unwrap();
        assert_eq!(selection.parameter_count, 4);
        assert_eq!(selection.named_count, 0);
        assert_eq!(
            named_finish(
                &mut ProfileComponentLookup::start(
                    &table.records(),
                    tokens[1],
                    &slots[tokens[1]],
                    &facts,
                    selection
                )
                .unwrap(),
                &slots
            ),
            Err(ComponentNameError::UnknownChannel)
        );
        let mut stale = selection;
        stale.resource = CssColorFactIdentity::new(5, 7);
        let mut lookup = ProfileComponentLookup::start(
            &table.records(),
            tokens[1],
            &slots[tokens[1]],
            &facts,
            stale,
        )
        .unwrap();
        assert_eq!(
            lookup.step(lookup.action()),
            Err(ComponentNameError::InvalidMetadata)
        );
        assert_eq!(lookup.action(), ComponentNameAction::Stopped);
        lookup.abandon();
        assert_eq!(lookup.result(), Err(ComponentNameError::Stopped));
        let zero = [CssColorProfileBinding::new(
            &name,
            id,
            CssColorProfileAvailability::Loaded,
            CssColorProfileRenderingIntent::Perceptual,
            0,
            &[],
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &zero,
            &[],
            CssDeviceColorFact::Unknown,
        );
        assert_eq!(
            finish(
                &mut ProfileLookup::start(
                    &table.records(),
                    tokens[0],
                    &slots[tokens[0]],
                    &facts,
                    id
                )
                .unwrap(),
                &slots
            ),
            Err(ProfileError::InvalidMetadata)
        );
    }
}
