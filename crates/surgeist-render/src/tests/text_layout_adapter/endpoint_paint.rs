//! Actual native endpoint geometry displaces retained Text ink without painting
//! an endpoint rectangle. The pixel oracle uses the licensed Ahem outline and
//! integer translation invariance, not observed raster-count snapshots.

use super::*;
use crate::{Options, Parameters, RenderRoute, Renderer, Size};
use skrifa::MetadataProvider;
use std::sync::Arc;

const SIDE: usize = 32;
const BASELINE: f64 = 12.0;

fn qualify_ahem() {
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.charmap().map('A').unwrap().to_u32(), 35);
    let head = table(AHEM, *b"head");
    assert_eq!(u16_at(head, 18), 1000);
    let metrics = usize::from(u16_at(table(AHEM, *b"hhea"), 34));
    assert!(metrics > 0);
    assert_eq!(u16_at(table(AHEM, *b"hmtx"), 35.min(metrics - 1) * 4), 1000);
    let loca = table(AHEM, *b"loca");
    let glyph_at = match u16_at(head, 50) {
        0 => usize::from(u16_at(loca, 35 * 2)) * 2,
        1 => usize::try_from(u32_at(loca, 35 * 4)).unwrap(),
        format => panic!("unexpected retained Ahem loca format{format}"),
    };
    let glyph = &table(AHEM, *b"glyf")[glyph_at..];
    let bounds = [2, 4, 6, 8].map(|at| i16::from_be_bytes(glyph[at..at + 2].try_into().unwrap()));
    // Literal licensed Ahem A contour facts, also recorded in its provenance.
    assert_eq!(bounds, [0, -200, 1000, 800]);
}

