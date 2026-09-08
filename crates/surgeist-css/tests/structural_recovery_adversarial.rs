use std::panic::{AssertUnwindSafe, catch_unwind};

use surgeist_css::{
    CssErrorCode, CssNamespaceConstraint, CssRecoveryAction, CssRule, CssScopedRule, CssSelector,
    CssSelectorCombinator, CssSourcePosition, ErrorKind, parse_sheet,
};

fn nested_layers(depth: usize, tail: &str) -> String {
    let mut source = "@layer{".repeat(depth);
    source.push_str(&"}".repeat(depth));
    source.push_str(tail);
    source
}

fn nested_supports(depth: usize, tail: &str) -> String {
    let mut source = "@supports (display:grid){".repeat(depth);
    source.push_str(&"}".repeat(depth));
    source.push_str(tail);
    source
}

fn namespace_nested_layers(total_depth: usize) -> String {
    let layer_depth = total_depth.saturating_sub(1);
    format!(
        "@namespace svg \"urn:svg\";{}svg|leaf{{color:red}}{}.after{{color:blue}}",
        "@layer{".repeat(layer_depth),
        "}".repeat(layer_depth),
    )
}

fn component_value(total_depth: usize, opener: &str, closer: &str) -> String {
    let component_depth = total_depth.saturating_sub(1);
    format!(
        ".target{{--x:{}x{};color:blue}}.after{{color:red}}",
        opener.repeat(component_depth),
        closer.repeat(component_depth),
    )
}

fn malformed_block(prefix: &str, total_depth: usize, tail: &str) -> String {
    let component_depth = total_depth.saturating_sub(1);
    format!(
        "{prefix}{}x{};}}{tail}",
        "f(".repeat(component_depth),
        ")".repeat(component_depth),
    )
}

fn distinct_nested_styles(
    depth: usize,
    declaration_levels: &[usize],
    relative_children: bool,
) -> String {
    let mut source = String::new();
    for level in 0..depth {
        if level > 0 && relative_children {
            source.push_str(&format!("& > .n{level:03}{{"));
        } else {
            source.push_str(&format!(".n{level:03}{{"));
        }
        if declaration_levels.contains(&level) {
            source.push_str("color:red;");
        }
    }
    source.push_str("display:block;");
    for level in (0..depth).rev() {
        if declaration_levels.contains(&level) {
            source.push_str("opacity:1;");
        }
        source.push('}');
    }
    source.push_str(".unrelated-empty{}");
    source
}

fn nested_empty_rule_order(depth: usize, scoped: bool) -> String {
    if !scoped {
        return format!(
            ".before-empty{{}}{}{}.after-empty{{}}",
            (0..depth)
                .map(|level| format!(".n{level:03}{{"))
                .collect::<String>(),
            "}".repeat(depth),
        );
    }
    let parent = "@scope{";
    format!(
        "{}.before-empty{{}}@layer{{}}.after-empty{{}}{}",
        parent.repeat(depth - 1),
        "}".repeat(depth - 1),
    )
}

fn selector_classes(selector: &CssSelector) -> Vec<&str> {
    match selector {
        CssSelector::Class(name) => vec![name],
        CssSelector::Complex(selector) => std::iter::once(selector.first())
            .chain(selector.rest().iter().map(|part| part.selector()))
            .map(|compound| {
                let [name] = compound.classes() else {
                    panic!("expected one class per selector compound");
                };
                name.as_str()
            })
            .collect(),
        unexpected => panic!("expected class selector chain, got {unexpected:?}"),
    }
}

fn assert_ascii_position(position: CssSourcePosition, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 0);
    assert_eq!(position.column().value(), offset as u32);
}

fn style_rules(
    report: &surgeist_css::CssParseReport<surgeist_css::CssSheet>,
) -> Vec<&surgeist_css::CssStyleRule> {
    report
        .syntax()
        .rules()
        .iter()
        .map(|rule| {
            let CssRule::Style(rule) = rule else {
                panic!("expected flattened style rule, got {rule:?}");
            };
            rule
        })
        .collect()
}

fn nesting_detail(
    report: &surgeist_css::CssParseReport<surgeist_css::CssSheet>,
) -> (
    &surgeist_css::CssNestingLimitError,
    surgeist_css::CssSourceSpan,
) {
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "expected exactly one nesting-limit diagnostic: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("expected typed nesting-limit detail");
    };
    (detail, diagnostic.span())
}

