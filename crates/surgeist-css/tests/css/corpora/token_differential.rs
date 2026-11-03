//! A black-box differential of the `Lexer` against cssparser 0.37.0 over the
//! CSSTree fixture sources and the CSS text that the WPT parsing vectors feed
//! the parser.
//!
//! The comparison is compatibility evidence only. It shows where the lexer and
//! the other tokenizer disagree on a token's kind, spelling, source range or
//! block structure; it never establishes a clause, and agreement with cssparser
//! proves nothing about conformance. Every disagreement is either a cited
//! correction (a clause of the pinned CSS Syntax 3 edition that the lexer
//! follows and cssparser does not) or a bug, and a human reads the clause to
//! tell which.
//!
//! cssparser is a compiled dev-dependency used through its public API: the
//! `Parser` over a `ParserInput`, `next_including_whitespace_and_comments`,
//! `parse_nested_block` and `position`. Its outputs are the only thing read.
//!
//! The disagreements are committed in `token_differential/differences.tsv`, one
//! row per (input, field), each citing the section of the pinned edition that
//! makes the lexer's output correct and naming its class. The test fails when
//!
//! 1. a disagreement appears that no row lists,
//! 2. a listed disagreement does not appear, or
//! 3. a row names no section of the pinned edition.
//!
//! Set `SURGEIST_TOKEN_DIFFERENTIAL_DUMP=1` to print the disagreements as rows
//! without failing, to see what a changed corpus looks like.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use cssparser::{Parser, ParserInput, Token as Css};
use surgeist_css_parser::Token;
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::token::{HashKind, Number, NumberKind};

use crate::css::support::csstree::load_csstree_corpus;

// ----------------------------------------------------------- trees

/// One token of a tree, with its source range and, for a block opener, what the
/// block holds and where it ends (after its closing token, or at the end of the
/// text).
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

fn quote(text: &str) -> String {
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

fn number_flag(number: Number<'_>) -> &'static str {
    match number.kind() {
        NumberKind::Integer => "int",
        NumberKind::Number => "num",
    }
}

fn render(token: Token<'_>) -> String {
    match token {
        Token::Ident(value) => format!("Ident({})", quote(value)),
        Token::Function(value) => format!("Function({})", quote(value)),
        Token::AtKeyword(value) => format!("AtKeyword({})", quote(value)),
        Token::Hash { value, kind } => format!(
            "Hash({},{})",
            quote(value),
            match kind {
                HashKind::Id => "id",
                HashKind::Unrestricted => "unrestricted",
            }
        ),
        Token::String(value) => format!("String({})", quote(value)),
        Token::BadString => "BadString".into(),
        Token::Url(value) => format!("Url({})", quote(value)),
        Token::BadUrl => "BadUrl".into(),
        Token::Delim(value) => format!("Delim({value:?})"),
        Token::Number(number) => format!(
            "Number({},{})",
            quote(number.representation()),
            number_flag(number)
        ),
        Token::Percentage(number) => format!(
            "Percentage({},{})",
            quote(number.representation()),
            number_flag(number)
        ),
        Token::Dimension { number, unit } => format!(
            "Dimension({},{},{})",
            quote(number.representation()),
            number_flag(number),
            quote(unit)
        ),
        Token::UnicodeRange(range) => {
            format!("UnicodeRange({:#x},{:#x})", range.start(), range.end())
        }
        Token::Whitespace => "Whitespace".into(),
        Token::Comment => "Comment".into(),
        Token::Colon => "Colon".into(),
        Token::Semicolon => "Semicolon".into(),
        Token::Comma => "Comma".into(),
        Token::OpenSquare => "OpenSquare".into(),
        Token::OpenParen => "OpenParen".into(),
        Token::OpenCurly => "OpenCurly".into(),
        Token::CloseSquare => "CloseSquare".into(),
        Token::CloseParen => "CloseParen".into(),
        Token::CloseCurly => "CloseCurly".into(),
        Token::Cdo => "Cdo".into(),
        Token::Cdc => "Cdc".into(),
    }
}

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
        let rendered = render(token);
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
        match closer_of(&token) {
            Some(closer) => stack.push(Open {
                node,
                closer,
                children: Vec::new(),
            }),
            None => match stack.last_mut() {
                Some(parent) => parent.children.push(node),
                None => top.push(node),
            },
        }
    }
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
        Css::Number { int_value, .. } => format!(
            "Number({},{})",
            quote(&number(spelling)),
            flag(int_value.is_some())
        ),
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
/// not, and the sections that make the lexer's output correct. The ids are the
/// deviations of the design's section 6 (V-2, V-5).
struct Class {
    id: &'static str,
    sections: &'static str,
}

const CLASSES: [Class; 2] = [
    // `~`, `|`, `^`, `$` and `*` are delim tokens and `=` is another (4.3.1:
    // anything else); cssparser makes one token of each pair.
    Class {
        id: "V-2",
        sections: "4.3.1",
    },
    // A non-ASCII code point that the ranges of 4.2 do not list is not an
    // ident code point, so it ends an ident sequence (4.3.12) and is a delim
    // (4.3.1); cssparser admits every code point from U+0080 in a name.
    Class {
        id: "V-5",
        sections: "4.2, 4.3.1, 4.3.12",
    },
];

fn class_of(id: &str) -> Option<&'static Class> {
    CLASSES.iter().find(|class| class.id == id)
}

