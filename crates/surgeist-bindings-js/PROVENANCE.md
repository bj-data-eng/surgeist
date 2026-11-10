# External JavaScript backend

The direct dependency is crates.io `v8` exactly **152.2.0**, the Rust SDK from
[`denoland/rusty_v8`](https://github.com/denoland/rusty_v8/tree/v152.2.0).
The root lockfile authenticates its package with SHA-256
`a10fe1a92da5c32c7c7f838ce36c0ccfcfd5edf0865b58bdde820aa64cea9888`.
Its packaged VCS record names `2768994f664e8a6e3aba27503606c58339136e2a`
and marks the package dirty; this identifies provenance, not byte identity
with a clean checkout. Surgeist does not ingest its implementation.

The manifest uses the SDK's defaults, including `use_custom_libcxx`.
The qualified configuration is Rust **1.99.1**, edition **2024**, native
**aarch64-apple-darwin**, SDK default features, and the upstream release static
library. Other SDK targets/features have not been executed or qualified here.
Owned library/test code uses only safe SDK operations. Safe pinned initialization
replaces convenience scope macros whose expansion contains unsafe code.

## Native archive

The selected [release asset](https://github.com/denoland/rusty_v8/releases/tag/v152.2.0)
is `librusty_v8_release_aarch64-apple-darwin.a.gz`, **40,635,999 bytes**.
The release API publishes SHA-256
`a4a5484444ad4b5f10dc7bed141236dd9dbeabf2006d14520f19b935d4db4227`.
The qualified decompressed archive is **157,813,600 bytes**, SHA-256
`680617c57021c1ad7490749e4c10c4504978e84c3e3a288b8bb39a31a9e0d1f8`.
This is release metadata and observed-byte qualification, not an independent
reproducible build or a publisher signature.

[`tools/native.py`](tools/native.py) validates this exact archive and supplies
`RUSTY_V8_ARCHIVE` and `RUSTY_V8_ARCHIVE_SHA256` before executing a command.
By default it refuses missing/mismatched input and performs no download.
Explicit `--download` retrieves only the pinned asset, verifies its published
compressed digest and bounded decoded bytes, and refuses to overwrite an
existing mismatched archive. No source build, toolchain or makefile is used.

From the repository root, with the archive in a chosen cache location:

```sh
python3 crates/surgeist-bindings-js/tools/native.py --archive /absolute/path/librusty_v8.a -- cargo test --offline --locked -j 1 -p surgeist-bindings-js
```

Use `--download` before `--` only when acquisition is authorized and necessary.
Cargo's `--offline` does not itself disable native build-script downloads, so
the explicit local artifact is required for reproducible offline verification.
The runner's initial artifact is for the native target only; cross compilation
requires separate target/artifact qualification. Cache retention follows the
repository's normal owned-cache cleanup guidance.

## Ownership constraints

The pinned SDK's `OwnedIsolate::new` enters the isolate, and its destructor
checks that it is current before exiting. A HandleScope does not switch
isolates. Accordingly `Process` owns the only entered isolate until teardown;
temporary `Execution` guards only borrow it. A static owner token refuses a
second live process token before creating another isolate. Non-Send/non-Sync
owners, scoped values and private owner-checked roots prevent unsupported thread
entry and foreign handle conversion.

The SDK initialization contract additionally requires protection-key ancestry on
some x86 hosts. Bootstrap records and confines all future engine entry to the
actual initializing thread rather than permitting arbitrary pre-existing pools.
The ARM64 results do not qualify a future worker-transfer or shared-isolate API.
The process-lived platform is initialized once and deliberately never disposed
or dynamically unloaded. Per-owner roots, contexts and isolate use safe RAII.

The [crate notice](NOTICE.md) distinguishes SDK, native V8 and bundled-library
terms. A source-qualified dependency notice is not certification of every
possible downstream executable or static-link composition.

## Scoped object operations

The backend captures pristine `Reflect.set` and `Reflect.setPrototypeOf` to
preserve Boolean rejection and original proxy exceptions. The SDK's direct
prototype setter suppresses proxy trap exceptions in the pinned
[`SetPrototypeImpl`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/src/api/api.cc)
path. These functions and the selected `Array.prototype.values` intrinsic are
retained in private managed heap data. Callback registrations retain those heap
edges rather than adding persistent Rust roots back to their creation context.

Rust-side dense array construction is limited to **16,777,216 elements**, checked
before root-vector allocation and SDK construction. This is an embedding policy,
not the JavaScript array-length limit. The pinned
[`fixed-array-base.h`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/src/objects/fixed-array-base.h)
defines a capacity of 16 Mi elements in lower-limits mode and 128 Mi otherwise.
The safe SDK exposes no runtime query for that build choice; the conservative
minimum avoids its capacity checks in either mode. Boundary qualification uses
scalar length checks, without allocating an enormous array.

## Declared constructors

The scoped constructor uses a private constructible target and a public Proxy
with a native construct trap. The pinned SDK's direct constructor path allocates
before entering Rust and falls back to the target realm's Object prototype,
which does not implement Web IDL's argument-conversion ordering or same-interface
prototype fallback. The
[`Proxy construct path`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/src/builtins/builtins-proxy-gen.cc)
instead invokes the trap without allocating the target's default receiver.
The handler selects the prototype after conversion and returns an explicit
object. The private target is not exposed through prototype.constructor.

Function length is an explicit own Number data property: nonwritable,
nonenumerable and configurable. The internal native template uses zero rather
than routing metadata through its
[`16-bit length storage`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/src/objects/templates-inl.h).
Every u32 value is exactly representable as Number. Qualification checks the
65,535/65,536 boundary, 1,000,000 and u32::MAX, including binding and construction.

The SDK does not expose semantic GetFunctionRealm. For nonobject prototype
fallback, the backend safely unwraps outer Proxies and makes a private sanitized
Proxy of the ordinary function. Its trap returns own data values or undefined
without invoking guest name/length accessors. Captured pristine bind creates a
fresh bound shell without an own prototype. A second private Proxy forces that
shell's prototype lookup to undefined; captured Reflect.construct and Object
then execute the engine's real
[`GetFunctionRealm` fallback](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/src/objects/js-function.cc).
The resulting Object prototype's creation context identifies the target realm;
the probe object's own creation context does not.

This composition preserves the original single prototype Get, including a getter
that freezes its property while returning a primitive. Both private handlers
have null prototypes, and descriptor reads test own fields before access.
[`Bind's metadata processing`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/src/objects/js-function.cc)
does not coerce object-valued name/length data. Qualification covers ordinary,
bound and wrapped function families reachable through this bridge, including
cross-realm Proxy targets and revocation. Independently injected SDK callables
with access checks or interceptors are outside this owned execution contract.

Creation context, pristine intrinsics and private handler edges remain in the
managed heap. Context metadata stores only the checked realm key, so dropping a
public realm token does not invalidate a reachable constructor or root a cycle.
No guest source snippets, unsafe code or patched dependency are needed.
The pinned Web IDL leaves constructorless interfaces' IsConstructor status
unresolved; this declared-constructor capability selects no policy for them.
