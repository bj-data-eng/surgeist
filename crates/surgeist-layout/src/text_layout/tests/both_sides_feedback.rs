//! Explicit native BothSides facts, existing registered Ahem and real en-US data.
use super::*;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn style(hyphens: text::Hyphens) -> text::Style {
    text::Style {
        hyphens,
        locale: Some("en-US".into()),
        white_space: text::WhiteSpace::Collapse,
        overflow_wrap: text::OverflowWrap::Normal,
        hyphenate_character: text::HyphenateCharacter::Auto,
        native_hyphenation_convention: Some(
            text::NativeHyphenationConvention::language_script(
                "en-US",
                "Latn",
                "A".into(),
                text::HyphenationPlacement::BothSides,
            )
            .unwrap(),
        ),
        ..ahem_style(10.0, 10.0)
    }
}

fn block<S: LayoutScalar>(
    width: f64,
    last: HyphenateLimitLast,
    limit: HyphenateLimitLines,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::AUTO,
        ),
        hyphenate_limit_last: last,
        hyphenate_limit_lines: limit,
        ..NodeInputOf::default()
    }
}

fn host<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: f64,
    last: HyphenateLimitLast,
    limit: HyphenateLimitLines,
) -> CachedHost<S> {
    CachedHost::new(
        PublicLayoutTreeOf::new()
            .children(2, [10])
            .children(10, [])
            .style(2, block(width, last, limit))
            .input(10, p.text_input(10)),
    )
}

fn sequence<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    expected: &[(usize, usize, f64, bool)],
) -> Vec<text::FinalizedLine> {
    let layout = batch.placed_text()[0].layout();
    let mut actual = vec![None; layout.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        if let Some(previous) = &actual[fragment.line_index()] {
            assert_eq!(previous, &line);
        }
        actual[fragment.line_index()] = Some(line);
    }
    let actual = actual.into_iter().map(Option::unwrap).collect::<Vec<_>>();
    assert_eq!(actual.len(), expected.len());
    let mut incoming = None;
    for (index, (line, &(start, end, advance, ending))) in actual.iter().zip(expected).enumerate() {
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(request.incoming, incoming);
        assert_eq!(request.alternative.is_some(), ending);
        assert_eq!(layout.lines()[index].range(), text::Range::new(start, end));
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().occupied_start,
                line.feedback().occupied_end
            ),
            (advance, 0.0, advance)
        );
        assert_eq!(
            layout.source().text().code_units(),
            line.prepared_text()
                .analyzed_source()
                .source()
                .text()
                .code_units()
        );
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(
                run.face().catalogue_identity(),
                line.prepared_text().font_catalogue_identity()
            );
            let instance = run.effective_instance().unwrap();
            assert!(instance.face().is_same_face(&run.face()));
            assert_eq!(instance.font_size(), 10.0);
        }
        let mut owners = Vec::new();
        if let Some(previous) = &incoming {
            owners.push(previous.alternative());
        }
        if let Some(outgoing) = &request.alternative {
            owners.push(outgoing.clone());
        }
        let actual_owners = line
            .fragment_views()
            .filter_map(|row| match row.owner() {
                text::FinalizedFragmentOwner::Alternative(owner) => Some(owner),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_owners.len(), owners.len());
        assert!(owners.iter().all(|owner| actual_owners.contains(owner)));
        incoming = line.hyphenation_continuation();
        assert_eq!(incoming.is_some(), ending);
        if let Some(pending) = &incoming {
            assert_eq!(Some(pending.alternative()), request.alternative);
        }
    }
    assert!(incoming.is_none());
    actual
}

fn intrinsic<S: LayoutScalar>(
    p: &PreparedParagraph,
    limit: HyphenateLimitLines,
    available: AvailableOf<S>,
    expected: f64,
) {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(2, [10])
        .children(10, [])
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                hyphenate_limit_lines: limit,
                ..NodeInputOf::default()
            },
        )
        .input(10, p.text_input(10));
    let captured = TextLayoutInput::try_new(&tree, 2, vec![p.binding(2)]).unwrap();
    let input =
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap();
    let batch = compute_layout_with_text(&captured, 2, input).unwrap();
    assert_eq!(output(&batch, 2).size.width, S::from_f64(expected));
}

