//! Independent native endpoint contributions and consumed joining context.
use super::*;

fn block(mut base: t::Style, rtl: bool, break_all: bool) -> t::SourceProcessingScope {
    if break_all {
        base.word_break = t::WordBreak::BreakAll;
    }
    t::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: t::SourceProcessingTopology::Complete,
        trim: t::WhiteSpaceTrim::default(),
        base_style: base,
        used_direction: if rtl {
            t::ParagraphBaseDirection::RightToLeft
        } else {
            t::ParagraphBaseDirection::LeftToRight
        },
    }
}

fn context(p: &t::PreparedText, index: usize, width: f64) -> t::InlineBoundaryContextAllocation {
    t::InlineBoundaryContextAllocation {
        occurrence: p.control_views().nth(index).unwrap().handle(),
        boundary: t::InlineBoundaryAllocation::contribution(width).unwrap(),
    }
}

fn request(
    p: &t::PreparedText,
    start: t::PreparedBoundary,
    end: t::PreparedOpportunity,
    facts: &[(usize, f64)],
    width: f64,
) -> t::LineSelectionRequest {
    let boundary = end.view().boundary();
    let from = start.view().participant_offset();
    let to = boundary.view().participant_offset();
    let participants = p
        .participants()
        .skip(from)
        .take(to - from)
        .collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            t::PreparedParticipant::Control(occurrence) => {
                let index = control_indices(p, std::iter::once(occurrence.clone()))[0];
                Some(t::ControlAllocation {
                    occurrence: occurrence.clone(),
                    strut: None,
                    boundary: facts
                        .iter()
                        .find(|fact| fact.0 == index)
                        .map(|fact| t::InlineBoundaryAllocation::contribution(fact.1).unwrap()),
                })
            }
            _ => None,
        })
        .collect();
    t::LineSelectionRequest {
        inline_boxes: None,
        segment: p.source_segments().next().unwrap(),
        start,
        end: boundary,
        participants,
        termination: t::PreparedTermination::Ordinary(end.clone()),
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_box_struts: Vec::new(),
        line_strut: None,
        context: t::LineContext {
            alignment_insets: surgeist_text::LineAlignmentInsets::ZERO,
            formatted_line: surgeist_text::FormattedLineContext::default(),
            base_direction: if p.analyzed_source().direction() == t::Direction::RightToLeft {
                t::ParagraphBaseDirection::RightToLeft
            } else {
                t::ParagraphBaseDirection::LeftToRight
            },
            line_box_direction: if p.analyzed_source().direction() == t::Direction::RightToLeft {
                surgeist_text::LineBoxDirection::RightToLeft
            } else {
                surgeist_text::LineBoxDirection::LeftToRight
            },
            orientation: t::LineOrientation::Horizontal,
            start: if from == 0 {
                t::LineStart::Paragraph
            } else {
                t::LineStart::Continuation
            },
            end: match end.view().kind() {
                t::PreparedBreakKind::Forced => t::LineEnd::Forced,
                t::PreparedBreakKind::ParagraphEnd => t::LineEnd::Paragraph,
                _ => t::LineEnd::Soft,
            },
            available_inline: Some(width),
            indent: 0.0,
            alignment: t::Alignment::Start,
            tabs: t::TabContext::NoTabs,
            spacing: t::SpacingPolicy::Spaces,
        },
    }
}

fn answer(p: &t::PreparedText, request: &mut t::LineSelectionRequest, facts: &[(usize, f64)]) {
    while let Some(occurrence) = p.boundary_context_demand(request).unwrap() {
        let index = control_indices(p, std::iter::once(occurrence.clone()))[0];
        let width = facts
            .iter()
            .find(|fact| fact.0 == index)
            .expect("fixture supplies this actual demanded endpoint")
            .1;
        request
            .boundary_context
            .push(t::InlineBoundaryContextAllocation {
                occurrence,
                boundary: t::InlineBoundaryAllocation::contribution(width).unwrap(),
            });
    }
}

fn finalize(p: &t::PreparedText, request: t::LineSelectionRequest) -> t::FinalizedLine {
    p.select(request)
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap()
}

fn ids(line: &t::FinalizedLine) -> Vec<(t::Range, u32)> {
    let mut ids = line
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    ids.sort_by_key(|row| (row.0.start, row.0.end));
    ids
}

