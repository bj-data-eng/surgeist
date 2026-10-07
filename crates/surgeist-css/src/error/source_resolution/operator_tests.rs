//! Literal source-token diagnostics required by Syntax3 delimiter fallback.
//! Next and previous observers share actual authored one-character boundaries.
use super::*;

#[test]
fn next_operator_observation_uses_each_authored_delimiter() {
    for (prefix, source) in [
        ('~', "~="),
        ('|', "|="),
        ('^', "^="),
        ('$', "$="),
        ('*', "*="),
    ] {
        for (offset, expected) in [(0, prefix.to_string()), (1, "=".to_owned())] {
            let (start, summary) = next_authored_token_at(
                source,
                CssSourcePosition::from_byte_offset_in(source, offset),
            )
            .unwrap();
            assert_eq!(start, offset);
            assert_eq!(summary.kind(), CssTokenKind::Delim);
            assert_eq!(summary.authored(), expected);
        }
    }
}

#[test]
fn previous_operator_observation_uses_actual_last_delimiter() {
    for source in ["~=", "|=", "^=", "$=", "*="] {
        let (start, summary) = previous_authored_token_before(
            source,
            CssSourcePosition::from_byte_offset_in(source, 2),
        )
        .unwrap();
        assert_eq!(start, 1);
        assert_eq!(summary.kind(), CssTokenKind::Delim);
        assert_eq!(summary.authored(), "=");
        let (start, summary) = previous_authored_token_before(
            source,
            CssSourcePosition::from_byte_offset_in(source, 1),
        )
        .unwrap();
        assert_eq!(start, 0);
        assert_eq!(summary.kind(), CssTokenKind::Delim);
        assert_eq!(summary.authored(), &source[..1]);
    }
}

#[test]
fn diagnostic_token_walkers_preserve_whitespace_and_comment_controls() {
    let source = " \t/**/~ ";
    let (start, summary) =
        next_authored_token_at(source, CssSourcePosition::from_byte_offset_in(source, 0)).unwrap();
    assert_eq!(start, 6);
    assert_eq!(summary.kind(), CssTokenKind::Delim);
    assert_eq!(summary.authored(), "~");
    let source = "x/**/ !";
    let (start, summary) = previous_authored_token_before(
        source,
        CssSourcePosition::from_byte_offset_in(source, source.len()),
    )
    .unwrap();
    assert_eq!(start, 6);
    assert_eq!(summary.kind(), CssTokenKind::Delim);
    assert_eq!(summary.authored(), "!");
}
