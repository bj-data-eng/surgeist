//! Public consumers of pending owner preparation, synchronization and retirement.
use std::sync::Arc;
use surgeist_css::property::CssPropertyGrammar;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_css::{CssParser, CssParserContext};
use surgeist_cssom::*;

fn version(role: CssomInputRole, name: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: name.into(),
        revision,
    }
}
fn packet(name: &str, revision: u64, supported: bool) -> CssomInlineDeclarationContext {
    let properties = if supported {
        [
            "width",
            "height",
            "margin",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
        ]
        .into_iter()
        .map(|name| {
            (
                CssPropertyGrammar::from_name(name).unwrap(),
                CssomUsabilityRequirement::AllCheckedValues,
            )
        })
        .collect()
    } else {
        Vec::new()
    };
    CssomInlineDeclarationContext::try_new(CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Owner, name, revision),
                data: CssomInputData::Owner {
                    identity: name.into(),
                    attribute: Some("style".into()),
                },
            },
            CssomInput {
                version: version(
                    CssomInputRole::ParserMode,
                    &format!("{name}:parser"),
                    revision,
                ),
                data: CssomInputData::ParserMode(CssParserContext::default()),
            },
            CssomInput {
                version: version(
                    CssomInputRole::Support,
                    &format!("{name}:support"),
                    revision,
                ),
                data: CssomInputData::Support(
                    CssomDeclarationSupport::try_new(properties).unwrap(),
                ),
            },
        ],
        linked: Vec::new(),
    })
    .unwrap()
}
fn new_store(limits: CssomLimits) -> CssomStore {
    CssomStore::new(limits, CssomContext::default()).unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(store: &mut CssomStore, packet: CssomInlineDeclarationContext) -> CssomBlockId {
    let mut edit = batch(store);
    let ticket = edit.create_inline_declarations(packet).unwrap();
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    commit.block(&ticket).unwrap().clone()
}
fn set(edit: &mut CssomBatch, block: &CssomBlockId, name: &str, value: &str) {
    let prepared = edit
        .prepare_set_property(block, name, value, "", Default::default())
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
}
fn set_and_settle(store: &mut CssomStore, block: &CssomBlockId, name: &str, value: &str) {
    let mut edit = batch(store);
    set(&mut edit, block, name, value);
    for mut effect in store.commit(edit).unwrap().take_owner_effects() {
        effect.cancel().unwrap();
    }
}
fn text(store: &CssomStore, block: &CssomBlockId) -> String {
    store
        .css_style_declaration(block)
        .unwrap()
        .css_text(store, Default::default())
        .unwrap()
}
fn value(store: &CssomStore, block: &CssomBlockId, name: &str) -> String {
    store
        .css_style_declaration(block)
        .unwrap()
        .get_property_value(store, name, Default::default())
        .unwrap()
}
fn owner(store: &CssomStore, block: &CssomBlockId) -> CssomInputVersion {
    store
        .snapshot()
        .block(block)
        .unwrap()
        .owner()
        .unwrap()
        .clone()
}
fn synchronize(store: &mut CssomStore, block: &CssomBlockId) -> CssomOwnerEffect {
    let before = store.snapshot();
    let mut edit = batch(store);
    edit.request_owner_synchronization(block, Default::default())
        .unwrap();
    let mut commit = store.commit(edit).unwrap();
    assert!(
        matches!(commit.publication(), CssomPublication::Unchanged { revision } if revision == before.revision())
    );
    assert_eq!(store.snapshot().inputs(), before.inputs());
    let mut effects = commit.take_owner_effects();
    assert_eq!(effects.len(), 1);
    effects.pop().unwrap()
}
fn unchanged(store: &CssomStore, before: &CssomSnapshot) {
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(store.snapshot().inputs(), before.inputs());
    assert!(matches!(
        store.changes_since(before),
        CssomChanges::Unchanged
    ));
}
fn echo(effect: &CssomOwnerEffect) -> CssomStyleAttributeChange<'static> {
    CssomStyleAttributeChange {
        owner: effect.owner().clone(),
        local_name: "style".into(),
        namespace: None,
        value: Some(CssomStyleAttributeInput::Utf16(Arc::new(
            effect.value().encode_utf16().collect(),
        ))),
        echo: Some(effect.id().clone()),
    }
}

