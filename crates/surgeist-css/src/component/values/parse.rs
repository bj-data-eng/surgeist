use crate::component::values::{
    ComponentData, CssComponentValueErrorKind, Lexeme, NumericToken, TokenData, ValueToken,
    serialize,
};
use crate::component::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueLimits,
    CssComponentValues, CssFunctionValue, CssHashFlag, CssNumericTokenKind, CssSimpleBlock,
    CssValueOrigin,
};
use crate::source::Provenance;
use crate::source::{CssParsedOrigin, CssSourceSnapshot};

#[cfg(test)]
pub(crate) mod spelling_allocation_tests;

pub(crate) mod bounded;

pub(super) fn parse(
    source: &str,
    limits: CssComponentValueLimits,
) -> Result<CssComponentValues, CssComponentValueError> {
    if source.len() > limits.max_css_bytes {
        return Err(CssComponentValueError::new(
            CssComponentValueErrorKind::ByteLimit,
            CssValueOrigin::UnretainedInput {
                byte_length: source.len(),
            },
        ));
    }
    let snapshot = CssSourceSnapshot::new(source);
    let document =
        match crate::syntax_consumption::source_document_snapshot_with_limits(&snapshot, limits) {
            Ok(document) => document,
            Err(error) => return Err(earlier_fault(&snapshot, error)),
        };
    let values = promote_nodes(&document, document.list(document.root_list()), limits)?;
    serialize::validate(&values, limits.max_css_bytes)?;
    Ok(values)
}

/// The components of a document of source text, as `collect` reads them from
/// a cursor over the same text, under the default limits.
pub(crate) fn promote_window(
    document: &crate::parser::syntax_types::Document<'_>,
) -> Result<CssComponentValues, CssComponentValueError> {
    promote_run(document, document.list(document.root_list()))
}

/// The components of the sibling `nodes` of a document of source text, under
/// the default limits.
pub(crate) fn promote_run(
    document: &crate::parser::syntax_types::Document<'_>,
    nodes: &[surgeist_css_parser::document::NodeId],
) -> Result<CssComponentValues, CssComponentValueError> {
    let limits = CssComponentValueLimits::default();
    let values = promote_nodes(document, nodes, limits)?;
    serialize::validate(&values, limits.max_css_bytes)?;
    Ok(values)
}

/// The tokens are consumed in order: a lexical fault before the token that
/// exhausted a limit is the failure, and the limit only a failure when
/// nothing before the token is faulty. The tree is built whole before its
/// tokens are promoted, so `limit` is the failure of a build that has seen the
/// whole text; this reads again only the text up to the end of the responsible
/// token (a bad string is bad because of the newline that ends it, which may be
/// that token), under no limit, for a fault that begins before the token.
fn earlier_fault(
    snapshot: &CssSourceSnapshot,
    limit: CssComponentValueError,
) -> CssComponentValueError {
    earlier_fault_scan(snapshot, limit).0
}

/// [`earlier_fault`], and how many nodes it read.
fn earlier_fault_scan(
    snapshot: &CssSourceSnapshot,
    limit: CssComponentValueError,
) -> (CssComponentValueError, usize) {
    let CssValueOrigin::Parsed(origin) = limit.origin() else {
        return (limit, 0);
    };
    let start = origin.span().start().byte_offset().value();
    let end = origin.span().end().byte_offset().value();
    let Ok(document) = crate::syntax_consumption::source_prefix_unlimited(snapshot, end) else {
        return (limit, 0);
    };
    let read = document.node_count();
    let fault = document.nodes().find(|node| {
        document.component(*node).fault().is_some()
            && document
                .range(*node)
                .is_some_and(|range| range.start.value() < start)
    });
    let failure = match fault {
        Some(node) => promote_nodes(&document, &[node], CssComponentValueLimits::default())
            .err()
            .unwrap_or(limit),
        None => limit,
    };
    (failure, read)
}

