//! A complete parsed input remains owned even when recovery retains no rules.
use surgeist_cssom::*;

#[test]
fn comment_only_inputs_share_the_owners_finite_retained_source_budget() {
    const INPUT_BYTES: usize = 16 * 1024 * 1024;
    const SOURCE_BYTES: usize = 64 * 1024 * 1024;
    let mut owner = CssomStore::new(
        CssomLimits {
            max_input_bytes: INPUT_BYTES,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let source = format!("/*{}*/", " ".repeat(INPUT_BYTES - 4));
    assert_eq!(source.len(), INPUT_BYTES);
    for index in 0..5 {
        let before = owner.snapshot();
        let mut edit = owner.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
        let inputs = CssomSheetInputs::external(
            CssomInputVersion {
                role: CssomInputRole::Owner,
                identity: format!("style-{index}"),
                revision: 1,
            },
            true,
        );
        match edit.create_parsed_sheet(&source, inputs) {
            Ok(ticket) => {
                let commit = owner.commit(edit).unwrap();
                let sheet = commit.sheet(&ticket).unwrap();
                assert!(owner.snapshot().sheet_rules(sheet).unwrap().is_empty());
            }
            Err(CssomError::Limit {
                resource: "retained parse source bytes",
                maximum: SOURCE_BYTES,
            }) => {
                assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
                assert_eq!(owner.snapshot().revision(), before.revision());
                assert_eq!(owner.snapshot().sheets(), before.sheets());
                return;
            }
            Err(error) => panic!("unexpected input rejection: {error:?}"),
        }
    }
    panic!("five distinct 16 MiB inputs must exceed the 64 MiB retained-source budget");
}
