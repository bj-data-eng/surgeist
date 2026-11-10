# JavaScript binding regression inputs

These inputs preserve native-host behavior independently of an ECMAScript engine
implementation. The consuming binding and harness tests own setup, actual native
objects, deterministic job delivery and observations. The fixtures supply guest
programs and independently specified results; this directory is not a runnable
suite or a claim that the planned bindings are implemented.

`cases.json` defines the guest-only cases. Run each source in a fresh host.
For a checkpoint case, observe `before`, perform one complete host microtask
checkpoint and evaluate `observe` to compare `after`. Other cases compare the
script result directly. Capability requirements identify the real producer a
consumer must install; missing capabilities must never produce a synthetic pass.

## Native setup and failure cases

The binding owner also retains the following requirements, which need genuine
native setup or fault injection rather than a guest-only input:

| Trigger | Required observation |
| --- | --- |
| Re-expose one native object, then force GC while a guest reference remains | Strict JS identity is preserved; a distinct native is distinct |
| Remove all Rust owner handles while a JS wrapper remains | Native data stays alive; collecting the last wrapper releases it once |
| Collect a wrapper while the canonical native still exists | A fresh wrapper observes the same current native data |
| Install native classes only in a second realm | Constructors are visible and usable there, with its prototype identity; the original realm has no injected registration |
| Retain, replace or remove a callback across scopes and GC | Only accepted roots survive; replacement/removal releases the old closure and its native captures |
| Invoke a callback that reenters bindings and removes itself | Invocation completes without a native borrow conflict; the registration remains removed |
| Reject publication before accepting a microtask | No callback executes; the unaccepted task and capture are retired |
| Report a host error after accepting a microtask | Accepted task custody remains live and it completes exactly once when driven |
| Report a host error after accepting a timer | Its cancellation token remains live; explicit teardown revokes it |
| Abandon a message receiver or retire its executor | The native waiter observes cancellation rather than a fabricated message |
| Abort with an expired listener | Abort state/reason remain committed; validation publishes no prefix and retains the original listener custody |
| Abort with a publication failure | Committed abort state and every listener tail settle; accepted tasks retain custody and unaccepted tasks retire |
| Clone cycles, shared objects, Map/Set edges or repeated Dates | Destination identity preserves aliases and cycles; dates retain their exact time, including invalid dates |
| Tear down a host after script failure or with queued jobs | Callback roots, native leases and host state are released; abandoned jobs do not execute |

The source-qualified host tests below establish why these expectations are
retained; their engine-specific Rust APIs are not a compatibility contract:

- [Microtask publication and discard](https://github.com/bj-data-eng/surgeist/blob/201c073501785761767f98cbb1a06f3f7bab644b/tools/surgeist-js-test-web-api/src/microtask/tests.rs)
- [Timer publication and cancellation](https://github.com/bj-data-eng/surgeist/blob/201c073501785761767f98cbb1a06f3f7bab644b/tools/surgeist-js-test-web-api/src/timers/tests.rs)
- [Message cancellation](https://github.com/bj-data-eng/surgeist/blob/201c073501785761767f98cbb1a06f3f7bab644b/tools/surgeist-js-test-host/src/message/tests.rs)
- [Abort and listener tails](https://github.com/bj-data-eng/surgeist/blob/201c073501785761767f98cbb1a06f3f7bab644b/tools/surgeist-js-test-host/src/abort/tests.rs)
- [Clone graph identity](https://github.com/bj-data-eng/surgeist/blob/201c073501785761767f98cbb1a06f3f7bab644b/tools/surgeist-js-test-web-api/src/store/mod.rs)

Realm registration is specified by [#2224](https://github.com/bj-data-eng/surgeist/issues/2224);
structured-clone transfer ordering remains [#1892](https://github.com/bj-data-eng/surgeist/issues/1892).
The other inputs use the owning DOM/HTML and Web IDL contracts selected in
[the source catalog](../../../references/SOURCE-EDITIONS.md), including lossless
UTF-16, native identity and the microtask checkpoint model. The guest fixtures
are independently authored against those contracts rather than copied from an
upstream test suite. This directory contains no engine or host implementation.
