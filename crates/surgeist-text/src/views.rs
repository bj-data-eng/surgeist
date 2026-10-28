use std::fmt;

use crate::engine::{
    PlacedLine,
    layout::data::{FinalizedRun, PreparedRun},
};
use crate::{
    Brush, DecorationKind, DecorationOffset, DecorationRun, DecorationThickness,
    EffectiveFontInstance, FontFace, Layout, Point, Range, Rect, Result,
};

/// One paint-order run borrowed from the immutable committed layout.
pub struct RunView<'a> {
    layout: &'a Layout,
    line: usize,
    placed: &'a PlacedLine,
    run: &'a FinalizedRun,
    shaped: &'a PreparedRun,
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
    /// Borrows actual paint-order runs without allocating metadata or glyph vectors.
    pub fn run_views(&self) -> impl Iterator<Item = RunView<'_>> {
        self.positioned_runs()
            .map(|(line, placed, run, shaped)| RunView {
                layout: self,
                line,
                placed,
                run,
                shaped,
            })
    }

    // The owning System calls this before publishing the immutable Layout. Readback
    // uses exactly the same calculation, including zero-width metric-only runs.
    pub(crate) fn qualify_decorations(&self) -> Result<()> {
        for decoration in self.decoration_views() {
            let rect = decoration.rect();
            for (field, value) in [
                ("decoration x", rect.origin.x),
                ("decoration y", rect.origin.y),
                ("decoration width", rect.size.width),
                ("decoration height", rect.size.height),
                ("decoration right", rect.origin.x + rect.size.width),
                ("decoration bottom", rect.origin.y + rect.size.height),
            ] {
                if !value.is_finite() {
                    return Err(crate::error::text_numeric_error(
                        field,
                        crate::TextNumericPhase::Placement,
                        crate::TextNumericError::NonFinite,
                        value,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Visits actual decoration rectangles after the glyph paint pass, without allocation.
    pub fn decoration_views(&self) -> impl Iterator<Item = DecorationRun> + '_ {
        self.run_views()
            .flat_map(|run| run_decorations(&run).into_iter().flatten())
    }
}

impl RunView<'_> {
    /// Actual line ordinal in the borrowed layout.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
    /// Admitted line basis. The inline pen axis is separate from glyph axes.
    #[must_use]
    pub const fn orientation(&self) -> crate::LineOrientation {
        self.placed.orientation
    }
    /// Actual outline axes, derived from the authenticated uniform orientation.
    #[must_use]
    pub const fn glyph_basis(&self) -> crate::GlyphBasis {
        crate::GlyphBasis::for_orientation(self.orientation())
    }
    /// Actual shaped-run ordinal within this finalized line.
    #[must_use]
    pub const fn run_index(&self) -> usize {
        self.run.run
    }
    /// Original UTF-16 source extent.
    #[must_use]
    pub const fn range(&self) -> Range {
        self.run.source
    }
    /// Actual effective paint brush.
    #[must_use]
    pub fn brush(&self) -> Brush {
        self.layout.inner.prepared.styles[self.run.style]
            .style
            .brush
    }
    /// Canonical shaping size.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.shaped.font_size
    }
    /// Actual retained resource, face and catalogue publication.
    #[must_use]
    pub fn face(&self) -> FontFace<'_> {
        self.shaped.font.face()
    }
    /// Physical baseline origin, Central for a whole-upright run.
    ///
    /// With origin `(X,Y)`, pen `p` and raw offsets `(dx,dy)`, glyph origins are
    /// `(X+p+dx,Y+dy)` for Horizontal, `(X-dy,Y+p+dx)` for Sideways and
    /// `(X+dx,Y+p+dy)` for Vertical. The outline axes follow [`Self::glyph_basis`].
    #[must_use]
    pub fn origin(&self) -> Point {
        crate::layout::line_point(
            self.placed,
            Point::new(self.run.x, self.placed.finalized.data.metrics.baseline),
        )
    }
    /// Raw canonical glyph offsets and advances with separate logical run-local pens.
    pub fn glyphs(&self) -> impl Iterator<Item = GlyphPlacement> + '_ {
        self.placed.finalized.data.glyphs[self.run.glyphs.clone()]
            .iter()
            .map(|glyph| GlyphPlacement {
                id: glyph.glyph.id,
                x: glyph.glyph.x,
                y: glyph.glyph.y,
                advance: glyph.glyph.advance,
                pen: glyph.pen,
            })
    }
    /// Measures an unhinted named design baseline for this actual run's script
    /// and retained instance.
    pub fn baseline(
        &self,
        expected: &crate::FontCatalogueIdentity,
        axis: crate::FontBaselineAxis,
        name: crate::FontBaseline,
    ) -> Result<crate::FontBaselineMeasurement<'_>> {
        let script =
            crate::FontBaselineScript::from_script(self.shaped.script).ok_or_else(|| {
                crate::Error::new(
                    crate::ErrorCode::InvalidFontMetrics,
                    "unavailable actual baseline script",
                )
                .with_detail(crate::ErrorDetail::FontMetricTable {
                    table: crate::FontMetricTable::Base,
                    reason: crate::FontMetricTableError::BaselineScriptUnavailable,
                })
            })?;
        self.effective_instance()?
            .baseline(expected, script, axis, name)
    }

    /// Qualifies complete canonical coordinates and the actual executable synthesis.
    pub fn effective_instance(&self) -> Result<EffectiveFontInstance<'_>> {
        EffectiveFontInstance::qualify_synthesized(
            self.face(),
            self.shaped.font_size,
            &self.shaped.coords,
            self.shaped.synthesis,
        )
    }
}

