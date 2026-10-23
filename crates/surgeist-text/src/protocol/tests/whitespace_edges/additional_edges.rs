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
        source: source.clone(),
        base_style,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
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
