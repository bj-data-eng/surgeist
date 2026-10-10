use super::CssGridLineName;
use crate::{CssSpecifiedNonNegativeFlex, CssSpecifiedNonNegativeLengthPercentage};

pub(crate) fn adjacent_line_names<T>(
    previous: Option<&T>,
    next: &T,
    is_line_names: impl Fn(&T) -> bool,
) -> bool {
    previous.is_some_and(|previous| is_line_names(previous) && is_line_names(next))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridLineNames {
    names: Vec<CssGridLineName>,
}

impl CssGridLineNames {
    /// Creates an authored line-name group, including the valid empty group `[]`.
    #[must_use]
    pub fn new(names: Vec<CssGridLineName>) -> Self {
        Self { names }
    }

    #[must_use]
    pub fn names(&self) -> &[CssGridLineName] {
        &self.names
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridAutoFlowAxis {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// An explicit Grid auto-flow axis and dense selection.
pub struct CssGridAutoFlowMode {
    axis: CssGridAutoFlowAxis,
    dense: bool,
}

impl CssGridAutoFlowMode {
    #[must_use]
    pub const fn new(axis: CssGridAutoFlowAxis, dense: bool) -> Self {
        Self { axis, dense }
    }

    #[must_use]
    pub const fn axis(self) -> CssGridAutoFlowAxis {
        self.axis
    }

    #[must_use]
    pub const fn dense(self) -> bool {
        self.dense
    }
}

/// The six authored `grid-auto-flow` states. An unspecified direction remains
/// symbolic for the layout layer; in particular, bare `dense` is not `row dense`.
///
/// ```
/// use surgeist_css::{CssGridAutoFlowAxis, CssGridAutoFlow};
///
/// let dense = CssGridAutoFlow::Dense;
/// let row_dense = CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, true);
/// assert_ne!(dense, row_dense);
/// assert_eq!(dense.serialize_specified().unwrap(), "dense");
/// assert_eq!(row_dense.serialize_specified().unwrap(), "row dense");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridAutoFlow {
    Normal,
    Dense,
    ExplicitAxis(CssGridAutoFlowMode),
}

impl CssGridAutoFlow {
    /// Constructs an explicit row or column choice, with optional dense packing.
    #[must_use]
    pub const fn explicit_axis(axis: CssGridAutoFlowAxis, dense: bool) -> Self {
        Self::ExplicitAxis(CssGridAutoFlowMode::new(axis, dense))
    }

    /// Emits the canonical authored spelling without resolving the direction.
    pub fn serialize_specified(self) -> Result<String, crate::CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(crate::CssSpecifiedValueSerializationLimits::default())
    }

    /// Emits canonical text under cumulative input, projection, and byte limits.
    pub fn serialize_specified_with_limits(
        self,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<String, crate::CssSpecifiedValueSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let (text, nodes) = match self {
            Self::Normal => ("normal", 1),
            Self::Dense => ("dense", 1),
            Self::ExplicitAxis(value) => match (value.axis(), value.dense()) {
                (CssGridAutoFlowAxis::Row, false) => ("row", 1),
                (CssGridAutoFlowAxis::Row, true) => ("row dense", 2),
                (CssGridAutoFlowAxis::Column, false) => ("column", 1),
                (CssGridAutoFlowAxis::Column, true) => ("column dense", 2),
            },
        };
        let context = &mut writer.context;
        context.charge_input(nodes)?;
        context.charge_projection(nodes)?;
        context.append(&mut writer.css, text)
    }
}

/// The semantic branch of an authored Grid track breadth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackBreadthKind {
    Length,
    Fraction,
    MinContent,
    MaxContent,
    Auto,
}

/// A checked authored Grid track breadth.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTrackBreadth {
    representation: CssGridTrackBreadthRepresentation,
}

#[derive(Clone, Debug, PartialEq)]
enum CssGridTrackBreadthRepresentation {
    Length(CssSpecifiedNonNegativeLengthPercentage),
    Fraction(CssSpecifiedNonNegativeFlex),
    MinContent,
    MaxContent,
    Auto,
}

impl CssGridTrackBreadth {
    pub fn from_length_percentage(value: CssSpecifiedNonNegativeLengthPercentage) -> Self {
        Self {
            representation: CssGridTrackBreadthRepresentation::Length(value),
        }
    }
    pub fn from_flex(value: CssSpecifiedNonNegativeFlex) -> Self {
        Self {
            representation: CssGridTrackBreadthRepresentation::Fraction(value),
        }
    }

    pub const fn min_content() -> Self {
        Self {
            representation: CssGridTrackBreadthRepresentation::MinContent,
        }
    }

    pub const fn max_content() -> Self {
        Self {
            representation: CssGridTrackBreadthRepresentation::MaxContent,
        }
    }

    pub const fn auto() -> Self {
        Self {
            representation: CssGridTrackBreadthRepresentation::Auto,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssGridTrackBreadthKind {
        match self.representation {
            CssGridTrackBreadthRepresentation::Length(_) => CssGridTrackBreadthKind::Length,
            CssGridTrackBreadthRepresentation::Fraction(_) => CssGridTrackBreadthKind::Fraction,
            CssGridTrackBreadthRepresentation::MinContent => CssGridTrackBreadthKind::MinContent,
            CssGridTrackBreadthRepresentation::MaxContent => CssGridTrackBreadthKind::MaxContent,
            CssGridTrackBreadthRepresentation::Auto => CssGridTrackBreadthKind::Auto,
        }
    }

    pub fn length_percentage(&self) -> Option<&CssSpecifiedNonNegativeLengthPercentage> {
        match &self.representation {
            CssGridTrackBreadthRepresentation::Length(specified) => Some(specified),
            _ => None,
        }
    }

    pub fn flex(&self) -> Option<&CssSpecifiedNonNegativeFlex> {
        match &self.representation {
            CssGridTrackBreadthRepresentation::Fraction(specified) => Some(specified),
            _ => None,
        }
    }

    pub(crate) const fn is_fixed(&self) -> bool {
        matches!(
            self.representation,
            CssGridTrackBreadthRepresentation::Length(_)
        )
    }

    pub(crate) const fn is_inflexible(&self) -> bool {
        !matches!(
            self.representation,
            CssGridTrackBreadthRepresentation::Fraction(_)
        )
    }
}

/// The semantic branch of an authored Grid track size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackSizeKind {
    Breadth,
    MinMax,
    FitContent,
}

/// A checked authored Grid track size.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTrackSize {
    representation: CssGridTrackSizeRepresentation,
}

