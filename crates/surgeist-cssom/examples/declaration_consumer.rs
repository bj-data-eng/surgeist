//! Concrete composed-owner support and source edits, without a host binding.
use surgeist_css::{CssPropertyGrammar, parse_declaration_block_contents};
use surgeist_cssom::*;

fn main() -> Result<(), CssomError> {
    let support = CssomDeclarationSupport::try_new(vec![(
        CssPropertyGrammar::from_name("width").expect("CSS-owned grammar"),
        CssomUsabilityRequirement::AllCheckedValues,
    )])?;
    let context = CssomContext {
        inputs: vec![CssomInput {
            version: CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "example-width-owner".into(),
                revision: 1,
            },
            data: CssomInputData::Support(support),
        }],
        ..Default::default()
    };
    let mut store = CssomStore::new(CssomLimits::default(), context)?;
    let mut batch = store.batch(CssomEditGuard::from_snapshot(&store.snapshot()))?;
    let ticket = batch.create_declarations(
        parse_declaration_block_contents("width: 1px")
            .into_parts()
            .0,
        false,
        false,
        None,
    )?;
    let id = store.commit(batch)?.block(&ticket)?.clone();
    let before = store.snapshot();
    let mut batch = store.batch(CssomEditGuard::from_snapshot(&before))?;
    let prepared = batch.prepare_set_property(&id, "width", "2px", "", Default::default())?;
    batch.apply_declaration(prepared, &[])?;
    let commit = store.commit(batch)?;
    assert!(commit.owner_effects().is_empty()); // This block has a null owner.
    let after = store.snapshot();
    drop(store);
    assert_eq!(
        before
            .declarations(&id, Default::default())?
            .get_property_value("width")?,
        "1px"
    );
    assert_eq!(
        after
            .declarations(&id, Default::default())?
            .get_property_value("width")?,
        "2px"
    );
    println!("owning old/new declaration captures: 1px → 2px");
    Ok(())
}
