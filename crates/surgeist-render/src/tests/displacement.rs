//! Independent public native displacement pixel witnesses. Native map bytes are
//! straight RGBA8, never an Image or a premultiplied capture. This wholly new
//! receiver has no executable preimplementation regression boundary.
use super::convolution::{assert_pixel, bounds, image, pixel, renderer_and_surface};
use crate::{
    accounting::RenderAccountingContext, displacement::DisplacementChannel as Channel,
    displacement::DisplacementInput, displacement::DisplacementSource,
    displacement::PreparedDisplacement, displacement::PreparedDisplacementMap,
    displacement::prepare_displacement, displacement::prepare_native_displacement_map,
    filter_domains::FilterDomains, filter_model::FilterColorSpace, geometry::Rect, geometry::Size,
    geometry::Transform, image::Image, image::ImageBuffer, image::ImageFit, layer::Layer,
    paint::Color, renderer::Renderer, resource::WorkingFormat, scene::Scene, surface::Parameters,
    surface::Surface,
};
use std::sync::Arc;

pub(super) const FORMATS: [WorkingFormat; 2] = [
    WorkingFormat::HighPrecision,
    WorkingFormat::ReducedPrecision,
];

pub(super) fn operation(input: DisplacementInput, space: FilterColorSpace) -> PreparedDisplacement {
    prepare_displacement(input, space, &mut RenderAccountingContext::default()).unwrap()
}

pub(super) fn active(scale: f64, x: Channel, y: Channel) -> PreparedDisplacement {
    operation(
        DisplacementInput::new()
            .with_scale(scale)
            .with_channels(x, y),
        FilterColorSpace::Srgb,
    )
}

pub(super) fn native_map(
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    map: Rect,
) -> PreparedDisplacementMap {
    prepare_native_displacement_map(
        Size::new(f64::from(width), f64::from(height)),
        Arc::from(bytes),
        map,
        &mut RenderAccountingContext::default(),
    )
    .unwrap()
}

pub(super) fn source_scene(input: Image, destination: Rect) -> Scene {
    let mut scene = Scene::new();
    scene.image(input, destination, ImageFit::Stretch);
    scene
}

pub(super) fn finite_scene(
    source: Scene,
    domains: FilterDomains,
    effect: PreparedDisplacement,
    map: PreparedDisplacementMap,
    layer: Layer,
) -> Scene {
    let mut scene = Scene::new();
    scene
        .displacement_layer(
            layer,
            effect,
            DisplacementSource::finite(source, domains),
            map,
        )
        .unwrap();
    scene
}

pub(super) fn render(
    renderer: &mut Renderer,
    surface: &mut Surface,
    scene: &Scene,
    format: WorkingFormat,
) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
        surface,
        scene,
        Parameters::default(),
        format,
    ))
    .expect("public native displacement must execute on a compatible real GPU");
    assert_eq!(stats, renderer.stats());
    // Empty O legitimately takes the ordinary empty-frame route.
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

pub(super) fn assert_premul(
    output: &ImageBuffer,
    x: u32,
    y: u32,
    expected: [f64; 4],
    tolerance: f64,
) {
    let actual = pixel(output, x, y);
    let alpha = f64::from(actual[3]) / 255.0;
    assert!(
        (alpha - expected[3]).abs() * 255.0 <= tolerance,
        "({x},{y}) {actual:?}, expected premul {expected:?}"
    );
    for c in 0..3 {
        assert!(
            (f64::from(actual[c]) / 255.0 * alpha - expected[c]).abs() * 255.0 <= tolerance,
            "({x},{y}) {actual:?}, expected premul {expected:?}"
        );
    }
}

