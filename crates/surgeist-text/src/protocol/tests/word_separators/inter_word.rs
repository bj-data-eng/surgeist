//! Functional qualification of native InterWord and immutable spacing reuse.
use super::*;
use crate::{
    SpacingDeferredCause as Deferred, SpacingSlotGeometry as Geometry, SpacingSlotOwner,
    SpacingStretchability as Stretch,
};

fn own(word_spacing: f64) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        word_spacing,
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn input(source: &Source, style: Style, terminal: crate::ParagraphTerminalEdge) -> ParagraphInput {
    let len = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style: style,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: terminal,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if len == 0 {
            vec![crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: Affinity::After,
            }]
        } else {
            vec![]
        },
    }
}

fn prepare_input(
    system: &mut System,
    value: ParagraphInput,
    orientation: LineOrientation,
) -> PreparedText {
    let analysis = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            value,
            if orientation == LineOrientation::Vertical {
                crate::AnalysisTypography::Upright {
                    computed_direction: ParagraphBaseDirection::RightToLeft,
                }
            } else {
                crate::AnalysisTypography::HorizontalComposition
            },
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    system
        .prepare(
            &analysis,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap()
}

fn selected(prepared: &PreparedText) -> SelectedLine {
    let mut selection = request(prepared);
    selection.context.orientation = prepared.orientation();
    selection.context.available_inline = None;
    selection.context.spacing = SpacingPolicy::InterWord;
    prepared.select(selection).unwrap()
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

fn feedback(line: &FinalizedLine, intrinsic: f64, authored: f64, allocation: f64, advance: f64) {
    let mut rows = line.spacing_slots();
    let row = rows.next().unwrap();
    assert!(rows.next().is_none());
    assert_eq!(
        row.geometry,
        Geometry::Independent {
            intrinsic_advance: intrinsic
        }
    );
    assert_eq!(row.authored_word_spacing, authored);
    assert_eq!(row.allocation, allocation);
    assert_eq!(row.advance, advance);
    assert!(matches!(
        row.disposition,
        crate::SpacingSlotDisposition::Retained { .. }
    ));
}

fn source_piece(slot: &SpacingSlot, expected: &PreparedOrigin, local: Range, aggregate: Range) {
    let pieces = slot.pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    let SpacingSlotOwner::Source {
        origin,
        local: actual_local,
        aggregate: actual_aggregate,
    } = pieces[0].owner()
    else {
        panic!("expected an actual source-origin piece")
    };
    assert_eq!(&origin, expected);
    assert_eq!(actual_local, local);
    assert_eq!(actual_aggregate, aggregate);
}

#[test]
fn real_interword_adds_authored_and_allocated_halves_and_replaces_an_immutable_natural_basis() {
    let bytes = mapped('\u{1361}', false);
    let source = Source::identified("A\u{1361}A", Id::from_u64(1564020), 3);
    for orientation in FRAMES {
        let mut system = super::system(&bytes, '\u{1361}', false, false);
        let prepared = super::prepare(&mut system, &source, orientation, 4.0);
        let selected = selected(&prepared);
        let slots = selected.spacing_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].selected_line(), selected);
        let crate::SpacingSlotTarget::Separator(unit) = slots[0].target() else {
            panic!("InterWord targets this actual separator unit")
        };
        assert_eq!(
            unit.pieces()
                .map(|piece| {
                    let range = piece.presentation_range();
                    (range.start(), range.end())
                })
                .collect::<Vec<_>>(),
            [(1, 2)]
        );
        source_piece(
            &slots[0],
            &prepared.origin_views().next().unwrap().handle(),
            Range::new(1, 2),
            Range::new(1, 2),
        );
        assert!(
            slots[0]
                .participants()
                .all(|participant| selected.request().participants.contains(participant))
        );
        let natural = selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(natural.feedback().advance, 34.0);
        assert_eq!(paints(&natural), [0.0, 12.0, 24.0]);
        feedback(&natural, 10.0, 4.0, 0.0, 14.0);
        let row = natural.spacing_slots().next().unwrap();
        assert_eq!(row.stretchability, Stretch::Direct);
        assert!(row.can_expand());
        let traversal = row.traversal_order;
        drop(system);
        let expanded = natural.with_spacing(amounts(&selected, &[6.0])).unwrap();
        assert_eq!(expanded.selected_line(), selected);
        assert_eq!(raw(&expanded), raw(&natural));
        assert_eq!(expanded.feedback().advance, 40.0);
        assert_eq!(paints(&expanded), [0.0, 15.0, 30.0]);
        feedback(&expanded, 10.0, 4.0, 6.0, 20.0);
        assert_eq!(
            expanded.spacing_slots().next().unwrap().traversal_order,
            traversal
        );
        for run in expanded.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(
                run.face().catalogue_identity(),
                prepared.font_catalogue_identity()
            );
        }
        super::assert_placement(
            &prepared,
            expanded.clone(),
            &source,
            [0.0, 15.0, 30.0],
            40.0,
            false,
        );
        let replaced = expanded.with_spacing(amounts(&selected, &[2.0])).unwrap();
        assert_eq!(replaced.feedback().advance, 36.0);
        assert_eq!(paints(&replaced), [0.0, 13.0, 26.0]);
        feedback(&replaced, 10.0, 4.0, 2.0, 16.0);
        let fresh = selected
            .finalize(crate::LineAllocations {
                spacing: amounts(&selected, &[2.0]),
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(raw(&replaced), raw(&fresh));
        assert_eq!(paints(&replaced), paints(&fresh));
        let restored = replaced.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(restored.selected_line(), selected);
        assert_eq!(raw(&restored), raw(&natural));
        assert_eq!(restored.feedback().advance, 34.0);
        assert_eq!(paints(&restored), [0.0, 12.0, 24.0]);
        assert_eq!(paints(&expanded), [0.0, 15.0, 30.0]);
    }
}

#[test]
fn intrinsic_zero_is_not_authored_cancellation_and_only_the_latter_can_expand() {
    let source = Source::identified("A A", Id::from_u64(1564021), 1);
    for orientation in FRAMES {
        let native_zero = orientation == LineOrientation::Vertical;
        let bytes = if native_zero {
            vertical_zero_space()
        } else {
            mapped(' ', true)
        };
        let mut system = super::system(&bytes, ' ', true, native_zero);
        let prepared = super::prepare(&mut system, &source, orientation, 4.0);
        let selection = selected(&prepared);
        let natural = selection
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let body = if native_zero { 12.0 } else { 10.0 };
        feedback(&natural, 0.0, 0.0, 0.0, 0.0);
        assert!(!natural.spacing_slots().next().unwrap().can_expand());
        assert_eq!(natural.feedback().advance, body * 2.0);
        let zero = natural.with_spacing(amounts(&selection, &[0.0])).unwrap();
        assert_eq!(paints(&zero), [0.0, body, body]);
        assert_eq!(
            reason(
                &natural
                    .with_spacing(amounts(&selection, &[6.0]))
                    .unwrap_err()
            ),
            &ProtocolError::IneligibleSpacingSlot { slot: 0 }
        );
        assert_eq!(raw(&zero), raw(&natural));
        assert_eq!(natural.feedback().advance, body * 2.0);

        let bytes = mapped(' ', false);
        let mut system = super::system(&bytes, ' ', false, false);
        let prepared = super::prepare(&mut system, &source, orientation, -10.0);
        let selection = selected(&prepared);
        let natural = selection
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        feedback(&natural, 10.0, -10.0, 0.0, 0.0);
        assert!(natural.spacing_slots().next().unwrap().can_expand());
        assert_eq!(natural.feedback().advance, 20.0);
        assert_eq!(paints(&natural), [0.0, 5.0, 10.0]);
        let allocated = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
        feedback(&allocated, 10.0, -10.0, 6.0, 6.0);
        assert_eq!(allocated.feedback().advance, 26.0);
        assert_eq!(paints(&allocated), [0.0, 8.0, 16.0]);
        assert_eq!(raw(&allocated), raw(&natural));
    }
}

#[test]
fn complete_allocation_custody_and_numeric_refusals_leave_the_same_basis_retryable() {
    let bytes = mapped('\u{1361}', false);
    let mut system = super::system(&bytes, '\u{1361}', false, false);
    let source = Source::new("A\u{1361}A");
    let prepared = super::prepare(&mut system, &source, LineOrientation::Horizontal, 4.0);
    let selection = selected(&prepared);
    let other_selection = selected(&prepared);
    let natural = selection
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        reason(
            &natural
                .with_spacing(SpacingAllocations::Complete(vec![]))
                .unwrap_err()
        ),
        &ProtocolError::MissingSpacingSlot { slot: 0 }
    );
    let slot = selection.spacing_slots().next().unwrap();
    assert_eq!(
        reason(
            &natural
                .with_spacing(SpacingAllocations::Complete(vec![
                    SpacingAllocation {
                        slot: slot.clone(),
                        amount: 1.0
                    },
                    SpacingAllocation {
                        slot: slot.clone(),
                        amount: 2.0
                    }
                ]))
                .unwrap_err()
        ),
        &ProtocolError::DuplicateSpacingSlot { slot: 0 }
    );
    assert_eq!(
        reason(
            &natural
                .with_spacing(SpacingAllocations::Complete(vec![SpacingAllocation {
                    slot: other_selection.spacing_slots().next().unwrap(),
                    amount: 1.0
                }]))
                .unwrap_err()
        ),
        &ProtocolError::ForeignSelectedLine
    );
    for (amount, category) in [
        (f64::NAN, crate::TextNumericError::NonFinite),
        (f64::INFINITY, crate::TextNumericError::NonFinite),
        (-1.0, crate::TextNumericError::OutOfRange),
    ] {
        let error = natural
            .with_spacing(amounts(&selection, &[amount]))
            .unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
        assert!(
            matches!(error.detail(), Some(crate::ErrorDetail::InvalidTextNumeric { phase: crate::TextNumericPhase::Finalization, category: actual, .. }) if *actual == category),
            "{error:?}"
        );
    }
    assert_eq!(natural.selected_line(), selection);
    assert_eq!(natural.feedback().advance, 34.0);
    assert_eq!(paints(&natural), [0.0, 12.0, 24.0]);
    let retry = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
    assert_eq!(retry.feedback().advance, 40.0);
    assert_eq!(raw(&retry), raw(&natural));
    assert_eq!(
        paints(&retry.with_spacing(SpacingAllocations::None).unwrap()),
        [0.0, 12.0, 24.0]
    );
}

