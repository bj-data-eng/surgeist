use std::sync::Arc;
use surgeist_css::*;
use surgeist_cssom::*;

fn version(owner: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: owner.into(),
        revision,
    }
}
fn batch(owner: &CssomStore) -> CssomBatch {
    owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot()))
        .unwrap()
}
fn setup(limits: CssomLimits, name: &str, title: Vec<u16>) -> (CssomStore, CssomSheetId) {
    setup_in(limits, name, title, CssomContext::default())
}
fn setup_in(
    limits: CssomLimits,
    name: &str,
    title: Vec<u16>,
    context: CssomContext,
) -> (CssomStore, CssomSheetId) {
    let mut owner = CssomStore::new(limits, context).unwrap();
    let mut inputs = CssomSheetInputs::external(version(name, 1), true);
    inputs.owner = Some(name.into());
    inputs.title = Arc::new(title);
    let mut edit = batch(&owner);
    let ticket = edit.create_typed_sheet(CssSheet::new(), inputs).unwrap();
    let id = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    (owner, id)
}
fn media(context: CssParserContext, raw: Arc<Vec<u16>>) -> CssParseReport<CssMediaQueryList> {
    context
        .parse_media_query_list_utf16_with_limits(raw, CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap()
}

#[test]
fn exact_titles_round_trip_through_live_and_old_capture_then_absence_clears() {
    let raw = vec![0xd800, 65, 0xd83d, 0xde00, 0xdc00];
    let (mut owner, id) = setup(CssomLimits::default(), "style", raw.clone());
    let live = owner.css_style_sheet(&id).unwrap();
    let before = owner.snapshot();
    let captured = live.capture(&before).unwrap();
    assert_eq!(live.title(&owner).unwrap(), Some(raw.as_slice()));
    assert_eq!(captured.title().unwrap(), Some(raw.as_slice()));
    let replacement = [0xdc00, 0, 0xd800];
    let mut edit = batch(&owner);
    edit.update_sheet_owner_attributes(
        &id,
        CssomSheetOwnerAttributes {
            owner: "style",
            version: version("style", 2),
            title: Some(&replacement),
            media: CssomOwnerMediaInput::Utf8(None),
        },
    )
    .unwrap();
    let commit = owner.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    assert_eq!(live.title(&owner).unwrap(), Some(replacement.as_slice()));
    assert_eq!(captured.title().unwrap(), Some(raw.as_slice()));
    let mut edit = batch(&owner);
    edit.update_sheet_owner_attributes(
        &id,
        CssomSheetOwnerAttributes {
            owner: "style",
            version: version("style", 3),
            title: None,
            media: CssomOwnerMediaInput::Utf8(None),
        },
    )
    .unwrap();
    owner.commit(edit).unwrap();
    assert_eq!(live.title(&owner).unwrap(), None);
    assert_eq!(captured.title().unwrap(), Some(raw.as_slice()));
    drop(owner);
    assert_eq!(captured.title().unwrap(), Some(raw.as_slice()));
}

#[test]
fn title_byte_limit_accepts_exact_and_refuses_one_short_with_atomic_rollback() {
    // Empty opaque owner identities contribute no unrelated string bytes.
    for (limit, success) in [(4, true), (3, false)] {
        let (mut owner, id) = setup(
            CssomLimits {
                max_string_bytes: limit,
                ..Default::default()
            },
            "",
            vec![],
        );
        let before = owner.snapshot();
        let mut edit = batch(&owner);
        let result = edit.update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "",
                version: version("", 2),
                title: Some(&[0xd800, 65]),
                media: CssomOwnerMediaInput::Utf8(None),
            },
        );
        if success {
            assert!(result.unwrap().changed());
            owner.commit(edit).unwrap();
            assert_eq!(
                owner.css_style_sheet(&id).unwrap().title(&owner).unwrap(),
                Some([0xd800, 65].as_slice())
            );
        } else {
            assert!(matches!(
                result,
                Err(CssomError::Limit {
                    resource: "owner title bytes",
                    maximum: 3
                })
            ));
            assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
            assert_eq!(owner.snapshot().revision(), before.revision());
        }
    }
}

