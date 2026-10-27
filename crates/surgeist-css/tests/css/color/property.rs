//! Generated colour sources: every source has an expected outcome, and the
//! fronts and encodings that resolve it agree.
//!
//! CSS Values 4 §10.9.1 censors a top-level infinity to the context's
//! maximum and NaN to zero, and §10.12 with the numeric-types range rule
//! makes a finite value beyond binary64 the closest supported value. CSS
//! Color 4 conversions over those finite channels are total. So every
//! colour source, however extreme its literals and `calc()` values, resolves
//! to a used colour, a standard failure, or a reached unavailable,
//! unsupported or fact outcome; any other error fails. Its UTF-8 and UTF-16
//! spellings resolve to identical bits, and authored resolution of the
//! retained `color` value agrees with the source front (one CSS resolver).
//! The case generator and codec belong to this crate; `surgeist-fuzz`
//! supplies the bounded proptest runner.

use surgeist_color::Color;
use surgeist_css::CssParser;
use surgeist_css::color::{
    CssAuthoredColorOrdinaryLimits, CssAuthoredColorOutcome, CssColorFactIdentity, CssColorFacts,
    CssColorGrammarRequest, CssColorInput, CssColorOrdinaryLimits, CssColorOrdinaryOutcome,
    CssColorPaletteEntry, CssColorPropertyFacts, CssColorResolveError, CssColorSchemeFact,
    CssDeviceColorFact, CssSystemColor, parse_used_color,
};
use surgeist_css::declaration::CssImportance;
use surgeist_css::property::{
    CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef, CssPropertyTextTarget,
};
use surgeist_fuzz::engine::{PropertyConfig, run_property};
use surgeist_fuzz::generation::property::bytes;
use surgeist_fuzz::{
    BuildRequirements, CaseFormat, CaseOutcome, CaseTarget, FailureClass, FailureKey, FuzzError,
    FuzzErrorKind, Generation, GeneratorIdentity, InputEncoding, InputLimits, PropertyId, Result,
    TargetDescriptor, TargetDescriptorSpec, TargetId,
};

const OWNER: &str = "surgeist-css";
const CASES: u32 = 32;
const ENTROPY_BYTES: usize = 160;
const SOURCE_BYTES: usize = 8192;

/// Extreme literals, overflowing and infinite calculations, NaN, `none`
/// and percentages, including values that are invalid in some slots.
///
/// A literal beyond binary64 (`1e999`) is already the extreme `±f64::MAX`
/// (#1297), and exact conversion of an in-range 300-digit literal such as
/// `1e308` costs seconds of unoptimized work per occurrence, so the
/// in-range extremes are named regression cases instead of generated ones.
const VALUES: [&str; 25] = [
    "0",
    "-0",
    "1",
    "0.5",
    "none",
    "1e999",
    "-1e999",
    "1e-400",
    "-1e-400",
    "calc(infinity)",
    "calc(-infinity)",
    "calc(NaN)",
    "calc(1 / 0)",
    "calc(-1 / 0)",
    "calc(1e999 * 10)",
    "calc(-1e999 / 1e-400)",
    "calc(exp(710))",
    "calc(-1 * exp(710))",
    "calc(calc(1e999 * 2) - 1e999)",
    "calc(1e999 * 1e999 / 1e-400)",
    "1e999%",
    "-1e999%",
    "calc(infinity * 1%)",
    "calc(-1e999% * 10)",
    "50%",
];
const PERCENTAGES: [&str; 6] = ["0%", "50%", "100%", "1e-400%", "33.3%", "calc(1e-400% * 1)"];
const LEAVES: [&str; 6] = [
    "red",
    "#0f08",
    "currentcolor",
    "canvas",
    "transparent",
    "rgb(1e999 -1e999 calc(infinity))",
];
const FUNCTIONS: [(&str, [&str; 3]); 8] = [
    ("rgb", ["r", "g", "b"]),
    ("hsl", ["h", "s", "l"]),
    ("hwb", ["h", "w", "b"]),
    ("lab", ["l", "a", "b"]),
    ("lch", ["l", "c", "h"]),
    ("oklab", ["l", "a", "b"]),
    ("oklch", ["l", "c", "h"]),
    ("color", ["r", "g", "b"]),
];
const PREDEFINED: [&str; 12] = [
    "srgb",
    "srgb-linear",
    "display-p3",
    "display-p3-linear",
    "a98-rgb",
    "prophoto-rgb",
    "rec2020",
    "xyz",
    "xyz-d50",
    "xyz-d65",
    "srgb",
    "rec2020",
];
const MIX_SPACES: [&str; 16] = [
    "srgb",
    "srgb-linear",
    "display-p3",
    "display-p3-linear",
    "a98-rgb",
    "prophoto-rgb",
    "rec2020",
    "xyz",
    "xyz-d50",
    "xyz-d65",
    "hsl",
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
];
const HUES: [&str; 5] = [
    "",
    " shorter hue",
    " longer hue",
    " increasing hue",
    " decreasing hue",
];