fn automatic_priority<S: LayoutScalar>() {
    let p = PreparedParagraph::plain(
        10,
        Source::new("as\u{ad}sociate"),
        style(text::Hyphens::Auto),
    );
    assert_eq!(
        p.prepared.analyzed_source().source().text().code_units(),
        "as\u{ad}sociate".encode_utf16().collect::<Vec<_>>()
    );
    let word = p
        .prepared
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    assert_eq!(word.resource(), text::HyphenationResource::EnUs2005);
    assert_eq!(
        word.provenance(),
        Some(text::HyphenationProvenance::SourceException { entry: "associate" })
    );
    let manual = p
        .prepared
        .alternative_views()
        .find(|row| {
            matches!(
                row.definition(),
                text::PreparedAlternativeDefinition::Source { .. }
            )
        })
        .unwrap()
        .handle();
    let auto = p
        .prepared
        .alternative_views()
        .find(|row| match row.definition() {
            text::PreparedAlternativeDefinition::Automatic { cut } => {
                cut.view().counts()
                    == Some(text::HyphenationCounts {
                        total: 9,
                        before: 4,
                        after: 5,
                    })
            }
            _ => false,
        })
        .unwrap()
        .handle();

    // Ahem10, real 9-letter word. The whole manual residual is A+sociate80.
    // The finite chooser selects one furthest-fitting Auto4 ending, not all
    // intrinsic pieces: asso+A50 followed by A+ciate60.
    for (width, expected, ending) in [
        (
            80.0,
            vec![(0, 3, 30.0, true), (3, 10, 80.0, false)],
            &manual,
        ),
        (75.0, vec![(0, 5, 50.0, true), (5, 10, 60.0, false)], &auto),
        (60.0, vec![(0, 5, 50.0, true), (5, 10, 60.0, false)], &auto),
    ] {
        let batch = host::<S>(
            &p,
            width,
            HyphenateLimitLast::None,
            HyphenateLimitLines::NoLimit,
        )
        .compute(&p.binding(2), &[])
        .unwrap();
        let lines = sequence(&batch, &expected);
        assert_eq!(
            lines[0].selected_line().request().alternative.as_ref(),
            Some(ending)
        );
    }

    // Intrinsics refine the retained manual portions30/40/60 independently of
    // finite residual priority. Query order cannot consume committed capacity.
    for limit in [
        HyphenateLimitLines::Count(0),
        HyphenateLimitLines::Count(1),
        HyphenateLimitLines::NoLimit,
        HyphenateLimitLines::Count(0),
    ] {
        intrinsic::<S>(
            &p,
            limit,
            AvailableOf::MIN_CONTENT,
            if limit == HyphenateLimitLines::Count(0) {
                80.0
            } else {
                60.0
            },
        );
        intrinsic::<S>(&p, limit, AvailableOf::MAX_CONTENT, 90.0);
        let finite = host::<S>(
            &p,
            75.0,
            HyphenateLimitLast::None,
            HyphenateLimitLines::Count(1),
        )
        .compute(&p.binding(2), &[])
        .unwrap();
        let rows = sequence(&finite, &[(0, 5, 50.0, true), (5, 10, 60.0, false)]);
        assert_eq!(
            rows[0].selected_line().request().alternative.as_ref(),
            Some(&auto)
        );
    }
    let zero = host::<S>(
        &p,
        75.0,
        HyphenateLimitLast::None,
        HyphenateLimitLines::Count(0),
    )
    .compute(&p.binding(2), &[])
    .unwrap();
    let rows = sequence(&zero, &[(0, 3, 30.0, true), (3, 10, 80.0, false)]);
    assert_eq!(
        rows[0].selected_line().request().alternative.as_ref(),
        Some(&manual)
    );
    let manual_only = PreparedParagraph::plain(
        10,
        Source::new("as\u{ad}sociate"),
        style(text::Hyphens::Manual),
    );
    intrinsic::<S>(
        &manual_only,
        HyphenateLimitLines::NoLimit,
        AvailableOf::MIN_CONTENT,
        80.0,
    );
}

