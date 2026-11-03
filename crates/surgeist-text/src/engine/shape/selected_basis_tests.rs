//! Executed-provider and native-receipt qualification of immutable selection.
use super::*;
use crate::{
    Affinity, Alignment, AnalysisTypography, AnalyzedSource, FinalizedLine, Font,
    FontCatalogueMode, LineContext, LineEnd, LineOrientation, LineSelectionRequest, LineStart,
    ParagraphBaseDirection, ParagraphInput, ParagraphOrigin, ParagraphOriginKind, ParagraphStrut,
    PreparationOptions, PreparedBreakKind, PreparedTermination, PreparedText, SelectedLine, Source,
    SpacingAllocation, SpacingAllocations, SpacingPolicy, Style, System, SystemOptions, TabContext,
    WhiteSpace, Wrap,
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
};

#[path = "../../../tests/default_ignorable/font.rs"]
mod font;

const LTR: ParagraphBaseDirection = ParagraphBaseDirection::LeftToRight;
const RTL: ParagraphBaseDirection = ParagraphBaseDirection::RightToLeft;

fn own(tracking: f64) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: crate::LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        wrap: Wrap::Word,
        letter_spacing: tracking,
        ..Style::default()
    }
}

fn input(source: &Source, style: Style, direction: ParagraphBaseDirection) -> ParagraphInput {
    let extent = Range::new(0, source.text().len());
    ParagraphInput {
        source: source.clone(),
        base_style: style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
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
    }
}

fn system(bytes: &[u8]) -> System {
    let mut value = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    value.register_fonts(bytes.to_vec()).unwrap();
    value
}

fn prepare(
    system: &mut System,
    value: ParagraphInput,
    orientation: LineOrientation,
) -> PreparedText {
    let typography = if orientation == LineOrientation::Vertical {
        AnalysisTypography::Upright {
            computed_direction: value.base_direction,
        }
    } else {
        AnalysisTypography::HorizontalComposition
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, typography).unwrap(),
        Wrap::Word,
    );
    system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap()
}

fn request(
    prepared: &PreparedText,
    direction: ParagraphBaseDirection,
    method: SpacingPolicy,
) -> LineSelectionRequest {
    let offer = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    LineSelectionRequest {
        inline_boxes: None,
        segment: crate::test_wrapping::only_source_segment(prepared),
        boundary_context: vec![],
        incoming: None,
        start: prepared.boundary_views().next().unwrap().handle(),
        end: offer.view().boundary(),
        termination: PreparedTermination::Ordinary(offer),
        alternative: None,
        participants: prepared.participants().collect(),
        atomics: vec![],
        controls: vec![],
        line_box_struts: vec![],
        line_strut: None,
        context: LineContext {
            alignment_insets: crate::LineAlignmentInsets::ZERO,
            formatted_line: crate::FormattedLineContext::default(),
            base_direction: direction,
            line_box_direction: if direction == LTR {
                crate::LineBoxDirection::LeftToRight
            } else {
                crate::LineBoxDirection::RightToLeft
            },
            orientation: prepared.orientation(),
            start: LineStart::Paragraph,
            end: LineEnd::Paragraph,
            available_inline: None,
            indent: 0.0,
            alignment: Alignment::Left,
            tabs: TabContext::NoTabs,
            spacing: method,
        },
    }
}

fn amounts(selected: &SelectedLine, values: &[f64]) -> SpacingAllocations {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), values.len());
    SpacingAllocations::Complete(
        slots
            .into_iter()
            .zip(values)
            .map(|(slot, &amount)| SpacingAllocation { slot, amount })
            .collect(),
    )
}

