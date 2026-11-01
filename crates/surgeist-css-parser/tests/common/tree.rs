//! Helpers shared by the document and stream tests: building a document from
//! text and rendering its tree as a string, so an expectation can be written
//! by hand from the specification.
//!
//! The renderings are a notation; they carry no behavior of their own.

use std::cell::{Cell, RefCell};
use std::ops::Range;

use surgeist_css_parser::document::{Component, GroupEnd, ListId};
use surgeist_css_parser::lexer::Options;
use surgeist_css_parser::limits::{DepthExcess, Failure, Limits, Meter, Refused};
use surgeist_css_parser::{Document, Offset};

use super::render;

/// Parses `text` with the default limits and options.
pub fn parse(text: &str) -> Document<'_> {
    Document::parse(
        text,
        &Options::default(),
        Limits::default(),
        0,
        DepthExcess::Fail,
    )
    .expect("the default limits admit the text")
}

/// The interval as plain numbers.
pub fn span(range: Range<Offset>) -> Range<usize> {
    range.start.value()..range.end.value()
}

/// Renders the list as the tokens it holds, a group as its opening token, its
/// contents between `<` and `>`, and a `…` before the `>` if the input ended
/// the group instead of a closing token.
///
/// `Function("f")<Ident("a")>` is `f(a)`; `Function("f")<Ident("a")…>` is
/// `f(a`.
pub fn outline(document: &Document<'_>, list: ListId) -> String {
    let mut parts = Vec::new();
    for &node in document.list(list) {
        parts.push(match document.component(node) {
            Component::Token { token, .. } => render(token),
            Component::Function { children, .. } | Component::Block { children, .. } => {
                let ended = match document.group_end(node) {
                    Some(GroupEnd::Eof { .. }) => "…",
                    _ => "",
                };
                format!(
                    "{}<{}{ended}>",
                    render(document.token(node)),
                    outline(document, children)
                )
            }
            Component::Denied { .. } => "Denied".to_owned(),
            _ => unreachable!("the components of the specification and the denied block"),
        });
    }
    parts.join(" ")
}

/// The outline of the root list of `text`.
pub fn outline_of(text: &str) -> String {
    let document = parse(text);
    outline(&document, document.root_list())
}

/// One call a container made on its meter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Call {
    Step,
    Add(usize),
    Released(usize),
    Note(Failure),
}

/// A meter that records every call and refuses the fallible call (a `step` or
/// an `add`, counted from zero in the order they are made) at one chosen index.
/// It also keeps the live storage it was told about.
#[derive(Default)]
pub struct Recording {
    calls: RefCell<Vec<Call>>,
    fallible: Cell<usize>,
    live: Cell<i64>,
    refuse_at: Cell<Option<usize>>,
    refuse_from: Cell<Option<usize>>,
}

impl Recording {
    pub fn admitting() -> Self {
        Self::default()
    }

    pub fn refusing_call(index: usize) -> Self {
        Self {
            refuse_at: Cell::new(Some(index)),
            ..Self::default()
        }
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.borrow().clone()
    }

    /// Forgets the calls recorded so far and starts counting fallible calls
    /// from zero; the live storage is kept.
    pub fn forget(&self) {
        self.calls.borrow_mut().clear();
        self.fallible.set(0);
    }

    /// Admits every fallible call again.
    pub fn admit_all(&self) {
        self.refuse_at.set(None);
        self.refuse_from.set(None);
    }

    /// Forgets the calls recorded so far and refuses the fallible call at
    /// `index`, counted from now.
    pub fn refuse_call(&self, index: usize) {
        self.forget();
        self.refuse_at.set(Some(index));
        self.refuse_from.set(None);
    }

    /// Forgets the calls recorded so far and refuses every fallible call from
    /// `index` on, counted from now: a host that has said stop keeps saying it.
    pub fn refuse_from(&self, index: usize) {
        self.forget();
        self.refuse_at.set(None);
        self.refuse_from.set(Some(index));
    }

    /// The bytes added and not yet released.
    pub fn live(&self) -> i64 {
        self.live.get()
    }

