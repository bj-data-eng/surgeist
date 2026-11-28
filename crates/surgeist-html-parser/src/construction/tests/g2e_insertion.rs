//! The check an insertion makes of the tree it inserts into, when that tree is the contents of a
//! template, reads the record and not the table of templates.
use super::*;

/// The record visits the check of whether the contents of the first template are a fragment that
/// an associated document owns charges, in a session that made `templates` templates through the
/// tag path of the parser, and the rows of the table of templates it holds.
fn check_visits(templates: usize) -> (u64, usize) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture, token::Event};
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<template>".repeat(templates).as_str()),
        g2_d2::corpus_limits(),
    );
    let (mut c, mut session, _proof) = fresh_script_owner(&request);
    let mut lexer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut first = None;
    for _ in 0..templates {
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
        let node = session.allocate_html_tag(&mut c, &mut incoming).unwrap();
        first.get_or_insert(node);
    }
    let template = first.expect("a template");
    let contents = session
        .template_contents(&mut c, &template)
        .unwrap()
        .expect("a template has contents");
    let rows = session.active().unwrap().payloads.facts.templates.len();
    let storage = session.active_mut().unwrap();
    let before = c.usage().work(HtmlParseWorkCategory::RecordVisit);
    let associated = storage
        .associated_document_parent(&mut c, contents.index)
        .unwrap();
    assert!(
        !associated,
        "the contents of a template are not an associated fragment"
    );
    let visits = c.usage().work(HtmlParseWorkCategory::RecordVisit) - before;
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    (visits, rows)
}

/// The oracle is the shape of the check: it names the owner of the contents, which the record
/// holds, so the visits it charges do not depend on how many templates the parse has made.
#[test]
fn the_check_of_a_template_contents_insertion_charges_the_same_visits_whatever_the_templates() {
    let (small, small_rows) = check_visits(4);
    let (large, large_rows) = check_visits(1_024);
    assert_eq!(
        (small_rows, large_rows),
        (4, 1_024),
        "every template has a row"
    );
    assert_eq!(small, large, "visits with 4 and with 1,024 templates");
}

/// The batches the parse charges for `lookups` reads of the metadata of the first template, in a
/// session that made `templates` templates through the tag path of the parser.
fn lookup_batches(templates: usize, lookups: usize) -> u64 {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture, token::Event};
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<template>".repeat(templates).as_str()),
        g2_d2::corpus_limits(),
    );
    let (mut c, mut session, _proof) = fresh_script_owner(&request);
    let mut lexer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut first = None;
    for _ in 0..templates {
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
        let node = session.allocate_html_tag(&mut c, &mut incoming).unwrap();
        first.get_or_insert(node);
    }
    let template = first.expect("a template");
    for _ in 0..lookups {
        session.template_state(&mut c, &template).unwrap();
    }
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    drop(c);
    crate::parser::control::last_parse_counts().batches
}

/// A lookup of the metadata of a template is a fixed number of charges: the keys its search reads
/// are charged with the visits of the lookup, so the number of charge calls does not grow with
/// the templates. The oracle is the count of batches the parse charges, with and without the
/// lookup, in sessions of 4 and of 1,024 templates.
#[test]
fn a_template_lookup_charges_the_same_number_of_batches_whatever_the_templates() {
    let small = lookup_batches(4, 1) - lookup_batches(4, 0);
    let large = lookup_batches(1_024, 1) - lookup_batches(1_024, 0);
    assert_eq!(
        small, large,
        "batches of one lookup with 4 and with 1,024 templates"
    );
}
