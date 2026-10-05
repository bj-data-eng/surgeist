//! Checked authored coordinate families and consumer-specific position restrictions.
use super::{CssSpecifiedLength, CssSpecifiedLengthPercentage, optional_numeric_eq};
use std::{error::Error, fmt};

/// An intrinsic failure to construct a checked authored position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPositionConstructionError {
    /// An edge-relative offset needs a matching offset on the other axis.
    UnpairedEdgeOffsets,
    /// A physical-only consumer cannot accept an axis-relative keyword.
    NonPhysicalKeyword,
}
impl fmt::Display for CssPositionConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnpairedEdgeOffsets => "position edge offsets must be paired",
            Self::NonPhysicalKeyword => "physical position cannot contain axis-relative keywords",
        })
    }
}
impl Error for CssPositionConstructionError {}

/// A keyword on the authored horizontal Cartesian axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHorizontalPositionKeyword {
    Left,
    Center,
    Right,
    XStart,
    XEnd,
}

/// The exact authored horizontal Cartesian axis, retaining scalar origins.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssHorizontalPosition {
    Left,
    Center,
    Right,
    XStart,
    XEnd,
    /// A free length-percentage from the axis start, without an edge keyword.
    Offset(CssSpecifiedLengthPercentage),
    LeftOffset(CssSpecifiedLengthPercentage),
    RightOffset(CssSpecifiedLengthPercentage),
    XStartOffset(CssSpecifiedLengthPercentage),
    XEndOffset(CssSpecifiedLengthPercentage),
}

impl PartialEq for CssHorizontalPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Offset(left), Self::Offset(right))
            | (Self::LeftOffset(left), Self::LeftOffset(right))
            | (Self::RightOffset(left), Self::RightOffset(right))
            | (Self::XStartOffset(left), Self::XStartOffset(right))
            | (Self::XEndOffset(left), Self::XEndOffset(right)) => left.structural_eq(right),
            (Self::Left, Self::Left)
            | (Self::Center, Self::Center)
            | (Self::Right, Self::Right)
            | (Self::XStart, Self::XStart)
            | (Self::XEnd, Self::XEnd) => true,
            _ => false,
        }
    }
}
impl CssHorizontalPosition {
    fn is_edge_offset(&self) -> bool {
        matches!(
            self,
            Self::LeftOffset(_)
                | Self::RightOffset(_)
                | Self::XStartOffset(_)
                | Self::XEndOffset(_)
        )
    }
    fn is_physical(&self) -> bool {
        !matches!(
            self,
            Self::XStart | Self::XEnd | Self::XStartOffset(_) | Self::XEndOffset(_)
        )
    }
}

/// A keyword on the authored vertical Cartesian axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVerticalPositionKeyword {
    Top,
    Center,
    Bottom,
    YStart,
    YEnd,
}

/// The exact authored vertical Cartesian axis, retaining scalar origins.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssVerticalPosition {
    Top,
    Center,
    Bottom,
    YStart,
    YEnd,
    /// A free length-percentage from the axis start, without an edge keyword.
    Offset(CssSpecifiedLengthPercentage),
    TopOffset(CssSpecifiedLengthPercentage),
    BottomOffset(CssSpecifiedLengthPercentage),
    YStartOffset(CssSpecifiedLengthPercentage),
    YEndOffset(CssSpecifiedLengthPercentage),
}

impl PartialEq for CssVerticalPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Offset(left), Self::Offset(right))
            | (Self::TopOffset(left), Self::TopOffset(right))
            | (Self::BottomOffset(left), Self::BottomOffset(right))
            | (Self::YStartOffset(left), Self::YStartOffset(right))
            | (Self::YEndOffset(left), Self::YEndOffset(right)) => left.structural_eq(right),
            (Self::Top, Self::Top)
            | (Self::Center, Self::Center)
            | (Self::Bottom, Self::Bottom)
            | (Self::YStart, Self::YStart)
            | (Self::YEnd, Self::YEnd) => true,
            _ => false,
        }
    }
}
impl CssVerticalPosition {
    fn is_edge_offset(&self) -> bool {
        matches!(
            self,
            Self::TopOffset(_)
                | Self::BottomOffset(_)
                | Self::YStartOffset(_)
                | Self::YEndOffset(_)
        )
    }
    fn is_physical(&self) -> bool {
        !matches!(
            self,
            Self::YStart | Self::YEnd | Self::YStartOffset(_) | Self::YEndOffset(_)
        )
    }
}

