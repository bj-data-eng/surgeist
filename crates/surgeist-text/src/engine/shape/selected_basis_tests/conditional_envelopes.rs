//! Real native outcomes and canonical endpoint coverage of upfront envelopes.
use super::*;
use crate::engine::spacing::character::{PlanReason, PlanRequirements};
use icu_properties::{
    CodePointMapData, CodePointSetData,
    props::{
        DefaultIgnorableCodePoint, ExtendedPictographic, GeneralCategory, GraphemeClusterBreak,
        Script,
    },
};

fn requirements(selected: &SelectedLine) -> &PlanRequirements {
    selected
        .inner
        .basis
        .shaped
        .character_requirements
        .as_deref()
        .expect("the actual selected basis retains its upfront requirements")
}

fn coverage_signature(selected: &SelectedLine) -> Vec<(std::ops::Range<usize>, bool)> {
    requirements(selected)
        .coverage
        .iter()
        .map(|row| {
            (
                row.atoms.clone(),
                matches!(&row.reason, PlanReason::ConditionalEnvelope),
            )
        })
        .collect()
}

fn assert_covered(selected: &SelectedLine, atoms: std::ops::Range<usize>, conditional: bool) {
    let plan = requirements(selected);
    for atom in atoms {
        assert!(
            plan.coverage.iter().any(|row| row.atoms.contains(&atom)
                && (!conditional || matches!(&row.reason, PlanReason::ConditionalEnvelope))),
            "actual endpoint atom{atom} lacks its upfront coverage: {:?}",
            coverage_signature(selected)
        );
    }
}

fn every_final_material_endpoint_was_covered(selected: &SelectedLine) {
    // Inspect the actual receipt-qualified graph. This does not construct any
    // potential units, exclusions, segmentation or envelope of its own.
    let basis = &selected.inner.basis;
    for (edge, boundary) in basis.characters.boundaries.iter().enumerate() {
        if boundary.authored == 0.0 {
            continue;
        }
        let movement = basis
            .characters
            .movement_geometry(edge, &basis.shaped.character_invocations);
        if matches!(
            movement,
            crate::SpacingBoundaryGeometry::ProtectedRequired
                | crate::SpacingBoundaryGeometry::ProtectedCursive
        ) || boundary.geometry == crate::SpacingBoundaryGeometry::ProtectedCursive
        {
            continue;
        }
        for endpoint in [boundary.before, boundary.after] {
            for atoms in &basis.characters.units[endpoint].atoms {
                assert_covered(selected, atoms.clone(), false);
            }
        }
    }
}

fn no_optional_controls(selected: &SelectedLine) {
    assert!(
        selected
            .inner
            .basis
            .shaped
            .character_requirements
            .as_ref()
            .is_none_or(|plan| plan.coverage.is_empty())
    );
}

fn select_counted(
    prepared: &PreparedText,
    direction: ParagraphBaseDirection,
) -> (SelectedLine, usize) {
    reset_provider_calls();
    let selected = prepared
        .select(request(prepared, direction, SpacingPolicy::None))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    (selected, calls)
}

fn finalize_retained(selected: &SelectedLine, calls: usize) -> FinalizedLine {
    let coverage = coverage_signature(selected);
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_calls(calls);
    let restored = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_calls(calls);
    assert_eq!(restored.feedback().advance, line.feedback().advance);
    assert_eq!(coverage_signature(selected), coverage);
    every_final_material_endpoint_was_covered(selected);
    line
}

fn with_cmap(bytes: &[u8], extra: &[(u32, u32)]) -> Vec<u8> {
    let mut mappings = vec![
        (0x20, 274),
        (0x41, 35),
        (0x42, 36),
        (0x43, 100),
        (0x51, 277),
        (0x52, 58),
    ];
    mappings.extend_from_slice(extra);
    let carrier = crate::shaping_test_fonts::range_ligature_ahem(&[82, 82], &mappings);
    crate::test_fonts::replace_tables(
        bytes,
        vec![(
            *b"cmap",
            crate::test_fonts::table(&carrier, *b"cmap").to_vec(),
        )],
    )
}

#[derive(Debug)]
struct Native {
    glyphs: Vec<(u32, u32, i32, i32, i32)>,
    clusters: Vec<u32>,
    ligatures: Vec<Vec<u32>>,
    residual: Vec<u32>,
    deleted: Vec<u32>,
}

fn native(bytes: &[u8], text: &str, direction: ParagraphBaseDirection) -> Native {
    native_in_context(bytes, text, direction, None)
}

