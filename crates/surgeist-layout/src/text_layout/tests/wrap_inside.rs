//! Independent #65 box-priority acceptance through real native Layout inputs.
//! Captures below supply original event topology, never shaped widths or cuts.

use super::*;
use std::{cell::Cell, collections::HashMap, ops::Range, sync::Arc};
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const PHRASE: &[u8] =
    include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf");

struct Captured {
    binding: TextParagraphBinding<u32>,
    scopes: Vec<text::CapturedScope>,
}

#[derive(Clone, Copy)]
struct CaptureProfile {
    rtl: bool,
    endpoints: bool,
    topology: text::SourceProcessingTopology,
}

fn break_all() -> text::Style {
    text::Style {
        word_break: text::WordBreak::BreakAll,
        hyphens: text::Hyphens::None,
        white_space: text::WhiteSpace::Preserve,
        ..ahem_style(10.0, 10.0)
    }
}

/// Ranges are explicitly authored nested/disjoint inline boxes, outer first.
/// Every split is an actual scalar boundary. No inline endpoint is fabricated.
fn captured(
    raw: &str,
    style: text::Style,
    ranges: &[Range<usize>],
    extra: &[(usize, ParagraphControlKind)],
    rtl: bool,
    endpoints: bool,
    font: &[u8],
) -> Captured {
    captured_with_profile(
        raw,
        style,
        ranges,
        extra,
        CaptureProfile {
            rtl,
            endpoints,
            topology: text::SourceProcessingTopology::Complete,
        },
        font,
    )
}

fn captured_with_profile(
    raw: &str,
    style: text::Style,
    ranges: &[Range<usize>],
    extra: &[(usize, ParagraphControlKind)],
    profile: CaptureProfile,
    font: &[u8],
) -> Captured {
    use text::SourceProcessingEvent as E;
    let CaptureProfile {
        rtl,
        endpoints,
        topology,
    } = profile;
    let units: Vec<_> = raw.encode_utf16().collect();
    let direction = if rtl {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    };
    let mut source = Source::new(raw);
    source.span(text::Range::new(0, units.len()), style.clone());
    let mut scopes = vec![text::SourceProcessingScope::Block {
        topology,
        trim: Default::default(),
        base_style: style.clone(),
        used_direction: direction,
    }];
    scopes.extend(ranges.iter().map(|_| text::SourceProcessingScope::Inline {
        trim: Default::default(),
        style: Some(style.clone()),
        boundaries: None,
    }));
    let mut cuts = vec![0, units.len()];
    cuts.extend(ranges.iter().flat_map(|r| [r.start, r.end]));
    cuts.extend(extra.iter().map(|(at, _)| *at));
    cuts.sort_unstable();
    cuts.dedup();
    let mut controls = Vec::new();
    let mut pair_starts = vec![0; ranges.len()];
    let mut origins = Vec::new();
    let mut connections = Vec::new();
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    for (position, &at) in cuts.iter().enumerate() {
        for (i, range) in ranges.iter().enumerate().rev() {
            if range.end == at {
                if endpoints {
                    let end = controls.len();
                    controls.push(text::ParagraphControl {
                        at,
                        kind: ParagraphControlKind::InlineBoundaryEnd,
                    });
                    events.push(E::Control {
                        control: end,
                        affinity: text::Affinity::After,
                    });
                    let text::SourceProcessingScope::Inline { boundaries, .. } = &mut scopes[i + 1]
                    else {
                        unreachable!()
                    };
                    *boundaries = Some(text::SourceProcessingScopeBoundaries {
                        start: pair_starts[i],
                        end,
                    });
                }
                events.push(E::ExitScope(i + 1));
            }
        }
        if position > 0 && position + 1 < cuts.len() {
            let connection = connections.len();
            connections.push(text::SourceProcessingConnection {
                before_origin: origins.len() - 1,
                after_origin: origins.len(),
                kind: text::ParagraphConnectionKind::Continuous,
            });
            events.push(E::Connection(connection));
        }
        for (i, range) in ranges.iter().enumerate() {
            if range.start == at {
                events.push(E::EnterScope(i + 1));
                if endpoints {
                    pair_starts[i] = controls.len();
                    controls.push(text::ParagraphControl {
                        at,
                        kind: ParagraphControlKind::InlineBoundaryStart,
                    });
                    events.push(E::Control {
                        control: pair_starts[i],
                        affinity: text::Affinity::After,
                    });
                }
            }
        }
        for (_, kind) in extra.iter().filter(|(point, _)| *point == at) {
            let control = controls.len();
            controls.push(text::ParagraphControl {
                at,
                kind: kind.clone(),
            });
            events.push(E::Control {
                control,
                affinity: text::Affinity::After,
            });
        }
        if let Some(&end) = cuts.get(position + 1) {
            let original = Source::identified(
                String::from_utf16(&units[at..end]).unwrap(),
                text::Id::from_u64(650_000 + origins.len() as u64),
                31,
            );
            let origin = origins.len();
            origins.push(text::ParagraphOrigin {
                text_node: None,
                local: text::Range::new(0, end - at),
                source: original,
                aggregate: text::Range::new(at, end),
                kind: text::ParagraphOriginKind::Authored,
            });
            events.push(E::Origin {
                origin,
                affinity: text::Affinity::After,
            });
        }
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let nodes = events
        .iter()
        .filter_map(|e| match e {
            E::Origin { origin, .. } => Some(1000 + *origin as u32),
            E::Control { control, .. } => Some(10000 + *control as u32),
            _ => None,
        })
        .collect();
    let boundary_policies = (0..connections.len())
        .map(|i| text::ParagraphBoundaryPolicy {
            definition: text::ParagraphBoundaryDefinition::Connection(i),
            preceding_wrap: style.wrap,
            following_wrap: style.wrap,
            ancestor_wrap: style.wrap,
        })
        .collect();
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            computed_line_boundaries: vec![],
            source,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins,
            connections,
            controls,
            boundary_policies,
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
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, style.wrap);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let scopes = prepared
        .captured_paragraph()
        .unwrap()
        .scope_intersections()
        .map(|i| i.scope())
        .filter(|s| s.role() == text::ScopeRole::Inline)
        .collect();
    let origins = prepared
        .origin_views()
        .enumerate()
        .map(|(i, origin)| {
            assert_eq!(
                origin.source().id(),
                Some(text::Id::from_u64(650_000 + i as u64))
            );
            assert_eq!(origin.source().revision(), 31);
            TextOriginBinding {
                origin: origin.handle(),
                node: 1000 + i as u32,
            }
        })
        .collect();
    let controls = prepared
        .control_views()
        .enumerate()
        .map(|(i, control)| TextControlBinding {
            occurrence: control.handle(),
            node: 10000 + i as u32,
        })
        .collect();
    Captured {
        binding: TextParagraphBinding {
            prepared,
            container: 0,
            nodes,
            origins,
            controls,
            atomics: vec![],
            line_strut: None,
        },
        scopes,
    }
}