#[test]
fn natural_shared_provider_geometry_stays_usable_until_a_real_interword_allocation_demands_it() {
    let bytes =
        crate::shaping_test_fonts::range_ligature_ahem(&[35, 58], &[(0x20, 58), (0x41, 35)]);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert!(parsed.gsub().is_ok());
    assert_eq!(parsed.charmap().map(' '), Some(GlyphId::new(58)));
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let source = Source::identified("A A", Id::from_u64(1564022), 1);
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
    ] {
        let mut style = own(0.0);
        style.font = style.font.feature("\"ccmp\" 1");
        let prepared = prepare_input(
            &mut system,
            input(&source, style, crate::ParagraphTerminalEdge::BlockEnd),
            orientation,
        );
        let selection = selected(&prepared);
        let natural = selection
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(
            raw(&natural),
            [
                (Range::new(0, 2), 58, 0.0, 0.0, 10.0),
                (Range::new(2, 3), 35, 0.0, 0.0, 10.0)
            ]
        );
        assert_eq!(natural.feedback().advance, 20.0);
        let row = natural.spacing_slots().next().unwrap();
        assert_eq!(row.geometry, Geometry::SharedProviderGroup);
        assert!(!row.can_expand());
        let inactive = natural.with_spacing(amounts(&selection, &[0.0])).unwrap();
        assert_eq!(raw(&inactive), raw(&natural));
        let error = natural
            .with_spacing(amounts(&selection, &[6.0]))
            .unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
        let Some(crate::ErrorDetail::UnsupportedSpacing {
            stage,
            operation,
            pieces,
            provider_atoms,
            ..
        }) = error.detail()
        else {
            panic!("expected precise provider-group refusal: {error:?}")
        };
        assert_eq!(*stage, crate::SpacingStage::SelectedShaping);
        assert_eq!(*operation, crate::SpacingOperation::InterWordAllocation);
        assert_eq!(
            pieces.as_ref(),
            [crate::SpacingPiece::Source {
                origin: 0,
                local: Range::new(1, 2),
                aggregate: Range::new(1, 2)
            }]
        );
        assert_eq!(provider_atoms.as_ref(), std::slice::from_ref(&(0..2)));
        assert_eq!(
            raw(&natural.with_spacing(SpacingAllocations::None).unwrap()),
            raw(&natural)
        );
    }
}

