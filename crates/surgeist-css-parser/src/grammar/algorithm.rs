//! The consume algorithms of section 5.5: an at-rule, a qualified rule and a
//! declaration.
//!
//! A document holds a function or a simple block as one node that owns a list
//! of its own, so "consume a component value" is one step past a node and
//! "consume a block" is one step past the node that is the block. Each
//! algorithm here therefore finds the extent of a unit by walking the nodes of
//! one list, and hands the grammar streams over the parts: the prelude is the
//! nodes before the terminator, the block is the node's own list. What the
//! specification builds as lists of values is never copied.
//!
//! Every node a walk reaches for the first time is one meter step
//! ([`Stream::peek_at`]); looking again at a node already reached (to trim
//! whitespace, to find the last two values of a declaration) takes none. A
//! refusal ends the walk with the stream error, and the callers report it; no
//! algorithm treats a refusal as the end of the stream.

use super::{AtRule, Declaration, Failed, QualifiedRule, RuleEnd, RuleGrammar, Unit, UnitKind};
use crate::document::{BlockKind, Component, ListId};
use crate::stream::StreamError;
use crate::{Site, Stream, SyntaxError, Token};

/// What a component is, as far as the algorithms care.
pub(super) enum Class<'i> {
    Whitespace,
    Comment,
    Semicolon,
    CloseCurly,
    /// A `{}`-block, with its list unless the build denied it.
    Curly(Option<ListId>),
    AtKeyword(&'i str),
    Cdo,
    Cdc,
    Ident(&'i str),
    Colon,
    Other,
}

pub(super) fn classify<'i, P>(component: &Component<'i, P>) -> Class<'i> {
    match component {
        Component::Token { token, .. } => match token {
            Token::Whitespace => Class::Whitespace,
            Token::Comment => Class::Comment,
            Token::Semicolon => Class::Semicolon,
            Token::CloseCurly => Class::CloseCurly,
            Token::AtKeyword(name) => Class::AtKeyword(name),
            Token::Cdo => Class::Cdo,
            Token::Cdc => Class::Cdc,
            Token::Ident(value) => Class::Ident(value),
            Token::Colon => Class::Colon,
            _ => Class::Other,
        },
        Component::Block {
            kind: BlockKind::Curly,
            children,
            ..
        } => Class::Curly(Some(*children)),
        Component::Denied { .. } => Class::Curly(None),
        Component::Function { .. } | Component::Block { .. } => Class::Other,
    }
}

/// The token a component starts with: for a group, its opening token.
pub(super) fn opening_token<'i, P>(component: &Component<'i, P>) -> Token<'i> {
    match component {
        Component::Token { token, .. } => *token,
        Component::Function { name, .. } => Token::Function(name),
        Component::Block { kind, .. } => match kind {
            BlockKind::Paren => Token::OpenParen,
            BlockKind::Square => Token::OpenSquare,
            BlockKind::Curly => Token::OpenCurly,
        },
        Component::Denied { .. } => Token::OpenCurly,
    }
}

fn is_trivia<P>(component: &Component<'_, P>) -> bool {
    matches!(
        component,
        Component::Token {
            token: Token::Whitespace | Token::Comment,
            ..
        }
    )
}

/// The result of a qualified rule attempt (section 5.5.3).
pub(super) enum Qualified<'i, T, E, P> {
    /// The rule reached its block. `result` is the grammar's verdict, or the
    /// denial of the block.
    Rule {
        result: Result<T, SyntaxError<'i, E, P>>,
        start: Site<'i, P>,
        end: Site<'i, P>,
    },
    /// The attempt returned nothing: the stream, a stop token or a `}` came
    /// before a block.
    Nothing {
        reason: StreamError<'i, P>,
        start: Site<'i, P>,
        end: Site<'i, P>,
        rule_end: RuleEnd,
    },
    /// The attempt returned nothing because the rule looked like a custom
    /// property, and its remnants were consumed.
    Quiet {
        start: Site<'i, P>,
        end: Site<'i, P>,
        rule_end: RuleEnd,
    },
}

