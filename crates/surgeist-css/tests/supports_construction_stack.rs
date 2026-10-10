#![forbid(unsafe_code)]
//! Public construction, projection and owned-tree operations at supported depth.
#[path = "support/isolated_stack.rs"]
mod isolated_stack;

use surgeist_css::{
    CssNamespaceContext, CssSupportsCondition, CssSupportsConditionKind, parse_component_values,
};

fn construct(source: &str) -> CssSupportsCondition {
    CssSupportsCondition::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

#[test]
fn supported_supports_depth_and_owned_child_lifetimes_fit_an_ordinary_caller_stack() {
    const CHILD: &str = "SURGEIST_SUPPORTS_CONSTRUCTION_STACK_CHILD";
    const TEST: &str =
        "supported_supports_depth_and_owned_child_lifetimes_fit_an_ordinary_caller_stack";
    isolated_stack::run(
        TEST,
        CHILD,
        "supports construction, serialization and destruction completed",
        None,
        || {
            // The public component limit permits 256 nested enclosures.
            let grouped = format!("{}(width:1px){}", "(".repeat(255), ")".repeat(255));
            let negated = format!("{}(width:1px){}", "not (".repeat(255), ")".repeat(255));
            let opaque = format!("{}1{}", "Future(".repeat(256), ")".repeat(256));
            let numeric = format!("(opacity:{}1{})", "calc(".repeat(255), ")".repeat(255));
            let mut conjunction = "(width:1px)".to_owned();
            for _ in 0..255 {
                conjunction = format!("({conjunction} and future())");
            }
            for source in [grouped, negated, opaque, numeric, conjunction] {
                let condition = construct(&source);
                let cloned = condition.clone();
                assert_eq!(condition, cloned);
                assert_eq!(condition.serialize().unwrap().as_css(), source);
                assert_eq!(cloned.serialize().unwrap().as_css(), source);
                drop(condition);
                assert_eq!(cloned.serialize().unwrap().as_css(), source);
                drop(cloned);
            }
            let child_source = format!("{}(width:1px){}", "not (".repeat(253), ")".repeat(253));
            let parent = construct(&format!("({child_source}) and future()"));
            let CssSupportsConditionKind::And(children) = parent.kind() else {
                panic!("conjunction")
            };
            let child = children.conditions()[0].clone();
            let expected = format!("({child_source})");
            assert_eq!(child.serialize().unwrap().as_css(), expected);
            drop(parent);
            let copy = child.clone();
            assert_eq!(copy, child);
            assert_eq!(copy.serialize().unwrap().as_css(), expected);
            drop(child);
            drop(copy);
        },
    );
}
