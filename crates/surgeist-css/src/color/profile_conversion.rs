//! Admitted custom-channel normalization and concrete forward conversion join.

use super::channels::{self, Component};
use super::facts::{
    CssColorFactIdentity, CssColorFacts, CssColorProfileAvailability, CssColorProfileConversion,
};
use super::profile::ProfileSelection;
use super::protocol::{CssColorProtocolError, CssColorSlot, Data, LexicalTable};
use super::{CssUsedColor, CssUsedColorError};
use crate::{CssColorInterpolationSpace, tokenization::color_tokens::Span};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Protocol(CssColorProtocolError),
    ForeignFacts,
    InvalidMetadata,
    InvalidColor,
    Unavailable,
    Unsupported,
    DuplicateConversion,
    Numerical(CssUsedColorError),
    Stopped,
}
fn checked_header(facts: &CssColorFacts<'_>, selection: ProfileSelection) -> Result<(), Error> {
    if facts.identity != selection.facts {
        return Err(Error::ForeignFacts);
    }
    let header = facts
        .profiles
        .get(selection.binding)
        .ok_or(Error::InvalidMetadata)?;
    if header.resource != selection.resource
        || header.intent != selection.intent
        || header.availability != selection.availability
        || header.parameter_count != selection.parameter_count
        || header.components.len() != selection.named_count
    {
        return Err(Error::InvalidMetadata);
    }
    match header.availability {
        CssColorProfileAvailability::Unknown => Err(Error::Unavailable),
        CssColorProfileAvailability::Loaded if header.parameter_count != 0 => Ok(()),
        CssColorProfileAvailability::Loaded => Err(Error::InvalidMetadata),
        _ => Err(Error::InvalidColor),
    }
}
/// One admitted fixed-header check precedes this precise reached capability.
/// The current conversion record outputs a three-channel standard-space color;
/// it cannot lend arbitrary profile channels for a reversed relative origin.
pub(crate) fn reverse_custom_channels(
    facts: &CssColorFacts<'_>,
    selection: ProfileSelection,
) -> Result<(), Error> {
    checked_header(facts, selection)?;
    Err(Error::Unsupported)
}

