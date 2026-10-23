//! Native literal-color adapter over the public CSS/Color operations.

use super::{
    BankKind, DomError, Formation, InputProcessing, InputValue, OutputBuffer, Payload, extent,
    metadata, output_payload, source_cleanup, source_marker, sum,
};
use crate::DocumentString;
use std::{mem::size_of, sync::atomic::AtomicBool};
use surgeist_color::{
    ActionLayout, Color, ColorSpace, DigitSlot, OperationOwner, PredefinedColorSpace,
    Serialization, SerializationAction, SerializationReceipt, SerializationRequest,
    SerializationStep, Transformation,
};
use surgeist_css::{
    CssColorActionLayout, CssColorBank, CssColorError, CssColorFactIdentity, CssColorFacts,
    CssColorGrammarRequest, CssColorInputKind, CssColorOperationOwner, CssColorParse,
    CssColorParseAction, CssColorParseOutcome, CssColorParseStep, CssColorPropertyFacts,
    CssColorSchemeFact, CssColorSlot, CssColorTransform, CssDeviceColorFact,
    CssUsedColorCheckpoint,
};
use surgeist_html::{HtmlInputColorEffect, HtmlInputColorSerialization, HtmlInputColorSpace};

fn error(error: CssColorError) -> DomError {
    use CssColorError as E;
    match error {
        E::Unavailable(value) => DomError::InputColorUnavailable(value),
        E::Unsupported(value) => DomError::InputColorUnsupported(value),
        E::Facts(value) => DomError::InputColorFacts(value),
        // The adapter's UTF-16 input admits every unit, so a malformed-input
        // report is an adapter inconsistency.
        E::MalformedInput { .. } => DomError::InputProcessingProtocol,
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
    // A typed action is its own description; it is observed with its layout.
    meter.charge(sum(&[register, register])?, source, cancel)?;
    let fixed = sum(&[guard, size_of::<CssColorActionLayout>()])?;
    meter.fixed(fixed, fixed, source, cancel)?;
    Ok(fixed)
}

/// A Color action is its own description; it is observed with its layout.
fn begin_numerical_action(
    meter: &mut InputProcessing,
    register: usize,
    guard: usize,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<usize, DomError> {
    // The register is observed and its acknowledged transition may copy it.
    meter.charge(sum(&[register, register])?, source, cancel)?;
    let fixed = sum(&[guard, size_of::<ActionLayout>()])?;
    meter.fixed(fixed, fixed, source, cancel)?;
    Ok(fixed)
}

fn numerical_payload(
    meter: &mut InputProcessing,
    layout: ActionLayout,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    meter.charge(
        sum(&[layout.read_bytes, layout.write_bytes, layout.output_bytes])?,
        source,
        cancel,
    )
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

/// Services one typed parse action through the actual caller banks.
fn service<'owner, 'facts>(
    meter: &mut InputProcessing,
    action: CssColorParseAction<'owner, 'facts>,
    banks: &mut [Vec<CssColorSlot<'owner>>; 3],
    input: &[u16],
    source: Option<&Formation>,
    cancel: &AtomicBool,
    #[cfg(test)] written: &mut crate::store::CopyObservation,
) -> Result<CssColorParse<'owner, 'facts>, DomError> {
    payload(meter, action.layout(), source, cancel)?;
    match action {
        CssColorParseAction::ReadInput(read) => {
            let unit = *input
                .get(read.offset())
                .ok_or(DomError::InputProcessingProtocol)?;
            read.read(unit).map_err(error)
        }
        CssColorParseAction::ReadSlot(read) => {
            let slot = banks[bank_index(read.bank())]
                .get(read.index())
                .ok_or(DomError::InputProcessingProtocol)?;
            read.read(slot).map_err(error)
        }
        CssColorParseAction::AppendSlot(write) => {
            let index = write.index();
            let bank_storage = &mut banks[bank_index(write.bank())];
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
            let (record, receipt) = write.record();
            bank_storage.push(record);
            #[cfg(test)]
            written.pushed::<CssColorSlot<'_>>();
            let next = receipt.stored(&bank_storage[index]);
            #[cfg(test)]
            super::committed(super::Commit::ColorSlot, cancel);
            Ok(next)
        }
        CssColorParseAction::UpdateSlot(write) => {
            let index = write.index();
            let stored = banks[bank_index(write.bank())]
                .get_mut(index)
                .ok_or(DomError::InputProcessingProtocol)?;
            let (record, receipt) = write.record();
            *stored = record;
            Ok(receipt.stored(stored))
        }
        CssColorParseAction::ReadFact(read) => read.read().map_err(error),
        CssColorParseAction::Compute(compute) => compute.compute().map_err(error),
    }
}

fn parse<'facts>(
    meter: &mut InputProcessing,
    owner: &mut CssColorOperationOwner,
    facts: &'facts CssColorFacts<'facts>,
    input: &[u16],
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<CssUsedColorCheckpoint<'facts>, DomError> {
    let base = meter.live;
    let marker = source_marker(source);
    let fixed = sum(&[
        size_of::<CssColorParse<'_, '_>>(),
        size_of::<[Vec<CssColorSlot<'_>>; 3]>(),
        size_of::<CssColorParseOutcome<'_>>(),
    ])?;
    meter.fixed(fixed, fixed, source, cancel)?;
    #[cfg(test)]
    let mut written = crate::store::CopyObservation::default();
    let mut banks: [Vec<CssColorSlot<'_>>; 3] = std::array::from_fn(|_| Vec::new());
    let mut parse = CssColorParse::start(
        owner,
        CssColorInputKind::Utf16,
        input.len(),
        CssColorGrammarRequest::WholeColor,
        facts,
        facts.identity(),
    )
    .map_err(error)?;
    // A refusal drops the pending parse or action, which abandons the parse.
    let parsed = (|| {
        let outcome = loop {
            meter.poll(cancel)?;
            meter.inspect(source, cancel)?;
            let action = match parse.step() {
                CssColorParseStep::Complete(outcome) => break outcome,
                CssColorParseStep::Pending(action) => action,
            };
            let bytes = begin_action(
                meter,
                CssColorParse::register_bytes(),
                CssColorParse::action_bytes(),
                source,
                cancel,
            )?;
            let serviced = service(
                meter,
                action,
                &mut banks,
                input,
                source,
                cancel,
                #[cfg(test)]
                &mut written,
            );
            meter.release(bytes, metadata(bytes), source);
            parse = serviced?;
        };
        meter.charge(
            sum(&[1, size_of::<CssColorParseOutcome<'_>>()])?,
            source,
            cancel,
        )?;
        Ok(match outcome {
            CssColorParseOutcome::Used(value) => value,
            CssColorParseOutcome::StandardFailure(failure) => {
                let [red, green, blue] =
                    surgeist_html::input_color_standard_failure_fallback().channels();
                failure.fallback(Color::from_srgba8([red, green, blue, 255]))
            }
        })
    })();
    // These records genuinely borrow the operation owner. Destroy them,
    // rather than relabelling headers, before entering numerical phases.
    drop(banks);
    meter.live = base;
    source_cleanup(source, marker);
    parsed
}

/// One total transformation; a refused admission drops it, which declines it.
fn transform<'facts>(
    meter: &mut InputProcessing,
    checkpoint: CssUsedColorCheckpoint<'facts>,
    owner: &mut OperationOwner,
    effect: Transformation,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<CssUsedColorCheckpoint<'facts>, DomError> {
    let base = meter.live;
    let marker = source_marker(source);
    meter.fixed(
        CssColorTransform::register_bytes(),
        CssColorTransform::register_bytes(),
        source,
        cancel,
    )?;
    let result = (|| {
        meter.poll(cancel)?;
        meter.inspect(source, cancel)?;
        let transformation = checkpoint.transform(owner, effect);
        let bytes = begin_action(
            meter,
            CssColorTransform::register_bytes(),
            0,
            source,
            cancel,
        )?;
        let admitted = payload(meter, transformation.layout(), source, cancel);
        meter.release(bytes, metadata(bytes), source);
        admitted?;
        let checkpoint = transformation.compute();
        meter.charge(
            sum(&[1, CssColorTransform::register_bytes()])?,
            source,
            cancel,
        )?;
        Ok(checkpoint)
    })();
    meter.live = base;
    source_cleanup(source, marker);
    result
}

/// CSS does not wrap serialization: Color serializes the checkpoint's value.
fn serialize(
    meter: &mut InputProcessing,
    checkpoint: CssUsedColorCheckpoint<'_>,
    owner: &mut OperationOwner,
    request: SerializationRequest,
    source: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<OutputBuffer, DomError> {
    let base = meter.live;
    let marker = source_marker(source);
    meter.fixed(
        sum(&[
            size_of::<Option<Serialization<'_>>>(),
            size_of::<Vec<DigitSlot<'_>>>(),
            size_of::<Vec<u16>>(),
            size_of::<SerializationReceipt>(),
        ])?,
        sum(&[
            size_of::<Option<Serialization<'_>>>(),
            size_of::<Vec<DigitSlot<'_>>>(),
            size_of::<Vec<u16>>(),
            size_of::<SerializationReceipt>(),
        ])?,
        source,
        cancel,
    )?;
    let result = (|| {
        let value = checkpoint.value();
        #[cfg(test)]
        let mut written = crate::store::CopyObservation::default();
        let mut digits: Vec<DigitSlot<'_>> = Vec::new();
        let mut output = Vec::new();
        let serialized = {
            let mut serialization = Some(Serialization::new(owner, value, request));
            let serialized = (|| {
                let receipt = loop {
                    meter.poll(cancel)?;
                    meter.inspect(source, cancel)?;
                    let action = match serialization
                        .take()
                        .ok_or(DomError::InputProcessingProtocol)?
                        .step()
                    {
                        SerializationStep::Complete(receipt) => break receipt,
                        SerializationStep::Pending(action) => action,
                    };
                    let bytes = begin_numerical_action(
                        meter,
                        Serialization::register_bytes(),
                        Serialization::action_bytes(),
                        source,
                        cancel,
                    )?;
                    let serviced = (|| {
                        numerical_payload(meter, action.layout(), source, cancel)?;
                        Ok(match action {
                            SerializationAction::Compute(compute) => compute.compute(),
                            SerializationAction::ReadDigit(read) => {
                                let slot = digits
                                    .get(read.index())
                                    .ok_or(DomError::InputProcessingProtocol)?;
                                read.read(slot)
                            }
                            SerializationAction::AppendDigit(write) => {
                                let index = write.index();
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
                                let (record, receipt) = write.record();
                                digits.push(record);
                                #[cfg(test)]
                                written.pushed::<DigitSlot<'_>>();
                                let next = receipt.stored(&digits[index]);
                                #[cfg(test)]
                                super::committed(super::Commit::ColorDigit, cancel);
                                next
                            }
                            SerializationAction::ReplaceDigit(write) => {
                                let stored = digits
                                    .get_mut(write.index())
                                    .ok_or(DomError::InputProcessingProtocol)?;
                                let (record, receipt) = write.record();
                                *stored = record;
                                receipt.stored(stored)
                            }
                            SerializationAction::Emit(emit) => {
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
                                output.push(u16::from(emit.unit()));
                                #[cfg(test)]
                                written.pushed::<u16>();
                                let next = emit.emitted();
                                #[cfg(test)]
                                super::committed(super::Commit::OutputUnit, cancel);
                                next
                            }
                        })
                    })();
                    meter.release(bytes, metadata(bytes), source);
                    serialization = Some(serviced?);
                };
                meter.charge(
                    sum(&[1, size_of::<SerializationReceipt>()])?,
                    source,
                    cancel,
                )?;
                if receipt.length() != output.len() {
                    return Err(DomError::InputProcessingProtocol);
                }
                Ok(())
            })();
            // An unfinished serialization ends here, before its records go.
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
    let initialization = CssColorParse::initialization_layout();
    meter.fixed(
        sum(&[
            size_of::<InputProcessing>(),
            initialization.owner_bytes,
            initialization.fact_identity_bytes,
            OperationOwner::layout_bytes(),
            size_of::<CssColorFacts<'_>>(),
            size_of::<CssUsedColorCheckpoint<'_>>(),
            size_of::<HtmlInputColorSerialization>(),
            size_of::<DocumentString>(),
        ])?,
        (sum(&[
            size_of::<InputProcessing>(),
            initialization.owner_bytes,
            initialization.fact_identity_bytes,
            OperationOwner::layout_bytes(),
            size_of::<CssColorFacts<'_>>(),
            size_of::<CssUsedColorCheckpoint<'_>>(),
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
    let mut checkpoint = parse(
        meter,
        &mut css_owner,
        &facts,
        input.code_units(),
        source,
        cancel,
    )?;
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
        checkpoint = transform(
            meter,
            checkpoint,
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
        checkpoint,
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
        assert!(meter.peak > CssColorParse::register_bytes() + CssColorSlot::record_bytes());
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
