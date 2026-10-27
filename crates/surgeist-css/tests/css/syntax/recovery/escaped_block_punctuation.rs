//! An escaped `\{`, `\}`, `\(`, `\)`, `\[` or `\]` is an ident code point, not a
//! block delimiter.
//!
//! CSS Syntax 3 (2026 CRD) §4.3.7 "consume an escaped code point" consumes the
//! code point after a valid backslash into the surrounding name, so `\{` is an
//! `<ident-token>` whose value is `{` and never a `<{-token>`; the same holds
//! for the other five. Every grammar in §5 then consumes tokens by type, so an
//! escaped punctuation mark behaves exactly like any other ident. Two oracles
//! follow from that and neither reads the implementation's output for the
//! escaped input:
//!
//! - literal expectations derived by hand from §5.4.8 and §5.5.3 to §5.5.6 for
//!   the inputs that crashed (`a{\{}`, `a{b:c;\{}`), and
//! - a metamorphic property: replacing each escape by a plain ident (`z`)
//!   cannot change which units are retained or dropped, their actions or
//!   error codes, nor where they lie, other than the shift that the longer
//!   spelling adds to later byte offsets.
//!
//! No front may panic on any of them. The property's generator and codec
//! belong to this crate; `surgeist-fuzz` supplies the bounded proptest runner.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use surgeist_css::component::{CssComponentValue, CssComponentValues};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTextTarget};
use surgeist_css::query::media::{CssMediaQuery, CssMediaQueryList};
use surgeist_css::query::supports::{CssSupportsTestBody, CssSupportsTestChild};
use surgeist_css::rule::font_feature_values::{
    CssFontFeatureValueBlock, CssFontFeatureValueKind, CssFontFeatureValuesFamilyList,
    CssFontFeatureValuesItem,
};
use surgeist_css::rule::keyframes::{
    CssKeyframeBlock, CssKeyframeDeclarationList, CssParsedKeyframeRule, CssParsedKeyframeSelectors,
};
use surgeist_css::rule::page::{
    CssMarginDeclarationBlock, CssMarginRule, CssPageBody, CssPageSelectorList,
};
use surgeist_css::rule::{
    CssBlockContext, CssBlockFragment, CssRule, CssRuleList, CssRuleSyntax, CssStyleAncestor,
    CssStyleBlock,
};
use surgeist_css::selector::{
    CssNamespaceContext, CssParsedStyleSelectors, CssRelativeSelectorList, CssSelector,
    CssStyleSelectorContext,
};
use surgeist_css::source::{
    CssDecodedStylesheetInput, CssStylesheetDecodeHints, CssUtf16InputLimits,
};
use surgeist_css::{CssParseReport, CssParser};
use surgeist_fuzz::engine::{PropertyConfig, run_property};
use surgeist_fuzz::generation::property::bytes;
use surgeist_fuzz::{
    BuildRequirements, CaseFormat, CaseOutcome, CaseTarget, FailureClass, FailureKey, FuzzError,
    FuzzErrorKind, Generation, GeneratorIdentity, InputEncoding, InputLimits, PropertyId, Result,
    TargetDescriptor, TargetDescriptorSpec, TargetId,
};

/// The six escapes, and the plain ident each stands in for in its twin.
const ESCAPES: [&str; 6] = ["\\{", "\\}", "\\(", "\\)", "\\[", "\\]"];
const PLAIN: &str = "z";

/// One front's observable outcome, without messages or coordinates other than
/// byte offsets.
#[derive(Debug, PartialEq, Eq)]
struct Summary {
    clean: bool,
    shape: usize,
    /// Action, code, span start and end, and error position, in report order.
    diagnostics: Vec<(CssRecoveryAction, CssErrorCode, usize, usize, usize)>,
}

fn summarize<T>(report: &CssParseReport<T>, shape: usize) -> Summary {
    Summary {
        clean: report.is_clean(),
        shape,
        diagnostics: report
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.action(),
                    diagnostic.error().code(),
                    diagnostic.span().start().byte_offset().value(),
                    diagnostic.span().end().byte_offset().value(),
                    diagnostic.error().position().byte_offset().value(),
                )
            })
            .collect(),
    }
}

fn optional<T>(report: &CssParseReport<Option<T>>) -> Summary {
    summarize(report, usize::from(report.syntax().is_some()))
}

