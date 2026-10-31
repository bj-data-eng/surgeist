//! Ordinary authored tracking keeps Native character admission outside the
//! unrelated scope/Cell ledger, even when that ledger has no work allowance.

use super::*;
use crate::engine::{layout::data::CellArenaOwnership, spacing::character};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use skrifa::MetadataProvider;

#[test]
fn ordinary_native_tracking_admission_does_not_spend_an_unrelated_scope_ledger() {
    let font = parent_font();
    let face = skrifa::FontRef::new(&font).unwrap();
    assert_eq!(face.charmap().map('B'), Some(skrifa::GlyphId::new(36)));
    assert_eq!(sfnt::u16_at(sfnt::table(&font, *b"hmtx"), 36 * 4), 1000);
    let extent = Range::new(0, 2);
    let style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::none(),
        text_combine_upright: TextCombineUpright::None,
        letter_spacing: 2.0,
        word_spacing: 0.0,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let mut source = Source::identified("BB", Id::from_u64(784_948), 1);
    source.span(extent, style.clone());
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: style,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                origins: vec![ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: extent,
                    aggregate: extent,
                    kind: ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        Wrap::Word,
    );
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.combined_cells().len(), 0);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.clone()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .expect("ordinary genuine Ahem BB preparation succeeds before Native admission is tested");
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.cell_views().len(), 0);
    assert!(prepared.inner.shaped.cells.is_empty());
    assert_eq!(
        prepared.inner.shaped.cell_arenas,
        CellArenaOwnership::Native
    );
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let selected = prepared
        .select(LineSelectionRequest {
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
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .expect("the ordinary non-Cell authored-tracking selection is already admitted");
    assert!(selected.inner.basis.shaped.cells.is_empty());
    assert_eq!(
        selected.inner.basis.shaped.cell_arenas,
        CellArenaOwnership::Native
    );
    let graph = &selected.inner.basis.characters;
    assert_eq!(
        graph
            .units
            .iter()
            .map(|unit| unit
                .atoms
                .iter()
                .map(|atoms| (atoms.start, atoms.end))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [vec![(0, 1)], vec![(1, 2)]]
    );
    assert!(graph.units.iter().all(|unit| unit.atomics.is_empty()));
    let [boundary] = graph.boundaries.as_slice() else {
        panic!("one actual B/B tracking boundary");
    };
    assert_eq!(
        (boundary.before, boundary.after, boundary.authored),
        (0, 1, 2.0)
    );
    // Actual Native graph storage is not retained by the Cell/scope visitor.
    let mut cell_owned_allocations = 0;
    graph
        .visit_allocations(&mut |_| {
            cell_owned_allocations += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(cell_owned_allocations, 0);
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .expect("published ordinary tracking remains usable before the helper probe");
    assert_eq!(line.feedback().advance, 22.0); // Ahem10 + mean(2,2) + Ahem10.
    assert_eq!(line.cell_views().len(), 0);
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), font.as_slice());
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [(Range::new(0, 1), 36, 10.0), (Range::new(1, 2), 36, 10.0)]
    );
    // Valid independent scope-owner input: ample row/byte limits, but zero work.
    // Native admission owns neither this allowance nor any of its usage.
    let limits = ScopeAlignmentLimits::new(4096, 4096, 0, 16_777_216);
    let mut ledger = Some(ScopeAlignmentBudget::new(limits, 0, 0).unwrap());
    let before = ledger.as_ref().unwrap().usage();
    assert_eq!(before.structural_work(), 0);
    character::admit(
        selected.inner.presentation_input(),
        graph,
        false,
        &mut ledger,
    )
    .expect("ordinary Native admission must not spend an unrelated scope-owner ledger");
    assert_eq!(ledger.as_ref().unwrap().usage(), before);
}
