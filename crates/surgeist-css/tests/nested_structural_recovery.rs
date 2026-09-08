use surgeist_css::{
    CssErrorCode, CssKeyframeSelector, CssPropertyNameRef, CssRecoveryAction, CssRule,
    CssScopedRule, CssSelector, ErrorKind, parse_sheet,
};

fn style_name(rule: &CssRule) -> &str {
    let CssRule::Style(rule) = rule else {
        panic!("expected style rule, got {rule:?}");
    };
    let CssSelector::Class(name) = rule.selector() else {
        panic!("expected class selector, got {:?}", rule.selector());
    };
    name
}

fn property_names(declarations: &surgeist_css::CssKeyframeDeclarationList) -> Vec<&str> {
    declarations
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            _ => panic!("unexpected future property-name kind"),
        })
        .collect()
}

fn assert_drop(
    source: &str,
    diagnostic: &surgeist_css::CssRecoveryDiagnostic,
    unit: &str,
    occurrence: usize,
    code: CssErrorCode,
    action: CssRecoveryAction,
    responsible: usize,
) {
    let start = source
        .match_indices(unit)
        .nth(occurrence)
        .map(|(start, _)| start)
        .expect("recovery unit in source");
    assert_eq!(diagnostic.error().code(), code);
    assert_eq!(diagnostic.action(), action);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + unit.len()
    );
    assert!(diagnostic.span().start() < diagnostic.span().end());
}

#[test]
fn nested_structural_group_contexts_retain_siblings_around_balanced_at_rule_failure() {
    let failed = "@mystery fn({x; y}) { .lost { color: black; } }";
    let cases = [
        format!("@layer theme {{ .before {{ color: red; }} {failed} .after {{ color: blue; }} }}"),
        format!("@media screen {{ .before {{ color: red; }} {failed} .after {{ color: blue; }} }}"),
        format!(
            "@supports (display: grid) {{ .before {{ color: red; }} {failed} .after {{ color: blue; }} }}"
        ),
        format!(
            "@container (width > 1px) {{ .before {{ color: red; }} {failed} .after {{ color: blue; }} }}"
        ),
    ];

    for source in cases {
        let report = parse_sheet(&source);
        let children = match &report.syntax().rules()[0] {
            CssRule::LayerBlock(rule) => rule.rules(),
            CssRule::Media(rule) => rule.rules(),
            CssRule::Supports(rule) => rule.rules(),
            CssRule::Container(rule) => rule.rules(),
            unexpected => panic!("expected group parent, got {unexpected:?}"),
        };
        assert_eq!(children.len(), 2, "{source}");
        assert_eq!(style_name(&children[0]), "before");
        assert_eq!(style_name(&children[1]), "after");

        let [diagnostic] = report.diagnostics() else {
            panic!("expected one nested at-rule diagnostic for {source}");
        };
        let start = source.find(failed).unwrap();
        assert_drop(
            &source,
            diagnostic,
            failed,
            0,
            CssErrorCode::UnknownAtRule,
            CssRecoveryAction::DropAtRule,
            start,
        );
        let ErrorKind::UnknownAtRule(detail) = diagnostic.error().kind() else {
            panic!("expected unknown at-rule detail");
        };
        assert_eq!(detail.name().as_str(), "mystery");
    }
}

#[test]
fn nested_structural_qualified_failures_recover_in_group_scope_and_style_contexts() {
    let failed = ".bad:is(.one, .two), { color: black; }";
    let group_source =
        format!("@media screen {{ .before {{ color: red; }} {failed} .after {{ color: blue; }} }}");
    let group = parse_sheet(&group_source);
    let [CssRule::Media(media)] = group.syntax().rules() else {
        panic!("expected retained media rule");
    };
    assert_eq!(style_name(&media.rules()[0]), "before");
    assert_eq!(style_name(&media.rules()[1]), "after");
    let group_start = group_source.find(failed).unwrap();
    assert_drop(
        &group_source,
        &group.diagnostics()[0],
        failed,
        0,
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        group_start + failed.find('{').unwrap(),
    );

    let scope_source =
        format!("@scope {{ .before {{ color: red; }} {failed} .after {{ color: blue; }} }}");
    let scope = parse_sheet(&scope_source);
    let [CssRule::Scope(scope_rule)] = scope.syntax().rules() else {
        panic!("expected retained scope rule");
    };
    assert!(matches!(
        scope_rule.rules().rules(),
        [CssScopedRule::Style(_), CssScopedRule::Style(_)]
    ));
    let scope_start = scope_source.find(failed).unwrap();
    assert_drop(
        &scope_source,
        &scope.diagnostics()[0],
        failed,
        0,
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        scope_start + failed.find('{').unwrap(),
    );

    let style_source = format!(
        ".host {{ color: red; & .before {{ width: 1px; }} {failed} & .after {{ height: 2px; }} opacity: 1; }}"
    );
    let style = parse_sheet(&style_source);
    assert_eq!(style.syntax().rules().len(), 4);
    assert!(
        style
            .syntax()
            .rules()
            .iter()
            .all(|rule| matches!(rule, CssRule::Style(_)))
    );
    let style_start = style_source.find(failed).unwrap();
    assert_drop(
        &style_source,
        &style.diagnostics()[0],
        failed,
        0,
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        style_start + failed.find('{').unwrap(),
    );
}