// Shared storage has no independent public construction or serialization cost.
#[derive(Clone, Debug, PartialEq)]
struct PositionAxes {
    horizontal: CssHorizontalPosition,
    vertical: CssVerticalPosition,
}
impl PositionAxes {
    fn is_physical(&self) -> bool {
        self.horizontal.is_physical() && self.vertical.is_physical()
    }
}
fn paired_offsets(block: bool, inline: bool) -> Result<(), CssPositionConstructionError> {
    if block == inline {
        Ok(())
    } else {
        Err(CssPositionConstructionError::UnpairedEdgeOffsets)
    }
}

/// A checked Cartesian position with physical or x/y-relative axes.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCartesianPosition {
    axes: PositionAxes,
}
impl CssCartesianPosition {
    /// Checks the generic one/two/four-component arity: edge offsets must be paired.
    pub fn try_new(
        horizontal: CssHorizontalPosition,
        vertical: CssVerticalPosition,
    ) -> Result<Self, CssPositionConstructionError> {
        paired_offsets(horizontal.is_edge_offset(), vertical.is_edge_offset())?;
        Ok(Self {
            axes: PositionAxes {
                horizontal,
                vertical,
            },
        })
    }
    /// Returns the symbolic horizontal axis and its authored offset origin.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        &self.axes.horizontal
    }
    /// Returns the symbolic vertical axis and its authored offset origin.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        &self.axes.vertical
    }
}

/// A checked physical-only generic position for older selected consumer grammars.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPhysicalPosition {
    cartesian: CssCartesianPosition,
}
impl CssPhysicalPosition {
    /// Checks generic arity and rejects every x/y-relative keyword or edge offset.
    pub fn try_new(
        horizontal: CssHorizontalPosition,
        vertical: CssVerticalPosition,
    ) -> Result<Self, CssPositionConstructionError> {
        Self::try_from_cartesian(CssCartesianPosition::try_new(horizontal, vertical)?)
    }
    pub(crate) fn try_from_cartesian(
        cartesian: CssCartesianPosition,
    ) -> Result<Self, CssPositionConstructionError> {
        if !cartesian.axes.is_physical() {
            return Err(CssPositionConstructionError::NonPhysicalKeyword);
        }
        Ok(Self { cartesian })
    }
    /// Returns the authored physical horizontal axis.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        self.cartesian.horizontal()
    }
    /// Returns the authored physical vertical axis.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        self.cartesian.vertical()
    }
}

/// One authored named block component; center cannot be an offset origin.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBlockPosition {
    Center,
    Start,
    End,
    StartOffset(CssSpecifiedLengthPercentage),
    EndOffset(CssSpecifiedLengthPercentage),
}
impl PartialEq for CssBlockPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StartOffset(left), Self::StartOffset(right))
            | (Self::EndOffset(left), Self::EndOffset(right)) => left.structural_eq(right),
            (Self::Center, Self::Center) | (Self::Start, Self::Start) | (Self::End, Self::End) => {
                true
            }
            _ => false,
        }
    }
}
impl CssBlockPosition {
    fn is_edge_offset(&self) -> bool {
        matches!(self, Self::StartOffset(_) | Self::EndOffset(_))
    }
}

/// One authored named inline component; center cannot be an offset origin.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssInlinePosition {
    Center,
    Start,
    End,
    StartOffset(CssSpecifiedLengthPercentage),
    EndOffset(CssSpecifiedLengthPercentage),
}
impl PartialEq for CssInlinePosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StartOffset(left), Self::StartOffset(right))
            | (Self::EndOffset(left), Self::EndOffset(right)) => left.structural_eq(right),
            (Self::Center, Self::Center) | (Self::Start, Self::Start) | (Self::End, Self::End) => {
                true
            }
            _ => false,
        }
    }
}
impl CssInlinePosition {
    fn is_edge_offset(&self) -> bool {
        matches!(self, Self::StartOffset(_) | Self::EndOffset(_))
    }
}

/// One authored relative flow component; center cannot be an offset origin.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssRelativeAxisPosition {
    Center,
    Start,
    End,
    StartOffset(CssSpecifiedLengthPercentage),
    EndOffset(CssSpecifiedLengthPercentage),
}
impl PartialEq for CssRelativeAxisPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StartOffset(left), Self::StartOffset(right))
            | (Self::EndOffset(left), Self::EndOffset(right)) => left.structural_eq(right),
            (Self::Center, Self::Center) | (Self::Start, Self::Start) | (Self::End, Self::End) => {
                true
            }
            _ => false,
        }
    }
}
impl CssRelativeAxisPosition {
    fn is_edge_offset(&self) -> bool {
        matches!(self, Self::StartOffset(_) | Self::EndOffset(_))
    }
}

