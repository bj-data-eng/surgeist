# html5gum: pinned parser contracts

Research capture: 2026-10-10. Source revision `51c00d7c47a5780aff913c7937dea064a239b730`. These are implementation witnesses, not normative HTML requirements or an adoption decision. Original source bodies are retained verbatim in fenced blocks; partial files explicitly identify original line ranges. SHA-256 identifies the complete original file.

Attribution and licenses: [local notice](../licenses/html5gum/NOTICE.md).

## README.md

[Exact source](https://github.com/untitaker/html5gum/blob/51c00d7c47a5780aff913c7937dea064a239b730/README.md); SHA-256 `d0ce8890af33fe6d141fa000d040959cad5998492d250126d6740a2bf7897130`.

Complete file:

````text
# html5gum

[![docs.rs](https://img.shields.io/docsrs/html5gum)](https://docs.rs/html5gum)
[![crates.io](https://img.shields.io/crates/l/html5gum.svg)](https://crates.io/crates/html5gum)

`html5gum` is a WHATWG-compliant HTML tokenizer.

```rust
use std::fmt::Write;
use html5gum::{Tokenizer, Token};

let html = "<title   >hello world</title>";
let mut new_html = String::new();

for Ok(token) in Tokenizer::new(html) {
    match token {
        Token::StartTag(tag) => {
            write!(new_html, "<{}>", String::from_utf8_lossy(&tag.name)).unwrap();
        }
        Token::String(hello_world) => {
            write!(new_html, "{}", String::from_utf8_lossy(&hello_world)).unwrap();
        }
        Token::EndTag(tag) => {
            write!(new_html, "</{}>", String::from_utf8_lossy(&tag.name)).unwrap();
        }
        _ => panic!("unexpected input"),
    }
}

assert_eq!(new_html, "<title>hello world</title>");
```

`html5gum` provides multiple kinds of APIs:

* Iterating over tokens as shown above.
* Implementing your own `Emitter` for maximum performance, see [the `custom_emitter.rs` example][examples/custom_emitter.rs].
* A callbacks-based API for a middleground between convenience and performance, see [the `callback_emitter.rs` example][examples/callback_emitter.rs].
* With the `tree-builder` feature, html5gum can be integrated with `html5ever` and `scraper`. See [the `scraper.rs` example][examples/scraper.rs].

## What a tokenizer does and what it does not do

`html5gum` fully implements [13.2.5 of the WHATWG HTML
spec](https://html.spec.whatwg.org/#tokenization), i.e. is able to tokenize HTML documents and passes [html5lib's tokenizer
test suite](https://github.com/html5lib/html5lib-tests/tree/master/tokenizer). Since it is just a tokenizer, this means:

* `html5gum` **does not** [implement charset
  detection.](https://html.spec.whatwg.org/#determining-the-character-encoding)
  This implementation takes and returns bytes, but assumes UTF-8. It recovers
  gracefully from invalid UTF-8.
* `html5gum` **does not** [correct mis-nested
  tags.](https://html.spec.whatwg.org/#an-introduction-to-error-handling-and-strange-cases-in-the-parser)
* `html5gum` doesn't implement the DOM, and unfortunately in the HTML spec,
  constructing the DOM ("tree construction") influences how tokenization is
  done. For an example of which problems this causes see [this example
  code][examples/tokenize_with_state_switches.rs].
* `html5gum` **does not** generally qualify as a browser-grade HTML *parser* as
  per the WHATWG spec. This can change in the future, see [issue
  21](https://github.com/untitaker/html5gum/issues/21).

With those caveats in mind, `html5gum` can pretty much ~parse~ _tokenize_
anything that browsers can. However, using the experimental `tree-builder`
feature, html5gum can be integrated with `html5ever` and `scraper`. See [the
`scraper.rs` example][examples/scraper.rs].

## Other features

* No unsafe Rust
* Only dependency is `jetscii`, and can be disabled via crate features (see `Cargo.toml`)

## Alternative HTML parsers

`html5gum` was created out of a need to parse HTML tag soup efficiently. Previous options were to:

* use [quick-xml](https://github.com/tafia/quick-xml/) or
  [xmlparser](https://github.com/RazrFalcon/xmlparser) with some hacks to make
  either one not choke on bad HTML. For some (rather large) set of HTML input
  this works well (particularly `quick-xml` can be configured to be very
  lenient about parsing errors) and parsing speed is stellar. But neither can
  parse all HTML.

  For my own usecase `html5gum` is about 2x slower than `quick-xml`.

* use [html5ever's own
  tokenizer](https://docs.rs/html5ever/0.25.1/html5ever/tokenizer/index.html)
  to avoid as much tree-building overhead as possible. This was functional but
  had poor performance for my own usecase (10-15x slower than `quick-xml`).

* use [lol-html](https://github.com/cloudflare/lol-html), which would probably
  perform at least as well as `html5gum`, but comes with a closure-based API
  that I didn't manage to get working for my usecase.

## Etymology

Why is this library called `html5gum`?

* G.U.M: **G**iant **U**nreadable **M**atch-statement

* \<insert "how it feels to <s>chew 5 gum</s> _parse HTML_" meme here\>

## License

Licensed under the MIT license, see [`./LICENSE`][LICENSE].


<!-- These link destinations are defined like this so that src/lib.rs can override them. -->
[LICENSE]: ./LICENSE
[examples/tokenize_with_state_switches.rs]: ./examples/tokenize_with_state_switches.rs
[examples/custom_emitter.rs]: ./examples/custom_emitter.rs
[examples/callback_emitter.rs]: ./examples/callback_emitter.rs
[examples/scraper.rs]: ./examples/scraper.rs
````

## Cargo.toml

[Exact source](https://github.com/untitaker/html5gum/blob/51c00d7c47a5780aff913c7937dea064a239b730/Cargo.toml); SHA-256 `a3dddc03f898f9c42e38c6939a2e25a7a9f945df5559a846d67db02c223ad142`.

Complete file:

````text
[package]
name = "html5gum"
authors = ["Markus Unterwaditzer <markus-honeypot@unterwaditzer.net>"]
description = "A WHATWG-compliant HTML5 tokenizer and tag soup parser."
edition = "2018"
readme = "README.md"
keywords = ["html", "html5", "whatwg", "parser", "tokenizer"]
categories = [ "parser-implementations", "web-programming" ]
license = "MIT"
repository = "https://github.com/untitaker/html5gum"
version = "0.8.4"
include = ["src/**/*", "LICENSE", "README.md", "benches"]

[dev-dependencies]
pretty_assertions = "1.0.0"
serde = { version = "1.0.130", features = ["derive"] }
serde_json = "1.0.71"
test-generator = "0.3.0"
serde_bytes = "0.11.5"
glob = "0.3.0"
libtest-mimic = "0.8.1"
# https://github.com/bheisler/iai/issues/34
# need to have cache simulation running because of bencher.dev
iai = { git = "https://github.com/sigaloid/iai", rev = "d56a597" }
markup5ever_rcdom = "0.5.0-unofficial"
# required for examples/scraper.rs
scraper = "0.21.0"
argh = "0.1.12"
annotate-snippets = "0.11.5"

[features]
# By default this crate depends on the jetscii library for best performance.
# Disabling this feature will leave you with 100% safe Rust and no dependencies.
# This may come in handy if you encounter packaging/build problems.
default = ["jetscii"]

# The tree-builder feature contains utilities to use html5ever's DOM and tree
# builder with html5gum's tokenizer.
tree-builder = ["html5ever"]

# Features for guided fuzzing using IJON: https://github.com/AFLplusplus/AFLplusplus/blob/stable/docs/IJON.md
afl-ijon = ["afl"]

[dependencies]
html5ever = { version = "0.29.0", optional = true }
jetscii = { version = "0.5.1", optional = true }
afl = { version = "0.17.0", optional = true }

[[bench]]
name = "patterns"
harness = false

[[bench]]
name = "files"
harness = false

[[test]]
name = "html5lib-tokenizer"
path = "tests/html5lib_tokenizer.rs"
harness = false

[[test]]
name = "html5lib-tree-builder"
path = "tests/html5lib_tree_builder.rs"
required-features = ["tree-builder"]
harness = false

[[test]]
name = "test_spans"
path = "tests/test_spans.rs"
harness = false

[[example]]
name = "build_tree"
required-features = ["tree-builder"]

[[example]]
name = "custom_emitter"

[[example]]
name = "callback_emitter"

[[example]]
name = "scraper"
required-features = ["tree-builder"]

[[example]]
name = "spans"

[lib]
bench = false
````

## src/span.rs

[Exact source](https://github.com/untitaker/html5gum/blob/51c00d7c47a5780aff913c7937dea064a239b730/src/span.rs); SHA-256 `2076ff7f853bb45518cfaca79fb336b2c05ead1ebcb04eca77c10ad140606cbe`.

Complete file:

````text
/// A single bound of a [`Span`].
///
/// For example use `()` as a bound to ignore spans and use [`usize`] for the default
/// implementation.
pub trait SpanBound:
    Sized
    + Clone
    + Copy
    + std::fmt::Debug
    + Default
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + std::hash::Hash
{
    /// Offset the bound by a given value.
    #[must_use]
    fn offset(self, by: isize) -> Self;
}

/// Position/ boundary `start..end` in the input.
///
/// The position will mostly be a byte offset, but depending on the [crate::Reader] it originates
/// from, it can be something entirely else.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Span<B: SpanBound = usize> {
    /// Start position (inclusive) of the span.
    pub start: B,
    /// End position (exclusive) of the span.
    pub end: B,
}

impl Span<()> {
    /// Dummy empty span for tests.
    #[doc(hidden)]
    pub const DUMMY: Self = Self { start: (), end: () };
}

impl SpanBound for () {
    fn offset(self, _by: isize) -> Self {}
}

impl SpanBound for usize {
    fn offset(self, by: isize) -> Self {
        self.saturating_add_signed(by)
    }
}

/// A value together with its [`Span`].
///
/// This type implements [`Deref`](std::ops::Deref) and [`DerefMut`](std::ops::DerefMut),
/// allowing you to access the inner value directly without using `.value`:
///
/// ```
/// # use html5gum::Spanned;
/// let spanned: Spanned<String, ()> = "hello".to_string().into();
/// assert_eq!(spanned.len(), 5);  // calls String::len() via Deref
/// assert_eq!(&*spanned, "hello"); // dereference to get &String
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Spanned<T, B: SpanBound = usize> {
    pub value: T,
    pub span: Span<B>,
}

impl<T, B: SpanBound> From<T> for Spanned<T, B> {
    fn from(value: T) -> Self {
        Self {
            value,
            span: Span::default(),
        }
    }
}

impl<B: SpanBound> From<Vec<u8>> for Spanned<crate::HtmlString, B> {
    fn from(value: Vec<u8>) -> Self {
        Self {
            value: value.into(),
            span: Span::default(),
        }
    }
}

impl<T, B: SpanBound> std::ops::Deref for Spanned<T, B> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, B: SpanBound> std::ops::DerefMut for Spanned<T, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
````

## src/emitters/html5ever.rs

[Exact source](https://github.com/untitaker/html5gum/blob/51c00d7c47a5780aff913c7937dea064a239b730/src/emitters/html5ever.rs); SHA-256 `ac744d64ca4bfd1e6abe417b8a91de922ad20719d5059ef669401e524651bc90`.

Complete file:

````text
//! See [`examples/scraper.rs`] for usage.
use std::convert::Infallible;

use crate::emitters::callback::{Callback, CallbackEmitter, CallbackEvent};
use crate::utils::trace_log;
use crate::{Emitter, ForwardingEmitter, Readable, Reader, Span, State, Tokenizer};

use html5ever::interface::{create_element, TreeSink};
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::states::State as Html5everState;
use html5ever::tokenizer::{
    states::RawKind, Doctype, Tag, TagKind, Token as Html5everToken, TokenSink, TokenSinkResult,
};
use html5ever::tree_builder::TreeBuilder;
use html5ever::LocalName;
use html5ever::ParseOpts;
use html5ever::{Attribute, QualName};

const BOGUS_LINENO: u64 = 1;

fn to_tendril(bytes: &[u8]) -> StrTendril {
    StrTendril::from_slice(&String::from_utf8_lossy(bytes))
}

fn to_local_name(bytes: &[u8]) -> LocalName {
    LocalName::from(&*String::from_utf8_lossy(bytes))
}

#[derive(Debug)]
struct OurCallback<'a, S> {
    sink: &'a mut S,
    current_start_tag: Option<Tag>,
    next_state: Option<State>,
}

impl<'a, S: TokenSink> OurCallback<'a, S> {
    fn handle_sink_result<H>(&mut self, result: TokenSinkResult<H>) {
        match result {
            TokenSinkResult::Continue => {}
            TokenSinkResult::Script(_) => {
                self.next_state = Some(State::Data);
                // TODO: suspend tokenizer for script
            }
            TokenSinkResult::Plaintext => {
                self.next_state = Some(State::PlainText);
            }
            TokenSinkResult::RawData(RawKind::Rcdata) => {
                self.next_state = Some(State::RcData);
            }
            TokenSinkResult::RawData(RawKind::Rawtext) => {
                self.next_state = Some(State::RawText);
            }
            TokenSinkResult::RawData(RawKind::ScriptData) => {
                self.next_state = Some(State::ScriptData);
            }
            TokenSinkResult::RawData(RawKind::ScriptDataEscaped(_)) => {
                todo!()
            }
        }
    }

    fn sink_token(&mut self, token: Html5everToken) {
        trace_log!("sink_token: {:?}", token);
        let result = self.sink.process_token(token, BOGUS_LINENO);
        self.handle_sink_result(result);
    }
}

impl<'a, S: TokenSink> Callback<Infallible, ()> for OurCallback<'a, S> {
    fn handle_event(&mut self, event: CallbackEvent<'_>, _span: Span<()>) -> Option<Infallible> {
        trace_log!("Html5everEmitter::handle_event: {:?}", event);
        match event {
            CallbackEvent::OpenStartTag { name } => {
                self.current_start_tag = Some(Tag {
                    kind: TagKind::StartTag,
                    name: to_local_name(name),
                    self_closing: false,
                    attrs: Default::default(),
                });
            }
            CallbackEvent::AttributeName { name } => {
                if let Some(ref mut tag) = self.current_start_tag {
                    tag.attrs.push(Attribute {
                        name: QualName::new(None, Default::default(), to_local_name(name)),
                        value: Default::default(),
                    });
                }
            }
            CallbackEvent::AttributeValue { value } => {
                if let Some(ref mut tag) = self.current_start_tag {
                    if let Some(attr) = tag.attrs.last_mut() {
                        attr.value.push_slice(&String::from_utf8_lossy(value));
                    }
                }
            }
            CallbackEvent::CloseStartTag { self_closing } => {
                if let Some(mut tag) = self.current_start_tag.take() {
                    tag.self_closing = self_closing;
                    self.sink_token(Html5everToken::TagToken(tag));
                }
            }
            CallbackEvent::EndTag { name } => {
                self.sink_token(Html5everToken::TagToken(Tag {
                    kind: TagKind::EndTag,
                    name: to_local_name(name),
                    self_closing: false,
                    attrs: Default::default(),
                }));
            }
            CallbackEvent::String { value } => {
                let mut first = true;
                for part in value.split(|&b| b == 0) {
                    if !first {
                        self.sink_token(Html5everToken::NullCharacterToken);
                    }

                    first = false;
                    self.sink_token(Html5everToken::CharacterTokens(to_tendril(part)));
                }
            }
            CallbackEvent::Comment { value } => {
                self.sink_token(Html5everToken::CommentToken(to_tendril(value)));
            }
            CallbackEvent::Doctype {
                name,
                public_identifier,
                system_identifier,
                force_quirks,
            } => {
                self.sink_token(Html5everToken::DoctypeToken(Doctype {
                    name: Some(name).filter(|x| !x.is_empty()).map(to_tendril),
                    public_id: public_identifier.map(to_tendril),
                    system_id: system_identifier.map(to_tendril),
                    force_quirks,
                }));
            }
            CallbackEvent::Error(error) => {
                self.sink_token(Html5everToken::ParseError(error.as_str().into()));
            }
        }

        None
    }
}

/// A compatibility layer that allows you to plug the TreeBuilder from html5ever into the tokenizer
/// from html5gum.
///
/// See [`examples/scraper.rs`] for usage.
#[derive(Debug)]
pub struct Html5everEmitter<'a, S: TokenSink> {
    emitter_inner: CallbackEmitter<OurCallback<'a, S>>,
}

impl<'a, S: TokenSink> Html5everEmitter<'a, S> {
    /// Construct the compatibility layer.
    pub fn new(sink: &'a mut S) -> Self {
        Html5everEmitter {
            emitter_inner: CallbackEmitter::new(OurCallback {
                sink,
                current_start_tag: None,
                next_state: None,
            }),
        }
    }
}

impl<'a, S: TokenSink> ForwardingEmitter for Html5everEmitter<'a, S> {
    type Token = Infallible;

    fn inner(&mut self) -> &mut impl Emitter<Token = Self::Token> {
        &mut self.emitter_inner
    }

    fn emit_eof(&mut self) {
        self.emitter_inner.emit_eof();
        let sink = &mut self.emitter_inner.callback_mut().sink;
        let _ignored = sink.process_token(Html5everToken::EOFToken, BOGUS_LINENO);
        sink.end();
    }

    fn emit_current_tag(&mut self) -> Option<State> {
        assert!(self.emitter_inner.emit_current_tag().is_none());
        self.emitter_inner.callback_mut().next_state.take()
    }

    fn adjusted_current_node_present_but_not_in_html_namespace(&mut self) -> bool {
        self.emitter_inner
            .callback_mut()
            .sink
            .adjusted_current_node_present_but_not_in_html_namespace()
    }
}

fn map_tokenizer_state(input: Html5everState) -> State {
    match input {
        Html5everState::Data => State::Data,
        Html5everState::Plaintext => State::PlainText,
        Html5everState::RawData(RawKind::Rcdata) => State::RcData,
        Html5everState::RawData(RawKind::Rawtext) => State::RawText,
        Html5everState::RawData(RawKind::ScriptData) => State::ScriptData,
        x => todo!("{:?}", x),
    }
}

/// Parse an HTML fragment
///
/// This is a convenience function for using [Html5everEmitter] together with html5ever. It is
/// equivalent to the same functions in [html5ever::driver].
///
/// ```
/// use html5ever::{local_name, interface::TreeSink, QualName, ns, namespace_url}; // extern crate html5ever;
/// use scraper::{HtmlTreeSink, Html}; // extern crate scraper;
///
/// let input = "<h1>hello world</h1>";
///
/// // equivalent to `Html::parse_fragment`
/// let dom = Html::new_fragment();
/// let tree_sink = HtmlTreeSink::new(dom);
/// let Ok(tree_sink) = html5gum::emitters::html5ever::parse_fragment(
///     input,
///     tree_sink,
///     Default::default(),
///     QualName::new(None, ns!(html), local_name!("body")),
///     Vec::new()
/// );
/// let dom: Html = tree_sink.finish();
/// ```
pub fn parse_fragment<'a, R, Sink>(
    input: R,
    sink: Sink,
    opts: ParseOpts,
    context_name: QualName,
    context_attrs: Vec<Attribute>,
) -> Result<Sink, <R::Reader as Reader>::Error>
where
    R: Readable<'a>,
    Sink: TreeSink,
{
    let context_elem = create_element(&sink, context_name, context_attrs);
    parse_fragment_for_element(input, sink, opts, context_elem, None)
}

/// Like `parse_fragment`, but with an existing context element
/// and optionally a form element.
///
/// This is a convenience function for using [Html5everEmitter] together with html5ever. It is
/// equivalent to the same functions in [html5ever::driver].
pub fn parse_fragment_for_element<'a, R, Sink>(
    input: R,
    sink: Sink,
    opts: ParseOpts,
    context_element: Sink::Handle,
    form_element: Option<Sink::Handle>,
) -> Result<Sink, <R::Reader as Reader>::Error>
where
    R: Readable<'a>,
    Sink: TreeSink,
{
    let mut tree_builder =
        TreeBuilder::new_for_fragment(sink, context_element, form_element, opts.tree_builder);

    let initial_state = map_tokenizer_state(tree_builder.tokenizer_state_for_context_elem());
    let token_emitter = Html5everEmitter::new(&mut tree_builder);
    let mut tokenizer = Tokenizer::new_with_emitter(input, token_emitter);
    tokenizer.set_state(initial_state);
    tokenizer.finish()?;
    Ok(tree_builder.sink)
}

/// Parse an HTML document.
///
/// This is a convenience function for using [Html5everEmitter] together with html5ever. It is
/// equivalent to the same functions in [html5ever::driver].
///
/// ```rust
/// use html5ever::interface::TreeSink; // extern crate html5ever;
/// use scraper::{HtmlTreeSink, Html}; // extern crate scraper;
///
/// let input = "<h1>hello world</h1>";
///
/// // equivalent to `Html::parse_document`
/// let dom = Html::new_document();
/// let tree_sink = HtmlTreeSink::new(dom);
/// let Ok(tree_sink) = html5gum::emitters::html5ever::parse_document(
///     input,
///     tree_sink,
///     Default::default()
/// );
/// let dom: Html = tree_sink.finish();
/// ```
pub fn parse_document<'a, R, Sink>(
    input: R,
    sink: Sink,
    opts: ParseOpts,
) -> Result<Sink, <R::Reader as Reader>::Error>
where
    R: Readable<'a>,
    Sink: TreeSink,
{
    let mut tree_builder = TreeBuilder::new(sink, opts.tree_builder);
    let token_emitter = Html5everEmitter::new(&mut tree_builder);
    let tokenizer = Tokenizer::new_with_emitter(input, token_emitter);
    tokenizer.finish()?;
    Ok(tree_builder.sink)
}
````
