//! Canonical identity and parser wiring for recognized CSS properties.
//!
//! The crate-private schema in this module is the single authority for the
//! frozen property set. Public identity values describe authored property names;
//! they do not apply cascade, substitute variables, or resolve authored values.

use crate::syntax::*;
use crate::{
    BorderSpacingLength, CornerRadiusLength, CssContainIntrinsicSizeValue, CssCue, CssGapValue,
    CssInsetValue, CssInterestDelayValue, CssLineWidth, CssMarginValue, CssMaxSizeValue,
    CssPaddingValue, CssScrollSnapAlignment, CssSizeValue, CssSpeechBreak, ScrollMarginSide,
    ScrollPaddingSide,
};

macro_rules! property_schema {
    ($callback:ident, $input:ident, $numeric:ident) => {
        // Flexbox 1 normative Appendix B supplies the twelve name-equivalent
        // -webkit- aliases; their canonical property definitions keep their own sources.
        $callback! {
            $input, $numeric;
            All, "all", [], "baseline.property.all", crate::CssAllDeclaredValue, parse_all_property, { parse_all_property($input)? }, expansion = universal { exclude_custom: true, excluded: [Direction, UnicodeBidi] };
            ContainerName, "container-name", [], "official.property.container-name", crate::CssContainerNames, parse_container_names, { parse_container_names($input)? }, expansion = longhand { value: CssContainerNames, inherited: false, initial_kind: value, initial: CssContainerNames::None };
            ContainerType, "container-type", [], "official.property.container-type", crate::CssContainerType, parse_container_type, { parse_container_type($input)? }, expansion = longhand { value: CssContainerType, inherited: false, initial_kind: value, initial: CssContainerType::Normal };
            Container, "container", [], "official.property.container", crate::CssContainer, parse_container, { parse_container($input)? }, expansion = shorthand { members: [ ContainerName => |value: &CssContainer| Some(value.names().clone()), ContainerType => |value: &CssContainer| Some(value.container_type()) ], reset_only: [] };
            Display, "display", [], "baseline.property.display", crate::CssDisplayValue, parse_display, { parse_display($input)? }, expansion = longhand { value: crate::CssDisplayValue, inherited: false, initial_kind: value, initial: crate::CssDisplayValue::OutsideInside { outside: crate::CssDisplayOutside::Inline, inside: crate::CssDisplayInside::Flow } };
            BoxSizing, "box-sizing", [], "baseline.property.box-sizing", crate::CssBoxSizing, parse_box_sizing, { parse_box_sizing($input)? }, expansion = longhand { value: CssBoxSizing, inherited: false, initial_kind: value, initial: CssBoxSizing::ContentBox };
            BorderCollapse, "border-collapse", [], "official.property.border-collapse", crate::CssBorderCollapse, parse_border_collapse, { parse_border_collapse($input)? }, expansion = longhand { value: CssBorderCollapse, inherited: true, initial_kind: value, initial: CssBorderCollapse::Separate };
            BorderSpacing, "border-spacing", [], "official.property.border-spacing", crate::CssSidePair<BorderSpacingLength>, parse_border_spacing, { parse_border_spacing($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<crate::BorderSpacingLength>, inherited: true, initial_kind: value, initial: crate::CssSidePair::new(crate::CssNumeric::<crate::CssLength, crate::CssNonNegative>::exact_zero(), None) };
            CaptionSide, "caption-side", [], "official.property.caption-side", crate::CssCaptionSide, parse_caption_side, { parse_caption_side($input)? }, expansion = longhand { value: CssCaptionSide, inherited: true, initial_kind: value, initial: CssCaptionSide::Top };
            Clip, "clip", [], "official.property.clip", crate::CssClip, parse_clip, { parse_clip($input, $numeric)? }, expansion = longhand { value: CssClip, inherited: false, initial_kind: value, initial: CssClip::Auto };
            EmptyCells, "empty-cells", [], "official.property.empty-cells", crate::CssEmptyCells, parse_empty_cells, { parse_empty_cells($input)? }, expansion = longhand { value: CssEmptyCells, inherited: true, initial_kind: value, initial: CssEmptyCells::Show };
            Orphans, "orphans", [], "official.property.orphans", crate::CssNumeric<CssInteger, CssPositive>, parse_page_line_minimum, { parse_page_line_minimum($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssInteger, CssPositive>, inherited: true, initial_kind: value, initial: CssNumeric::<CssInteger, CssPositive>::try_from_literal(CssNumericLiteral::from_i32(2)).expect("positive initial line minimum") };
            BreakAfter, "break-after", [], "official.property.break-after", crate::CssBreakBetween, parse_break_between, { parse_break_between($input)? }, expansion = longhand { value: CssBreakBetween, inherited: false, initial_kind: value, initial: CssBreakBetween::Auto };
            BreakBefore, "break-before", [], "official.property.break-before", crate::CssBreakBetween, parse_break_between, { parse_break_between($input)? }, expansion = longhand { value: CssBreakBetween, inherited: false, initial_kind: value, initial: CssBreakBetween::Auto };
            BreakInside, "break-inside", [], "official.property.break-inside", crate::CssBreakInside, parse_break_inside, { parse_break_inside($input)? }, expansion = longhand { value: CssBreakInside, inherited: false, initial_kind: value, initial: CssBreakInside::Auto };
            Quotes, "quotes", [], "official.property.quotes", crate::CssQuotes, parse_quotes, { parse_quotes($input)? }, expansion = longhand { value: CssQuotes, inherited: true, initial_kind: value, initial: CssQuotes::Auto };
            TableLayout, "table-layout", [], "official.property.table-layout", crate::CssTableLayout, parse_table_layout, { parse_table_layout($input)? }, expansion = longhand { value: CssTableLayout, inherited: false, initial_kind: value, initial: CssTableLayout::Auto };
            ScrollSnapType, "scroll-snap-type", [], "official.property.scroll-snap-type", crate::CssScrollSnapType, parse_scroll_snap_type, { parse_scroll_snap_type($input)? }, expansion = longhand { value: CssScrollSnapType, inherited: false, initial_kind: value, initial: CssScrollSnapType::None };
            ScrollSnapAlign, "scroll-snap-align", [], "official.property.scroll-snap-align", crate::CssSidePair<CssScrollSnapAlignment>, parse_scroll_snap_align, { parse_scroll_snap_align($input)? }, expansion = longhand { value: crate::CssSidePair<CssScrollSnapAlignment>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssScrollSnapAlignment::None, None) };
            ScrollSnapStop, "scroll-snap-stop", [], "official.property.scroll-snap-stop", crate::CssScrollSnapStop, parse_scroll_snap_stop, { parse_scroll_snap_stop($input)? }, expansion = longhand { value: CssScrollSnapStop, inherited: false, initial_kind: value, initial: CssScrollSnapStop::Normal };
            ScrollPaddingTop, "scroll-padding-top", [], "official.property.scroll-padding-top", crate::ScrollPaddingSide, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: crate::ScrollPaddingSide, inherited: false, initial_kind: value, initial: CssAutoOr::Auto, logical_group: ScrollPadding, mapping: Physical };
            ScrollPaddingRight, "scroll-padding-right", [], "official.property.scroll-padding-right", crate::ScrollPaddingSide, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: crate::ScrollPaddingSide, inherited: false, initial_kind: value, initial: CssAutoOr::Auto, logical_group: ScrollPadding, mapping: Physical };
            ScrollPaddingBottom, "scroll-padding-bottom", [], "official.property.scroll-padding-bottom", crate::ScrollPaddingSide, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: crate::ScrollPaddingSide, inherited: false, initial_kind: value, initial: CssAutoOr::Auto, logical_group: ScrollPadding, mapping: Physical };
            ScrollPaddingLeft, "scroll-padding-left", [], "official.property.scroll-padding-left", crate::ScrollPaddingSide, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: crate::ScrollPaddingSide, inherited: false, initial_kind: value, initial: CssAutoOr::Auto, logical_group: ScrollPadding, mapping: Physical };
            ScrollPaddingBlockStart, "scroll-padding-block-start", [], "official.property.scroll-padding-block-start", crate::ScrollPaddingSide, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: crate::ScrollPaddingSide, inherited: false, initial_kind: value, initial: CssAutoOr::Auto, logical_group: ScrollPadding, mapping: Logical };
            ScrollPaddingBlockEnd, "scroll-padding-block-end", [], "official.property.scroll-padding-block-end", crate::ScrollPaddingSide, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: crate::ScrollPaddingSide, inherited: false, initial_kind: value, initial: CssAutoOr::Auto, logical_group: ScrollPadding, mapping: Logical };
            ScrollPaddingInlineStart, "scroll-padding-inline-start", [], "official.property.scroll-padding-inline-start", crate::ScrollPaddingSide, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: crate::ScrollPaddingSide, inherited: false, initial_kind: value, initial: CssAutoOr::Auto, logical_group: ScrollPadding, mapping: Logical };
            ScrollPaddingInlineEnd, "scroll-padding-inline-end", [], "official.property.scroll-padding-inline-end", crate::ScrollPaddingSide, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: crate::ScrollPaddingSide, inherited: false, initial_kind: value, initial: CssAutoOr::Auto, logical_group: ScrollPadding, mapping: Logical };
            ScrollMarginTop, "scroll-margin-top", [], "official.property.scroll-margin-top", crate::CssNumeric<CssLength>, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero(), logical_group: ScrollMargin, mapping: Physical };
            ScrollMarginRight, "scroll-margin-right", [], "official.property.scroll-margin-right", crate::CssNumeric<CssLength>, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero(), logical_group: ScrollMargin, mapping: Physical };
            ScrollMarginBottom, "scroll-margin-bottom", [], "official.property.scroll-margin-bottom", crate::CssNumeric<CssLength>, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero(), logical_group: ScrollMargin, mapping: Physical };
            ScrollMarginLeft, "scroll-margin-left", [], "official.property.scroll-margin-left", crate::CssNumeric<CssLength>, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero(), logical_group: ScrollMargin, mapping: Physical };
            ScrollMarginBlockStart, "scroll-margin-block-start", [], "official.property.scroll-margin-block-start", crate::CssNumeric<CssLength>, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero(), logical_group: ScrollMargin, mapping: Logical };
            ScrollMarginBlockEnd, "scroll-margin-block-end", [], "official.property.scroll-margin-block-end", crate::CssNumeric<CssLength>, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero(), logical_group: ScrollMargin, mapping: Logical };
            ScrollMarginInlineStart, "scroll-margin-inline-start", [], "official.property.scroll-margin-inline-start", crate::CssNumeric<CssLength>, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero(), logical_group: ScrollMargin, mapping: Logical };
            ScrollMarginInlineEnd, "scroll-margin-inline-end", [], "official.property.scroll-margin-inline-end", crate::CssNumeric<CssLength>, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero(), logical_group: ScrollMargin, mapping: Logical };
            ScrollPaddingBlock, "scroll-padding-block", [], "official.property.scroll-padding-block", crate::CssSidePair<ScrollPaddingSide>, parse_scroll_padding_pair, { parse_scroll_padding_pair($input, $numeric)? }, expansion = shorthand { members: [ ScrollPaddingBlockStart => |value: &crate::CssSidePair<crate::ScrollPaddingSide>| Some(value.first().clone()), ScrollPaddingBlockEnd => |value: &crate::CssSidePair<crate::ScrollPaddingSide>| Some(value.second().clone()) ], reset_only: [] };
            ScrollPaddingInline, "scroll-padding-inline", [], "official.property.scroll-padding-inline", crate::CssSidePair<ScrollPaddingSide>, parse_scroll_padding_pair, { parse_scroll_padding_pair($input, $numeric)? }, expansion = shorthand { members: [ ScrollPaddingInlineStart => |value: &crate::CssSidePair<crate::ScrollPaddingSide>| Some(value.first().clone()), ScrollPaddingInlineEnd => |value: &crate::CssSidePair<crate::ScrollPaddingSide>| Some(value.second().clone()) ], reset_only: [] };
            ScrollMarginBlock, "scroll-margin-block", [], "official.property.scroll-margin-block", crate::CssSidePair<ScrollMarginSide>, parse_scroll_margin_pair, { parse_scroll_margin_pair($input, $numeric)? }, expansion = shorthand { members: [ ScrollMarginBlockStart => |value: &crate::CssSidePair<crate::ScrollMarginSide>| Some(value.first().clone()), ScrollMarginBlockEnd => |value: &crate::CssSidePair<crate::ScrollMarginSide>| Some(value.second().clone()) ], reset_only: [] };
            ScrollMarginInline, "scroll-margin-inline", [], "official.property.scroll-margin-inline", crate::CssSidePair<ScrollMarginSide>, parse_scroll_margin_pair, { parse_scroll_margin_pair($input, $numeric)? }, expansion = shorthand { members: [ ScrollMarginInlineStart => |value: &crate::CssSidePair<crate::ScrollMarginSide>| Some(value.first().clone()), ScrollMarginInlineEnd => |value: &crate::CssSidePair<crate::ScrollMarginSide>| Some(value.second().clone()) ], reset_only: [] };
            ScrollPadding, "scroll-padding", [], "official.property.scroll-padding", crate::CssFourSides<ScrollPaddingSide>, parse_scroll_padding_shorthand, { parse_scroll_padding_shorthand($input, $numeric)? }, expansion = four_side { mode: |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.kind(), physical: [ ScrollPaddingTop => |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.assigned()[0].clone(), ScrollPaddingRight => |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.assigned()[1].clone(), ScrollPaddingBottom => |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.assigned()[2].clone(), ScrollPaddingLeft => |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.assigned()[3].clone() ], logical: [ ScrollPaddingBlockStart => |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.assigned()[0].clone(), ScrollPaddingInlineStart => |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.assigned()[1].clone(), ScrollPaddingBlockEnd => |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.assigned()[2].clone(), ScrollPaddingInlineEnd => |value: &crate::CssFourSides<crate::ScrollPaddingSide>| value.assigned()[3].clone() ] };
            ScrollMargin, "scroll-margin", [], "official.property.scroll-margin", crate::CssFourSides<ScrollMarginSide>, parse_scroll_margin_shorthand, { parse_scroll_margin_shorthand($input, $numeric)? }, expansion = four_side { mode: |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.kind(), physical: [ ScrollMarginTop => |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.assigned()[0].clone(), ScrollMarginRight => |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.assigned()[1].clone(), ScrollMarginBottom => |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.assigned()[2].clone(), ScrollMarginLeft => |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.assigned()[3].clone() ], logical: [ ScrollMarginBlockStart => |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.assigned()[0].clone(), ScrollMarginInlineStart => |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.assigned()[1].clone(), ScrollMarginBlockEnd => |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.assigned()[2].clone(), ScrollMarginInlineEnd => |value: &crate::CssFourSides<crate::ScrollMarginSide>| value.assigned()[3].clone() ] };
            Widows, "widows", [], "official.property.widows", crate::CssNumeric<CssInteger, CssPositive>, parse_page_line_minimum, { parse_page_line_minimum($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssInteger, CssPositive>, inherited: true, initial_kind: value, initial: CssNumeric::<CssInteger, CssPositive>::try_from_literal(CssNumericLiteral::from_i32(2)).expect("positive initial line minimum") };
            WordSpacing, "word-spacing", [], "official.property.word-spacing", crate::CssTextSpacingAdjustment, parse_word_spacing, { parse_word_spacing($input, $numeric)? }, expansion = longhand { value: crate::CssTextSpacingAdjustment, inherited: true, initial_kind: value, initial: crate::CssTextSpacingAdjustment::Normal };
            Position, "position", [], "baseline.property.position", crate::CssLayoutPosition, parse_position, { parse_position($input)? }, expansion = longhand { value: CssLayoutPosition, inherited: false, initial_kind: value, initial: CssLayoutPosition::Static };
            Direction, "direction", [], "baseline.property.direction", crate::CssDirection, parse_direction, { parse_direction($input)? }, expansion = longhand { value: CssDirection, inherited: true, initial_kind: value, initial: CssDirection::Ltr };
            Overflow, "overflow", [], "baseline.property.overflow", crate::CssOverflowValue, parse_overflow_value, { parse_overflow_value($input)? }, expansion = shorthand { members: [ OverflowX => |value: &CssOverflowValue| Some(value.x()), OverflowY => |value: &CssOverflowValue| Some(value.y()) ], reset_only: [] };
            OverflowX, "overflow-x", [], "baseline.property.overflow-x", crate::CssOverflow, parse_overflow, { parse_overflow($input)? }, expansion = longhand { value: CssOverflow, inherited: false, initial_kind: value, initial: CssOverflow::Visible, logical_group: Overflow, mapping: Physical };
            OverflowY, "overflow-y", [], "baseline.property.overflow-y", crate::CssOverflow, parse_overflow, { parse_overflow($input)? }, expansion = longhand { value: CssOverflow, inherited: false, initial_kind: value, initial: CssOverflow::Visible, logical_group: Overflow, mapping: Physical };
            OverflowBlock, "overflow-block", [], "ext.property.overflow-block", crate::CssOverflow, parse_overflow, { parse_overflow($input)? }, expansion = longhand { value: CssOverflow, inherited: false, initial_kind: value, initial: CssOverflow::Visible, logical_group: Overflow, mapping: Logical };
            OverflowInline, "overflow-inline", [], "ext.property.overflow-inline", crate::CssOverflow, parse_overflow, { parse_overflow($input)? }, expansion = longhand { value: CssOverflow, inherited: false, initial_kind: value, initial: CssOverflow::Visible, logical_group: Overflow, mapping: Logical };
            OverflowClipMargin, "overflow-clip-margin", [], "ext.property.overflow-clip-margin", crate::CssOverflowClipMargin, parse_overflow_clip_margin, { parse_overflow_clip_margin($input, $numeric)? }, expansion = longhand { value: CssOverflowClipMargin, inherited: false, initial_kind: value, initial: CssOverflowClipMargin::initial() };
            ViewTransitionName, "view-transition-name", [], "official.property.view-transition-name", crate::CssViewTransitionName, parse_view_transition_name, { parse_view_transition_name($input)? }, expansion = longhand { value: crate::CssViewTransitionName, inherited: false, initial_kind: value, initial: crate::CssViewTransitionName::None };
            WillChange, "will-change", [], "ext.property.will-change", crate::CssWillChange, parse_will_change, { parse_will_change($input)? }, expansion = longhand { value: crate::CssWillChange, inherited: false, initial_kind: value, initial: crate::CssWillChange::Auto };
            OverflowAnchor, "overflow-anchor", [], "ext.property.overflow-anchor", crate::CssOverflowAnchor, parse_overflow_anchor, { parse_overflow_anchor($input)? }, expansion = longhand { value: crate::CssOverflowAnchor, inherited: false, initial_kind: value, initial: crate::CssOverflowAnchor::Auto };
            ScrollBehavior, "scroll-behavior", [], "ext.property.scroll-behavior", crate::CssScrollBehavior, parse_scroll_behavior, { parse_scroll_behavior($input)? }, expansion = longhand { value: CssScrollBehavior, inherited: false, initial_kind: value, initial: CssScrollBehavior::Auto };
            ScrollbarGutter, "scrollbar-gutter", [], "ext.property.scrollbar-gutter", crate::CssScrollbarGutter, parse_scrollbar_gutter, { parse_scrollbar_gutter($input)? }, expansion = longhand { value: CssScrollbarGutter, inherited: false, initial_kind: value, initial: CssScrollbarGutter::Auto };
            VoiceDuration, "voice-duration", [], "official.property.voice-duration", crate::CssVoiceDuration, parse_voice_duration, { parse_voice_duration($input, $numeric)? }, expansion = longhand { value: CssVoiceDuration, inherited: false, initial_kind: value, initial: CssVoiceDuration::Auto };
            VoiceBalance, "voice-balance", [], "official.property.voice-balance", crate::CssVoiceBalance, parse_voice_balance, { parse_voice_balance($input, $numeric)? }, expansion = longhand { value: CssVoiceBalance, inherited: true, initial_kind: value, initial: CssVoiceBalance::from_keyword(CssVoiceBalanceKeyword::Center) };
            VoiceVolume, "voice-volume", [], "official.property.voice-volume", crate::CssVoiceVolume, parse_voice_volume, { parse_voice_volume($input, $numeric)? }, expansion = longhand { value: CssVoiceVolume, inherited: true, initial_kind: value, initial: CssVoiceVolume::Level { level: CssVoiceVolumeLevel::Medium, decibel: None } };
            VoicePitch, "voice-pitch", [], "official.property.voice-pitch", crate::CssVoicePitchRange, parse_voice_pitch_range, { parse_voice_pitch_range($input, $numeric)? }, expansion = longhand { value: CssVoicePitchRange, inherited: true, initial_kind: value, initial: CssVoicePitchRange::level_only(CssVoiceLevel::Medium) };
            VoiceRange, "voice-range", [], "official.property.voice-range", crate::CssVoicePitchRange, parse_voice_pitch_range, { parse_voice_pitch_range($input, $numeric)? }, expansion = longhand { value: CssVoicePitchRange, inherited: true, initial_kind: value, initial: CssVoicePitchRange::level_only(CssVoiceLevel::Medium) };
            VoiceRate, "voice-rate", [], "official.property.voice-rate", crate::CssVoiceRate, parse_voice_rate, { parse_voice_rate($input, $numeric)? }, expansion = longhand { value: CssVoiceRate, inherited: true, initial_kind: value, initial: CssVoiceRate::keyword_only(CssVoiceRateKeyword::Normal) };
            VoiceFamily, "voice-family", [], "official.property.voice-family", crate::CssVoiceFamily, parse_voice_family, { parse_voice_family($input, $numeric)? }, expansion = longhand { value: CssVoiceFamily, inherited: true, initial_kind: user_agent, initial: CssUserAgentInitial::VoiceFamily };
            VoiceStress, "voice-stress", [], "official.property.voice-stress", crate::CssVoiceStress, parse_voice_stress, { parse_voice_stress($input)? }, expansion = longhand { value: CssVoiceStress, inherited: true, initial_kind: value, initial: CssVoiceStress::Normal };
            Speak, "speak", [], "official.property.speak", crate::CssSpeak, parse_speak, { parse_speak($input)? }, expansion = longhand { value: CssSpeak, inherited: true, initial_kind: value, initial: CssSpeak::Auto };
            SpeakAs, "speak-as", [], "official.property.speak-as", crate::CssSpeakAs, parse_speak_as, { parse_speak_as($input)? }, expansion = longhand { value: CssSpeakAs, inherited: true, initial_kind: value, initial: CssSpeakAs::normal() };
            PauseBefore, "pause-before", [], "official.property.pause-before", crate::CssSpeechBreak, parse_speech_break, { parse_speech_break($input, $numeric)? }, expansion = longhand { value: CssSpeechBreak, inherited: false, initial_kind: value, initial: CssSpeechBreak::None };
            PauseAfter, "pause-after", [], "official.property.pause-after", crate::CssSpeechBreak, parse_speech_break, { parse_speech_break($input, $numeric)? }, expansion = longhand { value: CssSpeechBreak, inherited: false, initial_kind: value, initial: CssSpeechBreak::None };
            Pause, "pause", [], "official.property.pause", crate::CssSidePair<CssSpeechBreak>, parse_speech_break_pair, { parse_speech_break_pair($input, $numeric)? }, expansion = shorthand { members: [ PauseBefore => |value: &crate::CssSidePair<crate::CssSpeechBreak>| Some(value.first().clone()), PauseAfter => |value: &crate::CssSidePair<crate::CssSpeechBreak>| Some(value.second().clone()) ], reset_only: [] };
            RestBefore, "rest-before", [], "official.property.rest-before", crate::CssSpeechBreak, parse_speech_break, { parse_speech_break($input, $numeric)? }, expansion = longhand { value: CssSpeechBreak, inherited: false, initial_kind: value, initial: CssSpeechBreak::None };
            RestAfter, "rest-after", [], "official.property.rest-after", crate::CssSpeechBreak, parse_speech_break, { parse_speech_break($input, $numeric)? }, expansion = longhand { value: CssSpeechBreak, inherited: false, initial_kind: value, initial: CssSpeechBreak::None };
            Rest, "rest", [], "official.property.rest", crate::CssSidePair<CssSpeechBreak>, parse_speech_break_pair, { parse_speech_break_pair($input, $numeric)? }, expansion = shorthand { members: [ RestBefore => |value: &crate::CssSidePair<crate::CssSpeechBreak>| Some(value.first().clone()), RestAfter => |value: &crate::CssSidePair<crate::CssSpeechBreak>| Some(value.second().clone()) ], reset_only: [] };
            CueBefore, "cue-before", [], "official.property.cue-before", crate::CssCue, parse_cue, { parse_cue($input, $numeric)? }, expansion = longhand { value: CssCue, inherited: false, initial_kind: value, initial: CssCue::None };
            CueAfter, "cue-after", [], "official.property.cue-after", crate::CssCue, parse_cue, { parse_cue($input, $numeric)? }, expansion = longhand { value: CssCue, inherited: false, initial_kind: value, initial: CssCue::None };
            Cue, "cue", [], "official.property.cue", crate::CssSidePair<CssCue>, parse_cue_pair, { parse_cue_pair($input, $numeric)? }, expansion = shorthand { members: [ CueBefore => |value: &crate::CssSidePair<crate::CssCue>| Some(value.first().clone()), CueAfter => |value: &crate::CssSidePair<crate::CssCue>| Some(value.second().clone()) ], reset_only: [] };
            FlexDirection, "flex-direction", ["-webkit-flex-direction"], "baseline.property.flex-direction", crate::CssFlexDirection, parse_flex_direction, { parse_flex_direction($input)? }, expansion = longhand { value: CssFlexDirection, inherited: false, initial_kind: value, initial: CssFlexDirection::Row };
            FlexFlow, "flex-flow", ["-webkit-flex-flow"], "official.property.flex-flow", crate::CssFlexFlow, parse_flex_flow, { parse_flex_flow($input)? }, expansion = shorthand { members: [ FlexDirection => |value: &CssFlexFlow| Some(value.direction()), FlexWrap => |value: &CssFlexFlow| Some(value.wrap()) ], reset_only: [] };
            FlexWrap, "flex-wrap", ["-webkit-flex-wrap"], "baseline.property.flex-wrap", crate::CssFlexWrap, parse_flex_wrap, { parse_flex_wrap($input)? }, expansion = longhand { value: CssFlexWrap, inherited: false, initial_kind: value, initial: CssFlexWrap::NoWrap };
            Float, "float", [], "baseline.property.float", crate::CssFloat, parse_float, { parse_float($input)? }, expansion = longhand { value: CssFloat, inherited: false, initial_kind: value, initial: CssFloat::None };
            Clear, "clear", [], "baseline.property.clear", crate::CssClear, parse_clear, { parse_clear($input)? }, expansion = longhand { value: CssClear, inherited: false, initial_kind: value, initial: CssClear::None };
            AlignContent, "align-content", ["-webkit-align-content"], "baseline.property.align-content", crate::CssAlignContentValue, parse_align_content_value, { parse_align_content_value($input)? }, expansion = longhand { value: CssAlignContentValue, inherited: false, initial_kind: value, initial: CssAlignContentValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid align-content initial") };
            JustifyContent, "justify-content", ["-webkit-justify-content"], "baseline.property.justify-content", crate::CssJustifyContentValue, parse_justify_content_value, { parse_justify_content_value($input)? }, expansion = longhand { value: CssJustifyContentValue, inherited: false, initial_kind: value, initial: CssJustifyContentValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid justify-content initial") };
            AlignItems, "align-items", ["-webkit-align-items"], "baseline.property.align-items", crate::CssAlignItemsValue, parse_align_items_value, { parse_align_items_value($input)? }, expansion = longhand { value: CssAlignItemsValue, inherited: false, initial_kind: value, initial: CssAlignItemsValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid align-items initial") };
            AlignSelf, "align-self", ["-webkit-align-self"], "baseline.property.align-self", crate::CssAlignSelfValue, parse_align_self_value, { parse_align_self_value($input)? }, expansion = longhand { value: CssAlignSelfValue, inherited: false, initial_kind: value, initial: CssAlignSelfValue::try_new(CssAlignmentValue::Auto).expect("valid align-self initial") };
            JustifyItems, "justify-items", [], "baseline.property.justify-items", crate::CssJustifyItemsValue, parse_justify_items_value, { parse_justify_items_value($input)? }, expansion = longhand { value: CssJustifyItemsValue, inherited: false, initial_kind: value, initial: CssJustifyItemsValue::try_new(CssAlignmentValue::Legacy(None)).expect("valid justify-items initial") };
            JustifySelf, "justify-self", [], "baseline.property.justify-self", crate::CssJustifySelfValue, parse_justify_self_value, { parse_justify_self_value($input)? }, expansion = longhand { value: CssJustifySelfValue, inherited: false, initial_kind: value, initial: CssJustifySelfValue::try_new(CssAlignmentValue::Auto).expect("valid justify-self initial") };
            PlaceContent, "place-content", [], "baseline.property.place-content", crate::CssPlaceContentValue, parse_place_content_value, { parse_place_content_value($input)? }, expansion = shorthand { members: [ AlignContent => |value: &CssPlaceContentValue| Some(value.align()), JustifyContent => |value: &CssPlaceContentValue| Some(value.justify()) ], reset_only: [] };
            PlaceItems, "place-items", [], "baseline.property.place-items", crate::CssPlaceItemsValue, parse_place_items_value, { parse_place_items_value($input)? }, expansion = shorthand { members: [ AlignItems => |value: &CssPlaceItemsValue| Some(value.align()), JustifyItems => |value: &CssPlaceItemsValue| Some(value.justify()) ], reset_only: [] };
            PlaceSelf, "place-self", [], "baseline.property.place-self", crate::CssPlaceSelfValue, parse_place_self_value, { parse_place_self_value($input)? }, expansion = shorthand { members: [ AlignSelf => |value: &CssPlaceSelfValue| Some(value.align()), JustifySelf => |value: &CssPlaceSelfValue| Some(value.justify()) ], reset_only: [] };
            Visibility, "visibility", [], "baseline.property.visibility", crate::CssVisibility, parse_visibility, { parse_visibility($input)? }, expansion = longhand { value: CssVisibility, inherited: true, initial_kind: value, initial: CssVisibility::Visible };
            Content, "content", [], "baseline.property.content", crate::CssContentValue, parse_content, { parse_content($input, $numeric)? }, expansion = longhand { value: crate::CssContentValue, inherited: false, initial_kind: value, initial: crate::CssContentValue::Normal };
            StringSet, "string-set", [], "official.property.string-set", crate::CssStringSet, parse_string_set, { parse_string_set($input)? }, expansion = longhand { value: crate::CssStringSet, inherited: false, initial_kind: value, initial: crate::CssStringSet::none() };
            BookmarkLevel, "bookmark-level", [], "official.property.bookmark-level", crate::CssBookmarkLevel, parse_bookmark_level, { parse_bookmark_level($input, $numeric)? }, expansion = longhand { value: crate::CssBookmarkLevel, inherited: false, initial_kind: value, initial: crate::CssBookmarkLevel::none() };
            BookmarkLabel, "bookmark-label", [], "official.property.bookmark-label", crate::CssContentList, parse_bookmark_label, { parse_bookmark_label($input, $numeric)? }, expansion = longhand { value: crate::CssContentList, inherited: false, initial_kind: value, initial: crate::CssContentList::try_new(vec![crate::CssContentValueItem::Content(Some(crate::CssContentReferenceMode::Text))]).expect("one content(text) initial item") };
            BookmarkState, "bookmark-state", [], "official.property.bookmark-state", crate::CssBookmarkState, parse_bookmark_state, { parse_bookmark_state($input)? }, expansion = longhand { value: crate::CssBookmarkState, inherited: false, initial_kind: value, initial: crate::CssBookmarkState::Open };
            ContentVisibility, "content-visibility", [], "baseline.property.content-visibility", crate::CssContentVisibility, parse_content_visibility, { parse_content_visibility($input)? }, expansion = longhand { value: CssContentVisibility, inherited: false, initial_kind: value, initial: CssContentVisibility::Visible };
            ListStyleType, "list-style-type", [], "baseline.property.list-style-type", crate::CssListStyleTypeValue, parse_list_style_type, { parse_list_style_type($input, $numeric)? }, expansion = longhand { value: crate::CssListStyleTypeValue, inherited: true, initial_kind: value, initial: crate::CssListStyleTypeValue::initial() };
            ListStylePosition, "list-style-position", [], "baseline.property.list-style-position", crate::CssListStylePosition, parse_list_style_position, { parse_list_style_position($input)? }, expansion = longhand { value: CssListStylePosition, inherited: true, initial_kind: value, initial: CssListStylePosition::Outside };
            ListStyleImage, "list-style-image", [], "baseline.property.list-style-image", crate::CssImageValue, parse_list_style_image, { parse_list_style_image($input, $numeric)? }, expansion = longhand { value: CssImageValue, inherited: true, initial_kind: value, initial: CssImageValue::None };
            ListStyle, "list-style", [], "baseline.property.list-style", crate::CssListStyleValue, parse_list_style, { parse_list_style($input, $numeric)? }, expansion = shorthand { members: [ ListStylePosition => |value: &crate::CssListStyleValue| value.position(), ListStyleImage => |value: &crate::CssListStyleValue| value.image().cloned(), ListStyleType => |value: &crate::CssListStyleValue| value.style_type().cloned() ], reset_only: [] };
            MarkerSide, "marker-side", [], "official.property.marker-side", crate::CssMarkerSide, parse_marker_side, { parse_marker_side($input)? }, expansion = longhand { value: crate::CssMarkerSide, inherited: true, initial_kind: value, initial: crate::CssMarkerSide::MatchSelf };
            CounterReset, "counter-reset", [], "baseline.property.counter-reset", crate::CssCounterChangesValue, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { value: crate::CssCounterChangesValue, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            CounterIncrement, "counter-increment", [], "baseline.property.counter-increment", crate::CssCounterChangesValue, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { value: crate::CssCounterChangesValue, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            CounterSet, "counter-set", [], "baseline.property.counter-set", crate::CssCounterChangesValue, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { value: crate::CssCounterChangesValue, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            Width, "width", [], "baseline.property.width", crate::CssSizeValue, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto, logical_group: Size, mapping: Physical };
            Height, "height", [], "baseline.property.height", crate::CssSizeValue, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto, logical_group: Size, mapping: Physical };
            InlineSize, "inline-size", [], "official.property.inline-size", crate::CssSizeValue, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto, logical_group: Size, mapping: Logical };
            BlockSize, "block-size", [], "official.property.block-size", crate::CssSizeValue, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto, logical_group: Size, mapping: Logical };
            MinWidth, "min-width", [], "baseline.property.min-width", crate::CssSizeValue, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto, logical_group: MinSize, mapping: Physical };
            MinHeight, "min-height", [], "baseline.property.min-height", crate::CssSizeValue, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto, logical_group: MinSize, mapping: Physical };
            MinInlineSize, "min-inline-size", [], "official.property.min-inline-size", crate::CssSizeValue, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto, logical_group: MinSize, mapping: Logical };
            MinBlockSize, "min-block-size", [], "official.property.min-block-size", crate::CssSizeValue, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto, logical_group: MinSize, mapping: Logical };
            MaxWidth, "max-width", [], "baseline.property.max-width", crate::CssMaxSizeValue, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { value: CssMaxSizeValue, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE, logical_group: MaxSize, mapping: Physical };
            MaxHeight, "max-height", [], "baseline.property.max-height", crate::CssMaxSizeValue, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { value: CssMaxSizeValue, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE, logical_group: MaxSize, mapping: Physical };
            MaxInlineSize, "max-inline-size", [], "official.property.max-inline-size", crate::CssMaxSizeValue, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { value: CssMaxSizeValue, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE, logical_group: MaxSize, mapping: Logical };
            MaxBlockSize, "max-block-size", [], "official.property.max-block-size", crate::CssMaxSizeValue, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { value: CssMaxSizeValue, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE, logical_group: MaxSize, mapping: Logical };
            Size, "size", [], "ext.property.size", crate::CssSidePair<CssSizeValue>, parse_size_pair, { parse_size_pair($input, $numeric)? }, expansion = shorthand { members: [ Width => |value: &crate::CssSidePair<crate::CssSizeValue>| Some(value.first().clone()), Height => |value: &crate::CssSidePair<crate::CssSizeValue>| Some(value.second().clone()) ], reset_only: [] };
            MinSize, "min-size", [], "ext.property.min-size", crate::CssSidePair<CssSizeValue>, parse_size_pair, { parse_size_pair($input, $numeric)? }, expansion = shorthand { members: [ MinWidth => |value: &crate::CssSidePair<crate::CssSizeValue>| Some(value.first().clone()), MinHeight => |value: &crate::CssSidePair<crate::CssSizeValue>| Some(value.second().clone()) ], reset_only: [] };
            MaxSize, "max-size", [], "ext.property.max-size", crate::CssSidePair<CssMaxSizeValue>, parse_max_size_pair, { parse_max_size_pair($input, $numeric)? }, expansion = shorthand { members: [ MaxWidth => |value: &crate::CssSidePair<crate::CssMaxSizeValue>| Some(value.first().clone()), MaxHeight => |value: &crate::CssSidePair<crate::CssMaxSizeValue>| Some(value.second().clone()) ], reset_only: [] };
            FrameSizing, "frame-sizing", [], "ext.property.frame-sizing", crate::CssFrameSizing, parse_frame_sizing, { parse_frame_sizing($input)? }, expansion = longhand { value: CssFrameSizing, inherited: false, initial_kind: value, initial: CssFrameSizing::Auto };
            MinIntrinsicSizing, "min-intrinsic-sizing", [], "ext.property.min-intrinsic-sizing", crate::CssMinIntrinsicSizing, parse_min_intrinsic_sizing, { parse_min_intrinsic_sizing($input)? }, expansion = longhand { value: CssMinIntrinsicSizing, inherited: false, initial_kind: value, initial: CssMinIntrinsicSizing::Legacy };
            ContainIntrinsicWidth, "contain-intrinsic-width", [], "ext.property.contain-intrinsic-width", crate::CssContainIntrinsicSizeValue, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { value: CssContainIntrinsicSizeValue, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None), logical_group: ContainIntrinsicSize, mapping: Physical };
            ContainIntrinsicHeight, "contain-intrinsic-height", [], "ext.property.contain-intrinsic-height", crate::CssContainIntrinsicSizeValue, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { value: CssContainIntrinsicSizeValue, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None), logical_group: ContainIntrinsicSize, mapping: Physical };
            ContainIntrinsicInlineSize, "contain-intrinsic-inline-size", [], "ext.property.contain-intrinsic-inline-size", crate::CssContainIntrinsicSizeValue, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { value: CssContainIntrinsicSizeValue, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None), logical_group: ContainIntrinsicSize, mapping: Logical };
            ContainIntrinsicBlockSize, "contain-intrinsic-block-size", [], "ext.property.contain-intrinsic-block-size", crate::CssContainIntrinsicSizeValue, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { value: CssContainIntrinsicSizeValue, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None), logical_group: ContainIntrinsicSize, mapping: Logical };
            ContainIntrinsicSize, "contain-intrinsic-size", [], "ext.property.contain-intrinsic-size", crate::CssSidePair<CssContainIntrinsicSizeValue>, parse_contain_intrinsic_size, { parse_contain_intrinsic_size($input, $numeric)? }, expansion = shorthand { members: [ ContainIntrinsicWidth => |value: &crate::CssSidePair<CssContainIntrinsicSizeValue>| Some(value.first().clone()), ContainIntrinsicHeight => |value: &crate::CssSidePair<CssContainIntrinsicSizeValue>| Some(value.second().clone()) ], reset_only: [] };
            FlexBasis, "flex-basis", ["-webkit-flex-basis"], "baseline.property.flex-basis", crate::CssFlexBasisValue, parse_flex_basis, { parse_flex_basis($input, $numeric)? }, expansion = longhand { value: CssFlexBasisValue, inherited: false, initial_kind: value, initial: CssFlexBasisValue::from(CssSizeValue::Auto) };
            Gap, "gap", ["grid-gap"], "baseline.property.gap", crate::CssSidePair<CssGapValue>, parse_gap_shorthand, { parse_gap_shorthand($input, $numeric)? }, expansion = shorthand { members: [ RowGap => |value: &crate::CssSidePair<crate::CssGapValue>| Some(value.first().clone()), ColumnGap => |value: &crate::CssSidePair<crate::CssGapValue>| Some(value.second().clone()) ], reset_only: [] };
            RowGap, "row-gap", ["grid-row-gap"], "baseline.property.row-gap", crate::CssGapValue, parse_gap_value, { parse_gap_value($input, $numeric)? }, expansion = longhand { value: CssGapValue, inherited: false, initial_kind: value, initial: CssGapValue::Normal };
            ColumnGap, "column-gap", ["grid-column-gap"], "baseline.property.column-gap", crate::CssGapValue, parse_gap_value, { parse_gap_value($input, $numeric)? }, expansion = longhand { value: CssGapValue, inherited: false, initial_kind: value, initial: CssGapValue::Normal };
            ColumnCount, "column-count", [], "official.property.column-count", crate::CssColumnCount, parse_column_count, { parse_column_count($input, $numeric)? }, expansion = longhand { value: CssColumnCount, inherited: false, initial_kind: value, initial: CssColumnCount::Auto };
            ColumnFill, "column-fill", [], "official.property.column-fill", crate::CssColumnFill, parse_column_fill, { parse_column_fill($input)? }, expansion = longhand { value: CssColumnFill, inherited: false, initial_kind: value, initial: CssColumnFill::Balance };
            ColumnRule, "column-rule", [], "official.property.column-rule", crate::CssColumnRule, parse_column_rule, { parse_column_rule($input, $numeric)? }, expansion = shorthand { members: [ ColumnRuleWidth => |value: &CssColumnRule| value.width().cloned(), ColumnRuleStyle => |value: &CssColumnRule| value.style(), ColumnRuleColor => |value: &CssColumnRule| value.color().cloned() ], reset_only: [] };
            ColumnRuleColor, "column-rule-color", [], "official.property.column-rule-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            ColumnRuleStyle, "column-rule-style", [], "official.property.column-rule-style", crate::CssLineStyle, parse_line_style, { parse_line_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None };
            ColumnRuleWidth, "column-rule-width", [], "official.property.column-rule-width", crate::CssLineWidth, parse_line_width, { parse_line_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium };
            ColumnSpan, "column-span", [], "official.property.column-span", crate::CssColumnSpan, parse_column_span, { parse_column_span($input)? }, expansion = longhand { value: CssColumnSpan, inherited: false, initial_kind: value, initial: CssColumnSpan::None };
            ColumnWidth, "column-width", [], "official.property.column-width", crate::CssSizeValue, parse_column_width, { parse_column_width($input, $numeric)? }, expansion = longhand { value: CssSizeValue, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            Columns, "columns", [], "official.property.columns", crate::CssColumns, parse_columns, { parse_columns($input, $numeric)? }, expansion = shorthand { members: [ ColumnWidth => |value: &CssColumns| Some(value.width().clone()), ColumnCount => |value: &CssColumns| Some(value.count().clone()) ], reset_only: [] };
            ItemDirection, "item-direction", [], "ext.property.item-direction", crate::CssItemDirection, parse_item_direction, { parse_item_direction($input)? }, expansion = longhand { value: crate::CssItemDirection, inherited: false, initial_kind: value, initial: crate::CssItemDirection::Auto };
            ItemWrap, "item-wrap", [], "ext.property.item-wrap", crate::CssItemWrap, parse_item_wrap, { parse_item_wrap($input)? }, expansion = longhand { value: crate::CssItemWrap, inherited: false, initial_kind: value, initial: crate::CssItemWrap::default() };
            ItemPack, "item-pack", [], "ext.property.item-pack", crate::CssItemPack, parse_item_pack, { parse_item_pack($input)? }, expansion = longhand { value: crate::CssItemPack, inherited: false, initial_kind: value, initial: crate::CssItemPack::Normal };
            ItemFlow, "item-flow", [], "ext.property.item-flow", crate::CssItemFlow, parse_item_flow, { parse_item_flow($input, $numeric)? }, expansion = shorthand { members: [ ItemDirection => |value: &crate::CssItemFlow| Some(value.direction()), ItemWrap => |value: &crate::CssItemFlow| Some(value.wrap()), ItemPack => |value: &crate::CssItemFlow| Some(value.pack()), FlowTolerance => |value: &crate::CssItemFlow| Some(value.tolerance().clone()) ], reset_only: [] };
            FlowTolerance, "flow-tolerance", [], "ext.property.flow-tolerance", crate::CssFlowTolerance, parse_flow_tolerance, { parse_flow_tolerance($input, $numeric)? }, expansion = longhand { value: CssFlowTolerance, inherited: false, initial_kind: value, initial: CssFlowTolerance::normal() };
            GridTemplateRows, "grid-template-rows", [], "baseline.property.grid-template-rows", crate::CssGridTrackList, parse_grid_track_list, { parse_grid_track_list($input, $numeric)? }, expansion = longhand { value: CssGridTrackList, inherited: false, initial_kind: value, initial: CssGridTrackList::none() };
            GridTemplateColumns, "grid-template-columns", [], "baseline.property.grid-template-columns", crate::CssGridTrackList, parse_grid_track_list, { parse_grid_track_list($input, $numeric)? }, expansion = longhand { value: CssGridTrackList, inherited: false, initial_kind: value, initial: CssGridTrackList::none() };
            GridTemplateAreas, "grid-template-areas", [], "baseline.property.grid-template-areas", crate::CssGridTemplateAreas, parse_grid_template_areas, { parse_grid_template_areas($input)? }, expansion = longhand { value: crate::CssGridTemplateAreas, inherited: false, initial_kind: value, initial: crate::CssGridTemplateAreas::None };
            GridTemplate, "grid-template", [], "baseline.property.grid-template", crate::CssGridTemplate, parse_grid_template, { parse_grid_template($input, $numeric)? }, expansion = shorthand { members: [ GridTemplateRows => |value: &CssGridTemplate| crate::expansion::grid_template_rows(value), GridTemplateColumns => |value: &CssGridTemplate| crate::expansion::grid_template_columns(value), GridTemplateAreas => |value: &CssGridTemplate| crate::expansion::grid_template_areas(value) ], reset_only: [] };
            GridAutoRows, "grid-auto-rows", [], "baseline.property.grid-auto-rows", crate::CssGridTrackSizeList, parse_grid_auto_track_sizes, { parse_grid_auto_track_sizes($input, $numeric)? }, expansion = longhand { value: CssGridTrackSizeList, inherited: false, initial_kind: value, initial: CssGridTrackSizeList::try_new(vec![CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto())]).expect("one auto initial track size") };
            GridAutoColumns, "grid-auto-columns", [], "baseline.property.grid-auto-columns", crate::CssGridTrackSizeList, parse_grid_auto_track_sizes, { parse_grid_auto_track_sizes($input, $numeric)? }, expansion = longhand { value: CssGridTrackSizeList, inherited: false, initial_kind: value, initial: CssGridTrackSizeList::try_new(vec![CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto())]).expect("one auto initial track size") };
            GridAutoFlow, "grid-auto-flow", [], "baseline.property.grid-auto-flow", crate::CssGridAutoFlow, parse_grid_auto_flow, { parse_grid_auto_flow($input)? }, expansion = longhand { value: CssGridAutoFlow, inherited: false, initial_kind: value, initial: CssGridAutoFlow::Normal };
            GridRowStart, "grid-row-start", [], "baseline.property.grid-row-start", crate::CssGridLine, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { value: CssGridLine, inherited: false, initial_kind: value, initial: CssGridLine::Auto };
            GridRowEnd, "grid-row-end", [], "baseline.property.grid-row-end", crate::CssGridLine, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { value: CssGridLine, inherited: false, initial_kind: value, initial: CssGridLine::Auto };
            GridColumnStart, "grid-column-start", [], "baseline.property.grid-column-start", crate::CssGridLine, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { value: CssGridLine, inherited: false, initial_kind: value, initial: CssGridLine::Auto };
            GridColumnEnd, "grid-column-end", [], "baseline.property.grid-column-end", crate::CssGridLine, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { value: CssGridLine, inherited: false, initial_kind: value, initial: CssGridLine::Auto };
            GridRow, "grid-row", [], "baseline.property.grid-row", crate::CssGridLineRange, parse_grid_line_range, { parse_grid_line_range($input, $numeric)? }, expansion = shorthand { members: [ GridRowStart => |value: &CssGridLineRange| Some(value.start().clone()), GridRowEnd => |value: &CssGridLineRange| Some(value.effective_end()) ], reset_only: [] };
            GridColumn, "grid-column", [], "baseline.property.grid-column", crate::CssGridLineRange, parse_grid_line_range, { parse_grid_line_range($input, $numeric)? }, expansion = shorthand { members: [ GridColumnStart => |value: &CssGridLineRange| Some(value.start().clone()), GridColumnEnd => |value: &CssGridLineRange| Some(value.effective_end()) ], reset_only: [] };
            GridArea, "grid-area", [], "baseline.property.grid-area", crate::CssGridArea, parse_grid_area, { parse_grid_area($input, $numeric)? }, expansion = shorthand { members: [ GridRowStart => |value: &CssGridArea| Some(value.row_start().clone()), GridColumnStart => |value: &CssGridArea| Some(value.effective_column_start()), GridRowEnd => |value: &CssGridArea| Some(value.effective_row_end()), GridColumnEnd => |value: &CssGridArea| Some(value.effective_column_end()) ], reset_only: [] };
            Grid, "grid", [], "baseline.property.grid", crate::CssGrid, parse_grid, { parse_grid($input, $numeric)? }, expansion = shorthand { members: [ GridTemplateRows => |value: &CssGrid| crate::expansion::grid_rows(value), GridTemplateColumns => |value: &CssGrid| crate::expansion::grid_columns(value), GridTemplateAreas => |value: &CssGrid| crate::expansion::grid_areas(value), GridAutoRows => |value: &CssGrid| crate::expansion::grid_auto_rows(value), GridAutoColumns => |value: &CssGrid| crate::expansion::grid_auto_columns(value), GridAutoFlow => |value: &CssGrid| crate::expansion::grid_auto_flow(value) ], reset_only: [] };
            FontSize, "font-size", [], "baseline.property.font-size", crate::CssFontSize, parse_font_size, { parse_font_size($input, $numeric)? }, expansion = longhand { value: CssFontSize, inherited: true, initial_kind: value, initial: CssFontSize::Medium };
            LineHeight, "line-height", [], "baseline.property.line-height", crate::CssLineHeight, parse_line_height, { parse_line_height($input, $numeric)? }, expansion = longhand { value: CssLineHeight, inherited: true, initial_kind: value, initial: CssLineHeight::Normal };
            TextCombineUpright, "text-combine-upright", [], "official.property.text-combine-upright", crate::CssTextCombineUpright, parse_text_combine_upright, { parse_text_combine_upright($input, $numeric)? }, expansion = longhand { value: CssTextCombineUpright, inherited: true, initial_kind: value, initial: CssTextCombineUpright::None };
            TextOrientation, "text-orientation", [], "official.property.text-orientation", crate::CssTextOrientation, parse_text_orientation, { parse_text_orientation($input)? }, expansion = longhand { value: CssTextOrientation, inherited: true, initial_kind: value, initial: CssTextOrientation::Mixed };
            UnicodeBidi, "unicode-bidi", [], "official.property.unicode-bidi", crate::CssUnicodeBidi, parse_unicode_bidi, { parse_unicode_bidi($input)? }, expansion = longhand { value: CssUnicodeBidi, inherited: false, initial_kind: value, initial: CssUnicodeBidi::Normal };
            WritingMode, "writing-mode", [], "baseline.property.writing-mode", crate::CssWritingMode, parse_writing_mode, { parse_writing_mode($input)? }, expansion = longhand { value: CssWritingMode, inherited: true, initial_kind: value, initial: CssWritingMode::HorizontalTb };
            TextAlign, "text-align", [], "baseline.property.text-align", crate::CssTextAlignValue, parse_text_align, { parse_text_align($input, $numeric)? }, expansion = shorthand { members: [ TextAlignAll => |value: &CssTextAlignValue| Some(crate::text_alignment::shorthand_all(value)), TextAlignLast => |value: &CssTextAlignValue| Some(crate::text_alignment::shorthand_last(value)) ], reset_only: [] };
            TextAlignAll, "text-align-all", [], "official.property.text-align-all", crate::CssTextAlignAllValue, parse_text_align_all, { parse_text_align_all($input, $numeric)? }, expansion = longhand { value: CssTextAlignAllValue, inherited: true, initial_kind: value, initial: CssTextAlignAllValue::initial() };
            TextAlignLast, "text-align-last", [], "baseline.property.text-align-last", crate::CssTextAlignLastValue, parse_text_align_last, { parse_text_align_last($input)? }, expansion = longhand { value: CssTextAlignLastValue, inherited: true, initial_kind: value, initial: CssTextAlignLastValue::initial() };
            TextIndent, "text-indent", [], "baseline.property.text-indent", crate::CssTextIndent, parse_text_indent, { parse_text_indent($input, $numeric)? }, expansion = longhand { value: CssTextIndent, inherited: true, initial_kind: value, initial: CssTextIndent::new(CssNumeric::<CssLengthPercentage>::exact_zero(), false, false) };
            VerticalAlign, "vertical-align", [], "baseline.property.vertical-align", crate::CssVerticalAlign, parse_vertical_align, { parse_vertical_align($input, $numeric)? }, expansion = longhand { value: CssVerticalAlign, inherited: false, initial_kind: value, initial: CssVerticalAlign::Baseline };
            FontFamily, "font-family", [], "baseline.property.font-family", crate::CssFontFamilyList, parse_font_family_list, { parse_font_family_list($input)? }, expansion = longhand { value: CssFontFamilyList, inherited: true, initial_kind: user_agent, initial: CssUserAgentInitial::FontFamily };
            Font, "font", [], "baseline.property.font", crate::CssFontValue, parse_font, { parse_font($input, $numeric)? }, expansion = shorthand { members: [ FontFamily => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => Some(font.families().clone()), CssFontValue::System(_) => None }, FontSize => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => Some(font.size().clone()), CssFontValue::System(_) => None }, FontWidth => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.stretch().map(CssFontWidth::Keyword), CssFontValue::System(_) => None }, FontStyle => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.style().cloned(), CssFontValue::System(_) => None }, FontVariantCaps => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.variant().map(|variant| match variant { CssFontVariant::Normal => CssFontVariantCaps::Normal, CssFontVariant::SmallCaps => CssFontVariantCaps::SmallCaps }), CssFontValue::System(_) => None }, FontWeight => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.weight().cloned(), CssFontValue::System(_) => None }, LineHeight => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.line_height().cloned(), CssFontValue::System(_) => None } ], reset_only: [FontFeatureSettings, FontKerning, FontLanguageOverride, FontOpticalSizing, FontSizeAdjust, FontVariantAlternates, FontVariantEastAsian, FontVariantEmoji, FontVariantLigatures, FontVariantNumeric, FontVariantPosition, FontVariationSettings] };
            FontWeight, "font-weight", [], "baseline.property.font-weight", crate::CssFontWeight, parse_font_weight, { parse_font_weight($input, $numeric)? }, expansion = longhand { value: CssFontWeight, inherited: true, initial_kind: value, initial: CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal) };
            FontStyle, "font-style", [], "baseline.property.font-style", crate::CssFontStyle, parse_font_style, { parse_font_style($input, $numeric)? }, expansion = longhand { value: CssFontStyle, inherited: true, initial_kind: value, initial: CssFontStyle::Keyword(CssFontStyleKeyword::Normal) };
            FontWidth, "font-width", ["font-stretch"], "baseline.property.font-stretch", crate::CssFontWidth, parse_font_width, { parse_font_width($input, $numeric)? }, expansion = longhand { value: CssFontWidth, inherited: true, initial_kind: value, initial: CssFontWidth::Keyword(CssFontWidthKeyword::Normal) };
            FontVariant, "font-variant", [], "baseline.property.font-variant", crate::CssFontVariantValue, parse_font_variant, { parse_font_variant($input)? }, expansion = shorthand { members: [ FontVariantLigatures => |value: &CssFontVariantValue| Some(value.expanded_ligatures()), FontVariantCaps => |value: &CssFontVariantValue| Some(value.expanded_caps()), FontVariantAlternates => |value: &CssFontVariantValue| Some(value.expanded_alternates()), FontVariantNumeric => |value: &CssFontVariantValue| Some(value.expanded_numeric()), FontVariantEastAsian => |value: &CssFontVariantValue| Some(value.expanded_east_asian()), FontVariantPosition => |value: &CssFontVariantValue| Some(value.expanded_position()), FontVariantEmoji => |value: &CssFontVariantValue| Some(value.expanded_emoji()) ], reset_only: [] };
            FontVariantCaps, "font-variant-caps", [], "official.property.font-variant-caps", crate::CssFontVariantCaps, parse_font_variant_caps, { parse_font_variant_caps($input)? }, expansion = longhand { value: CssFontVariantCaps, inherited: true, initial_kind: value, initial: CssFontVariantCaps::Normal };
            FontVariantEastAsian, "font-variant-east-asian", [], "official.property.font-variant-east-asian", crate::CssFontVariantEastAsian, parse_font_variant_east_asian, { parse_font_variant_east_asian($input)? }, expansion = longhand { value: CssFontVariantEastAsian, inherited: true, initial_kind: value, initial: CssFontVariantEastAsian::Normal };
            FontVariantLigatures, "font-variant-ligatures", [], "official.property.font-variant-ligatures", crate::CssFontVariantLigatures, parse_font_variant_ligatures, { parse_font_variant_ligatures($input)? }, expansion = longhand { value: CssFontVariantLigatures, inherited: true, initial_kind: value, initial: CssFontVariantLigatures::Normal };
            FontVariantNumeric, "font-variant-numeric", [], "official.property.font-variant-numeric", crate::CssFontVariantNumeric, parse_font_variant_numeric, { parse_font_variant_numeric($input)? }, expansion = longhand { value: CssFontVariantNumeric, inherited: true, initial_kind: value, initial: CssFontVariantNumeric::Normal };
            FontVariantPosition, "font-variant-position", [], "official.property.font-variant-position", crate::CssFontVariantPosition, parse_font_variant_position, { parse_font_variant_position($input)? }, expansion = longhand { value: CssFontVariantPosition, inherited: true, initial_kind: value, initial: CssFontVariantPosition::Normal };
            FontVariantAlternates, "font-variant-alternates", [], "official.property.font-variant-alternates", crate::CssFontVariantAlternates, parse_font_variant_alternates, { parse_font_variant_alternates($input)? }, expansion = longhand { value: CssFontVariantAlternates, inherited: true, initial_kind: value, initial: CssFontVariantAlternates::Normal };
            FontVariantEmoji, "font-variant-emoji", [], "official.property.font-variant-emoji", crate::CssFontVariantEmoji, parse_font_variant_emoji, { parse_font_variant_emoji($input)? }, expansion = longhand { value: CssFontVariantEmoji, inherited: true, initial_kind: value, initial: CssFontVariantEmoji::Normal };
            FontFeatureSettings, "font-feature-settings", [], "baseline.property.font-feature-settings", crate::CssAuthoredFontFeatureSettings, parse_font_feature_settings, { parse_font_feature_settings($input, $numeric)? }, expansion = longhand { value: CssAuthoredFontFeatureSettings, inherited: true, initial_kind: value, initial: CssAuthoredFontFeatureSettings::Normal };
            FontKerning, "font-kerning", [], "official.property.font-kerning", crate::CssFontKerning, parse_font_kerning, { parse_font_kerning($input)? }, expansion = longhand { value: CssFontKerning, inherited: true, initial_kind: value, initial: CssFontKerning::Auto };
            FontSizeAdjust, "font-size-adjust", [], "official.property.font-size-adjust", crate::CssFontSizeAdjust, parse_font_size_adjust, { parse_font_size_adjust($input, $numeric)? }, expansion = longhand { value: CssFontSizeAdjust, inherited: true, initial_kind: value, initial: CssFontSizeAdjust::None };
            FontLanguageOverride, "font-language-override", [], "official.property.font-language-override", crate::CssFontLanguageOverride, parse_font_language_override, { parse_font_language_override($input, $numeric)? }, expansion = longhand { value: CssFontLanguageOverride, inherited: true, initial_kind: value, initial: CssFontLanguageOverride::Normal };
            FontOpticalSizing, "font-optical-sizing", [], "official.property.font-optical-sizing", crate::CssFontOpticalSizing, parse_font_optical_sizing, { parse_font_optical_sizing($input)? }, expansion = longhand { value: CssFontOpticalSizing, inherited: true, initial_kind: value, initial: CssFontOpticalSizing::Auto };
            FontVariationSettings, "font-variation-settings", [], "official.property.font-variation-settings", crate::CssFontVariationSettings, parse_font_variation_settings, { parse_font_variation_settings($input, $numeric)? }, expansion = longhand { value: CssFontVariationSettings, inherited: true, initial_kind: value, initial: CssFontVariationSettings::Normal };
            FontSynthesis, "font-synthesis", [], "official.property.font-synthesis", crate::CssFontSynthesis, parse_font_synthesis, { parse_font_synthesis($input)? }, expansion = shorthand { members: [ FontSynthesisWeight => |value: &CssFontSynthesis| Some(value.expanded_weight()), FontSynthesisStyle => |value: &CssFontSynthesis| Some(value.expanded_style()), FontSynthesisSmallCaps => |value: &CssFontSynthesis| Some(value.expanded_small_caps()), FontSynthesisPosition => |value: &CssFontSynthesis| Some(value.expanded_position()) ], reset_only: [] };
            FontSynthesisWeight, "font-synthesis-weight", [], "official.property.font-synthesis-weight", crate::CssFontSynthesisWeight, parse_font_synthesis_weight, { parse_font_synthesis_weight($input)? }, expansion = longhand { value: CssFontSynthesisWeight, inherited: true, initial_kind: value, initial: CssFontSynthesisWeight::Auto };
            FontSynthesisStyle, "font-synthesis-style", [], "official.property.font-synthesis-style", crate::CssFontSynthesisStyle, parse_font_synthesis_style, { parse_font_synthesis_style($input)? }, expansion = longhand { value: CssFontSynthesisStyle, inherited: true, initial_kind: value, initial: CssFontSynthesisStyle::Auto };
            FontSynthesisSmallCaps, "font-synthesis-small-caps", [], "official.property.font-synthesis-small-caps", crate::CssFontSynthesisSmallCaps, parse_font_synthesis_small_caps, { parse_font_synthesis_small_caps($input)? }, expansion = longhand { value: CssFontSynthesisSmallCaps, inherited: true, initial_kind: value, initial: CssFontSynthesisSmallCaps::Auto };
            FontSynthesisPosition, "font-synthesis-position", [], "official.property.font-synthesis-position", crate::CssFontSynthesisPosition, parse_font_synthesis_position, { parse_font_synthesis_position($input)? }, expansion = longhand { value: CssFontSynthesisPosition, inherited: true, initial_kind: value, initial: CssFontSynthesisPosition::Auto };
            FontPalette, "font-palette", [], "official.property.font-palette", crate::CssFontPalette, parse_font_palette, { parse_font_palette($input, $numeric)? }, expansion = longhand { value: CssFontPalette, inherited: true, initial_kind: value, initial: CssFontPalette::Normal };
            LetterSpacing, "letter-spacing", [], "baseline.property.letter-spacing", crate::CssTextSpacingAdjustment, parse_letter_spacing, { parse_letter_spacing($input, $numeric)? }, expansion = longhand { value: crate::CssTextSpacingAdjustment, inherited: true, initial_kind: value, initial: crate::CssTextSpacingAdjustment::Normal };
            TextWrap, "text-wrap", [], "baseline.property.text-wrap", crate::CssTextWrap, parse_text_wrap, { parse_text_wrap($input)? }, expansion = shorthand { members: [ TextWrapMode => |value: &CssTextWrap| value.mode(), TextWrapStyle => |value: &CssTextWrap| value.style() ], reset_only: [] };
            TextWrapMode, "text-wrap-mode", [], "ext.property.text-wrap-mode", crate::CssTextWrapMode, parse_text_wrap_mode, { parse_text_wrap_mode($input)? }, expansion = longhand { value: CssTextWrapMode, inherited: true, initial_kind: value, initial: CssTextWrapMode::Wrap };
            TextWrapStyle, "text-wrap-style", [], "ext.property.text-wrap-style", crate::CssTextWrapStyle, parse_text_wrap_style, { parse_text_wrap_style($input)? }, expansion = longhand { value: CssTextWrapStyle, inherited: true, initial_kind: value, initial: CssTextWrapStyle::Auto };
            WhiteSpace, "white-space", [], "baseline.property.white-space", crate::CssWhiteSpace, parse_white_space, { parse_white_space($input)? }, expansion = shorthand { members: [ WhiteSpaceCollapse => |value: &CssWhiteSpace| crate::expansion::white_space_collapse(value), TextWrapMode => |value: &CssWhiteSpace| crate::expansion::white_space_mode(value), WhiteSpaceTrim => |value: &CssWhiteSpace| crate::expansion::white_space_trim(value) ], reset_only: [] };
            WhiteSpaceCollapse, "white-space-collapse", [], "ext.property.white-space-collapse", crate::CssWhiteSpaceCollapse, parse_white_space_collapse, { parse_white_space_collapse($input)? }, expansion = longhand { value: CssWhiteSpaceCollapse, inherited: true, initial_kind: value, initial: CssWhiteSpaceCollapse::Collapse };
            WhiteSpaceTrim, "white-space-trim", [], "ext.property.white-space-trim", crate::CssWhiteSpaceTrim, parse_white_space_trim, { parse_white_space_trim($input)? }, expansion = longhand { value: CssWhiteSpaceTrim, inherited: false, initial_kind: value, initial: CssWhiteSpaceTrim::none() };
            WordBreak, "word-break", [], "baseline.property.word-break", crate::CssWordBreak, parse_word_break, { parse_word_break($input)? }, expansion = longhand { value: CssWordBreak, inherited: true, initial_kind: value, initial: CssWordBreak::Normal };
            OverflowWrap, "overflow-wrap", ["word-wrap"], "baseline.property.overflow-wrap", crate::CssOverflowWrap, parse_overflow_wrap, { parse_overflow_wrap($input)? }, expansion = longhand { value: CssOverflowWrap, inherited: true, initial_kind: value, initial: CssOverflowWrap::Normal };
            TextOverflow, "text-overflow", [], "baseline.property.text-overflow", crate::CssTextOverflow, parse_text_overflow, { parse_text_overflow($input)? }, expansion = longhand { value: CssTextOverflow, inherited: false, initial_kind: value, initial: CssTextOverflow::Clip };
            TextDecoration, "text-decoration", [], "baseline.property.text-decoration", crate::CssTextDecoration, parse_text_decoration, { parse_text_decoration($input, $numeric)? }, expansion = shorthand { members: [ TextDecorationLine => |value: &CssTextDecoration| value.line().cloned(), TextDecorationThickness => |value: &CssTextDecoration| value.thickness().cloned(), TextDecorationStyle => |value: &CssTextDecoration| value.style(), TextDecorationColor => |value: &CssTextDecoration| value.color().cloned() ], reset_only: [] };
            TextDecorationLine, "text-decoration-line", [], "baseline.property.text-decoration-line", crate::CssTextDecorationLine, parse_text_decoration_line, { parse_text_decoration_line($input)? }, expansion = longhand { value: CssTextDecorationLine, inherited: false, initial_kind: value, initial: CssTextDecorationLine::none() };
            TextDecorationColor, "text-decoration-color", [], "baseline.property.text-decoration-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            TextDecorationStyle, "text-decoration-style", [], "baseline.property.text-decoration-style", crate::CssTextDecorationStyle, parse_text_decoration_style, { parse_text_decoration_style($input)? }, expansion = longhand { value: CssTextDecorationStyle, inherited: false, initial_kind: value, initial: CssTextDecorationStyle::Solid };
            TextDecorationThickness, "text-decoration-thickness", [], "baseline.property.text-decoration-thickness", crate::CssTextDecorationThickness, parse_text_decoration_thickness, { parse_text_decoration_thickness($input, $numeric)? }, expansion = longhand { value: CssTextDecorationThickness, inherited: false, initial_kind: value, initial: CssTextDecorationThickness::Auto };
            TextUnderlinePosition, "text-underline-position", [], "ext.property.text-underline-position", crate::CssTextUnderlinePosition, parse_text_underline_position, { parse_text_underline_position($input)? }, expansion = longhand { value: CssTextUnderlinePosition, inherited: true, initial_kind: value, initial: CssTextUnderlinePosition::Auto };
            TextUnderlineOffset, "text-underline-offset", [], "ext.property.text-underline-offset", crate::CssTextUnderlineOffset, parse_text_underline_offset, { parse_text_underline_offset($input, $numeric)? }, expansion = longhand { value: CssTextUnderlineOffset, inherited: true, initial_kind: value, initial: CssTextUnderlineOffset::Auto };
            TextDecorationSkip, "text-decoration-skip", [], "ext.property.text-decoration-skip", crate::CssTextDecorationSkip, parse_text_decoration_skip, { parse_text_decoration_skip($input)? }, expansion = shorthand { members: [ TextDecorationSkipSelf => |value: &CssTextDecorationSkip| Some(value.expanded_self()), TextDecorationSkipBox => |value: &CssTextDecorationSkip| Some(value.expanded_box()), TextDecorationSkipInset => |value: &CssTextDecorationSkip| Some(value.expanded_inset()), TextDecorationSkipSpaces => |value: &CssTextDecorationSkip| Some(value.expanded_spaces()), TextDecorationSkipInk => |value: &CssTextDecorationSkip| Some(value.expanded_ink()) ], reset_only: [] };
            TextDecorationSkipSelf, "text-decoration-skip-self", [], "ext.property.text-decoration-skip-self", crate::CssTextDecorationSkipSelf, parse_text_decoration_skip_self, { parse_text_decoration_skip_self($input)? }, expansion = longhand { value: CssTextDecorationSkipSelf, inherited: true, initial_kind: value, initial: CssTextDecorationSkipSelf::Objects };
            TextDecorationSkipBox, "text-decoration-skip-box", [], "ext.property.text-decoration-skip-box", crate::CssTextDecorationSkipBox, parse_text_decoration_skip_box, { parse_text_decoration_skip_box($input)? }, expansion = longhand { value: CssTextDecorationSkipBox, inherited: true, initial_kind: value, initial: CssTextDecorationSkipBox::None };
            TextDecorationSkipInset, "text-decoration-skip-inset", [], "ext.property.text-decoration-skip-inset", crate::CssTextDecorationSkipInset, parse_text_decoration_skip_inset, { parse_text_decoration_skip_inset($input)? }, expansion = longhand { value: CssTextDecorationSkipInset, inherited: true, initial_kind: value, initial: CssTextDecorationSkipInset::None };
            TextDecorationSkipSpaces, "text-decoration-skip-spaces", [], "ext.property.text-decoration-skip-spaces", crate::CssTextDecorationSkipSpaces, parse_text_decoration_skip_spaces, { parse_text_decoration_skip_spaces($input)? }, expansion = longhand { value: CssTextDecorationSkipSpaces, inherited: true, initial_kind: value, initial: CssTextDecorationSkipSpaces::StartEnd };
            TextDecorationSkipInk, "text-decoration-skip-ink", [], "ext.property.text-decoration-skip-ink", crate::CssTextDecorationSkipInk, parse_text_decoration_skip_ink, { parse_text_decoration_skip_ink($input)? }, expansion = longhand { value: CssTextDecorationSkipInk, inherited: true, initial_kind: value, initial: CssTextDecorationSkipInk::Auto };
            TextEmphasis, "text-emphasis", [], "ext.property.text-emphasis", crate::CssTextEmphasis, parse_text_emphasis, { parse_text_emphasis($input, $numeric)? }, expansion = shorthand { members: [ TextEmphasisStyle => |value: &CssTextEmphasis| value.style().cloned(), TextEmphasisColor => |value: &CssTextEmphasis| value.color().cloned() ], reset_only: [] };
            TextEmphasisStyle, "text-emphasis-style", [], "ext.property.text-emphasis-style", crate::CssTextEmphasisStyle, parse_text_emphasis_style, { parse_text_emphasis_style($input, $numeric)? }, expansion = longhand { value: CssTextEmphasisStyle, inherited: true, initial_kind: value, initial: CssTextEmphasisStyle::None };
            TextEmphasisColor, "text-emphasis-color", [], "ext.property.text-emphasis-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: true, initial_kind: value, initial: CssColor::current_color() };
            TextEmphasisPosition, "text-emphasis-position", [], "ext.property.text-emphasis-position", crate::CssTextEmphasisPosition, parse_text_emphasis_position, { parse_text_emphasis_position($input, $numeric)? }, expansion = longhand { value: CssTextEmphasisPosition, inherited: true, initial_kind: value, initial: CssTextEmphasisPosition::new(CssTextEmphasisVertical::Over, Some(CssTextSide::Right)) };
            TextEmphasisSkip, "text-emphasis-skip", [], "ext.property.text-emphasis-skip", crate::CssTextEmphasisSkip, parse_text_emphasis_skip, { parse_text_emphasis_skip($input)? }, expansion = longhand { value: CssTextEmphasisSkip, inherited: true, initial_kind: value, initial: CssTextEmphasisSkip::initial() };
            TextShadow, "text-shadow", [], "ext.property.text-shadow", crate::CssTextShadow, parse_text_shadow, { parse_text_shadow($input, $numeric)? }, expansion = longhand { value: CssTextShadow, inherited: true, initial_kind: value, initial: CssTextShadow::None };
            WrapInside, "wrap-inside", [], "ext.property.wrap-inside", crate::CssWrapInside, parse_wrap_inside, { parse_wrap_inside($input)? }, expansion = longhand { value: CssWrapInside, inherited: false, initial_kind: value, initial: CssWrapInside::Auto };
            WrapBefore, "wrap-before", [], "ext.property.wrap-before", crate::CssWrapBoundary, parse_wrap_boundary, { parse_wrap_boundary($input)? }, expansion = longhand { value: CssWrapBoundary, inherited: false, initial_kind: value, initial: CssWrapBoundary::Auto };
            WrapAfter, "wrap-after", [], "ext.property.wrap-after", crate::CssWrapBoundary, parse_wrap_boundary, { parse_wrap_boundary($input)? }, expansion = longhand { value: CssWrapBoundary, inherited: false, initial_kind: value, initial: CssWrapBoundary::Auto };
            LineBreak, "line-break", [], "ext.property.line-break", crate::CssLineBreak, parse_line_break, { parse_line_break($input)? }, expansion = longhand { value: CssLineBreak, inherited: true, initial_kind: value, initial: CssLineBreak::Auto };
            WordSpaceTransform, "word-space-transform", [], "ext.property.word-space-transform", crate::CssWordSpaceTransform, parse_word_space_transform, { parse_word_space_transform($input)? }, expansion = longhand { value: CssWordSpaceTransform, inherited: true, initial_kind: value, initial: CssWordSpaceTransform::None };
            TabSize, "tab-size", [], "ext.property.tab-size", crate::CssTabSize, parse_tab_size, { parse_tab_size($input, $numeric)? }, expansion = longhand { value: CssTabSize, inherited: true, initial_kind: value, initial: CssTabSize::Number(crate::CssNumeric::<crate::CssNumber, crate::CssNonNegative>::try_from_component(crate::CssComponentValue::try_number("8").expect("ordinary eight")).expect("nonnegative initial tab interval")) };
            Hyphens, "hyphens", [], "ext.property.hyphens", crate::CssHyphens, parse_hyphens, { parse_hyphens($input)? }, expansion = longhand { value: CssHyphens, inherited: true, initial_kind: value, initial: CssHyphens::Manual };
            HyphenateCharacter, "hyphenate-character", [], "ext.property.hyphenate-character", crate::CssHyphenateCharacter, parse_hyphenate_character, { parse_hyphenate_character($input, $numeric)? }, expansion = longhand { value: CssHyphenateCharacter, inherited: true, initial_kind: value, initial: CssHyphenateCharacter::Auto };
            HyphenateLimitZone, "hyphenate-limit-zone", [], "ext.property.hyphenate-limit-zone", crate::CssNumeric<CssLengthPercentage>, parse_hyphenate_limit_zone, { parse_hyphenate_limit_zone($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLengthPercentage>, inherited: true, initial_kind: value, initial: CssNumeric::<CssLengthPercentage>::exact_zero() };
            HyphenateLimitChars, "hyphenate-limit-chars", [], "ext.property.hyphenate-limit-chars", crate::CssHyphenateLimitChars, parse_hyphenate_limit_chars, { parse_hyphenate_limit_chars($input, $numeric)? }, expansion = longhand { value: CssHyphenateLimitChars, inherited: true, initial_kind: value, initial: CssHyphenateLimitChars::auto() };
            HyphenateLimitLines, "hyphenate-limit-lines", [], "ext.property.hyphenate-limit-lines", crate::CssHyphenateLimitLines, parse_hyphenate_limit_lines, { parse_hyphenate_limit_lines($input, $numeric)? }, expansion = longhand { value: CssHyphenateLimitLines, inherited: true, initial_kind: value, initial: CssHyphenateLimitLines::NoLimit };
            HyphenateLimitLast, "hyphenate-limit-last", [], "ext.property.hyphenate-limit-last", crate::CssHyphenateLimitLast, parse_hyphenate_limit_last, { parse_hyphenate_limit_last($input)? }, expansion = longhand { value: CssHyphenateLimitLast, inherited: true, initial_kind: value, initial: CssHyphenateLimitLast::None };
            TextJustify, "text-justify", [], "ext.property.text-justify", crate::CssTextJustify, parse_text_justify, { parse_text_justify($input)? }, expansion = longhand { value: CssTextJustify, inherited: true, initial_kind: value, initial: CssTextJustify::auto() };
            TextGroupAlign, "text-group-align", [], "ext.property.text-group-align", crate::CssTextGroupAlign, parse_text_group_align, { parse_text_group_align($input)? }, expansion = longhand { value: CssTextGroupAlign, inherited: false, initial_kind: value, initial: CssTextGroupAlign::None };
            LinePadding, "line-padding", [], "ext.property.line-padding", crate::CssNumeric<CssLength>, parse_line_padding, { parse_line_padding($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: true, initial_kind: value, initial: CssNumeric::<CssLength>::zero() };
            TextAutospace, "text-autospace", [], "ext.property.text-autospace", crate::CssTextAutospace, parse_text_autospace, { parse_text_autospace($input)? }, expansion = longhand { value: CssTextAutospace, inherited: true, initial_kind: value, initial: CssTextAutospace::Normal };
            TextSpacingTrim, "text-spacing-trim", [], "ext.property.text-spacing-trim", crate::CssTextSpacingTrim, parse_text_spacing_trim, { parse_text_spacing_trim($input)? }, expansion = longhand { value: CssTextSpacingTrim, inherited: true, initial_kind: value, initial: CssTextSpacingTrim::Trim(CssSpacingTrim::Normal) };
            HangingPunctuation, "hanging-punctuation", [], "ext.property.hanging-punctuation", crate::CssHangingPunctuation, parse_hanging_punctuation, { parse_hanging_punctuation($input)? }, expansion = longhand { value: CssHangingPunctuation, inherited: true, initial_kind: value, initial: CssHangingPunctuation::None };
            TextBoxTrim, "text-box-trim", [], "official.property.text-box-trim", crate::CssTextBoxTrim, parse_text_box_trim, { parse_text_box_trim($input)? }, expansion = longhand { value: CssTextBoxTrim, inherited: false, initial_kind: value, initial: CssTextBoxTrim::None };
            TextBoxEdge, "text-box-edge", [], "official.property.text-box-edge", crate::CssTextBoxEdge, parse_text_box_edge, { parse_text_box_edge($input)? }, expansion = longhand { value: CssTextBoxEdge, inherited: true, initial_kind: value, initial: CssTextBoxEdge::Auto };
            TextBox, "text-box", [], "official.property.text-box", crate::CssTextBox, parse_text_box, { parse_text_box($input)? }, expansion = shorthand { members: [ TextBoxTrim => crate::expansion::text_box_trim, TextBoxEdge => crate::expansion::text_box_edge ], reset_only: [] };
            TextSpacing, "text-spacing", [], "ext.property.text-spacing", crate::CssTextSpacing, parse_text_spacing, { parse_text_spacing($input)? }, expansion = shorthand { members: [ TextSpacingTrim => crate::expansion::text_spacing_trim, TextAutospace => crate::expansion::text_spacing_autospace ], reset_only: [] };
            TextTransform, "text-transform", [], "baseline.property.text-transform", crate::CssTextTransform, parse_text_transform, { parse_text_transform($input)? }, expansion = longhand { value: CssTextTransform, inherited: true, initial_kind: value, initial: CssTextTransform::None };
            Inset, "inset", [], "baseline.property.inset", crate::CssFourSides<CssInsetValue>, parse_inset_shorthand, { parse_inset_shorthand($input, $numeric)? }, expansion = four_side { mode: |value: &crate::CssFourSides<crate::CssInsetValue>| value.kind(), physical: [ Top => |value: &crate::CssFourSides<crate::CssInsetValue>| value.assigned()[0].clone(), Right => |value: &crate::CssFourSides<crate::CssInsetValue>| value.assigned()[1].clone(), Bottom => |value: &crate::CssFourSides<crate::CssInsetValue>| value.assigned()[2].clone(), Left => |value: &crate::CssFourSides<crate::CssInsetValue>| value.assigned()[3].clone() ], logical: [ InsetBlockStart => |value: &crate::CssFourSides<crate::CssInsetValue>| value.assigned()[0].clone(), InsetInlineStart => |value: &crate::CssFourSides<crate::CssInsetValue>| value.assigned()[1].clone(), InsetBlockEnd => |value: &crate::CssFourSides<crate::CssInsetValue>| value.assigned()[2].clone(), InsetInlineEnd => |value: &crate::CssFourSides<crate::CssInsetValue>| value.assigned()[3].clone() ] };
            Top, "top", [], "baseline.property.top", crate::CssInsetValue, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, inherited: false, initial_kind: value, initial: CssInsetValue::Auto, logical_group: Inset, mapping: Physical };
            Right, "right", [], "baseline.property.right", crate::CssInsetValue, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, inherited: false, initial_kind: value, initial: CssInsetValue::Auto, logical_group: Inset, mapping: Physical };
            Bottom, "bottom", [], "baseline.property.bottom", crate::CssInsetValue, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, inherited: false, initial_kind: value, initial: CssInsetValue::Auto, logical_group: Inset, mapping: Physical };
            Left, "left", [], "baseline.property.left", crate::CssInsetValue, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, inherited: false, initial_kind: value, initial: CssInsetValue::Auto, logical_group: Inset, mapping: Physical };
            InsetBlockStart, "inset-block-start", [], "official.property.inset-block-start", crate::CssInsetValue, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, inherited: false, initial_kind: value, initial: CssInsetValue::Auto, logical_group: Inset, mapping: Logical };
            InsetBlockEnd, "inset-block-end", [], "official.property.inset-block-end", crate::CssInsetValue, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, inherited: false, initial_kind: value, initial: CssInsetValue::Auto, logical_group: Inset, mapping: Logical };
            InsetInlineStart, "inset-inline-start", [], "official.property.inset-inline-start", crate::CssInsetValue, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, inherited: false, initial_kind: value, initial: CssInsetValue::Auto, logical_group: Inset, mapping: Logical };
            InsetInlineEnd, "inset-inline-end", [], "official.property.inset-inline-end", crate::CssInsetValue, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, inherited: false, initial_kind: value, initial: CssInsetValue::Auto, logical_group: Inset, mapping: Logical };
            InsetBlock, "inset-block", [], "official.property.inset-block", crate::CssSidePair<CssInsetValue>, parse_inset_pair, { parse_inset_pair($input, $numeric)? }, expansion = shorthand { members: [ InsetBlockStart => |value: &crate::CssSidePair<crate::CssInsetValue>| Some(value.first().clone()), InsetBlockEnd => |value: &crate::CssSidePair<crate::CssInsetValue>| Some(value.second().clone()) ], reset_only: [] };
            InsetInline, "inset-inline", [], "official.property.inset-inline", crate::CssSidePair<CssInsetValue>, parse_inset_pair, { parse_inset_pair($input, $numeric)? }, expansion = shorthand { members: [ InsetInlineStart => |value: &crate::CssSidePair<crate::CssInsetValue>| Some(value.first().clone()), InsetInlineEnd => |value: &crate::CssSidePair<crate::CssInsetValue>| Some(value.second().clone()) ], reset_only: [] };
            ZIndex, "z-index", [], "baseline.property.z-index", crate::CssZIndexValue, parse_z_index, { parse_z_index($input, $numeric)? }, expansion = longhand { value: CssZIndexValue, inherited: false, initial_kind: value, initial: CssZIndexValue::Auto };
            BoxDecorationBreak, "box-decoration-break", [], "baseline.property.box-decoration-break", crate::CssBoxDecorationBreak, parse_box_decoration_break, { parse_box_decoration_break($input)? }, expansion = longhand { value: CssBoxDecorationBreak, inherited: false, initial_kind: value, initial: CssBoxDecorationBreak::Slice };
            Margin, "margin", [], "baseline.property.margin", crate::CssFourSides<CssMarginValue>, parse_box_margin_shorthand, { parse_box_margin_shorthand($input, $numeric)? }, expansion = four_side { mode: |value: &crate::CssFourSides<crate::CssMarginValue>| value.kind(), physical: [ MarginTop => |value: &crate::CssFourSides<crate::CssMarginValue>| value.assigned()[0].clone(), MarginRight => |value: &crate::CssFourSides<crate::CssMarginValue>| value.assigned()[1].clone(), MarginBottom => |value: &crate::CssFourSides<crate::CssMarginValue>| value.assigned()[2].clone(), MarginLeft => |value: &crate::CssFourSides<crate::CssMarginValue>| value.assigned()[3].clone() ], logical: [ MarginBlockStart => |value: &crate::CssFourSides<crate::CssMarginValue>| value.assigned()[0].clone(), MarginInlineStart => |value: &crate::CssFourSides<crate::CssMarginValue>| value.assigned()[1].clone(), MarginBlockEnd => |value: &crate::CssFourSides<crate::CssMarginValue>| value.assigned()[2].clone(), MarginInlineEnd => |value: &crate::CssFourSides<crate::CssMarginValue>| value.assigned()[3].clone() ] };
            MarginTop, "margin-top", [], "baseline.property.margin-top", crate::CssMarginValue, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssNumeric::<CssLengthPercentage>::exact_zero()), logical_group: Margin, mapping: Physical };
            MarginRight, "margin-right", [], "baseline.property.margin-right", crate::CssMarginValue, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssNumeric::<CssLengthPercentage>::exact_zero()), logical_group: Margin, mapping: Physical };
            MarginBottom, "margin-bottom", [], "baseline.property.margin-bottom", crate::CssMarginValue, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssNumeric::<CssLengthPercentage>::exact_zero()), logical_group: Margin, mapping: Physical };
            MarginLeft, "margin-left", [], "baseline.property.margin-left", crate::CssMarginValue, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssNumeric::<CssLengthPercentage>::exact_zero()), logical_group: Margin, mapping: Physical };
            MarginBlockStart, "margin-block-start", [], "official.property.margin-block-start", crate::CssMarginValue, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssNumeric::<CssLengthPercentage>::exact_zero()), logical_group: Margin, mapping: Logical };
            MarginBlockEnd, "margin-block-end", [], "official.property.margin-block-end", crate::CssMarginValue, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssNumeric::<CssLengthPercentage>::exact_zero()), logical_group: Margin, mapping: Logical };
            MarginInlineStart, "margin-inline-start", [], "official.property.margin-inline-start", crate::CssMarginValue, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssNumeric::<CssLengthPercentage>::exact_zero()), logical_group: Margin, mapping: Logical };
            MarginInlineEnd, "margin-inline-end", [], "official.property.margin-inline-end", crate::CssMarginValue, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssNumeric::<CssLengthPercentage>::exact_zero()), logical_group: Margin, mapping: Logical };
            MarginBlock, "margin-block", [], "official.property.margin-block", crate::CssSidePair<CssMarginValue>, parse_box_margin_pair, { parse_box_margin_pair($input, $numeric)? }, expansion = shorthand { members: [ MarginBlockStart => |value: &crate::CssSidePair<crate::CssMarginValue>| Some(value.first().clone()), MarginBlockEnd => |value: &crate::CssSidePair<crate::CssMarginValue>| Some(value.second().clone()) ], reset_only: [] };
            MarginInline, "margin-inline", [], "official.property.margin-inline", crate::CssSidePair<CssMarginValue>, parse_box_margin_pair, { parse_box_margin_pair($input, $numeric)? }, expansion = shorthand { members: [ MarginInlineStart => |value: &crate::CssSidePair<crate::CssMarginValue>| Some(value.first().clone()), MarginInlineEnd => |value: &crate::CssSidePair<crate::CssMarginValue>| Some(value.second().clone()) ], reset_only: [] };
            Padding, "padding", [], "baseline.property.padding", crate::CssFourSides<CssPaddingValue>, parse_box_padding_shorthand, { parse_box_padding_shorthand($input, $numeric)? }, expansion = four_side { mode: |value: &crate::CssFourSides<crate::CssPaddingValue>| value.kind(), physical: [ PaddingTop => |value: &crate::CssFourSides<crate::CssPaddingValue>| value.assigned()[0].clone(), PaddingRight => |value: &crate::CssFourSides<crate::CssPaddingValue>| value.assigned()[1].clone(), PaddingBottom => |value: &crate::CssFourSides<crate::CssPaddingValue>| value.assigned()[2].clone(), PaddingLeft => |value: &crate::CssFourSides<crate::CssPaddingValue>| value.assigned()[3].clone() ], logical: [ PaddingBlockStart => |value: &crate::CssFourSides<crate::CssPaddingValue>| value.assigned()[0].clone(), PaddingInlineStart => |value: &crate::CssFourSides<crate::CssPaddingValue>| value.assigned()[1].clone(), PaddingBlockEnd => |value: &crate::CssFourSides<crate::CssPaddingValue>| value.assigned()[2].clone(), PaddingInlineEnd => |value: &crate::CssFourSides<crate::CssPaddingValue>| value.assigned()[3].clone() ] };
            PaddingTop, "padding-top", [], "baseline.property.padding-top", crate::CssPaddingValue, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero()), logical_group: Padding, mapping: Physical };
            PaddingRight, "padding-right", [], "baseline.property.padding-right", crate::CssPaddingValue, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero()), logical_group: Padding, mapping: Physical };
            PaddingBottom, "padding-bottom", [], "baseline.property.padding-bottom", crate::CssPaddingValue, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero()), logical_group: Padding, mapping: Physical };
            PaddingLeft, "padding-left", [], "baseline.property.padding-left", crate::CssPaddingValue, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero()), logical_group: Padding, mapping: Physical };
            PaddingBlockStart, "padding-block-start", [], "official.property.padding-block-start", crate::CssPaddingValue, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero()), logical_group: Padding, mapping: Logical };
            PaddingBlockEnd, "padding-block-end", [], "official.property.padding-block-end", crate::CssPaddingValue, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero()), logical_group: Padding, mapping: Logical };
            PaddingInlineStart, "padding-inline-start", [], "official.property.padding-inline-start", crate::CssPaddingValue, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero()), logical_group: Padding, mapping: Logical };
            PaddingInlineEnd, "padding-inline-end", [], "official.property.padding-inline-end", crate::CssPaddingValue, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero()), logical_group: Padding, mapping: Logical };
            PaddingBlock, "padding-block", [], "official.property.padding-block", crate::CssSidePair<CssPaddingValue>, parse_box_padding_pair, { parse_box_padding_pair($input, $numeric)? }, expansion = shorthand { members: [ PaddingBlockStart => |value: &crate::CssSidePair<crate::CssPaddingValue>| Some(value.first().clone()), PaddingBlockEnd => |value: &crate::CssSidePair<crate::CssPaddingValue>| Some(value.second().clone()) ], reset_only: [] };
            PaddingInline, "padding-inline", [], "official.property.padding-inline", crate::CssSidePair<CssPaddingValue>, parse_box_padding_pair, { parse_box_padding_pair($input, $numeric)? }, expansion = shorthand { members: [ PaddingInlineStart => |value: &crate::CssSidePair<crate::CssPaddingValue>| Some(value.first().clone()), PaddingInlineEnd => |value: &crate::CssSidePair<crate::CssPaddingValue>| Some(value.second().clone()) ], reset_only: [] };
            Border, "border", [], "baseline.property.border", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderTopWidth => |value: &CssBorder| value.width().cloned(), BorderRightWidth => |value: &CssBorder| value.width().cloned(), BorderBottomWidth => |value: &CssBorder| value.width().cloned(), BorderLeftWidth => |value: &CssBorder| value.width().cloned(), BorderTopStyle => |value: &CssBorder| value.style(), BorderRightStyle => |value: &CssBorder| value.style(), BorderBottomStyle => |value: &CssBorder| value.style(), BorderLeftStyle => |value: &CssBorder| value.style(), BorderTopColor => |value: &CssBorder| value.color().cloned(), BorderRightColor => |value: &CssBorder| value.color().cloned(), BorderBottomColor => |value: &CssBorder| value.color().cloned(), BorderLeftColor => |value: &CssBorder| value.color().cloned() ], reset_only: [ BorderImageSource, BorderImageSlice, BorderImageWidth, BorderImageOutset, BorderImageRepeat ] };
            BorderTop, "border-top", [], "baseline.property.border-top", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderTopWidth => |value: &CssBorder| value.width().cloned(), BorderTopStyle => |value: &CssBorder| value.style(), BorderTopColor => |value: &CssBorder| value.color().cloned() ], reset_only: [  ] };
            BorderRight, "border-right", [], "baseline.property.border-right", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderRightWidth => |value: &CssBorder| value.width().cloned(), BorderRightStyle => |value: &CssBorder| value.style(), BorderRightColor => |value: &CssBorder| value.color().cloned() ], reset_only: [  ] };
            BorderBottom, "border-bottom", [], "baseline.property.border-bottom", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderBottomWidth => |value: &CssBorder| value.width().cloned(), BorderBottomStyle => |value: &CssBorder| value.style(), BorderBottomColor => |value: &CssBorder| value.color().cloned() ], reset_only: [  ] };
            BorderLeft, "border-left", [], "baseline.property.border-left", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderLeftWidth => |value: &CssBorder| value.width().cloned(), BorderLeftStyle => |value: &CssBorder| value.style(), BorderLeftColor => |value: &CssBorder| value.color().cloned() ], reset_only: [  ] };
            BorderBlockStart, "border-block-start", [], "official.property.border-block-start", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderBlockStartWidth => |value: &CssBorder| value.width().cloned(), BorderBlockStartStyle => |value: &CssBorder| value.style(), BorderBlockStartColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderBlockEnd, "border-block-end", [], "official.property.border-block-end", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderBlockEndWidth => |value: &CssBorder| value.width().cloned(), BorderBlockEndStyle => |value: &CssBorder| value.style(), BorderBlockEndColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderInlineStart, "border-inline-start", [], "official.property.border-inline-start", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderInlineStartWidth => |value: &CssBorder| value.width().cloned(), BorderInlineStartStyle => |value: &CssBorder| value.style(), BorderInlineStartColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderInlineEnd, "border-inline-end", [], "official.property.border-inline-end", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderInlineEndWidth => |value: &CssBorder| value.width().cloned(), BorderInlineEndStyle => |value: &CssBorder| value.style(), BorderInlineEndColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderBlock, "border-block", [], "official.property.border-block", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderBlockStartWidth => |value: &CssBorder| value.width().cloned(), BorderBlockEndWidth => |value: &CssBorder| value.width().cloned(), BorderBlockStartStyle => |value: &CssBorder| value.style(), BorderBlockEndStyle => |value: &CssBorder| value.style(), BorderBlockStartColor => |value: &CssBorder| value.color().cloned(), BorderBlockEndColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderInline, "border-inline", [], "official.property.border-inline", crate::CssBorder, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { members: [ BorderInlineStartWidth => |value: &CssBorder| value.width().cloned(), BorderInlineEndWidth => |value: &CssBorder| value.width().cloned(), BorderInlineStartStyle => |value: &CssBorder| value.style(), BorderInlineEndStyle => |value: &CssBorder| value.style(), BorderInlineStartColor => |value: &CssBorder| value.color().cloned(), BorderInlineEndColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderWidth, "border-width", [], "baseline.property.border-width", crate::CssFourSides<CssLineWidth>, parse_exact_border_width_shorthand, { parse_exact_border_width_shorthand($input, $numeric)? }, expansion = four_side { mode: |value: &crate::CssFourSides<crate::CssLineWidth>| value.kind(), physical: [ BorderTopWidth => |value: &crate::CssFourSides<crate::CssLineWidth>| value.assigned()[0].clone(), BorderRightWidth => |value: &crate::CssFourSides<crate::CssLineWidth>| value.assigned()[1].clone(), BorderBottomWidth => |value: &crate::CssFourSides<crate::CssLineWidth>| value.assigned()[2].clone(), BorderLeftWidth => |value: &crate::CssFourSides<crate::CssLineWidth>| value.assigned()[3].clone() ], logical: [ BorderBlockStartWidth => |value: &crate::CssFourSides<crate::CssLineWidth>| value.assigned()[0].clone(), BorderInlineStartWidth => |value: &crate::CssFourSides<crate::CssLineWidth>| value.assigned()[1].clone(), BorderBlockEndWidth => |value: &crate::CssFourSides<crate::CssLineWidth>| value.assigned()[2].clone(), BorderInlineEndWidth => |value: &crate::CssFourSides<crate::CssLineWidth>| value.assigned()[3].clone() ] };
            BorderTopWidth, "border-top-width", [], "baseline.property.border-top-width", crate::CssLineWidth, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium, logical_group: BorderWidth, mapping: Physical };
            BorderRightWidth, "border-right-width", [], "baseline.property.border-right-width", crate::CssLineWidth, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium, logical_group: BorderWidth, mapping: Physical };
            BorderBottomWidth, "border-bottom-width", [], "baseline.property.border-bottom-width", crate::CssLineWidth, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium, logical_group: BorderWidth, mapping: Physical };
            BorderLeftWidth, "border-left-width", [], "baseline.property.border-left-width", crate::CssLineWidth, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium, logical_group: BorderWidth, mapping: Physical };
            BorderBlockStartWidth, "border-block-start-width", [], "official.property.border-block-start-width", crate::CssLineWidth, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium, logical_group: BorderWidth, mapping: Logical };
            BorderBlockEndWidth, "border-block-end-width", [], "official.property.border-block-end-width", crate::CssLineWidth, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium, logical_group: BorderWidth, mapping: Logical };
            BorderInlineStartWidth, "border-inline-start-width", [], "official.property.border-inline-start-width", crate::CssLineWidth, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium, logical_group: BorderWidth, mapping: Logical };
            BorderInlineEndWidth, "border-inline-end-width", [], "official.property.border-inline-end-width", crate::CssLineWidth, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium, logical_group: BorderWidth, mapping: Logical };
            BorderBlockWidth, "border-block-width", [], "official.property.border-block-width", crate::CssSidePair<CssLineWidth>, parse_exact_border_width_pair, { parse_exact_border_width_pair($input, $numeric)? }, expansion = shorthand { members: [ BorderBlockStartWidth => |value: &crate::CssSidePair<crate::CssLineWidth>| Some(value.first().clone()), BorderBlockEndWidth => |value: &crate::CssSidePair<crate::CssLineWidth>| Some(value.second().clone()) ], reset_only: [] };
            BorderInlineWidth, "border-inline-width", [], "official.property.border-inline-width", crate::CssSidePair<CssLineWidth>, parse_exact_border_width_pair, { parse_exact_border_width_pair($input, $numeric)? }, expansion = shorthand { members: [ BorderInlineStartWidth => |value: &crate::CssSidePair<crate::CssLineWidth>| Some(value.first().clone()), BorderInlineEndWidth => |value: &crate::CssSidePair<crate::CssLineWidth>| Some(value.second().clone()) ], reset_only: [] };
            Color, "color", [], "baseline.property.color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: true, initial_kind: value, initial: CssColor::from_system(CssSystemColor::CanvasText) };
            Background, "background", [], "baseline.property.background", crate::CssBackground, parse_background, { parse_background($input, $numeric)? }, expansion = shorthand { members: [
                BackgroundImage => crate::expansion::background_images,
                BackgroundPosition => crate::expansion::background_positions,
                BackgroundSize => crate::expansion::background_sizes,
                BackgroundRepeat => crate::expansion::background_repeats,
                BackgroundAttachment => crate::expansion::background_attachments,
                BackgroundOrigin => crate::expansion::background_origins,
                BackgroundClip => crate::expansion::background_clips,
                BackgroundColor => |value: &CssBackground| value.layers().last().and_then(CssBackgroundLayer::color).cloned()
            ], reset_only: [BackgroundBlendMode] };
            BackgroundColor, "background-color", [], "baseline.property.background-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::transparent() };
            BorderColor, "border-color", [], "baseline.property.border-color", crate::CssFourSides<CssColor>, parse_border_colors, { parse_border_colors($input, $numeric)? }, expansion = four_side { mode: |value: &crate::CssFourSides<crate::CssColor>| value.kind(), physical: [ BorderTopColor => |value: &crate::CssFourSides<crate::CssColor>| value.assigned()[0].clone(), BorderRightColor => |value: &crate::CssFourSides<crate::CssColor>| value.assigned()[1].clone(), BorderBottomColor => |value: &crate::CssFourSides<crate::CssColor>| value.assigned()[2].clone(), BorderLeftColor => |value: &crate::CssFourSides<crate::CssColor>| value.assigned()[3].clone() ], logical: [ BorderBlockStartColor => |value: &crate::CssFourSides<crate::CssColor>| value.assigned()[0].clone(), BorderInlineStartColor => |value: &crate::CssFourSides<crate::CssColor>| value.assigned()[1].clone(), BorderBlockEndColor => |value: &crate::CssFourSides<crate::CssColor>| value.assigned()[2].clone(), BorderInlineEndColor => |value: &crate::CssFourSides<crate::CssColor>| value.assigned()[3].clone() ] };
            BorderTopColor, "border-top-color", [], "baseline.property.border-top-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color(), logical_group: BorderColor, mapping: Physical };
            BorderRightColor, "border-right-color", [], "baseline.property.border-right-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color(), logical_group: BorderColor, mapping: Physical };
            BorderBottomColor, "border-bottom-color", [], "baseline.property.border-bottom-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color(), logical_group: BorderColor, mapping: Physical };
            BorderLeftColor, "border-left-color", [], "baseline.property.border-left-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color(), logical_group: BorderColor, mapping: Physical };
            BorderBlockStartColor, "border-block-start-color", [], "official.property.border-block-start-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color(), logical_group: BorderColor, mapping: Logical };
            BorderBlockEndColor, "border-block-end-color", [], "official.property.border-block-end-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color(), logical_group: BorderColor, mapping: Logical };
            BorderInlineStartColor, "border-inline-start-color", [], "official.property.border-inline-start-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color(), logical_group: BorderColor, mapping: Logical };
            BorderInlineEndColor, "border-inline-end-color", [], "official.property.border-inline-end-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::current_color(), logical_group: BorderColor, mapping: Logical };
            BorderBlockColor, "border-block-color", [], "official.property.border-block-color", crate::CssSidePair<CssColor>, parse_border_color_pair, { parse_border_color_pair($input, $numeric)? }, expansion = shorthand { members: [ BorderBlockStartColor => |value: &crate::CssSidePair<crate::CssColor>| Some(value.first().clone()), BorderBlockEndColor => |value: &crate::CssSidePair<crate::CssColor>| Some(value.second().clone()) ], reset_only: [] };
            BorderInlineColor, "border-inline-color", [], "official.property.border-inline-color", crate::CssSidePair<CssColor>, parse_border_color_pair, { parse_border_color_pair($input, $numeric)? }, expansion = shorthand { members: [ BorderInlineStartColor => |value: &crate::CssSidePair<crate::CssColor>| Some(value.first().clone()), BorderInlineEndColor => |value: &crate::CssSidePair<crate::CssColor>| Some(value.second().clone()) ], reset_only: [] };
            BackgroundImage, "background-image", [], "baseline.property.background-image", crate::CssImageValueList, parse_image_layer_list, { parse_image_layer_list($input, $numeric)? }, expansion = longhand { value: CssImageValueList, inherited: false, initial_kind: value, initial: CssImageValueList::try_new(vec![CssImageValue::None]).expect("one initial image") };
            BackgroundPosition, "background-position", [], "baseline.property.background-position", crate::CssBackgroundPositionList, parse_background_position_list, { parse_background_position_list($input, $numeric)? }, expansion = longhand { value: CssBackgroundPositionList, inherited: false, initial_kind: value, initial: {
                let zero = CssNumeric::<CssLengthPercentage>::try_from_component(crate::CssComponentValue::try_token("0%").expect("percentage token")).expect("zero percentage is a position offset");
                CssBackgroundPositionList::try_new(vec![CssBackgroundPosition::try_new(CssHorizontalPosition::Offset(zero.clone()), CssVerticalPosition::Offset(zero)).expect("two free offsets are a background position")]).expect("one initial position")
            } };
            ObjectPosition, "object-position", [], "official.property.object-position", crate::CssPosition, parse_object_position, { parse_object_position($input, $numeric)? }, expansion = longhand { value: CssPosition, inherited: false, initial_kind: value, initial: crate::expansion::initial_position() };
            BackgroundSize, "background-size", [], "baseline.property.background-size", crate::CssBackgroundSizeList, parse_background_size_list, { parse_background_size_list($input, $numeric)? }, expansion = longhand { value: CssBackgroundSizeList, inherited: false, initial_kind: value, initial: CssBackgroundSizeList::try_new(vec![CssBackgroundSize::Explicit { width: CssAutoOr::Auto, height: None }]).expect("one initial size") };
            BackgroundRepeat, "background-repeat", [], "baseline.property.background-repeat", crate::CssBackgroundRepeatList, parse_background_repeat_list, { parse_background_repeat_list($input)? }, expansion = longhand { value: CssBackgroundRepeatList, inherited: false, initial_kind: value, initial: CssBackgroundRepeatList::try_new(vec![CssBackgroundRepeat::Axes { x: CssBackgroundRepeatStyle::Repeat, y: CssBackgroundRepeatStyle::Repeat }]).expect("one initial repeat") };
            BackgroundOrigin, "background-origin", [], "baseline.property.background-origin", crate::CssBackgroundBoxList, parse_background_box_list, { parse_background_box_list($input)? }, expansion = longhand { value: CssBackgroundBoxList, inherited: false, initial_kind: value, initial: CssBackgroundBoxList::try_new(vec![CssBackgroundBox::PaddingBox]).expect("one initial origin box") };
            BackgroundClip, "background-clip", [], "baseline.property.background-clip", crate::CssBackgroundBoxList, parse_background_box_list, { parse_background_box_list($input)? }, expansion = longhand { value: CssBackgroundBoxList, inherited: false, initial_kind: value, initial: CssBackgroundBoxList::try_new(vec![CssBackgroundBox::BorderBox]).expect("one initial clip box") };
            BackgroundAttachment, "background-attachment", [], "baseline.property.background-attachment", crate::CssBackgroundAttachmentList, parse_background_attachment_list, { parse_background_attachment_list($input)? }, expansion = longhand { value: CssBackgroundAttachmentList, inherited: false, initial_kind: value, initial: CssBackgroundAttachmentList::try_new(vec![CssBackgroundAttachment::Scroll]).expect("one initial attachment") };
            BorderImage, "border-image", [], "official.property.border-image", crate::CssBorderImage, parse_border_image, { parse_border_image($input, $numeric)? }, expansion = shorthand { members: [
                BorderImageSource => |value: &CssBorderImage| value.source().cloned(),
                BorderImageSlice => |value: &CssBorderImage| value.slice().cloned(),
                BorderImageWidth => |value: &CssBorderImage| value.width().cloned(),
                BorderImageOutset => |value: &CssBorderImage| value.outset().cloned(),
                BorderImageRepeat => |value: &CssBorderImage| value.repeat()
            ], reset_only: [] };
            BorderImageOutset, "border-image-outset", [], "official.property.border-image-outset", crate::CssBorderImageOutset, parse_border_image_outset, { parse_border_image_outset($input, $numeric)? }, expansion = longhand { value: CssBorderImageOutset, inherited: false, initial_kind: value, initial: CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Number(crate::CssNumeric::<crate::CssNumber, crate::CssNonNegative>::try_from_component(crate::CssComponentValue::try_number("0").expect("number token")).expect("0 is non-negative"))]).expect("one outset component is valid") };
            BorderImageRepeat, "border-image-repeat", [], "official.property.border-image-repeat", crate::CssBorderImageRepeat, parse_border_image_repeat, { parse_border_image_repeat($input)? }, expansion = longhand { value: CssBorderImageRepeat, inherited: false, initial_kind: value, initial: CssBorderImageRepeat::new(CssBorderImageRepeatKeyword::Stretch, CssBorderImageRepeatKeyword::Stretch) };
            BorderImageSlice, "border-image-slice", [], "official.property.border-image-slice", crate::CssBorderImageSlice, parse_border_image_slice, { parse_border_image_slice($input, $numeric)? }, expansion = longhand { value: CssBorderImageSlice, inherited: false, initial_kind: value, initial: CssBorderImageSlice::try_new(vec![CssBorderImageSliceComponent::Percentage(crate::CssNumeric::<crate::CssPercentage, crate::CssNonNegative>::try_from_component(crate::CssComponentValue::try_token("100%").expect("percentage token")).expect("100% is non-negative"))], false).expect("one slice component is valid") };
            BorderImageSource, "border-image-source", [], "official.property.border-image-source", crate::CssImageValue, parse_border_image_source, { parse_border_image_source($input, $numeric)? }, expansion = longhand { value: CssImageValue, inherited: false, initial_kind: value, initial: CssImageValue::None };
            BorderImageWidth, "border-image-width", [], "official.property.border-image-width", crate::CssBorderImageWidth, parse_border_image_width, { parse_border_image_width($input, $numeric)? }, expansion = longhand { value: CssBorderImageWidth, inherited: false, initial_kind: value, initial: CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Number(crate::CssNumeric::<crate::CssNumber, crate::CssNonNegative>::try_from_component(crate::CssComponentValue::try_number("1").expect("number token")).expect("1 is non-negative"))]).expect("one width component is valid") };
            ImageOrientation, "image-orientation", [], "official.property.image-orientation", crate::CssImageOrientation, parse_image_orientation, { parse_image_orientation($input, $numeric)? }, expansion = longhand { value: CssImageOrientation, inherited: true, initial_kind: value, initial: CssImageOrientation::FromImage };
            ImageRendering, "image-rendering", [], "official.property.image-rendering", crate::CssImageRendering, parse_image_rendering, { parse_image_rendering($input)? }, expansion = longhand { value: CssImageRendering, inherited: true, initial_kind: value, initial: CssImageRendering::Auto };
            ObjectFit, "object-fit", [], "official.property.object-fit", crate::CssObjectFit, parse_object_fit, { parse_object_fit($input)? }, expansion = longhand { value: CssObjectFit, inherited: false, initial_kind: value, initial: CssObjectFit::Fill };
            BorderStyle, "border-style", [], "baseline.property.border-style", crate::CssFourSides<CssLineStyle>, parse_border_style_shorthand, { parse_border_style_shorthand($input)? }, expansion = four_side { mode: |value: &crate::CssFourSides<crate::CssLineStyle>| value.kind(), physical: [ BorderTopStyle => |value: &crate::CssFourSides<crate::CssLineStyle>| value.assigned()[0].clone(), BorderRightStyle => |value: &crate::CssFourSides<crate::CssLineStyle>| value.assigned()[1].clone(), BorderBottomStyle => |value: &crate::CssFourSides<crate::CssLineStyle>| value.assigned()[2].clone(), BorderLeftStyle => |value: &crate::CssFourSides<crate::CssLineStyle>| value.assigned()[3].clone() ], logical: [ BorderBlockStartStyle => |value: &crate::CssFourSides<crate::CssLineStyle>| value.assigned()[0].clone(), BorderInlineStartStyle => |value: &crate::CssFourSides<crate::CssLineStyle>| value.assigned()[1].clone(), BorderBlockEndStyle => |value: &crate::CssFourSides<crate::CssLineStyle>| value.assigned()[2].clone(), BorderInlineEndStyle => |value: &crate::CssFourSides<crate::CssLineStyle>| value.assigned()[3].clone() ] };
            BorderTopStyle, "border-top-style", [], "baseline.property.border-top-style", crate::CssLineStyle, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None, logical_group: BorderStyle, mapping: Physical };
            BorderRightStyle, "border-right-style", [], "baseline.property.border-right-style", crate::CssLineStyle, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None, logical_group: BorderStyle, mapping: Physical };
            BorderBottomStyle, "border-bottom-style", [], "baseline.property.border-bottom-style", crate::CssLineStyle, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None, logical_group: BorderStyle, mapping: Physical };
            BorderLeftStyle, "border-left-style", [], "baseline.property.border-left-style", crate::CssLineStyle, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None, logical_group: BorderStyle, mapping: Physical };
            BorderBlockStartStyle, "border-block-start-style", [], "official.property.border-block-start-style", crate::CssLineStyle, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None, logical_group: BorderStyle, mapping: Logical };
            BorderBlockEndStyle, "border-block-end-style", [], "official.property.border-block-end-style", crate::CssLineStyle, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None, logical_group: BorderStyle, mapping: Logical };
            BorderInlineStartStyle, "border-inline-start-style", [], "official.property.border-inline-start-style", crate::CssLineStyle, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None, logical_group: BorderStyle, mapping: Logical };
            BorderInlineEndStyle, "border-inline-end-style", [], "official.property.border-inline-end-style", crate::CssLineStyle, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssLineStyle, inherited: false, initial_kind: value, initial: CssLineStyle::None, logical_group: BorderStyle, mapping: Logical };
            BorderBlockStyle, "border-block-style", [], "official.property.border-block-style", crate::CssSidePair<CssLineStyle>, parse_border_style_pair, { parse_border_style_pair($input)? }, expansion = shorthand { members: [ BorderBlockStartStyle => |value: &crate::CssSidePair<crate::CssLineStyle>| Some(*value.first()), BorderBlockEndStyle => |value: &crate::CssSidePair<crate::CssLineStyle>| Some(*value.second()) ], reset_only: [] };
            BorderInlineStyle, "border-inline-style", [], "official.property.border-inline-style", crate::CssSidePair<CssLineStyle>, parse_border_style_pair, { parse_border_style_pair($input)? }, expansion = shorthand { members: [ BorderInlineStartStyle => |value: &crate::CssSidePair<crate::CssLineStyle>| Some(*value.first()), BorderInlineEndStyle => |value: &crate::CssSidePair<crate::CssLineStyle>| Some(*value.second()) ], reset_only: [] };
            BorderRadius, "border-radius", [], "baseline.property.border-radius", crate::CssBorderRadiusShorthand, parse_exact_border_radius, { parse_exact_border_radius($input, $numeric)? }, expansion = shorthand { members: [ BorderTopLeftRadius => |value: &CssBorderRadiusShorthand| Some(value.top_left()), BorderTopRightRadius => |value: &CssBorderRadiusShorthand| Some(value.top_right()), BorderBottomRightRadius => |value: &CssBorderRadiusShorthand| Some(value.bottom_right()), BorderBottomLeftRadius => |value: &CssBorderRadiusShorthand| Some(value.bottom_left()) ], reset_only: [] };
            BorderTopLeftRadius, "border-top-left-radius", [], "baseline.property.border-top-left-radius", crate::CssSidePair<CornerRadiusLength>, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<CornerRadiusLength>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero(), None), logical_group: BorderRadius, mapping: Physical };
            BorderTopRightRadius, "border-top-right-radius", [], "baseline.property.border-top-right-radius", crate::CssSidePair<CornerRadiusLength>, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<CornerRadiusLength>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero(), None), logical_group: BorderRadius, mapping: Physical };
            BorderBottomRightRadius, "border-bottom-right-radius", [], "baseline.property.border-bottom-right-radius", crate::CssSidePair<CornerRadiusLength>, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<CornerRadiusLength>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero(), None), logical_group: BorderRadius, mapping: Physical };
            BorderBottomLeftRadius, "border-bottom-left-radius", [], "baseline.property.border-bottom-left-radius", crate::CssSidePair<CornerRadiusLength>, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<CornerRadiusLength>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero(), None), logical_group: BorderRadius, mapping: Physical };
            BorderStartStartRadius, "border-start-start-radius", [], "official.property.border-start-start-radius", crate::CssSidePair<CornerRadiusLength>, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<CornerRadiusLength>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero(), None), logical_group: BorderRadius, mapping: Logical };
            BorderStartEndRadius, "border-start-end-radius", [], "official.property.border-start-end-radius", crate::CssSidePair<CornerRadiusLength>, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<CornerRadiusLength>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero(), None), logical_group: BorderRadius, mapping: Logical };
            BorderEndStartRadius, "border-end-start-radius", [], "official.property.border-end-start-radius", crate::CssSidePair<CornerRadiusLength>, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<CornerRadiusLength>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero(), None), logical_group: BorderRadius, mapping: Logical };
            BorderEndEndRadius, "border-end-end-radius", [], "official.property.border-end-end-radius", crate::CssSidePair<CornerRadiusLength>, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: crate::CssSidePair<CornerRadiusLength>, inherited: false, initial_kind: value, initial: crate::CssSidePair::new(CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero(), None), logical_group: BorderRadius, mapping: Logical };
            BoxShadow, "box-shadow", [], "baseline.property.box-shadow", crate::CssBoxShadow, parse_box_shadow, { parse_box_shadow($input, $numeric)? }, expansion = longhand { value: CssBoxShadow, inherited: false, initial_kind: value, initial: CssBoxShadow::None };
            Opacity, "opacity", [], "baseline.property.opacity", crate::CssOpacityValue, parse_opacity, { parse_opacity($input, $numeric)? }, expansion = longhand { value: CssOpacityValue, inherited: false, initial_kind: value, initial: CssOpacityValue::Scalar(crate::CssOpacityScalar::try_from_component(crate::CssComponentValue::try_number("1").expect("valid initial number")).expect("valid opacity scalar")) };
            FlexGrow, "flex-grow", ["-webkit-flex-grow"], "baseline.property.flex-grow", crate::CssNumeric<CssNumber, CssNonNegative>, parse_flex_factor, { parse_flex_factor($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssNumber, CssNonNegative>, inherited: false, initial_kind: value, initial: crate::flex::initial_grow() };
            FlexShrink, "flex-shrink", ["-webkit-flex-shrink"], "baseline.property.flex-shrink", crate::CssNumeric<CssNumber, CssNonNegative>, parse_flex_factor, { parse_flex_factor($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssNumber, CssNonNegative>, inherited: false, initial_kind: value, initial: crate::flex::initial_shrink() };
            Order, "order", ["-webkit-order"], "baseline.property.order", crate::CssNumeric<CssInteger>, parse_order, { parse_order($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssInteger>, inherited: false, initial_kind: value, initial: CssNumeric::<CssInteger>::try_from_literal(crate::CssNumericLiteral::<crate::CssInteger>::from_i32(0)).expect("integer zero") };
            Flex, "flex", ["-webkit-flex"], "baseline.property.flex", crate::CssFlexValue, parse_flex, { parse_flex($input, $numeric)? }, expansion = shorthand { members: [ FlexGrow => crate::flex::effective_grow, FlexShrink => crate::flex::effective_shrink, FlexBasis => crate::flex::effective_basis ], reset_only: [] };
            AspectRatio, "aspect-ratio", [], "baseline.property.aspect-ratio", crate::CssAspectRatioValue, parse_aspect_ratio, { parse_aspect_ratio($input, $numeric)? }, expansion = longhand { value: CssAspectRatioValue, inherited: false, initial_kind: value, initial: CssAspectRatioValue::Auto };
            ScrollbarWidth, "scrollbar-width", [], "baseline.property.scrollbar-width", crate::CssScrollbarWidth, parse_scrollbar_width, { parse_scrollbar_width($input)? }, expansion = longhand { value: CssScrollbarWidth, inherited: false, initial_kind: value, initial: CssScrollbarWidth::Auto };
            ScrollbarColor, "scrollbar-color", [], "official.property.scrollbar-color", crate::CssScrollbarColor, parse_scrollbar_color, { parse_scrollbar_color($input, $numeric)? }, expansion = longhand { value: CssScrollbarColor, inherited: true, initial_kind: value, initial: CssScrollbarColor::auto() };
            ColorScheme, "color-scheme", [], "official.property.color-scheme", crate::CssColorScheme, parse_color_scheme, { parse_color_scheme($input)? }, expansion = longhand { value: CssColorScheme, inherited: true, initial_kind: value, initial: CssColorScheme::normal() };
            ForcedColorAdjust, "forced-color-adjust", [], "official.property.forced-color-adjust", crate::CssForcedColorAdjust, parse_forced_color_adjust, { parse_forced_color_adjust($input)? }, expansion = longhand { value: CssForcedColorAdjust, inherited: true, initial_kind: value, initial: CssForcedColorAdjust::Auto };
            PrintColorAdjust, "print-color-adjust", [], "official.property.print-color-adjust", crate::CssPrintColorAdjust, parse_print_color_adjust, { parse_print_color_adjust($input)? }, expansion = longhand { value: CssPrintColorAdjust, inherited: true, initial_kind: value, initial: CssPrintColorAdjust::Economy };
            ColorAdjust, "color-adjust", [], "official.property.color-adjust", crate::CssPrintColorAdjust, parse_print_color_adjust, { parse_print_color_adjust($input)? }, expansion = shorthand { members: [ PrintColorAdjust => |value: &CssPrintColorAdjust| Some(*value) ], reset_only: [] };
            Cursor, "cursor", [], "baseline.property.cursor", crate::CssCursor, parse_cursor, { parse_cursor($input, $numeric)? }, expansion = longhand { value: CssCursor, inherited: true, initial_kind: value, initial: CssCursor::Keyword(CssCursorKeyword::Auto) };
            CaretColor, "caret-color", [], "official.property.caret-color", crate::CssCaretColor, parse_caret_color, { parse_caret_color($input, $numeric)? }, expansion = longhand { value: CssCaretColor, inherited: true, initial_kind: value, initial: CssCaretColor::Auto };
            PointerEvents, "pointer-events", [], "baseline.property.pointer-events", crate::CssPointerEvents, parse_pointer_events, { parse_pointer_events($input)? }, expansion = longhand { value: CssPointerEvents, inherited: true, initial_kind: value, initial: CssPointerEvents::Auto };
            UserSelect, "user-select", ["-webkit-user-select"], "baseline.property.user-select", crate::CssUserSelect, parse_user_select, { parse_user_select($input)? }, expansion = longhand { value: CssUserSelect, inherited: false, initial_kind: value, initial: CssUserSelect::Auto };
            Resize, "resize", [], "official.property.resize", crate::CssResize, parse_resize, { parse_resize($input)? }, expansion = longhand { value: CssResize, inherited: false, initial_kind: value, initial: CssResize::None };
            Contain, "contain", [], "official.property.contain", crate::CssContain, parse_contain, { parse_contain($input)? }, expansion = longhand { value: CssContain, inherited: false, initial_kind: value, initial: CssContain::None };
            Outline, "outline", [], "baseline.property.outline", crate::CssOutline, parse_outline, { parse_outline($input, $numeric)? }, expansion = shorthand { members: [ OutlineWidth => |value: &CssOutline| value.width().cloned(), OutlineStyle => |value: &CssOutline| value.style(), OutlineColor => |value: &CssOutline| value.color().cloned() ], reset_only: [] };
            OutlineColor, "outline-color", [], "baseline.property.outline-color", crate::CssOutlineColor, parse_outline_color, { parse_outline_color($input, $numeric)? }, expansion = longhand { value: CssOutlineColor, inherited: false, initial_kind: value, initial: CssOutlineColor::Auto };
            OutlineOffset, "outline-offset", [], "official.property.outline-offset", crate::CssNumeric<CssLength>, parse_outline_offset, { parse_outline_offset($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLength>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLength>::zero() };
            OutlineStyle, "outline-style", [], "baseline.property.outline-style", crate::CssOutlineStyle, parse_outline_style, { parse_outline_style($input)? }, expansion = longhand { value: CssOutlineStyle, inherited: false, initial_kind: value, initial: CssOutlineStyle::None };
            OutlineWidth, "outline-width", [], "baseline.property.outline-width", crate::CssLineWidth, parse_outline_width, { parse_outline_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, inherited: false, initial_kind: value, initial: CssLineWidth::Medium };
            NavUp, "nav-up", [], "official.property.nav-up", crate::CssNavigation, parse_navigation, { parse_navigation($input, $numeric)? }, expansion = longhand { value: CssNavigation, inherited: false, initial_kind: value, initial: CssNavigation::Auto };
            NavRight, "nav-right", [], "official.property.nav-right", crate::CssNavigation, parse_navigation, { parse_navigation($input, $numeric)? }, expansion = longhand { value: CssNavigation, inherited: false, initial_kind: value, initial: CssNavigation::Auto };
            NavDown, "nav-down", [], "official.property.nav-down", crate::CssNavigation, parse_navigation, { parse_navigation($input, $numeric)? }, expansion = longhand { value: CssNavigation, inherited: false, initial_kind: value, initial: CssNavigation::Auto };
            NavLeft, "nav-left", [], "official.property.nav-left", crate::CssNavigation, parse_navigation, { parse_navigation($input, $numeric)? }, expansion = longhand { value: CssNavigation, inherited: false, initial_kind: value, initial: CssNavigation::Auto };
            CaretAnimation, "caret-animation", [], "official.property.caret-animation", crate::CssCaretAnimation, parse_caret_animation, { parse_caret_animation($input)? }, expansion = longhand { value: CssCaretAnimation, inherited: true, initial_kind: value, initial: CssCaretAnimation::Auto };
            CaretShape, "caret-shape", [], "official.property.caret-shape", crate::CssCaretShape, parse_caret_shape, { parse_caret_shape($input)? }, expansion = longhand { value: CssCaretShape, inherited: true, initial_kind: value, initial: CssCaretShape::Auto };
            Caret, "caret", [], "official.property.caret", crate::CssCaret, parse_caret, { parse_caret($input, $numeric)? }, expansion = shorthand { members: [ CaretColor => |value: &CssCaret| value.color().cloned(), CaretAnimation => |value: &CssCaret| value.animation(), CaretShape => |value: &CssCaret| value.shape() ], reset_only: [] };
            Interactivity, "interactivity", [], "official.property.interactivity", crate::CssInteractivity, parse_interactivity, { parse_interactivity($input)? }, expansion = longhand { value: CssInteractivity, inherited: true, initial_kind: value, initial: CssInteractivity::Auto };
            InterestDelayStart, "interest-delay-start", [], "official.property.interest-delay-start", crate::CssInterestDelayValue, parse_interest_delay_value, { parse_interest_delay_value($input, $numeric)? }, expansion = longhand { value: CssInterestDelayValue, inherited: true, initial_kind: value, initial: CssInterestDelayValue::Normal };
            InterestDelayEnd, "interest-delay-end", [], "official.property.interest-delay-end", crate::CssInterestDelayValue, parse_interest_delay_value, { parse_interest_delay_value($input, $numeric)? }, expansion = longhand { value: CssInterestDelayValue, inherited: true, initial_kind: value, initial: CssInterestDelayValue::Normal };
            InterestDelay, "interest-delay", [], "official.property.interest-delay", crate::CssSidePair<CssInterestDelayValue>, parse_interest_delay, { parse_interest_delay($input, $numeric)? }, expansion = shorthand { members: [ InterestDelayStart => |value: &crate::CssSidePair<crate::CssInterestDelayValue>| Some(value.first().clone()), InterestDelayEnd => |value: &crate::CssSidePair<crate::CssInterestDelayValue>| Some(value.second().clone()) ], reset_only: [] };
            AccentColor, "accent-color", [], "official.property.accent-color", crate::CssAccentColor, parse_accent_color, { parse_accent_color($input, $numeric)? }, expansion = longhand { value: CssAccentColor, inherited: true, initial_kind: value, initial: CssAccentColor::Auto };
            Appearance, "appearance", ["-webkit-appearance"], "official.property.appearance", crate::CssAppearance, parse_appearance, { parse_appearance($input)? }, expansion = longhand { value: CssAppearance, inherited: false, initial_kind: value, initial: CssAppearance::None };

            Transform, "transform", [], "baseline.property.transform", crate::CssTransform, parse_transform, { parse_transform($input, $numeric)? }, expansion = longhand { value: CssTransform, inherited: false, initial_kind: value, initial: CssTransform::None };
            TransformBox, "transform-box", [], "official.property.transform-box", crate::CssTransformBox, parse_transform_box, { parse_transform_box($input)? }, expansion = longhand { value: CssTransformBox, inherited: false, initial_kind: value, initial: CssTransformBox::try_new(CssBoxEdgeKeyword::ViewBox).expect("valid initial reference box") };
            TransformOrigin, "transform-origin", [], "baseline.property.transform-origin", crate::CssTransformOrigin, parse_transform_origin, { parse_transform_origin($input, $numeric)? }, expansion = longhand { value: CssTransformOrigin, inherited: false, initial_kind: value, initial: CssTransformOrigin::try_new(crate::expansion::initial_transform_position(), None).expect("initial planar position") };
            Translate, "translate", [], "baseline.property.translate", crate::CssTranslate, parse_translate, { parse_translate($input, $numeric)? }, expansion = longhand { value: CssTranslate, inherited: false, initial_kind: value, initial: CssTranslate::None };
            Rotate, "rotate", [], "baseline.property.rotate", crate::CssRotate, parse_rotate, { parse_rotate($input, $numeric)? }, expansion = longhand { value: CssRotate, inherited: false, initial_kind: value, initial: CssRotate::None };
            Scale, "scale", [], "baseline.property.scale", crate::CssScale, parse_scale, { parse_scale($input, $numeric)? }, expansion = longhand { value: CssScale, inherited: false, initial_kind: value, initial: CssScale::None };
            Perspective, "perspective", [], "official.property.perspective", crate::CssPerspective, parse_perspective, { parse_perspective($input, $numeric)? }, expansion = longhand { value: CssPerspective, inherited: false, initial_kind: value, initial: CssPerspective::None };
            PerspectiveOrigin, "perspective-origin", [], "official.property.perspective-origin", crate::CssPosition, parse_object_position, { parse_object_position($input, $numeric)? }, expansion = longhand { value: CssPosition, inherited: false, initial_kind: value, initial: crate::expansion::initial_position() };
            TransformStyle, "transform-style", [], "official.property.transform-style", crate::CssTransformStyle, parse_transform_style, { parse_transform_style($input)? }, expansion = longhand { value: CssTransformStyle, inherited: false, initial_kind: value, initial: CssTransformStyle::Flat };
            BackfaceVisibility, "backface-visibility", [], "official.property.backface-visibility", crate::CssBackfaceVisibility, parse_backface_visibility, { parse_backface_visibility($input)? }, expansion = longhand { value: CssBackfaceVisibility, inherited: false, initial_kind: value, initial: CssBackfaceVisibility::Visible };
            Filter, "filter", [], "baseline.property.filter", crate::CssFilter, parse_filter, { parse_filter($input, $numeric)? }, expansion = longhand { value: CssFilter, inherited: false, initial_kind: value, initial: CssFilter::None };
            FloodColor, "flood-color", [], "official.property.flood-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::from_named(crate::CssNamedColor::try_new("black").expect("valid initial named color")) };
            FloodOpacity, "flood-opacity", [], "official.property.flood-opacity", crate::CssOpacityValue, parse_opacity, { parse_opacity($input, $numeric)? }, expansion = longhand { value: CssOpacityValue, inherited: false, initial_kind: value, initial: CssOpacityValue::Scalar(crate::CssOpacityScalar::try_from_component(crate::CssComponentValue::try_number("1").expect("valid initial number")).expect("valid opacity scalar")) };
            LightingColor, "lighting-color", [], "official.property.lighting-color", crate::CssColor, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, inherited: false, initial_kind: value, initial: CssColor::from_named(crate::CssNamedColor::try_new("white").expect("valid initial named color")) };
            ColorInterpolationFilters, "color-interpolation-filters", [], "official.property.color-interpolation-filters", crate::CssColorInterpolationFilters, parse_color_interpolation_filters, { parse_color_interpolation_filters($input)? }, expansion = longhand { value: CssColorInterpolationFilters, inherited: true, initial_kind: value, initial: CssColorInterpolationFilters::LinearRgb };
            BackdropFilter, "backdrop-filter", [], "baseline.property.backdrop-filter", crate::CssFilter, parse_filter, { parse_filter($input, $numeric)? }, expansion = longhand { value: CssFilter, inherited: false, initial_kind: value, initial: CssFilter::None };
            ClipPath, "clip-path", [], "baseline.property.clip-path", crate::CssClipPath, parse_clip_path, { parse_clip_path($input, $numeric)? }, expansion = longhand { value: crate::CssClipPath, inherited: false, initial_kind: value, initial: crate::CssClipPath::None };
            ShapeOutside, "shape-outside", [], "official.property.shape-outside", crate::CssShapeOutside, parse_shape_outside, { parse_shape_outside($input, $numeric)? }, expansion = longhand { value: crate::CssShapeOutside, inherited: false, initial_kind: value, initial: crate::CssShapeOutside::None };
            OffsetPosition, "offset-position", [], "official.property.offset-position", crate::CssOffsetPosition, parse_offset_position, { parse_offset_position($input, $numeric)? }, expansion = longhand { value: CssOffsetPosition, inherited: false, initial_kind: value, initial: CssOffsetPosition::Normal };
            OffsetPath, "offset-path", [], "official.property.offset-path", crate::CssOffsetPath, parse_offset_path, { parse_offset_path($input, $numeric)? }, expansion = longhand { value: CssOffsetPath, inherited: false, initial_kind: value, initial: CssOffsetPath::none() };
            OffsetDistance, "offset-distance", [], "official.property.offset-distance", crate::CssNumeric<CssLengthPercentage>, parse_length_percentage, { values::parse_length_percentage($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLengthPercentage>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLengthPercentage>::exact_zero() };
            OffsetRotate, "offset-rotate", [], "official.property.offset-rotate", crate::CssOffsetRotate, parse_offset_rotate, { parse_offset_rotate($input, $numeric)? }, expansion = longhand { value: CssOffsetRotate, inherited: false, initial_kind: value, initial: CssOffsetRotate::auto() };
            OffsetAnchor, "offset-anchor", [], "official.property.offset-anchor", crate::CssOffsetAnchor, parse_offset_anchor, { parse_offset_anchor($input, $numeric)? }, expansion = longhand { value: CssOffsetAnchor, inherited: false, initial_kind: value, initial: CssOffsetAnchor::Auto };
            Offset, "offset", [], "official.property.offset", crate::CssOffset, parse_offset, { parse_offset($input, $numeric)? }, expansion = shorthand { members: [ OffsetPosition => |value: &CssOffset| value.position().cloned(), OffsetPath => |value: &CssOffset| value.path().cloned(), OffsetDistance => |value: &CssOffset| value.distance().cloned(), OffsetRotate => |value: &CssOffset| value.rotate().cloned(), OffsetAnchor => |value: &CssOffset| value.anchor().cloned() ], reset_only: [] };
            ShapeImageThreshold, "shape-image-threshold", [], "official.property.shape-image-threshold", crate::CssOpacityValue, parse_opacity, { parse_opacity($input, $numeric)? }, expansion = longhand { value: CssOpacityValue, inherited: false, initial_kind: value, initial: CssOpacityValue::Scalar(crate::CssOpacityScalar::try_from_component(crate::CssComponentValue::try_number("0").expect("valid initial number")).expect("valid opacity scalar")) };
            ShapeMargin, "shape-margin", [], "official.property.shape-margin", crate::CssNumeric<CssLengthPercentage, CssNonNegative>, parse_nonnegative_length_percentage, { values::parse_nonnegative_length_percentage($input, $numeric)? }, expansion = longhand { value: CssNumeric<CssLengthPercentage, CssNonNegative>, inherited: false, initial_kind: value, initial: CssNumeric::<CssLengthPercentage, CssNonNegative>::exact_zero() };
            BackgroundBlendMode, "background-blend-mode", [], "official.property.background-blend-mode", crate::CssBlendModeList, parse_blend_mode_list, { parse_blend_mode_list($input)? }, expansion = longhand { value: CssBlendModeList, inherited: false, initial_kind: value, initial: CssBlendModeList::try_new(vec![CssBlendMode::Normal]).expect("one initial blend mode") };
            Isolation, "isolation", [], "official.property.isolation", crate::CssIsolation, parse_isolation, { parse_isolation($input)? }, expansion = longhand { value: CssIsolation, inherited: false, initial_kind: value, initial: CssIsolation::Auto };
            MixBlendMode, "mix-blend-mode", [], "official.property.mix-blend-mode", crate::CssBlendMode, parse_blend_mode, { parse_blend_mode($input)? }, expansion = longhand { value: CssBlendMode, inherited: false, initial_kind: value, initial: CssBlendMode::Normal };
            Mask, "mask", [], "baseline.property.mask", crate::CssMaskList, parse_mask_list, { parse_mask_list($input, $numeric)? }, expansion = shorthand { members: [
                MaskImage => crate::expansion::mask_images,
                MaskPosition => crate::expansion::mask_positions,
                MaskSize => crate::expansion::mask_sizes,
                MaskRepeat => crate::expansion::mask_repeats,
                MaskOrigin => crate::expansion::mask_origins,
                MaskClip => crate::expansion::mask_clips,
                MaskComposite => crate::expansion::mask_composites,
                MaskMode => crate::expansion::mask_modes
            ], reset_only: [MaskBorderSource, MaskBorderSlice, MaskBorderWidth, MaskBorderOutset, MaskBorderRepeat, MaskBorderMode] };
            MaskImage, "mask-image", [], "baseline.property.mask-image", crate::CssImageValueList, parse_image_layer_list, { parse_image_layer_list($input, $numeric)? }, expansion = longhand { value: CssImageValueList, inherited: false, initial_kind: value, initial: CssImageValueList::try_new(vec![CssImageValue::None]).expect("one initial image") };
            MaskSize, "mask-size", [], "baseline.property.mask-size", crate::CssBackgroundSizeList, parse_background_size_list, { parse_background_size_list($input, $numeric)? }, expansion = longhand { value: CssBackgroundSizeList, inherited: false, initial_kind: value, initial: CssBackgroundSizeList::try_new(vec![CssBackgroundSize::Explicit { width: CssAutoOr::Auto, height: None }]).expect("one initial size") };
            MaskPosition, "mask-position", [], "baseline.property.mask-position", crate::CssPositionList, parse_mask_position_list, { parse_mask_position_list($input, $numeric)? }, expansion = longhand { value: CssPositionList, inherited: false, initial_kind: value, initial: crate::expansion::initial_mask_positions() };
            MaskRepeat, "mask-repeat", [], "baseline.property.mask-repeat", crate::CssBackgroundRepeatList, parse_background_repeat_list, { parse_background_repeat_list($input)? }, expansion = longhand { value: CssBackgroundRepeatList, inherited: false, initial_kind: value, initial: CssBackgroundRepeatList::try_new(vec![CssBackgroundRepeat::Axes { x: CssBackgroundRepeatStyle::Repeat, y: CssBackgroundRepeatStyle::Repeat }]).expect("one initial repeat") };
            MaskMode, "mask-mode", [], "official.property.mask-mode", crate::CssMaskModeList, parse_mask_mode_list, { parse_mask_mode_list($input)? }, expansion = longhand { value: CssMaskModeList, inherited: false, initial_kind: value, initial: CssMaskModeList::try_new(vec![CssMaskMode::MatchSource]).expect("one initial mode") };
            MaskOrigin, "mask-origin", [], "official.property.mask-origin", crate::CssMaskBoxList, parse_mask_origin_list, { parse_mask_origin_list($input)? }, expansion = longhand { value: CssMaskBoxList, inherited: false, initial_kind: value, initial: CssMaskBoxList::try_new(vec![CssCoordBox::BorderBox]).expect("one initial origin") };
            MaskClip, "mask-clip", [], "official.property.mask-clip", crate::CssMaskClipList, parse_mask_clip_list, { parse_mask_clip_list($input)? }, expansion = longhand { value: CssMaskClipList, inherited: false, initial_kind: value, initial: CssMaskClipList::try_new(vec![CssMaskClip::Box(CssCoordBox::BorderBox)]).expect("one initial clip") };
            MaskComposite, "mask-composite", [], "official.property.mask-composite", crate::CssMaskCompositeList, parse_mask_composite_list, { parse_mask_composite_list($input)? }, expansion = longhand { value: CssMaskCompositeList, inherited: false, initial_kind: value, initial: CssMaskCompositeList::try_new(vec![CssMaskComposite::Add]).expect("one initial operator") };
            MaskBorderSource, "mask-border-source", [], "official.property.mask-border-source", crate::CssImageValue, parse_border_image_source, { parse_border_image_source($input, $numeric)? }, expansion = longhand { value: CssImageValue, inherited: false, initial_kind: value, initial: CssImageValue::None };
            MaskBorderSlice, "mask-border-slice", [], "official.property.mask-border-slice", crate::CssBorderImageSlice, parse_mask_border_slice, { parse_mask_border_slice($input, $numeric)? }, expansion = longhand { value: CssBorderImageSlice, inherited: false, initial_kind: value, initial: crate::expansion::initial_mask_border_slice() };
            MaskBorderWidth, "mask-border-width", [], "official.property.mask-border-width", crate::CssBorderImageWidth, parse_border_image_width, { parse_border_image_width($input, $numeric)? }, expansion = longhand { value: CssBorderImageWidth, inherited: false, initial_kind: value, initial: CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Auto]).expect("one initial width") };
            MaskBorderOutset, "mask-border-outset", [], "official.property.mask-border-outset", crate::CssBorderImageOutset, parse_border_image_outset, { parse_border_image_outset($input, $numeric)? }, expansion = longhand { value: CssBorderImageOutset, inherited: false, initial_kind: value, initial: CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Number(crate::CssNumeric::<crate::CssNumber, crate::CssNonNegative>::try_from_component(crate::CssComponentValue::try_number("0").expect("number token")).expect("nonnegative zero"))]).expect("one initial outset") };
            MaskBorderRepeat, "mask-border-repeat", [], "official.property.mask-border-repeat", crate::CssBorderImageRepeat, parse_border_image_repeat, { parse_border_image_repeat($input)? }, expansion = longhand { value: CssBorderImageRepeat, inherited: false, initial_kind: value, initial: CssBorderImageRepeat::new(CssBorderImageRepeatKeyword::Stretch, CssBorderImageRepeatKeyword::Stretch) };
            MaskBorderMode, "mask-border-mode", [], "official.property.mask-border-mode", crate::CssMaskType, parse_mask_type, { parse_mask_type($input)? }, expansion = longhand { value: CssMaskType, inherited: false, initial_kind: value, initial: CssMaskType::Alpha };
            ClipRule, "clip-rule", [], "official.property.clip-rule", crate::CssClipRule, parse_clip_rule, { parse_clip_rule($input)? }, expansion = longhand { value: CssClipRule, inherited: true, initial_kind: value, initial: CssClipRule::Nonzero };
            MaskType, "mask-type", [], "official.property.mask-type", crate::CssMaskType, parse_mask_type, { parse_mask_type($input)? }, expansion = longhand { value: CssMaskType, inherited: false, initial_kind: value, initial: CssMaskType::Luminance };
            MaskBorder, "mask-border", [], "official.property.mask-border", crate::CssMaskBorder, parse_mask_border, { parse_mask_border($input, $numeric)? }, expansion = shorthand { members: [
                MaskBorderSource => |value: &CssMaskBorder| value.source().cloned(),
                MaskBorderSlice => |value: &CssMaskBorder| value.slice().cloned(),
                MaskBorderWidth => |value: &CssMaskBorder| value.width().cloned(),
                MaskBorderOutset => |value: &CssMaskBorder| value.outset().cloned(),
                MaskBorderRepeat => |value: &CssMaskBorder| value.repeat(),
                MaskBorderMode => |value: &CssMaskBorder| value.mode()
            ], reset_only: [] };
            TransitionProperty, "transition-property", [], "baseline.property.transition-property", crate::CssTransitionPropertyList, parse_transition_property_list, { parse_transition_property_list($input)? }, expansion = longhand { value: CssTransitionPropertyList, inherited: false, initial_kind: value, initial: CssTransitionPropertyList::try_new(vec![CssTransitionProperty::All]).expect("one initial transition property") };
            TransitionDuration, "transition-duration", [], "baseline.property.transition-duration", crate::CssDurationList, parse_duration_list, { parse_duration_list($input, $numeric)? }, expansion = longhand { value: CssDurationList, inherited: false, initial_kind: value, initial: CssDurationList::try_new(vec![crate::expansion::initial_zero_time()]).expect("one closed initial duration") };
            TransitionDelay, "transition-delay", [], "baseline.property.transition-delay", crate::CssDelayList, parse_delay_list, { parse_delay_list($input, $numeric)? }, expansion = longhand { value: CssDelayList, inherited: false, initial_kind: value, initial: CssDelayList::try_new(vec![crate::expansion::initial_zero_time()]).expect("one closed initial delay") };
            TransitionTimingFunction, "transition-timing-function", [], "baseline.property.transition-timing-function", crate::CssEasingList, parse_easing_list, { parse_easing_list($input, $numeric)? }, expansion = longhand { value: CssEasingList, inherited: false, initial_kind: value, initial: CssEasingList::try_new(vec![CssEasing::Keyword(CssEasingKeyword::Ease)]).expect("one initial easing") };
            Transition, "transition", [], "baseline.property.transition", crate::CssTransitionList, parse_transition_value_list, { parse_transition_value_list($input, $numeric)? }, expansion = shorthand { members: [
                TransitionProperty => crate::expansion::transition_properties,
                TransitionDuration => crate::expansion::transition_durations,
                TransitionTimingFunction => crate::expansion::transition_timing_functions,
                TransitionDelay => crate::expansion::transition_delays
            ], reset_only: [] };
            AnimationName, "animation-name", [], "baseline.property.animation-name", crate::CssAnimationNameList, parse_animation_name_list, { parse_animation_name_list($input)? }, expansion = longhand { value: CssAnimationNameList, inherited: false, initial_kind: value, initial: CssAnimationNameList::try_new(vec![CssAnimationName::None]).expect("one initial animation name") };
            AnimationDuration, "animation-duration", [], "baseline.property.animation-duration", crate::CssDurationList, parse_duration_list, { parse_duration_list($input, $numeric)? }, expansion = longhand { value: CssDurationList, inherited: false, initial_kind: value, initial: CssDurationList::try_new(vec![crate::expansion::initial_zero_time()]).expect("one closed initial duration") };
            AnimationDelay, "animation-delay", [], "baseline.property.animation-delay", crate::CssDelayList, parse_delay_list, { parse_delay_list($input, $numeric)? }, expansion = longhand { value: CssDelayList, inherited: false, initial_kind: value, initial: CssDelayList::try_new(vec![crate::expansion::initial_zero_time()]).expect("one closed initial delay") };
            AnimationTimingFunction, "animation-timing-function", [], "baseline.property.animation-timing-function", crate::CssEasingList, parse_easing_list, { parse_easing_list($input, $numeric)? }, expansion = longhand { value: CssEasingList, inherited: false, initial_kind: value, initial: CssEasingList::try_new(vec![CssEasing::Keyword(CssEasingKeyword::Ease)]).expect("one initial easing") };
            AnimationIterationCount, "animation-iteration-count", [], "baseline.property.animation-iteration-count", crate::CssAnimationIterationCountList, parse_animation_iteration_value_list, { parse_animation_iteration_value_list($input, $numeric)? }, expansion = longhand { value: CssAnimationIterationCountList, inherited: false, initial_kind: value, initial: CssAnimationIterationCountList::try_new(vec![CssAnimationIterationCount::Number(crate::CssNumeric::<crate::CssNumber, crate::CssNonNegative>::try_from_component(crate::CssComponentValue::try_number("1").expect("ordinary one")).expect("nonnegative initial count"))]).expect("one initial iteration count") };
            AnimationDirection, "animation-direction", [], "baseline.property.animation-direction", crate::CssAnimationDirectionList, parse_animation_direction_list, { parse_animation_direction_list($input)? }, expansion = longhand { value: CssAnimationDirectionList, inherited: false, initial_kind: value, initial: CssAnimationDirectionList::try_new(vec![CssAnimationDirection::Normal]).expect("one initial direction") };
            AnimationFillMode, "animation-fill-mode", [], "baseline.property.animation-fill-mode", crate::CssAnimationFillModeList, parse_animation_fill_mode_list, { parse_animation_fill_mode_list($input)? }, expansion = longhand { value: CssAnimationFillModeList, inherited: false, initial_kind: value, initial: CssAnimationFillModeList::try_new(vec![CssAnimationFillMode::None]).expect("one initial fill mode") };
            AnimationPlayState, "animation-play-state", [], "baseline.property.animation-play-state", crate::CssAnimationPlayStateList, parse_animation_play_state_list, { parse_animation_play_state_list($input)? }, expansion = longhand { value: CssAnimationPlayStateList, inherited: false, initial_kind: value, initial: CssAnimationPlayStateList::try_new(vec![CssAnimationPlayState::Running]).expect("one initial play state") };
            Animation, "animation", [], "baseline.property.animation", crate::CssAnimationList, parse_animation_value_list, { parse_animation_value_list($input, $numeric)? }, expansion = shorthand { members: [
                AnimationDuration => crate::expansion::animation_durations,
                AnimationTimingFunction => crate::expansion::animation_timing_functions,
                AnimationDelay => crate::expansion::animation_delays,
                AnimationIterationCount => crate::expansion::animation_iteration_counts,
                AnimationDirection => crate::expansion::animation_directions,
                AnimationFillMode => crate::expansion::animation_fill_modes,
                AnimationPlayState => crate::expansion::animation_play_states,
                AnimationName => crate::expansion::animation_names
            ], reset_only: [] };
        }
    };
}

