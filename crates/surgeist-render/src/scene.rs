use super::{
    command::{RenderCommands, normalize_commands},
    *,
};
use std::{mem, ops::Deref};
mod inspect;
mod value;
pub use inspect::SceneSummary;

/// An authored, ordered tree of renderer-facing draw commands.
///
/// Scene builders append commands in call order. Nested layers own child command
/// sequences, and the default scene is empty. Validation and capability
/// normalization occur when a renderer consumes the scene.
/// Clearing or dropping a scene releases its commands with a bounded call
/// stack. Nested disposal may use temporary storage for layers with pending
/// siblings.
/// Cloning, comparing, and formatting nested commands may use temporary
/// storage proportional to the nesting depth.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub(crate) commands: Vec<Command>,
}

impl Scene {
    #[must_use]
    /// Creates an empty authored scene.
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes every top-level command and nested command tree from this scene.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Appends a filled shape and returns this scene for chaining.
    pub fn fill(&mut self, shape: impl Into<Shape>, paint: impl Into<Paint>) -> &mut Self {
        self.commands.push(Command::Fill {
            shape: shape.into(),
            paint: paint.into(),
        });
        self
    }

    /// Appends a stroked shape and returns this scene for chaining.
    pub fn stroke(
        &mut self,
        shape: impl Into<Shape>,
        stroke: Stroke,
        paint: impl Into<Paint>,
    ) -> &mut Self {
        self.commands.push(Command::Stroke {
            shape: shape.into(),
            stroke,
            paint: paint.into(),
        });
        self
    }

    /// Appends one shadow draw for `shape` and returns this scene for chaining.
    pub fn shadow(&mut self, shape: impl Into<Shape>, shadow: Shadow) -> &mut Self {
        self.commands.push(Command::Shadow {
            shape: shape.into(),
            shadow,
        });
        self
    }

    /// Appends one shadow draw per ordered list entry and returns this scene for chaining.
    ///
    /// The supplied shape is cloned for each shadow.
    pub fn shadows(&mut self, shape: impl Into<Shape>, shadows: ShadowList) -> &mut Self {
        let shape = shape.into();
        for shadow in shadows.into_vec() {
            self.commands.push(Command::Shadow {
                shape: shape.clone(),
                shadow,
            });
        }
        self
    }

    /// Appends an image draw into a logical destination rectangle using `fit`.
    pub fn image(&mut self, image: Image, rect: Rect, fit: ImageFit) -> &mut Self {
        self.commands.push(Command::Image { image, rect, fit });
        self
    }

    /// Maps an image source rectangle in pixel coordinates to a destination.
    ///
    /// The source must be nonempty and within the original image. Its fractional
    /// bounds control the mapping. Sampling clamps to the enclosing texel cells.
    /// The renderer validates the source when it normalizes the scene.
    pub fn image_region(&mut self, image: Image, source: Rect, destination: Rect) -> &mut Self {
        self.commands.push(Command::ImageRegion {
            image,
            source,
            rect: destination,
        });
        self
    }

    /// Copies an authored text run into the scene's owned command storage.
    ///
    /// Borrowed font names and glyph slices no longer need to outlive the call.
    pub fn text_run(&mut self, run: TextRun<'_>) -> &mut Self {
        self.commands.push(Command::TextRun {
            font: run.font().to_owned_static(),
            size: run.size(),
            transform: run.transform(),
            paint: run.paint().clone(),
            glyphs: run.glyphs().to_vec(),
            bounds: run.bounds(),
        });
        self
    }

    /// Appends one shared, already admitted glyph run with run-local paint.
    /// Geometry and font resources remain owned independently of the caller.
    /// Numeric backend composition is checked before frame publication.
    pub fn prepared_glyph_run(
        &mut self,
        run: PreparedGlyphRun,
        transform: Transform,
        paint: TextPaint,
    ) -> &mut Self {
        self.commands.push(Command::PreparedGlyphRun {
            run,
            transform,
            paint,
        });
        self
    }

    /// Copies an authored text run and its ordered shadows into owned command storage.
    pub fn text_shadow_run(&mut self, run: TextShadowRun<'_>) -> &mut Self {
        let text = run.run();
        self.commands.push(Command::TextShadowRun {
            font: text.font().to_owned_static(),
            size: text.size(),
            transform: text.transform(),
            paint: text.paint().clone(),
            glyphs: text.glyphs().to_vec(),
            bounds: text.bounds(),
            shadows: run.shadows().clone(),
        });
        self
    }

    /// Lowers a validated solid text-decoration line to a stroked path command.
    ///
    /// A non-identity decoration transform is represented by a nested transform layer.
    pub fn text_decoration_line(&mut self, line: TextDecorationLine) -> &mut Self {
        let mut path = Path::new();
        path.move_to(line.start()).line_to(line.end());
        let stroke = line.stroke();
        if line.transform() == Transform::identity() {
            return self.stroke(Shape::path(path), stroke, line.paint().clone());
        }

        let paint = line.paint().clone();
        self.transform(line.transform(), move |scene| {
            scene.stroke(Shape::path(path), stroke, paint);
        });
        self
    }

    /// Builds a child scene and appends it as one layer command.
    ///
    /// The callback runs synchronously, and its commands retain their authored order.
    pub fn layer(&mut self, layer: Layer, children: impl FnOnce(&mut Scene)) -> &mut Self {
        let mut child = Scene::new();
        children(&mut child);
        self.commands.push(Command::Layer {
            layer,
            children: child.commands.into(),
        });
        self
    }

