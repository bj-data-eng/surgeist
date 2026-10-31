//! Character targets consume actual tab and captured terminal response facts.
use super::*;

fn tabs<S: LayoutScalar>() {
    let base = own(0.0, Wrap::None, text::OverflowWrap::Normal, false);
    let mut source = Source::new("AB\tC\tD");
    for (range, period) in [(text::Range::new(2, 3), 4.0), (text::Range::new(4, 5), 3.0)] {
        let mut style = base.clone();
        style.tab_size = text::TabSize::spaces(period).unwrap();
        source.span(range, style);
    }
    let paragraph = PreparedParagraph::with_css_tabs(
        vec![ParagraphPart::text(1, source, base.clone())],
        base,
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::LeftToRight,
    );
    let natural = finite::<S>(&paragraph, 70.0, TextAlignLast::Justify);
    let retained = lines(&natural).remove(0);
    amounts(&retained, &[0.0]);
    let row = retained.spacing_slots().next().unwrap();
    boundary(&row.slot, &paragraph, 0, 0.0);
    assert_eq!(
        row.stretchability,
        text::SpacingStretchability::TabDependent
    );
    assert_eq!(
        retained.selected_line().request().context.tabs,
        text::TabContext::CssStops { origin: Some(0.0) }
    );
    // Ahem SPACE and ZERO are10: periods40/30 and a strict5px threshold.
    // Incoming30 and35 still use40; incoming40 bumps to80, and the next
    // C10 then advances to120. These are actual responses on the same basis.
    for (amount, advance) in [(10.0, 70.0), (15.0, 70.0), (20.0, 130.0), (60.0, 160.0)] {
        let probe = retained
            .with_spacing(text::SpacingAllocations::Complete(vec![
                text::SpacingAllocation {
                    slot: row.slot.clone(),
                    amount,
                },
            ]))
            .unwrap();
        assert_eq!(probe.measurements().alignment().advance(), advance);
        amounts(&probe, &[amount]);
    }
    let actual = finite::<S>(&paragraph, 130.0, TextAlignLast::Justify);
    let line = lines(&actual).remove(0);
    let row = line.spacing_slots().next().unwrap();
    boundary(&row.slot, &paragraph, 0, 0.0);
    assert_eq!(
        row.stretchability,
        text::SpacingStretchability::TabDependent
    );
    assert!(row.allocation > 15.0 && row.allocation <= 55.0);
    amounts(&line, &[row.allocation]);
    assert_eq!(
        line.selected_line().request().context.spacing,
        text::SpacingPolicy::InterCharacter
    );
    assert_eq!(
        line.selected_line().request().context.alignment,
        text::Alignment::Justify
    );
    assert_eq!(line.measurements().alignment().advance(), 130.0);
    assert_eq!(line.feedback().advance, 130.0);
    assert_eq!(
        raw(&line, AHEM),
        [
            (text::Range::new(0, 1), 35, 10.0),
            (text::Range::new(1, 2), 36, 10.0),
            (text::Range::new(3, 4), 37, 10.0),
            (text::Range::new(5, 6), 38, 10.0),
        ]
    );
    assert_eq!(
        paints(&actual),
        [
            (0, text::Range::new(0, 1), 35, 0.0),
            (0, text::Range::new(1, 2), 36, 10.0 + row.allocation),
            (0, text::Range::new(3, 4), 37, 80.0),
            (0, text::Range::new(5, 6), 38, 120.0),
        ]
    );
    assert_eq!(retained.measurements().alignment().advance(), 70.0);
    amounts(&retained, &[0.0]);
    copy(&actual, "AB\tC\tD");
}

#[test]
fn character_gap_before_mixed_period_tabs_fills_only_after_actual_response_correction() {
    tabs::<f32>();
    tabs::<f64>();
}

fn captured(source: &str, terminal: text::ParagraphTerminalEdge) -> PreparedParagraph {
    let style = text::Style {
        word_spacing: -10.0,
        ..own(0.0, Wrap::Word, text::OverflowWrap::Normal, false)
    };
    PreparedParagraph::at_boundary(
        vec![ParagraphPart::text(1, Source::new(source), style.clone())],
        style,
        ParagraphBaseDirection::LeftToRight,
        terminal,
    )
}

