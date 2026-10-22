//! Generic property contracts, run once for every independently authored
//! record through every authored front and both importances.
//!
//! Cascade 4 (CR 2022-01-13) §7.3 makes the five CSS-wide keywords valid for
//! every property as the whole value; Cascade 5 adds `revert-layer`. A
//! shorthand set to one of them sets each longhand member, including reset-only
//! members, to that keyword (Cascade 4 §1.2), while `all` keeps one universal
//! reset (Cascade 4 §3.2). CSSOM 1 (WD 2021-08-26) §6.7.2 serializes the
//! keyword as authored in lowercase, and a declaration as `name: value;`, with
//! ` !important` before the semicolon when important. CSS Variables 1 §3 and
//! Values 5 §7.7 admit `var()`, `env()` and `attr()` at parse time and defer
//! the grammar to substitution, where the replacement is checked strictly.
//! Syntax 3 (CRD 2021-12-24) §5.4 closes an unfinished comment, function or
//! block at the end of input with a parse error, so recovery fronts retain a
//! recovered value while checked construction rejects it at the closure.
//!
//! Each record's `accepts`/`rejects` rows and the catalog samples they extend
//! are handwritten from the property's grammar; the expected serializations
//! are never taken from the implementation under test.

use crate::css::support::authored_property::{self, assert_source};
use crate::css::support::property_admission::assert_implicit_closure;
use crate::css::support::property_expectations::{self, MetadataExpectation, PropertyExpectation};
use crate::css::support::serialization_limits::LimitLadder;
use std::panic::{AssertUnwindSafe, catch_unwind};
use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
const IMPORTANCE: [CssImportance; 2] = [CssImportance::Normal, CssImportance::Important];
/// Whole-value substitutions. The last spelling keeps the `*` argument that
/// the family suites previously exercised; admission does not depend on it.
const PENDING: [&str; 4] = [
    "var(--known)",
    "env(known)",
    "attr(data-known)",
    "attr(data-known *)",
];
/// Replacements that still contain a substitution function, including an
/// escaped `var` function name (Syntax 3 §4.3.4), and functions or comments
/// closed only by the end of input, which stay residual first.
const RESIDUAL: [&str; 12] = [
    "var(--again)",
    "env(again)",
    "attr(data-again)",
    "attr(data-again *)",
    r"\76 ar(--again)",
    "var(--again",
    "env(again",
    "attr(data-again",
    "attr(data-again *",
    "var(--again)/*",
    "unknown var(--again",
    "unknown var(--again)",
];
/// Physical members that a CSS-wide keyword selects for each four-side
/// shorthand (Box 3 §4.2/§5.2, Backgrounds 3 §3.1-§3.3, Position 3 §3.1,
/// Scroll Snap 1 §6.1-§6.2), in top, right, bottom, left order.
const FOUR_SIDE: [(&str, [&str; 4]); 8] = [
    (
        "border-color",
        [
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
        ],
    ),
    (
        "border-style",
        [
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
        ],
    ),
    (
        "border-width",
        [
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ],
    ),
    ("inset", ["top", "right", "bottom", "left"]),
    (
        "margin",
        ["margin-top", "margin-right", "margin-bottom", "margin-left"],
    ),
    (
        "padding",
        [
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ],
    ),
    (
        "scroll-margin",
        [
            "scroll-margin-top",
            "scroll-margin-right",
            "scroll-margin-bottom",
            "scroll-margin-left",
        ],
    ),
    (
        "scroll-padding",
        [
            "scroll-padding-top",
            "scroll-padding-right",
            "scroll-padding-bottom",
            "scroll-padding-left",
        ],
    ),
];

#[derive(Clone, Copy, Debug)]
enum Front {
    Attribute,
    UppercaseAttribute,
    CheckedName,
    CheckedGrammar,
    TextName,
    TextGrammar,
}

const FRONTS: [Front; 6] = [
    Front::Attribute,
    Front::UppercaseAttribute,
    Front::CheckedName,
    Front::CheckedGrammar,
    Front::TextName,
    Front::TextGrammar,
];

/// Properties whose rejected values the shared Color 4 grammar diagnoses
/// (an implementation-defined attribution; the declaration is still dropped).
const COLOR_DIAGNOSTICS: [P; 3] = [P::CaretColor, P::BorderColor, P::ScrollbarColor];

const PREFIX: &str = "/*😀*/";

fn important_suffix(importance: CssImportance) -> &'static str {
    match importance {
        CssImportance::Normal => "",
        CssImportance::Important => "!important",
    }
}

fn specified(name: &str, value: &str, importance: CssImportance) -> String {
    match importance {
        CssImportance::Normal => format!("{name}: {value};"),
        CssImportance::Important => format!("{name}: {value} !important;"),
    }
}

