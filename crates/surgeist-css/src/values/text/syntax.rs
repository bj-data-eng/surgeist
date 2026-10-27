use crate::numeric::{
    CssCalculation, CssInteger, CssLength, CssLengthPercentage, CssNonNegative, CssNumber,
    CssNumeric,
};

/// # Specified serialization
///
/// Serializes the canonical specified direction keyword without applying inheritance or contextual
/// text and layout behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssDirection {
    /// `ltr`: the inline base direction (bidi directionality) is line-left to line-right.
    Ltr,
    /// `rtl`: the inline base direction (bidi directionality) is line-right to line-left.
    Rtl,
}

/// # Specified serialization
///
/// Serializes the canonical specified writing-mode keyword without applying inheritance or
/// contextual text and layout behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWritingMode {
    /// `horizontal-tb`: top-to-bottom block flow direction; both the writing mode and the typographic mode are horizontal.
    HorizontalTb,
    /// `vertical-rl`: right-to-left block flow direction; both the writing mode and the typographic mode are vertical.
    VerticalRl,
    /// `vertical-lr`: left-to-right block flow direction; both the writing mode and the typographic mode are vertical.
    VerticalLr,
    /// `sideways-rl`: right-to-left block flow direction; the writing mode is vertical while the typographic mode is horizontal.
    SidewaysRl,
    /// `sideways-lr`: left-to-right block flow direction; the writing mode is vertical while the typographic mode is horizontal.
    SidewaysLr,
}

/// The authored `text-combine-upright` value, preserving an omitted digit count.
///
/// # Specified serialization
///
/// Serializes the specified value, preserving omitted counts and deferring computed integer
/// rounding and range clamping. Finite calculated counts use the shared six-place specified number
/// text policy.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTextCombineUpright {
    /// `none`: no special processing.
    None,
    /// `all`: attempts to typeset horizontally all consecutive typographic character units in the box so that they take up the space of a single typographic character unit.
    All,
    /// An omitted count computes to two later; specified values retain omission.
    Digits(Option<CssTextCombineDigitCount>),
}

#[derive(Clone, Debug, PartialEq)]
enum CssTextCombineDigitCountValue {
    Literal(i32),
    Calculation(CssCalculation<CssInteger>),
}

/// A literal digit count from two through four, or an authored integer calculation.
/// Calculations retain their specified value without computed rounding or clamping.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextCombineDigitCount {
    value: CssTextCombineDigitCountValue,
}

impl CssTextCombineDigitCount {
    /// Checks a literal digit count.
    ///
    /// Returns `None` unless `value` is from `2` through `4`.
    #[must_use]
    pub const fn try_literal(value: i32) -> Option<Self> {
        if value >= 2 && value <= 4 {
            Some(Self {
                value: CssTextCombineDigitCountValue::Literal(value),
            })
        } else {
            None
        }
    }

    /// Wraps an authored integer calculation as a digit count, without range checking.
    #[must_use]
    pub const fn from_calculation(value: CssCalculation<CssInteger>) -> Self {
        Self {
            value: CssTextCombineDigitCountValue::Calculation(value),
        }
    }

    /// Returns the literal count, or `None` when the count is a calculation.
    #[must_use]
    pub const fn literal(&self) -> Option<i32> {
        match self.value {
            CssTextCombineDigitCountValue::Literal(value) => Some(value),
            CssTextCombineDigitCountValue::Calculation(_) => None,
        }
    }

    /// Returns the calculation, or `None` when the count is a literal.
    #[must_use]
    pub const fn calculation(&self) -> Option<&CssCalculation<CssInteger>> {
        match &self.value {
            CssTextCombineDigitCountValue::Literal(_) => None,
            CssTextCombineDigitCountValue::Calculation(value) => Some(value),
        }
    }
}

/// The authored `text-orientation` keyword.
///
/// # Specified serialization
///
/// Serializes the canonical specified text-orientation keyword without applying inheritance or
/// contextual text and layout behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextOrientation {
    /// `mixed`: in vertical writing modes, characters from horizontal-only scripts are typeset sideways and characters from vertical scripts keep their intrinsic orientation.
    Mixed,
    /// `upright`: in vertical writing modes, characters from horizontal-only scripts are typeset upright and characters from vertical scripts keep their intrinsic orientation; the used `direction` is `ltr`.
    Upright,
    /// `sideways`: in vertical writing modes, all text is typeset sideways, as in a horizontal layout rotated 90 degrees clockwise.
    Sideways,
}