// Each x and y selects a different opaque texel. These are source colors, not
// displacement-table expectations. All fixtures stay within x<64 and y<64.
fn palette_pixel(x: u32, y: u32) -> [u8; 4] {
    [
        (3 * x + 7) as u8,
        (3 * y + 11) as u8,
        ((x + 2 * y) % 47 + 70) as u8,
        255,
    ]
}
fn palette(width: u32, height: u32) -> Scene {
    let bytes = (0..height)
        .flat_map(|y| (0..width).flat_map(move |x| palette_pixel(x, y)))
        .collect();
    source_scene(image(width, height, bytes), bounds(width, height))
}
fn selected_scene(effect: PreparedDisplacement, map: PreparedDisplacementMap) -> Scene {
    let full = bounds(48, 16);
    finite_scene(
        palette(48, 16),
        FilterDomains::try_new(full, full, Rect::new(8.0, 8.0, 1.0, 1.0)).unwrap(),
        effect,
        map,
        Layer::new(),
    )
}

#[test]
fn public_displacement_signed_byte_extrema_and_independent_channels_copy_literal_texels() {
    let (mut renderer, mut surface) = renderer_and_surface(48, 16);
    let one = Rect::new(8.0, 8.0, 1.0, 1.0);
    for format in FORMATS {
        // Ordered binary64 gives +4: c0 -> -1, c255 -> +2;
        // -4 reverses these to +2 and -1. X and Y are independent.
        for (scale, bytes, expected) in [
            (4.0, [0, 255, 128, 255], (7, 10)),
            (4.0, [255, 0, 128, 255], (10, 7)),
            (-4.0, [0, 255, 128, 255], (10, 7)),
            (-4.0, [255, 0, 128, 255], (7, 10)),
            (1.5, [255, 0, 128, 255], (9, 8)),
            (-1.5, [255, 0, 128, 255], (8, 9)),
        ] {
            let scene = selected_scene(
                active(scale, Channel::R, Channel::G),
                native_map(1, 1, bytes.to_vec(), one),
            );
            let output = render(&mut renderer, &mut surface, &scene, format);
            assert_pixel(&output, 8, 8, palette_pixel(expected.0, expected.1), 1);
            assert_pixel(&output, 7, 8, [0; 4], 0);
        }
    }
}

#[test]
fn public_displacement_ordered_binary64_thresholds_and_neighbor_scales_reach_distinct_texels() {
    let (mut renderer, mut surface) = renderer_and_surface(48, 16);
    let one = Rect::new(8.0, 8.0, 1.0, 1.0);
    for format in FORMATS {
        for (scale, c, offset) in [
            (-153.0, 85, 26),
            (-152.99999999999997, 85, 26),
            (-153.00000000000003, 85, 26),
            (-155.0, 102, 16),
            (-154.99999999999997, 102, 16),
            (-155.00000000000003, 102, 16),
        ] {
            // These literal offsets follow separately ordered f64 products and
            // sums: at -155, c102 the pre-trunc value is16.000000000000007.
            // Separately rounded f32 would address25/15 for the central cases.
            let scene = selected_scene(
                active(scale, Channel::R, Channel::G),
                native_map(1, 1, vec![c, 128, 0, 255], one),
            );
            let output = render(&mut renderer, &mut surface, &scene, format);
            assert_pixel(&output, 8, 8, palette_pixel(8 + offset, 8), 1);
        }
    }
}

