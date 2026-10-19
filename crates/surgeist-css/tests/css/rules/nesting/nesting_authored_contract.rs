#![forbid(unsafe_code)]
//! Authored nesting follows CSS Nesting WD20260122 sections 3.1 and 4, plus
//! the selected Syntax 3 block-contents definition at revision f971255463f01fb.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/
//! https://github.com/w3c/csswg-drafts/blob/f971255463f01fb740e2a3a7ecfe83e319cddab9/css-syntax-3/Overview.bs
//! Public normalization retains complete parent lists and symbolic anchor identity;
//! these characterizations exercise syntax and context without evaluating matching
//! or specificity.

use surgeist_css::{
    CssDeclaration, CssErrorCode, CssNamespaceContext, CssNormalizedItem, CssNormalizedSheet,
    CssPseudoClass, CssRecoveryAction, CssRule, CssRuleContextKindRef, CssSelector,
    CssSelectorBinding, CssSelectorCombinator, CssStyleSelector, CssTokenKind, ErrorKind,
    normalize_sheet, parse_selector, parse_sheet,
};

fn declaration_text(declaration: &CssDeclaration) -> (&str, &str) {
    let name = declaration
        .parsed_name()
        .expect("original parsed declaration name");
    let value = declaration
        .parsed_value()
        .expect("original parsed declaration value");
    let name_text = &name.source().as_str()
        [name.span().start().byte_offset().value()..name.span().end().byte_offset().value()];
    let value_text = &value.source().as_str()
        [value.span().start().byte_offset().value()..value.span().end().byte_offset().value()];
    (name_text, value_text.trim())
}

