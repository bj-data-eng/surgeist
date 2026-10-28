//! Properties of the `An+B` microsyntax and of the rule drivers over generated
//! input.
//!
//! The case generators and the codec belong to this crate (`tests/common`);
//! `surgeist-fuzz` supplies the bounded, seeded runner. Every run uses a fixed
//! seed and a fixed number of cases, so a failure reproduces.
//!
//! - A1, serialization (section 9.1): the `An+B` serialization of any two
//!   `i32` reads back as the same two.
//! - A2, recognition (section 6.2): whatever notation-like text `anb::parse`
//!   accepts, its serialization is accepted and gives the same value, and
//!   parsing never panics.
//! - D1, totality: the stylesheet and block drivers end on every input, produce
//!   no more units than the document has nodes, report intervals inside the
//!   text and in order, and leave a stylesheet's stream at its end.
//! - D2, whole units (section 4.4 of the design, sections 5.5.2 and 5.5.3): what
//!   a grammar decides about a rule does not change what the driver consumed, so
//!   the extents of a stylesheet's units are the same for a grammar that accepts
//!   everything and one that rejects everything.
//! - D3, escaped punctuation: a block is handed to a grammar only when it is
//!   the contents of a group opened by an unescaped `{`, whatever the text does
//!   with `\{`, `\}` and the other escaped punctuation.

mod common;

use std::panic::{AssertUnwindSafe, catch_unwind};

use common::generate::text_of;
use surgeist_css_parser::anb::{self, AnPlusB};
use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, Declaration, QualifiedRule, RuleGrammar, Unit,
};
use surgeist_css_parser::lexer::Options;
use surgeist_css_parser::limits::{DepthExcess, Limits};
use surgeist_css_parser::{Document, Offset, Stream, SyntaxError};
use surgeist_fuzz::engine::{PropertyConfig, run_property};
use surgeist_fuzz::generation::property::bytes;
use surgeist_fuzz::{
    BuildRequirements, CaseFormat, CaseOutcome, CaseTarget, FailureClass, FailureKey, FuzzError,
    FuzzErrorKind, Generation, GeneratorIdentity, InputEncoding, InputLimits, PropertyId, Result,
    TargetDescriptor, TargetDescriptorSpec, TargetId,
};

const CASES: u32 = 1200;
const TEXT_BYTES: usize = 512;
const ENTROPY_BYTES: usize = 97;
const SEED: u64 = 0x1508_0007;

// ----------------------------------------------------------------- generators

/// Pieces of notations: the tokens of the production of section 6.2, near
/// misses of them, and the edges of `i32`.
const NOTATION: &[&str] = &[
    "n",
    "N",
    "-n",
    "+n",
    "+",
    "-",
    "odd",
    "even",
    "ODD",
    "1",
    "2",
    "23",
    "-5",
    "+7",
    "0",
    "2n",
    "-3n",
    "+4N",
    "n-",
    "n-1",
    "-n-2",
    "2n-",
    "2n-3",
    "n+",
    " ",
    "  ",
    "\n",
    "/**/",
    "2147483647",
    "2147483648",
    "-2147483648",
    "-2147483649",
    "99999999999999999999",
    ".5",
    "1e1",
    "a",
    "(",
    ")",
    "%",
    "x",
    "n1",
    "--n",
    "1.5n",
];

fn notation(entropy: &[u8]) -> String {
    let count = 1 + usize::from(entropy.first().copied().unwrap_or(0)) % 6;
    (0..count)
        .map(|index| {
            let byte = usize::from(entropy.get(1 + index).copied().unwrap_or(0));
            NOTATION[byte % NOTATION.len()]
        })
        .collect()
}

/// Two `i32`: from the edges of the type half the time.
fn pair(entropy: &[u8]) -> (i32, i32) {
    const EDGES: [i32; 9] = [
        0,
        1,
        -1,
        2,
        -2,
        i32::MAX,
        i32::MIN,
        i32::MAX - 1,
        i32::MIN + 1,
    ];
    let word = |at: usize| {
        let mut buffer = [0_u8; 4];
        for (slot, byte) in buffer.iter_mut().zip(entropy.iter().skip(at)) {
            *slot = *byte;
        }
        i32::from_le_bytes(buffer)
    };
    let pick = |selector: u8, at: usize| {
        if selector.is_multiple_of(2) {
            EDGES[usize::from(selector / 2) % EDGES.len()]
        } else {
            word(at)
        }
    };
    let selectors = (
        entropy.first().copied().unwrap_or(0),
        entropy.get(1).copied().unwrap_or(0),
    );
    (pick(selectors.0, 2), pick(selectors.1, 6))
}