#[test]
fn public_displacement_raw_straight_rgb_survives_low_alpha_and_hidden_rgb_is_suppressed() {
    let (mut renderer, mut surface) = renderer_and_surface(48, 16);
    let one = Rect::new(8.0, 8.0, 1.0, 1.0);
    for format in FORMATS {
        for (bytes, x, y, expected) in [
            ([255, 128, 0, 64], Channel::R, Channel::G, (12, 8)),
            ([255, 128, 0, 64], Channel::B, Channel::G, (5, 8)),
            ([255, 128, 0, 64], Channel::A, Channel::G, (7, 8)),
            ([255, 128, 0, 1], Channel::R, Channel::G, (12, 8)),
            // Premul8 would round R128*a1 to one byte and recover R255.
            ([128, 128, 0, 1], Channel::R, Channel::G, (8, 8)),
            ([255, 128, 0, 0], Channel::R, Channel::G, (5, 5)),
        ] {
            let output = render(
                &mut renderer,
                &mut surface,
                &selected_scene(active(8.0, x, y), native_map(1, 1, bytes.to_vec(), one)),
                format,
            );
            assert_pixel(&output, 8, 8, palette_pixel(expected.0, expected.1), 1);
        }
        let half = Rect::new(8.0, 8.0, 1.0, 0.5);
        for (channel, expected_x) in [(Channel::R, 12), (Channel::A, 6)] {
            // m=.5 keeps straight R255, while covered A64 becomes32.
            // S8,c32 -> trunc(-2.496...)= -2. Output alpha stays opaque.
            let output = render(
                &mut renderer,
                &mut surface,
                &selected_scene(
                    active(8.0, channel, Channel::G),
                    native_map(1, 1, vec![255, 128, 0, 64], half),
                ),
                format,
            );
            assert_pixel(&output, 8, 8, palette_pixel(expected_x, 8), 1);
        }
        // Covered alpha1*.25 quantizes to0, but the raw alpha and unquantized
        // coverage are positive. Straight RGB128 must still select offset0.
        let output = render(
            &mut renderer,
            &mut surface,
            &selected_scene(
                active(8.0, Channel::R, Channel::G),
                native_map(1, 1, vec![128, 128, 0, 1], Rect::new(8.0, 8.0, 1.0, 0.25)),
            ),
            format,
        );
        assert_pixel(&output, 8, 8, palette_pixel(8, 8), 1);
    }
}

#[test]
fn public_displacement_half_up_covered_alpha_and_map_only_linear_conversion_change_addresses() {
    let (mut renderer, mut surface) = renderer_and_surface(48, 16);
    let one = Rect::new(8.0, 8.0, 1.0, 1.0);
    for format in FORMATS {
        for (space, expected_x) in [
            (FilterColorSpace::Srgb, 8),
            (FilterColorSpace::LinearSrgb, 7),
        ] {
            let op = operation(
                DisplacementInput::new()
                    .with_scale(8.0)
                    .with_channels(Channel::R, Channel::A),
                space,
            );
            // Numeric R128 gives offset0. Linear conversion quantizes R to55,
            // giving -1. A128 gives0 in either color space. The copied palette
            // remains encoded RGB; only the map changes representation.
            let output = render(
                &mut renderer,
                &mut surface,
                &selected_scene(op, native_map(1, 1, vec![128; 4], one)),
                format,
            );
            assert_pixel(&output, 8, 8, palette_pixel(expected_x, 8), 1);
        }
        // A253 * .5 =126.5, half-up ->127. S510 gives offset0;
        // rounding down to126 would give -2. R127 keeps Y offset0.
        let output = render(
            &mut renderer,
            &mut surface,
            &selected_scene(
                active(510.0, Channel::A, Channel::R),
                native_map(1, 1, vec![127, 0, 0, 253], Rect::new(8.0, 8.0, 1.0, 0.5)),
            ),
            format,
        );
        assert_pixel(&output, 8, 8, palette_pixel(8, 8), 1);
    }
}

#[test]
fn public_displacement_native_nearest_ties_and_adjacent_binary64_map_origins_use_uploaded_records()
{
    let (mut renderer, mut surface) = renderer_and_surface(12, 12);
    let full = bounds(12, 12);
    let output_domain = Rect::new(4.0, 4.0, 1.0, 1.0);
    for format in FORMATS {
        for (map_x, expected_x) in [(3.5, 6), (3.5000000000000004, 3), (3.4999999999999996, 6)] {
            // Center4.5 is exactly the native-cell boundary when M.x=3.5.
            // Higher-cell ties select R255; moving M one f64 ULP right selects0.
            let scene = finite_scene(
                palette(12, 12),
                FilterDomains::try_new(full, full, output_domain).unwrap(),
                active(4.0, Channel::R, Channel::G),
                native_map(
                    2,
                    1,
                    vec![0, 128, 0, 255, 255, 128, 0, 255],
                    Rect::new(map_x, 4.0, 2.0, 1.0),
                ),
                Layer::new(),
            );
            let output = render(&mut renderer, &mut surface, &scene, format);
            assert_pixel(&output, 4, 4, palette_pixel(expected_x, 4), 1);
        }
    }
}

