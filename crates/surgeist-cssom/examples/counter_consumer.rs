//! Partial counter descriptors, genuine parsed origins and immutable captures.
use surgeist_css::rule::counter_style::CssCounterStyleDescriptorKind;
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
    let object = store.css_counter_style_rule(&rule)?;
    let historical = object.capture(&before)?;
    let _system = object.set_system(&mut store, "fixed 4", Default::default())?;
    let _prefix = object.set_prefix(&mut store, "'§ '", Default::default())?;
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
    let current = object.capture(&after)?;
    drop(store);
    assert_eq!(historical.prefix(Default::default())?, "");
    assert_eq!(current.prefix(Default::default())?, "\"§ \"");
    println!(
        "{}: fixed → fixed 4, prefix → § ",
        current.name(Default::default())?
    );
    Ok(())
}
