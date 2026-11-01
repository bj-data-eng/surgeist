//! Captured scope facts reach real Layout geometry. Ahem's retained tables and
//! CSS2 10.8.1 aligned subtrees supply the independent metric/edge expectations.
#[path = "scope_alignment/resource_comparisons.rs"]
mod resource_comparisons;
#[path = "scope_alignment/wrapping_continuation.rs"]
mod wrapping_continuation;

use super::*;
use std::sync::Arc;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn table(tag: [u8; 4]) -> &'static [u8] {
    let word = |at| u16::from_be_bytes(AHEM[at..at + 2].try_into().unwrap());
    let record = (0..usize::from(word(4)))
        .map(|i| 12 + 16 * i)
        .find(|&at| AHEM[at..at + 4] == tag)
        .unwrap();
    let dword = |at| u32::from_be_bytes(AHEM[at..at + 4].try_into().unwrap());
    let at = usize::try_from(dword(record + 8)).unwrap();
    &AHEM[at..at + usize::try_from(dword(record + 12)).unwrap()]
}

fn font_premises() {
    let word = |bytes: &[u8], at| u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap());
    assert_eq!(word(table(*b"head"), 18), 1000);
    assert_eq!(word(table(*b"hmtx"), 58 * 4), 1000); // X
    let signed = |at| i16::from_be_bytes(table(*b"hhea")[at..at + 2].try_into().unwrap());
    assert_eq!((signed(4), signed(6), signed(8)), (800, -200, 0));
    assert_eq!(word(table(*b"OS/2"), 62) & 0x80, 0);
    assert_eq!(word(table(*b"OS/2"), 16), 143); // distinct subscript font offset
    for tag in [*b"BASE", *b"GSUB", *b"GPOS"] {
        assert!(!(0..usize::from(word(AHEM, 4))).any(|i| AHEM[12 + 16 * i..16 + 16 * i] == tag));
    }
    // Inspect the actual Windows Unicode format4 mapping: X88 in the
    // 40..126 segment with delta-30 and no glyph-array indirection is gid58.
    let cmap = table(*b"cmap");
    let record = (0..usize::from(word(cmap, 2)))
        .map(|i| 4 + 8 * i)
        .find(|&at| (word(cmap, at), word(cmap, at + 2)) == (3, 1))
        .unwrap();
    let offset = usize::try_from(u32::from_be_bytes(
        cmap[record + 4..record + 8].try_into().unwrap(),
    ))
    .unwrap();
    let subtable = &cmap[offset..];
    assert_eq!(word(subtable, 0), 4);
    let segments = usize::from(word(subtable, 6)) / 2;
    let segment = (0..segments)
        .find(|&i| {
            word(subtable, 16 + 2 * segments + 2 * i) <= 88 && 88 <= word(subtable, 14 + 2 * i)
        })
        .unwrap();
    assert_eq!(word(subtable, 16 + 4 * segments + 2 * segment) as i16, -30);
    assert_eq!(word(subtable, 16 + 6 * segments + 2 * segment), 0);
}

fn reference() -> text::ScopeFontReference {
    text::ScopeFontReference {
        script: text::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: text::FontBaselineAxis::HorizontalY,
        baseline: text::FontBaseline::Alphabetic,
    }
}

