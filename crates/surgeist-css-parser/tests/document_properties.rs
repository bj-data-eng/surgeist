//! Properties of the document and the stream over generated text (P5).
//!
//! The case generators and the codec belong to this crate (`tests/common`);
//! `surgeist-fuzz` supplies the bounded, seeded runner. Every run uses a fixed
//! seed and a fixed number of cases, so a failure reproduces.
//!
//! - Tree: the document of a text is the tree that sections 5.5.7 to 5.5.10
//!   give. The oracle here is a small recursive reading of those sections over
//!   the lexer's tokens, written for this file, not the iterative builder under
//!   test. Walking the document reproduces the lexer's tokens and intervals;
//!   every group has exactly one end, a closing token or the end of the text;
//!   `maximum_depth`, the component count and the byte count equal a recount;
//!   navigation (`location`, `children`, `parent_group`, `node_at_offset`)
//!   agrees with the lists.
//! - Stream: `mark` and `restore` are idempotent, and entering a group and
//!   exhausting it leaves the stream where skipping it does.
//! - Depth: a text builds under a limit exactly when its measured depth does not
//!   exceed it, and the `DenyBlock` policy denies only `{}`-blocks.
//! - Totality: no text makes `Document::parse` panic.

mod common;

use std::panic::{AssertUnwindSafe, catch_unwind};

use common::generate::generate;
use common::render;
use common::tree::{outline, span};
use surgeist_css_parser::document::{Component, GroupEnd, GroupKind, ListId};
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::limits::{DepthExcess, Limits};
use surgeist_css_parser::{Document, Offset, Stream, SyntaxError, Token};
use surgeist_fuzz::engine::{PropertyConfig, run_property};
use surgeist_fuzz::generation::property::bytes;
use surgeist_fuzz::{
    BuildRequirements, CaseFormat, CaseOutcome, CaseTarget, FailureClass, FailureKey, FuzzError,
    FuzzErrorKind, Generation, GeneratorIdentity, InputEncoding, InputLimits, PropertyId, Result,
    TargetDescriptor, TargetDescriptorSpec, TargetId,
};

const CASES: u32 = 600;
const TEXT_BYTES: usize = 512;
const ENTROPY_BYTES: usize = 97;
const SEED: u64 = 0x1508_0006;

// ----------------------------------------------------------------- generators

/// A text of arbitrary ASCII bytes and a few wide code points.
fn raw_text(entropy: &[u8]) -> String {
    const WIDE: [char; 4] = ['\u{e9}', '\u{2603}', '\u{1f600}', '\u{fffd}'];
    entropy
        .iter()
        .take(96)
        .map(|&byte| {
            if byte < 0x80 {
                char::from(byte)
            } else {
                WIDE[usize::from(byte) % WIDE.len()]
            }
        })
        .collect()
}

/// A text heavy in groups: a few kinds of opening and closing tokens, names and
/// separators, so nesting, mismatches and unclosed groups are common.
fn bracket_text(entropy: &[u8]) -> String {
    const PIECES: [&str; 14] = [
        "(", ")", "[", "]", "{", "}", "f(", "a", " ", ";", ",", "!", "/**/", "url(x)",
    ];
    entropy
        .iter()
        .take(64)
        .map(|&byte| PIECES[usize::from(byte) % PIECES.len()])
        .collect()
}

/// The first entropy byte picks the family: authored pieces, raw bytes, or
/// brackets.
fn text_of(entropy: &[u8]) -> String {
    let rest = entropy.get(1..).unwrap_or(&[]);
    match entropy.first().map(|first| first % 3) {
        Some(1) => raw_text(rest),
        Some(2) => bracket_text(rest),
        _ => generate(rest),
    }
}

// ----------------------------------------------------------------- the oracle

/// A node of the oracle's tree: the sections 5.5.7 to 5.5.10 reading.
enum Reference {
    Leaf(String),
    Group {
        opening: String,
        children: Vec<Reference>,
        closed: bool,
    },
}