fn stream(sheet: &CssNormalizedSheet) -> Vec<String> {
    sheet
        .items()
        .iter()
        .map(|item| match item {
            CssNormalizedItem::Rule(rule) => match rule.kind() {
                CssRuleContextKindRef::Style(context) => format!(
                    "style:{}",
                    context
                        .selectors()
                        .iter()
                        .map(|entry| { entry.selector().to_specified_css().unwrap() })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                CssRuleContextKindRef::NestedDeclarations(_) => "run".to_owned(),
                other => panic!("unexpected rule in this specimen: {other:?}"),
            },
            CssNormalizedItem::Declaration(declaration) => {
                let (name, value) = declaration_text(declaration.source());
                format!("decl:{name}={value}")
            }
            other => panic!("unexpected normalized item: {other:?}"),
        })
        .collect()
}

#[test]
fn mixed_nested_members_share_the_complete_original_parent_context() {
    let source = ".Parent,#Alt { > &.Child, .Else &, .Plain { color:red } .Last { color:blue } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let original = report.clone();
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("one parent");
    };
    let [CssRule::Style(child), CssRule::Style(last)] = parent.rules() else {
        panic!("two ordered children");
    };
    assert!(parent.declarations().is_empty());
    assert!(matches!(parent.selectors().selectors(), [
        CssStyleSelector::Selector(CssSelector::Class(name)),
        CssStyleSelector::Selector(CssSelector::Key(id)),
    ] if name == "Parent" && id == "Alt"));
    assert_eq!(parent.position().byte_offset().value(), 0);
    assert_eq!(
        child.position().byte_offset().value(),
        source.find("> &.Child").unwrap()
    );
    assert_eq!(
        last.position().byte_offset().value(),
        source.find(".Last").unwrap()
    );
    let [
        CssStyleSelector::Relative(relative),
        CssStyleSelector::Selector(explicit),
        CssStyleSelector::Selector(plain),
    ] = child.selectors().selectors()
    else {
        panic!("three authored members, without parent-list multiplication");
    };
    assert_eq!(relative.combinator(), CssSelectorCombinator::Child);
    let CssSelector::Compound(compound) = relative.selector() else {
        panic!("anchored child compound");
    };
    assert_eq!(compound.nesting_selectors(), 1);
    assert_eq!(compound.classes(), &["Child".to_owned()]);
    assert_eq!(explicit.to_specified_css().unwrap(), ".Else &");
    assert_eq!(plain, &CssSelector::Class("Plain".to_owned()));
    assert_eq!(
        child
            .selectors()
            .selectors()
            .iter()
            .map(|entry| { entry.selector().to_specified_css().unwrap() })
            .collect::<Vec<_>>(),
        ["&.Child", ".Else &", ".Plain"]
    );
    assert_eq!(
        last.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".Last"
    );
    assert_eq!(declaration_text(&child.declarations()[0]), ("color", "red"));
    assert_eq!(declaration_text(&last.declarations()[0]), ("color", "blue"));

    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(
        stream(&normalized),
        [
            "style:.Parent, #Alt",
            "style:&.Child, .Else &, .Plain",
            "decl:color=red",
            "style:.Last",
            "decl:color=blue",
        ]
    );
    let CssNormalizedItem::Rule(parent_rule) = &normalized.items()[0] else {
        panic!("parent occurrence first");
    };
    let CssRuleContextKindRef::Style(parent_context) = parent_rule.kind() else {
        panic!("parent selector context");
    };
    let CssNormalizedItem::Declaration(red) = &normalized.items()[2] else {
        panic!("first child declaration");
    };
    let CssNormalizedItem::Declaration(blue) = &normalized.items()[4] else {
        panic!("last child declaration");
    };
    let context = red.selector_context();
    assert_eq!(context.selectors().len(), 3);
    assert_eq!(
        context
            .selectors()
            .iter()
            .map(|entry| entry.binding())
            .collect::<Vec<_>>(),
        [
            CssSelectorBinding::LeadingCombinator(CssSelectorCombinator::Child),
            CssSelectorBinding::ExplicitAnchors,
            CssSelectorBinding::ImplicitDescendant,
        ]
    );
    assert!(context.parent().unwrap().same_context(parent_context));
    assert!(
        blue.selector_context()
            .parent()
            .unwrap()
            .same_context(parent_context)
    );
    assert_eq!(parent_context.selectors().len(), 2);
    assert_eq!(
        parent_context.selectors()[0].selector(),
        &CssSelector::Class("Parent".to_owned())
    );
    assert_eq!(
        parent_context.selectors()[1].selector(),
        &CssSelector::Key("Alt".to_owned())
    );
    assert!(parent_context.parent().is_none());
    assert!(!context.same_context(blue.selector_context()));
    assert_eq!(
        blue.selector_context().selectors()[0].binding(),
        CssSelectorBinding::ImplicitDescendant
    );
    assert!(
        red.rule_context()
            .parent()
            .unwrap()
            .same_context(parent_rule)
    );
    assert!(
        blue.rule_context()
            .parent()
            .unwrap()
            .same_context(parent_rule)
    );
    assert!(red.source().same_occurrence(&child.declarations()[0]));
    assert!(blue.source().same_occurrence(&last.declarations()[0]));
    assert_eq!((red.order(), blue.order()), (0, 1));
    for declaration in [red, blue] {
        assert_eq!(
            declaration
                .source()
                .parsed_name()
                .unwrap()
                .source()
                .as_str(),
            source
        );
    }
    assert_eq!(report, original);
    assert!(report.into_validation_result().is_ok());
}

#[test]
fn invalid_ampersand_child_drops_its_declarations_and_grandchild_between_siblings() {
    let source = ".P{color:red;.Before{color:blue}&div{color:green;.Leak{width:1px}}.After{height:2px}opacity:1}.Outside{}";
    let failed = "&div{color:green;.Leak{width:1px}}";
    let report = parse_sheet(source);
    let original = report.clone();
    let [CssRule::Style(parent), CssRule::Style(outside)] = report.syntax().rules() else {
        panic!("parent and outside survive");
    };
    let [
        CssRule::Style(before),
        CssRule::Style(after),
        CssRule::NestedDeclarations(run),
    ] = parent.rules()
    else {
        panic!("valid siblings and trailing run only");
    };
    assert_eq!(
        parent.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".P"
    );
    assert_eq!(
        before.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".Before"
    );
    assert_eq!(
        after.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".After"
    );
    assert_eq!(
        outside.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".Outside"
    );
    assert_eq!(
        declaration_text(&parent.declarations()[0]),
        ("color", "red")
    );
    assert_eq!(parent.declarations().len(), 1);
    assert_eq!(before.declarations().len(), 1);
    assert_eq!(after.declarations().len(), 1);
    assert_eq!(run.declarations().len(), 1);
    assert!(before.rules().is_empty() && after.rules().is_empty() && outside.rules().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("one whole-child diagnostic");
    };
    let start = source.find(failed).unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        start + 1
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + failed.len()
    );
    assert_eq!(
        &source[diagnostic.span().start().byte_offset().value()
            ..diagnostic.span().end().byte_offset().value()],
        failed
    );
    assert_eq!(diagnostic.span().start().line().value(), 0);
    assert_eq!(diagnostic.span().start().column().value() as usize, start);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(
        stream(&normalized),
        [
            "style:.P",
            "decl:color=red",
            "style:.Before",
            "decl:color=blue",
            "style:.After",
            "decl:height=2px",
            "run",
            "decl:opacity=1",
            "style:.Outside",
        ]
    );
    assert_eq!(report, original);
    assert!(report.into_validation_result().is_err());
}

