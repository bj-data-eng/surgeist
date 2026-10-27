use crate::numeric::{CssInteger, CssLengthPercentage, CssNumeric};

/// An authored `aspect-ratio` value: `auto`, a `<ratio>`, or both.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAspectRatioValue {
    /// `auto`: replaced elements use their natural aspect ratio and other boxes have none.
    Auto,
    /// A preferred `<ratio>` without the `auto` keyword.
    Ratio(crate::numeric::CssSpecifiedRatio),
    /// `auto <ratio>`: the natural aspect ratio when the box has one, otherwise the authored `<ratio>`.
    AutoRatio(crate::numeric::CssSpecifiedRatio),
}

/// # Specified serialization
///
/// Serializes the intrinsic specified keyword without resolving a box size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBoxSizing {
    /// `content-box`: width and height size the content box.
    ContentBox,
    /// `border-box`: width and height size the border box, including padding and border.
    BorderBox,
}

/// # Specified serialization
///
/// Serializes the canonical authored position keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLayoutPosition {
    /// `static`: the box is not a positioned box and is laid out by its parent formatting context; the inset properties do not apply.
    Static,
    /// `relative`: the box is laid out as for `static`, then offset from the resulting position.
    Relative,
    /// `absolute`: the box is taken out of flow and positioned and sized in reference to its absolute positioning containing block, as modified by its inset properties.
    Absolute,
    /// `fixed`: like `absolute`, but positioned and sized relative to a fixed positioning containing block, usually the viewport in continuous media or the page area in paged media.
    Fixed,
    /// `sticky`: like `relative`, but its offsets are adjusted in reference to the nearest ancestor scroll container's scrollport to keep the box in view within its containing block.
    Sticky,
}

/// # Specified serialization
///
/// Serializes the canonical authored overflow keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflow {
    /// `visible`: there is no special handling of overflow; content is rendered outside the box if positioned there, and the box is not a scroll container.
    Visible,
    /// `clip`: content is clipped to the overflow clip edge, and scrolling is forbidden through any mechanism.
    Clip,
    /// `hidden`: content is clipped to the padding box, no scrolling user interface is provided and direct user scrolling is not allowed, but the content remains scrollable programmatically.
    Hidden,
    /// `scroll`: content is clipped to the padding box and can be scrolled into view; a visible scrolling mechanism is displayed whether or not content is clipped.
    Scroll,
    /// `auto`: like `scroll` when the box has scrollable overflow, like `hidden` otherwise.
    Auto,
}

/// # Specified serialization
///
/// Serializes the specified float keyword without resolving the containing block's writing mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFloat {
    /// `left`: the box floats to the left side.
    Left,
    /// `right`: the box floats to the right side.
    Right,
    /// `none`: the box does not float.
    None,
    /// `inline-start`: the box floats to the inline-start side of its containing block.
    InlineStart,
    /// `inline-end`: the box floats to the inline-end side of its containing block.
    InlineEnd,
}

/// # Specified serialization
///
/// Serializes the specified clear keyword without resolving the containing block's writing mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssClear {
    /// `left`: the box is moved below preceding left floats.
    Left,
    /// `right`: the box is moved below preceding right floats.
    Right,
    /// `both`: the box is moved below preceding floats on both sides.
    Both,
    /// `none`: the box is not moved relative to floats.
    None,
    /// `inline-start`: the box is moved below preceding floats on the inline-start side.
    InlineStart,
    /// `inline-end`: the box is moved below preceding floats on the inline-end side.
    InlineEnd,
}

/// # Specified serialization
///
/// Serializes the specified visibility keyword without applying inheritance or changing box
/// rendering and layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVisibility {
    /// `visible`: the box is rendered.
    Visible,
    /// `hidden`: the box is invisible but still affects layout; descendants can be visible if they have `visibility: visible`.
    Hidden,
    /// `collapse`: the box is collapsed, which can take up less space in a formatting-context-specific way (table rows, columns and groups; flex items); in all other cases it makes the box invisible, like `hidden`.
    Collapse,
}

/// # Specified serialization
///
/// Serializes the specified content-visibility keyword without deciding whether contents are
/// skipped or which containment applies at use time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContentVisibility {
    /// `visible`: no effect; the element's contents are laid out and rendered as normal.
    Visible,
    /// `hidden`: the element skips its contents.
    Hidden,
    /// `auto`: turns on layout, style and paint containment, and the element skips its contents while it is not relevant to the user.
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
    /// `normal`: a used tolerance of `1em` in grid lanes layout and `0` in all other layout modes.
    Normal,
    /// `infinite`: an infinite tie threshold, so items are placed strictly in order without considering track lengths.
    Infinite,
    /// A borrowed signed `<length-percentage>` tolerance.
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
    /// `auto`: the stack level of the box in the current stacking context is 0, and the box does not establish a new stacking context unless it is the root element.
    Auto,
    /// An `<integer>` stack level, as a literal or an unresolved calculation.
    Integer(CssNumeric<CssInteger>),
}

/// # Specified serialization
///
/// Emits the canonical authored fragmentation-decoration keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBoxDecorationBreak {
    /// `slice`: the element is rendered as though it had no breaks and then sliced by them; no border or padding is inserted at a break.
    Slice,
    /// `clone`: each box fragment is independently wrapped with the border, padding and margin, and the background is drawn independently in each fragment.
    Clone,
}

/// One independently authored Containment 2 containment kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainComponent {
    /// `size`: size containment.
    Size,
    /// `layout`: layout containment.
    Layout,
    /// `style`: style containment.
    Style,
    /// `paint`: paint containment.
    Paint,
}

/// A nonempty, duplicate-free authored list of containment kinds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContainComponentList {
    components: Vec<CssContainComponent>,
}

impl CssContainComponentList {
    /// Builds a containment list from kinds in authored order.
    ///
    /// Returns `None` when `components` is empty or contains the same kind twice.
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

    /// Returns the containment kinds in authored order.
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
    /// `none`: no containment applies.
    None,
    /// `strict`: the authored shorthand for size, layout, style and paint containment.
    Strict,
    /// `content`: the authored shorthand for layout, style and paint containment.
    Content,
    /// An explicit list of containment kinds.
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
    /// `auto`: no forced break, and none is forbidden.
    Auto,
    /// `avoid`: avoid a break at this point.
    Avoid,
    /// `avoid-page`: avoid a page break at this point.
    AvoidPage,
    /// `page`: force a page break at this point.
    Page,
    /// `left`: force one or two page breaks so the next page is formatted as a left page.
    Left,
    /// `right`: force one or two page breaks so the next page is formatted as a right page.
    Right,
    /// `recto`: force one or two page breaks so the next page is formatted as either a left or a right page, whichever is second in a page spread according to the page progression.
    Recto,
    /// `verso`: force one or two page breaks so the next page is formatted as either a left or a right page, whichever is first in a page spread according to the page progression.
    Verso,
    /// `avoid-column`: avoid a column break at this point.
    AvoidColumn,
    /// `column`: force a column break at this point.
    Column,
    /// `avoid-region`: avoid a region break at this point.
    AvoidRegion,
    /// `region`: force a region break at this point.
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
    /// `auto`: impose no additional breaking constraints within the box.
    Auto,
    /// `avoid`: avoid breaks within the box.
    Avoid,
    /// `avoid-page`: avoid a page break inside the box.
    AvoidPage,
    /// `avoid-column`: avoid a column break inside the box.
    AvoidColumn,
    /// `avoid-region`: avoid a region break inside the box.
    AvoidRegion,
}