#[test]
fn structural_recovery_accepts_256_rule_blocks_and_drops_only_level_257() {
    for depth in [255, 256] {
        let source = nested_layers(depth, ".after{color:red}");
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().rules().len(), 2);
        let CssRule::LayerBlock(layer) = &report.syntax().rules()[0] else {
            panic!("expected outer layer chain");
        };
        let mut layer = layer;
        let mut retained_depth = 1;
        while let [CssRule::LayerBlock(nested)] = layer.rules() {
            layer = nested;
            retained_depth += 1;
        }
        assert_eq!(retained_depth, depth);
    }

    let source = nested_layers(257, ".after{color:red}");
    let report = parse_sheet(&source);
    assert_eq!(
        report.syntax().rules().len(),
        2,
        "outer layer and later sibling survive"
    );
    assert!(matches!(report.syntax().rules()[1], CssRule::Style(_)));
    let (detail, span) = nesting_detail(&report);
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.rule.layer-block"
    );
    let failed_start = "@layer{".len() * 256;
    assert_eq!(span.start().byte_offset().value(), failed_start);
    assert_eq!(
        span.end().byte_offset().value(),
        failed_start + "@layer{}".len()
    );
    assert_eq!(
        report.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        failed_start + "@layer".len(),
    );
}

#[test]
fn structural_split_preserves_namespace_bindings_through_the_256_level_boundary() {
    for depth in [255, 256] {
        let source = namespace_nested_layers(depth);
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let [
            CssRule::Namespace(_),
            CssRule::LayerBlock(layer),
            CssRule::Style(after),
        ] = report.syntax().rules()
        else {
            panic!(
                "expected namespace, retained layer parent, and later sibling at depth {depth}: {:#?}",
                report.syntax().rules()
            );
        };

        let mut layer = layer;
        for _ in 1..depth - 1 {
            let [CssRule::LayerBlock(nested)] = layer.rules() else {
                panic!("expected retained layer chain at depth {depth}: {layer:#?}");
            };
            layer = nested;
        }
        let [CssRule::Style(leaf)] = layer.rules() else {
            panic!("expected retained namespace-qualified child at depth {depth}: {layer:#?}");
        };
        let CssSelector::Compound(selector) = leaf.selector() else {
            panic!("expected namespace-qualified compound selector at depth {depth}");
        };
        assert!(matches!(
            selector
                .type_selector()
                .expect("namespace-qualified type selector")
                .namespace(),
            CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
        ));
        assert_eq!(leaf.declarations().len(), 1);
        assert_eq!(after.declarations().len(), 1);
    }

    let source = namespace_nested_layers(257);
    let report = parse_sheet(&source);
    let [
        CssRule::Namespace(_),
        CssRule::LayerBlock(layer),
        CssRule::Style(after),
    ] = report.syntax().rules()
    else {
        panic!(
            "expected namespace, retained parents, and later sibling at depth 257: {:#?}",
            report.syntax().rules()
        );
    };
    let mut layer = layer;
    for _ in 1..256 {
        let [CssRule::LayerBlock(nested)] = layer.rules() else {
            panic!("expected all 256 retained layer parents: {layer:#?}");
        };
        layer = nested;
    }
    assert!(layer.rules().is_empty());
    assert_eq!(after.declarations().len(), 1);

    let (detail, span) = nesting_detail(&report);
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.rule.style"
    );
    let failed_start = source.find("svg|leaf").expect("level 257 style");
    assert_eq!(span.start().byte_offset().value(), failed_start);
    assert_eq!(
        span.end().byte_offset().value(),
        failed_start + "svg|leaf{color:red}".len()
    );
}

#[test]
fn structural_preflight_preserves_256_nested_supports_groups_and_later_sibling() {
    let source = nested_supports(256, ".after{}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(rule), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("expected supports chain and later style");
    };
    let mut rule = rule;
    let mut depth = 1;
    while let [CssRule::Supports(nested)] = rule.rules() {
        rule = nested;
        depth += 1;
    }
    assert_eq!(depth, 256);
    assert!(rule.rules().is_empty());
    assert!(after.declarations().is_empty());
}

