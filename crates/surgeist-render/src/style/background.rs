use super::super::{
    Capabilities, Color, Error, Image, Paint, PrimitiveFamily, PrimitiveOperation, Rect, Result,
    UnsupportedPrimitive,
};
use super::image::{
    BackgroundAreas, BackgroundClipGeometry, ImageAttachmentPlan, ImagePlacementInput,
    ImageRepeatPlan, ResolvedImagePlacement, ResolvedImageRepeat, ResolvedImageResource,
    StyleImageLayer, StyleImageSource, StyleImageSourceKind,
};
use crate::{
    accounting::{
        Account, AccountedVec, Hold, RenderAccountingContext, RenderAttemptKind, RenderPhase,
        RenderResource, RenderWork,
    },
    paint::PaintKind,
};

/// Operation-local custody for the plain standalone output. It is disarmed only
/// after every nested normalization succeeds; scratch registry backing is dropped.
pub(crate) struct StyleCustody<'a> {
    holds: AccountedVec<Hold>,
    images: AccountedVec<&'a Image>,
}
impl<'a> StyleCustody<'a> {
    pub(crate) fn new(account: &Account) -> Result<Self> {
        Ok(Self {
            holds: account.temporary(0)?,
            images: account.temporary(0)?,
        })
    }
    pub(crate) fn hold(&mut self, hold: Hold) -> Result<()> {
        self.holds.push(hold)
    }
    pub(crate) fn optional_hold(&mut self, hold: Option<Hold>) -> Result<()> {
        if let Some(hold) = hold {
            self.hold(hold)?;
        }
        Ok(())
    }
    pub(crate) fn image(&mut self, image: &'a Image, account: &Account) -> Result<()> {
        for previous in self.images.iter() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if std::sync::Arc::ptr_eq(&previous.bytes, &image.bytes) {
                return Ok(());
            }
        }
        let hold = account.acquire(RenderResource::RetainedCpuBytes, image.bytes.len())?;
        self.images.push(image)?;
        self.hold(hold)
    }
    pub(crate) fn paint(&mut self, paint: &'a Paint, account: &Account) -> Result<Paint> {
        crate::validation::validate_paint_in(paint, account)?;
        let (copied, hold) = paint.try_clone_in(account)?;
        self.optional_hold(hold)?;
        if let PaintKind::Image(image) = paint.kind() {
            self.image(image, account)?;
        }
        Ok(copied)
    }
    pub(crate) fn transfer<T>(mut self, payload: T) -> Result<T> {
        let renderer = self.holds.iter().find_map(|hold| match hold {
            Hold::Renderer(hold) => Some(hold.account()),
            Hold::Preparation(_) => None,
        });
        if let Some(renderer) = renderer {
            let error =
                Error::from(crate::accounting::StorageContractError::WrongRetentionLifetime);
            return renderer.maintenance_phase(move || {
                // The real offending owner observes payload destruction while
                // all credits remain live, then disposes the intact registry.
                drop(payload);
                drop(self);
                Err(error)
            });
        }
        // The read-only closed-variant walk proved all leases Preparation.
        // No mutation can change a lease's lifetime before its consumed transfer.
        while let Some(hold) = self.holds.pop() {
            hold.into_preparation()?.retain();
        }
        Ok(payload)
    }
}

#[derive(Clone, Debug, PartialEq)]
/// One authored background layer backed by an image-style layer.
pub struct BackgroundLayer {
    image: StyleImageLayer,
}

impl BackgroundLayer {
    /// Creates a background layer from its authored image-layer values.
    #[must_use]
    pub const fn new(image: StyleImageLayer) -> Self {
        Self { image }
    }

    /// Returns the image-layer description.
    #[must_use]
    pub const fn image(&self) -> &StyleImageLayer {
        &self.image
    }
}

#[derive(Clone, Debug, PartialEq)]
/// An authored background color and ordered image layers.
///
/// At least a color or one layer is present. During normalization, the color is
/// emitted first and layers are emitted in reverse of this stored order.
pub struct BackgroundStack {
    color: Option<Color>,
    layers: Vec<BackgroundLayer>,
}

