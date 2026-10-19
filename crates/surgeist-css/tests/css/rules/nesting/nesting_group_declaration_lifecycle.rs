#![forbid(unsafe_code)]
//! Composed authored lifecycle qualification for #527 on accepted #642.
//! Independent oracles: Nesting WD20260122 §§3.3, 3.3.1, 3.4, 5 and 6.1;
//! selected Syntax block-contents supplement f9712554. Eligibility follows
//! style-containing grammars, rather than an exhaustive at-rule name list.
//! Conditions, ancestry and pseudo-element identities remain symbolic.
use surgeist_css::*;

fn declarations(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect()
}

fn rule_at<'a>(sheet: &'a CssNormalizedSheet, source: &str, marker: &str) -> &'a CssRuleContext {
    let offset = source.find(marker).unwrap();
    sheet
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Rule(rule)
                if rule
                    .position()
                    .is_some_and(|position| position.byte_offset().value() == offset) =>
            {
                Some(rule)
            }
            _ => None,
        })
        .expect("retained original rule occurrence")
}

fn selectors(rule: &CssRuleContext) -> &CssSelectorContext {
    match rule.kind() {
        CssRuleContextKindRef::Style(value) | CssRuleContextKindRef::ScopedStyle(value) => value,
        _ => panic!("selector-bearing style occurrence"),
    }
}

fn assert_parent_list(context: &CssSelectorContext) {
    let [first, second] = context.selectors() else {
        panic!("complete two-member parent list");
    };
    assert_eq!(first.selector(), &CssSelector::Class("P".to_owned()));
    assert!(second.selector().has_pseudo_elements());
    assert!(context.parent().is_none());
}

fn assert_values<'a>(
    source: &str,
    sheet: &'a CssNormalizedSheet,
    expected: &[(&str, &str, CssImportance)],
) -> Vec<&'a CssNormalizedDeclaration> {
    let values = declarations(sheet);
    assert_eq!(values.len(), expected.len());
    for (ordinal, (value, (name, css, importance))) in values.iter().zip(expected).enumerate() {
        let declaration = value.source();
        assert_eq!(
            declaration.known().unwrap().property().canonical_name(),
            *name
        );
        assert_eq!(
            declaration
                .value_components()
                .serialize()
                .unwrap()
                .as_css()
                .trim(),
            *css
        );
        assert_eq!(declaration.importance(), *importance);
        assert_eq!(value.order(), ordinal);
        let offset = source
            .find(&format!("{name}:{css}"))
            .or_else(|| source.find(&format!("{name}: {css}")))
            .unwrap();
        let position = declaration.position().unwrap();
        assert_eq!(position.byte_offset().value(), offset);
        let line_start = source[..offset]
            .rfind('\n')
            .map_or(0, |newline| newline + 1);
        assert_eq!(
            position.column().value() as usize,
            source[line_start..offset].encode_utf16().count()
        );
        assert_eq!(declaration.parsed_name().unwrap().source().as_str(), source);
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
        assert_eq!(
            declaration
                .parsed_name()
                .unwrap()
                .span()
                .start()
                .byte_offset()
                .value(),
            offset
        );
    }
    values
}

fn ordinary_children(rule: &CssRule) -> &[CssRule] {
    match rule {
        CssRule::Media(value) => value.rules(),
        CssRule::Supports(value) => value.rules(),
        CssRule::Container(value) => value.rules(),
        CssRule::LayerBlock(value) => value.rules(),
        CssRule::When(value) => value.rules(),
        CssRule::Else(value) => value.rules(),
        _ => panic!("ordinary style-containing group"),
    }
}

