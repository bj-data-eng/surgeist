use surgeist_cssom::*;

fn version(role: CssomInputRole, identity: &str) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: identity.into(),
        revision: 1,
    }
}
fn context() -> CssomContext {
    CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Document, "document"),
                data: CssomInputData::Document {
                    identity: "document".into(),
                },
            },
            CssomInput {
                version: version(CssomInputRole::Window, "window"),
                data: CssomInputData::Window {
                    identity: "window".into(),
                    document: "document".into(),
                },
            },
            CssomInput {
                version: version(CssomInputRole::Support, "support"),
                data: CssomInputData::Support(CssomDeclarationSupport::default()),
            },
        ],
        linked: Vec::new(),
    }
}
fn media_facts() -> CssomQueryFacts {
    CssomQueryFacts::Media {
        document: version(CssomInputRole::Document, "document"),
        window: version(CssomInputRole::Window, "window"),
    }
}
fn supports_facts() -> CssomQueryFacts {
    CssomQueryFacts::Supports {
        support: version(CssomInputRole::Support, "support"),
    }
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn fixture(context: CssomContext) -> (CssomStore, CssomSheetId, Vec<CssomRuleId>) {
    source_fixture(
        context,
        "@media print {} @supports (display: grid) {} @media screen {}",
    )
}
fn source_fixture(
    context: CssomContext,
    source: &str,
) -> (CssomStore, CssomSheetId, Vec<CssomRuleId>) {
    let mut owner = CssomStore::new(CssomLimits::default(), context).unwrap();
    let mut edit = batch(&owner);
    let ticket = edit
        .create_parsed_sheet(
            source,
            CssomSheetInputs::constructed(
                version(CssomInputRole::Document, "document"),
                "document".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = owner.snapshot();
    let rules = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()
        .to_vec();
    (owner, sheet, rules)
}

#[test]
fn container_plural_getters_preserve_order_duplicates_opaque_queries_and_name_only_entries() {
    let (owner, _, rules) = source_fixture(
        context(),
        r"@container alpha, (width > 1px), alpha, beta future(foo), \31 escaped {} @container \31 escaped {} @container (width > 1px) {}",
    );
    let snapshot = owner.snapshot();
    assert_eq!(rules.len(), 3);
    let expected = vec![
        CssomContainerCondition {
            name: "alpha".into(),
            query: "".into(),
        },
        CssomContainerCondition {
            name: "".into(),
            query: "(width > 1px)".into(),
        },
        CssomContainerCondition {
            name: "alpha".into(),
            query: "".into(),
        },
        CssomContainerCondition {
            name: "beta".into(),
            query: "future(foo)".into(),
        },
        CssomContainerCondition {
            name: r"\31 escaped".into(),
            query: "".into(),
        },
    ];
    assert_eq!(
        snapshot.container_conditions(&rules[0], 1024).unwrap(),
        expected
    );
    let text = r"alpha, (width > 1px), alpha, beta future(foo), \31 escaped";
    assert_eq!(
        snapshot
            .container_condition_text(&rules[0], text.len())
            .unwrap(),
        text
    );
    assert!(
        snapshot
            .container_condition_text(&rules[0], text.len() - 1)
            .is_err()
    );
    assert_eq!(snapshot.container_name(&rules[0], 0).unwrap(), "");
    assert_eq!(snapshot.container_query(&rules[0], 0).unwrap(), "");
    assert_eq!(
        snapshot.container_name(&rules[1], 1024).unwrap(),
        r"\31 escaped"
    );
    assert_eq!(snapshot.container_query(&rules[1], 0).unwrap(), "");
    assert_eq!(
        snapshot.container_condition_text(&rules[1], 1024).unwrap(),
        r"\31 escaped"
    );
    assert_eq!(snapshot.container_name(&rules[2], 0).unwrap(), "");
    // Specified token meaning permits the provider's retained trailing trivia.
    assert_eq!(
        snapshot.container_query(&rules[2], 1024).unwrap(),
        "(width > 1px) "
    );
    let mut local = snapshot.container_conditions(&rules[0], 1024).unwrap();
    local[0].name = "locally modified dictionary".into();
    assert_eq!(
        snapshot.container_conditions(&rules[0], 1024).unwrap(),
        expected
    );
    let fields_len: usize = expected
        .iter()
        .map(|entry| entry.name.len() + entry.query.len())
        .sum();
    assert_eq!(
        snapshot
            .container_conditions(&rules[0], fields_len)
            .unwrap(),
        expected
    );
    assert!(
        snapshot
            .container_conditions(&rules[0], fields_len - 1)
            .is_err()
    );
    assert_eq!(snapshot.revision(), owner.snapshot().revision());
}

#[test]
fn omitted_scope_bounds_and_defined_named_support_name_do_not_require_whole_wrappers() {
    let (owner, _, rules) = source_fixture(
        context(),
        "@scope { .inside {} } @supports-condition --feature { color: red; .test { width: 1px; } }",
    );
    let snapshot = owner.snapshot();
    assert_eq!(rules.len(), 2);
    let zero = surgeist_css::CssSpecifiedValueSerializationLimits::new(0, 0, 0);
    assert_eq!(snapshot.scope_start(&rules[0], zero).unwrap(), None);
    assert_eq!(snapshot.scope_end(&rules[0], zero).unwrap(), None);
    assert_eq!(
        snapshot.named_support_condition_name(&rules[1]).unwrap(),
        "--feature"
    );
    assert!(matches!(
        snapshot.child_rules(&rules[1]),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        snapshot.named_support_condition_name(&rules[0]),
        Err(CssomError::WrongKind)
    ));
}

#[test]
fn local_layer_names_use_native_escaping_without_ancestor_prefixes_or_wrapper_support() {
    let (owner, _, rules) = source_fixture(
        context(),
        r"@layer outer { @layer inner {} @layer {} } @layer outer.inner, \31 edge, outer.inner;",
    );
    let snapshot = owner.snapshot();
    let nested = snapshot
        .rules(snapshot.child_rules(&rules[0]).unwrap())
        .unwrap();
    assert_eq!(snapshot.layer_block_name(&rules[0], 1024).unwrap(), "outer");
    assert_eq!(
        snapshot.layer_block_name(&nested[0], 1024).unwrap(),
        "inner"
    );
    assert_eq!(snapshot.layer_block_name(&nested[1], 0).unwrap(), "");
    let expected = vec![
        "outer.inner".to_owned(),
        r"\31 edge".to_owned(),
        "outer.inner".to_owned(),
    ];
    let bytes = expected.iter().map(String::len).sum();
    assert_eq!(
        snapshot.layer_statement_names(&rules[1], bytes).unwrap(),
        expected
    );
    assert!(
        snapshot
            .layer_statement_names(&rules[1], bytes - 1)
            .is_err()
    );
    let mut local = snapshot.layer_statement_names(&rules[1], bytes).unwrap();
    local[0].clear();
    assert_eq!(
        snapshot.layer_statement_names(&rules[1], bytes).unwrap(),
        expected
    );
    assert!(matches!(
        snapshot.layer_block_name(&rules[1], 1024),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        snapshot.layer_statement_names(&rules[0], 1024),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        snapshot.rule_css_text(&rules[0], &CssomRuleFormatLimits::default()),
        Err(CssomError::Format(_))
    ));
    let mut owner = CssomStore::new(CssomLimits::default(), context()).unwrap();
    let mut edit = batch(&owner);
    let mut inputs = CssomSheetInputs::constructed(
        version(CssomInputRole::Document, "document"),
        "document".into(),
        None,
    );
    inputs.constructed = false;
    let sheet = edit
        .create_parsed_sheet(
            r"@import 'a'; @import 'b' layer; @import 'c' layer(outer.\31 edge);",
            inputs,
        )
        .unwrap();
    let sheet = owner.commit(edit).unwrap().sheet(&sheet).unwrap().clone();
    let snapshot = owner.snapshot();
    let imports = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap();
    assert_eq!(imports.len(), 3);
    assert_eq!(snapshot.import_layer_name(&imports[0], 0).unwrap(), None);
    assert_eq!(
        snapshot
            .import_layer_name(&imports[1], 0)
            .unwrap()
            .as_deref(),
        Some("")
    );
    assert_eq!(
        snapshot
            .import_layer_name(&imports[2], 1024)
            .unwrap()
            .as_deref(),
        Some(r"outer.\31 edge")
    );
}
fn query_error<T: std::fmt::Debug>(result: Result<T, CssomError>, expected: CssomQueryMatchError) {
    assert!(
        matches!(&result, Err(CssomError::QueryMatch(actual)) if actual == &expected),
        "expected {expected:?}, actual {result:?}"
    );
}

#[test]
fn supplied_query_results_keep_actual_role_rule_and_owning_capture() {
    let (owner, _, rules) = fixture(context());
    let snapshot = owner.snapshot();
    let media = snapshot
        .capture_query_result(&rules[0], media_facts(), true)
        .unwrap();
    let supports = snapshot
        .capture_query_result(&rules[1], supports_facts(), false)
        .unwrap();
    assert!(snapshot.condition_matches(&rules[0], Some(&media)).unwrap());
    assert!(
        !snapshot
            .condition_matches(&rules[1], Some(&supports))
            .unwrap()
    );
    assert_eq!(media.owner(), snapshot.owner());
    assert_eq!(media.revision(), snapshot.revision());
    assert_eq!(media.rule(), &rules[0]);
    assert_eq!(media.role(), CssomQueryResultRole::Media);
    assert_eq!(media.context(), snapshot.context());
    assert_eq!(media.facts(), &media_facts());
    query_error(
        snapshot.condition_matches(&rules[0], None),
        CssomQueryMatchError::MissingResult,
    );
    query_error(
        snapshot.condition_matches(&rules[1], Some(&media)),
        CssomQueryMatchError::WrongRole,
    );
    query_error(
        snapshot.condition_matches(&rules[2], Some(&media)),
        CssomQueryMatchError::WrongRule,
    );
    query_error(
        snapshot.capture_query_result(&rules[0], supports_facts(), true),
        CssomQueryMatchError::WrongRole,
    );
    let (foreign, _, foreign_rules) = fixture(context());
    let foreign_result = foreign
        .snapshot()
        .capture_query_result(&foreign_rules[0], media_facts(), true)
        .unwrap();
    query_error(
        snapshot.condition_matches(&rules[0], Some(&foreign_result)),
        CssomQueryMatchError::ForeignOwner,
    );
    drop(owner);
    assert!(snapshot.condition_matches(&rules[0], Some(&media)).unwrap());
}

#[test]
fn result_rejects_state_and_input_drift_without_invalidating_older_capture() {
    let (mut owner, sheet, rules) = fixture(context());
    let before = owner.snapshot();
    let old = before
        .capture_query_result(&rules[0], media_facts(), true)
        .unwrap();
    let mut edit = batch(&owner);
    edit.set_disabled(&sheet, true).unwrap();
    owner.commit(edit).unwrap();
    query_error(
        owner.snapshot().condition_matches(&rules[0], Some(&old)),
        CssomQueryMatchError::StaleRevision,
    );
    assert!(before.condition_matches(&rules[0], Some(&old)).unwrap());
    let current = owner.snapshot();
    let support = current
        .capture_query_result(&rules[1], supports_facts(), true)
        .unwrap();
    let mut changed = context();
    changed.inputs[2].version.revision = 2;
    let mut edit = batch(&owner);
    edit.update_context(changed).unwrap();
    owner.commit(edit).unwrap();
    query_error(
        owner
            .snapshot()
            .condition_matches(&rules[1], Some(&support)),
        CssomQueryMatchError::StaleContext,
    );
    assert!(
        current
            .condition_matches(&rules[1], Some(&support))
            .unwrap()
    );
}

#[test]
fn result_requires_one_actual_fact_and_rejects_payload_drift_even_at_same_version() {
    let mut duplicated = context();
    duplicated.inputs.push(duplicated.inputs[0].clone());
    assert!(matches!(
        CssomStore::new(CssomLimits::default(), duplicated),
        Err(CssomError::InvalidInput(
            "duplicate supplied input identity"
        ))
    ));
    let (mut owner, _, rules) = fixture(context());
    let before = owner.snapshot();
    let result = before
        .capture_query_result(&rules[1], supports_facts(), false)
        .unwrap();
    let mut changed = context();
    changed.inputs[2].data = CssomInputData::Support(
        CssomDeclarationSupport::try_new(vec![(
            surgeist_css::CssPropertyGrammar::from_name("color").unwrap(),
            CssomUsabilityRequirement::AllCheckedValues,
        )])
        .unwrap(),
    );
    let mut edit = batch(&owner);
    edit.update_context(changed).unwrap();
    owner.commit(edit).unwrap();
    query_error(
        owner.snapshot().condition_matches(&rules[1], Some(&result)),
        CssomQueryMatchError::StaleContext,
    );
    assert!(!before.condition_matches(&rules[1], Some(&result)).unwrap());
    let (owner, _, rules) = source_fixture(context(), ".style {}");
    assert!(matches!(
        owner
            .snapshot()
            .capture_query_result(&rules[0], supports_facts(), true),
        Err(CssomError::WrongKind)
    ));
}

#[test]
fn evaluator_facts_cannot_default_or_substitute_window_document_and_support_roles() {
    let (mut owner, sheet, rules) = fixture(CssomContext::default());
    query_error(
        owner
            .snapshot()
            .capture_query_result(&rules[0], media_facts(), true),
        CssomQueryMatchError::MissingFact(CssomInputRole::Document),
    );
    query_error(
        owner
            .snapshot()
            .capture_query_result(&rules[1], supports_facts(), false),
        CssomQueryMatchError::MissingFact(CssomInputRole::Support),
    );
    let mut missing_window = context();
    missing_window.inputs.remove(1);
    let mut edit = batch(&owner);
    edit.update_context(missing_window).unwrap();
    owner.commit(edit).unwrap();
    query_error(
        owner
            .snapshot()
            .capture_query_result(&rules[0], media_facts(), true),
        CssomQueryMatchError::MissingFact(CssomInputRole::Window),
    );
    let mut mismatched = context();
    mismatched.inputs[1].data = CssomInputData::Window {
        identity: "window".into(),
        document: "other-document".into(),
    };
    let mut edit = batch(&owner);
    edit.update_context(mismatched).unwrap();
    owner.commit(edit).unwrap();
    query_error(
        owner
            .snapshot()
            .capture_query_result(&rules[0], media_facts(), true),
        CssomQueryMatchError::FactBindingMismatch(CssomInputRole::Document),
    );
    let wrong = CssomQueryFacts::Media {
        document: version(CssomInputRole::Support, "support"),
        window: version(CssomInputRole::Window, "window"),
    };
    query_error(
        owner
            .snapshot()
            .capture_query_result(&rules[0], wrong, false),
        CssomQueryMatchError::InvalidFactRole {
            expected: CssomInputRole::Document,
            actual: CssomInputRole::Support,
        },
    );
    let mut edit = batch(&owner);
    edit.update_context(context()).unwrap();
    edit.delete_sheet_rule(&sheet, 0).unwrap();
    owner.commit(edit).unwrap();
    query_error(
        owner
            .snapshot()
            .capture_query_result(&rules[0], media_facts(), true),
        CssomQueryMatchError::MissingFact(CssomInputRole::Document),
    );
    assert!(
        owner
            .snapshot()
            .capture_query_result(&rules[1], supports_facts(), true)
            .unwrap()
            .supplied_match()
    );
}
