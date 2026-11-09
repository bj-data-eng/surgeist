use super::{
    BackdropFilterInput, ClipInput, Error, Image, Paint, Point, Rect, Result, Shape, Transform,
    style::validate_clip_input,
    validation::{
        validate_filter, validate_finite_f64, validate_non_negative_f64, validate_paint,
        validate_point, validate_positive_f64, validate_shape, validate_transform,
    },
};

/// Authored grouping parameters applied to a nested scene.
///
/// A layer can carry one clip, one authored or resolved mask, one foreground
/// filter, one backdrop-filter input, a finite logical transform, a finite
/// opacity, and a complete composition operation. Explicit isolation starts the
/// group's children against transparent black. The default is an unmodified
/// normal layer.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    clip: Option<ClipInput>,
    transform: Transform,
    opacity: f32,
    composition: LayerComposition,
    isolated: bool,
    mask: Option<LayerMask>,
    filter: Option<Filter>,
    backdrop_filter: Option<Box<BackdropFilterInput>>,
}

impl Layer {
    #[must_use]
    /// Creates the default unmodified layer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this layer with an authored shape clip, replacing any prior clip.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] when the shape is invalid.
    pub fn try_clip(self, clip: Shape) -> Result<Self> {
        validate_shape(&clip)?;
        Ok(self.with_shape_clip(clip))
    }

    /// Installs a Shape whose construction already establishes intrinsic validity.
    pub(crate) fn with_shape_clip(mut self, clip: Shape) -> Self {
        self.clip = Some(ClipInput::from_validated_shape(clip));
        self
    }

    /// Returns this layer with a validated authored clip input, replacing any prior clip.
    ///
    /// Invalid clip geometry or coordinate-space facts return a typed diagnostic.
    pub fn try_clip_input(mut self, clip: ClipInput) -> Result<Self> {
        validate_clip_input(&clip)?;
        self.clip = Some(clip);
        Ok(self)
    }

    /// Returns this layer with an authored shape mask, replacing any prior mask.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] when the shape is invalid.
    pub fn try_mask(mut self, mask: Shape) -> Result<Self> {
        validate_shape(&mask)?;
        self.mask = Some(LayerMask::AuthoredShape(mask));
        Ok(self)
    }

    /// Installs an already validated resolved alpha mask for GPU-graph execution.
    ///
    /// This infallible transition accepts only a [`ResolvedLayerAlphaMask`]; use
    /// its fallible constructor to validate authored bounds first. It replaces
    /// any previously installed authored or resolved mask.
    #[must_use]
    pub fn with_resolved_alpha_mask(mut self, alpha_mask: ResolvedLayerAlphaMask) -> Self {
        self.mask = Some(LayerMask::ResolvedAlpha(alpha_mask));
        self
    }

    /// Returns this layer with a validated foreground blur filter, replacing any prior filter.
    ///
    /// Returns a typed input diagnostic if canonical filter validation fails.
    pub fn try_filter(mut self, filter: Filter) -> Result<Self> {
        validate_filter(filter)?;
        self.filter = Some(filter);
        Ok(self)
    }

    /// Returns this layer with the supplied validated backdrop-filter input.
    ///
    /// The new input replaces any previous backdrop-filter input. This builder
    /// currently performs no additional fallible validation and returns `Ok`.
    pub fn try_backdrop_filter(mut self, backdrop_filter: BackdropFilterInput) -> Result<Self> {
        self.backdrop_filter = Some(Box::new(backdrop_filter));
        Ok(self)
    }

    /// Returns this layer with a finite logical transform, replacing the previous transform.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for a non-finite transform.
    pub fn try_transform(mut self, transform: Transform) -> Result<Self> {
        validate_transform(transform, "layer transform")?;
        self.transform = transform;
        Ok(self)
    }

    #[must_use]
    /// Replaces the complete composition with source-over blending or local Plus.
    ///
    /// This replaces any previously selected Porter-Duff operator.
    pub const fn blend(mut self, blend: BlendMode) -> Self {
        self.composition = LayerComposition::from_blend(blend);
        self
    }

