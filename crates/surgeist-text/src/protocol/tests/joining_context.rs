use super::*;
use crate::{Range, WordBreak};

fn breaking_style() -> Style {
    Style {
        word_break: WordBreak::BreakAll,
        ..style()
    }
}
fn arabic_system() -> System {
    let mut value = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    value
        .register_fonts(crate::shaping_test_fonts::arabic_ahem())
        .unwrap();
    value
}
fn placed_forms(layout: &crate::Layout) -> Vec<(Range, u32)> {
    let fixture = crate::shaping_test_fonts::arabic_ahem();
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), fixture);
        assert_eq!(run.font_size(), 20.0);
        assert_eq!(
            run.face().catalogue_identity(),
            layout.key().font_catalogue()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 20.0);
    }

    let mut result: Vec<_> = layout
        .glyph_runs()
        .iter()
        .flat_map(|run| run.glyphs().iter().map(|glyph| (glyph.range(), glyph.id())))
        .collect();
    result.sort_by_key(|(range, _)| (range.start, range.end));
    result
}
#[test]
fn standalone_metric_policies_keep_unforced_arabic_joining() {
    let mut actual = Vec::new();
    for quantize in [false, true] {
        let mut system = arabic_system();
        let source = Source::new("\u{628}\u{628}\u{628}");
        let control = system
            .layout(
                source.clone(),
                breaking_style(),
                crate::Options {
                    quantize,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        // Existing Arabic fixture's literal init/medi/fina forms, in source order.
        assert_eq!(
            placed_forms(&control),
            [
                (Range::new(0, 1), 100),
                (Range::new(1, 2), 277),
                (Range::new(2, 3), 82)
            ]
        );
        let split = system
            .layout(
                source.clone(),
                breaking_style(),
                crate::Options {
                    width: Some(20.0),
                    quantize,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(split.source(), &source);
        actual.push((quantize, split.lines().len(), placed_forms(&split)));
    }
    assert_eq!(
        actual,
        [
            (
                false,
                3,
                vec![
                    (Range::new(0, 1), 100),
                    (Range::new(1, 2), 277),
                    (Range::new(2, 3), 82)
                ]
            ),
            (
                true,
                3,
                vec![
                    (Range::new(0, 1), 100),
                    (Range::new(1, 2), 277),
                    (Range::new(2, 3), 82)
                ]
            ),
        ]
    );
}
