# html5ever: pinned parser contracts

Research capture: 2026-10-10. Source revision `d7232d746d3112f08926d169591576bd12fe2fdf`. These are implementation witnesses, not normative HTML requirements or an adoption decision. Original source bodies are retained verbatim in fenced blocks; partial files explicitly identify original line ranges. SHA-256 identifies the complete original file.

Attribution and licenses: [local notice](../licenses/html5ever/NOTICE.md).

## README.md

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/README.md); SHA-256 `0f0989d8abaf2539dc05d6b0dc5851dc432c6badae34d9bd23b76c56991ea254`.

Complete file:

````text
# html5ever

[![Build Status](https://github.com/servo/html5ever/actions/workflows/main.yml/badge.svg)](https://github.com/servo/html5ever/actions)
[![crates.io](https://img.shields.io/crates/v/html5ever.svg)](https://crates.io/crates/html5ever)

[API Documentation][API documentation]

html5ever is an HTML parser developed as part of the [Servo][] project.

It can parse and serialize HTML according to the [WHATWG](https://whatwg.org/) specs (aka "HTML5"). However, there are some differences in the actual behavior currently, most of which are documented [in the bug tracker][]. html5ever passes all tokenizer tests from [html5lib-tests][], with most tree builder tests outside of the unimplemented features. The goal is to pass all html5lib tests, while also providing all hooks needed by a production web browser, e.g. `document.write`.

Note that the HTML syntax is very similar to XML. For correct parsing of XHTML, use an XML parser (that said, many XHTML documents in the wild are serialized in an HTML-compatible form).

html5ever is written in [Rust][], therefore it avoids the notorious security problems that come along with using C. Being built with Rust also makes the library come with the high-grade performance you would expect from an HTML parser written in C. html5ever is basically a C HTML parser, but without needing a garbage collector or other heavy runtime processes.


## Getting started in Rust

Add html5ever as a dependency:

```bash
cargo add html5ever
```

You should also take a look at [`examples/html2html.rs`], [`examples/print-rcdom.rs`], and the [API documentation][].


## Getting started in other languages

Bindings for Python and other languages are much desired.


## Working on html5ever

To fetch the test suite, you need to run

```
git submodule update --init
```

Run `cargo doc` in the repository root to build local documentation under `target/doc/`.


## Details

html5ever uses callbacks to manipulate the DOM, therefore it does not provide any DOM tree representation. 

html5ever exclusively uses UTF-8 to represent strings. In the future it will support other document encodings (and UCS-2 `document.write`) by converting input.

The code is cross-referenced with the WHATWG syntax spec, and eventually we will have a way to present code and spec side-by-side.

html5ever builds against the official stable releases of Rust, though some optimizations are only supported on nightly releases.

[API documentation]: https://docs.rs/html5ever
[Servo]: https://github.com/servo/servo
[Rust]: https://www.rust-lang.org/
[in the bug tracker]: https://github.com/servo/html5ever/issues?q=is%3Aopen+is%3Aissue+label%3Aweb-compat
[html5lib-tests]: https://github.com/html5lib/html5lib-tests
[`examples/html2html.rs`]: https://github.com/servo/html5ever/blob/main/rcdom/examples/html2html.rs
[`examples/print-rcdom.rs`]: https://github.com/servo/html5ever/blob/main/rcdom/examples/print-rcdom.rs
````

## Cargo.toml

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/Cargo.toml); SHA-256 `89e8dd7d1342fb3a83de05a52a5ecd01376c34e99f3e5bd7980d50554e523584`.

Complete file:

````text
[workspace]
resolver = "2"
members = [
    "web_atoms",
    "markup5ever",
    "html5ever",
    "rcdom",
    "xml5ever",
    "tendril",
    "tendril-bench",
]

[workspace.package]
version = "0.40.1"
license = "MIT OR Apache-2.0"
authors = [ "The html5ever Project Developers" ]
repository = "https://github.com/servo/html5ever"
edition = "2021"
rust-version = "1.85"

[workspace.dependencies]
# Repo dependencies
tendril = { version = "0.5", path = "tendril" }
web_atoms = { version = "0.3.0", path = "web_atoms" }
markup5ever = { version = "0.40", path = "markup5ever" }
xml5ever = { version = "0.40", path = "xml5ever" }
html5ever = { version = "0.40", path = "html5ever" }

# External dependencies
encoding_rs = "0.8.12"
log = "0.4"
memchr = "2.8.0"
new_debug_unreachable = "1.0.2"
phf = "0.14"
phf_codegen = "0.14"
string_cache = { version = "0.11.0", default-features = false }
string_cache_codegen = "0.11.0"

# Dev dependencies
criterion = "0.8"
env_logger = "0.11"
libtest-mimic = "0.8.1"
rand = "0.9"
serde_json = "1.0"
typed-arena = "2.0.2"
````

## html5ever/Cargo.toml

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/html5ever/Cargo.toml); SHA-256 `843b72c46f17c45eb127e10abc642d75911ca08048d908a3638415f47070eba3`.

Complete file:

````text
[package]
name = "html5ever"
description = "High-performance browser-grade HTML5 parser"
documentation = "https://docs.rs/html5ever"
categories = [ "parser-implementations", "web-programming" ]
keywords = ["html", "html5", "parser", "parsing"]
readme = "../README.md"
version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
edition.workspace = true
rust-version.workspace = true

[features]
trace_tokenizer = []
serde = ["markup5ever/serde"]

[dependencies]
markup5ever = { workspace = true }
memchr = { workspace = true }
log = { workspace = true }

[dev-dependencies]
criterion = { workspace = true }
typed-arena = { workspace = true }

[[bench]]
name = "html5ever"
harness = false
````

## markup5ever/Cargo.toml

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/markup5ever/Cargo.toml); SHA-256 `eb5b6f8e08ca88767d271c6fa89b6484fe310f23bbe44cf717b818527e3afa63`.

Complete file:

````text
[package]
name = "markup5ever"
description = "Common code for xml5ever and html5ever"
documentation = "https://docs.rs/markup5ever"
categories = [ "parser-implementations", "web-programming" ]
version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
edition.workspace = true
rust-version.workspace = true

[lib]
path = "lib.rs"

[features]
serde = ["web_atoms/serde"]

[dependencies]
web_atoms = { workspace = true }
tendril = { workspace = true }
log = { workspace = true }
````

## html5ever/src/driver.rs

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/html5ever/src/driver.rs); SHA-256 `760eca0d36fcb90c07858102928aa41efea9adb1eac2df74b972352c2b8c21b3`.

Complete file:

````text
// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! High-level interface to the parser.

use crate::buffer_queue::BufferQueue;
use crate::tokenizer::{Tokenizer, TokenizerOpts};
use crate::tree_builder::{create_element, TreeBuilder, TreeBuilderOpts, TreeSink};
use crate::{Attribute, QualName};
use markup5ever::TokenizerResult;
use std::borrow::Cow;

use crate::tendril;
use crate::tendril::stream::{TendrilSink, Utf8LossyDecoder};
use crate::tendril::StrTendril;

/// All-encompassing options struct for the parser.
#[derive(Clone, Default)]
pub struct ParseOpts {
    /// Tokenizer options.
    pub tokenizer: TokenizerOpts,

    /// Tree builder options.
    pub tree_builder: TreeBuilderOpts,
}

/// Parse an HTML document
///
/// The returned value implements `tendril::TendrilSink`
/// so that Unicode input may be provided incrementally,
/// or all at once with the `one` method.
///
/// If your input is bytes, use `Parser::from_utf8`.
pub fn parse_document<Sink>(sink: Sink, opts: ParseOpts) -> Parser<Sink>
where
    Sink: TreeSink,
{
    let tb = TreeBuilder::new(sink, opts.tree_builder);
    let tok = Tokenizer::new(tb, opts.tokenizer);
    Parser {
        tokenizer: tok,
        input_buffer: BufferQueue::default(),
    }
}

/// Parse an HTML fragment
///
/// The returned value implements `tendril::TendrilSink`
/// so that Unicode input may be provided incrementally,
/// or all at once with the `one` method.
///
/// If your input is bytes, use `Parser::from_utf8`.
pub fn parse_fragment<Sink>(
    sink: Sink,
    opts: ParseOpts,
    context_name: QualName,
    context_attrs: Vec<Attribute>,
    context_element_allows_scripting: bool,
) -> Parser<Sink>
where
    Sink: TreeSink,
{
    let context_elem = create_element(&sink, context_name, context_attrs);
    parse_fragment_for_element(
        sink,
        opts,
        context_elem,
        context_element_allows_scripting,
        None,
    )
}

/// Like `parse_fragment`, but with an existing context element
/// and optionally a form element.
pub fn parse_fragment_for_element<Sink>(
    sink: Sink,
    opts: ParseOpts,
    context_element: Sink::Handle,
    context_element_allows_scripting: bool,
    form_element: Option<Sink::Handle>,
) -> Parser<Sink>
where
    Sink: TreeSink,
{
    let tree_builder =
        TreeBuilder::new_for_fragment(sink, context_element, form_element, opts.tree_builder);
    let tokenizer_options = TokenizerOpts {
        initial_state: Some(
            tree_builder.tokenizer_state_for_context_elem(context_element_allows_scripting),
        ),
        ..opts.tokenizer
    };
    let tokenizer = Tokenizer::new(tree_builder, tokenizer_options);
    Parser {
        tokenizer,
        input_buffer: BufferQueue::default(),
    }
}

/// An HTML parser,
/// ready to receive Unicode input through the `tendril::TendrilSink` trait’s methods.
pub struct Parser<Sink>
where
    Sink: TreeSink,
{
    pub tokenizer: Tokenizer<TreeBuilder<Sink::Handle, Sink>>,
    pub input_buffer: BufferQueue,
}

impl<Sink: TreeSink> TendrilSink<tendril::fmt::UTF8> for Parser<Sink> {
    fn process(&mut self, t: StrTendril) {
        self.input_buffer.push_back(t);
        self.loop_until_done();
    }

    // FIXME: Is it too noisy to report every character decoding error?
    fn error(&mut self, desc: Cow<'static, str>) {
        self.tokenizer.sink.sink.parse_error(desc)
    }

    type Output = Sink::Output;

    fn finish(self) -> Self::Output {
        self.loop_until_done();

        assert!(
            self.input_buffer.is_empty(),
            "parser finished with remaining input"
        );
        self.tokenizer.end();
        self.tokenizer.sink.sink.finish()
    }
}

impl<Sink: TreeSink> Parser<Sink> {
    /// Wrap this parser into a `TendrilSink` that accepts UTF-8 bytes.
    ///
    /// Use this when your input is bytes that are known to be in the UTF-8 encoding.
    /// Decoding is lossy, like `String::from_utf8_lossy`.
    #[allow(clippy::wrong_self_convention)]
    pub fn from_utf8(self) -> Utf8LossyDecoder<Self> {
        Utf8LossyDecoder::new(self)
    }

    fn loop_until_done(&self) {
        // FIXME: Properly support </script> and encoding indicators somehow.
        loop {
            if matches!(
                self.tokenizer.feed(&self.input_buffer),
                TokenizerResult::Done
            ) {
                break;
            }
        }
    }
}
````

## markup5ever/interface/tree_builder.rs

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/markup5ever/interface/tree_builder.rs); SHA-256 `139026eb80fe1c063592509104125bed1cae024e85c98be5c613b9a5ce528ab1`.

