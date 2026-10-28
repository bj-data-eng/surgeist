//! Tests for the token stream: the operations of CSS Syntax Module Level 3
//! section 5.3 (next token, empty, consume a token, discard whitespace, mark and
//! restore a mark), the consumption of blocks and lists of section 5.5.7 to
//! 5.5.10, and the errors a grammar over a stream reports.
//!
//! Expectations are written by hand from those sections and from the contracts
//! documented on the stream's methods. The comment is the one thing section 5
//! does not have; the stream treats it as transparent, as section 9 allows.

mod common;

use common::render;
use common::tree::parse;
use surgeist_css_parser::document::{BlockKind, GroupEnd, GroupKind};
use surgeist_css_parser::lexer::Options;
use surgeist_css_parser::limits::{DepthExcess, Limits};
use surgeist_css_parser::stream::{StopToken, StopTokens};
use surgeist_css_parser::token::TokenKind;
use surgeist_css_parser::{Document, Offset, Stream, SyntaxError, Token};

type Failed<'i> = SyntaxError<'i, ()>;

/// A way to consume that the stream should refuse.
type Probe = fn(&mut Stream<'_>) -> bool;

fn kinds(stream: &mut Stream<'_>) -> Vec<String> {
    let mut out = Vec::new();
    while let Ok(token) = stream.consume_token() {
        out.push(render(token));
    }
    out
}

// ----------------------------------------------------------------- section 5.3

#[test]
fn the_next_token_is_what_a_consume_returns_and_the_end_is_the_eof_token() {
    let document = parse("a b");
    let mut stream = document.root();
    assert_eq!(stream.next_token(), Some(Token::Ident("a")));
    // Looking does not consume.
    assert_eq!(stream.next_token(), Some(Token::Ident("a")));
    assert_eq!(stream.consume_token().unwrap(), Token::Ident("a"));
    assert_eq!(stream.next_token(), Some(Token::Whitespace));
    assert_eq!(stream.consume_token().unwrap(), Token::Whitespace);
    assert_eq!(stream.consume_token().unwrap(), Token::Ident("b"));
    // The <eof-token>: nothing is next, and consuming is an error.
    assert_eq!(stream.next_token(), None);
    assert!(stream.is_empty());
    assert!(stream.consume_token().unwrap_err().is_end_of_stream());
    assert!(stream.consume_token().unwrap_err().is_end_of_stream());
}

#[test]
fn a_stream_is_empty_only_when_the_next_token_is_eof_so_whitespace_is_not_empty() {
    let document = parse(" ");
    let mut stream = document.root();
    assert!(!stream.is_empty());
    assert!(stream.only_whitespace_remains());
    stream.discard_whitespace();
    assert!(stream.is_empty());
    assert!(parse("").root().is_empty());
}

#[test]
fn whitespace_is_a_token_and_comments_are_transparent_except_when_asked_for() {
    let document = parse("a/*c*/ b");
    assert_eq!(
        kinds(&mut document.root()),
        ["Ident(\"a\")", "Whitespace", "Ident(\"b\")"]
    );
    let mut stream = document.root();
    let mut all = Vec::new();
    while let Ok(token) = stream.consume_token_with_comments() {
        all.push(render(token));
    }
    assert_eq!(
        all,
        ["Ident(\"a\")", "Comment", "Whitespace", "Ident(\"b\")"]
    );
    // A comment before the first token is skipped by `next_token` too.
    let document = parse("/*c*/a");
    assert_eq!(document.root().next_token(), Some(Token::Ident("a")));
}

#[test]
fn discarding_whitespace_discards_whitespace_tokens_and_comments_and_nothing_else() {
    let document = parse(" /*c*/ \n a b");
    let mut stream = document.root();
    stream.discard_whitespace();
    assert_eq!(stream.next_token(), Some(Token::Ident("a")));
    stream.discard_whitespace();
    assert_eq!(stream.next_token(), Some(Token::Ident("a")));
    // With nothing to discard it does nothing; at the end it does nothing.
    let document = parse(" ");
    let mut stream = document.root();
    stream.discard_whitespace();
    stream.discard_whitespace();
    assert!(stream.is_empty());
}

#[test]
fn only_whitespace_remains_ignores_comments_and_sees_any_other_token() {
    assert!(parse("").root().only_whitespace_remains());
    assert!(parse(" /*c*/ \n").root().only_whitespace_remains());
    assert!(!parse(" a").root().only_whitespace_remains());
    assert!(!parse("/*c*/;").root().only_whitespace_remains());
    // It does not consume.
    let document = parse(" a");
    let stream = document.root();
    assert!(!stream.only_whitespace_remains());
    assert_eq!(stream.next_token(), Some(Token::Whitespace));
}

#[test]
fn require_empty_succeeds_at_the_end_after_whitespace_and_names_the_first_token_left() {
    let document = parse(" \t /*c*/ ");
    let mut stream = document.root();
    stream.require_empty().unwrap();
    assert!(stream.is_empty());

    let document = parse("  a b");
    let mut stream = document.root();
    let before = stream.site();
    let error = stream.require_empty().unwrap_err();
    assert!(error.is_unexpected_token());
    assert_eq!(error.unexpected_token(), Some(Token::Ident("a")));
    // The error is about the boundary before `a`, after the whitespace.
    assert_eq!(error.at().offset(), Some(Offset::new(2)));
    assert_eq!(error.at().spelling(), Some("a"));
    // A failed `require_empty` leaves the stream where it was.
    assert_eq!(stream.site(), before);
    assert_eq!(stream.next_token(), Some(Token::Whitespace));
}

#[test]
fn consuming_after_whitespace_skips_whitespace_first() {
    let document = parse("  /*c*/ a");
    let mut stream = document.root();
    assert_eq!(
        stream.consume_after_whitespace().unwrap(),
        Token::Ident("a")
    );
    assert!(
        stream
            .consume_after_whitespace()
            .unwrap_err()
            .is_end_of_stream()
    );
}

#[test]
fn a_mark_restores_the_index_and_can_be_restored_to_again() {
    let document = parse("a b c");
    let mut stream = document.root();
    stream.consume_ident().unwrap();
    let mark = stream.mark();
    assert_eq!(mark.site(), stream.site());
    stream.consume_ident().unwrap();
    assert_eq!(stream.consume_ident().ok(), Some("c"));
    stream.restore(mark);
    assert_eq!(stream.consume_ident().ok(), Some("b"));
    stream.restore(mark);
    assert_eq!(stream.consume_ident().ok(), Some("b"));
    // A copy of the stream accepts the stream's marks.
    let mut copy = stream.clone();
    copy.restore(mark);
    assert_eq!(copy.site(), mark.site());
    assert_ne!(stream.site(), copy.site());
}

#[test]
fn restoring_a_mark_brings_back_the_unentered_group_it_was_taken_with() {
    let document = parse("f(a b) tail");
    let mut stream = document.root();
    stream.consume_function_named("f").unwrap();
    let mark = stream.mark();
    assert_eq!(stream.unentered_block(), Some(GroupKind::Function));
    let inside = stream
        .within_block(|inner| Ok::<_, Failed<'_>>(inner.consume_ident()?))
        .unwrap_err();
    // `b` was left in the block, which the contents-must-be-empty rule rejects.
    assert_eq!(inside.unexpected_token(), Some(Token::Ident("b")));
    assert_eq!(stream.unentered_block(), None);
    stream.restore(mark);
    assert_eq!(stream.unentered_block(), Some(GroupKind::Function));
    stream
        .within_block(|inner| {
            inner.consume_keyword("a")?;
            inner.consume_keyword("b")?;
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
    assert_eq!(stream.consume_ident().ok(), Some("tail"));
}

#[test]
#[should_panic(expected = "a mark is restored on the stream it was taken on")]
fn a_mark_cannot_be_restored_on_another_stream() {
    let document = parse("a b");
    let mark = document.root().mark();
    let other = parse("a b");
    other.root().restore(mark);
}

#[test]
#[should_panic(expected = "a mark is restored on the stream it was taken on")]
fn a_mark_cannot_be_restored_on_a_stream_over_another_span() {
    let document = parse("a, b");
    let mut stream = document.root();
    let mark = stream.mark();
    let _ = stream.within_list_until(StopToken::Comma, |inner| {
        inner.restore(mark);
        Ok::<_, Failed<'_>>(())
    });
}

#[test]
fn an_attempt_keeps_what_succeeded_and_restores_what_failed() {
    let document = parse("a b");
    let mut stream = document.root();
    let kept = stream.attempt(|inner| inner.consume_ident());
    assert_eq!(kept.ok(), Some("a"));
    assert_eq!(stream.next_token(), Some(Token::Whitespace));
    let before = stream.site();
    let failed = stream.attempt(|inner| {
        inner.consume_after_whitespace()?;
        inner.consume_delim('+')
    });
    // `+` was wanted and the stream was empty.
    assert!(failed.unwrap_err().is_end_of_stream());
    assert_eq!(stream.site(), before);
    // The error is the grammar's own type, whatever it is.
    let failed: Result<(), &str> = stream.attempt(|_| Err("nothing matched"));
    assert_eq!(failed, Err("nothing matched"));
}

#[test]
fn parsing_completely_requires_the_grammar_to_use_everything_but_whitespace() {
    let document = parse("a b ");
    let mut stream = document.root();
    let value = stream.parse_complete(|inner| {
        inner.consume_keyword("a")?;
        Ok::<_, Failed<'_>>(inner.consume_ident()?)
    });
    assert_eq!(value.ok(), Some("b"));
    let document = parse("a b c");
    let error = document
        .root()
        .parse_complete(|inner| {
            inner.consume_keyword("a")?;
            Ok::<_, Failed<'_>>(inner.consume_ident()?)
        })
        .unwrap_err();
    assert_eq!(error.unexpected_token(), Some(Token::Ident("c")));
    // The grammar's own failure passes through.
    let document = parse("a");
    let error = document
        .root()
        .parse_complete(|inner| Err::<(), _>(inner.grammar_error::<&str, &str>("no")))
        .unwrap_err();
    assert_eq!(error.custom_value(), Some(&"no"));
}

// ----------------------------------------------------------------- consuming one kind of token

#[test]
fn each_consume_takes_its_own_kind_of_token_after_whitespace_and_names_a_wrong_one() {
    let document = parse(" a F( \"s\" +) , : ;");
    let mut stream = document.root();
    assert_eq!(stream.consume_ident().ok(), Some("a"));
    assert_eq!(stream.consume_function_name().ok(), Some("F"));
    stream
        .within_block(|inner| {
            assert_eq!(inner.consume_string()?, "s");
            inner.consume_delim('+')?;
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
    stream.consume_comma().unwrap();
    stream.consume_colon().unwrap();
    stream.consume_semicolon().unwrap();
    assert!(stream.consume_semicolon().unwrap_err().is_end_of_stream());
}

#[test]
fn a_keyword_and_a_function_name_match_ascii_case_insensitively() {
    let document = parse("BoLd Calc(");
    let mut stream = document.root();
    stream.consume_keyword("bold").unwrap();
    stream.consume_function_named("cALC").unwrap();
    // Not ASCII: a non-ASCII letter matches only itself.
    let document = parse("\u{e9}");
    let error = document.root().consume_keyword("\u{c9}").unwrap_err();
    assert!(error.is_unexpected_token());
}

#[test]
fn a_consume_of_the_wrong_kind_reports_the_token_and_where_it_was() {
    let cases: [(&str, Probe); 9] = [
        ("1", |stream| stream.consume_ident().is_err()),
        ("1", |stream| stream.consume_keyword("a").is_err()),
        ("a", |stream| stream.consume_function_name().is_err()),
        ("f(", |stream| stream.consume_function_named("g").is_err()),
        ("a", |stream| stream.consume_string().is_err()),
        ("a", |stream| stream.consume_delim('+').is_err()),
        ("a", |stream| stream.consume_comma().is_err()),
        ("a", |stream| stream.consume_colon().is_err()),
        ("a", |stream| stream.consume_semicolon().is_err()),
    ];
    for (text, attempt) in cases {
        let source = format!(" {text}");
        let document = parse(&source);
        let mut stream = document.root();
        assert!(attempt(&mut stream), "{text:?}");
    }
    let document = parse(" 12 ");
    let mut stream = document.root();
    let error = stream.consume_ident().unwrap_err();
    assert!(matches!(error.unexpected_token(), Some(Token::Number(_))));
    // The error is at the boundary before the token, after the whitespace.
    assert_eq!(error.at().offset(), Some(Offset::new(1)));
    // The token stays consumed: a caller that wants to look again uses a mark.
    assert_eq!(stream.consume_token().unwrap(), Token::Whitespace);
    assert!(stream.is_empty());
    // A delimiter must be the one asked for.
    let document = parse("+");
    let error = document.root().consume_delim('-').unwrap_err();
    assert_eq!(error.unexpected_token(), Some(Token::Delim('+')));
}

#[test]
fn consuming_at_the_end_is_an_end_of_stream_error_at_the_end_site() {
    let document = parse("a");
    let mut stream = document.root();
    stream.consume_ident().unwrap();
    let error = stream.consume_comma().unwrap_err();
    assert!(error.is_end_of_stream());
    assert!(!error.is_unexpected_token() && !error.is_block_expected() && !error.is_refused());
    assert_eq!(error.unexpected_token(), None);
    assert_eq!(error.at(), stream.end_site());
    assert_eq!(error.at().offset(), Some(Offset::new(1)));
    assert!(error.to_string().contains("end"));
}

// ----------------------------------------------------------------- blocks (5.5.9, 5.5.10)

#[test]
fn a_block_is_entered_after_the_token_that_opens_it_and_must_be_empty_afterwards() {
    for (text, kind) in [
        ("f(a)", GroupKind::Function),
        ("(a)", GroupKind::Block(BlockKind::Paren)),
        ("[a]", GroupKind::Block(BlockKind::Square)),
        ("{a}", GroupKind::Block(BlockKind::Curly)),
    ] {
        let document = parse(text);
        let mut stream = document.root();
        assert_eq!(stream.unentered_block(), None);
        stream.consume_token().unwrap();
        assert_eq!(stream.unentered_block(), Some(kind), "{text:?}");
        let name = stream
            .within_block(|inner| Ok::<_, Failed<'_>>(inner.consume_ident()?))
            .unwrap();
        assert_eq!(name, "a");
        // Entering consumed the group.
        assert_eq!(stream.unentered_block(), None);
        assert!(stream.is_empty());
    }
}

#[test]
fn leaving_tokens_in_a_block_is_an_error_naming_the_first_one() {
    let document = parse("(a b)");
    let mut stream = document.root();
    stream.consume_token().unwrap();
    let error = stream
        .within_block(|inner| Ok::<_, Failed<'_>>(inner.consume_ident()?))
        .unwrap_err();
    assert!(error.is_stream_error() && error.is_unexpected_token());
    assert_eq!(error.unexpected_token(), Some(Token::Ident("b")));
    // Whitespace left in the block is not an error.
    let document = parse("( a )");
    let mut stream = document.root();
    stream.consume_token().unwrap();
    stream
        .within_block(|inner| Ok::<_, Failed<'_>>(inner.consume_ident()?))
        .unwrap();
}

#[test]
fn entering_a_block_with_none_pending_is_an_error_not_a_panic() {
    let document = parse("a (b)");
    let mut stream = document.root();
    // Nothing consumed yet.
    let error = stream
        .within_block(|_| Ok::<(), Failed<'_>>(()))
        .unwrap_err();
    assert!(error.is_block_expected());
    assert!(error.is_stream_error());
    stream.consume_ident().unwrap();
    // The last consume was an identifier, not a group.
    assert!(
        stream
            .within_block(|_| Ok::<(), Failed<'_>>(()))
            .unwrap_err()
            .is_block_expected()
    );
    // Whitespace discarded after a group opens drops it.
    stream.discard_whitespace();
    stream.consume_token().unwrap();
    assert_eq!(
        stream.unentered_block(),
        Some(GroupKind::Block(BlockKind::Paren))
    );
    stream.discard_whitespace();
    assert_eq!(stream.unentered_block(), None);
    assert!(
        stream
            .within_block(|_| Ok::<(), Failed<'_>>(()))
            .unwrap_err()
            .is_block_expected()
    );
    // A group can be entered only once.
    let document = parse("(a)");
    let mut stream = document.root();
    stream.consume_token().unwrap();
    stream
        .within_block(|inner| Ok::<_, Failed<'_>>(inner.consume_ident()?))
        .unwrap();
    assert!(
        stream
            .within_block(|_| Ok::<(), Failed<'_>>(()))
            .unwrap_err()
            .is_block_expected()
    );
}

#[test]
fn a_denied_block_has_nothing_to_enter() {
    let limits = Limits::new(0, usize::MAX, usize::MAX);
    let document =
        Document::<()>::parse("{a}", &Options::default(), limits, DepthExcess::DenyBlock).unwrap();
    let mut stream = document.root();
    assert_eq!(stream.consume_token().unwrap(), Token::OpenCurly);
    assert_eq!(stream.unentered_block(), None);
    assert!(
        stream
            .within_block(|_| Ok::<(), Failed<'_>>(()))
            .unwrap_err()
            .is_block_expected()
    );
    assert!(stream.is_empty());
}

#[test]
fn a_block_not_entered_is_skipped_whole() {
    // Section 5.5.8: a group is one component value; a grammar that does not look
    // inside consumes it in one step.
    let document = parse("f(a b [c] d) tail");
    let mut stream = document.root();
    stream.consume_function_name().unwrap();
    assert_eq!(stream.consume_token().unwrap(), Token::Whitespace);
    assert_eq!(stream.consume_ident().ok(), Some("tail"));
}

#[test]
fn a_failed_nested_parse_still_advances_the_outer_stream_and_a_restore_brings_it_back() {
    let document = parse("f(a b) tail");
    let mut stream = document.root();
    stream.consume_function_named("f").unwrap();
    let checkpoint = stream.mark();
    let error = stream
        .within_block(|nested| {
            nested.consume_keyword("a")?;
            nested.consume_keyword("wrong")?;
            Ok::<_, Failed<'_>>(())
        })
        .unwrap_err();
    // The error is about `b`, which is where the grammar went wrong.
    assert_eq!(error.at().spelling(), Some("b"));
    assert_eq!(error.unexpected_token(), Some(Token::Ident("b")));
    // Entering consumed the group even though the grammar failed.
    assert_eq!(stream.consume_ident().ok(), Some("tail"));
    stream.restore(checkpoint);
    stream
        .within_block(|nested| {
            nested.consume_keyword("a")?;
            nested.consume_keyword("b")?;
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
    assert_eq!(stream.consume_ident().ok(), Some("tail"));
}

#[test]
fn the_error_at_the_end_of_a_block_is_its_closing_token_and_at_the_end_of_the_root_the_text_end() {
    let document = parse("f()");
    let mut stream = document.root();
    stream.consume_function_named("f").unwrap();
    let error = stream
        .within_block(|nested| Ok::<_, Failed<'_>>(nested.consume_ident()?))
        .unwrap_err();
    assert!(error.is_end_of_stream());
    let at = error.at();
    assert_eq!(at.offset(), Some(Offset::new(2)));
    // The boundary is in the function's own list, before its end.
    assert!(!at.is_closing());
    assert_eq!(at.node(), None);
    let closing = document.node_at_offset(Offset::new(2)).unwrap();
    assert!(closing.is_closing());
    assert_eq!(closing.token(), Some(Token::CloseParen));
    assert_eq!(closing.spelling(), Some(")"));
    // At the end of the root the stream's end is the end of the text.
    let error = stream.consume_ident().unwrap_err();
    assert!(error.is_end_of_stream());
    assert_eq!(error.at().offset(), Some(Offset::new(3)));
}

#[test]
fn the_closing_site_of_a_group_the_input_ended_has_no_token() {
    let document = parse("(a");
    let group = document.list(document.root_list())[0];
    assert!(matches!(
        document.group_end(group),
        Some(GroupEnd::Eof { at: Some(_) })
    ));
    // No token starts at the end of the text.
    assert!(document.node_at_offset(Offset::new(2)).is_none());
}

#[test]
fn lexical_error_tokens_stay_visible_until_a_grammar_visits_them() {
    // `url(a b)` is a bad url (section 4.3.6); the `!` after it is a delimiter.
    let document = parse("url(a b)!");
    let mut stream = document.root();
    assert_eq!(stream.consume_after_whitespace().unwrap(), Token::BadUrl);
    assert_eq!(
        stream.consume_after_whitespace().unwrap(),
        Token::Delim('!')
    );
    assert!(stream.is_empty());
}

// ----------------------------------------------------------------- lists (5.5.7)

#[test]
fn a_list_ends_at_the_first_stop_token_of_its_own_list_and_leaves_it_unconsumed() {
    let document = parse("a f(b;c);tail");
    let mut stream = document.root();
    let seen = stream
        .within_list_until(StopToken::Semicolon, |nested| {
            nested.consume_keyword("a")?;
            nested.consume_function_named("f")?;
            // The semicolon inside the function is not this list's.
            Ok::<_, Failed<'_>>(nested.start_site().offset())
        })
        .unwrap();
    assert_eq!(seen, Some(Offset::new(0)));
    // The stream sits at the real outer delimiter.
    assert_eq!(stream.site().spelling(), Some(";"));
    stream.consume_semicolon().unwrap();
    assert_eq!(stream.consume_ident().ok(), Some("tail"));
}

#[test]
fn every_stop_token_ends_a_list_and_only_the_ones_asked_for() {
    for (text, stop, other) in [
        ("a, b", StopToken::Comma, StopToken::Semicolon),
        ("a; b", StopToken::Semicolon, StopToken::Comma),
        ("a ! b", StopToken::ExclamationMark, StopToken::Comma),
        ("a {b} c", StopToken::OpenCurly, StopToken::Semicolon),
    ] {
        let document = parse(text);
        let mut stream = document.root();
        stream
            .within_list_until(stop, |nested| {
                nested.consume_ident()?;
                Ok::<_, Failed<'_>>(())
            })
            .unwrap();
        assert!(!stream.is_empty(), "{text:?}: stopped before the end");
        // Another stop token does not end the list.
        let mut stream = document.root();
        let mut seen = Vec::new();
        stream
            .within_list_until(other, |nested| {
                seen = kinds(nested);
                Ok::<_, Failed<'_>>(())
            })
            .unwrap();
        assert!(stream.is_empty(), "{text:?}: {other:?} stopped the list");
        assert!(seen.len() >= 3, "{text:?}: {seen:?}");
    }
}

#[test]
fn a_set_of_stop_tokens_ends_at_the_first_of_any() {
    let stops = StopToken::Comma | StopToken::Semicolon | StopToken::OpenCurly;
    assert!(stops.contains(StopToken::Comma) && !stops.contains(StopToken::ExclamationMark));
    assert!(!stops.is_empty() && StopTokens::NONE.is_empty());
    let document = parse("a b ; c , d");
    let mut stream = document.root();
    stream
        .within_list_until(stops, |nested| {
            nested.consume_ident()?;
            nested.consume_ident()?;
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
    assert_eq!(stream.site().spelling(), Some(";"));
    // A set with no stop token never stops early.
    let mut stream = document.root();
    stream
        .within_list_until(StopTokens::NONE, |nested| {
            assert_eq!(kinds(nested).len(), 11);
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
    assert!(stream.is_empty());
    assert_eq!(
        StopTokens::from(StopToken::Comma),
        StopTokens::NONE | StopToken::Comma
    );
    assert_eq!(
        StopToken::Comma | StopToken::Comma,
        StopTokens::from(StopToken::Comma)
    );
    assert_eq!(
        (StopToken::Comma | StopToken::Semicolon) | (StopToken::Semicolon | StopToken::OpenCurly),
        stops
    );
}

#[test]
fn a_curly_block_whether_opened_or_denied_is_the_open_curly_stop_token() {
    let limits = Limits::new(0, usize::MAX, usize::MAX);
    let document = Document::<()>::parse(
        "a {b} c",
        &Options::default(),
        limits,
        DepthExcess::DenyBlock,
    )
    .unwrap();
    let mut stream = document.root();
    stream
        .within_list_until(StopToken::OpenCurly, |nested| {
            nested.consume_ident()?;
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
    assert_eq!(stream.next_token(), Some(Token::OpenCurly));
    // A paren block is not a curly one.
    let document = parse("a (b) c");
    let mut stream = document.root();
    stream
        .within_list_until(StopToken::OpenCurly, |nested| {
            assert_eq!(kinds(nested).len(), 5);
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
}

#[test]
fn a_list_part_must_be_empty_when_the_grammar_is_done() {
    let document = parse("a b, c");
    let mut stream = document.root();
    let error = stream
        .within_list_until(StopToken::Comma, |nested| {
            nested.consume_ident()?;
            Ok::<_, Failed<'_>>(())
        })
        .unwrap_err();
    assert_eq!(error.unexpected_token(), Some(Token::Ident("b")));
    // Even so the stream moved to the stop token.
    assert_eq!(stream.next_token(), Some(Token::Comma));
}

#[test]
fn the_nested_stream_covers_its_part_with_its_own_start_and_end() {
    let document = parse("a b; c");
    let mut stream = document.root();
    stream
        .within_list_until(StopToken::Semicolon, |nested| {
            assert_eq!(nested.start_site().offset(), Some(Offset::new(0)));
            assert_eq!(nested.end_site().offset(), Some(Offset::new(3)));
            assert_eq!(nested.offset(), Some(Offset::new(0)));
            nested.consume_token()?;
            assert_eq!(nested.offset(), Some(Offset::new(1)));
            assert_eq!(nested.text_since(nested.start_site()), Some("a"));
            kinds(nested);
            assert_eq!(nested.site(), nested.end_site());
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
}

#[test]
fn a_comma_segments_hands_the_grammar_every_segment_including_empty_ones() {
    fn names<'i>(stream: &mut Stream<'i>) -> Result<&'i str, Failed<'i>> {
        Ok(stream.consume_ident()?)
    }
    let document = parse("a, b ,c");
    assert_eq!(
        document.root().comma_segments(names).unwrap(),
        ["a", "b", "c"]
    );
    // One segment without a comma.
    assert_eq!(parse(" a ").root().comma_segments(names).unwrap(), ["a"]);
    // An empty segment is the grammar's to judge: here it fails.
    let document = parse("a,,b");
    let error = document.root().comma_segments(names).unwrap_err();
    assert!(error.is_end_of_stream());
    assert_eq!(error.at().offset(), Some(Offset::new(2)));
    let document = parse("a,");
    assert!(
        document
            .root()
            .comma_segments(names)
            .unwrap_err()
            .is_end_of_stream()
    );
    // An empty stream is one empty segment.
    assert!(
        parse("")
            .root()
            .comma_segments(names)
            .unwrap_err()
            .is_end_of_stream()
    );
    // A grammar that accepts an empty segment sees them all.
    let segments = parse(",a,")
        .root()
        .comma_segments(|segment| {
            let empty = segment.only_whitespace_remains();
            segment.discard_whitespace();
            if !empty {
                segment.consume_ident()?;
            }
            Ok::<_, Failed<'_>>(empty)
        })
        .unwrap();
    assert_eq!(segments, [true, false, true]);
    // Commas inside groups do not split.
    let document = parse("f(a, b), c");
    let lengths = document
        .root()
        .comma_segments(|segment| {
            let count = kinds(segment).len();
            Ok::<_, Failed<'_>>(count)
        })
        .unwrap();
    assert_eq!(lengths, [1, 2]);
}

#[test]
fn a_comma_segments_leaves_the_stream_at_the_end() {
    let document = parse("a, b");
    let mut stream = document.root();
    stream
        .comma_segments(|segment| Ok::<_, Failed<'_>>(segment.consume_ident()?))
        .unwrap();
    assert!(stream.is_empty());
}

// ----------------------------------------------------------------- source text

#[test]
fn the_text_between_two_sites_is_the_source_as_it_was_written() {
    let document = parse("a  \\62  f(c)");
    let mut stream = document.root();
    let start = stream.site();
    stream.consume_ident().unwrap();
    let after_a = stream.site();
    stream.consume_after_whitespace().unwrap();
    let after_b = stream.site();
    assert_eq!(stream.text_between(start, after_a), Some("a"));
    assert_eq!(stream.text_between(after_a, after_b), Some("  \\62 "));
    assert_eq!(stream.text_since(start), Some("a  \\62 "));
    assert_eq!(stream.text_between(start, start), Some(""));
    // Backwards is nothing.
    assert_eq!(stream.text_between(after_b, start), None);
    // The text is the whole span, a group included.
    stream.consume_after_whitespace().unwrap();
    assert_eq!(
        stream.text_between(stream.start_site(), stream.end_site()),
        Some("a  \\62  f(c)")
    );
    let inside = document.list(document.root_list())[4];
    let group = document.stream(document.children(inside).unwrap());
    assert_eq!(
        group.text_between(group.start_site(), group.end_site()),
        Some("c")
    );
}

#[test]
fn text_is_unavailable_for_components_and_for_sites_of_another_document() {
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    builder.leaf(root, Token::Ident("a"), None, None).unwrap();
    let built = builder.finish(None);
    let stream = built.root();
    assert_eq!(
        stream.text_between(stream.start_site(), stream.end_site()),
        None
    );
    assert_eq!(stream.offset(), None);
    assert_eq!(stream.site().spelling(), None);
    // A site of another document says nothing here.
    let one = parse("a");
    let two = parse("a");
    let stream = one.root();
    assert_eq!(
        stream.text_between(two.root().start_site(), stream.end_site()),
        None
    );
}

#[test]
fn a_site_reports_where_it_is_the_token_and_the_payload_free_facts() {
    let document = parse("f(a)");
    let mut stream = document.root();
    let before = stream.site();
    assert_eq!(before.list(), document.root_list());
    assert_eq!(before.index(), 0);
    assert!(!before.is_closing());
    assert_eq!(before.node(), Some(document.list(document.root_list())[0]));
    assert_eq!(before.token(), Some(Token::Function("f")));
    assert_eq!(
        before.token().map(|token| token.kind()),
        Some(TokenKind::Function)
    );
    assert_eq!(before.spelling(), Some("f("));
    assert_eq!(before.offset(), Some(Offset::new(0)));
    assert_eq!(before.payload(), None);
    assert!(std::ptr::eq(before.document(), &document));
    stream.consume_token().unwrap();
    let after = stream.site();
    assert_eq!(after.index(), 1);
    assert_eq!(after.node(), None);
    assert_eq!(after.token(), None);
    assert_eq!(after.offset(), Some(Offset::new(4)));
    assert_ne!(before, after);
    let same = before;
    assert_eq!(before, same);
}

// ----------------------------------------------------------------- errors

#[test]
fn the_error_constructors_build_the_errors_a_grammar_returns() {
    let document = parse("a 1");
    let mut stream = document.root();
    stream.consume_ident().unwrap();
    let site = stream.site();
    let end: Failed<'_> = stream.end_of_stream();
    assert!(end.is_end_of_stream() && end.is_stream_error());
    assert_eq!(end.at(), site);
    let unexpected: SyntaxError<'_, &str> = stream.unexpected(Token::Comma);
    assert!(unexpected.is_unexpected_token());
    assert_eq!(unexpected.unexpected_token(), Some(Token::Comma));
    assert_eq!(unexpected.at(), site);
    assert_eq!(
        unexpected.stream_error().map(|error| error.at()),
        Some(site)
    );
    assert!(unexpected.to_string().contains("unexpected"));

    #[derive(Debug, PartialEq)]
    struct Detail(u8);
    impl From<u8> for Detail {
        fn from(value: u8) -> Self {
            Self(value)
        }
    }
    let custom: SyntaxError<'_, Detail> = stream.grammar_error(7_u8);
    assert!(custom.is_custom());
    assert!(
        !custom.is_stream_error() && !custom.is_unexpected_token() && !custom.is_end_of_stream()
    );
    assert_eq!(custom.at(), site);
    assert_eq!(custom.custom_value(), Some(&Detail(7)));
    assert_eq!(custom.unexpected_token(), None);
    assert!(custom.stream_error().is_none());
    assert_eq!(custom.into_custom().ok(), Some(Detail(7)));
    // A failure that is not the grammar's own comes back unchanged.
    let back = unexpected.into_custom().unwrap_err();
    assert!(back.is_unexpected_token());
}

#[test]
fn a_custom_error_is_built_at_a_site_and_a_stream_error_converts_with_the_question_mark() {
    let document = parse("a");
    let site = document.root().site();
    let custom = SyntaxError::custom(site, "not a color");
    assert_eq!(custom.custom_value(), Some(&"not a color"));
    assert_eq!(custom.to_string(), "not a color");
    assert!(format!("{custom:?}").contains("not a color"));
    fn converts<'i>(stream: &mut Stream<'i>) -> Result<(), SyntaxError<'i, &'static str>> {
        stream.consume_comma()?;
        Ok(())
    }
    let error = converts(&mut document.root()).unwrap_err();
    assert!(error.is_unexpected_token());
}

#[test]
fn the_rule_protocol_errors_report_their_kind_and_the_name_they_reject() {
    let document = parse("@x");
    let mut stream = document.root();
    let site = stream.site();
    let Ok(Token::AtKeyword(name)) = stream.consume_token() else {
        panic!("an at-keyword");
    };
    let unknown: SyntaxError<'_, &str> = SyntaxError::unknown_at_rule_error(site, name);
    assert!(unknown.is_unknown_at_rule());
    assert_eq!(unknown.unknown_at_rule(), Some("x"));
    assert!(unknown.to_string().contains("@x"));
    let body: SyntaxError<'_, &str> = SyntaxError::invalid_at_rule_body_error(site);
    assert!(body.is_invalid_at_rule_body() && !body.is_unknown_at_rule());
    assert_eq!(body.unknown_at_rule(), None);
    let qualified: SyntaxError<'_, &str> = SyntaxError::invalid_qualified_rule_error(site);
    assert!(qualified.is_invalid_qualified_rule() && !qualified.is_invalid_at_rule_body());
    for error in [&unknown, &body, &qualified] {
        assert_eq!(error.at(), site);
        assert!(!error.is_custom() && !error.is_stream_error());
        assert!(!error.is_end_of_stream() && !error.is_refused() && !error.is_block_expected());
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn errors_are_standard_errors_and_the_stream_error_is_comparable() {
    fn is_error<T: std::error::Error>(_: &T) {}
    let document = parse("1");
    let mut stream = document.root();
    let error = stream.consume_ident().unwrap_err();
    is_error(&error);
    assert_eq!(error, error.clone());
    let wrapped: SyntaxError<'_, &str> = error.clone().into();
    is_error(&wrapped);
    assert_eq!(wrapped.stream_error(), Some(&error));
    assert!(!format!("{error:?}").is_empty());
}

#[test]
fn the_error_a_grammar_returns_stays_small_however_large_its_own_error_is() {
    // The grammar's own error is boxed, so the failure does not grow with it.
    let small = std::mem::size_of::<SyntaxError<'static, ()>>();
    let large = std::mem::size_of::<SyntaxError<'static, [u8; 4096]>>();
    assert_eq!(small, large);
    assert!(small <= 128, "{small} bytes");
}
