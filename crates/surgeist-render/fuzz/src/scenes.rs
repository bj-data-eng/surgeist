//! Bounded authored inspection recipes. These observations do not validate or
//! normalize a whole scene and make no assertion about rendered pixels.
use crate::{
    codec::{Reader, Writer},
    descriptor, error, evaluated, product_observation, rejected, violated,
};
use surgeist_fuzz::{
    CaseOutcome, CaseTarget, FuzzError, FuzzErrorKind, Generation, GenerationRejection, Result,
    TargetDescriptor,
    generation::{Budget, BudgetCharge, Entropy},
};
use surgeist_render::{self as render, Scene, SceneSummary};

mod geometry;
#[cfg(test)]
mod tests;

const MAGIC: [u8; 4] = *b"SRFS";
const STRUCTURE: &str = "surgeist-render::scene-structure";
const IDENTITY: &str = "surgeist-render::image-content-identity";
const RETRY: &str = "surgeist-render::inspection-refusal-retry";
const MAX_IMAGES: usize = 4;
const MAX_RGBA: usize = 4096;
const MAX_COMMANDS: usize = 64;
const MAX_DEPTH: usize = 8;
const ROOT: u8 = 255;

enum GenerationFailure {
    EntropyExhausted,
    Harness(FuzzError),
}
impl From<FuzzError> for GenerationFailure {
    fn from(cause: FuzzError) -> Self {
        Self::Harness(cause)
    }
}
type GenerationResult<T> = std::result::Result<T, GenerationFailure>;

/// Admit expected entropy exhaustion before calling fallible framework helpers.
/// Every error from an admitted helper remains a harness/resource failure.
struct GenerationEntropy<'a>(Entropy<'a>);
impl<'a> GenerationEntropy<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self(Entropy::new(input))
    }
    fn require(&self, count: usize) -> GenerationResult<()> {
        if self.0.remaining() < count {
            return Err(GenerationFailure::EntropyExhausted);
        }
        Ok(())
    }
    fn range(&mut self, minimum: u64, maximum: u64) -> GenerationResult<u64> {
        if minimum < maximum {
            self.require(8)?;
        }
        self.0.range(minimum, maximum).map_err(Into::into)
    }
    fn bytes(&mut self, count: usize, budget: &mut Budget) -> GenerationResult<Vec<u8>> {
        self.require(count)?;
        self.0.bytes(count, budget).map_err(Into::into)
    }
}

/// A checked, bounded concrete recipe, rather than generation entropy.
#[derive(Clone, Debug)]
pub struct SceneCase {
    payload: Payload,
}