#[test]
fn identifier_led_invalid_child_is_one_recovery_unit_without_descendant_promotion() {
    let source =
        ".P{color:red;div&extra{color:green;.Leak{width:1px}}color:blue;.After{height:2px}}";
    let failed = "div&extra{color:green;.Leak{width:1px}}";
    let report = parse_sheet(source);
    let original = report.clone();
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent retained");
    };
    let [CssRule::NestedDeclarations(blue), CssRule::Style(after)] = parent.rules() else {
        panic!("later declaration run then valid child");
    };
    assert_eq!(parent.declarations().len(), 1);
    assert_eq!(
        declaration_text(&parent.declarations()[0]),
        ("color", "red")
    );
    assert_eq!(blue.declarations().len(), 1);
    assert_eq!(declaration_text(&blue.declarations()[0]), ("color", "blue"));
    assert_eq!(
        after.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".After"
    );
    assert!(after.rules().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("one complete failed child");
    };
    let start = source.find(failed).unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + failed.len()
    );
    assert_eq!(
        &source[diagnostic.span().start().byte_offset().value()
            ..diagnostic.span().end().byte_offset().value()],
        failed
    );
    assert_eq!(diagnostic.span().start().line().value(), 0);
    assert_eq!(diagnostic.span().start().column().value() as usize, start);
    // cssparser 0.37's declaration fallback retains expect_colon's original
    // error after the qualified-rule attempt fails. Unlike the &-led case,
    // the encountered token is the ampersand following the first identifier.
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        start + 3
    );
    let ErrorKind::UnexpectedToken(detail) = diagnostic.error().kind() else {
        panic!("original declaration fallback error");
    };
    assert_eq!(detail.encountered().kind(), CssTokenKind::Delim);
    assert_eq!(detail.encountered().authored(), "&");
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(
        stream(&normalized),
        [
            "style:.P",
            "decl:color=red",
            "run",
            "decl:color=blue",
            "style:.After",
            "decl:height=2px",
        ]
    );
    assert_eq!(report, original);
    assert!(report.into_validation_result().is_err());
}

