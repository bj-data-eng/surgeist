//! Canonical identity and parser wiring for recognized CSS properties.
//!
//! The crate-private schema in this module is the single authority for the
//! frozen property set. Public identity values describe authored property names;
//! they do not apply cascade, substitute variables, or resolve authored values.

use crate::CssSpecifiedNonNegativeNumber;
use crate::alignment::*;
use crate::border_color::{
    CssBorderColorPair, CssBorderColorShorthand, CssParsedBorderColorShorthand,
};
use crate::border_radius::*;
use crate::border_style::*;
use crate::border_width::*;
use crate::box_spacing::*;
use crate::contain_intrinsic_size::*;
use crate::content_values::{CssContentValue, content_i01};
use crate::display::*;
use crate::flex::{
    CssFlexBasisRef, CssFlexBasisValue, CssFlexDirection, CssFlexFlow, CssFlexValue, CssFlexWrap,
};
use crate::font_controls::*;
use crate::font_settings::*;
use crate::font_variant::*;
use crate::gap::*;
use crate::inset::*;
use crate::overflow::CssOverflowValue;
use crate::overflow_controls::{CssOverflowClipMargin, CssScrollBehavior, CssScrollbarGutter};
use crate::scroll_snap::*;
use crate::sizing::*;
use crate::sizing_controls::*;
use crate::syntax::*;
use crate::text_alignment::*;
use crate::text_spacing::CssTextSpacingAdjustment;
use crate::{
    CssComponentValueRef, CssContainer, CssContainerNames, CssContainerType, CssSpecifiedLength,
    CssSpecifiedLengthPercentage, CssValueTokenRef,
};
use crate::{CssFontSize, CssFontStyle, CssFontWeight, CssFontWidth, CssLineHeight};

