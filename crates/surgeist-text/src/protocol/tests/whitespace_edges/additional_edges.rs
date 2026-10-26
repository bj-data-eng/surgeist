//! Existing-callable Phase II observations, before the new measurement API.
use super::*;

fn prepare_source(system: &mut System, source: Source, base_style: Style) -> PreparedText {
    prepare_direction(
        system,
        source,
        base_style,
        ParagraphBaseDirection::LeftToRight,
    )
}

fn prepare_direction(
    system: &mut System,
    source: Source,
    base_style: Style,
    direction: ParagraphBaseDirection,
) -> PreparedText {
    let end = source.text().len();
    let input = ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        source: source.clone(),
        base_style,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, end),
            aggregate: Range::new(0, end),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(input).unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::default(),
        )
        .unwrap()
}

fn select(
    p: &PreparedText,
    start: PreparedBoundary,
    ending: PreparedOpportunity,
    width: f64,
    alignment: crate::Alignment,
) -> FinalizedLine {
    selected(p, start, ending, width, alignment)
        .finalize(SpacingAllocations::None)
        .unwrap()
}

fn selected(
    p: &PreparedText,
    start: PreparedBoundary,
    ending: PreparedOpportunity,
    width: f64,
    alignment: crate::Alignment,
) -> SelectedLine {
    let mut r = request(p);
    r.context.base_direction = match p.analyzed_source().direction() {
        crate::Direction::LeftToRight => ParagraphBaseDirection::LeftToRight,
        crate::Direction::RightToLeft => ParagraphBaseDirection::RightToLeft,
        crate::Direction::Auto => panic!("analysis returns a resolved base direction"),
    };
    r.context.line_box_direction = match p.analyzed_source().direction() {
        crate::Direction::LeftToRight => crate::LineBoxDirection::LeftToRight,
        crate::Direction::RightToLeft => crate::LineBoxDirection::RightToLeft,
        crate::Direction::Auto => panic!("analysis returns a resolved base direction"),
    };
    r.start = start;
    r.end = ending.view().boundary();
    let from = r.start.view().participant_offset();
    let to = r.end.view().participant_offset();
    r.participants = p.participants().skip(from).take(to - from).collect();
    r.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.end = match ending.view().kind() {
        PreparedBreakKind::Allowed => LineEnd::Soft,
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
        _ => unreachable!(),
    };
    r.termination = crate::PreparedTermination::Ordinary(ending);
    r.context.available_inline = Some(width);
    r.context.alignment = alignment;
    p.select(r).unwrap()
}

fn first(p: &PreparedText) -> PreparedBoundary {
    p.boundary_views().next().unwrap().handle()
}

fn ending(p: &PreparedText, kind: PreparedBreakKind) -> PreparedOpportunity {
    p.opportunity_views()
        .find(|o| o.kind() == kind)
        .unwrap()
        .handle()
}

fn continuation(p: &PreparedText, start: PreparedBoundary, width: f64) -> FinalizedLine {
    select(
        p,
        start,
        ending(p, PreparedBreakKind::ParagraphEnd),
        width,
        crate::Alignment::Center,
    )
}

#[test]
fn soft_preserved_space_reports_unconditional_hanging_without_losing_source() {
    let mut system = system();
    controls(&mut system);
    let p = prepare(&mut system, "X Y", WhiteSpace::Preserve);
    let soft = ending(&p, PreparedBreakKind::Allowed);
    assert_eq!(soft.view().boundary().view().source_after().get(), 2);
    let line = select(&p, first(&p), soft.clone(), 40.0, crate::Alignment::Center);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        glyphs(&line, 2),
        [
            (Range::new(0, 1), 58, 10.0, 15.0),
            (Range::new(1, 2), 3, 10.0, 25.0)
        ]
    );
    let hanging = line.feedback().hanging_end;
    let next = continuation(&p, soft.view().boundary(), 40.0);
    assert_eq!(glyphs(&next, 3), [(Range::new(2, 3), 59, 10.0, 15.0)]);
    custody(&p, vec![line, next], &[88, 32, 89]);
    assert_eq!(hanging, 10.0);
}

