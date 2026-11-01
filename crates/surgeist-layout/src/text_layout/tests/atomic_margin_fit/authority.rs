//! CSS-composed fit retains positioned Native obligations and replay authority.

use super::*;
use text::{AtomicFitAllocation, MarginEdgeFitAuthority};

#[derive(Clone, Copy)]
struct Spec {
    border: f64,
    advance: f64,
    occupied: (f64, f64),
    fit: AtomicFitAllocation,
}
fn native() -> Spec {
    Spec {
        border: 10.0,
        advance: 6.0,
        occupied: (0.0, 10.0),
        fit: AtomicFitAllocation::NativeOccupied,
    }
}
fn css(border: f64, end: f64, advance: f64, occupied: (f64, f64)) -> Spec {
    let receipt = text::CssMarginBoxFit::try_new(0.0, border, end).unwrap();
    assert_eq!(receipt.margin_start(), 0.0);
    assert_eq!(receipt.border_inline_size(), border);
    assert_eq!(receipt.margin_end(), end);
    assert_eq!(receipt.advance(), advance);
    assert_eq!(receipt.occupied_inline_bounds(), occupied);
    Spec {
        border,
        advance,
        occupied,
        fit: AtomicFitAllocation::CssMarginBox(receipt),
    }
}
fn prepared(specs: &[Spec]) -> PreparedParagraph {
    // Genuine authored inline boxes, one distinct issued occurrence each,
    // even though all have the SAME source offset0 in this no-text paragraph.
    let p = PreparedParagraph::new(
        specs
            .iter()
            .enumerate()
            .map(|(index, spec)| ParagraphPart::Atomic {
                node: 100 + u32::try_from(index).unwrap(),
                size: text::Size::new(spec.border, 10.0),
            })
            .collect(),
        ahem_style(10.0, 10.0),
        ParagraphBaseDirection::LeftToRight,
    );
    assert!(p.prepared.analyzed_source().source().text().is_empty());
    assert_eq!(p.prepared.origin_views().len(), 0);
    let views = p.prepared.atomic_views().collect::<Vec<_>>();
    assert_eq!(views.len(), specs.len());
    let binding = p.binding(0);
    for (index, (view, spec)) in views.iter().zip(specs).enumerate() {
        assert_eq!(view.source_position().get(), 0);
        assert_eq!(view.original_box().kind(), text::InlineBoxKind::InFlow);
        assert_eq!(
            view.original_box().size(),
            text::Size::new(spec.border, 10.0)
        );
        assert_eq!(
            binding.atomics[index].node,
            100 + u32::try_from(index).unwrap()
        );
        assert_eq!(binding.atomics[index].occurrence, view.handle());
        for other in &views[..index] {
            assert_ne!(view.handle(), other.handle());
        }
    }
    assert_eq!(
        p.prepared.participants().collect::<Vec<_>>(),
        views
            .iter()
            .map(|view| text::PreparedParticipant::Atomic(view.handle()))
            .collect::<Vec<_>>()
    );
    p
}
fn request(p: &text::PreparedText, specs: &[Spec], band: f64) -> text::LineSelectionRequest {
    let segment = p.source_segments().next().unwrap();
    let actual = p.atomic_views().collect::<Vec<_>>();
    assert_eq!(actual.len(), specs.len());
    text::LineSelectionRequest {
        inline_boxes: None,
        segment: segment.clone(),
        start: segment.start(),
        end: segment.end(),
        participants: p.participants().collect(),
        termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
        alternative: None,
        incoming: None,
        atomics: actual
            .into_iter()
            .zip(specs)
            .map(|(view, spec)| text::AtomicAllocation {
                occurrence: view.handle(),
                allocation: text::AtomicLineAllocation::InFlow {
                    flow: text::AtomicFlowAllocation {
                        advance: spec.advance,
                        before_baseline: 8.0,
                        after_baseline: 2.0,
                        occupied_start: spec.occupied.0,
                        occupied_end: spec.occupied.1,
                    },
                    fit: spec.fit,
                    box_geometry: text::AtomicBoxAllocation {
                        border: text::Rect::new(0.0, -8.0, spec.border, 10.0),
                        content: text::Rect::new(0.0, -8.0, spec.border, 10.0),
                        baseline: 0.0,
                    },
                    alignment: text::LineBoxAlignment::Baseline,
                },
            })
            .collect(),
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![],
        context: text::LineContext {
            alignment_insets: text::LineAlignmentInsets::ZERO,
            formatted_line: text::FormattedLineContext::default(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: text::LineBoxDirection::LeftToRight,
            orientation: text::LineOrientation::Horizontal,
            start: text::LineStart::Paragraph,
            end: text::LineEnd::Paragraph,
            available_inline: Some(band),
            indent: 0.0,
            alignment: text::Alignment::Start,
            tabs: text::TabContext::NoTabs,
            spacing: text::SpacingPolicy::None,
        },
    }
}
fn finalize(p: &text::PreparedText, request: text::LineSelectionRequest) -> text::FinalizedLine {
    p.select(request)
        .unwrap()
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap()
}
#[derive(Clone, Copy)]
struct Expected {
    authority: MarginEdgeFitAuthority,
    f: f64,
    l: f64,
    r: f64,
    w: f64,
    raw: (f64, f64),
}
fn evidence(line: &text::FinalizedLine, expected: Expected) {
    let measurements = line.measurements();
    // Pure atomic operations have no trimming, hanging or spacing causes to
    // change this selected line between its three operation-specific views.
    for observed in [
        measurements.margin_edge_fit(),
        measurements.margin_edge_min_content_contribution(),
        measurements.margin_edge_max_content_contribution(),
    ] {
        assert_eq!(observed.authority(), expected.authority);
        assert_eq!(observed.advance(), expected.f);
        assert_eq!(observed.constraint_start(), expected.l);
        assert_eq!(observed.constraint_end(), expected.r);
        assert_eq!(observed.required_inline_size(), expected.w);
    }
    let raw = measurements.fit();
    assert_eq!(raw.advance(), expected.f);
    assert_eq!((raw.occupied_start(), raw.occupied_end()), expected.raw);
    assert!(
        line.run_views().all(|run| run.glyphs().next().is_none()),
        "no dummy glyph/source"
    );
}
fn positions(line: &text::FinalizedLine, specs: &[Spec], incoming: &[f64]) {
    let slots = line.atomic_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), specs.len());
    let views = line.prepared_text();
    let actual = views.atomic_views().collect::<Vec<_>>();
    for (index, ((slot, spec), &p)) in slots.iter().zip(specs).zip(incoming).enumerate() {
        assert_eq!(slot.occurrence, actual[index].handle());
        assert_eq!(
            slot.origin.x, p,
            "actual incoming pen; never re-zero/clamp a negative rectangle"
        );
        let text::AtomicLineAllocation::InFlow {
            flow,
            fit,
            box_geometry,
            ..
        } = slot.allocation
        else {
            panic!("actual in-flow slot");
        };
        assert_eq!(fit, spec.fit);
        assert_eq!(
            (flow.advance, flow.occupied_start, flow.occupied_end),
            (spec.advance, spec.occupied.0, spec.occupied.1)
        );
        assert_eq!(box_geometry.border.size, text::Size::new(spec.border, 10.0));
        assert_eq!(slot.origin.x + box_geometry.border.origin.x, p);
        assert_eq!(
            slot.origin.x + box_geometry.border.origin.x + box_geometry.border.size.width,
            p + spec.border
        );
        assert_eq!(slot.origin.y + box_geometry.border.origin.y, 0.0);
    }
}
fn execute(specs: &[Spec], incoming: &[f64], band: f64, expected: Expected) -> text::FinalizedLine {
    let p = prepared(specs);
    let issued = request(&p.prepared, specs, band);
    let line = finalize(&p.prepared, issued.clone());
    assert_eq!(line.prepared_text(), p.prepared);
    assert_eq!(line.selected_line().request(), &issued);
    evidence(&line, expected);
    positions(&line, specs, incoming);
    let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(replay.selected_line().request(), &issued);
    evidence(&replay, expected);
    positions(&replay, specs, incoming);
    // The preceding output remains intact while both owners coexist.
    evidence(&line, expected);
    line
}