    /// The number of fallible calls made since the last [`forget`](Self::forget).
    pub fn fallible_calls(&self) -> usize {
        self.fallible.get()
    }

    fn admit(&self, call: Call) -> Result<(), Refused> {
        self.calls.borrow_mut().push(call);
        let index = self.fallible.get();
        self.fallible.set(index + 1);
        if self.refuse_at.get() == Some(index)
            || self.refuse_from.get().is_some_and(|from| index >= from)
        {
            Err(Refused)
        } else {
            Ok(())
        }
    }
}

impl Meter for Recording {
    fn step(&self) -> Result<(), Refused> {
        self.admit(Call::Step)
    }

    fn add(&self, bytes: usize) -> Result<(), Refused> {
        self.admit(Call::Add(bytes))?;
        self.live
            .set(self.live.get() + i64::try_from(bytes).unwrap());
        Ok(())
    }

    fn released(&self, bytes: usize) {
        self.calls.borrow_mut().push(Call::Released(bytes));
        self.live
            .set(self.live.get() - i64::try_from(bytes).unwrap());
    }

    fn note(&self, failure: Failure) -> Refused {
        self.calls.borrow_mut().push(Call::Note(failure));
        Refused
    }
}

/// A document built under `meter` from a list of the tokens named by a small
/// notation, separated by spaces: `_` whitespace, `/` a comment, `;`, `:`, `,`,
/// `!` (a delim), `{` and `(` and `[` and `f(` opening a group, `}`, `)` and `]`
/// closing the innermost one (a closing token with none open is a token of the
/// root list), `@x` an at-keyword, and anything else an ident.
/// The meter admits everything and its record is emptied afterwards, so a test
/// sees only what a stream does.
pub fn metered_document<'m>(meter: &'m Recording, notation: &str) -> Document<'m> {
    use surgeist_css_parser::Token;
    use surgeist_css_parser::document::{NodeId, TreeBuilder};

    fn leak(text: &str) -> &'static str {
        Box::leak(text.to_owned().into_boxed_str())
    }

    meter.admit_all();
    let mut builder: TreeBuilder<'m, ()> = Document::builder(Limits::default(), 0, Some(meter))
        .expect("the builder admits its first allocation");
    let mut open: Vec<(NodeId, ListId)> = Vec::new();
    for word in notation.split_whitespace() {
        let parent = open
            .last()
            .map_or_else(|| builder.root(), |(_, list)| *list);
        match word {
            "{" => open.push(builder.open(parent, Token::OpenCurly, None, None).unwrap()),
            "(" => open.push(builder.open(parent, Token::OpenParen, None, None).unwrap()),
            "[" => open.push(builder.open(parent, Token::OpenSquare, None, None).unwrap()),
            "}" | ")" | "]" if open.is_empty() => {
                // A closing token that closes nothing is a token of the list.
                let token = match word {
                    "}" => Token::CloseCurly,
                    ")" => Token::CloseParen,
                    _ => Token::CloseSquare,
                };
                builder.leaf(parent, token, None, None).unwrap();
            }
            "}" | ")" | "]" => {
                let (group, _) = open.pop().expect("a group is open");
                builder
                    .close(
                        group,
                        GroupEnd::Closed {
                            range: None,
                            payload: None,
                        },
                    )
                    .unwrap();
            }
            word if word.len() > 1 && word.ends_with('(') => {
                let name = leak(&word[..word.len() - 1]);
                open.push(
                    builder
                        .open(parent, Token::Function(name), None, None)
                        .unwrap(),
                );
            }
            _ => {
                let token = match word {
                    "_" => Token::Whitespace,
                    "/" => Token::Comment,
                    ";" => Token::Semicolon,
                    ":" => Token::Colon,
                    "," => Token::Comma,
                    "!" => Token::Delim('!'),
                    word if word.starts_with('@') => Token::AtKeyword(leak(&word[1..])),
                    word => Token::Ident(leak(word)),
                };
                builder.leaf(parent, token, None, None).unwrap();
            }
        }
    }
    while let Some((group, _)) = open.pop() {
        builder.close(group, GroupEnd::Eof { at: None }).unwrap();
    }
    let document = builder.finish(None);
    meter.forget();
    document
}
