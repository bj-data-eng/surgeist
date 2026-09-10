//! Original UTF-8 offsets and zero-based UTF-16 coordinates are public contracts.

use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssSourcePosition, CssValueOrigin, CssValueTokenRef,
    parse_component_values,
};

fn assert_position(position: CssSourcePosition, byte: usize, line: u32, column: u32) {
    assert_eq!(position.byte_offset().value(), byte);
    assert_eq!(position.line().value(), line);
    assert_eq!(position.column().value(), column);
}

fn assert_origin(
    token: &CssComponentValue,
    source: &str,
    start: (usize, u32, u32),
    end: (usize, u32, u32),
) {
    let CssValueOrigin::Parsed(origin) = token.origin() else {
        panic!("parsed tokens retain original positions");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_position(origin.span().start(), start.0, start.1, start.2);
    assert_position(origin.span().end(), end.0, end.1, end.2);
}

#[test]
fn line_endings_and_supplementary_characters_keep_original_token_coordinates() {
    // Shift each newline and four-byte character across several source-index
    // boundaries. CRLF is one line ending; an emoji occupies two UTF-16 units.
    for padding in (59..=68).chain(123..=132) {
        for newline in ["\r\n", "\r", "\n", "\u{000c}"] {
            let source = format!("{}{newline}😀;z", "a".repeat(padding));
            let values = parse_component_values(&source).expect("valid component sequence");
            let [prefix, whitespace, emoji, semicolon, last] = values.items() else {
                panic!("identifier, whitespace, emoji, semicolon and final identifier");
            };
            let line_start = padding + newline.len();
            assert_origin(prefix, &source, (0, 0, 0), (padding, 0, padding as u32));
            assert_origin(
                whitespace,
                &source,
                (padding, 0, padding as u32),
                (line_start, 1, 0),
            );
            assert!(matches!(
                emoji.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("😀"))
            ));
            assert_origin(emoji, &source, (line_start, 1, 0), (line_start + 4, 1, 2));
            assert_origin(
                semicolon,
                &source,
                (line_start + 4, 1, 2),
                (line_start + 5, 1, 3),
            );
            assert_origin(
                last,
                &source,
                (line_start + 5, 1, 3),
                (line_start + 6, 1, 4),
            );
        }
    }
}

#[test]
fn token_after_multiple_long_lines_uses_the_original_line_and_utf16_column() {
    for padding in 59..=68 {
        let prefix = "a".repeat(padding);
        let long_line = "b".repeat(141);
        let source = format!("{prefix}\r\n{long_line}\r😀\u{000c}{long_line}\n😀;z");
        let values = parse_component_values(&source).expect("valid component sequence");
        let last = values.items().last().expect("last identifier");
        assert!(matches!(
            last.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Ident("z"))
        ));
        assert_origin(
            last,
            &source,
            (source.len() - 1, 4, 3),
            (source.len(), 4, 4),
        );
    }
}
