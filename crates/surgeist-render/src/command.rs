#[cfg(test)]
use super::stats::collect_render_stats;
use super::{
    frame::{FrameContext, FramePlan, LogicalBounds},
    geometry::offset_radii,
    image::{ResolvedImageRegion, ResolvedMaskUploadDescriptor},
    layer::{LayerFilter, MaskApplication, MaskValue},
    paint::PaintKind,
    scene::Command,
    shape::ShapeKind,
    style::{ClipGeometryKind, FilterBlur, FilterList, FilterOp},
    validation::*,
    *,
};
use crate::accounting::{
    Account, AccountedIntoIter, AccountedVec, CpuImport, DisposalCompletion, Hold, OwnedBox,
    RenderPhase, RenderWork, SharedValue,
};
use kurbo::Shape as KurboShape;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderCommands {
    pub(crate) commands: SharedCommandSequence,
}

/// The final shared owner walks existing sequence allocations to dispose of
/// descendants. Teardown requires neither new backing nor a forward budget.
#[derive(Clone)]
pub(crate) struct SharedCommandSequence(Option<SharedValue<Option<OwnedBox<CommandSequence>>>>);

struct CommandSequence {
    commands: AccountedIntoIter<RenderCommand>,
    parent: Option<OwnedBox<Self>>,
}

impl SharedCommandSequence {
    fn new(commands: AccountedVec<RenderCommand>, account: &Account) -> Result<Self> {
        let storage = account.storage();
        let node = OwnedBox::new(
            CommandSequence {
                commands: commands.into_iter(),
                parent: None,
            },
            &storage,
        )?;
        Ok(Self(Some(SharedValue::new(Some(node), &storage)?)))
    }

    pub(crate) fn as_slice(&self) -> &[RenderCommand] {
        self.0
            .as_deref()
            .and_then(Option::as_ref)
            .map_or(&[], |node| node.commands.as_slice())
    }

    pub(crate) fn same_allocation(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(left), Some(right)) => left.same_allocation(right),
            _ => false,
        }
    }

    fn take_last_owner(&mut self) -> Option<OwnedBox<CommandSequence>> {
        self.0.take()?.take_last_box()
    }
}

impl std::ops::Deref for SharedCommandSequence {
    type Target = [RenderCommand];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::fmt::Debug for SharedCommandSequence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(formatter)
    }
}

