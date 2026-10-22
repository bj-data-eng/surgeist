//! Exact-instance, headless outline preparation. The retained path is the draw source.

use std::{mem::size_of, sync::Arc};

use skrifa::{
    GlyphId, Tag,
    instance::{LocationRef, NormalizedCoord, Size},
    outline::{DrawSettings, Hinting, OutlineGlyphFormat, OutlinePen, pen::PathStyle},
    raw::TableProvider,
};

use crate::{Error, FontData, PathElement, Point, Rect, Result, TextGlyph, TextRunBounds};

/// One exact provider-normalized 2.14 coordinate, in the closed interval [-1, 1].
/// Axis order is the actual face's fvar order, not a user-axis setting order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalizedCoordinate(i16);

impl NormalizedCoordinate {
    /// Validates signed F2Dot14 bits without normalization or rounding.
    pub fn try_from_bits(bits: i16) -> Result<Self> {
        if !(-16384..=16384).contains(&bits) {
            return Err(OutlinePreparationError::CoordinateValue {
                value: f64::from(bits),
            }
            .into());
        }
        Ok(Self(bits))
    }

    /// Accepts a scalar only when it is exactly on the provider's F2Dot14 lattice.
    /// Already-shaped coordinates are never silently quantized to another instance.
    pub fn try_from_scalar(value: f32) -> Result<Self> {
        if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
            return Err(OutlinePreparationError::CoordinateValue {
                value: f64::from(value),
            }
            .into());
        }
        let bits = value * 16384.0;
        if bits.fract() != 0.0 {
            return Err(OutlinePreparationError::CoordinateLoss { value }.into());
        }
        // The range and exact integral representation were checked above.
        Ok(Self(bits as i16))
    }

    /// Returns the exact signed provider bits.
    #[must_use]
    pub const fn bits(self) -> i16 {
        self.0
    }
}

/// Concrete font-outline synthesis, applied before y reflection and glyph placement.
/// Nonzero emboldening is a distinguishable unsupported request on this entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlineSynthesis {
    skew: f32,
    embolden: f32,
}

impl OutlineSynthesis {
    /// The real no-synthesis case.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            skew: 0.0,
            embolden: 0.0,
        }
    }

    /// Validates x += skew*y and a nonnegative font-outline emboldening displacement.
    pub fn try_new(skew: f32, embolden: f32) -> Result<Self> {
        if !skew.is_finite() || !embolden.is_finite() || embolden < 0.0 {
            return Err(OutlinePreparationError::InvalidSynthesis.into());
        }
        Ok(Self { skew, embolden })
    }

    /// Returns the executable font-outline skew factor.
    #[must_use]
    pub const fn skew(self) -> f32 {
        self.skew
    }
    /// Returns the requested emboldening displacement.
    #[must_use]
    pub const fn embolden(self) -> f32 {
        self.embolden
    }
}

/// Resolved renderer input, without font matching, axis resolution or shaping.
/// A face-dependent exact axis-count check occurs during preparation.
#[derive(Clone, Debug, PartialEq)]
pub struct TextInstance {
    size: f32,
    coordinates: Vec<NormalizedCoordinate>,
    synthesis: OutlineSynthesis,
}

impl TextInstance {
    /// Takes canonical shaping size and all actual normalized coordinates in fvar order.
    pub fn try_new(
        size: f32,
        coordinates: Vec<NormalizedCoordinate>,
        synthesis: OutlineSynthesis,
    ) -> Result<Self> {
        if !size.is_finite() || size <= 0.0 {
            return Err(OutlinePreparationError::Numeric {
                phase: OutlineNumericPhase::Size,
                value: f64::from(size),
            }
            .into());
        }
        Ok(Self {
            size,
            coordinates,
            synthesis,
        })
    }
    /// Returns the canonical shaping size.
    #[must_use]
    pub const fn size(&self) -> f32 {
        self.size
    }
    /// Returns exact provider coordinates, including zero-valued axes.
    #[must_use]
    pub fn coordinates(&self) -> &[NormalizedCoordinate] {
        &self.coordinates
    }
    /// Returns the concrete requested synthesis.
    #[must_use]
    pub const fn synthesis(&self) -> OutlineSynthesis {
        self.synthesis
    }
}

/// Finite per-invocation bounds. Retained bytes include the font backing, glyphs,
/// coordinates, combined path capacity/reallocation overlap and fixed result
/// backing; scratch is separate.
/// These are storage bounds, not a provider CPU-time guarantee.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutlineLimits {
    glyphs: usize,
    path_elements: usize,
    scratch_bytes: usize,
    retained_bytes: usize,
}

impl OutlineLimits {
    /// Creates explicit limits, including meaningful zero-capacity limits.
    #[must_use]
    pub const fn new(
        glyphs: usize,
        path_elements: usize,
        scratch_bytes: usize,
        retained_bytes: usize,
    ) -> Self {
        Self {
            glyphs,
            path_elements,
            scratch_bytes,
            retained_bytes,
        }
    }
}