#[test]
fn public_displacement_map_exterior_is_zero_and_map_lookup_is_undisplaced() {
    let (mut renderer, mut surface) = renderer_and_surface(12, 12);
    let full = bounds(12, 12);
    for format in FORMATS {
        for (map, expected) in [
            (
                native_map(1, 1, vec![255, 128, 0, 255], Rect::new(5.0, 4.0, 1.0, 1.0)),
                [(3, 3), (7, 4), (5, 3), (6, 3)],
            ),
            (
                native_map(
                    4,
                    1,
                    [255_u8, 0, 255, 0]
                        .into_iter()
                        .flat_map(|r| [r, 128, 0, 255])
                        .collect(),
                    Rect::new(4.0, 4.0, 4.0, 1.0),
                ),
                [(6, 4), (4, 4), (8, 4), (6, 4)],
            ),
        ] {
            let scene = finite_scene(
                palette(12, 12),
                FilterDomains::try_new(full, full, Rect::new(4.0, 4.0, 4.0, 1.0)).unwrap(),
                active(4.0, Channel::R, Channel::G),
                map,
                Layer::new(),
            );
            let output = render(&mut renderer, &mut surface, &scene, format);
            for (x, (sx, sy)) in (4..8).zip(expected) {
                assert_pixel(&output, x, 4, palette_pixel(sx, sy), 1);
            }
        }
    }
}

#[test]
fn public_displacement_empty_native_maps_stay_active_while_zero_scale_and_semantic_maps_bypass() {
    let (mut renderer, mut surface) = renderer_and_surface(48, 16);
    let one = Rect::new(8.0, 8.0, 1.0, 1.0);
    for format in FORMATS {
        for map in [
            native_map(0, 1, vec![], one),
            native_map(1, 0, vec![], one),
            native_map(1, 1, vec![255; 4], Rect::new(8.0, 8.0, 0.0, 1.0)),
        ] {
            let output = render(
                &mut renderer,
                &mut surface,
                &selected_scene(active(4.0, Channel::R, Channel::G), map),
                format,
            );
            assert_pixel(&output, 8, 8, palette_pixel(7, 7), 1);
        }
        for map in [
            PreparedDisplacementMap::source_graphic(),
            PreparedDisplacementMap::source_alpha(),
            PreparedDisplacementMap::suppressed(),
        ] {
            let output = render(
                &mut renderer,
                &mut surface,
                &selected_scene(active(f64::MAX, Channel::R, Channel::G), map),
                format,
            );
            assert_pixel(&output, 8, 8, palette_pixel(8, 8), 1);
        }
        let output = render(
            &mut renderer,
            &mut surface,
            &selected_scene(
                operation(DisplacementInput::new(), FilterColorSpace::LinearSrgb),
                native_map(1, 1, vec![0; 4], one),
            ),
            format,
        );
        assert_pixel(&output, 8, 8, palette_pixel(8, 8), 1);
        // A/A defaults are independent of the RGB bytes when scale is active.
        let output = render(
            &mut renderer,
            &mut surface,
            &selected_scene(
                operation(
                    DisplacementInput::new().with_scale(4.0),
                    FilterColorSpace::Srgb,
                ),
                native_map(1, 1, vec![0, 0, 0, 255], one),
            ),
            format,
        );
        assert_pixel(&output, 8, 8, palette_pixel(10, 10), 1);
    }
}

