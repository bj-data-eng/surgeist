//! Actual unanchored Native obligations compose with signed indentation once.

use super::*;

fn actual_unanchored_receipt(
    line: &text::FinalizedLine,
    indent: f64,
    opaque: Option<(f64, f64)>,
    width: Option<f64>,
) {
    let measurements = line.measurements();
    // Pure atomics have no whitespace, hanging, trimming or spacing cause that
    // changes their actual participant/receipt ownership across these views.
    for admission in [
        measurements.margin_edge_fit(),
        measurements.margin_edge_min_content_contribution(),
        measurements.margin_edge_max_content_contribution(),
    ] {
        assert_eq!(admission.native_occupied_bounds(), opaque);
        assert_eq!(
            crate::text_layout::fit::css_margin_edge_width(admission, indent)
                .unwrap_or_else(|_| panic!("literal finite signed-indent policy operands")),
            width
        );
    }
}

fn executed(
    specs: &[Spec],
    actual_incoming: &[f64],
    indent: f64,
    opaque: Option<(f64, f64)>,
    width: Option<f64>,
    expected: Expected,
) {
    let p = prepared(specs);
    let mut issued = request(&p.prepared, specs, 200.0);
    // This is also a REAL supplied Text selection indent. It moves actual slots
    // below; neither the receipt nor raw measured flow/envelope is pre-shifted.
    // 200 is only a generous real available band for this direct Text operation,
    // not an independently asserted Layout chooser or float-fit outcome.
    issued.context.indent = indent;
    let line = finalize(&p.prepared, issued.clone());
    assert_eq!(line.selected_line().request(), &issued);
    assert_eq!(line.selected_line().request().context.indent, indent);
    evidence(&line, expected);
    positions(&line, specs, actual_incoming);
    actual_unanchored_receipt(&line, indent, opaque, width);

    let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(replay.selected_line().request(), &issued);
    evidence(&replay, expected);
    positions(&replay, specs, actual_incoming);
    actual_unanchored_receipt(&replay, indent, opaque, width);
    // Replay and the policy observation do not mutate the original output,
    // source occurrence, signed incoming positions, raw envelope or receipt.
    evidence(&line, expected);
    positions(&line, specs, actual_incoming);
    actual_unanchored_receipt(&line, indent, opaque, width);
}

#[test]
fn negative_indent_uses_actual_positive_native_extrema_not_a_shifted_artificial_zero() {
    // Source-local CSS100 occupies0..100, then Native incoming100 occupies
    //100..110. UnindentedF106, raw0..110 and actual opaque100..110.
    // I-20 => FI86, LI=min(0,80)=0, RI=max(86,90)=90 => W90.
    // Shifting the old artificial L0 instead gives LI-20 and wrong W110.
    // Actual slots are -20/80 and borders -20..80/80..90, not relocated boxes.
    executed(
        &[css(100.0, 0.0, 100.0, (0.0, 100.0)), native()],
        &[-20.0, 80.0],
        -20.0,
        Some((100.0, 110.0)),
        Some(90.0),
        Expected {
            authority: MarginEdgeFitAuthority::CssComposed,
            f: 106.0,
            l: 0.0,
            r: 110.0,
            w: 110.0,
            raw: (0.0, 110.0),
        },
    );
}

#[test]
fn positive_indent_keeps_negative_native_extrema_without_an_artificial_zero_upper_anchor() {
    // CSS10 end-110 has signedflow-100. Actual Native incoming-100 occupies
    //-100..-90; F-94. I10 => FI-84, LI-90, RI-80 => W10.
    // Both Native endpoints stay negative; RI must not be raised to0.
    // Actual CSS border is10..20; Native border is-90..-80. Neither its raw
    // geometric union nor an earlier CSS border becomes a new opaque bound.
    executed(
        &[css(10.0, -110.0, -100.0, (-100.0, 10.0)), native()],
        &[10.0, -90.0],
        10.0,
        Some((-100.0, -90.0)),
        Some(10.0),
        Expected {
            authority: MarginEdgeFitAuthority::CssComposed,
            f: -94.0,
            l: -100.0,
            r: -90.0,
            w: 10.0,
            raw: (-100.0, 10.0),
        },
    );
}

#[test]
fn genuine_absence_of_native_extrema_is_not_a_fake_zero_width_native_constraint() {
    // Every case contains ONE real CSS-authorized border10 atomic, not an
    // empty paragraph, placeholder or invented Native [0,0] allocation.
    // Independent AC literal controls, with terminal floor only AFTER I+F:
    // F-20,I10 => max0(-10)=0; F0,I10=>10; F20,I-10=>10.
    // Fake opaque[0,0] would wrongly give10 in the first and20 in the third.
    for (spec, indent, incoming, expected, width) in [
        (
            css(10.0, -30.0, -20.0, (-20.0, 10.0)),
            10.0,
            [10.0],
            Expected {
                authority: MarginEdgeFitAuthority::CssComposed,
                f: -20.0,
                l: 0.0,
                r: -20.0,
                w: 0.0,
                raw: (-20.0, 10.0),
            },
            0.0,
        ),
        (
            css(10.0, -10.0, 0.0, (0.0, 10.0)),
            10.0,
            [10.0],
            Expected {
                authority: MarginEdgeFitAuthority::CssComposed,
                f: 0.0,
                l: 0.0,
                r: 0.0,
                w: 0.0,
                raw: (0.0, 10.0),
            },
            10.0,
        ),
        (
            css(10.0, 10.0, 20.0, (0.0, 20.0)),
            -10.0,
            [-10.0],
            Expected {
                authority: MarginEdgeFitAuthority::CssComposed,
                f: 20.0,
                l: 0.0,
                r: 20.0,
                w: 20.0,
                raw: (0.0, 20.0),
            },
            10.0,
        ),
    ] {
        executed(&[spec], &incoming, indent, None, Some(width), expected);
    }
}

#[test]
fn native_only_admission_keeps_its_owner_and_does_not_acquire_css_indent_policy() {
    for (indent, incoming) in [(-20.0, [-20.0]), (10.0, [10.0])] {
        // A genuine Native6/occupied10 object exists and moves with the supplied
        // indent. None here means no CssComposed unanchored receipt, NOT absent
        // raw geometry and NOT inferred CSS authority from numeric values.
        executed(
            &[native()],
            &incoming,
            indent,
            None,
            None,
            Expected {
                authority: MarginEdgeFitAuthority::NativeOnly,
                f: 6.0,
                l: 0.0,
                r: 10.0,
                w: 10.0,
                raw: (0.0, 10.0),
            },
        );
    }
}
