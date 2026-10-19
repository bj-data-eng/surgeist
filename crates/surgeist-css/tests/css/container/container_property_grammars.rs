#![forbid(unsafe_code)]
//! Independent expectations: Conditional5 2025-10-30 sections 5.1–5.3;
//! Values4 2024-03-12 sections 2 and 4.1–4.2; Syntax3 2021-12-24 section 4;
//! Cascade5 2022-01-13 sections 3 and 7.3; Variables1 2022-06-16 section 3.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-type
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#custom-idents
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#tokenization
//! https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#shorthand
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#variables-in-shorthands
//! No new type/variant assumptions, containment execution, or variable resolution.
use surgeist_css::*;

const CONDITIONAL5: &str = "https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/";

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name)
        .unwrap_or_else(|| panic!("the selected container property must be recognized: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_declaration(&source);
    assert!(report.is_clean(), "{source:?}: {report:?}");
    let result = report.syntax().as_ref().expect("one declaration");
    assert_eq!(result.known().unwrap().property().canonical_name(), name);
    assert_eq!(result.importance(), CssImportance::Important);
    result.clone()
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("intrinsic expansion supported")
    else {
        panic!("completed longhands")
    };
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
    values
}

fn ordinary(name: &str, value: &str) -> CssLonghandValue {
    let values = expanded(&declaration(name, value));
    let [item] = values.items() else {
        panic!("one terminal property")
    };
    assert_eq!(item.property().canonical_name(), name);
    item.ordinary_value().expect("ordinary longhand").clone()
}

fn member<'a>(values: &'a CssLonghandContributions, name: &str) -> &'a CssLonghandContribution {
    let mut matching = values
        .items()
        .iter()
        .filter(|item| item.property().canonical_name() == name);
    let result = matching.next().unwrap_or_else(|| panic!("missing {name}"));
    assert!(matching.next().is_none(), "no duplicate {name}");
    result
}

fn assert_pair(values: &CssLonghandContributions, names: &str, kind: &str) {
    // Name then type is the adopted schema order; the spec requires both members.
    assert_eq!(
        values
            .items()
            .iter()
            .map(|item| item.property().canonical_name())
            .collect::<Vec<_>>(),
        ["container-name", "container-type"]
    );
    assert_eq!(
        member(values, "container-name").ordinary_value(),
        Some(&ordinary("container-name", names))
    );
    assert_eq!(
        member(values, "container-type").ordinary_value(),
        Some(&ordinary("container-type", kind))
    );
}

fn accepted(name: &str, values: &[&str]) {
    for value in values {
        let source = declaration(name, value);
        assert!(
            source.known().unwrap().property_value().is_some(),
            "{name}:{value}"
        );
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
    }
}

