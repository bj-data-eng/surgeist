//! Builds a checked authored container declaration from semantic values.
#![forbid(unsafe_code)]
use surgeist_css::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let names = CssContainerNames::Names(
        CssContainerNameList::try_new(vec![
            CssContainerName::try_from_decoded("1pane").expect("non-reserved decoded name"),
        ])
        .expect("nonempty names"),
    );
    let value = CssContainer::new(names, CssContainerType::InlineSizeScrollState);
    let limits = CssComponentValueLimits::try_new(0, 16, 128).expect("leaf limits");
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Container),
        value.to_components_with_limits(limits)?,
        CssImportance::Normal,
    )?;
    assert!(declaration.position().is_none());
    let CssExpansion::Contributions(CssContributions::Longhands(members)) =
        expand_declaration(&declaration)?
    else {
        panic!("ordinary shorthand")
    };
    println!("container: {}", value.serialize_with_limit(128)?.as_css());
    for member in members.items() {
        println!(
            "{}: {:?}",
            member.property().canonical_name(),
            member.ordinary_value().expect("ordinary member").view()
        );
    }
    Ok(())
}
