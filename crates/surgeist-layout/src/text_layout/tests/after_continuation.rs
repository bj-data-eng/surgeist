//! Continuations participate in actual choice, intrinsic and rollback contexts.
use super::*;

fn style(hyphens: text::Hyphens, overflow: text::OverflowWrap) -> text::Style {
    text::Style {
        hyphens,
        locale: Some("en-US".into()),
        overflow_wrap: overflow,
        white_space: text::WhiteSpace::Collapse,
        native_hyphenation_convention: Some(
            text::NativeHyphenationConvention::language_script(
                "en-US",
                "Latn",
                text::Text::from("A"),
                text::HyphenationPlacement::LogicalAfter,
            )
            .unwrap(),
        ),
        ..ahem_style(10.0, 10.0)
    }
}

fn root<S: LayoutScalar>(
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

fn tree<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: f64,
    last: HyphenateLimitLast,
    limit: HyphenateLimitLines,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(2, [10])
        .children(10, [])
        .style(2, root(width, last, limit))
        .input(10, p.text_input(10))
}

fn finalized<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut rows = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        if let Some(previous) = &rows[fragment.line_index()] {
            assert_eq!(previous, &line);
        }
        rows[fragment.line_index()] = Some(line);
    }
    rows.into_iter().map(Option::unwrap).collect()
}

fn sequence<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    expected: &[(usize, usize, f64, bool)],
) {
    let actual = finalized(batch);
    assert_eq!(actual.len(), expected.len());
    let mut pending = None;
    for (index, (line, &(start, end, advance, hyphenated))) in
        actual.iter().zip(expected).enumerate()
    {
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(request.incoming, pending);
        assert_eq!(request.alternative.is_some(), hyphenated);
        assert_eq!(line.feedback().advance, advance);
        assert_eq!(
            (line.feedback().occupied_start, line.feedback().occupied_end),
            (0.0, advance)
        );
        assert_eq!(
            batch.placed_text()[0].layout().lines()[index].range(),
            text::Range::new(start, end)
        );
        pending = line.hyphenation_continuation();
        assert_eq!(pending.is_some(), hyphenated);
    }
    assert!(
        pending.is_none(),
        "the complete paragraph discharges its outgoing obligation"
    );
}

fn compute<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: f64,
    last: HyphenateLimitLast,
    limit: HyphenateLimitLines,
) -> CompletedLayoutBatchOf<u32, S> {
    CachedHost::new(tree(p, width, last, limit))
        .compute(&p.binding(2), &[])
        .unwrap()
}

fn intrinsic<S: LayoutScalar>(p: &PreparedParagraph, available: AvailableOf<S>, expected: f64) {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(2, [10])
        .children(10, [])
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(10, p.text_input(10));
    let captured = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![p.binding(2)],
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let request =
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap();
    let batch = compute_layout_with_flow(&captured, 2, request).unwrap();
    assert_eq!(output(&batch, 2).size.width, S::from_f64(expected));
}

