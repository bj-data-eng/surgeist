//! Native Cc presentation is distinct from original Unicode BK/NL properties.

use super::*;
use icu_properties::{CodePointMapData, props::LineBreak};

const SOURCE_ID: Id = Id::from_u64(108072);
const REVISION: u64 = 4;

fn original(separator: u16) -> Source {
    Source::identified(
        Text::from_code_units(vec![0x41, separator, 0x42]),
        SOURCE_ID,
        REVISION,
    )
}

fn native_visible_control(separator: u16, class: LineBreak, white_space: WhiteSpace) {
    // Original Unicode properties stay unchanged; the native presentation
    // contract gives eligible undeclared Cc the ordinary-symbol AL profile.
    assert_eq!(
        CodePointMapData::<LineBreak>::new().get32(u32::from(separator)),
        class
    );
    let source = original(separator);
    let analyzed = AnalyzedSource::from_paragraph(ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            white_space,
            wrap: Wrap::Word,
            ..style()
        },
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(0, 3),
            aggregate: Range::new(0, 3),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: Vec::new(),
        controls: Vec::new(),
        boundary_policies: Vec::new(),
        replacements: Vec::new(),
        anchor_order: Vec::new(),
    })
    .expect("the original control source is admitted");
    assert_eq!(analyzed.source(), &source);
    assert_eq!(
        analyzed.processed_source().processed_text().code_units(),
        [0x41, separator, 0x42],
        "LineFeedDefault leaves undeclared controls unchanged"
    );
    let mut system = registered_ahem();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .expect("registered Ahem prepares the actual source");
    drop(system);
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.control_views().len(), 0);
    let symbol = prepared
        .unit_views()
        .find(|unit| unit.source_range() == Range::new(1, 2))
        .expect("the original control has an actual source-owned unit")
        .handle();
    assert_eq!(symbol.prepared_text(), prepared);
    assert!(symbol.view().advance() > 0.0);
    let after = prepared
        .participants()
        .position(|participant| participant == PreparedParticipant::Text(symbol.clone()))
        .expect("the symbol participates in this exact preparation")
        + 1;
    let following: Vec<_> = prepared
        .opportunity_views()
        .filter(|offer| offer.boundary().view().participant_offset() == after)
        .map(|offer| offer.kind())
        .collect();
    assert!(following.is_empty(), "AL between A and B creates no break");
    use skrifa::MetadataProvider;
    let view = symbol.view();
    let instance = view.font_instances().next().unwrap().unwrap();
    let face = instance.face();
    assert_ne!(face.bytes(), AHEM);
    let font = skrifa::FontRef::from_index(face.bytes(), face.index()).unwrap();
    let glyph = font.charmap().map(u32::from(separator)).unwrap();
    let bounds = instance
        .glyph_outline(glyph.to_u32(), face.catalogue_identity())
        .unwrap()
        .bounds()
        .unwrap();
    assert!(bounds.size.width > 0.0 && bounds.size.height > 0.0);
}

#[test]
fn native_vertical_tab_is_a_symbol_in_preserve() {
    native_visible_control(0x000b, LineBreak::MandatoryBreak, WhiteSpace::Preserve);
}

#[test]
fn native_vertical_tab_is_a_symbol_in_collapse() {
    native_visible_control(0x000b, LineBreak::MandatoryBreak, WhiteSpace::Collapse);
}

#[test]
fn native_form_feed_is_a_symbol_in_preserve() {
    native_visible_control(0x000c, LineBreak::MandatoryBreak, WhiteSpace::Preserve);
}

#[test]
fn native_form_feed_is_a_symbol_in_collapse() {
    native_visible_control(0x000c, LineBreak::MandatoryBreak, WhiteSpace::Collapse);
}

#[test]
fn native_next_line_is_a_symbol_in_preserve() {
    native_visible_control(0x0085, LineBreak::NextLine, WhiteSpace::Preserve);
}

#[test]
fn native_next_line_is_a_symbol_in_collapse() {
    native_visible_control(0x0085, LineBreak::NextLine, WhiteSpace::Collapse);
}

fn standalone_visible_control(separator: u16) {
    let source = original(separator);
    let actual_style = Style {
        white_space: WhiteSpace::Preserve,
        ..style()
    };
    let actual_options = Options {
        width: None,
        quantize: false,
        ..Options::default()
    };
    assert_eq!(actual_options.width, None);
    let mut system = registered_ahem();
    let lf = system
        .layout(original(0x000a), actual_style.clone(), actual_options)
        .expect("the existing LF hard-break control lays out");
    assert_eq!(lf.metrics().line_count(), 2);
    let direct = system
        .layout(source.clone(), actual_style.clone(), actual_options)
        .expect("the actual separator is admitted by standalone layout");
    let built = system
        .builder(source.text().clone())
        .identity(SOURCE_ID, REVISION)
        .default_style(actual_style)
        .options(actual_options)
        .build()
        .expect("the actual separator is admitted by Builder::build");
    drop(system);
    // Both public entries execute the same symbol presentation. LF retains
    // its independent hard-break behavior under the same unconstrained layout.
    for output in [direct, built] {
        assert_eq!(output.source(), &source);
        assert_eq!(output.source().id(), Some(SOURCE_ID));
        assert_eq!(output.source().revision(), REVISION);
        assert_eq!(output.source().text().code_units(), [0x41, separator, 0x42]);
        assert_eq!(output.metrics().line_count(), 1);
        assert_eq!(
            output
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 3)]
        );
        let run = output
            .run_views()
            .find(|r| r.range() == Range::new(1, 2))
            .unwrap();
        assert_ne!(run.face().bytes(), AHEM);
        let instance = run.effective_instance().unwrap();
        let glyph = run.glyphs().next().unwrap();
        let bounds = instance
            .glyph_outline(glyph.id(), run.face().catalogue_identity())
            .unwrap()
            .bounds()
            .unwrap();
        assert!(bounds.size.width > 0.0 && bounds.size.height > 0.0);
    }
}

#[test]
fn standalone_vertical_tab_is_visible_without_a_forced_line() {
    standalone_visible_control(0x000b);
}

#[test]
fn standalone_form_feed_is_visible_without_a_forced_line() {
    standalone_visible_control(0x000c);
}

#[test]
fn standalone_next_line_is_visible_without_a_forced_line() {
    standalone_visible_control(0x0085);
}
