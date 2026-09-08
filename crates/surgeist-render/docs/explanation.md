# Explanation

## Rendering boundary

`surgeist-render` owns rendering contracts, backend-facing draw data, scene
validation, GPU execution, surface resources, explicit headless readback, and
render diagnostics. Style interpretation, layout, text shaping, and application
and window lifecycle belong outside this rendering boundary.

The [current public API](../src/lib.rs) also exports style-facing authored values
and normalization inputs for backgrounds, decorations, image placement, masks,
clips, and filters. Those are existing contracts, not a claim that their
long-term authored-model ownership has been settled. The separately deferred
cross-crate ownership question does not change their current availability.

The root `surgeist` repository owns the public composition facade,
Surgeist-to-Surgeist lowering and adapters, browser-host integration, generated
API audit artifacts, and workspace wiring. This workspace crate owns its source
API and focused verification. Root artifacts are generated and updated by the
root tool; the crate carries no generated copies.

## Two GPU execution routes

Every successfully published frame reports exactly one `Stats::route`:

- `RenderRoute::DirectVello` uses one transaction-owned internal Vello raster
  pass for an effect-free scene.
- `RenderRoute::GpuGraph` uses crate-owned WGPU image/composite passes for
  resolved image alpha masks, supported blend/composition, and bounded backdrop
  filter lists with ordered color, blur, and drop-shadow operations.

Both routes remain GPU-only. There is no production CPU fallback, CPU effect
retry, implicit readback, or Vello-atlas re-entry for graph results. CPU pixel
algorithms exist as test-only quality oracles. Unsupported inputs or missing
device and surface capabilities produce typed errors instead of a different
pixel execution path.

## Precision is an explicit policy

Effect graphs prefer high-precision premultiplied numeric-sRGB `Rgba16Float`.
The default `EffectQualityPolicy::RequireHighPrecision` rejects a device that
cannot provide it. `EffectQualityPolicy::AllowReducedPrecision` permits
`Rgba8Unorm` only when high precision is unavailable.

The chosen precision is observable in `Stats::effect_precision`:
`EffectPrecision::High` or `EffectPrecision::Reduced` for a successful graph
frame, and `None` for a direct frame or before the first successful publication.
Neither quality policy permits CPU execution.

## Publication is failure-atomic

A render uses a transaction-owned command submission. Validation, allocation,
encoding, submission, device loss, cancellation, or presentation failure leaves
the previous complete publication and the renderer's last successful `Stats`
unchanged. A failed attempt can still affect runtime availability, but it does
not publish partial pixels or partial frame statistics.

`Renderer::stats` therefore describes the last successfully published frame.
Callers request CPU-visible pixels separately by awaiting
`Renderer::read_headless`, which reads the current complete headless publication
as tightly packed straight-alpha RGBA8 physical pixels. Keeping readback explicit
preserves the GPU-only render path.

## Host evidence has a boundary

Headless execution needs a GPU but no native window. Presented native rendering
uses the caller's live window lifecycle; the tracked smoke example owns that
lifecycle only for its two-frame demonstration. Display-free unit-test harnesses
exercise presentation contracts without proving a live graphical host works.

The `wasm32-unknown-unknown` leaf contract is compile-only with `render-web` and
`--lib --tests`. A browser host must supply the real canvas event loop and
presentation lifecycle. Successful compilation is useful leaf evidence, while
successful browser execution remains root integration evidence.

See the [reference](reference.md) for exact public entry points and host-feature
facts, or the [how-to guide](how-to.md) to exercise the supported host checks.