macro_rules! property_schema {
    ($callback:ident, $input:ident, $numeric:ident) => {
        // Flexbox 1 normative Appendix B supplies the twelve name-equivalent
        // -webkit- aliases; their canonical property definitions keep their own sources.
        $callback! {
            $input, $numeric;
            All, "all", [], "baseline.property.all", CssAllDeclaredValue, CssAllPropertyValue, CssAllPropertyValueRepresentation, parse_all_property, { parse_all_property($input)? }, expansion = universal { exclude_custom: true, excluded: [Direction, UnicodeBidi] };
            ContainerName, "container-name", [], "official.property.container-name", CssContainerNames, CssContainerNamePropertyValue, CssContainerNamePropertyValueRepresentation, parse_container_names, { parse_container_names($input)? }, expansion = longhand { wrapper: existing, value: CssContainerNames, accessor: names, inherited: false, initial_kind: value, initial: CssContainerNames::None };
            ContainerType, "container-type", [], "official.property.container-type", CssContainerType, CssContainerTypePropertyValue, CssContainerTypePropertyValueRepresentation, parse_container_type, { parse_container_type($input)? }, expansion = longhand { wrapper: existing, value: CssContainerType, accessor: container_type, inherited: false, initial_kind: value, initial: CssContainerType::Normal };
            Container, "container", [], "official.property.container", CssContainer, CssContainerPropertyValue, CssContainerPropertyValueRepresentation, parse_container, { parse_container($input)? }, expansion = shorthand { wrapper: existing, accessor: container, members: [ ContainerName => |value: &CssContainer| Some(value.names().clone()), ContainerType => |value: &CssContainer| Some(value.container_type()) ], reset_only: [] };
            Display, "display", [], "baseline.property.display", CssDisplay, CssDisplayPropertyValue, CssDisplayPropertyValueRepresentation, parse_display, { parse_display($input)? }, expansion = longhand { wrapper: existing, value: crate::CssDisplayValue, accessor: value, inherited: false, initial_kind: value, initial: crate::CssDisplayValue::OutsideInside { outside: crate::CssDisplayOutside::Inline, inside: crate::CssDisplayInside::Flow } };
            BoxSizing, "box-sizing", [], "baseline.property.box-sizing", CssBoxSizing, CssBoxSizingPropertyValue, CssBoxSizingPropertyValueRepresentation, parse_box_sizing, { parse_box_sizing($input)? }, expansion = longhand { wrapper: fallback, value: CssBoxSizing, accessor: current, inherited: false, initial_kind: value, initial: CssBoxSizing::ContentBox };
            BorderCollapse, "border-collapse", [], "official.property.border-collapse", CssBorderCollapse, CssBorderCollapsePropertyValue, CssBorderCollapsePropertyValueRepresentation, parse_border_collapse, { parse_border_collapse($input)? }, expansion = longhand { wrapper: existing, value: CssBorderCollapse, accessor: collapse, inherited: true, initial_kind: value, initial: CssBorderCollapse::Separate };
            BorderSpacing, "border-spacing", [], "official.property.border-spacing", CssBorderSpacing, CssBorderSpacingPropertyValue, CssBorderSpacingPropertyValueRepresentation, parse_border_spacing, { parse_border_spacing($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssBorderSpacing, accessor: spacing, inherited: true, initial_kind: value, initial: CssBorderSpacing::try_new(CssLength::Zero, CssLength::Zero).expect("valid border-spacing initial") };
            CaptionSide, "caption-side", [], "official.property.caption-side", CssCaptionSide, CssCaptionSidePropertyValue, CssCaptionSidePropertyValueRepresentation, parse_caption_side, { parse_caption_side($input)? }, expansion = longhand { wrapper: existing, value: CssCaptionSide, accessor: side, inherited: true, initial_kind: value, initial: CssCaptionSide::Top };
            Clip, "clip", [], "official.property.clip", CssClip, CssClipPropertyValue, CssClipPropertyValueRepresentation, parse_clip, { parse_clip($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssClip, accessor: clip, inherited: false, initial_kind: value, initial: CssClip::Auto };
            EmptyCells, "empty-cells", [], "official.property.empty-cells", CssEmptyCells, CssEmptyCellsPropertyValue, CssEmptyCellsPropertyValueRepresentation, parse_empty_cells, { parse_empty_cells($input)? }, expansion = longhand { wrapper: existing, value: CssEmptyCells, accessor: cells, inherited: true, initial_kind: value, initial: CssEmptyCells::Show };
            Orphans, "orphans", [], "official.property.orphans", CssPageLineMinimum, CssOrphansPropertyValue, CssOrphansPropertyValueRepresentation, parse_page_line_minimum, { parse_page_line_minimum($input, $numeric, "orphans")? }, expansion = longhand { wrapper: existing, value: CssPageLineMinimum, accessor: minimum, inherited: true, initial_kind: value, initial: CssPageLineMinimum::initial() };
            BreakAfter, "break-after", [], "official.property.break-after", CssBreakBetween, CssBreakAfterPropertyValue, CssBreakAfterPropertyValueRepresentation, parse_break_between, { parse_break_between($input)? }, expansion = longhand { wrapper: additive, value: CssBreakBetween, accessor: current, inherited: false, initial_kind: value, initial: CssBreakBetween::Auto };
            BreakBefore, "break-before", [], "official.property.break-before", CssBreakBetween, CssBreakBeforePropertyValue, CssBreakBeforePropertyValueRepresentation, parse_break_between, { parse_break_between($input)? }, expansion = longhand { wrapper: additive, value: CssBreakBetween, accessor: current, inherited: false, initial_kind: value, initial: CssBreakBetween::Auto };
            BreakInside, "break-inside", [], "official.property.break-inside", CssBreakInside, CssBreakInsidePropertyValue, CssBreakInsidePropertyValueRepresentation, parse_break_inside, { parse_break_inside($input)? }, expansion = longhand { wrapper: additive, value: CssBreakInside, accessor: current, inherited: false, initial_kind: value, initial: CssBreakInside::Auto };
            Quotes, "quotes", [], "official.property.quotes", CssQuotes, CssQuotesPropertyValue, CssQuotesPropertyValueRepresentation, parse_quotes, { parse_quotes($input)? }, expansion = longhand { wrapper: existing, value: CssQuotes, accessor: quotes, inherited: true, initial_kind: value, initial: CssQuotes::Auto };
            TableLayout, "table-layout", [], "official.property.table-layout", CssTableLayout, CssTableLayoutPropertyValue, CssTableLayoutPropertyValueRepresentation, parse_table_layout, { parse_table_layout($input)? }, expansion = longhand { wrapper: existing, value: CssTableLayout, accessor: layout, inherited: false, initial_kind: value, initial: CssTableLayout::Auto };
            ScrollSnapType, "scroll-snap-type", [], "official.property.scroll-snap-type", CssScrollSnapType, CssScrollSnapTypePropertyValue, CssScrollSnapTypePropertyValueRepresentation, parse_scroll_snap_type, { parse_scroll_snap_type($input)? }, expansion = longhand { wrapper: additive, value: CssScrollSnapType, accessor: current, inherited: false, initial_kind: value, initial: CssScrollSnapType::None };
            ScrollSnapAlign, "scroll-snap-align", [], "official.property.scroll-snap-align", CssScrollSnapAlign, CssScrollSnapAlignPropertyValue, CssScrollSnapAlignPropertyValueRepresentation, parse_scroll_snap_align, { parse_scroll_snap_align($input)? }, expansion = longhand { wrapper: additive, value: CssScrollSnapAlign, accessor: current, inherited: false, initial_kind: value, initial: CssScrollSnapAlign::new(CssScrollSnapAlignment::None, None) };
            ScrollSnapStop, "scroll-snap-stop", [], "official.property.scroll-snap-stop", CssScrollSnapStop, CssScrollSnapStopPropertyValue, CssScrollSnapStopPropertyValueRepresentation, parse_scroll_snap_stop, { parse_scroll_snap_stop($input)? }, expansion = longhand { wrapper: additive, value: CssScrollSnapStop, accessor: current, inherited: false, initial_kind: value, initial: CssScrollSnapStop::Normal };
            ScrollPaddingTop, "scroll-padding-top", [], "official.property.scroll-padding-top", CssScrollPaddingValue, CssScrollPaddingTopPropertyValue, CssScrollPaddingTopPropertyValueRepresentation, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssScrollPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingRight, "scroll-padding-right", [], "official.property.scroll-padding-right", CssScrollPaddingValue, CssScrollPaddingRightPropertyValue, CssScrollPaddingRightPropertyValueRepresentation, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssScrollPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingBottom, "scroll-padding-bottom", [], "official.property.scroll-padding-bottom", CssScrollPaddingValue, CssScrollPaddingBottomPropertyValue, CssScrollPaddingBottomPropertyValueRepresentation, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssScrollPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingLeft, "scroll-padding-left", [], "official.property.scroll-padding-left", CssScrollPaddingValue, CssScrollPaddingLeftPropertyValue, CssScrollPaddingLeftPropertyValueRepresentation, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssScrollPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingBlockStart, "scroll-padding-block-start", [], "official.property.scroll-padding-block-start", CssScrollPaddingValue, CssScrollPaddingBlockStartPropertyValue, CssScrollPaddingBlockStartPropertyValueRepresentation, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssScrollPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingBlockEnd, "scroll-padding-block-end", [], "official.property.scroll-padding-block-end", CssScrollPaddingValue, CssScrollPaddingBlockEndPropertyValue, CssScrollPaddingBlockEndPropertyValueRepresentation, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssScrollPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingInlineStart, "scroll-padding-inline-start", [], "official.property.scroll-padding-inline-start", CssScrollPaddingValue, CssScrollPaddingInlineStartPropertyValue, CssScrollPaddingInlineStartPropertyValueRepresentation, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssScrollPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingInlineEnd, "scroll-padding-inline-end", [], "official.property.scroll-padding-inline-end", CssScrollPaddingValue, CssScrollPaddingInlineEndPropertyValue, CssScrollPaddingInlineEndPropertyValueRepresentation, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssScrollPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollMarginTop, "scroll-margin-top", [], "official.property.scroll-margin-top", CssSpecifiedLength, CssScrollMarginTopPropertyValue, CssScrollMarginTopPropertyValueRepresentation, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSpecifiedLength, accessor: current, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginRight, "scroll-margin-right", [], "official.property.scroll-margin-right", CssSpecifiedLength, CssScrollMarginRightPropertyValue, CssScrollMarginRightPropertyValueRepresentation, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSpecifiedLength, accessor: current, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginBottom, "scroll-margin-bottom", [], "official.property.scroll-margin-bottom", CssSpecifiedLength, CssScrollMarginBottomPropertyValue, CssScrollMarginBottomPropertyValueRepresentation, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSpecifiedLength, accessor: current, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginLeft, "scroll-margin-left", [], "official.property.scroll-margin-left", CssSpecifiedLength, CssScrollMarginLeftPropertyValue, CssScrollMarginLeftPropertyValueRepresentation, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSpecifiedLength, accessor: current, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginBlockStart, "scroll-margin-block-start", [], "official.property.scroll-margin-block-start", CssSpecifiedLength, CssScrollMarginBlockStartPropertyValue, CssScrollMarginBlockStartPropertyValueRepresentation, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSpecifiedLength, accessor: current, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginBlockEnd, "scroll-margin-block-end", [], "official.property.scroll-margin-block-end", CssSpecifiedLength, CssScrollMarginBlockEndPropertyValue, CssScrollMarginBlockEndPropertyValueRepresentation, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSpecifiedLength, accessor: current, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginInlineStart, "scroll-margin-inline-start", [], "official.property.scroll-margin-inline-start", CssSpecifiedLength, CssScrollMarginInlineStartPropertyValue, CssScrollMarginInlineStartPropertyValueRepresentation, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSpecifiedLength, accessor: current, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginInlineEnd, "scroll-margin-inline-end", [], "official.property.scroll-margin-inline-end", CssSpecifiedLength, CssScrollMarginInlineEndPropertyValue, CssScrollMarginInlineEndPropertyValueRepresentation, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSpecifiedLength, accessor: current, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollPaddingBlock, "scroll-padding-block", [], "official.property.scroll-padding-block", CssScrollPaddingPair, CssScrollPaddingBlockPropertyValue, CssScrollPaddingBlockPropertyValueRepresentation, parse_scroll_padding_pair, { parse_scroll_padding_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ ScrollPaddingBlockStart => |value: &CssScrollPaddingPair| Some(value.start().clone()), ScrollPaddingBlockEnd => |value: &CssScrollPaddingPair| Some(value.end().clone()) ], reset_only: [] };
            ScrollPaddingInline, "scroll-padding-inline", [], "official.property.scroll-padding-inline", CssScrollPaddingPair, CssScrollPaddingInlinePropertyValue, CssScrollPaddingInlinePropertyValueRepresentation, parse_scroll_padding_pair, { parse_scroll_padding_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ ScrollPaddingInlineStart => |value: &CssScrollPaddingPair| Some(value.start().clone()), ScrollPaddingInlineEnd => |value: &CssScrollPaddingPair| Some(value.end().clone()) ], reset_only: [] };
            ScrollMarginBlock, "scroll-margin-block", [], "official.property.scroll-margin-block", CssScrollMarginPair, CssScrollMarginBlockPropertyValue, CssScrollMarginBlockPropertyValueRepresentation, parse_scroll_margin_pair, { parse_scroll_margin_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ ScrollMarginBlockStart => |value: &CssScrollMarginPair| Some(value.start().clone()), ScrollMarginBlockEnd => |value: &CssScrollMarginPair| Some(value.end().clone()) ], reset_only: [] };
            ScrollMarginInline, "scroll-margin-inline", [], "official.property.scroll-margin-inline", CssScrollMarginPair, CssScrollMarginInlinePropertyValue, CssScrollMarginInlinePropertyValueRepresentation, parse_scroll_margin_pair, { parse_scroll_margin_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ ScrollMarginInlineStart => |value: &CssScrollMarginPair| Some(value.start().clone()), ScrollMarginInlineEnd => |value: &CssScrollMarginPair| Some(value.end().clone()) ], reset_only: [] };
            ScrollPadding, "scroll-padding", [], "official.property.scroll-padding", CssScrollPaddingShorthand, CssScrollPaddingPropertyValue, CssScrollPaddingPropertyValueRepresentation, parse_scroll_padding_shorthand, { parse_scroll_padding_shorthand($input, $numeric)? }, expansion = unresolved { wrapper: additive, reason: CssUnresolvedStandard::LogicalShorthandResetMembership };
            ScrollMargin, "scroll-margin", [], "official.property.scroll-margin", CssScrollMarginShorthand, CssScrollMarginPropertyValue, CssScrollMarginPropertyValueRepresentation, parse_scroll_margin_shorthand, { parse_scroll_margin_shorthand($input, $numeric)? }, expansion = unresolved { wrapper: additive, reason: CssUnresolvedStandard::LogicalShorthandResetMembership };
            Widows, "widows", [], "official.property.widows", CssPageLineMinimum, CssWidowsPropertyValue, CssWidowsPropertyValueRepresentation, parse_page_line_minimum, { parse_page_line_minimum($input, $numeric, "widows")? }, expansion = longhand { wrapper: existing, value: CssPageLineMinimum, accessor: minimum, inherited: true, initial_kind: value, initial: CssPageLineMinimum::initial() };
            WordSpacing, "word-spacing", [], "official.property.word-spacing", CssWordSpacing, CssWordSpacingPropertyValue, CssWordSpacingPropertyValueRepresentation, parse_word_spacing, { parse_word_spacing($input, $numeric)? }, expansion = longhand { wrapper: existing, value: crate::CssTextSpacingAdjustment, accessor: spacing, inherited: true, initial_kind: value, initial: crate::CssTextSpacingAdjustment::Normal };
            Position, "position", [], "baseline.property.position", CssLayoutPosition, CssPositionPropertyValue, CssPositionPropertyValueRepresentation, parse_position, { parse_position($input)? }, expansion = longhand { wrapper: fallback, value: CssLayoutPosition, accessor: current, inherited: false, initial_kind: value, initial: CssLayoutPosition::Static };
            Direction, "direction", [], "baseline.property.direction", CssDirection, CssDirectionPropertyValue, CssDirectionPropertyValueRepresentation, parse_direction, { parse_direction($input)? }, expansion = longhand { wrapper: fallback, value: CssDirection, accessor: current, inherited: true, initial_kind: value, initial: CssDirection::Ltr };
            Overflow, "overflow", [], "baseline.property.overflow", CssOverflowValue, CssOverflowPropertyValue, CssOverflowPropertyValueRepresentation, parse_overflow_value, { parse_overflow_value($input)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ OverflowX => |value: &CssOverflowValue| Some(value.x()), OverflowY => |value: &CssOverflowValue| Some(value.y()) ], reset_only: [] };
            OverflowX, "overflow-x", [], "baseline.property.overflow-x", CssOverflow, CssOverflowXPropertyValue, CssOverflowXPropertyValueRepresentation, parse_overflow, { parse_overflow($input)? }, expansion = longhand { wrapper: existing, value: CssOverflow, accessor: current, inherited: false, initial_kind: value, initial: CssOverflow::Visible };
            OverflowY, "overflow-y", [], "baseline.property.overflow-y", CssOverflow, CssOverflowYPropertyValue, CssOverflowYPropertyValueRepresentation, parse_overflow, { parse_overflow($input)? }, expansion = longhand { wrapper: existing, value: CssOverflow, accessor: current, inherited: false, initial_kind: value, initial: CssOverflow::Visible };
            OverflowBlock, "overflow-block", [], "ext.property.overflow-block", CssOverflow, CssOverflowBlockPropertyValue, CssOverflowBlockPropertyValueRepresentation, parse_overflow, { parse_overflow($input)? }, expansion = longhand { wrapper: additive, value: CssOverflow, accessor: current, inherited: false, initial_kind: value, initial: CssOverflow::Visible };
            OverflowInline, "overflow-inline", [], "ext.property.overflow-inline", CssOverflow, CssOverflowInlinePropertyValue, CssOverflowInlinePropertyValueRepresentation, parse_overflow, { parse_overflow($input)? }, expansion = longhand { wrapper: additive, value: CssOverflow, accessor: current, inherited: false, initial_kind: value, initial: CssOverflow::Visible };
            OverflowClipMargin, "overflow-clip-margin", [], "ext.property.overflow-clip-margin", CssOverflowClipMargin, CssOverflowClipMarginPropertyValue, CssOverflowClipMarginPropertyValueRepresentation, parse_overflow_clip_margin, { parse_overflow_clip_margin($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssOverflowClipMargin, accessor: current, inherited: false, initial_kind: value, initial: CssOverflowClipMargin::initial() };
            ScrollBehavior, "scroll-behavior", [], "ext.property.scroll-behavior", CssScrollBehavior, CssScrollBehaviorPropertyValue, CssScrollBehaviorPropertyValueRepresentation, parse_scroll_behavior, { parse_scroll_behavior($input)? }, expansion = longhand { wrapper: additive, value: CssScrollBehavior, accessor: current, inherited: false, initial_kind: value, initial: CssScrollBehavior::Auto };
            ScrollbarGutter, "scrollbar-gutter", [], "ext.property.scrollbar-gutter", CssScrollbarGutter, CssScrollbarGutterPropertyValue, CssScrollbarGutterPropertyValueRepresentation, parse_scrollbar_gutter, { parse_scrollbar_gutter($input)? }, expansion = longhand { wrapper: additive, value: CssScrollbarGutter, accessor: current, inherited: false, initial_kind: value, initial: CssScrollbarGutter::Auto };
            FlexDirection, "flex-direction", ["-webkit-flex-direction"], "baseline.property.flex-direction", CssFlexDirection, CssFlexDirectionPropertyValue, CssFlexDirectionPropertyValueRepresentation, parse_flex_direction, { parse_flex_direction($input)? }, expansion = longhand { wrapper: existing, value: CssFlexDirection, accessor: current, inherited: false, initial_kind: value, initial: CssFlexDirection::Row };
            FlexFlow, "flex-flow", ["-webkit-flex-flow"], "official.property.flex-flow", CssFlexFlow, CssFlexFlowPropertyValue, CssFlexFlowPropertyValueRepresentation, parse_flex_flow, { parse_flex_flow($input)? }, expansion = shorthand { wrapper: existing, accessor: flow, members: [ FlexDirection => |value: &CssFlexFlow| Some(value.direction()), FlexWrap => |value: &CssFlexFlow| Some(value.wrap()) ], reset_only: [] };
            FlexWrap, "flex-wrap", ["-webkit-flex-wrap"], "baseline.property.flex-wrap", CssFlexWrap, CssFlexWrapPropertyValue, CssFlexWrapPropertyValueRepresentation, parse_flex_wrap, { parse_flex_wrap($input)? }, expansion = longhand { wrapper: existing, value: CssFlexWrap, accessor: current, inherited: false, initial_kind: value, initial: CssFlexWrap::NoWrap };
            Float, "float", [], "baseline.property.float", CssFloat, CssFloatPropertyValue, CssFloatPropertyValueRepresentation, parse_float, { parse_float($input)? }, expansion = longhand { wrapper: existing, value: CssFloat, accessor: current, inherited: false, initial_kind: value, initial: CssFloat::None };
            Clear, "clear", [], "baseline.property.clear", CssClear, CssClearPropertyValue, CssClearPropertyValueRepresentation, parse_clear, { parse_clear($input)? }, expansion = longhand { wrapper: existing, value: CssClear, accessor: current, inherited: false, initial_kind: value, initial: CssClear::None };
            AlignContent, "align-content", ["-webkit-align-content"], "baseline.property.align-content", CssAlignment, CssAlignContentPropertyValue, CssAlignContentPropertyValueRepresentation, parse_align_content_value, { parse_align_content_value($input)? }, expansion = longhand { wrapper: existing, value: CssAlignContentValue, accessor: current, inherited: false, initial_kind: value, initial: CssAlignContentValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid align-content initial") };
            JustifyContent, "justify-content", ["-webkit-justify-content"], "baseline.property.justify-content", CssAlignment, CssJustifyContentPropertyValue, CssJustifyContentPropertyValueRepresentation, parse_justify_content_value, { parse_justify_content_value($input)? }, expansion = longhand { wrapper: existing, value: CssJustifyContentValue, accessor: current, inherited: false, initial_kind: value, initial: CssJustifyContentValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid justify-content initial") };
            AlignItems, "align-items", ["-webkit-align-items"], "baseline.property.align-items", CssAlignItems, CssAlignItemsPropertyValue, CssAlignItemsPropertyValueRepresentation, parse_align_items_value, { parse_align_items_value($input)? }, expansion = longhand { wrapper: existing, value: CssAlignItemsValue, accessor: current, inherited: false, initial_kind: value, initial: CssAlignItemsValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid align-items initial") };
            AlignSelf, "align-self", ["-webkit-align-self"], "baseline.property.align-self", CssAlignItems, CssAlignSelfPropertyValue, CssAlignSelfPropertyValueRepresentation, parse_align_self_value, { parse_align_self_value($input)? }, expansion = longhand { wrapper: existing, value: CssAlignSelfValue, accessor: current, inherited: false, initial_kind: value, initial: CssAlignSelfValue::try_new(CssAlignmentValue::Auto).expect("valid align-self initial") };
            JustifyItems, "justify-items", [], "baseline.property.justify-items", CssAlignItems, CssJustifyItemsPropertyValue, CssJustifyItemsPropertyValueRepresentation, parse_justify_items_value, { parse_justify_items_value($input)? }, expansion = longhand { wrapper: existing, value: CssJustifyItemsValue, accessor: current, inherited: false, initial_kind: value, initial: CssJustifyItemsValue::try_new(CssAlignmentValue::Legacy(None)).expect("valid justify-items initial") };
            JustifySelf, "justify-self", [], "baseline.property.justify-self", CssAlignItems, CssJustifySelfPropertyValue, CssJustifySelfPropertyValueRepresentation, parse_justify_self_value, { parse_justify_self_value($input)? }, expansion = longhand { wrapper: existing, value: CssJustifySelfValue, accessor: current, inherited: false, initial_kind: value, initial: CssJustifySelfValue::try_new(CssAlignmentValue::Auto).expect("valid justify-self initial") };
            PlaceContent, "place-content", [], "baseline.property.place-content", CssPlaceAlignment, CssPlaceContentPropertyValue, CssPlaceContentPropertyValueRepresentation, parse_place_content_value, { parse_place_content_value($input)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ AlignContent => |value: &CssPlaceContentValue| Some(value.align()), JustifyContent => |value: &CssPlaceContentValue| Some(value.justify()) ], reset_only: [] };
            PlaceItems, "place-items", [], "baseline.property.place-items", CssPlaceAlignment, CssPlaceItemsPropertyValue, CssPlaceItemsPropertyValueRepresentation, parse_place_items_value, { parse_place_items_value($input)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ AlignItems => |value: &CssPlaceItemsValue| Some(value.align()), JustifyItems => |value: &CssPlaceItemsValue| Some(value.justify()) ], reset_only: [] };
            PlaceSelf, "place-self", [], "baseline.property.place-self", CssPlaceAlignment, CssPlaceSelfPropertyValue, CssPlaceSelfPropertyValueRepresentation, parse_place_self_value, { parse_place_self_value($input)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ AlignSelf => |value: &CssPlaceSelfValue| Some(value.align()), JustifySelf => |value: &CssPlaceSelfValue| Some(value.justify()) ], reset_only: [] };
            Visibility, "visibility", [], "baseline.property.visibility", CssVisibility, CssVisibilityPropertyValue, CssVisibilityPropertyValueRepresentation, parse_visibility, { parse_visibility($input)? }, expansion = longhand { wrapper: fallback, value: CssVisibility, accessor: current, inherited: true, initial_kind: value, initial: CssVisibility::Visible };
            Content, "content", [], "baseline.property.content", crate::CssContentValue, CssContentPropertyValue, CssContentPropertyValueRepresentation, parse_content, { parse_content($input, $numeric)? }, expansion = longhand { wrapper: existing, value: crate::CssContentValue, accessor: current, inherited: false, initial_kind: value, initial: crate::CssContentValue::Normal };
            ContentVisibility, "content-visibility", [], "baseline.property.content-visibility", CssContentVisibility, CssContentVisibilityPropertyValue, CssContentVisibilityPropertyValueRepresentation, parse_content_visibility, { parse_content_visibility($input)? }, expansion = longhand { wrapper: fallback, value: CssContentVisibility, accessor: current, inherited: false, initial_kind: value, initial: CssContentVisibility::Visible };
            ListStyleType, "list-style-type", [], "baseline.property.list-style-type", crate::CssListStyleTypeValue, CssListStyleTypePropertyValue, CssListStyleTypePropertyValueRepresentation, parse_list_style_type, { parse_list_style_type($input, $numeric)? }, expansion = longhand { wrapper: existing, value: crate::CssListStyleTypeValue, accessor: current, inherited: true, initial_kind: value, initial: crate::CssListStyleTypeValue::initial() };
            ListStylePosition, "list-style-position", [], "baseline.property.list-style-position", CssListStylePosition, CssListStylePositionPropertyValue, CssListStylePositionPropertyValueRepresentation, parse_list_style_position, { parse_list_style_position($input)? }, expansion = longhand { wrapper: existing, value: CssListStylePosition, accessor: current, inherited: true, initial_kind: value, initial: CssListStylePosition::Outside };
            ListStyleImage, "list-style-image", [], "baseline.property.list-style-image", CssImageValue, CssListStyleImagePropertyValue, CssListStyleImagePropertyValueRepresentation, parse_list_style_image, { parse_list_style_image($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssImageValue, accessor: current, inherited: true, initial_kind: value, initial: CssImageValue::None };
            ListStyle, "list-style", [], "baseline.property.list-style", crate::CssListStyleValue, CssListStylePropertyValue, CssListStylePropertyValueRepresentation, parse_list_style, { parse_list_style($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ ListStylePosition => |value: &crate::CssListStyleValue| value.position(), ListStyleImage => |value: &crate::CssListStyleValue| value.image().cloned(), ListStyleType => |value: &crate::CssListStyleValue| value.style_type().cloned() ], reset_only: [] };
            MarkerSide, "marker-side", [], "official.property.marker-side", crate::CssMarkerSide, CssMarkerSidePropertyValue, CssMarkerSidePropertyValueRepresentation, parse_marker_side, { parse_marker_side($input)? }, expansion = longhand { wrapper: additive, value: crate::CssMarkerSide, accessor: current, inherited: true, initial_kind: value, initial: crate::CssMarkerSide::MatchSelf };
            CounterReset, "counter-reset", [], "baseline.property.counter-reset", crate::CssCounterChangesValue, CssCounterResetPropertyValue, CssCounterResetPropertyValueRepresentation, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { wrapper: existing, value: crate::CssCounterChangesValue, accessor: current, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            CounterIncrement, "counter-increment", [], "baseline.property.counter-increment", crate::CssCounterChangesValue, CssCounterIncrementPropertyValue, CssCounterIncrementPropertyValueRepresentation, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { wrapper: existing, value: crate::CssCounterChangesValue, accessor: current, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            CounterSet, "counter-set", [], "baseline.property.counter-set", crate::CssCounterChangesValue, CssCounterSetPropertyValue, CssCounterSetPropertyValueRepresentation, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { wrapper: existing, value: crate::CssCounterChangesValue, accessor: current, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            Width, "width", [], "baseline.property.width", CssSizeValue, CssWidthPropertyValue, CssWidthPropertyValueRepresentation, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            Height, "height", [], "baseline.property.height", CssSizeValue, CssHeightPropertyValue, CssHeightPropertyValueRepresentation, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            InlineSize, "inline-size", [], "official.property.inline-size", CssSizeValue, CssInlineSizePropertyValue, CssInlineSizePropertyValueRepresentation, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            BlockSize, "block-size", [], "official.property.block-size", CssSizeValue, CssBlockSizePropertyValue, CssBlockSizePropertyValueRepresentation, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MinWidth, "min-width", [], "baseline.property.min-width", CssSizeValue, CssMinWidthPropertyValue, CssMinWidthPropertyValueRepresentation, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MinHeight, "min-height", [], "baseline.property.min-height", CssSizeValue, CssMinHeightPropertyValue, CssMinHeightPropertyValueRepresentation, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MinInlineSize, "min-inline-size", [], "official.property.min-inline-size", CssSizeValue, CssMinInlineSizePropertyValue, CssMinInlineSizePropertyValueRepresentation, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MinBlockSize, "min-block-size", [], "official.property.min-block-size", CssSizeValue, CssMinBlockSizePropertyValue, CssMinBlockSizePropertyValueRepresentation, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MaxWidth, "max-width", [], "baseline.property.max-width", CssMaxSizeValue, CssMaxWidthPropertyValue, CssMaxWidthPropertyValueRepresentation, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMaxSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE };
            MaxHeight, "max-height", [], "baseline.property.max-height", CssMaxSizeValue, CssMaxHeightPropertyValue, CssMaxHeightPropertyValueRepresentation, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMaxSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE };
            MaxInlineSize, "max-inline-size", [], "official.property.max-inline-size", CssMaxSizeValue, CssMaxInlineSizePropertyValue, CssMaxInlineSizePropertyValueRepresentation, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMaxSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE };
            MaxBlockSize, "max-block-size", [], "official.property.max-block-size", CssMaxSizeValue, CssMaxBlockSizePropertyValue, CssMaxBlockSizePropertyValueRepresentation, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMaxSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE };
            Size, "size", [], "ext.property.size", CssSizePair, CssSizePropertyValue, CssSizePropertyValueRepresentation, parse_size_pair, { parse_size_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ Width => |value: &CssSizePair| Some(value.width().clone()), Height => |value: &CssSizePair| Some(value.height().clone()) ], reset_only: [] };
            MinSize, "min-size", [], "ext.property.min-size", CssSizePair, CssMinSizePropertyValue, CssMinSizePropertyValueRepresentation, parse_size_pair, { parse_size_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ MinWidth => |value: &CssSizePair| Some(value.width().clone()), MinHeight => |value: &CssSizePair| Some(value.height().clone()) ], reset_only: [] };
            MaxSize, "max-size", [], "ext.property.max-size", CssMaxSizePair, CssMaxSizePropertyValue, CssMaxSizePropertyValueRepresentation, parse_max_size_pair, { parse_max_size_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ MaxWidth => |value: &CssMaxSizePair| Some(value.width().clone()), MaxHeight => |value: &CssMaxSizePair| Some(value.height().clone()) ], reset_only: [] };
            FrameSizing, "frame-sizing", [], "ext.property.frame-sizing", CssFrameSizing, CssFrameSizingPropertyValue, CssFrameSizingPropertyValueRepresentation, parse_frame_sizing, { parse_frame_sizing($input)? }, expansion = longhand { wrapper: additive, value: CssFrameSizing, accessor: current, inherited: false, initial_kind: value, initial: CssFrameSizing::Auto };
            MinIntrinsicSizing, "min-intrinsic-sizing", [], "ext.property.min-intrinsic-sizing", CssMinIntrinsicSizing, CssMinIntrinsicSizingPropertyValue, CssMinIntrinsicSizingPropertyValueRepresentation, parse_min_intrinsic_sizing, { parse_min_intrinsic_sizing($input)? }, expansion = longhand { wrapper: additive, value: CssMinIntrinsicSizing, accessor: current, inherited: false, initial_kind: value, initial: CssMinIntrinsicSizing::Legacy };
            ContainIntrinsicWidth, "contain-intrinsic-width", [], "ext.property.contain-intrinsic-width", CssContainIntrinsicSizeValue, CssContainIntrinsicWidthPropertyValue, CssContainIntrinsicWidthPropertyValueRepresentation, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssContainIntrinsicSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None) };
            ContainIntrinsicHeight, "contain-intrinsic-height", [], "ext.property.contain-intrinsic-height", CssContainIntrinsicSizeValue, CssContainIntrinsicHeightPropertyValue, CssContainIntrinsicHeightPropertyValueRepresentation, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssContainIntrinsicSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None) };
            ContainIntrinsicInlineSize, "contain-intrinsic-inline-size", [], "ext.property.contain-intrinsic-inline-size", CssContainIntrinsicSizeValue, CssContainIntrinsicInlineSizePropertyValue, CssContainIntrinsicInlineSizePropertyValueRepresentation, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssContainIntrinsicSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None) };
            ContainIntrinsicBlockSize, "contain-intrinsic-block-size", [], "ext.property.contain-intrinsic-block-size", CssContainIntrinsicSizeValue, CssContainIntrinsicBlockSizePropertyValue, CssContainIntrinsicBlockSizePropertyValueRepresentation, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssContainIntrinsicSizeValue, accessor: current, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None) };
            ContainIntrinsicSize, "contain-intrinsic-size", [], "ext.property.contain-intrinsic-size", CssContainIntrinsicSize, CssContainIntrinsicSizePropertyValue, CssContainIntrinsicSizePropertyValueRepresentation, parse_contain_intrinsic_size, { parse_contain_intrinsic_size($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ ContainIntrinsicWidth => |value: &CssContainIntrinsicSize| Some(value.width().clone()), ContainIntrinsicHeight => |value: &CssContainIntrinsicSize| Some(value.height().clone()) ], reset_only: [] };
            FlexBasis, "flex-basis", ["-webkit-flex-basis"], "baseline.property.flex-basis", CssLength, CssFlexBasisPropertyValue, CssFlexBasisPropertyValueRepresentation, parse_flex_basis, { parse_flex_basis($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssFlexBasisValue, accessor: current, inherited: false, initial_kind: value, initial: CssFlexBasisValue::from(CssSizeValue::Auto) };
            Gap, "gap", ["grid-gap"], "baseline.property.gap", CssGapShorthand, CssGapPropertyValue, CssGapPropertyValueRepresentation, parse_gap_shorthand, { parse_gap_shorthand($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ RowGap => |value: &CssGapShorthand| Some(value.row().clone()), ColumnGap => |value: &CssGapShorthand| Some(value.column().clone()) ], reset_only: [] };
            RowGap, "row-gap", ["grid-row-gap"], "baseline.property.row-gap", CssGapValue, CssRowGapPropertyValue, CssRowGapPropertyValueRepresentation, parse_gap_value, { parse_gap_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssGapValue, accessor: current, inherited: false, initial_kind: value, initial: CssGapValue::Normal };
            ColumnGap, "column-gap", ["grid-column-gap"], "baseline.property.column-gap", CssGapValue, CssColumnGapPropertyValue, CssColumnGapPropertyValueRepresentation, parse_gap_value, { parse_gap_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssGapValue, accessor: current, inherited: false, initial_kind: value, initial: CssGapValue::Normal };
            ColumnCount, "column-count", [], "official.property.column-count", CssColumnCount, CssColumnCountPropertyValue, CssColumnCountPropertyValueRepresentation, parse_column_count, { parse_column_count($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssColumnCount, accessor: count, inherited: false, initial_kind: value, initial: CssColumnCount::Auto };
            ColumnFill, "column-fill", [], "official.property.column-fill", CssColumnFill, CssColumnFillPropertyValue, CssColumnFillPropertyValueRepresentation, parse_column_fill, { parse_column_fill($input)? }, expansion = longhand { wrapper: existing, value: CssColumnFill, accessor: fill, inherited: false, initial_kind: value, initial: CssColumnFill::Balance };
            ColumnRule, "column-rule", [], "official.property.column-rule", CssColumnRule, CssColumnRulePropertyValue, CssColumnRulePropertyValueRepresentation, parse_column_rule, { parse_column_rule($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: rule, members: [ ColumnRuleWidth => |value: &CssColumnRule| value.width().cloned(), ColumnRuleStyle => |value: &CssColumnRule| value.style(), ColumnRuleColor => |value: &CssColumnRule| value.color().cloned() ], reset_only: [] };
            ColumnRuleColor, "column-rule-color", [], "official.property.column-rule-color", CssParsedColor, CssColumnRuleColorPropertyValue, CssColumnRuleColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            ColumnRuleStyle, "column-rule-style", [], "official.property.column-rule-style", CssLineStyle, CssColumnRuleStylePropertyValue, CssColumnRuleStylePropertyValueRepresentation, parse_line_style, { parse_line_style($input)? }, expansion = longhand { wrapper: existing, value: CssLineStyle, accessor: style, inherited: false, initial_kind: value, initial: CssLineStyle::None };
            ColumnRuleWidth, "column-rule-width", [], "official.property.column-rule-width", CssLineWidth, CssColumnRuleWidthPropertyValue, CssColumnRuleWidthPropertyValueRepresentation, parse_line_width, { parse_line_width($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssLineWidth, accessor: width, inherited: false, initial_kind: value, initial: CssLineWidth::Medium };
            ColumnSpan, "column-span", [], "official.property.column-span", CssColumnSpan, CssColumnSpanPropertyValue, CssColumnSpanPropertyValueRepresentation, parse_column_span, { parse_column_span($input)? }, expansion = longhand { wrapper: existing, value: CssColumnSpan, accessor: span, inherited: false, initial_kind: value, initial: CssColumnSpan::None };
            ColumnWidth, "column-width", [], "official.property.column-width", CssColumnWidth, CssColumnWidthPropertyValue, CssColumnWidthPropertyValueRepresentation, parse_column_width, { parse_column_width($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssColumnWidth, accessor: width, inherited: false, initial_kind: value, initial: CssColumnWidth::Auto };
            Columns, "columns", [], "official.property.columns", CssColumns, CssColumnsPropertyValue, CssColumnsPropertyValueRepresentation, parse_columns, { parse_columns($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: columns, members: [ ColumnWidth => |value: &CssColumns| Some(value.width().clone()), ColumnCount => |value: &CssColumns| Some(value.count().clone()) ], reset_only: [] };
            FlowTolerance, "flow-tolerance", [], "ext.property.flow-tolerance", CssFlowTolerance, CssFlowTolerancePropertyValue, CssFlowTolerancePropertyValueRepresentation, parse_flow_tolerance, { parse_flow_tolerance($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssFlowTolerance, accessor: value, inherited: false, initial_kind: value, initial: CssFlowTolerance::normal() };
            GridTemplateRows, "grid-template-rows", [], "baseline.property.grid-template-rows", CssGridTrackList, CssGridTemplateRowsPropertyValue, CssGridTemplateRowsPropertyValueRepresentation, parse_grid_track_list, { parse_grid_track_list($input, $numeric)? };
            GridTemplateColumns, "grid-template-columns", [], "baseline.property.grid-template-columns", CssGridTrackList, CssGridTemplateColumnsPropertyValue, CssGridTemplateColumnsPropertyValueRepresentation, parse_grid_track_list, { parse_grid_track_list($input, $numeric)? };
            GridTemplateAreas, "grid-template-areas", [], "baseline.property.grid-template-areas", CssGridTemplateAreas, CssGridTemplateAreasPropertyValue, CssGridTemplateAreasPropertyValueRepresentation, parse_grid_template_areas, { parse_grid_template_areas($input)? };
            GridTemplate, "grid-template", [], "baseline.property.grid-template", CssGridTemplate, CssGridTemplatePropertyValue, CssGridTemplatePropertyValueRepresentation, parse_grid_template, { parse_grid_template($input, $numeric)? };
            GridAutoRows, "grid-auto-rows", [], "baseline.property.grid-auto-rows", CssGridTrackList, CssGridAutoRowsPropertyValue, CssGridAutoRowsPropertyValueRepresentation, parse_grid_auto_track_sizes, { parse_grid_auto_track_sizes($input, $numeric)? };
            GridAutoColumns, "grid-auto-columns", [], "baseline.property.grid-auto-columns", CssGridTrackList, CssGridAutoColumnsPropertyValue, CssGridAutoColumnsPropertyValueRepresentation, parse_grid_auto_track_sizes, { parse_grid_auto_track_sizes($input, $numeric)? };
            GridAutoFlow, "grid-auto-flow", [], "baseline.property.grid-auto-flow", CssGridAutoFlow, CssGridAutoFlowPropertyValue, CssGridAutoFlowPropertyValueRepresentation, parse_grid_auto_flow, { parse_grid_auto_flow($input)? };
            GridRowStart, "grid-row-start", [], "baseline.property.grid-row-start", CssAuthoredGridLine, CssGridRowStartPropertyValue, CssGridRowStartPropertyValueRepresentation, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredGridLine, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredGridLine::Auto };
            GridRowEnd, "grid-row-end", [], "baseline.property.grid-row-end", CssAuthoredGridLine, CssGridRowEndPropertyValue, CssGridRowEndPropertyValueRepresentation, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredGridLine, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredGridLine::Auto };
            GridColumnStart, "grid-column-start", [], "baseline.property.grid-column-start", CssAuthoredGridLine, CssGridColumnStartPropertyValue, CssGridColumnStartPropertyValueRepresentation, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredGridLine, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredGridLine::Auto };
            GridColumnEnd, "grid-column-end", [], "baseline.property.grid-column-end", CssAuthoredGridLine, CssGridColumnEndPropertyValue, CssGridColumnEndPropertyValueRepresentation, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredGridLine, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredGridLine::Auto };
            GridRow, "grid-row", [], "baseline.property.grid-row", CssAuthoredGridLineRange, CssGridRowPropertyValue, CssGridRowPropertyValueRepresentation, parse_grid_line_range, { parse_grid_line_range($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ GridRowStart => |value: &CssAuthoredGridLineRange| Some(value.start().clone()), GridRowEnd => |value: &CssAuthoredGridLineRange| Some(value.effective_end()) ], reset_only: [] };
            GridColumn, "grid-column", [], "baseline.property.grid-column", CssAuthoredGridLineRange, CssGridColumnPropertyValue, CssGridColumnPropertyValueRepresentation, parse_grid_line_range, { parse_grid_line_range($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ GridColumnStart => |value: &CssAuthoredGridLineRange| Some(value.start().clone()), GridColumnEnd => |value: &CssAuthoredGridLineRange| Some(value.effective_end()) ], reset_only: [] };
            GridArea, "grid-area", [], "baseline.property.grid-area", CssAuthoredGridArea, CssGridAreaPropertyValue, CssGridAreaPropertyValueRepresentation, parse_grid_area, { parse_grid_area($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ GridRowStart => |value: &CssAuthoredGridArea| Some(value.row_start().clone()), GridColumnStart => |value: &CssAuthoredGridArea| Some(value.effective_column_start()), GridRowEnd => |value: &CssAuthoredGridArea| Some(value.effective_row_end()), GridColumnEnd => |value: &CssAuthoredGridArea| Some(value.effective_column_end()) ], reset_only: [] };
            Grid, "grid", [], "baseline.property.grid", CssGrid, CssGridPropertyValue, CssGridPropertyValueRepresentation, parse_grid, { parse_grid($input, $numeric)? };
            FontSize, "font-size", [], "baseline.property.font-size", CssFontSize, CssFontSizePropertyValue, CssFontSizePropertyValueRepresentation, parse_font_size, { parse_font_size($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssFontSize, accessor: size, inherited: true, initial_kind: value, initial: CssFontSize::Medium };
            LineHeight, "line-height", [], "baseline.property.line-height", CssLineHeight, CssLineHeightPropertyValue, CssLineHeightPropertyValueRepresentation, parse_line_height, { parse_line_height($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssLineHeight, accessor: line_height, inherited: true, initial_kind: value, initial: CssLineHeight::Normal };
            TextCombineUpright, "text-combine-upright", [], "official.property.text-combine-upright", CssTextCombineUpright, CssTextCombineUprightPropertyValue, CssTextCombineUprightPropertyValueRepresentation, parse_text_combine_upright, { parse_text_combine_upright($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssTextCombineUpright, accessor: combine, inherited: true, initial_kind: value, initial: CssTextCombineUpright::None };
            TextOrientation, "text-orientation", [], "official.property.text-orientation", CssTextOrientation, CssTextOrientationPropertyValue, CssTextOrientationPropertyValueRepresentation, parse_text_orientation, { parse_text_orientation($input)? }, expansion = longhand { wrapper: existing, value: CssTextOrientation, accessor: orientation, inherited: true, initial_kind: value, initial: CssTextOrientation::Mixed };
            UnicodeBidi, "unicode-bidi", [], "official.property.unicode-bidi", CssUnicodeBidi, CssUnicodeBidiPropertyValue, CssUnicodeBidiPropertyValueRepresentation, parse_unicode_bidi, { parse_unicode_bidi($input)? }, expansion = longhand { wrapper: existing, value: CssUnicodeBidi, accessor: bidi, inherited: false, initial_kind: value, initial: CssUnicodeBidi::Normal };
            WritingMode, "writing-mode", [], "baseline.property.writing-mode", CssWritingMode, CssWritingModePropertyValue, CssWritingModePropertyValueRepresentation, parse_writing_mode, { parse_writing_mode($input)? }, expansion = longhand { wrapper: fallback, value: CssWritingMode, accessor: current, inherited: true, initial_kind: value, initial: CssWritingMode::HorizontalTb };
            TextAlign, "text-align", [], "baseline.property.text-align", CssTextAlignValue, CssTextAlignPropertyValue, CssTextAlignPropertyValueRepresentation, parse_text_align, { parse_text_align($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ TextAlignAll => |value: &CssTextAlignValue| Some(crate::text_alignment::shorthand_all(value)), TextAlignLast => |value: &CssTextAlignValue| Some(crate::text_alignment::shorthand_last(value)) ], reset_only: [] };
            TextAlignAll, "text-align-all", [], "official.property.text-align-all", CssTextAlignAllValue, CssTextAlignAllPropertyValue, CssTextAlignAllPropertyValueRepresentation, parse_text_align_all, { parse_text_align_all($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssTextAlignAllValue, accessor: current, inherited: true, initial_kind: value, initial: CssTextAlignAllValue::initial() };
            TextAlignLast, "text-align-last", [], "baseline.property.text-align-last", CssTextAlignLastValue, CssTextAlignLastPropertyValue, CssTextAlignLastPropertyValueRepresentation, parse_text_align_last, { parse_text_align_last($input)? }, expansion = longhand { wrapper: existing, value: CssTextAlignLastValue, accessor: current, inherited: true, initial_kind: value, initial: CssTextAlignLastValue::initial() };
            TextIndent, "text-indent", [], "baseline.property.text-indent", CssTextIndent, CssTextIndentPropertyValue, CssTextIndentPropertyValueRepresentation, parse_text_indent, { parse_text_indent($input, $numeric)? };
            VerticalAlign, "vertical-align", [], "baseline.property.vertical-align", CssVerticalAlign, CssVerticalAlignPropertyValue, CssVerticalAlignPropertyValueRepresentation, parse_vertical_align, { parse_vertical_align($input, $numeric)? };
            FontFamily, "font-family", [], "baseline.property.font-family", CssFontFamilyList, CssFontFamilyPropertyValue, CssFontFamilyPropertyValueRepresentation, parse_font_family_list, { parse_font_family_list($input)? }, expansion = longhand { wrapper: existing, value: CssFontFamilyList, accessor: families, inherited: true, initial_kind: user_agent, initial: CssUserAgentInitial::FontFamily };
            Font, "font", [], "baseline.property.font", CssFontValue, CssFontPropertyValue, CssFontPropertyValueRepresentation, parse_font, { parse_font($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: font, members: [ FontFamily => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => Some(font.families().clone()), CssFontValue::System(_) => None }, FontSize => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => Some(font.size().clone()), CssFontValue::System(_) => None }, FontWidth => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.stretch().map(CssFontWidth::Keyword), CssFontValue::System(_) => None }, FontStyle => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.style().cloned(), CssFontValue::System(_) => None }, FontVariantCaps => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.variant().map(|variant| match variant { CssFontVariant::Normal => CssFontVariantCaps::Normal, CssFontVariant::SmallCaps => CssFontVariantCaps::SmallCaps }), CssFontValue::System(_) => None }, FontWeight => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.weight().cloned(), CssFontValue::System(_) => None }, LineHeight => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.line_height().cloned(), CssFontValue::System(_) => None } ], reset_only: [FontFeatureSettings, FontKerning, FontLanguageOverride, FontOpticalSizing, FontSizeAdjust, FontVariantAlternates, FontVariantEastAsian, FontVariantEmoji, FontVariantLigatures, FontVariantNumeric, FontVariantPosition, FontVariationSettings] };
            FontWeight, "font-weight", [], "baseline.property.font-weight", CssFontWeight, CssFontWeightPropertyValue, CssFontWeightPropertyValueRepresentation, parse_font_weight, { parse_font_weight($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssFontWeight, accessor: current, inherited: true, initial_kind: value, initial: CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal) };
            FontStyle, "font-style", [], "baseline.property.font-style", CssFontStyle, CssFontStylePropertyValue, CssFontStylePropertyValueRepresentation, parse_font_style, { parse_font_style($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssFontStyle, accessor: current, inherited: true, initial_kind: value, initial: CssFontStyle::Keyword(CssFontStyleKeyword::Normal) };
            FontWidth, "font-width", ["font-stretch"], "baseline.property.font-stretch", CssFontWidth, CssFontWidthPropertyValue, CssFontWidthPropertyValueRepresentation, parse_font_width, { parse_font_width($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssFontWidth, accessor: current, inherited: true, initial_kind: value, initial: CssFontWidth::Keyword(CssFontWidthKeyword::Normal) };
            FontVariant, "font-variant", [], "baseline.property.font-variant", CssFontVariantValue, CssFontVariantPropertyValue, CssFontVariantPropertyValueRepresentation, parse_font_variant, { parse_font_variant($input)? }, expansion = shorthand { wrapper: existing, accessor: variant, members: [ FontVariantLigatures => |value: &CssFontVariantValue| Some(value.expanded_ligatures()), FontVariantCaps => |value: &CssFontVariantValue| Some(value.expanded_caps()), FontVariantAlternates => |value: &CssFontVariantValue| Some(value.expanded_alternates()), FontVariantNumeric => |value: &CssFontVariantValue| Some(value.expanded_numeric()), FontVariantEastAsian => |value: &CssFontVariantValue| Some(value.expanded_east_asian()), FontVariantPosition => |value: &CssFontVariantValue| Some(value.expanded_position()), FontVariantEmoji => |value: &CssFontVariantValue| Some(value.expanded_emoji()) ], reset_only: [] };
            FontVariantCaps, "font-variant-caps", [], "official.property.font-variant-caps", CssFontVariantCaps, CssFontVariantCapsPropertyValue, CssFontVariantCapsPropertyValueRepresentation, parse_font_variant_caps, { parse_font_variant_caps($input)? }, expansion = longhand { wrapper: existing, value: CssFontVariantCaps, accessor: caps, inherited: true, initial_kind: value, initial: CssFontVariantCaps::Normal };
            FontVariantEastAsian, "font-variant-east-asian", [], "official.property.font-variant-east-asian", CssFontVariantEastAsian, CssFontVariantEastAsianPropertyValue, CssFontVariantEastAsianPropertyValueRepresentation, parse_font_variant_east_asian, { parse_font_variant_east_asian($input)? }, expansion = longhand { wrapper: existing, value: CssFontVariantEastAsian, accessor: east_asian, inherited: true, initial_kind: value, initial: CssFontVariantEastAsian::Normal };
            FontVariantLigatures, "font-variant-ligatures", [], "official.property.font-variant-ligatures", CssFontVariantLigatures, CssFontVariantLigaturesPropertyValue, CssFontVariantLigaturesPropertyValueRepresentation, parse_font_variant_ligatures, { parse_font_variant_ligatures($input)? }, expansion = longhand { wrapper: existing, value: CssFontVariantLigatures, accessor: ligatures, inherited: true, initial_kind: value, initial: CssFontVariantLigatures::Normal };
            FontVariantNumeric, "font-variant-numeric", [], "official.property.font-variant-numeric", CssFontVariantNumeric, CssFontVariantNumericPropertyValue, CssFontVariantNumericPropertyValueRepresentation, parse_font_variant_numeric, { parse_font_variant_numeric($input)? }, expansion = longhand { wrapper: existing, value: CssFontVariantNumeric, accessor: numeric, inherited: true, initial_kind: value, initial: CssFontVariantNumeric::Normal };
            FontVariantPosition, "font-variant-position", [], "official.property.font-variant-position", CssFontVariantPosition, CssFontVariantPositionPropertyValue, CssFontVariantPositionPropertyValueRepresentation, parse_font_variant_position, { parse_font_variant_position($input)? }, expansion = longhand { wrapper: existing, value: CssFontVariantPosition, accessor: position, inherited: true, initial_kind: value, initial: CssFontVariantPosition::Normal };
            FontVariantAlternates, "font-variant-alternates", [], "official.property.font-variant-alternates", CssFontVariantAlternates, CssFontVariantAlternatesPropertyValue, CssFontVariantAlternatesPropertyValueRepresentation, parse_font_variant_alternates, { parse_font_variant_alternates($input)? }, expansion = longhand { wrapper: existing, value: CssFontVariantAlternates, accessor: alternates, inherited: true, initial_kind: value, initial: CssFontVariantAlternates::Normal };
            FontVariantEmoji, "font-variant-emoji", [], "official.property.font-variant-emoji", CssFontVariantEmoji, CssFontVariantEmojiPropertyValue, CssFontVariantEmojiPropertyValueRepresentation, parse_font_variant_emoji, { parse_font_variant_emoji($input)? }, expansion = longhand { wrapper: existing, value: CssFontVariantEmoji, accessor: emoji, inherited: true, initial_kind: value, initial: CssFontVariantEmoji::Normal };
            FontFeatureSettings, "font-feature-settings", [], "baseline.property.font-feature-settings", CssAuthoredFontFeatureSettings, CssFontFeatureSettingsPropertyValue, CssFontFeatureSettingsPropertyValueRepresentation, parse_font_feature_settings, { parse_font_feature_settings($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredFontFeatureSettings, accessor: settings, inherited: true, initial_kind: value, initial: CssAuthoredFontFeatureSettings::Normal };
            FontKerning, "font-kerning", [], "official.property.font-kerning", CssFontKerning, CssFontKerningPropertyValue, CssFontKerningPropertyValueRepresentation, parse_font_kerning, { parse_font_kerning($input)? }, expansion = longhand { wrapper: existing, value: CssFontKerning, accessor: kerning, inherited: true, initial_kind: value, initial: CssFontKerning::Auto };
            FontSizeAdjust, "font-size-adjust", [], "official.property.font-size-adjust", CssFontSizeAdjust, CssFontSizeAdjustPropertyValue, CssFontSizeAdjustPropertyValueRepresentation, parse_font_size_adjust, { parse_font_size_adjust($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssFontSizeAdjust, accessor: size_adjust, inherited: true, initial_kind: value, initial: CssFontSizeAdjust::None };
            FontLanguageOverride, "font-language-override", [], "official.property.font-language-override", CssFontLanguageOverride, CssFontLanguageOverridePropertyValue, CssFontLanguageOverridePropertyValueRepresentation, parse_font_language_override, { parse_font_language_override($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssFontLanguageOverride, accessor: language_override, inherited: true, initial_kind: value, initial: CssFontLanguageOverride::Normal };
            FontOpticalSizing, "font-optical-sizing", [], "official.property.font-optical-sizing", CssFontOpticalSizing, CssFontOpticalSizingPropertyValue, CssFontOpticalSizingPropertyValueRepresentation, parse_font_optical_sizing, { parse_font_optical_sizing($input)? }, expansion = longhand { wrapper: additive, value: CssFontOpticalSizing, accessor: optical_sizing, inherited: true, initial_kind: value, initial: CssFontOpticalSizing::Auto };
            FontVariationSettings, "font-variation-settings", [], "official.property.font-variation-settings", CssFontVariationSettings, CssFontVariationSettingsPropertyValue, CssFontVariationSettingsPropertyValueRepresentation, parse_font_variation_settings, { parse_font_variation_settings($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssFontVariationSettings, accessor: variations, inherited: true, initial_kind: value, initial: CssFontVariationSettings::Normal };
            FontSynthesis, "font-synthesis", [], "official.property.font-synthesis", CssFontSynthesis, CssFontSynthesisPropertyValue, CssFontSynthesisPropertyValueRepresentation, parse_font_synthesis, { parse_font_synthesis($input)? };
            LetterSpacing, "letter-spacing", [], "baseline.property.letter-spacing", CssLetterSpacing, CssLetterSpacingPropertyValue, CssLetterSpacingPropertyValueRepresentation, parse_letter_spacing, { parse_letter_spacing($input, $numeric)? }, expansion = longhand { wrapper: existing, value: crate::CssTextSpacingAdjustment, accessor: current, inherited: true, initial_kind: value, initial: crate::CssTextSpacingAdjustment::Normal };
            TextWrap, "text-wrap", [], "baseline.property.text-wrap", CssTextWrap, CssTextWrapPropertyValue, CssTextWrapPropertyValueRepresentation, parse_text_wrap, { parse_text_wrap($input)? };
            WhiteSpace, "white-space", [], "baseline.property.white-space", CssWhiteSpace, CssWhiteSpacePropertyValue, CssWhiteSpacePropertyValueRepresentation, parse_white_space, { parse_white_space($input)? };
            WordBreak, "word-break", [], "baseline.property.word-break", CssWordBreak, CssWordBreakPropertyValue, CssWordBreakPropertyValueRepresentation, parse_word_break, { parse_word_break($input)? };
            OverflowWrap, "overflow-wrap", ["word-wrap"], "baseline.property.overflow-wrap", CssOverflowWrap, CssOverflowWrapPropertyValue, CssOverflowWrapPropertyValueRepresentation, parse_overflow_wrap, { parse_overflow_wrap($input)? }, expansion = longhand { wrapper: fallback, value: CssOverflowWrap, accessor: current, inherited: true, initial_kind: value, initial: CssOverflowWrap::Normal };
            TextOverflow, "text-overflow", [], "baseline.property.text-overflow", CssTextOverflow, CssTextOverflowPropertyValue, CssTextOverflowPropertyValueRepresentation, parse_text_overflow, { parse_text_overflow($input)? }, expansion = longhand { wrapper: fallback, value: CssTextOverflow, accessor: current, inherited: false, initial_kind: value, initial: CssTextOverflow::Clip };
            TextDecoration, "text-decoration", [], "baseline.property.text-decoration", CssTextDecoration, CssTextDecorationPropertyValue, CssTextDecorationPropertyValueRepresentation, parse_text_decoration, { parse_text_decoration($input, $numeric)? };
            TextDecorationLine, "text-decoration-line", [], "baseline.property.text-decoration-line", CssTextDecorationLine, CssTextDecorationLinePropertyValue, CssTextDecorationLinePropertyValueRepresentation, parse_text_decoration_line, { parse_text_decoration_line($input)? };
            TextDecorationColor, "text-decoration-color", [], "baseline.property.text-decoration-color", CssColor, CssTextDecorationColorPropertyValue, CssTextDecorationColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? };
            TextDecorationStyle, "text-decoration-style", [], "baseline.property.text-decoration-style", CssTextDecorationStyle, CssTextDecorationStylePropertyValue, CssTextDecorationStylePropertyValueRepresentation, parse_text_decoration_style, { parse_text_decoration_style($input)? };
            TextDecorationThickness, "text-decoration-thickness", [], "baseline.property.text-decoration-thickness", CssTextDecorationThickness, CssTextDecorationThicknessPropertyValue, CssTextDecorationThicknessPropertyValueRepresentation, parse_text_decoration_thickness, { parse_text_decoration_thickness($input, $numeric)? };
            TextTransform, "text-transform", [], "baseline.property.text-transform", CssTextTransform, CssTextTransformPropertyValue, CssTextTransformPropertyValueRepresentation, parse_text_transform, { parse_text_transform($input)? };
            Inset, "inset", [], "baseline.property.inset", CssInsetShorthand, CssInsetPropertyValue, CssInsetPropertyValueRepresentation, parse_inset_shorthand, { parse_inset_shorthand($input, $numeric)? }, expansion = unresolved { wrapper: existing, reason: CssUnresolvedStandard::LogicalShorthandResetMembership };
            Top, "top", [], "baseline.property.top", CssInsetValue, CssTopPropertyValue, CssTopPropertyValueRepresentation, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssInsetValue, accessor: current, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            Right, "right", [], "baseline.property.right", CssInsetValue, CssRightPropertyValue, CssRightPropertyValueRepresentation, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssInsetValue, accessor: current, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            Bottom, "bottom", [], "baseline.property.bottom", CssInsetValue, CssBottomPropertyValue, CssBottomPropertyValueRepresentation, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssInsetValue, accessor: current, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            Left, "left", [], "baseline.property.left", CssInsetValue, CssLeftPropertyValue, CssLeftPropertyValueRepresentation, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssInsetValue, accessor: current, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetBlockStart, "inset-block-start", [], "official.property.inset-block-start", CssInsetValue, CssInsetBlockStartPropertyValue, CssInsetBlockStartPropertyValueRepresentation, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssInsetValue, accessor: current, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetBlockEnd, "inset-block-end", [], "official.property.inset-block-end", CssInsetValue, CssInsetBlockEndPropertyValue, CssInsetBlockEndPropertyValueRepresentation, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssInsetValue, accessor: current, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetInlineStart, "inset-inline-start", [], "official.property.inset-inline-start", CssInsetValue, CssInsetInlineStartPropertyValue, CssInsetInlineStartPropertyValueRepresentation, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssInsetValue, accessor: current, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetInlineEnd, "inset-inline-end", [], "official.property.inset-inline-end", CssInsetValue, CssInsetInlineEndPropertyValue, CssInsetInlineEndPropertyValueRepresentation, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssInsetValue, accessor: current, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetBlock, "inset-block", [], "official.property.inset-block", CssInsetPair, CssInsetBlockPropertyValue, CssInsetBlockPropertyValueRepresentation, parse_inset_pair, { parse_inset_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ InsetBlockStart => |value: &CssInsetPair| Some(value.start().clone()), InsetBlockEnd => |value: &CssInsetPair| Some(value.end().clone()) ], reset_only: [] };
            InsetInline, "inset-inline", [], "official.property.inset-inline", CssInsetPair, CssInsetInlinePropertyValue, CssInsetInlinePropertyValueRepresentation, parse_inset_pair, { parse_inset_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ InsetInlineStart => |value: &CssInsetPair| Some(value.start().clone()), InsetInlineEnd => |value: &CssInsetPair| Some(value.end().clone()) ], reset_only: [] };
            ZIndex, "z-index", [], "baseline.property.z-index", CssZIndex, CssZIndexPropertyValue, CssZIndexPropertyValueRepresentation, parse_z_index, { parse_z_index($input, $numeric)? };
            BoxDecorationBreak, "box-decoration-break", [], "baseline.property.box-decoration-break", CssBoxDecorationBreak, CssBoxDecorationBreakPropertyValue, CssBoxDecorationBreakPropertyValueRepresentation, parse_box_decoration_break, { parse_box_decoration_break($input)? };
            Margin, "margin", [], "baseline.property.margin", CssMarginShorthand, CssMarginPropertyValue, CssMarginPropertyValueRepresentation, parse_box_margin_shorthand, { parse_box_margin_shorthand($input, $numeric)? }, expansion = unresolved { wrapper: additive, reason: CssUnresolvedStandard::LogicalShorthandResetMembership };
            MarginTop, "margin-top", [], "baseline.property.margin-top", CssMarginValue, CssMarginTopPropertyValue, CssMarginTopPropertyValueRepresentation, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMarginValue, accessor: current, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginRight, "margin-right", [], "baseline.property.margin-right", CssMarginValue, CssMarginRightPropertyValue, CssMarginRightPropertyValueRepresentation, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMarginValue, accessor: current, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginBottom, "margin-bottom", [], "baseline.property.margin-bottom", CssMarginValue, CssMarginBottomPropertyValue, CssMarginBottomPropertyValueRepresentation, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMarginValue, accessor: current, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginLeft, "margin-left", [], "baseline.property.margin-left", CssMarginValue, CssMarginLeftPropertyValue, CssMarginLeftPropertyValueRepresentation, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMarginValue, accessor: current, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginBlockStart, "margin-block-start", [], "official.property.margin-block-start", CssMarginValue, CssMarginBlockStartPropertyValue, CssMarginBlockStartPropertyValueRepresentation, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMarginValue, accessor: current, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginBlockEnd, "margin-block-end", [], "official.property.margin-block-end", CssMarginValue, CssMarginBlockEndPropertyValue, CssMarginBlockEndPropertyValueRepresentation, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMarginValue, accessor: current, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginInlineStart, "margin-inline-start", [], "official.property.margin-inline-start", CssMarginValue, CssMarginInlineStartPropertyValue, CssMarginInlineStartPropertyValueRepresentation, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMarginValue, accessor: current, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginInlineEnd, "margin-inline-end", [], "official.property.margin-inline-end", CssMarginValue, CssMarginInlineEndPropertyValue, CssMarginInlineEndPropertyValueRepresentation, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssMarginValue, accessor: current, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginBlock, "margin-block", [], "official.property.margin-block", CssMarginPair, CssMarginBlockPropertyValue, CssMarginBlockPropertyValueRepresentation, parse_box_margin_pair, { parse_box_margin_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ MarginBlockStart => |value: &CssMarginPair| Some(value.start().clone()), MarginBlockEnd => |value: &CssMarginPair| Some(value.end().clone()) ], reset_only: [] };
            MarginInline, "margin-inline", [], "official.property.margin-inline", CssMarginPair, CssMarginInlinePropertyValue, CssMarginInlinePropertyValueRepresentation, parse_box_margin_pair, { parse_box_margin_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ MarginInlineStart => |value: &CssMarginPair| Some(value.start().clone()), MarginInlineEnd => |value: &CssMarginPair| Some(value.end().clone()) ], reset_only: [] };
            Padding, "padding", [], "baseline.property.padding", CssPaddingShorthand, CssPaddingPropertyValue, CssPaddingPropertyValueRepresentation, parse_box_padding_shorthand, { parse_box_padding_shorthand($input, $numeric)? }, expansion = unresolved { wrapper: additive, reason: CssUnresolvedStandard::LogicalShorthandResetMembership };
            PaddingTop, "padding-top", [], "baseline.property.padding-top", CssPaddingValue, CssPaddingTopPropertyValue, CssPaddingTopPropertyValueRepresentation, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingRight, "padding-right", [], "baseline.property.padding-right", CssPaddingValue, CssPaddingRightPropertyValue, CssPaddingRightPropertyValueRepresentation, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingBottom, "padding-bottom", [], "baseline.property.padding-bottom", CssPaddingValue, CssPaddingBottomPropertyValue, CssPaddingBottomPropertyValueRepresentation, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingLeft, "padding-left", [], "baseline.property.padding-left", CssPaddingValue, CssPaddingLeftPropertyValue, CssPaddingLeftPropertyValueRepresentation, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingBlockStart, "padding-block-start", [], "official.property.padding-block-start", CssPaddingValue, CssPaddingBlockStartPropertyValue, CssPaddingBlockStartPropertyValueRepresentation, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingBlockEnd, "padding-block-end", [], "official.property.padding-block-end", CssPaddingValue, CssPaddingBlockEndPropertyValue, CssPaddingBlockEndPropertyValueRepresentation, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingInlineStart, "padding-inline-start", [], "official.property.padding-inline-start", CssPaddingValue, CssPaddingInlineStartPropertyValue, CssPaddingInlineStartPropertyValueRepresentation, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingInlineEnd, "padding-inline-end", [], "official.property.padding-inline-end", CssPaddingValue, CssPaddingInlineEndPropertyValue, CssPaddingInlineEndPropertyValueRepresentation, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssPaddingValue, accessor: current, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingBlock, "padding-block", [], "official.property.padding-block", CssPaddingPair, CssPaddingBlockPropertyValue, CssPaddingBlockPropertyValueRepresentation, parse_box_padding_pair, { parse_box_padding_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ PaddingBlockStart => |value: &CssPaddingPair| Some(value.start().clone()), PaddingBlockEnd => |value: &CssPaddingPair| Some(value.end().clone()) ], reset_only: [] };
            PaddingInline, "padding-inline", [], "official.property.padding-inline", CssPaddingPair, CssPaddingInlinePropertyValue, CssPaddingInlinePropertyValueRepresentation, parse_box_padding_pair, { parse_box_padding_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ PaddingInlineStart => |value: &CssPaddingPair| Some(value.start().clone()), PaddingInlineEnd => |value: &CssPaddingPair| Some(value.end().clone()) ], reset_only: [] };
            Border, "border", [], "baseline.property.border", CssBorderValue, CssBorderPropertyValue, CssBorderPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ BorderTopWidth => |value: &CssBorderValue| value.width().cloned(), BorderRightWidth => |value: &CssBorderValue| value.width().cloned(), BorderBottomWidth => |value: &CssBorderValue| value.width().cloned(), BorderLeftWidth => |value: &CssBorderValue| value.width().cloned(), BorderTopStyle => |value: &CssBorderValue| value.style(), BorderRightStyle => |value: &CssBorderValue| value.style(), BorderBottomStyle => |value: &CssBorderValue| value.style(), BorderLeftStyle => |value: &CssBorderValue| value.style(), BorderTopColor => |value: &CssBorderValue| value.color().cloned(), BorderRightColor => |value: &CssBorderValue| value.color().cloned(), BorderBottomColor => |value: &CssBorderValue| value.color().cloned(), BorderLeftColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [ BorderImageSource, BorderImageSlice, BorderImageWidth, BorderImageOutset, BorderImageRepeat ] };
            BorderTop, "border-top", [], "baseline.property.border-top", CssBorderValue, CssBorderTopPropertyValue, CssBorderTopPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ BorderTopWidth => |value: &CssBorderValue| value.width().cloned(), BorderTopStyle => |value: &CssBorderValue| value.style(), BorderTopColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [  ] };
            BorderRight, "border-right", [], "baseline.property.border-right", CssBorderValue, CssBorderRightPropertyValue, CssBorderRightPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ BorderRightWidth => |value: &CssBorderValue| value.width().cloned(), BorderRightStyle => |value: &CssBorderValue| value.style(), BorderRightColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [  ] };
            BorderBottom, "border-bottom", [], "baseline.property.border-bottom", CssBorderValue, CssBorderBottomPropertyValue, CssBorderBottomPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ BorderBottomWidth => |value: &CssBorderValue| value.width().cloned(), BorderBottomStyle => |value: &CssBorderValue| value.style(), BorderBottomColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [  ] };
            BorderLeft, "border-left", [], "baseline.property.border-left", CssBorderValue, CssBorderLeftPropertyValue, CssBorderLeftPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ BorderLeftWidth => |value: &CssBorderValue| value.width().cloned(), BorderLeftStyle => |value: &CssBorderValue| value.style(), BorderLeftColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [  ] };
            BorderBlockStart, "border-block-start", [], "official.property.border-block-start", CssBorderValue, CssBorderBlockStartPropertyValue, CssBorderBlockStartPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)?.into_parts().0 }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderBlockStartWidth => |value: &CssBorderValue| value.width().cloned(), BorderBlockStartStyle => |value: &CssBorderValue| value.style(), BorderBlockStartColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [] };
            BorderBlockEnd, "border-block-end", [], "official.property.border-block-end", CssBorderValue, CssBorderBlockEndPropertyValue, CssBorderBlockEndPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)?.into_parts().0 }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderBlockEndWidth => |value: &CssBorderValue| value.width().cloned(), BorderBlockEndStyle => |value: &CssBorderValue| value.style(), BorderBlockEndColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [] };
            BorderInlineStart, "border-inline-start", [], "official.property.border-inline-start", CssBorderValue, CssBorderInlineStartPropertyValue, CssBorderInlineStartPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)?.into_parts().0 }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderInlineStartWidth => |value: &CssBorderValue| value.width().cloned(), BorderInlineStartStyle => |value: &CssBorderValue| value.style(), BorderInlineStartColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [] };
            BorderInlineEnd, "border-inline-end", [], "official.property.border-inline-end", CssBorderValue, CssBorderInlineEndPropertyValue, CssBorderInlineEndPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)?.into_parts().0 }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderInlineEndWidth => |value: &CssBorderValue| value.width().cloned(), BorderInlineEndStyle => |value: &CssBorderValue| value.style(), BorderInlineEndColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [] };
            BorderBlock, "border-block", [], "official.property.border-block", CssBorderValue, CssBorderBlockPropertyValue, CssBorderBlockPropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)?.into_parts().0 }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderBlockStartWidth => |value: &CssBorderValue| value.width().cloned(), BorderBlockEndWidth => |value: &CssBorderValue| value.width().cloned(), BorderBlockStartStyle => |value: &CssBorderValue| value.style(), BorderBlockEndStyle => |value: &CssBorderValue| value.style(), BorderBlockStartColor => |value: &CssBorderValue| value.color().cloned(), BorderBlockEndColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [] };
            BorderInline, "border-inline", [], "official.property.border-inline", CssBorderValue, CssBorderInlinePropertyValue, CssBorderInlinePropertyValueRepresentation, parse_exact_border, { parse_exact_border($input, $numeric)?.into_parts().0 }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderInlineStartWidth => |value: &CssBorderValue| value.width().cloned(), BorderInlineEndWidth => |value: &CssBorderValue| value.width().cloned(), BorderInlineStartStyle => |value: &CssBorderValue| value.style(), BorderInlineEndStyle => |value: &CssBorderValue| value.style(), BorderInlineStartColor => |value: &CssBorderValue| value.color().cloned(), BorderInlineEndColor => |value: &CssBorderValue| value.color().cloned() ], reset_only: [] };
            BorderWidth, "border-width", [], "baseline.property.border-width", CssBorderWidthShorthand, CssBorderWidthPropertyValue, CssBorderWidthPropertyValueRepresentation, parse_exact_border_width_shorthand, { parse_exact_border_width_shorthand($input, $numeric)? }, expansion = unresolved { wrapper: existing, reason: CssUnresolvedStandard::LogicalShorthandResetMembership };
            BorderTopWidth, "border-top-width", [], "baseline.property.border-top-width", CssBorderWidth, CssBorderTopWidthPropertyValue, CssBorderTopWidthPropertyValueRepresentation, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssBorderWidth, accessor: current, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderRightWidth, "border-right-width", [], "baseline.property.border-right-width", CssBorderWidth, CssBorderRightWidthPropertyValue, CssBorderRightWidthPropertyValueRepresentation, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssBorderWidth, accessor: current, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderBottomWidth, "border-bottom-width", [], "baseline.property.border-bottom-width", CssBorderWidth, CssBorderBottomWidthPropertyValue, CssBorderBottomWidthPropertyValueRepresentation, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssBorderWidth, accessor: current, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderLeftWidth, "border-left-width", [], "baseline.property.border-left-width", CssBorderWidth, CssBorderLeftWidthPropertyValue, CssBorderLeftWidthPropertyValueRepresentation, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssBorderWidth, accessor: current, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderBlockStartWidth, "border-block-start-width", [], "official.property.border-block-start-width", CssBorderWidth, CssBorderBlockStartWidthPropertyValue, CssBorderBlockStartWidthPropertyValueRepresentation, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssBorderWidth, accessor: current, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderBlockEndWidth, "border-block-end-width", [], "official.property.border-block-end-width", CssBorderWidth, CssBorderBlockEndWidthPropertyValue, CssBorderBlockEndWidthPropertyValueRepresentation, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssBorderWidth, accessor: current, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderInlineStartWidth, "border-inline-start-width", [], "official.property.border-inline-start-width", CssBorderWidth, CssBorderInlineStartWidthPropertyValue, CssBorderInlineStartWidthPropertyValueRepresentation, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssBorderWidth, accessor: current, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderInlineEndWidth, "border-inline-end-width", [], "official.property.border-inline-end-width", CssBorderWidth, CssBorderInlineEndWidthPropertyValue, CssBorderInlineEndWidthPropertyValueRepresentation, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssBorderWidth, accessor: current, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderBlockWidth, "border-block-width", [], "official.property.border-block-width", CssBorderWidthPair, CssBorderBlockWidthPropertyValue, CssBorderBlockWidthPropertyValueRepresentation, parse_exact_border_width_pair, { parse_exact_border_width_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderBlockStartWidth => |value: &CssBorderWidthPair| Some(value.start().clone()), BorderBlockEndWidth => |value: &CssBorderWidthPair| Some(value.end().clone()) ], reset_only: [] };
            BorderInlineWidth, "border-inline-width", [], "official.property.border-inline-width", CssBorderWidthPair, CssBorderInlineWidthPropertyValue, CssBorderInlineWidthPropertyValueRepresentation, parse_exact_border_width_pair, { parse_exact_border_width_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderInlineStartWidth => |value: &CssBorderWidthPair| Some(value.start().clone()), BorderInlineEndWidth => |value: &CssBorderWidthPair| Some(value.end().clone()) ], reset_only: [] };
            Color, "color", [], "baseline.property.color", CssColor, CssColorPropertyValue, CssColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredColor, accessor: current, inherited: true, initial_kind: value, initial: CssAuthoredColor::from_system(CssAuthoredSystemColor::CanvasText) };
            Background, "background", [], "baseline.property.background", CssColor, CssBackgroundPropertyValue, CssBackgroundPropertyValueRepresentation, parse_background, { parse_background($input, $numeric)? };
            BackgroundColor, "background-color", [], "baseline.property.background-color", CssColor, CssBackgroundColorPropertyValue, CssBackgroundColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? };
            BorderColor, "border-color", [], "baseline.property.border-color", CssBorderColorShorthand, CssBorderColorPropertyValue, CssBorderColorPropertyValueRepresentation, parse_border_colors, { parse_border_colors($input, $numeric)? }, expansion = unresolved { wrapper: existing, reason: CssUnresolvedStandard::LogicalShorthandResetMembership };
            BorderTopColor, "border-top-color", [], "baseline.property.border-top-color", CssColor, CssBorderTopColorPropertyValue, CssBorderTopColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            BorderRightColor, "border-right-color", [], "baseline.property.border-right-color", CssColor, CssBorderRightColorPropertyValue, CssBorderRightColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            BorderBottomColor, "border-bottom-color", [], "baseline.property.border-bottom-color", CssColor, CssBorderBottomColorPropertyValue, CssBorderBottomColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            BorderLeftColor, "border-left-color", [], "baseline.property.border-left-color", CssColor, CssBorderLeftColorPropertyValue, CssBorderLeftColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            BorderBlockStartColor, "border-block-start-color", [], "official.property.border-block-start-color", CssAuthoredColor, CssBorderBlockStartColorPropertyValue, CssBorderBlockStartColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            BorderBlockEndColor, "border-block-end-color", [], "official.property.border-block-end-color", CssAuthoredColor, CssBorderBlockEndColorPropertyValue, CssBorderBlockEndColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            BorderInlineStartColor, "border-inline-start-color", [], "official.property.border-inline-start-color", CssAuthoredColor, CssBorderInlineStartColorPropertyValue, CssBorderInlineStartColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            BorderInlineEndColor, "border-inline-end-color", [], "official.property.border-inline-end-color", CssAuthoredColor, CssBorderInlineEndColorPropertyValue, CssBorderInlineEndColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssAuthoredColor, accessor: current, inherited: false, initial_kind: value, initial: CssAuthoredColor::current_color() };
            BorderBlockColor, "border-block-color", [], "official.property.border-block-color", CssBorderColorPair, CssBorderBlockColorPropertyValue, CssBorderBlockColorPropertyValueRepresentation, parse_border_color_pair, { parse_border_color_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderBlockStartColor => |value: &CssBorderColorPair| Some(value.start().clone()), BorderBlockEndColor => |value: &CssBorderColorPair| Some(value.end().clone()) ], reset_only: [] };
            BorderInlineColor, "border-inline-color", [], "official.property.border-inline-color", CssBorderColorPair, CssBorderInlineColorPropertyValue, CssBorderInlineColorPropertyValueRepresentation, parse_border_color_pair, { parse_border_color_pair($input, $numeric)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderInlineStartColor => |value: &CssBorderColorPair| Some(value.start().clone()), BorderInlineEndColor => |value: &CssBorderColorPair| Some(value.end().clone()) ], reset_only: [] };
            BackgroundImage, "background-image", [], "baseline.property.background-image", CssImageLayerList, CssBackgroundImagePropertyValue, CssBackgroundImagePropertyValueRepresentation, parse_image_layer_list, { parse_image_layer_list($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssImageValueList, accessor: images, inherited: false, initial_kind: value, initial: CssImageValueList::try_new(vec![CssImageValue::None]).expect("one initial image") };
            BackgroundPosition, "background-position", [], "baseline.property.background-position", CssBackgroundPositionList, CssBackgroundPositionPropertyValue, CssBackgroundPositionPropertyValueRepresentation, parse_background_position_list, { parse_background_position_list($input, $numeric)? };
            ObjectPosition, "object-position", [], "official.property.object-position", CssObjectPosition, CssObjectPositionPropertyValue, CssObjectPositionPropertyValueRepresentation, parse_object_position, { parse_object_position($input, $numeric)? };
            BackgroundSize, "background-size", [], "baseline.property.background-size", CssBackgroundSizeList, CssBackgroundSizePropertyValue, CssBackgroundSizePropertyValueRepresentation, parse_background_size_list, { parse_background_size_list($input, $numeric)? };
            BackgroundRepeat, "background-repeat", [], "baseline.property.background-repeat", CssBackgroundRepeatList, CssBackgroundRepeatPropertyValue, CssBackgroundRepeatPropertyValueRepresentation, parse_background_repeat_list, { parse_background_repeat_list($input)? };
            BackgroundOrigin, "background-origin", [], "baseline.property.background-origin", CssBackgroundBox, CssBackgroundOriginPropertyValue, CssBackgroundOriginPropertyValueRepresentation, parse_background_box_list, { parse_background_box_list($input)? };
            BackgroundClip, "background-clip", [], "baseline.property.background-clip", CssBackgroundBox, CssBackgroundClipPropertyValue, CssBackgroundClipPropertyValueRepresentation, parse_background_box_list, { parse_background_box_list($input)? };
            BackgroundAttachment, "background-attachment", [], "baseline.property.background-attachment", CssBackgroundAttachmentList, CssBackgroundAttachmentPropertyValue, CssBackgroundAttachmentPropertyValueRepresentation, parse_background_attachment_list, { parse_background_attachment_list($input)? };
            BorderImage, "border-image", [], "official.property.border-image", CssBorderImage, CssBorderImagePropertyValue, CssBorderImagePropertyValueRepresentation, parse_border_image, { parse_border_image($input, $numeric)? };
            BorderImageOutset, "border-image-outset", [], "official.property.border-image-outset", CssBorderImageOutset, CssBorderImageOutsetPropertyValue, CssBorderImageOutsetPropertyValueRepresentation, parse_border_image_outset, { parse_border_image_outset($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssBorderImageOutset, accessor: outsets, inherited: false, initial_kind: value, initial: CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Number(CssNonNegativeNumber::try_new(0.0).expect("0 is non-negative"))]).expect("one outset component is valid") };
            BorderImageRepeat, "border-image-repeat", [], "official.property.border-image-repeat", CssBorderImageRepeat, CssBorderImageRepeatPropertyValue, CssBorderImageRepeatPropertyValueRepresentation, parse_border_image_repeat, { parse_border_image_repeat($input)? }, expansion = longhand { wrapper: existing, value: CssBorderImageRepeat, accessor: repeat, inherited: false, initial_kind: value, initial: CssBorderImageRepeat::new(CssBorderImageRepeatKeyword::Stretch, CssBorderImageRepeatKeyword::Stretch) };
            BorderImageSlice, "border-image-slice", [], "official.property.border-image-slice", CssBorderImageSlice, CssBorderImageSlicePropertyValue, CssBorderImageSlicePropertyValueRepresentation, parse_border_image_slice, { parse_border_image_slice($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssBorderImageSlice, accessor: slice, inherited: false, initial_kind: value, initial: CssBorderImageSlice::try_new(vec![CssBorderImageSliceComponent::Percentage(CssNonNegativeNumber::try_new(100.0).expect("100 is non-negative"))], false).expect("one slice component is valid") };
            BorderImageSource, "border-image-source", [], "official.property.border-image-source", CssImageValue, CssBorderImageSourcePropertyValue, CssBorderImageSourcePropertyValueRepresentation, parse_border_image_source, { parse_border_image_source($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssImageValue, accessor: source, inherited: false, initial_kind: value, initial: CssImageValue::None };
            BorderImageWidth, "border-image-width", [], "official.property.border-image-width", CssBorderImageWidth, CssBorderImageWidthPropertyValue, CssBorderImageWidthPropertyValueRepresentation, parse_border_image_width, { parse_border_image_width($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssBorderImageWidth, accessor: widths, inherited: false, initial_kind: value, initial: CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Number(CssNonNegativeNumber::try_new(1.0).expect("1 is non-negative"))]).expect("one width component is valid") };
            ImageOrientation, "image-orientation", [], "official.property.image-orientation", CssImageOrientation, CssImageOrientationPropertyValue, CssImageOrientationPropertyValueRepresentation, parse_image_orientation, { parse_image_orientation($input, $numeric)? };
            ImageRendering, "image-rendering", [], "official.property.image-rendering", CssImageRendering, CssImageRenderingPropertyValue, CssImageRenderingPropertyValueRepresentation, parse_image_rendering, { parse_image_rendering($input)? };
            ObjectFit, "object-fit", [], "official.property.object-fit", CssObjectFit, CssObjectFitPropertyValue, CssObjectFitPropertyValueRepresentation, parse_object_fit, { parse_object_fit($input)? };
            BorderStyle, "border-style", [], "baseline.property.border-style", CssBorderStyleShorthand, CssBorderStylePropertyValue, CssBorderStylePropertyValueRepresentation, parse_border_style_shorthand, { parse_border_style_shorthand($input)? }, expansion = unresolved { wrapper: existing, reason: CssUnresolvedStandard::LogicalShorthandResetMembership };
            BorderTopStyle, "border-top-style", [], "baseline.property.border-top-style", CssBorderStyle, CssBorderTopStylePropertyValue, CssBorderTopStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? }, expansion = longhand { wrapper: fallback, value: CssBorderStyle, accessor: current, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderRightStyle, "border-right-style", [], "baseline.property.border-right-style", CssBorderStyle, CssBorderRightStylePropertyValue, CssBorderRightStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? }, expansion = longhand { wrapper: fallback, value: CssBorderStyle, accessor: current, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderBottomStyle, "border-bottom-style", [], "baseline.property.border-bottom-style", CssBorderStyle, CssBorderBottomStylePropertyValue, CssBorderBottomStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? }, expansion = longhand { wrapper: fallback, value: CssBorderStyle, accessor: current, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderLeftStyle, "border-left-style", [], "baseline.property.border-left-style", CssBorderStyle, CssBorderLeftStylePropertyValue, CssBorderLeftStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? }, expansion = longhand { wrapper: fallback, value: CssBorderStyle, accessor: current, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderBlockStartStyle, "border-block-start-style", [], "official.property.border-block-start-style", CssBorderStyle, CssBorderBlockStartStylePropertyValue, CssBorderBlockStartStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? }, expansion = longhand { wrapper: additive, value: CssBorderStyle, accessor: current, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderBlockEndStyle, "border-block-end-style", [], "official.property.border-block-end-style", CssBorderStyle, CssBorderBlockEndStylePropertyValue, CssBorderBlockEndStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? }, expansion = longhand { wrapper: additive, value: CssBorderStyle, accessor: current, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderInlineStartStyle, "border-inline-start-style", [], "official.property.border-inline-start-style", CssBorderStyle, CssBorderInlineStartStylePropertyValue, CssBorderInlineStartStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? }, expansion = longhand { wrapper: additive, value: CssBorderStyle, accessor: current, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderInlineEndStyle, "border-inline-end-style", [], "official.property.border-inline-end-style", CssBorderStyle, CssBorderInlineEndStylePropertyValue, CssBorderInlineEndStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? }, expansion = longhand { wrapper: additive, value: CssBorderStyle, accessor: current, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderBlockStyle, "border-block-style", [], "official.property.border-block-style", CssBorderStylePair, CssBorderBlockStylePropertyValue, CssBorderBlockStylePropertyValueRepresentation, parse_border_style_pair, { parse_border_style_pair($input)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderBlockStartStyle => |value: &CssBorderStylePair| Some(*value.start()), BorderBlockEndStyle => |value: &CssBorderStylePair| Some(*value.end()) ], reset_only: [] };
            BorderInlineStyle, "border-inline-style", [], "official.property.border-inline-style", CssBorderStylePair, CssBorderInlineStylePropertyValue, CssBorderInlineStylePropertyValueRepresentation, parse_border_style_pair, { parse_border_style_pair($input)? }, expansion = shorthand { wrapper: additive, accessor: current, members: [ BorderInlineStartStyle => |value: &CssBorderStylePair| Some(*value.start()), BorderInlineEndStyle => |value: &CssBorderStylePair| Some(*value.end()) ], reset_only: [] };
            BorderRadius, "border-radius", [], "baseline.property.border-radius", CssBorderRadiusShorthand, CssBorderRadiusPropertyValue, CssBorderRadiusPropertyValueRepresentation, parse_exact_border_radius, { parse_exact_border_radius($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: current, members: [ BorderTopLeftRadius => |value: &CssBorderRadiusShorthand| Some(value.top_left()), BorderTopRightRadius => |value: &CssBorderRadiusShorthand| Some(value.top_right()), BorderBottomRightRadius => |value: &CssBorderRadiusShorthand| Some(value.bottom_right()), BorderBottomLeftRadius => |value: &CssBorderRadiusShorthand| Some(value.bottom_left()) ], reset_only: [] };
            BorderTopLeftRadius, "border-top-left-radius", [], "baseline.property.border-top-left-radius", CssCornerRadiusValue, CssBorderTopLeftRadiusPropertyValue, CssBorderTopLeftRadiusPropertyValueRepresentation, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssCornerRadiusValue, accessor: current, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderTopRightRadius, "border-top-right-radius", [], "baseline.property.border-top-right-radius", CssCornerRadiusValue, CssBorderTopRightRadiusPropertyValue, CssBorderTopRightRadiusPropertyValueRepresentation, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssCornerRadiusValue, accessor: current, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderBottomRightRadius, "border-bottom-right-radius", [], "baseline.property.border-bottom-right-radius", CssCornerRadiusValue, CssBorderBottomRightRadiusPropertyValue, CssBorderBottomRightRadiusPropertyValueRepresentation, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssCornerRadiusValue, accessor: current, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderBottomLeftRadius, "border-bottom-left-radius", [], "baseline.property.border-bottom-left-radius", CssCornerRadiusValue, CssBorderBottomLeftRadiusPropertyValue, CssBorderBottomLeftRadiusPropertyValueRepresentation, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssCornerRadiusValue, accessor: current, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderStartStartRadius, "border-start-start-radius", [], "official.property.border-start-start-radius", CssCornerRadiusValue, CssBorderStartStartRadiusPropertyValue, CssBorderStartStartRadiusPropertyValueRepresentation, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssCornerRadiusValue, accessor: current, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderStartEndRadius, "border-start-end-radius", [], "official.property.border-start-end-radius", CssCornerRadiusValue, CssBorderStartEndRadiusPropertyValue, CssBorderStartEndRadiusPropertyValueRepresentation, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssCornerRadiusValue, accessor: current, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderEndStartRadius, "border-end-start-radius", [], "official.property.border-end-start-radius", CssCornerRadiusValue, CssBorderEndStartRadiusPropertyValue, CssBorderEndStartRadiusPropertyValueRepresentation, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssCornerRadiusValue, accessor: current, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderEndEndRadius, "border-end-end-radius", [], "official.property.border-end-end-radius", CssCornerRadiusValue, CssBorderEndEndRadiusPropertyValue, CssBorderEndEndRadiusPropertyValueRepresentation, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { wrapper: additive, value: CssCornerRadiusValue, accessor: current, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BoxShadow, "box-shadow", [], "baseline.property.box-shadow", CssBoxShadow, CssBoxShadowPropertyValue, CssBoxShadowPropertyValueRepresentation, parse_box_shadow, { parse_box_shadow($input, $numeric)? };
            Opacity, "opacity", [], "baseline.property.opacity", CssOpacity, CssOpacityPropertyValue, CssOpacityPropertyValueRepresentation, parse_opacity, { parse_opacity($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssOpacityValue, accessor: value, inherited: false, initial_kind: value, initial: CssOpacityValue::Literal(CssOpacity::try_new(1.0).expect("one is a valid opacity")) };
            FlexGrow, "flex-grow", ["-webkit-flex-grow"], "baseline.property.flex-grow", CssFlexFactor, CssFlexGrowPropertyValue, CssFlexGrowPropertyValueRepresentation, parse_flex_factor, { parse_flex_factor($input, $numeric, "flex-grow")? }, expansion = longhand { wrapper: existing, value: CssSpecifiedNonNegativeNumber, accessor: factor, inherited: false, initial_kind: value, initial: crate::flex::initial_grow() };
            FlexShrink, "flex-shrink", ["-webkit-flex-shrink"], "baseline.property.flex-shrink", CssFlexFactor, CssFlexShrinkPropertyValue, CssFlexShrinkPropertyValueRepresentation, parse_flex_factor, { parse_flex_factor($input, $numeric, "flex-shrink")? }, expansion = longhand { wrapper: existing, value: CssSpecifiedNonNegativeNumber, accessor: factor, inherited: false, initial_kind: value, initial: crate::flex::initial_shrink() };
            Order, "order", ["-webkit-order"], "baseline.property.order", CssOrder, CssOrderPropertyValue, CssOrderPropertyValueRepresentation, parse_order, { parse_order($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssIntegerValue, accessor: value, inherited: false, initial_kind: value, initial: CssIntegerValue::Literal(0) };
            Flex, "flex", ["-webkit-flex"], "baseline.property.flex", CssFlex, CssFlexPropertyValue, CssFlexPropertyValueRepresentation, parse_flex, { parse_flex($input, $numeric)? }, expansion = shorthand { wrapper: existing, accessor: value, members: [ FlexGrow => crate::flex::effective_grow, FlexShrink => crate::flex::effective_shrink, FlexBasis => crate::flex::effective_basis ], reset_only: [] };
            JustifyTracks, "justify-tracks", [], "baseline.property.justify-tracks", CssAlignment, CssJustifyTracksPropertyValue, CssJustifyTracksPropertyValueRepresentation, parse_content_alignment, { parse_content_alignment($input)? };
            AlignTracks, "align-tracks", [], "baseline.property.align-tracks", CssAlignment, CssAlignTracksPropertyValue, CssAlignTracksPropertyValueRepresentation, parse_content_alignment, { parse_content_alignment($input)? };
            AspectRatio, "aspect-ratio", [], "baseline.property.aspect-ratio", CssAspectRatio, CssAspectRatioPropertyValue, CssAspectRatioPropertyValueRepresentation, parse_aspect_ratio, { parse_aspect_ratio($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssAspectRatioValue, accessor: ratio, inherited: false, initial_kind: value, initial: CssAspectRatioValue::Auto };
            ScrollbarWidth, "scrollbar-width", [], "baseline.property.scrollbar-width", CssScrollbarWidth, CssScrollbarWidthPropertyValue, CssScrollbarWidthPropertyValueRepresentation, parse_scrollbar_width, { parse_scrollbar_width($input)? };
            Cursor, "cursor", [], "baseline.property.cursor", CssCursor, CssCursorPropertyValue, CssCursorPropertyValueRepresentation, parse_cursor, { parse_cursor($input, $numeric)? };
            CaretColor, "caret-color", [], "official.property.caret-color", CssCaretColor, CssCaretColorPropertyValue, CssCaretColorPropertyValueRepresentation, parse_caret_color, { parse_caret_color($input, $numeric)? };
            PointerEvents, "pointer-events", [], "baseline.property.pointer-events", CssPointerEvents, CssPointerEventsPropertyValue, CssPointerEventsPropertyValueRepresentation, parse_pointer_events, { parse_pointer_events($input)? };
            UserSelect, "user-select", [], "baseline.property.user-select", CssUserSelect, CssUserSelectPropertyValue, CssUserSelectPropertyValueRepresentation, parse_user_select, { parse_user_select($input)? };
            Resize, "resize", [], "official.property.resize", CssResize, CssResizePropertyValue, CssResizePropertyValueRepresentation, parse_resize, { parse_resize($input)? };
            Contain, "contain", [], "official.property.contain", CssContain, CssContainPropertyValue, CssContainPropertyValueRepresentation, parse_contain, { parse_contain($input)? };
            Outline, "outline", [], "baseline.property.outline", CssOutline, CssOutlinePropertyValue, CssOutlinePropertyValueRepresentation, parse_outline, { parse_outline($input, $numeric)? };
            OutlineColor, "outline-color", [], "baseline.property.outline-color", CssColor, CssOutlineColorPropertyValue, CssOutlineColorPropertyValueRepresentation, parse_color, { parse_color($input, $numeric)? };
            OutlineOffset, "outline-offset", [], "official.property.outline-offset", CssOutlineOffset, CssOutlineOffsetPropertyValue, CssOutlineOffsetPropertyValueRepresentation, parse_outline_offset, { parse_outline_offset($input, $numeric)? };
            OutlineStyle, "outline-style", [], "baseline.property.outline-style", CssOutlineStyle, CssOutlineStylePropertyValue, CssOutlineStylePropertyValueRepresentation, parse_outline_style, { parse_outline_style($input)? };
            OutlineWidth, "outline-width", [], "baseline.property.outline-width", CssOutlineWidth, CssOutlineWidthPropertyValue, CssOutlineWidthPropertyValueRepresentation, parse_outline_width, { parse_outline_width($input, $numeric)? };
            Transform, "transform", [], "baseline.property.transform", CssTransform, CssTransformPropertyValue, CssTransformPropertyValueRepresentation, parse_transform, { parse_transform($input, $numeric)? };
            TransformBox, "transform-box", [], "official.property.transform-box", CssTransformBox, CssTransformBoxPropertyValue, CssTransformBoxPropertyValueRepresentation, parse_transform_box, { parse_transform_box($input)? };
            TransformOrigin, "transform-origin", [], "baseline.property.transform-origin", CssTransformOrigin, CssTransformOriginPropertyValue, CssTransformOriginPropertyValueRepresentation, parse_transform_origin, { parse_transform_origin($input, $numeric)? };
            Translate, "translate", [], "baseline.property.translate", CssTranslate, CssTranslatePropertyValue, CssTranslatePropertyValueRepresentation, parse_translate, { parse_translate($input, $numeric)? };
            Rotate, "rotate", [], "baseline.property.rotate", CssRotate, CssRotatePropertyValue, CssRotatePropertyValueRepresentation, parse_rotate, { parse_rotate($input)? };
            Scale, "scale", [], "baseline.property.scale", CssScale, CssScalePropertyValue, CssScalePropertyValueRepresentation, parse_scale, { parse_scale($input)? };
            Filter, "filter", [], "baseline.property.filter", CssFilter, CssFilterPropertyValue, CssFilterPropertyValueRepresentation, parse_filter, { parse_filter($input, $numeric)? };
            BackdropFilter, "backdrop-filter", [], "baseline.property.backdrop-filter", CssFilter, CssBackdropFilterPropertyValue, CssBackdropFilterPropertyValueRepresentation, parse_filter, { parse_filter($input, $numeric)? };
            ClipPath, "clip-path", [], "baseline.property.clip-path", CssClipPath, CssClipPathPropertyValue, CssClipPathPropertyValueRepresentation, parse_clip_path, { parse_clip_path($input, $numeric)? };
            BackgroundBlendMode, "background-blend-mode", [], "official.property.background-blend-mode", CssBlendModeList, CssBackgroundBlendModePropertyValue, CssBackgroundBlendModePropertyValueRepresentation, parse_blend_mode_list, { parse_blend_mode_list($input)? };
            Isolation, "isolation", [], "official.property.isolation", CssIsolation, CssIsolationPropertyValue, CssIsolationPropertyValueRepresentation, parse_isolation, { parse_isolation($input)? };
            MixBlendMode, "mix-blend-mode", [], "official.property.mix-blend-mode", CssBlendMode, CssMixBlendModePropertyValue, CssMixBlendModePropertyValueRepresentation, parse_blend_mode, { parse_blend_mode($input)? };
            Mask, "mask", [], "baseline.property.mask", CssMaskList, CssMaskPropertyValue, CssMaskPropertyValueRepresentation, parse_mask_list, { parse_mask_list($input, $numeric)? };
            MaskImage, "mask-image", [], "baseline.property.mask-image", CssImageLayerList, CssMaskImagePropertyValue, CssMaskImagePropertyValueRepresentation, parse_image_layer_list, { parse_image_layer_list($input, $numeric)? }, expansion = longhand { wrapper: existing, value: CssImageValueList, accessor: images, inherited: false, initial_kind: value, initial: CssImageValueList::try_new(vec![CssImageValue::None]).expect("one initial image") };
            MaskSize, "mask-size", [], "baseline.property.mask-size", CssBackgroundSizeList, CssMaskSizePropertyValue, CssMaskSizePropertyValueRepresentation, parse_background_size_list, { parse_background_size_list($input, $numeric)? };
            MaskPosition, "mask-position", [], "baseline.property.mask-position", CssMaskPositionList, CssMaskPositionPropertyValue, CssMaskPositionPropertyValueRepresentation, parse_mask_position_list, { parse_mask_position_list($input, $numeric)? };
            MaskRepeat, "mask-repeat", [], "baseline.property.mask-repeat", CssBackgroundRepeatList, CssMaskRepeatPropertyValue, CssMaskRepeatPropertyValueRepresentation, parse_background_repeat_list, { parse_background_repeat_list($input)? };
            TransitionProperty, "transition-property", [], "baseline.property.transition-property", CssTransitionPropertyList, CssTransitionPropertyPropertyValue, CssTransitionPropertyPropertyValueRepresentation, parse_transition_property_list, { parse_transition_property_list($input)? };
            TransitionDuration, "transition-duration", [], "baseline.property.transition-duration", CssTimeList, CssTransitionDurationPropertyValue, CssTransitionDurationPropertyValueRepresentation, parse_duration_list, { parse_duration_list($input, $numeric)? };
            TransitionDelay, "transition-delay", [], "baseline.property.transition-delay", CssTimeList, CssTransitionDelayPropertyValue, CssTransitionDelayPropertyValueRepresentation, parse_delay_list, { parse_delay_list($input, $numeric)? };
            TransitionTimingFunction, "transition-timing-function", [], "baseline.property.transition-timing-function", CssEasingList, CssTransitionTimingFunctionPropertyValue, CssTransitionTimingFunctionPropertyValueRepresentation, parse_easing_list, { parse_easing_list($input, $numeric)? };
            Transition, "transition", [], "baseline.property.transition", CssTransitionList, CssTransitionPropertyValue, CssTransitionPropertyValueRepresentation, parse_transition_value_list, { parse_transition_value_list($input, $numeric)? };
            AnimationName, "animation-name", [], "baseline.property.animation-name", CssAnimationNameList, CssAnimationNamePropertyValue, CssAnimationNamePropertyValueRepresentation, parse_animation_name_list, { parse_animation_name_list($input)? };
            AnimationDuration, "animation-duration", [], "baseline.property.animation-duration", CssTimeList, CssAnimationDurationPropertyValue, CssAnimationDurationPropertyValueRepresentation, parse_duration_list, { parse_duration_list($input, $numeric)? };
            AnimationDelay, "animation-delay", [], "baseline.property.animation-delay", CssTimeList, CssAnimationDelayPropertyValue, CssAnimationDelayPropertyValueRepresentation, parse_delay_list, { parse_delay_list($input, $numeric)? };
            AnimationTimingFunction, "animation-timing-function", [], "baseline.property.animation-timing-function", CssEasingList, CssAnimationTimingFunctionPropertyValue, CssAnimationTimingFunctionPropertyValueRepresentation, parse_easing_list, { parse_easing_list($input, $numeric)? };
            AnimationIterationCount, "animation-iteration-count", [], "baseline.property.animation-iteration-count", CssAnimationIterationCountList, CssAnimationIterationCountPropertyValue, CssAnimationIterationCountPropertyValueRepresentation, parse_animation_iteration_value_list, { parse_animation_iteration_value_list($input, $numeric)? };
            AnimationDirection, "animation-direction", [], "baseline.property.animation-direction", CssAnimationDirectionList, CssAnimationDirectionPropertyValue, CssAnimationDirectionPropertyValueRepresentation, parse_animation_direction_list, { parse_animation_direction_list($input)? };
            AnimationFillMode, "animation-fill-mode", [], "baseline.property.animation-fill-mode", CssAnimationFillModeList, CssAnimationFillModePropertyValue, CssAnimationFillModePropertyValueRepresentation, parse_animation_fill_mode_list, { parse_animation_fill_mode_list($input)? };
            AnimationPlayState, "animation-play-state", [], "baseline.property.animation-play-state", CssAnimationPlayStateList, CssAnimationPlayStatePropertyValue, CssAnimationPlayStatePropertyValueRepresentation, parse_animation_play_state_list, { parse_animation_play_state_list($input)? };
            Animation, "animation", [], "baseline.property.animation", CssAnimationList, CssAnimationPropertyValue, CssAnimationPropertyValueRepresentation, parse_animation_value_list, { parse_animation_value_list($input, $numeric)? };
        }
    };
}

pub(crate) use property_schema;

/// Projects only exactly representable legacy numeric payloads; callers apply
/// their own property domain after this shared conversion.
fn length_percentage_i01_projection(value: &CssSpecifiedLengthPercentage) -> Option<CssLength> {
    if let Some(calculation) = value.calculation() {
        return Some(CssLength::Calc(CssCalcLength::Typed(calculation.clone())));
    }
    let component = value.literal_component()?;
    match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
            (crate::exact_decimal::exact_legacy_value(number.representation())? == 0.0)
                .then_some(CssLength::Zero)
        }
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => {
            CssLength::try_percent(crate::exact_decimal::exact_legacy_value(
                number.representation(),
            )?)
        }
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
            CssLength::try_dimension(
                crate::exact_decimal::exact_legacy_value(number.representation())?,
                CssLengthUnit::from_css_unit(unit)?,
            )
        }
        _ => None,
    }
}

