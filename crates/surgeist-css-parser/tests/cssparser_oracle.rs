//! A black-box differential of the `Lexer` against cssparser 0.37.0.
//!
//! The comparison is compatibility evidence only. It shows where the lexer and
//! the incumbent tokenizer disagree on a token's kind, spelling, source range
//! or block structure; it never establishes a clause, and agreement with
//! cssparser proves nothing about conformance. Every disagreement is either a
//! cited correction (a clause of the pinned CSS Syntax 3 edition that the
//! lexer follows and cssparser does not) or a bug, and a human reads the clause
//! to tell which.
//!
//! cssparser is a compiled dev-dependency used through its public API: the
//! `Parser` over a `ParserInput`, `next_including_whitespace_and_comments`,
//! `parse_nested_block` and `position`. Its outputs are the only thing read.
//!
//! The expected disagreements are committed in `tests/oracle/differences.tsv`,
//! one row per (input, field), each citing the section of the pinned edition
//! that makes the lexer's output correct. The test fails when
//!
//! 1. a disagreement appears that no row lists,
//! 2. a listed disagreement no longer appears, or
//! 3. a row names no section of the pinned edition.
//!
//! The 4,000 generated inputs are too many for a row each. A difference on one
//! is checked by class instead: it must belong to one of the classes the table
//! demonstrates with literal rows (each class names the sections that make the
//! lexer correct and is decided by the structure of the text and of the two
//! outputs), and the generator must reach every class. The spec-derived inputs
//! are checked row by row. The hand-written cases for each class are in
//! `tests/lexer.rs`, so the table is not the only record of them.
//!
//! Set `SURGEIST_ORACLE_DUMP=1` to print the disagreements as rows without
//! failing, to see what a changed input or a new case looks like.

mod common;
mod oracle;

use std::collections::BTreeSet;
use std::ops::Range;

use cssparser::{Parser, ParserInput, Token as Css};
use surgeist_css_parser::Token;
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::serialize::{Adjacency, needs_separator};

/// One token of a tree, in the notation of `tests/common/mod.rs`, with its
/// source range and, for a block opener, what the block holds and where it
/// ends (after its closing token, or at the end of the text).
#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    token: String,
    range: Range<usize>,
    block: Option<Block>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Block {
    children: Vec<Node>,
    end: usize,
}

/// Flattens a tree to pre-order `(depth, token, range)` rows followed, for a
/// block, by a `block-end` row.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Row {
    depth: usize,
    token: String,
    range: Range<usize>,
}

fn flatten(nodes: &[Node], depth: usize, rows: &mut Vec<Row>) {
    for node in nodes {
        rows.push(Row {
            depth,
            token: node.token.clone(),
            range: node.range.clone(),
        });
        if let Some(block) = &node.block {
            flatten(&block.children, depth + 1, rows);
            rows.push(Row {
                depth,
                token: "BlockEnd".into(),
                range: block.end..block.end,
            });
        }
    }
}

// ----------------------------------------------------------- the lexer's tree

fn closer_of(token: &Token<'_>) -> Option<&'static str> {
    match token {
        Token::Function(_) | Token::OpenParen => Some("CloseParen"),
        Token::OpenSquare => Some("CloseSquare"),
        Token::OpenCurly => Some("CloseCurly"),
        _ => None,
    }
}

fn lexer_tree(text: &str, options: &Options) -> Vec<Node> {
    struct Open {
        node: Node,
        closer: &'static str,
        children: Vec<Node>,
    }
    let mut lexer = Lexer::new(text, options);
    let mut stack: Vec<Open> = Vec::new();
    let mut top: Vec<Node> = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        let token = lexeme.token();
        let range = lexeme.range().start.value()..lexeme.range().end.value();
        let rendered = common::render(token);
        let closes = stack
            .last()
            .is_some_and(|open| open.closer == rendered.as_str());
        if closes {
            let mut open = stack.pop().expect("a block is open");
            open.node.block = Some(Block {
                children: open.children,
                end: range.end,
            });
            match stack.last_mut() {
                Some(parent) => parent.children.push(open.node),
                None => top.push(open.node),
            }
            continue;
        }
        let node = Node {
            token: rendered,
            range,
            block: None,
        };
        let sink = |node: Node, stack: &mut Vec<Open>, top: &mut Vec<Node>| match stack.last_mut() {
            Some(parent) => parent.children.push(node),
            None => top.push(node),
        };
        match closer_of(&token) {
            Some(closer) => stack.push(Open {
                node,
                closer,
                children: Vec::new(),
            }),
            None => sink(node, &mut stack, &mut top),
        }
    }
    // Blocks still open at the end of the text close there (5.5.9, 5.5.10:
    // EOF is a parse error that returns the block).
    while let Some(mut open) = stack.pop() {
        open.node.block = Some(Block {
            children: open.children,
            end: text.len(),
        });
        match stack.last_mut() {
            Some(parent) => parent.children.push(open.node),
            None => top.push(open.node),
        }
    }
    top
}

// ----------------------------------------------------------- cssparser's tree

fn quote(text: &str) -> String {
    // The notation of `common::render`: Debug-like, but non-ASCII as itself.
    let mut quoted = String::from("\"");
    for point in text.chars() {
        match point {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            point if point.is_ascii_control() => {
                quoted.push_str(&format!("\\u{{{:x}}}", u32::from(point)));
            }
            point => quoted.push(point),
        }
    }
    quoted.push('"');
    quoted
}

fn flag(integer: bool) -> &'static str {
    if integer { "int" } else { "num" }
}

