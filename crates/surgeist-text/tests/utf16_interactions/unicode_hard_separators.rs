//! CSS Text's unconditional BK/NL hard-break semantics, independent of the
//! document's LF segment-break declaration and whitespace collapsing mode.

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

fn native_hard_separator(separator: u16, class: LineBreak, white_space: WhiteSpace) {
    // VT/FF are BK; NEL is NL. CSS Text 3 #unicode-forced-breaks (and
    // Text 4's same clause) requires these regardless of white-space.
    assert_eq!(
        CodePointMapData::<LineBreak>::new().get32(u32::from(separator)),
        class
    );
    let source = original(separator);
    let analyzed = AnalyzedSource::from_paragraph(ParagraphInput {
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
    .expect("the actual BK/NL source is admitted");
    assert_eq!(analyzed.source(), &source);
    assert_eq!(
        analyzed.processed_source().processed_text().code_units(),
        [0x41, separator, 0x42],
        "LineFeedDefault does not redefine BK/NL as a collapsible document delimiter"
    );
    let mut system = registered_ahem();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .expect("registered Ahem prepares the actual source");
    drop(system);
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.control_views().len(), 0);
    let hard = prepared
        .unit_views()
        .find(|unit| unit.source_range() == Range::new(1, 2))
        .expect("the original separator has an actual source-owned unit")
        .handle();
    assert_eq!(hard.prepared_text(), prepared);
    let after = prepared
        .participants()
        .position(|participant| participant == PreparedParticipant::Text(hard.clone()))
        .expect("the hard unit participates in this exact preparation")
        + 1;
    let following: Vec<_> = prepared
        .opportunity_views()
        .filter(|offer| offer.boundary().view().participant_offset() == after)
        .map(|offer| offer.kind())
        .collect();
    assert_eq!(
        following,
        [PreparedBreakKind::Forced],
        "BK/NL requires a forced offer after its real unit, not an optional wrap"
    );
}

#[test]
fn native_vertical_tab_is_hard_in_preserve() {
    native_hard_separator(0x000b, LineBreak::MandatoryBreak, WhiteSpace::Preserve);
}

#[test]
fn native_vertical_tab_is_hard_in_collapse() {
    native_hard_separator(0x000b, LineBreak::MandatoryBreak, WhiteSpace::Collapse);
}

#[test]
fn native_form_feed_is_hard_in_preserve() {
    native_hard_separator(0x000c, LineBreak::MandatoryBreak, WhiteSpace::Preserve);
}

#[test]
fn native_form_feed_is_hard_in_collapse() {
    native_hard_separator(0x000c, LineBreak::MandatoryBreak, WhiteSpace::Collapse);
}

#[test]
fn native_next_line_is_hard_in_preserve() {
    native_hard_separator(0x0085, LineBreak::NextLine, WhiteSpace::Preserve);
}

#[test]
fn native_next_line_is_hard_in_collapse() {
    native_hard_separator(0x0085, LineBreak::NextLine, WhiteSpace::Collapse);
}

fn standalone_hard_separator(separator: u16) {
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
    // No width constraint can cause a soft wrap. Both existing public entries
    // must execute the hard separator; LF supplies the same font/line geometry
    // without guessing a glyph or requiring a font mapping for the control.
    for output in [direct, built] {
        assert_eq!(output.source(), &source);
        assert_eq!(output.source().id(), Some(SOURCE_ID));
        assert_eq!(output.source().revision(), REVISION);
        assert_eq!(output.source().text().code_units(), [0x41, separator, 0x42]);
        assert_eq!(output.metrics().line_count(), 2);
        assert_eq!(
            output
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 3)]
        );
        assert_eq!(output.lines(), lf.lines());
    }
}

#[test]
fn standalone_vertical_tab_ends_the_line_without_a_width_constraint() {
    standalone_hard_separator(0x000b);
}

#[test]
fn standalone_form_feed_ends_the_line_without_a_width_constraint() {
    standalone_hard_separator(0x000c);
}

#[test]
fn standalone_next_line_ends_the_line_without_a_width_constraint() {
    standalone_hard_separator(0x0085);
}
