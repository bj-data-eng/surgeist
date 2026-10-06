#![forbid(unsafe_code)]
//! Existing-API retention contracts for Conditional 5 WD20251030 sections 3/4
//! and the adopted complete authored grammar profile in issue 647.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/
//! https://github.com/bj-data-eng/surgeist/issues/647
//! Body recovery follows Conditional 3 and Syntax 3; style-ancestor group
//! contents follow Nesting WD20260122 section 3.3. Conditions remain symbolic.

use surgeist_css::{
    CssDeclaration, CssNormalizedDeclaration, CssNormalizedItem, CssNormalizedSheet,
    CssRecoveryAction, CssRule, CssRuleContext, CssRuleContextKindRef, CssSelector,
    CssSelectorBinding, normalize_sheet, parse_sheet,
};

fn declaration_text(declaration: &CssDeclaration) -> (&str, &str) {
    let name = declaration.parsed_name().expect("original parsed name");
    let value = declaration.parsed_value().expect("original parsed value");
    (
        &name.source().as_str()
            [name.span().start().byte_offset().value()..name.span().end().byte_offset().value()],
        value.source().as_str()
            [value.span().start().byte_offset().value()..value.span().end().byte_offset().value()]
            .trim(),
    )
}

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

fn assert_declarations<'a>(
    source: &str,
    sheet: &'a CssNormalizedSheet,
    expected: &[(&str, &str)],
) -> Vec<&'a CssNormalizedDeclaration> {
    let values = declarations(sheet);
    assert_eq!(
        values
            .iter()
            .map(|value| declaration_text(value.source()))
            .collect::<Vec<_>>(),
        expected
    );
    for (index, (value, (name, css))) in values.iter().zip(expected).enumerate() {
        assert_eq!(value.order(), index);
        let origin = value.source().parsed_name().unwrap();
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            value.source().parsed_value().unwrap().source().as_str(),
            source
        );
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find(&format!("{name}:{css}")).unwrap()
        );
    }
    values
}

fn rule_at<'a>(sheet: &'a CssNormalizedSheet, source: &str, authored: &str) -> &'a CssRuleContext {
    let offset = source.find(authored).unwrap();
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
        .expect("retained rule occurrence at its original source position")
}

#[test]
fn valid_when_conditions_retain_their_body_and_following_sibling_without_evaluation() {
    // The first four cases exercise the three defined media-feature leaf
    // shapes and the supports declaration leaf, rather than broader queries.
    // Later cases use the adopted homogeneous/grouped/not algebra and opaque
    // fallback. This existing surface proves retention, not typed leaf identity.
    for condition in [
        "media(width)",
        "media(width:1px)",
        "media(1px < width < 10px)",
        "supports(display:grid)",
        "media(width) and supports(display:grid)",
        "media(width) or media(height)",
        "not supports(display:grid)",
        "(media(width) or media(height)) and supports(display:grid)",
        "media(unknown-feature)",
        "media()",
        "supports()",
        "()",
        "font-tech(color-COLRv1)",
        "(media(width) and supports(display:grid) or media(height))",
        "media(screen and (width:1px))",
        "supports((display:grid))",
    ] {
        let source = format!("@when {condition}{{.Inside{{color:red}}}}.After{{color:blue}}");
        let report = parse_sheet(&source);
        let original = report.clone();
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let values =
            assert_declarations(&source, &normalized, &[("color", "red"), ("color", "blue")]);
        let group = rule_at(&normalized, &source, "@when");
        let inside = rule_at(&normalized, &source, ".Inside");
        let after = rule_at(&normalized, &source, ".After");
        assert!(group.parent().is_none());
        assert!(inside.parent().unwrap().same_context(group));
        assert!(values[0].rule_context().same_context(inside));
        assert!(values[1].rule_context().same_context(after));
        assert!(after.parent().is_none());
        assert!(
            report.is_clean(),
            "valid adopted condition {condition}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report, original);
        assert!(report.into_validation_result().is_ok());
    }
}

#[test]
fn whitespace_comment_else_chain_retains_omitted_and_later_conditioned_bodies() {
    let source = "@when media(width) and supports(display:grid){.When{color:red}} /*adjacent*/\n@else{.Omitted{color:blue}}\t@else supports(display:flex){.Later{color:green}}.After{height:2px}";
    let report = parse_sheet(source);
    let original = report.clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values = assert_declarations(
        source,
        &normalized,
        &[
            ("color", "red"),
            ("color", "blue"),
            ("color", "green"),
            ("height", "2px"),
        ],
    );
    for (value, group_source, child_source) in [
        (values[0], "@when", ".When"),
        (values[1], "@else{", ".Omitted"),
        (values[2], "@else supports", ".Later"),
    ] {
        let group = rule_at(&normalized, source, group_source);
        let child = rule_at(&normalized, source, child_source);
        assert!(group.parent().is_none());
        assert!(child.parent().unwrap().same_context(group));
        assert!(value.rule_context().same_context(child));
    }
    assert!(values[3].rule_context().parent().is_none());
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report, original);
    assert!(report.into_validation_result().is_ok());
}

