use surgeist_css::{
    CssDeclarationContextRef, CssErrorCode, CssPropertyNameRef, CssRecoveryAction, CssRule,
    CssScopedRule, CssSelector, CssSelectorCombinator, CssTokenKind, ErrorKind, parse_sheet,
};

fn property_names(declarations: &surgeist_css::CssDeclarationList) -> Vec<&str> {
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
    code: CssErrorCode,
    action: CssRecoveryAction,
    responsible: usize,
) {
    let start = source.find(unit).expect("recovery unit in source");
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
fn block_item_recovery_declaration_error_classes_retain_ordered_siblings() {
    let cases = [
        ("mystery: 1;", CssErrorCode::UnknownProperty, "mystery"),
        ("width: nope;", CssErrorCode::InvalidPropertyValue, "nope"),
        (
            "width: 2px !oops;",
            CssErrorCode::InvalidDeclarationAnnotation,
            "!",
        ),
        ("broken;", CssErrorCode::UnexpectedEnd, ";"),
    ];

    for (unit, code, responsible_text) in cases {
        let source = format!(".x {{ color: red; {unit} --kept: yes; height: 3px; }}");
        let report = parse_sheet(&source);
        let [CssRule::Style(rule)] = report.syntax().rules() else {
            panic!("expected retained style rule for {unit}");
        };
        assert_eq!(
            property_names(rule.declarations()),
            ["color", "--kept", "height"],
            "{unit}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one declaration diagnostic for {unit}");
        };
        let unit_start = source.find(unit).expect("unit");
        let responsible = unit_start + unit.find(responsible_text).expect("responsible token");
        assert_drop(
            &source,
            diagnostic,
            unit,
            code,
            CssRecoveryAction::DropDeclaration,
            responsible,
        );
    }
}

#[test]
fn block_item_recovery_invalid_custom_annotation_retains_ordered_siblings_and_context() {
    let unit = "--Accent: ready !oops;";
    let source = format!(".x {{ color: red; {unit} width: 2px; }}");
    let report = parse_sheet(&source);

    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("expected retained style rule");
    };
    assert_eq!(property_names(rule.declarations()), ["color", "width"]);

    let [diagnostic] = report.diagnostics() else {
        panic!("expected exactly one custom declaration diagnostic");
    };
    assert_drop(
        &source,
        diagnostic,
        unit,
        CssErrorCode::InvalidDeclarationAnnotation,
        CssRecoveryAction::DropDeclaration,
        source.find('!').expect("first responsible bang"),
    );
    let ErrorKind::InvalidDeclarationAnnotation(detail) = diagnostic.error().kind() else {
        panic!("expected invalid declaration annotation detail");
    };
    let CssDeclarationContextRef::CustomProperty(name) = detail.context() else {
        panic!("expected custom-property declaration context");
    };
    assert_eq!(name.as_str(), "--Accent");
    assert_eq!(detail.encountered().kind(), CssTokenKind::Delim);
    assert_eq!(detail.encountered().authored(), "!");
}

#[test]
fn block_item_recovery_balanced_semicolons_and_block_end_bound_one_declaration_each() {
    for unit in ["mystery: fn({x; y});", "mystery: fn({x; y})"] {
        let source = if unit.ends_with(';') {
            format!(".x {{ width: 1px; {unit} height: 2px; }}")
        } else {
            format!(".x {{ width: 1px; {unit}}}")
        };
        let report = parse_sheet(&source);
        let [CssRule::Style(rule)] = report.syntax().rules() else {
            panic!("expected retained style rule");
        };
        let expected = if unit.ends_with(';') {
            vec!["width", "height"]
        } else {
            vec!["width"]
        };
        assert_eq!(property_names(rule.declarations()), expected);
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one declaration diagnostic");
        };
        assert_drop(
            &source,
            diagnostic,
            unit,
            CssErrorCode::UnknownProperty,
            CssRecoveryAction::DropDeclaration,
            source.find("mystery").unwrap(),
        );
    }
}

#[test]
fn block_item_recovery_repeated_declaration_failures_progress_to_later_valid_item() {
    let source = ".x { color: red; first: fn(a;b); second: nope; width: 2px !bad; height: 3px; }";
    let report = parse_sheet(source);
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("expected retained style rule");
    };
    assert_eq!(property_names(rule.declarations()), ["color", "height"]);
    assert_eq!(report.diagnostics().len(), 3);
    for (diagnostic, unit, code, responsible) in [
        (
            &report.diagnostics()[0],
            "first: fn(a;b);",
            CssErrorCode::UnknownProperty,
            source.find("first").unwrap(),
        ),
        (
            &report.diagnostics()[1],
            "second: nope;",
            CssErrorCode::UnknownProperty,
            source.find("second").unwrap(),
        ),
        (
            &report.diagnostics()[2],
            "width: 2px !bad;",
            CssErrorCode::InvalidDeclarationAnnotation,
            source.find('!').unwrap(),
        ),
    ] {
        assert_drop(
            source,
            diagnostic,
            unit,
            code,
            CssRecoveryAction::DropDeclaration,
            responsible,
        );
    }
}

