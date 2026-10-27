//! Intrinsic authored Decoration 4 values. Font, color and painting context remain downstream.

use crate::color::CssColor;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueRef, CssValueOrigin,
    CssValueTokenRef,
};
use crate::numeric::{CssLength, CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::effects::CssShadow;
type Result<T> = std::result::Result<T, CssSerializationError>;

macro_rules! keyword {
    ($(#[$meta:meta])* $ty:ident, $($(#[$variant_meta:meta])* $variant:ident => $text:literal),+ $(,)?) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum $ty { $($(#[$variant_meta])* $variant),+ }
        provider!($ty, value, writer => writer.keyword(match value { $(Self::$variant => $text),+ }));
    };
}
macro_rules! provider {
    ($ty:ty, $value:ident, $writer:ident => $body:expr) => {
        impl crate::serialization::specified_rule_serialization::SpecifiedProvider for $ty {
            fn append_to_rule_writer(&self, $writer: &mut SpecifiedRuleWriter) -> Result<()> {
                let $value = self;
                $body
            }
        }
        crate::serialization::specified_rule_serialization::specified_serialization!($ty);
    };
}
keyword!(
    /// The `from-font` or `under` keyword of `text-underline-position`.
    CssUnderlinePositionVertical,
    /// `from-font`: use the underline offset from the first available font's metrics, otherwise as `auto`.
    FromFont => "from-font",
    /// `under`: place the underline under the element's text content, usually clear of descenders.
    Under => "under"
);
keyword!(
    /// A `left` or `right` side keyword of `text-underline-position` and `text-emphasis-position`.
    CssTextSide,
    /// `left`: the left side in vertical typographic modes.
    Left => "left",
    /// `right`: the right side in vertical typographic modes.
    Right => "right"
);
/// A nonempty pair of independently optional vertical and side choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUnderlinePosition {
    vertical: Option<CssUnderlinePositionVertical>,
    side: Option<CssTextSide>,
}
impl CssUnderlinePosition {
    /// Checks a `text-underline-position` combination.
    ///
    /// Returns `None` when both the vertical keyword and the side are omitted.
    pub const fn try_new(
        vertical: Option<CssUnderlinePositionVertical>,
        side: Option<CssTextSide>,
    ) -> Option<Self> {
        if vertical.is_none() && side.is_none() {
            None
        } else {
            Some(Self { vertical, side })
        }
    }
    /// Returns the `from-font` or `under` keyword, or `None` when it was omitted.
    pub const fn vertical(&self) -> Option<CssUnderlinePositionVertical> {
        self.vertical
    }
    /// Returns the `left` or `right` side, or `None` when it was omitted.
    pub const fn side(&self) -> Option<CssTextSide> {
        self.side
    }
}
/// The value of `text-underline-position`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextUnderlinePosition {
    /// `auto`: the user agent places the underline at or under the alphabetic baseline.
    Auto,
    /// An explicit position: a `from-font` or `under` keyword and/or a `left` or `right` side.
    Position(CssUnderlinePosition),
}
/// The value of `text-underline-offset`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTextUnderlineOffset {
    /// `auto`: the user agent chooses the underline offset.
    Auto,
    /// An explicit `<length-percentage>` offset.
    Length(CssTextUnderlineOffsetLength),
}
/// A signed offset with strictly closed original numeric components.
/// Equality compares the numeric value; the child retains, but does not compare, its origins.
#[derive(Clone, Debug)]
pub struct CssTextUnderlineOffsetLength {
    value: CssNumeric<CssLengthPercentage>,
}
impl PartialEq for CssTextUnderlineOffsetLength {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl CssTextUnderlineOffsetLength {
    /// Checks an underline offset.
    ///
    /// Returns `None` when the value is a calculation that was closed by browser recovery rather than by an explicit closing parenthesis.
    pub fn try_new(value: CssNumeric<CssLengthPercentage>) -> Option<Self> {
        length_percentage_closed(&value).then_some(Self { value })
    }
    pub(crate) fn from_parsed(value: CssNumeric<CssLengthPercentage>) -> Self {
        Self { value }
    }
    /// Returns the authored signed `<length-percentage>` offset; a percentage is relative to 1em.
    pub const fn value(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.value
    }
}
provider!(CssTextUnderlineOffsetLength, value, writer => value.value.append_to_rule_writer(writer));
keyword!(
    /// The value of the `text-decoration-skip` shorthand.
    CssTextDecorationSkip,
    /// `none`: sets all sub-properties to `none`.
    None => "none",
    /// `auto`: sets all sub-properties to their initial values.
    Auto => "auto"
);
keyword!(
    /// The value of `text-decoration-skip-self`.
    CssTextDecorationSkipSelf,
    /// `none`: skip nothing; line decorations from ancestor decorating boxes are propagated to or drawn across this box.
    None => "none",
    /// `objects`: skip this element's entire margin box if it is an atomic inline, such as an image or inline-block.
    Objects => "objects"
);
keyword!(
    /// The value of `text-decoration-skip-box`.
    CssTextDecorationSkipBox,
    /// `none`: skip nothing; line decorations from ancestor decorating boxes are drawn from margin edge to margin edge.
    None => "none",
    /// `all`: when drawing decoration lines of an ancestor decorating box, skips the box's own margin, border and padding areas and draws only within its content area.
    All => "all"
);
keyword!(
    /// The value of `text-decoration-skip-inset`.
    CssTextDecorationSkipInset,
    /// `none`: skip nothing; text decoration is drawn from box edge to box edge.
    None => "none",
    /// `auto`: the start and end of the line are placed slightly inwards from the content edge of the decorating box, by an amount that is user-agent defined but not zero.
    Auto => "auto"
);
/// The value of `text-decoration-skip-spaces`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationSkipSpaces {
    /// `none`: spacers are not skipped; they are decorated like any other character.
    None,
    /// `all`: all spacers, plus any adjacent letter-spacing or word-spacing, are skipped.
    All,
    /// `start`: spacers at the start of the line, plus any adjacent letter-spacing or word-spacing, are skipped.
    Start,
    /// `end`: spacers at the end of the line, plus any adjacent letter-spacing or word-spacing, are skipped.
    End,
    /// `start end`: the `start` and `end` behaviors combined, as the grammar `[ start || end ]` allows; the initial value.
    StartEnd,
}
provider!(CssTextDecorationSkipSpaces, value, writer => match value { Self::StartEnd => { writer.keyword("start")?; writer.append(" ")?; writer.keyword("end") }, Self::None => writer.keyword("none"), Self::All => writer.keyword("all"), Self::Start => writer.keyword("start"), Self::End => writer.keyword("end") });
keyword!(
    /// The value of `text-decoration-skip-ink`.
    CssTextDecorationSkipInk,
    /// `auto`: the user agent may interrupt underlines and overlines where the line would cross glyph ink, and should consider the script of the text.
    Auto => "auto",
    /// `none`: the user agent must draw continuous underlines and overlines, without interruptions where they cross a glyph.
    None => "none",
    /// `all`: the user agent must interrupt underlines and overlines where the line would cross glyph ink.
    All => "all"
);
impl CssTextDecorationSkip {
    /// Returns the `text-decoration-skip-self` value the shorthand sets: `none` for `none`, `objects` for `auto`.
    pub const fn expanded_self(self) -> CssTextDecorationSkipSelf {
        match self {
            Self::None => CssTextDecorationSkipSelf::None,
            Self::Auto => CssTextDecorationSkipSelf::Objects,
        }
    }
    /// Returns the `text-decoration-skip-box` value the shorthand sets, which is always `none`.
    pub const fn expanded_box(self) -> CssTextDecorationSkipBox {
        CssTextDecorationSkipBox::None
    }
    /// Returns the `text-decoration-skip-inset` value the shorthand sets, which is always `none`.
    pub const fn expanded_inset(self) -> CssTextDecorationSkipInset {
        CssTextDecorationSkipInset::None
    }
    /// Returns the `text-decoration-skip-spaces` value the shorthand sets: `none` for `none`, `start end` for `auto`.
    pub const fn expanded_spaces(self) -> CssTextDecorationSkipSpaces {
        match self {
            Self::None => CssTextDecorationSkipSpaces::None,
            Self::Auto => CssTextDecorationSkipSpaces::StartEnd,
        }
    }
    /// Returns the `text-decoration-skip-ink` value the shorthand sets: `none` for `none`, `auto` for `auto`.
    pub const fn expanded_ink(self) -> CssTextDecorationSkipInk {
        match self {
            Self::None => CssTextDecorationSkipInk::None,
            Self::Auto => CssTextDecorationSkipInk::Auto,
        }
    }
}
keyword!(
    /// The `filled` or `open` fill keyword of `text-emphasis-style`.
    CssTextEmphasisFill,
    /// `filled`: the mark shape is filled with solid color.
    Filled => "filled",
    /// `open`: the mark shape is hollow.
    Open => "open"
);
keyword!(
    /// The mark shape keyword of `text-emphasis-style`.
    CssTextEmphasisShape,
    /// `dot`: small circles as marks (U+2022 filled, U+25E6 open).
    Dot => "dot",
    /// `circle`: large circles as marks (U+25CF filled, U+25CB open).
    Circle => "circle",
    /// `double-circle`: double circles as marks (U+25C9 filled, U+25CE open).
    DoubleCircle => "double-circle",
    /// `triangle`: triangles as marks (U+25B2 filled, U+25B3 open).
    Triangle => "triangle",
    /// `sesame`: sesames as marks (U+FE45 filled, U+FE46 open).
    Sesame => "sesame"
);
/// Authored fill and shape slots; omitted shape remains contextual and unresolved.
/// Equality compares authored slots independently of their occurrence origins.
#[derive(Clone, Debug)]
pub struct CssTextEmphasisMark {
    fill: Option<CssTextEmphasisFill>,
    shape: Option<CssTextEmphasisShape>,
    fill_origin: Option<CssValueOrigin>,
    shape_origin: Option<CssValueOrigin>,
}
impl PartialEq for CssTextEmphasisMark {
    fn eq(&self, other: &Self) -> bool {
        self.fill == other.fill && self.shape == other.shape
    }
}
impl Eq for CssTextEmphasisMark {}
impl CssTextEmphasisMark {
    /// Checks a `text-emphasis-style` mark combination.
    ///
    /// Returns `None` when both the fill and the shape are omitted. The retained origins are programmatic.
    pub fn try_new(
        fill: Option<CssTextEmphasisFill>,
        shape: Option<CssTextEmphasisShape>,
    ) -> Option<Self> {
        if fill.is_none() && shape.is_none() {
            None
        } else {
            Some(Self {
                fill,
                shape,
                fill_origin: if fill.is_some() {
                    Some(CssValueOrigin::Programmatic)
                } else {
                    None
                },
                shape_origin: if shape.is_some() {
                    Some(CssValueOrigin::Programmatic)
                } else {
                    None
                },
            })
        }
    }
    pub(crate) fn from_parsed(
        fill: Option<CssTextEmphasisFill>,
        shape: Option<CssTextEmphasisShape>,
        fill_origin: Option<CssValueOrigin>,
        shape_origin: Option<CssValueOrigin>,
    ) -> Option<Self> {
        let mut value = Self::try_new(fill, shape)?;
        value.fill_origin = fill_origin;
        value.shape_origin = shape_origin;
        Some(value)
    }
    /// Returns the origin of the fill keyword, or `None` when the fill was omitted.
    pub const fn fill_origin(&self) -> Option<&CssValueOrigin> {
        self.fill_origin.as_ref()
    }
    /// Returns the origin of the shape keyword, or `None` when the shape was omitted.
    pub const fn shape_origin(&self) -> Option<&CssValueOrigin> {
        self.shape_origin.as_ref()
    }
    /// Returns the authored `filled` or `open` keyword, or `None` when it was omitted.
    pub const fn fill(&self) -> Option<CssTextEmphasisFill> {
        self.fill
    }
    /// Returns the authored shape keyword, or `None` when it was omitted; an omitted shape is contextual and stays unresolved.
    pub const fn shape(&self) -> Option<CssTextEmphasisShape> {
        self.shape
    }
    /// Returns the authored fill, or `filled` when it was omitted.
    pub const fn effective_fill(&self) -> CssTextEmphasisFill {
        match self.fill {
            Some(value) => value,
            None => CssTextEmphasisFill::Filled,
        }
    }
}
/// A retained string, without author-advice truncation or glyph selection.
/// Leaf equality retains original spelling and provenance; style equality compares decoded content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTextEmphasisString {
    component: Box<CssComponentValue>,
}
impl CssTextEmphasisString {
    /// Wraps a string-token component as an emphasis string.
    ///
    /// Returns `None` when the component is not a string token or its string was closed by browser recovery at the end of input.
    pub fn try_from_component(component: CssComponentValue) -> Option<Self> {
        let value = Self::from_parsed(component)?;
        value.is_closed().then_some(value)
    }
    pub(crate) fn from_parsed(component: CssComponentValue) -> Option<Self> {
        matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::String(_))
        )
        .then(|| Self {
            component: Box::new(component),
        })
    }
    fn is_closed(&self) -> bool {
        !matches!(
            self.component.implicit_termination_origin(),
            Some(CssValueOrigin::ImplicitClosure { .. })
        )
    }
    /// Creates an emphasis string from decoded text.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidString` error when `value` contains U+0000 or does not tokenize back to the same text.
    pub fn try_new(value: impl Into<String>) -> std::result::Result<Self, CssComponentValueError> {
        Ok(Self {
            component: Box::new(CssComponentValue::try_string(value)?),
        })
    }
    /// Returns the retained string component, which keeps the original token spelling and provenance.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    /// Returns the origin of the string component.
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
    /// Returns the decoded string text.
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::String(value)) => value,
            _ => unreachable!("checked string"),
        }
    }
}
/// The style part of `text-emphasis`.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTextEmphasisStyle {
    /// `none`: no emphasis marks.
    None,
    /// An emphasis mark given by a fill and/or a shape keyword.
    Mark(CssTextEmphasisMark),
    /// An authored `<string>` used as the emphasis mark.
    String(CssTextEmphasisString),
}
impl PartialEq for CssTextEmphasisStyle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Mark(left), Self::Mark(right)) => left == right,
            (Self::String(left), Self::String(right)) => left.as_str() == right.as_str(),
            _ => false,
        }
    }
}
/// The `text-emphasis` shorthand: an optional style and an optional color.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextEmphasis {
    style: Option<CssTextEmphasisStyle>,
    color: Option<Box<CssColor>>,
}
impl CssTextEmphasis {
    /// Checks a `text-emphasis` combination.
    ///
    /// Returns `None` when both parts are omitted, or when the color or the emphasis string was closed by browser recovery.
    pub fn try_new(style: Option<CssTextEmphasisStyle>, color: Option<CssColor>) -> Option<Self> {
        if style.is_none() && color.is_none()
            || color.as_ref().is_some_and(|v| !v.is_closed())
            || matches!(style.as_ref(), Some(CssTextEmphasisStyle::String(value)) if !value.is_closed())
        {
            return None;
        }
        Some(Self::new(style, color))
    }
    pub(crate) fn new(style: Option<CssTextEmphasisStyle>, color: Option<CssColor>) -> Self {
        Self {
            style,
            color: color.map(Box::new),
        }
    }
    /// Returns the authored style, or `None` when it was omitted.
    pub const fn style(&self) -> Option<&CssTextEmphasisStyle> {
        self.style.as_ref()
    }
    /// Returns the authored color, or `None` when it was omitted.
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }
}
keyword!(
    /// The `over` or `under` keyword of `text-emphasis-position`.
    CssTextEmphasisVertical,
    /// `over`: marks are drawn over the text in horizontal typographic modes.
    Over => "over",
    /// `under`: marks are drawn under the text in horizontal typographic modes.
    Under => "under"
);
/// Authored vertical and optional side slots, with separately retained occurrence origins.
/// Equality retains the distinction between an omitted and explicit side.
#[derive(Clone, Debug)]
pub struct CssTextEmphasisPosition {
    vertical: CssTextEmphasisVertical,
    side: Option<CssTextSide>,
    vertical_origin: CssValueOrigin,
    side_origin: Option<CssValueOrigin>,
}
impl PartialEq for CssTextEmphasisPosition {
    fn eq(&self, other: &Self) -> bool {
        self.vertical == other.vertical && self.side == other.side
    }
}
impl Eq for CssTextEmphasisPosition {}
impl CssTextEmphasisPosition {
    /// Creates a position from a vertical keyword and an optional side; both origins are programmatic.
    pub fn new(vertical: CssTextEmphasisVertical, side: Option<CssTextSide>) -> Self {
        Self {
            vertical,
            side,
            vertical_origin: CssValueOrigin::Programmatic,
            side_origin: if side.is_some() {
                Some(CssValueOrigin::Programmatic)
            } else {
                None
            },
        }
    }
    pub(crate) fn from_parsed(
        vertical: CssTextEmphasisVertical,
        side: Option<CssTextSide>,
        vertical_origin: CssValueOrigin,
        side_origin: Option<CssValueOrigin>,
    ) -> Self {
        Self {
            vertical,
            side,
            vertical_origin,
            side_origin,
        }
    }
    /// Returns the origin of the vertical keyword.
    pub const fn vertical_origin(&self) -> &CssValueOrigin {
        &self.vertical_origin
    }
    /// Returns the origin of the side keyword, or `None` when the side was omitted.
    pub const fn side_origin(&self) -> Option<&CssValueOrigin> {
        self.side_origin.as_ref()
    }
    /// Returns the `over` or `under` keyword.
    pub const fn vertical(&self) -> CssTextEmphasisVertical {
        self.vertical
    }
    /// Returns the `left` or `right` side, or `None` when it was omitted.
    pub const fn side(&self) -> Option<CssTextSide> {
        self.side
    }
    /// Returns the authored side, or `right` when it was omitted.
    pub const fn effective_side(&self) -> CssTextSide {
        match self.side {
            Some(value) => value,
            None => CssTextSide::Right,
        }
    }
}
/// A nonempty unique subset of emphasis skipping roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTextEmphasisSkip {
    spaces: bool,
    punctuation: bool,
    symbols: bool,
    narrow: bool,
}
impl CssTextEmphasisSkip {
    /// Checks a set of emphasis skip roles.
    ///
    /// Returns `None` when no role is selected.
    pub const fn try_new(
        spaces: bool,
        punctuation: bool,
        symbols: bool,
        narrow: bool,
    ) -> Option<Self> {
        if spaces || punctuation || symbols || narrow {
            Some(Self {
                spaces,
                punctuation,
                symbols,
                narrow,
            })
        } else {
            None
        }
    }
    /// Returns the initial value: `spaces` and `punctuation` are skipped.
    pub const fn initial() -> Self {
        Self {
            spaces: true,
            punctuation: true,
            symbols: false,
            narrow: false,
        }
    }
    /// Returns whether `spaces` is selected: word separators and other Unicode separator characters are skipped.
    pub const fn spaces(&self) -> bool {
        self.spaces
    }
    /// Returns whether `punctuation` is selected: punctuation characters that are not symbols are skipped.
    pub const fn punctuation(&self) -> bool {
        self.punctuation
    }
    /// Returns whether `symbols` is selected: symbol characters are skipped.
    pub const fn symbols(&self) -> bool {
        self.symbols
    }
    /// Returns whether `narrow` is selected: characters that are not fullwidth or wide are skipped.
    pub const fn narrow(&self) -> bool {
        self.narrow
    }
}
/// A checked text shadow, narrowing only the signed box-shadow spread policy.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextShadowLayer {
    shadow: CssShadow,
}
impl CssTextShadowLayer {
    /// Checks the parts of a text shadow layer.
    ///
    /// Returns `None` when a spread radius is given without a blur radius, or when a part contains a calculation or color closed by browser recovery.
    pub fn try_new(
        inset: bool,
        offset_x: CssNumeric<CssLength>,
        offset_y: CssNumeric<CssLength>,
        blur_radius: Option<CssNumeric<CssLength, CssNonNegative>>,
        spread_radius: Option<CssNumeric<CssLength, CssNonNegative>>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        let spread = spread_radius.map(CssNumeric::into_any_range);
        Self::try_from_shadow(CssShadow::try_new(
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread,
            color,
        )?)
    }
    /// Wraps a shadow as a text shadow layer.
    ///
    /// Returns `None` when the spread radius is negative or when a part contains a calculation or color closed by browser recovery.
    pub fn try_from_shadow(shadow: CssShadow) -> Option<Self> {
        let layer = Self::from_parsed(shadow)?;
        layer.is_closed().then_some(layer)
    }
    fn is_closed(&self) -> bool {
        let shadow = &self.shadow;
        length_closed(shadow.offset_x())
            && length_closed(shadow.offset_y())
            && shadow.blur_radius().is_none_or(|v| {
                v.calculation()
                    .is_none_or(|c| c.components().first_implicit_origin().is_none())
            })
            && shadow.spread_radius().is_none_or(length_closed)
            && shadow.color().is_none_or(CssColor::is_closed)
    }
    pub(crate) fn from_parsed(shadow: CssShadow) -> Option<Self> {
        if let Some(spread) = shadow.spread_radius() {
            spread.clone().try_narrow::<CssNonNegative>().ok()?;
        }
        Some(Self { shadow })
    }
    /// Returns the underlying shadow.
    pub const fn shadow(&self) -> &CssShadow {
        &self.shadow
    }
}
pub(crate) fn length_closed(value: &CssNumeric<CssLength>) -> bool {
    value
        .calculation()
        .is_none_or(|c| c.components().first_implicit_origin().is_none())
}
pub(crate) fn length_percentage_closed(value: &CssNumeric<CssLengthPercentage>) -> bool {
    value
        .calculation()
        .is_none_or(|c| c.components().first_implicit_origin().is_none())
}
/// A nonempty comma-separated list of text shadow layers.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextShadowList {
    shadows: Vec<CssTextShadowLayer>,
}
impl CssTextShadowList {
    /// Checks a list of text shadow layers.
    ///
    /// Returns `None` when `shadows` is empty or any layer contains a part closed by browser recovery.
    pub fn try_new(shadows: Vec<CssTextShadowLayer>) -> Option<Self> {
        if !shadows.iter().all(CssTextShadowLayer::is_closed) {
            return None;
        }
        Self::from_parsed(shadows)
    }
    pub(crate) fn from_parsed(shadows: Vec<CssTextShadowLayer>) -> Option<Self> {
        if shadows.is_empty() {
            None
        } else {
            Some(Self { shadows })
        }
    }
    /// Returns the layers in authored order; the slice is never empty.
    pub fn shadows(&self) -> &[CssTextShadowLayer] {
        &self.shadows
    }
}
/// The value of `text-shadow`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTextShadow {
    /// `none`: no shadow.
    None,
    /// One or more shadow layers.
    Shadows(CssTextShadowList),
}