/// The resource for a finite admission or allocation refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutlineResource {
    /// Original glyph facts.
    Glyphs,
    /// Combined run-wide path elements.
    PathElements,
    /// Temporary provider draw memory and coordinate conversion.
    ScratchBytes,
    /// Actual retained immutable backing.
    RetainedBytes,
}

/// A numeric boundary in exact outline admission or placement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutlineNumericPhase {
    /// Canonical shaping size.
    Size,
    /// Actual provider point/delta/fixed arithmetic cannot preserve the instance.
    ProviderPoint,
    /// Actual outline/skew/placement coordinate conversion.
    PathPoint,
    /// Positive source geometry collapsed during conversion.
    InkExtent,
    /// Actual backend affine conversion/composition.
    Placement,
}

/// Variation metadata whose malformed presence cannot mean static/default font.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariationTable {
    /// Axis inventory and instance metadata.
    Fvar,
    /// Selected glyph variation data.
    Gvar,
    /// Presence controls the provider's phantom-point rounding.
    Hvar,
}

/// Outline processing that is not yet numerically qualified by this entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutlineCapability {
    /// TrueType component transforms and recursive assembly.
    CompositeGlyph,
    /// PostScript font matrices and charstring arithmetic.
    PostScript,
    /// Variable PostScript font matrices and charstring arithmetic.
    PostScriptVariation,
    /// Variable composite transforms and component outline formats.
    VariableComposite,
    /// Sparse gvar tuples requiring the provider's private IUP interpolation.
    SparseVariation,
}

/// Semantic headless preparation refusal; no partial run or Scene is published.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutlinePreparationError {
    /// An actual selected outline capability has not been qualified on this entry.
    UnsupportedOutline {
        /// Required capability.
        capability: OutlineCapability,
    },
    /// Present font or outline metadata is invalid.
    InvalidFont,
    /// A selected glyph is outside the actual face.
    GlyphRange {
        /// Requested glyph.
        glyph: u32,
        /// Actual glyph count.
        count: u32,
    },
    /// Coordinates do not cover exactly the actual face's axes.
    CoordinateCount {
        /// Supplied count.
        supplied: usize,
        /// Actual axis count.
        required: usize,
    },
    /// Coordinate is not in the normalized domain.
    CoordinateValue {
        /// Rejected value.
        value: f64,
    },
    /// Scalar would change during provider quantization.
    CoordinateLoss {
        /// Rejected scalar.
        value: f32,
    },
    /// Malformed present variation metadata.
    InvalidVariation {
        /// Malformed table.
        table: VariationTable,
    },
    /// Selected COLR drawing is outside this entry's supported domain.
    UnsupportedColor,
    /// Selected bitmap drawing is outside this entry's supported domain.
    UnsupportedBitmap,
    /// Nonfinite or negative synthesis input.
    InvalidSynthesis,
    /// Nonzero emboldening needs a qualified expansion implementation.
    UnsupportedEmboldening,
    /// Canonical numeric representation cannot preserve this input/output.
    Numeric {
        /// Failing boundary.
        phase: OutlineNumericPhase,
        /// Rejected value.
        value: f64,
    },
    /// The provider refused an actual selected outline.
    Provider {
        /// Selected glyph.
        glyph: u32,
    },
    /// Required storage exceeds a caller limit.
    Limit {
        /// Bounded resource.
        resource: OutlineResource,
        /// Required amount.
        required: usize,
        /// Caller limit.
        limit: usize,
    },
    /// Checked storage-size arithmetic overflowed.
    CapacityOverflow,
    /// Safe fallible reservation failed.
    Allocation {
        /// Refused resource.
        resource: OutlineResource,
    },
}

/// Owner-issued immutable run whose retained canonical path is also its GPU draw source.
/// Clones share one preparation identity and actual font/instance/geometry backing.
#[derive(Clone, Debug)]
pub struct PreparedOutline(Arc<PreparedOutlineData>);

#[derive(Debug)]
struct PreparedOutlineData {
    font: FontData,
    instance: TextInstance,
    glyphs: Vec<TextGlyph>,
    path: kurbo::BezPath,
    bounds: TextRunBounds,
}

