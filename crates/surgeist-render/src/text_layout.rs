//! Consumers of authentic immutable Text layouts and finalized leader patterns.

mod leader;
pub use leader::{
    LeaderPatternCause, LeaderPatternError, PreparedLeaderGlyph, PreparedLeaderGroup,
    PreparedLeaderPattern, PreparedLeaderRun, prepare_leader_pattern,
};

use crate::accounting::{
    Account, RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderResource, RenderWork,
};
use std::mem::size_of;

use surgeist_font::ResourceId;
use surgeist_text::{Brush, Layout, RunView};

use crate::{
    Color, Error, FontData, GlyphRunLimits, GlyphSynthesis, Layer, NormalizedCoordinate, Paint,
    PreparedGlyphRun, Result, Scene, Shape, TextGlyph, TextInstance, TextPaint, Transform,
    scene::Command,
    text::{qualify_preparation_bounds, qualify_preparation_placement},
};

/// Whole-invocation bounds for a placed Text-layout scene.
///
/// Glyph/path/paint/image/temporary/retained limits apply across all runs, rather
/// than resetting for each run. Retained bytes include the output command tree,
/// prepared geometry and each shared font resource retained by the scene once.
/// Temporary bytes include mapping and resource identity buffers. The remaining
/// storage borrowed from Text is outside these bounds.
/// Ordinary failure-diagnostic String/Box allocations are outside these bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLayoutLimits {
    glyph_limits: GlyphRunLimits,
    runs: usize,
    coordinate_values: usize,
    decorations: usize,
}

impl TextLayoutLimits {
    /// Sets explicit limits, including meaningful zero-capacity limits.
    #[must_use]
    pub const fn new(
        glyph_limits: GlyphRunLimits,
        runs: usize,
        coordinate_values: usize,
        decorations: usize,
    ) -> Self {
        Self {
            glyph_limits,
            runs,
            coordinate_values,
            decorations,
        }
    }
}

/// One scene paired with the exact placed snapshot that supplied its facts.
///
/// The scene owns admitted paint resources; the borrowed layout remains the
/// authority for interaction and source/catalogue provenance. No second-layout
/// or external-fragment join is accepted. Dropping this value has no callbacks.
#[derive(Debug)]
pub struct PreparedTextScene<'a> {
    layout: &'a Layout,
    scene: Scene,
}

impl<'a> PreparedTextScene<'a> {
    /// Returns the exact immutable input, without a deep clone or reconstructed key.
    #[must_use]
    pub const fn layout(&self) -> &'a Layout {
        self.layout
    }

    /// Returns the complete owned scene, available only after all admission succeeds.
    #[must_use]
    pub const fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Borrows admitted glyph draws in emitted paint order, including empty ink.
    ///
    /// The adapter stores glyph draws at the scene root. Its layers contain
    /// decoration fills, which are outside this glyph-draw view.
    pub fn glyph_draws(&self) -> impl Iterator<Item = PreparedGlyphDraw<'_>> {
        self.scene
            .commands
            .iter()
            .filter_map(|command| match command {
                Command::PreparedGlyphRun {
                    run,
                    transform,
                    paint,
                } => Some(PreparedGlyphDraw {
                    run,
                    transform: *transform,
                    paint,
                }),
                _ => None,
            })
    }
}

/// An immutable draw in a prepared Text scene.
#[derive(Clone, Copy, Debug)]
pub struct PreparedGlyphDraw<'scene> {
    run: &'scene PreparedGlyphRun,
    transform: Transform,
    paint: &'scene TextPaint,
}

impl<'scene> PreparedGlyphDraw<'scene> {
    /// Borrows the actual admitted run and its retained font and ink.
    #[must_use]
    pub const fn run(self) -> &'scene PreparedGlyphRun {
        self.run
    }

    /// Returns the stored transform from run coordinates to scene coordinates.
    #[must_use]
    pub const fn transform(self) -> Transform {
        self.transform
    }

    /// Borrows the paint stored with the draw.
    #[must_use]
    pub const fn paint(self) -> &'scene TextPaint {
        self.paint
    }
}

