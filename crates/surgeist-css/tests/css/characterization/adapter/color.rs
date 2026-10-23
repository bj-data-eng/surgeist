//! Colour outcomes: ordinary used colours, authored resolution and the
//! controlled protocol with an HTML-style transformation sequence.

use surgeist_color::{
    Color, ColorSpace, DigitSlot, OperationOwner, PredefinedColorSpace, Serialization,
    SerializationAction, SerializationFamily, SerializationReceipt, SerializationRequest,
    SerializationStep, Transformation,
};
use surgeist_css::{
    CssAuthoredColorOrdinaryLimits, CssAuthoredColorOutcome, CssColorActionDescription as Action,
    CssColorBank, CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInput,
    CssColorInputKind, CssColorOperation, CssColorOperationError, CssColorOperationOwner,
    CssColorOrdinaryError, CssColorOrdinaryLimits, CssColorOrdinaryOutcome,
    CssColorOrdinaryResource, CssColorPaletteEntry, CssColorParseOutcome, CssColorPropertyFacts,
    CssColorSchemeFact, CssColorSlot, CssColorStandardFailure, CssColorStandardFailureKind,
    CssColorTransformation, CssColorUnavailable, CssColorUnsupported, CssDeviceColorFact,
    CssImportance, CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef,
    CssUsedColorCheckpoint, parse_property_value_text, parse_used_color, resolve_authored_color,
};

use super::{Observation, emitted, error_chain};

const IDENTITY: CssColorFactIdentity = CssColorFactIdentity::new(7, 3);
const PALETTE: &[CssColorPaletteEntry] = &[
    CssColorPaletteEntry::new(
        surgeist_css::CssSystemColor::CanvasText,
        CssColorFactIdentity::new(8, 4),
        Color::from_srgba8([16, 32, 48, 255]),
    ),
    CssColorPaletteEntry::new(
        surgeist_css::CssSystemColor::Canvas,
        CssColorFactIdentity::new(8, 5),
        Color::from_srgba8([250, 251, 252, 255]),
    ),
];

fn facts(set: &str) -> CssColorFacts<'static> {
    match set {
        "absent" => CssColorFacts::new(
            IDENTITY,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        ),
        "light" => CssColorFacts::new(
            IDENTITY,
            CssColorPropertyFacts::InitialProperties,
            CssColorSchemeFact::Light,
            PALETTE,
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        ),
        "current" => CssColorFacts::new(
            IDENTITY,
            CssColorPropertyFacts::Current(Color::from_srgba8([0, 128, 0, 200])),
            CssColorSchemeFact::Dark,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        ),
        other => panic!("unknown colour fact set `{other}`"),
    }
}

const ORDINARY_LIMITS: CssColorOrdinaryLimits = CssColorOrdinaryLimits {
    input_units: 4096,
    algorithm_records: 65_536,
    arithmetic_records: 16_384,
    evaluation_records: 16_384,
    actions: 16_000_000,
};

const AUTHORED_LIMITS: CssAuthoredColorOrdinaryLimits = CssAuthoredColorOrdinaryLimits {
    path_steps: 4096,
    algorithm_records: 65_536,
    arithmetic_records: 16_384,
    evaluation_records: 16_384,
    actions: 16_000_000,
};