#[test]
fn forced_preserved_space_hangs_only_its_actual_overflow() {
    let mut system = system();
    controls(&mut system);
    let p = prepare(&mut system, "X \nY", WhiteSpace::Preserve);
    let forced = ending(&p, PreparedBreakKind::Forced);
    let line = select(
        &p,
        first(&p),
        forced.clone(),
        15.0,
        crate::Alignment::Center,
    );
    assert_eq!(line.feedback().advance, 20.0);
    let actual = glyphs(&line, 2);
    assert_eq!(
        actual.iter().map(|r| (r.0, r.1, r.2)).collect::<Vec<_>>(),
        [(Range::new(0, 1), 58, 10.0), (Range::new(1, 2), 3, 10.0)]
    );
    let observed = (
        line.feedback().hanging_end,
        actual.iter().map(|r| r.3).collect::<Vec<_>>(),
    );
    let next = continuation(&p, forced.view().boundary(), 15.0);
    assert_eq!(glyphs(&next, 4), [(Range::new(3, 4), 59, 10.0, 2.5)]);
    custody(&p, vec![line, next], &[88, 32, 10, 89]);
    // Raw20 exceeds W15 by5. Excluding that5 leaves alignment measure15.
    assert_eq!(observed, (5.0, vec![0.0, 10.0]));
}

#[test]
fn collapse_other_space_separator_hangs_but_is_not_nbsp() {
    let mut system = system();
    controls(&mut system);
    assert_eq!(
        skrifa::FontRef::new(AHEM)
            .unwrap()
            .charmap()
            .map(0x2003_u32)
            .unwrap()
            .to_u32(),
        247
    );
    let p = prepare(&mut system, "X\u{2003}", WhiteSpace::Collapse);
    let line = paragraph(&p);
    assert_eq!(line.feedback().advance, 20.0);
    let rows = glyphs(&line, 2);
    assert_eq!(
        rows.iter().map(|r| (r.0, r.1, r.2)).collect::<Vec<_>>(),
        [(Range::new(0, 1), 58, 10.0), (Range::new(1, 2), 247, 10.0)]
    );
    let observed = (
        line.feedback().hanging_end,
        rows.iter().map(|r| r.3).collect::<Vec<_>>(),
    );
    custody(&p, vec![line], &[88, 8195]);
    assert_eq!(observed, (10.0, vec![15.0, 25.0]));
}

#[test]
fn trailing_ogham_is_removed_while_leading_ogham_keeps_real_advance() {
    // Add only a cmap entry to an owned memory variant. The unused AA ligature
    // cannot match either source; U1680 uses the real 10px SPACE outline/advance.
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[35, 35],
        &[(0x20, 3), (0x58, 58), (0x59, 59), (0xa0, 153), (0x1680, 3)],
    );
    assert_eq!(
        skrifa::FontRef::new(&bytes)
            .unwrap()
            .charmap()
            .map(0x1680_u32)
            .unwrap()
            .to_u32(),
        3
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let inspect = |line: &FinalizedLine| {
        line.run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), bytes);
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                let origin = run.origin().x;
                run.glyphs()
                    .map(move |g| (g.range(), g.id(), g.advance(), origin + g.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    let leading = prepare(&mut system, "\u{1680}X", WhiteSpace::Collapse);
    let line = paragraph(&leading);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        inspect(&line),
        [
            (Range::new(0, 1), 3, 10.0, 10.0),
            (Range::new(1, 2), 58, 10.0, 20.0)
        ]
    );
    custody(&leading, vec![line], &[5760, 88]);
    let trailing = prepare(&mut system, "X\u{1680}", WhiteSpace::Collapse);
    assert_eq!(
        trailing
            .unit_views()
            .map(|u| (u.source_range(), u.advance()))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 10.0), (Range::new(1, 2), 10.0)]
    );
    let line = paragraph(&trailing);
    let observed = (line.feedback().advance, inspect(&line));
    custody(&trailing, vec![line], &[88, 5760]);
    assert_eq!(observed, (10.0, vec![(Range::new(0, 1), 58, 10.0, 15.0)]));
}

#[test]
fn negative_preserved_space_contribution_is_not_absolute_hanging() {
    let mut system = system();
    controls(&mut system);
    let base = Style {
        size: 10.0,
        white_space: WhiteSpace::Preserve,
        ..style()
    };
    let mut source = Source::new("X \nY");
    source.span(
        Range::new(1, 2),
        Style {
            word_spacing: -12.0,
            ..base.clone()
        },
    );
    let p = prepare_source(&mut system, source, base);
    let forced = ending(&p, PreparedBreakKind::Forced);
    let line = select(
        &p,
        first(&p),
        forced.clone(),
        20.0,
        crate::Alignment::Center,
    );
    assert_eq!(line.feedback().advance, 8.0);
    let rows = glyphs(&line, 2);
    assert_eq!(
        rows.iter().map(|r| (r.0, r.1, r.2)).collect::<Vec<_>>(),
        [(Range::new(0, 1), 58, 10.0), (Range::new(1, 2), 3, 10.0)]
    );
    let observed = (
        line.feedback().hanging_end,
        rows.iter().map(|r| r.3).collect::<Vec<_>>(),
    );
    let next = continuation(&p, forced.view().boundary(), 20.0);
    let layout = custody(&p, vec![line, next], &[88, 32, 10, 89]);
    let x = |i| {
        layout
            .cursor(Cursor::new(i, Affinity::Before))
            .unwrap()
            .rect()
            .origin
            .x
    };
    assert_eq!((x(1) - x(0), x(2) - x(1)), (10.0, -2.0));
    assert_eq!(observed, (0.0, vec![6.0, 16.0]));
}