Complete file:

````text
// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains functionality for managing the DOM, including adding/removing nodes.
//!
//! It can be used by a parser to create the DOM graph structure in memory.

use crate::interface::{Attribute, ExpandedName, QualName};
use std::borrow::Cow;
use std::fmt::Debug;
use tendril::StrTendril;
use web_atoms::{LocalName, Namespace};

pub use self::NodeOrText::{AppendNode, AppendText};
pub use self::QuirksMode::{LimitedQuirks, NoQuirks, Quirks};

/// Something which can be inserted into the DOM.
///
/// Adjacent sibling text nodes are merged into a single node, so
/// the sink may not want to allocate a `Handle` for each.
pub enum NodeOrText<Handle> {
    AppendNode(Handle),
    AppendText(StrTendril),
}

/// A document's quirks mode, for compatibility with old browsers. See [quirks mode on wikipedia]
/// for more information.
///
/// [quirks mode on wikipedia]: https://en.wikipedia.org/wiki/Quirks_mode
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum QuirksMode {
    /// Full quirks mode
    Quirks,
    /// Almost standards mode
    LimitedQuirks,
    /// Standards mode
    NoQuirks,
}

/// Special properties of an element, useful for tagging elements with this information.
#[derive(Default)]
#[non_exhaustive]
pub struct ElementFlags {
    /// A document fragment should be created, associated with the element,
    /// and returned in TreeSink::get_template_contents.
    ///
    /// See [template-contents in the whatwg spec][whatwg template-contents].
    ///
    /// [whatwg template-contents]: https://html.spec.whatwg.org/multipage/#template-contents
    pub template: bool,