/// Runs `check` for every record, collecting each record's failure so one
/// disagreement does not hide the others.
fn each_record(check: impl Fn(&PropertyExpectation)) {
    let cases = property_expectations::CASES;
    assert!(!cases.is_empty(), "independent record table");
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|case| {
            catch_unwind(AssertUnwindSafe(|| check(case)))
                .err()
                .map(|panic| {
                    let message = panic
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_owned()))
                        .unwrap_or_default();
                    format!("{}: {message}", case.name)
                })
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{} record failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The declaration that `front` admits for `value` with `importance`, after
/// checking the identity, importance and provenance that front owns.
#[track_caller]
fn admit(
    front: Front,
    case: &PropertyExpectation,
    value: &str,
    importance: CssImportance,
) -> CssDeclaration {
    let property = case.property;
    let source = match front {
        Front::Attribute | Front::UppercaseAttribute => {
            let name = if matches!(front, Front::Attribute) {
                case.name.to_owned()
            } else {
                case.name.to_ascii_uppercase()
            };
            let css = format!("{PREFIX}{name}:{value}{}", important_suffix(importance));
            let report = parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            assert_eq!(
                &validate_style_attribute(&css).unwrap(),
                report.syntax(),
                "{css}: validation"
            );
            let [source] = report.syntax().as_slice() else {
                panic!("{css}: expected one occurrence")
            };
            let position = source.position().expect("authored position");
            assert_eq!(position.byte_offset().value(), PREFIX.len(), "{css}");
            assert_eq!(position.line().value(), 0, "{css}");
            assert_eq!(
                position.column().value() as usize,
                PREFIX.encode_utf16().count(),
                "{css}"
            );
            assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
            assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
            source.clone()
        }
        Front::CheckedName | Front::CheckedGrammar => {
            let components = parse_component_values(value).unwrap();
            let before = components.clone();
            let source = authored_property::checked_components(
                property,
                components.clone(),
                matches!(front, Front::CheckedGrammar),
                importance,
            )
            .unwrap_or_else(|error| panic!("{value:?} via {front:?}/{importance:?}: {error:?}"));
            assert_eq!(components, before, "{value:?}: immutable input");
            assert_eq!(source.value_components(), &components);
            assert!(source.position().is_none());
            assert!(source.parsed_name().is_none());
            assert!(source.parsed_value().is_none());
            source
        }
        Front::TextName | Front::TextGrammar => {
            let report = text_report(case, value, matches!(front, Front::TextGrammar), importance);
            assert!(
                report.is_clean(),
                "{value:?} via {front:?}/{importance:?}: {:?}",
                report.diagnostics()
            );
            let source = report.syntax().as_ref().expect("retained value").clone();
            assert_eq!(source.parsed_value().unwrap().source().as_str(), value);
            assert!(source.position().is_none());
            assert!(source.parsed_name().is_none());
            source
        }
    };
    let known = source.known().expect("known declaration");
    assert_eq!(known.property(), property, "{value:?} via {front:?}");
    assert_eq!(
        known.grammar(),
        property.grammar(),
        "{value:?} via {front:?}"
    );
    assert_eq!(source.importance(), importance, "{value:?} via {front:?}");
    source
}

fn text_report(
    case: &PropertyExpectation,
    value: &str,
    grammar: bool,
    importance: CssImportance,
) -> CssParseReport<Option<CssDeclaration>> {
    if grammar {
        parse_property_value_text_for_grammar(value, case.property.grammar(), importance)
    } else {
        parse_property_value_text(value, CssPropertyNameRef::Known(case.property), importance)
    }
}

/// The specified serialization of `source`, unchanged by serializing, and
/// re-parsed by the style attribute into the same serialization.
#[track_caller]
fn assert_specified(source: &CssDeclaration, expected: &str, label: &str) {
    let before = source.clone();
    assert_eq!(source.to_specified_css().unwrap(), expected, "{label}");
    assert_eq!(source, &before, "{label}: serialization mutated the source");
    let report = parse_style_attribute(expected);
    assert!(
        report.is_clean(),
        "{label}: re-parse {expected:?}: {:?}",
        report.diagnostics()
    );
    let [reparsed] = report.syntax().as_slice() else {
        panic!("{label}: re-parse {expected:?} retained one declaration")
    };
    assert_eq!(
        reparsed.to_specified_css().unwrap(),
        expected,
        "{label}: idempotent re-parse"
    );
}

/// The longhands that a CSS-wide keyword sets for `case`, in member order;
/// `None` for the universal reset.
fn css_wide_members(case: &PropertyExpectation) -> Option<Vec<P>> {
    match &case.metadata {
        MetadataExpectation::Longhand { .. } => Some(vec![case.property]),
        MetadataExpectation::Shorthand { settable, reset } => {
            Some(settable.iter().chain(reset.iter()).copied().collect())
        }
        MetadataExpectation::FourSide => {
            let (_, members) = FOUR_SIDE
                .iter()
                .find(|(name, _)| *name == case.name)
                .expect("independent four-side members");
            Some(
                members
                    .iter()
                    .map(|name| P::from_name(name).expect("known four-side member"))
                    .collect(),
            )
        }
        MetadataExpectation::UniversalReset => None,
        MetadataExpectation::Unavailable => panic!("no expansion metadata"),
    }
}

/// `contributions` set every CSS-wide member of `case` to `keyword` for the
/// occurrence `source`, carrying `replacement` when reentered.
#[track_caller]
fn assert_global_contributions(
    case: &PropertyExpectation,
    contributions: CssContributions,
    source: &CssDeclaration,
    keyword: CssGlobalKeyword,
    replacement: Option<&CssComponentValues>,
) {
    match (css_wide_members(case), contributions) {
        (Some(members), CssContributions::Longhands(values)) => {
            let actual: Vec<_> = values.items().iter().map(|item| item.property()).collect();
            assert_eq!(actual, members, "{keyword:?} members");
            for item in values.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.ordinary_value().is_none());
                assert_eq!(item.replacement_components(), replacement);
                assert_source(item, source);
            }
        }
        (None, CssContributions::UniversalReset(reset)) => {
            assert_eq!(reset.keyword(), keyword);
            assert!(reset.source().same_occurrence(source));
            assert_eq!(reset.source().importance(), source.importance());
            assert_eq!(reset.replacement_components(), replacement);
        }
        (_, other) => panic!("{keyword:?}: unexpected contributions {other:?}"),
    }
}

/// `contributions` are the ordinary members of `case` for `source`.
#[track_caller]
fn assert_ordinary_contributions(
    case: &PropertyExpectation,
    values: &CssLonghandContributions,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    let actual: Vec<_> = values.items().iter().map(|item| item.property()).collect();
    match &case.metadata {
        MetadataExpectation::FourSide => assert_eq!(actual.len(), 4, "four sides"),
        _ => assert_eq!(Some(actual), css_wide_members(case), "ordinary members"),
    }
    for item in values.items() {
        assert!(
            !matches!(item.value(), CssContributionValueRef::Global(_)),
            "ordinary member {:?}",
            item.property()
        );
        assert_eq!(item.replacement_components(), replacement);
        assert_source(item, source);
    }
    if let MetadataExpectation::Longhand { .. } = case.metadata {
        let item = &values.items()[0];
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            case.property
        );
    }
}

