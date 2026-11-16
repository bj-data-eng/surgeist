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

/// The registry rows a plain element holds. The document of a plain element is the document of
/// the tree it was created in, which the parse holds once for the tree, so no row of the element
/// names it. The oracle is the families an element of the HTML namespace has independently of its
/// document, in a parse that records detail: a lookup registry row, a nonce row, the strings of its
/// name, its namespace and its empty nonce value, and the cause of its creation.
fn registry_rows(session: &ParserArenaSession) -> usize {
    session.active().unwrap().payloads.values.len()
}

#[test]
fn a_plain_element_holds_no_registry_row_for_its_document() {
    let rows = |elements: usize| with_plain_elements(elements, |_, session| registry_rows(session));
    let empty = rows(0);
    for elements in [1, 16, 64] {
        assert_eq!(
            rows(elements) - empty,
            elements * PLAIN_ELEMENT_ROWS,
            "{elements} plain elements"
        );
    }
}

/// The registry rows of one plain element once the document it is in is held by its tree: a lookup
/// registry, a nonce, three strings (name, namespace, empty nonce value) and a construction cause.
const PLAIN_ELEMENT_ROWS: usize = 6;

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

/// The document of every node a published report answers for is the document the column named
/// for it when the parse was frozen. The report holds the document of each tree by its root and
/// not of each node, so this is the contract that every node of a tree has the document of its
/// root. The column is read from its copy in a test build; the oracle is that column, which is
/// written by the creation and the adoption of each node, not by the climb the report makes. A
/// new read with no earlier form, so the test has no failing state on the code it started from.
#[test]
fn the_document_a_report_answers_for_a_node_is_the_one_its_column_named_at_publication() {
    use crate::construction::report::HtmlParseMetadataDocument;
    let sources = [
        "<!doctype html><p>a<template><b>x<template><i>y</i></template></b></template><p>z",
        "<!doctype html><table><tr><td>a<template><td>b</template></table>",
        "<!doctype html><div><span><a href=x>y</a><link rel=x href=y></span></div>",
        "<!doctype html><svg><script/></svg><select><option>a</select>",
    ];
    for detail in [
        crate::detail::DetailSet::NONE,
        crate::detail::DetailSet::ALL,
    ] {
        for source in sources {
            let Ok(request) =
                crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits())
                    .with_detail(detail)
            else {
                continue;
            };
            let report = request.parse().unwrap();
            let storage = report.history().storage();
            let contexts = &storage.payloads.facts.contexts;
            let arena = &storage.view;
            assert_eq!(contexts.frozen_column.len(), arena.len(), "{source}");
            for (position, &entry) in contexts.frozen_column.iter().enumerate() {
                let id = arena.node_at(position).unwrap().as_node().unwrap().id();
                let answer = report.node_document_for_test(&id).unwrap().document();
                match (contexts.document(entry), answer) {
                    (None, None) => {}
                    (
                        Some(DocumentRef::Owned(expected)),
                        Some(HtmlParseMetadataDocument::Owned(found)),
                    ) => {
                        let expected = arena.node_at(*expected).unwrap().as_node().unwrap().id();
                        assert_eq!(found.id(), expected, "{source}: node {position}");
                    }
                    (
                        Some(DocumentRef::Captured(_)),
                        Some(HtmlParseMetadataDocument::Captured(_)),
                    ) => {}
                    (expected, _) => panic!("{source}: node {position} named {expected:?}"),
                }
            }
        }
    }
}