impl PartialEq for PreparedOutline {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl PreparedOutline {
    /// Reports identity of the actual preparation, not equal font bytes or caller labels.
    #[must_use]
    pub fn is_same_preparation(&self, other: &Self) -> bool {
        self == other
    }
    /// Returns the retained actual admitted face.
    #[must_use]
    pub fn font(&self) -> &FontData {
        &self.0.font
    }
    /// Returns the exact executed instance.
    #[must_use]
    pub fn instance(&self) -> &TextInstance {
        &self.0.instance
    }
    /// Returns original glyph facts, including advances distinct from ink.
    #[must_use]
    pub fn glyphs(&self) -> &[TextGlyph] {
        &self.0.glyphs
    }
    /// Returns actual run-local Ink or proved Empty; never Unspecified.
    #[must_use]
    pub fn bounds(&self) -> TextRunBounds {
        self.0.bounds
    }
    /// Reads the canonical run-wide path without exposing a provider/backend object.
    pub fn path_elements(&self) -> impl Iterator<Item = PathElement> + '_ {
        fn point(p: kurbo::Point) -> Point {
            Point::try_new(p.x, p.y).expect("prepared path points are finite")
        }
        self.0.path.elements().iter().map(|element| match *element {
            kurbo::PathEl::MoveTo(p) => PathElement::MoveTo(point(p)),
            kurbo::PathEl::LineTo(p) => PathElement::LineTo(point(p)),
            kurbo::PathEl::QuadTo(a, p) => PathElement::QuadTo(point(a), point(p)),
            kurbo::PathEl::CurveTo(a, b, p) => PathElement::CubicTo(point(a), point(b), point(p)),
            kurbo::PathEl::ClosePath => PathElement::Close,
        })
    }
    pub(crate) fn path(&self) -> &kurbo::BezPath {
        &self.0.path
    }
}

