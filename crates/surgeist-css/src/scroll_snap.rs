//! Authored Scroll Snap values, before writing-mode mapping and snap selection.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssSpecifiedLength, CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// The selected scroll-snap axis spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapAxis {
    X,
    Y,
    Block,
    Inline,
    Both,
}

impl CssScrollSnapAxis {
    const fn as_css(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Block => "block",
            Self::Inline => "inline",
            Self::Both => "both",
        }
    }
}

/// The optional scroll-snap strictness; omission means proximity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapStrictness {
    Mandatory,
    Proximity,
}

/// The authored scroll-snap-type value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapType {
    None,
    Axis {
        axis: CssScrollSnapAxis,
        strictness: Option<CssScrollSnapStrictness>,
    },
}

impl CssScrollSnapType {
    /// Constructs an axis value while retaining whether strictness was authored.
    #[must_use]
    pub const fn axis(
        axis: CssScrollSnapAxis,
        strictness: Option<CssScrollSnapStrictness>,
    ) -> Self {
        Self::Axis { axis, strictness }
    }

    /// Serializes canonical specified keywords without resolving the scroll axis.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under independent input, projection, and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        match self {
            Self::None => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(&mut output, "none")?;
            }
            Self::Axis { axis, strictness } => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(&mut output, axis.as_css())?;
                if let Some(strictness) = strictness {
                    context.charge_input(1)?;
                    context.charge_projection(1)?;
                    if *strictness == CssScrollSnapStrictness::Mandatory {
                        context.append(&mut output, " mandatory")?;
                    }
                }
            }
        }
        Ok(output)
    }
}

/// One alignment keyword, interpreted separately for block and inline axes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapAlignment {
    None,
    Start,
    End,
    Center,
}

impl CssScrollSnapAlignment {
    const fn as_css(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Start => "start",
            Self::End => "end",
            Self::Center => "center",
        }
    }
}

/// One or two authored scroll-snap-align keywords.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssScrollSnapAlign {
    block: CssScrollSnapAlignment,
    inline: Option<CssScrollSnapAlignment>,
}

impl CssScrollSnapAlign {
    /// Retains the optional second authored keyword.
    #[must_use]
    pub const fn new(
        block: CssScrollSnapAlignment,
        inline: Option<CssScrollSnapAlignment>,
    ) -> Self {
        Self { block, inline }
    }

    #[must_use]
    pub const fn block(&self) -> CssScrollSnapAlignment {
        self.block
    }

    /// Returns the authored inline keyword, absent when the first applies to both axes.
    #[must_use]
    pub const fn authored_inline(&self) -> Option<CssScrollSnapAlignment> {
        self.inline
    }

    #[must_use]
    pub const fn inline(&self) -> CssScrollSnapAlignment {
        match self.inline {
            Some(value) => value,
            None => self.block,
        }
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        context.charge_input(1)?;
        context.charge_projection(1)?;
        if self.inline.is_some() {
            context.charge_input(1)?;
            context.charge_projection(1)?;
        }
        let mut output = String::new();
        context.append(&mut output, self.block.as_css())?;
        if let Some(value) = self.inline.filter(|value| *value != self.block) {
            context.append(&mut output, " ")?;
            context.append(&mut output, value.as_css())?;
        }
        Ok(output)
    }
}

/// The authored scroll-snap-stop keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapStop {
    Normal,
    Always,
}

impl CssScrollSnapStop {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        crate::specified_serialization::serialize_keyword_sequence(
            match self {
                Self::Normal => "normal",
                Self::Always => "always",
            },
            limits,
        )
    }
}

/// One nonnegative scroll-padding side or its UA-dependent `auto` value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollPaddingValue {
    Auto,
    LengthPercentage(CssSpecifiedNonNegativeLengthPercentage),
}

impl CssScrollPaddingValue {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match self {
            Self::Auto => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                Ok("auto".to_owned())
            }
            Self::LengthPercentage(value) => value.capture_specified(context),
        }
    }
}

/// Whether four authored shorthand sides use physical or flow-relative roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSideKind {
    Physical,
    Logical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScrollPair<T> {
    start: T,
    end: Option<T>,
}

