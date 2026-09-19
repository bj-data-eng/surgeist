#![forbid(unsafe_code)]
//! New authored prelude API contracts: ordered independent entries from pinned
//! Conditional Rules 5 section 5.4, and shared component provenance/limits.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssContainerConditionKind, CssContainerConstructionError, CssContainerName,
    CssContainerPrelude, CssNormalizedItem, CssRule, CssRuleContextKindRef, CssSerializedOrigin,
    CssValueOrigin, CssValueTokenRef, normalize_sheet, parse_component_values, parse_sheet,
};

fn construct(source: &str) -> CssContainerPrelude {
    CssContainerPrelude::try_from_components(parse_component_values(source).unwrap()).unwrap()
}

#[test]
fn entries_retain_independent_names_queries_duplicates_and_order() {
    let prelude = construct("Card, (width:1px), card style(--theme), Card");
    let entries = prelude.entries();
    assert_eq!(entries.len(), 4);
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.name().map(CssContainerName::as_str))
            .collect::<Vec<_>>(),
        [Some("Card"), None, Some("card"), Some("Card")]
    );
    assert!(entries[0].query().is_none());
    assert!(entries[3].query().is_none());
    assert!(matches!(
        entries[1].query().unwrap().kind(),
        CssContainerConditionKind::Feature(_)
    ));
    assert!(matches!(
        entries[2].query().unwrap().kind(),
        CssContainerConditionKind::Style(_)
    ));
    assert_eq!(entries[1].serialize().unwrap().as_css(), " (width:1px)");
    assert_eq!(
        entries[2].serialize().unwrap().as_css(),
        " card style(--theme)"
    );
}

#[test]
fn parsed_and_checked_preludes_share_components_and_original_token_origins() {
    let source = r"@container C\61 rd /*left*/, /*right*/ style(--tokens: a,b) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("container")
    };
    let prelude = rule.prelude();
    let components = CssComponentValues::try_new(prelude.components().to_vec()).unwrap();
    let checked = CssContainerPrelude::try_from_components(components).unwrap();
    assert_eq!(&checked, prelude);
    assert_eq!(prelude.entries()[0].name().unwrap().as_str(), "Card");
    assert_eq!(
        prelude.position().unwrap().byte_offset().value(),
        source.find(r"C\61 rd").unwrap()
    );
    assert_eq!(
        prelude.entries()[1]
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        source.find("style(").unwrap()
    );
    let commas: Vec<_> = prelude
        .components()
        .iter()
        .filter(|component| {
            matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Comma)
            )
        })
        .collect();
    assert_eq!(commas.len(), 1, "nested comma belongs to the function");
    let serialized = prelude.serialize().unwrap();
    let comma_offset = serialized.as_css().find(',').unwrap();
    assert_eq!(
        serialized.origin_at(comma_offset),
        Some(&CssSerializedOrigin::Token(commas[0].origin().clone()))
    );
    assert!(serialized.as_css().contains("/*left*/, /*right*/"));
    assert!(serialized.as_css().contains("style(--tokens: a,b)"));
    let reparsed = parse_sheet(&format!("@container{}{{}}", serialized.as_css()));
    assert!(reparsed.is_clean(), "{reparsed:?}");
    let [CssRule::Container(reparsed)] = reparsed.syntax().rules() else {
        panic!("container")
    };
    assert_eq!(
        reparsed.prelude().serialize().unwrap().as_css(),
        serialized.as_css()
    );
}

