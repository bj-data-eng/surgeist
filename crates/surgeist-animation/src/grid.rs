//! Structured computed Grid tracks without used-size resolution.
//!
//! CSS/Style supplies effective longhands, decoded line names and computed
//! numeric leaves. Animation preserves explicit track/function tags, fixed and
//! automatic repeats, subgrid name lists and symbolic percentages. Layout owns
//! repeat expansion, subgrid adoption, percentage bases and intrinsic sizing;
//! consumers own final range handling. Masonry is outside this Grid 1/2 model.
//!
//! Equal outer lists combine corresponding items; unequal lists switch as a
//! whole at progress 0.5. Incompatible fixed-repeat structure also switches the
//! whole template. Otherwise names and incompatible track leaves switch locally;
//! two explicit `minmax()` functions combine their bounds independently. A plain
//! breadth and an explicit `minmax()` remain distinct computed input tags.
//! Automatic-repeat templates and subgrid forms switch as complete originals.
//! Addition and default accumulation share the same traversal, adding numeric
//! leaves while taking effect names and incompatible ordinary items. Incompatible
//! structure, automatic repeats and subgrid composition select the effect.
//!
//! ```
//! use surgeist_animation::grid::{ImplicitTrackList, TrackBreadth, TrackSize};
//! let tracks = ImplicitTrackList::new(vec![
//!     TrackSize::breadth(TrackBreadth::flex(1.0).unwrap()),
//! ]).unwrap();
//! assert_eq!(tracks.tracks().len(), 1);
//! assert!(ImplicitTrackList::new(vec![]).is_err());
//! ```

use core::fmt;
use std::sync::Arc;

use crate::computed::{
    ComputedError, ComputedNumeric, ComputedOperation, DimensionKind, NumericType,
};
use crate::interpolation::{DiscreteValue, InterpolationProgress};

/// Constructor or sampling failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridErrorKind {
    EmptyList,
    ZeroRepeatCount,
    InvalidNumericType,
    FlexibleMinimum,
    InvalidAutomaticTrack,
    MultipleAutomaticRepeats,
    Numeric,
    Capacity,
}

/// The numeric role inside a track that failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridComponent {
    Breadth,
    Minimum,
    Maximum,
    FitContentLimit,
}

/// A checked failure, with zero-based outer/repeat indices when sampling.
#[derive(Debug, Clone, PartialEq)]
pub struct GridError {
    kind: GridErrorKind,
    section_index: Option<usize>,
    repeat_index: Option<usize>,
    component: Option<GridComponent>,
    actual_type: Option<NumericType>,
    numeric: Option<ComputedError>,
}

impl GridError {
    fn new(kind: GridErrorKind) -> Self {
        Self {
            kind,
            section_index: None,
            repeat_index: None,
            component: None,
            actual_type: None,
            numeric: None,
        }
    }
    fn numeric(error: ComputedError) -> Self {
        Self {
            numeric: Some(error),
            ..Self::new(GridErrorKind::Numeric)
        }
    }
    fn invalid_type(actual: NumericType) -> Self {
        Self {
            actual_type: Some(actual),
            ..Self::new(GridErrorKind::InvalidNumericType)
        }
    }
    fn at(mut self, component: GridComponent) -> Self {
        self.component = Some(component);
        self
    }
    fn section(mut self, index: usize) -> Self {
        self.section_index = Some(index);
        self
    }
    fn repeated(mut self, index: usize) -> Self {
        self.repeat_index = Some(index);
        self
    }
    pub const fn kind(&self) -> GridErrorKind {
        self.kind
    }
    pub const fn section_index(&self) -> Option<usize> {
        self.section_index
    }
    pub const fn repeat_index(&self) -> Option<usize> {
        self.repeat_index
    }
    pub const fn component(&self) -> Option<GridComponent> {
        self.component
    }
    pub const fn actual_type(&self) -> Option<NumericType> {
        self.actual_type
    }
    pub const fn numeric_error(&self) -> Option<&ComputedError> {
        self.numeric.as_ref()
    }
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            GridErrorKind::EmptyList => {
                f.write_str("computed Grid track list must not be empty")?
            }
            GridErrorKind::ZeroRepeatCount => {
                f.write_str("fixed Grid repeat count must be positive")?
            }
            GridErrorKind::InvalidNumericType => {
                write!(f, "invalid Grid numeric type {:?}", self.actual_type)?
            }
            GridErrorKind::FlexibleMinimum => {
                f.write_str("Grid minmax minimum cannot be flexible")?
            }
            GridErrorKind::InvalidAutomaticTrack => f.write_str(
                "automatic Grid repeat requires fixed-size tracks throughout its template",
            )?,
            GridErrorKind::MultipleAutomaticRepeats => {
                f.write_str("Grid list cannot contain multiple automatic repeats")?
            }
            GridErrorKind::Numeric => f.write_str("Grid numeric operation failed")?,
            GridErrorKind::Capacity => f.write_str("Grid result storage could not be reserved")?,
        }
        if let Some(index) = self.section_index {
            write!(f, " at section {index}")?;
        }
        if let Some(index) = self.repeat_index {
            write!(f, " repeat item {index}")?;
        }
        if let Some(component) = self.component {
            write!(f, " {component:?}")?;
        }
        if let Some(error) = &self.numeric {
            write!(f, ": {error}")?;
        }
        Ok(())
    }
}
impl std::error::Error for GridError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.numeric
            .as_ref()
            .map(|error| error as &dyn std::error::Error)
    }
}