/// Reads generation entropy; exhausted entropy reads as zero bytes.
struct Entropy<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl Entropy<'_> {
    fn index(&mut self, count: usize) -> usize {
        let value = self.bytes.get(self.offset).copied().unwrap_or(0);
        self.offset += 1;
        usize::from(value) % count
    }
    fn value(&mut self) -> &'static str {
        VALUES[self.index(VALUES.len())]
    }
    /// A relative channel: its keyword, a scaled or negated keyword, or a value.
    fn relative(&mut self, keyword: &str) -> String {
        match self.index(5) {
            0 => keyword.to_owned(),
            1 => format!("calc({keyword} * -1)"),
            2 => format!("calc({keyword} * {})", self.value()),
            3 => format!("calc({keyword} + {})", self.value()),
            _ => self.value().to_owned(),
        }
    }
    fn alpha(&mut self) -> String {
        match self.index(4) {
            0 => String::new(),
            1 => format!(" / {}", self.value()),
            2 => " / 50%".to_owned(),
            _ => " / none".to_owned(),
        }
    }
    fn color(&mut self, depth: usize) -> String {
        let kinds = if depth == 0 { 3 } else { 8 };
        match self.index(kinds) {
            0 => self.absolute(),
            1 => LEAVES[self.index(LEAVES.len())].to_owned(),
            2 => self.predefined(),
            3 => self.relative_color(depth - 1),
            4 => self.mix(depth - 1),
            5 => format!("contrast-color({})", self.color(depth - 1)),
            6 => format!(
                "light-dark({}, {})",
                self.color(depth - 1),
                self.color(depth - 1)
            ),
            _ => format!(
                "device-cmyk({} {} {} {}{})",
                self.value(),
                self.value(),
                self.value(),
                self.value(),
                self.alpha()
            ),
        }
    }
    fn absolute(&mut self) -> String {
        let (function, _) = FUNCTIONS[self.index(FUNCTIONS.len() - 1)];
        format!(
            "{function}({} {} {}{})",
            self.value(),
            self.value(),
            self.value(),
            self.alpha()
        )
    }
    fn predefined(&mut self) -> String {
        format!(
            "color({} {} {} {}{})",
            PREDEFINED[self.index(PREDEFINED.len())],
            self.value(),
            self.value(),
            self.value(),
            self.alpha()
        )
    }
    fn relative_color(&mut self, depth: usize) -> String {
        let (function, keywords) = FUNCTIONS[self.index(FUNCTIONS.len())];
        let origin = self.color(depth);
        let (function, keywords) = if function == "color" {
            let space = PREDEFINED[self.index(PREDEFINED.len())];
            let keywords = if space.starts_with("xyz") {
                ["x", "y", "z"]
            } else {
                keywords
            };
            (format!("color(from {origin} {space}"), keywords)
        } else {
            (format!("{function}(from {origin}"), keywords)
        };
        let alpha = match self.index(3) {
            0 => String::new(),
            1 => " / alpha".to_owned(),
            _ => format!(" / {}", self.relative("alpha")),
        };
        format!(
            "{function} {} {} {}{alpha})",
            self.relative(keywords[0]),
            self.relative(keywords[1]),
            self.relative(keywords[2]),
        )
    }
    fn mix(&mut self, depth: usize) -> String {
        let space = MIX_SPACES[self.index(MIX_SPACES.len())];
        let hue = if matches!(space, "hsl" | "hwb" | "lch" | "oklch") {
            HUES[self.index(HUES.len())]
        } else {
            ""
        };
        let first = self.color(depth);
        let first_percentage = self.percentage();
        let second = self.color(depth);
        let second_percentage = self.percentage();
        format!(
            "color-mix(in {space}{hue}, {first}{first_percentage}, {second}{second_percentage})"
        )
    }
    fn percentage(&mut self) -> String {
        match self.index(3) {
            0 => String::new(),
            _ => format!(" {}", PERCENTAGES[self.index(PERCENTAGES.len())]),
        }
    }
}