pub fn observe(front: &str, set: &str, input: &str) -> Observation {
    let facts = facts(set);
    let units = input.encode_utf16().collect::<Vec<_>>();
    match front {
        "color.used.utf8.whole" => used(
            CssColorInput::Utf8(input.as_bytes()),
            CssColorGrammarRequest::WholeColor,
            &facts,
        ),
        "color.used.utf16.whole" => used(
            CssColorInput::Utf16(&units),
            CssColorGrammarRequest::WholeColor,
            &facts,
        ),
        "color.used.utf8.prefix" => used(
            CssColorInput::Utf8(input.as_bytes()),
            CssColorGrammarRequest::PrefixColor,
            &facts,
        ),
        "color.used.utf16.prefix" => used(
            CssColorInput::Utf16(&units),
            CssColorGrammarRequest::PrefixColor,
            &facts,
        ),
        "color.authored" => authored(input, &facts, AUTHORED_LIMITS),
        // The overflow test's 100M-action allowance, so the ordinary path
        // reaches its numerical outcome instead of the action limit.
        "color.used.large-actions" => used_with(
            CssColorInput::Utf8(input.as_bytes()),
            CssColorGrammarRequest::WholeColor,
            &facts,
            CssColorOrdinaryLimits {
                actions: 100_000_000,
                ..ORDINARY_LIMITS
            },
        ),
        "color.authored.large-actions" => authored(
            input,
            &facts,
            CssAuthoredColorOrdinaryLimits {
                actions: 100_000_000,
                ..AUTHORED_LIMITS
            },
        ),
        "color.protocol.html-srgb" => protocol(
            &units,
            &facts,
            &[
                Transformation::ForceOpaqueAlpha,
                Transformation::Convert(ColorSpace::Predefined(PredefinedColorSpace::Srgb)),
                Transformation::QuantizeSrgb,
            ],
            SerializationRequest::HtmlCompatible,
        ),
        "color.protocol.quantize-p3" => protocol(
            &units,
            &facts,
            &[
                Transformation::Convert(ColorSpace::Predefined(PredefinedColorSpace::DisplayP3)),
                Transformation::QuantizeSrgb,
            ],
            SerializationRequest::HtmlCompatible,
        ),
        "color.limits" => used_limits(input.as_bytes(), &facts),
        "color.protocol.css-p3" => protocol(
            &units,
            &facts,
            &[
                Transformation::Convert(ColorSpace::Predefined(PredefinedColorSpace::DisplayP3)),
                Transformation::ColorFunction,
            ],
            SerializationRequest::Css,
        ),
        other => panic!("unknown colour front `{other}`"),
    }
}

/// `parse_used_color` under the fixed ordinary limits. The controlled
/// protocol rows record the charged work: action counts and bank lengths.
fn used(
    input: CssColorInput<'_>,
    request: CssColorGrammarRequest,
    facts: &CssColorFacts<'_>,
) -> Observation {
    used_with(input, request, facts, ORDINARY_LIMITS)
}

fn used_with(
    input: CssColorInput<'_>,
    request: CssColorGrammarRequest,
    facts: &CssColorFacts<'_>,
    limits: CssColorOrdinaryLimits,
) -> Observation {
    let mut observation = Observation::default();
    match parse_used_color(input, request, facts, facts.identity(), limits) {
        Ok(result) => {
            let consumed = result.source().consumed;
            match result.outcome() {
                CssColorOrdinaryOutcome::Used(color) => {
                    observation.shape = "used".to_owned();
                    observation.add_fact(color_bits(color));
                }
                CssColorOrdinaryOutcome::StandardFailure(failure) => {
                    observation.shape = "standard-failure".to_owned();
                    observation.add_fact(standard_failure(failure));
                }
            }
            observation.add_fact(format!("consumed={}..{}", consumed.start, consumed.end));
        }
        Err(error) => {
            observation.shape = "err".to_owned();
            observation.add_fact(ordinary_error(&error));
        }
    }
    observation
}

/// The smallest ordinary limit of each resource at which `parse_used_color`
/// keeps its outcome, every other limit at the fixed value. CSS has no
/// public default ordinary colour limits, so these rows show where each
/// limit starts to refuse.
fn used_limits(input: &[u8], facts: &CssColorFacts<'_>) -> Observation {
    let observe = |limits: CssColorOrdinaryLimits| {
        let mut observation = Observation::default();
        match parse_used_color(
            CssColorInput::Utf8(input),
            CssColorGrammarRequest::WholeColor,
            facts,
            facts.identity(),
            limits,
        ) {
            Ok(result) => match result.outcome() {
                CssColorOrdinaryOutcome::Used(color) => {
                    observation.shape = "used".to_owned();
                    observation.add_fact(color_bits(color));
                }
                CssColorOrdinaryOutcome::StandardFailure(failure) => {
                    observation.shape = "standard-failure".to_owned();
                    observation.add_fact(standard_failure(failure));
                }
            },
            Err(error) => {
                observation.shape = "err".to_owned();
                observation.add_fact(ordinary_error(&error));
            }
        }
        observation
    };
    let mut observation = observe(ORDINARY_LIMITS);
    let fields: [(&str, OrdinaryLimit); 5] = [
        ("units", |limits| &mut limits.input_units),
        ("algorithm", |limits| &mut limits.algorithm_records),
        ("arithmetic", |limits| &mut limits.arithmetic_records),
        ("evaluation", |limits| &mut limits.evaluation_records),
        ("actions", |limits| &mut limits.actions),
    ];
    let mut parts = Vec::new();
    for (name, field) in fields {
        let mut maximum = ORDINARY_LIMITS;
        let value = super::smallest(*field(&mut maximum), |value| {
            let mut limits = ORDINARY_LIMITS;
            *field(&mut limits) = value;
            observe(limits) == observation
        });
        parts.push(format!("{name}={}", super::amount(value)));
    }
    observation.add_charge(format!("color:{}", parts.join(",")));
    observation
}

