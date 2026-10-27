//! Existing-public-API authored wrapping control contract.
//! Text4 WD20260814 #propdef-wrap-inside (1602–1622), #propdef-wrap-before /
//! #propdef-wrap-after (1680–1736), #propdef-line-break (2486–2521) supply
//! independent finite keyword/initial/inheritance oracles. Text3 CRD20260814
//! agrees on line-break. Syntax3 recovery and existing strict checked contracts
//! distinguish browser implicit closure from original-component admission.
//! Adopted keyword price is1input/1projection; declaration/name add2. Output
//! punctuation/importance cost bytes only; whole-sheet work totals are not guessed.
//! No new typed models, wrapper symbols, borrowed variants, stubs or private
//! helpers are referenced. Exact new payload/primitive provider/suppression and
//! new typed API tests accompany functional implementation. Opaque initial/value
//! equality is exercised through existing generic public boundaries.
//! Nested breaking, flex/forced propagation, language and white-space execution
//! remain contextual. This group neither changes nor certifies #53 constituents.

use crate::css::support::{
    authored_checked_declaration as checked, authored_parsed_declaration as parsed,
    known_property as property,
};
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues};
use surgeist_css::declaration::{CssGlobalKeyword, CssImportance};
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssPropertyKindRef,
};
use surgeist_css::serialization::CssSerializationLimits;

use crate::css::support::authored_property;
use crate::css::support::check_named_components as checked_components;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::named_property::invalid_named_value as invalid;
use crate::css::support::named_property::sole_completed_contributions as completed;
use authored_property::assert_source;

const FAMILY: [&str; 4] = ["wrap-inside", "wrap-before", "wrap-after", "line-break"];
const INSIDE: [&str; 2] = ["auto", "avoid"];
const BOUNDARY: [&str; 6] = ["auto", "avoid", "avoid-line", "avoid-flex", "line", "flex"];
const LINE_BREAK: [&str; 5] = ["auto", "loose", "normal", "strict", "anywhere"];
fn keywords(name: &str) -> &'static [&'static str] {
    match name {
        "wrap-inside" => &INSIDE,
        "wrap-before" | "wrap-after" => &BOUNDARY,
        "line-break" => &LINE_BREAK,
        _ => panic!("selected finite grammar"),
    }
}