fn inset_i01_projection(value: &CssInsetValue) -> Option<CssLength> {
    match value {
        CssInsetValue::Auto => Some(CssLength::Auto),
        CssInsetValue::LengthPercentage(value) => length_percentage_i01_projection(value),
    }
}

fn spacing_length_i01_projection(value: &CssSpecifiedLengthPercentage) -> Option<CssLength> {
    let length = length_percentage_i01_projection(value)?;
    // The historical pure-length constructors can re-admit a calculation under
    // a length root, so reject percentage context before that reconstruction.
    if matches!(&length, CssLength::Calc(calculation) if calculation.uses_percentage()) {
        return None;
    }
    Some(length)
}

fn word_spacing_i01_projection(value: &CssTextSpacingAdjustment) -> Option<CssWordSpacing> {
    match value {
        CssTextSpacingAdjustment::Normal => Some(CssWordSpacing::Normal),
        CssTextSpacingAdjustment::LengthPercentage(value) => {
            CssWordSpacingLength::try_new(spacing_length_i01_projection(value)?)
                .map(CssWordSpacing::Length)
        }
    }
}

fn letter_spacing_i01_projection(value: &CssTextSpacingAdjustment) -> Option<CssLetterSpacing> {
    match value {
        CssTextSpacingAdjustment::Normal => Some(CssLetterSpacing::Normal),
        CssTextSpacingAdjustment::LengthPercentage(value) => {
            CssLetterSpacingLength::try_new(spacing_length_i01_projection(value)?)
                .map(CssLetterSpacing::Length)
        }
    }
}

