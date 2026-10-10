#![forbid(unsafe_code)]

//! Authored size controls from Sizing 4 WD (2026-09-04) §§3.1, 5.3, 5.5.
//! Used sizing and CSSOM shorthand ordering are downstream of these declarations.

use surgeist_css::*;

const SHORTHANDS: [&str; 3] = ["size", "min-size", "max-size"];
const TERMINALS: [&str; 2] = ["frame-sizing", "min-intrinsic-sizing"];

fn names() -> impl Iterator<Item = &'static str> {
    SHORTHANDS.into_iter().chain(TERMINALS)
}

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("selected property: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

fn accepted(name: &str, value: &str) {
    for source in [declaration(name, value), checked(name, value)] {
        assert_eq!(
            source.known().unwrap().property(),
            grammar(name).target_property()
        );
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            value
        );
    }
}

fn invalid(name: &str, value: &str) {
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid-value diagnostic: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
    assert!(validate_style_attribute(&source).is_err());
    assert!(
        parse_property_value_for_grammar(
            grammar(name),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err(),
        "checked construction accepted {name}:{value}"
    );
}

fn one_value(source: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one terminal contribution")
    };
    let [item] = values.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    item.ordinary_value().unwrap().clone()
}

fn members(name: &str) -> [&'static str; 2] {
    match name {
        "size" => ["width", "height"],
        "min-size" => ["min-width", "min-height"],
        "max-size" => ["max-width", "max-height"],
        _ => panic!("selected sizing shorthand: {name}"),
    }
}

#[test]
fn five_distinct_properties_have_complete_dated_sizing4_support() {
    let mut identities = Vec::new();
    for name in names() {
        let grammar = grammar(name);
        let property = grammar.target_property();
        assert_eq!(grammar.name(), name);
        assert_eq!(property.canonical_name(), name);
        assert_eq!(
            grammar.feature_id().as_str(),
            format!("ext.property.{name}")
        );
        assert!(
            !identities.contains(&property),
            "{name} aliases another property"
        );
        identities.push(property);
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(
            support.feature().source().id().as_str(),
            "X-SIZING4-20260904"
        );
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
    }
    assert_eq!(identities.len(), 5);
}

#[test]
fn sizing_shorthands_follow_their_member_domain_and_exact_one_or_two_values() {
    for name in ["size", "min-size"] {
        accepted(name, "auto");
        accepted(name, "auto 1px");
        invalid(name, "none");
    }
    accepted("max-size", "none");
    accepted("max-size", "none 1px");
    invalid("max-size", "auto");
    invalid("max-size", "1px auto");
    invalid("max-size", "calc-size(auto, size)");

    for name in SHORTHANDS {
        for value in [
            "0",
            "-0px",
            "1px",
            "1%",
            "1e999px",
            "1e-999%",
            "1px 2%",
            "min-content",
            "max-content",
            "fit-content",
            "stretch",
            "contain",
            "fit-content(1px)",
            "calc(1px - 2px)",
            "calc-size(min-content, size + 1px)",
            "fit-content(1px) calc-size(min-content, size + 1px)",
        ] {
            accepted(name, value);
        }
        for value in [
            "",
            "-1px",
            "-1e-999%",
            "1px -1px",
            "1",
            "1fr",
            "1px 2px 3px",
            "logical 1px",
            "fit-content(-1px)",
            "calc(1 + 2)",
            "initial 1px",
        ] {
            invalid(name, value);
        }
    }
}

#[test]
fn three_sizing_shorthands_set_only_physical_width_then_height_and_repeat_first() {
    for name in SHORTHANDS {
        let members = members(name);
        let CssPropertyKindRef::Shorthand(shorthand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a shorthand")
        };
        assert!(!shorthand.is_legacy());
        assert!(shorthand.reset_only_members().is_empty());
        assert_eq!(
            shorthand
                .settable_members()
                .iter()
                .map(|member| member.known_property().canonical_name())
                .collect::<Vec<_>>(),
            members
        );
        let first = if name == "max-size" { "none" } else { "auto" };
        for (authored, width, height) in [
            ("1px", "1px", "1px"),
            ("1px 2%", "1px", "2%"),
            ("fit-content(1px)", "fit-content(1px)", "fit-content(1px)"),
            (
                "calc-size(min-content, size + 1px) 2px",
                "calc-size(min-content, size + 1px)",
                "2px",
            ),
            (first, first, first),
        ] {
            let source = declaration(name, authored);
            checked(name, authored);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{name} expands to two physical longhands")
            };
            let [width_item, height_item] = values.items() else {
                panic!("exactly width then height")
            };
            for (item, member, expected) in [
                (width_item, members[0], width),
                (height_item, members[1], height),
            ] {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(
                    item.ordinary_value(),
                    Some(&one_value(&declaration(member, expected)))
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert!(item.replacement_components().is_none());
            }
        }
    }
}

