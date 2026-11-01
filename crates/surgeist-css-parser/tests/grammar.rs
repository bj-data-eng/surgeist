//! Tests for the parser entry points and rule algorithms of CSS Syntax Module
//! Level 3 sections 5.4 and 5.5.
//!
//! Every expectation is written by hand from the pinned edition: a group of
//! tests for each algorithm, a case for each branch of its "process" steps, and
//! the examples of the text. The grammar that receives the units
//! (`common::grammar::Dump`) only describes what it was handed; it decides
//! nothing beyond rejecting the names a test tells it to, which is the
//! specification's "valid in the current context".
//!
//! Offsets are byte offsets in the text, counted from the literal.

mod common;

use common::grammar::{
    Dump, Rejected, Silent, block, block_with, failure, sheet, sheet_with, show,
};
use common::render;
use common::tree::{Recording, metered_document, parse};
use surgeist_css_parser::document::Component;
use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, Declaration, QualifiedRule, RuleEnd, RuleGrammar, UnitKind,
};
use surgeist_css_parser::lexer::Options;
use surgeist_css_parser::limits::{DepthExcess, Limits};
use surgeist_css_parser::{Document, Stream, SyntaxError, Token};

// ------------------------------------------------- 5.5.1 consume a stylesheet's contents

#[test]
fn whitespace_cdo_and_cdc_between_rules_are_discarded() {
    // Section 5.5.1: a whitespace-token, a CDO-token and a CDC-token are
    // discarded; the end of the stream returns the rules.
    assert_eq!(
        sheet("  <!-- --> \n a {} <!--"),
        ["rule [Ident(\"a\") Whitespace] {} at=13..17"]
    );
    assert!(sheet("").is_empty());
    assert!(sheet(" \n\t ").is_empty());
    assert!(sheet("<!----><!--").is_empty());
}

#[test]
fn an_at_keyword_starts_an_at_rule_and_anything_else_a_qualified_rule() {
    assert_eq!(
        sheet("@a;b{}"),
        [
            "@a [] end=semicolon at=0..3",
            "rule [Ident(\"b\")] {} at=3..6"
        ]
    );
    // A token that is not an at-keyword but looks like the start of one is a
    // qualified rule's: a lone `@` is a delim.
    assert_eq!(sheet("@ {}"), ["rule [Delim('@') Whitespace] {} at=0..4"]);
}

#[test]
fn a_qualified_rule_is_a_prelude_and_a_block() {
    assert_eq!(
        sheet("a{b:c}"),
        ["rule [Ident(\"a\")] {Ident(\"b\") Colon Ident(\"c\")} at=0..6"]
    );
    assert_eq!(sheet("{}"), ["rule [] {} at=0..2"]);
    assert_eq!(
        sheet("a b {c}"),
        ["rule [Ident(\"a\") Whitespace Ident(\"b\") Whitespace] {Ident(\"c\")} at=0..7"]
    );
    // The prelude takes functions and blocks whole.
    assert_eq!(
        sheet("f(a) [b] (c) {d}"),
        [
            "rule [Function(\"f\")<Ident(\"a\")> Whitespace OpenSquare<Ident(\"b\")> Whitespace OpenParen<Ident(\"c\")> Whitespace] {Ident(\"d\")} at=0..16"
        ]
    );
    // A `{}`-block inside a group is that group's, not the rule's block.
    assert_eq!(
        sheet("({}) {x}"),
        ["rule [OpenParen<OpenCurly<>> Whitespace] {Ident(\"x\")} at=0..8"]
    );
}

// ------------------------------------------------- 5.5.2 consume an at-rule

#[test]
fn an_at_rule_ends_at_a_semicolon_a_block_or_the_end_of_the_stream() {
    // <semicolon-token>: discard it, and the rule is returned if valid.
    assert_eq!(sheet("@a;"), ["@a [] end=semicolon at=0..3"]);
    assert_eq!(
        sheet("@a b c;"),
        ["@a [Whitespace Ident(\"b\") Whitespace Ident(\"c\")] end=semicolon at=0..7"]
    );
    // <EOF-token>: the same, with nothing to discard.
    assert_eq!(
        sheet("@a b"),
        ["@a [Whitespace Ident(\"b\")] end=eof at=0..4"]
    );
    assert_eq!(sheet("@a"), ["@a [] end=eof at=0..2"]);
    // <{-token>: consume a block; its contents are the rule's.
    assert_eq!(
        sheet("@a {x}"),
        ["@a [Whitespace] {Ident(\"x\")} end=block at=0..6"]
    );
    assert_eq!(
        sheet("@a{}@b;"),
        ["@a [] {} end=block at=0..4", "@b [] end=semicolon at=4..7"]
    );
}

#[test]
fn a_semicolon_in_a_block_or_a_group_does_not_end_an_at_rule() {
    // The semicolon inside the block is the block's. The rule that follows has
    // a semicolon of its own, and as a qualified rule it needs a block.
    assert_eq!(
        sheet("@a {;} b;"),
        [
            "@a [Whitespace] {Semicolon} end=block at=0..6",
            "!qualified-rule (end-of-stream) end=eof at=7..9"
        ]
    );
    // A function and a parenthesis block and a square block are one component
    // value each, so the semicolons inside them are theirs.
    assert_eq!(
        sheet("@a (;) [;] f(;) b;"),
        [
            "@a [Whitespace OpenParen<Semicolon> Whitespace OpenSquare<Semicolon> Whitespace Function(\"f\")<Semicolon> Whitespace Ident(\"b\")] end=semicolon at=0..18"
        ]
    );
}

#[test]
fn a_close_brace_is_a_component_value_of_the_prelude_when_the_rule_is_not_nested() {
    // Section 5.5.2, <}-token>, nested false: consume a token and append it.
    assert_eq!(
        sheet("@a } b;"),
        ["@a [Whitespace CloseCurly Whitespace Ident(\"b\")] end=semicolon at=0..7"]
    );
    // Section 5.5.3 likewise.
    assert_eq!(
        sheet("a } b {}"),
        ["rule [Ident(\"a\") Whitespace CloseCurly Whitespace Ident(\"b\") Whitespace] {} at=0..8"]
    );
}

