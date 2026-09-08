use surgeist_css::{
    CssErrorCode, CssRecoveryAction, CssRule, CssTokenKind, ErrorKind, parse_sheet,
};

fn actions(source: &str) -> Vec<CssRecoveryAction> {
    parse_sheet(source)
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.action())
        .collect()
}

fn assert_implicit_closures(source: &str, expected: usize) {
    let report = parse_sheet(source);
    let eof = source.len();
    let implicit = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .collect::<Vec<_>>();

    assert_eq!(
        implicit.len(),
        expected,
        "{source}: {:?}",
        report.diagnostics()
    );
    for diagnostic in implicit {
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(diagnostic.error().position().byte_offset().value(), eof);
        assert_eq!(diagnostic.span().start().byte_offset().value(), eof);
        assert_eq!(diagnostic.span().end().byte_offset().value(), eof);
        let ErrorKind::UnexpectedEnd(detail) = diagnostic.error().kind() else {
            panic!("expected typed unexpected-end detail")
        };
        assert_eq!(detail.expectation().as_str(), "valid CSS syntax");
    }
}

fn nested_selector(depth: usize, close: bool) -> String {
    let mut source = ":is(".repeat(depth);
    source.push_str(".leaf");
    if close {
        source.push_str(&")".repeat(depth));
        source.push_str("{color:red}");
    }
    source
}

fn nested_media(depth: usize, close: bool) -> String {
    let mut source = String::from("@media ");
    source.push_str(&"(".repeat(depth));
    source.push_str("width:1px");
    if close {
        source.push_str(&")".repeat(depth));
        source.push_str("{.leaf{color:red}}");
    }
    source
}

fn nested_supports(depth: usize, close: bool) -> String {
    let mut source = String::from("@supports ");
    source.push_str(&"f(".repeat(depth));
    source.push('x');
    if close {
        source.push_str(&")".repeat(depth));
        source.push_str("{.leaf{color:red}}");
    }
    source
}

#[test]
fn specialized_boundary_final_rule_blocks_retain_with_one_exact_eof_closure() {
    let sources = [
        ".x{color:red",
        "@layer{.x{color:red;}",
        "@media screen{.x{color:red;}",
        "@supports (display:grid){.x{color:red;}",
        "@container (width > 1px){.x{color:red;}",
        "@scope{.x{color:red;}",
        "@font-face{font-family:Demo;src:url(\"demo.woff2\");",
        "@keyframes fade{from{opacity:0;}",
    ];

    for source in sources {
        let report = parse_sheet(source);
        assert_eq!(report.syntax().rules().len(), 1, "{source}");
        assert_implicit_closures(source, 1);
    }
}

#[test]
fn specialized_boundary_nested_eof_closures_are_innermost_to_outermost() {
    let source = "@keyframes fade{from{--v:f(x";
    let report = parse_sheet(source);

    assert!(matches!(report.syntax().rules(), [CssRule::Keyframes(_)]));
    assert_implicit_closures(source, 3);
    assert_eq!(
        actions(source),
        vec![
            CssRecoveryAction::RetainWithImplicitClosure,
            CssRecoveryAction::RetainWithImplicitClosure,
            CssRecoveryAction::RetainWithImplicitClosure,
        ]
    );
}

#[test]
fn specialized_boundary_nested_components_allocate_one_closure_each() {
    let source = ".x{--v:f(g([x";
    let report = parse_sheet(source);

    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("expected retained style rule")
    };
    assert_eq!(rule.declarations().len(), 1);
    assert_implicit_closures(source, 4);
}

#[test]
fn specialized_boundary_balanced_declaration_components_each_get_one_eof_closure() {
    for opener in ["f(", "(", "[", "{"] {
        let source = format!(".x{{--v:{opener}x");
        let report = parse_sheet(&source);
        let [CssRule::Style(rule)] = report.syntax().rules() else {
            panic!("expected retained style rule for {source}")
        };
        assert_eq!(rule.declarations().len(), 1, "{source}");
        assert_implicit_closures(&source, 2);
    }

    let descriptor = "@font-face{font-family:Demo;src:local(Demo";
    assert!(matches!(
        parse_sheet(descriptor).syntax().rules(),
        [CssRule::FontFace(_)]
    ));
    assert_implicit_closures(descriptor, 2);

    let format = "@font-face{font-family:Demo;src:url(face) format(\"woff2\"";
    assert!(matches!(
        parse_sheet(format).syntax().rules(),
        [CssRule::FontFace(_)]
    ));
    assert_implicit_closures(format, 2);
}

