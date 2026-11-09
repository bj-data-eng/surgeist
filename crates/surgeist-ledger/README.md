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

## Declare dimensions

A consumer declares each dimension set with `dimensions!`. The macro writes a
fieldless enum with one variant per dimension and a table struct with one public
field per variant, named like the variant. The enum is the dimension type; the
table holds one value per dimension. An operation takes one set for work and one
for live resources. `NoDimension` is the empty set, and `NoTable::EMPTY` is its
table.

```rust
use surgeist_ledger::{Change, Limits, Operation};

surgeist_ledger::dimensions! {
    /// Work a parser charges.
    enum ParseWork => ParseWorkTable {
        /// One input byte read.
        InputByte,
        /// One tree node visited.
        NodeVisit,
    }
}

surgeist_ledger::dimensions! {
    /// Resources a parser holds while it runs.
    enum ParseResource => ParseResourceTable {
        /// Bytes of buffered input.
        Buffer,
    }
}

let limits = Limits::new(
    ParseWorkTable { InputByte: 4096, NodeVisit: 512 },
    ParseResourceTable { Buffer: 1024 },
);
let mut operation = Operation::new("parse", limits, 1)?;
let root = operation.root();
operation.apply_batch(
    root,
    &[
        Change::Acquire(ParseResource::Buffer, 64),
        Change::Work(ParseWork::InputByte, 64),
    ],
)?;
let usage = operation.usage(root)?;
assert_eq!(usage.work(ParseWork::InputByte), 64);
assert_eq!(usage.live(ParseResource::Buffer), 64);
assert_eq!(limits.work(ParseWork::NodeVisit), 512);
# Ok::<(), surgeist_ledger::Error<ParseWork, ParseResource>>(())
```

A table names every dimension, so a limit or a tariff cannot be left out, and a
lookup cannot go out of range. `Limits::from_fn`, `Tariffs::from_fn` and
`Limits::unlimited` build tables by calling a function once per dimension, and
`Tariffs::identity` keeps every dimension and unit in the parent. `Limits::work`
and `Limits::live` read a cap back. `Operation::limits` returns the caps a scope has, through a `Phase` as well, and `SharedOperation::limits` returns a
copy. It reports that scope's own caps, not those of its ancestors, so a consumer
that runs inside a scope its caller supplied can read the caps it works under.

`Operation::set_limits` and `SharedOperation::set_limits` replace a scope's caps for
the batches admitted after the call, for a consumer whose allowance changes between
calls. Usage, peaks, direct custody, the profile and open intervals stay as they are.
The new caps are not checked against the usage: a scope that holds more than a new
cap refuses its next acquisition of that dimension, even of zero, and still accepts
releases. A work cap set to the work admitted so far plus an allowance limits one
call's work.

A component that creates scopes of its own below a scope its caller supplied, checks a
batch before it applies it and reads the usage of its scopes takes a `ScopeLedger`. An
owned `Operation`, a `SharedOperation` and a mutable reference to either implement it, so
the component runs the same on each.

Dimensions of different sets are different types. A resource dimension cannot be
passed where a work dimension is expected, and a dimension of another set cannot
name a change, a limit or a usage read of an operation. An operation's `Limits`,
`Tariffs`, `Change`, `Usage` and reports carry the same two types.

```compile_fail,E0308
use surgeist_ledger::{Change, Limits, Operation};

surgeist_ledger::dimensions! {
    enum Visit => VisitTable { Node }
}

surgeist_ledger::dimensions! {
    enum Buffer => BufferTable { Bytes }
}

let mut operation =
    Operation::<Visit, Buffer>::new("parse", Limits::unlimited(), 1).unwrap();
let root = operation.root();
// A resource dimension is not a work dimension.
let _ = operation.apply(root, Change::Work(Buffer::Bytes, 1));
```

```compile_fail,E0308
use surgeist_ledger::{Change, Limits, Operation};

surgeist_ledger::dimensions! {
    enum Visit => VisitTable { Node }
}

surgeist_ledger::dimensions! {
    enum Buffer => BufferTable { Bytes }
}

surgeist_ledger::dimensions! {
    enum Elsewhere => ElsewhereTable { Node }
}

let mut operation =
    Operation::<Visit, Buffer>::new("parse", Limits::unlimited(), 1).unwrap();
let root = operation.root();
// A dimension of a set the operation does not use cannot name a change.
let _ = operation.apply(root, Change::Work(Elsewhere::Node, 1));
```

```compile_fail,E0308
use surgeist_ledger::{Limits, NoDimension, Operation};

surgeist_ledger::dimensions! {
    enum Visit => VisitTable { Node }
}

surgeist_ledger::dimensions! {
    enum Elsewhere => ElsewhereTable { Node }
}

// Limits of another schema do not configure this operation.
let limits: Limits<Elsewhere, NoDimension> = Limits::unlimited();
let _ = Operation::<Visit, NoDimension>::new("parse", limits, 1);
```

