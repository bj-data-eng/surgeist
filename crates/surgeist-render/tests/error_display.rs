//! Display text and `source()` of the render error enums that are available without features.
//!
//! Each expected string is written from the documented meaning of its variant (the doc comment on
//! the variant and on the type), not from the `Debug` output. Every table is paired with an
//! exhaustive `match` without a wildcard that maps a variant to its row index, so a new variant
//! fails to compile here until it gets a message, and with a check that every row index occurs.
//! The numeric `value` of the `Numeric` variants is deliberately not part of the message: several
//! raise sites pass `0.0` when no value exists, so printing it would state a false fact.

use core::error::Error as StdError;
use core::fmt::Debug;
use surgeist_render::{
    ConvolutionNumericPhase, ConvolutionParameter, ConvolutionPreparationError,
    DisplacementNumericPhase, DisplacementParameter, DisplacementPreparationError,
    DropShadowNumericPhase, DropShadowPreparationError, GaussianNumericPhase, GaussianParameter,
    GaussianPreparationError, GeometryRealizationError, GlyphCapability, GlyphNumericPhase,
    GlyphPreparationError, GlyphResource, GradientHintError, GradientRealizationError,
    MorphologyNumericPhase, MorphologyParameter, MorphologyPreparationError, OffsetNumericPhase,
    OffsetParameter, OffsetPreparationError, Rect, RenderAccountingSetupError, Size,
    VariationTable,
};

/// Checks one table: message, no Debug form, no struct syntax, `source()` presence, row coverage.
fn check<E: StdError + Debug + 'static>(
    name: &str,
    rows: &[(E, &str)],
    index: fn(&E) -> usize,
    variants: usize,
) {
    let mut seen = vec![false; variants];
    for (value, expected) in rows {
        let text = value.to_string();
        assert_eq!(text, *expected, "{name}: message of {value:?}");
        assert_ne!(
            text,
            format!("{value:?}"),
            "{name}: Display is the Debug form"
        );
        assert!(
            !text.contains(['{', '}']),
            "{name}: message of {value:?} contains struct syntax"
        );
        assert!(!text.is_empty());
        assert!(
            value.source().is_none(),
            "{name}: leaf error {value:?} must not report a source"
        );
        seen[index(value)] = true;
    }
    assert!(
        seen.iter().all(|covered| *covered),
        "{name}: a variant has no message row: {seen:?}"
    );
}