#[test]
fn block_item_recovery_all_invalid_declarations_retain_empty_ordinary_style_in_order() {
    let unit = "mystery: 1;";
    let source = format!(".before {{ color: red; }} .x {{ {unit} }} .after {{ height: 2px; }}");
    let report = parse_sheet(&source);

    let [
        CssRule::Style(before),
        CssRule::Style(empty),
        CssRule::Style(after),
    ] = report.syntax().rules()
    else {
        panic!("expected the empty owning style between its retained siblings");
    };
    assert_eq!(before.selector(), &CssSelector::Class("before".to_owned()));
    assert_eq!(property_names(before.declarations()), ["color"]);
    assert_eq!(empty.selector(), &CssSelector::Class("x".to_owned()));
    assert!(empty.declarations().is_empty());
    assert_eq!(after.selector(), &CssSelector::Class("after".to_owned()));
    assert_eq!(property_names(after.declarations()), ["height"]);

    let [diagnostic] = report.diagnostics() else {
        panic!("expected exactly one declaration diagnostic");
    };
    assert_drop(
        &source,
        diagnostic,
        unit,
        CssErrorCode::UnknownProperty,
        CssRecoveryAction::DropDeclaration,
        source.find("mystery").unwrap(),
    );
    let ErrorKind::UnknownProperty(detail) = diagnostic.error().kind() else {
        panic!("expected unknown-property detail");
    };
    assert_eq!(detail.name().as_str(), "mystery");
}

#[test]
fn block_item_recovery_all_invalid_declarations_retain_empty_nested_style_in_order() {
    let unit = "mystery: 1;";
    let source = format!(
        ".host {{ color: red; & .child {{ {unit} }} opacity: 1; }} .after {{ height: 2px; }}"
    );
    let report = parse_sheet(&source);

    let [
        CssRule::Style(before),
        CssRule::Style(empty),
        CssRule::Style(after),
        CssRule::Style(sibling),
    ] = report.syntax().rules()
    else {
        panic!("expected the empty nested style between retained parent segments and sibling");
    };
    assert_eq!(before.selector(), &CssSelector::Class("host".to_owned()));
    assert_eq!(property_names(before.declarations()), ["color"]);
    let CssSelector::Complex(empty_selector) = empty.selector() else {
        panic!("expected flattened descendant selector for the empty nested style");
    };
    assert_eq!(empty_selector.first().classes(), &["host".to_owned()]);
    let [child] = empty_selector.rest() else {
        panic!("expected exactly one descendant selector part");
    };
    assert_eq!(child.combinator(), CssSelectorCombinator::Descendant);
    assert_eq!(child.selector().classes(), &["child".to_owned()]);
    assert!(empty.declarations().is_empty());
    assert_eq!(after.selector(), &CssSelector::Class("host".to_owned()));
    assert_eq!(property_names(after.declarations()), ["opacity"]);
    assert_eq!(sibling.selector(), &CssSelector::Class("after".to_owned()));
    assert_eq!(property_names(sibling.declarations()), ["height"]);

    let [diagnostic] = report.diagnostics() else {
        panic!("expected exactly one nested declaration diagnostic");
    };
    assert_drop(
        &source,
        diagnostic,
        unit,
        CssErrorCode::UnknownProperty,
        CssRecoveryAction::DropDeclaration,
        source.find("mystery").unwrap(),
    );
    let ErrorKind::UnknownProperty(detail) = diagnostic.error().kind() else {
        panic!("expected nested unknown-property detail");
    };
    assert_eq!(detail.name().as_str(), "mystery");
}

