#![forbid(unsafe_code)]
//! Conditional3 section 3 permits unrestricted rules in ordinary conditional lists.
//! Cascade6 admits global name definitions inside ordinary scope; Nesting1
//! does not admit descriptor-body definitions beneath style-rule ancestors.
//! https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/
//! https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/#the-counter-style-rule
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-nesting
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting-other-at-rules
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
//! Exercises authored placement without query evaluation or definition lookup.
use surgeist_css::*;

const COUNTER: &str = "@counter-style Tick { system: cyclic; symbols: \"x\"; }";
const FONT: &str = "@font-face { font-family: Audit; src: local(Audit); }";
const KEYFRAMES: &str = "@keyframes audit { from { opacity: 0 } to { opacity: 1 } }";

fn kind_name(kind: CssRuleContextKindRef<'_>) -> &'static str {
    match kind {
        CssRuleContextKindRef::Style(_) | CssRuleContextKindRef::ScopedStyle(_) => "style",
        CssRuleContextKindRef::Media(_) => "media",
        CssRuleContextKindRef::Supports(_) => "supports",
        CssRuleContextKindRef::LayerBlock(_) => "layer",
        CssRuleContextKindRef::Container { .. } => "container",
        CssRuleContextKindRef::Scope { .. } => "scope",
        CssRuleContextKindRef::CounterStyle(_) => "counter",
        CssRuleContextKindRef::FontFace(_) => "font",
        CssRuleContextKindRef::Keyframes(_) => "keyframes",
        _ => "other",
    }
}

fn rule_contexts(sheet: &CssNormalizedSheet) -> Vec<&CssRuleContext> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => Some(context),
            _ => None,
        })
        .collect()
}

fn assert_position(source: &str, marker: &str, position: CssSourcePosition) {
    let offset = source.find(marker).unwrap();
    assert_eq!(position.byte_offset().value(), offset);
    let before = &source[..offset];
    assert_eq!(
        position.line().value() as usize,
        before.bytes().filter(|b| *b == b'\n').count()
    );
    assert_eq!(
        position.column().value() as usize,
        before.rsplit('\n').next().unwrap().encode_utf16().count()
    );
}

fn assert_payload(context: &CssRuleContext, source: &str, expected: &str) {
    assert_eq!(kind_name(context.kind()), expected);
    let (marker, position) = match context.kind() {
        CssRuleContextKindRef::CounterStyle(rule) => {
            assert_eq!(rule.name().as_str(), "Tick");
            assert!(matches!(
                rule.descriptors().system().unwrap().value(),
                CssCounterStyleSystem::Cyclic
            ));
            assert!(
                matches!(rule.descriptors().symbols().unwrap().symbols(), [CssCounterSymbol::String(symbol)] if symbol.as_str() == "x")
            );
            assert_position(
                source,
                "system:",
                rule.descriptors().system().unwrap().position(),
            );
            ("@counter-style", rule.position())
        }
        CssRuleContextKindRef::FontFace(rule) => {
            let family = rule.descriptors().font_family().unwrap();
            assert_eq!(family.value().as_str(), "Audit");
            assert!(
                matches!(rule.descriptors().src().unwrap().value().sources(), [CssFontFaceSource::Local(name)] if name.as_str() == "Audit")
            );
            assert_position(source, "font-family:", family.position());
            ("@font-face", rule.position())
        }
        CssRuleContextKindRef::Keyframes(rule) => {
            assert!(
                matches!(rule.name(), CssKeyframesName::Ident(name) if name.as_str() == "audit")
            );
            assert_eq!(rule.blocks().len(), 2);
            for block in rule.blocks() {
                let [declaration] = block.declarations().as_slice() else {
                    panic!("one keyframe declaration")
                };
                assert_eq!(
                    declaration.known().unwrap().property(),
                    CssKnownProperty::Opacity
                );
            }
            assert_position(source, "from", rule.blocks()[0].position());
            assert_position(source, "to {", rule.blocks()[1].position());
            ("@keyframes", rule.position())
        }
        _ => panic!("intact definition payload"),
    };
    assert_position(source, marker, position);
    assert_eq!(context.position(), Some(position));
}