#[test]
fn earlier_unconditional_separator_depends_on_following_conditional_sequence() {
    let mut system = system();
    controls(&mut system);
    let base = Style {
        size: 10.0,
        white_space: WhiteSpace::Preserve,
        ..style()
    };
    let mut source = Source::new("X\u{2003} \nY");
    source.span(
        Range::new(1, 2),
        Style {
            white_space: WhiteSpace::Collapse,
            ..base.clone()
        },
    );
    let p = prepare_source(&mut system, source, base);
    let forced = ending(&p, PreparedBreakKind::Forced);
    let mut observations = vec![];
    for (width, alignment) in [
        (40.0, crate::Alignment::Center),
        (15.0, crate::Alignment::Left),
    ] {
        let line = select(&p, first(&p), forced.clone(), width, alignment);
        assert_eq!(line.feedback().advance, 30.0);
        let rows = glyphs(&line, 3);
        assert_eq!(
            rows.iter().map(|r| (r.0, r.1, r.2)).collect::<Vec<_>>(),
            [
                (Range::new(0, 1), 58, 10.0),
                (Range::new(1, 2), 247, 10.0),
                (Range::new(2, 3), 3, 10.0)
            ]
        );
        observations.push((
            line.feedback().hanging_end,
            rows.iter().map(|r| r.3).collect::<Vec<_>>(),
        ));
        let next = continuation(&p, forced.view().boundary(), width);
        custody(&p, vec![line, next], &[88, 8195, 32, 10, 89]);
    }
    assert_eq!(
        observations,
        [(0.0, vec![5.0, 15.0, 25.0]), (20.0, vec![0.0, 10.0, 20.0])]
    );
}

fn allocated(selected: &SelectedLine, amounts: &[f64]) -> FinalizedLine {
    let slots: Vec<_> = selected.spacing_slots().collect();
    assert_eq!(slots.len(), amounts.len());
    selected
        .finalize(SpacingAllocations::Complete(
            slots
                .into_iter()
                .zip(amounts)
                .map(|(slot, &amount)| SpacingAllocation { slot, amount })
                .collect(),
        ))
        .unwrap()
}

#[test]
fn allocated_conditional_space_remeasures_signed_exclusion_in_natural_frame() {
    let mut system = system();
    controls(&mut system);
    let p = prepare(&mut system, "X \nY", WhiteSpace::Preserve);
    let forced = ending(&p, PreparedBreakKind::Forced);
    let selection = selected(
        &p,
        first(&p),
        forced.clone(),
        15.0,
        crate::Alignment::Center,
    );
    let natural = selection.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 20.0);
    let mut observed = vec![];
    for amount in [4.0, -3.0, -5.0, -8.0, -12.0] {
        let line = allocated(&selection, &[amount]);
        assert_eq!(line.feedback().advance, 20.0 + amount);
        assert_eq!(
            glyphs(&line, 2)
                .iter()
                .map(|r| (r.0, r.1, r.2))
                .collect::<Vec<_>>(),
            [(Range::new(0, 1), 58, 10.0), (Range::new(1, 2), 3, 10.0)]
        );
        observed.push((
            line.feedback().hanging_end,
            glyphs(&line, 2).iter().map(|r| r.3).collect::<Vec<_>>(),
        ));
        let next = continuation(&p, forced.view().boundary(), 15.0);
        custody(&p, vec![line, next], &[88, 32, 10, 89]);
    }
    // Natural eligibility is fixed; only signed contribution beyond edge15 is
    // remeasured after each complete allocation. No reverse allocation guess.
    assert_eq!(
        observed,
        [
            (9.0, vec![0.0, 10.0]),
            (2.0, vec![0.0, 10.0]),
            (0.0, vec![0.0, 10.0]),
            (0.0, vec![1.5, 11.5]),
            (0.0, vec![3.5, 13.5]),
        ]
    );
}

#[test]
fn naturally_fitting_conditional_space_does_not_become_eligible_after_expansion() {
    let mut system = system();
    let p = prepare(&mut system, "X \nY", WhiteSpace::Preserve);
    let forced = ending(&p, PreparedBreakKind::Forced);
    let selection = selected(&p, first(&p), forced.clone(), 40.0, crate::Alignment::Left);
    assert_eq!(
        selection
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        20.0
    );
    let line = allocated(&selection, &[24.0]);
    assert_eq!(line.feedback().advance, 44.0);
    assert_eq!(
        glyphs(&line, 2),
        [
            (Range::new(0, 1), 58, 10.0, 0.0),
            (Range::new(1, 2), 3, 10.0, 10.0)
        ]
    );
    assert_eq!(
        (line.feedback().hanging_start, line.feedback().hanging_end),
        (0.0, 0.0)
    );
    let next = continuation(&p, forced.view().boundary(), 40.0);
    custody(&p, vec![line, next], &[88, 32, 10, 89]);
}