impl BackgroundStack {
    /// Creates a non-empty background stack.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] when both `color` is absent
    /// and `layers` is empty.
    pub fn try_new(color: Option<Color>, layers: Vec<BackgroundLayer>) -> Result<Self> {
        if color.is_none() && layers.is_empty() {
            return Err(Error::invalid_value(
                "background stack",
                "none + []",
                "must include a color or at least one layer",
            ));
        }
        Ok(Self { color, layers })
    }

    /// Returns the optional concrete background color.
    #[must_use]
    pub const fn color(&self) -> Option<Color> {
        self.color
    }

    /// Returns the authored layers in their stored order.
    #[must_use]
    pub fn layers(&self) -> &[BackgroundLayer] {
        &self.layers
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A non-empty authored list of blend modes for background layers.
///
/// The current rendering contract accepts only [`BackgroundBlendMode::Normal`]
/// entries; other modeled choices produce an unsupported-primitive diagnostic.
pub struct BackgroundBlendList {
    modes: Vec<BackgroundBlendMode>,
}

impl BackgroundBlendList {
    /// Validates a background blend-mode list.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for an empty list and an
    /// unsupported-primitive error when any mode is not `Normal`.
    pub fn try_new(modes: Vec<BackgroundBlendMode>) -> Result<Self> {
        if modes.is_empty() {
            return Err(Error::invalid_value(
                "background blend list",
                "[]",
                "must contain at least one mode",
            ));
        }
        if modes
            .iter()
            .any(|mode| *mode != BackgroundBlendMode::Normal)
        {
            return Err(Error::unsupported_render_primitive(
                UnsupportedPrimitive::new(
                    PrimitiveFamily::Compositing,
                    PrimitiveOperation::BackgroundBlendMode,
                ),
            ));
        }
        Ok(Self { modes })
    }

    /// Returns the validated modes in authored order.
    #[must_use]
    pub fn modes(&self) -> &[BackgroundBlendMode] {
        &self.modes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// An authored blend choice between background layers.
pub enum BackgroundBlendMode {
    /// Standard source-over blending.
    Normal,
    /// Multiply blending; currently reported as unsupported by list validation.
    Multiply,
    /// Screen blending; currently reported as unsupported by list validation.
    Screen,
    /// Overlay blending; currently reported as unsupported by list validation.
    Overlay,
    /// Darken blending; currently reported as unsupported by list validation.
    Darken,
    /// Lighten blending; currently reported as unsupported by list validation.
    Lighten,
    /// Additive blending; currently reported as unsupported by list validation.
    Plus,
}

#[derive(Clone, Debug, PartialEq)]
/// Context supplied to normalize a background stack into render-facing commands.
///
/// The background areas provide the logical border, padding, and content boxes.
/// Per-layer clip overrides may replace the clip derived from each layer.
pub struct BackgroundNormalizationInput {
    stack: BackgroundStack,
    areas: BackgroundAreas,
    layer_clip_overrides: Vec<Option<BackgroundClipGeometry>>,
}

#[derive(Clone, Debug, PartialEq)]
/// An intrinsically validated, ordered sequence of background render commands.
pub struct NormalizedBackgroundStack {
    commands: Vec<NormalizedBackgroundCommand>,
}

#[derive(Clone, Debug, PartialEq)]
/// One normalized background command paired with its clip geometry.
pub struct NormalizedBackgroundCommand {
    clip: BackgroundClipGeometry,
    kind: NormalizedBackgroundCommandKind,
}

#[derive(Clone, Debug, PartialEq)]
/// A normalized background layer with resolved placement, repetition, and attachment.
pub struct NormalizedBackgroundLayer {
    source: NormalizedBackgroundLayerSource,
    placement: ResolvedImagePlacement,
    repeat: ResolvedImageRepeat,
    attachment: ImageAttachmentPlan,
}

#[derive(Clone, Debug, PartialEq)]
/// The render-facing source retained by a normalized background layer.
pub enum NormalizedBackgroundLayerSource {
    /// A validated paint source.
    Paint(Paint),
    /// A validated image carrying pixel content.
    Image(Image),
    /// An image resource already resolved by the surrounding context.
    ResolvedImage(ResolvedImageResource),
}

#[derive(Clone, Debug, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "background commands keep their planned public shape for direct matching"
)]
/// The operation represented by a normalized background command.
pub enum NormalizedBackgroundCommandKind {
    /// Fills a logical rectangle with a concrete color.
    ColorFill {
        /// The logical rectangle to fill.
        rect: Rect,
        /// The concrete fill color.
        color: Color,
    },
    /// Draws one normalized image-style layer.
    Layer {
        /// The normalized layer payload.
        layer: NormalizedBackgroundLayer,
    },
}

impl BackgroundNormalizationInput {
    /// Creates normalization input with no per-layer clip overrides.
    pub fn try_new(stack: BackgroundStack, areas: BackgroundAreas) -> Result<Self> {
        let layer_clip_overrides = vec![None; stack.layers().len()];
        Ok(Self {
            stack,
            areas,
            layer_clip_overrides,
        })
    }

    /// Returns the authored background stack.
    #[must_use]
    pub fn stack(&self) -> &BackgroundStack {
        &self.stack
    }

    /// Returns the logical background areas used during normalization.
    #[must_use]
    pub const fn areas(&self) -> BackgroundAreas {
        self.areas
    }

    /// Returns one optional clip override for each background layer.
    #[must_use]
    pub fn layer_clip_overrides(&self) -> &[Option<BackgroundClipGeometry>] {
        &self.layer_clip_overrides
    }

    /// Replaces the per-layer clip overrides.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] unless the override count
    /// equals the background layer count. `None` keeps that layer's authored
    /// background-box clip.
    pub fn with_layer_clip_overrides(
        mut self,
        layer_clip_overrides: Vec<Option<BackgroundClipGeometry>>,
    ) -> Result<Self> {
        if layer_clip_overrides.len() != self.stack.layers().len() {
            return Err(Error::invalid_value(
                "background layer clip overrides",
                layer_clip_overrides.len(),
                "must match background layer count",
            ));
        }
        self.layer_clip_overrides = layer_clip_overrides;
        Ok(self)
    }

    /// Normalizes the stack using the supplied semantic capability contract.
    ///
    /// The optional color command is emitted first, followed by image layers in
    /// reverse stored order. Symbolic image references return an unresolved-resource
    /// diagnostic; invalid source, placement, repeat, attachment, or clip data and
    /// currently unsupported choices return their corresponding typed errors.
    pub fn normalize(
        &self,
        capabilities: Capabilities,
        context: &mut RenderAccountingContext,
    ) -> Result<NormalizedBackgroundStack> {
        context.run(
            RenderAttemptKind::Background,
            RenderPhase::Normalize,
            |account| {
                let mut custody = StyleCustody::new(account)?;
                let value = self.normalize_in(capabilities, account, &mut custody)?;
                custody.transfer(value)
            },
        )
    }

    pub(crate) fn normalize_in<'a>(
        &'a self,
        capabilities: Capabilities,
        account: &Account,
        custody: &mut StyleCustody<'a>,
    ) -> Result<NormalizedBackgroundStack> {
        let scoped = account.at(RenderPhase::Normalize);
        scoped.phase(RenderPhase::Normalize, || {
            self.normalize_body(capabilities, &scoped, custody)
        })
    }

    fn normalize_body<'a>(
        &'a self,
        capabilities: Capabilities,
        account: &Account,
        custody: &mut StyleCustody<'a>,
    ) -> Result<NormalizedBackgroundStack> {
        let mut commands = account.retained(0)?;
        if let Some(color) = self.stack.color() {
            let rect = self.areas.border_box();
            let command = NormalizedBackgroundCommand {
                clip: BackgroundClipGeometry::try_rect(rect)?,
                kind: NormalizedBackgroundCommandKind::ColorFill { rect, color },
            };
            commands.grow(1)?;
            account.work(RenderWork::CommandsEmitted, 1)?;
            account.work(RenderWork::FillsEmitted, 1)?;
            commands.push(command)?;
        }

        for (layer_index, layer) in self.stack.layers().iter().enumerate().rev() {
            account.work(RenderWork::CommandVisits, 1)?;
            let command =
                self.normalize_layer(layer_index, layer.image(), capabilities, account, custody)?;
            commands.grow(1)?;
            account.work(RenderWork::CommandsEmitted, 1)?;
            if let NormalizedBackgroundCommandKind::Layer { layer } = &command.kind {
                match &layer.source {
                    NormalizedBackgroundLayerSource::Image(_) => {
                        account.work(RenderWork::ImagesEmitted, 1)?
                    }
                    NormalizedBackgroundLayerSource::Paint(paint)
                        if matches!(paint.kind(), PaintKind::Image(_)) =>
                    {
                        account.work(RenderWork::ImagesEmitted, 1)?
                    }
                    _ => {}
                }
            }
            commands.push(command)?;
        }

        let (commands, hold) = commands.into_parts()?;
        custody.hold(hold)?;
        Ok(NormalizedBackgroundStack { commands })
    }

