//! Owner media input retains its complete source even when no queries survive.
use surgeist_css::CssSheet;
use surgeist_cssom::*;

fn version(revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "style".into(),
        revision,
    }
}

#[test]
fn comment_only_owner_media_obeys_source_budget_and_refuses_atomically() {
    let mut owner = CssomStore::new(
        CssomLimits {
            max_retained_source_bytes: 0,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut inputs = CssomSheetInputs::external(version(1), true);
    inputs.owner = Some("style".into());
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot()))
        .unwrap();
    let ticket = edit.create_typed_sheet(CssSheet::new(), inputs).unwrap();
    let id = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let before = owner.snapshot();
    let sheet = before.sheet(&id).unwrap();
    assert_eq!(sheet.inputs().owner.as_deref(), Some("style"));
    assert_eq!(sheet.inputs().version, version(1));
    assert!(sheet.authored().parse_source().is_none());
    assert!(before.media(sheet.media()).unwrap().queries().is_empty());

    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&before))
        .unwrap();
    match edit.update_sheet_owner_attributes(
        &id,
        CssomSheetOwnerAttributes {
            owner: "style",
            version: version(2),
            title: None,
            media: Some("/* complete owner media input */"),
        },
    ) {
        Err(CssomError::Limit {
            resource: "retained parse source bytes",
            maximum: 0,
        }) => {}
        Err(error) => panic!("unexpected owner media rejection: {error:?}"),
        Ok(_) => panic!("comment-only owner media must retain and admit its source"),
    }
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    let after = owner.snapshot();
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.sheets(), before.sheets());
    assert_eq!(after.sheet(&id).unwrap().inputs().version, version(1));
    assert!(after.media(sheet.media()).unwrap().queries().is_empty());
}
