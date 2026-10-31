//! A final source-context change can expose IC demand beyond a DI component.
use super::*;
use icu_properties::script::ScriptWithExtensions;

fn bang_font() -> Vec<u8> {
    let optional = conditional_skip_font(*b"liga");
    let mut gsub = crate::test_fonts::table(&optional, *b"GSUB").to_vec();
    let list = usize::from(crate::test_fonts::u16_at(&gsub, 8));
    assert_eq!(crate::test_fonts::u16_at(&gsub, list), 1);
    let lookup = list + usize::from(crate::test_fonts::u16_at(&gsub, list + 2));
    assert_eq!(crate::test_fonts::u16_at(&gsub, lookup), 4);
    assert_eq!(crate::test_fonts::u16_at(&gsub, lookup + 2), 0);
    let sub = lookup + usize::from(crate::test_fonts::u16_at(&gsub, lookup + 6));
    let set = sub + usize::from(crate::test_fonts::u16_at(&gsub, sub + 6));
    let ligature = set + usize::from(crate::test_fonts::u16_at(&gsub, set + 2));
    assert_eq!(crate::test_fonts::u16_at(&gsub, ligature), font::R);
    assert_eq!(crate::test_fonts::u16_at(&gsub, ligature + 2), 2);
    assert_eq!(crate::test_fonts::u16_at(&gsub, ligature + 4), font::B);
    // Genuine optional A35+A35 -> R58. Literal ! aliases A35; its source
    // category and script remain punctuation/Common, independently of glyph ID.
    gsub[ligature + 4..ligature + 6].copy_from_slice(&font::A.to_be_bytes());
    let bytes = crate::test_fonts::replace_tables(&optional, vec![(*b"GSUB", gsub)]);
    with_cmap(
        &bytes,
        &[(0x21, u32::from(font::A)), (0x180e, u32::from(font::Q))],
    )
}

