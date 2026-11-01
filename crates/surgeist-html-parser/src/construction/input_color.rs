//! HTML877d #update-a-color-well-control-color. The CSS owner supplies all
//! grammar and source qualification; Color supplies numerical work and formatting.
//! This adapter services finite actions through the original parser controller and private banks.
//! The stack anchor, slots and unpublished output never escape this operation.

#[cfg(test)]
use super::css_color::Banks;
use super::css_color::{action_layout, frame, numerical_layout};
use super::*;
use provenance::{ColorComputation, ColorOperandTable, ColorValueSource, InputValueComputation};
use surgeist_css::color::controlled::{
    CssColorOperationOwner, CssColorParseOutcome, CssColorTransform, CssUsedColorCheckpoint,
};
#[cfg(test)]
use surgeist_css::color::controlled::{CssColorParse, CssColorSlot};
use surgeist_css::color::{
    CssColorError, CssColorFactIdentity, CssColorFacts, CssColorPropertyFacts, CssColorSchemeFact,
    CssColorStandardFailure, CssDeviceColorFact,
};

use surgeist_color::{
    Color, ColorSpace, DigitSlot, OperationOwner, PredefinedColorSpace, Serialization,
    SerializationAction, SerializationRequest, SerializationStep, Transformation,
};

/// This identity qualifies only the fixed unavailable-environment policy below.
/// It is not a source issuer, resource observation, palette or DOM certificate.
pub(super) const fn environment_identity() -> CssColorFactIdentity {
    CssColorFactIdentity::new(0x4854_4d4c_434f_4c52, 1)
}
fn css_error(c: &mut ParseControl, error: CssColorError) -> SessionError {
    c.stop_color_processing(error).into()
}
fn integrity(c: &mut ParseControl) -> SessionError {
    c.fail_invariant(crate::parser::control::Invariant::InputCursorState)
        .into()
}
/// One total transformation; a refused admission drops it, which declines it.
fn transform<'f>(
    c: &mut ParseControl,
    checkpoint: CssUsedColorCheckpoint<'f>,
    numerical_owner: &mut OperationOwner,
    effect: Transformation,
) -> Result<CssUsedColorCheckpoint<'f>, SessionError> {
    frame(c, CssColorTransform::register_bytes(), 0)?;
    let transformation = checkpoint.transform(numerical_owner, effect);
    action_layout(c, transformation.layout())?;
    let checkpoint = transformation.compute();
    #[cfg(test)]
    tests::interrupt_at(tests::ColorCancelPoint::TransformCompute);
    frame(
        c,
        CssColorTransform::register_bytes(),
        mem::size_of::<CssUsedColorCheckpoint<'_>>(),
    )?;
    Ok(checkpoint)
}

/// Complete synchronous servicing; only final UTF16 output escapes. CSS phase
/// slots are reclaimed before transformation and before this stack anchor ends.
fn execute(
    c: &mut ParseControl,
    input: &[u16],
    alpha: bool,
    space: crate::HtmlInputColorSpace,
) -> Result<(Utf16Buffer, Color, Option<CssColorStandardFailure>), SessionError> {
    let facts = CssColorFacts::new(
        environment_identity(),
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    let mut numerical_owner = OperationOwner::new();
    let mut owner = CssColorOperationOwner::new();
    let mut output = None;
    let result = (|| {
        let parsed = css_color::parse(
            c,
            input,
            &mut owner,
            &facts,
            css_error,
            OperationOwner::layout_bytes(),
            #[cfg(test)]
            || tests::interrupt_at(tests::ColorCancelPoint::ParseSlotCommit),
        )?;
        let mut checkpoint = match parsed {
            CssColorParseOutcome::Used(value) => value,
            CssColorParseOutcome::StandardFailure(failure) => {
                let [red, green, blue] = crate::input_color_standard_failure_fallback().channels();
                failure.fallback(Color::from_srgba8([red, green, blue, 255]))
            }
        };
        let mut sequence = crate::HtmlInputColorSerialization::new(alpha, space);
        loop {
            payload_write::<crate::HtmlInputColorSerialization>(c, 1)?;
            c.charge(WorkCategory::StateStep, 1)?;
            let Some(effect) = sequence.next_effect() else {
                break;
            };
            let effect = match effect {
                crate::HtmlInputColorEffect::ForceOpaqueAlpha => Transformation::ForceOpaqueAlpha,
                crate::HtmlInputColorEffect::Convert(crate::HtmlInputColorSpace::LimitedSrgb) => {
                    Transformation::Convert(ColorSpace::Predefined(PredefinedColorSpace::Srgb))
                }
                crate::HtmlInputColorEffect::Convert(crate::HtmlInputColorSpace::DisplayP3) => {
                    Transformation::Convert(ColorSpace::Predefined(PredefinedColorSpace::DisplayP3))
                }
                crate::HtmlInputColorEffect::QuantizeSrgb => Transformation::QuantizeSrgb,
                crate::HtmlInputColorEffect::ColorFunction => Transformation::ColorFunction,
            };
            checkpoint = transform(c, checkpoint, &mut numerical_owner, effect)?;
        }
        // CSS does not wrap serialization: the checkpoint keeps its standard
        // failure for publication, and Color serializes its value directly.
        let failure = checkpoint.standard_failure();
        frame(
            c,
            Serialization::register_bytes(),
            const {
                mem::size_of::<Option<Utf16Buffer>>()
                    + mem::size_of::<Option<OwnedVec<DigitSlot<'_>>>>()
            },
        )?;
        let request = if sequence.html_compatible() {
            SerializationRequest::HtmlCompatible
        } else {
            SerializationRequest::Css
        };
        let mut digits = Some(OwnedVec::<DigitSlot<'_>>::new(
            HtmlParseResource::AuxiliaryEntries,
        )?);
        let mut serialization = Some(Serialization::new(
            &mut numerical_owner,
            checkpoint.value(),
            request,
        ));
        output = Some(Utf16Buffer::new(c)?);
        let serialized = (|| {
            loop {
                frame(
                    c,
                    Serialization::register_bytes(),
                    Serialization::action_bytes(),
                )?;
                let action = match serialization.take().ok_or_else(|| integrity(c))?.step() {
                    SerializationStep::Complete(receipt) => break Ok(receipt),
                    SerializationStep::Pending(action) => action,
                };
                // A refused admission drops the action, abandoning the serialization.
                numerical_layout(c, action.layout())?;
                serialization = Some(match action {
                    SerializationAction::Compute(compute) => compute.compute(),
                    SerializationAction::ReadDigit(read) => {
                        let bank = digits.as_mut().ok_or_else(|| integrity(c))?;
                        bank.require_owner(c)?;
                        let slot = bank
                            .as_slice()
                            .get(read.index())
                            .ok_or_else(|| integrity(c))?;
                        read.read(slot)
                    }
                    SerializationAction::AppendDigit(write) => {
                        let bank = digits.as_mut().ok_or_else(|| integrity(c))?;
                        if bank.len() != write.index() {
                            return Err(integrity(c));
                        }
                        bank.reserve_geometric(c, 1)?;
                        c.check_acquire_many(&[(HtmlParseResource::AuxiliaryEntries, 1)])?;
                        let (record, receipt) = write.record();
                        if let Err((error, _uncommitted)) = bank.try_push_recover(c, record) {
                            return Err(error.into());
                        }
                        let slot = bank.as_slice().last().ok_or_else(|| integrity(c))?;
                        let next = receipt.stored(slot);
                        #[cfg(test)]
                        tests::interrupt_at(tests::ColorCancelPoint::NumericalDigitCommit);
                        next
                    }
                    SerializationAction::ReplaceDigit(write) => {
                        let index = write.index();
                        let bank = digits.as_mut().ok_or_else(|| integrity(c))?;
                        let slot = bank.get_mut(index, c)?.ok_or_else(|| integrity(c))?;
                        let (record, receipt) = write.record();
                        *slot = record;
                        receipt.stored(slot)
                    }
                    SerializationAction::Emit(emit) => {
                        output
                            .as_mut()
                            .ok_or_else(|| integrity(c))?
                            .push_unit(u16::from(emit.unit()), c)?;
                        let next = emit.emitted();
                        #[cfg(test)]
                        if output.as_ref().is_some_and(|value| value.len() == 3) {
                            tests::interrupt_at(tests::ColorCancelPoint::ThirdOutputUnit);
                        }
                        next
                    }
                });
            }
        })();
        // An unfinished serialization is abandoned; it publishes nothing.
        if let Some(digits) = digits.take() {
            digits.release(c)?;
        }
        let receipt = serialized?;
        if receipt.length() != output.as_ref().ok_or_else(|| integrity(c))?.len() {
            return Err(integrity(c));
        }
        Ok((receipt.value(), failure))
    })();
    if result.is_err() {
        let output_disposal = output.take().map(|output| output.release(c)).transpose();
        output_disposal?;
        return Err(result.err().ok_or_else(|| integrity(c))?);
    }
    let (value, failure) = result?;
    Ok((output.take().ok_or_else(|| integrity(c))?, value, failure))
}

