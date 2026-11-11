// Copy an admitted source window, with optional nearest-neighbor expansion.
// Vello applies the draw's selected sampling quality to the isolated grid.
struct ImageSource {
    source_dimensions: vec2<u32>,
    atlas_origin: vec2<u32>,
    factors: vec2<u32>,
    source_stride: u32,
    first_pixel: u32,
    pixels: array<u32>,
}

@group(0) @binding(0) var<storage, read> source: ImageSource;
@group(0) @binding(1) var atlas: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn expand_image(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let point = invocation.xy;
    if (any(point >= source.source_dimensions * source.factors)) { return; }
    let original = point / source.factors;
    let pixel = source.pixels[source.first_pixel + original.y * source.source_stride + original.x];
    textureStore(atlas, vec2<i32>(source.atlas_origin + point), unpack4x8unorm(pixel));
}