#[test]
fn structural_recovery_nesting_limit_at_eof_spans_remaining_bounded_unit() {
    let source = "@layer{".repeat(257);
    let report = parse_sheet(&source);
    let (detail, span) = nesting_detail(&report);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.rule.layer-block"
    );
    assert_eq!(span.start().byte_offset().value(), "@layer{".len() * 256);
    assert_eq!(span.end().byte_offset().value(), source.len());
}

#[test]
fn structural_recovery_shares_rule_and_component_depth_for_functions_and_blocks() {
    for (opener, closer) in [("f(", ")"), ("(", ")"), ("[", "]"), ("{", "}")] {
        for depth in [255, 256] {
            let source = component_value(depth, opener, closer);
            let report = parse_sheet(&source);
            assert!(
                report.is_clean(),
                "{opener} at depth {depth}: {:?}",
                report.diagnostics()
            );
            let CssRule::Style(target) = &report.syntax().rules()[0] else {
                panic!("expected target style rule");
            };
            assert_eq!(target.declarations().len(), 2);
        }

        let source = component_value(257, opener, closer);
        let report = parse_sheet(&source);
        assert_eq!(report.syntax().rules().len(), 2);
        let CssRule::Style(target) = &report.syntax().rules()[0] else {
            panic!("expected retained target style rule");
        };
        assert_eq!(
            target.declarations().len(),
            1,
            "only the excessive declaration is dropped"
        );
        let (detail, span) = nesting_detail(&report);
        assert_eq!(detail.limit(), 256);
        assert_eq!(detail.enclosing_production().as_str(), "css.declaration");
        let declaration_start = source.find("--x:").expect("generated declaration");
        let declaration_end = source
            .find(";color")
            .expect("generated declaration boundary")
            + 1;
        assert_eq!(span.start().byte_offset().value(), declaration_start);
        assert_eq!(span.end().byte_offset().value(), declaration_end);
    }
}

#[test]
fn structural_recovery_drops_only_an_excessively_nested_descriptor() {
    let excessive = format!("src:{}x{};", "f(".repeat(256), ")".repeat(256));
    let source = format!(
        "@font-face{{font-family:Demo;{excessive}src:url(\"demo.woff2\");}}.after{{color:red}}"
    );

    let report = parse_sheet(&source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::FontFace(_), CssRule::Style(_)]
    ));
    let (detail, span) = nesting_detail(&report);
    assert_eq!(detail.enclosing_production().as_str(), "css.descriptor");
    let start = source.find(&excessive).expect("generated descriptor");
    assert_eq!(span.start().byte_offset().value(), start);
    assert_eq!(span.end().byte_offset().value(), start + excessive.len());
}

#[test]
fn structural_recovery_reports_non_bmp_coordinates_and_keeps_later_siblings() {
    let prefix = "/* 🦊\n */";
    let source = format!("{prefix}{}", nested_layers(257, ".after{color:red}"));
    let report = parse_sheet(&source);
    let (_, span) = nesting_detail(&report);
    let failed_start = prefix.len() + "@layer{".len() * 256;
    assert_eq!(span.start().byte_offset().value(), failed_start);
    assert_eq!(span.start().line().value(), 1);
    assert_eq!(
        span.start().column().value(),
        3 + "@layer{".len() as u32 * 256
    );
    assert!(matches!(
        report.syntax().rules().last(),
        Some(CssRule::Style(_))
    ));
}