fn text_tree<S: LayoutScalar>(
    binding: &TextParagraphBinding<u32>,
    rtl: bool,
) -> PublicLayoutTreeOf<S> {
    let direction = if rtl { Direction::Rtl } else { Direction::Ltr };
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, binding.nodes.iter().copied())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction,
                ..NodeInputOf::default()
            },
        );
    for row in &binding.origins {
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
        );
    }
    for row in &binding.controls {
        let input = match row.occurrence.view().kind() {
            ParagraphControlKind::InlineBoundaryStart | ParagraphControlKind::InlineBoundaryEnd => {
                let kind =
                    if *row.occurrence.view().kind() == ParagraphControlKind::InlineBoundaryStart {
                        InlineBoundaryKind::Start
                    } else {
                        InlineBoundaryKind::End
                    };
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::new(
                        kind,
                        InlineMetricsOf::try_new(S::from_f64(8.0), S::from_f64(10.0)).unwrap(),
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .with_direction(direction),
                )
            }
            _ => LayoutInputOf::text_control(TextControlParticipation::new(row.occurrence.clone())),
        };
        tree = tree.children(row.node, []).input(row.node, input);
    }
    tree
}

fn request_at<S: LayoutScalar>(width: AvailableOf<S>) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap()
}

fn policies(fixture: &Captured, values: &[WrapInside]) -> Vec<WrappingScopeInput<u32>> {
    assert_eq!(fixture.scopes.len(), values.len());
    fixture
        .scopes
        .iter()
        .cloned()
        .zip(values)
        .map(|(scope, &wrap_inside)| WrappingScopeInput::Captured {
            edges: Default::default(),
            scope,
            wrap_inside,
        })
        .collect()
}

fn compute<S: LayoutScalar>(
    fixture: &Captured,
    values: &[WrapInside],
    width: AvailableOf<S>,
    rtl: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = text_tree(&fixture.binding, rtl);
    let input = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        crate::WrappingInput {
            scopes: policies(fixture, values),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    compute_layout_with_flow(&input, 0, request_at(width)).unwrap()
}

fn cuts<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<usize> {
    assert_eq!(batch.placed_text().len(), 1);
    batch.placed_text()[0]
        .layout()
        .lines()
        .iter()
        .map(|line| line.range().end)
        .collect()
}

fn finalized<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    lines
        .into_iter()
        .map(|line| line.expect("each nonempty authored line retains real fragment custody"))
        .collect()
}

fn assert_ascii_custody<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    fixture: &Captured,
    raw: &str,
) {
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text(), &text::Text::from(raw));
    let rows = finalized(batch);
    let mut ranges = Vec::new();
    for line in rows {
        assert_eq!(line.prepared_text(), fixture.binding.prepared);
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            for glyph in run.glyphs() {
                assert_eq!(glyph.advance(), 10.0);
                ranges.push(glyph.range());
            }
        }
    }
    ranges.sort_by_key(|r| r.start);
    assert_eq!(
        ranges,
        (0..raw.len())
            .map(|i| text::Range::new(i, i + 1))
            .collect::<Vec<_>>()
    );
}

