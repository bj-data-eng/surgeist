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
by the ordinary-document Initial/BeforeHtml prologue and doctype tables. The
[origin record](../../crates/surgeist-html/PARSER-ORIGINS.md) identifies four
complete original file hashes and separate initial adapted checkpoints.
The original eight-line copyright/license notices are retained verbatim;
adaptation notes identify the changed source and selected HTML authority.
The exact copyright, authorship and MIT OR Apache-2.0 material above applies.
This test-compiled kernel uses Surgeist-owned controlled storage. It does not
claim complete insertion modes or public parser execution.
