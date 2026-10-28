//! Tests for "consume the value of a unicode-range descriptor" (CSS Syntax
//! Module Level 3 section 5.5.11) and the way section 5.5.6 step 8 reaches it.
//!
//! Expectations are written by hand from sections 4.3.11, 4.3.14 and 5.5.11.
//! The tokens of the range itself are tested with the lexer (`tests/lexer.rs`);
//! these tests are about the algorithm: the text is tokenized with unicode
//! ranges allowed, and the list of component values of the result is returned.

mod common;

use common::grammar::show;
use common::render;
use common::tree::parse;
use surgeist_css_parser::grammar::{AtRule, BlockGrammar, Declaration, QualifiedRule, RuleGrammar};
use surgeist_css_parser::limits::Limits;
use surgeist_css_parser::unicode_range::consume_value;
use surgeist_css_parser::{Document, Offset, SyntaxError};

fn value(text: &str) -> String {
    let document: Document = consume_value(text, Offset::new(0), Limits::default()).unwrap();
    show(&document.root())
}

#[test]
fn the_text_is_tokenized_with_unicode_ranges_allowed_and_returned_as_component_values() {
    assert_eq!(value("U+26"), "UnicodeRange(0x26,0x26)");
    assert_eq!(value("u+0025-00FF"), "UnicodeRange(0x25,0xff)");
    assert_eq!(value("U+4??"), "UnicodeRange(0x400,0x4ff)");
    assert_eq!(
        value("U+0-7F, U+1-2"),
        "UnicodeRange(0x0,0x7f) Comma Whitespace UnicodeRange(0x1,0x2)"
    );
    // The list is the whole text, whatever it holds.
    assert_eq!(
        value("U+26 junk f(U+1)"),
        "UnicodeRange(0x26,0x26) Whitespace Ident(\"junk\") Whitespace Function(\"f\")<UnicodeRange(0x1,0x1)>"
    );
    assert_eq!(value(""), "");
}

#[test]
fn a_range_is_a_token_with_the_values_section_4_3_14_gives_it() {
    // An end before the start, and bounds above the maximum code point, are
    // values of the token; the descriptor's grammar judges them.
    assert_eq!(value("U+20-10"), "UnicodeRange(0x20,0x10)");
    assert_eq!(value("U+110000-110001"), "UnicodeRange(0x110000,0x110001)");
    assert_eq!(value("U+FFFFFF"), "UnicodeRange(0xffffff,0xffffff)");
    assert_eq!(value("U+??????"), "UnicodeRange(0x0,0xffffff)");
}

#[test]
fn text_that_does_not_start_a_range_is_read_as_ordinary_tokens() {
    // Section 4.3.11: `U+` must be followed by a hex digit or a question mark.
    assert_eq!(value("U+"), "Ident(\"U\") Delim('+')");
    assert_eq!(value("U+x"), "Ident(\"U\") Delim('+') Ident(\"x\")");
    assert_eq!(value("u"), "Ident(\"u\")");
}

#[test]
fn without_the_algorithm_the_same_text_is_an_ident_and_a_number() {
    // The reason the descriptor needs it (section 5.5.11 and its note).
    let document = parse("U+26, U+4??");
    assert_eq!(
        show(&document.root()),
        "Ident(\"U\") Number(\"+26\",int) Comma Whitespace Ident(\"U\") Number(\"+4\",int) Delim('?') Delim('?')"
    );
}

#[test]
fn the_origin_makes_every_offset_a_position_in_the_larger_text() {
    let document: Document =
        consume_value("U+4??, U+26", Offset::new(40), Limits::default()).unwrap();
    let mut stream = document.root();
    assert_eq!(stream.offset(), Some(Offset::new(40)));
    stream.consume_token().unwrap();
    assert_eq!(stream.offset(), Some(Offset::new(45)));
    stream.consume_comma().unwrap();
    stream.consume_after_whitespace().unwrap();
    assert_eq!(stream.offset(), Some(Offset::new(51)));
    // Text between sites is taken from the text given, at those positions.
    let start = document.root().start_site();
    assert_eq!(
        stream.text_between(start, stream.site()),
        Some("U+4??, U+26")
    );
}

#[test]
fn the_limits_apply_as_for_any_document() {
    let error = consume_value::<()>(
        "f(U+26)",
        Offset::new(0),
        Limits::new(0, usize::MAX, usize::MAX),
    )
    .unwrap_err();
    assert!(error.is_depth_limit());
    let error = consume_value::<()>("U+26, U+27", Offset::new(0), Limits::new(8, 2, usize::MAX))
        .unwrap_err();
    assert!(error.is_node_limit());
}

// ----------------------------------------------- section 5.5.6 step 8

/// Hands back what a `unicode-range` declaration offers.
struct Collect;

impl<'i> RuleGrammar<'i> for Collect {
    type Item = (String, Option<&'i str>, Option<Offset>);
    type Error = ();

    fn at_rule(&mut self, rule: AtRule<'i>) -> Result<Self::Item, SyntaxError<'i, ()>> {
        Err(SyntaxError::unknown_at_rule_error(
            rule.start(),
            rule.name(),
        ))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i>,
    ) -> Result<Self::Item, SyntaxError<'i, ()>> {
        Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
    }
}

impl<'i> BlockGrammar<'i> for Collect {
    fn declaration(&mut self, d: Declaration<'i>) -> Result<Self::Item, SyntaxError<'i, ()>> {
        Ok((
            d.name().to_owned(),
            d.original_text(),
            d.value().start_site().offset(),
        ))
    }
}

#[test]
fn a_unicode_range_declaration_offers_its_text_which_the_algorithm_tokenizes_at_its_place() {
    let text = "src: x; unicode-range: U+26, U+4?? !important; color: red";
    let document = parse(text);
    let mut grammar = Collect;
    let items: Vec<_> = document
        .root()
        .block_contents(&mut grammar)
        .map(Result::unwrap)
        .collect();
    assert_eq!(items.len(), 3);
    // `src` and `color` are not asked for it.
    assert_eq!(items[0], ("src".to_owned(), None, Some(Offset::new(5))));
    assert_eq!(items[2].1, None);
    // `unicode-range`: the text of the value, without the importance.
    let (name, original, offset) = &items[1];
    assert_eq!(name, "unicode-range");
    assert_eq!(*original, Some("U+26, U+4??"));
    assert_eq!(*offset, Some(Offset::new(23)));
    // The grammar replaces the value with the tokens of the algorithm, placed
    // where they were.
    let ranges: Document =
        consume_value(original.unwrap(), offset.unwrap(), Limits::default()).unwrap();
    let mut stream = ranges.root();
    assert_eq!(
        render(stream.consume_token().unwrap()),
        "UnicodeRange(0x26,0x26)"
    );
    assert_eq!(stream.offset(), Some(Offset::new(27)));
    stream.consume_comma().unwrap();
    assert_eq!(
        render(stream.consume_after_whitespace().unwrap()),
        "UnicodeRange(0x400,0x4ff)"
    );
    assert!(stream.is_empty());
}