#[test]
fn pending_inspection_is_pure_and_every_echo_field_is_qualified() {
    let mut store = new_store(Default::default());
    let block = create(&mut store, packet("element", 1, true));
    set_and_settle(&mut store, &block, "width", "1px");
    let mut effect = synchronize(&mut store, &block);
    let mut other = synchronize(&mut store, &block);
    let before = store.snapshot();
    let original_id = effect.id().clone();
    let matching = echo(&effect);
    store.validate_pending_owner_effect(&effect).unwrap();
    assert_eq!(
        store
            .classify_pending_owner_notification(&effect, &matching)
            .unwrap(),
        CssomOwnerNotification::MatchingEcho
    );
    let utf8 = CssomStyleAttributeChange {
        value: Some(effect.value().into()),
        ..matching.clone()
    };
    assert_eq!(
        store
            .classify_pending_owner_notification(&effect, &utf8)
            .unwrap(),
        CssomOwnerNotification::MatchingEcho
    );
    for case in 0..12 {
        let mut change = matching.clone();
        match case {
            0 => change.echo = None,
            1 => change.echo = Some(other.id().clone()),
            2 => change.owner.identity.push_str("-other"),
            3 => change.owner.revision += 1,
            4 => change.local_name = "STYLE".into(),
            5 => change.namespace = Some(String::new()),
            6 => change.namespace = Some("urn:other".into()),
            7 => change.value = None,
            8 => change.value = Some("width: 2px;".into()),
            9 => change.owner.role = CssomInputRole::Document,
            10 => change.value = Some("".into()),
            11 => {
                let mut units: Vec<_> = effect.value().encode_utf16().collect();
                units.push(0xd800);
                change.value = Some(CssomStyleAttributeInput::Utf16(Arc::new(units)));
            }
            _ => unreachable!(),
        }
        assert_eq!(
            store
                .classify_pending_owner_notification(&effect, &change)
                .unwrap(),
            CssomOwnerNotification::Unrelated
        );
        assert_eq!(effect.status(), CssomOwnerEffectStatus::Pending);
        assert_eq!(effect.id(), &original_id);
        assert!(!store.declaration_flags(&block).unwrap().updating);
        unchanged(&store, &before);
    }
    other.cancel().unwrap();
    let application = store.begin_owner_application(&mut effect).unwrap();
    assert!(application.flags().updating);
    assert_eq!(
        application.classify(&matching),
        CssomOwnerNotification::MatchingEcho
    );
    application.finish_success();
    assert_eq!(effect.id(), &original_id);
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Applied);
    assert!(!store.declaration_flags(&block).unwrap().updating);
    unchanged(&store, &before);
}

#[test]
fn bounded_consumer_read_refusal_leaves_the_same_effect_retryable() {
    let mut store = new_store(Default::default());
    let block = create(&mut store, packet("element", 1, true));
    set_and_settle(&mut store, &block, "width", "1px");
    let mut effect = synchronize(&mut store, &block);
    let id = effect.id().clone();
    let before = store.snapshot();
    store.validate_pending_owner_effect(&effect).unwrap();
    let refused = CssomDeclarationRequestLimits {
        provider: CssSerializationLimits::new(1000, 1000, 0),
        ..Default::default()
    };
    assert!(matches!(
        store
            .css_style_declaration(&block)
            .unwrap()
            .css_text(&store, refused),
        Err(CssomError::Declaration(_))
    ));
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Pending);
    assert_eq!(effect.id(), &id);
    store.validate_pending_owner_effect(&effect).unwrap();
    assert_eq!(effect.value(), "width: 1px;");
    store
        .begin_owner_application(&mut effect)
        .unwrap()
        .finish_success();
    unchanged(&store, &before);
}