    /// This boolean should be recorded with the element and returned
    /// in TreeSink::is_mathml_annotation_xml_integration_point
    ///
    /// See [html-integration-point in the whatwg spec][whatwg integration-point].
    ///
    /// [whatwg integration-point]: https://html.spec.whatwg.org/multipage/#html-integration-point
    pub mathml_annotation_xml_integration_point: bool,

    /// Whether duplicate attributes were encountered during tokenization.
    /// This is used for CSP nonce validation - elements with duplicate
    /// attributes are not nonceable per the CSP spec.
    ///
    /// See [CSP Level 3 - Is element nonceable](https://www.w3.org/TR/CSP/#is-element-nonceable)
    pub had_duplicate_attributes: bool,
}

/// A constructor for an element.
///
/// # Examples
///
/// Create an element like `<div class="test-class-name"></div>`:
pub fn create_element<Sink>(sink: &Sink, name: QualName, attrs: Vec<Attribute>) -> Sink::Handle
where
    Sink: TreeSink,
{
    create_element_with_flags(sink, name, attrs, false)
}

/// A constructor for an element with duplicate attribute information.
///
/// This variant allows passing whether duplicate attributes were encountered
/// during tokenization, which is needed for CSP nonce validation.
pub fn create_element_with_flags<Sink>(
    sink: &Sink,
    name: QualName,
    attrs: Vec<Attribute>,
    had_duplicate_attributes: bool,
) -> Sink::Handle
where
    Sink: TreeSink,
{
    let mut flags = ElementFlags::default();
    match name.expanded() {
        expanded_name!(html "template") => flags.template = true,
        expanded_name!(mathml "annotation-xml") => {
            flags.mathml_annotation_xml_integration_point = attrs.iter().any(|attr| {
                attr.name.expanded() == expanded_name!("", "encoding")
                    && (attr.value.eq_ignore_ascii_case("text/html")
                        || attr.value.eq_ignore_ascii_case("application/xhtml+xml"))
            })
        },
        _ => {},
    }
    flags.had_duplicate_attributes = had_duplicate_attributes;
    sink.create_element(name, attrs, flags)
}

