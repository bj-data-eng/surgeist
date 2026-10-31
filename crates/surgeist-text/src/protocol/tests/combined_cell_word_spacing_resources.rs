//! Named resource subsets for an actual normal-fit NBSP word execution.
//! This child of the signed fixture uses its qualified glyph153 font. It does
//! not infer funding from width-arena ownership or observe transient trials.

use super::*;
use crate::engine::{
    layout::data::{
        CellArenaOwnership, CellClusterPosition, CellPayload, FinalizedRunFrame, ShapedData,
    },
    scope_alignment::{accounting, census, operation_limits, visit_prepared_allocations},
};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use std::{mem::size_of, sync::Arc};

struct WordFixture {
    prepared: PreparedText,
    processed: Arc<ProcessedSource>,
    font: Vec<u8>,
}

fn normal_fit() -> WordFixture {
    // The owning helper independently qualifies NBSP153, intrinsic1000,
    // blank outlines and Complete/NoAction for every width feature.
    let font = qualified_nbsp_font(false);
    let (processed, analyzed) = captured("\u{a0}", WhiteSpace::Preserve, -2.0);
    source_custody(&processed, &analyzed, &[0], -2.0);
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
    assert_eq!(
        prepared.inner.shaped.cell_arenas,
        CellArenaOwnership::Native
    );
    assert!(prepared.cell_views().next().unwrap().has_provider_input());
    assert_word(&prepared.inner.shaped);
    WordFixture {
        prepared,
        processed,
        font,
    }
}

fn positions(shaped: &ShapedData) -> &Vec<CellClusterPosition> {
    let [cell] = shaped.cells.as_slice() else {
        panic!("one genuine NBSP Cell");
    };
    match &cell.payload {
        CellPayload::Shaped { positions, .. } => positions,
        CellPayload::NoProviderInput => panic!("surviving NBSP is real provider input"),
    }
}

fn assert_word(shaped: &ShapedData) {
    let [cell] = shaped.cells.as_slice() else {
        panic!("one genuine NBSP Cell");
    };
    assert_eq!((cell.extent, cell.advance), (10.0, 10.0));
    let CellPayload::Shaped {
        clusters,
        glyphs,
        positions,
        advance,
        frame,
        ..
    } = &cell.payload
    else {
        panic!("real NBSP provider packet");
    };
    let [position] = positions.as_slice() else {
        panic!("one actual cluster position");
    };
    assert!(clusters.contains(&position.cluster));
    assert_eq!(
        (position.pen, position.word.half, position.word.unit_start),
        (0.0, -1.0, true)
    );
    assert_eq!((advance.start, advance.end), (-1.0, 9.0));
    assert_eq!((frame.x_scale, frame.x_offset), (1.0, 1.0));
    let cluster = &shaped.clusters[position.cluster];
    assert_eq!(cluster.atoms, 0..1);
    assert_eq!((cluster.shaped_advance, cluster.advance), (10.0, 8.0));
    assert_eq!(cluster.logical_adjustments[1], -2.0);
    assert_eq!(
        shaped.glyphs[glyphs.clone()]
            .iter()
            .map(|g| (g.id, g.advance))
            .collect::<Vec<_>>(),
        [(153, 10.0)]
    );
    // Temporary inner word receipts must not become an outer word arena.
    assert!(shaped.word_spacing.is_empty());
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
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    match error.detail() {
        Some(ErrorDetail::ScopeAlignment {
            reason:
                ScopeAlignmentError::Limit {
                    kind: ScopeAlignmentLimitKind::PayloadBytes,
                    required,
                    limit,
                },
        }) => assert!(required > limit),
        detail => panic!("expected actual payload refusal, got {detail:?}"),
    }
}

fn assert_backing<T>(values: &Vec<T>, actual: &[census::AllocationWitness]) -> usize {
    let identity = census::AllocationWitness::buffer(values, 0)
        .unwrap()
        .identity();
    // Only identity comes from the constructor. These are independent physical
    // capacity/row expectations, not a sum calculated by the production walker.
    let records = values.len();
    let bytes = values.capacity() * size_of::<T>();
    let matching = actual
        .iter()
        .filter(|row| row.identity() == identity)
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].records(), records);
    assert_eq!(matching[0].payload_bytes(), bytes);
    bytes
}