#[test]
fn preparation_unwind_leaves_pending_but_application_unwind_is_terminal() {
    let mut store = new_store(CssomLimits {
        max_pending_owner_effects: 1,
        ..Default::default()
    });
    let block = create(&mut store, packet("element", 1, true));
    let mut effect = synchronize(&mut store, &block);
    let before = store.snapshot();
    let id = effect.id().clone();
    let preparation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        store.validate_pending_owner_effect(&effect).unwrap();
        assert_eq!(
            store
                .classify_pending_owner_notification(&effect, &echo(&effect))
                .unwrap(),
            CssomOwnerNotification::MatchingEcho
        );
        panic!("binding preparation unwound before application");
    }));
    assert!(preparation.is_err());
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Pending);
    assert_eq!(effect.id(), &id);
    assert!(!store.declaration_flags(&block).unwrap().updating);
    store.validate_pending_owner_effect(&effect).unwrap();
    let application = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _application = store.begin_owner_application(&mut effect).unwrap();
        panic!("binding application unwound after entering updating");
    }));
    assert!(application.is_err());
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Abandoned);
    assert!(!store.declaration_flags(&block).unwrap().updating);
    let mut recovery = synchronize(&mut store, &block);
    assert_ne!(recovery.id(), &id);
    recovery.cancel().unwrap();
    unchanged(&store, &before);
}

#[test]
fn synchronization_requires_supported_owner_state_but_does_not_mutate_readonly_declarations() {
    let mut store = new_store(Default::default());
    let mut edit = batch(&store);
    let unbound = edit
        .create_declarations(
            CssParser::default().parse_block_contents("").into_parts().0,
            false,
            false,
            None,
        )
        .unwrap();
    let computed = edit
        .create_declarations(
            CssParser::default().parse_block_contents("").into_parts().0,
            true,
            false,
            Some(version(CssomInputRole::Owner, "computed", 1)),
        )
        .unwrap();
    let readonly = edit
        .create_declarations(
            CssParser::default()
                .parse_block_contents("width: 2px")
                .into_parts()
                .0,
            false,
            true,
            Some(version(CssomInputRole::Owner, "readonly", 1)),
        )
        .unwrap();
    let commit = store.commit(edit).unwrap();
    let unbound = commit.block(&unbound).unwrap().clone();
    let computed = commit.block(&computed).unwrap().clone();
    let readonly = commit.block(&readonly).unwrap().clone();
    let mut foreign = new_store(Default::default());
    let foreign_block = create(&mut foreign, packet("foreign", 1, true));
    let before = store.snapshot();
    for case in 0..3 {
        let mut edit = batch(&store);
        let error = edit
            .request_owner_synchronization(
                match case {
                    0 => &unbound,
                    1 => &computed,
                    2 => &foreign_block,
                    _ => unreachable!(),
                },
                Default::default(),
            )
            .unwrap_err();
        assert!(match case {
            0 => matches!(
                error,
                CssomError::InvalidInput("owner synchronization requires a bound declaration")
            ),
            1 => matches!(error, CssomError::ComputedStyleUpdatePrecondition),
            2 => matches!(error, CssomError::ForeignOwner),
            _ => false,
        });
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        unchanged(&store, &before);
    }
    let mut effect = synchronize(&mut store, &readonly);
    assert_eq!(effect.value(), "width: 2px;");
    assert!(store.declaration_flags(&readonly).unwrap().readonly);
    effect.cancel().unwrap();
    unchanged(&store, &before);
}