/// The authored `unicode-bidi` keyword.
///
/// # Specified serialization
///
/// Serializes the canonical specified unicode-bidi keyword without applying inheritance or
/// contextual text and layout behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssUnicodeBidi {
    /// `normal`: the element opens no additional level of embedding.
    Normal,
    /// `embed`: on an inline box, opens an additional level of embedding whose direction is given by `direction`; no effect on other boxes.
    Embed,
    /// `isolate`: on an inline box, isolates its contents from the surrounding text for bidirectional processing; no effect on other boxes.
    Isolate,
    /// `bidi-override`: puts the box's immediate inline content in a directional override, reordered strictly in sequence according to `direction`.
    BidiOverride,
    /// `isolate-override`: combines the isolation of `isolate` with the directional override of `bidi-override`.
    IsolateOverride,
    /// `plaintext`: behaves as `isolate`, except that the base directionality of each bidi paragraph or isolated sequence is determined from its content (rules P2 and P3 of the Unicode bidirectional algorithm) rather than from `direction`.
    Plaintext,
}

/// A `text-align` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextAlign {
    /// `start`: inline-level content is aligned to the start edge of the line box.
    Start,
    /// `end`: inline-level content is aligned to the end edge of the line box.
    End,
    /// `left`: inline-level content is aligned to the line-left edge of the line box.
    Left,
    /// `right`: inline-level content is aligned to the line-right edge of the line box.
    Right,
    /// `center`: inline-level content is centered within the line box.
    Center,
    /// `justify`: text is justified, by the method of `text-justify`, to exactly fill the line box.
    Justify,
    /// `match-parent`: as `inherit`, except that an inherited `start` or `end` is interpreted against the parent's `direction` and computes to `left` or `right`.
    MatchParent,
}

/// # Specified serialization
///
/// Emits canonical specified syntax from retained semantic fields. Optional values remain omitted;
/// explicit initials remain explicit. This does not resolve percentages or floor actual decoration
/// thickness.
#[derive(Clone, Debug)]
pub struct CssTextIndent {
    length: CssNumeric<CssLengthPercentage>,
    hanging: bool,
    each_line: bool,
}

impl PartialEq for CssTextIndent {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length
            && self.hanging == other.hanging
            && self.each_line == other.each_line
    }
}

impl CssTextIndent {
    /// Retains the checked indent and independently authored keyword flags.
    pub const fn new(
        length: CssNumeric<CssLengthPercentage>,
        hanging: bool,
        each_line: bool,
    ) -> Self {
        Self {
            length,
            hanging,
            each_line,
        }
    }
    /// Returns the indent as an authored `<length-percentage>`.
    pub const fn length(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.length
    }
    /// Returns whether the `hanging` keyword was written, inverting which lines are indented.
    pub const fn hanging(&self) -> bool {
        self.hanging
    }
    /// Returns whether the `each-line` keyword was written, also indenting lines after forced breaks.
    pub const fn each_line(&self) -> bool {
        self.each_line
    }
}

/// # Specified serialization
///
/// Emits canonical specified syntax from retained semantic fields. Optional values remain omitted;
/// explicit initials remain explicit. This does not resolve percentages or floor actual decoration
/// thickness.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssVerticalAlign {
    /// `baseline`: aligns the box baseline with the parent's baseline.
    Baseline,
    /// `sub`: lowers the baseline to the parent's subscript position.
    Sub,
    /// `super`: raises the baseline to the parent's superscript position.
    Super,
    /// `text-top`: aligns the top of the box with the top of the parent's content area.
    TextTop,
    /// `text-bottom`: aligns the bottom of the box with the bottom of the parent's content area.
    TextBottom,
    /// `middle`: aligns the vertical midpoint of the box with the parent's baseline plus half its x-height.
    Middle,
    /// `top`: aligns the top of the aligned subtree with the top of the line box.
    Top,
    /// `bottom`: aligns the bottom of the aligned subtree with the bottom of the line box.
    Bottom,
    /// A `<length-percentage>` that raises (positive) or lowers (negative) the box; a percentage is relative to the element's `line-height`.
    Length(CssNumeric<CssLengthPercentage>),
}

