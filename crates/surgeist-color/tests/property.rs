//! Generated properties of every numerical colour operation.
//!
//! CSS Values 4 §10.12 and CSS Color 4 §4 require a finite value that would
//! leave the supported range to become the closest supported value, so every
//! operation over proven operands returns a value; the proof types guarantee
//! finiteness. These properties check what the types cannot: the
//! spec-defined shape of each result (alpha kept by conversion, identity
//! within a space, present cross-space channels, nonnegative radii and
//! normalized hues, contrast as opaque white or black, interpolation
//! endpoints, naive-conversion signs), and that the controlled computation
//! returns exactly the direct call's bits. Cases draw channels
//! from binary64 extremes (`±f64::MAX`, the smallest subnormals, signed zeros
//! and raw bit patterns), every colour space, every interpolation method and
//! unit-interval progress. The cases and their codec belong to this crate;
//! `surgeist-fuzz` supplies the bounded proptest runner.
#![forbid(unsafe_code)]

use surgeist_color::{
    Color, ColorSpace as Space, Computation, Finite, HslCoordinates, HueInterpolation as Hue,
    HwbCoordinates, InterpolationMethod, NumericalRequest, OperationOwner, PolarCoordinates,
    PredefinedColorSpace as Predefined, RectangularCoordinates, RectangularSpace,
    SerializationFamily, Transformation, UnitInterval, convert_hsl_to_srgb, convert_hwb_to_srgb,
    convert_lab_to_lch, convert_lch_to_lab, convert_oklab_to_oklch, convert_oklch_to_oklab,
    convert_srgb_to_hsl, convert_srgb_to_hwb, naively_convert_cmyk_to_srgba,
    naively_convert_srgba_to_cmyk,
};
use surgeist_fuzz::engine::{PropertyConfig, run_property};
use surgeist_fuzz::generation::property::bytes;
use surgeist_fuzz::{
    BuildRequirements, CaseFormat, CaseOutcome, CaseTarget, FailureClass, FailureKey, FuzzError,
    FuzzErrorKind, Generation, GeneratorIdentity, InputEncoding, InputLimits, PropertyId, Result,
    TargetDescriptor, TargetDescriptorSpec, TargetId,
};

const OWNER: &str = "surgeist-color";
const CASES: u32 = 512;
/// Canonical case bytes: two samples, a destination, a method, progress,
/// alpha, a transformation and the standalone CMYK and RGBA samples.
const CASE_BYTES: usize = 2 * SAMPLE_BYTES + 1 + 2 + 8 + 9 + 2 + 5 * 8 + 4 * 8;
const SAMPLE_BYTES: usize = 1 + 3 * 9 + 9;
const ENTROPY_BYTES: usize = 256;

const SPACES: [Space; 15] = [
    Space::Predefined(Predefined::Srgb),
    Space::Predefined(Predefined::SrgbLinear),
    Space::Predefined(Predefined::DisplayP3),
    Space::Predefined(Predefined::DisplayP3Linear),
    Space::Predefined(Predefined::A98Rgb),
    Space::Predefined(Predefined::ProphotoRgb),
    Space::Predefined(Predefined::Rec2020),
    Space::Predefined(Predefined::XyzD50),
    Space::Predefined(Predefined::XyzD65),
    Space::Hsl,
    Space::Hwb,
    Space::Lab,
    Space::Lch,
    Space::Oklab,
    Space::Oklch,
];
const HUES: [Hue; 4] = [Hue::Shorter, Hue::Longer, Hue::Increasing, Hue::Decreasing];

/// Finite edges: signed zeros, both extremes, the smallest subnormals, the
/// smallest normal, unit values and large decimal magnitudes.
const FINITE_EDGES: [f64; 13] = [
    0.0,
    -0.0,
    f64::MAX,
    -f64::MAX,
    5e-324,
    -5e-324,
    f64::MIN_POSITIVE,
    1.0,
    -1.0,
    1e308,
    -1e308,
    100.0,
    0.5,
];
const UNIT_EDGES: [f64; 6] = [0.0, 1.0, 0.5, 5e-324, 1.0 - f64::EPSILON / 2.0, -0.0];