#[track_caller]
fn completed(source: &CssDeclaration) -> CssContributions {
    let CssExpansion::Contributions(contributions) = expand_declaration(source).unwrap() else {
        panic!("completed expansion")
    };
    contributions
}

#[track_caller]
fn longhands(contributions: CssContributions) -> CssLonghandContributions {
    let CssContributions::Longhands(values) = contributions else {
        panic!("longhand contributions: {contributions:?}")
    };
    values
}

/// The original implicit closure of a recovered component list: the first
/// closing origin of a function or block closed by the end of input, else the
/// serialized origin of an unfinished comment.
fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    fn nested(values: &CssComponentValues) -> Option<CssValueOrigin> {
        values.items().iter().find_map(|value| {
            let (closing, inner) = match value.view() {
                CssComponentValueRef::Function(function) => {
                    (function.closing_origin(), function.values())
                }
                CssComponentValueRef::Block(block) => (block.closing_origin(), block.values()),
                _ => return None,
            };
            nested(inner).or_else(|| {
                matches!(closing, CssValueOrigin::ImplicitClosure { .. }).then(|| closing.clone())
            })
        })
    }
    nested(components).unwrap_or_else(|| {
        let serialized = components.serialize().unwrap();
        (0..serialized.as_css().len())
            .find_map(|offset| match serialized.origin_at(offset) {
                Some(CssSerializedOrigin::Token(
                    origin @ CssValueOrigin::ImplicitClosure { .. },
                )) => Some(origin.clone()),
                _ => None,
            })
            .expect("public original implicit closing origin")
    })
}

#[track_caller]
fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin, text: &str) {
    assert!(
        matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
        ),
        "{text:?}: expected an unexpected end, got {error:?}"
    );
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone())),
        "{text:?}"
    );
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("{text:?}: implicit closure")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}

/// `value` is dropped atomically by every front at both importances. The
/// style-attribute diagnostic names the property's grammar, or the shared
/// color grammar when a color component owns the failure.
#[track_caller]
fn invalid(case: &PropertyExpectation, value: &str) {
    let css = format!("color:red;{}:{value};color:blue", case.name);
    let report = parse_style_attribute(&css);
    let retained: Vec<_> = report
        .syntax()
        .iter()
        .map(|item| item.known().unwrap().property())
        .collect();
    assert_eq!(retained, [P::Color, P::Color], "{css}");
    let [diagnostic] = report.diagnostics() else {
        panic!("{css}: one atomic grammar failure")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{css}"
    );
    match diagnostic.error().kind() {
        ErrorKind::InvalidPropertyValue(detail) => assert_eq!(detail.property(), case.property),
        ErrorKind::InvalidColorSyntax(_) if COLOR_DIAGNOSTICS.contains(&case.property) => {}
        other => panic!("{css}: grammar diagnostic, got {other:?}"),
    }
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    for importance in IMPORTANCE {
        for grammar in [false, true] {
            let error = authored_property::checked_components(
                case.property,
                components.clone(),
                grammar,
                importance,
            )
            .unwrap_err();
            assert!(
                matches!(error.kind(), CssPropertyValueErrorKind::Grammar(_)),
                "{value:?}: {error:?}"
            );
            let report = text_report(case, value, grammar, importance);
            assert!(report.syntax().is_none(), "{value:?}: retained");
            assert!(!report.is_clean(), "{value:?}: diagnostic");
        }
    }
    assert_eq!(components, before);
}

#[test]
fn every_record_css_wide_keyword_serializes_and_expands_through_every_front_and_importance() {
    each_record(|case| {
        for (text, keyword) in GLOBALS {
            for importance in IMPORTANCE {
                let expected = specified(case.name, text, importance);
                for front in FRONTS {
                    let label = format!("{text} via {front:?}/{importance:?}");
                    let source = admit(front, case, text, importance);
                    let known = source.known().unwrap();
                    assert_eq!(known.global(), Some(keyword), "{label}");
                    assert!(
                        matches!(known.declared_value(), CssKnownDeclaredValueRef::Global(actual) if actual == keyword),
                        "{label}"
                    );
                    assert!(known.property_value().is_none(), "{label}");
                    assert!(known.substitution_dependent().is_none(), "{label}");
                    assert_specified(&source, &expected, &label);
                    // One declaration, its name and one symbolic keyword:
                    // three input and three projection nodes, never the
                    // initial value's members.
                    let before = source.clone();
                    LimitLadder::new(&expected, 3, 3)
                        .assert_unchanged(&source, CssDeclaration::to_specified_css_with_limits);
                    assert_global_contributions(case, completed(&source), &source, keyword, None);
                    assert_eq!(source, before, "{label}: expansion mutated the source");
                }
            }
        }
    });
}

#[test]
fn every_record_programmatic_css_wide_keyword_keeps_origin_through_checked_fronts() {
    each_record(|case| {
        for (text, keyword) in GLOBALS {
            let components =
                CssComponentValues::try_new(vec![CssComponentValue::try_ident(text).unwrap()])
                    .unwrap();
            for importance in IMPORTANCE {
                for grammar in [false, true] {
                    let source = authored_property::checked_components(
                        case.property,
                        components.clone(),
                        grammar,
                        importance,
                    )
                    .unwrap();
                    assert_eq!(
                        source.value_components().items()[0].origin(),
                        &CssValueOrigin::Programmatic
                    );
                    assert_eq!(source.known().unwrap().global(), Some(keyword));
                    assert_eq!(
                        source.to_specified_css().unwrap(),
                        specified(case.name, text, importance)
                    );
                    assert_global_contributions(case, completed(&source), &source, keyword, None);
                }
            }
        }
    });
}

