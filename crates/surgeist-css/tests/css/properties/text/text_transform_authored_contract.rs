//! Text4 WD20260814 #propdef-text-transform and Values4 §2.2 supply grammar,
//! none initial and inheritance. Canonical case/width/kana order and transparent
//! emitted-keyword pricing are explicit product selections, not normative
//! canonical-order claims (Text4 says n/a). Existing singleton costs remain 1/1;
//! declaration/name add two, spaces and punctuation charge only UTF-8 bytes.

use crate::css::support::authored_property;
use authored_property::{ParserFront, assert_source, checked, checked_components, invalid, parsed};

use crate::css::support::named_property::sole_completed_contributions as completed;
use crate::css::support::serialization_limits::LimitLadder;
use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const PROPERTY: P = P::TextTransform;
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
// The literal 17-state oracle follows three independent roles in the grammar,
// with one optional case alternative and two distinct optional flags, nonempty.
const STATES: [&str; 17] = [
    "none",
    "math-auto",
    "capitalize",
    "uppercase",
    "lowercase",
    "full-width",
    "full-size-kana",
    "full-width full-size-kana",
    "capitalize full-width",
    "capitalize full-size-kana",
    "capitalize full-width full-size-kana",
    "uppercase full-width",
    "uppercase full-size-kana",
    "uppercase full-width full-size-kana",
    "lowercase full-width",
    "lowercase full-size-kana",
    "lowercase full-width full-size-kana",
];
const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];
fn primitive(source: &CssDeclaration) -> &CssTextTransform {
    let CssKnownPropertyValueRef::TextTransform(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("existing typed TextTransform wrapper")
    };
    wrapper.value()
}
fn canonical(source: &CssDeclaration, expected: &str) {
    assert_eq!(primitive(source).serialize_specified().unwrap(), expected);
    assert_eq!(
        source.to_specified_css().unwrap(),
        format!("text-transform: {expected} !important;")
    );
}
fn accepted(input: &str, expected: &str) {
    for front in FRONTS {
        let source = front.valid(PROPERTY, input, expected);
        let before = source.clone();
        assert_eq!(primitive(&source).serialize_specified().unwrap(), expected);
        let values = completed(&source);
        let item = &values.items()[0];
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            PROPERTY
        );
        assert!(item.replacement_components().is_none());
        assert_eq!(source, before);
    }
    canonical(&checked(PROPERTY, expected, true), expected);
}

#[test]
fn all_seventeen_states_and_every_role_permutation_emit_idempotent_canonical_values() {
    for expected in STATES {
        let words: Vec<_> = expected.split_whitespace().collect();
        accepted(expected, expected);
        if words.len() == 2 {
            accepted(&format!("{} {}", words[1], words[0]), expected);
        }
        if words.len() == 3 {
            for [a, b, c] in [[0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
                accepted(&format!("{} {} {}", words[a], words[b], words[c]), expected);
            }
        }
    }
}
#[test]
fn decoded_keywords_comments_and_case_preserve_authored_syntax_and_canonical_roles() {
    for (input, expected) in [
        (
            "FULL-SIZE-KANA/**/UPPERCASE /**/FULL-WIDTH",
            "uppercase full-width full-size-kana",
        ),
        (
            r"\66 ull-width \75 ppercase \66 ull-size-kana",
            "uppercase full-width full-size-kana",
        ),
        (r"\6d ath-auto", "math-auto"),
        (r"\6e one", "none"),
    ] {
        accepted(input, expected);
        let source = parsed(PROPERTY, input);
        let CssKnownPropertyValueRef::TextTransform(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed wrapper")
        };
        assert_eq!(wrapper.as_css(), input);
    }
}
#[test]
fn duplicate_conflicting_and_exclusive_choices_reject_atomically_with_color_siblings() {
    for value in [
        "",
        "auto",
        "normal",
        "fullwidth",
        "full-size",
        "1",
        "-1px",
        "25%",
        "calc(1 + 2)",
        "min(1,2)",
        "uppercase,full-width",
        "[uppercase]",
        "\"uppercase\"",
        "uppercase uppercase",
        "full-width full-width",
        "full-size-kana full-size-kana",
        "full-width uppercase full-width",
        "full-size-kana lowercase full-size-kana",
        "uppercase full-width full-size-kana lowercase",
        "none none",
        "math-auto math-auto",
        "initial uppercase",
        "uppercase inherit",
    ] {
        invalid(PROPERTY, value);
    }
    for left in ["capitalize", "uppercase", "lowercase"] {
        for right in ["capitalize", "uppercase", "lowercase"] {
            invalid(PROPERTY, &format!("{left} full-width {right}"));
        }
    }
    for exclusive in ["none", "math-auto"] {
        for other in [
            "none",
            "math-auto",
            "capitalize",
            "uppercase",
            "lowercase",
            "full-width",
            "full-size-kana",
        ] {
            invalid(PROPERTY, &format!("{exclusive} {other}"));
            invalid(PROPERTY, &format!("{other} {exclusive}"));
        }
    }
}
#[test]
fn singleton_primitive_provider_controls_execute_without_intrinsic_metadata() {
    for value in ["none", "capitalize", "uppercase", "lowercase"] {
        let source = checked(PROPERTY, value, false);
        canonical(&source, value);
        assert_primitive_limits(primitive(&source), value, 1);
    }
}
#[test]
fn programmatic_components_have_checked_identity_importance_and_original_origins() {
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("uppercase").unwrap()])
            .unwrap();
    for grammar in [false, true] {
        let source = checked_components(
            PROPERTY,
            components.clone(),
            grammar,
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(source.value_components(), &components);
        assert_eq!(
            source.value_components().items()[0].origin(),
            &CssValueOrigin::Programmatic
        );
        assert_eq!(source.importance(), CssImportance::Important);
        assert!(source.parsed_value().is_none());
        canonical(&source, "uppercase");
        completed(&source);
    }
}
#[test]
fn intrinsic_metadata_is_one_inherited_ordinary_initial_terminal() {
    assert_eq!(P::from_name("TEXT-TRANSFORM"), Some(PROPERTY));
    let CssPropertyKindRef::Longhand(meta) = PROPERTY.metadata().unwrap().kind() else {
        panic!("longhand metadata")
    };
    assert_eq!(meta.property().known_property(), PROPERTY);
    assert!(meta.inherited_by_default());
    let initial = meta.initial_value();
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("ordinary none initial")
    };
    assert_eq!(value.property().known_property(), PROPERTY);
    let none = checked(PROPERTY, "none", true);
    canonical(&none, "none");
    let none_contribution = completed(&none);
    assert_eq!(none_contribution.items()[0].ordinary_value(), Some(value));
    // The opaque initial equals the authored none contribution. Direct enum
    // payload inspection additionally belongs with the new borrowed view.
    accepted("none", "none");
}
#[test]
fn normalization_counts_terminals_and_occurrences_cumulatively_preserving_order_contexts() {
    let report = parse_sheet(
        "@media screen{.a{text-transform:uppercase!important;text-transform:full-size-kana full-width;text-transform:var(--transform);text-transform:unset!important;color:red}}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    // Four TextTransform occurrences each contribute one terminal/handle, plus
    // the supported color control: five occurrences and five contributions.
    let exact = CssNormalizationLimits::try_new(1, 2, 5, 5).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 5);
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property(),
            if index == 4 { P::Color } else { PROPERTY }
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 || index == 3 {
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
                assert_eq!(index, 2);
                assert!(handle.source().same_occurrence(item.source()));
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1);
                assert_source(&values.items()[0], item.source());
                if index == 3 {
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
            _ => panic!("intrinsic expansion"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 2, 5, 4).unwrap(),
            CssNormalizationResource::Contributions,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 4, 5).unwrap(),
            CssNormalizationResource::Declarations,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit: 4 }
            );
            assert_eq!(error.declaration_order(), Some(4));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[4].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
