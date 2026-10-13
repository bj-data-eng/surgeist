# HTML fuzz attribution

This package is repository-owned MIT verification source. It consumes the existing
HTML implementation and pinned behavioral references; it embeds no upstream
parser implementation or acquired corpus.

Shared dependencies and their retained licenses are described in the
[surgeist-fuzz notice](../../surgeist-fuzz/NOTICE.md). Optional coverage uses
[libfuzzer-sys](https://github.com/rust-fuzz/libfuzzer), version 0.4.13, with
retained [MIT](../../surgeist-fuzz/tests/fixtures/subject/licenses/libfuzzer-sys/LICENSE-MIT),
[Apache 2.0](../../surgeist-fuzz/tests/fixtures/subject/licenses/libfuzzer-sys/LICENSE-APACHE)
and [LLVM notices](../../surgeist-fuzz/tests/fixtures/subject/licenses/libfuzzer-sys/LLVM-LICENSE.txt).
These existing witnesses are reused; no software/license acquisition occurred.
