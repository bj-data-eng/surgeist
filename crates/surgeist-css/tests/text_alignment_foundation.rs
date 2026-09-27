#![forbid(unsafe_code)]
//! Text 3 §§6.1–6.3 and Text 4 §§1.4, 7.1–7.4 (2026-08-14), with the
//! selected one-extended-grapheme-cluster reconciliation of UAX29 revision 47.
//! Exact current-value payloads require the later functional typed-API tests.

use surgeist_css::*;

const SHORTHAND: &str = "text-align";
const ALL: &str = "text-align-all";
const LAST: &str = "text-align-last";

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("selected property: {name}"))
}

fn parsed(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).expect("valid CSS component syntax"),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

fn accepted(name: &str, value: &str) {
    for declaration in [parsed(name, value), checked(name, value)] {
        assert_eq!(declaration.known().unwrap().grammar(), grammar(name));
        assert_eq!(declaration.importance(), CssImportance::Important);
        assert!(!declaration.value_components().items().is_empty());
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
    if let Ok(components) = parse_component_values(value) {
        assert!(
            parse_property_value_for_grammar(grammar(name), components, CssImportance::Normal)
                .is_err(),
            "checked construction accepted {name}:{value}"
        );
    }
}

fn contributions(source: &CssDeclaration, members: &[&str]) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("selected intrinsic alignment expansion")
    else {
        panic!("terminal alignment expansion")
    };
    assert_eq!(values.items().len(), members.len());
    for (item, member) in values.items().iter().zip(members) {
        assert_eq!(item.property(), grammar(member).target_property());
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
    }
    values.items().to_vec()
}

#[test]
fn three_alignment_names_have_text4_provenance_and_two_inherited_initials() {
    for name in [SHORTHAND, ALL, LAST] {
        let grammar = grammar(name);
        assert_eq!(grammar.name(), name);
        assert_eq!(grammar.target_property().canonical_name(), name);
        let expected_id = if name == ALL {
            "official.property.text-align-all".to_owned()
        } else {
            format!("baseline.property.{name}")
        };
        assert_eq!(grammar.feature_id().as_str(), expected_id);
        let support = property_support_metadata(name).expect("selected metadata");
        assert_eq!(support.property(), grammar.target_property());
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "X-TEXT4");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2026/WD-css-text-4-20260814/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
    }

    let CssPropertyKindRef::Shorthand(shorthand) = grammar(SHORTHAND).metadata().unwrap().kind()
    else {
        panic!("text-align is the two-member shorthand")
    };
    assert!(!shorthand.is_legacy());
    assert!(shorthand.reset_only_members().is_empty());
    assert_eq!(
        shorthand
            .settable_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>(),
        [ALL, LAST]
    );
    for (name, initial) in [(ALL, "start"), (LAST, "auto")] {
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a longhand")
        };
        assert!(longhand.inherited_by_default());
        let initial_value = longhand.initial_value();
        let CssInitialValueRef::Value(value) = initial_value.view() else {
            panic!("{name} has a fixed initial")
        };
        let assigned = contributions(&parsed(name, initial), &[name]);
        let [assigned] = assigned.as_slice() else {
            panic!("one longhand")
        };
        assert_eq!(value, assigned.ordinary_value().unwrap());
    }
}

#[test]
fn alignment_keywords_accept_exact_property_domains_and_expand_in_member_order() {
    let positional = ["start", "end", "left", "right", "center"];
    for name in [SHORTHAND, ALL, LAST] {
        for keyword in positional.into_iter().chain(["justify", "match-parent"]) {
            accepted(name, keyword);
            let members: &[&str] = if name == SHORTHAND {
                &[ALL, LAST]
            } else if name == ALL {
                &[ALL]
            } else {
                &[LAST]
            };
            contributions(&parsed(name, keyword), members);
        }
    }
    accepted(SHORTHAND, "justify-all");
    contributions(&parsed(SHORTHAND, "justify-all"), &[ALL, LAST]);
    accepted(LAST, "auto");
    contributions(&parsed(LAST, "auto"), &[LAST]);
    for (name, bad) in [
        (SHORTHAND, "auto"),
        (ALL, "auto"),
        (ALL, "justify-all"),
        (LAST, "justify-all"),
        (LAST, "\".\""),
    ] {
        invalid(name, bad);
    }

    // These four shorthand forms have different specified effects. Exact typed
    // payload checks follow with the new public value API after the RED boundary.
    for keyword in ["center", "\".\"", "match-parent", "justify-all"] {
        contributions(&parsed(SHORTHAND, keyword), &[ALL, LAST]);
    }
}

