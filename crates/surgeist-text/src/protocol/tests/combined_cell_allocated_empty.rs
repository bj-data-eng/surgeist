//! A real lone outer Cell has no exterior allocation target, independently of
//! its inner EGC/glyph count or whether the accepted source has provider input.
//! Inner SPACE is source/payload, never an exterior separator allocation slot.

use super::*;

fn request(prepared: &PreparedText, spacing: SpacingPolicy) -> LineSelectionRequest {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    LineSelectionRequest {
        segment: segment.clone(),
        start: segment.start(),
        end: segment.end(),
        participants: prepared.participants().collect(),
        termination: PreparedTermination::Ordinary(segment.end_opportunity()),
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![],
        context: LineContext {
            alignment_insets: LineAlignmentInsets::ZERO,
            formatted_line: FormattedLineContext::default(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: LineBoxDirection::LeftToRight,
            orientation: LineOrientation::Vertical,
            start: LineStart::Paragraph,
            end: LineEnd::Paragraph,
            available_inline: None,
            indent: 0.0,
            alignment: Alignment::Start,
            tabs: TabContext::NoTabs,
            spacing,
        },
    }
}

fn output(prepared: &PreparedText, line: &FinalizedLine, text: &str, font: &[u8]) {
    one_em(line);
    assert_eq!(line.spacing_slots().count(), 0);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one true fixed outer square");
    };
    assert_eq!(cell.square().origin.x, 0.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.scope_displacement().get(), 0.0);
    let mut raw = Vec::new();
    for run in line.run_views() {
        assert!(run.cell().is_some());
        assert_eq!(run.face().bytes(), font);
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        for glyph in run.glyphs() {
            raw.push((
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
            ));
        }
    }
    let (expected, scale) = match text {
        "AA" => (
            vec![(35, 0.0, 0.0, 0.0, 10.0), (35, 10.0, 0.0, 0.0, 10.0)],
            0.5,
        ),
        "A A" => (
            vec![
                (35, 0.0, 0.0, 0.0, 10.0),
                (3, 10.0, 0.0, 0.0, 10.0),
                (35, 20.0, 0.0, 0.0, 10.0),
            ],
            1.0 / 3.0,
        ),
        " " => (vec![], 1.0),
        _ => unreachable!("closed genuine fixture set"),
    };
    assert_eq!(raw, expected);
    let placed = placed_cell(prepared, line.clone(), font, scale);
    assert_eq!(placed.source(), prepared.analyzed_source().source());
    assert_eq!(placed.bounds(), Rect::new(95.0, 20.0, 10.0, 10.0));
}

fn zero_targets(text: &str, whitespace: WhiteSpace, policy: SpacingPolicy) {
    let font = qualified_font();
    let (processed, analyzed) = captured(text, whitespace, 0.0);
    let used = if text == " " {
        vec![]
    } else {
        (0..text.len()).collect()
    };
    source_custody(&processed, &analyzed, &used, 0.0);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.clone()).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one authentic prepared Cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert_eq!(cell.has_provider_input(), text != " ");
    assert_eq!(
        cell.source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    let issued = prepared.cells().next().unwrap();
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        [PreparedParticipant::Cell(issued)]
    );
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        font.as_slice()
    );
    // This existing callable zero-extra path executes all source/provider/frame
    // premises BEFORE the non-None selection capability under test.
    let natural = prepared
        .select(request(&prepared, SpacingPolicy::None))
        .unwrap();
    assert_eq!(natural.spacing_slots().count(), 0);
    let baseline = natural
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    output(&prepared, &baseline, text, &font);
    let selected = prepared
        .select(request(&prepared, policy))
        .expect("valid non-None policy on one opaque Cell must succeed with zero exterior targets");
    assert_eq!(selected.spacing_slots().count(), 0);
    // Complete(empty) is exactly one amount for each of the zero issued slots.
    let finalized = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::Complete(vec![]),
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    output(&prepared, &finalized, text, &font);
    drop(system);
    let replay = finalized
        .with_spacing(SpacingAllocations::Complete(vec![]))
        .unwrap();
    source_custody(
        &processed,
        &replay.prepared_text().analyzed_source(),
        &used,
        0.0,
    );
    output(&prepared, &replay, text, &font);
}

#[test]
fn lone_aa_spaces_has_no_exterior_target() {
    zero_targets("AA", WhiteSpace::Preserve, SpacingPolicy::Spaces);
}
#[test]
fn lone_aa_inter_word_has_no_exterior_target() {
    zero_targets("AA", WhiteSpace::Preserve, SpacingPolicy::InterWord);
}
#[test]
fn lone_aa_inter_character_has_no_exterior_target() {
    zero_targets("AA", WhiteSpace::Preserve, SpacingPolicy::InterCharacter);
}
#[test]
fn lone_provider_empty_cell_spaces_has_no_exterior_target() {
    zero_targets(" ", WhiteSpace::Collapse, SpacingPolicy::Spaces);
}
#[test]
fn lone_provider_empty_cell_inter_word_has_no_exterior_target() {
    zero_targets(" ", WhiteSpace::Collapse, SpacingPolicy::InterWord);
}
#[test]
fn lone_provider_empty_cell_inter_character_has_no_exterior_target() {
    zero_targets(" ", WhiteSpace::Collapse, SpacingPolicy::InterCharacter);
}
#[test]
fn preserved_inner_space_is_not_an_outer_spaces_target() {
    zero_targets("A A", WhiteSpace::Preserve, SpacingPolicy::Spaces);
}
#[test]
fn preserved_inner_space_is_not_an_outer_inter_word_target() {
    zero_targets("A A", WhiteSpace::Preserve, SpacingPolicy::InterWord);
}