fn inset_shorthand_i01_projection(value: &CssInsetShorthand) -> Option<CssEdges> {
    if value.kind() != CssBoxSideKind::Physical {
        return None;
    }
    let [top, right, bottom, left] = value.assigned_values();
    Some(CssEdges::new(
        inset_i01_projection(top)?,
        inset_i01_projection(right)?,
        inset_i01_projection(bottom)?,
        inset_i01_projection(left)?,
    ))
}

fn display_i01_projection(value: &CssDisplayValue) -> Option<CssDisplay> {
    match value {
        CssDisplayValue::OutsideInside {
            outside: CssDisplayOutside::Block,
            inside: CssDisplayInside::Flow,
        } => Some(CssDisplay::Block),
        CssDisplayValue::OutsideInside {
            outside: CssDisplayOutside::Block,
            inside: CssDisplayInside::Flex,
        } => Some(CssDisplay::Flex),
        CssDisplayValue::OutsideInside {
            outside: CssDisplayOutside::Block,
            inside: CssDisplayInside::Grid,
        } => Some(CssDisplay::Grid),
        CssDisplayValue::Legacy(CssDisplayLegacy::InlineBlock) => Some(CssDisplay::InlineBlock),
        CssDisplayValue::Legacy(CssDisplayLegacy::InlineGrid) => Some(CssDisplay::InlineGrid),
        CssDisplayValue::GridLanes => Some(CssDisplay::GridLanes),
        CssDisplayValue::InlineGridLanes => Some(CssDisplay::InlineGridLanes),
        CssDisplayValue::Box(CssDisplayBox::None) => Some(CssDisplay::None),
        _ => None,
    }
}