fn rejected(name: &str, values: &[&str]) {
    // Recognition is required first: rejecting every unknown property cannot
    // falsely satisfy the intrinsic grammar rejection test.
    let _ = grammar(name);
    for value in values {
        let source = format!("color:red;{name}:{value};display:block");
        let report = parse_style_attribute(&source);
        let [before, after] = report.syntax().as_slice() else {
            panic!("only valid neighbors survive {source:?}: {report:?}")
        };
        assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(after.known().unwrap().property(), CssKnownProperty::Display);
        let [diagnostic] = report.diagnostics() else {
            panic!("one local recovery for {source:?}: {report:?}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            "color:red;".len()
        );
    }
}

#[test]
fn container_type_admits_exact_keyword_combinations_and_decoded_case() {
    accepted(
        "container-type",
        &[
            "normal",
            "size",
            "inline-size",
            "scroll-state",
            "size scroll-state",
            "scroll-state size",
            "inline-size scroll-state",
            "SCROLL-STATE INLINE-SIZE",
            r"s\69 ze",
            "size/**/scroll-state",
        ],
    );
}

#[test]
fn container_name_admits_ordered_identifiers_and_escaped_token_values() {
    accepted(
        "container-name",
        &[
            "none",
            "NoNe",
            "pane",
            "pane pane PANE",
            "normal size inline-size scroll-state",
            "--pane café",
            r"\31 pane",
            r"a\ b",
            r"a\,b",
            r"\-",
            r"n\6f ne",
            r"\0",
        ],
    );
}

#[test]
fn container_shorthand_requires_names_and_accepts_optional_type() {
    accepted(
        "container",
        &[
            "pane",
            "none",
            "size",
            "pane pane / size",
            "none / normal",
            "pane / scroll-state inline-size",
            "normal size / scroll-state",
            r"\31 pane / s\69 ze",
            "pane/**// /**/size",
        ],
    );
}

#[test]
fn container_type_rejects_duplicates_incompatible_keywords_and_other_tokens() {
    rejected(
        "container-type",
        &[
            "",
            " ",
            "size inline-size",
            "normal scroll-state",
            "size size",
            "scroll-state scroll-state",
            "style",
            "none",
            "size,scroll-state",
            "1",
            "10%",
            "\"size\"",
            "size inherit",
            "default",
            "var()",
        ],
    );
}

#[test]
fn container_name_rejects_reserved_decoded_identifiers_and_nonidentifiers() {
    rejected(
        "container-name",
        &[
            "",
            "none pane",
            "pane none",
            "and",
            "NOT",
            "or",
            "default",
            r"\61 nd",
            r"pane n\6f ne",
            "pane initial",
            "pane revert-layer",
            "pane,pane",
            "\"pane\"",
            "1pane",
            "ident(pane)",
            "var(foo)",
        ],
    );
}

#[test]
fn container_shorthand_rejects_missing_components_and_embedded_wide_values() {
    rejected(
        "container",
        &[
            "",
            "/ size",
            "pane /",
            "pane / size / normal",
            "pane / size inline-size",
            "pane / none",
            "none pane / size",
            "inherit / size",
            "pane / inherit",
            "pane initial",
            "default",
            "pane,other / size",
            "ident(pane) / size",
            "var()",
        ],
    );
}

fn assert_catalog(name: &str) -> CssPropertyGrammar {
    let value = grammar(name);
    assert_eq!(value.name(), name);
    assert_eq!(
        CssPropertyGrammar::from_name(&name.to_ascii_uppercase()),
        Some(value)
    );
    let support = property_support_metadata(name).expect("central property support metadata");
    assert_eq!(support.property(), value.target_property());
    assert_eq!(support.canonical_name(), name);
    assert_eq!(support.feature().id(), value.feature_id());
    assert_eq!(support.feature().source().url(), Some(CONDITIONAL5));
    assert_eq!(support.feature().status(), CssSupportStatus::Complete);
    value
}

fn assert_longhand_metadata(name: &str, initial: &str) {
    let value = assert_catalog(name);
    let metadata = value.metadata().expect("intrinsic metadata");
    assert_eq!(metadata.grammar(), value);
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("longhand")
    };
    assert_eq!(
        longhand.property().known_property(),
        value.target_property()
    );
    assert!(!longhand.inherited_by_default());
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial_value) = initial_value.view() else {
        panic!("fixed intrinsic initial")
    };
    assert_eq!(initial_value, &ordinary(name, initial));
    let CssPropertyKindRef::UniversalReset(all) = CssKnownProperty::All.metadata().unwrap().kind()
    else {
        panic!("all")
    };
    assert!(!all.excludes(CssPropertyNameRef::Known(value.target_property())));
}

#[test]
fn container_type_has_noninherited_normal_initial_and_selected_source() {
    assert_longhand_metadata("container-type", "normal");
}

#[test]
fn container_name_has_noninherited_none_initial_and_selected_source() {
    assert_longhand_metadata("container-name", "none");
}

#[test]
fn container_metadata_has_exactly_two_settable_members_and_no_reset_only_member() {
    let value = assert_catalog("container");
    let metadata = value.metadata().unwrap();
    assert_eq!(metadata.grammar(), value);
    let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
        panic!("shorthand")
    };
    let names = shorthand
        .members()
        .iter()
        .map(|p| p.known_property().canonical_name())
        .collect::<Vec<_>>();
    assert_eq!(names, ["container-name", "container-type"]);
    assert_eq!(shorthand.settable_members(), shorthand.members());
    assert!(shorthand.reset_only_members().is_empty());
    assert!(!shorthand.is_legacy());
}

