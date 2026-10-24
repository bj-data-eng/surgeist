//! BothSides uses one choice and independently shaped emissions on each side.
use super::*;

fn both(text: &str) -> Style {
    own_style(text, HyphenationPlacement::BothSides)
}
fn pair(p: &PreparedText) -> (PreparedOpportunity, FinalizedLine, FinalizedLine) {
    let offer = cut(p, 3, counts(2, 3));
    let first = finish(p, selected_request(p, start(p), offer.clone(), None, true)).unwrap();
    let second = finish(
        p,
        selected_request(
            p,
            offer.view().boundary(),
            end(p),
            first.hyphenation_continuation(),
            false,
        ),
    )
    .unwrap();
    (offer, first, second)
}
fn consecutive(
    mut source: Source,
    suffix: &str,
    bytes: &[u8],
) -> (PreparedText, Vec<FinalizedLine>) {
    source.span(Range::new(3, 4), both(suffix));
    let p = capture(source, both("A"), bytes);
    let one = cut(&p, 3, counts(2, 3));
    let two = cut(&p, 4, counts(2, 3));
    let first = finish(&p, selected_request(&p, start(&p), one.clone(), None, true)).unwrap();
    let middle = finish(
        &p,
        selected_request(
            &p,
            one.view().boundary(),
            two.clone(),
            first.hyphenation_continuation(),
            true,
        ),
    )
    .unwrap();
    let last = finish(
        &p,
        selected_request(
            &p,
            two.view().boundary(),
            end(&p),
            middle.hyphenation_continuation(),
            false,
        ),
    )
    .unwrap();
    (p, vec![first, middle, last])
}

