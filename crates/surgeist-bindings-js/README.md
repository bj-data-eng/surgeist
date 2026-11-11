# Surgeist JavaScript bindings

`surgeist-bindings-js` owns the safe embedding boundary and native JavaScript
bindings. The initial backend is external [`rusty_v8`](https://github.com/denoland/rusty_v8).
It does not own a JavaScript VM, garbage collector or replacement DOM.

The current core supplies one thread-affine `Process`, borrowed `Execution`,
multiple realms, scoped values, explicitly retained roots, exact UTF-16 strings,
property operations, calls, original exception values, Web IDL primitive
conversion, metered scalar, enumeration, nullable, dictionary and synchronous
sequence conversion,
declared argument defaults, scoped Rust callbacks,
declared constructors and explicit shared microtask checkpoints.
The [Runtime adapter](../surgeist-runtime/README.md) uses this boundary
for attached Window services and standalone Event dispatch. Native owners
retain their own interface state and execution authority.

Window setup uses a pending realm so the backend can establish the immutable
global prototype chain when it creates the actual global. The host installs
the native EventTarget interface on the reserved prototype before obtaining
the ordinary realm capability.

## Ownership

Bootstrap on the worker's entry thread. Only that thread can reacquire the
backend, and only after the prior process token has dropped. One token owns one
entered isolate until teardown; execution guards only borrow it. This avoids
unsupported isolate switching and destruction order, including when a caller
forgets a guard. Multiple realms share the isolate and explicit job queue.
The platform remains initialized for the lifetime of the operating-system
process. Do not independently initialize or enter the same V8 dependency.

`in_realm` accepts a fallible closure and returns its flattened `Result`, including
cleanup at the host exit. Scoped values cannot escape that closure. Explicit
`Root` tokens contain
checked owner and allocation identities; the backend keeps the actual handles.
Foreign tokens fail before SDK conversion. Context and value roots are reaped
at operation boundaries after their external Rust custody is dropped. Native
projections can store private object references traced by the guest heap.
A weak projection token can reopen the exact live wrapper in its owning
heap without prolonging its lifetime. Callback
registrations retain their creation context and pristine intrinsics through
guest heap edges, allowing unreachable context/function cycles to be collected.
Guest references and pending jobs can retain their objects after Rust roots
are released.
Dropping the process releases its handles and isolate without running jobs.

GC finalizers only mark callback registrations for retirement. The owner releases
their Rust captures at a host boundary after SDK calls and registry borrows end.
Capture destructors retain ordinary Rust unwind behavior there; callback-body
panic refusal is a separate contract.

Callback scopes share the production root registry through short checked
borrows. No registry borrow crosses SDK allocation, coercion or a guest call.
An internal registry conflict permanently refuses the owner and requests V8
termination. V8 observes termination at interrupt checks; every native entry
and continuation after guest coercion must also check the sticky failure before
accepting native work. Refusal preserves the original host error and still
permits required resource cleanup.

Native host adapters may observe the entered realm and weak process liveness
through `Scope::realm_authority`. In a callback this observes its creation realm;
the adapter separately authenticates native authority. `HostFailureSlot` accepts
pre-admitted storage and freezes one original typed native cause without a late
allocation. The cause and its debit travel together with retained failure tokens.
`Scope::refuse_host` preserves the first host failure, distinct from a guest throw.

`Callback` supports recursive Rust callables through `Scope::function`.
Arguments preserve omission versus explicit undefined; values and completions
remain confined to their scope unless explicitly retained. A callback returns a value,
undefined or an exact guest throw. A returned host error permanently refuses
the owner. The SDK boxes primitive receivers and maps null/undefined to the
function's creation-realm global; native adapters still validate brands.
`Scope::function` remains callable only. `Scope::constructor` uses a distinct
`ConstructorCallback` with an exact UTF-16 name and full-u32 length. Its handler runs
in the creation realm, converts arguments, then consumes `select_prototype` to
read NewTarget.prototype once. A nonobject result resolves the registered
same-interface prototype in NewTarget's function realm; native creation remains
in the original realm. Ordinary calls throw TypeError before the handler.
Construction requires an explicit object result or exact guest throw; a primitive
return is a terminal host refusal. The adapter still owns native authority and
interface algorithms. The backend's [qualified constructor path](PROVENANCE.md#declared-constructors)
avoids allocating a platform receiver before conversion.

Property operations take explicit `PropertyKey` values: exact UTF-16 names or
existing String/Symbol values, without implicit key coercion. Complete data and
accessor descriptors, explicit prototypes, dense arrays and selected pristine
intrinsics support binding installation without evaluating source snippets.
Array construction has the [qualified embedding bound](PROVENANCE.md#scoped-object-operations).

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

The other primitive conversions preserve noncoercive boolean semantics, exact
DOMString code units, USVString surrogate replacement and checked ByteString
bytes. Restricted and unrestricted floating conversions preserve the selected
standard's rounding, signed zero, overflow and canonical NaN rules. Conversions
that invoke guest coercion do so once and preserve the original thrown value.
`Primitive` constructs undefined, null, Boolean and Number values directly.
Value-kind and callability checks invoke no guest properties or coercion hooks;
classes and revoked callable proxies remain callable even though calls throw.

[`conversion`](src/conversion/mod.rs) provides closed typed descriptors for
scalars, enumerations, nullable values, synchronous sequences and complete
brand-free dictionaries.
`Conversion::value` accepts the sealed descriptor family, including
`IntegerType`; nullable descriptors preserve an inner type that includes
undefined and cannot be wrapped in a second nullable descriptor. A `ConversionAccount` attaches an authenticated child to a caller-owned
Ledger operation. Its `run` method borrows the entered scope; `Converted` outputs
retain their requested Rust buffer charges until their payloads are destroyed.
Charges cover binding work and storage, excluding engine instructions and heap
allocation. Timing follows the Ledger operation's profiling configuration.
The existing primitive methods remain available without this account.

The [generated descriptors](src/conversion/declarations.rs) derive dictionary
member order, argument indices, optionality and defaults from the same authored
JSON as the typed values. Dictionary conversion visits inherited members before
derived members, sorts each declaration lexicographically, and converts each
member immediately after reading it. Undefined and null dictionaries use the
declared member defaults. A dictionary with any unavailable native brand is
excluded as a whole.

`Conversion::argument` converts one already-selected declaration argument from
`arguments.get(descriptor.index())`. Optional omission and explicit undefined
select the declared default or `ArgumentValue::Missing`; null follows the
selected type's ordinary conversion. These descriptors do not perform overload
selection, authenticate a native receiver or install an interface. Returned
nullable and argument values retain the original output's storage custody.

`descriptor.sequence()` composes through the same `Conversion::value` front.
It caches the original iterator method and next function, reads done before
value, and converts each yielded item before advancing. The pinned algorithm
performs no IteratorClose on conversion failure. Iterator operations and items
are admitted before their guest effects. Empty sequences allocate no element
buffer; requested capacity starts at four slots and doubles with checked bounds.
Growth charges the complete old and new backing at once, counts every moved
accepted element, and frees old backing before releasing its charge. The result
owns a vector of `Converted` children, each retaining its original payload
custody in addition to the outer buffer. Nested sequences and nullable values
can outlive their realm or process without retaining engine handles.

Metered validation throws an intrinsic TypeError; guest coercion preserves the
original exception. Host refusal is terminal. If cleanup also fails at an actual
host or callback exit, its terminal report preserves the selected primary
exception and exposes the cleanup fault separately. A prior host refusal remains
first. Cleanup guards enter no SDK code, and cleanup diagnostics remain readable
after an account drops. Enumeration attribute assignment ignores a nonmember
after one coercion. Generated
values and conversion descriptors derive from the single
[authored description](bindings/dom-core.json). Dictionaries that require native
brands and union and overload selection have separate binding work.

[Runtime's native document attachment](../surgeist-runtime/README.md) installs
the standard `CSS` namespace in the selected Window realm. `CSS.escape` uses
the generated required DOMString argument descriptor and the native CSSOM
serializer. The fixed-arity callback captures at most one guest argument.
Conversion consumes cumulative per-realm work and live Rust buffer limits.
Native identifier limits bound each call, and a native refusal retains the
original typed cause.

The same attachment installs the Web IDL `DOMException` interface. Its
instances and interface prototype use V8's native Error allocation and stack
state. Getters read private instance data. The generated constructor converts
both optional DOMStrings before reading NewTarget's prototype. Each installed
realm has its own constructor, prototype and conversion limits. Its account
charges each explicit UTF-16 copy between V8 and Rust, including outbound
copies when a native callback creates a typed diagnostic.
`Scope::dom_exception` creates a guest exception value for a standard
`DomExceptionName` using the entered realm's private prototype. A native
callback can return the value as `Completion::Throw`.

## Verification

Use the exact native artifact and offline environment described in
[PROVENANCE.md](PROVENANCE.md). Cargo's offline mode alone does not prevent the
dependency's build script from fetching a native library. The execution test is
a dedicated worker-process executable, so it exercises bootstrap and all realms
on one entry thread rather than unrelated test-pool threads.
Capture-disposal tests use isolated workers to check GC and owner teardown,
including destructor unwinds and preservation of prior host refusal.
The conversion executable exercises real callbacks, admission, retained storage,
reentry and profiling equivalence; private fault tests exercise cleanup failures.

```sh
cargo test --offline --locked -j 1 -p surgeist-bindings-js
cargo clippy --offline --locked -j 1 -p surgeist-bindings-js --all-targets --no-deps -- -F unsafe-code -D warnings
cargo fmt --all -- --check
python3 -B crates/surgeist-bindings-js/tools/generate_bindings.py --check
python3 -B crates/surgeist-bindings-js/tools/test_generate_bindings.py --rustc
```
