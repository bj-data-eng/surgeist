//! Independent signed native-font struts and adopted decoration frames.
use super::*;

fn system_with(bytes: Vec<u8>) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}

fn named(
    system: &mut System,
    input: &AnalyzedSource,
    orientation: LineOrientation,
) -> PreparedText {
    system
        .prepare(
            input,
            PreparationOptions::default()
                .with_orientation(orientation)
                .with_baseline_reference(ALPHA),
        )
        .unwrap()
}

#[test]
fn signed_fractional_and_outside_font_references_survive_body_and_empty_default_struts() {
    for (romn, b) in [(-100, -1.0), (25, 0.25), (2000, 20.0), (-500, -5.0)] {
        let bytes = font(romn);
        qualify(&bytes, romn);
        let mut system = system_with(bytes);
        for height in [10.0, 6.0] {
            // Native font/default struts retain signed sides. These are not
            // supplied LineStrut allocations (which have a different contract).
            let half = (height - 10.0) * 0.5;
            let before = 8.0 - b + half;
            let after = b + 2.0 + half;
            assert_eq!(before + after, height);
            for orientation in [
                LineOrientation::Horizontal,
                LineOrientation::SidewaysClockwise,
            ] {
                for text in ["A", ""] {
                    let input = analyzed(
                        Source::identified(text, Id::from_u64(1561010), 1),
                        Style {
                            line_height: LineHeight::Absolute(height),
                            ..own_style()
                        },
                    );
                    let prepared = named(&mut system, &input, orientation);
                    assert_eq!(prepared.baseline_reference(), Some(ALPHA));
                    let line = finalize(&prepared);
                    assert_eq!(line.feedback().baseline, before, "romn={romn} {text:?}");
                    assert_eq!(line.feedback().line_height, height);
                    assert_eq!(
                        line.feedback().advance,
                        if text.is_empty() { 0.0 } else { 10.0 }
                    );
                    if text.is_empty() {
                        assert!(line.run_views().all(|run| run.glyphs().len() == 0));
                        continue;
                    }
                    assert_eq!(
                        line.run_views().next().unwrap().origin(),
                        Point::new(0.0, 8.0 + half)
                    );
                    let anchor = if orientation == LineOrientation::Horizontal {
                        70.0
                    } else {
                        100.0
                    };
                    let layout = placed(&prepared, line, anchor);
                    let (origin, bounds, caret) = if orientation == LineOrientation::Horizontal {
                        (
                            Point::new(20.0, 70.0 + b),
                            Rect::new(20.0, 70.0 - before, 200.0, height),
                            Rect::new(20.0, 70.0 - before, 1.0, height),
                        )
                    } else {
                        (
                            Point::new(100.0 - b, 20.0),
                            Rect::new(100.0 - after, 20.0, height, 200.0),
                            Rect::new(100.0 - after, 20.0, height, 1.0),
                        )
                    };
                    assert_eq!(layout.bounds(), bounds);
                    assert_eq!(layout.run_views().next().unwrap().origin(), origin);
                    assert_eq!(layout.glyph_runs()[0].origin(), origin);
                    assert_eq!(
                        layout
                            .cursor(Cursor::new(0, Affinity::After))
                            .unwrap()
                            .rect(),
                        caret
                    );
                    assert_eq!(layout.source().text().code_units(), [65]);
                }
            }
        }
    }
}

