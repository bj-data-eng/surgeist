//! Same-effective-instance outline geometry. Raw glyf admission is derived from
//! the repository-owned Render qualifier; no Render dependency or retained path.

use super::EffectiveFontInstance;
use crate::{
    Error, ErrorCode, ErrorDetail, FontCatalogueIdentity, Rect, Result, TextNumericError,
    TextNumericPhase,
};
use skrifa::{
    GlyphId, Tag,
    instance::{LocationRef, NormalizedCoord, Size},
    outline::{DrawSettings, Hinting, OutlineGlyphFormat, OutlinePen, pen::PathStyle},
    raw::TableProvider,
};
use std::mem::size_of;

/// A font-outline representation not qualified by this geometry entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontOutlineCapability {
    /// Composite transforms, attachment, variation, or other unqualified flags.
    CompositeGlyph,
    /// CFF outlines.
    PostScript,
    /// CFF2 variable outlines.
    PostScriptVariation,
    /// Variable composite outlines.
    VariableComposite,
    /// Sparse point deltas requiring IUP interpolation.
    SparseVariation,
}

/// Precise refusal of an outline query, independent of missing painted ink.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontOutlineError {
    /// Malformed directory, outline, metrics, or variation source.
    InvalidFont,
    /// Requested glyph is outside the actual face's glyph range.
    GlyphRange {
        /// Actual number of glyphs.
        count: u32,
    },
    /// This face has no available outline representation.
    UnavailableOutline,
    /// A present representation lies outside the qualified domain.
    Unsupported {
        /// Actual unqualified capability.
        capability: FontOutlineCapability,
    },
    /// The provider cannot faithfully decode or draw this actual glyph.
    ProviderRefused {
        /// Actual failing glyph, possibly a composite child.
        glyph: u32,
    },
    /// An actual scratch extent cannot be represented.
    CapacityOverflow,
    /// Reserving actual temporary backing failed.
    AllocationRefused,
}

/// Geometry of the exact retained instance, in baseline-relative y-down units.
///
/// Bounds enclose the actual canonical outline's endpoints and control points;
/// they are conservative, not tight curve extrema. Shear and the positive local-x
/// bold copy are included; advances, glyph placement, color and bitmap paint are
/// not. No path is retained. The resource and coordinate loan remain attached.
#[derive(Debug)]
pub struct GlyphOutlineGeometry<'a> {
    instance: EffectiveFontInstance<'a>,
    glyph_id: u32,
    bounds: Option<Rect>,
}
impl<'a> GlyphOutlineGeometry<'a> {
    /// Exact resource, canonical coordinates, size and synthesis used.
    #[must_use]
    pub const fn instance(&self) -> &EffectiveFontInstance<'a> {
        &self.instance
    }
    /// Actual queried glyph.
    #[must_use]
    pub const fn glyph_id(&self) -> u32 {
        self.glyph_id
    }
    /// Conservative finite canonical outline bounds, or source-proved Empty.
    #[must_use]
    pub const fn bounds(&self) -> Option<Rect> {
        self.bounds
    }
}

enum Failure {
    Outline(FontOutlineError),
    Numeric {
        field: &'static str,
        reason: TextNumericError,
        value: f64,
    },
}
impl From<FontOutlineError> for Failure {
    fn from(reason: FontOutlineError) -> Self {
        Self::Outline(reason)
    }
}
type Checked<T> = std::result::Result<T, Failure>;
impl Failure {
    fn error(self, glyph: u32) -> Error {
        match self {
            Self::Outline(reason) => {
                Error::new(ErrorCode::FontOutlineRefused, "font outline refused")
                    .with_detail(ErrorDetail::FontOutline { glyph, reason })
            }
            Self::Numeric {
                field,
                reason,
                value,
            } => crate::error::text_numeric_error(
                field,
                TextNumericPhase::GlyphOutline,
                reason,
                value,
            ),
        }
    }
}
fn provider_point(value: f64) -> Failure {
    Failure::Numeric {
        field: "provider outline point",
        reason: TextNumericError::OutOfRange,
        value,
    }
}
fn reserved<T>(capacity: usize) -> Checked<Vec<T>> {
    capacity
        .checked_mul(size_of::<T>())
        .ok_or(FontOutlineError::CapacityOverflow)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| FontOutlineError::AllocationRefused)?;
    values
        .capacity()
        .checked_mul(size_of::<T>())
        .ok_or(FontOutlineError::CapacityOverflow)?;
    Ok(values)
}