/// Checked symbolic flow axes, created only through the full position owner.
#[derive(Clone, Debug, PartialEq)]
pub struct CssNamedFlowPosition {
    block: CssBlockPosition,
    inline: CssInlinePosition,
}
impl CssNamedFlowPosition {
    /// Returns the symbolic block axis and retained scalar origin.
    #[must_use]
    pub const fn block(&self) -> &CssBlockPosition {
        &self.block
    }
    /// Returns the symbolic inline axis and retained scalar origin.
    #[must_use]
    pub const fn inline(&self) -> &CssInlinePosition {
        &self.inline
    }
}

/// Checked symbolic flow axes, created only through the full position owner.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeFlowPosition {
    block: CssRelativeAxisPosition,
    inline: CssRelativeAxisPosition,
}
impl CssRelativeFlowPosition {
    /// Returns the symbolic block axis and retained scalar origin.
    #[must_use]
    pub const fn block(&self) -> &CssRelativeAxisPosition {
        &self.block
    }
    /// Returns the symbolic inline axis and retained scalar origin.
    #[must_use]
    pub const fn inline(&self) -> &CssRelativeAxisPosition {
        &self.inline
    }
}

#[derive(Clone, Debug, PartialEq)]
enum PositionFamily {
    Cartesian(CssCartesianPosition),
    NamedFlow(CssNamedFlowPosition),
    RelativeFlow(CssRelativeFlowPosition),
}

/// The borrowed coordinate family of a checked authored position.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPositionRef<'a> {
    Cartesian(&'a CssCartesianPosition),
    NamedFlow(&'a CssNamedFlowPosition),
    RelativeFlow(&'a CssRelativeFlowPosition),
}

/// A checked Values 5 position, preserving unresolved coordinate families.
///
/// Every owning construction path normalizes all-center flow axes to Cartesian
/// center/center. Noncentral flow families retain their authored meaning.
///
/// ```
/// use surgeist_css::{CssBlockPosition, CssInlinePosition, CssPosition, CssPositionRef};
/// let position = CssPosition::try_from_named_axes(
///     CssBlockPosition::Start, CssInlinePosition::End,
/// ).unwrap();
/// let CssPositionRef::NamedFlow(axes) = position.view() else { panic!("named axes") };
/// assert_eq!(axes.block(), &CssBlockPosition::Start);
/// assert_eq!(axes.inline(), &CssInlinePosition::End);
/// assert_eq!(position.serialize_specified().unwrap(), "block-start inline-end");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssPosition {
    family: PositionFamily,
}
impl CssPosition {
    /// Lifts an already checked Cartesian position without changing its axes.
    #[must_use]
    pub const fn from_cartesian(position: CssCartesianPosition) -> Self {
        Self {
            family: PositionFamily::Cartesian(position),
        }
    }
    /// Checks paired named-flow edge offsets and canonicalizes center/center.
    pub fn try_from_named_axes(
        block: CssBlockPosition,
        inline: CssInlinePosition,
    ) -> Result<Self, CssPositionConstructionError> {
        paired_offsets(block.is_edge_offset(), inline.is_edge_offset())?;
        if matches!(block, CssBlockPosition::Center) && matches!(inline, CssInlinePosition::Center)
        {
            return Ok(Self::center());
        }
        Ok(Self {
            family: PositionFamily::NamedFlow(CssNamedFlowPosition { block, inline }),
        })
    }
    /// Checks paired relative-flow edge offsets in block/inline order.
    pub fn try_from_relative_axes(
        block: CssRelativeAxisPosition,
        inline: CssRelativeAxisPosition,
    ) -> Result<Self, CssPositionConstructionError> {
        paired_offsets(block.is_edge_offset(), inline.is_edge_offset())?;
        if matches!(block, CssRelativeAxisPosition::Center)
            && matches!(inline, CssRelativeAxisPosition::Center)
        {
            return Ok(Self::center());
        }
        Ok(Self {
            family: PositionFamily::RelativeFlow(CssRelativeFlowPosition { block, inline }),
        })
    }
    const fn center() -> Self {
        Self::from_cartesian(CssCartesianPosition {
            axes: PositionAxes {
                horizontal: CssHorizontalPosition::Center,
                vertical: CssVerticalPosition::Center,
            },
        })
    }
    /// Observes the checked coordinate family without exposing owning construction.
    #[must_use]
    pub const fn view(&self) -> CssPositionRef<'_> {
        match &self.family {
            PositionFamily::Cartesian(value) => CssPositionRef::Cartesian(value),
            PositionFamily::NamedFlow(value) => CssPositionRef::NamedFlow(value),
            PositionFamily::RelativeFlow(value) => CssPositionRef::RelativeFlow(value),
        }
    }
}
impl From<CssPhysicalPosition> for CssPosition {
    fn from(position: CssPhysicalPosition) -> Self {
        Self::from_cartesian(position.cartesian)
    }
}