#[test]
fn foreign_and_stale_inspection_refuse_before_consuming_the_pending_effect() {
    let mut first = new_store(Default::default());
    let block = create(&mut first, packet("element", 1, true));
    let mut effect = synchronize(&mut first, &block);
    let matching = echo(&effect);
    let mut foreign = new_store(Default::default());
    assert!(matches!(
        foreign.validate_pending_owner_effect(&effect),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        foreign.classify_pending_owner_notification(&effect, &matching),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        foreign.begin_owner_application(&mut effect),
        Err(CssomError::ForeignOwner)
    ));
    // Existing CSSOM behavior permits unrelated publications. DOM's later
    // inline-store gate, not this producer, owns preventing this drift.
    create(&mut first, packet("another", 1, true));
    assert!(matches!(
        first.validate_pending_owner_effect(&effect),
        Err(CssomError::StaleOwnerEffect)
    ));
    assert!(matches!(
        first.classify_pending_owner_notification(&effect, &matching),
        Err(CssomError::StaleOwnerEffect)
    ));
    assert!(matches!(
        first.begin_owner_application(&mut effect),
        Err(CssomError::StaleOwnerEffect)
    ));
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Pending);
    effect.cancel().unwrap();
}

#[test]
fn terminal_effects_stay_terminal_and_fresh_sync_uses_current_checked_state_and_context() {
    for terminal in 0..4 {
        let mut store = new_store(CssomLimits {
            max_pending_owner_effects: 1,
            ..Default::default()
        });
        let block = create(&mut store, packet("element", 1, true));
        set_and_settle(&mut store, &block, "width", "7px");
        let mut effect = synchronize(&mut store, &block);
        let old_id = effect.id().clone();
        let status = match terminal {
            0 => {
                store
                    .begin_owner_application(&mut effect)
                    .unwrap()
                    .finish_host_failure();
                CssomOwnerEffectStatus::HostFailed
            }
            1 => {
                drop(store.begin_owner_application(&mut effect).unwrap());
                CssomOwnerEffectStatus::Abandoned
            }
            2 => {
                effect.cancel().unwrap();
                CssomOwnerEffectStatus::Canceled
            }
            3 => {
                store
                    .begin_owner_application(&mut effect)
                    .unwrap()
                    .finish_success();
                CssomOwnerEffectStatus::Applied
            }
            _ => unreachable!(),
        };
        assert_eq!(effect.status(), status);
        assert!(matches!(
            store.validate_pending_owner_effect(&effect),
            Err(CssomError::EffectAlreadyConsumed)
        ));
        assert!(matches!(
            store.classify_pending_owner_notification(&effect, &echo(&effect)),
            Err(CssomError::EffectAlreadyConsumed)
        ));
        assert!(matches!(
            store.begin_owner_application(&mut effect),
            Err(CssomError::EffectAlreadyConsumed)
        ));
        // The new profile deliberately admits NO properties. Current checked
        // width must still serialize: synchronization is not ingestion/admission.
        let old = store.snapshot();
        let mut edit = batch(&store);
        edit.update_inline_declaration_context(&block, packet("element", 2, false))
            .unwrap();
        store.commit(edit).unwrap();
        let before = store.snapshot();
        let mut current = synchronize(&mut store, &block);
        assert_ne!(current.id(), &old_id);
        assert_eq!(
            current.owner(),
            &version(CssomInputRole::Owner, "element", 2)
        );
        assert_eq!(current.revision(), before.revision());
        assert_eq!(current.value(), "width: 7px;");
        assert_eq!(value(&store, &block, "width"), "7px");
        assert_eq!(
            old.block(&block).unwrap().owner(),
            Some(&version(CssomInputRole::Owner, "element", 1))
        );
        assert_eq!(effect.status(), status);
        current.cancel().unwrap();
        unchanged(&store, &before);
    }
}