fn assert_group_payload(context: &CssRuleContext, rule: &CssRule) {
    match (context.kind(), rule) {
        (CssRuleContextKindRef::Media(query), CssRule::Media(value)) => {
            assert_eq!(query, value.query())
        }
        (CssRuleContextKindRef::Supports(condition), CssRule::Supports(value)) => {
            assert_eq!(condition, value.condition())
        }
        (CssRuleContextKindRef::Container { prelude }, CssRule::Container(value)) => {
            assert_eq!(prelude, value.prelude())
        }
        (CssRuleContextKindRef::LayerBlock(name), CssRule::LayerBlock(value)) => {
            assert_eq!(name, value.name())
        }
        (CssRuleContextKindRef::When(condition), CssRule::When(value)) => {
            assert_eq!(condition, value.condition())
        }
        (CssRuleContextKindRef::Else(condition), CssRule::Else(value)) => {
            assert_eq!(condition, value.condition())
        }
        (CssRuleContextKindRef::Scope { root, limit }, CssRule::Scope(value)) => {
            assert_eq!(root, value.root());
            assert_eq!(limit, value.limit());
        }
        _ => panic!("intact symbolic group header"),
    }
}

fn assert_child_runs(declarations: &CssDeclarationList, rules: &[CssRule]) {
    assert_eq!(declarations.len(), 1);
    let [CssRule::Media(media), CssRule::NestedDeclarations(after)] = rules else {
        panic!("child media then child declaration run");
    };
    let [CssRule::NestedDeclarations(inner)] = media.rules() else {
        panic!("direct child media declaration run");
    };
    assert_eq!(inner.declarations().len(), 1);
    assert_eq!(after.declarations().len(), 1);
}

const MATRIX_BODY: &str = "width:1px;height:2px;.Child{color:blue;@media screen{opacity:0.2}width:3px}color:green!important;opacity:0.3";