// ----------------------------------------------------------------- the checks

fn parse_text(text: &str) -> std::result::Result<Document<'_>, String> {
    Document::<()>::parse(
        text,
        &Options::default(),
        Limits::default(),
        DepthExcess::Fail,
    )
    .map_err(|error| format!("{text:?}: {error}"))
}

/// A1.
fn check_serialization(a: i32, b: i32) -> std::result::Result<(), String> {
    let text = AnPlusB::new(a, b).to_string();
    let document = parse_text(&text)?;
    let mut stream = document.root();
    let value = anb::parse(&mut stream).map_err(|error| format!("{text:?}: {error}"))?;
    if (value.a(), value.b()) != (a, b) || !stream.is_empty() {
        return Err(format!(
            "({a}, {b}) serialized as {text:?} reads as ({}, {}) with {} left",
            value.a(),
            value.b(),
            if stream.is_empty() {
                "nothing"
            } else {
                "something"
            }
        ));
    }
    Ok(())
}

/// A2.
fn check_recognition(text: &str) -> std::result::Result<(), String> {
    let document = parse_text(text)?;
    let mut stream = document.root();
    let parsed = catch_unwind(AssertUnwindSafe(|| anb::parse(&mut stream)))
        .map_err(|_| format!("anb::parse panicked on {text:?}"))?;
    let Ok(value) = parsed else {
        return Ok(());
    };
    let written = value.to_string();
    let document = parse_text(&written)?;
    let mut stream = document.root();
    let again = anb::parse(&mut stream).map_err(|error| {
        format!(
            "{text:?} is ({}, {}), written as {written:?}: {error}",
            value.a(),
            value.b()
        )
    })?;
    if again != value || !stream.is_empty() {
        return Err(format!(
            "{text:?} is ({}, {}), written as {written:?}, which reads as ({}, {})",
            value.a(),
            value.b(),
            again.a(),
            again.b()
        ));
    }
    Ok(())
}

/// What a unit covered: its start and end offsets, or why there are none.
type Extent = (Option<Offset>, Option<Offset>);

/// A grammar that notes the extent of every unit it is handed, accepting or
/// rejecting each.
struct Extents {
    reject: bool,
    seen: Vec<Extent>,
}

impl Extents {
    fn answer<'i, T>(
        &mut self,
        start: surgeist_css_parser::Site<'i>,
        end: surgeist_css_parser::Site<'i>,
        item: T,
    ) -> std::result::Result<T, SyntaxError<'i, ()>> {
        self.seen.push((start.offset(), end.offset()));
        if self.reject {
            Err(SyntaxError::custom(start, ()))
        } else {
            Ok(item)
        }
    }
}

impl<'i> RuleGrammar<'i> for Extents {
    type Item = ();
    type Error = ();

    fn at_rule(&mut self, rule: AtRule<'i>) -> std::result::Result<(), SyntaxError<'i, ()>> {
        self.answer(rule.start(), rule.end(), ())
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i>,
    ) -> std::result::Result<(), SyntaxError<'i, ()>> {
        self.answer(rule.start(), rule.end(), ())
    }
}