#[test]
fn changed_owner_retains_empty_media_source_and_true_noop_keeps_old_source() {
    let raw = Arc::new(
        "/* empty selected media */"
            .encode_utf16()
            .collect::<Vec<_>>(),
    );
    let report = media(CssParserContext::default(), raw.clone());
    let source = report.syntax().parse_source().unwrap().source().clone();
    let (mut owner, id) = setup(CssomLimits::default(), "style", vec![]);
    let mut edit = batch(&owner);
    assert!(
        edit.update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version("style", 2),
                title: None,
                media: CssomOwnerMediaInput::Parsed(report),
            }
        )
        .unwrap()
        .changed()
    );
    let commit = owner.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    let snapshot = owner.snapshot();
    let media_id = snapshot.sheet(&id).unwrap().media().clone();
    assert!(
        snapshot
            .media(&media_id)
            .unwrap()
            .parse_source()
            .unwrap()
            .source()
            .same_snapshot(&source)
    );
    let next = media(CssParserContext::default(), Arc::new(vec![]));
    let mut edit = batch(&owner);
    assert!(
        !edit
            .update_sheet_owner_attributes(
                &id,
                CssomSheetOwnerAttributes {
                    owner: "style",
                    version: version("style", 2),
                    title: None,
                    media: CssomOwnerMediaInput::Parsed(next),
                }
            )
            .unwrap()
            .changed()
    );
    let commit = owner.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    assert_eq!(owner.snapshot().revision(), snapshot.revision());
    assert!(
        owner
            .snapshot()
            .media(&media_id)
            .unwrap()
            .parse_source()
            .unwrap()
            .source()
            .same_snapshot(&source)
    );
    drop(owner);
    assert!(Arc::ptr_eq(
        snapshot
            .media(&media_id)
            .unwrap()
            .parse_source()
            .unwrap()
            .source()
            .original_utf16()
            .unwrap(),
        &raw
    ));
}

#[test]
fn parsed_media_checks_complete_context_and_full_comment_only_input() {
    let contexts = [
        CssParserContext::new(CssParserMode::Quirks),
        CssParserContext::default().with_svg_glyph_orientation_vertical(),
    ];
    for context in contexts {
        let (mut owner, id) = setup(CssomLimits::default(), "style", vec![]);
        let before = owner.snapshot();
        let mut edit = batch(&owner);
        assert!(matches!(
            edit.update_sheet_owner_attributes(
                &id,
                CssomSheetOwnerAttributes {
                    owner: "style",
                    version: version("style", 2),
                    title: None,
                    media: CssomOwnerMediaInput::Parsed(media(context, Arc::new(vec![]))),
                }
            ),
            Err(CssomError::InvalidInput("media parser context mismatch"))
        ));
        assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(owner.snapshot().revision(), before.revision());
        let context_input = CssomContext {
            inputs: vec![CssomInput {
                version: CssomInputVersion {
                    role: CssomInputRole::ParserMode,
                    identity: "parser".into(),
                    revision: 1,
                },
                data: CssomInputData::ParserMode(context),
            }],
            linked: vec![],
        };
        let (mut matching, id) = setup_in(CssomLimits::default(), "style", vec![], context_input);
        let mut edit = batch(&matching);
        edit.update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version("style", 2),
                title: None,
                media: CssomOwnerMediaInput::Parsed(media(context, Arc::new(vec![]))),
            },
        )
        .unwrap();
        matching.commit(edit).unwrap();
    }
    let (mut owner, id) = setup(
        CssomLimits {
            max_input_bytes: 4,
            ..Default::default()
        },
        "style",
        vec![],
    );
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version("style", 2),
                title: None,
                media: CssomOwnerMediaInput::Parsed(media(
                    CssParserContext::default(),
                    Arc::new("/*long*/".encode_utf16().collect())
                )),
            }
        ),
        Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: 4
        })
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
}

#[test]
fn late_media_source_failure_rolls_back_prior_title_change_and_keeps_capture() {
    let (mut owner, id) = setup(
        CssomLimits {
            max_retained_source_bytes: 1024,
            ..Default::default()
        },
        "style",
        vec![65],
    );
    let before = owner.snapshot();
    let mut raw = Vec::with_capacity(4096);
    raw.extend("screen".encode_utf16());
    let mut edit = batch(&owner);
    edit.update_sheet_owner_attributes(
        &id,
        CssomSheetOwnerAttributes {
            owner: "style",
            version: version("style", 2),
            title: Some(&[66]),
            media: CssomOwnerMediaInput::Utf8(None),
        },
    )
    .unwrap();
    assert!(matches!(
        edit.update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version("style", 3),
                title: Some(&[0xd800]),
                media: CssomOwnerMediaInput::Parsed(media(
                    CssParserContext::default(),
                    Arc::new(raw)
                )),
            }
        ),
        Err(CssomError::Limit {
            resource: "retained parse source bytes",
            maximum: 1024
        })
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
    assert_eq!(
        owner.css_style_sheet(&id).unwrap().title(&owner).unwrap(),
        Some([65].as_slice())
    );
}