/// A supplied computed line-name group; order, case and group boundaries remain intact.
/// CSS/Style validates custom identifiers before supplying them here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineNames {
    names: Arc<[DiscreteValue]>,
}
impl LineNames {
    pub fn new(names: Vec<DiscreteValue>) -> Self {
        Self {
            names: names.into(),
        }
    }
    pub fn names(&self) -> &[DiscreteValue] {
        &self.names
    }
}

/// The semantic kind of a computed track breadth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackBreadthKind {
    LengthPercentage,
    Flex,
    Auto,
    MinContent,
    MaxContent,
}

#[derive(Debug, Clone, PartialEq)]
enum Breadth {
    LengthPercentage(ComputedNumeric),
    Flex(ComputedNumeric),
    Auto,
    MinContent,
    MaxContent,
}

/// A guarded length-percentage, typed flex factor or intrinsic keyword.
/// Signed intermediate numbers are retained for caller-owned final clamping.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackBreadth {
    value: Breadth,
}
impl TrackBreadth {
    pub fn length_percentage(numeric: ComputedNumeric) -> Result<Self, GridError> {
        check_length_percentage(&numeric)?;
        Ok(Self {
            value: Breadth::LengthPercentage(numeric),
        })
    }
    pub fn flex(factor: f64) -> Result<Self, GridError> {
        Ok(Self {
            value: Breadth::Flex(ComputedNumeric::number(factor).map_err(GridError::numeric)?),
        })
    }
    pub const fn auto() -> Self {
        Self {
            value: Breadth::Auto,
        }
    }
    pub const fn min_content() -> Self {
        Self {
            value: Breadth::MinContent,
        }
    }
    pub const fn max_content() -> Self {
        Self {
            value: Breadth::MaxContent,
        }
    }
    pub const fn kind(&self) -> TrackBreadthKind {
        match self.value {
            Breadth::LengthPercentage(_) => TrackBreadthKind::LengthPercentage,
            Breadth::Flex(_) => TrackBreadthKind::Flex,
            Breadth::Auto => TrackBreadthKind::Auto,
            Breadth::MinContent => TrackBreadthKind::MinContent,
            Breadth::MaxContent => TrackBreadthKind::MaxContent,
        }
    }
    pub const fn numeric(&self) -> Option<&ComputedNumeric> {
        match &self.value {
            Breadth::LengthPercentage(v) | Breadth::Flex(v) => Some(v),
            _ => None,
        }
    }
    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, GridError> {
        let value = match (&self.value, &other.value) {
            (Breadth::LengthPercentage(a), Breadth::LengthPercentage(b)) => {
                Breadth::LengthPercentage(combine_numeric(a, b, operation)?)
            }
            (Breadth::Flex(a), Breadth::Flex(b)) => {
                Breadth::Flex(combine_numeric(a, b, operation)?)
            }
            _ => return Ok(select(self, other, operation)),
        };
        Ok(Self { value })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Size {
    Breadth(TrackBreadth),
    Minmax(TrackBreadth, TrackBreadth),
    FitContent(ComputedNumeric),
}

/// One computed track size, preserving explicit function identity.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackSize {
    value: Size,
}
impl TrackSize {
    fn is_fixed_size(&self) -> bool {
        match &self.value {
            Size::Breadth(value) => value.kind() == TrackBreadthKind::LengthPercentage,
            Size::Minmax(minimum, maximum) => {
                minimum.kind() == TrackBreadthKind::LengthPercentage
                    || maximum.kind() == TrackBreadthKind::LengthPercentage
            }
            Size::FitContent(_) => false,
        }
    }
    pub const fn breadth(value: TrackBreadth) -> Self {
        Self {
            value: Size::Breadth(value),
        }
    }
    pub fn minmax(minimum: TrackBreadth, maximum: TrackBreadth) -> Result<Self, GridError> {
        if minimum.kind() == TrackBreadthKind::Flex {
            return Err(GridError::new(GridErrorKind::FlexibleMinimum).at(GridComponent::Minimum));
        }
        Ok(Self {
            value: Size::Minmax(minimum, maximum),
        })
    }
    pub fn fit_content(limit: ComputedNumeric) -> Result<Self, GridError> {
        check_length_percentage(&limit)
            .map_err(|error| error.at(GridComponent::FitContentLimit))?;
        Ok(Self {
            value: Size::FitContent(limit),
        })
    }
    pub const fn as_breadth(&self) -> Option<&TrackBreadth> {
        match &self.value {
            Size::Breadth(value) => Some(value),
            _ => None,
        }
    }
    pub const fn minmax_bounds(&self) -> Option<(&TrackBreadth, &TrackBreadth)> {
        match &self.value {
            Size::Minmax(min, max) => Some((min, max)),
            _ => None,
        }
    }
    pub const fn fit_content_limit(&self) -> Option<&ComputedNumeric> {
        match &self.value {
            Size::FitContent(limit) => Some(limit),
            _ => None,
        }
    }
    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, GridError> {
        let value = match (&self.value, &other.value) {
            (Size::Breadth(a), Size::Breadth(b)) => Size::Breadth(
                a.combine(b, operation)
                    .map_err(|error| error.at(GridComponent::Breadth))?,
            ),
            (Size::Minmax(a, b), Size::Minmax(c, d)) => Size::Minmax(
                a.combine(c, operation)
                    .map_err(|error| error.at(GridComponent::Minimum))?,
                b.combine(d, operation)
                    .map_err(|error| error.at(GridComponent::Maximum))?,
            ),
            (Size::FitContent(a), Size::FitContent(b)) => Size::FitContent(
                combine_numeric(a, b, operation)
                    .map_err(|error| error.at(GridComponent::FitContentLimit))?,
            ),
            _ => return Ok(select(self, other, operation)),
        };
        Ok(Self { value })
    }
}

/// A fixed positive repeat with nonempty, nonnested track sizes and line groups.
#[derive(Debug, Clone, PartialEq)]
pub struct FixedRepeat {
    count: usize,
    leading: LineNames,
    tracks: Arc<[(TrackSize, LineNames)]>,
}
impl FixedRepeat {
    pub fn new(
        count: usize,
        leading: LineNames,
        tracks: Vec<(TrackSize, LineNames)>,
    ) -> Result<Self, GridError> {
        if count == 0 {
            return Err(GridError::new(GridErrorKind::ZeroRepeatCount));
        }
        nonempty(&tracks)?;
        Ok(Self {
            count,
            leading,
            tracks: tracks.into(),
        })
    }
    pub const fn count(&self) -> usize {
        self.count
    }
    pub const fn leading_names(&self) -> &LineNames {
        &self.leading
    }
    pub fn tracks(&self) -> &[(TrackSize, LineNames)] {
        &self.tracks
    }
    fn compatible(&self, other: &Self) -> bool {
        self.count == other.count && self.tracks.len() == other.tracks.len()
    }
    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, GridError> {
        let mut tracks = storage(self.tracks.len())?;
        for (index, ((a, names), (b, other_names))) in
            self.tracks.iter().zip(other.tracks.iter()).enumerate()
        {
            tracks.push((
                a.combine(b, operation)
                    .map_err(|error| error.repeated(index))?,
                select(names, other_names, operation),
            ));
        }
        Ok(Self {
            count: self.count,
            leading: select(&self.leading, &other.leading, operation),
            tracks: tracks.into(),
        })
    }
}

/// An automatic repeat keyword, retained without calculating a used count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomaticRepeatKind {
    AutoFill,
    AutoFit,
}