#[test]
fn explicit_container_expansion_preserves_names_and_both_type_keywords() {
    for (value, names, kind) in [
        (
            "Pane Pane pane / scroll-state size",
            "Pane Pane pane",
            "size scroll-state",
        ),
        ("none / inline-size", "none", "inline-size"),
        (r"a\ b / scroll-state", r"a\ b", "scroll-state"),
    ] {
        assert_pair(&expanded(&declaration("container", value)), names, kind);
    }
}

#[test]
fn omitted_container_type_resets_to_normal_instead_of_preserving_previous_type() {
    for names in ["pane", "size", "none", "pane pane"] {
        assert_pair(&expanded(&declaration("container", names)), names, "normal");
    }
}

#[test]
fn container_name_values_preserve_order_duplicates_and_case() {
    let list = ordinary("container-name", "pane pane PANE");
    assert_ne!(list, ordinary("container-name", "pane PANE"));
    assert_ne!(list, ordinary("container-name", "PANE pane pane"));
    assert_ne!(
        ordinary("container-name", "pane"),
        ordinary("container-name", "PANE")
    );
    assert_eq!(
        ordinary("container-name", r"p\61 ne"),
        ordinary("container-name", "pane")
    );
    assert_eq!(
        ordinary("container-type", "scroll-state size"),
        ordinary("container-type", "size scroll-state")
    );
}

fn assert_wide(name: &str, members: &[&str]) {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
        (r"\69 nherit", CssGlobalKeyword::Inherit),
    ] {
        let source = declaration(name, text);
        assert_eq!(source.known().unwrap().global(), Some(keyword));
        assert!(source.known().unwrap().property_value().is_none());
        let values = expanded(&source);
        assert_eq!(values.items().len(), members.len());
        for name in members {
            assert_eq!(
                member(&values, name).value(),
                CssContributionValueRef::Global(keyword)
            );
        }
    }
}

#[test]
fn container_type_wide_values_remain_symbolic() {
    assert_wide("container-type", &["container-type"]);
}
#[test]
fn container_name_wide_values_remain_symbolic() {
    assert_wide("container-name", &["container-name"]);
}
#[test]
fn container_wide_values_propagate_to_both_members() {
    assert_wide("container", &["container-name", "container-type"]);
}

fn assert_pending(
    name: &str,
    authored: &str,
    replacement: &str,
    invalid: &str,
    members: &[(&str, &str)],
) {
    let source = declaration(name, authored);
    assert!(source.known().unwrap().substitution_dependent().is_some());
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    assert!(pending.source().same_occurrence(&source));
    let error = pending
        .reenter(parse_component_values("var(--still-pending)").unwrap())
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssExpansionErrorKind::ResidualSubstitution
    ));
    let error = pending
        .reenter(parse_component_values(invalid).unwrap())
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    let replacement = parse_component_values(replacement).unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("completed replacement")
    };
    assert_eq!(values.items().len(), members.len());
    for (name, expected) in members {
        let item = member(&values, name);
        assert_eq!(item.ordinary_value(), Some(&ordinary(name, expected)));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        let retained = item.replacement_components().unwrap();
        assert_eq!(retained, &replacement);
        for (actual, expected) in retained.items().iter().zip(replacement.items()) {
            assert_eq!(actual.origin(), expected.origin());
        }
    }
    // Retry a different valid terminal branch after both failures and success.
    let CssContributions::Longhands(wide) = pending
        .reenter(parse_component_values("revert-layer").unwrap())
        .unwrap()
    else {
        panic!("wide replacement")
    };
    assert_eq!(wide.items().len(), members.len());
    for (name, _) in members {
        assert_eq!(
            member(&wide, name).value(),
            CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
        );
    }
    assert!(pending.source().same_occurrence(&source));
}