#[derive(Clone, Debug)]
enum Payload {
    Inspect(Inspection),
    Coefficients([f64; 6]),
    Compose {
        first: [f64; 6],
        next: [f64; 6],
        origin: [f64; 2],
    },
    CompositionOverflow,
    Clip(geometry::ClipRecipe),
    Mask {
        image: ImageRecipe,
        bounds: [f64; 4],
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ImageRecipe {
    width: u8,
    height: u8,
    rgba: Vec<u8>,
}
impl ImageRecipe {
    fn validate(&self) -> Result<()> {
        if self.width == 0
            || self.height == 0
            || self.width > 16
            || self.height > 16
            || self.rgba.len() != usize::from(self.width) * usize::from(self.height) * 4
        {
            return Err(error("image recipe dimensions or RGBA length"));
        }
        Ok(())
    }
    fn materialize(&self) -> render::Result<render::Image> {
        render::Image::from_rgba(
            render::Size::try_new(f64::from(self.width), f64::from(self.height))?,
            self.rgba.clone(),
        )
    }
    fn read(reader: &mut Reader<'_>, budget: &mut Budget, max_rgba: usize) -> Result<Self> {
        // Reserve the hard payload ceiling before Reader copies any RGBA bytes.
        credit(budget, MAX_RGBA + std::mem::size_of::<Self>(), 0, 0)?;
        let width = reader.u8()?;
        let height = reader.u8()?;
        if width == 0 || height == 0 || width > 16 || height > 16 {
            return Err(error("image dimension ceiling"));
        }
        let expected = usize::from(width) * usize::from(height) * 4;
        if expected > max_rgba {
            return Err(error("aggregate RGBA ceiling"));
        }
        let image = Self {
            width,
            height,
            rgba: reader.bytes(expected)?,
        };
        image.validate()?;
        Ok(image)
    }
    fn write(&self, writer: &mut Writer) -> Result<()> {
        writer.u8(self.width)?;
        writer.u8(self.height)?;
        writer.bytes(&self.rgba, MAX_RGBA)
    }
}

#[derive(Clone, Debug)]
struct Inspection {
    images: Vec<ImageRecipe>,
    commands: Vec<Node>,
    refusal: Refusal,
}
#[derive(Clone, Copy, Debug)]
struct Node {
    parent: u8,
    draw: Draw,
}
#[derive(Clone, Copy, Debug)]
enum Draw {
    Layer,
    Fill(u8),
    Stroke(u8),
    Shadow(u8),
    Image(u8),
    Glyphs(u8),
}
#[derive(Clone, Copy, Debug)]
enum Refusal {
    None,
    Commands { limit: u64, retry_limit: u64 },
    ResourceProbe { retry_limit: u64 },
    IdentityByte { retry_limit: u64 },
}

/// Public Render front doors shared by canonical replay and generation.
pub struct SceneTarget {
    descriptor: TargetDescriptor,
}
impl Default for SceneTarget {
    fn default() -> Self {
        Self::new()
    }
}
impl SceneTarget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptor: descriptor("surgeist-render::scene-inspection"),
        }
    }

    /// Independently authored canonical seed recipes; unknown selectors reject.
    pub fn literal(&self, which: u8) -> Result<SceneCase> {
        let pixel = || ImageRecipe {
            width: 1,
            height: 1,
            rgba: vec![10, 20, 30, 255],
        };
        let two = || Inspection {
            images: vec![],
            commands: vec![
                Node {
                    parent: ROOT,
                    draw: Draw::Fill(ROOT),
                },
                Node {
                    parent: ROOT,
                    draw: Draw::Fill(ROOT),
                },
            ],
            refusal: Refusal::None,
        };
        let identity = [1., 0., 0., 1., 0., 0.];
        let payload = match which {
            0 => Payload::Inspect(Inspection {
                images: vec![
                    pixel(),
                    pixel(),
                    ImageRecipe {
                        width: 1,
                        height: 2,
                        rgba: vec![1; 8],
                    },
                    ImageRecipe {
                        width: 2,
                        height: 1,
                        rgba: vec![1; 8],
                    },
                ],
                commands: vec![
                    Node {
                        parent: ROOT,
                        draw: Draw::Fill(ROOT),
                    },
                    Node {
                        parent: ROOT,
                        draw: Draw::Layer,
                    },
                    Node {
                        parent: 1,
                        draw: Draw::Image(0),
                    },
                    Node {
                        parent: 1,
                        draw: Draw::Fill(1),
                    },
                    Node {
                        parent: 1,
                        draw: Draw::Layer,
                    },
                    Node {
                        parent: 4,
                        draw: Draw::Stroke(0),
                    },
                    Node {
                        parent: 4,
                        draw: Draw::Shadow(1),
                    },
                    Node {
                        parent: ROOT,
                        draw: Draw::Image(2),
                    },
                    Node {
                        parent: ROOT,
                        draw: Draw::Image(3),
                    },
                    Node {
                        parent: ROOT,
                        draw: Draw::Glyphs(3),
                    },
                ],
                refusal: Refusal::None,
            }),
            1 => Payload::Inspect(two()),
            2 => {
                let mut case = two();
                case.refusal = Refusal::Commands {
                    limit: 1,
                    retry_limit: 64,
                };
                Payload::Inspect(case)
            }
            3 => Payload::Coefficients([1., -0., 2., 1., -3., 4.]),
            4 => Payload::Compose {
                first: [2., 1., -1., 3., 5., -7.],
                next: [1., -2., 3., 1., -4., 8.],
                origin: [3., -5.],
            },
            5 => Payload::CompositionOverflow,
            6 => Payload::Clip(geometry::ClipRecipe {
                geometry: geometry::ClipSource::Rect([1., 2., 3., 4.]),
                transform: [2., 0., 0., -1., 10., 20.],
            }),
            7 => Payload::Clip(geometry::ClipRecipe {
                geometry: geometry::ClipSource::Path(vec![[1., 2.], [4., 2.], [4., 6.]]),
                transform: identity,
            }),
            8 => Payload::Clip(geometry::ClipRecipe {
                geometry: geometry::ClipSource::Rect([1., 2., 3., 4.]),
                transform: [f64::MAX, 0., 0., 1., 0., 0.],
            }),
            9 => Payload::Mask {
                image: pixel(),
                bounds: [-2., 3., 5., 7.],
            },
            10 => Payload::Mask {
                image: pixel(),
                bounds: [0., 0., 0., 1.],
            },
            11 => Payload::Coefficients([
                1.,
                0.,
                f64::from_bits(0x7ff8_0000_0000_1234),
                1.,
                f64::INFINITY,
                -0.,
            ]),
            _ => return Err(error("unknown scene literal")),
        };
        let case = SceneCase { payload };
        self.validate(&case)?;
        Ok(case)
    }

    fn budget(&self) -> Budget {
        Budget::new(self.descriptor.limits())
    }
    fn validate(&self, case: &SceneCase) -> Result<()> {
        let mut budget = self.budget();
        credit(&mut budget, 1024, 0, 0)?;
        match &case.payload {
            Payload::Inspect(recipe) => recipe.validate(&mut budget),
            Payload::Compose {
                first,
                next,
                origin,
            } => {
                if first
                    .iter()
                    .chain(next)
                    .chain(origin)
                    .any(|x| !x.is_finite() || x.fract() != 0. || x.abs() > 1024.)
                {
                    return Err(error(
                        "composition oracle requires bounded integer lattice coefficients",
                    ));
                }
                Ok(())
            }
            Payload::Clip(recipe) => recipe.validate(&mut budget),
            Payload::Mask { image, .. } => {
                image.validate()?;
                credit(&mut budget, image.rgba.len() * 3, 1, 1)
            }
            Payload::Coefficients(_) | Payload::CompositionOverflow => Ok(()),
        }
    }
}

