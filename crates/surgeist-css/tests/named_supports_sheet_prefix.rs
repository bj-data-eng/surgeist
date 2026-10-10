#![forbid(unsafe_code)]
//! Conditional 5 §8 permits named supports definitions before imports and
//! namespaces. Definitions preserve the current prelude phase; they do not
//! reopen imports after a namespace or imports/namespaces after a body rule.
//! https://drafts.csswg.org/css-conditional-5/#supports-condition-rule

use surgeist_css::{
    CssErrorCode, CssNamespaceContext, CssNamespacePrefix, CssRecoveryAction, CssRule,
    CssRuleConstructionErrorKind, CssSheet, CssValueOrigin, parse_sheet,
};

fn clean_rules(source: &str) -> Vec<CssRule> {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().rules().to_vec()
}

fn named_origins(sheet: &CssSheet, source: &str, names: &[&str]) {
    let definitions: Vec<_> = sheet
        .rules()
        .iter()
        .filter_map(|rule| match rule {
            CssRule::SupportsCondition(definition) => Some(definition),
            _ => None,
        })
        .collect();
    assert_eq!(definitions.len(), names.len());
    for (definition, name) in definitions.into_iter().zip(names) {
        assert_eq!(definition.name().as_str(), *name);
        let offset = source.find(&format!("@supports-condition {name}")).unwrap();
        assert_eq!(definition.position().unwrap().byte_offset().value(), offset);
        let CssValueOrigin::Parsed(origin) = definition.origin() else {
            panic!("the original definition occurrence must survive assembly");
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), offset);
    }
}

#[test]
fn parser_keeps_interleaved_definitions_imports_namespaces_and_real_origins() {
    let source = concat!(
        "/* 😀 */\n",
        "@supports-condition --first{}",
        "@import 'first.css';",
        "@supports-condition --second{}",
        "@import 'second.css' supports(--second);",
        "@namespace n 'urn:n';",
        "@supports-condition --third{}",
        "@namespace m 'urn:m';",
        "n|item, m|item {}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::SupportsCondition(_),
            CssRule::Import(_),
            CssRule::SupportsCondition(_),
            CssRule::Import(_),
            CssRule::Namespace(_),
            CssRule::SupportsCondition(_),
            CssRule::Namespace(_),
            CssRule::Style(_),
        ]
    ));
    named_origins(report.syntax(), source, &["--first", "--second", "--third"]);
    let bindings = CssNamespaceContext::from_sheet(report.syntax());
    for (prefix, namespace) in [("n", "urn:n"), ("m", "urn:m")] {
        assert_eq!(
            bindings
                .named_namespace(&CssNamespacePrefix::try_new(prefix).unwrap())
                .unwrap()
                .as_str(),
            namespace
        );
    }
}

#[test]
fn checked_sheet_accepts_named_definition_before_import_and_namespace() {
    let source = concat!(
        "/* 😀 */\n@supports-condition --theme{}",
        "@import 'theme.css' supports(--theme);",
        "@namespace n 'urn:n';n|item {}",
    );
    let owned_input = source.to_owned();
    let rules = clean_rules(&owned_input);
    drop(owned_input);
    let original = rules.clone();
    let sheet = CssSheet::try_from_rules(rules).expect("Conditional 5 named prefix is legal");
    assert_eq!(sheet.rules(), original);
    named_origins(&sheet, source, &["--theme"]);
}

#[test]
fn checked_sheet_keeps_named_definitions_transparent_between_imports_and_namespaces() {
    let source = concat!(
        "@import 'first.css';@supports-condition --imports{}",
        "@import 'second.css';@namespace n 'urn:n';",
        "@supports-condition --namespaces{}@namespace m 'urn:m';",
        "n|item, m|item {}",
    );
    let rules = clean_rules(source);
    let original = rules.clone();
    let sheet =
        CssSheet::try_from_rules(rules).expect("named definitions preserve both prefix phases");
    assert_eq!(sheet.rules(), original);
    named_origins(&sheet, source, &["--imports", "--namespaces"]);
    let bindings = CssNamespaceContext::from_sheet(&sheet);
    assert_eq!(
        bindings
            .named_namespace(&CssNamespacePrefix::try_new("m").unwrap())
            .unwrap()
            .as_str(),
        "urn:m"
    );
}

#[test]
fn named_definition_does_not_reopen_imports_after_a_namespace() {
    let source = "@namespace n 'urn:n';@supports-condition --x{}@import 'late.css';n|item {}";
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Namespace(_),
            CssRule::SupportsCondition(_),
            CssRule::Style(_),
        ]
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("one dropped late import");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );

    let mut rules = clean_rules("@namespace n 'urn:n';@supports-condition --x{}");
    let late = clean_rules("/* late */@import 'late.css';");
    let CssRule::Import(import) = &late[0] else {
        panic!("one import");
    };
    let position = import.position();
    rules.extend(late);
    let error = CssSheet::try_from_rules(rules).unwrap_err();
    assert_eq!(
        error.kind(),
        CssRuleConstructionErrorKind::InvalidPreludeOrder
    );
    assert_eq!(error.path(), &[2]);
    assert_eq!(error.position(), position);
}

#[test]
fn named_definition_does_not_reopen_imports_or_namespaces_after_ordinary_body() {
    for late in ["@import 'late.css';", "@namespace late 'urn:late';"] {
        let prefix = ".body{}@supports-condition --x{}";
        let source = format!("{prefix}{late}.after{{}}");
        let report = parse_sheet(&source);
        assert!(matches!(
            report.syntax().rules(),
            [
                CssRule::Style(_),
                CssRule::SupportsCondition(_),
                CssRule::Style(_),
            ]
        ));
        let [diagnostic] = report.diagnostics() else {
            panic!("one dropped late prefix rule");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePlacement
        );
        let mut rules = clean_rules(prefix);
        rules.extend(clean_rules(late));
        let error = CssSheet::try_from_rules(rules).unwrap_err();
        assert_eq!(
            error.kind(),
            CssRuleConstructionErrorKind::InvalidPreludeOrder
        );
        assert_eq!(error.path(), &[2]);
    }
}

#[test]
fn named_prefix_preserves_namespace_mismatch_validation() {
    let mut rules = clean_rules("@supports-condition --x{}@namespace n 'urn:n';");
    let body = clean_rules("@namespace n 'urn:other';n|item {}");
    rules.push(body[1].clone());
    let error = CssSheet::try_from_rules(rules).unwrap_err();
    assert_eq!(
        error.kind(),
        CssRuleConstructionErrorKind::NamespaceMismatch
    );
    assert_eq!(error.path(), &[2]);
}
