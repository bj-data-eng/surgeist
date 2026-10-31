//! Prepared width ownership and replay over genuine captured AA/AAAA cells.
//! Expected allocation sizes are live capacities and Rust sizes, not visitor
//! totals. This unregistered child reuses its parent's real source/font factory.

use super::*;
use crate::engine::layout::data::CellPayload;
use crate::engine::scope_alignment::{census, visit_prepared_allocations};
use std::mem::{size_of, size_of_val};

struct ExpectedAllocation {
    name: &'static str,
    identity: census::AllocationIdentity,
    records: usize,
    bytes: usize,
    retained: bool,
}

fn backing<T>(name: &'static str, values: &Vec<T>, retained: bool) -> ExpectedAllocation {
    ExpectedAllocation {
        name,
        // Use the constructor only for its private identity, never its totals.
        identity: census::AllocationWitness::buffer(values, 0)
            .unwrap()
            .identity(),
        records: values.len(),
        bytes: values.capacity() * size_of::<T>(),
        retained,
    }
}

fn header<T>(name: &'static str, value: &T, retained: bool) -> ExpectedAllocation {
    ExpectedAllocation {
        name,
        identity: census::AllocationWitness::object(value, 0).identity(),
        records: 0,
        bytes: size_of_val(value),
        retained,
    }
}

fn width_prepare(text: &str) -> PreparedText {
    // Keep the authentic factory's default features. In particular, AB has a
    // real unacted B operand; it is not an authored-off/no-trial substitute.
    let (mut system, processed, analyzed, body) = source_with_full_coverage(text);
    if text == "AB" {
        width_actions_cover_only_original_a(&body);
    }
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(prepared.analyzed_source().source(), processed.source());
    assert_eq!(
        prepared.analyzed_source().source().text(),
        &Text::from(text)
    );
    prepared
}

fn maximum(prepared: &PreparedText) -> ScopeAlignmentAllowance {
    let limits = prepared
        .analyzed_source()
        .processed_source()
        .captured_scope(0)
        .unwrap()
        .scope_alignment_limits()
        .unwrap();
    ScopeAlignmentAllowance::new(limits.records(), limits.payload_bytes())
}

fn payload_limit(error: Error) {
    match error.detail() {
        Some(ErrorDetail::ScopeAlignment {
            reason:
                ScopeAlignmentError::Limit {
                    kind: ScopeAlignmentLimitKind::PayloadBytes,
                    required,
                    limit,
                },
        }) => assert!(required > limit),
        detail => panic!("expected actual payload admission refusal, got {detail:?}"),
    }
}

// Return only an independently calculated retained subset, not a reconstruction
// of the owner's complete census. Capture/tables/other scope metadata are extra.
fn assert_prepared_backings(prepared: &PreparedText, owned: bool) -> usize {
    let data = &prepared.inner;
    let shaped = &data.shaped;
    let mut expected = vec![
        header("run vector", &shaped.runs, owned),
        backing("whole run backing", &shaped.runs, owned),
        header("cluster vector", &shaped.clusters, owned),
        backing("whole cluster backing", &shaped.clusters, owned),
        header("glyph vector", &shaped.glyphs, owned),
        backing("whole glyph backing", &shaped.glyphs, owned),
        header("provider-group vector", &shaped.provider_groups, owned),
        backing(
            "whole provider-group backing",
            &shaped.provider_groups,
            owned,
        ),
        header(
            "winning exclusion map",
            shaped.character_exclusions.as_ref(),
            owned,
        ),
        backing(
            "winning exclusion rows",
            &shaped.character_exclusions.rows,
            owned,
        ),
        header("outer item vector", &shaped.items, true),
        backing("outer item backing", &shaped.items, true),
        header("cell vector", &shaped.cells, true),
        backing("cell backing", &shaped.cells, true),
    ];
    for run in &shaped.runs {
        // The header is already inside the whole PreparedRun backing. Account
        // for the actual nested coordinate buffer, including zero capacity.
        expected.push(backing("run coordinates", &run.coords, owned));
    }
    for cell in &shaped.cells {
        if let CellPayload::LaidOut { layout, .. } = &cell.payload {
            expected.push(backing("winning cell positions", &layout.positions, true));
            if layout.tabs.capacity() != 0 {
                expected.push(backing("winning cell tabs", &layout.tabs, true));
            }
        }
    }
    assert!(!shaped.runs.is_empty());
    assert!(!shaped.clusters.is_empty());
    assert!(!shaped.glyphs.is_empty());
    let mut actual = Vec::new();
    visit_prepared_allocations(
        &data.scope_alignment,
        &data.analyzed,
        shaped,
        &data.tables,
        &mut |allocation| {
            actual.push(allocation);
            Ok(())
        },
    )
    .unwrap();
    for expected in &expected {
        let matching = actual
            .iter()
            .filter(|allocation| allocation.identity() == expected.identity)
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            usize::from(expected.retained),
            "{}",
            expected.name
        );
        for allocation in matching {
            assert_eq!(allocation.records(), expected.records, "{}", expected.name);
            assert_eq!(
                allocation.payload_bytes(),
                expected.bytes,
                "{}",
                expected.name
            );
        }
    }
    // Resource identity is actual shared backing, not equal bytes, metrics, face
    // ordinals or run count. Scope and body may pin the same registered blob.
    let mut resources: Vec<&[u8]> = Vec::new();
    for font in data
        .scope_alignment
        .metrics
        .iter()
        .map(|metric| &metric.font)
        .chain(shaped.runs.iter().filter(|_| owned).map(|run| &run.font))
    {
        let bytes = font.bytes();
        if resources
            .iter()
            .any(|other| other.as_ptr() == bytes.as_ptr() && other.len() == bytes.len())
        {
            continue;
        }
        resources.push(bytes);
        let identity = census::AllocationWitness::resource(bytes).identity();
        let mut present = false;
        for allocation in &actual {
            if allocation.identity() == identity {
                present = true;
                assert_eq!(allocation.records(), 0);
                assert_eq!(allocation.payload_bytes(), bytes.len());
            }
        }
        assert!(present, "the real registered resource must be retained");
    }
    expected
        .iter()
        .filter(|expected| expected.retained)
        .map(|expected| expected.bytes)
        .sum::<usize>()
        + resources.iter().map(|bytes| bytes.len()).sum::<usize>()
}