/// An abstraction over any type that can represent an element's local name and namespace.
pub trait ElemName: Debug {
    fn ns(&self) -> &Namespace;
    fn local_name(&self) -> &LocalName;

    #[inline(always)]
    fn expanded(&self) -> ExpandedName<'_> {
        ExpandedName {
            ns: self.ns(),
            local: self.local_name(),
        }
    }
}

/// Methods a parser can use to create the DOM. The DOM provider implements this trait.
///
/// Having this as a trait potentially allows multiple implementations of the DOM to be used with
/// the same parser.
pub trait TreeSink {
    /// `Handle` is a reference to a DOM node.  The tree builder requires
    /// that a `Handle` implements `Clone` to get another reference to
    /// the same node.
    type Handle: Clone;

    /// The overall result of parsing.
    ///
    /// This should default to Self, but default associated types are not stable yet.
    /// [rust-lang/rust#29661](https://github.com/rust-lang/rust/issues/29661)
    type Output;

    //
    type ElemName<'a>: ElemName
    where
        Self: 'a;

    /// Consume this sink and return the overall result of parsing.
    ///
    /// TODO:This should default to `fn finish(self) -> Self::Output { self }`,
    /// but default associated types are not stable yet.
    /// [rust-lang/rust#29661](https://github.com/rust-lang/rust/issues/29661)
    fn finish(self) -> Self::Output;

    /// Signal a parse error.
    fn parse_error(&self, msg: Cow<'static, str>);

    /// Get a handle to the `Document` node.
    fn get_document(&self) -> Self::Handle;

    /// What is the name of this element?
    ///
    /// Should never be called on a non-element node;
    /// feel free to `panic!`.
    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a>;

    /// Create an element.
    ///
    /// When creating a template element (`name.ns.expanded() == expanded_name!(html "template")`),
    /// an associated document fragment called the "template contents" should
    /// also be created. Later calls to self.get_template_contents() with that
    /// given element return it.
    /// See [the template element in the whatwg spec][whatwg template].
    ///
    /// [whatwg template]: https://html.spec.whatwg.org/multipage/#the-template-element
    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle;

    /// Create a comment node.
    fn create_comment(&self, text: StrTendril) -> Self::Handle;

    /// Create a Processing Instruction node.
    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle;

    /// Append a node as the last child of the given node.  If this would
    /// produce adjacent sibling text nodes, it should concatenate the text
    /// instead.
    ///
    /// The child node will not already have a parent.
    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>);

