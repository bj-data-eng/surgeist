#![forbid(unsafe_code)]
//! Global definitions retain the same payload inside ordinary scope (Cascade6
//! §2.5.5); surrounding selectors and authored parents retain their scope context.
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-nesting
use surgeist_css::*;

const DEFINITIONS: &str = concat!(
    "@counter-style Tick{system:cyclic;symbols:\"x\"}",
    "@font-face{font-family:Audit;src:local(Audit)}",
    "@keyframes audit{from{opacity:0}to{opacity:1}}"
);

fn assert_leaves(rules: &[CssScopedRule]) {
    let [
        CssScopedRule::CounterStyle(counter),
        CssScopedRule::FontFace(font),
        CssScopedRule::Keyframes(keyframes),
    ] = rules
    else {
        panic!("three typed global definitions: {rules:?}")
    };
    assert_eq!(counter.name().as_str(), "Tick");
    assert!(matches!(
        counter.descriptors().system().unwrap().value(),
        CssCounterStyleSystem::Cyclic
    ));
    assert!(
        matches!(counter.descriptors().symbols().unwrap().symbols(), [CssCounterSymbol::String(value)] if value.as_str() == "x")
    );
    assert_eq!(
        font.descriptors().font_family().unwrap().value().as_str(),
        "Audit"
    );
    assert!(
        matches!(font.descriptors().src().unwrap().value().sources(), [CssFontFaceSource::Local(value)] if value.as_str() == "Audit")
    );
    assert!(
        matches!(keyframes.name(), CssKeyframesName::Ident(value) if value.as_str() == "audit")
    );
    assert_eq!(keyframes.blocks().len(), 2);
    assert_eq!(keyframes.blocks()[0].declarations().len(), 1);
    assert_eq!(keyframes.blocks()[1].declarations().len(), 1);
}

#[test]
fn scoped_variants_reuse_existing_owned_payloads_and_preserve_detached_values() {
    let source = format!("@scope(.host){{{DEFINITIONS}}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    assert_leaves(scope.rules().rules());
    let leaves = scope.rules().rules().to_vec();
    assert_eq!(leaves, scope.rules().rules());
    let ordinary = leaves
        .iter()
        .map(|leaf| match leaf {
            CssScopedRule::CounterStyle(value) => CssRule::CounterStyle(value.clone()),
            CssScopedRule::FontFace(value) => CssRule::FontFace(value.clone()),
            CssScopedRule::Keyframes(value) => CssRule::Keyframes(value.clone()),
            _ => panic!("global leaf"),
        })
        .collect::<Vec<_>>();
    drop(report);
    assert_leaves(&leaves);
    for (leaf, value) in leaves.iter().zip(ordinary) {
        match (leaf, value) {
            (CssScopedRule::CounterStyle(left), CssRule::CounterStyle(right)) => {
                assert_eq!(*left, right)
            }
            (CssScopedRule::FontFace(left), CssRule::FontFace(right)) => assert_eq!(*left, right),
            (CssScopedRule::Keyframes(left), CssRule::Keyframes(right)) => assert_eq!(*left, right),
            _ => panic!("same payload identity"),
        }
    }
}

fn normalized_definitions(sheet: &CssNormalizedSheet) -> Vec<&CssRuleContext> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context)
                if matches!(
                    context.kind(),
                    CssRuleContextKindRef::CounterStyle(_)
                        | CssRuleContextKindRef::FontFace(_)
                        | CssRuleContextKindRef::Keyframes(_)
                ) =>
            {
                Some(context)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn bounded_scoped_chunks_keep_definition_positions_order_and_parentage() {
    // The parser splits at 64 structural blocks. Groups at depths 63, 64 and 65
    // exercise both chunk conversion directions and sibling splicing around it.
    for group in [
        "@media all{",
        "@supports(display:grid){",
        "@container(width>1px){",
        "@layer audit{",
    ] {
        let mut source = "@scope(.root){".to_owned();
        source.push_str(&group.repeat(61));
        for _ in 0..3 {
            source.push_str(group);
            source.push_str(DEFINITIONS);
        }
        source.push_str("tail{}");
        source.push_str(&"}".repeat(65));
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{group}: {:?}", report.diagnostics());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let definitions = normalized_definitions(&normalized);
        assert_eq!(definitions.len(), 9);
        let expected_offsets: Vec<_> = source
            .match_indices('@')
            .filter(|(offset, _)| {
                let tail = &source[*offset..];
                tail.starts_with("@counter-style")
                    || tail.starts_with("@font-face")
                    || tail.starts_with("@keyframes")
            })
            .map(|(offset, _)| offset)
            .collect();
        assert_eq!(
            definitions
                .iter()
                .map(|c| c.position().unwrap().byte_offset().value())
                .collect::<Vec<_>>(),
            expected_offsets
        );
        for chunk in definitions.chunks_exact(3) {
            let parent = chunk[0].parent().unwrap();
            assert!(
                chunk
                    .iter()
                    .all(|c| c.parent().unwrap().same_context(parent))
            );
            assert!(
                matches!(chunk[0].kind(), CssRuleContextKindRef::CounterStyle(value) if value.name().as_str() == "Tick")
            );
            assert!(
                matches!(chunk[1].kind(), CssRuleContextKindRef::FontFace(value) if value.descriptors().font_family().unwrap().value().as_str() == "Audit")
            );
            assert!(
                matches!(chunk[2].kind(), CssRuleContextKindRef::Keyframes(value) if value.blocks().len() == 2)
            );
        }
    }
}

#[test]
fn bounded_scoped_chunks_preserve_style_ancestor_rejection_and_neighboring_styles() {
    let source = format!(
        "host{{{}@scope{{before{{}}{DEFINITIONS}after{{}}}}{}}}",
        "@media all{".repeat(64),
        "}".repeat(64)
    );
    let report = parse_sheet(&source);
    assert_eq!(report.diagnostics().len(), 3, "{:?}", report.diagnostics());
    for diagnostic in report.diagnostics() {
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePlacement
        );
    }
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert!(normalized_definitions(&normalized).is_empty());
    for marker in ["before{}", "after{}"] {
        let offset = source.find(marker).unwrap();
        assert!(normalized.items().iter().any(|item| matches!(item, CssNormalizedItem::Rule(context) if context.position().is_some_and(|p| p.byte_offset().value() == offset))));
    }
}