// Promote only an admitted, error-free generic candidate. Checked input values
// are reused directly; no serialization or tokenization is needed to
// reconstruct these lexical owners.
pub(crate) fn promote_nodes(
    document: &crate::parser::syntax_types::Document<'_>,
    nodes: &[surgeist_css_parser::document::NodeId],
    limits: CssComponentValueLimits,
) -> Result<CssComponentValues, CssComponentValueError> {
    use crate::parser::syntax_types::Attached;
    use crate::syntax_consumption::DocumentExt;
    use surgeist_css_parser::document::{BlockKind, GroupEnd, GroupKind, NodeId};
    enum Work {
        Node(NodeId),
        Finish(NodeId),
    }
    let mut work: Vec<_> = nodes.iter().rev().copied().map(Work::Node).collect();
    let mut frames = vec![Vec::new()];
    while let Some(item) = work.pop() {
        match item {
            Work::Node(id) => {
                if let Some(error) = document.denied_error(id) {
                    return Err(error);
                }
                if let Some(Attached::Component(component)) = document.payload(id) {
                    frames
                        .last_mut()
                        .expect("promotion frame")
                        .push((*component).clone());
                    continue;
                }
                if let Some(children) = document.children(id) {
                    // A group that would open past the depth limit is the failure,
                    // before the groups inside it are read.
                    if frames.len() > limits.max_depth as usize {
                        return Err(CssComponentValueError::new(
                            CssComponentValueErrorKind::NestingLimit,
                            document.origin(id),
                        ));
                    }
                    work.push(Work::Finish(id));
                    work.extend(
                        document
                            .list(children)
                            .iter()
                            .rev()
                            .copied()
                            .map(Work::Node),
                    );
                    frames.push(Vec::new());
                    continue;
                }
                let origin = document.origin(id);
                let CssValueOrigin::Parsed(parsed) = &origin else {
                    return Err(CssComponentValueError::new(
                        CssComponentValueErrorKind::InvalidToken,
                        origin,
                    ));
                };
                let spelling = Lexeme {
                    text: Provenance::new(document.spelling(id).expect("source lexeme").into()),
                    origin: origin.clone().into(),
                };
                let value = syntax_leaf(
                    parsed.source(),
                    document.token(id),
                    spelling,
                    parsed.clone(),
                    parsed.span().end().byte_offset().value(),
                )?;
                frames.last_mut().expect("promotion frame").push(value);
            }
            Work::Finish(id) => {
                let origin = document.origin(id);
                let CssValueOrigin::Parsed(parsed) = &origin else {
                    return Err(CssComponentValueError::new(
                        CssComponentValueErrorKind::InvalidToken,
                        origin,
                    ));
                };
                let (kind, name) = match document.group_kind(id).expect("group continuation") {
                    GroupKind::Function => {
                        let surgeist_css_parser::Token::Function(name) = document.token(id) else {
                            unreachable!("function opening");
                        };
                        (CssBlockKind::Parenthesis, Some(name.into()))
                    }
                    GroupKind::Block(BlockKind::Paren) => (CssBlockKind::Parenthesis, None),
                    GroupKind::Block(BlockKind::Square) => (CssBlockKind::SquareBracket, None),
                    GroupKind::Block(BlockKind::Curly) => (CssBlockKind::CurlyBracket, None),
                };
                let closing = match document.group_end(id).expect("group end") {
                    GroupEnd::Closed { .. } => {
                        let closing = document.closing_origin(id);
                        let CssValueOrigin::Parsed(parsed) = &closing else {
                            return Err(CssComponentValueError::new(
                                CssComponentValueErrorKind::InvalidToken,
                                closing,
                            ));
                        };
                        parsed.span().start().byte_offset().value()
                            ..parsed.span().end().byte_offset().value()
                    }
                    GroupEnd::Eof { at } => {
                        let at = at.expect("source group EOF").value();
                        at..at
                    }
                    _ => {
                        return Err(CssComponentValueError::new(
                            CssComponentValueErrorKind::InvalidToken,
                            origin.clone(),
                        ));
                    }
                };
                let frame = OpenFrame {
                    kind,
                    name,
                    opening: Lexeme {
                        text: Provenance::new(
                            document.spelling(id).expect("source opening").into(),
                        ),
                        origin: origin.clone().into(),
                    },
                    origin: parsed.clone(),
                    children: frames.pop().expect("child promotion frame"),
                };
                let value = frame.finish(parsed.source(), closing, limits, None)?;
                frames.last_mut().expect("promotion frame").push(value);
            }
        }
    }
    CssComponentValues::from_items(frames.pop().expect("root promotion frame"), limits)
}