#[test]
fn synchronization_serializes_selected_members_instead_of_reexpanding_old_shorthand() {
    let mut store = new_store(Default::default());
    let block = create(&mut store, packet("element", 1, true));
    set_and_settle(&mut store, &block, "margin", "1px");
    let historical = store.snapshot();
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(&block, "margin-left", Default::default())
            .unwrap(),
        "1px"
    );
    for mut effect in store.commit(edit).unwrap().take_owner_effects() {
        effect.cancel().unwrap();
    }
    let mut effect = synchronize(&mut store, &block);
    assert_eq!(value(&store, &block, "margin-left"), "");
    for name in ["margin-top", "margin-right", "margin-bottom"] {
        assert_eq!(value(&store, &block, name), "1px");
    }
    // Independently consume the serialized effect through CSS's parser and a
    // separate canonical block. Replaying the old shorthand would restore left.
    let mut consumer = new_store(Default::default());
    let mut import = batch(&consumer);
    let ticket = import
        .create_declarations(
            CssParser::default()
                .parse_block_contents(effect.value())
                .into_parts()
                .0,
            false,
            false,
            None,
        )
        .unwrap();
    let copied = consumer
        .commit(import)
        .unwrap()
        .block(&ticket)
        .unwrap()
        .clone();
    assert_eq!(value(&consumer, &copied, "margin-left"), "");
    for name in ["margin-top", "margin-right", "margin-bottom"] {
        assert_eq!(value(&consumer, &copied, name), "1px");
    }
    assert_eq!(
        store
            .css_style_declaration(&block)
            .unwrap()
            .capture(&historical)
            .unwrap()
            .get_property_value("margin-left", Default::default())
            .unwrap(),
        "1px"
    );
    effect.cancel().unwrap();
}

#[test]
fn count_failure_does_not_spend_identity_and_terminal_release_restores_capacity() {
    let mut store = new_store(CssomLimits {
        max_pending_owner_effects: 1,
        max_owner_effect_identity: 3,
        ..Default::default()
    });
    let block = create(&mut store, packet("element", 1, true));
    set_and_settle(&mut store, &block, "width", "1px"); // first effect identity
    let mut first = synchronize(&mut store, &block); // second identity
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.request_owner_synchronization(&block, Default::default())
        .unwrap();
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::Limit {
            resource: "pending owner effects",
            maximum: 1
        })
    ));
    unchanged(&store, &before);
    store.validate_pending_owner_effect(&first).unwrap();
    first.cancel().unwrap();
    let mut last = synchronize(&mut store, &block); // third; failed attempt spent none
    assert_ne!(first.id(), last.id());
    last.cancel().unwrap();
    let mut edit = batch(&store);
    edit.request_owner_synchronization(&block, Default::default())
        .unwrap();
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::EffectIdentityExhausted)
    ));
    unchanged(&store, &before);
}

#[test]
fn byte_and_serialization_refusals_release_partial_reservations_without_publication() {
    let serialized_bytes = "width: 1px;".len();
    let mut store = new_store(CssomLimits {
        max_pending_owner_effects: 2,
        max_pending_owner_effect_bytes: serialized_bytes * 2 - 1,
        ..Default::default()
    });
    let block = create(&mut store, packet("element", 1, true));
    set_and_settle(&mut store, &block, "width", "1px");
    let mut first = synchronize(&mut store, &block);
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.request_owner_synchronization(&block, Default::default())
        .unwrap();
    assert!(
        matches!(store.commit(edit), Err(CssomError::Limit { resource: "pending owner effect bytes", maximum }) if maximum == serialized_bytes * 2 - 1)
    );
    first.cancel().unwrap();
    let mut retry = synchronize(&mut store, &block);
    retry.cancel().unwrap();
    let mut edit = batch(&store);
    edit.request_owner_synchronization(
        &block,
        CssomDeclarationRequestLimits {
            provider: CssSerializationLimits::new(1000, 1000, 0),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::Declaration(_))
    ));
    let mut retry = synchronize(&mut store, &block);
    retry.cancel().unwrap();
    unchanged(&store, &before);
}