fn measures(line: &t::FinalizedLine) -> [f64; 5] {
    let values = line.measurements();
    [
        line.feedback().advance,
        values.fit().advance(),
        values.alignment().advance(),
        values.min_content_contribution().advance(),
        values.max_content_contribution().advance(),
    ]
}

#[test]
fn positive_end_owns_its_interval_and_stops_hanging_without_donating_source_geometry() {
    let p = prepare_capture(
        t::Source::new("X \nY"),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 2), (2, 4)],
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
    let forced = p
        .opportunity_views()
        .find(|offer| offer.kind() == t::PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let start = p.boundary_views().next().unwrap().handle();
    let mut zero_request = request(
        &p,
        start.clone(),
        forced.clone(),
        &[(0, 0.0), (1, 0.0)],
        15.0,
    );
    answer(&p, &mut zero_request, &[(0, 0.0), (1, 0.0)]);
    let zero = finalize(&p, zero_request);
    assert_eq!(measures(&zero), [20.0, 10.0, 15.0, 10.0, 20.0]);
    assert_eq!(zero.feedback().hanging_end, 5.0);
    let mut positive_request = request(&p, start, forced.clone(), &[(0, 0.0), (1, 5.0)], 15.0);
    answer(&p, &mut positive_request, &[(0, 0.0), (1, 5.0)]);
    let positive = finalize(&p, positive_request);
    assert_eq!(measures(&positive), [25.0; 5]);
    assert_eq!(positive.feedback().hanging_end, 0.0);
    let end = p.control_views().nth(1).unwrap().handle();
    assert_eq!(
        positive
            .control_slots()
            .find(|slot| slot.control() == end)
            .unwrap()
            .strip_interval(),
        (20.0, 25.0)
    );
    let fragment = positive
        .fragment_views()
        .find(|fragment| fragment.owner() == t::FinalizedFragmentOwner::Control(end.clone()))
        .unwrap();
    assert_eq!(fragment.bounds().unwrap().size.width, 5.0);
    assert!(
        fragment.origin().is_none()
            && fragment.aggregate_range().is_none()
            && fragment.local_range().is_none()
            && fragment.processed_range().is_none()
    );
    let mut last = request(&p, forced.view().boundary(), paragraph_end(&p), &[], 100.0);
    answer(&p, &mut last, &[(0, 0.0), (1, 5.0)]);
    let last = finalize(&p, last);
    let owner = p.origin_views().next().unwrap().handle();
    let layout = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: p.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![
            t::PlacementEntry::Formatted(t::LinePlacement {
                line: positive,
                anchor: t::LineAnchor {
                    inline_origin: 0.0,
                    baseline: 8.0,
                },
                orientation: t::LineOrientation::Horizontal,
                atomics: vec![],
            }),
            t::PlacementEntry::Formatted(t::LinePlacement {
                line: last,
                anchor: t::LineAnchor {
                    inline_origin: 0.0,
                    baseline: 18.0,
                },
                orientation: t::LineOrientation::Horizontal,
                atomics: vec![],
            }),
        ],
    })
    .unwrap();
    let units = layout
        .source_range_fragments(&owner, t::Range::new(1, 2))
        .unwrap();
    assert_eq!(
        units
            .units()
            .flat_map(|unit| unit
                .pieces()
                .map(|piece| {
                    let t::SourceRangeGeometry::Line(piece) = piece else {
                        panic!("ordinary Range fixture must retain Line geometry");
                    };
                    piece.advance_edges()
                })
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [(10.0, 20.0)]
    );
    let t::Hit::Text(left) = layout.hit(t::Point::new(21.0, 5.0)) else {
        panic!("endpoint interval admits an actual placed position")
    };
    let t::Hit::Text(right) = layout.hit(t::Point::new(24.0, 5.0)) else {
        panic!("endpoint's other side admits an actual placed position")
    };
    assert_eq!(
        layout.position_geometry(&left).unwrap().rect().origin.x,
        20.0
    );
    assert_eq!(
        layout.position_geometry(&right).unwrap().rect().origin.x,
        25.0
    );
    let selected = layout.position_selection(&left, &right).unwrap();
    assert!(selected.source_text().is_empty() && selected.plaintext().is_empty());
    assert_eq!(
        selected
            .geometry()
            .rects()
            .iter()
            .map(|rect| rect.rect().size.width)
            .sum::<f64>(),
        5.0
    );
    let before = layout
        .positions()
        .find(|position| position.source_cursor() == t::Cursor::new(0, t::Affinity::After))
        .unwrap();
    let mixed = layout.position_selection(&before, &right).unwrap();
    assert_eq!(mixed.source_text(), &t::Text::from("X "));
    assert_eq!(mixed.plaintext(), &t::Text::from("X "));
    assert_eq!(
        mixed
            .geometry()
            .rects()
            .iter()
            .map(|rect| rect.rect().size.width)
            .sum::<f64>(),
        25.0
    );
}

