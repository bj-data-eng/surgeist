# Surgeist Ledger

Surgeist Ledger provides synchronous operation budgets and resource accounting.
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