fn opacity_i01_projection(value: &CssOpacityValue) -> Option<CssOpacity> {
    match value {
        CssOpacityValue::Literal(value) => Some(*value),
        CssOpacityValue::Calculation(_)
        | CssOpacityValue::Number(_)
        | CssOpacityValue::Percentage(_)
        | CssOpacityValue::PercentageCalculation(_)
        | CssOpacityValue::ExactScalar(_) => None,
    }
}

fn flex_factor_i01_projection(value: &CssSpecifiedNonNegativeNumber) -> Option<CssFlexFactor> {
    let component = value.literal_component()?;
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
        return None;
    };
    CssFlexFactor::try_new(crate::exact_decimal::exact_legacy_value(
        number.representation(),
    )?)
}

fn flex_basis_i01_projection(value: &CssFlexBasisValue) -> Option<CssLength> {
    match value.view() {
        CssFlexBasisRef::Content | CssFlexBasisRef::CalcSize(_) => None,
        CssFlexBasisRef::Size(CssSizeValue::Auto) => Some(CssLength::Auto),
        CssFlexBasisRef::Size(CssSizeValue::BoxSize(box_size)) => match box_size {
            CssBoxSize::MinContent => Some(CssLength::MinContent),
            CssBoxSize::MaxContent => Some(CssLength::MaxContent),
            CssBoxSize::FitContent => Some(CssLength::FitContent),
            CssBoxSize::LengthPercentage(value) => {
                if let Some(calculation) = value.calculation() {
                    return Some(CssLength::Calc(CssCalcLength::Typed(calculation.clone())));
                }
                match value.literal_component()?.view() {
                    CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
                        (crate::exact_decimal::exact_legacy_value(number.representation())? == 0.0)
                            .then_some(CssLength::Zero)
                    }
                    CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => {
                        CssLength::try_percent(crate::exact_decimal::exact_legacy_value(
                            number.representation(),
                        )?)
                    }
                    CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
                        CssLength::try_dimension(
                            crate::exact_decimal::exact_legacy_value(number.representation())?,
                            CssLengthUnit::from_css_unit(unit)?,
                        )
                    }
                    _ => None,
                }
            }
            _ => None,
        },
    }
}

