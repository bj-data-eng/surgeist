//! Independent real-provider qualification of movement across shaping relations.
//! Authored SFNT rules, Unicode properties and endpoint half contributions are
//! the oracles. These new functional cases claim no preimplementation RED.
use super::fixtures::{allocations, ligature_font, select_method};
use super::*;
use crate::{SpacingBoundaryGeometry, SpacingSlotGeometry, SpacingSlotOwner, SpacingSlotTarget};
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, JoiningType, Script},
    script::ScriptWithExtensions,
};

const THREE: [Range; 3] = [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)];

fn words(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn source_piece(range: Range) -> crate::SpacingPiece {
    crate::SpacingPiece::Source {
        origin: 0,
        local: range,
        aggregate: range,
    }
}

fn try_prepare(bytes: &[u8], source: &Source, style: Style) -> crate::Result<PreparedText> {
    let extent = Range::new(0, source.text().len());
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: style,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![crate::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: extent,
                aggregate: extent,
                kind: crate::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system.prepare(&analyzed, PreparationOptions::default())
}

/// Real DFLT/curs -> GPOS3, A exit(300,500), B entry(100,200).
/// Ahem outlines, cmap and metrics are unchanged. This is an actual attachment
/// between two Latin EGCs, rather than a mark inside one EGC or joining context
/// outside an invocation. C is outside the lookup's coverage.
fn cross_unit_cursive_font() -> Vec<u8> {
    let mut gpos = words(&[1, 0, 10, 30, 44, 1]);
    gpos.extend(*b"DFLT");
    gpos.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0, 1]));
    gpos.extend(*b"curs");
    gpos.extend(words(&[8, 0, 1, 0, 1, 4, 3, 0, 1, 8]));
    gpos.extend(words(&[
        1, 14, 2, 0, 22, 28, 0, 1, 2, 35, 36, 1, 300, 500, 1, 100, 200,
    ]));
    let bytes = sfnt::replace_tables(AHEM, vec![(*b"GPOS", gpos)]);
    font_premises(&bytes, false);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map('C'), Some(GlyphId::new(37)));
    assert!(font.gpos().unwrap().lookup_list().is_ok());
    let table = sfnt::table(&bytes, *b"GPOS");
    let scripts = usize::from(word(table, 4));
    assert_eq!(word(table, scripts), 1);
    assert_eq!(&table[scripts + 2..scripts + 6], b"DFLT");
    let script = scripts + usize::from(word(table, scripts + 6));
    let language = script + usize::from(word(table, script));
    assert_eq!(word(table, language + 2), 0xffff);
    assert_eq!(
        (word(table, language + 4), word(table, language + 6)),
        (1, 0)
    );
    let features = usize::from(word(table, 6));
    assert_eq!(word(table, features), 1);
    assert_eq!(&table[features + 2..features + 6], b"curs");
    let feature = features + usize::from(word(table, features + 6));
    assert_eq!((word(table, feature + 2), word(table, feature + 4)), (1, 0));
    let list = usize::from(word(table, 8));
    assert_eq!(word(table, list), 1);
    let lookup = list + usize::from(word(table, list + 2));
    assert_eq!(
        (
            word(table, lookup),
            word(table, lookup + 2),
            word(table, lookup + 4)
        ),
        (3, 0, 1)
    );
    let sub = lookup + usize::from(word(table, lookup + 6));
    assert_eq!((word(table, sub), word(table, sub + 4)), (1, 2));
    let coverage = sub + usize::from(word(table, sub + 2));
    assert_eq!(
        (0..4)
            .map(|i| word(table, coverage + 2 * i))
            .collect::<Vec<_>>(),
        [1, 2, 35, 36]
    );
    assert_eq!((word(table, sub + 6), word(table, sub + 12)), (0, 0));
    for (record, anchor) in [(8, [1, 300, 500]), (10, [1, 100, 200])] {
        let start = sub + usize::from(word(table, sub + record));
        assert_eq!(
            (0..3)
                .map(|i| word(table, start + 2 * i))
                .collect::<Vec<_>>(),
            anchor
        );
    }
    bytes
}

