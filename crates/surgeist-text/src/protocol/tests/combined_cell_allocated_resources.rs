//! Exact owner-visitor attribution and aggregate allowance/census qualification
//! over real exterior Cell slots. Expected sizes come independently from actual
//! capacities and Rust types; public census remains an aggregate observation.

use super::*;
use crate::engine::scope_alignment::census::{AllocationIdentity, AllocationWitness};
use std::{
    mem::{size_of, size_of_val},
    sync::Arc,
};

struct ExpectedAllocation {
    identity: AllocationIdentity,
    records: usize,
    bytes: usize,
}

#[derive(Default)]
struct Inventory {
    records: usize,
    bytes: usize,
    identities: Vec<AllocationIdentity>,
    allocations: Vec<ExpectedAllocation>,
}
impl Inventory {
    fn backing<T>(&mut self, values: &Vec<T>) {
        let records = values.len();
        let bytes = values.capacity() * size_of::<T>();
        self.records += records;
        self.bytes += bytes;
        self.allocations.push(ExpectedAllocation {
            identity: AllocationWitness::buffer(values, 0).unwrap().identity(),
            records,
            bytes,
        });
        if values.capacity() > 0 && size_of::<T>() > 0 {
            // Identity only: expected records/bytes are the real Vec operands.
            let identity = AllocationWitness::buffer(values, 0).unwrap().identity();
            assert!(
                !self.identities.contains(&identity),
                "separately owned slot backings remain distinct"
            );
            self.identities.push(identity);
        }
    }
}

