//! Standalone consumer of an authentic immutable Text layout.

use std::mem::size_of;

use surgeist_text::{Brush, Layout, RunView};

use crate::{
    Color, Error, Layer, NormalizedCoordinate, OutlineLimits, OutlineSynthesis, Paint, Result,
    Scene, Shape, TextGlyph, TextInstance, TextPaint, Transform,
    scene::Command,
    text::{AdmittedOutlineFont, qualify_bounds_placement, qualify_outline_placement},
};

/// Whole-invocation bounds for a standalone Text-layout scene.
///
/// Outline glyph/path/temporary/retained limits apply across all runs, rather
/// than resetting for each run. Input-loaned Text storage is excluded. Retained
/// bytes include the output command tree, font copies and prepared geometry;
/// temporary bytes include mapping and face-table capacities and copy overlap.
/// Ordinary failure-diagnostic String/Box allocations are outside these bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLayoutLimits {
    outlines: OutlineLimits,
    runs: usize,
    coordinate_values: usize,
    decorations: usize,
}

impl TextLayoutLimits {
    /// Sets explicit limits, including meaningful zero-capacity limits.
    #[must_use]
    pub const fn new(
        outlines: OutlineLimits,
        runs: usize,
        coordinate_values: usize,
        decorations: usize,
    ) -> Self {
        Self {
            outlines,
            runs,
            coordinate_values,
            decorations,
        }
    }
}

/// One scene paired with the exact standalone snapshot that supplied its facts.
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
}

/// An aggregate count or storage resource in standalone layout admission.
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

/// Typed standalone adapter refusal; no prefix scene is published.
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
/// from Text. The published outline domain is unchanged; unsupported runs refuse
/// the complete operation. Supplied placement is qualified for glyph ink and
/// decoration geometry before returning. Future parent/surface composition is
/// still checked by the actual renderer. This is a standalone-layout adapter,
/// not a finalized Layout-fragment or captured-style join.
pub fn prepare_text_layout(
    layout: &Layout,
    placement: Transform,
    limits: TextLayoutLimits,
) -> Result<PreparedTextScene<'_>> {
    let counts = preflight(layout, placement, limits)?;
    let mut budget = Budget::new(limits.outlines);
    budget.retain(size_of::<PreparedTextScene<'_>>())?;
    let layered = placement != Transform::identity() && counts.decorations != 0;
    let command_count = add(counts.runs, if layered { 1 } else { counts.decorations })?;
    let mut commands = budget.output::<Command>(command_count)?;
    let mut decoration_commands = if layered {
        budget.output::<Command>(counts.decorations)?
    } else {
        Vec::new()
    };
    let mut faces = budget.temporary::<FaceCopy<'_>>(counts.runs)?;
    let mut glyphs = budget.temporary::<TextGlyph>(counts.max_glyphs)?;

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
        let face = instance.face();
        let face_index = faces
            .iter()
            .position(|copy| face.is_same_face(&copy.source.face()));
        let new_copy = if face_index.is_none() {
            let copy = AdmittedOutlineFont::copy_from(
                face.bytes(),
                face.index(),
                budget.remaining_scratch(),
                budget.remaining_retained(),
            )
            .map_err(|e| TextLayoutError::render(location, e))?;
            budget.retain(copy.retained_bytes())?;
            Some(copy)
        } else {
            None
        };
        glyphs.clear();
        for glyph in run.glyphs() {
            glyphs.push(text_glyph(glyph).map_err(|e| TextLayoutError::render(location, e))?);
        }
        let mut coordinates = budget.temporary::<NormalizedCoordinate>(instance.axis_count())?;
        let coordinate_bytes = bytes::<NormalizedCoordinate>(coordinates.capacity())?;
        for bits in instance.normalized_coords() {
            coordinates.push(
                NormalizedCoordinate::try_from_bits(bits)
                    .map_err(|e| TextLayoutError::render(location, e))?,
            );
        }
        let synthesis = OutlineSynthesis::try_new(instance.x_shear(), instance.embolden())
            .map_err(|e| TextLayoutError::render(location, e))?;
        let instance = TextInstance::try_new(instance.font_size(), coordinates, synthesis)
            .map_err(|e| TextLayoutError::render(location, e))?;
        let remaining = budget.outline_limits();
        let admitted = match face_index {
            Some(index) => &faces[index].admitted,
            None => new_copy.as_ref().expect("new face has an admitted copy"),
        };
        let (outline, retained) = admitted
            .prepare(&instance, &glyphs, remaining)
            .map_err(|e| TextLayoutError::render(location, e))?;
        budget.retain(retained)?;
        budget.glyphs = add(budget.glyphs, glyphs.len())?;
        budget.path_elements = add(budget.path_elements, outline.path().elements().len())?;
        qualify_outline_placement(kurbo::Affine::new(run_transform.as_array()), &outline)
            .map_err(|e| TextLayoutError::render(location, e))?;
        let paint = TextPaint::try_fill(Paint::from(
            color(run.brush()).map_err(|e| TextLayoutError::render(location, e))?,
        ))
        .map_err(|e| TextLayoutError::render(location, e))?;
        commands.push(Command::PreparedOutline {
            outline,
            transform: run_transform,
            paint,
        });
        drop(instance);
        budget.release_scratch(coordinate_bytes);
        if let Some(admitted) = new_copy {
            faces.push(FaceCopy {
                source: run,
                admitted,
            });
        }
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
            decoration_commands.push(command);
        } else {
            commands.push(command);
        }
    }
    if layered {
        let layer = Layer::new().try_transform(placement)?;
        commands.push(Command::Layer {
            layer,
            children: decoration_commands,
        });
    }
    // These actual buffers are disposed before their custody can leave the builder.
    drop(glyphs);
    drop(faces);
    Ok(PreparedTextScene {
        layout,
        scene: Scene { commands },
    })
}