/// A nonempty, nonnested automatic repeat of fixed-size tracks.
/// Both automatic keywords select complete templates discretely, even when equal.
///
/// ```
/// use surgeist_animation::computed::{ComputedDimension, ComputedNumeric, DimensionKind};
/// use surgeist_animation::grid::{AutomaticRepeat, AutomaticRepeatKind, LineNames, TrackBreadth, TrackSize};
/// let size = TrackSize::breadth(TrackBreadth::length_percentage(ComputedNumeric::dimension(
///     ComputedDimension::new(DimensionKind::Length, 20.0).unwrap(),
/// )).unwrap());
/// let repeat = AutomaticRepeat::new(AutomaticRepeatKind::AutoFill,
///     LineNames::new(vec![]), vec![(size, LineNames::new(vec![]))]).unwrap();
/// assert_eq!(repeat.kind(), AutomaticRepeatKind::AutoFill);
/// ```
///
/// Unchecked empty bodies cannot bypass construction.
///
/// ```compile_fail,E0451
/// use surgeist_animation::grid::{AutomaticRepeat, AutomaticRepeatKind, LineNames};
/// let unchecked = AutomaticRepeat { kind: AutomaticRepeatKind::AutoFill,
///     leading: LineNames::new(vec![]), tracks: Vec::new().into() };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AutomaticRepeat {
    kind: AutomaticRepeatKind,
    leading: LineNames,
    tracks: Arc<[(TrackSize, LineNames)]>,
}
impl AutomaticRepeat {
    pub fn new(
        kind: AutomaticRepeatKind,
        leading: LineNames,
        tracks: Vec<(TrackSize, LineNames)>,
    ) -> Result<Self, GridError> {
        nonempty(&tracks)?;
        check_fixed_tracks(&tracks)?;
        Ok(Self {
            kind,
            leading,
            tracks: tracks.into(),
        })
    }
    pub const fn kind(&self) -> AutomaticRepeatKind {
        self.kind
    }
    pub const fn leading_names(&self) -> &LineNames {
        &self.leading
    }
    pub fn tracks(&self) -> &[(TrackSize, LineNames)] {
        &self.tracks
    }
}