#[test]
fn foreign_association_and_stale_guard_cannot_publish_title_or_media() {
    let (mut owner, id) = setup(CssomLimits::default(), "style", vec![]);
    let (mut other, _) = setup(CssomLimits::default(), "style", vec![]);
    let mut edit = batch(&other);
    assert!(matches!(
        edit.update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version("style", 2),
                title: None,
                media: CssomOwnerMediaInput::Utf8(None),
            }
        ),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(other.commit(edit), Err(CssomError::BatchAborted)));
    let before = owner.snapshot();
    let stale = batch(&owner);
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "wrong",
                version: version("style", 2),
                title: None,
                media: CssomOwnerMediaInput::Utf8(None),
            }
        ),
        Err(CssomError::InvalidInput(
            "owner attribute association mismatch"
        ))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    let mut edit = batch(&owner);
    edit.update_sheet_owner_attributes(
        &id,
        CssomSheetOwnerAttributes {
            owner: "style",
            version: version("style", 2),
            title: Some(&[65]),
            media: CssomOwnerMediaInput::Utf8(None),
        },
    )
    .unwrap();
    owner.commit(edit).unwrap();
    assert!(owner.commit(stale).is_err());
    assert_ne!(owner.snapshot().revision(), before.revision());
}

#[test]
fn initial_title_spare_backing_is_charged_as_real_owned_bytes() {
    let mut owner = CssomStore::new(
        CssomLimits {
            max_string_bytes: 128,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let before = owner.snapshot();
    let mut title = Vec::with_capacity(512);
    title.push(65);
    let mut inputs = CssomSheetInputs::external(version("style", 1), true);
    inputs.owner = Some("style".into());
    inputs.title = Arc::new(title);
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.create_typed_sheet(CssSheet::new(), inputs),
        Err(CssomError::Limit {
            resource: "host and live string bytes",
            maximum: 128
        })
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
}

#[test]
fn recovered_media_diagnostics_keep_source_and_resource_failure_poisons() {
    let (mut owner, id) = setup(CssomLimits::default(), "style", vec![]);
    let report = media(
        CssParserContext::default(),
        Arc::new("screen, ???".encode_utf16().collect()),
    );
    let source = report.syntax().parse_source().unwrap().source().clone();
    let mut edit = batch(&owner);
    let outcome = edit
        .update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version("style", 2),
                title: None,
                media: CssomOwnerMediaInput::Parsed(report),
            },
        )
        .unwrap();
    assert!(!outcome.diagnostics().is_empty());
    assert!(
        outcome.diagnostics()[0]
            .source()
            .unwrap()
            .same_snapshot(&source)
    );
    owner.commit(edit).unwrap();
    let before = owner.snapshot();
    let deep = format!("{}color{}", "(".repeat(2000), ")".repeat(2000));
    let report = media(
        CssParserContext::default(),
        Arc::new(deep.encode_utf16().collect()),
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| matches!(d.error().kind(), ErrorKind::NestingLimit(_)))
    );
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version("style", 3),
                title: Some(&[0xd800]),
                media: CssomOwnerMediaInput::Parsed(report),
            }
        ),
        Err(CssomError::ParseResource(_))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
    drop(owner);
    assert_eq!(
        outcome.diagnostics()[0].source().unwrap().as_str(),
        "screen, ???"
    );
}

