//! Colour outcomes: ordinary used colours, authored resolution and the
//! controlled protocol with an HTML-style transformation sequence.

use surgeist_color::{
    Color, ColorSpace, DigitSlot, OperationOwner, PredefinedColorSpace, Serialization,
    SerializationAction, SerializationFamily, SerializationReceipt, SerializationRequest,
    SerializationStep, Transformation,
};
use surgeist_css::{
    CssAuthoredColorOrdinaryLimits, CssAuthoredColorOutcome, CssColorBank, CssColorError,
    CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInput, CssColorInputKind,
    CssColorOperationOwner, CssColorOrdinaryLimits, CssColorOrdinaryOutcome,
    CssColorOrdinaryResource, CssColorPaletteEntry, CssColorParse, CssColorParseAction,
    CssColorParseOutcome, CssColorParseStep, CssColorPropertyFacts, CssColorResolveError,
    CssColorSchemeFact, CssColorSlot, CssColorStandardFailure, CssColorStandardFailureKind,
    CssColorTransform, CssColorUnavailable, CssColorUnsupported, CssDeviceColorFact, CssImportance,
    CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef, CssUsedColorCheckpoint,
    parse_property_value_text, parse_used_color, resolve_authored_color,
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

const BANKS: [CssColorBank; 3] = [
    CssColorBank::Algorithm,
    CssColorBank::Arithmetic,
    CssColorBank::Evaluation,
];

/// The number of computations one transformation takes. The consuming
/// typestate exposes exactly one `compute` per transform, so the `computes=`
/// field records that guarantee; it is not a count the adapter observes.
const TRANSFORM_COMPUTES: usize = 1;

/// Completes a transformation with its single computation.
fn drive_transformation<'facts>(
    transformation: CssColorTransform<'_, 'facts>,
) -> CssUsedColorCheckpoint<'facts> {
    transformation.compute()
}

/// Services every parse action; returns the outcome and the parse's final
/// acknowledged bank lengths, read before the completing step.
fn drive_parse<'owner, 'facts>(
    mut parse: CssColorParse<'owner, 'facts>,
    input: &[u16],
    banks: &mut [Vec<CssColorSlot<'owner>>; 3],
    counts: &mut ActionCounts,
) -> Result<(CssColorParseOutcome<'facts>, [usize; 3]), CssColorError> {
    loop {
        let lengths = BANKS.map(|bank| parse.bank_length(bank));
        let action = match parse.step() {
            CssColorParseStep::Complete(outcome) => return Ok((outcome, lengths)),
            CssColorParseStep::Pending(action) => action,
        };
        parse = match action {
            CssColorParseAction::ReadInput(read) => {
                counts.read_input += 1;
                let unit = input[read.offset()];
                read.read(unit)?
            }
            CssColorParseAction::ReadSlot(read) => {
                counts.read_slot += 1;
                let slot = &banks[ordinal(read.bank())][read.index()];
                read.read(slot)?
            }
            CssColorParseAction::AppendSlot(write) => {
                counts.append_slot += 1;
                let target = &mut banks[ordinal(write.bank())];
                assert_eq!(write.index(), target.len(), "append at the bank end");
                let (record, receipt) = write.record();
                target.push(record);
                receipt.stored(&target[target.len() - 1])
            }
            CssColorParseAction::UpdateSlot(write) => {
                counts.update_slot += 1;
                let index = write.index();
                let target = &mut banks[ordinal(write.bank())];
                let (record, receipt) = write.record();
                target[index] = record;
                receipt.stored(&target[index])
            }
            CssColorParseAction::Compute(compute) => {
                counts.compute += 1;
                compute.compute()?
            }
            CssColorParseAction::ReadFact(read) => {
                counts.read_fact += 1;
                read.read()?
            }
        };
    }
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
    let parse = match CssColorParse::start(
        &mut owner,
        CssColorInputKind::Utf16,
        input.len(),
        CssColorGrammarRequest::WholeColor,
        facts,
        facts.identity(),
    ) {
        Ok(parse) => parse,
        Err(error) => {
            observation.shape = "err".to_owned();
            observation.add_fact(format!("start-error={}", error_chain(&error)));
            return observation;
        }
    };
    let driven = drive_parse(parse, input, &mut banks, &mut counts);
    observation.add_fact(format!(
        "parse-actions:read-input={},read-slot={},append-slot={},update-slot={},compute={},read-fact={}",
        counts.read_input,
        counts.read_slot,
        counts.append_slot,
        counts.update_slot,
        counts.compute,
        counts.read_fact
    ));
    let (outcome, lengths) = match driven {
        Ok(driven) => driven,
        Err(error) => {
            observation.shape = "err".to_owned();
            observation.add_fact(format!("parse-error={}", error_chain(&error)));
            return observation;
        }
    };
    observation.add_fact(format!(
        "banks=algorithm:{},arithmetic:{},evaluation:{}",
        lengths[0], lengths[1], lengths[2]
    ));
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
    let mut checkpoint = checkpoint;
    for transformation in transformations {
        let name = transformation_name(*transformation);
        checkpoint =
            drive_transformation(checkpoint.transform(&mut numerical_owner, *transformation));
        observation.add_fact(format!(
            "{name}:computes={TRANSFORM_COMPUTES} -> {}",
            color_bits(checkpoint.value())
        ));
    }
    // CSS does not wrap serialization: the consumer serializes the checkpoint
    // value with Color and keeps the checkpoint's standard failure.
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
fn ordinary_error(error: &CssColorResolveError) -> String {
    let category = if let Some(operation) = error.color_error() {
        if operation.fact_defect().is_some() {
            "operation:facts".to_owned()
        } else if let Some(offset) = operation.malformed_input_offset() {
            format!("operation:malformed-input:offset={offset}")
        } else if let Some(unavailable) = operation.unavailable() {
            format!("operation:unavailable:{}", unavailable_name(&unavailable))
        } else if let Some(unsupported) = operation.unsupported() {
            format!("operation:unsupported:{}", unsupported_name(&unsupported))
        } else {
            "operation:future".to_owned()
        }
    } else if let Some((resource, allowed, required)) = error.exhausted_limit() {
        format!(
            "limit:{}:required={required}:allowed={allowed}",
            resource_name(resource)
        )
    } else if let Some(resource) = error.failed_allocation() {
        format!("allocation:{}", resource_name(resource))
    } else {
        "future".to_owned()
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
        _ => "future",
    }
}