fn result<T, E>(outcome: &std::result::Result<T, E>) -> Summary {
    Summary {
        clean: outcome.is_ok(),
        shape: usize::from(outcome.is_ok()),
        diagnostics: Vec::new(),
    }
}

fn utf16(source: &str) -> Arc<Vec<u16>> {
    Arc::new(source.encode_utf16().collect())
}

type Front = (&'static str, Box<dyn Fn(&str) -> Summary>);

fn block<B: surgeist_css::rule::CssBlockBody>(
    source: &str,
    context: &CssBlockContext<'_>,
) -> Summary {
    optional(&CssBlockFragment::<B>::parse(
        &CssParser::default(),
        source,
        context,
    ))
}

/// Every public front that takes source text and can reach a curly block, a
/// function or a bracket.
fn fronts() -> Vec<Front> {
    let mut fronts: Vec<Front> = Vec::new();
    macro_rules! front {
        ($name:expr, |$source:ident| $body:expr) => {
            fronts.push(($name, Box::new(|$source: &str| $body)))
        };
    }
    front!("sheet", |s| {
        let report = CssParser::default().parse_sheet(s);
        summarize(&report, report.syntax().rules().len())
    });
    front!("sheet.bytes", |s| {
        let decoded = CssDecodedStylesheetInput::decode(
            s.as_bytes(),
            CssStylesheetDecodeHints {
                protocol_label: None,
                environment_encoding: None,
            },
            Default::default(),
        )
        .expect("ASCII decodes");
        let report = CssParser::default().parse_sheet_bytes(&decoded);
        summarize(&report, report.syntax().rules().len())
    });
    front!("sheet.utf16", |s| {
        let report = CssParser::default()
            .parse_sheet_utf16(
                utf16(s),
                CssUtf16InputLimits::default(),
                || Ok::<(), ()>(()),
            )
            .expect("prepared");
        summarize(&report, report.syntax().rules().len())
    });
    front!("block-contents", |s| {
        let report = CssParser::default().parse_block_contents(s);
        summarize(&report, report.syntax().len())
    });
    front!("block-contents.utf16", |s| {
        let prepared = CssParser::default()
            .parse_block_contents_utf16(utf16(s), CssUtf16InputLimits::default(), || {
                Ok::<(), ()>(())
            })
            .expect("prepared");
        summarize(prepared.report(), prepared.report().syntax().len())
    });
    front!("block-contents.page", |s| optional(
        &CssParser::default().parse_page_block_contents(s)
    ));
    front!("block-contents.font-face", |s| optional(
        &CssParser::default().parse_font_face_block_contents(s)
    ));
    front!("property-value.width", |s| optional(
        &CssParser::default().parse_property_value(
            s,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
            CssImportance::Normal,
        )
    ));
    front!("declaration", |s| optional(&CssDeclaration::parse(
        &CssParser::default(),
        s
    )));
    front!("rule", |s| optional(&CssRule::parse(
        &CssParser::default(),
        s,
        &CssNamespaceContext::default()
    )));
    front!("rule-syntax", |s| optional(&CssRuleSyntax::classify(
        &CssParser::default(),
        s
    )));
    front!("block.style", |s| optional(&CssStyleBlock::parse(
        &CssParser::default(),
        s,
        &CssNamespaceContext::default()
    )));
    front!("block.rule-list", |s| block::<CssRuleList>(
        s,
        &CssBlockContext::default()
    ));
    front!("block.group", |s| block::<CssRuleList>(
        s,
        &CssBlockContext::group(&CssNamespaceContext::default())
    ));
    front!("block.scope-present", |s| block::<CssRuleList>(
        s,
        &CssBlockContext::scope(&CssNamespaceContext::default(), CssStyleAncestor::Present)
    ));
    front!("block.scope-absent", |s| block::<CssRuleList>(
        s,
        &CssBlockContext::scope(&CssNamespaceContext::default(), CssStyleAncestor::Absent)
    ));
    front!("block.scoped-group-present", |s| block::<CssRuleList>(
        s,
        &CssBlockContext::scoped_group(&CssNamespaceContext::default(), CssStyleAncestor::Present)
    ));
    front!("block.scoped-group-absent", |s| block::<CssRuleList>(
        s,
        &CssBlockContext::scoped_group(&CssNamespaceContext::default(), CssStyleAncestor::Absent)
    ));
    front!("block.page", |s| block::<CssPageBody>(
        s,
        &CssBlockContext::default()
    ));
    front!("block.margin", |s| block::<CssMarginDeclarationBlock>(
        s,
        &CssBlockContext::default()
    ));
    front!("block.keyframes", |s| block::<Vec<CssKeyframeBlock>>(
        s,
        &CssBlockContext::default()
    ));
    front!("block.keyframe-declarations", |s| block::<
        CssKeyframeDeclarationList,
    >(
        s,
        &CssBlockContext::default()
    ));
    front!("block.supports-test", |s| block::<CssSupportsTestBody>(
        s,
        &CssBlockContext::default()
    ));
    front!("block.font-face", |s| block::<
        surgeist_css::rule::font_face::CssFontFaceDescriptors,
    >(s, &CssBlockContext::default()));
    front!("block.counter-style", |s| block::<
        surgeist_css::rule::counter_style::CssCounterStyleDescriptors,
    >(
        s, &CssBlockContext::default()
    ));
    front!("block.font-palette-values", |s| block::<
        Vec<surgeist_css::rule::font_palette_values::CssFontPaletteDescriptor>,
    >(
        s,
        &CssBlockContext::default()
    ));
    front!("block.color-profile", |s| block::<
        Vec<surgeist_css::rule::color_profile::CssColorProfileDescriptor>,
    >(
        s, &CssBlockContext::default()
    ));
    front!("block.font-feature-values", |s| block::<
        Vec<CssFontFeatureValuesItem>,
    >(
        s,
        &CssBlockContext::default()
    ));
    for kind in [
        CssFontFeatureValueKind::Styleset,
        CssFontFeatureValueKind::Swash,
        CssFontFeatureValueKind::CharacterVariant,
    ] {
        fronts.push((
            "block.font-feature-value",
            Box::new(move |s: &str| {
                optional(
                    &CssBlockFragment::<CssFontFeatureValueBlock>::parse_definition(
                        &CssParser::default(),
                        s,
                        kind,
                    ),
                )
            }),
        ));
    }
    front!("selector", |s| optional(&CssSelector::parse(
        &CssParser::default(),
        s,
        &CssNamespaceContext::default()
    )));
    front!("selector-list", |s| optional(
        &CssParsedStyleSelectors::parse(
            &CssParser::default(),
            s,
            &CssNamespaceContext::default(),
            CssStyleSelectorContext::Nested,
        )
    ));
    front!("relative-selector-list", |s| optional(
        &CssRelativeSelectorList::parse(&CssParser::default(), s, &CssNamespaceContext::default())
    ));
    front!("media-query", |s| summarize(
        &CssMediaQuery::parse(&CssParser::default(), s),
        0
    ));
    front!("media-query-list", |s| summarize(
        &CssMediaQueryList::parse(&CssParser::default(), s),
        0
    ));
    front!("keyframe-rule", |s| optional(
        &CssParsedKeyframeRule::parse(&CssParser::default(), s)
    ));
    front!("keyframe-selectors", |s| optional(
        &CssParsedKeyframeSelectors::parse(&CssParser::default(), s)
    ));
    front!("page-selectors", |s| optional(&CssPageSelectorList::parse(
        &CssParser::default(),
        s
    )));
    front!("page-margin-rule", |s| optional(&CssMarginRule::parse(
        &CssParser::default(),
        s
    )));
    front!("supports.test-child", |s| optional(
        &CssSupportsTestChild::parse(&CssParser::default(), s)
    ));
    front!("supports.test-body", |s| optional(
        &CssSupportsTestBody::parse(&CssParser::default(), s)
    ));
    front!("font-feature-family-list", |s| optional(
        &CssFontFeatureValuesFamilyList::parse(&CssParser::default(), s)
    ));
    front!("component-value", |s| optional(&CssComponentValue::parse(
        &CssParser::default(),
        s
    )));
    front!("component-values", |s| result(&CssComponentValues::parse(
        &CssParser::default(),
        s
    )));
    fronts
}

/// Contexts with a hole for the escape: style blocks, at-rule preludes and
/// blocks, nested rules, declaration values, custom properties and the top
/// level. Each also appears closed and unclosed.
const TEMPLATES: [&str; 40] = [
    "a{@}",
    "a{color:red;@}",
    "a{@;color:red}",
    "a{color:@}",
    "a{color:red @}",
    "a{--x:@}",
    "a{--x:@;color:red}",
    "a{--x:f(@)}",
    "a{--x:[@]}",
    "a{--x:{@}}",
    "a{b:c;@",
    "a{@",
    "a{@ x}",
    "a{@ x;color:red}",
    "a{b{@}}",
    "a{&{@}}",
    "a{& b{@}}",
    "a{@media x{@}}",
    "a{@media @{}}",
    "a{@foo @;color:red}",
    "@",
    "@}",
    "@ {}",
    "@;a{color:red}",
    "@{}",
    "@media @{a{color:red}}",
    "@media x{a{@}}",
    "@media x{@}",
    "@media x{@",
    "@foo @;",
    "@foo @{}",
    "@font-face{@}",
    "@font-face{src:@}",
    "@page{@}",
    "@page @{}",
    "@keyframes k{@}",
    "@keyframes @{from{color:red}}",
    "@keyframes k{from{@}}",
    "@supports (@){a{color:red}}",
    "a@{color:red}",
];

fn instantiate(template: &str, escape: &str) -> String {
    template.replace('@', escape)
}

/// Whether the front panics; the panic message goes to the test's stderr.
fn panics(front: &Front, source: &str) -> bool {
    catch_unwind(AssertUnwindSafe(|| (front.1)(source))).is_err()
}

#[test]
fn no_front_panics_on_an_escaped_bracket_or_brace_in_any_context() {
    let fronts = fronts();
    let mut failures = Vec::new();
    for template in TEMPLATES {
        for escape in ESCAPES {
            let source = instantiate(template, escape);
            for front in &fronts {
                if panics(front, &source) {
                    failures.push(format!("{}: {source:?}", front.0));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "panicking fronts:\n{}",
        failures.join("\n")
    );
}

#[test]
fn no_front_panics_on_an_escaped_brace_after_a_bad_string_or_inside_one() {
    // A string ends at a newline unclosed, so its last byte can be `{`; the
    // `{` is then string content, never a block opener.
    let fronts = fronts();
    let mut failures = Vec::new();
    for source in [
        "a{\"{\n}",
        "a{\"\\{\n}",
        "a{b:\"{\n}",
        "a{b:c;\"{\n}",
        "a{'{\n}",
        "@media \"{\n{}",
        "\"{\n}",
        "a{/*{*/}",
        "a{url(\\{)}",
        "a{url(a\\{b)}",
    ] {
        for front in &fronts {
            if panics(front, source) {
                failures.push(format!("{}: {source:?}", front.0));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "panicking fronts:\n{}",
        failures.join("\n")
    );
}

type Diagnostic = (CssRecoveryAction, CssErrorCode, usize, usize, usize);

fn sheet(source: &str) -> (usize, Vec<Diagnostic>) {
    let report = CssParser::default().parse_sheet(source);
    (
        report.syntax().rules().len(),
        summarize(&report, 0).diagnostics,
    )
}

fn contents(source: &str) -> (usize, Vec<Diagnostic>) {
    let report = CssParser::default().parse_block_contents(source);
    (report.syntax().len(), summarize(&report, 0).diagnostics)
}

// Syntax 3 (2026 CRD) §5.5.5: in `a{\{}` the style rule's contents are the
// ident `\{` and then `}`. Consume a declaration fails after the ident (no
// colon: §5.5.6 consumes the remnants of a bad declaration, stopping before
// the nested `}`), so §5.5.5 restores the mark and consumes a qualified rule
// (§5.5.3): its prelude is the ident, and the `}` it then meets is a parse
// error that returns nothing. One unit, exactly the escape (bytes 2 to 4), is
// dropped, the error lies at the `}` (byte 4), and the rule survives empty.
#[test]
fn an_escaped_brace_alone_in_a_style_block_drops_only_that_unit() {
    let (rules, diagnostics) = sheet("a{\\{}");
    assert_eq!(rules, 1);
    assert_eq!(
        diagnostics,
        [(
            CssRecoveryAction::DropDeclaration,
            CssErrorCode::UnexpectedEnd,
            2,
            4,
            4
        )]
    );
}

// The same algorithm for `a{color:red;\{}`: the declaration `color:red` ends
// at its semicolon and survives (`a`, `{`, `color` 2..7, `:` 7, `red` 8..11,
// `;` 11); the unit `\{` is bytes 12 to 14 and the `}` is byte 14.
#[test]
fn an_escaped_brace_after_a_declaration_drops_only_that_unit() {
    let report = CssParser::default().parse_sheet("a{color:red;\\{}");
    let rules = report.syntax().rules();
    assert_eq!(rules.len(), 1);
    let CssRule::Style(style) = &rules[0] else {
        panic!("one style rule: {report:?}");
    };
    assert_eq!(style.declarations().len(), 1);
    assert_eq!(
        summarize(&report, 0).diagnostics,
        [(
            CssRecoveryAction::DropDeclaration,
            CssErrorCode::UnexpectedEnd,
            12,
            14,
            14
        )]
    );
}

// §5.5.6 for `a{b:c;\{}`: `b:c;` is a complete declaration of an unknown
// property (bytes 2 to 6), then the same lone-ident unit as above (6 to 8).
#[test]
fn the_reported_crash_inputs_report_each_unit_once() {
    let (rules, diagnostics) = sheet("a{b:c;\\{}");
    assert_eq!(rules, 1);
    assert_eq!(
        diagnostics,
        [
            (
                CssRecoveryAction::DropDeclaration,
                CssErrorCode::UnknownProperty,
                2,
                6,
                2
            ),
            (
                CssRecoveryAction::DropDeclaration,
                CssErrorCode::UnexpectedEnd,
                6,
                8,
                8
            ),
        ]
    );
}

// The other five escapes are idents in the same way (§4.3.7), so each yields
// the report of the lone-ident unit; a following `}` still closes the block.
#[test]
fn every_escaped_bracket_is_an_ident_unit_in_a_style_block() {
    for escape in ESCAPES {
        let source = format!("a{{{escape}}}b{{color:red}}");
        let report = CssParser::default().parse_sheet(&source);
        assert_eq!(report.syntax().rules().len(), 2, "{source}");
        assert_eq!(
            summarize(&report, 0).diagnostics,
            [(
                CssRecoveryAction::DropDeclaration,
                CssErrorCode::UnexpectedEnd,
                2,
                4,
                4
            )],
            "{source}"
        );
    }
}

// A custom property's value is any component values (§5.5.6, §5.4.8), so an
// escaped brace in one is an ident in a retained declaration with no report.
#[test]
fn an_escaped_brace_in_a_custom_property_value_is_retained_cleanly() {
    for escape in ESCAPES {
        let source = format!("a{{--x:{escape};color:red}}");
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssRule::Style(style) = &report.syntax().rules()[0] else {
            panic!("one style rule: {source}");
        };
        assert_eq!(style.declarations().len(), 2, "{source}");
    }
}

// At the top level (§5.4.2 and §5.5.3 with nested false) the ident `\{` is a
// prelude component, and the `}` after it is appended to the prelude too, so
// the qualified rule runs to end of input and is dropped whole: bytes 0 to 3.
#[test]
fn an_escaped_brace_before_a_top_level_close_is_one_unfinished_rule() {
    let escaped = sheet("\\{}");
    let plain = sheet("z}");
    assert_eq!(escaped.0, 0);
    assert_eq!(escaped.1.len(), 1);
    assert_eq!(escaped.1[0].0, CssRecoveryAction::DropQualifiedRule);
    assert_eq!((escaped.1[0].2, escaped.1[0].3), (0, 3));
    assert_eq!(escaped.1[0].1, plain.1[0].1);
}

// The crate's location rule names the last authored token when input ends
// first. `\{\{` is one ident token (bytes 0 to 4; two adjacent escapes, §4.3.7),
// so the unfinished rule is located at its start, as `zz` would be, and not
// inside it at a `{` byte.
#[test]
fn an_unfinished_rule_of_escaped_braces_is_located_at_its_ident() {
    let report = CssRuleSyntax::classify(&CssParser::default(), "\\{\\{");
    assert_eq!(
        summarize(&report, 0).diagnostics,
        [(
            CssRecoveryAction::RejectInput,
            CssErrorCode::InvalidQualifiedRule,
            0,
            4,
            0
        )]
    );
}

// §4.3.7: the escape belongs to the at-keyword, so `@font-face\{` is the
// at-keyword `font-face{`, which no at-rule grammar names. The unit is
// dropped as an unknown at-rule (bytes 0 to 13) and not as a `@font-face`
// that lacks its block.
#[test]
fn an_escape_inside_an_at_keyword_is_part_of_its_name() {
    let (rules, diagnostics) = sheet("@font-face\\{;");
    assert_eq!(rules, 0);
    assert_eq!(
        diagnostics,
        [(
            CssRecoveryAction::DropAtRule,
            CssErrorCode::UnknownAtRule,
            0,
            13,
            0
        )]
    );
}

// §4.2 makes `_` an ident-start code point and a digit a name code point, so
// an at-keyword (§4.3.1, which consumes an ident sequence per §4.3.11) takes
// them: `@media_x` is one at-keyword named `media_x`, not `@media` followed by
// text. No at-rule grammar names it, so the unit is dropped as an unknown
// at-rule although it ends in `;` and begins with a block-bodied rule's name.
#[test]
fn an_at_keyword_name_runs_through_every_name_code_point() {
    for (source, end) in [("@media_x;", 9), ("@page_1;", 8), ("@font-face\\{x;", 14)] {
        let (rules, diagnostics) = sheet(source);
        assert_eq!(rules, 0, "{source}");
        assert_eq!(
            diagnostics,
            [(
                CssRecoveryAction::DropAtRule,
                CssErrorCode::UnknownAtRule,
                0,
                end,
                0
            )],
            "{source}"
        );
    }
}

// §5.5.5 at the root of block contents: the unit is the ident (bytes 0 to 2)
// and the `}` ends the contents.
#[test]
fn an_escaped_brace_in_block_contents_drops_only_that_unit() {
    let (declarations, diagnostics) = contents("color:red;\\{}width:1px");
    assert_eq!(declarations, 1);
    assert_eq!(
        diagnostics,
        [(
            CssRecoveryAction::DropDeclaration,
            CssErrorCode::UnexpectedEnd,
            10,
            12,
            12
        )]
    );
}

/// The case generator: pieces of authored CSS and the six escapes, joined
/// without separators, so escapes land inside names, preludes, values, blocks
/// and at the top level, balanced and not.
const PIECES: [&str; 40] = [
    "a",
    ".a",
    "#a",
    " ",
    "\n",
    "{",
    "}",
    ";",
    ":",
    "(",
    ")",
    "[",
    "]",
    ",",
    "color",
    "color:red",
    "--x:",
    "--x",
    "red",
    "1",
    "@media",
    "@media x",
    "@font-face",
    "@page",
    "@keyframes k",
    "@supports",
    "@foo",
    "@layer",
    "&",
    "!important",
    "'s'",
    "\"",
    "/*c*/",
    "url(",
    "calc(",
    "a{",
    "a{b:c;",
    "}",
    "{}",
    "c;",
];
const MAX_PIECES: usize = 24;
const CASES: u32 = 500;
const ENTROPY_BYTES: usize = 2 * MAX_PIECES + 1;
const SOURCE_BYTES: usize = 512;

fn generate(entropy: &[u8]) -> String {
    let byte = |index: usize| usize::from(entropy.get(index).copied().unwrap_or(0));
    let count = 1 + byte(0) % MAX_PIECES;
    let mut source = String::new();
    for piece in 0..count {
        let pick = byte(1 + 2 * piece);
        let index = byte(2 + 2 * piece);
        // About two in five pieces are escapes.
        if pick % 5 < 2 {
            source.push_str(ESCAPES[index % ESCAPES.len()]);
        } else {
            source.push_str(PIECES[index % PIECES.len()]);
        }
    }
    source
}

/// The twin of `source` with each escape replaced by a plain ident, and the
/// plain-source offsets at which a replacement starts.
fn twin(source: &str) -> (String, Vec<usize>) {
    let mut plain = String::new();
    let mut replaced = Vec::new();
    let mut rest = source;
    while let Some(index) = rest.find('\\') {
        plain.push_str(&rest[..index]);
        replaced.push(plain.len());
        plain.push_str(PLAIN);
        rest = &rest[index + 2..];
    }
    plain.push_str(rest);
    (plain, replaced)
}

/// A plain-source offset in the escaped source: each replacement that starts
/// before it adds one byte.
fn shifted(offset: usize, replaced: &[usize]) -> usize {
    offset + replaced.iter().filter(|&&start| start < offset).count()
}

fn property(name: &str) -> PropertyId {
    PropertyId::new(format!("surgeist-css::{name}")).expect("owner-qualified property")
}

/// No front panics on the source.
const NO_PANIC: &str = "no-panic";
/// The escaped source and its plain twin differ only in offsets.
const TWIN_AGREES: &str = "twin-agrees";
/// UTF-8 and UTF-16 spellings of the source report identically.
const ENCODINGS_AGREE: &str = "encodings-agree";
const PROPERTIES: [&str; 3] = [NO_PANIC, TWIN_AGREES, ENCODINGS_AGREE];

struct Target {
    descriptor: TargetDescriptor,
}
impl Target {
    fn new() -> Self {
        let descriptor = TargetDescriptor::new(TargetDescriptorSpec {
            id: TargetId::new("surgeist-css::escaped-block-punctuation").unwrap(),
            owner: "surgeist-css".into(),
            binary: "surgeist-css-escaped-block-punctuation-property".into(),
            properties: PROPERTIES.map(property).to_vec(),
            source_references: vec![
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-escaped-code-point"
                    .into(),
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents"
                    .into(),
            ],
            format: CaseFormat::new("surgeist-css::escaped-punctuation-source", 1).unwrap(),
            generator_identity: GeneratorIdentity::new("surgeist-css::escaped-pieces", 1).unwrap(),
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
        String::from_utf8(input.to_vec())
            .map_err(|_| FuzzError::new(FuzzErrorKind::InvalidCase, "decode css source", "UTF-8"))
    }
    fn encode(&self, case: &String) -> Result<Vec<u8>> {
        Ok(case.as_bytes().to_vec())
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<String>> {
        Ok(Generation::Generated(generate(input)))
    }
    fn evaluate(&self, source: &String) -> Result<CaseOutcome> {
        let (plain, replaced) = twin(source);
        let fronts = fronts();
        for front in &fronts {
            for text in [source, &plain] {
                if panics(front, text) {
                    return failure(NO_PANIC, front.0, text.clone());
                }
            }
        }
        for front in &fronts {
            let escaped = (front.1)(source);
            let mut expected = (front.1)(&plain);
            for diagnostic in &mut expected.diagnostics {
                diagnostic.2 = shifted(diagnostic.2, &replaced);
                diagnostic.3 = shifted(diagnostic.3, &replaced);
                diagnostic.4 = shifted(diagnostic.4, &replaced);
            }
            if escaped != expected {
                return failure(
                    TWIN_AGREES,
                    front.0,
                    format!("{source:?}: {escaped:?} against twin {plain:?}: {expected:?}"),
                );
            }
        }
        for (utf8, utf16) in [
            ("sheet", "sheet.utf16"),
            ("block-contents", "block-contents.utf16"),
        ] {
            let by_name = |name: &str| fronts.iter().find(|front| front.0 == name).unwrap();
            let (left, right) = ((by_name(utf8).1)(source), (by_name(utf16).1)(source));
            if left != right {
                return failure(
                    ENCODINGS_AGREE,
                    utf8,
                    format!("{source:?}: utf-8 {left:?} utf-16 {right:?}"),
                );
            }
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

#[test]
fn generated_escaped_punctuation_sources_never_panic_and_match_their_plain_twins() {
    let target = Target::new();
    let report = run_property(
        &target,
        &bytes(ENTROPY_BYTES, target.descriptor().limits()),
        PropertyConfig {
            cases: CASES,
            max_rejections: 0,
            max_shrink_iterations: 96,
            seed: 0x1487_0e5c,
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
fn the_twin_replaces_each_escape_and_maps_offsets_back() {
    let (plain, replaced) = twin("a\\{b\\)");
    assert_eq!(plain, "azbz");
    assert_eq!(replaced, [1, 3]);
    // Offsets at or before a replacement are unmoved; later ones gain a byte.
    assert_eq!(shifted(1, &replaced), 1);
    assert_eq!(shifted(2, &replaced), 3);
    assert_eq!(shifted(4, &replaced), 6);
}