#[test]
fn duplicate_and_multiple_choices_reject_atomically_and_keep_color_siblings() {
    for name in FAMILY {
        for left in keywords(name) {
            for right in keywords(name) {
                invalid(name, &format!("{left} {right}"));
            }
        }
        for value in [
            "",
            "none",
            "nowrap",
            "wrap",
            "pretty",
            "balance",
            "1",
            "-1px",
            "25%",
            "calc(1 + 2)",
            "min(1,2)",
            "auto,auto",
            "[auto]",
            "\"auto\"",
            "initial auto",
            "auto inherit",
        ] {
            invalid(name, value);
        }
    }
    for value in [
        "avoid-line",
        "avoid-flex",
        "line",
        "flex",
        "loose",
        "normal",
        "strict",
        "anywhere",
    ] {
        invalid("wrap-inside", value);
    }
    for name in ["wrap-before", "wrap-after"] {
        for value in ["loose", "normal", "strict", "anywhere"] {
            invalid(name, value);
        }
    }
    for value in ["avoid", "avoid-line", "avoid-flex", "line", "flex"] {
        invalid("line-break", value);
    }
}
#[test]
fn auto_initials_are_ordinary_longhands_and_only_line_break_inherits() {
    for name in FAMILY {
        let p = property(name);
        let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
            panic!("one longhand, no shorthand/reset members")
        };
        assert_eq!(metadata.property().known_property(), p);
        assert_eq!(metadata.inherited_by_default(), name == "line-break");
        let initial = metadata.initial_value();
        assert_eq!(initial.property().known_property(), p);
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary auto initial")
        };
        let components = CssComponentValues::try_new(
            vec![CssComponentValue::try_ident("auto").unwrap()],
            Default::default(),
        )
        .unwrap();
        let auto = checked_components(name, components, false).unwrap();
        assert_eq!(
            auto.serialize_specified(Default::default()).unwrap(),
            format!("{name}: auto !important;")
        );
        let values = completed(&auto);
        assert_eq!(values.items()[0].ordinary_value(), Some(initial));
    }
}
#[test]
fn normalization_preserves_order_contexts_importance_and_exact_cumulative_terminal_counts() {
    let report = CssParser::default().parse_sheet("@media screen{.a{wrap-inside:avoid!important;wrap-before:line;wrap-after:flex;line-break:anywhere;wrap-before:var(--before);wrap-after:unset!important;line-break:normal;wrap-inside:auto;color:red}}");
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 9, 9).unwrap();
    let normalized = report.syntax().normalize(exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 9);
    let names = [
        "wrap-inside",
        "wrap-before",
        "wrap-after",
        "line-break",
        "wrap-before",
        "wrap-after",
        "line-break",
        "wrap-inside",
        "color",
    ];
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            names[index]
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 || index == 5 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        if index > 0 {
            assert!(
                !item
                    .source()
                    .same_occurrence(declarations[index - 1].source())
            );
        }
        match item.expansion() {
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 4);
                assert!(handle.source().same_occurrence(item.source()));
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1);
                assert_source(&values.items()[0], item.source());
                assert_eq!(
                    values.items()[0].property(),
                    item.source().known().unwrap().property()
                );
                if index == 5 {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                    ));
                } else {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Ordinary(_)
                    ));
                }
            }
            _ => panic!("intrinsic single terminal/handle"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 2, 9, 8).unwrap(),
            CssNormalizationResource::Contributions,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 8, 9).unwrap(),
            CssNormalizationResource::Declarations,
        ),
    ] {
        for _ in 0..2 {
            let error = report.syntax().normalize(limits).unwrap_err();
            assert_eq!(error.exhausted_limit(), Some((resource, 8)));
            assert_eq!(error.declaration_order(), Some(8));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[8].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(report.syntax().normalize(exact).is_ok());
}
#[test]
fn canonical_keyword_declarations_have_exact_node_byte_budgets_and_atomic_reusable_failures() {
    use crate::css::support::failure_classes::SerializationFailure as Kind;
    use CssSerializationLimits as Limits;
    for name in FAMILY {
        for &text in keywords(name) {
            for source in [parsed(name, text), checked(name, text, true)] {
                let before = source.clone();
                let expected = format!("{name}: {text} !important;");
                let exact = Limits::new(3, 3, expected.len());
                assert_eq!(source.serialize_specified(exact).unwrap(), expected);
                for (limits, kind) in [
                    (Limits::new(2, 3, expected.len()), Kind::InputNodeLimit),
                    (Limits::new(3, 2, expected.len()), Kind::ProjectionNodeLimit),
                    (Limits::new(3, 3, expected.len() - 1), Kind::ByteLimit),
                ] {
                    for _ in 0..2 {
                        assert_eq!(
                            serialization_failure(&source.serialize_specified(limits).unwrap_err()),
                            kind
                        );
                        assert_eq!(source, before);
                    }
                }
                assert_eq!(source.serialize_specified(exact).unwrap(), expected);
            }
        }
    }
}
#[test]
fn sheet_siblings_share_exact_final_bytes_and_failed_output_leaves_the_input_reusable() {
    use CssSerializationLimits as Limits;
    let report = CssParser::default().parse_sheet(".a{wrap-inside:avoid!important}.b{wrap-before:line}.c{wrap-after:flex}.d{line-break:anywhere}");
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { wrap-inside: avoid !important; }\n.b { wrap-before: line; }\n.c { wrap-after: flex; }\n.d { line-break: anywhere; }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.serialize_specified(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|rule| rule.serialize_specified(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.serialize_specified(short).unwrap_err();
        assert!(error.is_byte_limit(), "{error:?}");
        assert_eq!(error.rule_index(), Some(3));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.serialize_specified(exact).unwrap(), expected);
}
#[test]
fn flow_flex_grid_item_and_color_controls_execute_independently_of_wrap_lookup() {
    for (name, input, expected, count) in [
        (
            "flow-tolerance",
            "calc(-2px - 3%)",
            "flow-tolerance: calc(-3% - 2px);",
            1,
        ),
        (
            "flex-flow",
            "wrap row-reverse",
            "flex-flow: row-reverse wrap;",
            2,
        ),
        ("grid-template", "none", "grid-template: none;", 3),
        ("item-flow", "row", "item-flow: row auto normal normal;", 4),
        ("color", "red", "color: red;", 1),
    ] {
        let report = CssParser::default().parse_block_contents(format!("{name}:{input}"));
        assert!(report.is_clean());
        for source in report.syntax().iter() {
            assert_eq!(
                source.serialize_specified(Default::default()).unwrap(),
                expected
            );
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                source.expand().unwrap()
            else {
                panic!("supported independent control")
            };
            assert_eq!(values.items().len(), count);
            for item in values.items() {
                assert_source(item, source);
            }
        }
    }
}