#[test]
fn character_alignment_accepts_one_decoded_cluster_and_both_pair_orders() {
    for name in [SHORTHAND, ALL] {
        for value in [
            "\".\"",
            "\".\" left",
            "left \".\"",
            "\".\"center",
            "center\".\"",
            "\".\"/**/center",
            "center/**/\".\"",
            "\"é\"",
            "\"한\"",
            "\"👩\u{200d}💻\"",
            "\"🇺🇸\"",
            "\"\\65\\301 \"",
            "\".\\\n\"",
        ] {
            accepted(name, value);
        }
        let members: &[&str] = if name == SHORTHAND {
            &[ALL, LAST]
        } else {
            &[ALL]
        };
        contributions(&parsed(name, "\".\" right"), members);
        contributions(&parsed(name, "right \".\""), members);
        for value in [
            "\"\"",
            "\"ab\"",
            "\"🇺🇸🇨\"",
            "\".\" left right",
            "left right \".\"",
            "\".\" \"x\"",
            "\".\" justify",
            "justify \".\"",
            "\".\" match-parent",
            "\".\" justify-all",
            "start end",
        ] {
            invalid(name, value);
        }
    }
    invalid(SHORTHAND, "\"a\\\nb\"");
    invalid(ALL, "\"a\\\nb\"");
    for name in [SHORTHAND, ALL] {
        let source = format!("color:red;{name}:\"unterminated;color:blue");
        let report = parse_style_attribute(&source);
        assert!(!report.is_clean(), "bad string must be diagnosed");
        assert!(validate_style_attribute(&source).is_err());
    }
}

#[test]
fn css_wide_keywords_and_universal_all_remain_symbolic() {
    for (spelling, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for name in [SHORTHAND, ALL, LAST] {
            let source = parsed(name, spelling);
            let members: &[&str] = if name == SHORTHAND {
                &[ALL, LAST]
            } else if name == ALL {
                &[ALL]
            } else {
                &[LAST]
            };
            let values = contributions(&source, members);
            assert!(
                values
                    .iter()
                    .all(|item| item.value() == CssContributionValueRef::Global(keyword))
            );
        }
    }
    for name in [SHORTHAND, ALL, LAST] {
        invalid(name, "inherit center");
        invalid(name, "\".\" unset");
    }
    let source = parsed("all", "revert-layer");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("universal reset")
    };
    for name in [ALL, LAST] {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
    }
}

#[test]
fn pending_alignment_reentry_is_strict_retryable_and_keeps_original_occurrence() {
    for (name, valid, members) in [
        (SHORTHAND, "justify-all", &[ALL, LAST][..]),
        (ALL, "\".\" left", &[ALL][..]),
        (LAST, "match-parent", &[LAST][..]),
    ] {
        for pending in ["var(--align)", "env(--align)"] {
            let source = parsed(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} remains pending")
            };
            assert!(handle.source().same_occurrence(&source));
            for invalid_value in ["start end", "inherit center"] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(invalid_value).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            for residual in ["var(--again)", "env(--again)"] {
                assert_eq!(
                    handle
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
            }
            let replacement = parse_component_values(valid).unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("terminal replacement")
                };
                assert_eq!(values.items().len(), members.len());
                for (item, member) in values.items().iter().zip(members) {
                    assert_eq!(item.property(), grammar(member).target_property());
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
            let CssContributions::Longhands(values) = handle
                .reenter(parse_component_values("unset").unwrap())
                .unwrap()
            else {
                panic!("global replacement")
            };
            assert_eq!(values.items().len(), members.len());
            assert!(values.items().iter().all(
                |item| item.value() == CssContributionValueRef::Global(CssGlobalKeyword::Unset)
            ));
        }
    }
}

#[test]
fn normalization_preserves_alignment_order_context_and_atomic_two_member_limit() {
    let text = ".a{text-align:center;text-align-all:\".\" left;text-align-last:match-parent;text-align:justify-all!important}";
    let report = parse_sheet(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 4);
    let authored_positions = [
        text.find("text-align:center").unwrap(),
        text.find("text-align-all:").unwrap(),
        text.find("text-align-last:").unwrap(),
        text.find("text-align:justify-all").unwrap(),
    ];
    for (index, (item, (name, members))) in declarations
        .iter()
        .zip([
            (SHORTHAND, &[ALL, LAST][..]),
            (ALL, &[ALL][..]),
            (LAST, &[LAST][..]),
            (SHORTHAND, &[ALL, LAST][..]),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().grammar().name(), name);
        assert_eq!(
            item.selector_context().selectors()[0].selector(),
            &CssSelector::Class("a".into())
        );
        if index > 0 {
            assert!(
                item.selector_context()
                    .same_context(declarations[0].selector_context())
            );
        }
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            authored_positions[index]
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("terminal normalized alignment")
        };
        assert_eq!(values.items().len(), members.len());
        for (value, member) in values.items().iter().zip(members) {
            assert_eq!(value.property(), grammar(member).target_property());
            assert!(value.source().same_occurrence(item.source()));
        }
    }
    assert_eq!(
        declarations[3].source().importance(),
        CssImportance::Important
    );
    let limit = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
    let one = parse_sheet(".a{text-align:justify-all}");
    assert!(one.is_clean());
    let before = one.syntax().clone();
    let error = normalize_sheet_with_limits(one.syntax(), limit).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 1
        }
    );
    assert_eq!(error.declaration_order(), Some(0));
    assert_eq!(error.position().unwrap().byte_offset().value(), 3);
    let failed = error
        .declaration()
        .expect("responsible shorthand occurrence");
    assert_eq!(failed.known().unwrap().grammar().name(), SHORTHAND);
    assert_eq!(failed.position(), error.position());
    let context = error.rule_context().expect("owning .a style rule");
    assert_eq!(context.position().unwrap().byte_offset().value(), 0);
    assert_eq!(one.syntax(), &before);
}
