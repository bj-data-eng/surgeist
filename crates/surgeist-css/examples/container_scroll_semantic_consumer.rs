#![forbid(unsafe_code)]
//! Inspect scroll keyword and pending domains without resolving them.
use surgeist_css::*;

fn main() {
    let condition = CssContainerCondition::try_from_components(
        parse_component_values("scroll-state((stuck: top) and (snapped: var(--axis, both)))")
            .unwrap(),
    )
    .unwrap();
    let CssContainerConditionKind::ScrollState(query) = condition.kind() else {
        panic!("scroll query")
    };
    visit(query);
}

fn visit(query: &CssContainerScrollQuery) {
    match query.kind() {
        CssContainerScrollQueryKind::Feature(feature) => match feature {
            CssContainerScrollFeature::Boolean(kind) => println!("boolean {}", kind.name()),
            CssContainerScrollFeature::Stuck(value) => match value.view() {
                CssContainerStuckValueRef::Keyword(keyword) => println!("stuck {}", keyword.name()),
                CssContainerStuckValueRef::Pending(value) => pending(value),
                _ => {}
            },
            CssContainerScrollFeature::Snapped(value) => match value.view() {
                CssContainerSnappedValueRef::Keyword(keyword) => {
                    println!("snapped {}", keyword.name())
                }
                CssContainerSnappedValueRef::Pending(value) => pending(value),
                _ => {}
            },
            CssContainerScrollFeature::Scrollable(value)
            | CssContainerScrollFeature::Scrolled(value) => match value.view() {
                CssContainerScrollDirectionValueRef::Keyword(keyword) => {
                    println!("direction {}", keyword.name())
                }
                CssContainerScrollDirectionValueRef::Pending(value) => pending(value),
                _ => {}
            },
            _ => {}
        },
        CssContainerScrollQueryKind::Parenthesized(child)
        | CssContainerScrollQueryKind::Not(child) => visit(child),
        CssContainerScrollQueryKind::And(list) | CssContainerScrollQueryKind::Or(list) => {
            for child in list.queries() {
                visit(child);
            }
        }
        CssContainerScrollQueryKind::GeneralEnclosed(opaque) => {
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