fn integer_i01_projection(value: &CssIntegerValue) -> Option<i32> {
    match value {
        CssIntegerValue::Literal(value) => Some(*value),
        CssIntegerValue::ExactLiteral(value) => {
            crate::integer_value::exact_i32(value.numeric().representation())
        }
        CssIntegerValue::Calculation(_) => None,
    }
}

fn aspect_ratio_i01_projection(value: &CssAspectRatioValue) -> Option<CssAspectRatio> {
    match value {
        CssAspectRatioValue::Ratio(ratio) if ratio.denominator().is_none() => {
            CssAspectRatio::try_new(ratio.numerator().exact_positive_f32()?)
        }
        _ => None,
    }
}

fn flex_i01_projection(value: &CssFlexValue) -> Option<CssFlex> {
    match value {
        CssFlexValue::None => Some(CssFlex::None),
        CssFlexValue::Auto => Some(CssFlex::Auto),
        CssFlexValue::Components(components) => {
            let grow = flex_factor_i01_projection(components.grow()?)?;
            let shrink = match components.shrink() {
                Some(value) => Some(flex_factor_i01_projection(value)?),
                None => None,
            };
            Some(CssFlex::components(
                grow,
                shrink,
                match components.basis() {
                    Some(basis) => Some(flex_basis_i01_projection(basis)?),
                    None => None,
                },
            ))
        }
    }
}

fn duration_i01_projection(value: &CssDuration) -> Option<CssTime> {
    match value {
        CssDuration::Literal(value) => CssTime::try_new(value.value(), value.unit()),
        CssDuration::Calculation(_) => None,
    }
}

fn delay_i01_projection(value: &CssDelay) -> Option<CssTime> {
    match value {
        CssDelay::Literal(value) => CssTime::try_new(value.value(), value.unit()),
        CssDelay::Calculation(_) => None,
    }
}

fn duration_list_i01_projection(value: &CssDurationList) -> Option<CssTimeList> {
    let values = value
        .values()
        .iter()
        .map(duration_i01_projection)
        .collect::<Option<Vec<_>>>()?;
    CssTimeList::try_new(values)
}

fn delay_list_i01_projection(value: &CssDelayList) -> Option<CssTimeList> {
    let values = value
        .values()
        .iter()
        .map(delay_i01_projection)
        .collect::<Option<Vec<_>>>()?;
    CssTimeList::try_new(values)
}

fn iteration_i01_projection(
    value: &CssAnimationIterationValue,
) -> Option<CssAnimationIterationCount> {
    match value {
        CssAnimationIterationValue::Infinite => Some(CssAnimationIterationCount::Infinite),
        CssAnimationIterationValue::Number(value) => {
            CssAnimationIterationCount::try_number(value.value())
        }
        CssAnimationIterationValue::Calculation(_) => None,
    }
}

fn iteration_list_i01_projection(
    value: &CssAnimationIterationValueList,
) -> Option<CssAnimationIterationCountList> {
    let values = value
        .values()
        .iter()
        .map(iteration_i01_projection)
        .collect::<Option<Vec<_>>>()?;
    CssAnimationIterationCountList::try_new(values)
}

fn transition_i01_projection(value: &CssTransitionValue) -> Option<CssTransition> {
    let duration = match value.duration() {
        Some(value) => Some(duration_i01_projection(value)?),
        None => None,
    };
    let delay = match value.delay() {
        Some(value) => Some(delay_i01_projection(value)?),
        None => None,
    };
    CssTransition::try_new(
        value.property().cloned(),
        duration,
        delay,
        value.timing_function().cloned(),
    )
}

fn transition_list_i01_projection(value: &CssTransitionValueList) -> Option<CssTransitionList> {
    let values = value
        .values()
        .iter()
        .map(transition_i01_projection)
        .collect::<Option<Vec<_>>>()?;
    CssTransitionList::try_new(values)
}

fn animation_i01_projection(value: &CssAnimationValue) -> Option<CssAnimation> {
    let duration = match value.duration() {
        Some(value) => Some(duration_i01_projection(value)?),
        None => None,
    };
    let delay = match value.delay() {
        Some(value) => Some(delay_i01_projection(value)?),
        None => None,
    };
    let iteration_count = match value.iteration_count() {
        Some(value) => Some(iteration_i01_projection(value)?),
        None => None,
    };
    CssAnimation::try_new(CssAnimationComponents {
        name: value.name().cloned(),
        duration,
        delay,
        timing_function: value.timing_function().cloned(),
        iteration_count,
        direction: value.direction(),
        fill_mode: value.fill_mode(),
        play_state: value.play_state(),
    })
}

fn animation_list_i01_projection(value: &CssAnimationValueList) -> Option<CssAnimationList> {
    let values = value
        .values()
        .iter()
        .map(animation_i01_projection)
        .collect::<Option<Vec<_>>>()?;
    CssAnimationList::try_new(values)
}

fn background_position_list_i01_projection(
    value: &CssBackgroundPositionList,
) -> Option<CssPositionList> {
    let positions = value
        .positions()
        .iter()
        .map(|position| position.legacy().cloned())
        .collect::<Option<Vec<_>>>()?;
    CssPositionList::try_new(positions)
}

fn mask_position_list_i01_projection(value: &CssMaskPositionList) -> Option<CssPositionList> {
    let positions = value
        .positions()
        .iter()
        .map(|position| position.legacy().cloned())
        .collect::<Option<Vec<_>>>()?;
    CssPositionList::try_new(positions)
}

fn transform_origin_i01_projection(value: &CssTransformOrigin) -> Option<CssPosition> {
    value.legacy().cloned()
}

fn background_box_list_i01_projection(value: &CssBackgroundBoxList) -> Option<CssBackgroundBox> {
    match value.boxes() {
        [value] => Some(*value),
        _ => None,
    }
}

fn exact_i01_projection<T: Clone>(value: &T) -> Option<T> {
    Some(value.clone())
}