#[test]
fn later_effect_refusal_releases_earlier_batch_reservation_and_spends_no_serials() {
    let mut store = new_store(CssomLimits {
        max_pending_owner_effects: 1,
        max_owner_effect_identity: 4,
        ..Default::default()
    });
    let first = create(&mut store, packet("first", 1, true));
    let second = create(&mut store, packet("second", 1, true));
    set_and_settle(&mut store, &first, "width", "1px"); // identity 1
    set_and_settle(&mut store, &second, "height", "2px"); // identity 2
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.request_owner_synchronization(&first, Default::default())
        .unwrap();
    edit.request_owner_synchronization(&second, Default::default())
        .unwrap();
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::Limit {
            resource: "pending owner effects",
            maximum: 1
        })
    ));
    unchanged(&store, &before);
    let mut retry_first = synchronize(&mut store, &first); // identity 3
    assert_eq!(retry_first.value(), "width: 1px;");
    retry_first.cancel().unwrap();
    let mut retry_second = synchronize(&mut store, &second); // identity 4
    assert_eq!(retry_second.value(), "height: 2px;");
    retry_second.cancel().unwrap();
    unchanged(&store, &before);
}

#[test]
fn retirement_removes_live_identity_and_packet_but_preserves_capture_and_reuses_only_capacity() {
    let mut store = new_store(CssomLimits {
        max_objects: 2,
        ..Default::default()
    });
    let block = create(&mut store, packet("element", 1, true));
    set_and_settle(&mut store, &block, "width", "1px");
    let object = store.css_style_declaration(&block).unwrap();
    let before = store.snapshot();
    let captured = object.capture(&before).unwrap();
    let expected = owner(&store, &block);
    let mut full = batch(&store);
    assert!(matches!(
        full.create_inline_declarations(packet("another", 1, true)),
        Err(CssomError::Limit {
            resource: "objects",
            maximum: 2
        })
    ));
    assert!(matches!(store.commit(full), Err(CssomError::BatchAborted)));
    let mut edit = batch(&store);
    edit.retire_inline_declarations(&block, &expected).unwrap();
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("retirement is a canonical publication")
    };
    assert!(
        summary
            .affected()
            .contains(&CssomObjectId::Block(block.clone()))
    );
    assert!(summary.categories().contains(&CssomChange::Owner));
    assert!(summary.categories().contains(&CssomChange::Declarations));
    assert!(
        summary
            .before_inputs()
            .inline_declarations
            .iter()
            .any(|(id, _)| id == &block)
    );
    assert!(
        !summary
            .after_inputs()
            .inline_declarations
            .iter()
            .any(|(id, _)| id == &block)
    );
    assert!(
        !summary
            .after_inputs()
            .blocks
            .iter()
            .any(|(id, _)| id == &block)
    );
    assert!(
        !summary
            .after_inputs()
            .declaration_admissions
            .iter()
            .any(|(id, _)| id == &block)
    );
    assert!(matches!(
        store.snapshot().block(&block),
        Err(CssomError::MissingObject)
    ));
    assert!(matches!(
        object.css_text(&store, Default::default()),
        Err(CssomError::MissingObject)
    ));
    assert_eq!(
        captured.css_text(Default::default()).unwrap(),
        "width: 1px;"
    );
    let mut missing = batch(&store);
    assert!(matches!(
        missing.retire_inline_declarations(&block, &expected),
        Err(CssomError::MissingObject)
    ));
    assert!(matches!(
        store.commit(missing),
        Err(CssomError::BatchAborted)
    ));
    let fresh = create(&mut store, packet("element", 1, true));
    assert_ne!(fresh, block);
    assert_eq!(text(&store, &fresh), "");
    assert_eq!(
        captured.css_text(Default::default()).unwrap(),
        "width: 1px;"
    );
}

