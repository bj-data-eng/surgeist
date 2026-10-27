//! #75 logical following-box ownership composed through real Text and Layout.
//! Sources are original occurrences in complete sibling/nested inline captures.
//! Font glyph0 below qualifies actual .notdef geometry, never script coverage.

use super::*;
use std::sync::Arc;
use surgeist_text as text;

#[path = "boundary_owner_composition/shared_unit_endpoints.rs"]
mod shared_unit_endpoints;

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
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
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
    let analyzed =
        text::AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
            .unwrap();
    // The only cross-origin NCA is the captured outer inline (nested) or block
    // (siblings). Same-origin pairs remain in their own Word-wrapping child.
    let policies = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| {
            let wrap = match (r.before(), r.after()) {
                (
                    text::WrappingNeighbor::Text { origin: a, .. },
                    text::WrappingNeighbor::Text { origin: b, .. },
                ) if a == b => Wrap::Word,
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
                Some(if a == b { Wrap::Word } else { ancestor })
            );
        }
    }
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let captured = prepared.captured_paragraph().unwrap();
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
                InlineBoundaryInputOf::new(
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
                .with_direction(if rtl { Direction::Rtl } else { Direction::Ltr }),
            ),
        );
    }
    tree
}

fn compute<S: LayoutScalar>(
    p: &TextParagraphBinding<u32>,
    rtl: bool,
    width: AvailableOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = tree::<S>(p, rtl, &[]);
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.clone()],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap(),
    )
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
    let actual = compute::<S>(p, rtl, AvailableOf::definite(S::from_f64(10.0)));
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
            AvailableOf::MIN_CONTENT,
            if cut.is_some() { 10.0 } else { 20.0 },
        ),
        (AvailableOf::MAX_CONTENT, 20.0),
    ] {
        let batch = compute::<S>(p, rtl, width);
        assert_eq!(
            batch.placed_text()[0].layout().source().text().code_units(),
            source.encode_utf16().collect::<Vec<_>>()
        );
        let root = batch
            .unrounded_entries()
            .iter()
            .find(|row| row.node() == 0)
            .unwrap()
            .output();
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
                        scopes: Vec::new(),
                        child_order: Vec::new(),
                    },
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
