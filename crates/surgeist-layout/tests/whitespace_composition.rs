#![forbid(unsafe_code)]

//! Public native composition of the existing whitespace operations.

use std::{convert::Infallible, sync::Arc};
use surgeist_layout as layout;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

// Consumer-owned cmap/metrics over the unchanged licensed Ahem outlines. Hebrew
// letters and Ogham use genuine existing glyphs; full-width X and U3000 have
// distinct real 2000-unit advances. No glyph output or measurement is mocked.
fn font() -> Vec<u8> {
    fn u16_at(bytes: &[u8], at: usize) -> u16 {
        u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
    }
    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
    }
    fn checksum(bytes: &[u8]) -> u32 {
        bytes.chunks(4).fold(0_u32, |sum, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    }
    fn record(bytes: &[u8], tag: &[u8; 4]) -> usize {
        (0..usize::from(u16_at(bytes, 4)))
            .map(|index| 12 + index * 16)
            .find(|at| &bytes[*at..*at + 4] == tag)
            .unwrap()
    }
    let mappings = [
        (0x20_u32, 3_u32),
        (0x58, 58),
        (0x59, 59),
        (0xa0, 153),
        (0x5d0, 58),
        (0x5d1, 59),
        (0x1680, 3),
        (0x3000, 1),
        (0xff38, 36),
    ];
    let mut cmap = Vec::from(&[0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12][..]);
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    cmap.extend(
        u32::try_from(16 + mappings.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(u32::try_from(mappings.len()).unwrap().to_be_bytes());
    for (scalar, glyph) in mappings {
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let mut bytes = AHEM.to_vec();
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let at = u32::try_from(bytes.len()).unwrap();
    let entry = record(&bytes, b"cmap");
    bytes[entry + 4..entry + 8].copy_from_slice(&checksum(&cmap).to_be_bytes());
    bytes[entry + 8..entry + 12].copy_from_slice(&at.to_be_bytes());
    bytes[entry + 12..entry + 16]
        .copy_from_slice(&u32::try_from(cmap.len()).unwrap().to_be_bytes());
    bytes.extend(cmap);
    let entry = record(&bytes, b"hmtx");
    let at = usize::try_from(u32_at(&bytes, entry + 8)).unwrap();
    let length = usize::try_from(u32_at(&bytes, entry + 12)).unwrap();
    for glyph in [1, 36] {
        bytes[at + glyph * 4..at + glyph * 4 + 2].copy_from_slice(&2000_u16.to_be_bytes());
    }
    let sum = checksum(&bytes[at..at + length]);
    bytes[entry + 4..entry + 8].copy_from_slice(&sum.to_be_bytes());
    let entry = record(&bytes, b"head");
    let at = usize::try_from(u32_at(&bytes, entry + 8)).unwrap();
    let length = usize::try_from(u32_at(&bytes, entry + 12)).unwrap();
    assert_eq!(u16_at(&bytes, at + 18), 1000);
    bytes[at + 8..at + 12].fill(0);
    let sum = checksum(&bytes[at..at + length]);
    bytes[entry + 4..entry + 8].copy_from_slice(&sum.to_be_bytes());
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    bytes[at + 8..at + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    bytes
}

struct Tree {
    inputs: Vec<layout::LayoutInput>,
    non_box: layout::NodeInput,
}

impl layout::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::ops::Range<usize>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 {
            1..self.inputs.len()
        } else {
            0..0
        }
    }
    fn child_count(&self, node: usize) -> usize {
        self.children(node).len()
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl layout::LayoutTree for Tree {
    type MeasureError = Infallible;

    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

struct Case<'a> {
    value: &'a str,
    white_space: text::WhiteSpace,
    wrap: text::Wrap,
    rtl: bool,
    forced_at: Option<usize>,
    trim: bool,
    full_width: bool,
}

impl<'a> Case<'a> {
    fn new(value: &'a str, white_space: text::WhiteSpace) -> Self {
        Self {
            value,
            white_space,
            wrap: text::Wrap::Word,
            rtl: false,
            forced_at: None,
            trim: false,
            full_width: false,
        }
    }
}

struct Fixture {
    prepared: text::PreparedText,
    tree: Tree,
    binding: layout::TextParagraphBinding<usize>,
    bytes: Vec<u8>,
}

fn fixture(case: Case<'_>) -> Fixture {
    let direction = if case.rtl {
        text::ParagraphBaseDirection::RightToLeft
    } else {
        text::ParagraphBaseDirection::LeftToRight
    };
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: case.white_space,
        wrap: case.wrap,
        text_transform: text::TextTransform::Set(
            text::TextTransformSet::none().with_full_width(case.full_width),
        ),
        ..text::Style::default()
    };
    let source = text::Source::new(case.value);
    let end = source.text().len();
    let ranges = case.forced_at.map_or_else(
        || vec![text::Range::new(0, end)],
        |at| vec![text::Range::new(0, at), text::Range::new(at, end)],
    );
    let origins: Vec<_> = ranges
        .into_iter()
        .map(|range| text::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: text::ParagraphOriginKind::Authored,
        })
        .collect();
    let controls: Vec<_> = case
        .forced_at
        .into_iter()
        .map(|at| text::ParagraphControl {
            at,
            kind: text::ParagraphControlKind::ForcedBreak,
        })
        .collect();
    let analyzed = if case.trim {
        assert!(controls.is_empty());
        use text::SourceProcessingEvent as E;
        let processed = Arc::new(
            text::process_contextual_source(text::ContextualSourceInput {
                scope_alignment_limits: None,
                computed_line_boundaries: vec![],
                source,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                envelope: text::SourceProcessingEnvelope::Complete,
                origins,
                connections: vec![],
                controls,
                boundary_policies: vec![],
                replacements: vec![],
                scopes: vec![
                    text::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        font_reference: None,
                        topology: text::SourceProcessingTopology::Unknown,
                        trim: text::WhiteSpaceTrim::default(),
                        base_style: style,
                        used_direction: direction,
                    },
                    text::SourceProcessingScope::Inline {
                        horizontal_inner_direction: None,
                        alignment_boundary: None,
                        font_reference: None,
                        style: None,
                        boundaries: None,
                        trim: text::WhiteSpaceTrim {
                            discard_inner: true,
                            ..Default::default()
                        },
                    },
                ],
                paragraphs: vec![text::SourceProcessingParagraph {
                    base_direction: direction,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                }],
                events: vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::EnterScope(1),
                    E::Origin {
                        origin: 0,
                        affinity: text::Affinity::After,
                    },
                    E::ExitScope(1),
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        text::AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap()
    } else {
        text::AnalyzedSource::from_paragraph(
            text::ParagraphInput {
                terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
                source,
                base_style: style,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: direction,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
                tab_reference: None,
                origins,
                connections: case
                    .forced_at
                    .map(|at| text::ParagraphConnection {
                        at,
                        kind: text::ParagraphConnectionKind::Continuous,
                    })
                    .into_iter()
                    .collect(),
                controls,
                boundary_policies: case
                    .forced_at
                    .map(|_| text::ParagraphBoundaryPolicy {
                        definition: text::ParagraphBoundaryDefinition::Connection(0),
                        preceding_wrap: case.wrap,
                        following_wrap: case.wrap,
                        ancestor_wrap: case.wrap,
                    })
                    .into_iter()
                    .collect(),
                replacements: vec![],
                anchor_order: case
                    .forced_at
                    .map(|_| text::ParagraphAnchor {
                        item: text::ParagraphAnchorItem::Control(0),
                        affinity: text::Affinity::Before,
                    })
                    .into_iter()
                    .collect(),
            },
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap()
    };
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: case.wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let bytes = font();
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    bind(prepared, case.rtl, bytes)
}

fn bind(prepared: text::PreparedText, rtl: bool, bytes: Vec<u8>) -> Fixture {
    let mut inputs = vec![layout::LayoutInput::box_input(layout::NodeInput {
        display: layout::Display::Block,
        direction: if rtl {
            layout::Direction::Rtl
        } else {
            layout::Direction::Ltr
        },
        ..layout::NodeInput::DEFAULT
    })];
    let mut origins = Vec::new();
    let mut controls = Vec::new();
    for (index, origin) in prepared.origin_views().enumerate() {
        if index == 1 && prepared.control_views().len() == 1 {
            let occurrence = prepared.control_views().next().unwrap().handle();
            controls.push(layout::TextControlBinding {
                occurrence: occurrence.clone(),
                node: inputs.len(),
            });
            inputs.push(layout::LayoutInput::text_control(
                layout::TextControlParticipation::new(occurrence),
            ));
        }
        let origin = origin.handle();
        origins.push(layout::TextOriginBinding {
            origin: origin.clone(),
            node: inputs.len(),
        });
        inputs.push(layout::LayoutInput::inline_text(
            layout::TextParticipation::new(origin),
        ));
    }
    let binding = layout::TextParagraphBinding {
        prepared: prepared.clone(),
        container: 0,
        nodes: (1..inputs.len()).collect(),
        origins,
        controls,
        atomics: vec![],
        line_strut: None,
    };
    Fixture {
        prepared,
        tree: Tree {
            inputs,
            non_box: layout::NodeInput::non_box(),
        },
        binding,
        bytes,
    }
}

fn compute(fixture: &Fixture, width: layout::Available) -> layout::CompletedLayoutBatch<usize> {
    let capture = layout::LayoutFlowInput::try_new(
        &fixture.tree,
        0,
        vec![fixture.binding.clone()],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    layout::compute_layout_with_flow(
        &capture,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            width,
            layout::Available::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}

fn finalized(batch: &layout::CompletedLayoutBatch<usize>) -> Vec<text::FinalizedLine> {
    let mut result = Vec::new();
    for fragment in batch.final_inline_fragments() {
        let line = fragment.fragment().finalized_fragment().finalized_line();
        if !result.contains(&line) {
            result.push(line);
        }
    }
    result
}

fn source_and_font(fixture: &Fixture, batch: &layout::CompletedLayoutBatch<usize>, original: &str) {
    assert_eq!(batch.placed_text().len(), 1);
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        original.encode_utf16().collect::<Vec<_>>()
    );
    for run in batch.placed_text()[0].layout().run_views() {
        assert_eq!(run.face().bytes(), fixture.bytes);
        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
    }
}

fn root_width(batch: &layout::CompletedLayoutBatch<usize>) -> f32 {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output()
        .size
        .width
}

#[test]
fn preserve_spaces_conversion_soft_hanging_and_intrinsics_survive_bidi_placement() {
    for (value, rtl, first_x) in [("X\t\nY", false, 0.0), ("\u{5d0}\t\n\u{5d1}", true, 5.0)] {
        let mut case = Case::new(value, text::WhiteSpace::PreserveSpaces);
        case.rtl = rtl;
        let fixture = fixture(case);
        let processed = fixture.prepared.analyzed_source();
        let expected = if rtl { "\u{5d0}  \u{5d1}" } else { "X  Y" };
        assert_eq!(
            processed.processed_source().processed_text().code_units(),
            expected.encode_utf16().collect::<Vec<_>>()
        );
        let batch = compute(&fixture, layout::Available::definite(15.0));
        source_and_font(&fixture, &batch, value);
        let lines = finalized(&batch);
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0].selected_line().request().context.end,
            text::LineEnd::Soft
        );
        assert_eq!(
            lines[1].selected_line().request().context.end,
            text::LineEnd::Paragraph
        );
        assert_eq!(lines[0].feedback().advance, 30.0);
        assert_eq!(
            (
                lines[0].feedback().hanging_start,
                lines[0].feedback().hanging_end
            ),
            if rtl { (20.0, 0.0) } else { (0.0, 20.0) },
        );
        assert_eq!(
            (
                lines[0].measurements().fit().advance(),
                lines[0].measurements().alignment().advance()
            ),
            (10.0, 10.0)
        );
        assert_eq!(lines[1].feedback().advance, 10.0);
        let placed = batch.placed_text()[0].layout();
        let first: Vec<_> = placed
            .run_views()
            .flat_map(|run| {
                let line = run.line();
                let origin = run.origin().x;
                run.glyphs()
                    .filter(|glyph| glyph.id() == 58)
                    .map(move |glyph| (line, origin + glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(first, [(0, first_x)]);
        assert_eq!(
            root_width(&compute(&fixture, layout::Available::MIN_CONTENT)),
            10.0
        );
        assert_eq!(
            root_width(&compute(&fixture, layout::Available::MAX_CONTENT)),
            40.0
        );
    }
}

#[test]
fn preserve_spaces_block_end_is_conditional_and_nowrap_keeps_ordinary_measures() {
    for wrap in [text::Wrap::Word, text::Wrap::None] {
        let mut case = Case::new("X\t", text::WhiteSpace::PreserveSpaces);
        case.wrap = wrap;
        let fixture = fixture(case);
        for width in [15.0, 40.0] {
            let batch = compute(&fixture, layout::Available::definite(width));
            source_and_font(&fixture, &batch, "X\t");
            let lines = finalized(&batch);
            assert_eq!(lines.len(), 1);
            let line = &lines[0];
            assert_eq!(
                line.selected_line().request().context.end,
                text::LineEnd::Paragraph
            );
            let wraps = wrap == text::Wrap::Word;
            assert_eq!(line.feedback().advance, 20.0);
            assert_eq!(
                line.measurements().fit().advance(),
                if wraps { 10.0 } else { 20.0 }
            );
            assert_eq!(
                line.measurements().alignment().advance(),
                if wraps && width == 15.0 { 15.0 } else { 20.0 }
            );
            assert_eq!(
                line.feedback().hanging_end,
                if wraps && width == 15.0 { 5.0 } else { 0.0 }
            );
            let range = batch.placed_text()[0]
                .layout()
                .source_range_fragments(&fixture.binding.origins[0].origin, text::Range::new(1, 2))
                .unwrap();
            let pieces: Vec<_> = range
                .units()
                .flat_map(|unit| {
                    unit.pieces()
                        .map(|piece| piece.advance_edges())
                        .collect::<Vec<_>>()
                })
                .collect();
            assert_eq!(pieces, [(10.0, 20.0)]);
        }
        assert_eq!(
            root_width(&compute(&fixture, layout::Available::MIN_CONTENT)),
            if wrap == text::Wrap::Word { 10.0 } else { 20.0 }
        );
        assert_eq!(
            root_width(&compute(&fixture, layout::Available::MAX_CONTENT)),
            20.0
        );
    }
}

#[test]
fn preserve_spaces_converted_tab_before_a_real_forced_control_hangs_conditionally() {
    let mut case = Case::new("X\tY", text::WhiteSpace::PreserveSpaces);
    case.forced_at = Some(2);
    let fixture = fixture(case);
    for (width, hanging) in [(15.0, 5.0), (40.0, 0.0)] {
        let batch = compute(&fixture, layout::Available::definite(width));
        source_and_font(&fixture, &batch, "X\tY");
        let lines = finalized(&batch);
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0].selected_line().request().context.end,
            text::LineEnd::Forced
        );
        assert_eq!(
            (lines[0].feedback().advance, lines[0].feedback().hanging_end),
            (20.0, hanging)
        );
        assert_eq!(lines[0].measurements().fit().advance(), 10.0);
        assert_eq!(lines[1].feedback().advance, 10.0);
    }
    assert_eq!(
        root_width(&compute(&fixture, layout::Available::MIN_CONTENT)),
        10.0
    );
    assert_eq!(
        root_width(&compute(&fixture, layout::Available::MAX_CONTENT)),
        20.0
    );
}

#[test]
fn discard_removes_document_whitespace_breaks_but_keeps_independent_zwsp_and_forced_control() {
    for (value, forced_at, ending, advances) in [
        ("X \t\r\n Y", None, text::LineEnd::Paragraph, vec![20.0]),
        (
            "X \t\u{200b}\n Y",
            None,
            text::LineEnd::Soft,
            vec![10.0, 10.0],
        ),
        ("X \tY", Some(3), text::LineEnd::Forced, vec![10.0, 10.0]),
    ] {
        let mut case = Case::new(value, text::WhiteSpace::Discard);
        case.forced_at = forced_at;
        let fixture = fixture(case);
        let offers: Vec<_> = fixture
            .prepared
            .opportunity_views()
            .map(|offer| offer.kind())
            .collect();
        // Removed originals can retain distinct source-custody receipts for
        // the same processed cut. The only semantic soft cut here belongs to
        // the independent ZWSP, never the discarded document whitespace.
        let soft_positions: std::collections::BTreeSet<_> = fixture
            .prepared
            .opportunity_views()
            .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
            .map(|offer| offer.boundary().view().processed_position().get())
            .collect();
        assert_eq!(
            soft_positions,
            if ending == text::LineEnd::Soft {
                [2].into_iter().collect()
            } else {
                Default::default()
            }
        );
        assert_eq!(
            offers
                .iter()
                .filter(|kind| **kind == text::PreparedBreakKind::Forced)
                .count(),
            usize::from(ending == text::LineEnd::Forced)
        );
        let batch = compute(&fixture, layout::Available::definite(10.0));
        source_and_font(&fixture, &batch, value);
        let lines = finalized(&batch);
        assert_eq!(
            lines
                .iter()
                .map(|line| line.feedback().advance)
                .collect::<Vec<_>>(),
            advances
        );
        assert_eq!(lines[0].selected_line().request().context.end, ending);
        assert!(lines.iter().all(|line| line.feedback().hanging_end == 0.0));
        assert_eq!(
            root_width(&compute(&fixture, layout::Available::MIN_CONTENT)),
            if ending == text::LineEnd::Paragraph {
                20.0
            } else {
                10.0
            }
        );
        assert_eq!(
            root_width(&compute(&fixture, layout::Available::MAX_CONTENT)),
            if ending == text::LineEnd::Forced {
                10.0
            } else {
                20.0
            }
        );
    }
}

#[test]
fn collapse_removes_only_trailing_ogham_and_retains_its_original_range() {
    let fixture = fixture(Case::new("\u{1680}X\u{1680}", text::WhiteSpace::Collapse));
    // Both Ogham occurrences shape to the registered font's real 10px glyph
    // before selected-line Phase II removes only the trailing one.
    assert_eq!(
        fixture
            .prepared
            .unit_views()
            .map(|unit| (unit.source_range(), unit.advance()))
            .collect::<Vec<_>>(),
        [
            (text::Range::new(0, 1), 10.0),
            (text::Range::new(1, 2), 10.0),
            (text::Range::new(2, 3), 10.0)
        ],
    );
    let batch = compute(&fixture, layout::Available::definite(40.0));
    source_and_font(&fixture, &batch, "\u{1680}X\u{1680}");
    let lines = finalized(&batch);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].feedback().advance, 20.0);
    let placed = batch.placed_text()[0].layout();
    for (range, expected) in [
        (text::Range::new(0, 1), vec![(0.0, 10.0)]),
        // Removed trailing Ogham has source custody, but no placed glyph unit.
        (text::Range::new(2, 3), vec![]),
    ] {
        let observed = placed
            .source_range_fragments(&fixture.binding.origins[0].origin, range)
            .unwrap();
        let pieces: Vec<_> = observed
            .units()
            .flat_map(|unit| {
                unit.pieces()
                    .map(|piece| piece.advance_edges())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(pieces, expected);
    }
}

#[test]
fn contextual_trim_precedes_preserve_spaces_conversion_and_full_width_presentation() {
    // Min-content can break after the leading converted SPACE; its largest
    // remaining unit is the actual 20px full-width X.
    for (trim, processed, raw, minimum) in [(false, " X ", 60.0, 20.0), (true, "X", 20.0, 20.0)] {
        let mut case = Case::new("\tX\t", text::WhiteSpace::PreserveSpaces);
        case.trim = trim;
        case.full_width = true;
        let fixture = fixture(case);
        assert_eq!(
            fixture
                .prepared
                .analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            processed.encode_utf16().collect::<Vec<_>>()
        );
        let batch = compute(&fixture, layout::Available::definite(100.0));
        source_and_font(&fixture, &batch, "\tX\t");
        let lines = finalized(&batch);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].feedback().advance, raw);
        let glyphs: Vec<_> = batch.placed_text()[0]
            .layout()
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            glyphs,
            if trim {
                vec![(36, 20.0)]
            } else {
                vec![(1, 20.0), (36, 20.0), (1, 20.0)]
            }
        );
        assert_eq!(
            root_width(&compute(&fixture, layout::Available::MIN_CONTENT)),
            minimum
        );
        assert_eq!(
            root_width(&compute(&fixture, layout::Available::MAX_CONTENT)),
            raw as f32
        );
    }
}

#[test]
fn collapsed_sibling_space_permission_controls_actual_narrow_layout() {
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for (first, second, cut) in [
        (text::Wrap::None, text::Wrap::Word, 3),
        (text::Wrap::Word, text::Wrap::None, 2),
    ] {
        for ancestor in [text::Wrap::Word, text::Wrap::None] {
            let style = |wrap| text::Style {
                font: text::Font::new().family("Ahem"),
                size: 10.0,
                line_height: text::LineHeight::Absolute(10.0),
                white_space: text::WhiteSpace::Collapse,
                wrap,
                ..text::Style::default()
            };
            let mut source = text::Source::new("X  Y");
            source.span(text::Range::new(0, 2), style(first));
            source.span(text::Range::new(2, 4), style(second));
            let origin = |start, end| text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::Range::new(start, end),
                aggregate: text::Range::new(start, end),
                kind: text::ParagraphOriginKind::Authored,
            };
            use text::SourceProcessingEvent as E;
            let processed = Arc::new(
                text::process_contextual_source(text::ContextualSourceInput {
                    scope_alignment_limits: None,
                    computed_line_boundaries: vec![],
                    source: source.clone(),
                    segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                    envelope: text::SourceProcessingEnvelope::Complete,
                    origins: vec![origin(0, 2), origin(2, 4)],
                    connections: vec![text::SourceProcessingConnection {
                        before_origin: 0,
                        after_origin: 1,
                        kind: text::ParagraphConnectionKind::Continuous,
                    }],
                    controls: vec![],
                    boundary_policies: vec![text::ParagraphBoundaryPolicy {
                        definition: text::ParagraphBoundaryDefinition::Connection(0),
                        preceding_wrap: first,
                        following_wrap: second,
                        ancestor_wrap: ancestor,
                    }],
                    replacements: vec![],
                    scopes: vec![
                        text::SourceProcessingScope::Block {
                            writing_axis: None,
                            horizontal_inner_direction: None,
                            font_reference: None,
                            topology: text::SourceProcessingTopology::Unknown,
                            trim: text::WhiteSpaceTrim::default(),
                            base_style: style(ancestor),
                            used_direction: text::ParagraphBaseDirection::LeftToRight,
                        },
                        text::SourceProcessingScope::Inline {
                            horizontal_inner_direction: None,
                            alignment_boundary: None,
                            font_reference: None,
                            trim: text::WhiteSpaceTrim::default(),
                            style: Some(style(first)),
                            boundaries: None,
                        },
                        text::SourceProcessingScope::Inline {
                            horizontal_inner_direction: None,
                            alignment_boundary: None,
                            font_reference: None,
                            trim: text::WhiteSpaceTrim::default(),
                            style: Some(style(second)),
                            boundaries: None,
                        },
                    ],
                    paragraphs: vec![text::SourceProcessingParagraph {
                        base_direction: text::ParagraphBaseDirection::LeftToRight,
                        paragraph_strut: text::ParagraphStrut::DefaultFont,
                    }],
                    events: vec![
                        E::EnterScope(0),
                        E::EnterParagraph(0),
                        E::EnterScope(1),
                        E::Origin {
                            origin: 0,
                            affinity: text::Affinity::After,
                        },
                        E::ExitScope(1),
                        E::Connection(0),
                        E::EnterScope(2),
                        E::Origin {
                            origin: 1,
                            affinity: text::Affinity::After,
                        },
                        E::ExitScope(2),
                        E::ExitParagraph(0),
                        E::ExitScope(0),
                    ],
                })
                .unwrap(),
            );
            assert_eq!(processed.processed_text().code_units(), [88, 32, 89]);
            let analyzed = text::AnalyzedSource::from_processed_paragraph(
                processed.paragraphs().next().unwrap(),
                text::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let bytes = font();
            let mut system = text::System::new(text::SystemOptions {
                font_catalogue: text::FontCatalogueMode::RegisteredOnly,
            })
            .unwrap();
            system.register_fonts(bytes.clone()).unwrap();
            let prepared = system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap();
            assert_eq!(
                prepared
                    .unit_views()
                    .map(|unit| (unit.source_range(), unit.advance()))
                    .collect::<Vec<_>>(),
                [
                    (text::Range::new(0, 1), 10.0),
                    (text::Range::new(1, 2), 10.0),
                    (text::Range::new(2, 3), 0.0),
                    (text::Range::new(3, 4), 10.0),
                ]
            );
            let cuts: Vec<_> = prepared
                .opportunity_views()
                .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
                .map(|offer| offer.boundary().view().source_after().get())
                .collect();
            let fixture = bind(prepared, false, bytes);
            let batch = compute(&fixture, layout::Available::definite(10.0));
            source_and_font(&fixture, &batch, "X  Y");
            let advances: Vec<_> = finalized(&batch)
                .iter()
                .map(|line| line.feedback().advance)
                .collect();
            actual.push((first, second, ancestor, cuts, advances));
            // The disappearing SPACE owns its own local permission. The
            // common ancestor cannot donate or remove either SPACE's cut.
            expected.push((first, second, ancestor, vec![cut], vec![10.0, 10.0]));
        }
    }
    assert_eq!(actual, expected);
}