impl PartialEq for CssVerticalAlign {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Length(left), Self::Length(right)) => left == right,
            (Self::Baseline, Self::Baseline)
            | (Self::Sub, Self::Sub)
            | (Self::Super, Self::Super)
            | (Self::TextTop, Self::TextTop)
            | (Self::TextBottom, Self::TextBottom)
            | (Self::Middle, Self::Middle)
            | (Self::Top, Self::Top)
            | (Self::Bottom, Self::Bottom) => true,
            _ => false,
        }
    }
}

/// The authored unforced-wrapping mode, independent of the line selection style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextWrapMode {
    /// `wrap`: lines may wrap at allowed soft wrap opportunities.
    Wrap,
    /// `nowrap`: lines break only at forced line breaks.
    NoWrap,
}

/// The authored line selection style; no wrapping algorithm is executed here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextWrapStyle {
    /// `auto`: the algorithm for selecting the soft wrap opportunity to break at is user-agent defined.
    Auto,
    /// `balance`: line breaks are chosen to balance the remaining (empty) space in each line box, if better balance than `auto` is possible.
    Balance,
    /// `stable`: content on subsequent lines should not be considered when making break decisions; otherwise as `auto`.
    Stable,
    /// `pretty`: the user agent should bias for better layout over speed; otherwise as `auto`.
    Pretty,
    /// `avoid-short-last-line`: the user agent should avoid an excessively short last line.
    AvoidShortLastLine,
}

/// Nonempty authored text-wrap constituents, retaining omitted whole roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTextWrap {
    mode: Option<CssTextWrapMode>,
    style: Option<CssTextWrapStyle>,
}
impl CssTextWrap {
    /// Admits at least one constituent without inserting projection defaults.
    #[must_use]
    pub const fn try_new(
        mode: Option<CssTextWrapMode>,
        style: Option<CssTextWrapStyle>,
    ) -> Option<Self> {
        if mode.is_none() && style.is_none() {
            None
        } else {
            Some(Self { mode, style })
        }
    }
    /// Returns the authored mode, or its original omission.
    #[must_use]
    pub const fn mode(&self) -> Option<CssTextWrapMode> {
        self.mode
    }
    /// Returns the authored style, or its original omission.
    #[must_use]
    pub const fn style(&self) -> Option<CssTextWrapStyle> {
        self.style
    }
}

/// The authored whitespace preservation/collapse choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWhiteSpaceCollapse {
    /// `collapse`: sequences of white space are collapsed into a single character (or, in some cases, no character).
    Collapse,
    /// `discard`: all white space in the element is discarded.
    Discard,
    /// `preserve`: sequences of white space are not collapsed; segment breaks are preserved as forced line breaks.
    Preserve,
    /// `preserve-breaks`: as `collapse`, except that segment breaks are preserved as forced line breaks.
    PreserveBreaks,
    /// `preserve-spaces`: sequences of white space are not collapsed, and tabs and segment breaks are converted to spaces.
    PreserveSpaces,
    /// `break-spaces`: as `preserve`, except that preserved white space and other space separators always take up space, including at the end of a line, and a soft wrap opportunity exists after each of them.
    BreakSpaces,
}

/// Independent trim flags; the empty set is the authored none value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssWhiteSpaceTrim {
    discard_before: bool,
    discard_after: bool,
    discard_inner: bool,
}
impl CssWhiteSpaceTrim {
    /// Constructs any of the eight intrinsically valid trim sets.
    #[must_use]
    pub const fn new(discard_before: bool, discard_after: bool, discard_inner: bool) -> Self {
        Self {
            discard_before,
            discard_after,
            discard_inner,
        }
    }
    /// Constructs the empty none set.
    #[must_use]
    pub const fn none() -> Self {
        Self::new(false, false, false)
    }
    /// Whether `discard-before` was written, collapsing collapsible white space immediately before the start of the element.
    #[must_use]
    pub const fn discard_before(&self) -> bool {
        self.discard_before
    }
    /// Whether `discard-after` was written, collapsing collapsible white space immediately after the end of the element.
    #[must_use]
    pub const fn discard_after(&self) -> bool {
        self.discard_after
    }
    /// Whether `discard-inner` was written, discarding white space at the beginning and end of the element (in a block container, the leading run up to and including the last segment break before the first non-white-space character, and the trailing run starting at the first segment break after the last non-white-space character).
    #[must_use]
    pub const fn discard_inner(&self) -> bool {
        self.discard_inner
    }
    /// Whether this is the empty none set.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        !self.discard_before && !self.discard_after && !self.discard_inner
    }
}