#[test]
fn malformed_at_rule_recovery_checks_balanced_component_depth() {
    for depth in [255, 256] {
        let source = malformed_block("@unknown{", depth, ".after{color:red}");
        let report = parse_sheet(&source);
        assert_eq!(report.diagnostics().len(), 1, "depth {depth}: {report:?}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    }

    let source = malformed_block("@unknown{", 257, ".after{color:red}");
    let unit_end = source.find(".after").expect("generated sibling boundary");
    let report = parse_sheet(&source);
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    let (detail, span) = nesting_detail(&report);
    assert_eq!(detail.enclosing_production().as_str(), "css.at-rule");
    assert_eq!(span.start().byte_offset().value(), 0);
    assert_eq!(span.end().byte_offset().value(), unit_end);
    assert_eq!(
        report.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        "@unknown{".len() + "f(".len() * 255,
    );
}

#[test]
fn nested_malformed_qualified_rule_recovery_checks_balanced_component_depth() {
    for depth in [255, 256] {
        let nested = malformed_block("???{", depth - 1, ".inside{color:blue}");
        let source = format!("@media screen{{{nested}}}.after{{color:red}}");
        let report = parse_sheet(&source);
        assert_eq!(report.diagnostics().len(), 1, "depth {depth}: {report:?}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropQualifiedRule
        );
        let CssRule::Media(media) = &report.syntax().rules()[0] else {
            panic!("expected retained media parent");
        };
        assert!(matches!(media.rules(), [CssRule::Style(_)]));
        assert!(matches!(
            report.syntax().rules().last(),
            Some(CssRule::Style(_))
        ));
    }

    let nested = malformed_block("???{", 256, ".inside{color:blue}");
    let source = format!("@media screen{{{nested}}}.after{{color:red}}");
    let unit_start = source.find("???").expect("generated failed unit");
    let unit_end = source.find(".inside").expect("generated nested sibling");
    let report = parse_sheet(&source);
    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!("expected retained media parent");
    };
    assert!(matches!(media.rules(), [CssRule::Style(_)]));
    assert!(matches!(
        report.syntax().rules().last(),
        Some(CssRule::Style(_))
    ));
    let (detail, span) = nesting_detail(&report);
    assert_eq!(detail.enclosing_production().as_str(), "css.qualified-rule");
    assert_eq!(span.start().byte_offset().value(), unit_start);
    assert_eq!(span.end().byte_offset().value(), unit_end);
    assert_eq!(
        report.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        unit_start + "???{".len() + "f(".len() * 254,
    );
}

#[test]
fn structural_preflight_ignores_delimiters_in_comments_strings_and_escapes() {
    let comment_delimiters = "({[".repeat(1024);
    let string_delimiters = "})]".repeat(1024);
    let source = format!(
        "/*{comment_delimiters}*/.target{{--x:\"{string_delimiters}\\\"tail\";--y:ident\\(\\[\\{{;color:blue}}.after{{color:red}}"
    );

    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{report:?}");
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Style(_), CssRule::Style(_)]
    ));
}

#[test]
fn structural_preflight_accepts_256_mixed_scope_and_style_blocks() {
    let source = format!(
        "{} .x{{color:red}} {}",
        "@scope{".repeat(255),
        "}".repeat(255)
    );
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("expected outer scope chain");
    };
    let mut scope = scope;
    let mut scope_depth = 1;
    while let [CssScopedRule::Scope(nested)] = scope.rules().rules() {
        scope = nested;
        scope_depth += 1;
    }
    assert_eq!(scope_depth, 255);
    assert!(matches!(scope.rules().rules(), [CssScopedRule::Style(_)]));
}

#[test]
fn structural_preflight_accepts_256_nested_style_blocks_without_losing_declarations() {
    let source = format!(
        "{}color:red{}}}.after{{color:blue}}",
        ".x{".repeat(256),
        "}".repeat(255),
    );
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 2);
    assert!(
        report
            .syntax()
            .rules()
            .iter()
            .all(|rule| matches!(rule, CssRule::Style(rule) if rule.declarations().len() == 1))
    );
}