fn captured_pair(system: &mut text::System) -> text::PreparedText {
    use text::SourceProcessingEvent as E;
    let source = text::Source::identified(text::Text::from("AA"), text::Id::from_u64(640725), 1);
    let base = text::Style {
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        wrap: text::Wrap::Word,
        ..style()
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: [(0, 1), (1, 2)]
                .into_iter()
                .map(|(start, end)| text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: text::Range::new(start, end),
                    aggregate: text::Range::new(start, end),
                    kind: text::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![text::SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: text::ParagraphConnectionKind::Continuous,
            }],
            controls: vec![
                text::ParagraphControl {
                    at: 1,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: 2,
                    kind: text::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: vec![text::ParagraphBoundaryPolicy {
                definition: text::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: text::Wrap::Word,
                following_wrap: text::Wrap::Word,
                ancestor_wrap: text::Wrap::Word,
            }],
            replacements: vec![],
            scopes: vec![
                text::SourceProcessingScope::Block {
                    font_reference: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: text::WhiteSpaceTrim::default(),
                    base_style: base.clone(),
                    used_direction: text::ParagraphBaseDirection::LeftToRight,
                },
                text::SourceProcessingScope::Inline {
                    alignment_boundary: None,
                    font_reference: None,
                    trim: text::WhiteSpaceTrim::default(),
                    style: Some(base),
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
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
                E::Origin {
                    origin: 1,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &text::Text::from("AA"));
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: text::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.control_views().count(), 2);
    assert!(
        prepared
            .control_views()
            .all(|control| control.capture_position().is_some()
                && control.handle().captured_control().is_some())
    );
    prepared
}

fn endpoint_width(
    control: &text::PreparedControl,
    opening: f64,
) -> Option<text::InlineBoundaryAllocation> {
    match control.view().kind() {
        text::ParagraphControlKind::InlineBoundaryStart => {
            Some(text::InlineBoundaryAllocation::try_new(opening).unwrap())
        }
        text::ParagraphControlKind::InlineBoundaryEnd => Some(text::InlineBoundaryAllocation::ZERO),
        _ => None,
    }
}

fn placed_pair(prepared: &text::PreparedText, opening: f64) -> text::Layout {
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let participants = prepared.participants().collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            text::PreparedParticipant::Control(occurrence) => Some(text::ControlAllocation {
                occurrence: occurrence.clone(),
                strut: None,
                boundary: endpoint_width(occurrence, opening),
            }),
            _ => None,
        })
        .collect();
    let mut request = text::LineSelectionRequest {
        line_box_struts: Vec::new(),
        start: prepared.boundary_views().next().unwrap().handle(),
        end: end.view().boundary(),
        termination: text::PreparedTermination::Ordinary(end),
        participants,
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_strut: None,
        context: text::LineContext {
            alignment_insets: surgeist_text::LineAlignmentInsets::ZERO,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            line_box_direction: surgeist_text::LineBoxDirection::LeftToRight,
            formatted_line: text::FormattedLineContext::default(),
            orientation: text::LineOrientation::Horizontal,
            start: text::LineStart::Paragraph,
            end: text::LineEnd::Paragraph,
            available_inline: Some(32.0),
            indent: 0.0,
            alignment: text::Alignment::Start,
            tabs: text::TabContext::NoTabs,
            spacing: text::SpacingPolicy::None,
        },
    };
    while let Some(occurrence) = prepared.boundary_context_demand(&request).unwrap() {
        let boundary = endpoint_width(&occurrence, opening)
            .expect("only actual endpoint geometry is demanded");
        request
            .boundary_context
            .push(text::InlineBoundaryContextAllocation {
                occurrence,
                boundary,
            });
    }
    assert!(
        request.boundary_context.is_empty(),
        "all genuine controls are selected in this whole paragraph"
    );
    let line = prepared
        .select(request)
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 20.0 + opening);
    let slots = line.control_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].inline_bounds(), (10.0, 10.0 + opening));
    assert_eq!(slots[1].inline_bounds(), (20.0 + opening, 20.0 + opening));
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared: prepared.clone(),
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 0.0,
                baseline: BASELINE,
            },
            orientation: text::LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source().text(), &text::Text::from("AA"));
    assert_eq!(layout.metrics().full_width(), 20.0 + opening);
    let before = layout
        .positions()
        .find(|position| position.source_cursor() == text::Cursor::new(0, text::Affinity::After))
        .unwrap();
    let after = layout
        .positions()
        .find(|position| position.source_cursor() == text::Cursor::new(2, text::Affinity::Before))
        .unwrap();
    let selection = layout.position_selection(&before, &after).unwrap();
    assert_eq!(selection.source_text(), &text::Text::from("AA"));
    assert_eq!(selection.plaintext(), &text::Text::from("AA"));
    let mut glyphs = Vec::new();
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.face().index(), 0);
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 10.0);
        assert_eq!(
            instance
                .metrics(run.face().catalogue_identity())
                .unwrap()
                .units_per_em(),
            1000
        );
        for glyph in run.glyphs() {
            assert_eq!(
                instance
                    .glyph_outline(glyph.id(), run.face().catalogue_identity())
                    .unwrap()
                    .bounds(),
                Some(text::Rect::new(0.0, -8.0, 10.0, 10.0))
            );
            glyphs.push((
                glyph.id(),
                run.origin().x + glyph.pen() + f64::from(glyph.x()),
                run.origin().y + f64::from(glyph.y()),
                glyph.advance(),
            ));
        }
    }
    assert_eq!(
        glyphs,
        [
            (35, 0.0, BASELINE, 10.0),
            (35, 10.0 + opening, BASELINE, 10.0)
        ]
    );
    layout
}

fn pixel(pixels: &[u8], x: usize, y: usize) -> &[u8] {
    let at = (y * SIDE + x) * 4;
    &pixels[at..at + 4]
}