/// An ordinary track or preserved fixed/automatic repeat section.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackSection {
    Single(TrackSize),
    Repeat(FixedRepeat),
    AutomaticRepeat(AutomaticRepeat),
}

/// A nonempty computed template list, with leading and trailing line groups.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateTrackList {
    data: Arc<TemplateTrackListData>,
}

#[derive(Debug, PartialEq)]
struct TemplateTrackListData {
    leading: LineNames,
    sections: Arc<[(TrackSection, LineNames)]>,
}
impl TemplateTrackList {
    pub fn new(
        leading: LineNames,
        sections: Vec<(TrackSection, LineNames)>,
    ) -> Result<Self, GridError> {
        nonempty(&sections)?;
        let automatic_count = sections
            .iter()
            .filter(|(section, _)| matches!(section, TrackSection::AutomaticRepeat(_)))
            .count();
        if automatic_count > 1 {
            return Err(GridError::new(GridErrorKind::MultipleAutomaticRepeats));
        }
        if automatic_count == 1 {
            for (index, (section, _)) in sections.iter().enumerate() {
                match section {
                    TrackSection::Single(track) if !track.is_fixed_size() => {
                        return Err(
                            GridError::new(GridErrorKind::InvalidAutomaticTrack).section(index)
                        );
                    }
                    TrackSection::Repeat(repeat) => {
                        check_fixed_tracks(repeat.tracks()).map_err(|error| error.section(index))?
                    }
                    _ => {}
                }
            }
        }
        Ok(Self {
            data: Arc::new(TemplateTrackListData {
                leading,
                sections: sections.into(),
            }),
        })
    }
    pub fn leading_names(&self) -> &LineNames {
        &self.data.leading
    }
    pub fn sections(&self) -> &[(TrackSection, LineNames)] {
        &self.data.sections
    }
    fn compatible(&self, other: &Self) -> bool {
        self.data.sections.len() == other.data.sections.len()
            && self
                .data
                .sections
                .iter()
                .zip(other.data.sections.iter())
                .all(|((a, _), (b, _))| match (a, b) {
                    (TrackSection::Single(_), TrackSection::Single(_)) => true,
                    (TrackSection::Repeat(a), TrackSection::Repeat(b)) => a.compatible(b),
                    _ => false,
                })
    }
    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, GridError> {
        if !self.compatible(other) {
            return Ok(select(self, other, operation));
        }
        let mut sections = storage(self.data.sections.len())?;
        for (index, ((a, names), (b, other_names))) in self
            .data
            .sections
            .iter()
            .zip(other.data.sections.iter())
            .enumerate()
        {
            let section = match (a, b) {
                (TrackSection::Single(a), TrackSection::Single(b)) => TrackSection::Single(
                    a.combine(b, operation)
                        .map_err(|error| error.section(index))?,
                ),
                (TrackSection::Repeat(a), TrackSection::Repeat(b)) => TrackSection::Repeat(
                    a.combine(b, operation)
                        .map_err(|error| error.section(index))?,
                ),
                _ => return Ok(select(self, other, operation)),
            };
            sections.push((section, select(names, other_names, operation)));
        }
        Ok(Self {
            data: Arc::new(TemplateTrackListData {
                leading: select(&self.data.leading, &other.data.leading, operation),
                sections: sections.into(),
            }),
        })
    }
}