#[derive(Clone, Debug, PartialEq)]
enum CssGridTrackSizeRepresentation {
    Breadth(CssGridTrackBreadth),
    MinMax {
        min: CssGridTrackBreadth,
        max: CssGridTrackBreadth,
    },
    FitContent(CssSpecifiedNonNegativeLengthPercentage),
}

impl CssGridTrackSize {
    pub const fn from_breadth(value: CssGridTrackBreadth) -> Self {
        Self {
            representation: CssGridTrackSizeRepresentation::Breadth(value),
        }
    }

    pub(crate) const fn from_minmax(min: CssGridTrackBreadth, max: CssGridTrackBreadth) -> Self {
        Self {
            representation: CssGridTrackSizeRepresentation::MinMax { min, max },
        }
    }

    pub fn try_minmax(min: CssGridTrackBreadth, max: CssGridTrackBreadth) -> Option<Self> {
        if min.is_inflexible() {
            Some(Self::from_minmax(min, max))
        } else {
            None
        }
    }

    pub fn from_fit_content(value: CssSpecifiedNonNegativeLengthPercentage) -> Self {
        Self {
            representation: CssGridTrackSizeRepresentation::FitContent(value),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssGridTrackSizeKind {
        match self.representation {
            CssGridTrackSizeRepresentation::Breadth(_) => CssGridTrackSizeKind::Breadth,
            CssGridTrackSizeRepresentation::MinMax { .. } => CssGridTrackSizeKind::MinMax,
            CssGridTrackSizeRepresentation::FitContent(_) => CssGridTrackSizeKind::FitContent,
        }
    }

    #[must_use]
    pub const fn breadth(&self) -> Option<&CssGridTrackBreadth> {
        match &self.representation {
            CssGridTrackSizeRepresentation::Breadth(value) => Some(value),
            CssGridTrackSizeRepresentation::MinMax { .. }
            | CssGridTrackSizeRepresentation::FitContent(_) => None,
        }
    }

    #[must_use]
    pub const fn minmax(&self) -> Option<(&CssGridTrackBreadth, &CssGridTrackBreadth)> {
        match &self.representation {
            CssGridTrackSizeRepresentation::MinMax { min, max } => Some((min, max)),
            CssGridTrackSizeRepresentation::Breadth(_)
            | CssGridTrackSizeRepresentation::FitContent(_) => None,
        }
    }

    pub fn fit_content(&self) -> Option<&CssSpecifiedNonNegativeLengthPercentage> {
        match &self.representation {
            CssGridTrackSizeRepresentation::FitContent(specified) => Some(specified),
            _ => None,
        }
    }

    pub(crate) const fn is_fixed(&self) -> bool {
        match &self.representation {
            CssGridTrackSizeRepresentation::Breadth(value) => value.is_fixed(),
            CssGridTrackSizeRepresentation::MinMax { min, max } => {
                min.is_fixed() || (min.is_inflexible() && max.is_fixed())
            }
            CssGridTrackSizeRepresentation::FitContent(_) => false,
        }
    }
}

/// A fixed size admitted around automatic repetition or in an integer fixed-repeat.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridFixedSize {
    size: CssGridTrackSize,
}

impl CssGridFixedSize {
    pub fn try_new(size: CssGridTrackSize) -> Option<Self> {
        if size.is_fixed() {
            Some(Self::new(size))
        } else {
            None
        }
    }

    pub(crate) const fn new(size: CssGridTrackSize) -> Self {
        Self { size }
    }

    #[must_use]
    pub const fn size(&self) -> &CssGridTrackSize {
        &self.size
    }
}

/// One non-recursive member of integer or automatic track-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackRepeatComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssGridTrackSize),
}

/// A nonempty repeat-free sequence for repeat bodies or explicit area columns.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTrackRepeatContent {
    components: Vec<CssGridTrackRepeatComponent>,
}

impl CssGridTrackRepeatContent {
    pub fn try_new(components: Vec<CssGridTrackRepeatComponent>) -> Option<Self> {
        (components
            .iter()
            .any(|item| matches!(item, CssGridTrackRepeatComponent::TrackSize(_)))
            && !components.windows(2).any(|pair| {
                adjacent_line_names(Some(&pair[0]), &pair[1], |item| {
                    matches!(item, CssGridTrackRepeatComponent::LineNames(_))
                })
            }))
        .then(|| Self::new(components))
    }

    pub(crate) const fn new(components: Vec<CssGridTrackRepeatComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssGridTrackRepeatComponent] {
        &self.components
    }
}

/// One non-recursive member of fixed-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridFixedRepeatComponent {
    LineNames(CssGridLineNames),
    FixedSize(CssGridFixedSize),
}

/// Non-empty, non-recursive fixed-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridFixedRepeatContent {
    components: Vec<CssGridFixedRepeatComponent>,
}

impl CssGridFixedRepeatContent {
    pub fn try_new(components: Vec<CssGridFixedRepeatComponent>) -> Option<Self> {
        (components
            .iter()
            .any(|item| matches!(item, CssGridFixedRepeatComponent::FixedSize(_)))
            && !components.windows(2).any(|pair| {
                adjacent_line_names(Some(&pair[0]), &pair[1], |item| {
                    matches!(item, CssGridFixedRepeatComponent::LineNames(_))
                })
            }))
        .then(|| Self::new(components))
    }

    pub(crate) const fn new(components: Vec<CssGridFixedRepeatComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssGridFixedRepeatComponent] {
        &self.components
    }
}

/// A positive-integer repetition whose content may use any track size.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridIntegerTrackRepeat {
    count: crate::CssPositiveIntegerValue,
    content: CssGridTrackRepeatContent,
}