#[test]
fn legal_type_first_and_noninitial_repeated_anchors_have_stable_specified_output() {
    let namespaces = CssNamespaceContext::default();
    for (source, expected, type_name, anchors) in [
        ("div&&.Card", "div&&.Card", Some("div"), 2),
        (".Card&&", "&&.Card", None, 2),
        ("div&", "div&", Some("div"), 1),
        // CSS Syntax hexadecimal escape 44 decodes to D; it remains a type name.
        (r"\44 iv&&.Card", "Div&&.Card", Some("Div"), 2),
    ] {
        let report = parse_selector(source, &namespaces);
        assert!(report.is_clean(), "{source}: {report:?}");
        let original = report.clone();
        let Some(CssSelector::Compound(compound)) = report.syntax() else {
            panic!("one legal compound");
        };
        assert_eq!(
            compound.type_selector().and_then(|name| name.local_name()),
            type_name
        );
        assert_eq!(compound.nesting_selectors(), anchors);
        assert!(!compound.has_scope_anchor());
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .to_specified_css()
                .unwrap(),
            expected
        );
        assert_eq!(report, original);
        assert!(report.into_validation_result().is_ok());
    }
    let report = parse_selector(".Lead > .Card&&", &namespaces);
    assert!(report.is_clean(), "{report:?}");
    let Some(CssSelector::Complex(complex)) = report.syntax() else {
        panic!("away-from-start anchors");
    };
    assert_eq!(complex.first().nesting_selectors(), 0);
    assert_eq!(complex.first().classes(), &["Lead".to_owned()]);
    let [part] = complex.rest() else {
        panic!("one child relation");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Child);
    assert_eq!(part.selector().nesting_selectors(), 2);
    assert_eq!(part.selector().classes(), &["Card".to_owned()]);
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        ".Lead > &&.Card"
    );
    for source in ["&div", "&-suffix"] {
        let report = parse_selector(source, &namespaces);
        assert!(report.syntax().is_none(), "{source}");
        assert!(!report.is_clean());
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn parentless_nesting_identity_is_distinct_from_literal_scope_identity() {
    let report = parse_sheet("& {color:red} :scope {color:blue}");
    assert!(report.is_clean(), "{report:?}");
    let original = report.clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(
        stream(&normalized),
        [
            "style:&",
            "decl:color=red",
            "style::scope",
            "decl:color=blue"
        ]
    );
    let CssNormalizedItem::Declaration(anchor) = &normalized.items()[1] else {
        panic!("nesting declaration");
    };
    let CssNormalizedItem::Declaration(scope) = &normalized.items()[3] else {
        panic!("scope declaration");
    };
    let anchor_context = anchor.selector_context();
    let scope_context = scope.selector_context();
    assert!(anchor_context.parent().is_none() && scope_context.parent().is_none());
    assert!(anchor_context.scope_context().is_none() && scope_context.scope_context().is_none());
    assert_eq!(
        anchor_context.selectors()[0].binding(),
        CssSelectorBinding::ExplicitAnchors
    );
    assert_eq!(
        scope_context.selectors()[0].binding(),
        CssSelectorBinding::Absolute
    );
    let CssSelector::Compound(compound) = anchor_context.selectors()[0].selector() else {
        panic!("symbolic nesting count");
    };
    assert_eq!(compound.nesting_selectors(), 1);
    assert_eq!(compound.scope_anchors(), 0);
    assert!(compound.pseudo_classes().is_empty());
    assert!(matches!(
        scope_context.selectors()[0].selector(),
        CssSelector::PseudoClass(CssPseudoClass::Scope)
    ));
    assert!(!anchor_context.same_context(scope_context));
    assert_eq!(
        anchor_context.selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        "&"
    );
    assert_eq!(
        scope_context.selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ":scope"
    );
    assert_eq!(report, original);
    assert!(report.into_validation_result().is_ok());
}

#[test]
fn nested_lists_keep_parent_identity_and_original_sources_across_deep_boundaries() {
    // Total open-block depths surround the structural chunk boundary (64)
    // and bounded parser-thread selection boundary (128). Every depth must
    // satisfy the same public nesting contract, independent of execution path.
    for depth in [63, 64, 65, 127, 128, 129] {
        let ancestors = depth - 2;
        let mut source = String::new();
        for level in 0..ancestors {
            source.push_str(&format!(".n{level:03}{{"));
        }
        source.push_str(
            ".Parent,#Alt{> &.Child,.Else &,.Plain{color:red}.Last{color:blue}opacity:1}",
        );
        source.push_str(&"}".repeat(ancestors));
        source.push_str(".Outside{}");

        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let original = report.clone();
        let [CssRule::Style(root), CssRule::Style(outside)] = report.syntax().rules() else {
            panic!("nested chain and unrelated sibling at depth {depth}");
        };
        let mut parent = root;
        for level in 0..ancestors {
            let name = format!("n{level:03}");
            assert_eq!(parent.selectors().selectors().len(), 1);
            assert_eq!(
                parent.selectors().selectors()[0].selector(),
                &CssSelector::Class(name.clone())
            );
            assert_eq!(
                parent.position().byte_offset().value(),
                source.find(&format!(".{name}{{")).unwrap()
            );
            assert!(parent.declarations().is_empty());
            let [CssRule::Style(child)] = parent.rules() else {
                panic!("one retained ancestor child at level {level}, depth {depth}");
            };
            parent = child;
        }
        assert_eq!(
            parent.position().byte_offset().value(),
            source.find(".Parent").unwrap()
        );
        assert!(matches!(parent.selectors().selectors(), [
            CssStyleSelector::Selector(CssSelector::Class(name)),
            CssStyleSelector::Selector(CssSelector::Key(id)),
        ] if name == "Parent" && id == "Alt"));
        assert!(parent.declarations().is_empty());
        let [
            CssRule::Style(child),
            CssRule::Style(last),
            CssRule::NestedDeclarations(run),
        ] = parent.rules()
        else {
            panic!("two ordered children and a trailing declaration run at depth {depth}");
        };
        assert_eq!(
            child.position().byte_offset().value(),
            source.find("> &.Child").unwrap()
        );
        assert_eq!(
            last.position().byte_offset().value(),
            source.find(".Last").unwrap()
        );
        assert_eq!(
            run.position().byte_offset().value(),
            source.find("opacity:1").unwrap()
        );
        assert_eq!(
            outside.position().byte_offset().value(),
            source.find(".Outside").unwrap()
        );
        assert!(outside.declarations().is_empty() && outside.rules().is_empty());
        let [red_source] = child.declarations().as_slice() else {
            panic!("one red declaration");
        };
        let [blue_source] = last.declarations().as_slice() else {
            panic!("one blue declaration");
        };
        let [opacity_source] = run.declarations().as_slice() else {
            panic!("one opacity declaration");
        };
        assert_eq!(declaration_text(red_source), ("color", "red"));
        assert_eq!(declaration_text(blue_source), ("color", "blue"));
        assert_eq!(declaration_text(opacity_source), ("opacity", "1"));

        let normalized = normalize_sheet(report.syntax()).unwrap();
        let mut expected = (0..ancestors)
            .map(|level| format!("style:.n{level:03}"))
            .collect::<Vec<_>>();
        expected.extend(
            [
                "style:.Parent, #Alt",
                "style:&.Child, .Else &, .Plain",
                "decl:color=red",
                "style:.Last",
                "decl:color=blue",
                "run",
                "decl:opacity=1",
                "style:.Outside",
            ]
            .map(str::to_owned),
        );
        assert_eq!(
            stream(&normalized),
            expected,
            "complete order at depth {depth}"
        );
        let CssNormalizedItem::Rule(parent_rule) = &normalized.items()[ancestors] else {
            panic!("the original two-member parent occurrence");
        };
        let CssRuleContextKindRef::Style(parent_context) = parent_rule.kind() else {
            panic!("the original parent selector context");
        };
        assert_eq!(parent_context.selectors().len(), 2);
        let CssNormalizedItem::Declaration(red) = &normalized.items()[ancestors + 2] else {
            panic!("red child value");
        };
        let CssNormalizedItem::Declaration(blue) = &normalized.items()[ancestors + 4] else {
            panic!("blue child value");
        };
        let CssNormalizedItem::Declaration(opacity) = &normalized.items()[ancestors + 6] else {
            panic!("trailing parent value");
        };
        assert_eq!(
            red.selector_context()
                .selectors()
                .iter()
                .map(|entry| entry.binding())
                .collect::<Vec<_>>(),
            [
                CssSelectorBinding::LeadingCombinator(CssSelectorCombinator::Child),
                CssSelectorBinding::ExplicitAnchors,
                CssSelectorBinding::ImplicitDescendant,
            ]
        );
        assert!(
            red.selector_context()
                .parent()
                .unwrap()
                .same_context(parent_context)
        );
        assert!(
            blue.selector_context()
                .parent()
                .unwrap()
                .same_context(parent_context)
        );
        assert!(opacity.selector_context().same_context(parent_context));
        assert!(
            red.rule_context()
                .parent()
                .unwrap()
                .same_context(parent_rule)
        );
        assert!(
            blue.rule_context()
                .parent()
                .unwrap()
                .same_context(parent_rule)
        );
        assert!(
            opacity
                .rule_context()
                .parent()
                .unwrap()
                .same_context(parent_rule)
        );
        assert_eq!((red.order(), blue.order(), opacity.order()), (0, 1, 2));
        for (value, original_declaration, authored) in [
            (red, red_source, "color:red"),
            (blue, blue_source, "color:blue"),
            (opacity, opacity_source, "opacity:1"),
        ] {
            assert!(value.source().same_occurrence(original_declaration));
            let origin = value.source().parsed_name().unwrap();
            assert_eq!(origin.source().as_str(), source);
            assert_eq!(
                origin.span().start().byte_offset().value(),
                source.find(authored).unwrap()
            );
        }
        for index in 0..=ancestors {
            let CssNormalizedItem::Rule(rule) = &normalized.items()[index] else {
                panic!("ancestor occurrence");
            };
            if index == 0 {
                assert!(rule.parent().is_none());
            } else {
                let CssNormalizedItem::Rule(expected_parent) = &normalized.items()[index - 1]
                else {
                    panic!("preceding ancestor occurrence");
                };
                assert!(rule.parent().unwrap().same_context(expected_parent));
            }
        }
        assert_eq!(report, original);
        assert!(report.into_validation_result().is_ok());
    }
}
