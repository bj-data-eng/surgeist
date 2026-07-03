//! Lower resolved style values into the layout calculator surface.

use super::{AdapterError, AdapterErrorKind, AdapterResult};
use crate::{layout, style};
use style::{
    CalcLength, CalcOperator, Display, Edges, GridAutoFlow, GridFlowTolerance, GridLine,
    GridTrackComponent, GridTrackList, Length, MaxTrackSizing, MinTrackSizing, Property, Resolved,
    TrackRepeat, TrackRepeatCount, TrackSizing, Value,
};

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutLoweringOutput {
    pub node: layout::NodeInput,
    pub calc_store: layout::LayoutCalcStore,
}

#[derive(Clone, Debug, Default)]
pub struct LayoutLoweringSession {
    calc_store: layout::LayoutCalcStore,
}

pub fn lower_style_to_layout(resolved: &Resolved) -> AdapterResult<layout::NodeInput> {
    if let Some(property) = resolved_calc_property(resolved) {
        return Err(unsupported_property(
            property,
            "calc values require lower_style_to_layout_with_store",
        ));
    }
    let mut session = LayoutLoweringSession::new();
    session.lower_node(resolved)
}

pub fn lower_style_to_layout_with_store(
    resolved: &Resolved,
) -> AdapterResult<LayoutLoweringOutput> {
    let mut session = LayoutLoweringSession::new();
    let node = session.lower_node(resolved)?;
    Ok(LayoutLoweringOutput {
        node,
        calc_store: session.finish(),
    })
}

impl LayoutLoweringSession {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lower_node(&mut self, resolved: &Resolved) -> AdapterResult<layout::NodeInput> {
        lower_node_with_session(resolved, self)
    }

    #[must_use]
    pub fn finish(self) -> layout::LayoutCalcStore {
        self.calc_store
    }

    fn push_calc_length(&mut self, calc: &CalcLength) -> layout::CalcId {
        let expression = lower_calc_expression(calc);
        self.calc_store.push(expression)
    }
}

fn resolved_calc_property(resolved: &Resolved) -> Option<Property> {
    [
        (Property::Width, length_uses_calc(resolved.width())),
        (Property::Height, length_uses_calc(resolved.height())),
        (
            Property::Inset,
            edges_use_calc(edges(resolved, Property::Inset)),
        ),
        (
            Property::MinWidth,
            length_uses_calc(length(resolved, Property::MinWidth)),
        ),
        (
            Property::MinHeight,
            length_uses_calc(length(resolved, Property::MinHeight)),
        ),
        (
            Property::MaxWidth,
            length_uses_calc(length(resolved, Property::MaxWidth)),
        ),
        (
            Property::MaxHeight,
            length_uses_calc(length(resolved, Property::MaxHeight)),
        ),
        (Property::Margin, edges_use_calc(resolved.margin_edges())),
        (Property::Padding, edges_use_calc(resolved.padding_edges())),
        (
            Property::BorderWidth,
            edges_use_calc(resolved.border_width_edges()),
        ),
        (
            Property::ColumnGap,
            length_uses_calc(length(resolved, Property::ColumnGap)),
        ),
        (
            Property::RowGap,
            length_uses_calc(length(resolved, Property::RowGap)),
        ),
        (
            Property::FlexBasis,
            length_uses_calc(length(resolved, Property::FlexBasis)),
        ),
        (Property::FontSize, length_uses_calc(resolved.font_size())),
        (
            Property::GridFlowTolerance,
            grid_flow_tolerance_uses_calc(grid_flow_tolerance(resolved)),
        ),
        (
            Property::GridTemplateColumns,
            track_list_uses_calc(track_list(resolved, Property::GridTemplateColumns)),
        ),
        (
            Property::GridTemplateRows,
            track_list_uses_calc(track_list(resolved, Property::GridTemplateRows)),
        ),
        (
            Property::GridAutoColumns,
            track_list_uses_calc(track_list(resolved, Property::GridAutoColumns)),
        ),
        (
            Property::GridAutoRows,
            track_list_uses_calc(track_list(resolved, Property::GridAutoRows)),
        ),
    ]
    .into_iter()
    .find_map(|(property, uses_calc)| uses_calc.then_some(property))
}

fn length_uses_calc(length: Length) -> bool {
    matches!(length, Length::Calc(_))
}

fn edges_use_calc(edges: Edges) -> bool {
    length_uses_calc(edges.top)
        || length_uses_calc(edges.right)
        || length_uses_calc(edges.bottom)
        || length_uses_calc(edges.left)
}

