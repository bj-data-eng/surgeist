# Native diffuse lighting

`prepare_lighting` takes a resolved diffuse material, exactly one distant,
point, or spot light, and a filter working color space. It validates the
reached inputs and returns a reusable `PreparedLighting`. A
`LightingSource::finite` owns the bump-map Scene and its `FilterDomains`.
`Scene::lighting_layer` consumes that source and installs lighting as the
layer's foreground effect.

```rust
use surgeist_render::{
    accounting::RenderAccountingContext,
    effect::{FilterColorSpace, FilterDomains},
    geometry::Rect,
    lighting::{DiffuseLightingInput, LightingLightInput, LightingSource, prepare_lighting},
    scene::{Layer, Scene},
};

fn lit_scene(bump: Scene, area: Rect) -> surgeist_render::Result<Scene> {
    let domains = FilterDomains::try_new(area, area, area)?;
    let light = LightingLightInput::Distant { azimuth: 0.0, elevation: 45.0 };
    let material = DiffuseLightingInput::new().with_surface_scale(2.0);
    let mut accounting = RenderAccountingContext::default();
    let lighting = prepare_lighting(material, light, FilterColorSpace::LinearSrgb, &mut accounting)?;
    let mut scene = Scene::new();
    scene.lighting_layer(Layer::new(), lighting, LightingSource::finite(bump, domains));
    Ok(scene)
}
```

## Sampling and placement

Source alpha defines the bump height. Render captures the Scene and supported
same-layer backdrop in local coordinates. It samples transparent black beyond
the intersection of the declared source and filter domains. An empty source
supplies zero height throughout the domain. It can still produce lit pixels.

The kernel grid starts at the filter domain's minimum corner, with each sample
at the center of its cell. The default step is one texel of the local capture
raster. A positive explicit pair sets horizontal and vertical step lengths in
local units. The full filter domain fixes the phase and defines where to use
the one-sided Sobel boundary weights. An axis with one cell has zero derivative.
An axis with two cells uses a one-sided derivative at both ends.

Lighting uses the chosen working color space, with the straight light-color
RGB independent of its alpha. Subnormal working components count as zero
before applying light strength. The operation clamps the computed RGB
components individually and gives the diffuse result opaque alpha. A point at
the surface sample and a spot with coincident source and target contribute
zero incident light. A zero-degree cone requires the represented direction of
the sample to coincide with the spot axis. A positive cone ramps inward from
its cutoff before applying full focus strength.

Render reconstructs the lit result and applies coverage of the intersection of
the filter and primitive result domains once. The complete outer layer
transform then places that result, with the layer's clip, mask, opacity, and
composition handled by the layer compositor.

The equations and ordinary Sobel edge weights follow [Filter Effects Level
1](https://drafts.csswg.org/filter-effects-1/#feDiffuseLightingElement).
The finite grid phase, tiny-axis extension, and cone taper define Surgeist's
native behavior for resolved inputs. Document adapters resolve syntax, units,
the selected light child, and style before calling Render.

## Errors and accounting

`Error::lighting_diagnostic` identifies invalid light or material parameters
and reached grids or arithmetic that exceed the native representation. GPU and
accounting refusals retain their own diagnostic types. Failed attempts publish
neither pixels nor successful frame statistics.

The Render Ledger records four bump resampling opportunities per reached bump
cell and nine normal-stencil opportunities per shaded cell when a bump texture
is present. It records one lighting evaluation per shaded cell and four result
sampling opportunities per output cell. An absent bump has no alpha texture
reads. These counts describe scheduled work. The existing frame phases record
planning, GPU preparation, encoding, submission, and publication through the
same accounting context.
