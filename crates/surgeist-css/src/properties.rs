//! Canonical identity and parser wiring for recognized CSS properties.
//!
//! The crate-private schema in this module is the single authority for the
//! frozen property set. Public identity values describe authored property names;
//! they do not apply cascade, substitute variables, or resolve authored values.

use crate::syntax::*;

macro_rules! property_schema {
    ($callback:ident, $input:ident) => {
        $callback! {
            $input;
            All, "all", [], "baseline.property.all", CssAllDeclaredValue, CssAllPropertyValue, CssAllPropertyValueRepresentation, parse_all_property, { parse_all_property($input)? };
            Display, "display", [], "baseline.property.display", CssDisplay, CssDisplayPropertyValue, CssDisplayPropertyValueRepresentation, parse_display, { parse_display($input)? };
            BoxSizing, "box-sizing", [], "baseline.property.box-sizing", CssBoxSizing, CssBoxSizingPropertyValue, CssBoxSizingPropertyValueRepresentation, parse_box_sizing, { parse_box_sizing($input)? };
            BorderCollapse, "border-collapse", [], "official.property.border-collapse", CssBorderCollapse, CssBorderCollapsePropertyValue, CssBorderCollapsePropertyValueRepresentation, parse_border_collapse, { parse_border_collapse($input)? };
            BorderSpacing, "border-spacing", [], "official.property.border-spacing", CssBorderSpacing, CssBorderSpacingPropertyValue, CssBorderSpacingPropertyValueRepresentation, parse_border_spacing, { parse_border_spacing($input)? };
            CaptionSide, "caption-side", [], "official.property.caption-side", CssCaptionSide, CssCaptionSidePropertyValue, CssCaptionSidePropertyValueRepresentation, parse_caption_side, { parse_caption_side($input)? };
            Clip, "clip", [], "official.property.clip", CssClip, CssClipPropertyValue, CssClipPropertyValueRepresentation, parse_clip, { parse_clip($input)? };
            EmptyCells, "empty-cells", [], "official.property.empty-cells", CssEmptyCells, CssEmptyCellsPropertyValue, CssEmptyCellsPropertyValueRepresentation, parse_empty_cells, { parse_empty_cells($input)? };
            Orphans, "orphans", [], "official.property.orphans", CssPageLineMinimum, CssOrphansPropertyValue, CssOrphansPropertyValueRepresentation, parse_page_line_minimum, { parse_page_line_minimum($input, "orphans")? };
            PageBreakAfter, "page-break-after", [], "official.property.page-break-after", CssPageBreak, CssPageBreakAfterPropertyValue, CssPageBreakAfterPropertyValueRepresentation, parse_page_break, { parse_page_break($input)? };
            PageBreakBefore, "page-break-before", [], "official.property.page-break-before", CssPageBreak, CssPageBreakBeforePropertyValue, CssPageBreakBeforePropertyValueRepresentation, parse_page_break, { parse_page_break($input)? };
            PageBreakInside, "page-break-inside", [], "official.property.page-break-inside", CssPageBreakInside, CssPageBreakInsidePropertyValue, CssPageBreakInsidePropertyValueRepresentation, parse_page_break_inside, { parse_page_break_inside($input)? };
            Quotes, "quotes", [], "official.property.quotes", CssQuotes, CssQuotesPropertyValue, CssQuotesPropertyValueRepresentation, parse_quotes, { parse_quotes($input)? };
            TableLayout, "table-layout", [], "official.property.table-layout", CssTableLayout, CssTableLayoutPropertyValue, CssTableLayoutPropertyValueRepresentation, parse_table_layout, { parse_table_layout($input)? };
            Widows, "widows", [], "official.property.widows", CssPageLineMinimum, CssWidowsPropertyValue, CssWidowsPropertyValueRepresentation, parse_page_line_minimum, { parse_page_line_minimum($input, "widows")? };
            WordSpacing, "word-spacing", [], "official.property.word-spacing", CssWordSpacing, CssWordSpacingPropertyValue, CssWordSpacingPropertyValueRepresentation, parse_word_spacing, { parse_word_spacing($input)? };
            Position, "position", [], "baseline.property.position", CssLayoutPosition, CssPositionPropertyValue, CssPositionPropertyValueRepresentation, parse_position, { parse_position($input)? };
            Direction, "direction", [], "baseline.property.direction", CssDirection, CssDirectionPropertyValue, CssDirectionPropertyValueRepresentation, parse_direction, { parse_direction($input)? };
            Overflow, "overflow", [], "baseline.property.overflow", CssOverflowI01PropertyValue, CssOverflowPropertyValue, CssOverflowPropertyValueRepresentation, parse_overflow_property, { parse_overflow_property($input)? };
            OverflowX, "overflow-x", [], "baseline.property.overflow-x", CssOverflow, CssOverflowXPropertyValue, CssOverflowXPropertyValueRepresentation, parse_overflow, { parse_overflow($input)? };
            OverflowY, "overflow-y", [], "baseline.property.overflow-y", CssOverflow, CssOverflowYPropertyValue, CssOverflowYPropertyValueRepresentation, parse_overflow, { parse_overflow($input)? };
            FlexDirection, "flex-direction", [], "baseline.property.flex-direction", CssFlexDirection, CssFlexDirectionPropertyValue, CssFlexDirectionPropertyValueRepresentation, parse_flex_direction, { parse_flex_direction($input)? };
            FlexFlow, "flex-flow", [], "official.property.flex-flow", CssFlexFlow, CssFlexFlowPropertyValue, CssFlexFlowPropertyValueRepresentation, parse_flex_flow, { parse_flex_flow($input)? };
            FlexWrap, "flex-wrap", [], "baseline.property.flex-wrap", CssFlexWrap, CssFlexWrapPropertyValue, CssFlexWrapPropertyValueRepresentation, parse_flex_wrap, { parse_flex_wrap($input)? };
            Float, "float", [], "baseline.property.float", CssFloat, CssFloatPropertyValue, CssFloatPropertyValueRepresentation, parse_float, { parse_float($input)? };
            Clear, "clear", [], "baseline.property.clear", CssClear, CssClearPropertyValue, CssClearPropertyValueRepresentation, parse_clear, { parse_clear($input)? };
            AlignContent, "align-content", [], "baseline.property.align-content", CssAlignment, CssAlignContentPropertyValue, CssAlignContentPropertyValueRepresentation, parse_content_alignment, { parse_content_alignment($input)? };
            JustifyContent, "justify-content", [], "baseline.property.justify-content", CssAlignment, CssJustifyContentPropertyValue, CssJustifyContentPropertyValueRepresentation, parse_content_alignment, { parse_content_alignment($input)? };
            AlignItems, "align-items", [], "baseline.property.align-items", CssAlignItems, CssAlignItemsPropertyValue, CssAlignItemsPropertyValueRepresentation, parse_align_items, { parse_align_items($input)? };
            AlignSelf, "align-self", [], "baseline.property.align-self", CssAlignItems, CssAlignSelfPropertyValue, CssAlignSelfPropertyValueRepresentation, parse_align_items, { parse_align_items($input)? };
            JustifyItems, "justify-items", [], "baseline.property.justify-items", CssAlignItems, CssJustifyItemsPropertyValue, CssJustifyItemsPropertyValueRepresentation, parse_align_items, { parse_align_items($input)? };
            JustifySelf, "justify-self", [], "baseline.property.justify-self", CssAlignItems, CssJustifySelfPropertyValue, CssJustifySelfPropertyValueRepresentation, parse_align_items, { parse_align_items($input)? };
            PlaceContent, "place-content", [], "baseline.property.place-content", CssPlaceAlignment, CssPlaceContentPropertyValue, CssPlaceContentPropertyValueRepresentation, parse_place_alignment, { parse_place_alignment($input, parse_content_alignment, CssPlaceAlignment::content)? };
            PlaceItems, "place-items", [], "baseline.property.place-items", CssPlaceAlignment, CssPlaceItemsPropertyValue, CssPlaceItemsPropertyValueRepresentation, parse_place_alignment, { parse_place_alignment($input, parse_align_items, CssPlaceAlignment::items)? };
            PlaceSelf, "place-self", [], "baseline.property.place-self", CssPlaceAlignment, CssPlaceSelfPropertyValue, CssPlaceSelfPropertyValueRepresentation, parse_place_alignment, { parse_place_alignment($input, parse_align_items, CssPlaceAlignment::items)? };
            Visibility, "visibility", [], "baseline.property.visibility", CssVisibility, CssVisibilityPropertyValue, CssVisibilityPropertyValueRepresentation, parse_visibility, { parse_visibility($input)? };
            Content, "content", [], "baseline.property.content", CssContent, CssContentPropertyValue, CssContentPropertyValueRepresentation, parse_content, { parse_content($input)? };
            ContentVisibility, "content-visibility", [], "baseline.property.content-visibility", CssContentVisibility, CssContentVisibilityPropertyValue, CssContentVisibilityPropertyValueRepresentation, parse_content_visibility, { parse_content_visibility($input)? };
            ListStyleType, "list-style-type", [], "baseline.property.list-style-type", CssListStyleType, CssListStyleTypePropertyValue, CssListStyleTypePropertyValueRepresentation, parse_list_style_type, { parse_list_style_type($input)? };
            ListStylePosition, "list-style-position", [], "baseline.property.list-style-position", CssListStylePosition, CssListStylePositionPropertyValue, CssListStylePositionPropertyValueRepresentation, parse_list_style_position, { parse_list_style_position($input)? };
            ListStyleImage, "list-style-image", [], "baseline.property.list-style-image", CssListStyleImage, CssListStyleImagePropertyValue, CssListStyleImagePropertyValueRepresentation, parse_list_style_image, { parse_list_style_image($input)? };
            ListStyle, "list-style", [], "baseline.property.list-style", CssListStyle, CssListStylePropertyValue, CssListStylePropertyValueRepresentation, parse_list_style, { parse_list_style($input)? };
            CounterReset, "counter-reset", [], "baseline.property.counter-reset", CssCounterChanges, CssCounterResetPropertyValue, CssCounterResetPropertyValueRepresentation, parse_counter_changes, { parse_counter_changes($input)? };
            CounterIncrement, "counter-increment", [], "baseline.property.counter-increment", CssCounterChanges, CssCounterIncrementPropertyValue, CssCounterIncrementPropertyValueRepresentation, parse_counter_changes, { parse_counter_changes($input)? };
            CounterSet, "counter-set", [], "baseline.property.counter-set", CssCounterChanges, CssCounterSetPropertyValue, CssCounterSetPropertyValueRepresentation, parse_counter_changes, { parse_counter_changes($input)? };
            Width, "width", [], "baseline.property.width", CssLength, CssWidthPropertyValue, CssWidthPropertyValueRepresentation, parse_box_size_value, { parse_box_size_value($input)? };
            Height, "height", [], "baseline.property.height", CssLength, CssHeightPropertyValue, CssHeightPropertyValueRepresentation, parse_box_size_value, { parse_box_size_value($input)? };
            MinWidth, "min-width", [], "baseline.property.min-width", CssLength, CssMinWidthPropertyValue, CssMinWidthPropertyValueRepresentation, parse_box_size_value, { parse_box_size_value($input)? };
            MinHeight, "min-height", [], "baseline.property.min-height", CssLength, CssMinHeightPropertyValue, CssMinHeightPropertyValueRepresentation, parse_box_size_value, { parse_box_size_value($input)? };
            MaxWidth, "max-width", [], "baseline.property.max-width", CssLength, CssMaxWidthPropertyValue, CssMaxWidthPropertyValueRepresentation, parse_box_size_value, { parse_box_size_value($input)? };
            MaxHeight, "max-height", [], "baseline.property.max-height", CssLength, CssMaxHeightPropertyValue, CssMaxHeightPropertyValueRepresentation, parse_box_size_value, { parse_box_size_value($input)? };
            FlexBasis, "flex-basis", [], "baseline.property.flex-basis", CssLength, CssFlexBasisPropertyValue, CssFlexBasisPropertyValueRepresentation, parse_box_size_value, { parse_box_size_value($input)? };
            Gap, "gap", [], "baseline.property.gap", CssLength, CssGapPropertyValue, CssGapPropertyValueRepresentation, parse_gap_value, { parse_gap_value($input)? };
            RowGap, "row-gap", [], "baseline.property.row-gap", CssLength, CssRowGapPropertyValue, CssRowGapPropertyValueRepresentation, parse_gap_value, { parse_gap_value($input)? };
            ColumnGap, "column-gap", [], "baseline.property.column-gap", CssLength, CssColumnGapPropertyValue, CssColumnGapPropertyValueRepresentation, parse_gap_value, { parse_gap_value($input)? };
            ColumnCount, "column-count", [], "official.property.column-count", CssColumnCount, CssColumnCountPropertyValue, CssColumnCountPropertyValueRepresentation, parse_column_count, { parse_column_count($input)? };
            ColumnFill, "column-fill", [], "official.property.column-fill", CssColumnFill, CssColumnFillPropertyValue, CssColumnFillPropertyValueRepresentation, parse_column_fill, { parse_column_fill($input)? };
            ColumnRule, "column-rule", [], "official.property.column-rule", CssColumnRule, CssColumnRulePropertyValue, CssColumnRulePropertyValueRepresentation, parse_column_rule, { parse_column_rule($input)? };
            ColumnRuleColor, "column-rule-color", [], "official.property.column-rule-color", CssParsedColor, CssColumnRuleColorPropertyValue, CssColumnRuleColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            ColumnRuleStyle, "column-rule-style", [], "official.property.column-rule-style", CssLineStyle, CssColumnRuleStylePropertyValue, CssColumnRuleStylePropertyValueRepresentation, parse_line_style, { parse_line_style($input)? };
            ColumnRuleWidth, "column-rule-width", [], "official.property.column-rule-width", CssLineWidth, CssColumnRuleWidthPropertyValue, CssColumnRuleWidthPropertyValueRepresentation, parse_line_width, { parse_line_width($input)? };
            ColumnSpan, "column-span", [], "official.property.column-span", CssColumnSpan, CssColumnSpanPropertyValue, CssColumnSpanPropertyValueRepresentation, parse_column_span, { parse_column_span($input)? };
            ColumnWidth, "column-width", [], "official.property.column-width", CssColumnWidth, CssColumnWidthPropertyValue, CssColumnWidthPropertyValueRepresentation, parse_column_width, { parse_column_width($input)? };
            Columns, "columns", [], "official.property.columns", CssColumns, CssColumnsPropertyValue, CssColumnsPropertyValueRepresentation, parse_columns, { parse_columns($input)? };
            GridFlowTolerance, "grid-flow-tolerance", [], "baseline.property.grid-flow-tolerance", CssGridFlowTolerance, CssGridFlowTolerancePropertyValue, CssGridFlowTolerancePropertyValueRepresentation, parse_grid_flow_tolerance, { parse_grid_flow_tolerance($input)? };
            GridTemplateRows, "grid-template-rows", [], "baseline.property.grid-template-rows", CssGridTrackList, CssGridTemplateRowsPropertyValue, CssGridTemplateRowsPropertyValueRepresentation, parse_grid_track_list, { parse_grid_track_list($input)? };
            GridTemplateColumns, "grid-template-columns", [], "baseline.property.grid-template-columns", CssGridTrackList, CssGridTemplateColumnsPropertyValue, CssGridTemplateColumnsPropertyValueRepresentation, parse_grid_track_list, { parse_grid_track_list($input)? };
            GridTemplateAreas, "grid-template-areas", [], "baseline.property.grid-template-areas", CssGridTemplateAreas, CssGridTemplateAreasPropertyValue, CssGridTemplateAreasPropertyValueRepresentation, parse_grid_template_areas, { parse_grid_template_areas($input)? };
            GridTemplate, "grid-template", [], "baseline.property.grid-template", CssGridTemplate, CssGridTemplatePropertyValue, CssGridTemplatePropertyValueRepresentation, parse_grid_template, { parse_grid_template($input)? };
            GridAutoRows, "grid-auto-rows", [], "baseline.property.grid-auto-rows", CssGridTrackList, CssGridAutoRowsPropertyValue, CssGridAutoRowsPropertyValueRepresentation, parse_grid_auto_track_sizes, { parse_grid_auto_track_sizes($input)? };
            GridAutoColumns, "grid-auto-columns", [], "baseline.property.grid-auto-columns", CssGridTrackList, CssGridAutoColumnsPropertyValue, CssGridAutoColumnsPropertyValueRepresentation, parse_grid_auto_track_sizes, { parse_grid_auto_track_sizes($input)? };
            GridAutoFlow, "grid-auto-flow", [], "baseline.property.grid-auto-flow", CssGridAutoFlow, CssGridAutoFlowPropertyValue, CssGridAutoFlowPropertyValueRepresentation, parse_grid_auto_flow, { parse_grid_auto_flow($input)? };
            GridRowStart, "grid-row-start", [], "baseline.property.grid-row-start", CssGridLine, CssGridRowStartPropertyValue, CssGridRowStartPropertyValueRepresentation, parse_grid_line, { parse_grid_line($input)? };
            GridRowEnd, "grid-row-end", [], "baseline.property.grid-row-end", CssGridLine, CssGridRowEndPropertyValue, CssGridRowEndPropertyValueRepresentation, parse_grid_line, { parse_grid_line($input)? };
            GridColumnStart, "grid-column-start", [], "baseline.property.grid-column-start", CssGridLine, CssGridColumnStartPropertyValue, CssGridColumnStartPropertyValueRepresentation, parse_grid_line, { parse_grid_line($input)? };
            GridColumnEnd, "grid-column-end", [], "baseline.property.grid-column-end", CssGridLine, CssGridColumnEndPropertyValue, CssGridColumnEndPropertyValueRepresentation, parse_grid_line, { parse_grid_line($input)? };
            GridRow, "grid-row", [], "baseline.property.grid-row", CssGridLineRange, CssGridRowPropertyValue, CssGridRowPropertyValueRepresentation, parse_grid_line_range, { parse_grid_line_range($input)? };
            GridColumn, "grid-column", [], "baseline.property.grid-column", CssGridLineRange, CssGridColumnPropertyValue, CssGridColumnPropertyValueRepresentation, parse_grid_line_range, { parse_grid_line_range($input)? };
            GridArea, "grid-area", [], "baseline.property.grid-area", CssGridArea, CssGridAreaPropertyValue, CssGridAreaPropertyValueRepresentation, parse_grid_area, { parse_grid_area($input)? };
            Grid, "grid", [], "baseline.property.grid", CssGrid, CssGridPropertyValue, CssGridPropertyValueRepresentation, parse_grid, { parse_grid($input)? };
            FontSize, "font-size", [], "baseline.property.font-size", CssFontSize, CssFontSizePropertyValue, CssFontSizePropertyValueRepresentation, parse_font_size, { parse_font_size($input)? };
            LineHeight, "line-height", [], "baseline.property.line-height", CssLineHeight, CssLineHeightPropertyValue, CssLineHeightPropertyValueRepresentation, parse_line_height, { parse_line_height($input)? };
            TextCombineUpright, "text-combine-upright", [], "official.property.text-combine-upright", CssTextCombineUpright, CssTextCombineUprightPropertyValue, CssTextCombineUprightPropertyValueRepresentation, parse_text_combine_upright, { parse_text_combine_upright($input)? };
            TextOrientation, "text-orientation", [], "official.property.text-orientation", CssTextOrientation, CssTextOrientationPropertyValue, CssTextOrientationPropertyValueRepresentation, parse_text_orientation, { parse_text_orientation($input)? };
            UnicodeBidi, "unicode-bidi", [], "official.property.unicode-bidi", CssUnicodeBidi, CssUnicodeBidiPropertyValue, CssUnicodeBidiPropertyValueRepresentation, parse_unicode_bidi, { parse_unicode_bidi($input)? };
            WritingMode, "writing-mode", [], "baseline.property.writing-mode", CssWritingMode, CssWritingModePropertyValue, CssWritingModePropertyValueRepresentation, parse_writing_mode, { parse_writing_mode($input)? };
            TextAlign, "text-align", [], "baseline.property.text-align", CssTextAlign, CssTextAlignPropertyValue, CssTextAlignPropertyValueRepresentation, parse_text_align, { parse_text_align($input)? };
            TextAlignLast, "text-align-last", [], "baseline.property.text-align-last", CssTextAlignLast, CssTextAlignLastPropertyValue, CssTextAlignLastPropertyValueRepresentation, parse_text_align_last, { parse_text_align_last($input)? };
            TextIndent, "text-indent", [], "baseline.property.text-indent", CssTextIndent, CssTextIndentPropertyValue, CssTextIndentPropertyValueRepresentation, parse_text_indent, { parse_text_indent($input)? };
            VerticalAlign, "vertical-align", [], "baseline.property.vertical-align", CssVerticalAlign, CssVerticalAlignPropertyValue, CssVerticalAlignPropertyValueRepresentation, parse_vertical_align, { parse_vertical_align($input)? };
            FontFamily, "font-family", [], "baseline.property.font-family", CssFontFamilyList, CssFontFamilyPropertyValue, CssFontFamilyPropertyValueRepresentation, parse_font_family_list, { parse_font_family_list($input)? };
            Font, "font", [], "baseline.property.font", CssFontValue, CssFontPropertyValue, CssFontPropertyValueRepresentation, parse_font, { parse_font($input)? };
            FontWeight, "font-weight", [], "baseline.property.font-weight", CssFontWeight, CssFontWeightPropertyValue, CssFontWeightPropertyValueRepresentation, parse_font_weight, { parse_font_weight($input)? };
            FontStyle, "font-style", [], "baseline.property.font-style", CssFontStyle, CssFontStylePropertyValue, CssFontStylePropertyValueRepresentation, parse_font_style, { parse_font_style($input)? };
            FontStretch, "font-stretch", [], "baseline.property.font-stretch", CssFontStretch, CssFontStretchPropertyValue, CssFontStretchPropertyValueRepresentation, parse_font_stretch, { parse_font_stretch($input)? };
            FontVariant, "font-variant", [], "baseline.property.font-variant", CssFontVariantValue, CssFontVariantPropertyValue, CssFontVariantPropertyValueRepresentation, parse_font_variant, { parse_font_variant($input)? };
            FontVariantCaps, "font-variant-caps", [], "official.property.font-variant-caps", CssFontVariantCaps, CssFontVariantCapsPropertyValue, CssFontVariantCapsPropertyValueRepresentation, parse_font_variant_caps, { parse_font_variant_caps($input)? };
            FontVariantEastAsian, "font-variant-east-asian", [], "official.property.font-variant-east-asian", CssFontVariantEastAsian, CssFontVariantEastAsianPropertyValue, CssFontVariantEastAsianPropertyValueRepresentation, parse_font_variant_east_asian, { parse_font_variant_east_asian($input)? };
            FontVariantLigatures, "font-variant-ligatures", [], "official.property.font-variant-ligatures", CssFontVariantLigatures, CssFontVariantLigaturesPropertyValue, CssFontVariantLigaturesPropertyValueRepresentation, parse_font_variant_ligatures, { parse_font_variant_ligatures($input)? };
            FontVariantNumeric, "font-variant-numeric", [], "official.property.font-variant-numeric", CssFontVariantNumeric, CssFontVariantNumericPropertyValue, CssFontVariantNumericPropertyValueRepresentation, parse_font_variant_numeric, { parse_font_variant_numeric($input)? };
            FontVariantPosition, "font-variant-position", [], "official.property.font-variant-position", CssFontVariantPosition, CssFontVariantPositionPropertyValue, CssFontVariantPositionPropertyValueRepresentation, parse_font_variant_position, { parse_font_variant_position($input)? };
            FontFeatureSettings, "font-feature-settings", [], "baseline.property.font-feature-settings", CssFontFeatureSettings, CssFontFeatureSettingsPropertyValue, CssFontFeatureSettingsPropertyValueRepresentation, parse_font_feature_settings, { parse_font_feature_settings($input)? };
            FontKerning, "font-kerning", [], "official.property.font-kerning", CssFontKerning, CssFontKerningPropertyValue, CssFontKerningPropertyValueRepresentation, parse_font_kerning, { parse_font_kerning($input)? };
            FontSizeAdjust, "font-size-adjust", [], "official.property.font-size-adjust", CssFontSizeAdjust, CssFontSizeAdjustPropertyValue, CssFontSizeAdjustPropertyValueRepresentation, parse_font_size_adjust, { parse_font_size_adjust($input)? };
            FontSynthesis, "font-synthesis", [], "official.property.font-synthesis", CssFontSynthesis, CssFontSynthesisPropertyValue, CssFontSynthesisPropertyValueRepresentation, parse_font_synthesis, { parse_font_synthesis($input)? };
            LetterSpacing, "letter-spacing", [], "baseline.property.letter-spacing", CssLetterSpacing, CssLetterSpacingPropertyValue, CssLetterSpacingPropertyValueRepresentation, parse_letter_spacing, { parse_letter_spacing($input)? };
            TextWrap, "text-wrap", [], "baseline.property.text-wrap", CssTextWrap, CssTextWrapPropertyValue, CssTextWrapPropertyValueRepresentation, parse_text_wrap, { parse_text_wrap($input)? };
            WhiteSpace, "white-space", [], "baseline.property.white-space", CssWhiteSpace, CssWhiteSpacePropertyValue, CssWhiteSpacePropertyValueRepresentation, parse_white_space, { parse_white_space($input)? };
            WordBreak, "word-break", [], "baseline.property.word-break", CssWordBreak, CssWordBreakPropertyValue, CssWordBreakPropertyValueRepresentation, parse_word_break, { parse_word_break($input)? };
            OverflowWrap, "overflow-wrap", [], "baseline.property.overflow-wrap", CssOverflowWrap, CssOverflowWrapPropertyValue, CssOverflowWrapPropertyValueRepresentation, parse_overflow_wrap, { parse_overflow_wrap($input)? };
            TextOverflow, "text-overflow", [], "baseline.property.text-overflow", CssTextOverflow, CssTextOverflowPropertyValue, CssTextOverflowPropertyValueRepresentation, parse_text_overflow, { parse_text_overflow($input)? };
            TextDecoration, "text-decoration", [], "baseline.property.text-decoration", CssTextDecoration, CssTextDecorationPropertyValue, CssTextDecorationPropertyValueRepresentation, parse_text_decoration, { parse_text_decoration($input)? };
            TextDecorationLine, "text-decoration-line", [], "baseline.property.text-decoration-line", CssTextDecorationLine, CssTextDecorationLinePropertyValue, CssTextDecorationLinePropertyValueRepresentation, parse_text_decoration_line, { parse_text_decoration_line($input)? };
            TextDecorationColor, "text-decoration-color", [], "baseline.property.text-decoration-color", CssColor, CssTextDecorationColorPropertyValue, CssTextDecorationColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            TextDecorationStyle, "text-decoration-style", [], "baseline.property.text-decoration-style", CssTextDecorationStyle, CssTextDecorationStylePropertyValue, CssTextDecorationStylePropertyValueRepresentation, parse_text_decoration_style, { parse_text_decoration_style($input)? };
            TextDecorationThickness, "text-decoration-thickness", [], "baseline.property.text-decoration-thickness", CssTextDecorationThickness, CssTextDecorationThicknessPropertyValue, CssTextDecorationThicknessPropertyValueRepresentation, parse_text_decoration_thickness, { parse_text_decoration_thickness($input)? };
            TextTransform, "text-transform", [], "baseline.property.text-transform", CssTextTransform, CssTextTransformPropertyValue, CssTextTransformPropertyValueRepresentation, parse_text_transform, { parse_text_transform($input)? };
            Inset, "inset", [], "baseline.property.inset", CssEdges, CssInsetPropertyValue, CssInsetPropertyValueRepresentation, parse_edges, { parse_edges($input, parse_inset_component)? };
            Top, "top", [], "baseline.property.top", CssLength, CssTopPropertyValue, CssTopPropertyValueRepresentation, parse_inset_component, { parse_inset_component($input)? };
            Right, "right", [], "baseline.property.right", CssLength, CssRightPropertyValue, CssRightPropertyValueRepresentation, parse_inset_component, { parse_inset_component($input)? };
            Bottom, "bottom", [], "baseline.property.bottom", CssLength, CssBottomPropertyValue, CssBottomPropertyValueRepresentation, parse_inset_component, { parse_inset_component($input)? };
            Left, "left", [], "baseline.property.left", CssLength, CssLeftPropertyValue, CssLeftPropertyValueRepresentation, parse_inset_component, { parse_inset_component($input)? };
            ZIndex, "z-index", [], "baseline.property.z-index", CssZIndex, CssZIndexPropertyValue, CssZIndexPropertyValueRepresentation, parse_z_index, { parse_z_index($input)? };
            BoxDecorationBreak, "box-decoration-break", [], "baseline.property.box-decoration-break", CssBoxDecorationBreak, CssBoxDecorationBreakPropertyValue, CssBoxDecorationBreakPropertyValueRepresentation, parse_box_decoration_break, { parse_box_decoration_break($input)? };
            Margin, "margin", [], "baseline.property.margin", CssEdges, CssMarginPropertyValue, CssMarginPropertyValueRepresentation, parse_edges, { parse_edges($input, parse_margin_component)? };
            MarginTop, "margin-top", [], "baseline.property.margin-top", CssLength, CssMarginTopPropertyValue, CssMarginTopPropertyValueRepresentation, parse_margin_component, { parse_margin_component($input)? };
            MarginRight, "margin-right", [], "baseline.property.margin-right", CssLength, CssMarginRightPropertyValue, CssMarginRightPropertyValueRepresentation, parse_margin_component, { parse_margin_component($input)? };
            MarginBottom, "margin-bottom", [], "baseline.property.margin-bottom", CssLength, CssMarginBottomPropertyValue, CssMarginBottomPropertyValueRepresentation, parse_margin_component, { parse_margin_component($input)? };
            MarginLeft, "margin-left", [], "baseline.property.margin-left", CssLength, CssMarginLeftPropertyValue, CssMarginLeftPropertyValueRepresentation, parse_margin_component, { parse_margin_component($input)? };
            Padding, "padding", [], "baseline.property.padding", CssEdges, CssPaddingPropertyValue, CssPaddingPropertyValueRepresentation, parse_edges, { parse_edges($input, parse_padding_component)? };
            PaddingTop, "padding-top", [], "baseline.property.padding-top", CssLength, CssPaddingTopPropertyValue, CssPaddingTopPropertyValueRepresentation, parse_padding_component, { parse_padding_component($input)? };
            PaddingRight, "padding-right", [], "baseline.property.padding-right", CssLength, CssPaddingRightPropertyValue, CssPaddingRightPropertyValueRepresentation, parse_padding_component, { parse_padding_component($input)? };
            PaddingBottom, "padding-bottom", [], "baseline.property.padding-bottom", CssLength, CssPaddingBottomPropertyValue, CssPaddingBottomPropertyValueRepresentation, parse_padding_component, { parse_padding_component($input)? };
            PaddingLeft, "padding-left", [], "baseline.property.padding-left", CssLength, CssPaddingLeftPropertyValue, CssPaddingLeftPropertyValueRepresentation, parse_padding_component, { parse_padding_component($input)? };
            Border, "border", [], "baseline.property.border", CssBorder, CssBorderPropertyValue, CssBorderPropertyValueRepresentation, parse_border, { parse_border($input)? };
            BorderTop, "border-top", [], "baseline.property.border-top", CssBorder, CssBorderTopPropertyValue, CssBorderTopPropertyValueRepresentation, parse_border, { parse_border($input)? };
            BorderRight, "border-right", [], "baseline.property.border-right", CssBorder, CssBorderRightPropertyValue, CssBorderRightPropertyValueRepresentation, parse_border, { parse_border($input)? };
            BorderBottom, "border-bottom", [], "baseline.property.border-bottom", CssBorder, CssBorderBottomPropertyValue, CssBorderBottomPropertyValueRepresentation, parse_border, { parse_border($input)? };
            BorderLeft, "border-left", [], "baseline.property.border-left", CssBorder, CssBorderLeftPropertyValue, CssBorderLeftPropertyValueRepresentation, parse_border, { parse_border($input)? };
            BorderWidth, "border-width", [], "baseline.property.border-width", CssEdges, CssBorderWidthPropertyValue, CssBorderWidthPropertyValueRepresentation, parse_edges, { parse_edges($input, parse_border_width_component)? };
            BorderTopWidth, "border-top-width", [], "baseline.property.border-top-width", CssLength, CssBorderTopWidthPropertyValue, CssBorderTopWidthPropertyValueRepresentation, parse_border_width_component, { parse_border_width_component($input)? };
            BorderRightWidth, "border-right-width", [], "baseline.property.border-right-width", CssLength, CssBorderRightWidthPropertyValue, CssBorderRightWidthPropertyValueRepresentation, parse_border_width_component, { parse_border_width_component($input)? };
            BorderBottomWidth, "border-bottom-width", [], "baseline.property.border-bottom-width", CssLength, CssBorderBottomWidthPropertyValue, CssBorderBottomWidthPropertyValueRepresentation, parse_border_width_component, { parse_border_width_component($input)? };
            BorderLeftWidth, "border-left-width", [], "baseline.property.border-left-width", CssLength, CssBorderLeftWidthPropertyValue, CssBorderLeftWidthPropertyValueRepresentation, parse_border_width_component, { parse_border_width_component($input)? };
            Color, "color", [], "baseline.property.color", CssColor, CssColorPropertyValue, CssColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            Background, "background", [], "baseline.property.background", CssColor, CssBackgroundPropertyValue, CssBackgroundPropertyValueRepresentation, parse_background, { parse_background($input)? };
            BackgroundColor, "background-color", [], "baseline.property.background-color", CssColor, CssBackgroundColorPropertyValue, CssBackgroundColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            BorderColor, "border-color", [], "baseline.property.border-color", CssColor, CssBorderColorPropertyValue, CssBorderColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            BorderTopColor, "border-top-color", [], "baseline.property.border-top-color", CssColor, CssBorderTopColorPropertyValue, CssBorderTopColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            BorderRightColor, "border-right-color", [], "baseline.property.border-right-color", CssColor, CssBorderRightColorPropertyValue, CssBorderRightColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            BorderBottomColor, "border-bottom-color", [], "baseline.property.border-bottom-color", CssColor, CssBorderBottomColorPropertyValue, CssBorderBottomColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            BorderLeftColor, "border-left-color", [], "baseline.property.border-left-color", CssColor, CssBorderLeftColorPropertyValue, CssBorderLeftColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            BackgroundImage, "background-image", [], "baseline.property.background-image", CssImageLayerList, CssBackgroundImagePropertyValue, CssBackgroundImagePropertyValueRepresentation, parse_image_layer_list, { parse_image_layer_list($input)? };
            BackgroundPosition, "background-position", [], "baseline.property.background-position", CssBackgroundPositionList, CssBackgroundPositionPropertyValue, CssBackgroundPositionPropertyValueRepresentation, parse_background_position_list, { parse_background_position_list($input)? };
            ObjectPosition, "object-position", [], "official.property.object-position", CssObjectPosition, CssObjectPositionPropertyValue, CssObjectPositionPropertyValueRepresentation, parse_object_position, { parse_object_position($input)? };
            BackgroundSize, "background-size", [], "baseline.property.background-size", CssBackgroundSizeList, CssBackgroundSizePropertyValue, CssBackgroundSizePropertyValueRepresentation, parse_background_size_list, { parse_background_size_list($input)? };
            BackgroundRepeat, "background-repeat", [], "baseline.property.background-repeat", CssBackgroundRepeatList, CssBackgroundRepeatPropertyValue, CssBackgroundRepeatPropertyValueRepresentation, parse_background_repeat_list, { parse_background_repeat_list($input)? };
            BackgroundOrigin, "background-origin", [], "baseline.property.background-origin", CssBackgroundBox, CssBackgroundOriginPropertyValue, CssBackgroundOriginPropertyValueRepresentation, parse_background_box_list, { parse_background_box_list($input)? };
            BackgroundClip, "background-clip", [], "baseline.property.background-clip", CssBackgroundBox, CssBackgroundClipPropertyValue, CssBackgroundClipPropertyValueRepresentation, parse_background_box_list, { parse_background_box_list($input)? };
            BackgroundAttachment, "background-attachment", [], "baseline.property.background-attachment", CssBackgroundAttachmentList, CssBackgroundAttachmentPropertyValue, CssBackgroundAttachmentPropertyValueRepresentation, parse_background_attachment_list, { parse_background_attachment_list($input)? };
            BorderImage, "border-image", [], "official.property.border-image", CssBorderImage, CssBorderImagePropertyValue, CssBorderImagePropertyValueRepresentation, parse_border_image, { parse_border_image($input)? };
            BorderImageOutset, "border-image-outset", [], "official.property.border-image-outset", CssBorderImageOutset, CssBorderImageOutsetPropertyValue, CssBorderImageOutsetPropertyValueRepresentation, parse_border_image_outset, { parse_border_image_outset($input)? };
            BorderImageRepeat, "border-image-repeat", [], "official.property.border-image-repeat", CssBorderImageRepeat, CssBorderImageRepeatPropertyValue, CssBorderImageRepeatPropertyValueRepresentation, parse_border_image_repeat, { parse_border_image_repeat($input)? };
            BorderImageSlice, "border-image-slice", [], "official.property.border-image-slice", CssBorderImageSlice, CssBorderImageSlicePropertyValue, CssBorderImageSlicePropertyValueRepresentation, parse_border_image_slice, { parse_border_image_slice($input)? };
            BorderImageSource, "border-image-source", [], "official.property.border-image-source", CssImageValue, CssBorderImageSourcePropertyValue, CssBorderImageSourcePropertyValueRepresentation, parse_border_image_source, { parse_border_image_source($input)? };
            BorderImageWidth, "border-image-width", [], "official.property.border-image-width", CssBorderImageWidth, CssBorderImageWidthPropertyValue, CssBorderImageWidthPropertyValueRepresentation, parse_border_image_width, { parse_border_image_width($input)? };
            ImageOrientation, "image-orientation", [], "official.property.image-orientation", CssImageOrientation, CssImageOrientationPropertyValue, CssImageOrientationPropertyValueRepresentation, parse_image_orientation, { parse_image_orientation($input)? };
            ImageRendering, "image-rendering", [], "official.property.image-rendering", CssImageRendering, CssImageRenderingPropertyValue, CssImageRenderingPropertyValueRepresentation, parse_image_rendering, { parse_image_rendering($input)? };
            ObjectFit, "object-fit", [], "official.property.object-fit", CssObjectFit, CssObjectFitPropertyValue, CssObjectFitPropertyValueRepresentation, parse_object_fit, { parse_object_fit($input)? };
            BorderStyle, "border-style", [], "baseline.property.border-style", CssBorderStyles, CssBorderStylePropertyValue, CssBorderStylePropertyValueRepresentation, parse_border_styles, { parse_border_styles($input)? };
            BorderTopStyle, "border-top-style", [], "baseline.property.border-top-style", CssBorderStyle, CssBorderTopStylePropertyValue, CssBorderTopStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? };
            BorderRightStyle, "border-right-style", [], "baseline.property.border-right-style", CssBorderStyle, CssBorderRightStylePropertyValue, CssBorderRightStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? };
            BorderBottomStyle, "border-bottom-style", [], "baseline.property.border-bottom-style", CssBorderStyle, CssBorderBottomStylePropertyValue, CssBorderBottomStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? };
            BorderLeftStyle, "border-left-style", [], "baseline.property.border-left-style", CssBorderStyle, CssBorderLeftStylePropertyValue, CssBorderLeftStylePropertyValueRepresentation, parse_border_style, { parse_border_style($input)? };
            BorderRadius, "border-radius", [], "baseline.property.border-radius", CssBorderRadii, CssBorderRadiusPropertyValue, CssBorderRadiusPropertyValueRepresentation, parse_border_radius, { parse_border_radius($input)? };
            BorderTopLeftRadius, "border-top-left-radius", [], "baseline.property.border-top-left-radius", CssCornerRadius, CssBorderTopLeftRadiusPropertyValue, CssBorderTopLeftRadiusPropertyValueRepresentation, parse_corner_radius, { parse_corner_radius($input)? };
            BorderTopRightRadius, "border-top-right-radius", [], "baseline.property.border-top-right-radius", CssCornerRadius, CssBorderTopRightRadiusPropertyValue, CssBorderTopRightRadiusPropertyValueRepresentation, parse_corner_radius, { parse_corner_radius($input)? };
            BorderBottomRightRadius, "border-bottom-right-radius", [], "baseline.property.border-bottom-right-radius", CssCornerRadius, CssBorderBottomRightRadiusPropertyValue, CssBorderBottomRightRadiusPropertyValueRepresentation, parse_corner_radius, { parse_corner_radius($input)? };
            BorderBottomLeftRadius, "border-bottom-left-radius", [], "baseline.property.border-bottom-left-radius", CssCornerRadius, CssBorderBottomLeftRadiusPropertyValue, CssBorderBottomLeftRadiusPropertyValueRepresentation, parse_corner_radius, { parse_corner_radius($input)? };
            BoxShadow, "box-shadow", [], "baseline.property.box-shadow", CssBoxShadow, CssBoxShadowPropertyValue, CssBoxShadowPropertyValueRepresentation, parse_box_shadow, { parse_box_shadow($input)? };
            Opacity, "opacity", [], "baseline.property.opacity", CssOpacity, CssOpacityPropertyValue, CssOpacityPropertyValueRepresentation, parse_opacity, { parse_opacity($input)? };
            FlexGrow, "flex-grow", [], "baseline.property.flex-grow", CssFlexFactor, CssFlexGrowPropertyValue, CssFlexGrowPropertyValueRepresentation, parse_flex_factor, { parse_flex_factor($input, "flex-grow")? };
            FlexShrink, "flex-shrink", [], "baseline.property.flex-shrink", CssFlexFactor, CssFlexShrinkPropertyValue, CssFlexShrinkPropertyValueRepresentation, parse_flex_factor, { parse_flex_factor($input, "flex-shrink")? };
            Order, "order", [], "baseline.property.order", CssOrder, CssOrderPropertyValue, CssOrderPropertyValueRepresentation, parse_order, { parse_order($input)? };
            Flex, "flex", [], "baseline.property.flex", CssFlex, CssFlexPropertyValue, CssFlexPropertyValueRepresentation, parse_flex, { parse_flex($input)? };
            JustifyTracks, "justify-tracks", [], "baseline.property.justify-tracks", CssAlignment, CssJustifyTracksPropertyValue, CssJustifyTracksPropertyValueRepresentation, parse_content_alignment, { parse_content_alignment($input)? };
            AlignTracks, "align-tracks", [], "baseline.property.align-tracks", CssAlignment, CssAlignTracksPropertyValue, CssAlignTracksPropertyValueRepresentation, parse_content_alignment, { parse_content_alignment($input)? };
            AspectRatio, "aspect-ratio", [], "baseline.property.aspect-ratio", CssAspectRatio, CssAspectRatioPropertyValue, CssAspectRatioPropertyValueRepresentation, parse_aspect_ratio, { parse_aspect_ratio($input)? };
            ScrollbarWidth, "scrollbar-width", [], "baseline.property.scrollbar-width", CssScrollbarWidth, CssScrollbarWidthPropertyValue, CssScrollbarWidthPropertyValueRepresentation, parse_scrollbar_width, { parse_scrollbar_width($input)? };
            Cursor, "cursor", [], "baseline.property.cursor", CssCursor, CssCursorPropertyValue, CssCursorPropertyValueRepresentation, parse_cursor, { parse_cursor($input)? };
            CaretColor, "caret-color", [], "official.property.caret-color", CssCaretColor, CssCaretColorPropertyValue, CssCaretColorPropertyValueRepresentation, parse_caret_color, { parse_caret_color($input)? };
            PointerEvents, "pointer-events", [], "baseline.property.pointer-events", CssPointerEvents, CssPointerEventsPropertyValue, CssPointerEventsPropertyValueRepresentation, parse_pointer_events, { parse_pointer_events($input)? };
            UserSelect, "user-select", [], "baseline.property.user-select", CssUserSelect, CssUserSelectPropertyValue, CssUserSelectPropertyValueRepresentation, parse_user_select, { parse_user_select($input)? };
            Resize, "resize", [], "official.property.resize", CssResize, CssResizePropertyValue, CssResizePropertyValueRepresentation, parse_resize, { parse_resize($input)? };
            Contain, "contain", [], "official.property.contain", CssContain, CssContainPropertyValue, CssContainPropertyValueRepresentation, parse_contain, { parse_contain($input)? };
            Outline, "outline", [], "baseline.property.outline", CssOutline, CssOutlinePropertyValue, CssOutlinePropertyValueRepresentation, parse_outline, { parse_outline($input)? };
            OutlineColor, "outline-color", [], "baseline.property.outline-color", CssColor, CssOutlineColorPropertyValue, CssOutlineColorPropertyValueRepresentation, parse_color, { parse_color($input)? };
            OutlineOffset, "outline-offset", [], "official.property.outline-offset", CssOutlineOffset, CssOutlineOffsetPropertyValue, CssOutlineOffsetPropertyValueRepresentation, parse_outline_offset, { parse_outline_offset($input)? };
            OutlineStyle, "outline-style", [], "baseline.property.outline-style", CssOutlineStyle, CssOutlineStylePropertyValue, CssOutlineStylePropertyValueRepresentation, parse_outline_style, { parse_outline_style($input)? };
            OutlineWidth, "outline-width", [], "baseline.property.outline-width", CssOutlineWidth, CssOutlineWidthPropertyValue, CssOutlineWidthPropertyValueRepresentation, parse_outline_width, { parse_outline_width($input)? };
            Transform, "transform", [], "baseline.property.transform", CssTransform, CssTransformPropertyValue, CssTransformPropertyValueRepresentation, parse_transform, { parse_transform($input)? };
            TransformBox, "transform-box", [], "official.property.transform-box", CssTransformBox, CssTransformBoxPropertyValue, CssTransformBoxPropertyValueRepresentation, parse_transform_box, { parse_transform_box($input)? };
            TransformOrigin, "transform-origin", [], "baseline.property.transform-origin", CssTransformOrigin, CssTransformOriginPropertyValue, CssTransformOriginPropertyValueRepresentation, parse_transform_origin, { parse_transform_origin($input)? };
            Translate, "translate", [], "baseline.property.translate", CssTranslate, CssTranslatePropertyValue, CssTranslatePropertyValueRepresentation, parse_translate, { parse_translate($input)? };
            Rotate, "rotate", [], "baseline.property.rotate", CssRotate, CssRotatePropertyValue, CssRotatePropertyValueRepresentation, parse_rotate, { parse_rotate($input)? };
            Scale, "scale", [], "baseline.property.scale", CssScale, CssScalePropertyValue, CssScalePropertyValueRepresentation, parse_scale, { parse_scale($input)? };
            Filter, "filter", [], "baseline.property.filter", CssFilter, CssFilterPropertyValue, CssFilterPropertyValueRepresentation, parse_filter, { parse_filter($input)? };
            BackdropFilter, "backdrop-filter", [], "baseline.property.backdrop-filter", CssFilter, CssBackdropFilterPropertyValue, CssBackdropFilterPropertyValueRepresentation, parse_filter, { parse_filter($input)? };
            ClipPath, "clip-path", [], "baseline.property.clip-path", CssClipPath, CssClipPathPropertyValue, CssClipPathPropertyValueRepresentation, parse_clip_path, { parse_clip_path($input)? };
            BackgroundBlendMode, "background-blend-mode", [], "official.property.background-blend-mode", CssBlendModeList, CssBackgroundBlendModePropertyValue, CssBackgroundBlendModePropertyValueRepresentation, parse_blend_mode_list, { parse_blend_mode_list($input)? };
            Isolation, "isolation", [], "official.property.isolation", CssIsolation, CssIsolationPropertyValue, CssIsolationPropertyValueRepresentation, parse_isolation, { parse_isolation($input)? };
            MixBlendMode, "mix-blend-mode", [], "official.property.mix-blend-mode", CssBlendMode, CssMixBlendModePropertyValue, CssMixBlendModePropertyValueRepresentation, parse_blend_mode, { parse_blend_mode($input)? };
            Mask, "mask", [], "baseline.property.mask", CssMaskList, CssMaskPropertyValue, CssMaskPropertyValueRepresentation, parse_mask_list, { parse_mask_list($input)? };
            MaskImage, "mask-image", [], "baseline.property.mask-image", CssImageLayerList, CssMaskImagePropertyValue, CssMaskImagePropertyValueRepresentation, parse_image_layer_list, { parse_image_layer_list($input)? };
            MaskSize, "mask-size", [], "baseline.property.mask-size", CssBackgroundSizeList, CssMaskSizePropertyValue, CssMaskSizePropertyValueRepresentation, parse_background_size_list, { parse_background_size_list($input)? };
            MaskPosition, "mask-position", [], "baseline.property.mask-position", CssMaskPositionList, CssMaskPositionPropertyValue, CssMaskPositionPropertyValueRepresentation, parse_mask_position_list, { parse_mask_position_list($input)? };
            MaskRepeat, "mask-repeat", [], "baseline.property.mask-repeat", CssBackgroundRepeatList, CssMaskRepeatPropertyValue, CssMaskRepeatPropertyValueRepresentation, parse_background_repeat_list, { parse_background_repeat_list($input)? };
            TransitionProperty, "transition-property", [], "baseline.property.transition-property", CssTransitionPropertyList, CssTransitionPropertyPropertyValue, CssTransitionPropertyPropertyValueRepresentation, parse_transition_property_list, { parse_transition_property_list($input)? };
            TransitionDuration, "transition-duration", [], "baseline.property.transition-duration", CssTimeList, CssTransitionDurationPropertyValue, CssTransitionDurationPropertyValueRepresentation, parse_duration_list, { parse_duration_list($input)? };
            TransitionDelay, "transition-delay", [], "baseline.property.transition-delay", CssTimeList, CssTransitionDelayPropertyValue, CssTransitionDelayPropertyValueRepresentation, parse_delay_list, { parse_delay_list($input)? };
            TransitionTimingFunction, "transition-timing-function", [], "baseline.property.transition-timing-function", CssEasingList, CssTransitionTimingFunctionPropertyValue, CssTransitionTimingFunctionPropertyValueRepresentation, parse_easing_list, { parse_easing_list($input)? };
            Transition, "transition", [], "baseline.property.transition", CssTransitionList, CssTransitionPropertyValue, CssTransitionPropertyValueRepresentation, parse_transition_value_list, { parse_transition_value_list($input)? };
            AnimationName, "animation-name", [], "baseline.property.animation-name", CssAnimationNameList, CssAnimationNamePropertyValue, CssAnimationNamePropertyValueRepresentation, parse_animation_name_list, { parse_animation_name_list($input)? };
            AnimationDuration, "animation-duration", [], "baseline.property.animation-duration", CssTimeList, CssAnimationDurationPropertyValue, CssAnimationDurationPropertyValueRepresentation, parse_duration_list, { parse_duration_list($input)? };
            AnimationDelay, "animation-delay", [], "baseline.property.animation-delay", CssTimeList, CssAnimationDelayPropertyValue, CssAnimationDelayPropertyValueRepresentation, parse_delay_list, { parse_delay_list($input)? };
            AnimationTimingFunction, "animation-timing-function", [], "baseline.property.animation-timing-function", CssEasingList, CssAnimationTimingFunctionPropertyValue, CssAnimationTimingFunctionPropertyValueRepresentation, parse_easing_list, { parse_easing_list($input)? };
            AnimationIterationCount, "animation-iteration-count", [], "baseline.property.animation-iteration-count", CssAnimationIterationCountList, CssAnimationIterationCountPropertyValue, CssAnimationIterationCountPropertyValueRepresentation, parse_animation_iteration_value_list, { parse_animation_iteration_value_list($input)? };
            AnimationDirection, "animation-direction", [], "baseline.property.animation-direction", CssAnimationDirectionList, CssAnimationDirectionPropertyValue, CssAnimationDirectionPropertyValueRepresentation, parse_animation_direction_list, { parse_animation_direction_list($input)? };
            AnimationFillMode, "animation-fill-mode", [], "baseline.property.animation-fill-mode", CssAnimationFillModeList, CssAnimationFillModePropertyValue, CssAnimationFillModePropertyValueRepresentation, parse_animation_fill_mode_list, { parse_animation_fill_mode_list($input)? };
            AnimationPlayState, "animation-play-state", [], "baseline.property.animation-play-state", CssAnimationPlayStateList, CssAnimationPlayStatePropertyValue, CssAnimationPlayStatePropertyValueRepresentation, parse_animation_play_state_list, { parse_animation_play_state_list($input)? };
            Animation, "animation", [], "baseline.property.animation", CssAnimationList, CssAnimationPropertyValue, CssAnimationPropertyValueRepresentation, parse_animation_value_list, { parse_animation_value_list($input)? };
        }
    };
}