#[test]
fn every_record_programmatic_keyword_row_keeps_origin_through_checked_fronts() {
    // A row whose authored value is one lowercase identifier serializes as
    // itself when supplied as a programmatic component (CSSOM 1 §6.7.2).
    each_record(|case| {
        for (authored, canonical) in case.accepted_rows() {
            let lowercase_ident = authored.starts_with(|c: char| c.is_ascii_lowercase())
                && authored
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            if !lowercase_ident || authored != canonical {
                continue;
            }
            let ident = CssComponentValue::try_ident(authored).unwrap();
            let components = CssComponentValues::try_new(vec![ident]).unwrap();
            let before = components.clone();
            for importance in IMPORTANCE {
                for grammar in [false, true] {
                    let source = authored_property::checked_components(
                        case.property,
                        components.clone(),
                        grammar,
                        importance,
                    )
                    .unwrap_or_else(|error| panic!("{authored}: {error:?}"));
                    assert_eq!(source.value_components(), &components);
                    assert_eq!(
                        source.value_components().items()[0].origin(),
                        &CssValueOrigin::Programmatic
                    );
                    assert!(source.position().is_none());
                    assert!(source.parsed_name().is_none());
                    assert!(source.parsed_value().is_none());
                    assert_eq!(
                        source.to_specified_css().unwrap(),
                        specified(case.name, canonical, importance)
                    );
                    let values = longhands(completed(&source));
                    assert_ordinary_contributions(case, &values, &source, None);
                }
            }
            assert_eq!(components, before);
        }
    });
}

#[test]
fn every_record_accepted_row_serializes_canonically_and_idempotently_through_every_front() {
    each_record(|case| {
        for (authored, canonical) in case.accepted_rows() {
            for importance in IMPORTANCE {
                let expected = specified(case.name, canonical, importance);
                let mut sources = Vec::new();
                for value in [authored, canonical] {
                    for front in FRONTS {
                        let label = format!("{value:?} via {front:?}/{importance:?}");
                        let source = admit(front, case, value, importance);
                        sources.push(source.clone());
                        let known = source.known().unwrap();
                        assert!(known.property_value().is_some(), "{label}");
                        assert!(known.global().is_none(), "{label}");
                        assert!(known.substitution_dependent().is_none(), "{label}");
                        assert_specified(&source, &expected, &label);
                        let values = longhands(completed(&source));
                        assert_ordinary_contributions(case, &values, &source, None);
                        if value == authored {
                            assert!(
                                source
                                    .value_components()
                                    .items()
                                    .iter()
                                    .all(|item| !matches!(
                                        item.origin(),
                                        CssValueOrigin::Programmatic
                                    )),
                                "{label}: original components"
                            );
                        }
                    }
                }
                // Checked construction from the parsed components admits the
                // same ordinary value as the parser that produced them.
                for source in &sources {
                    let parsed = longhands(completed(source));
                    for grammar in [false, true] {
                        let checked = authored_property::checked_components(
                            case.property,
                            source.value_components().clone(),
                            grammar,
                            importance,
                        )
                        .unwrap();
                        let checked = longhands(completed(&checked));
                        let parsed: Vec<_> =
                            parsed.items().iter().map(|item| item.value()).collect();
                        let checked: Vec<_> =
                            checked.items().iter().map(|item| item.value()).collect();
                        assert_eq!(parsed, checked, "{authored:?}: same value");
                    }
                }
            }
        }
    });
}

#[test]
fn every_record_rejected_row_drops_atomically_through_every_front_and_importance() {
    each_record(|case| {
        for value in case.rejected_rows() {
            if parse_component_values(value).is_ok() {
                invalid(case, value);
            } else {
                // A bad URL fails tokenization (Syntax 3 §4.3.6), before any
                // component list exists for the checked fronts.
                for importance in IMPORTANCE {
                    for grammar in [false, true] {
                        let report = text_report(case, value, grammar, importance);
                        assert!(report.syntax().is_none(), "{value:?}: retained");
                        assert!(!report.is_clean(), "{value:?}: diagnostic");
                    }
                }
            }
            for importance in IMPORTANCE {
                let css = format!(
                    "color:red;{}:{value}{};color:blue",
                    case.name.to_ascii_uppercase(),
                    important_suffix(importance)
                );
                let report = parse_style_attribute(&css);
                let retained: Vec<_> = report
                    .syntax()
                    .iter()
                    .map(|item| item.known().unwrap().property())
                    .collect();
                assert_eq!(retained, [P::Color, P::Color], "{css}");
                assert!(!report.is_clean(), "{css}");
                if parse_component_values(value).is_err() {
                    continue;
                }
                let [diagnostic] = report.diagnostics() else {
                    panic!("{css}: one diagnostic")
                };
                assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
                match diagnostic.error().kind() {
                    ErrorKind::InvalidPropertyValue(detail) => {
                        assert_eq!(detail.property(), case.property, "{css}")
                    }
                    ErrorKind::InvalidColorSyntax(_)
                        if COLOR_DIAGNOSTICS.contains(&case.property) => {}
                    other => panic!("{css}: grammar diagnostic, got {other:?}"),
                }
            }
        }
    });
}