#[test]
fn nested_structural_repeated_failures_retain_empty_permitted_group_and_later_sibling() {
    let first = "@one fn({a; b});";
    let second = ".bad, { color: red; }";
    let source = format!("@layer empty {{ {first} {second} }} .after {{ color: blue; }}");

    let report = parse_sheet(&source);
    let [CssRule::LayerBlock(layer), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("expected empty layer and later style sibling");
    };
    assert!(layer.rules().is_empty());
    assert_eq!(style_name(&CssRule::Style(after.clone())), "after");
    assert_eq!(report.diagnostics().len(), 2);
    assert_drop(
        &source,
        &report.diagnostics()[0],
        first,
        0,
        CssErrorCode::UnknownAtRule,
        CssRecoveryAction::DropAtRule,
        source.find(first).unwrap(),
    );
    assert_drop(
        &source,
        &report.diagnostics()[1],
        second,
        0,
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        source.find(second).unwrap() + second.find('{').unwrap(),
    );
}

#[test]
fn nested_selectors3_failures_retain_valid_pseudo_siblings_in_authored_order() {
    let first = ".bad:lang() { color: black; }";
    let second = ".bad::first-line:hover { color: black; }";
    let source = format!(
        "@media screen {{ .before:target {{ color: red; }} {first} {second} .after:visited {{ color: blue; }} }}"
    );
    let report = parse_sheet(&source);

    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("expected retained media group")
    };
    assert_eq!(media.rules().len(), 2);
    assert_eq!(report.diagnostics().len(), 2);
    assert_drop(
        &source,
        &report.diagnostics()[0],
        first,
        0,
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        source.find(first).unwrap() + first.find(')').unwrap(),
    );
    assert_drop(
        &source,
        &report.diagnostics()[1],
        second,
        0,
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        source.find(second).unwrap() + second.find(":hover").unwrap(),
    );
}

#[test]
fn nested_structural_style_and_scope_at_rule_failures_keep_authored_siblings() {
    let failed = "@mystery fn({x; y});";
    let style_source = format!(
        ".host {{ color: red; & .before {{ width: 1px; }} {failed} & .after {{ height: 2px; }} opacity: 1; }}"
    );
    let style = parse_sheet(&style_source);
    assert_eq!(style.syntax().rules().len(), 4);
    let style_start = style_source.find(failed).unwrap();
    assert_drop(
        &style_source,
        &style.diagnostics()[0],
        failed,
        0,
        CssErrorCode::UnknownAtRule,
        CssRecoveryAction::DropAtRule,
        style_start,
    );

    let scope_source =
        format!("@scope {{ .before {{ color: red; }} {failed} .after {{ color: blue; }} }}");
    let scope = parse_sheet(&scope_source);
    let [CssRule::Scope(scope_rule)] = scope.syntax().rules() else {
        panic!("expected retained scope rule");
    };
    assert!(matches!(
        scope_rule.rules().rules(),
        [CssScopedRule::Style(_), CssScopedRule::Style(_)]
    ));
    let scope_start = scope_source.find(failed).unwrap();
    assert_drop(
        &scope_source,
        &scope.diagnostics()[0],
        failed,
        0,
        CssErrorCode::UnknownAtRule,
        CssRecoveryAction::DropAtRule,
        scope_start,
    );
}

