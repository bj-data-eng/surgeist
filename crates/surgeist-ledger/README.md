# Surgeist Ledger

Surgeist Ledger provides operation budgets and resource accounting.
Consumers choose their work and resource dimensions, limits, and tariffs.
The crate uses only the Rust standard library.

An operation has a root and a bounded tree of named scopes. Usage in each
scope includes its descendants, expressed in that scope's units. A child can
count local actions while its parent counts converted work credits. A report
should use the root's totals and display child usage as attribution.

A batch checks changes in input order, starting at the originating scope for
each change. A scope's local limit is checked before projecting to its parent.
Overflow or refusal leaves the entire batch unchanged, including its peak.
Earlier accepted batches remain recorded.

Work is cumulative. Live resources are acquired and explicitly released by
the scope that owns them. Use an acquire followed by a release for temporary
resources. Peak records the highest accepted live usage, even when a resource
is released later in the same batch. Integral tariffs preserve partial
release amounts across scopes.

Consumers charge their own operations and decide when to check cancellation,
stop, or map a refusal. Cleanup can release previously admitted resources
after a stop. Treat charges as definitions of work and resource custody.
Use a separate elapsed-time or process-memory measurement for those quantities.

## Profile an operation

Select a mode at setup. Summary mode counts admissions and ended phase
invocations, including phases ended by unwinding or cancellation. Turn on timing
to measure elapsed time for the operation and inclusive time for each invocation. Overlapping invocations at the same scope
can contribute more total time than the operation elapsed.

Detailed mode retains selected accepted items, first-failed items of refused
batches, and phase completions up to its configured event limit. An atomic batch
shares a timestamp and contributes accepted items only when it commits.
The first refusal and admission totals are also retained per originating scope.
The report excludes empty batches and invalid foreign-scope calls. A filter
selects event origins by their static scope names. Filtered events are omitted
from the drop count, which measures selected events lost to the capacity bound.
Use fixed labels that describe the work. Ledger events carry dimensions and
quantities.

Observer buffers are reserved at setup. The summary has one record per scope
up to the operation's scope limit. Detail has a separate event limit. Once
setup succeeds, observation never grows either buffer during admission.
The accepted ledger totals remain authoritative when detail is incomplete.

```rust
use surgeist_ledger::{Change, Limits, Operation, ProfileMode, Work};

const VISITS: Work<1> = Work::new(0).unwrap();
let mut operation =
    Operation::<1, 0>::with_profile("parse", Limits::new([10], []), 1,
        ProfileMode::Summary { timing: true })?;
let root = operation.root();
operation.with_phase(root, |operation| {
    operation.apply(root, Change::Work(VISITS, 2))
})??;
let report = operation.finish().expect("summary enabled");
assert_eq!(report.scopes[0].usage.work(VISITS), 2);
println!("{}", report.terminal());
# Ok::<(), surgeist_ledger::Error<1, 0>>(())
```

Pass `ProfileMode::Off` to skip observation, or `timing: false` to collect work
and resource summaries without reading a clock. The structured report is
available independently of its terminal display adapter.

Run the release-mode profiling comparison with
`cargo bench --offline --locked -j 1 -p surgeist-ledger --bench profiling`.
The fixture times identical nested admissions with one phase per 32 batches.
Its observer capacity column counts reserved report records and events, excluding
the operation ledger, allocator metadata, and process memory.

Platforms without a supported standard clock can select untimed profiling
or supply a monotonic source through `Operation::with_profile_clock`.
A phase callback borrows the operation through a `Phase` view, so the ledger
and its report remain owned by the outer operation until the phase ends.

Use `with_async_phase` with an async callback to measure awaited work. Its phase
starts when the returned future is first polled, includes pending intervals, and
ends on completion, unwinding, or cancellation. Dropping an unpolled future
does not create a phase. The callback can admit work and run nested phases through
its borrowed `Phase`; the crate does not supply an executor.

After dropping a pending phase, the caller can release admitted resources and
inspect `snapshot()` on the same operation. A snapshot returns an independent,
fallibly allocated copy of the retained profile without consuming or resetting
the operation. Profiling off returns no report. Operation elapsed time includes
the interval up to the snapshot, while phase summaries include only ended
phases. Resource peaks are lifetime peaks; snapshots do not calculate interval
peaks by subtraction.