    /// When the insertion point is decided by the existence of a parent node of the
    /// element, we consider both possibilities and send the element which will be used
    /// if a parent node exists, along with the element to be used if there isn't one.
    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    );

    /// Append a `DOCTYPE` element to the `Document` node.
    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    );

    /// Mark a HTML `<script>` as "already started".
    fn mark_script_already_started(&self, _node: &Self::Handle) {}

    /// Indicate that a node was popped off the stack of open elements.
    fn pop(&self, _node: &Self::Handle) {}

    /// Get a handle to a template's template contents. The tree builder
    /// promises this will never be called with something else than
    /// a template element.
    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle;

    /// Do two handles refer to the same node?
    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool;

    /// Set the document's quirks mode.
    fn set_quirks_mode(&self, mode: QuirksMode);

    /// Append a node as the sibling immediately before the given node.
    ///
    /// The tree builder promises that `sibling` is not a text node.  However its
    /// old previous sibling, which would become the new node's previous sibling,
    /// could be a text node.  If the new node is also a text node, the two should
    /// be merged, as in the behavior of `append`.
    ///
    /// NB: `new_node` may have an old parent, from which it should be removed.
    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>);

    /// Add each attribute to the given element, if no attribute with that name
    /// already exists. The tree builder promises this will never be called
    /// with something else than an element.
    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>);

    /// Associate the given form-associatable element with the form element
    fn associate_with_form(
        &self,
        _target: &Self::Handle,
        _form: &Self::Handle,
        _nodes: (&Self::Handle, Option<&Self::Handle>),
    ) {
    }

    /// Detach the given node from its parent.
    fn remove_from_parent(&self, target: &Self::Handle);

    /// Remove all the children from node and append them to new_parent.
    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle);

    /// Returns true if the adjusted current node is an HTML integration point
    /// and the token is a start tag.
    fn is_mathml_annotation_xml_integration_point(&self, _handle: &Self::Handle) -> bool {
        false
    }

    /// Called whenever the line number changes.
    fn set_current_line(&self, _line_number: u64) {}

    fn allow_declarative_shadow_roots(&self, _intended_parent: &Self::Handle) -> bool {
        true
    }

    /// Attempt to attach a declarative shadow root at the given location.
    ///
    /// Returns a boolean indicating whether the operation succeeded or not.
    fn attach_declarative_shadow(
        &self,
        _location: &Self::Handle,
        _template: &Self::Handle,
        _attrs: &[Attribute],
    ) -> bool {
        false
    }

    /// Implements [`maybe clone an option into selectedcontent`](https://html.spec.whatwg.org/#maybe-clone-an-option-into-selectedcontent).
    ///
    /// The provided handle is guaranteed to be an `<option>` element.
    ///
    /// Leaving this method unimplemented will not cause panics, but will result in a (slightly) incorrect DOM tree.
    ///
    /// This method will never be called from `xml5ever`.
    fn maybe_clone_an_option_into_selectedcontent(&self, option: &Self::Handle) {
        _ = option;
    }
}

/// Trace hooks for a garbage-collected DOM.
pub trait Tracer {
    type Handle;

    /// Upon a call to `trace_handles`, the tree builder will call this method
    /// for each handle in its internal state.
    fn trace_handle(&self, node: &Self::Handle);
}
````

## html5ever/src/tokenizer/interface.rs

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/html5ever/src/tokenizer/interface.rs); SHA-256 `cdab80709a6e27663b9a349a607cf85da93fcc9ee3b3a54cf97ee635a9ae7cf4`.

Complete file:

````text
// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use markup5ever::ns;

use crate::interface::Attribute;
use crate::tendril::StrTendril;
use crate::tokenizer::states;
use crate::LocalName;
use std::borrow::Cow;

pub use self::TagKind::{EndTag, StartTag};
pub use self::Token::{CharacterTokens, CommentToken, DoctypeToken, TagToken};
pub use self::Token::{EOFToken, NullCharacterToken, ParseError};

/// A `DOCTYPE` token.
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct Doctype {
    pub name: Option<StrTendril>,
    pub public_id: Option<StrTendril>,
    pub system_id: Option<StrTendril>,
    /// Indicates if this DOCTYPE token should put the document in [quirks mode].
    ///
    /// [quirks mode]: https://dom.spec.whatwg.org/#concept-document-quirks
    pub force_quirks: bool,
}

/// Whether the tag is a start or an end tag.
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum TagKind {
    StartTag,
    EndTag,
}

/// A tag token.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Tag {
    /// Whether the tag is a start or an end tag.
    pub kind: TagKind,
    pub name: LocalName,
    /// Whether the tag closes itself.
    ///
    /// An example of a self closing tag is `<foo />`.
    pub self_closing: bool,
    pub attrs: Vec<Attribute>,
    /// Whether duplicate attributes were encountered during tokenization.
    /// This is used for CSP nonce validation - elements with duplicate
    /// attributes are not nonceable per the CSP spec.
    pub had_duplicate_attributes: bool,
}

impl Tag {
    /// Are the tags equivalent when we don't care about attribute order?
    /// Also ignores the self-closing flag.
    pub fn equiv_modulo_attr_order(&self, other: &Tag) -> bool {
        if (self.kind != other.kind) || (self.name != other.name) {
            return false;
        }

        let mut self_attrs = self.attrs.clone();
        let mut other_attrs = other.attrs.clone();
        self_attrs.sort();
        other_attrs.sort();

        self_attrs == other_attrs
    }

