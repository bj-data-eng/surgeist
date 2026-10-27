use crate::numeric::{CssInteger, CssLengthPercentage, CssNumeric};

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAspectRatioValue {
    Auto,
    Ratio(crate::numeric::CssSpecifiedRatio),
    AutoRatio(crate::numeric::CssSpecifiedRatio),
}

/// # Specified serialization
///
/// Serializes the intrinsic specified keyword without resolving a box size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBoxSizing {
    ContentBox,
    BorderBox,
}

/// # Specified serialization
///
/// Serializes the canonical authored position keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLayoutPosition {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// # Specified serialization
///
/// Serializes the canonical authored overflow keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflow {
    Visible,
    Clip,
    Hidden,
    Scroll,
    Auto,
}

/// # Specified serialization
///
/// Serializes the specified float keyword without resolving the containing block's writing mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFloat {
    Left,
    Right,
    None,
    InlineStart,
    InlineEnd,
}

/// # Specified serialization
///
/// Serializes the specified clear keyword without resolving the containing block's writing mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssClear {
    Left,
    Right,
    Both,
    None,
    InlineStart,
    InlineEnd,
}

/// # Specified serialization
///
/// Serializes the specified visibility keyword without applying inheritance or changing box
/// rendering and layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVisibility {
    Visible,
    Hidden,
    Collapse,
}

/// # Specified serialization
///
/// Serializes the specified content-visibility keyword without deciding whether contents are
/// skipped or which containment applies at use time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContentVisibility {
    Visible,
    Hidden,
    Auto,
}

/// A checked authored `flow-tolerance`, before contextual used-value resolution.
///
/// Grid3 permits `normal`, `infinite`, or a signed length-percentage. `normal`
/// remains symbolic because its used value depends on the downstream layout mode.
///
/// # Specified serialization
///
/// Serializes canonical specified tolerance without resolving `normal`, relative lengths, or the
/// percentage basis. Exact ordinary magnitudes and supported symbolic math use the shared length-
/// percentage serializer.
///
/// # Specified serialization
///
/// Exact ordinary magnitudes and supported symbolic math use the shared length-percentage
/// serializer.
#[derive(Clone, Debug)]
pub struct CssFlowTolerance {
    value: FlowToleranceValue,
}

impl PartialEq for CssFlowTolerance {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (FlowToleranceValue::Normal, FlowToleranceValue::Normal)
            | (FlowToleranceValue::Infinite, FlowToleranceValue::Infinite) => true,
            (
                FlowToleranceValue::LengthPercentage(left),
                FlowToleranceValue::LengthPercentage(right),
            ) => left == right,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum FlowToleranceValue {
    Normal,
    Infinite,
    LengthPercentage(CssNumeric<CssLengthPercentage>),
}

/// Borrows the checked authored branch without resolving relative units or `normal`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFlowToleranceRef<'a> {
    Normal,
    Infinite,
    LengthPercentage(&'a CssNumeric<CssLengthPercentage>),
}

impl CssFlowTolerance {
    /// Constructs the symbolic initial value.
    #[must_use]
    pub const fn normal() -> Self {
        Self {
            value: FlowToleranceValue::Normal,
        }
    }

    /// Constructs the symbolic infinite placement tolerance.
    #[must_use]
    pub const fn infinite() -> Self {
        Self {
            value: FlowToleranceValue::Infinite,
        }
    }

    /// Composes a checked signed length-percentage without resolving it.
    ///
    /// The scalar owner excludes keywords and preserves its original origin.
    /// Signed operands and typed math remain symbolic until contextual resolution.
    #[must_use]
    pub fn length_percentage(value: CssNumeric<CssLengthPercentage>) -> Self {
        Self {
            value: FlowToleranceValue::LengthPercentage(value),
        }
    }

    /// Returns the checked authored branch and its unchanged numeric payload.
    #[must_use]
    pub const fn as_ref(&self) -> CssFlowToleranceRef<'_> {
        match &self.value {
            FlowToleranceValue::Normal => CssFlowToleranceRef::Normal,
            FlowToleranceValue::Infinite => CssFlowToleranceRef::Infinite,
            FlowToleranceValue::LengthPercentage(value) => {
                CssFlowToleranceRef::LengthPercentage(value)
            }
        }
    }
}

impl Default for CssFlowTolerance {
    fn default() -> Self {
        Self::normal()
    }
}

/// An authored `z-index` value before contextual stacking or integer rounding.
///
/// CSS2 §9.9.1 defines `auto` or a signed integer. Integer literals preserve
/// their exact component origins; calculations retain unresolved numeric input.
///
/// # Specified serialization
///
/// Produces canonical specified `auto` or integer text, retaining authored identity. Integer
/// literals retain exact magnitude. Math uses the shared specified projection without computed
/// integer rounding or contextual stacking policy.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssZIndexValue {
    Auto,
    Integer(CssNumeric<CssInteger>),
}

/// # Specified serialization
///
/// Emits the canonical authored fragmentation-decoration keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBoxDecorationBreak {
    Slice,
    Clone,
}

/// One independently authored Containment 2 containment kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainComponent {
    Size,
    Layout,
    Style,
    Paint,
}

/// A nonempty, duplicate-free authored list of containment kinds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContainComponentList {
    components: Vec<CssContainComponent>,
}

impl CssContainComponentList {
    #[must_use]
    pub fn try_new(components: Vec<CssContainComponent>) -> Option<Self> {
        if components.is_empty()
            || components
                .iter()
                .enumerate()
                .any(|(index, component)| components[..index].contains(component))
        {
            return None;
        }
        Some(Self { components })
    }

    #[must_use]
    pub fn components(&self) -> &[CssContainComponent] {
        &self.components
    }
}

/// The authored Containment 2 `contain` value.
///
/// This is syntax only and does not apply size, layout, style, or paint containment.
///
/// # Specified serialization
///
/// Emits Containment 2 syntax in size/layout/style/paint grammar order, preserving `strict` and
/// `content`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContain {
    None,
    Strict,
    Content,
    Components(CssContainComponentList),
}

/// The authored `break-before` or `break-after` keyword.
///
/// # Specified serialization
///
/// Serializes one canonical specified break keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBreakBetween {
    Auto,
    Avoid,
    AvoidPage,
    Page,
    Left,
    Right,
    Recto,
    Verso,
    AvoidColumn,
    Column,
    AvoidRegion,
    Region,
}

/// The authored `break-inside` keyword.
///
/// # Specified serialization
///
/// Serializes one canonical specified break-inside keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBreakInside {
    Auto,
    Avoid,
    AvoidPage,
    AvoidColumn,
    AvoidRegion,
}
