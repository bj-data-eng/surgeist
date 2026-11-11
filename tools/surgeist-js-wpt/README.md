# Surgeist JavaScript conformance worker transport

This workspace tool supervises one process per fixture variant. Its caller
supplies a full source revision, relative fixture path, and exact root-relative
WPT URL, including any query or fragment. The fixture inventory owns the
association between those identities and the verified source bytes.

A worker reads one bounded request from stdin and writes one terminal response
to stdout. Both use the shared length-prefixed JSON frame. The response echoes
the request identity. Stderr carries bounded diagnostics. A report requires the
complete request transfer, one matching response followed by EOF, successful
process exit, and verified child and process-group cleanup.

The harness result retains the overall harness status and the ordered subtest
results. Guest text travels as UTF-16 code units. Worker-reported guest
failures, native refusals, unavailable capabilities, and engine incompatibility
have explicit forms. The supervisor preserves process exit, stop reason, and
cleanup evidence independently of the report. A process deadline cannot
supply a harness timeout or subtest status.

The existing process backend currently admits Apple-Silicon macOS. It owns
process-group cleanup and reports the absence of a hard memory ceiling and
RSS sampling. V8 heap and native host resource accounting belong to the
future native worker. Process-group cleanup covers members that remain in the
group.

The package currently exercises transport with a repository-authored Rust
worker. Loading and running the pinned WPT material through native Window
and Runtime capabilities is tracked by [#2260](https://github.com/bj-data-eng/surgeist/issues/2260).

```sh
cargo test --offline --locked -j 1 -p surgeist-js-wpt
cargo clippy --offline --locked -j 1 -p surgeist-js-wpt --all-targets --no-deps -- -F unsafe-code -D warnings
```