fn committed_counter<S: LayoutScalar>() {
    let p = PreparedParagraph::plain(10, Source::new("cooperate"), style(text::Hyphens::Auto));
    let word = p
        .prepared
        .word_views()
        .find(|word| word.query() == Some("cooperate"))
        .unwrap();
    assert_eq!(word.resource(), text::HyphenationResource::EnUs2005);
    assert_eq!(
        p.prepared
            .hyphenation_cut_views()
            .map(|cut| {
                assert_eq!(cut.provenance(), text::HyphenationProvenance::Pattern);
                cut.counts().unwrap()
            })
            .collect::<Vec<_>>(),
        [2, 4, 6].map(|before| text::HyphenationCounts {
            total: 9,
            before,
            after: 9 - before
        })
    );
    // First co+A30; each middle A+two originals+A40. Receiving alone is not
    // another ending, and rejected farther offers do not consume the limit.
    for (limit, expected) in [
        (HyphenateLimitLines::Count(0), vec![(0, 9, 90.0, false)]),
        (
            HyphenateLimitLines::Count(1),
            vec![(0, 2, 30.0, true), (2, 9, 80.0, false)],
        ),
        (
            HyphenateLimitLines::Count(2),
            vec![(0, 2, 30.0, true), (2, 4, 40.0, true), (4, 9, 60.0, false)],
        ),
        (
            HyphenateLimitLines::NoLimit,
            vec![
                (0, 2, 30.0, true),
                (2, 4, 40.0, true),
                (4, 6, 40.0, true),
                (6, 9, 40.0, false),
            ],
        ),
    ] {
        let batch = host::<S>(&p, 40.0, HyphenateLimitLast::None, limit)
            .compute(&p.binding(2), &[])
            .unwrap();
        sequence(&batch, &expected);
    }
}

fn tail_cache<S: LayoutScalar>() {
    let p = PreparedParagraph::plain(10, Source::new("AB\u{ad}CDE"), style(text::Hyphens::Manual));
    let binding = p.binding(2);
    let mut current = host::<S>(
        &p,
        40.0,
        HyphenateLimitLast::None,
        HyphenateLimitLines::NoLimit,
    );
    let cold = current.compute(&binding, &[]).unwrap();
    sequence(&cold, &[(0, 3, 30.0, true), (3, 6, 40.0, false)]);
    cold.apply_to(&mut current).unwrap();
    let warm = current.compute(&binding, &[]).unwrap();
    super::conditional_cache::assert_same(&cold, &warm);
    warm.apply_to(&mut current).unwrap();
    for last in [
        HyphenateLimitLast::Always,
        HyphenateLimitLast::None,
        HyphenateLimitLast::Always,
    ] {
        current.tree.insert_input(
            2,
            LayoutInputOf::box_input(block(40.0, last, HyphenateLimitLines::NoLimit)),
        );
        let changed = current.compute(&binding, &[2]).unwrap();
        assert_eq!(
            changed.placed_text()[0]
                .layout()
                .source()
                .text()
                .code_units(),
            [65, 66, 173, 67, 68, 69]
        );
        if last == HyphenateLimitLast::Always {
            let rows = sequence(&changed, &[(0, 6, 50.0, false)]);
            assert!(
                rows[0].fragment_views().all(|row| !matches!(
                    row.owner(),
                    text::FinalizedFragmentOwner::Alternative(_)
                ))
            );
        } else {
            sequence(&changed, &[(0, 3, 30.0, true), (3, 6, 40.0, false)]);
        }
        let fresh = CachedHost::new(current.tree.clone())
            .compute(&binding, &[])
            .unwrap();
        super::conditional_cache::assert_same(&changed, &fresh);
        changed.apply_to(&mut current).unwrap();
    }
}