#[test]
fn retirement_refusals_dropped_batches_and_stale_batches_preserve_current_block() {
    let mut store = new_store(Default::default());
    let block = create(&mut store, packet("element", 1, true));
    let expected = owner(&store, &block);
    let mut foreign = new_store(Default::default());
    let foreign_block = create(&mut foreign, packet("element", 1, true));
    let mut legacy_batch = batch(&store);
    let ticket = legacy_batch
        .create_declarations(
            CssParser::default().parse_block_contents("").into_parts().0,
            false,
            false,
            Some(expected.clone()),
        )
        .unwrap();
    let legacy = store
        .commit(legacy_batch)
        .unwrap()
        .block(&ticket)
        .unwrap()
        .clone();
    let before = store.snapshot();
    for case in 0..3 {
        let mut edit = batch(&store);
        let mut stale = expected.clone();
        stale.revision += 1;
        let error = match case {
            0 => edit
                .retire_inline_declarations(&foreign_block, &expected)
                .unwrap_err(),
            1 => edit.retire_inline_declarations(&block, &stale).unwrap_err(),
            2 => edit
                .retire_inline_declarations(&legacy, &expected)
                .unwrap_err(),
            _ => unreachable!(),
        };
        assert!(match case {
            0 => matches!(error, CssomError::ForeignOwner),
            1 => matches!(error, CssomError::StaleContext),
            2 => matches!(error, CssomError::WrongKind),
            _ => false,
        });
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        unchanged(&store, &before);
    }
    let mut legacy_effect = synchronize(&mut store, &legacy);
    assert_eq!(legacy_effect.owner(), &expected);
    assert_eq!(legacy_effect.value(), "");
    legacy_effect.cancel().unwrap();
    let mut dropped = batch(&store);
    dropped
        .retire_inline_declarations(&block, &expected)
        .unwrap();
    drop(dropped);
    unchanged(&store, &before);
    let mut stale = batch(&store);
    stale.retire_inline_declarations(&block, &expected).unwrap();
    create(&mut store, packet("other", 1, true));
    let before = store.snapshot();
    assert!(matches!(
        store.commit(stale),
        Err(CssomError::StaleRevision)
    ));
    unchanged(&store, &before);
    assert!(store.css_style_declaration(&block).is_ok());
}

#[test]
fn unresolved_preparation_blocks_retirement_publication_and_applied_edit_emits_no_dead_owner_write()
{
    let mut store = new_store(Default::default());
    let block = create(&mut store, packet("element", 1, true));
    let expected = owner(&store, &block);
    let before = store.snapshot();
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(&block, "width", "1px", "", Default::default())
        .unwrap();
    edit.retire_inline_declarations(&block, &expected).unwrap();
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::UnresolvedDeclarationPreparation)
    ));
    drop(prepared);
    unchanged(&store, &before);
    let mut edit = batch(&store);
    set(&mut edit, &block, "width", "2px");
    edit.retire_inline_declarations(&block, &expected).unwrap();
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    assert!(matches!(
        store.snapshot().block(&block),
        Err(CssomError::MissingObject)
    ));
}

#[test]
fn retirement_does_not_rewrite_or_release_an_externally_held_pending_effect() {
    let mut store = new_store(CssomLimits {
        max_pending_owner_effects: 1,
        ..Default::default()
    });
    let block = create(&mut store, packet("element", 1, true));
    let mut effect = synchronize(&mut store, &block);
    let expected = owner(&store, &block);
    let mut edit = batch(&store);
    edit.retire_inline_declarations(&block, &expected).unwrap();
    store.commit(edit).unwrap();
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Pending);
    assert!(matches!(
        store.validate_pending_owner_effect(&effect),
        Err(CssomError::StaleOwnerEffect)
    ));
    assert!(matches!(
        store.begin_owner_application(&mut effect),
        Err(CssomError::StaleOwnerEffect)
    ));
    let fresh = create(&mut store, packet("other", 1, true));
    let mut edit = batch(&store);
    edit.request_owner_synchronization(&fresh, Default::default())
        .unwrap();
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::Limit {
            resource: "pending owner effects",
            maximum: 1
        })
    ));
    effect.cancel().unwrap();
    let mut retry = synchronize(&mut store, &fresh);
    retry.cancel().unwrap();
}