impl<'a> EffectiveFontInstance<'a> {
    /// Reads actual same-instance glyph-outline geometry under the captured catalogue.
    ///
    /// Supports qualified simple/dense-gvar glyf and static translation-only
    /// composite closures. Other outline representations refuse explicitly. This
    /// operation neither selects color/bitmap paint nor changes paragraph layout.
    /// Failed decoding, missing glyphs and unavailable tables never become Empty.
    pub fn glyph_outline(
        &self,
        glyph_id: u32,
        expected_catalogue: &FontCatalogueIdentity,
    ) -> Result<GlyphOutlineGeometry<'a>> {
        if self.face().catalogue_identity() != expected_catalogue {
            return Err(Error::new(
                ErrorCode::FontCatalogueMismatch,
                "associated font catalogue mismatch",
            )
            .with_detail(ErrorDetail::FontCatalogueMismatch {
                expected: expected_catalogue.clone(),
                actual: self.face().catalogue_identity().clone(),
            }));
        }
        let bounds = self
            .outline_bounds(glyph_id)
            .map_err(|failure| failure.error(glyph_id))?;
        Ok(GlyphOutlineGeometry {
            instance: *self,
            glyph_id,
            bounds,
        })
    }
    fn outline_bounds(&self, glyph_id: u32) -> Checked<Option<Rect>> {
        let invalid = || Failure::from(FontOutlineError::InvalidFont);
        let font = skrifa::FontRef::from_index(self.face().bytes(), self.face().index())
            .map_err(|_| invalid())?;
        for (index, record) in font.table_directory().table_records().iter().enumerate() {
            if font.table_directory().table_records()[..index]
                .iter()
                .any(|prior| prior.tag() == record.tag())
                || font.table_data(record.tag()).is_none()
            {
                return Err(invalid());
            }
        }
        let head = font.head().map_err(|_| invalid())?;
        if head.version().major != 1
            || head.version().minor != 0
            || head.magic_number() != 0x5f0f3cf5
            || !matches!(head.index_to_loc_format(), 0 | 1)
            || head.glyph_data_format() != 0
        {
            return Err(invalid());
        }
        let units = head.units_per_em();
        if !(16..=16384).contains(&units) {
            return Err(invalid());
        }
        let count = u32::from(font.maxp().map_err(|_| invalid())?.num_glyphs());
        if glyph_id >= count {
            return Err(FontOutlineError::GlyphRange { count }.into());
        }
        let mut coordinates = reserved(self.axis_count())?;
        coordinates.extend(self.normalized_coords().map(NormalizedCoord::from_bits));
        let coordinate_bytes = coordinates
            .capacity()
            .checked_mul(size_of::<NormalizedCoord>())
            .ok_or(FontOutlineError::CapacityOverflow)?;
        if font.table_data(Tag::new(b"gvar")).is_some() {
            let gvar = font.gvar().map_err(|_| invalid())?;
            if coordinates.is_empty()
                || usize::from(gvar.axis_count()) != coordinates.len()
                || u32::from(gvar.glyph_count()) != count
                || gvar.version().major != 1
            {
                return Err(invalid());
            }
            gvar.shared_tuples().map_err(|_| invalid())?;
        }
        let selected = [
            (b"VARC", FontOutlineCapability::VariableComposite),
            (b"CFF2", FontOutlineCapability::PostScriptVariation),
            (b"CFF ", FontOutlineCapability::PostScript),
        ];
        if font.table_data(Tag::new(b"VARC")).is_some() {
            return Err(FontOutlineError::Unsupported {
                capability: selected[0].1,
            }
            .into());
        }
        if font.table_data(Tag::new(b"glyf")).is_none() {
            for (tag, capability) in selected.into_iter().skip(1) {
                if font.table_data(Tag::new(tag)).is_some() {
                    return Err(FontOutlineError::Unsupported { capability }.into());
                }
            }
            return Err(FontOutlineError::UnavailableOutline.into());
        }
        // The glyf provider defaults absent maxp fields and accepts arbitrary
        // versions. Require the actual TrueType header before its metadata walk,
        // including on a source-proved empty glyph.
        let maxp = font.maxp().map_err(|_| invalid())?;
        if maxp.version() != skrifa::raw::types::Version16Dot16::VERSION_1_0
            || font
                .table_data(Tag::new(b"maxp"))
                .ok_or_else(invalid)?
                .len()
                < 32
        {
            return Err(invalid());
        }
        let loca_len = (usize::try_from(count).map_err(|_| FontOutlineError::CapacityOverflow)?
            + 1)
        .checked_mul(if head.index_to_loc_format() == 1 {
            4
        } else {
            2
        })
        .ok_or(FontOutlineError::CapacityOverflow)?;
        if font
            .table_data(Tag::new(b"loca"))
            .ok_or_else(invalid)?
            .len()
            < loca_len
        {
            return Err(invalid());
        }
        let glyph = GlyphId::new(glyph_id);
        glyf::root_metrics(&font, glyph)?;
        let summary = glyf::qualify(&font, glyph, &coordinates, coordinate_bytes)?;
        if summary.points == 0 {
            return Ok(None);
        }
        let outlines = skrifa::OutlineGlyphCollection::with_format(&font, OutlineGlyphFormat::Glyf)
            .ok_or_else(invalid)?;
        let outline = outlines
            .get(glyph)
            .ok_or(FontOutlineError::ProviderRefused { glyph: glyph_id })?;
        let len = outline.draw_memory_size(Hinting::None);
        len.checked_add(coordinate_bytes)
            .ok_or(FontOutlineError::CapacityOverflow)?;
        let mut scratch = reserved(len)?;
        scratch
            .capacity()
            .checked_add(coordinate_bytes)
            .ok_or(FontOutlineError::CapacityOverflow)?;
        scratch.resize(len, 0u8);
        let mut pen = BoundsPen::new(
            f64::from(self.font_size()) / f64::from(units),
            self.x_shear(),
        );
        for copy in 0..if self.embolden() == 0.0 { 1 } else { 2 } {
            pen.copy_offset = if copy == 0 { 0.0 } else { self.embolden() };
            outline
                .draw(
                    DrawSettings::unhinted(Size::unscaled(), LocationRef::new(&coordinates))
                        .with_path_style(PathStyle::HarfBuzz)
                        .with_memory(Some(&mut scratch)),
                    &mut pen,
                )
                .map_err(|_| FontOutlineError::ProviderRefused { glyph: glyph_id })?;
            if let Some(failure) = pen.failed.take() {
                return Err(failure);
            }
        }
        if !pen.drawing {
            return Err(FontOutlineError::ProviderRefused { glyph: glyph_id }.into());
        }
        pen.finish().map(Some)
    }
}

