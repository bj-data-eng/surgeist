//! Public Mixed consumers: one Vertical frame and independently qualified axes.
use super::*;

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_001,
        "{actual} != {expected}"
    );
}

fn central() -> l::InlineBaselineContext {
    l::InlineBaselineContext::Named(l::BaselineName::Central)
}

fn mixed_font() -> Vec<u8> {
    // The existing builder preserves the licensed asymmetric chi outline
    // [0,400,1000,600]. Only its horizontal advance becomes500; absent vmtx
    // still synthesizes the vertical advance from A-D=800-(-200)=1000.
    let control = chi_font();
    qualify_font(&control);
    let bytes = mapped_ahem(
        &[(0x41, 0x41, 274), (0x4e00, 0x4e00, 274)],
        Some((274, 500)),
    );
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    for tag in [*b"hhea", *b"loca", *b"glyf", *b"maxp"] {
        assert_eq!(table(&bytes, tag), table(&control, tag));
    }
    assert_eq!(u16_at(table(&bytes, *b"head"), 18), 1000);
    let mut expected = table(&control, *b"hmtx").to_vec();
    expected[274 * 4..274 * 4 + 2].copy_from_slice(&500_u16.to_be_bytes());
    assert_eq!(table(&bytes, *b"hmtx"), expected);
    let cmap = table(&bytes, *b"cmap");
    assert_eq!(u32_at(cmap, 24), 2);
    assert_eq!(
        [28, 32, 36, 40, 44, 48].map(|at| u32_at(cmap, at)),
        [0x41, 0x41, 274, 0x4e00, 0x4e00, 274]
    );
    bytes
}

