use crate::{css, style};

use super::{AdapterError, AdapterErrorKind, AdapterResult};

pub fn lower_css_sheet_to_style(sheet: &css::CssSheet) -> AdapterResult<style::Sheet> {
    let mut style_sheet = style::Sheet::new();
    for rule in sheet.rules() {
        style_sheet.push_rule(lower_selector(rule.selector())?, lower_declarations(rule)?);
    }
    Ok(style_sheet)
}

fn lower_selector(selector: &css::CssSelector) -> AdapterResult<style::Selector> {
    match selector {
        css::CssSelector::Tag(tag) => lower_style_selector(style::Selector::tag(tag)),
        css::CssSelector::Key(key) => lower_style_selector(style::Selector::key(key)),
        css::CssSelector::Class(class) => lower_style_selector(style::Selector::class(class)),
        css::CssSelector::Compound(compound) => {
            let mut selector = style::Selector::compound();
            if let Some(tag) = compound.tag() {
                selector = lower_style_selector(selector.tag(tag))?;
            }
            if let Some(key) = compound.key() {
                selector = lower_style_selector(selector.key(key))?;
            }
            for class in compound.classes() {
                selector = lower_style_selector(selector.class(class))?;
            }
            Ok(selector.selector())
        }
    }
}

fn lower_declarations(rule: &css::CssRule) -> AdapterResult<style::Declarations> {
    let mut declarations = style::Declarations::new();
    for declaration in rule.declarations() {
        let property = lower_property(declaration.property());
        let value = lower_value(declaration)?;
        declarations
            .try_insert(property, value)
            .map_err(|error| validation_error(declaration, error))?;
    }
    Ok(declarations)
}

fn lower_property(property: css::CssProperty) -> style::Property {
    match property {
        css::CssProperty::Display => style::Property::Display,
        css::CssProperty::BoxSizing => style::Property::BoxSizing,
        css::CssProperty::Position => style::Property::Position,
        css::CssProperty::Direction => style::Property::Direction,
        css::CssProperty::Overflow => style::Property::Overflow,
        css::CssProperty::OverflowX => style::Property::OverflowX,
        css::CssProperty::OverflowY => style::Property::OverflowY,
        css::CssProperty::FlexDirection => style::Property::FlexDirection,
        css::CssProperty::FlexWrap => style::Property::FlexWrap,
        css::CssProperty::AlignItems => style::Property::AlignItems,
        css::CssProperty::AlignSelf => style::Property::AlignSelf,
        css::CssProperty::JustifyItems => style::Property::JustifyItems,
        css::CssProperty::JustifySelf => style::Property::JustifySelf,
        css::CssProperty::Width => style::Property::Width,
        css::CssProperty::Height => style::Property::Height,
        css::CssProperty::MinWidth => style::Property::MinWidth,
        css::CssProperty::MinHeight => style::Property::MinHeight,
        css::CssProperty::MaxWidth => style::Property::MaxWidth,
        css::CssProperty::MaxHeight => style::Property::MaxHeight,
        css::CssProperty::FlexBasis => style::Property::FlexBasis,
        css::CssProperty::Gap => style::Property::Gap,
        css::CssProperty::RowGap => style::Property::RowGap,
        css::CssProperty::ColumnGap => style::Property::ColumnGap,
        css::CssProperty::GridFlowTolerance => style::Property::GridFlowTolerance,
        css::CssProperty::FontSize => style::Property::FontSize,
        css::CssProperty::LineHeight => style::Property::LineHeight,
        css::CssProperty::Margin => style::Property::Margin,
        css::CssProperty::Padding => style::Property::Padding,
        css::CssProperty::BorderWidth => style::Property::BorderWidth,
        css::CssProperty::Color => style::Property::Color,
        css::CssProperty::Background => style::Property::Background,
        css::CssProperty::BorderColor => style::Property::BorderColor,
        css::CssProperty::Opacity => style::Property::Opacity,
        css::CssProperty::FlexGrow => style::Property::FlexGrow,
        css::CssProperty::FlexShrink => style::Property::FlexShrink,
        css::CssProperty::AspectRatio => style::Property::AspectRatio,
        css::CssProperty::ScrollbarWidth => style::Property::ScrollbarWidth,
    }
}

