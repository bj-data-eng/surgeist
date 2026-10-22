use std::fmt;

use crate::{
    Brush, DecorationKind, DecorationRun, EffectiveFontInstance, Error, ErrorCode, ErrorDetail,
    FontFace, FontSynthesisKind, Layout, Range, Rect, Result,
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
        let font = self.run.run().font();
        FontFace::new(&font.data, font.index, self.layout.key.font_catalogue())
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