fn automatic_priority<S: LayoutScalar>() {
    let p = PreparedParagraph::plain(
        10,
        Source::new("as\u{ad}sociate"),
        style(text::Hyphens::Auto, text::OverflowWrap::Normal),
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
        .find(|a| {
            matches!(
                a.definition(),
                text::PreparedAlternativeDefinition::Source { .. }
            )
        })
        .unwrap()
        .handle();
    for width in [75.0, 60.0, 80.0, 75.0] {
        let batch = compute::<S>(
            &p,
            width,
            HyphenateLimitLast::None,
            HyphenateLimitLines::NoLimit,
        );
        if width == 80.0 {
            sequence(&batch, &[(0, 3, 20.0, true), (3, 10, 80.0, false)]);
            assert_eq!(
                finalized(&batch)[0]
                    .selected_line()
                    .request()
                    .alternative
                    .as_ref(),
                Some(&manual)
            );
        } else {
            sequence(&batch, &[(0, 5, 40.0, true), (5, 10, 60.0, false)]);
            let ending = finalized(&batch)[0]
                .selected_line()
                .request()
                .alternative
                .clone()
                .unwrap();
            let text::PreparedAlternativeDefinition::Automatic { cut } = ending.view().definition()
            else {
                panic!("actual residual overflow admits the automatic ending")
            };
            assert_eq!(
                cut.view().counts().unwrap(),
                text::HyphenationCounts {
                    total: 9,
                    before: 4,
                    after: 5
                }
            );
        }
    }
    intrinsic::<S>(&p, AvailableOf::MIN_CONTENT, 60.0);
    intrinsic::<S>(&p, AvailableOf::MAX_CONTENT, 90.0);
    let manual_only = PreparedParagraph::plain(
        10,
        Source::new("as\u{ad}sociate"),
        style(text::Hyphens::Manual, text::OverflowWrap::Normal),
    );
    intrinsic::<S>(&manual_only, AvailableOf::MIN_CONTENT, 80.0);
}

#[test]
fn after_manual_residual_controls_actual_auto_priority_and_intrinsics_f32() {
    automatic_priority::<f32>();
}
#[test]
fn after_manual_residual_controls_actual_auto_priority_and_intrinsics_f64() {
    automatic_priority::<f64>();
}

fn committed_count<S: LayoutScalar>() {
    let p = PreparedParagraph::plain(
        10,
        Source::new("cooperate"),
        style(text::Hyphens::Auto, text::OverflowWrap::Normal),
    );
    assert_eq!(
        p.prepared
            .hyphenation_cut_views()
            .map(|cut| cut.counts().unwrap().before)
            .collect::<Vec<_>>(),
        [2, 4, 6]
    );
    // An incoming A is not a second ending. Every Auto/portion trial remains
    // pure: only the accepted outgoing alternative advances the block counter.
    for (limit, expected) in [
        (
            HyphenateLimitLines::Count(1),
            vec![(0, 2, 20.0, true), (2, 9, 80.0, false)],
        ),
        (
            HyphenateLimitLines::Count(2),
            vec![(0, 2, 20.0, true), (2, 4, 30.0, true), (4, 9, 60.0, false)],
        ),
        (
            HyphenateLimitLines::NoLimit,
            vec![
                (0, 2, 20.0, true),
                (2, 4, 30.0, true),
                (4, 6, 30.0, true),
                (6, 9, 40.0, false),
            ],
        ),
    ] {
        sequence(
            &compute::<S>(&p, 30.0, HyphenateLimitLast::None, limit),
            &expected,
        );
    }
}
#[test]
fn after_receiving_prefix_does_not_commit_an_extra_hyphenated_ending_f32() {
    committed_count::<f32>();
}
#[test]
fn after_receiving_prefix_does_not_commit_an_extra_hyphenated_ending_f64() {
    committed_count::<f64>();
}

fn tail_rollback<S: LayoutScalar>() {
    for overflow in [text::OverflowWrap::Normal, text::OverflowWrap::Anywhere] {
        let p = PreparedParagraph::plain(
            10,
            Source::new("as\u{ad}sociate"),
            style(text::Hyphens::Manual, overflow),
        );
        let binding = p.binding(2);
        let mut host = CachedHost::new(tree::<S>(
            &p,
            80.0,
            HyphenateLimitLast::None,
            HyphenateLimitLines::NoLimit,
        ));
        let cold = host.compute(&binding, &[]).unwrap();
        sequence(&cold, &[(0, 3, 20.0, true), (3, 10, 80.0, false)]);
        cold.apply_to(&mut host).unwrap();
        for last in [
            HyphenateLimitLast::Always,
            HyphenateLimitLast::None,
            HyphenateLimitLast::Always,
        ] {
            host.tree.insert_input(
                2,
                LayoutInputOf::box_input(root(80.0, last, HyphenateLimitLines::NoLimit)),
            );
            let changed = host.compute(&binding, &[2]).unwrap();
            if last == HyphenateLimitLast::None {
                sequence(&changed, &[(0, 3, 20.0, true), (3, 10, 80.0, false)]);
            } else if overflow == text::OverflowWrap::Anywhere {
                sequence(&changed, &[(0, 9, 80.0, false), (9, 10, 10.0, false)]);
            } else {
                sequence(&changed, &[(0, 10, 90.0, false)]);
            }
            let fresh = CachedHost::new(host.tree.clone())
                .compute(&binding, &[])
                .unwrap();
            super::conditional_cache::assert_same(&changed, &fresh);
            changed.apply_to(&mut host).unwrap();
        }
    }
}
#[test]
fn tail_reflow_withdraws_after_obligation_and_all_receiving_cache_geometry_f32() {
    tail_rollback::<f32>();
}
#[test]
fn tail_reflow_withdraws_after_obligation_and_all_receiving_cache_geometry_f64() {
    tail_rollback::<f64>();
}

fn prefix_float_feedback<S: LayoutScalar>() {
    let base = style(text::Hyphens::Manual, text::OverflowWrap::Normal);
    let tall = text::Style {
        line_height: text::LineHeight::Absolute(30.0),
        ..base.clone()
    };
    let mut source = Source::new("AB\u{ad}");
    // Original A independently makes the ending line30px. Only the actual
    // indication's governing SHY style can make the receiving CDE line30px.
    source.span(text::Range::new(0, 1), tall.clone());
    source.span(text::Range::new(2, 3), tall);
    let p = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, source, base.clone()),
            ParagraphPart::text(11, Source::new("CDE"), base.clone()),
        ],
        base,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = p.binding(2);
    binding.nodes.push(30);
    let tree = tree::<S>(
        &p,
        40.0,
        HyphenateLimitLast::None,
        HyphenateLimitLines::NoLimit,
    )
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
    let mut host = CachedHost::new(tree);
    let cold = host.compute(&binding, &[]).unwrap();
    sequence(&cold, &[(0, 3, 20.0, true), (3, 6, 40.0, false)]);
    for line in finalized(&cold) {
        // Ahem10: ascent8/descent2, with20px leading shared equally.
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (18.0, 30.0)
        );
    }
    assert_eq!(cold.placed_text()[0].layout().lines()[1].baseline(), 48.0);
    let incoming = cold
        .final_inline_fragments()
        .iter()
        .find(|row| {
            row.owner() == &crate::InlineFragmentOwner::Node(10)
                && row.fragment().line_index() == 1
                && matches!(
                    row.fragment().finalized_fragment().view().owner(),
                    text::FinalizedFragmentOwner::Alternative(_)
                )
        })
        .expect("the receiving prefix retains its preceding origin's actual node");
    assert_eq!(
        incoming.fragment().rect().unwrap().size().height,
        S::from_f64(30.0)
    );
    assert!(
        cold.final_inline_fragments()
            .iter()
            .any(|row| row.owner() == &crate::InlineFragmentOwner::Node(11)
                && row.fragment().line_index() == 1)
    );
    // The actual40px receiver plus8px float cannot share width40. It reserves
    // its full30px height; measuring only original CDE would wrongly allow it.
    assert_eq!(
        output(&cold, 30).location,
        Point::new(S::ZERO, S::from_f64(60.0))
    );
    cold.apply_to(&mut host).unwrap();
    let warm = host.compute(&binding, &[]).unwrap();
    super::conditional_cache::assert_same(&cold, &warm);
    warm.apply_to(&mut host).unwrap();
    host.tree.insert_input(
        2,
        LayoutInputOf::box_input(root(
            40.0,
            HyphenateLimitLast::Always,
            HyphenateLimitLines::NoLimit,
        )),
    );
    let withdrawn = host.compute(&binding, &[2]).unwrap();
    sequence(&withdrawn, &[(0, 6, 50.0, false)]);
    assert_eq!(
        output(&withdrawn, 30).location,
        Point::new(S::ZERO, S::from_f64(30.0))
    );
    let fresh = CachedHost::new(host.tree.clone())
        .compute(&binding, &[])
        .unwrap();
    super::conditional_cache::assert_same(&withdrawn, &fresh);
    withdrawn.apply_to(&mut host).unwrap();

    // Rejecting the complete replacement at the actual batch sink leaves the
    // published no-prefix sequence and its line/float caches intact.
    let published = host.published.clone().unwrap();
    let caches = host.caches.clone();
    host.tree.insert_input(
        2,
        LayoutInputOf::box_input(root(
            40.0,
            HyphenateLimitLast::None,
            HyphenateLimitLines::NoLimit,
        )),
    );
    let recovered = host.compute(&binding, &[2]).unwrap();
    sequence(&recovered, &[(0, 3, 20.0, true), (3, 6, 40.0, false)]);
    host.reject_batch = true;
    assert_eq!(
        recovered.apply_to(&mut host),
        Err("reject complete replacement")
    );
    assert_eq!(host.caches, caches);
    super::conditional_cache::assert_same(host.published.as_ref().unwrap(), &published);
    host.reject_batch = false;
    recovered.apply_to(&mut host).unwrap();
    let fresh = CachedHost::new(host.tree.clone())
        .compute(&binding, &[])
        .unwrap();
    super::conditional_cache::assert_same(&recovered, &fresh);
}
#[test]
fn after_prefix_height_and_float_feedback_rebuild_transactionally_f32() {
    prefix_float_feedback::<f32>();
}
#[test]
fn after_prefix_height_and_float_feedback_rebuild_transactionally_f64() {
    prefix_float_feedback::<f64>();
}