fn outside_and_nested<S: LayoutScalar>() {
    for rtl in [false, true] {
        let f = captured(
            "ABCDEFGH",
            break_all(),
            std::slice::from_ref(&(2..6)),
            &[],
            rtl,
            false,
            AHEM,
        );
        let auto = compute::<S>(
            &f,
            &[WrapInside::Auto],
            AvailableOf::definite(S::from_f64(40.0)),
            rtl,
        );
        let avoid = compute::<S>(
            &f,
            &[WrapInside::Avoid],
            AvailableOf::definite(S::from_f64(40.0)),
            rtl,
        );
        assert_eq!(cuts(&auto), [4, 8]);
        assert_eq!(cuts(&avoid), [2, 6, 8]);
        assert_ascii_custody(&avoid, &f, "ABCDEFGH");
        // The same actual canonical CDEF box also has genuine endpoint nodes.
        // Request its NativePair form and exercise selection/placement, rather
        // than treating successful constructor admission as behavior evidence.
        let physical = captured(
            "ABCDEFGH",
            break_all(),
            std::slice::from_ref(&(2..6)),
            &[],
            rtl,
            true,
            AHEM,
        );
        let tree = text_tree::<S>(&physical.binding, rtl);
        let flow = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![physical.binding.clone()],
            crate::WrappingInput {
                scopes: vec![pair(10000, 10001, WrapInside::Avoid)],
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let matched = compute_layout_with_flow(
            &flow,
            0,
            request_at(AvailableOf::definite(S::from_f64(40.0))),
        )
        .unwrap();
        assert_eq!(cuts(&matched), [2, 6, 8]);
        assert_ascii_custody(&matched, &physical, "ABCDEFGH");
        for (index, line) in matched.placed_text()[0].layout().lines().iter().enumerate() {
            assert_eq!(line.bounds().origin.y, index as f64 * 10.0);
            assert_eq!(line.bounds().size.height, 10.0);
            assert_eq!(line.baseline(), 8.0 + index as f64 * 10.0);
        }
        assert_eq!(
            finalized(&matched)
                .iter()
                .map(|line| line.measurements().fit().advance())
                .collect::<Vec<_>>(),
            [20.0, 40.0, 20.0],
        );
        let nested = captured("ABCDEF", break_all(), &[0..6, 2..6], &[], rtl, false, AHEM);
        assert_eq!(
            cuts(&compute::<S>(
                &nested,
                &[WrapInside::Avoid; 2],
                AvailableOf::definite(S::from_f64(40.0)),
                rtl
            )),
            [2, 6]
        );
        // Auto is noninherited: this inner value does not cancel the outer box.
        assert_eq!(
            cuts(&compute::<S>(
                &nested,
                &[WrapInside::Avoid, WrapInside::Auto],
                AvailableOf::definite(S::from_f64(40.0)),
                rtl
            )),
            [4, 6]
        );
        let noninherited = captured(
            "ABCDEFGH",
            break_all(),
            &[2..6, 3..6],
            &[],
            rtl,
            false,
            AHEM,
        );
        assert_eq!(
            cuts(&compute::<S>(
                &noninherited,
                &[WrapInside::Avoid, WrapInside::Auto],
                AvailableOf::definite(S::from_f64(40.0)),
                rtl
            )),
            [2, 6, 8],
        );
    }
}

#[test]
fn outside_and_outer_points_win_without_losing_original_sources() {
    outside_and_nested::<f32>();
    outside_and_nested::<f64>();
}

fn necessary_and_intrinsic<S: LayoutScalar>() {
    let f = captured(
        "ABCDEF",
        break_all(),
        std::slice::from_ref(&(0..6)),
        &[],
        false,
        false,
        AHEM,
    );
    assert_eq!(
        cuts(&compute::<S>(
            &f,
            &[WrapInside::Avoid],
            AvailableOf::definite(S::from_f64(20.0)),
            false
        )),
        [2, 4, 6]
    );
    for (available, expected) in [
        (AvailableOf::MIN_CONTENT, 10.0),
        (AvailableOf::MAX_CONTENT, 60.0),
    ] {
        for policy in [WrapInside::Auto, WrapInside::Avoid] {
            let batch = compute::<S>(&f, &[policy], available, false);
            let root = batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output();
            assert_eq!(root.size.width, S::from_f64(expected));
        }
    }
}

#[test]
fn avoidance_yields_to_necessary_legal_cuts_and_preserves_intrinsics() {
    necessary_and_intrinsic::<f32>();
    necessary_and_intrinsic::<f64>();
}

fn incomparable<S: LayoutScalar>() {
    let normal = text::Style {
        word_break: text::WordBreak::Normal,
        ..break_all()
    };
    let f = captured(
        "ABCDEF",
        normal.clone(),
        &[0..3, 3..6, 4..6],
        &[
            (
                2,
                ParagraphControlKind::Wbr {
                    style: Box::new(normal.clone()),
                },
            ),
            (
                5,
                ParagraphControlKind::Wbr {
                    style: Box::new(normal),
                },
            ),
        ],
        false,
        false,
        AHEM,
    );
    assert_eq!(
        cuts(&compute::<S>(
            &f,
            &[WrapInside::Avoid; 3],
            AvailableOf::definite(S::from_f64(50.0)),
            false
        )),
        [5, 6]
    );
}

#[test]
fn unrelated_scope_chains_preserve_normal_preference_instead_of_depth_ranking() {
    incomparable::<f32>();
    incomparable::<f64>();
}

fn restrictions<S: LayoutScalar>() {
    let normal = text::Style {
        word_break: text::WordBreak::Normal,
        ..break_all()
    };
    let f = captured(
        "ABCDEF",
        normal.clone(),
        std::slice::from_ref(&(2..4)),
        &[],
        false,
        false,
        AHEM,
    );
    assert_eq!(
        cuts(&compute::<S>(
            &f,
            &[WrapInside::Avoid],
            AvailableOf::definite(S::from_f64(20.0)),
            false
        )),
        [6]
    );
    let f = captured(
        "AB\nCD",
        break_all(),
        std::slice::from_ref(&(0..5)),
        &[],
        false,
        false,
        AHEM,
    );
    let batch = compute::<S>(
        &f,
        &[WrapInside::Avoid],
        AvailableOf::definite(S::from_f64(100.0)),
        false,
    );
    assert_eq!(cuts(&batch), [3, 5]);
    assert_eq!(
        finalized(&batch)[0].selected_line().request().context.end,
        text::LineEnd::Forced
    );
    let f = captured(
        "AB CD",
        text::Style {
            wrap: Wrap::None,
            ..normal.clone()
        },
        std::slice::from_ref(&(0..5)),
        &[],
        false,
        false,
        AHEM,
    );
    assert_eq!(
        cuts(&compute::<S>(
            &f,
            &[WrapInside::Avoid],
            AvailableOf::definite(S::from_f64(20.0)),
            false
        )),
        [5]
    );
    for raw in ["A\u{2060}B", "A\u{a0}B"] {
        let f = captured(
            raw,
            normal.clone(),
            std::slice::from_ref(&(0..3)),
            &[],
            false,
            false,
            AHEM,
        );
        assert_eq!(
            cuts(&compute::<S>(
                &f,
                &[WrapInside::Avoid],
                AvailableOf::definite(S::from_f64(10.0)),
                false
            )),
            [3]
        );
    }
    // The genuine inline boundary lies inside one EGC. No interior cut appears.
    let f = captured(
        "A\u{301}BC",
        break_all(),
        std::slice::from_ref(&(1..3)),
        &[],
        false,
        false,
        AHEM,
    );
    let batch = compute::<S>(
        &f,
        &[WrapInside::Avoid],
        AvailableOf::definite(S::from_f64(10.0)),
        false,
    );
    assert_eq!(cuts(&batch), [2, 3, 4]);
    assert!(!cuts(&batch).contains(&1));
}

#[test]
fn avoidance_keeps_forced_nowrap_glue_and_whole_tcu_constraints() {
    restrictions::<f32>();
    restrictions::<f64>();
}

fn margin<S: LayoutScalar>(rtl: bool) -> Edges<S> {
    let mut edges = Edges::ZERO;
    edges.left = S::from_f64(if rtl { -5.0 } else { 5.0 });
    edges.right = S::from_f64(if rtl { 5.0 } else { -5.0 });
    edges
}

fn numeric_tree<S: LayoutScalar>(
    permissions: [InlineBreakOpportunity; 3],
    rtl: bool,
    outer: bool,
) -> PublicLayoutTreeOf<S> {
    let direction = if rtl { Direction::Rtl } else { Direction::Ltr };
    let children = if outer {
        vec![6, 1, 2, 3, 4, 5, 7]
    } else {
        vec![1, 2, 3, 4, 5]
    };
    let mut tree = PublicLayoutTreeOf::new().children(0, children).style(
        0,
        NodeInputOf {
            display: Display::Block,
            direction,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(25.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        },
    );
    for (node, following) in [1, 3, 4].into_iter().zip(permissions) {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::InlineBlock,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                margin: margin::<S>(rtl).map(LengthAutoOf::px),
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    text::BidiLevel::try_new(0).unwrap(),
                    following,
                )),
                ..NodeInputOf::default()
            },
        );
    }
    for (node, kind) in [
        (2, InlineBoundaryKind::Start),
        (5, InlineBoundaryKind::End),
        (6, InlineBoundaryKind::Start),
        (7, InlineBoundaryKind::End),
    ] {
        if outer || node < 6 {
            tree = tree.children(node, []).input(
                node,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::new(
                        kind,
                        InlineMetricsOf::try_new(S::from_f64(10.0), S::from_f64(10.0)).unwrap(),
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .with_direction(direction),
                ),
            );
        }
    }
    tree
}

fn pair(start: u32, end: u32, policy: WrapInside) -> WrappingScopeInput<u32> {
    WrappingScopeInput::NativePair {
        edges: Default::default(),
        start,
        end,
        wrap_inside: policy,
    }
}

fn numeric<S: LayoutScalar>() {
    use InlineBreakOpportunity::{Allowed, Mandatory, Prohibited};
    for rtl in [false, true] {
        for (permissions, policy, outer, first) in [
            ([Allowed; 3], WrapInside::Auto, false, 2),
            ([Allowed; 3], WrapInside::Avoid, false, 1),
            ([Allowed; 3], WrapInside::Avoid, true, 1),
            ([Prohibited, Allowed, Allowed], WrapInside::Avoid, false, 2),
            ([Allowed, Mandatory, Allowed], WrapInside::Avoid, false, 2),
        ] {
            let tree = numeric_tree::<S>(permissions, rtl, outer);
            let mut scopes = vec![pair(2, 5, policy)];
            if outer {
                scopes.push(pair(6, 7, WrapInside::Avoid));
            }
            let flow = LayoutFlowInput::try_new(
                &tree,
                0,
                vec![],
                crate::WrappingInput {
                    scopes,
                    child_order: Vec::new(),
                },
            )
            .unwrap();
            let batch = compute_layout_with_flow(
                &flow,
                0,
                request_at(AvailableOf::definite(S::from_f64(25.0))),
            )
            .unwrap();
            assert!(batch.placed_text().is_empty());
            let output = |node| {
                batch
                    .final_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output()
            };
            assert_eq!(output(0).size.height, S::from_f64(20.0));
            for (i, node) in [1, 3, 4].into_iter().enumerate() {
                let (slot, y) = if i < first {
                    (i, 0.0)
                } else {
                    (i - first, 10.0)
                };
                let x = if rtl {
                    10.0 - slot as f64 * 10.0
                } else {
                    5.0 + slot as f64 * 10.0
                };
                assert_eq!(
                    output(node).location,
                    Point::new(S::from_f64(x), S::from_f64(y))
                );
                assert_eq!(output(node).size, Size::splat(S::from_f64(10.0)));
                assert_eq!(output(node).margin, margin(rtl));
            }
            // Real output membership yields actual used margin flow 10/20,
            // distinct from the origin-anchored placed border envelope15/25.
            for (members, expected_flow) in [
                (&[1, 3, 4][..first], first as f64 * 10.0),
                (&[1, 3, 4][first..], (3 - first) as f64 * 10.0),
            ] {
                let flow = members.iter().fold(S::ZERO, |sum, &node| {
                    let row = output(node);
                    sum + row.size.width + row.margin.left + row.margin.right
                });
                assert_eq!(flow, S::from_f64(expected_flow));
                let origin = if rtl { S::from_f64(25.0) } else { S::ZERO };
                let (left, right) =
                    members
                        .iter()
                        .fold((origin, origin), |(left, right), &node| {
                            let row = output(node);
                            (
                                left.min(row.location.x),
                                right.max(row.location.x + row.size.width),
                            )
                        });
                assert_eq!(right - left, S::from_f64(expected_flow + 5.0));
            }
        }
    }
}