/// What the oracle needs to know of a token.
#[derive(Clone)]
struct Lexed {
    rendered: String,
    opens: Option<Closer>,
    closes: Option<Closer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Closer {
    Paren,
    Square,
    Curly,
}

fn lex(text: &str) -> (Vec<Lexed>, Vec<std::ops::Range<usize>>) {
    let mut lexer = Lexer::new(text, &Options::default());
    let (mut tokens, mut ranges) = (Vec::new(), Vec::new());
    while let Some(lexeme) = lexer.next_lexeme() {
        let token = lexeme.token();
        let opens = match token {
            Token::Function(_) | Token::OpenParen => Some(Closer::Paren),
            Token::OpenSquare => Some(Closer::Square),
            Token::OpenCurly => Some(Closer::Curly),
            _ => None,
        };
        let closes = match token {
            Token::CloseParen => Some(Closer::Paren),
            Token::CloseSquare => Some(Closer::Square),
            Token::CloseCurly => Some(Closer::Curly),
            _ => None,
        };
        tokens.push(Lexed {
            rendered: render(token),
            opens,
            closes,
        });
        ranges.push(span(lexeme.range()));
    }
    (tokens, ranges)
}

/// Consume a list of component values, ended by `ending` (5.5.7, 5.5.9, 5.5.10):
/// the end of the tokens ends any list, the matching closing token ends a
/// group's, and any other token, a closing token that does not match
/// included, is a component value (5.5.8).
fn consume_list(
    tokens: &[Lexed],
    at: &mut usize,
    ending: Option<Closer>,
) -> (Vec<Reference>, bool) {
    let mut list = Vec::new();
    while let Some(token) = tokens.get(*at) {
        if ending.is_some() && token.closes == ending {
            *at += 1;
            return (list, true);
        }
        *at += 1;
        if let Some(closer) = token.opens {
            let (children, closed) = consume_list(tokens, at, Some(closer));
            list.push(Reference::Group {
                opening: token.rendered.clone(),
                children,
                closed,
            });
        } else {
            list.push(Reference::Leaf(token.rendered.clone()));
        }
    }
    (list, false)
}

fn reference_outline(list: &[Reference]) -> String {
    list.iter()
        .map(|node| match node {
            Reference::Leaf(rendered) => rendered.clone(),
            Reference::Group {
                opening,
                children,
                closed,
            } => format!(
                "{opening}<{}{}>",
                reference_outline(children),
                if *closed { "" } else { "…" }
            ),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn reference_depth(list: &[Reference]) -> usize {
    list.iter()
        .map(|node| match node {
            Reference::Leaf(_) => 0,
            Reference::Group { children, .. } => 1 + reference_depth(children),
        })
        .max()
        .unwrap_or(0)
}

// ----------------------------------------------------------------- the checks

fn parse(text: &str) -> std::result::Result<Document<'_>, String> {
    Document::parse(
        text,
        &Options::default(),
        Limits::new(u32::MAX, usize::MAX, usize::MAX),
        DepthExcess::Fail,
    )
    .map_err(|error| format!("{text:?}: the build failed: {error}"))
}

/// Walks the document in order, an opening token, its contents, its closing
/// token, collecting each token's rendering and interval.
fn walk(
    document: &Document<'_>,
    list: ListId,
    out: &mut Vec<(String, std::ops::Range<usize>)>,
) -> std::result::Result<(), String> {
    for &node in document.list(list) {
        let opening = document
            .spelling(node)
            .map(str::len)
            .ok_or("a node of a text has a spelling")?;
        let range = span(document.range(node).ok_or("a node of a text has a range")?);
        out.push((
            render(document.token(node)),
            range.start..range.start + opening,
        ));
        if let Some(children) = document.children(node) {
            walk(document, children, out)?;
            match document.group_end(node) {
                Some(GroupEnd::Closed {
                    range: Some(closer),
                    ..
                }) => {
                    let kind = document.group_kind(node).ok_or("a group has a kind")?;
                    let token = match kind {
                        GroupKind::Block(surgeist_css_parser::document::BlockKind::Square) => {
                            Token::CloseSquare
                        }
                        GroupKind::Block(surgeist_css_parser::document::BlockKind::Curly) => {
                            Token::CloseCurly
                        }
                        _ => Token::CloseParen,
                    };
                    out.push((render(token), span(closer.clone())));
                }
                Some(GroupEnd::Eof { .. }) => {}
                _ => return Err("a group has one end".into()),
            }
        }
    }
    Ok(())
}

fn max_depth_of(document: &Document<'_>, list: ListId) -> usize {
    document
        .list(list)
        .iter()
        .map(|&node| {
            document
                .children(node)
                .map_or(0, |c| 1 + max_depth_of(document, c))
        })
        .max()
        .unwrap_or(0)
}

fn all_lists(document: &Document<'_>) -> Vec<ListId> {
    let mut lists = vec![document.root_list()];
    let mut next = 0;
    while next < lists.len() {
        for &node in document.list(lists[next]) {
            if let Some(children) = document.children(node) {
                lists.push(children);
            }
        }
        next += 1;
    }
    lists
}

/// Tree property.
fn check_tree(text: &str) -> std::result::Result<(), String> {
    let document = parse(text)?;
    let (tokens, ranges) = lex(text);
    // The oracle's tree.
    let mut at = 0;
    let (reference, _) = consume_list(&tokens, &mut at, None);
    let expected = reference_outline(&reference);
    let actual = outline(&document, document.root_list());
    if actual != expected {
        return Err(format!(
            "{text:?}: the tree is {actual}, the sections give {expected}"
        ));
    }
    // Walking reproduces the lexer's tokens and their intervals.
    let mut walked = Vec::new();
    walk(&document, document.root_list(), &mut walked)?;
    let lexed: Vec<_> = tokens
        .iter()
        .map(|token| token.rendered.clone())
        .zip(ranges.iter().cloned())
        .collect();
    if walked != lexed {
        return Err(format!("{text:?}: walked {walked:?}, lexed {lexed:?}"));
    }
    // The measures equal a recount.
    let depth = reference_depth(&reference);
    if usize::try_from(document.maximum_depth()).ok() != Some(depth)
        || max_depth_of(&document, document.root_list()) != depth
    {
        return Err(format!(
            "{text:?}: maximum_depth {} is not {depth}",
            document.maximum_depth()
        ));
    }
    if document.metrics().components() != document.node_count() {
        return Err(format!("{text:?}: components and nodes differ"));
    }
    if document.metrics().spelling_bytes() != text.len() {
        return Err(format!("{text:?}: spelling bytes differ from the text"));
    }
    // Every group ends exactly once, in the text.
    for list in all_lists(&document) {
        for (index, &node) in document.list(list).iter().enumerate() {
            if document.location(node) != (list, index) {
                return Err(format!("{text:?}: location of {node:?}"));
            }
            match (document.children(node), document.group_end(node)) {
                (None, None) => {}
                (Some(children), Some(end)) => {
                    if document.parent_group(children) != Some(node) {
                        return Err(format!("{text:?}: parent_group of {children:?}"));
                    }
                    let list_end = document.list_end(children).ok_or("a list has an end")?;
                    match end {
                        GroupEnd::Closed {
                            range: Some(closer),
                            ..
                        } => {
                            if closer.start != list_end || closer.end.value() > text.len() {
                                return Err(format!("{text:?}: closer of {node:?}"));
                            }
                        }
                        GroupEnd::Eof { at: Some(end) } => {
                            if end.value() != text.len() || list_end != *end {
                                return Err(format!("{text:?}: eof of {node:?}"));
                            }
                        }
                        _ => return Err(format!("{text:?}: an end without a place")),
                    }
                }
                _ => {
                    return Err(format!(
                        "{text:?}: a node with children or an end, not both"
                    ));
                }
            }
        }
    }
    if document.parent_group(document.root_list()).is_some() {
        return Err("the root list has a parent".into());
    }
    // node_at_offset finds every token and closing token, and only those.
    let mut starts = std::collections::BTreeSet::new();
    for (_, range) in &lexed {
        starts.insert(range.start);
    }
    for offset in 0..=text.len() {
        let found = document.node_at_offset(Offset::new(offset));
        if found.is_some() != starts.contains(&offset) {
            return Err(format!("{text:?}: node_at_offset({offset}) is {found:?}"));
        }
        if let Some(site) = found
            && site.offset() != Some(Offset::new(offset))
        {
            return Err(format!(
                "{text:?}: the site at {offset} reports {:?}",
                site.offset()
            ));
        }
    }
    // Component reports its node and kind consistently.
    for list in all_lists(&document) {
        for &node in document.list(list) {
            let component = document.component(node);
            let consistent = match &component {
                Component::Function { .. } => {
                    document.group_kind(node) == Some(GroupKind::Function)
                }
                Component::Block { kind, .. } => {
                    document.group_kind(node) == Some(GroupKind::Block(*kind))
                }
                Component::Token { .. } => document.children(node).is_none(),
                _ => false,
            };
            if component.node() != node || !consistent {
                return Err(format!("{text:?}: component of {node:?}"));
            }
        }
    }
    Ok(())
}

/// Drains `stream` with `consume_token_with_comments`, entering every group when
/// `enter`, and returns the number of tokens consumed in all lists.
fn drain(stream: &mut Stream<'_>, enter: bool) -> usize {
    let mut count = 0;
    while stream.consume_token_with_comments().is_ok() {
        count += 1;
        if enter && stream.unentered_block().is_some() {
            let mut inside = 0;
            let entered = stream.within_block(|nested| {
                inside = drain(nested, true);
                Ok::<_, SyntaxError<'_, ()>>(())
            });
            if entered.is_ok() {
                count += inside;
            }
        }
    }
    count
}

/// Stream property.
fn check_stream(text: &str, picks: &[u8]) -> std::result::Result<(), String> {
    let document = parse(text)?;
    // Entering a group and exhausting it equals skipping it: the same top-level
    // sites, one by one, and every token of the document is visited once.
    let mut skipping = document.root();
    let mut entering = document.root();
    loop {
        let skipped = skipping.consume_token_with_comments();
        let entered = entering.consume_token_with_comments();
        if skipped.is_err() != entered.is_err() {
            return Err(format!("{text:?}: skipping and entering ended apart"));
        }
        if skipped.is_err() {
            break;
        }
        if entering.unentered_block().is_some() {
            let drained = entering.within_block(|nested| {
                drain(nested, true);
                Ok::<_, SyntaxError<'_, ()>>(())
            });
            if drained.is_err() {
                return Err(format!("{text:?}: a group could not be exhausted"));
            }
        }
        if skipping.site() != entering.site() {
            return Err(format!(
                "{text:?}: skipping and entering are at different sites"
            ));
        }
    }
    let visited = drain(&mut document.root(), true);
    if visited != document.node_count() {
        return Err(format!(
            "{text:?}: {visited} tokens visited of {}",
            document.node_count()
        ));
    }
    // A mark restores the stream to where it was taken, however far it went.
    let mut stream = document.root();
    let mut marks = Vec::new();
    for &pick in picks.iter().take(8) {
        for _ in 0..(pick % 4) {
            let _ = stream.consume_token_with_comments();
        }
        marks.push((stream.mark(), stream.site()));
    }
    let _ = drain(&mut stream, picks.first().is_some_and(|pick| pick % 2 == 0));
    for (mark, site) in marks.into_iter().rev() {
        stream.restore(mark);
        if stream.site() != site {
            return Err(format!(
                "{text:?}: restore moved the stream to {:?}",
                stream.site()
            ));
        }
        // What follows a restored mark is what follows it on a stream that went
        // to the end and came back, and what follows does not depend on how often
        // the mark was restored.
        let mut advanced = stream.clone();
        let _ = drain(&mut advanced, false);
        advanced.restore(mark);
        advanced.restore(mark);
        let (mut first, mut second) = (stream.clone(), advanced);
        let (mut left, mut right) = (Vec::new(), Vec::new());
        while let Ok(token) = first.consume_token_with_comments() {
            left.push(render(token));
        }
        while let Ok(token) = second.consume_token_with_comments() {
            right.push(render(token));
        }
        if left != right {
            return Err(format!("{text:?}: a restored mark changed what follows"));
        }
    }
    Ok(())
}

/// Depth property.
fn check_depth(text: &str) -> std::result::Result<(), String> {
    let document = parse(text)?;
    let depth = document.maximum_depth();
    let at = |limit: u32, excess| {
        Document::<()>::parse(
            text,
            &Options::default(),
            Limits::new(limit, usize::MAX, usize::MAX),
            excess,
        )
    };
    if at(depth, DepthExcess::Fail).is_err() {
        return Err(format!(
            "{text:?}: depth {depth} is refused at its own limit"
        ));
    }
    if depth > 0 {
        match at(depth - 1, DepthExcess::Fail) {
            Err(error) if error.is_depth_limit() => {}
            other => return Err(format!("{text:?}: one under the depth gave {other:?}")),
        }
        // Denying keeps every curly block it can and never denies another kind.
        match at(depth - 1, DepthExcess::DenyBlock) {
            Ok(denied) => {
                if denied.maximum_depth() > depth - 1 {
                    return Err(format!("{text:?}: a denied build is deeper than its limit"));
                }
                for list in all_lists(&denied) {
                    for &node in denied.list(list) {
                        if let Component::Denied { range, .. } = denied.component(node) {
                            let range = range.ok_or("a denied block has a range")?;
                            if !text.is_char_boundary(range.start.value())
                                || !text.is_char_boundary(range.end.value())
                                || !text[range.start.value()..].starts_with('{')
                            {
                                return Err(format!(
                                    "{text:?}: a denied block is not a curly block"
                                ));
                            }
                        }
                    }
                }
            }
            Err(error) if error.is_depth_limit() => {}
            Err(error) => return Err(format!("{text:?}: deny failed with {error}")),
        }
    } else if at(0, DepthExcess::DenyBlock).is_err() {
        return Err(format!("{text:?}: a flat text was denied"));
    }
    Ok(())
}

/// Totality property.
fn check_total(text: &str) -> std::result::Result<(), String> {
    for (limit, excess) in [
        (256, DepthExcess::Fail),
        (3, DepthExcess::Fail),
        (3, DepthExcess::DenyBlock),
        (0, DepthExcess::DenyBlock),
    ] {
        let limits = Limits::new(limit, usize::MAX, usize::MAX);
        catch_unwind(AssertUnwindSafe(|| {
            let _ = Document::<()>::parse(text, &Options::default(), limits, excess);
        }))
        .map_err(|_| format!("{text:?}: parse panicked at limit {limit}"))?;
    }
    Ok(())
}

// ----------------------------------------------------------------- the target

fn property(name: &str) -> PropertyId {
    PropertyId::new(format!("surgeist-css-parser::{name}")).expect("owner-qualified property")
}

const TREE: &str = "tree-invariants";
const STREAM: &str = "stream-marks-and-groups";
const DEPTH: &str = "depth-policy";
const TOTALITY: &str = "document-totality";
const PROPERTIES: [&str; 4] = [TREE, STREAM, DEPTH, TOTALITY];

struct Target {
    descriptor: TargetDescriptor,
}

impl Target {
    fn new() -> Self {
        let descriptor = TargetDescriptor::new(TargetDescriptorSpec {
            id: TargetId::new("surgeist-css-parser::document").unwrap(),
            owner: "surgeist-css-parser".into(),
            binary: "surgeist-css-parser-document-property".into(),
            properties: PROPERTIES.map(property).to_vec(),
            source_references: vec![
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#parser-definitions".into(),
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-list-of-components"
                    .into(),
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-simple-block".into(),
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-function".into(),
            ],
            format: CaseFormat::new("surgeist-css-parser::source-text", 1).unwrap(),
            generator_identity: GeneratorIdentity::new(
                "surgeist-css-parser::pieces-raw-brackets",
                1,
            )
            .unwrap(),
            encoding: InputEncoding::RawBytes,
            limits: InputLimits::new(TEXT_BYTES as u64, 0, 0, 0, 0).unwrap(),
            requirements: BuildRequirements::new(vec![], vec![]).unwrap(),
            min_evaluated: u64::from(CASES),
            max_generation_rejections: 0,
        })
        .unwrap();
        Self { descriptor }
    }
}

fn failure(name: &str, detail: String) -> Result<CaseOutcome> {
    let mut detail = detail.replace(char::is_control, " ");
    detail.truncate(2048);
    CaseOutcome::failure(
        FailureKey::new(property(name), FailureClass::PropertyViolation, None)?,
        detail.trim().to_owned(),
    )
}

impl CaseTarget for Target {
    type Case = String;

    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }

    fn decode(&self, input: &[u8]) -> Result<String> {
        String::from_utf8(input.to_vec())
            .map_err(|_| FuzzError::new(FuzzErrorKind::InvalidCase, "decode css text", "UTF-8"))
    }

    fn encode(&self, case: &String) -> Result<Vec<u8>> {
        Ok(case.as_bytes().to_vec())
    }

    fn generate(&self, input: &[u8]) -> Result<Generation<String>> {
        Ok(Generation::Generated(text_of(input)))
    }

    fn evaluate(&self, text: &String) -> Result<CaseOutcome> {
        // The text is its own entropy for the stream picks: its bytes.
        let picks = text.as_bytes();
        if let Err(detail) = check_total(text) {
            return failure(TOTALITY, detail);
        }
        if let Err(detail) = check_tree(text) {
            return failure(TREE, detail);
        }
        if let Err(detail) = check_stream(text, picks) {
            return failure(STREAM, detail);
        }
        if let Err(detail) = check_depth(text) {
            return failure(DEPTH, detail);
        }
        CaseOutcome::evaluated(PROPERTIES.map(property).to_vec())
    }
}

#[test]
fn generated_text_builds_the_tree_of_the_specification_and_streams_over_it_consistently() {
    let target = Target::new();
    let report = run_property(
        &target,
        &bytes(ENTROPY_BYTES, target.descriptor().limits()),
        PropertyConfig {
            cases: CASES,
            max_rejections: 0,
            max_shrink_iterations: 96,
            seed: SEED,
        },
        &mut |_| Ok(()),
    )
    .expect("property run completes with sufficient coverage");
    if let Some(failure) = report.failure {
        panic!(
            "{:?}: {}\ntext: {:?}",
            failure.outcome.failure_key(),
            failure.outcome.detail().unwrap_or_default(),
            String::from_utf8_lossy(&failure.canonical)
        );
    }
    assert!(report.counts.evaluated() >= u64::from(CASES));
}

#[test]
fn the_checks_can_fail() {
    // A tree with a group closed by the wrong token is not the tree of the
    // sections, and the oracle sees the difference: `(]` leaves the paren block
    // unclosed with a stray square closer inside it.
    let (tokens, _) = lex("(]");
    let mut at = 0;
    let (reference, _) = consume_list(&tokens, &mut at, None);
    assert_eq!(reference_outline(&reference), "OpenParen<CloseSquare…>");
    let (tokens, _) = lex("()");
    let mut at = 0;
    let (reference, _) = consume_list(&tokens, &mut at, None);
    assert_eq!(reference_outline(&reference), "OpenParen<>");
    assert_eq!(reference_depth(&reference), 1);
    // The generators reach every family.
    assert!(text_of(&[1, 0, 13, 12, 92, 10, 34]).contains('\r'));
    assert!(text_of(&[2, 0, 1, 2, 3]).contains('('));
}