#[test]
fn upstream_allocation_moves_eligible_conditional_space_before_exclusion() {
    let mut system = system();
    let p = prepare(&mut system, "X X \nY", WhiteSpace::Preserve);
    let forced = ending(&p, PreparedBreakKind::Forced);
    let selection = selected(&p, first(&p), forced.clone(), 35.0, crate::Alignment::Left);
    assert_eq!(
        selection
            .spacing_slots()
            .map(|s| s.source_range())
            .collect::<Vec<_>>(),
        [Range::new(1, 2), Range::new(3, 4)]
    );
    assert_eq!(
        selection
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        40.0
    );
    let line = allocated(&selection, &[4.0, 0.0]);
    assert_eq!(line.feedback().advance, 44.0);
    assert_eq!(
        glyphs(&line, 4).iter().map(|r| r.3).collect::<Vec<_>>(),
        [0.0, 10.0, 24.0, 34.0]
    );
    let hanging = line.feedback().hanging_end;
    let next = continuation(&p, forced.view().boundary(), 35.0);
    custody(&p, vec![line, next], &[88, 32, 88, 32, 10, 89]);
    assert_eq!(hanging, 9.0);
}

#[test]
fn natural_terminal_group_stays_fixed_through_reverse_conditional_advance() {
    let mut system = system();
    let base = Style {
        size: 10.0,
        white_space: WhiteSpace::Preserve,
        ..style()
    };
    let mut source = Source::new("X\u{2003} \nY");
    source.span(
        Range::new(1, 2),
        Style {
            white_space: WhiteSpace::Collapse,
            ..base.clone()
        },
    );
    let p = prepare_source(&mut system, source, base);
    let forced = ending(&p, PreparedBreakKind::Forced);
    let selection = selected(&p, first(&p), forced.clone(), 15.0, crate::Alignment::Left);
    assert_eq!(
        selection
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        30.0
    );
    let mut observed = vec![];
    for amount in [-8.0, -12.0] {
        let line = allocated(&selection, &[amount]);
        assert_eq!(line.feedback().advance, 30.0 + amount);
        assert_eq!(
            glyphs(&line, 3).iter().map(|r| r.3).collect::<Vec<_>>(),
            [0.0, 10.0, 20.0]
        );
        observed.push(line.feedback().hanging_end);
        let next = continuation(&p, forced.view().boundary(), 15.0);
        custody(&p, vec![line, next], &[88, 8195, 32, 10, 89]);
    }
    // EM SPACE remains naturally terminal. Following conditional contribution
    // is +2 or -2 wholly beyond15; signed totals are10+2 and10-2.
    assert_eq!(observed, [12.0, 8.0]);
}

#[test]
fn rtl_allocated_conditional_space_reports_the_physical_start_edge() {
    let mut system = system();
    let p = prepare_direction(
        &mut system,
        Source::new("X \nY"),
        Style {
            size: 10.0,
            white_space: WhiteSpace::Preserve,
            ..style()
        },
        ParagraphBaseDirection::RightToLeft,
    );
    let forced = ending(&p, PreparedBreakKind::Forced);
    let selection = selected(
        &p,
        first(&p),
        forced.clone(),
        15.0,
        crate::Alignment::Center,
    );
    assert_eq!(
        selection
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        20.0
    );
    let mut observed = vec![];
    for amount in [4.0, -3.0, -5.0, -8.0, -12.0] {
        let line = allocated(&selection, &[amount]);
        assert_eq!(line.feedback().advance, 20.0 + amount);
        let rows = glyphs(&line, 2);
        assert_eq!(rows.iter().map(|r| r.1).collect::<Vec<_>>(), [3, 58]);
        observed.push((line.feedback().hanging_start, line.feedback().hanging_end));
        let next = continuation(&p, forced.view().boundary(), 15.0);
        custody(&p, vec![line, next], &[88, 32, 10, 89]);
    }
    assert_eq!(
        observed,
        [(9.0, 0.0), (2.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)]
    );
}

