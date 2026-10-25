#![forbid(unsafe_code)]

//! Source-owned collapsing opportunities and actual control/ignorable outline ink.

use std::sync::Arc;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

fn style(wrap: text::Wrap) -> text::Style {
    text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        wrap,
        ..text::Style::default()
    }
}

fn system(bytes: &[u8]) -> text::System {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system
}

fn collapsed_space_cuts(first: text::Wrap, second: text::Wrap, ancestor: text::Wrap) -> Vec<usize> {
    let local = |wrap| text::Style {
        white_space: text::WhiteSpace::Collapse,
        ..style(wrap)
    };
    let mut source = text::Source::new("A  B");
    source.span(text::Range::new(0, 2), local(first));
    source.span(text::Range::new(2, 4), local(second));
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
            replacements: vec![],
            boundary_policies: vec![text::ParagraphBoundaryPolicy {
                definition: text::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: first,
                following_wrap: second,
                ancestor_wrap: ancestor,
            }],
            scopes: vec![
                text::SourceProcessingScope::Block {
                    topology: text::SourceProcessingTopology::Unknown,
                    trim: text::WhiteSpaceTrim::default(),
                    base_style: local(ancestor),
                    used_direction: text::ParagraphBaseDirection::LeftToRight,
                },
                text::SourceProcessingScope::Inline {
                    trim: text::WhiteSpaceTrim::default(),
                    style: Some(local(first)),
                    boundaries: None,
                },
                text::SourceProcessingScope::Inline {
                    trim: text::WhiteSpaceTrim::default(),
                    style: Some(local(second)),
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
    assert_eq!(processed.source().text().code_units(), [65, 32, 32, 66]);
    assert_eq!(processed.processed_text().code_units(), [65, 32, 66]);
    let paragraph = processed.paragraphs().next().unwrap();
    assert_eq!(
        paragraph.source_pieces().collect::<Vec<_>>(),
        [text::Range::new(0, 2), text::Range::new(2, 4)]
    );
    let analyzed = text::AnalyzedSource::from_processed_paragraph(paragraph).unwrap();
    let prepared = system(AHEM)
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        [65, 32, 32, 66]
    );
    // Real Ahem cells advance10px; the collapsed second SPACE retains its own
    // original unit/cuts while contributing zero advance after Phase I.
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
    prepared
        .opportunity_views()
        .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
        .map(|offer| {
            let boundary = offer.boundary();
            assert_eq!(
                boundary.view().source_before(),
                boundary.view().source_after()
            );
            boundary.view().source_after().get()
        })
        .collect()
}

fn both_ancestors(first: text::Wrap, second: text::Wrap) -> Vec<(text::Wrap, Vec<usize>)> {
    [text::Wrap::Word, text::Wrap::None]
        .into_iter()
        .map(|ancestor| (ancestor, collapsed_space_cuts(first, second, ancestor)))
        .collect()
}

#[test]
fn collapsed_following_space_keeps_its_own_word_wrap_permission() {
    // Disappearing-character ownership is local to the second genuine inline:
    // its cut3 is permitted even when the surviving first SPACE cannot wrap.
    assert_eq!(
        both_ancestors(text::Wrap::None, text::Wrap::Word),
        [(text::Wrap::Word, vec![3]), (text::Wrap::None, vec![3])]
    );
}

#[test]
fn collapsed_following_nowrap_space_cannot_borrow_the_surviving_spaces_permission() {
    // Only the first SPACE's own cut2 may wrap. Its permission cannot donate
    // cut3 to the collapsed second SPACE in a different nowrap inline.
    assert_eq!(
        both_ancestors(text::Wrap::Word, text::Wrap::None),
        [(text::Wrap::Word, vec![2]), (text::Wrap::None, vec![2])]
    );
}

#[test]
fn homogeneous_word_spaces_keep_both_original_cuts_under_either_nca_mode() {
    assert_eq!(
        both_ancestors(text::Wrap::Word, text::Wrap::Word),
        [
            (text::Wrap::Word, vec![2, 3]),
            (text::Wrap::None, vec![2, 3])
        ]
    );
}

#[test]
fn homogeneous_nowrap_spaces_offer_neither_original_cut_under_either_nca_mode() {
    assert_eq!(
        both_ancestors(text::Wrap::None, text::Wrap::None),
        [(text::Wrap::Word, vec![]), (text::Wrap::None, vec![])]
    );
}

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

// Minimal safe adaptation of src/test_fonts.rs's repo-owned sfnt rewriting.
// Only cmap and its required checksums change. The licensed Ahem outlines and
// metrics remain real and unchanged (fixtures/fonts/ahem/PROVENANCE.md).
fn remap(entries: &[(u32, u32)]) -> Vec<u8> {
    assert!(entries.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let mut cmap = Vec::new();
    for value in [0_u16, 1, 3, 10] {
        cmap.extend(value.to_be_bytes());
    }
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    cmap.extend(
        u32::try_from(16 + entries.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(u32::try_from(entries.len()).unwrap().to_be_bytes());
    for &(scalar, glyph) in entries {
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let mut tables: Vec<_> = (0..usize::from(u16_at(AHEM, 4)))
        .map(|index| {
            let record = 12 + index * 16;
            let tag: [u8; 4] = AHEM[record..record + 4].try_into().unwrap();
            let start = usize::try_from(u32_at(AHEM, record + 8)).unwrap();
            let length = usize::try_from(u32_at(AHEM, record + 12)).unwrap();
            (
                tag,
                if tag == *b"cmap" {
                    cmap.clone()
                } else {
                    AHEM[start..start + length].to_vec()
                },
            )
        })
        .collect();
    tables.sort_by_key(|table| table.0);
    let count = tables.len();
    let mut bytes = vec![0; 12 + count * 16];
    bytes[..4].copy_from_slice(&AHEM[..4]);
    bytes[4..6].copy_from_slice(&u16::try_from(count).unwrap().to_be_bytes());
    let power = count.next_power_of_two() / if count.is_power_of_two() { 1 } else { 2 };
    bytes[6..8].copy_from_slice(&u16::try_from(power * 16).unwrap().to_be_bytes());
    bytes[8..10].copy_from_slice(&u16::try_from(power.ilog2()).unwrap().to_be_bytes());
    bytes[10..12].copy_from_slice(&u16::try_from((count - power) * 16).unwrap().to_be_bytes());
    for (index, (tag, mut data)) in tables.into_iter().enumerate() {
        if tag == *b"head" {
            data[8..12].fill(0);
        }
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let record = 12 + index * 16;
        let offset = u32::try_from(bytes.len()).unwrap();
        bytes[record..record + 4].copy_from_slice(&tag);
        bytes[record + 4..record + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        bytes[record + 8..record + 12].copy_from_slice(&offset.to_be_bytes());
        bytes[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        bytes.extend(data);
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    let head = (0..count)
        .map(|index| 12 + index * 16)
        .find(|record| bytes[*record..*record + 4] == *b"head")
        .unwrap();
    let offset = usize::try_from(u32_at(&bytes, head + 8)).unwrap();
    bytes[offset + 8..offset + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    bytes
}

struct Rendered {
    ink: text::LineOutlineInk,
    glyphs: Vec<(u32, f32, text::Range)>,
    registered_faces: bool,
    end_caret_x: f64,
}

fn rendered(bytes: &[u8], value: &str) -> Rendered {
    let source = text::Source::new(value);
    let end = source.text().len();
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        source: source.clone(),
        base_style: style(text::Wrap::Word),
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![text::ParagraphOrigin {
            text_node: None,
            source,
            local: text::Range::new(0, end),
            aggregate: text::Range::new(0, end),
            kind: text::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    let prepared = system(bytes)
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let termination = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = prepared
        .select(text::LineSelectionRequest {
            boundary_context: Vec::new(),
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            termination: text::PreparedTermination::Ordinary(termination),
            alternative: None,
            participants: prepared.participants().collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
            context: text::LineContext {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                orientation: text::LineOrientation::Horizontal,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: Some(100.0),
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::Spaces,
            },
        })
        .unwrap()
        .finalize(text::SpacingAllocations::None)
        .unwrap();
    let registered_faces = line.run_views().all(|run| run.face().bytes() == bytes);
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance(), glyph.range()))
                .collect::<Vec<_>>()
        })
        .collect();
    let ink = line.outline_ink().unwrap();
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared,
        coverage: text::PlacementCoverage::WholeParagraph,
        lines: vec![text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 0.0,
                baseline: 8.0,
            },
            orientation: text::LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    let units: Vec<_> = value.encode_utf16().collect();
    assert_eq!(layout.source().text().code_units(), units);
    let end_caret_x = layout
        .cursor(text::Cursor::new(end, text::Affinity::Before))
        .unwrap()
        .rect()
        .origin
        .x;
    assert!(end_caret_x.is_finite());
    let positions: Vec<_> = layout.positions().collect();
    let before = positions
        .iter()
        .find(|position| position.source_cursor() == text::Cursor::new(0, text::Affinity::After))
        .unwrap();
    let after = positions
        .iter()
        .find(|position| position.source_cursor() == text::Cursor::new(end, text::Affinity::Before))
        .unwrap();
    let selection = layout.position_selection(before, after).unwrap();
    assert_eq!(selection.source_text().code_units(), units);
    assert_eq!(selection.plaintext().code_units(), units);
    Rendered {
        ink,
        glyphs,
        registered_faces,
        end_caret_x,
    }
}

fn positive_ink(ink: text::LineOutlineInk) {
    assert!(
        matches!(ink, text::LineOutlineInk::Bounds(bounds)
        if bounds.size.width > 0.0 && bounds.size.height > 0.0),
        "actual finalized outline must have positive ink: {ink:?}"
    );
}

fn qualified_font(control: Option<u32>, visible_space: bool) -> Vec<u8> {
    // The sole replacement Unicode cmap deliberately has NO U+2065 entry.
    // Existing glyph3 is blank SPACE; glyph58 is the licensed real X outline.
    let mut entries = Vec::new();
    if let Some(glyph) = control {
        entries.push((1, glyph));
    }
    entries.extend([(32, if visible_space { 58 } else { 3 }), (88, 58)]);
    let bytes = remap(&entries);
    let x = rendered(&bytes, "X");
    positive_ink(x.ink);
    assert!(x.registered_faces);
    assert_eq!(x.glyphs, [(58, 10.0, text::Range::new(0, 1))]);
    assert_eq!(x.end_caret_x, 10.0);
    let space = rendered(&bytes, " ");
    assert!(space.registered_faces);
    assert_eq!(
        space.glyphs,
        [(
            if visible_space { 58 } else { 3 },
            10.0,
            text::Range::new(0, 1)
        )]
    );
    assert_eq!(space.end_caret_x, 10.0);
    if visible_space {
        positive_ink(space.ink);
    } else {
        assert_eq!(space.ink, text::LineOutlineInk::Empty);
    }
    bytes
}

#[test]
fn registered_derivative_x_and_space_qualify_actual_font_ink_and_source() {
    for (control, visible_space) in [
        (Some(3), false),
        (Some(58), false),
        (None, false),
        (None, true),
    ] {
        qualified_font(control, visible_space);
    }
}

#[test]
fn control_character_mapped_to_a_blank_glyph_still_requires_positive_actual_outline_ink() {
    let bytes = qualified_font(Some(3), false);
    let control = rendered(&bytes, "\u{1}");
    // Visibility is not proved by its nonzero advance or the line box. The
    // contract permits any valid synthetic shape/font/advance, but requires ink.
    positive_ink(control.ink);
}

#[test]
fn control_character_with_a_real_visible_mapping_retains_positive_ink() {
    let bytes = qualified_font(Some(58), false);
    positive_ink(rendered(&bytes, "\u{1}").ink);
}

fn zero_advance_and_caret(value: &Rendered) {
    assert_eq!(value.glyphs.iter().map(|glyph| glyph.1).sum::<f32>(), 0.0);
    assert_eq!(value.end_caret_x, 0.0);
}

#[test]
fn unsupported_default_ignorable_with_blank_space_is_zero_advance_and_empty_ink() {
    let bytes = qualified_font(None, false);
    let ignored = rendered(&bytes, "\u{2065}");
    zero_advance_and_caret(&ignored);
    assert_eq!(ignored.ink, text::LineOutlineInk::Empty);
}

#[test]
fn unsupported_default_ignorable_with_visible_space_is_still_empty_ink() {
    let bytes = qualified_font(None, true);
    let ignored = rendered(&bytes, "\u{2065}");
    // Zero advance, end caret and exact original/selection/plaintext custody
    // are qualified independently before the actual outline visibility check.
    zero_advance_and_caret(&ignored);
    assert_eq!(ignored.ink, text::LineOutlineInk::Empty);
}