/// One resource of the ordinary colour limits.
type OrdinaryLimit = fn(&mut CssColorOrdinaryLimits) -> &mut usize;

/// Authored resolution of the colour retained by `color: <input>`.
fn authored(
    input: &str,
    facts: &CssColorFacts<'_>,
    limits: CssAuthoredColorOrdinaryLimits,
) -> Observation {
    let report = parse_property_value_text(
        input,
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        CssImportance::Normal,
    );
    let mut observation = Observation::default();
    observation.report(&report);
    let Some(declaration) = report.syntax() else {
        observation.shape = "parse:none".to_owned();
        return observation;
    };
    let color = match declaration.known().and_then(|known| known.property_value()) {
        Some(CssKnownPropertyValueRef::Color(value)) => value.value(),
        _ => {
            observation.shape = "parse:not-typed".to_owned();
            return observation;
        }
    };
    match resolve_authored_color(color, facts, facts.identity(), limits) {
        Ok(result) => match result.outcome() {
            CssAuthoredColorOutcome::Used(color) => {
                observation.shape = "used".to_owned();
                observation.add_fact(color_bits(*color));
            }
            CssAuthoredColorOutcome::StandardFailure(failure) => {
                observation.shape = "standard-failure".to_owned();
                observation.add_fact(format!(
                    "failure={} path-steps={} origin={}",
                    failure_kind(failure.kind),
                    failure.path.steps().len(),
                    if failure.origin.is_some() {
                        "parsed"
                    } else {
                        "-"
                    }
                ));
            }
        },
        Err(error) => {
            observation.shape = "err".to_owned();
            observation.add_fact(ordinary_error(&error));
        }
    }
    observation
}

#[derive(Default)]
struct ActionCounts {
    read_input: usize,
    read_slot: usize,
    append_slot: usize,
    update_slot: usize,
    compute: usize,
    read_fact: usize,
}

/// Services every parse action until the operation is ready or failed.
fn drive_parse<'owner>(
    operation: &mut CssColorOperation<'owner, '_>,
    input: &[u16],
    banks: &mut [Vec<CssColorSlot<'owner>>; 3],
    counts: &mut ActionCounts,
) -> Result<(), CssColorOperationError> {
    while !matches!(
        operation.action(),
        Action::UsedReady | Action::StandardFailed
    ) {
        let mut action = operation.next_action();
        match action.description() {
            Action::ReadInput { offset } => {
                counts.read_input += 1;
                action.read_input(offset, input[offset])?;
            }
            Action::ReadSlot { bank, index } => {
                counts.read_slot += 1;
                action.read_slot(bank, index, &banks[ordinal(bank)][index])?;
            }
            Action::AppendSlot { bank, index } => {
                counts.append_slot += 1;
                assert_eq!(index, banks[ordinal(bank)].len(), "append at the bank end");
                let record = action.record()?;
                banks[ordinal(bank)].push(record);
                action.written(bank, index, &banks[ordinal(bank)][index])?;
            }
            Action::UpdateSlot { bank, index } => {
                counts.update_slot += 1;
                banks[ordinal(bank)][index] = action.record()?;
                action.written(bank, index, &banks[ordinal(bank)][index])?;
            }
            Action::Compute(_) => {
                counts.compute += 1;
                action.compute()?;
            }
            Action::ReadFact(_) => {
                counts.read_fact += 1;
                action.compute()?;
            }
            _ => panic!("unexpected colour parse action"),
        }
    }
    Ok(())
}