#[test]
fn eligible_nested_bodies_retain_two_runs_full_parent_identity_and_child_restoration() {
    // These rows exercise currently admitted eligible grammars; the source's
    // generic eligibility rule is not defined by this characterization table.
    for (header, starter) in [
        ("@media screen", ""),
        ("@supports (display:grid)", ""),
        ("@container (width>1px)", ""),
        ("@layer palette", ""),
        ("@layer", ""),
        ("@scope", ""),
        ("@when media(width)", ""),
        ("@else", "@media screen{}"),
    ] {
        let source = format!(
            "/*🦀*/\n.P,.P::before{{color:red;opacity:0.1;{starter}{header}{{{MATRIX_BODY}}}display:block;color:black}}.Outside{{height:4px}}"
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{header}: {:?}", report.diagnostics());
        let before = report.clone();
        let [CssRule::Style(parent), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("parent and outside sibling");
        };
        assert_eq!(parent.declarations().len(), 2);
        let group_index = usize::from(!starter.is_empty());
        let group = &parent.rules()[group_index];
        let [CssRule::NestedDeclarations(trailing)] = &parent.rules()[group_index + 1..] else {
            panic!("parent trailing two-declaration run");
        };
        assert_eq!(trailing.declarations().len(), 2);
        if let CssRule::Scope(scope) = group {
            let [
                CssScopedRule::NestedDeclarations(first),
                CssScopedRule::Style(child),
                CssScopedRule::NestedDeclarations(last),
            ] = scope.rules().rules()
            else {
                panic!("scope runs surround its own scoped child");
            };
            assert_eq!(first.declarations().len(), 2);
            assert_eq!(last.declarations().len(), 2);
            assert_child_runs(child.declarations(), child.rules());
        } else {
            let [
                CssRule::NestedDeclarations(first),
                CssRule::Style(child),
                CssRule::NestedDeclarations(last),
            ] = ordinary_children(group)
            else {
                panic!("group runs surround child style");
            };
            assert_eq!(first.declarations().len(), 2);
            assert_eq!(last.declarations().len(), 2);
            assert_child_runs(child.declarations(), child.rules());
        }
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let values = assert_values(
            &source,
            &normalized,
            &[
                ("color", "red", CssImportance::Normal),
                ("opacity", "0.1", CssImportance::Normal),
                ("width", "1px", CssImportance::Normal),
                ("height", "2px", CssImportance::Normal),
                ("color", "blue", CssImportance::Normal),
                ("opacity", "0.2", CssImportance::Normal),
                ("width", "3px", CssImportance::Normal),
                ("color", "green", CssImportance::Important),
                ("opacity", "0.3", CssImportance::Normal),
                ("display", "block", CssImportance::Normal),
                ("color", "black", CssImportance::Normal),
                ("height", "4px", CssImportance::Normal),
            ],
        );
        let parent_context = rule_at(&normalized, &source, ".P,");
        let parent_selectors = selectors(parent_context);
        assert_parent_list(parent_selectors);
        let group_context = rule_at(&normalized, &source, header);
        assert_group_payload(group_context, group);
        assert!(group_context.parent().unwrap().same_context(parent_context));
        for index in [0, 1, 2, 3, 7, 8, 9, 10] {
            assert!(
                values[index]
                    .selector_context()
                    .same_context(parent_selectors)
            );
        }
        for (left, right, owner) in [
            (2, 3, group_context),
            (7, 8, group_context),
            (9, 10, parent_context),
        ] {
            assert!(
                values[left]
                    .rule_context()
                    .same_context(values[right].rule_context())
            );
            assert!(
                values[left]
                    .rule_context()
                    .parent()
                    .unwrap()
                    .same_context(owner)
            );
            assert!(
                matches!(values[left].rule_context().kind(), CssRuleContextKindRef::NestedDeclarations(inherited) if inherited.same_context(parent_selectors))
            );
        }
        assert!(
            !values[2]
                .rule_context()
                .same_context(values[7].rule_context())
        );
        let child = rule_at(&normalized, &source, ".Child");
        let child_selectors = selectors(child);
        assert!(child.parent().unwrap().same_context(group_context));
        for index in [4, 5, 6] {
            assert!(
                values[index]
                    .selector_context()
                    .same_context(child_selectors)
            );
        }
        if matches!(group, CssRule::Scope(_)) {
            assert!(child_selectors.parent().is_none());
            assert!(
                child_selectors
                    .scope_context()
                    .unwrap()
                    .same_context(group_context)
            );
            assert_eq!(
                child_selectors.selectors()[0].binding(),
                CssSelectorBinding::Absolute
            );
        } else {
            assert!(
                child_selectors
                    .parent()
                    .unwrap()
                    .same_context(parent_selectors)
            );
            assert_eq!(
                child_selectors.selectors()[0].binding(),
                CssSelectorBinding::ImplicitDescendant
            );
        }
        let child_media = rule_at(&normalized, &source, "@media screen{opacity");
        assert!(child_media.parent().unwrap().same_context(child));
        assert!(
            values[5]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(child_media)
        );
        assert!(
            values[6]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(child)
        );
        assert!(!values[11].selector_context().same_context(parent_selectors));
        assert_eq!(report, before);
    }
}

#[test]
fn mixed_groups_scope_anchors_and_local_style_rebinding_survive_complete_specified_output() {
    let source = ".P,.P::before{color:red;@media screen{width:1px;@layer palette{height:2px;@when media(width){opacity:0.2;@scope (& > .Root) to (& .Limit){padding:3px;& .Child{color:blue;@supports (display:grid){height:4px;& .Leaf{opacity:0.4}width:5px}margin:6px}opacity:0.3}width:7px}@else{color:green!important}display:block}height:8px}color:black}.Outside{height:9px}";
    let expected = ".P, .P::before { color: red; @media screen { width: 1px; @layer palette { height: 2px; @when media(width) { opacity: 0.2; @scope (& > .Root) to (& .Limit) { padding: 3px; & .Child { color: blue; @supports (display:grid) { height: 4px; & .Leaf { opacity: 0.4; } width: 5px; } margin: 6px; } opacity: 0.3; } width: 7px; } @else { color: green !important; } display: block; } height: 8px; } color: black; }\n.Outside { height: 9px; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let expected_values = [
        ("color", "red", CssImportance::Normal),
        ("width", "1px", CssImportance::Normal),
        ("height", "2px", CssImportance::Normal),
        ("opacity", "0.2", CssImportance::Normal),
        ("padding", "3px", CssImportance::Normal),
        ("color", "blue", CssImportance::Normal),
        ("height", "4px", CssImportance::Normal),
        ("opacity", "0.4", CssImportance::Normal),
        ("width", "5px", CssImportance::Normal),
        ("margin", "6px", CssImportance::Normal),
        ("opacity", "0.3", CssImportance::Normal),
        ("width", "7px", CssImportance::Normal),
        ("color", "green", CssImportance::Important),
        ("display", "block", CssImportance::Normal),
        ("height", "8px", CssImportance::Normal),
        ("color", "black", CssImportance::Normal),
        ("height", "9px", CssImportance::Normal),
    ];
    let values = assert_values(source, &normalized, &expected_values);
    let parent = rule_at(&normalized, source, ".P,");
    let parent_selectors = selectors(parent);
    assert_parent_list(parent_selectors);
    let media = rule_at(&normalized, source, "@media");
    let layer = rule_at(&normalized, source, "@layer");
    let when = rule_at(&normalized, source, "@when");
    let scope = rule_at(&normalized, source, "@scope");
    let otherwise = rule_at(&normalized, source, "@else");
    for (child, owner) in [
        (media, parent),
        (layer, media),
        (when, layer),
        (scope, when),
        (otherwise, layer),
    ] {
        assert!(child.parent().unwrap().same_context(owner));
    }
    let CssRuleContextKindRef::When(condition) = when.kind() else {
        panic!("symbolic When condition");
    };
    let CssValueOrigin::Parsed(origin) = condition.origin() else {
        panic!("original condition provenance");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("media(width)").unwrap()
    );
    assert!(matches!(
        otherwise.kind(),
        CssRuleContextKindRef::Else(None)
    ));
    let CssRuleContextKindRef::Scope {
        root: Some(root),
        limit: Some(limit),
    } = scope.kind()
    else {
        panic!("both symbolic scope boundaries");
    };
    let [CssScopeSelector::Selector(CssSelector::Complex(root))] = root.selectors() else {
        panic!("explicit root parent anchor");
    };
    let [CssScopeSelector::Selector(CssSelector::Complex(limit))] = limit.selectors() else {
        panic!("explicit limit scope anchor");
    };
    assert_eq!(root.first().nesting_selectors(), 1);
    assert!(!root.first().has_scope_anchor());
    assert_eq!(limit.first().nesting_selectors(), 0);
    assert!(limit.first().has_scope_anchor());
    let child = rule_at(&normalized, source, "& .Child");
    let child_selectors = selectors(child);
    assert!(child_selectors.parent().is_none());
    assert_eq!(
        child_selectors.selectors()[0].binding(),
        CssSelectorBinding::ScopeAnchors
    );
    assert!(child_selectors.scope_context().unwrap().same_context(scope));
    assert!(child.parent().unwrap().same_context(scope));
    let supports = rule_at(&normalized, source, "@supports");
    let leaf = rule_at(&normalized, source, "& .Leaf");
    assert!(supports.parent().unwrap().same_context(child));
    assert!(leaf.parent().unwrap().same_context(supports));
    assert!(
        selectors(leaf)
            .parent()
            .unwrap()
            .same_context(child_selectors)
    );
    assert_eq!(
        selectors(leaf).selectors()[0].binding(),
        CssSelectorBinding::ExplicitAnchors
    );
    for index in [5, 6, 8, 9] {
        assert!(
            values[index]
                .selector_context()
                .same_context(child_selectors)
        );
    }
    for index in [0, 1, 2, 3, 4, 10, 11, 12, 13, 14, 15] {
        assert!(
            values[index]
                .selector_context()
                .same_context(parent_selectors)
        );
    }
    for (index, owner) in [
        (4, scope),
        (6, supports),
        (8, supports),
        (9, child),
        (10, scope),
        (11, when),
        (12, otherwise),
        (13, layer),
        (14, media),
        (15, parent),
    ] {
        assert!(
            values[index]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(owner)
        );
    }
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    let reparsed = parse_sheet(expected);
    assert!(reparsed.is_clean(), "{:?}", reparsed.diagnostics());
    assert_values(
        expected,
        &normalize_sheet(reparsed.syntax()).unwrap(),
        &expected_values,
    );
    let reused = CssSheet::try_from_rules(report.syntax().rules().to_vec()).unwrap();
    assert!(reused.location().is_none());
    assert_eq!(reused.to_specified_css().unwrap(), expected);
    assert_values(source, &normalize_sheet(&reused).unwrap(), &expected_values);
    assert_eq!(report, before);
}