#[test]
fn cancelled_terminal_candidates_keep_closed_deferred_causes_until_spacing_makes_them_material() {
    let bytes = mapped(' ', false);
    let mut system = super::system(&bytes, ' ', false, false);
    let source = Source::identified(" ", Id::from_u64(1564023), 1);
    for (terminal, cause) in [
        (
            crate::ParagraphTerminalEdge::Unknown,
            Some(Deferred::MissingTerminalBlockEdge),
        ),
        (
            crate::ParagraphTerminalEdge::NonBlockBoundary,
            Some(Deferred::NonBlockParagraphBoundary),
        ),
        (crate::ParagraphTerminalEdge::BlockEnd, None),
    ] {
        let prepared = prepare_input(
            &mut system,
            input(&source, own(-10.0), terminal),
            LineOrientation::Horizontal,
        );
        assert_eq!(prepared.terminal_edge(), terminal);
        let mut request = request(&prepared);
        request.context.spacing = SpacingPolicy::InterWord;
        request.context.available_inline = Some(15.0);
        let selection = prepared.select(request).unwrap();
        let natural = selection
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        feedback(&natural, 10.0, -10.0, 0.0, 0.0);
        assert_eq!(natural.feedback().advance, 0.0);
        let zero = natural.with_spacing(amounts(&selection, &[0.0])).unwrap();
        assert_eq!(zero.feedback().advance, 0.0);
        if let Some(cause) = cause {
            let row = natural.spacing_slots().next().unwrap();
            assert_eq!(row.stretchability, Stretch::Deferred(cause));
            assert!(!row.can_expand());
            let error = natural
                .with_spacing(amounts(&selection, &[6.0]))
                .unwrap_err();
            assert_eq!(
                reason(&error),
                &match cause {
                    Deferred::MissingTerminalBlockEdge => ProtocolError::MissingPhaseTwoContext(
                        crate::PhaseTwoContext::TerminalBlockEdge
                    ),
                    Deferred::NonBlockParagraphBoundary => ProtocolError::UnsupportedContext {
                        field: ProtocolContextField::CssParagraphBoundary
                    },
                }
            );
        } else {
            assert_eq!(
                natural.spacing_slots().next().unwrap().stretchability,
                Stretch::Direct
            );
            let allocated = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
            assert_eq!(allocated.feedback().advance, 6.0);
            assert_eq!(allocated.measurements().alignment().advance(), 6.0);
            feedback(&allocated, 10.0, -10.0, 6.0, 6.0);
        }
        assert_eq!(natural.feedback().advance, 0.0);
        assert_eq!(zero.selected_line(), selection);
    }
}