fn assert_primitive_limits(value: &CssTextTransform, expected: &str, nodes: usize) {
    LimitLadder::new(expected, nodes, nodes)
        .assert_unchanged(value, CssTextTransform::serialize_specified_with_limits);
}
#[test]
fn primitive_and_declaration_resources_charge_only_emitted_keywords_and_retry_atomically() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for text in STATES {
        let source = checked(PROPERTY, text, true);
        // This is the adopted semantic tariff: one per keyword. Counting the
        // independent literal canonical oracle never measures implementation work.
        let nodes = text.split_whitespace().count();
        assert_primitive_limits(primitive(&source), text, nodes);
        let expected = format!("text-transform: {text} !important;");
        let before = source.clone();
        let exact = Limits::new(nodes + 2, nodes + 2, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(nodes + 1, nodes + 2, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes + 2, nodes + 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes + 2, nodes + 2, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, before);
            }
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
    }
    for (text, _) in GLOBALS {
        let source = checked(PROPERTY, text, true);
        let expected = format!("text-transform: {text} !important;");
        assert_eq!(
            source
                .to_specified_css_with_limits(Limits::new(3, 3, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (Limits::new(2, 3, expected.len()), Kind::InputNodeLimit),
            (Limits::new(3, 2, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(3, 3, expected.len() - 1), Kind::ByteLimit),
        ] {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(
            source
                .to_specified_css_with_limits(Limits::new(3, 3, expected.len()))
                .unwrap(),
            expected
        );
    }
}
#[test]
fn whole_sheet_siblings_share_exact_utf8_bytes_and_failed_attempts_leave_reusable_input() {
    use CssSpecifiedValueSerializationLimits as Limits;
    let report = parse_sheet(
        ".a{text-transform:full-size-kana uppercase full-width!important}.b{text-transform:math-auto}",
    );
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { text-transform: uppercase full-width full-size-kana !important; }\n.b { text-transform: math-auto; }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|rule| rule.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
#[test]
fn selected_numeric_flow_and_color_controls_remain_independent_of_new_transform_metadata() {
    let report = parse_style_attribute("flow-tolerance:calc(-2px - 3%);color:red");
    assert!(report.is_clean());
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "flow-tolerance: calc(-3% - 2px);"
    );
    for source in report.syntax().iter() {
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(source).unwrap()
        else {
            panic!("supported control")
        };
        assert_eq!(values.items().len(), 1);
        assert_source(&values.items()[0], source);
    }
}