/// The leaf of a token of the parser crate's token tree, as `leaf` makes one
/// of a token of the lexer.
fn syntax_leaf(
    source: &CssSourceSnapshot,
    token: surgeist_css_parser::Token<'_>,
    spelling: Lexeme,
    origin: CssParsedOrigin,
    end: usize,
) -> Result<CssComponentValue, CssComponentValueError> {
    use surgeist_css_parser::Token as Syntax;
    let invalid = match &token {
        Syntax::BadString => Some(CssComponentValueErrorKind::BadString),
        Syntax::BadUrl => Some(CssComponentValueErrorKind::BadUrl),
        Syntax::CloseParen | Syntax::CloseSquare | Syntax::CloseCurly => {
            Some(CssComponentValueErrorKind::UnmatchedClosingDelimiter)
        }
        _ => None,
    };
    if let Some(kind) = invalid {
        return Err(CssComponentValueError::new(
            kind,
            spelling.origin.inner().clone(),
        ));
    }
    let implicit_end = syntax_termination(&token, &spelling.text).map(|text| Lexeme {
        text: text.into_boxed_str().into(),
        origin: CssValueOrigin::ImplicitClosure {
            opening: origin.clone(),
            at: range_origin(source, end..end),
        }
        .into(),
    });
    let data = if token == Syntax::Comment {
        let text = &*spelling.text;
        let closed = text.len() >= 4 && text.ends_with("*/");
        let content = if closed {
            text.get(2..text.len() - 2)
        } else {
            text.get(2..)
        }
        .unwrap_or_default();
        ComponentData::Comment {
            content: content.into(),
            spelling,
            implicit_end: implicit_end.into(),
        }
    } else {
        let data = syntax_token_data(&token).ok_or_else(|| {
            CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                spelling.origin.inner().clone(),
            )
        })?;
        ComponentData::Token(ValueToken {
            data,
            spelling,
            implicit_end: implicit_end.into(),
        })
    };
    Ok(CssComponentValue {
        data,
        parsed: Some(origin).into(),
    })
}

fn syntax_numeric(number: surgeist_css_parser::token::Number<'_>) -> NumericToken {
    NumericToken {
        representation: number.representation().into(),
        kind: match number.kind() {
            surgeist_css_parser::token::NumberKind::Integer => CssNumericTokenKind::Integer,
            surgeist_css_parser::token::NumberKind::Number => CssNumericTokenKind::Number,
        },
        has_sign: number.sign().is_some(),
    }
}

fn syntax_token_data(token: &surgeist_css_parser::Token<'_>) -> Option<TokenData> {
    use surgeist_css_parser::Token as Syntax;
    use surgeist_css_parser::token::HashKind;
    Some(match *token {
        Syntax::Ident(value) => TokenData::Ident(value.into()),
        Syntax::AtKeyword(value) => TokenData::AtKeyword(value.into()),
        Syntax::Hash { value, kind } => TokenData::Hash {
            value: value.into(),
            flag: match kind {
                HashKind::Id => CssHashFlag::Id,
                HashKind::Unrestricted => CssHashFlag::Unrestricted,
            },
        },
        Syntax::String(value) => TokenData::String(value.into()),
        Syntax::Url(value) => TokenData::Url(value.into()),
        Syntax::Delim(value) => TokenData::Delim(value),
        Syntax::Number(number) => TokenData::Number(syntax_numeric(number)),
        Syntax::Percentage(number) => TokenData::Percentage(syntax_numeric(number)),
        Syntax::Dimension { number, unit } => TokenData::Dimension {
            number: syntax_numeric(number),
            unit: unit.into(),
        },
        Syntax::Whitespace => TokenData::Whitespace,
        Syntax::Colon => TokenData::Colon,
        Syntax::Semicolon => TokenData::Semicolon,
        Syntax::Comma => TokenData::Comma,
        Syntax::Cdo => TokenData::Cdo,
        Syntax::Cdc => TokenData::Cdc,
        Syntax::Comment
        | Syntax::Function(_)
        | Syntax::OpenParen
        | Syntax::OpenSquare
        | Syntax::OpenCurly
        | Syntax::BadUrl
        | Syntax::BadString
        | Syntax::CloseParen
        | Syntax::CloseSquare
        | Syntax::CloseCurly
        | Syntax::UnicodeRange(_) => return None,
    })
}