impl CssGridIntegerTrackRepeat {
    /// Checks ordinary positive counts while retaining function math symbolically.
    /// Typed child recovery remains observable; this does not certify original CSS.
    pub fn try_new(
        count: crate::CssPositiveIntegerValue,
        content: CssGridTrackRepeatContent,
    ) -> Option<Self> {
        Some(Self {
            count: count.normalized_positive_root()?,
            content,
        })
    }

    #[must_use]
    pub const fn count(&self) -> &crate::CssPositiveIntegerValue {
        &self.count
    }

    #[must_use]
    pub const fn content(&self) -> &CssGridTrackRepeatContent {
        &self.content
    }
}

/// A positive-integer repetition constrained to fixed-size content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridIntegerFixedRepeat {
    count: crate::CssPositiveIntegerValue,
    content: CssGridFixedRepeatContent,
}

impl CssGridIntegerFixedRepeat {
    /// Checks ordinary positive counts while retaining function math symbolically.
    pub fn try_new(
        count: crate::CssPositiveIntegerValue,
        content: CssGridFixedRepeatContent,
    ) -> Option<Self> {
        Some(Self {
            count: count.normalized_positive_root()?,
            content,
        })
    }

    #[must_use]
    pub const fn count(&self) -> &crate::CssPositiveIntegerValue {
        &self.count
    }

    #[must_use]
    pub const fn content(&self) -> &CssGridFixedRepeatContent {
        &self.content
    }
}

/// The automatic repetition mode in an authored Grid track list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridAutoRepeatKind {
    AutoFill,
    AutoFit,
}

/// The single automatic repetition admitted by an auto track list.
///
/// Its non-recursive body admits general track sizes under CSS Grid 3. Surrounding
/// tracks and integer repetitions in the same list remain constrained to fixed sizes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridAutoRepeat {
    kind: CssGridAutoRepeatKind,
    content: CssGridTrackRepeatContent,
}

impl CssGridAutoRepeat {
    pub const fn new(kind: CssGridAutoRepeatKind, content: CssGridTrackRepeatContent) -> Self {
        Self { kind, content }
    }

    #[must_use]
    pub const fn kind(&self) -> CssGridAutoRepeatKind {
        self.kind
    }

    #[must_use]
    pub const fn content(&self) -> &CssGridTrackRepeatContent {
        &self.content
    }
}

/// One component of a general track list, which never contains automatic repetition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridGeneralTrackComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssGridTrackSize),
    Repeat(CssGridIntegerTrackRepeat),
}

/// A non-empty general Grid track list.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridGeneralTrackList {
    components: Vec<CssGridGeneralTrackComponent>,
}

impl CssGridGeneralTrackList {
    pub fn try_new(components: Vec<CssGridGeneralTrackComponent>) -> Option<Self> {
        (components
            .iter()
            .any(|item| !matches!(item, CssGridGeneralTrackComponent::LineNames(_)))
            && !components.windows(2).any(|pair| {
                adjacent_line_names(Some(&pair[0]), &pair[1], |item| {
                    matches!(item, CssGridGeneralTrackComponent::LineNames(_))
                })
            }))
        .then(|| Self::new(components))
    }

    pub(crate) const fn new(components: Vec<CssGridGeneralTrackComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssGridGeneralTrackComponent] {
        &self.components
    }
}

/// One component of an auto track list.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridAutoTrackComponent {
    LineNames(CssGridLineNames),
    FixedSize(CssGridFixedSize),
    Repeat(CssGridIntegerFixedRepeat),
    AutoRepeat(CssGridAutoRepeat),
}

/// A Grid track list containing exactly one automatic repetition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridAutoTrackList {
    components: Vec<CssGridAutoTrackComponent>,
}

impl CssGridAutoTrackList {
    pub fn try_new(components: Vec<CssGridAutoTrackComponent>) -> Option<Self> {
        (components
            .iter()
            .filter(|item| matches!(item, CssGridAutoTrackComponent::AutoRepeat(_)))
            .count()
            == 1
            && !components.windows(2).any(|pair| {
                adjacent_line_names(Some(&pair[0]), &pair[1], |item| {
                    matches!(item, CssGridAutoTrackComponent::LineNames(_))
                })
            }))
        .then(|| Self::new(components))
    }

    pub(crate) const fn new(components: Vec<CssGridAutoTrackComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssGridAutoTrackComponent] {
        &self.components
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssGridTrackListRepresentation {
    None,
    General(CssGridGeneralTrackList),
    Auto(CssGridAutoTrackList),
    Subgrid(Vec<CssGridSubgridComponent>),
}

/// An authored Grid axis: none, sized tracks, or subgrid line names.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTrackList {
    representation: CssGridTrackListRepresentation,
}

impl CssGridTrackList {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            representation: CssGridTrackListRepresentation::None,
        }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self.representation, CssGridTrackListRepresentation::None)
    }

    /// Constructs a subgrid line-name list, including bare `subgrid`.
    /// At most one automatic name repetition is permitted across the list.
    pub fn try_subgrid(components: Vec<CssGridSubgridComponent>) -> Option<Self> {
        (components
            .iter()
            .filter(|component| {
                matches!(component,
            CssGridSubgridComponent::Repeat(value) if value.is_auto_fill())
            })
            .count()
            <= 1)
            .then_some(Self {
                representation: CssGridTrackListRepresentation::Subgrid(components),
            })
    }

    #[must_use]
    pub fn subgrid_components(&self) -> Option<&[CssGridSubgridComponent]> {
        match &self.representation {
            CssGridTrackListRepresentation::Subgrid(components) => Some(components),
            _ => None,
        }
    }

    pub const fn general(value: CssGridGeneralTrackList) -> Self {
        Self {
            representation: CssGridTrackListRepresentation::General(value),
        }
    }

    pub const fn auto(value: CssGridAutoTrackList) -> Self {
        Self {
            representation: CssGridTrackListRepresentation::Auto(value),
        }
    }

    #[must_use]
    pub const fn general_list(&self) -> Option<&CssGridGeneralTrackList> {
        match &self.representation {
            CssGridTrackListRepresentation::General(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn auto_list(&self) -> Option<&CssGridAutoTrackList> {
        match &self.representation {
            CssGridTrackListRepresentation::Auto(value) => Some(value),
            _ => None,
        }
    }
}

/// One group or non-recursive name repetition in a subgrid axis.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridSubgridComponent {
    LineNames(CssGridLineNames),
    Repeat(CssGridNameRepeat),
}

#[derive(Clone, Debug, PartialEq)]
enum CssGridNameRepeatCount {
    Counted(crate::CssPositiveIntegerValue),
    AutoFill,
}

/// A nonempty sequence of line-name groups repeated symbolically.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridNameRepeat {
    count: CssGridNameRepeatCount,
    groups: Vec<CssGridLineNames>,
}