/// D1.
fn check_totality(text: &str) -> std::result::Result<(), String> {
    let document = parse_text(text)?;
    let nodes = document.node_count();
    let end = Offset::new(text.len());
    let check_unit =
        |label: &str, start: Option<Offset>, finish: Option<Offset>, before: &mut Offset| {
            let (Some(start), Some(finish)) = (start, finish) else {
                return Err(format!("{text:?}: a {label} without offsets"));
            };
            if start > finish || finish > end || start < *before {
                return Err(format!(
                    "{text:?}: a {label} at {}..{} after {}",
                    start.value(),
                    finish.value(),
                    before.value()
                ));
            }
            *before = finish;
            Ok(())
        };
    for reject in [false, true] {
        let mut grammar = Extents {
            reject,
            seen: Vec::new(),
        };
        let mut sheet = document.root().stylesheet_contents(&mut grammar);
        let mut count = 0_usize;
        let mut before = Offset::new(0);
        for unit in sheet.by_ref() {
            count += 1;
            if count > nodes + 1 {
                return Err(format!("{text:?}: more sheet units than nodes"));
            }
            if let Err(failed) = unit {
                check_unit(
                    "sheet failure",
                    failed.start().offset(),
                    failed.end().offset(),
                    &mut before,
                )?;
            }
        }
        if !sheet.stream().is_empty() {
            return Err(format!("{text:?}: the sheet stream is not at its end"));
        }
        let mut blocks = BlockExtents { reject };
        let mut contents = document.root().block_contents(&mut blocks);
        let mut count = 0_usize;
        let mut before = Offset::new(0);
        for unit in contents.by_ref() {
            count += 1;
            if count > nodes + 1 {
                return Err(format!("{text:?}: more block units than nodes"));
            }
            if let Err(failed) = unit {
                check_unit(
                    "block failure",
                    failed.start().offset(),
                    failed.end().offset(),
                    &mut before,
                )?;
            }
        }
    }
    Ok(())
}

struct BlockExtents {
    reject: bool,
}

impl<'i> RuleGrammar<'i> for BlockExtents {
    type Item = ();
    type Error = ();

    fn at_rule(&mut self, rule: AtRule<'i>) -> std::result::Result<(), SyntaxError<'i, ()>> {
        self.verdict(rule.start())
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i>,
    ) -> std::result::Result<(), SyntaxError<'i, ()>> {
        self.verdict(rule.start())
    }
}

impl<'i> BlockGrammar<'i> for BlockExtents {
    fn declaration(
        &mut self,
        declaration: Declaration<'i>,
    ) -> std::result::Result<(), SyntaxError<'i, ()>> {
        self.verdict(declaration.start())
    }
}

impl BlockExtents {
    fn verdict<'i>(
        &self,
        at: surgeist_css_parser::Site<'i>,
    ) -> std::result::Result<(), SyntaxError<'i, ()>> {
        if self.reject {
            Err(SyntaxError::custom(at, ()))
        } else {
            Ok(())
        }
    }
}

/// D2.
fn check_whole_units(text: &str) -> std::result::Result<(), String> {
    let document = parse_text(text)?;
    let accepting = unit_extents(&document, false);
    let rejecting = unit_extents(&document, true);
    if accepting == rejecting {
        Ok(())
    } else {
        Err(format!(
            "{text:?}: accepting consumed {accepting:?}, rejecting {rejecting:?}"
        ))
    }
}

/// The extent of every unit of the stylesheet: the grammar's rules and the
/// driver's failures, in order.
fn unit_extents(document: &Document<'_>, reject: bool) -> Vec<(Option<Offset>, Option<Offset>)> {
    struct Recording {
        reject: bool,
    }
    impl<'i> RuleGrammar<'i> for Recording {
        type Item = Extent;
        type Error = ();

        fn at_rule(
            &mut self,
            rule: AtRule<'i>,
        ) -> std::result::Result<Extent, SyntaxError<'i, ()>> {
            let extent = (rule.start().offset(), rule.end().offset());
            if self.reject {
                Err(SyntaxError::custom(rule.start(), ()))
            } else {
                Ok(extent)
            }
        }

        fn qualified_rule(
            &mut self,
            rule: QualifiedRule<'i>,
        ) -> std::result::Result<Extent, SyntaxError<'i, ()>> {
            let extent = (rule.start().offset(), rule.end().offset());
            if self.reject {
                Err(SyntaxError::custom(rule.start(), ()))
            } else {
                Ok(extent)
            }
        }
    }
    let mut grammar = Recording { reject };
    document
        .root()
        .stylesheet_contents(&mut grammar)
        .map(|unit: Unit<'_, Extent, ()>| match unit {
            Ok(extent) => extent,
            Err(failed) => (failed.start().offset(), failed.end().offset()),
        })
        .collect()
}

/// Looks at the group a block belongs to: its opening token must be spelled
/// `{`, which an escaped brace is not.
struct Origin;

