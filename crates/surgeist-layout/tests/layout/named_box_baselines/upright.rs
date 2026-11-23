//! Real native Central cells through Layout's named and Physical adapters.
//! Font sides2/2 are . The separately supplied line strut8/2 establishes
//! finalized B8/H10, distinguishing line-over from vertical block progression.
use super::*;
use surgeist_text::font::FontBaseline;
mod atomics;
mod constraints;
mod mixed;
mod positioned;
mod supplemental;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));
const STRUT: t::allocation::LineStrut = t::allocation::LineStrut {
    baseline: 8.0,
    line_extent: 10.0,
};

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let at = (0..usize::from(u16_at(bytes, 4)))
        .map(|index| 12 + 16 * index)
        .find(|at| bytes[*at..*at + 4] == tag)
        .unwrap();
    let start = usize::try_from(u32_at(bytes, at + 8)).unwrap();
    let len = usize::try_from(u32_at(bytes, at + 12)).unwrap();
    &bytes[start..start + len]
}
fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, bytes| {
        let mut word = [0; 4];
        word[..bytes.len()].copy_from_slice(bytes);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn chi_font() -> Vec<u8> {
    // The same safe cmap-only derivative as Render's text_layout_adapter/
    // upright.rs. U+4E00 maps to licensed Ahem chi274, not a claimed Han design.
    mapped_ahem(&[(0x4e00, 0x4e00, 274)], None)
}

fn mapped_ahem(groups: &[(u32, u32, u32)], advance: Option<(u16, u16)>) -> Vec<u8> {
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10];
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend([0, 12, 0, 0]);
    for word in [
        16 + u32::try_from(groups.len()).unwrap() * 12,
        0,
        u32::try_from(groups.len()).unwrap(),
    ] {
        cmap.extend(word.to_be_bytes());
    }
    for &(first, last, glyph) in groups {
        for word in [first, last, glyph] {
            cmap.extend(word.to_be_bytes());
        }
    }
    let mut output = vec![0; 12 + usize::from(u16_at(AHEM, 4)) * 16];
    output[..12].copy_from_slice(&AHEM[..12]);
    for index in 0..usize::from(u16_at(AHEM, 4)) {
        let record = 12 + index * 16;
        let tag: [u8; 4] = AHEM[record..record + 4].try_into().unwrap();
        let mut data = if tag == *b"cmap" {
            cmap.clone()
        } else {
            table(AHEM, tag).to_vec()
        };
        if tag == *b"head" {
            data[8..12].fill(0);
        }
        if tag == *b"hmtx"
            && let Some((glyph, width)) = advance
        {
            let at = usize::from(glyph) * 4;
            data[at..at + 2].copy_from_slice(&width.to_be_bytes());
        }
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        let start = u32::try_from(output.len()).unwrap();
        output[record..record + 4].copy_from_slice(&tag);
        output[record + 4..record + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        output[record + 8..record + 12].copy_from_slice(&start.to_be_bytes());
        output[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    while !output.len().is_multiple_of(4) {
        output.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&output));
    let head = (0..usize::from(u16_at(&output, 4)))
        .map(|index| 12 + index * 16)
        .find(|at| output[*at..*at + 4] == *b"head")
        .unwrap();
    let start = usize::try_from(u32_at(&output, head + 8)).unwrap();
    output[start + 8..start + 12].copy_from_slice(&adjustment.to_be_bytes());
    output
}

fn qualify_font(bytes: &[u8]) {
    assert_eq!(checksum(bytes), 0xb1b0_afba);
    assert_eq!(u16_at(table(bytes, *b"head"), 18), 1000);
    assert_eq!(u16_at(table(bytes, *b"maxp"), 4), 281);
    for tag in [*b"glyf", *b"loca", *b"maxp", *b"hmtx", *b"hhea"] {
        assert_eq!(table(bytes, tag), table(AHEM, tag));
    }
    let tags = (0..usize::from(u16_at(bytes, 4)))
        .map(|index| &bytes[12 + index * 16..16 + index * 16])
        .collect::<Vec<_>>();
    for absent in [
        b"BASE", b"vhea", b"vmtx", b"fvar", b"CFF ", b"CFF2", b"VARC",
    ] {
        assert!(!tags.contains(&absent.as_slice()));
    }
    let hhea = table(bytes, *b"hhea");
    assert_eq!(
        [4, 6, 8].map(|at| i16::from_be_bytes(hhea[at..at + 2].try_into().unwrap())),
        [800, -200, 0]
    );
    assert_eq!(u16_at(table(bytes, *b"hmtx"), 274 * 4), 1000);
    let cmap = table(bytes, *b"cmap");
    assert_eq!(u16_at(cmap, 2), 1);
    assert_eq!(u16_at(cmap, 12), 12);
    assert_eq!(u32_at(cmap, 24), 1);
    assert_eq!(
        [28, 32, 36].map(|at| u32_at(cmap, at)),
        [0x4e00, 0x4e00, 274]
    );
    // Independent stored outline bounds; no inferred font em from glyph ink.
    assert_eq!(u16_at(table(bytes, *b"head"), 50), 0);
    let loca = table(bytes, *b"loca");
    let start = usize::from(u16_at(loca, 274 * 2)) * 2;
    let end = usize::from(u16_at(loca, 275 * 2)) * 2;
    assert!(end > start + 10);
    let glyph = &table(bytes, *b"glyf")[start..end];
    assert_eq!(
        [2, 4, 6, 8].map(|at| i16::from_be_bytes(glyph[at..at + 2].try_into().unwrap())),
        [0, 400, 1000, 600]
    );
}

fn select_line(
    prepared: &t::prepared::PreparedText,
    start: t::prepared::PreparedBoundary,
    offer: t::prepared::PreparedOpportunity,
    available: Option<f64>,
) -> t::finalized::FinalizedLine {
    select_line_in_direction(
        prepared,
        start,
        offer,
        available,
        l::geometry::Direction::Ltr,
    )
}

fn select_line_in_direction(
    prepared: &t::prepared::PreparedText,
    start: t::prepared::PreparedBoundary,
    offer: t::prepared::PreparedOpportunity,
    available: Option<f64>,
    direction: l::geometry::Direction,
) -> t::finalized::FinalizedLine {
    let from = start.view().participant_offset();
    let end = offer.view().boundary();
    let to = end.view().participant_offset();
    let terminal = offer.view().kind() == t::prepared::PreparedBreakKind::ParagraphEnd;
    prepared
        .select(t::selection::LineSelectionRequest {
            inline_boxes: None,
            segment: prepared.source_segments().next().unwrap(),
            start,
            end,
            participants: prepared.participants().skip(from).take(to - from).collect(),
            termination: t::selection::PreparedTermination::Ordinary(offer),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: Some(STRUT),
            line_box_struts: vec![],
            context: t::selection::LineContext {
                alignment_insets: t::allocation::LineAlignmentInsets::ZERO,
                formatted_line: t::selection::FormattedLineContext::default(),
                base_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
                line_box_direction: if direction == l::geometry::Direction::Ltr {
                    t::selection::LineBoxDirection::LeftToRight
                } else {
                    t::selection::LineBoxDirection::RightToLeft
                },
                orientation: t::selection::LineOrientation::Vertical,
                start: if from == 0 {
                    t::selection::LineStart::Paragraph
                } else {
                    t::selection::LineStart::Continuation
                },
                end: if terminal {
                    t::selection::LineEnd::Paragraph
                } else {
                    t::selection::LineEnd::Soft
                },
                available_inline: available,
                indent: 0.0,
                alignment: t::options::Alignment::Start,
                tabs: t::tabs::TabContext::NoTabs,
                spacing: t::spacing::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(surgeist_text::scope_alignment::LineAllocations {
            spacing: t::spacing::SpacingAllocations::None,
            scopes: surgeist_text::scope_alignment::ScopeAllocations::None,
        })
        .unwrap()
}

struct Fixture {
    prepared: t::prepared::PreparedText,
    source: t::source::Source,
    bytes: Vec<u8>,
}

fn fixture() -> Fixture {
    fixture_in_direction(l::geometry::Direction::Ltr)
}

fn fixture_in_direction(direction: l::geometry::Direction) -> Fixture {
    let bytes = chi_font();
    qualify_font(&bytes);
    let mut system = t::System::default();
    system
        .register_fonts(bytes.clone(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let face = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(face.bytes(), bytes);
    assert_eq!(face.index(), 0);
    let instance = face.unsynthesized_instance(4.0, &[]).unwrap();
    assert_eq!(instance.axis_count(), 0);
    let catalogue = system.font_catalogue_identity();
    let nominal = instance.vertical_glyph_metrics(274, catalogue).unwrap();
    // Actual absence synthesis: (A-D,W/2,A)*size/UPEM, not fabricated em sides.
    assert_eq!(
        (nominal.advance(), nominal.origin_x(), nominal.origin_y()),
        (4.0, 2.0, 3.2)
    );
    let cross = instance
        .upright_metrics(
            catalogue,
            t::font::FontBaselineScript::from_iso15924(*b"Hani").unwrap(),
        )
        .unwrap();
    assert_eq!(
        cross.reference(),
        t::font::UprightMetricReference::NominalVerticalPenX
    );
    assert_eq!(
        (
            cross.central_coordinate_or_distance(),
            cross.used_over(),
            cross.used_under()
        ),
        (0.0, 2.0, -2.0)
    );
    let style = t::style::Style {
        font: t::style::Font::new().family("Ahem"),
        size: 4.0,
        line_height: t::style::LineHeight::Absolute(4.0),
        white_space: t::style::WhiteSpace::Preserve,
        wrap: t::style::Wrap::Word,
        ..t::style::Style::default()
    };
    let source =
        t::source::Source::identified("\u{4e00}\u{4e00}", t::source::Id::from_u64(1_550_001), 1);
    use t::capture::SourceProcessingEvent as E;
    let capture = Arc::new(
        t::capture::process_contextual_source(t::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::capture::SourceProcessingEnvelope::Complete,
            origins: vec![t::paragraph::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: t::source::Range::new(0, 2),
                aggregate: t::source::Range::new(0, 2),
                kind: t::paragraph::ParagraphOriginKind::Authored,
            }],
            controls: vec![],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![t::capture::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: t::capture::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: if direction == l::geometry::Direction::Ltr {
                    t::paragraph::ParagraphBaseDirection::LeftToRight
                } else {
                    t::paragraph::ParagraphBaseDirection::RightToLeft
                },
            }],
            paragraphs: vec![t::capture::SourceProcessingParagraph {
                base_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: t::layout::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let typography = t::analyzed::AnalysisTypography::Upright {
        computed_direction: t::paragraph::ParagraphBaseDirection::RightToLeft,
    };
    let analyzed = t::analyzed::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        typography,
    )
    .unwrap();
    let wrapping = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| t::analyzed::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: t::style::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(wrapping).unwrap();
    assert_eq!(analyzed.typography(), typography);
    assert_eq!(analyzed.direction(), t::style::Direction::LeftToRight);
    assert_eq!(analyzed.source(), &source);
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [t::source::Range::new(0, 1), t::source::Range::new(1, 2)]
    );
    let prepared = system
        .prepare(
            &analyzed,
            t::prepared::PreparationOptions::default()
                .with_orientation(t::selection::LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(
        prepared.orientation(),
        t::selection::LineOrientation::Vertical
    );
    assert_eq!(prepared.origin_views().count(), 1);
    let first = prepared.boundary_views().next().unwrap().handle();
    let terminal = prepared
        .opportunity_views()
        .find(|o| o.kind() == t::prepared::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let whole = if direction == l::geometry::Direction::Ltr {
        select_line(&prepared, first.clone(), terminal.clone(), None)
    } else {
        select_line_in_direction(&prepared, first.clone(), terminal.clone(), None, direction)
    };
    assert_eq!(
        (
            whole.feedback().baseline,
            whole.feedback().line_height,
            whole.feedback().advance
        ),
        (8.0, 10.0, 8.0)
    );
    let glyphs = whole
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.glyph_basis(), t::geometry::GlyphBasis::Unrotated);
            assert_eq!(run.face().bytes(), bytes);
            run.glyphs()
                .map(|g| (g.range(), g.id(), g.x(), g.y(), g.advance(), g.pen()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [
            (t::source::Range::new(0, 1), 274, -2.0, 3.2, 4.0, 0.0),
            (t::source::Range::new(1, 2), 274, -2.0, 3.2, 4.0, 4.0)
        ]
    );
    let soft = prepared
        .opportunity_views()
        .find(|o| {
            o.kind() == t::prepared::PreparedBreakKind::Allowed
                && o.boundary().view().source_before().get() == 1
        })
        .unwrap()
        .handle();
    let next = soft.view().boundary();
    for line in [
        select_line_in_direction(&prepared, first, soft, Some(4.0), direction),
        select_line_in_direction(&prepared, next, terminal, Some(4.0), direction),
    ] {
        // The actual allocation8/2 combines with font2/2 as max8/max2.
        assert_eq!(
            (
                line.feedback().baseline,
                line.feedback().line_height,
                line.feedback().advance
            ),
            (8.0, 10.0, 4.0)
        );
        assert_eq!(line.selected_line().request().line_strut, Some(STRUT));
        let fragments = line
            .fragment_views()
            .filter(|f| f.origin().is_some())
            .collect::<Vec<_>>();
        assert_eq!(fragments.len(), 1);
        // Font-owned source bounds are4 high at local v6..10, not allocation10.
        assert_eq!(
            fragments[0].bounds(),
            Some(t::geometry::Rect::new(0.0, 6.0, 4.0, 4.0))
        );
    }
    drop(system);
    Fixture {
        prepared,
        source,
        bytes,
    }
}

fn compute<S: l::scalar::LayoutScalar>(
    fixture: &Fixture,
    mode: l::geometry::WritingMode,
    context: l::baseline::InlineBaselineContext,
) -> l::batch::CompletedLayoutBatchOf<u32, S> {
    let mut input = root(mode, l::geometry::Direction::Ltr, context);
    input.size = l::geometry::Size::new(px(40.0), px(4.0));
    let mut host = Host::<S>::new(input);
    let origin = fixture.prepared.origin_views().next().unwrap().handle();
    assert_eq!(
        host.push(
            Some(0),
            l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                origin.clone()
            ))
        ),
        1
    );
    let binding = l::text_layout::TextParagraphBinding {
        inline_box_fragments:
            surgeist_layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
        prepared: fixture.prepared.clone(),
        container: 0,
        nodes: vec![1],
        origins: vec![l::text_layout::TextOriginBinding { origin, node: 1 }],
        controls: vec![],
        atomics: vec![],
        line_strut: Some(STRUT),
    };
    let flow = l::text_layout::LayoutFlowInput::try_new(
        &host,
        0,
        vec![binding],
        l::inline_wrapping::WrappingInput::default(),
        Vec::new(),
    )
    .expect("processed capture and issued origin must bind to the actual host");
    let viewport = {
        let sizing_request = crate::layout::native_request::sizing(
            surgeist_layout::geometry::Size::new(Some(n(40.0)), Some(n(4.0))),
            surgeist_layout::geometry::Size::new(
                surgeist_layout::constraint::SizingOperation::Normal,
                surgeist_layout::constraint::SizingOperation::Normal,
            ),
        );
        l::request::LayoutRootRequestOf::standalone(
            sizing_request,
            surgeist_layout::geometry::FlowAxes::new(
                surgeist_layout::tree::LayoutTree::node_input(&host, 0).writing_mode,
                surgeist_layout::tree::LayoutTree::node_input(&host, 0).direction,
            ),
            sizing_request
                .orthogonal_fallback()
                .initial_containing_block(),
        )
    };
    l::compute_layout_with_flow(&flow, 0, viewport)
        .expect("qualified native upright Central cells must reach completed Layout output")
}

fn geometry<S: l::scalar::LayoutScalar>(
    fixture: &Fixture,
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    centers: [f64; 2],
) {
    assert_eq!(
        node(crate::layout::observation::phase(batch, false), 0).size,
        l::geometry::Size::new(n(40.0), n(4.0))
    );
    assert_eq!(batch.placed_text().len(), 1);
    let placed = &batch.placed_text()[0];
    assert_eq!(placed.container(), 0);
    assert_eq!(placed.nodes(), [1]);
    let text = placed.layout();
    assert_eq!(text.source(), &fixture.source);
    assert_eq!(text.source().text().code_units(), [0x4e00, 0x4e00]);
    assert_eq!(
        text.analyzed_source().identity(),
        fixture.prepared.analyzed_source().identity()
    );
    assert_eq!(text.orientation(), t::selection::LineOrientation::Vertical);
    assert_eq!(text.lines().len(), 2);
    for (index, line) in text.lines().iter().enumerate() {
        let c = centers[index];
        assert_eq!(line.range(), t::source::Range::new(index, index + 1));
        assert_eq!(line.orientation(), t::selection::LineOrientation::Vertical);
        assert_eq!(line.baseline(), c);
        assert_eq!(
            line.bounds(),
            t::geometry::Rect::new(c - 2.0, 0.0, 10.0, 4.0)
        );
    }
    let runs = text.run_views().collect::<Vec<_>>();
    assert_eq!(runs.len(), 2);
    for (index, run) in runs.iter().enumerate() {
        assert_eq!(run.origin(), t::geometry::Point::new(centers[index], 0.0));
        assert_eq!(run.range(), t::source::Range::new(index, index + 1));
        assert_eq!(run.glyph_basis(), t::geometry::GlyphBasis::Unrotated);
        assert_eq!(run.face().bytes(), fixture.bytes);
        assert_eq!(run.face().index(), 0);
        assert_eq!(
            run.face().catalogue_identity(),
            fixture.prepared.font_catalogue_identity()
        );
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 4.0);
        assert_eq!(
            instance.normalized_coords().collect::<Vec<_>>(),
            Vec::<i16>::new()
        );
        let glyphs = run.glyphs().collect::<Vec<_>>();
        assert_eq!(glyphs.len(), 1);
        let glyph = glyphs[0];
        assert_eq!(
            (
                glyph.id(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
                glyph.pen()
            ),
            (274, -2.0, 3.2, 4.0, 0.0)
        );
    }
    let origin = fixture.prepared.origin_views().next().unwrap().handle();
    let fragments = batch
        .unrounded_inline_fragments()
        .iter()
        .filter(|row| matches!(row.owner(), l::text_layout::InlineFragmentOwner::Node(1)))
        .collect::<Vec<_>>();
    assert_eq!(fragments.len(), 2);
    for (index, row) in fragments.iter().enumerate() {
        let fragment = row.fragment();
        assert_eq!(fragment.line_index(), index);
        assert_eq!(
            fragment.baseline(),
            l::geometry::Point::new(n(centers[index]), n(0.0))
        );
        let rect = l::scroll::ScrollRectOf::try_new(
            l::geometry::Point::new(n(centers[index] - 2.0), n(0.0)),
            l::geometry::Size::new(n(4.0), n(4.0)),
        )
        .unwrap();
        assert_eq!(fragment.rect(), Some(rect));
        let receipt = fragment.finalized_fragment();
        assert_eq!(receipt.view().origin(), Some(origin.clone()));
        assert_eq!(
            receipt.view().aggregate_range(),
            Some(t::source::Range::new(index, index + 1))
        );
        assert_eq!(
            receipt.view().local_range(),
            Some(t::source::Range::new(index, index + 1))
        );
        let line = receipt.finalized_line();
        assert_eq!(line.prepared_text(), fixture.prepared);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (8.0, 10.0)
        );
        assert_eq!(line.selected_line().request().line_strut, Some(STRUT));
    }
}

fn named_columns<S: l::scalar::LayoutScalar>() {
    let fixture = fixture();
    for (mode, centers) in [
        (l::geometry::WritingMode::VerticalRl, [32.0, 22.0]),
        (l::geometry::WritingMode::VerticalLr, [2.0, 12.0]),
    ] {
        // Allocation H10/B8: Vrl C=40-k-8; Vlr C=k+(10-8), k=0/10.
        let batch = compute::<S>(
            &fixture,
            mode,
            l::baseline::InlineBaselineContext::Named(FontBaseline::Central),
        );
        geometry(&fixture, &batch, centers);
        let facts = node(crate::layout::observation::phase(&batch, false), 0).named_baselines;
        let pair = facts.get(FontBaseline::Central);
        for (fact, c) in [(pair.first(), centers[0]), (pair.last(), centers[1])] {
            let coordinate = l::baseline::BaselineCoordinateOf::try_new(
                l::geometry::PhysicalAxis::Horizontal,
                n(c),
            )
            .unwrap();
            assert_eq!(fact, l::baseline::BaselineFactOf::Available(coordinate));
        }
        let alpha = facts.get(FontBaseline::Alphabetic);
        assert_eq!(
            (alpha.first(), alpha.last()),
            (
                l::baseline::BaselineFactOf::Unknown,
                l::baseline::BaselineFactOf::Unknown
            )
        );
    }
}

fn physical_columns<S: l::scalar::LayoutScalar>() {
    let fixture = fixture();
    for (mode, centers) in [
        (l::geometry::WritingMode::VerticalRl, [32.0, 22.0]),
        (l::geometry::WritingMode::VerticalLr, [2.0, 12.0]),
    ] {
        let batch = compute::<S>(&fixture, mode, l::baseline::InlineBaselineContext::Physical);
        geometry(&fixture, &batch, centers);
        assert_eq!(
            node(crate::layout::observation::phase(&batch, false), 0).named_baselines,
            l::baseline::NamedBaselinesOf::unknown()
        );
    }
}

#[test]
fn native_upright_named_columns_retain_actual_central_f32() {
    named_columns::<f32>();
}
#[test]
fn native_upright_named_columns_retain_actual_central_f64() {
    named_columns::<f64>();
}
#[test]
fn native_upright_physical_columns_use_the_same_cell_frame_f32() {
    physical_columns::<f32>();
}
#[test]
fn native_upright_physical_columns_use_the_same_cell_frame_f64() {
    physical_columns::<f64>();
}