fn native_bangs(
    bytes: &[u8],
    disabled: Option<std::ops::Range<usize>>,
    expected: &[(u32, u32, i32)],
    ligature: Option<&[u32]>,
) -> Native {
    let face = surgeist_shaper::FontRef::new(bytes).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut input = surgeist_shaper::UnicodeBuffer::new();
    for scalar in 0..3 {
        input.add('!', scalar);
    }
    input.set_direction(surgeist_shaper::Direction::LeftToRight);
    input.set_script(surgeist_shaper::script::MONGOLIAN);
    input.observe_components();
    // Authentic disconnections fence both contexts. The source Common run
    // inherits the preceding real Mongolian script, not a guessed Latin script.
    let features = disabled
        .map(|range| {
            surgeist_shaper::Feature::optional_default(surgeist_shaper::Tag::new(b"liga"), 0, range)
        })
        .into_iter()
        .collect::<Vec<_>>();
    let output = shaper.shape(input, &features);
    assert_eq!(
        output.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    let di = output.default_ignorable_receipt().unwrap();
    assert!(di.residual_input_positions().is_empty());
    assert!(di.deleted_input_positions().is_empty());
    let glyphs = output
        .glyph_infos()
        .iter()
        .zip(output.glyph_positions())
        .map(|(info, pos)| {
            (
                info.glyph_id,
                info.cluster,
                pos.x_advance,
                pos.x_offset,
                pos.y_offset,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        expected
            .iter()
            .map(|&(id, cluster, advance)| (id, cluster, advance, 0, 0))
            .collect::<Vec<_>>()
    );
    let receipt = output.component_receipt().unwrap().unwrap();
    assert_eq!(
        receipt.coverage(),
        surgeist_shaper::ComponentCoverage::Complete
    );
    let relations = receipt.ligatures().collect::<Vec<_>>();
    if let Some(components) = ligature {
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].components(), components);
        assert!(
            relations[0]
                .provenance()
                .iter()
                .any(|row| row.tag() == surgeist_shaper::Tag::new(b"liga")
                    && row.role() == surgeist_shaper::FeatureRole::Optional)
        );
    } else {
        assert!(relations.is_empty());
    }
    Native {
        glyphs,
        clusters: output
            .glyph_infos()
            .iter()
            .map(|info| info.cluster)
            .collect(),
        ligatures: relations
            .iter()
            .map(|row| row.components().to_vec())
            .collect(),
        residual: vec![],
        deleted: vec![],
    }
}

fn every_final_ic_endpoint_was_covered(selected: &SelectedLine) {
    assert_eq!(
        selected.inner.request.context.spacing,
        SpacingPolicy::InterCharacter
    );
    let basis = &selected.inner.basis;
    for (edge, boundary) in basis.characters.boundaries.iter().enumerate() {
        let movement = basis
            .characters
            .movement_geometry(edge, &basis.shaped.character_invocations, &mut None)
            .unwrap();
        if matches!(
            movement,
            crate::SpacingBoundaryGeometry::ProtectedRequired
                | crate::SpacingBoundaryGeometry::ProtectedCursive
        ) || boundary.geometry == crate::SpacingBoundaryGeometry::ProtectedCursive
        {
            continue;
        }
        // IC potential is material even with authored0. Inspect only actual
        // final units and the retained interval union; do not plan contexts.
        for endpoint in [boundary.before, boundary.after] {
            for atoms in &basis.characters.units[endpoint].atoms {
                assert_covered(selected, atoms.clone(), false);
            }
        }
    }
}

#[test]
fn residual_mvs_context_change_covers_later_ic_endpoints_across_disconnections() {
    let bytes = bang_font();
    packet_metrics(
        &bytes,
        &[
            ('A', 35, 1000.0),
            ('!', 35, 1000.0),
            ('\u{180e}', 277, 2500.0),
            ('R', 58, 1500.0),
        ],
    );
    // Establish actual native MVS residual exclusion before the target. The
    // positive preserved packet rules out a missing-font/zero-metric proxy.
    assert_eq!(
        font::native_glyphs(&bytes, "\u{180e}\u{180e}"),
        [(277, 2500), (277, 2500)]
    );
    let mvs = native_in_context(
        &bytes,
        "\u{180e}\u{180e}",
        LTR,
        Some((surgeist_shaper::script::MONGOLIAN, "", "", false)),
    );
    assert!(mvs.glyphs.is_empty());
    assert_eq!(mvs.residual, [0, 1]);
    assert!(mvs.deleted.is_empty());
    let a = native_in_context(
        &bytes,
        "A",
        LTR,
        Some((surgeist_shaper::script::LATIN, "", "", false)),
    );
    assert_eq!(a.glyphs, [(35, 0, 1000, 0, 0)]);
    assert!(a.residual.is_empty() && a.deleted.is_empty());
    let _ordinary = native_bangs(&bytes, None, &[(58, 0, 1500), (35, 2, 1000)], Some(&[0, 1]));
    let _first_only = native_bangs(
        &bytes,
        Some(0..1),
        &[(35, 0, 1000), (58, 1, 1500)],
        Some(&[1, 2]),
    );
    let independent = native_bangs(
        &bytes,
        Some(0..3),
        &[(35, 0, 1000), (35, 1, 1000), (35, 2, 1000)],
        None,
    );
    let native = join_native([(0, a), (1, mvs), (3, independent)]);

    let scripts = CodePointMapData::<Script>::new();
    assert_eq!(scripts.get('A'), Script::Latin);
    assert_eq!(scripts.get('!'), Script::Common);
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get('!'),
        GeneralCategory::OtherPunctuation
    );
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get('\u{180e}'),
        GeneralCategory::Format
    );
    assert!(CodePointSetData::new::<DefaultIgnorableCodePoint>().contains('\u{180e}'));
    assert_eq!(
        CodePointMapData::<GraphemeClusterBreak>::new().get('\u{180e}'),
        GraphemeClusterBreak::Control
    );
    assert_eq!(
        ScriptWithExtensions::new()
            .get_script_extensions_val32(0x180e)
            .iter()
            .collect::<Vec<_>>(),
        [Script::Mongolian]
    );
    // Six original singleton EGCs give two real Mongolian source units versus
    // one Latin unit. Both MVS occurrences are later excluded by actual native
    // receipts: the remaining Common punctuation resolves against Latin alone.
    // These are property/source premises, not another context/spacing planner.
    let source = Source::new("A\u{180e}\u{180e}!!!");
    let mut value = input(&source, own(0.0), LTR);
    value.base_style.locale = None;
    value.origins = [Range::new(0, 1), Range::new(1, 3), Range::new(3, 6)]
        .into_iter()
        .map(|range| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    value.connections = [1, 3]
        .into_iter()
        .map(|at| crate::ParagraphConnection {
            at,
            kind: crate::ParagraphConnectionKind::Disconnected,
        })
        .collect();
    let mut system = system(&bytes);
    let prepared = prepare(&mut system, value, LineOrientation::Horizontal);
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        (0..6).map(|i| Range::new(i, i + 1)).collect::<Vec<_>>()
    );
    assert!(
        prepared
            .inner
            .styles
            .iter()
            .all(|row| row.style.locale.is_none() && row.style.letter_spacing == 0.0)
    );
    for range in [Range::new(1, 2), Range::new(2, 3)] {
        assert_eq!(
            prepared
                .unit_views()
                .find(|unit| unit.source_range() == range)
                .unwrap()
                .advance(),
            0.0
        );
    }
    reset_provider_calls();
    let selected = prepared
        .select(request(&prepared, LTR, SpacingPolicy::InterCharacter))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    let presentation = &selected.inner.presentation;
    // Ordinary content has no indication-local domain. The two real native
    // context barriers therefore do not erase its shared source-profile domain.
    assert!(presentation.domains.is_empty());
    assert_eq!(presentation.projection.barrier_atoms, [1, 3]);
    assert_eq!(
        presentation
            .analysis
            .infos
            .iter()
            .map(|(info, _)| info.script)
            .collect::<Vec<_>>(),
        [
            Script::Latin,
            Script::Mongolian,
            Script::Mongolian,
            Script::Common,
            Script::Common,
            Script::Common
        ]
    );
    let shaped = &selected.inner.basis.shaped;
    assert_eq!(exclusion_rows(shaped), [(1, false), (2, false)]);
    assert_eq!(
        shaped
            .runs
            .iter()
            .map(|run| (run.atoms.clone(), run.script))
            .collect::<Vec<_>>(),
        [
            (0..1, Script::Latin),
            (1..3, Script::Mongolian),
            (3..6, Script::Mongolian)
        ]
    );
    assert!(
        shaped
            .runs
            .iter()
            .all(|run| run.font.bytes() == bytes.as_slice())
    );
    assert_eq!(
        shaped
            .character_invocations
            .iter()
            .map(|run| (run.atoms.clone(), run.run))
            .collect::<Vec<_>>(),
        [(0..1, 0), (1..3, 1), (3..6, 2)]
    );
    let graph = &selected.inner.basis.characters;
    assert_eq!(graph.units.len(), 4);
    assert_eq!(
        graph.units.iter().map(|unit| unit.base).collect::<Vec<_>>(),
        [Some(0), Some(3), Some(4), Some(5)]
    );
    assert!(
        graph
            .units
            .iter()
            .all(|unit| unit.left == 0.0 && unit.right == 0.0)
    );
    assert_eq!(graph.boundaries.len(), 3);
    assert!(
        graph
            .boundaries
            .iter()
            .all(|boundary| boundary.authored == 0.0
                && boundary.geometry == crate::SpacingBoundaryGeometry::Independent)
    );
    every_final_ic_endpoint_was_covered(&selected);

    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 3);
    let origins = prepared
        .origin_views()
        .map(|origin| origin.handle())
        .collect::<Vec<_>>();
    for (slot, pair) in slots.iter().zip([[0, 3], [3, 4], [4, 5]]) {
        let crate::SpacingSlotTarget::Boundary {
            visual_before,
            visual_after,
        } = slot.target()
        else {
            panic!("actual final IC boundary");
        };
        for (unit, atom) in [visual_before, visual_after].into_iter().zip(pair) {
            let range = Range::new(atom, atom + 1);
            assert_eq!(
                unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
                [crate::SpacingSlotOwner::Source {
                    origin: origins[if atom == 0 { 0 } else { 2 }].clone(),
                    local: range,
                    aggregate: range
                }]
            );
        }
    }
    let coverage = coverage_signature(&selected);
    let natural = finalize_retained(&selected, calls);
    assert_eq!(natural.feedback().advance, 40.0);
    assert_native_packets(&natural, &bytes, "A\u{180e}\u{180e}!!!", &native);
    assert_eq!(inline_paints(&natural), [0.0, 10.0, 20.0, 30.0]);
    assert!(natural.spacing_slots().all(|slot| slot.geometry
        == crate::SpacingSlotGeometry::Boundary(crate::SpacingBoundaryGeometry::Independent)));
    let allocated = natural
        .with_spacing(amounts(&selected, &[0.0, 2.0, 3.0]))
        .unwrap();
    assert_calls(calls);
    assert_eq!(allocated.feedback().advance, 45.0);
    assert_eq!(inline_paints(&allocated), [0.0, 10.0, 22.0, 35.0]);
    assert_eq!(packets(&allocated, &bytes), packets(&natural, &bytes));
    let restored = allocated.with_spacing(SpacingAllocations::None).unwrap();
    assert_calls(calls);
    assert_eq!(restored.feedback().advance, 40.0);
    assert_eq!(inline_paints(&restored), [0.0, 10.0, 20.0, 30.0]);
    assert_eq!(natural.feedback().advance, 40.0);
    assert_eq!(coverage_signature(&selected), coverage);
    source_copy(&prepared, allocated, &source);
}
