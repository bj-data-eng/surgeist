use super::*;
use crate::*;

fn prepare(bytes: Vec<u8>, source: &str, style: Style) -> PreparedText {
    let analyzed = AnalyzedSource::new(Source::new(source), style).unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}
fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 12.5,
        line_height: LineHeight::Absolute(16.0),
        ..Style::default()
    }
}
fn selected_end(
    prepared: &PreparedText,
    termination: PreparedOpportunity,
    alternative: Option<PreparedAlternative>,
) -> SelectedLine {
    let end = termination.view().boundary();
    let count = end.view().participant_offset();
    let participants: Vec<_> = prepared.participants().take(count).collect();
    let controls = participants
        .iter()
        .filter_map(|p| match p {
            PreparedParticipant::Control(control) => Some(ControlAllocation {
                occurrence: control.clone(),
                strut: None,
            }),
            _ => None,
        })
        .collect();
    prepared
        .select(LineSelectionRequest {
            start: prepared.boundary_views().next().unwrap().handle(),
            end,
            termination: crate::PreparedTermination::Ordinary(termination.clone()),
            participants,
            alternative,
            context: LineContext {
                base_direction: if prepared.analyzed_source().direction() == Direction::RightToLeft
                {
                    ParagraphBaseDirection::RightToLeft
                } else {
                    ParagraphBaseDirection::LeftToRight
                },
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end: match termination.view().kind() {
                    PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                    PreparedBreakKind::Forced => LineEnd::Forced,
                    _ => LineEnd::Soft,
                },
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                hanging: HangingContext::None,
                spacing: SpacingPolicy::Spaces,
            },
            atomics: vec![],
            controls,
            line_strut: None,
        })
        .unwrap()
}
fn whole(prepared: &PreparedText) -> FinalizedLine {
    let termination = prepared
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    selected_end(prepared, termination, None)
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn native(
    source: &str,
    style: Style,
    controls: Vec<ParagraphControl>,
    replacements: Vec<ParagraphReplacement>,
) -> PreparedText {
    let source = Source::new(source);
    let len = source.text().len();
    let mut anchor_order: Vec<_> = (0..controls.len())
        .map(|i| ParagraphAnchor {
            item: ParagraphAnchorItem::Control(i),
            affinity: Affinity::Before,
        })
        .collect();
    if len == 0 {
        anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::EmptyOrigin(0),
            affinity: Affinity::Before,
        });
    }
    let analyzed = AnalyzedSource::from_paragraph(ParagraphInput {
        tab_reference: None,
        source: source.clone(),
        base_style: style,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![ParagraphOrigin {
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls,
        boundary_policies: vec![],
        replacements,
        anchor_order,
    })
    .unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::ligature_ahem())
        .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word);
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}
fn seam(prepared: &PreparedText, at: usize) -> TextSeam {
    let origin = prepared.origin_views().next().unwrap().handle();
    let local = SourcePosition::try_new(origin.view().source().text(), at).unwrap();
    TextSeam::Origin { origin, local }
}
#[test]
fn finalized_prefix_counts_whole_real_ligature_and_combining_unit() {
    for (font, text) in [
        (crate::shaping_test_fonts::ligature_ahem(), "AA"),
        (crate::shaping_test_fonts::mark_ahem(), "A\u{301}"),
    ] {
        let prepared = prepare(font, text, style());
        let line = whole(&prepared);
        assert_eq!(line.feedback().advance, 12.5);
        let glyphs: Vec<_> = line
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.range()).collect::<Vec<_>>())
            .collect();
        assert_eq!(glyphs[0], Range::new(0, 2));
        for (at, expected, interior) in [(0, 0.0, false), (1, 12.5, true), (2, 12.5, false)] {
            let query = seam(&prepared, at);
            let observed = line.prefix_at(&query).unwrap();
            assert_eq!(observed.seam(), &query);
            assert_eq!(observed.advance(), expected);
            assert_eq!(observed.interior_unit().is_some(), interior);
            if let Some(unit) = observed.interior_unit() {
                assert_eq!(unit.source_range(), Range::new(0, 2));
                assert_eq!(unit.advance().unwrap(), 12.5);
            }
        }
        // Independently queried events share one text prefix, not two halves.
        assert_eq!(
            line.prefix_at(&seam(&prepared, 1)).unwrap().advance(),
            line.prefix_at(&seam(&prepared, 1)).unwrap().advance()
        );
        let foreign = prepare(crate::shaping_test_fonts::ligature_ahem(), "AA", style());
        assert!(line.prefix_at(&seam(&foreign, 1)).is_err());
        let observation = {
            let transient = seam(&prepared, 1);
            line.prefix_at(&transient).unwrap()
        };
        assert_eq!(observation.finalized_line(), &line);
        assert_eq!(observation.advance(), 12.5);
        let outside = Source::new("ZZZ");
        let invalid = TextSeam::Origin {
            origin: prepared.origin_views().next().unwrap().handle(),
            local: SourcePosition::try_new(outside.text(), 3).unwrap(),
        };
        assert!(matches!(
            line.prefix_at(&invalid).unwrap_err().detail(),
            Some(ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::InvalidOriginSeam { position: 3 }
            })
        ));
    }
}
#[test]
fn finalized_prefix_retains_actual_signed_zero_and_numeric_failure_facts() {
    let font = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec();
    for (spacing, expected) in [(-12.5, 0.0), (-15.0, -2.5)] {
        let mut input = style();
        input.letter_spacing = spacing;
        let prepared = prepare(font.clone(), "AX", input);
        let line = whole(&prepared);
        assert_eq!(
            line.prefix_at(&seam(&prepared, 1)).unwrap().advance(),
            expected
        );
    }
    // These are actual checked contribution operations, independent of a
    // successful cumulative receipt with a cancelling preceding item.
    let bad = Contribution::Value(-f64::MAX).add(-f64::MAX);
    assert!(matches!(
        bad.value().unwrap_err().detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            category: TextNumericError::NonFinite,
            ..
        })
    ));
    assert_eq!(Contribution::Value(0.0).add(12.5).value().unwrap(), 12.5);
}
#[test]
fn finalized_prefix_is_logical_for_actual_joined_arabic() {
    let prepared = prepare(
        crate::shaping_test_fonts::arabic_ahem(),
        "\u{628}\u{627}",
        style(),
    );
    let line = whole(&prepared);
    assert_eq!(line.feedback().advance, 25.0);
    assert_eq!(line.prefix_at(&seam(&prepared, 1)).unwrap().advance(), 12.5);
    assert_eq!(line.prefix_at(&seam(&prepared, 2)).unwrap().advance(), 25.0);
}