struct FaceCopy<'a> {
    source: RunView<'a>,
    admitted: AdmittedOutlineFont,
}

struct Counts {
    runs: usize,
    decorations: usize,
    max_glyphs: usize,
}

fn preflight(layout: &Layout, placement: Transform, limits: TextLayoutLimits) -> Result<Counts> {
    let mut runs = 0;
    let mut glyphs = 0;
    let mut coordinates = 0;
    let mut max_glyphs = 0;
    let [glyph_limit, _, _, _] = limits.outlines.capacities();
    for (index, run) in layout.run_views().enumerate() {
        if run.glyphs().next().is_none() {
            continue;
        }
        runs = add(runs, 1)?;
        check(TextLayoutResource::Runs, runs, limits.runs)?;
        let location = TextLayoutLocation::Run {
            line: run.line(),
            run: index,
        };
        let instance = run
            .effective_instance()
            .map_err(|e| TextLayoutError::text(location, e))?;
        run_transform(&run, placement).map_err(|e| TextLayoutError::render(location, e))?;
        coordinates = add(coordinates, instance.axis_count())?;
        check(
            TextLayoutResource::CoordinateValues,
            coordinates,
            limits.coordinate_values,
        )?;
        let mut run_glyphs = 0;
        for glyph in run.glyphs() {
            glyphs = add(glyphs, 1)?;
            run_glyphs = add(run_glyphs, 1)?;
            check(TextLayoutResource::Glyphs, glyphs, glyph_limit)?;
            text_glyph(glyph).map_err(|e| TextLayoutError::render(location, e))?;
        }
        max_glyphs = max_glyphs.max(run_glyphs);
        color(run.brush()).map_err(|e| TextLayoutError::render(location, e))?;
    }
    let mut decorations = 0;
    for (index, decoration) in layout.decoration_views().enumerate() {
        decorations = add(decorations, 1)?;
        check(
            TextLayoutResource::Decorations,
            decorations,
            limits.decorations,
        )?;
        let location = TextLayoutLocation::Decoration { decoration: index };
        let rect =
            decoration_rect(decoration.rect()).map_err(|e| TextLayoutError::render(location, e))?;
        qualify_bounds_placement(kurbo::Affine::new(placement.as_array()), Some(rect))
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

fn text_glyph(glyph: surgeist_text::GlyphPlacement) -> Result<TextGlyph> {
    TextGlyph::try_new(
        glyph.id(),
        glyph.pen() + f64::from(glyph.x()),
        f64::from(glyph.y()),
        glyph.advance(),
    )
}

fn run_transform(run: &RunView<'_>, placement: Transform) -> Result<Transform> {
    let origin = run.origin();
    Transform::translation(origin.x, origin.y)?.then(placement)
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
fn bytes<T>(count: usize) -> Result<usize> {
    count
        .checked_mul(size_of::<T>())
        .ok_or_else(|| TextLayoutError::CapacityOverflow.into())
}
fn check(resource: TextLayoutResource, required: usize, limit: usize) -> Result<()> {
    if required > limit {
        return Err(TextLayoutError::Limit {
            resource,
            required,
            limit,
        }
        .into());
    }
    Ok(())
}

struct Budget {
    capacities: [usize; 4],
    retained: usize,
    scratch: usize,
    glyphs: usize,
    path_elements: usize,
}
impl Budget {
    fn new(limits: OutlineLimits) -> Self {
        Self {
            capacities: limits.capacities(),
            retained: 0,
            scratch: 0,
            glyphs: 0,
            path_elements: 0,
        }
    }
    fn retain(&mut self, amount: usize) -> Result<()> {
        let total = add(self.retained, amount)?;
        check(TextLayoutResource::RetainedBytes, total, self.capacities[3])?;
        self.retained = total;
        Ok(())
    }
    fn output<T>(&mut self, capacity: usize) -> Result<Vec<T>> {
        check(
            TextLayoutResource::RetainedBytes,
            add(self.retained, bytes::<T>(capacity)?)?,
            self.capacities[3],
        )?;
        let mut values = Vec::new();
        values.try_reserve_exact(capacity).map_err(|_| {
            Error::from(TextLayoutError::Allocation {
                resource: TextLayoutResource::RetainedBytes,
            })
        })?;
        self.retain(bytes::<T>(values.capacity())?)?;
        Ok(values)
    }
    fn temporary<T>(&mut self, capacity: usize) -> Result<Vec<T>> {
        check(
            TextLayoutResource::ScratchBytes,
            add(self.scratch, bytes::<T>(capacity)?)?,
            self.capacities[2],
        )?;
        let mut values = Vec::new();
        values.try_reserve_exact(capacity).map_err(|_| {
            Error::from(TextLayoutError::Allocation {
                resource: TextLayoutResource::ScratchBytes,
            })
        })?;
        self.scratch = add(self.scratch, bytes::<T>(values.capacity())?)?;
        check(
            TextLayoutResource::ScratchBytes,
            self.scratch,
            self.capacities[2],
        )?;
        Ok(values)
    }
    fn release_scratch(&mut self, amount: usize) {
        self.scratch -= amount;
    }
    fn remaining_scratch(&self) -> usize {
        self.capacities[2] - self.scratch
    }
    fn remaining_retained(&self) -> usize {
        self.capacities[3] - self.retained
    }
    fn outline_limits(&self) -> OutlineLimits {
        OutlineLimits::new(
            self.capacities[0] - self.glyphs,
            self.capacities[1] - self.path_elements,
            self.remaining_scratch(),
            self.remaining_retained(),
        )
    }
}
