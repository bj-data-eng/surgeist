//! Real Wbr effective instances survive selected overlay and reused catalogues.
use super::*;
use crate::{FinalizedFragmentOwner, Id, WordSpaceTransform};

fn input(
    source: Source,
    specifications: &[(WordSpaceTransform, f32)],
    reverse: bool,
) -> ParagraphInput {
    let len = source.text().len();
    let base_style = Style {
        white_space: WhiteSpace::Discard,
        ..own_style("A", HyphenationPlacement::LogicalBefore)
    };
    ParagraphInput {
        source: source.clone(),
        base_style: base_style.clone(),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        replacements: vec![],
        boundary_policies: vec![],
        controls: specifications
            .iter()
            .map(|&(operation, size)| paragraph::ParagraphControl {
                at: 1,
                kind: paragraph::ParagraphControlKind::Wbr {
                    style: Box::new(Style {
                        size,
                        line_height: LineHeight::Absolute(f64::from(size)),
                        word_space_transform: operation,
                        ..base_style.clone()
                    }),
                },
            })
            .collect(),
        anchor_order: (0..specifications.len())
            .map(|index| paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Control(if reverse {
                    specifications.len() - 1 - index
                } else {
                    index
                }),
                affinity: crate::Affinity::After,
            })
            .collect(),
    }
}
fn catalogue() -> System {
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(crate::shaping_test_fonts::width_ahem())
        .unwrap();
    s
}
fn analyzed(value: ParagraphInput) -> AnalyzedSource {
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    )
}
fn prepare(s: &mut System, a: &AnalyzedSource) -> PreparedText {
    s.prepare(a, PreparationOptions::default()).unwrap()
}
fn observations(line: &FinalizedLine) -> Vec<(u32, f32, f32)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), crate::shaping_test_fonts::width_ahem());
            let instance = run.effective_instance().unwrap();
            assert!(instance.face().is_same_face(&run.face()));
            let size = instance.font_size();
            run.glyphs()
                .map(|g| (g.id(), g.advance(), size))
                .collect::<Vec<_>>()
        })
        .collect()
}
fn whole(p: &PreparedText) -> FinalizedLine {
    finish(p, selected_request(p, start(p), end(p), None, false)).unwrap()
}
fn complete(p: &PreparedText, lines: Vec<FinalizedLine>) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 16.0 + 20.0 * i as f64,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap()
}
#[test]
fn selected_source_indication_overlay_retains_real_wbr_twenty_pixel_instance_and_forty_pixel_glyph()
{
    let source = Source::identified("AA\u{ad}A", Id::from_u64(6320), 4);
    let mut capture = input(
        source.clone(),
        &[(WordSpaceTransform::IdeographicSpace, 20.0)],
        false,
    );
    // The generated separator leaves the real short word A-SHY-A. Its native
    // initial 5/2/2 minima suppress the cut, so explicitly admit 2/1/1 here.
    capture.base_style.hyphenate_limit_chars = crate::HyphenateLimitChars {
        total: crate::HyphenationMinimum::Count(2),
        before: crate::HyphenationMinimum::Count(1),
        after: crate::HyphenationMinimum::Count(1),
    };
    let a = analyzed(capture);
    let p = prepare(&mut catalogue(), &a);
    let offer = p
        .opportunity_views()
        .find(|v| {
            v.kind() == PreparedBreakKind::Discretionary
                && v.boundary().view().source_before().get() == 3
        })
        .unwrap()
        .handle();
    let chosen = alternative(&offer);
    assert_eq!(chosen.view().text().code_units(), [0x41]);
    assert_eq!(
        chosen.view().hyphenation_cut().view().counts(),
        Some(counts(1, 1))
    );
    assert!(
        matches!(chosen.view().definition(),crate::PreparedAlternativeDefinition::Source {source,..} if source==Range::new(2,3))
    );
    let first = finish(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
    )
    .unwrap();
    let second = finish(
        &p,
        selected_request(&p, offer.view().boundary(), end(&p), None, false),
    )
    .unwrap();
    assert_eq!(
        first.inner.selected.presentation.projection.text,
        "A\u{3000}AA"
    );
    assert_eq!(first.feedback().advance, 70.0);
    assert_eq!(second.feedback().advance, 10.0);
    let visible: Vec<_> = observations(&first)
        .into_iter()
        .filter(|g| g.1 != 0.0)
        .collect();
    assert_eq!(
        visible,
        [
            (35, 10.0, 10.0),
            (1, 40.0, 20.0),
            (35, 10.0, 10.0),
            (35, 10.0, 10.0)
        ]
    );
    let control = p.control_views().next().unwrap().handle();
    let bounds = first
        .fragment_views()
        .find(|v| matches!(v.owner(),FinalizedFragmentOwner::Control(ref owner) if owner==&control))
        .unwrap()
        .bounds()
        .unwrap();
    assert_eq!(bounds.origin.x, 10.0);
    assert_eq!(bounds.size.width, 40.0);
    let layout = complete(&p, vec![first, second]);
    let positions: Vec<_> = layout.positions().collect();
    let selected = layout
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(selected.source_text(), source.text());
    let origin = p.origin_views().next().unwrap().handle();
    let shy = layout
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    assert_eq!(shy.units().len(), 1);
    assert_eq!(
        shy.units()
            .next()
            .unwrap()
            .pieces()
            .map(|piece| piece.rect().size.width)
            .sum::<f64>(),
        10.0
    );
}
#[test]
fn reused_catalogue_matches_fresh_for_same_analysis_and_changed_control_instance_and_order() {
    let source = Source::identified("A\u{3099}", Id::from_u64(6321), 5);
    let first = analyzed(input(
        source.clone(),
        &[
            (WordSpaceTransform::Space, 10.0),
            (WordSpaceTransform::IdeographicSpace, 20.0),
        ],
        false,
    ));
    let altered = analyzed(input(
        source.clone(),
        &[
            (WordSpaceTransform::Space, 30.0),
            (WordSpaceTransform::IdeographicSpace, 20.0),
        ],
        false,
    ));
    let reversed = analyzed(input(
        source.clone(),
        &[
            (WordSpaceTransform::Space, 10.0),
            (WordSpaceTransform::IdeographicSpace, 20.0),
        ],
        true,
    ));
    let mut warm = catalogue();
    let p1 = prepare(&mut warm, &first);
    let p2 = prepare(&mut warm, &first);
    let c1 = p1.control_views().next().unwrap().handle();
    let c2 = p2.control_views().next().unwrap().handle();
    assert_eq!(c1.view().kind(), c2.view().kind());
    assert_eq!(observations(&whole(&p1)), observations(&whole(&p2)));
    for (a, expected, advance, last_control, mark_width) in [
        (
            &first,
            vec![
                (35, 10.0, 10.0),
                (3, 5.0, 10.0),
                (1, 40.0, 20.0),
                (100, 0.0, 10.0),
            ],
            55.0,
            1,
            40.0,
        ),
        (
            &altered,
            vec![
                (35, 10.0, 10.0),
                (3, 15.0, 30.0),
                (1, 40.0, 20.0),
                (100, 0.0, 10.0),
            ],
            65.0,
            1,
            40.0,
        ),
        (
            &reversed,
            vec![
                (35, 10.0, 10.0),
                (1, 40.0, 20.0),
                (3, 5.0, 10.0),
                (100, 0.0, 10.0),
            ],
            55.0,
            0,
            5.0,
        ),
        (
            &first,
            vec![
                (35, 10.0, 10.0),
                (3, 5.0, 10.0),
                (1, 40.0, 20.0),
                (100, 0.0, 10.0),
            ],
            55.0,
            1,
            40.0,
        ),
    ] {
        let reused = prepare(&mut warm, a);
        let fresh = prepare(&mut catalogue(), a);
        assert_ne!(reused, fresh);
        for p in [reused, fresh] {
            let controls: Vec<_> = p.control_views().map(|v| v.handle()).collect();
            let actual = p
                .control_views()
                .find(|v| match v.kind() {
                    paragraph::ParagraphControlKind::Wbr { style } => {
                        style.word_space_transform
                            == if last_control == 0 {
                                WordSpaceTransform::Space
                            } else {
                                WordSpaceTransform::IdeographicSpace
                            }
                    }
                    _ => false,
                })
                .unwrap()
                .handle();
            let line = whole(&p);
            assert_eq!(line.feedback().advance, advance);
            assert_eq!(observations(&line), expected);
            let origin = p.origin_views().next().unwrap().handle();
            let layout = complete(&p, vec![line]);
            let mark = layout
                .source_range_fragments(&origin, Range::new(1, 2))
                .unwrap();
            let unit = mark.units().next().unwrap();
            assert_eq!(
                unit.controls().collect::<Vec<_>>().as_slice(),
                std::slice::from_ref(&actual)
            );
            assert!(controls.contains(&actual));
            assert_eq!(
                unit.pieces().map(|p| p.rect().size.width).sum::<f64>(),
                mark_width
            );
            let positions: Vec<_> = layout.positions().collect();
            let selected = layout
                .position_selection(positions.first().unwrap(), positions.last().unwrap())
                .unwrap();
            assert_eq!(selected.source_text(), source.text());
        }
    }
}