#[test]
fn numeric_signed_margin_boxes_use_real_scopes_permissions_and_placed_geometry() {
    numeric::<f32>();
    numeric::<f64>();
}

#[test]
fn malformed_native_scope_definitions_refuse_without_guessing_flat_pairs() {
    let tree = numeric_tree::<f64>([InlineBreakOpportunity::Allowed; 3], false, false);
    for (definitions, expected) in [
        (
            vec![pair(1, 5, WrapInside::Avoid)],
            WrappingScopeError::EndpointRole,
        ),
        (
            vec![pair(2, 99, WrapInside::Avoid)],
            WrappingScopeError::NodeNotReachable,
        ),
        (
            vec![pair(2, 5, WrapInside::Avoid); 2],
            WrappingScopeError::DuplicateScope,
        ),
        (
            vec![pair(2, 5, WrapInside::Avoid), pair(2, 5, WrapInside::Auto)],
            WrappingScopeError::ConflictingScope,
        ),
    ] {
        let error = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![],
            crate::WrappingInput {
                scopes: definitions,
                child_order: Vec::new(),
            },
        )
        .err()
        .expect("invalid normalized scope refuses");
        assert!(
            matches!(error.kind(), LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(reason)) if *reason == expected),
            "{error:?}"
        );
    }
    let crossing = numeric_tree::<f64>([InlineBreakOpportunity::Allowed; 3], false, true)
        .children(0, [6, 1, 2, 3, 7, 4, 5]);
    let error = LayoutFlowInput::try_new(
        &crossing,
        0,
        vec![],
        crate::WrappingInput {
            scopes: vec![pair(6, 7, WrapInside::Avoid), pair(2, 5, WrapInside::Avoid)],
            child_order: Vec::new(),
        },
    )
    .err()
    .expect("crossing actual endpoint pairs refuse");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::CrossingScopes
        ))
    ));
}

#[test]
fn foreign_canonical_scope_is_not_reinterpreted_by_matching_source_or_ordinal() {
    let one = captured(
        "ABCDEF",
        break_all(),
        std::slice::from_ref(&(2..6)),
        &[],
        false,
        false,
        AHEM,
    );
    let other = captured(
        "ABCDEF",
        break_all(),
        std::slice::from_ref(&(2..6)),
        &[],
        false,
        false,
        AHEM,
    );
    let tree = text_tree::<f64>(&one.binding, false);
    let error = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![one.binding.clone()],
        crate::WrappingInput {
            scopes: policies(&other, &[WrapInside::Avoid]),
            child_order: Vec::new(),
        },
    )
    .err()
    .expect("different capture cannot certify this source");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::ForeignCapture
        ))
    ));
    let block = one.scopes[0].parent().unwrap();
    let error = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![one.binding.clone()],
        crate::WrappingInput {
            scopes: vec![WrappingScopeInput::Captured {
                edges: Default::default(),
                scope: block,
                wrap_inside: WrapInside::Avoid,
            }],
            child_order: Vec::new(),
        },
    )
    .err()
    .expect("block scope is not an inline declaration");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::NonInlineScope
        ))
    ));
    let numeric = numeric_tree::<f64>([InlineBreakOpportunity::Allowed; 3], false, false);
    let error = LayoutFlowInput::try_new(
        &numeric,
        0,
        vec![],
        crate::WrappingInput {
            scopes: policies(&one, &[WrapInside::Avoid]),
            child_order: Vec::new(),
        },
    )
    .err()
    .expect("numeric flow does not manufacture capture");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::MissingCapture
        ))
    ));
}

#[test]
fn native_pair_roles_do_not_override_a_different_complete_canonical_box() {
    let f = captured(
        "ABCDEFGH",
        break_all(),
        &[0..2, 2..6],
        &[],
        false,
        true,
        AHEM,
    );
    let tree = text_tree::<f64>(&f.binding, false);
    // These genuine Start/End nodes are ordered and noncrossing but belong
    // to DIFFERENT canonical boxes. An asserted enclosing box cannot replace
    // the complete source capture's actual pairing.
    let error = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![f.binding.clone()],
        crate::WrappingInput {
            scopes: vec![pair(10000, 10003, WrapInside::Avoid)],
            child_order: Vec::new(),
        },
    )
    .err()
    .expect("known canonical pairing is authoritative");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::CaptureAssociation
        ))
    ));
    LayoutFlowInput::try_new(
        &tree,
        0,
        vec![f.binding.clone()],
        crate::WrappingInput {
            scopes: vec![pair(10000, 10001, WrapInside::Avoid)],
            child_order: Vec::new(),
        },
    )
    .expect("the actual canonical pair remains admitted");

    let unmatched = captured(
        "ABCDEF",
        break_all(),
        std::slice::from_ref(&(0..6)),
        &[
            (0, ParagraphControlKind::InlineBoundaryStart),
            (6, ParagraphControlKind::InlineBoundaryEnd),
        ],
        false,
        false,
        AHEM,
    );
    assert!(unmatched.scopes[0].entry_control().is_none());
    assert!(unmatched.scopes[0].exit_control().is_none());
    let tree = text_tree::<f64>(&unmatched.binding, false);
    // Complete capture retained both actual standalone control events, but
    // supplied no canonical pairing for them. Their roles/order alone do not
    // authorize inventing a box. Neutral construction still permits controls.
    LayoutFlowInput::try_new(
        &tree,
        0,
        vec![unmatched.binding.clone()],
        crate::WrappingInput {
            scopes: vec![],
            child_order: Vec::new(),
        },
    )
    .expect("standalone source controls remain a valid neutral input");
    let error = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![unmatched.binding.clone()],
        crate::WrappingInput {
            scopes: vec![pair(10000, 10001, WrapInside::Avoid)],
            child_order: Vec::new(),
        },
    )
    .err()
    .expect("the requested pair has no canonical box in complete capture");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::CaptureAssociation
        ))
    ));
}

#[test]
fn captured_physical_interior_tcu_profile_retains_its_existing_precise_refusal() {
    let f = captured(
        "A\u{301}B",
        break_all(),
        &[0..1, 1..3],
        &[],
        false,
        true,
        AHEM,
    );
    let tree = text_tree::<f64>(&f.binding, false);
    let error = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![f.binding.clone()],
        crate::WrappingInput {
            scopes: policies(&f, &[WrapInside::Avoid; 2]),
            child_order: Vec::new(),
        },
    )
    .err()
    .expect("known paired cross-origin TCU profile remains unsupported");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ParticipantOrder
        ))
    ));
}