fn attachment_refusal(
    error: &crate::Error,
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
) {
    assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
    let Some(crate::ErrorDetail::UnsupportedSpacing {
        stage: actual_stage,
        operation: actual_operation,
        reason,
        pieces,
        provider_atoms,
        contexts,
        constraints,
    }) = error.detail()
    else {
        panic!("expected actual cross-unit attachment capability: {error:?}")
    };
    assert_eq!(*actual_stage, stage);
    assert_eq!(*actual_operation, operation);
    assert_eq!(*reason, crate::SpacingFailureReason::CrossUnitConstraint);
    assert_eq!(
        pieces.as_ref(),
        [source_piece(THREE[0]), source_piece(THREE[1])]
    );
    assert_eq!(provider_atoms.as_ref(), [0..1, 1..2]);
    assert_eq!(contexts.len(), 2);
    for (context, extent) in contexts.iter().zip(&THREE[..2]) {
        assert_eq!(context.pieces.as_ref(), [source_piece(*extent)]);
    }
    assert_eq!(constraints.len(), 1);
    assert_eq!(
        constraints[0].kind,
        crate::SpacingConstraintKind::CursiveAttachment
    );
    assert_eq!(constraints[0].run, 0);
    assert_eq!(constraints[0].components.as_ref(), [0..1, 1..2]);
    // Actual GPOS attachment identity has no fabricated GSUB feature provenance.
    assert!(constraints[0].provenance.is_empty());
}

#[test]
fn actual_cross_egc_gpos_attachment_accepts_zero_refuses_material_and_keeps_outside_movement() {
    let bytes = cross_unit_cursive_font();
    let source = Source::identified("ABC", Id::from_u64(1566100), 1);
    let prepared = prepare(&bytes, &source, own(0.0), &THREE);
    // At size10: A ends at exit.x=3, B shifts by -entry.x=-1 and
    // advances10-1=9; B's cross offset is -(exit.y-entry.y)=-3.
    let expected = [
        (THREE[0], 35, 0.0, 0.0, 3.0),
        (THREE[1], 36, -1.0, -3.0, 9.0),
        (THREE[2], 37, 0.0, 0.0, 10.0),
    ];
    let ordinary = whole(&prepared);
    assert_eq!(raw(&ordinary, &bytes), expected);
    assert_eq!(ordinary.feedback().advance, 22.0);
    assert_eq!(
        paint_offsets(&ordinary),
        [(0.0, 0.0), (2.0, -3.0), (12.0, 0.0)]
    );
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(raw(&natural, &bytes), expected);
    let rows = natural.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Constrained)
    );
    assert_eq!(
        rows[1].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Independent)
    );
    assert!(!rows[0].can_expand());
    assert!(rows[1].can_expand());
    let zero = natural
        .with_spacing(allocations(&selected, &[0.0, 0.0]))
        .unwrap();
    assert_eq!(raw(&zero, &bytes), expected);
    assert_eq!(zero.feedback().advance, 22.0);
    let error = natural
        .with_spacing(allocations(&selected, &[2.0, 0.0]))
        .unwrap_err();
    attachment_refusal(
        &error,
        crate::SpacingStage::SelectedShaping,
        crate::SpacingOperation::InterCharacterAllocation,
    );
    let outside = natural
        .with_spacing(allocations(&selected, &[0.0, 4.0]))
        .unwrap();
    assert_eq!(outside.feedback().advance, 26.0);
    assert_eq!(outside.measurements().alignment().advance(), 26.0);
    assert_eq!(raw(&outside, &bytes), expected);
    assert_eq!(
        paint_offsets(&outside),
        [(0.0, 0.0), (2.0, -3.0), (16.0, 0.0)]
    );
    let error = try_prepare(&bytes, &source, own(4.0)).unwrap_err();
    attachment_refusal(
        &error,
        crate::SpacingStage::Preparation,
        crate::SpacingOperation::AuthoredTracking,
    );
    // Authored movement only outside the attachment succeeds as well: A0/B0
    // has zero mean, B0/C8 has4. A failed material operation cannot mutate it.
    let mut authored_source = source.clone();
    authored_source.span(THREE[2], own(8.0));
    let authored = whole(&prepare(&bytes, &authored_source, own(0.0), &THREE));
    assert_eq!(authored.feedback().advance, 26.0);
    assert_eq!(raw(&authored, &bytes), expected);
    assert_eq!(paint_offsets(&authored), paint_offsets(&outside));
    let restored = outside.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(restored.feedback().advance, 22.0);
    assert_eq!(raw(&restored, &bytes), expected);
    assert_eq!(paint_offsets(&restored), paint_offsets(&ordinary));
    assert_eq!(natural.feedback().advance, 22.0);
}