fn capture(raw: &str, id: u64, boxes: &[(u64, t::InlineBoxKind, t::Size)]) -> Fixture {
    let bytes = mixed_font();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let face = system.registered_font_faces().next().unwrap().to_owned();
    let instance = face.unsynthesized_instance(4.0, &[]).unwrap();
    let nominal = instance
        .vertical_glyph_metrics(274, system.font_catalogue_identity())
        .unwrap();
    assert_eq!(
        (nominal.advance(), nominal.origin_x(), nominal.origin_y()),
        (4.0, 1.0, 3.2)
    );
    let horizontal = instance
        .baseline(
            system.font_catalogue_identity(),
            t::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            t::FontBaselineAxis::HorizontalY,
            t::FontBaseline::Central,
        )
        .unwrap();
    let t::FontBaselineOutcome::Available(horizontal) = horizontal.outcome() else {
        panic!("the real horizontal A/D midpoint supplies Central")
    };
    near(horizontal.coordinate(), 1.2);
    assert_eq!(
        horizontal.provenance(),
        t::FontBaselineProvenance::InstanceMetricsCssMidpoint
    );
    let style = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 4.0,
        line_height: t::LineHeight::Absolute(4.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        ..t::Style::default()
    };
    let mut source = t::Source::identified(raw, t::Id::from_u64(id), 1);
    for &(id, kind, size) in boxes {
        source.inline_box(t::InlineBox::new(t::Id::from_u64(id), kind, 1, size));
    }
    let len = raw.encode_utf16().count();
    let ranges = if boxes.is_empty() {
        vec![t::Range::new(0, len)]
    } else {
        assert_eq!(len, 2);
        vec![t::Range::new(0, 1), t::Range::new(1, 2)]
    };
    let connections = if boxes.is_empty() {
        vec![]
    } else {
        vec![t::ParagraphConnection {
            at: 1,
            kind: t::ParagraphConnectionKind::Continuous,
        }]
    };
    let mut policies = if boxes.is_empty() {
        vec![]
    } else {
        vec![t::ParagraphBoundaryDefinition::Connection(0)]
    };
    for (index, (_, kind, _)) in boxes.iter().enumerate() {
        if *kind == t::InlineBoxKind::InFlow {
            policies.extend([
                t::ParagraphBoundaryDefinition::AtomicBefore(index),
                t::ParagraphBoundaryDefinition::AtomicAfter(index),
            ]);
        }
    }
    let analysis = t::AnalyzedSource::from_paragraph(
        t::ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: t::ParagraphBaseDirection::LeftToRight,
            terminal_edge: t::ParagraphTerminalEdge::BlockEnd,
            paragraph_strut: t::ParagraphStrut::DefaultFont,
            tab_reference: None,
            origins: ranges
                .into_iter()
                .map(|range| t::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: range,
                    aggregate: range,
                    kind: t::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections,
            controls: vec![],
            boundary_policies: policies
                .into_iter()
                .map(|definition| t::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: t::Wrap::Word,
                    following_wrap: t::Wrap::Word,
                    ancestor_wrap: t::Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            anchor_order: (0..boxes.len())
                .map(|index| t::ParagraphAnchor {
                    item: t::ParagraphAnchorItem::Atomic(index),
                    affinity: t::Affinity::Before,
                })
                .collect(),
        },
        t::AnalysisTypography::Mixed,
    )
    .unwrap();
    let missing = analysis
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| t::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let analysis = analysis.with_wrapping_policies(missing).unwrap();
    let prepared = system
        .prepare(
            &analysis,
            t::PreparationOptions::default().with_orientation(t::LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(
        prepared.analyzed_source().typography(),
        t::AnalysisTypography::Mixed
    );
    assert_eq!(
        prepared.analyzed_source().direction(),
        t::Direction::LeftToRight
    );
    assert_eq!(
        prepared.baseline_reference(),
        Some(t::FontBaseline::Central)
    );
    Fixture {
        prepared,
        source,
        bytes,
    }
}

fn binding(
    f: &Fixture,
    container: u32,
    nodes: &[u32],
    strut: Option<t::LineStrut>,
) -> l::TextParagraphBinding<u32> {
    l::TextParagraphBinding {
        prepared: f.prepared.clone(),
        container,
        nodes: nodes.to_vec(),
        origins: f
            .prepared
            .origin_views()
            .zip(nodes)
            .map(|(view, &node)| l::TextOriginBinding {
                origin: view.handle(),
                node,
            })
            .collect(),
        atomics: vec![],
        controls: vec![],
        line_strut: strut,
    }
}

fn solve<S: l::LayoutScalar>(
    host: &Host<S>,
    bindings: &[l::TextParagraphBinding<u32>],
    space: l::Size<l::AvailableOf<S>>,
    dirty: &[u32],
) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
    let flow = l::LayoutFlowInput::try_new(
        host,
        0,
        bindings.to_vec(),
        l::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    l::compute_layout_with_flow_invalidated(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(space).unwrap(),
        dirty,
    )
}

fn available<S: l::LayoutScalar>(height: f64) -> l::Size<l::AvailableOf<S>> {
    l::Size::new(
        l::AvailableOf::definite(n(40.0)),
        l::AvailableOf::definite(n(height)),
    )
}

fn central_facts<S: l::LayoutScalar>(facts: l::NamedBaselinesOf<S>, first: f64, last: f64) {
    let pair = facts.get(l::BaselineName::Central);
    for (actual, expected) in [(pair.first(), first), (pair.last(), last)] {
        let point =
            l::BaselineCoordinateOf::try_new(l::PhysicalAxis::Horizontal, n(expected)).unwrap();
        assert_eq!(actual, l::BaselineFactOf::Available(point));
    }
}

fn placed<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    container: u32,
) -> &t::Layout {
    batch
        .placed_text()
        .iter()
        .find(|row| row.container() == container)
        .unwrap()
        .layout()
}

fn packets(text: &t::Layout, fixture: &Fixture, centers: &[f64], inline: &[f64]) {
    assert_eq!(text.source(), &fixture.source);
    assert_eq!(text.orientation(), t::LineOrientation::Vertical);
    assert_eq!(
        text.analyzed_source().identity(),
        fixture.prepared.analyzed_source().identity()
    );
    let mut seen = Vec::new();
    for run in text.run_views() {
        assert_eq!(run.face().bytes(), fixture.bytes);
        assert_eq!(
            run.face().catalogue_identity(),
            fixture.prepared.font_catalogue_identity()
        );
        assert_eq!(run.font_size(), 4.0);
        for glyph in run.glyphs() {
            let index = run.range().start;
            let upright = index % 2 == 1;
            assert_eq!(run.range(), t::Range::new(index, index + 1));
            assert_eq!(glyph.id(), 274);
            assert_eq!(
                run.glyph_basis(),
                if upright {
                    t::GlyphBasis::Unrotated
                } else {
                    t::GlyphBasis::Clockwise
                }
            );
            assert_eq!(
                (glyph.x(), glyph.y(), glyph.advance()),
                if upright {
                    (-1.0, 3.2, 4.0)
                } else {
                    (0.0, 0.0, 2.0)
                }
            );
            near(
                run.origin().x,
                centers[run.line()] - if upright { 0.0 } else { 1.2 },
            );
            near(run.origin().y + glyph.pen(), inline[index]);
            seen.push(index);
        }
    }
    seen.sort_unstable();
    assert_eq!(seen, (0..inline.len()).collect::<Vec<_>>());
    let start = text
        .positions()
        .find(|p| p.source_cursor() == t::Cursor::new(0, t::Affinity::After))
        .unwrap();
    let end = text
        .positions()
        .find(|p| p.source_cursor() == t::Cursor::new(inline.len(), t::Affinity::Before))
        .unwrap();
    let selection = text.position_selection(&start, &end).unwrap();
    assert_eq!(selection.source_text(), fixture.source.text());
    assert_eq!(selection.plaintext(), fixture.source.text());
}

fn admission<S: l::LayoutScalar>() {
    let fixture = capture("A一", 1_570_099, &[]);
    for (mode, c) in [
        (l::WritingMode::VerticalRl, 32.0),
        (l::WritingMode::VerticalLr, 2.0),
    ] {
        let mut input = root(mode, l::Direction::Ltr, central());
        input.size = l::Size::new(px(40.0), px(6.0));
        let mut host = Host::<S>::new(input);
        let b = binding(&fixture, 0, &[1], Some(STRUT));
        host.push(
            Some(0),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(b.origins[0].origin.clone())),
        );
        let batch = solve(&host, &[b], available(6.0), &[]).unwrap();
        // One actual mixed line is enough to require the same authenticated
        // Central name as the already supported whole-upright operation.
        central_facts(node(batch.unrounded_entries(), 0).named_baselines, c, c);
        assert_eq!(placed(&batch, 0).lines().len(), 1);
        packets(placed(&batch, 0), &fixture, &[c], &[0.0, 2.0]);
    }
}

#[test]
fn native_mixed_named_central_admission_f32() {
    admission::<f32>();
}
#[test]
fn native_mixed_named_central_admission_f64() {
    admission::<f64>();
}

fn columns<S: l::LayoutScalar>() {
    let fixture = capture("A一A一", 1_570_100, &[]);
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        for context in [central(), l::InlineBaselineContext::Physical] {
            let mut input = root(mode, l::Direction::Ltr, context);
            input.size = l::Size::new(px(40.0), px(6.0));
            let mut host = Host::<S>::new(input);
            let b = binding(&fixture, 0, &[1], Some(STRUT));
            host.push(
                Some(0),
                l::LayoutInputOf::inline_text(l::TextParticipation::new(
                    b.origins[0].origin.clone(),
                )),
            );
            let batch = solve(&host, &[b], available(6.0), &[])
                .expect("real Mixed named Central must be admitted by Layout");
            let centers = if mode == l::WritingMode::VerticalRl {
                [32.0, 22.0]
            } else {
                [2.0, 12.0]
            };
            if context == central() {
                central_facts(
                    node(batch.unrounded_entries(), 0).named_baselines,
                    centers[0],
                    centers[1],
                );
            } else {
                assert_eq!(
                    node(batch.unrounded_entries(), 0).named_baselines,
                    l::NamedBaselinesOf::unknown()
                );
            }
            let text = placed(&batch, 0);
            assert_eq!(
                text.lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [t::Range::new(0, 2), t::Range::new(2, 4)]
            );
            for (line, c) in text.lines().iter().zip(centers) {
                assert_eq!(line.baseline(), c);
                assert_eq!(line.bounds(), t::Rect::new(c - 2.0, 0.0, 10.0, 6.0));
            }
            packets(text, &fixture, &centers, &[0.0, 2.0, 0.0, 2.0]);
            let mut ranges = Vec::new();
            for row in batch.final_inline_fragments() {
                let f = row.fragment();
                let issued = f.finalized_fragment();
                let source = issued.view().aggregate_range().unwrap();
                assert_eq!(
                    issued.view().origin(),
                    Some(fixture.prepared.origin_views().next().unwrap().handle())
                );
                assert_eq!(issued.view().local_range(), Some(source));
                let c = centers[f.line_index()];
                let (y, height) = if source.start % 2 == 0 {
                    (0.0, 2.0)
                } else {
                    (2.0, 4.0)
                };
                let rect = f.rect().unwrap();
                assert_eq!(
                    (rect.origin(), rect.size()),
                    (
                        l::Point::new(n(c - 2.0), n(y)),
                        l::Size::new(n(4.0), n(height))
                    )
                );
                ranges.push(source);
            }
            assert_eq!(
                ranges,
                [
                    t::Range::new(0, 1),
                    t::Range::new(1, 2),
                    t::Range::new(2, 3),
                    t::Range::new(3, 4)
                ]
            );
        }
        let mut input = root(mode, l::Direction::Ltr, central());
        input.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO);
        let mut host = Host::<S>::new(input);
        let b = binding(&fixture, 0, &[1], Some(STRUT));
        host.push(
            Some(0),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(b.origins[0].origin.clone())),
        );
        let batch = solve(
            &host,
            &[b],
            l::Size::splat(l::AvailableOf::MAX_CONTENT),
            &[],
        )
        .unwrap();
        assert_eq!(
            node(batch.unrounded_entries(), 0).size,
            l::Size::new(n(10.0), n(12.0))
        );
        central_facts(node(batch.unrounded_entries(), 0).named_baselines, 2.0, 2.0);
        packets(placed(&batch, 0), &fixture, &[2.0], &[0.0, 2.0, 6.0, 8.0]);
    }
}

