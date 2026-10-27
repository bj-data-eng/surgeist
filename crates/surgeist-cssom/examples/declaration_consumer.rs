//! Concrete composed-owner support and source edits, without a host binding.
use surgeist_css::CssParser;
use surgeist_css::property::CssPropertyGrammar;
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
        CssParser::default()
            .parse_block_contents("width: 1px")
            .into_parts()
            .0,
        false,
        false,
        None,
    )?;
    let id = store.commit(batch)?.block(&ticket)?.clone();
    let declaration = store.css_style_properties(&id)?;
    let before = declaration.capture(&store.snapshot())?;
    let outcome =
        declaration.set_property(&mut store, "width", "2px", None, Default::default(), |_| {
            unreachable!("all checked values need no decision")
        })?;
    assert!(outcome.owner_effects().is_empty()); // Null owner.
    let after = declaration.capture(&store.snapshot())?;
    drop(store);
    assert_eq!(
        before.get_property_value("width", Default::default())?,
        "1px"
    );
    assert_eq!(
        after.get_property_value("width", Default::default())?,
        "2px"
    );
    println!("owning old/new declaration captures: 1px → 2px");
    Ok(())
}
