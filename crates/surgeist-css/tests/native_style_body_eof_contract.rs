#![forbid(unsafe_code)]
//! Native style-body missing-input and recovery contracts.
//! Syntax3 CRD 20211224 §§5.4.2,5.4.8–9 and 9; selected Nesting1
//! WD20260122 §§3.3–3.4; original-source error and clean-report contracts.
use surgeist_css::*;

fn span(actual: CssSourceSpan, start: usize, end: usize) {
    assert_eq!(actual.start().byte_offset().value(), start);
    assert_eq!(actual.end().byte_offset().value(), end);
}

fn position(diagnostic: &CssRecoveryDiagnostic, byte: usize, line: u32, column: u32) {
    let actual = diagnostic.error().position();
    assert_eq!(actual.byte_offset().value(), byte);
    assert_eq!(actual.line().value(), line);
    assert_eq!(actual.column().value(), column);
}

fn style<'a>(rule: &'a CssRule, name: &str) -> &'a CssStyleRule {
    let CssRule::Style(style) = rule else {
        panic!("retained style {name}")
    };
    let [selector] = style.selectors().selectors() else {
        panic!("one class selector")
    };
    assert_eq!(selector.selector(), &CssSelector::Class(name.to_owned()));
    style
}

fn declaration(
    list: &CssDeclarationList,
    source: &str,
    name: CssKnownProperty,
    value: &str,
    at: usize,
    value_start: usize,
    value_end: usize,
) {
    let [declaration] = list.as_slice() else {
        panic!("one original declaration")
    };
    assert_eq!(declaration.property_name(), CssPropertyNameRef::Known(name));
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        value
    );
    assert_eq!(declaration.position().unwrap().byte_offset().value(), at);
    let original = declaration.parsed_value().unwrap();
    assert_eq!(original.source().as_str(), source);
    span(original.span(), value_start, value_end);
}

fn media_eof(diagnostic: &CssRecoveryDiagnostic, start: usize, end: usize, line: u32, column: u32) {
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert!(
        matches!(diagnostic.error().kind(), ErrorKind::UnexpectedEnd(_)),
        "a bounded production EOF supplies no authored semicolon: {diagnostic:?}"
    );
    position(diagnostic, end, line, column);
    span(diagnostic.span(), start, end);
}

fn closure(diagnostic: &CssRecoveryDiagnostic, eof: usize) {
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert!(matches!(
        diagnostic.error().kind(),
        ErrorKind::UnexpectedEnd(_)
    ));
    position(diagnostic, eof, 0, eof as u32);
    span(diagnostic.span(), eof, eof);
}

fn strict(source: &str, report: &CssParseReport<CssSheet>) {
    assert!(!report.is_clean());
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert_eq!(
        report
            .clone()
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

fn scoped_parent<'a>(report: &'a CssParseReport<CssSheet>, source: &str) -> &'a CssScopeRule {
    let parent = style(&report.syntax().rules()[0], "x");
    declaration(
        parent.declarations(),
        source,
        CssKnownProperty::Color,
        "red",
        3,
        9,
        12,
    );
    let [CssRule::Scope(scope)] = parent.rules() else {
        panic!("retained scope, no media child")
    };
    let [CssScopedRule::NestedDeclarations(run)] = scope.rules().rules() else {
        panic!("only the original scoped declaration run survives")
    };
    declaration(
        run.declarations(),
        source,
        CssKnownProperty::Width,
        "1px",
        20,
        26,
        29,
    );
    scope
}

