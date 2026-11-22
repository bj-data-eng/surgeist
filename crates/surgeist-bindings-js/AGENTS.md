# Surgeist JavaScript bindings

This crate owns the safe JavaScript embedding boundary and native bindings.
The public front door is [`src/lib.rs`](src/lib.rs).
[`backend::Engine`](src/backend/mod.rs) defines the private replacement
boundary and [`backend::boa`](src/backend/boa.rs) implements it. Native
interface algorithms use the binding types.

The [README](README.md) describes capabilities and focused checks.
[PROVENANCE.md](PROVENANCE.md) identifies the pinned engine source and
[NOTICE.md](NOTICE.md) records attribution.

`Process` owns the thread-affine context. `Execution` borrows it and its realms
share the explicit job queue. Check owner identity before handle conversion,
preserve exact UTF-16 and thrown values, and keep callbacks' creation-realm
custody. Teardown releases roots and pending jobs without running a checkpoint.
