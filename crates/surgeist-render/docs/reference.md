# Reference

## Package and public API

[Cargo.toml](../Cargo.toml) defines package `surgeist-render`, version `0.1.0`,
Rust edition 2024, and library import name `surgeist_render`. The public front
door is [src/lib.rs](../src/lib.rs); its reexports and their rustdoc own the
current source API.

| Area | Public entry points | Source |
| --- | --- | --- |
| Renderer and options | `Renderer`, `Options`, `Parameters`, `EffectQualityPolicy` | [Renderer](../src/renderer/mod.rs), [options](../src/renderer/options.rs), [surface parameters](../src/surface.rs) |
| Draw data | `Scene`, geometry, shapes, paints, images, layers, and positioned text runs | [Scene](../src/scene.rs), [public reexports](../src/lib.rs) |
| Placed Text consumer (`render-text`) | `prepare_text_layout`, `PreparedTextScene`, `TextLayoutLimits`, `TextLayoutError` | [Adapter](../src/text_layout.rs) |
| Surface and pixels | `Surface`, `SurfaceOptions`, `Attachment`, `ImageBuffer` | [Surfaces](../src/surface.rs), [images](../src/image.rs) |
| Support and diagnostics | `Capabilities`, `RuntimeCapabilities`, `Error`, `ErrorCode` | [Capabilities](../src/capability.rs), [errors](../src/error.rs) |
| Successful frame observations | `Stats`, `RenderRoute`, `EffectPrecision` | [Statistics](../src/stats.rs) |
| Frame and lifetime accounting | `RenderFrameReport`, `RenderCpuSnapshot`, `RenderDeviceSnapshot`, named scopes and typed intervals | [Frame accounting](#frame-accounting), [generation custody](#device-generation-custody) |
| Preparation accounting | `RenderAccountingContext`, `RenderAccountingOptions`, `RenderLimits`, `RenderAttemptReport`, named work/resources/phases | [Accounting](../src/accounting/mod.rs), [units and reports](../src/accounting/model.rs) |
| Current style-facing models | Background, border, outline, image-placement, mask, clip, and filter values | [Style module](../src/style/mod.rs), [public reexports](../src/lib.rs) |

The `Renderer` methods `new`, `create_surface`, `create_headless`,
`resume_surface`, `render`, and `read_headless` are asynchronous GPU operations.
`Renderer::read_headless` returns an
`ImageBuffer` containing tightly packed straight-alpha RGBA8 physical pixels from
the current complete headless publication. Rendering does not invoke this
readback implicitly.

## Capabilities and errors

`Capabilities::CURRENT` reports semantic support for authored rendering
operations and compiled surface availability. Its surface report supports
headless construction and enables its web-canvas flag only with both
`render-web` and the `wasm32` architecture. Native window attachment is separately
compiled behind `render-window`.

`Renderer::runtime_capabilities` reports availability or immutable facts about
the selected device and surface: surface format, supported effect precisions,
and maximum two-dimensional effect texture size. Cargo features select compiled
host adapters; enabling a feature does not establish runtime device support.

Unsupported semantic operations and unavailable runtime capabilities have
distinct typed diagnostics. Missing adapters, required formats, device limits,
and surface lifecycle availability do not authorize partial frame publication.
The [error API](../src/error.rs) defines the operation and reason payloads.

## Layer blending

`Layer::blend` accepts `BlendMode::{Normal, Multiply, Screen, Overlay, Darken,
Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion}`.
These separable functions combine corresponding straight numeric-sRGB source
and backdrop components. The result is bounded to the component range and
modulated by backdrop alpha before source-over composition. Transparent
backdrops therefore preserve the source contribution. Inputs, intermediate
textures and readback keep their existing alpha conventions; public readback
is straight-alpha RGBA8.

Color dodge returns zero for a zero backdrop before considering a source of
one. Color burn returns one for a backdrop of one before considering a source
of zero. Hard light branches on the source component at one half; overlay
branches on the backdrop. Soft light uses its polynomial curve through a
backdrop of one quarter, then its square-root curve. Each channel is independent.

`BlendMode::Plus` remains a separate additive composition choice. The layer API
does not select arbitrary Porter-Duff operators. Nonseparable blend functions,
background-layer blend interpretation and their typed capability boundaries
remain separate from this supported set.

Effect-free layers use the direct Vello route. A resolved alpha mask or other
supported graph requirement selects the GPU graph without changing the selected
blend function; nested capture-local layers can still use Vello raster spans.
Clipping, mask alpha and opacity attenuate the premultiplied source before
composition. Both routes retain the same Ledger admission, resource custody and
atomic publication rules as other layers. Host-observed profiling durations do
not establish pixel-value-independent execution timing.

## Image sampling

`Image::rendering` accepts an `ImageRendering` hint already selected by the
surrounding style owner. Callers resolve inheritance and apply that selection to
each applicable image. Render executes the selection for image draws, image
paints and resolved image alpha masks without inspecting an element tree.

| Hint | Current execution |
| --- | --- |
| `Auto` | Default bilinear sampling. |
| `Smooth` | Bilinear smoothing. |
| `HighQuality` | Bicubic smoothing. |
| `CrispEdges` | Nearest-neighbor sampling, preserving source colors. |
| `Pixelated` | Nearest-neighbor enlargement to the nearest positive integer multiple on each source axis, followed by bilinear scaling to the destination. |

Pixelated multiples use each transformed source axis independently, including
device scale. A half-integer scale selects the larger multiple, and downscaling
uses a multiple of one. Color images are enlarged by a GPU compute pass into
the image atlas. Alpha masks sample the equivalent virtual enlarged grid.
Both paths smooth premultiplied contributions; transparent source colors do not
bleed into visible output.

`Image::quality` remains available for explicit backend quality selection.
Selecting a rendering hint replaces that quality selection, and selecting a
quality clears the rendering hint. `Image::rendering_hint` reports the selected
hint. Pixel content and its compact fingerprint do not change, while complete
image equality includes the sampling selection.

There is currently no automatic quality degradation under resource pressure.
Checked atlas representation and runtime device limits can refuse a frame;
publication remains atomic. `HighQuality` therefore retains its selected
smoothing rather than silently falling back to another hint.

Image-atlas preparation uses `surgeist-ledger` to admit aggregate original upload
bytes and expanded texel work before packing and recording uploads. These are
defined operation costs, not allocator or process-memory measurements. The
existing resource manager continues to own GPU residency and cross-frame leases.

## Host features

| Configuration | Leaf boundary |
| --- | --- |
| Default features (`[]`) | Native headless GPU execution without a window. |
| `render-text` | Optional direct Text dependency, without Text default features; immutable placed-layout consumption, independent of window/web/accessibility adapters. |
| `render-window` | Enables `surgeist-window` and native window attachment; live presentation requires its graphical host lifecycle. |
| `render-web` | Enables `wgpu/webgpu`; web-canvas support requires `wasm32`. The documented wasm leaf check compiles `--lib --tests`. |
| `render-window,render-web` on native | Additive compiled features; uses the same native live-window smoke target. |

Native WebCanvas surface creation returns a typed unsupported-platform
diagnostic. Real browser event-loop execution and presentation belong to root
integration, even after a successful wasm compilation.

## Compatibility

This crate's [manifest](../Cargo.toml) declares edition 2024 and intentionally has
no `rust-version` field. The [root manifest](../../../Cargo.toml) owns the
facade integration floor.

## Glyph admission

`FontData::try_from_bytes` takes an original face index and caller-supplied
`surgeist_font::DecodeLimits`. Font admits raw SFNT or collection bytes and
decodes WOFF or WOFF2. Decode refusals remain available as the source of the
Render error. `FontData::from_face` takes an already admitted Font face.
`FontData::admitted_face` exposes its original index, decoded resource and
exact encoded source. These entry points are available in standalone Render.

`FontData::prepare_glyph_run` retains the selected representation and its font,
executed `TextInstance` and original `TextGlyph` facts. Its `PreparedGlyphRun`
provides representation-derived `Ink` or source-proved `Empty` bounds.
`Scene::prepared_glyph_run` shares those immutable resources with direct and graph
execution; glyph advances never substitute for ink.

Consecutive foreground outline operations share one NonZero fill, preserving
combined winding and opacity. An intrinsic COLR or bitmap selection separates
those spans even when its output is Empty. Intrinsic paints retain layer order,
clip and currentColor behavior. Caller paint applies to foreground operations.

The supported intrinsic profile is COLRv0, a COLRv1 solid/glyph/translation/static
clip subset, and sbix noninterlaced RGBA8 PNG containing only IHDR, contiguous IDAT
and IEND chunks. Other paint operations, formats and PNG chunks refuse explicitly.
Color and bitmap admission requires a default effective instance without
synthesis; supported outline variation and synthesis follow the policy below.
Malformed selected COLRv1 data cannot fall back to a COLRv0 representation.

Bitmap placement follows the pinned Skrifa 0.42.1 Skia/CoreText convention:
font-unit outer bearings and strike-pixel offsets establish the selected quad.
Drawing fills the retained canonical endpoint rectangle with its retained image
sampling mapping. PNG admission validates chunk order and CRC, exact zlib input
and output extent, Adler checksum and all five scanline filters before retaining
pixels. The inflater uses fixed-state miniz_oxide with caller-owned buffers.

`GlyphRunLimits` bounds glyphs, retained outline path elements, scratch storage
and retained logical custody. Charges include the retained Font resource,
defined fixed result units, actual buffer capacities, fixed provider workspace
and growth overlap. Font's separate `DecodeLimits` cover container admission.
These limits do not measure the allocator or bound CPU time. A failure discards
the entire unpublished result; the caller's inputs remain available for retry.
The authored `TextRun` route retains its existing provider behavior and supports
`Unspecified` bounds only on its direct route.

The optional `prepare_text_layout` adapter consumes the actual placed Text
snapshot without reshaping or choosing lines. Its complete `TextLayoutLimits`
budget also covers retained Font resources, mapping buffers and decoration
commands. The adapter retains each run's actual face and charges the resource
once per invocation, including when several faces use one collection. It
consumes each run's executed `GlyphFrame` with its physical origin, outline axes
and independent pen direction in outline coordinates.
Native upright runs keep their outlines unrotated and place each glyph at
`(origin.x + dx, origin.y + pen + dy)`. Combined cells retain horizontal inner
pens and the outline frame supplied by Text inside their vertical outer line.
Clockwise and counterclockwise sideways runs receive their exact right-angle
basis. The counterclockwise matrix is
`[0,-1,1,0,origin.x,origin.y]`; its raw pen is converted into the actual glyph
basis before applying that matrix. Existing outline synthesis operates in glyph coordinates
before that basis; caller placement follows once. Decorations are already
physical and receive caller placement once. Text owns admission of the actual
shaping profile; font, outline and numeric refusals retain their existing owners.

## Frame accounting

Configure `RenderAccountingOptions` through `Options::with_accounting` before
`Renderer::new`. Limits, profile mode and clock are fixed for that Renderer and
reach lifetime owners before native initialization. Defaults are checked-unlimited
limits with `RenderProfile::Off`. A frame uses three disjoint Ledger owners:

| Domain | Scope and admission |
| --- | --- |
| Fresh frame attempt | Actual forward work, including normalization, repeated visits, raster streams, graph/filter work, native requests/probes, uploads, encoded passes/copies, submissions and surface operations. Work limits restart for each frame. Finite repeat and atlas quotas apply separately to each local call. |
| Renderer CPU lifetime | Actual working and persistent backing, history drafts, shader keys/indexes, and imported font/image/prepared-run payloads. CPU limits bound simultaneous live backing, including old/new growth overlap; they are not a fresh byte allowance per frame. |
| Device generation | Descriptor-defined GPU custody, opaque native handles, cache classifications and mandatory lifecycle work. This owner survives Ready-state destruction and retires once; there is no caller-configurable active GPU allocation cap. |

Work units describe actual visits and invocations. Unordered source traversal can
vary comparison/probe totals between otherwise equal inputs; those comparisons
remain measured in their owning domain.
Resource probes include complete registered-entry validation, idle summaries and
repeated explicit entry lookups as well as candidate selection. Forward registry
audits spend the current attempt's allowance before inspecting each entry; a
refusal preserves registry and lease state. Mandatory return, abort, removal and
summary walks use generation-lifetime observations and continue after a spent
forward allowance. Opaque container-internal comparisons are not estimated.
Both image-atlas routes charge the selected source bytes and texels before
building the atlas. Ordinary residency follows the producer's Blob identity;
pixelated residency additionally distinguishes the selected integer expansion.
Repeated preparation rebuilds and charges that work even when content history
already recognizes the image. Local quotas renew independently of cumulative work.
Ordinary `Atlas` elapsed time includes the producer's indivisible glyph, ramp and
packed-stream resolution; it does not isolate image packing from that shared call.

Lifecycle/transfer units cannot be configured as work limits. Fixed Renderer
options accept the union of actual frame and [readback](#readback-accounting)
producers; each operation receives a fresh allowance. Standalone fronts reject
limits belonging to other operations. Mandatory releases and
rollback cannot be vetoed by an exhausted forward-work allowance. Reused native
resources make no construction request. Native requests are charged before the
constructor; generation custody observes returned provisional handles even when
later native error-scope validation rejects them.

On first poll, a render reserves its attempt/report and CPU/generation interval
storage before frame effects. An unpolled future starts nothing. Setup failure
preserves the previous report. `Renderer::last_attempt` borrows the latest started
`RenderFrameReport`; `take_last_attempt` moves it to caller custody. Terminal
status is `Succeeded`, `Refused`, `Failed(ErrorCode)` or `Canceled`, after inner
cleanup. `attempt`, `renderer_cpu` and `generation` expose distinct typed closing
intervals. A domain whose extraction failed returns its typed fault;
already closed intervals keep their real boundary, including when later rollback
occurs. A cleanup fault remains separate from the causal diagnostic.
Accounting errors retain their typed cause without allocating during construction.
`Error::message` formats and caches human-readable text on first observation;
typed diagnostic access and Display formatting leave that cache uninitialized.

`Renderer::cpu_accounting_snapshot` observes current Renderer CPU lifetime usage.
`device_accounting_snapshot(None)` observes the existing default generation;
passing `Some(&surface)` selects that Renderer's owned surface generation. These
read-only operations neither select a new adapter nor retire a device, replace
the latest attempt or require a renderable generation. They expose initialization,
current custody and retained terminal state without an archive of old generations.
Snapshots provide exact root usage with profiling Off; scoped lifetime usage
requires an enabled profile. Frame intervals retain exact scoped usage in all
profile modes.

`Summary` and `Detailed` optionally collect host elapsed time using Ledger's
platform clock or `RenderClock::Host`. Timed phases surround actual work and
awaits, including pending preparation, submission resolution and surface
acquisition/presentation. Off and untimed modes perform no clock reads. Detailed
event limits apply independently to each owner and interval; phase/domain filters
select detail only. Lost events do not alter authoritative usage, peaks or
outcomes. Owner lifetimes include idle gaps; phase durations are inclusive and
can overlap. Do not add parent/child durations, peaks or the three domains, or
interpret these durations as GPU hardware execution timing. Persistent cleanup
and timed snapshots can observe the clock after a frame publishes. A host clock
must remain monotonic, nonpanicking and free of observable side effects.

Successful `Stats` projects normalized emissions and actual encoded native work
from the attempt, plus cache allocation/reuse work and idle classification from
the generation interval. Counts use `u64`. `frame_time`, `preparation_time`,
inclusive `execution_time`, `presentation_time` and `surface_acquisition_time`
are `Option<Duration>`, absent when untimed. `texture_copy_operations` counts
native texture copies rather than graph nodes. `image_passes` includes actual
image-processing shader draws and attachment-clear passes. `queue_upload_bytes`
counts actual queue-upload input bytes. Exact-content `image_content_hits`,
`image_content_misses` and `first_observed_image_bytes` describe this frame's
observations against Renderer history, independently of debug diagnostics.

CPU custody measures actual Vec/String capacity bytes, explicit separately
heap-owned state and known producer payloads, including image-expansion and
font-backing index records. Known heap-owned font and image adapter payloads are
counted separately from opaque Arc/Blob headers. Each image Blob adapter owns a
known `Arc<[u8]>` payload: Renderer imports of clones share its admission, while
distinct adapters over one pixel allocation have separate admissions, including
zero-byte images.
History-only pixel identities retain no adapter. Pixel backing deduplicates by
Arc allocation identity, fonts by producer `ResourceId`, and prepared-run allocations
by their canonical Arc identity. Run-owned bytes are separate from shared
font/image payloads. Inline structs, allocator headers/buckets, Ledger/control
storage, dependency-private scratch and process/RSS usage are excluded. Native
device-state, manager and handle-registry infrastructure is also excluded; its
nested source/key/index Vecs and retained payloads remain measured.
Temporary text values and resource iterators retain their backing admissions
through actual destruction inside the owning CPU disposal phase, including
early failure and cancellation.
The bounded raster adapter admits replacement capacity for its eleven public
Encoding streams before producer calls. The dependency-owned mask lookup
constructors offer no fill API: their known 1024/8192-byte payloads are previewed,
then actual returned capacity is admitted before retention or upload. Their
infallible allocation and private scratch remain an explicit dependency boundary.

Exact-content history uses measured sorted records and preserves canonical
baseline identities. A draft accounts for baseline/new records and simultaneous
backing. Finalization can dispose old storage before report closure; a later
failure removes new records in place without allocating, preserving previous
logical history while charging any larger surviving capacity. All fallible
finalization and statistics projection precede successful metadata/headless
publication. The interval may still include old/new headless textures; dropping
the old publication is later generation-lifetime release. Host presentation may
already have occurred before a later fault resolves, so this is not physical
display rollback. Explicit readback has its own report and final caller-transfer
boundary, described below.

## Readback accounting

Awaiting `Renderer::read_headless` starts a separate readback attempt on first
poll. An unpolled future starts nothing. Report and CPU/generation checkpoint
storage is reserved before native effects; setup failure preserves the previous
readback report. `last_readback_attempt` borrows its latest started
`RenderReadbackReport`, and `take_last_readback_attempt` moves it to caller custody.
These operations preserve the latest frame report, last-successful `Stats`,
canonical image observations and complete headless publication.

Available zero-area surfaces return validated empty output without native work.
Nonzero surfaces without a publication return the existing typed uninitialized
error; rejected requests produce started-attempt outcomes without fabricated
copy, submission or mapping work. A generation interval is present only after a
generation is associated with the nonempty readback.

The attempt admits real native constructor requests through the existing
generation producers, followed by actual copy, submission, mapping and row work:

| Unit | Meaning |
| --- | --- |
| `ResourceAllocationRequests` | Staging-buffer, command-encoder and command-buffer constructors actually invoked. |
| `ReadbackCopies` | Encoded texture-to-buffer copies; texture-to-texture `CopyOperations` remains distinct. |
| `Submissions` | Actual queue submissions. |
| `ReadbackMapRequests` | Actual native mapping requests registered. |
| `ReadbackRows` | Source rows inspected, including an incomplete row that fails validation. |
| `ReadbackBytes` | Logical unpadded RGBA8 bytes actually copied into output. |

Each row is admitted before inspection, and its logical payload before the copy.
Refusal or malformed rows discard the complete private output, retaining only
actual accepted-prefix work. There is no CPU alpha-conversion loop: readback
copies the rendered straight-alpha pixels. Native map internals and driver
storage are outside these observable units.

Output uses fallible Vec construction with actual returned capacity under
`RenderCpuScope::ReadbackOutput`. Its admission remains paired with the data
through ImageBuffer validation, final device validation and report extraction.
Renderer CPU caps cover this allocation together with existing lifetime backing;
they do not reset for each readback. Failure and cancellation destroy owned data
before mandatory release. Callback/helper owners carry only generation custody,
never a readback attempt. A callback can still retain staging after the report
closes; late delivery, destruction and retirement cannot change that report.

Successful report intervals close **before final caller transfer**, with complete
output capacity still live. The final CPU-lifetime atomic batch records
`ReadbackTransferredBytes` for that admitted capacity and releases its custody
without destroying the data. Rejection commits neither change, disposes the
guarded output and records the precise failed outcome. Already-closed intervals
retain their earlier numbers; later disposal appears in lifetime observations.
After a successful batch, only infallible report storage and the plain
ImageBuffer move remain. Caller values and clones carry no hidden participant.

For 3×2 RGBA8, two row visits copy 24 logical bytes from a 512-byte padded staging
descriptor. If the Vec's actual capacity is `C ≥ 24`, the successful CPU interval
closes with `C` output bytes still live and zero transfer work. Subsequent CPU
lifetime usage records `C` transferred bytes and releases that output custody.
Transferred capacity is distinct from logical payload and is not deallocation.

Ledger's `Readback` phase covers the body through guarded pretransfer completion.
`ReadbackPrepare` includes native construction/copy encoding and its
`ReadbackSubmit` child; `ReadbackResolve` measures actual awaited error scopes.
`ReadbackMap` includes registration, Pending waits and completion, including its
`ReadbackDecode` child. `ReadbackCleanup` times staging-owner disposal; the CPU
owner times output allocation destruction. These are
inclusive host observations, not GPU hardware timings or additive durations.
Intervals and timing end before final transfer. Off/untimed modes read no clock;
bounded detail loss or filtering does not alter usage, limits or outcomes.

## Standalone accounting

`Scene::inspect` requires a mutable `RenderAccountingContext` and returns a
`SceneSummary` of nested authored commands, fills, strokes, shadows, images,
glyphs and layers. Its `Authored*` units differ from normalized `*Emitted` work;
actual repeated traversals charge visit units separately. Image-content
observations are local to that inspection, not Renderer history.
Inspection stores borrowed image observations: its scratch backing is measured,
while `ImageIdentityDrops` remains zero because no owned retained-identity record
is created or destroyed. That unit covers owned history/identity records rather
than borrowed slots or ordinary Image clones.

`ImageRepeatPlan::resolve`, `BackgroundNormalizationInput::normalize`,
`ClipInput::normalize` and `BoxDecorationInput::normalize` likewise require the
context. Nested work uses the same participant and local quotas. Actual
scratch/output backing is admitted; only complete successful output transfers to
caller custody. Pure construction, validation and constant-size
`ImagePlacementInput::resolve` remain context-free. Reuse the context to change
options for the next call, inspect `last_attempt`, or move its report with
`take_last_attempt`. Returned values and later caller clones do not retain its
participant.

## Preparation accounting

`FontData::prepare_glyph_run` and `prepare_text_layout` take a mutable
`RenderAccountingContext` after their existing limits. Reuse the context to
change `RenderAccountingOptions` for the next call, inspect `last_attempt`, or
move its report to caller custody with `take_last_attempt`. This is a deliberate
source API change; every preparation path requires the context.

The measured boundary is standalone glyph and placed-Text preparation, including
outline/variation qualification, provider callbacks, intrinsic color/bitmap
processing, PNG decoding and coordinate conversion. Scene/GPU work dimensions
remain zero in preparation reports, and configuring them for a preparation
operation returns `InvalidLimit`. Text-only dimensions also
require the `render-text` feature. Renderer, device-generation and
readback accounting are outside this preparation contract.

| Preparation work | Unit and boundary |
| --- | --- |
| `GlyphInputs`, `CoordinateInputs`, `TextRuns`, `TextDecorations` | Original glyphs, normalized coordinates, nonempty runs and decoration records accepted once. Whole-Text preflight owns admission; nested glyph runs never accept these again. |
| `GlyphVisits`, `FontRecordVisits`, `CoordinateVisits` | One actual glyph fact, directory/instance record, bitmap strike/index record or scalar/axis record inspected. Repeated traversals charge separately; early exits retain only the visited prefix. |
| `PathElementVisits`, `OutlinePointVisits`, `VariationTupleVisits`, `GlyphComponentVisits` | One raw contour endpoint/provider path element, point-flag record/point/delta, variation tuple or composite child edge inspected. Dependency internals are not reported as Render visits. |
| `PaintVisits`, `ResourceProbes` | One raw COLRv1 paint node, consumed recorded layer/retained paint (including repeated placement qualification), or shared-font identity comparison. |
| `PathElementsEmitted` | One retained glyph path element appended, including actual synthesis and clip expansion. This is the existing path-element limit unit. |
| `SerializedBytes`, `PngByteVisits`, `ImageTexelVisits` | IDAT payload bytes copied; chunk type/payload bytes processed by CRC plus scanline bytes unfiltered; decoded RGBA pixels inspected for transparency. |
| `PreparedTransferredBytes` | Explicit retained backing units transferred only when the complete prepared result returns successfully. This dimension cannot be caller capped. |

`RetainedCpuBytes` covers actual Vec capacities multiplied by element size,
font resources using Font's admitted retained-byte unit once per `ResourceId`,
immutable image payload lengths, and explicit fixed result units. A glyph run's
fixed unit is `size_of::<PreparedGlyphRunData>()`;
a Text scene's is `size_of::<PreparedTextScene>()`. Opaque Arc/header estimates
are excluded from both result units.
`ScratchCpuBytes` covers actual live qualifier/provider/coordinate/mapping/PNG
buffer capacities and the fixed inline miniz decoder state and its declared
Huffman arrays. Arc allocator headers, hash-table bucket overhead, dependency
internal allocations and process/RSS usage are outside these units.
The fixed accounting participant, bounded reserved report/profile storage and
typed error/control-plane storage are also outside admitted preparation backing.
Failure reporting therefore does not recursively consume an exhausted output cap.

Capacity growth previews the requested replacement without committing usage,
reserves fallibly, admits its observed actual capacity while the old backing is
still live, then drops and releases the old backing. Reservation failure creates
no acquired peak. All temporary holds release at drop. Failed results discard
their retained output before closing custody; successful complete outputs transfer
their admitted backing to the caller. Transfer is not deallocation. Subsequent
caller clones and output lifetimes have no hidden accounting callbacks.
A preview refusal is retained in the attempt status and typed refusal; it does
not create a Ledger admission event or update admission observations.

The fixed tree has `Attempt -> Preparation`, with `TextPreflight`, `GlyphPrepare`,
`Coordinates`, `Output` and `Cleanup` below Preparation. GlyphPrepare has
`Qualify`, `Outline`, `Color` and `Bitmap`; `Png` is below Bitmap. The general
limits apply at Attempt, and existing finite glyph/Text limits at Preparation.
They therefore cover all Text runs, preflight and glyph phases together, without
remaining-limit arithmetic. CPU resource custody originates at Preparation;
work and elapsed phases retain their named origin. Phase usage is inclusive:
never add child values to ancestor values. Phase durations may overlap and do
not sum to wall time or measure GPU execution time.

Defaults are checked-unlimited general limits with `RenderProfile::Off`. Off and
untimed profiles perform no clock reads. Timed profiles use Ledger's platform
clock or an injected `RenderClock::Host` callback; unavailable timing refuses
setup. Detailed profiles require an explicit event bound and optional phase
filter. Filtering and dropped detail never change authoritative totals or peaks.

One report slot retains the latest started attempt as `Succeeded`, `Refused`,
`Failed(ErrorCode)` or `Canceled`, with exact accepted-prefix usage, cleaned
closing balances and first typed accounting refusal. All bounded report storage
is reserved before starting; setup failure preserves the previous report. Legacy
glyph/Text cap refusals preserve their typed resources and actual item locations.
Nested Text path refusals report whole-attempt required/limit values: two
five-element runs under a cap of nine report `required: 10, limit: 9` at the
second run, in place of the former remaining-run values `5, 4`.
These attempted reports are separate from last-successful-frame `Stats`.

## Device-generation custody

Every native device generation creates one Ledger before engine and shader
initialization. It remains available after Ready state is dropped, and terminal
transition or device-state drop retires it once. Work is checked-unlimited;
Renderer's selected profile and clock apply from initialization onward. Typed
generation intervals and `RenderDeviceSnapshot` expose its usage separately from
fresh frame work and Renderer CPU lifetime custody.

`GpuBytes` measures descriptor-defined buffer lengths and texture payload sizes
for observed handles returned to Render, including provisional handles awaiting
checked error-scope validation. It does not prove successful driver allocation,
resident memory or allocator overhead. Native errors can arrive after a handle
has been returned; rejected provisional owners still release custody exactly
once. `ResourceAllocationRequests` counts actual constructor invocations,
`ResourceAllocations` counts returned descriptor-bearing handles, and
`GpuObjectsCreated` counts returned opaque handles/explicit utility aggregates.
A refused noncommitting preview invokes no native constructor and changes no
usage or peak. Setup failure also leaves a healthy generation retryable.

| Fixed generation scope | Observed ownership |
| --- | --- |
| `render.device -> cache -> vello / effects / masks / kernels` | Managed buffer/texture descriptor payload, cache registration, idle classification, allocation/reuse/probe/return/eviction work. |
| `render.device -> external -> surface_targets / pass_buffers / readback_staging` | Headless/presented target textures, pass operation/parameter buffers, and padded staging-buffer descriptors. |
| `render.device -> objects -> shader_module / bind_group_layout / pipeline_layout / compute_pipeline / render_pipeline / sampler / bind_group / texture_view / command_encoder / command_buffer / dependency_utility` | One observed opaque handle per owner; short-lived shader modules/layouts/encoders release at their actual consumption/drop. |
| `render.device -> retirement` | Once-only terminal retirement work, after custody cleanup. |
| `render.device -> initialization` | Actual awaited native engine initialization. |

These are fixed named scopes, including the root and grouping scopes. Attribution is
inclusive: ancestor and child quantities must never be added. `OpaqueGpuObjects`
uses one unit for each explicitly owned handle, and one aggregate unit for the
opaque `wgpu::util::TextureBlitter` dependency utility. Its internal objects and
per-copy creations are outside that aggregate observation; no primitive count is
inferred from dependency implementation details. Imported swapchain textures,
device/queue anchors, dependency/driver internal allocations and internal handle
retention are outside these descriptor units. Render-created views of imported
swapchain textures are observed opaque handles. Borrow-scoped render/compute pass
recording guards remain within command-encoder custody; they are recording phases,
not additional standalone native allocation units. Actual pass/dispatch work is
charged to the surrounding frame attempt at native encoding.

Native clones share a single physical hold. Explicitly created texture views also
share their known parent texture hold, so dropping the texture wrapper while a
view survives does not prematurely release its descriptor payload. This tracks a
direct relationship established by Render's factory; it does not infer a general
WGPU dependency graph. Existing resource leases and pass/submission owners retain
their source buffers/views through their observed use. Native values drop before
their shared custody token, and native constructors, Drop callbacks, submission,
and awaits run without a Ledger/lifecycle lock. Driver retention after the last
explicit Render owner ends is outside the measured domain.

`RegisteredCacheGpuBytes` classifies payload represented by current managed
entries; `IdleCacheGpuBytes` classifies registered entries eligible for idle
retention. Neither is additional physical storage. Removing registration clears
those classifications while a surviving native clone/view can keep physical
`GpuBytes` live. Cache retention compares the target with all physical cache bytes,
including active and idle resources, and evicts only idle entries. The target is
an idle-retention policy, never an active-GPU allocation cap.

Exact entry-set/identity checks independently validate registration against the
Ledger. Committed transition/invariant failures preserve the first health fault
and block new work. Mandatory release remains nonpanicking and cannot be vetoed
by cleanup work-counter overflow. Retirement releases each authoritative leaf
and its projections once; surviving owners and late readback callbacks then
release harmlessly and cannot acquire into the retired generation. Readback
cancellation retains staging custody until its native map callback is delivered
or destroyed, rather than releasing while a callback-owned buffer clone survives.

One outer public-render checkpoint spans direct or graph execution, nested cache
lease scopes and cache cleanup. Successful publication projects `cache_allocations`
and `cache_reuses` from the cache subtree's interval work, and `idle_cache_bytes`
from its closing classification. These `u64` fields replace
`effect_texture_allocations`, `effect_texture_reuses` and `retained_effect_bytes`;
this is an intentional source API change.
A cold direct Vello frame now reports its real managed allocations. Failed or
canceled attempts preserve prior published headless pixels and successful
metadata/`Stats`. Host presentation may already have occurred before a later
backend error resolves; this does not promise rollback of the physical display.

If a poisoned registration balance underflows during mandatory cleanup, a later acquisition
can now return `ErrorCode::Accounting` with the exact first
`GenerationRefused { quantity: RegisteredCacheGpuBytes, current, amount, reason:
Underflow }` diagnostic before invoking another native constructor. The original
manager identity/fault diagnostic remains available and unchanged.

The interval closes before the nonfallible surface/publication commit. Its
closing physical custody may therefore include both the previous published
headless texture and the new draft; replacing/dropping the old publication is a
later generation-lifetime release outside that interval. The three cache fields
already follow cache cleanup and do not claim a complete postpublication frame
resource report. Generation timing is independent of frame elapsed time; the
attempt supplies successful-frame duration and semantic work projections.

## Outline admission

The outline branch of `FontData::prepare_glyph_run` retains one canonical path
for measured ink and GPU drawing. Static glyf composites admit signed XY translations, including
explicit identity transforms, complete opaque unhinted programs and first-record
overlap flags. Their full original child closure is checked before provider draw;
repeated children retain every occurrence. Provider depth32 and real expanded
points65531 (plus four phantom points) are actual provider boundaries. Empty loca
children contribute no real points to metadata, but their actual draw-time load
still consumes a recursion level. Cycles, truncated fields and
malformed simple extents refuse as invalid font data.

Composite variation, point attachment, nonidentity transforms, metric overrides,
unknown flags and nonzero composite residual bytes remain unsupported. These are
supported-domain limits, not assertions that every refused font is malformed.
Simple residual bytes after fully proved consumed extents remain uninterpreted.
Valid simple encodings whose complete flag prefix exceeds their logical point
count refuse as `Provider`: the installed decoder cannot faithfully consume them.
A repeat count of zero is accepted when the complete prefix fits that domain.

Existing storage limits include live qualifier buffers and their growth overlap,
then the provider's actual draw-buffer capacity after qualifier disposal. They do
not bound CPU time. All failures discard the unpublished run, while the original
font, instance and glyph inputs remain available for retry. Other outline formats and
sparse IUP retain their explicit capability refusals. Concrete
positive bold appends a local-x copy before canonical point conversion under the
[selected policy](../../../docs/text-layout-contract.md#concrete-outline-synthesis);
a displacement absorbed by conversion refuses with a synthesis numeric error.

## Verification sources and commands

The focused tests are grouped in [src/tests/mod.rs](../src/tests/mod.rs), covering
models, authored-style normalization, frame planning, GPU passes, Vello raster
execution, surfaces/readback, and platform contracts. The [Ahem
fixture](../tests/fixtures/fonts/ahem/PROVENANCE.md) provides deterministic glyph
inputs. CPU pixel reference algorithms are compiled only for tests.

The repository command inventory is owned by [AGENTS.md](../AGENTS.md). With
dependencies already present, its standard Cargo commands can run offline:

```sh
CARGO_NET_OFFLINE=true cargo check -p surgeist-render
CARGO_NET_OFFLINE=true cargo test -p surgeist-render
CARGO_NET_OFFLINE=true cargo clippy -p surgeist-render --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo fmt -p surgeist-render --check
```

Tests include real GPU work. No passing result is implied when the required
adapter is unavailable. The [how-to guide](how-to.md) owns the live native
presentation and wasm compilation procedures; the [getting-started
guide](getting-started.md) selects one focused headless proof.