fn lower_value(declaration: &css::CssDeclaration) -> AdapterResult<style::Value> {
    let property = declaration.property();
    match declaration.value() {
        css::CssValue::GlobalKeyword(value) => lower_global_keyword(*value, property),
        css::CssValue::Display(value) => Ok(style::Value::Display(lower_display(*value))),
        css::CssValue::BoxSizing(value) => Ok(style::Value::BoxSizing(lower_box_sizing(*value))),
        css::CssValue::Position(value) => Ok(style::Value::Position(lower_position(*value))),
        css::CssValue::Direction(value) => Ok(style::Value::Direction(lower_direction(*value))),
        css::CssValue::Overflow(value) => Ok(style::Value::Overflow(lower_overflow(*value))),
        css::CssValue::OverflowAxes(value) => Ok(style::Value::OverflowAxes(
            style::OverflowAxes::new(lower_overflow(value.x), lower_overflow(value.y)),
        )),
        css::CssValue::FlexDirection(value) => {
            Ok(style::Value::FlexDirection(lower_flex_direction(*value)))
        }
        css::CssValue::FlexWrap(value) => Ok(style::Value::FlexWrap(lower_flex_wrap(*value))),
        css::CssValue::AlignItems(value) => Ok(style::Value::AlignItems(lower_align_items(*value))),
        css::CssValue::Length(value) => Ok(style::Value::Length(lower_length(value, property)?)),
        css::CssValue::GridFlowTolerance(value) => Ok(style::Value::GridFlowTolerance(
            lower_grid_flow_tolerance(value, property)?,
        )),
        css::CssValue::Edges(value) => Ok(style::Value::Edges(lower_edges(value, property)?)),
        css::CssValue::Color(value) => Ok(style::Value::Color(lower_color(*value))),
        css::CssValue::Number(value) => Ok(style::Value::Number(*value)),
    }
}

fn lower_global_keyword(
    value: css::CssGlobalKeyword,
    property: css::CssProperty,
) -> AdapterResult<style::Value> {
    match value {
        css::CssGlobalKeyword::Inherit => Ok(style::Value::Keyword(style::Keyword::Inherit)),
        css::CssGlobalKeyword::Initial => Ok(style::Value::Keyword(style::Keyword::Initial)),
        css::CssGlobalKeyword::Unset => Ok(style::Value::Keyword(style::Keyword::Unset)),
        css::CssGlobalKeyword::Revert => Err(unsupported_property_value(
            property,
            "unsupported CSS global keyword `revert`".to_owned(),
        )),
        css::CssGlobalKeyword::RevertLayer => Err(unsupported_property_value(
            property,
            "unsupported CSS global keyword `revert-layer`".to_owned(),
        )),
    }
}

fn lower_display(value: css::CssDisplay) -> style::Display {
    match value {
        css::CssDisplay::Block => style::Display::Block,
        css::CssDisplay::Flex => style::Display::Flex,
        css::CssDisplay::Grid => style::Display::Grid,
        css::CssDisplay::InlineBlock => style::Display::InlineBlock,
        css::CssDisplay::InlineGrid => style::Display::InlineGrid,
        css::CssDisplay::GridLanes => style::Display::GridLanes,
        css::CssDisplay::InlineGridLanes => style::Display::InlineGridLanes,
        css::CssDisplay::None => style::Display::None,
    }
}

fn lower_box_sizing(value: css::CssBoxSizing) -> style::BoxSizing {
    match value {
        css::CssBoxSizing::ContentBox => style::BoxSizing::ContentBox,
        css::CssBoxSizing::BorderBox => style::BoxSizing::BorderBox,
    }
}

fn lower_position(value: css::CssLayoutPosition) -> style::LayoutPosition {
    match value {
        css::CssLayoutPosition::Relative => style::LayoutPosition::Relative,
        css::CssLayoutPosition::Absolute => style::LayoutPosition::Absolute,
    }
}

fn lower_direction(value: css::CssDirection) -> style::Direction {
    match value {
        css::CssDirection::Ltr => style::Direction::Ltr,
        css::CssDirection::Rtl => style::Direction::Rtl,
    }
}

fn lower_overflow(value: css::CssOverflow) -> style::Overflow {
    match value {
        css::CssOverflow::Visible => style::Overflow::Visible,
        css::CssOverflow::Clip => style::Overflow::Clip,
        css::CssOverflow::Hidden => style::Overflow::Hidden,
        css::CssOverflow::Scroll => style::Overflow::Scroll,
    }
}

fn lower_flex_direction(value: css::CssFlexDirection) -> style::FlexDirection {
    match value {
        css::CssFlexDirection::Row => style::FlexDirection::Row,
        css::CssFlexDirection::Column => style::FlexDirection::Column,
        css::CssFlexDirection::RowReverse => style::FlexDirection::RowReverse,
        css::CssFlexDirection::ColumnReverse => style::FlexDirection::ColumnReverse,
    }
}

fn lower_flex_wrap(value: css::CssFlexWrap) -> style::FlexWrap {
    match value {
        css::CssFlexWrap::NoWrap => style::FlexWrap::NoWrap,
        css::CssFlexWrap::Wrap => style::FlexWrap::Wrap,
        css::CssFlexWrap::WrapReverse => style::FlexWrap::WrapReverse,
    }
}

