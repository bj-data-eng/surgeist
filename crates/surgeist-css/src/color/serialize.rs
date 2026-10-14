//! CSS qualification for the numerical resolved-color serializer.
//!
//! Source/fact checkpoints remain CSS-owned. This adapter checks the actual
//! anchor, phase, bank, kind and index before lending a digit to the numerical
//! continuation; it performs no numerical planning or formatting itself.

use super::CssColorSerializationRequest;
pub(crate) use super::numerical::serialization::SerializationAction;
use super::numerical::serialization::{
    SerializationCursor, SerializationError as NumericalSerializationError, SerializationKernel,
    SerializationProtocolError,
};
use super::protocol::{CssColorOperationOwner, CssColorProtocolError, CssColorSlot, Data};
use super::{CssUsedColor, CssUsedColorError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SerializationError {
    Protocol(CssColorProtocolError),
    Numerical(CssUsedColorError),
}
impl From<SerializationProtocolError> for CssColorProtocolError {
    fn from(error: SerializationProtocolError) -> Self {
        match error {
            SerializationProtocolError::WrongAction => Self::WrongAction,
            SerializationProtocolError::WrongIndex => Self::WrongIndex,
            SerializationProtocolError::Overflow => Self::Overflow,
            SerializationProtocolError::Terminal => Self::Terminal,
        }
    }
}
impl From<NumericalSerializationError> for SerializationError {
    fn from(error: NumericalSerializationError) -> Self {
        match error {
            NumericalSerializationError::Protocol(error) => Self::Protocol(error.into()),
            NumericalSerializationError::Numerical(error) => Self::Numerical(error),
        }
    }
}

/// Real CSS record qualification surrounding one numerical continuation.
pub(crate) struct UsedSerializer<'owner> {
    owner: &'owner CssColorOperationOwner,
    phase: usize,
    numerical: SerializationCursor,
}
impl<'owner> UsedSerializer<'owner> {
    pub(crate) fn start(
        owner: &'owner mut CssColorOperationOwner,
        value: CssUsedColor,
        request: CssColorSerializationRequest,
    ) -> Result<Self, CssColorProtocolError> {
        let phase = owner.begin()?;
        Ok(Self::start_phase(owner, phase, value, request))
    }
    pub(super) fn start_phase(
        owner: &'owner CssColorOperationOwner,
        phase: usize,
        value: CssUsedColor,
        request: CssColorSerializationRequest,
    ) -> Self {
        Self {
            owner,
            phase,
            numerical: SerializationCursor::new(value, request),
        }
    }
    pub(crate) fn action(&self) -> SerializationAction {
        self.numerical.action()
    }
    pub(crate) fn kernel(&self) -> super::CssColorKernel {
        match self.numerical.kernel() {
            SerializationKernel::Plan => super::CssColorKernel::SerializationPlan,
            SerializationKernel::Format => super::CssColorKernel::Format,
            SerializationKernel::Transition => super::CssColorKernel::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), SerializationError> {
        self.numerical.compute().map_err(Into::into)
    }
    fn digit(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<crate::finite_format::bounded::FormatDigit, CssColorProtocolError> {
        slot.qualify(
            self.owner,
            self.phase,
            index,
            super::protocol::CssColorBank::Algorithm,
        )?;
        let Data::FormatDigit(digit) = slot.data else {
            return Err(CssColorProtocolError::WrongSlot);
        };
        Ok(digit)
    }
    pub(crate) fn read_digit(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        if self.action() != (SerializationAction::ReadDigit { index }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        let digit = self.digit(index, slot)?;
        self.numerical.read_digit(index, digit).map_err(Into::into)
    }
    pub(crate) fn digit_record(
        &self,
        index: usize,
    ) -> Result<CssColorSlot<'owner>, CssColorProtocolError> {
        let digit = self.numerical.digit_record(index)?;
        Ok(CssColorSlot::new(
            self.owner,
            self.phase,
            index,
            Data::FormatDigit(digit),
        ))
    }
    pub(crate) fn digit_written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        let digit = self.digit(index, slot)?;
        self.numerical
            .digit_written(index, digit)
            .map_err(Into::into)
    }
    pub(crate) fn emitted(&mut self, unit: u8) -> Result<(), CssColorProtocolError> {
        self.numerical.emitted(unit).map_err(Into::into)
    }
    pub(crate) fn abandon(&mut self) {
        self.numerical.abandon();
    }
    pub(crate) fn completed_length(&self) -> Result<usize, CssColorProtocolError> {
        self.numerical.completed_length().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssColorInterpolationSpace as Space, CssPredefinedColorSpace as Predefined};
    fn serialize(value: CssUsedColor, request: CssColorSerializationRequest) -> String {
        let mut owner = CssColorOperationOwner::new();
        let mut serializer = UsedSerializer::start(&mut owner, value, request).unwrap();
        let mut digits = Vec::new();
        let mut output = Vec::new();
        loop {
            match serializer.action() {
                SerializationAction::Plan | SerializationAction::Compute => {
                    serializer.compute().unwrap()
                }
                SerializationAction::AppendDigit { index } => {
                    assert_eq!(index, digits.len());
                    digits.push(serializer.digit_record(index).unwrap());
                    serializer.digit_written(index, &digits[index]).unwrap();
                }
                SerializationAction::ReplaceDigit { index } => {
                    digits[index] = serializer.digit_record(index).unwrap();
                    serializer.digit_written(index, &digits[index]).unwrap();
                }
                SerializationAction::ReadDigit { index } => {
                    serializer.read_digit(index, &digits[index]).unwrap()
                }
                SerializationAction::Emit { unit } => {
                    output.push(unit);
                    serializer.emitted(unit).unwrap();
                }
                SerializationAction::Done => {
                    assert_eq!(serializer.completed_length().unwrap(), output.len());
                    break;
                }
                SerializationAction::Stopped => panic!("unexpected stop"),
            }
        }
        String::from_utf8(output).unwrap()
    }
    #[test]
    fn hex_requires_actual_byte_proof_and_html_request() {
        let value = CssUsedColor::from_srgba8([255, 0, 255, 255]);
        assert_eq!(
            serialize(value, CssColorSerializationRequest::HtmlCompatible),
            "#ff00ff"
        );
        assert_eq!(
            serialize(value, CssColorSerializationRequest::Css),
            "rgb(255, 0, 255)"
        );
        assert_eq!(
            serialize(
                value.with_color_function(),
                CssColorSerializationRequest::HtmlCompatible
            ),
            "#ff00ff"
        );
        let float = CssUsedColor::try_new(value.space(), value.channels(), value.alpha()).unwrap();
        assert_eq!(
            serialize(float, CssColorSerializationRequest::HtmlCompatible),
            "color(srgb 1 0 1)"
        );
    }
    #[test]
    fn legacy_byte_alpha_and_floating_alpha_remain_distinct() {
        let byte = CssUsedColor::from_srgba8([255, 0, 255, 237]);
        assert_eq!(
            serialize(byte, CssColorSerializationRequest::Css),
            "rgba(255, 0, 255, 0.93)"
        );
        assert_eq!(
            serialize(
                CssUsedColor::from_srgba8([0, 0, 0, 236]),
                CssColorSerializationRequest::Css
            ),
            "rgba(0, 0, 0, 0.925)"
        );
        let float = CssUsedColor::try_new(byte.space(), byte.channels(), byte.alpha()).unwrap();
        assert_eq!(
            serialize(float, CssColorSerializationRequest::Css),
            "color(srgb 1 0 1 / 0.929412)"
        );
    }
    #[test]
    fn missing_forms_extended_coordinates_and_alias_are_source_defined() {
        for (space, channels, alpha, expected) in [
            (
                Space::Hsl,
                [None, Some(50.0), Some(20.0)],
                None,
                "hsl(none 50% 20% / none)",
            ),
            (
                Space::Hwb,
                [Some(270.0), None, Some(2.0)],
                Some(1.0),
                "hwb(270 none 2%)",
            ),
            (
                Space::Lab,
                [Some(29.0), Some(39.0), Some(20.0)],
                Some(1.0),
                "lab(29 39 20)",
            ),
            (
                Space::Predefined(Predefined::DisplayP3),
                [Some(-0.2), Some(1.2), None],
                Some(0.5),
                "color(display-p3 -0.2 1.2 none / 0.5)",
            ),
            (
                Space::Predefined(Predefined::XyzD65),
                [Some(1.0), Some(2.0), Some(3.0)],
                Some(1.0),
                "color(xyz-d65 1 2 3)",
            ),
        ] {
            assert_eq!(
                serialize(
                    CssUsedColor::try_new(space, channels, alpha).unwrap(),
                    CssColorSerializationRequest::Css
                ),
                expected
            );
        }
    }
    #[test]
    fn refusal_and_foreign_digits_do_not_advance_or_emit() {
        let mut owner = CssColorOperationOwner::new();
        let mut foreign = CssColorOperationOwner::new();
        let value = CssUsedColor::from_srgba8([255, 0, 0, 255]);
        let mut serializer =
            UsedSerializer::start(&mut owner, value, CssColorSerializationRequest::Css).unwrap();
        let mut other =
            UsedSerializer::start(&mut foreign, value, CssColorSerializationRequest::Css).unwrap();
        let reach_append = |operation: &mut UsedSerializer<'_>| {
            loop {
                match operation.action() {
                    SerializationAction::Plan | SerializationAction::Compute => {
                        operation.compute().unwrap()
                    }
                    SerializationAction::Emit { unit } => operation.emitted(unit).unwrap(),
                    SerializationAction::AppendDigit { index } => return index,
                    _ => panic!("unexpected request"),
                }
            }
        };
        let index = reach_append(&mut serializer);
        let other_index = reach_append(&mut other);
        let pending = serializer.action();
        let foreign_slot = other.digit_record(other_index).unwrap();
        assert_eq!(
            serializer.digit_written(index, &foreign_slot),
            Err(CssColorProtocolError::ForeignSlot)
        );
        assert_eq!(serializer.action(), pending);
        {
            let _refused = serializer.digit_record(index).unwrap();
        }
        assert_eq!(serializer.action(), pending);
        serializer.abandon();
        assert_eq!(
            serializer.completed_length(),
            Err(CssColorProtocolError::Terminal)
        );
    }
}
