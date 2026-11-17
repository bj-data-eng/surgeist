//! `Error` built from the Text layout errors prints the whole cause chain.
#![cfg(feature = "render-text")]

use surgeist_render::{
    Error, ErrorCode, text_layout::LeaderPatternCause, text_layout::LeaderPatternError,
    text_layout::TextLayoutCause, text_layout::TextLayoutError, text_layout::TextLayoutLocation,
};

fn text_error() -> surgeist_text::Error {
    surgeist_text::Error::new(
        surgeist_text::ErrorCode::NoUsableFontFace,
        "no face for the run",
    )
}

fn render_error() -> Error {
    Error::invalid_value("radius", -1, "must be non-negative")
}

#[test]
fn the_error_wrapper_prints_the_whole_chain() {
    // `Error` is built by the crate from these values; its text is the message, which renders the
    // cause chain. Here the same chain is rendered by hand.
    let layout = TextLayoutError::Item {
        location: TextLayoutLocation::Run { line: 0, run: 0 },
        cause: TextLayoutCause::Render(Box::new(render_error())),
    };
    let error = Error::from(layout);
    assert_eq!(error.code(), ErrorCode::TextLayout);
    assert_eq!(
        error.to_string(),
        format!(
            "layout item at run 0 on line 0 was refused: render could not prepare the item: {}",
            render_error()
        )
    );
    let leader = Error::from(LeaderPatternError::Run {
        run: 4,
        cause: LeaderPatternCause::Text(Box::new(text_error())),
    });
    assert_eq!(leader.code(), ErrorCode::LeaderPattern);
    assert_eq!(
        leader.to_string(),
        format!(
            "leader run 4 was refused: text could not qualify the retained instance or glyph frame: {}",
            text_error()
        )
    );
}
