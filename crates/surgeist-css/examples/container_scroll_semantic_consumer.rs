#![forbid(unsafe_code)]
//! Inspect scroll keyword and pending domains without resolving them.
use surgeist_css::*;

fn main() {
    let condition = CssCondition::<CssContainerLeaf>::try_from_components(
        parse_component_values("scroll-state((stuck: top) and (snapped: var(--axis, both)))")
            .unwrap(),
        Default::default(),
    )
    .unwrap();
    let CssConditionKind::Leaf(CssContainerLeaf::ScrollState(query)) = condition.kind() else {
        panic!("scroll query")
    };
    visit(query);
}

fn visit(query: &CssCondition<CssScrollStateLeaf>) {
    match query.kind() {
        CssConditionKind::Leaf(CssScrollStateLeaf::Feature(feature)) => match feature {
            CssContainerScrollFeature::Boolean(kind) => println!("boolean {}", kind.as_str()),
            CssContainerScrollFeature::Stuck(value) => match value.view() {
                CssContainerStuckValueRef::Keyword(keyword) => {
                    println!("stuck {}", keyword.as_str())
                }
                CssContainerStuckValueRef::Pending(value) => pending(value),
                _ => {}
            },
            CssContainerScrollFeature::Snapped(value) => match value.view() {
                CssContainerSnappedValueRef::Keyword(keyword) => {
                    println!("snapped {}", keyword.as_str())
                }
                CssContainerSnappedValueRef::Pending(value) => pending(value),
                _ => {}
            },
            CssContainerScrollFeature::Scrollable(value)
            | CssContainerScrollFeature::Scrolled(value) => match value.view() {
                CssContainerScrollDirectionValueRef::Keyword(keyword) => {
                    println!("direction {}", keyword.as_str())
                }
                CssContainerScrollDirectionValueRef::Pending(value) => pending(value),
                _ => {}
            },
            _ => {}
        },
        CssConditionKind::Parenthesized(child) | CssConditionKind::Not(child) => visit(child),
        CssConditionKind::And(list) | CssConditionKind::Or(list) => {
            for child in list.conditions() {
                visit(child);
            }
        }
        CssConditionKind::Leaf(CssScrollStateLeaf::GeneralEnclosed(opaque)) => {
            println!("unknown at {:?}", opaque.position());
        }
        _ => {}
    }
}

fn pending(value: &CssContainerPendingValue) {
    println!(
        "pending {:?}: {} components",
        value.domain(),
        value.components().component_count()
    );
}
