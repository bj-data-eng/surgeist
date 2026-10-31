//! Real retained outer-edge storage belongs to the finalized Arc once. Expected
//! records/bytes are actual row lengths, capacities and Rust sizes; production
//! allocation witnesses supply identity only, never the expected totals.

use super::*;
use crate::engine::scope_alignment::census::AllocationWitness as W;
use crate::protocol::measurements::{EdgeRows, OuterEdge};
use std::mem::{size_of, size_of_val};

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

fn actual_outer_storage(line: &FinalizedLine) {
    let measurements = line.measurements();
    let EdgeRows::Outer(rows) = &measurements.edges else {
        panic!("genuine Cell line retains outer observations");
    };
    let [OuterEdge::Cell { cell: 0 }] = rows.as_slice() else {
        panic!("one sealed actual finalized Cell target");
    };
    assert_eq!(line.inner.data.cells.len(), 1);
    assert!(rows.capacity() > 0);
    let expected_bytes = rows.capacity() * size_of::<OuterEdge>();
    let expected_records = rows.len();
    let backing = W::buffer(rows, 0).unwrap().identity();
    let owner = W::object(line.inner.as_ref(), 0).identity();
    let embedded = [
        W::object(measurements, 0).identity(),
        W::object(&measurements.edges, 0).identity(),
        W::object(rows, 0).identity(),
    ];
    let mut buffers = 0;
    let mut headers = 0;
    line.inner
        .visit_scope_allocations(&mut |allocation| {
            if allocation.identity() == backing {
                buffers += 1;
                assert_eq!(allocation.records(), expected_records);
                assert_eq!(allocation.payload_bytes(), expected_bytes);
            }
            if allocation.identity() == owner {
                headers += 1;
                assert_eq!(allocation.records(), 0);
                assert_eq!(allocation.payload_bytes(), size_of_val(line.inner.as_ref()));
            }
            // These fixed headers are already embedded in that actual Arc object.
            assert!(
                !embedded.contains(&allocation.identity()),
                "embedded edge/measurement headers must not be charged twice"
            );
            Ok(())
        })
        .unwrap();
    assert_eq!(buffers, 1);
    assert_eq!(headers, 1);
}

fn edges(line: &FinalizedLine) -> Vec<(f64, Rect, Vec<Range>)> {
    line.edge_units()
        .map(|edge| {
            (
                edge.advance(),
                edge.bounds(),
                edge.source_pieces().collect(),
            )
        })
        .collect()
}

#[test]
fn retained_outer_edge_capacity_and_header_are_owned_once_for_visible_and_provider_empty_cells() {
    for (text, whitespace, provider) in [
        ("AA", WhiteSpace::Preserve, true),
        (" ", WhiteSpace::Collapse, false),
    ] {
        let font = qualified_font();
        let (processed, analyzed) = captured(text, whitespace, 0.0);
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(font).unwrap();
        let prepared = system
            .prepare(
                &analyzed,
                PreparationOptions::default().with_orientation(LineOrientation::Vertical),
            )
            .unwrap();
        assert_eq!(
            prepared.cell_views().next().unwrap().has_provider_input(),
            provider
        );
        let line = complete_line(&prepared);
        assert_eq!(line.cell_views().len(), 1);
        actual_outer_storage(&line);
        let before = edges(&line);
        assert_eq!(before.len(), 1);
        assert_eq!(before[0].0, 10.0);
        assert_eq!(
            before[0].2,
            (0..text.len())
                .map(|index| Range::new(index, index + 1))
                .collect::<Vec<_>>()
        );
        let allowance = maximum(&prepared);
        let retained = line.scope_alignment_retention().unwrap();
        assert!(retained.payload_bytes() > 0);
        let failure = line
            .with_spacing_with_scope_allowance(
                SpacingAllocations::None,
                ScopeAlignmentAllowance::new(allowance.records(), retained.payload_bytes() - 1),
            )
            .unwrap_err();
        assert!(
            matches!(failure.detail(), Some(ErrorDetail::ScopeAlignment {
            reason: ScopeAlignmentError::Limit { kind: ScopeAlignmentLimitKind::PayloadBytes, required, limit },
        }) if required > limit),
            "expected actual retained-input byte refusal: {failure:?}"
        );
        assert_eq!(edges(&line), before); // Existing output remains fully usable.
        actual_outer_storage(&line);
        drop(system);
        let replay = line
            .with_spacing_with_scope_allowance(SpacingAllocations::None, allowance)
            .unwrap();
        assert_eq!(edges(&replay), before);
        actual_outer_storage(&replay);
        assert_eq!(
            replay.prepared_text().analyzed_source().source(),
            processed.source()
        );
        // Repeated handles and the already-retained Prepared/Selected ancestors
        // do not manufacture another edge/header allocation in the live union.
        let selected = replay.selected_line();
        let clone = replay.clone();
        let related = [
            ScopeAlignmentProduct::Prepared(&prepared),
            ScopeAlignmentProduct::Selected(&selected),
            ScopeAlignmentProduct::Finalized(&clone),
        ];
        let union = scope_alignment_census(
            ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&replay)),
            ScopeAlignmentProducts::Mixed(&related),
            allowance,
        )
        .unwrap();
        assert_eq!(
            union.input_retention(),
            replay.scope_alignment_retention().unwrap()
        );
        assert_eq!(union.external_retention().records(), 0);
        assert_eq!(union.external_retention().payload_bytes(), 0);
    }
}
