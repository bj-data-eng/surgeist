# Surgeist JavaScript bindings

`surgeist-bindings-js` owns the safe embedding boundary and native JavaScript
bindings. The initial backend is external [`rusty_v8`](https://github.com/denoland/rusty_v8).
It does not own a JavaScript VM, garbage collector or replacement DOM.

The current core supplies one thread-affine `Process`, borrowed `Execution`,
multiple realms, scoped values, explicitly retained roots, exact UTF-16 strings,
property operations, calls, original exception values, Web IDL integer
conversion and explicit shared microtask checkpoints. Installed Web IDL
interfaces, DOM, CSSOM and Runtime adapters are not
yet implemented. This is not a browser environment or a WPT runner.

## Ownership

Bootstrap on the worker's entry thread. Only that thread can reacquire the
backend, and only after the prior process token has dropped. One token owns one
entered isolate until teardown; execution guards only borrow it. This avoids
unsupported isolate switching and destruction order, including when a caller
forgets a guard. Multiple realms share the isolate and explicit job queue.
The platform remains initialized for the lifetime of the operating-system
process. Do not independently initialize or enter the same V8 dependency.

Scoped values cannot escape an `in_realm` closure. Explicit `Root` tokens contain
checked owner and allocation identities; the backend keeps the actual handles.
Foreign tokens fail before SDK conversion. Context and value roots are reaped
at operation boundaries after their last Rust token is dropped. Guest references
and pending jobs can retain their objects after Rust roots are released.
Dropping the process releases its handles and isolate without running jobs.

Callback scopes share the production root registry through short checked
borrows. No registry borrow crosses SDK allocation, coercion or a guest call.
An internal registry conflict permanently refuses the owner and requests V8
termination. V8 observes termination at interrupt checks; every native entry
and continuation after guest coercion must also check the sticky failure before
accepting native work. Refusal preserves the original host error and still
permits required resource cleanup.

Exception capture never invokes a guest `toString` or `stack` getter. Explicit
`to_utf16` conversion follows JavaScript ToString and can itself throw.
Checkpoints are shared across realms. The current SDK checkpoint swallows job
callback exceptions; promise rejection reporting requires the host adapter.
Heap exhaustion/native fatal errors remain dependency-level process failures;
this core does not claim to bound guest CPU or native heap allocation.

The private [`backend::Engine`](src/backend/mod.rs) trait isolates rooted values,
conversions, invocation, exception capture, job entry and destruction from binding
algorithms. It is an intentional engine replacement boundary, not a claim that a
second backend or live heap migration is supported.

Integer conversion follows the complete pinned
[Web IDL/ECMAScript clauses](../../references/js-bindings-standards.md).
It preserves one guest ToNumber coercion, exact modulo, ties-even clamping and
the selected standard's safe-integer bounds for 64-bit range/clamp conversions.
`IntegerKind` selects a standard IDL type and `IntegerValue` retains that type
with its corresponding fixed-width Rust payload.

## Verification

Use the exact native artifact and offline environment described in
[PROVENANCE.md](PROVENANCE.md). Cargo's offline mode alone does not prevent the
dependency's build script from fetching a native library. The execution test is
a dedicated worker-process executable, so it exercises bootstrap and all realms
on one entry thread rather than unrelated test-pool threads.

```sh
cargo test --offline --locked -j 1 -p surgeist-bindings-js
cargo clippy --offline --locked -j 1 -p surgeist-bindings-js --all-targets --no-deps -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```