/// The length of the number a numeric token's source text starts with, by
/// section 4.3.13 alone: a sign, digits, a fraction of a full stop and digits,
/// and an exponent of `e` or `E`, a sign and digits. Written here from the
/// section so that the oracle reads nothing of the lexer.
fn numeric_prefix(text: &str) -> usize {
    let bytes = text.as_bytes();
    let digit = |index: usize| bytes.get(index).is_some_and(u8::is_ascii_digit);
    let mut end = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        end += 1;
    }
    while digit(end) {
        end += 1;
    }
    if bytes.get(end) == Some(&b'.') && digit(end + 1) {
        end += 1;
        while digit(end) {
            end += 1;
        }
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let mut at = end + 1;
        if matches!(bytes.get(at), Some(b'+' | b'-')) {
            at += 1;
        }
        if digit(at) {
            end = at;
            while digit(end) {
                end += 1;
            }
        }
    }
    end
}

/// cssparser's token in the notation of `common::render`. Number spellings come
/// from the token's own source text: cssparser keeps no representation.
fn css_token(token: &Css<'_>, spelling: &str) -> String {
    let number = |text: &str| text.get(..numeric_prefix(text)).unwrap_or("").to_owned();
    match token {
        Css::Ident(value) => format!("Ident({})", quote(value)),
        Css::Function(value) => format!("Function({})", quote(value)),
        Css::AtKeyword(value) => format!("AtKeyword({})", quote(value)),
        Css::Hash(value) => format!("Hash({},unrestricted)", quote(value)),
        Css::IDHash(value) => format!("Hash({},id)", quote(value)),
        Css::QuotedString(value) => format!("String({})", quote(value)),
        Css::UnquotedUrl(value) => format!("Url({})", quote(value)),
        Css::BadUrl(_) => "BadUrl".into(),
        Css::BadString(_) => "BadString".into(),
        Css::Delim(value) => format!("Delim({value:?})"),
        Css::Number { int_value, .. } => {
            format!(
                "Number({},{})",
                quote(&number(spelling)),
                flag(int_value.is_some())
            )
        }
        Css::Percentage { int_value, .. } => format!(
            "Percentage({},{})",
            quote(&number(spelling)),
            flag(int_value.is_some())
        ),
        Css::Dimension {
            int_value, unit, ..
        } => format!(
            "Dimension({},{},{})",
            quote(&number(spelling)),
            flag(int_value.is_some()),
            quote(unit)
        ),
        Css::WhiteSpace(_) => "Whitespace".into(),
        Css::Comment(_) => "Comment".into(),
        Css::Colon => "Colon".into(),
        Css::Semicolon => "Semicolon".into(),
        Css::Comma => "Comma".into(),
        Css::IncludeMatch => "IncludeMatch".into(),
        Css::DashMatch => "DashMatch".into(),
        Css::PrefixMatch => "PrefixMatch".into(),
        Css::SuffixMatch => "SuffixMatch".into(),
        Css::SubstringMatch => "SubstringMatch".into(),
        Css::CDO => "Cdo".into(),
        Css::CDC => "Cdc".into(),
        Css::ParenthesisBlock => "OpenParen".into(),
        Css::SquareBracketBlock => "OpenSquare".into(),
        Css::CurlyBracketBlock => "OpenCurly".into(),
        Css::CloseParenthesis => "CloseParen".into(),
        Css::CloseSquareBracket => "CloseSquare".into(),
        Css::CloseCurlyBracket => "CloseCurly".into(),
    }
}

fn collect(parser: &mut Parser<'_, '_>, text: &str, out: &mut Vec<Node>) {
    loop {
        let start = parser.position().byte_index();
        let token = match parser.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(_) => return,
        };
        let end = parser.position().byte_index();
        let spelling = text.get(start..end).unwrap_or("");
        let mut node = Node {
            token: css_token(&token, spelling),
            range: start..end,
            block: None,
        };
        if matches!(
            token,
            Css::Function(_)
                | Css::ParenthesisBlock
                | Css::SquareBracketBlock
                | Css::CurlyBracketBlock
        ) {
            let mut children = Vec::new();
            let _: Result<(), cssparser::ParseError<'_, ()>> = parser.parse_nested_block(|inner| {
                collect(inner, text, &mut children);
                Ok(())
            });
            node.block = Some(Block {
                children,
                end: parser.position().byte_index(),
            });
        }
        out.push(node);
    }
}

fn cssparser_tree(text: &str) -> Vec<Node> {
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    let mut nodes = Vec::new();
    collect(&mut parser, text, &mut nodes);
    nodes
}

// ----------------------------------------------------------- comparison

/// One disagreement on one input.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Difference {
    input: String,
    field: &'static str,
    cssparser: String,
    surgeist: String,
}

/// The row of a tree at a given position, as the text of the comparison.
fn describe(row: &Row) -> String {
    format!("{}@{}..{}", row.token, row.range.start, row.range.end)
}