impl<T> ScrollPair<T> {
    const fn new(start: T, end: Option<T>) -> Self {
        Self { start, end }
    }

    const fn start(&self) -> &T {
        &self.start
    }

    const fn authored_end(&self) -> Option<&T> {
        self.end.as_ref()
    }

    const fn end(&self) -> &T {
        match self.end.as_ref() {
            Some(value) => value,
            None => &self.start,
        }
    }
}

/// One or two authored scroll-padding values for a logical axis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssScrollPaddingPair(ScrollPair<CssScrollPaddingValue>);

impl CssScrollPaddingPair {
    #[must_use]
    pub const fn new(start: CssScrollPaddingValue, end: Option<CssScrollPaddingValue>) -> Self {
        Self(ScrollPair::new(start, end))
    }

    #[must_use]
    pub const fn start(&self) -> &CssScrollPaddingValue {
        self.0.start()
    }

    #[must_use]
    pub const fn authored_end(&self) -> Option<&CssScrollPaddingValue> {
        self.0.authored_end()
    }

    #[must_use]
    pub const fn end(&self) -> &CssScrollPaddingValue {
        self.0.end()
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize_pair(
            self.start(),
            self.authored_end(),
            limits,
            |value, context| value.capture_specified(context),
            padding_equal,
        )
    }
}

/// One or two authored scroll-margin lengths for a logical axis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssScrollMarginPair(ScrollPair<CssSpecifiedLength>);

impl CssScrollMarginPair {
    #[must_use]
    pub const fn new(start: CssSpecifiedLength, end: Option<CssSpecifiedLength>) -> Self {
        Self(ScrollPair::new(start, end))
    }

    #[must_use]
    pub const fn start(&self) -> &CssSpecifiedLength {
        self.0.start()
    }

    #[must_use]
    pub const fn authored_end(&self) -> Option<&CssSpecifiedLength> {
        self.0.authored_end()
    }

    #[must_use]
    pub const fn end(&self) -> &CssSpecifiedLength {
        self.0.end()
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize_pair(
            self.start(),
            self.authored_end(),
            limits,
            |value, context| value.capture_specified(context),
            margin_equal,
        )
    }
}

fn length_equal(
    left: Option<&crate::CssComponentValue>,
    right: Option<&crate::CssComponentValue>,
    left_css: &str,
    right_css: &str,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            crate::specified_numeric::ordinary_length_literal_equal(left, right)
        }
        _ => left_css == right_css,
    }
}

fn padding_equal(
    left: &CssScrollPaddingValue,
    right: &CssScrollPaddingValue,
    left_css: &str,
    right_css: &str,
) -> bool {
    match (left, right) {
        (CssScrollPaddingValue::Auto, CssScrollPaddingValue::Auto) => true,
        (
            CssScrollPaddingValue::LengthPercentage(left),
            CssScrollPaddingValue::LengthPercentage(right),
        ) => length_equal(
            left.literal_component(),
            right.literal_component(),
            left_css,
            right_css,
        ),
        _ => false,
    }
}

fn margin_equal(
    left: &CssSpecifiedLength,
    right: &CssSpecifiedLength,
    left_css: &str,
    right_css: &str,
) -> bool {
    length_equal(
        left.literal_component(),
        right.literal_component(),
        left_css,
        right_css,
    )
}

fn serialize_pair<T>(
    start: &T,
    end: Option<&T>,
    limits: CssSpecifiedValueSerializationLimits,
    capture: impl Fn(&T, &mut SpecifiedSerializationContext) -> SerializationResult<String>,
    component_equal: impl Fn(&T, &T, &str, &str) -> bool,
) -> SerializationResult<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    let first = capture(start, &mut context)?;
    let second = end.map(|end| capture(end, &mut context)).transpose()?;
    let mut output = String::new();
    context.append(&mut output, &first)?;
    if let (Some(end), Some(second)) = (end, second)
        && !component_equal(start, end, &first, &second)
    {
        context.append(&mut output, " ")?;
        context.append(&mut output, &second)?;
    }
    Ok(output)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScrollFour<T> {
    kind: CssScrollSideKind,
    authored: Vec<T>,
}