impl FontData {
    /// Prepares qualified simple-glyf ink at an exact effective instance without a GPU.
    ///
    /// Unscaled HarfBuzz contour interpretation is followed by checked f64
    /// size/UPEM scaling, synthesis, reflection and local placement into one f32
    /// path. Dense gvar tuples are supported; sparse IUP, composite glyphs and
    /// CFF/CFF2/VARC return typed unsupported capabilities.
    ///
    /// All glyphs succeed before an immutable result is published. Selected color,
    /// bitmap and nonzero emboldening requests refuse explicitly, never fall back.
    /// The returned geometry retains this face independently of the input lifetime.
    pub fn prepare_outline(
        &self,
        instance: &TextInstance,
        glyphs: &[TextGlyph],
        limits: OutlineLimits,
    ) -> Result<PreparedOutline> {
        check_limit(OutlineResource::Glyphs, glyphs.len(), limits.glyphs)?;
        let font = skrifa::FontRef::from_index(self.bytes(), self.index())
            .map_err(|_| Error::from(OutlinePreparationError::InvalidFont))?;
        // table_data returns None for an out-of-bounds present record as well as
        // absence. Neither that nor duplicate records may hide variation/color
        // metadata and accidentally select a static or outline fallback.
        for (index, record) in font.table_directory().table_records().iter().enumerate() {
            let tag = record.tag();
            let duplicate = font.table_directory().table_records()[..index]
                .iter()
                .any(|prior| prior.tag() == tag);
            if duplicate || font.table_data(tag).is_none() {
                return Err(match tag.to_be_bytes() {
                    [b'f', b'v', b'a', b'r'] => OutlinePreparationError::InvalidVariation {
                        table: VariationTable::Fvar,
                    },
                    [b'g', b'v', b'a', b'r'] => OutlinePreparationError::InvalidVariation {
                        table: VariationTable::Gvar,
                    },
                    _ => OutlinePreparationError::InvalidFont,
                }
                .into());
            }
        }
        let units = font
            .head()
            .map_err(|_| Error::from(OutlinePreparationError::InvalidFont))?
            .units_per_em();
        if !(16..=16384).contains(&units) {
            return Err(OutlinePreparationError::InvalidFont.into());
        }
        let glyph_count = u32::from(
            font.maxp()
                .map_err(|_| Error::from(OutlinePreparationError::InvalidFont))?
                .num_glyphs(),
        );
        validate_variations(&font, instance, glyphs, glyph_count)?;
        if instance.synthesis.embolden != 0.0 {
            return Err(OutlinePreparationError::UnsupportedEmboldening.into());
        }
        // Blob::from(Vec) boxes the byte slice before sharing it. Charge that
        // observable extent plus declared conservative logical owner headers,
        // never the historical input Vec's discarded spare capacity.
        let base_bytes = self
            .bytes()
            .len()
            .checked_add(size_of::<Box<[u8]>>() + 2 * size_of::<usize>())
            .and_then(|n| n.checked_add(size_of::<PreparedOutlineData>() + 2 * size_of::<usize>()))
            .and_then(|n| {
                glyphs
                    .len()
                    .checked_mul(size_of::<TextGlyph>())
                    .and_then(|g| n.checked_add(g))
            })
            .and_then(|n| {
                instance
                    .coordinates
                    .len()
                    .checked_mul(size_of::<NormalizedCoordinate>())
                    .and_then(|g| n.checked_add(g))
            })
            .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))?;
        check_limit(
            OutlineResource::RetainedBytes,
            base_bytes,
            limits.retained_bytes,
        )?;
        let coord_bytes = instance
            .coordinates
            .len()
            .checked_mul(size_of::<NormalizedCoord>())
            .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))?;
        check_limit(
            OutlineResource::ScratchBytes,
            coord_bytes,
            limits.scratch_bytes,
        )?;
        let mut coordinates = reserved(instance.coordinates.len(), OutlineResource::ScratchBytes)?;
        coordinates.extend(
            instance
                .coordinates
                .iter()
                .map(|c| NormalizedCoord::from_bits(c.0)),
        );
        let coord_bytes = coordinates
            .capacity()
            .checked_mul(size_of::<NormalizedCoord>())
            .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))?;
        check_limit(
            OutlineResource::ScratchBytes,
            coord_bytes,
            limits.scratch_bytes,
        )?;
        let mut owned_coordinates =
            reserved(instance.coordinates.len(), OutlineResource::RetainedBytes)?;
        owned_coordinates.extend_from_slice(&instance.coordinates);
        let mut owned_glyphs = reserved(glyphs.len(), OutlineResource::Glyphs)?;
        owned_glyphs.extend_from_slice(glyphs);
        let extra_capacity = (owned_glyphs.capacity() - glyphs.len())
            .checked_mul(size_of::<TextGlyph>())
            .and_then(|n| {
                (owned_coordinates.capacity() - instance.coordinates.len())
                    .checked_mul(size_of::<NormalizedCoordinate>())
                    .and_then(|c| n.checked_add(c))
            })
            .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))?;
        let base_bytes = base_bytes
            .checked_add(extra_capacity)
            .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))?;
        check_limit(
            OutlineResource::RetainedBytes,
            base_bytes,
            limits.retained_bytes,
        )?;
        let mut pen = CollectingPen::new(limits, base_bytes);
        // Preserve the provider's source priority, but require the selected
        // collection to construct successfully. Its convenience constructor
        // would otherwise try a lower-priority table after a malformed one.
        let selected_format = [
            (Tag::new(b"VARC"), OutlineGlyphFormat::Varc),
            (Tag::new(b"glyf"), OutlineGlyphFormat::Glyf),
            (Tag::new(b"CFF2"), OutlineGlyphFormat::Cff2),
            (Tag::new(b"CFF "), OutlineGlyphFormat::Cff),
        ]
        .into_iter()
        .find(|(tag, _)| font.table_data(*tag).is_some())
        .map(|(_, format)| format);
        let outlines = selected_format
            .filter(|format| matches!(format, OutlineGlyphFormat::Glyf))
            .and_then(|format| skrifa::OutlineGlyphCollection::with_format(&font, format));
        for glyph in glyphs {
            crate::vello_engine::glyph::require_outline_selection(
                &font,
                self,
                glyph.id(),
                instance.size,
            )?;
            let unsupported = match selected_format {
                Some(OutlineGlyphFormat::Cff) => Some(OutlineCapability::PostScript),
                Some(OutlineGlyphFormat::Cff2) => Some(OutlineCapability::PostScriptVariation),
                Some(OutlineGlyphFormat::Varc) => Some(OutlineCapability::VariableComposite),
                _ => None,
            };
            if let Some(capability) = unsupported {
                return Err(OutlinePreparationError::UnsupportedOutline { capability }.into());
            }
            let outlines = outlines
                .as_ref()
                .ok_or_else(|| Error::from(OutlinePreparationError::InvalidFont))?;
            if matches!(
                qualify_simple_glyf(
                    &font,
                    GlyphId::new(glyph.id()),
                    &coordinates,
                    limits,
                    coord_bytes
                )?,
                QualifiedGlyph::Empty
            ) {
                continue;
            }
            let outline = outlines.get(GlyphId::new(glyph.id())).ok_or_else(|| {
                Error::from(OutlinePreparationError::Provider { glyph: glyph.id() })
            })?;
            let scratch_len = outline.draw_memory_size(Hinting::None);
            let required = scratch_len
                .checked_add(coord_bytes)
                .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))?;
            check_limit(
                OutlineResource::ScratchBytes,
                required,
                limits.scratch_bytes,
            )?;
            let mut scratch = reserved(scratch_len, OutlineResource::ScratchBytes)?;
            let actual_scratch = scratch
                .capacity()
                .checked_add(coord_bytes)
                .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))?;
            check_limit(
                OutlineResource::ScratchBytes,
                actual_scratch,
                limits.scratch_bytes,
            )?;
            scratch.resize(scratch_len, 0u8);
            pen.begin_glyph(
                *glyph,
                instance.synthesis.skew,
                f64::from(instance.size) / f64::from(units),
            );
            outline
                .draw(
                    DrawSettings::unhinted(Size::unscaled(), LocationRef::new(&coordinates))
                        .with_path_style(PathStyle::HarfBuzz)
                        .with_memory(Some(&mut scratch)),
                    &mut pen,
                )
                .map_err(|_| {
                    Error::from(OutlinePreparationError::Provider { glyph: glyph.id() })
                })?;
            pen.finish_glyph()?;
        }
        let (path, bounds) = pen.finish()?;
        Ok(PreparedOutline(Arc::new(PreparedOutlineData {
            font: self.clone(),
            instance: TextInstance {
                size: instance.size,
                coordinates: owned_coordinates,
                synthesis: instance.synthesis,
            },
            glyphs: owned_glyphs,
            path,
            bounds,
        })))
    }
}

