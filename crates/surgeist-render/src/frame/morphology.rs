//! Exact separable morphology over admitted native source grids.
use super::{
    FilterOutputFootprint, FrameContext, GraphLoweringSpatialDescriptor,
    bounds::{LogicalBounds, NonEmptyFrameSpatialPlan},
    filter_grid::{self as grid, FilterGridMapping, FilterGridOwner, PeriodCapture},
};
use crate::{
    Color, MorphologyOperator, Transform,
    accounting::{Account, RenderWork, SharedValue},
    error::{Error, Result},
    filter_source::FilterSourceDescriptor,
    morphology::MorphologyForeground,
};

const OWNER: FilterGridOwner = FilterGridOwner::Morphology;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MorphologyAxis {
    X,
    Y,
}
impl MorphologyAxis {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
        }
    }
    pub(crate) const fn shader_value(self) -> u32 {
        self.index() as u32
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MorphologyStage {
    RebaseSource,
    Copy,
    Prepare,
    Table { axis: MorphologyAxis, level: u32 },
    FirstQuery { axis: MorphologyAxis },
    RefineQuery { axis: MorphologyAxis, level: u32 },
    Query { axis: MorphologyAxis, level: u32 },
    Resolve,
    Constant,
}
impl MorphologyStage {
    pub(crate) const fn axis(self) -> Option<MorphologyAxis> {
        match self {
            Self::Table { axis, .. }
            | Self::FirstQuery { axis }
            | Self::RefineQuery { axis, .. }
            | Self::Query { axis, .. } => Some(axis),
            _ => None,
        }
    }
    pub(crate) const fn level(self) -> u32 {
        match self {
            Self::Table { level, .. }
            | Self::RefineQuery { level, .. }
            | Self::Query { level, .. } => level,
            _ => 0,
        }
    }
    pub(crate) const fn span(self) -> u32 {
        match 1_u32.checked_shl(self.level()) {
            Some(span) => span,
            None => 0,
        }
    }
    pub(super) const fn float_output(self) -> bool {
        matches!(
            self,
            Self::Prepare
                | Self::Table { .. }
                | Self::FirstQuery { .. }
                | Self::RefineQuery { .. }
                | Self::Query { .. }
        )
    }
    pub(super) const fn read_count(self) -> usize {
        match self {
            Self::Constant => 0,
            Self::RefineQuery { .. } => 2,
            _ => 1,
        }
    }
}

/// A clamped affine endpoint. Constant endpoints are proved before narrowing a shift.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MorphologyEndpoint {
    kind: i32,
    delta: i32,
}
impl MorphologyEndpoint {
    pub(crate) const fn kind(self) -> i32 {
        self.kind
    }
    pub(crate) const fn delta(self) -> i32 {
        self.delta
    }
    fn clamped(delta: i64, n: u32, m: u32) -> Result<Self> {
        let last = i64::from(m) - 1;
        if last
            .checked_add(delta)
            .ok_or_else(|| numeric(crate::MorphologyNumericPhase::EndpointMapping, delta as f64))?
            <= 0
        {
            return Ok(Self { kind: 0, delta: 0 });
        }
        if delta >= i64::from(n) {
            return Ok(Self { kind: 1, delta: 0 });
        }
        // A reachable nonconstant map is bounded by the actual input/output lengths.
        let delta = i32::try_from(delta)
            .map_err(|_| numeric(crate::MorphologyNumericPhase::EndpointMapping, delta as f64))?;
        i32::try_from(last + i64::from(delta))
            .map_err(|_| numeric(crate::MorphologyNumericPhase::EndpointMapping, last as f64))?;
        Ok(Self { kind: 2, delta })
    }
    fn at(self, j: i64, n: u32) -> i64 {
        match self.kind {
            0 => 0,
            1 => i64::from(n),
            _ => (j + i64::from(self.delta)).clamp(0, i64::from(n)),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MorphologyQuery {
    lower: MorphologyEndpoint,
    upper: MorphologyEndpoint,
    input_length: u32,
    output_length: u32,
    constant_length: Option<u32>,
    max_level: u32,
    valid_output: [i32; 2],
    point: bool,
}
impl MorphologyQuery {
    pub(crate) const fn lower(self) -> MorphologyEndpoint {
        self.lower
    }
    pub(crate) const fn upper(self) -> MorphologyEndpoint {
        self.upper
    }
    pub(crate) const fn input_length(self) -> u32 {
        self.input_length
    }
    pub(crate) const fn constant_length(self) -> Option<u32> {
        self.constant_length
    }
    pub(crate) const fn max_level(self) -> u32 {
        self.max_level
    }
    pub(crate) const fn valid_output(self) -> [i32; 2] {
        self.valid_output
    }
    fn pass_count(self) -> usize {
        if self.max_level == 0 {
            1
        } else if self.constant_length.is_some() {
            self.max_level as usize + 1
        } else {
            self.max_level as usize * 2 + 1
        }
    }
    fn stage_at(self, axis: MorphologyAxis, index: usize) -> Option<MorphologyStage> {
        if index >= self.pass_count() {
            return None;
        }
        if self.max_level == 0 {
            return Some(MorphologyStage::FirstQuery { axis });
        }
        if self.constant_length.is_some() {
            if index < self.max_level as usize {
                Some(MorphologyStage::Table {
                    axis,
                    level: index as u32 + 1,
                })
            } else {
                Some(MorphologyStage::Query {
                    axis,
                    level: self.max_level,
                })
            }
        } else if index == 0 {
            Some(MorphologyStage::FirstQuery { axis })
        } else if index % 2 == 1 {
            Some(MorphologyStage::Table {
                axis,
                level: (index as u32).div_ceil(2),
            })
        } else {
            Some(MorphologyStage::RefineQuery {
                axis,
                level: index as u32 / 2,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MorphologyMapping {
    grid: FilterGridMapping,
    query: Option<MorphologyQuery>,
    previous: bool,
}
impl MorphologyMapping {
    pub(crate) fn source_spatial(self) -> Option<GraphLoweringSpatialDescriptor> {
        self.grid.source_spatial()
    }
    pub(crate) fn previous_query_spatial(self) -> Option<GraphLoweringSpatialDescriptor> {
        self.previous.then(|| self.grid.target_spatial())
    }
    pub(crate) fn target_spatial(self) -> GraphLoweringSpatialDescriptor {
        self.grid.target_spatial()
    }
    pub(crate) fn source_extent(self) -> [u32; 2] {
        self.grid.source_extent()
    }
    pub(crate) const fn target_extent(self) -> [u32; 2] {
        self.grid.target_extent()
    }
    pub(crate) const fn source_base(self) -> [i32; 2] {
        self.grid.source_base()
    }
    pub(crate) const fn source_clip(self) -> [f64; 4] {
        self.grid.source_clip()
    }
    pub(crate) const fn output_clip(self) -> [f64; 4] {
        self.grid.output_clip()
    }
    pub(crate) const fn periodic_base(self) -> [f64; 2] {
        self.grid.periodic_base()
    }
    pub(crate) const fn periodic_step(self) -> [f64; 2] {
        self.grid.periodic_step()
    }
    pub(crate) const fn periodic(self) -> bool {
        self.grid.periodic()
    }
    pub(crate) const fn converted(self) -> bool {
        self.grid.converted()
    }
    pub(crate) const fn color(self) -> Color {
        self.grid.color()
    }
    pub(crate) const fn query(self) -> Option<MorphologyQuery> {
        self.query
    }
    pub(super) fn output_footprint(self) -> FilterOutputFootprint {
        self.grid.output_footprint()
    }
    pub(super) const fn target_plan(self) -> NonEmptyFrameSpatialPlan {
        self.grid.target_plan()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MorphologyRouteKind {
    Constant(Color),
    Copy,
    Active([MorphologyQuery; 2]),
}
#[derive(Debug, PartialEq)]
struct MorphologyRouteData {
    foreground: MorphologyForeground,
    raw: Option<NonEmptyFrameSpatialPlan>,
    original: Option<NonEmptyFrameSpatialPlan>,
    result: NonEmptyFrameSpatialPlan,
    period: Option<PeriodCapture>,
    kind: MorphologyRouteKind,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct MorphologyRoute(SharedValue<MorphologyRouteData>);
impl MorphologyRoute {
    pub(super) fn try_new(
        context: FrameContext,
        foreground: MorphologyForeground,
        raw: Option<NonEmptyFrameSpatialPlan>,
        transform: Transform,
        account: &Account,
    ) -> Result<Self> {
        let output = grid::result_domain(foreground.source);
        let result = grid::require_spatial(context, output, transform, OWNER)?;
        let (original, period, kind) = match foreground.source {
            FilterSourceDescriptor::Constant { color, .. } => {
                (None, None, MorphologyRouteKind::Constant(color))
            }
            _ if raw.is_none() || grid::source_is_empty(foreground.source) => (
                None,
                None,
                MorphologyRouteKind::Constant(Color::TRANSPARENT),
            ),
            source => {
                let period = match source {
                    FilterSourceDescriptor::Periodic { period, .. } => Some(
                        PeriodCapture::try_new_for(context, period, transform, account, OWNER)?,
                    ),
                    _ => None,
                };
                let original = if let Some(period) = period {
                    period.spatial
                } else {
                    grid::require_spatial(
                        context,
                        grid::finite_domain(source).ok_or_else(|| {
                            numeric(crate::MorphologyNumericPhase::EndpointMapping, 0.0)
                        })?,
                        transform,
                        OWNER,
                    )?
                };
                if period.is_some() && raw != Some(original) {
                    return Err(numeric(crate::MorphologyNumericPhase::EndpointMapping, 0.0));
                }
                if let Some(raw) = raw
                    && period.is_none()
                {
                    let a = raw.logical_bounds.rect();
                    let d = original.logical_bounds.rect();
                    if a.x() > d.x()
                        || a.y() > d.y()
                        || a.max().x() < d.max().x()
                        || a.max().y() < d.max().y()
                    {
                        return Err(numeric(crate::MorphologyNumericPhase::EndpointMapping, 0.0));
                    }
                }
                let kind = if foreground.operation.is_disabled() {
                    MorphologyRouteKind::Copy
                } else {
                    let radii = foreground
                        .operation
                        .radii()
                        .ok_or_else(|| numeric(crate::MorphologyNumericPhase::RadiusGrid, 0.0))?;
                    let density = period
                        .map(|p| p.density)
                        .unwrap_or([original.raster_scale.get(); 2]);
                    let base = if period.is_some() {
                        [0; 2]
                    } else {
                        grid::base_between(original, result, OWNER)?
                    };
                    let input = grid::extent(original);
                    let output = if period.is_some() {
                        input
                    } else {
                        grid::extent(result)
                    };
                    let x = query_geometry(
                        foreground.operation.operator(),
                        radii[0],
                        density[0],
                        MorphologyAxisGrid {
                            input_length: input[0],
                            output_length: output[0],
                            base: base[0],
                            periodic: period.is_some(),
                        },
                        account,
                    )?;
                    let y = if x.is_some() {
                        query_geometry(
                            foreground.operation.operator(),
                            radii[1],
                            density[1],
                            MorphologyAxisGrid {
                                input_length: input[1],
                                output_length: output[1],
                                base: base[1],
                                periodic: period.is_some(),
                            },
                            account,
                        )?
                    } else {
                        None
                    };
                    match (x, y) {
                        (Some(x), Some(y)) if x.point && y.point => MorphologyRouteKind::Copy,
                        (Some(x), Some(y)) => MorphologyRouteKind::Active([x, y]),
                        _ => MorphologyRouteKind::Constant(Color::TRANSPARENT),
                    }
                };
                (Some(original), period, kind)
            }
        };
        SharedValue::new(
            MorphologyRouteData {
                foreground,
                raw,
                original,
                result,
                period,
                kind,
            },
            &account.storage(),
        )
        .map(Self)
    }
    fn rebase(&self) -> bool {
        !matches!(self.0.kind, MorphologyRouteKind::Constant(_)) && self.0.raw != self.0.original
    }
    pub(super) fn len(&self) -> usize {
        usize::from(self.rebase())
            + match self.0.kind {
                MorphologyRouteKind::Constant(_) | MorphologyRouteKind::Copy => 1,
                MorphologyRouteKind::Active(q) => 2 + q[0].pass_count() + q[1].pass_count(),
            }
    }
    fn stage_at(&self, mut index: usize) -> Option<MorphologyStage> {
        if self.rebase() {
            if index == 0 {
                return Some(MorphologyStage::RebaseSource);
            }
            index -= 1;
        }
        match self.0.kind {
            MorphologyRouteKind::Constant(_) => (index == 0).then_some(MorphologyStage::Constant),
            MorphologyRouteKind::Copy => (index == 0).then_some(MorphologyStage::Copy),
            MorphologyRouteKind::Active(q) => {
                if index == 0 {
                    return Some(MorphologyStage::Prepare);
                }
                index -= 1;
                for axis in [MorphologyAxis::X, MorphologyAxis::Y] {
                    let query = q[axis.index()];
                    if index < query.pass_count() {
                        return query.stage_at(axis, index);
                    }
                    index -= query.pass_count();
                }
                (index == 0).then_some(MorphologyStage::Resolve)
            }
        }
    }
    fn mapping(&self, index: usize, account: &Account) -> Result<MorphologyMapping> {
        account.work(RenderWork::FilterOperationVisits, 1)?;
        let stage = self
            .stage_at(index)
            .ok_or_else(|| numeric(crate::MorphologyNumericPhase::TableLayout, index as f64))?;
        let data = &*self.0;
        let result = data.result;
        let output = grid::result_domain(data.foreground.source);
        let allocation = |size| grid::allocation_spatial(route_context(result), size, OWNER);
        let make = |source, target| FilterGridMapping::new(source, target, OWNER);
        let mut query = None;
        let mut previous = false;
        let mapping = if stage == MorphologyStage::Constant {
            let MorphologyRouteKind::Constant(color) = data.kind else {
                return Err(numeric(crate::MorphologyNumericPhase::TableLayout, 0.0));
            };
            make(None, result)
                .with_color(color)
                .with_output_clip(output, result, account)?
        } else {
            let original = data
                .original
                .ok_or_else(|| numeric(crate::MorphologyNumericPhase::EndpointMapping, 0.0))?;
            let input_size = grid::extent(original);
            let periodic = data.period.is_some();
            let output_size = if periodic {
                input_size
            } else {
                grid::extent(result)
            };
            match stage {
                MorphologyStage::RebaseSource => {
                    let raw = data.raw.ok_or_else(|| {
                        numeric(crate::MorphologyNumericPhase::EndpointMapping, 0.0)
                    })?;
                    make(Some(raw), original).with_base(
                        grid::base_between(raw, original, OWNER)?,
                        [0; 2],
                        account,
                    )?
                }
                MorphologyStage::Copy => {
                    let mapping =
                        make(Some(original), result).with_output_clip(output, result, account)?;
                    if let Some(period) = data.period {
                        mapping.with_periodic_resolve(period, result, account)?
                    } else {
                        mapping
                            .with_base(
                                grid::base_between(original, result, OWNER)?,
                                [0; 2],
                                account,
                            )?
                            .with_source_clip(
                                grid::finite_domain(data.foreground.source).ok_or_else(|| {
                                    numeric(crate::MorphologyNumericPhase::EndpointMapping, 0.0)
                                })?,
                                original,
                                account,
                            )?
                    }
                }
                MorphologyStage::Prepare => {
                    let mapping = make(Some(original), allocation(input_size)?)
                        .with_flags(periodic, false, false);
                    if let Some(domain) = grid::finite_domain(data.foreground.source) {
                        mapping.with_source_clip(domain, original, account)?
                    } else {
                        mapping
                    }
                }
                MorphologyStage::Resolve => {
                    let mapping = make(Some(allocation(output_size)?), result)
                        .with_flags(periodic, true, true)
                        .with_output_clip(output, result, account)?;
                    if let Some(period) = data.period {
                        mapping.with_periodic_resolve(period, result, account)?
                    } else {
                        mapping
                    }
                }
                MorphologyStage::Table { axis, level: _ }
                | MorphologyStage::FirstQuery { axis }
                | MorphologyStage::RefineQuery { axis, level: _ }
                | MorphologyStage::Query { axis, level: _ } => {
                    let MorphologyRouteKind::Active(queries) = data.kind else {
                        return Err(numeric(crate::MorphologyNumericPhase::TableLayout, 0.0));
                    };
                    let q = queries[axis.index()];
                    query = Some(q);
                    let mut table_size = if axis == MorphologyAxis::X {
                        input_size
                    } else {
                        [output_size[0], input_size[1]]
                    };
                    let level = stage.level();
                    let span = stage.span();
                    if span == 0 || span > q.input_length {
                        return Err(numeric(
                            crate::MorphologyNumericPhase::TableLayout,
                            f64::from(span),
                        ));
                    }
                    let target = if matches!(stage, MorphologyStage::Table { .. }) {
                        if !periodic {
                            table_size[axis.index()] = q.input_length - span / 2 + 1;
                        }
                        let source = allocation(table_size)?;
                        if !periodic {
                            table_size[axis.index()] = q.input_length - span + 1;
                        }
                        (source, allocation(table_size)?)
                    } else {
                        if !periodic {
                            table_size[axis.index()] = q.input_length - span + 1;
                        }
                        let source = allocation(table_size)?;
                        let target = if periodic {
                            input_size
                        } else if axis == MorphologyAxis::X {
                            [output_size[0], input_size[1]]
                        } else {
                            output_size
                        };
                        (source, allocation(target)?)
                    };
                    let _ = level;
                    previous = matches!(stage, MorphologyStage::RefineQuery { .. });
                    make(Some(target.0), target.1).with_flags(periodic, true, false)
                }
                MorphologyStage::Constant => {
                    return Err(numeric(crate::MorphologyNumericPhase::TableLayout, 0.0));
                }
            }
        };
        Ok(MorphologyMapping {
            grid: mapping,
            query,
            previous,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MorphologyPassFacts(SharedValue<MorphologyPassData>);
#[derive(Debug, PartialEq)]
struct MorphologyPassData {
    route: MorphologyRoute,
    index: usize,
    mapping: MorphologyMapping,
    stage: MorphologyStage,
}
impl MorphologyPassFacts {
    pub(super) fn try_new(route: MorphologyRoute, index: usize, account: &Account) -> Result<Self> {
        let stage = route
            .stage_at(index)
            .ok_or_else(|| numeric(crate::MorphologyNumericPhase::TableLayout, index as f64))?;
        let mapping = route.mapping(index, account)?;
        SharedValue::new(
            MorphologyPassData {
                route,
                index,
                mapping,
                stage,
            },
            &account.storage(),
        )
        .map(Self)
    }
    #[cfg(test)]
    pub(crate) fn with_stage_for_test(
        &self,
        stage: MorphologyStage,
        account: &Account,
    ) -> Result<Self> {
        SharedValue::new(
            MorphologyPassData {
                route: self.0.route.clone(),
                index: self.0.index,
                mapping: self.0.mapping,
                stage,
            },
            &account.storage(),
        )
        .map(Self)
    }
    pub(crate) fn operation(&self) -> &crate::PreparedMorphology {
        &self.0.route.0.foreground.operation
    }
    pub(crate) fn mapping(&self) -> MorphologyMapping {
        self.0.mapping
    }
    pub(crate) fn stage(&self) -> MorphologyStage {
        self.0.stage
    }
    pub(crate) fn route_index(&self) -> usize {
        self.0.index
    }
    pub(crate) fn route_len(&self) -> usize {
        self.0.route.len()
    }
    pub(crate) fn expected_stage(
        &self,
        index: usize,
        account: &Account,
    ) -> Result<Option<MorphologyStage>> {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        Ok(self.0.route.stage_at(index))
    }
    pub(crate) fn same_route_in(&self, other: &Self, account: &Account) -> Result<bool> {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        Ok(self.0.route == other.0.route)
    }
    /// Physical scratch capacity classes derived from the already closed route.
    pub(crate) fn capacity_extents(&self) -> Option<([u32; 2], [u32; 2])> {
        let data = &*self.0.route.0;
        if !matches!(data.kind, MorphologyRouteKind::Active(_)) {
            return None;
        }
        let input = grid::extent(data.original?);
        if data.period.is_some() {
            return Some((input, input));
        }
        let output = grid::extent(data.result);
        Some((
            [input[0].max(output[0]), input[1]],
            [output[0], input[1].max(output[1])],
        ))
    }
    pub(crate) fn needs_float32(&self) -> bool {
        self.0.stage.float_output()
    }
    pub(crate) fn sample_opportunities(&self) -> u64 {
        match self.stage() {
            MorphologyStage::Constant => 0,
            MorphologyStage::Table { .. } | MorphologyStage::Query { .. } => 2,
            MorphologyStage::RefineQuery { .. } => 3,
            MorphologyStage::Resolve | MorphologyStage::Copy if self.mapping().periodic() => 4,
            _ => 1,
        }
    }
    pub(crate) fn proves_exact(&self, account: &Account) -> Result<bool> {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let data = &*self.0.route.0;
        // The raw descriptor is anchored to the first actual read by the graph/runtime owner.
        let rebuilt = MorphologyRoute::try_new(
            route_context(data.result),
            data.foreground,
            data.raw,
            Transform::identity(),
            account,
        )?;
        let period_equal = match (data.period, rebuilt.0.period) {
            (Some(a), Some(b)) => {
                a.period == b.period
                    && a.spatial == b.spatial
                    && a.transform == b.transform
                    && a.density == b.density
                    && a.context.surface_scale == b.context.surface_scale
                    && a.context.output_bounds == b.context.output_bounds
            }
            (None, None) => true,
            _ => false,
        };
        Ok(period_equal
            && data.foreground == rebuilt.0.foreground
            && data.raw == rebuilt.0.raw
            && data.original == rebuilt.0.original
            && data.result == rebuilt.0.result
            && data.kind == rebuilt.0.kind
            && self.expected_stage(self.route_index(), account)? == Some(self.stage())
            && self.mapping() == self.0.route.mapping(self.route_index(), account)?)
    }
}
fn route_context(result: NonEmptyFrameSpatialPlan) -> FrameContext {
    FrameContext {
        output_bounds: LogicalBounds::NonEmpty(result.logical_bounds),
        surface_scale: result.raster_scale.get(),
        antialiasing: crate::renderer::Antialiasing::Area,
        base_color: Color::TRANSPARENT,
    }
}
fn numeric(phase: crate::MorphologyNumericPhase, value: f64) -> Error {
    crate::MorphologyPreparationError::Numeric { phase, value }.into()
}

/// An axis of the checked source/output grids and their aligned relative origin.
/// Production callers derive every field from qualified spatial plans.
#[derive(Clone, Copy)]
struct MorphologyAxisGrid {
    input_length: u32,
    output_length: u32,
    base: i64,
    periodic: bool,
}

/// Fixed-size endpoint geometry, with no scan over output pixels or source pixels.
fn query_geometry(
    operator: MorphologyOperator,
    radius: f64,
    density: f64,
    axis_grid: MorphologyAxisGrid,
    account: &Account,
) -> Result<Option<MorphologyQuery>> {
    let MorphologyAxisGrid {
        input_length: n,
        output_length: m,
        base,
        periodic,
    } = axis_grid;
    let all = [0, m as i32];
    if periodic && n == 1 {
        return Ok(Some(MorphologyQuery {
            lower: MorphologyEndpoint { kind: 2, delta: 0 },
            upper: MorphologyEndpoint { kind: 1, delta: 0 },
            input_length: n,
            output_length: m,
            constant_length: Some(1),
            max_level: 0,
            valid_output: all,
            point: true,
        }));
    }
    account.work(RenderWork::CoordinateVisits, 1)?;
    let scaled = radius * density;
    if periodic {
        let full = scaled >= f64::from(n / 2);
        let (length, phase, point) = if full {
            (n, 0, false)
        } else {
            if !scaled.is_finite() || scaled < 0.0 {
                return Err(numeric(crate::MorphologyNumericPhase::RadiusGrid, scaled));
            }
            let r = scaled.floor() as u32;
            (
                2 * r + 1,
                (-(i64::from(r))).rem_euclid(i64::from(n)) as i32,
                r == 0,
            )
        };
        return Ok(Some(MorphologyQuery {
            lower: MorphologyEndpoint {
                kind: 2,
                delta: phase,
            },
            upper: MorphologyEndpoint { kind: 1, delta: 0 },
            input_length: n,
            output_length: m,
            constant_length: Some(length),
            max_level: 31 - length.leading_zeros(),
            valid_output: all,
            point,
        }));
    }
    if operator == MorphologyOperator::Erode && scaled >= f64::from(n.div_ceil(2)) {
        return Ok(None);
    }
    let last = base
        .checked_add(i64::from(m) - 1)
        .ok_or_else(|| numeric(crate::MorphologyNumericPhase::EndpointMapping, base as f64))?;
    let full_threshold = last.max(i64::from(n) - 1 - base).max(0);
    if operator == MorphologyOperator::Dilate && scaled >= full_threshold as f64 {
        return Ok(Some(MorphologyQuery {
            lower: MorphologyEndpoint { kind: 0, delta: 0 },
            upper: MorphologyEndpoint { kind: 1, delta: 0 },
            input_length: n,
            output_length: m,
            constant_length: Some(n),
            max_level: 31 - n.leading_zeros(),
            valid_output: all,
            point: scaled < 1.0,
        }));
    }
    if !scaled.is_finite() || scaled < 0.0 || scaled.floor() > i64::MAX as f64 {
        return Err(numeric(crate::MorphologyNumericPhase::RadiusGrid, scaled));
    }
    let r = scaled.floor() as i64;
    let lo = base
        .checked_sub(r)
        .ok_or_else(|| numeric(crate::MorphologyNumericPhase::EndpointMapping, base as f64))?;
    let hi = base
        .checked_add(r)
        .and_then(|v| v.checked_add(1))
        .ok_or_else(|| numeric(crate::MorphologyNumericPhase::EndpointMapping, base as f64))?;
    let lower = MorphologyEndpoint::clamped(lo, n, m)?;
    let upper = MorphologyEndpoint::clamped(hi, n, m)?;
    let valid_output = if operator == MorphologyOperator::Erode {
        let start = (r - base).clamp(0, i64::from(m));
        let end = (i64::from(n) - r - base).clamp(0, i64::from(m));
        [start as i32, end as i32]
    } else {
        all
    };
    let mut min = u32::MAX;
    let mut max = 0;
    let candidates = [
        0,
        i64::from(m) - 1,
        (-lo).clamp(0, i64::from(m) - 1),
        (i64::from(n) - lo).clamp(0, i64::from(m) - 1),
        (-hi).clamp(0, i64::from(m) - 1),
        (i64::from(n) - hi).clamp(0, i64::from(m) - 1),
    ];
    for j in candidates {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let length = if j < i64::from(valid_output[0]) || j >= i64::from(valid_output[1]) {
            0
        } else {
            (upper.at(j, n) - lower.at(j, n)).max(0) as u32
        };
        min = min.min(length);
        max = max.max(length);
    }
    // Erosion uses a single full-length query wherever its separate exterior predicate succeeds.
    let constant_length = if operator == MorphologyOperator::Erode {
        let length = r
            .checked_mul(2)
            .and_then(|v| v.checked_add(1))
            .ok_or_else(|| numeric(crate::MorphologyNumericPhase::RadiusGrid, r as f64))?;
        if valid_output[0] >= valid_output[1] {
            return Ok(None);
        }
        Some(
            u32::try_from(length)
                .map_err(|_| numeric(crate::MorphologyNumericPhase::RadiusGrid, length as f64))?,
        )
    } else {
        (min == max).then_some(max)
    };
    if let Some(length) = constant_length {
        max = length;
    }
    if max == 0 {
        return Ok(None);
    }
    Ok(Some(MorphologyQuery {
        lower,
        upper,
        input_length: n,
        output_length: m,
        constant_length,
        max_level: 31 - max.leading_zeros(),
        valid_output,
        point: r == 0,
    }))
}

/// Analytic dead-input proof runs before child planning, so omitted captures have no orphan backing.
pub(super) fn input_is_unreachable(
    context: FrameContext,
    foreground: MorphologyForeground,
    transform: Transform,
    account: &Account,
) -> Result<bool> {
    let FilterSourceDescriptor::Finite { .. } = foreground.source else {
        return Ok(false);
    };
    if grid::source_is_empty(foreground.source)
        || grid::empty(grid::result_domain(foreground.source))
    {
        return Ok(true);
    }
    let Some(radii) = foreground.operation.radii() else {
        return Ok(false);
    };
    let input = grid::require_spatial(
        context,
        grid::finite_domain(foreground.source)
            .ok_or_else(|| numeric(crate::MorphologyNumericPhase::EndpointMapping, 0.0))?,
        transform,
        OWNER,
    )?;
    let result = grid::require_spatial(
        context,
        grid::result_domain(foreground.source),
        transform,
        OWNER,
    )?;
    let base = grid::base_between(input, result, OWNER)?;
    let n = grid::extent(input);
    let m = grid::extent(result);
    for axis in 0..2 {
        if query_geometry(
            foreground.operation.operator(),
            radii[axis],
            input.raster_scale.get(),
            MorphologyAxisGrid {
                input_length: n[axis],
                output_length: m[axis],
                base: base[axis],
                periodic: false,
            },
            account,
        )?
        .is_none()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FilterColorSpace, FilterDomains, Layer, MorphologyInput, MorphologySource, Rect,
        RenderAccountingContext, Scene, Size,
    };
    fn context(scale: f64) -> FrameContext {
        FrameContext::try_new(
            Size::new(64.0, 64.0),
            scale,
            crate::renderer::Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap()
    }
    fn operation(operator: MorphologyOperator, radii: [f64; 2]) -> crate::PreparedMorphology {
        crate::prepare_morphology(
            MorphologyInput::new()
                .with_operator(operator)
                .with_radii(radii[0], radii[1]),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap()
    }
    fn finite_scene() -> Scene {
        let d = Rect::new(-1.5, 0.25, 5.0, 3.0);
        let p = Rect::new(-1.5, 0.25, 5.0, 1.0);
        let f = Rect::new(-4.0, -4.0, 12.0, 12.0);
        let mut input = Scene::new();
        input.fill(d, Color::BLACK);
        let mut scene = Scene::new();
        scene
            .morphology_layer(
                Layer::new(),
                operation(MorphologyOperator::Dilate, [2.0, 0.25]),
                MorphologySource::finite(input, FilterDomains::try_new(d, f, p).unwrap()),
            )
            .unwrap();
        scene
    }
    fn planned(scene: &Scene) -> crate::accounting::OwnedBox<super::super::GpuRenderGraph> {
        let account = Account::for_test();
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, &account)
            .unwrap();
        match super::super::FramePlan::try_from_commands(commands, context(1.0), &account).unwrap()
        {
            super::super::FramePlan::GpuGraph(graph) => graph,
            _ => panic!("morphology must use the graph"),
        }
    }
    #[test]
    fn clipped_query_geometry_matches_literal_integer_intersections_across_levels() {
        for n in [1_u32, 2, 3, 4, 5, 8, 9] {
            for m in [1_u32, 3, 7, 11] {
                for b in [-12_i64, -2, 0, 3, 12] {
                    for r in [0_i64, 1, 2, 4, 9] {
                        let q = query_geometry(
                            MorphologyOperator::Dilate,
                            r as f64 + 0.25,
                            1.0,
                            MorphologyAxisGrid {
                                input_length: n,
                                output_length: m,
                                base: b,
                                periodic: false,
                            },
                            &Account::for_test(),
                        )
                        .unwrap();
                        let literal: Vec<u32> = (0..m)
                            .map(|j| {
                                let p = b + i64::from(j);
                                let low = (p - r).max(0);
                                let high = (p + r).min(i64::from(n) - 1);
                                if high < low {
                                    0
                                } else {
                                    (high - low + 1) as u32
                                }
                            })
                            .collect();
                        let max = *literal.iter().max().unwrap();
                        let min = *literal.iter().min().unwrap();
                        if max == 0 {
                            assert!(q.is_none());
                            continue;
                        }
                        let q = q.unwrap();
                        assert_eq!(q.max_level, 31 - max.leading_zeros());
                        assert_eq!(q.constant_length, (max == min).then_some(max));
                        for (j, length) in literal.into_iter().enumerate() {
                            assert_eq!(
                                (q.upper.at(j as i64, n) - q.lower.at(j as i64, n)) as u32,
                                length
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn finite_erosion_keeps_exterior_predicate_separate_from_clipped_endpoints() {
        let q = query_geometry(
            MorphologyOperator::Erode,
            1.0,
            1.0,
            MorphologyAxisGrid {
                input_length: 5,
                output_length: 9,
                base: -2,
                periodic: false,
            },
            &Account::for_test(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(q.valid_output, [3, 6]);
        assert_eq!(q.constant_length, Some(3));
        assert_eq!(q.max_level, 1);
        assert!(
            query_geometry(
                MorphologyOperator::Erode,
                3.0,
                1.0,
                MorphologyAxisGrid {
                    input_length: 5,
                    output_length: 5,
                    base: 0,
                    periodic: false
                },
                &Account::for_test(),
            )
            .unwrap()
            .is_none()
        );
    }
    #[test]
    fn overflow_radii_use_proved_full_input_period_or_exterior_before_narrowing() {
        let account = Account::for_test();
        let finite = query_geometry(
            MorphologyOperator::Dilate,
            f64::MAX,
            2.0,
            MorphologyAxisGrid {
                input_length: 5,
                output_length: 7,
                base: -2,
                periodic: false,
            },
            &account,
        )
        .unwrap()
        .unwrap();
        assert_eq!(finite.constant_length, Some(5));
        assert_eq!(finite.lower.kind(), 0);
        assert_eq!(finite.upper.kind(), 1);
        assert!(
            query_geometry(
                MorphologyOperator::Erode,
                f64::MAX,
                2.0,
                MorphologyAxisGrid {
                    input_length: 5,
                    output_length: 7,
                    base: -2,
                    periodic: false
                },
                &account,
            )
            .unwrap()
            .is_none()
        );
        let period = query_geometry(
            MorphologyOperator::Erode,
            f64::MAX,
            2.0,
            MorphologyAxisGrid {
                input_length: 9,
                output_length: 9,
                base: 0,
                periodic: true,
            },
            &account,
        )
        .unwrap()
        .unwrap();
        assert_eq!(period.constant_length, Some(9));
        assert_eq!(period.lower.delta(), 0);
        let one = query_geometry(
            MorphologyOperator::Dilate,
            f64::MAX,
            f64::MAX,
            MorphologyAxisGrid {
                input_length: 1,
                output_length: 1,
                base: 0,
                periodic: true,
            },
            &account,
        )
        .unwrap()
        .unwrap();
        assert!(one.point);
        assert_eq!(one.max_level, 0);
    }
    #[test]
    fn quantized_point_and_disabled_routes_copy_without_color_or_float_tables() {
        let d = Rect::new(0.0, 0.0, 3.0, 2.0);
        let source = FilterSourceDescriptor::Finite {
            domains: FilterDomains::try_new(d, d, d).unwrap(),
            edge: crate::FilterEdgeMode::None,
        };
        let account = Account::for_test();
        let raw = grid::require_spatial(context(1.0), d, Transform::identity(), OWNER).unwrap();
        for radii in [[0.0, f64::MAX], [0.25, 0.25]] {
            let route = MorphologyRoute::try_new(
                context(1.0),
                MorphologyForeground {
                    operation: operation(MorphologyOperator::Dilate, radii),
                    source,
                },
                Some(raw),
                Transform::identity(),
                &account,
            )
            .unwrap();
            assert_eq!(route.len(), 1);
            let facts = MorphologyPassFacts::try_new(route, 0, &account).unwrap();
            assert_eq!(facts.stage(), MorphologyStage::Copy);
            assert!(!facts.needs_float32());
            assert!(!facts.mapping().converted());
            assert_eq!(facts.sample_opportunities(), 1);
            assert!(facts.proves_exact(&account).unwrap());
        }
    }
    #[test]
    fn horizontal_queries_retain_every_source_row_before_vertical_completion() {
        let graph = planned(&finite_scene());
        let x = graph
            .morphologies
            .iter()
            .find(|entry| {
                matches!(
                    entry.facts.stage(),
                    MorphologyStage::FirstQuery {
                        axis: MorphologyAxis::X
                    }
                )
            })
            .unwrap();
        assert_eq!(x.facts.mapping().target_extent()[1], 4);
        let y = graph
            .morphologies
            .iter()
            .find(|entry| {
                matches!(
                    entry.facts.stage(),
                    MorphologyStage::FirstQuery {
                        axis: MorphologyAxis::Y
                    }
                )
            })
            .unwrap();
        assert_eq!(y.facts.mapping().source_extent()[1], 4);
        assert_eq!(y.facts.mapping().target_extent()[1], 2);
        graph.lowering_view(&Account::for_test()).unwrap();
    }
    #[test]
    fn either_refinement_input_cannot_be_deleted() {
        for delete in [0, 1] {
            let account = Account::for_test();
            let mut graph = planned(&finite_scene());
            let pass = graph
                .morphologies
                .iter()
                .find(|entry| matches!(entry.facts.stage(), MorphologyStage::RefineQuery { .. }))
                .unwrap()
                .pass
                .index
                .as_usize()
                .unwrap();
            let mut reads = account.retained(1).unwrap();
            reads.push(graph.passes[pass].reads[1 - delete]).unwrap();
            graph.passes[pass].reads =
                SharedValue::from_accounted(reads, &account.storage()).unwrap();
            assert!(graph.lowering_view(&account).is_err());
        }
    }
    #[test]
    fn wrong_table_level_and_unfinished_query_cannot_close() {
        let account = Account::for_test();
        let mut graph = planned(&finite_scene());
        let entry = graph
            .morphologies
            .iter_mut()
            .find(|entry| matches!(entry.facts.stage(), MorphologyStage::Table { level: 1, .. }))
            .unwrap();
        entry.facts = entry
            .facts
            .with_stage_for_test(
                MorphologyStage::Table {
                    axis: MorphologyAxis::X,
                    level: 2,
                },
                &account,
            )
            .unwrap();
        assert!(graph.lowering_view(&account).is_err());
        let mut graph = planned(&finite_scene());
        let seed = graph
            .morphologies
            .iter()
            .find(|entry| {
                matches!(
                    entry.facts.stage(),
                    MorphologyStage::FirstQuery {
                        axis: MorphologyAxis::X
                    }
                )
            })
            .unwrap()
            .pass
            .index
            .as_usize()
            .unwrap();
        let super::super::graph::SemanticPassResult::Resource(seed) = graph.passes[seed].result
        else {
            panic!("seed must be resource")
        };
        let resolve = graph
            .morphologies
            .last()
            .unwrap()
            .pass
            .index
            .as_usize()
            .unwrap();
        let mut reads = account.retained(1).unwrap();
        reads.push(seed).unwrap();
        graph.passes[resolve].reads =
            SharedValue::from_accounted(reads, &account.storage()).unwrap();
        assert!(graph.lowering_view(&account).is_err());
    }
    #[test]
    fn table_and_previous_query_cannot_retire_before_their_remaining_consumers() {
        for previous in [false, true] {
            let mut graph = planned(&finite_scene());
            let entry = graph
                .morphologies
                .iter()
                .find(|entry| {
                    if previous {
                        matches!(
                            entry.facts.stage(),
                            MorphologyStage::FirstQuery {
                                axis: MorphologyAxis::X
                            }
                        )
                    } else {
                        matches!(
                            entry.facts.stage(),
                            MorphologyStage::Table {
                                axis: MorphologyAxis::X,
                                level: 1
                            }
                        )
                    }
                })
                .unwrap();
            let producer = entry.pass;
            let super::super::graph::SemanticPassResult::Resource(resource) =
                graph.passes[producer.index.as_usize().unwrap()].result
            else {
                panic!("table must be resource")
            };
            graph.resources[resource.index.as_usize().unwrap()].releasable_after = Some(producer);
            assert!(graph.lowering_view(&Account::for_test()).is_err());
        }
    }
    #[test]
    fn shared_pass_payload_releases_only_after_last_fact_clone() {
        let graph = planned(&finite_scene());
        let facts = graph.morphologies[0].facts.clone();
        let live = facts.0.allocation_liveness_for_test();
        let clone = facts.clone();
        assert!(facts.0.same_allocation(&clone.0));
        drop(graph);
        drop(facts);
        assert!(live());
        drop(clone);
        assert!(!live());
    }
}