#[test]
fn a_close_brace_ends_a_nested_at_rule_without_being_consumed() {
    // Section 5.5.2, <}-token>, nested true: if the rule is valid return it. The
    // `}` stays for the caller; here it ends the contents of the block.
    let document = parse("@x a } b");
    let mut grammar = Dump::accepting();
    let mut units = document.root().block_contents(&mut grammar);
    let first = units.next().unwrap().unwrap();
    assert_eq!(
        first,
        "@x [Whitespace Ident(\"a\") Whitespace] end=close-brace at=0..5"
    );
    assert!(units.next().is_none());
    assert_eq!(units.stream().next_token(), Some(Token::CloseCurly));
}

#[test]
fn an_at_rule_the_grammar_rejects_is_reported_and_the_next_rule_is_unaffected() {
    let mut grammar = Dump {
        reject_at: vec!["x"],
        ..Dump::default()
    };
    assert_eq!(
        sheet_with("@x; @y;", &mut grammar),
        [
            "!at-rule (grammar) end=semicolon at=0..3",
            "@y [] end=semicolon at=4..7"
        ]
    );
    // A rejected at-rule with a block has been consumed with its block: what
    // follows is read as the next rule, not as the rejected one's contents.
    let mut grammar = Dump {
        reject_at: vec!["x"],
        ..Dump::default()
    };
    assert_eq!(
        sheet_with("@x { a {} } b {}", &mut grammar),
        [
            "!at-rule (grammar) end=block at=0..11",
            "rule [Ident(\"b\") Whitespace] {} at=12..16"
        ]
    );
}

// ------------------------------------------------- 5.5.3 consume a qualified rule

#[test]
fn a_qualified_rule_without_a_block_is_dropped_at_the_end_of_the_stream() {
    // "This is a parse error. Return nothing." A semicolon does not stop a rule
    // that has no stop token.
    assert_eq!(
        sheet("a b"),
        ["!qualified-rule (end-of-stream) end=eof at=0..3"]
    );
    assert_eq!(
        sheet("a b; c {}"),
        [
            "rule [Ident(\"a\") Whitespace Ident(\"b\") Semicolon Whitespace Ident(\"c\") Whitespace] {} at=0..9"
        ]
    );
}

#[test]
fn a_qualified_rule_the_grammar_rejects_is_an_invalid_rule_error() {
    let mut grammar = Dump::rejecting_qualified();
    assert_eq!(
        sheet_with("a {} b {}", &mut grammar),
        [
            "!qualified-rule (grammar) end=block at=0..4 invalid-rule-error",
            "!qualified-rule (grammar) end=block at=5..9 invalid-rule-error"
        ]
    );
}

#[test]
fn a_qualified_rule_that_looks_like_a_custom_property_is_consumed_whole_and_reports_nothing() {
    // Section 5.5.3, <{-token>: the first two non-whitespace values of the
    // prelude are an ident that starts with two hyphen-minuses and a colon.
    // Not nested: consume a block and return nothing.
    assert_eq!(
        sheet("--x:hover {color: blue} a {}"),
        ["rule [Ident(\"a\") Whitespace] {} at=24..28"]
    );
    // Whitespace between the two values does not matter, and neither does a
    // comment, which is not a token.
    assert_eq!(
        sheet("--x :hover{} a{}"),
        ["rule [Ident(\"a\")] {} at=13..16"]
    );
    assert_eq!(
        sheet("/*c*/ --x:y{} a{}"),
        ["rule [Ident(\"a\")] {} at=14..17"]
    );
    // `--` is an ident that starts with two hyphen-minuses.
    assert!(sheet("--:x{}").is_empty());
}

#[test]
fn a_rule_that_does_not_look_like_a_custom_property_is_a_rule() {
    // No colon: the second value is whitespace's successor, a block.
    assert_eq!(
        sheet("--x {} a{}"),
        [
            "rule [Ident(\"--x\") Whitespace] {} at=0..6",
            "rule [Ident(\"a\")] {} at=7..10"
        ]
    );
    // The colon is not second.
    assert_eq!(
        sheet("x:--y {}"),
        ["rule [Ident(\"x\") Colon Ident(\"--y\") Whitespace] {} at=0..8"]
    );
    // One hyphen-minus is not two.
    assert_eq!(
        sheet("-x:y{}"),
        ["rule [Ident(\"-x\") Colon Ident(\"y\")] {} at=0..6"]
    );
}

#[test]
fn a_rule_with_a_denied_block_is_reported_and_not_given_to_the_grammar() {
    // A build that denies blocks nested deeper than its limit keeps no contents
    // for them; the grammar is not shown a rule it could mistake for empty.
    let document: Document = Document::parse(
        "a {b} @m {x}",
        &Options::default(),
        Limits::new(0, usize::MAX, usize::MAX),
        0,
        DepthExcess::DenyBlock,
    )
    .unwrap();
    let mut grammar = Dump::accepting();
    let units: Vec<String> = document
        .root()
        .stylesheet_contents(&mut grammar)
        .map(|unit| unit.unwrap_or_else(|failed| failure(&failed)))
        .collect();
    assert_eq!(
        units,
        [
            "!qualified-rule (denied-block) end=block at=0..5",
            "!at-rule (denied-block) end=block at=6..12"
        ]
    );
}

// ------------------------------------------------- 5.5.5 consume a block's contents

#[test]
fn declarations_are_consumed_with_their_whitespace_and_semicolons_discarded() {
    assert_eq!(
        block("a: b; c: d"),
        [
            "decl a [Ident(\"b\")] end=semicolon at=0..4",
            "decl c [Ident(\"d\")] end=eof at=6..10"
        ]
    );
    // Whitespace-token and semicolon-token: discard.
    assert_eq!(
        block(" ;; a:b ;"),
        ["decl a [Ident(\"b\")] end=semicolon at=4..8"]
    );
    assert!(block("").is_empty());
    assert!(block(" ; ; ").is_empty());
}

#[test]
fn a_close_brace_ends_the_contents_and_stays_in_the_stream() {
    let document = parse("a:b } c:d");
    let mut grammar = Dump::accepting();
    let mut units = document.root().block_contents(&mut grammar);
    assert_eq!(
        units.next().unwrap().unwrap(),
        "decl a [Ident(\"b\")] end=close-brace at=0..4"
    );
    assert!(units.next().is_none());
    assert!(units.next().is_none());
    assert_eq!(units.stream().next_token(), Some(Token::CloseCurly));
}

