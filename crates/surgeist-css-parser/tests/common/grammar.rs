//! A grammar that describes what a driver handed it, for the tests of the rule
//! algorithms.
//!
//! The descriptions are a notation for writing expectations by hand from
//! sections 5.4 and 5.5; the grammar has no behavior of its own beyond saying
//! "not valid" for the names it is told to reject.

#![allow(dead_code)]

use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, Declaration, Failed, QualifiedRule, RuleEnd, RuleGrammar, UnitKind,
};
use surgeist_css_parser::{Document, Offset, Stream, SyntaxError, Token};

use super::render;
use super::tree::parse;

/// The grammar's own error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rejected;

/// Describes every unit it is handed, and rejects the ones it is told to.
#[derive(Default)]
pub struct Dump {
    /// At-rule names (as written) that are not valid.
    pub reject_at: Vec<&'static str>,
    /// Whether every qualified rule is not valid.
    pub reject_qualified: bool,
    /// Declaration names (as written) that are not valid.
    pub reject_declarations: Vec<&'static str>,
    /// Whether the block of an at-rule is described as a block's contents
    /// (declarations and rules) and not as component values.
    pub blocks_as_contents: bool,
}

impl Dump {
    pub fn accepting() -> Self {
        Self::default()
    }

    pub fn rejecting_qualified() -> Self {
        Self {
            reject_qualified: true,
            ..Self::default()
        }
    }
}

/// Renders the rest of the stream as the tokens it holds, a function or block
/// as its opening token, its contents between `<` and `>`, in order and
/// separated by spaces. Comments are transparent, as the stream makes them.
pub fn show(stream: &Stream<'_>) -> String {
    drain(&mut stream.clone())
}

fn drain(stream: &mut Stream<'_>) -> String {
    let mut parts = Vec::new();
    while let Ok(token) = stream.consume_token() {
        if stream.unentered_block().is_some() {
            let inner = stream
                .within_block(|inner| Ok::<_, SyntaxError<'_, ()>>(drain(inner)))
                .expect("a block is waiting");
            parts.push(format!("{}<{inner}>", render(token)));
        } else {
            parts.push(render(token));
        }
    }
    parts.join(" ")
}

fn end_name(end: RuleEnd) -> &'static str {
    match end {
        RuleEnd::Semicolon => "semicolon",
        RuleEnd::Block => "block",
        RuleEnd::Eof => "eof",
        RuleEnd::CloseBrace => "close-brace",
        _ => "other",
    }
}

fn span(range: Option<std::ops::Range<Offset>>) -> String {
    range.map_or_else(
        || "?".to_owned(),
        |range| format!("{}..{}", range.start.value(), range.end.value()),
    )
}

impl<'i> RuleGrammar<'i> for Dump {
    type Item = String;
    type Error = Rejected;

    fn at_rule(&mut self, rule: AtRule<'i>) -> Result<String, SyntaxError<'i, Rejected>> {
        if self.reject_at.contains(&rule.name()) {
            return Err(SyntaxError::custom(rule.start(), Rejected));
        }
        let block = match rule.block() {
            None => String::new(),
            Some(block) if self.blocks_as_contents => {
                let items: Vec<String> = block
                    .block_contents(self)
                    .map(|unit| unit.unwrap_or_else(|failed| failure(&failed)))
                    .collect();
                format!(" {{{}}}", items.join("|"))
            }
            Some(block) => format!(" {{{}}}", show(&block)),
        };
        Ok(format!(
            "@{} [{}]{block} end={} at={}",
            rule.name(),
            show(&rule.prelude()),
            end_name(rule.rule_end()),
            span(rule.range()),
        ))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i>,
    ) -> Result<String, SyntaxError<'i, Rejected>> {
        if self.reject_qualified {
            return Err(SyntaxError::custom(rule.start(), Rejected));
        }
        let block = if self.blocks_as_contents {
            let items: Vec<String> = rule
                .block()
                .block_contents(self)
                .map(|unit| unit.unwrap_or_else(|failed| failure(&failed)))
                .collect();
            items.join("|")
        } else {
            show(&rule.block())
        };
        Ok(format!(
            "rule [{}] {{{block}}} at={}",
            show(&rule.prelude()),
            span(rule.range()),
        ))
    }
}