#[test]
fn parsed_sheet_import_qualifies_initial_media_context_and_retains_source() {
    let context =
        CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical();
    let report = media(
        context,
        Arc::new("/* initial media */".encode_utf16().collect()),
    );
    let source = report.syntax().parse_source().unwrap().source().clone();
    let inputs = CssomSheetInputs::external(version("style", 1), true);
    let mut mismatched = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&mismatched);
    assert!(matches!(
        edit.create_parsed_sheet_with_media_reports(
            parse_sheet(""),
            report.clone(),
            inputs.clone()
        ),
        Err(CssomError::InvalidInput("media parser context mismatch"))
    ));
    assert!(matches!(
        mismatched.commit(edit),
        Err(CssomError::BatchAborted)
    ));
    let mut owner = CssomStore::new(
        CssomLimits::default(),
        CssomContext {
            inputs: vec![CssomInput {
                version: CssomInputVersion {
                    role: CssomInputRole::ParserMode,
                    identity: "parser".into(),
                    revision: 1,
                },
                data: CssomInputData::ParserMode(context),
            }],
            linked: vec![],
        },
    )
    .unwrap();
    let mut edit = batch(&owner);
    let construction = edit
        .create_parsed_sheet_with_media_reports(context.parse_sheet(""), report, inputs)
        .unwrap();
    let id = owner
        .commit(edit)
        .unwrap()
        .sheet(construction.ticket())
        .unwrap()
        .clone();
    let snapshot = owner.snapshot();
    assert!(
        snapshot
            .media(snapshot.sheet(&id).unwrap().media())
            .unwrap()
            .parse_source()
            .unwrap()
            .source()
            .same_snapshot(&source)
    );
}

#[test]
fn two_report_creation_checks_full_media_input_and_keeps_diagnostics_separate() {
    for (maximum, success) in [(8, true), (7, false)] {
        let mut owner = CssomStore::new(
            CssomLimits {
                max_input_bytes: maximum,
                ..Default::default()
            },
            CssomContext::default(),
        )
        .unwrap();
        let before = owner.snapshot();
        let mut edit = batch(&owner);
        let result = edit.create_parsed_sheet_with_media_reports(
            parse_sheet(""),
            parse_media_query_list("/*long*/"),
            CssomSheetInputs::external(version("style", 1), true),
        );
        if success {
            let construction = result.unwrap();
            assert!(construction.media_diagnostics().is_empty());
            let id = owner
                .commit(edit)
                .unwrap()
                .sheet(construction.ticket())
                .unwrap()
                .clone();
            let snapshot = owner.snapshot();
            assert_eq!(
                snapshot
                    .media(snapshot.sheet(&id).unwrap().media())
                    .unwrap()
                    .parse_source()
                    .unwrap()
                    .source()
                    .as_str(),
                "/*long*/"
            );
        } else {
            assert!(matches!(
                result,
                Err(CssomError::Limit {
                    resource: "parse input bytes",
                    maximum: 7
                })
            ));
            assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
            assert_eq!(owner.snapshot().revision(), before.revision());
        }
    }
    let mut owner = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&owner);
    let construction = edit
        .create_parsed_sheet_with_media_reports(
            parse_sheet("@unknown;"),
            parse_media_query_list("???"),
            CssomSheetInputs::external(version("style", 1), true),
        )
        .unwrap();
    assert_eq!(construction.media_diagnostics().len(), 1);
    let id = owner
        .commit(edit)
        .unwrap()
        .sheet(construction.ticket())
        .unwrap()
        .clone();
    let snapshot = owner.snapshot();
    assert_eq!(snapshot.sheet(&id).unwrap().diagnostics().len(), 1);
    assert_eq!(
        snapshot.sheet(&id).unwrap().diagnostics()[0]
            .source()
            .unwrap()
            .as_str(),
        "@unknown;"
    );
    assert_eq!(
        construction.media_diagnostics()[0]
            .source()
            .unwrap()
            .as_str(),
        "???"
    );
}

#[test]
fn two_report_creation_rejects_resource_recovery_and_conflicting_media_atomically() {
    let mut owner = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    let deep = format!("{}color{}", "(".repeat(2000), ")".repeat(2000));
    assert!(matches!(
        edit.create_parsed_sheet_with_media_reports(
            parse_sheet(""),
            parse_media_query_list(&deep),
            CssomSheetInputs::external(version("style", 1), true)
        ),
        Err(CssomError::ParseResource(_))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
    let mut inputs = CssomSheetInputs::external(version("style", 1), true);
    inputs.media = parse_media_query_list("screen").into_parts().0;
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.create_parsed_sheet_with_media_reports(
            parse_sheet(""),
            parse_media_query_list("print"),
            inputs
        ),
        Err(CssomError::InvalidInput(
            "initial media input already supplied"
        ))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
}
