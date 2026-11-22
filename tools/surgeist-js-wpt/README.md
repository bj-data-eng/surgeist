# Surgeist JavaScript conformance workers

This workspace tool supervises one process per fixture variant. Its caller
supplies a full source revision, relative fixture path, and exact root-relative
WPT URL, including any query or fragment. The fixture inventory owns the
association between those identities and the verified source bytes.

A worker reads one bounded request from stdin and writes one terminal response
to stdout. Both use the shared length-prefixed JSON frame. The response echoes
the request identity and carries the outcome and worker resource observations.
Stderr carries bounded diagnostics. A report requires the complete request
transfer, one matching response followed by EOF, successful process exit, and
verified child and process-group cleanup.

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
It matches the request's full identity against the compiled fixture inventory.
For an [authored qualification fixture](fixtures/qualification/), the revision
is its Git blob ID and the worker attaches to a native Runtime-owned Window on
the entry thread. The worker runs Boa with finite native and report limits.
Its supervisor enforces the process deadline.

The [upstream inventory](fixtures/wpt/README.md) uses the WPT commit as the
revision. The worker verifies the exact HTMLCollection page and its two
embedded support scripts, attaches before EOF, and advances the live parser.
The worker resolves each external script from the verified local inventory and
executes it at the parser's blocking pause. It observes the public
`add_completion_callback` report after the native Window load and timer
tasks. The harness's public `output: false` setting disables its optional DOM
status table, which uses selector APIs outside this native profile.

The two EventTarget Window variants report their execution environment as
unavailable. A request outside the inventory reports an unknown fixture. A
source integrity failure stops the worker before it emits a terminal report.

A qualified native response reports cumulative Runtime guest adapter work,
live backing after teardown, and peak backing for the same Ledger. The Boa
observation records the current thread's shallow GC and ephemeron box bytes
before teardown, including boxes awaiting collection. This counter excludes
separate buffers, allocator overhead, and other process memory. Boa supplies
no heap limit in this report.
Setup refusal can leave the account unattached and the selected engine policy
without a sample. After a deadline or crash, the supervisor reports the process
outcome and memory capabilities.

```sh
cargo test --offline --locked -j 1 -p surgeist-js-wpt
cargo clippy --offline --locked -j 1 -p surgeist-js-wpt --all-targets --no-deps -- -F unsafe-code -D warnings
```

Run the native qualification on the supported process host with the pinned
Boa checkout initialized.

```sh
cargo test --offline --locked -j 1 -p surgeist-js-wpt --features native-worker
```

Further upstream harness and platform capabilities are tracked in
[#2260](https://github.com/bj-data-eng/surgeist/issues/2260).