fn native_in_context(
    bytes: &[u8],
    text: &str,
    direction: ParagraphBaseDirection,
    context: Option<(surgeist_shaper::Script, &str, &str, bool)>,
) -> Native {
    let face = surgeist_shaper::FontRef::new(bytes).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut input = surgeist_shaper::UnicodeBuffer::new();
    // Actual Text input is indexed by local scalar ordinals, not UTF-8 bytes.
    for (ordinal, scalar) in text.chars().enumerate() {
        input.add(scalar, u32::try_from(ordinal).unwrap());
    }
    input.set_direction(if direction == LTR {
        surgeist_shaper::Direction::LeftToRight
    } else {
        surgeist_shaper::Direction::RightToLeft
    });
    input.observe_components();
    input.guess_segment_properties();
    if let Some((script, before, after, continuation_mark)) = context {
        input.set_script(script);
        input.set_pre_context(before);
        input.set_post_context(after);
        input.set_flags(if continuation_mark {
            surgeist_shaper::BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE
        } else {
            surgeist_shaper::BufferFlags::empty()
        });
    }
    let output = shaper.shape(input, &[]);
    assert_eq!(
        output.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    let receipt = output.default_ignorable_receipt().unwrap();
    let glyphs = output
        .glyph_infos()
        .iter()
        .zip(output.glyph_positions())
        .enumerate()
        .filter(|(index, _)| {
            !receipt
                .residual_glyph_positions()
                .contains(&u32::try_from(*index).unwrap())
        })
        .map(|(_, (info, pos))| {
            (
                info.glyph_id,
                info.cluster,
                pos.x_advance,
                pos.x_offset,
                pos.y_offset,
            )
        })
        .collect();
    Native {
        glyphs,
        clusters: output
            .glyph_infos()
            .iter()
            .map(|info| info.cluster)
            .collect(),
        ligatures: output
            .component_receipt()
            .unwrap()
            .unwrap()
            .ligatures()
            .map(|relation| relation.components().to_vec())
            .collect(),
        residual: receipt.residual_input_positions().to_vec(),
        deleted: receipt.deleted_input_positions().to_vec(),
    }
}

// Join real native receipts only; offsets are the independently established
// invocation input ownership, not source ranges inferred from Text glyph output.
fn join_native(controls: impl IntoIterator<Item = (u32, Native)>) -> Native {
    let mut combined = Native {
        glyphs: vec![],
        clusters: vec![],
        ligatures: vec![],
        residual: vec![],
        deleted: vec![],
    };
    for (offset, control) in controls {
        combined.glyphs.extend(
            control
                .glyphs
                .into_iter()
                .map(|(id, cluster, advance, x, y)| (id, cluster + offset, advance, x, y)),
        );
        combined
            .clusters
            .extend(control.clusters.into_iter().map(|c| c + offset));
        combined.ligatures.extend(
            control
                .ligatures
                .into_iter()
                .map(|components| components.into_iter().map(|c| c + offset).collect()),
        );
        combined
            .residual
            .extend(control.residual.into_iter().map(|c| c + offset));
        combined
            .deleted
            .extend(control.deleted.into_iter().map(|c| c + offset));
    }
    combined
}

fn assert_native_packets(line: &FinalizedLine, bytes: &[u8], source: &str, native: &Native) {
    // Independent provider control supplies original scalar cluster indices;
    // this direct UTF-16 map preserves their exact source packet custody. Keep
    // residual glyph-cluster stops too: hidden output cannot donate its range.
    let mut offsets = vec![0];
    for scalar in source.chars() {
        offsets.push(offsets.last().copied().unwrap() + scalar.len_utf16());
    }
    let mut stops = native
        .clusters
        .iter()
        .map(|&cluster| usize::try_from(cluster).unwrap())
        .collect::<Vec<_>>();
    stops.push(offsets.len() - 1);
    stops.sort_unstable();
    stops.dedup();
    let expected = native
        .glyphs
        .iter()
        .map(|&(id, cluster, advance, x, y)| {
            let start = usize::try_from(cluster).unwrap();
            let end = stops.iter().copied().find(|&stop| stop > start).unwrap();
            (
                Range::new(offsets[start], offsets[end]),
                id,
                x as f32 / 100.0,
                -y as f32 / 100.0,
                advance as f32 / 100.0,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(packets(line, bytes), expected);
}

fn packet_metrics(bytes: &[u8], expected: &[(char, u32, f32)]) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for &(scalar, glyph, advance) in expected {
        assert_eq!(font.charmap().map(scalar), Some(GlyphId::new(glyph)));
        assert_eq!(metrics.advance_width(GlyphId::new(glyph)), Some(advance));
    }
}

#[test]
fn genuine_normal_di_keeps_the_upfront_conditional_plan_when_final_means_are_zero() {
    let bytes = font::fixture(Some('\u{2065}'), font::Action::PartialDelete);
    resource_premises(&bytes);
    assert_eq!(font::native_glyphs(&bytes, "C"), [(58, 1500)]);
    let control = native(&bytes, "A\u{2065}B", LTR);
    assert_eq!(
        control.glyphs,
        [
            (35, 0, 1000, 0, 0),
            (58, 1, 1500, 0, 0),
            (36, 2, 2000, 0, 0)
        ]
    );
    assert!(control.residual.is_empty() && control.deleted.is_empty());
    let source = conditional_tracking_source(4.0);
    let mut system = system(&bytes);
    let prepared = prepare(
        &mut system,
        input(&source, own(4.0), LTR),
        LineOrientation::Horizontal,
    );
    let (selected, calls) = select_counted(&prepared, LTR);
    assert!(exclusion_rows(&selected.inner.basis.shaped).is_empty());
    assert_eq!(selected.inner.basis.characters.boundaries.len(), 2);
    assert!(
        selected
            .inner
            .basis
            .characters
            .boundaries
            .iter()
            .all(|boundary| boundary.authored == 0.0)
    );
    assert!(requirements(&selected).observe);
    assert_covered(&selected, 0..1, true);
    assert_covered(&selected, 2..3, true);
    let line = finalize_retained(&selected, calls);
    assert_eq!(
        packets(&line, &bytes),
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 58, 0.0, 0.0, 15.0),
            (Range::new(2, 3), 36, 0.0, 0.0, 20.0)
        ]
    );
    assert_eq!(inline_paints(&line), [0.0, 10.0, 25.0]);
    assert_eq!(line.feedback().advance, 45.0);
    assert_native_packets(&line, &bytes, "A\u{2065}B", &control);
    source_copy(&prepared, line, &source);
}

fn marked_font() -> Vec<u8> {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    // Keep the real GDEF and GPOS mark/base relation; only add nominal DI
    // coverage on unrelated glyph277 and preserve the literal licensed metrics.
    let carrier = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[
            (0x20, 3),
            (0x41, 35),
            (0x42, 36),
            (0x301, 100),
            (0x2065, 277),
        ],
    );
    crate::test_fonts::replace_tables(
        &bytes,
        vec![(
            *b"cmap",
            crate::test_fonts::table(&carrier, *b"cmap").to_vec(),
        )],
    )
}

