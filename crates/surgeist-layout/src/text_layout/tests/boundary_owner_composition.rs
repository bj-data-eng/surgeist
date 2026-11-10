//! #75 logical following-box ownership composed through real Text and Layout.
//! Sources are original occurrences in complete sibling/nested inline captures.
//! Font glyph0 below qualifies actual .notdef geometry, never script coverage.

use super::*;
use crate::test_support::readback::{self, Phase};
use std::sync::Arc;
use surgeist_text as text;

#[path = "boundary_owner_composition/shared_unit_endpoints.rs"]
mod shared_unit_endpoints;

#[path = "boundary_owner_composition/float_end_prefix_controls.rs"]
mod float_end_prefix_controls;

const PHRASE: &[u8] =
    include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf");
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn style(word: text::WordBreak, line: text::LineBreak, phrase_font: bool) -> text::Style {
    text::Style {
        font: text::Font::new().family(if phrase_font {
            "Surgeist Phrase Ahem"
        } else {
            "Ahem"
        }),
        locale: Some("ja".into()),
        word_break: word,
        line_break: line,
        overflow_wrap: text::OverflowWrap::Normal,
        hyphens: text::Hyphens::None,
        white_space: text::WhiteSpace::Preserve,
        wrap: Wrap::Word,
        ..ahem_style(10.0, 10.0)
    }
}