#[test]
fn a_unit_that_is_not_a_declaration_is_read_again_as_a_qualified_rule_that_a_semicolon_ends() {
    // Section 5.5.5: restore the mark and consume a qualified rule with a
    // semicolon as the stop token; the stop token returns nothing.
    assert_eq!(
        block("a b; c: d"),
        [
            "!declaration (unexpected Ident(\"b\")) end=semicolon at=0..3",
            "decl c [Ident(\"d\")] end=eof at=5..9"
        ]
    );
    // The end of the stream and a `}` do the same.
    assert_eq!(
        block("a b"),
        ["!declaration (unexpected Ident(\"b\")) end=eof at=0..3"]
    );
    assert_eq!(
        block("a b } c"),
        ["!declaration (unexpected Ident(\"b\")) end=close-brace at=0..4"]
    );
    // A rule with a block is a rule.
    assert_eq!(
        block("&:hover { x: y } z: 1"),
        [
            "rule [Delim('&') Colon Ident(\"hover\") Whitespace] {Whitespace Ident(\"x\") Colon Whitespace Ident(\"y\") Whitespace} at=0..16",
            "decl z [Number(\"1\",int)] end=eof at=17..21"
        ]
    );
}

#[test]
fn the_block_of_a_rule_can_be_read_as_a_blocks_contents_by_the_grammar() {
    let mut grammar = Dump {
        blocks_as_contents: true,
        ..Dump::default()
    };
    assert_eq!(
        block_with("&:hover { x: y } @m { z: 1; w: 2 }", &mut grammar),
        [
            "rule [Delim('&') Colon Ident(\"hover\") Whitespace] {decl x [Ident(\"y\")] end=eof at=10..15} at=0..16",
            "@m [Whitespace] {decl z [Number(\"1\",int)] end=semicolon at=22..26|decl w [Number(\"2\",int)] end=eof at=28..33} end=block at=17..34"
        ]
    );
}

#[test]
fn an_at_rule_in_a_block_is_consumed_nested() {
    assert_eq!(
        block("a:b; @x; c:d"),
        [
            "decl a [Ident(\"b\")] end=semicolon at=0..3",
            "@x [] end=semicolon at=5..8",
            "decl c [Ident(\"d\")] end=eof at=9..12"
        ]
    );
}

#[test]
fn a_declaration_the_grammar_rejects_is_read_again_as_a_rule() {
    // Step 9: invalid in the current context returns nothing, and the caller
    // restores the mark.
    let mut grammar = Dump {
        reject_declarations: vec!["a"],
        ..Dump::default()
    };
    assert_eq!(
        block_with("a: b; c", &mut grammar),
        [
            "!declaration (grammar) end=semicolon at=0..4",
            "!declaration (end-of-stream) end=eof at=6..7"
        ]
    );
    // With a block as its whole value it is a valid declaration, which the
    // grammar rejects, and then a qualified rule the grammar accepts.
    let mut grammar = Dump {
        reject_declarations: vec!["a"],
        ..Dump::default()
    };
    assert_eq!(
        block_with("a: {x}", &mut grammar),
        ["rule [Ident(\"a\") Colon Whitespace] {Ident(\"x\")} at=0..6"]
    );
    // Both attempts rejected: the unit is an invalid rule error that carries the
    // declaration's error too.
    let mut grammar = Dump {
        reject_declarations: vec!["a"],
        reject_qualified: true,
        ..Dump::default()
    };
    assert_eq!(
        block_with("a: {x}", &mut grammar),
        [
            "!qualified-rule (grammar) end=block at=0..6 invalid-rule-error declaration-error=grammar"
        ]
    );
}

#[test]
fn a_rejected_declaration_that_is_a_custom_property_lookalike_is_not_lost() {
    // The rule attempt sees `--a:b {x}` as a custom property, consumes its
    // remnants through the semicolon and returns nothing (section 5.5.3,
    // nested). The grammar's verdict on the declaration is reported.
    let mut grammar = Dump {
        reject_declarations: vec!["--a"],
        ..Dump::default()
    };
    assert_eq!(
        block_with("--a:b {x}; c: d", &mut grammar),
        [
            "!declaration (grammar) end=semicolon at=0..10",
            "decl c [Ident(\"d\")] end=eof at=11..15"
        ]
    );
    // Remnants stop at a `}` and leave it.
    let mut grammar = Dump {
        reject_declarations: vec!["--a"],
        ..Dump::default()
    };
    assert_eq!(
        block_with("--a:b {x} y } z", &mut grammar),
        ["!declaration (grammar) end=close-brace at=0..12"]
    );
}

// ------------------------------------------------- 5.5.6 consume a declaration

#[test]
fn a_declaration_needs_an_ident_and_a_colon() {
    // Step 1: not an ident. Step 3: not a colon.
    assert_eq!(
        block("a b: c"),
        ["!declaration (unexpected Ident(\"b\")) end=eof at=0..6"]
    );
    assert_eq!(
        block("1: c;"),
        ["!declaration (unexpected Number(\"1\",int)) end=semicolon at=0..4"]
    );
    assert_eq!(block("a"), ["!declaration (end-of-stream) end=eof at=0..1"]);
    // Steps 2 and 4: whitespace around the colon is discarded; an empty value
    // is a value.
    assert_eq!(block("a : b"), ["decl a [Ident(\"b\")] end=eof at=0..5"]);
    assert_eq!(block("a:"), ["decl a [] end=eof at=0..2"]);
    assert_eq!(
        block("a:   b   ;"),
        ["decl a [Ident(\"b\")] end=semicolon at=0..9"]
    );
}

#[test]
fn the_value_runs_to_a_semicolon_the_end_or_a_nested_close_brace_and_takes_groups_whole() {
    assert_eq!(
        block("a: f(b; c) [d;] (e;); x"),
        [
            "decl a [Function(\"f\")<Ident(\"b\") Semicolon Whitespace Ident(\"c\")> Whitespace OpenSquare<Ident(\"d\") Semicolon> Whitespace OpenParen<Ident(\"e\") Semicolon>] end=semicolon at=0..20",
            "!declaration (end-of-stream) end=eof at=22..23"
        ]
    );
}