fn slot_inventory(selected: &SelectedLine) -> Inventory {
    let rows = &selected.inner.spacing_slots;
    assert_eq!(rows.len(), 2);
    // One actual selected Vec field header, not a second SelectedData header.
    // Row/side Vec headers already live inside each SlotData backing element.
    let mut result = Inventory {
        bytes: size_of_val(rows),
        ..Inventory::default()
    };
    result.allocations.push(ExpectedAllocation {
        identity: AllocationWitness::object(rows, 0).identity(),
        records: 0,
        bytes: size_of_val(rows),
    });
    result.backing(rows);
    for row in rows {
        assert_eq!(row.participants.len(), 2);
        assert_eq!(row.pieces.len(), 2);
        result.backing(&row.participants);
        result.backing(&row.pieces);
        let crate::protocol::selection::SpacingTargetData::Boundary { units, .. } = &row.target
        else {
            panic!("two actual exterior character boundaries");
        };
        for side in units {
            assert_eq!(side.participants.len(), 1);
            result.backing(&side.participants);
        }
    }
    assert_eq!(result.records, 14); // 2 slots +4 union +4 source pieces +4 side memberships.
    assert!(result.bytes > size_of_val(rows));
    let mut seen = vec![0; result.allocations.len()];
    crate::protocol::selection::spacing::visit_allocations(rows, &mut |actual| {
        let index = result.allocations.iter().position(|expected| expected.identity == actual.identity())
            .expect("the owner visitor emits exactly the real slot field/backings, with no extra embedded header");
        let expected = &result.allocations[index];
        seen[index] += 1;
        assert_eq!(actual.records(), expected.records);
        assert_eq!(actual.payload_bytes(), expected.bytes);
        Ok(())
    }).unwrap();
    assert!(seen.iter().all(|&count| count == 1));
    result
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

fn payload_refusal(error: Error) {
    assert!(
        matches!(error.detail(), Some(ErrorDetail::ScopeAlignment {
        reason: ScopeAlignmentError::Limit { kind: ScopeAlignmentLimitKind::PayloadBytes, required, limit },
    }) if required > limit),
        "expected actual insufficient payload allowance: {error:?}"
    );
}

fn amounts(selected: &SelectedLine) -> SpacingAllocations {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    SpacingAllocations::Complete(
        slots
            .into_iter()
            .zip([2.0, 3.0])
            .map(|(slot, amount)| SpacingAllocation { slot, amount })
            .collect(),
    )
}

#[test]
fn actual_cell_slot_arenas_survive_aggregate_census_refusal_and_replay_without_ancestor_duplication()
 {
    let (mut system, processed, analyzed, font) = tracked_empty_cell(0.0);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let cell = prepared.cell_views().next().unwrap();
    assert_eq!(cell.extent(), 10.0);
    assert!(!cell.has_provider_input());
    let natural = finalize(&prepared);
    tracked_cell_line(&prepared, &natural, &processed, &font, 30.0, 10.0, 20.0);
    let selected = prepared
        .select(request(&prepared))
        .expect("the real exterior-slot path must be admitted before its resource qualification");
    let inventory = slot_inventory(&selected);
    let selected_retention = selected.scope_alignment_retention().unwrap();
    let input_retention = prepared.scope_alignment_retention().unwrap();
    // Necessary independent lower bounds, not per-buffer census attribution.
    assert!(selected_retention.records() >= input_retention.records() + inventory.records);
    assert!(
        selected_retention.payload_bytes() >= input_retention.payload_bytes() + inventory.bytes
    );
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    let issued_clone = slots[0].clone();
    assert!(Arc::ptr_eq(&issued_clone.selected, &selected.inner));
    assert_eq!(issued_clone, slots[0]);
    let alias = selected.clone();
    let allowance = maximum(&prepared);
    let ancestors = [
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&alias),
    ];
    let census = scope_alignment_census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Selected(&selected)),
        ScopeAlignmentProducts::Mixed(&ancestors),
        allowance,
    )
    .unwrap();
    assert_eq!(census.input_retention(), selected_retention);
    assert_eq!(census.external_retention().records(), 0);
    assert_eq!(census.external_retention().payload_bytes(), 0);
    let selection_cap = selected_retention.payload_bytes() - 1;
    assert!(selection_cap >= input_retention.payload_bytes());
    payload_refusal(
        prepared
            .select_with_scope_allowance(
                request(&prepared),
                ScopeAlignmentAllowance::new(allowance.records(), selection_cap),
            )
            .unwrap_err(),
    );
    assert_eq!(
        selected.scope_alignment_retention().unwrap(),
        selected_retention
    );
    assert_eq!(selected.spacing_slots().collect::<Vec<_>>(), slots);
    let recovered = prepared
        .select_with_scope_allowance(request(&prepared), allowance)
        .unwrap();
    assert_eq!(recovered.spacing_slots().count(), 2);
    let line = selected
        .finalize(LineAllocations {
            spacing: amounts(&selected),
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    tracked_cell_line(&prepared, &line, &processed, &font, 35.0, 12.0, 25.0);
    let retained = line.scope_alignment_retention().unwrap();
    payload_refusal(
        line.with_spacing_with_scope_allowance(
            amounts(&selected),
            ScopeAlignmentAllowance::new(allowance.records(), retained.payload_bytes()),
        )
        .unwrap_err(),
    );
    tracked_cell_line(&prepared, &line, &processed, &font, 35.0, 12.0, 25.0);
    assert_eq!(selected.spacing_slots().collect::<Vec<_>>(), slots);
    drop(system);
    let replay = line
        .with_spacing_with_scope_allowance(amounts(&selected), allowance)
        .unwrap();
    tracked_cell_line(&prepared, &replay, &processed, &font, 35.0, 12.0, 25.0);
    let shared = [
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&alias),
    ];
    let input = ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&replay));
    let shared_census =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&shared), allowance).unwrap();
    assert_eq!(
        shared_census.input_retention(),
        replay.scope_alignment_retention().unwrap()
    );
    assert_eq!(shared_census.external_retention().records(), 0);
    assert_eq!(shared_census.external_retention().payload_bytes(), 0);
    // Distinct immutable output arrays coexist, while their actual selected
    // SlotData owner (also retained by the issued clone) remains shared.
    let old_only = [ScopeAlignmentProduct::Finalized(&line)];
    let both =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&old_only), allowance).unwrap();
    assert!(
        both.external_retention().records() > 0 && both.external_retention().payload_bytes() > 0
    );
    let old_clone = line.clone();
    let duplicates = [
        ScopeAlignmentProduct::Finalized(&line),
        ScopeAlignmentProduct::Finalized(&old_clone),
        ScopeAlignmentProduct::Selected(&selected),
    ];
    let dedup =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&duplicates), allowance)
            .unwrap();
    assert_eq!(dedup.input_retention(), both.input_retention());
    assert_eq!(dedup.external_retention(), both.external_retention());
    assert!(Arc::ptr_eq(&issued_clone.selected, &replay.inner.selected));
}