pub(crate) use property_schema;

fn opacity_i01_projection(value: &CssOpacityValue) -> Option<CssOpacity> {
    match value {
        CssOpacityValue::Literal(value) => Some(*value),
        CssOpacityValue::Calculation(_)
        | CssOpacityValue::Number(_)
        | CssOpacityValue::Percentage(_)
        | CssOpacityValue::PercentageCalculation(_) => None,
    }
}

fn flex_factor_i01_projection(value: &CssNonNegativeNumberValue) -> Option<CssFlexFactor> {
    match value {
        CssNonNegativeNumberValue::Literal(value) => CssFlexFactor::try_new(value.value()),
        CssNonNegativeNumberValue::Calculation(_) => None,
    }
}

fn integer_i01_projection(value: &CssIntegerValue) -> Option<i32> {
    match value {
        CssIntegerValue::Literal(value) => Some(*value),
        CssIntegerValue::Calculation(_) => None,
    }
}

fn aspect_ratio_i01_projection(value: &CssAspectRatioValue) -> Option<CssAspectRatio> {
    match value {
        CssAspectRatioValue::Literal(value) => Some(*value),
        CssAspectRatioValue::Calculation(_) => None,
    }
}

fn flex_i01_projection(value: &CssFlexValue) -> Option<CssFlex> {
    match value {
        CssFlexValue::None => Some(CssFlex::None),
        CssFlexValue::Auto => Some(CssFlex::Auto),
        CssFlexValue::Components(components) => {
            let grow = flex_factor_i01_projection(components.grow())?;
            let shrink = match components.shrink() {
                Some(value) => Some(flex_factor_i01_projection(value)?),
                None => None,
            };
            Some(CssFlex::components(
                grow,
                shrink,
                components.basis().cloned(),
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

fn font_size_i01_projection(value: &CssFontSize) -> Option<CssLength> {
    match value {
        CssFontSize::LengthPercentage(value) => Some(value.value().clone()),
        CssFontSize::XxSmall
        | CssFontSize::XSmall
        | CssFontSize::Small
        | CssFontSize::Medium
        | CssFontSize::Large
        | CssFontSize::XLarge
        | CssFontSize::XxLarge
        | CssFontSize::Larger
        | CssFontSize::Smaller => None,
    }
}

fn line_height_i01_projection(value: &CssLineHeight) -> Option<CssLength> {
    match value {
        CssLineHeight::Normal => Some(CssLength::Normal),
        CssLineHeight::Number(CssNonNegativeNumberValue::Literal(value))
            if value.value() == 0.0 =>
        {
            Some(CssLength::Zero)
        }
        CssLineHeight::Number(_) => None,
        CssLineHeight::LengthPercentage(value) => Some(value.value().clone()),
    }
}

fn font_family_i01_projection(value: &CssFontFamilyList) -> Option<CssFontFamilyList> {
    let families = value
        .families()
        .iter()
        .map(|family| match family.kind() {
            CssFontFamilyNameKind::Generic => CssFontFamilyName::ident_sequence(family.as_str()),
            CssFontFamilyNameKind::Quoted | CssFontFamilyNameKind::IdentSequence => family.clone(),
        })
        .collect();
    CssFontFamilyList::try_new(families)
}

fn font_i01_projection(value: &CssFontValue) -> Option<CssFont> {
    let CssFontValue::Explicit(value) = value else {
        return None;
    };
    let line_height = match value.line_height() {
        Some(value) => Some(line_height_i01_projection(value)?),
        None => None,
    };
    CssFont::try_new(
        value.style(),
        value.variant(),
        value.weight(),
        value.stretch(),
        font_size_i01_projection(value.size())?,
        line_height,
        font_family_i01_projection(value.families())?,
    )
}

fn font_variant_i01_projection(value: &CssFontVariantValue) -> Option<CssFontVariant> {
    match value {
        CssFontVariantValue::Normal => Some(CssFontVariant::Normal),
        CssFontVariantValue::Values(values)
            if values.ligatures().is_none()
                && values.position().is_none()
                && values.caps() == Some(CssFontVariantCaps::SmallCaps)
                && values.numeric().is_none()
                && values.east_asian().is_none() =>
        {
            Some(CssFontVariant::SmallCaps)
        }
        CssFontVariantValue::None | CssFontVariantValue::Values(_) => None,
    }
}

fn font_feature_settings_i01_projection(
    value: &CssAuthoredFontFeatureSettings,
) -> CssFontFeatureSettings {
    match value {
        CssAuthoredFontFeatureSettings::Normal => CssFontFeatureSettings::Normal,
        CssAuthoredFontFeatureSettings::Features(features) => {
            let features = features
                .features()
                .iter()
                .map(|feature| {
                    let value = match feature.value() {
                        CssAuthoredFontFeatureValue::Omitted => None,
                        CssAuthoredFontFeatureValue::On => Some(CssFontFeatureValue::On),
                        CssAuthoredFontFeatureValue::Off => Some(CssFontFeatureValue::Off),
                        CssAuthoredFontFeatureValue::Index(value) => {
                            Some(CssFontFeatureValue::Integer(value.value()))
                        }
                    };
                    CssFontFeature::new(feature.tag().as_str(), value)
                })
                .collect();
            CssFontFeatureSettings::Features(CssFontFeatureList::new(features))
        }
    }
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

        #[doc = concat!("A parser-produced authored ordinary value for `", $canonical, "`.")]
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

macro_rules! define_additive_current_property_value {
    (
        $canonical:literal, $wrapper:ident, $representation:ident,
        $current:ty, $accessor:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $representation {
            current: $current,
        }

        #[doc = concat!("A parser-produced current authored value for `", $canonical, "`.")]
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

        #[doc = concat!("A parser-produced authored ordinary value for `", $canonical, "`.")]
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

        #[doc = concat!("A parser-produced authored ordinary value for `", $canonical, "`.")]
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

        #[doc = concat!("A parser-produced authored ordinary value for `", $canonical, "`.")]
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

        #[doc = concat!("A parser-produced current authored value for `", $canonical, "`.")]
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

            /// Returns the parser-owned current authored Grid value.
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

macro_rules! define_property_value {
    (
        FlexFlow, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
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
        PageBreakAfter, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            page_break
        );
    };
    (
        PageBreakBefore, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            page_break
        );
    };
    (
        PageBreakInside, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            page_break
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
        define_additive_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            $value,
            spacing
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
        FontSize, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontSize,
            CssLength,
            size,
            font_size_i01_projection
        );
    };
    (
        LineHeight, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssLineHeight,
            CssLength,
            line_height,
            line_height_i01_projection
        );
    };
    (
        FontFamily, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontFamilyList,
            CssFontFamilyList,
            families,
            font_family_i01_projection
        );
    };
    (
        Font, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontValue,
            CssFont,
            font,
            font_i01_projection
        );
    };
    (
        FontVariant, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssFontVariantValue,
            CssFontVariant,
            variant,
            font_variant_i01_projection
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
        FontFeatureSettings, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssAuthoredFontFeatureSettings,
            CssFontFeatureSettings,
            settings,
            |value| Some(font_feature_settings_i01_projection(value))
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

        /// A parser-produced authored ordinary value for `background`.
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
        define_color_property_value!($canonical, $wrapper, $representation);
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
        define_authored_color_aggregate_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorder
        );
    };
    (
        BorderTop, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_authored_color_aggregate_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorder
        );
    };
    (
        BorderRight, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_authored_color_aggregate_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorder
        );
    };
    (
        BorderBottom, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_authored_color_aggregate_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorder
        );
    };
    (
        BorderLeft, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_authored_color_aggregate_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssBorder
        );
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

        /// A parser-produced authored ordinary `box-shadow` value.
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

        /// A parser-produced authored ordinary value for `border-image`.
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

        /// A parser-produced authored ordinary `clip-path` value.
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

        /// A parser-produced authored ordinary value for `transform`.
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

        /// A parser-produced authored ordinary value for `object-position`.
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
        FlexGrow, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        define_current_property_value!(
            $canonical,
            $wrapper,
            $representation,
            CssNonNegativeNumberValue,
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
            CssNonNegativeNumberValue,
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
        GridFlowTolerance, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct CssGridFlowTolerancePropertyValueRepresentation {
            current: CssGridFlowToleranceValue,
            i01_subset: Option<CssGridFlowTolerance>,
        }

        impl CssGridFlowTolerancePropertyValueRepresentation {
            #[must_use]
            pub(crate) const fn new(
                current: CssGridFlowToleranceValue,
                i01_subset: Option<CssGridFlowTolerance>,
            ) -> Self {
                Self {
                    current,
                    i01_subset,
                }
            }
        }

        /// A parser-produced authored ordinary value for `grid-flow-tolerance`.
        ///
        /// The current checked value remains distinct from the frozen I01 compatibility payload.
        #[derive(Clone, Debug, PartialEq)]
        pub struct CssGridFlowTolerancePropertyValue {
            authored: CssAuthoredDeclarationValue,
            representation: CssGridFlowTolerancePropertyValueRepresentation,
        }

        impl CssGridFlowTolerancePropertyValue {
            #[must_use]
            pub(crate) const fn new(
                authored: CssAuthoredDeclarationValue,
                representation: CssGridFlowTolerancePropertyValueRepresentation,
            ) -> Self {
                Self {
                    authored,
                    representation,
                }
            }

            /// Returns the exact authored ordinary value slice, excluding boundary trivia and a
            /// terminal importance annotation.
            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }

            /// Returns the checked current authored value.
            #[must_use]
            pub const fn value(&self) -> &CssGridFlowToleranceValue {
                &self.representation.current
            }

            /// Returns the frozen I01 compatibility payload when this value belongs to that
            /// subset.
            #[must_use]
            pub const fn i01_subset(&self) -> Option<&CssGridFlowTolerance> {
                self.representation.i01_subset.as_ref()
            }
        }
    };
    (
        $variant:ident, $canonical:literal, $value:ty, $wrapper:ident,
        $representation:ident
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        enum $representation {
            I01($value),
        }

        #[doc = concat!("A parser-produced authored ordinary value for `", $canonical, "`.")]
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

        #[doc = concat!("A parser-produced authored ordinary value for `", $canonical, "`.")]
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

        #[doc = concat!("A parser-produced authored ordinary value for `", $canonical, "`.")]
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

macro_rules! define_property_identity {
    ($input:ident;
        All, $all_canonical:literal, [$($all_alias:literal),*], $all_stable_id:literal,
        $all_value:ty, $all_wrapper:ident, $all_representation:ident,
        $all_parser:ident, $all_dispatch:block;
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $wrapper:ident, $representation:ident, $parser:ident, $dispatch:block;
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

        $(define_property_value!(
            $variant, $canonical, $value, $wrapper, $representation
        );)*

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

        /// A parser-owned property-coupled known declaration in the authored syntax phase.
        ///
        /// Its private field is the sole property identity and value discriminator. Callers can
        /// inspect, but cannot construct or mutate, a property/value mismatch.
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
            value: CssKnownDeclarationValue,
        }

        impl CssKnownDeclaration {
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
                Self { value }
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
                Self { value }
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
                Self { value }
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

property_schema!(define_property_identity, schema_input);

/// A reviewed authored property-name resolution outside name-equivalent schema aliases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CssLegacyPropertyAlias {
    GlyphOrientationVertical,
}

impl CssLegacyPropertyAlias {
    pub(crate) const fn target(self) -> CssKnownProperty {
        match self {
            Self::GlyphOrientationVertical => CssKnownProperty::TextOrientation,
        }
    }
}

pub(crate) fn resolve_property_name(name: &str) -> Option<CssResolvedPropertyName> {
    if let Some(property) = CssKnownProperty::from_name(name) {
        return Some(CssResolvedPropertyName::Canonical(property));
    }
    name.eq_ignore_ascii_case("glyph-orientation-vertical")
        .then_some(CssResolvedPropertyName::LegacyShorthand(
            CssLegacyPropertyAlias::GlyphOrientationVertical,
        ))
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
