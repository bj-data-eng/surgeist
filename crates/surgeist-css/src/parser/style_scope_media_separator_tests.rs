#![forbid(unsafe_code)]
// New cfg(test) parser child; do not change the existing boundary draft/module.
use super::{
    BoundedParseContext, StyleContextCaptures, conditional_chains, parse_sheet_bounded, recovery,
};
use crate::*;

const PREFIX: &str = "/*😀*/\r\n@namespace svg 'urn:original';";
const AFTER: &str = ".after{color:black}";

fn original(position: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 1);
    assert_eq!(
        position.column().value() as usize,
        source[..offset]
            .rsplit_once("\r\n")
            .unwrap()
            .1
            .encode_utf16()
            .count()
    );
}

fn color(declaration: &CssDeclaration, source: &str, expected: &str) {
    let offset = source.find(&format!("color:{expected}")).unwrap();
    original(declaration.position().unwrap(), source, offset);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        expected
    );
    let name = declaration.parsed_name().unwrap();
    let value = declaration.parsed_value().unwrap();
    assert_eq!(name.source().as_str(), source);
    assert!(name.source().same_snapshot(value.source()));
    original(value.span().start(), source, offset + 6);
    original(value.span().end(), source, offset + 6 + expected.len());
}

fn bounded(source: &str) -> CssParseReport<CssSheet> {
    let snapshot = CssSourceSnapshot::new(source);
    let report = parse_sheet_bounded(
        source,
        &snapshot,
        0,
        BoundedParseContext::Rules { top_level: true },
        StyleContextCaptures::default(),
        CssParserContext::default(),
    );
    recovery::finish_report(source, conditional_chains::sheet(source, report))
}

fn on_two_mib(run: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("style-scope-separators".to_owned())
        .stack_size(2 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

fn styled_source(depth: usize) -> String {
    format!(
        "{PREFIX}.host{{@scope{{{}color:red;svg|leaf.child{{color:blue}}color:green{}}}}}{AFTER}",
        "@media all{;/**/;".repeat(depth - 3),
        "}".repeat(depth - 3)
    )
}

fn check_styled(depth: usize) {
    let source = styled_source(depth);
    let report = bounded(&source);
    assert!(
        report.is_clean(),
        "depth {depth}: {:?}",
        report.diagnostics()
    );
    let [
        CssRule::Namespace(namespace),
        CssRule::Style(host),
        CssRule::Style(after),
    ] = report.syntax().rules()
    else {
        panic!("namespace, complete host and independent root sibling")
    };
    assert_eq!(namespace.prefix().unwrap().as_str(), "svg");
    assert_eq!(namespace.name().as_str(), "urn:original");
    original(host.position(), &source, source.find(".host{").unwrap());
    original(after.position(), &source, source.find(".after{").unwrap());
    let [after_color] = after.declarations().as_slice() else {
        panic!("later root color")
    };
    color(after_color, &source, "black");
    let [CssRule::Scope(scope)] = host.rules() else {
        panic!("style-nested Scope")
    };
    original(
        scope.position().unwrap(),
        &source,
        source.find("@scope{").unwrap(),
    );
    let mut children = scope.rules().rules();
    for (offset, _) in source.match_indices("@media all{") {
        let [CssScopedRule::Media(media)] = children else {
            panic!("every Media survives")
        };
        original(media.position().unwrap(), &source, offset);
        children = media.rules().rules();
    }
    let [
        CssScopedRule::NestedDeclarations(red),
        CssScopedRule::Style(leaf),
        CssScopedRule::NestedDeclarations(green),
    ] = children
    else {
        panic!(
            "depth {depth}: run, scoped leaf, restored sibling run in authored order; actual {children:?}"
        )
    };
    let [red_color] = red.declarations().as_slice() else {
        panic!("red run")
    };
    let [green_color] = green.declarations().as_slice() else {
        panic!("green run")
    };
    let [blue_color] = leaf.declarations().as_slice() else {
        panic!("leaf color")
    };
    color(red_color, &source, "red");
    color(blue_color, &source, "blue");
    color(green_color, &source, "green");
    assert!(
        red_color
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(after_color.parsed_value().unwrap().source())
    );
    original(
        leaf.position(),
        &source,
        source.find("svg|leaf.child{").unwrap(),
    );
    let [CssScopedStyleSelector::Selector(CssSelector::Compound(compound))] =
        leaf.selectors().selectors()
    else {
        panic!("namespaced scoped leaf")
    };
    assert_eq!(
        compound.type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap())
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 4);
    for (order, expected) in ["red", "blue", "green", "black"].into_iter().enumerate() {
        assert_eq!(values[order].order(), order);
        color(values[order].source(), &source, expected);
    }
    let host_context = normalized
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Rule(rule)
                if rule.position().is_some_and(|position| {
                    position.byte_offset().value() == source.find(".host{").unwrap()
                }) =>
            {
                Some(rule)
            }
            _ => None,
        })
        .unwrap();
    let CssRuleContextKindRef::Style(host_selectors) = host_context.kind() else {
        panic!("host context")
    };
    assert_eq!(
        host_selectors.selectors()[0].selector(),
        &CssSelector::Class("host".into())
    );
    for index in [0, 2] {
        assert!(
            values[index]
                .selector_context()
                .same_context(host_selectors)
        );
        assert!(values[index].selector_context().scope_context().is_none());
        let mut ancestor = values[index].rule_context().parent().unwrap();
        for (offset, _) in source
            .match_indices("@media all{")
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            assert!(matches!(ancestor.kind(), CssRuleContextKindRef::Media(_)));
            original(ancestor.position().unwrap(), &source, offset);
            ancestor = ancestor.parent().unwrap();
        }
        assert!(matches!(
            ancestor.kind(),
            CssRuleContextKindRef::Scope { .. }
        ));
        original(
            ancestor.position().unwrap(),
            &source,
            source.find("@scope{").unwrap(),
        );
        assert!(ancestor.parent().unwrap().same_context(host_context));
    }
    assert!(!values[1].selector_context().same_context(host_selectors));
    assert!(values[1].selector_context().parent().is_none());
    assert!(values[1].selector_context().scope_context().is_some());
    assert!(!values[3].selector_context().same_context(host_selectors));
    assert_eq!(
        values[3].selector_context().selectors()[0].selector(),
        &CssSelector::Class("after".into())
    );
}