fn taller_prefix_float<S: LayoutScalar>() {
    let base = style(text::Hyphens::Manual);
    let tall = text::Style {
        line_height: text::LineHeight::Absolute(30.0),
        ..base.clone()
    };
    let mut ending = Source::new("AB\u{ad}");
    ending.span(text::Range::new(0, 1), tall.clone());
    ending.span(text::Range::new(2, 3), tall);
    let p = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, ending, base.clone()),
            ParagraphPart::text(11, Source::new("CDE"), base.clone()),
        ],
        base,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = p.binding(2);
    binding.nodes.push(30);
    let mut current = host::<S>(
        &p,
        40.0,
        HyphenateLimitLast::None,
        HyphenateLimitLines::NoLimit,
    );
    current.tree = current
        .tree
        .children(2, [10, 11, 30])
        .children(11, [])
        .input(11, p.text_input(11))
        .children(30, [])
        .style(
            30,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(8.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let cold = current.compute(&binding, &[]).unwrap();
    let rows = sequence(&cold, &[(0, 3, 30.0, true), (3, 6, 40.0, false)]);
    for line in rows {
        // Ahem10 ascent8/descent2 plus equal10px half-leading.
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (18.0, 30.0)
        );
    }
    assert_eq!(cold.placed_text()[0].layout().lines()[1].baseline(), 48.0);
    assert!(
        cold.final_inline_fragments()
            .iter()
            .any(|row| row.node() == 10
                && row.fragment().line_index() == 1
                && matches!(
                    row.fragment().finalized_fragment().view().owner(),
                    text::FinalizedFragmentOwner::Alternative(_)
                ))
    );
    assert!(
        cold.final_inline_fragments()
            .iter()
            .any(|row| row.node() == 11 && row.fragment().line_index() == 1)
    );
    // The trailing float is encountered only after CDE. Receiver A+CDE40 plus
    // float8 cannot share width40, so two30px lines reserve y60. Without the
    // prefix, CDE30+float8 would fit beside that line; no guessed band is used.
    assert_eq!(
        output(&cold, 30).location,
        Point::new(S::ZERO, S::from_f64(60.0))
    );
    cold.apply_to(&mut current).unwrap();
    let warm = current.compute(&binding, &[]).unwrap();
    super::conditional_cache::assert_same(&cold, &warm);
    warm.apply_to(&mut current).unwrap();
    current.tree.insert_input(
        2,
        LayoutInputOf::box_input(block(
            40.0,
            HyphenateLimitLast::Always,
            HyphenateLimitLines::NoLimit,
        )),
    );
    let withdrawn = current.compute(&binding, &[2]).unwrap();
    sequence(&withdrawn, &[(0, 6, 50.0, false)]);
    assert_eq!(
        output(&withdrawn, 30).location,
        Point::new(S::ZERO, S::from_f64(30.0))
    );
    let fresh = CachedHost::new(current.tree.clone())
        .compute(&binding, &[])
        .unwrap();
    super::conditional_cache::assert_same(&withdrawn, &fresh);
}

#[test]
fn native_both_resource_priority_and_intrinsics_f32() {
    automatic_priority::<f32>();
}
#[test]
fn native_both_resource_priority_and_intrinsics_f64() {
    automatic_priority::<f64>();
}
#[test]
fn native_both_counter_tracks_outgoing_endings_only_f32() {
    committed_counter::<f32>();
}
#[test]
fn native_both_counter_tracks_outgoing_endings_only_f64() {
    committed_counter::<f64>();
}
#[test]
fn native_both_tail_withdraws_both_emissions_from_warm_cache_f32() {
    tail_cache::<f32>();
}
#[test]
fn native_both_tail_withdraws_both_emissions_from_warm_cache_f64() {
    tail_cache::<f64>();
}
#[test]
fn native_both_taller_prefix_drives_real_float_feedback_f32() {
    taller_prefix_float::<f32>();
}
#[test]
fn native_both_taller_prefix_drives_real_float_feedback_f64() {
    taller_prefix_float::<f64>();
}