fn compare(text: &str, options: &Options) -> Vec<Difference> {
    let (mut theirs, mut ours) = (Vec::new(), Vec::new());
    flatten(&cssparser_tree(text), 0, &mut theirs);
    flatten(&lexer_tree(text, options), 0, &mut ours);
    let mut differences = Vec::new();
    let mut push = |field: &'static str, cssparser: String, surgeist: String| {
        differences.push(Difference {
            input: text.to_owned(),
            field,
            cssparser,
            surgeist,
        });
    };
    // Walk both pre-order lists by source position. A block's end sorts before
    // the token that starts where the block ends.
    let (mut i, mut j) = (0, 0);
    while i < theirs.len() || j < ours.len() {
        match (theirs.get(i), ours.get(j)) {
            (Some(left), Some(right)) if left.range.start < right.range.start => {
                push("only-cssparser", describe(left), String::new());
                i += 1;
            }
            (Some(left), Some(right)) if right.range.start < left.range.start => {
                push("only-surgeist", String::new(), describe(right));
                j += 1;
            }
            (Some(left), Some(right)) if left.token == right.token => {
                // Same token at the same place: the extent and the depth are
                // the fields left to compare.
                if left.range.end != right.range.end || left.depth != right.depth {
                    push("range", describe(left), describe(right));
                }
                i += 1;
                j += 1;
            }
            (Some(left), Some(_)) if left.token == "BlockEnd" => {
                push("only-cssparser", describe(left), String::new());
                i += 1;
            }
            (Some(_), Some(right)) if right.token == "BlockEnd" => {
                push("only-surgeist", String::new(), describe(right));
                j += 1;
            }
            (Some(left), Some(right)) => {
                // Same place, different token. The kind (and spelling) differ;
                // say which: the variant name before the first parenthesis.
                let kind = |row: &Row| row.token.split('(').next().unwrap_or("").to_owned();
                let field = if kind(left) == kind(right) {
                    "spelling"
                } else {
                    "kind"
                };
                push(field, describe(left), describe(right));
                i += 1;
                j += 1;
            }
            (Some(left), None) => {
                push("only-cssparser", describe(left), String::new());
                i += 1;
            }
            (None, Some(right)) => {
                push("only-surgeist", String::new(), describe(right));
                j += 1;
            }
            (None, None) => break,
        }
    }
    differences
}

// ----------------------------------------------------------- the classes

/// A class of disagreement: what cssparser does that the pinned edition does
/// not, and the sections that make the lexer's output correct.
struct Class {
    id: &'static str,
    sections: &'static str,
    summary: &'static str,
}

const CLASSES: [Class; 4] = [
    Class {
        id: "V-1",
        sections: "4.3.6, 4.3.15",
        summary: "A backslash that follows the whitespace in a url starts the remnants of a bad url (4.3.6), where a backslash and a right parenthesis are a valid escape that does not end it (4.3.15); cssparser ends the bad url at that parenthesis.",
    },
    Class {
        id: "V-2",
        sections: "4.3.1",
        summary: "`~`, `|`, `^`, `$` and `*` are delim tokens and `=` is another (4.3.1: anything else); cssparser makes one token of each pair.",
    },
    Class {
        id: "V-3",
        sections: "4.3.1, 4.3.3, 4.3.9",
        summary: "A hyphen-minus before a backslash and a newline starts no ident sequence (4.3.9), so it is a delim, an at-keyword or a dimension unit does not start with it (4.3.1, 4.3.3); cssparser makes an ident, an at-keyword or a unit of it.",
    },
    Class {
        id: "V-4",
        sections: "4.3.1, 4.3.9",
        summary: "A hash token's type flag is \"id\" only when the next three code points would start an ident sequence (4.3.1, 4.3.9); for a hyphen-minus before a backslash and a newline they do not, and cssparser flags it as an id.",
    },
];

fn class_of(id: &str) -> Option<&'static Class> {
    CLASSES.iter().find(|class| class.id == id)
}

/// The token name, start and end that a comparison description holds: the
/// `Name(..)@start..end` of [`describe`].
fn parts(description: &str) -> Option<(&str, usize, usize)> {
    let (token, at) = description.rsplit_once('@')?;
    let (start, end) = at.split_once("..")?;
    Some((token, start.parse().ok()?, end.parse().ok()?))
}

fn variant(token: &str) -> &str {
    token.split('(').next().unwrap_or("")
}

/// The extent of the difference: the union of the two descriptions' extents.
fn extent(difference: &Difference) -> Option<(usize, usize)> {
    let sides = [&difference.cssparser, &difference.surgeist]
        .into_iter()
        .filter_map(|side| parts(side))
        .collect::<Vec<_>>();
    let start = sides.iter().map(|side| side.1).min()?;
    let end = sides.iter().map(|side| side.2).max()?;
    Some((start, end))
}