#[derive(Clone, Copy, Debug, PartialEq)]
struct Sample {
    space: usize,
    channels: [Option<f64>; 3],
    alpha: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Case {
    first: Sample,
    second: Sample,
    destination: usize,
    method_space: usize,
    /// Zero is the method default; one to four select an explicit hue method.
    hue: usize,
    progress: f64,
    alpha: Option<f64>,
    /// Zero to three: opaque alpha, conversion, quantization, colour function.
    transformation: usize,
    transformation_space: usize,
    cmyka: [f64; 5],
    rgba: [f64; 4],
}

/// Reads generation entropy; exhausted entropy reads as zero bytes.
struct Entropy<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl Entropy<'_> {
    fn byte(&mut self) -> u8 {
        let value = self.bytes.get(self.offset).copied().unwrap_or(0);
        self.offset += 1;
        value
    }
    fn bits(&mut self) -> u64 {
        u64::from_le_bytes(std::array::from_fn(|_| self.byte()))
    }
    fn index(&mut self, count: usize) -> usize {
        usize::from(self.byte()) % count
    }
    /// An edge, or raw bits whose nonfinite patterns become the signed extreme.
    fn finite(&mut self) -> f64 {
        let tag = usize::from(self.byte()) % (FINITE_EDGES.len() + 3);
        if let Some(edge) = FINITE_EDGES.get(tag) {
            return *edge;
        }
        let value = f64::from_bits(self.bits());
        if value.is_finite() {
            value
        } else {
            f64::MAX.copysign(value)
        }
    }
    fn unit(&mut self) -> f64 {
        let tag = usize::from(self.byte()) % (UNIT_EDGES.len() + 1);
        UNIT_EDGES
            .get(tag)
            .copied()
            .unwrap_or_else(|| (self.bits() >> 11) as f64 / (1_u64 << 53) as f64)
    }
    fn channel(&mut self) -> Option<f64> {
        let missing = self.byte().is_multiple_of(5);
        let value = self.finite();
        (!missing).then_some(value)
    }
    fn alpha(&mut self) -> Option<f64> {
        let missing = self.byte().is_multiple_of(7);
        let value = self.unit();
        (!missing).then_some(value)
    }
    fn sample(&mut self) -> Sample {
        Sample {
            space: self.index(SPACES.len()),
            channels: [self.channel(), self.channel(), self.channel()],
            alpha: self.alpha(),
        }
    }
}