#[test]
fn block_item_recovery_nested_and_scoped_style_lists_own_declaration_recovery() {
    let nested_source =
        ".host { color: red; & .child { width: 1px; bad: fn(a;b); height: 2px; } opacity: 1; }";
    let nested = parse_sheet(nested_source);
    let [nested_diagnostic] = nested.diagnostics() else {
        panic!("expected exactly one nested declaration diagnostic");
    };
    assert_drop(
        nested_source,
        nested_diagnostic,
        "bad: fn(a;b);",
        CssErrorCode::UnknownProperty,
        CssRecoveryAction::DropDeclaration,
        nested_source.find("bad").unwrap(),
    );
    let ErrorKind::UnknownProperty(nested_detail) = nested_diagnostic.error().kind() else {
        panic!("expected nested unknown-property detail");
    };
    assert_eq!(nested_detail.name().as_str(), "bad");
    let nested_declarations: Vec<Vec<&str>> = nested
        .syntax()
        .rules()
        .iter()
        .filter_map(|rule| match rule {
            CssRule::Style(rule) => Some(property_names(rule.declarations())),
            _ => None,
        })
        .collect();
    assert_eq!(
        nested_declarations,
        [vec!["color"], vec!["width", "height"], vec!["opacity"]]
    );

    let scoped_source = "@scope { :scope { color: red; bad: x; width: 2px; } }";
    let scoped = parse_sheet(scoped_source);
    let [CssRule::Scope(scope)] = scoped.syntax().rules() else {
        panic!("expected retained scope rule");
    };
    let [CssScopedRule::Style(style)] = scope.rules().rules() else {
        panic!("expected retained scoped style rule");
    };
    assert_eq!(property_names(style.declarations()), ["color", "width"]);
    let [scoped_diagnostic] = scoped.diagnostics() else {
        panic!("expected exactly one scoped declaration diagnostic");
    };
    assert_drop(
        scoped_source,
        scoped_diagnostic,
        "bad: x;",
        CssErrorCode::UnknownProperty,
        CssRecoveryAction::DropDeclaration,
        scoped_source.find("bad").unwrap(),
    );
    let ErrorKind::UnknownProperty(scoped_detail) = scoped_diagnostic.error().kind() else {
        panic!("expected scoped unknown-property detail");
    };
    assert_eq!(scoped_detail.name().as_str(), "bad");
}

#[test]
fn block_item_recovery_group_rule_merges_child_diagnostic_without_parent_drop() {
    let unit = "bad: fn({x; y});";
    let source = format!(
        ".before {{ color: red; }} @media screen {{ .inside {{ width: 1px; {unit} height: 2px; }} }} .after {{ opacity: 1; }}"
    );
    let report = parse_sheet(&source);
    assert_eq!(report.syntax().rules().len(), 3);
    let CssRule::Media(media) = &report.syntax().rules()[1] else {
        panic!("expected retained media parent");
    };
    let [CssRule::Style(inside)] = media.rules() else {
        panic!("expected retained media child");
    };
    assert_eq!(property_names(inside.declarations()), ["width", "height"]);
    let [diagnostic] = report.diagnostics() else {
        panic!("one child declaration diagnostic without parent drop");
    };
    assert_drop(
        &source,
        diagnostic,
        unit,
        CssErrorCode::UnknownProperty,
        CssRecoveryAction::DropDeclaration,
        source.find("bad").unwrap(),
    );
}

#[test]
fn block_item_recovery_supports_merges_child_diagnostic_without_parent_drop() {
    let unit = "bad: fn({x; y});";
    let source =
        format!("@supports (display: grid) {{ .inside {{ width: 1px; {unit} height: 2px; }} }}");
    let report = parse_sheet(&source);
    let [CssRule::Supports(supports)] = report.syntax().rules() else {
        panic!("expected retained supports parent");
    };
    let [CssRule::Style(inside)] = supports.rules() else {
        panic!("expected retained supports child");
    };
    assert_eq!(property_names(inside.declarations()), ["width", "height"]);
    let [diagnostic] = report.diagnostics() else {
        panic!("one child declaration diagnostic without parent drop");
    };
    assert_drop(
        &source,
        diagnostic,
        unit,
        CssErrorCode::UnknownProperty,
        CssRecoveryAction::DropDeclaration,
        source.find("bad").unwrap(),
    );
}

#[test]
fn block_item_recovery_font_face_drops_bad_and_retains_duplicate_optional_descriptors() {
    let source = "@font-face { font-family: Inter; mystery: fn(a;b); src: url(i); font-display: swap; font-display: block; unicode-range: U+0-7F; }";
    let report = parse_sheet(source);
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("expected retained font face");
    };
    let descriptors = rule.descriptors();
    assert_eq!(descriptors.font_family().value().as_str(), "Inter");
    assert_eq!(
        descriptors.src().position().byte_offset().value(),
        source.find("src").unwrap()
    );
    assert_eq!(
        descriptors.font_display().unwrap().value(),
        &surgeist_css::CssFontDisplay::Block
    );
    assert!(descriptors.unicode_range().is_some());
    assert_eq!(report.diagnostics().len(), 1);
    assert_drop(
        source,
        &report.diagnostics()[0],
        "mystery: fn(a;b);",
        CssErrorCode::UnknownDescriptor,
        CssRecoveryAction::DropDescriptor,
        source.find("mystery").unwrap(),
    );
    assert_eq!(
        descriptors
            .occurrences()
            .filter(|descriptor| matches!(
                descriptor,
                surgeist_css::CssFontFaceDescriptorRef::FontDisplay(_)
            ))
            .count(),
        2
    );
}