#[test]
fn important_is_a_bang_and_an_ident_that_matches_ascii_case_insensitively_at_the_end() {
    // Step 6: the last two non-whitespace values; step 7 trims the whitespace
    // that was before them.
    assert_eq!(
        block("a: b !important"),
        ["decl a [Ident(\"b\")] important end=eof at=0..15"]
    );
    assert_eq!(
        block("a: b ! important ;"),
        ["decl a [Ident(\"b\")] important end=semicolon at=0..17"]
    );
    assert_eq!(
        block("a: b !IMPORTant"),
        ["decl a [Ident(\"b\")] important end=eof at=0..15"]
    );
    assert_eq!(
        block("a: !important"),
        ["decl a [] important end=eof at=0..13"]
    );
    // A comment is not a value.
    assert_eq!(
        block("a: b !important/**/"),
        ["decl a [Ident(\"b\")] important end=eof at=0..19"]
    );
    // Only the last two count, and only if they are `!` and `important`.
    assert_eq!(
        block("a: important"),
        ["decl a [Ident(\"important\")] end=eof at=0..12"]
    );
    assert_eq!(
        block("a: ! b"),
        ["decl a [Delim('!') Whitespace Ident(\"b\")] end=eof at=0..6"]
    );
    assert_eq!(
        block("a: b !important c"),
        [
            "decl a [Ident(\"b\") Whitespace Delim('!') Ident(\"important\") Whitespace Ident(\"c\")] end=eof at=0..17"
        ]
    );
    assert_eq!(
        block("a: b !important !important"),
        [
            "decl a [Ident(\"b\") Whitespace Delim('!') Ident(\"important\")] important end=eof at=0..26"
        ]
    );
    // Values inside a function are not top-level values.
    assert_eq!(
        block("a: f(!important)"),
        ["decl a [Function(\"f\")<Delim('!') Ident(\"important\")>] end=eof at=0..16"]
    );
}

#[test]
fn a_block_is_only_allowed_as_the_entire_value_of_a_non_custom_property() {
    // Step 8: "foo: {} bar" is not a declaration; the text is read as a rule, and
    // the lone `bar;` after it is dropped (the note of section 5.5.3).
    assert_eq!(
        block("a: {} b;"),
        [
            "rule [Ident(\"a\") Colon Whitespace] {} at=0..5",
            "!declaration (unexpected Semicolon) end=semicolon at=6..7"
        ]
    );
    // The same text with the block second is the same.
    assert_eq!(
        block("a: b {x} ; c"),
        [
            "rule [Ident(\"a\") Colon Whitespace Ident(\"b\") Whitespace] {Ident(\"x\")} at=0..8",
            "!declaration (end-of-stream) end=eof at=11..12"
        ]
    );
    // As the entire value, whitespace aside, it is a declaration.
    assert_eq!(
        block("a: {x}"),
        ["decl a [OpenCurly<Ident(\"x\")>] end=eof at=0..6"]
    );
    assert_eq!(
        block("a:  {x} ;"),
        ["decl a [OpenCurly<Ident(\"x\")>] end=semicolon at=0..8"]
    );
}

#[test]
fn a_custom_property_keeps_the_original_text_of_its_value_and_may_hold_blocks() {
    // Step 8: the original text corresponds to the tokens of the value, after
    // the importance and the whitespace at both ends are gone.
    assert_eq!(
        block("--a: {x} y;"),
        [
            "decl --a [OpenCurly<Ident(\"x\")> Whitespace Ident(\"y\")] original=\"{x} y\" end=semicolon at=0..10"
        ]
    );
    assert_eq!(
        block("--a:  1 /*c*/ 2  ;"),
        [
            "decl --a [Number(\"1\",int) Whitespace Whitespace Number(\"2\",int)] original=\"1 /*c*/ 2\" end=semicolon at=0..17"
        ]
    );
    assert_eq!(
        block("--a: x !important;"),
        ["decl --a [Ident(\"x\")] important original=\"x\" end=semicolon at=0..17"]
    );
    assert_eq!(
        block("--a:;"),
        ["decl --a [] original=\"\" end=semicolon at=0..4"]
    );
    // Only a name that starts with two hyphen-minuses is a custom property: the
    // check is on the decoded name and is case-sensitive about nothing else.
    assert_eq!(block("-a: x"), ["decl -a [Ident(\"x\")] end=eof at=0..5"]);
}

#[test]
fn the_original_text_of_a_unicode_range_declaration_is_offered_for_the_tokenizer_option() {
    // Step 8: `unicode-range` (ASCII case-insensitive) asks for the original
    // text, which `unicode_range::consume_value` tokenizes.
    for name in ["unicode-range", "Unicode-RANGE"] {
        let text = format!("{name}: U+26, U+4??;");
        let out = block(&text);
        assert_eq!(out.len(), 1);
        assert!(
            out[0].contains("original=\"U+26, U+4??\""),
            "{name}: {}",
            out[0]
        );
    }
    // Not for another name.
    assert!(!block("unicode-ranges: U+26;")[0].contains("original"));
}

// ------------------------------------------------- the grouping of 5.5.5

/// The items a consumer groups as section 5.5.5 does.
#[derive(Debug, Eq, PartialEq)]
enum Group {
    Declarations(Vec<String>),
    Rule(String),
}

#[derive(Debug)]
enum Item {
    Declaration(String),
    AtRule(String),
    Rule(String),
}

struct Kinds {
    reject_qualified: bool,
    reject_at: bool,
}

impl<'i> RuleGrammar<'i> for Kinds {
    type Item = Item;
    type Error = Rejected;

    fn at_rule(&mut self, rule: AtRule<'i>) -> Result<Item, SyntaxError<'i, Rejected>> {
        if self.reject_at {
            return Err(SyntaxError::custom(rule.start(), Rejected));
        }
        Ok(Item::AtRule(format!("@{}", rule.name())))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i>,
    ) -> Result<Item, SyntaxError<'i, Rejected>> {
        if self.reject_qualified {
            return Err(SyntaxError::custom(rule.start(), Rejected));
        }
        Ok(Item::Rule(show(&rule.prelude()).replace(['\"', ' '], "")))
    }
}