#[test]
fn finalized_prefix_provider_groups_survive_actual_resource_instance_and_plan_cache_hits() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::ligature_ahem())
        .unwrap();
    let analyzed = AnalyzedSource::new(Source::new("AA"), style()).unwrap();
    let cold = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let before = system.shape_cache_hits();
    let warm = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let after = system.shape_cache_hits();
    for (old, new) in before.into_iter().zip(after) {
        assert!(
            new > old,
            "actual shape resource/instance/plan lookup must hit"
        );
    }
    for prepared in [&cold, &warm] {
        let line = whole(prepared);
        assert_eq!(
            prepared
                .inner
                .shaped
                .clusters
                .iter()
                .map(|c| c.provider_group.clone())
                .collect::<Vec<_>>(),
            vec![0..2, 0..2]
        );
        let at = seam(prepared, 1);
        let observation = line.prefix_at(&at).unwrap();
        assert_eq!(observation.advance(), 12.5);
        assert_eq!(
            observation.interior_unit().unwrap().source_range(),
            Range::new(0, 2)
        );
    }
}
#[test]
fn finalized_prefix_optional_unit_failure_does_not_poison_other_actual_queries() {
    // Default-ignorable glyphs have genuine zero provider advances. Paint
    // styles split only the finalized runs; the two ZWJs remain one actual
    // grapheme while their provider clusters are individually retained.
    let mut source = Source::new("\u{200B}\u{200D}\u{200D}");
    let mut leading = style();
    leading.letter_spacing = -f64::MAX;
    let mut joined = style();
    joined.letter_spacing = f64::MAX;
    let mut last = joined.clone();
    last.brush = Brush::color(1.0, 0.0, 0.0, 1.0);
    source.span(Range::new(0, 1), leading);
    source.span(Range::new(1, 2), joined);
    source.span(Range::new(2, 3), last);
    let analyzed = AnalyzedSource::new(source, style()).unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::mark_ahem())
        .unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let line = whole(&prepared);
    assert!(
        line.run_views()
            .flat_map(|r| r.glyphs().map(|g| g.advance()).collect::<Vec<_>>())
            .all(|advance| advance == 0.0)
    );
    assert!(
        line.inner
            .data
            .prefix
            .groups
            .iter()
            .any(|group| group.source == Range::new(1, 3))
    );
    assert_eq!(line.prefix_at(&seam(&prepared, 0)).unwrap().advance(), 0.0);
    assert_eq!(
        line.prefix_at(&seam(&prepared, 1)).unwrap().advance(),
        -f64::MAX
    );
    assert!(matches!(
        line.prefix_at(&seam(&prepared, 2)).unwrap_err().detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            category: TextNumericError::NonFinite,
            ..
        })
    ));
    assert!(line.prefix_at(&seam(&prepared, 3)).is_ok());
}