fn linked_source() -> (CssomLinkedInput, usize) {
    // Equal text in separate original allocations creates two genuine source
    // identities. CSS's public storage report, not this test, prices backing.
    let mut units = Vec::with_capacity(4096);
    units.extend("/* retained original */ .a { --x: token; }".encode_utf16());
    let report = CssParser::new(CssParserContext::default(), Default::default())
        .parse_sheet_utf16(Arc::new(units), CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap();
    let bytes = report
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .storage_usage()
        .retained_bytes();
    let mut leaf = new_store(Default::default());
    let mut edit = batch(&leaf);
    let ticket = edit
        .create_typed_sheet(
            report.syntax().clone(),
            CssomSheetInputs::external(version(CssomInputRole::Owner, "sheet", 1), true),
        )
        .unwrap();
    let sheet = leaf.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    (
        CssomLinkedInput {
            version: version(CssomInputRole::Import, "linked-source", 1),
            snapshot: leaf.snapshot(),
            sheet,
        },
        bytes,
    )
}
fn linked_packet(name: &str, linked: CssomLinkedInput) -> CssomInlineDeclarationContext {
    let mut context = packet(name, 1, true).context().clone();
    context.linked.push(linked);
    CssomInlineDeclarationContext::try_new(context).unwrap()
}

#[test]
fn retirement_removes_packet_source_from_current_quota_while_old_capture_retains_it() {
    let (first_source, bytes) = linked_source();
    let (second_source, second_bytes) = linked_source();
    assert_eq!(
        bytes, second_bytes,
        "identical independently allocated fixture capacities"
    );
    let mut store = new_store(CssomLimits {
        max_retained_source_bytes: bytes,
        ..Default::default()
    });
    let first = create(&mut store, linked_packet("first", first_source.clone()));
    let keeper = create(&mut store, linked_packet("keeper", first_source));
    let old = store.snapshot();
    let expected = owner(&store, &first);
    let next_packet = linked_packet("second", second_source);
    let mut refused = batch(&store);
    assert!(
        matches!(refused.create_inline_declarations(next_packet.clone()), Err(CssomError::Limit {
        resource: "retained parse source bytes", maximum,
    }) if maximum == bytes)
    );
    assert!(matches!(
        store.commit(refused),
        Err(CssomError::BatchAborted)
    ));
    unchanged(&store, &old);
    let mut edit = batch(&store);
    edit.retire_inline_declarations(&first, &expected).unwrap();
    store.commit(edit).unwrap();
    // Another current block retains the same actual source. Retirement must
    // not release that source's quota merely because one reference disappeared.
    let after_first = store.snapshot();
    let mut still_full = batch(&store);
    assert!(
        matches!(still_full.create_inline_declarations(next_packet.clone()), Err(CssomError::Limit {
        resource: "retained parse source bytes", maximum,
    }) if maximum == bytes)
    );
    assert!(matches!(
        store.commit(still_full),
        Err(CssomError::BatchAborted)
    ));
    unchanged(&store, &after_first);
    let keeper_owner = owner(&store, &keeper);
    let mut edit = batch(&store);
    edit.retire_inline_declarations(&keeper, &keeper_owner)
        .unwrap();
    store.commit(edit).unwrap();
    // The old capture deliberately remains alive. Only the current owner loses
    // the packet/source reachability; this is not a process-RSS reclamation test.
    let second = create(&mut store, next_packet);
    assert_ne!(first, second);
    assert!(old.block(&first).unwrap().inline_context().is_some());
    assert!(matches!(
        store.snapshot().block(&first),
        Err(CssomError::MissingObject)
    ));
    assert_eq!(store.snapshot().inputs().inline_declarations.len(), 1);
}
