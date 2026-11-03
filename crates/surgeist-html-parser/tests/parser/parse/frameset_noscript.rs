//! Fragment-case `</frameset>` and the scripting-mode `noscript` rules.
//!
//! Every expected tree below is derived by hand from the pinned HTML Standard
//! (revision 877d6146487f, `references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md`),
//! never from parser output. Capture line numbers are 1-based.
//!
//! * "in frameset", `</frameset>` (capture line 5652): pop the current node,
//!   "If the parser's fragment context element is null and the current node is
//!   no longer a frameset element, then switch the insertion mode to 'after
//!   frameset'." With a non-null context element the mode stays "in frameset".
//!   Line 5648: a current node that is the root `html` element is a fragment
//!   case, and the token is ignored.
//! * "after body", `</html>` (capture line 5604): "If the parser's fragment
//!   context element is non-null, this is a parse error; ignore the token."
//! * Parser scripting mode (capture lines 1347-1370, 6786): Normal, Disabled,
//!   Inert or Fragment. The fragment parsing algorithm takes Inert or Fragment;
//!   a context document without scripting forces Disabled (the request layer
//!   records that resolution in `tests/parser/requests.rs`).
//! * "in head" (capture line 3926) and "in body" (capture line 4738): "A start
//!   tag whose tag name is "noscript", if scripting mode is not Disabled:
//!   follow the generic raw text element parsing algorithm." That algorithm
//!   (insert the element, tokenizer to RAWTEXT, switch to "text") makes the
//!   element's content one text node; the "text" mode's end tag pops it and
//!   restores the original insertion mode.
//! * "in table", anything else (capture line 5182): enable foster parenting,
//!   process the token using the "in body" rules, disable foster parenting.
//! * "in template", any other start tag (capture line 5552): replace the
//!   current template insertion mode with "in body" and reprocess the token.
//! * Fragment parsing algorithm, tokenizer state for a `noscript` context
//!   (capture line 6837): RAWTEXT when the scripting mode is not Disabled,
//!   otherwise the data state. Step 19 (capture line 6860) makes the context's
//!   start tag only the context's token for integration-point checks, so no
//!   end tag is "appropriate" (capture line 1391) and `</noscript>` is text.
//! * "in head", `noscript` start tag with scripting Disabled (capture line
//!   3932): insert the element and switch to "in head noscript", where a start
//!   tag other than basefont, bgsound, link, meta, noframes or style is a parse
//!   error that pops the `noscript` and reprocesses it in "in head".

use crate::parser::support::{fragment_context_for, limits};
use surgeist_html::{
    construction::{HtmlNode, HtmlNodeId, HtmlNodeKind},
    parsing::HtmlParserScriptingMode,
};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::HtmlScriptingStatus;

fn render<'a>(
    lookup: &dyn Fn(&HtmlNodeId) -> HtmlNode<'a>,
    parent: HtmlNode<'a>,
    depth: usize,
    out: &mut String,
) {
    let pad = "  ".repeat(depth);
    for id in parent.children() {
        let node = lookup(&id);
        match node.kind() {
            HtmlNodeKind::Element(element) => {
                let name = String::from_utf16(element.name().local_name().code_units()).unwrap();
                out.push_str(&format!("{pad}<{name}>\n"));
            }
            HtmlNodeKind::Text(text) => {
                let text = String::from_utf16(text.code_units()).unwrap();
                out.push_str(&format!("{pad}\"{text}\"\n"));
            }
            HtmlNodeKind::Comment(text) => {
                let text = String::from_utf16(text.code_units()).unwrap();
                out.push_str(&format!("{pad}<!--{text}-->\n"));
            }
            other => panic!("unexpected node kind {other:?}"),
        }
        render(lookup, node, depth + 1, out);
    }
}

/// The fragment's children rendered as an indented tree, for both encodings.
fn fragment_trees(context_local: &str, scripting: HtmlScriptingStatus, input: &str) -> [String; 2] {
    [
        HtmlSource::utf8(input),
        HtmlSource::utf16(surgeist_html::values::DocumentString::from(input)),
    ]
    .map(|source| {
        let request = HtmlFragmentParseRequest::with_options(
            source,
            fragment_context_for(context_local, scripting),
            limits(),
            HtmlParserScriptingMode::Inert,
            false,
        )
        .unwrap();
        let report: HtmlParseReport = request
            .parse()
            .unwrap_or_else(|failure| panic!("fragment must parse: {:?}", failure.reason()));
        let fragment = report.fragment().unwrap();
        let mut out = String::new();
        render(
            &|id| fragment.node(id).unwrap(),
            fragment.root(),
            0,
            &mut out,
        );
        out
    })
}

