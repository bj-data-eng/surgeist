//! Current-trial frame extraction requires its actual unrounded custody.
use super::*;
use crate::test_support::readback::{self, Phase};

#[test]
fn actual_final_root_entry_cannot_replace_current_trial_measured_basis_custody() {
    // Reuse this owner's registered Ahem/captured scope fixture and
    // complete public result, separately from the new staged private session.
    let fixture = Fixture::new(WORK);
    let flow = fixture.flow();
    let old = compute_layout_with_flow(&flow, 0, request()).unwrap();
    publication(&old, &fixture);
    let old_entries = readback::continuous_custody(&old, Phase::Final);
    let old_fragments = old
        .final_inline_fragments()
        .iter()
        .map(|row| row.fragment().finalized_fragment().clone())
        .collect::<Vec<_>>();

    let mut session = ComputeSession::with_flow(&flow, Vec::new());
    stage(&mut session); // actual compute_root and round_layout, no fabricated entries
    let valid = session.canonical_frames(0).unwrap();
    let slot = session
        .outputs
        .unrounded
        .iter()
        .position(|entry| entry.node() == 0)
        .unwrap();
    let original_unrounded = session.outputs.unrounded[slot];
    let actual_final = *session
        .outputs
        .rounded
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap();
    assert_eq!(actual_final.node(), original_unrounded.node());
    // The exact final entry came from this SAME performed trial. Its geometry
    // is valid final publication; it is not valid unrounded measured custody.
    session.outputs.unrounded[slot] = actual_final;
    let outcome = session.canonical_frames(0);
    let Err(error) = outcome else {
        panic!("final publication cannot issue the current-trial measured inset basis");
    };
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::TextPlacement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidOutputGeometryPhase)
    ));

    // Restoring the SAME actual current-trial raw entry restores valid frame
    // extraction. This is a private staged-slot negative, not host warm return;
    // invalid host warm custody retains its existing cache-miss/recompute policy.
    session.outputs.unrounded[slot] = original_unrounded;
    let recovered = session.canonical_frames(0).unwrap();
    assert_eq!(recovered, valid);
    assert_eq!(
        readback::continuous_custody(&old, Phase::Final).as_slice(),
        old_entries.as_slice()
    );
    assert_eq!(
        old.final_inline_fragments()
            .iter()
            .map(|row| row.fragment().finalized_fragment().clone())
            .collect::<Vec<_>>(),
        old_fragments
    );
    publication(&old, &fixture);
}