fn deferred<S: LayoutScalar>() {
    for (terminal, cause, reason) in [
        (
            text::ParagraphTerminalEdge::Unknown,
            text::SpacingDeferredCause::MissingTerminalBlockEdge,
            text::ProtocolError::MissingPhaseTwoContext(text::PhaseTwoContext::TerminalBlockEdge),
        ),
        (
            text::ParagraphTerminalEdge::NonBlockBoundary,
            text::SpacingDeferredCause::NonBlockParagraphBoundary,
            text::ProtocolError::UnsupportedContext {
                field: text::ProtocolContextField::CssParagraphBoundary,
            },
        ),
    ] {
        let paragraph = captured("AB  ", terminal);
        let actual = finite::<S>(&paragraph, 30.0, TextAlignLast::Justify);
        let line = lines(&actual).remove(0);
        assert_eq!(
            line.selected_line().request().context.spacing,
            text::SpacingPolicy::InterCharacter
        );
        assert_eq!(line.measurements().alignment().advance(), 30.0);
        assert_eq!(line.feedback().advance, 30.0);
        amounts(&line, &[5.0, 5.0, 0.0]);
        for (index, row) in line.spacing_slots().enumerate() {
            boundary(&row.slot, &paragraph, index, 0.0);
            assert_eq!(
                row.geometry,
                text::SpacingSlotGeometry::Boundary(text::SpacingBoundaryGeometry::Independent)
            );
            assert_eq!(
                row.stretchability,
                if index == 2 {
                    text::SpacingStretchability::Deferred(cause)
                } else {
                    text::SpacingStretchability::Direct
                }
            );
        }
        assert_eq!(
            raw(&line, AHEM),
            [
                (text::Range::new(0, 1), 35, 10.0),
                (text::Range::new(1, 2), 36, 10.0),
                (text::Range::new(2, 3), 3, 10.0),
                (text::Range::new(3, 4), 3, 10.0),
            ]
        );
        assert_eq!(
            paints(&actual)
                .iter()
                .map(|fact| fact.3)
                .collect::<Vec<_>>(),
            // Each SPACE has intrinsic10 + authored word(-10) = cell0.
            // Its glyph retains the independent word half(-5), so the two
            // trailing cells start30 while their unchanged raw packets paint25.
            [0.0, 15.0, 25.0, 25.0]
        );
        let layout = actual.placed_text()[0].layout();
        for (index, x) in [0.0, 15.0, 30.0, 30.0, 30.0].into_iter().enumerate() {
            assert_eq!(
                layout
                    .cursor(text::Cursor::new(index, text::Affinity::Before))
                    .unwrap()
                    .rect()
                    .origin
                    .x,
                x
            );
        }
        copy(&actual, "AB  ");

        // Two genuine trailing SPACE units retain one character boundary.
        // No Direct boundary exists to satisfy a positive budget instead.
        let only = captured("  ", terminal);
        let exact = finite::<S>(&only, 0.0, TextAlignLast::Justify);
        let exact = lines(&exact).remove(0);
        amounts(&exact, &[0.0]);
        boundary(&exact.spacing_slots().next().unwrap().slot, &only, 0, 0.0);
        assert_eq!(
            exact.spacing_slots().next().unwrap().stretchability,
            text::SpacingStretchability::Deferred(cause)
        );
        assert_eq!(exact.measurements().alignment().advance(), 0.0);
        let error = run::<S>(
            &only,
            AvailableOf::definite(S::from_f64(15.0)),
            PreferredSizeOf::px(S::from_f64(15.0)),
            TextAlignLast::Justify,
        )
        .unwrap_err();
        let LayoutErrorKindOf::Text(error) = error.kind() else {
            panic!("Text owns the terminal refusal: {error:?}")
        };
        assert!(
            matches!(error.detail(), Some(text::ErrorDetail::InvalidTextProtocol { reason: actual }) if *actual == reason),
            "{error:?}"
        );
        assert_eq!(line.measurements().alignment().advance(), 30.0);
    }
}

#[test]
fn direct_character_boundaries_fill_with_complete_zero_deferred_tail_and_demand_needed_tail() {
    deferred::<f32>();
    deferred::<f64>();
}