fn contextual(source: &Source, style: Style, cuts: &[usize]) -> AnalyzedSource {
    use crate::SourceProcessingEvent as E;
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    for origin in 0..cuts.len() - 1 {
        if origin != 0 {
            events.push(E::Connection(origin - 1));
        }
        events.push(E::Origin {
            origin,
            affinity: Affinity::After,
        });
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let wrap = style.wrap;
    let processed = std::sync::Arc::new(
        crate::process_contextual_source(crate::ContextualSourceInput {
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            envelope: crate::SourceProcessingEnvelope::Complete,
            origins: cuts
                .windows(2)
                .map(|pair| crate::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(pair[0], pair[1]),
                    aggregate: Range::new(pair[0], pair[1]),
                    kind: crate::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..cuts.len() - 2)
                .map(|before_origin| crate::SourceProcessingConnection {
                    before_origin,
                    after_origin: before_origin + 1,
                    kind: crate::ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..cuts.len() - 2)
                .map(|connection| crate::ParagraphBoundaryPolicy {
                    definition: crate::ParagraphBoundaryDefinition::Connection(connection),
                    preceding_wrap: wrap,
                    following_wrap: wrap,
                    ancestor_wrap: wrap,
                })
                .collect(),
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![crate::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: crate::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![crate::SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        wrap,
    )
}

fn register(bytes: Vec<u8>) -> System {
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}

#[test]
fn a_complete_marked_candidate_keeps_both_real_source_origin_pieces_and_moves_every_attached_glyph()
{
    let cmap = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[(0x41, 35), (0x301, 100), (0x1361, 35)],
    );
    let bytes = sfnt::replace_tables(
        &crate::shaping_test_fonts::mark_ahem(),
        vec![(*b"cmap", sfnt::table(&cmap, *b"cmap").to_vec())],
    );
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert!(parsed.gpos().is_ok());
    assert_eq!(parsed.charmap().map('\u{1361}'), Some(GlyphId::new(35)));
    assert_eq!(parsed.charmap().map('\u{301}'), Some(GlyphId::new(100)));
    let mut system = register(bytes);
    let source = Source::identified("A\u{1361}\u{301}A", Id::from_u64(1564024), 1);
    let analysis = contextual(&source, own(4.0), &[0, 2, 4]);
    assert_eq!(
        analysis.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)]
    );
    let prepared = system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap();
    let origins = prepared
        .origin_views()
        .map(|view| {
            assert!(view.capture_position().is_some());
            view.handle()
        })
        .collect::<Vec<_>>();
    assert_eq!(origins.len(), 2);
    assert_ne!(origins[0], origins[1]);
    let selection = selected(&prepared);
    let slots = selection.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let crate::SpacingSlotTarget::Separator(unit) = slots[0].target() else {
        panic!("split origins retain one complete InterWord separator")
    };
    assert_eq!(
        unit.pieces()
            .map(|piece| {
                let range = piece.presentation_range();
                (range.start(), range.end())
            })
            .collect::<Vec<_>>(),
        [(1, 2), (2, 3)]
    );
    let pieces = slots[0].pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 2);
    for (index, expected) in [Range::new(1, 2), Range::new(2, 3)].into_iter().enumerate() {
        let SpacingSlotOwner::Source {
            origin,
            local,
            aggregate,
        } = pieces[index].owner()
        else {
            panic!("expected actual split-origin piece")
        };
        assert_eq!(origin, origins[index]);
        assert_eq!(local, expected);
        assert_eq!(aggregate, expected);
        assert_eq!(
            (
                pieces[index].presentation_range().start(),
                pieces[index].presentation_range().end()
            ),
            (expected.start, expected.end)
        );
    }
    let natural = selection
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        raw(&natural),
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
            (Range::new(1, 3), 35, 0.0, 0.0, 10.0),
            (Range::new(1, 3), 100, -7.0, -4.0, 0.0),
            (Range::new(3, 4), 35, 0.0, 0.0, 10.0)
        ]
    );
    assert_eq!(paints(&natural), [0.0, 12.0, 22.0, 24.0]);
    let allocated = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
    feedback(&allocated, 10.0, 4.0, 6.0, 20.0);
    assert_eq!(allocated.feedback().advance, 40.0);
    assert_eq!(paints(&allocated), [0.0, 15.0, 25.0, 30.0]);
    assert_eq!(raw(&allocated), raw(&natural));
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn selected_supplementary_indication_slots_keep_alternative_local_custody_and_original_shy_span() {
    let bytes = mapped('\u{10100}', false);
    let mut system = super::system(&bytes, '\u{10100}', false, false);
    let source = Source::identified("AAAAA\u{ad}AAAAA", Id::from_u64(1564025), 2);
    let style = Style {
        hyphens: crate::Hyphens::Manual,
        hyphenate_character: crate::HyphenateCharacter::String(crate::Text::from("\u{10100}")),
        ..own(4.0)
    };
    let prepared = prepare_input(
        &mut system,
        input(&source, style, crate::ParagraphTerminalEdge::BlockEnd),
        LineOrientation::Horizontal,
    );
    let cut = prepared
        .hyphenation_cut_views()
        .find(|cut| cut.boundary().view().source_before().get() == 6)
        .unwrap();
    assert_eq!(
        cut.counts(),
        Some(crate::HyphenationCounts {
            total: 10,
            before: 5,
            after: 5
        })
    );
    let offer = cut.opportunity().unwrap();
    let alternative = offer.view().alternatives().next().unwrap().handle();
    assert_eq!(alternative.view().text().code_units(), [0xd800, 0xdd00]);
    assert_eq!(alternative.view().source_range(), Some(Range::new(5, 6)));
    let mut request = request(&prepared);
    request.context.spacing = SpacingPolicy::InterWord;
    request.context.available_inline = None;
    request.context.end = LineEnd::Soft;
    request.end = offer.view().boundary();
    request.participants = prepared
        .participants()
        .take(request.end.view().participant_offset())
        .collect();
    request.termination = crate::PreparedTermination::Ordinary(offer);
    request.alternative = Some(alternative.clone());
    let selection = prepared.select(request).unwrap();
    let slots = selection.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let pieces = slots[0].pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    let SpacingSlotOwner::Alternative {
        alternative: actual,
        local,
    } = pieces[0].owner()
    else {
        panic!("an indication cannot donate an original source range")
    };
    assert_eq!(actual, alternative);
    assert_eq!(local, Range::new(0, 2));
    let crate::SpacingSlotTarget::Separator(unit) = slots[0].target() else {
        panic!("the indication retains its complete supplementary separator")
    };
    assert_eq!(unit.pieces().len(), 1);
    assert_eq!(unit.pieces().collect::<Vec<_>>(), pieces);
    assert_eq!(
        pieces[0].presentation_range().end() - pieces[0].presentation_range().start(),
        2
    );
    let natural = selection
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(natural.feedback().advance, 64.0);
    assert_eq!(paints(&natural).last().copied(), Some(52.0));
    assert_eq!(raw(&natural).last().unwrap().0, Range::new(6, 6));
    let allocated = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
    assert_eq!(allocated.feedback().advance, 70.0);
    assert_eq!(paints(&allocated).last().copied(), Some(55.0));
    feedback(&allocated, 10.0, 4.0, 6.0, 20.0);
    assert_eq!(raw(&allocated), raw(&natural));
    assert_eq!(
        allocated.selected_line().request().alternative,
        Some(alternative)
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn a_real_expanded_wbr_keeps_a_control_piece_without_fabricating_an_authored_space() {
    let bytes = mapped(' ', false);
    let mut system = super::system(&bytes, ' ', false, false);
    let source = Source::identified("AA", Id::from_u64(1564026), 1);
    let mut value = input(&source, own(0.0), crate::ParagraphTerminalEdge::BlockEnd);
    value.controls = vec![crate::ParagraphControl {
        at: 1,
        kind: crate::ParagraphControlKind::Wbr {
            style: Box::new(Style {
                word_space_transform: crate::WordSpaceTransform::Space,
                ..own(4.0)
            }),
        },
    }];
    value.anchor_order = vec![crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::Control(0),
        affinity: Affinity::After,
    }];
    let prepared = prepare_input(&mut system, value, LineOrientation::Horizontal);
    let control = prepared.control_views().next().unwrap().handle();
    let selection = selected(&prepared);
    let slots = selection.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let pieces = slots[0].pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].owner(), SpacingSlotOwner::Control(control));
    assert_eq!(
        (
            pieces[0].presentation_range().start(),
            pieces[0].presentation_range().end()
        ),
        (1, 2)
    );
    let natural = selection
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(natural.feedback().advance, 34.0);
    assert_eq!(
        raw(&natural).iter().map(|row| row.0).collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 1), Range::new(1, 2)]
    );
    let allocated = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
    assert_eq!(allocated.feedback().advance, 40.0);
    assert_eq!(paints(&allocated), [0.0, 15.0, 30.0]);
    assert_eq!(raw(&allocated), raw(&natural));
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn a_published_phrase_separator_keeps_its_actual_virtual_owner_through_interword_reallocation() {
    // Retained BudouX0.9.3 README row: 今日は▁天気です。. The authored
    // contextual operation supplies the actual domain; this is not a manually
    // inserted SPACE or a new linguistic inference oracle.
    let text = "今日は天気です。";
    let mut mappings = text
        .chars()
        .map(|ch| (u32::from(ch), 35))
        .collect::<Vec<_>>();
    mappings.push((0x20, 58));
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(&[82, 82], &mappings);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    for ch in text.chars() {
        assert_eq!(parsed.charmap().map(ch), Some(GlyphId::new(35)));
    }
    assert_eq!(parsed.charmap().map(' '), Some(GlyphId::new(58)));
    let mut system = register(bytes);
    let source = Source::identified(text, Id::from_u64(1564027), 1);
    assert_eq!(source.text().len(), 8);
    let analysis = contextual(
        &source,
        Style {
            locale: Some("ja".into()),
            word_space_transform: crate::WordSpaceTransform::SpaceAutoPhrase,
            ..own(4.0)
        },
        &[0, 8],
    );
    let prepared = system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap();
    assert!(prepared.captured_paragraph().is_some());
    let virtuals = prepared.virtual_separators().collect::<Vec<_>>();
    assert_eq!(virtuals.len(), 1);
    assert_eq!(virtuals[0].view().source_position().get(), 3);
    let selection = selected(&prepared);
    let slots = selection.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let pieces = slots[0].pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(
        pieces[0].owner(),
        SpacingSlotOwner::VirtualSeparator(virtuals[0].clone())
    );
    assert_eq!(
        (
            pieces[0].presentation_range().start(),
            pieces[0].presentation_range().end()
        ),
        (3, 4)
    );
    let natural = selection
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(natural.feedback().advance, 94.0);
    assert_eq!(
        paints(&natural),
        [0.0, 10.0, 20.0, 32.0, 44.0, 54.0, 64.0, 74.0, 84.0]
    );
    let allocated = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
    assert_eq!(allocated.feedback().advance, 100.0);
    assert_eq!(
        paints(&allocated),
        [0.0, 10.0, 20.0, 35.0, 50.0, 60.0, 70.0, 80.0, 90.0]
    );
    assert_eq!(raw(&allocated), raw(&natural));
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn preserved_terminal_space_excludes_expansion_from_alignment_in_both_paragraph_directions() {
    let bytes = mapped(' ', false);
    let source = Source::identified("A ", Id::from_u64(1564028), 1);
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
    ] {
        for direction in [
            ParagraphBaseDirection::LeftToRight,
            ParagraphBaseDirection::RightToLeft,
        ] {
            let mut system = super::system(&bytes, ' ', false, false);
            let mut captured = input(&source, own(0.0), crate::ParagraphTerminalEdge::BlockEnd);
            captured.base_direction = direction;
            let prepared = prepare_input(&mut system, captured, orientation);
            assert_eq!(
                prepared.terminal_edge(),
                crate::ParagraphTerminalEdge::BlockEnd
            );
            // A is strong LTR: an RTL paragraph resolves A to level2 and its
            // trailing SPACE to the actual paragraph level1, rather than
            // pretending that the Latin glyph itself is strong RTL.
            let levels = if direction == ParagraphBaseDirection::LeftToRight {
                [0, 0]
            } else {
                [2, 1]
            };
            assert_eq!(
                prepared
                    .unit_views()
                    .map(|unit| (unit.source_range(), unit.level().get()))
                    .collect::<Vec<_>>(),
                [(Range::new(0, 1), levels[0]), (Range::new(1, 2), levels[1])]
            );
            let mut request = request(&prepared);
            request.context.orientation = orientation;
            request.context.base_direction = direction;
            request.context.line_box_direction = match direction {
                ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
                ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
            };
            request.context.available_inline = Some(15.0);
            request.context.alignment = crate::Alignment::Center;
            request.context.spacing = SpacingPolicy::InterWord;
            let selection = prepared.select(request).unwrap();
            let slots = selection.spacing_slots().collect::<Vec<_>>();
            assert_eq!(slots.len(), 1);
            source_piece(
                &slots[0],
                &prepared.origin_views().next().unwrap().handle(),
                Range::new(1, 2),
                Range::new(1, 2),
            );
            let natural = selection
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            feedback(&natural, 10.0, 0.0, 0.0, 10.0);
            let mut glyphs = raw(&natural);
            glyphs.sort_by_key(|glyph| glyph.0.start);
            assert_eq!(
                glyphs,
                [
                    (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
                    (Range::new(1, 2), 58, 0.0, 0.0, 10.0)
                ]
            );
            let allocated = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
            feedback(&allocated, 10.0, 0.0, 6.0, 16.0);
            // Existing Phase II conditional hanging keeps the five units
            // inside the15px line. Expansion lengthens only its outside part:
            // LTR clips20/26 at15; RTL clips0..10/16 at20/26−15.
            for (line, advance, excluded) in [(&natural, 20.0, 5.0), (&allocated, 26.0, 11.0)] {
                assert_eq!(line.feedback().advance, advance);
                assert_eq!(line.measurements().alignment().advance(), 15.0);
                let edge = line
                    .edge_units()
                    .find(|edge| edge.source_pieces().collect::<Vec<_>>() == [Range::new(1, 2)])
                    .unwrap();
                assert_eq!(
                    edge.disposition(),
                    crate::LineEdgeDisposition::Conditional {
                        side: if direction == ParagraphBaseDirection::LeftToRight {
                            crate::LineEdgeSide::End
                        } else {
                            crate::LineEdgeSide::Start
                        },
                        eligible: true
                    }
                );
                assert_eq!(edge.advance(), advance - 10.0);
                assert_eq!(edge.excluded_contribution(), excluded);
            }
            assert_eq!(raw(&allocated), raw(&natural));
            assert_eq!(allocated.selected_line(), selection);
            assert_eq!(prepared.analyzed_source().source(), &source);
            // Response describes the independently established alignment
            // derivative, even though the nonzero SPACE remains in flow.
            for line in [&natural, &allocated] {
                let row = line.spacing_slots().next().unwrap();
                assert_eq!(row.stretchability, Stretch::Excluded);
                assert!(!row.can_expand());
            }
        }
    }
}

#[test]
fn allocation_only_half_underflow_reports_finalization_and_preserves_the_retained_basis() {
    let bytes = mapped('\u{1361}', false);
    let source = Source::identified("A\u{1361}A", Id::from_u64(1564029), 1);
    let mut system = super::system(&bytes, '\u{1361}', false, false);
    let prepared = super::prepare(&mut system, &source, LineOrientation::Horizontal, 0.0);
    let selection = selected(&prepared);
    let natural = selection
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    feedback(&natural, 10.0, 0.0, 0.0, 10.0);
    assert_eq!(natural.feedback().advance, 30.0);
    assert_eq!(paints(&natural), [0.0, 10.0, 20.0]);
    let retained_raw = raw(&natural);
    assert_eq!(
        retained_raw,
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 58, 0.0, 0.0, 10.0),
            (Range::new(2, 3), 35, 0.0, 0.0, 10.0)
        ]
    );
    drop(system);
    // Binary64's smallest positive subnormal divided by two rounds to zero.
    // Authored spacing is zero: only this allocation demands the lost half.
    let allocation = f64::from_bits(1);
    assert!(allocation.is_finite() && allocation > 0.0);
    assert_eq!(allocation * 0.5, 0.0);
    let error = natural
        .with_spacing(amounts(&selection, &[allocation]))
        .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
    assert!(
        matches!(
            error.detail(),
            Some(crate::ErrorDetail::InvalidTextNumeric {
                field: "word spacing half",
                phase: crate::TextNumericPhase::Finalization,
                category: crate::TextNumericError::NonZeroLost,
                value,
            }) if *value == allocation
        ),
        "{error:?}"
    );
    assert_eq!(natural.selected_line(), selection);
    assert_eq!(natural.feedback().advance, 30.0);
    assert_eq!(paints(&natural), [0.0, 10.0, 20.0]);
    assert_eq!(raw(&natural), retained_raw);
    assert_eq!(prepared.analyzed_source().source(), &source);
    let retry = natural.with_spacing(amounts(&selection, &[6.0])).unwrap();
    feedback(&retry, 10.0, 0.0, 6.0, 16.0);
    assert_eq!(retry.selected_line(), selection);
    assert_eq!(retry.feedback().advance, 36.0);
    assert_eq!(paints(&retry), [0.0, 13.0, 26.0]);
    assert_eq!(raw(&retry), retained_raw);
    for run in retry.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(
            run.face().catalogue_identity(),
            prepared.font_catalogue_identity()
        );
    }
    let restored = retry.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(restored.selected_line(), selection);
    assert_eq!(restored.feedback().advance, 30.0);
    assert_eq!(paints(&restored), [0.0, 10.0, 20.0]);
    assert_eq!(raw(&restored), retained_raw);
    assert_eq!(prepared.analyzed_source().source(), &source);
}