#[test]
fn block_item_recovery_font_face_required_loss_emits_child_before_parent_drop() {
    let source =
        "@font-face { font-family: Inter; src: nope; font-display: swap; } .after { color: blue; }";
    let report = parse_sheet(source);
    let [CssRule::Style(after)] = report.syntax().rules() else {
        panic!("unrepresentable font face must not survive and later rule must remain");
    };
    assert_eq!(property_names(after.declarations()), ["color"]);
    assert_eq!(report.diagnostics().len(), 2);
    assert_drop(
        source,
        &report.diagnostics()[0],
        "src: nope;",
        CssErrorCode::InvalidDescriptorValue,
        CssRecoveryAction::DropDescriptor,
        source.find("nope").unwrap(),
    );
    assert_eq!(
        report.diagnostics()[1].error().code(),
        CssErrorCode::InvalidAtRuleBody
    );
    assert_eq!(
        report.diagnostics()[1].action(),
        CssRecoveryAction::DropAtRule
    );
    assert_eq!(
        report.diagnostics()[1].span().start().byte_offset().value(),
        0
    );
    assert_eq!(
        report.diagnostics()[1].span().end().byte_offset().value(),
        source.find(" .after").unwrap()
    );
}

#[test]
fn block_item_recovery_each_font_face_descriptor_value_failure_has_exact_scope() {
    let cases = [
        ("font-family: ;", "font-family", true),
        ("src: nope;", "src", true),
        ("font-weight: nope;", "font-weight", false),
        ("font-style: nope;", "font-style", false),
        ("font-stretch: nope;", "font-stretch", false),
        ("font-display: nope;", "font-display", false),
        ("unicode-range: nope;", "unicode-range", false),
    ];

    for (unit, name, required) in cases {
        let mut required_descriptors = "font-family: Inter; src: url(i);".to_owned();
        if name == "font-family" {
            required_descriptors = "src: url(i);".to_owned();
        } else if name == "src" {
            required_descriptors = "font-family: Inter;".to_owned();
        }
        let source = format!(
            "@font-face {{ {required_descriptors} {unit} font-display: swap; }} .after {{ color: blue; }}"
        );
        let report = parse_sheet(&source);
        let child = &report.diagnostics()[0];
        let responsible = if unit.ends_with(": ;") {
            source.find(unit).unwrap() + unit.find(' ').unwrap()
        } else {
            source.find("nope").unwrap()
        };
        assert_drop(
            &source,
            child,
            unit,
            CssErrorCode::InvalidDescriptorValue,
            CssRecoveryAction::DropDescriptor,
            responsible,
        );

        if required {
            assert_eq!(report.diagnostics().len(), 2, "{name}");
            assert_eq!(
                report.diagnostics()[1].error().code(),
                CssErrorCode::InvalidAtRuleBody,
                "{name}"
            );
            assert_eq!(
                report.diagnostics()[1].action(),
                CssRecoveryAction::DropAtRule,
                "{name}"
            );
            assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
        } else {
            assert_eq!(report.diagnostics().len(), 1, "{name}");
            assert!(matches!(
                report.syntax().rules(),
                [CssRule::FontFace(_), CssRule::Style(_)]
            ));
        }
    }
}

#[test]
fn block_item_recovery_descriptor_annotation_and_block_end_retain_parent() {
    let unit = "font-display: swap !important";
    let source = format!("@font-face {{ font-family: Inter; src: url(i); {unit}}}");
    let report = parse_sheet(&source);
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("optional descriptor loss must retain font face");
    };
    assert!(rule.descriptors().font_display().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("one descriptor annotation diagnostic");
    };
    assert_drop(
        &source,
        diagnostic,
        unit,
        CssErrorCode::InvalidDeclarationAnnotation,
        CssRecoveryAction::DropDescriptor,
        source.find('!').unwrap(),
    );
}

#[test]
fn block_item_recovery_repeated_descriptor_failures_progress_to_required_siblings() {
    let source =
        "@font-face { one: fn(a;b); font-display: nope; two: x; font-family: Inter; src: url(i) }";
    let report = parse_sheet(source);
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("later required descriptors must retain font face");
    };
    assert_eq!(rule.descriptors().font_family().value().as_str(), "Inter");
    assert_eq!(report.diagnostics().len(), 3);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.error().code())
            .collect::<Vec<_>>(),
        [
            CssErrorCode::UnknownDescriptor,
            CssErrorCode::InvalidDescriptorValue,
            CssErrorCode::UnknownDescriptor,
        ]
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDescriptor)
    );
}