#[test]
fn every_record_pending_substitution_is_admitted_and_reenters_strictly_and_repeatably() {
    each_record(|case| {
        let accepted = case.accepted_rows();
        let rejected = case.rejected_rows();
        for pending in PENDING {
            for importance in IMPORTANCE {
                let expected = specified(case.name, pending, importance);
                for front in FRONTS {
                    let label = format!("{pending} via {front:?}/{importance:?}");
                    let source = admit(front, case, pending, importance);
                    let before = source.clone();
                    let known = source.known().unwrap();
                    assert!(known.substitution_dependent().is_some(), "{label}");
                    assert!(known.global().is_none(), "{label}");
                    assert!(known.property_value().is_none(), "{label}");
                    assert_specified(&source, &expected, &label);
                    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                        panic!("{label}: pending handle")
                    };
                    assert!(handle.source().same_occurrence(&source), "{label}");
                    assert_eq!(handle.source().importance(), importance, "{label}");
                    let mut residual: Vec<String> = RESIDUAL.map(str::to_owned).to_vec();
                    let mut rejected: Vec<String> =
                        rejected.iter().map(|text| (*text).to_owned()).collect();
                    rejected.push("initial inherit".to_owned());
                    for (authored, _) in &accepted {
                        residual.push(format!("{authored} var(--again)"));
                        residual.push(format!("{authored} var(--again"));
                        // A replacement value cannot carry an importance
                        // annotation or a declaration separator.
                        rejected.push(format!("{authored}!important"));
                        rejected.push(format!("{authored};color:red"));
                    }
                    for text in &residual {
                        let replacement = parse_component_values(text).unwrap();
                        let snapshot = replacement.clone();
                        for _ in 0..2 {
                            assert_eq!(
                                handle.reenter(replacement.clone()).unwrap_err().kind(),
                                &CssExpansionErrorKind::ResidualSubstitution,
                                "{label}: {text}"
                            );
                        }
                        assert_eq!(replacement, snapshot);
                    }
                    for text in &rejected {
                        // A substitution in the replacement is residual first.
                        let lower = text.to_ascii_lowercase();
                        if ["var(", "env(", "attr("]
                            .iter()
                            .any(|name| lower.contains(name))
                        {
                            continue;
                        }
                        let Ok(replacement) = parse_component_values(text) else {
                            continue;
                        };
                        let direct = authored_property::checked_components(
                            case.property,
                            replacement.clone(),
                            true,
                            importance,
                        )
                        .unwrap_err();
                        for _ in 0..2 {
                            let error = handle.reenter(replacement.clone()).unwrap_err();
                            let CssExpansionErrorKind::InvalidReplacement(detail) = error.kind()
                            else {
                                panic!("{label}: {text}: strict replacement, got {error:?}")
                            };
                            assert!(
                                matches!(detail.kind(), CssPropertyValueErrorKind::Grammar(_)),
                                "{label}: {text}: {detail:?}"
                            );
                            assert_eq!(detail, &direct, "{label}: {text}: checked failure");
                        }
                    }
                    for (text, keyword) in GLOBALS {
                        for spelling in [text.to_owned(), format!("{PREFIX}{text}/**/")] {
                            let replacement = parse_component_values(&spelling).unwrap();
                            for _ in 0..2 {
                                assert_global_contributions(
                                    case,
                                    handle.reenter(replacement.clone()).unwrap(),
                                    &source,
                                    keyword,
                                    Some(&replacement),
                                );
                            }
                        }
                    }
                    for (authored, _) in &accepted {
                        let replacement = parse_component_values(authored).unwrap();
                        let direct = authored_property::checked_components(
                            case.property,
                            replacement.clone(),
                            false,
                            importance,
                        )
                        .unwrap();
                        let direct = longhands(completed(&direct));
                        for _ in 0..2 {
                            let values = longhands(handle.reenter(replacement.clone()).unwrap());
                            assert_ordinary_contributions(
                                case,
                                &values,
                                &source,
                                Some(&replacement),
                            );
                            let reentered: Vec<_> =
                                values.items().iter().map(|item| item.value()).collect();
                            let checked: Vec<_> =
                                direct.items().iter().map(|item| item.value()).collect();
                            assert_eq!(reentered, checked, "{label}: {authored}");
                            for item in values.items() {
                                for (actual, supplied) in item
                                    .replacement_components()
                                    .unwrap()
                                    .items()
                                    .iter()
                                    .zip(replacement.items())
                                {
                                    assert_eq!(actual.origin(), supplied.origin());
                                    if let (
                                        CssValueOrigin::Parsed(actual),
                                        CssValueOrigin::Parsed(supplied),
                                    ) = (actual.origin(), supplied.origin())
                                    {
                                        assert!(actual.source().same_snapshot(supplied.source()));
                                        assert_eq!(actual.source().as_str(), *authored);
                                    }
                                }
                            }
                        }
                    }
                    assert!(handle.source().same_occurrence(&source));
                    assert_eq!(source, before, "{label}: atomic reentry");
                }
            }
        }
    });
}