fn validate_variations(
    font: &skrifa::FontRef<'_>,
    instance: &TextInstance,
    glyphs: &[TextGlyph],
    glyph_count: u32,
) -> Result<()> {
    let invalid = |table| Error::from(OutlinePreparationError::InvalidVariation { table });
    let axes = if font.table_data(Tag::new(b"fvar")).is_some() {
        let fvar = font.fvar().map_err(|_| invalid(VariationTable::Fvar))?;
        let arrays = fvar
            .axis_instance_arrays()
            .map_err(|_| invalid(VariationTable::Fvar))?;
        if fvar.version().major != 1 || fvar.axis_size() != 20 {
            return Err(invalid(VariationTable::Fvar));
        }
        for axis in arrays.axes() {
            if axis.min_value() > axis.default_value() || axis.default_value() > axis.max_value() {
                return Err(invalid(VariationTable::Fvar));
            }
        }
        for record in arrays.instances().iter() {
            record.map_err(|_| invalid(VariationTable::Fvar))?;
        }
        usize::from(fvar.axis_count())
    } else {
        0
    };
    if instance.coordinates.len() != axes {
        return Err(OutlinePreparationError::CoordinateCount {
            supplied: instance.coordinates.len(),
            required: axes,
        }
        .into());
    }
    let gvar = if font.table_data(Tag::new(b"gvar")).is_some() {
        let gvar = font.gvar().map_err(|_| invalid(VariationTable::Gvar))?;
        if axes == 0
            || usize::from(gvar.axis_count()) != axes
            || u32::from(gvar.glyph_count()) != glyph_count
            || gvar.version().major != 1
        {
            return Err(invalid(VariationTable::Gvar));
        }
        gvar.shared_tuples()
            .map_err(|_| invalid(VariationTable::Gvar))?;
        Some(gvar)
    } else {
        None
    };
    for glyph in glyphs {
        if glyph.id() >= glyph_count {
            return Err(OutlinePreparationError::GlyphRange {
                glyph: glyph.id(),
                count: glyph_count,
            }
            .into());
        }
        if let Some(gvar) = &gvar {
            gvar.glyph_variation_data(GlyphId::new(glyph.id()))
                .map_err(|_| invalid(VariationTable::Gvar))?;
        }
    }
    Ok(())
}

// The unscaled HarfBuzz path avoids the provider's requested-size fixed scale.
// Qualify the actual simple/dense inputs and decoder errors before drawing;
// sparse IUP, components and other outline formats remain explicit capabilities.
enum QualifiedGlyph {
    Empty,
    Simple,
}