fn grid_flow_tolerance_uses_calc(tolerance: GridFlowTolerance) -> bool {
    matches!(tolerance, GridFlowTolerance::Length(Length::Calc(_)))
}

fn track_list_uses_calc(list: &GridTrackList) -> bool {
    list.components.iter().any(track_component_uses_calc)
}

fn track_component_uses_calc(component: &GridTrackComponent) -> bool {
    match component {
        GridTrackComponent::Track(track) => track_sizing_uses_calc(track),
        GridTrackComponent::Repeat(repeat) => {
            repeat.components().iter().any(track_component_uses_calc)
        }
        GridTrackComponent::LineNames(_) | GridTrackComponent::Subgrid(_) => false,
    }
}

fn track_sizing_uses_calc(track: &TrackSizing) -> bool {
    min_track_sizing_uses_calc(&track.min) || max_track_sizing_uses_calc(&track.max)
}

fn min_track_sizing_uses_calc(track: &MinTrackSizing) -> bool {
    matches!(track, MinTrackSizing::Length(Length::Calc(_)))
}

fn max_track_sizing_uses_calc(track: &MaxTrackSizing) -> bool {
    matches!(
        track,
        MaxTrackSizing::Length(Length::Calc(_)) | MaxTrackSizing::FitContent(Length::Calc(_))
    )
}