#[test]
fn partly_hanging_conditional_space_leaves_preceding_separator_terminal() {
    let mut system = system();
    controls(&mut system);
    let base = Style {
        size: 10.0,
        white_space: WhiteSpace::Preserve,
        ..style()
    };
    let mut source = Source::new("X\u{2003} \nY");
    source.span(
        Range::new(1, 2),
        Style {
            white_space: WhiteSpace::Collapse,
            ..base.clone()
        },
    );
    let p = prepare_source(&mut system, source, base);
    let forced = ending(&p, PreparedBreakKind::Forced);
    assert_eq!(forced.view().boundary().view().source_after().get(), 4);
    let line = select(
        &p,
        first(&p),
        forced.clone(),
        25.0,
        crate::Alignment::Center,
    );
    assert_eq!(line.feedback().advance, 30.0);
    let rows = glyphs(&line, 3);
    assert_eq!(
        rows.iter().map(|r| (r.0, r.1, r.2)).collect::<Vec<_>>(),
        [
            (Range::new(0, 1), 58, 10.0),
            (Range::new(1, 2), 247, 10.0),
            (Range::new(2, 3), 3, 10.0)
        ]
    );
    let observed = (
        line.feedback().hanging_end,
        rows.iter().map(|r| r.3).collect::<Vec<_>>(),
    );
    let next = continuation(&p, forced.view().boundary(), 25.0);
    assert_eq!(glyphs(&next, 5), [(Range::new(4, 5), 59, 10.0, 7.5)]);
    custody(&p, vec![line, next], &[88, 8195, 32, 10, 89]);
    // SPACE20..30 partially hangs beyond25. It therefore counts as hanging,
    // leaving EM SPACE terminal: exclusion10+5, alignment30-15, center shift5.
    assert_eq!(observed, (15.0, vec![5.0, 15.0, 25.0]));
}

#[test]
fn fitting_zero_advance_conditional_space_keeps_preceding_separator_nonterminal() {
    let mut system = system();
    controls(&mut system);
    let base = Style {
        size: 10.0,
        white_space: WhiteSpace::Preserve,
        ..style()
    };
    let mut source = Source::new("X\u{2003} \nY");
    source.span(
        Range::new(1, 2),
        Style {
            white_space: WhiteSpace::Collapse,
            ..base.clone()
        },
    );
    source.span(
        Range::new(2, 3),
        Style {
            word_spacing: -10.0,
            ..base.clone()
        },
    );
    let p = prepare_source(&mut system, source, base);
    let forced = ending(&p, PreparedBreakKind::Forced);
    assert_eq!(forced.view().boundary().view().source_after().get(), 4);
    let line = select(
        &p,
        first(&p),
        forced.clone(),
        40.0,
        crate::Alignment::Center,
    );
    assert_eq!(line.feedback().advance, 20.0);
    let rows = glyphs(&line, 3);
    assert_eq!(
        rows.iter().map(|r| (r.0, r.1, r.2)).collect::<Vec<_>>(),
        [
            (Range::new(0, 1), 58, 10.0),
            (Range::new(1, 2), 247, 10.0),
            (Range::new(2, 3), 3, 10.0)
        ]
    );
    let observed = (
        line.feedback().hanging_end,
        rows.iter().map(|r| r.3).collect::<Vec<_>>(),
    );
    let next = continuation(&p, forced.view().boundary(), 40.0);
    custody(&p, vec![line, next], &[88, 8195, 32, 10, 89]);
    // The actual preserved SPACE ends at20, inside40. It does not hang, so
    // the preceding EM SPACE is not terminal (CSS Text4 section9.2).
    assert_eq!(observed, (0.0, vec![10.0, 20.0, 30.0]));
}

#[test]
fn ordinary_start_alignment_uses_actual_geometry_without_hanging_capacity() {
    let mut system = system();
    controls(&mut system);
    let p = prepare(&mut system, "X", WhiteSpace::Preserve);
    let mut r = request(&p);
    r.context.available_inline = Some(1e308);
    r.context.indent = 1.0;
    r.context.alignment = crate::Alignment::Start;
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .expect("ordinary start-aligned text consumes no hanging edge");
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(
        (line.feedback().hanging_start, line.feedback().hanging_end),
        (0.0, 0.0)
    );
    assert_eq!(glyphs(&line, 1), [(Range::new(0, 1), 58, 10.0, 1.0)]);
    custody(&p, vec![line], &[88]);
}

