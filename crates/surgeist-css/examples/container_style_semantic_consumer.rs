#![forbid(unsafe_code)]
//! Visit authored style features without reparsing text or evaluating the query.
use surgeist_css::*;

fn main() {
    let condition = CssCondition::<CssContainerLeaf>::try_from_components(
        CssComponentValues::parse(
            &CssParser::default(),
            "style((color: var(--theme)) and (1px < --size <= 10px))",
        )
        .unwrap(),
        Default::default(),
    )
    .unwrap();
    let CssConditionKind::Leaf(CssContainerLeaf::Style(query)) = condition.kind() else {
        panic!("style query")
    };
    visit(query);
}

fn visit(query: &CssCondition<CssStyleQueryLeaf>) {
    match query.kind() {
        CssConditionKind::Leaf(CssStyleQueryLeaf::Feature(feature)) => match feature {
            CssContainerStyleFeature::Boolean(name) => println!("boolean {name:?}"),
            CssContainerStyleFeature::Plain { name, value } => {
                println!("plain {name:?}: {} components", value.components().len());
            }
            CssContainerStyleFeature::Range(range) => match range.view() {
                CssContainerStyleRangeRef::Binary {
                    left,
                    comparison,
                    right,
                } => {
                    println!("binary {comparison:?}");
                    operand(left);
                    operand(right);
                }
                CssContainerStyleRangeRef::Ascending {
                    left,
                    middle,
                    right,
                    ..
                }
                | CssContainerStyleRangeRef::Descending {
                    left,
                    middle,
                    right,
                    ..
                } => {
                    operand(left);
                    operand(middle);
                    operand(right);
                }
                _ => {}
            },
            _ => {}
        },
        CssConditionKind::Parenthesized(child) | CssConditionKind::Not(child) => visit(child),
        CssConditionKind::And(list) | CssConditionKind::Or(list) => {
            for query in list.conditions() {
                visit(query);
            }
        }
        CssConditionKind::Leaf(CssStyleQueryLeaf::GeneralEnclosed(opaque)) => {
            println!("unknown syntax at {:?}", opaque.position());
        }
        _ => {}
    }
}

fn operand(value: &CssContainerStyleRangeOperand) {
    match value.view() {
        CssContainerStyleRangeOperandRef::CustomProperty(name) => {
            println!("implicit custom reference {}", name.as_str());
        }
        CssContainerStyleRangeOperandRef::Value(value) => {
            println!("authored operand: {} components", value.components().len());
        }
        _ => {}
    }
}
