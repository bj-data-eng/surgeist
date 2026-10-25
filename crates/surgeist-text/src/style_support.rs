/// Text-owned style-facing feature names for root/style integration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextStyleFeature {
    FontFamilyList,
    NamedFontWeight,
    NumericFontWeight,
    BasicFontStretch,
    ExpandedFontStretch,
    FontStyle,
    ObliqueSlant,
    FontVariant,
    FontFeatureSettings,
    FontVariationSettings,
    FontSize,
    LineHeight,
    LetterSpacing,
    WordSpacing,
    ConcreteTextColor,
    SymbolicTextColor,
    Locale,
    ExplicitTextDirection,
    WhiteSpacePreserve,
    WhiteSpaceCollapse,
    /// Discard executes in the processed-paragraph phase.
    WhiteSpaceDiscard,
    /// PreserveSpaces executes in the processed-paragraph phase.
    WhiteSpacePreserveSpaces,
    /// PreserveBreaks executes in the processed-paragraph phase.
    WhiteSpacePreserveBreaks,
    /// BreakSpaces executes in the processed-paragraph phase.
    WhiteSpaceBreakSpaces,
    WordBreak,
    LineBreak,
    TextWrap,
    OverflowWrap,
    TextOverflow,
    TextAlignment,
    TextAlignLast,
    TextIndent,
    InlineBoxVerticalAlign,
    VerticalAlign,
    Underline,
    Strikethrough,
    Overline,
    DecorationOffset,
    DecorationThickness,
    DecorationStyle,
    ConcreteDecorationColor,
    SymbolicDecorationColor,
    ExplicitWordSeparators,
    NativeWbr,
    /// Japanese automatic phrase expansion with demanded authentic scope capture.
    NativeAutomaticPhrases,
    MathAuto,
    FullSizeKana,
    FullWidth,
    Uppercase,
    Lowercase,
    Capitalize,
    SelectionColor,
}

/// The actual executor whose support is being queried.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextStylePhase {
    /// Existing raw-source convenience layout.
    Standalone,
    /// Explicit paragraph capture with executed source processing.
    ProcessedParagraph,
}

impl TextStyleFeature {
    pub const ALL: &'static [Self] = &[
        Self::FontFamilyList,
        Self::NamedFontWeight,
        Self::NumericFontWeight,
        Self::BasicFontStretch,
        Self::ExpandedFontStretch,
        Self::FontStyle,
        Self::ObliqueSlant,
        Self::FontVariant,
        Self::FontFeatureSettings,
        Self::FontVariationSettings,
        Self::FontSize,
        Self::LineHeight,
        Self::LetterSpacing,
        Self::WordSpacing,
        Self::ConcreteTextColor,
        Self::SymbolicTextColor,
        Self::Locale,
        Self::ExplicitTextDirection,
        Self::WhiteSpacePreserve,
        Self::WhiteSpaceCollapse,
        Self::WhiteSpaceDiscard,
        Self::WhiteSpacePreserveSpaces,
        Self::WhiteSpacePreserveBreaks,
        Self::WhiteSpaceBreakSpaces,
        Self::WordBreak,
        Self::LineBreak,
        Self::TextWrap,
        Self::OverflowWrap,
        Self::TextOverflow,
        Self::TextAlignment,
        Self::TextAlignLast,
        Self::TextIndent,
        Self::InlineBoxVerticalAlign,
        Self::VerticalAlign,
        Self::Underline,
        Self::Strikethrough,
        Self::Overline,
        Self::DecorationOffset,
        Self::DecorationThickness,
        Self::DecorationStyle,
        Self::ConcreteDecorationColor,
        Self::SymbolicDecorationColor,
        Self::ExplicitWordSeparators,
        Self::NativeWbr,
        Self::NativeAutomaticPhrases,
        Self::MathAuto,
        Self::FullSizeKana,
        Self::FullWidth,
        Self::Uppercase,
        Self::Lowercase,
        Self::Capitalize,
        Self::SelectionColor,
    ];

    #[must_use]
    pub const fn support(self, phase: TextStylePhase) -> TextStyleSupport {
        if matches!(phase, TextStylePhase::ProcessedParagraph)
            && matches!(
                self,
                Self::MathAuto
                    | Self::NativeWbr
                    | Self::NativeAutomaticPhrases
                    | Self::WhiteSpaceCollapse
                    | Self::WhiteSpaceDiscard
                    | Self::WhiteSpacePreserveSpaces
                    | Self::WhiteSpacePreserveBreaks
                    | Self::WhiteSpaceBreakSpaces
            )
        {
            return TextStyleSupport::Supported;
        }
        match self {
            Self::NativeWbr | Self::NativeAutomaticPhrases => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresProcessedSourceLayout,
            ),
            Self::MathAuto => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresCompleteTextNodeContext,
            ),
            Self::FontFamilyList
            | Self::NamedFontWeight
            | Self::NumericFontWeight
            | Self::BasicFontStretch
            | Self::ExpandedFontStretch
            | Self::FontStyle
            | Self::ObliqueSlant
            | Self::FontFeatureSettings
            | Self::FontVariationSettings
            | Self::FontSize
            | Self::LineHeight
            | Self::LetterSpacing
            | Self::WordSpacing
            | Self::ConcreteTextColor
            | Self::Locale
            | Self::WhiteSpacePreserve
            | Self::WordBreak
            | Self::LineBreak
            | Self::TextWrap
            | Self::OverflowWrap
            | Self::TextAlignment
            | Self::TextIndent
            | Self::InlineBoxVerticalAlign
            | Self::Underline
            | Self::Strikethrough
            | Self::DecorationOffset
            | Self::DecorationThickness
            | Self::ConcreteDecorationColor
            | Self::ExplicitWordSeparators
            | Self::FullSizeKana
            | Self::FullWidth
            | Self::Uppercase
            | Self::Lowercase
            | Self::Capitalize
            | Self::SelectionColor => TextStyleSupport::Supported,
            Self::ExplicitTextDirection => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresParagraphBaseDirection,
            ),
            Self::WhiteSpaceCollapse => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresProcessedSourceLayout,
            ),
            Self::WhiteSpaceDiscard
            | Self::WhiteSpacePreserveSpaces
            | Self::WhiteSpacePreserveBreaks
            | Self::WhiteSpaceBreakSpaces => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresProcessedSourceLayout,
            ),
            Self::FontVariant => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresFontPolicy)
            }
            Self::TextAlignLast | Self::TextOverflow => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresTextFlowPolicy)
            }
            Self::Overline | Self::DecorationStyle => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresDecorationSelectionPolicy,
            ),
            Self::SymbolicTextColor | Self::SymbolicDecorationColor => {
                TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresColorResolution)
            }
            Self::VerticalAlign => TextStyleSupport::Unsupported(
                UnsupportedTextStyleReason::RequiresBroadVerticalAlignPolicy,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextStyleSupport {
    Supported,
    Unsupported(UnsupportedTextStyleReason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnsupportedTextStyleReason {
    RequiresCompleteTextNodeContext,
    RequiresParagraphBaseDirection,
    /// This standalone input requires the explicit processed-paragraph executor.
    RequiresProcessedSourceLayout,
    RequiresFontPolicy,
    RequiresTextFlowPolicy,
    RequiresDecorationSelectionPolicy,
    RequiresColorResolution,
    RequiresInlineMetricContract,
    RequiresBroadVerticalAlignPolicy,
    RequiresStandaloneIndentPolicy,
}
