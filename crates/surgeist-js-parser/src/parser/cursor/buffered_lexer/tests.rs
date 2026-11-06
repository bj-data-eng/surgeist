use crate::{
    frontend::{FrontendLimit, FrontendWork},
    lexer::{Lexer, Token, TokenKind},
    parser::cursor::BufferedLexer,
    source::UTF8Input,
};
use surgeist_js_ast::Punctuator;
use surgeist_js_interner::Interner;
use surgeist_js_macros::utf16;

#[test]
fn buffered_requests_charge_independently_of_source_work() {
    let mut work = Vec::new();
    let mut control = |event| {
        work.push(event);
        Ok(())
    };
    let lexer = Lexer::with_control(UTF8Input::new(&b"\n;"[..]), Some(&mut control));
    let mut cursor = BufferedLexer::from(lexer);
    let mut interner = Interner::default();
    let first = TokenKind::from(Punctuator::Semicolon);
    assert_eq!(
        cursor.peek(0, true, &mut interner).unwrap().unwrap().kind(),
        &first
    );
    assert_eq!(
        cursor.peek(0, true, &mut interner).unwrap().unwrap().kind(),
        &first
    );
    assert_eq!(
        cursor.next(true, &mut interner).unwrap().unwrap().kind(),
        &first
    );
    assert!(cursor.peek(0, false, &mut interner).unwrap().is_none());
    assert!(cursor.next(false, &mut interner).unwrap().is_none());

    assert_eq!(work.first(), Some(&FrontendWork::TokenProbe));
    assert_eq!(
        work.iter()
            .filter(|event| **event == FrontendWork::LexicalDispatch)
            .count(),
        2
    );
    let requests: Vec<_> = work
        .into_iter()
        .filter(|event| matches!(event, FrontendWork::TokenProbe | FrontendWork::TokenAdvance))
        .collect();
    assert_eq!(
        requests,
        [
            FrontendWork::TokenProbe,
            FrontendWork::TokenProbe,
            FrontendWork::TokenAdvance,
            FrontendWork::TokenProbe,
            FrontendWork::TokenAdvance
        ]
    );
}

#[test]
fn refused_buffered_requests_do_not_fill_or_change_the_ring() {
    for prefetched in [false, true] {
        let mut probes_remaining = usize::from(prefetched);
        let mut work = Vec::new();
        let mut control = |event| {
            work.push(event);
            match event {
                FrontendWork::TokenProbe if probes_remaining == 0 => {
                    Err(FrontendLimit::TokenProbe(0))
                }
                FrontendWork::TokenProbe => {
                    probes_remaining -= 1;
                    Ok(())
                }
                FrontendWork::TokenAdvance => Err(FrontendLimit::TokenAdvance(0)),
                _ => Ok(()),
            }
        };
        let lexer = Lexer::with_control(UTF8Input::new(&b"\n;"[..]), Some(&mut control));
        let mut cursor = BufferedLexer::from(lexer);
        let mut interner = Interner::default();
        if prefetched {
            assert!(cursor.peek(0, false, &mut interner).unwrap().is_some());
        }
        let slots = cursor.peeked.clone();
        let position = (cursor.read_index, cursor.write_index, cursor.linear_pos());
        assert_eq!(
            cursor
                .peek(0, true, &mut interner)
                .unwrap_err()
                .frontend_limit(),
            Some(FrontendLimit::TokenProbe(0))
        );
        assert_eq!(
            cursor
                .next(true, &mut interner)
                .unwrap_err()
                .frontend_limit(),
            Some(FrontendLimit::TokenAdvance(0))
        );
        assert_eq!(cursor.peeked, slots);
        assert_eq!(
            (cursor.read_index, cursor.write_index, cursor.linear_pos()),
            position
        );
        assert_eq!(
            work[work.len() - 2..],
            [FrontendWork::TokenProbe, FrontendWork::TokenAdvance]
        );
        if !prefetched {
            assert_eq!(work.len(), 2);
        }
    }
}

#[test]
fn skipping_a_trailing_line_terminator_keeps_the_cached_eof() {
    let mut cursor = BufferedLexer::from(&b"\n"[..]);
    let mut interner = Interner::default();
    assert!(cursor.peek(0, true, &mut interner).unwrap().is_none());
    let eof_index = (cursor.read_index + 1) % super::PEEK_BUF_SIZE;
    let write_index = cursor.write_index;

    assert!(cursor.peek(2, true, &mut interner).unwrap().is_none());
    assert_eq!(cursor.write_index, write_index, "peek filled beyond EOF");
    assert!(cursor.next(true, &mut interner).unwrap().is_none());
    assert_eq!(cursor.read_index, eof_index, "advance removed cached EOF");
    assert!(cursor.next(true, &mut interner).unwrap().is_none());
    assert_eq!(cursor.write_index, write_index, "advance filled beyond EOF");
}

#[test]
fn peek_skip_ascending() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
}

#[test]
fn peek_skip_next() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("d", utf16!("d")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("e", utf16!("e")))
    );
    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("f", utf16!("f")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("g", utf16!("g")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("h", utf16!("h")))
    );
}

#[test]
fn peek_skip_next_alternating() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("d", utf16!("d")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("f", utf16!("f")))
    );
}

#[test]
fn peek_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    loop {
        let peek = cur.peek(0, false, interner).unwrap().cloned();
        let next = cur.next(false, interner).unwrap();

        assert_eq!(peek, next);

        if peek.is_none() {
            break;
        }
    }
}

#[test]
fn peek_skip_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    let mut peeked: [Option<Token>; super::MAX_PEEK_SKIP + 1] =
        [None::<Token>, None::<Token>, None::<Token>, None::<Token>];

    loop {
        for (i, peek) in peeked.iter_mut().enumerate() {
            *peek = cur.peek(i, false, interner).unwrap().cloned();
        }

        for peek in &peeked {
            assert_eq!(&cur.next(false, interner).unwrap(), peek);
        }

        if peeked[super::MAX_PEEK_SKIP - 1].is_none() {
            break;
        }
    }
}

#[test]
fn skip_peeked_terminators() {
    let mut cur = BufferedLexer::from(&b"A \n B"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("A", utf16!("A")))
    );
    assert_eq!(
        *cur.peek(0, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("A", utf16!("A")))
    );

    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::LineTerminator,
    );
    assert_eq!(
        *cur.peek(1, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("B", utf16!("B"))) /* This value is after the line terminator */
    );

    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("B", utf16!("B")))
    );
    // End of stream
    assert!(cur.peek(2, true, interner).unwrap().is_none());
}

#[test]
fn issue_1768() {
    let mut cur = BufferedLexer::from(&b"\n(\nx\n)\n"[..]);
    let interner = &mut Interner::default();

    assert!(cur.peek(3, true, interner).unwrap().is_none());
}
