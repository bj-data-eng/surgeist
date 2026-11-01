//! Non-atomic components remain collapse-transparent; only actual BP blocks hanging.
//! Preserved separators and punctuation use their separate genuine Phase-II causes.
use super::*;

fn whole_inline(value: &str, own: t::Style) -> t::PreparedText {
    let n = value.encode_utf16().count();
    prepare_capture(
        t::Source::new(value),
        block(own, false, false),
        &[(0, n)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, n)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
        ],
        AHEM,
    )
}

#[test]
fn signed_and_cancelling_edges_remain_transparent_to_actual_collapsible_space() {
    for (value, leading, space, glyph_range) in [
        (" A", true, t::Range::new(0, 1), t::Range::new(1, 2)),
        ("A ", false, t::Range::new(1, 2), t::Range::new(0, 1)),
    ] {
        let p = whole_inline(value, style(t::WhiteSpace::Collapse));
        for (edge, flow, x, strip) in [
            (
                c(-5.0, 0.0, 0.0),
                5.0,
                if leading { -5.0 } else { 0.0 },
                if leading { (-5.0, -5.0) } else { (10.0, 10.0) },
            ),
            (
                c(-5.0, 2.0, 3.0),
                10.0,
                0.0,
                if leading { (-5.0, 0.0) } else { (10.0, 15.0) },
            ),
        ] {
            let line = signed_full(
                &p,
                if leading {
                    [edge, zero()]
                } else {
                    [zero(), edge]
                },
                100.0,
            );
            assert_eq!(paints(&line), [(glyph_range, 35, 10.0, x)]);
            assert_eq!(line.feedback().advance, flow);
            let row = line
                .edge_units()
                .find(|row| row.source_pieces().any(|range| range == space))
                .unwrap();
            assert_eq!(row.disposition(), t::LineEdgeDisposition::Removed);
            assert_eq!(row.advance(), 0.0);
            assert_eq!(p.analyzed_source().source().text(), &t::Text::from(value));
            let occurrence = p
                .control_views()
                .nth(usize::from(!leading))
                .unwrap()
                .handle();
            assert_eq!(
                line.control_slots()
                    .find(|slot| slot.control() == occurrence)
                    .unwrap()
                    .strip_interval(),
                strip
            );
            let replay = line.with_spacing(t::SpacingAllocations::None).unwrap();
            assert_eq!(paints(&replay), paints(&line));
        }
    }
    // Genuine <Inline>A SPACE</Inline>B: this SPACE is internal, not terminal.
    let p = prepare_capture(
        t::Source::new("A B"),
        block(style(t::WhiteSpace::Collapse), false, false),
        &[(0, 2), (2, 3)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 2)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
            E::Connection(0),
            origin(1),
        ],
        AHEM,
    );
    for (edge, b, flow) in [
        (c(-5.0, 0.0, 0.0), 15.0, 25.0),
        (c(-5.0, 2.0, 3.0), 20.0, 30.0),
    ] {
        let line = signed_full(&p, [zero(), edge], 100.0);
        assert_eq!(
            paints(&line),
            [
                (t::Range::new(0, 1), 35, 10.0, 0.0),
                (t::Range::new(1, 2), 3, 10.0, 10.0),
                (t::Range::new(2, 3), 36, 10.0, b)
            ]
        );
        assert_eq!(line.feedback().advance, flow);
        let space = line
            .edge_units()
            .find(|row| row.source_pieces().any(|r| r == t::Range::new(1, 2)))
            .unwrap();
        assert_ne!(space.disposition(), t::LineEdgeDisposition::Removed);
        assert_eq!(space.advance(), 10.0);
    }
}

#[test]
fn margin_only_preserved_separator_hanging_uses_actual_geometry_and_bp_remains_blocking() {
    // Pinned Text3:2776/Text4:5066: the BP blocker applies to ANY
    // hangable glyph, before the punctuation subsection. Margin-only does not
    // stop eligibility; actual SPACE10..20 still overflows the real band15.
    for mode in [t::WhiteSpace::Preserve, t::WhiteSpace::PreserveSpaces] {
        let p = whole_inline("A ", style(mode));
        for (edge, raw, fit, align, min, max) in [
            (zero(), 20.0, 10.0, 15.0, 10.0, 20.0),
            (c(5.0, 0.0, 0.0), 25.0, 15.0, 20.0, 15.0, 25.0),
            (c(-5.0, 0.0, 0.0), 15.0, 5.0, 10.0, 5.0, 15.0),
            (c(-5.0, 2.0, 3.0), 20.0, 20.0, 20.0, 20.0, 20.0),
        ] {
            let line = signed_full(&p, [zero(), edge], 15.0);
            assert_eq!(
                paints(&line),
                [
                    (t::Range::new(0, 1), 35, 10.0, 0.0),
                    (t::Range::new(1, 2), 3, 10.0, 10.0)
                ]
            );
            assert_eq!(measures(&line), [raw, fit, align, min, max]);
            assert_eq!(
                line.measurements().margin_edge_fit().required_inline_size(),
                fit
            );
            let space = line
                .edge_units()
                .find(|row| row.source_pieces().any(|r| r == t::Range::new(1, 2)))
                .unwrap();
            assert_ne!(space.disposition(), t::LineEdgeDisposition::Removed);
            assert_eq!(space.advance(), 10.0);
            // With no BP, conditional fit/min exclude the whole SPACE10;
            // alignment excludes its actual5px overflow. End margin is still
            // actual signed flow, not a generic trailing-whitespace subtraction.
            let replay = line.with_spacing(t::SpacingAllocations::None).unwrap();
            assert_eq!(measures(&replay), measures(&line));
        }
    }
}

#[test]
fn cancelling_bp_blocks_real_period_hanging_while_margin_alone_does_not() {
    let mut own = style(t::WhiteSpace::Preserve);
    own.hanging_punctuation =
        t::HangingPunctuation::new(false, Some(t::HangingPunctuationEnd::ForceEnd), false);
    let p = whole_inline("A.", own);
    // Maintained hanging_punctuation.rs qualifies Ahem A35/period16, size10,
    // advances10. ForceEnd is unconditional; first/last context usage is NOT
    // an applied-hanging flag. Observe actual period exclusion independently.
    for (edge, raw, measured, excluded) in [
        (zero(), 20.0, 10.0, 10.0),
        (c(5.0, 0.0, 0.0), 25.0, 15.0, 10.0),
        (c(-5.0, 0.0, 0.0), 15.0, 5.0, 10.0),
        (c(-5.0, 2.0, 3.0), 20.0, 20.0, 0.0),
    ] {
        let line = signed_full(&p, [zero(), edge], 40.0);
        assert_eq!(
            paints(&line),
            [
                (t::Range::new(0, 1), 35, 10.0, 0.0),
                (t::Range::new(1, 2), 16, 10.0, 10.0)
            ]
        );
        assert_eq!(
            measures(&line),
            [raw, measured, measured, measured, measured]
        );
        let period = line
            .edge_units()
            .find(|row| row.source_pieces().any(|r| r == t::Range::new(1, 2)))
            .unwrap();
        assert_eq!(period.advance(), 10.0);
        assert_eq!(period.excluded_contribution(), excluded);
        assert_eq!(
            line.measurements().margin_edge_fit().required_inline_size(),
            measured
        );
        assert_eq!(p.analyzed_source().source().text(), &t::Text::from("A."));
    }
}