#[test]
fn conditional_space_allocation_to_zero_preserves_source_and_alignment() {
    let mut system = system();
    controls(&mut system);
    let p = prepare(&mut system, "X \nY", WhiteSpace::Preserve);
    let forced = ending(&p, PreparedBreakKind::Forced);
    let selection = selected(
        &p,
        first(&p),
        forced.clone(),
        15.0,
        crate::Alignment::Center,
    );
    let natural = selection.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 20.0);
    assert_eq!(
        glyphs(&natural, 2)
            .iter()
            .map(|r| (r.0, r.1, r.2))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 58, 10.0), (Range::new(1, 2), 3, 10.0)]
    );
    let line = allocated(&selection, &[-10.0]);
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.feedback().hanging_end, 0.0);
    assert_eq!(
        glyphs(&line, 2),
        [
            (Range::new(0, 1), 58, 10.0, 2.5),
            (Range::new(1, 2), 3, 10.0, 12.5)
        ]
    );
    let next = continuation(&p, forced.view().boundary(), 15.0);
    custody(&p, vec![line, next], &[88, 32, 10, 89]);
}

fn prepared_discard_separator(
    terminal_edge: crate::ParagraphTerminalEdge,
) -> (Vec<u8>, PreparedText) {
    let bytes = crate::shaping_test_fonts::width_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map(0x41_u32).unwrap().to_u32(), 35);
    assert_eq!(font.charmap().map(0x3000_u32).unwrap().to_u32(), 1);
    assert_eq!(
        crate::test_fonts::u16_at(crate::test_fonts::table(&bytes, *b"head"), 18),
        1000
    );
    let hmtx = crate::test_fonts::table(&bytes, *b"hmtx");
    assert_eq!(crate::test_fonts::u16_at(hmtx, 35 * 4), 1000);
    assert_eq!(crate::test_fonts::u16_at(hmtx, 4), 2000);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let source = Source::identified("A\u{3000}", crate::Id::from_u64(7401), 7);
    let input = ParagraphInput {
        terminal_edge,
        source: source.clone(),
        base_style: Style {
            size: 10.0,
            white_space: WhiteSpace::Discard,
            ..style()
        },
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, 2),
            aggregate: Range::new(0, 2),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    assert_eq!(analyzed.source().text().code_units(), [0x41, 0x3000]);
    assert_eq!(analyzed.source().id(), Some(crate::Id::from_u64(7401)));
    assert_eq!(analyzed.source().revision(), 7);
    assert_eq!(
        analyzed.processed_source().processed_text().code_units(),
        [0x41, 0x3000]
    );
    let prepared = system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap();
    (bytes, prepared)
}

fn discard_separator_custody(p: &PreparedText, line: &FinalizedLine, bytes: &[u8]) {
    let actual: Vec<_> = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            let origin = run.origin().x;
            run.glyphs()
                .map(move |glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        glyph.advance(),
                        origin + glyph.pen(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        actual,
        [
            (Range::new(0, 1), 35, 10.0, 0.0),
            (Range::new(1, 2), 1, 20.0, 10.0),
        ]
    );
    assert_eq!(line.feedback().advance, 30.0);
    let layout = custody(p, vec![line.clone()], &[0x41, 0x3000]);
    assert_eq!(layout.source().id(), Some(crate::Id::from_u64(7401)));
    assert_eq!(layout.source().revision(), 7);
}

#[test]
fn discard_surviving_separator_keeps_ordinary_advance_at_block_end() {
    // Adopted #74 policy: Discard gives surviving characters ordinary advance,
    // without whitespace-rule hanging. Actual A10 + U3000(20) therefore occupies
    // 0..30 even when the available width is only10.
    let (bytes, p) = prepared_discard_separator(crate::ParagraphTerminalEdge::BlockEnd);
    let line = select(
        &p,
        first(&p),
        ending(&p, PreparedBreakKind::ParagraphEnd),
        10.0,
        crate::Alignment::Left,
    );
    discard_separator_custody(&p, &line, &bytes);
    let fit = line.measurements().fit();
    assert_eq!(
        (
            line.feedback().hanging_start,
            line.feedback().hanging_end,
            fit.advance(),
            fit.occupied_start(),
            fit.occupied_end()
        ),
        (0.0, 0.0, 30.0, 0.0, 30.0)
    );
}

#[test]
fn discard_surviving_separator_needs_no_hanging_terminal_or_available_edge() {
    let (bytes, p) = prepared_discard_separator(crate::ParagraphTerminalEdge::Unknown);
    let mut r = request(&p);
    r.context.available_inline = None;
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .expect("ordinary Discard advance needs neither terminal-block nor available-edge hanging context");
    discard_separator_custody(&p, &line, &bytes);
    let fit = line.measurements().fit();
    assert_eq!(
        (
            line.feedback().hanging_start,
            line.feedback().hanging_end,
            fit.advance(),
            fit.occupied_start(),
            fit.occupied_end()
        ),
        (0.0, 0.0, 30.0, 0.0, 30.0)
    );
}

fn discard_edge_capture(
    source: Source,
    base_style: Style,
    direction: ParagraphBaseDirection,
    terminal_edge: crate::ParagraphTerminalEdge,
    forced_at: Option<usize>,
) -> PreparedText {
    let end = source.text().len();
    let input = ParagraphInput {
        source: source.clone(),
        base_style,
        base_direction: direction,
        terminal_edge,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, end),
            aggregate: Range::new(0, end),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        controls: forced_at
            .into_iter()
            .map(|at| paragraph::ParagraphControl {
                at,
                kind: paragraph::ParagraphControlKind::ForcedBreak,
            })
            .collect(),
        anchor_order: forced_at
            .into_iter()
            .map(|_| paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Control(0),
                affinity: crate::Affinity::After,
            })
            .collect(),
        connections: vec![],
        replacements: vec![],
        boundary_policies: vec![],
    };
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::width_ahem())
        .unwrap();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(input).unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::default(),
        )
        .unwrap()
}