    /// Applies an ordered stack of resolved masks to one isolated group of children.
    ///
    /// Supply masks from top to bottom. Coverage combines from bottom to top,
    /// starting with the bottom mask's coverage. The callback builds a fresh
    /// child scene. An empty stack adds its commands directly to this scene.
    /// A transparent entry still participates in the fold.
    pub fn masked_group(
        &mut self,
        masks: &[ResolvedMaskLayer],
        children: impl FnOnce(&mut Scene),
    ) -> &mut Self {
        let Some((bottom, above)) = masks.split_last() else {
            let mut child = Scene::new();
            children(&mut child);
            self.commands.append(&mut child.commands);
            return self;
        };
        self.layer(Layer::new().isolated(), |target| {
            children(target);
            let apply = Layer::new().with_composition(LayerComposition::normal_porter_duff(
                PorterDuffOperator::DestinationIn,
            ));
            target.layer(apply, |coverage| {
                coverage.append_mask_coverage(&bottom.source, PorterDuffOperator::SourceOver);
                for layer in above.iter().rev() {
                    let operator = match layer.composite {
                        MaskCompositeMode::Add => PorterDuffOperator::SourceOver,
                        MaskCompositeMode::Subtract => PorterDuffOperator::SourceOut,
                        MaskCompositeMode::Intersect => PorterDuffOperator::SourceIn,
                        MaskCompositeMode::Exclude => PorterDuffOperator::Xor,
                    };
                    coverage.append_mask_coverage(&layer.source, operator);
                }
            });
        })
    }

    fn append_mask_coverage(&mut self, source: &ResolvedMaskSource, operator: PorterDuffOperator) {
        self.layer(Layer::mask_coverage(source, operator), |_| {});
    }

    /// Builds children under a logical transform layer.
    ///
    /// The supplied [`Transform`] already guarantees finite coefficients.
    pub fn transform(
        &mut self,
        transform: Transform,
        children: impl FnOnce(&mut Scene),
    ) -> &mut Self {
        let layer = Layer::new().with_transform(transform);
        self.layer(layer, children)
    }

    /// Builds children under an authored shape clip layer.
    ///
    /// The supplied [`Shape`] retains its validated construction invariants.
    pub fn clip(
        &mut self,
        shape: impl Into<Shape>,
        children: impl FnOnce(&mut Scene),
    ) -> &mut Self {
        let layer = Layer::new().with_shape_clip(shape.into());
        self.layer(layer, children)
    }

    #[must_use]
    /// Returns the number of top-level commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    #[must_use]
    /// Returns whether the scene contains no top-level commands.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Inspects nested authored structure under the supplied accounting context.
    ///
    /// Image observations compare exact pixel content. Counts describe authored
    /// commands rather than normalized renderer emissions.
    pub fn inspect(&self, context: &mut RenderAccountingContext) -> Result<SceneSummary> {
        context.run(
            RenderAttemptKind::SceneInspection,
            RenderPhase::SceneInspect,
            |account| inspect::inspect_commands(&self.commands, account),
        )
    }

    pub(crate) fn normalize(
        &self,
        capabilities: Capabilities,
        account: &crate::accounting::Account,
    ) -> Result<RenderCommands> {
        normalize_commands(&self.commands, capabilities, account)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Command {
    PreparedGlyphRun {
        run: PreparedGlyphRun,
        transform: Transform,
        paint: TextPaint,
    },
    Fill {
        shape: Shape,
        paint: Paint,
    },
    Stroke {
        shape: Shape,
        stroke: Stroke,
        paint: Paint,
    },
    Shadow {
        shape: Shape,
        shadow: Shadow,
    },
    Image {
        image: Image,
        rect: Rect,
        fit: ImageFit,
    },
    ImageRegion {
        image: Image,
        source: Rect,
        rect: Rect,
    },
    TextRun {
        font: FontRef<'static>,
        size: f32,
        transform: Transform,
        paint: TextPaint,
        glyphs: Vec<TextGlyph>,
        bounds: TextRunBounds,
    },
    TextShadowRun {
        font: FontRef<'static>,
        size: f32,
        transform: Transform,
        paint: TextPaint,
        glyphs: Vec<TextGlyph>,
        bounds: TextRunBounds,
        shadows: ShadowList,
    },
    Layer {
        layer: Layer,
        children: LayerCommands,
    },
}

/// Owns a layer's authored children and disposes of their tree iteratively.
pub(crate) struct LayerCommands(Vec<Command>);

impl From<Vec<Command>> for LayerCommands {
    fn from(commands: Vec<Command>) -> Self {
        Self(commands)
    }
}

impl Deref for LayerCommands {
    type Target = Vec<Command>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for LayerCommands {
    fn drop(&mut self) {
        if self.0.is_empty() {
            return;
        }
        let mut commands = mem::take(&mut self.0).into_iter();
        // The authored scene owns this temporary backing. It holds one
        // iterator per ancestor with pending siblings and stays empty
        // for a single-child chain.
        let mut pending = Vec::new();
        loop {
            let Some(command) = commands.next() else {
                let Some(siblings) = pending.pop() else {
                    return;
                };
                commands = siblings;
                continue;
            };
            if let Command::Layer {
                layer,
                mut children,
            } = command
            {
                drop(layer);
                if !children.is_empty() {
                    if !commands.as_slice().is_empty() {
                        pending.push(commands);
                    }
                    // Empty the nested owner before it drops. Its allocation
                    // and the remaining commands now belong to the iterator.
                    commands = mem::take(&mut children.0).into_iter();
                }
            }
        }
    }
}