#[test]
fn rule_list_and_definition_bodies_do_not_become_style_declaration_groups() {
    for (header, starter) in [
        ("@media screen", ""),
        ("@supports (display:grid)", ""),
        ("@container (width>1px)", ""),
        ("@layer palette", ""),
        ("@layer", ""),
        ("@scope", ""),
        ("@when media(width)", ""),
        ("@else", "@media screen{}"),
    ] {
        let source = format!(
            "{starter}{header}{{.Child{{color:blue}}color:red;width:1px;}}.Outside{{height:2px}}"
        );
        let report = parse_sheet(&source);
        let normalized = normalize_sheet(report.syntax()).unwrap();
        assert_values(
            &source,
            &normalized,
            &[
                ("color", "blue", CssImportance::Normal),
                ("height", "2px", CssImportance::Normal),
            ],
        );
        assert!(!normalized.items().iter().any(|item| matches!(item, CssNormalizedItem::Rule(rule) if matches!(rule.kind(), CssRuleContextKindRef::NestedDeclarations(_)))));
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one ordinary rule-list tail recovery: {:?}",
                report.diagnostics()
            );
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        let tail = "color:red;width:1px;";
        let start = source.find(tail).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + tail.len()
        );
    }
    for excluded in [
        "@font-face{font-family:Lost;src:url(x)}",
        "@keyframes lost{from{opacity:0}}",
        "@counter-style lost{system:cyclic;symbols:'x'}",
        "@namespace p 'urn:p';",
        "@import 'early.css';",
    ] {
        let source =
            format!(".P,.P::before{{color:red;{excluded}width:1px}}.Outside{{height:2px}}");
        let report = parse_sheet(&source);
        assert_values(
            &source,
            &normalize_sheet(report.syntax()).unwrap(),
            &[
                ("color", "red", CssImportance::Normal),
                ("width", "1px", CssImportance::Normal),
                ("height", "2px", CssImportance::Normal),
            ],
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one explicitly excluded style placement");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePlacement
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        let start = source.find(excluded).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + excluded.len()
        );
    }
    let source = ".P{color:red;@supports-condition --test{display:grid;}width:1px}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent style");
    };
    let [
        CssRule::SupportsCondition(definition),
        CssRule::NestedDeclarations(_),
    ] = parent.rules()
    else {
        panic!("feature-test definition then style run");
    };
    let [CssSupportsTestItem::Declarations(run)] = definition.body().items() else {
        panic!("definition-owned feature test");
    };
    assert_eq!(run.declarations()[0].property(), "display");
    assert_values(
        source,
        &normalize_sheet(report.syntax()).unwrap(),
        &[
            ("color", "red", CssImportance::Normal),
            ("width", "1px", CssImportance::Normal),
        ],
    );
}

