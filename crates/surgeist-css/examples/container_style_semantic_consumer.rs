#![forbid(unsafe_code)]
//! Visit authored style features without reparsing text or evaluating the query.
use surgeist_css::*;

fn main() {
    let condition = CssContainerCondition::try_from_components(
        parse_component_values("style((color: var(--theme)) and (1px < --size <= 10px))").unwrap(),
    )
    .unwrap();
    let CssContainerConditionKind::Style(query) = condition.kind() else {
        panic!("style query")
    };
    visit(query);
}

fn visit(query: &CssContainerStyleQuery) {
    match query.kind() {
        CssContainerStyleQueryKind::Feature(feature) => match feature {
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
        CssContainerStyleQueryKind::Parenthesized(child)
        | CssContainerStyleQueryKind::Not(child) => visit(child),
        CssContainerStyleQueryKind::And(list) | CssContainerStyleQueryKind::Or(list) => {
            for query in list.queries() {
                visit(query);
            }
        }
        CssContainerStyleQueryKind::GeneralEnclosed(opaque) => {
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
