//! Intrinsic authored Decoration 4 values. Font, color and painting context remain downstream.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::*;
type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! keyword {
    ($ty:ident, $($variant:ident => $text:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum $ty { $($variant),+ }
        provider!($ty, value, writer => writer.keyword(match value { $(Self::$variant => $text),+ }));
    };
}
macro_rules! provider {
    ($ty:ty, $value:ident, $writer:ident => $body:expr) => {
        impl $ty {
            /// Emits canonical specified syntax without contextual computation.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }
            /// Shares input, projection and UTF-8 byte limits across all children; failure is atomic.
            pub fn serialize_specified_with_limits(&self, limits: CssSpecifiedValueSerializationLimits) -> Result<String> {
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }
            pub(crate) fn append_to_rule_writer(&self, $writer: &mut SpecifiedRuleWriter) -> Result<()> {
                let $value = self;
                $body
            }
        }
    };
}
keyword!(CssUnderlinePositionVertical, FromFont => "from-font", Under => "under");
keyword!(CssTextSide, Left => "left", Right => "right");
/// A nonempty pair of independently optional vertical and side choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUnderlinePosition {
    vertical: Option<CssUnderlinePositionVertical>,
    side: Option<CssTextSide>,
}
impl CssUnderlinePosition {
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
    pub const fn vertical(&self) -> Option<CssUnderlinePositionVertical> {
        self.vertical
    }
    pub const fn side(&self) -> Option<CssTextSide> {
        self.side
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextUnderlinePosition {
    Auto,
    Position(CssUnderlinePosition),
}
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTextUnderlineOffset {
    Auto,
    Length(CssTextUnderlineOffsetLength),
}
/// A signed offset with strictly closed original numeric components.
/// Equality compares exact numeric structure; the borrowed child retains its origins.
#[derive(Clone, Debug)]
pub struct CssTextUnderlineOffsetLength {
    value: CssSpecifiedLengthPercentage,
}
impl PartialEq for CssTextUnderlineOffsetLength {
    fn eq(&self, other: &Self) -> bool {
        self.value.structural_eq(&other.value)
    }
}
impl CssTextUnderlineOffsetLength {
    pub fn try_new(value: CssSpecifiedLengthPercentage) -> Option<Self> {
        length_percentage_closed(&value).then_some(Self { value })
    }
    pub(crate) fn from_parsed(value: CssSpecifiedLengthPercentage) -> Self {
        Self { value }
    }
    pub const fn value(&self) -> &CssSpecifiedLengthPercentage {
        &self.value
    }
}
provider!(CssTextUnderlineOffsetLength, value, writer => value.value.append_to_rule_writer(writer));
keyword!(CssTextDecorationSkip, None => "none", Auto => "auto");
keyword!(CssTextDecorationSkipSelf, None => "none", Objects => "objects");
keyword!(CssTextDecorationSkipBox, None => "none", All => "all");
keyword!(CssTextDecorationSkipInset, None => "none", Auto => "auto");
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationSkipSpaces {
    None,
    All,
    Start,
    End,
    StartEnd,
}
provider!(CssTextDecorationSkipSpaces, value, writer => match value { Self::StartEnd => { writer.keyword("start")?; writer.append(" ")?; writer.keyword("end") }, Self::None => writer.keyword("none"), Self::All => writer.keyword("all"), Self::Start => writer.keyword("start"), Self::End => writer.keyword("end") });
keyword!(CssTextDecorationSkipInk, Auto => "auto", None => "none", All => "all");
impl CssTextDecorationSkip {
    pub const fn expanded_self(self) -> CssTextDecorationSkipSelf {
        match self {
            Self::None => CssTextDecorationSkipSelf::None,
            Self::Auto => CssTextDecorationSkipSelf::Objects,
        }
    }
    pub const fn expanded_box(self) -> CssTextDecorationSkipBox {
        CssTextDecorationSkipBox::None
    }
    pub const fn expanded_inset(self) -> CssTextDecorationSkipInset {
        CssTextDecorationSkipInset::None
    }
    pub const fn expanded_spaces(self) -> CssTextDecorationSkipSpaces {
        match self {
            Self::None => CssTextDecorationSkipSpaces::None,
            Self::Auto => CssTextDecorationSkipSpaces::StartEnd,
        }
    }
    pub const fn expanded_ink(self) -> CssTextDecorationSkipInk {
        match self {
            Self::None => CssTextDecorationSkipInk::None,
            Self::Auto => CssTextDecorationSkipInk::Auto,
        }
    }
}
keyword!(CssTextEmphasisFill, Filled => "filled", Open => "open");
keyword!(CssTextEmphasisShape, Dot => "dot", Circle => "circle", DoubleCircle => "double-circle", Triangle => "triangle", Sesame => "sesame");
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
    pub const fn fill_origin(&self) -> Option<&CssValueOrigin> {
        self.fill_origin.as_ref()
    }
    pub const fn shape_origin(&self) -> Option<&CssValueOrigin> {
        self.shape_origin.as_ref()
    }
    pub const fn fill(&self) -> Option<CssTextEmphasisFill> {
        self.fill
    }
    pub const fn shape(&self) -> Option<CssTextEmphasisShape> {
        self.shape
    }
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
    pub fn try_new(value: impl Into<String>) -> std::result::Result<Self, CssComponentValueError> {
        Ok(Self {
            component: Box::new(CssComponentValue::try_string(value)?),
        })
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::String(value)) => value,
            _ => unreachable!("checked string"),
        }
    }
}
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTextEmphasisStyle {
    None,
    Mark(CssTextEmphasisMark),
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
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextEmphasis {
    style: Option<CssTextEmphasisStyle>,
    color: Option<Box<CssColor>>,
}
impl CssTextEmphasis {
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
    pub const fn style(&self) -> Option<&CssTextEmphasisStyle> {
        self.style.as_ref()
    }
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }
}
keyword!(CssTextEmphasisVertical, Over => "over", Under => "under");
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
    pub const fn vertical_origin(&self) -> &CssValueOrigin {
        &self.vertical_origin
    }
    pub const fn side_origin(&self) -> Option<&CssValueOrigin> {
        self.side_origin.as_ref()
    }
    pub const fn vertical(&self) -> CssTextEmphasisVertical {
        self.vertical
    }
    pub const fn side(&self) -> Option<CssTextSide> {
        self.side
    }
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
    pub const fn initial() -> Self {
        Self {
            spaces: true,
            punctuation: true,
            symbols: false,
            narrow: false,
        }
    }
    pub const fn spaces(&self) -> bool {
        self.spaces
    }
    pub const fn punctuation(&self) -> bool {
        self.punctuation
    }
    pub const fn symbols(&self) -> bool {
        self.symbols
    }
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
    pub fn try_new(
        inset: bool,
        offset_x: CssSpecifiedLength,
        offset_y: CssSpecifiedLength,
        blur_radius: Option<CssSpecifiedNonNegativeLength>,
        spread_radius: Option<CssSpecifiedNonNegativeLength>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        let spread = match spread_radius {
            Some(value) => Some(if let Some(component) = value.literal_component() {
                CssSpecifiedLength::try_from_component(component.clone()).ok()?
            } else {
                CssSpecifiedLength::try_from_calculation(value.calculation()?.clone()).ok()?
            }),
            None => None,
        };
        Self::try_from_shadow(CssShadow::try_new(
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread,
            color,
        )?)
    }
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
            if let Some(component) = spread.literal_component() {
                CssSpecifiedNonNegativeLength::try_from_component(component.clone()).ok()?;
            } else {
                CssSpecifiedNonNegativeLength::try_from_calculation(spread.calculation()?.clone())
                    .ok()?;
            }
        }
        Some(Self { shadow })
    }
    pub const fn shadow(&self) -> &CssShadow {
        &self.shadow
    }
}
pub(crate) fn length_closed(value: &CssSpecifiedLength) -> bool {
    value
        .calculation()
        .is_none_or(|c| c.components().first_implicit_origin().is_none())
}
pub(crate) fn length_percentage_closed(value: &CssSpecifiedLengthPercentage) -> bool {
    value
        .calculation()
        .is_none_or(|c| c.components().first_implicit_origin().is_none())
}
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextShadowList {
    shadows: Vec<CssTextShadowLayer>,
}
impl CssTextShadowList {
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
    pub fn shadows(&self) -> &[CssTextShadowLayer] {
        &self.shadows
    }
}
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTextShadow {
    None,
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
provider!(CssTextEmphasis, value, writer => { writer.node()?; if let Some(style) = &value.style { style.append_to_rule_writer(writer)?; }
    if let Some(color) = &value.color { if value.style.is_some() { writer.append(" ")?; } color.append_specified(&mut writer.context, &mut writer.css)?; } Ok(()) });
provider!(CssTextEmphasisPosition, value, writer => { value.vertical.append_to_rule_writer(writer)?; if let Some(side) = value.side { if side == CssTextSide::Left { writer.append(" ")?; side.append_to_rule_writer(writer)?; } else { writer.context.charge_input(1)?; } } Ok(()) });
provider!(CssTextEmphasisSkip, value, writer => { let mut separated = false; for (present, word) in [(value.spaces,"spaces"),(value.punctuation,"punctuation"),(value.symbols,"symbols"),(value.narrow,"narrow")] { if present { if separated { writer.append(" ")?; } writer.keyword(word)?; separated = true; } } Ok(()) });
provider!(CssTextShadowLayer, value, writer => value.shadow.append_to_rule_writer(writer));
provider!(CssTextShadowList, value, writer => { writer.node()?; for (index, shadow) in value.shadows.iter().enumerate() { if index != 0 { writer.append(", ")?; } shadow.append_to_rule_writer(writer)?; } Ok(()) });
provider!(CssTextShadow, value, writer => match value { Self::None => writer.keyword("none"), Self::Shadows(value) => value.append_to_rule_writer(writer) });

#[cfg(test)]
mod provider_tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as K;
    use crate::CssSpecifiedValueSerializationLimits as L;

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
                    CssSpecifiedLengthPercentage::try_from_component(
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
            CssSpecifiedLength::try_from_component(CssComponentValue::try_token(text).unwrap())
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