/// Names the class of every difference of one input, or `None` where no class
/// explains it. The classes are decided by the structure of the text and of the
/// two descriptions, not by listing outputs.
fn classify(text: &str, differences: &[Difference]) -> Vec<Option<&'static str>> {
    let bytes = text.as_bytes();
    let newline = |index: usize| matches!(bytes.get(index), Some(b'\n' | b'\r' | 0x0c));
    // Where a hyphen-minus is followed by a backslash and a newline (4.3.9).
    let hyphen_before_newline: Vec<usize> = (0..bytes.len())
        .filter(|&at| bytes[at] == b'-' && bytes.get(at + 1) == Some(&b'\\') && newline(at + 2))
        .collect();
    // Where cssparser makes one token of a delim and an equals sign.
    let pair_starts: Vec<usize> = differences
        .iter()
        .filter_map(|difference| {
            let (token, start, _) = parts(&difference.cssparser)?;
            matches!(
                variant(token),
                "IncludeMatch" | "DashMatch" | "PrefixMatch" | "SuffixMatch" | "SubstringMatch"
            )
            .then_some(start)
        })
        .collect();
    // The bad urls that end in different places, as (lower end, higher end).
    let bad_urls: Vec<(usize, usize)> = differences
        .iter()
        .filter(|difference| difference.field == "range")
        .filter_map(|difference| {
            let (left, _, left_end) = parts(&difference.cssparser)?;
            let (right, _, right_end) = parts(&difference.surgeist)?;
            (variant(left) == "BadUrl" && variant(right) == "BadUrl")
                .then(|| (left_end.min(right_end), left_end.max(right_end)))
        })
        .collect();
    differences
        .iter()
        .map(|difference| {
            let (start, end) = extent(difference)?;
            let cssparser = parts(&difference.cssparser);
            let surgeist = parts(&difference.surgeist);
            // V-4: the same hash, flagged differently.
            if let (Some((left, ..)), Some((right, ..))) = (cssparser, surgeist)
                && variant(left) == "Hash"
                && variant(right) == "Hash"
                && hyphen_before_newline
                    .iter()
                    .any(|&at| (start..end).contains(&at))
            {
                return Some("V-4");
            }
            // V-2: a pair token, and the equals sign it swallowed.
            if let Some((token, ..)) = cssparser
                && difference.field == "kind"
                && matches!(
                    variant(token),
                    "IncludeMatch" | "DashMatch" | "PrefixMatch" | "SuffixMatch" | "SubstringMatch"
                )
                && surgeist.is_some_and(|(right, ..)| right.starts_with("Delim("))
            {
                return Some("V-2");
            }
            if difference.field == "only-surgeist"
                && surgeist.is_some_and(|(right, ..)| right == "Delim('=')")
                && start > 0
                && pair_starts.contains(&(start - 1))
            {
                return Some("V-2");
            }
            // V-3: a difference that covers a hyphen-minus before a backslash
            // and a newline.
            if hyphen_before_newline
                .iter()
                .any(|&at| (start..end).contains(&at))
            {
                return Some("V-3");
            }
            // V-1: the end of a bad url, or a token inside what only one of
            // the two reads as part of it.
            if let (Some((left, ..)), Some((right, ..))) = (cssparser, surgeist)
                && difference.field == "range"
                && variant(left) == "BadUrl"
                && variant(right) == "BadUrl"
            {
                return Some("V-1");
            }
            if bad_urls
                .iter()
                .any(|&(low, high)| (low..high).contains(&start))
            {
                return Some("V-1");
            }
            // V-1's consequence for block structure: once the two disagree
            // where a bad url ends, a parenthesis one of them consumed closes a
            // block for the other, so block ends and the depth of the tokens
            // after them differ.
            if let Some(&(first, _)) = bad_urls.iter().min() {
                let block_end = [&difference.cssparser, &difference.surgeist]
                    .into_iter()
                    .any(|side| {
                        parts(side).is_some_and(|(token, ..)| {
                            token == "BlockEnd" || variant(token).starts_with("Close")
                        })
                    });
                let same_token_moved = difference.field == "range"
                    && cssparser
                        .zip(surgeist)
                        .is_some_and(|(left, right)| left.0 == right.0 && left.1 == right.1);
                if start >= first && (block_end || same_token_moved) {
                    return Some("V-1");
                }
            }
            None
        })
        .collect()
}

// ----------------------------------------------------------- the table

#[derive(Clone, Debug)]
struct Entry {
    id: String,
    difference: Difference,
    section: String,
    class: String,
}