fn lower_align_items(value: css::CssAlignItems) -> style::AlignItems {
    match value {
        css::CssAlignItems::Start => style::AlignItems::Start,
        css::CssAlignItems::End => style::AlignItems::End,
        css::CssAlignItems::SafeEnd => style::AlignItems::SafeEnd,
        css::CssAlignItems::FlexStart => style::AlignItems::FlexStart,
        css::CssAlignItems::FlexEnd => style::AlignItems::FlexEnd,
        css::CssAlignItems::SafeFlexEnd => style::AlignItems::SafeFlexEnd,
        css::CssAlignItems::Center => style::AlignItems::Center,
        css::CssAlignItems::SafeCenter => style::AlignItems::SafeCenter,
        css::CssAlignItems::Baseline => style::AlignItems::Baseline,
        css::CssAlignItems::LastBaseline => style::AlignItems::LastBaseline,
        css::CssAlignItems::Stretch => style::AlignItems::Stretch,
    }
}

fn lower_length(
    value: &css::CssLength,
    property: css::CssProperty,
) -> AdapterResult<style::Length> {
    match value {
        css::CssLength::Px(value) => Ok(style::Length::px(*value)),
        css::CssLength::Percent(value) => Ok(style::Length::percent(*value)),
        css::CssLength::Zero => Ok(style::Length::ZERO),
        css::CssLength::Auto => Ok(style::Length::Auto),
        css::CssLength::MinContent => Ok(style::Length::MinContent),
        css::CssLength::MaxContent => Ok(style::Length::MaxContent),
        css::CssLength::FitContent => Ok(style::Length::Fit),
        css::CssLength::Normal => Ok(style::Length::NORMAL),
        css::CssLength::Calc(value) => Ok(style::Length::Calc(lower_calc(value, property)?)),
    }
}

fn lower_edges(value: &css::CssEdges, property: css::CssProperty) -> AdapterResult<style::Edges> {
    Ok(style::Edges::new(
        lower_length(&value.top, property)?,
        lower_length(&value.right, property)?,
        lower_length(&value.bottom, property)?,
        lower_length(&value.left, property)?,
    ))
}

fn lower_calc(
    value: &css::CssCalcLength,
    property: css::CssProperty,
) -> AdapterResult<style::CalcLength> {
    match value {
        css::CssCalcLength::Px(value) => Ok(style::CalcLength::px(*value)),
        css::CssCalcLength::Percent(value) => Ok(style::CalcLength::percent(*value)),
        css::CssCalcLength::Sum(terms) => {
            let mut lowered = Vec::with_capacity(terms.len());
            for term in terms {
                let value = lower_calc(term.value(), property)?;
                lowered.push(match term.operator() {
                    css::CssCalcOperator::Add => style::CalcLengthTerm::add(value),
                    css::CssCalcOperator::Subtract => style::CalcLengthTerm::sub(value),
                });
            }
            style::CalcLength::try_sum(lowered)
                .map_err(|error| unsupported_property_value(property, error.to_string()))
        }
    }
}

fn lower_grid_flow_tolerance(
    value: &css::CssGridFlowTolerance,
    property: css::CssProperty,
) -> AdapterResult<style::GridFlowTolerance> {
    match value {
        css::CssGridFlowTolerance::Normal => Ok(style::GridFlowTolerance::Normal),
        css::CssGridFlowTolerance::Infinite => Ok(style::GridFlowTolerance::Infinite),
        css::CssGridFlowTolerance::Length(value) => Ok(style::GridFlowTolerance::Length(
            lower_length(value, property)?,
        )),
        css::CssGridFlowTolerance::Percent(value) => Ok(style::GridFlowTolerance::Percent(*value)),
    }
}

fn lower_color(value: css::CssColor) -> style::Color {
    style::Color::rgba(value.r, value.g, value.b, value.a)
}

fn lower_style_selector<T>(result: style::Result<T>) -> AdapterResult<T> {
    result.map_err(|error| {
        AdapterError::new(AdapterErrorKind::CssValueUnsupported {
            property: "selector".to_owned(),
            reason: error.to_string(),
        })
    })
}

fn validation_error(declaration: &css::CssDeclaration, error: style::Error) -> AdapterError {
    AdapterError::new(AdapterErrorKind::CssStyleValidation {
        property: format!("{:?}", lower_property(declaration.property())),
        reason: format!(
            "line {}, column {}: {}",
            declaration.line(),
            declaration.column(),
            error
        ),
    })
}

fn unsupported_property_value(property: css::CssProperty, reason: String) -> AdapterError {
    AdapterError::new(AdapterErrorKind::CssValueUnsupported {
        property: format!("{:?}", lower_property(property)),
        reason,
    })
}