fn assert_fragment(
    context_local: &str,
    scripting: HtmlScriptingStatus,
    input: &str,
    expected: &str,
) {
    for (encoding, actual) in
        ["utf8", "utf16"]
            .into_iter()
            .zip(fragment_trees(context_local, scripting, input))
    {
        assert_eq!(
            actual, expected,
            "context {context_local:?} {scripting:?} input {input:?} ({encoding})"
        );
    }
}

fn document_tree(input: &str) -> String {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(input), limits())
        .parse()
        .unwrap_or_else(|failure| panic!("document must parse: {:?}", failure.reason()));
    let document = report.document().unwrap();
    let mut out = String::new();
    render(
        &|id| document.node(id).unwrap(),
        document.root(),
        0,
        &mut out,
    );
    out
}

#[test]
fn frameset_context_keeps_in_frameset_after_the_end_tag_pops_to_html() {
    // Stack [html]; mode "in frameset". `<frameset>` is inserted and pushed;
    // `</frameset>` pops it and the current node is `html`. The fragment case
    // keeps the mode, so `<frame>` is inserted as a second child of `html`.
    assert_fragment(
        "frameset",
        HtmlScriptingStatus::Disabled,
        "<frameset></frameset><frame>",
        "<frameset>\n<frame>\n",
    );
}

#[test]
fn frameset_context_keeps_a_trailing_frame_after_a_leading_frame_and_frameset() {
    assert_fragment(
        "frameset",
        HtmlScriptingStatus::Disabled,
        "<frame><frameset></frameset><frame>",
        "<frame>\n<frameset>\n<frame>\n",
    );
}

#[test]
fn frameset_context_pops_nested_framesets_and_stays_in_frameset() {
    // The inner `</frameset>` leaves a `frameset` as the current node (no mode
    // change in any case); the outer one leaves `html`, a fragment case.
    assert_fragment(
        "frameset",
        HtmlScriptingStatus::Disabled,
        "<frameset><frameset></frameset></frameset><frame>",
        "<frameset>\n  <frameset>\n<frame>\n",
    );
}

#[test]
fn html_context_frameset_end_tag_keeps_in_frameset() {
    // Context `html` starts at "before head". `<frameset>` implies `head`, is
    // reprocessed through "in head" and "after head", where it inserts the
    // `frameset` and switches to "in frameset". The fragment case then keeps
    // that mode after `</frameset>`, so `<frame>` is inserted under `html`.
    assert_fragment(
        "html",
        HtmlScriptingStatus::Disabled,
        "<frameset></frameset><frame>",
        "<head>\n<frameset>\n<frame>\n",
    );
}

#[test]
fn frameset_context_ignores_an_end_tag_when_the_current_node_is_html() {
    assert_fragment(
        "frameset",
        HtmlScriptingStatus::Disabled,
        "</frameset><frame>",
        "<frame>\n",
    );
}

#[test]
fn document_frameset_end_tag_switches_to_after_frameset() {
    // No fragment context element: after `</frameset>` the current node is
    // `html`, so the mode becomes "after frameset", which ignores `<frame>`.
    assert_eq!(
        document_tree("<frameset></frameset><frame>"),
        "<html>\n  <head>\n  <frameset>\n"
    );
}

#[test]
fn html_context_after_body_html_end_tag_stays_in_after_body() {
    // Context `html`: `<body>` implies `head`, then inserts `body`; `</body>`
    // switches to "after body". The fragment case ignores `</html>`, so the
    // comment is inserted into the first element on the stack, `html`.
    assert_fragment(
        "html",
        HtmlScriptingStatus::Disabled,
        "<body></body></html><!--c-->",
        "<head>\n<body>\n<!--c-->\n",
    );
}

#[test]
fn inert_fragment_in_body_noscript_is_raw_text() {
    // Context `div` ("in body") in an Enabled document: effective mode Inert,
    // which is not Disabled, so `noscript` follows the raw text algorithm and
    // its content is a single text node.
    assert_fragment(
        "div",
        HtmlScriptingStatus::Enabled,
        "<noscript><p>x</p></noscript>",
        "<noscript>\n  \"<p>x</p>\"\n",
    );
}

#[test]
fn inert_fragment_in_body_noscript_raw_text_returns_to_in_body() {
    assert_fragment(
        "div",
        HtmlScriptingStatus::Enabled,
        "<noscript><b></noscript><i>y",
        "<noscript>\n  \"<b>\"\n<i>\n  \"y\"\n",
    );
}

#[test]
fn disabled_fragment_in_body_noscript_is_an_ordinary_element() {
    // The context document has scripting disabled, which forces Disabled; in
    // "in body" a Disabled `noscript` is an ordinary element.
    assert_fragment(
        "div",
        HtmlScriptingStatus::Disabled,
        "<noscript><p>x</p></noscript>",
        "<noscript>\n  <p>\n    \"x\"\n",
    );
}