#[test]
fn every_record_checked_fronts_and_reentry_reject_recovered_ordinary_values_at_original_closure() {
    each_record(|case| {
        // (recovered text, ordinary value it completes, reentry closure applies)
        let mut texts: Vec<(String, Option<&str>, bool)> =
            vec![(format!("{PREFIX}\"unterminated"), None, true)];
        for prefix in ["", PREFIX] {
            for (text, _) in GLOBALS {
                texts.push((format!("{prefix}{text}/*"), None, true));
            }
            for pending in PENDING {
                // A closed substitution stays residual first on reentry.
                texts.push((format!("{prefix}{pending}/*"), None, false));
                texts.push((
                    format!("{prefix}{}", &pending[..pending.len() - 1]),
                    None,
                    false,
                ));
            }
        }
        for (authored, _) in case.accepted_rows() {
            if let Some(open) = authored.strip_suffix(')') {
                // The final function closed only by the end of input.
                texts.push((format!("{PREFIX}{open}"), Some(authored), true));
            }
            texts.push((format!("{authored}/*"), Some(authored), true));
            texts.push((format!("{PREFIX}{authored}/*"), Some(authored), true));
            texts.push((
                format!("{PREFIX}{authored}/*unfinished"),
                Some(authored),
                true,
            ));
        }
        for (text, authored, reentry) in texts {
            let components = parse_component_values(&text).unwrap();
            let before = components.clone();
            let origin = implicit_origin(&components);
            for importance in IMPORTANCE {
                for grammar in [false, true] {
                    assert_closure(
                        &authored_property::checked_components(
                            case.property,
                            components.clone(),
                            grammar,
                            importance,
                        )
                        .unwrap_err(),
                        &origin,
                        &text,
                    );
                }
                if !reentry {
                    continue;
                }
                for pending in PENDING {
                    let source = admit(Front::CheckedName, case, pending, importance);
                    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                        panic!("pending handle")
                    };
                    for _ in 0..2 {
                        let error = handle.reenter(components.clone()).unwrap_err();
                        let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                            panic!("{text:?}: original replacement closure, got {error:?}")
                        };
                        assert_closure(error, &origin, &text);
                    }
                    let Some(authored) = authored else {
                        continue;
                    };
                    for complete in [format!("{authored}/**/"), format!("{authored} /**/")] {
                        let complete = parse_component_values(&complete).unwrap();
                        let values = longhands(handle.reenter(complete.clone()).unwrap());
                        assert_ordinary_contributions(case, &values, &source, Some(&complete));
                        for grammar in [false, true] {
                            authored_property::checked_components(
                                case.property,
                                complete.clone(),
                                grammar,
                                importance,
                            )
                            .unwrap();
                        }
                    }
                }
            }
            assert_eq!(components, before);
        }
        // Matched comments around a CSS-wide keyword or a substitution keep
        // both checked fronts callable.
        for text in GLOBALS
            .iter()
            .map(|(text, _)| format!("{text}/**/"))
            .chain(PENDING.map(|pending| format!("{pending}/**/")))
        {
            let components = parse_component_values(&text).unwrap();
            for importance in IMPORTANCE {
                for grammar in [false, true] {
                    authored_property::checked_components(
                        case.property,
                        components.clone(),
                        grammar,
                        importance,
                    )
                    .unwrap_or_else(|error| panic!("{text}: {error:?}"));
                }
            }
        }
    });
}

#[test]
fn every_record_text_fronts_retain_recovered_closures_as_unclean_values() {
    each_record(|case| {
        let mut texts: Vec<(String, Option<CssGlobalKeyword>, bool)> = GLOBALS
            .iter()
            .map(|(text, keyword)| (format!("{text}/*unfinished"), Some(*keyword), false))
            .collect();
        texts.extend(
            ["var(--known", "env(known", "attr(data-known"]
                .map(|text| (text.to_owned(), None, true)),
        );
        texts.extend(
            case.accepted_rows()
                .first()
                .map(|(authored, _)| (format!("{authored}/*"), None, false)),
        );
        for (text, keyword, pending) in &texts {
            for importance in IMPORTANCE {
                for grammar in [false, true] {
                    let report = text_report(case, text, grammar, importance);
                    assert!(!report.is_clean(), "{text:?}/{grammar}/{importance:?}");
                    let source = report
                        .syntax()
                        .as_ref()
                        .unwrap_or_else(|| panic!("{text:?}: retained recovered value"));
                    let known = source.known().unwrap();
                    assert_eq!(known.property(), case.property);
                    assert_eq!(source.importance(), importance);
                    assert_eq!(known.global(), *keyword, "{text:?}");
                    assert_eq!(
                        known.substitution_dependent().is_some(),
                        *pending,
                        "{text:?}"
                    );
                    // Syntax 3 §4.3.2 ignores an unterminated comment;
                    // §5.4.8/§5.4.9 close a function or block at EOF.
                    let action = if text.contains("/*") {
                        CssRecoveryAction::IgnoreUnterminatedComment
                    } else {
                        CssRecoveryAction::RetainWithImplicitClosure
                    };
                    let actions: Vec<_> = report
                        .diagnostics()
                        .iter()
                        .map(|diagnostic| diagnostic.action())
                        .collect();
                    assert_eq!(actions, [action], "{text:?}");
                }
            }
        }
    });
}

#[test]
fn every_record_sheet_recovery_retains_original_occurrences_within_exact_contribution_budgets() {
    each_record(|case| {
        let members = css_wide_members(case).map_or(1, |members| members.len());
        let mut texts: Vec<(String, usize)> = GLOBALS
            .iter()
            .map(|(text, _)| (format!("{text}/*"), members))
            .collect();
        texts.extend(
            [
                "var(--known",
                "env(known",
                "attr(data-known",
                "attr(data-known *",
            ]
            .map(|text| (text.to_owned(), 1)),
        );
        for (authored, _) in case.accepted_rows() {
            let source = admit(Front::CheckedName, case, authored, CssImportance::Normal);
            let count = longhands(completed(&source)).items().len();
            texts.push((format!("{authored}/*"), count));
        }
        for (text, count) in texts
            .iter()
            .flat_map(|(text, count)| [("", text, *count), (PREFIX, text, *count)])
            .map(|(prefix, text, count)| (format!("{prefix}.a{{{}:{text}", case.name), count))
        {
            let css = text;
            let report = parse_sheet(&css);
            assert!(!report.is_clean(), "{css}");
            assert_implicit_closure(&css, &report);
            assert_eq!(
                validate_sheet(&css).unwrap_err().diagnostics(),
                report.diagnostics(),
                "{css}"
            );
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|diagnostic| diagnostic.action()
                        == CssRecoveryAction::RetainWithImplicitClosure),
                "{css}"
            );
            let before = report.clone();
            let exact = CssNormalizationLimits::try_new(0, 1, 1, count).unwrap();
            let normalized = normalize_report_with_limits(&report, exact).unwrap();
            assert_eq!(normalized.diagnostics(), report.diagnostics());
            let declarations: Vec<_> = normalized
                .syntax()
                .items()
                .iter()
                .filter_map(|item| match item {
                    CssNormalizedItem::Declaration(value) => Some(value),
                    _ => None,
                })
                .collect();
            let [item] = declarations.as_slice() else {
                panic!("{css}: retained recovered declaration")
            };
            assert_eq!(item.order(), 0);
            assert_eq!(item.source().known().unwrap().property(), case.property);
            assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
            assert_eq!(item.source().parsed_value().unwrap().source().as_str(), css);
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(values.items().len(), count, "{css}");
                    for value in values.items() {
                        assert_source(value, item.source());
                    }
                }
                CssExpansion::Contributions(CssContributions::UniversalReset(reset)) => {
                    assert!(reset.source().same_occurrence(item.source()))
                }
                other => panic!("{css}: retained lifecycle {other:?}"),
            }
            let error = normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, count - 1).unwrap(),
            )
            .unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded {
                    resource: CssNormalizationResource::Contributions,
                    limit: count - 1,
                },
                "{css}"
            );
            assert_eq!(error.declaration_order(), Some(0));
            assert!(error.declaration().unwrap().same_occurrence(item.source()));
            assert_eq!(report, before);
            assert!(normalize_report_with_limits(&report, exact).is_ok());
        }
    });
}