struct BoundsPen {
    scale: f64,
    shear: f32,
    copy_offset: f32,
    bounds: Option<[f64; 4]>,
    drawing: bool,
    failed: Option<Failure>,
}
impl BoundsPen {
    fn new(scale: f64, shear: f32) -> Self {
        Self {
            scale,
            shear,
            copy_offset: 0.0,
            bounds: None,
            drawing: false,
            failed: None,
        }
    }
    fn point(&mut self, x: f32, y: f32) {
        if self.failed.is_some() {
            return;
        }
        let original_x =
            f64::from(x) * self.scale + f64::from(self.shear) * f64::from(y) * self.scale;
        let values = [
            original_x + f64::from(self.copy_offset),
            -f64::from(y) * self.scale,
        ];
        let mut canonical = [0.0; 2];
        for (index, value) in values.into_iter().enumerate() {
            let narrow = value as f32;
            let reason = if !value.is_finite() || !narrow.is_finite() {
                Some(TextNumericError::NonFinite)
            } else if (value != 0.0 && narrow == 0.0)
                || (index == 0 && self.copy_offset > 0.0 && narrow == original_x as f32)
            {
                Some(TextNumericError::NonZeroLost)
            } else {
                None
            };
            if let Some(reason) = reason {
                self.failed = Some(Failure::Numeric {
                    field: "canonical outline point",
                    reason,
                    value,
                });
                return;
            }
            canonical[index] = f64::from(narrow);
        }
        let [x, y] = canonical;
        match &mut self.bounds {
            Some(b) => {
                b[0] = b[0].min(x);
                b[1] = b[1].min(y);
                b[2] = b[2].max(x);
                b[3] = b[3].max(y);
            }
            None => self.bounds = Some([x, y, x, y]),
        }
    }
    fn finish(self) -> Checked<Rect> {
        let [x0, y0, x1, y1] = self.bounds.expect("drawing has actual points");
        let width = x1 - x0;
        let height = y1 - y0;
        for value in [width, height] {
            // Endpoints were qualified as canonical f32 points. Their logical
            // aggregate extent belongs to Rect's f64 domain, not another f32
            // provider argument or a renderer-specific ink-width limit.
            if !value.is_finite() || value < 0.0 {
                return Err(Failure::Numeric {
                    field: "outline extent",
                    reason: TextNumericError::OutOfRange,
                    value,
                });
            }
        }
        Ok(Rect::new(x0, y0, width, height))
    }
}
impl OutlinePen for BoundsPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.point(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.drawing = true;
        self.point(x, y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.drawing = true;
        self.point(cx, cy);
        self.point(x, y);
    }
    fn curve_to(&mut self, ax: f32, ay: f32, bx: f32, by: f32, x: f32, y: f32) {
        self.drawing = true;
        self.point(ax, ay);
        self.point(bx, by);
        self.point(x, y);
    }
    fn close(&mut self) {}
}