#[test]
fn actual_width_need_retains_complete_prepared_backings_even_when_normal_wins() {
    for (text, owned, glyphs) in [
        ("A", false, vec![(35, 10.0)]),
        ("AA", true, vec![(36, 5.0); 2]),
        ("AAAA", true, vec![(16, 3.0); 4]),
        ("AB", true, vec![(35, 10.0), (36, 5.0)]),
    ] {
        let prepared = width_prepare(text);
        // Independent font premises: A normal10 fits; AA chooses hwid5;
        // AAAA chooses/compresses qwid3. AB's original B is unacted, so all
        // width candidates lack coverage and the real normal15 packet wins.
        let line = finalize(&prepared);
        assert_eq!(
            line.run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            glyphs
        );
        one_em(&line);
        let minimum = assert_prepared_backings(&prepared, owned);
        assert!(
            prepared
                .scope_alignment_retention()
                .unwrap()
                .payload_bytes()
                >= minimum
        );
    }
}

#[test]
fn reconstructed_width_replay_deduplicates_live_ancestors_and_refuses_a_real_small_allowance() {
    let prepared = width_prepare("AAAA");
    let minimum = assert_prepared_backings(&prepared, true);
    let line = finalize(&prepared);
    let selected = line.selected_line();
    let allowance = maximum(&prepared);
    let replay = line
        .with_spacing_with_scope_allowance(SpacingAllocations::None, allowance)
        .unwrap();
    let input = ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&replay));
    let ancestors = [
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&selected),
    ];
    let shared =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&ancestors), allowance)
            .unwrap();
    assert_eq!(shared.external_retention().records(), 0);
    assert_eq!(shared.external_retention().payload_bytes(), 0);
    let old = [ScopeAlignmentProduct::Finalized(&line)];
    let both =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&old), allowance).unwrap();
    let old_glyphs = &line.inner.data.glyphs;
    assert!(!old_glyphs.is_empty());
    assert_ne!(old_glyphs.as_ptr(), replay.inner.data.glyphs.as_ptr());
    let actual_old_glyph_bytes =
        old_glyphs.capacity() * size_of::<crate::engine::layout::data::FinalizedGlyph>();
    assert!(both.external_retention().records() >= old_glyphs.len());
    assert!(both.external_retention().payload_bytes() >= actual_old_glyph_bytes);
    let clone = line.clone();
    let repeated = [
        ScopeAlignmentProduct::Finalized(&line),
        ScopeAlignmentProduct::Finalized(&clone),
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&selected),
    ];
    let dedup =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&repeated), allowance).unwrap();
    assert_eq!(dedup.input_retention(), both.input_retention());
    assert_eq!(dedup.external_retention(), both.external_retention());
    // This cap comes from actual owned capacities and unique real resource
    // backings, not a visitor/census total. Additional metadata only raises need.
    assert!(minimum > 0 && minimum <= allowance.payload_bytes());
    payload_limit(
        scope_alignment_census(
            input,
            ScopeAlignmentProducts::Mixed(&repeated),
            ScopeAlignmentAllowance::new(allowance.records(), minimum - 1),
        )
        .unwrap_err(),
    );
    // Failed admission neither invalidates the retained winner nor publishes a
    // replacement frame. The prior and replay still expose the qualified qwid.
    for usable in [&line, &replay] {
        one_em(usable);
        assert_eq!(usable.prepared_text(), prepared);
        assert_eq!(
            usable
                .run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![(16, 3.0); 4]
        );
    }
}