fn qualify_simple_glyf(
    font: &skrifa::FontRef<'_>,
    glyph: GlyphId,
    coordinates: &[NormalizedCoord],
    limits: OutlineLimits,
    coordinate_bytes: usize,
) -> Result<QualifiedGlyph> {
    use skrifa::raw::{
        tables::glyf::{Glyph, PointFlags},
        types::{Fixed, Point as RawPoint},
    };
    let invalid = || Error::from(OutlinePreparationError::InvalidFont);
    let invalid_variation = || {
        Error::from(OutlinePreparationError::InvalidVariation {
            table: VariationTable::Gvar,
        })
    };
    let head = font.head().map_err(|_| invalid())?;
    let loca = font
        .loca(Some(head.index_to_loc_format() == 1))
        .map_err(|_| invalid())?;
    let glyf = font.glyf().map_err(|_| invalid())?;
    let selected = loca.get_glyf(glyph, &glyf).map_err(|_| invalid())?;
    let count = match &selected {
        None => 0,
        Some(Glyph::Simple(simple)) => simple.num_points(),
        Some(Glyph::Composite(_)) => {
            return Err(OutlinePreparationError::UnsupportedOutline {
                capability: OutlineCapability::CompositeGlyph,
            }
            .into());
        }
    };
    let points = count
        .checked_add(4)
        .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))?;
    let charge = |source_capacity: usize, flag_capacity: usize, delta_capacity: usize| {
        source_capacity
            .checked_mul(size_of::<RawPoint<i32>>())
            .and_then(|n| n.checked_add(flag_capacity.checked_mul(size_of::<PointFlags>())?))
            .and_then(|n| n.checked_add(delta_capacity.checked_mul(size_of::<RawPoint<f32>>())?))
            .and_then(|n| n.checked_add(coordinate_bytes))
            .ok_or_else(|| Error::from(OutlinePreparationError::CapacityOverflow))
    };
    check_limit(
        OutlineResource::ScratchBytes,
        charge(points, points, points)?,
        limits.scratch_bytes,
    )?;
    let mut source = reserved(points, OutlineResource::ScratchBytes)?;
    source.resize(points, RawPoint { x: 0i32, y: 0i32 });
    let mut flags = reserved(points, OutlineResource::ScratchBytes)?;
    flags.resize(points, PointFlags::default());
    let mut deltas = reserved(points, OutlineResource::ScratchBytes)?;
    deltas.resize(points, RawPoint { x: 0f32, y: 0f32 });
    check_limit(
        OutlineResource::ScratchBytes,
        charge(source.capacity(), flags.capacity(), deltas.capacity())?,
        limits.scratch_bytes,
    )?;
    if let Some(Glyph::Simple(simple)) = &selected {
        simple
            .read_points_fast(&mut source[..count], &mut flags[..count])
            .map_err(|_| invalid())?;
        if source[..count].iter().any(|p| {
            p.x < i32::from(simple.x_min())
                || p.x > i32::from(simple.x_max())
                || p.y < i32::from(simple.y_min())
                || p.y > i32::from(simple.y_max())
        }) {
            return Err(invalid());
        }
    }
    // The collection caches HVAR with .ok(). Validate its present header,
    // even though this entry consumes no horizontal metric result. A source-
    // proved empty glyph bypasses the provider's metric-only phantom draw.
    if font.table_data(Tag::new(b"HVAR")).is_some() {
        font.hvar().map_err(|_| {
            Error::from(OutlinePreparationError::InvalidVariation {
                table: VariationTable::Hvar,
            })
        })?;
    }
    let qualified = if count == 0 {
        QualifiedGlyph::Empty
    } else {
        QualifiedGlyph::Simple
    };
    let Some(_) = font.table_data(Tag::new(b"gvar")) else {
        return Ok(qualified);
    };
    let gvar = font.gvar().map_err(|_| invalid_variation())?;
    let Some(raw) = gvar.data_for_gid(glyph).map_err(|_| invalid_variation())? else {
        return Ok(qualified);
    };
    let header: [u8; 2] = raw
        .as_bytes()
        .get(..2)
        .ok_or_else(invalid_variation)?
        .try_into()
        .map_err(|_| invalid_variation())?;
    let expected = usize::from(u16::from_be_bytes(header) & 0x0fff);
    let data = gvar
        .glyph_variation_data(glyph)
        .map_err(|_| invalid_variation())?
        .ok_or_else(invalid_variation)?;
    let mut tuples = 0;
    for tuple in data.tuples() {
        tuples += 1;
        let peak = tuple.peak();
        if peak.len() != coordinates.len()
            || (0..peak.len()).any(|i| {
                peak.get(i)
                    .is_none_or(|p| !(-16384..=16384).contains(&p.to_bits()))
            })
        {
            return Err(invalid_variation());
        }
        for region in [tuple.intermediate_start(), tuple.intermediate_end()]
            .into_iter()
            .flatten()
        {
            if region.len() != coordinates.len()
                || (0..region.len()).any(|i| {
                    region
                        .get(i)
                        .is_none_or(|p| !(-16384..=16384).contains(&p.to_bits()))
                })
            {
                return Err(invalid_variation());
            }
        }
        if let (Some(start), Some(end)) = (tuple.intermediate_start(), tuple.intermediate_end())
            && (0..peak.len()).any(|i| {
                let start = start.get(i).expect("validated tuple axis count");
                let peak = peak.get(i).expect("validated tuple axis count");
                let end = end.get(i).expect("validated tuple axis count");
                start > peak || peak > end
            })
        {
            return Err(invalid_variation());
        }
        if !tuple.has_deltas_for_all_points() {
            return Err(OutlinePreparationError::UnsupportedOutline {
                capability: OutlineCapability::SparseVariation,
            }
            .into());
        }
        // Verify both streams with the provider's fallible decoder, rather
        // than relying on its silently truncating iterator. Scalar ONE
        // converts raw i32 directly to finite f32, avoiding Fixed conversion.
        tuple
            .accumulate_dense_deltas(&mut deltas, Fixed::ONE)
            .map_err(|_| invalid_variation())?;
        deltas.fill(RawPoint { x: 0.0, y: 0.0 });
        let scalar = if coordinates.is_empty() {
            None
        } else {
            tuple.compute_scalar(coordinates)
        };
        if let Some(scalar) = scalar
            && !(0..=65536).contains(&scalar.to_bits())
        {
            return Err(provider_point(f64::from(scalar.to_bits())));
        }
        let mut decoded = 0;
        for delta in tuple.deltas() {
            let index = usize::from(delta.position);
            if index != decoded || index >= points {
                return Err(invalid_variation());
            }
            decoded += 1;
            if let Some(scalar) = scalar {
                for value in [delta.x_delta, delta.y_delta] {
                    if f64::from(value as f32) != f64::from(value) {
                        return Err(provider_point(f64::from(value)));
                    }
                    // The generic raw decoder uses Fixed::from_i32 and Fixed
                    // multiplication on partial scalars even for f32 output.
                    if scalar != Fixed::ONE {
                        let fixed = i64::from(value) * 65536;
                        let scaled = i64::from(value) * i64::from(scalar.to_bits());
                        if i32::try_from(fixed).is_err() || i32::try_from(scaled).is_err() {
                            return Err(provider_point(f64::from(value)));
                        }
                    }
                }
            }
        }
        if decoded != points {
            return Err(invalid_variation());
        }
        if let Some(scalar) = scalar {
            // Same decoder/scalar/output type as HarfBuzzScaler. Validate
            // each active tuple before its real draw-time accumulation. The
            // 12-bit tuple count and i32 raw delta domain also bound the total
            // magnitude far below f32 overflow.
            // Any error would otherwise be ignored by its simple-glyph path.
            tuple
                .accumulate_dense_deltas(&mut deltas, scalar)
                .map_err(|_| invalid_variation())?;
            if deltas.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
                return Err(provider_point(f64::INFINITY));
            }
        }
    }
    if tuples != expected {
        return Err(invalid_variation());
    }
    Ok(qualified)
}