/// Immutable header over the caller's actual normalized input slots.
#[derive(Clone, Copy)]
pub(crate) struct ProfileInputs<'owner> {
    table: LexicalTable<'owner>,
    selection: ProfileSelection,
    span: Span,
    alpha: Option<f64>,
}
impl ProfileInputs<'_> {
    pub(crate) const fn length(&self) -> usize {
        self.selection.parameter_count
    }
    pub(crate) const fn index(&self, ordinal: usize) -> Option<usize> {
        if ordinal < self.length() {
            self.span.start.checked_add(ordinal)
        } else {
            None
        }
    }
    pub(crate) fn read(
        &self,
        ordinal: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<Option<f64>, Error> {
        slot.qualify(
            self.table.owner,
            self.table.phase,
            slot.index,
            super::protocol::CssColorBank::Algorithm,
        )
        .map_err(Error::Protocol)?;
        if self.index(ordinal) != Some(slot.index) || slot.index >= self.span.end {
            return Err(Error::Protocol(CssColorProtocolError::WrongIndex));
        }
        let Data::ProfileChannel {
            resource,
            ordinal: actual,
            value,
        } = slot.data
        else {
            return Err(Error::Protocol(CssColorProtocolError::WrongSlot));
        };
        if resource != self.selection.resource || actual != ordinal {
            return Err(Error::Protocol(CssColorProtocolError::WrongSlot));
        }
        Ok(value)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputAction {
    ReadHeader,
    ReadComponent { ordinal: usize },
    Append { index: usize },
    Done,
    Stopped,
}
enum InputState {
    Header,
    Component,
    Pending,
    Done,
    Stopped,
}
pub(crate) struct ProfileInputBuilder<'owner, 'facts> {
    table: LexicalTable<'owner>,
    facts: &'facts CssColorFacts<'facts>,
    selection: ProfileSelection,
    supplied: usize,
    input: usize,
    output: usize,
    base: usize,
    pending: Option<f64>,
    alpha: Option<f64>,
    state: InputState,
}
impl<'owner, 'facts> ProfileInputBuilder<'owner, 'facts> {
    pub(crate) fn start(
        table: &LexicalTable<'owner>,
        facts: &'facts CssColorFacts<'facts>,
        selection: ProfileSelection,
        supplied: usize,
        base: usize,
        alpha: Option<f64>,
    ) -> Result<Self, Error> {
        if supplied == 0 {
            return Err(Error::InvalidColor);
        }
        if base < table.length() || base.checked_add(selection.parameter_count).is_none() {
            return Err(Error::Protocol(CssColorProtocolError::Overflow));
        }
        if alpha.is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value)) {
            return Err(Error::InvalidColor);
        }
        Ok(Self {
            table: *table,
            facts,
            selection,
            supplied,
            input: 0,
            output: 0,
            base,
            pending: None,
            alpha,
            state: InputState::Header,
        })
    }
    pub(crate) fn action(&self) -> InputAction {
        match self.state {
            InputState::Header => InputAction::ReadHeader,
            InputState::Component => InputAction::ReadComponent {
                ordinal: self.input,
            },
            InputState::Pending => InputAction::Append {
                index: self.base + self.output,
            },
            InputState::Done => InputAction::Done,
            InputState::Stopped => InputAction::Stopped,
        }
    }
    fn fail(&mut self, error: Error) -> Result<(), Error> {
        self.state = InputState::Stopped;
        Err(error)
    }
    pub(crate) fn read_header(&mut self) -> Result<(), Error> {
        if self.action() != InputAction::ReadHeader {
            return Err(Error::Protocol(CssColorProtocolError::WrongAction));
        }
        if let Err(error) = checked_header(self.facts, self.selection) {
            if error == Error::ForeignFacts {
                return Err(error);
            }
            return self.fail(error);
        }
        self.state = InputState::Component;
        Ok(())
    }
    pub(crate) fn component(&mut self, ordinal: usize, component: Component) -> Result<(), Error> {
        if self.facts.identity != self.selection.facts {
            return Err(Error::ForeignFacts);
        }
        if self.action() != (InputAction::ReadComponent { ordinal }) {
            return Err(Error::Protocol(CssColorProtocolError::WrongAction));
        }
        let value = match channels::scalar(component, 1.0) {
            Ok(value) => value.map(|value| value.clamp(0.0, 1.0)),
            Err(_) => return self.fail(Error::InvalidColor),
        };
        let Some(next) = self.input.checked_add(1) else {
            return self.fail(Error::Protocol(CssColorProtocolError::Overflow));
        };
        self.input = next;
        if self.output < self.selection.parameter_count {
            self.pending = value;
            self.state = InputState::Pending;
        } else {
            self.after_input();
        }
        Ok(())
    }
    fn after_input(&mut self) {
        if self.input < self.supplied {
            self.state = InputState::Component;
        } else if self.output < self.selection.parameter_count {
            self.pending = Some(0.0);
            self.state = InputState::Pending;
        } else {
            self.state = InputState::Done;
        }
    }
    pub(crate) fn record(&self, index: usize) -> Result<CssColorSlot<'owner>, Error> {
        if self.action() != (InputAction::Append { index }) {
            return Err(Error::Protocol(CssColorProtocolError::WrongAction));
        }
        self.output
            .checked_add(1)
            .ok_or(Error::Protocol(CssColorProtocolError::Overflow))?;
        index
            .checked_add(1)
            .ok_or(Error::Protocol(CssColorProtocolError::Overflow))?;
        Ok(CssColorSlot::new(
            self.table.owner,
            self.table.phase,
            index,
            Data::ProfileChannel {
                resource: self.selection.resource,
                ordinal: self.output,
                value: self.pending,
            },
        ))
    }
    pub(crate) fn appended(&mut self, index: usize, slot: &CssColorSlot<'_>) -> Result<(), Error> {
        if self.action() != (InputAction::Append { index }) {
            return Err(Error::Protocol(CssColorProtocolError::WrongAction));
        }
        slot.qualify(
            self.table.owner,
            self.table.phase,
            index,
            super::protocol::CssColorBank::Algorithm,
        )
        .map_err(Error::Protocol)?;
        if slot.index != index {
            return Err(Error::Protocol(CssColorProtocolError::WrongIndex));
        }
        let Data::ProfileChannel {
            resource,
            ordinal,
            value,
        } = slot.data
        else {
            return Err(Error::Protocol(CssColorProtocolError::WrongSlot));
        };
        if resource != self.selection.resource
            || ordinal != self.output
            || value.map(f64::to_bits) != self.pending.map(f64::to_bits)
        {
            return Err(Error::Protocol(CssColorProtocolError::WrongSlot));
        }
        let next = self
            .output
            .checked_add(1)
            .ok_or(Error::Protocol(CssColorProtocolError::Overflow))?;
        self.output = next;
        self.after_input();
        Ok(())
    }
    pub(crate) fn finish(self) -> Result<ProfileInputs<'owner>, Error> {
        if !matches!(self.state, InputState::Done) {
            return Err(Error::Stopped);
        }
        Ok(ProfileInputs {
            table: self.table,
            selection: self.selection,
            span: Span {
                start: self.base,
                end: self.base + self.output,
            },
            alpha: self.alpha,
        })
    }
    pub(crate) fn abandon(&mut self) {
        self.state = InputState::Stopped;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversionAction {
    ReadLength,
    ReadHeader { ordinal: usize },
    ReadInput { index: usize, ordinal: usize },
    ReadFactChannel { conversion: usize, ordinal: usize },
    FinishCandidate,
    Commit,
    Done,
    Stopped,
}
enum ConversionState {
    Length,
    Header,
    Input,
    Channel,
    Finish,
    Commit,
    Done,
    Stopped,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProfileResult {
    pub(crate) value: CssUsedColor,
    pub(crate) facts: CssColorFactIdentity,
    pub(crate) resource: CssColorFactIdentity,
}
/// Forward custom profile conversion only. Reverse standard-to-custom RCS
/// needs an arbitrary-channel result carrier absent from the adopted facts.
pub(crate) struct ProfileConversionLookup<'owner, 'facts> {
    inputs: ProfileInputs<'owner>,
    facts: &'facts CssColorFacts<'facts>,
    target: CssColorInterpolationSpace,
    state: ConversionState,
    conversions: usize,
    conversion: usize,
    channel: usize,
    value: f64,
    matched: bool,
    selected: Option<usize>,
    capability: Option<Error>,
    result: Option<ProfileResult>,
}
impl<'owner, 'facts> ProfileConversionLookup<'owner, 'facts> {
    pub(crate) const fn start(
        inputs: &ProfileInputs<'owner>,
        facts: &'facts CssColorFacts<'facts>,
        target: CssColorInterpolationSpace,
    ) -> Self {
        Self {
            inputs: *inputs,
            facts,
            target,
            state: ConversionState::Length,
            conversions: 0,
            conversion: 0,
            channel: 0,
            value: 0.0,
            matched: false,
            selected: None,
            capability: None,
            result: None,
        }
    }
    pub(crate) fn committed_header(&self) -> Option<usize> {
        if matches!(self.state, ConversionState::Commit) {
            self.selected
        } else {
            None
        }
    }
    pub(crate) fn action(&self) -> ConversionAction {
        match self.state {
            ConversionState::Length => ConversionAction::ReadLength,
            ConversionState::Header => ConversionAction::ReadHeader {
                ordinal: self.conversion,
            },
            ConversionState::Input => ConversionAction::ReadInput {
                index: self
                    .inputs
                    .index(self.channel)
                    .expect("actual normalized ordinal"),
                ordinal: self.channel,
            },
            ConversionState::Channel => ConversionAction::ReadFactChannel {
                conversion: self.conversion,
                ordinal: self.channel,
            },
            ConversionState::Finish => ConversionAction::FinishCandidate,
            ConversionState::Commit => ConversionAction::Commit,
            ConversionState::Done => ConversionAction::Done,
            ConversionState::Stopped => ConversionAction::Stopped,
        }
    }
    fn fail(&mut self, error: Error) -> Result<(), Error> {
        self.state = ConversionState::Stopped;
        Err(error)
    }
    pub(crate) fn read_input(
        &mut self,
        index: usize,
        ordinal: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        if self.facts.identity != self.inputs.selection.facts {
            return Err(Error::ForeignFacts);
        }
        if self.action() != (ConversionAction::ReadInput { index, ordinal }) {
            return Err(Error::Protocol(CssColorProtocolError::WrongAction));
        }
        self.value = self.inputs.read(ordinal, slot)?.unwrap_or(0.0);
        self.state = ConversionState::Channel;
        Ok(())
    }
    pub(crate) fn step(&mut self, action: ConversionAction) -> Result<(), Error> {
        if self.facts.identity != self.inputs.selection.facts {
            return Err(Error::ForeignFacts);
        }
        if action != self.action() {
            return Err(Error::Protocol(CssColorProtocolError::WrongAction));
        }
        match action {
            ConversionAction::ReadLength => {
                if let Err(error) = checked_header(self.facts, self.inputs.selection) {
                    return self.fail(error);
                }
                self.conversions = self.facts.conversions.len();
                self.state = if self.conversions == 0 {
                    ConversionState::Commit
                } else {
                    ConversionState::Header
                };
            }
            ConversionAction::ReadHeader { ordinal } => {
                let Some(record) = self.facts.conversions.get(ordinal) else {
                    return self.fail(Error::InvalidMetadata);
                };
                self.matched = false;
                self.channel = 0;
                match *record {
                    CssColorProfileConversion::Unsupported { profile }
                        if profile == self.inputs.selection.resource =>
                    {
                        if self.capability.is_none() {
                            self.capability = Some(Error::Unsupported);
                        }
                        self.state = ConversionState::Finish;
                    }
                    CssColorProfileConversion::Unavailable { profile }
                        if profile == self.inputs.selection.resource =>
                    {
                        self.capability = Some(Error::Unavailable);
                        self.state = ConversionState::Finish;
                    }
                    CssColorProfileConversion::Resolved(record)
                        if record.profile == self.inputs.selection.resource
                            && record.intent == self.inputs.selection.intent
                            && record.target == self.target =>
                    {
                        if record.channels.len() != self.inputs.length()
                            || record.value.space() != self.target
                        {
                            return self.fail(Error::InvalidMetadata);
                        }
                        self.matched = true;
                        self.state = ConversionState::Input;
                    }
                    _ => self.state = ConversionState::Finish,
                }
            }
            ConversionAction::ReadFactChannel {
                conversion,
                ordinal,
            } => {
                let Some(CssColorProfileConversion::Resolved(record)) =
                    self.facts.conversions.get(conversion)
                else {
                    return self.fail(Error::InvalidMetadata);
                };
                let Some(value) = record.channels.get(ordinal).copied() else {
                    return self.fail(Error::InvalidMetadata);
                };
                if !value.is_finite() {
                    return self.fail(Error::InvalidMetadata);
                }
                if value.to_bits() != self.value.to_bits() {
                    self.matched = false;
                    self.state = ConversionState::Finish;
                } else {
                    let Some(next) = self.channel.checked_add(1) else {
                        return self.fail(Error::Protocol(CssColorProtocolError::Overflow));
                    };
                    self.channel = next;
                    self.state = if next == self.inputs.length() {
                        ConversionState::Finish
                    } else {
                        ConversionState::Input
                    };
                }
            }
            ConversionAction::FinishCandidate => {
                if self.matched {
                    if self.selected.is_some() {
                        return self.fail(Error::DuplicateConversion);
                    }
                    self.selected = Some(self.conversion);
                }
                let Some(next) = self.conversion.checked_add(1) else {
                    return self.fail(Error::Protocol(CssColorProtocolError::Overflow));
                };
                self.conversion = next;
                self.state = if next < self.conversions {
                    ConversionState::Header
                } else {
                    ConversionState::Commit
                };
            }
            ConversionAction::Commit => {
                let Some(index) = self.selected else {
                    return self.fail(self.capability.unwrap_or(Error::Unavailable));
                };
                let Some(CssColorProfileConversion::Resolved(record)) =
                    self.facts.conversions.get(index)
                else {
                    return self.fail(Error::InvalidMetadata);
                };
                let value = match record.value.with_resolved_alpha(self.inputs.alpha) {
                    Ok(value) => value,
                    Err(error) => return self.fail(Error::Numerical(error)),
                };
                self.result = Some(ProfileResult {
                    value,
                    facts: self.inputs.selection.facts,
                    resource: self.inputs.selection.resource,
                });
                self.state = ConversionState::Done;
            }
            _ => return Err(Error::Protocol(CssColorProtocolError::WrongAction)),
        }
        Ok(())
    }
    pub(crate) fn result(&self) -> Result<ProfileResult, Error> {
        if matches!(self.state, ConversionState::Done) {
            self.result.ok_or(Error::Stopped)
        } else {
            Err(Error::Stopped)
        }
    }
    pub(crate) fn abandon(&mut self) {
        self.state = ConversionState::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        facts::{
            CssColorProfileBinding, CssColorProfileConversionRecord, CssColorPropertyFacts,
            CssColorSchemeFact, CssDeviceColorFact,
        },
        profile::{ProfileAction, ProfileLookup},
        protocol::{ActionDescription, CssColorOperationOwner, LexicalOperation},
    };
    use super::*;
    use crate::{
        CssColorProfileName, CssColorProfileRenderingIntent as Intent, CssPredefinedColorSpace,
        tokenization::bounded::InputKind,
    };
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
                ActionDescription::Stopped => panic!("stopped"),
            }
        }
        slots
    }
    fn select(
        table: &LexicalTable<'_>,
        slots: &[CssColorSlot<'_>],
        facts: &CssColorFacts<'_>,
    ) -> ProfileSelection {
        let token = table.length() - 1;
        let mut lookup =
            ProfileLookup::start(table, token, &slots[token], facts, facts.identity).unwrap();
        loop {
            match lookup.action() {
                ProfileAction::ReadPrefix { index } | ProfileAction::ReadSource { index } => {
                    lookup.read_source(index, &slots[index]).unwrap()
                }
                ProfileAction::Done => return lookup.result().unwrap(),
                action => lookup.step(action).unwrap(),
            }
        }
    }
    fn inputs<'owner, 'facts>(
        table: &LexicalTable<'owner>,
        facts: &'facts CssColorFacts<'facts>,
        selection: ProfileSelection,
        components: &[Component],
        alpha: Option<f64>,
        slots: &mut Vec<CssColorSlot<'owner>>,
    ) -> Result<ProfileInputs<'owner>, Error> {
        let mut builder = ProfileInputBuilder::start(
            table,
            facts,
            selection,
            components.len(),
            slots.len(),
            alpha,
        )?;
        loop {
            match builder.action() {
                InputAction::ReadHeader => builder.read_header()?,
                InputAction::ReadComponent { ordinal } => {
                    builder.component(ordinal, components[ordinal])?
                }
                InputAction::Append { index } => {
                    let slot = builder.record(index)?;
                    slots.push(slot);
                    builder.appended(index, &slots[index])?;
                }
                InputAction::Done => return builder.finish(),
                InputAction::Stopped => return Err(Error::Stopped),
            }
        }
    }
    fn finish(
        cursor: &mut ProfileConversionLookup<'_, '_>,
        slots: &[CssColorSlot<'_>],
    ) -> Result<ProfileResult, Error> {
        loop {
            match cursor.action() {
                ConversionAction::ReadInput { index, ordinal } => {
                    cursor.read_input(index, ordinal, &slots[index])?
                }
                ConversionAction::Done => return cursor.result(),
                action => cursor.step(action)?,
            }
        }
    }
    fn facts<'a>(
        bindings: &'a [CssColorProfileBinding<'a>],
        conversions: &'a [CssColorProfileConversion<'a>],
    ) -> CssColorFacts<'a> {
        CssColorFacts::new(
            CssColorFactIdentity::new(1, 2),
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            bindings,
            conversions,
            CssDeviceColorFact::Unknown,
        )
    }
    fn srgb() -> CssUsedColor {
        CssUsedColor::try_new(
            CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
            [Some(0.2), Some(0.4), Some(0.6)],
            Some(1.0),
        )
        .unwrap()
    }
    #[test]
    fn unnamed_four_parameter_inputs_default_zero_and_join_exact_forward_record() {
        let name = CssColorProfileName::try_new("--p").unwrap();
        let resource = CssColorFactIdentity::new(7, 8);
        let bindings = [CssColorProfileBinding::new(
            &name,
            resource,
            CssColorProfileAvailability::Loaded,
            Intent::Perceptual,
            4,
            &[],
        )];
        let channels = [0.0, 0.5, 0.0, 0.0];
        let value = srgb();
        let conversions = [CssColorProfileConversion::Resolved(
            CssColorProfileConversionRecord::new(
                resource,
                &channels,
                Intent::Perceptual,
                value.space(),
                value,
            ),
        )];
        let facts = facts(&bindings, &conversions);
        let mut owner = CssColorOperationOwner::new();
        let mut operation = LexicalOperation::start(&mut owner, InputKind::Utf8, 3).unwrap();
        let mut slots = lex(&mut operation, "--p");
        let table = operation.finish().unwrap();
        let selection = select(&table, &slots, &facts);
        let original = slots.len();
        let input = inputs(
            &table,
            &facts,
            selection,
            &[
                Component::number(-2.0),
                Component {
                    domain: channels::Domain::Percentage,
                    value: 50.0,
                },
            ],
            None,
            &mut slots,
        )
        .unwrap();
        assert_eq!(input.length(), 4);
        assert_eq!(slots.len(), original + 4);
        for (ordinal, expected) in channels.into_iter().enumerate() {
            assert_eq!(
                input.read(ordinal, &slots[input.index(ordinal).unwrap()]),
                Ok(Some(expected))
            );
        }
        let result = finish(
            &mut ProfileConversionLookup::start(&input, &facts, value.space()),
            &slots,
        )
        .unwrap();
        assert_eq!(result.value.channels(), value.channels());
        assert_eq!(result.value.alpha(), None);
        assert_eq!(result.resource, resource);
        assert_eq!(result.facts, facts.identity);
        assert_eq!(
            reverse_custom_channels(&facts, selection),
            Err(Error::Unsupported)
        );
        let mut stale = selection;
        stale.resource = CssColorFactIdentity::new(7, 9);
        assert_eq!(
            reverse_custom_channels(&facts, stale),
            Err(Error::InvalidMetadata)
        );
    }
    #[test]
    fn excess_checked_inputs_are_ignored_and_missing_channel_stays_distinct() {
        let name = CssColorProfileName::try_new("--p").unwrap();
        let resource = CssColorFactIdentity::new(7, 8);
        let names = [crate::CssColorProfileComponentName::try_new("first").unwrap()];
        let bindings = [CssColorProfileBinding::new(
            &name,
            resource,
            CssColorProfileAvailability::Loaded,
            Intent::RelativeColorimetric,
            2,
            &names,
        )];
        let facts = facts(&bindings, &[]);
        let mut owner = CssColorOperationOwner::new();
        let mut operation = LexicalOperation::start(&mut owner, InputKind::Utf8, 3).unwrap();
        let mut slots = lex(&mut operation, "--p");
        let table = operation.finish().unwrap();
        let selection = select(&table, &slots, &facts);
        let before = slots.len();
        let input = inputs(
            &table,
            &facts,
            selection,
            &[
                Component::MISSING,
                Component::number(f64::INFINITY),
                Component::number(0.75),
            ],
            Some(0.5),
            &mut slots,
        )
        .unwrap();
        assert_eq!(slots.len(), before + 2);
        assert_eq!(input.read(0, &slots[before]), Ok(None));
        assert_eq!(input.read(1, &slots[before + 1]), Ok(Some(1.0)));
        assert_eq!(
            finish(
                &mut ProfileConversionLookup::start(&input, &facts, srgb().space()),
                &slots
            ),
            Err(Error::Unavailable)
        );
        let mut builder =
            ProfileInputBuilder::start(&table, &facts, selection, 3, slots.len(), Some(1.0))
                .unwrap();
        builder.read_header().unwrap();
        for ordinal in 0..2 {
            builder.component(ordinal, Component::number(0.25)).unwrap();
            let InputAction::Append { index } = builder.action() else {
                panic!("append")
            };
            let slot = builder.record(index).unwrap();
            builder.appended(index, &slot).unwrap();
        }
        // An ignored excess value still crosses the canonical numeric-domain
        // grammar boundary; ignored does not mean author-invalid is accepted.
        assert_eq!(
            builder.component(
                2,
                Component {
                    domain: channels::Domain::Degrees,
                    value: 1.0
                }
            ),
            Err(Error::InvalidColor)
        );
        assert_eq!(builder.action(), InputAction::Stopped);
    }
    #[test]
    fn mismatched_inputs_and_duplicate_conversion_are_not_contextual_success() {
        let name = CssColorProfileName::try_new("--p").unwrap();
        let resource = CssColorFactIdentity::new(7, 8);
        let bindings = [CssColorProfileBinding::new(
            &name,
            resource,
            CssColorProfileAvailability::Loaded,
            Intent::Perceptual,
            1,
            &[],
        )];
        let positive = [0.0];
        let negative = [-0.0];
        let value = srgb();
        let records = [
            CssColorProfileConversion::Resolved(CssColorProfileConversionRecord::new(
                resource,
                &positive,
                Intent::Perceptual,
                value.space(),
                value,
            )),
            CssColorProfileConversion::Resolved(CssColorProfileConversionRecord::new(
                resource,
                &negative,
                Intent::Perceptual,
                value.space(),
                value,
            )),
        ];
        let facts = facts(&bindings, &records);
        let mut owner = CssColorOperationOwner::new();
        let mut operation = LexicalOperation::start(&mut owner, InputKind::Utf8, 3).unwrap();
        let mut slots = lex(&mut operation, "--p");
        let table = operation.finish().unwrap();
        let selection = select(&table, &slots, &facts);
        let input = inputs(
            &table,
            &facts,
            selection,
            &[Component::number(-0.0)],
            Some(1.0),
            &mut slots,
        )
        .unwrap();
        assert_eq!(
            input
                .read(0, &slots[input.index(0).unwrap()])
                .unwrap()
                .unwrap()
                .to_bits(),
            (-0.0f64).to_bits()
        );
        assert!(
            finish(
                &mut ProfileConversionLookup::start(&input, &facts, value.space()),
                &slots
            )
            .is_ok()
        );
        let duplicates = [records[1]; 2];
        let duplicate_facts = self::facts(&bindings, &duplicates);
        assert_eq!(
            finish(
                &mut ProfileConversionLookup::start(&input, &duplicate_facts, value.space()),
                &slots
            ),
            Err(Error::DuplicateConversion)
        );
        let unsupported = [CssColorProfileConversion::Unsupported { profile: resource }];
        let unsupported_facts = self::facts(&bindings, &unsupported);
        assert_eq!(
            finish(
                &mut ProfileConversionLookup::start(&input, &unsupported_facts, value.space()),
                &slots
            ),
            Err(Error::Unsupported)
        );
        let nan = [f64::NAN];
        let invalid = [CssColorProfileConversion::Resolved(
            CssColorProfileConversionRecord::new(
                resource,
                &nan,
                Intent::Perceptual,
                value.space(),
                value,
            ),
        )];
        let invalid_facts = self::facts(&bindings, &invalid);
        assert_eq!(
            finish(
                &mut ProfileConversionLookup::start(&input, &invalid_facts, value.space()),
                &slots
            ),
            Err(Error::InvalidMetadata)
        );
    }
    #[test]
    fn refused_append_foreign_slot_and_abandon_keep_actual_caller_storage() {
        let name = CssColorProfileName::try_new("--p").unwrap();
        let resource = CssColorFactIdentity::new(7, 8);
        let bindings = [CssColorProfileBinding::new(
            &name,
            resource,
            CssColorProfileAvailability::Loaded,
            Intent::Perceptual,
            1,
            &[],
        )];
        let facts = facts(&bindings, &[]);
        let mut owner = CssColorOperationOwner::new();
        let mut operation = LexicalOperation::start(&mut owner, InputKind::Utf8, 3).unwrap();
        let slots = lex(&mut operation, "--p");
        let table = operation.finish().unwrap();
        let selection = select(&table, &slots, &facts);
        let mut builder =
            ProfileInputBuilder::start(&table, &facts, selection, 1, slots.len(), Some(1.0))
                .unwrap();
        builder.read_header().unwrap();
        builder.component(0, Component::number(0.25)).unwrap();
        let action = builder.action();
        let InputAction::Append { index } = action else {
            panic!("append")
        };
        {
            let _refused = builder.record(index).unwrap();
        }
        assert_eq!(builder.action(), action);
        let local = builder.record(index).unwrap();
        let mut foreign_owner = CssColorOperationOwner::new();
        let mut other = LexicalOperation::start(&mut foreign_owner, InputKind::Utf8, 3).unwrap();
        let other_slots = lex(&mut other, "--p");
        let other_table = other.finish().unwrap();
        let other_selection = select(&other_table, &other_slots, &facts);
        let mut other_builder = ProfileInputBuilder::start(
            &other_table,
            &facts,
            other_selection,
            1,
            slots.len(),
            Some(1.0),
        )
        .unwrap();
        other_builder.read_header().unwrap();
        other_builder.component(0, Component::number(0.25)).unwrap();
        let foreign = other_builder.record(index).unwrap();
        assert_eq!(
            builder.appended(index, &foreign),
            Err(Error::Protocol(CssColorProtocolError::ForeignSlot))
        );
        assert_eq!(builder.action(), action);
        builder.appended(index, &local).unwrap();
        let input = builder.finish().unwrap();
        assert_eq!(
            input.read(0, &foreign),
            Err(Error::Protocol(CssColorProtocolError::ForeignSlot))
        );
        assert_eq!(input.read(0, &local), Ok(Some(0.25)));
        let mut cursor = ProfileConversionLookup::start(&input, &facts, srgb().space());
        cursor.abandon();
        assert_eq!(cursor.result(), Err(Error::Stopped));
        other_builder.abandon();
        assert!(other_builder.finish().is_err());
        assert_eq!(input.read(0, &local), Ok(Some(0.25)));
    }
}
