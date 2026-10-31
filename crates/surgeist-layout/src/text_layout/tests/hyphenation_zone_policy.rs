//! Computed zone policy over actual natural fallbacks and registered Ahem.
use super::*;
use surgeist_text::{
    HyphenateCharacter, HyphenationCounts, HyphenationProvenance, HyphenationResource, Hyphens,
    PreparedAlternativeDefinition, Range, Text, WhiteSpace,
};
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
fn style(white: WhiteSpace, indication: &str) -> surgeist_text::Style {
    let mut value = ahem_style(10.0, 10.0);
    value.locale = Some("en-US".into());
    value.hyphens = Hyphens::Auto;
    value.hyphenate_character = HyphenateCharacter::String(Text::from(indication));
    value.white_space = white;
    value
}
fn paragraph(white: WhiteSpace, indication: &str) -> PreparedParagraph {
    PreparedParagraph::plain(1, Source::new("X associate"), style(white, indication))
}
fn input<S: LayoutScalar>(zone: f64, percent: f64, indent: f64, rtl: bool) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        hyphenate_limit_zone: LengthPercentageOf::from_coefficients(
            S::from_f64(zone),
            S::from_f64(percent),
        )
        .unwrap(),
        text_indent: TextIndentOf {
            amount: LengthPercentageOf::px(S::from_f64(indent)).unwrap(),
            each_line: false,
            hanging: false,
        },
        direction: if rtl { Direction::Rtl } else { Direction::Ltr },
        ..NodeInputOf::default()
    }
}
fn compute<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: AvailableOf<S>,
    policy: NodeInputOf<S>,
    float: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let binding = p.binding(0);
    let mut children = binding.nodes.clone();
    if float {
        children.insert(0, 9);
    }
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, children)
        .style(0, policy.clone());
    for origin in &binding.origins {
        tree = tree
            .children(origin.node, [])
            .input(origin.node, p.text_input(origin.node));
    }
    for control in &binding.controls {
        tree = tree.children(control.node, []).input(
            control.node,
            LayoutInputOf::text_control(TextControlParticipation::new(control.occurrence.clone())),
        );
    }
    if float {
        tree = tree.children(9, []).style(
            9,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::px(S::from_f64(100.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MaxContent)).unwrap(),
    )
    .unwrap()
}
fn lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &PreparedParagraph,
    expected_font: &[u8],
) -> Vec<surgeist_text::FinalizedLine> {
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        p.prepared.analyzed_source().source().text().code_units()
    );
    let mut result = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        result[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    result
        .into_iter()
        .map(|line| {
            let line = line.unwrap();
            assert!(
                line.run_views()
                    .all(|run| run.face().bytes() == expected_font
                        && run.effective_instance().unwrap().font_size() == 10.0)
            );
            line
        })
        .collect()
}
fn values<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &PreparedParagraph,
    expected_font: &[u8],
) -> Vec<(usize, f64, bool)> {
    lines(batch, p, expected_font)
        .into_iter()
        .map(|line| {
            let s = line.selected_line();
            (
                s.request().end.view().source_before().get(),
                line.feedback().advance,
                s.request().alternative.is_some(),
            )
        })
        .collect()
}
fn qualify(p: &PreparedParagraph) {
    let word = p
        .prepared
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    assert_eq!(word.resource(), HyphenationResource::EnUs2005);
    assert_eq!(
        word.provenance(),
        Some(HyphenationProvenance::SourceException { entry: "associate" })
    );
    assert_eq!(
        word.cuts()
            .map(|cut| {
                assert!(cut.opportunity().is_some());
                (
                    cut.boundary().view().source_before().get(),
                    cut.counts().unwrap(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                4,
                HyphenationCounts {
                    total: 9,
                    before: 2,
                    after: 7
                }
            ),
            (
                6,
                HyphenationCounts {
                    total: 9,
                    before: 4,
                    after: 5
                }
            )
        ]
    );
    let start = p.prepared.boundary_views().next().unwrap().handle();
    assert!(
        word.ordinary_fallbacks(&start)
            .unwrap()
            .iter()
            .all(|offer| offer.view().boundary().view().source_before().get() == 2)
    );
    assert!(!word.ordinary_fallbacks(&start).unwrap().is_empty());
}
#[test]
fn zone_strict_gap_uses_discarded_or_hanging_space_and_gates_automatic_empty_only() {
    fn check<S: LayoutScalar>() {
        for white in [
            WhiteSpace::Collapse,
            WhiteSpace::Preserve,
            WhiteSpace::PreserveSpaces,
        ] {
            let p = paragraph(white, "A");
            qualify(&p);
            for (zone, allow) in [
                (-5.0, true),
                (45.0, true),
                (49.0, true),
                (50.0, false),
                (51.0, false),
                (1000.0, false),
            ] {
                let b = compute::<S>(
                    &p,
                    AvailableOf::definite(S::from_f64(60.0)),
                    input(zone, 0.0, 0.0, false),
                    false,
                );
                let first = if white == WhiteSpace::Collapse {
                    10.0
                } else {
                    20.0
                };
                assert_eq!(
                    values(&b, &p, AHEM),
                    if allow {
                        vec![(4, 50.0, true), (6, 30.0, true), (11, 50.0, false)]
                    } else {
                        vec![(2, first, false), (6, 50.0, true), (11, 50.0, false)]
                    },
                    "{white:?} {zone}"
                );
                if !allow {
                    assert_eq!(
                        lines(&b, &p, AHEM)[0].measurements().alignment().advance(),
                        10.0
                    );
                }
            }
        }
        let p = paragraph(WhiteSpace::Collapse, "");
        for (zone, expected) in [
            (45.0, vec![(6, 60.0, true), (11, 50.0, false)]),
            (
                50.0,
                vec![(2, 10.0, false), (6, 40.0, true), (11, 50.0, false)],
            ),
        ] {
            let b = compute::<S>(
                &p,
                AvailableOf::definite(S::from_f64(60.0)),
                input(zone, 0.0, 0.0, false),
                false,
            );
            assert_eq!(values(&b, &p, AHEM), expected);
            assert!(matches!(
                lines(&b, &p, AHEM)
                    .iter()
                    .find(|line| line.selected_line().request().alternative.is_some())
                    .unwrap()
                    .selected_line()
                    .request()
                    .alternative
                    .as_ref()
                    .unwrap()
                    .view()
                    .definition(),
                PreparedAlternativeDefinition::Automatic { .. }
            ));
        }
        let mut manual = style(WhiteSpace::Collapse, "");
        manual.hyphens = Hyphens::Manual;
        let source = PreparedParagraph::plain(1, Source::new("X as\u{ad}sociate"), manual.clone());
        let b = compute::<S>(
            &source,
            AvailableOf::definite(S::from_f64(60.0)),
            input(1000.0, 0.0, 0.0, false),
            false,
        );
        assert_eq!(
            values(&b, &source, AHEM),
            [(5, 40.0, true), (12, 70.0, false)]
        );
        assert!(
            matches!(lines(&b,&source, AHEM)[0].selected_line().request().alternative.as_ref().unwrap().view().definition(),PreparedAlternativeDefinition::Source{source,..} if source==Range::new(4,5))
        );
        let control = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("X as"), manual.clone()),
                ParagraphPart::Control {
                    node: 2,
                    kind: ParagraphControlKind::DiscretionaryBreak {
                        style: Box::new(manual.clone()),
                    },
                },
                ParagraphPart::text(3, Source::new("sociate"), manual.clone()),
            ],
            manual,
            ParagraphBaseDirection::LeftToRight,
        );
        let b = compute::<S>(
            &control,
            AvailableOf::definite(S::from_f64(60.0)),
            input(1000.0, 0.0, 0.0, false),
            false,
        );
        assert_eq!(
            values(&b, &control, AHEM),
            [(4, 40.0, true), (11, 70.0, false)]
        );
        assert!(matches!(
            lines(&b, &control, AHEM)[0]
                .selected_line()
                .request()
                .alternative
                .as_ref()
                .unwrap()
                .view()
                .definition(),
            PreparedAlternativeDefinition::Control { .. }
        ));
    }
    check::<f32>();
    check::<f64>();
}
#[test]
fn zone_signed_indent_and_percentage_use_full_actual_float_band_in_both_directions() {
    fn check<S: LayoutScalar>() {
        let p = paragraph(WhiteSpace::Collapse, "A");
        for (indent, zone, cut, advance) in [
            (-10.0, 55.0, 6, 70.0),
            (10.0, 45.0, 2, 10.0),
            (10.0, 35.0, 4, 50.0),
        ] {
            let b = compute::<S>(
                &p,
                AvailableOf::definite(S::from_f64(60.0)),
                input(zone, 0.0, indent, false),
                false,
            );
            assert_eq!(values(&b, &p, AHEM)[0], (cut, advance, cut != 2));
        }
        for rtl in [false, true] {
            let style = style(WhiteSpace::Collapse, "A");
            let p = PreparedParagraph::new(
                vec![ParagraphPart::text(
                    1,
                    Source::new("X associate"),
                    style.clone(),
                )],
                style,
                if rtl {
                    ParagraphBaseDirection::RightToLeft
                } else {
                    ParagraphBaseDirection::LeftToRight
                },
            );
            for (fraction, cut, advance) in [(0.7, 6, 70.0), (0.75, 2, 10.0), (0.8, 2, 10.0)] {
                let b = compute::<S>(
                    &p,
                    AvailableOf::definite(S::from_f64(100.0)),
                    input(0.0, fraction, 10.0, rtl),
                    true,
                );
                assert_eq!(
                    values(&b, &p, AHEM)[0],
                    (cut, advance, cut != 2),
                    "rtl={rtl} fraction={fraction}"
                );
                let float = b
                    .final_entries()
                    .iter()
                    .find(|entry| entry.node() == 9)
                    .unwrap()
                    .output();
                assert_eq!(
                    float.location,
                    Point::new(S::from_f64(if rtl { 80.0 } else { 0.0 }), S::ZERO)
                );
                assert_eq!(float.size, Size::new(S::from_f64(20.0), S::from_f64(100.0)));
            }
        }
    }
    check::<f32>();
    check::<f64>();
}
#[test]
fn zone_is_inapplicable_without_ordinary_prefix_and_intrinsics_ignore_it() {
    fn check<S: LayoutScalar>() {
        let p = PreparedParagraph::plain(
            1,
            Source::new("associate"),
            style(WhiteSpace::Collapse, "A"),
        );
        let b = compute::<S>(
            &p,
            AvailableOf::definite(S::from_f64(30.0)),
            input(1000.0, 0.0, 0.0, false),
            false,
        );
        assert_eq!(
            values(&b, &p, AHEM),
            [(2, 30.0, true), (4, 30.0, true), (9, 50.0, false)]
        );
        let p = paragraph(WhiteSpace::Collapse, "A");
        for zone in [0.0, 45.0, 1000.0] {
            for limit in [HyphenateLimitLines::NoLimit, HyphenateLimitLines::Count(0)] {
                let mut policy = input(zone, 0.0, 0.0, false);
                policy.hyphenate_limit_lines = limit;
                let prior = values(
                    &compute::<S>(
                        &p,
                        AvailableOf::definite(S::from_f64(60.0)),
                        policy.clone(),
                        false,
                    ),
                    &p,
                    AHEM,
                );
                for minimum in [true, false, false, true] {
                    let b = compute::<S>(
                        &p,
                        if minimum {
                            AvailableOf::MinContent
                        } else {
                            AvailableOf::MaxContent
                        },
                        policy.clone(),
                        false,
                    );
                    let width = b
                        .unrounded_entries()
                        .iter()
                        .find(|entry| entry.node() == 0)
                        .unwrap()
                        .output()
                        .size
                        .width
                        .to_f64();
                    assert_eq!(
                        width,
                        if !minimum {
                            110.0
                        } else if limit == HyphenateLimitLines::Count(0) {
                            90.0
                        } else {
                            50.0
                        }
                    );
                }
                assert_eq!(
                    values(
                        &compute::<S>(&p, AvailableOf::definite(S::from_f64(60.0)), policy, false),
                        &p,
                        AHEM,
                    ),
                    prior
                );
            }
        }
    }
    check::<f32>();
    check::<f64>();
}
#[test]
fn negative_zone_is_not_clamped_at_a_genuine_zero_gap() {
    fn check<S: LayoutScalar>() {
        let font = super::canonical::fonts::positioned_ahem(&[
            (58, -1000),
            (3, -1000),
            (67, -1000),
            (85, -1000),
        ]);
        let mut normal = style(WhiteSpace::Collapse, "");
        normal.font = normal.font.feature("\"kern\" 0");
        let mut zero = normal.clone();
        zero.font = surgeist_text::Font::new()
            .family("Ahem")
            .feature("\"kern\" 1");
        let p = PreparedParagraph::with_registered_font(
            vec![
                ParagraphPart::text(1, Source::new("X as"), zero),
                ParagraphPart::text(2, Source::new("sociate"), normal.clone()),
            ],
            normal,
            ParagraphBaseDirection::LeftToRight,
            &font,
        );
        let whole = crate::test_support::text_fragment::fragment_for(&p.prepared).finalized_line();
        let actual = whole
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), font);
                run.glyphs()
                    .map(|g| (g.range(), g.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 11);
        for (range, advance) in actual {
            assert_eq!(advance, if range.end <= 4 { 0.0 } else { 10.0 });
        }
        for (zone, end, automatic) in [(0.0, 2, false), (-1.0, 4, true)] {
            let b = compute::<S>(
                &p,
                AvailableOf::definite(S::ZERO),
                input(zone, 0.0, 0.0, false),
                false,
            );
            assert_eq!(values(&b, &p, &font)[0], (end, 0.0, automatic));
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn zone_reference_is_natural_even_when_the_committed_fallback_is_justified() {
    fn check<S: LayoutScalar>() {
        let p = PreparedParagraph::plain(
            1,
            Source::new("X Y associate"),
            style(WhiteSpace::Collapse, "A"),
        );
        // Genuine fallback X SPACE Y: natural alignment30 in span100 gives U70.
        // Its actual justified line expands its retained interior space to100.
        for (zone, end, automatic) in [(60.0, 8, true), (70.0, 4, false)] {
            let mut policy = input(zone, 0.0, 0.0, false);
            policy.text_align = TextAlign::Justify;
            policy.text_justification = TextJustification::Spaces;
            let b = compute::<S>(&p, AvailableOf::definite(S::from_f64(100.0)), policy, false);
            let rows = lines(&b, &p, AHEM);
            let first = &rows[0];
            assert_eq!(
                first
                    .selected_line()
                    .request()
                    .end
                    .view()
                    .source_before()
                    .get(),
                end
            );
            assert_eq!(
                first.selected_line().request().alternative.is_some(),
                automatic
            );
            assert_eq!(first.measurements().alignment().advance(), 100.0);
            assert_eq!(first.feedback().advance, 100.0);
            assert!(first.spacing_slots().any(|slot| matches!(
                slot.disposition,
                surgeist_text::SpacingSlotDisposition::Retained { .. }
            ) && slot.advance > 10.0));
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn zone_resolution_is_consumed_only_by_a_real_fallback_and_preserves_numeric_failure() {
    type ProbeResult = Result<CompletedLayoutBatchOf<u32, f64>, Box<LayoutErrorOf<u32, f64>>>;
    fn run(p: &PreparedParagraph) -> ProbeResult {
        let policy = input::<f64>(0.0, f64::MAX, 0.0, false);
        let tree = PublicLayoutTreeOf::<f64>::new()
            .children(0, [1])
            .children(1, [])
            .style(0, policy)
            .input(1, p.text_input(1));
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![p.binding(0)],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        compute_layout_with_flow(&capture, 0, request(60.0)).map_err(Box::new)
    }
    // No genuine before-word line: the huge but finite coefficient is unused.
    let p = PreparedParagraph::plain(
        1,
        Source::new("associate"),
        style(WhiteSpace::Collapse, "A"),
    );
    assert_eq!(
        values(&run(&p).unwrap(), &p, AHEM),
        [(4, 50.0, true), (9, 50.0, false)]
    );
    let p = paragraph(WhiteSpace::Collapse, "A");
    qualify(&p);
    let error = run(&p).unwrap_err();
    assert!(
        matches!(error.kind(),LayoutErrorKindOf::TextNumeric {value,reason:LayoutTextNumericError::NonFinite} if value.is_infinite())
    );
}