/// An aggregate count or storage resource in placed layout admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextLayoutResource {
    /// Actual positioned glyph runs.
    Runs,
    /// All effective-instance coordinate values, including proven default axes.
    CoordinateValues,
    /// Existing underline/strikethrough rectangles.
    Decorations,
    /// Original glyph-placement facts across all runs.
    Glyphs,
    /// Combined retained outline path elements across all runs.
    PathElements,
    /// Simultaneously live temporary capacities.
    ScratchBytes,
    /// Simultaneously live owned output capacities.
    RetainedBytes,
}

/// Actual item location within the immutable layout's paint traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextLayoutLocation {
    /// A positioned run (global paint ordinal and actual line ordinal).
    Run {
        /// Actual line.
        line: usize,
        /// Global run ordinal.
        run: usize,
    },
    /// An existing decoration rectangle.
    Decoration {
        /// Global decoration ordinal.
        decoration: usize,
    },
}

/// A preserved lower-level refusal at an actual layout item.
#[derive(Debug)]
pub enum TextLayoutCause {
    /// Text could not qualify the actual face/effective instance or synthesis.
    Text(Box<surgeist_text::Error>),
    /// Render refused the actual font, paint, numeric facts or outline capability.
    Render(Box<Error>),
}

/// Typed placed-layout adapter refusal; no prefix scene is published.
#[derive(Debug)]
pub enum TextLayoutError {
    /// An aggregate requirement exceeds its caller-supplied bound.
    Limit {
        /// Bounded resource.
        resource: TextLayoutResource,
        /// Actual required amount.
        required: usize,
        /// Supplied bound.
        limit: usize,
    },
    /// Checked count/storage arithmetic overflowed.
    CapacityOverflow,
    /// Fallible buffer reservation failed.
    Allocation {
        /// Allocation's logical resource.
        resource: TextLayoutResource,
    },
    /// A lower-level refusal with authentic traversal context.
    Item {
        /// Actual item.
        location: TextLayoutLocation,
        /// Preserved typed cause.
        cause: TextLayoutCause,
    },
}

impl TextLayoutError {
    fn render(location: TextLayoutLocation, cause: Error) -> Error {
        Self::Item {
            location,
            cause: TextLayoutCause::Render(Box::new(cause)),
        }
        .into()
    }
    fn text(location: TextLayoutLocation, cause: surgeist_text::Error) -> Error {
        Self::Item {
            location,
            cause: TextLayoutCause::Text(Box::new(cause)),
        }
        .into()
    }
}