#[test]
fn native_style_media_ends_at_parent_bound_and_keeps_original_sibling() {
    for (source, color, value_start, value_end, unit_start, end, line, column, after) in [
        (
            ".x{color:red;@media screen}.after{height:1px}",
            3,
            9,
            12,
            13,
            26,
            0,
            26,
            27,
        ),
        (
            "/*😀*/\r\n.x{color:red;@media screen}.after{height:1px}",
            13,
            19,
            22,
            23,
            36,
            1,
            26,
            37,
        ),
    ] {
        let report = parse_sheet(source);
        let [parent, sibling] = report.syntax().rules() else {
            panic!("parent and later sibling")
        };
        let parent = style(parent, "x");
        declaration(
            parent.declarations(),
            source,
            CssKnownProperty::Color,
            "red",
            color,
            value_start,
            value_end,
        );
        assert!(parent.rules().is_empty());
        let sibling = style(sibling, "after");
        assert_eq!(sibling.position().byte_offset().value(), after);
        declaration(
            sibling.declarations(),
            source,
            CssKnownProperty::Height,
            "1px",
            after + 7,
            after + 14,
            after + 17,
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("only missing nested media body")
        };
        media_eof(diagnostic, unit_start, end, line, column);
        assert!(end < source.len());
        strict(source, &report);
    }
    let source = ".x{color:red;@media screen}";
    let report = parse_sheet(source);
    let [parent] = report.syntax().rules() else {
        panic!("retained parent")
    };
    declaration(
        style(parent, "x").declarations(),
        source,
        CssKnownProperty::Color,
        "red",
        3,
        9,
        12,
    );
    assert!(style(parent, "x").rules().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("bounded missing body")
    };
    media_eof(diagnostic, 13, 26, 0, 26);
    strict(source, &report);
}

#[test]
fn native_style_media_at_whole_source_eof_keeps_parent_and_separate_closure_fault() {
    let source = ".x{color:red;@media screen";
    let report = parse_sheet(source);
    let [parent] = report.syntax().rules() else {
        panic!("retained implicitly closed parent")
    };
    let parent = style(parent, "x");
    declaration(
        parent.declarations(),
        source,
        CssKnownProperty::Color,
        "red",
        3,
        9,
        12,
    );
    assert!(parent.rules().is_empty());
    let [dropped, implicit] = report.diagnostics() else {
        panic!("media rejection and one parent closure")
    };
    media_eof(dropped, 13, 26, 0, 26);
    closure(implicit, 26);
    strict(source, &report);
}

#[test]
fn scoped_style_ancestor_media_uses_scope_bound_and_preserves_both_declaration_owners() {
    let source = ".x{color:red;@scope{width:1px;@media screen}}.after{height:1px}";
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 2);
    let scope = scoped_parent(&report, source);
    assert!(scope.root().is_none() && scope.limit().is_none());
    let sibling = style(&report.syntax().rules()[1], "after");
    declaration(
        sibling.declarations(),
        source,
        CssKnownProperty::Height,
        "1px",
        52,
        59,
        62,
    );
    assert_eq!(sibling.position().byte_offset().value(), 45);
    let [diagnostic] = report.diagnostics() else {
        panic!("only dropped scoped media")
    };
    media_eof(diagnostic, 30, 43, 0, 43);
    strict(source, &report);
}

#[test]
fn scoped_style_ancestor_whole_eof_retains_scope_and_parent_closure_faults() {
    let source = ".x{color:red;@scope{width:1px;@media screen";
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 1);
    scoped_parent(&report, source);
    let [dropped, scope_closure, style_closure] = report.diagnostics() else {
        panic!("media rejection plus two retained actual openings")
    };
    media_eof(dropped, 30, 43, 0, 43);
    closure(scope_closure, 43);
    closure(style_closure, 43);
    strict(source, &report);
}