fn provider_point(value: f64) -> Error {
    OutlinePreparationError::Numeric {
        phase: OutlineNumericPhase::ProviderPoint,
        value,
    }
    .into()
}

fn check_limit(resource: OutlineResource, required: usize, limit: usize) -> Result<()> {
    if required > limit {
        Err(OutlinePreparationError::Limit {
            resource,
            required,
            limit,
        }
        .into())
    } else {
        Ok(())
    }
}

fn reserved<T>(capacity: usize, resource: OutlineResource) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| Error::from(OutlinePreparationError::Allocation { resource }))?;
    Ok(values)
}

struct CollectingPen {
    elements: Vec<kurbo::PathEl>,
    limits: OutlineLimits,
    base_bytes: usize,
    failed: Option<OutlinePreparationError>,
    glyph: TextGlyph,
    skew: f32,
    scale: f64,
    glyph_bounds: Option<kurbo::Rect>,
    bounds: Option<kurbo::Rect>,
    drawing: bool,
}

impl CollectingPen {
    fn new(limits: OutlineLimits, base_bytes: usize) -> Self {
        Self {
            elements: Vec::new(),
            limits,
            base_bytes,
            failed: None,
            glyph: TextGlyph::try_new(0, 0.0, 0.0, 0.0).expect("zero glyph facts are finite"),
            skew: 0.0,
            scale: 1.0,
            glyph_bounds: None,
            bounds: None,
            drawing: false,
        }
    }
    fn begin_glyph(&mut self, glyph: TextGlyph, skew: f32, scale: f64) {
        self.glyph = glyph;
        self.skew = skew;
        self.scale = scale;
        self.glyph_bounds = None;
        self.drawing = false;
    }
    fn point(&mut self, x: f32, y: f32) -> kurbo::Point {
        let values = [
            f64::from(x) * self.scale
                + f64::from(self.skew) * f64::from(y) * self.scale
                + f64::from(self.glyph.x()),
            -f64::from(y) * self.scale + f64::from(self.glyph.y()),
        ];
        let mut canonical = [0.0; 2];
        for (index, value) in values.into_iter().enumerate() {
            let narrowed = value as f32;
            if !value.is_finite() || !narrowed.is_finite() || (value != 0.0 && narrowed == 0.0) {
                self.failed.get_or_insert(OutlinePreparationError::Numeric {
                    phase: OutlineNumericPhase::PathPoint,
                    value,
                });
            }
            canonical[index] = f64::from(narrowed);
        }
        let point = kurbo::Point::new(canonical[0], canonical[1]);
        self.glyph_bounds = Some(
            self.glyph_bounds
                .map_or(kurbo::Rect::from_points(point, point), |r| {
                    r.union_pt(point)
                }),
        );
        point
    }
    fn push(&mut self, element: kurbo::PathEl) {
        if self.failed.is_some() {
            return;
        }
        let Some(required) = self.elements.len().checked_add(1) else {
            self.failed = Some(OutlinePreparationError::CapacityOverflow);
            return;
        };
        if required > self.limits.path_elements {
            self.failed = Some(OutlinePreparationError::Limit {
                resource: OutlineResource::PathElements,
                required,
                limit: self.limits.path_elements,
            });
            return;
        }
        // A reallocation may retain old and new buffers simultaneously. Admit
        // that overlap before requesting growth; charge actual capacity afterward.
        let old_capacity = self.elements.capacity();
        let growing = required > old_capacity;
        let Some(capacity) = (if growing {
            old_capacity.checked_add(required)
        } else {
            Some(old_capacity)
        }) else {
            self.failed = Some(OutlinePreparationError::CapacityOverflow);
            return;
        };
        let Some(bytes) = capacity
            .checked_mul(size_of::<kurbo::PathEl>())
            .and_then(|n| n.checked_add(self.base_bytes))
        else {
            self.failed = Some(OutlinePreparationError::CapacityOverflow);
            return;
        };
        if bytes > self.limits.retained_bytes {
            self.failed = Some(OutlinePreparationError::Limit {
                resource: OutlineResource::RetainedBytes,
                required: bytes,
                limit: self.limits.retained_bytes,
            });
            return;
        }
        if self.elements.try_reserve_exact(1).is_err() {
            self.failed = Some(OutlinePreparationError::Allocation {
                resource: OutlineResource::PathElements,
            });
            return;
        }
        let Some(bytes) = self
            .elements
            .capacity()
            .checked_add(if growing { old_capacity } else { 0 })
            .and_then(|n| n.checked_mul(size_of::<kurbo::PathEl>()))
            .and_then(|n| n.checked_add(self.base_bytes))
        else {
            self.failed = Some(OutlinePreparationError::CapacityOverflow);
            return;
        };
        if bytes > self.limits.retained_bytes {
            self.failed = Some(OutlinePreparationError::Limit {
                resource: OutlineResource::RetainedBytes,
                required: bytes,
                limit: self.limits.retained_bytes,
            });
            return;
        }
        self.elements.push(element);
    }
    fn finish_glyph(&mut self) -> Result<()> {
        if let Some(error) = self.failed {
            return Err(error.into());
        }
        if self.drawing {
            let r = self.glyph_bounds.expect("drawing includes a point");
            if r.width() <= 0.0 || r.height() <= 0.0 {
                return Err(OutlinePreparationError::Numeric {
                    phase: OutlineNumericPhase::InkExtent,
                    value: r.area(),
                }
                .into());
            }
            self.bounds = Some(self.bounds.map_or(r, |previous| previous.union(r)));
        }
        Ok(())
    }
    fn finish(self) -> Result<(kurbo::BezPath, TextRunBounds)> {
        let bounds = match self.bounds {
            None => TextRunBounds::empty(),
            Some(r) => TextRunBounds::try_ink(Rect::try_new(r.x0, r.y0, r.width(), r.height())?)?,
        };
        Ok((kurbo::BezPath::from_vec(self.elements), bounds))
    }
}