fn qualify_simple_glyf(
    font: &skrifa::FontRef<'_>,
    glyph: GlyphId,
    coordinates: &[NormalizedCoord],
    coordinate_bytes: usize,
) -> Checked<glyf::Summary> {
    use skrifa::raw::{
        tables::glyf::{Glyph, PointFlags},
        types::{Fixed, Point as RawPoint},
    };
    let invalid = || Failure::from(FontOutlineError::InvalidFont);
    let invalid_variation = || Failure::from(FontOutlineError::InvalidFont);
    let head = font.head().map_err(|_| invalid())?;
    let loca = font
        .loca(Some(head.index_to_loc_format() == 1))
        .map_err(|_| invalid())?;
    let glyf = font.glyf().map_err(|_| invalid())?;
    let raw = glyf::raw_glyph(font, glyph)?;
    let (count, contours) = if raw.is_empty() {
        (0, 0)
    } else {
        glyf::simple_structure(raw, glyph)?
    };
    let selected = loca.get_glyf(glyph, &glyf).map_err(|_| invalid())?;
    match &selected {
        None => {}
        Some(Glyph::Simple(_)) => {}
        Some(Glyph::Composite(_)) => {
            return Err(FontOutlineError::Unsupported {
                capability: FontOutlineCapability::CompositeGlyph,
            }
            .into());
        }
    }
    let points = count
        .checked_add(4)
        .ok_or_else(|| Failure::from(FontOutlineError::CapacityOverflow))?;
    let charge = |source_capacity: usize, flag_capacity: usize, delta_capacity: usize| {
        source_capacity
            .checked_mul(size_of::<RawPoint<i32>>())
            .and_then(|n| n.checked_add(flag_capacity.checked_mul(size_of::<PointFlags>())?))
            .and_then(|n| n.checked_add(delta_capacity.checked_mul(size_of::<RawPoint<f32>>())?))
            .and_then(|n| n.checked_add(coordinate_bytes))
            .ok_or_else(|| Failure::from(FontOutlineError::CapacityOverflow))
    };
    let _ = charge(points, points, points)?;
    let mut source = reserved(points)?;
    let _ = charge(source.capacity(), points, points)?;
    source.resize(points, RawPoint { x: 0i32, y: 0i32 });
    let mut flags = reserved(points)?;
    let _ = charge(source.capacity(), flags.capacity(), points)?;
    flags.resize(points, PointFlags::default());
    let mut deltas = reserved(points)?;
    deltas.resize(points, RawPoint { x: 0f32, y: 0f32 });
    let _ = charge(source.capacity(), flags.capacity(), deltas.capacity())?;
    if let Some(Glyph::Simple(simple)) = &selected {
        simple
            .read_points_fast(&mut source[..count], &mut flags[..count])
            .map_err(|_| {
                Failure::from(FontOutlineError::ProviderRefused {
                    glyph: glyph.to_u32(),
                })
            })?;
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
        font.hvar()
            .map_err(|_| Failure::from(FontOutlineError::InvalidFont))?;
    }
    let mut bounds: Option<glyf::Bounds> = None;
    for p in &source[..count] {
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
            return Err(FontOutlineError::Unsupported {
                capability: FontOutlineCapability::SparseVariation,
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

mod glyf {
    use super::*;
    fn invalid() -> Failure {
        FontOutlineError::InvalidFont.into()
    }
    fn overflow() -> Failure {
        FontOutlineError::CapacityOverflow.into()
    }
    fn unsupported() -> Failure {
        FontOutlineError::Unsupported {
            capability: FontOutlineCapability::CompositeGlyph,
        }
        .into()
    }
    fn add(a: usize, b: usize) -> Checked<usize> {
        a.checked_add(b).ok_or_else(overflow)
    }
    fn mul(a: usize, b: usize) -> Checked<usize> {
        a.checked_mul(b).ok_or_else(overflow)
    }
    fn word(data: &[u8], at: usize) -> Checked<u16> {
        let bytes = data.get(at..add(at, 2)?).ok_or_else(invalid)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }
    fn signed(data: &[u8], at: usize) -> Checked<i16> {
        Ok(word(data, at)? as i16)
    }

    pub(super) fn raw_glyph<'a>(font: &skrifa::FontRef<'a>, glyph: GlyphId) -> Checked<&'a [u8]> {
        let count = u32::from(font.maxp().map_err(|_| invalid())?.num_glyphs());
        if glyph.to_u32() >= count {
            // Public roots have their existing GlyphRange check. An invalid child
            // reference is malformed composite source, not another caller request.
            return Err(invalid());
        }
        let head = font.head().map_err(|_| invalid())?;
        let loca = font
            .loca(Some(head.index_to_loc_format() == 1))
            .map_err(|_| invalid())?;
        let at = usize::try_from(glyph.to_u32()).map_err(|_| overflow())?;
        let start =
            usize::try_from(loca.get_raw(at).ok_or_else(invalid)?).map_err(|_| overflow())?;
        let end = usize::try_from(loca.get_raw(add(at, 1)?).ok_or_else(invalid)?)
            .map_err(|_| overflow())?;
        font.table_data(Tag::new(b"glyf"))
            .ok_or_else(invalid)?
            .as_bytes()
            .get(start..end)
            .ok_or_else(invalid)
    }

    #[derive(Clone, Copy, Debug)]
    pub(super) struct Bounds {
        min_x: i32,
        min_y: i32,
        max_x: i32,
        max_y: i32,
    }
    impl Bounds {
        fn header(data: &[u8]) -> Checked<Self> {
            let result = Self {
                min_x: i32::from(signed(data, 2)?),
                min_y: i32::from(signed(data, 4)?),
                max_x: i32::from(signed(data, 6)?),
                max_y: i32::from(signed(data, 8)?),
            };
            if result.min_x > result.max_x || result.min_y > result.max_y {
                return Err(invalid());
            }
            Ok(result)
        }
        pub(super) fn point(x: i32, y: i32) -> Self {
            Self {
                min_x: x,
                min_y: y,
                max_x: x,
                max_y: y,
            }
        }
        pub(super) fn include(&mut self, other: Self) {
            self.min_x = self.min_x.min(other.min_x);
            self.min_y = self.min_y.min(other.min_y);
            self.max_x = self.max_x.max(other.max_x);
            self.max_y = self.max_y.max(other.max_y);
        }
        fn translated(self, x: i32, y: i32) -> Checked<Self> {
            Ok(Self {
                min_x: self.min_x.checked_add(x).ok_or_else(overflow)?,
                min_y: self.min_y.checked_add(y).ok_or_else(overflow)?,
                max_x: self.max_x.checked_add(x).ok_or_else(overflow)?,
                max_y: self.max_y.checked_add(y).ok_or_else(overflow)?,
            })
        }
        fn contains(self, other: Self) -> bool {
            self.min_x <= other.min_x
                && self.min_y <= other.min_y
                && self.max_x >= other.max_x
                && self.max_y >= other.max_y
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub(super) struct Summary {
        pub(super) points: usize,
        contours: usize,
        max_simple: usize,
        max_other: usize,
        component_stack: usize,
        depth: usize,
        pub(super) bounds: Option<Bounds>,
    }
    impl Summary {
        pub(super) fn simple(
            points: usize,
            contours: usize,
            bounds: Option<Bounds>,
        ) -> Checked<Self> {
            Ok(Self {
                points,
                contours,
                max_simple: add(points, 4)?,
                max_other: add(points, 4)?,
                bounds,
                ..Self::default()
            })
        }
        fn metadata(self) -> Checked<()> {
            let points = add(self.points, 4)?;
            if points > usize::from(u16::MAX) || self.depth > 32 {
                return Err(unsupported());
            }
            // Outline::required_buffer_size(None), static glyf: prove each otherwise
            // unchecked arithmetic operation before get()/draw_memory_size().
            let bytes = add(
                mul(points, size_of::<skrifa::raw::types::Point<i32>>())?,
                mul(self.max_other, size_of::<skrifa::raw::types::Point<i32>>())?,
            )?;
            let bytes = add(bytes, mul(self.contours, size_of::<u16>())?)?;
            let bytes = add(
                bytes,
                mul(points, size_of::<skrifa::raw::tables::glyf::PointFlags>())?,
            )?;
            let _ = add(bytes, std::mem::align_of::<i32>())?;
            Ok(())
        }
    }

    /// Validate exact source extents, not generated accessors' empty defaults.
    /// Residual bytes after coordinates are uninterpreted, not part of decoding.
    pub(super) fn simple_structure(data: &[u8], glyph: GlyphId) -> Checked<(usize, usize)> {
        let contours = usize::try_from(signed(data, 0)?).map_err(|_| invalid())?;
        Bounds::header(data)?;
        let ends_end = add(10, mul(contours, 2)?)?;
        let mut prior = None;
        for at in (10..ends_end).step_by(2) {
            let end = word(data, at)?;
            if prior.is_some_and(|last| end <= last) {
                return Err(invalid());
            }
            prior = Some(end);
        }
        let count = prior.map_or(0, |end| usize::from(end) + 1);
        let instructions = usize::from(word(data, ends_end)?);
        let flags_start = add(add(ends_end, 2)?, instructions)?;
        data.get(..flags_start).ok_or_else(invalid)?;
        let mut at = flags_start;
        let mut logical = 0;
        let mut x_bytes = 0;
        let mut y_bytes = 0;
        while logical < count {
            let flag = *data.get(at).ok_or_else(invalid)?;
            at = add(at, 1)?;
            let repeated = if flag & 8 != 0 {
                let repeat = usize::from(*data.get(at).ok_or_else(invalid)?);
                at = add(at, 1)?;
                add(repeat, 1)?
            } else {
                1
            };
            logical = add(logical, repeated)?;
            if logical > count {
                return Err(invalid());
            }
            let x = if flag & 2 != 0 {
                1
            } else if flag & 16 != 0 {
                0
            } else {
                2
            };
            let y = if flag & 4 != 0 {
                1
            } else if flag & 32 != 0 {
                0
            } else {
                2
            };
            x_bytes = add(x_bytes, mul(x, repeated)?)?;
            y_bytes = add(y_bytes, mul(y, repeated)?)?;
        }
        let end = add(add(at, x_bytes)?, y_bytes)?;
        data.get(..end).ok_or_else(invalid)?;
        // read_points_fast prefetches at most count bytes. Full prefix <= count
        // proves every encoded flag was prefetched, its clamp is inactive, and
        // coordinate decoding begins exactly after the complete flag prefix.
        if at - flags_start > count {
            return Err(FontOutlineError::ProviderRefused {
                glyph: glyph.to_u32(),
            }
            .into());
        }
        Ok((count, contours))
    }

    #[derive(Clone, Copy)]
    enum State {
        Active,
        Qualified(Summary),
    }
    struct Memo {
        glyph: GlyphId,
        state: State,
    }
    struct Frame<'a> {
        glyph: GlyphId,
        data: &'a [u8],
        at: usize,
        terminal: bool,
        instructions: bool,
        first_overlap: bool,
        components: usize,
        pending_offset: (i32, i32),
        summary: Summary,
    }
    impl<'a> Frame<'a> {
        fn new(glyph: GlyphId, data: &'a [u8]) -> Checked<Self> {
            if signed(data, 0)? >= 0 {
                return Err(invalid());
            }
            Bounds::header(data)?;
            Ok(Self {
                glyph,
                data,
                at: 10,
                terminal: false,
                instructions: false,
                first_overlap: false,
                components: 0,
                pending_offset: (0, 0),
                summary: Summary::default(),
            })
        }
        fn next(&mut self) -> Checked<GlyphId> {
            let flags = word(self.data, self.at)?;
            let glyph = GlyphId::new(u32::from(word(self.data, add(self.at, 2)?)?));
            self.at = add(self.at, 4)?;
            let (x, y) = if flags & 1 != 0 {
                let result = (
                    i32::from(signed(self.data, self.at)?),
                    i32::from(signed(self.data, add(self.at, 2)?)?),
                );
                self.at = add(self.at, 4)?;
                result
            } else {
                let bytes = self
                    .data
                    .get(self.at..add(self.at, 2)?)
                    .ok_or_else(invalid)?;
                self.at = add(self.at, 2)?;
                (i32::from(bytes[0] as i8), i32::from(bytes[1] as i8))
            };
            let transforms = flags & (8 | 64 | 128);
            if transforms.count_ones() > 1 || flags & 0x1800 == 0x1800 {
                return Err(invalid());
            }
            let identity = match transforms {
                8 => {
                    let v = signed(self.data, self.at)?;
                    self.at = add(self.at, 2)?;
                    v == 16384
                }
                64 => {
                    let a = signed(self.data, self.at)?;
                    let b = signed(self.data, add(self.at, 2)?)?;
                    self.at = add(self.at, 4)?;
                    a == 16384 && b == 16384
                }
                128 => {
                    let values = [
                        signed(self.data, self.at)?,
                        signed(self.data, add(self.at, 2)?)?,
                        signed(self.data, add(self.at, 4)?)?,
                        signed(self.data, add(self.at, 6)?)?,
                    ];
                    self.at = add(self.at, 8)?;
                    values == [16384, 0, 0, 16384]
                }
                _ => true,
            };
            let more = flags & 32 != 0;
            // All defined flags except obsolete/reserved bit 0x10 are explicit.
            if flags & !0x1fef != 0
                || flags & 2 == 0
                || flags & 512 != 0
                || !identity
                || (flags & 256 != 0 && more)
                || (flags & 1024 != 0 && self.components != 0 && !self.first_overlap)
            {
                return Err(unsupported());
            }
            if self.components == 0 {
                self.first_overlap = flags & 1024 != 0;
            }
            self.terminal = !more;
            self.instructions = flags & 256 != 0;
            self.components = add(self.components, 1)?;
            self.pending_offset = (x, y);
            Ok(glyph)
        }
        fn include(&mut self, child: Summary) -> Checked<()> {
            self.summary.points = add(self.summary.points, child.points)?;
            self.summary.contours = add(self.summary.contours, child.contours)?;
            self.summary.max_simple = self.summary.max_simple.max(child.max_simple);
            self.summary.max_other = self.summary.max_other.max(child.max_other);
            self.summary.component_stack = self.summary.component_stack.max(child.component_stack);
            // Draw calls Scaler::load for every child, including loca-equal None.
            // Only the metadata walk skips None; its zero point/storage summary
            // must not erase the actual draw recursion level.
            self.summary.depth = self.summary.depth.max(add(child.depth, 1)?);
            if let Some(bounds) = child.bounds {
                let bounds = bounds.translated(self.pending_offset.0, self.pending_offset.1)?;
                if let Some(existing) = &mut self.summary.bounds {
                    existing.include(bounds);
                } else {
                    self.summary.bounds = Some(bounds);
                }
            }
            self.summary.metadata()
        }
        fn finish(mut self) -> Checked<Summary> {
            if self.instructions {
                let len = usize::from(word(self.data, self.at)?);
                self.at = add(add(self.at, 2)?, len)?;
                self.data.get(..self.at).ok_or_else(invalid)?;
                self.summary.max_other = self.summary.max_other.max(add(self.summary.points, 4)?);
            }
            if self
                .data
                .get(self.at..)
                .ok_or_else(invalid)?
                .iter()
                .any(|&byte| byte != 0)
            {
                return Err(unsupported());
            }
            if let Some(bounds) = self.summary.bounds
                && !Bounds::header(self.data)?.contains(bounds)
            {
                return Err(invalid());
            }
            self.summary.component_stack =
                add(self.summary.component_stack, add(self.components, 4)?)?;
            self.summary.metadata()?;
            Ok(self.summary)
        }
    }

    fn vector_bytes<T>(values: &Vec<T>) -> Checked<usize> {
        mul(values.capacity(), size_of::<T>())
    }
    fn grow<T>(values: &mut Vec<T>, other: usize, base: usize) -> Checked<()> {
        if values.len() < values.capacity() {
            return Ok(());
        }
        let requested = add(values.len(), 1)?;
        let old = vector_bytes(values)?;
        // Check simultaneous old/new backing plus the other live qualifier buffers
        // before reserving. This query has no caller-supplied quota or CPU promise.
        let _ = add(
            add(base, other)?,
            add(old, mul(requested, size_of::<T>())?)?,
        )?;
        values
            .try_reserve_exact(1)
            .map_err(|_| Failure::from(FontOutlineError::AllocationRefused))?;
        let _ = add(add(base, other)?, add(old, vector_bytes(values)?)?)?;
        Ok(())
    }

    pub(super) fn root_metrics(font: &skrifa::FontRef<'_>, glyph: GlyphId) -> Checked<()> {
        let count = usize::from(font.maxp().map_err(|_| invalid())?.num_glyphs());
        let hhea = font.hhea().map_err(|_| invalid())?;
        if hhea.version().major != 1 || hhea.version().minor != 0 || hhea.metric_data_format() != 0
        {
            return Err(invalid());
        }
        let records = usize::from(hhea.number_of_h_metrics());
        if records == 0 || records > count {
            return Err(invalid());
        }
        let glyph = usize::try_from(glyph.to_u32()).map_err(|_| overflow())?;
        let at = if glyph < records {
            add(mul(glyph, 4)?, 2)?
        } else {
            add(mul(records, 4)?, mul(glyph - records, 2)?)?
        };
        let hmtx = font.table_data(Tag::new(b"hmtx")).ok_or_else(invalid)?;
        hmtx.as_bytes()
            .get(..add(mul(records, 4)?, mul(count - records, 2)?)?)
            .ok_or_else(invalid)?;
        word(hmtx.as_bytes(), at)?;
        // The final complete long metric supplies advance width for trailing LSBs.
        word(hmtx.as_bytes(), mul(records - 1, 4)?)?;
        Ok(())
    }

    pub(super) fn qualify(
        font: &skrifa::FontRef<'_>,
        glyph: GlyphId,
        coordinates: &[NormalizedCoord],
        coordinate_bytes: usize,
    ) -> Checked<Summary> {
        let raw = raw_glyph(font, glyph)?;
        if raw.is_empty() || signed(raw, 0)? >= 0 {
            return super::qualify_simple_glyf(font, glyph, coordinates, coordinate_bytes);
        }
        if !coordinates.is_empty()
            || [b"fvar", b"gvar", b"HVAR", b"VARC"]
                .iter()
                .any(|tag| font.table_data(Tag::new(tag)).is_some())
        {
            return Err(unsupported());
        }
        let mut memos = Vec::<Memo>::new();
        let mut frames = Vec::<Frame<'_>>::new();
        grow(&mut memos, 0, coordinate_bytes)?;
        memos.push(Memo {
            glyph,
            state: State::Active,
        });
        grow(&mut frames, vector_bytes(&memos)?, coordinate_bytes)?;
        frames.push(Frame::new(glyph, raw)?);
        loop {
            if frames.last().expect("root frame").terminal {
                let frame = frames.pop().expect("current frame");
                let completed_glyph = frame.glyph;
                let summary = frame.finish()?;
                memos
                    .iter_mut()
                    .find(|m| m.glyph == completed_glyph)
                    .expect("active memo")
                    .state = State::Qualified(summary);
                if let Some(parent) = frames.last_mut() {
                    parent.include(summary)?;
                } else {
                    return Ok(summary);
                }
                continue;
            }
            let child = frames.last_mut().expect("current frame").next()?;
            if let Some(memo) = memos.iter().find(|m| m.glyph == child) {
                let State::Qualified(summary) = memo.state else {
                    return Err(invalid());
                };
                if add(frames.len(), summary.depth)? > 32 {
                    return Err(unsupported());
                }
                frames.last_mut().expect("parent frame").include(summary)?;
                continue;
            }
            let raw = raw_glyph(font, child)?;
            if frames.len() > 32 {
                return Err(unsupported());
            }
            if raw.is_empty() || signed(raw, 0)? >= 0 {
                let live = add(
                    coordinate_bytes,
                    add(vector_bytes(&memos)?, vector_bytes(&frames)?)?,
                )?;
                let summary = super::qualify_simple_glyf(font, child, coordinates, live)?;
                grow(&mut memos, vector_bytes(&frames)?, coordinate_bytes)?;
                memos.push(Memo {
                    glyph: child,
                    state: State::Qualified(summary),
                });
                frames.last_mut().expect("parent frame").include(summary)?;
            } else {
                grow(&mut memos, vector_bytes(&frames)?, coordinate_bytes)?;
                memos.push(Memo {
                    glyph: child,
                    state: State::Active,
                });
                grow(&mut frames, vector_bytes(&memos)?, coordinate_bytes)?;
                frames.push(Frame::new(child, raw)?);
            }
        }
    }
}

#[cfg(test)]
mod tests;