#[test]
fn residual_removal_coalesces_the_real_mark_unit_and_covers_its_outside_endpoint() {
    let bytes = marked_font();
    packet_metrics(&bytes, &[('A', 35, 1000.0), ('B', 36, 1000.0)]);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map('\u{301}'), Some(GlyphId::new(100)));
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get('\u{301}'),
        GeneralCategory::NonspacingMark
    );
    let expected = [
        (35, 0, 1000, 0, 0),
        (100, 0, 0, -700, 400),
        (36, 2, 1000, 0, 0),
    ];
    assert_eq!(native(&bytes, "A\u{301}B", LTR).glyphs, expected);
    let raw = native(&bytes, "A\u{2065}\u{301}B", LTR);
    assert_eq!(raw.residual, [1]);
    assert!(raw.deleted.is_empty());
    assert_eq!(
        raw.glyphs
            .iter()
            .map(|g| (g.0, g.2, g.3, g.4))
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|g| (g.0, g.2, g.3, g.4))
            .collect::<Vec<_>>()
    );
    let mut source = Source::new("A\u{2065}\u{301}B");
    source.span(Range::new(1, 3), own(-4.0));
    let mut system = system(&bytes);
    let prepared = prepare(
        &mut system,
        input(&source, own(4.0), LTR),
        LineOrientation::Horizontal,
    );
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [
            Range::new(0, 1),
            Range::new(1, 2),
            Range::new(2, 3),
            Range::new(3, 4)
        ]
    );
    let (selected, calls) = select_counted(&prepared, LTR);
    assert_eq!(exclusion_rows(&selected.inner.basis.shaped), [(1, false)]);
    let graph = &selected.inner.basis.characters;
    assert_eq!(graph.units.len(), 2);
    assert_eq!(
        graph.units[0].atoms,
        std::iter::once(0..3).collect::<Vec<_>>()
    );
    assert_eq!(graph.units[0].base, Some(0));
    assert_eq!(
        graph.units[1].atoms,
        std::iter::once(3..4).collect::<Vec<_>>()
    );
    assert_covered(&selected, 3..4, true);
    let line = finalize_retained(&selected, calls);
    assert_eq!(line.feedback().advance, 24.0);
    assert_native_packets(&line, &bytes, "A\u{2065}\u{301}B", &raw);
    assert_eq!(
        packets(&line, &bytes)
            .iter()
            .map(|p| (p.1, p.2, p.3, p.4))
            .collect::<Vec<_>>(),
        [
            (35, 0.0, 0.0, 10.0),
            (100, -7.0, -4.0, 0.0),
            (36, 0.0, 0.0, 10.0)
        ]
    );
    assert_eq!(inline_paints(&line), [0.0, 10.0, 14.0]);
    let painted = line
        .run_views()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .map(|glyph| {
                    (
                        origin.x + glyph.pen() + f64::from(glyph.x()),
                        origin.y - line.feedback().baseline + f64::from(glyph.y()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(painted, [(0.0, 0.0), (3.0, -4.0), (14.0, 0.0)]);
    source_copy(&prepared, line, &source);
}

#[test]
fn selective_deletion_with_normal_zwj_covers_the_endpoint_beyond_the_joined_emoji_unit() {
    let bytes = with_cmap(
        &font::fixture(None, font::Action::PartialDelete),
        &[
            (0x1f469, 35),
            (0x2065, 277),
            (0x200d, 100),
            (0x1f468, 36),
            (0x58, 37),
        ],
    );
    packet_metrics(
        &bytes,
        &[('👩', 35, 1000.0), ('👨', 36, 2000.0), ('X', 37, 1000.0)],
    );
    for scalar in ['👩', '👨'] {
        assert!(CodePointSetData::new::<ExtendedPictographic>().contains(scalar));
    }
    assert_eq!(
        CodePointMapData::<GraphemeClusterBreak>::new().get('\u{200d}'),
        GraphemeClusterBreak::ZWJ
    );
    for scalar in ['\u{2065}', '\u{200d}'] {
        assert!(CodePointSetData::new::<DefaultIgnorableCodePoint>().contains(scalar));
    }
    let segmenter = icu_segmenter::GraphemeClusterSegmenter::new();
    assert_eq!(
        segmenter
            .segment_str("👩\u{2065}\u{200d}👨X")
            .collect::<Vec<_>>(),
        [0, 4, 7, 10, 14, 15]
    );
    assert_eq!(
        segmenter.segment_str("👩\u{200d}👨X").collect::<Vec<_>>(),
        [0, 11, 12]
    );
    assert_eq!(
        segmenter.segment_str("👩👨X").collect::<Vec<_>>(),
        [0, 4, 8, 9]
    );
    let native = native(&bytes, "👩\u{2065}\u{200d}👨X", LTR);
    assert!(native.residual.is_empty());
    assert_eq!(native.deleted, [1]);
    assert_eq!(
        native
            .glyphs
            .iter()
            .map(|g| (g.0, g.2, g.3, g.4))
            .collect::<Vec<_>>(),
        [
            (35, 1000, 0, 0),
            (58, 1500, 0, 0),
            (36, 2000, 0, 0),
            (37, 1000, 0, 0)
        ]
    );
    let mut system = system(&bytes);
    // A genuine independent no-DI source is the all-removed contrast, never a
    // substitute for inspecting the actual partial native receipt above.
    let mut removed = Source::new("👩👨X");
    removed.span(Range::new(2, 4), own(-4.0));
    let contrast = prepare(
        &mut system,
        input(&removed, own(4.0), LTR),
        LineOrientation::Horizontal,
    );
    let (contrast_selected, contrast_calls) = select_counted(&contrast, LTR);
    no_optional_controls(&contrast_selected);
    let contrast_line = finalize_retained(&contrast_selected, contrast_calls);
    assert_eq!(contrast_line.feedback().advance, 40.0);
    assert_eq!(
        packets(&contrast_line, &bytes)
            .iter()
            .map(|p| (p.1, p.4))
            .collect::<Vec<_>>(),
        [(35, 10.0), (36, 20.0), (37, 10.0)]
    );
    source_copy(&contrast, contrast_line, &removed);
    let mut source = Source::new("👩\u{2065}\u{200d}👨X");
    source.span(Range::new(2, 6), own(-4.0));
    let prepared = prepare(
        &mut system,
        input(&source, own(4.0), LTR),
        LineOrientation::Horizontal,
    );
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [
            Range::new(0, 2),
            Range::new(2, 3),
            Range::new(3, 4),
            Range::new(4, 6),
            Range::new(6, 7)
        ]
    );
    let (selected, calls) = select_counted(&prepared, LTR);
    assert_eq!(exclusion_rows(&selected.inner.basis.shaped), [(1, true)]);
    let graph = &selected.inner.basis.characters;
    assert_eq!(graph.units.len(), 2);
    assert_eq!(
        graph.units[0].atoms,
        std::iter::once(0..4).collect::<Vec<_>>()
    );
    assert_eq!(graph.units[0].base, Some(0));
    assert_eq!(
        graph.units[1].atoms,
        std::iter::once(4..5).collect::<Vec<_>>()
    );
    assert_covered(&selected, 4..5, true);
    let line = finalize_retained(&selected, calls);
    assert_eq!(line.feedback().advance, 59.0);
    assert_native_packets(&line, &bytes, "👩\u{2065}\u{200d}👨X", &native);
    assert_eq!(
        packets(&line, &bytes)
            .iter()
            .map(|p| (p.1, p.2, p.3, p.4))
            .collect::<Vec<_>>(),
        [
            (35, 0.0, 0.0, 10.0),
            (58, 0.0, 0.0, 15.0),
            (36, 0.0, 0.0, 20.0),
            (37, 0.0, 0.0, 10.0)
        ]
    );
    assert_eq!(inline_paints(&line), [0.0, 10.0, 25.0, 49.0]);
    source_copy(&prepared, line, &source);
}

#[test]
fn source_neutral_and_non_governing_mark_controls_keep_ordinary_optional_output() {
    let optional = conditional_skip_font(*b"liga");
    native_conditional_skip_control(&optional, *b"liga");
    let mut system = system(&optional);
    let mut no_di = Source::new("AB");
    no_di.span(Range::new(1, 2), own(-4.0));
    let zero_di = conditional_tracking_source(0.0);
    for (source, tracking, expected) in [(&no_di, 4.0, 15.0), (&zero_di, 0.0, 15.0)] {
        let prepared = prepare(
            &mut system,
            input(source, own(tracking), LTR),
            LineOrientation::Horizontal,
        );
        let (selected, calls) = select_counted(&prepared, LTR);
        no_optional_controls(&selected);
        let line = finalize_retained(&selected, calls);
        assert_eq!(line.feedback().advance, expected);
        assert_eq!(
            packets(&line, &optional)
                .iter()
                .map(|p| (p.1, p.4))
                .collect::<Vec<_>>(),
            [(58, 15.0)]
        );
        source_copy(&prepared, line, source);
    }
    // Four fixed ordinary governors are stronger than the lone-unit no-gap
    // control: the selector cannot split its original A+selector EGC or take
    // over its base's full value. Nonadjacent A4/A4 are not visual neighbors.
    let mut gsub = crate::test_fonts::table(&optional, *b"GSUB").to_vec();
    let list = usize::from(crate::test_fonts::u16_at(&gsub, 8));
    assert_eq!(crate::test_fonts::u16_at(&gsub, list), 1);
    let lookup = list + usize::from(crate::test_fonts::u16_at(&gsub, list + 2));
    assert_eq!(crate::test_fonts::u16_at(&gsub, lookup), 4);
    assert_eq!(crate::test_fonts::u16_at(&gsub, lookup + 2), 0);
    // The only table change is a genuine IgnoreMarks lookup flag. Native Mn
    // U180F otherwise blocks matching; no synthetic eligibility is supplied.
    gsub[lookup + 2..lookup + 4].copy_from_slice(&0x0008_u16.to_be_bytes());
    let stable = crate::test_fonts::replace_tables(&optional, vec![(*b"GSUB", gsub)]);
    assert!(
        skrifa::FontRef::new(&stable)
            .unwrap()
            .charmap()
            .map('\u{180f}')
            .is_none()
    );
    let ordinary_native = native(&stable, "ABAB", LTR);
    assert_eq!(
        ordinary_native
            .glyphs
            .iter()
            .map(|glyph| (glyph.0, glyph.2))
            .collect::<Vec<_>>(),
        [(58, 1500), (58, 1500)]
    );
    let stable_native = native(&stable, "A\u{180f}BAB", LTR);
    assert_eq!(stable_native.residual, [1]);
    assert!(stable_native.deleted.is_empty());
    assert_eq!(stable_native.ligatures, [vec![0, 2], vec![3, 4]]);
    // A single provider call above is a context contrast, not Text's topology.
    // Raw Script is Latin/Mongolian/Latin, so Text itemizes A / selector / BAB
    // even though A+selector remains one original EGC. Tracking is deliberately
    // absent from same_shaping_style and cannot cause these three invocations.
    let scripts = CodePointMapData::<Script>::new();
    assert_eq!(scripts.get('A'), Script::Latin);
    assert_eq!(scripts.get('B'), Script::Latin);
    assert_eq!(scripts.get('\u{180f}'), Script::Mongolian);
    let first = native_in_context(
        &stable,
        "A",
        LTR,
        Some((surgeist_shaper::script::LATIN, "", "\u{180f}BAB", false)),
    );
    let selector = native_in_context(
        &stable,
        "\u{180f}",
        LTR,
        Some((surgeist_shaper::script::MONGOLIAN, "A", "BAB", true)),
    );
    let last = native_in_context(
        &stable,
        "BAB",
        LTR,
        Some((surgeist_shaper::script::LATIN, "A\u{180f}", "", false)),
    );
    assert_eq!(first.glyphs, [(35, 0, 1000, 0, 0)]);
    assert!(first.residual.is_empty() && first.deleted.is_empty());
    assert!(selector.glyphs.is_empty());
    assert_eq!(selector.residual, [0]);
    assert!(selector.deleted.is_empty());
    assert_eq!(last.glyphs, [(36, 0, 2000, 0, 0), (58, 1, 1500, 0, 0)]);
    assert!(last.residual.is_empty() && last.deleted.is_empty());
    assert_eq!(last.ligatures, [vec![1, 2]]);
    // Compose only independently executed native receipts, mapping each actual
    // invocation's local scalar ordinal back to the unchanged global source.
    let scoped_native = join_native([(0, first), (1, selector), (2, last)]);
    let mut stable_source = Source::new("A\u{180f}BAB");
    stable_source.span(Range::new(1, 2), own(20.0));
    stable_source.span(Range::new(2, 3), own(-4.0));
    stable_source.span(Range::new(4, 5), own(-4.0));
    let mut zero_governors = Source::new("A\u{180f}BAB");
    zero_governors.span(Range::new(1, 2), own(20.0));
    let mut stable_system = super::system(&stable);
    let mut ordinary_packets = None;
    for (source, tracking, governors) in [
        (&zero_governors, 0.0, [0.0; 4]),
        (&stable_source, 4.0, [4.0, -4.0, 4.0, -4.0]),
    ] {
        let stable_prepared = prepare(
            &mut stable_system,
            input(source, own(tracking), LTR),
            LineOrientation::Horizontal,
        );
        assert_eq!(
            stable_prepared
                .analyzed_source()
                .graphemes()
                .collect::<Vec<_>>(),
            [
                Range::new(0, 2),
                Range::new(2, 3),
                Range::new(3, 4),
                Range::new(4, 5)
            ],
        );
        assert_eq!(
            stable_prepared
                .inner
                .analyzed
                .analysis
                .infos
                .iter()
                .map(|(info, _)| info.script)
                .collect::<Vec<_>>(),
            [
                Script::Latin,
                Script::Mongolian,
                Script::Latin,
                Script::Latin,
                Script::Latin
            ],
        );
        assert!(
            stable_prepared
                .inner
                .styles
                .iter()
                .all(|row| same_shaping_style(&stable_prepared.inner.styles[0].style, &row.style))
        );
        let (stable_selected, stable_calls) = select_counted(&stable_prepared, LTR);
        let shaped = &stable_selected.inner.basis.shaped;
        assert_eq!(
            shaped
                .runs
                .iter()
                .map(|run| (run.atoms.clone(), run.script, run.level))
                .collect::<Vec<_>>(),
            [
                (0..1, Script::Latin, 0),
                (1..2, Script::Mongolian, 0),
                (2..5, Script::Latin, 0)
            ],
        );
        assert!(
            shaped
                .runs
                .iter()
                .all(|run| run.font.bytes() == stable.as_slice())
        );
        assert_eq!(
            shaped
                .character_invocations
                .iter()
                .map(|run| (run.atoms.clone(), run.run))
                .collect::<Vec<_>>(),
            [(0..1, 0), (1..2, 1), (2..5, 2)],
        );
        assert_eq!(exclusion_rows(shaped), [(1, false)]);
        let graph = &stable_selected.inner.basis.characters;
        assert_eq!(graph.units.len(), 4);
        assert_eq!(
            graph.units.iter().map(|unit| unit.base).collect::<Vec<_>>(),
            [Some(0), Some(2), Some(3), Some(4)]
        );
        for (unit, governor) in graph.units.iter().zip(governors) {
            assert_eq!((unit.left, unit.right), (governor, governor));
        }
        assert_eq!(graph.boundaries.len(), 3);
        assert!(
            graph
                .boundaries
                .iter()
                .all(|boundary| boundary.authored == 0.0)
        );
        no_optional_controls(&stable_selected);
        let stable_line = finalize_retained(&stable_selected, stable_calls);
        assert_native_packets(&stable_line, &stable, "A\u{180f}BAB", &scoped_native);
        // Exact native invocations give A10 + B20 + R15, with no outside or
        // internal canceled spacing. Compare to the same ordinary Text source,
        // retaining the nongoverning selector's20 in both authored controls.
        assert_eq!(stable_line.feedback().advance, 45.0);
        let actual = packets(&stable_line, &stable);
        if let Some(ordinary) = &ordinary_packets {
            assert_eq!(&actual, ordinary);
        } else {
            ordinary_packets = Some(actual);
        }
        source_copy(&stable_prepared, stable_line, source);
    }

    let bytes = font::fixture(Some('\u{180f}'), font::Action::PartialDelete);
    packet_metrics(
        &bytes,
        &[('A', 35, 1000.0), ('C', 100, 700.0), ('R', 58, 1500.0)],
    );
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert!(face.table_data(skrifa::Tag::new(b"GDEF")).is_none());
    assert!(face.table_data(skrifa::Tag::new(b"GPOS")).is_none());
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get('\u{180f}'),
        GeneralCategory::NonspacingMark
    );
    assert!(CodePointSetData::new::<DefaultIgnorableCodePoint>().contains('\u{180f}'));
    // The same real D->R,Q then Q deletion produces a15px R for literal C.
    // Mn U180F instead carries the native synthesized Mark class through GSUB;
    // without a GDEF override, the default shaper zeros that descendant's
    // advance late in positioning. Normal custody is independent of this zero.
    let literal = native(&bytes, "C", LTR);
    assert_eq!(
        literal
            .glyphs
            .iter()
            .map(|glyph| (glyph.0, glyph.2))
            .collect::<Vec<_>>(),
        [(58, 1500)]
    );
    let native = native(&bytes, "A\u{180f}", LTR);
    assert!(native.residual.is_empty() && native.deleted.is_empty());
    assert_eq!(
        native.glyphs.iter().map(|g| (g.0, g.2)).collect::<Vec<_>>(),
        [(35, 1000), (58, 0)]
    );
    // Whole-EGC native shaping above is only a mark-class control. Text uses
    // the raw Latin/Mongolian script split, so qualify each real input body.
    let a = native_in_context(
        &bytes,
        "A",
        LTR,
        Some((surgeist_shaper::script::LATIN, "", "\u{180f}", false)),
    );
    let mark = native_in_context(
        &bytes,
        "\u{180f}",
        LTR,
        Some((surgeist_shaper::script::MONGOLIAN, "A", "", true)),
    );
    assert_eq!(a.glyphs, [(35, 0, 1000, 0, 0)]);
    // Late Mark zeroing subtracts the original1500 from x_offset, then sets
    // advance to0. Normal R still belongs to this invocation's one scalar.
    assert_eq!(mark.glyphs, [(58, 0, 0, -1500, 0)]);
    assert!(a.residual.is_empty() && a.deleted.is_empty());
    assert!(mark.residual.is_empty() && mark.deleted.is_empty());
    let scoped_native = join_native([(0, a), (1, mark)]);
    let mut source = Source::new("A\u{180f}");
    source.span(Range::new(1, 2), own(20.0));
    let mut system = super::system(&bytes);
    let prepared = prepare(
        &mut system,
        input(&source, own(0.0), LTR),
        LineOrientation::Horizontal,
    );
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        std::iter::once(Range::new(0, 2)).collect::<Vec<_>>()
    );
    assert!(exclusion_rows(&prepared.inner.shaped).is_empty());
    let (selected, calls) = select_counted(&prepared, LTR);
    let shaped = &selected.inner.basis.shaped;
    assert!(exclusion_rows(shaped).is_empty());
    assert_eq!(
        shaped
            .runs
            .iter()
            .map(|run| (run.atoms.clone(), run.script))
            .collect::<Vec<_>>(),
        [(0..1, Script::Latin), (1..2, Script::Mongolian)],
    );
    assert_eq!(
        shaped
            .character_invocations
            .iter()
            .map(|run| (run.atoms.clone(), run.run))
            .collect::<Vec<_>>(),
        [(0..1, 0), (1..2, 1)],
    );
    assert!(
        shaped
            .runs
            .iter()
            .all(|run| run.font.bytes() == bytes.as_slice())
    );
    assert_eq!(selected.inner.basis.characters.units.len(), 1);
    assert_eq!(selected.inner.basis.characters.units[0].base, Some(0));
    assert_eq!(
        selected.inner.basis.characters.units[0].atoms,
        std::iter::once(0..2).collect::<Vec<_>>()
    );
    assert!(selected.inner.basis.characters.boundaries.is_empty());
    no_optional_controls(&selected);
    let line = finalize_retained(&selected, calls);
    assert_eq!(line.feedback().advance, 10.0);
    assert_native_packets(&line, &bytes, "A\u{180f}", &scoped_native);
    assert_eq!(
        packets(&line, &bytes)
            .iter()
            .map(|p| (p.1, p.4))
            .collect::<Vec<_>>(),
        [(35, 10.0), (58, 0.0)]
    );
    source_copy(&prepared, line, &source);
}

