//! Computed ink coordinates and the selected #822 colorimetric branch.

use super::channels::{self, Component, Domain, Error as ChannelError};
use super::facts::{CssColorFactIdentity, CssColorFacts, CssDeviceColorFact};
use super::{CssUsedColor, CssUsedColorError};
use crate::{CssColorInterpolationSpace as Space, CssColorSyntax, CssPredefinedColorSpace};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ComputedCmyk {
    pub(crate) channels: [Option<f64>; 4],
    pub(crate) alpha: Option<f64>,
}
impl ComputedCmyk {
    pub(crate) fn new(
        components: [Component; 4],
        alpha: Option<Component>,
        syntax: CssColorSyntax,
    ) -> Result<Self, ChannelError> {
        let legacy = syntax == CssColorSyntax::Legacy;
        if legacy
            && (alpha.is_some()
                || components
                    .iter()
                    .any(|value| value.domain != Domain::Number))
        {
            return Err(ChannelError::Grammar);
        }
        let one = |component| {
            channels::scalar(component, 1.0).map(|value| value.map(|value| value.clamp(0.0, 1.0)))
        };
        Ok(Self {
            channels: [
                one(components[0])?,
                one(components[1])?,
                one(components[2])?,
                one(components[3])?,
            ],
            alpha: channels::alpha(alpha, legacy)?,
        })
    }
    fn naive(self) -> Result<CssUsedColor, CssUsedColorError> {
        let [c, m, y, k] = self.channels.map(|value| value.unwrap_or(0.0));
        let result = crate::naively_convert_cmyk_to_srgba([c, m, y, k, self.alpha.unwrap_or(1.0)])
            .map_err(|_| CssUsedColorError::Unrepresentable)?;
        CssUsedColor::try_new(
            Space::Predefined(CssPredefinedColorSpace::Srgb),
            [Some(result[0]), Some(result[1]), Some(result[2])],
            self.alpha,
        )
        .map(CssUsedColor::with_legacy_srgb)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeviceError {
    WrongAction,
    ForeignFacts,
    Unavailable,
    ConversionMismatch,
    InvalidFacts,
    Numerical(CssUsedColorError),
    Stopped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeviceAction {
    ReadHeader,
    ReadChannel { index: usize },
    ComputeFallback,
    CommitConversion,
    Done,
    Stopped,
}
enum State {
    Header,
    Channel(usize),
    Fallback,
    Commit,
    Done,
    Stopped,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DeviceResult {
    pub(crate) value: CssUsedColor,
    pub(crate) facts: CssColorFactIdentity,
    pub(crate) resource: Option<CssColorFactIdentity>,
    pub(crate) intent: Option<crate::CssColorProfileRenderingIntent>,
    pub(crate) computed: ComputedCmyk,
}
pub(crate) struct DeviceResolution<'facts> {
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    computed: ComputedCmyk,
    state: State,
    result: Option<DeviceResult>,
}
impl<'facts> DeviceResolution<'facts> {
    pub(crate) const fn start(
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
        computed: ComputedCmyk,
    ) -> Self {
        Self {
            facts,
            expected,
            computed,
            state: State::Header,
            result: None,
        }
    }
    pub(crate) fn action(&self) -> DeviceAction {
        match self.state {
            State::Header => DeviceAction::ReadHeader,
            State::Channel(index) => DeviceAction::ReadChannel { index },
            State::Fallback => DeviceAction::ComputeFallback,
            State::Commit => DeviceAction::CommitConversion,
            State::Done => DeviceAction::Done,
            State::Stopped => DeviceAction::Stopped,
        }
    }
    fn fail(&mut self, error: DeviceError) -> Result<(), DeviceError> {
        self.state = State::Stopped;
        Err(error)
    }
    pub(crate) fn step(&mut self, action: DeviceAction) -> Result<(), DeviceError> {
        if action != self.action() {
            return Err(DeviceError::WrongAction);
        }
        if self.expected != self.facts.identity {
            return Err(DeviceError::ForeignFacts);
        }
        match action {
            DeviceAction::ReadHeader => match self.facts.device {
                CssDeviceColorFact::Unknown => return self.fail(DeviceError::Unavailable),
                CssDeviceColorFact::Absent
                | CssDeviceColorFact::Unsupported
                | CssDeviceColorFact::Invalid
                | CssDeviceColorFact::WrongSpace => self.state = State::Fallback,
                CssDeviceColorFact::Resolved(record) => {
                    // Wrong-space knowledge is its explicit fact branch. An
                    // internally contradictory resolved record is not absence.
                    if record.channels.len() != 4
                        || record.target != Space::Lab
                        || record.value.space() != Space::Lab
                    {
                        return self.fail(DeviceError::InvalidFacts);
                    }
                    self.state = State::Channel(0);
                }
            },
            DeviceAction::ReadChannel { index } => {
                let CssDeviceColorFact::Resolved(record) = self.facts.device else {
                    return self.fail(DeviceError::InvalidFacts);
                };
                let Some(value) = record.channels.get(index).copied() else {
                    return self.fail(DeviceError::InvalidFacts);
                };
                if !value.is_finite() {
                    return self.fail(DeviceError::InvalidFacts);
                }
                // Actual computed device input, including source-defined
                // missing-to-zero conversion, qualifies this concrete result.
                if value.to_bits() != self.computed.channels[index].unwrap_or(0.0).to_bits() {
                    return self.fail(DeviceError::ConversionMismatch);
                }
                self.state = if index == 3 {
                    State::Commit
                } else {
                    State::Channel(index + 1)
                };
            }
            DeviceAction::ComputeFallback => {
                let value = match self.computed.naive() {
                    Ok(value) => value,
                    Err(error) => return self.fail(DeviceError::Numerical(error)),
                };
                self.result = Some(DeviceResult {
                    value,
                    facts: self.expected,
                    resource: None,
                    intent: None,
                    computed: self.computed,
                });
                self.state = State::Done;
            }
            DeviceAction::CommitConversion => {
                let CssDeviceColorFact::Resolved(record) = self.facts.device else {
                    return self.fail(DeviceError::InvalidFacts);
                };
                let value = match record.value.with_resolved_alpha(self.computed.alpha) {
                    Ok(value) => value,
                    Err(error) => return self.fail(DeviceError::Numerical(error)),
                };
                self.result = Some(DeviceResult {
                    value,
                    facts: self.expected,
                    resource: Some(record.profile),
                    intent: Some(record.intent),
                    computed: self.computed,
                });
                self.state = State::Done;
            }
            DeviceAction::Done | DeviceAction::Stopped => return Err(DeviceError::Stopped),
        }
        Ok(())
    }
    pub(crate) fn result(&self) -> Result<DeviceResult, DeviceError> {
        if !matches!(self.state, State::Done) {
            return Err(DeviceError::Stopped);
        }
        self.result.ok_or(DeviceError::Stopped)
    }
    pub(crate) fn abandon(&mut self) {
        self.state = State::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::super::facts::{
        CssColorProfileConversionRecord, CssColorPropertyFacts, CssColorSchemeFact,
    };
    use super::*;
    fn facts(device: CssDeviceColorFact<'_>) -> CssColorFacts<'_> {
        CssColorFacts::new(
            CssColorFactIdentity::new(1, 2),
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            device,
        )
    }
    fn finish(cursor: &mut DeviceResolution<'_>) -> Result<DeviceResult, DeviceError> {
        loop {
            if cursor.action() == DeviceAction::Done {
                return cursor.result();
            }
            cursor.step(cursor.action())?;
        }
    }
    #[test]
    fn computed_inks_are_distinct_from_known_naive_colorimetric_results() {
        let percentage = |value| Component {
            domain: Domain::Percentage,
            value,
        };
        let computed = ComputedCmyk::new(
            [
                Component::number(-1.0),
                percentage(50.0),
                Component::number(2.0),
                Component::number(0.25),
            ],
            Some(Component::MISSING),
            CssColorSyntax::Modern,
        )
        .unwrap();
        assert_eq!(
            computed.channels,
            [Some(0.0), Some(0.5), Some(1.0), Some(0.25)]
        );
        assert_eq!(computed.alpha, None);
        for device in [
            CssDeviceColorFact::Absent,
            CssDeviceColorFact::Unsupported,
            CssDeviceColorFact::Invalid,
            CssDeviceColorFact::WrongSpace,
        ] {
            let facts = facts(device);
            let result = finish(&mut DeviceResolution::start(
                &facts,
                facts.identity,
                computed,
            ))
            .unwrap();
            assert_eq!(result.computed, computed);
            assert_eq!(
                result.value.channels(),
                [Some(0.75), Some(0.375), Some(0.0)]
            );
            assert_eq!(result.value.alpha(), None);
            assert_eq!(result.resource, None);
        }
        let unknown = facts(CssDeviceColorFact::Unknown);
        assert_eq!(
            finish(&mut DeviceResolution::start(
                &unknown,
                unknown.identity,
                computed
            )),
            Err(DeviceError::Unavailable)
        );
        assert_eq!(
            ComputedCmyk::new([percentage(1.0); 4], None, CssColorSyntax::Legacy),
            Err(ChannelError::Grammar)
        );
    }
    #[test]
    fn applicable_lab_result_has_exact_input_and_resource_without_alpha_invention() {
        let computed = ComputedCmyk::new(
            [Component::number(0.25); 4],
            Some(Component::number(0.5)),
            CssColorSyntax::Modern,
        )
        .unwrap();
        let profile = CssColorFactIdentity::new(9, 8);
        let input = [0.25; 4];
        let lab = CssUsedColor::try_new(
            Space::Lab,
            [Some(45.06), Some(45.477), Some(35.459)],
            Some(1.0),
        )
        .unwrap();
        let record = CssColorProfileConversionRecord::new(
            profile,
            &input,
            crate::CssColorProfileRenderingIntent::RelativeColorimetric,
            Space::Lab,
            lab,
        );
        let facts = facts(CssDeviceColorFact::Resolved(record));
        let result = finish(&mut DeviceResolution::start(
            &facts,
            facts.identity,
            computed,
        ))
        .unwrap();
        assert_eq!(result.resource, Some(profile));
        assert_eq!(result.value.space(), Space::Lab);
        assert_eq!(result.value.channels(), lab.channels());
        assert_eq!(result.value.alpha(), Some(0.5));
        let wrong = ComputedCmyk {
            channels: [Some(0.5); 4],
            ..computed
        };
        assert_eq!(
            finish(&mut DeviceResolution::start(&facts, facts.identity, wrong)),
            Err(DeviceError::ConversionMismatch)
        );
    }
    #[test]
    fn foreign_and_cancelled_observations_cannot_turn_unknown_into_fallback() {
        let computed =
            ComputedCmyk::new([Component::MISSING; 4], None, CssColorSyntax::Modern).unwrap();
        let facts = facts(CssDeviceColorFact::Absent);
        let mut cursor = DeviceResolution::start(&facts, CssColorFactIdentity::new(1, 3), computed);
        assert_eq!(
            cursor.step(DeviceAction::ReadHeader),
            Err(DeviceError::ForeignFacts)
        );
        assert_eq!(cursor.action(), DeviceAction::ReadHeader);
        cursor.abandon();
        assert_eq!(cursor.action(), DeviceAction::Stopped);
        assert_eq!(cursor.result(), Err(DeviceError::Stopped));
    }
}