## Defects

`Error::Defect` reports that the ledger found one of its own invariants broken,
such as a scope identity that names no scope or a report with fewer records than
the operation has scopes. No valid use of the API produces it. Each `Defect`
variant names the broken invariant.

A defect is latched. After the ledger finds one, every later fallible call on
the operation, including `finish` and `snapshot`, returns the same
`Error::Defect`. A defect found on a path with no error return, such as phase
completion when a `Phase` is dropped, is returned by the operation's next
fallible call. `finish` returns it instead of a report, so a report is never
built from a state the ledger no longer trusts.

## Profile an operation

Select a mode at setup. Summary mode counts admissions and ended phase
invocations. Borrowed and shared phase helpers also complete on unwinding or
cancellation; detached observations require an explicit end. Turn on timing to
measure elapsed time for the operation and inclusive time for each invocation.
Overlapping invocations at the same scope can contribute more total time than
the operation elapsed.

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
use surgeist_ledger::{Change, Limits, NoDimension, NoTable, Operation, ProfileMode};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Visit }
}

let mut operation = Operation::<Work, NoDimension>::with_profile(
    "parse",
    Limits::new(WorkTable { Visit: 10 }, NoTable::EMPTY),
    1,
    ProfileMode::Summary { timing: true },
)?;
let root = operation.root();
operation.with_phase(root, |operation| {
    operation.apply(root, Change::Work(Work::Visit, 2))
})??;
let report = operation.finish()?.expect("summary enabled");
assert_eq!(report.scopes[0].usage.work(Work::Visit), 2);
println!("{}", report.terminal());
# Ok::<(), surgeist_ledger::Error<Work, NoDimension>>(())
```

Pass `ProfileMode::Off` to skip observation, or `timing: false` to collect work
and resource summaries without reading a clock. The structured report is
available independently of its terminal display adapter.

Run the release-mode profiling comparison with
`cargo bench --offline --locked -j 1 -p surgeist-ledger --bench profiling`.
The fixture compares borrowed `with_phase` and detached begin/end observations
around identical nested admissions, with one phase per 32 batches. Both styles
use the same six profiling modes and 11 rounds, rotating their execution order.
Its observer capacity column counts reserved report records and events, excluding
the operation ledger, allocator metadata, and process memory.

Run the admission cost comparison with
`cargo bench --offline --locked -j 1 -p surgeist-ledger --bench admission`.
It applies batches of one to ten changes at the root, below a capped root and at
the leaf of a four-scope chain, for a narrow and a wide dimension set. A small batch on a wide set changes only the values it
touches and journals their old values, so its cost does not grow with the width
of the sets; a refused batch is rolled back from the journal and leaves usage,
peaks and direct custody as they were.

Platforms without a supported standard clock can select untimed profiling
or supply a monotonic source through `Operation::with_profile_clock`.
A phase callback borrows the operation through a `Phase` view, so the ledger
and its report remain owned by the outer operation until the phase ends.

Use `with_async_phase` with an async callback to measure awaited work. Its phase
starts when the returned future is first polled, includes pending intervals, and
ends on completion, unwinding, or cancellation. Dropping an unpolled future
does not create a phase. The callback can admit work and run nested phases through
its borrowed `Phase`; the crate does not supply an executor.

Use `begin_phase` and `end_phase` when a callback or individual async poll must
run without retaining a mutable ledger borrow. The opaque `PhaseObservation`
holds its originating scope and optional start time. It does not borrow the
operation, allocate a registry entry or acquire a resource. Nested and
overlapping observations may end in any order; each contributes its own
inclusive interval.

```rust
use surgeist_ledger::{Change, Limits, NoDimension, Operation, ProfileMode};

