#![forbid(unsafe_code)]

use std::ptr;

use surgeist_font::{
    CataloguePublication, DecodeError, DecodeLimits, FontFace, FontResource, InvalidKind,
    LimitKind, SourceFormat,
};
use surgeist_text as t;

mod common;

const TTF: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const WOFF2: &[u8] = include_bytes!("../../surgeist-font/tests/fixtures/ahem-original/Ahem.woff2");

fn woff2_limits(decoded: usize) -> DecodeLimits {
    // Allow the Brotli state and reconstruction workspace under the same
    // finite envelope as the Font codec's compressed-resource tests.
    DecodeLimits::new(2 << 20, decoded, 2 << 20, 256, 16, 32 << 20, 40 << 20)
}

fn u32_at(bytes: &[u8], offset: usize) -> usize {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize
}

fn put32(bytes: &mut [u8], offset: usize, value: usize) {
    bytes[offset..offset + 4].copy_from_slice(&u32::try_from(value).unwrap().to_be_bytes());
}

// The WOFF1 wire fixture stores literal tables from the independently retained
// 1.50 SFNT. Directory order is by tag and payload order is by original offset.
fn literal_woff1() -> Vec<u8> {
    let count = usize::from(u16::from_be_bytes(TTF[4..6].try_into().unwrap()));
    let mut wire = vec![0; 44 + 20 * count];
    wire[..4].copy_from_slice(b"wOFF");
    wire[4..8].copy_from_slice(&TTF[..4]);
    wire[12..14].copy_from_slice(&TTF[4..6]);
    let mut records = (0..count).collect::<Vec<_>>();
    records.sort_unstable_by_key(|&index| u32_at(TTF, 20 + 16 * index));
    let mut decoded_len = 12 + 16 * count;
    for index in records {
        let record = 12 + 16 * index;
        let entry = 44 + 20 * index;
        let original = u32_at(TTF, record + 8);
        let len = u32_at(TTF, record + 12);
        let wire_offset = wire.len();
        wire[entry..entry + 4].copy_from_slice(&TTF[record..record + 4]);
        put32(&mut wire, entry + 4, wire_offset);
        put32(&mut wire, entry + 8, len);
        put32(&mut wire, entry + 12, len);
        wire[entry + 16..entry + 20].copy_from_slice(&TTF[record + 4..record + 8]);
        wire.extend_from_slice(&TTF[original..original + len]);
        wire.resize(wire.len().next_multiple_of(4), 0);
        decoded_len += len.next_multiple_of(4);
    }
    let wire_len = wire.len();
    put32(&mut wire, 8, wire_len);
    put32(&mut wire, 16, decoded_len);
    wire
}

fn style() -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        ..t::Style::default()
    }
}

