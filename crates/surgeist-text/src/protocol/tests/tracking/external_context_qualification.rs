//! Functional qualification of Text-owned external joining context constraints.
use super::super::fixtures::allocations;
use super::*;
use crate::{SpacingBoundaryGeometry, SpacingSlotGeometry, SpacingSlotOwner, SpacingSlotTarget};

fn system(bytes: &[u8]) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system
}

fn selected(prepared: &PreparedText) -> SelectedLine {
    let mut value = request(prepared);
    value.context.base_direction = ParagraphBaseDirection::RightToLeft;
    value.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
    value.context.available_inline = None;
    value.context.spacing = SpacingPolicy::InterCharacter;
    prepared.select(value).unwrap()
}

fn glyphs(line: &FinalizedLine, bytes: &[u8]) -> Vec<RawGlyph> {
    let mut rows = raw(line, bytes);
    rows.sort_by_key(|glyph| glyph.0.start);
    rows
}

fn paints(line: &FinalizedLine) -> Vec<(Range, f64)> {
    let mut rows = line
        .run_views()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .map(move |glyph| (glyph.range(), origin.x + glyph.pen() + f64::from(glyph.x())))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.0.start);
    rows
}

fn disconnected(source: &Source, tracking: f64) -> AnalyzedSource {
    assert_eq!(source.text().len(), 2);
    let value = ParagraphInput {
        source: source.clone(),
        base_style: own(tracking),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::RightToLeft,
        origins: [Range::new(0, 1), Range::new(1, 2)]
            .into_iter()
            .map(|range| crate::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: crate::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: vec![crate::ParagraphConnection {
            at: 1,
            kind: crate::ParagraphConnectionKind::Disconnected,
        }],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let result = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(result.source(), source);
    assert_eq!(
        result.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    result
}

fn independent(line: &FinalizedLine, authored: f64, allocation: f64) {
    let rows = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Independent)
    );
    assert_eq!(rows[0].authored_letter_spacing, authored);
    assert_eq!(rows[0].allocation, allocation);
    assert_eq!(rows[0].advance, authored + allocation);
    assert!(rows[0].can_expand());
}

#[test]
fn real_external_context_retains_a_complete_constrained_zero_basis_and_precise_material_refusal() {
    let bytes = fixture();
    qualify(&bytes);
    let source = Source::identified("\u{628}\u{712}", Id::from_u64(1566053), 1);
    let mut system = system(&bytes);
    let prepared = system
        .prepare(&analyzed(&source, 0.0), PreparationOptions::default())
        .unwrap();
    let control = execute(&prepared).unwrap();
    let expected = [
        (Range::new(0, 1), 100, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 82, 0.0, 0.0, 10.0),
    ];
    assert_eq!(control.run_views().len(), 2);
    assert_eq!(glyphs(&control, &bytes), expected);
    assert_eq!(control.feedback().advance, 20.0);
    let selected = selected(&prepared);
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slots[0].target()
    else {
        panic!("complete actual visual endpoints required")
    };
    let origin = prepared.origin_views().next().unwrap().handle();
    for (unit, index) in [(visual_before, 1), (visual_after, 0)] {
        let extent = Range::new(index, index + 1);
        assert_eq!(
            unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
            [SpacingSlotOwner::Source {
                origin: origin.clone(),
                local: extent,
                aggregate: extent
            }]
        );
        assert_eq!(
            unit.participants().cloned().collect::<Vec<_>>(),
            [PreparedParticipant::Text(
                prepared.unit_views().nth(index).unwrap().handle()
            )]
        );
    }
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(glyphs(&natural, &bytes), expected);
    assert_eq!(natural.feedback().advance, 20.0);
    assert_eq!(natural.measurements().alignment().advance(), 20.0);
    let row = natural.spacing_slots().next().unwrap();
    assert_eq!(
        row.geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Constrained)
    );
    assert_eq!(
        (row.authored_letter_spacing, row.allocation, row.advance),
        (0.0, 0.0, 0.0)
    );
    assert!(!row.can_expand());
    let zero = natural
        .with_spacing(allocations(&selected, &[0.0]))
        .unwrap();
    assert_eq!(glyphs(&zero, &bytes), expected);
    assert_eq!(zero.feedback().advance, 20.0);
    let error = natural
        .with_spacing(allocations(&selected, &[2.0]))
        .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
    let Some(crate::ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        pieces,
        provider_atoms,
        reason,
        contexts,
        constraints,
    }) = error.detail()
    else {
        panic!("precise actual external-context capability required: {error:?}")
    };
    assert_eq!(*stage, crate::SpacingStage::SelectedShaping);
    assert_eq!(
        *operation,
        crate::SpacingOperation::InterCharacterAllocation
    );
    assert_eq!(
        *reason,
        crate::SpacingFailureReason::UnqualifiedProviderPath
    );
    let expected_pieces = [
        crate::SpacingPiece::Source {
            origin: 0,
            local: Range::new(1, 2),
            aggregate: Range::new(1, 2),
        },
        crate::SpacingPiece::Source {
            origin: 0,
            local: Range::new(0, 1),
            aggregate: Range::new(0, 1),
        },
    ];
    assert_eq!(pieces.as_ref(), expected_pieces);
    let mut atoms = provider_atoms
        .iter()
        .flat_map(|range| range.clone())
        .collect::<Vec<_>>();
    atoms.sort_unstable();
    atoms.dedup();
    assert_eq!(atoms, [0, 1]);
    assert_eq!(contexts.len(), 2);
    for (context, (piece, script)) in contexts
        .iter()
        .zip(expected_pieces.into_iter().zip([*b"Syrc", *b"Arab"]))
    {
        assert_eq!(context.pieces.as_ref(), [piece]);
        assert!(context.locale.is_none());
        assert_eq!(
            context
                .candidates
                .iter()
                .map(|script| script.iso15924())
                .collect::<Vec<_>>(),
            [script]
        );
        assert_eq!(
            context
                .resolved_scripts
                .iter()
                .map(|script| script.iso15924())
                .collect::<Vec<_>>(),
            [script]
        );
        assert!(context.writing_systems.is_empty());
        assert_eq!(
            (context.letter_spacing_before, context.letter_spacing_after),
            (0.0, 0.0)
        );
    }
    assert_eq!(constraints.len(), 2);
    let mut scopes = Vec::new();
    for constraint in constraints.iter() {
        let crate::SpacingConstraintKind::ExternalJoiningContext { body, context } =
            &constraint.kind
        else {
            panic!(
                "a Text external scope must not be fabricated as a Shaper path or applied attachment"
            )
        };
        assert!(constraint.provenance.is_empty());
        let mut components = constraint
            .components
            .iter()
            .flat_map(|range| range.clone())
            .collect::<Vec<_>>();
        components.sort_unstable();
        components.dedup();
        assert_eq!(components, [0, 1]);
        scopes.push((constraint.run, body.clone(), context.clone()));
    }
    scopes.sort_by_key(|scope| scope.0);
    assert_eq!(scopes, [(0, 0..1, 0..2), (1, 1..2, 0..2)]);
    assert_eq!(natural.selected_line(), selected);
    assert_eq!(glyphs(&natural, &bytes), expected);
    assert_eq!(natural.feedback().advance, 20.0);
    assert_eq!(
        natural
            .with_spacing(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        20.0
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn same_script_joining_is_protected_without_inventing_an_external_uncertainty() {
    let bytes = fixture();
    qualify(&bytes);
    let mut system = system(&bytes);
    for text in ["\u{628}\u{628}", "\u{712}\u{712}"] {
        let source = Source::new(text);
        let control_prepared = system
            .prepare(&analyzed(&source, 0.0), PreparationOptions::default())
            .unwrap();
        let control = execute(&control_prepared).unwrap();
        let expected = [
            (Range::new(0, 1), 100, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 82, 0.0, 0.0, 10.0),
        ];
        assert_eq!(control.run_views().len(), 1);
        assert_eq!(glyphs(&control, &bytes), expected);
        assert_eq!(control.feedback().advance, 20.0);
        let prepared = system
            .prepare(&analyzed(&source, 4.0), PreparationOptions::default())
            .unwrap();
        let authored = execute(&prepared).unwrap();
        assert_eq!(glyphs(&authored, &bytes), expected);
        assert_eq!(authored.feedback().advance, 20.0);
        let selected = selected(&prepared);
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(glyphs(&natural, &bytes), expected);
        assert_eq!(
            paints(&natural),
            [(Range::new(0, 1), 10.0), (Range::new(1, 2), 0.0)]
        );
        let rows = natural.spacing_slots().collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].geometry,
            SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::ProtectedCursive)
        );
        assert_eq!(rows[0].authored_letter_spacing, 0.0);
        assert!(!rows[0].can_expand());
        assert_eq!(
            reason(
                &natural
                    .with_spacing(allocations(&selected, &[2.0]))
                    .unwrap_err()
            ),
            &ProtocolError::IneligibleSpacingSlot { slot: 0 }
        );
        assert_eq!(natural.feedback().advance, 20.0);
        assert_eq!(prepared.analyzed_source().source(), &source);
    }
}