const IDENTITY: CssColorFactIdentity = CssColorFactIdentity::new(17, 2);
const PALETTE: &[CssColorPaletteEntry] = &[
    CssColorPaletteEntry::new(
        CssSystemColor::Canvas,
        CssColorFactIdentity::new(18, 1),
        Color::from_srgba8([250, 251, 252, 255]),
    ),
    CssColorPaletteEntry::new(
        CssSystemColor::CanvasText,
        CssColorFactIdentity::new(18, 2),
        Color::from_srgba8([16, 32, 48, 255]),
    ),
];
/// Current colour, a light scheme and the palette, so keyword branches reach
/// their colours; device colours take the naive fallback.
fn facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        IDENTITY,
        CssColorPropertyFacts::Current(Color::from_srgba8([0, 128, 0, 200])),
        CssColorSchemeFact::Light,
        PALETTE,
        &[],
        &[],
        CssDeviceColorFact::Absent,
    )
}
const ORDINARY_LIMITS: CssColorOrdinaryLimits = CssColorOrdinaryLimits {
    input_units: SOURCE_BYTES,
    algorithm_records: 1 << 20,
    arithmetic_records: 1 << 18,
    evaluation_records: 1 << 18,
    actions: 1 << 30,
};
const AUTHORED_LIMITS: CssAuthoredColorOrdinaryLimits = CssAuthoredColorOrdinaryLimits {
    path_steps: 1 << 16,
    algorithm_records: 1 << 20,
    arithmetic_records: 1 << 18,
    evaluation_records: 1 << 18,
    actions: 1 << 30,
};

fn property(name: &str) -> PropertyId {
    PropertyId::new(format!("{OWNER}::{name}")).expect("owner-qualified property")
}
/// Every error is a reached fact, capability or envelope outcome.
const EXPECTED_OUTCOMES: &str = "expected-outcomes";
/// UTF-8 and UTF-16 spellings of the same source resolve identically.
const ENCODINGS_AGREE: &str = "encodings-agree";
/// The source front and authored resolution of the retained value agree.
const FRONTS_AGREE: &str = "fronts-agree";
const PROPERTIES: [&str; 3] = [EXPECTED_OUTCOMES, ENCODINGS_AGREE, FRONTS_AGREE];