impl<T> ScrollFour<T> {
    fn try_new(kind: CssScrollSideKind, authored: Vec<T>) -> Option<Self> {
        (1..=4)
            .contains(&authored.len())
            .then_some(Self { kind, authored })
    }

    fn role_index(&self, index: usize) -> usize {
        match (self.authored.len(), index) {
            (1, _) => 0,
            (2, 0 | 2) => 0,
            (2, 1 | 3) => 1,
            (3, 3) => 1,
            (_, index) => index,
        }
    }

    fn role(&self, index: usize) -> &T {
        &self.authored[self.role_index(index)]
    }
}

/// The authored one-to-four values and physical/logical role marker for `scroll-padding`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssScrollPaddingShorthand(ScrollFour<CssScrollPaddingValue>);

impl CssScrollPaddingShorthand {
    #[must_use]
    pub fn try_new(kind: CssScrollSideKind, authored: Vec<CssScrollPaddingValue>) -> Option<Self> {
        ScrollFour::try_new(kind, authored).map(Self)
    }

    #[must_use]
    pub const fn kind(&self) -> CssScrollSideKind {
        self.0.kind
    }

    #[must_use]
    pub fn authored_values(&self) -> &[CssScrollPaddingValue] {
        &self.0.authored
    }

    /// Borrows one of four roles in top/right/bottom/left or block-start/inline-start/block-end/inline-end order.
    #[must_use]
    pub fn role(&self, index: usize) -> Option<&CssScrollPaddingValue> {
        (index < 4).then(|| self.0.role(index))
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize_four(
            &self.0,
            limits,
            |value, context| value.capture_specified(context),
            padding_equal,
        )
    }
}

/// The authored one-to-four lengths and physical/logical role marker for `scroll-margin`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssScrollMarginShorthand(ScrollFour<CssSpecifiedLength>);

impl CssScrollMarginShorthand {
    #[must_use]
    pub fn try_new(kind: CssScrollSideKind, authored: Vec<CssSpecifiedLength>) -> Option<Self> {
        ScrollFour::try_new(kind, authored).map(Self)
    }

    #[must_use]
    pub const fn kind(&self) -> CssScrollSideKind {
        self.0.kind
    }

    #[must_use]
    pub fn authored_values(&self) -> &[CssSpecifiedLength] {
        &self.0.authored
    }

    /// Borrows one of four roles in top/right/bottom/left or block-start/inline-start/block-end/inline-end order.
    #[must_use]
    pub fn role(&self, index: usize) -> Option<&CssSpecifiedLength> {
        (index < 4).then(|| self.0.role(index))
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize_four(
            &self.0,
            limits,
            |value, context| value.capture_specified(context),
            margin_equal,
        )
    }
}

fn serialize_four<T>(
    values: &ScrollFour<T>,
    limits: CssSpecifiedValueSerializationLimits,
    capture: impl Fn(&T, &mut SpecifiedSerializationContext) -> SerializationResult<String>,
    component_equal: impl Fn(&T, &T, &str, &str) -> bool,
) -> SerializationResult<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    if values.kind == CssScrollSideKind::Logical {
        context.charge_input(1)?;
        context.charge_projection(1)?;
    }
    let authored = values
        .authored
        .iter()
        .map(|value| capture(value, &mut context))
        .collect::<SerializationResult<Vec<_>>>()?;
    let role = |index: usize| -> &str { &authored[values.role_index(index)] };
    let equal = |left: usize, right: usize| {
        component_equal(
            values.role(left),
            values.role(right),
            role(left),
            role(right),
        )
    };
    let (first, second, third, fourth) = (role(0), role(1), role(2), role(3));
    let selected: Vec<&str> = if equal(0, 1) && equal(0, 2) && equal(0, 3) {
        vec![first]
    } else if equal(0, 2) && equal(1, 3) {
        vec![first, second]
    } else if equal(1, 3) {
        vec![first, second, third]
    } else {
        vec![first, second, third, fourth]
    };
    let mut output = String::new();
    if values.kind == CssScrollSideKind::Logical {
        context.append(&mut output, "logical ")?;
    }
    for (index, value) in selected.iter().enumerate() {
        if index > 0 {
            context.append(&mut output, " ")?;
        }
        context.append(&mut output, value)?;
    }
    Ok(output)
}
