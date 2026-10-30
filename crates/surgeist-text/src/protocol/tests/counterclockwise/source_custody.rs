//! Authored, selected and removed source retain their own issued identities.
use super::*;

#[test]
fn signed_spacing_hulls_and_css_tabs_use_one_directed_counterclockwise_frame() {
    let mut system = system_with(AHEM.to_vec());
    for (spacing, advance, space_pen, last_pen, minimum, width) in [
        (-12.0, 18.0, 4.0, 8.0, 0.0, 18.0),
        (-25.0, 5.0, -2.5, -5.0, -5.0, 15.0),
    ] {
        let p = capture(
            &mut system,
            "A A",
            Style {
                word_spacing: spacing,
                ..own_style()
            },
            false,
            CCW,
        );
        let line = whole(&p);
        // Each raw advance is10. The signed SPACE contribution is10+spacing;
        // half the spacing displaces its paint pen. The occupied hull includes
        // both endpoints even for a decreasing span. The separate native line
        // metric frame uses its alignment advance, not that occupied hull.
        assert_eq!(line.feedback().advance, advance);
        assert_eq!(
            glyphs(&line),
            [
                (35, 0.0, 0.0, 0.0, 10.0),
                (3, space_pen, 0.0, 0.0, 10.0),
                (35, last_pen, 0.0, 0.0, 10.0)
            ]
        );
        assert_eq!(line.metric_bounds(), Rect::new(0.0, 0.0, advance, 10.0));
        let layout = placed(&p, vec![(line, -7.25, 13.5)]);
        assert_eq!(
            layout.lines()[0].bounds(),
            Rect::new(-15.25, 13.5 - advance, 10.0, advance)
        );
        assert_eq!(
            layout.bounds(),
            Rect::new(-15.25, 13.5 - minimum - width, 10.0, width)
        );
        assert_eq!(
            caret(&layout, 2, Affinity::After),
            Rect::new(-15.25, 12.5 - last_pen, 10.0, 1.0)
        );
    }
    let style = Style {
        tab_size: crate::TabSize::length(40.0).unwrap(),
        ..own_style()
    };
    let mut value = paragraph(source("A\tB"), style.clone(), false);
    value.tab_reference = Some(
        crate::ParagraphTabReference::new(style, ParagraphBaseDirection::LeftToRight).unwrap(),
    );
    let p = system
        .prepare(
            &analyze(value),
            PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops)
                .unwrap()
                .with_orientation(CCW),
        )
        .unwrap();
    let mut r = selected_request(&p, start(&p), terminal(&p));
    r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    let selected = p.select(r).unwrap();
    let before = system.stats();
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(system.stats(), before);
    assert_eq!(line.feedback().advance, 50.0);
    assert_eq!(
        glyphs(&line),
        [(35, 0.0, 0.0, 0.0, 10.0), (36, 40.0, 0.0, 0.0, 10.0)]
    );
    let layout = placed(&p, vec![(line, -7.25, 13.5)]);
    assert_eq!(layout.bounds(), Rect::new(-15.25, -36.5, 10.0, 50.0));
    assert_eq!(layout.source().text().code_units(), [65, 9, 66]);
}

#[test]
fn retained_spacing_resources_replay_without_provider_calls_after_system_drop() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let mut system = system_with(bytes.clone());
    let p = capture(&mut system, "A A", own_style(), false, CCW);
    let mut r = selected_request(&p, start(&p), terminal(&p));
    r.context.spacing = SpacingPolicy::Spaces;
    let selected = p.select(r).unwrap();
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let stats = system.stats();
    let plain = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let expanded = plain
        .with_spacing(SpacingAllocations::Complete(vec![SpacingAllocation {
            slot: slots[0].clone(),
            amount: 20.0,
        }]))
        .unwrap();
    let restored = expanded.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(system.stats(), stats);
    assert_eq!(expanded.selected_line(), plain.selected_line());
    assert_eq!(glyphs(&plain), glyphs(&restored));
    assert_eq!(expanded.feedback().advance, 50.0);
    assert_eq!(
        glyphs(&expanded),
        [
            (35, 0.0, 0.0, 0.0, 10.0),
            (3, 10.0, 0.0, 0.0, 10.0),
            (35, 40.0, 0.0, 0.0, 10.0)
        ]
    );
    drop(system);
    let layout = placed(&p, vec![(expanded, 100.0, 20.0)]);
    assert_eq!(layout.bounds(), Rect::new(92.0, -30.0, 10.0, 50.0));
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert!(
            run.effective_instance()
                .unwrap()
                .normalized_coords()
                .next()
                .is_none()
        );
    }
    assert_eq!(
        placed(&p, vec![(restored, 100.0, 20.0)]).bounds(),
        Rect::new(92.0, -10.0, 10.0, 30.0)
    );
}