/// Services every transformation action; returns the number computed.
fn drive_transformation(
    transformation: &mut CssColorTransformation<'_, '_, '_>,
) -> Result<usize, CssColorOperationError> {
    let mut computed = 0;
    while transformation.action() != Action::UsedReady {
        computed += 1;
        transformation.next_action().compute()?;
    }
    Ok(computed)
}

/// Services every action of Color's serialization of the checkpoint value,
/// collecting the emitted text and counting actions by kind.
fn drive_serialization<'num>(
    mut serialization: Serialization<'num>,
    digits: &mut Vec<DigitSlot<'num>>,
    text: &mut String,
    stream: &mut [usize; 5],
) -> SerializationReceipt {
    loop {
        let action = match serialization.step() {
            SerializationStep::Complete(receipt) => return receipt,
            SerializationStep::Pending(action) => action,
        };
        serialization = match action {
            SerializationAction::Compute(compute) => {
                stream[0] += 1;
                compute.compute()
            }
            SerializationAction::ReadDigit(read) => {
                stream[1] += 1;
                let slot = &digits[read.index()];
                read.read(slot)
            }
            SerializationAction::AppendDigit(write) => {
                stream[2] += 1;
                assert_eq!(write.index(), digits.len(), "append at the digit end");
                let (record, receipt) = write.record();
                digits.push(record);
                receipt.stored(&digits[digits.len() - 1])
            }
            SerializationAction::ReplaceDigit(write) => {
                stream[3] += 1;
                let index = write.index();
                let (record, receipt) = write.record();
                digits[index] = record;
                receipt.stored(&digits[index])
            }
            SerializationAction::Emit(emit) => {
                stream[4] += 1;
                text.push(char::from(emit.unit()));
                emit.emitted()
            }
        };
    }
}

