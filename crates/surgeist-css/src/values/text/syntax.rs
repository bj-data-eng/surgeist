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
    Ltr,
    Rtl,
}

/// # Specified serialization
///
/// Serializes the canonical specified writing-mode keyword without applying inheritance or
/// contextual text and layout behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
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
    None,
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

    #[must_use]
    pub const fn from_calculation(value: CssCalculation<CssInteger>) -> Self {
        Self {
            value: CssTextCombineDigitCountValue::Calculation(value),
        }
    }

    #[must_use]
    pub const fn literal(&self) -> Option<i32> {
        match self.value {
            CssTextCombineDigitCountValue::Literal(value) => Some(value),
            CssTextCombineDigitCountValue::Calculation(_) => None,
        }
    }

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
    Mixed,
    Upright,
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
    Normal,
    Embed,
    Isolate,
    BidiOverride,
    IsolateOverride,
    Plaintext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
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
    pub const fn length(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.length
    }
    pub const fn hanging(&self) -> bool {
        self.hanging
    }
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
    Baseline,
    Sub,
    Super,
    TextTop,
    TextBottom,
    Middle,
    Top,
    Bottom,
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
    Wrap,
    NoWrap,
}

/// The authored line selection style; no wrapping algorithm is executed here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextWrapStyle {
    Auto,
    Balance,
    Stable,
    Pretty,
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
    Collapse,
    Discard,
    Preserve,
    PreserveBreaks,
    PreserveSpaces,
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
    /// Whether leading whitespace is marked for discarding.
    #[must_use]
    pub const fn discard_before(&self) -> bool {
        self.discard_before
    }
    /// Whether trailing whitespace is marked for discarding.
    #[must_use]
    pub const fn discard_after(&self) -> bool {
        self.discard_after
    }
    /// Whether inner whitespace is marked for discarding.
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
    Normal,
    Pre,
    PreWrap,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWordBreak {
    Normal,
    BreakAll,
    KeepAll,
    Manual,
    AutoPhrase,
    BreakWord,
}

/// # Specified serialization
///
/// Serializes the specified overflow wrapping keyword before layout resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflowWrap {
    Normal,
    BreakWord,
    Anywhere,
}

/// # Specified serialization
///
/// Serializes the canonical Overflow 3 ellipsis keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextOverflow {
    Clip,
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

    #[must_use]
    pub const fn line(&self) -> Option<&CssTextDecorationLine> {
        self.line.as_ref()
    }

    #[must_use]
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<CssTextDecorationStyle> {
        self.style
    }

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
    SpellingError,
    GrammarError,
}

impl CssTextDecorationLine {
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

    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self.kind, CssTextDecorationLineKind::None)
    }

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationLineComponent {
    Underline,
    Overline,
    LineThrough,
    Blink,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationStyle {
    Solid,
    Double,
    Dotted,
    Dashed,
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
    Auto,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextTransform {
    None,
    MathAuto,
    Transforms(CssTextTransformSet),
}

/// The exclusive authored case transformation, before language-sensitive processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextTransformCase {
    Capitalize,
    Uppercase,
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
    pub const fn case(&self) -> Option<CssTextTransformCase> {
        self.case
    }
    pub const fn full_width(&self) -> bool {
        self.full_width
    }
    pub const fn full_size_kana(&self) -> bool {
        self.full_size_kana
    }
}

/// An authored preference for breaking within an inline box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWrapInside {
    Auto,
    Avoid,
}

/// An authored break preference used independently by `wrap-before` and `wrap-after`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWrapBoundary {
    Auto,
    Avoid,
    AvoidLine,
    AvoidFlex,
    Line,
    Flex,
}

/// An authored line-breaking strictness, without language or layout execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLineBreak {
    Auto,
    Loose,
    Normal,
    Strict,
    Anywhere,
}

/// An authored separator transformation with a mandatory base outside `none`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWordSpaceTransform {
    None,
    Space { auto_phrase: bool },
    IdeographicSpace { auto_phrase: bool },
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
    Number(CssNumeric<CssNumber, CssNonNegative>),
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