#[test]
fn one_source_choice_has_two_query_units_without_broadening_partial_coverage() {
    let p = capture(
        Source::identified("AB\u{ad}CDE", crate::Id::from_u64(92001), 0),
        both("A"),
        AHEM,
    );
    let (offer, first, second) = pair(&p);
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (30.0, 40.0)
    );
    let chosen = alternative(&offer);
    assert_eq!(
        first.hyphenation_continuation().unwrap().alternative(),
        chosen
    );
    assert!(second.hyphenation_continuation().is_none());
    assert_eq!(
        p.alternative_views().count(),
        1,
        "one choice, not a second cut for the receiving prefix"
    );
    let origin = p.origin_views().next().unwrap().handle();
    let whole = place_lines(
        &p,
        PlacementCoverage::WholeParagraph,
        vec![first.clone(), second.clone()],
    )
    .unwrap();
    assert_eq!(
        whole.source().text().code_units(),
        [65, 66, 173, 67, 68, 69]
    );
    let fragments = whole
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    let units: Vec<_> = fragments.units().collect();
    assert_eq!(units.len(), 2);
    assert_eq!(
        units.iter().map(|u| u.line_index()).collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(!units[0].same_unit(&units[1]));
    for unit in &units {
        assert_eq!(
            unit.alternatives().collect::<Vec<_>>().as_slice(),
            std::slice::from_ref(&chosen)
        );
        assert_eq!(
            unit.defining_sources()
                .map(|s| s.aggregate_range())
                .collect::<Vec<_>>(),
            [Range::new(2, 3)]
        );
    }
    assert_eq!(
        units
            .iter()
            .flat_map(|u| u.pieces().map(|p| p.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            crate::Rect::new(20.0, 0.0, 10.0, 10.0),
            crate::Rect::new(0.0, 10.0, 10.0, 10.0)
        ]
    );
    let ending = place_lines(
        &p,
        PlacementCoverage::Range {
            start: first.selected_line().request().start.clone(),
            end: first.selected_line().request().end.clone(),
        },
        vec![first.clone()],
    )
    .unwrap();
    assert_eq!(
        ending
            .source_range_fragments(&origin, Range::new(2, 3))
            .unwrap()
            .units()
            .count(),
        1
    );
    let receiver = place_lines(
        &p,
        PlacementCoverage::Range {
            start: second.selected_line().request().start.clone(),
            end: second.selected_line().request().end.clone(),
        },
        vec![second.clone()],
    )
    .unwrap();
    let error = receiver
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap_err();
    assert!(
        matches!(error.detail(),Some(crate::ErrorDetail::SourceRangeFragments{reason:SourceRangeFragmentError::OutsidePlacement{aggregate}}) if *aggregate==Range::new(2,3))
    );
    let c = receiver
        .source_range_fragments(&origin, Range::new(3, 4))
        .unwrap();
    assert!(c.units().all(|u| u.alternatives().next().is_none()));
    assert_eq!(
        c.units()
            .flat_map(|u| u.pieces().map(|p| p.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [crate::Rect::new(10.0, 0.0, 10.0, 10.0)]
    );
    let mut absent = second.selected_line().request().clone();
    absent.incoming = None;
    let absent = finish(&p, absent).unwrap();
    hyphenation_reason(
        &place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, absent]).unwrap_err(),
        HyphenationError::InvalidContinuation,
    );
    let foreign = capture(Source::new("AB\u{ad}CDE"), both("A"), AHEM);
    let mut wrong = selected_request(
        &foreign,
        cut(&foreign, 3, counts(2, 3)).view().boundary(),
        end(&foreign),
        second.selected_line().request().incoming.clone(),
        false,
    );
    wrong.context.end = LineEnd::Paragraph;
    hyphenation_reason(
        &foreign.select(wrong).unwrap_err(),
        HyphenationError::InvalidContinuation,
    );
}

#[test]
fn a_source_less_control_owns_the_same_two_emissions_and_its_actual_font_size() {
    let source = Source::new("ABCDE");
    let mut own = both("A");
    own.size = 20.0;
    own.line_height = LineHeight::Absolute(20.0);
    let p = capture_input(
        ParagraphInput {
            source: source.clone(),
            base_style: own_style("X", HyphenationPlacement::LogicalBefore),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![paragraph::ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, 5),
                aggregate: Range::new(0, 5),
                kind: paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            controls: vec![paragraph::ParagraphControl {
                at: 2,
                kind: paragraph::ParagraphControlKind::DiscretionaryBreak {
                    style: Box::new(own),
                },
            }],
            anchor_order: vec![paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Control(0),
                affinity: crate::Affinity::After,
            }],
        },
        AHEM,
    );
    let offer = cut(&p, 2, counts(2, 3));
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
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (40.0, 50.0)
    );
    assert_eq!(
        first
            .run_views()
            .map(|r| r.effective_instance().unwrap().font_size())
            .collect::<Vec<_>>(),
        [10.0, 20.0]
    );
    assert_eq!(
        second
            .run_views()
            .map(|r| r.effective_instance().unwrap().font_size())
            .collect::<Vec<_>>(),
        [20.0, 10.0]
    );
    assert!(matches!(
        alternative(&offer).view().definition(),
        crate::PreparedAlternativeDefinition::Control { .. }
    ));
    assert!(alternative(&offer).view().source_range().is_none());
    let layout = place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
    assert_eq!(layout.source().text().code_units(), [65, 66, 67, 68, 69]);
}

