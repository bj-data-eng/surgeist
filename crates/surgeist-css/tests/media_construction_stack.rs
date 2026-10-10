#[path = "support/isolated_stack.rs"]
mod isolated_stack;

use surgeist_css::{CssMediaCondition, CssMediaQuery, parse_component_values};

#[test]
fn deepest_supported_media_construction_and_serialization_fit_an_ordinary_caller_stack() {
    const CHILD: &str = "SURGEIST_MEDIA_CONSTRUCTION_STACK_CHILD";
    const TEST: &str =
        "deepest_supported_media_construction_and_serialization_fit_an_ordinary_caller_stack";
    isolated_stack::run(
        TEST,
        CHILD,
        "media construction, serialization and destruction completed",
        None,
        || {
            let source = format!("{}(width: 1px){}", "(".repeat(255), ")".repeat(255));
            let components = parse_component_values(&source).unwrap();
            let condition = CssMediaCondition::try_from_components(components.clone()).unwrap();
            assert_eq!(condition.serialize().unwrap().as_css(), source);
            let query = CssMediaQuery::try_from_components(components).unwrap();
            let cloned = query.clone();
            assert_eq!(query, cloned);
            assert_eq!(cloned.serialize().unwrap().as_css(), source);
            drop(cloned);
            drop(query);
            drop(condition);
            let opaque = format!("(future: {}1{})", "calc(".repeat(255), ")".repeat(255));
            let condition =
                CssMediaCondition::try_from_components(parse_component_values(&opaque).unwrap())
                    .unwrap();
            let surgeist_css::CssMediaConditionKind::UnknownFeature(unknown) = condition.kind()
            else {
                panic!("unknown feature with valid numeric value");
            };
            assert_eq!(
                unknown.reason(),
                surgeist_css::CssUnknownMediaFeatureReason::UnknownName
            );
            assert_eq!(unknown.serialize().unwrap().as_css(), opaque);
        },
    );
}