/// Prepares real Text glyphs and decorations without reshaping or choosing lines.
///
/// Admission is atomic across the whole invocation, including simultaneous
/// temporary/output custody. Actual normalized bits and executable synthesis come
/// from Text. Qualified outlines, default-instance COLRv0/COLRv1 solid-glyph paints
/// and sbix RGBA PNG images retain their executed geometry and resources. Unsupported runs refuse
/// the complete operation. Supplied placement is qualified for glyph ink and
/// decoration geometry before returning. Future parent/surface composition is
/// still checked by the actual renderer. Both standalone Text layout and explicit
/// finalized-line placement supply this same immutable input. A Layout batch
/// lends its actual placed Text result; detached fragment rectangles do not
/// supply that authority. Live computed-style authentication remains upstream.
pub fn prepare_text_layout<'a>(
    layout: &'a Layout,
    placement: Transform,
    limits: TextLayoutLimits,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedTextScene<'a>> {
    accounting.prepare(
        RenderAttemptKind::TextLayout,
        limits.glyph_limits,
        Some([limits.runs, limits.coordinate_values, limits.decorations]),
        |account| prepare_inner(layout, placement, account),
    )
}
fn prepare_inner<'a>(
    layout: &'a Layout,
    placement: Transform,
    account: &Account,
) -> Result<PreparedTextScene<'a>> {
    let counts = account.phase(RenderPhase::TextPreflight, || {
        preflight(layout, placement, &account.at(RenderPhase::TextPreflight))
    })?;
    account.retain(size_of::<PreparedTextScene<'_>>())?;
    let layered = placement != Transform::identity() && counts.decorations != 0;
    let command_count = add(counts.runs, if layered { 1 } else { counts.decorations })?;
    let mut commands = account.retained::<Command>(command_count)?;
    let mut decoration_commands = if layered {
        account.retained::<Command>(counts.decorations)?
    } else {
        account.retained(0)?
    };
    let mut resources = account.temporary::<ResourceId>(counts.runs)?;
    let mut glyphs = account.temporary::<TextGlyph>(counts.max_glyphs)?;

    for (run_index, run) in layout.run_views().enumerate() {
        // Text retains metric-only font/style rows even when they have no
        // glyph paint. Decorations are visited independently below.
        if run.glyphs().next().is_none() {
            continue;
        }
        let location = TextLayoutLocation::Run {
            line: run.line(),
            run: run_index,
        };
        let instance = run
            .effective_instance()
            .map_err(|e| TextLayoutError::text(location, e))?;
        let run_transform =
            run_transform(&run, placement).map_err(|e| TextLayoutError::render(location, e))?;
        let face = instance.face().admitted_face();
        let resource = face.resource();
        let identity = resource.identity();
        let mut known = false;
        for prior in &resources {
            account.work(RenderWork::ResourceProbes, 1)?;
            if *prior == identity {
                known = true;
                break;
            }
        }
        if !known {
            account.retain(resource.retained_bytes())?;
            resources.push(identity)?;
        }
        let font = FontData::from_face(face.clone());
        glyphs.clear();
        for glyph in run.glyphs() {
            account.work(RenderWork::GlyphVisits, 1)?;
            glyphs
                .push(text_glyph(&run, glyph).map_err(|e| TextLayoutError::render(location, e))?)?;
        }
        let mut coordinates = account.temporary::<NormalizedCoordinate>(instance.axis_count())?;
        account.phase(RenderPhase::Coordinates, || {
            for bits in instance.normalized_coords() {
                account
                    .at(RenderPhase::Coordinates)
                    .work(RenderWork::CoordinateVisits, 1)?;
                coordinates.push(
                    NormalizedCoordinate::try_from_bits(bits)
                        .map_err(|e| TextLayoutError::render(location, e))?,
                )?;
            }
            Ok(())
        })?;
        let (coordinates, coordinate_hold) = coordinates.into_parts()?;
        let synthesis = GlyphSynthesis::try_new(instance.x_shear(), instance.embolden())
            .map_err(|e| TextLayoutError::render(location, e))?;
        let instance = TextInstance::try_new(instance.font_size(), coordinates, synthesis)
            .map_err(|e| TextLayoutError::render(location, e))?;
        let glyph_account = account.glyph();
        let outline = font
            .prepare_glyph_run_inner(&instance, &glyphs, &glyph_account, 0)
            .map_err(|e| TextLayoutError::render(location, e))?;
        let placement_account = account.at(RenderPhase::Qualify);
        qualify_preparation_placement(
            kurbo::Affine::new(run_transform.as_array()),
            &outline,
            &placement_account,
        )
        .map_err(|e| TextLayoutError::render(location, e))?;
        let paint = TextPaint::try_fill(Paint::from(
            color(run.brush()).map_err(|e| TextLayoutError::render(location, e))?,
        ))
        .map_err(|e| TextLayoutError::render(location, e))?;
        commands.push(Command::PreparedGlyphRun {
            run: outline,
            transform: run_transform,
            paint,
        })?;
        drop(instance);
        drop(coordinate_hold);
    }

    for (index, decoration) in layout.decoration_views().enumerate() {
        let location = TextLayoutLocation::Decoration { decoration: index };
        let shape =
            decoration_rect(decoration.rect()).map_err(|e| TextLayoutError::render(location, e))?;
        let paint = Paint::from(
            color(decoration.brush()).map_err(|e| TextLayoutError::render(location, e))?,
        );
        let command = Command::Fill {
            shape: Shape::rect(shape),
            paint,
        };
        if layered {
            decoration_commands.push(command)?;
        } else {
            commands.push(command)?;
        }
    }
    if layered {
        let layer = Layer::new().with_transform(placement);
        // This authored metadata box is part of the prepared scene's actual
        // transferred backing, independently of its command-vector storage.
        let layer_hold = account
            .acquire(RenderResource::RetainedCpuBytes, size_of::<Layer>())?
            .into_preparation()?;
        commands.push(Command::Layer {
            layer: Box::new(layer),
            children: decoration_commands
                .into_preparation()?
                .into_retained()
                .into(),
        })?;
        layer_hold.retain();
    }
    // These actual buffers are disposed before their custody can leave the builder.
    drop(glyphs);
    drop(resources);
    account.phase(RenderPhase::Output, || {
        Ok(PreparedTextScene {
            layout,
            scene: Scene {
                commands: commands.into_preparation()?.into_retained(),
            },
        })
    })
}