#[test]
fn mixed_carrier_boundaries_preserve_runs_and_ancestry_with_child_local_limit_loss() {
    let headers = [
        "@media screen",
        "@supports (display:grid)",
        "@container (width>1px)",
        "@layer audit",
        "@scope (.R)",
    ];
    for depth in [63, 64, 65, 127, 128, 129, 256, 257] {
        // Parent1 + wrappers(depth-4) + When1 + Child1 + child's Media1.
        let mut source = String::from(".P,.P::before{color:red;");
        let mut openings = Vec::new();
        for ordinal in 0..depth - 4 {
            openings.push(source.len());
            source.push_str(headers[ordinal % headers.len()]);
            source.push('{');
        }
        source.push_str("@when media(width){");
        source.push_str(MATRIX_BODY);
        source.push('}');
        source.push_str(&"}".repeat(depth - 4));
        source.push_str("display:block;color:black}.Outside{height:4px}");
        let report = parse_sheet(&source);
        let before = report.clone();
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let mut expected = vec![
            ("color", "red", CssImportance::Normal),
            ("width", "1px", CssImportance::Normal),
            ("height", "2px", CssImportance::Normal),
            ("color", "blue", CssImportance::Normal),
        ];
        if depth <= 256 {
            expected.push(("opacity", "0.2", CssImportance::Normal));
        }
        expected.extend([
            ("width", "3px", CssImportance::Normal),
            ("color", "green", CssImportance::Important),
            ("opacity", "0.3", CssImportance::Normal),
            ("display", "block", CssImportance::Normal),
            ("color", "black", CssImportance::Normal),
            ("height", "4px", CssImportance::Normal),
        ]);
        let values = assert_values(&source, &normalized, &expected);
        let parent = rule_at(&normalized, &source, ".P,");
        let parent_selectors = selectors(parent);
        assert_parent_list(parent_selectors);
        let when = rule_at(&normalized, &source, "@when");
        let mut ancestor = when.parent();
        for offset in openings.iter().rev() {
            let current = ancestor.unwrap();
            assert_eq!(current.position().unwrap().byte_offset().value(), *offset);
            ancestor = current.parent();
        }
        assert!(ancestor.unwrap().same_context(parent));
        let child = rule_at(&normalized, &source, ".Child");
        assert!(child.parent().unwrap().same_context(when));
        let child_selectors = selectors(child);
        assert!(child_selectors.parent().is_none());
        let mut nearest_scope = when.parent();
        while !matches!(
            nearest_scope.unwrap().kind(),
            CssRuleContextKindRef::Scope { .. }
        ) {
            nearest_scope = nearest_scope.unwrap().parent();
        }
        assert!(
            child_selectors
                .scope_context()
                .unwrap()
                .same_context(nearest_scope.unwrap())
        );
        let child_width = if depth <= 256 { 5 } else { 4 };
        for index in [3, child_width] {
            assert!(
                values[index]
                    .selector_context()
                    .same_context(child_selectors)
            );
        }
        if depth <= 256 {
            assert!(values[4].selector_context().same_context(child_selectors));
        }
        for (index, value) in values.iter().enumerate().take(values.len() - 1) {
            if index != 3 && index != child_width && !(depth <= 256 && index == 4) {
                assert!(value.selector_context().same_context(parent_selectors));
            }
        }
        assert!(
            !values
                .last()
                .unwrap()
                .selector_context()
                .same_context(parent_selectors)
        );
        if depth <= 256 {
            assert!(
                report.is_clean(),
                "depth {depth}: {:?}",
                report.diagnostics()
            );
            let inner = rule_at(&normalized, &source, "@media screen{opacity");
            assert!(inner.parent().unwrap().same_context(child));
        } else {
            let unit = "@media screen{opacity:0.2}";
            let start = source.find(unit).unwrap();
            let [diagnostic] = report.diagnostics() else {
                panic!(
                    "only over-limit child group loss: {:?}",
                    report.diagnostics()
                );
            };
            assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
            assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
            assert_eq!(diagnostic.span().start().byte_offset().value(), start);
            assert_eq!(
                diagnostic.span().end().byte_offset().value(),
                start + unit.len()
            );
        }
        assert_eq!(report, before);
    }
}

