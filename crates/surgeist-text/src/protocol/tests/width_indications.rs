use super::*;

#[test]
fn width_maps_source_body_but_leaves_each_chosen_indication_exact_on_both_sides() {
    let bytes = crate::shaping_test_fonts::width_ahem();
    for placement in [
        HyphenationPlacement::LogicalBefore,
        HyphenationPlacement::LogicalAfter,
        HyphenationPlacement::BothSides,
    ] {
        let source = Source::new("AA\u{ad}AAA");
        let style = Style {
            text_transform: crate::TextTransform::none().with_full_width(true),
            ..own_style("A", placement)
        };
        let p = capture(source.clone(), style, &bytes);
        let offer = cut(&p, 3, counts(2, 3));
        let first = finish(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
        )
        .unwrap();
        let second = finish(
            &p,
            selected_request(
                &p,
                offer.view().boundary(),
                end(&p),
                first.hyphenation_continuation(),
                false,
            ),
        )
        .unwrap();
        let before = placement != HyphenationPlacement::LogicalAfter;
        let after = placement != HyphenationPlacement::LogicalBefore;
        assert_eq!(
            first.inner.selected.presentation.projection.text,
            if before { "ＡＡA" } else { "ＡＡ" }
        );
        assert_eq!(
            second.inner.selected.presentation.projection.text,
            if after { "AＡＡＡ" } else { "ＡＡＡ" }
        );
        assert_eq!(
            (first.feedback().advance, second.feedback().advance),
            (
                if before { 50.0 } else { 40.0 },
                if after { 70.0 } else { 60.0 }
            )
        );
        let left = glyphs(&first, &bytes)
            .iter()
            .map(|g| g.id)
            .collect::<Vec<_>>();
        let right = glyphs(&second, &bytes)
            .iter()
            .map(|g| g.id)
            .collect::<Vec<_>>();
        assert_eq!(
            left,
            if before {
                vec![36, 36, 35]
            } else {
                vec![36, 36]
            }
        );
        assert_eq!(
            right,
            if after {
                vec![35, 36, 36, 36]
            } else {
                vec![36, 36, 36]
            }
        );
        let layout =
            place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
        let positions: Vec<_> = layout.positions().collect();
        assert_eq!(
            layout
                .position_selection(positions.first().unwrap(), positions.last().unwrap())
                .unwrap()
                .source_text(),
            source.text()
        );
    }
}