/// The "non-ASCII ident code point" ranges of section 4.2, copied from the
/// pinned edition's text.
const ADMITTED: [(u32, u32); 15] = [
    (0x00b7, 0x00b7),
    (0x00c0, 0x00d6),
    (0x00d8, 0x00f6),
    (0x00f8, 0x037d),
    (0x037f, 0x1fff),
    (0x200c, 0x200c),
    (0x200d, 0x200d),
    (0x203f, 0x203f),
    (0x2040, 0x2040),
    (0x2070, 0x218f),
    (0x2c00, 0x2fef),
    (0x3001, 0xd7ff),
    (0xf900, 0xfdcf),
    (0xfdf0, 0xfffd),
    (0x1_0000, 0x10_ffff),
];

fn admitted(point: char) -> bool {
    let value = u32::from(point);
    ADMITTED
        .iter()
        .any(|&(low, high)| (low..=high).contains(&value))
}

/// The token name, start and end that a comparison description holds.
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

fn is_pair_token(token: &str) -> bool {
    matches!(
        variant(token),
        "IncludeMatch" | "DashMatch" | "PrefixMatch" | "SuffixMatch" | "SubstringMatch"
    )
}

fn is_block_end(description: &str) -> bool {
    parts(description)
        .is_some_and(|(token, ..)| token == "BlockEnd" || variant(token).starts_with("Close"))
}

