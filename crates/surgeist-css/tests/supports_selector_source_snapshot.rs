#![forbid(unsafe_code)]
//! Clean constructed Supports selectors retain their typed grammar and authored outer origin.
use surgeist_css::{
    CssNamespaceContext, CssPseudoClass, CssSelector, CssSupportsCondition,
    CssSupportsConditionKind, CssValueOrigin, parse_component_values,
};

fn condition(source: &str) -> CssSupportsCondition {
    let components = parse_component_values(source).unwrap();
    let original = components.items()[0].origin().clone();
    let condition =
        CssSupportsCondition::try_from_components(components, &CssNamespaceContext::default())
            .expect("clean selector() grammar must be constructible");
    assert_eq!(condition.origin(), &original);
    let CssValueOrigin::Parsed(origin) = condition.origin() else {
        panic!("original parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(condition.serialize().unwrap().as_css(), source);
    condition
}

#[test]
fn ordinary_selector_without_forgiving_list_has_original_outer_origin() {
    let condition = condition("selector(.A)");
    assert_eq!(
        condition.kind(),
        &CssSupportsConditionKind::Selector(CssSelector::Class("A".into()))
    );
}

#[test]
fn clean_forgiving_selector_uses_its_generated_input_snapshot() {
    let condition = condition("selector(:is(.A))");
    let CssSupportsConditionKind::Selector(CssSelector::PseudoClass(CssPseudoClass::Is(list))) =
        condition.kind()
    else {
        panic!("typed clean Is selector")
    };
    assert_eq!(list.selectors(), [CssSelector::Class("A".into())]);
}