impl Origin {
    fn check<'i>(block: &Stream<'i>) -> std::result::Result<(), SyntaxError<'i, String>> {
        let site = block.start_site();
        let document = site.document();
        let opener = document
            .parent_group(site.list())
            .and_then(|node| document.spelling(node));
        if opener == Some("{") {
            Ok(())
        } else {
            Err(SyntaxError::custom(
                site,
                format!("a block of a group opened by {opener:?}"),
            ))
        }
    }
}

impl<'i> RuleGrammar<'i> for Origin {
    type Item = ();
    type Error = String;

    fn at_rule(&mut self, rule: AtRule<'i>) -> std::result::Result<(), SyntaxError<'i, String>> {
        match rule.block() {
            Some(block) => {
                Self::check(&block)?;
                self.units(block)
            }
            None => Ok(()),
        }
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i>,
    ) -> std::result::Result<(), SyntaxError<'i, String>> {
        let block = rule.block();
        Self::check(&block)?;
        self.units(block)
    }
}

impl<'i> BlockGrammar<'i> for Origin {
    fn declaration(
        &mut self,
        _: Declaration<'i>,
    ) -> std::result::Result<(), SyntaxError<'i, String>> {
        Ok(())
    }
}

impl Origin {
    /// Reads the block's contents too, so the check reaches every depth.
    fn units<'i>(&mut self, block: Stream<'i>) -> std::result::Result<(), SyntaxError<'i, String>> {
        for unit in block.block_contents(self) {
            if let Err(failed) = unit {
                let error = failed.into_error();
                if error.is_custom() {
                    return Err(error);
                }
            }
        }
        Ok(())
    }
}

/// D3.
fn check_escaped_punctuation(text: &str) -> std::result::Result<(), String> {
    let document = parse_text(text)?;
    let mut grammar = Origin;
    for unit in document.root().stylesheet_contents(&mut grammar) {
        if let Err(failed) = unit {
            let error = failed.into_error();
            if let Some(why) = error.custom_value() {
                return Err(format!("{text:?}: {why}"));
            }
        }
    }
    let mut grammar = Origin;
    for unit in document.root().block_contents(&mut grammar) {
        if let Err(failed) = unit {
            let error = failed.into_error();
            if let Some(why) = error.custom_value() {
                return Err(format!("{text:?}: {why}"));
            }
        }
    }
    Ok(())
}

// ----------------------------------------------------------------- the target

fn property(name: &str) -> PropertyId {
    PropertyId::new(format!("surgeist-css-parser::{name}")).expect("owner-qualified property")
}

const SERIALIZATION: &str = "anb-serialization-round-trip";
const RECOGNITION: &str = "anb-recognition-is-stable";
const TOTALITY: &str = "drivers-terminate-and-keep-order";
const WHOLE_UNITS: &str = "verdicts-do-not-change-extents";
const ESCAPED_PUNCTUATION: &str = "blocks-come-from-unescaped-braces";
const PROPERTIES: [&str; 5] = [
    SERIALIZATION,
    RECOGNITION,
    TOTALITY,
    WHOLE_UNITS,
    ESCAPED_PUNCTUATION,
];

/// One case: the entropy decides everything, so the case is the text of an
/// `An+B` pair, a notation, and a generated sheet, with the bytes that made them.
struct Target {
    descriptor: TargetDescriptor,
}

impl Target {
    fn new() -> Self {
        let descriptor = TargetDescriptor::new(TargetDescriptorSpec {
            id: TargetId::new("surgeist-css-parser::syntax-drivers").unwrap(),
            owner: "surgeist-css-parser".into(),
            binary: "surgeist-css-parser-syntax-property".into(),
            properties: PROPERTIES.map(property).to_vec(),
            source_references: vec![
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#anb-microsyntax".into(),
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#parser-algorithms".into(),
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#serializing-anb".into(),
            ],
            format: CaseFormat::new("surgeist-css-parser::entropy", 1).unwrap(),
            generator_identity: GeneratorIdentity::new("surgeist-css-parser::syntax-cases", 1)
                .unwrap(),
            encoding: InputEncoding::RawBytes,
            limits: InputLimits::new(TEXT_BYTES as u64, 0, 0, 0, 0).unwrap(),
            requirements: BuildRequirements::new(vec![], vec![]).unwrap(),
            min_evaluated: u64::from(CASES),
            max_generation_rejections: 0,
        })
        .unwrap();
        Self { descriptor }
    }
}

