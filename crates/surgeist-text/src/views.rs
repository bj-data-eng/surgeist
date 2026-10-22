use std::{fmt, sync::Arc};

use skrifa::{Tag, raw::TableProvider};

use crate::{
    Brush, DecorationKind, DecorationRun, Error, ErrorCode, ErrorDetail, FontSynthesisKind, Layout,
    NumericRequirement, Range, Rect, Result,
};

/// An actual positioned glyph run borrowed from one immutable layout.
pub struct RunView<'a> {
    layout: &'a Layout,
    line: usize,
    run: parley::GlyphRun<'a, Brush>,
}

impl fmt::Debug for RunView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunView")
            .field("line", &self.line)
            .field("range", &self.range())
            .field("font_size", &self.font_size())
            .finish_non_exhaustive()
    }
}

impl Layout {
    /// Visits actual paint-order runs without allocating owned metadata or glyph vectors.
    pub fn run_views(&self) -> impl Iterator<Item = RunView<'_>> {
        self.positioned_runs().map(|(line, run)| RunView {
            layout: self,
            line,
            run,
        })
    }

    /// Visits actual decoration rectangles after the glyph paint pass, without allocation.
    pub fn decoration_views(&self) -> impl Iterator<Item = DecorationRun> + '_ {
        self.positioned_runs()
            .flat_map(|(_, run)| run_decorations(&run).into_iter().flatten())
    }
}

impl RunView<'_> {
    /// Actual line ordinal in the borrowed layout.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Actual underlying shaped-run ordinal within its line.
    #[must_use]
    pub fn run_index(&self) -> usize {
        self.run.run().index()
    }

    /// UTF-16 source extent of the underlying shaped run.
    #[must_use]
    pub fn range(&self) -> Range {
        self.layout.projection.range(self.run.run().text_range())
    }

    /// Actual resolved color brush for this positioned run.
    #[must_use]
    pub fn brush(&self) -> Brush {
        self.run.style().brush
    }

    /// Actual shaping size, rather than an authored or inferred size.
    #[must_use]
    pub fn font_size(&self) -> f32 {
        self.run.run().font_size()
    }

    /// Actual resource and collection face used by the shaper.
    #[must_use]
    pub fn face(&self) -> FontFace<'_> {
        FontFace {
            data: self.run.run().font(),
        }
    }

    /// Fully positioned provider glyphs; offsets and baseline have already been applied.
    pub fn glyphs(&self) -> impl Iterator<Item = GlyphPlacement> + '_ {
        self.run.positioned_glyphs().map(|glyph| GlyphPlacement {
            id: glyph.id,
            x: glyph.x,
            y: glyph.y,
            advance: glyph.advance,
        })
    }

    /// Qualifies complete canonical normalized coordinates and unresolved synthesis.
    ///
    /// Empty stored coordinates mean a static face only if `fvar` is absent.
    /// A valid variable face whose shaper cleared all-zero coordinates exposes
    /// the complete axis count through a lazy zero iterator. Malformed present
    /// metadata, noncanonical coordinates and unresolved faux synthesis refuse.
    pub fn effective_instance(&self) -> Result<EffectiveFontInstance<'_>> {
        let run = self.run.run();
        let synthesis = run.synthesis();
        if synthesis.embolden() {
            return Err(unresolved(FontSynthesisKind::Embolden));
        }
        if synthesis.skew().is_some() {
            return Err(unresolved(FontSynthesisKind::Skew));
        }
        EffectiveFontInstance::qualify(self.face(), run.font_size(), run.normalized_coords())
    }
}

/// A source-free glyph placement read directly from an actual provider run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphPlacement {
    id: u32,
    x: f32,
    y: f32,
    advance: f32,
}

impl GlyphPlacement {
    /// Actual glyph identifier in the run's face.
    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }
    /// Fully positioned horizontal coordinate.
    #[must_use]
    pub const fn x(self) -> f32 {
        self.x
    }
    /// Fully positioned baseline-relative vertical coordinate.
    #[must_use]
    pub const fn y(self) -> f32 {
        self.y
    }
    /// Actual provider advance.
    #[must_use]
    pub const fn advance(self) -> f32 {
        self.advance
    }
}

/// A borrowed actual provider font resource and collection face.
#[derive(Clone, Copy)]
pub struct FontFace<'a> {
    data: &'a parley::FontData,
}

impl fmt::Debug for FontFace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontFace")
            .field("index", &self.index())
            .field("bytes", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

impl<'a> FontFace<'a> {
    /// Whole original resource bytes, borrowed without copying.
    #[must_use]
    pub fn bytes(&self) -> &'a [u8] {
        self.data.data.as_ref()
    }

    /// Actual face index within that resource.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.data.index
    }

    /// Compares the actual strong allocation and collection index, not IDs or content.
    #[must_use]
    pub fn is_same_face(&self, other: &Self) -> bool {
        let (left, _) = self.data.data.clone().into_raw_parts();
        let (right, _) = other.data.data.clone().into_raw_parts();
        self.index() == other.index() && Arc::ptr_eq(&left, &right)
    }
}

/// A borrowed, complete shaping instance with no unresolved faux synthesis.
#[derive(Debug)]
pub struct EffectiveFontInstance<'a> {
    face: FontFace<'a>,
    size: f32,
    axes: usize,
    stored: &'a [i16],
}