fn unescape(field: &str) -> String {
    let mut out = String::new();
    let mut chars = field.chars();
    while let Some(point) = chars.next() {
        if point != '\\' {
            out.push(point);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('f') => out.push('\u{c}'),
            Some('0') => out.push('\0'),
            Some('\\') => out.push('\\'),
            Some('u') => {
                let mut hex = String::new();
                for next in chars.by_ref() {
                    if next == '}' {
                        break;
                    }
                    if next != '{' {
                        hex.push(next);
                    }
                }
                if let Some(value) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    out.push(value);
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn escape(text: &str) -> String {
    let mut out = String::new();
    for point in text.chars() {
        match point {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{c}' => out.push_str("\\f"),
            '\0' => out.push_str("\\0"),
            point if point.is_control() => out.push_str(&format!("\\u{{{:x}}}", u32::from(point))),
            point => out.push(point),
        }
    }
    out
}

fn field_name(field: &str) -> Option<&'static str> {
    [
        "kind",
        "spelling",
        "range",
        "only-cssparser",
        "only-surgeist",
    ]
    .into_iter()
    .find(|name| *name == field)
}

fn parse_table(table: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    for (number, line) in table.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("id\t") {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            columns.len(),
            7,
            "line {}: seven tab-separated columns",
            number + 1
        );
        let field = field_name(columns[2])
            .unwrap_or_else(|| panic!("line {}: unknown field {:?}", number + 1, columns[2]));
        entries.push(Entry {
            id: columns[0].to_owned(),
            difference: Difference {
                input: unescape(columns[1]),
                field,
                cssparser: unescape(columns[3]),
                surgeist: unescape(columns[4]),
            },
            section: columns[5].to_owned(),
            class: columns[6].to_owned(),
        });
    }
    entries
}

fn row_of(id: &str, difference: &Difference, class: Option<&Class>) -> String {
    format!(
        "{id}\t{}\t{}\t{}\t{}\t{}\t{}",
        escape(&difference.input),
        difference.field,
        escape(&difference.cssparser),
        escape(&difference.surgeist),
        class.map_or("?", |class| class.sections),
        class.map_or("?", |class| class.id),
    )
}

/// The differences of every input in `inputs`, sorted and deduplicated, with the
/// class that explains each (`None` where none does).
fn differences_of(inputs: &[String]) -> Vec<(Difference, Option<&'static str>)> {
    let mut found: BTreeSet<(Difference, Option<&'static str>)> = BTreeSet::new();
    for text in inputs {
        let differences = compare(text, &Options::default());
        let classes = classify(text, &differences);
        found.extend(differences.into_iter().zip(classes));
    }
    found.into_iter().collect()
}

const TABLE: &str = include_str!("oracle/differences.tsv");

#[test]
fn every_difference_from_cssparser_is_listed_and_cites_the_pinned_edition() {
    let spec = differences_of(&oracle::spec_inputs());
    if std::env::var_os("SURGEIST_ORACLE_DUMP").is_some() {
        for (index, (difference, class)) in spec.iter().enumerate() {
            let class = class.and_then(class_of);
            eprintln!(
                "{}",
                row_of(&format!("D-{:03}", index + 1), difference, class)
            );
        }
        return;
    }
    let entries = parse_table(TABLE);

    // (3) Every row names a class and cites exactly the sections of that class,
    // all of them sections of the pinned edition, and the class is the one that
    // explains the difference.
    let explained: std::collections::BTreeMap<&Difference, Option<&'static str>> = spec
        .iter()
        .map(|(difference, class)| (difference, *class))
        .collect();
    for entry in &entries {
        let class = class_of(&entry.class)
            .unwrap_or_else(|| panic!("row {} names an unknown class {:?}", entry.id, entry.class));
        assert_eq!(
            entry.section, class.sections,
            "row {} cites sections other than its class's",
            entry.id
        );
        assert!(
            class
                .sections
                .split(',')
                .all(|section| is_cited_section(section.trim())),
            "row {} cites a section the pinned edition does not number: {}",
            entry.id,
            class.sections
        );
        if let Some(found) = explained.get(&entry.difference) {
            assert_eq!(
                *found,
                Some(class.id),
                "row {} is explained by a different class",
                entry.id
            );
        }
    }
    for class in &CLASSES {
        assert!(
            entries.iter().any(|entry| entry.class == class.id),
            "class {} has no listed difference to demonstrate it: {}",
            class.id,
            class.summary
        );
    }

    // (1) Nothing unlisted.
    let listed: BTreeSet<&Difference> = entries.iter().map(|entry| &entry.difference).collect();
    let unlisted: Vec<String> = spec
        .iter()
        .filter(|(difference, _)| !listed.contains(difference))
        .map(|(difference, class)| row_of("?", difference, class.and_then(class_of)))
        .collect();
    assert!(
        unlisted.is_empty(),
        "differences the table does not list:\n{}",
        unlisted.join("\n")
    );

    // (2) Nothing listed that vanished.
    let present: BTreeSet<&Difference> = spec.iter().map(|(difference, _)| difference).collect();
    let vanished: Vec<String> = entries
        .iter()
        .filter(|entry| !present.contains(&entry.difference))
        .map(|entry| row_of(&entry.id, &entry.difference, class_of(&entry.class)))
        .collect();
    assert!(
        vanished.is_empty(),
        "listed differences that no longer occur:\n{}",
        vanished.join("\n")
    );
}

#[test]
fn every_difference_on_a_generated_input_belongs_to_a_listed_class() {
    // The generated inputs are too many for one row each. Each difference on
    // one must be explained by a class that the table demonstrates with a
    // literal input.
    let generated = differences_of(&oracle::generated_inputs());
    let unexplained: Vec<String> = generated
        .iter()
        .filter(|(_, class)| class.is_none())
        .map(|(difference, _)| row_of("?", difference, None))
        .take(20)
        .collect();
    assert!(
        unexplained.is_empty(),
        "differences on generated inputs that no class explains:\n{}",
        unexplained.join("\n")
    );
    let entries = parse_table(TABLE);
    for (difference, class) in &generated {
        let class = class.expect("checked above");
        assert!(
            entries.iter().any(|entry| entry.class == class),
            "class {class} explains {difference:?} but the table lists none"
        );
    }
    // The generator reaches every class, so the property is not vacuous.
    for class in &CLASSES {
        assert!(
            generated.iter().any(|(_, found)| *found == Some(class.id)),
            "no generated input exercises class {}",
            class.id
        );
    }
}

/// Whether `section` names a numbered section of the pinned edition's tokenizer
/// chapter or preprocessing, such as `4.3.6` or `3.3`.
fn is_cited_section(section: &str) -> bool {
    [
        "3.3", "4.1", "4.2", "4.3.1", "4.3.2", "4.3.3", "4.3.4", "4.3.5", "4.3.6", "4.3.7",
        "4.3.8", "4.3.9", "4.3.10", "4.3.11", "4.3.12", "4.3.13", "4.3.14", "4.3.15",
    ]
    .contains(&section)
}

#[test]
fn the_comparison_itself_sees_a_difference_in_kind_range_and_structure() {
    // The harness must be able to fail: compare trees that differ in each way.
    let ours = lexer_tree("a(b)", &Options::default());
    let mut changed = ours.clone();
    changed[0].token = "Function(\"z\")".into();
    assert_ne!(ours, changed);
    let mut rows_a = Vec::new();
    let mut rows_b = Vec::new();
    flatten(&ours, 0, &mut rows_a);
    flatten(&changed, 0, &mut rows_b);
    assert_ne!(rows_a, rows_b);
    // It reports the known V-2 disagreement as two differences, and the bad-url
    // boundary as a difference of range.
    let pair = compare("a~=b", &Options::default());
    assert_eq!(pair.len(), 2);
    assert_eq!(classify("a~=b", &pair), [Some("V-2"), Some("V-2")]);
    let url = compare("url(a \\)b)c", &Options::default());
    assert!(url.iter().any(|difference| difference.field == "range"));
    // Text both tokenizers read alike, with nested blocks and every token
    // class, has no difference.
    assert_eq!(
        compare(
            "f(1 {2}) [3] a:b;#c .d @e 1px 2% 'x' url(y)",
            &Options::default()
        ),
        []
    );
}

// ============================================================ sections 6 and 9
//
// cssparser exposes three helpers with the same job as three parts of this
// crate, and they are compared through their public API only:
//
// - `parse_nth` and `anb::parse` (section 6.2),
// - `serialize_identifier` and `serialize_string`, and `serialize::identifier`
//   and `serialize::string` (CSSOM 2.1 under the round-trip rule of section 9),
// - `TokenSerializationType::needs_separator_when_before` and
//   `serialize::needs_separator` (the table of section 9).
//
// The comparison is compatibility evidence only, like the lexer's. Every
// difference is a row of `tests/oracle/syntax_differences.tsv` that cites the
// section of the pinned edition that makes this crate's answer correct, and the
// test fails on a difference no row lists, on a row that no longer occurs, and
// on a row that cites nothing. A difference on the generated notations must
// belong to a class the table demonstrates.

const SYNTAX_TABLE: &str = include_str!("oracle/syntax_differences.tsv");

/// One difference between the two implementations on one input.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SyntaxDifference {
    area: &'static str,
    input: String,
    cssparser: String,
    surgeist: String,
}

struct SyntaxRow {
    id: String,
    difference: SyntaxDifference,
    section: String,
    class: String,
}

fn area_name(area: &str) -> Option<&'static str> {
    ["anb", "adjacency", "escaping"]
        .into_iter()
        .find(|name| *name == area)
}

fn syntax_rows() -> Vec<SyntaxRow> {
    let mut rows = Vec::new();
    for (number, line) in SYNTAX_TABLE.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("id\t") {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            columns.len(),
            7,
            "line {}: seven tab-separated columns",
            number + 1
        );
        let area = area_name(columns[1])
            .unwrap_or_else(|| panic!("line {}: unknown area {:?}", number + 1, columns[1]));
        rows.push(SyntaxRow {
            id: columns[0].to_owned(),
            difference: SyntaxDifference {
                area,
                input: unescape(columns[2]),
                cssparser: unescape(columns[3]),
                surgeist: unescape(columns[4]),
            },
            section: columns[5].to_owned(),
            class: columns[6].to_owned(),
        });
    }
    rows
}