#[test]
fn style_ancestry_keeps_media_semicolon_separators_and_nearest_selector_at_31_32_33() {
    on_two_mib(|| {
        for depth in [31, 32, 33] {
            check_styled(depth);
        }
    });
}

#[test]
fn style_ancestry_keeps_media_semicolon_separators_and_nearest_selector_at_63_64_65() {
    on_two_mib(|| {
        for depth in [63, 64, 65] {
            check_styled(depth);
        }
    });
}

#[test]
fn unstyled_scoped_media_keeps_semicolon_in_one_failed_qualified_unit() {
    on_two_mib(|| {
        for depth in [31, 32, 33, 63, 64, 65] {
            let groups = depth - 3;
            let unit = ";/**/@supports (color:red){.hidden{color:red}}";
            let source = format!(
                "{PREFIX}@scope{{{}{unit}.kept{{color:blue}}{}}}{AFTER}",
                "@media all{".repeat(groups),
                "}".repeat(groups)
            );
            let report = bounded(&source);
            let [
                CssRule::Namespace(_),
                CssRule::Scope(scope),
                CssRule::Style(after),
            ] = report.syntax().rules()
            else {
                panic!("scope and root sibling")
            };
            let mut children = scope.rules().rules();
            for (offset, _) in source.match_indices("@media all{") {
                let [CssScopedRule::Media(media)] = children else {
                    panic!("ordinary scoped Media chain")
                };
                original(media.position().unwrap(), &source, offset);
                children = media.rules().rules();
            }
            let [CssScopedRule::Style(kept)] = children else {
                panic!("hidden child cannot escape failed whole unit")
            };
            let [kept_color] = kept.declarations().as_slice() else {
                panic!("later child color")
            };
            let [after_color] = after.declarations().as_slice() else {
                panic!("later root color")
            };
            color(kept_color, &source, "blue");
            color(after_color, &source, "black");
            original(kept.position(), &source, source.find(".kept{").unwrap());
            original(after.position(), &source, source.find(".after{").unwrap());
            let [diagnostic] = report.diagnostics() else {
                panic!("one whole failed qualified unit")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
            assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
            let start = source.find(unit).unwrap();
            original(diagnostic.error().position(), &source, start);
            original(diagnostic.span().start(), &source, start);
            original(diagnostic.span().end(), &source, start + unit.len());
            let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
                panic!("qualified selector")
            };
            assert_eq!(
                detail.encountered().unwrap().kind(),
                CssTokenKind::Semicolon
            );
            assert_eq!(detail.encountered().unwrap().authored(), ";");
        }
    });
}