/// A nonempty implicit-track list; no least-common-multiple repetition is applied.
///
/// Construction cannot bypass the nonempty-list check.
///
/// ```compile_fail,E0451
/// use surgeist_animation::grid::ImplicitTrackList;
/// let unchecked = ImplicitTrackList { tracks: Vec::new().into() };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ImplicitTrackList {
    tracks: Arc<[TrackSize]>,
}
impl ImplicitTrackList {
    pub fn new(tracks: Vec<TrackSize>) -> Result<Self, GridError> {
        nonempty(&tracks)?;
        Ok(Self {
            tracks: tracks.into(),
        })
    }
    pub fn tracks(&self) -> &[TrackSize] {
        &self.tracks
    }
    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, GridError> {
        if self.tracks.len() != other.tracks.len() {
            return Ok(select(self, other, operation));
        }
        let mut tracks = storage(self.tracks.len())?;
        for (index, (a, b)) in self.tracks.iter().zip(other.tracks.iter()).enumerate() {
            tracks.push(
                a.combine(b, operation)
                    .map_err(|error| error.section(index))?,
            );
        }
        Ok(Self {
            tracks: tracks.into(),
        })
    }
}

/// A repetition count for subgrid line-name groups; auto-fit is not allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameRepeatCount {
    Fixed(usize),
    AutoFill,
}

/// A preserved name-repeat with a positive fixed count or auto-fill and
/// nonempty groups. Empty individual groups remain valid positional anchors.
#[derive(Debug, Clone, PartialEq)]
pub struct NameRepeat {
    count: NameRepeatCount,
    groups: Arc<[LineNames]>,
}
impl NameRepeat {
    pub fn new(count: NameRepeatCount, groups: Vec<LineNames>) -> Result<Self, GridError> {
        if count == NameRepeatCount::Fixed(0) {
            return Err(GridError::new(GridErrorKind::ZeroRepeatCount));
        }
        nonempty(&groups)?;
        Ok(Self {
            count,
            groups: groups.into(),
        })
    }
    pub const fn count(&self) -> NameRepeatCount {
        self.count
    }
    pub fn groups(&self) -> &[LineNames] {
        &self.groups
    }
}

/// One supplied subgrid line group or preserved name-repeat. No track sizes
/// or nested repeats can be inserted into a name-repeat body.
#[derive(Debug, Clone, PartialEq)]
pub enum SubgridSection {
    Names(LineNames),
    Repeat(NameRepeat),
}