    pub(crate) fn get_attribute(&self, name: &LocalName) -> Option<StrTendril> {
        self.attrs
            .iter()
            .find(|attribute| attribute.name.ns == *ns!() && attribute.name.local == *name)
            .map(|attribute| attribute.value.clone())
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    /// A DOCTYPE declaration like `<!DOCTYPE html>`
    DoctypeToken(Doctype),
    /// A opening or closing tag, like `<foo>` or `</bar>`
    TagToken(Tag),
    /// A comment like `<!-- foo -->`.
    CommentToken(StrTendril),
    /// A sequence of characters.
    CharacterTokens(StrTendril),
    /// A `U+0000 NULL` character in the input.
    NullCharacterToken,
    EOFToken,
    ParseError(Cow<'static, str>),
}

/// The result of a [TokenSink] consuming a single token.
#[derive(Debug, PartialEq)]
#[must_use]
pub enum TokenSinkResult<Handle> {
    /// The tokenizer can continue parsing the input as usual.
    Continue,
    /// The token sink has completed parsing a `<script>` tag, blocking the tokenizer
    /// until the script is executed.
    Script(Handle),
    /// The tokenizer should set its state to the [PLAINTEXT state](https://html.spec.whatwg.org/#plaintext-state).
    Plaintext,
    /// The tokenizer should set its state to the given rawdata state.
    RawData(states::RawKind),
    /// The document indicated that the given encoding should be used to parse it.
    ///
    /// HTML5-compatible implementations should parse the encoding label using the algorithm
    /// described in <https://encoding.spec.whatwg.org/#concept-encoding-get>. The label
    /// has not been validated by html5ever. Invalid or unknown encodings can be ignored.
    ///
    /// If the decoder is confident that the current encoding is correct then this message
    /// can safely be ignored.
    EncodingIndicator(StrTendril),
}

/// Types which can receive tokens from the tokenizer.
pub trait TokenSink {
    /// The type of a DOM node.
    type Handle;

    /// Process a token.
    fn process_token(&self, token: Token, line_number: u64) -> TokenSinkResult<Self::Handle>;

    /// Signal that tokenization reached the end of the document.
    fn end(&self) {}

    /// Used in the [markup declaration open state]. By default, this always
    /// returns false and thus all CDATA sections are tokenized as bogus
    /// comments.
    ///
    /// [markup declaration open state]: https://html.spec.whatwg.org/multipage/#markup-declaration-open-state
    fn adjusted_current_node_present_but_not_in_html_namespace(&self) -> bool {
        false
    }
}
````

## rcdom/README.md

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/rcdom/README.md); SHA-256 `ef9d91fbc81cf9ed91fa5506f27c68e70b4e9972fda1fb583050ede646baa6c6`.

Complete file:

````text
# markup5ever_rcdom

This crate is built for the express purpose of writing automated tests for the `html5ever`
and `xml5ever` crates. It is not intended to be a production-quality DOM implementation,
and has not been fuzzed or tested against arbitrary, malicious, or nontrivial inputs. No maintenance
or support for any such issues will be provided. If you use this DOM implementation in a production,
user-facing system, you do so at your own risk.
````

## tendril/README.md

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/tendril/README.md); SHA-256 `4e9d7ae86776bbab2d7512272e883e78b3361b7f623fae02d65eb7390cf0ed06`.

Complete file:

````text
# tendril

**Warning**: This library is at a very early stage of development, and it
contains a substantial amount of `unsafe` code. Use at your own risk!