fn ordinary_discard_measurements(line: &FinalizedLine, advance: f64) {
    assert_eq!(line.feedback().advance, advance);
    assert_eq!(
        (line.feedback().hanging_start, line.feedback().hanging_end),
        (0.0, 0.0)
    );
    for measure in [
        line.measurements().fit(),
        line.measurements().alignment(),
        line.measurements().min_content_contribution(),
        line.measurements().max_content_contribution(),
    ] {
        assert_eq!(
            (
                measure.advance(),
                measure.occupied_start(),
                measure.occupied_end()
            ),
            (advance, 0.0, advance)
        );
    }
    assert!(
        line.edge_units()
            .all(|unit| unit.disposition() == crate::LineEdgeDisposition::Ordinary)
    );
}

#[test]
fn discard_surviving_separator_stays_ordinary_at_soft_forced_and_block_ends_in_both_directions() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        for wrap in [crate::Wrap::Word, crate::Wrap::None] {
            for (kind, text, forced) in [
                (PreparedBreakKind::Allowed, "A\u{3000}A", None),
                (PreparedBreakKind::Forced, "A\u{3000}A", Some(2)),
                (PreparedBreakKind::ParagraphEnd, "A\u{3000}", None),
            ] {
                if kind == PreparedBreakKind::Allowed && wrap == crate::Wrap::None {
                    continue; // No soft endpoint is issued for genuine nowrap.
                }
                let source = Source::new(text);
                let p = discard_edge_capture(
                    source.clone(),
                    Style {
                        size: 10.0,
                        white_space: WhiteSpace::Discard,
                        wrap,
                        ..style()
                    },
                    direction,
                    crate::ParagraphTerminalEdge::BlockEnd,
                    forced,
                );
                let stop = ending(&p, kind);
                assert_eq!(stop.view().boundary().view().source_after().get(), 2);
                let mut r = request(&p);
                r.end = stop.view().boundary();
                r.participants = p
                    .participants()
                    .take(r.end.view().participant_offset())
                    .collect();
                r.controls = r
                    .participants
                    .iter()
                    .filter_map(|participant| match participant {
                        PreparedParticipant::Control(control) => Some({
                            let occurrence = control.clone();
                            ControlAllocation {
                                boundary: matches!(
                                    occurrence.view().kind(),
                                    crate::ParagraphControlKind::InlineBoundaryStart
                                        | crate::ParagraphControlKind::InlineBoundaryEnd
                                )
                                .then_some(crate::InlineBoundaryAllocation::ZERO),
                                occurrence,
                                strut: None,
                            }
                        }),
                        _ => None,
                    })
                    .collect();
                r.termination = crate::PreparedTermination::Ordinary(stop.clone());
                r.context.base_direction = direction;
                r.context.line_box_direction = match direction {
                    crate::ParagraphBaseDirection::LeftToRight => {
                        crate::LineBoxDirection::LeftToRight
                    }
                    crate::ParagraphBaseDirection::RightToLeft => {
                        crate::LineBoxDirection::RightToLeft
                    }
                };
                r.context.end = match kind {
                    PreparedBreakKind::Allowed => LineEnd::Soft,
                    PreparedBreakKind::Forced => LineEnd::Forced,
                    PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                    _ => unreachable!(),
                };
                r.context.available_inline = Some(10.0);
                let line = p
                    .select(r)
                    .unwrap()
                    .finalize(SpacingAllocations::None)
                    .unwrap();
                ordinary_discard_measurements(&line, 30.0);
                let bytes = crate::shaping_test_fonts::width_ahem();
                let mut actual: Vec<_> = line
                    .run_views()
                    .flat_map(|run| {
                        assert_eq!(run.face().bytes(), bytes);
                        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                        run.glyphs()
                            .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                actual.sort_by_key(|glyph| glyph.0.start);
                assert_eq!(
                    actual,
                    [(Range::new(0, 1), 35, 10.0), (Range::new(1, 2), 1, 20.0)]
                );
                let mut lines = vec![line];
                if kind != PreparedBreakKind::ParagraphEnd {
                    lines.push(select(
                        &p,
                        stop.view().boundary(),
                        ending(&p, PreparedBreakKind::ParagraphEnd),
                        10.0,
                        crate::Alignment::Left,
                    ));
                }
                let placed = crate::Layout::from_finalized_lines(PlacementRequest {
                    prepared: p,
                    coverage: PlacementCoverage::WholeParagraph,
                    entries: lines
                        .into_iter()
                        .enumerate()
                        .map(|(index, line)| {
                            crate::PlacementEntry::Formatted(LinePlacement {
                                line,
                                anchor: LineAnchor {
                                    inline_origin: 0.0,
                                    baseline: 8.0 + index as f64 * 20.0,
                                },
                                orientation: LineOrientation::Horizontal,
                                atomics: vec![],
                            })
                        })
                        .collect(),
                })
                .unwrap();
                assert_eq!(placed.source(), &source);
            }
        }
    }
}