#[test]
fn unexpanded_wbr_keeps_split_origin_capture_order_and_original_typography_separately() {
    let source = Source::identified("A\u{3099}", Id::from_u64(6322), 6);
    let mut capture = input(source.clone(), &[(WordSpaceTransform::None, 10.0)], false);
    capture.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| paragraph::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: paragraph::ParagraphOriginKind::Authored,
        })
        .collect();
    capture.connections = vec![paragraph::ParagraphConnection {
        at: 1,
        kind: paragraph::ParagraphConnectionKind::Continuous,
    }];
    capture.boundary_policies = vec![paragraph::ParagraphBoundaryPolicy {
        definition: paragraph::ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: crate::Wrap::Word,
        following_wrap: crate::Wrap::Word,
        ancestor_wrap: crate::Wrap::Word,
    }];
    let a = analyzed(capture);
    assert_eq!(a.inner.projection.text, "A\u{3099}");
    assert_eq!(a.inner.projection.graphemes, [0, 2]);
    assert_eq!(a.inner.projection.anchor_atoms, [Some(2)]);
    let p = prepare(&mut catalogue(), &a);
    let origins: Vec<_> = p.origin_views().map(|v| v.handle()).collect();
    let control = p.control_views().next().unwrap().handle();
    let order: Vec<_> = p
        .participants()
        .map(|participant| match participant {
            PreparedParticipant::Text(unit) => {
                let actual: Vec<_> = unit.view().origins().map(|v| v.origin()).collect();
                assert_eq!(actual.len(), 1);
                if actual[0] == origins[0] {
                    0
                } else {
                    assert_eq!(actual[0], origins[1]);
                    2
                }
            }
            PreparedParticipant::Control(actual) => {
                assert_eq!(actual, control);
                1
            }
            PreparedParticipant::Atomic(_) => panic!("capture has no atomic"),
            PreparedParticipant::ComputedLineBoundary(_) => {
                panic!("capture has no computed boundaries")
            }
        })
        .collect();
    assert_eq!(
        order,
        [0, 1, 2],
        "real A/Wbr/mark node order is independent from the projected end-of-EGC anchor"
    );
    assert_eq!(
        observations(&whole(&p)),
        [(35, 10.0, 10.0), (100, 0.0, 10.0)]
    );
}