provider!(CssUnderlinePosition, value, writer => { if let Some(vertical) = value.vertical { vertical.append_to_rule_writer(writer)?; }
    if let Some(side) = value.side { if value.vertical.is_some() { writer.append(" ")?; } side.append_to_rule_writer(writer)?; } Ok(()) });
provider!(CssTextUnderlinePosition, value, writer => match value { Self::Auto => writer.keyword("auto"), Self::Position(value) => value.append_to_rule_writer(writer) });
provider!(CssTextUnderlineOffset, value, writer => match value { Self::Auto => writer.keyword("auto"), Self::Length(value) => value.append_to_rule_writer(writer) });
provider!(CssTextEmphasisMark, value, writer => {
    let emitted_fill = value.fill.filter(|v| *v != CssTextEmphasisFill::Filled || value.shape.is_none());
    if let Some(fill) = emitted_fill { fill.append_to_rule_writer(writer)?; } else if value.fill.is_some() { writer.context.charge_input(1)?; }
    if let Some(shape) = value.shape { if emitted_fill.is_some() { writer.append(" ")?; } shape.append_to_rule_writer(writer)?; } Ok(())
});
provider!(CssTextEmphasisString, value, writer => { writer.node()?; writer.append_string(value.as_str()) });
provider!(CssTextEmphasisStyle, value, writer => match value { Self::None => writer.keyword("none"), Self::Mark(value) => value.append_to_rule_writer(writer), Self::String(value) => value.append_to_rule_writer(writer) });
provider!(CssTextEmphasis, value, writer => { writer.node()?;
    if let Some(style) = &value.style { writer.source_member(0, |writer| style.append_to_rule_writer(writer))?; }
    if let Some(color) = &value.color { writer.source_member(1, |writer| {
        if value.style.is_some() { writer.append(" ")?; }
        color.append_specified(&mut writer.context, &mut writer.css)
    })?; } Ok(()) });
