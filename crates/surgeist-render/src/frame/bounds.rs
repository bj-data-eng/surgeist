use super::FrameContext;
use super::filter::foreground_filter_bounds;
use crate::accounting::{Account, AccountedVec, RenderWork, SharedValue};
use crate::{
    command::{
        NormalizedLayer, RenderClip, RenderCommand, RenderCommands, RenderLayerMask,
        commands_bounds_for_planning,
    },
    error::{Error, Result},
    filter::{DevicePixelConversionPolicy, FilterOutset, FilterRegionPlan, FilterSourceBounds},
    geometry::{PhysicalSize, Point, Rect, Transform},
    text::{TextGlyph, TextRunBounds, TextRunBoundsKind},
};
#[cfg(test)]
use crate::{geometry::Size, paint::Color, renderer::Antialiasing};

#[cfg(test)]
mod bounded_operation_tests;
#[cfg(test)]
mod contribution_stack_tests;

impl FrameContext {
    #[cfg(test)]
    pub(super) fn try_for_spatial_test(surface_scale: f64) -> Result<Self> {
        Self::try_new(
            Size::new(1.0, 1.0),
            surface_scale,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
    }

    pub(super) fn output_spatial_plan(self) -> Result<FrameSpatialPlan> {
        self.plan_local_bounds(self.output_bounds, Transform::identity())
    }

    pub(super) fn initial_parent_contribution(self) -> SemanticSourceBounds {
        match self.output_bounds {
            LogicalBounds::NonEmpty(bounds) if self.base_color.a() > 0.0 => {
                SemanticSourceBounds::exact_known(bounds)
            }
            LogicalBounds::Empty(_) | LogicalBounds::NonEmpty(_) => {
                SemanticSourceBounds::exactly_empty()
            }
        }
    }

    pub(super) fn plan_local_bounds(
        self,
        logical_bounds: LogicalBounds,
        transform: Transform,
    ) -> Result<FrameSpatialPlan> {
        let metrics = linear_transform_metrics(transform)?;
        let logical_bounds = match logical_bounds {
            LogicalBounds::Empty(bounds) => {
                return Ok(FrameSpatialPlan::Empty(EmptyFrameSpatialPlan {
                    logical_bounds: LogicalBounds::Empty(bounds),
                }));
            }
            LogicalBounds::NonEmpty(bounds) => bounds,
        };
        if metrics.rank_deficient {
            return Ok(FrameSpatialPlan::Empty(EmptyFrameSpatialPlan {
                logical_bounds: LogicalBounds::NonEmpty(logical_bounds),
            }));
        }
        let raster_scale = self.raster_scale(metrics.largest_singular_value)?;
        let source = FilterSourceBounds::try_new(logical_bounds.rect())?;
        let region = FilterRegionPlan::try_new(source, FilterOutset::zero(), None)?;
        let device = DevicePixelConversionPolicy::outward()
            .convert_region(region.execution_region(), raster_scale.get())?;
        let device_origin = SignedDeviceOrigin::new(device.x(), device.y());
        let device_extent = PositiveDeviceExtent::try_new(device.width(), device.height())?;
        let texel_center_mapping = TexelCenterMapping::try_new(device_origin, raster_scale)?;

        Ok(FrameSpatialPlan::NonEmpty(NonEmptyFrameSpatialPlan {
            logical_bounds,
            device_origin,
            device_extent,
            raster_scale,
            texel_center_mapping,
        }))
    }

    pub(super) fn local_raster_scale(self, transform: Transform) -> Result<Option<RasterScale>> {
        let metrics = linear_transform_metrics(transform)?;
        if metrics.rank_deficient {
            return Ok(None);
        }
        self.raster_scale(metrics.largest_singular_value).map(Some)
    }

    fn raster_scale(self, transform_scale: f64) -> Result<RasterScale> {
        RasterScale::try_new(checked_mul(
            self.surface_scale,
            transform_scale,
            "frame local raster scale",
        )?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SemanticSourceBounds {
    known_non_empty_extent: Option<NonEmptyLogicalBounds>,
    contains_unresolved_content: bool,
    sampled_mask_coverage: bool,
}

impl SemanticSourceBounds {
    const fn from_parts(
        known_non_empty_extent: Option<NonEmptyLogicalBounds>,
        contains_unresolved_content: bool,
    ) -> Self {
        Self {
            known_non_empty_extent,
            contains_unresolved_content,
            sampled_mask_coverage: false,
        }
    }

    const fn with_sampled_mask_coverage(mut self, sampled: bool) -> Self {
        self.sampled_mask_coverage = sampled && !self.is_exactly_empty();
        self
    }

    pub(super) const fn exactly_empty() -> Self {
        Self::from_parts(None, false)
    }

    pub(super) const fn exact_known(bounds: NonEmptyLogicalBounds) -> Self {
        Self::from_parts(Some(bounds), false)
    }

    const fn wholly_unresolved() -> Self {
        Self::from_parts(None, true)
    }

    pub(super) const fn is_exactly_empty(self) -> bool {
        self.known_non_empty_extent.is_none() && !self.contains_unresolved_content
    }

    const fn known_non_empty_extent(self) -> Option<NonEmptyLogicalBounds> {
        self.known_non_empty_extent
    }

    const fn contains_unresolved_content(self) -> bool {
        self.contains_unresolved_content
    }

    pub(super) const fn from_logical_bounds(bounds: LogicalBounds) -> Self {
        match bounds {
            LogicalBounds::Empty(_) => Self::exactly_empty(),
            LogicalBounds::NonEmpty(bounds) => Self::exact_known(bounds),
        }
    }

    fn try_for_command(command: &RenderCommand, account: &Account) -> Result<Self> {
        match commands_bounds_for_planning(std::slice::from_ref(command), account)? {
            Some(bounds) => Self::try_from_rect(bounds.rect()),
            None => Ok(Self::wholly_unresolved()),
        }
    }

    fn try_from_rect(rect: Rect) -> Result<Self> {
        LogicalBounds::try_from_rect(rect, "semantic source bounds").map(Self::from_logical_bounds)
    }

    pub(super) fn try_union(self, other: Self) -> Result<Self> {
        let known_non_empty_extent = match (
            self.known_non_empty_extent(),
            other.known_non_empty_extent(),
        ) {
            (Some(a), Some(b)) => Some(a.try_union(b, "semantic source bounds union")?),
            (Some(bounds), None) | (None, Some(bounds)) => Some(bounds),
            (None, None) => None,
        };
        Ok(Self::from_parts(
            known_non_empty_extent,
            self.contains_unresolved_content() || other.contains_unresolved_content(),
        )
        .with_sampled_mask_coverage(self.sampled_mask_coverage || other.sampled_mask_coverage))
    }

    pub(super) fn try_intersect(self, other: Self, name: &'static str) -> Result<Self> {
        if self.is_exactly_empty() || other.is_exactly_empty() {
            return Ok(Self::exactly_empty());
        }
        let known_non_empty_extent = match (
            self.known_non_empty_extent(),
            other.known_non_empty_extent(),
        ) {
            (Some(a), Some(b)) => {
                let a = a.rect();
                let b = b.rect();
                let a_max = a.max();
                let b_max = b.max();
                let min_x = a.x().max(b.x());
                let min_y = a.y().max(b.y());
                let max_x = a_max.x().min(b_max.x());
                let max_y = a_max.y().min(b_max.y());
                let width = checked_sub(max_x, min_x, &format!("{name} width"))?.max(0.0);
                let height = checked_sub(max_y, min_y, &format!("{name} height"))?.max(0.0);
                Self::try_from_rect(Rect::new(min_x, min_y, width, height))?
                    .known_non_empty_extent()
            }
            (Some(_), None) | (None, Some(_)) | (None, None) => None,
        };
        Ok(Self::from_parts(
            known_non_empty_extent,
            self.contains_unresolved_content() || other.contains_unresolved_content(),
        )
        .with_sampled_mask_coverage(self.sampled_mask_coverage || other.sampled_mask_coverage))
    }

    fn try_limit_to_output(self, output: Self, name: &'static str) -> Result<Self> {
        if self.sampled_mask_coverage {
            // The finite raster can contain samples on or beyond its
            // continuous edge. The sampler determines coverage within it.
            return Ok(output.with_sampled_mask_coverage(true));
        }
        self.try_intersect(output, name)
    }

    fn try_transform(self, transform: Transform, name: &'static str) -> Result<Self> {
        if linear_transform_is_rank_deficient(transform)? {
            return Ok(Self::exactly_empty());
        }
        let known_non_empty_extent = match self.known_non_empty_extent() {
            Some(bounds) => {
                match LogicalBounds::NonEmpty(bounds).try_transform(transform, name)? {
                    LogicalBounds::Empty(_) => None,
                    LogicalBounds::NonEmpty(bounds) => Some(bounds),
                }
            }
            None => None,
        };
        Ok(
            Self::from_parts(known_non_empty_extent, self.contains_unresolved_content())
                .with_sampled_mask_coverage(self.sampled_mask_coverage),
        )
    }

    pub(super) fn try_for_clip(clip: &RenderClip, account: &Account) -> Result<Self> {
        Self::try_from_rect(clip.bounds_for_planning(account)?.rect())
    }

    pub(super) fn require_non_empty_for_graph(
        self,
        name: &'static str,
    ) -> Result<Option<NonEmptyLogicalBounds>> {
        if self.contains_unresolved_content() {
            return Err(Error::invalid_value(
                name,
                "unspecified",
                "must be explicit before semantic graph source planning",
            ));
        }
        Ok(self.known_non_empty_extent())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticSourceContribution {
    pub(super) commands: RenderCommands,
    pub(super) source_bounds: SemanticSourceBounds,
    pub(super) affected_bounds: SemanticSourceBounds,
    current_parent: SemanticSourceBounds,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum SemanticContributionDomain {
    RootOutputBounded(LogicalBounds),
    LocalUnbounded,
}

/// One ordered image qualification. Only an existing command advances the
/// cursor; its visit is admitted before cloning or inspecting that command.
struct ContributionSequence {
    commands: RenderCommands,
    next_command: usize,
    contributing_commands: AccountedVec<RenderCommand>,
    source_bounds: SemanticSourceBounds,
    affected_bounds: SemanticSourceBounds,
    current_parent: SemanticSourceBounds,
    domain: SemanticContributionDomain,
    local_to_surface: Transform,
    context: FrameContext,
}

impl ContributionSequence {
    fn try_new(
        commands: RenderCommands,
        initial_parent: SemanticSourceBounds,
        domain: SemanticContributionDomain,
        local_to_surface: Transform,
        context: FrameContext,
        account: &Account,
    ) -> Result<Self> {
        let contributing_commands = account.retained(commands.len())?;
        Ok(Self {
            commands,
            next_command: 0,
            contributing_commands,
            source_bounds: SemanticSourceBounds::exactly_empty(),
            affected_bounds: SemanticSourceBounds::exactly_empty(),
            current_parent: initial_parent,
            domain,
            local_to_surface,
            context,
        })
    }

    fn next(&mut self, account: &Account) -> Result<Option<RenderCommand>> {
        let Some(command) = self.commands.get(self.next_command) else {
            return Ok(None);
        };
        account.work(RenderWork::CommandVisits, 1)?;
        let command = command.clone();
        // A real element exists at the old cursor, so advancing stays at most
        // the sequence length and cannot overflow.
        self.next_command += 1;
        Ok(Some(command))
    }

    fn include(&mut self, contribution: SemanticCommandContribution) -> Result<()> {
        let contribution = contribution.try_apply_domain(self.domain)?;
        self.current_parent = contribution.current_parent;
        if let Some(command) = contribution.command {
            self.source_bounds = self.source_bounds.try_union(contribution.source_bounds)?;
            self.affected_bounds = self
                .affected_bounds
                .try_union(contribution.affected_bounds)?;
            self.contributing_commands.append_slot()?.write(command);
        }
        Ok(())
    }

    fn finish(self, account: &Account) -> Result<SemanticSourceContribution> {
        Ok(SemanticSourceContribution {
            commands: RenderCommands::new_in(self.contributing_commands, account)?,
            source_bounds: self.source_bounds,
            affected_bounds: self.affected_bounds,
            current_parent: self.current_parent,
        })
    }
}

struct LayerQualification {
    layer: SharedValue<NormalizedLayer>,
    layer_to_surface: Transform,
    singular_source: bool,
    has_bounded_children: bool,
    transparent_wrapper: bool,
}

enum CommandQualification {
    Complete(SemanticCommandContribution),
    Children {
        layer: LayerQualification,
        commands: RenderCommands,
        initial_parent: SemanticSourceBounds,
        domain: SemanticContributionDomain,
        child_context: FrameContext,
        child_transform: Transform,
    },
}

/// Every entry is an actual layer awaiting its child's completed image. Its
/// parent retains sibling order, surrounding contribution and spatial domain.
struct SuspendedLayer {
    parent: ContributionSequence,
    layer: LayerQualification,
}

impl SemanticSourceContribution {
    pub(super) fn try_from_commands(
        commands: RenderCommands,
        initial_parent: SemanticSourceBounds,
        domain: SemanticContributionDomain,
        context: FrameContext,
        local_to_surface: Transform,
        account: &Account,
    ) -> Result<Self> {
        let mut current = ContributionSequence::try_new(
            commands,
            initial_parent,
            domain,
            local_to_surface,
            context,
            account,
        )?;
        let mut pending = account.temporary::<SuspendedLayer>(0)?;
        loop {
            if let Some(command) = current.next(account)? {
                match Self::try_qualify_command(
                    command,
                    current.current_parent,
                    current.domain,
                    current.local_to_surface,
                    current.context,
                    account,
                )? {
                    CommandQualification::Complete(contribution) => {
                        current.include(contribution)?
                    }
                    CommandQualification::Children {
                        layer,
                        commands,
                        initial_parent,
                        domain,
                        child_context,
                        child_transform,
                    } => {
                        let sequence = ContributionSequence::try_new(
                            commands,
                            initial_parent,
                            domain,
                            child_transform,
                            child_context,
                            account,
                        )?;
                        if pending.len() == pending.capacity() {
                            pending.grow(pending.capacity().max(1))?;
                        }
                        pending.push(SuspendedLayer {
                            parent: current,
                            layer,
                        })?;
                        current = sequence;
                    }
                }
            } else {
                let children = current.finish(account)?;
                let Some(SuspendedLayer { parent, layer }) = pending.pop() else {
                    return Ok(children);
                };
                let contribution = Self::finish_layer(
                    layer,
                    children,
                    parent.current_parent,
                    parent.domain,
                    parent.context,
                    parent.local_to_surface,
                    account,
                )?;
                current = parent;
                current.include(contribution)?;
            }
        }
    }

    fn try_qualify_command(
        command: RenderCommand,
        current_parent: SemanticSourceBounds,
        domain: SemanticContributionDomain,
        local_to_surface: Transform,
        context: FrameContext,
        account: &Account,
    ) -> Result<CommandQualification> {
        match command {
            RenderCommand::PreparedGlyphRun {
                run,
                transform,
                paint,
            } => {
                let bounds = match run.bounds().ink_rect() {
                    None => SemanticSourceBounds::exactly_empty(),
                    Some(rect) => SemanticSourceBounds::try_from_rect(rect)?,
                }
                .try_transform(transform, "prepared outline transform")?;
                SemanticCommandContribution::try_new(
                    RenderCommand::PreparedGlyphRun {
                        run,
                        transform,
                        paint,
                    },
                    bounds,
                    current_parent,
                )
                .map(CommandQualification::Complete)
            }
            RenderCommand::TextRun {
                font,
                size,
                transform,
                paint,
                glyphs,
                bounds,
            } => {
                let source_bounds = Self::text_source_bounds(&glyphs, bounds, transform)?;
                SemanticCommandContribution::try_new(
                    RenderCommand::TextRun {
                        font,
                        size,
                        transform,
                        paint,
                        glyphs,
                        bounds,
                    },
                    source_bounds,
                    current_parent,
                )
                .map(CommandQualification::Complete)
            }
            RenderCommand::Layer { layer, children } => Self::try_prepare_layer(
                layer,
                children,
                current_parent,
                domain,
                local_to_surface,
                context,
                account,
            ),
            command @ (RenderCommand::Fill { .. }
            | RenderCommand::Stroke { .. }
            | RenderCommand::Shadow { .. }
            | RenderCommand::Image { .. }
            | RenderCommand::ImageRegion { .. }) => {
                let bounds = SemanticSourceBounds::try_for_command(&command, account)?;
                SemanticCommandContribution::try_new(command, bounds, current_parent)
                    .map(CommandQualification::Complete)
            }
        }
    }

    fn text_source_bounds(
        glyphs: &[TextGlyph],
        bounds: TextRunBounds,
        transform: Transform,
    ) -> Result<SemanticSourceBounds> {
        if glyphs.is_empty() || bounds.kind() == TextRunBoundsKind::Empty {
            SemanticSourceBounds::exactly_empty().try_transform(transform, "text source transform")
        } else if bounds.kind() == TextRunBoundsKind::Unspecified {
            SemanticSourceBounds::wholly_unresolved()
                .try_transform(transform, "text source transform")
        } else {
            let ink_bounds = bounds.ink_rect().ok_or_else(|| {
                Error::invalid_value(
                    "text source bounds",
                    "missing ink rectangle",
                    "must carry an ink rectangle when the bounds kind is ink",
                )
            })?;
            SemanticSourceBounds::try_from_rect(ink_bounds)?
                .try_transform(transform, "text source transform")
        }
    }

    fn try_prepare_layer(
        layer: SharedValue<NormalizedLayer>,
        children: RenderCommands,
        current_parent: SemanticSourceBounds,
        domain: SemanticContributionDomain,
        local_to_surface: Transform,
        context: FrameContext,
        account: &Account,
    ) -> Result<CommandQualification> {
        let layer_to_surface = layer.transform.then_in(local_to_surface, account)?;
        let singular_source = linear_transform_is_rank_deficient(layer_to_surface)?;
        if singular_source
            && (layer.clip.is_some()
                || matches!(
                    layer.foreground_filters,
                    Some(
                        crate::command::NormalizedForegroundEffect::Gaussian(_)
                            | crate::command::NormalizedForegroundEffect::NativeShadow(_)
                    )
                ))
        {
            // Own clip coverage collapses under the complete ancestor mapping.
            // Remove the operation before its source prerequisites or graph route.
            return Ok(CommandQualification::Complete(
                SemanticCommandContribution {
                    command: None,
                    source_bounds: SemanticSourceBounds::exactly_empty(),
                    affected_bounds: SemanticSourceBounds::exactly_empty(),
                    current_parent,
                },
            ));
        }
        let gaussian_source_unreachable = layer.foreground_filters.as_ref().is_some_and(|effect| {
            super::bounded_filter_source(effect).is_some_and(|source| {
                super::gaussian::source_is_empty(source)
                    || super::gaussian::empty(super::gaussian::result_domain(source))
            })
        });
        let children = if gaussian_source_unreachable {
            RenderCommands::new_in(account.retained(0)?, account)?
        } else {
            children
        };
        let has_bounded_children = commands_have_bounded_operation(&children, account)?;
        // Only an explicit isolated group starts against transparent black.
        // A default transparent wrapper forwards the actual surrounding image.
        let transparent_wrapper = !layer.explicit_isolation
            && layer.composition.is_source_over_normal()
            && layer.mask.is_none()
            && layer.backdrop.is_none()
            && layer.foreground_filters.is_none()
            && (layer.opacity - 1.0).abs() < f32::EPSILON;
        let inverse = destination_to_layer_local_mapping(layer.transform)?;
        let child_parent = if transparent_wrapper && has_bounded_children {
            match inverse {
                Some(inverse) => {
                    current_parent.try_transform(inverse.affine, "wrapper parent bounds")?
                }
                None => SemanticSourceBounds::exactly_empty(),
            }
        } else {
            SemanticSourceBounds::exactly_empty()
        };
        // Ink outside the output can reach it through the filter's halo.
        let child_domain = match (domain, inverse, layer.foreground_filters.is_some()) {
            (_, _, true) => SemanticContributionDomain::LocalUnbounded,
            (SemanticContributionDomain::RootOutputBounded(bounds), Some(inverse), false) => {
                SemanticContributionDomain::RootOutputBounded(
                    bounds.try_transform(inverse.affine, "child image domain")?,
                )
            }
            (SemanticContributionDomain::RootOutputBounded(bounds), None, false) => {
                SemanticContributionDomain::RootOutputBounded(bounds)
            }
            (SemanticContributionDomain::LocalUnbounded, _, false) => {
                SemanticContributionDomain::LocalUnbounded
            }
        };
        let mut child_context = context;
        let mut child_transform = layer_to_surface;
        let child_domain = if let Some(source) = layer
            .foreground_filters
            .as_ref()
            .and_then(super::bounded_filter_source)
        {
            match source {
                crate::gaussian::GaussianSourceDescriptor::Periodic { period, .. }
                    if !super::gaussian::empty(super::gaussian::result_domain(source))
                        && !children.is_empty()
                        && layer.opacity > 0.0
                        && !singular_source
                        && !layer
                            .mask
                            .as_ref()
                            .is_some_and(RenderLayerMask::annihilates_source) =>
                {
                    let capture = super::gaussian::PeriodCapture::try_new(
                        context,
                        period,
                        layer_to_surface,
                        account,
                    )?;
                    child_context = capture.context;
                    child_transform = capture.transform;
                    SemanticContributionDomain::RootOutputBounded(LogicalBounds::try_from_rect(
                        period,
                        "Gaussian period domain",
                    )?)
                }
                crate::gaussian::GaussianSourceDescriptor::Finite { .. } => {
                    SemanticContributionDomain::RootOutputBounded(LogicalBounds::try_from_rect(
                        super::gaussian::finite_domain(source)
                            .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0)),
                        "Gaussian finite source domain",
                    )?)
                }
                _ => child_domain,
            }
        } else {
            child_domain
        };
        Ok(CommandQualification::Children {
            layer: LayerQualification {
                layer,
                layer_to_surface,
                singular_source,
                has_bounded_children,
                transparent_wrapper,
            },
            commands: children,
            initial_parent: child_parent,
            domain: child_domain,
            child_context,
            child_transform,
        })
    }

    fn finish_layer(
        qualification: LayerQualification,
        mut children: Self,
        current_parent: SemanticSourceBounds,
        domain: SemanticContributionDomain,
        context: FrameContext,
        local_to_surface: Transform,
        account: &Account,
    ) -> Result<SemanticCommandContribution> {
        let LayerQualification {
            layer,
            layer_to_surface,
            singular_source,
            has_bounded_children,
            transparent_wrapper,
        } = qualification;
        // Suspension only reads sealed metadata. Own a value copy exactly
        // where qualification starts pruning or replacing layer properties.
        let mut layer = (*layer).clone();
        let mut source_bounds = match layer.mask.as_ref().filter(|mask| mask.supplies_coverage()) {
            Some(mask) => {
                SemanticSourceBounds::try_from_rect(mask.bounds())?.with_sampled_mask_coverage(true)
            }
            None => children.current_parent,
        };
        let empty_offset = layer.foreground_filters.as_ref().is_some_and(|effect| {
            matches!(effect, crate::command::NormalizedForegroundEffect::Offset(operation)
                if super::offset::domains_are_empty(operation))
        });
        let empty_gaussian = layer.foreground_filters.as_ref().is_some_and(|effect| {
            super::bounded_filter_source(effect).is_some_and(|source| {
                super::gaussian::source_is_empty(source)
                    || super::gaussian::empty(super::gaussian::result_domain(source))
            })
        });
        if empty_offset
            || empty_gaussian
            || layer.foreground_filters.as_ref().is_some_and(|effect| {
                matches!(effect, crate::command::NormalizedForegroundEffect::Convolution(operation)
                if super::convolution::source_domain_is_empty(operation))
            })
        {
            // An empty convolution input excludes backdrop as well as child
            // ink; do not qualify or capture a source the primitive cannot read.
            layer.backdrop = None;
        }
        if empty_offset || empty_gaussian {
            // No sample can reach children or backdrop. Keep only the outer
            // transparent-source operation when Porter-Duff requires it.
            children.commands = RenderCommands::new_in(account.retained(0)?, account)?;
            source_bounds = SemanticSourceBounds::exactly_empty();
        }
        source_bounds = Self::include_backdrop_contribution(
            source_bounds,
            current_parent,
            &mut layer,
            context,
            layer_to_surface,
            account,
        )?;
        match &layer.foreground_filters {
            Some(crate::command::NormalizedForegroundEffect::Css(filters)) => {
                if let Some(bounds) = source_bounds.known_non_empty_extent()
                    && let Some(scale) = context.local_raster_scale(layer_to_surface)?
                {
                    source_bounds = SemanticSourceBounds::from_parts(
                        Some(foreground_filter_bounds(
                            bounds,
                            scale,
                            filters.ops(),
                            account,
                        )?),
                        source_bounds.contains_unresolved_content(),
                    );
                }
            }
            Some(crate::command::NormalizedForegroundEffect::Convolution(operation)) => {
                // Primitive output support is independent of child ink. In
                // particular, nonzero bias can paint an empty captured scene.
                let domains = operation.domains();
                source_bounds =
                    SemanticSourceBounds::from_logical_bounds(LogicalBounds::try_from_rect(
                        super::convolution::intersection(domains.result(), domains.filter()),
                        "convolution primitive result",
                    )?);
            }
            Some(crate::command::NormalizedForegroundEffect::Offset(operation))
                if !source_bounds.is_exactly_empty() && !empty_offset =>
            {
                // Translation cannot create ink from a transparent input;
                // consider the completed same-layer backdrop before this test.
                let domains = operation.domains();
                source_bounds =
                    SemanticSourceBounds::from_logical_bounds(LogicalBounds::try_from_rect(
                        super::convolution::intersection(domains.result(), domains.filter()),
                        "offset primitive result",
                    )?);
            }
            Some(crate::command::NormalizedForegroundEffect::Offset(_)) => {}
            Some(crate::command::NormalizedForegroundEffect::NativeShadow(foreground)) => {
                source_bounds =
                    SemanticSourceBounds::from_logical_bounds(LogicalBounds::try_from_rect(
                        super::gaussian::result_domain(foreground.source),
                        "native shadow primitive result",
                    )?);
            }
            Some(crate::command::NormalizedForegroundEffect::Gaussian(foreground)) => {
                // O is the operation footprint, including transparent Copy.
                source_bounds =
                    SemanticSourceBounds::from_logical_bounds(LogicalBounds::try_from_rect(
                        super::gaussian::result_domain(foreground.source),
                        "Gaussian primitive result",
                    )?);
            }
            None => {}
        }
        if let Some(clip) = layer.clip.as_ref() {
            source_bounds = source_bounds.try_intersect(
                SemanticSourceBounds::try_for_clip(clip, account)?,
                "layer clip intersection",
            )?;
        }
        if layer
            .mask
            .as_ref()
            .is_some_and(RenderLayerMask::annihilates_source)
            || layer.opacity <= 0.0
            || singular_source
        {
            source_bounds = SemanticSourceBounds::exactly_empty();
            // Keep the outer operation, but an annihilated source has no ink
            // or resource prerequisites. Do not let later graph preflight
            // require text bounds or native resources from this dead subtree.
            if !layer.composition.transparent_source_preserves_parent() {
                children.commands = RenderCommands::new_in(account.retained(0)?, account)?;
            }
        }
        source_bounds =
            source_bounds.try_transform(layer.transform, "semantic layer source transform")?;
        let affected_bounds = if !layer.composition.is_source_over() && !layer.composition.is_plus()
        {
            // The parent image fixes operation membership, independently of
            // source support, opacity, masks, or the source transform's rank.
            let mut affected = if let Some(source) = layer
                .foreground_filters
                .as_ref()
                .and_then(super::bounded_filter_source)
            {
                SemanticSourceBounds::from_logical_bounds(LogicalBounds::try_from_rect(
                    super::gaussian::result_domain(source),
                    "Gaussian operation result",
                )?)
                .try_transform(layer.transform, "Gaussian operation transform")?
            } else {
                match domain {
                    SemanticContributionDomain::RootOutputBounded(bounds) => {
                        SemanticSourceBounds::from_logical_bounds(bounds)
                    }
                    SemanticContributionDomain::LocalUnbounded => {
                        match destination_to_layer_local_mapping(local_to_surface)? {
                            Some(inverse) => {
                                SemanticSourceBounds::from_logical_bounds(context.output_bounds)
                                    .try_transform(
                                        inverse.affine,
                                        "local finite operation domain",
                                    )?
                            }
                            // A singular source mapping cannot remove the current
                            // parent image's finite operation membership.
                            None => {
                                SemanticSourceBounds::from_logical_bounds(context.output_bounds)
                            }
                        }
                    }
                }
            };
            if let Some(clip) = layer.clip.as_ref() {
                affected = affected.try_intersect(
                    SemanticSourceBounds::try_for_clip(clip, account)?
                        .try_transform(layer.transform, "operation clip transform")?,
                    "operation clip domain intersection",
                )?;
            }
            affected
        } else if transparent_wrapper && has_bounded_children {
            match domain {
                SemanticContributionDomain::RootOutputBounded(bounds) => {
                    SemanticSourceBounds::from_logical_bounds(bounds)
                }
                SemanticContributionDomain::LocalUnbounded => children.affected_bounds,
            }
        } else {
            source_bounds.try_union(
                children
                    .affected_bounds
                    .try_transform(layer.transform, "child operation affected bounds")?,
            )?
        };
        let affected_bounds = if transparent_wrapper {
            match layer.clip.as_ref() {
                Some(clip) => affected_bounds.try_intersect(
                    SemanticSourceBounds::try_for_clip(clip, account)?
                        .try_transform(layer.transform, "wrapper operation clip transform")?,
                    "wrapper operation clip domain intersection",
                )?,
                None => affected_bounds,
            }
        } else {
            affected_bounds
        };
        let no_operation = affected_bounds.is_exactly_empty()
            || (transparent_wrapper && children.commands.is_empty())
            || (source_bounds.is_exactly_empty()
                && layer.composition.transparent_source_preserves_parent()
                && !transparent_wrapper);
        let composition = layer.composition;
        let partially_covered = layer.clip.is_some();
        let command = if no_operation {
            None
        } else {
            Some(RenderCommand::layer_in(layer, children.commands, account)?)
        };
        SemanticCommandContribution::try_layer(
            command,
            source_bounds,
            affected_bounds,
            current_parent,
            composition,
            partially_covered,
        )
    }
}

fn commands_have_bounded_operation(commands: &RenderCommands, account: &Account) -> Result<bool> {
    let mut commands = commands.as_ref();
    let mut pending = account.temporary(0)?;
    loop {
        let Some((command, siblings)) = commands.split_first() else {
            let Some(siblings) = pending.pop() else {
                return Ok(false);
            };
            commands = siblings;
            continue;
        };
        commands = siblings;
        account.work(RenderWork::CommandVisits, 1)?;
        if let RenderCommand::Layer { layer, children } = command {
            if !layer.composition.is_source_over() && !layer.composition.is_plus() {
                return Ok(true);
            }
            if !children.is_empty() {
                if !siblings.is_empty() {
                    if pending.len() == pending.capacity() {
                        pending.grow(pending.capacity().max(1))?;
                    }
                    pending.push(siblings)?;
                }
                commands = children;
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SemanticCommandContribution {
    command: Option<RenderCommand>,
    source_bounds: SemanticSourceBounds,
    affected_bounds: SemanticSourceBounds,
    current_parent: SemanticSourceBounds,
}

impl SemanticCommandContribution {
    fn try_new(
        command: RenderCommand,
        source_bounds: SemanticSourceBounds,
        current_parent: SemanticSourceBounds,
    ) -> Result<Self> {
        if source_bounds.is_exactly_empty() {
            return Ok(Self {
                command: None,
                source_bounds,
                affected_bounds: source_bounds,
                current_parent,
            });
        }
        Ok(Self {
            command: Some(command),
            source_bounds,
            affected_bounds: source_bounds,
            current_parent: current_parent.try_union(source_bounds)?,
        })
    }

    fn try_layer(
        command: Option<RenderCommand>,
        source_bounds: SemanticSourceBounds,
        affected_bounds: SemanticSourceBounds,
        current_parent: SemanticSourceBounds,
        composition: crate::layer::LayerComposition,
        partially_covered: bool,
    ) -> Result<Self> {
        use crate::layer::PorterDuffOperator;
        let no_operation = command.is_none();
        // These are support bounds, not alpha estimates. In/atop factors bound
        // the result even when there is source ink; destructive operations are
        // still retained through their independent affected domain.
        let operation_result = match composition.operator() {
            Some(PorterDuffOperator::Clear) => SemanticSourceBounds::exactly_empty(),
            Some(
                PorterDuffOperator::Copy
                | PorterDuffOperator::SourceOut
                | PorterDuffOperator::DestinationAtop,
            ) => source_bounds,
            Some(
                PorterDuffOperator::Destination
                | PorterDuffOperator::DestinationOut
                | PorterDuffOperator::SourceAtop,
            ) => current_parent,
            Some(PorterDuffOperator::SourceIn)
                if source_bounds.sampled_mask_coverage && !current_parent.is_exactly_empty() =>
            {
                // Touching continuous mask bounds can share a device sample.
                source_bounds.try_union(current_parent)?
            }
            Some(PorterDuffOperator::DestinationIn) if source_bounds.sampled_mask_coverage => {
                // The target's antialiased pixels can extend beyond its
                // continuous support. Leave the attenuation to the sampler.
                current_parent
            }
            Some(PorterDuffOperator::SourceIn | PorterDuffOperator::DestinationIn) => {
                source_bounds.try_intersect(current_parent, "in-operator result support")?
            }
            Some(
                PorterDuffOperator::SourceOver
                | PorterDuffOperator::DestinationOver
                | PorterDuffOperator::Xor
                | PorterDuffOperator::PlusLighter,
            )
            | None => current_parent.try_union(source_bounds)?,
        };
        let result_support = if no_operation {
            current_parent
        } else if partially_covered {
            // Fractional and nonrectangular clips preserve some prior parent;
            // a single rectangle conservatively bounds that remainder.
            current_parent.try_union(operation_result)?
        } else {
            operation_result
        };
        Ok(Self {
            command,
            source_bounds,
            affected_bounds,
            current_parent: result_support,
        })
    }

    fn try_apply_domain(self, domain: SemanticContributionDomain) -> Result<Self> {
        let SemanticContributionDomain::RootOutputBounded(output_bounds) = domain else {
            return Ok(self);
        };
        let output_bounds = SemanticSourceBounds::from_logical_bounds(output_bounds);
        let source_bounds = self.source_bounds.try_limit_to_output(
            output_bounds,
            "root observable source output-domain intersection",
        )?;
        let affected_bounds = self.affected_bounds.try_limit_to_output(
            output_bounds,
            "root observable affected output-domain intersection",
        )?;
        let current_parent = self.current_parent.try_limit_to_output(
            output_bounds,
            "root current-parent output-domain intersection",
        )?;
        let command = if affected_bounds.is_exactly_empty() {
            None
        } else {
            self.command
        };
        Ok(Self {
            command,
            source_bounds,
            affected_bounds,
            current_parent,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LogicalBounds {
    Empty(EmptyLogicalBounds),
    NonEmpty(NonEmptyLogicalBounds),
}

impl LogicalBounds {
    pub(crate) fn try_from_rect(rect: Rect, name: &str) -> Result<Self> {
        validate_finite(rect.x(), &format!("{name} x"))?;
        validate_finite(rect.y(), &format!("{name} y"))?;
        validate_non_negative(rect.width(), &format!("{name} width"))?;
        validate_non_negative(rect.height(), &format!("{name} height"))?;
        checked_add(rect.x(), rect.width(), &format!("{name} max x"))?;
        checked_add(rect.y(), rect.height(), &format!("{name} max y"))?;

        if rect.width() == 0.0 || rect.height() == 0.0 {
            Ok(Self::Empty(EmptyLogicalBounds { rect }))
        } else {
            Ok(Self::NonEmpty(NonEmptyLogicalBounds { rect }))
        }
    }

    pub(crate) fn try_transform(self, transform: Transform, name: &str) -> Result<Self> {
        let Self::NonEmpty(bounds) = self else {
            return Ok(self);
        };
        let rect = bounds.rect();
        let max = rect.max();
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for (x, y) in [
            (rect.x(), rect.y()),
            (max.x(), rect.y()),
            (max.x(), max.y()),
            (rect.x(), max.y()),
        ] {
            let point = transform_point(transform, x, y, name)?;
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
        }
        let width = checked_sub(max_x, min_x, &format!("{name} width"))?;
        let height = checked_sub(max_y, min_y, &format!("{name} height"))?;
        let transformed_rect = Rect::new(min_x, min_y, width, height);
        let transformed = Self::try_from_rect(transformed_rect, name)?;
        if linear_transform_is_rank_deficient(transform)?
            && matches!(transformed, Self::NonEmpty(_))
        {
            return Ok(Self::Empty(EmptyLogicalBounds {
                rect: Rect::new(min_x, min_y, 0.0, 0.0),
            }));
        }
        Ok(transformed)
    }

    pub(crate) fn union(self, other: Self, name: &str) -> Result<Self> {
        match (self, other) {
            (Self::Empty(_), bounds) | (bounds, Self::Empty(_)) => Ok(bounds),
            (Self::NonEmpty(a), Self::NonEmpty(b)) => {
                let a = a.rect();
                let b = b.rect();
                let a_max = a.max();
                let b_max = b.max();
                let min_x = a.x().min(b.x());
                let min_y = a.y().min(b.y());
                let max_x = a_max.x().max(b_max.x());
                let max_y = a_max.y().max(b_max.y());
                let width = checked_sub(max_x, min_x, &format!("{name} width"))?;
                let height = checked_sub(max_y, min_y, &format!("{name} height"))?;
                Self::try_from_rect(Rect::new(min_x, min_y, width, height), name)
            }
        }
    }

    #[must_use]
    pub(crate) const fn rect(self) -> Rect {
        match self {
            Self::Empty(bounds) => bounds.rect(),
            Self::NonEmpty(bounds) => bounds.rect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EmptyLogicalBounds {
    rect: Rect,
}

impl EmptyLogicalBounds {
    #[must_use]
    const fn rect(self) -> Rect {
        self.rect
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NonEmptyLogicalBounds {
    rect: Rect,
}

impl NonEmptyLogicalBounds {
    #[must_use]
    pub(super) const fn rect(self) -> Rect {
        self.rect
    }

    pub(super) fn try_inflate_uniform(self, amount: f64, name: &str) -> Result<Self> {
        validate_non_negative(amount, &format!("{name} outset"))?;
        let rect = self.rect();
        let doubled = checked_mul(amount, 2.0, &format!("{name} total outset"))?;
        non_empty_logical_bounds(
            Rect::new(
                checked_sub(rect.x(), amount, &format!("{name} x"))?,
                checked_sub(rect.y(), amount, &format!("{name} y"))?,
                checked_add(rect.width(), doubled, &format!("{name} width"))?,
                checked_add(rect.height(), doubled, &format!("{name} height"))?,
            ),
            name,
        )
    }

    pub(super) fn try_translate(self, offset: Point, name: &str) -> Result<Self> {
        let rect = self.rect();
        non_empty_logical_bounds(
            Rect::new(
                checked_add(rect.x(), offset.x(), &format!("{name} x"))?,
                checked_add(rect.y(), offset.y(), &format!("{name} y"))?,
                rect.width(),
                rect.height(),
            ),
            name,
        )
    }

    pub(super) fn try_union(self, other: Self, name: &str) -> Result<Self> {
        match LogicalBounds::NonEmpty(self).union(LogicalBounds::NonEmpty(other), name)? {
            LogicalBounds::NonEmpty(bounds) => Ok(bounds),
            LogicalBounds::Empty(_) => Err(Error::invalid_value(
                name,
                "empty",
                "must remain non-empty when two non-empty bounds are united",
            )),
        }
    }
}

fn non_empty_logical_bounds(rect: Rect, name: &str) -> Result<NonEmptyLogicalBounds> {
    match LogicalBounds::try_from_rect(rect, name)? {
        LogicalBounds::NonEmpty(bounds) => Ok(bounds),
        LogicalBounds::Empty(_) => Err(Error::invalid_value(
            name,
            "empty",
            "must have positive width and height",
        )),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SignedDeviceOrigin {
    pub(super) x: i32,
    pub(super) y: i32,
}

impl SignedDeviceOrigin {
    #[must_use]
    pub(super) const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PositiveDeviceExtent {
    pub(super) width: u32,
    pub(super) height: u32,
}

impl PositiveDeviceExtent {
    fn try_new(width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::invalid_value(
                "frame device extent",
                format!("{width}x{height}"),
                "must have positive width and height",
            ));
        }
        Ok(Self { width, height })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RasterScale(f64);

impl RasterScale {
    fn try_new(value: f64) -> Result<Self> {
        if !value.is_finite() || value <= 0.0 {
            return Err(Error::invalid_value(
                "frame local raster scale",
                value,
                "must be finite and greater than 0",
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub(super) const fn get(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TexelCenterMapping {
    pub(super) origin: Point,
    pub(super) raster_scale: RasterScale,
}

impl TexelCenterMapping {
    fn try_new(device_origin: SignedDeviceOrigin, raster_scale: RasterScale) -> Result<Self> {
        let origin = Point::try_new(
            checked_div(
                f64::from(device_origin.x),
                raster_scale.get(),
                "frame texel mapping origin x",
            )?,
            checked_div(
                f64::from(device_origin.y),
                raster_scale.get(),
                "frame texel mapping origin y",
            )?,
        )?;
        Ok(Self {
            origin,
            raster_scale,
        })
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Texel-center mappings remain available for resolved pass lowering."
        )
    )]
    pub(super) fn point_for(self, i: u32, j: u32) -> Result<Point> {
        let x_offset = checked_div(
            checked_add(f64::from(i), 0.5, "frame texel center i")?,
            self.raster_scale.get(),
            "frame texel center x offset",
        )?;
        let y_offset = checked_div(
            checked_add(f64::from(j), 0.5, "frame texel center j")?,
            self.raster_scale.get(),
            "frame texel center y offset",
        )?;
        Point::try_new(
            checked_add(self.origin.x(), x_offset, "frame texel center x")?,
            checked_add(self.origin.y(), y_offset, "frame texel center y")?,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum FrameSpatialPlan {
    Empty(EmptyFrameSpatialPlan),
    NonEmpty(NonEmptyFrameSpatialPlan),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct EmptyFrameSpatialPlan {
    pub(super) logical_bounds: LogicalBounds,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NonEmptyFrameSpatialPlan {
    pub(super) logical_bounds: NonEmptyLogicalBounds,
    pub(super) device_origin: SignedDeviceOrigin,
    pub(super) device_extent: PositiveDeviceExtent,
    pub(super) raster_scale: RasterScale,
    pub(super) texel_center_mapping: TexelCenterMapping,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DestinationToLayerLocalMapping {
    pub(super) affine: Transform,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LinearTransformMetrics {
    largest_singular_value: f64,
    rank_deficient: bool,
}

fn linear_transform_metrics(transform: Transform) -> Result<LinearTransformMetrics> {
    let [a, b, c, d, _, _] = transform.as_array();
    let coefficient_scale = a.abs().max(b.abs()).max(c.abs()).max(d.abs());
    if coefficient_scale == 0.0 {
        return Ok(LinearTransformMetrics {
            largest_singular_value: 0.0,
            rank_deficient: true,
        });
    }

    let a = checked_div(a, coefficient_scale, "frame transform coefficient a")?;
    let b = checked_div(b, coefficient_scale, "frame transform coefficient b")?;
    let c = checked_div(c, coefficient_scale, "frame transform coefficient c")?;
    let d = checked_div(d, coefficient_scale, "frame transform coefficient d")?;
    let frobenius_squared = checked_add(
        checked_add(
            checked_mul(a, a, "frame transform a squared")?,
            checked_mul(b, b, "frame transform b squared")?,
            "frame transform first column norm",
        )?,
        checked_add(
            checked_mul(c, c, "frame transform c squared")?,
            checked_mul(d, d, "frame transform d squared")?,
            "frame transform second column norm",
        )?,
        "frame transform Frobenius norm squared",
    )?;
    let determinant = checked_sub(
        checked_mul(a, d, "frame transform normalized determinant ad")?,
        checked_mul(b, c, "frame transform normalized determinant bc")?,
        "frame transform normalized determinant",
    )?;
    let discriminant = checked_sub(
        checked_mul(
            frobenius_squared,
            frobenius_squared,
            "frame singular-value discriminant norm",
        )?,
        checked_mul(
            4.0,
            checked_mul(
                determinant,
                determinant,
                "frame singular-value determinant squared",
            )?,
            "frame singular-value determinant term",
        )?,
        "frame singular-value discriminant",
    )?
    .max(0.0);
    let largest_eigenvalue = checked_mul(
        0.5,
        checked_add(
            frobenius_squared,
            discriminant.sqrt(),
            "frame largest singular-value eigenvalue",
        )?,
        "frame largest singular-value eigenvalue",
    )?;
    let normalized = largest_eigenvalue.sqrt();
    validate_finite(normalized, "frame normalized largest singular value")?;
    let largest_singular_value = checked_mul(
        coefficient_scale,
        normalized,
        "frame largest singular value",
    )?;
    Ok(LinearTransformMetrics {
        largest_singular_value,
        rank_deficient: determinant == 0.0,
    })
}

fn linear_transform_is_rank_deficient(transform: Transform) -> Result<bool> {
    let [a, b, c, d, _, _] = transform.as_array();
    let coefficient_scale = a.abs().max(b.abs()).max(c.abs()).max(d.abs());
    if coefficient_scale == 0.0 {
        return Ok(true);
    }

    let a = checked_div(a, coefficient_scale, "frame transform coefficient a")?;
    let b = checked_div(b, coefficient_scale, "frame transform coefficient b")?;
    let c = checked_div(c, coefficient_scale, "frame transform coefficient c")?;
    let d = checked_div(d, coefficient_scale, "frame transform coefficient d")?;
    let determinant = checked_sub(
        checked_mul(a, d, "frame transform normalized determinant ad")?,
        checked_mul(b, c, "frame transform normalized determinant bc")?,
        "frame transform normalized determinant",
    )?;
    Ok(determinant == 0.0)
}

pub(super) fn destination_to_layer_local_mapping(
    layer_to_destination: Transform,
) -> Result<Option<DestinationToLayerLocalMapping>> {
    let [a, b, c, d, e, f] = layer_to_destination.as_array();
    let coefficient_scale = a.abs().max(b.abs()).max(c.abs()).max(d.abs());
    if coefficient_scale == 0.0 {
        return Ok(None);
    }
    let normalized_a = checked_div(
        a,
        coefficient_scale,
        "composition normalized affine coefficient a",
    )?;
    let normalized_b = checked_div(
        b,
        coefficient_scale,
        "composition normalized affine coefficient b",
    )?;
    let normalized_c = checked_div(
        c,
        coefficient_scale,
        "composition normalized affine coefficient c",
    )?;
    let normalized_d = checked_div(
        d,
        coefficient_scale,
        "composition normalized affine coefficient d",
    )?;
    let normalized_determinant = checked_sub(
        checked_mul(
            normalized_a,
            normalized_d,
            "composition normalized affine determinant ad",
        )?,
        checked_mul(
            normalized_b,
            normalized_c,
            "composition normalized affine determinant bc",
        )?,
        "composition normalized affine determinant",
    )?;
    if normalized_determinant == 0.0 {
        return Ok(None);
    }
    let inverse_denominator = checked_mul(
        coefficient_scale,
        normalized_determinant,
        "composition affine inverse denominator",
    )?;
    let inverse_a = checked_div(
        normalized_d,
        inverse_denominator,
        "composition inverse affine coefficient a",
    )?;
    let inverse_b = checked_div(
        -normalized_b,
        inverse_denominator,
        "composition inverse affine coefficient b",
    )?;
    let inverse_c = checked_div(
        -normalized_c,
        inverse_denominator,
        "composition inverse affine coefficient c",
    )?;
    let inverse_d = checked_div(
        normalized_a,
        inverse_denominator,
        "composition inverse affine coefficient d",
    )?;
    let inverse_e = checked_sub(
        0.0,
        checked_add(
            checked_mul(inverse_a, e, "composition inverse affine translation ae")?,
            checked_mul(inverse_c, f, "composition inverse affine translation cf")?,
            "composition inverse affine translation x",
        )?,
        "composition inverse affine translation x",
    )?;
    let inverse_f = checked_sub(
        0.0,
        checked_add(
            checked_mul(inverse_b, e, "composition inverse affine translation be")?,
            checked_mul(inverse_d, f, "composition inverse affine translation df")?,
            "composition inverse affine translation y",
        )?,
        "composition inverse affine translation y",
    )?;
    Ok(Some(DestinationToLayerLocalMapping {
        affine: Transform::try_new([
            inverse_a, inverse_b, inverse_c, inverse_d, inverse_e, inverse_f,
        ])?,
    }))
}

pub(super) fn mask_upload_spatial(
    image_dimensions: PhysicalSize,
) -> Result<NonEmptyFrameSpatialPlan> {
    let width = image_dimensions.width();
    let height = image_dimensions.height();
    let logical_bounds = non_empty_logical_bounds(
        Rect::new(0.0, 0.0, f64::from(width), f64::from(height)),
        "resolved mask image pixel bounds",
    )?;
    let device_origin = SignedDeviceOrigin::new(0, 0);
    let device_extent = PositiveDeviceExtent::try_new(width, height)?;
    let raster_scale = RasterScale::try_new(1.0)?;
    let texel_center_mapping = TexelCenterMapping::try_new(device_origin, raster_scale)?;
    Ok(NonEmptyFrameSpatialPlan {
        logical_bounds,
        device_origin,
        device_extent,
        raster_scale,
        texel_center_mapping,
    })
}

fn transform_point(transform: Transform, x: f64, y: f64, name: &str) -> Result<Point> {
    let [a, b, c, d, e, f] = transform.as_array();
    let transformed_x = checked_add(
        checked_add(
            checked_mul(a, x, &format!("{name} x product"))?,
            checked_mul(c, y, &format!("{name} x cross product"))?,
            &format!("{name} x linear value"),
        )?,
        e,
        &format!("{name} x"),
    )?;
    let transformed_y = checked_add(
        checked_add(
            checked_mul(b, x, &format!("{name} y cross product"))?,
            checked_mul(d, y, &format!("{name} y product"))?,
            &format!("{name} y linear value"),
        )?,
        f,
        &format!("{name} y"),
    )?;
    Point::try_new(transformed_x, transformed_y)
}

fn validate_finite(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() {
        return Err(Error::invalid_value(name, value, "must be finite"));
    }
    Ok(())
}

fn validate_non_negative(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() || value < 0.0 {
        return Err(Error::invalid_value(
            name,
            value,
            "must be finite and non-negative",
        ));
    }
    Ok(())
}

fn checked_add(left: f64, right: f64, name: &str) -> Result<f64> {
    checked_finite_result(left + right, name, left, "+", right)
}

fn checked_sub(left: f64, right: f64, name: &str) -> Result<f64> {
    checked_finite_result(left - right, name, left, "-", right)
}

pub(super) fn checked_mul(left: f64, right: f64, name: &str) -> Result<f64> {
    checked_finite_result(left * right, name, left, "*", right)
}

pub(super) fn checked_div(left: f64, right: f64, name: &str) -> Result<f64> {
    checked_finite_result(left / right, name, left, "/", right)
}

fn checked_finite_result(
    result: f64,
    name: &str,
    left: f64,
    operation: &'static str,
    right: f64,
) -> Result<f64> {
    if !result.is_finite() {
        return Err(Error::invalid_value(
            name,
            format!("{left} {operation} {right}"),
            "must produce a finite value",
        ));
    }
    Ok(result)
}

#[cfg(test)]
pub(crate) struct SpatialPrimitivesForTest {
    pub(crate) logical_and_device_phases_are_distinct: bool,
    pub(crate) logical_bounds: Option<[f64; 4]>,
    pub(crate) device_origin: Option<(i32, i32)>,
    pub(crate) device_extent: Option<(u32, u32)>,
    pub(crate) raster_scale: f64,
    pub(crate) texel_center: Option<(f64, f64)>,
    pub(crate) is_empty: bool,
}

#[cfg(test)]
pub(crate) fn transformed_logical_bounds_for_test(
    rect: Rect,
    transform: Transform,
) -> Result<[f64; 4]> {
    let transformed = LogicalBounds::try_from_rect(rect, "frame logical bounds")?
        .try_transform(transform, "frame transformed logical bounds")?
        .rect();
    Ok([
        transformed.x(),
        transformed.y(),
        transformed.width(),
        transformed.height(),
    ])
}

#[cfg(test)]
pub(crate) fn spatial_primitives_for_test(
    rect: Rect,
    transform: Transform,
    surface_scale: f64,
    texel: (u32, u32),
) -> Result<SpatialPrimitivesForTest> {
    let logical_bounds = LogicalBounds::try_from_rect(rect, "frame logical bounds")?;
    let plan = FrameContext::try_for_spatial_test(surface_scale)?
        .plan_local_bounds(logical_bounds, transform)?;
    let logical_rect = logical_bounds.rect();
    let logical_bounds = Some([
        logical_rect.x(),
        logical_rect.y(),
        logical_rect.width(),
        logical_rect.height(),
    ]);

    match plan {
        FrameSpatialPlan::Empty(plan) => {
            let _logical_bounds = plan.logical_bounds;
            Ok(SpatialPrimitivesForTest {
                logical_and_device_phases_are_distinct: true,
                logical_bounds,
                device_origin: None,
                device_extent: None,
                raster_scale: 0.0,
                texel_center: None,
                is_empty: true,
            })
        }
        FrameSpatialPlan::NonEmpty(plan) => {
            let _logical_bounds = plan.logical_bounds;
            let texel_center = plan.texel_center_mapping.point_for(texel.0, texel.1)?;
            Ok(SpatialPrimitivesForTest {
                logical_and_device_phases_are_distinct: true,
                logical_bounds,
                device_origin: Some((plan.device_origin.x, plan.device_origin.y)),
                device_extent: Some((plan.device_extent.width, plan.device_extent.height)),
                raster_scale: plan.raster_scale.get(),
                texel_center: Some((texel_center.x(), texel_center.y())),
                is_empty: false,
            })
        }
    }
}

#[cfg(test)]
mod operation_domain_tests {
    use super::*;
    use crate::{
        BlendMode, Capabilities, Layer, LayerComposition, PorterDuffOperator, Scene, Shape,
    };

    fn contribution(layer: Layer) -> SemanticSourceContribution {
        let account = Account::for_test();
        let mut scene = Scene::new();
        scene.layer(layer, |_| {});
        let commands = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        let context =
            FrameContext::try_new(Size::new(8.0, 6.0), 1.0, Antialiasing::Area, Color::BLACK)
                .unwrap();
        SemanticSourceContribution::try_from_commands(
            commands,
            context.initial_parent_contribution(),
            SemanticContributionDomain::RootOutputBounded(context.output_bounds),
            context,
            Transform::identity(),
            &account,
        )
        .unwrap()
    }

    fn selected(operator: PorterDuffOperator) -> Layer {
        Layer::new().with_composition(
            LayerComposition::try_porter_duff(operator, BlendMode::Normal).unwrap(),
        )
    }

    #[test]
    fn source_in_support_uses_the_group_backdrop_instead_of_source_ink_alone() {
        let account = Account::for_test();
        let context =
            FrameContext::try_new(Size::new(8.0, 6.0), 1.0, Antialiasing::Area, Color::BLACK)
                .unwrap();
        for isolated in [false, true] {
            let mut scene = Scene::new();
            let wrapper = if isolated {
                Layer::new().isolated()
            } else {
                Layer::new()
            };
            scene.layer(wrapper, |scene| {
                scene.layer(selected(PorterDuffOperator::SourceIn), |scene| {
                    scene.fill(Rect::new(2.0, 1.0, 3.0, 2.0), Color::BLACK);
                });
            });
            let contribution = SemanticSourceContribution::try_from_commands(
                scene.normalize(Capabilities::CURRENT, &account).unwrap(),
                context.initial_parent_contribution(),
                SemanticContributionDomain::RootOutputBounded(context.output_bounds),
                context,
                Transform::identity(),
                &account,
            )
            .unwrap();
            // An isolated first SourceIn has no destination support; a default
            // wrapper forwards the opaque current image into the same child.
            assert_eq!(contribution.source_bounds.is_exactly_empty(), isolated);
        }
    }

    #[test]
    fn empty_destructive_sources_retain_the_finite_parent_operation_domain() {
        for operator in [
            PorterDuffOperator::Clear,
            PorterDuffOperator::Copy,
            PorterDuffOperator::SourceIn,
            PorterDuffOperator::DestinationIn,
            PorterDuffOperator::SourceOut,
            PorterDuffOperator::DestinationAtop,
        ] {
            let contribution = contribution(selected(operator));
            assert!(contribution.source_bounds.is_exactly_empty());
            assert_eq!(
                contribution
                    .affected_bounds
                    .require_non_empty_for_graph("test domain")
                    .unwrap()
                    .unwrap()
                    .rect(),
                Rect::new(0.0, 0.0, 8.0, 6.0),
            );
            assert!(!contribution.commands.is_empty());
        }
    }

    #[test]
    fn singular_source_keeps_copy_domain_but_singular_own_clip_has_no_coverage() {
        let layer =
            selected(PorterDuffOperator::Copy).with_transform(Transform::scale(0.0, 1.0).unwrap());
        let unclipped = contribution(layer.clone());
        assert!(unclipped.source_bounds.is_exactly_empty());
        assert_eq!(
            unclipped
                .affected_bounds
                .require_non_empty_for_graph("test domain")
                .unwrap()
                .unwrap()
                .rect(),
            Rect::new(0.0, 0.0, 8.0, 6.0),
        );
        let clipped = contribution(
            layer
                .try_clip(Shape::rect(Rect::new(0.0, 0.0, 4.0, 4.0)))
                .unwrap(),
        );
        assert!(clipped.affected_bounds.is_exactly_empty());
        assert!(clipped.commands.is_empty());
    }

    #[test]
    fn translating_an_empty_source_transforms_its_clip_and_preserves_parent_domain() {
        let unclipped = selected(PorterDuffOperator::Copy)
            .with_transform(Transform::translation(2.0, 1.0).unwrap());
        assert_eq!(
            contribution(unclipped.clone())
                .affected_bounds
                .require_non_empty_for_graph("test domain")
                .unwrap()
                .unwrap()
                .rect(),
            Rect::new(0.0, 0.0, 8.0, 6.0),
        );
        let clipped = unclipped
            .try_clip(Shape::rect(Rect::new(0.0, 0.0, 3.0, 2.0)))
            .unwrap();
        assert_eq!(
            contribution(clipped)
                .affected_bounds
                .require_non_empty_for_graph("test domain")
                .unwrap()
                .unwrap()
                .rect(),
            Rect::new(2.0, 1.0, 3.0, 2.0),
        );
    }
}

#[cfg(test)]
mod command_contribution_tests {
    use super::*;
    use crate::{
        Capabilities, FontData, FontRef, GlyphRunLimits, GlyphSynthesis, Layer,
        RenderAccountingContext, Scene, TextGlyph, TextInstance, TextPaint, TextRun, TextRunBounds,
    };

    fn context() -> FrameContext {
        FrameContext::try_new(
            Size::new(32.0, 32.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap()
    }

    fn contribute(
        commands: RenderCommands,
        account: &Account,
    ) -> Result<SemanticSourceContribution> {
        let context = context();
        SemanticSourceContribution::try_from_commands(
            commands,
            context.initial_parent_contribution(),
            SemanticContributionDomain::RootOutputBounded(context.output_bounds),
            context,
            Transform::identity(),
            account,
        )
    }

    #[test]
    fn layer_bounds_composition_admits_coordinates_before_contributing() {
        use crate::{
            RenderAccountingError, RenderAccountingOptions, RenderAttemptKind, RenderLimits,
            RenderPhase, RenderQuantity,
        };

        let mut scene = Scene::new();
        scene.layer(
            Layer::new().with_transform(Transform::translation(4.0, 6.0).unwrap()),
            |scene| {
                scene.fill(Rect::new(1.0, 2.0, 2.0, 3.0), Color::BLACK);
            },
        );
        let commands = scene
            .normalize(Capabilities::CURRENT, &Account::for_test())
            .unwrap();
        for (cap, amount) in [(0, 12), (31, 1), (32, 1), (37, 1)] {
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::CoordinateVisits, cap)
                .unwrap();
            let mut accounting =
                RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
            let error = accounting
                .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                    contribute(commands.clone(), account).map(drop)
                })
                .expect_err("the layer product must refuse before contributing its bounds");
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("expected coordinate refusal: {error:?}");
            };
            assert_eq!(
                refusal.quantity(),
                RenderQuantity::Work(RenderWork::CoordinateVisits)
            );
            assert_eq!((refusal.current(), refusal.amount()), (cap, amount));
            let report = accounting.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Plan)
                    .work(RenderWork::CoordinateVisits),
                cap
            );
        }
        let result = contribute(commands, &Account::for_test()).unwrap();
        assert_eq!(
            result
                .source_bounds
                .known_non_empty_extent()
                .unwrap()
                .rect(),
            Rect::new(5.0, 8.0, 2.0, 3.0)
        );
    }

    fn add_text(scene: &mut Scene, bounds: TextRunBounds, glyphs: &[TextGlyph]) {
        scene.text_run(
            TextRun::try_new(
                FontRef::new(71_u64).named("contribution text"),
                10.0,
                Transform::translation(2.0, 3.0).unwrap(),
                TextPaint::try_fill(Color::BLACK.into()).unwrap(),
                glyphs,
                bounds,
            )
            .unwrap(),
        );
    }

    #[test]
    fn native_text_preserves_transformed_ink_unresolved_state_and_shared_payloads() {
        let glyphs = [TextGlyph::try_new(58, 0.0, 8.0, 10.0).unwrap()];
        for (bounds, empty_glyphs, retained, unresolved) in [
            (
                TextRunBounds::try_ink(Rect::new(1.0, 2.0, 4.0, 5.0)).unwrap(),
                false,
                true,
                false,
            ),
            (TextRunBounds::empty(), false, false, false),
            (TextRunBounds::unspecified(), false, true, true),
            (TextRunBounds::unspecified(), true, false, false),
        ] {
            let account = Account::for_test();
            let mut scene = Scene::new();
            add_text(&mut scene, bounds, if empty_glyphs { &[] } else { &glyphs });
            let normalized = scene.normalize(Capabilities::CURRENT, &account).unwrap();
            let actual = contribute(normalized.clone(), &account).unwrap();
            assert_eq!(actual.commands.len(), usize::from(retained));
            assert_eq!(
                actual.source_bounds.contains_unresolved_content(),
                unresolved
            );
            if retained {
                let (
                    RenderCommand::TextRun {
                        font,
                        paint,
                        glyphs,
                        ..
                    },
                    RenderCommand::TextRun {
                        font: original_font,
                        paint: original_paint,
                        glyphs: original_glyphs,
                        ..
                    },
                ) = (&actual.commands[0], &normalized[0])
                else {
                    panic!("retained text must preserve its complete normalized payload");
                };
                assert_eq!(actual.commands[0], normalized[0]);
                assert!(std::ptr::eq(&**font, &**original_font));
                assert!(std::ptr::eq(&**paint, &**original_paint));
                assert!(std::ptr::eq(&**glyphs, &**original_glyphs));
                if !unresolved {
                    assert_eq!(
                        actual
                            .source_bounds
                            .known_non_empty_extent()
                            .unwrap()
                            .rect(),
                        Rect::new(3.0, 5.0, 4.0, 5.0)
                    );
                }
            } else {
                assert!(actual.source_bounds.is_exactly_empty());
            }
        }
    }

    #[test]
    fn prepared_ink_preserves_transformed_geometry_and_original_preparation() {
        let font = FontData::try_from_bytes(
            include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let instance = TextInstance::try_new(10.0, vec![], GlyphSynthesis::none()).unwrap();
        let run = font
            .prepare_glyph_run(
                &instance,
                &[TextGlyph::try_new(58, 0.0, 8.0, 10.0).unwrap()],
                GlyphRunLimits::new(1, 16, 1 << 20, 1 << 20),
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
        let account = Account::for_test();
        let mut scene = Scene::new();
        scene.prepared_glyph_run(
            run.clone(),
            Transform::translation(2.0, 3.0).unwrap(),
            TextPaint::try_fill(Color::BLACK.into()).unwrap(),
        );
        let normalized = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        let actual = contribute(normalized.clone(), &account).unwrap();
        assert_eq!(
            actual
                .source_bounds
                .known_non_empty_extent()
                .unwrap()
                .rect(),
            Rect::new(2.0, 3.0, 10.0, 10.0)
        );
        assert_eq!(actual.commands, normalized);
        let (
            RenderCommand::PreparedGlyphRun {
                run: retained,
                paint,
                ..
            },
            RenderCommand::PreparedGlyphRun {
                run: original,
                paint: original_paint,
                ..
            },
        ) = (&actual.commands[0], &normalized[0])
        else {
            panic!("prepared contribution must retain its normalized payload");
        };
        assert!(retained.is_same_preparation(&run));
        assert!(std::ptr::eq(&**retained, &**original));
        assert!(std::ptr::eq(&**paint, &**original_paint));
    }

    #[test]
    fn nested_contributions_keep_exact_visit_prefix_order_and_fresh_retry_across_profiles() {
        use crate::accounting::{
            FrameFixture, RenderAccountingError, RenderAccountingOptions, RenderLimits,
            RenderPhase, RenderProfile, RenderQuantity, RenderRefusalReason, RendererCpu,
        };
        let mut scene = Scene::new();
        scene.fill(Rect::new(1.0, 1.0, 2.0, 2.0), Color::BLACK);
        scene.layer(Layer::new().isolated(), |scene| {
            scene.fill(Rect::new(5.0, 4.0, 3.0, 2.0), Color::BLACK);
        });
        scene.fill(Rect::new(10.0, 1.0, 2.0, 2.0), Color::BLACK);
        let original_account = Account::for_test();
        let normalized = scene
            .normalize(Capabilities::CURRENT, &original_account)
            .unwrap();
        // Three root dispatches, one child scan, one child dispatch, and three fill-bound visits.
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Summary { timing: true },
        ] {
            let options = RenderAccountingOptions::new().with_profile(profile);
            let cpu = RendererCpu::new(options).unwrap();
            for limit in [7, 8] {
                let bounded = options.with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::CommandVisits, limit)
                        .unwrap(),
                );
                let (_, _, report, fault) = FrameFixture::new(&cpu, bounded)
                    .run_sync(|attempt| {
                        let result = contribute(
                            normalized.clone(),
                            &attempt.account().at(RenderPhase::Plan),
                        );
                        assert_eq!(
                            attempt
                                .account()
                                .work_count(RenderWork::CommandVisits)
                                .unwrap(),
                            limit
                        );
                        if limit == 7 {
                            let error = result.unwrap_err();
                            let Some(RenderAccountingError::Refused(refusal)) =
                                error.accounting_diagnostic()
                            else {
                                panic!("the last fill bounds visit must refuse");
                            };
                            // The configured work limit belongs to the root attempt;
                            // work is still submitted from the Plan account.
                            assert_eq!(refusal.phase(), RenderPhase::Attempt);
                            assert_eq!(
                                refusal.quantity(),
                                RenderQuantity::Work(RenderWork::CommandVisits)
                            );
                            assert_eq!(refusal.current(), 7);
                            assert_eq!(refusal.amount(), 1);
                            assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 7 });
                            Err(error)
                        } else {
                            let actual = result.unwrap();
                            assert_eq!(actual.commands, normalized);
                            assert_eq!(
                                actual
                                    .source_bounds
                                    .known_non_empty_extent()
                                    .unwrap()
                                    .rect(),
                                Rect::new(1.0, 1.0, 11.0, 5.0)
                            );
                            drop(actual);
                            Ok(())
                        }
                    })
                    .unwrap()
                    .into_parts();
                assert_eq!(fault, None);
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::CommandVisits),
                    limit
                );
                assert_eq!(cpu.snapshot().unwrap().fault(), None);
            }
        }
        assert_eq!(scene.len(), 3);
    }
}