#[test]
fn finalized_prefix_observes_real_edge_discard_deletion_and_equal_offset_controls() {
    for mode in [WhiteSpace::Collapse, WhiteSpace::Discard] {
        let mut input = style();
        input.white_space = mode;
        let prepared = native("  X X  ", input, vec![], vec![]);
        let line = whole(&prepared);
        let gap = if mode == WhiteSpace::Collapse {
            12.5
        } else {
            0.0
        };
        assert_eq!(line.feedback().advance, 25.0 + gap);
        for (at, expected) in [
            (0, 0.0),
            (1, 0.0),
            (2, 0.0),
            (3, 12.5),
            (4, 12.5 + gap),
            (5, 25.0 + gap),
            (6, 25.0 + gap),
            (7, 25.0 + gap),
        ] {
            assert_eq!(
                line.prefix_at(&seam(&prepared, at)).unwrap().advance(),
                expected,
                "{mode:?} at{at}"
            );
        }
    }
    let controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::LeftToRightEmbedding,
        },
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::PopDirectionalFormat,
        },
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    let prepared = native("X", style(), controls, vec![]);
    let line = whole(&prepared);
    let handles: Vec<_> = prepared.control_views().map(|v| v.handle()).collect();
    assert_eq!(handles.len(), 4);
    for handle in handles {
        for at in [
            TextSeam::BeforeParticipant(PreparedParticipant::Control(handle.clone())),
            TextSeam::AfterParticipant(PreparedParticipant::Control(handle)),
        ] {
            let observed = line.prefix_at(&at).unwrap();
            assert_eq!(observed.seam(), &at);
            assert_eq!(observed.advance(), 0.0);
        }
    }
    let empty = native("", style(), vec![], vec![]);
    let line = whole(&empty);
    assert_eq!(line.prefix_at(&seam(&empty, 0)).unwrap().advance(), 0.0);
}
#[test]
fn finalized_prefix_replacement_is_bundled_with_its_real_defining_control() {
    let prepared = native(
        "AX",
        style(),
        vec![ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(style()),
            },
        }],
        vec![
            ParagraphReplacement::resolved_native_logical_before(
                ParagraphReplacementDefinition::Control {
                    control: 0,
                    origin: None,
                },
                1,
                vec![Text::from(" X ")],
            )
            .unwrap(),
        ],
    );
    let alternative = prepared.alternative_views().next().unwrap().handle();
    let selected = selected_end(
        &prepared,
        alternative.view().opportunity(),
        Some(alternative.clone()),
    );
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    let control = prepared.control_views().next().unwrap().handle();
    let before = TextSeam::BeforeParticipant(PreparedParticipant::Control(control.clone()));
    let after = TextSeam::AfterParticipant(PreparedParticipant::Control(control));
    assert_eq!(line.prefix_at(&before).unwrap().advance(), 12.5);
    assert_eq!(line.prefix_at(&after).unwrap().advance(), 50.0);
    let rows: Vec<_> = line
        .inner
        .data
        .prefix
        .groups
        .iter()
        .filter(|g| g.source == Range::new(1, 1))
        .collect();
    assert_eq!(rows.len(), 3);
    for row in rows {
        assert!(matches!(
            line.inner.selected.presentation.units[row.units.start].kind,
            PresentationUnitKind::Replacement { .. }
        ));
    }
    // A future original unit is genuinely outside this selected candidate.
    assert!(matches!(
        line.prefix_at(&seam(&prepared, 2)).unwrap_err().detail(),
        Some(ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::OutsideFinalizedLine
        })
    ));
}
#[test]
fn finalized_prefix_break_reshaping_replaces_the_actual_shared_association() {
    let mut input = style();
    input.word_break = WordBreak::BreakAll;
    let prepared = native("AA", input, vec![], vec![]);
    let whole = whole(&prepared);
    let query = seam(&prepared, 1);
    assert!(whole.prefix_at(&query).unwrap().interior_unit().is_some());
    let termination = prepared
        .opportunity_views()
        .find(|v| {
            v.kind() == PreparedBreakKind::Allowed && v.boundary().view().source_after().get() == 1
        })
        .unwrap()
        .handle();
    let line = selected_end(&prepared, termination, None)
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 12.5);
    assert!(line.prefix_at(&query).unwrap().interior_unit().is_none());
    assert_eq!(
        line.run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec![35]
    );
    assert_eq!(
        whole
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec![58]
    );
    assert_eq!(
        whole
            .prefix_at(&query)
            .unwrap()
            .interior_unit()
            .unwrap()
            .source_range(),
        Range::new(0, 2)
    );
}