#[test]
fn before_after_and_both_indications_keep_the_selected_source_and_horizontal_feature_default() {
    use crate::{HyphenationPlacement as P, NativeHyphenationConvention, Text};
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let mut system = system_with(bytes.clone());
    for (mode, advances) in [
        (P::LogicalBefore, (30.0, 30.0)),
        (P::LogicalAfter, (20.0, 40.0)),
        (P::BothSides, (30.0, 40.0)),
    ] {
        let style = Style {
            locale: Some("und-Latn".into()),
            hyphens: crate::Hyphens::Manual,
            native_hyphenation_convention: Some(
                NativeHyphenationConvention::language_script(
                    "und-Latn",
                    "Latn",
                    Text::from("X"),
                    mode,
                )
                .unwrap(),
            ),
            ..own_style()
        };
        let p = capture(&mut system, "AB\u{ad}CCC", style, false, CCW);
        let offer = p
            .hyphenation_cut_views()
            .find(|cut| cut.boundary().view().source_before().get() == 3)
            .unwrap()
            .opportunity()
            .unwrap();
        let alternative = offer.view().alternatives().next().unwrap().handle();
        assert_eq!(alternative.view().text().code_units(), [88]);
        assert_eq!(alternative.view().source_range(), Some(Range::new(2, 3)));
        let mut r = selected_request(&p, start(&p), offer.clone());
        r.alternative = Some(alternative.clone());
        let first = p
            .select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let mut r = selected_request(&p, offer.view().boundary(), terminal(&p));
        r.incoming = first.hyphenation_continuation();
        let second = p
            .select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(
            first.selected_line().request().alternative,
            Some(alternative)
        );
        assert_eq!(
            (first.feedback().advance, second.feedback().advance),
            advances
        );
        assert_eq!(
            glyphs(&first).iter().filter(|g| g.0 == 58).count(),
            usize::from(mode != P::LogicalAfter)
        );
        assert_eq!(
            glyphs(&second).iter().filter(|g| g.0 == 58).count(),
            usize::from(mode != P::LogicalBefore)
        );
        assert!(
            !glyphs(&first)
                .iter()
                .chain(glyphs(&second).iter())
                .any(|g| [82, 277, 274].contains(&g.0))
        );
        let layout = placed(&p, vec![(first, 100.25, 20.5), (second, -30.5, -10.25)]);
        assert_eq!(
            layout.lines()[0].bounds(),
            Rect::new(92.25, 20.5 - advances.0, 10.0, advances.0)
        );
        assert_eq!(
            layout.lines()[1].bounds(),
            Rect::new(-38.5, -10.25 - advances.1, 10.0, advances.1)
        );
        assert_eq!(
            layout.source().text().code_units(),
            [65, 66, 173, 67, 67, 67]
        );
        assert_eq!(layout.source().id(), Some(Id::from_u64(1_579_001)));
        for run in layout.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.glyph_basis(), GlyphBasis::Counterclockwise);
        }
    }
}

#[test]
fn supplementary_utf16_and_nonformatted_source_removals_preserve_distinct_custody() {
    let bytes = crate::mixed_test_fonts::with_cmap(AHEM, &[(0x41, 36), (0x1f600, 35)]);
    let mut system = system_with(bytes);
    let p = capture(&mut system, "\u{1f600}A", own_style(), false, CCW);
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    let layout = placed(&p, vec![(whole(&p), 100.0, 20.0)]);
    assert_eq!(layout.source().text().code_units(), [0xd83d, 0xde00, 65]);
    assert_eq!(
        layout
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs().iter().map(|g| (g.id(), g.range())))
            .collect::<Vec<_>>(),
        [(35, Range::new(0, 2)), (36, Range::new(2, 3))]
    );
    assert_eq!(
        caret(&layout, 2, Affinity::After),
        Rect::new(92.0, 9.0, 10.0, 1.0)
    );
    let first = layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(0, Affinity::After))
        .unwrap();
    let last = layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(3, Affinity::Before))
        .unwrap();
    assert_eq!(
        layout
            .position_selection(&first, &last)
            .unwrap()
            .source_text()
            .code_units(),
        [0xd83d, 0xde00, 65]
    );
    let mut system = system_with(AHEM.to_vec());
    let p = capture(
        &mut system,
        "  ",
        Style {
            white_space: WhiteSpace::Discard,
            ..own_style()
        },
        false,
        CCW,
    );
    let line = whole(&p);
    assert_eq!(
        (line.feedback().advance, line.feedback().line_height),
        (0.0, 10.0)
    );
    assert!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .is_empty()
    );
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::NonFormatted(
            crate::NonFormattedLinePlacement {
                line: line.nonformatted().unwrap(),
                origin: Point::new(-7.25, 13.5),
                orientation: CCW,
                atomics: vec![],
            },
        )],
    })
    .unwrap();
    assert!(layout.lines().is_empty());
    assert_eq!(layout.positions().count(), 0);
    assert_eq!(layout.hit(Point::new(-7.25, 13.5)), Hit::None);
    assert_eq!(layout.source().text().code_units(), [32, 32]);
    assert_eq!(
        layout.nonformatted_lines().next().unwrap().origin,
        Point::new(-7.25, 13.5)
    );
}