#[test]
fn mixed_origins_and_programmatic_escaped_names_serialize_without_reinterpretation() {
    let name = CssComponentValue::try_ident("1 pane,\\Card").unwrap();
    let comma = CssComponentValue::try_token(",").unwrap();
    let parsed = parse_component_values("style(--tokens: a,b)").unwrap();
    let parsed_origin = parsed.items()[0].origin().clone();
    let mut items = vec![name, comma];
    items.extend_from_slice(parsed.items());
    let prelude =
        CssContainerPrelude::try_from_components(CssComponentValues::try_new(items).unwrap())
            .unwrap();
    assert_eq!(
        prelude.entries()[0].name().unwrap().as_str(),
        "1 pane,\\Card"
    );
    assert_eq!(prelude.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(prelude.entries()[0].position(), None);
    assert_eq!(prelude.entries()[1].origin(), &parsed_origin);
    let serialized = prelude.serialize().unwrap();
    assert_eq!(
        serialized.origin_at(0),
        Some(&CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
    );
    let style_offset = serialized.as_css().find("style(").unwrap();
    assert_eq!(
        serialized.origin_at(style_offset),
        Some(&CssSerializedOrigin::Token(parsed_origin))
    );
    let rebuilt = construct(serialized.as_css());
    assert_eq!(rebuilt.entries().len(), 2);
    assert_eq!(
        rebuilt.entries()[0].name().unwrap().as_str(),
        "1 pane,\\Card"
    );
    assert!(rebuilt.entries()[0].query().is_none());
    assert!(matches!(
        rebuilt.entries()[1].query().unwrap().kind(),
        CssContainerConditionKind::Style(_)
    ));
}

#[test]
fn malformed_complete_preludes_return_typed_failure_and_the_offending_origin() {
    for source in [
        "",
        " ",
        ",card",
        "card,",
        "card,,other",
        "card other",
        "default",
        "none",
        "card,(width:1px) junk",
    ] {
        let values = parse_component_values(source).unwrap();
        let Err(CssContainerConstructionError::InvalidPreludeGrammar { .. }) =
            CssContainerPrelude::try_from_components(values)
        else {
            panic!("invalid full prelude: {source}")
        };
    }
    let values = parse_component_values("card,!").unwrap();
    let expected = values.items().last().unwrap().origin().clone();
    let error = CssContainerPrelude::try_from_components(values).unwrap_err();
    assert_eq!(error.origin(), &expected);
    let values =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("default").unwrap()])
            .unwrap();
    assert!(matches!(
        CssContainerPrelude::try_from_components(values),
        Err(CssContainerConstructionError::InvalidPreludeGrammar {
            origin: CssValueOrigin::Programmatic
        })
    ));
}

#[test]
fn full_prelude_limits_count_all_entries_commas_and_nested_components() {
    let source = "Card,(width:1px),style(--theme)";
    let components = parse_component_values(source).unwrap();
    let exact = CssComponentValueLimits::try_new(
        components.nesting_depth(),
        components.component_count(),
        source.len(),
    )
    .unwrap();
    let prelude =
        CssContainerPrelude::try_from_components_with_limits(components.clone(), exact).unwrap();
    assert_eq!(
        prelude.serialize_with_limit(source.len()).unwrap().as_css(),
        source
    );
    assert_eq!(
        prelude
            .serialize_with_limit(source.len() - 1)
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    let entry = &prelude.entries()[1];
    assert_eq!(
        entry
            .serialize_with_limit("(width:1px)".len())
            .unwrap()
            .as_css(),
        "(width:1px)"
    );
    assert_eq!(
        entry
            .serialize_with_limit("(width:1px)".len() - 1)
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, components.component_count() - 1, usize::MAX)
                .unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, source.len() - 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let Err(CssContainerConstructionError::Component(error)) =
            CssContainerPrelude::try_from_components_with_limits(components.clone(), limits)
        else {
            panic!("whole input limit")
        };
        assert_eq!(error.kind(), kind);
    }
    // Resource admission precedes grammar classification, even for invalid input.
    let limits = CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap();
    assert!(matches!(
        CssContainerPrelude::try_from_components_with_limits(
            parse_component_values("default").unwrap(),
            limits
        ),
        Err(CssContainerConstructionError::Component(_))
    ));
}