fn assert_wrapped_definition(
    prefix: &str,
    suffix: &str,
    groups: &[&str],
    definition: &str,
    expected: &str,
) {
    let source = format!("/*😀*/\n{prefix}before{{}} {definition} after{{}}{suffix}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let contexts = rule_contexts(&normalized);
    let mut expected_kinds = groups.to_vec();
    expected_kinds.extend(["style", expected, "style"]);
    assert_eq!(
        contexts
            .iter()
            .map(|c| kind_name(c.kind()))
            .collect::<Vec<_>>(),
        expected_kinds
    );
    for index in 0..groups.len() {
        if index == 0 {
            assert!(contexts[index].parent().is_none());
        } else {
            assert!(
                contexts[index]
                    .parent()
                    .unwrap()
                    .same_context(contexts[index - 1])
            );
        }
    }
    let children = &contexts[groups.len()..];
    for child in children {
        assert!(
            child
                .parent()
                .unwrap()
                .same_context(contexts[groups.len() - 1])
        );
    }
    assert_position(&source, "before{}", children[0].position().unwrap());
    assert_payload(children[1], &source, expected);
    assert_position(&source, "after{}", children[2].position().unwrap());
}

#[test]
fn ordinary_groups_retain_counter_definition_payload_and_neighbors() {
    for (prefix, suffix, groups) in [
        ("@media print{", "}", &["media"][..]),
        ("@supports (display:grid){", "}", &["supports"][..]),
        ("@layer base{", "}", &["layer"][..]),
        ("@container (width > 1px){", "}", &["container"][..]),
        (
            "@media print{@supports (display:grid){",
            "}}",
            &["media", "supports"][..],
        ),
    ] {
        assert_wrapped_definition(prefix, suffix, groups, COUNTER, "counter");
    }
}

fn assert_scoped_definition(definition: &str, expected: &str) {
    for (prefix, suffix, groups) in [
        ("@scope (.host){", "}", &["scope"][..]),
        (
            "@scope (.host){@media print{",
            "}}",
            &["scope", "media"][..],
        ),
        (
            "@supports (display:grid){@scope (.host){",
            "}}",
            &["supports", "scope"][..],
        ),
        ("@scope (.host){@layer base{", "}}", &["scope", "layer"][..]),
        (
            "@scope (.host){@container (width > 1px){",
            "}}",
            &["scope", "container"][..],
        ),
        (
            "@scope (.host){@scope (.child){",
            "}}",
            &["scope", "scope"][..],
        ),
    ] {
        assert_wrapped_definition(prefix, suffix, groups, definition, expected);
    }
}

#[test]
fn ordinary_scope_retains_counter_definition_through_group_combinations() {
    assert_scoped_definition(COUNTER, "counter");
}

#[test]
fn ordinary_scope_retains_font_face_through_group_combinations() {
    assert_scoped_definition(FONT, "font");
}

#[test]
fn ordinary_scope_retains_keyframes_through_group_combinations() {
    assert_scoped_definition(KEYFRAMES, "keyframes");
}

#[test]
fn admitted_counter_recovers_bad_descriptor_without_dropping_definition_or_neighbors() {
    for prefix in ["@media all", "@scope (.host)"] {
        let source = format!(
            "{prefix}{{before{{}}@counter-style Tick{{system:cyclic;unknown-descriptor:0;symbols:\"x\"}}after{{}}}}"
        );
        let report = parse_sheet(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one local recovery: {report:?}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_position(&source, "unknown-descriptor:", diagnostic.span().start());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let contexts = rule_contexts(&normalized);
        assert_eq!(contexts.len(), 4);
        assert_position(&source, "before{}", contexts[1].position().unwrap());
        assert_payload(contexts[2], &source, "counter");
        assert_position(&source, "after{}", contexts[3].position().unwrap());
    }
}

#[test]
fn scoped_font_and_keyframes_keep_valid_payload_after_local_recovery() {
    for (definition, expected, action, marker) in [
        (
            FONT.replace("src:", "unknown-descriptor: 0; src:"),
            "font",
            CssRecoveryAction::DropDescriptor,
            "unknown-descriptor:",
        ),
        (
            KEYFRAMES.replace("opacity: 0", "unknown-property: 0; opacity: 0"),
            "keyframes",
            CssRecoveryAction::DropDeclaration,
            "unknown-property:",
        ),
    ] {
        let source = format!("@scope (.host){{before{{}}{definition}after{{}}}}");
        let report = parse_sheet(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one local recovery: {report:?}")
        };
        assert_eq!(diagnostic.action(), action);
        assert_position(&source, marker, diagnostic.span().start());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let contexts = rule_contexts(&normalized);
        assert_eq!(contexts.len(), 4);
        assert_position(&source, "before{}", contexts[1].position().unwrap());
        assert_payload(contexts[2], &source, expected);
        assert_position(&source, "after{}", contexts[3].position().unwrap());
    }
}

#[test]
fn style_ancestors_still_reject_definitions_through_nested_scope_and_groups() {
    for definition in [COUNTER, FONT, KEYFRAMES] {
        for (prefix, suffix) in [
            ("host{", "}"),
            ("host{@media all{", "}}"),
            ("host{@scope{", "}}"),
            (
                "@scope (.root){host{@scope{@supports (display:grid){",
                "}}}}",
            ),
        ] {
            let source = format!("{prefix}before{{}}{definition}after{{}}{suffix}");
            let report = parse_sheet(&source);
            let [diagnostic] = report.diagnostics() else {
                panic!("one misplaced definition: {report:?}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
            let normalized = normalize_sheet(report.syntax()).unwrap();
            let contexts = rule_contexts(&normalized);
            assert!(!contexts.iter().any(|c| matches!(
                c.kind(),
                CssRuleContextKindRef::CounterStyle(_)
                    | CssRuleContextKindRef::FontFace(_)
                    | CssRuleContextKindRef::Keyframes(_)
            )));
            for marker in ["before{}", "after{}"] {
                let child = contexts
                    .iter()
                    .find(|c| {
                        c.position().is_some_and(|p| {
                            p.byte_offset().value() == source.find(marker).unwrap()
                        })
                    })
                    .unwrap();
                assert_eq!(kind_name(child.kind()), "style");
            }
        }
    }
}

#[test]
fn reserved_definition_names_drop_locally_at_existing_and_new_routes() {
    for invalid in [
        "@counter-style none{system:cyclic;symbols:x}",
        "@keyframes none{from{opacity:0}}",
    ] {
        for (prefix, suffix) in [("", ""), ("@media all{", "}"), ("@scope (.host){", "}")] {
            let source = format!("{prefix}before{{}}{invalid}after{{}}{suffix}");
            let report = parse_sheet(&source);
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid name: {report:?}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
            let normalized = normalize_sheet(report.syntax()).unwrap();
            let contexts = rule_contexts(&normalized);
            assert!(!contexts.iter().any(|c| matches!(
                c.kind(),
                CssRuleContextKindRef::CounterStyle(_) | CssRuleContextKindRef::Keyframes(_)
            )));
            let styles: Vec<_> = contexts
                .iter()
                .filter(|c| kind_name(c.kind()) == "style")
                .collect();
            assert_eq!(styles.len(), 2);
            assert_position(&source, "before{}", styles[0].position().unwrap());
            assert_position(&source, "after{}", styles[1].position().unwrap());
        }
    }
}

#[test]
fn nested_definition_admission_does_not_reopen_import_or_namespace_phase() {
    let source = format!(
        "@import 'early.css';@namespace svg 'urn:svg';@media all{{{COUNTER}}}@import 'late.css';@namespace late 'urn:late';tail{{}}"
    );
    let report = parse_sheet(&source);
    let [
        CssRule::Import(_),
        CssRule::Namespace(_),
        CssRule::Media(media),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!("phase-preserving root rules: {report:?}")
    };
    assert!(matches!(media.rules(), [CssRule::CounterStyle(_)]));
    assert_eq!(report.diagnostics().len(), 2);
    for (diagnostic, marker) in report
        .diagnostics()
        .iter()
        .zip(["@import 'late.css'", "@namespace late"])
    {
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_position(&source, marker, diagnostic.span().start());
    }
}
