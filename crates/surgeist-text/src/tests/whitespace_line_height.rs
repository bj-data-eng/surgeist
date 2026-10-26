//! Nonempty whitespace lines retain the metrics of their selected source.
use crate::{
    Font, FontCatalogueMode, Id, LineHeight, Options, Range, Source, Style, System, SystemOptions,
};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");

#[test]
fn standalone_whitespace_line_does_not_inherit_an_unselected_tall_span() {
    let base = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        ..Style::default()
    };
    let id = Id::from_u64(7203);
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for quantize in [false, true] {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        for text in ["X\n ", "X\n  "] {
            for tall in [false, true] {
                let mut source = Source::identified(text, id, 3);
                if tall {
                    source.span(
                        Range::new(0, 1),
                        Style {
                            line_height: LineHeight::Absolute(20.0),
                            ..base.clone()
                        },
                    );
                }
                let layout = system
                    .layout(
                        source,
                        base.clone(),
                        Options {
                            quantize,
                            ..Options::default()
                        },
                    )
                    .unwrap();
                assert_eq!(layout.source().id(), Some(id));
                assert_eq!(layout.source().revision(), 3);
                assert_eq!(
                    layout.source().text().code_units(),
                    text.encode_utf16().collect::<Vec<_>>()
                );
                let lines = layout.lines();
                assert_eq!(lines.len(), 2);
                assert_eq!(lines[0].range(), Range::new(0, 2));
                assert_eq!(lines[1].range(), Range::new(2, text.len()));
                // Retained Ahem cmap and hmtx: X58 and SPACE3 each advance10.
                // The second line has actual original SPACE content, not an
                // empty terminal line or an invented strut-only participant.
                assert!(layout.run_views().all(|run| run.face().bytes() == AHEM
                    && run.effective_instance().unwrap().font_size() == 10.0));
                let runs = layout.glyph_runs();
                let mut glyphs = runs
                    .iter()
                    .flat_map(|run| run.glyphs())
                    .filter(|g| g.advance() != 0.0)
                    .map(|g| (g.range(), g.id(), g.advance()))
                    .collect::<Vec<_>>();
                glyphs.sort_by_key(|g| g.0.start);
                let mut wanted = vec![(Range::new(0, 1), 58, 10.0)];
                wanted.extend((2..text.len()).map(|i| (Range::new(i, i + 1), 3, 10.0)));
                assert_eq!(glyphs, wanted);
                let height = if tall { 20.0 } else { 10.0 };
                let baseline = if tall { 13.0 } else { 8.0 };
                actual.push((
                    lines[0].bounds().size.height,
                    lines[0].baseline(),
                    lines[1].bounds().size.height,
                    lines[1].baseline(),
                ));
                // Ahem ascent8/descent2, so the final line's own LH10 has
                // baseline8 above its top, after the real first-line height.
                expected.push((height, baseline, 10.0, height + 8.0));
            }
        }
    }
    assert_eq!(actual, expected);
}