impl CssGridNameRepeat {
    /// Checks a positive ordinary count and at least one group, including `[]`.
    /// Function math retains its exact graph and deferred count range/rounding.
    pub fn try_new(
        count: crate::CssPositiveIntegerValue,
        groups: Vec<CssGridLineNames>,
    ) -> Option<Self> {
        let count = count.normalized_positive_root()?;
        (!groups.is_empty()).then_some(Self {
            count: CssGridNameRepeatCount::Counted(count),
            groups,
        })
    }

    /// Constructs automatic name repetition with at least one authored group.
    pub fn try_auto_fill(groups: Vec<CssGridLineNames>) -> Option<Self> {
        (!groups.is_empty()).then_some(Self {
            count: CssGridNameRepeatCount::AutoFill,
            groups,
        })
    }

    #[must_use]
    pub const fn count(&self) -> Option<&crate::CssPositiveIntegerValue> {
        match &self.count {
            CssGridNameRepeatCount::Counted(count) => Some(count),
            CssGridNameRepeatCount::AutoFill => None,
        }
    }

    #[must_use]
    pub const fn is_auto_fill(&self) -> bool {
        matches!(self.count, CssGridNameRepeatCount::AutoFill)
    }

    #[must_use]
    pub fn groups(&self) -> &[CssGridLineNames] {
        &self.groups
    }
}

/// A non-empty authored list for `grid-auto-rows` or `grid-auto-columns`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTrackSizeList {
    sizes: Vec<CssGridTrackSize>,
}

impl CssGridTrackSizeList {
    pub fn try_new(sizes: Vec<CssGridTrackSize>) -> Option<Self> {
        (!sizes.is_empty()).then(|| Self::new(sizes))
    }

    pub(crate) const fn new(sizes: Vec<CssGridTrackSize>) -> Self {
        Self { sizes }
    }

    #[must_use]
    pub fn sizes(&self) -> &[CssGridTrackSize] {
        &self.sizes
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssGridTemplateRepresentation {
    None,
    RowsColumns {
        rows: CssGridTrackList,
        columns: CssGridTrackList,
    },
    Areas {
        rows: Vec<CssGridTemplateAreaTrack>,
        columns: Option<CssGridTrackRepeatContent>,
    },
}

/// One area-string row with its authored optional size and boundary groups.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTemplateAreaTrack {
    area: crate::CssGridTemplateAreaRow,
    size: Option<CssGridTrackSize>,
    before: Option<CssGridLineNames>,
    after: Option<CssGridLineNames>,
}

impl CssGridTemplateAreaTrack {
    #[must_use]
    pub fn new(
        area: crate::CssGridTemplateAreaRow,
        size: Option<CssGridTrackSize>,
        before: Option<CssGridLineNames>,
        after: Option<CssGridLineNames>,
    ) -> Self {
        Self {
            area,
            size,
            before,
            after,
        }
    }
    #[must_use]
    pub const fn area(&self) -> &crate::CssGridTemplateAreaRow {
        &self.area
    }
    #[must_use]
    pub const fn size(&self) -> Option<&CssGridTrackSize> {
        self.size.as_ref()
    }
    #[must_use]
    pub const fn before(&self) -> Option<&CssGridLineNames> {
        self.before.as_ref()
    }
    #[must_use]
    pub const fn after(&self) -> Option<&CssGridLineNames> {
        self.after.as_ref()
    }
}

/// The authored `grid-template` aggregate.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTemplate {
    representation: CssGridTemplateRepresentation,
}

