//! Existing native plans retain their font offsets and outline axes through
//! finalized, placed borrowed and owned readback. Font-table premises and the
//! distinct vert/vrtr forms make the physical-origin expectations independent.

use super::*;
use crate::GlyphBasis;

#[test]
fn native_run_axes_and_glyph_origins_survive_all_readback_forms() {
    let bytes = vert_font();
    qualify(&bytes, &[(0x58, 58)]);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let upem = font.head().unwrap().units_per_em();
    assert_eq!(upem, 1000);
    let hhea = font.hhea().unwrap();
    let hmtx = font.hmtx().unwrap();
    // Glyphs beyond numberOfHMetrics inherit the last advance. This fixture
    // instead has281 glyphs and281 long entries, so all three indices are real.
    assert_eq!(hhea.number_of_h_metrics(), 281);
    assert_eq!(hmtx.h_metrics().len(), 281);
    for glyph in [58, 82, 274] {
        let raw_advance = hmtx.h_metrics()[glyph].advance.get();
        assert_eq!(raw_advance, 1000);
        assert_eq!(f64::from(raw_advance) * 10.0 / f64::from(upem), 10.0);
    }
    // qualify() checks UPEM1000, vmtx advance1200/TSB100 and vertical BASE
    // edges0/1200. The preserved Ahem horizontal sides are800/-200 with gap0.
    assert_eq!(
        (
            hhea.ascender().to_i16(),
            hhea.descender().to_i16(),
            hhea.line_gap().to_i16()
        ),
        (800, -200, 0)
    );
    assert!(
        !font
            .os2()
            .unwrap()
            .fs_selection()
            .contains(skrifa::raw::tables::os2::SelectionFlags::USE_TYPO_METRICS)
    );
    // The nonzero hhea sides are the used horizontal source. composed_line
    // requests Absolute12px at size10px, not MetricsRelative(1.2). Its signed
    // half-leading is (12-(8+2))/2=1, hence before9/after3 and total height12.
    let before = f64::from(hhea.ascender().to_i16()) * 10.0 / f64::from(upem);
    let after = -f64::from(hhea.descender().to_i16()) * 10.0 / f64::from(upem);
    let half_leading = (12.0 - (before + after)) * 0.5;
    assert_eq!((before, after, half_leading), (8.0, 2.0, 1.0));
    assert_eq!((before + half_leading, after + half_leading), (9.0, 3.0));
    let mut system = system_with(&bytes);
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
        LineOrientation::Vertical,
    ] {
        let typography = if orientation == LineOrientation::Vertical {
            AnalysisTypography::Upright {
                computed_direction: ParagraphBaseDirection::RightToLeft,
            }
        } else {
            AnalysisTypography::HorizontalComposition
        };
        let (basis, id, advance, dx, dy, baseline, origin, expected, bounds) = match orientation {
            LineOrientation::Horizontal => (
                GlyphBasis::Unrotated,
                58,
                10.0,
                0.0,
                0.0,
                9.0,
                Point::new(20.0, 100.0),
                [Point::new(20.0, 100.0), Point::new(30.0, 100.0)],
                Rect::new(20.0, 91.0, 20.0, 12.0),
            ),
            LineOrientation::SidewaysClockwise => (
                GlyphBasis::Clockwise,
                274,
                10.0,
                0.0,
                0.0,
                9.0,
                Point::new(100.0, 20.0),
                [Point::new(100.0, 20.0), Point::new(100.0, 30.0)],
                Rect::new(97.0, 20.0, 12.0, 20.0),
            ),
            LineOrientation::Vertical => (
                GlyphBasis::Unrotated,
                82,
                12.0,
                -6.0,
                1.0,
                6.0,
                Point::new(100.0, 20.0),
                [Point::new(94.0, 21.0), Point::new(94.0, 33.0)],
                Rect::new(94.0, 20.0, 12.0, 24.0),
            ),
        };
        let (prepared, line) = composed_line(
            &mut system,
            "XX",
            &[],
            typography,
            orientation,
            ParagraphBaseDirection::LeftToRight,
        );
        assert_eq!(line.feedback().advance, 2.0 * advance);
        let mut finalized = Vec::new();
        for run in line.run_views() {
            assert_eq!(run.glyph_basis(), basis);
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.origin().y, baseline);
            for glyph in run.glyphs() {
                assert_eq!(glyph.id(), id);
                assert_eq!((f64::from(glyph.x()), f64::from(glyph.y())), (dx, dy));
                assert_eq!(f64::from(glyph.advance()), advance);
                finalized.push((glyph.range(), run.origin().x + glyph.pen()));
            }
        }
        assert_eq!(
            finalized,
            [(Range::new(0, 1), 0.0), (Range::new(1, 2), advance)]
        );
        let layout = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 20.0,
                    baseline: 100.0,
                },
                orientation,
                atomics: vec![],
            })],
        })
        .unwrap();
        assert_eq!(layout.bounds(), bounds);
        let borrowed = layout.run_views().collect::<Vec<_>>();
        let owned = layout.glyph_runs();
        assert_eq!(borrowed.len(), owned.len());
        let mut placed = Vec::new();
        for (borrowed, owned) in borrowed.iter().zip(&owned) {
            assert_eq!(borrowed.orientation(), orientation);
            assert_eq!(owned.orientation(), orientation);
            assert_eq!(
                (borrowed.glyph_basis(), owned.glyph_basis()),
                (basis, basis)
            );
            assert_eq!(borrowed.origin(), owned.origin());
            assert_eq!(borrowed.face().bytes(), owned.font().bytes());
            let glyphs = borrowed.glyphs().collect::<Vec<_>>();
            assert_eq!(glyphs.len(), owned.glyphs().len());
            for (glyph, owned_glyph) in glyphs.iter().zip(owned.glyphs()) {
                assert_eq!(glyph.id(), owned_glyph.id());
                assert_eq!((glyph.x(), glyph.y()), (owned_glyph.x(), owned_glyph.y()));
                assert_eq!(glyph.pen(), owned_glyph.pen());
                let physical = crate::layout::glyph_origin(
                    borrowed.orientation(),
                    borrowed.glyph_basis(),
                    borrowed.origin(),
                    glyph.pen(),
                    f64::from(glyph.x()),
                    f64::from(glyph.y()),
                );
                placed.push(physical);
            }
        }
        assert_eq!(borrowed[0].origin(), origin);
        assert_eq!(placed, expected);
    }
}