/// Fixed-layout canonical codec, so replay bytes decode to the same case.
struct Writer(Vec<u8>);
impl Writer {
    fn index(&mut self, value: usize) {
        self.0
            .push(u8::try_from(value).expect("selector fits one byte"));
    }
    fn number(&mut self, value: f64) {
        self.0.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    fn optional(&mut self, value: Option<f64>) {
        self.0.push(u8::from(value.is_some()));
        self.number(value.unwrap_or(0.0));
    }
    fn sample(&mut self, sample: Sample) {
        self.index(sample.space);
        for channel in sample.channels {
            self.optional(channel);
        }
        self.optional(sample.alpha);
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl Reader<'_> {
    fn byte(&mut self) -> Result<u8> {
        let value = *self
            .bytes
            .get(self.offset)
            .ok_or_else(|| invalid("short case"))?;
        self.offset += 1;
        Ok(value)
    }
    fn index(&mut self, count: usize) -> Result<usize> {
        let value = usize::from(self.byte()?);
        (value < count)
            .then_some(value)
            .ok_or_else(|| invalid("selector out of range"))
    }
    fn number(&mut self) -> Result<f64> {
        let mut bits = [0; 8];
        for byte in &mut bits {
            *byte = self.byte()?;
        }
        let value = f64::from_le_bytes(bits);
        value
            .is_finite()
            .then_some(value)
            .ok_or_else(|| invalid("nonfinite operand"))
    }
    fn unit(&mut self) -> Result<f64> {
        let value = self.number()?;
        (0.0..=1.0)
            .contains(&value)
            .then_some(value)
            .ok_or_else(|| invalid("unit operand outside zero to one"))
    }
    fn optional(&mut self, unit: bool) -> Result<Option<f64>> {
        let present = match self.byte()? {
            0 => false,
            1 => true,
            _ => return Err(invalid("presence flag")),
        };
        let value = if unit { self.unit()? } else { self.number()? };
        if !present && value.to_bits() != 0 {
            return Err(invalid("missing operand has payload"));
        }
        Ok(present.then_some(value))
    }
    fn sample(&mut self) -> Result<Sample> {
        Ok(Sample {
            space: self.index(SPACES.len())?,
            channels: [
                self.optional(false)?,
                self.optional(false)?,
                self.optional(false)?,
            ],
            alpha: self.optional(true)?,
        })
    }
}
fn invalid(detail: &str) -> FuzzError {
    FuzzError::new(FuzzErrorKind::InvalidCase, "decode colour case", detail)
}

fn property(name: &str) -> PropertyId {
    PropertyId::new(format!("{OWNER}::{name}")).expect("owner-qualified property")
}
const INVARIANTS: &str = "operation-invariants";
const ONE_PATH: &str = "computation-matches-direct";
const STANDALONE: &str = "standalone-conversion-invariants";

struct Target {
    descriptor: TargetDescriptor,
}
impl Target {
    fn new() -> Self {
        let descriptor = TargetDescriptor::new(TargetDescriptorSpec {
            id: TargetId::new(format!("{OWNER}::numerical-operations")).unwrap(),
            owner: OWNER.into(),
            binary: "surgeist-color-property".into(),
            properties: vec![
                property(INVARIANTS),
                property(ONE_PATH),
                property(STANDALONE),
            ],
            source_references: vec![
                "https://www.w3.org/TR/2026/CRD-css-values-4-20260311/#calc-range".into(),
                "https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#color-conversion".into(),
            ],
            format: CaseFormat::new(format!("{OWNER}::numerical-case"), 1).unwrap(),
            generator_identity: GeneratorIdentity::new(format!("{OWNER}::edge-entropy"), 1)
                .unwrap(),
            encoding: InputEncoding::RawBytes,
            limits: InputLimits::new(ENTROPY_BYTES.max(CASE_BYTES) as u64, 0, 0, 0, 0).unwrap(),
            requirements: BuildRequirements::new(vec![], vec![]).unwrap(),
            min_evaluated: u64::from(CASES),
            max_generation_rejections: 0,
        })
        .unwrap();
        Self { descriptor }
    }
}
impl CaseTarget for Target {
    type Case = Case;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Case> {
        let mut reader = Reader {
            bytes: input,
            offset: 0,
        };
        let mut case = Case {
            first: reader.sample()?,
            second: reader.sample()?,
            destination: reader.index(SPACES.len())?,
            method_space: reader.index(SPACES.len())?,
            hue: reader.index(HUES.len() + 1)?,
            progress: reader.unit()?,
            alpha: reader.optional(true)?,
            transformation: reader.index(4)?,
            transformation_space: reader.index(SPACES.len())?,
            cmyka: [0.0; 5],
            rgba: [0.0; 4],
        };
        for value in case.cmyka.iter_mut().chain(&mut case.rgba) {
            *value = reader.number()?;
        }
        if reader.offset != input.len() {
            return Err(invalid("trailing bytes"));
        }
        if case.hue != 0 && !SPACES[case.method_space].is_polar() {
            return Err(invalid("hue method requires a polar space"));
        }
        Ok(case)
    }
    fn encode(&self, case: &Case) -> Result<Vec<u8>> {
        let mut writer = Writer(Vec::with_capacity(CASE_BYTES));
        writer.sample(case.first);
        writer.sample(case.second);
        writer.index(case.destination);
        writer.index(case.method_space);
        writer.index(case.hue);
        writer.number(case.progress);
        writer.optional(case.alpha);
        writer.index(case.transformation);
        writer.index(case.transformation_space);
        for value in case.cmyka.into_iter().chain(case.rgba) {
            writer.number(value);
        }
        assert_eq!(writer.0.len(), CASE_BYTES, "fixed canonical layout");
        Ok(writer.0)
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<Case>> {
        let mut entropy = Entropy {
            bytes: input,
            offset: 0,
        };
        let first = entropy.sample();
        let second = entropy.sample();
        let destination = entropy.index(SPACES.len());
        let method_space = entropy.index(SPACES.len());
        let hue = entropy.index(HUES.len() + 1);
        Ok(Generation::Generated(Case {
            first,
            second,
            destination,
            method_space,
            hue: if SPACES[method_space].is_polar() {
                hue
            } else {
                0
            },
            progress: entropy.unit(),
            alpha: entropy.alpha(),
            transformation: entropy.index(4),
            transformation_space: entropy.index(SPACES.len()),
            cmyka: std::array::from_fn(|_| entropy.finite()),
            rgba: std::array::from_fn(|_| entropy.finite()),
        }))
    }
    fn evaluate(&self, case: &Case) -> Result<CaseOutcome> {
        if let Err((operation, detail)) = operation_invariants(case) {
            return failure(INVARIANTS, operation, detail);
        }
        if let Err((operation, detail)) = computation_matches_direct(case) {
            return failure(ONE_PATH, operation, detail);
        }
        if let Err((operation, detail)) = standalone_invariants(case) {
            return failure(STANDALONE, operation, detail);
        }
        CaseOutcome::evaluated(vec![
            property(INVARIANTS),
            property(ONE_PATH),
            property(STANDALONE),
        ])
    }
}
fn failure(name: &str, operation: &str, detail: String) -> Result<CaseOutcome> {
    CaseOutcome::failure(
        FailureKey::new(
            property(name),
            FailureClass::PropertyViolation,
            Some(operation.into()),
        )?,
        detail,
    )
}

type Check = std::result::Result<(), (&'static str, String)>;

fn ensure(condition: bool, operation: &'static str, detail: impl FnOnce() -> String) -> Check {
    if condition {
        Ok(())
    } else {
        Err((operation, detail()))
    }
}
fn bits(value: Option<f64>) -> Option<u64> {
    value.map(f64::to_bits)
}
fn channel_bits(value: &Color) -> [Option<u64>; 3] {
    value.channels().map(bits)
}
fn normalized_hue(hue: Option<f64>) -> bool {
    hue.is_none_or(|hue| (0.0..360.0).contains(&hue))
}

/// The numerical API under test. Each operation is one request of the single
/// numerical path; `request` names the same operation as a computation.
mod api {
    use super::*;

    pub(super) fn finite(value: f64) -> Finite {
        Finite::new(value).expect("generated operands are finite")
    }
    pub(super) fn unit(value: f64) -> UnitInterval {
        UnitInterval::new(value).expect("generated unit operands lie in zero to one")
    }
    pub(super) fn channels(sample: Sample) -> [Option<Finite>; 3] {
        sample.channels.map(|channel| channel.map(finite))
    }
    pub(super) fn construct(sample: Sample) -> Color {
        Color::new(
            SPACES[sample.space],
            channels(sample),
            sample.alpha.map(unit),
        )
    }
    pub(super) fn method(case: &Case) -> InterpolationMethod {
        InterpolationMethod::try_new(
            SPACES[case.method_space],
            case.hue.checked_sub(1).map(|index| HUES[index]),
        )
        .expect("generated methods pair hue with polar spaces")
    }
    pub(super) fn transformation(case: &Case) -> Transformation {
        match case.transformation {
            0 => Transformation::ForceOpaqueAlpha,
            1 => Transformation::Convert(SPACES[case.transformation_space]),
            2 => Transformation::QuantizeSrgb,
            _ => Transformation::ColorFunction,
        }
    }
    /// The operations of one case, by stable kebab name.
    pub(super) fn operations(case: &Case) -> Vec<(&'static str, Color, NumericalRequest)> {
        let first = construct(case.first);
        let second = construct(case.second);
        let destination = SPACES[case.destination];
        let alpha = case.alpha.map(unit);
        let transformation = transformation(case);
        let progress = unit(case.progress);
        vec![
            (
                "construct",
                first,
                NumericalRequest::Construct {
                    space: SPACES[case.first.space],
                    channels: channels(case.first),
                    alpha: case.first.alpha.map(unit),
                },
            ),
            (
                "convert",
                first.converted(destination),
                NumericalRequest::Convert {
                    value: first,
                    destination,
                },
            ),
            (
                "prepare",
                first.prepared_for_interpolation(destination),
                NumericalRequest::Prepare {
                    value: first,
                    destination,
                },
            ),
            (
                "with-alpha",
                first.with_alpha(alpha),
                NumericalRequest::WithAlpha {
                    value: first,
                    alpha,
                },
            ),
            (
                "with-color-function",
                first.with_color_function(),
                NumericalRequest::WithColorFunction { value: first },
            ),
            (
                "to-legacy-srgb",
                first.to_legacy_srgb(),
                NumericalRequest::ToLegacySrgb { value: first },
            ),
            (
                "black-white-contrast",
                first.black_white_contrast(),
                NumericalRequest::BlackWhiteContrast { value: first },
            ),
            (
                "transform",
                first.transformed(transformation),
                NumericalRequest::Transform {
                    value: first,
                    transformation,
                },
            ),
            (
                "interpolate",
                first.interpolated(second, method(case), progress),
                NumericalRequest::Interpolate {
                    first,
                    second,
                    method: method(case),
                    progress,
                },
            ),
        ]
    }
    /// Executes the request through the controlled computation.
    pub(super) fn compute(request: NumericalRequest) -> Color {
        let mut owner = OperationOwner::new();
        Computation::new(&mut owner, request).compute()
    }
    pub(super) fn rectangular(space: Space) -> RectangularSpace {
        match space {
            Space::Predefined(space) => RectangularSpace::Predefined(space),
            Space::Oklab | Space::Oklch => RectangularSpace::Oklab,
            _ => RectangularSpace::Lab,
        }
    }
}

/// Color 4 §11.2 makes a cross-space result's channels present; polar and
/// cylindrical results keep a nonnegative radius and a hue in `[0, 360)`
/// (§9.5, §7.2) that only powerlessness may make missing.
fn converted_shape(operation: &'static str, value: &Color) -> Check {
    let [first, second, third] = value.channels();
    let ok = match value.space() {
        Space::Predefined(_) | Space::Lab | Space::Oklab => {
            first.is_some() && second.is_some() && third.is_some()
        }
        Space::Lch | Space::Oklch => {
            first.is_some() && second.is_some_and(|chroma| chroma >= 0.0) && normalized_hue(third)
        }
        Space::Hsl => {
            second.is_some_and(|saturation| saturation >= 0.0)
                && third.is_some()
                && normalized_hue(first)
        }
        Space::Hwb => second.is_some() && third.is_some() && normalized_hue(first),
        _ => false,
    };
    ensure(ok, operation, || format!("cross-space shape {value:?}"))
}

/// Spec-defined invariants of each `Color` operation.
fn operation_invariants(case: &Case) -> Check {
    let first = api::construct(case.first);
    let second = api::construct(case.second);
    let destination = SPACES[case.destination];
    let srgb = Space::Predefined(Predefined::Srgb);
    ensure(
        channel_bits(&first) == case.first.channels.map(bits)
            && bits(first.alpha()) == bits(case.first.alpha),
        "construct",
        || format!("construction changed operand bits {first:?}"),
    )?;
    // Conversion and preparation keep alpha, reach the destination, and are
    // the identity within one space.
    for (operation, value) in [
        ("convert", first.converted(destination)),
        ("prepare", first.prepared_for_interpolation(destination)),
    ] {
        ensure(
            value.space() == destination && bits(value.alpha()) == bits(first.alpha()),
            operation,
            || format!("space or alpha changed {value:?}"),
        )?;
        if destination == first.space() {
            ensure(value.identical(first), operation, || {
                format!("same-space result differs {value:?}")
            })?;
        } else if operation == "convert" {
            converted_shape(operation, &value)?;
        }
    }
    let alpha = case.alpha.map(api::unit);
    let replaced = first.with_alpha(alpha);
    ensure(
        channel_bits(&replaced) == channel_bits(&first)
            && bits(replaced.alpha()) == bits(case.alpha)
            && replaced.space() == first.space(),
        "with-alpha",
        || format!("{replaced:?}"),
    )?;
    let modern = first.with_color_function();
    ensure(
        channel_bits(&modern) == channel_bits(&first)
            && modern.serialization_family() == SerializationFamily::ColorFunction,
        "with-color-function",
        || format!("{modern:?}"),
    )?;
    let legacy = first.to_legacy_srgb();
    ensure(
        legacy.space() == srgb
            && legacy.serialization_family() == SerializationFamily::LegacySrgb
            && channel_bits(&legacy) == channel_bits(&first.converted(srgb))
            && bits(legacy.alpha()) == bits(first.alpha()),
        "to-legacy-srgb",
        || format!("{legacy:?}"),
    )?;
    // Color 5 contrast-color() picks opaque white or black.
    let contrast = first.black_white_contrast();
    ensure(
        matches!(contrast.srgb_bytes(), Some([0, 0, 0] | [255, 255, 255]))
            && contrast.alpha_byte() == Some(255),
        "black-white-contrast",
        || format!("{contrast:?}"),
    )?;
    let transformed = first.transformed(api::transformation(case));
    let expected = match api::transformation(case) {
        Transformation::ForceOpaqueAlpha => {
            channel_bits(&transformed) == channel_bits(&first) && transformed.alpha() == Some(1.0)
        }
        Transformation::Convert(space) => transformed.identical(first.converted(space)),
        Transformation::QuantizeSrgb => {
            transformed.space() == srgb
                && transformed.srgb_bytes().is_some()
                    == first.converted(srgb).channels().iter().all(Option::is_some)
                && bits(transformed.alpha()) == bits(first.alpha())
        }
        _ => transformed.identical(modern),
    };
    ensure(expected, "transform", || format!("{transformed:?}"))?;
    interpolation_endpoints(case, first, second)
}

/// Color 4 §12.3 at progress zero or one, with opaque present operands, gives
/// the prepared endpoint's non-hue channels exactly.
fn interpolation_endpoints(case: &Case, first: Color, second: Color) -> Check {
    let method = api::method(case);
    let mixed = first.interpolated(second, method, api::unit(case.progress));
    ensure(mixed.space() == method.space(), "interpolate", || {
        format!("space {mixed:?}")
    })?;
    let a = first.prepared_for_interpolation(method.space());
    let b = second.prepared_for_interpolation(method.space());
    let complete = |value: &Color| value.channels().iter().all(Option::is_some);
    if !(case.progress == 0.0 || case.progress == 1.0)
        || first.alpha() != Some(1.0)
        || second.alpha() != Some(1.0)
        || !complete(&a)
        || !complete(&b)
    {
        return Ok(());
    }
    let endpoint = if case.progress == 1.0 { b } else { a };
    let hue = match method.space() {
        Space::Hsl | Space::Hwb => Some(0),
        Space::Lch | Space::Oklch => Some(2),
        _ => None,
    };
    for index in (0..3).filter(|index| Some(*index) != hue) {
        ensure(
            mixed.channels()[index] == endpoint.channels()[index],
            "interpolate",
            || format!("endpoint {endpoint:?} mixed {mixed:?}"),
        )?;
    }
    ensure(mixed.alpha() == Some(1.0), "interpolate", || {
        format!("endpoint alpha {mixed:?}")
    })
}

fn computation_matches_direct(case: &Case) -> Check {
    for (operation, direct, request) in api::operations(case) {
        let computed = api::compute(request);
        if !computed.identical(direct) {
            return Err((
                operation,
                format!("computation {computed:?} differs from direct {direct:?}"),
            ));
        }
    }
    Ok(())
}

/// Spec-defined invariants of the standalone conversions.
fn standalone_invariants(case: &Case) -> Check {
    let given = api::channels(case.first);
    let zero = api::finite(0.0);
    let [l, a, b] = given.map(|channel| channel.unwrap_or(zero));
    // Color 4 §11.2: identity within a space, present channels across spaces.
    let source = api::rectangular(SPACES[case.first.space]);
    let destination = api::rectangular(SPACES[case.destination]);
    let converted = RectangularCoordinates::new(source, given).convert_to(destination);
    let plain =
        |channels: [Option<Finite>; 3]| channels.map(|channel| bits(channel.map(Finite::get)));
    ensure(
        if source == destination {
            plain(converted.channels()) == plain(given)
        } else {
            converted.channels().iter().all(Option::is_some)
        },
        "rectangular-convert",
        || format!("{converted:?}"),
    )?;
    // §9.5: L copied, nonnegative chroma, normalized hue missing exactly when
    // neutral cleanup zeroes the chroma.
    for (operation, polar) in [
        ("lab-to-lch", convert_lab_to_lch([l, a, b])),
        ("oklab-to-oklch", convert_oklab_to_oklch([l, a, b])),
    ] {
        let chroma = polar.chroma().get();
        let hue = polar.hue_degrees().map(Finite::get);
        ensure(
            polar.lightness().get().to_bits() == l.get().to_bits()
                && chroma >= 0.0
                && hue.is_none() == (chroma.to_bits() == 0)
                && normalized_hue(hue),
            operation,
            || format!("{polar:?}"),
        )?;
    }
    // §9.6: L copied; missing hue gives positive-zero axes; |a|, |b| <= C.
    let polar = PolarCoordinates::new(l, a, given[2]);
    ensure(
        polar.chroma().get() >= 0.0 && normalized_hue(polar.hue_degrees().map(Finite::get)),
        "polar-carrier",
        || format!("{polar:?}"),
    )?;
    for (operation, [lightness, x, y]) in [
        ("lch-to-lab", convert_lch_to_lab(&polar)),
        ("oklch-to-oklab", convert_oklch_to_oklab(&polar)),
    ] {
        let chroma = polar.chroma().get();
        ensure(
            lightness.get().to_bits() == l.get().to_bits()
                && x.get().abs() <= chroma
                && y.get().abs() <= chroma
                && (polar.hue_degrees().is_some()
                    || (x.get().to_bits() == 0 && y.get().to_bits() == 0)),
            operation,
            || format!("{polar:?} -> {x:?} {y:?}"),
        )?;
    }
    // §7.2 and §8.2: nonnegative saturation and normalized hue.
    let hsl = convert_srgb_to_hsl([l, a, b]);
    ensure(
        hsl.saturation().get() >= 0.0 && normalized_hue(hsl.hue_degrees().map(Finite::get)),
        "srgb-to-hsl",
        || format!("{hsl:?}"),
    )?;
    let hwb = convert_srgb_to_hwb([l, a, b]);
    ensure(
        normalized_hue(hwb.hue_degrees().map(Finite::get)),
        "srgb-to-hwb",
        || format!("{hwb:?}"),
    )?;
    // §7.1: zero saturation is gray; §8.1: W + B >= 100% is gray.
    let carrier = HslCoordinates::new(given[0], a, b);
    let [red, green, blue] = convert_hsl_to_srgb(&carrier).map(Finite::get);
    ensure(
        carrier.saturation().get() >= 0.0
            && (carrier.saturation().get() != 0.0 || (red == green && green == blue)),
        "hsl-to-srgb",
        || format!("{carrier:?} -> {red} {green} {blue}"),
    )?;
    let carrier = HwbCoordinates::new(given[0], a, b);
    let [red, green, blue] = convert_hwb_to_srgb(&carrier).map(Finite::get);
    ensure(
        a.get() / 100.0 + b.get() / 100.0 < 1.0 || (red == green && green == blue),
        "hwb-to-srgb",
        || format!("{carrier:?} -> {red} {green} {blue}"),
    )?;
    // Color 5 §6.1: nonnegative naive channels, alpha copied bit-for-bit,
    // zero inks for a zero maximum.
    let rgba = naively_convert_cmyk_to_srgba(case.cmyka.map(api::finite));
    ensure(
        rgba[..3].iter().all(|channel| channel.get() >= 0.0)
            && rgba[3].get().to_bits() == case.cmyka[4].to_bits(),
        "cmyk-to-srgba",
        || format!("{rgba:?}"),
    )?;
    let cmyka = naively_convert_srgba_to_cmyk(case.rgba.map(api::finite));
    let maximum = case.rgba[0].max(case.rgba[1]).max(case.rgba[2]);
    ensure(
        cmyka[4].get().to_bits() == case.rgba[3].to_bits()
            && (maximum != 0.0 || cmyka[..3].iter().all(|ink| ink.get() == 0.0)),
        "srgba-to-cmyk",
        || format!("{cmyka:?}"),
    )
}

fn run(seed: u64) {
    let target = Target::new();
    let limits = target.descriptor().limits();
    let report = run_property(
        &target,
        &bytes(ENTROPY_BYTES, limits),
        PropertyConfig {
            cases: CASES,
            max_rejections: 0,
            max_shrink_iterations: 512,
            seed,
        },
        &mut |_| Ok(()),
    )
    .expect("property run completes with sufficient coverage");
    if let Some(failure) = report.failure {
        let case = target.decode(&failure.canonical).expect("canonical case");
        panic!(
            "{:?}: {}\ncase: {case:#?}",
            failure.outcome.failure_key(),
            failure.outcome.detail().unwrap_or_default()
        );
    }
    assert!(report.counts.evaluated() >= u64::from(CASES));
}

#[test]
fn every_numerical_operation_keeps_its_spec_invariants_on_one_path() {
    run(0x5eed_c01a);
}

#[test]
fn generated_cases_replay_through_their_canonical_bytes() {
    let target = Target::new();
    for entropy in [
        vec![],
        vec![2; ENTROPY_BYTES],
        (0..=255).collect::<Vec<u8>>(),
    ] {
        let Generation::Generated(case) = target.generate(&entropy).unwrap() else {
            panic!("generation never discards");
        };
        let canonical = target.encode(&case).unwrap();
        assert_eq!(target.decode(&canonical).unwrap(), case);
    }
}