#[test]
fn definitions_do_not_modify_namespace_resolution_of_scoped_selectors() {
    let source = format!(
        "@namespace svg 'urn:svg';@scope(svg|root){{svg|before{{}}{DEFINITIONS}bad|lost{{}}svg|after{{}}}}"
    );
    let report = parse_sheet(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one undeclared prefix: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("bad|lost").unwrap()
    );
    let [CssRule::Namespace(_), CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("namespace and scope")
    };
    let [
        CssScopedRule::Style(before),
        leaves @ ..,
        CssScopedRule::Style(after),
    ] = scope.rules().rules()
    else {
        panic!("neighboring styles")
    };
    assert_leaves(leaves);
    for (style, local_name) in [(before, "before"), (after, "after")] {
        let [CssScopedStyleSelector::Selector(CssSelector::Compound(selector))] =
            style.selectors().selectors()
        else {
            panic!("namespace-qualified selector")
        };
        let name = selector.type_selector().unwrap();
        assert!(
            matches!(name.namespace(), CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg")
        );
        assert_eq!(name.local_name(), Some(local_name));
    }
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(normalized_definitions(&normalized).len(), 3);
}

#[test]
fn scoped_definitions_retain_payload_under_eof_closure() {
    for definition in [
        "@counter-style Tick{system:cyclic;symbols:x",
        "@font-face{font-family:Audit;src:local(Audit)",
        "@keyframes audit{from{opacity:0}",
    ] {
        let source = format!("@scope(.host){{{definition}");
        let report = parse_sheet(&source);
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure),
            "{report:?}"
        );
        assert!(!report.diagnostics().is_empty());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let leaves = normalized_definitions(&normalized);
        assert_eq!(leaves.len(), 1);
        assert_eq!(
            leaves[0].position().unwrap().byte_offset().value(),
            source.find(definition).unwrap()
        );
    }
}

#[test]
fn scoped_definition_depth_ceiling_drops_only_the_excess_definition() {
    // 255 ordinary scopes plus one descriptor block exhaust the depth-256 budget.
    let prefix = "@scope{".repeat(255);
    let allowed = format!(
        "{prefix}@counter-style Tick{{symbols:x}}{}",
        "}".repeat(255)
    );
    let report = parse_sheet(&allowed);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(normalized_definitions(&normalized).len(), 1);
    let exceeded = format!(
        "{prefix}@scope{{@counter-style Tick{{symbols:x}}}}{}tail{{}}",
        "}".repeat(255)
    );
    let report = parse_sheet(&exceeded);
    let [diagnostic] = report.diagnostics() else {
        panic!("one depth failure: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert!(matches!(
        report.syntax().rules().last(),
        Some(CssRule::Style(_))
    ));
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert!(normalized_definitions(&normalized).is_empty());
}