#[test]
fn container_type_pending_reentry_checks_the_full_replacement_grammar() {
    assert_pending(
        "container-type",
        "nonsense var(--type)",
        "scroll-state size",
        "size inline-size",
        &[("container-type", "size scroll-state")],
    );
}
#[test]
fn container_name_pending_reentry_checks_reserved_names() {
    assert_pending(
        "container-name",
        "none var(--names)",
        "Pane Pane pane",
        "pane none",
        &[("container-name", "Pane Pane pane")],
    );
}
#[test]
fn container_pending_reentry_is_atomic_over_both_longhands() {
    assert_pending(
        "container",
        "var(--layout)",
        "Pane / inline-size scroll-state",
        "Pane / initial",
        &[
            ("container-name", "Pane"),
            ("container-type", "inline-size scroll-state"),
        ],
    );
}

#[test]
fn checked_name_construction_preserves_hex_escape_and_separate_identifier_boundaries() {
    let value = grammar("container-name");
    let mut tokens = parse_component_values(r"\31").unwrap().items().to_vec();
    tokens.extend_from_slice(parse_component_values(" pane").unwrap().items());
    let components = CssComponentValues::try_new(tokens).unwrap();
    let serialized = components.serialize().unwrap();
    let reparsed = parse_component_values(serialized.as_css()).unwrap();
    let names = reparsed
        .items()
        .iter()
        .filter_map(|token| match token.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => Some(name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["1", "pane"]);
    let constructed =
        parse_property_value_for_grammar(value, components.clone(), CssImportance::Important)
            .unwrap();
    assert!(constructed.position().is_none());
    assert!(constructed.parsed_value().is_none());
    assert_eq!(constructed.value_components(), &components);
    let values = expanded(&constructed);
    assert_eq!(
        member(&values, "container-name").ordinary_value(),
        Some(&ordinary("container-name", r"\31  pane"))
    );
    assert_ne!(
        member(&values, "container-name").ordinary_value(),
        Some(&ordinary("container-name", r"\31 pane"))
    );
}

#[test]
fn checked_type_construction_does_not_merge_adjacent_identifier_tokens() {
    let value = grammar("container-type");
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("size").unwrap(),
        CssComponentValue::try_ident("scroll-state").unwrap(),
    ])
    .unwrap();
    let constructed =
        parse_property_value_for_grammar(value, components, CssImportance::Important).unwrap();
    let values = expanded(&constructed);
    assert_eq!(
        member(&values, "container-type").ordinary_value(),
        Some(&ordinary("container-type", "size scroll-state"))
    );
}

fn assert_style_identity(name: &str, authored_name: &str, valid_value: &str) {
    // Style query plain values retain declaration-value syntax; recognition of
    // the property name must not start evaluating or applying its value grammar.
    for value in [valid_value, "banana"] {
        let source = format!("style({authored_name}: {value})");
        let condition =
            CssContainerCondition::try_from_components(parse_component_values(&source).unwrap())
                .unwrap();
        let CssContainerConditionKind::Style(query) = condition.kind() else {
            panic!("style query")
        };
        let CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Plain {
            name: feature_name,
            value: feature_value,
        }) = query.kind()
        else {
            panic!("plain feature")
        };
        let CssContainerStyleFeatureName::Property(actual) = feature_name else {
            panic!("recognized {name} grammar, not an unknown-name placeholder")
        };
        assert_eq!(*actual, grammar(name));
        assert_eq!(
            feature_value.serialize().unwrap().as_css(),
            format!(" {value}")
        );
    }
}

#[test]
fn container_type_style_feature_uses_shared_decoded_property_identity() {
    assert_style_identity("container-type", r"CONTAINER-\74 YPE", "size scroll-state");
}

#[test]
fn container_name_style_feature_uses_shared_property_identity() {
    assert_style_identity("container-name", "CONTAINER-NAME", "Pane Pane pane");
}

#[test]
fn container_shorthand_style_feature_uses_shared_property_identity() {
    assert_style_identity("container", "container", "Pane / inline-size");
}