    /// Replaces this layer's complete validated composition operation.
    #[must_use]
    pub const fn with_composition(mut self, composition: LayerComposition) -> Self {
        self.composition = composition;
        self
    }

    /// Forces children to compose against a transparent initial group backdrop.
    ///
    /// This is a resolved execution fact; callers decide which semantic groups
    /// require isolation. The default layer permits transparent wrappers to flatten.
    #[must_use]
    pub const fn isolated(mut self) -> Self {
        self.isolated = true;
        self
    }

    /// Returns whether an isolated initial group backdrop was explicitly requested.
    #[must_use]
    pub const fn is_isolated(&self) -> bool {
        self.isolated
    }

    /// Returns the complete composition, including its selected basic operator.
    #[must_use]
    pub const fn composition(&self) -> LayerComposition {
        self.composition
    }

    /// Returns this layer with a finite opacity value.
    ///
    /// This boundary rejects non-finite values but does not clamp or otherwise
    /// restrict the numeric range.
    pub fn try_opacity(mut self, opacity: f32) -> Result<Self> {
        if !opacity.is_finite() {
            return Err(Error::invalid_value(
                "layer opacity",
                opacity,
                "must be finite",
            ));
        }
        self.opacity = opacity;
        Ok(self)
    }

    #[must_use]
    /// Returns the authored shape clip, if the clip input contains a shape.
    pub fn clip(&self) -> Option<&Shape> {
        self.clip.as_ref().and_then(ClipInput::shape)
    }

    #[must_use]
    /// Returns the complete authored clip input, if present.
    pub fn clip_input(&self) -> Option<&ClipInput> {
        self.clip.as_ref()
    }

    #[must_use]
    /// Returns the authored shape mask, excluding resolved alpha masks.
    pub fn mask(&self) -> Option<&Shape> {
        match &self.mask {
            Some(LayerMask::AuthoredShape(mask)) => Some(mask),
            Some(LayerMask::ResolvedAlpha(_)) | None => None,
        }
    }

    #[must_use]
    /// Returns the resolved image alpha mask, excluding authored shape masks.
    pub fn resolved_alpha_mask(&self) -> Option<&ResolvedLayerAlphaMask> {
        match &self.mask {
            Some(LayerMask::ResolvedAlpha(mask)) => Some(mask),
            Some(LayerMask::AuthoredShape(_)) | None => None,
        }
    }

    #[must_use]
    /// Returns the foreground blur filter, if present.
    pub const fn filter(&self) -> Option<Filter> {
        self.filter
    }

    #[must_use]
    /// Returns the authored backdrop-filter input, if present.
    pub fn backdrop_filter(&self) -> Option<&BackdropFilterInput> {
        match &self.backdrop_filter {
            Some(backdrop_filter) => Some(backdrop_filter.as_ref()),
            None => None,
        }
    }

    #[must_use]
    /// Returns the layer's logical transform.
    pub const fn transform(&self) -> Transform {
        self.transform
    }

    #[must_use]
    /// Returns the finite, unclamped layer opacity.
    pub const fn opacity(&self) -> f32 {
        self.opacity
    }

    #[must_use]
    /// Returns the layer's blend mode.
    pub const fn blend_mode(&self) -> BlendMode {
        self.composition.blend_mode()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LayerMask {
    AuthoredShape(Shape),
    ResolvedAlpha(ResolvedLayerAlphaMask),
}

/// A resolved image alpha channel mapped across finite positive layer-local logical bounds.
///
/// The render crate owns this resolved input. The GPU graph samples the image's
/// alpha using its retained identity, quality, and extend policy; it does not
/// materialize mask pixels on the CPU.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedLayerAlphaMask {
    image: Image,
    bounds: Rect,
}

impl ResolvedLayerAlphaMask {
    /// Creates a resolved mask with positive finite bounds in layer-local logical space.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for a non-finite origin or
    /// maximum, or for a non-positive width or height.
    pub fn try_new(image: Image, bounds: Rect) -> Result<Self> {
        validate_point(bounds.origin(), "resolved layer alpha mask bounds")?;
        validate_positive_f64(bounds.width(), "resolved layer alpha mask bounds width")?;
        validate_positive_f64(bounds.height(), "resolved layer alpha mask bounds height")?;
        validate_point(bounds.max(), "resolved layer alpha mask bounds maximum")?;
        Ok(Self { image, bounds })
    }

