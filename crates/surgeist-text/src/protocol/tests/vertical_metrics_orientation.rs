//! Nominal vertical glyph observations do not authorize vertical line layout.
use super::*;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

fn observe_actual_instance(line: &FinalizedLine, expected: &crate::FontCatalogueIdentity) {
    let run = line.run_views().next().unwrap();
    let font = skrifa::FontRef::new(run.face().bytes()).unwrap();
    assert_eq!(run.face().catalogue_identity(), expected);
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.charmap().map(0x58_u32).unwrap().to_u32(), 58);
    assert_eq!(
        font.glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(58)),
        Some(1000.0)
    );
    assert_eq!(
        (
            font.hhea().unwrap().ascender().to_i16(),
            font.hhea().unwrap().descender().to_i16()
        ),
        (800, -200)
    );
    assert!(font.table_data(skrifa::Tag::new(b"vhea")).is_none());
    assert!(font.table_data(skrifa::Tag::new(b"vmtx")).is_none());
    let instance = run.effective_instance().unwrap();
    assert_eq!(instance.font_size(), 20.0);
    let observation = instance.vertical_glyph_metrics(58, expected).unwrap();
    assert_eq!(observation.glyph_id(), 58);
    // Actual parent Ahem fixture: size20, W1000, A800, D-200, UPEM1000.
    assert_eq!(
        (
            observation.advance(),
            observation.origin_x(),
            observation.origin_y()
        ),
        (20.0, 10.0, 16.0)
    );
    assert!(observation.instance().face().is_same_face(&run.face()));
}

#[test]
fn successful_nominal_vertical_metrics_do_not_enable_nonhorizontal_selection() {
    let mut system = system();
    let p = prepared(&mut system, "X", WhiteSpace::Preserve);
    assert_eq!(
        p.font_catalogue_identity(),
        system.font_catalogue_identity()
    );
    let input = request(&p);
    let line = p
        .select(input.clone())
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        line.run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|g| (g.id(), g.advance(), g.range()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [(58, 20.0, crate::Range::new(0, 1))]
    );
    observe_actual_instance(&line, system.font_catalogue_identity());
    for orientation in [
        LineOrientation::Vertical,
        LineOrientation::SidewaysClockwise,
    ] {
        let mut rejected = input.clone();
        rejected.context.orientation = orientation;
        let error = p.select(rejected).unwrap_err();
        assert_eq!(
            reason(&error),
            &if orientation == LineOrientation::Vertical {
                ProtocolError::UnsupportedContext {
                    field: ProtocolContextField::Orientation,
                }
            } else {
                ProtocolError::MismatchedContext {
                    field: ProtocolContextField::Orientation,
                }
            }
        );
    }
    // The complete original horizontal receipt remains usable after refusals.
    let layout = place(p, line, 0.0, 30.0).unwrap();
    assert_eq!(layout.source().text().code_units(), [88]);
    assert_eq!(
        layout
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs())
            .map(|g| (g.id(), g.advance(), g.range()))
            .collect::<Vec<_>>(),
        [(58, 20.0, crate::Range::new(0, 1))]
    );
}

#[test]
fn successful_nominal_vertical_metrics_do_not_enable_nonhorizontal_placement() {
    let mut system = system();
    let p = prepared(&mut system, "X", WhiteSpace::Preserve);
    let line = p
        .select(request(&p))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    observe_actual_instance(&line, system.font_catalogue_identity());
    let horizontal = place(p.clone(), line.clone(), 0.0, 30.0).unwrap();
    let before = horizontal
        .glyph_runs()
        .iter()
        .flat_map(|run| run.glyphs())
        .map(|g| (g.id(), g.advance(), g.range()))
        .collect::<Vec<_>>();
    assert_eq!(before, [(58, 20.0, crate::Range::new(0, 1))]);
    for orientation in [
        LineOrientation::Vertical,
        LineOrientation::SidewaysClockwise,
    ] {
        let error = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: p.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line: line.clone(),
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 30.0,
                },
                orientation,
                atomics: vec![],
            })],
        })
        .unwrap_err();
        assert_eq!(
            reason(&error),
            &if orientation == LineOrientation::Vertical {
                ProtocolError::UnsupportedContext {
                    field: ProtocolContextField::Orientation,
                }
            } else {
                ProtocolError::MismatchedContext {
                    field: ProtocolContextField::Orientation,
                }
            }
        );
    }
    assert_eq!(horizontal.source().text().code_units(), [88]);
    assert_eq!(
        horizontal
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs())
            .map(|g| (g.id(), g.advance(), g.range()))
            .collect::<Vec<_>>(),
        before
    );
}