fn endpoint(unit: &crate::SpacingUnit, prepared: &PreparedText, extent: Range, tracking: f64) {
    assert_eq!(unit.letter_spacing_before(), tracking);
    assert_eq!(unit.letter_spacing_after(), tracking);
    assert_eq!(
        unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
        [SpacingSlotOwner::Source {
            origin: prepared.origin_views().next().unwrap().handle(),
            local: extent,
            aggregate: extent,
        }]
    );
}

#[test]
fn required_multi_egc_ligature_keeps_each_full_tracking_endpoint_for_the_outside_mean() {
    let bytes = ligature_font(*b"rlig");
    let mut source = Source::identified("fiA", Id::from_u64(1566101), 1);
    source.span(THREE[1], own(8.0));
    source.span(THREE[2], own(12.0));
    let prepared = prepare(&bytes, &source, own(2.0), &THREE);
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    for (slot, before, after) in [
        (&slots[0], (0, 2.0), (1, 8.0)),
        (&slots[1], (1, 8.0), (2, 12.0)),
    ] {
        let SpacingSlotTarget::Boundary {
            visual_before,
            visual_after,
        } = slot.target()
        else {
            panic!("required shaping cannot collapse distinct source TCU endpoints")
        };
        endpoint(&visual_before, &prepared, THREE[before.0], before.1);
        endpoint(&visual_after, &prepared, THREE[after.0], after.1);
    }
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let expected = [
        (Range::new(0, 2), 58, 0.0, 0.0, 10.0),
        (THREE[2], 35, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&line, &bytes), expected);
    // rlig suppresses only its internal gap. At i/A, full values8 and12
    // contribute4+6=10; the first component's2 must not replace i's8.
    assert_eq!(line.feedback().advance, 30.0);
    assert_eq!(paint_offsets(&line), [(0.0, 0.0), (20.0, 0.0)]);
    let rows = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(
        rows[0].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::ProtectedRequired)
    );
    assert_eq!(rows[0].advance, 0.0);
    assert_eq!(
        rows[1].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Independent)
    );
    assert_eq!(
        (rows[1].authored_letter_spacing, rows[1].advance),
        (10.0, 10.0)
    );
    let filled = line
        .with_spacing(allocations(&selected, &[0.0, 3.0]))
        .unwrap();
    assert_eq!(filled.feedback().advance, 33.0);
    assert_eq!(paint_offsets(&filled), [(0.0, 0.0), (23.0, 0.0)]);
    assert_eq!(raw(&filled, &bytes), expected);
    assert_eq!(
        reason(
            &line
                .with_spacing(allocations(&selected, &[1.0, 0.0]))
                .unwrap_err()
        ),
        &ProtocolError::IneligibleSpacingSlot { slot: 0 }
    );
    assert_eq!(line.feedback().advance, 30.0);
}