struct Counts {
    runs: usize,
    decorations: usize,
    max_glyphs: usize,
}

fn preflight(layout: &Layout, placement: Transform, account: &Account) -> Result<Counts> {
    let mut runs = 0;
    let mut max_glyphs = 0;
    for (index, run) in layout.run_views().enumerate() {
        if run.glyphs().next().is_none() {
            continue;
        }
        runs = add(runs, 1)?;
        account.work(RenderWork::TextRuns, 1)?;
        let location = TextLayoutLocation::Run {
            line: run.line(),
            run: index,
        };
        let instance = run
            .effective_instance()
            .map_err(|e| TextLayoutError::text(location, e))?;
        run_transform(&run, placement).map_err(|e| TextLayoutError::render(location, e))?;
        account.work(RenderWork::CoordinateInputs, instance.axis_count())?;
        let mut run_glyphs = 0;
        for glyph in run.glyphs() {
            account.work(RenderWork::GlyphInputs, 1)?;
            account.work(RenderWork::GlyphVisits, 1)?;
            run_glyphs = add(run_glyphs, 1)?;
            text_glyph(&run, glyph).map_err(|e| TextLayoutError::render(location, e))?;
        }
        max_glyphs = max_glyphs.max(run_glyphs);
        color(run.brush()).map_err(|e| TextLayoutError::render(location, e))?;
    }
    let mut decorations = 0;
    for (index, decoration) in layout.decoration_views().enumerate() {
        decorations = add(decorations, 1)?;
        account.work(RenderWork::TextDecorations, 1)?;
        let location = TextLayoutLocation::Decoration { decoration: index };
        let rect =
            decoration_rect(decoration.rect()).map_err(|e| TextLayoutError::render(location, e))?;
        qualify_preparation_bounds(
            kurbo::Affine::new(placement.as_array()),
            Some(rect),
            account,
        )
        .map_err(|e| TextLayoutError::render(location, e))?;
        color(decoration.brush()).map_err(|e| TextLayoutError::render(location, e))?;
    }
    Ok(Counts {
        runs,
        decorations,
        max_glyphs,
    })
}

fn color(brush: Brush) -> Result<Color> {
    Color::try_rgba(brush.r, brush.g, brush.b, brush.a)
}

fn text_glyph(run: &RunView<'_>, glyph: surgeist_text::GlyphPlacement) -> Result<TextGlyph> {
    let dx = f64::from(glyph.x());
    let dy = f64::from(glyph.y());
    let pen = glyph.pen();
    // The executed frame owns the signed unit pen axis in outline coordinates.
    // Keep native addition/subtraction order, including downward-positive y.
    let direction = run.glyph_frame().pen_direction();
    let (x, y) = if direction.x > 0.0 {
        (pen + dx, dy)
    } else if direction.x < 0.0 {
        (dx - pen, dy)
    } else if direction.y > 0.0 {
        (dx, pen + dy)
    } else {
        (dx, dy - pen)
    };
    TextGlyph::try_new(glyph.id(), x, y, glyph.advance())
}

fn run_transform(run: &RunView<'_>, placement: Transform) -> Result<Transform> {
    let frame = run.glyph_frame();
    let origin = frame.origin();
    let x = frame.outline_x();
    let y = frame.outline_y();
    let transform = Transform::try_new([x.x, x.y, y.x, y.y, origin.x, origin.y])?;
    transform.then(placement)
}

fn decoration_rect(rect: surgeist_text::Rect) -> Result<crate::Rect> {
    crate::Rect::try_new(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
    )
}

fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b)
        .ok_or_else(|| TextLayoutError::CapacityOverflow.into())
}
