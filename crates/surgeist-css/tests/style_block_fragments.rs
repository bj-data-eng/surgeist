#![forbid(unsafe_code)]
//! Real-brace style-body parsing retains authored contents and genuine origins.
use surgeist_css::{
    CssDeclaration, CssKnownProperty, CssKnownPropertyValueRef, CssNamespaceContext,
    CssNamespaceName, CssNamespacePrefix, CssRecoveryAction, CssRule, CssSelector,
    CssStyleSelector, CssValueOrigin, parse_style_block,
};

fn parse(source: &str) -> surgeist_css::CssParseReport<Option<surgeist_css::CssStyleBlock>> {
    parse_style_block(source, &CssNamespaceContext::default())
}

fn color(declaration: &CssDeclaration, expected: [u8; 3]) {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    let rgba = value.i01_subset().unwrap().as_rgba().unwrap();
    assert_eq!([rgba.red(), rgba.green(), rgba.blue()], expected);
    assert_eq!(rgba.alpha(), 1.0);
}

#[test]
fn empty_and_recovered_empty_blocks_have_distinct_retained_reports() {
    for source in ["{}", "{;}", "{/*comment*/ ; }"] {
        let report = parse(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let block = report.syntax().as_ref().unwrap();
        assert!(block.declarations().is_empty());
        assert!(block.rules().is_empty());
        assert_eq!(block.origin().span().start().byte_offset().value(), 0);
        assert_eq!(
            block.origin().span().end().byte_offset().value(),
            source.len()
        );
    }
    for (source, action) in [
        ("{unknown:x}", CssRecoveryAction::DropDeclaration),
        ("{@unknown x;}", CssRecoveryAction::DropAtRule),
    ] {
        let report = parse(source);
        let block = report.syntax().as_ref().unwrap();
        assert!(block.declarations().is_empty());
        assert!(block.rules().is_empty());
        let [diagnostic] = report.diagnostics() else {
            panic!("one inner error")
        };
        assert_eq!(diagnostic.action(), action);
    }
}

#[test]
fn typed_declarations_and_nested_runs_remain_in_authored_order() {
    let report =
        parse("{color:red; &{color:blue} width:1px; @media screen {height:2px} opacity:.5}");
    assert!(report.is_clean(), "{report:?}");
    let block = report.syntax().as_ref().unwrap();
    assert_eq!(block.declarations().len(), 1);
    color(&block.declarations()[0], [255, 0, 0]);
    let [
        CssRule::Style(child),
        CssRule::NestedDeclarations(width),
        CssRule::Media(media),
        CssRule::NestedDeclarations(opacity),
    ] = block.rules()
    else {
        panic!("ordered children and declaration runs")
    };
    color(&child.declarations()[0], [0, 0, 255]);
    let [CssStyleSelector::Selector(CssSelector::Compound(anchor))] = child.selectors().selectors()
    else {
        panic!("authored anchor")
    };
    assert_eq!(anchor.nesting_selectors(), 1);
    assert_eq!(
        width.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Width
    );
    let [CssRule::NestedDeclarations(height)] = media.rules() else {
        panic!("group declaration run")
    };
    assert_eq!(
        height.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Height
    );
    assert_eq!(
        opacity.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );

    let report = parse("{color:red; @unknown x; color:blue}");
    let block = report.syntax().as_ref().unwrap();
    assert!(block.rules().is_empty());
    assert_eq!(block.declarations().len(), 2);
    color(&block.declarations()[0], [255, 0, 0]);
    color(&block.declarations()[1], [0, 0, 255]);
}

#[test]
fn block_origin_excludes_outer_trivia_and_shares_the_original_snapshot() {
    let source = "/*😀*/\r\n{color:red;--x:{a:b;c:d}} /*tail*/";
    let report = parse(source);
    assert!(report.is_clean(), "{report:?}");
    let block = report.syntax().as_ref().unwrap();
    let origin = block.origin();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 10);
    assert_eq!(origin.span().start().line().value(), 1);
    assert_eq!(origin.span().start().column().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), source.len() - 9);
    for declaration in block.declarations().iter() {
        let value = declaration.parsed_value().unwrap();
        assert!(origin.source().same_snapshot(value.source()));
        assert!(
            origin
                .source()
                .same_snapshot(declaration.parsed_name().unwrap().source())
        );
        for component in declaration.value_components().items() {
            let CssValueOrigin::Parsed(token) = component.origin() else {
                panic!("original token")
            };
            assert!(origin.source().same_snapshot(token.source()));
        }
    }
    assert_eq!(
        block.declarations()[1].custom().unwrap().name().as_str(),
        "--x"
    );
    let second = parse(source);
    assert_eq!(report.syntax(), second.syntax());
    assert!(
        !origin
            .source()
            .same_snapshot(second.syntax().as_ref().unwrap().origin().source())
    );
    assert_ne!(parse("{}").syntax(), parse(" {} ").syntax());
}