/// A source-free glyph placement read directly from an actual committed run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphPlacement {
    id: u32,
    x: f32,
    y: f32,
    advance: f32,
    pen: f64,
}

impl GlyphPlacement {
    /// Actual glyph identifier in the run's face.
    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }
    /// Raw font-x offset, scaled once to layout units. The pen follows the
    /// run's inline axis independently; see [`RunView::origin`].
    #[must_use]
    pub const fn x(self) -> f32 {
        self.x
    }
    /// Raw converted font-y offset, positive downward before applying the
    /// run's glyph basis. See [`RunView::origin`] for physical positioning.
    #[must_use]
    pub const fn y(self) -> f32 {
        self.y
    }
    /// Actual logical run-local inline pen, including line spacing allocation.
    /// It follows physical y for Vertical, independently of the raw x offset.
    #[must_use]
    pub const fn pen(self) -> f64 {
        self.pen
    }
    /// Raw canonical provider advance, before logical spacing allocation.
    #[must_use]
    pub const fn advance(self) -> f32 {
        self.advance
    }
}

fn run_decorations(run: &RunView<'_>) -> [Option<DecorationRun>; 2] {
    let style = &run.layout.inner.prepared.styles[run.run.style].style;
    let metrics = run.shaped.metrics;
    let origin = run.origin();
    [
        (style.underline, DecorationKind::Underline),
        (style.strikethrough, DecorationKind::Strikethrough),
    ]
    .map(|(decoration, kind)| {
        if !decoration.enabled() {
            return None;
        }
        let offset = match decoration.offset() {
            DecorationOffset::Auto => {
                let horizontal = metrics
                    .horizontal()
                    .expect("automatic upright decoration offsets were refused during preparation");
                match kind {
                    DecorationKind::Underline => horizontal.underline_top,
                    DecorationKind::Strikethrough => horizontal.strike_center,
                }
            }
            DecorationOffset::Absolute(value) => value.get(),
        };
        let thickness = match decoration.thickness() {
            DecorationThickness::Auto => {
                let horizontal = metrics.horizontal().expect(
                    "automatic upright decoration thickness was refused during preparation",
                );
                match kind {
                    DecorationKind::Underline => horizontal.underline_thickness,
                    DecorationKind::Strikethrough => horizontal.strike_thickness,
                }
            }
            DecorationThickness::Absolute(value) => value.get(),
        };
        let top = crate::layout::decoration_top(origin.y, offset)
            - if kind == DecorationKind::Strikethrough {
                thickness * 0.5
            } else {
                0.0
            };
        let rect = if run.orientation() == crate::LineOrientation::Horizontal {
            Rect::new(origin.x, top, run.run.advance, thickness)
        } else {
            crate::layout::line_rect(
                run.placed,
                Rect::new(
                    run.run.x,
                    run.placed.finalized.data.metrics.baseline
                        - offset
                        - if kind == DecorationKind::Strikethrough {
                            thickness * 0.5
                        } else {
                            0.0
                        },
                    run.run.advance,
                    thickness,
                ),
            )
        };
        Some(DecorationRun::new(
            rect,
            match decoration.brush() {
                crate::DecorationBrush::TextColor => style.brush,
                crate::DecorationBrush::Color(brush) => brush,
            },
            kind,
        ))
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "views/upright_tests.rs"]
mod upright_tests;