#[test]
fn every_enum_is_a_standard_error() {
    fn assert_error<E: StdError + Debug + 'static>() {}
    assert_error::<ConvolutionPreparationError>();
    assert_error::<DisplacementPreparationError>();
    assert_error::<DropShadowPreparationError>();
    assert_error::<GaussianPreparationError>();
    assert_error::<MorphologyPreparationError>();
    assert_error::<OffsetPreparationError>();
    assert_error::<GlyphPreparationError>();
    assert_error::<GeometryRealizationError>();
    assert_error::<GradientHintError>();
    assert_error::<GradientRealizationError>();
    assert_error::<RenderAccountingSetupError>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_plain_data_errors_are_send_and_sync() {
    fn assert_send_sync<E: StdError + Send + Sync + 'static>() {}
    assert_send_sync::<ConvolutionPreparationError>();
    assert_send_sync::<DisplacementPreparationError>();
    assert_send_sync::<DropShadowPreparationError>();
    assert_send_sync::<GaussianPreparationError>();
    assert_send_sync::<MorphologyPreparationError>();
    assert_send_sync::<OffsetPreparationError>();
    assert_send_sync::<GlyphPreparationError>();
    assert_send_sync::<GeometryRealizationError>();
    assert_send_sync::<GradientHintError>();
    assert_send_sync::<GradientRealizationError>();
    assert_send_sync::<RenderAccountingSetupError>();
}

#[test]
fn convolution_messages_name_the_parameter_and_the_numeric_boundary() {
    use ConvolutionNumericPhase as Phase;
    use ConvolutionParameter as Parameter;
    use ConvolutionPreparationError as E;
    let mut rows = Vec::new();
    for (parameter, label) in [
        (Parameter::OrderX, "horizontal order"),
        (Parameter::OrderY, "vertical order"),
        (Parameter::TargetX, "horizontal target"),
        (Parameter::TargetY, "vertical target"),
        (Parameter::Coefficient, "kernel coefficient"),
        (Parameter::Divisor, "divisor"),
        (Parameter::Bias, "bias"),
    ] {
        rows.push((
            E::InvalidParameter {
                parameter,
                value: 2.5,
            },
            format!("convolution parameter {label} has the invalid value 2.5"),
        ));
    }
    for (phase, label) in [
        (Phase::TapCount, "tap count"),
        (Phase::DivisorSum, "divisor sum"),
        (Phase::WeightNormalization, "weight normalization"),
        (Phase::BiasNarrowing, "bias narrowing"),
        (Phase::AccumulationBound, "accumulation bound"),
        (Phase::GridIndex, "grid index"),
        (Phase::GridMapping, "grid mapping"),
    ] {
        rows.push((
            E::Numeric { phase, value: 0.0 },
            format!("convolution numeric check failed at {label}"),
        ));
    }
    let rows: Vec<(E, &str)> = rows.iter().map(|(e, s)| (*e, s.as_str())).collect();
    check(
        "ConvolutionPreparationError",
        &rows,
        |e| match e {
            E::InvalidParameter { .. } => 0,
            E::Numeric { .. } => 1,
        },
        2,
    );
}

#[test]
fn displacement_messages_describe_the_map_and_source_contract() {
    use DisplacementNumericPhase as Phase;
    use DisplacementPreparationError as E;
    let size = Size::try_new(2.5, 3.0).unwrap();
    let mut rows = vec![
        (
            E::InvalidParameter {
                parameter: DisplacementParameter::Scale,
                value: f64::INFINITY,
            },
            "displacement parameter scale has the invalid value inf".to_string(),
        ),
        (
            E::InvalidMapDimensions { size },
            "displacement map size 2.5 by 3 is not a pair of finite nonnegative integers that fit in u32"
                .to_string(),
        ),
        (
            E::MapPayloadOverflow { size },
            "the byte length of a displacement map of 2.5 by 3 pixels cannot be represented"
                .to_string(),
        ),
        (
            E::InvalidMapPayload {
                expected: 16,
                actual: 12,
            },
            "displacement map has 12 bytes but its dimensions require 16".to_string(),
        ),
        (
            E::EmptyPeriod,
            "a periodic source must have positive width and height".to_string(),
        ),
        (
            E::InfiniteSourceBackdrop,
            "a same-layer backdrop cannot be combined with a declared infinite source".to_string(),
        ),
    ];
    // A `Rect` is always valid, so the invalid-domain row carries an ordinary rectangle: the
    // message does not depend on the payload.
    rows.push((
        E::InvalidMapDomain {
            map: Rect::try_new(0.0, 0.0, 1.0, 1.0).unwrap(),
        },
        "displacement map rectangle has an invalid extent, origin or maximum".to_string(),
    ));
    for (phase, label) in [
        (Phase::GridMapping, "grid mapping"),
        (Phase::MapMapping, "map mapping"),
        (Phase::TableLayout, "table layout"),
        (Phase::GridIndex, "grid index"),
        (Phase::Sampling, "sampling"),
    ] {
        rows.push((
            E::Numeric { phase, value: 0.0 },
            format!("displacement numeric check failed at {label}"),
        ));
    }
    let rows: Vec<(E, &str)> = rows.iter().map(|(e, s)| (*e, s.as_str())).collect();
    check(
        "DisplacementPreparationError",
        &rows,
        |e| match e {
            E::InvalidParameter { .. } => 0,
            E::InvalidMapDimensions { .. } => 1,
            E::MapPayloadOverflow { .. } => 2,
            E::InvalidMapPayload { .. } => 3,
            E::InvalidMapDomain { .. } => 4,
            E::Numeric { .. } => 5,
            E::EmptyPeriod => 6,
            E::InfiniteSourceBackdrop => 7,
        },
        8,
    );
}

#[test]
fn drop_shadow_messages_cover_opacity_period_backdrop_and_merge_arithmetic() {
    use DropShadowNumericPhase as Phase;
    use DropShadowPreparationError as E;
    let mut rows = vec![
        (
            E::InvalidFloodOpacity { value: 1.5 },
            "drop shadow flood opacity 1.5 is not finite or is outside 0 to 1".to_string(),
        ),
        (
            E::EmptyPeriod,
            "a periodic source must have positive width and height".to_string(),
        ),
        (
            E::InfiniteSourceBackdrop,
            "a same-layer backdrop cannot be combined with a declared infinite source".to_string(),
        ),
    ];
    for (phase, label) in [
        (
            Phase::MergeMapping,
            "merge mapping, kernel extent or sampling",
        ),
        (Phase::GridIndex, "grid index"),
        (Phase::Sampling, "sampling"),
    ] {
        rows.push((
            E::Numeric { phase, value: 0.0 },
            format!("drop shadow numeric check failed at {label}"),
        ));
    }
    let rows: Vec<(E, &str)> = rows.iter().map(|(e, s)| (*e, s.as_str())).collect();
    check(
        "DropShadowPreparationError",
        &rows,
        |e| match e {
            E::InvalidFloodOpacity { .. } => 0,
            E::EmptyPeriod => 1,
            E::InfiniteSourceBackdrop => 2,
            E::Numeric { .. } => 3,
        },
        4,
    );
}

#[test]
fn gaussian_messages_name_the_deviation_axis_and_the_numeric_boundary() {
    use GaussianNumericPhase as Phase;
    use GaussianPreparationError as E;
    let mut rows = vec![
        (
            E::InvalidParameter {
                parameter: GaussianParameter::StandardDeviationX,
                value: f64::NAN,
            },
            "Gaussian parameter horizontal standard deviation has the invalid value NaN"
                .to_string(),
        ),
        (
            E::InvalidParameter {
                parameter: GaussianParameter::StandardDeviationY,
                value: f64::NEG_INFINITY,
            },
            "Gaussian parameter vertical standard deviation has the invalid value -inf".to_string(),
        ),
        (
            E::EmptyPeriod,
            "a periodic source must have positive width and height".to_string(),
        ),
        (
            E::InfiniteSourceBackdrop,
            "a same-layer backdrop cannot be combined with a declared infinite source".to_string(),
        ),
    ];
    for (phase, label) in [
        (Phase::KernelExtent, "kernel extent"),
        (Phase::GridIndex, "grid index"),
        (Phase::GridMapping, "grid mapping"),
        (Phase::Sampling, "sampling"),
    ] {
        rows.push((
            E::Numeric { phase, value: 0.0 },
            format!("Gaussian numeric check failed at {label}"),
        ));
    }
    let rows: Vec<(E, &str)> = rows.iter().map(|(e, s)| (*e, s.as_str())).collect();
    check(
        "GaussianPreparationError",
        &rows,
        |e| match e {
            E::InvalidParameter { .. } => 0,
            E::Numeric { .. } => 1,
            E::EmptyPeriod => 2,
            E::InfiniteSourceBackdrop => 3,
        },
        4,
    );
}

#[test]
fn morphology_messages_name_the_radius_axis_and_the_numeric_boundary() {
    use MorphologyNumericPhase as Phase;
    use MorphologyPreparationError as E;
    let mut rows = vec![
        (
            E::InvalidParameter {
                parameter: MorphologyParameter::RadiusX,
                value: f64::NAN,
            },
            "morphology parameter horizontal radius has the invalid value NaN".to_string(),
        ),
        (
            E::InvalidParameter {
                parameter: MorphologyParameter::RadiusY,
                value: f64::INFINITY,
            },
            "morphology parameter vertical radius has the invalid value inf".to_string(),
        ),
        (
            E::EmptyPeriod,
            "a periodic source must have positive width and height".to_string(),
        ),
        (
            E::InfiniteSourceBackdrop,
            "a same-layer backdrop cannot be combined with a declared infinite source".to_string(),
        ),
    ];
    for (phase, label) in [
        (Phase::RadiusGrid, "radius grid"),
        (Phase::EndpointMapping, "endpoint mapping"),
        (Phase::TableLayout, "table layout"),
        (Phase::GridIndex, "grid index"),
        (Phase::Sampling, "sampling"),
    ] {
        rows.push((
            E::Numeric { phase, value: 0.0 },
            format!("morphology numeric check failed at {label}"),
        ));
    }
    let rows: Vec<(E, &str)> = rows.iter().map(|(e, s)| (*e, s.as_str())).collect();
    check(
        "MorphologyPreparationError",
        &rows,
        |e| match e {
            E::InvalidParameter { .. } => 0,
            E::Numeric { .. } => 1,
            E::EmptyPeriod => 2,
            E::InfiniteSourceBackdrop => 3,
        },
        4,
    );
}

#[test]
fn offset_messages_name_the_displacement_axis_and_the_numeric_boundary() {
    use OffsetNumericPhase as Phase;
    use OffsetPreparationError as E;
    let mut rows = vec![
        (
            E::InvalidParameter {
                parameter: OffsetParameter::Dx,
                value: f64::NAN,
            },
            "offset parameter horizontal displacement has the invalid value NaN".to_string(),
        ),
        (
            E::InvalidParameter {
                parameter: OffsetParameter::Dy,
                value: f64::INFINITY,
            },
            "offset parameter vertical displacement has the invalid value inf".to_string(),
        ),
    ];
    for (phase, label) in [
        (Phase::GridMapping, "grid mapping"),
        (Phase::GridIndex, "grid index"),
    ] {
        rows.push((
            E::Numeric { phase, value: 0.0 },
            format!("offset numeric check failed at {label}"),
        ));
    }
    let rows: Vec<(E, &str)> = rows.iter().map(|(e, s)| (*e, s.as_str())).collect();
    check(
        "OffsetPreparationError",
        &rows,
        |e| match e {
            E::InvalidParameter { .. } => 0,
            E::Numeric { .. } => 1,
        },
        2,
    );
}

#[test]
fn glyph_preparation_messages_describe_each_refusal() {
    use GlyphPreparationError as E;
    let mut rows: Vec<(E, String)> =
        vec![
        (E::InvalidFont, "font, outline, color or bitmap data is invalid".into()),
        (
            E::GlyphRange {
                glyph: 90,
                count: 40,
            },
            "glyph 90 is outside the face's 40 glyphs".into(),
        ),
        (
            E::CoordinateCount {
                supplied: 1,
                required: 2,
            },
            "1 variation coordinates were supplied but the face has 2 axes".into(),
        ),
        (
            E::CoordinateValue { value: 20000.0 },
            "a variation coordinate is outside the normalized domain, which is -1 to 1 as a scalar and -16384 to 16384 as 2.14 bits".into(),
        ),
        (
            E::CoordinateLoss { value: 0.5 },
            "variation coordinate 0.5 would change during quantization".into(),
        ),
        (
            E::UnsupportedColor,
            "the selected COLR drawing is outside the supported domain".into(),
        ),
        (
            E::UnsupportedColorPaint { format: 9 },
            "COLRv1 paint format 9 is outside the supported graph".into(),
        ),
        (
            E::UnsupportedImageInstance,
            "color and bitmap glyphs require default variation coordinates and no synthesis".into(),
        ),
        (
            E::UnsupportedPng,
            "the selected PNG image or its color metadata is outside the supported decoder profile"
                .into(),
        ),
        (
            E::UnsupportedBitmap,
            "the selected bitmap drawing is outside the supported domain".into(),
        ),
        (
            E::InvalidSynthesis,
            "synthesis skew or embolden is not finite, or embolden is negative".into(),
        ),
        (
            E::Provider { glyph: 7 },
            "the provider refused glyph 7, or cannot faithfully decode its encoding".into(),
        ),
        (
            E::CapacityOverflow,
            "a size or count calculation overflowed".into(),
        ),
    ];
    for (capability, label) in [
        (
            GlyphCapability::CompositeGlyph,
            "composite glyphs outside the static XY domain",
        ),
        (GlyphCapability::PostScript, "PostScript outlines"),
        (
            GlyphCapability::PostScriptVariation,
            "variable PostScript outlines",
        ),
        (
            GlyphCapability::VariableComposite,
            "variable composite glyphs",
        ),
        (GlyphCapability::SparseVariation, "sparse variation tuples"),
    ] {
        rows.push((
            E::UnsupportedOutline { capability },
            format!("glyph outline processing for {label} is not supported"),
        ));
    }
    for (table, label) in [
        (VariationTable::Fvar, "fvar"),
        (VariationTable::Gvar, "gvar"),
        (VariationTable::Hvar, "HVAR"),
    ] {
        rows.push((
            E::InvalidVariation { table },
            format!("the {label} variation table is malformed"),
        ));
    }
    for (phase, label) in [
        (GlyphNumericPhase::Size, "canonical size"),
        (
            GlyphNumericPhase::ProviderPoint,
            "provider point arithmetic",
        ),
        (GlyphNumericPhase::PathPoint, "path point conversion"),
        (GlyphNumericPhase::InkExtent, "ink extent"),
        (GlyphNumericPhase::Placement, "placement"),
        (GlyphNumericPhase::Synthesis, "synthesis displacement"),
        (GlyphNumericPhase::BitmapColor, "bitmap color"),
    ] {
        rows.push((
            E::Numeric { phase, value: 0.0 },
            format!("glyph numeric check failed at {label}"),
        ));
    }
    for (resource, label) in [
        (GlyphResource::Glyphs, "glyphs"),
        (GlyphResource::PathElements, "path elements"),
        (GlyphResource::ScratchBytes, "scratch cpu bytes"),
        (GlyphResource::RetainedBytes, "retained cpu bytes"),
    ] {
        rows.push((
            E::Limit {
                resource,
                required: 9,
                limit: 4,
            },
            format!("{label} requires 9, above the limit of 4"),
        ));
        rows.push((
            E::Allocation { resource },
            format!("allocating {label} failed"),
        ));
    }
    let rows: Vec<(E, &str)> = rows.iter().map(|(e, s)| (*e, s.as_str())).collect();
    check(
        "GlyphPreparationError",
        &rows,
        |e| match e {
            E::UnsupportedOutline { .. } => 0,
            E::InvalidFont => 1,
            E::GlyphRange { .. } => 2,
            E::CoordinateCount { .. } => 3,
            E::CoordinateValue { .. } => 4,
            E::CoordinateLoss { .. } => 5,
            E::InvalidVariation { .. } => 6,
            E::UnsupportedColor => 7,
            E::UnsupportedColorPaint { .. } => 8,
            E::UnsupportedImageInstance => 9,
            E::UnsupportedPng => 10,
            E::UnsupportedBitmap => 11,
            E::InvalidSynthesis => 12,
            E::Numeric { .. } => 13,
            E::Provider { .. } => 14,
            E::Limit { .. } => 15,
            E::CapacityOverflow => 16,
            E::Allocation { .. } => 17,
        },
        18,
    );
}

#[test]
fn geometry_realization_messages_say_what_cannot_be_realized_natively() {
    use GeometryRealizationError as E;
    let rows = [
        (
            E::UnrepresentableTransform,
            "a transform coefficient cannot be represented natively",
        ),
        (
            E::UnrepresentablePath { element_index: 7 },
            "path element 7 cannot be converted or retained natively",
        ),
        (
            E::UnrepresentableStroke,
            "a stroke parameter cannot be used natively",
        ),
        (
            E::UnrepresentableGlyph { glyph_index: 3 },
            "glyph 3 cannot be represented natively",
        ),
        (
            E::UnrepresentableBlur,
            "a blur parameter cannot be represented natively",
        ),
        (
            E::InsufficientNativeCapacity,
            "native geometry cannot meet the required rank, precision, range or dash behavior",
        ),
    ];
    check(
        "GeometryRealizationError",
        &rows,
        |e| match e {
            E::UnrepresentableTransform => 0,
            E::UnrepresentablePath { .. } => 1,
            E::UnrepresentableStroke => 2,
            E::UnrepresentableGlyph { .. } => 3,
            E::UnrepresentableBlur => 4,
            E::InsufficientNativeCapacity => 5,
        },
        6,
    );
}

#[test]
fn gradient_hint_messages_name_the_stop_that_carries_the_hint() {
    use GradientHintError as E;
    let rows = [
        (
            E::TrailingHint { stop_index: 2 },
            "stop 2 is the last stop and cannot carry an outgoing hint",
        ),
        (
            E::HintOutsideInterval { stop_index: 1 },
            "the hint after stop 1 lies outside the interval of its adjacent colors",
        ),
        (
            E::UnsupportedSweep { stop_index: 0 },
            "a hinted full-turn sweep after stop 0 is not supported",
        ),
    ];
    check(
        "GradientHintError",
        &rows,
        |e| match e {
            E::TrailingHint { .. } => 0,
            E::HintOutsideInterval { .. } => 1,
            E::UnsupportedSweep { .. } => 2,
        },
        3,
    );
}

#[test]
fn gradient_realization_messages_say_what_the_native_field_cannot_hold() {
    use GradientRealizationError as E;
    let rows = [
        (
            E::DegenerateLinearField,
            "a linear gradient field has no direction",
        ),
        (
            E::UnrepresentableGeometry,
            "gradient range or transformed geometry cannot be represented natively",
        ),
        (
            E::UnrepresentableStop { stop_index: 4 },
            "stop 4 cannot keep its source position, ordering or separation natively",
        ),
        (
            E::UnrepresentableHint { stop_index: 3 },
            "the hint after stop 3 cannot be represented natively with its source equality and containment",
        ),
        (
            E::InsufficientNativeCapacity,
            "native gradient arithmetic cannot meet the required accuracy or range",
        ),
    ];
    check(
        "GradientRealizationError",
        &rows,
        |e| match e {
            E::DegenerateLinearField => 0,
            E::UnrepresentableGeometry => 1,
            E::UnrepresentableStop { .. } => 2,
            E::UnrepresentableHint { .. } => 3,
            E::InsufficientNativeCapacity => 4,
        },
        5,
    );
}

#[test]
fn accounting_setup_messages_say_why_no_attempt_started() {
    use RenderAccountingSetupError as E;
    let rows = [
        (
            E::Allocation,
            "accounting could not allocate its setup or report storage",
        ),
        (
            E::TimingUnavailable,
            "timed profiling is unavailable on this target",
        ),
        (E::IdentityExhausted, "accounting identities are exhausted"),
    ];
    check(
        "RenderAccountingSetupError",
        &rows,
        |e| match e {
            E::Allocation => 0,
            E::TimingUnavailable => 1,
            E::IdentityExhausted => 2,
        },
        3,
    );
}
