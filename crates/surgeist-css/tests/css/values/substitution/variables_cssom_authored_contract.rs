#![forbid(unsafe_code)]

//! Variables 1 CR20220616 §§3.2/4/4.1 defines pending-member serialization,
//! custom case and authored token fidelity. The public Rust requested-value
//! contract preserves absence/nonrepresentability as None, distinct from a
//! present empty terminal. CSSOM string APIs collapse those cases to empty text.

use surgeist_css::*;

fn declarations(source: &str) -> CssDeclarationList {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

fn read(block: &CssSpecifiedDeclarationBlock, property: CssKnownProperty) -> Option<String> {
    block
        .property_value(CssPropertyNameRef::Known(property))
        .unwrap()
}

fn custom_read(block: &CssSpecifiedDeclarationBlock, name: &str) -> Option<String> {
    let name = CssCustomPropertyName::try_new(name).unwrap();
    block
        .property_value(CssPropertyNameRef::Custom(&name))
        .unwrap()
}

fn original(entry: &CssSpecifiedDeclarationEntry, source: &CssDeclaration, input: &str) {
    assert!(entry.source().same_occurrence(source));
    assert_eq!(entry.source().importance(), source.importance());
    assert_eq!(entry.source().parser_context(), source.parser_context());
    assert_eq!(entry.source().value_components(), source.value_components());
    assert_eq!(entry.source().parsed_name(), source.parsed_name());
    assert_eq!(entry.source().parsed_value(), source.parsed_value());
    let name = entry.source().parsed_name().unwrap();
    let value = entry.source().parsed_value().unwrap();
    assert_eq!(name.source().as_str(), input);
    assert!(name.source().same_snapshot(value.source()));
}

#[test]
fn a_complete_original_pending_shorthand_has_empty_members_and_exact_retained_value() {
    const SOURCE: &str = "pause: \t/*Head*/ VaR(--P) /*Tail*/!important";
    const VALUE: &str = "/*Head*/ VaR(--P) /*Tail*/";
    let list = declarations(SOURCE);
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    assert_eq!(
        read(&block, CssKnownProperty::Pause).as_deref(),
        Some(VALUE)
    );
    assert_eq!(
        read(&block, CssKnownProperty::PauseBefore).as_deref(),
        Some("")
    );
    assert_eq!(
        read(&block, CssKnownProperty::PauseAfter).as_deref(),
        Some("")
    );
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "pause: /*Head*/ VaR(--P) /*Tail*/ !important;"
    );
    for entry in block.entries() {
        assert_eq!(entry.importance(), CssImportance::Important);
        let CssSpecifiedDeclarationValueRef::PendingShorthand(pending) = entry.value() else {
            panic!("a projected pending shorthand member")
        };
        assert!(pending.same_occurrence(&list[0]));
        original(entry, &list[0], SOURCE);
    }
}

#[test]
fn incomplete_or_mixed_pending_members_make_only_the_requested_shorthand_unrepresentable() {
    for (source, before, after, css) in [
        (
            "pause:var(--P);pause-after:1s",
            Some(""),
            Some("1s"),
            "pause-before: ; pause-after: 1s;",
        ),
        (
            "pause-before:var(--P);pause-after:1s",
            Some("var(--P)"),
            Some("1s"),
            "pause-before: var(--P); pause-after: 1s;",
        ),
        (
            "pause-before:var(--P);pause-after:var(--P)",
            Some("var(--P)"),
            Some("var(--P)"),
            "pause-before: var(--P); pause-after: var(--P);",
        ),
        (
            "pause-before:var(--P)",
            Some("var(--P)"),
            None,
            "pause-before: var(--P);",
        ),
    ] {
        let list = declarations(source);
        let original_list = list.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        assert_eq!(read(&block, CssKnownProperty::Pause), None, "{source}");
        assert_eq!(
            read(&block, CssKnownProperty::PauseBefore).as_deref(),
            before,
            "{source}"
        );
        assert_eq!(
            read(&block, CssKnownProperty::PauseAfter).as_deref(),
            after,
            "{source}"
        );
        assert_eq!(block.serialize_cssom().unwrap(), css, "{source}");
        assert_eq!(list, original_list);
    }
}