/// Grid 2 subgrid's distinct computed line-name list, including bare subgrid.
/// At most one name-repeat may use auto-fill. Layout owns adoption and expansion.
///
/// ```
/// use surgeist_animation::grid::{LineNames, NameRepeat, NameRepeatCount, SubgridSection, SubgridTrackList};
/// let repeat = NameRepeat::new(NameRepeatCount::Fixed(2), vec![LineNames::new(vec![])]).unwrap();
/// let subgrid = SubgridTrackList::new(vec![SubgridSection::Repeat(repeat)]).unwrap();
/// assert_eq!(subgrid.sections().len(), 1);
/// assert!(SubgridTrackList::new(vec![]).is_ok());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SubgridTrackList {
    sections: Arc<[SubgridSection]>,
}
impl SubgridTrackList {
    pub fn new(sections: Vec<SubgridSection>) -> Result<Self, GridError> {
        let automatic_count = sections
            .iter()
            .filter(|section| {
                matches!(section,
            SubgridSection::Repeat(repeat) if repeat.count() == NameRepeatCount::AutoFill)
            })
            .count();
        if automatic_count > 1 {
            return Err(GridError::new(GridErrorKind::MultipleAutomaticRepeats));
        }
        Ok(Self {
            sections: sections.into(),
        })
    }
    pub fn sections(&self) -> &[SubgridSection] {
        &self.sections
    }
}

/// A supplied effective computed Grid track value.
/// Template `none` is separate from nonempty explicit and implicit track lists.
#[derive(Debug, Clone, PartialEq)]
pub enum GridValue {
    None,
    Template(TemplateTrackList),
    Implicit(ImplicitTrackList),
    Subgrid(SubgridTrackList),
}
impl GridValue {
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, GridError> {
        self.combine(other, ComputedOperation::Interpolate(progress))
    }
    /// Adds corresponding compatible numeric components, taking effect names
    /// and incompatible ordinary components. Incompatible structure selects B.
    pub fn add(&self, effect: &Self) -> Result<Self, GridError> {
        self.combine(effect, ComputedOperation::Add)
    }
    /// Uses Values' default accumulation (addition) for Grid numeric leaves.
    /// Repeat counts remain structural; no used counts or list repetition occur.
    pub fn accumulate(&self, effect: &Self) -> Result<Self, GridError> {
        self.combine(effect, ComputedOperation::Accumulate)
    }
    fn combine(&self, other: &Self, operation: ComputedOperation) -> Result<Self, GridError> {
        match (self, other) {
            (Self::Template(a), Self::Template(b)) => a.combine(b, operation).map(Self::Template),
            (Self::Implicit(a), Self::Implicit(b)) => a.combine(b, operation).map(Self::Implicit),
            _ => Ok(select(self, other, operation)),
        }
    }
}

fn check_length_percentage(value: &ComputedNumeric) -> Result<(), GridError> {
    match value.numeric_type() {
        NumericType::Dimension(DimensionKind::Length)
        | NumericType::DimensionPercentage(DimensionKind::Length)
        | NumericType::Percentage => Ok(()),
        actual => Err(GridError::invalid_type(actual)),
    }
}
fn check_fixed_tracks(tracks: &[(TrackSize, LineNames)]) -> Result<(), GridError> {
    for (index, (track, _)) in tracks.iter().enumerate() {
        if !track.is_fixed_size() {
            return Err(GridError::new(GridErrorKind::InvalidAutomaticTrack).repeated(index));
        }
    }
    Ok(())
}
fn combine_numeric(
    from: &ComputedNumeric,
    to: &ComputedNumeric,
    operation: ComputedOperation,
) -> Result<ComputedNumeric, GridError> {
    match operation {
        ComputedOperation::Interpolate(progress) => from.interpolate(to, progress),
        ComputedOperation::Add => from.add(to),
        ComputedOperation::Accumulate => from.accumulate(to),
    }
    .map_err(GridError::numeric)
}
fn select<T: Clone>(from: &T, to: &T, operation: ComputedOperation) -> T {
    if matches!(operation, ComputedOperation::Interpolate(progress) if progress.value() < 0.5) {
        from.clone()
    } else {
        to.clone()
    }
}
fn nonempty<T>(values: &[T]) -> Result<(), GridError> {
    if values.is_empty() {
        Err(GridError::new(GridErrorKind::EmptyList))
    } else {
        Ok(())
    }
}
fn storage<T>(capacity: usize) -> Result<Vec<T>, GridError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| GridError::new(GridErrorKind::Capacity))?;
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::{GridErrorKind, TrackSize, storage};

    #[test]
    fn unrepresentable_grid_result_capacity_returns_a_typed_error() {
        assert_eq!(
            storage::<TrackSize>(usize::MAX).unwrap_err().kind(),
            GridErrorKind::Capacity
        );
    }
}