provider!(CssTextEmphasisPosition, value, writer => { value.vertical.append_to_rule_writer(writer)?; if let Some(side) = value.side { if side == CssTextSide::Left { writer.append(" ")?; side.append_to_rule_writer(writer)?; } else { writer.context.charge_input(1)?; } } Ok(()) });
provider!(CssTextEmphasisSkip, value, writer => { let mut separated = false; for (present, word) in [(value.spaces,"spaces"),(value.punctuation,"punctuation"),(value.symbols,"symbols"),(value.narrow,"narrow")] { if present { if separated { writer.append(" ")?; } writer.keyword(word)?; separated = true; } } Ok(()) });
provider!(CssTextShadowLayer, value, writer => value.shadow.append_to_rule_writer(writer));
provider!(CssTextShadowList, value, writer => { writer.node()?; for (index, shadow) in value.shadows.iter().enumerate() { if index != 0 { writer.append(", ")?; } shadow.append_to_rule_writer(writer)?; } Ok(()) });
provider!(CssTextShadow, value, writer => match value { Self::None => writer.keyword("none"), Self::Shadows(value) => value.append_to_rule_writer(writer) });

#[cfg(test)]
mod provider_tests {
    use super::*;
    use crate::serialization::CssSerializationLimits as L;
    use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as K;