/// The controlled protocol over UTF-16 input: parse actions by kind, final
/// bank lengths, then each transformation and the serialization stream.
fn protocol(
    input: &[u16],
    facts: &CssColorFacts<'_>,
    transformations: &[Transformation],
    request: SerializationRequest,
) -> Observation {
    let mut observation = Observation::default();
    let mut owner = CssColorOperationOwner::new();
    let mut counts = ActionCounts::default();
    let mut banks: [Vec<CssColorSlot<'_>>; 3] = std::array::from_fn(|_| Vec::new());
    let mut operation = match CssColorOperation::parse_used(
        &mut owner,
        CssColorInputKind::Utf16,
        input.len(),
        CssColorGrammarRequest::WholeColor,
        facts,
        facts.identity(),
    ) {
        Ok(operation) => operation,
        Err(error) => {
            observation.shape = "err".to_owned();
            observation.add_fact(format!("start-error={}", error_chain(&error)));
            return observation;
        }
    };
    let driven = drive_parse(&mut operation, input, &mut banks, &mut counts);
    observation.add_fact(format!(
        "parse-actions:read-input={},read-slot={},append-slot={},update-slot={},compute={},read-fact={}",
        counts.read_input,
        counts.read_slot,
        counts.append_slot,
        counts.update_slot,
        counts.compute,
        counts.read_fact
    ));
    if let Err(error) = driven {
        operation.abandon();
        observation.shape = "err".to_owned();
        observation.add_fact(format!("parse-error={}", error_chain(&error)));
        return observation;
    }
    let lengths = [
        CssColorBank::Algorithm,
        CssColorBank::Arithmetic,
        CssColorBank::Evaluation,
    ]
    .map(|bank| operation.bank_length(bank));
    observation.add_fact(format!(
        "banks=algorithm:{},arithmetic:{},evaluation:{}",
        lengths[0], lengths[1], lengths[2]
    ));
    let outcome = match CssColorOperation::take(&mut Some(operation)) {
        Ok(outcome) => outcome,
        Err(error) => {
            observation.shape = "err".to_owned();
            observation.add_fact(format!("take-error={}", error_chain(&error)));
            return observation;
        }
    };
    let checkpoint = match outcome {
        CssColorParseOutcome::Used(checkpoint) => {
            observation.shape = "used".to_owned();
            checkpoint
        }
        CssColorParseOutcome::StandardFailure(failure) => {
            observation.shape = "standard-failure".to_owned();
            observation.add_fact(standard_failure(failure.failure()));
            failure.fallback(Color::from_srgba8([0, 0, 0, 255]))
        }
    };
    observation.add_fact(format!("parsed={}", color_bits(checkpoint.value())));
    banks.iter_mut().for_each(Vec::clear);
    let mut numerical_owner = OperationOwner::new();
    let mut slot = Some(checkpoint);
    for transformation in transformations {
        let name = transformation_name(*transformation);
        let mut actual = match CssUsedColorCheckpoint::transform(
            &mut slot,
            &mut numerical_owner,
            *transformation,
        ) {
            Ok(actual) => actual,
            Err(error) => {
                observation.shape = "err".to_owned();
                observation.add_fact(format!("{name}-error={}", error_chain(&error)));
                return observation;
            }
        };
        let computed = match drive_transformation(&mut actual) {
            Ok(computed) => computed,
            Err(error) => {
                actual.abandon();
                observation.shape = "err".to_owned();
                observation.add_fact(format!("{name}-error={}", error_chain(&error)));
                return observation;
            }
        };
        match CssColorTransformation::take(&mut Some(actual)) {
            Ok(value) => {
                observation.add_fact(format!(
                    "{name}:computes={computed} -> {}",
                    color_bits(value.value())
                ));
                slot = Some(value);
            }
            Err(error) => {
                observation.shape = "err".to_owned();
                observation.add_fact(format!("{name}-error={}", error_chain(&error)));
                return observation;
            }
        }
    }
    // CSS does not wrap serialization: the consumer serializes the checkpoint
    // value with Color and keeps the checkpoint's standard failure.
    let checkpoint = slot.expect("the last transformation returned its checkpoint");
    let failure = checkpoint.standard_failure();
    let mut digits = Vec::new();
    let mut text = String::new();
    let mut stream = [0usize; 5];
    let receipt = drive_serialization(
        Serialization::new(&mut numerical_owner, checkpoint.value(), request),
        &mut digits,
        &mut text,
        &mut stream,
    );
    observation.add_fact(format!(
        "serialize-actions:compute={},read-digit={},append-digit={},update-digit={},emit={}",
        stream[0], stream[1], stream[2], stream[3], stream[4]
    ));
    observation.specified = emitted(text);
    observation.add_fact(format!(
        "receipt:length={} value={} failure={}",
        receipt.length(),
        color_bits(receipt.value()),
        failure.map_or_else(|| "-".to_owned(), standard_failure)
    ));
    observation
}

fn ordinal(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}

/// Space, channel and alpha bits, serialization family and sRGB bytes.
fn color_bits(color: Color) -> String {
    let bits = |value: Option<f64>| {
        value.map_or_else(
            || "none".to_owned(),
            |value| format!("{:016x}", value.to_bits()),
        )
    };
    let channels = color.channels().map(bits).join(",");
    format!(
        "space={} channels={channels} alpha={} family={} srgb={}",
        space_name(color.space()),
        bits(color.alpha()),
        family_name(color.serialization_family()),
        color.srgb_bytes().map_or_else(
            || "-".to_owned(),
            |[red, green, blue]| format!("{red:02x}{green:02x}{blue:02x}")
        )
    )
}

fn space_name(space: ColorSpace) -> &'static str {
    match space {
        ColorSpace::Hsl => "hsl",
        ColorSpace::Hwb => "hwb",
        ColorSpace::Lab => "lab",
        ColorSpace::Lch => "lch",
        ColorSpace::Oklab => "oklab",
        ColorSpace::Oklch => "oklch",
        ColorSpace::Predefined(space) => match space {
            PredefinedColorSpace::Srgb => "srgb",
            PredefinedColorSpace::SrgbLinear => "srgb-linear",
            PredefinedColorSpace::DisplayP3 => "display-p3",
            PredefinedColorSpace::DisplayP3Linear => "display-p3-linear",
            PredefinedColorSpace::A98Rgb => "a98-rgb",
            PredefinedColorSpace::ProphotoRgb => "prophoto-rgb",
            PredefinedColorSpace::Rec2020 => "rec2020",
            PredefinedColorSpace::XyzD50 => "xyz-d50",
            PredefinedColorSpace::XyzD65 => "xyz-d65",
            #[allow(unreachable_patterns)]
            _ => "future",
        },
        #[allow(unreachable_patterns)]
        _ => "future",
    }
}