impl PartialEq for SharedCommandSequence {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Drop for SharedCommandSequence {
    fn drop(&mut self) {
        let Some(mut owner) = self.take_last_owner() else {
            return;
        };
        owner.begin_disposal();
        loop {
            match owner.commands.next() {
                Some(RenderCommand::Layer {
                    layer,
                    mut children,
                }) => {
                    drop(layer);
                    if let Some(mut child) = children.commands.take_last_owner() {
                        child.begin_disposal();
                        child.parent = Some(owner);
                        owner = child;
                    }
                }
                Some(_) => {}
                None => {
                    let Some(next) = owner.parent.take() else {
                        break;
                    };
                    owner = next;
                }
            }
        }
    }
}

impl std::ops::Deref for RenderCommands {
    type Target = [RenderCommand];
    fn deref(&self) -> &Self::Target {
        self.commands.as_slice()
    }
}
impl<'a> IntoIterator for &'a RenderCommands {
    type Item = &'a RenderCommand;
    type IntoIter = std::slice::Iter<'a, RenderCommand>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl RenderCommands {
    pub(crate) fn new_in(commands: AccountedVec<RenderCommand>, account: &Account) -> Result<Self> {
        Ok(Self {
            commands: SharedCommandSequence::new(commands, account)?,
        })
    }
    pub(crate) fn plan_for(self, context: FrameContext, account: &Account) -> Result<FramePlan> {
        FramePlan::try_from_commands(self, context, account)
    }
    #[cfg(test)]
    #[must_use]
    pub(crate) fn stats(&self) -> Stats {
        let mut stats = Stats::default();
        let mut uploaded_images = std::collections::HashSet::new();
        collect_render_stats(&self.commands, &mut stats, &mut uploaded_images);
        stats
    }
}

pub(crate) fn shared<T>(
    value: T,
    hold: Option<Hold>,
    import: Option<CpuImport>,
    account: &Account,
) -> Result<SharedValue<T>> {
    SharedValue::with_parts(value, hold, import, &account.storage())
}
fn shared_path(path: &Path, account: &Account) -> Result<SharedValue<Path>> {
    let (path, hold) = path.try_clone_in(account)?;
    shared(path, Some(hold), None, account)
}
fn shared_paint(
    paint: &Paint,
    capabilities: Capabilities,
    account: &Account,
) -> Result<SharedValue<TextPaint>> {
    ensure_paint_supported(paint, capabilities)?;
    let (paint, hold) = paint.try_clone_in(account)?;
    let import = if let PaintKind::Image(image) = paint.kind() {
        account.import_image(image)?
    } else {
        None
    };
    shared(TextPaint::from_validated_fill(paint), hold, import, account)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RenderCommand {
    PreparedGlyphRun {
        run: SharedValue<PreparedGlyphRun>,
        transform: Transform,
        paint: SharedValue<TextPaint>,
    },
    Fill {
        shape: RenderShape,
        paint: RenderPaint,
    },
    Stroke {
        shape: RenderStrokeShape,
        stroke: RenderStroke,
        paint: RenderPaint,
    },
    Shadow {
        shape: ShadowShape,
        shadow: RenderShadow,
    },
    Image {
        image: SharedValue<Image>,
        rect: Rect,
        fit: ImageFit,
    },
    ImageRegion {
        image: SharedValue<Image>,
        rect: Rect,
        region: ResolvedImageRegion,
    },
    TextRun {
        font: SharedValue<FontRef<'static>>,
        size: f32,
        transform: Transform,
        paint: SharedValue<TextPaint>,
        glyphs: SharedValue<Vec<TextGlyph>>,
        bounds: TextRunBounds,
    },
    Layer {
        layer: SharedValue<NormalizedLayer>,
        children: RenderCommands,
    },
}

impl RenderCommand {
    /// Owns actual immutable layer metadata under central backing admission.
    /// The recursive child sequence retains its existing iterative final owner.
    pub(crate) fn layer_in(
        layer: NormalizedLayer,
        children: RenderCommands,
        account: &Account,
    ) -> Result<Self> {
        Ok(Self::Layer {
            layer: SharedValue::new(layer, &account.storage())?,
            children,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RenderShape {
    Rect(Rect),
    RoundedRect { rect: Rect, radii: Radii },
    Circle { center: Point, radius: f64 },
    Ellipse { center: Point, radii: Size },
    Path(SharedValue<Path>),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderClip {
    geometry: RenderClipGeometry,
    coordinate_space: Option<CoordinateSpaceTag>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RenderClipGeometry {
    Rect(Rect),
    RoundedRect {
        rect: Rect,
        radii: Radii,
    },
    Circle {
        center: Point,
        radius: f64,
    },
    Ellipse {
        center: Point,
        radii: Size,
    },
    Path {
        path: SharedValue<Path>,
        fill_rule: FillRule,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RenderStrokeShape {
    Rect(kurbo::Rect),
    RoundedRect(kurbo::RoundedRect),
    Circle(kurbo::Circle),
    Ellipse(kurbo::Ellipse),
    Path(SharedValue<kurbo::BezPath>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RenderStroke {
    pub(crate) width: f64,
    pub(crate) join: LineJoin,
    pub(crate) start_cap: LineCap,
    pub(crate) end_cap: LineCap,
    pub(crate) miter_limit: f64,
    pub(crate) dash: Option<Dash>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RenderPaint {
    Color(Color),
    Gradient(SharedValue<Gradient>),
    RepeatingGradient(SharedValue<RepeatingGradient>),
    Image(SharedValue<Image>),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ShadowShape {
    Rect(Rect),
    RoundedRect { rect: Rect, radii: Radii },
    Circle { center: Point, radius: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderShadow {
    pub(crate) offset: Point,
    pub(crate) blur: f64,
    pub(crate) spread: f64,
    pub(crate) color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NormalizedLayer {
    pub(crate) clip: Option<RenderClip>,
    pub(crate) transform: Transform,
    pub(crate) opacity: f32,
    pub(crate) composition: LayerComposition,
    pub(crate) explicit_isolation: bool,
    pub(crate) mask: Option<RenderLayerMask>,
    pub(crate) foreground_filters: Option<NormalizedForegroundEffect>,
    pub(crate) backdrop: Option<SharedValue<RenderBackdropCapture>>,
    pub(crate) isolation: LayerIsolation,
    pub(crate) pass_plan: LayerPassPlan,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NormalizedForegroundEffect {
    Css(SharedValue<FilterList>),
    Convolution(crate::PreparedConvolution),
    Offset(crate::PreparedOffset),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderLayerMask {
    bounds: Rect,
    upload: SharedValue<ResolvedMaskUploadDescriptor>,
    annihilates_source: bool,
    value: MaskValue,
    application: MaskApplication,
}

impl RenderLayerMask {
    fn from_resolved_in(
        image: &Image,
        bounds: Rect,
        value: MaskValue,
        application: MaskApplication,
        capabilities: Capabilities,
        account: &Account,
    ) -> Result<Self> {
        capabilities.ensure_supported(UnsupportedPrimitive::new(
            PrimitiveFamily::MasksAndClips,
            PrimitiveOperation::ResolvedAlphaMaskExecution,
        ))?;
        let image_size = image.size();
        let annihilates_source = image_size.width() == 0.0 || image_size.height() == 0.0;
        account.work(RenderWork::ImageIdentityCopies, 1)?;
        let upload = ResolvedMaskUploadDescriptor::try_from_image(image.clone())?;
        let upload = shared(upload, None, account.import_image(image)?, account)?;
        Ok(Self {
            bounds,
            upload,
            annihilates_source,
            value,
            application,
        })
    }

    pub(crate) const fn value(&self) -> MaskValue {
        self.value
    }

    pub(crate) const fn application(&self) -> MaskApplication {
        self.application
    }

    pub(crate) fn supplies_coverage(&self) -> bool {
        self.application == MaskApplication::CoverageSource
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn image(&self) -> &Image {
        self.upload.image()
    }

    #[must_use]
    pub(crate) const fn bounds(&self) -> Rect {
        self.bounds
    }

    #[must_use]
    pub(crate) fn upload(&self) -> &ResolvedMaskUploadDescriptor {
        &self.upload
    }
    pub(crate) fn shared_upload(&self) -> SharedValue<ResolvedMaskUploadDescriptor> {
        self.upload.clone()
    }

    #[must_use]
    pub(crate) const fn annihilates_source(&self) -> bool {
        self.annihilates_source
    }

    #[cfg(test)]
    pub(crate) const fn semantic_bounds_for_contract_test(&self) -> Rect {
        self.bounds
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderBackdropCapture {
    filters: SharedValue<FilterList>,
    capture_bounds: OffscreenBounds,
    clip: Option<RenderClip>,
}

impl RenderBackdropCapture {
    fn from_input_in(
        input: &BackdropFilterInput,
        capabilities: Capabilities,
        account: &Account,
    ) -> Result<SharedValue<Self>> {
        input.ensure_supported_for_planning_in(capabilities, account)?;
        let capture_bounds = OffscreenBounds::try_new(input.capture_bounds().rect())?;
        let clip = input
            .clip()
            .map(|clip| RenderClip::from_input_in(clip, capabilities, account))
            .transpose()?;
        let capture = Self {
            filters: {
                let (filters, hold) = input.filters().try_clone_in(account)?;
                shared(filters, hold, None, account)?
            },
            capture_bounds,
            clip,
        };
        shared(capture, None, None, account)
    }

    #[must_use]
    pub(crate) fn filters(&self) -> &FilterList {
        &self.filters
    }

    #[must_use]
    pub(crate) const fn capture_bounds(&self) -> OffscreenBounds {
        self.capture_bounds
    }

    #[must_use]
    pub(crate) const fn clip(&self) -> Option<&RenderClip> {
        self.clip.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LayerIsolation {
    None,
    ClipOnly,
    BackendLayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LayerPassRequirement {
    None,
    ClipOnly,
    DirectVelloOpacity,
    DirectVelloBlend,
    DirectVelloOpacityBlend,
    BoundedBackdropCapture,
    OffscreenTexture(PrimitiveOperation),
    DiagnosticBoundary(PrimitiveOperation),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LayerPassKind {
    None,
    ClipOnly,
    DirectVelloLayer,
    OffscreenTexture,
    DiagnosticBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OffscreenBounds {
    logical_bounds: LogicalBounds,
}

impl OffscreenBounds {
    pub(crate) fn try_new(rect: Rect) -> Result<Self> {
        Ok(Self {
            logical_bounds: LogicalBounds::try_from_rect(rect, "offscreen layer bounds")?,
        })
    }

    #[must_use]
    const fn from_logical(logical_bounds: LogicalBounds) -> Self {
        Self { logical_bounds }
    }

    #[must_use]
    pub(crate) const fn rect(self) -> Rect {
        self.logical_bounds.rect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayerPassPlan {
    requirement: LayerPassRequirement,
    kind: LayerPassKind,
    bounds: Option<OffscreenBounds>,
}

impl LayerPassPlan {
    fn new(
        requirement: LayerPassRequirement,
        kind: LayerPassKind,
        bounds: Option<OffscreenBounds>,
    ) -> Self {
        Self {
            requirement,
            kind,
            bounds,
        }
    }

    fn from_authored(
        layer: &Layer,
        clip: Option<&RenderClip>,
        children: &[RenderCommand],
        capabilities: Capabilities,
        account: &Account,
    ) -> Result<Self> {
        if let Some(unsupported) = unsupported_layer_effect(layer) {
            return Self::diagnostic_boundary(unsupported);
        }
        let outer_clip_bounds = clip_bounds(clip, account)?;
        if let Some(backdrop) = layer.backdrop_filter() {
            let bounds = match outer_clip_bounds {
                Some(bounds) => bounds,
                None => OffscreenBounds::try_new(backdrop.capture_bounds().rect())?,
            };
            return Ok(Self::new(
                LayerPassRequirement::BoundedBackdropCapture,
                LayerPassKind::OffscreenTexture,
                Some(bounds),
            ));
        }
        let bounds = match outer_clip_bounds {
            Some(bounds) => Some(bounds),
            None => commands_bounds(children, account)?,
        };

        let composition = layer.composition();
        if !composition.is_source_over() && !composition.is_plus() {
            let operation =
                if composition.operator() == Some(crate::PorterDuffOperator::PlusLighter) {
                    PrimitiveOperation::PlusLighterComposition
                } else {
                    PrimitiveOperation::BasicPorterDuffCompositeMode
                };
            capabilities.ensure_supported(UnsupportedPrimitive::new(
                PrimitiveFamily::Compositing,
                operation,
            ))?;
            return Ok(Self::new(
                LayerPassRequirement::OffscreenTexture(operation),
                LayerPassKind::OffscreenTexture,
                bounds,
            ));
        }
        if layer.foreground_filter().is_some() {
            capabilities.ensure_supported(UnsupportedPrimitive::new(
                PrimitiveFamily::OffscreenPipeline,
                PrimitiveOperation::LayerFilterExecution,
            ))?;
            return Ok(Self::new(
                LayerPassRequirement::OffscreenTexture(PrimitiveOperation::LayerFilterExecution),
                LayerPassKind::OffscreenTexture,
                bounds,
            ));
        }
        if layer.is_isolated() {
            capabilities.ensure_supported(UnsupportedPrimitive::new(
                PrimitiveFamily::Compositing,
                PrimitiveOperation::IsolatedLayerExecution,
            ))?;
            return Ok(Self::new(
                LayerPassRequirement::OffscreenTexture(PrimitiveOperation::IsolatedLayerExecution),
                LayerPassKind::OffscreenTexture,
                bounds,
            ));
        }

        let has_clip = clip.is_some();
        let has_resolved_mask = layer.resolved_image_mask().is_some();
        let opacity_delta = (layer.opacity() - 1.0).abs();
        let opacity_is_clip_identity = opacity_delta < f32::EPSILON;
        let has_opacity = opacity_delta > f32::EPSILON;
        let has_blend = layer.blend_mode() != BlendMode::Normal;

        if has_resolved_mask {
            capabilities.ensure_supported(UnsupportedPrimitive::new(
                PrimitiveFamily::MasksAndClips,
                PrimitiveOperation::ResolvedAlphaMaskExecution,
            ))?;
            return Ok(Self::new(
                LayerPassRequirement::OffscreenTexture(
                    PrimitiveOperation::ResolvedAlphaMaskExecution,
                ),
                LayerPassKind::OffscreenTexture,
                bounds,
            ));
        }

        if has_clip && opacity_is_clip_identity && !has_blend {
            return Ok(Self::new(
                LayerPassRequirement::ClipOnly,
                LayerPassKind::ClipOnly,
                bounds,
            ));
        }

        if has_clip || has_opacity || has_blend {
            let requirement = match (has_opacity, has_blend) {
                (true, true) => LayerPassRequirement::DirectVelloOpacityBlend,
                (true, false) => LayerPassRequirement::DirectVelloOpacity,
                (false, true) => LayerPassRequirement::DirectVelloBlend,
                (false, false) => LayerPassRequirement::DirectVelloOpacity,
            };
            if direct_vello_layer_supported(requirement, capabilities) {
                return Ok(Self::new(
                    requirement,
                    LayerPassKind::DirectVelloLayer,
                    bounds,
                ));
            }
            return Self::future_offscreen(
                PrimitiveOperation::OffscreenLayerRendering,
                bounds,
                capabilities,
            );
        }

        Ok(Self::new(
            LayerPassRequirement::None,
            LayerPassKind::None,
            bounds,
        ))
    }

    fn diagnostic_boundary(unsupported: UnsupportedPrimitive) -> Result<Self> {
        let _boundary = Self::new(
            LayerPassRequirement::DiagnosticBoundary(unsupported.operation()),
            LayerPassKind::DiagnosticBoundary,
            None,
        );
        Err(Error::unsupported_render_primitive(unsupported))
    }

    fn future_offscreen(
        operation: PrimitiveOperation,
        bounds: Option<OffscreenBounds>,
        capabilities: Capabilities,
    ) -> Result<Self> {
        capabilities.ensure_supported(UnsupportedPrimitive::new(
            PrimitiveFamily::OffscreenPipeline,
            operation,
        ))?;
        capabilities.ensure_supported(UnsupportedPrimitive::new(
            PrimitiveFamily::OffscreenPipeline,
            PrimitiveOperation::OffscreenLayerRendering,
        ))?;
        let bounds = bounds.ok_or_else(|| {
            Error::invalid_value(
                "offscreen layer bounds",
                "unknown",
                "must be explicit for offscreen texture passes",
            )
        })?;
        Ok(Self::new(
            LayerPassRequirement::OffscreenTexture(operation),
            LayerPassKind::OffscreenTexture,
            Some(bounds),
        ))
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn requirement(self) -> LayerPassRequirement {
        self.requirement
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn kind(self) -> LayerPassKind {
        self.kind
    }

    #[must_use]
    pub(crate) const fn bounds(self) -> Option<OffscreenBounds> {
        self.bounds
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn requires_offscreen_texture(self) -> bool {
        matches!(self.kind, LayerPassKind::OffscreenTexture)
    }

    #[must_use]
    const fn isolation(self) -> LayerIsolation {
        match self.kind {
            LayerPassKind::None => LayerIsolation::None,
            LayerPassKind::ClipOnly => LayerIsolation::ClipOnly,
            LayerPassKind::DirectVelloLayer
            | LayerPassKind::OffscreenTexture
            | LayerPassKind::DiagnosticBoundary => LayerIsolation::BackendLayer,
        }
    }
}

pub(crate) fn normalize_commands(
    commands: &[Command],
    capabilities: Capabilities,
    account: &Account,
) -> Result<RenderCommands> {
    let account = account.at(RenderPhase::Normalize);
    account.phase(RenderPhase::Normalize, || {
        normalize_commands_in_context(commands, capabilities, &account)
    })
}

struct PendingLayer<'a> {
    layer: &'a Layer,
    siblings: &'a [Command],
    parent: AccountedVec<RenderCommand>,
}

fn normalize_commands_in_context(
    mut commands: &[Command],
    capabilities: Capabilities,
    account: &Account,
) -> Result<RenderCommands> {
    let mut normalized = account.retained(commands.len())?;
    let mut pending: AccountedVec<PendingLayer<'_>> = account.temporary(0)?;
    loop {
        let value = if let Some((command, siblings)) = commands.split_first() {
            commands = siblings;
            account.work(RenderWork::CommandVisits, 1)?;
            if let Command::Layer { layer, children } = command {
                validate_layer_in(layer, account)?;
                reject_unsupported_layer_effect(layer)?;
                if layer.backdrop_filter().is_some() {
                    if !pending.is_empty() {
                        return Err(nested_backdrop_capture_error());
                    }
                    if layer.transform() != Transform::identity() {
                        return Err(transformed_backdrop_capture_error());
                    }
                    if commands_contain_backdrop_capture(&normalized, account)? {
                        return Err(repeated_top_level_backdrop_capture_error());
                    }
                }
                if pending.len() == pending.capacity() {
                    pending.grow(pending.capacity().max(1))?;
                }
                let child_output = account.retained(children.len())?;
                pending.push(PendingLayer {
                    layer,
                    siblings,
                    parent: std::mem::replace(&mut normalized, child_output),
                })?;
                commands = children;
                continue;
            }
            normalize_leaf(command, capabilities, account)?
        } else {
            let children = RenderCommands::new_in(normalized, account)?;
            let Some(PendingLayer {
                layer,
                siblings,
                parent,
            }) = pending.pop()
            else {
                return Ok(children);
            };
            commands = siblings;
            normalized = parent;
            RenderCommand::layer_in(
                NormalizedLayer::from_authored_in(layer, &children, capabilities, account)?,
                children,
                account,
            )?
        };
        emit_command(&value, account)?;
        normalized.push(value)?;
    }
}

fn normalize_leaf(
    command: &Command,
    capabilities: Capabilities,
    account: &Account,
) -> Result<RenderCommand> {
    Ok(match command {
        Command::Fill { shape, paint } => RenderCommand::Fill {
            shape: RenderShape::from_authored_in(shape, account)?,
            paint: RenderPaint::from_authored_in(paint, capabilities, account)?,
        },
        Command::Stroke {
            shape,
            stroke,
            paint,
        } => RenderCommand::Stroke {
            shape: RenderStrokeShape::from_authored_in(shape, *stroke, account)?,
            stroke: RenderStroke::try_from(*stroke)?,
            paint: RenderPaint::from_authored_in(paint, capabilities, account)?,
        },
        Command::Shadow { shape, shadow } => RenderCommand::Shadow {
            shape: ShadowShape::from_authored(shape, account)?,
            shadow: RenderShadow::from_authored(shadow, account)?,
        },
        Command::Image { image, rect, fit } => {
            validate_rect(*rect, "image target rectangle")?;
            account.work(RenderWork::ImageIdentityCopies, 1)?;
            RenderCommand::Image {
                image: shared(image.clone(), None, account.import_image(image)?, account)?,
                rect: *rect,
                fit: *fit,
            }
        }
        Command::ImageRegion {
            image,
            source,
            rect,
        } => {
            validate_rect(*rect, "image target rectangle")?;
            let region = ResolvedImageRegion::try_new(image, *source, account)?;
            account.work(RenderWork::ImageIdentityCopies, 1)?;
            RenderCommand::ImageRegion {
                image: shared(image.clone(), None, account.import_image(image)?, account)?,
                rect: *rect,
                region,
            }
        }
        Command::PreparedGlyphRun {
            run,
            transform,
            paint,
        } => {
            validate_transform(*transform, "prepared outline transform")?;
            validate_paint_in(paint.fill(), account)?;
            RenderCommand::PreparedGlyphRun {
                run: shared(run.clone(), None, account.import_prepared(run)?, account)?,
                transform: *transform,
                paint: shared_paint(paint.fill(), capabilities, account)?,
            }
        }
        Command::TextRun {
            font,
            size,
            transform,
            paint,
            glyphs,
            bounds,
        } => {
            validate_text_run_in(*size, *transform, glyphs, account)?;
            validate_paint_in(paint.fill(), account)?;
            let mut completion = DisposalCompletion::default();
            let (copied_font, hold) = font.try_to_owned_in(account)?;
            let hold = hold;
            let owned_font = copied_font;
            let start = completion.start([hold.as_ref(), None], None);
            let import = font
                .data
                .as_ref()
                .map(|data| account.import_font(data.admitted_face().resource()))
                .transpose()?
                .flatten();
            start.disarm();
            let font = shared(owned_font, hold, import, account)?;
            let mut copied = account.retained(glyphs.len())?;
            for glyph in glyphs {
                account.work(RenderWork::GlyphVisits, 1)?;
                copied.push(*glyph)?;
            }
            let paint = shared_paint(paint.fill(), capabilities, account)?;
            let (glyphs, hold) = copied.into_parts()?;
            RenderCommand::TextRun {
                font,
                size: *size,
                transform: *transform,
                paint,
                glyphs: shared(glyphs, Some(hold), None, account)?,
                bounds: *bounds,
            }
        }
        Command::TextShadowRun {
            size,
            transform,
            paint,
            glyphs,
            shadows,
            ..
        } => {
            validate_text_run_in(*size, *transform, glyphs, account)?;
            validate_paint_in(paint.fill(), account)?;
            for shadow in shadows.shadows() {
                validate_shadow_in(shadow, account)?;
            }
            return Err(unsupported_text_shadow_error(plan_text_shadow_run(
                shadows,
                capabilities,
                account,
            )?));
        }
        Command::Layer { .. } => return Err(crate::RenderAccountingError::Invariant.into()),
    })
}
fn emit_command(command: &RenderCommand, account: &Account) -> Result<()> {
    match command {
        RenderCommand::Fill { paint, .. } => {
            account.work_batch([
                (RenderWork::CommandsEmitted, 1),
                (RenderWork::FillsEmitted, 1),
            ])?;
            observe_render_paint(paint, account)?;
        }
        RenderCommand::Stroke { paint, .. } => {
            account.work_batch([
                (RenderWork::CommandsEmitted, 1),
                (RenderWork::StrokesEmitted, 1),
            ])?;
            observe_render_paint(paint, account)?;
        }
        RenderCommand::Shadow { .. } => account.work_batch([
            (RenderWork::CommandsEmitted, 1),
            (RenderWork::ShadowsEmitted, 1),
        ])?,
        RenderCommand::Image { image, .. } | RenderCommand::ImageRegion { image, .. } => {
            account.work(RenderWork::CommandsEmitted, 1)?;
            observe_image(image, account)?;
        }
        RenderCommand::TextRun { glyphs, paint, .. } => {
            account.work_batch([
                (RenderWork::CommandsEmitted, 1),
                (
                    RenderWork::GlyphsEmitted,
                    crate::accounting::as_u64(glyphs.len())?,
                ),
            ])?;
            observe_paint(paint.fill(), account)?;
        }
        RenderCommand::PreparedGlyphRun { run, paint, .. } => {
            account.work_batch([
                (RenderWork::CommandsEmitted, 1),
                (
                    RenderWork::GlyphsEmitted,
                    crate::accounting::as_u64(run.glyphs().len())?,
                ),
            ])?;
            observe_paint(paint.fill(), account)?;
            for paint in run.paints() {
                account.work(RenderWork::PaintVisits, 1)?;
                if let crate::text::GlyphPaint::Bitmap { image, .. } = paint {
                    observe_image(image, account)?;
                }
            }
        }
        RenderCommand::Layer { .. } => account.work_batch([
            (RenderWork::CommandsEmitted, 1),
            (RenderWork::LayersEmitted, 1),
        ])?,
    }
    Ok(())
}
fn observe_image(image: &Image, account: &Account) -> Result<()> {
    account.work(RenderWork::ImagesEmitted, 1)?;
    account.observe_image(image)
}
fn observe_paint(paint: &Paint, account: &Account) -> Result<()> {
    account.work(RenderWork::PaintVisits, 1)?;
    if let PaintKind::Image(image) = paint.kind() {
        observe_image(image, account)?;
    }
    Ok(())
}
fn observe_render_paint(paint: &RenderPaint, account: &Account) -> Result<()> {
    account.work(RenderWork::PaintVisits, 1)?;
    if let RenderPaint::Image(image) = paint {
        observe_image(image, account)?;
    }
    Ok(())
}

fn commands_contain_backdrop_capture(
    mut commands: &[RenderCommand],
    account: &Account,
) -> Result<bool> {
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
            if layer.backdrop.is_some() {
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

fn nested_backdrop_capture_error() -> Error {
    Error::unsupported_render_primitive(UnsupportedPrimitive::new(
        PrimitiveFamily::OffscreenPipeline,
        PrimitiveOperation::BroadBackdropExecution,
    )).append_message(
        ": nested backdrop capture crosses a layer isolation boundary and is not normalized in this task",
    )
}

fn transformed_backdrop_capture_error() -> Error {
    Error::unsupported_render_primitive(UnsupportedPrimitive::new(
        PrimitiveFamily::OffscreenPipeline,
        PrimitiveOperation::BroadBackdropExecution,
    )).append_message(
        ": transformed backdrop capture requires coordinate-space reconciliation before materialized execution",
    )
}

fn repeated_top_level_backdrop_capture_error() -> Error {
    Error::unsupported_render_primitive(UnsupportedPrimitive::new(
        PrimitiveFamily::OffscreenPipeline,
        PrimitiveOperation::BroadBackdropExecution,
    )).append_message(
        ": repeated top-level backdrop capture requires staged source reconciliation before materialized execution",
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextShadowPlan {
    SupportedZeroBlurSolid,
    UnsupportedZeroBlurSolid,
    UnsupportedGlyphAlphaCapture,
}

fn plan_text_shadow_run(
    shadows: &ShadowList,
    capabilities: Capabilities,
    account: &Account,
) -> Result<TextShadowPlan> {
    Ok(
        if is_zero_blur_solid_text_shadow_subset(shadows, account)? {
            if capabilities.shadows().supports_text_shadows() {
                TextShadowPlan::SupportedZeroBlurSolid
            } else {
                TextShadowPlan::UnsupportedZeroBlurSolid
            }
        } else {
            TextShadowPlan::UnsupportedGlyphAlphaCapture
        },
    )
}

fn is_zero_blur_solid_text_shadow_subset(shadows: &ShadowList, account: &Account) -> Result<bool> {
    for shadow in shadows.shadows() {
        if shadow.kind() != ShadowKind::Outer || shadow.blur() != 0.0 || shadow.spread() != 0.0 {
            return Ok(false);
        }
        account.work(RenderWork::PaintVisits, 1)?;
        if !matches!(shadow.paint().kind(), PaintKind::Color(_)) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn unsupported_text_shadow_error(plan: TextShadowPlan) -> Error {
    let error = Error::unsupported_render_primitive(UnsupportedPrimitive::new(
        PrimitiveFamily::Shadows,
        PrimitiveOperation::TextShadow,
    ));
    match plan {
        TextShadowPlan::SupportedZeroBlurSolid | TextShadowPlan::UnsupportedZeroBlurSolid => {
            error.append_message(
                ": zero-blur solid text shadows could be represented as repeated shifted glyph draws behind text, but this renderer has not claimed or enabled that executable subset yet",
            )
        }
        TextShadowPlan::UnsupportedGlyphAlphaCapture => {
            error.append_message(
                ": text-shadow execution depends on glyph-alpha/offscreen text capture before blurred shadows can be composited behind text",
            )
        }
    }
}

impl RenderShape {
    fn from_authored_in(shape: &Shape, account: &Account) -> Result<Self> {
        validate_shape_in(shape, account)?;
        Ok(match shape.kind() {
            ShapeKind::Rect(rect) => Self::Rect(*rect),
            ShapeKind::RoundedRect { rect, radii } => Self::RoundedRect {
                rect: *rect,
                radii: *radii,
            },
            ShapeKind::Circle { center, radius } => Self::Circle {
                center: *center,
                radius: *radius,
            },
            ShapeKind::Ellipse { center, radii } => Self::Ellipse {
                center: *center,
                radii: *radii,
            },
            ShapeKind::Path(path) => Self::Path(shared_path(path, account)?),
        })
    }
}

impl RenderClip {
    fn from_input_in(
        input: &ClipInput,
        capabilities: Capabilities,
        account: &Account,
    ) -> Result<Self> {
        let (geometry, hold) = input.normalize_geometry_in(capabilities, account)?;
        // Move the normalized path and its original backing hold, avoiding another copy.
        let coordinate_space = input.coordinate_space();
        let geometry = match geometry.into_kind() {
            ClipGeometryKind::Rect(rect) => RenderClipGeometry::Rect(rect),
            ClipGeometryKind::RoundedRect { rect, radii } => {
                RenderClipGeometry::RoundedRect { rect, radii }
            }
            ClipGeometryKind::Circle { center, radius } => {
                RenderClipGeometry::Circle { center, radius }
            }
            ClipGeometryKind::Ellipse { center, radii } => {
                RenderClipGeometry::Ellipse { center, radii }
            }
            ClipGeometryKind::Path(path) => {
                let (path, fill_rule) = path.into_parts();
                RenderClipGeometry::Path {
                    path: shared(path, hold, None, account)?,
                    fill_rule,
                }
            }
        };
        Ok(Self {
            geometry,
            coordinate_space,
        })
    }
    #[must_use]
    pub(crate) const fn geometry(&self) -> &RenderClipGeometry {
        &self.geometry
    }
    #[must_use]
    pub(crate) const fn coordinate_space(&self) -> Option<CoordinateSpaceTag> {
        self.coordinate_space
    }
    pub(crate) fn bounds_for_planning(&self, account: &Account) -> Result<OffscreenBounds> {
        render_clip_bounds(self, account)
    }
}

impl RenderStrokeShape {
    fn from_authored_in(shape: &Shape, stroke: Stroke, account: &Account) -> Result<Self> {
        validate_shape_in(shape, account)?;
        validate_stroke(stroke)?;

        let align = stroke.align_kind();
        let width = stroke.width();
        let offset = match align {
            StrokeAlign::Center => 0.0,
            StrokeAlign::Inside => -width * 0.5,
            StrokeAlign::Outside => width * 0.5,
        };

        Ok(match shape.kind() {
            ShapeKind::Rect(rect) => Self::Rect(kurbo::Rect::from(*rect).inflate(offset, offset)),
            ShapeKind::RoundedRect { rect, radii } => {
                let radii = offset_radii(*radii, offset);
                let rect = kurbo::Rect::from(*rect).inflate(offset, offset);
                Self::RoundedRect(kurbo_rounded_rect(
                    Rect::new(rect.x0, rect.y0, rect.width(), rect.height()),
                    radii,
                ))
            }
            ShapeKind::Circle { center, radius } => Self::Circle(kurbo::Circle::new(
                (center.x(), center.y()),
                (*radius + offset).max(0.0),
            )),
            ShapeKind::Ellipse { center, radii } => Self::Ellipse(kurbo::Ellipse::new(
                (center.x(), center.y()),
                (
                    (radii.width() + offset).max(0.0),
                    (radii.height() + offset).max(0.0),
                ),
                0.0,
            )),
            ShapeKind::Path(path) if align == StrokeAlign::Center => {
                let (path, hold) = path.to_kurbo_retained_in(account)?;
                Self::Path(shared(path, Some(hold), None, account)?)
            }
            ShapeKind::Path(_) => {
                return Err(Error::unsupported_render_primitive(
                    UnsupportedPrimitive::new(
                        PrimitiveFamily::GeometryTargets,
                        PrimitiveOperation::InsideOutsidePathStrokeAlignment,
                    ),
                ));
            }
        })
    }
}

impl TryFrom<Stroke> for RenderStroke {
    type Error = Error;

    fn try_from(stroke: Stroke) -> Result<Self> {
        validate_stroke(stroke)?;
        Ok(Self {
            width: stroke.width(),
            join: stroke.join_kind(),
            start_cap: stroke.start_cap(),
            end_cap: stroke.end_cap(),
            miter_limit: stroke.miter_limit(),
            dash: stroke.dash(),
        })
    }
}

impl RenderPaint {
    fn from_authored_in(
        paint: &Paint,
        capabilities: Capabilities,
        account: &Account,
    ) -> Result<Self> {
        ensure_paint_supported(paint, capabilities)?;
        validate_paint_in(paint, account)?;
        account.work(RenderWork::PaintVisits, 1)?;
        Ok(match paint.kind() {
            PaintKind::Color(color) => Self::Color(*color),
            PaintKind::RepeatingGradient(gradient) => {
                let (gradient, hold) = gradient.try_clone_in(account)?;
                Self::RepeatingGradient(shared(gradient, Some(hold), None, account)?)
            }
            PaintKind::Gradient(gradient) => {
                let (gradient, hold) = gradient.try_clone_in(account)?;
                Self::Gradient(shared(gradient, Some(hold), None, account)?)
            }
            PaintKind::Image(image) => {
                account.work(RenderWork::ImageIdentityCopies, 1)?;
                Self::Image(shared(
                    image.clone(),
                    None,
                    account.import_image(image)?,
                    account,
                )?)
            }
        })
    }
}

impl ShadowShape {
    fn from_authored(shape: &Shape, account: &Account) -> Result<Self> {
        validate_shape_in(shape, account)?;
        Ok(match shape.kind() {
            ShapeKind::Rect(rect) => Self::Rect(*rect),
            ShapeKind::RoundedRect { rect, radii } => Self::RoundedRect {
                rect: *rect,
                radii: *radii,
            },
            ShapeKind::Circle { center, radius } => Self::Circle {
                center: *center,
                radius: *radius,
            },
            ShapeKind::Ellipse { .. } | ShapeKind::Path(_) => {
                return Err(Error::unsupported_render_primitive(
                    UnsupportedPrimitive::new(
                        PrimitiveFamily::Shadows,
                        PrimitiveOperation::EllipsePathShadowShape,
                    ),
                ));
            }
        })
    }
}

impl RenderShadow {
    fn from_authored(shadow: &Shadow, account: &Account) -> Result<Self> {
        validate_shadow_in(shadow, account)?;
        if shadow.kind() == ShadowKind::Inset {
            return Err(Error::unsupported_render_primitive(
                UnsupportedPrimitive::new(
                    PrimitiveFamily::Shadows,
                    PrimitiveOperation::InsetBoxShadow,
                ),
            ));
        }
        Ok(Self {
            offset: shadow.offset(),
            blur: shadow.blur(),
            spread: shadow.spread(),
            color: solid_shadow_color(shadow.paint())?,
        })
    }
}

impl NormalizedLayer {
    fn from_authored_in(
        layer: &Layer,
        children: &[RenderCommand],
        capabilities: Capabilities,
        account: &Account,
    ) -> Result<Self> {
        let foreground_filters = if let Some(filter) = layer.foreground_filter() {
            capabilities.ensure_supported(UnsupportedPrimitive::new(
                PrimitiveFamily::Filters,
                PrimitiveOperation::LayerFilter,
            ))?;
            Some(match filter {
                LayerFilter::Offset(offset) => {
                    capabilities.ensure_supported(UnsupportedPrimitive::new(
                        PrimitiveFamily::Filters,
                        PrimitiveOperation::GpuOffsetFilterExecution,
                    ))?;
                    NormalizedForegroundEffect::Offset(*offset)
                }
                LayerFilter::Convolution(convolution) => {
                    capabilities.ensure_supported(UnsupportedPrimitive::new(
                        PrimitiveFamily::Filters,
                        PrimitiveOperation::GpuConvolutionFilterExecution,
                    ))?;
                    // The wrapper owns an import lease for the shared admitted payload.
                    let import = account.import_convolution(convolution)?;
                    NormalizedForegroundEffect::Convolution(
                        convolution.with_import(import, account)?,
                    )
                }
                LayerFilter::Blur(filter) => {
                    let operation = FilterOp::blur(FilterBlur::try_new(filter.blur_radius())?);
                    let (filters, hold) =
                        FilterList::copy_ops_in(std::slice::from_ref(&operation), account)?;
                    NormalizedForegroundEffect::Css(shared(filters, hold, None, account)?)
                }
                LayerFilter::List(filters) => {
                    let (filters, hold) = filters.try_clone_in(account)?;
                    NormalizedForegroundEffect::Css(shared(filters, hold, None, account)?)
                }
            })
        } else {
            None
        };
        let clip = layer
            .clip_input()
            .map(|clip| RenderClip::from_input_in(clip, capabilities, account))
            .transpose()?;
        let mask = layer
            .resolved_image_mask()
            .map(|(image, bounds, value, application)| {
                RenderLayerMask::from_resolved_in(
                    image,
                    bounds,
                    value,
                    application,
                    capabilities,
                    account,
                )
            })
            .transpose()?;
        let backdrop = layer
            .backdrop_filter()
            .map(|backdrop| RenderBackdropCapture::from_input_in(backdrop, capabilities, account))
            .transpose()?;
        let pass_plan =
            LayerPassPlan::from_authored(layer, clip.as_ref(), children, capabilities, account)?;
        Ok(Self {
            clip,
            transform: layer.transform(),
            opacity: layer.opacity(),
            composition: layer.composition(),
            explicit_isolation: layer.is_isolated(),
            mask,
            foreground_filters,
            backdrop,
            isolation: pass_plan.isolation(),
            pass_plan,
        })
    }
}

fn direct_vello_layer_supported(
    requirement: LayerPassRequirement,
    capabilities: Capabilities,
) -> bool {
    match requirement {
        LayerPassRequirement::DirectVelloOpacity => capabilities
            .offscreen_pipeline()
            .supports_direct_vello_opacity_isolation(),
        LayerPassRequirement::DirectVelloBlend => capabilities
            .offscreen_pipeline()
            .supports_direct_vello_blend_isolation(),
        LayerPassRequirement::DirectVelloOpacityBlend => {
            capabilities
                .offscreen_pipeline()
                .supports_direct_vello_opacity_isolation()
                && capabilities
                    .offscreen_pipeline()
                    .supports_direct_vello_blend_isolation()
        }
        LayerPassRequirement::None
        | LayerPassRequirement::ClipOnly
        | LayerPassRequirement::BoundedBackdropCapture
        | LayerPassRequirement::OffscreenTexture(_)
        | LayerPassRequirement::DiagnosticBoundary(_) => false,
    }
}

fn reject_unsupported_layer_effect(layer: &Layer) -> Result<()> {
    if let Some(unsupported) = unsupported_layer_effect(layer) {
        return Err(Error::unsupported_render_primitive(unsupported));
    }
    Ok(())
}

fn unsupported_layer_effect(layer: &Layer) -> Option<UnsupportedPrimitive> {
    if layer.mask().is_some() {
        return Some(UnsupportedPrimitive::new(
            PrimitiveFamily::MasksAndClips,
            PrimitiveOperation::LayerMask,
        ));
    }
    None
}

fn clip_bounds(clip: Option<&RenderClip>, account: &Account) -> Result<Option<OffscreenBounds>> {
    clip.map(|clip| render_clip_bounds(clip, account))
        .transpose()
}

pub(crate) fn commands_bounds_for_planning(
    commands: &[RenderCommand],
    account: &Account,
) -> Result<Option<OffscreenBounds>> {
    commands_bounds(commands, account)
}

fn commands_bounds(
    commands: &[RenderCommand],
    account: &Account,
) -> Result<Option<OffscreenBounds>> {
    let mut bounds: Option<OffscreenBounds> = None;
    for command in commands {
        account.work(RenderWork::CommandVisits, 1)?;
        if let Some(command_bounds) = command_bounds(command, account)? {
            bounds = Some(match bounds {
                Some(bounds) => union_bounds(bounds, command_bounds)?,
                None => command_bounds,
            });
        }
    }
    Ok(bounds)
}

fn command_bounds(command: &RenderCommand, account: &Account) -> Result<Option<OffscreenBounds>> {
    match command {
        RenderCommand::Fill { shape, .. } => render_shape_bounds(shape, account).map(Some),
        RenderCommand::Stroke { shape, stroke, .. } => stroke_shape_bounds(shape, stroke).map(Some),
        RenderCommand::Shadow { shape, shadow } => shadow_bounds(shape, shadow).map(Some),
        RenderCommand::Image { rect, .. } | RenderCommand::ImageRegion { rect, .. } => {
            OffscreenBounds::try_new(*rect).map(Some)
        }
        RenderCommand::PreparedGlyphRun {
            run: outline,
            transform,
            ..
        } => outline
            .bounds()
            .ink_rect()
            .map(|bounds| transform_bounds(bounds, *transform))
            .transpose(),
        RenderCommand::TextRun {
            transform, bounds, ..
        } => match bounds.kind() {
            TextRunBoundsKind::Unspecified | TextRunBoundsKind::Empty => Ok(None),
            TextRunBoundsKind::Ink => bounds
                .ink_rect()
                .map(|bounds| transform_bounds(bounds, *transform))
                .transpose(),
        },
        RenderCommand::Layer { layer, .. } => layer
            .pass_plan
            .bounds()
            .map(|bounds| transform_bounds(bounds.rect(), layer.transform))
            .transpose(),
    }
}

fn render_shape_bounds(shape: &RenderShape, account: &Account) -> Result<OffscreenBounds> {
    match shape {
        RenderShape::Rect(rect) | RenderShape::RoundedRect { rect, .. } => {
            OffscreenBounds::try_new(*rect)
        }
        RenderShape::Circle { center, radius } => OffscreenBounds::try_new(Rect::new(
            center.x() - radius,
            center.y() - radius,
            radius * 2.0,
            radius * 2.0,
        )),
        RenderShape::Ellipse { center, radii } => OffscreenBounds::try_new(Rect::new(
            center.x() - radii.width(),
            center.y() - radii.height(),
            radii.width() * 2.0,
            radii.height() * 2.0,
        )),
        RenderShape::Path(path) => {
            let path = path.kurbo_shape_in(account)?;
            kurbo_bounds(path.bounding_box())
        }
    }
}

fn render_clip_bounds(clip: &RenderClip, account: &Account) -> Result<OffscreenBounds> {
    let bounds = match clip.geometry() {
        RenderClipGeometry::Rect(rect) | RenderClipGeometry::RoundedRect { rect, .. } => {
            OffscreenBounds::try_new(*rect)?
        }
        RenderClipGeometry::Circle { center, radius } => OffscreenBounds::try_new(Rect::new(
            center.x() - radius,
            center.y() - radius,
            radius * 2.0,
            radius * 2.0,
        ))?,
        RenderClipGeometry::Ellipse { center, radii } => OffscreenBounds::try_new(Rect::new(
            center.x() - radii.width(),
            center.y() - radii.height(),
            radii.width() * 2.0,
            radii.height() * 2.0,
        ))?,
        RenderClipGeometry::Path { path, .. } => {
            let path = path.kurbo_shape_in(account)?;
            kurbo_bounds(path.bounding_box())?
        }
    };
    match clip.coordinate_space() {
        Some(coordinate_space) => transform_bounds(bounds.rect(), coordinate_space.transform()),
        None => Ok(bounds),
    }
}

fn stroke_shape_bounds(
    shape: &RenderStrokeShape,
    stroke: &RenderStroke,
) -> Result<OffscreenBounds> {
    if let RenderStrokeShape::Path(path) = shape
        && path.segments().next().is_none()
    {
        return kurbo_bounds(path.bounding_box());
    }

    let half_width = stroke.width * 0.5;
    let (bounds, inflate) = match shape {
        RenderStrokeShape::Rect(rect) => (*rect, half_width),
        RenderStrokeShape::RoundedRect(rect) => (rect.bounding_box(), half_width),
        RenderStrokeShape::Circle(circle) => (circle.bounding_box(), half_width),
        RenderStrokeShape::Ellipse(ellipse) => (ellipse.bounding_box(), half_width),
        RenderStrokeShape::Path(path) => {
            let mut inflate = match stroke.join {
                LineJoin::Miter => half_width * stroke.miter_limit.max(1.0),
                LineJoin::Round | LineJoin::Bevel => half_width,
            };
            if stroke.start_cap == LineCap::Square || stroke.end_cap == LineCap::Square {
                // A square corner combines orthogonal half-width vectors.
                // Multiply width directly to avoid halving underflow; round
                // outward because even this upper approximation of 1/sqrt(2)
                // can produce an inward-rounded product.
                let cap_support = (stroke.width * std::f64::consts::FRAC_1_SQRT_2).next_up();
                inflate = inflate.max(cap_support);
            }
            (path.bounding_box(), inflate)
        }
    };
    kurbo_bounds(bounds.inflate(inflate, inflate))
}

fn shadow_bounds(shape: &ShadowShape, shadow: &RenderShadow) -> Result<OffscreenBounds> {
    let base = match shape {
        ShadowShape::Rect(rect) | ShadowShape::RoundedRect { rect, .. } => *rect,
        ShadowShape::Circle { center, radius } => Rect::new(
            center.x() - radius,
            center.y() - radius,
            radius * 2.0,
            radius * 2.0,
        ),
    };
    let offset = Rect::new(
        base.x() + shadow.offset.x(),
        base.y() + shadow.offset.y(),
        base.width(),
        base.height(),
    );
    let blur_support = super::filter::vello_outer_shadow_support_radius(shadow.blur)?;
    let support = shadow.spread + blur_support;
    if !support.is_finite() {
        return Err(Error::invalid_value(
            "box shadow support",
            support,
            "must be finite",
        ));
    }
    OffscreenBounds::try_new(geometry::expand_rect(offset, support))
}

#[cfg(test)]
pub(crate) fn outer_box_shadow_bounds_for_test(command: &RenderCommand) -> Result<Option<Rect>> {
    match command {
        RenderCommand::Shadow { shape, shadow } => shadow_bounds(shape, shadow)
            .map(OffscreenBounds::rect)
            .map(Some),
        RenderCommand::Fill { .. }
        | RenderCommand::Stroke { .. }
        | RenderCommand::Image { .. }
        | RenderCommand::ImageRegion { .. }
        | RenderCommand::TextRun { .. }
        | RenderCommand::PreparedGlyphRun { .. }
        | RenderCommand::Layer { .. } => Ok(None),
    }
}

fn transform_bounds(rect: Rect, transform: Transform) -> Result<OffscreenBounds> {
    let logical_bounds = LogicalBounds::try_from_rect(rect, "command bounds")?
        .try_transform(transform, "command transformed bounds")?;
    Ok(OffscreenBounds::from_logical(logical_bounds))
}

fn union_bounds(a: OffscreenBounds, b: OffscreenBounds) -> Result<OffscreenBounds> {
    Ok(OffscreenBounds::from_logical(
        a.logical_bounds
            .union(b.logical_bounds, "command bounds union")?,
    ))
}

fn kurbo_bounds(bounds: kurbo::Rect) -> Result<OffscreenBounds> {
    OffscreenBounds::try_new(Rect::try_from(bounds)?)
}

fn ensure_paint_supported(paint: &Paint, capabilities: Capabilities) -> Result<()> {
    if matches!(paint.kind(), PaintKind::RepeatingGradient(_)) {
        capabilities.ensure_supported(UnsupportedPrimitive::new(
            PrimitiveFamily::PaintSources,
            PrimitiveOperation::RepeatingGradient,
        ))?;
    }
    Ok(())
}

fn solid_shadow_color(paint: &Paint) -> Result<Color> {
    match paint.kind() {
        PaintKind::Color(color) => Ok(*color),
        PaintKind::Gradient(_) | PaintKind::RepeatingGradient(_) | PaintKind::Image(_) => Err(
            Error::unsupported_render_primitive(UnsupportedPrimitive::new(
                PrimitiveFamily::PaintSources,
                PrimitiveOperation::NonSolidShadowPaint,
            )),
        ),
    }
}

pub(crate) fn kurbo_rounded_rect(rect: Rect, radii: Radii) -> kurbo::RoundedRect {
    kurbo::RoundedRect::from_rect(
        rect.into(),
        kurbo::RoundedRectRadii::new(
            radii.top_left(),
            radii.top_right(),
            radii.bottom_right(),
            radii.bottom_left(),
        ),
    )
}

#[cfg(all(test, not(target_family = "wasm")))]
mod deep_normalization_tests {
    use super::*;

    #[test]
    fn deep_normalized_tree_releases_final_shared_backing() {
        std::thread::Builder::new()
            .name("release deep normalized scene".into())
            .stack_size(512 * 1024)
            .spawn(|| {
                const DEPTH: usize = 8_192;
                let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
                let mut scene = Scene::new();
                scene.fill(rect, Color::BLACK);
                for _ in 0..DEPTH {
                    let child = scene;
                    scene = Scene::new();
                    scene.layer(Layer::new(), |target| *target = child);
                    scene.fill(rect, Color::TRANSPARENT);
                }
                let mut context = RenderAccountingContext::default();
                context
                    .run(
                        RenderAttemptKind::Frame,
                        RenderPhase::Normalize,
                        |account| {
                            let normalized = scene.normalize(Capabilities::CURRENT, account)?;
                            let retained = normalized.clone();
                            drop(normalized);
                            let mut current = retained.as_ref();
                            for _ in 0..DEPTH {
                                let [
                                    RenderCommand::Layer { children, .. },
                                    RenderCommand::Fill { .. },
                                ] = current
                                else {
                                    panic!("expected the authored layer followed by its sibling");
                                };
                                current = children;
                            }
                            assert!(matches!(current, [RenderCommand::Fill { .. }]));
                            drop(retained);
                            Ok(())
                        },
                    )
                    .unwrap();
                let report = context.last_attempt().unwrap();
                let usage = report.attempt().unwrap().usage();
                assert_eq!(
                    usage.work(RenderWork::CommandsEmitted),
                    (2 * DEPTH + 1) as u64
                );
                for resource in [
                    RenderResource::RetainedCpuBytes,
                    RenderResource::ScratchCpuBytes,
                ] {
                    assert!(usage.peak(resource) > 0);
                    assert_eq!(usage.live(resource), 0);
                }
                assert_eq!(report.cleanup_fault(), None);

                // The final sibling refuses after the root layer already
                // owns its complete normalized subtree.
                let cap = (2 * DEPTH) as u64;
                let limits = RenderLimits::new()
                    .with_work_limit(RenderWork::CommandsEmitted, cap)
                    .unwrap();
                context.set_options(RenderAccountingOptions::new().with_limits(limits));
                let error = context
                    .run(
                        RenderAttemptKind::Frame,
                        RenderPhase::Normalize,
                        |account| scene.normalize(Capabilities::CURRENT, account).map(drop),
                    )
                    .unwrap_err();
                let report = context.last_attempt().unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                let attempt = report.attempt().unwrap();
                let refusal = attempt.refusal().unwrap();
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(refusal))
                );
                assert_eq!(
                    refusal.quantity,
                    RenderQuantity::Work(RenderWork::CommandsEmitted)
                );
                assert_eq!(refusal.current, cap);
                assert_eq!(refusal.amount, 1);
                assert_eq!(attempt.usage().work(RenderWork::CommandsEmitted), cap);
                for resource in [
                    RenderResource::RetainedCpuBytes,
                    RenderResource::ScratchCpuBytes,
                ] {
                    assert!(attempt.usage().peak(resource) > 0);
                    assert_eq!(attempt.usage().live(resource), 0);
                }
                assert_eq!(report.cleanup_fault(), None);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}

#[cfg(test)]
mod layer_metadata_tests {
    use super::*;
    use crate::accounting::{FrameFixture, RendererCpu};

    fn empty_layer(account: &Account) -> Result<(NormalizedLayer, RenderCommands)> {
        let children = RenderCommands::new_in(account.retained(0)?, account)?;
        let layer = NormalizedLayer::from_authored_in(
            &Layer::new().try_opacity(0.375)?,
            &children,
            Capabilities::CURRENT,
            account,
        )?;
        Ok((layer, children))
    }

    #[test]
    fn normalized_layer_metadata_remains_live_until_final_command_clone_drops() {
        let options =
            RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
        let cpu = RendererCpu::new(options).unwrap();
        let opening = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let (result, _, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let account = attempt.account();
                let (layer, children) = empty_layer(account)?;
                let before = cpu
                    .snapshot()?
                    .usage()
                    .live(RenderResource::RetainedCpuBytes);
                let command = RenderCommand::layer_in(layer, children, account)?;
                let admitted = cpu
                    .snapshot()?
                    .usage()
                    .live(RenderResource::RetainedCpuBytes);
                assert_eq!(
                    admitted - before,
                    SharedValue::<NormalizedLayer>::allocation_size() as u64
                );
                let RenderCommand::Layer { layer, .. } = &command else {
                    panic!("expected a normalized layer");
                };
                assert_eq!(layer.opacity, 0.375);
                let metadata_live = layer.allocation_liveness_for_test();
                let clone = command.clone();
                drop(command);
                assert!(metadata_live());
                assert_eq!(
                    cpu.snapshot()?
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    admitted
                );
                drop(clone);
                assert!(!metadata_live());
                assert_eq!(
                    cpu.snapshot()?
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    opening
                );
                Ok(())
            })
            .unwrap()
            .into_parts();
        result.unwrap();
        assert_eq!(fault, None);
        let interval = report.renderer_cpu().unwrap();
        assert_eq!(
            interval.usage().live(RenderResource::RetainedCpuBytes),
            opening
        );
        assert_eq!(
            interval
                .scope_usage(RenderCpuScope::FrameBacking)
                .live(RenderResource::RetainedCpuBytes),
            0
        );
        assert!(
            interval
                .scope_usage(RenderCpuScope::FrameBacking)
                .peak(RenderResource::RetainedCpuBytes)
                > 0
        );
    }

    #[test]
    fn actual_layer_metadata_exact_budget_admits_and_one_under_releases_children() {
        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let (required, _, _, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let (layer, children) = empty_layer(attempt.account())?;
                let command = RenderCommand::layer_in(layer, children, attempt.account())?;
                let required = cpu
                    .snapshot()?
                    .usage()
                    .live(RenderResource::RetainedCpuBytes);
                drop(command);
                Ok(required)
            })
            .unwrap()
            .into_parts();
        let required = required.unwrap();
        assert_eq!(fault, None);
        for cap in [required, required - 1] {
            let options = options.with_limits(RenderLimits::new().with_retained_bytes(cap));
            let cpu = RendererCpu::new(options).unwrap();
            let opening = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let (result, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let (layer, children) = empty_layer(attempt.account())?;
                    let before = cpu
                        .snapshot()?
                        .usage()
                        .live(RenderResource::RetainedCpuBytes);
                    match RenderCommand::layer_in(layer, children, attempt.account()) {
                        Ok(command) => {
                            assert_eq!(cap, required);
                            assert_eq!(
                                cpu.snapshot()?
                                    .usage()
                                    .live(RenderResource::RetainedCpuBytes),
                                required
                            );
                            drop(command);
                            Ok(())
                        }
                        Err(error) => {
                            assert_eq!(cap, required - 1);
                            let Some(RenderAccountingError::CpuRefused {
                                quantity,
                                current,
                                amount,
                                reason,
                                ..
                            }) = error.accounting_diagnostic()
                            else {
                                panic!("expected actual metadata backing refusal: {error:?}");
                            };
                            assert_eq!(
                                quantity,
                                RenderQuantity::Resource(RenderResource::RetainedCpuBytes)
                            );
                            assert_eq!(current, before);
                            assert_eq!(
                                amount,
                                SharedValue::<NormalizedLayer>::allocation_size() as u64
                            );
                            assert_eq!(reason, RenderRefusalReason::Limit { limit: cap });
                            assert_eq!(
                                cpu.snapshot()?
                                    .usage()
                                    .live(RenderResource::RetainedCpuBytes),
                                opening
                            );
                            Err(error)
                        }
                    }
                })
                .unwrap()
                .into_parts();
            assert_eq!(result.is_ok(), cap == required);
            assert_eq!(fault, None);
            assert_eq!(report.cleanup_fault(), None);
            assert_eq!(
                cpu.snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                opening
            );
        }
    }
}

#[cfg(test)]
mod unsupported_input_tests {
    use super::*;
    use crate::accounting::{FrameFixture, RendererCpu};

    fn path() -> Shape {
        let mut path = Path::new();
        path.move_to(Point::new(1.0, 2.0))
            .line_to(Point::new(4.0, 6.0));
        Shape::path(path)
    }

    fn shadow(paint: impl Into<Paint>) -> Shadow {
        Shadow::try_new(Point::new(3.0, -2.0), 2.0, 1.0, paint).unwrap()
    }

    fn cases() -> Vec<(
        Command,
        PrimitiveFamily,
        PrimitiveOperation,
        &'static str,
        u64,
        u64,
    )> {
        let rect = Shape::rect(Rect::new(1.0, 2.0, 3.0, 4.0));
        let mut cases = Vec::new();
        for align in [StrokeAlign::Inside, StrokeAlign::Outside] {
            cases.push((Command::Stroke {
                shape: path(),
                stroke: Stroke::try_new(2.0).unwrap().align(align),
                paint: Color::BLACK.into(),
            }, PrimitiveFamily::GeometryTargets, PrimitiveOperation::InsideOutsidePathStrokeAlignment,
                "render primitive is unsupported: geometry targets / inside/outside path stroke alignment", 2, 0));
        }
        for (shape, visits) in [
            (
                Shape::try_ellipse(Point::new(2.0, 3.0), Size::new(4.0, 5.0)).unwrap(),
                0,
            ),
            (path(), 2),
        ] {
            cases.push((
                Command::Shadow {
                    shape,
                    shadow: shadow(Color::BLACK),
                },
                PrimitiveFamily::Shadows,
                PrimitiveOperation::EllipsePathShadowShape,
                "render primitive is unsupported: shadows / ellipse/path shadow shape",
                visits,
                0,
            ));
        }
        cases.push((
            Command::Shadow {
                shape: rect.clone(),
                shadow: Shadow::try_inset(Point::new(0.0, 0.0), 2.0, 1.0, Color::BLACK).unwrap(),
            },
            PrimitiveFamily::Shadows,
            PrimitiveOperation::InsetBoxShadow,
            "render primitive is unsupported: shadows / inset box shadow",
            0,
            1,
        ));
        let stops = vec![GradientStop::try_new(0.0, Color::BLACK).unwrap()];
        let gradient =
            Gradient::try_linear(Point::new(0.0, 0.0), Point::new(1.0, 0.0), stops.clone())
                .unwrap();
        let repeating =
            RepeatingGradient::try_linear(Point::new(0.0, 0.0), Point::new(1.0, 0.0), stops)
                .unwrap();
        let image = Image::from_rgba(Size::new(1.0, 1.0), vec![0, 0, 0, 255]).unwrap();
        for paint in [
            Paint::from(gradient),
            Paint::from(repeating),
            Paint::from(image),
        ] {
            cases.push((
                Command::Shadow {
                    shape: rect.clone(),
                    shadow: shadow(paint),
                },
                PrimitiveFamily::PaintSources,
                PrimitiveOperation::NonSolidShadowPaint,
                "render primitive is unsupported: paint sources / non-solid shadow paint",
                0,
                1,
            ));
        }
        cases
    }

    fn assert_unsupported(
        error: &Error,
        family: PrimitiveFamily,
        operation: PrimitiveOperation,
        message: &str,
    ) {
        assert_eq!(error.code(), ErrorCode::UnsupportedPrimitive);
        assert_eq!(
            error.unsupported_primitive(),
            Some(UnsupportedPrimitive::new(family, operation))
        );
        assert_eq!(error.message(), message);
        assert!(std::error::Error::source(error).is_none());
        assert!(error.invalid_value_diagnostic().is_none());
        assert!(error.accounting_diagnostic().is_none());
    }

    #[test]
    fn unsupported_authored_forms_preserve_diagnostic_and_exact_work_prefix() {
        for (command, family, operation, message, paths, paints) in cases() {
            let mut context = RenderAccountingContext::default();
            let error = context
                .run(
                    RenderAttemptKind::Frame,
                    RenderPhase::Normalize,
                    |account| {
                        normalize_commands(
                            std::slice::from_ref(&command),
                            Capabilities::CURRENT,
                            account,
                        )
                    },
                )
                .unwrap_err();
            assert_unsupported(&error, family, operation, message);
            let report = context.last_attempt().unwrap().attempt().unwrap();
            let usage = report.usage();
            assert_eq!(usage.work(RenderWork::CommandVisits), 1);
            assert_eq!(usage.work(RenderWork::PathElementVisits), paths);
            assert_eq!(usage.work(RenderWork::PaintVisits), paints);
            assert_eq!(usage.work(RenderWork::GradientStopVisits), 0);
            assert_eq!(usage.work(RenderWork::ImageIdentityCopies), 0);
        }
    }

    #[test]
    fn reached_work_limits_win_before_unsupported_diagnostics() {
        for (command, _, _, _, paths, paints) in cases() {
            let mut limits = vec![(RenderWork::CommandVisits, 0)];
            if paths > 0 {
                limits.push((RenderWork::PathElementVisits, paths - 1));
            }
            if paints > 0 {
                limits.push((RenderWork::PaintVisits, 0));
            }
            for (work, limit) in limits {
                let options = RenderAccountingOptions::new()
                    .with_limits(RenderLimits::new().with_work_limit(work, limit).unwrap());
                let mut context = RenderAccountingContext::new(options);
                let error = context
                    .run(
                        RenderAttemptKind::Frame,
                        RenderPhase::Normalize,
                        |account| {
                            normalize_commands(
                                std::slice::from_ref(&command),
                                Capabilities::CURRENT,
                                account,
                            )
                        },
                    )
                    .unwrap_err();
                let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
                else {
                    panic!("expected work refusal: {error:?}");
                };
                assert_eq!(error.code(), ErrorCode::Accounting);
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
                assert_eq!(refusal.current(), limit);
                assert_eq!(refusal.amount(), 1);
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
                assert_eq!(
                    context
                        .last_attempt()
                        .unwrap()
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(work),
                    limit
                );
            }
        }
    }

    #[test]
    fn validation_and_shape_refusal_keep_first_error_precedence() {
        // Private unchecked numeric constructors let this owning harness exercise
        // validation; public try constructors refuse these values before normalization.
        let mut invalid_path = Path::new();
        invalid_path.move_to(Point::new(f64::NAN, 0.0));
        let error = RenderStrokeShape::from_authored_in(
            &Shape::path(invalid_path),
            Stroke::try_new(2.0).unwrap().align(StrokeAlign::Inside),
            &Account::for_test(),
        )
        .unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "path point x"
        );
        assert!(Point::try_new(f64::NAN, 0.0).is_err());
        assert!(Stroke::try_new(0.0).is_err());
        assert!(Shadow::try_new(Point::new(0.0, 0.0), -1.0, 0.0, Color::BLACK).is_err());
        let gradient = Gradient::try_linear(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            vec![GradientStop::try_new(0.0, Color::BLACK).unwrap()],
        )
        .unwrap();
        let inset_gradient = Shadow::try_inset(Point::new(0.0, 0.0), 0.0, 0.0, gradient).unwrap();
        let error = RenderShadow::from_authored(&inset_gradient, &Account::for_test()).unwrap_err();
        assert_unsupported(
            &error,
            PrimitiveFamily::Shadows,
            PrimitiveOperation::InsetBoxShadow,
            "render primitive is unsupported: shadows / inset box shadow",
        );
        let inset = Shadow::try_inset(Point::new(0.0, 0.0), 0.0, 0.0, Color::BLACK).unwrap();
        let commands = [Command::Shadow {
            shape: path(),
            shadow: inset,
        }];
        let error =
            normalize_commands(&commands, Capabilities::CURRENT, &Account::for_test()).unwrap_err();
        assert_unsupported(
            &error,
            PrimitiveFamily::Shadows,
            PrimitiveOperation::EllipsePathShadowShape,
            "render primitive is unsupported: shadows / ellipse/path shadow shape",
        );
    }

    #[test]
    fn unsupported_suffix_releases_normalized_prefix_and_retries_unchanged_input() {
        let options =
            RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
        let cpu = RendererCpu::new(options).unwrap();
        let opening = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        for (command, family, operation, message, _, _) in cases() {
            let mut scene = Scene::new();
            scene.stroke(path(), Stroke::try_new(2.0).unwrap(), Color::BLACK);
            scene.commands.push(command);
            let authored = scene.commands.clone();
            for _ in 0..2 {
                let (_, _, report, fault) = FrameFixture::new(&cpu, options)
                    .run_sync(|attempt| {
                        let error = scene
                            .normalize(Capabilities::CURRENT, attempt.account())
                            .unwrap_err();
                        assert_unsupported(&error, family, operation, message);

                        Err::<(), _>(error)
                    })
                    .unwrap()
                    .into_parts();
                assert_eq!(fault, None);
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::CommandsEmitted),
                    1
                );
                let interval = report.renderer_cpu().unwrap();
                assert_eq!(
                    interval.usage().live(RenderResource::RetainedCpuBytes),
                    opening
                );
                assert_eq!(
                    interval
                        .scope_usage(RenderCpuScope::FrameBacking)
                        .live(RenderResource::RetainedCpuBytes),
                    0
                );
                assert!(
                    interval
                        .scope_usage(RenderCpuScope::FrameBacking)
                        .peak(RenderResource::RetainedCpuBytes)
                        > 0
                );
                assert_eq!(scene.len(), 2);
                assert_eq!(scene.commands, authored);
            }
        }
    }

    #[test]
    fn centered_path_and_solid_outer_shadow_preserve_payload_geometry_and_work() {
        let mut scene = Scene::new();
        scene.stroke(path(), Stroke::try_new(2.0).unwrap(), Color::BLACK);
        scene.shadow(Rect::new(1.0, 2.0, 3.0, 4.0), shadow(Color::BLACK));
        let mut context = RenderAccountingContext::default();
        context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::Normalize,
                |account| {
                    let normalized = scene.normalize(Capabilities::CURRENT, account)?;
                    let [
                        RenderCommand::Stroke {
                            shape: RenderStrokeShape::Path(path),
                            stroke,
                            paint: RenderPaint::Color(color),
                        },
                        RenderCommand::Shadow {
                            shape: ShadowShape::Rect(rect),
                            shadow,
                        },
                    ] = normalized.commands.as_slice()
                    else {
                        panic!("supported commands changed");
                    };
                    assert_eq!(
                        path.elements(),
                        &[
                            kurbo::PathEl::MoveTo((1.0, 2.0).into()),
                            kurbo::PathEl::LineTo((4.0, 6.0).into())
                        ]
                    );
                    assert_eq!(stroke.width, 2.0);
                    assert_eq!(*color, Color::BLACK);
                    assert_eq!(*rect, Rect::new(1.0, 2.0, 3.0, 4.0));
                    assert_eq!(shadow.offset, Point::new(3.0, -2.0));
                    assert_eq!(shadow.blur, 2.0);
                    assert_eq!(shadow.spread, 1.0);
                    assert_eq!(shadow.color, Color::BLACK);
                    // blur/2 * 2.5 plus spread gives 3.5 units of support,
                    // around the rectangle translated by (3,-2).
                    assert_eq!(
                        shadow_bounds(&ShadowShape::Rect(*rect), shadow)?.rect(),
                        Rect::new(0.5, -3.5, 10.0, 11.0)
                    );
                    Ok(())
                },
            )
            .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CommandVisits), 2);
        assert_eq!(usage.work(RenderWork::PathElementVisits), 4);
        // Stroke validation, lowering and emission each inspect paint;
        // the solid shadow adds its validation visit.
        assert_eq!(usage.work(RenderWork::PaintVisits), 4);
    }
}

#[cfg(test)]
mod stroke_bounds_tests {
    use super::*;

    fn vertical_path_bounds(stroke: Stroke) -> Result<Rect> {
        let mut path = Path::new();
        path.move_to(Point::try_new(0.0, 0.0).unwrap())
            .line_to(Point::try_new(0.0, 1.0).unwrap());
        let account = Account::for_test();
        let shape =
            RenderStrokeShape::from_authored_in(&Shape::path(path), stroke, &account).unwrap();
        stroke_shape_bounds(&shape, &RenderStroke::try_from(stroke).unwrap())
            .map(OffscreenBounds::rect)
    }

    #[test]
    fn square_cap_support_contains_the_exact_diagonal_radius_after_rounding() {
        // These dyadic widths include products that round below width/sqrt(2)
        // if ordinary nearest multiplication is used without outward rounding.
        for width in [
            1.0,
            f64::from_bits(0x3ff0_0000_0000_0006),
            f64::from_bits(0x3ff0_0000_0000_0012),
            f64::from_bits(0x3ff0_0000_0000_0023),
            1.5,
            2.0_f64.next_down(),
        ] {
            for caps in [
                (LineCap::Square, LineCap::Butt),
                (LineCap::Butt, LineCap::Square),
            ] {
                let bounds = vertical_path_bounds(
                    Stroke::try_new(width)
                        .unwrap()
                        .join(LineJoin::Bevel)
                        .caps(caps.0, caps.1)
                        .try_miter_limit(f64::MAX)
                        .unwrap(),
                )
                .unwrap();
                let radius = -bounds.x();
                // Width and radius are in [0.5,2), so exact integers in units
                // of 2^-53 fit in u128 even after squaring. The independent
                // geometric condition is 2*radius^2 >= width^2.
                let exact_units = |value: f64| {
                    let bits = value.to_bits();
                    let exponent = bits >> 52;
                    assert!((1022..=1023).contains(&exponent));
                    let significand = u128::from((bits & ((1 << 52) - 1)) | (1 << 52));
                    significand << (exponent - 1022)
                };
                let width_units = exact_units(width);
                let radius_units = exact_units(radius);
                assert!(
                    2 * radius_units * radius_units >= width_units * width_units,
                    "inward square support for width {width:?}: {radius:?}"
                );
                assert!(radius < width);
            }
        }
    }

    #[test]
    fn square_cap_support_survives_subnormal_width_without_halving_it_first() {
        let width = f64::from_bits(1);
        let bounds = vertical_path_bounds(
            Stroke::try_new(width)
                .unwrap()
                .join(LineJoin::Round)
                .caps(LineCap::Square, LineCap::Butt),
        )
        .unwrap();
        // A positive radius below the least subnormal still requires at least
        // one representable outward step. This bounds test makes no claim of
        // native GPU representability for the tiny width.
        assert!(-bounds.x() >= width);
    }

    #[test]
    fn extreme_square_support_keeps_finite_coordinates_and_typed_extent_guard() {
        let stroke = |width| {
            Stroke::try_new(width)
                .unwrap()
                .join(LineJoin::Bevel)
                .caps(LineCap::Square, LineCap::Butt)
        };
        let width = f64::MAX * 0.5;
        let bounds = vertical_path_bounds(stroke(width)).unwrap();
        let radius = -bounds.x();
        // The 3-4-5 direction needs 0.7*width along an axis; square support
        // remains less than width. Neither comparison squares huge values.
        assert!(radius >= width * 0.7);
        assert!(radius < width);
        let error = vertical_path_bounds(stroke(f64::MAX)).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        // The selected-style envelope's coordinates remain finite, but its
        // full rectangular extent cannot fit. This is conservative support,
        // not proof that this particular vertical stroke has infinite ink.
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "rectangle width"
        );
    }
}

#[cfg(test)]
mod paint_visit_tests {
    use super::*;

    fn solid_shadow() -> Shadow {
        Shadow::try_new(Point::new(0.0, 0.0), 0.0, 0.0, Color::BLACK).unwrap()
    }

    fn scene(shadows: Vec<Shadow>) -> Scene {
        let glyphs = [TextGlyph::try_new(58, 0.0, 0.0, 20.0).unwrap()];
        let run = TextRun::try_new(
            FontRef::new(1),
            20.0,
            Transform::identity(),
            TextPaint::try_fill(Color::BLACK.into()).unwrap(),
            &glyphs,
            TextRunBounds::unspecified(),
        )
        .unwrap();
        let mut scene = Scene::new();
        scene.text_shadow_run(
            TextShadowRun::try_new(run, ShadowList::try_new(shadows).unwrap()).unwrap(),
        );
        scene
    }

    fn normalize(scene: &Scene, context: &mut RenderAccountingContext) -> Result<()> {
        context.run(
            RenderAttemptKind::Frame,
            RenderPhase::Normalize,
            |account| scene.normalize(Capabilities::CURRENT, account).map(drop),
        )
    }

    #[test]
    fn text_shadow_diagnostic_counts_only_paint_conditions_reached_after_geometry_checks() {
        let inset = Shadow::try_inset(Point::new(0.0, 0.0), 0.0, 0.0, Color::BLACK).unwrap();
        let blurred = Shadow::try_new(Point::new(0.0, 0.0), 1.0, 0.0, Color::BLACK).unwrap();
        let spread = Shadow::try_new(Point::new(0.0, 0.0), 0.0, 1.0, Color::BLACK).unwrap();
        let gradient = Shadow::try_new(
            Point::new(0.0, 0.0),
            0.0,
            0.0,
            Gradient::try_linear(
                Point::new(0.0, 0.0),
                Point::new(2.0, 0.0),
                vec![GradientStop::try_new(0.0, Color::BLACK).unwrap()],
            )
            .unwrap(),
        )
        .unwrap();
        for (shadows, expected, diagnostic) in [
            (vec![solid_shadow()], 3, "zero-blur solid"),
            (vec![solid_shadow(), solid_shadow()], 5, "zero-blur solid"),
            (vec![inset, solid_shadow()], 3, "glyph-alpha/offscreen"),
            (
                vec![blurred.clone(), solid_shadow()],
                3,
                "glyph-alpha/offscreen",
            ),
            (vec![spread, solid_shadow()], 3, "glyph-alpha/offscreen"),
            (vec![solid_shadow(), blurred], 4, "glyph-alpha/offscreen"),
            (vec![gradient, solid_shadow()], 4, "glyph-alpha/offscreen"),
        ] {
            let mut context = RenderAccountingContext::default();
            let error = normalize(&scene(shadows), &mut context).unwrap_err();
            assert_eq!(error.code(), ErrorCode::UnsupportedPrimitive);
            assert!(error.message().contains(diagnostic));
            // Run paint and every shadow are validated first. The diagnostic
            // then inspects paint only after outer/zero-blur/zero-spread pass,
            // and `all` stops at its first false candidate.
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::PaintVisits),
                expected
            );
        }
    }

    #[test]
    fn text_shadow_diagnostic_refuses_after_the_exact_validation_and_candidate_prefix() {
        let scene = scene(vec![solid_shadow(), solid_shadow()]);
        for limit in [3, 4] {
            let mut context = RenderAccountingContext::new(
                RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::PaintVisits, limit)
                        .unwrap(),
                ),
            );
            let error = normalize(&scene, &mut context).unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("expected the reached paint inspection to refuse, got {error:?}");
            };
            // General work limits belong to Attempt; Normalize is the origin
            // of this inspection rather than the scope that owns the cap.
            assert_eq!(refusal.phase(), RenderPhase::Attempt);
            assert_eq!(
                refusal.quantity(),
                RenderQuantity::Work(RenderWork::PaintVisits)
            );
            assert_eq!(refusal.current(), limit);
            assert_eq!(refusal.amount(), 1);
            assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(report.usage().work(RenderWork::PaintVisits), limit);
        }
    }

    mod normalization_custody_tests {
        use super::*;
        use crate::accounting::{FrameFixture, RendererCpu};

        fn options(work: RenderWork, limit: u64) -> RenderAccountingOptions {
            RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: false })
                .with_limits(RenderLimits::new().with_work_limit(work, limit).unwrap())
        }

        fn text_scene(font: FontRef<'static>) -> Scene {
            let glyphs = [
                TextGlyph::try_new(58, 0.0, 16.0, 20.0).unwrap(),
                TextGlyph::try_new(58, 20.0, 16.0, 20.0).unwrap(),
            ];
            let paint = Gradient::try_linear(
                Point::new(0.0, 0.0),
                Point::new(40.0, 0.0),
                vec![GradientStop::try_new(0.0, Color::BLACK).unwrap()],
            )
            .unwrap();
            let mut scene = Scene::new();
            scene.text_run(
                TextRun::try_new(
                    font,
                    20.0,
                    Transform::identity(),
                    TextPaint::try_fill(paint.into()).unwrap(),
                    &glyphs,
                    TextRunBounds::unspecified(),
                )
                .unwrap(),
            );
            scene
        }

        fn refusal(error: &Error, work: RenderWork, current: u64) {
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("expected the selected normalization refusal, got {error:?}");
            };
            assert_eq!(refusal.phase(), RenderPhase::Attempt);
            assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
            assert_eq!(refusal.current(), current);
            assert_eq!(refusal.amount(), 1);
            assert_eq!(
                refusal.reason(),
                RenderRefusalReason::Limit { limit: current }
            );
        }

        #[test]
        fn named_font_import_refusal_releases_temporary_name_and_retains_caller_data() {
            let font = FontData::try_from_bytes(
                include_bytes!("../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
                0,
                surgeist_test::fixtures::FONT_LIMITS,
            )
            .unwrap();
            let options = options(RenderWork::ResourceProbes, 0);
            let cpu = RendererCpu::new(options).unwrap();
            // One real existing registry candidate makes the import's first
            // probe refusable after normalization has copied the display name.
            let imported = cpu
                .at(RenderCpuScope::FontPayloads)
                .import_font(font.admitted_face().resource(), None)
                .unwrap();
            let opening = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let scene = text_scene(
                FontRef::new(1)
                    .named("normalization font name")
                    .with_data(font.clone()),
            );
            let (_, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let error = scene
                        .normalize(Capabilities::CURRENT, attempt.account())
                        .unwrap_err();
                    refusal(&error, RenderWork::ResourceProbes, 0);

                    assert_eq!(attempt_status(&error), RenderAttemptStatus::Refused);
                    Err::<(), _>(error)
                })
                .unwrap()
                .into_parts();
            assert_eq!(fault, None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::ResourceProbes), 0);
            assert_eq!(usage.work(RenderWork::PaintVisits), 1);
            assert_eq!(usage.work(RenderWork::GlyphVisits), 2);
            let cpu_usage = report.renderer_cpu().unwrap();
            assert_eq!(
                cpu_usage.usage().live(RenderResource::RetainedCpuBytes),
                opening
            );
            assert_eq!(
                cpu_usage
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .live(RenderResource::RetainedCpuBytes),
                0
            );
            assert!(
                cpu_usage
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .peak(RenderResource::RetainedCpuBytes)
                    > 0
            );
            drop(imported);
            // The caller's font and authored scene still exist. Neither keeps
            // the renderer's imported-payload lease hidden in a public value.
            assert_eq!(
                cpu.snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes)
                    + font.admitted_face().resource().retained_bytes() as u64,
                opening,
            );
            assert_eq!(scene.len(), 1);
        }

        fn attempt_status(error: &Error) -> RenderAttemptStatus {
            assert_eq!(error.code(), ErrorCode::Accounting);
            RenderAttemptStatus::Refused
        }

        #[test]
        fn paint_copy_refusal_after_glyph_copy_releases_all_normalized_backing() {
            let options = options(RenderWork::PaintVisits, 1);
            let cpu = RendererCpu::new(options).unwrap();
            let opening = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let scene = text_scene(FontRef::new(1).named("copied glyphs"));
            let (_, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let error = scene
                        .normalize(Capabilities::CURRENT, attempt.account())
                        .unwrap_err();
                    refusal(&error, RenderWork::PaintVisits, 1);

                    assert_eq!(attempt_status(&error), RenderAttemptStatus::Refused);
                    Err::<(), _>(error)
                })
                .unwrap()
                .into_parts();
            assert_eq!(fault, None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::PaintVisits), 1);
            // Both glyphs were validated and then actually copied before paint.
            assert_eq!(usage.work(RenderWork::GlyphVisits), 4);
            // The immutable gradient needs no validation walk, and this paint
            // visit refuses before its stop-copy loop starts.
            assert_eq!(usage.work(RenderWork::GradientStopVisits), 0);
            let cpu_usage = report.renderer_cpu().unwrap();
            assert_eq!(
                cpu_usage.usage().live(RenderResource::RetainedCpuBytes),
                opening
            );
            assert_eq!(
                cpu_usage
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .live(RenderResource::RetainedCpuBytes),
                0
            );
            assert!(
                cpu_usage
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .peak(RenderResource::RetainedCpuBytes)
                    >= (2 * std::mem::size_of::<TextGlyph>()) as u64
            );
            assert_eq!(scene.len(), 1);
        }
    }
}