impl<'a> EffectiveFontInstance<'a> {
    fn qualify(face: FontFace<'a>, size: f32, stored: &'a [i16]) -> Result<Self> {
        if !size.is_finite() || size <= 0.0 {
            return Err(
                Error::new(ErrorCode::InvalidFontInstance, "invalid shaping size").with_detail(
                    ErrorDetail::InvalidNumericField {
                        field: "shaping font size",
                        value: size,
                        requirement: NumericRequirement::FiniteGreaterThanZero,
                    },
                ),
            );
        }
        let axes = axis_count(face)?;
        if !stored.is_empty() && stored.len() != axes {
            return Err(Error::new(
                ErrorCode::InvalidFontInstance,
                "normalized coordinates do not cover the actual axes",
            )
            .with_detail(ErrorDetail::FontCoordinateCount {
                expected: axes,
                actual: stored.len(),
            }));
        }
        for (axis, &bits) in stored.iter().enumerate() {
            if !(-16384..=16384).contains(&bits) {
                return Err(Error::new(
                    ErrorCode::InvalidFontInstance,
                    "noncanonical normalized font coordinate",
                )
                .with_detail(ErrorDetail::FontCoordinate { axis, bits }));
            }
        }
        Ok(Self {
            face,
            size,
            axes,
            stored,
        })
    }

    /// The actual borrowed face used by this instance.
    #[must_use]
    pub const fn face(&self) -> FontFace<'a> {
        self.face
    }

    /// Actual canonical shaping size.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.size
    }

    /// Complete actual variation-axis count; zero only for a proven static face.
    #[must_use]
    pub const fn axis_count(&self) -> usize {
        self.axes
    }

    /// Complete normalized F2Dot14 bits in actual axis order, without allocation.
    pub fn normalized_coords(&self) -> impl ExactSizeIterator<Item = i16> + Clone + '_ {
        (0..self.axes).map(|axis| {
            if self.stored.is_empty() {
                0
            } else {
                self.stored[axis]
            }
        })
    }
}

fn axis_count(face: FontFace<'_>) -> Result<usize> {
    let font = skrifa::FontRef::from_index(face.bytes(), face.index()).map_err(|_| malformed())?;
    let tag = Tag::new(b"fvar");
    let mut records = font
        .table_directory()
        .table_records()
        .iter()
        .filter(|record| record.tag() == tag);
    if records.next().is_none() {
        return Ok(0);
    }
    if records.next().is_some() {
        return Err(malformed());
    }
    // Table presence is established separately: inaccessible/truncated data must
    // never be reclassified as a static font by a permissive metadata accessor.
    let data = font.table_data(tag).ok_or_else(malformed)?;
    let fvar = font.fvar().map_err(|_| malformed())?;
    let count = usize::from(fvar.axis_count());
    let offset =
        usize::try_from(fvar.axis_instance_arrays_offset().to_u32()).map_err(|_| malformed())?;
    let instance_size = usize::from(fvar.instance_size());
    let required_instance = count.checked_mul(4).and_then(|n| n.checked_add(4));
    let axes_end = count.checked_mul(20).and_then(|n| offset.checked_add(n));
    let table_end = axes_end.and_then(|n| {
        usize::from(fvar.instance_count())
            .checked_mul(instance_size)
            .and_then(|instances| n.checked_add(instances))
    });
    if fvar.version().major != 1
        || fvar.version().minor != 0
        || data.as_bytes().get(6..8) != Some(&[0, 2])
        || count == 0
        || fvar.axis_size() != 20
        || offset < 16
        || !required_instance.is_some_and(|n| instance_size == n || instance_size == n + 2)
        || !table_end.is_some_and(|end| end <= data.len())
    {
        return Err(malformed());
    }
    let axes = fvar.axes().map_err(|_| malformed())?;
    if axes.len() != count {
        return Err(malformed());
    }
    for (index, axis) in axes.iter().enumerate() {
        if axis.min_value() > axis.default_value()
            || axis.default_value() > axis.max_value()
            || axes[..index]
                .iter()
                .any(|previous| previous.axis_tag() == axis.axis_tag())
        {
            return Err(malformed());
        }
    }
    Ok(count)
}

fn malformed() -> Error {
    Error::new(
        ErrorCode::InvalidFontInstance,
        "malformed actual font variation metadata",
    )
    .with_detail(ErrorDetail::MalformedFontVariations)
}

fn unresolved(kind: FontSynthesisKind) -> Error {
    Error::new(
        ErrorCode::UnsupportedFeature,
        "font synthesis needs a concrete outline operation",
    )
    .with_detail(ErrorDetail::UnresolvedFontSynthesis { kind })
}

pub(crate) fn run_decorations(run: &parley::GlyphRun<'_, Brush>) -> [Option<DecorationRun>; 2] {
    let style = run.style();
    let metrics = run.run().metrics();
    [
        style.underline.as_ref().map(|decoration| {
            DecorationRun::new(
                Rect::new(
                    run.offset(),
                    crate::layout::decoration_top(
                        run.baseline(),
                        decoration.offset.unwrap_or(metrics.underline_offset),
                    ),
                    run.advance(),
                    decoration.size.unwrap_or(metrics.underline_size),
                ),
                decoration.brush,
                DecorationKind::Underline,
            )
        }),
        style.strikethrough.as_ref().map(|decoration| {
            DecorationRun::new(
                Rect::new(
                    run.offset(),
                    crate::layout::decoration_top(
                        run.baseline(),
                        decoration.offset.unwrap_or(metrics.strikethrough_offset),
                    ),
                    run.advance(),
                    decoration.size.unwrap_or(metrics.strikethrough_size),
                ),
                decoration.brush,
                DecorationKind::Strikethrough,
            )
        }),
    ]
}

#[cfg(test)]
mod tests;