fn failure(name: &str, detail: String) -> Result<CaseOutcome> {
    let mut detail = detail.replace(char::is_control, " ");
    detail.truncate(2048);
    CaseOutcome::failure(
        FailureKey::new(property(name), FailureClass::PropertyViolation, None)?,
        detail.trim().to_owned(),
    )
}

impl CaseTarget for Target {
    type Case = Vec<u8>;

    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }

    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Err(FuzzError::new(
                FuzzErrorKind::InvalidCase,
                "decode entropy",
                "empty",
            ));
        }
        Ok(input.to_vec())
    }

    fn encode(&self, case: &Vec<u8>) -> Result<Vec<u8>> {
        Ok(case.clone())
    }

    fn generate(&self, input: &[u8]) -> Result<Generation<Vec<u8>>> {
        Ok(Generation::Generated(input.to_vec()))
    }

    fn evaluate(&self, entropy: &Vec<u8>) -> Result<CaseOutcome> {
        let (a, b) = pair(entropy);
        if let Err(detail) = check_serialization(a, b) {
            return failure(SERIALIZATION, detail);
        }
        if let Err(detail) = check_recognition(&notation(entropy)) {
            return failure(RECOGNITION, detail);
        }
        let text = text_of(entropy);
        if let Err(detail) = check_totality(&text) {
            return failure(TOTALITY, detail);
        }
        if let Err(detail) = check_whole_units(&text) {
            return failure(WHOLE_UNITS, detail);
        }
        if let Err(detail) = check_escaped_punctuation(&text) {
            return failure(ESCAPED_PUNCTUATION, detail);
        }
        CaseOutcome::evaluated(PROPERTIES.map(property).to_vec())
    }
}

#[test]
fn generated_notations_and_sheets_keep_the_properties_of_the_syntax_drivers() {
    let target = Target::new();
    let report = run_property(
        &target,
        &bytes(ENTROPY_BYTES, target.descriptor().limits()),
        PropertyConfig {
            cases: CASES,
            max_rejections: 0,
            max_shrink_iterations: 96,
            seed: SEED,
        },
        &mut |_| Ok(()),
    )
    .expect("property run completes with sufficient coverage");
    if let Some(failure) = report.failure {
        panic!(
            "{:?}: {}\nentropy: {:?}",
            failure.outcome.failure_key(),
            failure.outcome.detail().unwrap_or_default(),
            failure.canonical
        );
    }
    assert!(report.counts.evaluated() >= u64::from(CASES));
}

#[test]
fn the_properties_detect_a_wrong_driver_result() {
    // The checks must be able to fail. A pair written as its serialization and
    // read as another is a violation; extents that differ are; and a block of
    // a group opened by an escaped brace is.
    assert!(check_serialization(2, 1).is_ok());
    assert!(check_serialization(i32::MIN, i32::MAX).is_ok());
    assert_eq!(pair(&[0, 0, 9, 9, 9, 9, 9, 9, 9, 9]).0, 0);
    assert!(check_recognition("2n+1").is_ok());
    assert!(
        check_recognition("2147483648").is_ok(),
        "an overflow is not recognized"
    );
    assert!(check_whole_units("a{} @b; c").is_ok());
    // `\{` is an ident code point, not a block: the rule below has the one
    // real block, and the check passes.
    assert!(check_escaped_punctuation("a\\{b {x}").is_ok());
    // A block of a group opened by `(` is rejected by the origin check.
    let document = parse_text("(a) {}").unwrap();
    let list = document
        .children(document.list(document.root_list())[0])
        .unwrap();
    let wrong = document.stream(list);
    assert!(Origin::check(&wrong).is_err());
}

#[test]
fn the_notation_and_pair_generators_reach_the_edges() {
    let mut seen_overflow = false;
    let mut seen_edge = false;
    for seed in 0..=255_u8 {
        let entropy = [
            seed,
            seed ^ 0x35,
            seed.wrapping_mul(7),
            3,
            5,
            9,
            11,
            13,
            17,
            19,
        ];
        let text = notation(&entropy);
        seen_overflow |= text.contains("2147483648") || text.contains("99999999999999999999");
        let (a, b) = pair(&entropy);
        seen_edge |= a == i32::MAX || b == i32::MIN;
    }
    assert!(seen_overflow && seen_edge);
}