#[test]
fn every_record_css_wide_keyword_is_case_insensitive_escapable_and_serializes_in_lowercase() {
    // Syntax 3 §4.3.11 decodes escapes and Cascade 4 §7.3 matches the
    // keywords ASCII case-insensitively; CSSOM 1 §6.7.2 writes them lowercase.
    const SPELLINGS: [(&str, &str, CssGlobalKeyword); 5] = [
        ("INITIAL", "initial", CssGlobalKeyword::Initial),
        (r"\69 nherit", "inherit", CssGlobalKeyword::Inherit),
        ("UnSet", "unset", CssGlobalKeyword::Unset),
        ("/**/revert/**/", "revert", CssGlobalKeyword::Revert),
        (
            "Revert-Layer",
            "revert-layer",
            CssGlobalKeyword::RevertLayer,
        ),
    ];
    each_record(|case| {
        for (authored, canonical, keyword) in SPELLINGS {
            for importance in IMPORTANCE {
                for front in FRONTS {
                    let label = format!("{authored} via {front:?}/{importance:?}");
                    let source = admit(front, case, authored, importance);
                    assert_eq!(source.known().unwrap().global(), Some(keyword), "{label}");
                    assert_specified(
                        &source,
                        &specified(case.name, canonical, importance),
                        &label,
                    );
                    assert_global_contributions(case, completed(&source), &source, keyword, None);
                }
            }
        }
    });
}

#[test]
fn every_record_css_wide_keyword_is_only_valid_as_the_whole_value() {
    // Cascade 4 §7.3: a CSS-wide keyword cannot combine with other component
    // values, and Values 4 §4.2 excludes it from `<custom-ident>`.
    each_record(|case| {
        for (text, _) in GLOBALS {
            let mut values: Vec<String> = GLOBALS
                .iter()
                .map(|(other, _)| format!("{text} {other}"))
                .collect();
            values.push(format!("{text} extra"));
            for (authored, _) in case.accepted_rows() {
                values.push(format!("{text} {authored}"));
                values.push(format!("{authored} {text}"));
                values.push(format!("{text},{authored}"));
                values.push(format!("{authored},{text}"));
            }
            for value in values {
                invalid(case, &value);
            }
        }
    });
}

#[test]
fn every_record_member_participates_in_the_universal_reset_except_direction_and_unicode_bidi() {
    // Cascade 4 §3.2: `all` resets every property except `direction` and
    // `unicode-bidi`, and custom properties are not affected.
    let all = property_expectations::find(P::All).expect("all record");
    let CssPropertyKindRef::UniversalReset(metadata) = P::All.metadata().unwrap().kind() else {
        panic!("all metadata")
    };
    let custom = CssCustomPropertyName::try_new("--known").unwrap();
    for (text, keyword) in GLOBALS {
        for importance in IMPORTANCE {
            let source = admit(Front::Attribute, all, text, importance);
            let CssContributions::UniversalReset(reset) = completed(&source) else {
                panic!("universal reset")
            };
            assert_eq!(reset.keyword(), keyword);
            assert!(reset.excludes(CssPropertyNameRef::Custom(&custom)));
            assert!(metadata.excludes(CssPropertyNameRef::Custom(&custom)));
            each_record(|case| {
                for member in css_wide_members(case).unwrap_or_default() {
                    let excluded = matches!(member, P::Direction | P::UnicodeBidi);
                    assert_eq!(
                        reset.excludes(CssPropertyNameRef::Known(member)),
                        excluded,
                        "{member:?}"
                    );
                    assert_eq!(
                        metadata.excludes(CssPropertyNameRef::Known(member)),
                        excluded,
                        "{member:?} metadata"
                    );
                }
            });
        }
    }
}

