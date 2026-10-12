# html5ever reference attribution

Upstream: [servo/html5ever](https://github.com/servo/html5ever), revision `d7232d746d3112f08926d169591576bd12fe2fdf`.

The [source witness](../../references/html5ever-parser-contracts--d7232d746d31.md) retains selected original files or explicitly bounded excerpts for parser research. Fenced Markdown wrapping and provenance headings were added; retained source text was not modified. No parser implementation has been incorporated into Surgeist by this capture.

Copyright and authors are preserved in [COPYRIGHT](COPYRIGHT) and [AUTHORS](AUTHORS). Distributed under the upstream choice of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

The [supplemental parser witness](../../references/html5ever-parser-supplemental-contracts--d7232d746d31.md) adds exact bounded excerpts from `html5ever/src/tokenizer/char_ref/mod.rs`, `html5ever/src/tree_builder/rules.rs` and `markup5ever/util/buffer_queue.rs` at the same revision. Complete-file hashes and original line ranges identify the retained code; each file’s complete original notice remains verbatim. Changes on 2026-10-11 are selection, provenance headings and inert Markdown fencing only. These source references do not themselves incorporate the parser into production. The existing upstream MIT OR Apache-2.0 licensing and full copyright/authorship material apply.

## Incorporated tokenizer source

Separately from those research captures, Surgeist incorporates an adapted,
private tokenizer compiled for tests in `crates/surgeist-html/src/parser/tokenizer/`.
The [origin record](../../crates/surgeist-html/PARSER-ORIGINS.md) maps four original
source files to five adapted implementation files and records their
modifications on 2026-10-11. Exact original copyright and license headers remain
in those files; the copyright, authorship and MIT OR Apache-2.0 material above
applies to the adaptation. No html5ever Cargo package is added by this ingestion.

## Incorporated tree-insertion source

The same selected revision supplies a bounded adaptation of tree-builder
insertion-location and stack/scope operations plus selected tag sets, followed
by the ordinary-document Initial/BeforeHtml prologue and doctype tables, then
the owning BeforeHead/ordinary InHead/Disabled-noscript/Text driver, including
script insertion and EOF/end-tag dispatch adapted for the fresh Disabled operation,
then selected ordinary body/trailing-mode rows, generic end handling and
implied-end/special-element lists, then active formatting, Noah comparison,
reconstruction, adoption and applet/marquee/object marker handling, followed by
the listed non-resettable button/fieldset/object rows and ordinary frameset
entry, InFrameset and its trailing modes, then ordinary textarea entry from
`rules.rs` lines 874–878, followed by the input/hr select guards and ordinary
select/option/optgroup rows in `rules.rs` lines 815–862 and 895–945, then ordinary
non-script MathML/SVG entry from `rules.rs` lines 971–979, foreign dispatch and
recovery from `rules.rs` lines 1608–1688, and integration/name/attribute/breakout
machinery from `mod.rs` lines 1605–1692, 1694–1841 and 1837–1883. The new private
`src/parser/tree_builder/foreign.rs` and
`src/construction/parser_session/foreign.rs` retain the complete original notice.
HTML-qualified PI, annotation-xml and exact UTF-16 differences, plus controlled
backing/registry/qualified-clone ownership, are identified in the origin record.
Current end-select/selectedcontent dispatch and controlled intrinsic
selection/cloning/checkpoint machinery are Surgeist-authored adaptations of the
selected HTML/DOM clauses, distinguished from the upstream excerpts. The
[origin record](../../crates/surgeist-html/PARSER-ORIGINS.md) identifies four
complete original file hashes and separate initial adapted checkpoints.
The original eight-line copyright/license notices are retained verbatim;
adaptation notes identify the changed source and selected HTML authority.
The exact copyright, authorship and MIT OR Apache-2.0 material above applies.
This test-compiled kernel uses Surgeist-owned controlled storage. It does not
claim complete insertion modes or public parser execution.