struct Target {
    descriptor: TargetDescriptor,
}
impl Target {
    fn new() -> Self {
        let descriptor = TargetDescriptor::new(TargetDescriptorSpec {
            id: TargetId::new(format!("{OWNER}::color-sources")).unwrap(),
            owner: OWNER.into(),
            binary: "surgeist-css-color-property".into(),
            properties: PROPERTIES.map(property).to_vec(),
            source_references: vec![
                "https://www.w3.org/TR/2024/WD-css-values-4-20240312/#top-level-calculation".into(),
                "https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-range".into(),
                "https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#color-conversion".into(),
            ],
            format: CaseFormat::new(format!("{OWNER}::color-source"), 1).unwrap(),
            generator_identity: GeneratorIdentity::new(format!("{OWNER}::color-recipe"), 1)
                .unwrap(),
            encoding: InputEncoding::RawBytes,
            limits: InputLimits::new(SOURCE_BYTES as u64, 0, 0, 0, 0).unwrap(),
            requirements: BuildRequirements::new(vec![], vec![]).unwrap(),
            min_evaluated: u64::from(CASES),
            max_generation_rejections: 0,
        })
        .unwrap();
        Self { descriptor }
    }
}
impl CaseTarget for Target {
    type Case = String;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<String> {
        String::from_utf8(input.to_vec()).map_err(|_| {
            FuzzError::new(FuzzErrorKind::InvalidCase, "decode colour source", "UTF-8")
        })
    }
    fn encode(&self, case: &String) -> Result<Vec<u8>> {
        Ok(case.as_bytes().to_vec())
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<String>> {
        let mut entropy = Entropy {
            bytes: input,
            offset: 0,
        };
        Ok(Generation::Generated(entropy.color(2)))
    }
    fn evaluate(&self, source: &String) -> Result<CaseOutcome> {
        let facts = facts();
        let utf8 = used(CssColorInput::Utf8(source.as_bytes()), &facts);
        let units = source.encode_utf16().collect::<Vec<_>>();
        let utf16 = used(CssColorInput::Utf16(&units), &facts);
        let authored = authored(source, &facts);
        for outcome in [&utf8, &utf16].into_iter().chain(authored.as_ref()) {
            if let Outcome::Unexpected(detail) = outcome {
                return failure(EXPECTED_OUTCOMES, "unexpected-error", detail.clone());
            }
        }
        if !utf8.agrees(&utf16) {
            return failure(
                ENCODINGS_AGREE,
                "utf16-differs",
                format!("utf-8 {utf8:?} utf-16 {utf16:?}"),
            );
        }
        if let Some(authored) = authored
            && !utf8.agrees(&authored)
        {
            return failure(
                FRONTS_AGREE,
                "authored-differs",
                format!("source {utf8:?} authored {authored:?}"),
            );
        }
        CaseOutcome::evaluated(PROPERTIES.map(property).to_vec())
    }
}
fn failure(name: &str, operation: &str, detail: String) -> Result<CaseOutcome> {
    let mut detail = detail.replace(char::is_control, " ");
    detail.truncate(2048);
    CaseOutcome::failure(
        FailureKey::new(
            property(name),
            FailureClass::PropertyViolation,
            Some(operation.into()),
        )?,
        detail.trim().to_owned(),
    )
}

/// One front's observable outcome.
#[derive(Debug)]
enum Outcome {
    Used(Color),
    StandardFailure,
    /// A reached unavailable, unsupported or fact-envelope outcome.
    Expected(String),
    /// Anything else, including a resource limit or protocol error.
    Unexpected(String),
}
impl Outcome {
    /// Identical colour bits, or the same non-colour outcome.
    fn agrees(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Used(a), Self::Used(b)) => a.identical(*b),
            (Self::StandardFailure, Self::StandardFailure) => true,
            (Self::Expected(a), Self::Expected(b)) => a == b,
            _ => false,
        }
    }
}
/// The generous limits are never reached, so any error but a reached fact,
/// capability or envelope outcome violates the contract.
fn classify(error: &CssColorResolveError) -> Outcome {
    let reached = error.color_error().is_some_and(|error| {
        error.unavailable().is_some()
            || error.unsupported().is_some()
            || error.fact_defect().is_some()
    });
    if reached {
        Outcome::Expected(error.to_string())
    } else {
        Outcome::Unexpected(error.to_string())
    }
}
fn used(input: CssColorInput<'_>, facts: &CssColorFacts<'_>) -> Outcome {
    match parse_used_color(
        input,
        CssColorGrammarRequest::WholeColor,
        facts,
        facts.identity(),
        ORDINARY_LIMITS,
    ) {
        Ok(result) => match result.outcome() {
            CssColorOrdinaryOutcome::Used(value) => Outcome::Used(value),
            CssColorOrdinaryOutcome::StandardFailure(_) => Outcome::StandardFailure,
            other => Outcome::Unexpected(format!("{other:?}")),
        },
        Err(error) => classify(&error),
    }
}
/// Authored resolution of the retained `color` value, when the source is one.
fn authored(source: &str, facts: &CssColorFacts<'_>) -> Option<Outcome> {
    let report = CssParser::default().parse_property_value(
        source,
        CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
        CssImportance::Normal,
    );
    let declaration = report.syntax().as_ref()?;
    let Some(CssKnownPropertyValueRef::Color(value)) =
        declaration.known().and_then(|known| known.property_value())
    else {
        return None;
    };
    Some(
        match value
            .value()
            .resolve(facts, facts.identity(), AUTHORED_LIMITS)
        {
            Ok(result) => match result.outcome() {
                CssAuthoredColorOutcome::Used(value) => Outcome::Used(*value),
                CssAuthoredColorOutcome::StandardFailure(_) => Outcome::StandardFailure,
                other => Outcome::Unexpected(format!("{other:?}")),
            },
            Err(error) => classify(&error),
        },
    )
}