type Packet = (Range, u32, f32, f32, f32);
fn packets(line: &FinalizedLine, bytes: &[u8]) -> Vec<Packet> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
            assert_eq!(
                run.face().catalogue_identity(),
                line.prepared_text().font_catalogue_identity()
            );
            let instance = run.effective_instance().unwrap();
            assert!(instance.face().is_same_face(&run.face()));
            assert_eq!(instance.font_size(), 10.0);
            run.glyphs()
                .map(|g| (g.range(), g.id(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn inline_paints(line: &FinalizedLine) -> Vec<f64> {
    line.run_views()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .map(move |glyph| origin.x + glyph.pen())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn resource_premises(bytes: &[u8]) {
    let parsed = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    let metrics = parsed.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for (ch, id, advance) in [
        ('A', font::A, 1000.0),
        ('B', font::B, 2000.0),
        ('R', font::R, 1500.0),
        ('C', font::D, 700.0),
        ('Q', font::Q, 2500.0),
        (' ', font::SPACE, 1000.0),
    ] {
        assert_eq!(parsed.charmap().map(ch), Some(GlyphId::new(u32::from(id))));
        assert_eq!(
            metrics.advance_width(GlyphId::new(u32::from(id))),
            Some(advance)
        );
    }
}

fn assert_calls(calls: usize) {
    assert_eq!(
        provider_calls(),
        calls,
        "retained allocation must not invoke the provider"
    );
}

#[test]
fn first_and_repeated_allocation_use_one_actual_selected_provider_basis_in_native_frames() {
    let bytes = font::fixture(None, font::Action::None);
    resource_premises(&bytes);
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
        LineOrientation::Vertical,
    ] {
        let mut system = system(&bytes);
        let source = Source::identified("AB", crate::Id::from_u64(1566070), 1);
        let prepared = prepare(&mut system, input(&source, own(3.0), LTR), orientation);
        let second = if orientation == LineOrientation::Vertical {
            10.0
        } else {
            20.0
        };
        if orientation == LineOrientation::Vertical {
            let face = system.registered_font_faces().next().unwrap().to_owned();
            let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
            for glyph in [font::A, font::B] {
                assert_eq!(
                    instance
                        .vertical_glyph_metrics(u32::from(glyph), system.font_catalogue_identity())
                        .unwrap()
                        .advance(),
                    10.0
                );
            }
        }
        reset_provider_calls();
        let selected = prepared
            .select(request(&prepared, LTR, SpacingPolicy::InterCharacter))
            .unwrap();
        let calls = provider_calls();
        assert!(
            calls > 0,
            "select must execute an actual selected provider invocation"
        );
        assert_eq!(selected.spacing_slots().len(), 1);
        let basis = &selected.inner.basis;
        let exclusions = exclusion_rows(&selected.inner.basis.shaped);
        assert!(exclusions.is_empty());
        let old_catalogue = prepared.font_catalogue_identity().clone();
        system.refresh_fonts().unwrap();
        assert_ne!(system.font_catalogue_identity(), &old_catalogue);
        drop(system);
        assert_calls(calls);
        let natural = selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_calls(calls);
        assert_eq!(natural.feedback().advance, 13.0 + second);
        assert_eq!(inline_paints(&natural), [0.0, 13.0]);
        let raw = packets(&natural, &bytes);
        let retained_face = natural.run_views().next().unwrap().face().to_owned();
        assert_eq!(
            raw.iter().map(|g| (g.1, g.4)).collect::<Vec<_>>(),
            [
                (u32::from(font::A), 10.0),
                (u32::from(font::B), second as f32)
            ]
        );
        let repeated = selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_calls(calls);
        assert_eq!(packets(&repeated, &bytes), raw);
        let expanded = selected
            .finalize(crate::LineAllocations {
                spacing: amounts(&selected, &[4.0]),
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_calls(calls);
        assert_eq!(expanded.feedback().advance, 17.0 + second);
        assert_eq!(inline_paints(&expanded), [0.0, 17.0]);
        let replaced = expanded.with_spacing(amounts(&selected, &[1.0])).unwrap();
        assert_calls(calls);
        assert_eq!(replaced.feedback().advance, 14.0 + second);
        assert_eq!(inline_paints(&replaced), [0.0, 14.0]);
        let restored = replaced.with_spacing(SpacingAllocations::None).unwrap();
        assert_calls(calls);
        assert_eq!(restored.feedback().advance, 13.0 + second);
        let zeros = expanded.with_spacing(amounts(&selected, &[0.0])).unwrap();
        assert_calls(calls);
        assert_eq!(zeros.feedback().advance, 13.0 + second);
        let error = natural
            .with_spacing(amounts(&selected, &[f64::from_bits(1)]))
            .unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
        assert_calls(calls);
        let valid = natural.with_spacing(amounts(&selected, &[2.0])).unwrap();
        assert_calls(calls);
        assert_eq!(valid.feedback().advance, 15.0 + second);
        for line in [
            &natural, &repeated, &expanded, &replaced, &restored, &zeros, &valid,
        ] {
            assert_eq!(line.selected_line(), selected);
            assert_eq!(packets(line, &bytes), raw);
            assert!(std::ptr::eq(&line.inner.selected.basis, basis));
            assert_eq!(
                exclusion_rows(&line.inner.selected.basis.shaped),
                exclusions
            );
            assert_eq!(
                line.prepared_text().font_catalogue_identity(),
                &old_catalogue
            );
            for run in line.run_views() {
                assert!(run.face().is_same_face(&retained_face.face()));
            }
        }
        assert_eq!(natural.feedback().advance, 13.0 + second);
        assert_eq!(expanded.spacing_slots().next().unwrap().allocation, 4.0);
        assert_eq!(replaced.spacing_slots().next().unwrap().allocation, 1.0);
        assert_eq!(restored.spacing_slots().next().unwrap().allocation, 0.0);
        assert_eq!(natural.prepared_text().analyzed_source().source(), &source);
    }
}

fn exclusion_rows(shaped: &ShapedData) -> Vec<(usize, bool)> {
    shaped
        .character_exclusions
        .iter()
        .map(|(&atom, outcome)| {
            (
                atom,
                matches!(
                    outcome,
                    super::super::spacing::character::Exclusion::Deleted
                ),
            )
        })
        .collect()
}

fn source_copy(prepared: &PreparedText, line: FinalizedLine, source: &Source) {
    let layout = crate::Layout::from_finalized_lines(crate::PlacementRequest {
        prepared: prepared.clone(),
        coverage: crate::PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(crate::LinePlacement {
            line,
            anchor: crate::LineAnchor {
                inline_origin: 0.0,
                baseline: 8.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), source);
    let positions = layout.positions().collect::<Vec<_>>();
    let first = positions
        .iter()
        .find(|position| position.source_cursor() == crate::Cursor::new(0, Affinity::After))
        .unwrap();
    let last = positions
        .iter()
        .find(|position| {
            position.source_cursor() == crate::Cursor::new(source.text().len(), Affinity::Before)
        })
        .unwrap();
    let selection = layout.position_selection(first, last).unwrap();
    assert_eq!(selection.source_text(), source.text());
    assert_eq!(selection.plaintext(), source.text());
}

fn literal_control(bytes: &[u8], text: &str, expected: &[(u32, i32)]) {
    resource_premises(bytes);
    assert_eq!(font::native_glyphs(bytes, text), expected);
}

#[test]
fn actual_residual_only_receipts_publish_a_complete_empty_basis_and_keep_source_custody() {
    let bytes = font::fixture(Some('\u{2065}'), font::Action::None);
    literal_control(&bytes, "ACB", &[(35, 1000), (100, 700), (36, 2000)]);
    for direction in [LTR, RTL] {
        let source = Source::new("\u{2065}");
        let mut system = system(&bytes);
        let prepared = prepare(
            &mut system,
            input(&source, own(3.0), direction),
            LineOrientation::Horizontal,
        );
        assert_eq!(exclusion_rows(&prepared.inner.shaped), [(0, false)]);
        assert_eq!(
            prepared
                .unit_views()
                .map(|u| (u.source_range(), u.advance()))
                .collect::<Vec<_>>(),
            [(Range::new(0, 1), 0.0)]
        );
        reset_provider_calls();
        let selected = prepared
            .select(request(&prepared, direction, SpacingPolicy::InterCharacter))
            .unwrap();
        let calls = provider_calls();
        assert!(calls > 0);
        assert_eq!(exclusion_rows(&selected.inner.basis.shaped), [(0, false)]);
        assert_eq!(selected.spacing_slots().len(), 0);
        let line = selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_calls(calls);
        assert_eq!(line.feedback().advance, 0.0);
        assert_eq!(line.feedback().line_height, 10.0);
        assert!(packets(&line, &bytes).is_empty());
        assert_eq!(line.outline_ink().unwrap(), crate::LineOutlineInk::Empty);
        let zero = line.with_spacing(amounts(&selected, &[])).unwrap();
        assert_calls(calls);
        assert_eq!(zero.feedback().advance, 0.0);
        source_copy(&prepared, zero, &source);
    }
}

#[test]
fn residual_selector_and_repeated_edge_residuals_have_one_real_neighbor_gap_and_zero_own_contributions()
 {
    let bytes = font::fixture(None, font::Action::None);
    literal_control(&bytes, "AB", &[(35, 1000), (36, 2000)]);
    for direction in [LTR, RTL] {
        for (text, expected_exclusions) in [
            ("A\u{180f}B", vec![(1, false)]),
            (
                "\u{2065}A\u{180f}\u{2065}B\u{2065}",
                vec![(0, false), (2, false), (3, false), (5, false)],
            ),
        ] {
            let source = Source::new(text);
            let mut system = system(&bytes);
            let prepared = prepare(
                &mut system,
                input(&source, own(3.0), direction),
                LineOrientation::Horizontal,
            );
            assert_eq!(exclusion_rows(&prepared.inner.shaped), expected_exclusions);
            for (index, unit) in prepared.unit_views().enumerate() {
                if expected_exclusions.iter().any(|(atom, _)| *atom == index) {
                    assert_eq!(
                        unit.advance(),
                        0.0,
                        "excluded source input keeps no independent spacing"
                    );
                }
            }
            reset_provider_calls();
            let selected = prepared
                .select(request(&prepared, direction, SpacingPolicy::InterCharacter))
                .unwrap();
            let calls = provider_calls();
            assert!(calls > 0);
            assert_eq!(
                exclusion_rows(&selected.inner.basis.shaped),
                expected_exclusions
            );
            assert_eq!(selected.spacing_slots().len(), 1);
            let natural = selected
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            assert_calls(calls);
            assert_eq!(natural.feedback().advance, 33.0);
            assert_eq!(inline_paints(&natural), [0.0, 13.0]);
            assert_eq!(
                packets(&natural, &bytes)
                    .iter()
                    .map(|p| (p.1, p.4))
                    .collect::<Vec<_>>(),
                [(35, 10.0), (36, 20.0)]
            );
            let allocated = natural.with_spacing(amounts(&selected, &[4.0])).unwrap();
            assert_calls(calls);
            assert_eq!(allocated.feedback().advance, 37.0);
            assert_eq!(inline_paints(&allocated), [0.0, 17.0]);
            assert_eq!(packets(&allocated, &bytes), packets(&natural, &bytes));
            assert_eq!(natural.feedback().advance, 33.0);
            source_copy(&prepared, allocated, &source);
        }
    }
}

fn optional_di_font() -> Vec<u8> {
    // Actual D->R,Q then Q->[]; surviving R is Normal. Only the real feature
    // record changes from required ccmp to optional liga. The original font's
    // licensed raw metric R1500 remains the independent15px output oracle.
    let bytes = font::fixture(Some('\u{2065}'), font::Action::PartialDelete);
    let mut gsub = crate::test_fonts::table(&bytes, *b"GSUB").to_vec();
    let features = usize::from(u16::from_be_bytes(gsub[6..8].try_into().unwrap()));
    assert_eq!(&gsub[features + 2..features + 6], b"ccmp");
    gsub[features + 2..features + 6].copy_from_slice(b"liga");
    crate::test_fonts::replace_tables(&bytes, vec![(*b"GSUB", gsub)])
}

#[test]
fn optional_plan_changes_normal_di_to_selected_residual_before_complete_handles_are_issued() {
    let bytes = optional_di_font();
    literal_control(&bytes, "ACB", &[(35, 1000), (58, 1500), (36, 2000)]);
    assert_eq!(
        font::native_glyphs(&bytes, "A\u{2065}B"),
        [(35, 1000), (58, 1500), (36, 2000)]
    );
    let source = Source::new("A\u{2065}B");
    let mut system = system(&bytes);
    let prepared = prepare(
        &mut system,
        input(&source, own(0.0), LTR),
        LineOrientation::Horizontal,
    );
    assert!(
        exclusion_rows(&prepared.inner.shaped).is_empty(),
        "preparation actually retains the surviving Normal descendant"
    );
    assert_eq!(
        prepared
            .unit_views()
            .map(|u| u.advance())
            .collect::<Vec<_>>(),
        [10.0, 15.0, 20.0]
    );
    let ordinary = prepared
        .select(request(&prepared, LTR, SpacingPolicy::None))
        .unwrap();
    let ordinary_line = ordinary
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        packets(&ordinary_line, &bytes)
            .iter()
            .map(|p| (p.1, p.4))
            .collect::<Vec<_>>(),
        [(35, 10.0), (58, 15.0), (36, 20.0)]
    );
    assert_eq!(ordinary_line.feedback().advance, 45.0);
    reset_provider_calls();
    let selected = prepared
        .select(request(&prepared, LTR, SpacingPolicy::InterCharacter))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    assert_eq!(
        exclusion_rows(&selected.inner.basis.shaped),
        [(1, false)],
        "the actual selected optional-feature plan makes D residual"
    );
    assert_eq!(
        selected.spacing_slots().len(),
        1,
        "complete selected A-B basis must not retain preparation's A-D and D-B candidates"
    );
    let slot = selected.spacing_slots().next().unwrap();
    let crate::SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slot.target()
    else {
        panic!("actual neighbor boundary")
    };
    let origin = prepared.origin_views().next().unwrap().handle();
    assert_eq!(
        visual_before
            .pieces()
            .map(|p| p.owner())
            .collect::<Vec<_>>(),
        [crate::SpacingSlotOwner::Source {
            origin: origin.clone(),
            local: Range::new(0, 1),
            aggregate: Range::new(0, 1)
        }]
    );
    assert_eq!(
        visual_after.pieces().map(|p| p.owner()).collect::<Vec<_>>(),
        [crate::SpacingSlotOwner::Source {
            origin,
            local: Range::new(2, 3),
            aggregate: Range::new(2, 3)
        }]
    );
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_calls(calls);
    assert_eq!(natural.feedback().advance, 30.0);
    assert_eq!(inline_paints(&natural), [0.0, 10.0]);
    let allocated = natural.with_spacing(amounts(&selected, &[4.0])).unwrap();
    assert_calls(calls);
    assert_eq!(allocated.feedback().advance, 34.0);
    assert_eq!(inline_paints(&allocated), [0.0, 14.0]);
    assert_eq!(packets(&allocated, &bytes), packets(&natural, &bytes));
    let other = prepared
        .select(request(&prepared, LTR, SpacingPolicy::InterCharacter))
        .unwrap();
    assert_ne!(other, selected);
    let calls = provider_calls();
    let error = natural.with_spacing(amounts(&other, &[0.0])).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextProtocol {
            reason: crate::ProtocolError::ForeignSelectedLine
        })
    ));
    assert_calls(calls);
    assert_eq!(
        natural
            .with_spacing(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        30.0
    );
    assert_calls(calls);
    assert_eq!(ordinary_line.feedback().advance, 45.0);
    source_copy(&prepared, allocated, &source);
}

fn neutral_receipt(
    bytes: &[u8],
    text: &str,
    expected: &[(u32, f32)],
    exclusions: &[(usize, bool)],
) {
    let source = Source::new(text);
    let mut system = system(bytes);
    let prepared = prepare(
        &mut system,
        input(&source, own(0.0), LTR),
        LineOrientation::Horizontal,
    );
    reset_provider_calls();
    let selected = prepared
        .select(request(&prepared, LTR, SpacingPolicy::None))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    assert_eq!(exclusion_rows(&selected.inner.basis.shaped), exclusions);
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_calls(calls);
    assert_eq!(
        packets(&line, bytes)
            .iter()
            .map(|p| (p.1, p.4))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        line.feedback().advance,
        expected
            .iter()
            .map(|(_, advance)| f64::from(*advance))
            .sum::<f64>()
    );
    assert_eq!(line.feedback().line_height, 10.0);
    source_copy(&prepared, line, &source);
}

#[test]
fn real_gsub_final_receipts_distinguish_later_deletion_mixed_descendants_and_joint_normal_output() {
    use font::Action as A;
    for action in [A::Delete, A::ReplaceThenDelete, A::DeleteBoth] {
        let bytes = font::fixture(Some('\u{2065}'), action);
        literal_control(&bytes, "ACB", &[(35, 1000), (36, 2000)]);
        assert!(font::native_glyphs(&bytes, "\u{2065}").is_empty());
        neutral_receipt(&bytes, "\u{2065}", &[], &[(0, true)]);
        neutral_receipt(
            &bytes,
            "A\u{2065}B",
            &[(35, 10.0), (36, 20.0)],
            &[(1, true)],
        );
    }
    let partial = font::fixture(Some('\u{2065}'), A::PartialDelete);
    literal_control(&partial, "C", &[(58, 1500)]);
    neutral_receipt(&partial, "\u{2065}", &[(58, 15.0)], &[]);
    let consumed = font::fixture(Some('\u{2065}'), A::Consume);
    literal_control(&consumed, "ACB", &[(58, 1500), (36, 2000)]);
    neutral_receipt(&consumed, "A\u{2065}B", &[(58, 15.0), (36, 20.0)], &[]);
    let skipped = font::fixture(Some('\u{2065}'), A::Skip);
    literal_control(&skipped, "AB", &[(58, 1500)]);
    assert_eq!(
        font::native_glyphs(&skipped, "A\u{2065}B"),
        [(58, 1500), (100, 700)]
    );
    neutral_receipt(&skipped, "A\u{2065}B", &[(58, 15.0)], &[(1, false)]);
}

#[test]
fn real_morx_final_receipts_preserve_normal_insertions_and_retire_later_deleted_descendants() {
    use font::MorxAction as M;
    for with_gpos in [false, true] {
        for action in [M::Delete, M::NoncontextualDelete, M::ReplaceThenDelete] {
            let bytes = font::morx_fixture(action, with_gpos);
            literal_control(&bytes, "ACB", &[(35, 1000), (36, 2000)]);
            neutral_receipt(
                &bytes,
                "A\u{2065}B",
                &[(35, 10.0), (36, 20.0)],
                &[(1, true)],
            );
        }
        for marked in [false, true] {
            for delete_inserted in [false, true] {
                let bytes = font::morx_fixture(
                    M::Insert {
                        marked,
                        delete_inserted,
                    },
                    with_gpos,
                );
                literal_control(
                    &bytes,
                    "C",
                    if delete_inserted {
                        &[(100, 700)]
                    } else {
                        &[(58, 1500), (100, 700)]
                    },
                );
                neutral_receipt(
                    &bytes,
                    "\u{2065}",
                    if delete_inserted { &[] } else { &[(58, 15.0)] },
                    if delete_inserted { &[(0, false)] } else { &[] },
                );
            }
        }
        for delete_joint in [false, true] {
            let bytes = font::morx_fixture(M::Consume { delete_joint }, with_gpos);
            literal_control(
                &bytes,
                "ACB",
                if delete_joint {
                    &[(36, 2000)]
                } else {
                    &[(58, 1500), (36, 2000)]
                },
            );
            neutral_receipt(
                &bytes,
                "A\u{2065}B",
                if delete_joint {
                    &[(36, 20.0)]
                } else {
                    &[(58, 15.0), (36, 20.0)]
                },
                if delete_joint { &[(1, true)] } else { &[] },
            );
        }
        let bytes = font::morx_fixture(M::Skip, with_gpos);
        literal_control(&bytes, "ACB", &[(58, 1500), (100, 700)]);
        neutral_receipt(&bytes, "A\u{2065}B", &[(58, 15.0)], &[(1, false)]);
    }
}

#[test]
fn real_context_variation_mongolian_and_filler_resources_keep_native_receipt_meanings() {
    let bytes = font::context_only();
    literal_control(&bytes, "ACB", &[(58, 1500), (100, 700), (36, 2000)]);
    neutral_receipt(
        &bytes,
        "A\u{2065}B",
        &[(58, 15.0), (36, 20.0)],
        &[(1, false)],
    );
    let bytes = font::repeated();
    literal_control(&bytes, "ACBC", &[(35, 1000), (58, 1500), (36, 2000)]);
    neutral_receipt(
        &bytes,
        "\u{1bca0}A\u{2065}B\u{2065}",
        &[(35, 10.0), (58, 15.0), (36, 20.0)],
        &[(0, false), (4, true)],
    );
    let bytes = font::variation(true, false);
    literal_control(&bytes, "A\u{e0100}", &[(58, 1500)]);
    neutral_receipt(&bytes, "A\u{e0100}", &[(58, 15.0)], &[]);
    let bytes = font::variation(false, true);
    literal_control(&bytes, "A", &[(35, 1000)]);
    neutral_receipt(&bytes, "A\u{e0100}", &[(35, 10.0)], &[(1, false)]);
    for supported in [false, true] {
        let bytes = font::mongolian(supported);
        literal_control(&bytes, "\u{1821}", &[(100, 700)]);
        if supported {
            neutral_receipt(&bytes, "\u{1820}\u{180b}", &[(58, 15.0)], &[]);
        } else {
            neutral_receipt(&bytes, "\u{1820}\u{180b}", &[(35, 10.0)], &[(1, false)]);
        }
    }
    for mapped in [false, true] {
        let bytes = font::fillers(mapped);
        literal_control(&bytes, "F", &[(3, 1800)]);
        for scalar in ['\u{115f}', '\u{1160}', '\u{3164}', '\u{ffa0}'] {
            neutral_receipt(
                &bytes,
                &scalar.to_string(),
                if mapped { &[(3, 18.0)] } else { &[] },
                if mapped { &[] } else { &[(0, false)] },
            );
        }
    }
}

#[test]
fn authored_optional_override_preserves_normal_di_and_refuses_its_real_unqualified_boundary() {
    let bytes = optional_di_font();
    literal_control(&bytes, "ACB", &[(35, 1000), (58, 1500), (36, 2000)]);
    let source = Source::new("A\u{2065}B");
    let mut style = own(0.0);
    style.font = style.font.feature("\"liga\" 1");
    let mut system = system(&bytes);
    let prepared = prepare(
        &mut system,
        input(&source, style, LTR),
        LineOrientation::Horizontal,
    );
    assert!(exclusion_rows(&prepared.inner.shaped).is_empty());
    let ordinary = prepared
        .select(request(&prepared, LTR, SpacingPolicy::None))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(ordinary.feedback().advance, 45.0);
    assert_eq!(
        packets(&ordinary, &bytes)
            .iter()
            .map(|p| (p.1, p.4))
            .collect::<Vec<_>>(),
        [(35, 10.0), (58, 15.0), (36, 20.0)]
    );
    let error = prepared
        .select(request(&prepared, LTR, SpacingPolicy::InterCharacter))
        .unwrap_err();
    let Some(crate::ErrorDetail::UnsupportedSpacing {
        operation,
        reason,
        pieces,
        contexts,
        ..
    }) = error.detail()
    else {
        panic!("{error:?}")
    };
    assert_eq!(*operation, crate::SpacingOperation::InterCharacterBasis);
    assert_eq!(
        *reason,
        crate::SpacingFailureReason::UnqualifiedWritingSystem
    );
    assert_eq!(
        pieces.as_ref(),
        [crate::SpacingPiece::Source {
            origin: 0,
            local: Range::new(1, 2),
            aggregate: Range::new(1, 2)
        }]
    );
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].locale, None);
    assert_eq!(contexts[0].pieces.as_ref(), pieces.as_ref());
    assert_eq!(ordinary.feedback().advance, 45.0);
}

fn conditional_skip_font(tag: [u8; 4]) -> Vec<u8> {
    assert!([*b"ccmp", *b"liga"].contains(&tag));
    let bytes = font::fixture(Some('\u{2065}'), font::Action::Skip);
    resource_premises(&bytes);
    let mut gsub = crate::test_fonts::table(&bytes, *b"GSUB").to_vec();
    let word = |at: usize| u16::from_be_bytes(gsub[at..at + 2].try_into().unwrap());
    // Independently parse DFLT/default LangSys: one enabled feature, no
    // LangSys-required feature. The selected provider plan still requires ccmp;
    // liga is optional. This distinction does not come from a lookup blacklist.
    assert_eq!((word(0), word(2)), (1, 0));
    let scripts = usize::from(word(4));
    assert_eq!(word(scripts), 1);
    assert_eq!(&gsub[scripts + 2..scripts + 6], b"DFLT");
    let script = scripts + usize::from(word(scripts + 6));
    assert_eq!(word(script + 2), 0);
    let language = script + usize::from(word(script));
    assert_eq!(
        (
            word(language),
            word(language + 2),
            word(language + 4),
            word(language + 6)
        ),
        (0, 0xffff, 1, 0)
    );
    let features = usize::from(word(6));
    assert_eq!(word(features), 1);
    assert_eq!(&gsub[features + 2..features + 6], b"ccmp");
    let feature = features + usize::from(word(features + 6));
    assert_eq!(
        (word(feature), word(feature + 2), word(feature + 4)),
        (0, 1, 0)
    );
    let lookups = usize::from(word(8));
    assert_eq!(word(lookups), 1);
    let lookup = lookups + usize::from(word(lookups + 2));
    assert_eq!(
        (word(lookup), word(lookup + 2), word(lookup + 4)),
        (4, 0, 1)
    );
    let sub = lookup + usize::from(word(lookup + 6));
    assert_eq!((word(sub), word(sub + 4)), (1, 1));
    let coverage = sub + usize::from(word(sub + 2));
    assert_eq!(
        (word(coverage), word(coverage + 2), word(coverage + 4)),
        (1, 1, font::A)
    );
    let set = sub + usize::from(word(sub + 6));
    assert_eq!(word(set), 1);
    let ligature = set + usize::from(word(set + 2));
    assert_eq!(
        (word(ligature), word(ligature + 2), word(ligature + 4)),
        (font::R, 2, font::B)
    );
    // D100 is not a consumed component of the real A35+B36->R58 ligature.
    assert_ne!(font::D, font::A);
    assert_ne!(font::D, font::B);
    gsub[features + 2..features + 6].copy_from_slice(&tag);
    crate::test_fonts::replace_tables(&bytes, vec![(*b"GSUB", gsub)])
}

fn native_conditional_skip_control(bytes: &[u8], tag: [u8; 4]) {
    literal_control(bytes, "AB", &[(58, 1500)]);
    let face = surgeist_shaper::FontRef::new(bytes).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut input = surgeist_shaper::UnicodeBuffer::new();
    input.push_str("A\u{2065}B");
    input.set_direction(surgeist_shaper::Direction::LeftToRight);
    input.set_flags(surgeist_shaper::BufferFlags::PRESERVE_DEFAULT_IGNORABLES);
    input.observe_components();
    input.guess_segment_properties();
    let shaped = shaper.shape(input, &[]);
    assert_eq!(
        shaped.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    assert_eq!(
        shaped
            .glyph_infos()
            .iter()
            .zip(shaped.glyph_positions())
            .map(|(info, pos)| (info.glyph_id, pos.x_advance))
            .collect::<Vec<_>>(),
        [(58, 1500), (100, 700)]
    );
    let di = shaped.default_ignorable_receipt().unwrap();
    assert_eq!(di.residual_input_positions(), [1]);
    assert!(di.deleted_input_positions().is_empty());
    let receipt = shaped.component_receipt().unwrap().unwrap();
    assert_eq!(
        receipt.coverage(),
        surgeist_shaper::ComponentCoverage::Complete
    );
    assert_eq!(receipt.components(0), Some([0, 2].as_slice()));
    assert_eq!(receipt.components(1), Some([1].as_slice()));
    let relations = receipt.ligatures().collect::<Vec<_>>();
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].components(), [0, 2]);
    let role = if tag == *b"ccmp" {
        surgeist_shaper::FeatureRole::Required
    } else {
        surgeist_shaper::FeatureRole::Optional
    };
    assert!(
        relations[0]
            .provenance()
            .iter()
            .any(
                |provenance| provenance.tag() == surgeist_shaper::Tag::new(&tag)
                    && provenance.role() == role
            )
    );
    if tag == *b"liga" {
        // A real disabled-lookup control proves the unligated packet and input
        // cluster positions before Text applies its residual exclusion and gap.
        let mut input = surgeist_shaper::UnicodeBuffer::new();
        // push_str assigns UTF-8 byte clusters. Text's adapter instead uses
        // invocation-local scalar ordinals; add binds those explicitly here.
        for (ordinal, scalar) in "A\u{2065}B".chars().enumerate() {
            input.add(scalar, u32::try_from(ordinal).unwrap());
        }
        input.set_direction(surgeist_shaper::Direction::LeftToRight);
        input.set_flags(surgeist_shaper::BufferFlags::PRESERVE_DEFAULT_IGNORABLES);
        input.guess_segment_properties();
        let shaped = shaper.shape(
            input,
            &[surgeist_shaper::Feature::new(
                surgeist_shaper::Tag::new(b"liga"),
                0,
                ..,
            )],
        );
        assert_eq!(
            shaped.shaping_status(),
            surgeist_shaper::ShapingStatus::Complete
        );
        assert_eq!(
            shaped
                .glyph_infos()
                .iter()
                .zip(shaped.glyph_positions())
                .map(|(info, pos)| (
                    info.glyph_id,
                    info.cluster,
                    pos.x_advance,
                    pos.x_offset,
                    pos.y_offset
                ))
                .collect::<Vec<_>>(),
            [
                (35, 0, 1000, 0, 0),
                (100, 1, 700, 0, 0),
                (36, 2, 2000, 0, 0)
            ]
        );
    }
}

fn conditional_tracking_source(tracking: f64) -> Source {
    let mut source = Source::new("A\u{2065}B");
    // Only tracking differs at this real source span; font, size, features,
    // direction and native joining context are identical across the seam.
    source.span(Range::new(1, 2), own(-tracking));
    assert_eq!(source.spans()[0].range(), Range::new(1, 2));
    assert_eq!(source.spans()[0].style().letter_spacing, -tracking);
    source
}

fn conditional_zero_control(system: &mut System, bytes: &[u8]) {
    let source = conditional_tracking_source(0.0);
    let prepared = prepare(
        system,
        input(&source, own(0.0), LTR),
        LineOrientation::Horizontal,
    );
    assert_eq!(exclusion_rows(&prepared.inner.shaped), [(1, false)]);
    reset_provider_calls();
    let selected = prepared
        .select(request(&prepared, LTR, SpacingPolicy::None))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_calls(calls);
    assert_eq!(
        packets(&line, bytes),
        [(Range::new(0, 3), 58, 0.0, 0.0, 15.0)]
    );
    assert_eq!(line.feedback().advance, 15.0);
    assert_eq!(inline_paints(&line), [0.0]);
    assert_eq!(
        prepared
            .unit_views()
            .find(|u| u.source_range() == Range::new(1, 2))
            .unwrap()
            .advance(),
        0.0
    );
    source_copy(&prepared, line, &source);
}

fn conditional_boundary_custody(slot: &crate::SpacingSlot, prepared: &PreparedText) {
    let crate::SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slot.target()
    else {
        panic!("complete actual A-B boundary")
    };
    let origin = prepared.origin_views().next().unwrap().handle();
    for (unit, range) in [
        (&visual_before, Range::new(0, 1)),
        (&visual_after, Range::new(2, 3)),
    ] {
        assert_eq!(
            (unit.letter_spacing_before(), unit.letter_spacing_after()),
            (4.0, 4.0)
        );
        assert_eq!(
            unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
            [crate::SpacingSlotOwner::Source {
                origin: origin.clone(),
                local: range,
                aggregate: range
            }]
        );
    }
}

fn required_conditional_receipt(shaped: &ShapedData) {
    assert!(
        shaped
            .character_invocations
            .iter()
            .flat_map(|invocation| &invocation.constraints)
            .any(|constraint| {
                constraint.required
                    && constraint.kind == crate::SpacingConstraintKind::Ligature
                    && constraint.components == [0, 2]
                    && constraint.provenance.iter().any(|provenance| {
                        provenance.tag() == surgeist_shaper::Tag::new(b"ccmp")
                            && provenance.role() == surgeist_shaper::FeatureRole::Required
                    })
            }),
        "the actual native required A/B relation must survive selected receipt retention"
    );
}

#[test]
fn conditional_residual_demand_retains_required_ccmp_protection_after_neutral_provisional_edges() {
    let bytes = conditional_skip_font(*b"ccmp");
    native_conditional_skip_control(&bytes, *b"ccmp");
    let mut system = system(&bytes);
    conditional_zero_control(&mut system, &bytes);
    // A4/DI-4/B4 gives provisional means0/0. Actual residual exclusion exposes
    // A4/B4, whose material4 gap still cannot split the required native relation.
    let source = conditional_tracking_source(4.0);
    reset_provider_calls();
    let prepared = prepare(
        &mut system,
        input(&source, own(4.0), LTR),
        LineOrientation::Horizontal,
    );
    assert!(provider_calls() > 0);
    assert_eq!(exclusion_rows(&prepared.inner.shaped), [(1, false)]);
    assert_eq!(
        prepared
            .unit_views()
            .find(|u| u.source_range() == Range::new(1, 2))
            .unwrap()
            .advance(),
        0.0
    );
    assert_eq!(
        prepared.unit_views().map(|u| u.advance()).sum::<f64>(),
        15.0
    );
    required_conditional_receipt(&prepared.inner.shaped);
    reset_provider_calls();
    let ordinary = prepared
        .select(request(&prepared, LTR, SpacingPolicy::None))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    let ordinary_line = ordinary
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_calls(calls);
    let raw = [(Range::new(0, 3), 58, 0.0, 0.0, 15.0)];
    assert_eq!(packets(&ordinary_line, &bytes), raw);
    assert_eq!(ordinary_line.feedback().advance, 15.0);
    required_conditional_receipt(&ordinary.inner.basis.shaped);
    reset_provider_calls();
    let selected = prepared
        .select(request(&prepared, LTR, SpacingPolicy::InterCharacter))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    assert_eq!(selected.spacing_slots().len(), 1);
    assert_eq!(exclusion_rows(&selected.inner.basis.shaped), [(1, false)]);
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: amounts(&selected, &[0.0]),
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_calls(calls);
    let row = line.spacing_slots().next().unwrap();
    assert_eq!(
        row.geometry,
        crate::SpacingSlotGeometry::Boundary(crate::SpacingBoundaryGeometry::ProtectedRequired)
    );
    assert_eq!((row.advance, row.allocation), (0.0, 0.0));
    assert!(!row.can_expand());
    conditional_boundary_custody(&row.slot, &prepared);
    assert_eq!(packets(&line, &bytes), raw);
    assert_eq!(line.feedback().advance, 15.0);
    let repeated = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_calls(calls);
    assert_eq!(packets(&repeated, &bytes), raw);
    assert_eq!(repeated.feedback().advance, 15.0);
    source_copy(&prepared, line, &source);
}

#[test]
fn conditional_residual_demand_masks_optional_liga_before_reconciled_boundary_shaping() {
    let bytes = conditional_skip_font(*b"liga");
    native_conditional_skip_control(&bytes, *b"liga");
    let mut system = system(&bytes);
    conditional_zero_control(&mut system, &bytes);
    let source = conditional_tracking_source(4.0);
    reset_provider_calls();
    let prepared = prepare(
        &mut system,
        input(&source, own(4.0), LTR),
        LineOrientation::Horizontal,
    );
    assert!(provider_calls() > 0);
    assert_eq!(exclusion_rows(&prepared.inner.shaped), [(1, false)]);
    assert_eq!(
        prepared
            .unit_views()
            .find(|u| u.source_range() == Range::new(1, 2))
            .unwrap()
            .advance(),
        0.0
    );
    assert_eq!(
        prepared.unit_views().map(|u| u.advance()).sum::<f64>(),
        34.0
    );
    // Raw A1000/B2000 at size10/UPEM1000 plus one mean4 gap. The DI has no
    // glyph packet or independent amount, while its original range is retained.
    let raw = [
        (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
        (Range::new(2, 3), 36, 0.0, 0.0, 20.0),
    ];
    reset_provider_calls();
    let ordinary = prepared
        .select(request(&prepared, LTR, SpacingPolicy::None))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    let ordinary_line = ordinary
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_calls(calls);
    assert_eq!(packets(&ordinary_line, &bytes), raw);
    assert_eq!(ordinary_line.feedback().advance, 34.0);
    assert_eq!(inline_paints(&ordinary_line), [0.0, 14.0]);
    reset_provider_calls();
    let selected = prepared
        .select(request(&prepared, LTR, SpacingPolicy::InterCharacter))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    assert_eq!(selected.spacing_slots().len(), 1);
    assert_eq!(exclusion_rows(&selected.inner.basis.shaped), [(1, false)]);
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_calls(calls);
    let row = line.spacing_slots().next().unwrap();
    assert_eq!(
        row.geometry,
        crate::SpacingSlotGeometry::Boundary(crate::SpacingBoundaryGeometry::Independent)
    );
    assert_eq!(
        (row.authored_letter_spacing, row.advance, row.allocation),
        (4.0, 4.0, 0.0)
    );
    assert!(row.can_expand());
    conditional_boundary_custody(&row.slot, &prepared);
    assert_eq!(packets(&line, &bytes), raw);
    assert_eq!(line.feedback().advance, 34.0);
    let explicit_zero = line.with_spacing(amounts(&selected, &[0.0])).unwrap();
    assert_calls(calls);
    assert_eq!(packets(&explicit_zero, &bytes), raw);
    assert_eq!(explicit_zero.feedback().advance, 34.0);
    assert_eq!(ordinary_line.feedback().advance, 34.0);
    source_copy(&prepared, line, &source);
}

#[path = "selected_basis_tests/conditional_envelopes.rs"]
mod conditional_envelopes;

#[path = "mixed_basis_tests.rs"]
mod mixed_basis_tests;

#[path = "allocated_shift_tests.rs"]
mod allocated_shift_tests;