#[test]
fn adjacent_material_boundary_masks_optional_ligation_across_a_neighboring_zero_boundary() {
    for tag in [*b"liga", *b"clig"] {
        let bytes = ligature_font(tag);
        for (text, outside, pair, ligated, unligated, paints) in [
            (
                "Afi",
                THREE[0],
                [THREE[1], THREE[2]],
                vec![
                    (THREE[0], 35, 0.0, 0.0, 10.0),
                    (Range::new(1, 3), 58, 0.0, 0.0, 10.0),
                ],
                vec![
                    (THREE[0], 35, 0.0, 0.0, 10.0),
                    (THREE[1], 35, 0.0, 0.0, 10.0),
                    (THREE[2], 36, 0.0, 0.0, 10.0),
                ],
                vec![(0.0, 0.0), (12.0, 0.0), (22.0, 0.0)],
            ),
            (
                "fiA",
                THREE[2],
                [THREE[0], THREE[1]],
                vec![
                    (Range::new(0, 2), 58, 0.0, 0.0, 10.0),
                    (THREE[2], 35, 0.0, 0.0, 10.0),
                ],
                vec![
                    (THREE[0], 35, 0.0, 0.0, 10.0),
                    (THREE[1], 36, 0.0, 0.0, 10.0),
                    (THREE[2], 35, 0.0, 0.0, 10.0),
                ],
                vec![(0.0, 0.0), (10.0, 0.0), (22.0, 0.0)],
            ),
        ] {
            let mut source = Source::new(text);
            let control = whole(&prepare(&bytes, &source, own(0.0), &THREE));
            assert_eq!(raw(&control, &bytes), ligated);
            assert_eq!(control.feedback().advance, 20.0);
            source.span(outside, own(4.0));
            assert_eq!(source.spans()[0].range(), outside);
            // f0/i0 has zero mean, but the adjoining A4/f0 or i0/A4 boundary
            // masks one real component of fi. The real GSUB rule needs both.
            let line = whole(&prepare(&bytes, &source, own(0.0), &THREE));
            assert_eq!(raw(&line, &bytes), unligated);
            assert_eq!(line.feedback().advance, 32.0);
            assert_eq!(paint_offsets(&line), paints);

            // Text's CSS feature API is global within each real run. Finite
            // native Feature ranges are owned by Shaper's separate tests.
            let mut explicit = own(0.0);
            explicit.font = explicit
                .font
                .feature(format!("\"{}\" 1", std::str::from_utf8(&tag).unwrap()));
            let mut outside_style = explicit.clone();
            outside_style.letter_spacing = 4.0;
            let mut explicit_source = Source::new(text);
            explicit_source.span(outside, outside_style);
            let prepared = prepare(&bytes, &explicit_source, explicit, &THREE);
            let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
            let natural = selected
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            assert_eq!(raw(&natural, &bytes), ligated);
            assert_eq!(natural.feedback().advance, 22.0);
            let (pair_slot, exterior_slot) = if text == "Afi" { (1, 0) } else { (0, 1) };
            let rows = natural.spacing_slots().collect::<Vec<_>>();
            assert_eq!(
                rows[pair_slot].geometry,
                SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Constrained)
            );
            assert_eq!(rows[pair_slot].authored_letter_spacing, 0.0);
            assert_eq!(
                rows[exterior_slot].geometry,
                SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Independent)
            );
            assert_eq!(rows[exterior_slot].authored_letter_spacing, 2.0);
            let mut amounts = [0.0, 0.0];
            amounts[exterior_slot] = 4.0;
            let outside_line = natural
                .with_spacing(allocations(&selected, &amounts))
                .unwrap();
            assert_eq!(outside_line.feedback().advance, 26.0);
            assert_eq!(raw(&outside_line, &bytes), ligated);
            assert_eq!(paint_offsets(&outside_line), [(0.0, 0.0), (16.0, 0.0)]);
            amounts = [0.0, 0.0];
            amounts[pair_slot] = 1.0;
            let error = natural
                .with_spacing(allocations(&selected, &amounts))
                .unwrap_err();
            let Some(crate::ErrorDetail::UnsupportedSpacing {
                reason,
                stage,
                operation,
                pieces,
                provider_atoms,
                constraints,
                ..
            }) = error.detail()
            else {
                panic!("explicit optional pair must retain its real constraint: {error:?}")
            };
            assert_eq!(
                *reason,
                crate::SpacingFailureReason::PreservedOptionalLigature
            );
            assert_eq!(*stage, crate::SpacingStage::SelectedShaping);
            assert_eq!(
                *operation,
                crate::SpacingOperation::InterCharacterAllocation
            );
            assert_eq!(pieces.as_ref(), pair.map(source_piece));
            assert_eq!(
                provider_atoms
                    .iter()
                    .flat_map(|r| r.clone())
                    .collect::<Vec<_>>(),
                [pair[0].start, pair[1].start]
            );
            assert_eq!(constraints.len(), 1);
            assert_eq!(constraints[0].kind, crate::SpacingConstraintKind::Ligature);
            assert_eq!(
                constraints[0]
                    .components
                    .iter()
                    .flat_map(|r| r.clone())
                    .collect::<Vec<_>>(),
                [pair[0].start, pair[1].start]
            );
            assert!(constraints[0].provenance.iter().any(|p| p.tag() == surgeist_shaper::Tag::new(&tag) && p.role() == surgeist_shaper::FeatureRole::Optional && matches!(p.source(), surgeist_shaper::FeatureSource::Request(f) if f.origin() == surgeist_shaper::FeatureOrigin::Explicit && f.value == 1)));
            assert_eq!(natural.feedback().advance, 22.0);
            assert_eq!(raw(&natural, &bytes), ligated);
        }
    }
}

