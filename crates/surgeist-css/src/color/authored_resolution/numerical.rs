//! Authored-qualified checkpoints over the actual external numerical owner.

use super::{CssAuthoredColorCheckpoint, CssAuthoredColorOutcome, CssAuthoredColorStandardFailure};
use crate::color::bounded::numerical_layout;
use crate::color::numerical::{NumericalComputation, NumericalSerialization};
use crate::{
    CssColor, CssColorActionDescription, CssColorActionLayout, CssColorFactIdentity, CssColorFacts,
    CssColorOperationError, CssColorOperationOwner, CssColorProtocolError,
};
use std::mem::size_of;
use surgeist_color::{
    Color, Computation, DigitSlot, NumericalRequest, OperationOwner, Serialization,
    SerializationRequest, Transformation,
};

struct Context<'css, 'authored, 'facts> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
    standard_failure: Option<CssAuthoredColorStandardFailure>,
}
impl Context<'_, '_, '_> {
    fn require_current(&self) -> Result<(), CssColorProtocolError> {
        if self.owner.current_phase() == self.phase {
            Ok(())
        } else {
            Err(CssColorProtocolError::ForeignSlot)
        }
    }
}
/// A used value with its actual checked root, facts and optional original cause.
/// The standard-failure allocation moves into this context once.
pub struct CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts> {
    context: Context<'css, 'authored, 'facts>,
    value: Color,
}
impl<'css, 'authored, 'facts> CssAuthoredColorCheckpoint<'css, 'authored, 'facts> {
    pub const fn register_bytes() -> usize {
        size_of::<Self>()
    }
    /// Move only a completed Used outcome; a failure remains in its original slot.
    pub fn take_used(
        slot: &mut Option<Self>,
    ) -> Result<CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>, CssColorProtocolError>
    {
        let actual = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        if !actual.is_current() {
            return Err(CssColorProtocolError::ForeignSlot);
        }
        if !matches!(actual.outcome, CssAuthoredColorOutcome::Used(_)) {
            return Err(CssColorProtocolError::WrongAction);
        }
        let actual = slot.take().expect("shared preflight proved Used");
        let CssAuthoredColorOutcome::Used(value) = actual.outcome else {
            unreachable!()
        };
        Ok(CssAuthoredUsedColorCheckpoint {
            context: Context {
                owner: actual.owner,
                phase: actual.phase,
                root: actual.root,
                facts: actual.facts,
                standard_failure: None,
            },
            value,
        })
    }
    /// Only completed standard invalidity permits this explicit consumer policy.
    /// The actual cause/path moves into the returned context without a clone.
    pub fn fallback(
        slot: &mut Option<Self>,
        value: Color,
    ) -> Result<CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>, CssColorProtocolError>
    {
        let actual = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        if !actual.is_current() {
            return Err(CssColorProtocolError::ForeignSlot);
        }
        if !matches!(actual.outcome, CssAuthoredColorOutcome::StandardFailure(_)) {
            return Err(CssColorProtocolError::WrongAction);
        }
        let actual = slot
            .take()
            .expect("shared preflight proved StandardFailure");
        let CssAuthoredColorOutcome::StandardFailure(cause) = actual.outcome else {
            unreachable!()
        };
        Ok(CssAuthoredUsedColorCheckpoint {
            context: Context {
                owner: actual.owner,
                phase: actual.phase,
                root: actual.root,
                facts: actual.facts,
                standard_failure: Some(cause),
            },
            value,
        })
    }
}
impl<'css, 'authored, 'facts> CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts> {
    pub const fn register_bytes() -> usize {
        size_of::<Self>()
    }
    pub const fn value(&self) -> Color {
        self.value
    }
    pub const fn authored(&self) -> &'authored CssColor {
        self.context.root
    }
    pub const fn facts(&self) -> &'facts CssColorFacts<'facts> {
        self.context.facts
    }
    pub const fn fact_identity(&self) -> CssColorFactIdentity {
        self.context.facts.identity()
    }
    pub const fn standard_failure(&self) -> Option<&CssAuthoredColorStandardFailure> {
        self.context.standard_failure.as_ref()
    }
    pub fn is_current(&self) -> bool {
        self.context.require_current().is_ok()
    }
    pub fn transform<'num>(
        slot: &mut Option<Self>,
        numerical_owner: &'num mut OperationOwner,
        transformation: Transformation,
    ) -> Result<CssAuthoredColorTransformation<'css, 'num, 'authored, 'facts>, CssColorOperationError>
    {
        let actual = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        let next = actual.context.owner.prepare_advance(actual.context.phase)?;
        let numerical = Computation::new(
            numerical_owner,
            NumericalRequest::Transform {
                value: actual.value,
                transformation,
            },
        );
        actual
            .context
            .owner
            .commit_phase(actual.context.phase, next);
        let mut checkpoint = slot.take().expect("shared preflight kept checkpoint");
        checkpoint.context.phase = next;
        Ok(CssAuthoredColorTransformation {
            checkpoint,
            numerical: NumericalComputation::Pending(numerical),
        })
    }
    pub fn serialize<'num>(
        slot: &mut Option<Self>,
        numerical_owner: &'num mut OperationOwner,
        request: SerializationRequest,
    ) -> Result<CssAuthoredColorSerialization<'css, 'num, 'authored, 'facts>, CssColorOperationError>
    {
        let actual = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        let next = actual.context.owner.prepare_advance(actual.context.phase)?;
        let numerical = Serialization::new(numerical_owner, actual.value, request);
        actual
            .context
            .owner
            .commit_phase(actual.context.phase, next);
        let mut checkpoint = slot.take().expect("shared preflight kept checkpoint");
        checkpoint.context.phase = next;
        Ok(CssAuthoredColorSerialization {
            checkpoint,
            numerical: NumericalSerialization::new(numerical),
        })
    }
}
/// Fixed observations/copies and actual inline construction at numerical entry.
/// `register_bytes` includes context and native continuation once. The other
/// fields describe work, not additional register allocations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssAuthoredColorNumericalInitializationLayout {
    pub checkpoint_bytes: usize,
    pub css_owner_bytes: usize,
    pub numerical_owner_bytes: usize,
    pub register_bytes: usize,
}
/// Actual external numerical payloads and the CSS owner observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssAuthoredColorNumericalActionLayout {
    pub numerical: CssColorActionLayout,
    pub css_owner_read_bytes: usize,
    pub result_bytes: usize,
}
fn initialization(
    register_bytes: usize,
    checkpoint_bytes: usize,
) -> CssAuthoredColorNumericalInitializationLayout {
    CssAuthoredColorNumericalInitializationLayout {
        checkpoint_bytes,
        css_owner_bytes: size_of::<CssColorOperationOwner>(),
        numerical_owner_bytes: OperationOwner::layout_bytes(),
        register_bytes,
    }
}
fn action_layout(
    description: CssColorActionDescription,
    native: surgeist_color::ActionLayout,
) -> CssAuthoredColorNumericalActionLayout {
    let result_bytes = match description {
        CssColorActionDescription::AppendNumericalDigit { .. }
        | CssColorActionDescription::UpdateNumericalDigit { .. } => {
            size_of::<Result<DigitSlot<'_>, CssColorOperationError>>()
        }
        _ => size_of::<Result<(), CssColorOperationError>>(),
    };
    CssAuthoredColorNumericalActionLayout {
        numerical: numerical_layout(description, native),
        css_owner_read_bytes: size_of::<CssColorOperationOwner>(),
        result_bytes,
    }
}
/// Delegates one transformation; only successful take commits the new Color.
pub struct CssAuthoredColorTransformation<'css, 'num, 'authored, 'facts> {
    checkpoint: CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>,
    numerical: NumericalComputation<'num>,
}
impl<'css, 'num, 'authored, 'facts> CssAuthoredColorTransformation<'css, 'num, 'authored, 'facts> {
    pub const fn register_bytes() -> usize {
        size_of::<Self>()
    }
    pub const fn action_bytes() -> usize {
        size_of::<CssAuthoredColorTransformAction<'_, 'css, 'num, 'authored, 'facts>>()
    }
    pub fn initialization_layout() -> CssAuthoredColorNumericalInitializationLayout {
        initialization(
            Self::register_bytes(),
            CssAuthoredUsedColorCheckpoint::register_bytes(),
        )
    }
    pub fn action(&self) -> CssColorActionDescription {
        self.numerical.description()
    }
    pub fn next_action(
        &mut self,
    ) -> CssAuthoredColorTransformAction<'_, 'css, 'num, 'authored, 'facts> {
        CssAuthoredColorTransformAction {
            context: &self.checkpoint.context,
            numerical: &mut self.numerical,
        }
    }
    pub fn abandon(&mut self) {
        self.numerical.abandon();
    }
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>, CssColorOperationError>
    {
        let actual = slot.as_mut().ok_or(CssColorProtocolError::EmptySlot)?;
        actual.checkpoint.context.require_current()?;
        let value = actual.numerical.value()?;
        let mut actual = slot.take().expect("native take succeeded");
        actual.checkpoint.value = value;
        Ok(actual.checkpoint)
    }
    pub fn recover(
        slot: &mut Option<Self>,
    ) -> Result<CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>, CssColorProtocolError>
    {
        Ok(slot
            .take()
            .ok_or(CssColorProtocolError::EmptySlot)?
            .checkpoint)
    }
}
/// Lends the actual native numerical state and the independent prepared site.
pub struct CssAuthoredColorTransformAction<'step, 'css, 'num, 'authored, 'facts> {
    context: &'step Context<'css, 'authored, 'facts>,
    numerical: &'step mut NumericalComputation<'num>,
}
impl CssAuthoredColorTransformAction<'_, '_, '_, '_, '_> {
    pub fn description(&self) -> CssColorActionDescription {
        self.numerical.description()
    }
    pub fn layout(&self) -> CssAuthoredColorNumericalActionLayout {
        action_layout(self.description(), self.numerical.layout())
    }
    pub fn compute(self) -> Result<(), CssColorOperationError> {
        self.context.require_current()?;
        self.numerical.compute().map_err(Into::into)
    }
}
/// Resolved serialization owns no digit/output storage; the caller owns it.
pub struct CssAuthoredColorSerialization<'css, 'num, 'authored, 'facts> {
    checkpoint: CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>,
    numerical: NumericalSerialization<'num>,
}
impl<'css, 'num, 'authored, 'facts> CssAuthoredColorSerialization<'css, 'num, 'authored, 'facts> {
    pub const fn register_bytes() -> usize {
        size_of::<Self>()
    }
    pub const fn action_bytes() -> usize {
        size_of::<CssAuthoredColorSerializationAction<'_, 'css, 'num, 'authored, 'facts>>()
    }
    pub fn initialization_layout() -> CssAuthoredColorNumericalInitializationLayout {
        initialization(
            Self::register_bytes(),
            CssAuthoredUsedColorCheckpoint::register_bytes(),
        )
    }
    pub fn action(&self) -> CssColorActionDescription {
        self.numerical.description()
    }
    pub fn next_action(
        &mut self,
    ) -> CssAuthoredColorSerializationAction<'_, 'css, 'num, 'authored, 'facts> {
        CssAuthoredColorSerializationAction {
            context: &self.checkpoint.context,
            numerical: &mut self.numerical,
        }
    }
    pub fn abandon(&mut self) {
        self.numerical.abandon();
    }
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<CssAuthoredColorSerializationReceipt<'css, 'authored, 'facts>, CssColorOperationError>
    {
        let actual = slot.as_mut().ok_or(CssColorProtocolError::EmptySlot)?;
        actual.checkpoint.context.require_current()?;
        let length = actual.numerical.length()?;
        let actual = slot.take().expect("native take succeeded");
        Ok(CssAuthoredColorSerializationReceipt {
            checkpoint: actual.checkpoint,
            length,
        })
    }
    pub fn recover(
        slot: &mut Option<Self>,
    ) -> Result<CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>, CssColorProtocolError>
    {
        Ok(slot
            .take()
            .ok_or(CssColorProtocolError::EmptySlot)?
            .checkpoint)
    }
}
pub struct CssAuthoredColorSerializationAction<'step, 'css, 'num, 'authored, 'facts> {
    context: &'step Context<'css, 'authored, 'facts>,
    numerical: &'step mut NumericalSerialization<'num>,
}
impl<'num> CssAuthoredColorSerializationAction<'_, '_, 'num, '_, '_> {
    pub fn description(&self) -> CssColorActionDescription {
        self.numerical.description()
    }
    pub fn layout(&self) -> CssAuthoredColorNumericalActionLayout {
        action_layout(self.description(), self.numerical.layout())
    }
    pub fn compute(self) -> Result<(), CssColorOperationError> {
        self.context.require_current()?;
        self.numerical.compute().map_err(Into::into)
    }
    /// # Panics
    ///
    /// If `slot` was minted by another numerical operation or for another index.
    #[track_caller]
    pub fn read_numerical_digit(
        self,
        index: usize,
        slot: &DigitSlot<'_>,
    ) -> Result<(), CssColorOperationError> {
        self.context.require_current()?;
        self.numerical.read(index, slot).map_err(Into::into)
    }
    /// The record borrows the external numerical owner, beyond this guard and
    /// serializer. Recovering CSS provenance does not end that real borrow.
    ///
    /// ```compile_fail,E0499
    /// use surgeist_css::{CssAuthoredUsedColorCheckpoint, CssAuthoredColorSerialization,
    ///     CssColorActionDescription as A};
    /// use surgeist_color::{Computation, NumericalRequest, OperationOwner,
    ///     SerializationRequest, Transformation};
    /// fn retained_digit<'css, 'authored, 'facts>(
    ///     mut used: Option<CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>>,
    ///     owner: &mut OperationOwner,
    /// ) {
    ///     let mut serializer = Some(CssAuthoredUsedColorCheckpoint::serialize(
    ///         &mut used, owner, SerializationRequest::Css).unwrap());
    ///     let digit = loop {
    ///         let mut action = serializer.as_mut().unwrap().next_action();
    ///         match action.description() {
    ///             A::Compute(_) => action.compute().unwrap(),
    ///             A::EmitUnit { unit } => action.emitted(unit).unwrap(),
    ///             A::AppendNumericalDigit { .. } => break action.numerical_record().unwrap(),
    ///             _ => panic!("a value with coefficient digits"),
    ///         }
    ///     };
    ///     let checkpoint = CssAuthoredColorSerialization::recover(&mut serializer).unwrap();
    ///     drop(serializer);
    ///     let next = Computation::new(owner, NumericalRequest::Transform {
    ///         value: checkpoint.value(), transformation: Transformation::QuantizeSrgb,
    ///     });
    ///     drop(digit);
    ///     drop(next);
    /// }
    /// ```
    pub fn numerical_record(&mut self) -> Result<DigitSlot<'num>, CssColorOperationError> {
        self.context.require_current()?;
        self.numerical.record().map_err(Into::into)
    }
    /// # Panics
    ///
    /// If `slot` is not the record minted for the pending write.
    #[track_caller]
    pub fn numerical_written(
        self,
        index: usize,
        slot: &DigitSlot<'_>,
    ) -> Result<(), CssColorOperationError> {
        self.context.require_current()?;
        self.numerical.written(index, slot).map_err(Into::into)
    }
    pub fn emitted(self, unit: u8) -> Result<(), CssColorOperationError> {
        self.context.require_current()?;
        self.numerical.emitted(unit).map_err(Into::into)
    }
}
/// A successful receipt retains authored provenance, not the numerical borrow.
pub struct CssAuthoredColorSerializationReceipt<'css, 'authored, 'facts> {
    checkpoint: CssAuthoredUsedColorCheckpoint<'css, 'authored, 'facts>,
    length: usize,
}
impl<'authored, 'facts> CssAuthoredColorSerializationReceipt<'_, 'authored, 'facts> {
    pub const fn register_bytes() -> usize {
        size_of::<Self>()
    }
    pub const fn value(&self) -> Color {
        self.checkpoint.value()
    }
    pub const fn length(&self) -> usize {
        self.length
    }
    pub const fn authored(&self) -> &'authored CssColor {
        self.checkpoint.authored()
    }
    pub const fn facts(&self) -> &'facts CssColorFacts<'facts> {
        self.checkpoint.facts()
    }
    pub const fn fact_identity(&self) -> CssColorFactIdentity {
        self.checkpoint.fact_identity()
    }
    pub const fn standard_failure(&self) -> Option<&CssAuthoredColorStandardFailure> {
        self.checkpoint.standard_failure()
    }
    pub fn is_current(&self) -> bool {
        self.checkpoint.is_current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact, CssNamedColor};

    #[test]
    fn projections_and_observations_do_not_advance_phase_and_refusals_preserve_slots() {
        let root = CssColor::from_named(CssNamedColor::try_new("red").unwrap());
        let facts = CssColorFacts::new(
            CssColorFactIdentity::new(1, 2),
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Absent,
        );
        let mut css_owner = CssColorOperationOwner::new();
        assert_eq!(css_owner.begin().unwrap(), 1);
        // A valid private Used-state fixture: real checked red, matching facts,
        // exact numerical red, and the actual anchor's current phase.
        let mut resolved = Some(CssAuthoredColorCheckpoint {
            owner: &css_owner,
            phase: 1,
            root: &root,
            facts: &facts,
            outcome: CssAuthoredColorOutcome::Used(Color::from_srgba8([255, 0, 0, 255])),
        });
        assert!(matches!(
            CssAuthoredColorCheckpoint::fallback(&mut resolved, Color::opaque_black()),
            Err(CssColorProtocolError::WrongAction)
        ));
        assert_eq!(css_owner.current_phase(), 1);
        let mut used = Some(CssAuthoredColorCheckpoint::take_used(&mut resolved).unwrap());
        assert_eq!(used.as_ref().unwrap().context.phase, 1);
        assert_eq!(css_owner.current_phase(), 1);
        let mut numerical_owner = OperationOwner::new();
        // A foreign-phase negative fixture exercises the real shared preflight.
        used.as_mut().unwrap().context.phase = 2;
        assert!(matches!(
            CssAuthoredUsedColorCheckpoint::transform(
                &mut used,
                &mut numerical_owner,
                Transformation::ForceOpaqueAlpha,
            ),
            Err(CssColorOperationError::Protocol(
                CssColorProtocolError::ForeignSlot
            ))
        ));
        assert_eq!(used.as_ref().unwrap().context.phase, 2);
        assert_eq!(css_owner.current_phase(), 1);
        used.as_mut().unwrap().context.phase = 1;
        let mut transform = Some(
            CssAuthoredUsedColorCheckpoint::transform(
                &mut used,
                &mut numerical_owner,
                Transformation::QuantizeSrgb,
            )
            .unwrap(),
        );
        assert_eq!(css_owner.current_phase(), 2);
        assert!(CssAuthoredColorTransformation::take(&mut transform).is_err());
        assert_eq!(css_owner.current_phase(), 2);
        {
            let guard = transform.as_mut().unwrap().next_action();
            let _ = guard.description();
            let _ = guard.layout();
        }
        assert_eq!(css_owner.current_phase(), 2);
        transform.as_mut().unwrap().next_action().compute().unwrap();
        used = Some(CssAuthoredColorTransformation::take(&mut transform).unwrap());
        assert_eq!(used.as_ref().unwrap().context.phase, 2);
        assert_eq!(css_owner.current_phase(), 2);
        drop(transform);
        let mut serializer = Some(
            CssAuthoredUsedColorCheckpoint::serialize(
                &mut used,
                &mut numerical_owner,
                SerializationRequest::Css,
            )
            .unwrap(),
        );
        assert_eq!(css_owner.current_phase(), 3);
        assert!(matches!(
            serializer.as_mut().unwrap().next_action().emitted(b'x'),
            Err(CssColorOperationError::Protocol(
                CssColorProtocolError::WrongAction
            ))
        ));
        assert_eq!(css_owner.current_phase(), 3);
        let recovered = CssAuthoredColorSerialization::recover(&mut serializer).unwrap();
        assert_eq!(recovered.context.phase, 3);
        assert_eq!(css_owner.current_phase(), 3);
        assert_eq!(recovered.value().srgb_bytes(), Some([255, 0, 0]));
    }
}
