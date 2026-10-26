//! Positive dispatch characterization from the pinned CSS Syntax 3 CRD
//! (2026-10-01), §§4.3.1–4.3.2 and §5.5.9. Comments are consumed before
//! dispatch; a contiguous whitespace run yields one token; punctuation has
//! dedicated tokens or delimiter fallback. The public component model retains
//! comments and folds matched brackets into blocks. Its strict unmatched-closer
//! error remains the owning observable contract.
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-token>

use std::ops::Range;

use surgeist_css::{
    CssBlockKind, CssComponentValueRef, CssComponentValues, CssParser, CssValueOrigin,
    CssValueTokenRef,
};

fn assert_origin(origin: &CssValueOrigin, source: &str, expected: Range<usize>) {
    let CssValueOrigin::Parsed(parsed) = origin else {
        panic!("complete authored component has a genuine parsed origin");
    };
    assert_eq!(parsed.source().as_str(), source);
    assert_eq!(parsed.span().start().byte_offset().value(), expected.start);
    assert_eq!(parsed.span().end().byte_offset().value(), expected.end);
}

#[test]
fn consecutive_complete_comments_preserve_the_following_dispatched_token() {
    // §4.3.2 returns to the comment-consumption step after each complete comment.
    // Comment retention is the component model's contract, not a whitespace token.
    let source = "/*first*//*second*/@next";
    let values = CssComponentValues::parse(&CssParser::default(), source)
        .expect("complete comments then at-keyword");
    let [first, second, following] = values.items() else {
        panic!("two distinct retained comments followed by one dispatched token");
    };
    assert!(matches!(
        first.view(),
        CssComponentValueRef::Comment("first")
    ));
    assert!(matches!(
        second.view(),
        CssComponentValueRef::Comment("second")
    ));
    assert!(matches!(
        following.view(),
        CssComponentValueRef::Token(CssValueTokenRef::AtKeyword("next"))
    ));
    assert_origin(first.origin(), source, 0..9);
    assert_origin(second.origin(), source, 9..19);
    assert_origin(following.origin(), source, 19..24);
}

#[test]
fn contiguous_mixed_whitespace_is_one_component_with_original_spelling() {
    // §3.3 preprocesses CRLF, CR and FF as newlines for tokenization; §4.3.1
    // consumes the complete whitespace run. Retained spelling stays authored.
    let source = " \t\r\n\u{c}\n\r :";
    let whitespace = " \t\r\n\u{c}\n\r ";
    let values = CssComponentValues::parse(&CssParser::default(), source)
        .expect("one complete whitespace run");
    let [run, following] = values.items() else {
        panic!("one whitespace component followed by a colon");
    };
    assert!(matches!(
        run.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(actual)) if actual == whitespace
    ));
    assert!(matches!(
        following.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Colon)
    ));
    assert_origin(run.origin(), source, 0..8);
    assert_origin(following.origin(), source, 8..9);
}

#[test]
fn punctuation_dispatch_uses_dedicated_tokens_and_delimiter_fallback() {
    // These single-character inputs cannot start a number, identifier, hash,
    // comment, at-keyword, CDO or CDC. Their §4.3.1 fallback is unambiguous.
    // Existing boundary tests own complete CDO/CDC and escape lookahead cases.
    for source in [
        ":", ";", ",", "+", "-", ".", "<", "@", "#", "%", "!", "$", "&", "*", "/", "=", ">", "?",
        "^", "`", "|", "~",
    ] {
        let values = CssComponentValues::parse(&CssParser::default(), source)
            .expect("ordinary punctuation component");
        let [component] = values.items() else {
            panic!("one punctuation token: {source:?}");
        };
        match (source, component.view()) {
            (":", CssComponentValueRef::Token(CssValueTokenRef::Colon))
            | (";", CssComponentValueRef::Token(CssValueTokenRef::Semicolon))
            | (",", CssComponentValueRef::Token(CssValueTokenRef::Comma)) => {}
            (expected, CssComponentValueRef::Token(CssValueTokenRef::Delim(actual)))
                if !matches!(expected, ":" | ";" | ",") =>
            {
                assert_eq!(actual, expected.chars().next().unwrap(), "{source:?}");
            }
            (_, actual) => panic!("incorrect punctuation dispatch for {source:?}: {actual:?}"),
        }
        assert_origin(component.origin(), source, 0..1);
    }
}

#[test]
fn bracket_dispatch_matches_all_block_kinds_and_rejects_unmatched_closers() {
    // §4.3.1 dispatches all six bracket characters. §5.5.9 groups each opener
    // with its matching closer; closers are structural, not leaf token variants.
    let source = "()[]{}";
    let values = CssComponentValues::parse(&CssParser::default(), source)
        .expect("three complete empty blocks");
    assert_eq!(values.items().len(), 3);
    for (index, (component, expected)) in values
        .items()
        .iter()
        .zip([
            CssBlockKind::Parenthesis,
            CssBlockKind::SquareBracket,
            CssBlockKind::CurlyBracket,
        ])
        .enumerate()
    {
        let CssComponentValueRef::Block(block) = component.view() else {
            panic!("matched bracket pair forms a block");
        };
        assert_eq!(block.kind(), expected);
        assert!(block.values().items().is_empty());
        let start = index * 2;
        // The public component contract exposes the opening token's origin;
        // the matching closer has its own independently retained origin.
        assert_origin(component.origin(), source, start..start + 1);
        assert_origin(block.closing_origin(), source, start + 1..start + 2);
    }

    for source in [")", "]", "}"] {
        let error = CssComponentValues::parse(&CssParser::default(), source)
            .expect_err("strict unmatched closer");
        assert!(error.is_unmatched_closing_delimiter(), "{source:?}");
        assert_origin(error.origin(), source, 0..1);
    }
}