fn syntax_termination(token: &surgeist_css_parser::Token<'_>, text: &str) -> Option<String> {
    use surgeist_css_parser::Token as Syntax;
    match token {
        Syntax::String(_) => {
            let quote = text.as_bytes()[0];
            if has_unescaped_final(text, quote) {
                None
            } else {
                let mut suffix = String::new();
                // An escaped EOF contributes no code point inside a string.
                // An escaped newline preserves that effect before the new quote.
                if odd_trailing_backslashes(text) {
                    suffix.push('\n');
                }
                suffix.push(char::from(quote));
                Some(suffix)
            }
        }
        Syntax::Url(_) => {
            if has_unescaped_final(text, b')') {
                None
            } else {
                // Outside strings, an escaped EOF contributes U+FFFD.
                Some(
                    if odd_trailing_backslashes(text) {
                        "fffd )"
                    } else {
                        ")"
                    }
                    .to_owned(),
                )
            }
        }
        Syntax::Comment if text.len() < 4 || !text.ends_with("*/") => Some("*/".to_owned()),
        Syntax::Ident(_)
        | Syntax::AtKeyword(_)
        | Syntax::Hash { .. }
        | Syntax::Dimension { .. }
            if odd_trailing_backslashes(text) =>
        {
            Some("fffd ".to_owned())
        }
        _ => None,
    }
}

struct OpenFrame {
    kind: CssBlockKind,
    name: Option<Box<str>>,
    opening: Lexeme,
    origin: CssParsedOrigin,
    children: Vec<CssComponentValue>,
}

impl OpenFrame {
    fn finish(
        self,
        source: &CssSourceSnapshot,
        closing_range: std::ops::Range<usize>,
        limits: CssComponentValueLimits,
        control: Option<&crate::values::color_adjust::utf16_value::Control<'_>>,
    ) -> Result<CssComponentValue, CssComponentValueError> {
        let end = closing_range.end;
        let start = self.origin.span().start().byte_offset().value();
        let closing = if closing_range.is_empty() {
            Lexeme {
                text: copied(self.kind.delimiters().1, control, &self.origin)?.into(),
                origin: CssValueOrigin::ImplicitClosure {
                    opening: self.origin,
                    at: range_origin(source, closing_range),
                }
                .into(),
            }
        } else {
            Lexeme {
                text: copied(
                    &source.as_str()[closing_range.clone()],
                    control,
                    &self.origin,
                )?
                .into(),
                origin: CssValueOrigin::Parsed(range_origin(source, closing_range)).into(),
            }
        };
        let values = CssComponentValues::from_items_controlled(self.children, limits, control)?;
        let data = if let Some(name) = self.name {
            ComponentData::Function(CssFunctionValue {
                name,
                opening: self.opening,
                values,
                closing,
            })
        } else {
            ComponentData::Block(CssSimpleBlock {
                kind: self.kind,
                opening: self.opening,
                values,
                closing,
            })
        };
        Ok(CssComponentValue {
            data,
            parsed: Some(range_origin(source, start..end)).into(),
        })
    }
}

fn interrupted(origin: &CssParsedOrigin) -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::CapacityOverflow,
        CssValueOrigin::Parsed(origin.clone()),
    )
}

fn copied(
    value: &str,
    control: Option<&crate::values::color_adjust::utf16_value::Control<'_>>,
    origin: &CssParsedOrigin,
) -> Result<Box<str>, CssComponentValueError> {
    match control {
        Some(control) => control.boxed(value).map_err(|()| interrupted(origin)),
        None => Ok(value.into()),
    }
}