#[test]
fn finalized_original_prefix_membership_precedes_whole_unit_accounting() {
    for text in ["A\u{301}", "AA"] {
        let prepared = prepare(
            if text == "AA" {
                crate::shaping_test_fonts::ligature_ahem()
            } else {
                crate::shaping_test_fonts::mark_ahem()
            },
            text,
            style(),
        );
        let line = whole(&prepared);
        for (at, expected) in [(0, false), (1, true), (2, true)] {
            let observed = line.prefix_at(&seam(&prepared, at)).unwrap();
            assert_eq!(
                observed.has_retained_flow_participation(),
                expected,
                "{text:?} at{at}"
            );
        }
    }
    let mut input = style();
    input.white_space = WhiteSpace::Collapse;
    let prepared = native(" \u{301}", input, vec![], vec![]);
    let line = whole(&prepared);
    let observed = line.prefix_at(&seam(&prepared, 1)).unwrap();
    assert!(observed.interior_unit().is_some());
    assert!(!observed.has_retained_flow_participation());
    assert!(
        line.prefix_at(&seam(&prepared, 2))
            .unwrap()
            .has_retained_flow_participation()
    );

    for spacing in [-12.5, -15.0] {
        let mut input = style();
        input.letter_spacing = spacing;
        let prepared = prepare(crate::shaping_test_fonts::ligature_ahem(), "AX", input);
        let line = whole(&prepared);
        assert!(
            line.prefix_at(&seam(&prepared, 1))
                .unwrap()
                .has_retained_flow_participation()
        );
    }
    for mode in [WhiteSpace::Collapse, WhiteSpace::Discard] {
        let mut input = style();
        input.white_space = mode;
        let prepared = native("  X", input, vec![], vec![]);
        let line = whole(&prepared);
        assert!(
            !line
                .prefix_at(&seam(&prepared, 2))
                .unwrap()
                .has_retained_flow_participation()
        );
        assert!(
            line.prefix_at(&seam(&prepared, 3))
                .unwrap()
                .has_retained_flow_participation()
        );
    }
    let empty = native("", style(), vec![], vec![]);
    assert!(
        !whole(&empty)
            .prefix_at(&seam(&empty, 0))
            .unwrap()
            .has_retained_flow_participation()
    );
}

#[test]
fn finalized_original_prefix_membership_uses_actual_control_bundle_and_allocated_role() {
    let prepared = native(
        "X",
        style(),
        vec![
            ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        vec![],
    );
    let termination = prepared
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let selected = selected_end(&prepared, termination, None);
    let mut request = selected.request().clone();
    request.controls[1].strut = Some(ControlStrut {
        baseline: 0.0,
        line_extent: 0.0,
        alignment: LineBoxAlignment::Baseline,
    });
    let line = prepared
        .select(request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let controls: Vec<_> = prepared
        .control_views()
        .map(|v| PreparedParticipant::Control(v.handle()))
        .collect();
    assert!(
        !line
            .prefix_at(&TextSeam::AfterParticipant(controls[0].clone()))
            .unwrap()
            .has_retained_flow_participation()
    );
    assert!(
        !line
            .prefix_at(&TextSeam::BeforeParticipant(controls[1].clone()))
            .unwrap()
            .has_retained_flow_participation()
    );
    assert!(
        line.prefix_at(&TextSeam::AfterParticipant(controls[1].clone()))
            .unwrap()
            .has_retained_flow_participation()
    );

    let prepared = native(
        "X",
        style(),
        vec![ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(style()),
            },
        }],
        vec![
            ParagraphReplacement::resolved_native_logical_before(
                ParagraphReplacementDefinition::Control {
                    control: 0,
                    origin: None,
                },
                0,
                vec![Text::from("A")],
            )
            .unwrap(),
        ],
    );
    let alternative = prepared.alternative_views().next().unwrap().handle();
    let line = selected_end(
        &prepared,
        alternative.view().opportunity(),
        Some(alternative),
    )
    .finalize(SpacingAllocations::None)
    .unwrap();
    let control = PreparedParticipant::Control(prepared.control_views().next().unwrap().handle());
    assert!(
        !line
            .prefix_at(&TextSeam::BeforeParticipant(control.clone()))
            .unwrap()
            .has_retained_flow_participation()
    );
    assert!(
        line.prefix_at(&TextSeam::AfterParticipant(control))
            .unwrap()
            .has_retained_flow_participation()
    );
}

