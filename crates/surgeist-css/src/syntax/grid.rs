use super::CssCustomIdent;
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
    names: Vec<CssCustomIdent>,
}

impl CssGridLineNames {
    /// Creates an authored line-name group, including the valid empty group `[]`.
    #[must_use]
    pub fn new(names: Vec<CssCustomIdent>) -> Self {
        Self { names }
    }

    #[must_use]
    pub fn names(&self) -> &[CssCustomIdent] {
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

/// Non-empty, non-recursive integer or automatic track-repeat content.
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
    count: crate::CssPositiveIntegerLiteral,
    content: CssGridTrackRepeatContent,
}

impl CssGridIntegerTrackRepeat {
    pub const fn new(
        count: crate::CssPositiveIntegerLiteral,
        content: CssGridTrackRepeatContent,
    ) -> Self {
        Self { count, content }
    }

    #[must_use]
    pub const fn count(&self) -> &crate::CssPositiveIntegerLiteral {
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
    count: crate::CssPositiveIntegerLiteral,
    content: CssGridFixedRepeatContent,
}

impl CssGridIntegerFixedRepeat {
    pub const fn new(
        count: crate::CssPositiveIntegerLiteral,
        content: CssGridFixedRepeatContent,
    ) -> Self {
        Self { count, content }
    }

    #[must_use]
    pub const fn count(&self) -> &crate::CssPositiveIntegerLiteral {
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
    General(CssGridGeneralTrackList),
    Auto(CssGridAutoTrackList),
}

/// An authored Grid track list, classified as general or automatic.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTrackList {
    representation: CssGridTrackListRepresentation,
}

impl CssGridTrackList {
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
            CssGridTrackListRepresentation::Auto(_) => None,
        }
    }

    #[must_use]
    pub const fn auto_list(&self) -> Option<&CssGridAutoTrackList> {
        match &self.representation {
            CssGridTrackListRepresentation::Auto(value) => Some(value),
            CssGridTrackListRepresentation::General(_) => None,
        }
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
        columns: Option<CssGridTrackList>,
    },
}

/// The authored `grid-template` aggregate.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssGridTemplate {
    representation: CssGridTemplateRepresentation,
}

impl CssGridTemplate {
    pub(crate) const fn none() -> Self {
        Self {
            representation: CssGridTemplateRepresentation::None,
        }
    }

    pub(crate) const fn rows_columns(
        rows: CssGridTrackList,
        columns: Option<CssGridTrackList>,
    ) -> Self {
        Self {
            representation: CssGridTemplateRepresentation::RowsColumns { rows, columns },
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
            CssGridTemplateRepresentation::None => None,
        }
    }

    #[must_use]
    pub const fn columns(&self) -> Option<&CssGridTrackList> {
        match &self.representation {
            CssGridTemplateRepresentation::RowsColumns { columns, .. } => columns.as_ref(),
            CssGridTemplateRepresentation::None => None,
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
    pub(crate) const fn template(value: CssGridTemplate) -> Self {
        Self {
            representation: CssGridRepresentation::Template(value),
        }
    }

    pub(crate) const fn from_auto_flow(
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
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()>;
}

macro_rules! grid_serialization {
    ($($name:ident),+ $(,)?) => {$ (
        impl $name {
            pub fn serialize_specified(&self) -> GridSerializationResult<String> {
                self.serialize_specified_with_limits(crate::CssSpecifiedValueSerializationLimits::default())
            }

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
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {

                let context = &mut writer.context;
                let output = &mut writer.css;
                self.write_grid(context, output)?;
                Ok(())

    }
        }
    )+};
}

grid_serialization!(
    CssGridTrackBreadth,
    CssGridTrackSize,
    CssGridTrackSizeList,
    CssGridTrackList,
);

fn grid_node(
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
) -> GridSerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)
}

impl GridSpecified for CssGridTrackBreadth {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        match &self.representation {
            CssGridTrackBreadthRepresentation::Length(specified) => {
                let captured = specified.capture_specified(context)?;
                context.append(output, &captured)
            }
            CssGridTrackBreadthRepresentation::Fraction(specified) => {
                let captured = specified.capture_specified(context)?;
                context.append(output, &captured)
            }
            other => {
                grid_node(context)?;
                context.append(
                    output,
                    match other {
                        CssGridTrackBreadthRepresentation::MinContent => "min-content",
                        CssGridTrackBreadthRepresentation::MaxContent => "max-content",
                        CssGridTrackBreadthRepresentation::Auto => "auto",
                        _ => unreachable!(),
                    },
                )
            }
        }
    }
}