/// Whether the first two component values of `from..to` that are not
/// whitespace are an ident that starts with two hyphen-minuses and a colon
/// (the test of section 5.5.3 for a rule that looks like a custom property).
fn looks_like_custom_property<P>(stream: &Stream<'_, P>, from: usize, to: usize) -> bool {
    let mut significant = (from..to)
        .filter_map(|index| stream.component_unmetered(index))
        .filter(|component| !is_trivia(component));
    let first = significant.next().map(|component| classify(&component));
    let second = significant.next().map(|component| classify(&component));
    matches!(first, Some(Class::Ident(value)) if value.starts_with("--"))
        && matches!(second, Some(Class::Colon))
}

/// Consume the remnants of a bad declaration (section 5.5.6) from `from`, in a
/// nested context: through the next semicolon, or up to a `}` or the end.
/// Returns the index to continue from.
fn consume_remnants<'i, P>(
    stream: &Stream<'i, P>,
    from: usize,
) -> Result<(usize, RuleEnd), StreamError<'i, P>> {
    let mut index = from;
    loop {
        let Some(component) = stream.peek_at(index)? else {
            return Ok((index, RuleEnd::Eof));
        };
        match classify(&component) {
            Class::Semicolon => return Ok((index + 1, RuleEnd::Semicolon)),
            Class::CloseCurly => return Ok((index, RuleEnd::CloseBrace)),
            _ => index += 1,
        }
    }
}

/// Consumes a qualified rule (section 5.5.3) from the next node, which the
/// caller has positioned the stream at and already examined: `first` is that
/// node, or `None` if the stream is at its end. `stop_semicolon` is the
/// optional stop token, `nested` the bool of the same name.
///
/// On success the stream is past what the attempt consumed: past the block, or
/// at the stop token, `}` or end that ended it.
pub(super) fn consume_qualified_rule<'i, P, G>(
    stream: &mut Stream<'i, P>,
    first: Option<Component<'i, P>>,
    stop_semicolon: bool,
    nested: bool,
    grammar: &mut G,
) -> Result<Qualified<'i, G::Item, G::Error, P>, StreamError<'i, P>>
where
    G: RuleGrammar<'i, P>,
{
    let start_index = stream.next_index();
    let start = stream.site_at(start_index);
    let mut index = start_index;
    let mut component = first;
    loop {
        let class = match &component {
            None => {
                stream.seek(index);
                let at = stream.site_at(index);
                return Ok(Qualified::Nothing {
                    reason: StreamError::end_of_stream(at),
                    start,
                    end: at,
                    rule_end: RuleEnd::Eof,
                });
            }
            Some(component) => classify(component),
        };
        match class {
            Class::Semicolon if stop_semicolon => {
                stream.seek(index);
                let at = stream.site_at(index);
                return Ok(Qualified::Nothing {
                    reason: StreamError::unexpected(at, Token::Semicolon),
                    start,
                    end: at,
                    rule_end: RuleEnd::Semicolon,
                });
            }
            Class::CloseCurly if nested => {
                stream.seek(index);
                let at = stream.site_at(index);
                return Ok(Qualified::Nothing {
                    reason: StreamError::unexpected(at, Token::CloseCurly),
                    start,
                    end: at,
                    rule_end: RuleEnd::CloseBrace,
                });
            }
            Class::Curly(children) => {
                let block_at = stream.site_at(index);
                if looks_like_custom_property(stream, start_index, index) {
                    let (after, rule_end) = if nested {
                        consume_remnants(stream, index + 1)?
                    } else {
                        (index + 1, RuleEnd::Block)
                    };
                    stream.seek(after);
                    return Ok(Qualified::Quiet {
                        start,
                        end: stream.site_at(after),
                        rule_end,
                    });
                }
                stream.seek(index + 1);
                let end = stream.site_at(index + 1);
                let result = match children {
                    None => Err(SyntaxError::denied_block_error(block_at)),
                    Some(children) => grammar.qualified_rule(QualifiedRule {
                        prelude: stream.span(start_index, index),
                        block: stream.document().stream(children),
                        start,
                        end,
                    }),
                };
                return Ok(Qualified::Rule { result, start, end });
            }
            // Anything else, a `}` included when the rule is not nested, is a
            // component value of the prelude.
            _ => {
                index += 1;
                component = stream.peek_at(index)?;
            }
        }
    }
}