fn authored(size: f32) -> text::Style {
    text::Style {
        white_space: text::WhiteSpace::Preserve,
        word_break: text::WordBreak::BreakAll,
        wrap: text::Wrap::Word,
        hyphens: text::Hyphens::None,
        ..ahem_style(size, f64::from(size))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Single,
    Nested,
    GlyphFreeParent,
    Dormant,
    Atomic,
    ForcedControl,
    OutOfFlow,
    NumericChild,
    NativePairChild,
}

struct Case {
    binding: TextParagraphBinding<u32>,
    scopes: Vec<text::CapturedScope>,
    shape: Shape,
    child: Option<TextParagraphBinding<u32>>,
}

fn prepare(analyzed: &text::AnalyzedSource, scale: f32) -> text::PreparedText {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system
        .prepare(
            analyzed,
            text::PreparationOptions::try_new(scale, text::TabPreparation::FontAdvance)
                .unwrap()
                .with_baseline_reference(text::FontBaseline::Alphabetic),
        )
        .unwrap()
}

impl Case {
    fn new(
        shape: Shape,
        outer: text::ScopeAlignmentBoundary,
        base_size: f32,
        inner: &str,
        scale: f32,
    ) -> Self {
        Self::new_with_limits(
            shape,
            outer,
            base_size,
            inner,
            scale,
            text::ScopeAlignmentLimits::new(10000, 100, 1_000_000, 16 << 20),
        )
    }

    fn new_with_limits(
        shape: Shape,
        outer: text::ScopeAlignmentBoundary,
        base_size: f32,
        inner: &str,
        scale: f32,
        limits: text::ScopeAlignmentLimits,
    ) -> Self {
        use text::SourceProcessingEvent as E;
        let base = authored(base_size);
        let small = authored(10.0);
        let nested = matches!(shape, Shape::Nested | Shape::GlyphFreeParent);
        let numeric_child = matches!(shape, Shape::NumericChild | Shape::NativePairChild);
        let outer_style = if shape == Shape::GlyphFreeParent {
            authored(20.0)
        } else {
            small.clone()
        };
        let mut runs = vec![(1_u32, "X", base.clone())];
        if shape != Shape::GlyphFreeParent {
            runs.push((2, "X", small.clone()));
        }
        if nested {
            runs.push((3, inner, small.clone()));
        }
        let raw = runs.iter().map(|(_, raw, _)| *raw).collect::<String>();
        let mut source = text::Source::identified(raw.as_str(), text::Id::from_u64(1578001), 7);
        let mut cursor = 0;
        let origins = runs
            .iter()
            .map(|(node, raw, style)| {
                let start = cursor;
                cursor += raw.encode_utf16().count();
                source.span(text::Range::new(start, cursor), style.clone());
                text::ParagraphOrigin {
                    text_node: None,
                    source: text::Source::identified(
                        *raw,
                        text::Id::from_u64(1578100 + u64::from(*node)),
                        7,
                    ),
                    local: text::Range::new(0, raw.encode_utf16().count()),
                    aggregate: text::Range::new(start, cursor),
                    kind: text::ParagraphOriginKind::Authored,
                }
            })
            .collect::<Vec<_>>();
        if matches!(shape, Shape::Atomic | Shape::OutOfFlow) {
            source.inline_box(text::InlineBox::new(
                text::Id::from_u64(90),
                if shape == Shape::Atomic {
                    text::InlineBoxKind::InFlow
                } else {
                    text::InlineBoxKind::OutOfFlow
                },
                cursor,
                text::Size::new(6.0, 10.0),
            ));
        }
        let mut controls = vec![
            text::ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: cursor,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ];
        let mut scopes = vec![
            text::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: Some(reference()),
                topology: text::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: base,
                used_direction: ParagraphBaseDirection::LeftToRight,
            },
            text::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: Some(outer),
                font_reference: Some(reference()),
                trim: Default::default(),
                style: Some(outer_style),
                boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
            },
        ];
        if nested {
            controls.extend([
                text::ParagraphControl {
                    at: if shape == Shape::GlyphFreeParent {
                        1
                    } else {
                        2
                    },
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: cursor,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ]);
            scopes.push(text::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: Some(text::ScopeAlignmentBoundary::NonInitial),
                font_reference: Some(reference()),
                trim: Default::default(),
                style: Some(small),
                boundaries: Some(text::SourceProcessingScopeBoundaries { start: 2, end: 3 }),
            });
        }
        if shape == Shape::ForcedControl {
            controls.push(text::ParagraphControl {
                at: cursor,
                kind: ParagraphControlKind::ForcedBreak,
            });
        }
        if numeric_child {
            controls.extend([
                text::ParagraphControl {
                    at: cursor,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: cursor,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ]);
            scopes.push(text::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                style: None,
                trim: Default::default(),
                boundaries: (shape == Shape::NumericChild)
                    .then_some(text::SourceProcessingScopeBoundaries { start: 2, end: 3 }),
            });
        }
        if shape == Shape::Dormant {
            // Real empty capture, no source/control/metric occurrence. Missing
            // dormant font context must not become an eager resolver demand.
            scopes.push(text::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: Some(text::ScopeAlignmentBoundary::NonInitial),
                font_reference: None,
                style: None,
                trim: Default::default(),
                boundaries: None,
            });
        }
        let mut events = vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: text::Affinity::After,
            },
            E::Connection(0),
            E::EnterScope(1),
            E::Control {
                control: 0,
                affinity: text::Affinity::After,
            },
        ];
        if shape != Shape::GlyphFreeParent {
            events.push(E::Origin {
                origin: 1,
                affinity: text::Affinity::After,
            });
        }
        if nested {
            if shape != Shape::GlyphFreeParent {
                events.push(E::Connection(1));
            }
            events.extend([
                E::EnterScope(2),
                E::Control {
                    control: 2,
                    affinity: text::Affinity::After,
                },
                E::Origin {
                    origin: origins.len() - 1,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 3,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(2),
            ]);
        }
        if matches!(shape, Shape::Atomic | Shape::OutOfFlow) {
            events.push(E::Atomic {
                atomic: 0,
                affinity: text::Affinity::After,
            });
        }
        if shape == Shape::ForcedControl {
            events.push(E::Control {
                control: 2,
                affinity: text::Affinity::After,
            });
        }
        if numeric_child {
            events.extend([
                E::EnterScope(2),
                E::Control {
                    control: 2,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 3,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(2),
            ]);
        }
        events.extend([
            E::Control {
                control: 1,
                affinity: text::Affinity::After,
            },
            E::ExitScope(1),
        ]);
        if shape == Shape::Dormant {
            events.extend([E::EnterScope(2), E::ExitScope(2)]);
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let nodes = events
            .iter()
            .filter_map(|event| match *event {
                E::Origin { origin, .. } => Some(runs[origin].0),
                E::Control { control, .. } => Some(20 + u32::try_from(control).unwrap()),
                E::Atomic { .. } => Some(90),
                _ => None,
            })
            .collect();
        let mut definitions = (0..origins.len() - 1)
            .map(text::ParagraphBoundaryDefinition::Connection)
            .collect::<Vec<_>>();
        if matches!(shape, Shape::Atomic | Shape::OutOfFlow) {
            definitions.extend([
                text::ParagraphBoundaryDefinition::AtomicBefore(0),
                text::ParagraphBoundaryDefinition::AtomicAfter(0),
            ]);
        }
        let input_control_count = controls.len();
        let processed = Arc::new(
            text::process_contextual_source(text::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: Some(limits),
                source,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                envelope: text::SourceProcessingEnvelope::Complete,
                connections: (0..origins.len() - 1)
                    .map(|i| text::SourceProcessingConnection {
                        before_origin: i,
                        after_origin: i + 1,
                        kind: text::ParagraphConnectionKind::Continuous,
                    })
                    .collect(),
                origins,
                controls,
                boundary_policies: definitions
                    .into_iter()
                    .map(|definition| text::ParagraphBoundaryPolicy {
                        definition,
                        preceding_wrap: text::Wrap::Word,
                        following_wrap: text::Wrap::Word,
                        ancestor_wrap: text::Wrap::Word,
                    })
                    .collect(),
                replacements: vec![],
                computed_line_boundaries: vec![],
                scopes,
                paragraphs: vec![text::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        let scopes: Vec<_> = (0..if nested || numeric_child || shape == Shape::Dormant {
            3
        } else {
            2
        })
            .map(|i| processed.captured_scope(i).unwrap())
            .collect();
        assert_eq!(scopes[1].parent(), Some(scopes[0].clone()));
        assert_eq!(scopes[1].domain(), scopes[0]);
        if nested {
            assert_eq!(scopes[2].parent(), Some(scopes[1].clone()));
        }
        let analyzed = text::AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let analyzed =
            crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
        let prepared = prepare(&analyzed, scale);
        let binding = TextParagraphBinding {
            origins: prepared
                .origin_views()
                .zip(&runs)
                .map(|(origin, run)| TextOriginBinding {
                    origin: origin.handle(),
                    node: run.0,
                })
                .collect(),
            controls: prepared
                .control_views()
                .map(|control| {
                    let occurrence = control.handle();
                    let captured = occurrence.captured_control().unwrap();
                    // Prepared views preserve event order, whereas host node
                    // ids name our original input-control inventory. Associate
                    // the actual issued capture handles, never view ordinals.
                    let original = (0..input_control_count)
                        .find(|&index| {
                            processed.captured_control(index).as_ref() == Some(&captured)
                        })
                        .expect("prepared control retains an original captured identity");
                    TextControlBinding {
                        occurrence,
                        node: 20 + u32::try_from(original).unwrap(),
                    }
                })
                .collect(),
            atomics: prepared
                .atomic_views()
                .map(|atomic| TextAtomicBinding {
                    occurrence: atomic.handle(),
                    node: 90,
                })
                .collect(),
            prepared,
            container: 0,
            nodes,
            line_strut: None,
        };
        let child = if shape == Shape::Atomic {
            // Actual independent child paragraph: size5 gives Alpha4. A fixed
            // height10 box therefore has real before4/after6, not fake metrics.
            let source = text::Source::new("X");
            let range = text::Range::new(0, 1);
            let child = text::AnalyzedSource::from_paragraph(
                text::ParagraphInput {
                    terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
                    source: source.clone(),
                    base_style: authored(5.0),
                    segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                    tab_reference: None,
                    origins: vec![text::ParagraphOrigin {
                        source,
                        local: range,
                        aggregate: range,
                        kind: text::ParagraphOriginKind::Authored,
                        text_node: None,
                    }],
                    connections: vec![],
                    controls: vec![],
                    boundary_policies: vec![],
                    replacements: vec![],
                    anchor_order: vec![],
                },
                text::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let prepared = prepare(&child, 1.0);
            let origin = prepared.origin_views().next().unwrap().handle();
            Some(TextParagraphBinding {
                origins: vec![TextOriginBinding { origin, node: 91 }],
                prepared,
                container: 90,
                nodes: vec![91],
                controls: vec![],
                atomics: vec![],
                line_strut: None,
            })
        } else {
            None
        };
        Self {
            binding,
            scopes,
            shape,
            child,
        }
    }

    fn row<S: LayoutScalar>(
        &self,
        scope: usize,
        alignment: VerticalAlignOf<S>,
    ) -> TextScopeAlignmentInputOf<S> {
        TextScopeAlignmentInputOf {
            scope: self.scopes[scope].clone(),
            alignment,
        }
    }

    fn tree<S: LayoutScalar>(&self, width: f64, edge: VerticalAlignOf<S>) -> PublicLayoutTreeOf<S> {
        let mut tree = PublicLayoutTreeOf::new()
            .children(0, self.binding.nodes.iter().copied())
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    inline_baseline_context: InlineBaselineContext::Named(BaselineName::Alphabetic),
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(width)),
                        PreferredSizeOf::AUTO,
                    ),
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
            let forced = control.occurrence.view().kind() == &ParagraphControlKind::ForcedBreak;
            let numeric = matches!(self.shape, Shape::NumericChild | Shape::NativePairChild)
                && control.node >= 22;
            let metrics = InlineMetricsOf::try_new(
                InlineBaselineContext::Named(BaselineName::Alphabetic),
                S::from_f64(if forced {
                    4.0
                } else if numeric {
                    6.0
                } else {
                    0.0
                }),
                S::from_f64(if forced {
                    10.0
                } else if numeric {
                    14.0
                } else {
                    0.0
                }),
            )
            .unwrap();
            let input = if forced {
                LayoutInputOf::line_break(
                    LineBreakInputOf::new()
                        .with_metrics(metrics)
                        .with_vertical_align(edge),
                )
            } else {
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        if matches!(
                            control.occurrence.view().kind(),
                            ParagraphControlKind::InlineBoundaryStart
                        ) {
                            InlineBoundaryKind::Start
                        } else {
                            InlineBoundaryKind::End
                        },
                        metrics,
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap(),
                )
            };
            tree = tree.children(control.node, []).input(control.node, input);
        }
        if self.shape == Shape::Atomic {
            tree = tree
                .children(90, [91])
                .style(
                    90,
                    NodeInputOf {
                        display: Display::InlineBlock,
                        inline_baseline_context: InlineBaselineContext::Named(
                            BaselineName::Alphabetic,
                        ),
                        baseline_source: BaselineSource::First,
                        vertical_align: edge,
                        size: Size::new(
                            PreferredSizeOf::px(S::from_f64(6.0)),
                            PreferredSizeOf::px(S::from_f64(10.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .children(91, [])
                .input(
                    91,
                    LayoutInputOf::inline_text(TextParticipation::new(
                        self.child.as_ref().unwrap().origins[0].origin.clone(),
                    )),
                );
        } else if self.shape == Shape::OutOfFlow {
            tree = tree.children(90, []).style(
                90,
                NodeInputOf {
                    display: Display::Block,
                    position: Position::Absolute,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(6.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        }
        tree
    }

    fn bindings(&self) -> Vec<TextParagraphBinding<u32>> {
        let mut bindings = vec![self.binding.clone()];
        bindings.extend(self.child.clone());
        bindings
    }
}

fn shifted<S: LayoutScalar>(amount: f64) -> VerticalAlignOf<S> {
    VerticalAlignOf::Shift(BaselineOffsetOf::try_new(S::from_f64(amount)).unwrap())
}

fn solve<S: LayoutScalar>(
    case: &Case,
    tree: &PublicLayoutTreeOf<S>,
    rows: Vec<TextScopeAlignmentInputOf<S>>,
    width: f64,
) -> Result<CompletedLayoutBatchOf<u32, S>, LayoutErrorOf<u32, S>> {
    let input = LayoutFlowInput::try_new(tree, 0, case.bindings(), WrappingInput::default(), rows)?;
    compute_layout_with_flow(&input, 0, request(width))
}

fn lines<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let prepared = batch
        .placed_text()
        .iter()
        .find(|entry| entry.container() == 0)
        .unwrap()
        .layout()
        .analyzed_source()
        .identity();
    let mut lines = Vec::new();
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        if line.prepared_text().analyzed_source().identity() != prepared {
            continue;
        }
        if lines.len() <= fragment.line_index() {
            lines.resize_with(fragment.line_index() + 1, || None);
        }
        lines[fragment.line_index()] = Some(line);
    }
    lines.into_iter().map(Option::unwrap).collect()
}

fn placed<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> &text::Layout {
    batch
        .placed_text()
        .iter()
        .find(|entry| entry.container() == 0)
        .unwrap()
        .layout()
}

fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

fn error_is<S: LayoutScalar>(error: &LayoutErrorOf<u32, S>, expected: text::ScopeAlignmentError) {
    assert!(
        matches!(error.kind(), LayoutErrorKindOf::Text(error) if matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment { reason }) if *reason == expected)),
        "{error:?}"
    );
}

fn raw(layout: &text::Layout) {
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.face().index(), 0);
        for glyph in run.glyphs() {
            assert_eq!(glyph.id(), 58);
            assert_eq!(glyph.advance(), run.font_size());
            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
        }
    }
}

fn glyph_references(layout: &text::Layout) -> Vec<(text::Range, f64)> {
    // X has zero native offsets and Alpha=font zero in this actual Ahem.
    // Observe individual source glyphs, allowing valid same-frame run coalescing.
    layout
        .glyph_runs()
        .iter()
        .flat_map(|run| {
            run.glyphs()
                .iter()
                .map(|glyph| (glyph.range(), run.origin().y + f64::from(glyph.y())))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn source_custody(layout: &text::Layout) {
    let endpoint = |index, affinity| {
        layout
            .positions()
            .find(|position| position.source_cursor() == text::Cursor::new(index, affinity))
            .unwrap()
    };
    let selection = layout
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(layout.source().text().len(), text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(selection.source_text(), layout.source().text());
    assert_eq!(selection.plaintext(), layout.source().text());
}

fn own(line: &text::FinalizedLine, scope: &text::CapturedScope) -> (f64, f64) {
    let row = line
        .scope_allocations()
        .iter()
        .find(|row| &row.scope == scope)
        .unwrap();
    (row.displacement.get(), row.cumulative_displacement.get())
}

#[test]
fn normalized_alignment_admission_rejects_missing_foreign_duplicate_and_class_mismatch() {
    let case = Case::new(
        Shape::Single,
        text::ScopeAlignmentBoundary::NonInitial,
        10.0,
        "",
        1.0,
    );
    let tree = case.tree::<f64>(100.0, VerticalAlignOf::Baseline);
    let valid = case.row(1, shifted(3.0));
    let admitted = LayoutFlowInput::try_new(
        &tree,
        0,
        case.bindings(),
        WrappingInput::default(),
        vec![valid.clone()],
    )
    .unwrap();
    let prior = compute_layout_with_flow(&admitted, 0, request(100.0)).unwrap();
    let prior_entries = prior.final_entries().to_vec();
    let prior_line = lines(&prior).remove(0);
    let prior_feedback = prior_line.feedback();
    let prior_allocations = prior_line.scope_allocations().to_vec();
    let prior_glyphs = glyph_references(placed(&prior));
    // Supplied rows are validated at admission. Completeness belongs to the
    // actual selected occurrence demand, before any completed batch is issued.
    let missing =
        LayoutFlowInput::try_new(&tree, 0, case.bindings(), WrappingInput::default(), vec![])
            .unwrap();
    let error = match compute_layout_with_flow(&missing, 0, request(100.0)) {
        Ok(_) => panic!("missing demanded scope fact published a completed batch"),
        Err(error) => error,
    };
    assert!(matches!(error.kind(), LayoutErrorKindOf::Text(error)
        if error.code == text::ErrorCode::ScopeAlignmentRefused));
    error_is(
        &error,
        text::ScopeAlignmentError::MissingAllocation { scope: 1 },
    );
    assert_eq!(prior.final_entries(), prior_entries.as_slice());
    assert_eq!(prior_line.feedback(), prior_feedback);
    assert_eq!(prior_line.scope_allocations(), prior_allocations.as_slice());
    assert_eq!(glyph_references(placed(&prior)), prior_glyphs);
    assert_eq!(
        prior_line
            .with_spacing(text::SpacingAllocations::None)
            .unwrap()
            .feedback(),
        prior_feedback,
    );
    for (rows, expected) in [
        (
            vec![valid.clone(), valid],
            text::ScopeAlignmentError::DuplicateAllocation { scope: 1 },
        ),
        (
            vec![case.row(1, VerticalAlignOf::Baseline)],
            text::ScopeAlignmentError::BoundaryMismatch { scope: 1 },
        ),
    ] {
        let error = match LayoutFlowInput::try_new(
            &tree,
            0,
            case.bindings(),
            WrappingInput::default(),
            rows,
        ) {
            Ok(_) => panic!("invalid scope facts admitted"),
            Err(error) => error,
        };
        error_is(&error, expected);
    }
    let foreign = Case::new(
        Shape::Single,
        text::ScopeAlignmentBoundary::NonInitial,
        10.0,
        "",
        1.0,
    );
    let error = match LayoutFlowInput::try_new(
        &tree,
        0,
        case.bindings(),
        WrappingInput::default(),
        vec![foreign.row(1, shifted(3.0))],
    ) {
        Ok(_) => panic!("foreign capture admitted"),
        Err(error) => error,
    };
    error_is(&error, text::ScopeAlignmentError::ForeignCapture);
    let initial = Case::new(
        Shape::Single,
        text::ScopeAlignmentBoundary::Initial,
        10.0,
        "",
        1.0,
    );
    let tree = initial.tree::<f64>(100.0, VerticalAlignOf::Baseline);
    let error = match LayoutFlowInput::try_new(
        &tree,
        0,
        initial.bindings(),
        WrappingInput::default(),
        vec![initial.row(1, shifted(0.0))],
    ) {
        Ok(_) => panic!("explicit zero was treated as initial formatting"),
        Err(error) => error,
    };
    error_is(
        &error,
        text::ScopeAlignmentError::BoundaryMismatch { scope: 1 },
    );
}

#[test]
fn dormant_classified_scope_without_occurrences_needs_no_row_or_font_context() {
    let case = Case::new(
        Shape::Dormant,
        text::ScopeAlignmentBoundary::Initial,
        10.0,
        "",
        1.0,
    );
    assert!(case.scopes[2].style().is_none());
    assert!(case.scopes[2].font_reference().is_none());
    assert!(matches!(
        case.binding
            .prepared
            .scope_metrics(&case.scopes[2])
            .unwrap_err()
            .detail(),
        Some(text::ErrorDetail::ScopeAlignment {
            reason: text::ScopeAlignmentError::MissingMetrics { scope: 2 }
        })
    ));
    let tree = case.tree::<f64>(100.0, VerticalAlignOf::Baseline);
    let batch = solve(
        &case,
        &tree,
        vec![case.row(1, VerticalAlignOf::Baseline)],
        100.0,
    )
    .unwrap();
    assert_eq!(
        (
            lines(&batch)[0].feedback().baseline,
            lines(&batch)[0].feedback().line_height
        ),
        (8.0, 10.0)
    );
    raw(placed(&batch));
}

fn signed<S: LayoutScalar>() {
    font_premises();
    for delta in [-4.0, 0.0, 3.0] {
        let case = Case::new(
            Shape::Single,
            text::ScopeAlignmentBoundary::NonInitial,
            10.0,
            "",
            1.0,
        );
        let tree = case.tree::<S>(100.0, VerticalAlignOf::Baseline);
        let batch = solve(&case, &tree, vec![case.row(1, shifted(delta))], 100.0).unwrap();
        let line = lines(&batch).remove(0);
        let before = 8.0_f64.max(8.0 + delta);
        let after = 2.0_f64.max(2.0 - delta);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (before, before + after)
        );
        assert_eq!(own(&line, &case.scopes[1]), (delta, delta));
        let positions = glyph_references(placed(&batch));
        assert_eq!(
            positions,
            [
                (text::Range::new(0, 1), before),
                (text::Range::new(1, 2), before - delta)
            ]
        );
        let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
        assert_eq!(replay.scope_allocations(), line.scope_allocations());
        assert_eq!(replay.feedback(), line.feedback());
        let warm = solve(&case, &tree, vec![case.row(1, shifted(delta))], 100.0).unwrap();
        assert_eq!(batch.final_entries(), warm.final_entries());
        assert_eq!(placed(&batch).source().text().code_units(), &[88, 88]);
        raw(placed(&batch));
        source_custody(placed(&batch));
    }
}

#[test]
fn signed_and_explicit_zero_scope_offsets_move_true_glyph_references_and_replay_in_both_scalar_lanes()
 {
    signed::<f32>();
    signed::<f64>();
}

#[test]
fn glyph_free_parent_size_and_own_line_height_supply_actual_normalization_context() {
    font_premises();
    let case = Case::new(
        Shape::GlyphFreeParent,
        text::ScopeAlignmentBoundary::Initial,
        10.0,
        "X",
        1.0,
    );
    let parent = case
        .binding
        .prepared
        .scope_metrics(&case.scopes[1])
        .unwrap();
    assert_eq!(parent.font_size(), 20.0);
    assert_eq!(parent.effective_instance().unwrap().face().bytes(), AHEM);
    assert_eq!((parent.over(), parent.under()), (16.0, 4.0));
    let input =
        TextScopeAlignmentInputOf::<f64>::subscript(&case.binding.prepared, case.scopes[2].clone())
            .unwrap();
    assert_eq!(input.alignment, shifted(-4.0)); // parent20/5, not child10/5 or OS/2 2.86.
    let tree = case.tree::<f64>(100.0, VerticalAlignOf::Baseline);
    let batch = solve(
        &case,
        &tree,
        vec![case.row(1, VerticalAlignOf::Baseline), input],
        100.0,
    )
    .unwrap();
    let line = lines(&batch).remove(0);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (8.0, 14.0)
    );
    assert_eq!(own(&line, &case.scopes[2]), (-4.0, -4.0));
    assert!(
        placed(&batch)
            .run_views()
            .all(|run| run.font_size() == 10.0)
    );
    assert_eq!(
        glyph_references(placed(&batch)),
        [
            (text::Range::new(0, 1), 8.0),
            (text::Range::new(1, 2), 12.0)
        ]
    );
    raw(placed(&batch));
    let percentage = TextScopeAlignmentInputOf::<f64>::from_length_percentage(
        &case.binding.prepared,
        case.scopes[2].clone(),
        LengthPercentageOf::from_percent_fraction(0.5).unwrap(),
    )
    .unwrap();
    assert_eq!(percentage.alignment, shifted(5.0)); // own line-height10, not parent20 or expanded14.
    let percent = solve(
        &case,
        &tree,
        vec![case.row(1, VerticalAlignOf::Baseline), percentage],
        100.0,
    )
    .unwrap();
    let line = lines(&percent).remove(0);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (13.0, 15.0)
    );
    assert_eq!(own(&line, &case.scopes[2]), (5.0, 5.0));
}

#[test]
fn nested_fragments_recompute_selected_scope_frames_from_real_legal_cuts() {
    let case = Case::new(
        Shape::Nested,
        text::ScopeAlignmentBoundary::NonInitial,
        10.0,
        "XX",
        1.0,
    );
    let tree = case.tree::<f64>(20.0, VerticalAlignOf::Baseline);
    let batch = solve(
        &case,
        &tree,
        vec![case.row(1, shifted(3.0)), case.row(2, shifted(2.0))],
        20.0,
    )
    .unwrap();
    let lines = lines(&batch);
    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines
            .iter()
            .map(|line| (line.feedback().baseline, line.feedback().line_height))
            .collect::<Vec<_>>(),
        [(11.0, 13.0), (13.0, 15.0)]
    );
    assert_eq!(
        placed(&batch)
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [text::Range::new(0, 2), text::Range::new(2, 4)]
    );
    assert_eq!(own(&lines[1], &case.scopes[1]), (3.0, 3.0));
    assert_eq!(own(&lines[1], &case.scopes[2]), (2.0, 5.0));
    let outer = lines[0]
        .selected_line()
        .scopes()
        .find(|scope| scope.captured_scope() == case.scopes[1])
        .unwrap();
    let continued = lines[1]
        .selected_line()
        .scopes()
        .find(|scope| scope.captured_scope() == case.scopes[1])
        .unwrap();
    assert_ne!(outer, continued);
    raw(placed(&batch));
}

#[test]
fn f64_tiny_offset_survives_native_frames_and_scale_does_not_multiply_allocated_shift() {
    let delta = 2.0_f64.powi(-26);
    assert_eq!(8.0_f32 + delta as f32, 8.0_f32);
    for (scale, before, height) in [
        (1.0, 8.0 + delta, 10.0 + delta),
        (2.0, 16.0 + delta, 20.0 + delta),
    ] {
        let case = Case::new(
            Shape::Single,
            text::ScopeAlignmentBoundary::NonInitial,
            10.0,
            "",
            scale,
        );
        let tree = case.tree::<f64>(100.0, VerticalAlignOf::Baseline);
        let batch = solve(&case, &tree, vec![case.row(1, shifted(delta))], 100.0).unwrap();
        let line = lines(&batch).remove(0);
        assert_eq!(own(&line, &case.scopes[1]), (delta, delta));
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (before, height)
        );
        assert_eq!(
            case.binding
                .prepared
                .scope_metrics(&case.scopes[1])
                .unwrap()
                .font_size(),
            10.0 * scale
        );
        let references = glyph_references(placed(&batch));
        assert_eq!(references[0].1 - references[1].1, delta);
        raw(placed(&batch));
    }
}

fn edges<S: LayoutScalar>() {
    font_premises();
    for (alignment, inner_reference, inner_total, own_delta) in [
        (VerticalAlignOf::Top, 8.0, 8.0, 5.0),
        (VerticalAlignOf::Bottom, 18.0, -2.0, -5.0),
    ] {
        let case = Case::new(
            Shape::Nested,
            text::ScopeAlignmentBoundary::NonInitial,
            20.0,
            "X",
            1.0,
        );
        let tree = case.tree::<S>(100.0, VerticalAlignOf::Baseline);
        let batch = solve(
            &case,
            &tree,
            vec![case.row(1, shifted(3.0)), case.row(2, alignment)],
            100.0,
        )
        .unwrap();
        let line = lines(&batch).remove(0);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (16.0, 20.0)
        );
        assert_eq!(own(&line, &case.scopes[1]), (3.0, 3.0));
        assert_eq!(own(&line, &case.scopes[2]), (own_delta, inner_total));
        assert_eq!(
            glyph_references(placed(&batch)),
            [
                (text::Range::new(0, 1), 16.0),
                (text::Range::new(1, 2), 13.0),
                (text::Range::new(2, 3), inner_reference)
            ]
        );
        raw(placed(&batch));
    }
    for (alignment, control_baseline, top) in [
        (VerticalAlignOf::Top, 4.0, 0.0),
        (VerticalAlignOf::Bottom, 14.0, 10.0),
    ] {
        for shape in [Shape::Atomic, Shape::ForcedControl] {
            let case = Case::new(
                shape,
                text::ScopeAlignmentBoundary::NonInitial,
                20.0,
                "",
                1.0,
            );
            let tree = case.tree::<S>(100.0, alignment);
            let batch = solve(&case, &tree, vec![case.row(1, shifted(3.0))], 100.0).unwrap();
            let line = lines(&batch).remove(0);
            assert_eq!(
                (line.feedback().baseline, line.feedback().line_height),
                (16.0, 20.0)
            );
            assert_eq!(own(&line, &case.scopes[1]), (3.0, 3.0));
            if shape == Shape::Atomic {
                let slot = line.atomic_slots().next().unwrap();
                // Top/Bottom does not demand the child's named alignment
                // baseline. This native physical flow reference is its actual
                // line-under margin edge; the local Alpha4 remains separate.
                assert_eq!(slot.origin.y, top + 10.0);
                assert_eq!(slot.scope_displacement.get(), 0.0);
                let text::AtomicLineAllocation::InFlow {
                    flow,
                    box_geometry,
                    alignment: actual_alignment,
                    ..
                } = slot.allocation
                else {
                    panic!("actual in-flow atomic")
                };
                assert_eq!((flow.before_baseline, flow.after_baseline), (10.0, 0.0));
                assert_eq!(box_geometry.baseline - box_geometry.border.origin.y, 10.0);
                assert_eq!(
                    actual_alignment,
                    match alignment {
                        VerticalAlignOf::Top => text::LineBoxAlignment::Top,
                        VerticalAlignOf::Bottom => text::LineBoxAlignment::Bottom,
                        _ => unreachable!(),
                    }
                );
                let actual = output(&batch, 90);
                assert_eq!(actual.location.y.to_f64(), top);
                assert_eq!(actual.size, Size::new(S::from_f64(6.0), S::from_f64(10.0)));
                assert_eq!(
                    actual.named_baselines.get(BaselineName::Alphabetic).first(),
                    BaselineFactOf::Available(
                        BaselineCoordinateOf::try_new(PhysicalAxis::Vertical, S::from_f64(4.0))
                            .unwrap()
                    )
                );
            } else {
                let occurrence = case
                    .binding
                    .controls
                    .iter()
                    .find(|row| row.node == 22)
                    .unwrap();
                let slot = line
                    .control_slots()
                    .find(|slot| slot.control() == occurrence.occurrence)
                    .unwrap();
                assert_eq!(slot.slot().y, control_baseline);
            }
            assert_eq!(
                glyph_references(placed(&batch))
                    .into_iter()
                    .find(|(range, _)| *range == text::Range::new(1, 2))
                    .unwrap()
                    .1,
                13.0
            );
        }
    }
}

#[test]
fn nested_scope_and_existing_atomic_control_top_bottom_escape_ancestor_shift_in_same_line() {
    edges::<f32>();
    edges::<f64>();
}

#[test]
fn scoped_oof_static_anchor_moves_once_without_enlarging_the_line_or_absolute_box() {
    let case = Case::new(
        Shape::OutOfFlow,
        text::ScopeAlignmentBoundary::NonInitial,
        20.0,
        "",
        1.0,
    );
    let tree = case.tree::<f64>(100.0, VerticalAlignOf::Baseline);
    let rows = || vec![case.row(1, shifted(3.0))];
    let batch = solve(&case, &tree, rows(), 100.0).unwrap();
    let line = lines(&batch).remove(0);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (16.0, 20.0)
    );
    let absolute = output(&batch, 90);
    assert_eq!(absolute.location.y, -3.0);
    assert_eq!(absolute.size, Size::new(6.0, 10.0));
    assert_eq!(
        glyph_references(placed(&batch))
            .into_iter()
            .find(|(range, _)| *range == text::Range::new(1, 2))
            .unwrap()
            .1,
        13.0
    );
    let replay = solve(&case, &tree, rows(), 100.0).unwrap();
    assert_eq!(batch.final_entries(), replay.final_entries());
}