impl GridSpecified for CssGridTrackSize {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        match &self.representation {
            CssGridTrackSizeRepresentation::Breadth(value) => value.write_grid(context, output),
            CssGridTrackSizeRepresentation::MinMax { min, max } => {
                grid_node(context)?;
                context.append(output, "minmax(")?;
                min.write_grid(context, output)?;
                context.append(output, ", ")?;
                max.write_grid(context, output)?;
                context.append(output, ")")
            }
            CssGridTrackSizeRepresentation::FitContent(specified) => {
                grid_node(context)?;
                context.append(output, "fit-content(")?;
                let captured = specified.capture_specified(context)?;
                context.append(output, &captured)?;
                context.append(output, ")")
            }
        }
    }
}

impl GridSpecified for CssGridLineNames {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        grid_node(context)?;
        context.append(output, "[")?;
        for (index, name) in self.names.iter().enumerate() {
            if index != 0 {
                context.append(output, " ")?;
            }
            grid_node(context)?;
            let escaped = crate::numeric::capture_identifier(name.as_str(), context)?;
            context.append(output, &escaped)?;
        }
        context.append(output, "]")
    }
}

fn grid_items<T: GridSpecified>(
    items: &[T],
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    output: &mut String,
) -> GridSerializationResult<()> {
    for (index, item) in items.iter().enumerate() {
        if index != 0 {
            context.append(output, " ")?;
        }
        item.write_grid(context, output)?;
    }
    Ok(())
}

impl GridSpecified for CssGridTrackRepeatComponent {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(value) => value.write_grid(context, output),
            Self::TrackSize(value) => value.write_grid(context, output),
        }
    }
}

impl GridSpecified for CssGridFixedRepeatComponent {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(value) => value.write_grid(context, output),
            Self::FixedSize(value) => value.size.write_grid(context, output),
        }
    }
}

impl GridSpecified for CssGridIntegerTrackRepeat {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        grid_node(context)?;
        context.append(output, "repeat(")?;
        self.count.integer().append_specified(context, output)?;
        context.append(output, ", ")?;
        grid_items(&self.content.components, context, output)?;
        context.append(output, ")")
    }
}

impl GridSpecified for CssGridIntegerFixedRepeat {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        grid_node(context)?;
        context.append(output, "repeat(")?;
        self.count.integer().append_specified(context, output)?;
        context.append(output, ", ")?;
        grid_items(&self.content.components, context, output)?;
        context.append(output, ")")
    }
}

impl GridSpecified for CssGridAutoRepeat {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        grid_node(context)?;
        context.append(output, "repeat(")?;
        context.append(
            output,
            match self.kind {
                CssGridAutoRepeatKind::AutoFill => "auto-fill",
                CssGridAutoRepeatKind::AutoFit => "auto-fit",
            },
        )?;
        context.append(output, ", ")?;
        grid_items(&self.content.components, context, output)?;
        context.append(output, ")")
    }
}

impl GridSpecified for CssGridGeneralTrackComponent {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(value) => value.write_grid(context, output),
            Self::TrackSize(value) => value.write_grid(context, output),
            Self::Repeat(value) => value.write_grid(context, output),
        }
    }
}

impl GridSpecified for CssGridAutoTrackComponent {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        match self {
            Self::LineNames(value) => value.write_grid(context, output),
            Self::FixedSize(value) => value.size.write_grid(context, output),
            Self::Repeat(value) => value.write_grid(context, output),
            Self::AutoRepeat(value) => value.write_grid(context, output),
        }
    }
}

impl GridSpecified for CssGridTrackList {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        grid_node(context)?;
        match &self.representation {
            CssGridTrackListRepresentation::General(value) => {
                grid_items(&value.components, context, output)
            }
            CssGridTrackListRepresentation::Auto(value) => {
                grid_items(&value.components, context, output)
            }
        }
    }
}

impl GridSpecified for CssGridTrackSizeList {
    fn write_grid(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> GridSerializationResult<()> {
        grid_node(context)?;
        grid_items(&self.sizes, context, output)
    }
}