#[test]
fn public_displacement_independent_domains_keep_reachable_source_outside_output_and_cover_once() {
    let (mut renderer, mut surface) = renderer_and_surface(8, 2);
    for format in FORMATS {
        // S=[1.5,6], F=[0,5.5], P=[2.5,4]. Thus D=[1.5,5.5],
        // O=[2.5,4]. R0 moves x2->x1 (D coverage.5), and x3->x2.
        // The first output has another independent coverage.5: alpha.25.
        let source = Rect::new(1.5, 0.0, 4.5, 1.0);
        let filter = Rect::new(0.0, 0.0, 5.5, 1.0);
        let result = Rect::new(2.5, 0.0, 1.5, 1.0);
        let mut ink = Scene::new();
        ink.fill(bounds(8, 2), Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
        let scene = finite_scene(
            ink,
            FilterDomains::try_new(source, filter, result).unwrap(),
            active(4.0, Channel::R, Channel::G),
            native_map(1, 1, vec![0, 128, 0, 255], filter),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene, format);
        assert_premul(&output, 2, 0, [0.25, 0.0, 0.0, 0.25], 1.5);
        assert_pixel(&output, 3, 0, [255, 0, 0, 255], 1);
        assert_pixel(&output, 4, 0, [0; 4], 0);
    }
}

#[test]
fn public_displacement_touching_continuous_map_and_output_still_share_a_covered_lattice_cell() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let full = bounds(1, 1);
    for format in FORMATS {
        // O=[0,.5], M=[.5,1] have empty continuous intersection. Their common
        // cell nevertheless has map coverage.5, A255->128, giving offset0 at
        // scale4. Treating M as unused supplies c0 and incorrectly samples -1.
        let scene = finite_scene(
            source_scene(image(1, 1, vec![255, 0, 0, 255]), full),
            FilterDomains::try_new(full, full, Rect::new(0.0, 0.0, 0.5, 1.0)).unwrap(),
            active(4.0, Channel::A, Channel::A),
            native_map(1, 1, vec![255; 4], Rect::new(0.5, 0.0, 0.5, 1.0)),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene, format);
        assert_premul(&output, 0, 0, [0.5, 0.0, 0.0, 0.5], 1.5);
    }
}

#[test]
fn public_displacement_fractional_density_uses_raster_offsets_before_reflected_placement() {
    let (mut renderer, mut surface) = renderer_and_surface(12, 3);
    let full = bounds(8, 2);
    for format in FORMATS {
        // A=.5I gives rho=.5. Scale8 therefore has raster S4 and R255
        // selects +2 raster cells, i.e. four logical units. Four broad stripes
        // make each captured raster cell an independently known solid color.
        for reflected in [false, true] {
            let transform = if reflected {
                [-0.5, 0.0, 0.0, 0.5, 8.0, 0.0]
            } else {
                [0.5, 0.0, 0.0, 0.5, 0.0, 0.0]
            };
            let scene = finite_scene(
                source_scene(
                    image(
                        4,
                        1,
                        vec![
                            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
                        ],
                    ),
                    full,
                ),
                FilterDomains::try_new(full, full, full).unwrap(),
                active(8.0, Channel::R, Channel::G),
                native_map(1, 1, vec![255, 128, 0, 255], full),
                Layer::new().with_transform(Transform::try_new(transform).unwrap()),
            );
            let output = render(&mut renderer, &mut surface, &scene, format);
            let xs = if reflected {
                [7, 6, 5, 4]
            } else {
                [0, 1, 2, 3]
            };
            for (x, expected) in
                xs.into_iter()
                    .zip([[0, 0, 255, 255], [255, 255, 0, 255], [0; 4], [0; 4]])
            {
                assert_pixel(&output, x, 0, expected, 1);
            }
        }
    }
}