impl SessionStorage {
    /// Exact no-namespace installed attribute position, charged before each
    /// record/name observation. An unavailable field never becomes an absence.
    fn color_attribute(
        &self,
        c: &mut ParseControl,
        node: usize,
        name: &str,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(integrity(c));
        };
        for (position, attribute) in element.attributes().as_slice().iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            if attribute.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attribute.name().local_name()), name)?
            {
                return Ok(Some(position));
            }
        }
        Ok(None)
    }
    pub(super) fn update_input_color(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        slot: usize,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        let state = self.read_input_state(c, slot)?;
        if state.flags.input_type() != crate::HtmlInputType::Color {
            return Err(integrity(c));
        }
        let previous = self.input_raw_slot(c, slot)?;
        let dispose_previous = self.releasable_string(c, previous, false)?;
        let alpha_position = self.color_attribute(c, node, "alpha")?;
        let colorspace_position = self.color_attribute(c, node, "colorspace")?;
        let alpha_slot = alpha_position
            .map(|position| self.attribute_value_slot(c, node, position))
            .transpose()?;
        let colorspace_slot = colorspace_position
            .map(|position| self.attribute_value_slot(c, node, position))
            .transpose()?;
        let value_slot = match crate::input_color_value_source(state.flags.dirty_value()) {
            crate::HtmlInputColorValueSource::CurrentValue => Some(previous),
            crate::HtmlInputColorValueSource::ValueAttribute => self
                .color_attribute(c, node, "value")?
                .map(|position| self.attribute_value_slot(c, node, position))
                .transpose()?,
        };
        let source = match crate::input_color_value_source(state.flags.dirty_value()) {
            crate::HtmlInputColorValueSource::CurrentValue => ColorValueSource::CurrentValue,
            crate::HtmlInputColorValueSource::ValueAttribute if value_slot.is_some() => {
                ColorValueSource::InstalledValue
            }
            crate::HtmlInputColorValueSource::ValueAttribute => {
                ColorValueSource::MissingInstalledValue
            }
        };
        let original = value_slot
            .map(|slot| self.range_payload_edge(c, slot))
            .transpose()?;
        let alpha_edge = alpha_slot
            .map(|slot| self.range_payload_edge(c, slot))
            .transpose()?;
        let space_edge = colorspace_slot
            .map(|slot| self.range_payload_edge(c, slot))
            .transpose()?;
        let space = {
            let value = colorspace_slot
                .map(|slot| admitted_input_string(&self.payloads, c, slot))
                .transpose()?;
            let mut cursor = crate::HtmlInputColorSpaceCursor::new(value);
            while let Some(work) = cursor.required_work() {
                c.charge(WorkCategory::StateStep, 1)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<crate::HtmlInputColorSpaceCursor<'_>>() as u64,
                )?;
                payload_write::<crate::HtmlInputColorSpaceCursor<'_>>(c, 1)?;
                match work {
                    crate::HtmlInputRuleWork::CompareAsciiUnit => {
                        c.charge(WorkCategory::InspectByte, 3)?;
                    }
                    crate::HtmlInputRuleWork::ReadField => {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                    }
                    _ => return Err(integrity(c)),
                }
                cursor.advance();
            }
            cursor.result().ok_or_else(|| integrity(c))?
        };
        let input = value_slot
            .map(|slot| admitted_input_string(&self.payloads, c, slot))
            .transpose()?;
        let (output, value, failure) = execute(
            c,
            input.map_or(&[], DocumentString::code_units),
            alpha_slot.is_some(),
            space,
        )?;
        let units = output.len();
        let computation = InputValueComputation::Color(ColorComputation {
            source,
            original,
            operands: ColorOperandTable::new([alpha_edge, space_edge]),
            space,
            standard_failure: failure,
            environment: environment_identity(),
            value,
        });
        let group = match self.prepare_input_group(c, units, computation) {
            Ok(group) => group,
            Err(error) => {
                output.release(c)?;
                return Err(error);
            }
        };
        let frozen = output.freeze(c)?;
        let (native, next) = self.register_group_string_owned(c, frozen, Some(group))?;
        drop(native);
        payload_write::<usize>(c, 1)?;
        payload_write::<RetainedPayload>(c, 2)?;
        self.retire_field_group(c, previous)?;
        let Some(RetainedPayload::Input { raw, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(integrity(c));
        };
        *raw = next;
        if dispose_previous {
            self.dispose_aggregate_string(c, previous)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_digit_fault_is_a_color_conversion_invariant() {
        let mut c = ParseControl::new(color_limits(), crate::HtmlParseCancellation::new());
        let error = integrity(&mut c);
        let stop =
            ParseStop::InternalInvariant(crate::parser::control::Invariant::ColorConversionState);
        assert_eq!(error, SessionError::Stop(stop));
        assert_eq!(c.stop(), Some(stop));
    }

    // A thread-local observer of actual completed adapter actions. It changes
    // only the real request's cancellation flag. No production field, fee,
    // parser state, artificial tick, or second controller is introduced.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) enum ColorCancelPoint {
        ParseSlotCommit,
        TransformCompute,
        NumericalDigitCommit,
        ThirdOutputUnit,
    }
    struct TestCancellation {
        point: ColorCancelPoint,
        cancellation: crate::HtmlParseCancellation,
        reached: bool,
    }
    thread_local! {
        static CANCELLATION: std::cell::RefCell<Option<TestCancellation>> =
            const { std::cell::RefCell::new(None) };
    }
    pub(super) fn interrupt_at(point: ColorCancelPoint) {
        CANCELLATION.with_borrow_mut(|state| {
            if let Some(state) = state
                && state.point == point
                && !state.reached
            {
                state.reached = true;
                state.cancellation.cancel();
            }
        });
    }
    struct CancellationObservation;
    impl CancellationObservation {
        fn install(point: ColorCancelPoint, cancellation: crate::HtmlParseCancellation) -> Self {
            CANCELLATION.with_borrow_mut(|state| {
                assert!(state.is_none());
                *state = Some(TestCancellation {
                    point,
                    cancellation,
                    reached: false,
                });
            });
            Self
        }
        fn reached(&self) -> bool {
            CANCELLATION.with_borrow(|state| state.as_ref().is_some_and(|state| state.reached))
        }
    }
    impl Drop for CancellationObservation {
        fn drop(&mut self) {
            CANCELLATION.with_borrow_mut(|state| {
                *state = None;
            });
        }
    }
    // Same predeclared short-literal envelope as the public matrix: at most
    // four components,64 source cells and six distinct Color updates including
    // the managed alpha change/removal and subsequent reset below.
    fn color_limits() -> crate::HtmlParseLimits {
        let register = [
            CssColorParse::register_bytes(),
            Serialization::register_bytes(),
            CssColorTransform::register_bytes(),
        ]
        .into_iter()
        .max()
        .unwrap() as u64;
        // A transformation has no separate action: its frame charges only
        // the register and the resulting checkpoint.
        let guard = [
            CssColorParse::action_bytes(),
            Serialization::action_bytes(),
            mem::size_of::<CssUsedColorCheckpoint<'_>>(),
        ]
        .into_iter()
        .max()
        .unwrap() as u64;
        let cost = register
            .checked_mul(2)
            .unwrap()
            .checked_add(guard)
            .unwrap()
            .checked_add(
                (mem::size_of::<CssColorSlot<'_>>() as u64)
                    .checked_mul(16)
                    .unwrap(),
            )
            .unwrap()
            .checked_add(4096)
            .unwrap();
        let boundary = 1075_u64
            .checked_mul(2 * 1100 + 4)
            .unwrap()
            .checked_add(4 * 1100 + 2 * 17 + 8)
            .unwrap();
        let actions = 6_u64
            .checked_mul(
                4_u64
                    .checked_mul(64)
                    .unwrap()
                    .checked_mul(boundary)
                    .unwrap()
                    .checked_add(8_u64.checked_mul(1075).unwrap().checked_mul(1100).unwrap())
                    .unwrap()
                    .checked_add(4096)
                    .unwrap(),
            )
            .unwrap()
            .checked_mul(16)
            .unwrap();
        super::super::tests::limits()
            .into_builder()
            .work_credits(
                actions
                    .checked_mul(cost)
                    .unwrap()
                    .checked_add(100_000_000)
                    .unwrap(),
            )
            .finish()
            .unwrap()
    }
    fn fixture<'a>(
        request: &'a crate::HtmlDocumentParseRequest,
    ) -> (
        ParseControl,
        ParserArenaSession,
        crate::parser::tree_builder::ConstructionCapability<'a>,
        crate::parser::tokenizer::tests::LexerFixture<'a>,
        ParserNode,
    ) {
        let (mut c, mut session, cap, lexer, parent, mut tag, range) =
            super::super::tests::input_owner(request);
        let node = session
            .allocate_input_for_creation(
                &mut c,
                &mut tag,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &cap,
            )
            .unwrap();
        assert!(tag.is_none());
        (c, session, cap, lexer, node)
    }
    fn current(session: &ParserArenaSession, c: &mut ParseControl, node: &ParserNode) -> Vec<u16> {
        let mut output = Utf16Buffer::new(c).unwrap();
        session
            .append_input_internal_value(c, node, &mut output)
            .unwrap();
        let units = output.units().to_vec();
        output.release(c).unwrap();
        units
    }
    #[test]
    fn color_managed_clean_dirty_reset_and_clone_keep_actual_value_and_lineage() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><input type=color value=red><i alpha><i value=green>",
            ),
            color_limits(),
        );
        let (mut c, mut session, cap, mut lexer, node) = fixture(&request);
        let mut attr = super::super::tests::radio_next_attribute(&mut lexer, &mut c);
        session
            .set_input_attribute(&mut c, &node, &mut attr, &cap)
            .unwrap();
        assert!(attr.is_none());
        assert_eq!(
            current(&session, &mut c, &node),
            "color(srgb 1 0 0)".encode_utf16().collect::<Vec<_>>()
        );
        let (slot, mut state) = session
            .active()
            .unwrap()
            .radio_input(&mut c, node.index)
            .unwrap()
            .unwrap();
        state.flags = crate::HtmlInputFlags::from_retained(
            crate::HtmlInputType::Color,
            true,
            true,
            true,
            true,
            true,
        );
        session
            .active_mut()
            .unwrap()
            .write_input_state(&mut c, slot, state)
            .unwrap();
        let mut attr = super::super::tests::radio_next_attribute(&mut lexer, &mut c);
        session
            .set_input_attribute(&mut c, &node, &mut attr, &cap)
            .unwrap();
        assert_eq!(
            current(&session, &mut c, &node),
            "color(srgb 1 0 0)".encode_utf16().collect::<Vec<_>>()
        );
        session
            .remove_managed_attribute(&mut c, &node, "alpha", mutation::ManagedFamily::Input)
            .unwrap();
        assert_eq!(
            current(&session, &mut c, &node),
            "#ff0000".encode_utf16().collect::<Vec<_>>()
        );
        session
            .active_mut()
            .unwrap()
            .reset_input(&mut c, node.index, slot)
            .unwrap();
        assert_eq!(
            current(&session, &mut c, &node),
            "#008000".encode_utf16().collect::<Vec<_>>()
        );
        let reset = session.input_state(&mut c, &node).unwrap();
        assert!(!reset.flags.dirty_value());
        let copy = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .unwrap();
        let clone = session.active().unwrap().node(&mut c, copy).unwrap();
        assert_eq!(
            current(&session, &mut c, &clone),
            "#008000".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            session.input_state(&mut c, &clone).unwrap().flags,
            reset.flags
        );
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &clone, &mut output)
            .unwrap();
        assert_eq!(output.units(), "green".encode_utf16().collect::<Vec<_>>());
        output.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert!(c.stop().is_none());
    }
    #[test]
    fn color_type_removal_and_number_transition_keep_generated_source_whole_or_omitted() {
        for (suffix, expected_type, expected) in [
            ("", crate::HtmlInputType::Text, "#ff0000"),
            ("<i type=number>", crate::HtmlInputType::Number, ""),
            ("<i type=email>", crate::HtmlInputType::Email, "#ff0000"),
            ("<i type=file>", crate::HtmlInputType::File, ""),
        ] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!(
                    "<html><head></head><input type=color value=red>{suffix}"
                )),
                color_limits(),
            );
            let (mut c, mut session, cap, mut lexer, node) = fixture(&request);
            if suffix.is_empty() {
                session
                    .remove_managed_attribute(&mut c, &node, "type", mutation::ManagedFamily::Input)
                    .unwrap();
            } else {
                let mut attr = super::super::tests::radio_next_attribute(&mut lexer, &mut c);
                session
                    .set_input_attribute(&mut c, &node, &mut attr, &cap)
                    .unwrap();
                assert!(attr.is_none());
            }
            assert_eq!(
                session
                    .input_state(&mut c, &node)
                    .unwrap()
                    .flags
                    .input_type(),
                expected_type
            );
            assert_eq!(
                current(&session, &mut c, &node),
                expected.encode_utf16().collect::<Vec<_>>()
            );
            let copy = session
                .active_mut()
                .unwrap()
                .clone_node(&mut c, node.index, DocumentRef::Owned(0))
                .unwrap();
            let clone = session.active().unwrap().node(&mut c, copy).unwrap();
            assert_eq!(
                current(&session, &mut c, &clone),
                expected.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                session
                    .input_state(&mut c, &clone)
                    .unwrap()
                    .flags
                    .input_type(),
                expected_type
            );
            let mut output = Utf16Buffer::new(&mut c).unwrap();
            session
                .append_input_default_value(&mut c, &clone, &mut output)
                .unwrap();
            assert_eq!(output.units(), [114, 101, 100]);
            output.release(&mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
            assert!(c.stop().is_none());
        }
    }
    #[test]
    fn lexical_image_clone_preserves_current_default_and_reference_source() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=image value=A&#38;B>"),
            color_limits(),
        );
        let (mut c, mut session, _cap, lexer, node) = fixture(&request);
        let copy = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .unwrap();
        let clone = session.active().unwrap().node(&mut c, copy).unwrap();
        assert_eq!(current(&session, &mut c, &clone), [65, 38, 66]);
        assert_eq!(
            session
                .input_state(&mut c, &clone)
                .unwrap()
                .flags
                .input_type(),
            crate::HtmlInputType::Image
        );
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &clone, &mut output)
            .unwrap();
        assert_eq!(output.units(), [65, 38, 66]);
        output.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert!(c.stop().is_none());
    }
    #[test]
    fn generated_value_writeback_keeps_actual_native_name_value_history_and_refusal_owners() {
        for (kind, expected) in [("range", "50"), ("color", "#000000")] {
            for admitted in [true, false] {
                let limits = if admitted {
                    color_limits()
                } else {
                    color_limits()
                        .into_builder()
                        .managed_attributes(1)
                        .finish()
                        .unwrap()
                };
                let request = crate::HtmlDocumentParseRequest::offline(
                    crate::HtmlSource::utf8(format!(
                        "<html><head></head><input type={kind}><i type=image>"
                    )),
                    limits,
                );
                let (mut c, mut session, cap, mut lexer, node) = fixture(&request);
                assert_eq!(
                    current(&session, &mut c, &node),
                    expected.encode_utf16().collect::<Vec<_>>()
                );
                let mut attr = super::super::tests::radio_next_attribute(&mut lexer, &mut c);
                let result = session.set_input_attribute(&mut c, &node, &mut attr, &cap);
                if admitted {
                    result.unwrap();
                    assert!(attr.is_none());
                    let storage = session.active().unwrap();
                    let HtmlNodeKind::Element(element) = &storage
                        .construction
                        .node_at(node.index)
                        .expect("retained construction position")
                        .kind()
                    else {
                        panic!("actual element");
                    };
                    assert_eq!(element.attributes().len(), 2);
                    let value = &element.attributes().as_slice()[1];
                    assert_eq!(value.name().namespace().uri(), None);
                    assert_eq!(value.name().local_name(), &DocumentString::from("value"));
                    assert_eq!(value.value(), &DocumentString::from(expected));
                    assert_eq!(
                        current(&session, &mut c, &node),
                        expected.encode_utf16().collect::<Vec<_>>()
                    );
                    let copy = session
                        .active_mut()
                        .unwrap()
                        .clone_node(&mut c, node.index, DocumentRef::Owned(0))
                        .unwrap();
                    let clone = session.active().unwrap().node(&mut c, copy).unwrap();
                    assert_eq!(
                        current(&session, &mut c, &clone),
                        expected.encode_utf16().collect::<Vec<_>>()
                    );
                    assert_eq!(
                        session
                            .input_state(&mut c, &clone)
                            .unwrap()
                            .flags
                            .input_type(),
                        crate::HtmlInputType::Image
                    );
                    let eof = lexer
                        .next_in(
                            &mut c,
                            false,
                            session.active_mut().unwrap().evidence.as_mut().unwrap(),
                        )
                        .unwrap()
                        .unwrap();
                    assert!(
                        matches!(&eof, crate::parser::tokenizer::token::Event::Token(token)
                        if matches!(&token.token_for_tests().kind, TokenKind::Eof))
                    );
                    eof.release(&mut c).unwrap();
                    lexer.release(&mut c).unwrap();
                    session
                        .prepare_native_associations(&mut c, request.source())
                        .unwrap();
                    let history = session.freeze_history(&mut c).unwrap();
                    let view = history.view();
                    let mut generated_name = false;
                    for index in 0..view.len() {
                        let header = view.header(view.group(index).unwrap()).unwrap();
                        if matches!(&header.body, history::HistoryBody::Leaf(leaf) if leaf.units() == [118,97,108,117,101])
                            && header.units == 5
                            && header.cause.original_node == Some(node.index)
                            && header.cause.operation
                                == provenance::ConstructionOperation::Replacement
                        {
                            generated_name = true;
                            assert_eq!(
                                header.cause.source.as_ref().unwrap().source(),
                                request.source().identity()
                            );
                            let mut scratch = vec![history::HistoryFrame::default(); view.len()];
                            let mut cursor = view
                                .cursor(view.group(index).unwrap(), &mut scratch, None)
                                .unwrap();
                            let mut units = Vec::new();
                            loop {
                                match cursor.advance() {
                                    history::HistoryStep::Fault(fault) => panic!("{fault}"),
                                    history::HistoryStep::Leaf(leaf) => {
                                        assert!(leaf.origins().is_empty());
                                        units.extend_from_slice(leaf.units());
                                    }
                                    history::HistoryStep::Progress => {}
                                    history::HistoryStep::Complete => break,
                                    history::HistoryStep::Cancelled => panic!("no cancellation"),
                                }
                            }
                            assert_eq!(units, [118, 97, 108, 117, 101]);
                        }
                    }
                    assert!(generated_name);
                    history.release(&mut c).unwrap();
                } else {
                    assert!(matches!(
                        result,
                        Err(SessionError::Stop(ParseStop::LimitExceeded {
                            resource: HtmlParseResource::ManagedAttributes,
                            limit: 1,
                            current: 1,
                            requested_delta: 1
                        }))
                    ));
                    let first = c.stop();
                    let HtmlNodeKind::Element(element) = &session
                        .active()
                        .unwrap()
                        .construction
                        .node_at(node.index)
                        .expect("retained construction position")
                        .kind()
                    else {
                        panic!("actual element");
                    };
                    assert_eq!(element.attributes().len(), 1);
                    if let Some(attr) = attr.take() {
                        attr.name.release(&mut c).unwrap();
                        attr.value.release(&mut c).unwrap();
                    }
                    lexer.release(&mut c).unwrap();
                    session.release(&mut c).unwrap();
                    assert_eq!(c.stop(), first);
                }
                super::super::tests::storage_zero(&c);
                assert_eq!(c.usage().actual_capacity_bytes, 0);
            }
        }
    }
    #[test]
    fn color_protocol_red_uses_original_controller_and_reclaims_private_banks() {
        let limits = crate::parser::tree_builder::tests::limits(1000);
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        let (output, value, failure) = execute(
            &mut c,
            &[114, 101, 100],
            false,
            crate::HtmlInputColorSpace::LimitedSrgb,
        )
        .unwrap();
        assert_eq!(output.units(), &[35, 102, 102, 48, 48, 48, 48]);
        assert_eq!(value.srgb_bytes(), Some([255, 0, 0]));
        assert_eq!(failure, None);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        output.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert!(c.stop().is_none());
    }
    #[test]
    fn color_standard_invalid_uses_actual_failure_and_source_selected_serialization() {
        // HTML877d update steps 3–5: a standard-invalid CSS color becomes
        // opaque black, then uses the actual alpha/colorspace serialization.
        for (alpha, space, expected) in [
            (false, crate::HtmlInputColorSpace::LimitedSrgb, "#000000"),
            (
                true,
                crate::HtmlInputColorSpace::LimitedSrgb,
                "color(srgb 0 0 0)",
            ),
            (
                false,
                crate::HtmlInputColorSpace::DisplayP3,
                "color(display-p3 0 0 0)",
            ),
            (
                true,
                crate::HtmlInputColorSpace::DisplayP3,
                "color(display-p3 0 0 0)",
            ),
        ] {
            let mut c = ParseControl::new(color_limits(), crate::HtmlParseCancellation::new());
            let (output, value, failure) =
                execute(&mut c, &[103, 97, 114, 98, 97, 103, 101], alpha, space).unwrap();
            assert_eq!(
                output.units(),
                expected.encode_utf16().collect::<Vec<_>>(),
                "alpha={alpha}, space={space:?}"
            );
            assert_eq!(value.channels(), [Some(0.0); 3]);
            assert_eq!(value.alpha(), Some(1.0));
            assert_eq!(
                value.space(),
                ColorSpace::Predefined(match space {
                    crate::HtmlInputColorSpace::LimitedSrgb => PredefinedColorSpace::Srgb,
                    crate::HtmlInputColorSpace::DisplayP3 => PredefinedColorSpace::DisplayP3,
                })
            );
            assert!(failure.is_some(), "the actual standard failure is retained");
            assert!(c.stop().is_none());
            output.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
            assert_eq!(c.usage().actual_capacity_bytes, 0);
        }
    }
    #[test]
    fn color_overflowing_lab_axis_quantizes_its_closest_finite_srgb_and_reclaims_storage() {
        // Unlike CSS Lab lightness, a* is not clamped. Its cubic XYZ
        // reconstruction exceeds binary64; Color 4 conversion takes the
        // closest finite sRGB channels, signed by the D50 XYZ-to-sRGB X
        // column (+, -, +), which limited-sRGB quantization clamps to #ff00ff.
        let input: Vec<u16> = "lab(50 1e308 0)".encode_utf16().collect();
        let mut c = ParseControl::new(color_limits(), crate::HtmlParseCancellation::new());
        let (output, value, failure) = execute(
            &mut c,
            &input,
            false,
            crate::HtmlInputColorSpace::LimitedSrgb,
        )
        .unwrap();
        assert_eq!(output.units(), "#ff00ff".encode_utf16().collect::<Vec<_>>());
        assert_eq!(value.srgb_bytes(), Some([255, 0, 255]));
        assert_eq!(failure, None);
        assert!(c.stop().is_none());
        output.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn color_native_creation_installs_actual_default_and_generated_current() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=color value=red>"),
            super::super::tests::limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            super::super::tests::input_owner(&request);
        let result = session.allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        );
        let node = result.unwrap();
        let state = session.input_state(&mut c, &node).unwrap();
        assert_eq!(state.flags.input_type(), crate::HtmlInputType::Color);
        assert!(!state.flags.dirty_value());
        let mut value = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &node, &mut value)
            .unwrap();
        assert_eq!(value.units(), &[35, 102, 102, 48, 48, 48, 48]);
        value.clear(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &node, &mut value)
            .unwrap();
        assert_eq!(value.units(), &[114, 101, 100]);
        value.release(&mut c).unwrap();
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    #[test]
    fn unavailable_input_color_records_live_failed_prefix_before_private_bank_disposal() {
        let mut c = ParseControl::new(color_limits(), crate::HtmlParseCancellation::new());
        let raw: Vec<_> = "Canvas".encode_utf16().collect();
        assert!(execute(&mut c, &raw, false, crate::HtmlInputColorSpace::LimitedSrgb).is_err());
        assert!(matches!(
            c.stop(),
            Some(ParseStop::ColorProcessing(
                crate::parser::control::HtmlColorProcessingError::Unavailable(_)
            ))
        ));
        let stopped = c.first_stop_usage().expect("original failed action prefix");
        assert!(
            stopped
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current()
                > 0
        );
        assert!(
            stopped
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current()
                > 0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current(),
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current(),
            0
        );
    }
    #[test]
    fn color_initial_header_one_short_and_cancel_preserve_first_stop_before_input() {
        let layout = CssColorParse::initialization_layout();
        let guard = layout.owner_bytes
            + layout.fact_identity_bytes
            + mem::size_of::<CssColorFacts<'_>>()
            + mem::size_of::<Banks<'_>>()
            + mem::size_of::<OperationOwner>()
            + mem::size_of::<fn(&mut ParseControl, CssColorError) -> SessionError>();
        let first_header = (2 * layout.register_bytes + guard + 3) as u64;
        let limits = crate::parser::tree_builder::tests::limits(1000)
            .into_builder()
            .work_credits(first_header - 1)
            .finish()
            .unwrap();
        let cancellation = crate::HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits, cancellation.clone());
        assert!(
            execute(
                &mut c,
                &[114, 101, 100],
                false,
                crate::HtmlInputColorSpace::LimitedSrgb
            )
            .is_err()
        );
        let first = c.stop().unwrap();
        assert_eq!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: first_header - 1,
                current: first_header - 1,
                requested_delta: 1
            }
        );
        assert_eq!(c.usage().work(WorkCategory::InputUnitRead), 0);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        cancellation.cancel();
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        assert!(execute(&mut c, &[], false, crate::HtmlInputColorSpace::LimitedSrgb).is_err());
        assert_eq!(c.stop(), Some(first));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );

        let cancellation = crate::HtmlParseCancellation::new();
        cancellation.cancel();
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(1000),
            cancellation,
        );
        assert!(
            execute(
                &mut c,
                &[114, 101, 100],
                false,
                crate::HtmlInputColorSpace::LimitedSrgb
            )
            .is_err()
        );
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            0
        );
    }
    #[test]
    fn color_first_slot_refusal_disposes_prepared_banks_without_publishing_output() {
        let limits = crate::parser::tree_builder::tests::limits(1000)
            .into_builder()
            .auxiliary_entries(0)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        assert!(
            execute(
                &mut c,
                &[114, 101, 100],
                false,
                crate::HtmlInputColorSpace::LimitedSrgb
            )
            .is_err()
        );
        let first = c.stop().unwrap();
        assert_eq!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                limit: 0,
                current: 0,
                requested_delta: 1
            }
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert!(execute(&mut c, &[], false, crate::HtmlInputColorSpace::LimitedSrgb).is_err());
        assert_eq!(c.stop(), Some(first));
    }
    #[test]
    fn review_probe_color_image_text_restores_generated_native_default() {
        review_probe_image_text("color", "#000000");
    }
    #[test]
    fn review_probe_range_image_text_restores_generated_native_default() {
        review_probe_image_text("range", "50");
    }
    fn review_probe_image_text(kind: &str, expected: &str) {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!(
                "<html><head></head><input type={kind}><i type=image><i type=text>"
            )),
            color_limits(),
        );
        let (mut c, mut session, cap, mut lexer, node) = fixture(&request);
        let mut attr = super::super::tests::radio_next_attribute(&mut lexer, &mut c);
        session
            .set_input_attribute(&mut c, &node, &mut attr, &cap)
            .unwrap();
        assert_eq!(
            current(&session, &mut c, &node),
            expected.encode_utf16().collect::<Vec<_>>()
        );
        let mut attr = super::super::tests::radio_next_attribute(&mut lexer, &mut c);
        let result = session.set_input_attribute(&mut c, &node, &mut attr, &cap);
        eprintln!("{kind}->Image->Text: {result:?}; first stop {:?}", c.stop());
        if result.is_ok() {
            assert_eq!(
                current(&session, &mut c, &node),
                expected.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                session
                    .input_state(&mut c, &node)
                    .unwrap()
                    .flags
                    .input_type(),
                crate::HtmlInputType::Text
            );
            assert!(
                !session
                    .input_state(&mut c, &node)
                    .unwrap()
                    .flags
                    .dirty_value()
            );
        }
        if let Some(attr) = attr.take() {
            attr.name.release(&mut c).unwrap();
            attr.value.release(&mut c).unwrap();
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert!(
            result.is_ok(),
            "HTML877d Default->Value must restore actual generated native default: {result:?}"
        );
    }
    #[test]
    fn review_probe_dirty_color_clone_copies_current_without_resolving_default() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><input type=color value=red><i value=currentcolor>",
            ),
            color_limits(),
        );
        let (mut c, mut session, cap, mut lexer, node) = fixture(&request);
        let (slot, mut state) = session
            .active()
            .unwrap()
            .radio_input(&mut c, node.index)
            .unwrap()
            .unwrap();
        state.flags = crate::HtmlInputFlags::from_retained(
            crate::HtmlInputType::Color,
            true,
            false,
            false,
            false,
            false,
        );
        session
            .active_mut()
            .unwrap()
            .write_input_state(&mut c, slot, state)
            .unwrap();
        let mut attr = super::super::tests::radio_next_attribute(&mut lexer, &mut c);
        session
            .set_input_attribute(&mut c, &node, &mut attr, &cap)
            .unwrap();
        assert_eq!(
            current(&session, &mut c, &node),
            "#ff0000".encode_utf16().collect::<Vec<_>>()
        );
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &node, &mut output)
            .unwrap();
        assert_eq!(
            output.units(),
            "currentcolor".encode_utf16().collect::<Vec<_>>()
        );
        output.release(&mut c).unwrap();
        let result =
            session
                .active_mut()
                .unwrap()
                .clone_node(&mut c, node.index, DocumentRef::Owned(0));
        eprintln!(
            "dirty Color clone after default=currentcolor: {result:?}; first stop {:?}",
            c.stop()
        );
        if let Ok(copy) = result {
            let clone = session.active().unwrap().node(&mut c, copy).unwrap();
            assert_eq!(
                current(&session, &mut c, &clone),
                "#ff0000".encode_utf16().collect::<Vec<_>>()
            );
            assert!(
                session
                    .input_state(&mut c, &clone)
                    .unwrap()
                    .flags
                    .dirty_value()
            );
            let mut default = Utf16Buffer::new(&mut c).unwrap();
            session
                .append_input_default_value(&mut c, &clone, &mut default)
                .unwrap();
            assert_eq!(
                default.units(),
                "currentcolor".encode_utf16().collect::<Vec<_>>()
            );
            default.release(&mut c).unwrap();
            assert_eq!(
                session
                    .input_state(&mut c, &clone)
                    .unwrap()
                    .flags
                    .input_type(),
                crate::HtmlInputType::Color
            );
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert!(
            result.is_ok(),
            "HTML877d cloning copies current and dirty; unavailable default must not be resolved: {result:?}"
        );
    }

    #[test]
    fn color_output_exact_seven_and_one_short_retains_first_stop_and_disposes_partial_output() {
        // The input is borrowed and CSS slots use AuxiliaryEntries. This
        // BufferedUtf16Units cap therefore admits only the emitted #ff0000.
        for cap in [6, 7] {
            let cancellation = crate::HtmlParseCancellation::new();
            let limits = crate::parser::tree_builder::tests::limits(1000)
                .into_builder()
                .buffered_utf16_units(cap)
                .finish()
                .unwrap();
            let mut c = ParseControl::new(limits, cancellation.clone());
            let result = execute(
                &mut c,
                &[114, 101, 100],
                false,
                crate::HtmlInputColorSpace::LimitedSrgb,
            );
            if cap == 7 {
                let (output, _, failure) = result.unwrap();
                assert_eq!(output.units(), &[35, 102, 102, 48, 48, 48, 48]);
                assert_eq!(failure, None);
                assert!(c.stop().is_none());
                output.release(&mut c).unwrap();
            } else {
                assert!(result.is_err());
                let first = ParseStop::LimitExceeded {
                    resource: HtmlParseResource::BufferedUtf16Units,
                    limit: 6,
                    current: 6,
                    requested_delta: 1,
                };
                assert_eq!(c.stop(), Some(first));
                let at_stop = c.first_stop_usage().unwrap().clone();
                assert_eq!(
                    at_stop
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current,
                    6
                );
                assert!(at_stop.actual_capacity_bytes > 0);
                cancellation.cancel();
                assert!(
                    execute(&mut c, &[], false, crate::HtmlInputColorSpace::LimitedSrgb).is_err()
                );
                assert_eq!(c.stop(), Some(first));
                assert_eq!(c.first_stop_usage(), Some(&at_stop));
            }
            super::super::tests::storage_zero(&c);
            assert_eq!(c.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn color_second_bank_slot_refusal_keeps_real_committed_prefix_until_disposal() {
        let cancellation = crate::HtmlParseCancellation::new();
        let limits = crate::parser::tree_builder::tests::limits(1000)
            .into_builder()
            .auxiliary_entries(1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, cancellation.clone());
        assert!(
            execute(
                &mut c,
                &[114, 101, 100],
                false,
                crate::HtmlInputColorSpace::LimitedSrgb,
            )
            .is_err()
        );
        let first = ParseStop::LimitExceeded {
            resource: HtmlParseResource::AuxiliaryEntries,
            limit: 1,
            current: 1,
            requested_delta: 1,
        };
        assert_eq!(c.stop(), Some(first));
        let at_stop = c.first_stop_usage().unwrap().clone();
        assert_eq!(
            at_stop
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            1
        );
        assert!(at_stop.actual_capacity_bytes > 0);
        super::super::tests::storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        cancellation.cancel();
        assert!(execute(&mut c, &[], false, crate::HtmlInputColorSpace::LimitedSrgb).is_err());
        assert_eq!(c.stop(), Some(first));
        assert_eq!(c.first_stop_usage(), Some(&at_stop));
    }

    #[test]
    fn color_in_flight_cancel_at_real_slot_transform_and_partial_output_preserves_first_stop() {
        for point in [
            ColorCancelPoint::ParseSlotCommit,
            ColorCancelPoint::TransformCompute,
            ColorCancelPoint::NumericalDigitCommit,
            ColorCancelPoint::ThirdOutputUnit,
        ] {
            let cancellation = crate::HtmlParseCancellation::new();
            let observation = CancellationObservation::install(point, cancellation.clone());
            let mut c = ParseControl::new(
                crate::parser::tree_builder::tests::limits(1000),
                cancellation,
            );
            assert!(
                execute(
                    &mut c,
                    &[114, 101, 100],
                    false,
                    if point == ColorCancelPoint::NumericalDigitCommit {
                        crate::HtmlInputColorSpace::DisplayP3
                    } else {
                        crate::HtmlInputColorSpace::LimitedSrgb
                    },
                )
                .is_err()
            );
            assert!(
                observation.reached(),
                "actual source action must reach {point:?}"
            );
            assert_eq!(c.stop(), Some(ParseStop::Cancelled));
            let at_stop = c.first_stop_usage().unwrap().clone();
            match point {
                ColorCancelPoint::ParseSlotCommit => {
                    assert!(
                        at_stop
                            .resource(HtmlParseResource::AuxiliaryEntries)
                            .current
                            > 0
                    );
                    assert!(at_stop.actual_capacity_bytes > 0);
                    assert_eq!(
                        at_stop
                            .resource(HtmlParseResource::BufferedUtf16Units)
                            .current,
                        0
                    );
                }
                ColorCancelPoint::TransformCompute => {
                    // Parse banks were disposed before any HTML transform.
                    assert_eq!(
                        at_stop
                            .resource(HtmlParseResource::AuxiliaryEntries)
                            .current,
                        0
                    );
                    assert_eq!(
                        at_stop
                            .resource(HtmlParseResource::BufferedUtf16Units)
                            .current,
                        0
                    );
                    assert_eq!(at_stop.actual_capacity_bytes, 0);
                }
                ColorCancelPoint::NumericalDigitCommit => {
                    // The released CSS banks are empty; actual Color slots
                    // alone retain auxiliary storage at this checkpoint.
                    assert!(
                        at_stop
                            .resource(HtmlParseResource::AuxiliaryEntries)
                            .current
                            > 0
                    );
                    assert!(at_stop.actual_capacity_bytes > 0);
                }
                ColorCancelPoint::ThirdOutputUnit => {
                    assert_eq!(
                        at_stop
                            .resource(HtmlParseResource::BufferedUtf16Units)
                            .current,
                        3
                    );
                    assert!(at_stop.actual_capacity_bytes > 0);
                }
            }
            super::super::tests::storage_zero(&c);
            assert_eq!(c.usage().actual_capacity_bytes, 0);
            let stopped_work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            assert!(execute(&mut c, &[], false, crate::HtmlInputColorSpace::LimitedSrgb).is_err());
            assert_eq!(c.stop(), Some(ParseStop::Cancelled));
            assert_eq!(c.first_stop_usage(), Some(&at_stop));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                stopped_work
            );
        }
    }

    #[test]
    fn color_function_serialization_demand_is_pinned_for_work_credits_and_atomic_layout() {
        // Characterization of the complete red execution whose final phase
        // serializes `color(srgb 1 0 0)` through digit records. The values are
        // this tree's observed demand on a 64-bit target; a change in any
        // admitted frame or charge, including the serialization frames, fails.
        let cancellation = crate::HtmlParseCancellation::new();
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(1000),
            cancellation,
        );
        let (output, _, failure) = execute(
            &mut c,
            &[114, 101, 100],
            true,
            crate::HtmlInputColorSpace::LimitedSrgb,
        )
        .unwrap();
        assert_eq!(
            String::from_utf16(output.units()).unwrap(),
            "color(srgb 1 0 0)"
        );
        assert_eq!(failure, None);
        let usage = c.usage();
        let layout = usage.resource(HtmlParseResource::AtomicLayoutBytes);
        assert_eq!(
            (
                usage.resource(HtmlParseResource::WorkCredits).current,
                layout.current,
                layout.high_water,
            ),
            (446_718, 1440, 9472)
        );
        output.release(&mut c).unwrap();
    }
}
