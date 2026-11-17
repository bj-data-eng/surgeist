//! Error text of the standalone Text layout operation, observed through `prepare_text_layout`.
//!
//! The layout is the two-run text `AX` in Ahem, whose A and X glyphs are each one rectangle of five
//! path elements (a move, three lines and a close). The expected sentences are written from that
//! fact and from the limits the test sets, not from the message implementation.
//!
//! Not covered here: the leader-pattern types, which need a finalized leader pattern from a Text
//! layout of leaders that no test of this crate builds; `TextLayoutCause::Text` and
//! `TextLayoutError::Allocation`, which need a Text qualification failure or an allocator refusal;
//! and the `GlyphPreparationError` variants that depend on a malformed font table. The tables in
//! `error_display_text.rs` cover those texts.
#![cfg(feature = "render-text")]
#![cfg(not(target_arch = "wasm32"))]

use core::error::Error as _;
use surgeist_render::{
    Error, ErrorCode, GlyphPreparationError, GlyphRunLimits, TextLayoutLimits, Transform,
    prepare_text_layout,
};
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const ROOM: usize = 8 << 20;

fn layout() -> text::Layout {
    let mut system = text::System::default();
    system
        .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 20.0,
        line_height: text::LineHeight::Absolute(20.0),
        ..text::Style::default()
    };
    let mut source = text::Source::new("AX");
    // The second run is a separate span, so each glyph is its own item of the layout.
    source.span(
        text::Range::new(1, 2),
        text::Style {
            brush: text::Brush::color(1.0, 0.0, 0.0, 1.0),
            ..style.clone()
        },
    );
    system
        .layout(
            source,
            style,
            text::Options {
                quantize: false,
                ..text::Options::default()
            },
        )
        .unwrap()
}

fn limits(glyphs: usize, paths: usize, runs: usize) -> TextLayoutLimits {
    TextLayoutLimits::new(GlyphRunLimits::new(glyphs, paths, ROOM, ROOM), runs, 0, 0)
}

fn refuse(limits: TextLayoutLimits) -> Error {
    let layout = layout();
    prepare_text_layout(
        &layout,
        Transform::identity(),
        limits,
        &mut surgeist_render::RenderAccountingContext::default(),
    )
    .map(|_| ())
    .unwrap_err()
}

#[test]
fn a_caller_path_limit_reached_inside_an_item_is_not_described_as_a_refused_font() {
    // Both glyphs have five path elements; the whole attempt may emit nine, so the second run, which
    // brings the total to ten, is the item that is refused. The cause is the caller's own limit.
    let error = refuse(limits(2, 9, 2));
    assert_eq!(error.code(), ErrorCode::TextLayout);
    assert_eq!(
        error.to_string(),
        "layout item at run 1 on line 0 was refused: render could not prepare the item: path elements requires 10, above the limit of 9"
    );
    assert_eq!(error.message(), error.to_string());
    // The typed cause is the glyph limit, and `Error` itself does not return it as a source.
    let item = error.text_layout_diagnostic().unwrap();
    let source = item.source().unwrap();
    assert_eq!(source.to_string(), "render could not prepare the item");
    let cause = source.source().unwrap();
    assert_eq!(
        cause.to_string(),
        "path elements requires 10, above the limit of 9"
    );
    let glyph = cause
        .downcast_ref::<Error>()
        .unwrap()
        .glyph_preparation_diagnostic()
        .unwrap();
    assert!(matches!(
        glyph,
        GlyphPreparationError::Limit {
            required: 10,
            limit: 9,
            ..
        }
    ));
}

#[test]
fn a_layout_level_limit_names_the_counted_resource() {
    // Two runs against a caller limit of one run.
    let error = refuse(limits(2, 10, 1));
    assert_eq!(error.code(), ErrorCode::TextLayout);
    assert_eq!(
        error.to_string(),
        "placed layout needs 2 runs, above the limit of 1"
    );
}

#[test]
fn the_glyph_limit_of_the_layout_names_glyphs() {
    // Two glyphs against a caller limit of one glyph.
    let error = refuse(limits(1, 10, 2));
    assert_eq!(error.code(), ErrorCode::TextLayout);
    assert_eq!(
        error.to_string(),
        "placed layout needs 2 glyphs, above the limit of 1"
    );
}