#[test]
fn structural_preflight_at_64_preserves_nested_style_context_and_exact_empty_sibling() {
    let source = distinct_nested_styles(64, &[62], true);
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let styles = style_rules(&report);
    assert_eq!(styles.len(), 4, "{styles:#?}");
    assert_eq!(
        styles
            .iter()
            .map(|rule| selector_classes(rule.selector()).len())
            .collect::<Vec<_>>(),
        [63, 64, 63, 1],
    );
    assert_eq!(
        selector_classes(styles[1].selector()),
        (0..64)
            .map(|level| format!("n{level:03}"))
            .collect::<Vec<_>>(),
    );
    let CssSelector::Complex(final_selector) = styles[1].selector() else {
        panic!("expected composed complex selector");
    };
    assert!(
        final_selector
            .rest()
            .iter()
            .all(|part| part.combinator() == CssSelectorCombinator::Child)
    );
    assert_eq!(selector_classes(styles[3].selector()), ["unrelated-empty"]);
    assert!(styles[3].declarations().is_empty());

    let declaration_offsets = styles[..3]
        .iter()
        .map(|rule| {
            rule.declarations().as_slice()[0]
                .position()
                .byte_offset()
                .value()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        declaration_offsets,
        [
            source.find("color:red").expect("before declaration"),
            source.find("display:block").expect("deep declaration"),
            source.find("opacity:1").expect("after declaration"),
        ],
    );
}

#[test]
fn structural_preflight_at_256_preserves_every_style_chunk_context_and_source_order() {
    let declaration_levels = [62, 125, 188, 251];
    let source = distinct_nested_styles(256, &declaration_levels, false);
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let styles = style_rules(&report);
    assert_eq!(styles.len(), 10, "{styles:#?}");
    assert_eq!(
        styles
            .iter()
            .map(|rule| selector_classes(rule.selector()).len())
            .collect::<Vec<_>>(),
        [63, 126, 189, 252, 256, 252, 189, 126, 63, 1],
    );
    assert_eq!(
        selector_classes(styles[4].selector()),
        (0..256)
            .map(|level| format!("n{level:03}"))
            .collect::<Vec<_>>(),
    );
    assert_eq!(selector_classes(styles[9].selector()), ["unrelated-empty"]);
    assert!(styles[9].declarations().is_empty());

    let declaration_offsets = styles[..9]
        .iter()
        .map(|rule| {
            rule.declarations().as_slice()[0]
                .position()
                .byte_offset()
                .value()
        })
        .collect::<Vec<_>>();
    let mut expected_offsets = Vec::new();
    let mut cursor = 0;
    for _ in declaration_levels {
        let relative = source[cursor..]
            .find("color:red")
            .expect("before declaration");
        cursor += relative;
        expected_offsets.push(cursor);
        cursor += "color:red".len();
    }
    expected_offsets.push(source.find("display:block").expect("deep declaration"));
    cursor = 0;
    let mut after_offsets = Vec::new();
    while let Some(relative) = source[cursor..].find("opacity:1") {
        cursor += relative;
        after_offsets.push(cursor);
        cursor += "opacity:1".len();
    }
    expected_offsets.extend(after_offsets);
    assert_eq!(declaration_offsets, expected_offsets);
}

#[test]
fn structural_preflight_orders_empty_ordinary_styles_around_recovered_chunks() {
    for depth in [64, 256] {
        let source = nested_empty_rule_order(depth, false);
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );

        let [
            CssRule::Style(before),
            CssRule::Style(recovered),
            CssRule::Style(after),
        ] = report.syntax().rules()
        else {
            panic!(
                "expected authored empty rule order at depth {depth}: {:#?}",
                report.syntax().rules()
            );
        };
        assert_eq!(selector_classes(before.selector()), ["before-empty"]);
        assert_ascii_position(
            before.position(),
            source.find(".before-empty").expect("before style start"),
        );
        assert_eq!(
            selector_classes(recovered.selector()),
            (0..depth)
                .map(|level| format!("n{level:03}"))
                .collect::<Vec<_>>()
        );
        assert_ascii_position(
            recovered.position(),
            source
                .find(&format!(".n{:03}", depth - 1))
                .expect("recovered style start"),
        );
        assert_eq!(selector_classes(after.selector()), ["after-empty"]);
        assert_ascii_position(
            after.position(),
            source.find(".after-empty").expect("after style start"),
        );
        assert!(before.declarations().is_empty());
        assert!(recovered.declarations().is_empty());
        assert!(after.declarations().is_empty());
    }
}