fn run(seed: u64) {
    let target = Target::new();
    let report = run_property(
        &target,
        &bytes(ENTROPY_BYTES, target.descriptor().limits()),
        PropertyConfig {
            cases: CASES,
            max_rejections: 0,
            max_shrink_iterations: 96,
            seed,
        },
        &mut |_| Ok(()),
    )
    .expect("property run completes with sufficient coverage");
    if let Some(failure) = report.failure {
        panic!(
            "{:?}: {}\nsource: {}",
            failure.outcome.failure_key(),
            failure.outcome.detail().unwrap_or_default(),
            String::from_utf8_lossy(&failure.canonical)
        );
    }
    assert!(report.counts.evaluated() >= u64::from(CASES));
}

#[test]
fn generated_color_sources_have_expected_outcomes_and_agree_across_fronts() {
    run(0x00c0_10f5);
}

/// The used colour of `source` through both production fronts, which must
/// agree: the ordinary source front and authored resolution of the retained
/// `color` value.
fn resolved(source: &str) -> Color {
    let facts = facts();
    let used = match parse_used_color(
        CssColorInput::Utf8(source.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        ORDINARY_LIMITS,
    )
    .map(|result| result.outcome())
    {
        Ok(CssColorOrdinaryOutcome::Used(value)) => value,
        other => panic!("{source}: ordinary outcome {other:?}"),
    };
    let report = CssParser::default().parse_property_value(
        source,
        CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
        CssImportance::Normal,
    );
    let Some(declaration) = report.syntax() else {
        panic!("{source}: a declaration");
    };
    let Some(CssKnownPropertyValueRef::Color(value)) =
        declaration.known().and_then(|known| known.property_value())
    else {
        panic!("{source}: a typed color declaration");
    };
    let authored = match value
        .value()
        .resolve(&facts, facts.identity(), AUTHORED_LIMITS)
    {
        Ok(result) => match result.outcome() {
            CssAuthoredColorOutcome::Used(value) => *value,
            other => panic!("{source}: authored outcome {other:?}"),
        },
        Err(error) => panic!("{source}: authored error {error:?}"),
    };
    assert!(
        used.identical(authored),
        "{source}: fronts disagree {used:?} {authored:?}"
    );
    used
}

const MAX: f64 = f64::MAX;

#[test]
fn an_infinite_unbounded_channel_calculation_is_the_largest_finite_value() {
    // Values 4 §10.9.1: infinities do not escape a top-level calculation;
    // they clamp to the largest value the context allows, and Lab a/b and
    // color() channels are unbounded, so that is f64::MAX.
    let lab = resolved("lab(50 calc(infinity) 0)");
    assert_eq!(lab.space(), surgeist_color::ColorSpace::Lab);
    assert_eq!(lab.channels(), [Some(50.0), Some(MAX), Some(0.0)]);
    let srgb = resolved("color(srgb calc(-infinity) calc(1 / 0) 0)");
    assert_eq!(srgb.channels(), [Some(-MAX), Some(MAX), Some(0.0)]);
    assert_eq!(srgb.alpha(), Some(1.0));
}

#[test]
fn an_overflowing_finite_channel_calculation_is_the_closest_finite_value() {
    // Values 4 numeric-types range rule (#1297): a finite value beyond
    // binary64 is the closest supported value. exp(710) and 1e999 * 10 have
    // finite operands; the oklch percentage lightness clamps to 100%.
    let oklab = resolved("oklab(1 1 calc(exp(710)))");
    assert_eq!(oklab.channels(), [Some(1.0), Some(1.0), Some(MAX)]);
    let oklch = resolved("oklch(1e999% calc(1e999 * 10) 1)");
    assert_eq!(oklch.channels(), [Some(1.0), Some(MAX), Some(1.0)]);
}

#[test]
fn contrast_of_an_overflowing_color_uses_its_clamped_luminance() {
    // hsl() lightness 1e999 is f64::MAX; its XYZ luminance exceeds binary64
    // and clamps to one (Color 5 contrast-color), so black has more contrast.
    let black = resolved("contrast-color(hsl(0 50% 1e999))");
    assert_eq!(black.srgb_bytes(), Some([0, 0, 0]));
    assert_eq!(black.alpha(), Some(1.0));
}