/// The result of an at-rule attempt (section 5.5.2).
pub(super) struct AtRuleOutcome<'i, T, E, P> {
    /// The grammar's item, or why the rule was not accepted.
    pub(super) unit: Unit<'i, T, E, P>,
    /// The boundary after what the attempt consumed.
    pub(super) end: Site<'i, P>,
    /// What ended the rule.
    pub(super) rule_end: RuleEnd,
}

/// Consumes an at-rule (section 5.5.2) whose at-keyword `name` is the next
/// node.
///
/// On success the stream is past what the rule consumed.
pub(super) fn consume_at_rule<'i, P, G>(
    stream: &mut Stream<'i, P>,
    name: &'i str,
    nested: bool,
    grammar: &mut G,
) -> Result<AtRuleOutcome<'i, G::Item, G::Error, P>, StreamError<'i, P>>
where
    G: RuleGrammar<'i, P>,
{
    let start_index = stream.next_index();
    let start = stream.site_at(start_index);
    let mut index = start_index + 1;
    let mut component = stream.peek_at(index)?;
    let (prelude_end, after, block, rule_end) = loop {
        let Some(current) = &component else {
            break (index, index, None, RuleEnd::Eof);
        };
        match classify(current) {
            Class::Semicolon => break (index, index + 1, None, RuleEnd::Semicolon),
            Class::CloseCurly if nested => break (index, index, None, RuleEnd::CloseBrace),
            Class::Curly(children) => break (index, index + 1, Some(children), RuleEnd::Block),
            _ => {
                index += 1;
                component = stream.peek_at(index)?;
            }
        }
    };
    stream.seek(after);
    let end = stream.site_at(after);
    let block = match block {
        None => None,
        Some(Some(children)) => Some(stream.document().stream(children)),
        Some(None) => {
            let denied = SyntaxError::denied_block_error(stream.site_at(prelude_end));
            let unit = Err(Failed::new(
                UnitKind::AtRule,
                denied,
                start,
                end,
                Some(rule_end),
            ));
            return Ok(AtRuleOutcome {
                unit,
                end,
                rule_end,
            });
        }
    };
    let rule = AtRule {
        name,
        prelude: stream.span(start_index + 1, prelude_end),
        block,
        start,
        end,
        rule_end,
    };
    let unit = grammar
        .at_rule(rule)
        .map_err(|error| Failed::new(UnitKind::AtRule, error, start, end, Some(rule_end)));
    Ok(AtRuleOutcome {
        unit,
        end,
        rule_end,
    })
}