#[test]
fn native_mixed_named_central_columns_preserve_axes_source_and_intrinsic_geometry_f32() {
    columns::<f32>();
}
#[test]
fn native_mixed_named_central_columns_preserve_axes_source_and_intrinsic_geometry_f64() {
    columns::<f64>();
}

struct Scene<S: l::LayoutScalar> {
    host: Host<S>,
    parent: Fixture,
    bindings: Vec<l::TextParagraphBinding<u32>>,
}

fn scene<S: l::LayoutScalar>(mode: l::WritingMode) -> Scene<S> {
    let parent = capture(
        "A一",
        1_570_200,
        &[
            (2, t::InlineBoxKind::InFlow, t::Size::new(6.0, 8.0)),
            (3, t::InlineBoxKind::OutOfFlow, t::Size::new(5.0, 7.0)),
        ],
    );
    let child = capture("A一", 1_570_201, &[]);
    let absolute = capture("A一", 1_570_202, &[]);
    let mut input = root(mode, l::Direction::Ltr, central());
    input.size = l::Size::new(px(40.0), px(14.0));
    let mut host = Host::new(input);
    let origins = parent
        .prepared
        .origin_views()
        .map(|v| v.handle())
        .collect::<Vec<_>>();
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[0].clone())),
    );
    let mut input = root(l::WritingMode::VerticalLr, l::Direction::Ltr, central());
    input.display = l::Display::InlineBlock;
    input.size = l::Size::new(px(6.0), px(8.0));
    host.push(Some(0), l::LayoutInputOf::box_input(input));
    let mut input = root(
        l::WritingMode::VerticalLr,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    input.position = l::Position::Absolute;
    input.size = l::Size::new(px(5.0), px(7.0));
    host.push(Some(0), l::LayoutInputOf::box_input(input));
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[1].clone())),
    );
    let inner = binding(&child, 2, &[5], None);
    host.push(
        Some(2),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(inner.origins[0].origin.clone())),
    );
    let out = binding(&absolute, 3, &[6], None);
    host.push(
        Some(3),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(out.origins[0].origin.clone())),
    );
    let outer = l::TextParagraphBinding {
        prepared: parent.prepared.clone(),
        container: 0,
        nodes: vec![1, 2, 3, 4],
        origins: origins
            .into_iter()
            .zip([1, 4])
            .map(|(origin, node)| l::TextOriginBinding { origin, node })
            .collect(),
        atomics: parent
            .prepared
            .atomic_views()
            .zip([2, 3])
            .map(|(view, node)| l::TextAtomicBinding {
                occurrence: view.handle(),
                node,
            })
            .collect(),
        controls: vec![],
        line_strut: Some(STRUT),
    };
    Scene {
        host,
        parent,
        bindings: vec![outer, inner, out],
    }
}