#[test]
fn font_source_descriptor_components_observe_the_exact_nesting_limit() {
    let source = format!(
        "@font-face{{font-family:Demo;src:url(face) {}x{};}}",
        "f(".repeat(255),
        ")".repeat(255),
    );
    let report = parse_sheet(&source);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() != CssRecoveryAction::StopAtNestingLimit)
    );

    for depth in [256_usize, 257] {
        let source = format!(
            "@font-face{{font-family:Demo;src:url(face) {}x{};}}",
            "f(".repeat(depth),
            ")".repeat(depth),
        );
        let first_over_limit = source.match_indices("f(").nth(255).unwrap().0;
        let report = parse_sheet(&source);
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.action() == CssRecoveryAction::StopAtNestingLimit)
            .expect("component beyond the enclosing block depth must stop at the limit");
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            first_over_limit,
            "depth {depth}"
        );
    }
}

#[test]
fn counter_style_descriptor_components_observe_eof_and_exact_nesting_boundaries() {
    let eof = "@counter-style eof{system:additive;additive-symbols:1 I;negative:\"-\"";
    assert!(matches!(
        parse_sheet(eof).syntax().rules(),
        [CssRule::CounterStyle(_)]
    ));
    assert_implicit_closures(eof, 1);

    for depth in [255_usize, 256, 257] {
        let source = format!(
            "@counter-style deep{{system:additive;additive-symbols:1 {}x{};}}.after{{}}",
            "f(".repeat(depth),
            ")".repeat(depth),
        );
        let report = parse_sheet(&source);
        assert!(matches!(
            report.syntax().rules().last(),
            Some(CssRule::Style(_))
        ));
        let stopped = report.diagnostics().iter().any(|diagnostic| {
            diagnostic.error().code() == CssErrorCode::NestingLimit
                && diagnostic.action() == CssRecoveryAction::StopAtNestingLimit
        });
        assert_eq!(
            stopped,
            depth >= 256,
            "depth {depth}: {:?}",
            report.diagnostics()
        );
    }
}

#[test]
fn specialized_boundary_misleading_delimiters_do_not_change_eof_allocation() {
    let source = ".x{--v:f(\") }\"/* ] } */x";
    let report = parse_sheet(source);

    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("expected retained style rule")
    };
    assert_eq!(rule.declarations().len(), 1);
    assert_implicit_closures(source, 2);
}

#[test]
fn specialized_boundary_retained_import_media_component_gets_one_eof_closure() {
    let source = "@import \"theme.css\" (width:1px";
    let report = parse_sheet(source);

    assert!(matches!(report.syntax().rules(), [CssRule::Import(_)]));
    assert_implicit_closures(source, 1);
}

#[test]
fn specialized_boundary_nonrepresentable_eof_cases_do_not_gain_component_closures() {
    let cases = [
        "@media",
        "{color:red",
        ".x{color:",
        ".x{color",
        ".x{content:\"unterminated",
        ".x{background-image:url(bad url",
        "@font-face{font-family:Demo",
        "@keyframes fade{",
    ];

    for source in cases {
        let report = parse_sheet(source);
        let retained_closures = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
            })
            .count();
        assert!(
            retained_closures <= usize::from(!report.syntax().rules().is_empty()),
            "{source}: {:?}",
            report.diagnostics()
        );
    }
}