    fn shared(
        css: &str,
        input: usize,
        projection: usize,
        append: impl Fn(&mut SpecifiedRuleWriter) -> Result<()>,
    ) {
        let expected = format!("{css}éauto");
        for (limits, failure) in [
            (L::new(input + 1, projection + 1, expected.len()), None),
            (
                L::new(input, projection + 1, expected.len()),
                Some(K::InputNodeLimit),
            ),
            (
                L::new(input + 1, projection, expected.len()),
                Some(K::ProjectionNodeLimit),
            ),
            (
                L::new(input + 1, projection + 1, expected.len() - 1),
                Some(K::ByteLimit),
            ),
        ] {
            for _ in 0..2 {
                let mut writer = SpecifiedRuleWriter::new(limits);
                append(&mut writer).unwrap();
                writer.append("é").unwrap();
                let result = CssTextDecorationSkip::Auto.append_to_rule_writer(&mut writer);
                if let Some(kind) = failure {
                    assert_eq!(result.unwrap_err().kind(), kind);
                    assert_eq!(writer.css, format!("{css}é"));
                } else {
                    result.unwrap();
                    assert_eq!(writer.css, expected);
                }
            }
        }
        let mut writer = SpecifiedRuleWriter::new(L::new(input + 1, projection + 1, 4));
        writer
            .without_output(|writer| writer.without_output(|writer| append(writer)))
            .unwrap();
        assert!(writer.css.is_empty());
        CssTextDecorationSkip::Auto
            .append_to_rule_writer(&mut writer)
            .unwrap();
        assert_eq!(writer.css, "auto");
        for (limits, kind) in [
            (L::new(input - 1, usize::MAX, 0), K::InputNodeLimit),
            (
                L::new(usize::MAX, projection - 1, 0),
                K::ProjectionNodeLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(
                writer
                    .without_output(|writer| append(writer))
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert!(!writer.context.output_suppressed());
            assert!(writer.css.is_empty());
        }
    }

    #[test]
    fn new_providers_share_exact_work_with_siblings_and_nested_suppression() {
        shared("from-font right", 2, 2, |w| {
            CssUnderlinePosition::try_new(
                Some(CssUnderlinePositionVertical::FromFont),
                Some(CssTextSide::Right),
            )
            .unwrap()
            .append_to_rule_writer(w)
        });
        shared("auto", 1, 1, |w| {
            CssTextUnderlinePosition::Auto.append_to_rule_writer(w)
        });
        shared("-25%", 1, 1, |w| {
            CssTextUnderlineOffset::Length(
                CssTextUnderlineOffsetLength::try_new(
                    CssNumeric::<CssLengthPercentage>::try_from_component(
                        CssComponentValue::try_token("-25%").unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .append_to_rule_writer(w)
        });
        shared("none", 1, 1, |w| {
            CssTextDecorationSkip::None.append_to_rule_writer(w)
        });
        shared("objects", 1, 1, |w| {
            CssTextDecorationSkipSelf::Objects.append_to_rule_writer(w)
        });
        shared("all", 1, 1, |w| {
            CssTextDecorationSkipBox::All.append_to_rule_writer(w)
        });
        shared("auto", 1, 1, |w| {
            CssTextDecorationSkipInset::Auto.append_to_rule_writer(w)
        });
        shared("start end", 2, 2, |w| {
            CssTextDecorationSkipSpaces::StartEnd.append_to_rule_writer(w)
        });
        shared("all", 1, 1, |w| {
            CssTextDecorationSkipInk::All.append_to_rule_writer(w)
        });
        shared("dot", 2, 1, |w| {
            CssTextEmphasisMark::try_new(
                Some(CssTextEmphasisFill::Filled),
                Some(CssTextEmphasisShape::Dot),
            )
            .unwrap()
            .append_to_rule_writer(w)
        });
        shared("open triangle", 2, 2, |w| {
            CssTextEmphasisStyle::Mark(
                CssTextEmphasisMark::try_new(
                    Some(CssTextEmphasisFill::Open),
                    Some(CssTextEmphasisShape::Triangle),
                )
                .unwrap(),
            )
            .append_to_rule_writer(w)
        });
        shared("\"😀x\"", 1, 1, |w| {
            CssTextEmphasisString::try_new("😀x")
                .unwrap()
                .append_to_rule_writer(w)
        });
        shared("open currentcolor", 3, 3, |w| {
            CssTextEmphasis::try_new(
                Some(CssTextEmphasisStyle::Mark(
                    CssTextEmphasisMark::try_new(Some(CssTextEmphasisFill::Open), None).unwrap(),
                )),
                Some(CssColor::current_color()),
            )
            .unwrap()
            .append_to_rule_writer(w)
        });
        shared("over", 2, 1, |w| {
            CssTextEmphasisPosition::new(CssTextEmphasisVertical::Over, Some(CssTextSide::Right))
                .append_to_rule_writer(w)
        });
        shared("spaces punctuation symbols narrow", 4, 4, |w| {
            CssTextEmphasisSkip::try_new(true, true, true, true)
                .unwrap()
                .append_to_rule_writer(w)
        });
        shared("none", 1, 1, |w| {
            CssTextShadow::None.append_to_rule_writer(w)
        });
        let length = |text| {
            CssNumeric::<CssLength>::try_from_component(CssComponentValue::try_token(text).unwrap())
                .unwrap()
        };
        let layer = CssTextShadowLayer::try_new(
            true,
            length("1px"),
            length("2px"),
            None,
            None,
            Some(CssColor::current_color()),
        )
        .unwrap();
        let before = layer.clone();
        shared("currentcolor 1px 2px inset", 5, 5, |w| {
            layer.append_to_rule_writer(w)
        });
        let list = CssTextShadowList::try_new(vec![layer.clone(), layer.clone()]).unwrap();
        shared(
            "currentcolor 1px 2px inset, currentcolor 1px 2px inset",
            11,
            11,
            |w| list.append_to_rule_writer(w),
        );
        assert_eq!(layer, before);
        assert!(CssTextEmphasis::try_new(None, None).is_none());
        assert!(CssTextEmphasisSkip::try_new(false, false, false, false).is_none());
    }
}