/// The four special white-space spellings, distinct from constituent forms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWhiteSpaceKeyword {
    /// `normal`: collapses white space and wraps lines; equivalent to `collapse wrap`.
    Normal,
    /// `pre`: preserves white space and does not wrap; equivalent to `preserve nowrap`.
    Pre,
    /// `pre-wrap`: preserves white space and wraps lines; equivalent to `preserve wrap`.
    PreWrap,
    /// `pre-line`: preserves segment breaks, collapses other white space and wraps lines; equivalent to `preserve-breaks wrap`.
    PreLine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WhiteSpaceValue {
    Keyword(CssWhiteSpaceKeyword),
    Components {
        collapse: Option<CssWhiteSpaceCollapse>,
        mode: Option<CssTextWrapMode>,
        trim: Option<CssWhiteSpaceTrim>,
    },
}

/// A special white-space spelling or nonempty authored constituent composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssWhiteSpace {
    value: WhiteSpaceValue,
}
impl CssWhiteSpace {
    /// Retains a special spelling without manufacturing authored constituents.
    #[must_use]
    pub const fn from_keyword(keyword: CssWhiteSpaceKeyword) -> Self {
        Self {
            value: WhiteSpaceValue::Keyword(keyword),
        }
    }
    /// Admits a nonempty constituent composition, retaining each omission.
    #[must_use]
    pub const fn try_new(
        collapse: Option<CssWhiteSpaceCollapse>,
        mode: Option<CssTextWrapMode>,
        trim: Option<CssWhiteSpaceTrim>,
    ) -> Option<Self> {
        if collapse.is_none() && mode.is_none() && trim.is_none() {
            None
        } else {
            Some(Self {
                value: WhiteSpaceValue::Components {
                    collapse,
                    mode,
                    trim,
                },
            })
        }
    }
    /// Returns the special spelling, if this is the special-form branch.
    #[must_use]
    pub const fn keyword(&self) -> Option<CssWhiteSpaceKeyword> {
        match self.value {
            WhiteSpaceValue::Keyword(value) => Some(value),
            WhiteSpaceValue::Components { .. } => None,
        }
    }
    /// Returns only an authored component; special forms have no such field.
    #[must_use]
    pub const fn collapse(&self) -> Option<CssWhiteSpaceCollapse> {
        match self.value {
            WhiteSpaceValue::Components { collapse, .. } => collapse,
            WhiteSpaceValue::Keyword(_) => None,
        }
    }
    /// Returns only an authored component; special forms have no such field.
    #[must_use]
    pub const fn mode(&self) -> Option<CssTextWrapMode> {
        match self.value {
            WhiteSpaceValue::Components { mode, .. } => mode,
            WhiteSpaceValue::Keyword(_) => None,
        }
    }
    /// Returns only an authored component; special forms have no such field.
    #[must_use]
    pub const fn trim(&self) -> Option<CssWhiteSpaceTrim> {
        match self.value {
            WhiteSpaceValue::Components { trim, .. } => trim,
            WhiteSpaceValue::Keyword(_) => None,
        }
    }
}

/// The value of `word-break`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWordBreak {
    /// `normal`: words break according to their customary rules.
    Normal,
    /// `break-all`: breaking is allowed within words; typographic letter units, and units of line-breaking class `NU`, `AL` or `SA`, are treated as `ID` (ideographic) for line breaking, and hyphenation is not applied.
    BreakAll,
    /// `keep-all`: breaking is forbidden within words; implicit soft wrap opportunities between typographic letter units, and units of line-breaking class `NU`, `AL`, `AI` or `ID`, are suppressed, except where lexical word breaking provides them.
    KeepAll,
    /// `manual`: as `normal`, except that lexical word breaking is not performed.
    Manual,
    /// `auto-phrase`: as `normal`, except that language-specific content analysis keeps natural phrases together; behaves as `normal` when the content language is unknown or phrase boundaries cannot be detected for it.
    AutoPhrase,
    /// `break-word`: the deprecated keyword, with the same effect as `word-break: normal` and `overflow-wrap: anywhere` regardless of the actual `overflow-wrap` value.
    BreakWord,
}

