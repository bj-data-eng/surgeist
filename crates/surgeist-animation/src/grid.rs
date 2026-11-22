//! Structured computed Grid tracks without used-size resolution.
//!
//! CSS/Style supplies effective longhands, decoded line names and computed
//! numeric leaves. Animation preserves explicit track/function tags, fixed
//! repeats and symbolic percentages. Layout owns repeat expansion, percentage
//! bases and intrinsic sizing; consumers own final range handling. Automatic repeats,
//! subgrid, masonry, addition and accumulation are not represented here.
//!
//! Equal outer lists combine corresponding items; unequal lists switch as a
//! whole at progress 0.5. Incompatible fixed-repeat structure also switches the
//! whole template. Otherwise names and incompatible track leaves switch locally;
//! two explicit `minmax()` functions combine their bounds independently. A plain
//! breadth and an explicit `minmax()` remain distinct computed input tags.
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

use crate::computed::{ComputedError, ComputedNumeric, DimensionKind, NumericType};
use crate::interpolation::{DiscreteValue, InterpolationProgress};

/// Constructor or sampling failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridErrorKind {
    EmptyList,
    ZeroRepeatCount,
    InvalidNumericType,
    FlexibleMinimum,
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
            GridErrorKind::Numeric => f.write_str("Grid numeric interpolation failed")?,
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
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, GridError> {
        let value = match (&self.value, &other.value) {
            (Breadth::LengthPercentage(a), Breadth::LengthPercentage(b)) => {
                Breadth::LengthPercentage(a.interpolate(b, progress).map_err(GridError::numeric)?)
            }
            (Breadth::Flex(a), Breadth::Flex(b)) => {
                Breadth::Flex(a.interpolate(b, progress).map_err(GridError::numeric)?)
            }
            _ => return Ok(select(self, other, progress)),
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
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, GridError> {
        let value = match (&self.value, &other.value) {
            (Size::Breadth(a), Size::Breadth(b)) => Size::Breadth(
                a.interpolate(b, progress)
                    .map_err(|error| error.at(GridComponent::Breadth))?,
            ),
            (Size::Minmax(a, b), Size::Minmax(c, d)) => Size::Minmax(
                a.interpolate(c, progress)
                    .map_err(|error| error.at(GridComponent::Minimum))?,
                b.interpolate(d, progress)
                    .map_err(|error| error.at(GridComponent::Maximum))?,
            ),
            (Size::FitContent(a), Size::FitContent(b)) => {
                Size::FitContent(a.interpolate(b, progress).map_err(|error| {
                    GridError::numeric(error).at(GridComponent::FitContentLimit)
                })?)
            }
            _ => return Ok(select(self, other, progress)),
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
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, GridError> {
        let mut tracks = storage(self.tracks.len())?;
        for (index, ((a, names), (b, other_names))) in
            self.tracks.iter().zip(other.tracks.iter()).enumerate()
        {
            tracks.push((
                a.interpolate(b, progress)
                    .map_err(|error| error.repeated(index))?,
                select(names, other_names, progress),
            ));
        }
        Ok(Self {
            count: self.count,
            leading: select(&self.leading, &other.leading, progress),
            tracks: tracks.into(),
        })
    }
}

/// An ordinary track or a preserved fixed repeat section.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackSection {
    Single(TrackSize),
    Repeat(FixedRepeat),
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
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, GridError> {
        if !self.compatible(other) {
            return Ok(select(self, other, progress));
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
                    a.interpolate(b, progress)
                        .map_err(|error| error.section(index))?,
                ),
                (TrackSection::Repeat(a), TrackSection::Repeat(b)) => TrackSection::Repeat(
                    a.interpolate(b, progress)
                        .map_err(|error| error.section(index))?,
                ),
                _ => return Ok(select(self, other, progress)),
            };
            sections.push((section, select(names, other_names, progress)));
        }
        Ok(Self {
            data: Arc::new(TemplateTrackListData {
                leading: select(&self.data.leading, &other.data.leading, progress),
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
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, GridError> {
        if self.tracks.len() != other.tracks.len() {
            return Ok(select(self, other, progress));
        }
        let mut tracks = storage(self.tracks.len())?;
        for (index, (a, b)) in self.tracks.iter().zip(other.tracks.iter()).enumerate() {
            tracks.push(
                a.interpolate(b, progress)
                    .map_err(|error| error.section(index))?,
            );
        }
        Ok(Self {
            tracks: tracks.into(),
        })
    }
}

/// A supplied effective computed Grid track value.
/// Template `none` is separate from nonempty explicit and implicit track lists.
#[derive(Debug, Clone, PartialEq)]
pub enum GridValue {
    None,
    Template(TemplateTrackList),
    Implicit(ImplicitTrackList),
}
impl GridValue {
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, GridError> {
        match (self, other) {
            (Self::Template(a), Self::Template(b)) => {
                a.interpolate(b, progress).map(Self::Template)
            }
            (Self::Implicit(a), Self::Implicit(b)) => {
                a.interpolate(b, progress).map(Self::Implicit)
            }
            _ => Ok(select(self, other, progress)),
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
fn select<T: Clone>(from: &T, to: &T, progress: InterpolationProgress) -> T {
    if progress.value() < 0.5 {
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
