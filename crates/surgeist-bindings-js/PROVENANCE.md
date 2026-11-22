# Boa engine source

Bindings uses [Surgeist's Boa fork](https://github.com/bj-data-eng/surgeist-boa),
based on [Boa v0.22.0](https://github.com/boa-dev/boa/tree/v0.22). The root Git
tree pins the exact fork commit under `crates/surgeist-boa`. The Cargo graph
uses the checked-out `boa_engine` and `boa_gc` packages. The engine enables
Boa's `experimental` feature for the selected JavaScript surface.

Initialize the pinned checkout before building.

```sh
git submodule update --init crates/surgeist-boa
```

Fork changes use the same delivery process as Surgeist. Rebase a topic branch
onto the fork's current `main`, verify and review it, then merge its PR through
the required checks. Pin the resulting reviewed commit from the fork's
`main` and verify the consuming Surgeist branch. Upstream updates enter
through that same fork review and qualification process.

## Scoped object operations

The embedding boundary checks owner identity before opening an engine handle.
It preserves UTF-16 code units and bounds dense array construction to
16,777,216 elements. Allocation provenance is stored with the guest object.
The host can authenticate its creation realm after a prototype change or
cross-realm transfer. Callback custody and pending jobs keep referenced
objects and their realms alive until those references are released.

## Declared constructors

The host constructor callback admits its operands before reading
`NewTarget.prototype`. An object result becomes the native receiver prototype.
For any other result, Boa's GetFunctionRealm supplies the realm of the
registered fallback prototype. The native receiver is created in the
constructor's registered realm. A bound function or proxy can have its own
creation realm while GetFunctionRealm follows its target during fallback.