/// Reuse the owned four real joining lookups, selecting the advertised script's
/// actual OT route. The mapped Unicode scalar is not a Latin stand-in: ICU
/// independently verifies its script, letter category, extensions and joining.
fn joining_profile_font(cp: u32, script: Script, tag: [u8; 4]) -> Vec<u8> {
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get32(cp),
        GeneralCategory::OtherLetter
    );
    assert_eq!(
        CodePointMapData::<JoiningType>::new().get32(cp),
        JoiningType::DualJoining
    );
    assert_eq!(CodePointMapData::<Script>::new().get32(cp), script);
    assert_eq!(
        ScriptWithExtensions::new()
            .get_script_extensions_val32(cp)
            .iter()
            .collect::<Vec<_>>(),
        [script]
    );
    let base = crate::shaping_test_fonts::arabic_nonjoining_ahem();
    let mut gsub = sfnt::table(&base, *b"GSUB").to_vec();
    let scripts = usize::from(word(&gsub, 4));
    assert_eq!(word(&gsub, scripts), 1);
    assert_eq!(&gsub[scripts + 2..scripts + 6], b"arab");
    gsub[scripts + 2..scripts + 6].copy_from_slice(&tag);
    let mapping = crate::shaping_test_fonts::range_ligature_ahem(
        &[35, 36],
        &[(0x41, 35), (0x42, 36), (0x43, 37), (cp, 35)],
    );
    let bytes = sfnt::replace_tables(
        &base,
        vec![
            (*b"GSUB", gsub),
            (*b"cmap", sfnt::table(&mapping, *b"cmap").to_vec()),
        ],
    );
    font_premises(&bytes, false);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(
        font.charmap().map(char::from_u32(cp).unwrap()),
        Some(GlyphId::new(35))
    );
    assert!(font.table_data(skrifa::Tag::new(b"GPOS")).is_none());
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for id in [35, 82, 100, 277] {
        assert_eq!(metrics.advance_width(GlyphId::new(id)), Some(1000.0));
    }
    assert!(font.gsub().unwrap().lookup_list().is_ok());
    let gsub = sfnt::table(&bytes, *b"GSUB");
    assert_eq!(&gsub[scripts + 2..scripts + 6], &tag);
    let script_table = scripts + usize::from(word(gsub, scripts + 6));
    assert_eq!(word(gsub, script_table + 2), 0);
    let language = script_table + usize::from(word(gsub, script_table));
    assert_eq!(
        (word(gsub, language + 2), word(gsub, language + 4)),
        (0xffff, 4)
    );
    assert_eq!(
        (0..4)
            .map(|i| word(gsub, language + 6 + i * 2))
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    let features = usize::from(word(gsub, 6));
    let list = usize::from(word(gsub, 8));
    assert_eq!((word(gsub, features), word(gsub, list)), (4, 4));
    for (ordinal, (feature_tag, inputs, outputs)) in [
        (*b"fina", vec![35, 58], vec![82, 274]),
        (*b"init", vec![35], vec![100]),
        (*b"isol", vec![35], vec![35]),
        (*b"medi", vec![35], vec![277]),
    ]
    .into_iter()
    .enumerate()
    {
        let record = features + 2 + 6 * ordinal;
        assert_eq!(&gsub[record..record + 4], &feature_tag);
        let feature = features + usize::from(word(gsub, record + 4));
        assert_eq!(word(gsub, feature + 2), 1);
        let index = usize::from(word(gsub, feature + 4));
        let lookup = list + usize::from(word(gsub, list + 2 + index * 2));
        assert_eq!((word(gsub, lookup), word(gsub, lookup + 4)), (1, 1));
        let sub = lookup + usize::from(word(gsub, lookup + 6));
        assert_eq!(
            (word(gsub, sub), usize::from(word(gsub, sub + 4))),
            (2, outputs.len())
        );
        assert_eq!(
            (0..outputs.len())
                .map(|i| word(gsub, sub + 6 + 2 * i))
                .collect::<Vec<_>>(),
            outputs
        );
        let coverage = sub + usize::from(word(gsub, sub + 2));
        assert_eq!(
            (word(gsub, coverage), usize::from(word(gsub, coverage + 2))),
            (1, inputs.len())
        );
        assert_eq!(
            (0..inputs.len())
                .map(|i| word(gsub, coverage + 4 + 2 * i))
                .collect::<Vec<_>>(),
            inputs
        );
    }
    bytes
}