/// # Specified serialization
///
/// Serializes the specified overflow wrapping keyword before layout resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflowWrap {
    /// `normal`: lines break only at allowed break points.
    Normal,
    /// `break-word`: an otherwise unbreakable sequence may break at an arbitrary point if the line has no other acceptable break point; those breaks are not counted for min-content sizes.
    BreakWord,
    /// `anywhere`: an otherwise unbreakable sequence may break at an arbitrary point if the line has no other acceptable break point; those breaks are counted for min-content sizes.
    Anywhere,
}

/// # Specified serialization
///
/// Serializes the canonical Overflow 3 ellipsis keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextOverflow {
    /// `clip`: inline content that overflows its block container is clipped; characters may be only partially rendered.
    Clip,
    /// `ellipsis`: an ellipsis character (U+2026) is rendered to represent clipped inline content.
    Ellipsis,
}

/// # Specified serialization
///
/// Emits canonical specified syntax from retained semantic fields. Optional values remain omitted;
/// explicit initials remain explicit. This does not resolve percentages or floor actual decoration
/// thickness.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextDecoration {
    line: Option<CssTextDecorationLine>,
    color: Option<Box<CssColor>>,
    style: Option<CssTextDecorationStyle>,
    thickness: Option<CssTextDecorationThickness>,
}

impl CssTextDecoration {
    /// Rejects an empty aggregate and browser-recovered numeric or Color closure.
    /// Parsed recovery retains its authored graph through the parser-owned boundary.
    #[must_use]
    pub fn try_new(
        line: Option<CssTextDecorationLine>,
        color: Option<CssColor>,
        style: Option<CssTextDecorationStyle>,
        thickness: Option<CssTextDecorationThickness>,
    ) -> Option<Self> {
        if (line.is_none() && color.is_none() && style.is_none() && thickness.is_none())
            || color.as_ref().is_some_and(|value| !value.is_closed())
            || thickness.as_ref().is_some_and(|value| matches!(value, CssTextDecorationThickness::Length(value) if !crate::values::text::text_decoration::length_percentage_closed(value)))
        {
            None
        } else {
            Some(Self::new(line, color, style, thickness))
        }
    }

    pub(crate) fn new(
        line: Option<CssTextDecorationLine>,
        color: Option<CssColor>,
        style: Option<CssTextDecorationStyle>,
        thickness: Option<CssTextDecorationThickness>,
    ) -> Self {
        Self {
            line,
            color: color.map(Box::new),
            style,
            thickness,
        }
    }

    /// Returns the authored line component, or `None` when it was omitted.
    #[must_use]
    pub const fn line(&self) -> Option<&CssTextDecorationLine> {
        self.line.as_ref()
    }

    /// Returns the authored color component, or `None` when it was omitted.
    #[must_use]
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }

    /// Returns the authored style component, or `None` when it was omitted.
    #[must_use]
    pub const fn style(&self) -> Option<CssTextDecorationStyle> {
        self.style
    }

    /// Returns the authored thickness component, or `None` when it was omitted.
    #[must_use]
    pub const fn thickness(&self) -> Option<&CssTextDecorationThickness> {
        self.thickness.as_ref()
    }
}

/// # Specified serialization
///
/// Emits canonical specified syntax from retained semantic fields. Optional values remain omitted;
/// explicit initials remain explicit. This does not resolve percentages or floor actual decoration
/// thickness.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextDecorationLine {
    components: Vec<CssTextDecorationLineComponent>,
    kind: CssTextDecorationLineKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CssTextDecorationLineKind {
    None,
    Lines,
    Error(CssTextDecorationErrorLine),
}