#[test]
fn profile_foreign_and_numeric_failures_preserve_a_prior_accepted_layout() {
    let mut system = system_with(AHEM.to_vec());
    let p = capture(&mut system, "A", own_style(), false, CCW);
    let line = whole(&p);
    let old = placed(&p, vec![(line.clone(), -7.25, 13.5)]);
    for wrong in [H, CW] {
        let mut r = selected_request(&p, start(&p), terminal(&p));
        r.context.orientation = wrong;
        assert_eq!(
            reason(&p.select(r).unwrap_err()),
            &ProtocolError::MismatchedContext {
                field: ProtocolContextField::Orientation
            }
        );
        let mut values = entries(&p, vec![(line.clone(), -7.25, 13.5)]);
        let crate::PlacementEntry::Formatted(row) = &mut values[0] else {
            unreachable!()
        };
        row.orientation = wrong;
        let error = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: p.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            entries: values,
        })
        .unwrap_err();
        assert_eq!(
            reason(&error),
            &ProtocolError::MismatchedContext {
                field: ProtocolContextField::Orientation
            }
        );
    }
    let other = capture(&mut system, "A", own_style(), false, CCW);
    let error = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: other,
        coverage: PlacementCoverage::WholeParagraph,
        entries: entries(&p, vec![(line.clone(), -7.25, 13.5)]),
    })
    .unwrap_err();
    assert_eq!(
        reason(&error),
        &ProtocolError::ForeignFinalizedLine { line: 0 }
    );
    let error = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: entries(&p, vec![(line, f64::INFINITY, 13.5)]),
    })
    .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            category: crate::TextNumericError::NonFinite,
            ..
        })
    ));
    assert_eq!(old.bounds(), Rect::new(-15.25, 3.5, 10.0, 10.0));
    assert_eq!(old.glyph_runs()[0].glyphs()[0].id(), 35);
    assert_eq!(old.source().text().code_units(), [65]);
}

#[test]
fn generated_wbr_space_retains_control_custody_without_inventing_an_original_character() {
    let mut system = system_with(AHEM.to_vec());
    let mut value = paragraph(source("AA"), own_style(), false);
    value.controls = vec![crate::ParagraphControl {
        at: 1,
        kind: crate::ParagraphControlKind::Wbr {
            style: Box::new(Style {
                word_space_transform: crate::WordSpaceTransform::Space,
                ..own_style()
            }),
        },
    }];
    value.anchor_order = vec![crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::Control(0),
        affinity: Affinity::After,
    }];
    let p = prepare(&mut system, value, CCW);
    let control = p.control_views().next().unwrap().handle();
    let line = whole(&p);
    // Generated separator custody can split real runs. Pen is run-local;
    // run.origin.x+pen is the independently authored line-local placement.
    assert_eq!(
        line.run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|g| (g.id(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            (35, 0.0, 0.0, 10.0),
            (3, 0.0, 0.0, 10.0),
            (35, 0.0, 0.0, 10.0)
        ]
    );
    assert_eq!(
        line.run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|g| run.origin().x + g.pen())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [0.0, 10.0, 20.0]
    );
    let mappings = line
        .mapping_views()
        .filter(|m| m.kind() == PresentationMappingKind::GeneratedSeparator)
        .collect::<Vec<_>>();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].separator_control(), Some(control.clone()));
    assert_eq!(mappings[0].source_range(), Range::new(1, 1));
    assert!(mappings[0].processed_range().is_empty());
    let fragment = line
        .fragment_views()
        .find(|f| f.owner() == crate::FinalizedFragmentOwner::Control(control.clone()))
        .unwrap();
    assert!(fragment.origin().is_none());
    assert_eq!(fragment.bounds(), Some(Rect::new(10.0, 0.0, 10.0, 10.0)));
    let layout = placed(&p, vec![(line, -7.25, 13.5)]);
    assert_eq!(layout.bounds(), Rect::new(-15.25, -16.5, 10.0, 30.0));
    let first = layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(0, Affinity::After))
        .unwrap();
    let last = layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(2, Affinity::Before))
        .unwrap();
    let selected = layout.position_selection(&first, &last).unwrap();
    assert_eq!(selected.source_text().code_units(), [65, 65]);
    assert_eq!(selected.plaintext().code_units(), [65, 65]);
    assert_eq!(layout.source().text().code_units(), [65, 65]);
}