#[test]
fn nested_structural_keyframes_recover_blocks_and_declarations_in_authored_order() {
    let invalid_selector = "55 { opacity: .5; }";
    let invalid_declaration = "mystery: fn({a; b});";
    let source = format!(
        "@keyframes fade {{ from {{ opacity: 0; {invalid_declaration} width: 1px; }} {invalid_selector} to {{ opacity: 1; }} }} .after {{ color: blue; }}"
    );

    let report = parse_sheet(&source);
    let [CssRule::Keyframes(keyframes), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("expected recovered keyframes and later style sibling");
    };
    assert_eq!(style_name(&CssRule::Style(after.clone())), "after");
    assert_eq!(keyframes.blocks().len(), 2);
    assert!(matches!(
        keyframes.blocks()[0].selectors().selectors(),
        [CssKeyframeSelector::From]
    ));
    assert!(matches!(
        keyframes.blocks()[1].selectors().selectors(),
        [CssKeyframeSelector::To]
    ));
    assert_eq!(
        property_names(keyframes.blocks()[0].declarations()),
        ["opacity", "width"]
    );
    assert_eq!(report.diagnostics().len(), 2);
    assert_drop(
        &source,
        &report.diagnostics()[0],
        invalid_declaration,
        0,
        CssErrorCode::UnknownProperty,
        CssRecoveryAction::DropDeclaration,
        source.find(invalid_declaration).unwrap(),
    );
    assert_drop(
        &source,
        &report.diagnostics()[1],
        invalid_selector,
        0,
        CssErrorCode::UnexpectedEnd,
        CssRecoveryAction::DropKeyframeBlock,
        source.find(invalid_selector).unwrap(),
    );
}

#[test]
fn nested_structural_keyframe_declaration_loss_retains_empty_authored_parents() {
    let invalid_declaration = "mystery: 1;";
    let empty_block = format!("from {{ {invalid_declaration} }}");
    let source = format!("@keyframes fade {{ {empty_block} }} .after {{ color: blue; }}");

    let report = parse_sheet(&source);
    let [CssRule::Keyframes(keyframes), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("expected the empty keyframe parent and later style rule");
    };
    let [block] = keyframes.blocks() else {
        panic!("expected the authored keyframe block");
    };
    assert!(block.declarations().is_empty());
    assert_eq!(style_name(&CssRule::Style(after.clone())), "after");
    assert_eq!(report.diagnostics().len(), 1);
    assert_drop(
        &source,
        &report.diagnostics()[0],
        invalid_declaration,
        0,
        CssErrorCode::UnknownProperty,
        CssRecoveryAction::DropDeclaration,
        source.find(invalid_declaration).unwrap(),
    );
}

#[test]
fn nested_structural_keyframes_drop_malformed_and_repeated_balanced_blocks_only() {
    let malformed = "25% { opacity: .25; @media fn(a, b) { width: 1px; } height: 2px; }";
    let invalid_selector = "fn(a, b) { opacity: .5; }";
    let source = format!(
        "@keyframes fade {{ from {{ opacity: 0; }} {malformed} {invalid_selector} to {{ opacity: 1; }} }}"
    );

    let report = parse_sheet(&source);
    let [CssRule::Keyframes(keyframes)] = report.syntax().rules() else {
        panic!("expected representable keyframes after dropping malformed blocks");
    };
    assert_eq!(keyframes.blocks().len(), 2);
    assert!(matches!(
        keyframes.blocks()[0].selectors().selectors(),
        [CssKeyframeSelector::From]
    ));
    assert!(matches!(
        keyframes.blocks()[1].selectors().selectors(),
        [CssKeyframeSelector::To]
    ));
    assert_eq!(report.diagnostics().len(), 2);
    assert_drop(
        &source,
        &report.diagnostics()[0],
        malformed,
        0,
        CssErrorCode::InvalidAtRulePlacement,
        CssRecoveryAction::DropKeyframeBlock,
        source.find("@media").unwrap(),
    );
    let ErrorKind::InvalidAtRulePlacement(detail) = report.diagnostics()[0].error().kind() else {
        panic!("expected keyframe at-rule placement detail");
    };
    assert_eq!(detail.name().as_str(), "media");
    assert_eq!(
        detail.expected_context().as_str(),
        "a keyframe declaration list"
    );
    assert_drop(
        &source,
        &report.diagnostics()[1],
        invalid_selector,
        0,
        CssErrorCode::UnexpectedToken,
        CssRecoveryAction::DropKeyframeBlock,
        source.find(invalid_selector).unwrap(),
    );
}