[![Build Status](https://github.com/servo/html5ever/actions/workflows/main.yml/badge.svg)](https://github.com/servo/html5ever/actions)

[API Documentation](https://docs.rs/tendril)

## Introduction

`Tendril` is a compact string/buffer type, optimized for zero-copy parsing.
Tendrils have the semantics of owned strings, but are sometimes views into
shared buffers. When you mutate a tendril, an owned copy is made if necessary.
Further mutations occur in-place until the string becomes shared, e.g. with
`clone()` or `subtendril()`.

Buffer sharing is accomplished through thread-local (non-atomic) reference
counting, which has very low overhead. The Rust type system will prevent you at
compile time from sending a tendril between threads. (See below for thoughts on
relaxing this restriction.)

Whereas `String` allocates in the heap for any non-empty string, `Tendril` can
store small strings (up to 8 bytes) in-line, without a heap allocation.
`Tendril` is also smaller than `String` on 64-bit platforms — 16 bytes versus
24. `Option<Tendril>` is the same size as `Tendril`, thanks to
[`NonZero`][NonZero].

The maximum length of a tendril is 4 GB. The library will panic if you attempt
to go over the limit.

## Formats and encoding

`Tendril` uses
[phantom types](https://doc.rust-lang.org/stable/rust-by-example/generics/phantom.html)
to track a buffer's format. This determines at compile time which operations are
available on a given tendril. For example, `Tendril<UTF8>` and `Tendril<Bytes>`
can be borrowed as `&str` and `&[u8]` respectively.

`Tendril` also integrates with
[encoding_rs](https://github.com/hsivonen/encoding_rs) and has preliminary
support for [WTF-8][] buffers.

## Plans for the future

### Ropes

[html5ever][] will use `Tendril` as a zero-copy text representation. It would be
good to preserve this all the way through to Servo's DOM. This would reduce
memory consumption, and possibly speed up text shaping and painting. However,
DOM text may conceivably be larger than 4 GB, and will anyway not be contiguous
in memory around e.g. a character entity reference.

*Solution:* Build a **[rope][] on top of these strings** and use that as Servo's
representation of DOM text. We can perhaps do text shaping and/or painting in
parallel for different chunks of a rope. html5ever can additionally use this
rope type as a replacement for `BufferQueue`.

Because the underlying buffers are reference-counted, the bulk of this rope is
already a [persistent data structure][]. Consider what happens when appending
two ropes to get a "new" rope. A vector-backed rope would copy a vector of small
structs, one for each chunk, and would bump the corresponding refcounts. But it
would not copy any of the string data.

If we want more sharing, then a [2-3 finger tree][] could be a good choice. We
would probably stick with `VecDeque` for ropes under a certain size.

### UTF-16 compatibility

SpiderMonkey expects text to be in UCS-2 format for the most part. The semantics
of JavaScript strings are difficult to implement on UTF-8. This also applies to
HTML parsing via `document.write`. Also, passing SpiderMonkey a string that
isn't contiguous in memory will incur additional overhead and complexity, if not
a full copy.

*Solution:* Use **WTF-8 in parsing** and in the DOM. Servo will **convert to
contiguous UTF-16 when necessary**.  The conversion can easily be parallelized,
if we find a practical need to convert huge chunks of text all at once.

### Source span information

Some html5ever API consumers want to know the originating location in the HTML
source file(s) of each token or parse error. An example application would be a
command-line HTML validator with diagnostic output similar to `rustc`'s.

*Solution:* Accept **some metadata along with each input string**. The type of
metadata is chosen by the API consumer; it defaults to `()`, which has size
zero. For any non-inline string, we can provide the associated metadata as well
as a byte offset.

[NonZero]: https://doc.rust-lang.org/core/nonzero/struct.NonZero.html
[html5ever]: https://github.com/servo/html5ever
[WTF-8]: https://simonsapin.github.io/wtf-8/
[rope]: https://en.wikipedia.org/wiki/Rope_%28data_structure%29
[persistent data structure]: https://en.wikipedia.org/wiki/Persistent_data_structure
[2-3 finger tree]: https://www.staff.city.ac.uk/~ross/papers/FingerTree.html
````

## html5ever/src/tokenizer/mod.rs

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/html5ever/src/tokenizer/mod.rs); SHA-256 `387fef55aea53cf08c2b45192ab8f9f211e993e9dc76ec5031422777bf06773f`.

Original lines 1–9:

````text
// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

````

Original lines 70–110:

````text

/// Tokenizer options, with an impl for `Default`.
#[derive(Clone)]
pub struct TokenizerOpts {
    /// Report all parse errors described in the spec, at some
    /// performance penalty?  Default: false
    pub exact_errors: bool,

    /// Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning
    /// of the stream?  Default: true
    pub discard_bom: bool,

    /// Keep a record of how long we spent in each state?  Printed
    /// when `end()` is called.  Default: false
    pub profile: bool,

    /// Initial state override.  Only the test runner should use
    /// a non-`None` value!
    pub initial_state: Option<states::State>,

    /// Last start tag.  Only the test runner should use a
    /// non-`None` value!
    ///
    /// FIXME: Can't use Tendril because we want TokenizerOpts
    /// to be Send.
    pub last_start_tag_name: Option<String>,
}

impl Default for TokenizerOpts {
    fn default() -> TokenizerOpts {
        TokenizerOpts {
            exact_errors: false,
            discard_bom: true,
            profile: false,
            initial_state: None,
            last_start_tag_name: None,
        }
    }
}

/// The HTML tokenizer.
````

Original lines 530–575:

````text
            None => false,
        }
    }

    fn create_attribute(&self, c: char) {
        self.finish_attribute();

        self.current_attr_name.borrow_mut().push_char(c);
    }

    fn finish_attribute(&self) {
        if self.current_attr_name.borrow().is_empty() {
            return;
        }
        let name = LocalName::from(&**self.current_attr_name.borrow());
        self.current_attr_name.borrow_mut().clear();
        // Check for a duplicate attribute.
        // FIXME: the spec says we should error as soon as the name is finished.
        let dup = {
            self.current_tag_attrs
                .borrow()
                .iter()
                .any(|a| a.name.local == name)
        };

        if dup {
            self.emit_error(Borrowed("Duplicate attribute"));
            self.current_tag_had_duplicate_attributes.set(true);
            self.current_attr_value.borrow_mut().clear();
        } else {
            self.current_tag_attrs.borrow_mut().push(Attribute {
                // The tree builder will adjust the namespace if necessary.
                // This only happens in foreign elements.
                name: QualName::new(None, ns!(), name),
                value: mem::take(&mut self.current_attr_value.borrow_mut()),
            });
        }
    }

    fn emit_current_doctype(&self) {
        let doctype = self.current_doctype.take();
        self.process_token_and_continue(DoctypeToken(doctype));
    }

    fn doctype_id(&self, kind: DoctypeIdKind) -> RefMut<'_, Option<StrTendril>> {
        let current_doctype = self.current_doctype.borrow_mut();
````

Original lines 735–765:

````text
                    || self.ignore_lf.get())
                    && Self::is_supported_simd_feature_detected()
                {
                    let front_buffer = input.peek_front_chunk_mut();
                    let Some(mut front_buffer) = front_buffer else {
                        return ProcessResult::Suspend;
                    };

                    // Special case: The fast path is not worth taking if the first character is already in the set,
                    // which is fairly common
                    let first_char = front_buffer
                        .chars()
                        .next()
                        .expect("Input buffers are never empty");

                    if matches!(first_char, '\r' | '\0' | '&' | '<' | '\n') {
                        drop(front_buffer);
                        self.pop_except_from(input, set)
                    } else {
                        // SAFETY:
                        // This CPU is guaranteed to support SIMD due to the is_supported_simd_feature_detected check above
                        let result = unsafe { self.data_state_simd_fast_path(&mut front_buffer) };

                        if front_buffer.is_empty() {
                            drop(front_buffer);
                            input.pop_front();
                        }

                        result
                    }
                } else {
````


## html5ever/src/tree_builder/mod.rs

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/html5ever/src/tree_builder/mod.rs); SHA-256 `d69954162f05710cb62fb660666956ec5daa0878e26ba5df9f25db56d3284a36`.

Original lines 1–9:

````text
// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

````

Original lines 45–90:

````text
/// Tree builder options, with an impl for Default.
#[derive(Copy, Clone)]
pub struct TreeBuilderOpts {
    /// Report all parse errors described in the spec, at some
    /// performance penalty? Default: false
    pub exact_errors: bool,

    /// Is scripting enabled?
    ///
    /// This affects how `<noscript>` elements are parsed:
    ///   - If scripting **is** enabled then the contents of a `<noscript>` element are parsed as a single text node
    ///   - If scriping is **not** enabled then the contents of a `<noscript>` element are parsed as a normal tree of nodes
    pub scripting_enabled: bool,

    /// Is this document being parsed from the `srcdoc` attribute of an `<iframe>` element?
    ///
    /// This affects heuristics that infer `QuirksMode` from `<!DOCTYPE>`.
    pub iframe_srcdoc: bool,

    /// Should we drop the DOCTYPE (if any) from the tree?
    pub drop_doctype: bool,

    /// Initial TreeBuilder quirks mode. Default: NoQuirks
    pub quirks_mode: QuirksMode,
}

impl Default for TreeBuilderOpts {
    fn default() -> TreeBuilderOpts {
        TreeBuilderOpts {
            exact_errors: false,
            scripting_enabled: true,
            iframe_srcdoc: false,
            drop_doctype: false,
            quirks_mode: NoQuirks,
        }
    }
}

/// The HTML tree builder.
pub struct TreeBuilder<Handle, Sink> {
    /// Options controlling the behavior of the tree builder.
    opts: TreeBuilderOpts,

    /// Consumer of tree modifications.
    pub sink: Sink,

````

Original lines 1080–1110:

````text
            return;
        }
    }

    fn in_scope<TagSet, Pred>(&self, scope: TagSet, pred: Pred) -> bool
    where
        TagSet: Fn(ExpandedName) -> bool,
        Pred: Fn(Handle) -> bool,
    {
        for node in self.open_elems.borrow().iter().rev() {
            if pred(node.clone()) {
                return true;
            }
            if scope(self.sink.elem_name(node).expanded()) {
                return false;
            }
        }

        // supposed to be impossible, because <html> is always in scope

        false
    }

    fn elem_in<TagSet>(&self, elem: &Handle, set: TagSet) -> bool
    where
        TagSet: Fn(ExpandedName) -> bool,
    {
        set(self.sink.elem_name(elem).expanded())
    }

    fn html_elem_named(&self, elem: &Handle, name: LocalName) -> bool {
````