fn capture(
    parts: [&str; 2],
    styles: [text::Style; 2],
    font: &[u8],
    nested: bool,
    ancestor: Wrap,
    rtl: bool,
    endpoints: bool,
) -> TextParagraphBinding<u32> {
    use text::SourceProcessingEvent as E;
    let direction = if rtl {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    };
    let local_wrap = styles.each_ref().map(|style| style.wrap);
    let lengths = parts.map(|s| s.encode_utf16().count());
    let end = lengths[0] + lengths[1];
    let mut source = Source::new(parts.concat());
    source.span(text::Range::new(0, lengths[0]), styles[0].clone());
    source.span(text::Range::new(lengths[0], end), styles[1].clone());
    let base = text::Style {
        wrap: ancestor,
        ..styles[0].clone()
    };
    let mut scopes = vec![text::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: text::SourceProcessingTopology::Complete,
        trim: Default::default(),
        base_style: base.clone(),
        used_direction: direction,
    }];
    let mut controls = Vec::new();
    let mut add_scope = |scope_style: text::Style, start, end| {
        let scope = scopes.len();
        let first = controls.len();
        if endpoints {
            controls.extend([
                text::ParagraphControl {
                    at: start,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: end,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ]);
        }
        scopes.push(text::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: Some(scope_style),
            boundaries: endpoints.then_some(text::SourceProcessingScopeBoundaries {
                start: first,
                end: first + 1,
            }),
        });
        (scope, first)
    };
    let outer = nested.then(|| add_scope(base.clone(), 0, end));
    let before = add_scope(styles[0].clone(), 0, lengths[0]);
    let after = add_scope(styles[1].clone(), lengths[0], end);
    let control = |control| E::Control {
        control,
        affinity: text::Affinity::After,
    };
    let origin = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    if let Some((scope, start)) = outer {
        events.extend([E::EnterScope(scope), control(start)]);
    }
    events.extend([
        E::EnterScope(before.0),
        control(before.1),
        origin(0),
        control(before.1 + 1),
        E::ExitScope(before.0),
        E::Connection(0),
        E::EnterScope(after.0),
        control(after.1),
        origin(1),
        control(after.1 + 1),
        E::ExitScope(after.0),
    ]);
    if let Some((scope, start)) = outer {
        events.extend([control(start + 1), E::ExitScope(scope)]);
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    if !endpoints {
        events.retain(|e| !matches!(e, E::Control { .. }));
    }
    let nodes = events
        .iter()
        .filter_map(|e| match e {
            E::Control { control, .. } => Some(100 + u32::try_from(*control).unwrap()),
            E::Origin { origin, .. } => Some(1 + u32::try_from(*origin).unwrap()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let control_order = events
        .iter()
        .filter_map(|e| match e {
            E::Control { control, .. } => Some(*control),
            _ => None,
        })
        .collect::<Vec<_>>();
    let originals = parts
        .into_iter()
        .enumerate()
        .map(|(index, part)| Source::identified(part, text::Id::from_u64(75001 + index as u64), 23))
        .collect::<Vec<_>>();
    let origins = originals
        .iter()
        .enumerate()
        .map(|(index, original)| text::ParagraphOrigin {
            text_node: None,
            source: original.clone(),
            local: text::Range::new(0, lengths[index]),
            aggregate: if index == 0 {
                text::Range::new(0, lengths[0])
            } else {
                text::Range::new(lengths[0], end)
            },
            kind: text::ParagraphOriginKind::Authored,
        })
        .collect();
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins,
            connections: vec![text::SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: text::ParagraphConnectionKind::Continuous,
            }],
            controls,
            boundary_policies: vec![text::ParagraphBoundaryPolicy {
                definition: text::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: local_wrap[0],
                following_wrap: local_wrap[1],
                ancestor_wrap: ancestor,
            }],
            replacements: vec![],
            scopes,
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: direction,
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
    // The only cross-origin NCA is the captured outer inline (nested) or block
    // (siblings). Same-origin pairs use their own captured child policy.
    let policies = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| {
            let wrap = match (r.before(), r.after()) {
                (
                    text::WrappingNeighbor::Text { origin: a, .. },
                    text::WrappingNeighbor::Text { origin: b, .. },
                ) if a == b => local_wrap[a],
                _ => ancestor,
            };
            text::WrappingPolicy {
                relation: r.handle(),
                ancestor_wrap: wrap,
            }
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    for r in analyzed.wrapping_relations() {
        if let (
            text::WrappingNeighbor::Text { origin: a, .. },
            text::WrappingNeighbor::Text { origin: b, .. },
        ) = (r.before(), r.after())
        {
            assert_eq!(
                r.ancestor_wrap(),
                Some(if a == b { local_wrap[a] } else { ancestor })
            );
        }
    }
    let mut system = text::System::default();
    system
        .register_fonts(font.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let members = prepared
        .captured_owner()
        .unwrap()
        .members()
        .collect::<Vec<_>>();
    assert_eq!(members.len(), 1);
    let captured = &members[0];
    let inline = captured
        .scope_intersections()
        .map(|i| i.scope())
        .filter(|s| s.role() == text::ScopeRole::Inline)
        .collect::<Vec<_>>();
    assert_eq!(inline.len(), if nested { 3 } else { 2 });
    for scope in &inline {
        assert_eq!(scope.entry_control().is_some(), endpoints);
        assert_eq!(scope.exit_control().is_some(), endpoints);
    }
    let mut handles = prepared
        .control_views()
        .map(|c| c.handle())
        .collect::<Vec<_>>();
    handles.sort_by(|a, b| {
        a.captured_control()
            .unwrap()
            .event_position()
            .partial_cmp(&b.captured_control().unwrap().event_position())
            .unwrap()
    });
    let controls = handles
        .into_iter()
        .zip(control_order)
        .map(|(occurrence, index)| TextControlBinding {
            occurrence,
            node: 100 + u32::try_from(index).unwrap(),
        })
        .collect();
    let origins = prepared
        .origin_views()
        .enumerate()
        .map(|(index, view)| {
            assert_eq!(view.source(), &originals[index]);
            TextOriginBinding {
                origin: view.handle(),
                node: 1 + u32::try_from(index).unwrap(),
            }
        })
        .collect();
    TextParagraphBinding {
        inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
        prepared,
        container: 0,
        nodes,
        origins,
        controls,
        atomics: vec![],
        line_strut: None,
    }
}

fn tree<S: LayoutScalar>(
    p: &TextParagraphBinding<u32>,
    rtl: bool,
    edges: &[(u32, InlineBoundaryEdgeOf<S>)],
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, p.nodes.iter().copied())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                ..NodeInputOf::default()
            },
        );
    for origin in &p.origins {
        tree = tree.children(origin.node, []).input(
            origin.node,
            LayoutInputOf::inline_text(TextParticipation::new(origin.origin.clone())),
        );
    }
    for control in &p.controls {
        let kind = match control.occurrence.view().kind() {
            ParagraphControlKind::InlineBoundaryStart => InlineBoundaryKind::Start,
            ParagraphControlKind::InlineBoundaryEnd => InlineBoundaryKind::End,
            _ => unreachable!(),
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
                    edges
                        .iter()
                        .find_map(|(node, edge)| (*node == control.node).then_some(*edge))
                        .unwrap_or(InlineBoundaryEdgeOf::ZERO),
                )
                .unwrap()
                .with_direction(if rtl { Direction::Rtl } else { Direction::Ltr }),
            ),
        );
    }
    tree
}