    /// Returns the retained mask image, including its identity and sampling policy.
    #[must_use]
    pub const fn image(&self) -> &Image {
        &self.image
    }

    /// Returns the semantic rectangle in the owning layer's local logical coordinates.
    #[must_use]
    pub const fn bounds(&self) -> Rect {
        self.bounds
    }
}

/// Creates an unmodified layer with identity transform, opacity `1`, normal
/// blending, and no clip, mask, or filter.
impl Default for Layer {
    fn default() -> Self {
        Self {
            clip: None,
            transform: Transform::identity(),
            opacity: 1.0,
            composition: LayerComposition::from_blend(BlendMode::Normal),
            isolated: false,
            mask: None,
            filter: None,
            backdrop_filter: None,
        }
    }
}

/// The compositing operation used to combine a layer with its backdrop.
///
/// Mixing modes operate on straight numeric-sRGB components, then use
/// source-over composition with backdrop-alpha modulation. [`Self::Plus`] is
/// the separate additive composition choice. The default is [`Self::Normal`].
/// Pair a mixing function with a basic operator through [`LayerComposition`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlendMode {
    /// Standard source-over compositing.
    #[default]
    Normal,
    /// Multiply source and backdrop colors.
    Multiply,
    /// Apply screen blending.
    Screen,
    /// Apply overlay blending.
    Overlay,
    /// Select the darker source or backdrop contribution.
    Darken,
    /// Select the lighter source or backdrop contribution.
    Lighten,
    /// Brighten the backdrop with guarded straight-color division.
    ColorDodge,
    /// Darken the backdrop with guarded straight-color division.
    ColorBurn,
    /// Select multiply or screen from the straight source component.
    HardLight,
    /// Apply the polynomial/square-root soft-light curve.
    SoftLight,
    /// Take the absolute difference of straight source and backdrop components.
    Difference,
    /// Apply the lower-contrast straight-color exclusion function.
    Exclusion,
    /// Combine source hue with backdrop saturation and luminosity.
    Hue,
    /// Combine source saturation with backdrop hue and luminosity.
    Saturation,
    /// Combine source hue and saturation with backdrop luminosity.
    Color,
    /// Combine backdrop hue and saturation with source luminosity.
    Luminosity,
    /// Add source and backdrop contributions using plus composition.
    Plus,
}

/// One of the twelve basic Porter-Duff region selections.
///
/// These operators have bounded premultiplied color and alpha for valid inputs.
/// Normative lighter's unresolved output range is outside this set; the existing
/// local [`BlendMode::Plus`] operation remains separately selectable.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PorterDuffOperator {
    /// Discard source and destination.
    Clear,
    /// Replace destination with source, including transparent source samples.
    Copy,
    /// Preserve destination.
    Destination,
    /// Place source over destination.
    #[default]
    SourceOver,
    /// Place destination over source.
    DestinationOver,
    /// Keep source only where destination contributes alpha.
    SourceIn,
    /// Keep destination only where source contributes alpha.
    DestinationIn,
    /// Keep source only outside destination alpha.
    SourceOut,
    /// Keep destination only outside source alpha.
    DestinationOut,
    /// Place source atop destination, retaining destination alpha.
    SourceAtop,
    /// Place destination atop source, retaining source alpha.
    DestinationAtop,
    /// Keep the non-overlapping source and destination contributions.
    Xor,
}

/// One validated complete layer composition operation.
///
/// A basic Porter-Duff operation pairs one mixing function with one operator.
/// Local Plus is a separate complete operation, selected through [`Layer::blend`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayerComposition {
    kind: LayerCompositionKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LayerCompositionKind {
    Basic {
        operator: PorterDuffOperator,
        blend: BlendMode,
    },
    Plus,
}