impl CssGridTemplate {
    /// Exact inverse of the finite authored template alternatives. No repeat
    /// expansion, implicit tracks or area/column cycling is invented.
    pub(crate) fn from_specified_longhands(
        rows: CssGridTrackList,
        columns: CssGridTrackList,
        areas: crate::CssGridTemplateAreas,
    ) -> Result<Option<Self>, crate::CssSpecifiedValueSerializationError> {
        let mut capacity_failure = false;
        let result = (|| {
            let crate::CssGridTemplateAreas::Rows(areas) = areas else {
                return Some(if rows.is_none() && columns.is_none() {
                    Self::none()
                } else {
                    Self::rows_columns(rows, columns)
                });
            };
            let components = rows.general_list()?.components();
            let mut tracks = Vec::new();
            if tracks.try_reserve(areas.rows().len()).is_err() {
                capacity_failure = true;
                return None;
            }
            let mut before = None;
            for component in components {
                match component {
                    CssGridGeneralTrackComponent::LineNames(names) => {
                        if before.is_some() {
                            return None;
                        }
                        before = Some(names.clone());
                    }
                    CssGridGeneralTrackComponent::TrackSize(size) => {
                        let area = areas.rows().get(tracks.len())?.clone();
                        tracks.push(CssGridTemplateAreaTrack::new(
                            area,
                            Some(size.clone()),
                            before.take(),
                            None,
                        ));
                    }
                    _ => return None,
                }
            }
            if tracks.len() != areas.rows().len() {
                return None;
            }
            if let Some(names) = before {
                tracks.last_mut()?.after = Some(names);
            }
            let columns = if columns.is_none() {
                None
            } else {
                let source = columns.general_list()?.components();
                let mut components = Vec::new();
                if components.try_reserve(source.len()).is_err() {
                    capacity_failure = true;
                    return None;
                }
                for component in source {
                    components.push(match component {
                        CssGridGeneralTrackComponent::LineNames(names) => {
                            CssGridTrackRepeatComponent::LineNames(names.clone())
                        }
                        CssGridGeneralTrackComponent::TrackSize(size) => {
                            CssGridTrackRepeatComponent::TrackSize(size.clone())
                        }
                        _ => return None,
                    });
                }
                Some(CssGridTrackRepeatContent::try_new(components)?)
            };
            // The supplied area matrix is already intrinsically checked. Each
            // row is copied once in order, and column content was checked above.
            Some(Self {
                representation: CssGridTemplateRepresentation::Areas {
                    rows: tracks,
                    columns,
                },
            })
        })();
        if capacity_failure {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        Ok(result)
    }

    pub const fn none() -> Self {
        Self {
            representation: CssGridTemplateRepresentation::None,
        }
    }

    pub const fn rows_columns(rows: CssGridTrackList, columns: CssGridTrackList) -> Self {
        Self {
            representation: CssGridTemplateRepresentation::RowsColumns { rows, columns },
        }
    }

    /// Checks the decoded rectangular area matrix without constraining column count.
    pub fn try_areas(
        rows: Vec<CssGridTemplateAreaTrack>,
        columns: Option<CssGridTrackRepeatContent>,
    ) -> Result<Self, crate::CssGridTemplateAreaError> {
        crate::CssGridTemplateAreas::try_rows(rows.iter().map(|row| row.area.clone()).collect())?;
        Ok(Self {
            representation: CssGridTemplateRepresentation::Areas { rows, columns },
        })
    }

    #[must_use]
    pub fn area_rows(&self) -> Option<&[CssGridTemplateAreaTrack]> {
        match &self.representation {
            CssGridTemplateRepresentation::Areas { rows, .. } => Some(rows),
            _ => None,
        }
    }

    #[must_use]
    pub const fn area_columns(&self) -> Option<&CssGridTrackRepeatContent> {
        match &self.representation {
            CssGridTemplateRepresentation::Areas { columns, .. } => columns.as_ref(),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self.representation, CssGridTemplateRepresentation::None)
    }

    #[must_use]
    pub const fn rows(&self) -> Option<&CssGridTrackList> {
        match &self.representation {
            CssGridTemplateRepresentation::RowsColumns { rows, .. } => Some(rows),
            _ => None,
        }
    }

    #[must_use]
    pub const fn columns(&self) -> Option<&CssGridTrackList> {
        match &self.representation {
            CssGridTemplateRepresentation::RowsColumns { columns, .. } => Some(columns),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssGridRepresentation {
    Template(CssGridTemplate),
    AutoFlow {
        flow: CssGridAutoFlowMode,
        auto_tracks: Option<CssGridTrackSizeList>,
        explicit_tracks: CssGridTrackList,
    },
}

/// The authored `grid` aggregate.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGrid {
    representation: CssGridRepresentation,
}

impl CssGrid {
    pub const fn template(value: CssGridTemplate) -> Self {
        Self {
            representation: CssGridRepresentation::Template(value),
        }
    }

    pub const fn from_auto_flow(
        flow: CssGridAutoFlowMode,
        auto_tracks: Option<CssGridTrackSizeList>,
        explicit_tracks: CssGridTrackList,
    ) -> Self {
        Self {
            representation: CssGridRepresentation::AutoFlow {
                flow,
                auto_tracks,
                explicit_tracks,
            },
        }
    }

    #[must_use]
    pub const fn template_value(&self) -> Option<&CssGridTemplate> {
        match &self.representation {
            CssGridRepresentation::Template(value) => Some(value),
            CssGridRepresentation::AutoFlow { .. } => None,
        }
    }

    #[must_use]
    pub const fn auto_flow(&self) -> Option<CssGridAutoFlowMode> {
        match self.representation {
            CssGridRepresentation::AutoFlow { flow, .. } => Some(flow),
            CssGridRepresentation::Template(_) => None,
        }
    }

    #[must_use]
    pub const fn auto_tracks(&self) -> Option<&CssGridTrackSizeList> {
        match &self.representation {
            CssGridRepresentation::AutoFlow { auto_tracks, .. } => auto_tracks.as_ref(),
            CssGridRepresentation::Template(_) => None,
        }
    }

    #[must_use]
    pub const fn explicit_tracks(&self) -> Option<&CssGridTrackList> {
        match &self.representation {
            CssGridRepresentation::AutoFlow {
                explicit_tracks, ..
            } => Some(explicit_tracks),
            CssGridRepresentation::Template(_) => None,
        }
    }
}

type GridSerializationResult<T> = Result<T, crate::CssSpecifiedValueSerializationError>;

trait GridSpecified {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()>;
}

macro_rules! grid_serialization {
    ($($name:ident),+ $(,)?) => {$(
        impl $name {
            /// Returns canonical specified CSS for this represented Grid value.
            pub fn serialize_specified(&self) -> GridSerializationResult<String> {
                self.serialize_specified_with_limits(
                    crate::CssSpecifiedValueSerializationLimits::default(),
                )
            }

            /// Uses one cumulative input, projection and byte budget for all children.
            /// Failure returns no partial CSS and leaves the authored value unchanged.
            pub fn serialize_specified_with_limits(
                &self,
                limits: crate::CssSpecifiedValueSerializationLimits,
            ) -> GridSerializationResult<String> {
                let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }

            pub(crate) fn append_to_rule_writer(
                &self,
                writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> GridSerializationResult<()> {
                self.write_grid(writer)
            }
        }
    )+};
}

grid_serialization!(
    CssGridTrackBreadth,
    CssGridTrackSize,
    CssGridTrackSizeList,
    CssGridTrackList,
    CssGridTemplate,
    CssGrid,
);

fn grid_node(
    writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
) -> GridSerializationResult<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

impl GridSpecified for CssGridTrackBreadth {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        match &self.representation {
            CssGridTrackBreadthRepresentation::Length(specified) => {
                let captured = specified.capture_specified(&mut writer.context)?;
                writer.append(&captured)
            }
            CssGridTrackBreadthRepresentation::Fraction(specified) => {
                let captured = specified.capture_specified(&mut writer.context)?;
                writer.append(&captured)
            }
            other => {
                grid_node(writer)?;
                writer.append(match other {
                    CssGridTrackBreadthRepresentation::MinContent => "min-content",
                    CssGridTrackBreadthRepresentation::MaxContent => "max-content",
                    CssGridTrackBreadthRepresentation::Auto => "auto",
                    _ => unreachable!(),
                })
            }
        }
    }
}

impl GridSpecified for CssGridTrackSize {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        match &self.representation {
            CssGridTrackSizeRepresentation::Breadth(value) => value.write_grid(writer),
            CssGridTrackSizeRepresentation::MinMax { min, max } => {
                grid_node(writer)?;
                writer.append("minmax(")?;
                min.write_grid(writer)?;
                writer.append(", ")?;
                max.write_grid(writer)?;
                writer.append(")")
            }
            CssGridTrackSizeRepresentation::FitContent(specified) => {
                grid_node(writer)?;
                writer.append("fit-content(")?;
                let captured = specified.capture_specified(&mut writer.context)?;
                writer.append(&captured)?;
                writer.append(")")
            }
        }
    }
}

impl GridSpecified for CssGridLineNames {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        grid_node(writer)?;
        writer.append("[")?;
        for (index, name) in self.names.iter().enumerate() {
            if index != 0 {
                writer.append(" ")?;
            }
            grid_node(writer)?;
            if !writer.context.output_suppressed() {
                let escaped = crate::serialization_escaping::capture_identifier(
                    name.ident().as_str(),
                    &writer.context,
                )?;
                writer.append(&escaped)?;
            }
        }
        writer.append("]")
    }
}

fn grid_items<T: GridSpecified>(
    items: &[T],
    writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
) -> GridSerializationResult<()> {
    for (index, item) in items.iter().enumerate() {
        if index != 0 {
            writer.append(" ")?;
        }
        item.write_grid(writer)?;
    }
    Ok(())
}

impl GridSpecified for CssGridTrackRepeatComponent {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(value) => value.write_grid(writer),
            Self::TrackSize(value) => value.write_grid(writer),
        }
    }
}