#[test]
fn accepted_scoped_envelope_and_float_band_are_replayed_together() {
    let case = Case::new(
        Shape::Single,
        text::ScopeAlignmentBoundary::NonInitial,
        10.0,
        "",
        1.0,
    );
    let mut children = case.binding.nodes.clone();
    children.insert(0, 95);
    let tree = case
        .tree::<f64>(30.0, VerticalAlignOf::Baseline)
        .children(0, children)
        .children(95, [])
        .style(
            95,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSizeOf::px(10.0), PreferredSizeOf::px(10.0)),
                ..NodeInputOf::default()
            },
        );
    let rows = || vec![case.row(1, shifted(3.0))];
    let batch = solve(&case, &tree, rows(), 30.0).unwrap();
    let line = lines(&batch).remove(0);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (11.0, 13.0)
    );
    assert_eq!(output(&batch, 0).size, Size::new(30.0, 13.0));
    assert_eq!(output(&batch, 95).size, Size::new(10.0, 10.0));
    assert_eq!(
        placed(&batch).lines()[0].bounds(),
        text::Rect::new(10.0, 0.0, 20.0, 13.0)
    );
    assert_eq!(
        glyph_references(placed(&batch)),
        [
            (text::Range::new(0, 1), 11.0),
            (text::Range::new(1, 2), 8.0)
        ]
    );
    let replay = solve(&case, &tree, rows(), 30.0).unwrap();
    assert_eq!(batch.final_entries(), replay.final_entries());
    source_custody(placed(&batch));
}