macro_rules! define_current_property_value {
    (
        $canonical:literal, $wrapper:ident, $representation:ident,
        $current:ty, $i01:ty, $accessor:ident, $projection:expr
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: $current,
            i01_subset: Option<$i01>,
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(authored: CssAuthoredDeclarationValue, current: $current) -> Self {
                let i01_subset = ($projection)(&current);
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            #[must_use]
            pub const fn $accessor(&self) -> &$current {
                &self.representation.current
            }

            #[must_use]
            pub const fn i01_subset(&self) -> Option<&$i01> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

// The old inset calc projection retains diagnostic origins. It is redundant
// with `current`, so wrapper equality uses authored spelling and the checked
// current value while still exposing the historical projection.
macro_rules! define_inset_property_value {
    ($canonical:literal, $wrapper:ident, $representation:ident,
        $current:ty, $i01:ty, $projection:expr) => {
        #[derive(Clone, Debug)]
        pub(crate) struct $representation {
            current: $current,
            i01_subset: Option<$i01>,
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl PartialEq for $wrapper {
            fn eq(&self, other: &Self) -> bool {
                self.authored == other.authored
                    && self.representation.current == other.representation.current
            }
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(authored: CssAuthoredDeclarationValue, current: $current) -> Self {
                let i01_subset = ($projection)(&current);
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }
            #[must_use]
            pub const fn current(&self) -> &$current {
                &self.representation.current
            }
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&$i01> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

macro_rules! define_additive_current_property_value {
    (
        $canonical:literal, $wrapper:ident, $representation:ident,
        $current:ty, $accessor:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: $current,
        }

        #[doc = concat!("A grammar-checked current authored value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) const fn new(
                authored: CssAuthoredDeclarationValue,
                current: $current,
            ) -> Self {
                Self {
                    authored,
                    representation: $representation { current },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            #[must_use]
            pub const fn $accessor(&self) -> &$current {
                &self.representation.current
            }
        }
    };
}

macro_rules! define_easing_property_value {
    ($canonical:literal, $wrapper:ident, $representation:ident) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: CssEasingValueList,
            i01_subset: Option<CssEasingList>,
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedEasingList,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact checked current authored easing list.
            #[must_use]
            pub const fn current(&self) -> &CssEasingValueList {
                &self.representation.current
            }

            /// Returns the frozen keyword/authored-arguments compatibility projection.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssEasingList> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

macro_rules! define_color_property_value {
    ($canonical:literal, $wrapper:ident, $representation:ident) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: CssAuthoredColor,
            i01_subset: Option<CssColor>,
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedColor,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact checked current authored color.
            #[must_use]
            pub const fn current(&self) -> &CssAuthoredColor {
                &self.representation.current
            }

            /// Returns the frozen I01 compatibility payload when the authored value has an exact
            /// representation in that model.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssColor> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

macro_rules! define_authored_color_aggregate_property_value {
    ($canonical:literal, $wrapper:ident, $representation:ident, $value:ty) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: $value,
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) const fn new(
                authored: CssAuthoredDeclarationValue,
                current: $value,
            ) -> Self {
                Self {
                    authored,
                    representation: $representation { current },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact checked current authored aggregate value.
            #[must_use]
            pub const fn current(&self) -> &$value {
                &self.representation.current
            }

            /// Returns the frozen I01 payload only when every authored component projects exactly.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&$value> {
                if self.representation.current.has_exact_i01_projection() {
                    Some(&self.representation.current)
                } else {
                    None
                }
            }
        }
    };
}

macro_rules! define_border_width_property_value {
    ($canonical:literal, $wrapper:ident, $representation:ident, $current:ty, $legacy:ty, $projection:expr) => {
        #[derive(Clone, Debug)]
        pub(crate) struct $representation {
            current: $current,
            i01_subset: Option<$legacy>,
        }
        #[doc = concat!("A checked exact authored value for `", $canonical, "`.")]
        #[derive(Clone, Debug)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }
        impl PartialEq for $wrapper {
            fn eq(&self, other: &Self) -> bool {
                self.authored == other.authored
                    && self.representation.current == other.representation.current
            }
        }
        impl $wrapper {
            #[must_use]
            pub(crate) fn new(authored: CssAuthoredDeclarationValue, current: $current) -> Self {
                let i01_subset = ($projection)(&current);
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }
            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }
            #[must_use]
            pub const fn current(&self) -> &$current {
                &self.representation.current
            }
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&$legacy> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

macro_rules! define_border_triple_property_value {
    ($canonical:literal, $wrapper:ident, $representation:ident) => {
        #[derive(Clone, Debug)]
        pub(crate) struct $representation {
            current: CssBorderValue,
            i01_subset: Option<CssBorder>,
        }
        #[doc = concat!("A checked exact authored value for `", $canonical, "`.")]
        #[derive(Clone, Debug)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }
        impl PartialEq for $wrapper {
            fn eq(&self, other: &Self) -> bool {
                self.authored == other.authored
                    && self.representation.current == other.representation.current
            }
        }
        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedBorderValue,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }
            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }
            #[must_use]
            pub const fn current(&self) -> &CssBorderValue {
                &self.representation.current
            }
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssBorder> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

macro_rules! define_grid_property_value {
    (
        $canonical:literal, $wrapper:ident, $representation:ident,
        $current:ty, $i01:ty, $parsed:ty
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: $current,
            i01_subset: Option<$i01>,
        }

        #[doc = concat!("A grammar-checked current authored value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(authored: CssAuthoredDeclarationValue, parsed: $parsed) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the grammar-checked current authored Grid value.
            #[must_use]
            pub const fn current(&self) -> &$current {
                &self.representation.current
            }

            /// Returns the frozen I01 compatibility payload when the current value projects
            /// exactly into that representation.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&$i01> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

fn overflow_i01_projection(value: CssOverflow) -> Option<CssOverflow> {
    match value {
        CssOverflow::Visible | CssOverflow::Hidden | CssOverflow::Clip | CssOverflow::Scroll => {
            Some(value)
        }
        CssOverflow::Auto => None,
    }
}

macro_rules! define_property_value {
    (CounterReset, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            crate::CssCounterChangesValue,
            CssCounterChanges,
            current,
            crate::counter_changes::changes_i01
        );
    };
    (CounterIncrement, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            crate::CssCounterChangesValue,
            CssCounterChanges,
            current,
            crate::counter_changes::changes_i01
        );
    };
    (CounterSet, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            crate::CssCounterChangesValue,
            CssCounterChanges,
            current,
            crate::counter_changes::changes_i01
        );
    };
    (ListStyleType, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            crate::CssListStyleTypeValue,
            CssListStyleType,
            current,
            crate::list_styles::type_i01
        );
    };
    (ListStylePosition, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssListStylePosition,
            CssListStylePosition,
            current,
            |value: &CssListStylePosition| Some(*value)
        );
    };
    (ListStyleImage, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssImageValue,
            CssListStyleImage,
            current,
            crate::list_styles::image_i01
        );
    };
    (ListStyle, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            crate::CssListStyleValue,
            CssListStyle,
            current,
            crate::list_styles::shorthand_i01
        );
    };
    (Content, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssContentValue,
            CssContent,
            current,
            content_i01
        );
    };
    (AlignContent, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAlignContentValue,
            CssAlignment,
            current,
            crate::alignment::align_content_i01
        );
    };
    (JustifyContent, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssJustifyContentValue,
            CssAlignment,
            current,
            crate::alignment::justify_content_i01
        );
    };
    (AlignItems, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAlignItemsValue,
            CssAlignItems,
            current,
            crate::alignment::align_items_i01
        );
    };
    (JustifyItems, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssJustifyItemsValue,
            CssAlignItems,
            current,
            crate::alignment::justify_items_i01
        );
    };
    (AlignSelf, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAlignSelfValue,
            CssAlignItems,
            current,
            crate::alignment::align_self_i01
        );
    };
    (JustifySelf, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssJustifySelfValue,
            CssAlignItems,
            current,
            crate::alignment::justify_self_i01
        );
    };
    (PlaceContent, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssPlaceContentValue,
            CssPlaceAlignment,
            current,
            crate::alignment::place_content_i01
        );
    };
    (PlaceItems, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssPlaceItemsValue,
            CssPlaceAlignment,
            current,
            crate::alignment::place_items_i01
        );
    };
    (PlaceSelf, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssPlaceSelfValue,
            CssPlaceAlignment,
            current,
            crate::alignment::place_self_i01
        );
    };
    (TextAlign, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssTextAlignValue,
            CssTextAlign,
            current,
            crate::text_alignment::text_align_i01_projection
        );
    };
    (TextAlignLast, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssTextAlignLastValue,
            CssTextAlignLast,
            current,
            crate::text_alignment::text_align_last_i01_projection
        );
    };
    (BorderRadius, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderRadiusShorthand,
            CssBorderRadii,
            crate::border_radius::legacy_shorthand
        );
    };
    (BorderTopLeftRadius, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssCornerRadiusValue,
            CssCornerRadius,
            crate::border_radius::legacy_corner
        );
    };
    (BorderTopRightRadius, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssCornerRadiusValue,
            CssCornerRadius,
            crate::border_radius::legacy_corner
        );
    };
    (BorderBottomRightRadius, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssCornerRadiusValue,
            CssCornerRadius,
            crate::border_radius::legacy_corner
        );
    };
    (BorderBottomLeftRadius, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssCornerRadiusValue,
            CssCornerRadius,
            crate::border_radius::legacy_corner
        );
    };
    (BorderStyle, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderStyleShorthand,
            CssBorderStyles,
            crate::border_style::legacy_shorthand
        );
    };
    (BorderWidth, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderWidthShorthand,
            CssEdges,
            crate::border_width::legacy_shorthand
        );
    };
    (BorderTopWidth, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderWidth,
            CssLength,
            crate::border_width::legacy_width
        );
    };
    (BorderRightWidth, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderWidth,
            CssLength,
            crate::border_width::legacy_width
        );
    };
    (BorderBottomWidth, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderWidth,
            CssLength,
            crate::border_width::legacy_width
        );
    };
    (BorderLeftWidth, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_border_width_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderWidth,
            CssLength,
            crate::border_width::legacy_width
        );
    };
    (Inset, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_inset_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssInsetShorthand,
            CssEdges,
            inset_shorthand_i01_projection
        );
    };
    (Top, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_inset_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssInsetValue,
            CssLength,
            inset_i01_projection
        );
    };
    (Right, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_inset_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssInsetValue,
            CssLength,
            inset_i01_projection
        );
    };
    (Bottom, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_inset_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssInsetValue,
            CssLength,
            inset_i01_projection
        );
    };
    (Left, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_inset_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssInsetValue,
            CssLength,
            inset_i01_projection
        );
    };
    (Overflow, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssOverflowValue,
            CssOverflowI01PropertyValue,
            current,
            |value: &CssOverflowValue| {
                let x = overflow_i01_projection(value.x())?;
                match value.authored_y() {
                    Some(y) => Some(CssOverflowI01PropertyValue::Pair(CssOverflowAxes::new(
                        x,
                        overflow_i01_projection(y)?,
                    ))),
                    None => Some(CssOverflowI01PropertyValue::Single(x)),
                }
            }
        );
    };
    (OverflowX, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssOverflow,
            CssOverflow,
            current,
            |value: &CssOverflow| overflow_i01_projection(*value)
        );
    };
    (OverflowY, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssOverflow,
            CssOverflow,
            current,
            |value: &CssOverflow| overflow_i01_projection(*value)
        );
    };
    (Float, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFloat,
            CssFloat,
            current,
            |value: &CssFloat| match value {
                CssFloat::None | CssFloat::Left | CssFloat::Right => Some(*value),
                CssFloat::InlineStart | CssFloat::InlineEnd => None,
            }
        );
    };
    (Clear, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssClear,
            CssClear,
            current,
            |value: &CssClear| match value {
                CssClear::None | CssClear::Left | CssClear::Right | CssClear::Both => Some(*value),
                CssClear::InlineStart | CssClear::InlineEnd => None,
            }
        );
    };
    (ContainerName, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            names
        );
    };
    (ContainerType, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            container_type
        );
    };
    (Container, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            container
        );
    };

    (FlexDirection, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFlexDirection,
            CssFlexDirection,
            current,
            |value: &CssFlexDirection| Some(*value)
        );
        impl $wrapper {
            pub const fn exact_i01_projection(&self) -> Option<&CssFlexDirection> {
                self.i01_subset()
            }
        }
    };
    (FlexWrap, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFlexWrap,
            CssFlexWrap,
            current,
            |value: &CssFlexWrap| Some(*value)
        );
        impl $wrapper {
            pub const fn exact_i01_projection(&self) -> Option<&CssFlexWrap> {
                self.i01_subset()
            }
        }
    };
    (FlexFlow, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFlexFlow,
            flow
        );
    };
    (
        ColumnCount, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            count
        );
    };
    (
        ColumnFill, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            fill
        );
    };
    (
        ColumnRule, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            rule
        );
    };
    (
        ColumnRuleColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        ColumnRuleStyle, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            style
        );
    };
    (
        ColumnRuleWidth, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            width
        );
    };
    (
        ColumnSpan, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            span
        );
    };
    (
        ColumnWidth, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            width
        );
    };
    (
        Columns, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            columns
        );
    };
    (
        BorderCollapse, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            collapse
        );
    };
    (
        BorderSpacing, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            spacing
        );
    };
    (
        CaptionSide, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            side
        );
    };
    (
        Clip, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            clip
        );
    };
    (
        EmptyCells, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            cells
        );
    };
    (
        Orphans, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            minimum
        );
    };
    (
        BreakAfter, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            current
        );
    };
    (
        BreakBefore, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            current
        );
    };
    (
        BreakInside, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            current
        );
    };
    (
        Quotes, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            quotes
        );
    };
    (
        TableLayout, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            layout
        );
    };
    (
        Widows, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            minimum
        );
    };
    (
        WordSpacing, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssTextSpacingAdjustment,
            CssWordSpacing,
            spacing,
            word_spacing_i01_projection
        );
    };
    (
        LetterSpacing, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssTextSpacingAdjustment,
            CssLetterSpacing,
            current,
            letter_spacing_i01_projection
        );
    };
    (
        TextCombineUpright, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            combine
        );
    };
    (
        TextOrientation, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            orientation
        );
    };
    (
        UnicodeBidi, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            bidi
        );
    };
    (
        CaretColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            caret
        );
    };
    (
        OutlineOffset, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            offset
        );
    };
    (
        Resize, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            resize
        );
    };
    (
        Contain, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            containment
        );
    };
    (
        TransformBox, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            reference_box
        );
    };
    (
        BackgroundBlendMode, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            modes
        );
    };
    (
        Isolation, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            isolation
        );
    };
    (
        MixBlendMode, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            mode
        );
    };
    (
        FontFamily, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontFamilyList,
            families
        );
    };
    (
        Font, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontValue,
            font
        );
    };
    (
        FontVariant, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantValue,
            variant
        );
    };
    (
        FontVariantCaps, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantCaps,
            caps
        );
    };
    (
        FontVariantEastAsian, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantEastAsian,
            east_asian
        );
    };
    (
        FontVariantLigatures, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantLigatures,
            ligatures
        );
    };
    (
        FontVariantNumeric, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantNumeric,
            numeric
        );
    };
    (
        FontVariantPosition, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantPosition,
            position
        );
    };
    (
        FontVariantAlternates, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantAlternates,
            alternates
        );
    };
    (
        FontVariantEmoji, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantEmoji,
            emoji
        );
    };
    (
        FontFeatureSettings, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredFontFeatureSettings,
            settings
        );
    };
    (
        FontKerning, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontKerning,
            kerning
        );
    };
    (
        FontSizeAdjust, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontSizeAdjust,
            size_adjust
        );
    };
    (
        FontSynthesis, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontSynthesis,
            synthesis
        );
    };
    (
        GridTemplateRows, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_grid_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridTrackList,
            CssGridTrackList,
            CssParsedGridTrackList
        );
    };
    (
        GridTemplateColumns, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_grid_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridTrackList,
            CssGridTrackList,
            CssParsedGridTrackList
        );
    };
    (
        GridAutoRows, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_grid_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridTrackSizeList,
            CssGridTrackList,
            CssParsedGridTrackSizeList
        );
    };
    (
        GridAutoColumns, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_grid_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridTrackSizeList,
            CssGridTrackList,
            CssParsedGridTrackSizeList
        );
    };
    (
        GridTemplate, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_grid_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridTemplateValue,
            CssGridTemplate,
            CssParsedGridTemplate
        );
    };
    (
        Grid, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_grid_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridValue,
            CssGrid,
            CssParsedGrid
        );
    };
    (
        Color, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        Background, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: CssBackground,
            color_component: CssAuthoredColor,
            i01_subset: Option<CssColor>,
        }

        /// A grammar-checked authored ordinary value for `background`.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedBackground,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                let color_component = current
                    .layers()
                    .last()
                    .and_then(CssBackgroundLayer::color)
                    .cloned()
                    .unwrap_or_else(CssAuthoredColor::transparent);
                Self {
                    authored,
                    representation: $representation {
                        current,
                        color_component,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact authored shorthand layers.
            #[must_use]
            pub const fn background(&self) -> &CssBackground {
                &self.representation.current
            }

            /// Returns the shorthand's color component, preserving the existing
            /// color accessor and using its specified transparent initial when omitted.
            #[must_use]
            pub const fn current(&self) -> &CssAuthoredColor {
                &self.representation.color_component
            }

            /// Returns the frozen color-only I01 payload when the entire shorthand
            /// belongs to that compatibility subset.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssColor> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
    (
        BackgroundColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: Box<CssBorderColorShorthand>,
            i01_subset: Option<CssColor>,
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedBorderColorShorthand,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current: Box::new(current),
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns authored colors and physical or flow-relative assignments;
            /// complete shorthand expansion and reset membership remain unsettled.
            #[must_use]
            pub const fn current(&self) -> &CssBorderColorShorthand {
                &self.representation.current
            }

            /// Returns the frozen I01 payload only for an exactly representable
            /// single authored color component.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssColor> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
    (
        BorderTopColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderRightColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderBottomColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderLeftColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        OutlineColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        TextDecorationColor, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (
        TextDecoration, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_authored_color_aggregate_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssTextDecoration
        );
    };
    (
        Border, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_border_triple_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderTop, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_border_triple_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderRight, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_border_triple_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderBottom, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_border_triple_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderLeft, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_border_triple_property_value!($canonical, $wrapper, $representation);
    };
    (
        Outline, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_authored_color_aggregate_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssOutline
        );
    };
    (
        BoxShadow, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: CssBoxShadow,
            has_i01_subset: bool,
        }

        /// A grammar-checked authored ordinary `box-shadow` value.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                current: CssBoxShadow,
            ) -> Self {
                let has_i01_subset = current.has_exact_i01_projection();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        has_i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact checked current authored shadow value.
            #[must_use]
            pub const fn current(&self) -> &CssBoxShadow {
                &self.representation.current
            }

            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssBoxShadow> {
                if self.representation.has_i01_subset {
                    Some(&self.representation.current)
                } else {
                    None
                }
            }
        }
    };
    (
        BackgroundImage, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_image_property_value!($canonical, $wrapper, $representation);
    };
    (
        BorderImage, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: Box<CssBorderImage>,
        }

        /// A grammar-checked authored ordinary value for `border-image`.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                current: CssBorderImage,
            ) -> Self {
                Self {
                    authored,
                    representation: $representation {
                        current: Box::new(current),
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            #[must_use]
            pub const fn border_image(&self) -> &CssBorderImage {
                &self.representation.current
            }
        }
    };
    (
        BorderImageOutset, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderImageOutset,
            outsets
        );
    };
    (
        BorderImageRepeat, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderImageRepeat,
            repeat
        );
    };
    (
        BorderImageSlice, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderImageSlice,
            slice
        );
    };
    (
        BorderImageSource, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssImageValue,
            source
        );
    };
    (
        BorderImageWidth, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorderImageWidth,
            widths
        );
    };
    (
        ImageOrientation, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssImageOrientation,
            orientation
        );
    };
    (
        ImageRendering, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssImageRendering,
            rendering
        );
    };
    (
        ObjectFit, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssObjectFit,
            fit
        );
    };
    (
        MaskImage, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_image_property_value!($canonical, $wrapper, $representation);
    };
    (
        Filter, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_filter_property_value!($canonical, $wrapper, $representation);
    };
    (
        BackdropFilter, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_filter_property_value!($canonical, $wrapper, $representation);
    };
    (
        ClipPath, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: Option<CssClipPathValue>,
            i01_subset: Option<CssClipPath>,
        }

        /// A grammar-checked authored ordinary `clip-path` value.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedClipPath,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact checked current authored clip-path subset, when representable.
            #[must_use]
            pub const fn current(&self) -> Option<&CssClipPathValue> {
                self.representation.current.as_ref()
            }

            /// Returns the frozen authored-arguments compatibility projection.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssClipPath> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
    (
        Transform, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: CssTransformValue,
            i01_subset: CssTransform,
        }

        /// A grammar-checked authored ordinary value for `transform`.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedTransform,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact checked current authored transform value.
            #[must_use]
            pub const fn current(&self) -> &CssTransformValue {
                &self.representation.current
            }

            /// Returns the frozen kind/authored-arguments compatibility projection.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssTransform> {
                Some(&self.representation.i01_subset)
            }
        }
    };
    (
        ObjectPosition, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: CssObjectPosition,
        }

        /// A grammar-checked authored ordinary value for `object-position`.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) const fn new(
                authored: CssAuthoredDeclarationValue,
                current: CssObjectPosition,
            ) -> Self {
                Self {
                    authored,
                    representation: $representation { current },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact authored object position.
            #[must_use]
            pub const fn position(&self) -> &CssObjectPosition {
                &self.representation.current
            }
        }
    };
    (
        BackgroundPosition, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBackgroundPositionList,
            CssPositionList,
            positions,
            background_position_list_i01_projection
        );
    };
    (
        BackgroundSize, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBackgroundSizeList,
            CssBackgroundSizeList,
            sizes,
            exact_i01_projection
        );
    };
    (
        BackgroundRepeat, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBackgroundRepeatList,
            CssBackgroundRepeatList,
            repeats,
            exact_i01_projection
        );
    };
    (
        BackgroundOrigin, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBackgroundBoxList,
            CssBackgroundBox,
            boxes,
            background_box_list_i01_projection
        );
    };
    (
        BackgroundClip, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBackgroundBoxList,
            CssBackgroundBox,
            boxes,
            background_box_list_i01_projection
        );
    };
    (
        BackgroundAttachment, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBackgroundAttachmentList,
            CssBackgroundAttachmentList,
            attachments,
            exact_i01_projection
        );
    };
    (
        MaskPosition, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssMaskPositionList,
            CssPositionList,
            positions,
            mask_position_list_i01_projection
        );
    };
    (
        TransformOrigin, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssTransformOrigin,
            CssPosition,
            origin,
            transform_origin_i01_projection
        );
    };
    (
        TransitionTimingFunction, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_easing_property_value!($canonical, $wrapper, $representation);
    };
    (
        AnimationTimingFunction, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_easing_property_value!($canonical, $wrapper, $representation);
    };
    (
        TransitionDuration, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssDurationList,
            CssTimeList,
            durations,
            duration_list_i01_projection
        );
    };
    (
        TransitionDelay, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssDelayList,
            CssTimeList,
            delays,
            delay_list_i01_projection
        );
    };
    (
        AnimationDuration, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssDurationList,
            CssTimeList,
            durations,
            duration_list_i01_projection
        );
    };
    (
        AnimationDelay, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssDelayList,
            CssTimeList,
            delays,
            delay_list_i01_projection
        );
    };
    (
        AnimationIterationCount, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAnimationIterationValueList,
            CssAnimationIterationCountList,
            iteration_counts,
            iteration_list_i01_projection
        );
    };
    (
        Transition, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssTransitionValueList,
            CssTransitionList,
            transitions,
            transition_list_i01_projection
        );
    };
    (
        Animation, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAnimationValueList,
            CssAnimationList,
            animations,
            animation_list_i01_projection
        );
    };
    (
        Display, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssDisplayValue,
            CssDisplay,
            value,
            display_i01_projection
        );
    };
    (
        Opacity, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssOpacityValue,
            CssOpacity,
            value,
            opacity_i01_projection
        );
    };
    (
        FlexBasis, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFlexBasisValue,
            CssLength,
            current,
            flex_basis_i01_projection
        );
    };
    (
        FlexGrow, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssSpecifiedNonNegativeNumber,
            CssFlexFactor,
            factor,
            flex_factor_i01_projection
        );
    };
    (
        FlexShrink, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssSpecifiedNonNegativeNumber,
            CssFlexFactor,
            factor,
            flex_factor_i01_projection
        );
    };
    (
        Order, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssIntegerValue,
            CssOrder,
            value,
            |value: &CssIntegerValue| integer_i01_projection(value).map(CssOrder::Integer)
        );
    };
    (
        ZIndex, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssZIndexValue,
            CssZIndex,
            value,
            |value: &CssZIndexValue| match value {
                CssZIndexValue::Auto => Some(CssZIndex::Auto),
                CssZIndexValue::Integer(value) => {
                    integer_i01_projection(value).map(CssZIndex::Integer)
                }
            }
        );
    };
    (
        AspectRatio, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAspectRatioValue,
            CssAspectRatio,
            ratio,
            aspect_ratio_i01_projection
        );
    };
    (
        Flex, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFlexValue,
            CssFlex,
            value,
            flex_i01_projection
        );
    };
    (
        FlowTolerance, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFlowTolerance,
            value
        );
    };
    (
        GridRowStart, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridLine,
            CssGridLine,
            current,
            CssAuthoredGridLine::i01_subset
        );
    };
    (
        GridRowEnd, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridLine,
            CssGridLine,
            current,
            CssAuthoredGridLine::i01_subset
        );
    };
    (
        GridColumnStart, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridLine,
            CssGridLine,
            current,
            CssAuthoredGridLine::i01_subset
        );
    };
    (
        GridColumnEnd, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridLine,
            CssGridLine,
            current,
            CssAuthoredGridLine::i01_subset
        );
    };
    (
        GridRow, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridLineRange,
            CssGridLineRange,
            current,
            CssAuthoredGridLineRange::i01_subset
        );
    };
    (
        GridColumn, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridLineRange,
            CssGridLineRange,
            current,
            CssAuthoredGridLineRange::i01_subset
        );
    };
    (
        GridArea, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredGridArea,
            CssGridArea,
            current,
            CssAuthoredGridArea::i01_subset
        );
    };
    (
        $variant:ident, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        enum $representation {
            I01($value),
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        ///
        /// The private representation preserves property coupling while `as_css()` retains the
        /// exact authored slice and `i01_subset()` exposes only the frozen I01 payload.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) const fn new(authored: CssAuthoredDeclarationValue, value: $value) -> Self {
                Self {
                    authored,
                    representation: $representation::I01(value),
                }
            }

            /// Returns the exact authored ordinary value slice, excluding boundary trivia and a
            /// terminal importance annotation.
            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the property parser's frozen I01 payload when this value belongs to that
            /// subset.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&$value> {
                match &self.representation {
                    $representation::I01(value) => Some(value),
                }
            }
        }
    };
}