#[test]
fn actual_media_semicolon_has_missing_body_contract_and_local_following_runs() {
    for (source, start, end, height, value_start, value_end, scoped) in [
        (
            ".x{color:red;@media screen;height:1px}",
            13,
            27,
            27,
            34,
            37,
            false,
        ),
        (
            ".x{color:red;@scope{width:1px;@media screen;height:1px}}",
            30,
            44,
            44,
            51,
            54,
            true,
        ),
    ] {
        let report = parse_sheet(source);
        let [parent] = report.syntax().rules() else {
            panic!("retained parent")
        };
        let parent = style(parent, "x");
        declaration(
            parent.declarations(),
            source,
            CssKnownProperty::Color,
            "red",
            3,
            9,
            12,
        );
        if scoped {
            let [CssRule::Scope(scope)] = parent.rules() else {
                panic!("retained scope")
            };
            let [
                CssScopedRule::NestedDeclarations(before),
                CssScopedRule::NestedDeclarations(after),
            ] = scope.rules().rules()
            else {
                panic!("scope runs before and after dropped media")
            };
            declaration(
                before.declarations(),
                source,
                CssKnownProperty::Width,
                "1px",
                20,
                26,
                29,
            );
            declaration(
                after.declarations(),
                source,
                CssKnownProperty::Height,
                "1px",
                height,
                value_start,
                value_end,
            );
        } else {
            let [CssRule::NestedDeclarations(after)] = parent.rules() else {
                panic!("following declaration run")
            };
            declaration(
                after.declarations(),
                source,
                CssKnownProperty::Height,
                "1px",
                height,
                value_start,
                value_end,
            );
        }
        let [diagnostic] = report.diagnostics() else {
            panic!("one consumed-semicolon missing-body failure")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
        let ErrorKind::InvalidAtRuleBody(detail) = diagnostic.error().kind() else {
            panic!("media body owner")
        };
        assert_eq!(detail.name().as_str(), "media");
        assert_eq!(detail.production().as_str(), "baseline.rule.media");
        assert!(detail.encountered().is_none());
        position(diagnostic, end, 0, end as u32);
        span(diagnostic.span(), start, end);
        strict(source, &report);
    }
}

#[test]
fn legal_layer_statement_eof_is_retained_at_native_style_and_scoped_bounds() {
    for (source, start, end, scoped) in [
        (
            ".x{color:red;@layer theme}.after{height:1px}",
            13,
            25,
            false,
        ),
        (
            ".x{color:red;@scope{width:1px;@layer theme}}.after{height:1px}",
            30,
            42,
            true,
        ),
    ] {
        let report = parse_sheet(source);
        assert_eq!(report.syntax().rules().len(), 2);
        let parent = style(&report.syntax().rules()[0], "x");
        declaration(
            parent.declarations(),
            source,
            CssKnownProperty::Color,
            "red",
            3,
            9,
            12,
        );
        if scoped {
            let [CssRule::Scope(scope)] = parent.rules() else {
                panic!("scope")
            };
            let [
                CssScopedRule::NestedDeclarations(run),
                CssScopedRule::LayerStatement(layer),
            ] = scope.rules().rules()
            else {
                panic!("scoped declaration run then admitted layer statement")
            };
            declaration(
                run.declarations(),
                source,
                CssKnownProperty::Width,
                "1px",
                20,
                26,
                29,
            );
            assert_eq!(layer.names().names()[0].components(), &["theme".to_owned()]);
        } else {
            let [CssRule::LayerStatement(layer)] = parent.rules() else {
                panic!("admitted layer statement")
            };
            assert_eq!(layer.names().names()[0].components(), &["theme".to_owned()]);
        }
        assert_eq!(
            style(&report.syntax().rules()[1], "after")
                .position()
                .byte_offset()
                .value(),
            end + if scoped { 2 } else { 1 }
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one retained statement EOF")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainNonconformingRule
        );
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        let ErrorKind::UnexpectedEnd(detail) = diagnostic.error().kind() else {
            panic!("statement EOF")
        };
        assert_eq!(
            detail.expectation().as_str(),
            "a semicolon or block terminating an at-rule"
        );
        position(diagnostic, end, 0, end as u32);
        span(diagnostic.span(), start, end);
        strict(source, &report);
    }
    for source in [
        ".x{color:red;@layer theme;}",
        ".x{color:red;@scope{width:1px;@layer theme;}}",
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{report:?}");
        assert_eq!(validate_sheet(source).unwrap(), *report.syntax());
    }
}

#[test]
fn invalid_layer_prelude_precedes_missing_body_in_both_native_contexts() {
    for (source, start, end, responsible) in [
        (".x{color:red;@layer 2px}.after{height:1px}", 13, 23, 20),
        (
            ".x{color:red;@scope{width:1px;@layer 2px}}.after{height:1px}",
            30,
            40,
            37,
        ),
    ] {
        let report = parse_sheet(source);
        assert_eq!(report.syntax().rules().len(), 2);
        let parent = style(&report.syntax().rules()[0], "x");
        declaration(
            parent.declarations(),
            source,
            CssKnownProperty::Color,
            "red",
            3,
            9,
            12,
        );
        if start == 13 {
            assert!(parent.rules().is_empty());
        } else {
            scoped_parent(&report, source);
        }
        let [diagnostic] = report.diagnostics() else {
            panic!("only owning prelude failure, no synthetic body failure")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePrelude
        );
        let ErrorKind::InvalidAtRulePrelude(detail) = diagnostic.error().kind() else {
            panic!("layer prelude")
        };
        assert_eq!(detail.name().as_str(), "layer");
        assert_eq!(detail.production().as_str(), "baseline.rule.layer-block");
        let token = detail
            .encountered()
            .expect("actual invalid layer-name dimension");
        assert_eq!(token.kind(), CssTokenKind::Dimension);
        assert_eq!(token.authored(), "2px");
        position(diagnostic, responsible, 0, responsible as u32);
        span(diagnostic.span(), start, end);
        strict(source, &report);
    }
}

#[test]
fn contextual_when_body_keeps_owned_missing_token_category_at_native_eof() {
    let source = ".x{color:red;@when media(width)}.after{height:1px}";
    let report = parse_sheet(source);
    let [parent, sibling] = report.syntax().rules() else {
        panic!("retained sibling styles")
    };
    let parent = style(parent, "x");
    declaration(
        parent.declarations(),
        source,
        CssKnownProperty::Color,
        "red",
        3,
        9,
        12,
    );
    assert!(parent.rules().is_empty());
    assert_eq!(style(sibling, "after").position().byte_offset().value(), 32);
    let [diagnostic] = report.diagnostics() else {
        panic!("one missing when body")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
    let ErrorKind::InvalidAtRuleBody(detail) = diagnostic.error().kind() else {
        panic!("when body owner")
    };
    assert_eq!(detail.name().as_str(), "when");
    assert_eq!(detail.production().as_str(), "ext.rule.when");
    assert!(detail.encountered().is_none());
    position(diagnostic, 31, 0, 31);
    span(diagnostic.span(), 13, 31);
    strict(source, &report);
}

#[test]
fn isolated_real_style_block_preserves_native_bound_and_original_source_origin() {
    let context = CssNamespaceContext::default();
    for (source, end, implicit) in [
        ("{color:red;@media screen}", 24, false),
        ("{color:red;@media screen", 24, true),
    ] {
        let report = parse_style_block(source, &context);
        let block = report
            .syntax()
            .as_ref()
            .expect("retained genuine style block");
        declaration(
            block.declarations(),
            source,
            CssKnownProperty::Color,
            "red",
            1,
            7,
            10,
        );
        assert!(block.rules().is_empty());
        assert_eq!(block.origin().source().as_str(), source);
        span(block.origin().span(), 0, if implicit { 24 } else { 25 });
        media_eof(&report.diagnostics()[0], 11, end, 0, end as u32);
        if implicit {
            let [_, fault] = report.diagnostics() else {
                panic!("one retained block closure")
            };
            closure(fault, 24);
        } else {
            assert_eq!(report.diagnostics().len(), 1);
        }
        assert!(!report.is_clean());
        assert_eq!(
            report
                .clone()
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn complete_media_blocks_are_clean_in_both_native_owning_contexts() {
    let ordinary = ".x{color:red;@media screen{width:1px}}";
    let report = parse_sheet(ordinary);
    assert!(report.is_clean(), "{report:?}");
    let [parent] = report.syntax().rules() else {
        panic!("one parent style")
    };
    let parent = style(parent, "x");
    declaration(
        parent.declarations(),
        ordinary,
        CssKnownProperty::Color,
        "red",
        3,
        9,
        12,
    );
    let [CssRule::Media(media)] = parent.rules() else {
        panic!("retained complete media")
    };
    let [CssRule::NestedDeclarations(run)] = media.rules() else {
        panic!("direct media declaration run")
    };
    declaration(
        run.declarations(),
        ordinary,
        CssKnownProperty::Width,
        "1px",
        27,
        33,
        36,
    );
    assert_eq!(validate_sheet(ordinary).unwrap(), *report.syntax());

    let scoped = ".x{color:red;@scope{width:1px;@media screen{height:2px}}}";
    let report = parse_sheet(scoped);
    assert!(report.is_clean(), "{report:?}");
    let [parent] = report.syntax().rules() else {
        panic!("one scoped parent style")
    };
    let parent = style(parent, "x");
    declaration(
        parent.declarations(),
        scoped,
        CssKnownProperty::Color,
        "red",
        3,
        9,
        12,
    );
    let [CssRule::Scope(scope)] = parent.rules() else {
        panic!("retained scope")
    };
    let [
        CssScopedRule::NestedDeclarations(before),
        CssScopedRule::Media(media),
    ] = scope.rules().rules()
    else {
        panic!("direct scoped run and complete media")
    };
    declaration(
        before.declarations(),
        scoped,
        CssKnownProperty::Width,
        "1px",
        20,
        26,
        29,
    );
    let [CssScopedRule::NestedDeclarations(run)] = media.rules().rules() else {
        panic!("direct scoped media declaration run")
    };
    declaration(
        run.declarations(),
        scoped,
        CssKnownProperty::Height,
        "2px",
        44,
        51,
        54,
    );
    assert_eq!(validate_sheet(scoped).unwrap(), *report.syntax());
}

#[test]
fn explicit_scope_root_with_style_ancestor_retains_direct_color_and_sheet_siblings() {
    let source = ".before{width:1px}.x{@scope (.s){color:red;@media screen}}.after{height:1px}";
    let report = parse_sheet(source);
    let [before, parent, after] = report.syntax().rules() else {
        panic!("original preceding and following styles")
    };
    declaration(
        style(before, "before").declarations(),
        source,
        CssKnownProperty::Width,
        "1px",
        8,
        14,
        17,
    );
    let parent = style(parent, "x");
    assert_eq!(parent.position().byte_offset().value(), 18);
    assert!(parent.declarations().is_empty());
    let [CssRule::Scope(scope)] = parent.rules() else {
        panic!("style-nested scope survives media rejection")
    };
    assert_eq!(scope.position().unwrap().byte_offset().value(), 21);
    assert!(scope.limit().is_none());
    assert_eq!(
        scope.root().unwrap().selectors(),
        &[CssScopeSelector::Selector(CssSelector::Class(
            "s".to_owned()
        ))]
    );
    let [CssScopedRule::NestedDeclarations(run)] = scope.rules().rules() else {
        panic!("direct Color run, no dropped media node")
    };
    declaration(
        run.declarations(),
        source,
        CssKnownProperty::Color,
        "red",
        33,
        39,
        42,
    );
    let after = style(after, "after");
    assert_eq!(after.position().byte_offset().value(), 58);
    declaration(
        after.declarations(),
        source,
        CssKnownProperty::Height,
        "1px",
        65,
        72,
        75,
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one direct scoped media missing body")
    };
    media_eof(diagnostic, 43, 56, 0, 56);
    strict(source, &report);
}