#[test]
fn checked_reuse_requires_actual_style_ancestry_and_run_output_coalesces_only_on_reparse() {
    let source = ".P,.P::before{@media screen{width:1px;height:2px;.Child{color:blue}color:green!important;opacity:0.3}}";
    let report = parse_sheet(source);
    assert!(report.is_clean());
    let before = report.clone();
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("actual parsed style ancestry");
    };
    let [CssRule::Media(media)] = parent.rules() else {
        panic!("parsed nested group");
    };
    let [
        first @ CssRule::NestedDeclarations(_),
        CssRule::Style(_),
        last @ CssRule::NestedDeclarations(_),
    ] = media.rules()
    else {
        panic!("two separate original runs around a child");
    };
    let context = CssNamespaceContext::default();
    let detached =
        CssMediaRule::try_new(media.query().clone(), media.rules().to_vec(), &context).unwrap_err();
    assert_eq!(
        detached.kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
    assert_eq!(detached.path(), &[0]);
    assert_eq!(
        detached.position().unwrap().byte_offset().value(),
        source.find("width:1px").unwrap()
    );
    let detached_sheet = CssSheet::try_from_rules(vec![CssRule::Media(media.clone())]).unwrap_err();
    assert_eq!(
        detached_sheet.kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
    assert_eq!(detached_sheet.path(), &[0, 0]);
    let wrapper = CssMediaRule::try_new(
        media.query().clone(),
        vec![CssRule::Style(parent.clone())],
        &context,
    )
    .unwrap();
    let reused = CssSheet::try_from_rules(vec![CssRule::Media(wrapper)]).unwrap();
    let normalized = normalize_sheet(&reused).unwrap();
    let values = assert_values(
        source,
        &normalized,
        &[
            ("width", "1px", CssImportance::Normal),
            ("height", "2px", CssImportance::Normal),
            ("color", "blue", CssImportance::Normal),
            ("color", "green", CssImportance::Important),
            ("opacity", "0.3", CssImportance::Normal),
        ],
    );
    let reused_parent = rule_at(&normalized, source, ".P,");
    let parent_selectors = selectors(reused_parent);
    assert_parent_list(parent_selectors);
    for index in [0, 1, 3, 4] {
        assert!(
            values[index]
                .selector_context()
                .same_context(parent_selectors)
        );
    }
    assert!(
        values[2]
            .selector_context()
            .parent()
            .unwrap()
            .same_context(parent_selectors)
    );
    assert!(reused_parent.parent().unwrap().position().is_none());
    let fragment = CssWhenRule::try_new_in_style(
        parse_when_condition("media(width)").unwrap(),
        vec![first.clone(), last.clone()],
        &context,
    )
    .unwrap();
    assert_eq!(fragment.rules().len(), 2);
    let expected =
        "@when media(width) { width: 1px; height: 2px; color: green !important; opacity: 0.3; }";
    assert_eq!(
        CssRule::When(fragment.clone()).to_specified_css().unwrap(),
        expected
    );
    let detached_fragment = CssSheet::try_from_rules(vec![CssRule::When(fragment)]).unwrap_err();
    assert_eq!(
        detached_fragment.kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
    assert_eq!(detached_fragment.path(), &[0, 0]);
    let emitted = format!(".P,.P::before{{{expected}}}");
    let reparsed = parse_sheet(&emitted);
    assert!(reparsed.is_clean());
    let [CssRule::Style(parent)] = reparsed.syntax().rules() else {
        panic!("new actual style ancestry");
    };
    let [CssRule::When(when)] = parent.rules() else {
        panic!("specified group");
    };
    let [CssRule::NestedDeclarations(run)] = when.rules() else {
        panic!("adjacent emitted runs coalesce as §6.1 permits");
    };
    assert_eq!(run.declarations().len(), 4);
    let normalized = normalize_sheet(reparsed.syntax()).unwrap();
    let values = assert_values(
        &emitted,
        &normalized,
        &[
            ("width", "1px", CssImportance::Normal),
            ("height", "2px", CssImportance::Normal),
            ("color", "green", CssImportance::Important),
            ("opacity", "0.3", CssImportance::Normal),
        ],
    );
    let emitted_parent = selectors(rule_at(&normalized, &emitted, ".P,"));
    assert_parent_list(emitted_parent);
    assert!(
        values
            .iter()
            .all(|value| value.selector_context().same_context(emitted_parent))
    );
    assert_eq!(report, before);
}
