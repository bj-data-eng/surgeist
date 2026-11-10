# Surgeist JavaScript bindings

This crate owns the safe embedding boundary and native bindings, not an engine
implementation or a second live tree. The public front door is
[`src/lib.rs`](src/lib.rs); [`backend::Engine`](src/backend/mod.rs) owns the private
replacement boundary and [`backend::v8`](src/backend/v8.rs) contains every concrete
SDK operation. Native interface algorithms must remain independent of SDK types.

[`README.md`](README.md) describes current capability and focused checks.
[`PROVENANCE.md`](PROVENANCE.md) owns the exact dependency/native artifact and
offline setup; [`NOTICE.md`](NOTICE.md) owns its legal inventory. The native
runner validates the archive before Cargo, whose offline mode alone cannot stop
native build-script acquisition. The qualified target is native macOS ARM64.

`Process` alone owns the entered isolate. `Execution` only borrows it; realms
share its explicit queue. Keep concrete handles private, validate foreign owner
identities before SDK conversion, use exact UTF-16, and retain original thrown
values without guest diagnostic coercion. No checkpoint runs during teardown.
Production bootstrap never enables test GC flags. The native qualification runs
as one entry-thread test; public execution tests use a dedicated executable.