impl LayerComposition {
    /// Pairs a basic operator with one of the sixteen mixing functions.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for [`BlendMode::Plus`], which
    /// is a complete operation rather than a color mixing function.
    pub fn try_porter_duff(operator: PorterDuffOperator, blend: BlendMode) -> Result<Self> {
        if blend == BlendMode::Plus {
            return Err(Error::invalid_value(
                "Porter-Duff mixing function",
                "Plus",
                "must be a color mixing function; Plus is a complete operation",
            ));
        }
        Ok(Self {
            kind: LayerCompositionKind::Basic { operator, blend },
        })
    }

    /// Returns the basic operator, or `None` for the separate local Plus operation.
    #[must_use]
    pub const fn operator(self) -> Option<PorterDuffOperator> {
        match self.kind {
            LayerCompositionKind::Basic { operator, .. } => Some(operator),
            LayerCompositionKind::Plus => None,
        }
    }

    /// Returns the mixing function, or [`BlendMode::Plus`] for local Plus.
    #[must_use]
    pub const fn blend_mode(self) -> BlendMode {
        match self.kind {
            LayerCompositionKind::Basic { blend, .. } => blend,
            LayerCompositionKind::Plus => BlendMode::Plus,
        }
    }

    pub(crate) const fn from_blend(blend: BlendMode) -> Self {
        Self {
            kind: match blend {
                BlendMode::Plus => LayerCompositionKind::Plus,
                blend => LayerCompositionKind::Basic {
                    operator: PorterDuffOperator::SourceOver,
                    blend,
                },
            },
        }
    }

    pub(crate) const fn is_plus(self) -> bool {
        matches!(self.kind, LayerCompositionKind::Plus)
    }

    pub(crate) const fn is_source_over(self) -> bool {
        matches!(
            self.kind,
            LayerCompositionKind::Basic {
                operator: PorterDuffOperator::SourceOver,
                ..
            }
        )
    }

    pub(crate) const fn is_source_over_normal(self) -> bool {
        matches!(
            self.kind,
            LayerCompositionKind::Basic {
                operator: PorterDuffOperator::SourceOver,
                blend: BlendMode::Normal,
            }
        )
    }

    pub(crate) const fn transparent_source_preserves_parent(self) -> bool {
        matches!(
            self.kind,
            LayerCompositionKind::Plus
                | LayerCompositionKind::Basic {
                    operator: PorterDuffOperator::Destination
                        | PorterDuffOperator::SourceOver
                        | PorterDuffOperator::DestinationOver
                        | PorterDuffOperator::DestinationOut
                        | PorterDuffOperator::SourceAtop
                        | PorterDuffOperator::Xor,
                    ..
                }
        )
    }
}

impl Default for LayerComposition {
    fn default() -> Self {
        Self::from_blend(BlendMode::Normal)
    }
}

/// A validated foreground Gaussian-blur request in logical units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Filter {
    radius: f64,
}

impl Filter {
    /// Creates a blur filter with a finite, non-negative logical radius.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for a negative or non-finite radius.
    pub fn try_blur(radius: f64) -> Result<Self> {
        validate_non_negative_f64(radius, "layer blur radius")?;
        Ok(Self { radius })
    }

    #[must_use]
    /// Returns the blur radius in logical units.
    pub const fn blur_radius(self) -> f64 {
        self.radius
    }
}

/// An authored outer or inset shadow for a rendered shape or text run.
///
/// Offsets and spread use logical units, blur is finite and non-negative, and
/// the paint is validated at construction.
#[derive(Clone, Debug, PartialEq)]
pub struct Shadow {
    kind: ShadowKind,
    offset: Point,
    blur: f64,
    spread: f64,
    paint: Paint,
}

impl Shadow {
    fn new(
        kind: ShadowKind,
        offset: Point,
        blur: f64,
        spread: f64,
        paint: impl Into<Paint>,
    ) -> Self {
        Self {
            kind,
            offset,
            blur,
            spread,
            paint: paint.into(),
        }
    }