macro_rules! define_filter_property_value {
    ($canonical:literal, $wrapper:ident, $representation:ident) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: CssFilterValue,
            i01_subset: Option<CssFilter>,
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedFilter,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact checked current authored filter value.
            #[must_use]
            pub const fn current(&self) -> &CssFilterValue {
                &self.representation.current
            }

            /// Returns the frozen authored-arguments compatibility projection.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssFilter> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

macro_rules! define_image_property_value {
    ($canonical:literal, $wrapper:ident, $representation:ident) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: CssImageValueList,
            i01_subset: Option<CssImageLayerList>,
        }

        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $representation,
        }

        impl $wrapper {
            #[must_use]
            pub(crate) fn new(
                authored: CssAuthoredDeclarationValue,
                parsed: CssParsedImageValueList,
            ) -> Self {
                let (current, i01_subset) = parsed.into_parts();
                Self {
                    authored,
                    representation: $representation {
                        current,
                        i01_subset,
                    },
                }
            }

            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the exact checked current authored image list.
            #[must_use]
            pub const fn images(&self) -> &CssImageValueList {
                &self.representation.current
            }

            /// Returns the frozen URL/`none` compatibility projection when every image belongs
            /// to that representation.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssImageLayerList> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
}

// Selected schema rows whose current payload still uses the original
// representation need an additive current-value accessor. Expansion never reads
// the frozen compatibility projection.
macro_rules! define_expansion_current_accessor {
    ($wrapper:ident, $representation:ident, $value:ty,
        $kind:ident { wrapper: fallback, $($metadata:tt)* }
    ) => {
        impl $wrapper {
            /// Returns the exact checked current authored value.
            #[must_use]
            pub const fn current(&self) -> &$value {
                match &self.representation {
                    $representation::I01(value) => value,
                }
            }
        }
    };
    ($wrapper:ident, $representation:ident, $value:ty $(, $kind:ident { $($metadata:tt)* })?) => {};
}

// The schema annotation selects a current-only wrapper for newly authored
// families without maintaining a second list of their property names.
macro_rules! define_property_value_from_schema {
    (FontLanguageOverride, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontLanguageOverride,
            language_override
        );
    };
    (FontOpticalSizing, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontOpticalSizing,
            optical_sizing
        );
    };
    (FontVariationSettings, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariationSettings,
            variations
        );
    };
    (LineHeight, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssLineHeight,
            line_height
        );
    };
    (FontSize, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontSize,
            size
        );
    };
    (BorderBlockStartColor, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (BorderBlockEndColor, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (BorderInlineStartColor, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    (BorderInlineEndColor, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_color_property_value!($canonical, $wrapper, $representation);
    };
    ($variant:ident, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        longhand { wrapper: additive, $($metadata:tt)* }) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            current
        );
    };
    ($variant:ident, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        shorthand { wrapper: additive, $($metadata:tt)* }) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            current
        );
    };
    ($variant:ident, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident,
        unresolved { wrapper: additive, $($metadata:tt)* }) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            current
        );
    };
    ($variant:ident, $canonical:literal, $value:ty, $wrapper:ident, $representation:ident
        $(, $kind:ident { $($metadata:tt)* })?) => {
        define_property_value!($variant, $canonical, $value, $wrapper, $representation);
    };
}

macro_rules! define_property_identity {
    ($input:ident, $numeric:ident;
        All, $all_canonical:literal, [$($all_alias:literal),*], $all_stable_id:literal,
        $all_value:ty, $all_wrapper:ident, $all_representation:ident,
        $all_parser:ident, $all_dispatch:block $(, expansion = $all_expansion:ident { $($all_metadata:tt)* })?;
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $wrapper:ident, $representation:ident, $parser:ident, $dispatch:block
        $(, expansion = $expansion:ident { $($metadata:tt)* })?;
    )*) => {
        /// Canonical identity for a recognized non-custom authored CSS property.
        ///
        /// Identity lookup is ASCII-case-insensitive and normalizes aliases to their canonical
        /// property. This type classifies authored syntax only; it does not apply cascade,
        /// substitute variables, or resolve property values.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum CssKnownProperty {
            /// The authored `all` property.
            All,
            $(
                #[doc = concat!("The authored `", $canonical, "` property.")]
                $variant,
            )*
        }

        impl CssKnownProperty {
            /// Looks up a canonical property name or reviewed alias without changing CSS meaning.
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                property_implementation_inventory()
                    .iter()
                    .find(|row| {
                        row.name.eq_ignore_ascii_case(name)
                            || row.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name))
                    })
                    .map(|row| row.known_property)
            }

            /// Returns every frozen canonical property identity in schema order.
            #[must_use]
            pub const fn all() -> &'static [Self] {
                KNOWN_PROPERTIES
            }

            /// Returns the canonical lowercase authored spelling.
            #[must_use]
            pub const fn canonical_name(self) -> &'static str {
                match self {
                    Self::All => $all_canonical,
                    $(Self::$variant => $canonical,)*
                }
            }

            /// Returns the stable baseline identity for this authored property.
            #[must_use]
            pub const fn stable_id(self) -> &'static str {
                match self {
                    Self::All => $all_stable_id,
                    $(Self::$variant => $stable_id,)*
                }
            }

            /// Returns reviewed authored aliases that normalize to this property.
            #[must_use]
            pub const fn aliases(self) -> &'static [&'static str] {
                match self {
                    Self::All => &[$($all_alias),*],
                    $(Self::$variant => &[$($alias),*],)*
                }
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        struct $all_representation($all_value);

        /// The authored ordinary value of the `all` schema row.
        ///
        /// The current grammar never constructs this wrapper because `all` accepts only global or
        /// substitution-dependent values. Its presence keeps the generated schema/view inventory
        /// exact without making either symbolic branch masquerade as an ordinary property value.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $all_wrapper {
            authored: CssAuthoredDeclarationValue,
            representation: $all_representation,
        }

        impl $all_wrapper {
            /// Returns the exact authored ordinary value slice.
            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the I01 payload when this value belongs to the frozen I01 subset.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&$all_value> {
                Some(&self.representation.0)
            }
        }

        $(
            define_property_value_from_schema!(
                $variant, $canonical, $value, $wrapper, $representation
                $(, $expansion { $($metadata)* })?
            );
            define_expansion_current_accessor!(
                $wrapper, $representation, $value
                $(, $expansion { $($metadata)* })?
            );
        )*

        /// A borrowed property-specific ordinary-value view.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum CssKnownPropertyValueRef<'a> {
            /// The generated `all` schema-row wrapper. Current parsing never produces this branch.
            All(&'a $all_wrapper),
            $(
                #[doc = concat!("An authored `", $canonical, "` ordinary value.")]
                $variant(&'a $wrapper),
            )*
        }

        /// A borrowed known declaration value in the authored syntax phase.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum CssKnownDeclaredValueRef<'a> {
            /// A property-specific ordinary authored value.
            Property(CssKnownPropertyValueRef<'a>),
            /// A whole-value CSS-wide keyword.
            Global(CssGlobalKeyword),
            /// Authored syntax whose grammar depends on later substitution.
            SubstitutionDependent(&'a CssSubstitutionDependentValue),
        }

        #[derive(Clone, Debug, PartialEq)]
        pub(crate) enum CssKnownDeclarationValue {
            All(CssAllDeclaredValue),
            $($variant(CssDeclaredValue<$wrapper>),)*
        }

        /// A grammar-checked property-coupled known declaration in the authored syntax phase.
        ///
        /// Parsing and [`crate::parse_property_value`] construct this value through the same
        /// property grammar. Its private value discriminator determines canonical property
        /// identity; the coupled grammar handle additionally preserves legacy authored semantics.
        /// Callers cannot construct or mutate a property/value/grammar mismatch.
        ///
        /// ```compile_fail
        /// use surgeist_css::CssKnownDeclaration;
        /// let _ = CssKnownDeclaration { value: todo!() };
        /// ```
        ///
        /// ```compile_fail
        /// use surgeist_css::{CssAuthoredDeclarationValue, CssWidthPropertyValue};
        /// let _ = CssWidthPropertyValue {
        ///     authored: CssAuthoredDeclarationValue::try_new("1px").unwrap(),
        ///     representation: todo!(),
        /// };
        /// ```
        #[derive(Clone, Debug, PartialEq)]
        pub struct CssKnownDeclaration {
            grammar: CssPropertyGrammar,
            value: CssKnownDeclarationValue,
        }

        impl CssKnownDeclaration {
            /// Returns the exact authored grammar, including legacy shorthand identity.
            #[must_use]
            pub const fn grammar(&self) -> CssPropertyGrammar {
                self.grammar
            }
            pub(crate) fn with_grammar(mut self, grammar: CssPropertyGrammar) -> Self {
                assert_eq!(self.property(), grammar.target_property(), "grammar and parsed value share a target");
                self.grammar = grammar;
                self
            }
            /// Returns the canonical property identity derived from the active value variant.
            #[must_use]
            pub const fn property(&self) -> CssKnownProperty {
                match &self.value {
                    CssKnownDeclarationValue::All(_) => CssKnownProperty::All,
                    $(CssKnownDeclarationValue::$variant(_) => CssKnownProperty::$variant,)*
                }
            }

            /// Returns exactly one borrowed declared-value branch.
            #[must_use]
            pub const fn declared_value(&self) -> CssKnownDeclaredValueRef<'_> {
                match &self.value {
                    CssKnownDeclarationValue::All(CssAllDeclaredValue::Global(keyword)) => {
                        CssKnownDeclaredValueRef::Global(*keyword)
                    }
                    CssKnownDeclarationValue::All(
                        CssAllDeclaredValue::SubstitutionDependent(value),
                    ) => CssKnownDeclaredValueRef::SubstitutionDependent(value),
                    $(
                        CssKnownDeclarationValue::$variant(CssDeclaredValue::Value(value)) => {
                            CssKnownDeclaredValueRef::Property(
                                CssKnownPropertyValueRef::$variant(value),
                            )
                        }
                        CssKnownDeclarationValue::$variant(CssDeclaredValue::Global(keyword)) => {
                            CssKnownDeclaredValueRef::Global(*keyword)
                        }
                        CssKnownDeclarationValue::$variant(
                            CssDeclaredValue::SubstitutionDependent(value),
                        ) => CssKnownDeclaredValueRef::SubstitutionDependent(value),
                    )*
                }
            }

            /// Returns the property-specific ordinary-value view when present.
            #[must_use]
            pub const fn property_value(&self) -> Option<CssKnownPropertyValueRef<'_>> {
                match self.declared_value() {
                    CssKnownDeclaredValueRef::Property(value) => Some(value),
                    CssKnownDeclaredValueRef::Global(_)
                    | CssKnownDeclaredValueRef::SubstitutionDependent(_) => None,
                }
            }

            /// Returns the CSS-wide keyword when present.
            #[must_use]
            pub const fn global(&self) -> Option<CssGlobalKeyword> {
                match self.declared_value() {
                    CssKnownDeclaredValueRef::Global(keyword) => Some(keyword),
                    CssKnownDeclaredValueRef::Property(_)
                    | CssKnownDeclaredValueRef::SubstitutionDependent(_) => None,
                }
            }

            /// Returns the substitution-dependent authored value when present.
            #[must_use]
            pub const fn substitution_dependent(
                &self,
            ) -> Option<&CssSubstitutionDependentValue> {
                match self.declared_value() {
                    CssKnownDeclaredValueRef::SubstitutionDependent(value) => Some(value),
                    CssKnownDeclaredValueRef::Property(_)
                    | CssKnownDeclaredValueRef::Global(_) => None,
                }
            }

            pub(crate) const fn from_value(value: CssKnownDeclarationValue) -> Self {
                Self {
                    grammar: match &value {
                        CssKnownDeclarationValue::All(_) => CssKnownProperty::All.grammar(),
                        $(CssKnownDeclarationValue::$variant(_) => CssKnownProperty::$variant.grammar(),)*
                    },
                    value,
                }
            }

            pub(crate) fn from_global(
                property: CssKnownProperty,
                keyword: CssGlobalKeyword,
            ) -> Self {
                let value = match property {
                    CssKnownProperty::All => {
                        CssKnownDeclarationValue::All(CssAllDeclaredValue::Global(keyword))
                    }
                    $(CssKnownProperty::$variant => {
                        CssKnownDeclarationValue::$variant(CssDeclaredValue::Global(keyword))
                    },)*
                };
                Self::from_value(value)
            }

            pub(crate) fn from_substitution_dependent(
                property: CssKnownProperty,
                value: CssSubstitutionDependentValue,
            ) -> Self {
                let value = match property {
                    CssKnownProperty::All => {
                        CssKnownDeclarationValue::All(
                            CssAllDeclaredValue::SubstitutionDependent(value),
                        )
                    }
                    $(CssKnownProperty::$variant => {
                        CssKnownDeclarationValue::$variant(
                            CssDeclaredValue::SubstitutionDependent(value),
                        )
                    },)*
                };
                Self::from_value(value)
            }
        }

        const KNOWN_PROPERTIES: &[CssKnownProperty] = &[
            CssKnownProperty::All,
            $(CssKnownProperty::$variant,)*
        ];

        const IMPLEMENTED_PROPERTIES: &[PropertyImplementation] = &[
            PropertyImplementation {
                known_property: CssKnownProperty::All,
                schema_variant: stringify!(All),
                name: $all_canonical,
                aliases: &[$($all_alias),*],
                stable_id: $all_stable_id,
                authored_value_type: stringify!($all_value),
                wrapper: stringify!($all_wrapper),
                representation: stringify!($all_representation),
                parser: stringify!($all_parser),
            },
            $(
                PropertyImplementation {
                    known_property: CssKnownProperty::$variant,
                    schema_variant: stringify!($variant),
                    name: $canonical,
                    aliases: &[$($alias),*],
                    stable_id: $stable_id,
                    authored_value_type: stringify!($value),
                    wrapper: stringify!($wrapper),
                    representation: stringify!($representation),
                    parser: stringify!($parser),
                },
            )*
        ];
    };
}

property_schema!(define_property_identity, schema_input, schema_numeric);

/// A reviewed authored property-name resolution outside name-equivalent schema aliases.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CssResolvedPropertyName {
    Canonical(CssKnownProperty),
    LegacyShorthand(CssLegacyPropertyAlias),
}

impl CssResolvedPropertyName {
    pub(crate) const fn property(self) -> CssKnownProperty {
        match self {
            Self::Canonical(property) => property,
            Self::LegacyShorthand(alias) => alias.target(),
        }
    }
}

/// An explicitly parsed legacy shorthand whose value maps to a canonical longhand.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CssLegacyPropertyAlias {
    GlyphOrientationVertical,
    PageBreakBefore,
    PageBreakAfter,
    PageBreakInside,
}

impl CssLegacyPropertyAlias {
    pub(crate) const fn target(self) -> CssKnownProperty {
        match self {
            Self::GlyphOrientationVertical => CssKnownProperty::TextOrientation,
            Self::PageBreakBefore => CssKnownProperty::BreakBefore,
            Self::PageBreakAfter => CssKnownProperty::BreakAfter,
            Self::PageBreakInside => CssKnownProperty::BreakInside,
        }
    }
}

pub(crate) fn resolve_property_name(name: &str) -> Option<CssResolvedPropertyName> {
    if let Some(property) = CssKnownProperty::from_name(name) {
        return Some(CssResolvedPropertyName::Canonical(property));
    }
    let alias = if name.eq_ignore_ascii_case("glyph-orientation-vertical") {
        CssLegacyPropertyAlias::GlyphOrientationVertical
    } else if name.eq_ignore_ascii_case("page-break-before") {
        CssLegacyPropertyAlias::PageBreakBefore
    } else if name.eq_ignore_ascii_case("page-break-after") {
        CssLegacyPropertyAlias::PageBreakAfter
    } else if name.eq_ignore_ascii_case("page-break-inside") {
        CssLegacyPropertyAlias::PageBreakInside
    } else {
        return None;
    };
    Some(CssResolvedPropertyName::LegacyShorthand(alias))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PropertyImplementation {
    pub(crate) known_property: CssKnownProperty,
    pub(crate) schema_variant: &'static str,
    pub(crate) name: &'static str,
    pub(crate) aliases: &'static [&'static str],
    pub(crate) stable_id: &'static str,
    pub(crate) authored_value_type: &'static str,
    pub(crate) wrapper: &'static str,
    pub(crate) representation: &'static str,
    pub(crate) parser: &'static str,
}

pub(crate) const fn property_implementation_inventory() -> &'static [PropertyImplementation] {
    IMPLEMENTED_PROPERTIES
}

/// The authored value domain of the `overflow` shorthand.
///
/// One or two parsed axis values remain property-specific syntax. This value does not compute
/// scrolling behavior, layout, or used overflow values.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssOverflowI01PropertyValue {
    /// One authored overflow value applying to both axes.
    Single(CssOverflow),
    /// Two authored overflow values preserving distinct axes.
    Pair(CssOverflowAxes),
}

/// An immutable authored grammar identity, distinct from its canonical target.
///
/// This identifies grammar dispatch, not a standards revision. The pinned source
/// catalog owns revisions. Name-equivalent aliases share an identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssPropertyGrammar {
    resolved: CssResolvedPropertyName,
}
impl CssPropertyGrammar {
    /// Looks up a decoded name without trimming whitespace or parsing escapes.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        resolve_property_name(name).map(|resolved| Self { resolved })
    }
    /// Returns the canonical spelling of this grammar.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self.resolved {
            CssResolvedPropertyName::Canonical(p) => p.canonical_name(),
            CssResolvedPropertyName::LegacyShorthand(
                CssLegacyPropertyAlias::GlyphOrientationVertical,
            ) => "glyph-orientation-vertical",
            CssResolvedPropertyName::LegacyShorthand(CssLegacyPropertyAlias::PageBreakBefore) => {
                "page-break-before"
            }
            CssResolvedPropertyName::LegacyShorthand(CssLegacyPropertyAlias::PageBreakAfter) => {
                "page-break-after"
            }
            CssResolvedPropertyName::LegacyShorthand(CssLegacyPropertyAlias::PageBreakInside) => {
                "page-break-inside"
            }
        }
    }
    /// Returns the canonical property receiving this grammar's value.
    #[must_use]
    pub const fn target_property(self) -> CssKnownProperty {
        self.resolved.property()
    }
    /// Returns support-catalog identity independently of intrinsic metadata availability.
    #[must_use]
    pub const fn feature_id(self) -> crate::CssFeatureId {
        crate::CssFeatureId::new(match self.resolved {
            CssResolvedPropertyName::Canonical(p) => p.stable_id(),
            CssResolvedPropertyName::LegacyShorthand(
                CssLegacyPropertyAlias::GlyphOrientationVertical,
            ) => "official.property-alias.glyph-orientation-vertical",
            CssResolvedPropertyName::LegacyShorthand(CssLegacyPropertyAlias::PageBreakBefore) => {
                "official.property.page-break-before"
            }
            CssResolvedPropertyName::LegacyShorthand(CssLegacyPropertyAlias::PageBreakAfter) => {
                "official.property.page-break-after"
            }
            CssResolvedPropertyName::LegacyShorthand(CssLegacyPropertyAlias::PageBreakInside) => {
                "official.property.page-break-inside"
            }
        })
    }
    /// Returns intrinsic metadata for the annotated schema slice.
    pub fn metadata(
        self,
    ) -> Result<&'static crate::CssPropertyMetadata, crate::CssPropertyMetadataError> {
        crate::expansion::grammar_metadata(self)
    }
    pub(crate) const fn resolved(self) -> CssResolvedPropertyName {
        self.resolved
    }
    pub(crate) const fn from_resolved(resolved: CssResolvedPropertyName) -> Self {
        Self { resolved }
    }
}
impl CssKnownProperty {
    /// Returns this property's canonical authored grammar.
    #[must_use]
    pub const fn grammar(self) -> CssPropertyGrammar {
        CssPropertyGrammar {
            resolved: CssResolvedPropertyName::Canonical(self),
        }
    }
    /// Returns intrinsic metadata, or an explicit capability error.
    pub fn metadata(
        self,
    ) -> Result<&'static crate::CssPropertyMetadata, crate::CssPropertyMetadataError> {
        self.grammar().metadata()
    }
    /// Returns distinct legacy shorthand grammars targeting this property.
    #[must_use]
    pub const fn legacy_shorthands(self) -> &'static [CssPropertyGrammar] {
        match self {
            Self::TextOrientation => &[CssPropertyGrammar {
                resolved: CssResolvedPropertyName::LegacyShorthand(
                    CssLegacyPropertyAlias::GlyphOrientationVertical,
                ),
            }],
            Self::BreakBefore => &[CssPropertyGrammar {
                resolved: CssResolvedPropertyName::LegacyShorthand(
                    CssLegacyPropertyAlias::PageBreakBefore,
                ),
            }],
            Self::BreakAfter => &[CssPropertyGrammar {
                resolved: CssResolvedPropertyName::LegacyShorthand(
                    CssLegacyPropertyAlias::PageBreakAfter,
                ),
            }],
            Self::BreakInside => &[CssPropertyGrammar {
                resolved: CssResolvedPropertyName::LegacyShorthand(
                    CssLegacyPropertyAlias::PageBreakInside,
                ),
            }],
            _ => &[],
        }
    }
}