fn late_float_refusal<S: LayoutScalar>() {
    let base = style(text::Hyphens::Manual, text::OverflowWrap::Normal);
    let tall = text::Style {
        line_height: text::LineHeight::Absolute(30.0),
        ..base.clone()
    };
    let mut source = Source::new("AB\u{ad}CDE");
    source.span(text::Range::new(0, 1), tall.clone());
    source.span(text::Range::new(2, 3), tall);
    let p = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, source, base.clone()),
            ParagraphPart::text(11, Source::new(" X"), base.clone()),
        ],
        base,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = p.binding(2);
    binding.nodes = vec![10, 30, 11];
    let mut host = CachedHost::new(float_tree::<S>(&binding, 40.0, Float::Left, 1));
    let cold = host.compute(&binding, &[]).unwrap();
    sequence(
        &cold,
        &[(0, 3, 20.0, true), (3, 7, 40.0, false), (7, 8, 10.0, false)],
    );
    // Unlike a trailing float, this float genuinely constrains following X.
    // The after-bearing receiver reserves30px before that third line begins.
    assert_eq!(
        output(&cold, 30).location,
        Point::new(S::ZERO, S::from_f64(60.0))
    );
    cold.apply_to(&mut host).unwrap();
    let published = host.published.clone().unwrap();
    let caches = host.caches.clone();
    host.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            float_exclusion: FloatExclusion::Shape,
            ..host.tree.node_input(30).clone()
        }),
    );
    host.fail_shape = true;
    let error = host.compute(&binding, &[2, 30]).unwrap_err();
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement("actual float shape refused")
    ));
    assert_eq!(host.caches, caches);
    super::conditional_cache::assert_same(host.published.as_ref().unwrap(), &published);
    host.fail_shape = false;
    let recovered = host.compute(&binding, &[2, 30]).unwrap();
    sequence(
        &recovered,
        &[(0, 3, 20.0, true), (3, 7, 40.0, false), (7, 8, 10.0, false)],
    );
    let fresh = CachedHost::new(host.tree.clone())
        .compute(&binding, &[])
        .unwrap();
    super::conditional_cache::assert_same(&recovered, &fresh);
}
#[test]
fn after_receiving_sequence_does_not_publish_a_late_float_refusal_f32() {
    late_float_refusal::<f32>();
}
#[test]
fn after_receiving_sequence_does_not_publish_a_late_float_refusal_f64() {
    late_float_refusal::<f64>();
}