fn phrase<S: LayoutScalar>() {
    let style = text::Style {
        font: text::Font::new().family("Surgeist Phrase Ahem"),
        locale: Some("ja-Jpan".into()),
        word_break: text::WordBreak::AutoPhrase,
        hyphens: text::Hyphens::None,
        overflow_wrap: text::OverflowWrap::Anywhere,
        ..ahem_style(10.0, 10.0)
    };
    // Published Japanese provider row: 日本へ | ようこそ. The first phrase
    // is30 and overflows the fresh20 frame, authorizing Normal restoration.
    let f = captured(
        "日本へようこそ",
        style,
        std::slice::from_ref(&(1..3)),
        &[],
        false,
        false,
        PHRASE,
    );
    let batch = compute::<S>(
        &f,
        &[WrapInside::Avoid],
        AvailableOf::definite(S::from_f64(20.0)),
        false,
    );
    assert_eq!(cuts(&batch), [1, 3, 5, 7]);
    let rows = finalized(&batch);
    assert!(
        matches!(&rows[0].selected_line().request().termination, text::PreparedTermination::Conditional(point) if point.stage() == text::ConditionalStage::Normal)
    );
    // First restored cut1 is outside, restored cut2 is inside the avoided box.
    // No arbitrary stage may bypass this fitting Normal candidate.
    assert_eq!(rows[0].measurements().fit().advance(), 10.0);
    assert_eq!(rows[1].measurements().fit().advance(), 20.0);
}

#[test]
fn genuine_phrase_restoration_keeps_stage_order_and_ranks_actual_restored_cuts() {
    phrase::<f32>();
    phrase::<f64>();
}

fn hanging<S: LayoutScalar>() {
    let f = captured(
        "AB.CDEF",
        text::Style {
            // Explicit Anywhere supplies the genuine cut before IS period;
            // BreakAll alone must retain the ordinary punctuation restriction.
            line_break: text::LineBreak::Anywhere,
            hanging_punctuation: text::HangingPunctuation::new(
                false,
                Some(text::HangingPunctuationEnd::ForceEnd),
                false,
            ),
            ..break_all()
        },
        std::slice::from_ref(&(2..7)),
        &[],
        false,
        false,
        AHEM,
    );
    let auto = compute::<S>(
        &f,
        &[WrapInside::Auto],
        AvailableOf::definite(S::from_f64(30.0)),
        false,
    );
    let avoid = compute::<S>(
        &f,
        &[WrapInside::Avoid],
        AvailableOf::definite(S::from_f64(30.0)),
        false,
    );
    assert_eq!(cuts(&auto), [3, 6, 7]);
    assert_eq!(finalized(&auto)[0].feedback().advance, 30.0);
    assert_eq!(finalized(&auto)[0].measurements().fit().advance(), 20.0);
    assert_eq!(cuts(&avoid), [2, 5, 7]);
    assert_eq!(finalized(&avoid)[0].measurements().fit().advance(), 20.0);
    assert_ascii_custody(&avoid, &f, "AB.CDEF");
}

#[test]
fn priority_consumes_actual_hanging_measurements_and_preserves_glyphs() {
    hanging::<f32>();
    hanging::<f64>();
}

fn spacing<S: LayoutScalar>() {
    let f = captured(
        "ABCDEF",
        text::Style {
            letter_spacing: 5.0,
            ..break_all()
        },
        std::slice::from_ref(&(2..6)),
        &[],
        false,
        false,
        AHEM,
    );
    // Each retained Ahem10 advance plus the authored5 letter spacing is15.
    // The scope comparison must use finalized measures, not glyph10 alone.
    let auto = compute::<S>(
        &f,
        &[WrapInside::Auto],
        AvailableOf::definite(S::from_f64(45.0)),
        false,
    );
    let avoid = compute::<S>(
        &f,
        &[WrapInside::Avoid],
        AvailableOf::definite(S::from_f64(45.0)),
        false,
    );
    assert_eq!(cuts(&auto), [3, 6]);
    assert_eq!(cuts(&avoid), [2, 5, 6]);
    assert_eq!(finalized(&auto)[0].measurements().fit().advance(), 45.0);
    assert_eq!(finalized(&avoid)[0].measurements().fit().advance(), 30.0);
    assert_ascii_custody(&avoid, &f, "ABCDEF");
}

#[test]
fn actual_letter_spacing_changes_fit_before_scope_preference() {
    spacing::<f32>();
    spacing::<f64>();
}