#[test]
fn structural_preflight_orders_empty_scoped_styles_around_recovered_chunks() {
    for depth in [64, 256] {
        let source = nested_empty_rule_order(depth, true);
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );

        let [CssRule::Scope(scope)] = report.syntax().rules() else {
            panic!("expected outer scope at depth {depth}");
        };
        let mut scope = scope;
        for _ in 1..depth - 1 {
            let [CssScopedRule::Scope(nested)] = scope.rules().rules() else {
                panic!("expected nested scope chain at depth {depth}");
            };
            scope = nested;
        }
        let [
            CssScopedRule::Style(before),
            CssScopedRule::LayerBlock(recovered),
            CssScopedRule::Style(after),
        ] = scope.rules().rules()
        else {
            panic!("expected authored empty scoped rule order at depth {depth}: {scope:#?}");
        };
        let [surgeist_css::CssScopedStyleSelector::Selector(before_selector)] =
            before.selectors().selectors()
        else {
            panic!("expected one ordinary scoped selector at depth {depth}");
        };
        let [surgeist_css::CssScopedStyleSelector::Selector(after_selector)] =
            after.selectors().selectors()
        else {
            panic!("expected one ordinary scoped selector at depth {depth}");
        };
        assert_eq!(selector_classes(before_selector), ["before-empty"]);
        assert_ascii_position(
            before.position(),
            source
                .find(".before-empty")
                .expect("before scoped style start"),
        );
        assert!(recovered.rules().rules().is_empty());
        assert_ascii_position(
            recovered.position(),
            source.find("@layer").expect("recovered scoped layer start"),
        );
        assert_eq!(selector_classes(after_selector), ["after-empty"]);
        assert_ascii_position(
            after.position(),
            source
                .find(".after-empty")
                .expect("after scoped style start"),
        );
        assert!(before.declarations().is_empty());
        assert!(after.declarations().is_empty());
    }
}

#[test]
fn structural_preflight_drops_only_style_level_257_with_exact_parent_order() {
    let source = distinct_nested_styles(257, &[255], false);
    let report = parse_sheet(&source);

    let styles = style_rules(&report);
    assert_eq!(styles.len(), 3, "{styles:#?}");
    assert_eq!(
        styles
            .iter()
            .map(|rule| selector_classes(rule.selector()).len())
            .collect::<Vec<_>>(),
        [256, 256, 1],
    );
    assert_eq!(selector_classes(styles[2].selector()), ["unrelated-empty"]);
    assert!(styles[2].declarations().is_empty());
    assert_eq!(
        styles[..2]
            .iter()
            .map(|rule| {
                rule.declarations().as_slice()[0]
                    .position()
                    .byte_offset()
                    .value()
            })
            .collect::<Vec<_>>(),
        [
            source.find("color:red").expect("before declaration"),
            source.find("opacity:1").expect("after declaration"),
        ],
    );

    let (detail, span) = nesting_detail(&report);
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.rule.style"
    );
    let failed_start = source.find(".n256{").expect("level 257 style");
    assert_eq!(span.start().byte_offset().value(), failed_start);
    assert_eq!(
        span.end().byte_offset().value(),
        failed_start + ".n256{display:block;}".len()
    );
    assert_eq!(
        report.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        failed_start + ".n256".len(),
    );
}

#[test]
fn structural_recovery_retains_empty_keyframe_parents_after_declaration_loss() {
    let source = "@keyframes fade { from { mystery: 1; } } .after { color: red; }";
    let report = parse_sheet(source);
    let [CssRule::Keyframes(keyframes), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("expected retained empty keyframe structure and later style sibling");
    };
    let [block] = keyframes.blocks() else {
        panic!("expected authored keyframe block");
    };
    assert!(block.declarations().is_empty());
    assert_eq!(after.declarations().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one declaration recovery diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("mystery").unwrap(),
    );
}

#[test]
fn structural_recovery_never_unwinds_on_bounded_adversarial_text() {
    let deep = component_value(1024, "f(", ")");
    let repeated = format!("{}{}", "@bad{};".repeat(256), ".after{color:red}");
    let cases = [
        ("", false),
        (";;;;;}}}}\0\u{fffd}", false),
        ("🦊💥\n@unknown fn({a;b}); .after{color:red}", false),
        (deep.as_str(), true),
        (repeated.as_str(), true),
    ];

    for (source, expects_later_sibling) in cases {
        let result = catch_unwind(AssertUnwindSafe(|| parse_sheet(source)));
        let report = result.unwrap_or_else(|_| panic!("ordinary input unwound: {source:?}"));
        if source == deep {
            let (detail, span) = nesting_detail(&report);
            assert_eq!(detail.enclosing_production().as_str(), "css.declaration");
            let declaration_start = source.find("--x:").expect("generated declaration");
            let declaration_end = source
                .find(";color")
                .expect("generated declaration boundary")
                + 1;
            assert_eq!(span.start().byte_offset().value(), declaration_start);
            assert_eq!(span.end().byte_offset().value(), declaration_end);
        }
        if expects_later_sibling {
            assert!(
                matches!(report.syntax().rules().last(), Some(CssRule::Style(_))),
                "later sibling was lost for {source:?}: {report:?}"
            );
        }
    }
}
