//! Required font displacement must survive the physical placement boundary.
use super::*;

fn placement(prepared: &PreparedText, line: &FinalizedLine, baseline: f64) -> PlacementRequest {
    PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline,
            },
            orientation: prepared.orientation(),
            atomics: vec![],
        })],
    }
}

fn required_displacement(orientation: LineOrientation) {
    for (romn, b) in [(100, 1.0), (-100, -1.0)] {
        let bytes = font(romn);
        qualify(&bytes, romn);
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(bytes).unwrap();
        let input = analyzed(
            Source::identified("A", Id::from_u64(1561042), 1),
            own_style(),
        );
        let options = PreparationOptions::default().with_orientation(orientation);
        let raw = system.prepare(&input, options).unwrap();
        assert_eq!(raw.baseline_reference(), None);
        let raw_line = finalize(&raw);
        let prepared = system
            .prepare(&input, options.with_baseline_reference(ALPHA))
            .unwrap();
        assert_eq!(prepared.baseline_reference(), Some(ALPHA));
        let line = finalize(&prepared);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (8.0 - b, 10.0)
        );
        let safe_anchor = if orientation == LineOrientation::Horizontal {
            70.0
        } else {
            100.0
        };
        let retained =
            crate::Layout::from_finalized_lines(placement(&prepared, &line, safe_anchor)).unwrap();
        let old_glyphs = retained.glyph_runs();
        let old_bounds = retained.bounds();
        let old_origin = retained.run_views().next().unwrap().origin();
        assert_eq!(
            old_origin,
            if orientation == LineOrientation::Horizontal {
                Point::new(20.0, safe_anchor + b)
            } else {
                Point::new(safe_anchor - b, 20.0)
            }
        );
        for anchor in [1e16, -1e16] {
            // Binary64 spacing here is two. A requested signed one-pixel
            // displacement vanishes, but a ten-pixel metric span survives.
            let delta = if orientation == LineOrientation::Horizontal {
                b
            } else {
                -b
            };
            assert_eq!(anchor + delta, anchor);
            assert!(anchor + 10.0 > anchor);
            let raw_placed =
                crate::Layout::from_finalized_lines(placement(&raw, &raw_line, anchor)).unwrap();
            assert_eq!(
                if orientation == LineOrientation::Horizontal {
                    raw_placed.run_views().next().unwrap().origin().y
                } else {
                    raw_placed.run_views().next().unwrap().origin().x
                },
                anchor,
                "raw design-zero composition has no requested displacement"
            );
            let result = crate::Layout::from_finalized_lines(placement(&prepared, &line, anchor));
            assert!(
                result.is_err(),
                "required signed font displacement {delta} must not vanish at {anchor} in {orientation:?}"
            );
            let error = result.unwrap_err();
            assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
            assert!(
                matches!(error.detail(), Some(crate::ErrorDetail::InvalidTextNumeric {
                    field,
                    phase: crate::TextNumericPhase::Placement,
                    category: crate::TextNumericError::NonZeroLost,
                    value,
                }) if *field == if orientation == LineOrientation::Horizontal {
                    "placed run origin y"
                } else {
                    "placed run origin x"
                } && *value == delta),
                "{error:?}"
            );
            assert_eq!(retained.glyph_runs(), old_glyphs);
            assert_eq!(retained.bounds(), old_bounds);
            assert_eq!(retained.run_views().next().unwrap().origin(), old_origin);
            assert_eq!(retained.source().text().code_units(), [65]);
        }
    }
}

#[test]
fn horizontal_placement_refuses_a_required_font_displacement_lost_at_a_finite_anchor() {
    required_displacement(LineOrientation::Horizontal);
}

#[test]
fn clockwise_placement_refuses_a_required_font_displacement_lost_at_a_finite_anchor() {
    required_displacement(LineOrientation::Sideways);
}