#[test]
fn frame_and_min_intrinsic_terminals_have_exact_keyword_sets_and_initials() {
    for value in [
        "auto",
        "content-width",
        "content-height",
        "content-block-size",
        "content-inline-size",
        "CONTENT-INLINE-SIZE",
    ] {
        accepted("frame-sizing", value);
    }
    for value in [
        "none",
        "content-size",
        "content-width content-height",
        "1px",
        "initial auto",
    ] {
        invalid("frame-sizing", value);
    }
    for value in [
        "legacy",
        "zero-if-scroll",
        "zero-if-extrinsic",
        "zero-if-scroll zero-if-extrinsic",
        "zero-if-extrinsic zero-if-scroll",
    ] {
        accepted("min-intrinsic-sizing", value);
    }
    for value in [
        "",
        "auto",
        "zero-if-scroll zero-if-scroll",
        "zero-if-extrinsic zero-if-extrinsic",
        "legacy zero-if-scroll",
        "zero-if-extrinsic legacy",
        "zero-if-scroll zero-if-extrinsic legacy",
        "initial legacy",
    ] {
        invalid("min-intrinsic-sizing", value);
    }
    for (name, initial_text) in [("frame-sizing", "auto"), ("min-intrinsic-sizing", "legacy")] {
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a terminal")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} has a fixed initial")
        };
        assert_eq!(value, &one_value(&declaration(name, initial_text)));
    }
}

#[test]
fn css_wide_all_and_pending_reentry_keep_member_order_and_source() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("symbolic all reset")
    };
    for name in names() {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            checked(name, text);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{name} has symbolic global contributions")
            };
            let expected_names: Vec<&str> = if SHORTHANDS.contains(&name) {
                members(name).into_iter().collect()
            } else {
                vec![name]
            };
            assert_eq!(values.items().len(), expected_names.len());
            for (item, member) in values.items().iter().zip(expected_names) {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
        for pending in ["var(--size)", "env(--size)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} retains pending substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            let invalid_value = if name == "min-intrinsic-sizing" {
                "auto"
            } else {
                "1fr"
            };
            assert!(matches!(
                handle
                    .reenter(parse_component_values(invalid_value).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement_text = match name {
                "size" | "min-size" => "auto 2px",
                "max-size" => "none 2px",
                "frame-sizing" => "content-inline-size",
                "min-intrinsic-sizing" => "zero-if-scroll zero-if-extrinsic",
                _ => unreachable!(),
            };
            let expected_names: Vec<&str> = if SHORTHANDS.contains(&name) {
                members(name).into_iter().collect()
            } else {
                vec![name]
            };
            let expected_values: Vec<&str> = if SHORTHANDS.contains(&name) {
                replacement_text.split_whitespace().collect()
            } else {
                vec![replacement_text]
            };
            let replacement = parse_component_values(replacement_text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to terminal contributions")
            };
            assert_eq!(values.items().len(), expected_names.len());
            for ((item, member), expected_value) in values
                .items()
                .iter()
                .zip(expected_names)
                .zip(expected_values)
            {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(
                    item.ordinary_value(),
                    Some(&one_value(&declaration(member, expected_value)))
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
            let CssContributions::Longhands(globals) = handle
                .reenter(parse_component_values("inherit").unwrap())
                .unwrap()
            else {
                panic!("{name} reenters to global contributions")
            };
            assert_eq!(
                globals.items().len(),
                if SHORTHANDS.contains(&name) { 2 } else { 1 }
            );
            for item in globals.items() {
                assert_eq!(
                    item.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
                assert!(item.source().same_occurrence(&source));
            }
        }
    }
}

#[test]
fn ordinary_size_is_not_the_page_size_descriptor_and_normalization_keeps_order() {
    accepted("size", "10px 20px");
    let report = parse_sheet("@page{margin-top:1px;size:10px 20px;margin-bottom:2px}");
    assert!(report.is_clean());
    let [CssRule::Page(page)] = report.syntax().rules() else {
        panic!("one retained page rule")
    };
    assert!(matches!(
        page.declarations()
            .effective_descriptor(CssPageDescriptorKind::Size)
            .unwrap()
            .value()
            .view(),
        CssPageDescriptorValueRef::Size(CssPageSizeValue::Dimensions(_, Some(_)))
    ));
    assert_eq!(page.declarations().properties().len(), 2);
    assert_eq!(
        page.declarations().properties()[0]
            .known()
            .unwrap()
            .property(),
        grammar("margin-top").target_property()
    );
    assert_eq!(
        page.declarations().properties()[1]
            .known()
            .unwrap()
            .property(),
        grammar("margin-bottom").target_property()
    );

    let report = parse_sheet(
        ".a{size:1px 2px;frame-sizing:content-block-size;max-size:auto;min-size:fit-content(3px);min-intrinsic-sizing:zero-if-extrinsic zero-if-scroll;max-size:none}",
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one invalid max-size diagnostic: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 5);
    for (index, name) in [
        "size",
        "frame-sizing",
        "min-size",
        "min-intrinsic-sizing",
        "max-size",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        assert_eq!(
            declarations[index].source().known().unwrap().property(),
            grammar(name).target_property()
        );
        assert!(matches!(
            declarations[index].expansion(),
            CssExpansion::Contributions(CssContributions::Longhands(_))
        ));
    }
}