pub(crate) use property_schema;

/// A grammar-checked authored ordinary value: the exact authored text plus one checked value.
///
/// The property identity is not part of the type. It is carried by the
/// [`CssKnownPropertyValueRef`] variant that borrows the value, so one generic type serves
/// every property whose grammar produces `T`.
///
/// Values come from parsing or from checked declaration construction; there is no public
/// constructor.
///
/// ```compile_fail,E0451
/// use surgeist_css::{CssAuthoredDeclarationValue, CssPropertyValue, CssSizeValue};
/// let _ = CssPropertyValue::<CssSizeValue> {
///     authored: CssAuthoredDeclarationValue::try_new("1px").unwrap(),
///     value: todo!(),
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssPropertyValue<T> {
    authored: CssAuthoredDeclarationValue,
    value: T,
}

impl<T> CssPropertyValue<T> {
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredDeclarationValue, value: T) -> Self {
        Self { authored, value }
    }

    /// Returns the exact authored ordinary value slice.
    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }

    /// Returns the grammar-checked semantic value.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

impl CssPropertyValue<crate::CssColorScheme> {
    pub(crate) fn retained_backing_bytes_with<E>(
        &self,
        admit: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<usize, crate::CssRetainedPayloadError<E>> {
        admit().map_err(crate::CssRetainedPayloadError::admission)?;
        self.authored
            .retained_backing_bytes()
            .checked_add(self.value.retained_backing_bytes_with(admit)?)
            .ok_or_else(crate::CssRetainedPayloadError::capacity_overflow)
    }
}

macro_rules! define_property_identity {
    ($input:ident, $numeric:ident;
        All, $all_canonical:literal, [$($all_alias:literal),*], $all_stable_id:literal,
        $all_value:ty,
        $all_parser:ident, $all_dispatch:block $(, expansion = $all_expansion:ident { $($all_metadata:tt)* })?;
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $parser:ident, $dispatch:block
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

        /// A borrowed property-specific ordinary-value view.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum CssKnownPropertyValueRef<'a> {
            $(
                #[doc = concat!("An authored `", $canonical, "` ordinary value.")]
                $variant(&'a CssPropertyValue<$value>),
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
            $($variant(CssDeclaredValue<CssPropertyValue<$value>>),)*
        }

        /// A grammar-checked property-coupled known declaration in the authored syntax phase.
        ///
        /// Parsing and [`crate::CssParser::parse_property_value`] construct this value through the same
        /// property grammar. Its private value discriminator determines canonical property
        /// identity; the coupled grammar handle additionally preserves legacy authored semantics.
        /// Callers cannot construct or mutate a property/value/grammar mismatch.
        ///
        /// ```compile_fail
        /// use surgeist_css::CssKnownDeclaration;
        /// let _ = CssKnownDeclaration { value: todo!() };
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
    pub(crate) parser: &'static str,
}

pub(crate) const fn property_implementation_inventory() -> &'static [PropertyImplementation] {
    IMPLEMENTED_PROPERTIES
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