#[test]
fn discard_surviving_separator_consumes_no_terminal_or_available_context_in_both_directions() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        for wrap in [crate::Wrap::Word, crate::Wrap::None] {
            for terminal in [
                crate::ParagraphTerminalEdge::Unknown,
                crate::ParagraphTerminalEdge::NonBlockBoundary,
                crate::ParagraphTerminalEdge::BlockEnd,
            ] {
                let p = discard_edge_capture(
                    Source::new("A\u{3000}"),
                    Style {
                        size: 10.0,
                        white_space: WhiteSpace::Discard,
                        wrap,
                        ..style()
                    },
                    direction,
                    terminal,
                    None,
                );
                for width in [None, Some(10.0)] {
                    let mut r = request(&p);
                    r.context.base_direction = direction;
                    r.context.line_box_direction = match direction {
                        crate::ParagraphBaseDirection::LeftToRight => {
                            crate::LineBoxDirection::LeftToRight
                        }
                        crate::ParagraphBaseDirection::RightToLeft => {
                            crate::LineBoxDirection::RightToLeft
                        }
                    };
                    r.context.available_inline = width;
                    let line = p
                        .select(r)
                        .unwrap()
                        .finalize(SpacingAllocations::None)
                        .unwrap();
                    ordinary_discard_measurements(&line, 30.0);
                }
            }
        }
    }
}

#[test]
fn discard_ordinary_edge_blocks_hanging_from_preceding_mixed_whitespace_modes() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        for preceding in [
            WhiteSpace::Collapse,
            WhiteSpace::Preserve,
            WhiteSpace::PreserveSpaces,
        ] {
            let base = Style {
                size: 10.0,
                white_space: WhiteSpace::Discard,
                ..style()
            };
            let mut source = Source::new("A\u{3000}\u{3000}");
            source.span(
                Range::new(1, 2),
                Style {
                    white_space: preceding,
                    ..base.clone()
                },
            );
            let p = discard_edge_capture(
                source,
                base,
                direction,
                crate::ParagraphTerminalEdge::BlockEnd,
                None,
            );
            let line = select(
                &p,
                first(&p),
                ending(&p, PreparedBreakKind::ParagraphEnd),
                10.0,
                crate::Alignment::Left,
            );
            ordinary_discard_measurements(&line, 50.0);
        }
        // The actual trailing Preserve unit still hangs its own20px, but its
        // predecessor is a retained ordinary Discard unit and must retain20px.
        let base = Style {
            size: 10.0,
            white_space: WhiteSpace::Preserve,
            ..style()
        };
        let mut source = Source::new("A\u{3000}\u{3000}");
        source.span(
            Range::new(1, 2),
            Style {
                white_space: WhiteSpace::Discard,
                ..base.clone()
            },
        );
        let p = discard_edge_capture(
            source,
            base,
            direction,
            crate::ParagraphTerminalEdge::BlockEnd,
            None,
        );
        let line = select(
            &p,
            first(&p),
            ending(&p, PreparedBreakKind::ParagraphEnd),
            10.0,
            crate::Alignment::Left,
        );
        assert_eq!(line.feedback().advance, 50.0);
        let expected_hanging = if direction == ParagraphBaseDirection::RightToLeft {
            (20.0, 0.0)
        } else {
            (0.0, 20.0)
        };
        assert_eq!(
            (line.feedback().hanging_start, line.feedback().hanging_end),
            expected_hanging
        );
        assert_eq!(line.measurements().fit().advance(), 30.0);
        assert_eq!(line.measurements().alignment().advance(), 30.0);
        assert_eq!(
            line.measurements().min_content_contribution().advance(),
            30.0
        );
        assert_eq!(
            line.measurements().max_content_contribution().advance(),
            50.0
        );
    }
}