/// Consumes a declaration (section 5.5.6) from the next node, which the caller
/// has positioned the stream at and already examined: `first` is that node, or
/// `None` if the stream is at its end.
///
/// On success the stream is at the stop token, `}` or end that ended the value;
/// the stop token is not consumed. On failure the stream is wherever the walk
/// stopped, which the callers do not use: a block restores its mark and reads the
/// text again, and "parse a declaration" returns a syntax error. The failure is
/// the stream error that says why, and one for which `is_refused()` holds is a
/// refusal of the meter.
pub(super) fn consume_declaration<'i, P>(
    stream: &mut Stream<'i, P>,
    first: Option<Component<'i, P>>,
    nested: bool,
) -> Result<Declaration<'i, P>, StreamError<'i, P>> {
    let start_index = stream.next_index();
    let start = stream.site_at(start_index);

    // Step 1: an ident is the name.
    let name = match first {
        Some(Component::Token {
            token: Token::Ident(name),
            ..
        }) => name,
        Some(other) => {
            return Err(StreamError::unexpected(start, opening_token(&other)));
        }
        None => return Err(StreamError::end_of_stream(start)),
    };

    // Steps 2 and 3: whitespace, then a colon.
    let mut index = start_index + 1;
    let colon = loop {
        let at = stream.site_at(index);
        match stream.peek_at(index)? {
            Some(component) if is_trivia(&component) => index += 1,
            Some(Component::Token {
                token: Token::Colon,
                ..
            }) => break index,
            Some(other) => return Err(StreamError::unexpected(at, opening_token(&other))),
            None => return Err(StreamError::end_of_stream(at)),
        }
    };

    // Step 4: whitespace. Step 5: the value runs to the stop token (a
    // semicolon), the end of the stream, or, in a nested context, a `}`.
    index = colon + 1;
    let mut component = stream.peek_at(index)?;
    while component.as_ref().is_some_and(is_trivia) {
        index += 1;
        component = stream.peek_at(index)?;
    }
    let value_start = index;
    let rule_end = loop {
        let Some(current) = &component else {
            break RuleEnd::Eof;
        };
        match classify(current) {
            Class::Semicolon => break RuleEnd::Semicolon,
            Class::CloseCurly if nested => break RuleEnd::CloseBrace,
            _ => {
                index += 1;
                component = stream.peek_at(index)?;
            }
        }
    };
    let list_end = index;

    // Step 6: `!` and `important` as the last two values that are not
    // whitespace.
    let mut significant = (value_start..list_end)
        .rev()
        .filter_map(|at| {
            stream
                .component_unmetered(at)
                .map(|component| (at, component))
        })
        .filter(|(_, component)| !is_trivia(component));
    let last = significant.next();
    let before_last = significant.next();
    let mut value_end = list_end;
    let mut important = false;
    if let (
        Some((
            _,
            Component::Token {
                token: Token::Ident(word),
                ..
            },
        )),
        Some((
            bang,
            Component::Token {
                token: Token::Delim('!'),
                ..
            },
        )),
    ) = (last, before_last)
        && word.eq_ignore_ascii_case("important")
    {
        important = true;
        value_end = bang;
    }
    // Step 7: no whitespace at the end of the value.
    while value_end > value_start
        && stream
            .component_unmetered(value_end - 1)
            .is_some_and(|component| is_trivia(&component))
    {
        value_end -= 1;
    }

    // Step 8.
    let value_from = stream.site_at(value_start);
    let value_to = stream.site_at(value_end);
    let original_text = if name.starts_with("--") {
        // A custom property keeps the original text of its value.
        stream.text_between(value_from, value_to)
    } else {
        // A {}-block is only allowed as the entire value of a non-custom
        // property.
        let mut curly = None;
        let mut values = 0_usize;
        for at in value_start..value_end {
            let Some(component) = stream.component_unmetered(at) else {
                continue;
            };
            if is_trivia(&component) {
                continue;
            }
            values += 1;
            if curly.is_none() && matches!(classify(&component), Class::Curly(_)) {
                curly = Some(at);
            }
        }
        if let Some(at) = curly
            && values > 1
        {
            return Err(StreamError::unexpected(
                stream.site_at(at),
                Token::OpenCurly,
            ));
        }
        if name.eq_ignore_ascii_case("unicode-range") {
            stream.text_between(value_from, value_to)
        } else {
            None
        }
    };

    stream.seek(list_end);
    Ok(Declaration {
        name,
        value: stream.span(value_start, value_end),
        important,
        original_text,
        start,
        end: stream.site_at(list_end),
        rule_end,
    })
}