fn syntax_row_text(id: &str, difference: &SyntaxDifference, section: &str, class: &str) -> String {
    format!(
        "{id}\t{}\t{}\t{}\t{}\t{section}\t{class}",
        difference.area,
        escape(&difference.input),
        escape(&difference.cssparser),
        escape(&difference.surgeist),
    )
}

/// Whether `section` names a clause of the pinned edition that decides one of
/// these areas: the `An+B` type (6.2), its serialization (9.1), the serialization
/// chapter and its table (9), or the token rules the table is about (4.3.1).
fn is_syntax_section(section: &str) -> bool {
    section
        .split(", ")
        .all(|part| ["4.3.1", "6.2", "9", "9.1"].contains(&part))
}

// ------------------------------------------------------------------- An+B

/// What cssparser's `parse_nth` makes of `text`, as a match of the whole input:
/// it consumes eagerly and fails on a sign it cannot finish, so only a result
/// with nothing left is a match.
fn cssparser_nth(text: &str) -> String {
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    match cssparser::parse_nth(&mut parser) {
        Ok((a, b)) if parser.is_exhausted() => format!("({a},{b})"),
        _ => "error".to_owned(),
    }
}

/// What `anb::parse` makes of `text`, as a match of the whole input. An
/// overflow is reported as such: cssparser has no such answer.
fn surgeist_nth(text: &str) -> String {
    let document = common::tree::parse(text);
    let mut stream = document.root();
    match surgeist_css_parser::anb::parse(&mut stream) {
        Ok(value) if stream.only_whitespace_remains() => format!("({},{})", value.a(), value.b()),
        Ok(_) => "error".to_owned(),
        Err(error) if error.is_overflow() => "overflow".to_owned(),
        Err(_) => "error".to_owned(),
    }
}

fn anb_differences(inputs: &[String]) -> std::collections::BTreeSet<SyntaxDifference> {
    inputs
        .iter()
        .filter_map(|text| {
            let (theirs, ours) = (cssparser_nth(text), surgeist_nth(text));
            (theirs != ours).then(|| SyntaxDifference {
                area: "anb",
                input: text.clone(),
                cssparser: theirs,
                surgeist: ours,
            })
        })
        .collect()
}