#[test]
fn finalized_original_prefix_empty_occurrences_inherit_only_actual_preceding_content() {
    let analyzed = AnalyzedSource::from_paragraph(ParagraphInput {
        tab_reference: None,
        source: Source::new("A"),
        base_style: style(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![
            ParagraphOrigin {
                source: Source::new(""),
                local: Range::new(0, 0),
                aggregate: Range::new(0, 0),
                kind: ParagraphOriginKind::Authored,
            },
            ParagraphOrigin {
                source: Source::new("A"),
                local: Range::new(0, 1),
                aggregate: Range::new(0, 1),
                kind: ParagraphOriginKind::Authored,
            },
            ParagraphOrigin {
                source: Source::new(""),
                local: Range::new(0, 0),
                aggregate: Range::new(1, 1),
                kind: ParagraphOriginKind::Authored,
            },
        ],
        connections: vec![
            ParagraphConnection {
                at: 0,
                kind: ParagraphConnectionKind::Disconnected,
            },
            ParagraphConnection {
                at: 1,
                kind: ParagraphConnectionKind::Disconnected,
            },
        ],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![
            ParagraphAnchor {
                item: ParagraphAnchorItem::EmptyOrigin(0),
                affinity: Affinity::Before,
            },
            ParagraphAnchor {
                item: ParagraphAnchorItem::EmptyOrigin(2),
                affinity: Affinity::After,
            },
        ],
    })
    .unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::ligature_ahem())
        .unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let line = whole(&prepared);
    for (index, expected) in [(0, false), (2, true)] {
        let origin = prepared.origin_views().nth(index).unwrap().handle();
        let local = SourcePosition::try_new(origin.view().source().text(), 0).unwrap();
        assert_eq!(
            line.prefix_at(&TextSeam::Origin { origin, local })
                .unwrap()
                .has_retained_flow_participation(),
            expected
        );
    }
}

#[test]
fn finalized_original_prefix_source_replacement_keeps_its_defining_source_bundle() {
    let prepared = native(
        "\u{ad}X",
        style(),
        vec![],
        vec![
            ParagraphReplacement::resolved_native_logical_before(
                ParagraphReplacementDefinition::Source {
                    origin: 0,
                    source: Range::new(0, 1),
                },
                1,
                vec![Text::from("A")],
            )
            .unwrap(),
        ],
    );
    let alternative = prepared.alternative_views().next().unwrap().handle();
    let line = selected_end(
        &prepared,
        alternative.view().opportunity(),
        Some(alternative),
    )
    .finalize(SpacingAllocations::None)
    .unwrap();
    let defining=prepared.participants().find(|p|matches!(p,PreparedParticipant::Text(unit) if unit.view().source_range()==Range::new(0,1))).unwrap();
    assert!(
        !line
            .prefix_at(&TextSeam::BeforeParticipant(defining.clone()))
            .unwrap()
            .has_retained_flow_participation()
    );
    assert!(
        line.prefix_at(&TextSeam::AfterParticipant(defining))
            .unwrap()
            .has_retained_flow_participation()
    );
    assert!(
        !line
            .prefix_at(&seam(&prepared, 0))
            .unwrap()
            .has_retained_flow_participation()
    );
    assert!(
        line.prefix_at(&seam(&prepared, 1))
            .unwrap()
            .has_retained_flow_participation()
    );
}