#[test]
fn wrapping_numeric_child_keeps_source_owner_and_inherits_only_the_active_parent_frame() {
    for shape in [Shape::NumericChild, Shape::NativePairChild] {
        let case = Case::new(
            shape,
            text::ScopeAlignmentBoundary::NonInitial,
            10.0,
            "",
            1.0,
        );
        assert_eq!(case.scopes[2].parent(), Some(case.scopes[1].clone()));
        assert_eq!(case.scopes[2].alignment_boundary(), None);
        assert!(case.scopes[2].font_reference().is_none());
        assert!(matches!(
            case.binding
                .prepared
                .scope_metrics(&case.scopes[2])
                .unwrap_err()
                .detail(),
            Some(text::ErrorDetail::ScopeAlignment {
                reason: text::ScopeAlignmentError::MissingMetrics { scope: 2 }
            })
        ));
        for node in [22, 23] {
            let control = case
                .binding
                .controls
                .iter()
                .find(|row| row.node == node)
                .unwrap();
            assert_eq!(
                control.occurrence.captured_control().unwrap().scope(),
                case.scopes[2]
            );
        }
        if shape == Shape::NativePairChild {
            assert!(case.scopes[2].entry_control().is_none());
            assert!(case.scopes[2].exit_control().is_none());
        }
        let tree = case.tree::<f64>(100.0, VerticalAlignOf::Baseline);
        let wrapping = |wrong_end: bool| {
            let contribution = Some(InlineBoxMetricContribution::Strut(text::LineBoxStrut {
                scope: None, // Source attribution is issued by actual association.
                metrics: text::LineStrut {
                    baseline: 6.0,
                    line_extent: 14.0,
                },
                alignment: text::LineBoxAlignment::Baseline,
            }));
            let mut scopes = vec![if shape == Shape::NumericChild && !wrong_end {
                WrappingScopeInput::Captured {
                    scope: case.scopes[2].clone(),
                    wrap_inside: WrapInside::Auto,
                    edges: WrapEdges::default(),
                    metric_contribution: contribution,
                }
            } else {
                WrappingScopeInput::NativePair {
                    start: 22,
                    end: if wrong_end { 21 } else { 23 },
                    wrap_inside: WrapInside::Auto,
                    edges: WrapEdges::default(),
                    metric_contribution: contribution,
                }
            }];
            if shape == Shape::NativePairChild {
                // This captured outer box is distinct from the enclosed
                // native pair. Its own absence is an explicit producer fact.
                scopes.push(WrappingScopeInput::Captured {
                    scope: case.scopes[2].clone(),
                    wrap_inside: WrapInside::Auto,
                    edges: WrapEdges::default(),
                    metric_contribution: Some(InlineBoxMetricContribution::NoContribution),
                });
            }
            WrappingInput {
                scopes,
                child_order: vec![],
            }
        };
        let rows = || vec![case.row(1, shifted(3.0))];
        let input =
            LayoutFlowInput::try_new(&tree, 0, case.bindings(), wrapping(false), rows()).unwrap();
        let batch = compute_layout_with_flow(&input, 0, request(100.0)).unwrap();
        let line = lines(&batch).remove(0);
        // Child's actual6/8 bounds become9/5 under parent+3. The parent's
        // Ahem10 glyph becomes11/-1; actual root8/2 gives union11/5 =16.
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (11.0, 16.0)
        );
        assert_eq!(own(&line, &case.scopes[1]), (3.0, 3.0));
        assert!(
            line.selected_line()
                .scopes()
                .all(|scope| scope.captured_scope() != case.scopes[2])
        );
        let request = line.selected_line().request().clone();
        // The same-box endpoint allocations already own these metrics. The
        // matching wrapping declaration validates them without a duplicate row.
        assert!(request.line_box_struts.is_empty());
        for node in [22, 23] {
            let occurrence = &case
                .binding
                .controls
                .iter()
                .find(|row| row.node == node)
                .unwrap()
                .occurrence;
            assert_eq!(
                occurrence.captured_control().unwrap().scope(),
                case.scopes[2]
            );
            let allocation = request
                .controls
                .iter()
                .find(|row| &row.occurrence == occurrence)
                .unwrap();
            assert_eq!(
                allocation.strut,
                Some(text::ControlStrut {
                    baseline: 6.0,
                    line_extent: 14.0,
                    alignment: text::LineBoxAlignment::Baseline,
                })
            );
            let slot = line
                .control_slots()
                .find(|slot| &slot.control() == occurrence)
                .unwrap();
            assert_eq!(slot.slot().y, 8.0);
            assert_eq!(slot.scope_displacement().get(), 3.0);
        }
        assert_eq!(
            glyph_references(placed(&batch)),
            [
                (text::Range::new(0, 1), 11.0),
                (text::Range::new(1, 2), 8.0)
            ]
        );
        let error =
            match LayoutFlowInput::try_new(&tree, 0, case.bindings(), wrapping(true), rows()) {
                Ok(_) => panic!("native endpoints from different actual boxes admitted"),
                Err(error) => error,
            };
        assert!(
            matches!(
                error.kind(),
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                    WrappingScopeError::CaptureAssociation
                ))
            ),
            "{error:?}"
        );
        if shape == Shape::NativePairChild {
            let prior_entries = batch.final_entries().to_vec();
            let prior_feedback = line.feedback();
            let prior_frames = line.scope_allocations().to_vec();
            let prior_glyphs = glyph_references(placed(&batch));
            let controls = || {
                line.control_slots()
                    .map(|slot| (slot.control(), slot.slot(), slot.scope_displacement()))
                    .collect::<Vec<_>>()
            };
            let prior_controls = controls();
            let foreign = Case::new(
                shape,
                text::ScopeAlignmentBoundary::NonInitial,
                10.0,
                "",
                1.0,
            );
            for (assertion, expected) in [
                (
                    case.scopes[1].clone(),
                    text::ScopeAlignmentError::BoundaryMismatch { scope: 2 },
                ),
                (
                    foreign.scopes[2].clone(),
                    text::ScopeAlignmentError::ForeignCapture,
                ),
            ] {
                // A supplied owner is a checked assertion, never an authority
                // that can override the real captured Start/End control proof.
                let mut asserted = wrapping(false);
                match &mut asserted.scopes[0] {
                    WrappingScopeInput::NativePair {
                        metric_contribution: Some(InlineBoxMetricContribution::Strut(strut)),
                        ..
                    } => strut.scope = Some(assertion),
                    _ => unreachable!(),
                }
                // This raw optional assertion is checked against the actual
                // consumed box proof when constructing the Text request.
                let admitted =
                    LayoutFlowInput::try_new(&tree, 0, case.bindings(), asserted, rows()).unwrap();
                let error = match compute_layout_with_flow(&admitted, 0, super::request(100.0)) {
                    Ok(_) => panic!("unproven source owner published a completed batch"),
                    Err(error) => error,
                };
                assert!(matches!(error.kind(), LayoutErrorKindOf::Text(error)
                    if error.code == text::ErrorCode::ScopeAlignmentRefused));
                error_is(&error, expected);
                assert_eq!(batch.final_entries(), prior_entries.as_slice());
                assert_eq!(line.feedback(), prior_feedback);
                assert_eq!(line.scope_allocations(), prior_frames.as_slice());
                assert_eq!(line.selected_line().request(), &request);
                assert_eq!(controls(), prior_controls);
                assert_eq!(glyph_references(placed(&batch)), prior_glyphs);
                let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
                assert_eq!(replay.feedback(), prior_feedback);
                assert_eq!(replay.scope_allocations(), prior_frames.as_slice());
            }
        }
        assert_eq!(
            line.with_spacing(text::SpacingAllocations::None)
                .unwrap()
                .feedback(),
            line.feedback()
        );
    }
}