fn decorated_font(romn: i16) -> Vec<u8> {
    let bytes = font(romn);
    let mut post = sfnt::table(&bytes, *b"post").to_vec();
    post[8..10].copy_from_slice(&(-200_i16).to_be_bytes());
    post[10..12].copy_from_slice(&100_i16.to_be_bytes());
    let mut os2 = sfnt::table(&bytes, *b"OS/2").to_vec();
    os2[26..28].copy_from_slice(&100_i16.to_be_bytes());
    // Skrifa's strike position is the top, not its center. Top350 with
    // thickness100 gives the intended scaled center3 at size10.
    os2[28..30].copy_from_slice(&350_i16.to_be_bytes());
    let bytes = sfnt::replace_tables(&bytes, vec![(*b"post", post), (*b"OS/2", os2)]);
    // Parse the actual authored values; metrics/gap/outlines remain Ahem's.
    for (tag, offsets, expected) in [
        (*b"post", [8, 10], [-200, 100]),
        (*b"OS/2", [26, 28], [100, 350]),
        (*b"hhea", [4, 6], [800, -200]),
    ] {
        assert_eq!(
            offsets.map(|at| i16::from_be_bytes(
                sfnt::table(&bytes, tag)[at..at + 2].try_into().unwrap()
            )),
            expected
        );
    }
    for tag in [*b"hmtx", *b"glyf", *b"loca", *b"BASE"] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(&font(romn), tag));
    }
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    let raw = skrifa::metrics::Metrics::new(&parsed, FontSize::new(10.0), LocationRef::default())
        .strikeout
        .unwrap();
    assert_eq!((raw.offset, raw.thickness), (3.5, 1.0));
    bytes
}

#[test]
fn automatic_font_decorations_rebase_once_while_absolute_offsets_use_the_shared_reference() {
    for (romn, b) in [(100, 1.0), (-100, -1.0), (25, 0.25)] {
        let mut system = system_with(decorated_font(romn));
        {
            let face = system.registered_font_faces().next().unwrap().to_owned();
            let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
            let metrics = instance.metrics(system.font_catalogue_identity()).unwrap();
            assert_eq!(
                (
                    metrics.strikeout().center_offset(),
                    metrics.strikeout().thickness()
                ),
                (3.0, 1.0)
            );
        }
        for absolute in [false, true] {
            let underline = if absolute {
                crate::Decoration::solid()
                    .with_offset(crate::DecorationOffset::try_absolute(-2.0).unwrap())
                    .with_thickness(crate::DecorationThickness::try_absolute(1.0).unwrap())
            } else {
                crate::Decoration::solid()
            };
            let strike = if absolute {
                crate::Decoration::solid()
                    .with_offset(crate::DecorationOffset::try_absolute(3.0).unwrap())
                    .with_thickness(crate::DecorationThickness::try_absolute(1.0).unwrap())
            } else {
                crate::Decoration::solid()
            };
            let input = analyzed(
                Source::new("A"),
                Style {
                    underline,
                    strikethrough: strike,
                    ..own_style()
                },
            );
            let shift = if absolute { 0.0 } else { b };
            for orientation in [
                LineOrientation::Horizontal,
                LineOrientation::SidewaysClockwise,
            ] {
                let prepared = named(&mut system, &input, orientation);
                let (anchor, expected) = if orientation == LineOrientation::Horizontal {
                    (
                        70.0,
                        [
                            Rect::new(20.0, 72.0 + shift, 10.0, 1.0),
                            Rect::new(20.0, 66.5 + shift, 10.0, 1.0),
                        ],
                    )
                } else {
                    (
                        100.0,
                        [
                            Rect::new(97.0 - shift, 20.0, 1.0, 10.0),
                            Rect::new(102.5 - shift, 20.0, 1.0, 10.0),
                        ],
                    )
                };
                let layout = placed(&prepared, finalize(&prepared), anchor);
                assert_eq!(
                    layout
                        .decoration_views()
                        .map(|view| view.rect())
                        .collect::<Vec<_>>(),
                    expected,
                    "romn={romn} absolute={absolute} {orientation:?}"
                );
                assert_eq!(
                    layout
                        .decorations()
                        .iter()
                        .map(|view| view.rect())
                        .collect::<Vec<_>>(),
                    expected
                );
                assert_eq!(
                    layout.run_views().next().unwrap().origin(),
                    if orientation == LineOrientation::Horizontal {
                        Point::new(20.0, 70.0 + b)
                    } else {
                        Point::new(100.0 - b, 20.0)
                    }
                );
            }
        }
    }
}