impl<'i> BlockGrammar<'i> for Dump {
    fn declaration(
        &mut self,
        declaration: Declaration<'i>,
    ) -> Result<String, SyntaxError<'i, Rejected>> {
        if self.reject_declarations.contains(&declaration.name()) {
            return Err(SyntaxError::custom(declaration.start(), Rejected));
        }
        let original = declaration
            .original_text()
            .map_or_else(String::new, |text| format!(" original={text:?}"));
        Ok(format!(
            "decl {} [{}]{}{original} end={} at={}",
            declaration.name(),
            show(&declaration.value()),
            if declaration.is_important() {
                " important"
            } else {
                ""
            },
            end_name(declaration.rule_end()),
            span(declaration.range()),
        ))
    }
}

/// A grammar that accepts everything and never reads a stream, so a refusal
/// of the meter can only be met by a driver. Its items are the kind of unit
/// and, for a rule, its name and how it ended.
pub struct Silent;

impl<'i> RuleGrammar<'i> for Silent {
    type Item = String;
    type Error = Rejected;

    fn at_rule(&mut self, rule: AtRule<'i>) -> Result<String, SyntaxError<'i, Rejected>> {
        Ok(format!(
            "@{} end={} block={}",
            rule.name(),
            end_name(rule.rule_end()),
            rule.block().is_some()
        ))
    }

    fn qualified_rule(
        &mut self,
        _: QualifiedRule<'i>,
    ) -> Result<String, SyntaxError<'i, Rejected>> {
        Ok("rule".to_owned())
    }
}

impl<'i> BlockGrammar<'i> for Silent {
    fn declaration(
        &mut self,
        declaration: Declaration<'i>,
    ) -> Result<String, SyntaxError<'i, Rejected>> {
        Ok(format!(
            "decl {} important={} end={}",
            declaration.name(),
            declaration.is_important(),
            end_name(declaration.rule_end())
        ))
    }
}

fn unit_name(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::AtRule => "at-rule",
        UnitKind::QualifiedRule => "qualified-rule",
        UnitKind::Declaration => "declaration",
        _ => "other",
    }
}

/// Describes a failed unit: its kind, why, and how it ended.
pub fn failure(failed: &Failed<'_, Rejected>) -> String {
    let error = failed.error();
    let why = if error.is_custom() {
        "grammar".to_owned()
    } else if error.is_end_of_stream() {
        "end-of-stream".to_owned()
    } else if let Some(token) = error.unexpected_token() {
        format!("unexpected {}", render(token))
    } else if error.is_denied_block() {
        "denied-block".to_owned()
    } else if error.is_refused() {
        "refused".to_owned()
    } else if error.is_invalid_qualified_rule() {
        "invalid-qualified-rule".to_owned()
    } else if error.is_unknown_at_rule() {
        "unknown-at-rule".to_owned()
    } else {
        "other".to_owned()
    };
    let end = failed.rule_end().map_or("none", end_name);
    let invalid = if failed.is_invalid_rule_error() {
        " invalid-rule-error"
    } else {
        ""
    };
    let declaration = failed
        .declaration_error()
        .map_or_else(String::new, |error| {
            let why = if error.is_custom() {
                "grammar".to_owned()
            } else if let Some(token) = error.unexpected_token() {
                format!("unexpected {}", render(token))
            } else if error.is_end_of_stream() {
                "end-of-stream".to_owned()
            } else {
                "other".to_owned()
            };
            format!(" declaration-error={why}")
        });
    format!(
        "!{} ({why}) end={end} at={}{invalid}{declaration}",
        unit_name(failed.kind()),
        span(failed.range()),
    )
}

/// The units of a stylesheet's contents, described.
pub fn sheet_with(text: &str, grammar: &mut Dump) -> Vec<String> {
    let document: Document = parse(text);
    document
        .root()
        .stylesheet_contents(grammar)
        .map(|unit| unit.unwrap_or_else(|failed| failure(&failed)))
        .collect()
}

pub fn sheet(text: &str) -> Vec<String> {
    sheet_with(text, &mut Dump::accepting())
}

/// The units of a block's contents, described.
pub fn block_with(text: &str, grammar: &mut Dump) -> Vec<String> {
    let document: Document = parse(text);
    document
        .root()
        .block_contents(grammar)
        .map(|unit| unit.unwrap_or_else(|failed| failure(&failed)))
        .collect()
}

pub fn block(text: &str) -> Vec<String> {
    block_with(text, &mut Dump::accepting())
}

/// Tokens that stand for themselves are shown with `render`; this is the
/// notation for one.
pub fn tok(token: Token<'_>) -> String {
    render(token)
}