    /// Creates a validated outer shadow.
    ///
    /// Invalid offset, blur, spread, or paint returns a typed input diagnostic.
    pub fn try_new(offset: Point, blur: f64, spread: f64, paint: impl Into<Paint>) -> Result<Self> {
        Self::try_with_kind(ShadowKind::Outer, offset, blur, spread, paint)
    }

    /// Creates a validated inset shadow.
    ///
    /// Invalid offset, blur, spread, or paint returns a typed input diagnostic.
    pub fn try_inset(
        offset: Point,
        blur: f64,
        spread: f64,
        paint: impl Into<Paint>,
    ) -> Result<Self> {
        Self::try_with_kind(ShadowKind::Inset, offset, blur, spread, paint)
    }

    /// Creates a validated shadow with an explicit outer or inset kind.
    ///
    /// `blur` must be finite and non-negative; `offset` and `spread` must be
    /// finite, and the paint must be valid.
    pub fn try_with_kind(
        kind: ShadowKind,
        offset: Point,
        blur: f64,
        spread: f64,
        paint: impl Into<Paint>,
    ) -> Result<Self> {
        validate_point(offset, "shadow offset")?;
        validate_non_negative_f64(blur, "shadow blur")?;
        validate_finite_f64(spread, "shadow spread")?;
        let paint = paint.into();
        validate_paint(&paint)?;
        Ok(Self::new(kind, offset, blur, spread, paint))
    }

    #[must_use]
    /// Returns whether this is an outer or inset shadow.
    pub const fn kind(&self) -> ShadowKind {
        self.kind
    }

    #[must_use]
    /// Returns the shadow offset in logical units.
    pub const fn offset(&self) -> Point {
        self.offset
    }

    #[must_use]
    /// Returns the finite, non-negative blur radius in logical units.
    pub const fn blur(&self) -> f64 {
        self.blur
    }

    #[must_use]
    /// Returns the finite spread distance in logical units.
    pub const fn spread(&self) -> f64 {
        self.spread
    }

    #[must_use]
    /// Returns the shadow paint.
    pub const fn paint(&self) -> &Paint {
        &self.paint
    }
}

/// Placement of a shadow relative to its source geometry.
///
/// The default is [`Self::Outer`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ShadowKind {
    /// Render the shadow outside the source geometry.
    #[default]
    Outer,
    /// Render the shadow inside the source geometry.
    Inset,
}

/// A validated non-empty authored sequence of shadows.
///
/// Shadows retain their supplied order for rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowList {
    shadows: Vec<Shadow>,
}

impl ShadowList {
    /// Creates an ordered, non-empty list of valid shadows.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for an empty list or invalid
    /// shadow geometry, blur, spread, or paint.
    pub fn try_new(shadows: Vec<Shadow>) -> Result<Self> {
        if shadows.is_empty() {
            return Err(Error::invalid_value(
                "shadow list",
                "[]",
                "must contain at least one shadow",
            ));
        }
        for shadow in &shadows {
            validate_point(shadow.offset(), "shadow offset")?;
            validate_non_negative_f64(shadow.blur(), "shadow blur")?;
            validate_finite_f64(shadow.spread(), "shadow spread")?;
            validate_paint(shadow.paint())?;
        }
        Ok(Self { shadows })
    }

    #[must_use]
    /// Returns the shadows in rendering order.
    pub fn shadows(&self) -> &[Shadow] {
        &self.shadows
    }

    #[must_use]
    /// Returns the number of shadows.
    pub fn len(&self) -> usize {
        self.shadows.len()
    }

    #[must_use]
    /// Returns whether the list contains no shadows.
    ///
    /// Values produced by [`Self::try_new`] are never empty.
    pub fn is_empty(&self) -> bool {
        self.shadows.is_empty()
    }

    pub(crate) fn into_vec(self) -> Vec<Shadow> {
        self.shadows
    }
}