    fn normalize_layer<'a>(
        &'a self,
        layer_index: usize,
        layer: &'a StyleImageLayer,
        capabilities: Capabilities,
        account: &Account,
        custody: &mut StyleCustody<'a>,
    ) -> Result<NormalizedBackgroundCommand> {
        let clip = self.layer_clip_geometry(layer_index, layer, account, custody)?;
        let origin_rect = self.areas.rect_for(layer.origin());
        let (source, intrinsic_size) = match layer.source().kind() {
            StyleImageSourceKind::Paint(paint) => (
                NormalizedBackgroundLayerSource::Paint(custody.paint(paint, account)?),
                origin_rect.size(),
            ),
            StyleImageSourceKind::Image(image) => {
                custody.image(image, account)?;
                account.work(RenderWork::ImageIdentityCopies, 1)?;
                (
                    NormalizedBackgroundLayerSource::Image(image.clone()),
                    image.size(),
                )
            }
            StyleImageSourceKind::Resolved(resource) => (
                NormalizedBackgroundLayerSource::ResolvedImage(resource.clone()),
                resource.intrinsic_size(),
            ),
            StyleImageSourceKind::Unresolved(reference) => {
                return Err(StyleImageSource::unresolved_error(reference));
            }
        };
        let placement = ImagePlacementInput::try_new(
            origin_rect,
            intrinsic_size,
            layer.position(),
            layer.size(),
        )?
        .resolve_in(account)?;
        let (repeat, hold) = ImageRepeatPlan::try_new(layer.repeat(), capabilities)?
            .resolve_in(placement, account)?;
        custody.hold(hold)?;
        let attachment =
            ImageAttachmentPlan::try_new(layer.attachment(), layer.coordinate_space())?;
        Ok(NormalizedBackgroundCommand {
            clip,
            kind: NormalizedBackgroundCommandKind::Layer {
                layer: NormalizedBackgroundLayer {
                    source,
                    placement,
                    repeat,
                    attachment,
                },
            },
        })
    }

    fn layer_clip_geometry(
        &self,
        layer_index: usize,
        layer: &StyleImageLayer,
        account: &Account,
        custody: &mut StyleCustody<'_>,
    ) -> Result<BackgroundClipGeometry> {
        if let Some(override_clip) = &self.layer_clip_overrides[layer_index] {
            let (clip, hold) = override_clip.try_clone_in(account)?;
            custody.optional_hold(hold)?;
            return Ok(clip);
        }
        BackgroundClipGeometry::try_rect(self.areas.rect_for(layer.clip()))
    }
}