fn analysis() -> t::AnalyzedSource {
    let source = t::Source::new("AX\nXA");
    let input = t::ParagraphInput {
        source: source.clone(),
        base_style: style(),
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: t::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: t::ParagraphStrut::DefaultFont,
        terminal_edge: t::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![t::ParagraphOrigin {
            text_node: None,
            source,
            local: t::Range::new(0, 5),
            aggregate: t::Range::new(0, 5),
            kind: t::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed =
        t::AnalyzedSource::from_paragraph(input, t::AnalysisTypography::HorizontalComposition)
            .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    analyzed.with_wrapping_policies(policies).unwrap()
}

fn finish(
    p: &t::PreparedText,
    start: t::PreparedBoundary,
    opportunity: t::PreparedOpportunity,
    line_start: t::LineStart,
    line_end: t::LineEnd,
) -> t::FinalizedLine {
    let end = opportunity.view().boundary();
    let from = start.view().participant_offset();
    let to = end.view().participant_offset();
    let participants = p
        .participants()
        .skip(from)
        .take(to - from)
        .collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            t::PreparedParticipant::Control(occurrence) => Some(t::ControlAllocation {
                occurrence: occurrence.clone(),
                strut: None,
                boundary: None,
            }),
            _ => None,
        })
        .collect();
    p.select(t::LineSelectionRequest {
        inline_boxes: None,
        segment: p.source_segments().next().unwrap(),
        start,
        end,
        participants,
        termination: t::PreparedTermination::Ordinary(opportunity),
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_box_struts: vec![],
        line_strut: None,
        context: t::LineContext {
            alignment_insets: t::LineAlignmentInsets::ZERO,
            formatted_line: t::FormattedLineContext::default(),
            base_direction: t::ParagraphBaseDirection::LeftToRight,
            line_box_direction: t::LineBoxDirection::LeftToRight,
            orientation: t::LineOrientation::Horizontal,
            start: line_start,
            end: line_end,
            available_inline: Some(20.0),
            indent: 0.0,
            alignment: t::Alignment::Start,
            tabs: t::TabContext::NoTabs,
            spacing: t::SpacingPolicy::None,
        },
    })
    .unwrap()
    .finalize(t::LineAllocations {
        spacing: t::SpacingAllocations::None,
        scopes: t::ScopeAllocations::None,
    })
    .unwrap()
}

fn check_lines(
    p: &t::PreparedText,
    face: &FontFace,
    publication: &CataloguePublication,
    ids: [u32; 2],
) {
    assert_eq!(p.font_catalogue_identity(), publication);
    let start = p.boundary_views().next().unwrap().handle();
    let first = p
        .opportunity_views()
        .find(|offer| offer.kind() == t::PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let last = p
        .opportunity_views()
        .find(|offer| offer.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let middle = first.view().boundary();
    assert_eq!(start.view().source_after().get(), 0);
    assert_eq!(middle.view().source_after().get(), 3);
    assert_eq!(last.view().boundary().view().source_after().get(), 5);

    let first = finish(p, start, first, t::LineStart::Paragraph, t::LineEnd::Forced);
    let last = finish(
        p,
        middle,
        last,
        t::LineStart::Continuation,
        t::LineEnd::Paragraph,
    );
    for (line, start, expected) in [(first, 0, ids), (last, 3, [ids[1], ids[0]])] {
        assert_eq!(line.prepared_text().font_catalogue_identity(), publication);
        let feedback = line.feedback();
        assert_eq!(
            (feedback.advance, feedback.line_height, feedback.baseline),
            (20.0, 10.0, 8.0)
        );
        let mut glyphs = Vec::new();
        for run in line.run_views() {
            assert_eq!(run.face().admitted_face(), face);
            assert_eq!(run.face().catalogue_identity(), publication);
            assert!(ptr::eq(run.face().bytes(), face.resource().bytes()));
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.normalized_coords().len(), 0);
            let metrics = instance.metrics(publication).unwrap();
            assert_eq!(
                (
                    metrics.units_per_em(),
                    metrics.ascent(),
                    metrics.descent(),
                    metrics.leading()
                ),
                (1000, 8.0, -2.0, 0.0)
            );
            glyphs.extend(
                run.glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
            );
        }
        assert_eq!(
            glyphs,
            [
                (t::Range::new(start, start + 1), expected[0], 10.0),
                (t::Range::new(start + 1, start + 2), expected[1], 10.0)
            ]
        );
    }
}

#[test]
fn equivalent_raw_and_literal_woff1_drive_the_same_literal_lines() {
    let wrapped = literal_woff1();
    for (bytes, format) in [
        (TTF, SourceFormat::Sfnt),
        (wrapped.as_slice(), SourceFormat::Woff),
    ] {
        let resource = FontResource::decode(bytes.to_vec(), common::FONT_LIMITS).unwrap();
        let face = resource.face(0).unwrap();
        assert_eq!(resource.source().bytes(), bytes);
        assert_eq!(resource.source().format(), format);
        let mut system = t::System::default();
        system.register_font_resource(resource).unwrap();
        let prepared = system
            .prepare(&analysis(), t::PreparationOptions::default())
            .unwrap();
        let publication = system.font_catalogue_identity().clone();
        drop(system);
        // Ahem 1.50's checked-in fixture provenance supplies the glyph IDs,
        // one-em advances, and 800/-200/0 horizontal metric values.
        check_lines(&prepared, &face, &publication, [35, 58]);
    }
}

#[test]
fn original_ahem_woff2_keeps_its_source_and_face_across_refresh_and_finalization() {
    let mut system = t::System::default();
    system
        .register_fonts(WOFF2.to_vec(), woff2_limits(2 << 20))
        .unwrap();
    let original = system.registered_font_faces().next().unwrap();
    let face = original.admitted_face();
    assert_eq!(face.resource().source().format(), SourceFormat::Woff2);
    assert_eq!(face.resource().source().bytes(), WOFF2);
    assert_eq!(face.index(), 0);

    let analyzed = analysis();
    let old = system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    let before = system.font_catalogue_identity().clone();
    system.refresh_fonts().unwrap();
    let after = system.font_catalogue_identity().clone();
    assert_ne!(before, after);
    let fresh = system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    assert_ne!(old, fresh);
    let later = FontResource::decode(TTF.to_vec(), common::FONT_LIMITS).unwrap();
    assert_ne!(
        face.resource().source().identity(),
        later.source().identity()
    );
    assert_ne!(face.resource().bytes(), later.bytes());
    assert_ne!(face, &later.face(0).unwrap());
    let mut foreign = t::System::default();
    foreign.register_font_resource(later).unwrap();
    assert_eq!(
        original
            .unsynthesized_instance(10.0, &[])
            .unwrap()
            .metrics(foreign.font_catalogue_identity())
            .unwrap_err()
            .code,
        t::ErrorCode::FontCatalogueMismatch
    );

    drop(system);
    drop(foreign);
    // The original 1.1 cmap formats 0 and 4 independently map A and X to
    // 36 and 59. Its hmtx has 1000-unit advances and hhea is 800/-200/0.
    check_lines(&old, face, &before, [36, 59]);
    check_lines(&fresh, face, &after, [36, 59]);
}

#[test]
fn compressed_decode_and_budget_refusals_keep_a_warm_text_layout() {
    let mut system = t::System::default();
    system
        .register_fonts(WOFF2.to_vec(), woff2_limits(2 << 20))
        .unwrap();
    let source = t::Source::new("AX");
    let first = system
        .layout(source.clone(), style(), t::Options::default())
        .unwrap();
    let prior = system.font_catalogue_identity().clone();
    let stats = system.stats();
    let malformed = system
        .register_fonts(WOFF2[..8].to_vec(), woff2_limits(2 << 20))
        .unwrap_err();
    assert_eq!(malformed.code, t::ErrorCode::FontLoadFailed);
    assert!(
        matches!(malformed.detail(), Some(t::ErrorDetail::FontDecode(DecodeError::InvalidContainer(error))) if error.kind() == InvalidKind::WoffHeader)
    );
    let capped = woff2_limits(0);
    let refused = system.register_fonts(WOFF2.to_vec(), capped).unwrap_err();
    assert_eq!(refused.code, t::ErrorCode::FontLoadFailed);
    assert!(matches!(
        refused.detail(),
        Some(t::ErrorDetail::FontDecode(DecodeError::LimitExceeded {
            kind: LimitKind::DecodedBytes,
            limit: 0,
            ..
        }))
    ));
    assert_eq!(system.font_catalogue_identity(), &prior);
    assert_eq!(system.stats(), stats);
    assert_eq!(system.registered_font_faces().count(), 1);

    let again = system
        .layout(source, style(), t::Options::default())
        .unwrap();
    assert_eq!(first.key(), again.key());
    assert_eq!(system.stats().layout_hits(), stats.layout_hits() + 1);
    assert_eq!(system.stats().layout_misses(), stats.layout_misses());
    assert_eq!(first.metrics().full_width(), 20.0);
    assert_eq!(again.metrics().full_width(), 20.0);
    assert_eq!(
        first.glyph_runs()[0].font().admitted_face(),
        again.glyph_runs()[0].font().admitted_face()
    );
}
