#![forbid(unsafe_code)]
use surgeist_css::CssSerializationLimits;
use surgeist_cssom::*;

fn limits() -> CssSerializationLimits {
    CssSerializationLimits::default()
}
fn fixture(source: &str) -> (CssomStore, CssomSheetId, Vec<CssomRuleId>) {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let ticket = batch
        .create_parsed_sheet(
            source,
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: "sheet".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let rules = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()
        .to_vec();
    (store, sheet, rules)
}

#[test]
fn style_same_object_and_captured_selector_and_parent_chains() {
    let (mut store, _, roots) = fixture(".old { & .child {} }");
    let style = store.css_style_rule(&roots[0]).unwrap();
    let old = style.capture(&store.snapshot()).unwrap();
    let properties: CssStyleProperties = style.style(&store).unwrap();
    assert_eq!(properties, style.style(&store).unwrap());
    assert_eq!(old.style().unwrap().handle(), &properties);
    let child = old.css_rules().unwrap().item(0).unwrap().unwrap();
    assert_eq!(
        child.parent_rule().unwrap().unwrap().handle().id(),
        style.id()
    );
    let changed = style.set_selector_text(&mut store, ".new").unwrap();
    let _: () = *changed.value();
    assert_eq!(style.selector_text(&store, limits()).unwrap(), ".new");
    assert_eq!(old.selector_text(limits()).unwrap(), ".old");
    assert_eq!(style.style(&store).unwrap(), properties);
    let captured_parent = child
        .parent_rule()
        .unwrap()
        .unwrap()
        .as_style_rule()
        .unwrap()
        .unwrap();
    drop(store);
    assert_eq!(captured_parent.selector_text(limits()).unwrap(), ".old");
}

#[test]
fn grouping_default_index_and_atomic_errors_and_detached_captures() {
    let (mut store, sheet, roots) = fixture("@media all { a {} }");
    let media = store.css_media_rule(&roots[0]).unwrap();
    let list = media.css_rules(&store).unwrap();
    let old_list = media
        .capture(&store.snapshot())
        .unwrap()
        .css_rules()
        .unwrap();
    let outcome = media.insert_rule(&mut store, "b {}", None).unwrap();
    assert_eq!(*outcome.value(), 0);
    assert_eq!(list, media.css_rules(&store).unwrap());
    assert_eq!(list.length(&store).unwrap(), 2);
    assert_eq!(old_list.length().unwrap(), 1);
    let first = list.item(&store, 0).unwrap().unwrap();
    assert_eq!(
        first
            .as_style_rule(&store)
            .unwrap()
            .unwrap()
            .selector_text(&store, limits())
            .unwrap(),
        "b"
    );
    let revision = store.snapshot().revision().clone();
    assert!(matches!(
        media.insert_rule(&mut store, "garbage", Some(usize::MAX)),
        Err(CssomError::Source(CssomException::IndexSize))
    ));
    assert_eq!(store.snapshot().revision(), &revision);
    let deleted = media.delete_rule(&mut store, 0).unwrap();
    let _: () = *deleted.value();
    assert_eq!(first.parent_rule(&store).unwrap(), None);
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    batch.delete_sheet_rule(&sheet, 0).unwrap();
    store.commit(batch).unwrap();
    assert_eq!(media.parent_style_sheet(&store).unwrap(), None);
    assert_eq!(
        old_list
            .item(0)
            .unwrap()
            .unwrap()
            .parent_style_sheet()
            .unwrap()
            .unwrap()
            .handle()
            .id(),
        &sheet
    );
}

#[test]
fn container_readonly_plural_native_output_and_fresh_conditions() {
    let (store, _, roots) = fixture("@container alpha, (width > 1px), alpha, beta future(foo){}");
    let rule = store.css_container_rule(&roots[0]).unwrap();
    let text = "alpha, (width > 1px), alpha, beta future(foo)";
    let exact = CssSerializationLimits::new(1000, 1000, text.len());
    assert_eq!(rule.condition_text(&store, exact).unwrap(), text);
    assert_eq!(
        rule.as_condition_rule()
            .condition_text(&store, exact)
            .unwrap(),
        text
    );
    assert!(
        rule.condition_text(
            &store,
            CssSerializationLimits::new(1000, 1000, text.len() - 1)
        )
        .is_err()
    );
    assert_eq!(rule.container_name(&store, 0).unwrap(), "");
    assert_eq!(rule.container_query(&store, 0).unwrap(), "");
    let captured = rule.capture(&store.snapshot()).unwrap();
    let mut first = captured.conditions(1000).unwrap();
    first[0].name = "changed transport".into();
    assert_eq!(captured.conditions(1000).unwrap()[0].name, "alpha");
    assert_eq!(captured.condition_text(exact).unwrap(), text);
}

#[test]
fn rule_specific_declaration_facets_and_keyframes_distinct_index_and_last_match() {
    let (mut store, _, roots) = fixture(
        "@page :left { @top-left { content: 'x' } } @keyframes pulse { from { opacity: 0 } 0% { opacity: 1 } }",
    );
    let page = store.css_page_rule(&roots[0]).unwrap();
    let _: CssPageDescriptors = page.style(&store).unwrap();
    let captured_page = page.capture(&store.snapshot()).unwrap();
    let _: CssomCapture<CssPageDescriptors> = captured_page.style().unwrap();
    let margin = captured_page
        .css_rules()
        .unwrap()
        .item(0)
        .unwrap()
        .unwrap()
        .as_margin_rule()
        .unwrap()
        .unwrap();
    assert_eq!(margin.name().unwrap(), "top-left");
    let _: CssomCapture<CssMarginDescriptors> = margin.style().unwrap();
    let _ = page.set_selector_text(&mut store, ":right").unwrap();
    assert_eq!(page.selector_text(&store, limits()).unwrap(), ":right");
    assert_eq!(captured_page.selector_text(limits()).unwrap(), ":left");
    let keyframes = store.css_keyframes_rule(&roots[1]).unwrap();
    let old = keyframes.capture(&store.snapshot()).unwrap();
    assert!(matches!(
        keyframes.indexed(&store, 999).unwrap(),
        CssomKeyframeIndex::Undefined
    ));
    assert!(
        keyframes
            .css_rules(&store)
            .unwrap()
            .item(&store, 999)
            .unwrap()
            .is_none()
    );
    let last = keyframes
        .find_rule(&store, "from", &CssomLimits::default())
        .unwrap()
        .unwrap();
    let _: CssStyleDeclaration = last.style(&store).unwrap();
    let _: CssomCapture<CssStyleDeclaration> =
        last.capture(&store.snapshot()).unwrap().style().unwrap();
    match keyframes.indexed(&store, 1).unwrap() {
        CssomKeyframeIndex::Rule(rule) => assert_eq!(rule, last),
        _ => panic!("present index"),
    }
    assert_eq!(last.key_text(&store, limits()).unwrap(), "0%");
    let _ = keyframes.set_name(&mut store, "other").unwrap();
    assert_eq!(keyframes.name(&store).unwrap(), "other");
    assert_eq!(old.name().unwrap(), "pulse");
    let _ = keyframes.delete_rule(&mut store, "0%").unwrap();
    assert_eq!(keyframes.length(&store).unwrap(), 1);
    assert_ne!(
        keyframes
            .find_rule(&store, "from", &CssomLimits::default())
            .unwrap()
            .unwrap(),
        last
    );
    assert_eq!(
        old.find_rule("from", &CssomLimits::default())
            .unwrap()
            .unwrap()
            .handle(),
        &last
    );
    let _ = keyframes
        .append_rule(&mut store, "50% { opacity: .5 }")
        .unwrap();
    let middle = keyframes
        .find_rule(&store, "50%", &CssomLimits::default())
        .unwrap()
        .unwrap();
    let _ = middle.set_key_text(&mut store, "to").unwrap();
    assert_eq!(middle.key_text(&store, limits()).unwrap(), "100%");
    let revision = store.snapshot().revision().clone();
    let _ = keyframes.append_rule(&mut store, "garbage").unwrap();
    assert_eq!(store.snapshot().revision(), &revision);
    assert!(matches!(
        middle.set_key_text(&mut store, "garbage"),
        Err(CssomError::Source(CssomException::Syntax))
    ));
    assert_eq!(store.snapshot().revision(), &revision);
    drop(store);
    assert_eq!(old.length().unwrap(), 2);
    assert!(matches!(
        old.indexed(2).unwrap(),
        CssomKeyframeIndex::Undefined
    ));
}

#[test]
fn scalar_and_object_member_inventory_reads_retained_current_state() {
    let (store, _, roots) = fixture(
        "@import 'theme.css' layer(theme) supports(display: grid) screen; @namespace svg 'urn:svg'; @layer one, two; @layer local {} @scope (.start) to (.end) {} @supports-condition --ready { color: red; }",
    );
    assert_eq!(roots.len(), 6);
    let import = store.css_import_rule(&roots[0]).unwrap();
    assert_eq!(import.href(&store).unwrap(), "theme.css");
    assert_eq!(
        import.layer_name(&store, 100).unwrap().as_deref(),
        Some("theme")
    );
    assert!(import.supports_text(&store, 100).unwrap().is_some());
    assert_eq!(import.media(&store).unwrap(), import.media(&store).unwrap());
    assert!(import.style_sheet(&store).unwrap().is_none());
    let namespace = store.css_namespace_rule(&roots[1]).unwrap();
    assert_eq!(namespace.namespace_uri(&store).unwrap(), "urn:svg");
    assert_eq!(namespace.prefix(&store).unwrap(), "svg");
    let statement = store.css_layer_statement_rule(&roots[2]).unwrap();
    assert_eq!(statement.name_list(&store, 100).unwrap(), ["one", "two"]);
    let layer = store.css_layer_block_rule(&roots[3]).unwrap();
    assert_eq!(layer.name(&store, 100).unwrap(), "local");
    let scope = store.css_scope_rule(&roots[4]).unwrap();
    assert_eq!(
        scope.start(&store, limits()).unwrap().as_deref(),
        Some(".start")
    );
    assert_eq!(
        scope.end(&store, limits()).unwrap().as_deref(),
        Some(".end")
    );
    let named = store.css_supports_condition_rule(&roots[5]).unwrap();
    assert_eq!(named.name(&store).unwrap(), "--ready");
    let capture = named.capture(&store.snapshot()).unwrap();
    assert_eq!(capture.css_rules().unwrap().length().unwrap(), 1);
    assert_eq!(capture.name().unwrap(), "--ready");
    let _ = import.capture(&store.snapshot()).unwrap().media().unwrap();
    let _ = import
        .capture(&store.snapshot())
        .unwrap()
        .style_sheet()
        .unwrap();
    assert_eq!(
        namespace
            .capture(&store.snapshot())
            .unwrap()
            .prefix()
            .unwrap(),
        "svg"
    );
    assert_eq!(
        statement
            .capture(&store.snapshot())
            .unwrap()
            .name_list(100)
            .unwrap(),
        ["one", "two"]
    );
    assert_eq!(
        scope
            .capture(&store.snapshot())
            .unwrap()
            .start(limits())
            .unwrap()
            .as_deref(),
        Some(".start")
    );
}

#[test]
fn absence_foreign_owner_and_origin_guards_are_not_false_or_raw_list_access() {
    let (mut store, _, roots) = fixture("@media all {} @supports (display: grid) {}");
    let media = store.css_media_rule(&roots[0]).unwrap();
    let supports = store.css_supports_rule(&roots[1]).unwrap();
    assert!(matches!(
        media.matches(&store, None),
        Err(CssomError::QueryMatch(CssomQueryMatchError::MissingResult))
    ));
    assert!(matches!(
        supports.matches(&store, None),
        Err(CssomError::QueryMatch(CssomQueryMatchError::MissingResult))
    ));
    let other = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    assert!(matches!(
        media.condition_text(&other, limits()),
        Err(CssomError::ForeignOwner)
    ));
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let ticket = batch
        .create_parsed_sheet(
            "@media all {}",
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: "opaque".into(),
                    revision: 1,
                },
                false,
            ),
        )
        .unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let id = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()[0]
        .clone();
    let media = store.css_media_rule(&id).unwrap();
    assert!(matches!(
        media.css_rules(&store),
        Err(CssomError::Source(CssomException::Security))
    ));
    assert!(matches!(
        media.capture(&store.snapshot()).unwrap().css_rules(),
        Err(CssomError::Source(CssomException::Security))
    ));
}