impl GridSpecified for CssGridFixedRepeatComponent {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(value) => value.write_grid(writer),
            Self::FixedSize(value) => value.size.write_grid(writer),
        }
    }
}

impl GridSpecified for CssGridIntegerTrackRepeat {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        grid_node(writer)?;
        writer.append("repeat(")?;
        self.count
            .serialize_specified_into(&mut writer.context, &mut writer.css)?;
        writer.append(", ")?;
        grid_items(&self.content.components, writer)?;
        writer.append(")")
    }
}

impl GridSpecified for CssGridIntegerFixedRepeat {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        grid_node(writer)?;
        writer.append("repeat(")?;
        self.count
            .serialize_specified_into(&mut writer.context, &mut writer.css)?;
        writer.append(", ")?;
        grid_items(&self.content.components, writer)?;
        writer.append(")")
    }
}

impl GridSpecified for CssGridAutoRepeat {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        grid_node(writer)?;
        writer.append("repeat(")?;
        writer.append(match self.kind {
            CssGridAutoRepeatKind::AutoFill => "auto-fill",
            CssGridAutoRepeatKind::AutoFit => "auto-fit",
        })?;
        writer.append(", ")?;
        grid_items(&self.content.components, writer)?;
        writer.append(")")
    }
}

impl GridSpecified for CssGridGeneralTrackComponent {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(value) => value.write_grid(writer),
            Self::TrackSize(value) => value.write_grid(writer),
            Self::Repeat(value) => value.write_grid(writer),
        }
    }
}

impl GridSpecified for CssGridAutoTrackComponent {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(value) => value.write_grid(writer),
            Self::FixedSize(value) => value.size.write_grid(writer),
            Self::Repeat(value) => value.write_grid(writer),
            Self::AutoRepeat(value) => value.write_grid(writer),
        }
    }
}

impl GridSpecified for CssGridTrackList {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        grid_node(writer)?;
        match &self.representation {
            CssGridTrackListRepresentation::None => writer.append("none"),
            CssGridTrackListRepresentation::General(value) => grid_items(&value.components, writer),
            CssGridTrackListRepresentation::Auto(value) => grid_items(&value.components, writer),
            CssGridTrackListRepresentation::Subgrid(components) => {
                writer.append("subgrid")?;
                if !components.is_empty() {
                    writer.append(" ")?;
                }
                grid_items(components, writer)
            }
        }
    }
}

impl GridSpecified for CssGridSubgridComponent {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(names) => names.write_grid(writer),
            Self::Repeat(value) => {
                grid_node(writer)?;
                writer.append("repeat(")?;
                match &value.count {
                    CssGridNameRepeatCount::Counted(count) => {
                        count.serialize_specified_into(&mut writer.context, &mut writer.css)?
                    }
                    CssGridNameRepeatCount::AutoFill => writer.append("auto-fill")?,
                }
                writer.append(", ")?;
                grid_items(&value.groups, writer)?;
                writer.append(")")
            }
        }
    }
}

impl GridSpecified for CssGridTrackSizeList {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        grid_node(writer)?;
        grid_items(&self.sizes, writer)
    }
}

