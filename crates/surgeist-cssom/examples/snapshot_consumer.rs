//! A downstream consumer needs no private storage access or CSS text round trip.
use surgeist_css::{CssKnownProperty, CssPropertyNameRef};
use surgeist_cssom::*;

fn main() -> Result<(), CssomError> {
    let mut owner = CssomStore::new(CssomLimits::default(), CssomContext::default())?;
    let mut batch = owner.batch(CssomEditGuard::from_snapshot(&owner.snapshot()))?;
    let inputs = CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "example-document".into(),
            revision: 1,
        },
        "example-document".into(),
        None,
    );
    let ticket = batch.create_parsed_sheet("article { margin: 1px 2px 3px 4px; }", inputs)?;
    let result = owner.commit(batch)?;
    let sheet = result.sheet(&ticket)?.clone();
    let original = owner.snapshot();
    let rule = original.sheet_rules(&sheet)?[0].clone();
    let block = original
        .rule(&rule)?
        .data()
        .block()
        .ok_or(CssomError::WrongKind)?
        .clone();
    let mut edit = owner.batch(CssomEditGuard::from_snapshot(&original))?;
    edit.remove_selected_terminal(
        &block,
        CssPropertyNameRef::Known(CssKnownProperty::MarginTop),
    )?;
    owner.commit(edit)?;
    let current = owner.snapshot();
    match owner.changes_since(&original) {
        CssomChanges::Incremental(summary) if summary.supports_incremental(&original, &current) => {
            assert!(summary.categories().contains(&CssomChange::Declarations));
        }
        _ => panic!("full recompute would read one coherent current snapshot"),
    }
    drop(owner);
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(entries),
        ..
    } = current.block(&block)?.data()
    else {
        return Err(CssomError::WrongKind);
    };
    assert_eq!(entries.entries().len(), 3);
    assert!(entries.entries().iter().all(
        |entry| entry.property_name() != CssPropertyNameRef::Known(CssKnownProperty::MarginTop)
    ));
    assert_eq!(current.parent_style_sheet(&rule)?, Some(&sheet));
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(old),
        ..
    } = original.block(&block)?.data()
    else {
        return Err(CssomError::WrongKind);
    };
    assert_eq!(old.entries().len(), 4);
    println!(
        "captured revisions {} → {}; three checked margin terminals survive",
        original.revision().value(),
        current.revision().value()
    );
    Ok(())
}
