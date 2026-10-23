//! Native literal-color adapter over the public CSS/Color operations.

use super::{
    BankKind, DomError, Formation, InputProcessing, InputValue, OutputBuffer, Payload, extent,
    metadata, output_payload, source_cleanup, source_marker, sum,
};
use crate::DocumentString;
use std::{mem::size_of, sync::atomic::AtomicBool};
use surgeist_color::{
    Color, ColorSpace, DigitSlot, OperationOwner, PredefinedColorSpace, SerializationRequest,
    Transformation,
};
use surgeist_css::{
    CssColorActionDescription as Action, CssColorActionLayout, CssColorBank, CssColorBankLengths,
    CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInputKind,
    CssColorOperation, CssColorOperationError, CssColorOperationOwner, CssColorParseOutcome,
    CssColorPropertyFacts, CssColorSchemeFact, CssColorSerialization, CssColorSlot,
    CssColorTransformation, CssDeviceColorFact, CssUsedColorCheckpoint,
};
use surgeist_html::{HtmlInputColorEffect, HtmlInputColorSerialization, HtmlInputColorSpace};

fn error(error: CssColorOperationError) -> DomError {
    use CssColorOperationError as E;
    match error {
        E::Protocol(value) => DomError::InputColorProtocol(value),
        E::Unavailable(value) => DomError::InputColorUnavailable(value),
        E::Unsupported(value) => DomError::InputColorUnsupported(value),
        E::Facts(value) => DomError::InputColorFacts(value),
    }
}

fn bank_index(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}