#[test]
fn nested_declarations_return_properties_on_both_read_phases() {
    let (store, _, roots) = fixture(".parent { & .child {} color: red; }");
    let parent = store.css_style_rule(&roots[0]).unwrap();
    let nested = parent
        .css_rules(&store)
        .unwrap()
        .item(&store, 1)
        .unwrap()
        .unwrap()
        .as_nested_declarations(&store)
        .unwrap()
        .unwrap();
    let properties: CssStyleProperties = nested.style(&store).unwrap();
    let captured: CssomCapture<CssStyleProperties> =
        nested.capture(&store.snapshot()).unwrap().style().unwrap();
    assert_eq!(captured.handle(), &properties);
    assert_eq!(
        captured.parent_rule().unwrap().unwrap().handle().id(),
        nested.id()
    );
}

#[test]
fn native_mutation_quotas_fail_atomically_and_allow_later_retry() {
    let mut store = CssomStore::new(
        CssomLimits {
            max_input_bytes: 40,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let ticket = batch
        .create_parsed_sheet(
            "@media all{}",
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: "sheet".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let id = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()[0]
        .clone();
    let group = store.css_media_rule(&id).unwrap();
    assert!(
        group
            .insert_rule(&mut store, &format!(".{} {{}}", "long".repeat(30)), None)
            .is_err()
    );
    assert_eq!(store.snapshot().revision(), snapshot.revision());
    assert_eq!(group.css_rules(&store).unwrap().length(&store).unwrap(), 0);
    let outcome = group.insert_rule(&mut store, "a {}", None).unwrap();
    assert_eq!(*outcome.value(), 0);
    assert_eq!(group.css_rules(&store).unwrap().length(&store).unwrap(), 1);
}

fn assert_rejected_original_fragment(outcome: &CssomOperationOutcome<()>, source: &str) {
    assert!(matches!(
        outcome.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(outcome.diagnostics().len(), 1);
    let diagnostic = &outcome.diagnostics()[0];
    assert_eq!(
        diagnostic.action(),
        surgeist_css::CssRecoveryAction::RejectInput
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

#[test]
fn invalid_selector_noop_outcome_reports_its_actual_original_diagnostic() {
    let (mut store, _, roots) = fixture(".original {} @page :left {}");
    let style = store.css_style_rule(&roots[0]).unwrap();
    let recovered = style.set_selector_text(&mut store, ":is(.ok").unwrap();
    assert!(
        recovered
            .diagnostics()
            .iter()
            .any(|d| d.action() == surgeist_css::CssRecoveryAction::RetainWithImplicitClosure)
    );
    let before = store.snapshot();
    let rejected = style.set_selector_text(&mut store, ".bad,").unwrap();
    assert_rejected_original_fragment(&rejected, ".bad,");
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(
        store
            .snapshot()
            .rule(style.id())
            .unwrap()
            .admission_diagnostics(),
        before.rule(style.id()).unwrap().admission_diagnostics()
    );
    assert_eq!(style.selector_text(&store, limits()).unwrap(), ":is(.ok)");
    let page = store.css_page_rule(&roots[1]).unwrap();
    let rejected = page.set_selector_text(&mut store, ")").unwrap();
    assert_rejected_original_fragment(&rejected, ")");
    assert_eq!(page.selector_text(&store, limits()).unwrap(), ":left");
}

#[test]
fn invalid_keyframe_append_outcome_reports_rejection_without_inventing_a_member() {
    let (mut store, _, roots) = fixture("@keyframes pulse { from {} }");
    let keyframes = store.css_keyframes_rule(&roots[0]).unwrap();
    let before = store.snapshot();
    let rejected = keyframes.append_rule(&mut store, "garbage").unwrap();
    assert_rejected_original_fragment(&rejected, "garbage");
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(keyframes.length(&store).unwrap(), 1);
    assert!(
        keyframes
            .find_rule(&store, "50%", &CssomLimits::default())
            .unwrap()
            .is_none()
    );
}