fn compute<S: LayoutScalar>(
    p: &TextParagraphBinding<u32>,
    rtl: bool,
    width: crate::AxisSizingRequestOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = tree::<S>(p, rtl, &[]);
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.clone()],
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(&capture, 0, {
        let containing_flow_axes = {
            let root_input = crate::LayoutTree::node_input((capture).tree(), 0);
            crate::FlowAxes::new(root_input.writing_mode, root_input.direction)
        };
        let requested_axes = Size::new(
            width,
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::MaxContent,
            ),
        );
        let mut logical_requests = containing_flow_axes.logical_size(requested_axes);
        // These fixtures perform ordinary block-size layout independently of
        // the explicitly requested inline contribution operation.
        logical_requests.block = logical_requests
            .block
            .with_operation(crate::SizingOperation::Normal);
        let axes = containing_flow_axes.physical_size(logical_requests);
        // The old Native NoParent fixture explicitly supplied these percentage
        // bases. It supplied no independent ICB dimensions or ancestor scrollport.
        let percentage_parent = requested_axes.map(|axis| axis.space().definite_size());
        let fallback = crate::OrthogonalFallbackOf::new(
            crate::Size::NONE,
            crate::NearestScrollportOf::Absent,
            crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
        );
        crate::LayoutRootRequestOf::standalone(
            crate::SizingRequestOf::new(axes, fallback),
            containing_flow_axes,
            percentage_parent,
        )
    })
    .unwrap()
}

fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let count = u16::from_be_bytes(bytes[4..6].try_into().unwrap());
    let at = (0..usize::from(count))
        .map(|i| 12 + i * 16)
        .find(|at| bytes[*at..*at + 4] == tag)
        .unwrap();
    let offset = u32::from_be_bytes(bytes[at + 8..at + 12].try_into().unwrap()) as usize;
    let len = u32::from_be_bytes(bytes[at + 12..at + 16].try_into().unwrap()) as usize;
    &bytes[offset..offset + len]
}

fn nominal(bytes: &[u8], glyph: u32) -> f64 {
    let upem = u16::from_be_bytes(table(bytes, *b"head")[18..20].try_into().unwrap());
    let count = u16::from_be_bytes(table(bytes, *b"hhea")[34..36].try_into().unwrap());
    let at = usize::try_from(glyph.min(u32::from(count) - 1)).unwrap() * 4;
    let design = u16::from_be_bytes(table(bytes, *b"hmtx")[at..at + 2].try_into().unwrap());
    assert_eq!(upem, 1000);
    assert_eq!(design, 1000);
    f64::from(design) * 10.0 / f64::from(upem)
}

