use super::CheckedWgpuScope;

const WIDTH: usize = 512;
const HEIGHT: usize = 2048;

fn store_word(texture: &mut [u8], index: usize, word: u32) {
    texture[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
}

fn load_word(texture: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(texture[index * 4..index * 4 + 4].try_into().unwrap())
}

#[test]
fn rgba8_descriptor_codec_and_endpoint_selection_execute_on_gpu() {
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let (device, queue) = renderer
        .default_wgpu_device_queue()
        .expect("the endpoint descriptor probe requires a native GPU adapter");
    let scope = CheckedWgpuScope::begin(device);
    let mut texture_bytes = vec![0; WIDTH * HEIGHT * 4];
    // Every channel visits every byte, with distinct channel order. These are
    // arbitrary data words, not colors: unpacking must preserve the exact bits.
    for byte in 0..=u8::MAX {
        store_word(
            &mut texture_bytes,
            usize::from(byte),
            u32::from_le_bytes([
                byte,
                byte.wrapping_add(85),
                byte.wrapping_add(170),
                byte.wrapping_add(255),
            ]),
        );
    }
    let scalar_bits = [
        0,
        0x8000_0000,
        1,
        2,
        0x007f_ffff,
        0x0080_0000,
        0x3e80_0000,
        0x3f00_0000,
        0x3f80_0000,
        0x7f7f_ffff,
        0xff7f_ffff,
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_1234,
    ];
    for (index, &bits) in scalar_bits.iter().enumerate() {
        store_word(&mut texture_bytes, 256 + index, bits);
    }
    let row = u32::try_from(HEIGHT - 2).unwrap();
    let interior = u32::from_le_bytes([12, 24, 36, 128]);
    for x in 0..WIDTH {
        store_word(&mut texture_bytes, (HEIGHT - 2) * WIDTH + x, interior);
    }
    let first = u32::from_le_bytes([64, 16, 8, 128]);
    let last = u32::from_le_bytes([0, 96, 32, 128]);
    let low = u32::from_le_bytes([10, 20, 30, 64]);
    let high = u32::from_le_bytes([40, 50, 60, 128]);
    let zero_first = u32::from_le_bytes([9, 8, 7, 16]);
    let zero_last = u32::from_le_bytes([3, 2, 1, 8]);
    let final_descriptor = u32::try_from(WIDTH * HEIGHT - 8).unwrap();
    let descriptors = [
        (
            511_u32,
            [0, row, 0, 0x3e80_0000, 0x3f40_0000, first, last, 0],
        ),
        (1022, [0, row, 0, 1, 1, low, high, 0]),
        (2047, [0, row, 0, 0x3f00_0000, 0x3f00_0000, low, high, 0]),
        (3070, [0, row, 0, 0xbf80_0000, 0xbf80_0000, low, high, 0]),
        (4094, [0, row, 0, 0x7f80_0000, 0x7f80_0000, low, high, 0]),
        (
            final_descriptor,
            [0, row, 0, 0x8000_0000, 0, zero_first, zero_last, 0],
        ),
    ];
    for &(index, words) in &descriptors {
        for (offset, word) in words.into_iter().enumerate() {
            store_word(&mut texture_bytes, index as usize + offset, word);
        }
    }
    // Literal analytic fixtures: square-root Power, the two singular limits,
    // Repeat, and a source-ordered coincident interior run. Full-f32 source
    // premultiplication deliberately differs from the RGBA8 endpoint words.
    let red_half = u32::from_le_bytes([128, 0, 0, 128]);
    let blue = u32::from_le_bytes([0, 0, 255, 255]);
    let red = u32::from_le_bytes([255, 0, 0, 255]);
    let white = u32::MAX;
    for (descriptor, payload, kind, segment) in [
        (7000, 10001, 1, 1),
        (7500, 10500, 1, 2),
        (8000, 11000, 1, 3),
        (8500, 11500, 2, 1),
    ] {
        for (offset, word) in [kind, payload, 2, 0, 1.0_f32.to_bits(), red_half, blue, 0]
            .into_iter()
            .enumerate()
        {
            store_word(&mut texture_bytes, descriptor + offset, word);
        }
        for (index, (position, channels, outgoing)) in [
            (0.0_f32, [0.5_f32, 0.0, 0.0, 0.5], segment),
            (1.0, [0.0, 0.0, 1.0, 1.0], 0),
        ]
        .into_iter()
        .enumerate()
        {
            let words = [
                position.to_bits(),
                channels[0].to_bits(),
                channels[1].to_bits(),
                channels[2].to_bits(),
                channels[3].to_bits(),
                outgoing,
                0.5_f32.to_bits(),
            ];
            for (offset, word) in words.into_iter().enumerate() {
                store_word(
                    &mut texture_bytes,
                    payload as usize + index * 7 + offset,
                    word,
                );
            }
        }
    }
    for (offset, word) in [1, 12000, 4, 0, 1.0_f32.to_bits(), red, white, 0]
        .into_iter()
        .enumerate()
    {
        store_word(&mut texture_bytes, 9000 + offset, word);
    }
    for (index, (position, channels)) in [
        (0.0_f32, [1.0_f32, 0.0, 0.0, 1.0]),
        (0.5, [0.0, 1.0, 0.0, 1.0]),
        (0.5, [0.0, 0.0, 1.0, 1.0]),
        (1.0, [1.0, 1.0, 1.0, 1.0]),
    ]
    .into_iter()
    .enumerate()
    {
        for (offset, word) in [
            position.to_bits(),
            channels[0].to_bits(),
            channels[1].to_bits(),
            channels[2].to_bits(),
            channels[3].to_bits(),
            0,
            0,
        ]
        .into_iter()
        .enumerate()
        {
            store_word(&mut texture_bytes, 12000 + index * 7 + offset, word);
        }
    }
    // Distance fixtures use the same descriptor/record artifact. Folded Pad
    // includes signed controls; Repeat phase is applied only after sign folding.
    for (descriptor, payload, kind, a, b, phase) in [
        (13000, 15000, 4, -0.5_f32, 1.5_f32, 0.0_f32),
        (13500, 15500, 5, 0.0, 1.0, 0.25),
        (14000, 16000, 5, 0.0, 1.0, 1.0),
        (14500, 16500, 5, 0.0, 1.0, 2.0_f32.powi(-25)),
        // Accepted native controls from the compensated physical -MAX/+MAX
        // field. Probe the sampling envelope independently of shape encoding.
        (17500, 18000, 4, -2.0, 2.0, 0.0),
    ] {
        for (offset, word) in [
            kind,
            payload,
            2,
            a.to_bits(),
            b.to_bits(),
            red,
            blue,
            phase.to_bits(),
        ]
        .into_iter()
        .enumerate()
        {
            store_word(&mut texture_bytes, descriptor + offset, word);
        }
        for (index, (position, channels)) in
            [(a, [1.0_f32, 0.0, 0.0, 1.0]), (b, [0.0, 0.0, 1.0, 1.0])]
                .into_iter()
                .enumerate()
        {
            for (offset, word) in [
                position.to_bits(),
                channels[0].to_bits(),
                channels[1].to_bits(),
                channels[2].to_bits(),
                channels[3].to_bits(),
                0,
                0,
            ]
            .into_iter()
            .enumerate()
            {
                store_word(
                    &mut texture_bytes,
                    payload as usize + index * 7 + offset,
                    word,
                );
            }
        }
    }
    for (offset, word) in [3, row, 0, 1, 1, low, high, 0].into_iter().enumerate() {
        store_word(&mut texture_bytes, 17000 + offset, word);
    }
    // Each tuple contains the raw coordinate bits and independent expected
    // color/key. Threshold equality and immediately adjacent binary32 values
    // come from the ordinary endpoint contract, rather than ramp generation.
    let selections = [
        (511, 0xbf80_0000, first, 0x407f_ffff),
        (511, 0x3e80_0000, first, 0xbe80_0000),
        (511, 0x3e80_0001, interior, 0xbe80_0001),
        (511, 0x3f00_0000, interior, 0xbf00_0000),
        (511, 0x3f3f_ffff, interior, 0xbf3f_ffff),
        (511, 0x3f40_0000, last, 0xbf40_0000),
        (511, 0x7f80_0000, last, 0xff80_0000),
        (511, 0xff80_0000, first, 0x007f_ffff),
        (1022, 0, low, 0x8000_0000),
        (1022, 0x8000_0000, low, 0x8000_0000),
        (1022, 1, high, 0x8000_0001),
        (1022, 2, high, 0x8000_0002),
        (1022, 0x0080_0000, high, 0x8080_0000),
        (2047, 0x3eff_ffff, low, 0xbeff_ffff),
        (2047, 0x3f00_0000, high, 0xbf00_0000),
        (2047, 0x3f00_0001, high, 0xbf00_0001),
        (3070, 0xc000_0000, low, 0x3fff_ffff),
        (3070, 0xbf80_0000, high, 0x407f_ffff),
        (3070, 0xbf00_0000, high, 0x40ff_ffff),
        (4094, 0x7f7f_ffff, low, 0xff7f_ffff),
        (4094, 0x7f80_0000, high, 0xff80_0000),
        (final_descriptor, 0xbf80_0000, zero_first, 0x407f_ffff),
        (final_descriptor, 0, zero_last, 0x8000_0000),
        (final_descriptor, 0x8000_0000, zero_last, 0x8000_0000),
    ];
    let mut inputs = Vec::<[u32; 4]>::new();
    let mut expected = Vec::<[u32; 8]>::new();
    let mut add_case = |flat: u32, descriptor: u32, raw: u32, color: u32, key: u32| {
        inputs.push([flat, descriptor, raw, 0]);
        expected.push([
            load_word(&texture_bytes, flat as usize),
            load_word(&texture_bytes, descriptor as usize),
            load_word(&texture_bytes, descriptor as usize + 1),
            load_word(&texture_bytes, descriptor as usize + 2),
            color,
            key,
            raw,
            load_word(&texture_bytes, descriptor as usize + 7),
        ]);
    };
    for flat in 0..256 + u32::try_from(scalar_bits.len()).unwrap() {
        add_case(flat, 511, 0, first, 0x8000_0000);
    }
    for (descriptor, raw, color, key) in selections {
        add_case(descriptor + 4, descriptor, raw, color, key);
    }
    // The highest valid flat address is actually loaded on this GPU, without
    // claiming execution at the protocol's much larger 30-bit address ceiling.
    add_case(
        u32::try_from(WIDTH * HEIGHT - 1).unwrap(),
        final_descriptor,
        0,
        zero_last,
        0x8000_0000,
    );

    for (descriptor, q, color) in [
        (7000, 0.25_f32, u32::from_le_bytes([64, 0, 128, 191])),
        (7000, 0.5625, u32::from_le_bytes([32, 0, 191, 223])),
        (7000, -1.0, red_half),
        (7000, 0.0, red_half),
        (7000, 1.0, blue),
        (7500, 0.0, red_half),
        (7500, f32::from_bits(1), blue),
        (7500, 0.5, blue),
        (8000, 0.5, red_half),
        (8000, f32::from_bits(1.0_f32.to_bits() - 1), red_half),
        (8000, 1.0, blue),
        (8500, 0.25, u32::from_le_bytes([64, 0, 128, 191])),
        (8500, 1.25, u32::from_le_bytes([64, 0, 128, 191])),
        (8500, -0.75, u32::from_le_bytes([64, 0, 128, 191])),
        (8500, 1.0, red_half),
        (9000, 0.5, blue),
        (9000, 0.25, u32::from_le_bytes([128, 128, 0, 255])),
        (9000, 0.75, u32::from_le_bytes([128, 128, 255, 255])),
        (13000, 0.0, u32::from_le_bytes([191, 0, 64, 255])),
        (13000, -0.0, u32::from_le_bytes([191, 0, 64, 255])),
        (13000, 1.0, u32::from_le_bytes([64, 0, 191, 255])),
        (13000, -1.0, u32::from_le_bytes([64, 0, 191, 255])),
        (13000, 1.5, blue),
        (13000, -1.5, blue),
        (13500, 0.0, u32::from_le_bytes([64, 0, 191, 255])),
        (13500, 0.25, red),
        (13500, -0.25, red),
        (13500, 0.5, u32::from_le_bytes([191, 0, 64, 255])),
        (13500, -0.5, u32::from_le_bytes([191, 0, 64, 255])),
        (13500, 1.25, red),
        (13500, -1.25, red),
        (14000, 0.0, red),
        (14000, -0.25, u32::from_le_bytes([191, 0, 64, 255])),
        // This subtraction/wrap rounds to one. A second wrap would invent a
        // beginning-of-cycle sample, violating the lowered seam contract.
        (14500, 0.0, blue),
        (17000, 0.0, low),
        (17000, -0.0, low),
        (17000, f32::from_bits(1), high),
        (17000, -f32::from_bits(1), high),
        (17000, -1.0, high),
        (17500, 0.0, u32::from_le_bytes([128, 0, 128, 255])),
        (17500, 1.0, u32::from_le_bytes([64, 0, 191, 255])),
        (17500, -1.0, u32::from_le_bytes([64, 0, 191, 255])),
        (17500, 2.0, blue),
        (17500, -2.0, blue),
        (17500, 3.0, blue),
        (17500, -3.0, blue),
    ] {
        let bits = q.to_bits();
        let key = if bits & 0x7fff_ffff == 0 {
            0x8000_0000
        } else if bits & 0x8000_0000 == 0 {
            bits ^ 0x8000_0000
        } else {
            !bits
        };
        add_case(descriptor + 6, descriptor, bits, color, key);
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ordinary gradient endpoint codec texture"),
        size: wgpu::Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texture_bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(WIDTH as u32 * 4),
            rows_per_image: Some(HEIGHT as u32),
        },
        texture.size(),
    );
    let input = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ordinary gradient endpoint codec inputs"),
        size: inputs.len() as u64 * 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: true,
    });
    {
        let mut mapped = input.slice(..).get_mapped_range_mut();
        for (index, word) in inputs.iter().flatten().enumerate() {
            mapped
                .slice(index * 4..index * 4 + 4)
                .copy_from_slice(&word.to_le_bytes());
        }
    }
    input.unmap();
    let byte_len = inputs.len() as u64 * 32;
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ordinary gradient endpoint codec outputs"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ordinary gradient endpoint codec readback"),
        size: byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let source = format!(
        "const GRADIENT_WIDTH = 512;\n\
         @group(0) @binding(0) var gradients: texture_2d<f32>;\n\
         @group(0) @binding(1) var<storage, read> inputs: array<vec4<u32>>;\n\
         @group(0) @binding(2) var<storage, read_write> outputs: array<u32>;\n\
         {}\n\
         @compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id: vec3<u32>) {{\n\
             if (id.x >= arrayLength(&inputs)) {{ return; }}\n\
             let probe = inputs[id.x];\n\
             let raw = bitcast<f32>(probe.z);\n\
             let base = id.x * 8u;\n\
             outputs[base] = gradient_endpoint_word(probe.x);\n\
             outputs[base + 1u] = gradient_endpoint_word(probe.y);\n\
             outputs[base + 2u] = gradient_endpoint_word(probe.y + 1u);\n\
             outputs[base + 3u] = gradient_endpoint_word(probe.y + 2u);\n\
             outputs[base + 4u] = pack4x8unorm(ordinary_gradient_color(raw, probe.y));\n\
             outputs[base + 5u] = gradient_endpoint_ordered_bits(probe.z);\n\
             outputs[base + 6u] = bitcast<u32>(raw);\n\
             outputs[base + 7u] = gradient_endpoint_word(probe.y + 7u);\n\
         }}",
        include_str!("gradient_endpoints.wgsl")
    );
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shared ordinary gradient endpoint helper probe"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("shared ordinary gradient endpoint helper probe"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ordinary gradient endpoint helper probe bindings"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: input.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ordinary gradient endpoint helper probe commands"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.dispatch_workgroups(u32::try_from(inputs.len()).unwrap().div_ceil(64), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&output, 0, &staging, 0, byte_len);
    let submission = queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    staging
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    let deadline = std::time::Duration::from_secs(5);
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(deadline),
        })
        .expect("endpoint descriptor probe must complete within five seconds");
    receiver
        .recv_timeout(deadline)
        .expect("endpoint descriptor probe map callback must arrive within five seconds")
        .expect("endpoint descriptor probe output must map successfully");
    pollster::block_on(scope.finish("ordinary gradient endpoint helper probe failed")).unwrap();
    let mapped = staging.slice(..).get_mapped_range();
    for (index, (actual, expected)) in mapped.as_chunks::<32>().0.iter().zip(expected).enumerate() {
        let actual = std::array::from_fn::<_, 8, _>(|channel| {
            u32::from_le_bytes(actual[channel * 4..channel * 4 + 4].try_into().unwrap())
        });
        if expected[1] == 0 || expected[1] == 3 {
            assert_eq!(actual, expected, "GPU endpoint descriptor probe {index}");
        } else {
            for channel in 0..8 {
                if channel != 4 {
                    assert_eq!(
                        actual[channel], expected[channel],
                        "program metadata {index}/{channel}"
                    );
                }
            }
            for (actual, expected) in actual[4]
                .to_le_bytes()
                .into_iter()
                .zip(expected[4].to_le_bytes())
            {
                assert!(
                    actual.abs_diff(expected) <= 1,
                    "analytic program color {index}: {actual} vs {expected}"
                );
            }
        }
    }
    drop(mapped);
    staging.unmap();
}
