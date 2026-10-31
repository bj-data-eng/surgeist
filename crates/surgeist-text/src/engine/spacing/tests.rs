//! Failure diagnostics are outside the scope-alignment payload contract.
//! This private owner-boundary probe uses an actual published Native provider
//! packet; it does not claim to replay a complete public Cell operation.

use super::*;
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use crate::*;
use skrifa::{MetadataProvider, raw::TableProvider};
use std::mem::size_of;

#[test]
fn shared_word_geometry_keeps_its_diagnostic_outside_the_payload_allowance() {
    let bytes =
        crate::shaping_test_fonts::range_ligature_ahem(&[35, 58], &[(0x20, 58), (0x41, 35)]);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert!(parsed.gsub().is_ok());
    assert_eq!(parsed.charmap().map('A'), Some(skrifa::GlyphId::new(35)));
    assert_eq!(parsed.charmap().map(' '), Some(skrifa::GlyphId::new(58)));
    for glyph in [35, 58] {
        assert_eq!(
            crate::test_fonts::u16_at(crate::test_fonts::table(&bytes, *b"hmtx"), glyph * 4,),
            1000
        );
    }
    let style = Style {
        font: Font::new().family("Ahem").feature("\"ccmp\" 1"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::none(),
        letter_spacing: 0.0,
        word_spacing: 0.0,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let extent = Range::new(0, 3);
    let mut source = Source::identified("A A", Id::from_u64(784_989_751), 1);
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
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
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
                spacing: SpacingPolicy::InterWord,
            },
        })
        .unwrap();
    let natural = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(natural.feedback().advance, 20.0);
    let raw = |line: &FinalizedLine| {
        line.run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), bytes.as_slice());
                assert_eq!(run.font_size(), 10.0);
                run.glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    let expected = [(Range::new(0, 2), 58, 10.0), (Range::new(2, 3), 35, 10.0)];
    assert_eq!(raw(&natural), expected);
    let [word] = selected.inner.basis.shaped.word_spacing.as_slice() else {
        panic!("the actual SPACE has one retained zero-authored feedback receipt");
    };
    assert_eq!(word.atoms, 1..2);
    assert_eq!(word.base, 1);
    assert_eq!((word.authored, word.allocation), (0.0, 0.0));
    assert_eq!(word.geometry, SpacingSlotGeometry::SharedProviderGroup);
    assert_eq!(word.shared, Some(0..2));
    let [component] = word.components.as_slice() else {
        panic!("one actual SPACE component in the shared A/SPACE invocation");
    };
    let original = &selected.inner.basis.shaped;
    let receipt = original.clusters[component.cluster]
        .provider_receipt
        .unwrap();
    assert_eq!(original.clusters[component.cluster].atoms, 1..2);
    assert_eq!(original.provider_groups[receipt].atoms, 0..2);
    let candidate = SeparatorCandidate {
        atoms: word.atoms.clone(),
        base: word.base,
    };
    let members = word
        .components
        .iter()
        .map(|row| row.cluster)
        .collect::<Vec<_>>();

    // The caller retains all Native input arenas. This helper owns only its
    // real transient geometry and diagnostics. A Some ledger is a legitimate
    // input: CellWordExecution invokes this same helper for CellWordPlan's
    // source-issued candidates with the real scope ledger during preparation.
    // No Cell or provider group is manufactured to exercise that owner boundary.
    // One actual component names one distinct provider receipt, so functional
    // geometry needs one usize buffer; its in-place ordering allocates nothing.
    // Derive the capacity independently with the same allocator request, rather
    // than copying a visitor total or granting guessed diagnostic headroom.
    let mut receipt_storage = Vec::<usize>::new();
    receipt_storage.try_reserve_exact(1).unwrap();
    let functional_bytes = receipt_storage.capacity() * size_of::<usize>();
    drop(receipt_storage);
    let limits = ScopeAlignmentLimits::new(4096, 4096, 1_000_000, functional_bytes);
    let mut geometry_ledger = Some(ScopeAlignmentBudget::new(limits, 0, 0).unwrap());
    assert_eq!(
        geometry_accounted(
            original,
            &members,
            candidate.atoms.clone(),
            &mut geometry_ledger
        )
        .unwrap(),
        (SpacingSlotGeometry::SharedProviderGroup, Some(0..2))
    );
    assert_eq!(geometry_ledger.as_ref().unwrap().usage().payload_bytes(), 0);
    assert_eq!(
        geometry_ledger
            .as_ref()
            .unwrap()
            .usage()
            .peak_payload_bytes(),
        functional_bytes
    );

    let mut shaped = original.clone();
    let mut ledger = Some(ScopeAlignmentBudget::new(limits, 0, 0).unwrap());
    let error = authored_unit_accounted(
        &mut shaped,
        &candidate,
        &members,
        4.0,
        SpacingStage::Preparation,
        |ledger| {
            let mut pieces = Vec::new();
            for atom in candidate.atoms.clone() {
                original_pieces(&prepared.inner.analyzed, atom, &mut pieces, ledger)?;
            }
            Ok(pieces)
        },
        &mut ledger,
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        ErrorCode::LayoutFailed,
        "actual diagnostic: {error:?}"
    );
    let Some(ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        pieces,
        provider_atoms,
        reason,
        ..
    }) = error.detail()
    else {
        panic!(
            "shared authored word geometry must retain its own diagnostic, not a payload refusal: {error:?}"
        );
    };
    assert_eq!(*stage, SpacingStage::Preparation);
    assert_eq!(*operation, SpacingOperation::AuthoredWord);
    assert_eq!(*reason, SpacingFailureReason::CrossUnitConstraint);
    assert_eq!(
        pieces.as_ref(),
        [SpacingPiece::Source {
            origin: 0,
            local: Range::new(1, 2),
            aggregate: Range::new(1, 2),
        }]
    );
    assert_eq!(provider_atoms.as_ref(), std::slice::from_ref(&(0..2)));
    assert_eq!(ledger.as_ref().unwrap().usage().payload_bytes(), 0);
    assert_eq!(
        ledger.as_ref().unwrap().usage().peak_payload_bytes(),
        functional_bytes
    );
    assert_eq!(
        shaped
            .glyphs
            .iter()
            .map(|glyph| (glyph.source, glyph.id, glyph.advance))
            .collect::<Vec<_>>(),
        original
            .glyphs
            .iter()
            .map(|glyph| (glyph.source, glyph.id, glyph.advance))
            .collect::<Vec<_>>()
    );
    assert_eq!(raw(&natural), expected);
    assert_eq!(natural.feedback().advance, 20.0);
}