fn qualify<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &TextParagraphBinding<u32>,
    font: &[u8],
    source: &str,
    ranges: &[text::Range],
    expected_glyphs: &[(u32, f32)],
) -> Vec<text::FinalizedLine> {
    assert_eq!(batch.placed_text().len(), 1);
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        source.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        layout.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        ranges
    );
    for (index, origin) in p.origins.iter().enumerate() {
        assert_eq!(
            origin.origin.view().source().id(),
            Some(text::Id::from_u64(75001 + index as u64))
        );
        assert_eq!(origin.origin.view().source().revision(), 23);
        assert!(
            !layout
                .source_range_fragments(&origin.origin, origin.origin.view().local_range())
                .unwrap()
                .units()
                .collect::<Vec<_>>()
                .is_empty()
        );
    }
    let mut lines = vec![None; ranges.len()];
    for row in batch.final_inline_fragments() {
        let fragment = row.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), p.prepared);
        lines[fragment.line_index()] = Some(line);
    }
    let lines = lines.into_iter().map(Option::unwrap).collect::<Vec<_>>();
    let mut glyphs = Vec::new();
    for line in &lines {
        let selected = line.selected_line();
        let r = selected.request();
        assert!(r.alternative.is_none());
        assert!(r.incoming.is_none());
        let text::PreparedTermination::Ordinary(offer) = &r.termination else {
            panic!("ordinary owned opportunity required")
        };
        assert!(matches!(
            offer.view().kind(),
            text::PreparedBreakKind::Allowed | text::PreparedBreakKind::ParagraphEnd
        ));
        let mut raw = 0.0;
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), font);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(
                run.effective_instance()
                    .unwrap()
                    .metrics(run.face().catalogue_identity())
                    .unwrap()
                    .units_per_em(),
                1000
            );
            for glyph in run.glyphs() {
                glyphs.push((glyph.id(), glyph.advance()));
                if glyph.id() == 0 {
                    assert_eq!(font, PHRASE);
                    assert_eq!(
                        glyph.range(),
                        text::Range::new(1, 2),
                        "actual named CJK punctuation source, not script coverage"
                    );
                }
                if glyph.id() != 100 {
                    assert_eq!(f64::from(glyph.advance()), nominal(font, glyph.id()));
                } else {
                    // Literal retained GPOS anchors (400,600)-(100,200) at10px.
                    assert_eq!(glyph.pen() + f64::from(glyph.x()), 3.0);
                    assert_eq!(glyph.y(), -4.0);
                }
            }
            raw += run.advance();
        }
        assert_eq!(line.feedback().advance, raw);
        assert_eq!(line.measurements().fit().advance(), raw);
        assert_eq!(
            line.measurements().min_content_contribution().advance(),
            raw
        );
        assert_eq!(
            line.measurements().max_content_contribution().advance(),
            raw
        );
        assert_eq!(line.feedback().line_height, 10.0);
    }
    glyphs.sort_by_key(|g| g.0);
    let mut expected = expected_glyphs.to_vec();
    expected.sort_by_key(|g| g.0);
    assert_eq!(glyphs, expected);
    lines
}

fn assert_pair<S: LayoutScalar>(
    p: &TextParagraphBinding<u32>,
    font: &[u8],
    source: &str,
    cut: Option<usize>,
    rtl: bool,
    glyphs: &[(u32, f32)],
) {
    let end = source.encode_utf16().count();
    let ordinary = p
        .prepared
        .opportunity_views()
        .filter(|o| o.kind() == text::PreparedBreakKind::Allowed)
        .collect::<Vec<_>>();
    if let Some(cut) = cut {
        assert!(!ordinary.is_empty());
        assert!(
            ordinary
                .iter()
                .all(|o| o.boundary().view().source_before().get() == cut
                    && o.boundary().view().source_after().get() == cut)
        );
    } else {
        assert!(ordinary.is_empty());
    }
    let ranges = cut.map_or_else(
        || vec![text::Range::new(0, end)],
        |cut| vec![text::Range::new(0, cut), text::Range::new(cut, end)],
    );
    let actual = compute::<S>(
        p,
        rtl,
        crate::AxisSizingRequestOf::new(
            crate::AvailableSpaceOf::Definite(
                crate::NonNegativeFiniteOf::new(S::from_f64(10.0)).unwrap(),
            ),
            crate::SizingOperation::Normal,
        ),
    );
    let lines = qualify(&actual, p, font, source, &ranges, glyphs);
    assert_eq!(
        lines
            .iter()
            .map(|l| l.feedback().advance)
            .collect::<Vec<_>>(),
        if cut.is_some() {
            vec![10.0, 10.0]
        } else {
            vec![20.0]
        }
    );
    if let Some(cut) = cut {
        let selected = lines[0].selected_line();
        let r = selected.request();
        assert_eq!(r.context.end, text::LineEnd::Soft);
        assert_eq!(r.end.view().source_before().get(), cut);
        assert_eq!(r.end.view().source_after().get(), cut);
    }
    for (width, expected) in [
        (
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::MinContent,
            ),
            if cut.is_some() { 10.0 } else { 20.0 },
        ),
        (
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::MaxContent,
            ),
            20.0,
        ),
    ] {
        let batch = compute::<S>(p, rtl, width);
        assert_eq!(
            batch.placed_text()[0].layout().source().text().code_units(),
            source.encode_utf16().collect::<Vec<_>>()
        );
        let root = readback::single_box_output(&batch, 0, Phase::Unrounded);
        assert_eq!(root.size.width, S::from_f64(expected));
    }
}