#[test]
fn automatic_both_emissions_keep_the_actual_resource_cut_and_preceding_egc_owner() {
    let mut style = both("A");
    style.hyphens = Hyphens::Auto;
    style.locale = Some("en-US".into());
    style.native_hyphenation_convention = Some(
        NativeHyphenationConvention::language_script(
            "en-US",
            "Latn",
            Text::from("A"),
            HyphenationPlacement::BothSides,
        )
        .unwrap(),
    );
    let p = capture(Source::new("associate"), style, AHEM);
    assert_eq!(
        p.hyphenation_cut_views()
            .map(|c| c.boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [2, 4]
    );
    let native_cut = p
        .hyphenation_cut_views()
        .find(|c| c.boundary().view().source_before().get() == 4)
        .unwrap();
    assert!(matches!(
        native_cut.provenance(),
        crate::HyphenationProvenance::SourceException { .. }
    ));
    assert_eq!(native_cut.counts(), Some(counts(4, 5)));
    let owner = native_cut.handle();
    let offer = native_cut.opportunity().unwrap();
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
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (50.0, 60.0)
    );
    let chosen = alternative(&offer);
    assert_eq!(chosen.view().text().code_units(), [65]);
    assert!(matches!(
        chosen.view().definition(),
        crate::PreparedAlternativeDefinition::Automatic { .. }
    ));
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
    let fragments = layout
        .source_range_fragments(&origin, Range::new(3, 4))
        .unwrap();
    let computed: Vec<_> = fragments
        .units()
        .filter(|u| u.alternatives().next().is_some())
        .collect();
    assert_eq!(computed.len(), 2);
    assert_eq!(
        computed.iter().map(|u| u.line_index()).collect::<Vec<_>>(),
        [0, 1]
    );
    for u in computed {
        assert_eq!(
            u.alternatives().collect::<Vec<_>>().as_slice(),
            std::slice::from_ref(&chosen)
        );
        assert_eq!(
            u.hyphenation_cuts().collect::<Vec<_>>().as_slice(),
            std::slice::from_ref(&owner)
        );
        assert!(u.defining_sources().next().is_none());
    }
    assert_eq!(
        layout.source().text().code_units(),
        Source::new("associate").text().code_units()
    );
}

#[test]
fn empty_both_choice_has_one_real_ending_and_no_emission_or_continuation() {
    let p = capture(Source::new("AB\u{ad}CDE"), both(""), AHEM);
    let (offer, first, second) = pair(&p);
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (20.0, 30.0)
    );
    assert!(first.hyphenation_continuation().is_none());
    assert!(second.hyphenation_continuation().is_none());
    assert_eq!(
        first.selected_line().request().alternative,
        Some(alternative(&offer))
    );
    assert!(first.inner.selected.presentation.domains.is_empty());
    assert!(second.inner.selected.presentation.domains.is_empty());
    assert_eq!(p.alternative_views().count(), 1);
    place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
}

#[test]
fn before_and_after_use_opposite_authentic_attachments_for_the_same_mixed_string() {
    let mut cmap: Vec<_> = (65..=69).map(|cp| (cp, cp - 30)).collect();
    cmap.push((0x5d0, 36));
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(&[98, 99], &cmap);
    let p = capture(Source::new("AB\u{ad}\u{200f}CDE"), both("\u{5d0}A"), &bytes);
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
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (40.0, 50.0)
    );
    assert_eq!(
        (
            first.inner.selected.presentation.domains[0].outer,
            first.inner.selected.presentation.domains[0].base
        ),
        (0, 0)
    );
    assert_eq!(
        (
            second.inner.selected.presentation.domains[0].outer,
            second.inner.selected.presentation.domains[0].base
        ),
        (1, 1)
    );
    for (line, levels, ids) in [
        (&first, vec![1, 0], vec![36, 35]),
        (&second, vec![1, 2], vec![35, 36]),
    ] {
        let presentation = &line.inner.selected.presentation;
        let d = &presentation.domains[0];
        assert_eq!(
            presentation.units[d.units.clone()]
                .iter()
                .map(|u| u.level)
                .collect::<Vec<_>>(),
            levels
        );
        let actual: Vec<_> = line
            .run_views()
            .flat_map(|r| {
                r.glyphs()
                    .filter(|g| g.range() == Range::new(3, 3))
                    .map(|g| g.id())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(actual, ids);
        assert!(line.run_views().all(|r| r.face().bytes() == bytes));
    }
    assert_eq!(alternative(&offer).view().text().code_units(), [0x5d0, 65]);
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 66, 173, 0x200f, 67, 68, 69]
    );
    place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
}

