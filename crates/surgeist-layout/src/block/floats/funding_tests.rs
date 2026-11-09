use super::*;
use crate::text_layout::scope_alignment::{
    ScopeAlignmentFacts, ScopeAlignmentResources, ScopeAlignmentSnapshot,
};

const LIMITS: surgeist_text::ScopeAlignmentLimits =
    surgeist_text::ScopeAlignmentLimits::new(100_000, 100, 1_000_000, 16 << 20);

fn owner(binding: &crate::TextParagraphBinding<u32>) -> ScopeAlignmentResources<f64> {
    ScopeAlignmentResources::new(
        ScopeAlignmentSnapshot::admit(vec![], std::slice::from_ref(binding), false)
            .unwrap()
            .freeze(),
        Default::default(),
    )
}

fn pending(node: u32) -> PendingFloat<u32, f64> {
    PendingFloat {
        placement: PendingPlacement::Unresolved,
        node,
        source_index: usize::try_from(node).unwrap(),
        side: Float::Left,
        clear: Clear::None,
        block_start: 0.0,
        size: Size::splat(10.0),
        content_size: Size::splat(10.0),
        named_baselines: crate::NamedBaselinesOf::absent(),
        border: Edges::ZERO,
        padding: Edges::ZERO,
        margin: Edges::ZERO,
        float_exclusion: FloatExclusion::MarginBox,
        child_compute_geometry: None,
    }
}

fn remaining(funding: &EntryFunding<f64>) -> usize {
    let mut count = 0;
    while funding.visit(1).is_ok() {
        count += 1;
    }
    count
}

#[test]
fn funded_ledger_growth_replay_and_drop_keep_cumulative_work() {
    let capture = crate::test_support::inline_endpoint::Fixture::with_scope_limits("X", LIMITS);
    let control = owner(&capture.binding);
    let spent = owner(&capture.binding);
    let control_funding = EntryFunding::new(control, capture.binding.prepared.clone())
        .unwrap()
        .unwrap();
    let funding = EntryFunding::new(spent.clone(), capture.binding.prepared.clone())
        .unwrap()
        .unwrap();
    let initial = spent.retention();
    let spent_remaining = {
        let mut ledger = FloatExclusions::new(
            crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::splat(40.0),
            40.0,
            LogicalEdgesOf::new(0.0, 0.0, 0.0, 0.0),
        );
        ledger.fund_entry(Some(&funding));
        for node in 0..8 {
            let placement = ledger
                .place_float_for_line::<core::convert::Infallible>(100, &pending(node), None)
                .unwrap();
            // Four ten-pixel floats fit each forty-pixel row.
            assert_eq!(
                placement.logical,
                LogicalPointOf::new(f64::from(node % 4) * 10.0, f64::from(node / 4) * 10.0)
            );
        }
        let mut replay = ledger.try_clone().unwrap();
        let placement = replay
            .place_float_for_line::<core::convert::Infallible>(100, &pending(8), None)
            .unwrap();
        assert_eq!(placement.logical, LogicalPointOf::new(0.0, 20.0));
        assert_eq!(ledger.len(), 8);
        assert_eq!(replay.len(), 9);
        assert_ne!(spent.retention(), initial);
        let remaining = remaining(&funding);
        let live = spent.retention();
        let Err(error) = ledger.try_clone() else {
            panic!("replay copy must refuse spent work");
        };
        assert!(matches!(
            error.detail(),
            Some(surgeist_text::ErrorDetail::ScopeAlignment {
                reason: surgeist_text::ScopeAlignmentError::Limit {
                    kind: surgeist_text::ScopeAlignmentLimitKind::StructuralWork,
                    ..
                }
            })
        ));
        assert_eq!(
            spent.retention(),
            live,
            "failed copy retains no scratch backing"
        );
        remaining
    };
    assert_eq!(
        spent.retention(),
        initial,
        "discard releases actual live ledger backing"
    );
    assert!(
        spent_remaining < remaining(&control_funding),
        "discard does not refund executed placement or replay work"
    );
}
