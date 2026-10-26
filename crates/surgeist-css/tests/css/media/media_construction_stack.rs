use crate::css::support::isolated_stack;

use surgeist_css::{CssComponentValues, CssCondition, CssMediaLeaf, CssMediaQuery, CssParser};

#[test]
fn deepest_supported_media_construction_and_serialization_fit_an_ordinary_caller_stack() {
    const CHILD: &str = "SURGEIST_MEDIA_CONSTRUCTION_STACK_CHILD";
    const TEST: &str = "css::media::media_construction_stack::deepest_supported_media_construction_and_serialization_fit_an_ordinary_caller_stack";
    isolated_stack::run(
        TEST,
        CHILD,
        "media construction, serialization and destruction completed",
        None,
        || {
            let source = format!("{}(width: 1px){}", "(".repeat(255), ")".repeat(255));
            let components = CssComponentValues::parse(&CssParser::default(), &source).unwrap();
            let condition = CssCondition::<CssMediaLeaf>::try_from_components(
                components.clone(),
                Default::default(),
            )
            .unwrap();
            assert_eq!(
                condition
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            let query = CssMediaQuery::try_from_components(components, Default::default()).unwrap();
            let cloned = query.clone();
            assert_eq!(query, cloned);
            assert_eq!(
                cloned
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            drop(cloned);
            drop(query);
            drop(condition);
            let opaque = format!("(future: {}1{})", "calc(".repeat(255), ")".repeat(255));
            let condition = CssCondition::<CssMediaLeaf>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), &opaque).unwrap(),
                Default::default(),
            )
            .unwrap();
            let surgeist_css::CssConditionKind::Leaf(CssMediaLeaf::UnknownFeature(unknown)) =
                condition.kind()
            else {
                panic!("unknown feature with valid numeric value");
            };
            assert_eq!(
                unknown.reason(),
                surgeist_css::CssUnknownMediaFeatureReason::UnknownName
            );
            assert_eq!(
                unknown
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                opaque
            );
        },
    );
}