# surgeist_ledger::dimensions! {
#     enum Work => WorkTable { Visit }
# }
let mut operation = Operation::<Work, NoDimension>::with_profile(
    "callback", Limits::unlimited(), 1,
    ProfileMode::Summary { timing: true },
)?;
let root = operation.root();
let observation = operation.begin_phase(root)?;
// The operation remains available while the observation is unfinished.
operation.apply(root, Change::Work(Work::Visit, 3))?;
operation.end_phase(observation).expect("observation belongs to this operation");
let report = operation.finish()?.expect("summary enabled");
assert_eq!(report.scopes[0].phases, 1);
assert_eq!(report.scopes[0].usage.work(Work::Visit), 3);
# Ok::<(), surgeist_ledger::Error<Work, NoDimension>>(())
```

Detached observations complete only through explicit `end_phase`. Dropping an
unfinished observation discards it without a clock read or phase completion;
finishing or dropping its operation does not manufacture an end. A consumer
that needs completion on return, unwinding or cancellation owns that guard.
The existing borrowed and shared phase helpers retain their automatic completion
behavior. Ending on a foreign operation returns the original unfinished marker,
without reading a clock or changing observations, so it can be retried on its
owner. A marker begun before `share()` remains valid, and any cloned participant
of that shared owner can end it.

After dropping a pending borrowed async phase, the caller can release admitted
resources and inspect `snapshot()` on the same operation. A snapshot returns an independent,
fallibly allocated copy of the retained profile without consuming or resetting
the operation. Profiling off returns no report. Operation elapsed time includes
the interval up to the snapshot, while phase summaries include only ended
phases. Resource peaks are lifetime peaks; snapshots do not calculate interval
peaks by subtraction.

## Observe persistent custody

Configure `with_checkpoint_limit` once before sharing an operation. Each
`begin_checkpoint` reserves its report buffers before opening an interval.
The bound limits concurrent intervals; ending one frees its slot. A dropped
token remains active until explicitly ended or the operation is dropped.
Foreign tokens and already-ended tokens return typed errors, including after
slot reuse. Registry and checkpoint identities never wrap.

`end_checkpoint` moves the reserved buffers without allocation. Its exact scope
usage contains work admitted during the interval, opening and closing live
balances, and the highest committed live balance during the interval. Opening
custody counts toward the peak; acquire/release pairs in a single accepted batch
also count. A refused batch changes no interval balance or peak. New scopes
appear with zero opening balances. Overlapping checkpoints remain independent.

The optional interval `profile` covers that same interval: work deltas, closing
live balances, exact interval peaks, admission/refusal observations, and a fresh
bounded detail stream. Its elapsed time and event timestamps start at interval
opening. Its ended-phase durations are clipped at interval opening. Phases
still active at closing are excluded; finish an attempt's phases before ending
its checkpoint to include all of them. Profiling off still exposes exact interval
usage, without reading a clock or retaining observation counters/events.

## Charge through a caller's scope

A component that owns a bounded operation takes a `Charge` instead of an
operation and a scope. `charge` applies one ordered batch, all or nothing, and
`record_largest` records the largest value a gauge resource reached. Both return
the implementation's `Stop`. `Operation::charge_at` and
`SharedOperation::charge_at` give the stock implementations, which charge a scope
of the operation and report the ledger `Error`. A consumer with its own admission
rules implements the trait over its accounting and maps the refusal into its own
stop type. Dispatch is static and each call carries a whole batch.

## Share short accounting calls

`share()` moves an operation into `SharedOperation`, a cloneable participant with
private synchronization. Admissions, usage queries, snapshots and checkpoint
calls hold only its short internal lock. Shared phase callbacks run unlocked,
including future creation, polling, awaiting and unwinding. They take no borrowed
phase argument; use a cloned participant to admit work or nest phases. Async
phases begin on first poll, include pending time and close once on completion or
drop. Plain operations retain their borrowed phase API without synchronization.

```rust
use surgeist_ledger::{Change, Limits, NoDimension, Operation, ProfileMode};

# surgeist_ledger::dimensions! {
#     enum Work => WorkTable { Visit }
# }
let operation = Operation::<Work, NoDimension>::with_profile(
    "device", Limits::unlimited(), 1,
    ProfileMode::Summary { timing: false },
)?.with_checkpoint_limit(2)?.share();
let participant = operation.clone();
let root = operation.root();
let checkpoint = operation.begin_checkpoint()?;
operation.with_phase(root, || {
    participant.apply(root, Change::Work(Work::Visit, 3))
})??;
let report = operation.end_checkpoint(&checkpoint)?;
assert_eq!(report.scopes[0].usage.work(Work::Visit), 3);
# Ok::<(), surgeist_ledger::Error<Work, NoDimension>>(())
```

Retain a participant and checkpoint custodian outside cancelable work. After
cancellation, perform semantic resource cleanup, then end the checkpoint to
retrieve accepted-prefix accounting and ended phases. Ledger does not infer
terminal status or perform consumer resource cleanup. Consumer locks must not
be retained across awaits or callbacks; shared phase helpers never lend their
internal lock to caller code. Clones keep the accounting owner alive, and the
shared API exposes no replaceable underlying operation or raw lock.

Use `check_batch` before a fallible external allocation when its requested size
must be rejected before allocating. It runs the same ordered limits, custody,
arithmetic and ancestor-tariff checks as admission, with no committed usage,
peak or profiling observation. This is a preview, not a reservation: always
apply the actual capacity after allocation, and handle refusal again. Another
shared participant can invalidate a previously successful check.