fn sorted_raw(line: &FinalizedLine, bytes: &[u8]) -> Vec<RawGlyph> {
    let mut value = raw(line, bytes);
    value.sort_by_key(|glyph| glyph.0.start);
    value
}

fn source_paints(line: &FinalizedLine) -> Vec<(Range, f64)> {
    let mut value = line
        .run_views()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .map(move |glyph| (glyph.range(), origin.x + glyph.pen() + f64::from(glyph.x())))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    value.sort_by_key(|row| row.0.start);
    value
}

fn qualify_joining_profile(cp: u32, script: Script, tag: [u8; 4], rtl: bool) {
    let bytes = joining_profile_font(cp, script, tag);
    let ch = char::from_u32(cp).unwrap();
    let width = ch.len_utf16();
    let isolated_source = Source::new(ch.to_string());
    let isolated = whole(&prepare(
        &bytes,
        &isolated_source,
        own(0.0),
        &[Range::new(0, width)],
    ));
    assert_eq!(
        sorted_raw(&isolated, &bytes),
        [(Range::new(0, width), 35, 0.0, 0.0, 10.0)]
    );
    let pair_source = Source::new(format!("{ch}{ch}"));
    let pair_ranges = [Range::new(0, width), Range::new(width, width * 2)];
    let pair = whole(&prepare(&bytes, &pair_source, own(0.0), &pair_ranges));
    // Initial100/final82, unlike isolated35, prove this script's real joining
    // masks selected the authored lookups. The font has no GPOS attachments.
    assert_eq!(
        sorted_raw(&pair, &bytes),
        [
            (pair_ranges[0], 100, 0.0, 0.0, 10.0),
            (pair_ranges[1], 82, 0.0, 0.0, 10.0),
        ]
    );
    assert_eq!(pair.feedback().advance, 20.0);
    let ranges = [
        Range::new(0, 1),
        Range::new(1, 1 + width),
        Range::new(1 + width, 1 + width * 2),
    ];
    let mut source = Source::new(format!("A{ch}{ch}"));
    source.span(ranges[1], own(8.0));
    source.span(ranges[2], own(12.0));
    let prepared = prepare(&bytes, &source, own(4.0), &ranges);
    assert_eq!(
        prepared
            .unit_views()
            .map(|u| u.level().get())
            .collect::<Vec<_>>(),
        if rtl { vec![0, 1, 1] } else { vec![0, 0, 0] }
    );
    let ordinary = whole(&prepared);
    let expected = [
        (ranges[0], 35, 0.0, 0.0, 10.0),
        (ranges[1], 100, 0.0, 0.0, 10.0),
        (ranges[2], 82, 0.0, 0.0, 10.0),
    ];
    assert_eq!(sorted_raw(&ordinary, &bytes), expected);
    // Visual A4/last12 gives8 for RTL; A4/first8 gives6 for LTR. The
    // same-script letter pair's mean is suppressed despite unequal8/12.
    let gap = if rtl { 8.0 } else { 6.0 };
    assert_eq!(ordinary.feedback().advance, 30.0 + gap);
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    assert_eq!(selected.spacing_slots().len(), 2);
    let exterior = selected.spacing_slots().next().unwrap();
    let SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = exterior.target()
    else {
        panic!("outside profile boundary must retain its visual endpoints")
    };
    endpoint(&visual_before, &prepared, ranges[0], 4.0);
    endpoint(
        &visual_after,
        &prepared,
        ranges[if rtl { 2 } else { 1 }],
        if rtl { 12.0 } else { 8.0 },
    );
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let rows = natural.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Independent)
    );
    assert_eq!(rows[0].authored_letter_spacing, gap);
    assert!(rows[0].can_expand());
    assert_eq!(
        rows[1].geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::ProtectedCursive)
    );
    assert_eq!(
        (
            rows[1].authored_letter_spacing,
            rows[1].allocation,
            rows[1].advance
        ),
        (0.0, 0.0, 0.0)
    );
    assert!(!rows[1].can_expand());
    assert_eq!(sorted_raw(&natural, &bytes), expected);
    assert_eq!(natural.feedback().advance, 30.0 + gap);
    let expanded = natural
        .with_spacing(allocations(&selected, &[2.0, 0.0]))
        .unwrap();
    assert_eq!(sorted_raw(&expanded, &bytes), expected);
    assert_eq!(expanded.feedback().advance, 32.0 + gap);
    assert_eq!(expanded.measurements().alignment().advance(), 32.0 + gap);
    assert_eq!(
        source_paints(&expanded),
        [
            (ranges[0], 0.0),
            (ranges[1], if rtl { 22.0 + gap } else { 12.0 + gap }),
            (ranges[2], if rtl { 12.0 + gap } else { 22.0 + gap }),
        ]
    );
    assert_eq!(
        reason(
            &natural
                .with_spacing(allocations(&selected, &[0.0, 2.0]))
                .unwrap_err()
        ),
        &ProtocolError::IneligibleSpacingSlot { slot: 1 }
    );
    let restored = expanded.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(restored.feedback().advance, ordinary.feedback().advance);
    assert_eq!(source_paints(&restored), source_paints(&ordinary));
    assert_eq!(sorted_raw(&restored, &bytes), expected);
    assert_eq!(prepared.analyzed_source().source(), &source);
}