fn atomics<S: l::LayoutScalar>() {
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let scene = scene::<S>(mode);
        let batch = solve(&scene.host, &scene.bindings, available(14.0), &[]).unwrap();
        let c = if mode == l::WritingMode::VerticalRl {
            32.0
        } else {
            2.0
        };
        let out_x = if mode == l::WritingMode::VerticalRl {
            // Static block-start is C+B=32+8=40; absolute RTL placement
            // subtracts this child's actual width5, not the in-flow width6.
            35.0
        } else {
            0.0
        };
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            central_facts(node(entries, 0).named_baselines, c, c);
            let child = node(entries, 2);
            assert_eq!(
                (child.location, child.size),
                (
                    l::Point::new(n(c - 2.0), n(2.0)),
                    l::Size::new(n(6.0), n(8.0))
                )
            );
            central_facts(child.named_baselines, 2.0, 2.0);
            let out = node(entries, 3);
            assert_eq!(
                (out.location, out.size),
                (
                    l::Point::new(n(out_x), n(10.0)),
                    l::Size::new(n(5.0), n(7.0))
                )
            );
            assert_eq!(out.named_baselines, l::NamedBaselinesOf::unknown());
        }
        let text = placed(&batch, 0);
        packets(text, &scene.parent, &[c], &[0.0, 10.0]);
        assert_eq!(
            text.inline_boxes()
                .iter()
                .map(|b| (b.id(), b.kind(), b.rect()))
                .collect::<Vec<_>>(),
            [
                (
                    t::Id::from_u64(2),
                    t::InlineBoxKind::InFlow,
                    t::Rect::new(c - 2.0, 2.0, 6.0, 8.0)
                ),
                (
                    t::Id::from_u64(3),
                    t::InlineBoxKind::OutOfFlow,
                    t::Rect::new(out_x, 10.0, 5.0, 7.0)
                ),
            ]
        );
        assert_eq!(placed(&batch, 2).lines()[0].baseline(), c);
        assert_eq!(placed(&batch, 3).lines()[0].baseline(), out_x + 2.0);
    }
}