fn native_tree<S: LayoutScalar>(p: &PreparedParagraph) -> PublicLayoutTreeOf<S> {
    let binding = p.binding(0);
    let mut tree = text_tree::<S>(&binding, false);
    for atom in &binding.atomics {
        let size = atom.occurrence.view().original_box().size();
        tree = tree.children(atom.node, []).style(
            atom.node,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(size.width)),
                    PreferredSizeOf::px(S::from_f64(size.height)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn receiving<S: LayoutScalar>() {
    for placement in [
        text::HyphenationPlacement::LogicalBefore,
        text::HyphenationPlacement::LogicalAfter,
        text::HyphenationPlacement::BothSides,
    ] {
        let style = text::Style {
            hyphens: text::Hyphens::Manual,
            hyphenate_character: text::HyphenateCharacter::Auto,
            locale: Some("und-Latn".into()),
            native_hyphenation_convention: Some(
                text::NativeHyphenationConvention::language_script(
                    "und-Latn",
                    "Latn",
                    "A".into(),
                    placement,
                )
                .unwrap(),
            ),
            ..ahem_style(10.0, 10.0)
        };
        let p = PreparedParagraph::new(
            vec![
                ParagraphPart::Control {
                    node: 2,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphPart::text(
                    1,
                    Source::identified("AB\u{ad}CDE", text::Id::from_u64(650_900), 8),
                    style.clone(),
                ),
                ParagraphPart::Control {
                    node: 5,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            style,
            ParagraphBaseDirection::LeftToRight,
        );
        let tree = native_tree::<S>(&p);
        let flow = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![p.binding(0)],
            crate::WrappingInput {
                scopes: vec![pair(2, 5, WrapInside::Avoid)],
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &flow,
            0,
            request_at(AvailableOf::definite(S::from_f64(40.0))),
        )
        .unwrap();
        assert_eq!(cuts(&batch), [3, 6]);
        let rows = finalized(&batch);
        assert!(rows[0].selected_line().request().alternative.is_some());
        let after = matches!(
            placement,
            text::HyphenationPlacement::LogicalAfter | text::HyphenationPlacement::BothSides
        );
        assert_eq!(rows[0].hyphenation_continuation().is_some(), after);
        assert_eq!(
            rows[1].selected_line().request().incoming,
            rows[0].hyphenation_continuation()
        );
        assert_eq!(
            rows[0].feedback().advance,
            if placement == text::HyphenationPlacement::LogicalAfter {
                20.0
            } else {
                30.0
            }
        );
        assert_eq!(rows[1].feedback().advance, if after { 40.0 } else { 30.0 });
        assert_eq!(
            batch.placed_text()[0].layout().source().text(),
            &text::Text::from("AB\u{ad}CDE")
        );
    }
    for placement in [
        text::HyphenationPlacement::LogicalAfter,
        text::HyphenationPlacement::BothSides,
    ] {
        let style = text::Style {
            hyphens: text::Hyphens::Auto,
            locale: Some("en-US".into()),
            hyphenate_character: text::HyphenateCharacter::Auto,
            native_hyphenation_convention: Some(
                text::NativeHyphenationConvention::language_script(
                    "en-US",
                    "Latn",
                    "A".into(),
                    placement,
                )
                .unwrap(),
            ),
            ..ahem_style(10.0, 10.0)
        };
        let p = PreparedParagraph::new(
            vec![
                ParagraphPart::Control {
                    node: 2,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphPart::text(
                    1,
                    Source::identified("as\u{ad}sociate", text::Id::from_u64(650_902), 10),
                    style.clone(),
                ),
                ParagraphPart::Control {
                    node: 5,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            style,
            ParagraphBaseDirection::LeftToRight,
        );
        let word = p
            .prepared
            .word_views()
            .find(|word| word.query() == Some("associate"))
            .unwrap();
        assert_eq!(word.resource(), text::HyphenationResource::EnUs2005);
        assert_eq!(
            word.provenance(),
            Some(text::HyphenationProvenance::SourceException { entry: "associate" })
        );
        let tree = native_tree::<S>(&p);
        let flow = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![p.binding(0)],
            crate::WrappingInput {
                scopes: vec![pair(2, 5, WrapInside::Avoid)],
                child_order: Vec::new(),
            },
        )
        .unwrap();
        for (width, cut, automatic) in [(75.0, 5, true), (85.0, 3, false)] {
            let batch = compute_layout_with_flow(
                &flow,
                0,
                request_at(AvailableOf::definite(S::from_f64(width))),
            )
            .unwrap();
            let rows = finalized(&batch);
            assert_eq!(cuts(&batch), [cut, 10]);
            let selected = rows[0].selected_line();
            assert_eq!(
                matches!(
                    selected
                        .request()
                        .alternative
                        .as_ref()
                        .unwrap()
                        .view()
                        .definition(),
                    text::PreparedAlternativeDefinition::Automatic { .. }
                ),
                automatic
            );
            assert_eq!(
                rows[1].selected_line().request().incoming,
                rows[0].hyphenation_continuation()
            );
            // Full manual receiving target: seven letters70 plus the genuine
            // indication10. It fails75, so Auto is used; it fits85, so Manual wins.
            if !automatic {
                assert_eq!(rows[1].feedback().advance, 80.0);
            }
        }
    }
}

#[test]
fn full_manual_alternatives_and_real_receivers_retain_native_scope_custody() {
    receiving::<f32>();
    receiving::<f64>();
}

fn replacement_and_terminal<S: LayoutScalar>() {
    let style = text::Style {
        hyphens: text::Hyphens::Manual,
        ..ahem_style(10.0, 10.0)
    };
    let p = PreparedParagraph::with_replacements(
        vec![
            ParagraphPart::Control {
                node: 2,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphPart::text(
                1,
                Source::identified("AB\u{ad}CDE", text::Id::from_u64(650_901), 9),
                style.clone(),
            ),
            ParagraphPart::Control {
                node: 5,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
        vec![
            text::ParagraphReplacement::resolved_native_logical_before(
                text::ParagraphReplacementDefinition::Source {
                    origin: 0,
                    source: text::Range::new(2, 3),
                },
                3,
                vec![text::Text::from("")],
            )
            .unwrap(),
        ],
    );
    let tree = native_tree::<S>(&p);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(0)],
        crate::WrappingInput {
            scopes: vec![pair(2, 5, WrapInside::Avoid)],
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(
        &flow,
        0,
        request_at(AvailableOf::definite(S::from_f64(30.0))),
    )
    .unwrap();
    assert_eq!(cuts(&batch), [3, 6]);
    let rows = finalized(&batch);
    let selected = rows[0].selected_line();
    assert_eq!(
        selected
            .request()
            .alternative
            .as_ref()
            .unwrap()
            .view()
            .text(),
        &text::Text::from("")
    );
    assert_eq!(rows[0].feedback().advance, 20.0);
    assert_eq!(rows[1].feedback().advance, 30.0);
    assert_eq!(
        batch.placed_text()[0].layout().source().text(),
        &text::Text::from("AB\u{ad}CDE")
    );

    let f = captured(
        "AB\n",
        break_all(),
        std::slice::from_ref(&(0..3)),
        &[],
        false,
        false,
        AHEM,
    );
    let batch = compute::<S>(
        &f,
        &[WrapInside::Avoid],
        AvailableOf::definite(S::from_f64(100.0)),
        false,
    );
    assert_eq!(cuts(&batch), [3, 3]);
    assert_eq!(
        batch.placed_text()[0].layout().lines()[1]
            .bounds()
            .size
            .height,
        10.0
    );
}

#[test]
fn empty_native_replacement_and_genuine_terminal_empty_line_keep_their_owners() {
    replacement_and_terminal::<f32>();
    replacement_and_terminal::<f64>();
}

fn atomic_and_float<S: LayoutScalar>() {
    let style = text::Style {
        hyphens: text::Hyphens::None,
        ..ahem_style(10.0, 10.0)
    };
    let p = PreparedParagraph::new(
        vec![
            ParagraphPart::Control {
                node: 2,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphPart::text(1, Source::new("A\u{2060}"), style.clone()),
            ParagraphPart::Atomic {
                node: 9,
                size: text::Size::new(30.0, 10.0),
            },
            ParagraphPart::text(3, Source::new("B"), style.clone()),
            ParagraphPart::Control {
                node: 5,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let tree = native_tree::<S>(&p);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(0)],
        crate::WrappingInput {
            scopes: vec![pair(2, 5, WrapInside::Avoid)],
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(
        &flow,
        0,
        request_at(AvailableOf::definite(S::from_f64(20.0))),
    )
    .unwrap();
    assert_eq!(cuts(&batch), [2, 3]);
    assert_eq!(
        batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 9)
            .unwrap()
            .output()
            .size
            .width,
        S::from_f64(30.0)
    );

    let p = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("AB"), break_all()),
            ParagraphPart::Control {
                node: 2,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphPart::text(3, Source::new("CDEF"), break_all()),
            ParagraphPart::Control {
                node: 5,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        break_all(),
        ParagraphBaseDirection::LeftToRight,
    );
    let tree = native_tree::<S>(&p)
        .children(0, [90, 1, 2, 3, 5])
        .children(90, [])
        .style(
            90,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(0)],
        crate::WrappingInput {
            scopes: vec![pair(2, 5, WrapInside::Avoid)],
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(
        &flow,
        0,
        request_at(AvailableOf::definite(S::from_f64(60.0))),
    )
    .unwrap();
    // The actual first band is40: outside AB wins over inside ABCD. Below
    // the10-high float, the complete CDEF fits the real60-wide frame.
    assert_eq!(cuts(&batch), [2, 6]);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.lines()[0].bounds().origin.x, 20.0);
    assert_eq!(placed.lines()[1].bounds().origin.y, 10.0);
}

#[test]
fn glue_next_to_atomic_and_real_float_bands_keep_their_own_constraints() {
    atomic_and_float::<f32>();
    atomic_and_float::<f64>();
}

fn collapsed_and_explicit<S: LayoutScalar>() {
    let style = text::Style {
        word_break: text::WordBreak::Normal,
        white_space: text::WhiteSpace::Collapse,
        ..break_all()
    };
    let f = captured(
        "AB  CDEF",
        style.clone(),
        std::slice::from_ref(&(4..8)),
        &[],
        false,
        false,
        AHEM,
    );
    for policy in [WrapInside::Auto, WrapInside::Avoid] {
        let batch = compute::<S>(
            &f,
            &[policy],
            AvailableOf::definite(S::from_f64(40.0)),
            false,
        );
        assert_eq!(cuts(&batch), [4, 8]);
        assert_eq!(finalized(&batch)[0].measurements().fit().advance(), 20.0);
        assert_eq!(
            batch.placed_text()[0].layout().source().text(),
            &text::Text::from("AB  CDEF")
        );
    }
    let f = captured(
        "AB\u{200b}CDE",
        style,
        std::slice::from_ref(&(0..6)),
        &[],
        false,
        false,
        AHEM,
    );
    assert_eq!(
        cuts(&compute::<S>(
            &f,
            &[WrapInside::Avoid],
            AvailableOf::definite(S::from_f64(30.0)),
            false
        )),
        [3, 6]
    );
}

#[test]
fn collapsed_space_and_explicit_zwsp_keep_actual_source_seams() {
    collapsed_and_explicit::<f32>();
    collapsed_and_explicit::<f64>();
}

#[derive(Clone)]
struct Retained<S: LayoutScalar> {
    caches: HashMap<u32, CacheOf<S>>,
    outputs: HashMap<u32, NodeOutputOf<S>>,
    fragments: Vec<InlineFragmentOutputEntryOf<u32, S>>,
}

struct CachedTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    retained: Retained<S>,
    hits: Cell<usize>,
}

impl<S: LayoutScalar> CachedTree<S> {
    fn new(tree: PublicLayoutTreeOf<S>) -> Self {
        Self {
            tree,
            retained: Retained {
                caches: HashMap::new(),
                outputs: HashMap::new(),
                fragments: vec![],
            },
            hits: Cell::new(0),
        }
    }
}

impl<S: LayoutScalar> Traverse for CachedTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        S: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for CachedTree<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let result = self
            .retained
            .caches
            .get(&node)?
            .get_with_context(input, context);
        if result.is_some() {
            self.hits.set(self.hits.get() + 1);
        }
        result
    }
    fn unrounded_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.retained.outputs.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for CachedTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = Retained<S>;
    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Retained<S>, Self::Error> {
        let mut next = self.retained.clone();
        for entry in batch.cache_clear_entries() {
            next.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            next.caches
                .entry(entry.node())
                .or_default()
                .store_with_context(entry.input(), entry.context(), entry.output());
        }
        for entry in batch.unrounded_entries() {
            next.outputs.insert(entry.node(), entry.output());
        }
        next.fragments = batch.final_inline_fragments().to_vec();
        Ok(next)
    }
    fn commit_layout_batch(&mut self, next: Retained<S>) {
        self.retained = next;
    }
}

fn warm_policy<S: LayoutScalar>() {
    let f = captured(
        "ABCDEFGH",
        break_all(),
        std::slice::from_ref(&(2..6)),
        &[],
        false,
        false,
        AHEM,
    );
    let tree = text_tree::<S>(&f.binding, false)
        .children(100, [0, 90])
        .children(90, [])
        .style(
            100,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            90,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(1.0)),
                    PreferredSizeOf::px(S::from_f64(1.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let cold_tree = CachedTree::new(tree.clone());
    let mut warm_tree = CachedTree::new(tree);
    let run = |tree: &CachedTree<S>, policy| {
        let flow = LayoutFlowInput::try_new(
            tree,
            100,
            vec![f.binding.clone()],
            crate::WrappingInput {
                scopes: policies(&f, &[policy]),
                child_order: Vec::new(),
            },
        )
        .unwrap();
        compute_layout_with_flow(
            &flow,
            100,
            request_at(AvailableOf::definite(S::from_f64(40.0))),
        )
        .unwrap()
    };
    let old = run(&warm_tree, WrapInside::Auto);
    assert_eq!(cuts(&old), [4, 8]);
    old.apply_to(&mut warm_tree).unwrap();
    assert_eq!(warm_tree.retained.fragments, old.final_inline_fragments());
    let warm = run(&warm_tree, WrapInside::Avoid);
    let cold = run(&cold_tree, WrapInside::Avoid);
    assert_eq!(cuts(&warm), [2, 6, 8]);
    assert_eq!(cuts(&warm), cuts(&cold));
    assert_eq!(warm.final_entries(), cold.final_entries());
    assert!(
        warm_tree.hits.get() > 0,
        "independent sibling genuinely reused its retained cache"
    );
    assert_eq!(cold_tree.hits.get(), 0);
    // New scope selection does not mutate the already issued result's custody.
    assert_eq!(cuts(&old), [4, 8]);
    assert_ascii_custody(&old, &f, "ABCDEFGH");
    assert_ascii_custody(&warm, &f, "ABCDEFGH");
}

#[test]
fn changed_scope_policy_reselects_warm_lines_and_preserves_old_immutable_results() {
    warm_policy::<f32>();
    warm_policy::<f64>();
}

fn mixed_fixture(rtl: bool, captured_range: Range<usize>, native: Range<usize>) -> Captured {
    captured_with_profile(
        "ABCDEFGH",
        break_all(),
        std::slice::from_ref(&captured_range),
        &[
            (native.start, ParagraphControlKind::InlineBoundaryStart),
            (native.end, ParagraphControlKind::InlineBoundaryEnd),
        ],
        CaptureProfile {
            rtl,
            endpoints: false,
            topology: text::SourceProcessingTopology::Unknown,
        },
        AHEM,
    )
}

fn mixed_positions(f: &Captured) -> [text::CapturedEventPosition; 4] {
    assert_eq!(f.scopes.len(), 1);
    let scope = &f.scopes[0];
    assert_eq!(
        scope.domain().domain_topology(),
        text::SourceProcessingTopology::Unknown
    );
    assert!(scope.entry_control().is_none());
    assert!(scope.exit_control().is_none());
    assert_eq!(f.binding.controls.len(), 2);
    for control in &f.binding.controls {
        assert!(
            f.binding
                .prepared
                .control_is_boundary_exact(&control.occurrence)
                .unwrap()
        );
    }
    [
        f.binding.controls[0]
            .occurrence
            .captured_control()
            .unwrap()
            .event_position(),
        scope.enter_position(),
        f.binding.controls[1]
            .occurrence
            .captured_control()
            .unwrap()
            .event_position(),
        scope.exit_position(),
    ]
}

fn before(first: &text::CapturedEventPosition, second: &text::CapturedEventPosition) {
    assert_eq!(first.partial_cmp(second), Some(std::cmp::Ordering::Less));
}

fn assert_mixed_facts(f: &Captured, reverse: bool) {
    let [start, enter, end, exit] = mixed_positions(f);
    if reverse {
        before(&enter, &start);
        before(&start, &end);
        before(&end, &exit);
    } else {
        before(&start, &enter);
        before(&enter, &exit);
        before(&exit, &end);
    }
    let p = &f.binding.prepared;
    let cut = p
        .opportunity_views()
        .find(|offer| {
            offer.kind() == text::PreparedBreakKind::Allowed
                && offer.boundary().view().source_before().get() == 4
                && offer.boundary().view().source_after().get() == 4
        })
        .unwrap()
        .boundary();
    assert!(
        p.boundary_consumes_control(&cut, &f.binding.controls[0].occurrence)
            .unwrap()
    );
    assert!(
        !p.boundary_consumes_control(&cut, &f.binding.controls[1].occurrence)
            .unwrap()
    );
    assert!(
        p.boundary_inside_captured_scope(&cut, &f.scopes[0])
            .unwrap()
    );
    for at in [1, 7] {
        let cut = p
            .opportunity_views()
            .find(|offer| {
                offer.kind() == text::PreparedBreakKind::Allowed
                    && offer.boundary().view().source_before().get() == at
                    && offer.boundary().view().source_after().get() == at
            })
            .unwrap()
            .boundary();
        assert_eq!(
            p.boundary_inside_captured_scope(&cut, &f.scopes[0])
                .unwrap(),
            reverse
        );
        let expected_start_consumed = !reverse || at == 7;
        let expected_end_consumed = reverse && at == 7;
        assert_eq!(
            p.boundary_consumes_control(&cut, &f.binding.controls[0].occurrence)
                .unwrap(),
            expected_start_consumed
        );
        assert_eq!(
            p.boundary_consumes_control(&cut, &f.binding.controls[1].occurrence)
                .unwrap(),
            expected_end_consumed
        );
    }
}

fn assert_mixed_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    f: &Captured,
    expected: &[usize],
) {
    assert_eq!(cuts(batch), expected);
    assert_ascii_custody(batch, f, "ABCDEFGH");
    for (i, line) in batch.placed_text()[0].layout().lines().iter().enumerate() {
        assert_eq!(line.bounds().origin.y, i as f64 * 10.0);
        assert_eq!(line.bounds().size.height, 10.0);
        assert_eq!(line.baseline(), 8.0 + i as f64 * 10.0);
    }
    let expected_measures = if expected == [4, 8] {
        vec![40.0, 40.0]
    } else {
        vec![20.0, 40.0, 20.0]
    };
    assert_eq!(
        finalized(batch)
            .iter()
            .map(|line| line.measurements().fit().advance())
            .collect::<Vec<_>>(),
        expected_measures
    );
}

fn mixed_nested<S: LayoutScalar>(reverse: bool) -> Vec<String> {
    let mut failures = Vec::new();
    for rtl in [false, true] {
        let f = if reverse {
            mixed_fixture(rtl, 0..8, 2..6)
        } else {
            mixed_fixture(rtl, 2..6, 0..8)
        };
        assert_mixed_facts(&f, reverse);
        let tree = text_tree::<S>(&f.binding, rtl);
        let native = pair(10000, 10001, WrapInside::Avoid);
        let canonical = WrappingScopeInput::Captured {
            edges: Default::default(),
            scope: f.scopes[0].clone(),
            wrap_inside: WrapInside::Avoid,
        };
        let native_only = if reverse { vec![2, 6, 8] } else { vec![4, 8] };
        let captured_only = if reverse { vec![4, 8] } else { vec![2, 6, 8] };
        for (label, definitions, expected) in [
            ("neutral", vec![], vec![4, 8]),
            ("native-only", vec![native.clone()], native_only),
            ("captured-only", vec![canonical.clone()], captured_only),
        ] {
            let flow = LayoutFlowInput::try_new(
                &tree,
                0,
                vec![f.binding.clone()],
                crate::WrappingInput {
                    scopes: definitions,
                    child_order: Vec::new(),
                },
            )
            .unwrap();
            let batch = compute_layout_with_flow(
                &flow,
                0,
                request_at(AvailableOf::definite(S::from_f64(40.0))),
            )
            .unwrap();
            assert_mixed_geometry(&batch, &f, &expected);
            eprintln!(
                "mixed {} rtl={rtl} reverse={reverse} {label}: cuts {:?}",
                std::any::type_name::<S>(),
                cuts(&batch)
            );
        }
        for (label, definitions) in [
            ("native-first", vec![native.clone(), canonical.clone()]),
            ("captured-first", vec![canonical.clone(), native.clone()]),
        ] {
            let label = format!(
                "mixed {} rtl={rtl} reverse={reverse} {label}",
                std::any::type_name::<S>()
            );
            let flow = match LayoutFlowInput::try_new(
                &tree,
                0,
                vec![f.binding.clone()],
                crate::WrappingInput {
                    scopes: definitions,
                    child_order: Vec::new(),
                },
            ) {
                Ok(flow) => flow,
                Err(error) => {
                    failures.push(format!("{label}: constructor {error:?}"));
                    continue;
                }
            };
            match compute_layout_with_flow(
                &flow,
                0,
                request_at(AvailableOf::definite(S::from_f64(40.0))),
            ) {
                Ok(batch) => {
                    assert_mixed_geometry(&batch, &f, &[2, 6, 8]);
                    eprintln!("{label}: cuts {:?}", cuts(&batch));
                }
                Err(error) => failures.push(format!("{label}: layout {error:?}")),
            }
        }
    }
    failures
}

#[test]
fn unknown_topology_mixed_authorities_preserve_both_nested_orders() {
    let failures = [
        mixed_nested::<f32>(false),
        mixed_nested::<f64>(false),
        mixed_nested::<f32>(true),
        mixed_nested::<f64>(true),
    ]
    .concat();
    assert!(
        failures.is_empty(),
        "valid mixed nesting must select [2,6,8]:\n{}",
        failures.join("\n")
    );
}

fn mixed_crossing<S: LayoutScalar>() -> Vec<String> {
    let mut failures = Vec::new();
    for rtl in [false, true] {
        let f = mixed_fixture(rtl, 2..6, 0..4);
        let [start, enter, end, exit] = mixed_positions(&f);
        // These actual events prove crossing; equal text offsets or a source
        // hull are not the authority for a normalized inline box forest.
        before(&start, &enter);
        before(&enter, &end);
        before(&end, &exit);
        let tree = text_tree::<S>(&f.binding, rtl);
        let native = pair(10000, 10001, WrapInside::Avoid);
        let canonical = WrappingScopeInput::Captured {
            edges: Default::default(),
            scope: f.scopes[0].clone(),
            wrap_inside: WrapInside::Avoid,
        };
        for (label, definitions) in [
            ("native-first", vec![native.clone(), canonical.clone()]),
            ("captured-first", vec![canonical.clone(), native.clone()]),
        ] {
            let label = format!("crossing {} rtl={rtl} {label}", std::any::type_name::<S>());
            match LayoutFlowInput::try_new(
                &tree,
                0,
                vec![f.binding.clone()],
                crate::WrappingInput {
                    scopes: definitions,
                    child_order: Vec::new(),
                },
            ) {
                Ok(_) => failures.push(format!(
                    "{label}: constructor admitted crossing definitions"
                )),
                Err(error) => {
                    if !matches!(
                        error.kind(),
                        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                            WrappingScopeError::CrossingScopes
                        ))
                    ) {
                        failures.push(format!("{label}: wrong refusal {error:?}"));
                    }
                }
            }
        }
    }
    failures
}

#[test]
fn unknown_topology_mixed_crossing_refuses_at_scope_construction() {
    let failures = [mixed_crossing::<f32>(), mixed_crossing::<f64>()].concat();
    assert!(
        failures.is_empty(),
        "crossing normalized scopes must refuse:\n{}",
        failures.join("\n")
    );
}
