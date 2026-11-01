//! Late soft edge policy consumes real physical cuts, never reconstructed offsets.

use super::*;
use std::{ops::Range, sync::Arc};
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

struct Fixture {
    binding: TextParagraphBinding<u32>,
    scope: text::CapturedScope,
}

impl Fixture {
    fn new(raw: &str, range: Range<usize>, physical: bool) -> Self {
        use text::SourceProcessingEvent as E;
        let style = text::Style {
            word_break: text::WordBreak::BreakAll,
            hyphens: text::Hyphens::None,
            white_space: text::WhiteSpace::Preserve,
            ..ahem_style(10.0, 10.0)
        };
        let units = raw.encode_utf16().collect::<Vec<_>>();
        // These fixtures contain BMP scalars; even the combining mark is an
        // original scalar contribution, not an invented Text-unit boundary.
        assert_eq!(units.len(), raw.chars().count());
        assert!(range.start < range.end && range.end <= units.len());
        let mut source = Source::new(raw);
        source.span(text::Range::new(0, units.len()), style.clone());
        let mut controls = Vec::new();
        if physical {
            controls.extend([
                text::ParagraphControl {
                    at: range.start,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: range.end,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ]);
        }
        let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
        let mut nodes = Vec::new();
        let mut origins = Vec::new();
        for at in 0..=units.len() {
            if at == range.end {
                if physical {
                    events.push(E::Control {
                        control: 1,
                        affinity: text::Affinity::After,
                    });
                    nodes.push(21);
                }
                events.push(E::ExitScope(1));
            }
            if at > 0 && at < units.len() {
                events.push(E::Connection(at - 1));
            }
            if at == range.start {
                events.push(E::EnterScope(1));
                if physical {
                    events.push(E::Control {
                        control: 0,
                        affinity: text::Affinity::After,
                    });
                    nodes.push(20);
                }
            }
            if at < units.len() {
                origins.push(text::ParagraphOrigin {
                    text_node: None,
                    source: Source::identified(
                        String::from_utf16(&units[at..at + 1]).unwrap(),
                        text::Id::from_u64(710_000 + at as u64),
                        19,
                    ),
                    local: text::Range::new(0, 1),
                    aggregate: text::Range::new(at, at + 1),
                    kind: text::ParagraphOriginKind::Authored,
                });
                events.push(E::Origin {
                    origin: at,
                    affinity: text::Affinity::After,
                });
                nodes.push(100 + u32::try_from(at).unwrap());
            }
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let processed = Arc::new(
            text::process_contextual_source(text::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                computed_line_boundaries: vec![],
                source,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                envelope: text::SourceProcessingEnvelope::Complete,
                origins,
                connections: (1..units.len())
                    .map(|after_origin| text::SourceProcessingConnection {
                        before_origin: after_origin - 1,
                        after_origin,
                        kind: text::ParagraphConnectionKind::Continuous,
                    })
                    .collect(),
                controls,
                boundary_policies: (0..units.len() - 1)
                    .map(|connection| text::ParagraphBoundaryPolicy {
                        definition: text::ParagraphBoundaryDefinition::Connection(connection),
                        preceding_wrap: Wrap::Word,
                        following_wrap: Wrap::Word,
                        ancestor_wrap: Wrap::Word,
                    })
                    .collect(),
                replacements: vec![],
                scopes: vec![
                    text::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        font_reference: None,
                        topology: text::SourceProcessingTopology::Complete,
                        trim: Default::default(),
                        base_style: style.clone(),
                        used_direction: ParagraphBaseDirection::LeftToRight,
                    },
                    text::SourceProcessingScope::Inline {
                        horizontal_inner_direction: None,
                        alignment_boundary: None,
                        font_reference: None,
                        trim: Default::default(),
                        style: Some(style),
                        boundaries: physical
                            .then_some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                    },
                ],
                paragraphs: vec![text::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        let analyzed = text::AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::Word);
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = system
            .prepare(&analyzed, text::PreparationOptions::default())
            .unwrap();
        let members = prepared
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>();
        assert_eq!(members.len(), 1);
        let scope = members[0]
            .scope_intersections()
            .map(|row| row.scope())
            .find(|scope| scope.role() == text::ScopeRole::Inline)
            .unwrap();
        let origins = prepared
            .origin_views()
            .enumerate()
            .map(|(at, origin)| {
                assert_eq!(
                    origin.source().id(),
                    Some(text::Id::from_u64(710_000 + at as u64))
                );
                assert_eq!(origin.source().revision(), 19);
                TextOriginBinding {
                    origin: origin.handle(),
                    node: 100 + u32::try_from(at).unwrap(),
                }
            })
            .collect();
        let controls = prepared
            .control_views()
            .map(|control| TextControlBinding {
                occurrence: control.handle(),
                node: match control.kind() {
                    ParagraphControlKind::InlineBoundaryStart => 20,
                    ParagraphControlKind::InlineBoundaryEnd => 21,
                    _ => panic!("only genuine inline endpoints were authored"),
                },
            })
            .collect();
        Self {
            binding: TextParagraphBinding {
                inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
                prepared,
                container: 0,
                nodes,
                origins,
                controls,
                atomics: vec![],
                line_strut: None,
            },
            scope,
        }
    }

    fn tree<S: LayoutScalar>(&self) -> PublicLayoutTreeOf<S> {
        let mut tree = PublicLayoutTreeOf::new()
            .children(0, self.binding.nodes.iter().copied())
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            );
        for origin in &self.binding.origins {
            tree = tree.children(origin.node, []).input(
                origin.node,
                LayoutInputOf::inline_text(TextParticipation::new(origin.origin.clone())),
            );
        }
        for control in &self.binding.controls {
            let kind = if control.node == 20 {
                InlineBoundaryKind::Start
            } else {
                InlineBoundaryKind::End
            };
            tree = tree.children(control.node, []).input(
                control.node,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        kind,
                        InlineMetricsOf::try_new(
                            crate::InlineBaselineContext::Physical,
                            S::from_f64(8.0),
                            S::from_f64(10.0),
                        )
                        .unwrap(),
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap()
                    .with_direction(Direction::Ltr),
                ),
            );
        }
        tree
    }

    fn policies(&self, edges: WrapEdges, form: u8) -> Vec<WrappingScopeInput<u32>> {
        // Native witness for this box's declared Ahem10/absolute10 metrics:
        // baseline8, extent10 (ahem_style), explicitly Baseline-aligned. This
        // is not a receipt inferred from descendant glyphs or paragraph strut.
        let scope = WrappingScopeInput::Captured {
            metric_contribution: Some(InlineBoxMetricContribution::Strut(text::LineBoxStrut {
                scope: None,
                metrics: text::LineStrut {
                    baseline: 8.0,
                    line_extent: 10.0,
                },
                alignment: text::LineBoxAlignment::Baseline,
            })),
            scope: self.scope.clone(),
            wrap_inside: WrapInside::Auto,
            edges,
        };
        let pair = WrappingScopeInput::NativePair {
            metric_contribution: None,
            start: 20,
            end: 21,
            wrap_inside: WrapInside::Auto,
            edges,
        };
        match form {
            0 => vec![scope],
            1 => vec![pair],
            2 => vec![scope, pair],
            _ => unreachable!(),
        }
    }

    fn compute<S: LayoutScalar>(
        &self,
        edges: WrapEdges,
        form: u8,
    ) -> Result<CompletedLayoutBatchOf<u32, S>, LayoutErrorOf<u32, S>> {
        let tree = self.tree::<S>();
        let input = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![self.binding.clone()],
            WrappingInput {
                scopes: self.policies(edges, form),
                child_order: Vec::new(),
            },
            Vec::new(),
        )?;
        if !self.binding.controls.is_empty()
            && [edges.before, edges.after]
                .iter()
                .any(|edge| matches!(edge, WrapEdge::Avoid | WrapEdge::AvoidLine))
        {
            assert_eq!(
                input.wrapping.scope_pair(&self.scope),
                Some((20, 21)),
                "normalization must retain the genuine physical pair"
            );
        }
        compute_layout_with_flow(
            &input,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(30.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
    }
}

fn edges(before: bool, value: WrapEdge) -> WrapEdges {
    if before {
        WrapEdges {
            before: value,
            after: WrapEdge::Auto,
        }
    } else {
        WrapEdges {
            before: WrapEdge::Auto,
            after: value,
        }
    }
}

fn finalized<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    lines.into_iter().map(Option::unwrap).collect()
}

fn assert_ascii<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    fixture: &Fixture,
    first_end: usize,
) {
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text(), &text::Text::from("ABCDE"));
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [
            text::Range::new(0, first_end),
            text::Range::new(first_end, 5)
        ]
    );
    for (index, line) in placed.lines().iter().enumerate() {
        assert_eq!(line.bounds().origin.y, index as f64 * 10.0);
        assert_eq!(line.bounds().size.height, 10.0);
        assert_eq!(line.baseline(), 8.0 + index as f64 * 10.0);
    }
    let lines = finalized(batch);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.measurements().fit().advance())
            .collect::<Vec<_>>(),
        [first_end as f64 * 10.0, (5 - first_end) as f64 * 10.0]
    );
    assert_eq!(
        lines[0].selected_line().request().context.end,
        text::LineEnd::Soft
    );
    assert_eq!(
        lines[1].selected_line().request().context.end,
        text::LineEnd::Paragraph
    );
    let mut glyphs = Vec::new();
    for line in lines {
        assert_eq!(line.prepared_text(), fixture.binding.prepared);
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            glyphs.extend(
                run.glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
            );
        }
    }
    glyphs.sort_by_key(|row| row.0.start);
    assert_eq!(
        glyphs,
        (0..5)
            .map(|i| (text::Range::new(i, i + 1), 35 + i as u32, 10.0))
            .collect::<Vec<_>>()
    );
}

