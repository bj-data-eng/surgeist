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

## Platform monotonic clock

Use `surgeist_window::Instant` for draw deadlines and input receipt timestamps.
Every Window scheduler, callback scope, proxy and adapter uses this same clock;
`std::time::Duration` remains the elapsed-time unit. Native targets retain exact
`std::time::Instant` identity, so native callers pass their existing instants
directly without conversion.

On browser Wasm targets, the shared clock is the host context's
`Performance.now()` clock supplied by the pinned web-time implementation. This
also matches winit's browser deadline type. It requires a `Performance` object;
contexts such as worklets that lack one are unsupported. The clock makes no
promise to advance while a browser context is asleep or suspended.

Create deadlines in the owning event loop's clock domain. The upstream clock
adds `Performance.timeOrigin` for atomics-enabled workers; without that mode,
independent contexts do not establish a shared epoch. Window neither rebases
native and browser instants nor converts them through an elapsed duration.
Browser timestamps and deadlines use the browser type, rather than accepting
`std::time::Instant` as a second clock.

Window receipt times are host observations, not DOM performance timestamps.
Runtime retains ownership of its separate DOM epoch, source identity and
coarsening policy. Selecting the Window clock changes none of those contracts,
and does not establish browser support for unrelated consumer clocks.

The [scheduler](../src/scheduler.rs), [input payloads](../src/event.rs) and
[native deadline mapping](../src/winit_mapping.rs) consume the public clock.
Installed-target compilation qualifies their type agreement; it does not prove
browser event-loop wakeup or runtime behavior. See the
[verification boundary](how-to.md#verify-the-platform-clock).

## Native magnification input

Handle `InputEvent::Magnification` in `Handler::input` to receive native
magnification intent. A `MagnificationGesture::Pinch` retains its native
`TouchPhase` and raw signed fractional delta: positive means magnifying intent,
negative means shrinking intent, and either signed zero remains unchanged.
`MagnificationDelta` admits only finite values. A nonfinite native delta becomes
`None` while preserving the pinch phase; it never becomes an invented zero or
scale factor. `MagnificationGesture::Smart` is a distinct request with no
supplied delta or phase.

The event's optional position is the last observed logical mouse position,
which can be absent. It is pointer context, not a gesture focal point. Native
gesture payloads supply no coordinates. The adapter captures current modifiers
and a monotonic receipt timestamp, preserving the backend's source order,
including preceding pointer updates.

Receiving this input does not change window metrics, native display scale,
page zoom, visual viewport scale, host observations, layout or CSS, and emits no
window command. The consumer chooses whether and how to apply magnification or
restore an earlier view. This keeps native input separate from the
[CSS2 magnification guidance](../../../references/css2--ui.html--ff1f72803ea0.md#magnification),
which defines no authored zoom property.

The pinned winit backend emits pinch and smart-magnification gestures on macOS
and iOS when the native platform provides them. iOS recognition requires an
explicit opt-in that this adapter does not enable. Other backends have no
current producer for this input. The [native adapter](../src/winit_adapter.rs)
owns lowering; [focused tests](../src/magnification_tests.rs) exercise conversion
and the shared callback pump without a display or an actual native gesture.

## Host observations and consumer inputs

`WindowSnapshot::environment` retains a `HostEnvironment` with the window's
observed geometry. A native resize, scale or move transition commits its basic
environment using the event's metrics before delivering the corresponding callback. `EnvironmentChanged`
reports changes that do not need another event kind. The native adapter observes
the system theme independently of an application `SetTheme` request, so an
application override does not become the baseline user preference.

A changed metrics or position basis invalidates captured geometry and output
facts unless the transition supplies a fresh environment. Independent baseline
preferences, input/navigation facts and presentation mode remain captured.
Unchanged geometry preserves the existing capture. This rule also governs
deterministic commands and planner projections; they report unavailable facts
until a host supplies a new capture instead of inferring viewport policy from
a requested size.

Native display scale, page zoom and visual viewport scale have separate values.
The current native content host uses page zoom one and has no retractable UA
chrome, so its large, small and dynamic viewport sizes coincide. Hosts supplying
their own observations can retain distinct viewport variants and visual
magnification. Layout-dependent initial containing block, scroll offsets, paged
area and root scrollbar facts remain explicit inputs from their owning consumer.
The Web canvas adapter leaves browser viewport variants, page zoom and desktop
origins unavailable: a canvas's viewport-relative position and dimensions do not
establish those browser or desktop facts.

An unavailable observation is distinct from a known absent device or a known
`no-preference` value. The winit adapter supplies monitor and client geometry,
display scale and the system theme where its platform API supports them. It does
not infer primary pointer capabilities from the most recent input event or guess
navigation, reduced-data or output-blending facts. Those remain unavailable
until a host supplies them. Windows supplies the native input observations
described below. CSS matching, document activity,
origin preference overlays and coherent downstream publication remain with
their consuming owners.

On Windows, the native content host samples the complete Windows Runtime
[`PointerDevice`](https://learn.microsoft.com/en-us/windows/apps/develop/input/identify-input-devices)
collection at creation and existing safe idle opportunities. Its host policy
chooses a mouse or touchpad first, a pen second, and touch third. Mouse and
touchpad report fine accuracy and convenient hover; pens report fine accuracy
with hover from `MaxPointersWithZDistance`; touch reports coarse accuracy without
convenient hover. Among pens, a hover-capable pen takes precedence. Selection
does not depend on enumeration order or the last input event. The union retains
every device, so coarse and fine can both be true. A successful empty collection
reports known no-pointer; any failed query or unknown device type leaves the
complete input report unavailable rather than silently dropping a device.

The Windows API has a documented
[package-identity restriction](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/winrt-api-desktop-app-support#apis-that-require-package-identity).
The adapter attempts the observation and preserves failure as unavailable; a
successful native probe on an unpackaged desktop does not promise universal
deployment support. Native objects stay on the sampling thread; snapshots retain
only Rust capability values. Basic geometry events preserve committed input.
Native creation and idle refresh explicitly own the native input sample, while
other adapters retain ownership of supplied input, navigation and presentation
facts. No input-device subscription, independent wakeup or hotplug delivery
latency is promised.

On macOS, the native adapter samples AppKit's reduced-motion,
reduced-transparency and increased-contrast flags. A true reduction flag maps to
`Reduce`, and false maps to `NoPreference`. Increased contrast maps to `More`
when true and `NoPreference` when false; this flag does not detect `Less` or
`Custom`. These are explicit host-policy mappings of observed settings.

On Windows, safe Windows Runtime APIs sample `UISettings.AnimationsEnabled`,
[`UISettings.AdvancedEffectsEnabled`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.uisettings.advancedeffectsenabled)
and [`AccessibilitySettings.HighContrast`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.accessibilitysettings.highcontrast).
Disabled animation or transparency effects map to `Reduce`; enabled effects map
to `NoPreference`. An active contrast theme maps to `Custom`: this observer does
not classify the selected palette as particularly high or low contrast. An
inactive contrast theme maps to `NoPreference`. Each failed observation remains
independently unavailable, preserving successfully sampled siblings and the
separately supplied system color scheme. These APIs do not establish reduced-data
preferences. Other platforms retain unavailable values for these observations.

The complete preference baseline is sampled at resume, native/user events,
command application and idle opportunities. Changed facts commit before their
callbacks, including accessibility changes with an unchanged system color
scheme. Application theme and origin overrides remain separate. The adapter
installs no preference notification observer or wakeup, so an external setting
change does not promise to wake an otherwise idle loop.

On macOS, creation and safe idle capture also observe the current monitor's
available work area and WindowServer color-depth estimate. A nonzero winit native
monitor ID must match exactly one checked `NSNumber` under the SDK's
`NSScreenNumber` key. Missing, invalid, unmatched or ambiguous identity leaves
these facts unavailable; the adapter never selects a screen by its name, frame,
main-screen status or enumeration order.

The work area uses `visibleFrame` relative to that screen's own `frame`. It
converts the Cocoa bottom-up offset to the captured screen's right/down axes,
scaled by screen backing scale divided by the committed metrics scale. No
primary-screen height or additional safe-area subtraction participates.
The color estimate multiplies positive checked sample bits by color-component
count, excluding alpha and storage padding. Recognized grayscale retains its
total estimate without inventing a positive RGB media-feature bound; unknown
color-space semantics retain the public fallback. This estimate does not certify
renderer formats, HDR, or the complete media-device capability environment.

On Windows, safe WinSafe APIs enumerate native monitors and match the selected
winit monitor's device name exactly against `MONITORINFOEX.szDevice`.
[`GetMonitorInfo`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow)
supplies that monitor's screen and work rectangles in virtual-desktop units.
An empty, missing or ambiguous device identity, an incomplete native read, or
invalid work bounds leaves the observation unavailable. No primary-monitor,
enumeration-order or geometry-based fallback chooses a device.

Windows work rectangles already use top-left/right/down axes. Their offsets
within the selected monitor are divided by the committed display scale and
added to the captured logical screen origin. This preserves negative desktop
origins and each window's scale. Work bounds must be finite, nonnegative and
contained in a positive-area selected screen.

Windows color capture uses a synchronous read-only display-state query. The
selected device's fully enumerated PnP monitor interface must match exactly one
fresh connected `DisplayTarget.DeviceInterfacePath`; multiple child interfaces
remain ambiguous. The display state and selected target must remain fresh
through the read. The snapshot is acquired before resolving the GDI mapping, so
its freshness interval covers interface enumeration and wire queries. Failed
queries, invalid identities, missing wire formats or
unknown encodings leave color unavailable without discarding valid work bounds.
The native display manager is closed after every query; no display ownership or
configuration mutation is requested.

For RGB444 wire formats, the observed positive channel precision gives a
three-channel estimate and the matching RGB component bound. Alpha and storage
padding do not participate. Other encodings remain unavailable. This is an
active wire-format estimate, not a renderer format, panel precision or HDR
qualification. Pixel-storage depth is never substituted for that observation.
See the native contracts for
[monitor interface identity](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaydevicesw),
[target identity](https://learn.microsoft.com/en-us/uwp/api/windows.devices.display.core.displaytarget.deviceinterfacepath)
and [wire channel precision](https://learn.microsoft.com/en-us/uwp/api/windows.devices.display.core.displaywireformat.bitsperchannel).

Raw native event captures retain the existing basic geometry but defer these
monitor facts. At `about_to_wait`, after queued metric transitions have
committed, the adapter samples preferences once and captures each live window's
complete native environment. One changed callback per window carries combined
preference/output/input changes; equal samples are suppressed. Navigation and
presentation facts, and input outside the native Windows provider, remain
independently supplied. Refresh returns work
to the existing callback pump and adds no notification subscription, wakeup or
latency guarantee. See [the monitor observation owner](../src/native_monitor.rs).

## Native geometry admission

`NativeGeometryRequest` resolves a captured `WindowSnapshot` and an explicit
`NativeGeometryBasis` under a native host authorization. The basis names the
selected screen origin and logical units per CSS pixel; it does not silently
substitute display scale or page zoom. Admission can allow, deny or clamp a
request. A successful resolution produces an existing `Command` for ordinary
normalization, capability planning and application. Denied, unchanged and
unavailable geometry have explicit no-action outcomes.
Metric changes refresh native outer geometry where the host can observe it.
An unavailable fresh outer extent stays unavailable, so clamp admission cannot
mistake a prior window size for the current decoration extent.

Absolute movement uses the selected output-device origin; open features use
the selected exposed-screen origin. Relative movement uses the
captured outer position. Absolute resizing requests viewport dimensions;
relative resizing adds to the captured viewport size. A containment clamp uses
observed decoration extents and declines when they are unavailable or
inconsistent. An approved command expresses intent and does not prove the OS
accepted it. Resolution keeps the supplied capture even if the window changes
before command application.

`NativeOpenFeatures` consumes already-tokenized `left`, `top`, `width` and
`height` features. It uses HTML integer parsing, with zero for a syntax error and
independent zero-dimension skips. HTML owns feature-string tokenization. This
native admission policy does not assert that a native window satisfies the
browser's script-created auxiliary-context predicate; JavaScript bindings remain
outside this implementation.

`PointerEvent::native_coordinates` retains fractional client positions and uses
the captured client-area origin to obtain desktop and screen-relative positions.
Missing positions and origins remain unavailable. These values use native
logical units; DOM dispatch state, document scroll and target padding geometry
are needed separately for complete CSSOM View page and offset coordinates.

## Native UI fonts

`SystemFontEnvironment::capture` observes actual macOS UI fonts on the main
thread through safe AppKit APIs. The result retains family and face names, native
weight and traits, point size and vertical metrics. Unsupported platforms and
wrong-thread calls have explicit errors. Consumers own CSS-pixel conversion,
CSS shorthand resolution and selecting the matching text font resource; native
metrics do not replace shaped-text metrics.

The selected macOS role policy uses the system UI font for caption, icon and
message-box roles; the menu font for menu; the small system size for small
caption; and the system UI font at label size for status bar. This is a host
policy for obtaining real preferences, not a new CSS grammar.

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