impl NormalizedBackgroundStack {
    /// Returns the normalized commands in rendering order.
    #[must_use]
    pub fn commands(&self) -> &[NormalizedBackgroundCommand] {
        &self.commands
    }
}

impl NormalizedBackgroundCommand {
    /// Returns the geometry that clips this command.
    #[must_use]
    pub fn clip(&self) -> &BackgroundClipGeometry {
        &self.clip
    }

    /// Returns the normalized operation payload.
    #[must_use]
    pub fn kind(&self) -> &NormalizedBackgroundCommandKind {
        &self.kind
    }
}

impl NormalizedBackgroundLayer {
    /// Returns the validated or context-resolved layer source.
    #[must_use]
    pub fn source(&self) -> &NormalizedBackgroundLayerSource {
        &self.source
    }

    /// Returns the resolved logical paint and first-tile rectangles.
    #[must_use]
    pub const fn placement(&self) -> ResolvedImagePlacement {
        self.placement
    }

    /// Returns the resolved clip rectangle and ordered tile rectangles.
    #[must_use]
    pub fn repeat(&self) -> &ResolvedImageRepeat {
        &self.repeat
    }

    /// Returns the validated attachment and coordinate-space plan.
    #[must_use]
    pub const fn attachment(&self) -> ImageAttachmentPlan {
        self.attachment
    }
}

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::{BackgroundRepeat, Size, StyleImageSource};

    #[test]
    fn nested_repeats_join_background_attempt_and_share_source_backing() {
        let image = Image::from_rgba(Size::new(1.0, 1.0), vec![10, 20, 30, 255]).unwrap();
        let layer = BackgroundLayer::new(
            StyleImageLayer::try_new(StyleImageSource::image(image).unwrap())
                .unwrap()
                .with_repeat(BackgroundRepeat::no_repeat()),
        );
        let stack = BackgroundStack::try_new(None, vec![layer.clone(), layer]).unwrap();
        let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
        let input = BackgroundNormalizationInput::try_new(
            stack,
            BackgroundAreas::try_new(rect, rect, rect).unwrap(),
        )
        .unwrap();
        let mut context = RenderAccountingContext::default();
        let normalized = input
            .normalize(Capabilities::CURRENT, &mut context)
            .unwrap();
        assert_eq!(normalized.commands().len(), 2);
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.kind(), RenderAttemptKind::Background);
        assert_eq!(report.usage().work(RenderWork::RepeatTiles), 2);
        assert_eq!(report.usage().work(RenderWork::CommandsEmitted), 2);
        assert_eq!(report.usage().work(RenderWork::ImagesEmitted), 2);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
    }

    #[test]
    fn scene_normalization_counts_emissions_once_and_shared_clones_add_no_emissions() {
        use crate::{Color, ImageFit, Layer, Scene};
        let image = Image::from_rgba(Size::new(1.0, 1.0), vec![10, 20, 30, 255]).unwrap();
        let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
        let mut scene = Scene::new();
        scene.fill(rect, Color::BLACK);
        scene.layer(Layer::new(), |children| {
            children.fill(rect, Color::BLACK);
            children.image(image, rect, ImageFit::Fill);
        });
        let account = Account::for_test();
        let normalized = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        assert_eq!(account.work_count(RenderWork::CommandsEmitted).unwrap(), 4);
        assert_eq!(account.work_count(RenderWork::FillsEmitted).unwrap(), 2);
        assert_eq!(account.work_count(RenderWork::ImagesEmitted).unwrap(), 1);
        assert_eq!(account.work_count(RenderWork::LayersEmitted).unwrap(), 1);
        assert_eq!(account.work_count(RenderWork::AuthoredCommands).unwrap(), 0);
        assert_eq!(account.work_count(RenderWork::CommandVisits).unwrap(), 6);
        let clone = normalized.clone();
        assert_eq!(clone, normalized);
        assert_eq!(account.work_count(RenderWork::CommandsEmitted).unwrap(), 4);
        assert_eq!(account.work_count(RenderWork::CommandVisits).unwrap(), 6);
    }
}

#[cfg(test)]
mod vector_custody_tests;