#[test]
fn both_sides_retains_shared_aa_prefix_but_distinct_range_egcs() {
    let bytes = crate::shaping_test_fonts::ligature_ahem();
    let (p, lines) = consecutive(Source::new("XX\u{ad}\u{ad}XXX"), "A", &bytes);
    assert_eq!(
        lines
            .iter()
            .map(|l| l.feedback().advance)
            .collect::<Vec<_>>(),
        [30.0, 10.0, 40.0]
    );
    assert_eq!(
        glyphs(&lines[1], &bytes),
        [Glyph {
            id: 58,
            advance: 10.0,
            x: 0.0,
            y: 0.0
        }]
    );
    let incoming = lines[0].hyphenation_continuation().unwrap().alternative();
    let outgoing = lines[1].hyphenation_continuation().unwrap().alternative();
    assert_ne!(incoming, outgoing);
    let participant = p
        .unit_views()
        .find(|u| u.source_range() == Range::new(3, 4))
        .unwrap()
        .handle();
    let seam = TextSeam::BeforeParticipant(PreparedParticipant::Text(participant));
    let prefix = lines[1].prefix_at(&seam).unwrap();
    let joint = prefix.interior_unit().unwrap();
    assert_eq!(joint.advance().unwrap(), 10.0);
    assert_eq!(
        joint.alternatives().collect::<Vec<_>>(),
        [incoming.clone(), outgoing.clone()]
    );
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place_lines(&p, PlacementCoverage::WholeParagraph, lines).unwrap();
    let left = layout
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    let right = layout
        .source_range_fragments(&origin, Range::new(3, 4))
        .unwrap();
    let a = left.units().find(|u| u.line_index() == 1).unwrap();
    let b = right.units().find(|u| u.line_index() == 1).unwrap();
    assert!(!a.same_unit(&b));
    assert_eq!(a.alternatives().collect::<Vec<_>>(), [incoming]);
    assert_eq!(b.alternatives().collect::<Vec<_>>(), [outgoing]);
    assert_eq!(a.pieces().map(|p| p.rect().size.width).sum::<f64>(), 5.0);
    assert_eq!(b.pieces().map(|p| p.rect().size.width).sum::<f64>(), 5.0);
    assert_eq!((left.units().count(), right.units().count()), (2, 2));
}

#[test]
fn both_sides_keeps_joint_mark_egc_and_cross_egc_pair_positioning() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let (p, lines) = consecutive(Source::new("XX\u{ad}\u{ad}A\u{301}XX"), "\u{301}", &bytes);
    assert_eq!(lines[1].feedback().advance, 10.0);
    assert_eq!(
        glyphs(&lines[1], &bytes),
        [
            Glyph {
                id: 35,
                advance: 10.0,
                x: 0.0,
                y: 0.0
            },
            Glyph {
                id: 100,
                advance: 0.0,
                x: 3.0,
                y: -4.0
            }
        ]
    );
    let incoming = lines[0].hyphenation_continuation().unwrap().alternative();
    let outgoing = lines[1].hyphenation_continuation().unwrap().alternative();
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place_lines(&p, PlacementCoverage::WholeParagraph, lines).unwrap();
    let left = layout
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    let right = layout
        .source_range_fragments(&origin, Range::new(3, 4))
        .unwrap();
    let a = left.units().find(|u| u.line_index() == 1).unwrap();
    let b = right.units().find(|u| u.line_index() == 1).unwrap();
    assert!(a.same_unit(&b));
    assert_eq!(a.alternatives().collect::<Vec<_>>(), [incoming, outgoing]);
    assert_eq!(
        a.defining_sources()
            .map(|s| s.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(2, 3), Range::new(3, 4)]
    );
    let bytes = crate::shaping_test_fonts::pair_adjusted_ahem();
    let (p, lines) = consecutive(Source::new("XX\u{ad}\u{ad}XXX"), "B", &bytes);
    assert_eq!(lines[1].feedback().advance, 15.0);
    assert_eq!(
        glyphs(&lines[1], &bytes),
        [
            Glyph {
                id: 35,
                advance: 5.0,
                x: 0.0,
                y: 0.0
            },
            Glyph {
                id: 36,
                advance: 10.0,
                x: 5.0,
                y: 0.0
            }
        ]
    );
    assert!(lines[1].hyphenation_continuation().is_some());
    place_lines(&p, PlacementCoverage::WholeParagraph, lines).unwrap();
}