#[test]
fn positive_inline_endpoint_displaces_only_the_second_real_ahem_cell_without_paint() {
    qualify_ahem();
    let mut system = system(AHEM.to_vec());
    let prepared = captured_pair(&mut system);
    let layouts = [placed_pair(&prepared, 0.0), placed_pair(&prepared, 5.0)];
    let publication = layouts[0]
        .run_views()
        .next()
        .unwrap()
        .face()
        .catalogue_identity()
        .clone();
    system.refresh_fonts().unwrap();
    assert_ne!(
        &publication,
        system
            .registered_font_faces()
            .next()
            .unwrap()
            .catalogue_identity()
    );
    drop(prepared);
    drop(system);

    let mut renderer = pollster::block_on(Renderer::new(Options::default()))
        .expect("endpoint paint requires a real GPU adapter");
    assert!(renderer.default_wgpu_device_queue().is_some());
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(32.0, 32.0), 1.0)).unwrap();
    let mut frames = Vec::new();
    for (layout, opening) in layouts.iter().zip([0.0, 5.0]) {
        assert!(
            layout
                .run_views()
                .all(|run| run.face().catalogue_identity() == &publication)
        );
        let prepared = prepare_text_layout(layout, Transform::identity(), limits()).unwrap();
        let mut glyphs = Vec::new();
        for command in &prepared.scene().commands {
            // A control interaction fragment is not endpoint/background paint.
            let (run, transform) = outline(command);
            assert_eq!(run.font().bytes(), AHEM);
            assert_eq!(run.font().index(), 0);
            assert_eq!(run.instance().size(), 10.0);
            assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
            assert!(run.path_elements().count() > 0);
            let [a, b, c, d, tx, ty] = transform.as_array();
            assert_eq!([a, b, c, d], [1.0, 0.0, 0.0, 1.0]);
            for glyph in run.glyphs() {
                glyphs.push((glyph.id(), tx + glyph.x(), ty + glyph.y(), glyph.advance()));
            }
        }
        assert_eq!(
            glyphs,
            [
                (35, 0.0, BASELINE, 10.0),
                (35, 10.0 + opening, BASELINE, 10.0)
            ]
        );
        let stats = pollster::block_on(renderer.render(
            &mut surface,
            prepared.scene(),
            Parameters::default(),
        ))
        .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::DirectVello));
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_eq!((image.size().width(), image.size().height()), (32, 32));
        frames.push(image.into_rgba());
    }
    assert!(renderer.default_device_has_no_terminal_signal_for_test());
    let [zero, positive] = frames.as_slice() else {
        unreachable!()
    };
    let painted = |frame: &[u8]| frame.chunks_exact(4).filter(|rgba| rgba[3] > 0).count();
    assert!(painted(zero) > 0 && painted(positive) > 0);
    assert_eq!(
        painted(zero),
        painted(positive),
        "displacement preserves actual painted area"
    );
    for (frame, second_start) in [(zero, 10), (positive, 15)] {
        assert!(
            (0..SIDE).any(|y| (0..10).any(|x| pixel(frame, x, y)[3] > 0)),
            "first real cell paints"
        );
        assert!(
            (0..SIDE).any(|y| (second_start..second_start + 10).any(|x| pixel(frame, x, y)[3] > 0)),
            "second real cell paints"
        );
    }
    // Licensed A bounds at size10 and declared baseline12 give rows[4,14).
    // Integer5px displacement preserves raster samples exactly; outside the
    // two source outline cells remains transparent, including the new gap.
    for y in 0..SIDE {
        for x in 0..SIDE {
            if x < 10 {
                assert_eq!(pixel(positive, x, y), pixel(zero, x, y));
            } else if x < 15 {
                assert_eq!(pixel(positive, x, y)[3], 0, "endpoint gap pixel({x},{y})");
            } else {
                assert_eq!(pixel(positive, x, y), pixel(zero, x - 5, y));
            }
            if !(4..14).contains(&y) || x >= 20 {
                assert_eq!(pixel(zero, x, y)[3], 0);
            }
            if !(4..14).contains(&y) || x >= 25 {
                assert_eq!(pixel(positive, x, y)[3], 0);
            }
        }
    }
}
