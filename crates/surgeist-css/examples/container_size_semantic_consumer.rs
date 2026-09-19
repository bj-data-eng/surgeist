#![forbid(unsafe_code)]
//! Inspect authored size features without choosing containers or evaluating values.
use surgeist_css::*;

fn describe_range<T>(range: &CssMediaRange<T>, value: impl Fn(&T) -> String) -> String {
    match range.view() {
        CssMediaRangeRef::Plain { value: v } => format!("plain {}", value(v)),
        CssMediaRangeRef::Min { value: v } => format!("minimum {}", value(v)),
        CssMediaRangeRef::Max { value: v } => format!("maximum {}", value(v)),
        CssMediaRangeRef::FeatureFirst {
            comparison,
            value: v,
        } => {
            format!("feature {comparison:?} {}", value(v))
        }
        CssMediaRangeRef::ValueFirst {
            comparison,
            value: v,
        } => {
            format!("{} {comparison:?} feature", value(v))
        }
        CssMediaRangeRef::Ascending {
            left,
            left_inclusive,
            right,
            right_inclusive,
        } => {
            format!(
                "{} ascending({left_inclusive}, {right_inclusive}) {}",
                value(left),
                value(right)
            )
        }
        CssMediaRangeRef::Descending {
            left,
            left_inclusive,
            right,
            right_inclusive,
        } => {
            format!(
                "{} descending({left_inclusive}, {right_inclusive}) {}",
                value(left),
                value(right)
            )
        }
        _ => "future range grammar".into(),
    }
}
fn describe_length(value: &CssContainerLength) -> String {
    match value.view() {
        CssContainerLengthRef::Numeric(calculation) => {
            format!("symbolic {:?} expression", calculation.result_type())
        }
        CssContainerLengthRef::Pending(pending) => format!("pending {:?}", pending.domain()),
        _ => "future length operand".into(),
    }
}
fn describe_ratio(value: &CssContainerRatio) -> String {
    match value.view() {
        CssContainerRatioRef::Numeric(ratio) => {
            let numerator = match ratio.numerator().expression() {
                CssCalculationExpressionRef::TreeCounting(tree) => format!("{:?}", tree.function()),
                CssCalculationExpressionRef::Value(value) => {
                    value.literal().representation().into()
                }
                _ => "symbolic calculation".into(),
            };
            format!(
                "ratio numerator {numerator}, implicit denominator {}",
                ratio.denominator_is_omitted()
            )
        }
        CssContainerRatioRef::Pending(pending) => format!("pending {:?}", pending.domain()),
        _ => "future ratio operand".into(),
    }
}
fn main() {
    for source in [
        "(width)",
        "(height)",
        "(inline-size)",
        "(block-size)",
        "(aspect-ratio)",
        "(orientation)",
        "(min-width: -1px)",
        "(1em < height <= 30em)",
        "(30em >= inline-size > 1em)",
        "(var(--limit) < block-size)",
        "(aspect-ratio: sibling-count())",
        "(orientation: landscape)",
        "(orientation: var(--mode))",
        "future-size(1px)",
    ] {
        let condition =
            CssContainerCondition::try_from_components(parse_component_values(source).unwrap())
                .unwrap();
        let description = match condition.kind() {
            CssContainerConditionKind::Feature(feature) => match feature {
                CssContainerFeatureQuery::Boolean(kind) => format!("boolean {}", kind.name()),
                CssContainerFeatureQuery::Width(range) => {
                    format!("width {}", describe_range(range, describe_length))
                }
                CssContainerFeatureQuery::Height(range) => {
                    format!("height {}", describe_range(range, describe_length))
                }
                CssContainerFeatureQuery::InlineSize(range) => {
                    format!("inline-size {}", describe_range(range, describe_length))
                }
                CssContainerFeatureQuery::BlockSize(range) => {
                    format!("block-size {}", describe_range(range, describe_length))
                }
                CssContainerFeatureQuery::AspectRatio(range) => {
                    describe_range(range, describe_ratio)
                }
                CssContainerFeatureQuery::Orientation(value) => match value.view() {
                    CssContainerOrientationRef::Keyword(value) => format!("orientation {value:?}"),
                    CssContainerOrientationRef::Pending(pending) => {
                        format!("pending {:?}", pending.domain())
                    }
                    _ => "future orientation operand".into(),
                },
                _ => "future size feature".into(),
            },
            CssContainerConditionKind::GeneralEnclosed(_) => "unrecognized query operand".into(),
            _ => "combined query".into(),
        };
        println!("{source}: {description}");
    }
}