#[test]
fn native_mixed_actual_children_keep_named_physical_custody_and_unrotated_boxes_f32() {
    atomics::<f32>();
}
#[test]
fn native_mixed_actual_children_keep_named_physical_custody_and_unrotated_boxes_f64() {
    atomics::<f64>();
}

struct RefusingSink<'a, S: l::LayoutScalar>(&'a mut Host<S>);
impl<S: l::LayoutScalar> l::LayoutBatchSink<u32, S> for RefusingSink<'_, S> {
    type Error = &'static str;
    type Prepared = Published<S>;
    fn prepare_layout_batch(
        &self,
        batch: &l::CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let _staged = l::LayoutBatchSink::prepare_layout_batch(&*self.0, batch).unwrap();
        Err("reject complete staged Mixed replacement")
    }
    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        l::LayoutBatchSink::commit_layout_batch(self.0, prepared);
    }
}

fn publication<S: l::LayoutScalar>() {
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let mut scene = scene::<S>(mode);
        let old = solve(&scene.host, &scene.bindings, available(14.0), &[]).unwrap();
        old.apply_to(&mut scene.host).unwrap();
        let caches = scene.host.published.caches.clone();
        let nodes = scene.host.published.unrounded.clone();
        let position = placed(&old, 0).positions().next().unwrap();
        let original_input = scene.host.inputs[1].clone();
        let other = capture("A一", 1_570_203, &[]);
        let foreign_origin = other.prepared.origin_views().next().unwrap().handle();
        scene.host.inputs[1] =
            l::LayoutInputOf::inline_text(l::TextParticipation::new(foreign_origin));
        let stale = l::LayoutFlowInput::try_new(
            &scene.host,
            0,
            scene.bindings.clone(),
            l::WrappingInput::default(),
            Vec::new(),
        )
        .err()
        .expect("replacing a source occurrence cannot reuse its old paragraph binding");
        assert!(matches!(
            stale.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::TextBinding(
                l::TextBindingError::ParticipantRole
            ))
        ));
        scene.host.inputs[1] = original_input;
        scene.host.change(0, |input| input.size.height = px(18.0));
        scene.host.change(2, |input| input.size.height = px(10.0));
        let changed = solve(&scene.host, &scene.bindings, available(18.0), &[0, 2]).unwrap();
        assert_eq!(
            node(changed.final_entries(), 2).size,
            l::Size::new(n(6.0), n(10.0))
        );
        let foreign = placed(&changed, 0)
            .position_geometry(&position)
            .unwrap_err();
        assert_eq!(
            foreign.detail(),
            Some(&t::ErrorDetail::InvalidTextProtocol {
                reason: t::ProtocolError::ForeignPlacement
            })
        );
        assert_eq!(
            changed.apply_to(&mut RefusingSink(&mut scene.host)),
            Err("reject complete staged Mixed replacement")
        );
        assert_eq!(scene.host.published.caches, caches);
        assert_eq!(scene.host.published.unrounded, nodes);
        let retained = scene.host.published.batch.as_ref().unwrap();
        assert_eq!(retained.final_entries(), old.final_entries());
        assert_eq!(
            retained.final_inline_fragments(),
            old.final_inline_fragments()
        );
        assert_eq!(placed(retained, 0).source(), &scene.parent.source);
        assert_eq!(placed(retained, 0).lines(), placed(&old, 0).lines());
        assert_eq!(
            placed(retained, 0).inline_boxes(),
            placed(&old, 0).inline_boxes()
        );
        changed.apply_to(&mut scene.host).unwrap();
        assert_eq!(scene.host.commits, 2);
    }
}

#[test]
fn native_mixed_stale_positions_and_late_sink_refusal_preserve_prior_packet_f32() {
    publication::<f32>();
}
#[test]
fn native_mixed_stale_positions_and_late_sink_refusal_preserve_prior_packet_f64() {
    publication::<f64>();
}
