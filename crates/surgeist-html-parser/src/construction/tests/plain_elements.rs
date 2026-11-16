//! What a plain element and a text node retain in the payload registry, and the work that reads
//! of the parse's documents make for them.
use super::*;
use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture, token::Event};

/// Creates `elements` plain `span` elements through the tag path of the parser, in a session of
/// an empty ordinary Document, and hands the session to `then` before releasing it.
fn with_plain_elements<T>(
    elements: usize,
    then: impl FnOnce(&mut ParseControl, &mut ParserArenaSession) -> T,
) -> T {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<span>".repeat(elements.max(1)).as_str()),
        limits(),
    );
    let (mut c, mut session, _proof) = fresh_script_owner(&request);
    let mut lexer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    for _ in 0..elements {
        let mut incoming = loop {
            match lexer.next(&mut c, false).unwrap().unwrap() {
                Event::Token(event) => {
                    let token = event.into_token(&mut c).unwrap();
                    if let crate::parser::tokenizer::token::TokenKind::Tag(tag) = token.kind {
                        break Some(tag);
                    }
                    release_token_kind(&mut Some(token.kind), &mut c).unwrap();
                }
                event => event.release(&mut c).unwrap(),
            }
        };
        session.allocate_html_tag(&mut c, &mut incoming).unwrap();
    }
    let result = then(&mut c, &mut session);
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    result
}

/// The registry rows that name the document of a node. The document of a plain element is the
/// document of the tree it was created in, which the parse holds once for every node of the
/// tree; the oracle is no row for the element and none for any text node.
fn node_document_rows(session: &ParserArenaSession) -> usize {
    session
        .active()
        .unwrap()
        .payloads
        .values
        .iter()
        .filter(|row| matches!(row, RetainedPayload::NodeDocument { .. }))
        .count()
}

#[test]
fn a_plain_element_adds_no_node_document_row_to_the_registry() {
    let rows =
        |elements: usize| with_plain_elements(elements, |_, session| node_document_rows(session));
    let empty = rows(0);
    for elements in [1, 16, 64] {
        assert_eq!(
            rows(elements) - empty,
            0,
            "{elements} plain elements added node document rows"
        );
    }
}

#[test]
fn preparing_the_association_documents_visits_nothing_for_the_plain_elements() {
    // The walk visits the Documents the parse owns and the distinct Documents its nodes name,
    // and no row for a node: its visits at 0, 16 and 64 plain elements are the same.
    let visits = |elements: usize| {
        with_plain_elements(elements, |c, session| {
            let before = c.usage().work(HtmlParseWorkCategory::RecordVisit);
            session
                .active_mut()
                .unwrap()
                .prepare_association_documents(c)
                .unwrap();
            c.usage().work(HtmlParseWorkCategory::RecordVisit) - before
        })
    };
    let empty = visits(0);
    for elements in [16, 64] {
        assert_eq!(visits(elements), empty, "{elements} plain elements");
    }
}