fn unchanged_output(fixture: &WordFixture, line: &FinalizedLine) {
    source_custody(
        &fixture.processed,
        &line.prepared_text().analyzed_source(),
        &[0],
        -2.0,
    );
    output(
        &fixture.prepared,
        line,
        &fixture.processed,
        &fixture.font,
        &Expected {
            word: -2.0,
            raw: 10.0,
            pen: -1.0,
            terminal: 8.0,
            occupied: (-1.0, 9.0),
            scale: 1.0,
            offset: 1.0,
            physical: 95.0,
        },
    );
}

#[test]
fn normal_fit_native_arenas_retain_actual_word_position_storage_and_selected_copy() {
    let fixture = normal_fit();
    let prepared = &fixture.prepared;
    let data = &prepared.inner;
    let mut actual = Vec::new();
    visit_prepared_allocations(
        &data.scope_alignment,
        &data.analyzed,
        &data.shaped,
        &data.tables,
        &mut |row| {
            actual.push(row);
            Ok(())
        },
    )
    .unwrap();
    let source = positions(&data.shaped);
    assert_eq!(source.len(), 1);
    let bytes = assert_backing(source, &actual);
    assert!(bytes >= size_of::<CellClusterPosition>());
    assert!(
        prepared
            .scope_alignment_retention()
            .unwrap()
            .payload_bytes()
            >= bytes
    );

    let line = complete_line(prepared);
    let selected = line.selected_line();
    assert_word(&selected.inner.basis.shaped);
    let selected_positions = positions(&selected.inner.basis.shaped);
    assert_ne!(source.as_ptr(), selected_positions.as_ptr());
    assert_eq!(
        data.shaped.cells[0].binding,
        selected.inner.basis.shaped.cells[0].binding
    );
    // This actual lone-cell selection copies/remaps through the production
    // owner. Its ordinal happens to remain equal; no changed-index claim.
    unchanged_output(&fixture, &line);
}

#[test]
fn actual_word_position_deep_copies_coexist_and_drop_back_to_the_real_input_seed() {
    let fixture = normal_fit();
    let prepared = &fixture.prepared;
    let source = &prepared.inner.shaped;
    let seed = prepared.scope_alignment_retention().unwrap();
    let limits = operation_limits(&prepared.inner, None).unwrap().unwrap();
    let mut ledger =
        Some(ScopeAlignmentBudget::new(limits, seed.records(), seed.payload_bytes()).unwrap());
    let first = accounting::clone_shaped(&mut ledger, source).unwrap();
    assert_word(&first);
    let first_bytes = positions(&first).capacity() * size_of::<CellClusterPosition>();
    let after_first = ledger.as_ref().unwrap().usage();
    assert!(after_first.payload_bytes() >= seed.payload_bytes() + first_bytes);
    let second = accounting::clone_shaped(&mut ledger, source).unwrap();
    assert_word(&second);
    let second_bytes = positions(&second).capacity() * size_of::<CellClusterPosition>();
    assert_ne!(positions(source).as_ptr(), positions(&first).as_ptr());
    assert_ne!(positions(source).as_ptr(), positions(&second).as_ptr());
    assert_ne!(positions(&first).as_ptr(), positions(&second).as_ptr());
    assert!(
        ledger.as_ref().unwrap().usage().payload_bytes()
            >= after_first.payload_bytes() + second_bytes
    );
    for copy in [&first, &second] {
        assert_eq!(copy.runs.len(), source.runs.len());
        for (copied, original) in copy.runs.iter().zip(&source.runs) {
            assert_eq!(copied.font.bytes().as_ptr(), original.font.bytes().as_ptr());
            assert_eq!(copied.font.bytes().len(), original.font.bytes().len());
        }
    }
    accounting::drop_shaped(&mut ledger, first).unwrap();
    assert!(
        ledger.as_ref().unwrap().usage().payload_bytes() >= seed.payload_bytes() + second_bytes
    );
    accounting::drop_shaped(&mut ledger, second).unwrap();
    let usage = ledger.unwrap().usage();
    assert_eq!(usage.payload_bytes(), seed.payload_bytes());
    assert!(usage.records() > seed.records()); // Historical materialization is not refunded.

    // Reduce only the actual captured payload allowance. This has no spare
    // room for a new clone owner; it is not a position-specific failure claim.
    let limits = operation_limits(
        &prepared.inner,
        Some(ScopeAlignmentAllowance::new(
            limits.records(),
            seed.payload_bytes(),
        )),
    )
    .unwrap()
    .unwrap();
    let mut tight =
        Some(ScopeAlignmentBudget::new(limits, seed.records(), seed.payload_bytes()).unwrap());
    match accounting::clone_shaped(&mut tight, source) {
        Err(error) => payload_limit(error),
        Ok(copy) => {
            accounting::drop_shaped(&mut tight, copy).unwrap();
            panic!("a new clone needs real payload room");
        }
    }
    assert_word(source);
}

