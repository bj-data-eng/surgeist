# Surgeist JavaScript conformance workers

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

The process backend supports Apple-Silicon macOS. It owns process-group
cleanup and reports the availability of a hard memory ceiling and RSS
sampling. Both are unavailable on this backend. Process-group cleanup
covers members that remain in the group.

Enable `native-worker` to build the `surgeist-js-wpt-worker` executable.
It reads one request, matches the full identity against its compiled
[qualification fixtures](fixtures/qualification/), and attaches to a native
Runtime-owned Window on the entry thread. The revision for each authored
fixture is its actual Git blob ID. The worker uses finite native and report
limits and V8's default heap. Its supervisor enforces the process deadline.

```sh
cargo test --offline --locked -j 1 -p surgeist-js-wpt
cargo clippy --offline --locked -j 1 -p surgeist-js-wpt --all-targets --no-deps -- -F unsafe-code -D warnings
```

With the pinned V8 archive configured, run the native qualification on the
supported process host.

```sh
cargo test --offline --locked -j 1 -p surgeist-js-wpt --features native-worker
```

The native qualification runs authored assertions. The pinned WPT harness,
additional platform capabilities, and final memory reporting are tracked in
[#2260](https://github.com/bj-data-eng/surgeist/issues/2260).