#[test]
fn a_positive_edge_splits_real_joint_shaping_but_not_an_indivisible_grapheme() {
    let bytes = font::ligature();
    for (value, expected_positive, expected_zero, raw_zero) in [
        (
            "AA",
            vec![(t::Range::new(0, 1), 35), (t::Range::new(1, 2), 35)],
            vec![(t::Range::new(0, 2), 58)],
            10.0,
        ),
        (
            "A\u{301}B",
            vec![(t::Range::new(0, 2), 58), (t::Range::new(2, 3), 36)],
            vec![(t::Range::new(0, 2), 58), (t::Range::new(2, 3), 36)],
            20.0,
        ),
    ] {
        let len = value.encode_utf16().count();
        let p = prepare_capture(
            t::Source::new(value),
            block(style(t::WhiteSpace::Preserve), false, false),
            &[(0, 1), (1, len)],
            &[(K::InlineBoundaryStart, 1), (K::InlineBoundaryEnd, len)],
            &[(0, 1)],
            vec![
                origin(0),
                E::Connection(0),
                E::EnterScope(1),
                control(0),
                origin(1),
                control(1),
                E::ExitScope(1),
            ],
            &bytes,
        );
        let start = p.boundary_views().next().unwrap().handle();
        let zero = finalize(
            &p,
            request(
                &p,
                start.clone(),
                paragraph_end(&p),
                &[(0, 0.0), (1, 0.0)],
                100.0,
            ),
        );
        let positive = finalize(
            &p,
            request(&p, start, paragraph_end(&p), &[(0, 5.0), (1, 0.0)], 100.0),
        );
        assert_eq!(ids(&zero), expected_zero);
        assert_eq!(ids(&positive), expected_positive);
        assert_eq!(zero.feedback().advance, raw_zero);
        assert_eq!(positive.feedback().advance, 25.0);
        if len == 3 {
            let pens = |line: &t::FinalizedLine| {
                line.run_views()
                    .flat_map(|run| {
                        run.glyphs()
                            .map(|glyph| {
                                (
                                    glyph.id(),
                                    run.origin().x + glyph.pen() + f64::from(glyph.x()),
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(pens(&zero), [(58, 0.0), (36, 10.0)]);
            assert_eq!(pens(&positive), [(58, 0.0), (36, 15.0)]);
        }
    }
}

#[test]
fn consumed_off_line_endpoint_context_changes_joining_without_allocating_its_width() {
    for rtl in [false, true] {
        for first_size in [10.0, 20.0] {
            let bytes = font::arabic();
            let mut source = t::Source::new("\u{628}\u{628}\u{628}");
            let mut first_style = style(t::WhiteSpace::Preserve);
            first_style.word_break = t::WordBreak::BreakAll;
            first_style.size = first_size;
            source.span(t::Range::new(0, 1), first_style);
            let p = prepare_capture(
                source,
                block(style(t::WhiteSpace::Preserve), rtl, true),
                &[(0, 1), (1, 2), (2, 3)],
                &[(K::InlineBoundaryStart, 1), (K::InlineBoundaryEnd, 2)],
                &[(0, 1)],
                vec![
                    origin(0),
                    E::Connection(0),
                    E::EnterScope(1),
                    control(0),
                    origin(1),
                    control(1),
                    E::ExitScope(1),
                    E::Connection(1),
                    origin(2),
                ],
                &bytes,
            );
            let offers = soft_at(&p, 1);
            assert_eq!(offers.len(), 1);
            let base = request(
                &p,
                p.boundary_views().next().unwrap().handle(),
                offers[0].clone(),
                &[],
                100.0,
            );
            assert!(base.controls.is_empty());
            assert_eq!(
                p.boundary_context_demand(&base).unwrap(),
                Some(p.control_views().next().unwrap().handle())
            );
            let mut positive_request = base.clone();
            positive_request.boundary_context.push(context(&p, 0, 5.0));
            assert!(
                p.boundary_context_demand(&positive_request)
                    .unwrap()
                    .is_none()
            );
            let positive = finalize(&p, positive_request.clone());
            assert_eq!(ids(&positive), [(t::Range::new(0, 1), 35)]);
            assert_eq!(positive.feedback().advance, f64::from(first_size));
            assert_eq!(positive.control_slots().count(), 0);
            assert!(positive.fragment_views().all(|fragment| !matches!(
                fragment.owner(),
                t::FinalizedFragmentOwner::Control(_)
            )));
            let mut zero_request = base.clone();
            zero_request.boundary_context.push(context(&p, 0, 0.0));
            assert_eq!(
                p.boundary_context_demand(&zero_request).unwrap(),
                Some(p.control_views().nth(1).unwrap().handle())
            );
            let error = p.select(zero_request.clone()).unwrap_err();
            assert!(matches!(
                error.detail(),
                Some(t::ErrorDetail::InvalidTextProtocol {
                    reason: t::ProtocolError::MissingBoundaryContext { occurrence: 1 }
                })
            ));
            zero_request.boundary_context.push(context(&p, 1, 0.0));
            assert!(p.boundary_context_demand(&zero_request).unwrap().is_none());
            let zero = finalize(&p, zero_request);
            assert_eq!(ids(&zero), [(t::Range::new(0, 1), 100)]);
            assert_eq!(zero.feedback().advance, f64::from(first_size));
            // Actual canonical sizes distinguish the instances; neither side is
            // a metric-only attestation. Both execute the same qualified font face.
            for outgoing in [&zero, &positive] {
                for run in outgoing.run_views() {
                    assert_eq!(run.face().bytes(), bytes);
                    let instance = run.effective_instance().unwrap();
                    assert!(instance.face().is_same_face(&run.face()));
                    assert_eq!(instance.axis_count(), 0);
                    assert_eq!(instance.font_size(), first_size);
                    assert_eq!(run.font_size(), first_size);
                    assert!(run.glyphs().all(|glyph| glyph.advance() == first_size));
                }
            }
            for (edge, first_form) in [(0.0, 277), (5.0, 100)] {
                let mut receiving = request(
                    &p,
                    offers[0].view().boundary(),
                    paragraph_end(&p),
                    &[(0, edge), (1, 0.0)],
                    100.0,
                );
                answer(&p, &mut receiving, &[(0, edge), (1, 0.0)]);
                let receiving = finalize(&p, receiving);
                assert_eq!(
                    ids(&receiving),
                    [(t::Range::new(1, 2), first_form), (t::Range::new(2, 3), 82)]
                );
                assert_eq!(receiving.feedback().advance, 20.0 + edge);
                let opening = receiving
                    .control_slots()
                    .find(|slot| slot.control() == p.control_views().next().unwrap().handle())
                    .unwrap();
                let (left, right) = opening.strip_interval();
                assert_eq!(right - left, edge);
                for run in receiving.run_views() {
                    assert_eq!(run.face().bytes(), bytes);
                    let instance = run.effective_instance().unwrap();
                    assert!(instance.face().is_same_face(&run.face()));
                    assert_eq!(instance.axis_count(), 0);
                    assert_eq!(instance.font_size(), 10.0);
                    assert_eq!(run.font_size(), 10.0);
                    assert!(run.glyphs().all(|glyph| glyph.advance() == 10.0));
                }
            }
            let mut surplus = positive_request.clone();
            surplus.boundary_context.push(context(&p, 1, 0.0));
            assert!(matches!(
                p.select(surplus).unwrap_err().detail(),
                Some(t::ErrorDetail::InvalidTextProtocol {
                    reason: t::ProtocolError::UnexpectedBoundaryContext { occurrence: 1 }
                })
            ));
            let mut duplicate = positive_request;
            duplicate.boundary_context.push(context(&p, 0, 5.0));
            assert!(matches!(
                p.select(duplicate).unwrap_err().detail(),
                Some(t::ErrorDetail::InvalidTextProtocol {
                    reason: t::ProtocolError::DuplicateBoundaryContext { occurrence: 0 }
                })
            ));
            assert_eq!(ids(&zero), [(t::Range::new(0, 1), 100)]);
            assert_eq!(ids(&positive), [(t::Range::new(0, 1), 35)]);
        }
    }
}

#[test]
fn source_indication_preserves_bounded_original_forms_and_selected_presentation_forms() {
    // Five joining letters satisfy the native default whole/before/after
    // hyphenation counts. The actual SHY follows the first two letters.
    let value = "\u{628}\u{628}\u{ad}\u{628}\u{628}\u{628}";
    let bytes = font::arabic();
    for indication in ["", "\u{628}"] {
        let mut base = style(t::WhiteSpace::Preserve);
        base.hyphens = t::Hyphens::Manual;
        base.hyphenate_character = t::HyphenateCharacter::String(t::Text::from(indication));
        let p = prepare_capture(
            t::Source::new(value),
            block(base, false, false),
            &[(0, 3), (3, 4), (4, 6)],
            &[(K::InlineBoundaryStart, 3), (K::InlineBoundaryEnd, 4)],
            &[(0, 1)],
            vec![
                origin(0),
                E::Connection(0),
                E::EnterScope(1),
                control(0),
                origin(1),
                control(1),
                E::ExitScope(1),
                E::Connection(1),
                origin(2),
            ],
            &bytes,
        );
        let end = p
            .opportunity_views()
            .find(|offer| {
                offer.kind() == t::PreparedBreakKind::Discretionary
                    && offer.boundary().view().source_after().get() == 3
            })
            .expect("actual source SHY has an admitted indication")
            .handle();
        let alternative = end.view().alternatives().next().unwrap().handle();
        assert_eq!(alternative.view().text(), &t::Text::from(indication));
        assert_eq!(alternative.view().source_range(), Some(t::Range::new(2, 3)));
        let mut zero_request = request(
            &p,
            p.boundary_views().next().unwrap().handle(),
            end.clone(),
            &[],
            100.0,
        );
        zero_request.alternative = Some(alternative.clone());
        answer(&p, &mut zero_request, &[(0, 0.0), (1, 0.0)]);
        let mut positive_request = request(
            &p,
            p.boundary_views().next().unwrap().handle(),
            end.clone(),
            &[],
            100.0,
        );
        positive_request.alternative = Some(alternative);
        answer(&p, &mut positive_request, &[(0, 5.0)]);
        assert_eq!(zero_request.controls, positive_request.controls);
        assert!(positive_request.controls.is_empty());
        let zero = finalize(&p, zero_request);
        let positive = finalize(&p, positive_request);
        let originals = |line: &t::FinalizedLine| {
            ids(line)
                .into_iter()
                .filter(|row| !row.0.is_empty())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            originals(&zero),
            [(t::Range::new(0, 1), 100), (t::Range::new(1, 2), 277)]
        );
        assert_eq!(
            originals(&positive),
            [(t::Range::new(0, 1), 100), (t::Range::new(1, 2), 82)]
        );
        let computed = |line: &t::FinalizedLine| {
            ids(line)
                .into_iter()
                .filter(|row| row.0.is_empty())
                .collect::<Vec<_>>()
        };
        let expected = if indication.is_empty() {
            vec![]
        } else {
            vec![(t::Range::new(3, 3), 82)]
        };
        // Computed joining sees only the selected presentation; it never
        // borrows the unselected original letters after this line.
        assert_eq!(computed(&zero), expected);
        assert_eq!(computed(&positive), expected);
        let raw = if indication.is_empty() { 20.0 } else { 30.0 };
        assert_eq!(zero.feedback().advance, raw);
        assert_eq!(positive.feedback().advance, raw);
        assert_eq!(positive.control_slots().count(), 0);
        let mut receiving = request(
            &p,
            end.view().boundary(),
            paragraph_end(&p),
            &[(0, 5.0), (1, 0.0)],
            100.0,
        );
        answer(&p, &mut receiving, &[(0, 5.0), (1, 0.0)]);
        let receiving = finalize(&p, receiving);
        assert_eq!(receiving.feedback().advance, 35.0);
        assert_eq!(p.analyzed_source().source().text(), &t::Text::from(value));
        assert_eq!(
            originals(&zero),
            [(t::Range::new(0, 1), 100), (t::Range::new(1, 2), 277)]
        );
    }
}

#[test]
fn endpoint_allocation_and_selected_context_roles_refuse_invalid_facts() {
    assert_eq!(
        t::InlineBoundaryAllocation::contribution(-0.0)
            .unwrap()
            .advance()
            .to_bits(),
        0.0_f64.to_bits()
    );
    for (value, category) in [
        (-1.0, t::TextNumericError::OutOfRange),
        (f64::INFINITY, t::TextNumericError::NonFinite),
        (f64::NAN, t::TextNumericError::NonFinite),
    ] {
        let error = t::InlineBoundaryAllocation::contribution(value).unwrap_err();
        assert!(
            matches!(error.detail(), Some(t::ErrorDetail::InvalidTextNumeric { phase: t::TextNumericPhase::Selection, category: actual, .. }) if *actual == category)
        );
    }
    let p = closing("X Y");
    let start = p.boundary_views().next().unwrap().handle();
    let complete = request(
        &p,
        start.clone(),
        paragraph_end(&p),
        &[(0, 0.0), (1, 5.0)],
        100.0,
    );
    let good = finalize(&p, complete.clone());
    let mut absent = complete.clone();
    absent.controls[0].boundary = None;
    assert!(matches!(
        p.select(absent).unwrap_err().detail(),
        Some(t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::MissingControlBoundaryGeometry { occurrence: 0 }
        })
    ));
    let mut overlap = complete;
    overlap.boundary_context.push(context(&p, 0, 0.0));
    assert!(matches!(
        p.select(overlap).unwrap_err().detail(),
        Some(t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::DuplicateBoundaryContext { occurrence: 0 }
        })
    ));
    let foreign = closing("X Y");
    let first_end = soft_at(&p, 2).pop().unwrap();
    let mut foreign_request = request(&p, start, first_end, &[(0, 0.0), (1, 5.0)], 100.0);
    foreign_request
        .boundary_context
        .push(context(&foreign, 0, 0.0));
    assert!(matches!(
        p.select(foreign_request).unwrap_err().detail(),
        Some(t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::ForeignPreparation { .. }
        })
    ));
    assert_eq!(good.feedback().advance, 35.0);
}

#[test]
fn an_incoming_after_indication_stays_on_its_authentic_side_of_the_opening_edge() {
    let bytes = font::arabic();
    let value = "\u{628}\u{628}\u{ad}\u{628}\u{628}\u{628}";
    let mut base = style(t::WhiteSpace::Preserve);
    base.hyphens = t::Hyphens::Manual;
    base.locale = Some("und-Arab".into());
    base.native_hyphenation_convention = Some(
        t::NativeHyphenationConvention::language_script(
            "und-Arab",
            "Arab",
            t::Text::from("\u{628}"),
            t::HyphenationPlacement::LogicalAfter,
        )
        .unwrap(),
    );
    let p = prepare_capture(
        t::Source::new(value),
        block(base, false, false),
        &[(0, 3), (3, 4), (4, 6)],
        &[(K::InlineBoundaryStart, 3), (K::InlineBoundaryEnd, 4)],
        &[(0, 1)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            origin(1),
            control(1),
            E::ExitScope(1),
            E::Connection(1),
            origin(2),
        ],
        &bytes,
    );
    let ending = p
        .opportunity_views()
        .find(|offer| offer.kind() == t::PreparedBreakKind::Discretionary)
        .unwrap()
        .handle();
    let alternative = ending.view().alternatives().next().unwrap().handle();
    // The existing After contract attaches to the actual receiving text,
    // after its leading opening endpoint, rather than to the source SHY.
    // The computed prefix therefore still joins its selected receiver.
    for (edge, computed, first_original, receiving_raw) in
        [(0.0, 100, 277, 40.0), (5.0, 100, 100, 45.0)]
    {
        let mut outgoing = request(
            &p,
            p.boundary_views().next().unwrap().handle(),
            ending.clone(),
            &[],
            100.0,
        );
        outgoing.alternative = Some(alternative.clone());
        answer(&p, &mut outgoing, &[(0, edge), (1, 0.0)]);
        let outgoing = finalize(&p, outgoing);
        assert_eq!(outgoing.feedback().advance, 20.0);
        assert_eq!(outgoing.control_slots().count(), 0);
        let mut receiving = request(
            &p,
            ending.view().boundary(),
            paragraph_end(&p),
            &[(0, edge), (1, 0.0)],
            100.0,
        );
        receiving.incoming = Some(
            outgoing
                .hyphenation_continuation()
                .expect("nonempty After emission is genuinely owed"),
        );
        answer(&p, &mut receiving, &[(0, edge), (1, 0.0)]);
        let receiving = finalize(&p, receiving);
        assert_eq!(
            ids(&receiving),
            [
                (t::Range::new(3, 3), computed),
                (t::Range::new(3, 4), first_original),
                (t::Range::new(4, 5), 277),
                (t::Range::new(5, 6), 82),
            ]
        );
        assert_eq!(receiving.feedback().advance, receiving_raw);
        let slot = receiving
            .control_slots()
            .find(|slot| slot.control() == p.control_views().next().unwrap().handle())
            .unwrap();
        let (left, right) = slot.strip_interval();
        assert_eq!(right - left, edge);
        if edge > 0.0 {
            for run in receiving.run_views() {
                for glyph in run.glyphs().filter(|glyph| {
                    glyph.range() == t::Range::new(3, 3) || glyph.range() == t::Range::new(3, 4)
                }) {
                    let x = run.origin().x + glyph.pen() + f64::from(glyph.x());
                    if slot.level().is_rtl() {
                        assert!(
                            x + f64::from(glyph.advance()) <= left,
                            "range={:?} glyph={} x={x} advance={} endpoint={left}..{right} level={:?}",
                            glyph.range(),
                            glyph.id(),
                            glyph.advance(),
                            slot.level()
                        );
                    } else {
                        assert!(
                            x >= right,
                            "range={:?} glyph={} x={x} advance={} endpoint={left}..{right} level={:?}",
                            glyph.range(),
                            glyph.id(),
                            glyph.advance(),
                            slot.level()
                        );
                    }
                }
            }
        }
        assert_eq!(p.analyzed_source().source().text(), &t::Text::from(value));
    }
}

#[test]
fn incoming_after_retains_two_nested_opening_edges_in_ltr_and_rtl_blocks() {
    // Strong Latin originals resolve at level 0 in an LTR block and level 2
    // in an RTL block. All four endpoint anchors have After affinity to those
    // originals, so this fixture's actual L2 sequence remains left-to-right.
    for rtl in [false, true] {
        let mut base = style(t::WhiteSpace::Preserve);
        base.hyphens = t::Hyphens::Manual;
        base.locale = Some("und-Latn".into());
        base.native_hyphenation_convention = Some(
            t::NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                t::Text::from("A"),
                t::HyphenationPlacement::LogicalAfter,
            )
            .unwrap(),
        );
        let p = prepare_capture(
            t::Source::new("AA\u{ad}AAA"),
            block(base, rtl, false),
            &[(0, 3), (3, 4), (4, 6)],
            &[
                (K::InlineBoundaryStart, 3),
                (K::InlineBoundaryStart, 3),
                (K::InlineBoundaryEnd, 4),
                (K::InlineBoundaryEnd, 4),
            ],
            &[(0, 3), (1, 2)],
            vec![
                origin(0),
                E::Connection(0),
                E::EnterScope(1),
                control(0),
                E::EnterScope(2),
                control(1),
                origin(1),
                control(2),
                E::ExitScope(2),
                control(3),
                E::ExitScope(1),
                E::Connection(1),
                origin(2),
            ],
            AHEM,
        );
        let level = if rtl { 2 } else { 0 };
        assert!(p.control_views().all(|view| view.level().get() == level));
        let ending = p
            .opportunity_views()
            .find(|offer| offer.kind() == t::PreparedBreakKind::Discretionary)
            .unwrap()
            .handle();
        let alternative = ending.view().alternatives().next().unwrap().handle();
        for (outer, inner) in [(0.0, 0.0), (2.0, 3.0)] {
            let facts = [(0, outer), (1, inner), (2, 0.0), (3, 0.0)];
            let mut outgoing = request(
                &p,
                p.boundary_views().next().unwrap().handle(),
                ending.clone(),
                &[],
                100.0,
            );
            outgoing.alternative = Some(alternative.clone());
            answer(&p, &mut outgoing, &facts);
            let outgoing = finalize(&p, outgoing);
            assert_eq!(outgoing.feedback().advance, 20.0);
            let mut receiving = request(
                &p,
                ending.view().boundary(),
                paragraph_end(&p),
                &facts,
                100.0,
            );
            receiving.incoming = Some(outgoing.hyphenation_continuation().unwrap());
            answer(&p, &mut receiving, &facts);
            let receiving = finalize(&p, receiving);
            assert_eq!(receiving.feedback().advance, 40.0 + outer + inner);
            let slots = receiving.control_slots().collect::<Vec<_>>();
            assert_eq!(slots.len(), 4);
            assert_eq!(
                control_indices(&p, slots.iter().map(|s| s.control())),
                [0, 1, 2, 3]
            );
            assert!(slots.iter().all(|slot| slot.level().get() == level));
            let (left, outer_end) = slots[0].strip_interval();
            let (inner_start, inner_end) = slots[1].strip_interval();
            assert_eq!(outer_end - left, outer);
            assert_eq!(inner_start, outer_end);
            assert_eq!(inner_end - inner_start, inner);
            let mut positions = Vec::new();
            for run in receiving.run_views() {
                assert_eq!(run.face().bytes(), AHEM);
                for glyph in run.glyphs() {
                    assert_eq!(glyph.id(), 35);
                    assert_eq!(glyph.advance(), 10.0);
                    positions.push((
                        glyph.range(),
                        run.origin().x + glyph.pen() + f64::from(glyph.x()),
                    ));
                }
            }
            positions.sort_by_key(|row| (row.0.start, row.0.end));
            assert_eq!(
                positions,
                [
                    (t::Range::new(3, 3), inner_end),
                    (t::Range::new(3, 4), inner_end + 10.0),
                    (t::Range::new(4, 5), inner_end + 20.0),
                    (t::Range::new(5, 6), inner_end + 30.0),
                ]
            );
            assert_eq!(
                slots[2].strip_interval(),
                (inner_end + 20.0, inner_end + 20.0)
            );
            assert_eq!(slots[3].strip_interval(), slots[2].strip_interval());
            assert_eq!(
                p.analyzed_source().source().text(),
                &t::Text::from("AA\u{ad}AAA")
            );
        }
    }
}

#[test]
fn an_empty_original_occurrence_does_not_hide_the_actual_closing_endpoint() {
    let p = capture(
        "X Y",
        t::WhiteSpace::Preserve,
        &[(0, 2), (2, 2), (2, 3)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 2)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            E::Connection(0),
            origin(1),
            control(1),
            E::ExitScope(1),
            E::Connection(1),
            origin(2),
        ],
    );
    let observed = soft_at(&p, 2)
        .into_iter()
        .map(|offer| observe(&p, offer))
        .collect::<Vec<_>>();
    assert!(
        observed.iter().all(|cut| cut.outgoing_controls == [0, 1]
            && cut.outgoing_fragments == [0, 1]
            && cut.receiving_controls.is_empty()
            && cut.receiving_fragments.is_empty()),
        "real empty source metadata is transparent to closing custody: {observed:?}"
    );
}

#[path = "signed_source_consumers.rs"]
mod signed_source_consumers;

#[path = "geometry/signed_shaping.rs"]
mod signed_shaping;

#[path = "bidi_fragmentation_baseline.rs"]
mod bidi_fragmentation_baseline;

#[path = "family_admission_negative.rs"]
mod family_admission_negative;

#[path = "geometry/shared_packet_membership.rs"]
mod shared_packet_membership;

#[path = "deleted_only_box_family.rs"]
mod deleted_only_box_family;

#[path = "native_endpoint_uniqueness.rs"]
mod native_endpoint_uniqueness;