/// The `spelling-error` and `grammar-error` keywords of `text-decoration-line`, which are
/// exclusive alternatives and never combine with ordinary lines (Decoration 4 §2.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationErrorLine {
    /// `spelling-error`: the text decoration the user agent uses to highlight spelling mistakes; its appearance is user-agent defined.
    SpellingError,
    /// `grammar-error`: the text decoration the user agent uses to highlight grammar mistakes; its appearance is user-agent defined.
    GrammarError,
}

impl CssTextDecorationLine {
    /// Checks a list of line keywords.
    ///
    /// Returns `None` when `components` is empty or contains the same keyword twice.
    #[must_use]
    pub fn try_new(components: Vec<CssTextDecorationLineComponent>) -> Option<Self> {
        if components.is_empty() || has_duplicate_decoration_line_components(&components) {
            None
        } else {
            Some(Self::new(components))
        }
    }

    #[must_use]
    pub(crate) fn new(components: Vec<CssTextDecorationLineComponent>) -> Self {
        Self {
            components,
            kind: CssTextDecorationLineKind::Lines,
        }
    }

    /// Creates the `none` value, with no lines.
    #[must_use]
    pub fn none() -> Self {
        Self {
            components: Vec::new(),
            kind: CssTextDecorationLineKind::None,
        }
    }

    /// Constructs one exclusive `spelling-error` or `grammar-error` line.
    pub fn error_line(value: CssTextDecorationErrorLine) -> Self {
        Self {
            components: Vec::new(),
            kind: CssTextDecorationLineKind::Error(value),
        }
    }

    /// Returns the exclusive error-line keyword, when this line is one.
    #[must_use]
    pub const fn as_error_line(&self) -> Option<CssTextDecorationErrorLine> {
        match self.kind {
            CssTextDecorationLineKind::Error(value) => Some(value),
            _ => None,
        }
    }

    /// Returns whether this is the `none` keyword.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self.kind, CssTextDecorationLineKind::None)
    }

    /// Returns the ordinary line keywords in authored order; the slice is empty for `none` and for an error line.
    #[must_use]
    pub fn components(&self) -> &[CssTextDecorationLineComponent] {
        &self.components
    }
}

fn has_duplicate_decoration_line_components(components: &[CssTextDecorationLineComponent]) -> bool {
    components.iter().enumerate().any(|(index, component)| {
        components
            .iter()
            .skip(index + 1)
            .any(|candidate| candidate == component)
    })
}

/// A line keyword of `text-decoration-line`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationLineComponent {
    /// `underline`: each line of text is underlined.
    Underline,
    /// `overline`: each line of text has a line over it, on the side opposite an underline.
    Overline,
    /// `line-through`: each line of text has a line through its middle.
    LineThrough,
    /// `blink`: the text blinks (alternates between visible and invisible); user agents may simply not blink it, and the value is deprecated in favor of animations.
    Blink,
}

/// The `text-decoration-style` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationStyle {
    /// `solid`: a single line segment.
    Solid,
    /// `double`: two parallel solid lines with some space between them.
    Double,
    /// `dotted`: a series of round dots.
    Dotted,
    /// `dashed`: a series of square-ended dashes.
    Dashed,
    /// `wavy`: a wavy line.
    Wavy,
}

/// # Specified serialization
///
/// Emits canonical specified syntax from retained semantic fields. Optional values remain omitted;
/// explicit initials remain explicit. This does not resolve percentages or floor actual decoration
/// thickness.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTextDecorationThickness {
    /// `auto`: the user agent chooses the thickness.
    Auto,
    /// `from-font`: the thickness comes from the font's metrics, otherwise as `auto`.
    FromFont,
    /// A signed authored thickness, before the actual value's device-pixel floor.
    Length(CssNumeric<CssLengthPercentage>),
}

impl PartialEq for CssTextDecorationThickness {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Length(left), Self::Length(right)) => left == right,
            (Self::Auto, Self::Auto) | (Self::FromFont, Self::FromFont) => true,
            _ => false,
        }
    }
}

/// The value of `text-transform`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextTransform {
    /// `none`: no effects.
    None,
    /// `math-auto`: the MathML Core `math-auto` transform.
    MathAuto,
    /// A nonempty combination of case, `full-width` and `full-size-kana` transformations.
    Transforms(CssTextTransformSet),
}