#[test]
fn following_word_break_survives_sibling_nested_and_bidi_composition() {
    fn lane<S: LayoutScalar>() {
        use text::WordBreak::{BreakAll, KeepAll, Normal};
        for nested in [false, true] {
            for rtl in [false, true] {
                for (parts, before, after, cut) in [
                    (["日", "本"], Normal, KeepAll, None),
                    (["日", "本"], KeepAll, Normal, Some(1)),
                    (["A", "A"], BreakAll, Normal, None),
                    (["A", "A"], Normal, BreakAll, Some(1)),
                ] {
                    let p = capture(
                        parts,
                        [
                            style(before, text::LineBreak::Strict, true),
                            style(after, text::LineBreak::Strict, true),
                        ],
                        PHRASE,
                        nested,
                        Wrap::Word,
                        rtl,
                        true,
                    );
                    assert_pair::<S>(
                        &p,
                        PHRASE,
                        &parts.concat(),
                        cut,
                        rtl,
                        &[(35, 10.0), (35, 10.0)],
                    );
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn following_strictness_keeps_named_cjk_requirements_and_word_mode_precedence() {
    fn lane<S: LayoutScalar>() {
        use text::{
            LineBreak::{Loose, Normal, Strict},
            WordBreak::{BreakAll, KeepAll},
        };
        // The explicit named iteration / wave-dash rules remain operative under
        // BreakAll, and required Loose/Normal cuts cannot be erased by KeepAll.
        for nested in [false, true] {
            for rtl in [false, true] {
                for (after, before_line, after_line, before_word, after_word, cut) in [
                    ("々", Loose, Strict, KeepAll, BreakAll, None),
                    ("々", Strict, Loose, BreakAll, KeepAll, Some(1)),
                    ("〜", Normal, Strict, KeepAll, BreakAll, None),
                    ("〜", Strict, Normal, BreakAll, KeepAll, Some(1)),
                ] {
                    let p = capture(
                        ["日", after],
                        [
                            style(before_word, before_line, true),
                            style(after_word, after_line, true),
                        ],
                        PHRASE,
                        nested,
                        Wrap::Word,
                        rtl,
                        true,
                    );
                    // PhraseAhem really maps 日 to35. The named punctuation is absent;
                    // only its authentic retained .notdef0 metrics are qualified.
                    assert_pair::<S>(
                        &p,
                        PHRASE,
                        &format!("日{after}"),
                        cut,
                        rtl,
                        &[(35, 10.0), (0, 10.0)],
                    );
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn either_side_anywhere_obeys_actual_common_ancestor_and_rejects_stale_bindings() {
    fn lane<S: LayoutScalar>() {
        use text::LineBreak::{Anywhere, Strict};
        // Both original Ahem cmap records map NBSP to153 (not SPACE3); its
        // retained hmtx contribution is1000/1000em, qualified in nominal().
        for nested in [false, true] {
            for rtl in [false, true] {
                for (before, after, cut) in [
                    (Anywhere, Strict, Some(1)),
                    (Strict, Anywhere, Some(1)),
                    (Strict, Strict, None),
                ] {
                    let p = capture(
                        ["\u{a0}", "A"],
                        [
                            style(text::WordBreak::KeepAll, before, false),
                            style(text::WordBreak::KeepAll, after, false),
                        ],
                        AHEM,
                        nested,
                        Wrap::Word,
                        rtl,
                        true,
                    );
                    assert_pair::<S>(&p, AHEM, "\u{a0}A", cut, rtl, &[(153, 10.0), (35, 10.0)]);
                }
                let styles = [
                    style(text::WordBreak::KeepAll, Anywhere, false),
                    style(text::WordBreak::KeepAll, Strict, false),
                ];
                let old = capture(
                    ["\u{a0}", "A"],
                    styles.clone(),
                    AHEM,
                    nested,
                    Wrap::Word,
                    rtl,
                    true,
                );
                let fresh = capture(["\u{a0}", "A"], styles, AHEM, nested, Wrap::None, rtl, true);
                assert_pair::<S>(
                    &fresh,
                    AHEM,
                    "\u{a0}A",
                    None,
                    rtl,
                    &[(153, 10.0), (35, 10.0)],
                );
                let tree = tree::<S>(&old, rtl, &[]);
                let Err(error) = LayoutFlowInput::try_new(
                    &tree,
                    0,
                    vec![fresh],
                    crate::WrappingInput {
                        native_inline_boxes: Vec::new(),
                        gaps: Vec::new(),
                        scopes: Vec::new(),
                        child_order: Vec::new(),
                    },
                    Vec::new(),
                ) else {
                    panic!("new NCA context cannot reuse old preparation handles")
                };
                assert!(matches!(
                    error.kind(),
                    LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
                        TextBindingError::ParticipantRole
                    ))
                ));
                assert_pair::<S>(
                    &old,
                    AHEM,
                    "\u{a0}A",
                    Some(1),
                    rtl,
                    &[(153, 10.0), (35, 10.0)],
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn cross_origin_combining_unit_is_whole_with_anywhere_and_real_mark_attachment() {
    fn lane<S: LayoutScalar>() {
        let font = super::canonical::fonts::mark_ahem();
        for nested in [false, true] {
            for rtl in [false, true] {
                // These styling scopes have no authored boundary nodes. Their
                // real capture nesting, Style, origins and NCA remain present.
                let p = capture(
                    ["A", "\u{301}B"],
                    [
                        style(text::WordBreak::Normal, text::LineBreak::Anywhere, false),
                        style(text::WordBreak::KeepAll, text::LineBreak::Anywhere, false),
                    ],
                    &font,
                    nested,
                    Wrap::Word,
                    rtl,
                    false,
                );
                assert_eq!(
                    p.prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
                    [text::Range::new(0, 2), text::Range::new(2, 3)]
                );
                assert_pair::<S>(
                    &p,
                    &font,
                    "A\u{301}B",
                    Some(2),
                    rtl,
                    &[(35, 10.0), (100, 0.0), (36, 10.0)],
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn emergency_seam_uses_the_logical_before_box_for_fit_and_intrinsic_width() {
    use text::{LineBreak, OverflowWrap::*, WordBreak};

    fn lane<S: LayoutScalar>() {
        // At 35px only the unbroken four-glyph Ahem word overflows.
        // The first box owns both its interior cut and the seam at offset 2.
        for (before, after, overflow, intrinsic, cut, minimum) in [
            (Normal, Normal, &[][..], &[][..], None, 40.0),
            (Anywhere, Normal, &[1, 2][..], &[1, 2][..], Some(2), 20.0),
            (Normal, Anywhere, &[3][..], &[3][..], Some(3), 30.0),
            (BreakWord, Normal, &[1, 2][..], &[][..], Some(2), 40.0),
            (Normal, BreakWord, &[3][..], &[][..], Some(3), 40.0),
        ] {
            for (nested, rtl) in [(false, false), (false, true), (true, false), (true, true)] {
                let mode = |overflow_wrap| text::Style {
                    overflow_wrap,
                    ..style(WordBreak::Normal, LineBreak::Normal, false)
                };
                let p = capture(
                    ["AA", "AA"],
                    [mode(before), mode(after)],
                    AHEM,
                    nested,
                    Wrap::Word,
                    rtl,
                    true,
                );
                let prepared = &p.prepared;
                let start = prepared.boundary_views().next().unwrap().handle();
                let mut ordinary = prepared.opportunity_views().filter(|offer| {
                    offer.boundary().view().participant_offset() > start.view().participant_offset()
                });
                let terminal = ordinary.next().unwrap();
                assert_eq!(terminal.kind(), text::PreparedBreakKind::ParagraphEnd);
                assert!(ordinary.next().is_none());
                for (purpose, expected) in [
                    (text::ConditionalPurpose::Overflow, overflow),
                    (text::ConditionalPurpose::MinContent, intrinsic),
                ] {
                    let choices = prepared
                        .source_segments()
                        .next()
                        .unwrap()
                        .conditional_search(
                            &start,
                            None,
                            &terminal.handle(),
                            purpose,
                            &[],
                            text::ConditionalLineContext::Unframed { inline_boxes: None },
                        )
                        .unwrap()
                        .begin(&[])
                        .unwrap();
                    assert_eq!(
                        choices.opportunities().len(),
                        expected.len(),
                        "{before:?}/{after:?} {purpose:?} rtl={rtl} nested={nested}"
                    );
                    let cuts = choices
                        .opportunities()
                        .iter()
                        .map(|choice| {
                            let end = choice.boundary();
                            let boundary = end.view();
                            assert_eq!(boundary.source_before(), boundary.source_after());
                            boundary.source_after().get()
                        })
                        .collect::<std::collections::BTreeSet<_>>();
                    assert_eq!(
                        cuts.into_iter().collect::<Vec<_>>(),
                        expected,
                        "{before:?}/{after:?} {purpose:?} rtl={rtl} nested={nested}"
                    );
                }
                for (width, expected_cut) in [(35.0, cut), (40.0, None)] {
                    let finite = compute::<S>(
                        &p,
                        rtl,
                        crate::AxisSizingRequestOf::new(
                            crate::AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(S::from_f64(width)).unwrap(),
                            ),
                            crate::SizingOperation::Normal,
                        ),
                    );
                    let layout = finite.placed_text()[0].layout();
                    assert_eq!(layout.source().text().code_units(), [65; 4]);
                    assert_eq!(
                        layout.analyzed_source().identity(),
                        prepared.analyzed_source().identity()
                    );
                    let ranges = layout
                        .lines()
                        .iter()
                        .map(|line| line.range())
                        .collect::<Vec<_>>();
                    let expected = if let Some(cut) = expected_cut {
                        vec![text::Range::new(0, cut), text::Range::new(cut, 4)]
                    } else {
                        vec![text::Range::new(0, 4)]
                    };
                    assert_eq!(
                        ranges, expected,
                        "{before:?}/{after:?} width={width} rtl={rtl} nested={nested}"
                    );
                    assert!(!finite.final_inline_fragments().is_empty());
                    for row in finite.final_inline_fragments() {
                        let fragment = row.fragment();
                        let line = fragment.finalized_fragment().finalized_line();
                        assert_eq!(line.prepared_text(), *prepared);
                        let selected = line.selected_line();
                        let request = selected.request();
                        if fragment.line_index() == 0
                            && let Some(cut) = expected_cut
                        {
                            let text::PreparedTermination::Conditional(offer) =
                                &request.termination
                            else {
                                panic!("the overflowing word requires an issued conditional cut");
                            };
                            assert_eq!(offer.boundary().view().source_after().get(), cut);
                            assert_eq!(offer.purpose(), text::ConditionalPurpose::Overflow);
                        } else {
                            assert_eq!(request.context.end, text::LineEnd::Paragraph);
                        }
                    }
                }
                for (operation, expected) in [
                    (crate::SizingOperation::MinContent, minimum),
                    (crate::SizingOperation::MaxContent, 40.0),
                ] {
                    let intrinsic = compute::<S>(
                        &p,
                        rtl,
                        crate::AxisSizingRequestOf::new(
                            crate::AvailableSpaceOf::Indefinite,
                            operation,
                        ),
                    );
                    let root = readback::single_box_output(&intrinsic, 0, Phase::Unrounded);
                    assert_eq!(
                        root.size.width,
                        S::from_f64(expected),
                        "{before:?}/{after:?} {operation:?} rtl={rtl} nested={nested}"
                    );
                }
            }
        }
    }

    lane::<f32>();
    lane::<f64>();
}

#[test]
fn disappearing_space_uses_its_own_box_for_soft_fit_and_plaintext() {
    fn lane<S: LayoutScalar>() {
        for (nested, rtl) in [(false, false), (false, true), (true, false), (true, true)] {
            for (parts, space_box) in [(["A ", "A"], 0), (["A", " A"], 1)] {
                for (space_wrap, neighbor_wrap, expected_cut, minimum) in [
                    (Wrap::Word, Wrap::None, Some(2), 10.0),
                    (Wrap::None, Wrap::Word, None, 30.0),
                ] {
                    let mut styles = [neighbor_wrap; 2];
                    styles[space_box] = space_wrap;
                    let paragraph = capture(
                        parts,
                        styles.map(|wrap| text::Style {
                            wrap,
                            white_space: text::WhiteSpace::Collapse,
                            ..style(text::WordBreak::Normal, text::LineBreak::Normal, false)
                        }),
                        AHEM,
                        nested,
                        Wrap::Word,
                        rtl,
                        true,
                    );
                    let expected: std::collections::BTreeSet<_> =
                        expected_cut.into_iter().collect();
                    let ordinary = paragraph
                        .prepared
                        .opportunity_views()
                        .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
                        .map(|offer| offer.boundary().view().source_after().get())
                        .collect();
                    assert_eq!(
                        expected, ordinary,
                        "{parts:?} {space_wrap:?} rtl={rtl} nested={nested}"
                    );
                    let finite = compute::<S>(
                        &paragraph,
                        rtl,
                        crate::AxisSizingRequestOf::new(
                            crate::AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(S::from_f64(10.0)).unwrap(),
                            ),
                            crate::SizingOperation::Normal,
                        ),
                    );
                    let layout = finite.placed_text()[0].layout();
                    let ranges = layout
                        .lines()
                        .iter()
                        .map(|line| line.range())
                        .collect::<Vec<_>>();
                    assert_eq!(layout.source().text().code_units(), [65, 32, 65]);
                    super::whitespace_modes::assert_copy(layout, 3, "A A");
                    let expected = if expected_cut.is_some() {
                        vec![text::Range::new(0, 2), text::Range::new(2, 3)]
                    } else {
                        vec![text::Range::new(0, 3)]
                    };
                    assert_eq!(
                        ranges, expected,
                        "{parts:?} {space_wrap:?} rtl={rtl} nested={nested}"
                    );
                    for row in finite.final_inline_fragments() {
                        let fragment = row.fragment();
                        let line = fragment.finalized_fragment().finalized_line();
                        assert_eq!(line.prepared_text(), paragraph.prepared);
                        assert_eq!(
                            line.feedback().advance,
                            if expected_cut.is_some() { 10.0 } else { 30.0 }
                        );
                    }
                    for (operation, expected) in [
                        (crate::SizingOperation::MinContent, minimum),
                        (crate::SizingOperation::MaxContent, 30.0),
                    ] {
                        let intrinsic = compute::<S>(
                            &paragraph,
                            rtl,
                            crate::AxisSizingRequestOf::new(
                                crate::AvailableSpaceOf::Indefinite,
                                operation,
                            ),
                        );
                        let root = readback::single_box_output(&intrinsic, 0, Phase::Unrounded);
                        assert_eq!(
                            root.size.width,
                            S::from_f64(expected),
                            "{parts:?} {space_wrap:?} {operation:?} rtl={rtl} nested={nested}"
                        );
                    }
                }
            }
        }
    }

    lane::<f32>();
    lane::<f64>();
}