#[test]
fn style_nested_when_else_keep_declaration_runs_and_nearest_style_context() {
    let source = ".Parent,#Alt{color:red;@when media(width){width:1px;.Child{height:2px}opacity:1}@else{color:blue}display:block}.Outside{color:green}";
    let report = parse_sheet(source);
    let original = report.clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values = assert_declarations(
        source,
        &normalized,
        &[
            ("color", "red"),
            ("width", "1px"),
            ("height", "2px"),
            ("opacity", "1"),
            ("color", "blue"),
            ("display", "block"),
            ("color", "green"),
        ],
    );
    let parent = rule_at(&normalized, source, ".Parent");
    let when = rule_at(&normalized, source, "@when");
    let otherwise = rule_at(&normalized, source, "@else");
    let child = rule_at(&normalized, source, ".Child");
    let CssRuleContextKindRef::Style(parent_selectors) = parent.kind() else {
        panic!("parent style context");
    };
    assert_eq!(parent_selectors.selectors().len(), 2);
    assert_eq!(
        parent_selectors.selectors()[0].selector(),
        &CssSelector::Class("Parent".to_owned())
    );
    assert_eq!(
        parent_selectors.selectors()[1].selector(),
        &CssSelector::Key("Alt".to_owned())
    );
    assert!(when.parent().unwrap().same_context(parent));
    assert!(otherwise.parent().unwrap().same_context(parent));
    assert!(child.parent().unwrap().same_context(when));
    for index in [0, 1, 3, 4, 5] {
        assert!(
            values[index]
                .selector_context()
                .same_context(parent_selectors)
        );
    }
    for index in [1, 3] {
        assert!(
            values[index]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(when)
        );
    }
    assert!(
        values[4]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(otherwise)
    );
    assert!(
        values[5]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(parent)
    );
    assert!(values[2].rule_context().same_context(child));
    assert!(
        values[2]
            .selector_context()
            .parent()
            .unwrap()
            .same_context(parent_selectors)
    );
    assert_eq!(
        values[2].selector_context().selectors()[0].binding(),
        CssSelectorBinding::ImplicitDescendant
    );
    assert!(!values[6].selector_context().same_context(parent_selectors));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report, original);
    assert!(report.into_validation_result().is_ok());
}

#[test]
fn invalid_outer_when_condition_discards_its_whole_body_and_preserves_parent_runs() {
    for condition in [
        "",
        "media(width) and",
        "media(width) and supports(display:grid) or media(height)",
        "media(width), supports(display:grid)",
        "media(width) trailing",
    ] {
        let failed = format!("@when {condition}{{color:green;.Leak{{width:1px}}}}");
        let source = format!(".Parent{{color:red;{failed}color:blue}}.Outside{{height:2px}}");
        let report = parse_sheet(&source);
        let normalized = normalize_sheet(report.syntax()).unwrap();
        assert_declarations(
            &source,
            &normalized,
            &[("color", "red"), ("color", "blue"), ("height", "2px")],
        );
        let [CssRule::Style(parent), CssRule::Style(outside)] = report.syntax().rules() else {
            panic!("parent and sibling retained");
        };
        assert_eq!(parent.declarations().len(), 1);
        assert_eq!(
            declaration_text(&parent.declarations()[0]),
            ("color", "red")
        );
        let [CssRule::NestedDeclarations(after)] = parent.rules() else {
            panic!("later declarations transferred at at-rule boundary");
        };
        assert_eq!(after.declarations().len(), 1);
        assert_eq!(
            declaration_text(&after.declarations()[0]),
            ("color", "blue")
        );
        assert!(outside.rules().is_empty());
        let [diagnostic] = report.diagnostics() else {
            panic!("one whole invalid when diagnostic");
        };
        let start = source.find(&failed).unwrap();
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + failed.len()
        );
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn free_else_and_original_token_separators_discard_else_without_body_promotion() {
    for prefix in [
        ".Before{color:red}",
        "@media all{.Before{color:red}};",
        "@media all{.Before{color:red}}@unknown;",
        "@media all{.Before{color:red}}.bad,{.Hidden{width:1px}}",
    ] {
        let failed = "@else{.Leak{height:1px}}";
        let source = format!("{prefix}{failed}.After{{color:blue}}");
        let report = parse_sheet(&source);
        let normalized = normalize_sheet(report.syntax()).unwrap();
        assert_declarations(&source, &normalized, &[("color", "red"), ("color", "blue")]);
        let start = source.find(failed).unwrap();
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| {
                diagnostic.span().start().byte_offset().value() == start
                    && diagnostic.span().end().byte_offset().value() == start + failed.len()
            })
            .expect("whole free else recovery unit, even after discarded separator");
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn valid_when_recovers_invalid_body_children_locally_and_keeps_following_sibling() {
    let source = "@when supports(display:grid){.Good{color:red;mystery:1;width:1px}.Bad,{.Leak{height:2px}}.Last{color:blue}}.Outside{height:3px}";
    let report = parse_sheet(source);
    let original = report.clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values = assert_declarations(
        source,
        &normalized,
        &[
            ("color", "red"),
            ("width", "1px"),
            ("color", "blue"),
            ("height", "3px"),
        ],
    );
    let group = rule_at(&normalized, source, "@when");
    assert!(group.parent().is_none());
    for index in [0, 1, 2] {
        assert!(
            values[index]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(group)
        );
    }
    assert!(values[3].rule_context().parent().is_none());
    let [declaration_drop, rule_drop] = report.diagnostics() else {
        panic!("local declaration and child rule diagnostics only");
    };
    for (diagnostic, unit, action) in [
        (
            declaration_drop,
            "mystery:1;",
            CssRecoveryAction::DropDeclaration,
        ),
        (
            rule_drop,
            ".Bad,{.Leak{height:2px}}",
            CssRecoveryAction::DropQualifiedRule,
        ),
    ] {
        let start = source.find(unit).unwrap();
        assert_eq!(diagnostic.action(), action);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len()
        );
    }
    assert_eq!(report, original);
    assert!(report.into_validation_result().is_err());
}