fn begin_action(
    meter: &mut InputProcessing,
    register: usize,
    guard: usize,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<usize, DomError> {
    // The register is observed and its acknowledged transition may copy it.
    meter.charge(sum(&[register, register])?, source, cancel)?;
    let fixed = sum(&[
        guard,
        size_of::<Action>(),
        size_of::<CssColorActionLayout>(),
    ])?;
    meter.fixed(fixed, fixed, source, cancel)?;
    Ok(fixed)
}

fn payload(
    meter: &mut InputProcessing,
    layout: CssColorActionLayout,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    meter.charge(
        sum(&[
            layout.input_unit_bytes,
            layout.slot_read_bytes,
            layout.slot_write_bytes,
            layout.numerical_read_bytes,
            layout.numerical_write_bytes,
            layout.output_unit_bytes,
            layout.fact_records[0],
            layout.fact_records[1],
            layout.fact_records[2],
        ])?,
        source,
        cancel,
    )
}

fn parse<'owner, 'facts>(
    meter: &mut InputProcessing,
    owner: &'owner mut CssColorOperationOwner,
    facts: &'facts CssColorFacts<'facts>,
    input: &[u16],
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<CssUsedColorCheckpoint<'owner, 'facts>, DomError> {
    let base = meter.live;
    let marker = source_marker(source);
    let fixed = sum(&[
        size_of::<Option<CssColorOperation<'_, '_>>>(),
        size_of::<[Vec<CssColorSlot<'_>>; 3]>(),
        size_of::<CssColorParseOutcome<'_, '_>>(),
    ])?;
    meter.fixed(fixed, fixed, source, cancel)?;
    #[cfg(test)]
    let mut written = crate::store::CopyObservation::default();
    let mut banks: [Vec<CssColorSlot<'_>>; 3] = std::array::from_fn(|_| Vec::new());
    let parsed = {
        let mut operation = Some(
            CssColorOperation::parse_used(
                owner,
                CssColorInputKind::Utf16,
                input.len(),
                CssColorGrammarRequest::WholeColor,
                facts,
                facts.identity(),
            )
            .map_err(error)?,
        );
        let parsed = (|| {
            loop {
                meter.poll(cancel)?;
                meter.inspect(source, cancel)?;
                let actual = operation
                    .as_mut()
                    .ok_or(DomError::InputProcessingProtocol)?;
                if matches!(actual.action(), Action::UsedReady | Action::StandardFailed) {
                    break;
                }
                let bytes = begin_action(
                    meter,
                    CssColorOperation::register_bytes(),
                    CssColorOperation::action_bytes(),
                    source,
                    cancel,
                )?;
                let serviced = (|| {
                    let mut action = actual.next_action();
                    payload(meter, action.layout(), source, cancel)?;
                    match action.description() {
                        Action::ReadInput { offset } => action
                            .read_input(
                                offset,
                                *input.get(offset).ok_or(DomError::InputProcessingProtocol)?,
                            )
                            .map_err(error),
                        Action::ReadSlot { bank, index } => action
                            .read_slot(
                                bank,
                                index,
                                banks[bank_index(bank)]
                                    .get(index)
                                    .ok_or(DomError::InputProcessingProtocol)?,
                            )
                            .map_err(error),
                        Action::AppendSlot { bank, index } => {
                            let bank_storage = &mut banks[bank_index(bank)];
                            if bank_storage.len() != index {
                                return Err(DomError::InputProcessingProtocol);
                            }
                            meter.reserve(
                                bank_storage,
                                index
                                    .checked_add(1)
                                    .ok_or(DomError::RepresentationExhausted)?,
                                BankKind::Bytes,
                                source,
                                cancel,
                            )?;
                            bank_storage.push(action.record().map_err(error)?);
                            #[cfg(test)]
                            written.pushed::<CssColorSlot<'_>>();
                            action
                                .written(bank, index, &bank_storage[index])
                                .map_err(error)?;
                            #[cfg(test)]
                            super::committed(super::Commit::ColorSlot, cancel);
                            Ok(())
                        }
                        Action::UpdateSlot { bank, index } => {
                            let stored = banks[bank_index(bank)]
                                .get_mut(index)
                                .ok_or(DomError::InputProcessingProtocol)?;
                            *stored = action.record().map_err(error)?;
                            action.written(bank, index, stored).map_err(error)
                        }
                        Action::Compute(_) | Action::ReadFact(_) => action.compute().map_err(error),
                        _ => Err(DomError::InputProcessingProtocol),
                    }
                })();
                meter.release(bytes, metadata(bytes), source);
                serviced?;
            }
            meter.charge(
                sum(&[1, size_of::<CssColorParseOutcome<'_, '_>>()])?,
                source,
                cancel,
            )?;
            match CssColorOperation::take(&mut operation).map_err(DomError::InputColorProtocol)? {
                CssColorParseOutcome::Used(value) => Ok(value),
                CssColorParseOutcome::StandardFailure(failure) => {
                    let [red, green, blue] =
                        surgeist_html::input_color_standard_failure_fallback().channels();
                    Ok(failure.fallback(Color::from_srgba8([red, green, blue, 255])))
                }
            }
        })();
        if parsed.is_err()
            && let Some(operation) = operation.as_mut()
        {
            operation.abandon();
        }
        parsed
    };
    // These records genuinely borrow the operation owner. Destroy them,
    // rather than relabelling headers, before entering numerical phases.
    drop(banks);
    meter.live = base;
    source_cleanup(source, marker);
    parsed
}

fn transform<'css, 'facts>(
    meter: &mut InputProcessing,
    checkpoint: &mut Option<CssUsedColorCheckpoint<'css, 'facts>>,
    owner: &mut OperationOwner,
    effect: Transformation,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let base = meter.live;
    let marker = source_marker(source);
    meter.fixed(
        size_of::<Option<CssColorTransformation<'_, '_, '_>>>(),
        size_of::<Option<CssColorTransformation<'_, '_, '_>>>(),
        source,
        cancel,
    )?;
    let result = (|| {
        let mut operation = Some(
            CssUsedColorCheckpoint::transform(
                checkpoint,
                owner,
                effect,
                CssColorBankLengths::new(0, 0, 0),
            )
            .map_err(error)?,
        );
        let transformed = (|| {
            loop {
                meter.poll(cancel)?;
                meter.inspect(source, cancel)?;
                let actual = operation
                    .as_mut()
                    .ok_or(DomError::InputProcessingProtocol)?;
                if actual.action() == Action::UsedReady {
                    break;
                }
                let bytes = begin_action(
                    meter,
                    CssColorTransformation::register_bytes(),
                    CssColorTransformation::action_bytes(),
                    source,
                    cancel,
                )?;
                let serviced = (|| {
                    let action = actual.next_action();
                    payload(meter, action.layout(), source, cancel)?;
                    match action.description() {
                        Action::Compute(_) => action.compute().map_err(error),
                        _ => Err(DomError::InputProcessingProtocol),
                    }
                })();
                meter.release(bytes, metadata(bytes), source);
                serviced?;
            }
            meter.charge(
                sum(&[1, CssColorTransformation::register_bytes()])?,
                source,
                cancel,
            )?;
            *checkpoint = Some(CssColorTransformation::take(&mut operation).map_err(error)?);
            Ok(())
        })();
        if transformed.is_err()
            && let Some(operation) = operation.as_mut()
        {
            operation.abandon();
        }
        transformed
    })();
    meter.live = base;
    source_cleanup(source, marker);
    result
}

fn serialize<'css, 'facts>(
    meter: &mut InputProcessing,
    checkpoint: &mut Option<CssUsedColorCheckpoint<'css, 'facts>>,
    owner: &mut OperationOwner,
    request: SerializationRequest,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<OutputBuffer, DomError> {
    let base = meter.live;
    let marker = source_marker(source);
    meter.fixed(
        sum(&[
            size_of::<Option<CssColorSerialization<'_, '_, '_>>>(),
            size_of::<Vec<DigitSlot<'_>>>(),
            size_of::<Vec<u16>>(),
            size_of::<surgeist_css::CssColorSerializationReceipt<'_, '_>>(),
        ])?,
        sum(&[
            size_of::<Option<CssColorSerialization<'_, '_, '_>>>(),
            size_of::<Vec<DigitSlot<'_>>>(),
            size_of::<Vec<u16>>(),
            size_of::<surgeist_css::CssColorSerializationReceipt<'_, '_>>(),
        ])?,
        source,
        cancel,
    )?;
    let result = (|| {
        #[cfg(test)]
        let mut written = crate::store::CopyObservation::default();
        let mut digits: Vec<DigitSlot<'_>> = Vec::new();
        let mut output = Vec::new();
        let serialized = {
            let mut operation = Some(
                CssUsedColorCheckpoint::serialize(
                    checkpoint,
                    owner,
                    request,
                    CssColorBankLengths::new(0, 0, 0),
                    0,
                )
                .map_err(error)?,
            );
            let serialized = (|| {
                loop {
                    meter.poll(cancel)?;
                    meter.inspect(source, cancel)?;
                    let actual = operation
                        .as_mut()
                        .ok_or(DomError::InputProcessingProtocol)?;
                    if actual.action() == Action::Serialized {
                        break;
                    }
                    let bytes = begin_action(
                        meter,
                        CssColorSerialization::register_bytes(),
                        CssColorSerialization::action_bytes(),
                        source,
                        cancel,
                    )?;
                    let serviced = (|| {
                        let mut action = actual.next_action();
                        payload(meter, action.layout(), source, cancel)?;
                        match action.description() {
                            Action::Compute(_) => action.compute().map_err(error),
                            Action::ReadNumericalDigit { index } => action
                                .read_numerical_digit(
                                    index,
                                    digits.get(index).ok_or(DomError::InputProcessingProtocol)?,
                                )
                                .map_err(error),
                            Action::AppendNumericalDigit { index } => {
                                if index != digits.len() {
                                    return Err(DomError::InputProcessingProtocol);
                                }
                                meter.reserve(
                                    &mut digits,
                                    index
                                        .checked_add(1)
                                        .ok_or(DomError::RepresentationExhausted)?,
                                    BankKind::Bytes,
                                    source,
                                    cancel,
                                )?;
                                digits.push(action.numerical_record().map_err(error)?);
                                #[cfg(test)]
                                written.pushed::<DigitSlot<'_>>();
                                action
                                    .numerical_written(index, &digits[index])
                                    .map_err(error)?;
                                #[cfg(test)]
                                super::committed(super::Commit::ColorDigit, cancel);
                                Ok(())
                            }
                            Action::UpdateNumericalDigit { index } => {
                                let stored = digits
                                    .get_mut(index)
                                    .ok_or(DomError::InputProcessingProtocol)?;
                                *stored = action.numerical_record().map_err(error)?;
                                action.numerical_written(index, stored).map_err(error)
                            }
                            Action::EmitUnit { unit } => {
                                let required = output
                                    .len()
                                    .checked_add(1)
                                    .ok_or(DomError::RepresentationExhausted)?;
                                meter.reserve(
                                    &mut output,
                                    required,
                                    BankKind::Utf16,
                                    source,
                                    cancel,
                                )?;
                                // Producer emission is a byte; actual stored output
                                // and its capacity are UTF16, at two bytes per unit.
                                meter.charge(size_of::<u16>(), source, cancel)?;
                                output.push(u16::from(unit));
                                #[cfg(test)]
                                written.pushed::<u16>();
                                action.emitted(unit).map_err(error)?;
                                #[cfg(test)]
                                super::committed(super::Commit::OutputUnit, cancel);
                                Ok(())
                            }
                            _ => Err(DomError::InputProcessingProtocol),
                        }
                    })();
                    meter.release(bytes, metadata(bytes), source);
                    serviced?;
                }
                meter.charge(
                    sum(&[
                        1,
                        size_of::<surgeist_css::CssColorSerializationReceipt<'_, '_>>(),
                    ])?,
                    source,
                    cancel,
                )?;
                let receipt = CssColorSerialization::take(&mut operation).map_err(error)?;
                if receipt.length() != output.len() {
                    return Err(DomError::InputProcessingProtocol);
                }
                Ok(())
            })();
            if serialized.is_err()
                && let Some(operation) = operation.as_mut()
            {
                operation.abandon();
            }
            serialized
        };
        drop(digits);
        serialized?;
        Ok(output)
    })();
    // Output owns its actual Vec header and body continuously. Split that
    // admitted credit before dropping the other stage registers/banks; never
    // reset/reacquire Source credit while a live successful output remains.
    let result = result.and_then(|units| {
        let payload = source
            .map(|source| output_payload(&units).map(|payload| source.split_payload(payload)))
            .transpose()?;
        Ok(OutputBuffer { units, payload })
    });
    meter.live = base;
    source_cleanup(source, marker);
    if let Ok(output) = &result {
        #[cfg(test)]
        super::source_tests::serialized_output(source, &output.units, cancel);
        // This reacquisition belongs only to the original child scratch
        // tariff. Source already owns the same live output through its Charge.
        meter.acquire(
            sum(&[
                extent(output.units.capacity(), size_of::<u16>())?,
                size_of::<Vec<u16>>(),
            ])?,
            Payload::default(),
            None,
            cancel,
        )?;
    }
    result
}

pub(super) fn run(
    meter: &mut InputProcessing,
    input: &DocumentString,
    alpha: bool,
    space: HtmlInputColorSpace,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<InputValue, DomError> {
    let initialization = CssColorOperation::initialization_layout();
    meter.fixed(
        sum(&[
            size_of::<InputProcessing>(),
            initialization.owner_bytes,
            initialization.fact_identity_bytes,
            OperationOwner::layout_bytes(),
            size_of::<CssColorFacts<'_>>(),
            size_of::<Option<CssUsedColorCheckpoint<'_, '_>>>(),
            size_of::<HtmlInputColorSerialization>(),
            size_of::<DocumentString>(),
        ])?,
        (sum(&[
            size_of::<InputProcessing>(),
            initialization.owner_bytes,
            initialization.fact_identity_bytes,
            OperationOwner::layout_bytes(),
            size_of::<CssColorFacts<'_>>(),
            size_of::<Option<CssUsedColorCheckpoint<'_, '_>>>(),
            size_of::<HtmlInputColorSerialization>(),
            size_of::<DocumentString>(),
        ])?)
        .checked_sub(size_of::<InputProcessing>())
        .expect("original inline ledger"),
        source,
        cancel,
    )?;
    // HTML's literal parse supplies no context element. This identity qualifies
    // only the explicit unavailable-environment policy, never a DOM certificate.
    let identity = CssColorFactIdentity::new(0x444f_4d5f_434f_4c52, 1);
    let facts = CssColorFacts::new(
        identity,
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    let mut css_owner = CssColorOperationOwner::new();
    let mut numerical_owner = OperationOwner::new();
    let mut checkpoint = Some(parse(
        meter,
        &mut css_owner,
        &facts,
        input.code_units(),
        source,
        cancel,
    )?);
    let mut sequence = HtmlInputColorSerialization::new(alpha, space);
    loop {
        meter.charge(
            sum(&[1, size_of::<HtmlInputColorSerialization>()])?,
            source,
            cancel,
        )?;
        let Some(effect) = sequence.next_effect() else {
            break;
        };
        let effect = match effect {
            HtmlInputColorEffect::ForceOpaqueAlpha => Transformation::ForceOpaqueAlpha,
            HtmlInputColorEffect::Convert(HtmlInputColorSpace::LimitedSrgb) => {
                Transformation::Convert(ColorSpace::Predefined(PredefinedColorSpace::Srgb))
            }
            HtmlInputColorEffect::Convert(HtmlInputColorSpace::DisplayP3) => {
                Transformation::Convert(ColorSpace::Predefined(PredefinedColorSpace::DisplayP3))
            }
            HtmlInputColorEffect::QuantizeSrgb => Transformation::QuantizeSrgb,
            HtmlInputColorEffect::ColorFunction => Transformation::ColorFunction,
        };
        transform(
            meter,
            &mut checkpoint,
            &mut numerical_owner,
            effect,
            source,
            cancel,
        )?;
    }
    let request = if sequence.html_compatible() {
        SerializationRequest::HtmlCompatible
    } else {
        SerializationRequest::Css
    };
    let output = serialize(
        meter,
        &mut checkpoint,
        &mut numerical_owner,
        request,
        source,
        cancel,
    )?;
    meter.output(output.units, output.payload, source, cancel)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_envelope {
        use crate as surgeist_dom;
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/input_processing.rs"
        ));
    }

    fn semantic_fixture() -> InputProcessing {
        assert!(size_of::<InputProcessing>() <= test_envelope::ledger_frame_bytes());
        let (scratch, work) = test_envelope::color_limits(1);
        InputProcessing::new(scratch, work)
    }

    #[test]
    fn color_standard_failure_and_real_colors_use_selected_alpha_space_serialization() {
        for (raw, alpha, space, expected) in [
            (
                "garbage",
                false,
                HtmlInputColorSpace::LimitedSrgb,
                "#000000",
            ),
            (
                "garbage",
                true,
                HtmlInputColorSpace::LimitedSrgb,
                "color(srgb 0 0 0)",
            ),
            (
                "garbage",
                false,
                HtmlInputColorSpace::DisplayP3,
                "color(display-p3 0 0 0)",
            ),
            (
                "garbage",
                true,
                HtmlInputColorSpace::DisplayP3,
                "color(display-p3 0 0 0)",
            ),
            ("red", false, HtmlInputColorSpace::LimitedSrgb, "#ff0000"),
            (
                "rgb(255 0 0 / .5)",
                true,
                HtmlInputColorSpace::LimitedSrgb,
                "color(srgb 1 0 0 / 0.5)",
            ),
            (
                "color(display-p3 1 0 0 / .5)",
                false,
                HtmlInputColorSpace::DisplayP3,
                "color(display-p3 1 0 0)",
            ),
            (
                "color(display-p3 1 0 0 / .5)",
                true,
                HtmlInputColorSpace::DisplayP3,
                "color(display-p3 1 0 0 / 0.5)",
            ),
        ] {
            let cancel = AtomicBool::new(false);
            let mut meter = semantic_fixture();
            assert!(raw.encode_utf16().count() <= 64);
            let actual = meter
                .color(&DocumentString::from(raw), alpha, space, None, &cancel)
                .map(|output| output.value);
            let actual = actual.unwrap_or_else(|error| {
                panic!(
                    "{raw:?}, alpha={alpha}, space={space:?}: {error:?}; reached work {}, peak {}",
                    meter.work, meter.peak
                )
            });
            assert_eq!(
                actual,
                DocumentString::from(expected),
                "{raw}, alpha={alpha}, {space:?}"
            );
            assert_eq!(meter.live, 0);
            assert!(meter.stopped.is_none());
        }
    }

    #[test]
    fn color_unavailable_remains_a_typed_error_and_an_overflowing_axis_is_closest_finite() {
        let cancel = AtomicBool::new(false);
        let mut meter = super::super::tests::fixture();
        assert_eq!(
            meter
                .color(
                    &DocumentString::from("currentcolor"),
                    false,
                    HtmlInputColorSpace::LimitedSrgb,
                    None,
                    &cancel
                )
                .map(|output| output.value),
            Err(DomError::InputColorUnavailable(
                surgeist_css::CssColorUnavailable::Palette(
                    surgeist_css::CssSystemColor::CanvasText,
                )
            ))
        );
        assert_eq!(meter.live, 0);

        // Lab a* is unclamped; its cubic XYZ exceeds binary64, so Color 4
        // conversion takes the closest finite sRGB channels, signed by the D50
        // XYZ-to-sRGB X column (+, -, +), and limited sRGB quantizes them.
        let mut meter = semantic_fixture();
        let raw = DocumentString::from("lab(50 1e308 0)");
        let actual = meter
            .color(&raw, false, HtmlInputColorSpace::LimitedSrgb, None, &cancel)
            .map(|output| output.value);
        assert_eq!(actual, Ok(DocumentString::from("#ff00ff")));
        assert_eq!(meter.live, 0);
    }

    #[test]
    fn color_decimal_work_refusal_cleans_actual_banks_and_preserves_first_error() {
        let cancel = AtomicBool::new(false);
        let mut meter = super::super::tests::fixture();
        // Retain the original independently chosen 2B ceiling. Exact progress
        // diagnosis showed this literal reaches it in the real decimal parse
        // after hundreds of thousands of advancing read/write requests and
        // actual bank growth, before transforms/serialization; this is not an
        // initial-header refusal or a synthetic action counter.
        let first = DomError::InputProcessingWorkLimit {
            limit: 2_000_000_000,
        };
        assert_eq!(
            meter
                .color(
                    &DocumentString::from("rgb(255 0 0 / .5)"),
                    true,
                    HtmlInputColorSpace::LimitedSrgb,
                    None,
                    &cancel
                )
                .map(|output| output.value),
            Err(first.clone())
        );
        assert_eq!(meter.live, 0);
        let stopped_work = meter.work;
        assert!(stopped_work > 1_000_000_000);
        assert!(meter.peak > CssColorOperation::register_bytes() + CssColorSlot::record_bytes());
        cancel.store(true, std::sync::atomic::Ordering::Release);
        assert_eq!(
            meter
                .color(
                    &DocumentString::from("red"),
                    false,
                    HtmlInputColorSpace::LimitedSrgb,
                    None,
                    &cancel
                )
                .map(|output| output.value),
            Err(first)
        );
        assert_eq!(meter.work, stopped_work);
        assert_eq!(meter.live, 0);
    }
}
