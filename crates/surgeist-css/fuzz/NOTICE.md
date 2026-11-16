# CSS fuzz attribution

This package is repository-owned MIT verification source. It consumes the
existing CSS implementation and embeds no upstream parser, specification text
or acquired corpus.

The files under [seeds/csstree](seeds/csstree) are inputs copied unchanged from
the CSS-owned [CSSTree corpus](../tests/corpus/csstree/README.md), which adopts
fixtures from [CSSTree](https://github.com/csstree/csstree) at commit
`88e3d965c0b1628642a30a841745b410d6835052`. CSSTree is distributed under the
MIT License; its unchanged notice is the corpus
[LICENSE](../tests/corpus/csstree/LICENSE), which governs those seed files.
[corpus-import.toml](corpus-import.toml) records the corpus revision and
manifest digest, each imported case, and each excluded context.

Shared dependencies and their retained licenses are described in the
[surgeist-fuzz notice](../../surgeist-fuzz/NOTICE.md). Optional coverage uses
[libfuzzer-sys](https://github.com/rust-fuzz/libfuzzer), version 0.4.13, with
retained [MIT](../../surgeist-fuzz/tests/fixtures/subject/licenses/libfuzzer-sys/LICENSE-MIT),
[Apache 2.0](../../surgeist-fuzz/tests/fixtures/subject/licenses/libfuzzer-sys/LICENSE-APACHE)
and [LLVM notices](../../surgeist-fuzz/tests/fixtures/subject/licenses/libfuzzer-sys/LLVM-LICENSE.txt).
The corpus importer and its tests read the corpus with `serde_json`, a
development dependency pinned at the version the CSS crates already use. These
existing witnesses are reused; no software or license acquisition occurred.
