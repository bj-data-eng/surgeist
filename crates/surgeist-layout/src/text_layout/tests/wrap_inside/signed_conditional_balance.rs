//! Genuine issued SHY receipts and paragraph-quality consumers retain signed box demand.
use super::*;

fn signed_run<S: LayoutScalar>(
    fixture: &Captured,
    mode: TextWrapStyle,
    start_margin: f64,
    end_margin: f64,
    width: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let mut tree = text_tree::<S>(&fixture.binding, false).style(
        0,
        NodeInputOf {
            display: Display::Block,
            text_wrap_style: mode,
            direction: Direction::Ltr,
            ..NodeInputOf::default()
        },
    );
    for row in &fixture.binding.controls {
        let old = tree.layout_input(row.node).as_inline_boundary().unwrap();
        let margin = match old.kind() {
            InlineBoundaryKind::Start => start_margin,
            InlineBoundaryKind::End => end_margin,
        };
        let edge =
            InlineBoundaryEdgeOf::try_new(S::from_f64(margin), S::from_f64(0.0), S::from_f64(0.0))
                .unwrap();
        tree = tree.input(
            row.node,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::box_edge(
                    old.kind(),
                    old.metrics(),
                    edge,
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                )
                .with_direction(old.direction())
                .with_writing_mode(old.writing_mode())
                .with_vertical_align(old.vertical_align()),
            ),
        );
    }
    // Use the real captured scope and its existing Ahem8/10 strut declaration.
    // This fact does not allocate either off-selection edge's flow.
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        WrappingInput {
            scopes: policies(fixture, &[WrapInside::Auto]),
            child_order: vec![],
        },
        vec![],
    )
    .unwrap();
    compute_layout_with_flow(
        &flow,
        0,
        request_at(AvailableOf::definite(S::from_f64(width))),
    )
    .unwrap()
}

fn actual_font_premises() {
    let u16_at = |at: usize| u16::from_be_bytes(AHEM[at..at + 2].try_into().unwrap());
    let u32_at = |at: usize| u32::from_be_bytes(AHEM[at..at + 4].try_into().unwrap()) as usize;
    let table = |tag: &[u8; 4]| {
        let directory = (0..usize::from(u16_at(4)))
            .map(|i| 12 + i * 16)
            .find(|&at| &AHEM[at..at + 4] == tag)
            .unwrap();
        u32_at(directory + 8)
    };
    assert_eq!(u16_at(table(b"head") + 18), 1000);
    // These are actual hmtx operands, not widths inferred from Layout output.
    for glyph in [3, 35, 36, 37, 38, 39] {
        assert_eq!(u16_at(table(b"hmtx") + glyph * 4), 1000);
    }
}