/// Names the class of every difference of one input, or `None` where no class
/// explains it. The classes are decided by the structure of the text and of the
/// two descriptions, not by listing outputs.
fn classify(text: &str, differences: &[Difference]) -> Vec<Option<&'static str>> {
    // V-2: where cssparser makes one token of a delim and an equals sign.
    let pair_starts: Vec<usize> = differences
        .iter()
        .filter_map(|difference| {
            let (token, start, _) = parts(&difference.cssparser)?;
            is_pair_token(token).then_some(start)
        })
        .collect();
    // V-5: the spans of the name-like tokens of cssparser that hold a code
    // point 4.2 leaves out; the token ends at that code point here.
    let name_tokens: Vec<(usize, usize, bool)> = differences
        .iter()
        .filter_map(|difference| {
            let (token, start, end) = parts(&difference.cssparser)?;
            matches!(
                variant(token),
                "Ident" | "AtKeyword" | "Hash" | "Dimension" | "Function"
            )
            .then_some(())?;
            let spelling = text.get(start..end)?;
            spelling
                .chars()
                .any(|point| u32::from(point) >= 0x80 && !admitted(point))
                .then_some((
                    start,
                    end,
                    variant(token) == "Function" || text[end..].starts_with('('),
                ))
        })
        .collect();
    let first_opening = name_tokens
        .iter()
        .filter(|&&(_, _, opens)| opens)
        .map(|&(start, ..)| start)
        .min();
    let mut classes: Vec<Option<&'static str>> = differences
        .iter()
        .map(|difference| {
            let cssparser = parts(&difference.cssparser);
            let surgeist = parts(&difference.surgeist);
            // V-2: a pair token read here as the delim of its first code
            // point, and the equals sign it swallowed.
            if let Some((token, start, _)) = cssparser
                && difference.field == "kind"
                && is_pair_token(token)
                && let Some(first) = text.get(start..).and_then(|rest| rest.chars().next())
                && surgeist.is_some_and(|(right, low, high)| {
                    right == format!("Delim('{first}')") && low == start && high == start + 1
                })
            {
                return Some("V-2");
            }
            if difference.field == "only-surgeist"
                && surgeist.is_some_and(|(right, start, _)| {
                    right == "Delim('=')" && start > 0 && pair_starts.contains(&(start - 1))
                })
            {
                return Some("V-2");
            }
            None
        })
        .collect();
    // V-5: a difference that touches a name cssparser kept whole, or the extent
    // of a difference that does, follows from where the name ends here: the
    // tokens after it start over. The pass repeats until no other difference
    // joins.
    let touches = |(low, high): (usize, usize), (start, end): (usize, usize)| {
        if start == end {
            low <= start && start <= high
        } else {
            start < high && low < end
        }
    };
    let mut spans: Vec<(usize, usize)> = name_tokens
        .iter()
        .map(|&(start, end, _)| (start, end))
        .collect();
    let mut joined = vec![false; differences.len()];
    loop {
        let mut grew = false;
        for (index, difference) in differences.iter().enumerate() {
            let Some(found) = extent(difference) else {
                continue;
            };
            if !joined[index] && spans.iter().any(|&span| touches(span, found)) {
                joined[index] = true;
                classes[index] = classes[index].or(Some("V-5"));
                spans.push(found);
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    // Where cssparser keeps a name whole that ends here and opens a block, the
    // block can stay open to the end of the text while the other reading has
    // closed it: from there on, block ends differ and the same tokens sit at a
    // different depth.
    if let Some(first) = first_opening {
        for (index, difference) in differences.iter().enumerate() {
            let Some((start, _)) = extent(difference) else {
                continue;
            };
            let block_end =
                is_block_end(&difference.cssparser) || is_block_end(&difference.surgeist);
            let same_token_deeper =
                difference.field == "range" && difference.cssparser == difference.surgeist;
            if classes[index].is_none() && start >= first && (block_end || same_token_deeper) {
                classes[index] = Some("V-5");
            }
        }
    }
    classes
}

// ----------------------------------------------------------- the table

#[derive(Clone, Debug)]
struct Entry {
    id: String,
    difference: Difference,
    section: String,
    class: String,
}

/// The table writes every non-ASCII code point and every control as `\u{..}`,
/// a backslash as two, and a line feed, carriage return, tab and form feed as
/// `\n`, `\r`, `\t` and `\f`.
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
                let value = u32::from_str_radix(&hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .unwrap_or_else(|| panic!("table escape \\u{{{hex}}} is no code point"));
                out.push(value);
            }
            other => panic!("table escape \\{other:?} is unknown"),
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
            point if point.is_control() || !point.is_ascii() => {
                out.push_str(&format!("\\u{{{:x}}}", u32::from(point)));
            }
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

/// Whether `section` names a numbered section of the pinned edition's
/// preprocessing and tokenization chapters, such as `4.3.6` or `3.3`.
fn is_cited_section(section: &str) -> bool {
    [
        "3.3", "4.1", "4.2", "4.3.1", "4.3.2", "4.3.3", "4.3.4", "4.3.5", "4.3.6", "4.3.7",
        "4.3.8", "4.3.9", "4.3.10", "4.3.11", "4.3.12", "4.3.13", "4.3.14", "4.3.15",
    ]
    .contains(&section)
}

const TABLE: &str = include_str!("token_differential/differences.tsv");

// ----------------------------------------------------------- the inputs

/// The CSS text of every CSSTree case, in corpus order.
fn csstree_inputs() -> Vec<String> {
    load_csstree_corpus()
        .expect("committed CSSTree corpus should validate")
        .case_inputs()
        .map(str::to_owned)
        .collect()
}

/// The CSS text that `wpt_parsing.rs` feeds the parser, built the way that file
/// builds it: the `src` descriptor of each font case in an `@font-face` rule
/// (the `tech()` serialization replay reads the same rows), each nesting
/// selector case in its parent rule, each declaration-order case as authored,
/// and each replayed nested selector text case in its parent rule.
fn wpt_inputs() -> Vec<String> {
    fn cases(text: &str) -> Vec<serde_json::Value> {
        let vectors: serde_json::Value =
            serde_json::from_str(text).expect("literal WPT vector schema");
        vectors["cases"].as_array().expect("vector cases").clone()
    }
    fn field<'a>(value: &'a serde_json::Value, name: &str) -> &'a str {
        value[name].as_str().expect("string field")
    }
    let mut inputs = Vec::new();
    for case in cases(include_str!("../../corpus/wpt/font-face-src-presence.json")) {
        inputs.push(format!("@font-face {{ src: {}}}", field(&case, "src")));
    }
    for case in cases(include_str!(
        "../../corpus/wpt/nesting-selector-presence.json"
    )) {
        inputs.push(format!(
            "{} {{ {} {{ color: green; }} }}",
            field(&case, "parent"),
            field(&case, "selector")
        ));
    }
    for case in cases(include_str!(
        "../../corpus/wpt/nesting-declaration-order.json"
    )) {
        inputs.push(field(&case, "input").to_owned());
    }
    for case in cases(include_str!(
        "../../corpus/wpt/deferred-cssom-and-execution.json"
    )) {
        if field(&case, "disposition") == "replayed-nested-selector-text" {
            inputs.push(format!(
                "{} {{ {} {{ color: green; }}}}",
                field(&case["input"], "parent"),
                field(&case["input"], "selector")
            ));
        }
    }
    inputs
}

