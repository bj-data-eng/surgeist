//! Tests for metering through the document: the steps a stream takes for each
//! observation, what a refusing host sees, and the balance of the storage a
//! metered build charges.
//!
//! The expected sequences are derived by hand from the documented rules: a
//! stream takes one [`Meter::step`] for each node it examines (a comment it
//! skips, whitespace, or the end of the stream), and a build takes one step
//! for each token it adds and charges every growth of its vectors through
//! `limits::grow`, which returns every byte it charged.

mod common;

use common::tree::{Call, Recording};
use surgeist_css_parser::anb;
use surgeist_css_parser::document::{BuildError, GroupEnd};
use surgeist_css_parser::limits::Limits;
use surgeist_css_parser::{Document, Stream, SyntaxError, Token};

type Failed<'i> = SyntaxError<'i, ()>;

/// A document of `tokens` in the root list, built under `meter`, with the
/// meter's record emptied afterwards so a test sees only what a stream does.
fn metered<'m>(meter: &'m Recording, tokens: &[Token<'m>]) -> Document<'m> {
    meter.admit_all();
    let mut builder = Document::<()>::builder(Limits::default(), 0, Some(meter)).unwrap();
    let root = builder.root();
    for token in tokens {
        builder.leaf(root, *token, None, None).unwrap();
    }
    let document = builder.finish(None);
    meter.forget();
    document
}

fn steps(count: usize) -> Vec<Call> {
    vec![Call::Step; count]
}

type Operation = fn(&mut Stream<'_>);

#[test]
fn a_stream_takes_one_step_for_each_node_it_examines() {
    use Token::{Comment, Ident, Whitespace};
    // (the tokens of the stream, what is done, the steps it takes, what it shows)
    let cases: [(&[Token<'static>], Operation, usize, &str); 12] = [
        (
            &[Ident("a")],
            |s| {
                let _ = s.next_token();
            },
            1,
            "next_token on a token",
        ),
        (
            &[],
            |s| {
                let _ = s.next_token();
            },
            1,
            "next_token at the end",
        ),
        (
            &[Ident("a")],
            |s| {
                let _ = s.consume_token();
            },
            1,
            "consume_token",
        ),
        (
            &[Comment, Ident("a")],
            |s| {
                let _ = s.consume_token();
            },
            2,
            "a comment is skipped at the cost of a step",
        ),
        (
            &[Comment, Ident("a")],
            |s| {
                let _ = s.consume_token_with_comments();
            },
            1,
            "a comment consumed on request",
        ),
        (
            &[Whitespace, Ident("a")],
            |s| {
                let _ = s.consume_after_whitespace();
            },
            3,
            "whitespace and the stop, then the consume",
        ),
        (
            &[Whitespace, Whitespace, Ident("a")],
            |s| s.discard_whitespace(),
            3,
            "two whitespace tokens and the token that stops it",
        ),
        (
            &[],
            |s| s.discard_whitespace(),
            0,
            "discarding at the end examines nothing",
        ),
        (
            &[Whitespace, Whitespace],
            |s| {
                let _ = s.only_whitespace_remains();
            },
            3,
            "whitespace, whitespace, the end",
        ),
        (
            &[Ident("a")],
            |s| {
                let _ = s.only_whitespace_remains();
            },
            1,
            "a token stops the scan",
        ),
        (
            &[Whitespace],
            |s| {
                let _ = s.require_empty();
            },
            2,
            "discard the whitespace, then find the end",
        ),
        (
            &[Ident("a")],
            |s| {
                let _ = s.require_empty();
            },
            2,
            "discard stops at the token, then the token is found",
        ),
    ];
    for (tokens, operation, expected, what) in cases {
        let meter = Recording::admitting();
        let document = metered(&meter, tokens);
        let mut stream = document.root();
        operation(&mut stream);
        assert_eq!(meter.calls(), steps(expected), "{what}");
    }
}

#[test]
fn a_stream_without_a_meter_takes_no_steps_and_offers_none() {
    let document: Document = common::tree::parse("a b");
    let mut stream = document.root();
    assert!(stream.meter().is_none());
    assert!(document.meter().is_none());
    stream.consume_ident().unwrap();
    assert!(stream.consume_ident().is_ok());
}

#[test]
fn the_stream_offers_the_documents_meter_for_a_grammar_that_builds_storage() {
    let meter = Recording::admitting();
    let document = metered(&meter, &[Token::Ident("a")]);
    let stream = document.root();
    stream
        .meter()
        .expect("the document's meter")
        .step()
        .unwrap();
    document
        .meter()
        .expect("the document's meter")
        .step()
        .unwrap();
    assert_eq!(meter.calls(), steps(2));
}

#[test]
fn a_refused_step_is_a_refusal_error_that_moves_nothing() {
    let meter = Recording::admitting();
    for attempt in 0..4 {
        let document = metered(
            &meter,
            &[Token::Ident("a"), Token::Whitespace, Token::Ident("b")],
        );
        // Only the first call is refused. `consume_ident` and the other
        // consumers discard whitespace first, which stops quietly at a refusal,
        // and their own step then finds the document latched.
        meter.refuse_call(0);
        let mut stream = document.root();
        let before = stream.site();
        let error = match attempt {
            0 => stream.consume_token().unwrap_err(),
            1 => stream.consume_token_with_comments().unwrap_err(),
            2 => stream.consume_ident().unwrap_err(),
            _ => stream.consume_comma().unwrap_err(),
        };
        assert!(error.is_refused(), "attempt {attempt}");
        assert!(!error.is_end_of_stream() && !error.is_unexpected_token());
        assert_eq!(error.unexpected_token(), None);
        assert_eq!(error.at(), before);
        assert_eq!(stream.site(), before);
        assert!(error.to_string().contains("refused"));
    }
}

#[test]
fn the_observations_that_cannot_fail_read_a_refusal_as_the_end_and_say_so() {
    let meter = Recording::admitting();
    for what in 0..3 {
        let document = metered(&meter, &[Token::Ident("a")]);
        meter.refuse_call(0);
        let stream = document.root();
        match what {
            0 => assert_eq!(stream.next_token(), None),
            1 => assert!(stream.is_empty()),
            _ => assert!(stream.only_whitespace_remains()),
        }
        // The fallback is the end of the stream, and the refusal is visible.
        assert!(stream.is_refused(), "observation {what}");
    }
    // Discarding stops at a refusal and drops no token.
    let document = metered(
        &meter,
        &[Token::Whitespace, Token::Whitespace, Token::Ident("a")],
    );
    meter.refuse_call(1);
    let mut stream = document.root();
    stream.discard_whitespace();
    assert_eq!(stream.site().index(), 1);
    assert!(stream.is_refused());
}

#[test]
fn a_refused_require_empty_restores_the_stream() {
    let meter = Recording::admitting();
    let document = metered(&meter, &[Token::Whitespace]);
    meter.refuse_call(1);
    let mut stream = document.root();
    let before = stream.site();
    let error = stream.require_empty().unwrap_err();
    assert!(error.is_refused());
    assert_eq!(stream.site(), before);
}

/// A function holding `a, b`, under `meter`.
fn function_with_arguments(meter: &Recording) -> Document<'_> {
    meter.admit_all();
    let mut builder = Document::<()>::builder(Limits::default(), 0, Some(meter)).unwrap();
    let root = builder.root();
    let (function, inside) = builder
        .open(root, Token::Function("f"), None, None)
        .unwrap();
    for token in [
        Token::Ident("a"),
        Token::Comma,
        Token::Whitespace,
        Token::Ident("b"),
    ] {
        builder.leaf(inside, token, None, None).unwrap();
    }
    builder
        .close(
            function,
            GroupEnd::Closed {
                range: None,
                payload: None,
            },
        )
        .unwrap();
    let document = builder.finish(None);
    meter.forget();
    document
}

fn arguments<'i>(stream: &mut Stream<'i>) -> Result<Vec<&'i str>, Failed<'i>> {
    stream.parse_complete(|stream| {
        stream.consume_function_named("f")?;
        stream.within_block(|inside| {
            inside.comma_segments(|item| Ok::<_, Failed<'_>>(item.consume_ident()?))
        })
    })
}

#[test]
fn a_refusal_anywhere_in_a_grammar_is_a_refusal_error_never_another_failure() {
    // A function, a comma list and a complete parse over a metered document:
    // refuse each fallible call in turn. Every run either parses to the right
    // result or fails with a refusal, and none panics.
    let meter = Recording::admitting();
    let document = function_with_arguments(&meter);
    arguments(&mut document.root()).unwrap();
    let total = meter.fallible_calls();
    assert!(total > 10, "the grammar made {total} fallible calls");
    drop(document);
    for sticky in [false, true] {
        let mut refusals = 0;
        for index in 0..=total {
            // A fresh document each time: a refusal is latched on the document.
            let document = function_with_arguments(&meter);
            if sticky {
                meter.refuse_from(index);
            } else {
                // A host that would admit the next call after refusing one.
                meter.refuse_call(index);
            }
            match arguments(&mut document.root()) {
                Ok(names) => assert_eq!(names, ["a", "b"], "refusing call {index}"),
                Err(error) => {
                    assert!(error.is_refused(), "refusing call {index}: {error:?}");
                    assert!(document.is_refused());
                    refusals += 1;
                }
            }
        }
        // Every call before the last is refused as the run goes through; the
        // call after the last never happens.
        assert_eq!(refusals, total, "sticky host: {sticky}");
    }
}

#[test]
fn a_comma_list_on_a_metered_root_stream_is_never_truncated_by_a_refusal() {
    // `a, b` directly under `comma_segments`: refuse each call in turn, as a
    // host that stays refused and as one that would admit the next call. The
    // result is the whole list or a refusal, never a shorter list.
    fn names<'i>(stream: &mut Stream<'i>) -> Result<Vec<&'i str>, Failed<'i>> {
        stream.comma_segments(|item| Ok::<_, Failed<'_>>(item.consume_ident()?))
    }
    let meter = Recording::admitting();
    let tokens = [
        Token::Ident("a"),
        Token::Comma,
        Token::Whitespace,
        Token::Ident("b"),
    ];
    let document = metered(&meter, &tokens);
    assert_eq!(names(&mut document.root()).unwrap(), ["a", "b"]);
    let total = meter.fallible_calls();
    drop(document);
    for sticky in [false, true] {
        for index in 0..total {
            let document = metered(&meter, &tokens);
            if sticky {
                meter.refuse_from(index);
            } else {
                meter.refuse_call(index);
            }
            let error = names(&mut document.root()).unwrap_err();
            assert!(error.is_refused(), "refusing call {index}: {error:?}");
            assert!(document.is_refused());
        }
    }
}

#[test]
fn after_a_refusal_the_document_stays_refused_without_asking_the_meter_again() {
    let meter = Recording::admitting();
    let document = metered(&meter, &[Token::Ident("a"), Token::Whitespace]);
    let mut stream = document.root();
    assert!(!stream.is_refused() && !document.is_refused());
    // A host that would admit everything after its first refusal.
    meter.refuse_call(0);
    assert_eq!(stream.next_token(), None);
    assert!(stream.is_refused() && document.is_refused());
    assert_eq!(meter.calls(), steps(1));
    // Every later observation, of this stream or another, fails the same way
    // and the meter hears nothing more.
    let other = document.root();
    assert_eq!(other.next_token(), None);
    assert!(stream.is_empty());
    assert!(stream.only_whitespace_remains());
    stream.discard_whitespace();
    assert_eq!(stream.site().index(), 0);
    assert!(stream.consume_token().unwrap_err().is_refused());
    assert!(stream.require_empty().unwrap_err().is_refused());
    assert!(stream.consume_after_whitespace().unwrap_err().is_refused());
    let error: SyntaxError<'_, u64> = stream.grammar_error(1_u32);
    assert!(error.is_refused());
    assert_eq!(
        meter.calls(),
        steps(1),
        "no call reached the meter after the refusal"
    );
    // Looking at the end of a stream is not a refusal.
    let fine = Recording::admitting();
    let document = metered(&fine, &[Token::Ident("a")]);
    let mut stream = document.root();
    stream.consume_ident().unwrap();
    assert!(stream.is_empty() && stream.only_whitespace_remains());
    assert!(!stream.is_refused() && !document.is_refused());
}

#[test]
fn the_search_for_a_stop_token_takes_a_step_for_each_node_it_examines() {
    let tokens = [
        Token::Ident("a"),
        Token::Whitespace,
        Token::Semicolon,
        Token::Ident("b"),
    ];
    let meter = Recording::admitting();
    let document = metered(&meter, &tokens);
    let mut stream = document.root();
    stream
        .within_list_until(
            surgeist_css_parser::stream::StopToken::Semicolon,
            |inside| {
                inside.consume_ident()?;
                Ok::<_, Failed<'_>>(())
            },
        )
        .unwrap();
    // The scan looks at `a`, the whitespace and the semicolon: 3 steps. `consume_ident`
    // takes the step that ends the discard and its own: 2. `require_empty` takes the
    // whitespace and the end: 2.
    assert_eq!(meter.calls(), steps(7));
    // Without a stop token the scan looks at every node.
    let meter = Recording::admitting();
    let document = metered(&meter, &tokens);
    let mut stream = document.root();
    stream
        .within_list_until(surgeist_css_parser::stream::StopTokens::NONE, |inside| {
            while inside.consume_token().is_ok() {}
            Ok::<_, Failed<'_>>(())
        })
        .unwrap();
    // 4 for the scan, 5 for consuming four tokens and finding the end, 1 for `require_empty`.
    assert_eq!(meter.calls(), steps(10));
}

#[test]
fn a_refused_search_for_a_stop_token_does_not_move_the_stream() {
    let meter = Recording::admitting();
    let document = metered(
        &meter,
        &[Token::Ident("a"), Token::Comma, Token::Ident("b")],
    );
    let mut stream = document.root();
    let before = stream.site();
    meter.refuse_call(1);
    let error = stream
        .within_list_until(surgeist_css_parser::stream::StopToken::Comma, |_| {
            Ok::<_, Failed<'_>>(())
        })
        .unwrap_err();
    assert!(error.is_refused());
    assert_eq!(stream.site(), before);
}

#[test]
fn a_comma_segments_asks_the_meter_to_grow_its_result_and_reports_a_refusal() {
    let meter = Recording::admitting();
    let document = function_with_arguments(&meter);
    fn run<'i>(stream: &mut Stream<'i>) -> Result<Vec<&'i str>, Failed<'i>> {
        stream.consume_function_named("f").unwrap();
        stream.within_block(|inside| {
            inside.comma_segments(|item| Ok::<_, Failed<'_>>(item.consume_ident()?))
        })
    }
    run(&mut document.root()).unwrap();
    let calls = meter.calls();
    // The fallible calls before the first growth charge of the result.
    let first_add = calls
        .iter()
        .filter(|call| matches!(call, Call::Step | Call::Add(_)))
        .position(|call| matches!(call, Call::Add(_)))
        .expect("the result vector is charged");
    meter.refuse_call(first_add);
    let error = run(&mut document.root()).unwrap_err();
    assert!(error.is_refused());
}

#[test]
fn a_grammar_error_is_charged_to_the_meter_as_the_size_of_the_error_type() {
    let meter = Recording::admitting();
    let document = metered(&meter, &[Token::Ident("a")]);
    let stream = document.root();
    let error: SyntaxError<'_, u64> = stream.grammar_error(7_u32);
    // `u64: From<u32>`; the box holds a `u64`, charged as eight bytes.
    assert_eq!(meter.calls(), [Call::Add(8)]);
    assert_eq!(error.custom_value(), Some(&7_u64));
    // A refusal is reported instead of the error.
    meter.refuse_call(0);
    let error: SyntaxError<'_, u64> = stream.grammar_error(7_u32);
    assert!(error.is_refused());
    assert!(!error.is_custom());
}

// ----------------------------------------------------------------- the build

/// A function with arguments and a block, a stray closer, an unclosed group
/// and a sibling, all under `meter`.
fn build(meter: &Recording) -> Result<Document<'_>, BuildError> {
    let mut builder = Document::<()>::builder(Limits::default(), 0, Some(meter))?;
    let root = builder.root();
    let (function, inside) = builder.open(root, Token::Function("f"), None, None)?;
    builder.leaf(inside, Token::Ident("a"), None, None)?;
    let (block, within) = builder.open(inside, Token::OpenSquare, None, None)?;
    builder.leaf(within, Token::Delim('+'), None, None)?;
    builder.close(
        block,
        GroupEnd::Closed {
            range: None,
            payload: None,
        },
    )?;
    builder.leaf(inside, Token::CloseCurly, None, None)?;
    builder.close(function, GroupEnd::Eof { at: None })?;
    builder.leaf(root, Token::Ident("b"), None, None)?;
    builder.charge_spelling(5)?;
    Ok(builder.finish(None))
}

#[test]
fn a_metered_build_charges_storage_and_returns_all_of_it_when_the_document_is_dropped() {
    let meter = Recording::admitting();
    let document = build(&meter).unwrap();
    assert!(meter.live() > 0, "the build holds charged storage");
    assert_eq!(document.node_count(), 6);
    assert_eq!(document.metrics().components(), 6);
    assert_eq!(document.metrics().spelling_bytes(), 5);
    drop(document);
    assert_eq!(meter.live(), 0, "every charged byte is returned on drop");
}

#[test]
fn a_build_takes_a_step_for_a_token_before_it_grows_anything() {
    let meter = Recording::admitting();
    let mut builder = Document::<()>::builder(Limits::default(), 0, Some(&meter)).unwrap();
    meter.forget();
    let root = builder.root();
    builder.leaf(root, Token::Ident("a"), None, None).unwrap();
    let calls = meter.calls();
    assert_eq!(calls[0], Call::Step, "the token's own step comes first");
    assert!(
        calls[1..]
            .iter()
            .any(|call| matches!(call, Call::Add(bytes) if *bytes > 0)),
        "the node and list vectors are charged: {calls:?}"
    );
    // Adding to a vector that has room costs one step and no storage.
    meter.forget();
    builder.leaf(root, Token::Ident("b"), None, None).unwrap();
    assert_eq!(
        meter.calls(),
        steps(3),
        "the token's step, then the two vectors' steps"
    );
}

#[test]
fn a_build_refused_at_any_call_fails_with_a_refusal_and_returns_everything_it_charged() {
    let counting = Recording::admitting();
    drop(build(&counting).unwrap());
    let total = counting.fallible_calls();
    assert!(total > 20, "the build made {total} fallible calls");
    for index in 0..total {
        let meter = Recording::refusing_call(index);
        let error = build(&meter).unwrap_err();
        assert!(error.is_refused(), "refusing call {index}");
        assert_eq!(meter.live(), 0, "refusing call {index} leaked storage");
    }
    // Refusing past the last call changes nothing.
    let meter = Recording::refusing_call(total);
    drop(build(&meter).unwrap());
    assert_eq!(meter.live(), 0);
}

#[test]
fn the_storage_of_a_metered_build_is_charged_as_it_grows_and_never_exceeds_what_was_added() {
    // Doubling growth from four elements: the live amount only ever rises by an
    // add and falls by a release, and at the end it is what the vectors hold.
    let meter = Recording::admitting();
    let mut builder = Document::<()>::builder(Limits::default(), 0, Some(&meter)).unwrap();
    let root = builder.root();
    let mut previous = meter.live();
    for _ in 0..40 {
        builder.leaf(root, Token::Comma, None, None).unwrap();
        let live = meter.live();
        assert!(
            live >= previous,
            "storage does not shrink while the build grows"
        );
        previous = live;
    }
    drop(builder.finish(None));
    assert_eq!(meter.live(), 0);
}

#[test]
fn a_document_finished_after_a_refused_build_step_reports_the_refusal() {
    // A caller that ignores the error of a builder call and finishes the build
    // gets a document that is latched: its streams fail instead of running
    // against the host, even if the host would admit the next call.
    let counting = Recording::admitting();
    let (start, total) = {
        let mut builder = Document::<()>::builder(Limits::default(), 0, Some(&counting)).unwrap();
        // The calls that make the builder itself cannot be ignored: it is not returned.
        let start = counting.fallible_calls();
        let root = builder.root();
        builder.leaf(root, Token::Ident("a"), None, None).unwrap();
        builder.leaf(root, Token::Comma, None, None).unwrap();
        (start, counting.fallible_calls())
    };
    assert!(total > start + 4);
    let mut refused_at_least_once = 0;
    for index in start..total {
        let meter = Recording::refusing_call(index);
        let mut builder = Document::<()>::builder(Limits::default(), 0, Some(&meter)).unwrap();
        let root = builder.root();
        let first = builder.leaf(root, Token::Ident("a"), None, None);
        let second = builder.leaf(root, Token::Comma, None, None);
        let document = builder.finish(None);
        assert!(
            first.is_err() || second.is_err(),
            "call {index} was refused"
        );
        assert!(document.is_refused(), "call {index}");
        let mut stream = document.root();
        assert!(stream.is_refused());
        let calls = meter.calls().len();
        assert!(stream.consume_token().unwrap_err().is_refused());
        assert_eq!(meter.calls().len(), calls, "the meter is not asked again");
        refused_at_least_once += 1;
    }
    assert_eq!(refused_at_least_once, total - start);
    // A build nothing refused is not latched.
    let meter = Recording::admitting();
    let mut builder = Document::<()>::builder(Limits::default(), 0, Some(&meter)).unwrap();
    let root = builder.root();
    builder.leaf(root, Token::Ident("a"), None, None).unwrap();
    assert!(!builder.finish(None).is_refused());
}

#[test]
fn a_recognized_plus_n_notation_takes_the_steps_of_its_tokens_only() {
    use Token::{Comment, Delim, Ident};
    // `+n`: discarding whitespace examines `+` (1), consuming it examines it
    // again (1), `n` is examined once (1), and the look for a `B` examines the
    // end of the stream (1). A comment between the two is one more node.
    for (tokens, expected) in [
        (&[Delim('+'), Ident("n")][..], 4),
        (&[Delim('+'), Comment, Ident("n")][..], 5),
    ] {
        let meter = Recording::admitting();
        let document = metered(&meter, tokens);
        let mut stream = document.root();
        let value = anb::parse(&mut stream).unwrap();
        assert_eq!((value.a(), value.b()), (1, 0));
        assert_eq!(meter.calls(), steps(expected), "{tokens:?}");
    }
}

#[test]
fn the_next_site_takes_a_step_for_each_node_it_examines() {
    use Token::{Comment, Ident};
    let meter = Recording::admitting();
    let document = metered(&meter, &[Ident("a"), Comment, Comment, Ident("b")]);
    let mut stream = document.root();
    stream.consume_ident().unwrap();
    meter.forget();
    // Two comments passed over and the token stopped at.
    let _ = stream.next_site();
    assert_eq!(meter.calls(), steps(3));
}