/// The component structural owner shared by provider and bounded lexical input.
/// Feeders supply lexical payloads; matching delimiters, nesting admission and
/// implicit group completion have exactly one implementation here.
struct ComponentStack<'a, 'c> {
    source: &'a CssSourceSnapshot,
    limits: CssComponentValueLimits,
    base_depth: u32,
    items: Vec<CssComponentValue>,
    frames: Vec<OpenFrame>,
    control: Option<&'a crate::values::color_adjust::utf16_value::Control<'c>>,
}
impl<'a, 'c> ComponentStack<'a, 'c> {
    fn new(
        source: &'a CssSourceSnapshot,
        limits: CssComponentValueLimits,
        base_depth: u32,
        control: Option<&'a crate::values::color_adjust::utf16_value::Control<'c>>,
    ) -> Self {
        Self {
            source,
            limits,
            base_depth,
            items: Vec::new(),
            frames: Vec::new(),
            control,
        }
    }
    fn close(
        &mut self,
        kind: Option<CssBlockKind>,
        range: std::ops::Range<usize>,
    ) -> Result<bool, CssComponentValueError> {
        if let Some(frame) = self.frames.pop_if(|frame| Some(frame.kind) == kind) {
            let value = frame.finish(self.source, range, self.limits, self.control)?;
            self.push(value)?;
            return Ok(true);
        }
        Ok(false)
    }
    fn open(
        &mut self,
        kind: CssBlockKind,
        name: Option<Box<str>>,
        range: std::ops::Range<usize>,
        count: &mut usize,
    ) -> Result<(), CssComponentValueError> {
        let origin = range_origin(self.source, range.clone());
        admit_component(count, self.limits, &origin)?;
        if self.frames.len() >= self.limits.max_depth.saturating_sub(self.base_depth) as usize {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::NestingLimit,
                CssValueOrigin::Parsed(origin),
            ));
        }
        let opening = Lexeme {
            text: copied(&self.source.as_str()[range], self.control, &origin)?.into(),
            origin: CssValueOrigin::Parsed(origin.clone()).into(),
        };
        if let Some(control) = self.control {
            control
                .reserve(&mut self.frames, 1)
                .map_err(|()| interrupted(&origin))?;
        }
        self.frames.push(OpenFrame {
            kind,
            name,
            opening,
            origin,
            children: Vec::new(),
        });
        Ok(())
    }
    fn push(&mut self, value: CssComponentValue) -> Result<(), CssComponentValueError> {
        let destination = self
            .frames
            .last_mut()
            .map_or(&mut self.items, |frame| &mut frame.children);
        if let Some(control) = self.control {
            let origin = value.parsed.as_ref().expect("parsed lexical input");
            control
                .reserve(destination, 1)
                .map_err(|()| interrupted(origin))?;
        }
        destination.push(value);
        Ok(())
    }
    fn finish(mut self, end: usize) -> Result<Vec<CssComponentValue>, CssComponentValueError> {
        while let Some(frame) = self.frames.pop() {
            let value = frame.finish(self.source, end..end, self.limits, self.control)?;
            self.push(value)?;
        }
        let backing = self.frames.capacity() * std::mem::size_of::<OpenFrame>();
        drop(self.frames);
        if let Some(control) = self.control {
            control.released(backing);
        }
        Ok(self.items)
    }
}

fn admit_component(
    count: &mut usize,
    limits: CssComponentValueLimits,
    origin: &CssParsedOrigin,
) -> Result<(), CssComponentValueError> {
    let next = count.checked_add(1).ok_or_else(|| {
        CssComponentValueError::new(
            CssComponentValueErrorKind::CapacityOverflow,
            CssValueOrigin::Parsed(origin.clone()),
        )
    })?;
    if next > limits.max_components {
        return Err(CssComponentValueError::new(
            CssComponentValueErrorKind::ComponentLimit,
            CssValueOrigin::Parsed(origin.clone()),
        ));
    }
    *count = next;
    Ok(())
}

fn range_origin(source: &CssSourceSnapshot, range: std::ops::Range<usize>) -> CssParsedOrigin {
    CssParsedOrigin::from_range(source, range)
        .expect("tokenizer positions are ordered UTF-8 boundaries in the original source")
}

pub(crate) fn odd_trailing_backslashes(text: &str) -> bool {
    text.bytes().rev().take_while(|byte| *byte == b'\\').count() % 2 == 1
}

pub(crate) fn has_unescaped_final(text: &str, closing: u8) -> bool {
    text.len() > 1
        && text.as_bytes().last() == Some(&closing)
        && !odd_trailing_backslashes(&text[..text.len() - 1])
}