fn packet(line: &text::FinalizedLine) -> Vec<(u32, f32, f64)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
            let x = run.origin().x;
            run.glyphs()
                .map(move |glyph| {
                    (
                        glyph.id(),
                        glyph.advance(),
                        x + glyph.pen() + f64::from(glyph.x()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn fit(line: &text::FinalizedLine, advance: f64, width: f64) {
    let selected = line.selected_line();
    let request = selected.request();
    assert_eq!(request.context.available_inline, Some(width));
    let fit = line.measurements().margin_edge_fit();
    assert_eq!(fit.authority(), text::MarginEdgeFitAuthority::CssComposed);
    assert_eq!(fit.advance(), advance);
    assert_eq!(fit.required_inline_size(), advance);
    assert_eq!(line.feedback().advance, advance);
}

fn issued_both_sides<S: LayoutScalar>() {
    actual_font_premises();
    let style = text::Style {
        wrap: text::Wrap::Word,
        white_space: text::WhiteSpace::Collapse,
        hyphens: text::Hyphens::Manual,
        locale: Some("und-Latn".into()),
        hyphenate_character: text::HyphenateCharacter::Auto,
        native_hyphenation_convention: Some(
            text::NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                "A".into(),
                text::HyphenationPlacement::BothSides,
            )
            .unwrap(),
        ),
        ..ahem_style(10.0, 10.0)
    };
    let fixture = captured(
        "AB\u{ad}CDE",
        style,
        std::slice::from_ref(&(0..6)),
        &[],
        false,
        true,
        AHEM,
    );
    assert_eq!(fixture.binding.prepared.origin_views().count(), 1);
    let wide = signed_run::<S>(&fixture, TextWrapStyle::Auto, 0.0, 0.0, 50.0);
    assert_eq!(cuts(&wide), [6]);
    let wide_line = finalized(&wide).remove(0);
    assert!(wide_line.selected_line().request().alternative.is_none());
    assert!(wide_line.hyphenation_continuation().is_none());
    assert_eq!(
        packet(&wide_line),
        [
            (35, 10.0, 0.0),
            (36, 10.0, 10.0),
            (37, 10.0, 20.0),
            (38, 10.0, 30.0),
            (39, 10.0, 40.0)
        ]
    );
    fit(&wide_line, 50.0, 50.0);

    // Whole content50 minus the two REAL edges5+10 is35, still over band30.
    // Outgoing AB+A is30-5=25; receiving A+CDE is40-10=30.
    // A future end edge cannot lower the outgoing25 to15, and the already
    // consumed start edge cannot lower the receiver30 to25.
    let batch = signed_run::<S>(&fixture, TextWrapStyle::Auto, -5.0, -10.0, 30.0);
    assert_eq!(cuts(&batch), [3, 6]);
    assert_eq!(
        batch.placed_text()[0].layout().source().text(),
        &text::Text::from("AB\u{ad}CDE")
    );
    let rows = finalized(&batch);
    assert_eq!(rows.len(), 2);
    let selected = rows[0].selected_line();
    let alternative = selected
        .request()
        .alternative
        .as_ref()
        .expect("actual issued SHY alternative");
    let text::PreparedTermination::Ordinary(offer) = &selected.request().termination else {
        panic!("the manual alternative belongs to its actual issued ordinary offer")
    };
    assert_eq!(alternative.view().opportunity(), Some(offer.clone()));
    assert_eq!(alternative.view().boundary(), selected.request().end);
    let text::PreparedAlternativeDefinition::Source { origin, source } =
        alternative.view().definition()
    else {
        panic!("authored SHY must retain its actual source definition")
    };
    assert_eq!(source, text::Range::new(2, 3));
    assert_eq!(
        origin.view().source().id(),
        Some(text::Id::from_u64(650_000))
    );
    assert_eq!(origin.view().source().revision(), 31);
    let outgoing = rows[0]
        .hyphenation_continuation()
        .expect("BothSides issues a real receiver");
    assert_eq!(rows[1].selected_line().request().incoming, Some(outgoing));
    assert!(rows[1].selected_line().request().alternative.is_none());
    assert!(rows[1].hyphenation_continuation().is_none());
    fit(&rows[0], 25.0, 30.0);
    fit(&rows[1], 30.0, 30.0);
    assert_eq!(
        packet(&rows[0]),
        [(35, 10.0, -5.0), (36, 10.0, 5.0), (35, 10.0, 15.0)]
    );
    assert_eq!(
        packet(&rows[1]),
        [
            (35, 10.0, 0.0),
            (37, 10.0, 10.0),
            (38, 10.0, 20.0),
            (39, 10.0, 30.0)
        ]
    );
    assert_eq!(
        rows[0]
            .control_slots()
            .map(|c| c.flow_interval())
            .collect::<Vec<_>>(),
        [(0.0, -5.0)]
    );
    assert_eq!(
        rows[1]
            .control_slots()
            .map(|c| c.flow_interval())
            .collect::<Vec<_>>(),
        [(40.0, 30.0)]
    );
    assert_eq!(
        rows[0]
            .selected_line()
            .request()
            .end
            .view()
            .source_after()
            .get(),
        3
    );
    assert_eq!(
        rows[1]
            .selected_line()
            .request()
            .start
            .view()
            .source_after()
            .get(),
        3
    );
    // A second authentic operation must not replace the first allocation's
    // source/alternative packet or consume its immutable continuation receipt.
    let again = signed_run::<S>(&fixture, TextWrapStyle::Auto, -5.0, -10.0, 30.0);
    assert_eq!(cuts(&again), [3, 6]);
    assert_eq!(
        packet(&rows[0]),
        [(35, 10.0, -5.0), (36, 10.0, 5.0), (35, 10.0, 15.0)]
    );
    assert_eq!(packet(&wide_line).len(), 5);
}

fn balanced_signed_end<S: LayoutScalar>() {
    actual_font_premises();
    let fixture = captured(
        "AAA BB C",
        text::Style {
            wrap: text::Wrap::Word,
            white_space: text::WhiteSpace::Collapse,
            hyphens: text::Hyphens::None,
            ..ahem_style(10.0, 10.0)
        },
        std::slice::from_ref(&(0..8)),
        &[],
        false,
        true,
        AHEM,
    );
    // Zero-component preservation control: the maintained Balance oracle is30/40,
    // whereas Auto uses60/10. Trailing collapsible SPACE adds no line advance.
    let native = signed_run::<S>(&fixture, TextWrapStyle::Balance, 0.0, 0.0, 60.0);
    assert_eq!(cuts(&native), [4, 8]);
    let native_rows = finalized(&native);
    fit(&native_rows[0], 30.0, 60.0);
    fit(&native_rows[1], 40.0, 60.0);
    // The whole candidate80-10=70 still needs two lines. The end margin belongs
    // only to the last one. Auto60/0 has residuals0/60, variance900; Balance
    // 30/30 has residuals30/30, variance0. A prefix-max/clamp loses this result.
    let auto = signed_run::<S>(&fixture, TextWrapStyle::Auto, 0.0, -10.0, 60.0);
    assert_eq!(cuts(&auto), [7, 8]);
    let auto_rows = finalized(&auto);
    fit(&auto_rows[0], 60.0, 60.0);
    fit(&auto_rows[1], 0.0, 60.0);
    let balanced = signed_run::<S>(&fixture, TextWrapStyle::Balance, 0.0, -10.0, 60.0);
    assert_eq!(cuts(&balanced), [4, 8]);
    let rows = finalized(&balanced);
    fit(&rows[0], 30.0, 60.0);
    fit(&rows[1], 30.0, 60.0);
    for row in &rows {
        assert_eq!(row.prepared_text(), fixture.binding.prepared);
        assert!(row.selected_line().request().alternative.is_none());
        assert!(row.selected_line().request().incoming.is_none());
        assert!(matches!(
            &row.selected_line().request().termination,
            text::PreparedTermination::Ordinary(_)
        ));
    }
    assert_eq!(
        rows[0]
            .control_slots()
            .map(|c| c.flow_interval())
            .collect::<Vec<_>>(),
        [(0.0, 0.0)]
    );
    assert_eq!(
        rows[1]
            .control_slots()
            .map(|c| c.flow_interval())
            .collect::<Vec<_>>(),
        [(40.0, 30.0)]
    );
    // Assert actual letter placement independently of whether a collapsed
    // terminal SPACE retains a nonpainting provider row. The internal SPACE
    // still contributes10: C begins30 after the two10px Bs.
    assert_eq!(
        packet(&rows[0])
            .into_iter()
            .filter(|g| g.0 != 3)
            .collect::<Vec<_>>(),
        [(35, 10.0, 0.0), (35, 10.0, 10.0), (35, 10.0, 20.0)]
    );
    assert_eq!(
        packet(&rows[1])
            .into_iter()
            .filter(|g| g.0 != 3)
            .collect::<Vec<_>>(),
        [(36, 10.0, 0.0), (36, 10.0, 10.0), (37, 10.0, 30.0)]
    );
    assert_eq!(
        balanced.placed_text()[0].layout().source().text(),
        &text::Text::from("AAA BB C")
    );
    assert_eq!(auto_rows[0].feedback().advance, 60.0);
    assert_eq!(native_rows[1].feedback().advance, 40.0);
}

#[test]
fn signed_box_edges_preserve_real_shy_alternative_and_issued_receiving_frame() {
    issued_both_sides::<f32>();
    issued_both_sides::<f64>();
}

#[test]
fn actual_layout_balance_uses_only_its_selected_signed_end_demand() {
    balanced_signed_end::<f32>();
    balanced_signed_end::<f64>();
}