impl<'i> BlockGrammar<'i> for Kinds {
    fn declaration(&mut self, d: Declaration<'i>) -> Result<Item, SyntaxError<'i, Rejected>> {
        Ok(Item::Declaration(d.name().to_owned()))
    }
}

/// Groups the units of `text` by the rules of section 5.5.5 for `decls`: an
/// at-rule, a rule and an invalid rule error end a run of declarations, and a
/// unit that returned nothing does not.
fn grouped(text: &str, mut grammar: Kinds) -> Vec<Group> {
    let document = parse(text);
    let mut rules = Vec::new();
    let mut decls: Vec<String> = Vec::new();
    let flush = |rules: &mut Vec<Group>, decls: &mut Vec<String>| {
        if !decls.is_empty() {
            rules.push(Group::Declarations(std::mem::take(decls)));
        }
    };
    for unit in document.root().block_contents(&mut grammar) {
        match unit {
            Ok(Item::Declaration(name)) => decls.push(name),
            Ok(Item::AtRule(name) | Item::Rule(name)) => {
                flush(&mut rules, &mut decls);
                rules.push(Group::Rule(name));
            }
            Err(failed) if failed.kind() == UnitKind::AtRule => flush(&mut rules, &mut decls),
            Err(failed) if failed.is_invalid_rule_error() => flush(&mut rules, &mut decls),
            Err(_) => {}
        }
    }
    flush(&mut rules, &mut decls);
    rules
}

fn names(list: &[&str]) -> Group {
    Group::Declarations(list.iter().map(|name| (*name).to_owned()).collect())
}

#[test]
fn a_run_of_declarations_ends_before_a_rule_and_after_an_invalid_rule_error_but_not_after_nothing()
{
    let accepting = || Kinds {
        reject_qualified: false,
        reject_at: false,
    };
    // [a b] @x [c] &{} [d] `e f {}` [g j]: "h i;" returned nothing and did not
    // end the run that g started.
    assert_eq!(
        grouped(
            "a:1; b:2; @x; c:3; &{}; d:4; e f {} g:5; h i; j:6",
            accepting()
        ),
        [
            names(&["a", "b"]),
            Group::Rule("@x".to_owned()),
            names(&["c"]),
            Group::Rule("Delim('&')".to_owned()),
            names(&["d"]),
            Group::Rule("Ident(e)WhitespaceIdent(f)Whitespace".to_owned()),
            names(&["g", "j"]),
        ]
    );
    // An at-rule the grammar rejects still ends the run before it.
    assert_eq!(
        grouped(
            "a:1; @bad; b:2",
            Kinds {
                reject_qualified: false,
                reject_at: true,
            }
        ),
        [names(&["a"]), names(&["b"])]
    );
    // An invalid rule error ends the run.
    assert_eq!(
        grouped(
            "a:1; x {} b:2",
            Kinds {
                reject_qualified: true,
                reject_at: false,
            }
        ),
        [names(&["a"]), names(&["b"])]
    );
    // Nothing does not.
    assert_eq!(grouped("a:1; h i; b:2", accepting()), [names(&["a", "b"])]);
}

// ------------------------------------------------- 5.4.6 parse a rule

type Parsed = Result<String, String>;

fn parse_rule(text: &str, mut grammar: Dump) -> Parsed {
    let document = parse(text);
    document
        .root()
        .parse_rule(&mut grammar)
        .map_err(|failed| failure(&failed))
}

fn rule(text: &str) -> Parsed {
    parse_rule(text, Dump::accepting())
}

#[test]
fn parse_a_rule_discards_whitespace_then_consumes_one_rule_then_requires_the_end() {
    assert_eq!(
        rule("  @media a {}  "),
        Ok("@media [Whitespace Ident(\"a\") Whitespace] {} end=block at=2..13".to_owned())
    );
    assert_eq!(
        rule(" a b { c } "),
        Ok("rule [Ident(\"a\") Whitespace Ident(\"b\") Whitespace] {Whitespace Ident(\"c\") Whitespace} at=1..10".to_owned())
    );
    assert_eq!(rule("@x;"), Ok("@x [] end=semicolon at=0..3".to_owned()));
}

#[test]
fn parse_a_rule_returns_a_syntax_error_for_an_empty_stream_or_anything_after_the_rule() {
    // Step 3: <EOF-token>.
    assert_eq!(
        rule(""),
        Err("!qualified-rule (end-of-stream) end=none at=0..0".to_owned())
    );
    assert_eq!(
        rule("   "),
        Err("!qualified-rule (end-of-stream) end=none at=3..3".to_owned())
    );
    // Step 5: not at the end after the rule.
    assert_eq!(
        rule("@media a {} b {}"),
        Err("!at-rule (unexpected Ident(\"b\")) end=block at=0..11".to_owned())
    );
    assert_eq!(
        rule("a{} ;"),
        Err("!qualified-rule (unexpected Semicolon) end=block at=0..3".to_owned())
    );
}

#[test]
fn parse_a_rule_returns_a_syntax_error_for_nothing_and_for_an_invalid_rule_error() {
    // A qualified rule without a block returns nothing.
    assert_eq!(
        rule("a b"),
        Err("!qualified-rule (end-of-stream) end=eof at=0..3".to_owned())
    );
    // A rule that looks like a custom property returns nothing.
    assert_eq!(
        rule("--x:y {}"),
        Err("!qualified-rule (invalid-qualified-rule) end=block at=0..8".to_owned())
    );
    // The grammar's rejection of either kind.
    assert_eq!(
        parse_rule("a{}", Dump::rejecting_qualified()),
        Err("!qualified-rule (grammar) end=block at=0..3 invalid-rule-error".to_owned())
    );
    assert_eq!(
        parse_rule(
            "@x;",
            Dump {
                reject_at: vec!["x"],
                ..Dump::default()
            }
        ),
        Err("!at-rule (grammar) end=semicolon at=0..3".to_owned())
    );
}

// ------------------------------------------------- 5.4.7 parse a declaration

fn declaration(text: &str, mut grammar: Dump) -> Parsed {
    let document = parse(text);
    document
        .root()
        .parse_declaration(&mut grammar)
        .map_err(|failed| failure(&failed))
}

#[test]
fn parse_a_declaration_discards_whitespace_and_consumes_one_declaration_and_requires_nothing_after()
{
    assert_eq!(
        declaration(" color: red; x", Dump::accepting()),
        Ok("decl color [Ident(\"red\")] end=semicolon at=1..11".to_owned())
    );
    assert_eq!(
        declaration("a: b !important", Dump::accepting()),
        Ok("decl a [Ident(\"b\")] important end=eof at=0..15".to_owned())
    );
}

#[test]
fn parse_a_declaration_is_not_nested_so_a_close_brace_is_a_value() {
    assert_eq!(
        declaration("a: b } c", Dump::accepting()),
        Ok(
            "decl a [Ident(\"b\") Whitespace CloseCurly Whitespace Ident(\"c\")] end=eof at=0..8"
                .to_owned()
        )
    );
}

#[test]
fn parse_a_declaration_returns_a_syntax_error_when_no_declaration_is_returned() {
    assert_eq!(
        declaration("a b", Dump::accepting()),
        Err("!declaration (unexpected Ident(\"b\")) end=none at=0..0".to_owned())
    );
    // It parses a declaration and not an at-rule.
    assert_eq!(
        declaration("@x", Dump::accepting()),
        Err("!declaration (unexpected AtKeyword(\"x\")) end=none at=0..0".to_owned())
    );
    assert_eq!(
        declaration("", Dump::accepting()),
        Err("!declaration (end-of-stream) end=none at=0..0".to_owned())
    );
    assert_eq!(
        declaration(
            "a: b",
            Dump {
                reject_declarations: vec!["a"],
                ..Dump::default()
            }
        ),
        Err("!declaration (grammar) end=eof at=0..4".to_owned())
    );
    // A block beside another value, step 8.
    assert_eq!(
        declaration("a: {} b", Dump::accepting()),
        Err("!declaration (unexpected OpenCurly) end=none at=0..0".to_owned())
    );
}

// ------------------------------------------------- 5.4.8 to 5.4.10 and 5.5.8

#[test]
fn consume_a_component_value_takes_a_token_or_a_whole_group() {
    let document = parse("f(a b) /*c*/ {x}");
    let mut stream = document.root();
    assert!(matches!(
        stream.consume_component_value().unwrap(),
        Component::Function { name: "f", .. }
    ));
    assert!(matches!(
        stream.consume_component_value().unwrap(),
        Component::Token {
            token: Token::Whitespace,
            ..
        }
    ));
    // The comment is not a value; the block is one.
    assert!(matches!(
        stream.consume_component_value().unwrap(),
        Component::Token {
            token: Token::Whitespace,
            ..
        }
    ));
    assert!(matches!(
        stream.consume_component_value().unwrap(),
        Component::Block { .. }
    ));
    assert!(
        stream
            .consume_component_value()
            .unwrap_err()
            .is_end_of_stream()
    );
    assert!(stream.is_empty());
}

fn component_value(text: &str) -> Result<String, (bool, bool)> {
    let document = parse(text);
    let mut stream = document.root();
    let result = stream.parse_component_value();
    match result {
        Ok(component) => {
            assert!(stream.is_empty(), "a value is returned at the end");
            Ok(match component {
                Component::Token { token, .. } => render(token),
                Component::Function { name, .. } => format!("function {name}"),
                Component::Block { kind, .. } => format!("block {kind:?}"),
                _ => "other".to_owned(),
            })
        }
        Err(error) => Err((error.is_end_of_stream(), error.is_unexpected_token())),
    }
}

#[test]
fn parse_a_component_value_is_one_value_between_whitespace() {
    assert_eq!(component_value("  f(a b)  "), Ok("function f".to_owned()));
    assert_eq!(component_value("{x}"), Ok("block Curly".to_owned()));
    assert_eq!(component_value("a /*c*/ "), Ok("Ident(\"a\")".to_owned()));
    // Step 3: empty.
    assert_eq!(component_value(""), Err((true, false)));
    assert_eq!(component_value(" "), Err((true, false)));
    // Step 6: not empty after the value.
    assert_eq!(component_value("a b"), Err((false, true)));
}

#[test]
fn a_failed_parse_a_component_value_leaves_the_stream_where_it_was() {
    let document = parse(" a b");
    let mut stream = document.root();
    let before = stream.site();
    assert!(stream.parse_component_value().is_err());
    assert_eq!(stream.site(), before);
    assert_eq!(stream.next_token(), Some(Token::Whitespace));
}

#[test]
fn parse_a_list_of_component_values_takes_every_value_of_the_rest_a_close_brace_included() {
    let document = parse("a f(b) }");
    let values = document.root().parse_list_of_component_values().unwrap();
    assert_eq!(values.len(), 5);
    assert!(matches!(values[2], Component::Function { .. }));
    assert!(matches!(
        values[4],
        Component::Token {
            token: Token::CloseCurly,
            ..
        }
    ));
    // Comments are not values.
    let document = parse("a/**/b");
    assert_eq!(
        document
            .root()
            .parse_list_of_component_values()
            .unwrap()
            .len(),
        2
    );
    assert!(
        parse("")
            .root()
            .parse_list_of_component_values()
            .unwrap()
            .is_empty()
    );
}

fn groups(text: &str) -> Vec<String> {
    let document = parse(text);
    let mut stream = document.root();
    let groups = stream.parse_comma_separated_component_values().unwrap();
    assert!(stream.is_empty(), "the stream is at its end afterwards");
    groups.iter().map(show).collect()
}

#[test]
fn parse_a_comma_separated_list_of_component_values_follows_the_algorithm_literally() {
    // While the input is not empty: consume a list up to a comma, discard the
    // comma.
    assert!(groups("").is_empty());
    assert_eq!(groups("a"), ["Ident(\"a\")"]);
    assert_eq!(groups("a,b"), ["Ident(\"a\")", "Ident(\"b\")"]);
    // A final comma adds no group.
    assert_eq!(groups("a,"), ["Ident(\"a\")"]);
    assert_eq!(groups("a,,"), ["Ident(\"a\")", ""]);
    // An empty group between commas, and before the first.
    assert_eq!(groups("a,,b"), ["Ident(\"a\")", "", "Ident(\"b\")"]);
    assert_eq!(groups(",a"), ["", "Ident(\"a\")"]);
    assert_eq!(groups(","), [""]);
    // Whitespace is a token, so a stream of it is a group, and so is the rest
    // after a comma.
    assert_eq!(groups(" "), ["Whitespace"]);
    assert_eq!(groups("a, "), ["Ident(\"a\")", "Whitespace"]);
    // A comma in a function or block is not a top-level comma.
    assert_eq!(
        groups("f(a,b),(c,d)"),
        [
            "Function(\"f\")<Ident(\"a\") Comma Ident(\"b\")>",
            "OpenParen<Ident(\"c\") Comma Ident(\"d\")>"
        ]
    );
    // A comment is not a token: a stream of comments is empty.
    assert!(groups("/**/").is_empty());
}

fn list(text: &str) -> Vec<Result<String, bool>> {
    let document = parse(text);
    let mut stream = document.root();
    let results = stream
        .parse_comma_separated_list(|group| {
            Ok::<_, SyntaxError<'_, ()>>(render(Token::Ident(group.consume_ident()?)))
        })
        .unwrap();
    results
        .into_iter()
        .map(|result| result.map_err(|error| error.is_end_of_stream()))
        .collect()
}

#[test]
fn parse_a_comma_separated_list_according_to_a_grammar_parses_each_group_on_its_own() {
    // Step 2: only whitespace returns an empty list.
    assert!(list("").is_empty());
    assert!(list(" /**/ \n").is_empty());
    // Step 4: each group is parsed; a failure is that group's alone.
    assert_eq!(
        list("a, 1, b"),
        [
            Ok("Ident(\"a\")".to_owned()),
            Err(false),
            Ok("Ident(\"b\")".to_owned())
        ]
    );
    // An empty group is a failure of the grammar for that group, and a final
    // comma adds none.
    assert_eq!(
        list("a,,b"),
        [
            Ok("Ident(\"a\")".to_owned()),
            Err(true),
            Ok("Ident(\"b\")".to_owned())
        ]
    );
    assert_eq!(list("a,"), [Ok("Ident(\"a\")".to_owned())]);
    // "Parse something according to a grammar" requires the group to be used up:
    // `a b` is not an ident.
    assert_eq!(list("a b"), [Err(false)]);
}

#[test]
fn comma_segments_is_not_the_algorithm_of_section_5_4_2() {
    // `comma_segments` hands every segment to the grammar, an empty one after a
    // trailing comma included, and an empty stream is one empty segment.
    let count = |text: &str| {
        let document = parse(text);
        document
            .root()
            .comma_segments(|segment| {
                let mut tokens = 0;
                while segment.consume_token().is_ok() {
                    tokens += 1;
                }
                Ok::<_, SyntaxError<'_, ()>>(tokens)
            })
            .unwrap()
            .len()
    };
    assert_eq!(count("a,"), 2);
    assert_eq!(count(""), 1);
    assert_eq!(groups("a,").len(), 1);
    assert_eq!(list(" ").len(), 0);
}

#[test]
fn parse_something_according_to_a_grammar_requires_the_grammar_to_use_up_the_stream() {
    // Section 5.4.1: the list of component values is matched against the
    // grammar as a whole.
    let run = |text: &str| {
        let document = parse(text);
        document
            .root()
            .parse_complete(|stream| Ok::<_, SyntaxError<'_, ()>>(stream.consume_ident()?))
            .is_ok()
    };
    assert!(run("a"));
    assert!(run(" a  "));
    assert!(!run("a b"));
    assert!(!run(""));
}

// ------------------------------------------------- refusals of the meter

/// The notation of a stylesheet that reaches every branch of 5.5.1 to 5.5.3.
const SHEET: &str = "@a _ b ; _ c { d : e ; } _ @f { g ; } h ( i ; ) j { k : l ! important } m n @o _ @p q ; r { s } t u v";

/// The notation of a block that reaches every branch of 5.5.5 and 5.5.6.
const BLOCK: &str = "a : b ; _ c _ : _ d _ ! _ important ; @x _ y { z } ; e f ; g : { h } ; i j { k } ; --l : { m } n ; o : p } q";

fn run_sheet(document: &Document<'_>) -> Vec<String> {
    document
        .root()
        .stylesheet_contents(&mut Silent)
        .map(|unit| unit.unwrap_or_else(|failed| failure(&failed)))
        .collect()
}

fn run_block(document: &Document<'_>) -> Vec<String> {
    document
        .root()
        .block_contents(&mut Silent)
        .map(|unit| unit.unwrap_or_else(|failed| failure(&failed)))
        .collect()
}

/// For every call of the meter that a run makes, refuses that call and every
/// one after it, and checks what the drivers report: the units before the
/// refusal as they were, then a refused failure and nothing else.
fn check_refusals(notation: &str, run: fn(&Document<'_>) -> Vec<String>) {
    let counting = Recording::admitting();
    let document = metered_document(&counting, notation);
    let baseline = run(&document);
    let total = counting.fallible_calls();
    assert!(
        total > 20,
        "{total} steps: the notation is long enough to matter"
    );
    assert!(!baseline.iter().any(|unit| unit.contains("refused")));
    for from in 0..=total + 2 {
        let meter = Recording::admitting();
        let document = metered_document(&meter, notation);
        meter.refuse_from(from);
        let out = run(&document);
        if from >= total {
            assert_eq!(out, baseline, "no call refused, from {from}");
            assert!(!document.is_refused());
            continue;
        }
        let (last, before) = out.split_last().expect("a refusal is reported");
        assert!(
            last.starts_with('!') && last.contains("(refused)"),
            "from {from}: {last}"
        );
        assert!(
            baseline.starts_with(before),
            "from {from}: {before:?} against {baseline:?}"
        );
        assert!(document.is_refused());
    }
}

#[test]
fn a_refusal_ends_a_stylesheet_driver_with_a_refused_unit_and_never_with_a_plausible_end() {
    check_refusals(SHEET, run_sheet);
}

#[test]
fn a_refusal_ends_a_block_driver_with_a_refused_unit_and_never_with_a_plausible_end() {
    check_refusals(BLOCK, run_block);
}

#[test]
fn a_driver_that_met_a_refusal_reports_nothing_more() {
    let meter = Recording::admitting();
    let document = metered_document(&meter, SHEET);
    meter.refuse_from(3);
    let mut grammar = Silent;
    let mut units = document.root().stylesheet_contents(&mut grammar);
    let mut seen = 0;
    for unit in units.by_ref() {
        seen += 1;
        if unit.as_ref().is_err_and(|failed| failed.is_refused()) {
            break;
        }
        assert!(seen < 100);
    }
    assert!(units.next().is_none());
    assert!(units.next().is_none());
    // The latch stays: another stream over the document is refused too.
    assert!(document.is_refused());
    assert!(document.root().consume_token().unwrap_err().is_refused());
}

#[test]
fn a_refused_failure_says_what_was_being_consumed_and_that_nothing_ended_it() {
    let meter = Recording::admitting();
    let document = metered_document(&meter, "@a b ;");
    // The second step is the first node after the at-keyword.
    meter.refuse_from(1);
    let failed = document
        .root()
        .stylesheet_contents(&mut Silent)
        .next()
        .unwrap()
        .unwrap_err();
    assert!(failed.is_refused());
    assert!(failed.error().is_refused());
    assert_eq!(failed.kind(), UnitKind::AtRule);
    assert_eq!(failed.rule_end(), None::<RuleEnd>);
}

/// Runs an entry point under a refusal at every call and checks that a refusal
/// is an error and an admitted run is the baseline.
fn check_entry<T: PartialEq + std::fmt::Debug>(
    notation: &str,
    run: impl Fn(&Document<'_>) -> Result<T, bool>,
) {
    let counting = Recording::admitting();
    let document = metered_document(&counting, notation);
    let baseline = run(&document).expect("an admitted run succeeds");
    let total = counting.fallible_calls();
    assert!(total > 3);
    for from in 0..=total + 1 {
        let meter = Recording::admitting();
        let document = metered_document(&meter, notation);
        meter.refuse_from(from);
        let result = run(&document);
        if from >= total {
            assert_eq!(result.as_ref().ok(), Some(&baseline), "from {from}");
        } else {
            assert_eq!(
                result.err(),
                Some(true),
                "from {from}: a refusal is reported"
            );
        }
    }
}

#[test]
fn the_entry_points_report_a_refusal_and_never_a_shorter_or_valid_result() {
    check_entry("a , b f( c , d ) , , e", |document| {
        document
            .root()
            .parse_comma_separated_component_values()
            .map(|groups| groups.len())
            .map_err(|error| error.is_refused())
    });
    check_entry("a _ f( b ) { c } ; d", |document| {
        document
            .root()
            .parse_list_of_component_values()
            .map(|values| values.len())
            .map_err(|error| error.is_refused())
    });
    check_entry("a , b , _ c", |document| {
        document
            .root()
            .parse_comma_separated_list(|group| {
                group.consume_ident()?;
                Ok::<_, SyntaxError<'_, ()>>(())
            })
            .map(|results| results.iter().map(Result::is_ok).collect::<Vec<_>>())
            .map_err(|error| error.is_refused())
    });
    check_entry("_ f( a b ) _", |document| {
        document
            .root()
            .parse_component_value()
            .map(|_| ())
            .map_err(|error| error.is_refused())
    });
    check_entry("_ @a b { c } _ ", |document| {
        document
            .root()
            .parse_rule(&mut Silent)
            .map_err(|failed| failed.is_refused())
    });
    check_entry("_ a b { c } _ ", |document| {
        document
            .root()
            .parse_rule(&mut Silent)
            .map_err(|failed| failed.is_refused())
    });
    check_entry("_ a : b ! important ; c", |document| {
        document
            .root()
            .parse_declaration(&mut Silent)
            .map_err(|failed| failed.is_refused())
    });
}

// ------------------------------------------------- metering and the examples of the text

/// The meter steps a driver takes over `notation`.
fn steps(notation: &str, run: fn(&Document<'_>) -> Vec<String>) -> usize {
    let meter = Recording::admitting();
    let document = metered_document(&meter, notation);
    run(&document);
    meter.fallible_calls()
}

#[test]
fn a_driver_takes_one_meter_step_for_each_node_it_examines_and_one_for_the_end() {
    // Four nodes (the at-keyword, whitespace, `b`, the semicolon) and the end.
    assert_eq!(steps("@a _ b ;", run_sheet), 5);
    // Three nodes (`a`, whitespace, the block) and the end; the block is one
    // node however much it holds.
    assert_eq!(steps("a _ { x y z }", run_sheet), 4);
    // `a : b ;` is four nodes: the declaration examines each once, and the
    // loop examines the semicolon that stopped it again to discard it, and
    // then the end.
    assert_eq!(steps("a : b ;", run_block), 6);
}

#[test]
fn a_block_that_is_a_rule_and_a_semicolon_after_it_are_the_example_of_section_5_5_3() {
    // "foo:bar {}; is definitely a rule": the declaration attempt fails because
    // a {}-block beside another value is not a declaration, and the semicolon
    // after the rule is discarded by the loop.
    assert_eq!(
        block("foo:bar {};"),
        ["rule [Ident(\"foo\") Colon Ident(\"bar\") Whitespace] {} at=0..10"]
    );
    // "foo: {...}" with the block as the whole value is a declaration.
    assert_eq!(
        block("foo: {};"),
        ["decl foo [OpenCurly<>] end=semicolon at=0..7"]
    );
}

// ------------------------------------------------- the trait shapes

#[test]
fn a_grammar_can_read_the_prelude_and_the_block_it_is_handed_with_the_stream_methods() {
    // The units are streams over the document: the grammar reads them with the
    // methods it already has, and a block of rules with `block_contents`.
    struct Media;
    impl<'i> RuleGrammar<'i> for Media {
        type Item = (String, usize);
        type Error = ();

        fn at_rule(&mut self, rule: AtRule<'i>) -> Result<Self::Item, SyntaxError<'i, ()>> {
            let mut prelude = rule.prelude();
            let condition = prelude.consume_ident()?.to_owned();
            let mut block: Stream<'i> = rule.block().ok_or_else(|| prelude.end_of_stream())?;
            let mut count = 0;
            while block.consume_after_whitespace().is_ok() {
                count += 1;
            }
            Ok((condition, count))
        }

        fn qualified_rule(
            &mut self,
            rule: QualifiedRule<'i>,
        ) -> Result<Self::Item, SyntaxError<'i, ()>> {
            Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
        }
    }
    let document = parse("@media print { a b } @media screen;");
    let units: Vec<_> = document.root().stylesheet_contents(&mut Media).collect();
    assert_eq!(units[0].as_ref().unwrap(), &("print".to_owned(), 2));
    // A grammar error is the grammar's: `@media screen;` has no block.
    assert!(units[1].as_ref().unwrap_err().error().is_end_of_stream());
}