/// The exclusive authored case transformation, before language-sensitive processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextTransformCase {
    /// `capitalize`: the first typographic letter unit of each word, if lowercase, is put in titlecase; other characters are unaffected.
    Capitalize,
    /// `uppercase`: all letters are put in uppercase.
    Uppercase,
    /// `lowercase`: all letters are put in lowercase.
    Lowercase,
}

/// A nonempty authored combination of case, width and kana transformations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTextTransformSet {
    case: Option<CssTextTransformCase>,
    full_width: bool,
    full_size_kana: bool,
}

impl CssTextTransformSet {
    /// Rejects the empty set; `none` and `math-auto` are separate whole values.
    pub const fn try_new(
        case: Option<CssTextTransformCase>,
        full_width: bool,
        full_size_kana: bool,
    ) -> Option<Self> {
        if case.is_none() && !full_width && !full_size_kana {
            None
        } else {
            Some(Self {
                case,
                full_width,
                full_size_kana,
            })
        }
    }
    /// Returns the case transformation, or `None` when none was written.
    pub const fn case(&self) -> Option<CssTextTransformCase> {
        self.case
    }
    /// Returns whether `full-width` was written, putting characters in full-width form.
    pub const fn full_width(&self) -> bool {
        self.full_width
    }
    /// Returns whether `full-size-kana` was written, converting small kana to full-size kana.
    pub const fn full_size_kana(&self) -> bool {
        self.full_size_kana
    }
}

/// An authored preference for breaking within an inline box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWrapInside {
    /// `auto`: lines may break at allowed break points within the box.
    Auto,
    /// `avoid`: line breaking within the box is suppressed where other break points exist.
    Avoid,
}

/// An authored break preference used independently by `wrap-before` and `wrap-after`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWrapBoundary {
    /// `auto`: lines may break at allowed break points before or after the box.
    Auto,
    /// `avoid`: line breaking before or after the box is suppressed where other break points exist.
    Avoid,
    /// `avoid-line`: as `avoid`, but only for line breaks.
    AvoidLine,
    /// `avoid-flex`: as `avoid`, but only for flex line breaks.
    AvoidFlex,
    /// `line`: forces a line break before or after the box if it is an inline-level box.
    Line,
    /// `flex`: forces a flex line break before or after the box if it is a flex item in a multi-line flex container.
    Flex,
}

/// An authored line-breaking strictness, without language or layout execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLineBreak {
    /// `auto`: the user agent determines the line-breaking restrictions.
    Auto,
    /// `loose`: the least restrictive set of line-breaking rules.
    Loose,
    /// `normal`: the most common set of line-breaking rules.
    Normal,
    /// `strict`: the most stringent set of line-breaking rules.
    Strict,
    /// `anywhere`: a soft wrap opportunity exists around every typographic character unit.
    Anywhere,
}

/// An authored separator transformation with a mandatory base outside `none`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWordSpaceTransform {
    /// `none`: the property has no effect.
    None,
    /// `space`: expandable separators are replaced by U+0020 SPACE.
    Space {
        /// Whether `auto-phrase` was written: where the content language is known and supported, a virtual expandable separator is inserted at each detected phrase boundary that lacks one.
        auto_phrase: bool,
    },
    /// `ideographic-space`: expandable separators are replaced by U+3000 IDEOGRAPHIC SPACE.
    IdeographicSpace {
        /// Whether `auto-phrase` was written: where the content language is known and supported, a virtual expandable separator is inserted at each detected phrase boundary that lacks one.
        auto_phrase: bool,
    },
}

/// An authored tab interval: a space-advance multiplier or an exact length.
/// Literal ranges are checked by the child; calculation ranges remain deferred.
///
/// # Specified serialization
///
/// Emits canonical specified syntax from retained semantic fields. Optional values remain omitted;
/// explicit initials remain explicit. This does not resolve percentages or floor actual decoration
/// thickness.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTabSize {
    /// A non-negative `<number>` multiple of the advance width of a space character.
    Number(CssNumeric<CssNumber, CssNonNegative>),
    /// A non-negative `<length>`.
    Length(CssNumeric<CssLength, CssNonNegative>),
}

impl PartialEq for CssTabSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::Length(left), Self::Length(right)) => left == right,
            _ => false,
        }
    }
}
use crate::color::CssColor;
