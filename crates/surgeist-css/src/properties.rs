//! Canonical identity and parser wiring for recognized CSS properties.
//!
//! The crate-private schema in this module is the single authority for the
//! frozen property set. Public identity values describe authored property names;
//! they do not apply cascade, substitute variables, or resolve authored values.

use crate::syntax::*;

macro_rules! property_schema {
    ($callback:ident, $input:ident, $numeric:ident) => {
        // Flexbox 1 normative Appendix B supplies the twelve name-equivalent
        // -webkit- aliases; their canonical property definitions keep their own sources.
        $callback! {
            $input, $numeric;
            All, "all", [], "baseline.property.all", crate::CssAllDeclaredValue, parse_all_property, { parse_all_property($input)? }, expansion = universal { exclude_custom: true, excluded: [Direction, UnicodeBidi] };
            ContainerName, "container-name", [], "official.property.container-name", crate::CssContainerNames, CssContainerNamePropertyValue, names, parse_container_names, { parse_container_names($input)? }, expansion = longhand { value: CssContainerNames, accessor: names, inherited: false, initial_kind: value, initial: CssContainerNames::None };
            ContainerType, "container-type", [], "official.property.container-type", crate::CssContainerType, CssContainerTypePropertyValue, container_type, parse_container_type, { parse_container_type($input)? }, expansion = longhand { value: CssContainerType, accessor: container_type, inherited: false, initial_kind: value, initial: CssContainerType::Normal };
            Container, "container", [], "official.property.container", crate::CssContainer, CssContainerPropertyValue, container, parse_container, { parse_container($input)? }, expansion = shorthand { accessor: container, members: [ ContainerName => |value: &CssContainer| Some(value.names().clone()), ContainerType => |value: &CssContainer| Some(value.container_type()) ], reset_only: [] };
            Display, "display", [], "baseline.property.display", crate::CssDisplayValue, CssDisplayPropertyValue, value, parse_display, { parse_display($input)? }, expansion = longhand { value: crate::CssDisplayValue, accessor: value, inherited: false, initial_kind: value, initial: crate::CssDisplayValue::OutsideInside { outside: crate::CssDisplayOutside::Inline, inside: crate::CssDisplayInside::Flow } };
            BoxSizing, "box-sizing", [], "baseline.property.box-sizing", crate::CssBoxSizing, CssBoxSizingPropertyValue, value, parse_box_sizing, { parse_box_sizing($input)? }, expansion = longhand { value: CssBoxSizing, accessor: value, inherited: false, initial_kind: value, initial: CssBoxSizing::ContentBox };
            BorderCollapse, "border-collapse", [], "official.property.border-collapse", crate::CssBorderCollapse, CssBorderCollapsePropertyValue, collapse, parse_border_collapse, { parse_border_collapse($input)? }, expansion = longhand { value: CssBorderCollapse, accessor: collapse, inherited: true, initial_kind: value, initial: CssBorderCollapse::Separate };
            BorderSpacing, "border-spacing", [], "official.property.border-spacing", crate::CssBorderSpacing, CssBorderSpacingPropertyValue, spacing, parse_border_spacing, { parse_border_spacing($input, $numeric)? }, expansion = longhand { value: CssBorderSpacing, accessor: spacing, inherited: true, initial_kind: value, initial: CssBorderSpacing::new(crate::CssSpecifiedNonNegativeLength::zero(), crate::CssSpecifiedNonNegativeLength::zero()) };
            CaptionSide, "caption-side", [], "official.property.caption-side", crate::CssCaptionSide, CssCaptionSidePropertyValue, side, parse_caption_side, { parse_caption_side($input)? }, expansion = longhand { value: CssCaptionSide, accessor: side, inherited: true, initial_kind: value, initial: CssCaptionSide::Top };
            Clip, "clip", [], "official.property.clip", crate::CssClip, CssClipPropertyValue, clip, parse_clip, { parse_clip($input, $numeric)? }, expansion = longhand { value: CssClip, accessor: clip, inherited: false, initial_kind: value, initial: CssClip::Auto };
            EmptyCells, "empty-cells", [], "official.property.empty-cells", crate::CssEmptyCells, CssEmptyCellsPropertyValue, cells, parse_empty_cells, { parse_empty_cells($input)? }, expansion = longhand { value: CssEmptyCells, accessor: cells, inherited: true, initial_kind: value, initial: CssEmptyCells::Show };
            Orphans, "orphans", [], "official.property.orphans", crate::CssPageLineMinimum, CssOrphansPropertyValue, minimum, parse_page_line_minimum, { parse_page_line_minimum($input, $numeric, "orphans")? }, expansion = longhand { value: CssPageLineMinimum, accessor: minimum, inherited: true, initial_kind: value, initial: CssPageLineMinimum::initial() };
            BreakAfter, "break-after", [], "official.property.break-after", crate::CssBreakBetween, CssBreakAfterPropertyValue, value, parse_break_between, { parse_break_between($input)? }, expansion = longhand { value: CssBreakBetween, accessor: value, inherited: false, initial_kind: value, initial: CssBreakBetween::Auto };
            BreakBefore, "break-before", [], "official.property.break-before", crate::CssBreakBetween, CssBreakBeforePropertyValue, value, parse_break_between, { parse_break_between($input)? }, expansion = longhand { value: CssBreakBetween, accessor: value, inherited: false, initial_kind: value, initial: CssBreakBetween::Auto };
            BreakInside, "break-inside", [], "official.property.break-inside", crate::CssBreakInside, CssBreakInsidePropertyValue, value, parse_break_inside, { parse_break_inside($input)? }, expansion = longhand { value: CssBreakInside, accessor: value, inherited: false, initial_kind: value, initial: CssBreakInside::Auto };
            Quotes, "quotes", [], "official.property.quotes", crate::CssQuotes, CssQuotesPropertyValue, quotes, parse_quotes, { parse_quotes($input)? }, expansion = longhand { value: CssQuotes, accessor: quotes, inherited: true, initial_kind: value, initial: CssQuotes::Auto };
            TableLayout, "table-layout", [], "official.property.table-layout", crate::CssTableLayout, CssTableLayoutPropertyValue, layout, parse_table_layout, { parse_table_layout($input)? }, expansion = longhand { value: CssTableLayout, accessor: layout, inherited: false, initial_kind: value, initial: CssTableLayout::Auto };
            ScrollSnapType, "scroll-snap-type", [], "official.property.scroll-snap-type", crate::CssScrollSnapType, CssScrollSnapTypePropertyValue, value, parse_scroll_snap_type, { parse_scroll_snap_type($input)? }, expansion = longhand { value: CssScrollSnapType, accessor: value, inherited: false, initial_kind: value, initial: CssScrollSnapType::None };
            ScrollSnapAlign, "scroll-snap-align", [], "official.property.scroll-snap-align", crate::CssScrollSnapAlign, CssScrollSnapAlignPropertyValue, value, parse_scroll_snap_align, { parse_scroll_snap_align($input)? }, expansion = longhand { value: CssScrollSnapAlign, accessor: value, inherited: false, initial_kind: value, initial: CssScrollSnapAlign::new(CssScrollSnapAlignment::None, None) };
            ScrollSnapStop, "scroll-snap-stop", [], "official.property.scroll-snap-stop", crate::CssScrollSnapStop, CssScrollSnapStopPropertyValue, value, parse_scroll_snap_stop, { parse_scroll_snap_stop($input)? }, expansion = longhand { value: CssScrollSnapStop, accessor: value, inherited: false, initial_kind: value, initial: CssScrollSnapStop::Normal };
            ScrollPaddingTop, "scroll-padding-top", [], "official.property.scroll-padding-top", crate::CssScrollPaddingValue, CssScrollPaddingTopPropertyValue, value, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: CssScrollPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingRight, "scroll-padding-right", [], "official.property.scroll-padding-right", crate::CssScrollPaddingValue, CssScrollPaddingRightPropertyValue, value, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: CssScrollPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingBottom, "scroll-padding-bottom", [], "official.property.scroll-padding-bottom", crate::CssScrollPaddingValue, CssScrollPaddingBottomPropertyValue, value, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: CssScrollPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingLeft, "scroll-padding-left", [], "official.property.scroll-padding-left", crate::CssScrollPaddingValue, CssScrollPaddingLeftPropertyValue, value, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: CssScrollPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingBlockStart, "scroll-padding-block-start", [], "official.property.scroll-padding-block-start", crate::CssScrollPaddingValue, CssScrollPaddingBlockStartPropertyValue, value, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: CssScrollPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingBlockEnd, "scroll-padding-block-end", [], "official.property.scroll-padding-block-end", crate::CssScrollPaddingValue, CssScrollPaddingBlockEndPropertyValue, value, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: CssScrollPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingInlineStart, "scroll-padding-inline-start", [], "official.property.scroll-padding-inline-start", crate::CssScrollPaddingValue, CssScrollPaddingInlineStartPropertyValue, value, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: CssScrollPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollPaddingInlineEnd, "scroll-padding-inline-end", [], "official.property.scroll-padding-inline-end", crate::CssScrollPaddingValue, CssScrollPaddingInlineEndPropertyValue, value, parse_scroll_padding_value, { parse_scroll_padding_value($input, $numeric)? }, expansion = longhand { value: CssScrollPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssScrollPaddingValue::Auto };
            ScrollMarginTop, "scroll-margin-top", [], "official.property.scroll-margin-top", crate::CssSpecifiedLength, CssScrollMarginTopPropertyValue, value, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssSpecifiedLength, accessor: value, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginRight, "scroll-margin-right", [], "official.property.scroll-margin-right", crate::CssSpecifiedLength, CssScrollMarginRightPropertyValue, value, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssSpecifiedLength, accessor: value, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginBottom, "scroll-margin-bottom", [], "official.property.scroll-margin-bottom", crate::CssSpecifiedLength, CssScrollMarginBottomPropertyValue, value, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssSpecifiedLength, accessor: value, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginLeft, "scroll-margin-left", [], "official.property.scroll-margin-left", crate::CssSpecifiedLength, CssScrollMarginLeftPropertyValue, value, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssSpecifiedLength, accessor: value, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginBlockStart, "scroll-margin-block-start", [], "official.property.scroll-margin-block-start", crate::CssSpecifiedLength, CssScrollMarginBlockStartPropertyValue, value, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssSpecifiedLength, accessor: value, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginBlockEnd, "scroll-margin-block-end", [], "official.property.scroll-margin-block-end", crate::CssSpecifiedLength, CssScrollMarginBlockEndPropertyValue, value, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssSpecifiedLength, accessor: value, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginInlineStart, "scroll-margin-inline-start", [], "official.property.scroll-margin-inline-start", crate::CssSpecifiedLength, CssScrollMarginInlineStartPropertyValue, value, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssSpecifiedLength, accessor: value, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollMarginInlineEnd, "scroll-margin-inline-end", [], "official.property.scroll-margin-inline-end", crate::CssSpecifiedLength, CssScrollMarginInlineEndPropertyValue, value, parse_scroll_margin_length, { parse_scroll_margin_length($input, $numeric)? }, expansion = longhand { value: CssSpecifiedLength, accessor: value, inherited: false, initial_kind: value, initial: CssSpecifiedLength::zero() };
            ScrollPaddingBlock, "scroll-padding-block", [], "official.property.scroll-padding-block", crate::CssScrollPaddingPair, CssScrollPaddingBlockPropertyValue, value, parse_scroll_padding_pair, { parse_scroll_padding_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ ScrollPaddingBlockStart => |value: &CssScrollPaddingPair| Some(value.start().clone()), ScrollPaddingBlockEnd => |value: &CssScrollPaddingPair| Some(value.end().clone()) ], reset_only: [] };
            ScrollPaddingInline, "scroll-padding-inline", [], "official.property.scroll-padding-inline", crate::CssScrollPaddingPair, CssScrollPaddingInlinePropertyValue, value, parse_scroll_padding_pair, { parse_scroll_padding_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ ScrollPaddingInlineStart => |value: &CssScrollPaddingPair| Some(value.start().clone()), ScrollPaddingInlineEnd => |value: &CssScrollPaddingPair| Some(value.end().clone()) ], reset_only: [] };
            ScrollMarginBlock, "scroll-margin-block", [], "official.property.scroll-margin-block", crate::CssScrollMarginPair, CssScrollMarginBlockPropertyValue, value, parse_scroll_margin_pair, { parse_scroll_margin_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ ScrollMarginBlockStart => |value: &CssScrollMarginPair| Some(value.start().clone()), ScrollMarginBlockEnd => |value: &CssScrollMarginPair| Some(value.end().clone()) ], reset_only: [] };
            ScrollMarginInline, "scroll-margin-inline", [], "official.property.scroll-margin-inline", crate::CssScrollMarginPair, CssScrollMarginInlinePropertyValue, value, parse_scroll_margin_pair, { parse_scroll_margin_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ ScrollMarginInlineStart => |value: &CssScrollMarginPair| Some(value.start().clone()), ScrollMarginInlineEnd => |value: &CssScrollMarginPair| Some(value.end().clone()) ], reset_only: [] };
            ScrollPadding, "scroll-padding", [], "official.property.scroll-padding", crate::CssScrollPaddingShorthand, CssScrollPaddingPropertyValue, value, parse_scroll_padding_shorthand, { parse_scroll_padding_shorthand($input, $numeric)? }, expansion = four_side { accessor: value, mode: |value: &CssScrollPaddingShorthand| match value.kind() { crate::CssScrollSideKind::Physical => crate::CssBoxSideKind::Physical, crate::CssScrollSideKind::Logical => crate::CssBoxSideKind::Logical }, physical: [ ScrollPaddingTop => |value: &CssScrollPaddingShorthand| value.role(0).expect("four checked roles").clone(), ScrollPaddingRight => |value: &CssScrollPaddingShorthand| value.role(1).expect("four checked roles").clone(), ScrollPaddingBottom => |value: &CssScrollPaddingShorthand| value.role(2).expect("four checked roles").clone(), ScrollPaddingLeft => |value: &CssScrollPaddingShorthand| value.role(3).expect("four checked roles").clone() ], logical: [ ScrollPaddingBlockStart => |value: &CssScrollPaddingShorthand| value.role(0).expect("four checked roles").clone(), ScrollPaddingInlineStart => |value: &CssScrollPaddingShorthand| value.role(1).expect("four checked roles").clone(), ScrollPaddingBlockEnd => |value: &CssScrollPaddingShorthand| value.role(2).expect("four checked roles").clone(), ScrollPaddingInlineEnd => |value: &CssScrollPaddingShorthand| value.role(3).expect("four checked roles").clone() ] };
            ScrollMargin, "scroll-margin", [], "official.property.scroll-margin", crate::CssScrollMarginShorthand, CssScrollMarginPropertyValue, value, parse_scroll_margin_shorthand, { parse_scroll_margin_shorthand($input, $numeric)? }, expansion = four_side { accessor: value, mode: |value: &CssScrollMarginShorthand| match value.kind() { crate::CssScrollSideKind::Physical => crate::CssBoxSideKind::Physical, crate::CssScrollSideKind::Logical => crate::CssBoxSideKind::Logical }, physical: [ ScrollMarginTop => |value: &CssScrollMarginShorthand| value.role(0).expect("four checked roles").clone(), ScrollMarginRight => |value: &CssScrollMarginShorthand| value.role(1).expect("four checked roles").clone(), ScrollMarginBottom => |value: &CssScrollMarginShorthand| value.role(2).expect("four checked roles").clone(), ScrollMarginLeft => |value: &CssScrollMarginShorthand| value.role(3).expect("four checked roles").clone() ], logical: [ ScrollMarginBlockStart => |value: &CssScrollMarginShorthand| value.role(0).expect("four checked roles").clone(), ScrollMarginInlineStart => |value: &CssScrollMarginShorthand| value.role(1).expect("four checked roles").clone(), ScrollMarginBlockEnd => |value: &CssScrollMarginShorthand| value.role(2).expect("four checked roles").clone(), ScrollMarginInlineEnd => |value: &CssScrollMarginShorthand| value.role(3).expect("four checked roles").clone() ] };
            Widows, "widows", [], "official.property.widows", crate::CssPageLineMinimum, CssWidowsPropertyValue, minimum, parse_page_line_minimum, { parse_page_line_minimum($input, $numeric, "widows")? }, expansion = longhand { value: CssPageLineMinimum, accessor: minimum, inherited: true, initial_kind: value, initial: CssPageLineMinimum::initial() };
            WordSpacing, "word-spacing", [], "official.property.word-spacing", crate::CssTextSpacingAdjustment, CssWordSpacingPropertyValue, spacing, parse_word_spacing, { parse_word_spacing($input, $numeric)? }, expansion = longhand { value: crate::CssTextSpacingAdjustment, accessor: spacing, inherited: true, initial_kind: value, initial: crate::CssTextSpacingAdjustment::Normal };
            Position, "position", [], "baseline.property.position", crate::CssLayoutPosition, CssPositionPropertyValue, value, parse_position, { parse_position($input)? }, expansion = longhand { value: CssLayoutPosition, accessor: value, inherited: false, initial_kind: value, initial: CssLayoutPosition::Static };
            Direction, "direction", [], "baseline.property.direction", crate::CssDirection, CssDirectionPropertyValue, value, parse_direction, { parse_direction($input)? }, expansion = longhand { value: CssDirection, accessor: value, inherited: true, initial_kind: value, initial: CssDirection::Ltr };
            Overflow, "overflow", [], "baseline.property.overflow", crate::CssOverflowValue, CssOverflowPropertyValue, value, parse_overflow_value, { parse_overflow_value($input)? }, expansion = shorthand { accessor: value, members: [ OverflowX => |value: &CssOverflowValue| Some(value.x()), OverflowY => |value: &CssOverflowValue| Some(value.y()) ], reset_only: [] };
            OverflowX, "overflow-x", [], "baseline.property.overflow-x", crate::CssOverflow, CssOverflowXPropertyValue, value, parse_overflow, { parse_overflow($input)? }, expansion = longhand { value: CssOverflow, accessor: value, inherited: false, initial_kind: value, initial: CssOverflow::Visible };
            OverflowY, "overflow-y", [], "baseline.property.overflow-y", crate::CssOverflow, CssOverflowYPropertyValue, value, parse_overflow, { parse_overflow($input)? }, expansion = longhand { value: CssOverflow, accessor: value, inherited: false, initial_kind: value, initial: CssOverflow::Visible };
            OverflowBlock, "overflow-block", [], "ext.property.overflow-block", crate::CssOverflow, CssOverflowBlockPropertyValue, value, parse_overflow, { parse_overflow($input)? }, expansion = longhand { value: CssOverflow, accessor: value, inherited: false, initial_kind: value, initial: CssOverflow::Visible };
            OverflowInline, "overflow-inline", [], "ext.property.overflow-inline", crate::CssOverflow, CssOverflowInlinePropertyValue, value, parse_overflow, { parse_overflow($input)? }, expansion = longhand { value: CssOverflow, accessor: value, inherited: false, initial_kind: value, initial: CssOverflow::Visible };
            OverflowClipMargin, "overflow-clip-margin", [], "ext.property.overflow-clip-margin", crate::CssOverflowClipMargin, CssOverflowClipMarginPropertyValue, value, parse_overflow_clip_margin, { parse_overflow_clip_margin($input, $numeric)? }, expansion = longhand { value: CssOverflowClipMargin, accessor: value, inherited: false, initial_kind: value, initial: CssOverflowClipMargin::initial() };
            WillChange, "will-change", [], "ext.property.will-change", crate::CssWillChange, CssWillChangePropertyValue, value, parse_will_change, { parse_will_change($input)? }, expansion = longhand { value: crate::CssWillChange, accessor: value, inherited: false, initial_kind: value, initial: crate::CssWillChange::Auto };
            OverflowAnchor, "overflow-anchor", [], "ext.property.overflow-anchor", crate::CssOverflowAnchor, CssOverflowAnchorPropertyValue, value, parse_overflow_anchor, { parse_overflow_anchor($input)? }, expansion = longhand { value: crate::CssOverflowAnchor, accessor: value, inherited: false, initial_kind: value, initial: crate::CssOverflowAnchor::Auto };
            ScrollBehavior, "scroll-behavior", [], "ext.property.scroll-behavior", crate::CssScrollBehavior, CssScrollBehaviorPropertyValue, value, parse_scroll_behavior, { parse_scroll_behavior($input)? }, expansion = longhand { value: CssScrollBehavior, accessor: value, inherited: false, initial_kind: value, initial: CssScrollBehavior::Auto };
            ScrollbarGutter, "scrollbar-gutter", [], "ext.property.scrollbar-gutter", crate::CssScrollbarGutter, CssScrollbarGutterPropertyValue, value, parse_scrollbar_gutter, { parse_scrollbar_gutter($input)? }, expansion = longhand { value: CssScrollbarGutter, accessor: value, inherited: false, initial_kind: value, initial: CssScrollbarGutter::Auto };
            VoiceDuration, "voice-duration", [], "official.property.voice-duration", crate::CssVoiceDuration, CssVoiceDurationPropertyValue, value, parse_voice_duration, { parse_voice_duration($input, $numeric)? }, expansion = longhand { value: CssVoiceDuration, accessor: value, inherited: false, initial_kind: value, initial: CssVoiceDuration::Auto };
            VoiceBalance, "voice-balance", [], "official.property.voice-balance", crate::CssVoiceBalance, CssVoiceBalancePropertyValue, value, parse_voice_balance, { parse_voice_balance($input, $numeric)? }, expansion = longhand { value: CssVoiceBalance, accessor: value, inherited: true, initial_kind: value, initial: CssVoiceBalance::from_keyword(CssVoiceBalanceKeyword::Center) };
            VoiceVolume, "voice-volume", [], "official.property.voice-volume", crate::CssVoiceVolume, CssVoiceVolumePropertyValue, value, parse_voice_volume, { parse_voice_volume($input, $numeric)? }, expansion = longhand { value: CssVoiceVolume, accessor: value, inherited: true, initial_kind: value, initial: CssVoiceVolume::Level { level: CssVoiceVolumeLevel::Medium, decibel: None } };
            VoicePitch, "voice-pitch", [], "official.property.voice-pitch", crate::CssVoicePitchRange, CssVoicePitchPropertyValue, value, parse_voice_pitch_range, { parse_voice_pitch_range($input, $numeric)? }, expansion = longhand { value: CssVoicePitchRange, accessor: value, inherited: true, initial_kind: value, initial: CssVoicePitchRange::level_only(CssVoiceLevel::Medium) };
            VoiceRange, "voice-range", [], "official.property.voice-range", crate::CssVoicePitchRange, CssVoiceRangePropertyValue, value, parse_voice_pitch_range, { parse_voice_pitch_range($input, $numeric)? }, expansion = longhand { value: CssVoicePitchRange, accessor: value, inherited: true, initial_kind: value, initial: CssVoicePitchRange::level_only(CssVoiceLevel::Medium) };
            VoiceRate, "voice-rate", [], "official.property.voice-rate", crate::CssVoiceRate, CssVoiceRatePropertyValue, value, parse_voice_rate, { parse_voice_rate($input, $numeric)? }, expansion = longhand { value: CssVoiceRate, accessor: value, inherited: true, initial_kind: value, initial: CssVoiceRate::keyword_only(CssVoiceRateKeyword::Normal) };
            VoiceFamily, "voice-family", [], "official.property.voice-family", crate::CssVoiceFamily, CssVoiceFamilyPropertyValue, value, parse_voice_family, { parse_voice_family($input, $numeric)? }, expansion = longhand { value: CssVoiceFamily, accessor: value, inherited: true, initial_kind: user_agent, initial: CssUserAgentInitial::VoiceFamily };
            VoiceStress, "voice-stress", [], "official.property.voice-stress", crate::CssVoiceStress, CssVoiceStressPropertyValue, value, parse_voice_stress, { parse_voice_stress($input)? }, expansion = longhand { value: CssVoiceStress, accessor: value, inherited: true, initial_kind: value, initial: CssVoiceStress::Normal };
            Speak, "speak", [], "official.property.speak", crate::CssSpeak, CssSpeakPropertyValue, value, parse_speak, { parse_speak($input)? }, expansion = longhand { value: CssSpeak, accessor: value, inherited: true, initial_kind: value, initial: CssSpeak::Auto };
            SpeakAs, "speak-as", [], "official.property.speak-as", crate::CssSpeakAs, CssSpeakAsPropertyValue, value, parse_speak_as, { parse_speak_as($input)? }, expansion = longhand { value: CssSpeakAs, accessor: value, inherited: true, initial_kind: value, initial: CssSpeakAs::normal() };
            PauseBefore, "pause-before", [], "official.property.pause-before", crate::CssSpeechBreak, CssPauseBeforePropertyValue, value, parse_speech_break, { parse_speech_break($input, $numeric)? }, expansion = longhand { value: CssSpeechBreak, accessor: value, inherited: false, initial_kind: value, initial: CssSpeechBreak::None };
            PauseAfter, "pause-after", [], "official.property.pause-after", crate::CssSpeechBreak, CssPauseAfterPropertyValue, value, parse_speech_break, { parse_speech_break($input, $numeric)? }, expansion = longhand { value: CssSpeechBreak, accessor: value, inherited: false, initial_kind: value, initial: CssSpeechBreak::None };
            Pause, "pause", [], "official.property.pause", crate::CssSpeechBreakPair, CssPausePropertyValue, value, parse_speech_break_pair, { parse_speech_break_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ PauseBefore => |value: &CssSpeechBreakPair| Some(value.before().clone()), PauseAfter => |value: &CssSpeechBreakPair| Some(value.after().clone()) ], reset_only: [] };
            RestBefore, "rest-before", [], "official.property.rest-before", crate::CssSpeechBreak, CssRestBeforePropertyValue, value, parse_speech_break, { parse_speech_break($input, $numeric)? }, expansion = longhand { value: CssSpeechBreak, accessor: value, inherited: false, initial_kind: value, initial: CssSpeechBreak::None };
            RestAfter, "rest-after", [], "official.property.rest-after", crate::CssSpeechBreak, CssRestAfterPropertyValue, value, parse_speech_break, { parse_speech_break($input, $numeric)? }, expansion = longhand { value: CssSpeechBreak, accessor: value, inherited: false, initial_kind: value, initial: CssSpeechBreak::None };
            Rest, "rest", [], "official.property.rest", crate::CssSpeechBreakPair, CssRestPropertyValue, value, parse_speech_break_pair, { parse_speech_break_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ RestBefore => |value: &CssSpeechBreakPair| Some(value.before().clone()), RestAfter => |value: &CssSpeechBreakPair| Some(value.after().clone()) ], reset_only: [] };
            CueBefore, "cue-before", [], "official.property.cue-before", crate::CssCue, CssCueBeforePropertyValue, value, parse_cue, { parse_cue($input, $numeric)? }, expansion = longhand { value: CssCue, accessor: value, inherited: false, initial_kind: value, initial: CssCue::None };
            CueAfter, "cue-after", [], "official.property.cue-after", crate::CssCue, CssCueAfterPropertyValue, value, parse_cue, { parse_cue($input, $numeric)? }, expansion = longhand { value: CssCue, accessor: value, inherited: false, initial_kind: value, initial: CssCue::None };
            Cue, "cue", [], "official.property.cue", crate::CssCuePair, CssCuePropertyValue, value, parse_cue_pair, { parse_cue_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ CueBefore => |value: &CssCuePair| Some(value.before().clone()), CueAfter => |value: &CssCuePair| Some(value.after().clone()) ], reset_only: [] };
            FlexDirection, "flex-direction", ["-webkit-flex-direction"], "baseline.property.flex-direction", crate::CssFlexDirection, CssFlexDirectionPropertyValue, value, parse_flex_direction, { parse_flex_direction($input)? }, expansion = longhand { value: CssFlexDirection, accessor: value, inherited: false, initial_kind: value, initial: CssFlexDirection::Row };
            FlexFlow, "flex-flow", ["-webkit-flex-flow"], "official.property.flex-flow", crate::CssFlexFlow, CssFlexFlowPropertyValue, flow, parse_flex_flow, { parse_flex_flow($input)? }, expansion = shorthand { accessor: flow, members: [ FlexDirection => |value: &CssFlexFlow| Some(value.direction()), FlexWrap => |value: &CssFlexFlow| Some(value.wrap()) ], reset_only: [] };
            FlexWrap, "flex-wrap", ["-webkit-flex-wrap"], "baseline.property.flex-wrap", crate::CssFlexWrap, CssFlexWrapPropertyValue, value, parse_flex_wrap, { parse_flex_wrap($input)? }, expansion = longhand { value: CssFlexWrap, accessor: value, inherited: false, initial_kind: value, initial: CssFlexWrap::NoWrap };
            Float, "float", [], "baseline.property.float", crate::CssFloat, CssFloatPropertyValue, value, parse_float, { parse_float($input)? }, expansion = longhand { value: CssFloat, accessor: value, inherited: false, initial_kind: value, initial: CssFloat::None };
            Clear, "clear", [], "baseline.property.clear", crate::CssClear, CssClearPropertyValue, value, parse_clear, { parse_clear($input)? }, expansion = longhand { value: CssClear, accessor: value, inherited: false, initial_kind: value, initial: CssClear::None };
            AlignContent, "align-content", ["-webkit-align-content"], "baseline.property.align-content", crate::CssAlignContentValue, CssAlignContentPropertyValue, value, parse_align_content_value, { parse_align_content_value($input)? }, expansion = longhand { value: CssAlignContentValue, accessor: value, inherited: false, initial_kind: value, initial: CssAlignContentValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid align-content initial") };
            JustifyContent, "justify-content", ["-webkit-justify-content"], "baseline.property.justify-content", crate::CssJustifyContentValue, CssJustifyContentPropertyValue, value, parse_justify_content_value, { parse_justify_content_value($input)? }, expansion = longhand { value: CssJustifyContentValue, accessor: value, inherited: false, initial_kind: value, initial: CssJustifyContentValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid justify-content initial") };
            AlignItems, "align-items", ["-webkit-align-items"], "baseline.property.align-items", crate::CssAlignItemsValue, CssAlignItemsPropertyValue, value, parse_align_items_value, { parse_align_items_value($input)? }, expansion = longhand { value: CssAlignItemsValue, accessor: value, inherited: false, initial_kind: value, initial: CssAlignItemsValue::try_new(CssAlignmentValue::Normal { overflow: None }).expect("valid align-items initial") };
            AlignSelf, "align-self", ["-webkit-align-self"], "baseline.property.align-self", crate::CssAlignSelfValue, CssAlignSelfPropertyValue, value, parse_align_self_value, { parse_align_self_value($input)? }, expansion = longhand { value: CssAlignSelfValue, accessor: value, inherited: false, initial_kind: value, initial: CssAlignSelfValue::try_new(CssAlignmentValue::Auto).expect("valid align-self initial") };
            JustifyItems, "justify-items", [], "baseline.property.justify-items", crate::CssJustifyItemsValue, CssJustifyItemsPropertyValue, value, parse_justify_items_value, { parse_justify_items_value($input)? }, expansion = longhand { value: CssJustifyItemsValue, accessor: value, inherited: false, initial_kind: value, initial: CssJustifyItemsValue::try_new(CssAlignmentValue::Legacy(None)).expect("valid justify-items initial") };
            JustifySelf, "justify-self", [], "baseline.property.justify-self", crate::CssJustifySelfValue, CssJustifySelfPropertyValue, value, parse_justify_self_value, { parse_justify_self_value($input)? }, expansion = longhand { value: CssJustifySelfValue, accessor: value, inherited: false, initial_kind: value, initial: CssJustifySelfValue::try_new(CssAlignmentValue::Auto).expect("valid justify-self initial") };
            PlaceContent, "place-content", [], "baseline.property.place-content", crate::CssPlaceContentValue, CssPlaceContentPropertyValue, value, parse_place_content_value, { parse_place_content_value($input)? }, expansion = shorthand { accessor: value, members: [ AlignContent => |value: &CssPlaceContentValue| Some(value.align()), JustifyContent => |value: &CssPlaceContentValue| Some(value.justify()) ], reset_only: [] };
            PlaceItems, "place-items", [], "baseline.property.place-items", crate::CssPlaceItemsValue, CssPlaceItemsPropertyValue, value, parse_place_items_value, { parse_place_items_value($input)? }, expansion = shorthand { accessor: value, members: [ AlignItems => |value: &CssPlaceItemsValue| Some(value.align()), JustifyItems => |value: &CssPlaceItemsValue| Some(value.justify()) ], reset_only: [] };
            PlaceSelf, "place-self", [], "baseline.property.place-self", crate::CssPlaceSelfValue, CssPlaceSelfPropertyValue, value, parse_place_self_value, { parse_place_self_value($input)? }, expansion = shorthand { accessor: value, members: [ AlignSelf => |value: &CssPlaceSelfValue| Some(value.align()), JustifySelf => |value: &CssPlaceSelfValue| Some(value.justify()) ], reset_only: [] };
            Visibility, "visibility", [], "baseline.property.visibility", crate::CssVisibility, CssVisibilityPropertyValue, value, parse_visibility, { parse_visibility($input)? }, expansion = longhand { value: CssVisibility, accessor: value, inherited: true, initial_kind: value, initial: CssVisibility::Visible };
            Content, "content", [], "baseline.property.content", crate::CssContentValue, CssContentPropertyValue, value, parse_content, { parse_content($input, $numeric)? }, expansion = longhand { value: crate::CssContentValue, accessor: value, inherited: false, initial_kind: value, initial: crate::CssContentValue::Normal };
            ContentVisibility, "content-visibility", [], "baseline.property.content-visibility", crate::CssContentVisibility, CssContentVisibilityPropertyValue, value, parse_content_visibility, { parse_content_visibility($input)? }, expansion = longhand { value: CssContentVisibility, accessor: value, inherited: false, initial_kind: value, initial: CssContentVisibility::Visible };
            ListStyleType, "list-style-type", [], "baseline.property.list-style-type", crate::CssListStyleTypeValue, CssListStyleTypePropertyValue, value, parse_list_style_type, { parse_list_style_type($input, $numeric)? }, expansion = longhand { value: crate::CssListStyleTypeValue, accessor: value, inherited: true, initial_kind: value, initial: crate::CssListStyleTypeValue::initial() };
            ListStylePosition, "list-style-position", [], "baseline.property.list-style-position", crate::CssListStylePosition, CssListStylePositionPropertyValue, value, parse_list_style_position, { parse_list_style_position($input)? }, expansion = longhand { value: CssListStylePosition, accessor: value, inherited: true, initial_kind: value, initial: CssListStylePosition::Outside };
            ListStyleImage, "list-style-image", [], "baseline.property.list-style-image", crate::CssImageValue, CssListStyleImagePropertyValue, value, parse_list_style_image, { parse_list_style_image($input, $numeric)? }, expansion = longhand { value: CssImageValue, accessor: value, inherited: true, initial_kind: value, initial: CssImageValue::None };
            ListStyle, "list-style", [], "baseline.property.list-style", crate::CssListStyleValue, CssListStylePropertyValue, value, parse_list_style, { parse_list_style($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ ListStylePosition => |value: &crate::CssListStyleValue| value.position(), ListStyleImage => |value: &crate::CssListStyleValue| value.image().cloned(), ListStyleType => |value: &crate::CssListStyleValue| value.style_type().cloned() ], reset_only: [] };
            MarkerSide, "marker-side", [], "official.property.marker-side", crate::CssMarkerSide, CssMarkerSidePropertyValue, value, parse_marker_side, { parse_marker_side($input)? }, expansion = longhand { value: crate::CssMarkerSide, accessor: value, inherited: true, initial_kind: value, initial: crate::CssMarkerSide::MatchSelf };
            CounterReset, "counter-reset", [], "baseline.property.counter-reset", crate::CssCounterChangesValue, CssCounterResetPropertyValue, value, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { value: crate::CssCounterChangesValue, accessor: value, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            CounterIncrement, "counter-increment", [], "baseline.property.counter-increment", crate::CssCounterChangesValue, CssCounterIncrementPropertyValue, value, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { value: crate::CssCounterChangesValue, accessor: value, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            CounterSet, "counter-set", [], "baseline.property.counter-set", crate::CssCounterChangesValue, CssCounterSetPropertyValue, value, parse_counter_changes, { parse_counter_changes($input, $numeric)? }, expansion = longhand { value: crate::CssCounterChangesValue, accessor: value, inherited: false, initial_kind: value, initial: crate::CssCounterChangesValue::none() };
            Width, "width", [], "baseline.property.width", crate::CssSizeValue, CssWidthPropertyValue, value, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            Height, "height", [], "baseline.property.height", crate::CssSizeValue, CssHeightPropertyValue, value, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            InlineSize, "inline-size", [], "official.property.inline-size", crate::CssSizeValue, CssInlineSizePropertyValue, value, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            BlockSize, "block-size", [], "official.property.block-size", crate::CssSizeValue, CssBlockSizePropertyValue, value, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MinWidth, "min-width", [], "baseline.property.min-width", crate::CssSizeValue, CssMinWidthPropertyValue, value, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MinHeight, "min-height", [], "baseline.property.min-height", crate::CssSizeValue, CssMinHeightPropertyValue, value, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MinInlineSize, "min-inline-size", [], "official.property.min-inline-size", crate::CssSizeValue, CssMinInlineSizePropertyValue, value, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MinBlockSize, "min-block-size", [], "official.property.min-block-size", crate::CssSizeValue, CssMinBlockSizePropertyValue, value, parse_size_value, { parse_size_value($input, $numeric)? }, expansion = longhand { value: CssSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssSizeValue::Auto };
            MaxWidth, "max-width", [], "baseline.property.max-width", crate::CssMaxSizeValue, CssMaxWidthPropertyValue, value, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { value: CssMaxSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE };
            MaxHeight, "max-height", [], "baseline.property.max-height", crate::CssMaxSizeValue, CssMaxHeightPropertyValue, value, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { value: CssMaxSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE };
            MaxInlineSize, "max-inline-size", [], "official.property.max-inline-size", crate::CssMaxSizeValue, CssMaxInlineSizePropertyValue, value, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { value: CssMaxSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE };
            MaxBlockSize, "max-block-size", [], "official.property.max-block-size", crate::CssMaxSizeValue, CssMaxBlockSizePropertyValue, value, parse_max_size_value, { parse_max_size_value($input, $numeric)? }, expansion = longhand { value: CssMaxSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssMaxSizeValue::NONE };
            Size, "size", [], "ext.property.size", crate::CssSizePair, CssSizePropertyValue, value, parse_size_pair, { parse_size_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ Width => |value: &CssSizePair| Some(value.width().clone()), Height => |value: &CssSizePair| Some(value.height().clone()) ], reset_only: [] };
            MinSize, "min-size", [], "ext.property.min-size", crate::CssSizePair, CssMinSizePropertyValue, value, parse_size_pair, { parse_size_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ MinWidth => |value: &CssSizePair| Some(value.width().clone()), MinHeight => |value: &CssSizePair| Some(value.height().clone()) ], reset_only: [] };
            MaxSize, "max-size", [], "ext.property.max-size", crate::CssMaxSizePair, CssMaxSizePropertyValue, value, parse_max_size_pair, { parse_max_size_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ MaxWidth => |value: &CssMaxSizePair| Some(value.width().clone()), MaxHeight => |value: &CssMaxSizePair| Some(value.height().clone()) ], reset_only: [] };
            FrameSizing, "frame-sizing", [], "ext.property.frame-sizing", crate::CssFrameSizing, CssFrameSizingPropertyValue, value, parse_frame_sizing, { parse_frame_sizing($input)? }, expansion = longhand { value: CssFrameSizing, accessor: value, inherited: false, initial_kind: value, initial: CssFrameSizing::Auto };
            MinIntrinsicSizing, "min-intrinsic-sizing", [], "ext.property.min-intrinsic-sizing", crate::CssMinIntrinsicSizing, CssMinIntrinsicSizingPropertyValue, value, parse_min_intrinsic_sizing, { parse_min_intrinsic_sizing($input)? }, expansion = longhand { value: CssMinIntrinsicSizing, accessor: value, inherited: false, initial_kind: value, initial: CssMinIntrinsicSizing::Legacy };
            ContainIntrinsicWidth, "contain-intrinsic-width", [], "ext.property.contain-intrinsic-width", crate::CssContainIntrinsicSizeValue, CssContainIntrinsicWidthPropertyValue, value, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { value: CssContainIntrinsicSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None) };
            ContainIntrinsicHeight, "contain-intrinsic-height", [], "ext.property.contain-intrinsic-height", crate::CssContainIntrinsicSizeValue, CssContainIntrinsicHeightPropertyValue, value, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { value: CssContainIntrinsicSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None) };
            ContainIntrinsicInlineSize, "contain-intrinsic-inline-size", [], "ext.property.contain-intrinsic-inline-size", crate::CssContainIntrinsicSizeValue, CssContainIntrinsicInlineSizePropertyValue, value, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { value: CssContainIntrinsicSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None) };
            ContainIntrinsicBlockSize, "contain-intrinsic-block-size", [], "ext.property.contain-intrinsic-block-size", crate::CssContainIntrinsicSizeValue, CssContainIntrinsicBlockSizePropertyValue, value, parse_contain_intrinsic_size_value, { parse_contain_intrinsic_size_value($input, $numeric)? }, expansion = longhand { value: CssContainIntrinsicSizeValue, accessor: value, inherited: false, initial_kind: value, initial: CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None) };
            ContainIntrinsicSize, "contain-intrinsic-size", [], "ext.property.contain-intrinsic-size", crate::CssContainIntrinsicSize, CssContainIntrinsicSizePropertyValue, value, parse_contain_intrinsic_size, { parse_contain_intrinsic_size($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ ContainIntrinsicWidth => |value: &CssContainIntrinsicSize| Some(value.width().clone()), ContainIntrinsicHeight => |value: &CssContainIntrinsicSize| Some(value.height().clone()) ], reset_only: [] };
            FlexBasis, "flex-basis", ["-webkit-flex-basis"], "baseline.property.flex-basis", crate::CssFlexBasisValue, CssFlexBasisPropertyValue, value, parse_flex_basis, { parse_flex_basis($input, $numeric)? }, expansion = longhand { value: CssFlexBasisValue, accessor: value, inherited: false, initial_kind: value, initial: CssFlexBasisValue::from(CssSizeValue::Auto) };
            Gap, "gap", ["grid-gap"], "baseline.property.gap", crate::CssGapShorthand, CssGapPropertyValue, value, parse_gap_shorthand, { parse_gap_shorthand($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ RowGap => |value: &CssGapShorthand| Some(value.row().clone()), ColumnGap => |value: &CssGapShorthand| Some(value.column().clone()) ], reset_only: [] };
            RowGap, "row-gap", ["grid-row-gap"], "baseline.property.row-gap", crate::CssGapValue, CssRowGapPropertyValue, value, parse_gap_value, { parse_gap_value($input, $numeric)? }, expansion = longhand { value: CssGapValue, accessor: value, inherited: false, initial_kind: value, initial: CssGapValue::Normal };
            ColumnGap, "column-gap", ["grid-column-gap"], "baseline.property.column-gap", crate::CssGapValue, CssColumnGapPropertyValue, value, parse_gap_value, { parse_gap_value($input, $numeric)? }, expansion = longhand { value: CssGapValue, accessor: value, inherited: false, initial_kind: value, initial: CssGapValue::Normal };
            ColumnCount, "column-count", [], "official.property.column-count", crate::CssColumnCount, CssColumnCountPropertyValue, count, parse_column_count, { parse_column_count($input, $numeric)? }, expansion = longhand { value: CssColumnCount, accessor: count, inherited: false, initial_kind: value, initial: CssColumnCount::Auto };
            ColumnFill, "column-fill", [], "official.property.column-fill", crate::CssColumnFill, CssColumnFillPropertyValue, fill, parse_column_fill, { parse_column_fill($input)? }, expansion = longhand { value: CssColumnFill, accessor: fill, inherited: false, initial_kind: value, initial: CssColumnFill::Balance };
            ColumnRule, "column-rule", [], "official.property.column-rule", crate::CssColumnRule, CssColumnRulePropertyValue, rule, parse_column_rule, { parse_column_rule($input, $numeric)? }, expansion = shorthand { accessor: rule, members: [ ColumnRuleWidth => |value: &CssColumnRule| value.width().cloned(), ColumnRuleStyle => |value: &CssColumnRule| value.style(), ColumnRuleColor => |value: &CssColumnRule| value.color().cloned() ], reset_only: [] };
            ColumnRuleColor, "column-rule-color", [], "official.property.column-rule-color", crate::CssColor, CssColumnRuleColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            ColumnRuleStyle, "column-rule-style", [], "official.property.column-rule-style", crate::CssLineStyle, CssColumnRuleStylePropertyValue, style, parse_line_style, { parse_line_style($input)? }, expansion = longhand { value: CssLineStyle, accessor: style, inherited: false, initial_kind: value, initial: CssLineStyle::None };
            ColumnRuleWidth, "column-rule-width", [], "official.property.column-rule-width", crate::CssLineWidth, CssColumnRuleWidthPropertyValue, width, parse_line_width, { parse_line_width($input, $numeric)? }, expansion = longhand { value: CssLineWidth, accessor: width, inherited: false, initial_kind: value, initial: CssLineWidth::Medium };
            ColumnSpan, "column-span", [], "official.property.column-span", crate::CssColumnSpan, CssColumnSpanPropertyValue, span, parse_column_span, { parse_column_span($input)? }, expansion = longhand { value: CssColumnSpan, accessor: span, inherited: false, initial_kind: value, initial: CssColumnSpan::None };
            ColumnWidth, "column-width", [], "official.property.column-width", crate::CssColumnWidth, CssColumnWidthPropertyValue, width, parse_column_width, { parse_column_width($input, $numeric)? }, expansion = longhand { value: CssColumnWidth, accessor: width, inherited: false, initial_kind: value, initial: CssColumnWidth::Auto };
            Columns, "columns", [], "official.property.columns", crate::CssColumns, CssColumnsPropertyValue, columns, parse_columns, { parse_columns($input, $numeric)? }, expansion = shorthand { accessor: columns, members: [ ColumnWidth => |value: &CssColumns| Some(value.width().clone()), ColumnCount => |value: &CssColumns| Some(value.count().clone()) ], reset_only: [] };
            ItemDirection, "item-direction", [], "ext.property.item-direction", crate::CssItemDirection, CssItemDirectionPropertyValue, value, parse_item_direction, { parse_item_direction($input)? }, expansion = longhand { value: crate::CssItemDirection, accessor: value, inherited: false, initial_kind: value, initial: crate::CssItemDirection::Auto };
            ItemWrap, "item-wrap", [], "ext.property.item-wrap", crate::CssItemWrap, CssItemWrapPropertyValue, value, parse_item_wrap, { parse_item_wrap($input)? }, expansion = longhand { value: crate::CssItemWrap, accessor: value, inherited: false, initial_kind: value, initial: crate::CssItemWrap::default() };
            ItemPack, "item-pack", [], "ext.property.item-pack", crate::CssItemPack, CssItemPackPropertyValue, value, parse_item_pack, { parse_item_pack($input)? }, expansion = longhand { value: crate::CssItemPack, accessor: value, inherited: false, initial_kind: value, initial: crate::CssItemPack::Normal };
            ItemFlow, "item-flow", [], "ext.property.item-flow", crate::CssItemFlow, CssItemFlowPropertyValue, value, parse_item_flow, { parse_item_flow($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ ItemDirection => |value: &crate::CssItemFlow| Some(value.direction()), ItemWrap => |value: &crate::CssItemFlow| Some(value.wrap()), ItemPack => |value: &crate::CssItemFlow| Some(value.pack()), FlowTolerance => |value: &crate::CssItemFlow| Some(value.tolerance().clone()) ], reset_only: [] };
            FlowTolerance, "flow-tolerance", [], "ext.property.flow-tolerance", crate::CssFlowTolerance, CssFlowTolerancePropertyValue, value, parse_flow_tolerance, { parse_flow_tolerance($input, $numeric)? }, expansion = longhand { value: CssFlowTolerance, accessor: value, inherited: false, initial_kind: value, initial: CssFlowTolerance::normal() };
            GridTemplateRows, "grid-template-rows", [], "baseline.property.grid-template-rows", crate::CssGridTrackList, CssGridTemplateRowsPropertyValue, value, parse_grid_track_list, { parse_grid_track_list($input, $numeric)? }, expansion = longhand { value: CssGridTrackList, accessor: value, inherited: false, initial_kind: value, initial: CssGridTrackList::none() };
            GridTemplateColumns, "grid-template-columns", [], "baseline.property.grid-template-columns", crate::CssGridTrackList, CssGridTemplateColumnsPropertyValue, value, parse_grid_track_list, { parse_grid_track_list($input, $numeric)? }, expansion = longhand { value: CssGridTrackList, accessor: value, inherited: false, initial_kind: value, initial: CssGridTrackList::none() };
            GridTemplateAreas, "grid-template-areas", [], "baseline.property.grid-template-areas", crate::CssGridTemplateAreas, CssGridTemplateAreasPropertyValue, value, parse_grid_template_areas, { parse_grid_template_areas($input)? }, expansion = longhand { value: crate::CssGridTemplateAreas, accessor: value, inherited: false, initial_kind: value, initial: crate::CssGridTemplateAreas::None };
            GridTemplate, "grid-template", [], "baseline.property.grid-template", crate::CssGridTemplate, CssGridTemplatePropertyValue, value, parse_grid_template, { parse_grid_template($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ GridTemplateRows => |value: &CssGridTemplate| crate::expansion::grid_template_rows(value), GridTemplateColumns => |value: &CssGridTemplate| crate::expansion::grid_template_columns(value), GridTemplateAreas => |value: &CssGridTemplate| crate::expansion::grid_template_areas(value) ], reset_only: [] };
            GridAutoRows, "grid-auto-rows", [], "baseline.property.grid-auto-rows", crate::CssGridTrackSizeList, CssGridAutoRowsPropertyValue, value, parse_grid_auto_track_sizes, { parse_grid_auto_track_sizes($input, $numeric)? }, expansion = longhand { value: CssGridTrackSizeList, accessor: value, inherited: false, initial_kind: value, initial: CssGridTrackSizeList::try_new(vec![CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto())]).expect("one auto initial track size") };
            GridAutoColumns, "grid-auto-columns", [], "baseline.property.grid-auto-columns", crate::CssGridTrackSizeList, CssGridAutoColumnsPropertyValue, value, parse_grid_auto_track_sizes, { parse_grid_auto_track_sizes($input, $numeric)? }, expansion = longhand { value: CssGridTrackSizeList, accessor: value, inherited: false, initial_kind: value, initial: CssGridTrackSizeList::try_new(vec![CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto())]).expect("one auto initial track size") };
            GridAutoFlow, "grid-auto-flow", [], "baseline.property.grid-auto-flow", crate::CssGridAutoFlow, CssGridAutoFlowPropertyValue, value, parse_grid_auto_flow, { parse_grid_auto_flow($input)? }, expansion = longhand { value: CssGridAutoFlow, accessor: value, inherited: false, initial_kind: value, initial: CssGridAutoFlow::Normal };
            GridRowStart, "grid-row-start", [], "baseline.property.grid-row-start", crate::CssGridLine, CssGridRowStartPropertyValue, value, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { value: CssGridLine, accessor: value, inherited: false, initial_kind: value, initial: CssGridLine::Auto };
            GridRowEnd, "grid-row-end", [], "baseline.property.grid-row-end", crate::CssGridLine, CssGridRowEndPropertyValue, value, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { value: CssGridLine, accessor: value, inherited: false, initial_kind: value, initial: CssGridLine::Auto };
            GridColumnStart, "grid-column-start", [], "baseline.property.grid-column-start", crate::CssGridLine, CssGridColumnStartPropertyValue, value, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { value: CssGridLine, accessor: value, inherited: false, initial_kind: value, initial: CssGridLine::Auto };
            GridColumnEnd, "grid-column-end", [], "baseline.property.grid-column-end", crate::CssGridLine, CssGridColumnEndPropertyValue, value, parse_grid_line, { parse_grid_line($input, $numeric)? }, expansion = longhand { value: CssGridLine, accessor: value, inherited: false, initial_kind: value, initial: CssGridLine::Auto };
            GridRow, "grid-row", [], "baseline.property.grid-row", crate::CssGridLineRange, CssGridRowPropertyValue, value, parse_grid_line_range, { parse_grid_line_range($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ GridRowStart => |value: &CssGridLineRange| Some(value.start().clone()), GridRowEnd => |value: &CssGridLineRange| Some(value.effective_end()) ], reset_only: [] };
            GridColumn, "grid-column", [], "baseline.property.grid-column", crate::CssGridLineRange, CssGridColumnPropertyValue, value, parse_grid_line_range, { parse_grid_line_range($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ GridColumnStart => |value: &CssGridLineRange| Some(value.start().clone()), GridColumnEnd => |value: &CssGridLineRange| Some(value.effective_end()) ], reset_only: [] };
            GridArea, "grid-area", [], "baseline.property.grid-area", crate::CssGridArea, CssGridAreaPropertyValue, value, parse_grid_area, { parse_grid_area($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ GridRowStart => |value: &CssGridArea| Some(value.row_start().clone()), GridColumnStart => |value: &CssGridArea| Some(value.effective_column_start()), GridRowEnd => |value: &CssGridArea| Some(value.effective_row_end()), GridColumnEnd => |value: &CssGridArea| Some(value.effective_column_end()) ], reset_only: [] };
            Grid, "grid", [], "baseline.property.grid", crate::CssGrid, CssGridPropertyValue, value, parse_grid, { parse_grid($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ GridTemplateRows => |value: &CssGrid| crate::expansion::grid_rows(value), GridTemplateColumns => |value: &CssGrid| crate::expansion::grid_columns(value), GridTemplateAreas => |value: &CssGrid| crate::expansion::grid_areas(value), GridAutoRows => |value: &CssGrid| crate::expansion::grid_auto_rows(value), GridAutoColumns => |value: &CssGrid| crate::expansion::grid_auto_columns(value), GridAutoFlow => |value: &CssGrid| crate::expansion::grid_auto_flow(value) ], reset_only: [] };
            FontSize, "font-size", [], "baseline.property.font-size", crate::CssFontSize, CssFontSizePropertyValue, size, parse_font_size, { parse_font_size($input, $numeric)? }, expansion = longhand { value: CssFontSize, accessor: size, inherited: true, initial_kind: value, initial: CssFontSize::Medium };
            LineHeight, "line-height", [], "baseline.property.line-height", crate::CssLineHeight, CssLineHeightPropertyValue, line_height, parse_line_height, { parse_line_height($input, $numeric)? }, expansion = longhand { value: CssLineHeight, accessor: line_height, inherited: true, initial_kind: value, initial: CssLineHeight::Normal };
            TextCombineUpright, "text-combine-upright", [], "official.property.text-combine-upright", crate::CssTextCombineUpright, CssTextCombineUprightPropertyValue, combine, parse_text_combine_upright, { parse_text_combine_upright($input, $numeric)? }, expansion = longhand { value: CssTextCombineUpright, accessor: combine, inherited: true, initial_kind: value, initial: CssTextCombineUpright::None };
            TextOrientation, "text-orientation", [], "official.property.text-orientation", crate::CssTextOrientation, CssTextOrientationPropertyValue, orientation, parse_text_orientation, { parse_text_orientation($input)? }, expansion = longhand { value: CssTextOrientation, accessor: orientation, inherited: true, initial_kind: value, initial: CssTextOrientation::Mixed };
            UnicodeBidi, "unicode-bidi", [], "official.property.unicode-bidi", crate::CssUnicodeBidi, CssUnicodeBidiPropertyValue, bidi, parse_unicode_bidi, { parse_unicode_bidi($input)? }, expansion = longhand { value: CssUnicodeBidi, accessor: bidi, inherited: false, initial_kind: value, initial: CssUnicodeBidi::Normal };
            WritingMode, "writing-mode", [], "baseline.property.writing-mode", crate::CssWritingMode, CssWritingModePropertyValue, value, parse_writing_mode, { parse_writing_mode($input)? }, expansion = longhand { value: CssWritingMode, accessor: value, inherited: true, initial_kind: value, initial: CssWritingMode::HorizontalTb };
            TextAlign, "text-align", [], "baseline.property.text-align", crate::CssTextAlignValue, CssTextAlignPropertyValue, value, parse_text_align, { parse_text_align($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ TextAlignAll => |value: &CssTextAlignValue| Some(crate::text_alignment::shorthand_all(value)), TextAlignLast => |value: &CssTextAlignValue| Some(crate::text_alignment::shorthand_last(value)) ], reset_only: [] };
            TextAlignAll, "text-align-all", [], "official.property.text-align-all", crate::CssTextAlignAllValue, CssTextAlignAllPropertyValue, value, parse_text_align_all, { parse_text_align_all($input, $numeric)? }, expansion = longhand { value: CssTextAlignAllValue, accessor: value, inherited: true, initial_kind: value, initial: CssTextAlignAllValue::initial() };
            TextAlignLast, "text-align-last", [], "baseline.property.text-align-last", crate::CssTextAlignLastValue, CssTextAlignLastPropertyValue, value, parse_text_align_last, { parse_text_align_last($input)? }, expansion = longhand { value: CssTextAlignLastValue, accessor: value, inherited: true, initial_kind: value, initial: CssTextAlignLastValue::initial() };
            TextIndent, "text-indent", [], "baseline.property.text-indent", crate::CssTextIndent, CssTextIndentPropertyValue, value, parse_text_indent, { parse_text_indent($input, $numeric)? }, expansion = longhand { value: CssTextIndent, accessor: value, inherited: true, initial_kind: value, initial: CssTextIndent::new(CssSpecifiedLengthPercentage::zero(), false, false) };
            VerticalAlign, "vertical-align", [], "baseline.property.vertical-align", crate::CssVerticalAlign, CssVerticalAlignPropertyValue, value, parse_vertical_align, { parse_vertical_align($input, $numeric)? }, expansion = longhand { value: CssVerticalAlign, accessor: value, inherited: false, initial_kind: value, initial: CssVerticalAlign::Baseline };
            FontFamily, "font-family", [], "baseline.property.font-family", crate::CssFontFamilyList, CssFontFamilyPropertyValue, families, parse_font_family_list, { parse_font_family_list($input)? }, expansion = longhand { value: CssFontFamilyList, accessor: families, inherited: true, initial_kind: user_agent, initial: CssUserAgentInitial::FontFamily };
            Font, "font", [], "baseline.property.font", crate::CssFontValue, CssFontPropertyValue, font, parse_font, { parse_font($input, $numeric)? }, expansion = shorthand { accessor: font, members: [ FontFamily => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => Some(font.families().clone()), CssFontValue::System(_) => None }, FontSize => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => Some(font.size().clone()), CssFontValue::System(_) => None }, FontWidth => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.stretch().map(CssFontWidth::Keyword), CssFontValue::System(_) => None }, FontStyle => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.style().cloned(), CssFontValue::System(_) => None }, FontVariantCaps => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.variant().map(|variant| match variant { CssFontVariant::Normal => CssFontVariantCaps::Normal, CssFontVariant::SmallCaps => CssFontVariantCaps::SmallCaps }), CssFontValue::System(_) => None }, FontWeight => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.weight().cloned(), CssFontValue::System(_) => None }, LineHeight => |value: &CssFontValue| match value { CssFontValue::Explicit(font) => font.line_height().cloned(), CssFontValue::System(_) => None } ], reset_only: [FontFeatureSettings, FontKerning, FontLanguageOverride, FontOpticalSizing, FontSizeAdjust, FontVariantAlternates, FontVariantEastAsian, FontVariantEmoji, FontVariantLigatures, FontVariantNumeric, FontVariantPosition, FontVariationSettings] };
            FontWeight, "font-weight", [], "baseline.property.font-weight", crate::CssFontWeight, CssFontWeightPropertyValue, value, parse_font_weight, { parse_font_weight($input, $numeric)? }, expansion = longhand { value: CssFontWeight, accessor: value, inherited: true, initial_kind: value, initial: CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal) };
            FontStyle, "font-style", [], "baseline.property.font-style", crate::CssFontStyle, CssFontStylePropertyValue, value, parse_font_style, { parse_font_style($input, $numeric)? }, expansion = longhand { value: CssFontStyle, accessor: value, inherited: true, initial_kind: value, initial: CssFontStyle::Keyword(CssFontStyleKeyword::Normal) };
            FontWidth, "font-width", ["font-stretch"], "baseline.property.font-stretch", crate::CssFontWidth, CssFontWidthPropertyValue, value, parse_font_width, { parse_font_width($input, $numeric)? }, expansion = longhand { value: CssFontWidth, accessor: value, inherited: true, initial_kind: value, initial: CssFontWidth::Keyword(CssFontWidthKeyword::Normal) };
            FontVariant, "font-variant", [], "baseline.property.font-variant", crate::CssFontVariantValue, CssFontVariantPropertyValue, variant, parse_font_variant, { parse_font_variant($input)? }, expansion = shorthand { accessor: variant, members: [ FontVariantLigatures => |value: &CssFontVariantValue| Some(value.expanded_ligatures()), FontVariantCaps => |value: &CssFontVariantValue| Some(value.expanded_caps()), FontVariantAlternates => |value: &CssFontVariantValue| Some(value.expanded_alternates()), FontVariantNumeric => |value: &CssFontVariantValue| Some(value.expanded_numeric()), FontVariantEastAsian => |value: &CssFontVariantValue| Some(value.expanded_east_asian()), FontVariantPosition => |value: &CssFontVariantValue| Some(value.expanded_position()), FontVariantEmoji => |value: &CssFontVariantValue| Some(value.expanded_emoji()) ], reset_only: [] };
            FontVariantCaps, "font-variant-caps", [], "official.property.font-variant-caps", crate::CssFontVariantCaps, CssFontVariantCapsPropertyValue, caps, parse_font_variant_caps, { parse_font_variant_caps($input)? }, expansion = longhand { value: CssFontVariantCaps, accessor: caps, inherited: true, initial_kind: value, initial: CssFontVariantCaps::Normal };
            FontVariantEastAsian, "font-variant-east-asian", [], "official.property.font-variant-east-asian", crate::CssFontVariantEastAsian, CssFontVariantEastAsianPropertyValue, east_asian, parse_font_variant_east_asian, { parse_font_variant_east_asian($input)? }, expansion = longhand { value: CssFontVariantEastAsian, accessor: east_asian, inherited: true, initial_kind: value, initial: CssFontVariantEastAsian::Normal };
            FontVariantLigatures, "font-variant-ligatures", [], "official.property.font-variant-ligatures", crate::CssFontVariantLigatures, CssFontVariantLigaturesPropertyValue, ligatures, parse_font_variant_ligatures, { parse_font_variant_ligatures($input)? }, expansion = longhand { value: CssFontVariantLigatures, accessor: ligatures, inherited: true, initial_kind: value, initial: CssFontVariantLigatures::Normal };
            FontVariantNumeric, "font-variant-numeric", [], "official.property.font-variant-numeric", crate::CssFontVariantNumeric, CssFontVariantNumericPropertyValue, numeric, parse_font_variant_numeric, { parse_font_variant_numeric($input)? }, expansion = longhand { value: CssFontVariantNumeric, accessor: numeric, inherited: true, initial_kind: value, initial: CssFontVariantNumeric::Normal };
            FontVariantPosition, "font-variant-position", [], "official.property.font-variant-position", crate::CssFontVariantPosition, CssFontVariantPositionPropertyValue, position, parse_font_variant_position, { parse_font_variant_position($input)? }, expansion = longhand { value: CssFontVariantPosition, accessor: position, inherited: true, initial_kind: value, initial: CssFontVariantPosition::Normal };
            FontVariantAlternates, "font-variant-alternates", [], "official.property.font-variant-alternates", crate::CssFontVariantAlternates, CssFontVariantAlternatesPropertyValue, alternates, parse_font_variant_alternates, { parse_font_variant_alternates($input)? }, expansion = longhand { value: CssFontVariantAlternates, accessor: alternates, inherited: true, initial_kind: value, initial: CssFontVariantAlternates::Normal };
            FontVariantEmoji, "font-variant-emoji", [], "official.property.font-variant-emoji", crate::CssFontVariantEmoji, CssFontVariantEmojiPropertyValue, emoji, parse_font_variant_emoji, { parse_font_variant_emoji($input)? }, expansion = longhand { value: CssFontVariantEmoji, accessor: emoji, inherited: true, initial_kind: value, initial: CssFontVariantEmoji::Normal };
            FontFeatureSettings, "font-feature-settings", [], "baseline.property.font-feature-settings", crate::CssAuthoredFontFeatureSettings, CssFontFeatureSettingsPropertyValue, settings, parse_font_feature_settings, { parse_font_feature_settings($input, $numeric)? }, expansion = longhand { value: CssAuthoredFontFeatureSettings, accessor: settings, inherited: true, initial_kind: value, initial: CssAuthoredFontFeatureSettings::Normal };
            FontKerning, "font-kerning", [], "official.property.font-kerning", crate::CssFontKerning, CssFontKerningPropertyValue, kerning, parse_font_kerning, { parse_font_kerning($input)? }, expansion = longhand { value: CssFontKerning, accessor: kerning, inherited: true, initial_kind: value, initial: CssFontKerning::Auto };
            FontSizeAdjust, "font-size-adjust", [], "official.property.font-size-adjust", crate::CssFontSizeAdjust, CssFontSizeAdjustPropertyValue, size_adjust, parse_font_size_adjust, { parse_font_size_adjust($input, $numeric)? }, expansion = longhand { value: CssFontSizeAdjust, accessor: size_adjust, inherited: true, initial_kind: value, initial: CssFontSizeAdjust::None };
            FontLanguageOverride, "font-language-override", [], "official.property.font-language-override", crate::CssFontLanguageOverride, CssFontLanguageOverridePropertyValue, language_override, parse_font_language_override, { parse_font_language_override($input, $numeric)? }, expansion = longhand { value: CssFontLanguageOverride, accessor: language_override, inherited: true, initial_kind: value, initial: CssFontLanguageOverride::Normal };
            FontOpticalSizing, "font-optical-sizing", [], "official.property.font-optical-sizing", crate::CssFontOpticalSizing, CssFontOpticalSizingPropertyValue, optical_sizing, parse_font_optical_sizing, { parse_font_optical_sizing($input)? }, expansion = longhand { value: CssFontOpticalSizing, accessor: optical_sizing, inherited: true, initial_kind: value, initial: CssFontOpticalSizing::Auto };
            FontVariationSettings, "font-variation-settings", [], "official.property.font-variation-settings", crate::CssFontVariationSettings, CssFontVariationSettingsPropertyValue, variations, parse_font_variation_settings, { parse_font_variation_settings($input, $numeric)? }, expansion = longhand { value: CssFontVariationSettings, accessor: variations, inherited: true, initial_kind: value, initial: CssFontVariationSettings::Normal };
            FontSynthesis, "font-synthesis", [], "official.property.font-synthesis", crate::CssFontSynthesis, CssFontSynthesisPropertyValue, synthesis, parse_font_synthesis, { parse_font_synthesis($input)? }, expansion = shorthand { accessor: synthesis, members: [ FontSynthesisWeight => |value: &CssFontSynthesis| Some(value.expanded_weight()), FontSynthesisStyle => |value: &CssFontSynthesis| Some(value.expanded_style()), FontSynthesisSmallCaps => |value: &CssFontSynthesis| Some(value.expanded_small_caps()), FontSynthesisPosition => |value: &CssFontSynthesis| Some(value.expanded_position()) ], reset_only: [] };
            FontSynthesisWeight, "font-synthesis-weight", [], "official.property.font-synthesis-weight", crate::CssFontSynthesisWeight, CssFontSynthesisWeightPropertyValue, weight, parse_font_synthesis_weight, { parse_font_synthesis_weight($input)? }, expansion = longhand { value: CssFontSynthesisWeight, accessor: weight, inherited: true, initial_kind: value, initial: CssFontSynthesisWeight::Auto };
            FontSynthesisStyle, "font-synthesis-style", [], "official.property.font-synthesis-style", crate::CssFontSynthesisStyle, CssFontSynthesisStylePropertyValue, style, parse_font_synthesis_style, { parse_font_synthesis_style($input)? }, expansion = longhand { value: CssFontSynthesisStyle, accessor: style, inherited: true, initial_kind: value, initial: CssFontSynthesisStyle::Auto };
            FontSynthesisSmallCaps, "font-synthesis-small-caps", [], "official.property.font-synthesis-small-caps", crate::CssFontSynthesisSmallCaps, CssFontSynthesisSmallCapsPropertyValue, small_caps, parse_font_synthesis_small_caps, { parse_font_synthesis_small_caps($input)? }, expansion = longhand { value: CssFontSynthesisSmallCaps, accessor: small_caps, inherited: true, initial_kind: value, initial: CssFontSynthesisSmallCaps::Auto };
            FontSynthesisPosition, "font-synthesis-position", [], "official.property.font-synthesis-position", crate::CssFontSynthesisPosition, CssFontSynthesisPositionPropertyValue, position, parse_font_synthesis_position, { parse_font_synthesis_position($input)? }, expansion = longhand { value: CssFontSynthesisPosition, accessor: position, inherited: true, initial_kind: value, initial: CssFontSynthesisPosition::Auto };
            FontPalette, "font-palette", [], "official.property.font-palette", crate::CssFontPalette, CssFontPalettePropertyValue, palette, parse_font_palette, { parse_font_palette($input, $numeric)? }, expansion = longhand { value: CssFontPalette, accessor: palette, inherited: true, initial_kind: value, initial: CssFontPalette::Normal };
            LetterSpacing, "letter-spacing", [], "baseline.property.letter-spacing", crate::CssTextSpacingAdjustment, CssLetterSpacingPropertyValue, value, parse_letter_spacing, { parse_letter_spacing($input, $numeric)? }, expansion = longhand { value: crate::CssTextSpacingAdjustment, accessor: value, inherited: true, initial_kind: value, initial: crate::CssTextSpacingAdjustment::Normal };
            TextWrap, "text-wrap", [], "baseline.property.text-wrap", crate::CssTextWrap, CssTextWrapPropertyValue, value, parse_text_wrap, { parse_text_wrap($input)? }, expansion = shorthand { accessor: value, members: [ TextWrapMode => |value: &CssTextWrap| value.mode(), TextWrapStyle => |value: &CssTextWrap| value.style() ], reset_only: [] };
            TextWrapMode, "text-wrap-mode", [], "ext.property.text-wrap-mode", crate::CssTextWrapMode, CssTextWrapModePropertyValue, value, parse_text_wrap_mode, { parse_text_wrap_mode($input)? }, expansion = longhand { value: CssTextWrapMode, accessor: value, inherited: true, initial_kind: value, initial: CssTextWrapMode::Wrap };
            TextWrapStyle, "text-wrap-style", [], "ext.property.text-wrap-style", crate::CssTextWrapStyle, CssTextWrapStylePropertyValue, value, parse_text_wrap_style, { parse_text_wrap_style($input)? }, expansion = longhand { value: CssTextWrapStyle, accessor: value, inherited: true, initial_kind: value, initial: CssTextWrapStyle::Auto };
            WhiteSpace, "white-space", [], "baseline.property.white-space", crate::CssWhiteSpace, CssWhiteSpacePropertyValue, value, parse_white_space, { parse_white_space($input)? }, expansion = shorthand { accessor: value, members: [ WhiteSpaceCollapse => |value: &CssWhiteSpace| crate::expansion::white_space_collapse(value), TextWrapMode => |value: &CssWhiteSpace| crate::expansion::white_space_mode(value), WhiteSpaceTrim => |value: &CssWhiteSpace| crate::expansion::white_space_trim(value) ], reset_only: [] };
            WhiteSpaceCollapse, "white-space-collapse", [], "ext.property.white-space-collapse", crate::CssWhiteSpaceCollapse, CssWhiteSpaceCollapsePropertyValue, value, parse_white_space_collapse, { parse_white_space_collapse($input)? }, expansion = longhand { value: CssWhiteSpaceCollapse, accessor: value, inherited: true, initial_kind: value, initial: CssWhiteSpaceCollapse::Collapse };
            WhiteSpaceTrim, "white-space-trim", [], "ext.property.white-space-trim", crate::CssWhiteSpaceTrim, CssWhiteSpaceTrimPropertyValue, value, parse_white_space_trim, { parse_white_space_trim($input)? }, expansion = longhand { value: CssWhiteSpaceTrim, accessor: value, inherited: false, initial_kind: value, initial: CssWhiteSpaceTrim::none() };
            WordBreak, "word-break", [], "baseline.property.word-break", crate::CssWordBreak, CssWordBreakPropertyValue, value, parse_word_break, { parse_word_break($input)? }, expansion = longhand { value: CssWordBreak, accessor: value, inherited: true, initial_kind: value, initial: CssWordBreak::Normal };
            OverflowWrap, "overflow-wrap", ["word-wrap"], "baseline.property.overflow-wrap", crate::CssOverflowWrap, CssOverflowWrapPropertyValue, value, parse_overflow_wrap, { parse_overflow_wrap($input)? }, expansion = longhand { value: CssOverflowWrap, accessor: value, inherited: true, initial_kind: value, initial: CssOverflowWrap::Normal };
            TextOverflow, "text-overflow", [], "baseline.property.text-overflow", crate::CssTextOverflow, CssTextOverflowPropertyValue, value, parse_text_overflow, { parse_text_overflow($input)? }, expansion = longhand { value: CssTextOverflow, accessor: value, inherited: false, initial_kind: value, initial: CssTextOverflow::Clip };
            TextDecoration, "text-decoration", [], "baseline.property.text-decoration", crate::CssTextDecoration, CssTextDecorationPropertyValue, value, parse_text_decoration, { parse_text_decoration($input, $numeric)? };
            TextDecorationLine, "text-decoration-line", [], "baseline.property.text-decoration-line", crate::CssTextDecorationLine, CssTextDecorationLinePropertyValue, value, parse_text_decoration_line, { parse_text_decoration_line($input)? };
            TextDecorationColor, "text-decoration-color", [], "baseline.property.text-decoration-color", crate::CssColor, CssTextDecorationColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? };
            TextDecorationStyle, "text-decoration-style", [], "baseline.property.text-decoration-style", crate::CssTextDecorationStyle, CssTextDecorationStylePropertyValue, value, parse_text_decoration_style, { parse_text_decoration_style($input)? };
            TextDecorationThickness, "text-decoration-thickness", [], "baseline.property.text-decoration-thickness", crate::CssTextDecorationThickness, CssTextDecorationThicknessPropertyValue, value, parse_text_decoration_thickness, { parse_text_decoration_thickness($input, $numeric)? };
            WrapInside, "wrap-inside", [], "ext.property.wrap-inside", crate::CssWrapInside, CssWrapInsidePropertyValue, value, parse_wrap_inside, { parse_wrap_inside($input)? }, expansion = longhand { value: CssWrapInside, accessor: value, inherited: false, initial_kind: value, initial: CssWrapInside::Auto };
            WrapBefore, "wrap-before", [], "ext.property.wrap-before", crate::CssWrapBoundary, CssWrapBeforePropertyValue, value, parse_wrap_boundary, { parse_wrap_boundary($input)? }, expansion = longhand { value: CssWrapBoundary, accessor: value, inherited: false, initial_kind: value, initial: CssWrapBoundary::Auto };
            WrapAfter, "wrap-after", [], "ext.property.wrap-after", crate::CssWrapBoundary, CssWrapAfterPropertyValue, value, parse_wrap_boundary, { parse_wrap_boundary($input)? }, expansion = longhand { value: CssWrapBoundary, accessor: value, inherited: false, initial_kind: value, initial: CssWrapBoundary::Auto };
            LineBreak, "line-break", [], "ext.property.line-break", crate::CssLineBreak, CssLineBreakPropertyValue, value, parse_line_break, { parse_line_break($input)? }, expansion = longhand { value: CssLineBreak, accessor: value, inherited: true, initial_kind: value, initial: CssLineBreak::Auto };
            WordSpaceTransform, "word-space-transform", [], "ext.property.word-space-transform", crate::CssWordSpaceTransform, CssWordSpaceTransformPropertyValue, value, parse_word_space_transform, { parse_word_space_transform($input)? }, expansion = longhand { value: CssWordSpaceTransform, accessor: value, inherited: true, initial_kind: value, initial: CssWordSpaceTransform::None };
            TabSize, "tab-size", [], "ext.property.tab-size", crate::CssTabSize, CssTabSizePropertyValue, value, parse_tab_size, { parse_tab_size($input, $numeric)? }, expansion = longhand { value: CssTabSize, accessor: value, inherited: true, initial_kind: value, initial: CssTabSize::Number(crate::CssSpecifiedNonNegativeNumber::try_from_component(crate::CssComponentValue::try_number("8").expect("ordinary eight")).expect("nonnegative initial tab interval")) };
            TextTransform, "text-transform", [], "baseline.property.text-transform", crate::CssTextTransform, CssTextTransformPropertyValue, value, parse_text_transform, { parse_text_transform($input)? }, expansion = longhand { value: CssTextTransform, accessor: value, inherited: true, initial_kind: value, initial: CssTextTransform::None };
            Inset, "inset", [], "baseline.property.inset", crate::CssInsetShorthand, CssInsetPropertyValue, value, parse_inset_shorthand, { parse_inset_shorthand($input, $numeric)? }, expansion = four_side { accessor: value, mode: |value: &CssInsetShorthand| value.kind(), physical: [ Top => |value: &CssInsetShorthand| value.assigned_values()[0].clone(), Right => |value: &CssInsetShorthand| value.assigned_values()[1].clone(), Bottom => |value: &CssInsetShorthand| value.assigned_values()[2].clone(), Left => |value: &CssInsetShorthand| value.assigned_values()[3].clone() ], logical: [ InsetBlockStart => |value: &CssInsetShorthand| value.assigned_values()[0].clone(), InsetInlineStart => |value: &CssInsetShorthand| value.assigned_values()[1].clone(), InsetBlockEnd => |value: &CssInsetShorthand| value.assigned_values()[2].clone(), InsetInlineEnd => |value: &CssInsetShorthand| value.assigned_values()[3].clone() ] };
            Top, "top", [], "baseline.property.top", crate::CssInsetValue, CssTopPropertyValue, value, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, accessor: value, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            Right, "right", [], "baseline.property.right", crate::CssInsetValue, CssRightPropertyValue, value, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, accessor: value, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            Bottom, "bottom", [], "baseline.property.bottom", crate::CssInsetValue, CssBottomPropertyValue, value, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, accessor: value, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            Left, "left", [], "baseline.property.left", crate::CssInsetValue, CssLeftPropertyValue, value, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, accessor: value, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetBlockStart, "inset-block-start", [], "official.property.inset-block-start", crate::CssInsetValue, CssInsetBlockStartPropertyValue, value, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, accessor: value, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetBlockEnd, "inset-block-end", [], "official.property.inset-block-end", crate::CssInsetValue, CssInsetBlockEndPropertyValue, value, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, accessor: value, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetInlineStart, "inset-inline-start", [], "official.property.inset-inline-start", crate::CssInsetValue, CssInsetInlineStartPropertyValue, value, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, accessor: value, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetInlineEnd, "inset-inline-end", [], "official.property.inset-inline-end", crate::CssInsetValue, CssInsetInlineEndPropertyValue, value, parse_inset_value, { parse_inset_value($input, $numeric)? }, expansion = longhand { value: CssInsetValue, accessor: value, inherited: false, initial_kind: value, initial: CssInsetValue::Auto };
            InsetBlock, "inset-block", [], "official.property.inset-block", crate::CssInsetPair, CssInsetBlockPropertyValue, value, parse_inset_pair, { parse_inset_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ InsetBlockStart => |value: &CssInsetPair| Some(value.start().clone()), InsetBlockEnd => |value: &CssInsetPair| Some(value.end().clone()) ], reset_only: [] };
            InsetInline, "inset-inline", [], "official.property.inset-inline", crate::CssInsetPair, CssInsetInlinePropertyValue, value, parse_inset_pair, { parse_inset_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ InsetInlineStart => |value: &CssInsetPair| Some(value.start().clone()), InsetInlineEnd => |value: &CssInsetPair| Some(value.end().clone()) ], reset_only: [] };
            ZIndex, "z-index", [], "baseline.property.z-index", crate::CssZIndexValue, CssZIndexPropertyValue, value, parse_z_index, { parse_z_index($input, $numeric)? }, expansion = longhand { value: CssZIndexValue, accessor: value, inherited: false, initial_kind: value, initial: CssZIndexValue::Auto };
            BoxDecorationBreak, "box-decoration-break", [], "baseline.property.box-decoration-break", crate::CssBoxDecorationBreak, CssBoxDecorationBreakPropertyValue, value, parse_box_decoration_break, { parse_box_decoration_break($input)? }, expansion = longhand { value: CssBoxDecorationBreak, accessor: value, inherited: false, initial_kind: value, initial: CssBoxDecorationBreak::Slice };
            Margin, "margin", [], "baseline.property.margin", crate::CssMarginShorthand, CssMarginPropertyValue, value, parse_box_margin_shorthand, { parse_box_margin_shorthand($input, $numeric)? }, expansion = four_side { accessor: value, mode: |value: &CssMarginShorthand| value.kind(), physical: [ MarginTop => |value: &CssMarginShorthand| value.assigned_values()[0].clone(), MarginRight => |value: &CssMarginShorthand| value.assigned_values()[1].clone(), MarginBottom => |value: &CssMarginShorthand| value.assigned_values()[2].clone(), MarginLeft => |value: &CssMarginShorthand| value.assigned_values()[3].clone() ], logical: [ MarginBlockStart => |value: &CssMarginShorthand| value.assigned_values()[0].clone(), MarginInlineStart => |value: &CssMarginShorthand| value.assigned_values()[1].clone(), MarginBlockEnd => |value: &CssMarginShorthand| value.assigned_values()[2].clone(), MarginInlineEnd => |value: &CssMarginShorthand| value.assigned_values()[3].clone() ] };
            MarginTop, "margin-top", [], "baseline.property.margin-top", crate::CssMarginValue, CssMarginTopPropertyValue, value, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, accessor: value, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginRight, "margin-right", [], "baseline.property.margin-right", crate::CssMarginValue, CssMarginRightPropertyValue, value, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, accessor: value, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginBottom, "margin-bottom", [], "baseline.property.margin-bottom", crate::CssMarginValue, CssMarginBottomPropertyValue, value, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, accessor: value, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginLeft, "margin-left", [], "baseline.property.margin-left", crate::CssMarginValue, CssMarginLeftPropertyValue, value, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, accessor: value, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginBlockStart, "margin-block-start", [], "official.property.margin-block-start", crate::CssMarginValue, CssMarginBlockStartPropertyValue, value, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, accessor: value, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginBlockEnd, "margin-block-end", [], "official.property.margin-block-end", crate::CssMarginValue, CssMarginBlockEndPropertyValue, value, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, accessor: value, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginInlineStart, "margin-inline-start", [], "official.property.margin-inline-start", crate::CssMarginValue, CssMarginInlineStartPropertyValue, value, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, accessor: value, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginInlineEnd, "margin-inline-end", [], "official.property.margin-inline-end", crate::CssMarginValue, CssMarginInlineEndPropertyValue, value, parse_box_margin_value, { parse_box_margin_value($input, $numeric)? }, expansion = longhand { value: CssMarginValue, accessor: value, inherited: false, initial_kind: value, initial: CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero()) };
            MarginBlock, "margin-block", [], "official.property.margin-block", crate::CssMarginPair, CssMarginBlockPropertyValue, value, parse_box_margin_pair, { parse_box_margin_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ MarginBlockStart => |value: &CssMarginPair| Some(value.start().clone()), MarginBlockEnd => |value: &CssMarginPair| Some(value.end().clone()) ], reset_only: [] };
            MarginInline, "margin-inline", [], "official.property.margin-inline", crate::CssMarginPair, CssMarginInlinePropertyValue, value, parse_box_margin_pair, { parse_box_margin_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ MarginInlineStart => |value: &CssMarginPair| Some(value.start().clone()), MarginInlineEnd => |value: &CssMarginPair| Some(value.end().clone()) ], reset_only: [] };
            Padding, "padding", [], "baseline.property.padding", crate::CssPaddingShorthand, CssPaddingPropertyValue, value, parse_box_padding_shorthand, { parse_box_padding_shorthand($input, $numeric)? }, expansion = four_side { accessor: value, mode: |value: &CssPaddingShorthand| value.kind(), physical: [ PaddingTop => |value: &CssPaddingShorthand| value.assigned_values()[0].clone(), PaddingRight => |value: &CssPaddingShorthand| value.assigned_values()[1].clone(), PaddingBottom => |value: &CssPaddingShorthand| value.assigned_values()[2].clone(), PaddingLeft => |value: &CssPaddingShorthand| value.assigned_values()[3].clone() ], logical: [ PaddingBlockStart => |value: &CssPaddingShorthand| value.assigned_values()[0].clone(), PaddingInlineStart => |value: &CssPaddingShorthand| value.assigned_values()[1].clone(), PaddingBlockEnd => |value: &CssPaddingShorthand| value.assigned_values()[2].clone(), PaddingInlineEnd => |value: &CssPaddingShorthand| value.assigned_values()[3].clone() ] };
            PaddingTop, "padding-top", [], "baseline.property.padding-top", crate::CssPaddingValue, CssPaddingTopPropertyValue, value, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingRight, "padding-right", [], "baseline.property.padding-right", crate::CssPaddingValue, CssPaddingRightPropertyValue, value, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingBottom, "padding-bottom", [], "baseline.property.padding-bottom", crate::CssPaddingValue, CssPaddingBottomPropertyValue, value, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingLeft, "padding-left", [], "baseline.property.padding-left", crate::CssPaddingValue, CssPaddingLeftPropertyValue, value, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingBlockStart, "padding-block-start", [], "official.property.padding-block-start", crate::CssPaddingValue, CssPaddingBlockStartPropertyValue, value, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingBlockEnd, "padding-block-end", [], "official.property.padding-block-end", crate::CssPaddingValue, CssPaddingBlockEndPropertyValue, value, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingInlineStart, "padding-inline-start", [], "official.property.padding-inline-start", crate::CssPaddingValue, CssPaddingInlineStartPropertyValue, value, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingInlineEnd, "padding-inline-end", [], "official.property.padding-inline-end", crate::CssPaddingValue, CssPaddingInlineEndPropertyValue, value, parse_box_padding_value, { parse_box_padding_value($input, $numeric)? }, expansion = longhand { value: CssPaddingValue, accessor: value, inherited: false, initial_kind: value, initial: CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero()) };
            PaddingBlock, "padding-block", [], "official.property.padding-block", crate::CssPaddingPair, CssPaddingBlockPropertyValue, value, parse_box_padding_pair, { parse_box_padding_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ PaddingBlockStart => |value: &CssPaddingPair| Some(value.start().clone()), PaddingBlockEnd => |value: &CssPaddingPair| Some(value.end().clone()) ], reset_only: [] };
            PaddingInline, "padding-inline", [], "official.property.padding-inline", crate::CssPaddingPair, CssPaddingInlinePropertyValue, value, parse_box_padding_pair, { parse_box_padding_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ PaddingInlineStart => |value: &CssPaddingPair| Some(value.start().clone()), PaddingInlineEnd => |value: &CssPaddingPair| Some(value.end().clone()) ], reset_only: [] };
            Border, "border", [], "baseline.property.border", crate::CssBorder, CssBorderPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderTopWidth => |value: &CssBorder| value.width().cloned(), BorderRightWidth => |value: &CssBorder| value.width().cloned(), BorderBottomWidth => |value: &CssBorder| value.width().cloned(), BorderLeftWidth => |value: &CssBorder| value.width().cloned(), BorderTopStyle => |value: &CssBorder| value.style(), BorderRightStyle => |value: &CssBorder| value.style(), BorderBottomStyle => |value: &CssBorder| value.style(), BorderLeftStyle => |value: &CssBorder| value.style(), BorderTopColor => |value: &CssBorder| value.color().cloned(), BorderRightColor => |value: &CssBorder| value.color().cloned(), BorderBottomColor => |value: &CssBorder| value.color().cloned(), BorderLeftColor => |value: &CssBorder| value.color().cloned() ], reset_only: [ BorderImageSource, BorderImageSlice, BorderImageWidth, BorderImageOutset, BorderImageRepeat ] };
            BorderTop, "border-top", [], "baseline.property.border-top", crate::CssBorder, CssBorderTopPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderTopWidth => |value: &CssBorder| value.width().cloned(), BorderTopStyle => |value: &CssBorder| value.style(), BorderTopColor => |value: &CssBorder| value.color().cloned() ], reset_only: [  ] };
            BorderRight, "border-right", [], "baseline.property.border-right", crate::CssBorder, CssBorderRightPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderRightWidth => |value: &CssBorder| value.width().cloned(), BorderRightStyle => |value: &CssBorder| value.style(), BorderRightColor => |value: &CssBorder| value.color().cloned() ], reset_only: [  ] };
            BorderBottom, "border-bottom", [], "baseline.property.border-bottom", crate::CssBorder, CssBorderBottomPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderBottomWidth => |value: &CssBorder| value.width().cloned(), BorderBottomStyle => |value: &CssBorder| value.style(), BorderBottomColor => |value: &CssBorder| value.color().cloned() ], reset_only: [  ] };
            BorderLeft, "border-left", [], "baseline.property.border-left", crate::CssBorder, CssBorderLeftPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderLeftWidth => |value: &CssBorder| value.width().cloned(), BorderLeftStyle => |value: &CssBorder| value.style(), BorderLeftColor => |value: &CssBorder| value.color().cloned() ], reset_only: [  ] };
            BorderBlockStart, "border-block-start", [], "official.property.border-block-start", crate::CssBorder, CssBorderBlockStartPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderBlockStartWidth => |value: &CssBorder| value.width().cloned(), BorderBlockStartStyle => |value: &CssBorder| value.style(), BorderBlockStartColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderBlockEnd, "border-block-end", [], "official.property.border-block-end", crate::CssBorder, CssBorderBlockEndPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderBlockEndWidth => |value: &CssBorder| value.width().cloned(), BorderBlockEndStyle => |value: &CssBorder| value.style(), BorderBlockEndColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderInlineStart, "border-inline-start", [], "official.property.border-inline-start", crate::CssBorder, CssBorderInlineStartPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderInlineStartWidth => |value: &CssBorder| value.width().cloned(), BorderInlineStartStyle => |value: &CssBorder| value.style(), BorderInlineStartColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderInlineEnd, "border-inline-end", [], "official.property.border-inline-end", crate::CssBorder, CssBorderInlineEndPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderInlineEndWidth => |value: &CssBorder| value.width().cloned(), BorderInlineEndStyle => |value: &CssBorder| value.style(), BorderInlineEndColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderBlock, "border-block", [], "official.property.border-block", crate::CssBorder, CssBorderBlockPropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderBlockStartWidth => |value: &CssBorder| value.width().cloned(), BorderBlockEndWidth => |value: &CssBorder| value.width().cloned(), BorderBlockStartStyle => |value: &CssBorder| value.style(), BorderBlockEndStyle => |value: &CssBorder| value.style(), BorderBlockStartColor => |value: &CssBorder| value.color().cloned(), BorderBlockEndColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderInline, "border-inline", [], "official.property.border-inline", crate::CssBorder, CssBorderInlinePropertyValue, value, parse_border, { parse_border($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderInlineStartWidth => |value: &CssBorder| value.width().cloned(), BorderInlineEndWidth => |value: &CssBorder| value.width().cloned(), BorderInlineStartStyle => |value: &CssBorder| value.style(), BorderInlineEndStyle => |value: &CssBorder| value.style(), BorderInlineStartColor => |value: &CssBorder| value.color().cloned(), BorderInlineEndColor => |value: &CssBorder| value.color().cloned() ], reset_only: [] };
            BorderWidth, "border-width", [], "baseline.property.border-width", crate::CssBorderWidthShorthand, CssBorderWidthPropertyValue, value, parse_exact_border_width_shorthand, { parse_exact_border_width_shorthand($input, $numeric)? }, expansion = four_side { accessor: value, mode: |value: &CssBorderWidthShorthand| value.kind(), physical: [ BorderTopWidth => |value: &CssBorderWidthShorthand| value.assigned_values()[0].clone(), BorderRightWidth => |value: &CssBorderWidthShorthand| value.assigned_values()[1].clone(), BorderBottomWidth => |value: &CssBorderWidthShorthand| value.assigned_values()[2].clone(), BorderLeftWidth => |value: &CssBorderWidthShorthand| value.assigned_values()[3].clone() ], logical: [ BorderBlockStartWidth => |value: &CssBorderWidthShorthand| value.assigned_values()[0].clone(), BorderInlineStartWidth => |value: &CssBorderWidthShorthand| value.assigned_values()[1].clone(), BorderBlockEndWidth => |value: &CssBorderWidthShorthand| value.assigned_values()[2].clone(), BorderInlineEndWidth => |value: &CssBorderWidthShorthand| value.assigned_values()[3].clone() ] };
            BorderTopWidth, "border-top-width", [], "baseline.property.border-top-width", crate::CssBorderWidth, CssBorderTopWidthPropertyValue, value, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssBorderWidth, accessor: value, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderRightWidth, "border-right-width", [], "baseline.property.border-right-width", crate::CssBorderWidth, CssBorderRightWidthPropertyValue, value, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssBorderWidth, accessor: value, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderBottomWidth, "border-bottom-width", [], "baseline.property.border-bottom-width", crate::CssBorderWidth, CssBorderBottomWidthPropertyValue, value, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssBorderWidth, accessor: value, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderLeftWidth, "border-left-width", [], "baseline.property.border-left-width", crate::CssBorderWidth, CssBorderLeftWidthPropertyValue, value, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssBorderWidth, accessor: value, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderBlockStartWidth, "border-block-start-width", [], "official.property.border-block-start-width", crate::CssBorderWidth, CssBorderBlockStartWidthPropertyValue, value, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssBorderWidth, accessor: value, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderBlockEndWidth, "border-block-end-width", [], "official.property.border-block-end-width", crate::CssBorderWidth, CssBorderBlockEndWidthPropertyValue, value, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssBorderWidth, accessor: value, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderInlineStartWidth, "border-inline-start-width", [], "official.property.border-inline-start-width", crate::CssBorderWidth, CssBorderInlineStartWidthPropertyValue, value, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssBorderWidth, accessor: value, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderInlineEndWidth, "border-inline-end-width", [], "official.property.border-inline-end-width", crate::CssBorderWidth, CssBorderInlineEndWidthPropertyValue, value, parse_exact_border_width, { parse_exact_border_width($input, $numeric)? }, expansion = longhand { value: CssBorderWidth, accessor: value, inherited: false, initial_kind: value, initial: CssBorderWidth::Medium };
            BorderBlockWidth, "border-block-width", [], "official.property.border-block-width", crate::CssBorderWidthPair, CssBorderBlockWidthPropertyValue, value, parse_exact_border_width_pair, { parse_exact_border_width_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderBlockStartWidth => |value: &CssBorderWidthPair| Some(value.start().clone()), BorderBlockEndWidth => |value: &CssBorderWidthPair| Some(value.end().clone()) ], reset_only: [] };
            BorderInlineWidth, "border-inline-width", [], "official.property.border-inline-width", crate::CssBorderWidthPair, CssBorderInlineWidthPropertyValue, value, parse_exact_border_width_pair, { parse_exact_border_width_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderInlineStartWidth => |value: &CssBorderWidthPair| Some(value.start().clone()), BorderInlineEndWidth => |value: &CssBorderWidthPair| Some(value.end().clone()) ], reset_only: [] };
            Color, "color", [], "baseline.property.color", crate::CssColor, CssColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: true, initial_kind: value, initial: CssColor::from_system(CssSystemColor::CanvasText) };
            Background, "background", [], "baseline.property.background", crate::CssBackground, CssBackgroundPropertyValue, background, parse_background, { parse_background($input, $numeric)? }, expansion = shorthand { accessor: background, members: [
                BackgroundImage => crate::expansion::background_images,
                BackgroundPosition => crate::expansion::background_positions,
                BackgroundSize => crate::expansion::background_sizes,
                BackgroundRepeat => crate::expansion::background_repeats,
                BackgroundAttachment => crate::expansion::background_attachments,
                BackgroundOrigin => crate::expansion::background_origins,
                BackgroundClip => crate::expansion::background_clips,
                BackgroundColor => |value: &CssBackground| value.layers().last().and_then(CssBackgroundLayer::color).cloned()
            ], reset_only: [BackgroundBlendMode] };
            BackgroundColor, "background-color", [], "baseline.property.background-color", crate::CssColor, CssBackgroundColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::transparent() };
            BorderColor, "border-color", [], "baseline.property.border-color", crate::CssBorderColorShorthand, CssBorderColorPropertyValue, value, parse_border_colors, { parse_border_colors($input, $numeric)? }, expansion = four_side { accessor: value, mode: |value: &CssBorderColorShorthand| value.kind(), physical: [ BorderTopColor => |value: &CssBorderColorShorthand| value.assigned_values()[0].clone(), BorderRightColor => |value: &CssBorderColorShorthand| value.assigned_values()[1].clone(), BorderBottomColor => |value: &CssBorderColorShorthand| value.assigned_values()[2].clone(), BorderLeftColor => |value: &CssBorderColorShorthand| value.assigned_values()[3].clone() ], logical: [ BorderBlockStartColor => |value: &CssBorderColorShorthand| value.assigned_values()[0].clone(), BorderInlineStartColor => |value: &CssBorderColorShorthand| value.assigned_values()[1].clone(), BorderBlockEndColor => |value: &CssBorderColorShorthand| value.assigned_values()[2].clone(), BorderInlineEndColor => |value: &CssBorderColorShorthand| value.assigned_values()[3].clone() ] };
            BorderTopColor, "border-top-color", [], "baseline.property.border-top-color", crate::CssColor, CssBorderTopColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            BorderRightColor, "border-right-color", [], "baseline.property.border-right-color", crate::CssColor, CssBorderRightColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            BorderBottomColor, "border-bottom-color", [], "baseline.property.border-bottom-color", crate::CssColor, CssBorderBottomColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            BorderLeftColor, "border-left-color", [], "baseline.property.border-left-color", crate::CssColor, CssBorderLeftColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            BorderBlockStartColor, "border-block-start-color", [], "official.property.border-block-start-color", crate::CssColor, CssBorderBlockStartColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            BorderBlockEndColor, "border-block-end-color", [], "official.property.border-block-end-color", crate::CssColor, CssBorderBlockEndColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            BorderInlineStartColor, "border-inline-start-color", [], "official.property.border-inline-start-color", crate::CssColor, CssBorderInlineStartColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            BorderInlineEndColor, "border-inline-end-color", [], "official.property.border-inline-end-color", crate::CssColor, CssBorderInlineEndColorPropertyValue, value, parse_color, { parse_color($input, $numeric)? }, expansion = longhand { value: CssColor, accessor: value, inherited: false, initial_kind: value, initial: CssColor::current_color() };
            BorderBlockColor, "border-block-color", [], "official.property.border-block-color", crate::CssBorderColorPair, CssBorderBlockColorPropertyValue, value, parse_border_color_pair, { parse_border_color_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderBlockStartColor => |value: &CssBorderColorPair| Some(value.start().clone()), BorderBlockEndColor => |value: &CssBorderColorPair| Some(value.end().clone()) ], reset_only: [] };
            BorderInlineColor, "border-inline-color", [], "official.property.border-inline-color", crate::CssBorderColorPair, CssBorderInlineColorPropertyValue, value, parse_border_color_pair, { parse_border_color_pair($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderInlineStartColor => |value: &CssBorderColorPair| Some(value.start().clone()), BorderInlineEndColor => |value: &CssBorderColorPair| Some(value.end().clone()) ], reset_only: [] };
            BackgroundImage, "background-image", [], "baseline.property.background-image", crate::CssImageValueList, CssBackgroundImagePropertyValue, images, parse_image_layer_list, { parse_image_layer_list($input, $numeric)? }, expansion = longhand { value: CssImageValueList, accessor: images, inherited: false, initial_kind: value, initial: CssImageValueList::try_new(vec![CssImageValue::None]).expect("one initial image") };
            BackgroundPosition, "background-position", [], "baseline.property.background-position", crate::CssBackgroundPositionList, CssBackgroundPositionPropertyValue, positions, parse_background_position_list, { parse_background_position_list($input, $numeric)? }, expansion = longhand { value: CssBackgroundPositionList, accessor: positions, inherited: false, initial_kind: value, initial: {
                let zero = CssSpecifiedLengthPercentage::try_from_component(crate::CssComponentValue::try_token("0%").expect("percentage token")).expect("zero percentage is a position offset");
                CssBackgroundPositionList::try_new(vec![CssBackgroundPosition::try_new(CssHorizontalPosition::Offset(zero.clone()), CssVerticalPosition::Offset(zero)).expect("two free offsets are a background position")]).expect("one initial position")
            } };
            ObjectPosition, "object-position", [], "official.property.object-position", crate::CssPhysicalPosition, CssObjectPositionPropertyValue, position, parse_object_position, { parse_object_position($input, $numeric)? };
            BackgroundSize, "background-size", [], "baseline.property.background-size", crate::CssBackgroundSizeList, CssBackgroundSizePropertyValue, sizes, parse_background_size_list, { parse_background_size_list($input, $numeric)? }, expansion = longhand { value: CssBackgroundSizeList, accessor: sizes, inherited: false, initial_kind: value, initial: CssBackgroundSizeList::try_new(vec![CssBackgroundSize::Explicit { width: CssBackgroundSizeComponent::Auto, height: None }]).expect("one initial size") };
            BackgroundRepeat, "background-repeat", [], "baseline.property.background-repeat", crate::CssBackgroundRepeatList, CssBackgroundRepeatPropertyValue, repeats, parse_background_repeat_list, { parse_background_repeat_list($input)? }, expansion = longhand { value: CssBackgroundRepeatList, accessor: repeats, inherited: false, initial_kind: value, initial: CssBackgroundRepeatList::try_new(vec![CssBackgroundRepeat::Axes { x: CssBackgroundRepeatStyle::Repeat, y: CssBackgroundRepeatStyle::Repeat }]).expect("one initial repeat") };
            BackgroundOrigin, "background-origin", [], "baseline.property.background-origin", crate::CssBackgroundBoxList, CssBackgroundOriginPropertyValue, boxes, parse_background_box_list, { parse_background_box_list($input)? }, expansion = longhand { value: CssBackgroundBoxList, accessor: boxes, inherited: false, initial_kind: value, initial: CssBackgroundBoxList::try_new(vec![CssBackgroundBox::PaddingBox]).expect("one initial origin box") };
            BackgroundClip, "background-clip", [], "baseline.property.background-clip", crate::CssBackgroundBoxList, CssBackgroundClipPropertyValue, boxes, parse_background_box_list, { parse_background_box_list($input)? }, expansion = longhand { value: CssBackgroundBoxList, accessor: boxes, inherited: false, initial_kind: value, initial: CssBackgroundBoxList::try_new(vec![CssBackgroundBox::BorderBox]).expect("one initial clip box") };
            BackgroundAttachment, "background-attachment", [], "baseline.property.background-attachment", crate::CssBackgroundAttachmentList, CssBackgroundAttachmentPropertyValue, attachments, parse_background_attachment_list, { parse_background_attachment_list($input)? }, expansion = longhand { value: CssBackgroundAttachmentList, accessor: attachments, inherited: false, initial_kind: value, initial: CssBackgroundAttachmentList::try_new(vec![CssBackgroundAttachment::Scroll]).expect("one initial attachment") };
            BorderImage, "border-image", [], "official.property.border-image", crate::CssBorderImage, CssBorderImagePropertyValue, border_image, parse_border_image, { parse_border_image($input, $numeric)? }, expansion = shorthand { accessor: border_image, members: [
                BorderImageSource => |value: &CssBorderImage| value.source().cloned(),
                BorderImageSlice => |value: &CssBorderImage| value.slice().cloned(),
                BorderImageWidth => |value: &CssBorderImage| value.width().cloned(),
                BorderImageOutset => |value: &CssBorderImage| value.outset().cloned(),
                BorderImageRepeat => |value: &CssBorderImage| value.repeat()
            ], reset_only: [] };
            BorderImageOutset, "border-image-outset", [], "official.property.border-image-outset", crate::CssBorderImageOutset, CssBorderImageOutsetPropertyValue, outsets, parse_border_image_outset, { parse_border_image_outset($input, $numeric)? }, expansion = longhand { value: CssBorderImageOutset, accessor: outsets, inherited: false, initial_kind: value, initial: CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Number(crate::CssSpecifiedNonNegativeNumber::try_from_component(crate::CssComponentValue::try_number("0").expect("number token")).expect("0 is non-negative"))]).expect("one outset component is valid") };
            BorderImageRepeat, "border-image-repeat", [], "official.property.border-image-repeat", crate::CssBorderImageRepeat, CssBorderImageRepeatPropertyValue, repeat, parse_border_image_repeat, { parse_border_image_repeat($input)? }, expansion = longhand { value: CssBorderImageRepeat, accessor: repeat, inherited: false, initial_kind: value, initial: CssBorderImageRepeat::new(CssBorderImageRepeatKeyword::Stretch, CssBorderImageRepeatKeyword::Stretch) };
            BorderImageSlice, "border-image-slice", [], "official.property.border-image-slice", crate::CssBorderImageSlice, CssBorderImageSlicePropertyValue, slice, parse_border_image_slice, { parse_border_image_slice($input, $numeric)? }, expansion = longhand { value: CssBorderImageSlice, accessor: slice, inherited: false, initial_kind: value, initial: CssBorderImageSlice::try_new(vec![CssBorderImageSliceComponent::Percentage(crate::CssSpecifiedNonNegativePercentage::try_from_component(crate::CssComponentValue::try_token("100%").expect("percentage token")).expect("100% is non-negative"))], false).expect("one slice component is valid") };
            BorderImageSource, "border-image-source", [], "official.property.border-image-source", crate::CssImageValue, CssBorderImageSourcePropertyValue, source, parse_border_image_source, { parse_border_image_source($input, $numeric)? }, expansion = longhand { value: CssImageValue, accessor: source, inherited: false, initial_kind: value, initial: CssImageValue::None };
            BorderImageWidth, "border-image-width", [], "official.property.border-image-width", crate::CssBorderImageWidth, CssBorderImageWidthPropertyValue, widths, parse_border_image_width, { parse_border_image_width($input, $numeric)? }, expansion = longhand { value: CssBorderImageWidth, accessor: widths, inherited: false, initial_kind: value, initial: CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Number(crate::CssSpecifiedNonNegativeNumber::try_from_component(crate::CssComponentValue::try_number("1").expect("number token")).expect("1 is non-negative"))]).expect("one width component is valid") };
            ImageOrientation, "image-orientation", [], "official.property.image-orientation", crate::CssImageOrientation, CssImageOrientationPropertyValue, orientation, parse_image_orientation, { parse_image_orientation($input, $numeric)? };
            ImageRendering, "image-rendering", [], "official.property.image-rendering", crate::CssImageRendering, CssImageRenderingPropertyValue, rendering, parse_image_rendering, { parse_image_rendering($input)? };
            ObjectFit, "object-fit", [], "official.property.object-fit", crate::CssObjectFit, CssObjectFitPropertyValue, fit, parse_object_fit, { parse_object_fit($input)? };
            BorderStyle, "border-style", [], "baseline.property.border-style", crate::CssBorderStyleShorthand, CssBorderStylePropertyValue, value, parse_border_style_shorthand, { parse_border_style_shorthand($input)? }, expansion = four_side { accessor: value, mode: |value: &CssBorderStyleShorthand| value.kind(), physical: [ BorderTopStyle => |value: &CssBorderStyleShorthand| value.assigned_values()[0].clone(), BorderRightStyle => |value: &CssBorderStyleShorthand| value.assigned_values()[1].clone(), BorderBottomStyle => |value: &CssBorderStyleShorthand| value.assigned_values()[2].clone(), BorderLeftStyle => |value: &CssBorderStyleShorthand| value.assigned_values()[3].clone() ], logical: [ BorderBlockStartStyle => |value: &CssBorderStyleShorthand| value.assigned_values()[0].clone(), BorderInlineStartStyle => |value: &CssBorderStyleShorthand| value.assigned_values()[1].clone(), BorderBlockEndStyle => |value: &CssBorderStyleShorthand| value.assigned_values()[2].clone(), BorderInlineEndStyle => |value: &CssBorderStyleShorthand| value.assigned_values()[3].clone() ] };
            BorderTopStyle, "border-top-style", [], "baseline.property.border-top-style", crate::CssBorderStyle, CssBorderTopStylePropertyValue, value, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssBorderStyle, accessor: value, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderRightStyle, "border-right-style", [], "baseline.property.border-right-style", crate::CssBorderStyle, CssBorderRightStylePropertyValue, value, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssBorderStyle, accessor: value, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderBottomStyle, "border-bottom-style", [], "baseline.property.border-bottom-style", crate::CssBorderStyle, CssBorderBottomStylePropertyValue, value, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssBorderStyle, accessor: value, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderLeftStyle, "border-left-style", [], "baseline.property.border-left-style", crate::CssBorderStyle, CssBorderLeftStylePropertyValue, value, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssBorderStyle, accessor: value, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderBlockStartStyle, "border-block-start-style", [], "official.property.border-block-start-style", crate::CssBorderStyle, CssBorderBlockStartStylePropertyValue, value, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssBorderStyle, accessor: value, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderBlockEndStyle, "border-block-end-style", [], "official.property.border-block-end-style", crate::CssBorderStyle, CssBorderBlockEndStylePropertyValue, value, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssBorderStyle, accessor: value, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderInlineStartStyle, "border-inline-start-style", [], "official.property.border-inline-start-style", crate::CssBorderStyle, CssBorderInlineStartStylePropertyValue, value, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssBorderStyle, accessor: value, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderInlineEndStyle, "border-inline-end-style", [], "official.property.border-inline-end-style", crate::CssBorderStyle, CssBorderInlineEndStylePropertyValue, value, parse_border_style, { parse_border_style($input)? }, expansion = longhand { value: CssBorderStyle, accessor: value, inherited: false, initial_kind: value, initial: CssBorderStyle::None };
            BorderBlockStyle, "border-block-style", [], "official.property.border-block-style", crate::CssBorderStylePair, CssBorderBlockStylePropertyValue, value, parse_border_style_pair, { parse_border_style_pair($input)? }, expansion = shorthand { accessor: value, members: [ BorderBlockStartStyle => |value: &CssBorderStylePair| Some(*value.start()), BorderBlockEndStyle => |value: &CssBorderStylePair| Some(*value.end()) ], reset_only: [] };
            BorderInlineStyle, "border-inline-style", [], "official.property.border-inline-style", crate::CssBorderStylePair, CssBorderInlineStylePropertyValue, value, parse_border_style_pair, { parse_border_style_pair($input)? }, expansion = shorthand { accessor: value, members: [ BorderInlineStartStyle => |value: &CssBorderStylePair| Some(*value.start()), BorderInlineEndStyle => |value: &CssBorderStylePair| Some(*value.end()) ], reset_only: [] };
            BorderRadius, "border-radius", [], "baseline.property.border-radius", crate::CssBorderRadiusShorthand, CssBorderRadiusPropertyValue, value, parse_exact_border_radius, { parse_exact_border_radius($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ BorderTopLeftRadius => |value: &CssBorderRadiusShorthand| Some(value.top_left()), BorderTopRightRadius => |value: &CssBorderRadiusShorthand| Some(value.top_right()), BorderBottomRightRadius => |value: &CssBorderRadiusShorthand| Some(value.bottom_right()), BorderBottomLeftRadius => |value: &CssBorderRadiusShorthand| Some(value.bottom_left()) ], reset_only: [] };
            BorderTopLeftRadius, "border-top-left-radius", [], "baseline.property.border-top-left-radius", crate::CssCornerRadiusValue, CssBorderTopLeftRadiusPropertyValue, value, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: CssCornerRadiusValue, accessor: value, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderTopRightRadius, "border-top-right-radius", [], "baseline.property.border-top-right-radius", crate::CssCornerRadiusValue, CssBorderTopRightRadiusPropertyValue, value, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: CssCornerRadiusValue, accessor: value, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderBottomRightRadius, "border-bottom-right-radius", [], "baseline.property.border-bottom-right-radius", crate::CssCornerRadiusValue, CssBorderBottomRightRadiusPropertyValue, value, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: CssCornerRadiusValue, accessor: value, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderBottomLeftRadius, "border-bottom-left-radius", [], "baseline.property.border-bottom-left-radius", crate::CssCornerRadiusValue, CssBorderBottomLeftRadiusPropertyValue, value, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: CssCornerRadiusValue, accessor: value, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderStartStartRadius, "border-start-start-radius", [], "official.property.border-start-start-radius", crate::CssCornerRadiusValue, CssBorderStartStartRadiusPropertyValue, value, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: CssCornerRadiusValue, accessor: value, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderStartEndRadius, "border-start-end-radius", [], "official.property.border-start-end-radius", crate::CssCornerRadiusValue, CssBorderStartEndRadiusPropertyValue, value, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: CssCornerRadiusValue, accessor: value, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderEndStartRadius, "border-end-start-radius", [], "official.property.border-end-start-radius", crate::CssCornerRadiusValue, CssBorderEndStartRadiusPropertyValue, value, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: CssCornerRadiusValue, accessor: value, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BorderEndEndRadius, "border-end-end-radius", [], "official.property.border-end-end-radius", crate::CssCornerRadiusValue, CssBorderEndEndRadiusPropertyValue, value, parse_exact_corner_radius, { parse_exact_corner_radius($input, $numeric)? }, expansion = longhand { value: CssCornerRadiusValue, accessor: value, inherited: false, initial_kind: value, initial: CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None) };
            BoxShadow, "box-shadow", [], "baseline.property.box-shadow", crate::CssBoxShadow, CssBoxShadowPropertyValue, value, parse_box_shadow, { parse_box_shadow($input, $numeric)? }, expansion = longhand { value: CssBoxShadow, accessor: value, inherited: false, initial_kind: value, initial: CssBoxShadow::None };
            Opacity, "opacity", [], "baseline.property.opacity", crate::CssOpacityValue, CssOpacityPropertyValue, value, parse_opacity, { parse_opacity($input, $numeric)? }, expansion = longhand { value: CssOpacityValue, accessor: value, inherited: false, initial_kind: value, initial: CssOpacityValue::Scalar(crate::CssOpacityScalar::try_from_component(crate::CssComponentValue::try_number("1").expect("valid initial number")).expect("valid opacity scalar")) };
            FlexGrow, "flex-grow", ["-webkit-flex-grow"], "baseline.property.flex-grow", crate::CssSpecifiedNonNegativeNumber, CssFlexGrowPropertyValue, factor, parse_flex_factor, { parse_flex_factor($input, $numeric, "flex-grow")? }, expansion = longhand { value: CssSpecifiedNonNegativeNumber, accessor: factor, inherited: false, initial_kind: value, initial: crate::flex::initial_grow() };
            FlexShrink, "flex-shrink", ["-webkit-flex-shrink"], "baseline.property.flex-shrink", crate::CssSpecifiedNonNegativeNumber, CssFlexShrinkPropertyValue, factor, parse_flex_factor, { parse_flex_factor($input, $numeric, "flex-shrink")? }, expansion = longhand { value: CssSpecifiedNonNegativeNumber, accessor: factor, inherited: false, initial_kind: value, initial: crate::flex::initial_shrink() };
            Order, "order", ["-webkit-order"], "baseline.property.order", crate::CssIntegerValue, CssOrderPropertyValue, value, parse_order, { parse_order($input, $numeric)? }, expansion = longhand { value: CssIntegerValue, accessor: value, inherited: false, initial_kind: value, initial: CssIntegerValue::Literal(crate::CssIntegerLiteral::from_i32(0)) };
            Flex, "flex", ["-webkit-flex"], "baseline.property.flex", crate::CssFlexValue, CssFlexPropertyValue, value, parse_flex, { parse_flex($input, $numeric)? }, expansion = shorthand { accessor: value, members: [ FlexGrow => crate::flex::effective_grow, FlexShrink => crate::flex::effective_shrink, FlexBasis => crate::flex::effective_basis ], reset_only: [] };
            AspectRatio, "aspect-ratio", [], "baseline.property.aspect-ratio", crate::CssAspectRatioValue, CssAspectRatioPropertyValue, ratio, parse_aspect_ratio, { parse_aspect_ratio($input, $numeric)? }, expansion = longhand { value: CssAspectRatioValue, accessor: ratio, inherited: false, initial_kind: value, initial: CssAspectRatioValue::Auto };
            ScrollbarWidth, "scrollbar-width", [], "baseline.property.scrollbar-width", crate::CssScrollbarWidth, CssScrollbarWidthPropertyValue, value, parse_scrollbar_width, { parse_scrollbar_width($input)? }, expansion = longhand { value: CssScrollbarWidth, accessor: value, inherited: false, initial_kind: value, initial: CssScrollbarWidth::Auto };
            ScrollbarColor, "scrollbar-color", [], "official.property.scrollbar-color", crate::CssScrollbarColor, CssScrollbarColorPropertyValue, value, parse_scrollbar_color, { parse_scrollbar_color($input, $numeric)? }, expansion = longhand { value: CssScrollbarColor, accessor: value, inherited: true, initial_kind: value, initial: CssScrollbarColor::auto() };
            ColorScheme, "color-scheme", [], "official.property.color-scheme", crate::CssColorScheme, CssColorSchemePropertyValue, value, parse_color_scheme, { parse_color_scheme($input)? }, expansion = longhand { value: CssColorScheme, accessor: value, inherited: true, initial_kind: value, initial: CssColorScheme::normal() };
            ForcedColorAdjust, "forced-color-adjust", [], "official.property.forced-color-adjust", crate::CssForcedColorAdjust, CssForcedColorAdjustPropertyValue, value, parse_forced_color_adjust, { parse_forced_color_adjust($input)? }, expansion = longhand { value: CssForcedColorAdjust, accessor: value, inherited: true, initial_kind: value, initial: CssForcedColorAdjust::Auto };
            PrintColorAdjust, "print-color-adjust", [], "official.property.print-color-adjust", crate::CssPrintColorAdjust, CssPrintColorAdjustPropertyValue, value, parse_print_color_adjust, { parse_print_color_adjust($input)? }, expansion = longhand { value: CssPrintColorAdjust, accessor: value, inherited: true, initial_kind: value, initial: CssPrintColorAdjust::Economy };
            ColorAdjust, "color-adjust", [], "official.property.color-adjust", crate::CssPrintColorAdjust, CssColorAdjustPropertyValue, value, parse_print_color_adjust, { parse_print_color_adjust($input)? }, expansion = shorthand { accessor: value, members: [ PrintColorAdjust => |value: &CssPrintColorAdjust| Some(*value) ], reset_only: [] };
            Cursor, "cursor", [], "baseline.property.cursor", crate::CssCursor, CssCursorPropertyValue, value, parse_cursor, { parse_cursor($input, $numeric)? };
            CaretColor, "caret-color", [], "official.property.caret-color", crate::CssCaretColor, CssCaretColorPropertyValue, caret, parse_caret_color, { parse_caret_color($input, $numeric)? };
            PointerEvents, "pointer-events", [], "baseline.property.pointer-events", crate::CssPointerEvents, CssPointerEventsPropertyValue, value, parse_pointer_events, { parse_pointer_events($input)? };
            UserSelect, "user-select", [], "baseline.property.user-select", crate::CssUserSelect, CssUserSelectPropertyValue, value, parse_user_select, { parse_user_select($input)? };
            Resize, "resize", [], "official.property.resize", crate::CssResize, CssResizePropertyValue, resize, parse_resize, { parse_resize($input)? };
            Contain, "contain", [], "official.property.contain", crate::CssContain, CssContainPropertyValue, containment, parse_contain, { parse_contain($input)? };
            Outline, "outline", [], "baseline.property.outline", crate::CssOutline, CssOutlinePropertyValue, value, parse_outline, { parse_outline($input, $numeric)? };
            OutlineColor, "outline-color", [], "baseline.property.outline-color", crate::CssOutlineColor, CssOutlineColorPropertyValue, value, parse_outline_color, { parse_outline_color($input, $numeric)? };
            OutlineOffset, "outline-offset", [], "official.property.outline-offset", crate::CssSpecifiedLength, CssOutlineOffsetPropertyValue, offset, parse_outline_offset, { parse_outline_offset($input, $numeric)? };
            OutlineStyle, "outline-style", [], "baseline.property.outline-style", crate::CssOutlineStyle, CssOutlineStylePropertyValue, value, parse_outline_style, { parse_outline_style($input)? };
            OutlineWidth, "outline-width", [], "baseline.property.outline-width", crate::CssOutlineWidth, CssOutlineWidthPropertyValue, value, parse_outline_width, { parse_outline_width($input, $numeric)? };
            Transform, "transform", [], "baseline.property.transform", crate::CssTransform, CssTransformPropertyValue, value, parse_transform, { parse_transform($input, $numeric)? };
            TransformBox, "transform-box", [], "official.property.transform-box", crate::CssTransformBox, CssTransformBoxPropertyValue, reference_box, parse_transform_box, { parse_transform_box($input)? };
            TransformOrigin, "transform-origin", [], "baseline.property.transform-origin", crate::CssTransformOrigin, CssTransformOriginPropertyValue, origin, parse_transform_origin, { parse_transform_origin($input, $numeric)? };
            Translate, "translate", [], "baseline.property.translate", crate::CssTranslate, CssTranslatePropertyValue, value, parse_translate, { parse_translate($input, $numeric)? };
            Rotate, "rotate", [], "baseline.property.rotate", crate::CssRotate, CssRotatePropertyValue, value, parse_rotate, { parse_rotate($input, $numeric)? };
            Scale, "scale", [], "baseline.property.scale", crate::CssScale, CssScalePropertyValue, value, parse_scale, { parse_scale($input, $numeric)? };
            Filter, "filter", [], "baseline.property.filter", crate::CssFilter, CssFilterPropertyValue, value, parse_filter, { parse_filter($input, $numeric)? }, expansion = longhand { value: CssFilter, accessor: value, inherited: false, initial_kind: value, initial: CssFilter::None };
            BackdropFilter, "backdrop-filter", [], "baseline.property.backdrop-filter", crate::CssFilter, CssBackdropFilterPropertyValue, value, parse_filter, { parse_filter($input, $numeric)? }, expansion = longhand { value: CssFilter, accessor: value, inherited: false, initial_kind: value, initial: CssFilter::None };
            ClipPath, "clip-path", [], "baseline.property.clip-path", crate::CssClipPath, CssClipPathPropertyValue, value, parse_clip_path, { parse_clip_path($input, $numeric)? }, expansion = longhand { value: crate::CssClipPath, accessor: value, inherited: false, initial_kind: value, initial: crate::CssClipPath::None };
            BackgroundBlendMode, "background-blend-mode", [], "official.property.background-blend-mode", crate::CssBlendModeList, CssBackgroundBlendModePropertyValue, modes, parse_blend_mode_list, { parse_blend_mode_list($input)? }, expansion = longhand { value: CssBlendModeList, accessor: modes, inherited: false, initial_kind: value, initial: CssBlendModeList::try_new(vec![CssBlendMode::Normal]).expect("one initial blend mode") };
            Isolation, "isolation", [], "official.property.isolation", crate::CssIsolation, CssIsolationPropertyValue, isolation, parse_isolation, { parse_isolation($input)? }, expansion = longhand { value: CssIsolation, accessor: isolation, inherited: false, initial_kind: value, initial: CssIsolation::Auto };
            MixBlendMode, "mix-blend-mode", [], "official.property.mix-blend-mode", crate::CssBlendMode, CssMixBlendModePropertyValue, mode, parse_blend_mode, { parse_blend_mode($input)? }, expansion = longhand { value: CssBlendMode, accessor: mode, inherited: false, initial_kind: value, initial: CssBlendMode::Normal };
            Mask, "mask", [], "baseline.property.mask", crate::CssMaskList, CssMaskPropertyValue, value, parse_mask_list, { parse_mask_list($input, $numeric)? };
            MaskImage, "mask-image", [], "baseline.property.mask-image", crate::CssImageValueList, CssMaskImagePropertyValue, images, parse_image_layer_list, { parse_image_layer_list($input, $numeric)? }, expansion = longhand { value: CssImageValueList, accessor: images, inherited: false, initial_kind: value, initial: CssImageValueList::try_new(vec![CssImageValue::None]).expect("one initial image") };
            MaskSize, "mask-size", [], "baseline.property.mask-size", crate::CssBackgroundSizeList, CssMaskSizePropertyValue, sizes, parse_background_size_list, { parse_background_size_list($input, $numeric)? };
            MaskPosition, "mask-position", [], "baseline.property.mask-position", crate::CssPhysicalPositionList, CssMaskPositionPropertyValue, positions, parse_mask_position_list, { parse_mask_position_list($input, $numeric)? };
            MaskRepeat, "mask-repeat", [], "baseline.property.mask-repeat", crate::CssBackgroundRepeatList, CssMaskRepeatPropertyValue, repeats, parse_background_repeat_list, { parse_background_repeat_list($input)? };
            TransitionProperty, "transition-property", [], "baseline.property.transition-property", crate::CssTransitionPropertyList, CssTransitionPropertyPropertyValue, properties, parse_transition_property_list, { parse_transition_property_list($input)? }, expansion = longhand { value: CssTransitionPropertyList, accessor: properties, inherited: false, initial_kind: value, initial: CssTransitionPropertyList::try_new(vec![CssTransitionProperty::All]).expect("one initial transition property") };
            TransitionDuration, "transition-duration", [], "baseline.property.transition-duration", crate::CssDurationList, CssTransitionDurationPropertyValue, durations, parse_duration_list, { parse_duration_list($input, $numeric)? }, expansion = longhand { value: CssDurationList, accessor: durations, inherited: false, initial_kind: value, initial: CssDurationList::try_new(vec![crate::CssDuration::try_new(crate::expansion::initial_zero_time()).expect("nonnegative initial duration")]).expect("one closed initial duration") };
            TransitionDelay, "transition-delay", [], "baseline.property.transition-delay", crate::CssDelayList, CssTransitionDelayPropertyValue, delays, parse_delay_list, { parse_delay_list($input, $numeric)? }, expansion = longhand { value: CssDelayList, accessor: delays, inherited: false, initial_kind: value, initial: CssDelayList::try_new(vec![crate::expansion::initial_zero_time()]).expect("one closed initial delay") };
            TransitionTimingFunction, "transition-timing-function", [], "baseline.property.transition-timing-function", crate::CssEasingList, CssTransitionTimingFunctionPropertyValue, timing_functions, parse_easing_list, { parse_easing_list($input, $numeric)? }, expansion = longhand { value: CssEasingList, accessor: timing_functions, inherited: false, initial_kind: value, initial: CssEasingList::try_new(vec![CssEasing::Keyword(CssEasingKeyword::Ease)]).expect("one initial easing") };
            Transition, "transition", [], "baseline.property.transition", crate::CssTransitionList, CssTransitionPropertyValue, transitions, parse_transition_value_list, { parse_transition_value_list($input, $numeric)? }, expansion = shorthand { accessor: transitions, members: [
                TransitionProperty => crate::expansion::transition_properties,
                TransitionDuration => crate::expansion::transition_durations,
                TransitionTimingFunction => crate::expansion::transition_timing_functions,
                TransitionDelay => crate::expansion::transition_delays
            ], reset_only: [] };
            AnimationName, "animation-name", [], "baseline.property.animation-name", crate::CssAnimationNameList, CssAnimationNamePropertyValue, names, parse_animation_name_list, { parse_animation_name_list($input)? }, expansion = longhand { value: CssAnimationNameList, accessor: names, inherited: false, initial_kind: value, initial: CssAnimationNameList::try_new(vec![CssAnimationName::None]).expect("one initial animation name") };
            AnimationDuration, "animation-duration", [], "baseline.property.animation-duration", crate::CssDurationList, CssAnimationDurationPropertyValue, durations, parse_duration_list, { parse_duration_list($input, $numeric)? }, expansion = longhand { value: CssDurationList, accessor: durations, inherited: false, initial_kind: value, initial: CssDurationList::try_new(vec![crate::CssDuration::try_new(crate::expansion::initial_zero_time()).expect("nonnegative initial duration")]).expect("one closed initial duration") };
            AnimationDelay, "animation-delay", [], "baseline.property.animation-delay", crate::CssDelayList, CssAnimationDelayPropertyValue, delays, parse_delay_list, { parse_delay_list($input, $numeric)? }, expansion = longhand { value: CssDelayList, accessor: delays, inherited: false, initial_kind: value, initial: CssDelayList::try_new(vec![crate::expansion::initial_zero_time()]).expect("one closed initial delay") };
            AnimationTimingFunction, "animation-timing-function", [], "baseline.property.animation-timing-function", crate::CssEasingList, CssAnimationTimingFunctionPropertyValue, timing_functions, parse_easing_list, { parse_easing_list($input, $numeric)? }, expansion = longhand { value: CssEasingList, accessor: timing_functions, inherited: false, initial_kind: value, initial: CssEasingList::try_new(vec![CssEasing::Keyword(CssEasingKeyword::Ease)]).expect("one initial easing") };
            AnimationIterationCount, "animation-iteration-count", [], "baseline.property.animation-iteration-count", crate::CssAnimationIterationCountList, CssAnimationIterationCountPropertyValue, iteration_counts, parse_animation_iteration_value_list, { parse_animation_iteration_value_list($input, $numeric)? }, expansion = longhand { value: CssAnimationIterationCountList, accessor: iteration_counts, inherited: false, initial_kind: value, initial: CssAnimationIterationCountList::try_new(vec![CssAnimationIterationCount::Number(crate::CssSpecifiedNonNegativeNumber::try_from_component(crate::CssComponentValue::try_number("1").expect("ordinary one")).expect("nonnegative initial count"))]).expect("one initial iteration count") };
            AnimationDirection, "animation-direction", [], "baseline.property.animation-direction", crate::CssAnimationDirectionList, CssAnimationDirectionPropertyValue, directions, parse_animation_direction_list, { parse_animation_direction_list($input)? }, expansion = longhand { value: CssAnimationDirectionList, accessor: directions, inherited: false, initial_kind: value, initial: CssAnimationDirectionList::try_new(vec![CssAnimationDirection::Normal]).expect("one initial direction") };
            AnimationFillMode, "animation-fill-mode", [], "baseline.property.animation-fill-mode", crate::CssAnimationFillModeList, CssAnimationFillModePropertyValue, fill_modes, parse_animation_fill_mode_list, { parse_animation_fill_mode_list($input)? }, expansion = longhand { value: CssAnimationFillModeList, accessor: fill_modes, inherited: false, initial_kind: value, initial: CssAnimationFillModeList::try_new(vec![CssAnimationFillMode::None]).expect("one initial fill mode") };
            AnimationPlayState, "animation-play-state", [], "baseline.property.animation-play-state", crate::CssAnimationPlayStateList, CssAnimationPlayStatePropertyValue, play_states, parse_animation_play_state_list, { parse_animation_play_state_list($input)? }, expansion = longhand { value: CssAnimationPlayStateList, accessor: play_states, inherited: false, initial_kind: value, initial: CssAnimationPlayStateList::try_new(vec![CssAnimationPlayState::Running]).expect("one initial play state") };
            Animation, "animation", [], "baseline.property.animation", crate::CssAnimationList, CssAnimationPropertyValue, animations, parse_animation_value_list, { parse_animation_value_list($input, $numeric)? }, expansion = shorthand { accessor: animations, members: [
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

macro_rules! define_semantic_property_value {
    ($canonical:literal, $wrapper:ident, $value:ty, $accessor:ident) => {
        #[doc = concat!("A grammar-checked authored ordinary value for `", $canonical, "`.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper {
            authored: CssAuthoredDeclarationValue,
            value: $value,
        }
        impl $wrapper {
            #[must_use]
            pub(crate) const fn new(authored: CssAuthoredDeclarationValue, value: $value) -> Self {
                Self { authored, value }
            }
            /// Returns the exact authored ordinary value slice.
            #[must_use]
            pub fn as_css(&self) -> &str {
                self.authored.as_css()
            }
            /// Returns the grammar-checked semantic value.
            #[must_use]
            pub const fn $accessor(&self) -> &$value {
                &self.value
            }
        }
    };
}

macro_rules! define_property_identity {
    ($input:ident, $numeric:ident;
        All, $all_canonical:literal, [$($all_alias:literal),*], $all_stable_id:literal,
        $all_value:ty,
        $all_parser:ident, $all_dispatch:block $(, expansion = $all_expansion:ident { $($all_metadata:tt)* })?;
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $wrapper:ident, $accessor:ident, $parser:ident, $dispatch:block
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

        $(define_semantic_property_value!($canonical, $wrapper, $value, $accessor);)*

        /// A borrowed property-specific ordinary-value view.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum CssKnownPropertyValueRef<'a> {
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
        ///     value: todo!(),
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
                wrapper: None,
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
                    wrapper: Some(stringify!($wrapper)),
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
    pub(crate) wrapper: Option<&'static str>,
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