fn family_name(family: SerializationFamily) -> &'static str {
    match family {
        SerializationFamily::LegacySrgb => "legacy-srgb",
        SerializationFamily::ColorFunction => "color-function",
        SerializationFamily::Natural => "natural",
        #[allow(unreachable_patterns)]
        _ => "future",
    }
}

fn transformation_name(transformation: Transformation) -> String {
    match transformation {
        Transformation::ForceOpaqueAlpha => "force-opaque-alpha".to_owned(),
        Transformation::QuantizeSrgb => "quantize-srgb".to_owned(),
        Transformation::ColorFunction => "color-function".to_owned(),
        Transformation::Convert(space) => format!("convert-{}", space_name(space)),
        #[allow(unreachable_patterns)]
        _ => "future".to_owned(),
    }
}

fn standard_failure(failure: CssColorStandardFailure) -> String {
    format!(
        "failure={} range={}..{}",
        failure_kind(failure.kind),
        failure.source.start,
        failure.source.end
    )
}

fn failure_kind(kind: CssColorStandardFailureKind) -> &'static str {
    match kind {
        CssColorStandardFailureKind::ColorGrammar => "color-grammar",
        CssColorStandardFailureKind::ComponentGrammar => "component-grammar",
        CssColorStandardFailureKind::InvalidComponents => "invalid-components",
        CssColorStandardFailureKind::InvalidProfile => "invalid-profile",
        CssColorStandardFailureKind::NumericGrammar => "numeric-grammar",
        #[allow(unreachable_patterns)]
        _ => "future",
    }
}

/// The error category and its facts, then the `Display` chain.
fn ordinary_error(error: &CssColorOrdinaryError) -> String {
    let category = match error {
        CssColorOrdinaryError::Operation(operation) => match operation {
            CssColorOperationError::Facts(_) => "operation:facts".to_owned(),
            CssColorOperationError::Protocol(_) => "operation:protocol".to_owned(),
            CssColorOperationError::Unavailable(unavailable) => {
                format!("operation:unavailable:{}", unavailable_name(unavailable))
            }
            CssColorOperationError::Unsupported(unsupported) => {
                format!("operation:unsupported:{}", unsupported_name(unsupported))
            }
            #[allow(unreachable_patterns)]
            _ => "operation:future".to_owned(),
        },
        CssColorOrdinaryError::Limit {
            resource,
            required,
            allowed,
        } => format!(
            "limit:{}:required={required}:allowed={allowed}",
            resource_name(*resource)
        ),
        CssColorOrdinaryError::Allocation { resource } => {
            format!("allocation:{}", resource_name(*resource))
        }
    };
    format!("{category} | {}", super::error_text(error))
}

pub(super) fn unavailable_name(unavailable: &CssColorUnavailable) -> &'static str {
    match unavailable {
        CssColorUnavailable::CurrentColor => "current-color",
        CssColorUnavailable::Device => "device",
        CssColorUnavailable::Palette(_) => "palette",
        CssColorUnavailable::Profile => "profile",
        CssColorUnavailable::ProfileConversion => "profile-conversion",
        CssColorUnavailable::Scheme => "scheme",
        #[allow(unreachable_patterns)]
        _ => "future",
    }
}

pub(super) fn unsupported_name(unsupported: &CssColorUnsupported) -> &'static str {
    match unsupported {
        CssColorUnsupported::ContextualUnit(_) => "contextual-unit",
        CssColorUnsupported::CustomInterpolation => "custom-interpolation",
        CssColorUnsupported::ProfileConversion => "profile-conversion",
        CssColorUnsupported::ReverseProfile => "reverse-profile",
        #[allow(unreachable_patterns)]
        _ => "future",
    }
}

pub(super) fn resource_name(resource: CssColorOrdinaryResource) -> &'static str {
    match resource {
        CssColorOrdinaryResource::PathSteps => "path-steps",
        CssColorOrdinaryResource::InputUnits => "input-units",
        CssColorOrdinaryResource::AlgorithmRecords => "algorithm-records",
        CssColorOrdinaryResource::ArithmeticRecords => "arithmetic-records",
        CssColorOrdinaryResource::EvaluationRecords => "evaluation-records",
        CssColorOrdinaryResource::Actions => "actions",
    }
}
