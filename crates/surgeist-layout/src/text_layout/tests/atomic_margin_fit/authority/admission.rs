//! Public selection rejects CSS receipts inconsistent with their real allocation.

use super::*;

#[derive(Clone, Copy, Debug)]
enum Coupling {
    FlowAdvance,
    OccupiedStart,
    OccupiedEnd,
    BorderInlineSize,
}
impl Coupling {
    fn change(self, request: &mut text::LineSelectionRequest) {
        // The second ACTUAL issued atomic has index1 even though all three
        // genuine source occurrences have the same original source position0.
        let text::AtomicLineAllocation::InFlow {
            flow, box_geometry, ..
        } = &mut request.atomics[1].allocation
        else {
            panic!("the literal factory supplies a real in-flow allocation")
        };
        match self {
            Self::FlowAdvance => flow.advance = -99.0,
            Self::OccupiedStart => flow.occupied_start = -101.0,
            Self::OccupiedEnd => flow.occupied_end = 11.0,
            Self::BorderInlineSize => {
                // Keep a genuine contained content box and conservative Native
                // occupied[-100,10] while changing the CSS-coupled border size.
                box_geometry.border.size.width = 9.0;
                box_geometry.content.size.width = 9.0;
            }
        }
        // Every changed quantity is finite and every occupied interval remains
        // ordered; the width case leaves a positive border containing content.
        // Refusal must be the receipt-to-allocation coupling, not constructor
        // validation, a numeric failure, negative size or a reversed envelope.
        assert!(flow.advance.is_finite());
        assert!(flow.occupied_start.is_finite() && flow.occupied_end.is_finite());
        assert!(flow.occupied_start <= flow.occupied_end);
        assert!(box_geometry.border.size.width > 0.0);
    }
}

fn invalid_css_geometry(prepared: &text::PreparedText, request: text::LineSelectionRequest) {
    let error = match prepared.select(request) {
        Err(error) => error,
        Ok(_) => panic!("a mismatched CSS receipt must not issue SelectedLine authority"),
    };
    assert_eq!(error.code, text::ErrorCode::InvalidTextProtocol);
    assert!(matches!(
        error.detail(),
        Some(text::ErrorDetail::InvalidTextProtocol {
            reason: text::ProtocolError::InvalidAtomicGeometry { occurrence: 1 },
        })
    ));
}

#[test]
fn css_atomic_fit_receipt_must_match_actual_flow_envelope_and_border_at_selection() {
    // These exact literals come from the parent's independent signed-position
    // table. The CSS slot is incoming6, border10, end margin-110 => local
    // advance-100 and RAW local occupied[-100,10]. The last Native slot starts
    // at-94. Two independent Native intervals give L-94/R10/W104, finalF-88;
    // raw occupied[-94,16] remains distinct and must never be relocated/clamped.
    let specs = [
        native(),
        css(10.0, -110.0, -100.0, (-100.0, 10.0)),
        native(),
    ];
    let incoming = [0.0, 6.0, -94.0];
    let expected = Expected {
        authority: MarginEdgeFitAuthority::CssComposed,
        f: -88.0,
        l: -94.0,
        r: 10.0,
        w: 104.0,
        raw: (-94.0, 16.0),
    };
    let p = prepared(&specs);
    let issued = request(&p.prepared, &specs, 104.0);
    let actual = p
        .prepared
        .atomic_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(actual.len(), 3);
    assert_eq!(issued.atomics[1].occurrence, actual[1]);
    assert_ne!(actual[0], actual[1]);
    assert_ne!(actual[1], actual[2]);
    let valid = finalize(&p.prepared, issued.clone());
    assert_eq!(valid.selected_line().request(), &issued);
    evidence(&valid, expected);
    positions(&valid, &specs, &incoming);

    for coupling in [
        Coupling::FlowAdvance,
        Coupling::OccupiedStart,
        Coupling::OccupiedEnd,
        Coupling::BorderInlineSize,
    ] {
        let mut mismatch = issued.clone();
        coupling.change(&mut mismatch);
        let text::AtomicLineAllocation::InFlow { fit, .. } = mismatch.atomics[1].allocation else {
            unreachable!("actual in-flow allocation")
        };
        assert_eq!(
            fit, specs[1].fit,
            "the successfully constructed receipt stays unchanged"
        );
        assert_ne!(
            mismatch, issued,
            "{coupling:?} changes the actual allocation"
        );

        // Public phase admission is the operation under test. The typed ordinal
        // must identify the second issued occurrence, never source offset0.
        invalid_css_geometry(&p.prepared, mismatch.clone());

        // Independent control: NativeOccupied has no CSS-margin matching law.
        // The SAME finite allocation is a valid Native request; only its fit
        // authority changes. No parent/placed-rectangle policy is asserted.
        let mut native_request = mismatch.clone();
        let text::AtomicLineAllocation::InFlow { fit, .. } =
            &mut native_request.atomics[1].allocation
        else {
            unreachable!()
        };
        *fit = AtomicFitAllocation::NativeOccupied;
        let native = p
            .prepared
            .select(native_request.clone())
            .expect("the finite geometry is not itself invalid Native allocation");
        assert_eq!(native.request(), &native_request);

        // Failed CSS admission neither rewrites a previously returned finalized
        // output nor poisons its prepared owner. Retry the ORIGINAL valid request
        // through select+finalize, with the real retained receipt intact.
        assert_eq!(valid.prepared_text(), p.prepared);
        assert_eq!(valid.selected_line().request(), &issued);
        evidence(&valid, expected);
        positions(&valid, &specs, &incoming);
        let retry = finalize(&p.prepared, issued.clone());
        assert_eq!(retry.selected_line().request(), &issued);
        evidence(&retry, expected);
        positions(&retry, &specs, &incoming);
        // Coexistence matters: an admitted Native counterpart/retry cannot
        // launder the old CSS result's request, authority or RAW geometry.
        evidence(&valid, expected);
        positions(&valid, &specs, &incoming);
    }
    let replay = valid.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(replay.selected_line().request(), &issued);
    evidence(&replay, expected);
    positions(&replay, &specs, &incoming);
    evidence(&valid, expected);
}
