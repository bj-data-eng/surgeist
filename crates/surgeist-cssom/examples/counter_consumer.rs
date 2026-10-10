//! Partial counter descriptors, genuine parsed origins and immutable captures.
use surgeist_css::CssCounterStyleDescriptorKind;
use surgeist_cssom::*;

fn main() -> Result<(), CssomError> {
    let mut store = CssomStore::new(Default::default(), Default::default())?;
    let mut edit = store.batch(CssomEditGuard::from_snapshot(&store.snapshot()))?;
    let ticket = edit.create_parsed_sheet(
        "@counter-style demo { system:fixed; symbols:'A'; }",
        CssomSheetInputs::constructed(
            CssomInputVersion {
                role: CssomInputRole::Document,
                identity: "example".into(),
                revision: 1,
            },
            "example".into(),
            None,
        ),
    )?;
    let sheet = store.commit(edit)?.sheet(&ticket)?.clone();
    let before = store.snapshot();
    let rule = before.sheet_rules(&sheet)?[0].clone();
    let mut edit = store.batch(CssomEditGuard::from_snapshot(&before))?;
    assert!(edit.set_counter_descriptor(
        &rule,
        CssCounterStyleDescriptorKind::System,
        "fixed 4",
        Default::default()
    )?);
    assert!(edit.set_counter_descriptor(
        &rule,
        CssCounterStyleDescriptorKind::Prefix,
        "'§ '",
        Default::default()
    )?);
    store.commit(edit)?;
    let after = store.snapshot();
    let block = after.rule(&rule)?.data().block().expect("counter block");
    let values = after
        .block(block)?
        .counter_descriptors(Default::default())?;
    let prefix = values
        .effective(CssCounterStyleDescriptorKind::Prefix)
        .expect("specified prefix");
    assert!(prefix.occurrence().is_none());
    assert_eq!(
        prefix
            .value()
            .origin()
            .expect("raw input")
            .source()
            .as_str(),
        "'§ '"
    );
    drop(store);
    assert_eq!(
        before.counter_descriptor(
            &rule,
            CssCounterStyleDescriptorKind::Prefix,
            Default::default()
        )?,
        ""
    );
    assert_eq!(
        after.counter_descriptor(
            &rule,
            CssCounterStyleDescriptorKind::Prefix,
            Default::default()
        )?,
        "\"§ \""
    );
    println!(
        "{}: fixed → fixed 4, prefix → § ",
        after.counter_name(&rule, Default::default())?
    );
    Ok(())
}