#[test]
fn a_real_nonjoining_endpoint_admits_disjoint_script_tracking_and_character_allocation() {
    let base = fixture();
    let map = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[
            (0x41, 35),
            (0x42, 36),
            (0x621, 36),
            (0x628, 35),
            (0x712, 35),
        ],
    );
    let bytes = sfnt::replace_tables(
        &base,
        vec![(*b"cmap", sfnt::table(&map, *b"cmap").to_vec())],
    );
    qualify(&bytes);
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get32(0x621),
        GeneralCategory::OtherLetter
    );
    assert_eq!(
        CodePointMapData::<Script>::new().get32(0x621),
        Script::Arabic
    );
    assert_eq!(
        ScriptWithExtensions::new()
            .get_script_extensions_val32(0x621)
            .iter()
            .collect::<Vec<_>>(),
        [Script::Arabic]
    );
    assert_eq!(
        CodePointMapData::<JoiningType>::new().get32(0x621),
        JoiningType::NonJoining
    );
    assert_eq!(
        skrifa::FontRef::new(&bytes)
            .unwrap()
            .charmap()
            .map('\u{621}'),
        Some(GlyphId::new(36))
    );
    let source = Source::new("\u{621}\u{712}");
    let mut system = system(&bytes);
    let control_prepared = system
        .prepare(&analyzed(&source, 0.0), PreparationOptions::default())
        .unwrap();
    let control = execute(&control_prepared).unwrap();
    let expected = [
        (Range::new(0, 1), 36, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 35, 0.0, 0.0, 10.0),
    ];
    assert_eq!(control.run_views().len(), 2);
    assert_eq!(glyphs(&control, &bytes), expected);
    assert_eq!(control.feedback().advance, 20.0);
    let prepared = system
        .prepare(&analyzed(&source, 4.0), PreparationOptions::default())
        .unwrap();
    let authored = execute(&prepared).unwrap();
    assert_eq!(glyphs(&authored, &bytes), expected);
    assert_eq!(authored.feedback().advance, 24.0);
    assert_eq!(
        paints(&authored),
        [(Range::new(0, 1), 14.0), (Range::new(1, 2), 0.0)]
    );
    let selected = selected(&prepared);
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    independent(&natural, 4.0, 0.0);
    let allocated = natural
        .with_spacing(allocations(&selected, &[2.0]))
        .unwrap();
    independent(&allocated, 4.0, 2.0);
    assert_eq!(glyphs(&allocated, &bytes), expected);
    assert_eq!(allocated.feedback().advance, 26.0);
    assert_eq!(
        paints(&allocated),
        [(Range::new(0, 1), 16.0), (Range::new(1, 2), 0.0)]
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn a_genuine_disconnected_origin_connection_bounds_both_joining_contexts_and_admits_spacing() {
    let bytes = fixture();
    qualify(&bytes);
    let source = Source::identified("\u{628}\u{712}", Id::from_u64(1566051), 1);
    let mut system = system(&bytes);
    let control_prepared = system
        .prepare(&disconnected(&source, 0.0), PreparationOptions::default())
        .unwrap();
    let control = execute(&control_prepared).unwrap();
    let expected = [
        (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 35, 0.0, 0.0, 10.0),
    ];
    assert_eq!(control.run_views().len(), 2);
    assert_eq!(glyphs(&control, &bytes), expected);
    assert_eq!(control.feedback().advance, 20.0);
    let prepared = system
        .prepare(&disconnected(&source, 4.0), PreparationOptions::default())
        .unwrap();
    let origins = prepared
        .origin_views()
        .map(|view| {
            assert_eq!(view.source(), &source);
            view.handle()
        })
        .collect::<Vec<_>>();
    assert_eq!(origins.len(), 2);
    assert_ne!(origins[0], origins[1]);
    let authored = execute(&prepared).unwrap();
    assert_eq!(glyphs(&authored, &bytes), expected);
    assert_eq!(authored.feedback().advance, 24.0);
    let selected = selected(&prepared);
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slots[0].target()
    else {
        panic!("actual endpoint target required")
    };
    for (unit, origin, extent) in [
        (visual_before, origins[1].clone(), Range::new(1, 2)),
        (visual_after, origins[0].clone(), Range::new(0, 1)),
    ] {
        assert_eq!(
            unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
            [SpacingSlotOwner::Source {
                origin,
                local: extent,
                aggregate: extent
            }]
        );
    }
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    independent(&natural, 4.0, 0.0);
    let allocated = natural
        .with_spacing(allocations(&selected, &[2.0]))
        .unwrap();
    assert_eq!(allocated.feedback().advance, 26.0);
    assert_eq!(glyphs(&allocated, &bytes), expected);
    assert_eq!(
        paints(&allocated),
        [(Range::new(0, 1), 16.0), (Range::new(1, 2), 0.0)]
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn independent_outside_allocation_translates_the_whole_external_context_pair() {
    let bytes = fixture();
    qualify(&bytes);
    assert_eq!(CodePointMapData::<Script>::new().get32(0x41), Script::Latin);
    assert_eq!(
        CodePointMapData::<JoiningType>::new().get32(0x41),
        JoiningType::NonJoining
    );
    let source = Source::identified("\u{628}\u{712}A", Id::from_u64(1566052), 1);
    let mut system = system(&bytes);
    let prepared = system
        .prepare(&analyzed(&source, 0.0), PreparationOptions::default())
        .unwrap();
    let control = execute(&prepared).unwrap();
    let expected = [
        (Range::new(0, 1), 100, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 82, 0.0, 0.0, 10.0),
        (Range::new(2, 3), 35, 0.0, 0.0, 10.0),
    ];
    assert_eq!(control.run_views().len(), 3);
    assert_eq!(glyphs(&control, &bytes), expected);
    assert_eq!(control.feedback().advance, 30.0);
    assert_eq!(
        paints(&control),
        [
            (Range::new(0, 1), 20.0),
            (Range::new(1, 2), 10.0),
            (Range::new(2, 3), 0.0)
        ]
    );
    let selected = selected(&prepared);
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(glyphs(&natural, &bytes), expected);
    assert_eq!(natural.feedback().advance, 30.0);
    let rows = natural.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Independent)
    );
    assert!(rows[0].can_expand());
    assert_eq!(
        rows[1].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Constrained)
    );
    assert!(!rows[1].can_expand());
    let allocated = natural
        .with_spacing(allocations(&selected, &[12.0, 0.0]))
        .unwrap();
    assert_eq!(allocated.feedback().advance, 42.0);
    assert_eq!(glyphs(&allocated, &bytes), expected);
    assert_eq!(
        paints(&allocated),
        [
            (Range::new(0, 1), 32.0),
            (Range::new(1, 2), 22.0),
            (Range::new(2, 3), 0.0)
        ]
    );
    assert_eq!(allocated.selected_line(), selected);
    let restored = allocated.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(restored.feedback().advance, 30.0);
    assert_eq!(glyphs(&restored, &bytes), expected);
    assert_eq!(paints(&restored), paints(&control));
    assert_eq!(prepared.analyzed_source().source(), &source);
}