impl GridSpecified for CssGridTemplate {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        use crate::CssKnownProperty as Property;
        grid_node(writer)?;
        match &self.representation {
            CssGridTemplateRepresentation::None => writer.append("none"),
            CssGridTemplateRepresentation::RowsColumns { rows, columns } => {
                writer.source_property(Property::GridTemplateRows, |writer| {
                    rows.write_grid(writer)
                })?;
                writer.source_property(Property::GridTemplateColumns, |writer| {
                    writer.append(" / ")?;
                    columns.write_grid(writer)
                })
            }
            CssGridTemplateRepresentation::Areas { rows, columns } => {
                for (index, row) in rows.iter().enumerate() {
                    if let Some(names) = &row.before {
                        writer.source_property(Property::GridTemplateRows, |writer| {
                            if index != 0 {
                                writer.append(" ")?;
                            }
                            names.write_grid(writer)?;
                            writer.append(" ")
                        })?;
                    }
                    writer.source_property(Property::GridTemplateAreas, |writer| {
                        if index != 0 && row.before.is_none() {
                            writer.append(" ")?;
                        }
                        crate::grid_template_areas::write_area_row(
                            &row.area,
                            &mut writer.context,
                            &mut writer.css,
                            false,
                        )
                    })?;
                    if let Some(size) = &row.size {
                        writer.source_property(Property::GridTemplateRows, |writer| {
                            writer.append(" ")?;
                            size.write_grid(writer)
                        })?;
                    }
                    if let Some(names) = &row.after {
                        writer.source_property(Property::GridTemplateRows, |writer| {
                            writer.append(" ")?;
                            names.write_grid(writer)
                        })?;
                    }
                }
                if let Some(columns) = columns {
                    writer.source_property(Property::GridTemplateColumns, |writer| {
                        writer.append(" / ")?;
                        grid_node(writer)?;
                        grid_items(&columns.components, writer)
                    })?;
                }
                Ok(())
            }
        }
    }
}

impl GridSpecified for CssGrid {
    fn write_grid(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> GridSerializationResult<()> {
        use crate::CssKnownProperty as Property;
        match &self.representation {
            // The enum carrier is not another logical aggregate.
            CssGridRepresentation::Template(value) => value.write_grid(writer),
            CssGridRepresentation::AutoFlow {
                flow,
                auto_tracks,
                explicit_tracks,
            } => {
                writer.source_property(Property::GridAutoFlow, grid_node)?;
                if flow.axis() == CssGridAutoFlowAxis::Column {
                    writer.source_property(Property::GridTemplateRows, |writer| {
                        explicit_tracks.write_grid(writer)
                    })?;
                    writer
                        .source_property(Property::GridAutoFlow, |writer| writer.append(" / "))?;
                }
                writer.source_property(Property::GridAutoFlow, |writer| {
                    grid_node(writer)?;
                    writer.append("auto-flow")?;
                    if flow.dense() {
                        grid_node(writer)?;
                        writer.append(" dense")?;
                    }
                    Ok(())
                })?;
                if let Some(tracks) = auto_tracks {
                    let property = match flow.axis() {
                        CssGridAutoFlowAxis::Row => Property::GridAutoRows,
                        CssGridAutoFlowAxis::Column => Property::GridAutoColumns,
                    };
                    writer.source_property(property, |writer| {
                        writer.append(" ")?;
                        tracks.write_grid(writer)
                    })?;
                }
                if flow.axis() == CssGridAutoFlowAxis::Row {
                    writer.source_property(Property::GridTemplateColumns, |writer| {
                        writer.append(" / ")?;
                        explicit_tracks.write_grid(writer)
                    })?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod line_name_composition_contract {
    use super::*;
    use crate::specified_rule_serialization::SpecifiedRuleWriter;
    use crate::{
        CssSpecifiedValueSerializationErrorKind as ErrorKind,
        CssSpecifiedValueSerializationLimits as Limits,
    };

    fn names(value: &str) -> CssGridTrackList {
        CssGridTrackList::general(
            CssGridGeneralTrackList::try_new(vec![
                CssGridGeneralTrackComponent::LineNames(CssGridLineNames::new(vec![
                    CssGridLineName::try_new(crate::CssIdent::try_new(value).unwrap()).unwrap(),
                ])),
                CssGridGeneralTrackComponent::TrackSize(CssGridTrackSize::from_breadth(
                    CssGridTrackBreadth::auto(),
                )),
            ])
            .unwrap(),
        )
    }

    fn retained_name(value: &CssGridTrackList) -> &str {
        let CssGridGeneralTrackComponent::LineNames(group) =
            &value.general_list().unwrap().components()[0]
        else {
            panic!("retained line names");
        };
        group.names()[0].ident().as_str()
    }

    #[test]
    fn suppressed_names_visit_nodes_without_needing_escape_bytes() {
        let mut writer = SpecifiedRuleWriter::new(Limits::new(4, 4, 0));
        let result = writer.without_output(|writer| names("a b").append_to_rule_writer(writer));
        assert!(result.is_ok(), "suppressed names emit no bytes: {result:?}");
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            ErrorKind::InputNodeLimit
        );
        assert_eq!(writer.append("x").unwrap_err().kind(), ErrorKind::ByteLimit);
    }

    #[test]
    fn suppressed_names_preserve_a_fully_consumed_sibling_byte_budget() {
        let mut writer = SpecifiedRuleWriter::new(Limits::new(4, 4, 1));
        writer.append("x").unwrap();
        let result = writer.without_output(|writer| names("a").append_to_rule_writer(writer));
        assert!(
            result.is_ok(),
            "suppression spends no remaining bytes: {result:?}"
        );
        assert_eq!(writer.css, "x");
        assert!(!writer.context.output_suppressed());
    }

    #[test]
    fn suppressed_names_still_charge_semantic_work_and_restore_output() {
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 4, 1));
        let error = writer
            .without_output(|writer| names("a").append_to_rule_writer(writer))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InputNodeLimit);
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }

    #[test]
    fn emitted_names_escape_identity_and_fail_atomically_at_byte_limit() {
        let value = names("a b");
        assert_eq!(value.serialize_specified().unwrap(), "[a\\ b] auto");
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(4, 4, 10))
                .unwrap_err()
                .kind(),
            ErrorKind::ByteLimit,
        );
        assert_eq!(retained_name(&value), "a b");
    }

    #[test]
    fn nul_line_name_is_rejected_before_group_construction() {
        let error = crate::CssIdent::try_new("a\0b").unwrap_err();
        assert_eq!(
            error.kind(),
            crate::CssComponentValueErrorKind::InvalidIdentifier,
        );
        assert_eq!(error.origin(), &crate::CssValueOrigin::Programmatic);
    }
}

#[cfg(test)]
mod aggregate_composition_contract {
    use super::*;
    use crate::specified_rule_serialization::SpecifiedRuleWriter;
    use crate::{
        CssSpecifiedValueSerializationErrorKind as Kind,
        CssSpecifiedValueSerializationLimits as Limits,
    };

