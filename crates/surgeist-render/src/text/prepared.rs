//! Exact-instance, headless glyph admission. Retained paths, paint operations
//! and decoded images are the draw source; execution never reselects a font.

use crate::accounting::{
    Account, AccountedVec, RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderResource,
    RenderWork,
};
use std::{mem::size_of, sync::Arc};

use skrifa::{
    GlyphId, Tag,
    instance::{LocationRef, NormalizedCoord, Size},
    outline::{DrawSettings, Hinting, OutlineGlyphFormat, OutlinePen, pen::PathStyle},
    raw::TableProvider,
};

use crate::{Error, FontData, PathElement, Point, Rect, Result, TextGlyph, TextRunBounds};

mod bitmap;
mod color;
mod glyf;

/// One exact provider-normalized 2.14 coordinate, in the closed interval [-1, 1].
/// Axis order is the actual face's fvar order, not a user-axis setting order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalizedCoordinate(i16);

impl NormalizedCoordinate {
    /// Validates signed F2Dot14 bits without normalization or rounding.
    pub fn try_from_bits(bits: i16) -> Result<Self> {
        if !(-16384..=16384).contains(&bits) {
            return Err(GlyphPreparationError::CoordinateValue {
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
            return Err(GlyphPreparationError::CoordinateValue {
                value: f64::from(value),
            }
            .into());
        }
        let bits = value * 16384.0;
        if bits.fract() != 0.0 {
            return Err(GlyphPreparationError::CoordinateLoss { value }.into());
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

/// Concrete horizontal outline synthesis. Skew uses x += skew*y before scaling;
/// positive emboldening appends a local-x translated copy to the same path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphSynthesis {
    skew: f32,
    embolden: f32,
}

impl GlyphSynthesis {
    /// The real no-synthesis case.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            skew: 0.0,
            embolden: 0.0,
        }
    }

    /// Validates x += skew*y and a nonnegative scaled local-x copy displacement.
    /// The displacement is not a design-unit amount or an expansion radius.
    /// Text resolves permitted faux bold from actual size/36; this consumer
    /// executes the supplied canonical displacement without resolving it again.
    pub fn try_new(skew: f32, embolden: f32) -> Result<Self> {
        if !skew.is_finite() || !embolden.is_finite() || embolden < 0.0 {
            return Err(GlyphPreparationError::InvalidSynthesis.into());
        }
        Ok(Self { skew, embolden })
    }

    /// Returns the executable font-outline skew factor.
    #[must_use]
    pub const fn skew(self) -> f32 {
        self.skew
    }
    /// Returns the scaled local-x displacement used for the appended contour copy.
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
    synthesis: GlyphSynthesis,
}

impl TextInstance {
    /// Takes canonical shaping size and all actual normalized coordinates in fvar order.
    pub fn try_new(
        size: f32,
        coordinates: Vec<NormalizedCoordinate>,
        synthesis: GlyphSynthesis,
    ) -> Result<Self> {
        if !size.is_finite() || size <= 0.0 {
            return Err(GlyphPreparationError::Numeric {
                phase: GlyphNumericPhase::Size,
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
    pub const fn synthesis(&self) -> GlyphSynthesis {
        self.synthesis
    }
}

/// Finite per-invocation bounds. Retained bytes include the font backing, glyphs,
/// coordinates, retained paints/pixels, combined path capacity/reallocation overlap and fixed result
/// backing; scratch is separate.
/// These are storage bounds, not a provider CPU-time guarantee.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphRunLimits {
    glyphs: usize,
    path_elements: usize,
    scratch_bytes: usize,
    retained_bytes: usize,
}

impl GlyphRunLimits {
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

impl GlyphRunLimits {
    pub(crate) const fn capacities(self) -> [usize; 4] {
        [
            self.glyphs,
            self.path_elements,
            self.scratch_bytes,
            self.retained_bytes,
        ]
    }
}

/// The resource for a finite admission or allocation refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlyphResource {
    /// Original glyph facts.
    Glyphs,
    /// Combined run-wide path elements.
    PathElements,
    /// Live qualifier buffers, provider draw memory and coordinate conversion.
    ScratchBytes,
    /// Actual retained immutable backing.
    RetainedBytes,
}

/// A numeric boundary in exact outline admission or placement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlyphNumericPhase {
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
    /// Positive local-x synthesis displacement was absorbed by point conversion.
    Synthesis,
    /// Selected bitmap source-color calibration or channel arithmetic.
    BitmapColor,
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
pub enum GlyphCapability {
    /// TrueType composite processing outside the qualified static XY domain.
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
pub enum GlyphPreparationError {
    /// An actual selected outline capability has not been qualified on this entry.
    UnsupportedOutline {
        /// Required capability.
        capability: GlyphCapability,
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
    /// A selected COLRv1 paint node is outside the supported finite graph.
    UnsupportedColorPaint {
        /// Actual OpenType paint format.
        format: u8,
    },
    /// Color/bitmap currently requires default coordinates and no synthesis.
    UnsupportedImageInstance,
    /// The selected PNG encoding is outside the supported decoder profile.
    UnsupportedPng,
    /// Selected bitmap drawing is outside this entry's supported domain.
    UnsupportedBitmap,
    /// Nonfinite or negative synthesis input.
    InvalidSynthesis,
    /// Canonical numeric representation cannot preserve this input/output.
    Numeric {
        /// Failing boundary.
        phase: GlyphNumericPhase,
        /// Rejected value.
        value: f64,
    },
    /// The provider refused the selected outline, or its decoder cannot faithfully
    /// consume this structurally valid glyph encoding.
    Provider {
        /// Selected glyph.
        glyph: u32,
    },
    /// Required storage exceeds a caller limit.
    Limit {
        /// Bounded resource.
        resource: GlyphResource,
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
        resource: GlyphResource,
    },
}

/// Owner-issued immutable run whose retained canonical representation is its GPU draw source.
/// Clones share one preparation identity and actual font/instance/geometry backing.
#[derive(Clone, Debug)]
pub struct PreparedGlyphRun(Arc<PreparedGlyphRunData>);

#[derive(Debug)]
struct PreparedGlyphRunData {
    font: FontData,
    instance: TextInstance,
    glyphs: Vec<TextGlyph>,
    path: kurbo::BezPath,
    bounds: TextRunBounds,
    paints: Vec<GlyphPaint>,
    // BezPath exposes elements as a slice; retain its producer-observed actual
    // capacity without estimating it from the later element count.
    path_capacity: usize,
}

/// Executed in order. Paths borrow the one canonical element allocation.
#[derive(Debug)]
pub(crate) enum GlyphPaint {
    Fill {
        elements: std::ops::Range<usize>,
        color: Option<crate::Color>,
        clip: Option<Rect>,
    },
    Bitmap {
        image: crate::Image,
        rect: Rect,
        brush_transform: kurbo::Affine,
    },
}

impl PartialEq for PreparedGlyphRun {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl PreparedGlyphRun {
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
    /// Reads retained vector geometry without exposing a provider/backend object.
    /// Bitmap operations have no vector elements. This aggregate is metadata;
    /// scene drawing consumes the ordered admitted paints and images.
    pub fn path_elements(&self) -> impl Iterator<Item = PathElement> + '_ {
        fn point(p: kurbo::Point) -> Point {
            // The immutable path contains only successfully admitted canonical points.
            Point::new(p.x, p.y)
        }
        self.0.path.elements().iter().map(|element| match *element {
            kurbo::PathEl::MoveTo(p) => PathElement::MoveTo(point(p)),
            kurbo::PathEl::LineTo(p) => PathElement::LineTo(point(p)),
            kurbo::PathEl::QuadTo(a, p) => PathElement::QuadTo(point(a), point(p)),
            kurbo::PathEl::CurveTo(a, b, p) => PathElement::CubicTo(point(a), point(b), point(p)),
            kurbo::PathEl::ClosePath => PathElement::Close,
        })
    }
    pub(crate) fn paints(&self) -> &[GlyphPaint] {
        &self.0.paints
    }
    pub(crate) fn path(&self) -> &kurbo::BezPath {
        &self.0.path
    }
    /// Immutable directly owned backing facts, excluding independently shared
    /// FontResource and bitmap Image payloads and opaque allocation headers.
    pub(crate) fn own_backing_bytes(&self) -> Result<usize> {
        [
            size_of::<PreparedGlyphRunData>(),
            crate::accounting::bytes::<TextGlyph>(self.0.glyphs.capacity())?,
            crate::accounting::bytes::<NormalizedCoordinate>(
                self.0.instance.coordinates.capacity(),
            )?,
            crate::accounting::bytes::<kurbo::PathEl>(self.0.path_capacity)?,
            crate::accounting::bytes::<GlyphPaint>(self.0.paints.capacity())?,
        ]
        .into_iter()
        .try_fold(0usize, |total, bytes| {
            total
                .checked_add(bytes)
                .ok_or_else(|| GlyphPreparationError::CapacityOverflow.into())
        })
    }
}

impl FontData {
    /// Admits exact-instance glyph representations and ink without a GPU.
    ///
    /// Unscaled HarfBuzz contour interpretation is followed by checked f64
    /// size/UPEM scaling, synthesis, reflection and local placement into one f32
    /// path. Dense simple-glyph gvar tuples and static XY-translation composites
    /// are supported. Composite variation, nonidentity component transforms, point
    /// attachment, metric overrides, sparse IUP and CFF/CFF2/VARC refuse explicitly.
    ///
    /// Positive emboldening appends the original contours and a local-x copy
    /// into this one path for one NonZero fill. It leaves glyph facts unchanged.
    /// All glyphs succeed before an immutable result is published. COLRv0 and
    /// COLRv1 PaintGlyph→PaintSolid with one optional translation and static
    /// root clip are supported at default coordinates without synthesis.
    /// sbix noninterlaced 8-bit RGBA PNG with IHDR, contiguous IDAT, IEND and
    /// supported sRGB/gAMA/cHRM color chunks uses retained strike/bearing
    /// metadata and decoded pixels under that same instance restriction. Other selected
    /// operations/formats refuse explicitly, without representation fallback.
    /// Palette0 is the default; CurrentColor uses the caller's paint.
    /// Consecutive ordinary outlines share one NonZero fill. Intrinsic paints
    /// retain their order and prevent foreground merging across them.
    /// The returned geometry retains this face independently of the input lifetime.
    pub fn prepare_glyph_run(
        &self,
        instance: &TextInstance,
        glyphs: &[TextGlyph],
        limits: GlyphRunLimits,
        accounting: &mut RenderAccountingContext,
    ) -> Result<PreparedGlyphRun> {
        accounting.prepare(
            RenderAttemptKind::GlyphRun,
            limits,
            #[cfg(feature = "render-text")]
            None,
            |account| {
                account.work(RenderWork::GlyphInputs, glyphs.len())?;
                account.work(RenderWork::CoordinateInputs, instance.coordinates.len())?;
                let font_bytes = self.admitted_face().resource().retained_bytes();
                self.prepare_glyph_run_inner(instance, glyphs, account, font_bytes)
            },
        )
    }

    pub(crate) fn prepare_glyph_run_inner(
        &self,
        instance: &TextInstance,
        glyphs: &[TextGlyph],
        account: &Account,
        font_bytes: usize,
    ) -> Result<PreparedGlyphRun> {
        let account = account.at(RenderPhase::GlyphPrepare);
        account.phase(RenderPhase::GlyphPrepare, || {
            self.prepare_glyph_run_body(
                instance,
                glyphs,
                &account,
                font_bytes,
                #[cfg(feature = "render-text")]
                None,
            )
        })
    }
    /// Selects once and records temporary per-input-glyph envelopes alongside
    /// the unchanged canonical run. The adapter owns shared-font admission.
    #[cfg(feature = "render-text")]
    pub(crate) fn prepare_glyph_run_with_bounds(
        &self,
        instance: &TextInstance,
        glyphs: &[TextGlyph],
        account: &Account,
    ) -> Result<(PreparedGlyphRun, AccountedVec<TextRunBounds>)> {
        let account = account.at(RenderPhase::GlyphPrepare);
        account.phase(RenderPhase::GlyphPrepare, || {
            let mut recorded_bounds = account.temporary(glyphs.len())?;
            let run = self.prepare_glyph_run_body(
                instance,
                glyphs,
                &account,
                0,
                Some(&mut recorded_bounds),
            )?;
            Ok((run, recorded_bounds))
        })
    }

    fn prepare_glyph_run_body(
        &self,
        instance: &TextInstance,
        glyphs: &[TextGlyph],
        account: &Account,
        font_bytes: usize,
        #[cfg(feature = "render-text")] mut recorded_bounds: Option<
            &mut AccountedVec<TextRunBounds>,
        >,
    ) -> Result<PreparedGlyphRun> {
        let font = skrifa::FontRef::from_index(self.bytes(), self.index())
            .map_err(|_| Error::from(GlyphPreparationError::InvalidFont))?;
        // table_data returns None for an out-of-bounds present record as well as
        // absence. Neither that nor duplicate records may hide variation/color
        // metadata and accidentally select a static or outline fallback.
        for (index, record) in font.table_directory().table_records().iter().enumerate() {
            account.work(RenderWork::FontRecordVisits, 1)?;
            let tag = record.tag();
            let mut duplicate = false;
            for prior in &font.table_directory().table_records()[..index] {
                account.work(RenderWork::FontRecordVisits, 1)?;
                if prior.tag() == tag {
                    duplicate = true;
                    break;
                }
            }
            if duplicate || font.table_data(tag).is_none() {
                return Err(match tag.to_be_bytes() {
                    [b'f', b'v', b'a', b'r'] => GlyphPreparationError::InvalidVariation {
                        table: VariationTable::Fvar,
                    },
                    [b'g', b'v', b'a', b'r'] => GlyphPreparationError::InvalidVariation {
                        table: VariationTable::Gvar,
                    },
                    _ => GlyphPreparationError::InvalidFont,
                }
                .into());
            }
        }
        let units = font
            .head()
            .map_err(|_| Error::from(GlyphPreparationError::InvalidFont))?
            .units_per_em();
        if !(16..=16384).contains(&units) {
            return Err(GlyphPreparationError::InvalidFont.into());
        }
        let glyph_count = u32::from(
            font.maxp()
                .map_err(|_| Error::from(GlyphPreparationError::InvalidFont))?
                .num_glyphs(),
        );
        account.phase(RenderPhase::Qualify, || {
            validate_variations(&font, instance, glyphs, glyph_count, account)
        })?;
        // This explicit owned payload excludes opaque Arc/allocator headers.
        // Font supplies its own independently admitted retention unit.
        let fixed = size_of::<PreparedGlyphRunData>();
        let initial = font_bytes
            .checked_add(fixed)
            .and_then(|n| n.checked_add(glyphs.len().checked_mul(size_of::<TextGlyph>())?))
            .and_then(|n| {
                n.checked_add(
                    instance
                        .coordinates
                        .len()
                        .checked_mul(size_of::<NormalizedCoordinate>())?,
                )
            })
            .ok_or(GlyphPreparationError::CapacityOverflow)?;
        account.preview(RenderResource::RetainedCpuBytes, initial)?;
        account.retain(
            font_bytes
                .checked_add(fixed)
                .ok_or(GlyphPreparationError::CapacityOverflow)?,
        )?;
        let mut coordinates = account.temporary(instance.coordinates.len())?;
        account.phase(RenderPhase::Coordinates, || {
            for c in &instance.coordinates {
                account
                    .at(RenderPhase::Coordinates)
                    .work(RenderWork::CoordinateVisits, 1)?;
                coordinates.push(NormalizedCoord::from_bits(c.0))?;
            }
            Ok(())
        })?;
        let mut owned_coordinates = account.retained(instance.coordinates.len())?;
        owned_coordinates.extend_from_slice(&instance.coordinates)?;
        let mut owned_glyphs = account.retained(glyphs.len())?;
        owned_glyphs.extend_from_slice(glyphs)?;
        let mut pen = CollectingPen::new(account.clone())?;
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
        let context = PreparationContext {
            font: &font,
            instance,
            coordinates: &coordinates,
            units,
            outline_format: selected_format,
            outlines: outlines.as_ref(),
            account,
        };
        for glyph in glyphs {
            account.work(RenderWork::GlyphVisits, 1)?;
            #[cfg(feature = "render-text")]
            if recorded_bounds.is_some() {
                pen.glyph_bounds = Some(None);
            }
            if let Some(format) = crate::vello_engine::glyph::selected_color_format(
                &font,
                GlyphId::new(glyph.id()),
                self,
            )? {
                // Representation selection is a paint-order barrier even when
                // its admitted program contributes no visible operations.
                pen.coalesce_previous = false;
                account.phase(RenderPhase::Color, || {
                    color::prepare(&mut pen, &context, *glyph, format)
                })?;
            } else if let Some(bitmap) = crate::vello_engine::glyph::select_bitmap(
                &font,
                Size::new(instance.size),
                GlyphId::new(glyph.id()),
                self,
                Some(account),
            )
            .map_err(|error| {
                if error.accounting_diagnostic().is_some() {
                    error
                } else {
                    GlyphPreparationError::InvalidFont.into()
                }
            })? {
                pen.coalesce_previous = false;
                account.phase(RenderPhase::Bitmap, || {
                    bitmap::prepare(&mut pen, &context, *glyph, bitmap)
                })?;
            } else {
                let start = pen.elements.len();
                draw_outline(&mut pen, &context, *glyph, None)?;
                pen.paint_fill(start, None, None, true)?;
            }
            #[cfg(feature = "render-text")]
            if let Some(records) = recorded_bounds.as_mut() {
                records.push(admitted_bounds(pen.glyph_bounds.flatten(), account)?)?;
            }
        }
        let path_capacity = pen.elements.capacity();
        let (path, bounds, paints) = pen.finish()?;
        account.phase(RenderPhase::Output, || {
            Ok(PreparedGlyphRun(Arc::new(PreparedGlyphRunData {
                font: self.clone(),
                instance: TextInstance {
                    size: instance.size,
                    coordinates: owned_coordinates.into_preparation()?.into_retained(),
                    synthesis: instance.synthesis,
                },
                glyphs: owned_glyphs.into_preparation()?.into_retained(),
                path,
                bounds,
                paints,
                path_capacity,
            })))
        })
    }
}

struct PreparationContext<'a, 'font> {
    font: &'a skrifa::FontRef<'font>,
    instance: &'a TextInstance,
    coordinates: &'a [NormalizedCoord],
    units: u16,
    outline_format: Option<OutlineGlyphFormat>,
    outlines: Option<&'a skrifa::OutlineGlyphCollection<'font>>,
    account: &'a Account,
}

fn draw_outline(
    pen: &mut CollectingPen,
    context: &PreparationContext<'_, '_>,
    glyph: TextGlyph,
    clip: Option<kurbo::Rect>,
) -> Result<()> {
    context.account.phase(RenderPhase::Outline, || {
        draw_outline_body(pen, context, glyph, clip)
    })
}
fn draw_outline_body(
    pen: &mut CollectingPen,
    context: &PreparationContext<'_, '_>,
    glyph: TextGlyph,
    clip: Option<kurbo::Rect>,
) -> Result<()> {
    let PreparationContext {
        font,
        instance,
        coordinates,
        units,
        outline_format: selected_format,
        outlines,
        account,
    } = *context;
    let unsupported = match selected_format {
        Some(OutlineGlyphFormat::Cff) => Some(GlyphCapability::PostScript),
        Some(OutlineGlyphFormat::Cff2) => Some(GlyphCapability::PostScriptVariation),
        Some(OutlineGlyphFormat::Varc) => Some(GlyphCapability::VariableComposite),
        _ => None,
    };
    if let Some(capability) = unsupported {
        return Err(GlyphPreparationError::UnsupportedOutline { capability }.into());
    }
    let outlines = outlines.ok_or_else(|| Error::from(GlyphPreparationError::InvalidFont))?;
    if account
        .phase(RenderPhase::Qualify, || {
            glyf::qualify(font, GlyphId::new(glyph.id()), coordinates, account)
        })?
        .points
        == 0
    {
        return Ok(());
    }
    let outline = outlines
        .get(GlyphId::new(glyph.id()))
        .ok_or_else(|| Error::from(GlyphPreparationError::Provider { glyph: glyph.id() }))?;
    let scratch_len = outline.draw_memory_size(Hinting::None);
    let mut scratch = account.temporary(scratch_len)?;
    scratch.resize(scratch_len, 0u8)?;
    let copies = if instance.synthesis.embolden == 0.0 {
        1
    } else {
        2
    };
    for copy in 0..copies {
        let mut active = GlyphOutlinePen::new(
            pen,
            glyph,
            instance.synthesis.skew,
            f64::from(instance.size) / f64::from(units),
            if copy == 0 {
                0.0
            } else {
                instance.synthesis.embolden
            },
            clip,
        );
        outline
            .draw(
                DrawSettings::unhinted(Size::unscaled(), LocationRef::new(coordinates))
                    .with_path_style(PathStyle::HarfBuzz)
                    .with_memory(Some(&mut scratch)),
                &mut active,
            )
            .map_err(|_| Error::from(GlyphPreparationError::Provider { glyph: glyph.id() }))?;
        active.finish()?;
    }
    Ok(())
}

fn validate_variations(
    font: &skrifa::FontRef<'_>,
    instance: &TextInstance,
    glyphs: &[TextGlyph],
    glyph_count: u32,
    account: &Account,
) -> Result<()> {
    let invalid = |table| Error::from(GlyphPreparationError::InvalidVariation { table });
    let axes = if font.table_data(Tag::new(b"fvar")).is_some() {
        let fvar = font.fvar().map_err(|_| invalid(VariationTable::Fvar))?;
        let arrays = fvar
            .axis_instance_arrays()
            .map_err(|_| invalid(VariationTable::Fvar))?;
        if fvar.version().major != 1 || fvar.axis_size() != 20 {
            return Err(invalid(VariationTable::Fvar));
        }
        for axis in arrays.axes() {
            account
                .at(RenderPhase::Qualify)
                .work(RenderWork::CoordinateVisits, 1)?;
            if axis.min_value() > axis.default_value() || axis.default_value() > axis.max_value() {
                return Err(invalid(VariationTable::Fvar));
            }
        }
        for record in arrays.instances().iter() {
            account
                .at(RenderPhase::Qualify)
                .work(RenderWork::FontRecordVisits, 1)?;
            record.map_err(|_| invalid(VariationTable::Fvar))?;
        }
        usize::from(fvar.axis_count())
    } else {
        0
    };
    if instance.coordinates.len() != axes {
        return Err(GlyphPreparationError::CoordinateCount {
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
        account
            .at(RenderPhase::Qualify)
            .work(RenderWork::GlyphVisits, 1)?;
        if glyph.id() >= glyph_count {
            return Err(GlyphPreparationError::GlyphRange {
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
// Qualify simple/dense inputs and decoder errors before drawing. Composite
// closure reuses this same raw and decoder owner for each actual simple child.

fn qualify_simple_glyf(
    font: &skrifa::FontRef<'_>,
    glyph: GlyphId,
    coordinates: &[NormalizedCoord],
    account: &Account,
) -> Result<glyf::Summary> {
    use skrifa::raw::{
        tables::glyf::{Glyph, PointFlags},
        types::{Fixed, Point as RawPoint},
    };
    let invalid = || Error::from(GlyphPreparationError::InvalidFont);
    let invalid_variation = || {
        Error::from(GlyphPreparationError::InvalidVariation {
            table: VariationTable::Gvar,
        })
    };
    let head = font.head().map_err(|_| invalid())?;
    let loca = font
        .loca(Some(head.index_to_loc_format() == 1))
        .map_err(|_| invalid())?;
    let glyf = font.glyf().map_err(|_| invalid())?;
    let raw = glyf::raw_glyph(font, glyph, account)?;
    let (count, contours) = if raw.is_empty() {
        (0, 0)
    } else {
        glyf::simple_structure(raw, glyph, account)?
    };
    let selected = loca.get_glyf(glyph, &glyf).map_err(|_| invalid())?;
    match &selected {
        None => {}
        Some(Glyph::Simple(_)) => {}
        Some(Glyph::Composite(_)) => {
            return Err(GlyphPreparationError::UnsupportedOutline {
                capability: GlyphCapability::CompositeGlyph,
            }
            .into());
        }
    }
    let points = count
        .checked_add(4)
        .ok_or_else(|| Error::from(GlyphPreparationError::CapacityOverflow))?;
    let mut source = account.temporary(points)?;
    source.resize(points, RawPoint { x: 0i32, y: 0i32 })?;
    let mut flags = account.temporary(points)?;
    flags.resize(points, PointFlags::default())?;
    let mut deltas = account.temporary(points)?;
    deltas.resize(points, RawPoint { x: 0f32, y: 0f32 })?;
    if let Some(Glyph::Simple(simple)) = &selected {
        simple
            .read_points_fast(&mut source[..count], &mut flags[..count])
            .map_err(|_| {
                Error::from(GlyphPreparationError::Provider {
                    glyph: glyph.to_u32(),
                })
            })?;
        for p in &source[..count] {
            account
                .at(RenderPhase::Qualify)
                .work(RenderWork::OutlinePointVisits, 1)?;
            if p.x < i32::from(simple.x_min())
                || p.x > i32::from(simple.x_max())
                || p.y < i32::from(simple.y_min())
                || p.y > i32::from(simple.y_max())
            {
                return Err(invalid());
            }
        }
    }
    // The collection caches HVAR with .ok(). Validate its present header,
    // even though this entry consumes no horizontal metric result. A source-
    // proved empty glyph bypasses the provider's metric-only phantom draw.
    if font.table_data(Tag::new(b"HVAR")).is_some() {
        font.hvar().map_err(|_| {
            Error::from(GlyphPreparationError::InvalidVariation {
                table: VariationTable::Hvar,
            })
        })?;
    }
    let mut bounds: Option<glyf::Bounds> = None;
    for p in &source[..count] {
        account
            .at(RenderPhase::Qualify)
            .work(RenderWork::OutlinePointVisits, 1)?;
        let point = glyf::Bounds::point(p.x, p.y);
        if let Some(bounds) = &mut bounds {
            bounds.include(point);
        } else {
            bounds = Some(point);
        }
    }
    let qualified = if raw.is_empty() {
        glyf::Summary::default()
    } else {
        glyf::Summary::simple(count, contours, bounds)?
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
        account
            .at(RenderPhase::Qualify)
            .work(RenderWork::VariationTupleVisits, 1)?;
        tuples += 1;
        let peak = tuple.peak();
        if peak.len() != coordinates.len() {
            return Err(invalid_variation());
        }
        for i in 0..peak.len() {
            account
                .at(RenderPhase::Qualify)
                .work(RenderWork::CoordinateVisits, 1)?;
            if peak
                .get(i)
                .is_none_or(|p| !(-16384..=16384).contains(&p.to_bits()))
            {
                return Err(invalid_variation());
            }
        }
        for region in [tuple.intermediate_start(), tuple.intermediate_end()]
            .into_iter()
            .flatten()
        {
            if region.len() != coordinates.len() {
                return Err(invalid_variation());
            }
            for i in 0..region.len() {
                account
                    .at(RenderPhase::Qualify)
                    .work(RenderWork::CoordinateVisits, 1)?;
                if region
                    .get(i)
                    .is_none_or(|p| !(-16384..=16384).contains(&p.to_bits()))
                {
                    return Err(invalid_variation());
                }
            }
        }
        if let (Some(start), Some(end)) = (tuple.intermediate_start(), tuple.intermediate_end()) {
            // Every lane has the same checked axis count. Borrow together and
            // decode values only after their existing coordinate admission.
            for ((start, peak), end) in start.values().iter().zip(peak.values()).zip(end.values()) {
                account
                    .at(RenderPhase::Qualify)
                    .work(RenderWork::CoordinateVisits, 3)?;
                let start = start.get();
                let peak = peak.get();
                let end = end.get();
                if start > peak || peak > end {
                    return Err(invalid_variation());
                }
            }
        }
        if !tuple.has_deltas_for_all_points() {
            return Err(GlyphPreparationError::UnsupportedOutline {
                capability: GlyphCapability::SparseVariation,
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
            account
                .at(RenderPhase::Qualify)
                .work(RenderWork::OutlinePointVisits, 1)?;
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
            for p in &deltas {
                account
                    .at(RenderPhase::Qualify)
                    .work(RenderWork::OutlinePointVisits, 1)?;
                if !p.x.is_finite() || !p.y.is_finite() {
                    return Err(provider_point(f64::INFINITY));
                }
            }
        }
    }
    if tuples != expected {
        return Err(invalid_variation());
    }
    Ok(qualified)
}

fn provider_point(value: f64) -> Error {
    GlyphPreparationError::Numeric {
        phase: GlyphNumericPhase::ProviderPoint,
        value,
    }
    .into()
}

#[cfg(feature = "render-text")]
fn admitted_bounds(bounds: Option<kurbo::Rect>, account: &Account) -> Result<TextRunBounds> {
    match bounds {
        None => Ok(TextRunBounds::empty()),
        Some(r) => {
            account.work(RenderWork::CoordinateVisits, 4)?;
            let (x0, y0, x1, y1) = (r.x0, r.y0, r.x1, r.y1);
            let width = placement_action(account, || x1 - x0)?;
            let height = placement_action(account, || y1 - y0)?;
            TextRunBounds::try_ink(Rect::try_new(x0, y0, width, height)?)
        }
    }
}

struct CollectingPen {
    elements: AccountedVec<kurbo::PathEl>,
    account: Account,
    bounds: Option<kurbo::Rect>,
    paints: AccountedVec<GlyphPaint>,
    coalesce_previous: bool,
    #[cfg(feature = "render-text")]
    glyph_bounds: Option<Option<kurbo::Rect>>,
}

impl CollectingPen {
    fn include_bounds(
        &mut self,
        bounds: kurbo::Rect,
        #[cfg(feature = "render-text")] phase: RenderPhase,
    ) -> Result<()> {
        self.bounds = Some(
            self.bounds
                .map_or(bounds, |previous| previous.union(bounds)),
        );
        #[cfg(feature = "render-text")]
        if let Some(current) = &mut self.glyph_bounds {
            *current = Some(match *current {
                None => bounds,
                Some(previous) => {
                    self.account
                        .at(phase)
                        .work(RenderWork::CoordinateVisits, 4)?;
                    previous.union(bounds)
                }
            });
        }
        Ok(())
    }
    fn paint(&mut self, paint: GlyphPaint) -> Result<()> {
        self.account.work(RenderWork::PaintVisits, 1)?;
        self.coalesce_previous = false;
        self.paints.grow(1)?;
        self.paints.push(paint)
    }
    fn paint_fill(
        &mut self,
        start: usize,
        color: Option<crate::Color>,
        clip: Option<Rect>,
        coalesce: bool,
    ) -> Result<()> {
        let end = self.elements.len();
        if start == end {
            return Ok(());
        }
        if coalesce
            && self.coalesce_previous
            && let Some(previous) = self.paints.last_mut()
        {
            self.account.work(RenderWork::PaintVisits, 1)?;
            if let GlyphPaint::Fill {
                elements,
                color: None,
                clip: None,
            } = previous
                && elements.end == start
            {
                elements.end = end;
                return Ok(());
            }
        }
        self.paint(GlyphPaint::Fill {
            elements: start..end,
            color,
            clip,
        })?;
        self.coalesce_previous = coalesce;
        Ok(())
    }
    fn new(account: Account) -> Result<Self> {
        Ok(Self {
            elements: account.retained(0)?,
            bounds: None,
            paints: account.retained(0)?,
            account,
            coalesce_previous: false,
            #[cfg(feature = "render-text")]
            glyph_bounds: None,
        })
    }
    fn finish(self) -> Result<(kurbo::BezPath, TextRunBounds, Vec<GlyphPaint>)> {
        let bounds = match self.bounds {
            None => TextRunBounds::empty(),
            Some(r) => TextRunBounds::try_ink(Rect::try_new(r.x0, r.y0, r.width(), r.height())?)?,
        };
        Ok((
            kurbo::BezPath::from_vec(self.elements.into_preparation()?.into_retained()),
            bounds,
            self.paints.into_preparation()?.into_retained(),
        ))
    }
}

/// Bounds of every accepted provider point, including off-curve controls.
enum GlyphGeometry {
    Empty,
    PointOnlyBounds(kurbo::Rect),
    DrawingBounds(kurbo::Rect),
}

impl GlyphGeometry {
    fn move_to(&mut self, point: kurbo::Point) {
        *self = match *self {
            Self::Empty => Self::PointOnlyBounds(kurbo::Rect::from_points(point, point)),
            Self::PointOnlyBounds(bounds) => Self::PointOnlyBounds(bounds.union_pt(point)),
            Self::DrawingBounds(bounds) => Self::DrawingBounds(bounds.union_pt(point)),
        };
    }

    fn draw_to(&mut self, point: kurbo::Point) {
        *self = Self::DrawingBounds(match *self {
            Self::Empty => kurbo::Rect::from_points(point, point),
            Self::PointOnlyBounds(bounds) | Self::DrawingBounds(bounds) => bounds.union_pt(point),
        });
    }
}

/// One actual draw borrows retained run storage without replacing its context.
struct GlyphOutlinePen<'a> {
    collector: &'a mut CollectingPen,
    glyph: TextGlyph,
    skew: f32,
    scale: f64,
    copy_offset: f32,
    clip: Option<kurbo::Rect>,
    geometry: GlyphGeometry,
    failed: Option<Error>,
}

impl<'a> GlyphOutlinePen<'a> {
    fn new(
        collector: &'a mut CollectingPen,
        glyph: TextGlyph,
        skew: f32,
        scale: f64,
        copy_offset: f32,
        clip: Option<kurbo::Rect>,
    ) -> Self {
        Self {
            collector,
            glyph,
            skew,
            scale,
            copy_offset,
            clip,
            geometry: GlyphGeometry::Empty,
            failed: None,
        }
    }

    fn point(&self, x: f32, y: f32) -> Result<kurbo::Point> {
        self.collector
            .account
            .at(RenderPhase::Outline)
            .work(RenderWork::OutlinePointVisits, 1)?;
        let values = [
            f64::from(x) * self.scale
                + f64::from(self.skew) * f64::from(y) * self.scale
                + self.glyph.x(),
            -f64::from(y) * self.scale + self.glyph.y(),
        ];
        // Preserve the original scale/shear/placement evaluation. The copy
        // shift is in scaled local units and is added before one canonical
        // conversion, never by translating an already-rounded retained path.
        let original_x = values[0];
        let mut values = values;
        if self.copy_offset > 0.0 {
            values[0] += f64::from(self.copy_offset);
        }
        let mut canonical = [0.0; 2];
        for (index, value) in values.into_iter().enumerate() {
            let narrowed = value as f32;
            if !value.is_finite() || !narrowed.is_finite() || (value != 0.0 && narrowed == 0.0) {
                return Err(GlyphPreparationError::Numeric {
                    phase: GlyphNumericPhase::PathPoint,
                    value,
                }
                .into());
            }
            if index == 0 && self.copy_offset > 0.0 && narrowed == original_x as f32 {
                return Err(GlyphPreparationError::Numeric {
                    phase: GlyphNumericPhase::Synthesis,
                    value: f64::from(self.copy_offset),
                }
                .into());
            }
            canonical[index] = f64::from(narrowed);
        }
        Ok(kurbo::Point::new(canonical[0], canonical[1]))
    }

    fn emit(&mut self, element: impl FnOnce(&mut Self) -> Result<kurbo::PathEl>) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| {
            let element = element(self)?;
            self.collector
                .account
                .at(RenderPhase::Outline)
                .work(RenderWork::PathElementVisits, 1)?;
            self.collector.elements.grow(1)?;
            self.collector
                .account
                .at(RenderPhase::Outline)
                .work(RenderWork::PathElementsEmitted, 1)?;
            self.collector.elements.push(element)
        })();
        if let Err(error) = result {
            self.failed = Some(error);
        }
    }
    fn finish(self) -> Result<()> {
        if let Some(error) = self.failed {
            return Err(error);
        }
        if let GlyphGeometry::DrawingBounds(mut r) = self.geometry {
            if r.width() <= 0.0 || r.height() <= 0.0 {
                return Err(GlyphPreparationError::Numeric {
                    phase: GlyphNumericPhase::InkExtent,
                    value: r.area(),
                }
                .into());
            }
            if let Some(clip) = self.clip {
                r = r.intersect(clip);
            }
            if r.width() > 0.0 && r.height() > 0.0 {
                self.collector.include_bounds(
                    r,
                    #[cfg(feature = "render-text")]
                    RenderPhase::Outline,
                )?;
            }
        }
        Ok(())
    }
}

fn default_image_instance(instance: &TextInstance, account: &Account) -> Result<()> {
    for coordinate in &instance.coordinates {
        account.work(RenderWork::CoordinateVisits, 1)?;
        if coordinate.bits() != 0 {
            return Err(GlyphPreparationError::UnsupportedImageInstance.into());
        }
    }
    if instance.synthesis != GlyphSynthesis::none() {
        return Err(GlyphPreparationError::UnsupportedImageInstance.into());
    }
    Ok(())
}

fn canonical_point(value: f64) -> Result<f64> {
    let actual = value as f32;
    if !value.is_finite() || !actual.is_finite() || (value != 0.0 && actual == 0.0) {
        return Err(GlyphPreparationError::Numeric {
            phase: GlyphNumericPhase::PathPoint,
            value,
        }
        .into());
    }
    Ok(f64::from(actual))
}

impl OutlinePen for GlyphOutlinePen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.emit(|pen| {
            let point = pen.point(x, y)?;
            pen.geometry.move_to(point);
            Ok(kurbo::PathEl::MoveTo(point))
        });
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.emit(|pen| {
            let point = pen.point(x, y)?;
            pen.geometry.draw_to(point);
            Ok(kurbo::PathEl::LineTo(point))
        });
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.emit(|pen| {
            let control = pen.point(cx, cy)?;
            pen.geometry.draw_to(control);
            let point = pen.point(x, y)?;
            pen.geometry.draw_to(point);
            Ok(kurbo::PathEl::QuadTo(control, point))
        });
    }
    fn curve_to(&mut self, ax: f32, ay: f32, bx: f32, by: f32, x: f32, y: f32) {
        self.emit(|pen| {
            let first = pen.point(ax, ay)?;
            pen.geometry.draw_to(first);
            let second = pen.point(bx, by)?;
            pen.geometry.draw_to(second);
            let point = pen.point(x, y)?;
            pen.geometry.draw_to(point);
            Ok(kurbo::PathEl::CurveTo(first, second, point))
        });
    }
    fn close(&mut self) {
        self.emit(|_| Ok(kurbo::PathEl::ClosePath));
    }
}

/// Preparation keeps the intrinsic glyph-placement policy and error domain.
#[cfg(any(feature = "render-text", test))]
pub(crate) fn qualify_preparation_placement(
    transform: kurbo::Affine,
    outline: &PreparedGlyphRun,
    account: &Account,
) -> Result<()> {
    account.phase(RenderPhase::Qualify, || {
        if outline.paints().is_empty() {
            return Ok(());
        }
        qualify_preparation_bounds(transform, outline.bounds().ink_rect(), account)?;
        let mut control_box: Option<kurbo::Rect> = None;
        for element in outline.path().elements() {
            account.work(RenderWork::PathElementVisits, 1)?;
            let mut point = |point: kurbo::Point| -> Result<()> {
                account.work(RenderWork::CoordinateVisits, 2)?;
                let point = kurbo::Point::new(point.x, point.y);
                account.work(RenderWork::CoordinateVisits, 4)?;
                control_box = Some(match control_box {
                    Some(bounds) => bounds.union_pt(point),
                    None => kurbo::Rect::from_points(point, point),
                });
                Ok(())
            };
            match *element {
                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => point(p)?,
                kurbo::PathEl::QuadTo(a, b) => {
                    point(a)?;
                    point(b)?;
                }
                kurbo::PathEl::CurveTo(a, b, c) => {
                    point(a)?;
                    point(b)?;
                    point(c)?;
                }
                kurbo::PathEl::ClosePath => {}
            }
        }
        if let Some(bounds) = control_box {
            qualify_preparation_native_bounds(transform, Some(bounds), account)?;
        }
        for paint in outline.paints() {
            account.work(RenderWork::PaintVisits, 1)?;
            match paint {
                GlyphPaint::Fill {
                    clip: Some(clip), ..
                } => qualify_preparation_bounds(transform, Some(*clip), account)?,
                GlyphPaint::Bitmap {
                    image,
                    rect,
                    brush_transform,
                } => {
                    qualify_preparation_bounds(transform, Some(*rect), account)?;
                    let map = crate::vello_engine::geometry::affine_product(
                        transform,
                        *brush_transform,
                        account,
                    )?;
                    account.work(RenderWork::CoordinateVisits, 2)?;
                    let width = image.size().width();
                    let height = image.size().height();
                    qualify_preparation_native_bounds(
                        map,
                        Some(kurbo::Rect::new(0.0, 0.0, width, height)),
                        account,
                    )?;
                }
                _ => {}
            }
        }
        Ok(())
    })
}

/// Actual runtime receiving uses the shared native-geometry qualification.
pub(crate) fn qualify_receiving_placement(
    transform: kurbo::Affine,
    outline: &PreparedGlyphRun,
    account: &Account,
) -> Result<()> {
    account.phase(RenderPhase::Qualify, || {
        qualify_receiving_placement_inner(transform, outline, account)
    })
}

fn qualify_receiving_placement_inner(
    transform: kurbo::Affine,
    outline: &PreparedGlyphRun,
    account: &Account,
) -> Result<()> {
    if outline.paints().is_empty() {
        return Ok(());
    }
    let qualified = crate::vello_engine::geometry::qualify_transform(transform, account)?;
    if let Some(bounds) = outline.bounds().ink_rect() {
        qualify_receiving_bounds(&qualified, bounds, account)?;
    }
    // The actual complete immutable outlines subsequently pass through the
    // common checked scene path feed. A control-box replay is not that proof.
    for paint in outline.paints() {
        account.work(RenderWork::PaintVisits, 1)?;
        match paint {
            GlyphPaint::Fill {
                clip: Some(clip), ..
            } => qualify_receiving_bounds(&qualified, *clip, account)?,
            GlyphPaint::Bitmap {
                image,
                rect,
                brush_transform,
            } => {
                qualify_receiving_bounds(&qualified, *rect, account)?;
                let map = crate::vello_engine::geometry::affine_product(
                    transform,
                    *brush_transform,
                    account,
                )?;
                let map = crate::vello_engine::geometry::qualify_transform(map, account)?;
                account.work(RenderWork::CoordinateVisits, 2)?;
                let width = image.size().width();
                let height = image.size().height();
                crate::vello_engine::geometry::qualify_bounds(
                    &map,
                    kurbo::Rect::new(0.0, 0.0, width, height),
                    account,
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn qualify_receiving_bounds(
    qualified: &crate::vello_engine::geometry::QualifiedTransform,
    bounds: Rect,
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::CoordinateVisits, 4)?;
    let (x, y, width, height) = (bounds.x(), bounds.y(), bounds.width(), bounds.height());
    let x1 = placement_action(account, || x + width)?;
    let y1 = placement_action(account, || y + height)?;
    crate::vello_engine::geometry::qualify_bounds(
        qualified,
        kurbo::Rect::new(x, y, x1, y1),
        account,
    )
}

fn placement_action<T>(account: &Account, action: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(action())
}

pub(crate) fn qualify_preparation_bounds(
    transform: kurbo::Affine,
    bounds: Option<Rect>,
    account: &Account,
) -> Result<()> {
    qualify_preparation_native_bounds(
        transform,
        preparation_input_bounds(bounds, account)?,
        account,
    )
}

fn preparation_input_bounds(
    bounds: Option<Rect>,
    account: &Account,
) -> Result<Option<kurbo::Rect>> {
    let bounds = if let Some(bounds) = bounds {
        account.work(RenderWork::CoordinateVisits, 4)?;
        let (x, y, width, height) = (bounds.x(), bounds.y(), bounds.width(), bounds.height());
        let x1 = placement_action(account, || x + width)?;
        let y1 = placement_action(account, || y + height)?;
        Some(kurbo::Rect::new(x, y, x1, y1))
    } else {
        None
    };
    Ok(bounds)
}

fn qualify_preparation_native_bounds(
    transform: kurbo::Affine,
    bounds: Option<kurbo::Rect>,
    account: &Account,
) -> Result<()> {
    map_preparation_native_bounds(transform, bounds, account).map(drop)
}

#[cfg(feature = "render-text")]
pub(crate) fn map_preparation_bounds(
    transform: kurbo::Affine,
    bounds: TextRunBounds,
    account: &Account,
) -> Result<TextRunBounds> {
    // Empty carries no rectangle and therefore no native placement operands.
    // Whole-run qualification likewise skips an admitted run without paints.
    if bounds.ink_rect().is_none() {
        return Ok(TextRunBounds::empty());
    }
    let bounds = preparation_input_bounds(bounds.ink_rect(), account)?;
    admitted_bounds(
        map_preparation_native_bounds(transform, bounds, account)?,
        account,
    )
}

fn map_preparation_native_bounds(
    transform: kurbo::Affine,
    bounds: Option<kurbo::Rect>,
    account: &Account,
) -> Result<Option<kurbo::Rect>> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let coefficients = transform.as_coeffs();
    for value in coefficients {
        let narrowed = placement_action(account, || value as f32)?;
        if !placement_action(account, || value.is_finite())?
            || !placement_action(account, || narrowed.is_finite())?
            || (placement_action(account, || value != 0.0)?
                && placement_action(account, || narrowed == 0.0)?)
        {
            return Err(GlyphPreparationError::Numeric {
                phase: GlyphNumericPhase::Placement,
                value,
            }
            .into());
        }
    }
    let [a, b, c, d, _, _] = coefficients;
    let ad = placement_action(account, || a * d)?;
    let bc = placement_action(account, || b * c)?;
    let determinant = placement_action(account, || ad - bc)?;
    let mut narrowed = [0.0; 6];
    for (index, value) in coefficients.into_iter().enumerate() {
        let native = placement_action(account, || value as f32)?;
        narrowed[index] = placement_action(account, || f64::from(native))?;
    }
    let ad = placement_action(account, || narrowed[0] * narrowed[3])?;
    let bc = placement_action(account, || narrowed[1] * narrowed[2])?;
    let canonical_determinant = placement_action(account, || ad - bc)?;
    if placement_action(account, || determinant != 0.0)?
        && placement_action(account, || canonical_determinant == 0.0)?
    {
        return Err(GlyphPreparationError::Numeric {
            phase: GlyphNumericPhase::Placement,
            value: determinant,
        }
        .into());
    }
    if let Some(rect) = bounds {
        let mut native = [0.0; 6];
        for (index, value) in coefficients.into_iter().enumerate() {
            native[index] = placement_action(account, || value as f32)?;
        }
        let [a, b, c, d, e, f] = native;
        account.work(RenderWork::CoordinateVisits, 4)?;
        let (x, y, x1, y1) = (rect.x0, rect.y0, rect.x1, rect.y1);
        let width = placement_action(account, || x1 - x)?;
        let height = placement_action(account, || y1 - y)?;
        let map_corner = |x, y| -> Result<kurbo::Point> {
            let local_x = placement_action(account, || x as f32)?;
            let local_y = placement_action(account, || y as f32)?;
            let ax = placement_action(account, || a * local_x)?;
            let cy = placement_action(account, || c * local_y)?;
            let sum = placement_action(account, || ax + cy)?;
            let placed_x = placement_action(account, || sum + e)?;
            let bx = placement_action(account, || b * local_x)?;
            let dy = placement_action(account, || d * local_y)?;
            let sum = placement_action(account, || bx + dy)?;
            let placed_y = placement_action(account, || sum + f)?;
            for value in [placed_x, placed_y] {
                if !placement_action(account, || value.is_finite())? {
                    return Err(GlyphPreparationError::Numeric {
                        phase: GlyphNumericPhase::Placement,
                        value: placement_action(account, || f64::from(value))?,
                    }
                    .into());
                }
            }
            let x = placement_action(account, || f64::from(placed_x))?;
            let y = placement_action(account, || f64::from(placed_y))?;
            Ok(kurbo::Point::new(x, y))
        };
        let first = map_corner(x, y)?;
        // Rect::from_points evaluates min/max on both axes.
        account.work(RenderWork::CoordinateVisits, 4)?;
        let mut placed = kurbo::Rect::from_points(first, first);
        for (x, y) in [(x1, y), (x, y1), (x1, y1)] {
            let point = map_corner(x, y)?;
            account.work(RenderWork::CoordinateVisits, 4)?;
            placed = placed.union_pt(point);
        }
        if placement_action(account, || determinant != 0.0)?
            && placement_action(account, || width > 0.0)?
            && placement_action(account, || height > 0.0)?
        {
            let native_width = placement_action(account, || placed.x1 - placed.x0)?;
            let native_height = placement_action(account, || placed.y1 - placed.y0)?;
            if placement_action(account, || native_width == 0.0)?
                || placement_action(account, || native_height == 0.0)?
            {
                return Err(GlyphPreparationError::Numeric {
                    phase: GlyphNumericPhase::Placement,
                    value: placement_action(account, || native_width * native_height)?,
                }
                .into());
            }
        }
        return Ok(Some(placed));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn callback_contour(pen: &mut impl OutlinePen) {
        pen.move_to(1.0, 2.0);
        pen.line_to(4.0, 6.0);
        pen.quad_to(6.0, 8.0, 8.0, 10.0);
        pen.curve_to(10.0, 12.0, 12.0, 14.0, 14.0, 16.0);
        pen.close();
    }

    #[test]
    fn collector_callbacks_preserve_empty_point_only_and_drawing_bounds() {
        let mut context = RenderAccountingContext::default();
        context
            .run(
                RenderAttemptKind::GlyphRun,
                RenderPhase::Outline,
                |account| {
                    let mut pen = CollectingPen::new(account.clone())?;
                    let glyph = TextGlyph::try_new(58, 20.0, 40.0, 9.0)?;
                    GlyphOutlinePen::new(&mut pen, glyph, 0.5, 2.0, 3.0, None).finish()?;
                    assert_eq!(pen.bounds, None);
                    let mut active = GlyphOutlinePen::new(&mut pen, glyph, 0.5, 2.0, 3.0, None);
                    active.move_to(1.0, 2.0);
                    active.move_to(4.0, 6.0);
                    active.close();
                    active.finish()?;
                    // Multiple Move callbacks span a positive box without ink.
                    assert_eq!(pen.bounds, None);
                    assert_eq!(pen.elements.len(), 3);
                    assert_eq!(
                        pen.elements[0],
                        kurbo::PathEl::MoveTo(kurbo::Point::new(27.0, 36.0))
                    );
                    let mut active = GlyphOutlinePen::new(
                        &mut pen,
                        glyph,
                        0.5,
                        2.0,
                        3.0,
                        Some(kurbo::Rect::new(30.0, 10.0, 60.0, 30.0)),
                    );
                    callback_contour(&mut active);
                    active.finish()?;
                    // x = 2*x + y + 20 + 3, y = 40 - 2*y. The control
                    // points monotonically span (27,36)..(67,8), then clip.
                    assert_eq!(pen.bounds, Some(kurbo::Rect::new(30.0, 10.0, 60.0, 30.0)));
                    assert_eq!(pen.elements.len(), 8);
                    let mut active = GlyphOutlinePen::new(&mut pen, glyph, 0.0, 1.0, 0.0, None);
                    active.move_to(100.0, 100.0);
                    active.close();
                    active.finish()?;
                    assert_eq!(pen.bounds, Some(kurbo::Rect::new(30.0, 10.0, 60.0, 30.0)));
                    Ok(())
                },
            )
            .unwrap();
    }

    #[test]
    fn active_pen_latches_numeric_failures_without_emitting_substitute_points() {
        for (placement, scale, offset, x, y, phase, expected) in [
            (
                0.0,
                1.0,
                0.0,
                f32::INFINITY,
                0.0,
                GlyphNumericPhase::PathPoint,
                f64::INFINITY,
            ),
            (
                0.0,
                2.0_f64.powi(-149),
                0.0,
                0.5,
                0.0,
                GlyphNumericPhase::PathPoint,
                2.0_f64.powi(-150),
            ),
            (
                0.0,
                2.0,
                0.0,
                0.0,
                f32::MAX,
                GlyphNumericPhase::PathPoint,
                -f64::from(f32::MAX) * 2.0,
            ),
            (
                2.0_f64.powi(40),
                2.0,
                1.0,
                1.0,
                f32::MAX,
                GlyphNumericPhase::Synthesis,
                1.0,
            ),
        ] {
            let mut context = RenderAccountingContext::default();
            let error = context
                .run(
                    RenderAttemptKind::GlyphRun,
                    RenderPhase::Outline,
                    |account| {
                        let mut collector = CollectingPen::new(account.clone())?;
                        let mut active = GlyphOutlinePen::new(
                            &mut collector,
                            TextGlyph::try_new(58, placement, 0.0, 9.0)?,
                            0.0,
                            scale,
                            offset,
                            None,
                        );
                        active.quad_to(x, y, 1.0, 1.0);
                        callback_contour(&mut active);
                        assert!(active.collector.elements.is_empty());
                        active.finish()
                    },
                )
                .unwrap_err();
            match error.glyph_preparation_diagnostic() {
                Some(GlyphPreparationError::Numeric {
                    phase: actual,
                    value,
                }) => {
                    assert_eq!(*actual, phase);
                    assert_eq!(*value, expected);
                }
                other => panic!("expected the first numeric callback failure, got {other:?}"),
            }
            let report = context.last_attempt().unwrap();
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::OutlinePointVisits), 1);
            assert_eq!(usage.work(RenderWork::PathElementVisits), 0);
            assert_eq!(usage.work(RenderWork::PathElementsEmitted), 0);
            assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        }
    }

    #[test]
    fn callback_refusals_keep_exact_prefix_and_suppress_later_callbacks() {
        use crate::{RenderAccountingOptions, RenderLimits, RenderPhaseFilter, RenderProfile};
        // Move + Line consume two points/elements. Quad first consumes both
        // points, then visits/grows/emits its one element. These independent
        // prefixes distinguish the three admission boundaries.
        for (quantity, points, visits, emitted) in [
            (RenderWork::OutlinePointVisits, 2, 2, 2),
            (RenderWork::PathElementVisits, 4, 2, 2),
            (RenderWork::PathElementsEmitted, 4, 3, 2),
        ] {
            let mut previous = None;
            for profile in [
                RenderProfile::Off,
                RenderProfile::Summary { timing: false },
                RenderProfile::Detailed {
                    timing: false,
                    event_limit: 1,
                    filter: RenderPhaseFilter::All,
                },
            ] {
                let mut context = RenderAccountingContext::new(
                    RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_limits(RenderLimits::new().with_work_limit(quantity, 2).unwrap()),
                );
                let error = context
                    .run(
                        RenderAttemptKind::GlyphRun,
                        RenderPhase::Outline,
                        |account| {
                            let mut pen = CollectingPen::new(account.clone())?;
                            let mut active = GlyphOutlinePen::new(
                                &mut pen,
                                TextGlyph::try_new(58, 0.0, 0.0, 9.0)?,
                                0.0,
                                1.0,
                                0.0,
                                None,
                            );
                            callback_contour(&mut active);
                            active.move_to(f32::NAN, f32::INFINITY);
                            active.close();
                            assert_eq!(active.collector.elements.len(), 2);
                            active.finish()
                        },
                    )
                    .unwrap_err();
                assert!(error.accounting_diagnostic().is_some());
                let standalone = context.last_attempt().unwrap();
                assert_eq!(standalone.cleanup_fault(), None);
                let report = standalone.attempt().unwrap();
                let refusal = report.refusal().unwrap();
                assert_eq!(refusal.quantity(), crate::RenderQuantity::Work(quantity));
                assert_eq!(refusal.current(), 2);
                assert_eq!(refusal.amount(), 1);
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                let usage = report.usage();
                assert_eq!(usage.work(RenderWork::OutlinePointVisits), points);
                assert_eq!(usage.work(RenderWork::PathElementVisits), visits);
                assert_eq!(usage.work(RenderWork::PathElementsEmitted), emitted);
                assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
                assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
                assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
                if let Some(previous) = previous {
                    assert_eq!(usage, previous);
                }
                previous = Some(usage);
            }
        }
    }

    #[test]
    fn the_same_collapsed_placement_keeps_preparation_and_receiving_error_domains() {
        use skrifa::MetadataProvider;
        let bytes = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
        let provider = skrifa::FontRef::from_index(bytes, 0).unwrap();
        let glyph = provider.charmap().map('A').unwrap().to_u32();
        let font =
            FontData::try_from_bytes(bytes.to_vec(), 0, surgeist_test::fixtures::FONT_LIMITS)
                .unwrap();
        let instance = TextInstance::try_new(16.0, vec![], GlyphSynthesis::none()).unwrap();
        let glyphs = [TextGlyph::try_new(glyph, 0.0, 16.0, 16.0).unwrap()];
        let outline = font
            .prepare_glyph_run(
                &instance,
                &glyphs,
                GlyphRunLimits::new(1, 32, 1 << 20, 1 << 20),
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
        let placement = kurbo::Affine::translate((2.0_f64.powi(40), 2.0_f64.powi(40)));
        let mut preparation = RenderAccountingContext::default();
        let error = preparation
            .run(
                RenderAttemptKind::GlyphRun,
                RenderPhase::Qualify,
                |account| qualify_preparation_placement(placement, &outline, account),
            )
            .unwrap_err();
        assert!(matches!(
            error.glyph_preparation_diagnostic(),
            Some(GlyphPreparationError::Numeric {
                phase: GlyphNumericPhase::Placement,
                value: 0.0
            })
        ));
        assert!(error.geometry_realization_diagnostic().is_none());
        assert_eq!(
            preparation
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        let mut receiving = RenderAccountingContext::default();
        let error = receiving
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                qualify_receiving_placement(placement, &outline, account)
            })
            .unwrap_err();
        assert!(error.geometry_realization_diagnostic().is_some());
        assert!(error.glyph_preparation_diagnostic().is_none());
        assert_eq!(
            receiving
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        preparation
            .run(
                RenderAttemptKind::GlyphRun,
                RenderPhase::Qualify,
                |account| qualify_preparation_placement(kurbo::Affine::IDENTITY, &outline, account),
            )
            .unwrap();
        receiving
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                qualify_receiving_placement(kurbo::Affine::IDENTITY, &outline, account)
            })
            .unwrap();
    }

    #[test]
    fn bitmap_intrinsic_identity_map_admits_its_exact_scalar_prefix() {
        // Six source captures; two nonzero coefficient sequences of five
        // actions and four zero sequences of four; source determinant3,
        // canonical recasts/widens12, canonical determinant3 and rank tests2.
        const ACTIONS: u64 = 6 + 2 * 5 + 4 * 4 + 3 + 12 + 3 + 2;
        assert_eq!(ACTIONS, 52);
        let mut limited = RenderAccountingContext::new(
            crate::RenderAccountingOptions::new().with_limits(
                crate::RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, ACTIONS - 1)
                    .unwrap(),
            ),
        );
        let error = limited
            .run(
                RenderAttemptKind::GlyphRun,
                RenderPhase::Qualify,
                |account| qualify_preparation_bounds(kurbo::Affine::IDENTITY, None, account),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = limited.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), ACTIONS - 1);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        let mut retry = RenderAccountingContext::default();
        retry
            .run(
                RenderAttemptKind::GlyphRun,
                RenderPhase::Qualify,
                |account| qualify_preparation_bounds(kurbo::Affine::IDENTITY, None, account),
            )
            .unwrap();
        assert_eq!(
            retry
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            ACTIONS
        );
    }

    #[test]
    fn empty_run_retains_only_owned_result_payload_and_font_admitted_backing() {
        let bytes = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
        let font =
            FontData::try_from_bytes(bytes.to_vec(), 0, surgeist_test::fixtures::FONT_LIMITS)
                .unwrap();
        let instance = TextInstance::try_new(20.0, vec![], GlyphSynthesis::none()).unwrap();
        // No glyph/path/paint/coordinate backing is allocated for this input.
        // The owned result payload and Font's admitted unit are the only custody.
        let required =
            font.admitted_face().resource().retained_bytes() + size_of::<PreparedGlyphRunData>();
        let mut account = RenderAccountingContext::default();
        let error = font
            .prepare_glyph_run(
                &instance,
                &[],
                GlyphRunLimits::new(0, 0, 0, required - 1),
                &mut account,
            )
            .unwrap_err();
        assert!(matches!(
            error.glyph_preparation_diagnostic(),
            Some(GlyphPreparationError::Limit {
                resource: GlyphResource::RetainedBytes,
                ..
            })
        ));
        let output = font
            .prepare_glyph_run(
                &instance,
                &[],
                GlyphRunLimits::new(0, 0, 0, required),
                &mut account,
            )
            .unwrap();
        assert_eq!(output.bounds(), TextRunBounds::empty());
        assert_eq!(
            output.own_backing_bytes().unwrap(),
            size_of::<PreparedGlyphRunData>()
        );
        assert_eq!(
            account
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::PreparedTransferredBytes),
            required as u64
        );
    }
}