pub(super) fn programmatic_token(
    spelling: &str,
    invalid_kind: CssComponentValueErrorKind,
) -> Result<CssComponentValue, CssComponentValueError> {
    let mut values = parse(spelling, CssComponentValueLimits::default())
        .map_err(|_| CssComponentValueError::programmatic(invalid_kind))?;
    if values.items.len() != 1 {
        return Err(CssComponentValueError::programmatic(invalid_kind));
    }
    let ComponentData::Token(token) = &mut values.items[0].data else {
        return Err(CssComponentValueError::programmatic(invalid_kind));
    };
    if token.implicit_end.is_some() {
        return Err(CssComponentValueError::programmatic(invalid_kind));
    }
    token.spelling.origin = CssValueOrigin::Programmatic.into();
    values.items[0].parsed = Provenance::new(None);
    let [value] =
        <[_; 1]>::try_from(values.items.into_vec()).expect("exactly one token was checked");
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fault_before_the_token_that_exhausts_a_limit_is_the_failure_and_one_after_it_is_not() {
        let limits = CssComponentValueLimits::try_new(256, 2, 100).unwrap();
        // The bad string is the first token; the limit runs out at the third.
        let error = parse("\"x\n a b c", limits).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::BadString);
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("the bad string retains its source origin");
        };
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), 2);
        // The limit runs out at the third token, before the bad string.
        let error = parse("a b \"x\n", limits).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::ComponentLimit);
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("the third token retains its source origin");
        };
        assert_eq!(origin.span().start().byte_offset().value(), 2);
    }

    /// An opening is a component to admit before it is a level to enter. With
    /// room for no component the count refuses it, and with room for it but not
    /// for its level the depth does; either way the refusal names the opening.
    #[test]
    fn component_admission_precedes_the_depth_check_of_the_same_opening() {
        let source = "f(g(x)) tail";
        for (depth, count, kind, start) in [
            (0, 0, CssComponentValueErrorKind::ComponentLimit, 0),
            (0, 1, CssComponentValueErrorKind::NestingLimit, 0),
            (1, 1, CssComponentValueErrorKind::ComponentLimit, 2),
            (1, 2, CssComponentValueErrorKind::NestingLimit, 2),
        ] {
            let snapshot = CssSourceSnapshot::new(source);
            let limits = CssComponentValueLimits::try_new(depth, count, 100).unwrap();
            let error =
                crate::syntax_consumption::source_document_snapshot_with_limits(&snapshot, limits)
                    .expect_err("the limits refuse the source");
            assert_eq!(error.kind(), kind, "depth {depth} count {count}");
            let CssValueOrigin::Parsed(origin) = error.origin() else {
                panic!("the rejected opening retains its source origin");
            };
            assert_eq!(origin.span().start().byte_offset().value(), start);
            assert_eq!(origin.span().end().byte_offset().value(), start + 2);
        }
    }

    /// The second look is bounded by what the limited build admitted: a huge
    /// text with a tiny limit reads a handful of nodes, never the text.
    #[test]
    fn the_second_look_at_a_limit_failure_reads_only_what_the_build_admitted() {
        let limits = CssComponentValueLimits::try_new(256, 4, 100_000_000).unwrap();
        for text in [
            "a ".repeat(1_000_000),
            // A bad string needs its newline: the fault is found before the
            // limit token even though the newline ends it.
            format!("\"x\n{}", "a ".repeat(1_000_000)),
        ] {
            let snapshot = CssSourceSnapshot::new(&text);
            let document =
                crate::syntax_consumption::source_document_snapshot_with_limits(&snapshot, limits);
            let limit = document.expect_err("the fifth component is refused");
            assert_eq!(limit.kind(), CssComponentValueErrorKind::ComponentLimit);
            let (failure, read) = earlier_fault_scan(&snapshot, limit);
            assert!(read <= 16, "{read} nodes read for a tiny limit");
            if text.starts_with('"') {
                assert_eq!(failure.kind(), CssComponentValueErrorKind::BadString);
            } else {
                assert_eq!(failure.kind(), CssComponentValueErrorKind::ComponentLimit);
            }
        }
        // Depth: the nodes read are the levels admitted, not the input.
        let deep = "(".repeat(2_000_000);
        let snapshot = CssSourceSnapshot::new(&deep);
        let limit = crate::syntax_consumption::source_document_snapshot_with_limits(
            &snapshot,
            CssComponentValueLimits::default(),
        )
        .expect_err("nesting past the ceiling is refused");
        assert_eq!(limit.kind(), CssComponentValueErrorKind::NestingLimit);
        let (_, read) = earlier_fault_scan(&snapshot, limit);
        assert!(read <= 300, "{read} nodes read for the depth ceiling");
    }
}