fn assert_error<S: LayoutScalar>(error: LayoutErrorOf<u32, S>, expected: WrappingScopeError) {
    assert!(
        matches!(error.kind(), LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(reason)) if *reason == expected),
        "{error:?}"
    );
}

#[test]
fn physical_before_and_after_avoidance_selects_real_fitting_text_receipts() {
    fn lane<S: LayoutScalar>() {
        for before in [false, true] {
            let fixture = Fixture::new("ABCDE", if before { 3..5 } else { 1..3 }, true);
            for control in &fixture.binding.controls {
                assert!(
                    fixture
                        .binding
                        .prepared
                        .control_is_boundary_exact(&control.occurrence)
                        .unwrap()
                );
            }
            for form in 0..3 {
                for edge in [
                    WrapEdge::Auto,
                    WrapEdge::Avoid,
                    WrapEdge::AvoidLine,
                    WrapEdge::AvoidFlex,
                    WrapEdge::Flex,
                ] {
                    let batch = fixture.compute::<S>(edges(before, edge), form).unwrap();
                    let avoided = matches!(edge, WrapEdge::Avoid | WrapEdge::AvoidLine);
                    assert_ascii(&batch, &fixture, if avoided { 2 } else { 3 });
                    if !avoided {
                        // Establish the actual selected incidence, not merely
                        // a coincident UTF-16 position or an invented offer.
                        let node = if before { 20 } else { 21 };
                        let control = &fixture
                            .binding
                            .controls
                            .iter()
                            .find(|row| row.node == node)
                            .unwrap()
                            .occurrence;
                        let at = fixture.binding.prepared.participants().position(|p| matches!(p, text::PreparedParticipant::Control(c) if &c == control)).unwrap();
                        assert_eq!(
                            finalized(&batch)[0]
                                .selected_line()
                                .request()
                                .end
                                .view()
                                .participant_offset(),
                            at + usize::from(!before)
                        );
                    }
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn cross_origin_snapped_pair_admits_but_keeps_precise_edge_incidence_limits() {
    // Qualify this retained fixture's real precomposed glyph, not hypothetical
    // combining-mark coverage. Harfrust may canonically compose A + acute.
    let u16_at = |bytes: &[u8], at| u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap());
    let table = |tag: [u8; 4]| {
        let at = (0..usize::from(u16_at(AHEM, 4)))
            .map(|i| 12 + i * 16)
            .find(|at| AHEM[*at..*at + 4] == tag)
            .unwrap();
        let start = u32::from_be_bytes(AHEM[at + 8..at + 12].try_into().unwrap()) as usize;
        let len = u32::from_be_bytes(AHEM[at + 12..at + 16].try_into().unwrap()) as usize;
        &AHEM[start..start + len]
    };
    assert_eq!(u16_at(table(*b"head"), 18), 1000);
    assert!(u16_at(table(*b"hhea"), 34) > 165);
    for glyph in [165, 36, 37, 38, 39, 40] {
        assert_eq!(u16_at(table(*b"hmtx"), glyph * 4), 1000);
    }
    let cmap = table(*b"cmap");
    let record = (0..usize::from(u16_at(cmap, 2)))
        .map(|i| 4 + i * 8)
        .find(|at| u16_at(cmap, *at) == 3 && u16_at(cmap, *at + 2) == 1)
        .unwrap();
    let at = u32::from_be_bytes(cmap[record + 4..record + 8].try_into().unwrap()) as usize;
    let format4 = &cmap[at..];
    assert_eq!(u16_at(format4, 0), 4);
    let count = usize::from(u16_at(format4, 6)) / 2;
    let cp = 0xc1_u16;
    let segment = (0..count)
        .find(|i| {
            u16_at(format4, 16 + 2 * count + i * 2) <= cp && cp <= u16_at(format4, 14 + i * 2)
        })
        .unwrap();
    let start = u16_at(format4, 16 + 2 * count + segment * 2);
    let delta = u16_at(format4, 16 + 4 * count + segment * 2);
    let position = 16 + 6 * count + segment * 2;
    let offset = u16_at(format4, position);
    let glyph = if offset == 0 {
        cp
    } else {
        u16_at(
            format4,
            position + usize::from(offset) + usize::from(cp - start) * 2,
        )
    };
    assert_ne!(glyph, 0);
    assert_eq!(glyph.wrapping_add(delta), 165);
    fn lane<S: LayoutScalar>() {
        for before in [false, true] {
            let fixture = Fixture::new("A\u{301}BCDEF", if before { 1..3 } else { 0..1 }, true);
            // Prepared units retain individual transform-atom contributions;
            // they are not the finalized grapheme/provider group hull.
            assert_eq!(
                fixture
                    .binding
                    .prepared
                    .unit_views()
                    .take(2)
                    .map(|unit| unit.source_range())
                    .collect::<Vec<_>>(),
                [text::Range::new(0, 1), text::Range::new(1, 2)]
            );
            for (index, original) in ["A", "\u{301}"].into_iter().enumerate() {
                let origin = fixture.binding.prepared.origin_views().nth(index).unwrap();
                assert_eq!(origin.source().text(), &text::Text::from(original));
                assert_eq!(origin.aggregate_range(), text::Range::new(index, index + 1));
            }
            // BreakAll does not create a cut between a base and its Extend.
            // Check actual mixed prefixes, not a codepoint or source offset.
            let participants = fixture.binding.prepared.participants().collect::<Vec<_>>();
            let at = |range| {
                participants.iter().position(|participant| matches!(participant, text::PreparedParticipant::Text(unit) if unit.view().source_range() == range)).unwrap()
            };
            let base = at(text::Range::new(0, 1));
            let mark = at(text::Range::new(1, 2));
            for opportunity in fixture.binding.prepared.opportunity_views() {
                let end = opportunity.boundary().view().participant_offset();
                assert_eq!(
                    base < end,
                    mark < end,
                    "no issued opportunity separates the joint grapheme"
                );
            }
            let control = &fixture
                .binding
                .controls
                .iter()
                .find(|row| row.node == if before { 20 } else { 21 })
                .unwrap()
                .occurrence;
            assert!(
                !fixture
                    .binding
                    .prepared
                    .control_is_boundary_exact(control)
                    .unwrap()
            );
            let start = fixture
                .binding
                .prepared
                .boundary_views()
                .next()
                .unwrap()
                .handle();
            let end = fixture
                .binding
                .prepared
                .opportunity_views()
                .find(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
                .unwrap()
                .handle();
            let mut source_segments = fixture.binding.prepared.source_segments();
            let source_segment = source_segments.next().expect("fixture source segment");
            assert_eq!(source_segments.count(), 0, "one structural fixture segment");
            let search = source_segment
                .conditional_search(
                    &start,
                    None,
                    &end,
                    text::ConditionalPurpose::MinContent,
                    &[],
                    surgeist_text::ConditionalLineContext::Unframed { inline_boxes: None },
                )
                .unwrap();
            assert!(search.begin(&[]).unwrap().opportunities().is_empty());
            let hidden_side = if before {
                text::CapturedScopeIncidence::AtBefore
            } else {
                text::CapturedScopeIncidence::AtAfter
            };
            for offer in fixture
                .binding
                .prepared
                .opportunity_views()
                .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
            {
                let incidences = fixture
                    .binding
                    .prepared
                    .boundary_incidences(&source_segment, &offer.boundary())
                    .unwrap();
                assert!(!incidences.is_empty());
                for incidence in incidences {
                    assert_ne!(
                        fixture
                            .binding
                            .prepared
                            .incidence_scope_relation(&incidence, &fixture.scope)
                            .unwrap(),
                        hidden_side,
                        "the authenticated original cut proves the hidden side is a nonmatch"
                    );
                }
            }
            for form in 0..3 {
                let tree = fixture.tree::<S>();
                LayoutFlowInput::try_new(
                    &tree,
                    0,
                    vec![fixture.binding.clone()],
                    WrappingInput {
                        scopes: fixture.policies(WrapEdges::AUTO, form),
                        child_order: Vec::new(),
                    },
                    Vec::new(),
                )
                .expect("all genuine declaration forms retain strict neutral association");
            }
            // The known hidden side is a nonmatch, not missing correspondence.
            // Avoid cannot alter any actual ending here. Late Line still
            // requires an earlier source-processing receipt.
            for form in 0..3 {
                for edge in [
                    WrapEdge::Auto,
                    WrapEdge::Avoid,
                    WrapEdge::AvoidLine,
                    WrapEdge::AvoidFlex,
                    WrapEdge::Line,
                    WrapEdge::Flex,
                ] {
                    match edge {
                        WrapEdge::Line => assert_error(
                            fixture
                                .compute::<S>(edges(before, edge), form)
                                .expect_err("late Line requires its genuine early cause"),
                            WrappingScopeError::SourceProcessingRequired,
                        ),
                        WrapEdge::Auto
                        | WrapEdge::Avoid
                        | WrapEdge::AvoidLine
                        | WrapEdge::AvoidFlex
                        | WrapEdge::Flex => {
                            let batch = fixture.compute::<S>(edges(before, edge), form).unwrap();
                            let layout = batch.placed_text()[0].layout();
                            assert_eq!(layout.source().text(), &text::Text::from("A\u{301}BCDEF"));
                            // Actual Ahem cmap supplies canonical A-acute165
                            // at one em: [A+acute,B,C] and [D,E,F] each fill30.
                            assert_eq!(
                                layout
                                    .lines()
                                    .iter()
                                    .map(|line| line.range())
                                    .collect::<Vec<_>>(),
                                [text::Range::new(0, 4), text::Range::new(4, 7)]
                            );
                            let lines = finalized(&batch);
                            assert_eq!(
                                lines
                                    .iter()
                                    .map(|line| line.measurements().fit().advance())
                                    .collect::<Vec<_>>(),
                                [30.0, 30.0]
                            );
                            let glyphs = lines
                                .iter()
                                .flat_map(|line| {
                                    line.run_views().flat_map(|run| {
                                        assert_eq!(run.face().bytes(), AHEM);
                                        run.glyphs()
                                            .map(|glyph| {
                                                (glyph.id(), glyph.range(), glyph.advance())
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                })
                                .collect::<Vec<_>>();
                            let mut expected = vec![(165, text::Range::new(0, 2), 10.0)];
                            expected
                                .extend((2..7).map(|at| {
                                    (34 + at as u32, text::Range::new(at, at + 1), 10.0)
                                }));
                            assert_eq!(glyphs, expected);
                        }
                    }
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn authentic_logical_edge_supports_soft_incidence_and_late_line_requires_source_processing() {
    fn lane<S: LayoutScalar>() {
        for before in [false, true] {
            let range = if before { 3..5 } else { 1..3 };
            let logical = Fixture::new("ABCDE", range.clone(), false);
            assert!(logical.binding.controls.is_empty());
            assert_ascii(
                &logical
                    .compute::<S>(edges(before, WrapEdge::AvoidFlex), 0)
                    .unwrap(),
                &logical,
                3,
            );
            // The issued three-letter prefix has a genuine original box
            // edge even without physical controls. Authenticate that exact
            // shared cut instead of reconstructing an edge from UTF-16 alone.
            let prepared = &logical.binding.prepared;
            let mut source_segments = prepared.source_segments();
            let source_segment = source_segments.next().expect("fixture source segment");
            assert_eq!(source_segments.count(), 0, "one structural fixture segment");
            let boundary = prepared
                .opportunity_views()
                .find(|offer| {
                    offer.kind() == text::PreparedBreakKind::Allowed
                        && offer.boundary().view().source_before().get() == 3
                        && offer.boundary().view().source_after().get() == 3
                })
                .unwrap()
                .boundary();
            let expected = if before {
                text::CapturedScopeIncidence::AtBefore
            } else {
                text::CapturedScopeIncidence::AtAfter
            };
            assert!(
                prepared
                    .boundary_incidences(&source_segment, &boundary)
                    .unwrap()
                    .iter()
                    .any(|incidence| prepared
                        .incidence_scope_relation(incidence, &logical.scope)
                        .unwrap()
                        == expected)
            );
            // Three Ahem10 letters fit30, but that cut is avoided. The latest
            // unviolated issued cut is2; the remaining three letters fit30.
            assert_ascii(
                &logical
                    .compute::<S>(edges(before, WrapEdge::Avoid), 0)
                    .unwrap(),
                &logical,
                2,
            );
            for physical in [false, true] {
                let fixture = Fixture::new("ABCDE", range.clone(), physical);
                assert_error(
                    fixture
                        .compute::<S>(edges(before, WrapEdge::Line), 0)
                        .expect_err("a late force has no early computed-cause receipt"),
                    WrappingScopeError::SourceProcessingRequired,
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}