#[test]
fn two_side_profiles_do_not_manufacture_an_uncaptured_joint_mark_query() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let mut source = Source::new("XX\u{ad}\u{ad}XXX");
    source.span(Range::new(3, 4), both("\u{301}"));
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert!(font.charmap().map(65u32).is_some());
    assert!(font.charmap().map(0x301u32).is_some());
    let p = capture(source, both("A"), &bytes);
    let one = cut(&p, 3, counts(2, 3));
    let two = cut(&p, 4, counts(2, 3));
    let first = finish(&p, selected_request(&p, start(&p), one.clone(), None, true)).unwrap();
    let error = finish(
        &p,
        selected_request(
            &p,
            one.view().boundary(),
            two,
            first.hyphenation_continuation(),
            true,
        ),
    )
    .unwrap_err();
    hyphenation_reason(&error, HyphenationError::UncapturedIndicationFontQuery);
}

#[test]
fn manual_portions_and_raw_profiles_keep_both_emissions_without_native_count_bypass() {
    let p = capture(Source::new("AB\u{ad}CD\u{ad}EF"), both("A"), AHEM);
    let portions = p.word_views().next().unwrap().portions().unwrap();
    assert_eq!(portions.len(), 3);
    let mut observed = Vec::new();
    for portion in portions {
        let mut r = request(&p);
        r.start = portion.start();
        r.end = portion.end();
        r.alternative = portion.alternative();
        r.incoming = portion.incoming();
        r.termination = PreparedTermination::Measurement(portion);
        let from = r.start.view().participant_offset();
        let to = r.end.view().participant_offset();
        r.participants = p.participants().skip(from).take(to - from).collect();
        r.context.start = if from == 0 {
            LineStart::Paragraph
        } else {
            LineStart::Continuation
        };
        r.context.end = LineEnd::Soft;
        r.context.available_inline = None;
        r.context.spacing = SpacingPolicy::None;
        let line = finish(&p, r).unwrap();
        observed.push(line.feedback().advance);
        assert!(
            place_lines(
                &p,
                PlacementCoverage::Range {
                    start: line.selected_line().request().start.clone(),
                    end: line.selected_line().request().end.clone()
                },
                vec![line]
            )
            .is_err()
        );
    }
    assert_eq!(observed, [30.0, 40.0, 30.0]);
    let short = Source::new("AB\u{ad}CD");
    let native = capture(short.clone(), both("A"), AHEM);
    assert!(native.alternative_views().next().is_none());
    let unbroken = finish(&native, request(&native)).unwrap();
    assert_eq!(unbroken.feedback().advance, 40.0);
    for quantize in [false, true] {
        let mut system = system();
        let layout = system
            .layout(
                short.clone(),
                both("A"),
                crate::Options {
                    width: Some(30.0),
                    quantize,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|l| (l.range(), l.bounds().size.width))
                .collect::<Vec<_>>(),
            [(Range::new(0, 3), 30.0), (Range::new(3, 5), 30.0)]
        );
        assert_eq!(layout.source().text(), short.text());
        assert_eq!(
            layout
                .glyph_runs()
                .into_iter()
                .flat_map(|r| {
                    r.glyphs()
                        .iter()
                        .filter(|g| g.range() == Range::new(3, 3))
                        .map(|g| g.id())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            [35, 35]
        );
    }
}
