#![forbid(unsafe_code)]
//! Text4 WD20260814 #propdef-text-transform and Values4 §2.2 supply grammar,
//! none initial and inheritance. Canonical case/width/kana order and transparent
//! emitted-keyword pricing are explicit product selections, not normative
//! canonical-order claims (Text4 says n/a). Existing singleton costs remain 1/1;
//! declaration/name add two, spaces and punctuation charge only UTF-8 bytes.

#[path = "common/authored_property.rs"]
mod authored_property;
use authored_property::{ParserFront, assert_source, checked, checked_components, invalid, parsed};

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
fn fronts(value: &str) -> [CssDeclaration; 5] {
    FRONTS.map(|front| front.parse(PROPERTY, value))
}
fn primitive(source: &CssDeclaration) -> &CssTextTransform {
    let CssKnownPropertyValueRef::TextTransform(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("existing typed TextTransform wrapper")
    };
    wrapper.value()
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("intrinsic terminal contribution")
    };
    assert_eq!(values.items().len(), 1);
    let item = &values.items()[0];
    assert_eq!(item.property(), PROPERTY);
    assert_source(item, source);
    values
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
fn existing_singleton_primitive_provider_controls_execute_without_intrinsic_metadata() {
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
fn every_css_wide_keyword_is_symbolic_and_keeps_source_and_importance() {
    for (text, keyword) in GLOBALS {
        for source in fronts(text) {
            assert_eq!(source.known().unwrap().global(), Some(keyword));
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("text-transform: {text} !important;")
            );
            let values = completed(&source);
            assert!(
                matches!(values.items()[0].value(), CssContributionValueRef::Global(v) if v == keyword)
            );
            assert!(values.items()[0].replacement_components().is_none());
        }
    }
}
#[test]
fn var_env_attr_reentry_preserves_original_occurrence_and_replacement_snapshot() {
    for pending in [
        "var(--transform)",
        "env(transform)",
        "attr(data-transform *)",
    ] {
        for source in fronts(pending) {
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("text-transform: {pending} !important;")
            );
            for text in STATES {
                let replacement = parse_component_values(text).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("ordinary reentry")
                };
                assert_eq!(values.items().len(), 1);
                let item = &values.items()[0];
                assert_eq!(item.property(), PROPERTY);
                assert_source(item, &source);
                assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                assert_eq!(item.replacement_components(), Some(&replacement));
                for (actual, supplied) in item
                    .replacement_components()
                    .unwrap()
                    .items()
                    .iter()
                    .zip(replacement.items())
                {
                    assert_eq!(actual.origin(), supplied.origin());
                    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                        (actual.origin(), supplied.origin())
                    else {
                        panic!("replacement snapshot")
                    };
                    assert!(actual.source().same_snapshot(supplied.source()));
                    assert_eq!(actual.source().as_str(), text);
                }
                canonical(&checked(PROPERTY, text, false), text);
            }
            for (text, keyword) in GLOBALS {
                let replacement = parse_component_values(text).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("global reentry")
                };
                assert_eq!(values.items().len(), 1);
                let item = &values.items()[0];
                assert_source(item, &source);
                assert!(matches!(item.value(), CssContributionValueRef::Global(v) if v == keyword));
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
            assert_eq!(source, before);
        }
    }
}
#[test]
fn residual_and_invalid_replacements_fail_repeatedly_then_the_same_handle_succeeds() {
    let source = checked(PROPERTY, "var(--transform)", true);
    let before = source.clone();
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    for text in [
        "var(--again)",
        "env(again)",
        "attr(data-again *)",
        "full-width var(--again)",
        r"\76 ar(--again)",
    ] {
        for _ in 0..2 {
            assert!(matches!(
                handle
                    .reenter(parse_component_values(text).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::ResidualSubstitution
            ));
        }
    }
    for text in [
        "uppercase lowercase",
        "none full-width",
        "math-auto full-size-kana",
        "uppercase!important",
        "uppercase;color:red",
    ] {
        for _ in 0..2 {
            let error = handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("strict replacement")
            };
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
        }
    }
    let CssContributions::Longhands(values) = handle
        .reenter(parse_component_values("full-size-kana uppercase full-width").unwrap())
        .unwrap()
    else {
        panic!("reusable successful retry")
    };
    assert_eq!(values.items().len(), 1);
    assert_source(&values.items()[0], &source);
    assert!(handle.source().same_occurrence(&source));
    assert_eq!(source, before);
}
fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("public original implicit closing origin")
}
fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin, text: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("implicit EOF")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}
#[test]
fn checked_original_closure_and_strict_reentry_cover_ordinary_global_and_all_pending_forms() {
    // Independent existing checked admission: this group must reach closure
    // assertions even before intrinsic metadata/expansion is implemented.
    let mut texts = vec![
        "uppercase/*".to_owned(),
        "full-width uppercase/*".to_owned(),
    ];
    texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
    texts.extend(
        [
            "var(--transform)/*",
            "env(transform)/*",
            "attr(data-transform *)/*",
            "var(--transform",
            "env(transform",
            "attr(data-transform *",
        ]
        .map(str::to_owned),
    );
    for text in &texts {
        let components = parse_component_values(text).unwrap();
        let before = components.clone();
        let origin = implicit_origin(&components);
        for grammar in [false, true] {
            assert_closure(
                &checked_components(
                    PROPERTY,
                    components.clone(),
                    grammar,
                    CssImportance::Important,
                )
                .unwrap_err(),
                &origin,
                text,
            );
        }
        assert_eq!(components, before);
    }
}
#[test]
fn pending_reentry_rejects_original_closures_and_complete_comment_controls_succeed() {
    let source = checked(PROPERTY, "var(--transform)", false);
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    for text in [
        "uppercase/*",
        "initial/*",
        "full-size-kana uppercase full-width/*",
    ] {
        let components = parse_component_values(text).unwrap();
        let origin = implicit_origin(&components);
        for _ in 0..2 {
            let error = handle.reenter(components.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("strict original closure")
            };
            assert_closure(error, &origin, text);
        }
    }
    for text in ["var(--again", "env(again", "attr(data-again *"] {
        assert!(matches!(
            handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::ResidualSubstitution
        ));
    }
    for text in [
        "uppercase/**/",
        "initial/**/",
        "var(--transform)/**/",
        "env(transform)/**/",
        "attr(data-transform *)/**/",
    ] {
        checked(PROPERTY, text, false);
        checked(PROPERTY, text, true);
    }
    assert!(
        handle
            .reenter(parse_component_values("uppercase/**/").unwrap())
            .is_ok()
    );
}
#[test]
fn value_annotations_fail_at_original_tokens_while_authored_importance_remains_valid() {
    let text = "/*😀*/uppercase!important";
    let components = parse_component_values(text).unwrap();
    let serialized = components.serialize().unwrap();
    let origin = serialized
        .origin_at(serialized.as_css().find('!').unwrap())
        .unwrap();
    for grammar in [false, true] {
        let error = checked_components(
            PROPERTY,
            components.clone(),
            grammar,
            CssImportance::Important,
        )
        .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        assert_eq!(error.origin(), origin);
    }
    canonical(&parsed(PROPERTY, "uppercase"), "uppercase");
}
#[test]
fn browser_eof_recovery_retains_diagnostics_and_normalized_original_occurrence() {
    for text in ["uppercase/*", "var(--transform"] {
        let css = format!(".a{{text-transform:{text}");
        let report = parse_sheet(&css);
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declarations: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|v| match v {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        let [item] = declarations.as_slice() else {
            panic!("retained recovery")
        };
        assert_eq!(item.order(), 0);
        assert_eq!(item.source().known().unwrap().property(), PROPERTY);
        assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
        assert_eq!(report, before);
    }
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
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let before = *value;
    let exact = Limits::new(nodes, nodes, expected.len());
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(nodes - 1, nodes, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(nodes, nodes - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(nodes, nodes, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        for _ in 0..2 {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, &before);
        }
    }
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        expected
    );
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