#[test]
fn a_real_disconnection_keeps_an_unrelated_neutral_component_outside_conditional_controls() {
    let bytes = conditional_skip_font(*b"liga");
    native_conditional_skip_control(&bytes, *b"liga");
    let mut source = Source::new("A\u{2065}BAB");
    source.span(Range::new(1, 2), own(-4.0));
    source.span(Range::new(3, 4), own(-4.0));
    let mut value = input(&source, own(4.0), LTR);
    value.origins = [Range::new(0, 3), Range::new(3, 5)]
        .into_iter()
        .map(|range| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    value.connections = vec![crate::ParagraphConnection {
        at: 3,
        kind: crate::ParagraphConnectionKind::Disconnected,
    }];
    let mut system = system(&bytes);
    let prepared = prepare(&mut system, value, LineOrientation::Horizontal);
    let (selected, calls) = select_counted(&prepared, LTR);
    assert!(
        requirements(&selected)
            .coverage
            .iter()
            .all(|row| row.atoms.end <= 3 || row.atoms.start >= 5)
    );
    assert_covered(&selected, 0..1, true);
    assert_covered(&selected, 2..3, true);
    let line = finalize_retained(&selected, calls);
    assert_eq!(line.feedback().advance, 49.0);
    assert_eq!(
        packets(&line, &bytes),
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
            (Range::new(2, 3), 36, 0.0, 0.0, 20.0),
            (Range::new(3, 5), 58, 0.0, 0.0, 15.0)
        ]
    );
    assert_eq!(inline_paints(&line), [0.0, 14.0, 34.0]);
    assert_eq!(
        prepared
            .origin_views()
            .map(|origin| origin.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(3, 5)]
    );
    source_copy(&prepared, line, &source);
}

