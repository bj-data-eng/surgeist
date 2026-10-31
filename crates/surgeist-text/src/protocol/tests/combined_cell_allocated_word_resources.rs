//! Exact outer word-arena witnesses remain distinct from real inner Cell
//! positions. Expected storage is actual capacity/type size and row length;
//! public selected/finalized census checks remain aggregate observations.

use super::*;
use crate::engine::layout::data::CellPayload;
use crate::engine::scope_alignment::{
    census::{AllocationIdentity, AllocationWitness as W},
    visit_prepared_allocations,
};
use std::mem::{size_of, size_of_val};

struct Expected {
    identity: AllocationIdentity,
    records: usize,
    bytes: usize,
}
fn backing<T>(values: &Vec<T>) -> Expected {
    Expected {
        identity: W::buffer(values, 0).unwrap().identity(),
        records: values.len(),
        bytes: values.capacity() * size_of::<T>(),
    }
}

fn prepared_inner(mixed: &Mixed) -> AllocationIdentity {
    let data = &mixed.prepared.inner;
    let [cell] = data.shaped.cells.as_slice() else {
        panic!("one actual inner payload");
    };
    let CellPayload::LaidOut {
        layout,
        provider: Some(_),
    } = &cell.payload
    else {
        panic!("real A SPACE A provider input");
    };
    let positions = &layout.positions;
    assert_eq!(positions.len(), 3);
    let expected = backing(positions);
    let mut seen = 0;
    visit_prepared_allocations(
        &data.scope_alignment,
        &data.analyzed,
        &data.shaped,
        &data.tables,
        &mut |actual| {
            if actual.identity() == expected.identity {
                seen += 1;
                assert_eq!(actual.records(), expected.records);
                assert_eq!(actual.payload_bytes(), expected.bytes);
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(seen, 1);
    expected.identity
}

fn outer_words(selected: &SelectedLine, prepared_positions: AllocationIdentity) -> (usize, usize) {
    let shaped = &selected.inner.basis.shaped;
    let [cell] = shaped.cells.as_slice() else {
        panic!("one selected Cell");
    };
    let CellPayload::LaidOut {
        layout,
        provider: Some(_),
    } = &cell.payload
    else {
        panic!("retained inner provider packet");
    };
    let positions = &layout.positions;
    let clusters = &layout.clusters;
    assert_eq!(positions.len(), 3);
    let selected_positions = backing(positions).identity;
    assert_ne!(selected_positions, prepared_positions); // Real separately retained Vec copies.
    let rows = &shaped.word_spacing;
    assert_eq!(
        rows.len(),
        2,
        "only actual exterior SPACE units own outer word receipts"
    );
    let mut expected = vec![
        Expected {
            identity: W::object(rows, 0).identity(),
            records: 0,
            bytes: size_of_val(rows),
        },
        backing(rows),
    ];
    for row in rows {
        assert_eq!(row.components.len(), 1); // Exact real one-glyph SPACE qualified by parent.
        assert_eq!(row.authored, 0.0); // Inner WS4 belongs to the Cell payload, not these rows.
        assert_eq!(row.allocation, 0.0);
        for component in &row.components {
            assert!(!clusters.contains(&component.cluster));
        }
        expected.push(backing(&row.components));
    }
    for (index, allocation) in expected.iter().enumerate() {
        assert!(
            expected[..index]
                .iter()
                .all(|prior| prior.identity != allocation.identity)
        );
    }
    let mut seen = vec![0; expected.len()];
    crate::protocol::selection::spacing::visit_word_allocations(shaped, &mut |actual| {
        assert_ne!(actual.identity(), prepared_positions);
        assert_ne!(actual.identity(), selected_positions);
        let index = expected.iter().position(|expected| expected.identity == actual.identity())
            .expect("outer word visitor emits exactly its field/backings, not inner Cell positions or nested headers");
        seen[index] += 1;
        assert_eq!(actual.records(), expected[index].records);
        assert_eq!(actual.payload_bytes(), expected[index].bytes);
        Ok(())
    }).unwrap();
    assert!(seen.iter().all(|&count| count == 1));
    (
        expected.iter().map(|row| row.records).sum(),
        expected.iter().map(|row| row.bytes).sum(),
    )
}

#[test]
fn exterior_word_receipt_backings_are_counted_once_separately_from_the_retained_inner_packet() {
    let mixed = mixed_with_cell("B A A B", Range::new(2, 5), 4.0, 784_988_749);
    let space = exterior_space_metrics(mixed.parent.bytes());
    partial_cell_width_premises(mixed.body.bytes());
    let none = mixed
        .prepared
        .select(request(&mixed.prepared))
        .unwrap()
        .finalize(allocations())
        .unwrap();
    flow(&mixed, &none, space, SpacingPolicy::None, [0.0, 0.0]);
    let original = raw(&mixed, &none);
    let inner = prepared_inner(&mixed);
    let mut value = request(&mixed.prepared);
    value.context.spacing = SpacingPolicy::InterWord;
    let selected = mixed.prepared.select(value).unwrap();
    targets(&mixed, &selected);
    let (records, bytes) = outer_words(&selected, inner);
    let prepared_retention = mixed.prepared.scope_alignment_retention().unwrap();
    let selected_retention = selected.scope_alignment_retention().unwrap();
    // Necessary aggregate lower bounds; exact attribution above uses real owner helpers.
    assert!(selected_retention.records() >= prepared_retention.records() + records);
    assert!(selected_retention.payload_bytes() >= prepared_retention.payload_bytes() + bytes);
    let limits = mixed
        .processed
        .captured_scope(0)
        .unwrap()
        .scope_alignment_limits()
        .unwrap();
    let allowance = ScopeAlignmentAllowance::new(limits.records(), limits.payload_bytes());
    let alias = selected.clone();
    let related = [
        ScopeAlignmentProduct::Prepared(&mixed.prepared),
        ScopeAlignmentProduct::Selected(&alias),
    ];
    let input = ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Selected(&selected));
    let union =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&related), allowance).unwrap();
    assert_eq!(union.input_retention(), selected_retention);
    assert_eq!(union.external_retention().records(), 0);
    assert_eq!(union.external_retention().payload_bytes(), 0);
    let line = selected
        .finalize(LineAllocations {
            spacing: complete(&selected, [1.0, 3.0]),
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    flow(&mixed, &line, space, SpacingPolicy::InterWord, [1.0, 3.0]);
    assert_eq!(raw(&mixed, &line), original);
    let replay = line.with_spacing(complete(&selected, [2.0, 0.5])).unwrap();
    flow(&mixed, &replay, space, SpacingPolicy::InterWord, [2.0, 0.5]);
    assert_eq!(raw(&mixed, &replay), original);
    assert_eq!(outer_words(&selected, inner), (records, bytes));
    let input = ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&replay));
    let union =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&related), allowance).unwrap();
    assert_eq!(
        union.input_retention(),
        replay.scope_alignment_retention().unwrap()
    );
    assert_eq!(union.external_retention().records(), 0);
    assert_eq!(union.external_retention().payload_bytes(), 0);
}
