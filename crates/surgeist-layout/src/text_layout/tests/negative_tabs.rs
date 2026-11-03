//! Signed CSS tab periods consume actual Layout float displacement.
use super::*;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

fn paragraph() -> PreparedParagraph {
    let mut style = ahem_style(20.0, 20.0);
    style.white_space = surgeist_text::WhiteSpace::Preserve;
    style.tab_size = surgeist_text::TabSize::spaces(4.0).unwrap();
    // This is the actual captured block Style, also the body Style. There are
    // no authored SPACE characters, so word-spacing does not alter A/B.
    style.word_spacing = -21.0;
    PreparedParagraph::with_css_tabs(
        vec![ParagraphPart::text(1, Source::new("A\tB"), style.clone())],
        style,
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::LeftToRight,
    )
}

fn compute<S: LayoutScalar>(
    p: &PreparedParagraph,
    with_float: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, if with_float { vec![2, 1] } else { vec![1] })
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, p.text_input(1));
    if with_float {
        tree = tree.children(2, []).style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(30.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(0)],
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(&capture, 0, request(100.0)).unwrap()
}

fn check<S: LayoutScalar>() {
    let p = paragraph(); // current failure is Text NegativeTabInterval, not Layout setup
    assert_eq!(
        p.prepared.analyzed_source().source().text().code_units(),
        [65, 9, 66]
    );
    for (with_float, expected_origin, first_x, b_x, advance) in [
        (false, 0.0, 0.0, 28.0, 48.0),
        (true, -30.0, 30.0, 56.0, 46.0),
    ] {
        let batch = compute::<S>(&p, with_float);
        let placed = batch.placed_text()[0].layout();
        assert_eq!(placed.lines().len(), 1);
        let glyphs: Vec<_> = placed
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.font_size(), 20.0);
                assert_eq!(run.face().bytes(), AHEM);
                assert_eq!(run.effective_instance().unwrap().font_size(), 20.0);
                assert_eq!(
                    run.face().catalogue_identity(),
                    p.prepared.font_catalogue_identity()
                );
                run.glyphs()
                    .map(|g| {
                        (
                            g.id(),
                            g.advance(),
                            run.origin().x + g.pen() + f64::from(g.x()),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(glyphs, [(35, 20.0, first_x), (36, 20.0, b_x)]);
        let finalized = batch.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(finalized.feedback().advance, advance);
        assert_eq!(
            finalized.selected_line().request().context.tabs,
            surgeist_text::TabContext::CssStops {
                origin: Some(expected_origin)
            }
        );
        assert_eq!(placed.source().text().code_units(), [65, 9, 66]);
        assert_eq!(placed.lines()[0].range(), surgeist_text::Range::new(0, 3));
        let tab = placed
            .source_range_fragments(&p.origin(1), surgeist_text::Range::new(1, 2))
            .unwrap();
        let pieces = tab
            .units()
            .flat_map(|unit| unit.pieces().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].rect().origin.x, first_x + 20.0);
        assert_eq!(pieces[0].rect().size.width, advance - 40.0);
        assert!(
            placed
                .cursor(surgeist_text::Cursor::new(
                    1,
                    surgeist_text::Affinity::After
                ))
                .is_ok()
        );
        assert!(
            placed
                .cursor(surgeist_text::Cursor::new(
                    2,
                    surgeist_text::Affinity::Before
                ))
                .is_ok()
        );
        if with_float {
            let output = batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == 2)
                .unwrap()
                .output();
            assert_eq!(output.size.width.to_f64(), 30.0);
            assert_eq!(output.size.height.to_f64(), 20.0);
        }
    }
}

#[test]
fn negative_spacing_period_consumes_actual_float_displacement_f32() {
    check::<f32>();
}
#[test]
fn negative_spacing_period_consumes_actual_float_displacement_f64() {
    check::<f64>();
}