#[test]
fn rtl_actual_neighbor_endpoints_have_upfront_coverage_and_reversed_source_custody() {
    let bytes = with_cmap(
        &font::fixture(None, font::Action::None),
        &[(0x5d0, 35), (0x5d1, 36), (0x2065, 100)],
    );
    packet_metrics(&bytes, &[('א', 35, 1000.0), ('ב', 36, 2000.0)]);
    let raw = native(&bytes, "א\u{2065}ב", RTL);
    assert_eq!(raw.residual, [1]);
    assert!(raw.deleted.is_empty());
    assert_eq!(
        raw.glyphs.iter().map(|g| (g.0, g.2)).collect::<Vec<_>>(),
        [(36, 2000), (35, 1000)]
    );
    let mut source = Source::new("א\u{2065}ב");
    source.span(Range::new(1, 2), own(-4.0));
    let mut system = system(&bytes);
    let prepared = prepare(
        &mut system,
        input(&source, own(4.0), RTL),
        LineOrientation::Horizontal,
    );
    let (selected, calls) = select_counted(&prepared, RTL);
    let graph = &selected.inner.basis.characters;
    assert_eq!(graph.boundaries.len(), 1);
    let edge = &graph.boundaries[0];
    assert_eq!(
        graph.units[edge.before].atoms,
        std::iter::once(2..3).collect::<Vec<_>>()
    );
    assert_eq!(
        graph.units[edge.after].atoms,
        std::iter::once(0..1).collect::<Vec<_>>()
    );
    assert_covered(&selected, 0..1, true);
    assert_covered(&selected, 2..3, true);
    let line = finalize_retained(&selected, calls);
    assert_eq!(line.feedback().advance, 34.0);
    assert_native_packets(&line, &bytes, "א\u{2065}ב", &raw);
    assert_eq!(
        packets(&line, &bytes)
            .iter()
            .map(|p| (p.0, p.1, p.4))
            .collect::<Vec<_>>(),
        [(Range::new(2, 3), 36, 20.0), (Range::new(0, 1), 35, 10.0)]
    );
    assert_eq!(inline_paints(&line), [0.0, 24.0]);
    source_copy(&prepared, line, &source);
}

#[path = "context_dependency.rs"]
mod context_dependency;