#[test]
fn specialized_boundary_legacy_tokens_have_exact_nonempty_spans_and_keep_rule_order() {
    let source = "<!-- .before{color:red} --> .middle{color:blue} <!-- .after{color:black}";
    let report = parse_sheet(source);
    let names = report
        .syntax()
        .rules()
        .iter()
        .map(|rule| {
            let CssRule::Style(style) = rule else {
                panic!("expected style rule")
            };
            let surgeist_css::CssSelector::Class(name) = style.selector() else {
                panic!("expected class selector")
            };
            name.as_str()
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["before", "middle", "after"]);

    let expected = [
        (0, 4, CssTokenKind::Cdo),
        (24, 27, CssTokenKind::Cdc),
        (48, 52, CssTokenKind::Cdo),
    ];
    assert_eq!(report.diagnostics().len(), expected.len());
    for (diagnostic, (start, end, kind)) in report.diagnostics().iter().zip(expected) {
        assert_eq!(diagnostic.action(), CssRecoveryAction::IgnoreLegacyToken);
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
        assert_eq!(diagnostic.error().position().byte_offset().value(), start);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(diagnostic.span().end().byte_offset().value(), end);
        assert!(start < end);
        let ErrorKind::UnexpectedToken(detail) = diagnostic.error().kind() else {
            panic!("expected typed unexpected-token detail")
        };
        assert_eq!(detail.encountered().kind(), kind);
        assert_eq!(detail.encountered().authored(), &source[start..end]);
    }
}

#[test]
fn specialized_boundary_selector_depth_is_exact_at_255_256_and_257() {
    for depth in [255, 256] {
        let source = nested_selector(depth, true);
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().rules().len(), 1);
    }

    let source = nested_selector(257, true);
    let report = parse_sheet(&source);
    assert!(report.syntax().rules().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "expected one selector nesting diagnostic: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        ":is(".len() * 256 + 1
    );
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("expected typed nesting-limit detail")
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.selector.complex"
    );
}

#[test]
fn namespace_context_survives_selector_depth_boundaries() {
    for depth in [255, 256] {
        let mut source = String::from("@namespace svg \"urn:svg\";");
        source.push_str(&":is(".repeat(depth));
        source.push_str("svg|leaf");
        source.push_str(&")".repeat(depth));
        source.push_str("{color:red}");
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::Namespace(_), CssRule::Style(_)]
        ));
    }

    let mut source = String::from("@namespace svg \"urn:svg\";");
    source.push_str(&":is(".repeat(257));
    source.push_str("svg|leaf");
    source.push_str(&")".repeat(257));
    source.push_str("{color:red}");
    let report = parse_sheet(&source);
    assert!(matches!(report.syntax().rules(), [CssRule::Namespace(_)]));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one selector nesting diagnostic")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
}

#[test]
fn specialized_boundary_media_depth_is_exact_at_255_256_and_257() {
    for depth in [255, 256] {
        let source = nested_media(depth, true);
        let report = parse_sheet(&source);
        assert_eq!(report.syntax().rules().len(), 1);
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one ordinary media recovery at depth {depth}")
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
    }

    let source = nested_media(257, true);
    let report = parse_sheet(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "expected one media nesting diagnostic: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 6);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find('{').expect("media block opening")
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        "@media ".len() + 256
    );
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("expected typed nesting-limit detail")
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.media.query-list"
    );
}

#[test]
fn specialized_boundary_supports_depth_is_exact_at_255_256_and_257() {
    for depth in [255, 256] {
        let source = nested_supports(depth, true);
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert!(matches!(report.syntax().rules(), [CssRule::Supports(_)]));
    }

    let source = nested_supports(257, true);
    let report = parse_sheet(&source);
    assert!(report.syntax().rules().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one supports nesting diagnostic")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("expected typed nesting-limit detail")
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.rule.supports"
    );
}

#[test]
fn specialized_boundary_eof_over_limit_has_only_the_limit_action() {
    let mut declaration = String::from(".x{--v:");
    declaration.push_str(&"f(".repeat(256));
    declaration.push('x');
    for source in [
        nested_selector(257, false),
        nested_media(257, false),
        nested_supports(257, false),
        declaration,
    ] {
        let report = parse_sheet(&source);
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::NestingLimit)
                .count(),
            1,
            "{source}: {:?}",
            report.diagnostics()
        );
        assert!(!report.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        }));
    }
}