// Each advertised Appendix D script has its own executable case. Qualification
// is deliberately bounded to these two actual dual-joining letters plus an
// outside Latin boundary; it makes no claim about every orthography or font.
#[test]
fn syriac_real_script_route_keeps_same_script_no_gap_and_independent_outside_boundary() {
    qualify_joining_profile(0x712, Script::Syriac, *b"syrc", true);
}

#[test]
fn nko_real_joining_use_route_keeps_same_script_no_gap_and_independent_outside_boundary() {
    qualify_joining_profile(0x7ca, Script::Nko, *b"nko ", true);
}

#[test]
fn arabic_real_script_route_keeps_same_script_no_gap_and_independent_outside_boundary() {
    qualify_joining_profile(0x628, Script::Arabic, *b"arab", true);
}

#[test]
fn hanifi_rohingya_real_joining_use_route_keeps_same_script_no_gap_and_independent_outside_boundary()
 {
    qualify_joining_profile(0x10d01, Script::HanifiRohingya, *b"rohg", true);
}

#[test]
fn mandaic_real_joining_use_route_keeps_same_script_no_gap_and_independent_outside_boundary() {
    qualify_joining_profile(0x841, Script::Mandaic, *b"mand", true);
}

#[test]
fn mongolian_real_joining_use_route_keeps_same_script_no_gap_and_independent_outside_boundary() {
    qualify_joining_profile(0x1820, Script::Mongolian, *b"mong", false);
}

#[test]
fn phags_pa_real_joining_use_route_keeps_same_script_no_gap_and_independent_outside_boundary() {
    qualify_joining_profile(0xa840, Script::PhagsPa, *b"phag", false);
}
