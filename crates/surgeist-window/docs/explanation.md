# Explanation

## Owning the window boundary

`surgeist-window` owns Surgeist's window, app-host, event-loop, and platform-host
contracts: native identity, events, commands, handles, and capabilities. Its
public API is the app-facing boundary; backend adapters remain private.
Renderers, surfaces, UI semantics, layout, hit testing, and application behavior
belong outside this crate. [src/lib.rs](../src/lib.rs) is the public front door.

The deterministic and native hosts share command planning and the lifecycle
pump. `testing::Runner` exercises handler dispatch and ordering without a native
display. `testing::Host` provides callback-free command, state, and effect
recording. This separates tests of the contract from platform-owned event timing
and native effects. See [the pump](../src/pump.rs),
[deterministic backends](../src/testing.rs), and
[native backend](../src/winit_adapter.rs).

## Authored work and observed state

An `Open` builder and `Command` express authored intent. `App::into_loop` retains
authored startup commands without validation. Before native event-loop creation,
`App::run` or `Loop::run` intrinsically validates and normalizes the complete
startup batch against a virtual registry and lifecycle projection. The
deterministic `Runner::startup` performs this intrinsic validation when storing
its batch.

At first native resume, the host resolves capabilities and stages startup
planning before invoking `Handler::resume`. After that callback returns
successfully, the pump finishes joint planning for startup and callback commands,
then applies startup commands before the callback's commands. Startup windows
are therefore not yet live during the first resume callback. The deterministic
runner follows the same staging and application order with its configured
capability report. The implementation is in [Loop](../src/loop_.rs),
[WinitRunner](../src/winit_adapter.rs), and [Runner](../src/testing.rs);
`pump_startup_and_resume_apply_in_order_after_joint_preflight` in
[src/tests.rs](../src/tests.rs) checks this order.

`WindowSnapshot` and `Metrics` are observed runtime state. A buildable request can
still fail during planning or application; capability permission is not an
observation that the native operation succeeded. The error retains its semantic
code, target when available, command kind, and underlying source. The
[reference](reference.md#capabilities) lists the three capability outcomes.

## Callback transactions and terminal results

After successful creation, `Handler::event` receives `EventKind::Created` before
`Handler::ready`. Successful `Created` work is applied before readiness for a
still-live window, and a successful ready callback schedules a next-frame draw.
Snapshot-changing native events commit their state before callback delivery;
input and metrics use specialized callbacks. Close requests use `Handler::close`,
whose default accepts, and completed closing delivers `Handler::closed` once with
the final snapshot. Generic events do not represent close requests or destruction.

Each callback owns a short transaction. Commands and draw, close, or exit actions
queued through its `Context` or scope commit only after it returns `Ok`.
Returning an error discards that queued work. Validation, planning, and
application after callback success can still fail. A callback error,
committed-command failure, or terminal action retains the first terminal outcome
and suppresses later callbacks. Successful exit records `Ok(())` and closes proxy
ingress. See [Handler](../src/handler.rs), [Context](../src/context.rs), and
[Pump](../src/pump.rs).

Ordered command batches are preflighted against a virtual registry before backend
effects. A predictable invalid command rejects the batch without applying it.
If backend application fails after earlier commands succeed, that committed
prefix remains; the error is `CommandBatchFailed` with `completed_prefix`, and the
tail is not applied. These boundaries are defined by [planning](../src/planning.rs)
and [structured errors](../src/error.rs).

Clipboard text and image reads or writes happen immediately while the callback
borrows the loop-owned `Clipboard`. They can report errors and are external I/O,
so a later callback error does not undo a successful clipboard operation.
`MemoryClipboard` provides deterministic storage. See
[clipboard procedures](how-to.md#use-a-loop-owned-clipboard).

## Identity and proxy ingress

`Id` is a monotonically issued, never-reused runtime identity. An authored name is
an optional live lookup and uniqueness label; it never determines the `Id`.
Closing removes the window from live name and state lookup before cleanup.
Synchronous work against a closing generation reports `WindowClosing`; work
against an unknown or closed generation reports `StaleWindow`.

Typed proxy commands are intrinsically normalized before synchronous queue
acceptance. Malformed authored work returns an error without entering ingress.
Capability and lifecycle planning, and application, remain on the owning event
loop. Accepted work drains in queue order through the non-reentrant pump;
`testing::Runner::flush_proxy` follows this policy. The contract does not select
an order between concurrent producers. Target work found stale or closing during
drain is ignored without changing the terminal result. Closed ingress rejects
later work with `CommandFailed`. See [Registry and Proxy](../src/registry.rs),
[typed proxy submission](../src/dsl.rs), and [Runner](../src/testing.rs).

## Handles and closing ownership

`Context::access`, `Context::handle`, and the corresponding scope methods provide
live-window access. A retained `Handle` keeps raw window and display trait access
while its lease is `Live` or resource-owning `Closing`. After destruction or loop
exit, the raw-window-handle traits return
`raw_window_handle::HandleError::Unavailable`.

Closing completion and native resource release can wait until the final retained
owning `Handle` lease is dropped or released. This lets the host stop live work
while the native resource remains owned. `Handler::closed` receives historical
state only, with no live target or handle. The contract is implemented by
[handle access](../src/registry.rs), [lease ownership](../src/lease.rs), and
[native closing](../src/winit_adapter.rs).

## Optional accessibility

The `accessibility` feature enables typed accessibility events and AccessKit tree
submission. The default feature set does not require accessibility types or
adapter details. Even with the feature enabled, capability permission and the
live-window lifecycle govern submission.

Initial-tree requests establish when a live window can receive a complete tree.
Active trees may accept incremental updates; deactivation requires another
initial-tree request and another complete update. Submission is queued callback
work, so callback failure prevents it from committing, and later planning or
application failure becomes terminal. See [the exact initial-tree procedure](how-to.md#submit-an-initial-accessibility-tree),
[Context](../src/context.rs), and the [typed rustdoc example](../src/lib.rs).