#[test]
fn normalization_preserves_each_complete_prelude_once_without_duplicating_bodies() {
    for source in [
        "@container card, (width:1px), card { .child { color:red } }",
        "@scope (.host) { @container card, (width:1px), card { .child { color:red } } }",
        ".host { @container card, (width:1px), card { .child { color:red } } }",
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{report:?}");
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let mut containers = Vec::new();
        let mut declarations = Vec::new();
        for item in normalized.items() {
            match item {
                CssNormalizedItem::Rule(context) => {
                    if let CssRuleContextKindRef::Container { prelude } = context.kind() {
                        containers.push((context, prelude));
                    }
                }
                CssNormalizedItem::Declaration(declaration) => declarations.push(declaration),
                _ => panic!("only authored rules and color declaration"),
            }
        }
        let [(container, prelude)] = containers.as_slice() else {
            panic!("one complete container context")
        };
        assert_eq!(prelude.entries().len(), 3);
        assert_eq!(prelude.entries()[0].name().unwrap().as_str(), "card");
        assert!(prelude.entries()[1].name().is_none());
        assert_eq!(prelude.entries()[2].name().unwrap().as_str(), "card");
        let [declaration] = declarations.as_slice() else {
            panic!("one body contribution")
        };
        assert!(
            declaration
                .rule_context()
                .parent()
                .unwrap()
                .same_context(container)
        );
        let preserved = prelude.serialize().unwrap().as_css().to_owned();
        drop(report);
        assert_eq!(prelude.serialize().unwrap().as_css(), preserved);
    }
}

#[test]
fn detached_entry_equality_and_serialization_ignore_other_entries() {
    let head = parse_component_values("card (width:1px)").unwrap();
    let build = |tail: &str| {
        let mut items = head.items().to_vec();
        items.push(CssComponentValue::try_token(",").unwrap());
        items.extend_from_slice(parse_component_values(tail).unwrap().items());
        CssContainerPrelude::try_from_components(CssComponentValues::try_new(items).unwrap())
            .unwrap()
    };
    let one = build("alpha");
    let two = build("beta");
    assert_ne!(one, two);
    let entry = one.entries()[0].clone();
    assert_eq!(entry, two.entries()[0]);
    let query = entry.query().unwrap().clone();
    drop(one);
    drop(two);
    assert_eq!(entry.serialize().unwrap().as_css(), "card (width:1px)");
    drop(entry);
    assert_eq!(query.serialize().unwrap().as_css(), "(width:1px)");
}

#[test]
fn parser_recovered_closures_keep_their_provenance_through_checked_preludes() {
    // Public component parsing supplies trusted EOF closures; construction must
    // preserve that evidence, while emitted CSS explicitly closes each group.
    let components = parse_component_values("card,((width:1px").unwrap();
    let prelude = CssContainerPrelude::try_from_components(components).unwrap();
    let serialized = prelude.serialize().unwrap();
    assert_eq!(serialized.as_css(), "card,((width:1px))");
    for offset in [16, 17] {
        assert!(matches!(
            serialized.origin_at(offset),
            Some(CssSerializedOrigin::Token(
                CssValueOrigin::ImplicitClosure { .. }
            ))
        ));
    }
}

#[test]
fn incomplete_queries_keep_the_parser_end_of_prelude_diagnostic_position() {
    for prelude in ["not", "card not", "(width:1px) and", "card, not"] {
        let source = format!("@container {prelude} {{}} .after {{}}");
        let report = parse_sheet(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one incomplete-query diagnostic: {source}: {report:?}");
        };
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find('{').unwrap()
        );
        assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    }
}

#[test]
fn full_depth_prelude_lifecycle_is_safe_on_an_ordinary_stack() {
    const CHILD: &str = "SURGEIST_CONTAINER_PRELUDE_STACK_CHILD";
    const TEST: &str = "full_depth_prelude_lifecycle_is_safe_on_an_ordinary_stack";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                for query in [
                    format!("{}width:1px{}", "(".repeat(256), ")".repeat(256)),
                    format!("{}(width:1px){}", "not (".repeat(255), ")".repeat(255)),
                    format!("{}1{}", "future(".repeat(256), ")".repeat(256)),
                ] {
                    let source = format!("card,{query},last");
                    let prelude = construct(&source);
                    let clone = prelude.clone();
                    assert_eq!(prelude, clone);
                    let entry = prelude.entries()[1].clone();
                    let condition = entry.query().unwrap().clone();
                    drop(prelude);
                    assert_eq!(clone.serialize().unwrap().as_css(), source);
                    drop(clone);
                    assert_eq!(entry.serialize().unwrap().as_css(), query);
                    drop(entry);
                    assert_eq!(condition.serialize().unwrap().as_css(), query);
                    drop(condition);
                }
            })
            .unwrap()
            .join()
            .unwrap();
        println!("prelude owned lifecycle completed");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD, TEST)
        .env_remove("RUST_MIN_STACK")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("prelude owned lifecycle completed"));
}