impl OutlinePen for CollectingPen {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.push(kurbo::PathEl::MoveTo(p));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.drawing = true;
        let p = self.point(x, y);
        self.push(kurbo::PathEl::LineTo(p));
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.drawing = true;
        let a = self.point(cx, cy);
        let p = self.point(x, y);
        self.push(kurbo::PathEl::QuadTo(a, p));
    }
    fn curve_to(&mut self, ax: f32, ay: f32, bx: f32, by: f32, x: f32, y: f32) {
        self.drawing = true;
        let a = self.point(ax, ay);
        let b = self.point(bx, by);
        let p = self.point(x, y);
        self.push(kurbo::PathEl::CurveTo(a, b, p));
    }
    fn close(&mut self) {
        self.push(kurbo::PathEl::ClosePath);
    }
}

/// Qualifies the actual composed backend affine before any prepared-path encoding.
pub(crate) fn qualify_placement(transform: kurbo::Affine, outline: &PreparedOutline) -> Result<()> {
    let coefficients = transform.as_coeffs();
    for value in coefficients {
        let narrowed = value as f32;
        if !value.is_finite() || !narrowed.is_finite() || (value != 0.0 && narrowed == 0.0) {
            return Err(OutlinePreparationError::Numeric {
                phase: OutlineNumericPhase::Placement,
                value,
            }
            .into());
        }
    }
    let [a, b, c, d, _, _] = coefficients;
    let determinant = a * d - b * c;
    let narrowed = coefficients.map(|v| f64::from(v as f32));
    let canonical_determinant = narrowed[0] * narrowed[3] - narrowed[1] * narrowed[2];
    if determinant != 0.0 && canonical_determinant == 0.0 {
        return Err(OutlinePreparationError::Numeric {
            phase: OutlineNumericPhase::Placement,
            value: determinant,
        }
        .into());
    }
    if let Some(rect) = outline.bounds().ink_rect() {
        let coefficients = coefficients.map(|v| v as f32);
        let [a, b, c, d, e, f] = coefficients;
        let mut placed: Option<kurbo::Rect> = None;
        for (x, y) in [
            (rect.x(), rect.y()),
            (rect.x() + rect.width(), rect.y()),
            (rect.x(), rect.y() + rect.height()),
            (rect.x() + rect.width(), rect.y() + rect.height()),
        ] {
            let local_x = x as f32;
            let local_y = y as f32;
            let placed_x = a * local_x + c * local_y + e;
            let placed_y = b * local_x + d * local_y + f;
            for value in [placed_x, placed_y] {
                if !value.is_finite() {
                    return Err(OutlinePreparationError::Numeric {
                        phase: OutlineNumericPhase::Placement,
                        value: f64::from(value),
                    }
                    .into());
                }
            }
            let point = kurbo::Point::new(f64::from(placed_x), f64::from(placed_y));
            placed = Some(placed.map_or(kurbo::Rect::from_points(point, point), |r| {
                r.union_pt(point)
            }));
        }
        let placed = placed.expect("four corners are always visited");
        if determinant != 0.0 && (placed.width() == 0.0 || placed.height() == 0.0) {
            return Err(OutlinePreparationError::Numeric {
                phase: OutlineNumericPhase::Placement,
                value: placed.area(),
            }
            .into());
        }
    }
    Ok(())
}