#[test]
fn real_word_replay_deduplicates_shared_inputs_and_keeps_distinct_output_witnesses() {
    let fixture = normal_fit();
    let line = complete_line(&fixture.prepared);
    unchanged_output(&fixture, &line);
    let selected = line.selected_line();
    let allowance = maximum(&fixture.prepared);
    let replay = line
        .with_spacing_with_scope_allowance(SpacingAllocations::None, allowance)
        .unwrap();
    unchanged_output(&fixture, &replay);
    let mut old_rows = Vec::new();
    line.inner
        .data
        .visit_scope_allocations(&mut |row| {
            old_rows.push(row);
            Ok(())
        })
        .unwrap();
    let anchors = &line.inner.data.word_anchors;
    let [anchor] = anchors.as_slice() else {
        panic!("one actual blank-ink unit displacement");
    };
    assert!(matches!(anchor.frame, FinalizedRunFrame::Cell(0)));
    assert_eq!(
        (anchor.before, anchor.after, anchor.amount),
        (Point::new(0.0, 0.0), Point::new(-1.0, 0.0), -1.0)
    );
    let old_word_bytes = assert_backing(anchors, &old_rows);
    assert_ne!(anchors.as_ptr(), replay.inner.data.word_anchors.as_ptr());
    let input = ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&replay));
    let ancestors = [
        ScopeAlignmentProduct::Prepared(&fixture.prepared),
        ScopeAlignmentProduct::Selected(&selected),
    ];
    let shared =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&ancestors), allowance)
            .unwrap();
    assert_eq!(
        (
            shared.external_retention().records(),
            shared.external_retention().payload_bytes()
        ),
        (0, 0)
    );
    let old = [ScopeAlignmentProduct::Finalized(&line)];
    let both =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&old), allowance).unwrap();
    assert!(both.external_retention().records() >= anchors.len());
    assert!(both.external_retention().payload_bytes() >= old_word_bytes);
    let alias = line.clone();
    let repeated = [
        ScopeAlignmentProduct::Finalized(&line),
        ScopeAlignmentProduct::Finalized(&alias),
        ScopeAlignmentProduct::Prepared(&fixture.prepared),
        ScopeAlignmentProduct::Selected(&selected),
    ];
    let dedup =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&repeated), allowance).unwrap();
    assert_eq!(dedup.input_retention(), both.input_retention());
    assert_eq!(dedup.external_retention(), both.external_retention());

    let retained = line.scope_alignment_retention().unwrap();
    payload_limit(
        line.with_spacing_with_scope_allowance(
            SpacingAllocations::None,
            ScopeAlignmentAllowance::new(allowance.records(), retained.payload_bytes() - 1),
        )
        .unwrap_err(),
    );
    unchanged_output(&fixture, &line);
    unchanged_output(&fixture, &replay);
}