/// The differences of every distinct input in `inputs`, sorted, with the class
/// that explains each (`None` where none does).
fn differences_of(inputs: &[String]) -> Vec<(Difference, Option<&'static str>)> {
    let distinct: BTreeSet<&String> = inputs.iter().collect();
    let mut found: BTreeSet<(Difference, Option<&'static str>)> = BTreeSet::new();
    for text in distinct {
        let differences = compare(text, &Options::default());
        let classes = classify(text, &differences);
        found.extend(differences.into_iter().zip(classes));
    }
    found.into_iter().collect()
}

// ----------------------------------------------------------- the audit

/// Every way the table fails to account for `found`: a row that names an
/// unknown class or cites sections other than its class's or sections the
/// pinned edition does not number, a row whose class is not the one that
/// explains its difference, a class no row demonstrates, a difference no row
/// lists, a row listed twice, and a listed difference that does not occur.
fn audit(entries: &[Entry], found: &[(Difference, Option<&'static str>)]) -> Vec<String> {
    let mut problems = Vec::new();
    let explained: BTreeMap<&Difference, Option<&'static str>> = found
        .iter()
        .map(|(difference, class)| (difference, *class))
        .collect();
    for entry in entries {
        let Some(class) = class_of(&entry.class) else {
            problems.push(format!(
                "row {} names an unknown class {:?}",
                entry.id, entry.class
            ));
            continue;
        };
        if entry.section != class.sections {
            problems.push(format!(
                "row {} cites {:?}, not the sections of its class: {}",
                entry.id, entry.section, class.sections
            ));
        }
        if !entry.section.split(", ").all(is_cited_section) {
            problems.push(format!(
                "row {} cites a section the pinned edition does not number: {:?}",
                entry.id, entry.section
            ));
        }
        if let Some(found) = explained.get(&entry.difference)
            && *found != Some(class.id)
        {
            problems.push(format!(
                "row {} is class {} but the difference is explained by {:?}",
                entry.id, class.id, found
            ));
        }
    }
    for class in &CLASSES {
        if !entries.iter().any(|entry| entry.class == class.id) {
            problems.push(format!("class {} has no listed difference", class.id));
        }
    }
    let mut listed: BTreeSet<&Difference> = BTreeSet::new();
    for entry in entries {
        if !listed.insert(&entry.difference) {
            problems.push(format!("row {} is listed twice", entry.id));
        }
    }
    for (difference, class) in found {
        if !listed.contains(difference) {
            problems.push(format!(
                "unlisted difference: {}",
                row_of("?", difference, class.and_then(class_of))
            ));
        }
    }
    let present: BTreeSet<&Difference> = found.iter().map(|(difference, _)| difference).collect();
    for entry in entries {
        if !present.contains(&entry.difference) {
            problems.push(format!(
                "listed difference that does not occur: {}",
                row_of(&entry.id, &entry.difference, class_of(&entry.class))
            ));
        }
    }
    problems
}

fn all_inputs() -> Vec<String> {
    let mut inputs = csstree_inputs();
    inputs.extend(wpt_inputs());
    inputs
}

// ----------------------------------------------------------- the tests

#[test]
fn the_corpora_hold_the_inputs_the_comparison_reads() {
    let csstree = csstree_inputs();
    assert_eq!(csstree.len(), 935, "CSSTree cases");
    assert_eq!(
        csstree.iter().collect::<BTreeSet<_>>().len(),
        901,
        "distinct CSSTree sources"
    );
    let wpt = wpt_inputs();
    assert_eq!(wpt.len(), 109 + 32 + 5 + 31, "WPT texts fed to the parser");
    assert_eq!(
        wpt.iter().collect::<BTreeSet<_>>().len(),
        176,
        "distinct WPT texts"
    );
}

#[test]
fn every_corpus_token_difference_from_cssparser_is_listed_and_cites_the_pinned_edition() {
    let found = differences_of(&all_inputs());
    if std::env::var_os("SURGEIST_TOKEN_DIFFERENTIAL_DUMP").is_some() {
        for (index, (difference, class)) in found.iter().enumerate() {
            eprintln!(
                "{}",
                row_of(
                    &format!("C-{:03}", index + 1),
                    difference,
                    class.and_then(class_of)
                )
            );
        }
        return;
    }
    let problems = audit(&parse_table(TABLE), &found);
    assert!(
        problems.is_empty(),
        "the difference table does not account for the corpora:\n{}",
        problems.join("\n")
    );
}

#[test]
fn the_comparison_sees_a_difference_in_kind_range_and_structure() {
    let options = Options::default();
    // Trees that differ in a token, a range and a block flatten differently.
    let ours = lexer_tree("a(b)", &options);
    let mut changed = ours.clone();
    changed[0].token = "Function(\"z\")".into();
    let mut moved = ours.clone();
    moved[0].range = 0..1;
    let mut reopened = ours.clone();
    reopened[0].block = None;
    let flat = |nodes: &[Node]| {
        let mut rows = Vec::new();
        flatten(nodes, 0, &mut rows);
        rows
    };
    assert_ne!(flat(&ours), flat(&changed));
    assert_ne!(flat(&ours), flat(&moved));
    assert_ne!(flat(&ours), flat(&reopened));
    // A kind, a spelling, a range and a block end are each reported.
    let pair = compare("a~=b", &options);
    assert_eq!(
        pair.iter().map(|d| d.field).collect::<Vec<_>>(),
        ["kind", "only-surgeist"]
    );
    let spelling = compare("a\u{a0}b", &options);
    assert!(spelling.iter().any(|d| d.field == "spelling"));
    let url = compare("url(a \\)b)c", &options);
    assert!(url.iter().any(|d| d.field == "range"));
    assert!(
        url.iter()
            .any(|d| is_block_end(&d.cssparser) || is_block_end(&d.surgeist)),
        "{url:?}"
    );
    // Text both tokenizers read alike, with nested blocks and every token
    // class, has no difference.
    assert_eq!(
        compare("f(1 {2}) [3] a:b;#c .d @e 1px 2% 'x' url(y)", &options),
        []
    );
}

fn injected(field: &'static str, cssparser: &str, surgeist: &str) -> Difference {
    Difference {
        input: String::new(),
        field,
        cssparser: cssparser.to_owned(),
        surgeist: surgeist.to_owned(),
    }
}

#[test]
fn the_pair_class_explains_a_merged_pair_and_nothing_else() {
    let options = Options::default();
    let text = "[a~=b] c =";
    let real = compare(text, &options);
    assert_eq!(real.len(), 2, "{real:?}");
    assert_eq!(classify(text, &real), [Some("V-2"), Some("V-2")]);
    for (name, extra) in [
        (
            "a kind that is not a pair",
            injected("kind", "Ident(\"c\")@7..8", "Number(\"1\",int)@7..8"),
        ),
        (
            "a pair read as something other than a delim",
            injected("kind", "DashMatch@7..9", "Ident(\"c\")@7..8"),
        ),
        (
            "a pair read as a delim of another code point",
            injected("kind", "DashMatch@7..9", "Delim('?')@7..8"),
        ),
        (
            "a pair read as a delim of another extent",
            injected("kind", "DashMatch@7..9", "Delim('c')@7..9"),
        ),
        (
            "an equals sign no pair precedes",
            injected("only-surgeist", "", "Delim('=')@9..10"),
        ),
        (
            "a range",
            injected("range", "IncludeMatch@1..3", "IncludeMatch@1..4"),
        ),
    ] {
        let mut all = real.clone();
        all.push(extra);
        let classes = classify(text, &all);
        assert_eq!(
            classes[..real.len()],
            [Some("V-2"), Some("V-2")],
            "{name}: the real differences keep their class"
        );
        assert_eq!(
            classes[real.len()],
            None,
            "{name}: a difference outside the class surfaces as unexplained"
        );
    }
}

#[test]
fn the_excluded_code_point_class_explains_what_follows_from_a_name_and_nothing_else() {
    let options = Options::default();
    // `x`, U+00A0 (two bytes) and `y` are one name for cssparser: bytes 0..4.
    let text = "x\u{a0}y  z (1) w";
    let real = compare(text, &options);
    assert!(
        !real.is_empty(),
        "the excluded code point makes a difference"
    );
    assert!(
        classify(text, &real)
            .iter()
            .all(|class| *class == Some("V-5")),
        "{real:?}"
    );
    for (name, extra) in [
        (
            "a kind elsewhere",
            injected("kind", "Ident(\"z\")@6..7", "Number(\"1\",int)@6..7"),
        ),
        (
            "a token only one side reads",
            injected("only-cssparser", "Ident(\"w\")@16..17", ""),
        ),
        (
            "a block end elsewhere",
            injected("kind", "BlockEnd@14..15", "CloseParen@14..15"),
        ),
        (
            "a depth alone elsewhere",
            injected("range", "Ident(\"w\")@16..17", "Ident(\"w\")@16..17"),
        ),
    ] {
        let mut all = real.clone();
        all.push(extra);
        let classes = classify(text, &all);
        assert_eq!(
            classes[..real.len()],
            vec![Some("V-5"); real.len()],
            "{name}: the real differences keep their class"
        );
        assert_eq!(
            classes[real.len()],
            None,
            "{name}: a difference outside the class surfaces as unexplained"
        );
    }
    // A difference that touches the name is still explained by it.
    let mut touching = real.clone();
    touching.push(injected(
        "kind",
        "Ident(\"y\")@3..4",
        "Number(\"1\",int)@3..4",
    ));
    assert_eq!(classify(text, &touching).last(), Some(&Some("V-5")));
    // A name that opens a block for cssparser leaves that block open where
    // this lexer has read a url token: the block ends that follow are
    // consequences of the name.
    let text = "\u{a0}url(a) b {c}";
    let found = compare(text, &options);
    assert!(!found.is_empty(), "{text:?}");
    assert!(
        classify(text, &found)
            .iter()
            .all(|class| *class == Some("V-5")),
        "{text:?}: {found:?}"
    );
    // Neither class claims the other's differences.
    let pair = compare("a~=b", &options);
    assert!(classify("a~=b", &pair).iter().all(|c| *c == Some("V-2")));
}

#[test]
fn the_audit_rejects_a_table_that_does_not_account_for_the_corpora() {
    let found = differences_of(&all_inputs());
    let entries = parse_table(TABLE);
    assert_eq!(audit(&entries, &found), Vec::<String>::new());

    // (i) A difference no row lists.
    let mut without = entries.clone();
    let removed = without.remove(0);
    let problems = audit(&without, &found);
    assert!(
        problems.iter().any(|p| p.starts_with("unlisted difference")
            && p.contains(&escape(&removed.difference.input))),
        "{problems:?}"
    );

    // (ii) A listed difference that does not occur.
    let mut bogus = entries.clone();
    bogus.push(Entry {
        id: "C-999".into(),
        difference: injected("kind", "Ident(\"q\")@0..1", "Delim('q')@0..1"),
        section: "4.3.1".into(),
        class: "V-2".into(),
    });
    let problems = audit(&bogus, &found);
    assert!(
        problems
            .iter()
            .any(|p| p.starts_with("listed difference that does not occur")),
        "{problems:?}"
    );

    // (iii) A row without a section citation, with a section of another
    // class, with a section the edition does not number, and with a class
    // that does not explain its difference.
    for (name, section, class) in [
        ("no citation", "", "V-2"),
        ("another class's sections", "4.2", "V-2"),
        ("a section the edition lacks", "4.3.99", "V-2"),
        (
            "a class that does not explain it",
            "4.2, 4.3.1, 4.3.12",
            "V-5",
        ),
    ] {
        let mut edited = entries.clone();
        let target = edited
            .iter_mut()
            .find(|entry| entry.class == "V-2")
            .expect("a V-2 row");
        target.section = section.to_owned();
        target.class = class.to_owned();
        assert!(
            !audit(&edited, &found).is_empty(),
            "{name}: the audit accepts it"
        );
    }
    let mut unknown = entries.clone();
    unknown[0].class = "V-99".into();
    assert!(!audit(&unknown, &found).is_empty());

    // A row listed twice.
    let mut twice = entries.clone();
    twice.push(entries[0].clone());
    assert!(
        audit(&twice, &found)
            .iter()
            .any(|p| p.contains("listed twice"))
    );
}