#[test]
fn every_record_alias_keeps_the_canonical_lifecycle_and_alias_name_provenance() {
    each_record(|case| {
        let accepted = case.accepted_rows();
        for alias in case.aliases {
            for spelling in [(*alias).to_owned(), alias.to_ascii_uppercase()] {
                let mut values: Vec<(String, String)> = GLOBALS
                    .iter()
                    .map(|(text, _)| ((*text).to_owned(), (*text).to_owned()))
                    .collect();
                values.extend(PENDING.map(|text| (text.to_owned(), text.to_owned())));
                values.extend(accepted.iter().map(|(authored, canonical)| {
                    ((*authored).to_owned(), (*canonical).to_owned())
                }));
                for (value, canonical) in values {
                    for importance in IMPORTANCE {
                        let css =
                            format!("{PREFIX}{spelling}:{value}{}", important_suffix(importance));
                        let report = parse_style_attribute(&css);
                        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
                        let [source] = report.syntax().as_slice() else {
                            panic!("{css}: one alias occurrence")
                        };
                        assert_eq!(source.known().unwrap().property(), case.property);
                        assert_eq!(source.known().unwrap().grammar(), case.property.grammar());
                        assert_eq!(source.importance(), importance);
                        let name = source.parsed_name().unwrap();
                        assert_eq!(name.source().as_str(), css);
                        assert_eq!(name.span().start().byte_offset().value(), PREFIX.len());
                        assert_eq!(
                            name.span().end().byte_offset().value(),
                            PREFIX.len() + spelling.len()
                        );
                        assert_eq!(source.position(), Some(name.span().start()));
                        assert_specified(
                            source,
                            &specified(case.name, &canonical, importance),
                            &css,
                        );
                        match expand_declaration(source).unwrap() {
                            CssExpansion::Pending(handle) => {
                                assert!(handle.source().same_occurrence(source));
                                let replacement = parse_component_values("inherit").unwrap();
                                assert_global_contributions(
                                    case,
                                    handle.reenter(replacement.clone()).unwrap(),
                                    source,
                                    CssGlobalKeyword::Inherit,
                                    Some(&replacement),
                                );
                                for (authored, _) in &accepted {
                                    let replacement = parse_component_values(authored).unwrap();
                                    assert_ordinary_contributions(
                                        case,
                                        &longhands(handle.reenter(replacement.clone()).unwrap()),
                                        source,
                                        Some(&replacement),
                                    );
                                }
                            }
                            CssExpansion::Contributions(contributions) => {
                                if let Some(keyword) = source.known().unwrap().global() {
                                    assert_global_contributions(
                                        case,
                                        contributions,
                                        source,
                                        keyword,
                                        None,
                                    );
                                } else {
                                    assert_ordinary_contributions(
                                        case,
                                        &longhands(contributions),
                                        source,
                                        None,
                                    );
                                }
                            }
                            other => panic!("{css}: {other:?}"),
                        }
                    }
                }
            }
        }
    });
}

#[test]
fn every_record_checked_fronts_reject_an_authored_importance_annotation_at_its_token() {
    // CSSOM 1 §6.7.2 parses a value without `!important`; the annotation is
    // invalid grammar at its `!` token for checked and text fronts alike.
    each_record(|case| {
        for (authored, _) in case.accepted_rows() {
            let text = format!("{PREFIX}{authored}!important");
            let components = parse_component_values(&text).unwrap();
            let serialized = components.serialize().unwrap();
            let origin = serialized
                .origin_at(serialized.as_css().rfind('!').unwrap())
                .unwrap();
            for importance in IMPORTANCE {
                for grammar in [false, true] {
                    let error = authored_property::checked_components(
                        case.property,
                        components.clone(),
                        grammar,
                        importance,
                    )
                    .unwrap_err();
                    assert!(
                        matches!(error.kind(), CssPropertyValueErrorKind::Grammar(_)),
                        "{text}: {error:?}"
                    );
                    assert_eq!(error.origin(), origin, "{text}");
                    let report = text_report(case, &text, grammar, importance);
                    assert!(report.syntax().is_none(), "{text}: retained");
                    assert!(!report.is_clean(), "{text}: diagnostic");
                }
            }
        }
    });
}

#[test]
fn every_record_name_is_ascii_case_insensitive_and_escapable_at_every_lookup() {
    // Syntax 3 §4.3.11 decodes escapes; CSS property names match ASCII
    // case-insensitively (Cascade 4 §1.1, CSSOM 1 §6.7.1).
    each_record(|case| {
        let upper = case.name.to_ascii_uppercase();
        assert_eq!(P::from_name(case.name), Some(case.property));
        assert_eq!(P::from_name(&upper), Some(case.property));
        assert_eq!(
            CssPropertyGrammar::from_name(case.name),
            Some(case.property.grammar())
        );
        assert_eq!(
            CssPropertyGrammar::from_name(&upper),
            Some(case.property.grammar())
        );
        let first = case.name.chars().next().unwrap();
        let escaped = format!(
            "\\{:x} {}",
            u32::from(first),
            &case.name[first.len_utf8()..]
        );
        let value = case
            .accepted_rows()
            .first()
            .map_or("inherit", |(authored, _)| *authored);
        let canonical = case
            .accepted_rows()
            .first()
            .map_or("inherit", |(_, canonical)| *canonical);
        for importance in IMPORTANCE {
            let css = format!("{escaped}:{value}{}", important_suffix(importance));
            let report = parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            let [source] = report.syntax().as_slice() else {
                panic!("{css}: one declaration")
            };
            assert_eq!(source.known().unwrap().property(), case.property, "{css}");
            assert_eq!(
                source.to_specified_css().unwrap(),
                specified(case.name, canonical, importance),
                "{css}"
            );
        }
    });
}

#[test]
fn every_record_substitution_after_an_ordinary_value_defers_the_whole_value() {
    // CSS Variables 1 §3: a substitution function anywhere in the value makes
    // the whole value pending; the replacement replaces the whole value.
    each_record(|case| {
        let accepted = case.accepted_rows();
        let Some((authored, canonical)) = accepted.first().copied() else {
            return;
        };
        for pending in PENDING {
            let value = format!("{canonical} {pending}");
            for importance in IMPORTANCE {
                for front in FRONTS {
                    let label = format!("{value} via {front:?}/{importance:?}");
                    let source = admit(front, case, &value, importance);
                    let before = source.clone();
                    let known = source.known().unwrap();
                    assert!(known.substitution_dependent().is_some(), "{label}");
                    assert!(known.property_value().is_none(), "{label}");
                    assert_specified(&source, &specified(case.name, &value, importance), &label);
                    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                        panic!("{label}: whole-value pending")
                    };
                    assert!(handle.source().same_occurrence(&source), "{label}");
                    let replacement = parse_component_values(authored).unwrap();
                    let values = longhands(handle.reenter(replacement.clone()).unwrap());
                    assert_ordinary_contributions(case, &values, &source, Some(&replacement));
                    let replacement = parse_component_values("unset").unwrap();
                    assert_global_contributions(
                        case,
                        handle.reenter(replacement.clone()).unwrap(),
                        &source,
                        CssGlobalKeyword::Unset,
                        Some(&replacement),
                    );
                    assert_eq!(source, before, "{label}");
                }
            }
        }
    });
}