    fn template(text: &str) -> CssGridTemplate {
        let source = format!("grid-template:{text}");
        let report = crate::parse_style_attribute(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let crate::CssKnownPropertyValueRef::GridTemplate(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("template");
        };
        value.value().clone()
    }

    fn grid(text: &str) -> CssGrid {
        let source = format!("grid:{text}");
        let report = crate::parse_style_attribute(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let crate::CssKnownPropertyValueRef::Grid(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("grid");
        };
        value.value().clone()
    }

    #[test]
    fn template_uses_partially_consumed_work_and_prefix_bytes() {
        // One aggregate, two lists and two literal leaves: five nodes.
        let value = template("10px / 20px");
        let mut writer = SpecifiedRuleWriter::new(Limits::new(6, 6, 12));
        writer.context.charge_input(1).unwrap();
        writer.context.charge_projection(1).unwrap();
        writer.append("x").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "x10px / 20px");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append("x").unwrap_err().kind(), Kind::ByteLimit);
    }

    #[test]
    fn siblings_share_work_and_remaining_bytes_without_reset() {
        let value = grid("10px / 20px");
        let mut writer = SpecifiedRuleWriter::new(Limits::new(10, 10, 23));
        value.append_to_rule_writer(&mut writer).unwrap();
        writer.append(";").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "10px / 20px;10px / 20px");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        let mut short = SpecifiedRuleWriter::new(Limits::new(9, 10, 23));
        value.append_to_rule_writer(&mut short).unwrap();
        assert_eq!(
            value.append_to_rule_writer(&mut short).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
    }

    #[test]
    fn omitted_auto_sizes_have_no_synthetic_nodes_and_suppression_restores_bytes() {
        for text in ["auto-flow / 10px", "10px / auto-flow"] {
            let value = grid(text);
            let mut writer = SpecifiedRuleWriter::new(Limits::new(4, 4, 1));
            writer.append("x").unwrap();
            writer
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap();
            assert_eq!(writer.css, "x");
            assert!(!writer.context.output_suppressed());
            assert_eq!(
                writer.context.charge_input(1).unwrap_err().kind(),
                Kind::InputNodeLimit
            );
            assert_eq!(
                writer.context.charge_projection(1).unwrap_err().kind(),
                Kind::ProjectionNodeLimit
            );
        }
    }

    #[test]
    fn suppressed_numeric_and_named_children_do_work_without_format_scratch() {
        let value = grid("[a\\ b] calc(10px + 5%) / dense auto-flow fit-content(1e50px)");
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(100, 100, 0));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        assert_eq!(value, before);
        // A smaller semantic budget still fails even though all output is omitted.
        let mut limited = SpecifiedRuleWriter::new(Limits::new(1, 100, 1));
        assert_eq!(
            limited
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap_err()
                .kind(),
            Kind::InputNodeLimit
        );
        assert!(!limited.context.output_suppressed());
        limited.append("x").unwrap();
        assert_eq!(limited.css, "x");
    }

    #[test]
    fn dense_and_optional_sizes_are_each_counted_once_under_suppression() {
        for text in ["auto-flow dense 20px / 10px", "10px / auto-flow dense 20px"] {
            let value = grid(text);
            let mut writer = SpecifiedRuleWriter::new(Limits::new(7, 7, 0));
            writer
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap();
            assert_eq!(
                writer.context.charge_input(1).unwrap_err().kind(),
                Kind::InputNodeLimit
            );
            assert_eq!(
                writer.context.charge_projection(1).unwrap_err().kind(),
                Kind::ProjectionNodeLimit
            );
            assert!(writer.css.is_empty());
        }
    }

    #[test]
    fn suppressed_input_limit_failure_restores_enclosing_output() {
        let value = template("auto / auto");
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(0, 100, 1));
        assert_eq!(
            writer
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap_err()
                .kind(),
            Kind::InputNodeLimit
        );
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
        assert_eq!(value, before);
    }

    #[test]
    fn subgrid_and_area_suppression_preserves_semantic_work_and_restores_output() {
        for (text, nodes) in [
            ("subgrid repeat(2, [a] []) / none", 8),
            ("\"a\" auto / 1px", 6),
        ] {
            let value = template(text);
            let before = value.clone();
            let mut writer = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, 1));
            writer.append("x").unwrap();
            writer
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap();
            assert_eq!(writer.css, "x");
            assert!(!writer.context.output_suppressed());
            assert_eq!(
                writer.context.charge_input(1).unwrap_err().kind(),
                Kind::InputNodeLimit
            );
            assert_eq!(
                writer.context.charge_projection(1).unwrap_err().kind(),
                Kind::ProjectionNodeLimit
            );
            for (input, projection, kind) in [
                (nodes - 1, nodes, Kind::InputNodeLimit),
                (nodes, nodes - 1, Kind::ProjectionNodeLimit),
            ] {
                let mut short = SpecifiedRuleWriter::new(Limits::new(input, projection, 1));
                let error = short
                    .without_output(|writer| value.append_to_rule_writer(writer))
                    .unwrap_err();
                assert_eq!(error.kind(), kind);
                assert!(!short.context.output_suppressed());
                short.append("x").unwrap();
                assert_eq!(short.css, "x");
            }
            assert_eq!(value, before);
            assert_eq!(value.serialize_specified().unwrap(), text);
        }
    }

    #[test]
    fn area_rows_and_subgrid_siblings_share_partly_consumed_work_and_bytes() {
        let first = template("subgrid [] [a] / none"); // 1 + 4 + 1
        let second = template("\"a\""); // 1 + row + cell
        let expected = "xsubgrid [] [a] / none;\"a\"";
        let mut writer = SpecifiedRuleWriter::new(Limits::new(10, 10, expected.len()));
        writer.context.charge_input(1).unwrap();
        writer.context.charge_projection(1).unwrap();
        writer.append("x").unwrap();
        first.append_to_rule_writer(&mut writer).unwrap();
        writer.append(";").unwrap();
        second.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append("x").unwrap_err().kind(), Kind::ByteLimit);
    }
}