#[test]
fn zero_margin_css_authority_keeps_the_later_positioned_native_requirement() {
    let line = execute(
        &[css(100.0, 0.0, 100.0, (0.0, 100.0)), native()],
        &[0.0, 100.0],
        106.0,
        Expected {
            authority: MarginEdgeFitAuthority::CssComposed,
            f: 106.0,
            l: 0.0,
            r: 110.0,
            w: 110.0,
            raw: (0.0, 110.0),
        },
    );
    assert!(line.measurements().margin_edge_fit().required_inline_size() > 106.0);
    // Explicit CSS mode even with margins0; no "any nonzero margin" heuristic.
}
#[test]
fn signed_negative_opaque_endpoints_have_no_implicit_zero_upper_anchor() {
    execute(
        &[css(10.0, -110.0, -100.0, (-100.0, 10.0)), native()],
        &[0.0, -100.0],
        10.0,
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
fn a_later_css_advance_does_not_erase_the_earlier_native_position() {
    execute(
        &[
            css(10.0, -110.0, -100.0, (-100.0, 10.0)),
            native(),
            css(100.0, 0.0, 100.0, (0.0, 100.0)),
        ],
        &[0.0, -100.0, -94.0],
        106.0,
        Expected {
            authority: MarginEdgeFitAuthority::CssComposed,
            f: 6.0,
            l: -100.0,
            r: 6.0,
            w: 106.0,
            raw: (-100.0, 10.0),
        },
    );
}
#[test]
fn unknown_native_occupied10_cannot_fit6_because_a_different_css_box_retracts() {
    let line = execute(
        &[native(), css(10.0, -15.0, -5.0, (-5.0, 10.0))],
        &[0.0, 6.0],
        6.0,
        Expected {
            authority: MarginEdgeFitAuthority::CssComposed,
            f: 1.0,
            l: 0.0,
            r: 10.0,
            w: 10.0,
            raw: (0.0, 16.0),
        },
    );
    assert!(line.measurements().margin_edge_fit().required_inline_size() > 6.0);
}
#[test]
fn two_actual_native_intervals_keep_their_separation_instead_of_summing_widths() {
    execute(
        &[
            native(),
            css(10.0, -110.0, -100.0, (-100.0, 10.0)),
            native(),
        ],
        &[0.0, 6.0, -94.0],
        104.0,
        Expected {
            authority: MarginEdgeFitAuthority::CssComposed,
            f: -88.0,
            l: -94.0,
            r: 10.0,
            w: 104.0,
            raw: (-94.0, 16.0),
        },
    );
}
#[test]
fn identical_raw6_over10_geometry_does_not_launder_css_or_native_request_authority_on_replay() {
    let native_spec = native();
    let css_spec = css(10.0, -4.0, 6.0, (0.0, 10.0));
    let p = prepared(&[native_spec]);
    let native_request = request(&p.prepared, &[native_spec], 6.0);
    let css_request = request(&p.prepared, &[css_spec], 6.0);
    assert_ne!(native_request, css_request);
    // Demonstrate the role is the ONLY changed request fact, not different
    // source, geometry, metrics, font, participant order or available width.
    let mut restored = css_request.clone();
    let text::AtomicLineAllocation::InFlow { fit, .. } = &mut restored.atomics[0].allocation else {
        unreachable!();
    };
    *fit = AtomicFitAllocation::NativeOccupied;
    assert_eq!(restored, native_request);
    let n = finalize(&p.prepared, native_request.clone());
    let c = finalize(&p.prepared, css_request.clone());
    let native_expected = Expected {
        authority: MarginEdgeFitAuthority::NativeOnly,
        f: 6.0,
        l: 0.0,
        r: 10.0,
        w: 10.0,
        raw: (0.0, 10.0),
    };
    let css_expected = Expected {
        authority: MarginEdgeFitAuthority::CssComposed,
        f: 6.0,
        l: 0.0,
        r: 6.0,
        w: 6.0,
        raw: (0.0, 10.0),
    };
    evidence(&n, native_expected);
    evidence(&c, css_expected);
    assert_eq!(n.measurements().fit(), c.measurements().fit());
    positions(&n, &[native_spec], &[0.0]);
    positions(&c, &[css_spec], &[0.0]);
    assert!(n.measurements().margin_edge_fit().required_inline_size() > 6.0);
    assert!(c.measurements().margin_edge_fit().required_inline_size() <= 6.0);
    for (original, issued, spec, expected) in [
        (&n, &native_request, native_spec, native_expected),
        (&c, &css_request, css_spec, css_expected),
    ] {
        let replay = original
            .with_spacing(text::SpacingAllocations::None)
            .unwrap();
        assert_eq!(replay.selected_line().request(), issued);
        evidence(&replay, expected);
        positions(&replay, &[spec], &[0.0]);
    }
}

#[path = "authority/admission.rs"]
mod admission;

#[path = "authority/indent_policy.rs"]
mod indent_policy;