/// The notations of the specification's examples (sections 6 and 6.1) and the
/// edges of `i32`, each written as the text a grammar would hand over.
fn anb_spec_inputs() -> Vec<String> {
    [
        // Section 6.
        "2n+0",
        "even",
        "odd",
        "4n+1",
        "-1n+6",
        "-4n+10",
        // Section 6.1.
        "0n+5",
        "5",
        "1n+0",
        "n+0",
        "n",
        "2n",
        "3n-6",
        "3n + -6",
        "3n + 1",
        "+3n - 2",
        "-n+ 6",
        "+6",
        "3 n",
        "+ 2n",
        "+ 2",
        // The clauses of 6.2.
        "+n",
        "-n",
        "n-1",
        "-n-1",
        "2n-1",
        "2n- 1",
        "n- 1",
        "-n- 1",
        "2n +1",
        "2n -1",
        "n + 1",
        "-n - 1",
        // The edges of i32 (Q11: a typed overflow, not a saturated value).
        "2147483647",
        "-2147483648",
        "2147483648",
        "-2147483649",
        "2147483648n",
        "-2147483649n",
        "n+2147483648",
        "n-2147483648",
        "n-2147483649",
        "n- 2147483648",
        "n- 2147483649",
        "-n-2147483648",
        "n -2147483649",
        "99999999999999999999",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// Every concatenation of up to three pieces of notation.
fn anb_generated_inputs() -> Vec<String> {
    const PIECES: &[&str] = &[
        "n",
        "N",
        "-n",
        "+n",
        "+",
        "-",
        "odd",
        "even",
        "ODD",
        "1",
        "2",
        "23",
        "-5",
        "+7",
        "0",
        "2n",
        "-3n",
        "+4N",
        "n-",
        "n-1",
        "-n-2",
        "2n-",
        "2n-3",
        "n+",
        " ",
        "/**/",
        "2147483647",
        "2147483648",
        "-2147483648",
        "-2147483649",
        "99999999999999999999",
        ".5",
        "1e1",
        "a",
        "%",
        "x",
        "n1",
        "--n",
        "1.5n",
        "3n",
        "n-3",
    ];
    let mut inputs = Vec::new();
    for first in PIECES {
        inputs.push((*first).to_owned());
        for second in PIECES {
            inputs.push(format!("{first}{second}"));
            for third in PIECES {
                inputs.push(format!("{first}{second}{third}"));
            }
        }
    }
    inputs
}

/// The class that explains a difference on an `An+B` notation, from the shape of
/// the two answers.
///
/// - `V-7`: cssparser saturates a value outside `i32` where this crate reports
///   an overflow (6.2 defines A and B as integers), including the negative
///   `n- 2147483648`, where cssparser saturates the integer before it negates it
///   and so answers -2147483647 for the integer 2147483648 that 6.2 negates to
///   -2147483648.
fn anb_class(difference: &SyntaxDifference) -> Option<&'static str> {
    let saturated = |answer: &str| {
        answer == "error"
            || answer.contains(&i32::MAX.to_string())
            || answer.contains(&i32::MIN.to_string())
    };
    if difference.surgeist == "overflow" && saturated(&difference.cssparser) {
        return Some("V-7");
    }
    // `n- 2147483648`: the negation of 2147483648 is a value of i32.
    let theirs = difference
        .cssparser
        .strip_suffix(&format!(",{})", i32::MIN + 1));
    let ours = difference.surgeist.strip_suffix(&format!(",{})", i32::MIN));
    (theirs.is_some() && theirs == ours).then_some("V-7")
}

// ------------------------------------------------------------------- escaping

fn escaping_differences() -> std::collections::BTreeSet<SyntaxDifference> {
    let mut values = vec![String::new()];
    for code in 0_u32..=0x7f {
        let point = char::from_u32(code).unwrap();
        for value in [
            format!("{point}"),
            format!("a{point}"),
            format!("{point}a"),
            format!("-{point}"),
            format!("{point}{point}"),
        ] {
            values.push(value);
        }
    }
    for point in [
        '\u{80}',
        '\u{a0}',
        '\u{b7}',
        '\u{d7}',
        '\u{e9}',
        '\u{2603}',
        '\u{fffd}',
        '\u{fffe}',
        '\u{1f600}',
    ] {
        values.push(format!("{point}"));
        values.push(format!("a{point}"));
    }
    let mut found = std::collections::BTreeSet::new();
    for value in values {
        let mut theirs = String::new();
        let mut ours = String::new();
        cssparser::serialize_identifier(&value, &mut theirs).unwrap();
        surgeist_css_parser::serialize::identifier(&value, &mut ours).unwrap();
        if theirs != ours {
            found.insert(SyntaxDifference {
                area: "escaping",
                input: format!("identifier:{value}"),
                cssparser: theirs,
                surgeist: ours,
            });
        }
        let mut theirs = String::new();
        let mut ours = String::new();
        cssparser::serialize_string(&value, &mut theirs).unwrap();
        surgeist_css_parser::serialize::string(&value, &mut ours).unwrap();
        if theirs != ours {
            found.insert(SyntaxDifference {
                area: "escaping",
                input: format!("string:{value}"),
                cssparser: theirs,
                surgeist: ours,
            });
        }
    }
    found
}

// ------------------------------------------------------------------- adjacency

/// A token of cssparser and the adjacency of the same token here, for each
/// class of token the table or either implementation distinguishes.
fn adjacency_tokens() -> Vec<(&'static str, Adjacency, Css<'static>)> {
    vec![
        ("ident", Adjacency::Ident, Css::Ident("a".into())),
        (
            "at-keyword",
            Adjacency::AtKeyword,
            Css::AtKeyword("k".into()),
        ),
        ("hash", Adjacency::Hash, Css::IDHash("h".into())),
        (
            "dimension",
            Adjacency::Dimension,
            Css::Dimension {
                has_sign: false,
                value: 1.0,
                int_value: Some(1),
                unit: "px".into(),
            },
        ),
        (
            "number",
            Adjacency::Number,
            Css::Number {
                has_sign: false,
                value: 1.0,
                int_value: Some(1),
            },
        ),
        ("function", Adjacency::Function, Css::Function("f".into())),
        ("url", Adjacency::Url, Css::UnquotedUrl("u".into())),
        ("bad url", Adjacency::BadUrl, Css::BadUrl("x".into())),
        (
            "percentage",
            Adjacency::Percentage,
            Css::Percentage {
                has_sign: false,
                unit_value: 0.01,
                int_value: Some(1),
            },
        ),
        ("cdc", Adjacency::Cdc, Css::CDC),
        ("(", Adjacency::OpenParen, Css::ParenthesisBlock),
        ("#", Adjacency::Delim('#'), Css::Delim('#')),
        ("-", Adjacency::Delim('-'), Css::Delim('-')),
        ("@", Adjacency::Delim('@'), Css::Delim('@')),
        (".", Adjacency::Delim('.'), Css::Delim('.')),
        ("+", Adjacency::Delim('+'), Css::Delim('+')),
        ("/", Adjacency::Delim('/'), Css::Delim('/')),
        ("*", Adjacency::Delim('*'), Css::Delim('*')),
        ("%", Adjacency::Delim('%'), Css::Delim('%')),
        ("=", Adjacency::Delim('='), Css::Delim('=')),
        ("|", Adjacency::Delim('|'), Css::Delim('|')),
        ("<", Adjacency::Delim('<'), Css::Delim('<')),
        ("string", Adjacency::Other, Css::QuotedString("s".into())),
        ("comma", Adjacency::Other, Css::Comma),
        ("cdo", Adjacency::Other, Css::CDO),
    ]
}

fn adjacency_differences() -> std::collections::BTreeSet<SyntaxDifference> {
    let tokens = adjacency_tokens();
    let mut found = std::collections::BTreeSet::new();
    for (first_name, first, first_css) in &tokens {
        for (second_name, second, second_css) in &tokens {
            let theirs = first_css
                .serialization_type()
                .needs_separator_when_before(second_css.serialization_type());
            let ours = needs_separator(*first, *second);
            if theirs != ours {
                found.insert(SyntaxDifference {
                    area: "adjacency",
                    input: format!("{first_name} then {second_name}"),
                    cssparser: theirs.to_string(),
                    surgeist: ours.to_string(),
                });
            }
        }
    }
    found
}

// ------------------------------------------------------------------- the tests

fn committed(area: &str) -> Vec<SyntaxRow> {
    syntax_rows()
        .into_iter()
        .filter(|row| row.difference.area == area)
        .collect()
}

fn check_listed(area: &str, found: &std::collections::BTreeSet<SyntaxDifference>) {
    let rows = committed(area);
    let listed: std::collections::BTreeSet<SyntaxDifference> =
        rows.iter().map(|row| row.difference.clone()).collect();
    assert_eq!(listed.len(), rows.len(), "{area}: a row is listed twice");
    let unlisted: Vec<String> = found
        .difference(&listed)
        .map(|difference| syntax_row_text("?", difference, "?", "?"))
        .collect();
    assert!(
        unlisted.is_empty(),
        "{area}: differences that no row lists:\n{}",
        unlisted.join("\n")
    );
    let vanished: Vec<String> = listed
        .difference(found)
        .map(|difference| syntax_row_text("?", difference, "?", "?"))
        .collect();
    assert!(
        vanished.is_empty(),
        "{area}: listed differences that no longer occur:\n{}",
        vanished.join("\n")
    );
    for row in rows {
        assert!(
            is_syntax_section(&row.section),
            "{}: cites {:?}, which is not a clause that decides {area}",
            row.id,
            row.section
        );
        assert!(
            !row.class.is_empty() && row.class != "?",
            "{}: no class",
            row.id
        );
    }
}

#[test]
fn every_an_plus_b_difference_from_cssparser_is_listed_and_cites_the_pinned_edition() {
    let found = anb_differences(&anb_spec_inputs());
    if std::env::var_os("SURGEIST_ORACLE_DUMP").is_some() {
        for (index, difference) in found.iter().enumerate() {
            let class = anb_class(difference).unwrap_or("?");
            eprintln!(
                "{}",
                syntax_row_text(&format!("S-{:03}", index + 1), difference, "6.2", class)
            );
        }
        return;
    }
    check_listed("anb", &found);
}

#[test]
fn every_an_plus_b_difference_on_a_generated_notation_belongs_to_a_listed_class() {
    let found = anb_differences(&anb_generated_inputs());
    let unexplained: Vec<String> = found
        .iter()
        .filter(|difference| anb_class(difference).is_none())
        .map(|difference| syntax_row_text("?", difference, "?", "?"))
        .take(20)
        .collect();
    assert!(
        unexplained.is_empty(),
        "differences on generated notations that no class explains:\n{}",
        unexplained.join("\n")
    );
    let rows = committed("anb");
    for difference in &found {
        let class = anb_class(difference).expect("checked above");
        assert!(
            rows.iter().any(|row| row.class == class),
            "class {class} explains {difference:?} but the table lists none"
        );
    }
    // The generator reaches the class, so the property is not vacuous.
    assert!(
        found
            .iter()
            .any(|difference| anb_class(difference) == Some("V-7"))
    );
    assert!(found.len() > 100, "{} differences", found.len());
}

#[test]
fn the_escaping_functions_agree_with_cssparsers_on_every_input() {
    // Under the 2021 non-ASCII rule the two write the same text for every
    // identifier and string of the corpus, so the table lists no row. The 2026
    // ranges will add rows for the code points they exclude (CSSOM 2.1 writes
    // those raw; the round-trip rule of section 9 escapes them).
    check_listed("escaping", &escaping_differences());
}

#[test]
fn every_difference_in_the_table_of_section_9_from_cssparsers_is_listed_and_cited() {
    check_listed("adjacency", &adjacency_differences());
}

#[test]
fn the_syntax_comparisons_themselves_see_a_difference() {
    // The harness must be able to fail: the saturation of an overflow is a
    // difference, and so is a pair the two tables disagree on.
    assert_eq!(cssparser_nth("2147483648"), "(0,2147483647)");
    assert_eq!(surgeist_nth("2147483648"), "overflow");
    assert_eq!(cssparser_nth("n+"), surgeist_nth("n+"));
    assert_eq!(cssparser_nth("2n+1"), "(2,1)");
    assert!(anb_differences(&["2147483648".to_owned()]).len() == 1);
    assert!(anb_differences(&["2n+1".to_owned()]).is_empty());
    let number = Css::Number {
        has_sign: false,
        value: 1.0,
        int_value: Some(1),
    };
    assert!(
        number
            .serialization_type()
            .needs_separator_when_before(Css::Delim('-').serialization_type())
    );
    assert!(!needs_separator(Adjacency::Number, Adjacency::Delim('-')));
}