#[test]
fn priority_winners_reconstruct_from_the_selected_original_occurrence() {
    const SOURCE: &str = "pause:var(--Old)!important;pause:var(--New)";
    let list = declarations(SOURCE);
    assert!(!list[0].same_occurrence(&list[1]));
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    assert_eq!(
        read(&block, CssKnownProperty::Pause).as_deref(),
        Some("var(--Old)")
    );
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "pause: var(--Old) !important;"
    );
    for entry in block.entries() {
        assert_eq!(entry.authored_ordinal(), 0);
        original(entry, &list[0], SOURCE);
        assert!(!entry.source().same_occurrence(&list[1]));
    }

    let changed = list[0].with_importance(CssImportance::Normal);
    assert!(!changed.same_occurrence(&list[0]));
    assert_eq!(changed.value_components(), list[0].value_components());
    assert_eq!(changed.parsed_name(), list[0].parsed_name());
    assert_eq!(changed.parsed_value(), list[0].parsed_value());
    assert_eq!(changed.parser_context(), list[0].parser_context());
    assert_eq!(changed.to_specified_css().unwrap(), "pause: var(--Old);");
    assert_eq!(
        list[0].to_specified_css().unwrap(),
        "pause: var(--Old) !important;"
    );
}

#[test]
fn exact_custom_names_and_token_spelling_survive_priority_selection() {
    const SOURCE: &str = "--Case: \t/*H*/01.00PX  A/**/B/*T*/!important;--case:C;--Case:discard";
    const VALUE: &str = "/*H*/01.00PX  A/**/B/*T*/";
    let list = declarations(SOURCE);
    assert!(list.iter().all(CssDeclaration::is_name_case_sensitive));
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    assert_eq!(custom_read(&block, "--Case").as_deref(), Some(VALUE));
    assert_eq!(custom_read(&block, "--case").as_deref(), Some("C"));
    assert_eq!(custom_read(&block, "--CASE"), None);
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "--Case: /*H*/01.00PX  A/**/B/*T*/ !important; --case: C;"
    );
    let [upper, lower] = block.entries() else {
        panic!("two case-distinct winners")
    };
    original(upper, &list[0], SOURCE);
    original(lower, &list[1], SOURCE);
    assert_eq!(upper.importance(), CssImportance::Important);
    assert_eq!(lower.importance(), CssImportance::Normal);
    assert_eq!(upper.source().custom().unwrap().name().as_str(), "--Case");
    assert_eq!(lower.source().custom().unwrap().name().as_str(), "--case");
    assert!(
        upper
            .source()
            .parsed_name()
            .unwrap()
            .source()
            .same_snapshot(lower.source().parsed_name().unwrap().source())
    );
}

#[test]
fn valid_empty_custom_text_is_distinct_from_absence_and_symbolic_initial() {
    let list = declarations("--Empty: ;--Reset:INITIAL");
    let empty = list[0].custom().unwrap().value();
    let reset = list[1].custom().unwrap().value();
    assert!(empty.value().unwrap().is_empty());
    assert_eq!(empty.global(), None);
    assert_eq!(reset.global(), Some(CssGlobalKeyword::Initial));
    assert!(reset.value().is_none());
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    assert_eq!(custom_read(&block, "--Empty").as_deref(), Some(""));
    assert_eq!(custom_read(&block, "--Absent"), None);
    assert_eq!(custom_read(&block, "--Reset").as_deref(), Some("INITIAL"));
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "--Empty: ; --Reset: INITIAL;"
    );
}

#[test]
fn a_bounded_requested_pending_value_can_retry_without_changing_its_source() {
    const SOURCE: &str = "pause:var(--P)";
    const VALUE: &str = "var(--P)";
    let list = declarations(SOURCE);
    let original_source = list[0].clone();
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    let exact = CssSpecifiedValueSerializationLimits::new(10000, 10000, VALUE.len());
    assert_eq!(
        block
            .property_value_with_limits(CssPropertyNameRef::Known(CssKnownProperty::Pause), exact)
            .unwrap()
            .as_deref(),
        Some(VALUE)
    );
    let short = CssSpecifiedValueSerializationLimits::new(10000, 10000, VALUE.len() - 1);
    let error = block
        .property_value_with_limits(CssPropertyNameRef::Known(CssKnownProperty::Pause), short)
        .unwrap_err();
    let CssDeclarationBlockErrorKind::Serialization(cause) = error.kind() else {
        panic!("a typed specified byte bound")
    };
    assert_eq!(
        cause.kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        block
            .property_value_with_limits(CssPropertyNameRef::Known(CssKnownProperty::Pause), exact)
            .unwrap()
            .as_deref(),
        Some(VALUE)
    );
    assert!(list[0].same_occurrence(&original_source));
    for entry in block.entries() {
        original(entry, &original_source, SOURCE);
    }
}