fn credit(budget: &mut Budget, bytes: usize, nodes: usize, depth: usize) -> Result<()> {
    budget.credit(BudgetCharge {
        bytes: bytes as u64,
        nodes: nodes as u64,
        depth: depth as u64,
        operations: nodes as u64,
        ..BudgetCharge::default()
    })
}
fn write_floats<const N: usize>(writer: &mut Writer, values: &[f64; N]) -> Result<()> {
    for value in values {
        writer.f64(*value)?;
    }
    Ok(())
}
fn read_floats<const N: usize>(reader: &mut Reader<'_>) -> Result<[f64; N]> {
    let mut values = [0.; N];
    for value in &mut values {
        *value = reader.f64()?;
    }
    Ok(values)
}

impl CaseTarget for SceneTarget {
    type Case = SceneCase;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<SceneCase> {
        let mut reader = Reader::new(input, MAGIC)?;
        let mut budget = self.budget();
        credit(&mut budget, input.len(), 0, 0)?;
        let payload = match reader.u8()? {
            0 => Payload::Inspect(Inspection::read(&mut reader, &mut budget)?),
            1 => Payload::Coefficients(read_floats(&mut reader)?),
            2 => Payload::Compose {
                first: read_floats(&mut reader)?,
                next: read_floats(&mut reader)?,
                origin: read_floats(&mut reader)?,
            },
            3 => Payload::CompositionOverflow,
            4 => Payload::Clip(geometry::ClipRecipe::read(&mut reader, &mut budget)?),
            5 => Payload::Mask {
                image: ImageRecipe::read(&mut reader, &mut budget, MAX_RGBA)?,
                bounds: read_floats(&mut reader)?,
            },
            _ => return Err(error("unknown scene payload tag")),
        };
        reader.finish()?;
        let case = SceneCase { payload };
        self.validate(&case)?;
        Ok(case)
    }
    fn encode(&self, case: &SceneCase) -> Result<Vec<u8>> {
        self.validate(case)?;
        let mut writer = Writer::new(MAGIC)?;
        match &case.payload {
            Payload::Inspect(recipe) => {
                writer.u8(0)?;
                recipe.write(&mut writer)?;
            }
            Payload::Coefficients(values) => {
                writer.u8(1)?;
                write_floats(&mut writer, values)?;
            }
            Payload::Compose {
                first,
                next,
                origin,
            } => {
                writer.u8(2)?;
                write_floats(&mut writer, first)?;
                write_floats(&mut writer, next)?;
                write_floats(&mut writer, origin)?;
            }
            Payload::CompositionOverflow => writer.u8(3)?,
            Payload::Clip(recipe) => {
                writer.u8(4)?;
                recipe.write(&mut writer)?;
            }
            Payload::Mask { image, bounds } => {
                writer.u8(5)?;
                image.write(&mut writer)?;
                write_floats(&mut writer, bounds)?;
            }
        }
        Ok(writer.finish())
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<SceneCase>> {
        if input.len() as u64 > self.descriptor.limits().bytes() {
            return Err(error("scene entropy byte ceiling"));
        }
        let generated: GenerationResult<SceneCase> = (|| {
            let mut entropy = GenerationEntropy::new(input);
            let selector = entropy.range(0, 15)? as u8;
            if selector < 12 {
                return self.literal(selector).map_err(GenerationFailure::Harness);
            }
            let mut budget = self.budget();
            credit(&mut budget, 1024, 0, 0)?;
            if selector == 12 {
                let mut values = [0.; 6];
                for value in &mut values {
                    *value = f64::from_bits(entropy.range(0, u64::MAX)?);
                }
                return Ok(SceneCase {
                    payload: Payload::Coefficients(values),
                });
            }
            if selector == 14 {
                let mut first = [0.; 6];
                let mut next = [0.; 6];
                let mut origin = [0.; 2];
                for value in first.iter_mut().chain(&mut next).chain(&mut origin) {
                    *value = entropy.range(0, 32)? as f64 - 16.;
                }
                return Ok(SceneCase {
                    payload: Payload::Compose {
                        first,
                        next,
                        origin,
                    },
                });
            }
            if selector == 15 {
                let mut bounds = [0.; 4];
                for value in &mut bounds {
                    *value = entropy.range(0, 32)? as f64 - 16.;
                }
                return Ok(SceneCase {
                    payload: Payload::Clip(geometry::ClipRecipe {
                        geometry: geometry::ClipSource::Rect(bounds),
                        transform: [1., 0., 0., 1., 3., -2.],
                    }),
                });
            }
            let image_count = entropy.range(0, MAX_IMAGES as u64)? as usize;
            credit(
                &mut budget,
                image_count * std::mem::size_of::<ImageRecipe>(),
                image_count,
                usize::from(image_count != 0),
            )?;
            let mut images = Vec::new();
            images.try_reserve_exact(image_count).map_err(|cause| {
                FuzzError::with_source(
                    FuzzErrorKind::Resource,
                    "materialize scene generation",
                    "image recipe allocation failed",
                    cause,
                )
            })?;
            for _ in 0..image_count {
                let width = entropy.range(1, 16)? as u8;
                let height = entropy.range(1, 16)? as u8;
                let count = usize::from(width) * usize::from(height) * 4;
                credit(&mut budget, count * 2, 0, 0)?;
                let rgba = entropy.bytes(count, &mut budget)?;
                images.push(ImageRecipe {
                    width,
                    height,
                    rgba,
                });
            }
            let count = entropy.range(0, MAX_COMMANDS as u64)? as usize;
            // Charge case storage and conservative public Scene/text copies before allocation.
            credit(&mut budget, count * 1024, count, usize::from(count != 0))?;
            let mut commands = Vec::new();
            commands.try_reserve_exact(count).map_err(|cause| {
                FuzzError::with_source(
                    FuzzErrorKind::Resource,
                    "materialize scene generation",
                    "command recipe allocation failed",
                    cause,
                )
            })?;
            let mut depths = [0; MAX_COMMANDS];
            for index in 0..count {
                let mut eligible = [ROOT; MAX_COMMANDS + 1];
                let mut available = 1;
                for (index, node) in commands.iter().enumerate() {
                    if matches!(
                        node,
                        Node {
                            draw: Draw::Layer,
                            ..
                        }
                    ) && depths[index] < MAX_DEPTH
                    {
                        eligible[available] = index as u8;
                        available += 1;
                    }
                }
                let parent = eligible[entropy.range(0, (available - 1) as u64)? as usize];
                let image = if images.is_empty() {
                    ROOT
                } else {
                    entropy.range(0, image_count as u64)? as u8
                };
                // The extra paint choice explicitly denotes a solid color.
                let paint = if image as usize == image_count {
                    ROOT
                } else {
                    image
                };
                let draw = match entropy.range(0, if images.is_empty() { 4 } else { 5 })? {
                    0 => Draw::Fill(paint),
                    1 => Draw::Stroke(paint),
                    2 => Draw::Shadow(paint),
                    3 => Draw::Glyphs(entropy.range(0, 8)? as u8),
                    4 => Draw::Layer,
                    _ => Draw::Image(entropy.range(0, (image_count - 1) as u64)? as u8),
                };
                depths[index] = if parent == ROOT {
                    1
                } else {
                    depths[usize::from(parent)] + 1
                };
                commands.push(Node { parent, draw });
            }
            let recipe = Inspection {
                images,
                commands,
                refusal: Refusal::None,
            };
            let case = SceneCase {
                payload: Payload::Inspect(recipe),
            };
            self.validate(&case)?;
            Ok(case)
        })();
        match generated {
            Ok(case) => generation_outcome(Ok(case)),
            Err(GenerationFailure::EntropyExhausted) => Ok(Generation::Rejected(
                GenerationRejection::new("insufficient entropy for bounded scene generation")?,
            )),
            Err(GenerationFailure::Harness(cause)) => generation_outcome(Err(cause)),
        }
    }
    fn evaluate(&self, case: &SceneCase) -> Result<CaseOutcome> {
        self.validate(case)?;
        match &case.payload {
            Payload::Inspect(recipe) => recipe.evaluate(),
            Payload::Coefficients(values) => geometry::coefficients(*values),
            Payload::Compose {
                first,
                next,
                origin,
            } => geometry::compose(*first, *next, *origin),
            Payload::CompositionOverflow => geometry::overflow(),
            Payload::Clip(recipe) => recipe.evaluate(),
            Payload::Mask { image, bounds } => geometry::mask(image, *bounds),
        }
    }
}

fn generation_outcome(generated: Result<SceneCase>) -> Result<Generation<SceneCase>> {
    generated.map(Generation::Generated)
}

impl Inspection {
    fn validate(&self, budget: &mut Budget) -> Result<()> {
        if self.images.len() > MAX_IMAGES || self.commands.len() > MAX_COMMANDS {
            return Err(error("scene collection ceiling"));
        }
        let mut bytes = 0;
        for image in &self.images {
            image.validate()?;
            bytes += image.rgba.len();
        }
        if bytes > MAX_RGBA {
            return Err(error("aggregate image byte ceiling"));
        }
        credit(
            budget,
            bytes * 3 + self.images.len() * std::mem::size_of::<ImageRecipe>(),
            self.images.len(),
            usize::from(!self.images.is_empty()),
        )?;
        let mut depths = [0; MAX_COMMANDS];
        for (index, node) in self.commands.iter().enumerate() {
            let depth = if node.parent == ROOT {
                1
            } else {
                let parent = usize::from(node.parent);
                if parent >= index || !matches!(self.commands[parent].draw, Draw::Layer) {
                    return Err(error("parent must be an earlier layer"));
                }
                depths[parent] + 1
            };
            if depth > MAX_DEPTH {
                return Err(error("scene depth ceiling"));
            }
            depths[index] = depth;
            credit(budget, 1024, 1, depth)?;
            match node.draw {
                Draw::Fill(image) | Draw::Stroke(image) | Draw::Shadow(image) if image != ROOT => {
                    if usize::from(image) >= self.images.len() {
                        return Err(error("paint image reference"));
                    }
                }
                Draw::Image(image) if usize::from(image) >= self.images.len() => {
                    return Err(error("draw image reference"));
                }
                Draw::Glyphs(count) if count > 8 => return Err(error("glyph recipe ceiling")),
                _ => {}
            }
        }
        match self.refusal {
            Refusal::Commands { limit, retry_limit }
                if limit >= self.commands.len() as u64
                    || retry_limit < self.commands.len() as u64 =>
            {
                return Err(error("command refusal must reach its cap"));
            }
            Refusal::ResourceProbe { retry_limit } | Refusal::IdentityByte { retry_limit } => {
                let mut occurrence = None;
                let mut count = 0;
                for node in &self.commands {
                    if let Some(image) = self.image_for(node.draw) {
                        if occurrence.is_some_and(|previous| previous != image) {
                            return Err(error(
                                "identity refusal requires equal-content occurrences",
                            ));
                        }
                        occurrence = Some(image);
                        count += 1;
                    }
                }
                if count < 2 {
                    return Err(error("identity refusal requires repeated image"));
                }
                let required = match self.refusal {
                    Refusal::ResourceProbe { .. } => count - 1,
                    _ => (count - 1) * occurrence.expect("two image occurrences").rgba.len() as u64,
                };
                if retry_limit < required {
                    return Err(error(
                        "raised retry limit must admit complete identity work",
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn image_for(&self, draw: Draw) -> Option<&ImageRecipe> {
        match draw {
            Draw::Fill(index) | Draw::Stroke(index) | Draw::Shadow(index) | Draw::Image(index)
                if index != ROOT =>
            {
                self.images.get(usize::from(index))
            }
            _ => None,
        }
    }
    fn expected(&self) -> SceneSummary {
        let mut summary = SceneSummary::default();
        for node in &self.commands {
            summary.commands += 1;
            match node.draw {
                Draw::Layer => summary.layers += 1,
                Draw::Fill(_) => summary.fills += 1,
                Draw::Stroke(_) => summary.strokes += 1,
                Draw::Shadow(_) => summary.shadows += 1,
                Draw::Glyphs(count) => summary.glyphs += u64::from(count),
                Draw::Image(_) => {}
            }
        }
        // Set membership uses full dimensions and bytes, independently of Render's
        // compact content fingerprint, resource IDs, and pointer identity.
        let mut observed: Vec<&ImageRecipe> = Vec::new();
        for node in &self.commands {
            if let Some(image) = self.image_for(node.draw) {
                summary.images += 1;
                if observed.contains(&image) {
                    summary.image_content_hits += 1;
                } else {
                    summary.image_content_misses += 1;
                    summary.first_observed_image_bytes += image.rgba.len() as u64;
                    observed.push(image);
                }
            }
        }
        summary
    }
    fn build(&self) -> render::Result<Scene> {
        let images = self
            .images
            .iter()
            .map(ImageRecipe::materialize)
            .collect::<render::Result<Vec<_>>>()?;
        let mut scene = Scene::new();
        self.append(ROOT, &images, &mut scene)?;
        Ok(scene)
    }
    fn append(
        &self,
        parent: u8,
        images: &[render::Image],
        scene: &mut Scene,
    ) -> render::Result<()> {
        for (index, node) in self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, node)| node.parent == parent)
        {
            let rect = render::Rect::try_new(0., 0., 4., 4.)?;
            let paint = |image| -> render::Paint {
                if image == ROOT {
                    render::Color::BLACK.into()
                } else {
                    images[usize::from(image)].clone().into()
                }
            };
            match node.draw {
                Draw::Layer => {
                    let mut result = Ok(());
                    scene.layer(render::Layer::new(), |child| {
                        result = self.append(index as u8, images, child)
                    });
                    result?;
                }
                Draw::Fill(image) => {
                    scene.fill(rect, paint(image));
                }
                Draw::Stroke(image) => {
                    scene.stroke(rect, render::Stroke::try_new(1.)?, paint(image));
                }
                Draw::Shadow(image) => {
                    scene.shadow(
                        rect,
                        render::Shadow::try_new(render::Point::default(), 0., 0., paint(image))?,
                    );
                }
                Draw::Image(image) => {
                    scene.image(
                        images[usize::from(image)].clone(),
                        rect,
                        render::ImageFit::Fill,
                    );
                }
                Draw::Glyphs(count) => {
                    let glyphs = (0..count)
                        .map(|id| render::TextGlyph::try_new(u32::from(id), f64::from(id), 0., 1.))
                        .collect::<render::Result<Vec<_>>>()?;
                    scene.text_run(render::TextRun::try_new(
                        render::FontRef::new(1),
                        12.,
                        render::Transform::identity(),
                        render::TextPaint::try_fill(render::Color::BLACK.into())?,
                        &glyphs,
                        render::TextRunBounds::unspecified(),
                    )?);
                }
            }
        }
        Ok(())
    }
    fn evaluate(&self) -> Result<CaseOutcome> {
        let expected = self.expected();
        let scene = match product_observation(
            self.build(),
            STRUCTURE,
            "authored-scene-construction-rejected",
        )? {
            Ok(scene) => scene,
            Err(failure) => return Ok(failure),
        };
        let original = scene.clone();
        let mut context = render::RenderAccountingContext::default();
        if let Some(failure) = compare_inspection(
            expected,
            scene.inspect(&mut context),
            STRUCTURE,
            "authored-inspection-rejected",
        )? {
            return Ok(failure);
        }
        if !matches!(self.refusal, Refusal::None) {
            let (work, limit, retry_limit) = match self.refusal {
                Refusal::Commands { limit, retry_limit } => {
                    (render::RenderWork::AuthoredCommands, limit, retry_limit)
                }
                Refusal::ResourceProbe { retry_limit } => {
                    (render::RenderWork::ResourceProbes, 0, retry_limit)
                }
                Refusal::IdentityByte { retry_limit } => (
                    render::RenderWork::ImageIdentityByteComparisons,
                    1,
                    retry_limit,
                ),
                Refusal::None => unreachable!(),
            };
            let limits = match product_observation(
                render::RenderLimits::new().with_work_limit(work, limit),
                RETRY,
                "inspection-work-limit-rejected",
            )? {
                Ok(limits) => limits,
                Err(failure) => return Ok(failure),
            };
            context.set_options(render::RenderAccountingOptions::new().with_limits(limits));
            let outcome = scene.inspect(&mut context);
            let Err(cause) = outcome else {
                return violated(
                    RETRY,
                    "inspection-refusal-not-enforced",
                    "inspection accepted a request beyond its work cap",
                );
            };
            let Some(render::RenderAccountingError::Refused(refusal)) =
                cause.accounting_diagnostic()
            else {
                return violated(RETRY, "inspection-refusal-kind", format!("{cause:?}"));
            };
            let Some(report) = context.last_attempt() else {
                return violated(
                    RETRY,
                    "inspection-refusal-report-missing",
                    "public inspection refusal published no accepted-prefix report",
                );
            };
            let prefix = report.usage();
            let exact_prefix = match self.refusal {
                Refusal::Commands { .. } => {
                    prefix.work(render::RenderWork::CommandVisits) == limit + 1
                }
                Refusal::ResourceProbe { .. } | Refusal::IdentityByte { .. } => {
                    let first_image = self
                        .commands
                        .iter()
                        .find_map(|node| self.image_for(node.draw))
                        .expect("validated repeated images");
                    prefix.work(render::RenderWork::AuthoredCommands) >= 2
                        && prefix.work(render::RenderWork::AuthoredImages) == 2
                        && prefix.work(render::RenderWork::ImageContentMisses) == 1
                        && prefix.work(render::RenderWork::ImageContentHits) == 0
                        && prefix.work(render::RenderWork::ImageFirstObservedBytes)
                            == first_image.rgba.len() as u64
                }
                Refusal::None => unreachable!(),
            };
            if scene != original
                || report.status() != render::RenderAttemptStatus::Refused
                || !exact_prefix
                || prefix.work(work) != limit
                || prefix.live(render::RenderResource::ScratchCpuBytes) != 0
                || prefix.live(render::RenderResource::RetainedCpuBytes) != 0
                || prefix.work(render::RenderWork::AuthoredCommands) > expected.commands
                || refusal.quantity() != render::RenderQuantity::Work(work)
                || refusal.current() != limit
                || refusal.amount() != 1
                || refusal.reason() != (render::RenderRefusalReason::Limit { limit })
            {
                return violated(
                    RETRY,
                    "inspection-prefix-or-custody",
                    "refusal must preserve authored scene, accepted prefix, and release scratch",
                );
            }
            let retry_limits = match product_observation(
                render::RenderLimits::new().with_work_limit(work, retry_limit),
                RETRY,
                "inspection-work-limit-rejected",
            )? {
                Ok(limits) => limits,
                Err(failure) => return Ok(failure),
            };
            context.set_options(render::RenderAccountingOptions::new().with_limits(retry_limits));
            if let Some(failure) = compare_inspection(
                expected,
                scene.inspect(&mut context),
                RETRY,
                "inspection-retry-rejected",
            )? {
                return Ok(failure);
            }
            return rejected(
                &[STRUCTURE, IDENTITY, RETRY],
                "typed work refusal preserved bounded prefix and fresh retry succeeded",
            );
        }
        evaluated(&[STRUCTURE, IDENTITY])
    }
    fn read(reader: &mut Reader<'_>, budget: &mut Budget) -> Result<Self> {
        let count = reader.count(MAX_IMAGES)?;
        credit(budget, count * std::mem::size_of::<ImageRecipe>(), 0, 0)?;
        let mut images = Vec::new();
        images
            .try_reserve_exact(count)
            .map_err(|_| error("image slots allocation"))?;
        let mut aggregate = 0;
        for _ in 0..count {
            let image = ImageRecipe::read(reader, budget, MAX_RGBA - aggregate)?;
            aggregate += image.rgba.len();
            if aggregate > MAX_RGBA {
                return Err(error("aggregate RGBA ceiling"));
            }
            images.push(image);
        }
        let count = reader.count(MAX_COMMANDS)?;
        credit(budget, count * 1024, count, usize::from(count != 0))?;
        let mut commands = Vec::new();
        commands
            .try_reserve_exact(count)
            .map_err(|_| error("command slots allocation"))?;
        let mut depths = [0; MAX_COMMANDS];
        for index in 0..count {
            let parent = reader.u8()?;
            let depth = if parent == ROOT {
                1
            } else {
                let parent = usize::from(parent);
                if parent >= index
                    || !matches!(
                        commands[parent],
                        Node {
                            draw: Draw::Layer,
                            ..
                        }
                    )
                {
                    return Err(error("parent must be an earlier layer"));
                }
                depths[parent] + 1
            };
            if depth > MAX_DEPTH {
                return Err(error("scene depth ceiling"));
            }
            depths[index] = depth;
            let tag = reader.u8()?;
            let draw = match tag {
                0 => Draw::Layer,
                1 => Draw::Fill(reader.u8()?),
                2 => Draw::Stroke(reader.u8()?),
                3 => Draw::Shadow(reader.u8()?),
                4 => Draw::Image(reader.u8()?),
                5 => Draw::Glyphs(reader.u8()?),
                _ => return Err(error("unknown scene command tag")),
            };
            commands.push(Node { parent, draw });
        }
        let refusal = match reader.u8()? {
            0 => Refusal::None,
            1 => Refusal::Commands {
                limit: reader.u64()?,
                retry_limit: reader.u64()?,
            },
            2 => Refusal::ResourceProbe {
                retry_limit: reader.u64()?,
            },
            3 => Refusal::IdentityByte {
                retry_limit: reader.u64()?,
            },
            _ => return Err(error("unknown inspection refusal tag")),
        };
        Ok(Self {
            images,
            commands,
            refusal,
        })
    }
    fn write(&self, writer: &mut Writer) -> Result<()> {
        writer.count(self.images.len(), MAX_IMAGES)?;
        for image in &self.images {
            image.write(writer)?;
        }
        writer.count(self.commands.len(), MAX_COMMANDS)?;
        for node in &self.commands {
            writer.u8(node.parent)?;
            match node.draw {
                Draw::Layer => writer.u8(0)?,
                Draw::Fill(image) => {
                    writer.u8(1)?;
                    writer.u8(image)?;
                }
                Draw::Stroke(image) => {
                    writer.u8(2)?;
                    writer.u8(image)?;
                }
                Draw::Shadow(image) => {
                    writer.u8(3)?;
                    writer.u8(image)?;
                }
                Draw::Image(image) => {
                    writer.u8(4)?;
                    writer.u8(image)?;
                }
                Draw::Glyphs(count) => {
                    writer.u8(5)?;
                    writer.u8(count)?;
                }
            }
        }
        match self.refusal {
            Refusal::None => writer.u8(0),
            Refusal::Commands { limit, retry_limit } => {
                writer.u8(1)?;
                writer.u64(limit)?;
                writer.u64(retry_limit)
            }
            Refusal::ResourceProbe { retry_limit } => {
                writer.u8(2)?;
                writer.u64(retry_limit)
            }
            Refusal::IdentityByte { retry_limit } => {
                writer.u8(3)?;
                writer.u64(retry_limit)
            }
        }
    }
}

fn compare_summary(expected: SceneSummary, actual: SceneSummary) -> Result<Option<CaseOutcome>> {
    if (
        expected.commands,
        expected.fills,
        expected.strokes,
        expected.shadows,
        expected.glyphs,
        expected.layers,
    ) != (
        actual.commands,
        actual.fills,
        actual.strokes,
        actual.shadows,
        actual.glyphs,
        actual.layers,
    ) {
        return violated(
            STRUCTURE,
            "authored-summary-counts",
            format!("expected {expected:?}; observed {actual:?}"),
        )
        .map(Some);
    }
    if (
        expected.images,
        expected.image_content_hits,
        expected.image_content_misses,
        expected.first_observed_image_bytes,
    ) != (
        actual.images,
        actual.image_content_hits,
        actual.image_content_misses,
        actual.first_observed_image_bytes,
    ) {
        return violated(
            IDENTITY,
            "exact-image-content-counts",
            format!("expected {expected:?}; observed {actual:?}"),
        )
        .map(Some);
    }
    Ok(None)
}

fn compare_inspection(
    expected: SceneSummary,
    observed: render::Result<SceneSummary>,
    property: &str,
    discriminator: &str,
) -> Result<Option<CaseOutcome>> {
    match product_observation(observed, property, discriminator)? {
        Ok(actual) => compare_summary(expected, actual),
        Err(failure) => Ok(Some(failure)),
    }
}