/// A nonempty authored comma list of checked physical positions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPhysicalPositionList {
    positions: Vec<CssPhysicalPosition>,
}
impl CssPhysicalPositionList {
    /// Constructs a nonempty list of checked physical positions.
    #[must_use]
    pub fn try_new(positions: Vec<CssPhysicalPosition>) -> Option<Self> {
        (!positions.is_empty()).then_some(Self { positions })
    }
    /// Returns positions in authored comma order.
    #[must_use]
    pub fn positions(&self) -> &[CssPhysicalPosition] {
        &self.positions
    }
}

/// One checked physical layer of background-position's one/two/three/four grammar.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundPosition {
    axes: PositionAxes,
}
impl CssBackgroundPosition {
    /// Checks background arity directly, including a lone edge offset beside a keyword.
    #[must_use]
    pub fn try_new(
        horizontal: CssHorizontalPosition,
        vertical: CssVerticalPosition,
    ) -> Option<Self> {
        let axes = PositionAxes {
            horizontal,
            vertical,
        };
        if !axes.is_physical()
            || axes.horizontal.is_edge_offset()
                && matches!(axes.vertical, CssVerticalPosition::Offset(_))
            || axes.vertical.is_edge_offset()
                && matches!(axes.horizontal, CssHorizontalPosition::Offset(_))
        {
            return None;
        }
        Some(Self { axes })
    }
    /// Returns the authored horizontal axis and edge origin.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        &self.axes.horizontal
    }
    /// Returns the authored vertical axis and edge origin.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        &self.axes.vertical
    }
}

/// A nonempty authored comma list of `background-position` layers.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundPositionList {
    positions: Vec<CssBackgroundPosition>,
}

impl CssBackgroundPositionList {
    /// Constructs a nonempty list from already validated background-position layers.
    #[must_use]
    pub fn try_new(positions: Vec<CssBackgroundPosition>) -> Option<Self> {
        if positions.is_empty() {
            None
        } else {
            Some(Self::new(positions))
        }
    }

    #[must_use]
    pub(crate) const fn new(positions: Vec<CssBackgroundPosition>) -> Self {
        Self { positions }
    }

    /// Returns the authored layers in comma order.
    #[must_use]
    pub fn positions(&self) -> &[CssBackgroundPosition] {
        &self.positions
    }
}

/// A physical planar transform origin plus an optional authored pure-length Z.
#[derive(Clone, Debug)]
pub struct CssTransformOrigin {
    position: CssPhysicalPosition,
    z: Option<CssSpecifiedLength>,
}
numeric_fields_eq!(CssTransformOrigin, [], [z], [position]);
impl CssTransformOrigin {
    /// Borrows the checked planar owner without exposing another public getter.
    pub(crate) fn append_planar_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), crate::CssSpecifiedValueSerializationError> {
        self.position.append_to_rule_writer(writer)
    }

    /// Rejects planar edge offsets while retaining the optional checked Z length.
    #[must_use]
    pub fn try_new(position: CssPhysicalPosition, z: Option<CssSpecifiedLength>) -> Option<Self> {
        if position.horizontal().is_edge_offset() || position.vertical().is_edge_offset() {
            return None;
        }
        Some(Self { position, z })
    }
    /// Returns the authored horizontal planar axis.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        self.position.horizontal()
    }
    /// Returns the authored vertical planar axis.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        self.position.vertical()
    }
    /// Returns the optional checked authored Z length.
    #[must_use]
    pub const fn z(&self) -> Option<&CssSpecifiedLength> {
        self.z.as_ref()
    }
}