#[test]
fn inert_fragment_in_head_noscript_is_raw_text() {
    // Context `html` ("before head"): `noscript` implies `head` and is
    // reprocessed in "in head", where a non-Disabled mode selects raw text.
    // End of file in "after head" inserts the `body` (anything else).
    assert_fragment(
        "html",
        HtmlScriptingStatus::Enabled,
        "<noscript><p>x</p></noscript>",
        "<head>\n  <noscript>\n    \"<p>x</p>\"\n<body>\n",
    );
}

#[test]
fn inert_fragment_in_head_noscript_raw_text_returns_to_in_head() {
    // After the raw text element closes the mode is "in head" again, so a
    // `meta` start tag is a head child following the `noscript`.
    assert_fragment(
        "html",
        HtmlScriptingStatus::Enabled,
        "<noscript><p></noscript><meta>",
        "<head>\n  <noscript>\n    \"<p>\"\n  <meta>\n<body>\n",
    );
}

#[test]
fn disabled_fragment_in_head_noscript_uses_the_in_head_noscript_mode() {
    // Disabled: the `noscript` is inserted and "in head noscript" applies. `<p>`
    // pops the `noscript`, leaves the head and starts the body; the stray
    // `</noscript>` is ignored.
    assert_fragment(
        "html",
        HtmlScriptingStatus::Disabled,
        "<noscript><p>x</p></noscript>",
        "<head>\n  <noscript>\n<body>\n  <p>\n    \"x\"\n",
    );
}

#[test]
fn inert_head_noscript_with_a_leading_comment_keeps_the_comment_as_text() {
    // Raw text: markup-looking content, including `<!--`, is character data.
    assert_fragment(
        "html",
        HtmlScriptingStatus::Enabled,
        "<noscript><!--</noscript>",
        "<head>\n  <noscript>\n    \"<!--\"\n<body>\n",
    );
}

// The tests below are characterization checks of behaviour the scripting-mode
// rules reach through other insertion modes. Their trees are hand-derived.

#[test]
fn inert_noscript_in_a_table_is_foster_parented_raw_text() {
    // Context `div`: `<table>` is inserted and "in table" applies. `<noscript>`
    // is "anything else": foster parenting is enabled and the "in body" rule
    // inserts the raw text element before the table (the table's parent is
    // `html`). The element's text is data, the original insertion mode "in
    // table" is restored by `</noscript>`, and `</table>` closes the table.
    assert_fragment(
        "div",
        HtmlScriptingStatus::Enabled,
        "<table><noscript><p>y</p></noscript></table>",
        "<noscript>\n  \"<p>y</p>\"\n<table>\n",
    );
}

#[test]
fn inert_noscript_in_a_table_context_is_raw_text_before_table_rows() {
    // Context `table` ("in table", stack `[html]`): the `noscript` is inserted
    // into `html` with its text, "in table" returns, and `<tr>` clears the
    // stack back to a table context (already `html`), implies `tbody` and
    // inserts `tr`.
    assert_fragment(
        "table",
        HtmlScriptingStatus::Enabled,
        "<noscript>a</noscript><tr>",
        "<noscript>\n  \"a\"\n<tbody>\n  <tr>\n",
    );
}

#[test]
fn disabled_noscript_in_a_table_is_a_foster_parented_ordinary_element() {
    // Disabled: `noscript` is an ordinary element, inserted before the table
    // by foster parenting. It is then the current node, so the paragraph and
    // its text are inserted inside it (a foster-parent target that is not a
    // table part is the current node itself).
    assert_fragment(
        "div",
        HtmlScriptingStatus::Disabled,
        "<table><noscript><p>y</p></noscript></table>",
        "<noscript>\n  <p>\n    \"y\"\n<table>\n",
    );
}

#[test]
fn inert_noscript_in_a_template_context_is_raw_text() {
    // Context `template` starts in "in template" with that template mode. The
    // `noscript` start tag is "any other start tag": the mode becomes "in
    // body" and the token is reprocessed there as raw text.
    assert_fragment(
        "template",
        HtmlScriptingStatus::Enabled,
        "<noscript><p>x</p></noscript><i>z",
        "<noscript>\n  \"<p>x</p>\"\n<i>\n  \"z\"\n",
    );
}

#[test]
fn inert_noscript_context_tokenizes_its_content_as_raw_text() {
    // The tokenizer starts in RAWTEXT, and no start tag has been emitted, so
    // even `</noscript>` is character data; the insertion mode is "in body".
    assert_fragment(
        "noscript",
        HtmlScriptingStatus::Enabled,
        "<p>x</p>",
        "\"<p>x</p>\"\n",
    );
    assert_fragment(
        "noscript",
        HtmlScriptingStatus::Enabled,
        "a</noscript>b",
        "\"a</noscript>b\"\n",
    );
}

#[test]
fn disabled_noscript_context_tokenizes_its_content_as_data() {
    assert_fragment(
        "noscript",
        HtmlScriptingStatus::Disabled,
        "<p>x</p>",
        "<p>\n  \"x\"\n",
    );
}