#[test]
fn invalid_outer_input_discards_all_provisional_inner_recovery() {
    for source in [
        "",
        " ",
        "color:red",
        "a{}",
        "[]",
        "()",
        "{}{}",
        "{color:red};",
        "{color:red} a{color:blue}",
        "{unknown:x} }",
        "{unknown:x} )",
        "{unknown:x} ]",
    ] {
        let report = parse(source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one outer rejection")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn implicit_outer_brace_closes_at_actual_eof() {
    for source in ["{", "{color:red"] {
        let report = parse(source);
        let block = report.syntax().as_ref().unwrap();
        assert_eq!(block.origin().span().start().byte_offset().value(), 0);
        assert_eq!(
            block.origin().span().end().byte_offset().value(),
            source.len()
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("outer closure: {report:?}")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
    }
}

#[test]
fn relative_children_retain_namespace_constraints_without_a_fabricated_parent() {
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    let context = CssNamespaceContext::from_bindings([(
        Some(prefix.clone()),
        CssNamespaceName::new("urn:svg"),
    )]);
    let report = parse_style_block("{> svg|leaf{color:red} svg|leaf&{color:blue}}", &context);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Style(relative), CssRule::Style(absolute)] =
        report.syntax().as_ref().unwrap().rules()
    else {
        panic!("two children")
    };
    let [CssStyleSelector::Relative(relative)] = relative.selectors().selectors() else {
        panic!("relative child")
    };
    assert_eq!(
        relative.combinator(),
        surgeist_css::CssSelectorCombinator::Child
    );
    let CssSelector::Compound(compound) = relative.selector() else {
        panic!("qualified type")
    };
    assert_eq!(
        compound.type_selector().unwrap().namespace(),
        &surgeist_css::CssNamespaceConstraint::Named(prefix.clone())
    );
    let [CssStyleSelector::Selector(CssSelector::Compound(compound))] =
        absolute.selectors().selectors()
    else {
        panic!("qualified anchor")
    };
    assert_eq!(compound.nesting_selectors(), 1);
    assert_eq!(
        context.named_namespace(&prefix).unwrap().as_str(),
        "urn:svg"
    );
    let missing = parse("{svg|leaf{color:red}}");
    assert!(missing.syntax().is_some());
    assert!(missing.syntax().as_ref().unwrap().rules().is_empty());
    assert!(!missing.is_clean());
}

#[test]
fn bounded_child_resource_recovery_preserves_ancestors_and_following_declarations() {
    for depth in [63, 64, 254] {
        let source = format!(
            "{{{}a{{color:red}}{}}}",
            "a{".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let mut rules = report.syntax().as_ref().unwrap().rules();
        for _ in 0..depth {
            let [CssRule::Style(parent)] = rules else {
                panic!("retained style parent")
            };
            rules = parent.rules();
        }
        let [CssRule::Style(leaf)] = rules else {
            panic!("retained style leaf")
        };
        assert_eq!(leaf.declarations().len(), 1);
        color(&leaf.declarations()[0], [255, 0, 0]);
    }
    // One block plus 255 style ancestors exhausts the limit before the leaf.
    // The following declaration belongs to the retained deepest ancestor.
    let source = format!(
        "{{{}a{{color:red}}width:1px{}}}",
        "a{".repeat(255),
        "}".repeat(255)
    );
    let report = parse(&source);
    let block = report.syntax().as_ref().unwrap();
    let [diagnostic] = report.diagnostics() else {
        panic!("one resource error: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    let mut rules = block.rules();
    for index in 0..255 {
        let [CssRule::Style(style)] = rules else {
            panic!("retained ancestor {index}")
        };
        if index == 254 {
            assert_eq!(style.declarations().len(), 1);
            assert_eq!(
                style.declarations()[0].known().unwrap().property(),
                CssKnownProperty::Width
            );
            assert!(style.rules().is_empty());
        }
        rules = style.rules();
    }
}

#[test]
fn nested_groups_preserve_supported_depth_and_own_inner_limits() {
    for depth in [63, 64, 254] {
        let source = format!(
            "{{{}a{{color:red}}{}}}",
            "@media screen{".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let mut rules = report.syntax().as_ref().unwrap().rules();
        for _ in 0..depth {
            let [CssRule::Media(parent)] = rules else {
                panic!("retained media parent")
            };
            rules = parent.rules();
        }
        let [CssRule::Style(leaf)] = rules else {
            panic!("retained style leaf")
        };
        color(&leaf.declarations()[0], [255, 0, 0]);
    }
    let source = format!(
        "{{{}a{{color:red}}{}}}",
        "@media screen{".repeat(255),
        "}".repeat(255)
    );
    let report = parse(&source);
    assert!(report.syntax().is_some());
    let [diagnostic] = report.diagnostics() else {
        panic!("one group resource failure: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
}