fn lower_node_with_session(
    resolved: &Resolved,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::NodeInput> {
    Ok(layout::NodeInput {
        display: lower_display(resolved.display())?,
        box_sizing: lower_box_sizing(resolved),
        direction: lower_direction(resolved),
        text_align: lower_text_align(resolved),
        writing_mode: lower_writing_mode(resolved),
        overflow: layout::Point::new(
            lower_overflow(resolved, Property::OverflowX),
            lower_overflow(resolved, Property::OverflowY),
        ),
        scrollbar_width: number(resolved, Property::ScrollbarWidth),
        position: lower_position(resolved),
        float: lower_float(resolved),
        clear: lower_clear(resolved),
        inset: lower_edges_auto_with_session(edges(resolved, Property::Inset), session)?,
        size: layout::Size::new(
            lower_dimension_with_session(resolved.width(), session)?,
            lower_dimension_with_session(resolved.height(), session)?,
        ),
        min_size: layout::Size::new(
            lower_dimension_with_session(length(resolved, Property::MinWidth), session)?,
            lower_dimension_with_session(length(resolved, Property::MinHeight), session)?,
        ),
        max_size: layout::Size::new(
            lower_dimension_with_session(length(resolved, Property::MaxWidth), session)?,
            lower_dimension_with_session(length(resolved, Property::MaxHeight), session)?,
        ),
        aspect_ratio: aspect_ratio(resolved)?,
        margin: lower_edges_auto_with_session(resolved.margin_edges(), session)?,
        padding: lower_edges_with_session(resolved.padding_edges(), session)?,
        border: lower_edges_with_session(resolved.border_width_edges(), session)?,
        gap: layout::Size::new(
            lower_gap_length_with_session(length(resolved, Property::ColumnGap), session)?,
            lower_gap_length_with_session(length(resolved, Property::RowGap), session)?,
        ),
        align_items: lower_align_items(resolved, Property::AlignItems),
        align_self: lower_align_items(resolved, Property::AlignSelf),
        justify_items: lower_align_items(resolved, Property::JustifyItems),
        justify_self: lower_align_items(resolved, Property::JustifySelf),
        align_content: lower_align_content(resolved, Property::AlignContent),
        justify_content: lower_align_content(resolved, Property::JustifyContent),
        flex_direction: lower_flex_direction(resolved),
        flex_wrap: lower_flex_wrap(resolved),
        flex_basis: lower_dimension_with_session(length(resolved, Property::FlexBasis), session)?,
        flex_grow: number(resolved, Property::FlexGrow),
        flex_shrink: number(resolved, Property::FlexShrink),
        grid_template_columns: lower_track_list_with_session(
            track_list(resolved, Property::GridTemplateColumns),
            session,
        )?,
        grid_template_rows: lower_track_list_with_session(
            track_list(resolved, Property::GridTemplateRows),
            session,
        )?,
        grid_template_areas: lower_grid_template_areas(grid_template_areas(resolved)),
        grid_auto_columns: lower_track_list_with_session(
            track_list(resolved, Property::GridAutoColumns),
            session,
        )?,
        grid_auto_rows: lower_track_list_with_session(
            track_list(resolved, Property::GridAutoRows),
            session,
        )?,
        grid_auto_flow: lower_grid_auto_flow(grid_auto_flow(resolved)),
        grid_flow_tolerance: lower_grid_flow_tolerance_with_session(resolved, session)?,
        grid_column: lower_grid_placement(
            grid_line(resolved, Property::GridColumnStart),
            grid_line(resolved, Property::GridColumnEnd),
        )?,
        grid_row: lower_grid_placement(
            grid_line(resolved, Property::GridRowStart),
            grid_line(resolved, Property::GridRowEnd),
        )?,
        raw_grid_column: lower_raw_grid_placement(
            grid_line(resolved, Property::GridColumnStart),
            grid_line(resolved, Property::GridColumnEnd),
        )?,
        raw_grid_row: lower_raw_grid_placement(
            grid_line(resolved, Property::GridRowStart),
            grid_line(resolved, Property::GridRowEnd),
        )?,
        ..layout::NodeInput::DEFAULT
    })
}

fn lower_display(display: Display) -> AdapterResult<layout::Display> {
    match display {
        Display::Block => Ok(layout::Display::Block),
        Display::Flex => Ok(layout::Display::Flex),
        Display::Grid => Ok(layout::Display::Grid),
        Display::InlineBlock => Ok(layout::Display::InlineBlock),
        Display::InlineGrid => Ok(layout::Display::InlineGrid),
        Display::GridLanes => Ok(layout::Display::GridLanes),
        Display::InlineGridLanes => Ok(layout::Display::InlineGridLanes),
        Display::None => Ok(layout::Display::None),
    }
}

fn lower_box_sizing(resolved: &Resolved) -> layout::BoxSizing {
    match resolved.get(Property::BoxSizing) {
        Value::BoxSizing(style::BoxSizing::ContentBox) => layout::BoxSizing::ContentBox,
        Value::BoxSizing(style::BoxSizing::BorderBox) => layout::BoxSizing::BorderBox,
        _ => layout::BoxSizing::default(),
    }
}

fn lower_position(resolved: &Resolved) -> layout::Position {
    match resolved.get(Property::Position) {
        Value::Position(style::LayoutPosition::Relative) => layout::Position::Relative,
        Value::Position(style::LayoutPosition::Absolute) => layout::Position::Absolute,
        _ => layout::Position::default(),
    }
}

fn lower_direction(resolved: &Resolved) -> layout::Direction {
    match resolved.get(Property::Direction) {
        Value::Direction(style::Direction::Ltr) => layout::Direction::Ltr,
        Value::Direction(style::Direction::Rtl) => layout::Direction::Rtl,
        _ => layout::Direction::default(),
    }
}

fn lower_overflow(resolved: &Resolved, property: Property) -> layout::Overflow {
    match resolved.get(property) {
        Value::Overflow(style::Overflow::Visible) => layout::Overflow::Visible,
        Value::Overflow(style::Overflow::Clip) => layout::Overflow::Clip,
        Value::Overflow(style::Overflow::Hidden) => layout::Overflow::Hidden,
        Value::Overflow(style::Overflow::Scroll) => layout::Overflow::Scroll,
        _ => layout::Overflow::default(),
    }
}

fn lower_float(resolved: &Resolved) -> layout::Float {
    match resolved.get(Property::Float) {
        Value::Float(style::Float::None) => layout::Float::None,
        Value::Float(style::Float::Left) => layout::Float::Left,
        Value::Float(style::Float::Right) => layout::Float::Right,
        _ => layout::Float::default(),
    }
}

fn lower_clear(resolved: &Resolved) -> layout::Clear {
    match resolved.get(Property::Clear) {
        Value::Clear(style::Clear::None) => layout::Clear::None,
        Value::Clear(style::Clear::Left) => layout::Clear::Left,
        Value::Clear(style::Clear::Right) => layout::Clear::Right,
        Value::Clear(style::Clear::Both) => layout::Clear::Both,
        _ => layout::Clear::default(),
    }
}

fn lower_text_align(resolved: &Resolved) -> layout::TextAlign {
    match resolved.get(Property::TextAlign) {
        Value::TextAlign(style::StyleTextAlign::Auto) => layout::TextAlign::Auto,
        Value::TextAlign(style::StyleTextAlign::LegacyLeft) => layout::TextAlign::LegacyLeft,
        Value::TextAlign(style::StyleTextAlign::LegacyRight) => layout::TextAlign::LegacyRight,
        Value::TextAlign(style::StyleTextAlign::LegacyCenter) => layout::TextAlign::LegacyCenter,
        _ => layout::TextAlign::default(),
    }
}

fn lower_writing_mode(resolved: &Resolved) -> layout::WritingMode {
    match resolved.get(Property::WritingMode) {
        Value::WritingMode(style::WritingMode::HorizontalTb) => layout::WritingMode::HorizontalTb,
        Value::WritingMode(style::WritingMode::VerticalLr) => layout::WritingMode::VerticalLr,
        Value::WritingMode(style::WritingMode::VerticalRl) => layout::WritingMode::VerticalRl,
        _ => layout::WritingMode::default(),
    }
}

fn lower_flex_direction(resolved: &Resolved) -> layout::FlexDirection {
    match resolved.get(Property::FlexDirection) {
        Value::FlexDirection(style::FlexDirection::Row) => layout::FlexDirection::Row,
        Value::FlexDirection(style::FlexDirection::Column) => layout::FlexDirection::Column,
        Value::FlexDirection(style::FlexDirection::RowReverse) => layout::FlexDirection::RowReverse,
        Value::FlexDirection(style::FlexDirection::ColumnReverse) => {
            layout::FlexDirection::ColumnReverse
        }
        _ => layout::FlexDirection::default(),
    }
}

fn lower_flex_wrap(resolved: &Resolved) -> layout::FlexWrap {
    match resolved.get(Property::FlexWrap) {
        Value::FlexWrap(style::FlexWrap::NoWrap) => layout::FlexWrap::NoWrap,
        Value::FlexWrap(style::FlexWrap::Wrap) => layout::FlexWrap::Wrap,
        Value::FlexWrap(style::FlexWrap::WrapReverse) => layout::FlexWrap::WrapReverse,
        _ => layout::FlexWrap::default(),
    }
}

fn lower_align_items(resolved: &Resolved, property: Property) -> Option<layout::AlignItems> {
    match resolved.get(property) {
        Value::AlignItems(style::AlignItems::Auto) => None,
        Value::AlignItems(style::AlignItems::Start) => Some(layout::AlignItems::Start),
        Value::AlignItems(style::AlignItems::End) => Some(layout::AlignItems::End),
        Value::AlignItems(style::AlignItems::FlexStart) => Some(layout::AlignItems::FlexStart),
        Value::AlignItems(style::AlignItems::FlexEnd) => Some(layout::AlignItems::FlexEnd),
        Value::AlignItems(style::AlignItems::Center) => Some(layout::AlignItems::Center),
        Value::AlignItems(style::AlignItems::SafeEnd) => Some(layout::AlignItems::SafeEnd),
        Value::AlignItems(style::AlignItems::SafeFlexEnd) => Some(layout::AlignItems::SafeFlexEnd),
        Value::AlignItems(style::AlignItems::SafeCenter) => Some(layout::AlignItems::SafeCenter),
        Value::AlignItems(style::AlignItems::Baseline) => Some(layout::AlignItems::Baseline),
        Value::AlignItems(style::AlignItems::LastBaseline) => {
            Some(layout::AlignItems::LastBaseline)
        }
        Value::AlignItems(style::AlignItems::Stretch) => Some(layout::AlignItems::Stretch),
        _ => None,
    }
}

fn lower_align_content(resolved: &Resolved, property: Property) -> Option<layout::AlignContent> {
    match resolved.get(property) {
        Value::AlignContent(style::AlignContent::Auto) => None,
        Value::AlignContent(style::AlignContent::Start) => Some(layout::AlignContent::Start),
        Value::AlignContent(style::AlignContent::End) => Some(layout::AlignContent::End),
        Value::AlignContent(style::AlignContent::FlexStart) => {
            Some(layout::AlignContent::FlexStart)
        }
        Value::AlignContent(style::AlignContent::FlexEnd) => Some(layout::AlignContent::FlexEnd),
        Value::AlignContent(style::AlignContent::Center) => Some(layout::AlignContent::Center),
        Value::AlignContent(style::AlignContent::SafeEnd) => Some(layout::AlignContent::SafeEnd),
        Value::AlignContent(style::AlignContent::SafeFlexEnd) => {
            Some(layout::AlignContent::SafeFlexEnd)
        }
        Value::AlignContent(style::AlignContent::SafeCenter) => {
            Some(layout::AlignContent::SafeCenter)
        }
        Value::AlignContent(style::AlignContent::Stretch) => Some(layout::AlignContent::Stretch),
        Value::AlignContent(style::AlignContent::SpaceBetween) => {
            Some(layout::AlignContent::SpaceBetween)
        }
        Value::AlignContent(style::AlignContent::SpaceEvenly) => {
            Some(layout::AlignContent::SpaceEvenly)
        }
        Value::AlignContent(style::AlignContent::SpaceAround) => {
            Some(layout::AlignContent::SpaceAround)
        }
        _ => None,
    }
}

fn lower_dimension(length: Length) -> AdapterResult<layout::Dimension> {
    Ok(match length {
        Length::Normal => return Err(unsupported("normal dimension length")),
        Length::Px(value) => layout::Dimension::px(value),
        Length::Percent(value) => layout::Dimension::percent(percent(value)),
        Length::Fill => layout::Dimension::fr(1.0),
        Length::Fit | Length::Auto => layout::Dimension::AUTO,
        Length::MinContent => layout::Dimension::MIN_CONTENT,
        Length::MaxContent => layout::Dimension::MAX_CONTENT,
        Length::Calc(_) => return Err(unsupported("calc dimension length")),
    })
}

fn lower_dimension_with_session(
    length: Length,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::Dimension> {
    Ok(match length {
        Length::Calc(calc) => {
            let id = session.push_calc_length(&calc);
            layout::Dimension::calc(id)
        }
        length => lower_dimension(length)?,
    })
}

fn lower_length_auto(length: Length) -> AdapterResult<layout::LengthAuto> {
    Ok(match length {
        Length::Px(value) => layout::LengthAuto::px(value),
        Length::Percent(value) => layout::LengthAuto::percent(percent(value)),
        Length::Auto => layout::LengthAuto::AUTO,
        Length::Calc(_)
        | Length::Normal
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => {
            return Err(unsupported("intrinsic, flexible, or calc edge length"));
        }
    })
}

fn lower_length_auto_with_session(
    length: Length,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::LengthAuto> {
    Ok(match length {
        Length::Calc(calc) => {
            let id = session.push_calc_length(&calc);
            layout::LengthAuto::calc(id)
        }
        length => lower_length_auto(length)?,
    })
}

fn lower_length(length: Length) -> AdapterResult<layout::Length> {
    Ok(match length {
        Length::Px(value) => layout::Length::px(value),
        Length::Percent(value) => layout::Length::percent(percent(value)),
        Length::Calc(_)
        | Length::Normal
        | Length::Auto
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => {
            return Err(unsupported("non-definite length"));
        }
    })
}

fn lower_length_with_session(
    length: Length,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::Length> {
    Ok(match length {
        Length::Calc(calc) => {
            let id = session.push_calc_length(&calc);
            layout::Length::calc(id)
        }
        length => lower_length(length)?,
    })
}

fn lower_gap_length_with_session(
    length: Length,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::Length> {
    Ok(match length {
        Length::Normal => layout::Length::NORMAL,
        length => lower_length_with_session(length, session)?,
    })
}

fn lower_edges_with_session(
    edges: Edges,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::Edges<layout::Length>> {
    Ok(layout::Edges::new(
        lower_length_with_session(edges.top, session)?,
        lower_length_with_session(edges.right, session)?,
        lower_length_with_session(edges.bottom, session)?,
        lower_length_with_session(edges.left, session)?,
    ))
}

fn lower_edges_auto_with_session(
    edges: Edges,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::Edges<layout::LengthAuto>> {
    Ok(layout::Edges::new(
        lower_length_auto_with_session(edges.top, session)?,
        lower_length_auto_with_session(edges.right, session)?,
        lower_length_auto_with_session(edges.bottom, session)?,
        lower_length_auto_with_session(edges.left, session)?,
    ))
}

fn lower_track_list_with_session(
    list: &GridTrackList,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<Vec<layout::TrackComponent>> {
    let mut lowered = Vec::new();
    for component in &list.components {
        match component {
            GridTrackComponent::Track(track) => {
                lowered.push(layout::TrackComponent::Track(
                    lower_track_sizing_with_session(track, session)?,
                ));
            }
            GridTrackComponent::Repeat(repeat) => {
                lowered.push(layout::TrackComponent::Repeat(
                    lower_track_repeat_with_session(repeat, session)?,
                ));
            }
            GridTrackComponent::LineNames(names) => {
                lowered.push(layout::TrackComponent::LineNames(names.to_strings()));
            }
            GridTrackComponent::Subgrid(subgrid) => {
                lowered.push(layout::TrackComponent::Subgrid(layout::SubgridTrack {
                    name_components: lower_subgrid_line_name_components(subgrid.name_components()),
                }));
            }
        }
    }
    Ok(lowered)
}

pub(super) fn lower_track_repeat_with_session(
    repeat: &TrackRepeat,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::TrackRepetition> {
    let mut components = Vec::new();
    for component in repeat.components() {
        match component {
            GridTrackComponent::Track(track) => {
                components.push(layout::TrackComponent::Track(
                    lower_track_sizing_with_session(track, session)?,
                ));
            }
            GridTrackComponent::LineNames(names) => {
                components.push(layout::TrackComponent::LineNames(names.to_strings()));
            }
            GridTrackComponent::Repeat(_) => return Err(unsupported("nested grid track repeat")),
            GridTrackComponent::Subgrid(_) => return Err(unsupported("subgrid track repeat")),
        }
    }

    match repeat.count_value() {
        TrackRepeatCount::Count(count) => {
            layout::TrackRepetition::count_components(usize::from(count.get()), components)
                .map_err(map_track_repetition_error)
        }
        TrackRepeatCount::AutoFill => layout::TrackRepetition::auto_fill_components(components)
            .map_err(map_track_repetition_error),
        TrackRepeatCount::AutoFit => layout::TrackRepetition::auto_fit_components(components)
            .map_err(map_track_repetition_error),
    }
}

pub(super) fn map_track_repetition_error(error: layout::TrackRepetitionError) -> AdapterError {
    adapter_error(
        "grid track repeat",
        format!("layout track repeat cannot be lowered: {error}"),
    )
}

fn lower_subgrid_line_name_components(
    components: &[style::SubgridLineNameComponent],
) -> Vec<layout::SubgridLineNameComponent> {
    components
        .iter()
        .map(|component| match component {
            style::SubgridLineNameComponent::LineNames(names) => {
                layout::SubgridLineNameComponent::LineNames(names.to_strings())
            }
            style::SubgridLineNameComponent::Repeat {
                count,
                line_name_sets,
            } => layout::SubgridLineNameComponent::Repeat {
                count: match count {
                    style::SubgridLineNameRepeatCount::Count(count) => {
                        layout::SubgridLineNameRepeatCount::Count(count.get())
                    }
                    style::SubgridLineNameRepeatCount::AutoFill => {
                        layout::SubgridLineNameRepeatCount::AutoFill
                    }
                },
                line_name_sets: line_name_sets.to_string_sets(),
            },
        })
        .collect()
}

fn lower_track_sizing_with_session(
    track: &TrackSizing,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::TrackSizing> {
    Ok(layout::TrackSizing::new(
        lower_min_track_sizing_with_session(&track.min, session)?,
        lower_max_track_sizing_with_session(&track.max, session)?,
    ))
}

fn lower_min_track_sizing_with_session(
    track: &MinTrackSizing,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::MinTrackSizing> {
    Ok(match track {
        MinTrackSizing::Length(length) => {
            layout::MinTrackSizing::Length(lower_length_with_session(length.clone(), session)?)
        }
        MinTrackSizing::Auto => layout::MinTrackSizing::AUTO,
        MinTrackSizing::MinContent => layout::MinTrackSizing::MIN_CONTENT,
        MinTrackSizing::MaxContent => layout::MinTrackSizing::MAX_CONTENT,
    })
}

fn lower_max_track_sizing_with_session(
    track: &MaxTrackSizing,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::MaxTrackSizing> {
    Ok(match track {
        MaxTrackSizing::Length(length) => {
            layout::MaxTrackSizing::Length(lower_length_with_session(length.clone(), session)?)
        }
        MaxTrackSizing::Flex(value) => layout::MaxTrackSizing::fr(*value),
        MaxTrackSizing::Auto => layout::MaxTrackSizing::AUTO,
        MaxTrackSizing::MinContent => layout::MaxTrackSizing::MIN_CONTENT,
        MaxTrackSizing::MaxContent => layout::MaxTrackSizing::MAX_CONTENT,
        MaxTrackSizing::FitContent(limit) => {
            layout::MaxTrackSizing::fit_content(lower_length_with_session(limit.clone(), session)?)
        }
    })
}

fn lower_grid_auto_flow(flow: GridAutoFlow) -> layout::GridAutoFlow {
    match flow {
        GridAutoFlow::Row => layout::GridAutoFlow::Row,
        GridAutoFlow::Column => layout::GridAutoFlow::Column,
        GridAutoFlow::RowDense => layout::GridAutoFlow::RowDense,
        GridAutoFlow::ColumnDense => layout::GridAutoFlow::ColumnDense,
    }
}

fn lower_grid_flow_tolerance_with_session(
    resolved: &Resolved,
    session: &mut LayoutLoweringSession,
) -> AdapterResult<layout::GridFlowTolerance> {
    Ok(match grid_flow_tolerance(resolved) {
        GridFlowTolerance::Normal => layout::GridFlowTolerance::Normal {
            font_size: font_size_scalar(resolved.font_size())?,
        },
        GridFlowTolerance::Length(length) => {
            layout::GridFlowTolerance::Length(lower_length_with_session(length, session)?)
        }
        GridFlowTolerance::Percent(value) => layout::GridFlowTolerance::Percent(percent(value)),
        GridFlowTolerance::Infinite => layout::GridFlowTolerance::Infinite,
    })
}

pub(super) fn font_size_scalar(length: Length) -> AdapterResult<f32> {
    Ok(match length {
        Length::Px(value) => value,
        Length::Percent(value) => 16.0 * percent(value),
        Length::Normal
        | Length::Auto
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => 16.0,
        Length::Calc(_) => return Err(unsupported("calc font size")),
    })
}

pub(super) fn lower_grid_placement(
    start: GridLine,
    end: GridLine,
) -> AdapterResult<layout::GridPlacement> {
    Ok(match (start, end) {
        (GridLine::Auto, GridLine::Auto) => layout::GridPlacement::AUTO,
        (GridLine::Line(line), GridLine::Auto) => {
            layout::GridPlacement::line(layout_grid_line(line.get())?)
        }
        (GridLine::Auto, GridLine::Line(line)) => {
            layout::GridPlacement::end_line(layout_grid_line(line.get())?)
        }
        (GridLine::Line(start), GridLine::Line(end)) => layout::GridPlacement::lines(
            layout_grid_line(start.get())?,
            layout_grid_line(end.get())?,
        ),
        (GridLine::Line(line), GridLine::Span(span)) => layout::GridPlacement::line_span(
            layout_grid_line(line.get())?,
            layout_grid_span(span.get())?,
        ),
        (GridLine::Span(span), GridLine::Line(line)) => layout::GridPlacement::span_line(
            layout_grid_span(span.get())?,
            layout_grid_line(line.get())?,
        ),
        (GridLine::Span(span), GridLine::Auto) | (GridLine::Auto, GridLine::Span(span)) => {
            layout::GridPlacement::try_span(usize::from(span.get()))
                .ok_or_else(|| adapter_error("grid placement", "grid span count cannot be zero"))?
        }
        (GridLine::Span(_), GridLine::Span(_)) => {
            return Err(unsupported("span-to-span grid placement"));
        }
        (GridLine::BareIdent(_) | GridLine::NamedLine { .. } | GridLine::NamedSpan { .. }, _)
        | (_, GridLine::BareIdent(_) | GridLine::NamedLine { .. } | GridLine::NamedSpan { .. }) => {
            layout::GridPlacement::AUTO
        }
    })
}

pub(super) fn layout_grid_line(line: i16) -> AdapterResult<layout::GridLine> {
    let line = isize::from(line);
    layout::GridLine::new(line)
        .ok_or_else(|| adapter_error("grid line", "grid line index cannot be zero"))
}

pub(super) fn layout_grid_span(span: u16) -> AdapterResult<layout::GridSpan> {
    let span = usize::from(span);
    layout::GridSpan::new(span)
        .ok_or_else(|| adapter_error("grid span", "grid span count cannot be zero"))
}

fn lower_raw_grid_line(line: GridLine) -> AdapterResult<layout::RawGridLine> {
    Ok(match line {
        GridLine::Auto => layout::RawGridLine::Auto,
        GridLine::Line(line) => layout::RawGridLine::Line(isize::from(line.get())),
        GridLine::Span(span) => layout::RawGridLine::Span(usize::from(span.get())),
        GridLine::BareIdent(name) => layout::RawGridLine::BareIdent(name.into_string()),
        GridLine::NamedLine { name, index } => layout::RawGridLine::NamedLine {
            name: name.into_string(),
            index: isize::from(index.get()),
        },
        GridLine::NamedSpan { name, index } => layout::RawGridLine::NamedSpan {
            name: name.into_string(),
            index: usize::from(index.get()),
        },
    })
}

fn lower_raw_grid_placement(
    start: GridLine,
    end: GridLine,
) -> AdapterResult<layout::RawGridPlacement> {
    Ok(layout::RawGridPlacement::new(
        lower_raw_grid_line(start)?,
        lower_raw_grid_line(end)?,
    ))
}

fn length(resolved: &Resolved, property: Property) -> Length {
    match resolved.get(property) {
        Value::Length(length) => length.clone(),
        _ => Length::Auto,
    }
}

fn edges(resolved: &Resolved, property: Property) -> Edges {
    match resolved.get(property) {
        Value::Edges(edges) => edges.clone(),
        _ => Edges::default(),
    }
}

fn number(resolved: &Resolved, property: Property) -> f32 {
    match resolved.get(property) {
        Value::Number(value) => *value,
        _ => 0.0,
    }
}

pub(super) fn layout_aspect_ratio(value: f32) -> AdapterResult<Option<layout::AspectRatio>> {
    if value == 0.0 {
        return Ok(None);
    }

    layout::AspectRatio::new(value).map(Some).ok_or_else(|| {
        adapter_error(
            "aspect-ratio",
            "aspect ratio must be finite and greater than zero",
        )
    })
}

fn aspect_ratio(resolved: &Resolved) -> AdapterResult<Option<layout::AspectRatio>> {
    layout_aspect_ratio(number(resolved, Property::AspectRatio))
}

fn track_list(resolved: &Resolved, property: Property) -> &GridTrackList {
    match resolved.get(property) {
        Value::GridTrackList(list) => list,
        _ => unreachable!("resolved grid track property stores a grid track list"),
    }
}

fn grid_template_areas(resolved: &Resolved) -> &style::GridTemplateAreas {
    match resolved.get(Property::GridTemplateAreas) {
        Value::GridTemplateAreas(areas) => areas,
        _ => unreachable!("resolved grid-template-areas stores grid template areas"),
    }
}

fn lower_grid_template_areas(areas: &style::GridTemplateAreas) -> layout::GridTemplateAreas {
    layout::GridTemplateAreas {
        rows: areas
            .rows
            .iter()
            .map(|row| layout::GridTemplateAreaRow {
                cells: row.cells.clone(),
            })
            .collect(),
    }
}

fn grid_line(resolved: &Resolved, property: Property) -> GridLine {
    match resolved.get(property) {
        Value::GridLine(line) => line.clone(),
        _ => GridLine::Auto,
    }
}

fn grid_auto_flow(resolved: &Resolved) -> GridAutoFlow {
    match resolved.get(Property::GridAutoFlow) {
        Value::GridAutoFlow(flow) => *flow,
        _ => GridAutoFlow::Row,
    }
}

fn grid_flow_tolerance(resolved: &Resolved) -> GridFlowTolerance {
    match resolved.get(Property::GridFlowTolerance) {
        Value::GridFlowTolerance(tolerance) => tolerance.clone(),
        _ => GridFlowTolerance::Normal,
    }
}

fn percent(value: f32) -> f32 {
    value / 100.0
}

fn lower_calc_expression(calc: &CalcLength) -> layout::CalcExpression {
    match calc {
        CalcLength::Px(value) => layout::CalcExpression::sum([layout::CalcTerm::px(*value)]),
        CalcLength::Percent(value) => {
            layout::CalcExpression::sum([layout::CalcTerm::percent(percent(*value))])
        }
        CalcLength::Sum(terms) => {
            let mut lowered = Vec::new();
            for term in terms {
                let sign = match term.operator() {
                    CalcOperator::Add => 1.0,
                    CalcOperator::Sub => -1.0,
                };
                collect_calc_terms(term.value(), sign, &mut lowered);
            }
            layout::CalcExpression::sum(lowered)
        }
    }
}

fn collect_calc_terms(calc: &CalcLength, sign: f32, output: &mut Vec<layout::CalcTerm>) {
    match calc {
        CalcLength::Px(value) => output.push(layout::CalcTerm::px(sign * *value)),
        CalcLength::Percent(value) => {
            output.push(layout::CalcTerm::percent(sign * percent(*value)));
        }
        CalcLength::Sum(terms) => {
            for term in terms {
                let term_sign = match term.operator() {
                    CalcOperator::Add => sign,
                    CalcOperator::Sub => -sign,
                };
                collect_calc_terms(term.value(), term_sign, output);
            }
        }
    }
}

fn unsupported(feature: &str) -> AdapterError {
    adapter_error(
        "layout",
        format!("{feature} cannot be lowered to layout style yet"),
    )
}

fn unsupported_property(property: Property, feature: &str) -> AdapterError {
    adapter_error(
        property_name(property),
        format!("{feature} cannot be lowered to layout style yet"),
    )
}

fn adapter_error(property: impl Into<String>, reason: impl Into<String>) -> AdapterError {
    AdapterError::new(AdapterErrorKind::StyleLayoutValue {
        property: property.into(),
        reason: reason.into(),
    })
}

fn property_name(property: Property) -> String {
    let debug = format!("{property:?}");
    let mut name = String::with_capacity(debug.len());
    for (index, character) in debug.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                name.push('-');
            }
            name.push(character.to_ascii_lowercase());
        } else {
            name.push(character);
        }
    }
    name
}
