// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Modified for surgeist-render; see NOTICE-VELLO.md for provenance and adaptations.

use vello_shaders::{BindType, ComputeShader, SHADERS};

use crate::{
    BackendErrorCode, Error, Result,
    accounting::{
        Account, AccountedVec, CpuAccount, Generation, GpuObjectKind, OwnedGpu, RenderResource,
        RenderWork, StorageAccount,
    },
};

use super::recording::RasterKernel;

#[must_use = "checked WGPU scopes must be explicitly resolved"]
pub(super) struct CheckedWgpuScope<'a> {
    device: &'a wgpu::Device,
    validation: Option<wgpu::ErrorScopeGuard>,
    out_of_memory: Option<wgpu::ErrorScopeGuard>,
    internal: Option<wgpu::ErrorScopeGuard>,
}

impl<'a> CheckedWgpuScope<'a> {
    pub(super) fn begin(device: &'a wgpu::Device) -> Self {
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let out_of_memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        Self {
            device,
            validation: Some(validation),
            out_of_memory: Some(out_of_memory),
            internal: Some(internal),
        }
    }

    pub(super) fn device(&self) -> &wgpu::Device {
        self.device
    }

    pub(super) async fn finish(mut self, failure_message: &'static str) -> Result<()> {
        let validation = match self.validation.take() {
            Some(scope) => scope,
            None => {
                return Err(render_failed(
                    "internal Vello validation scope was already resolved",
                ));
            }
        };
        let out_of_memory = match self.out_of_memory.take() {
            Some(scope) => scope,
            None => {
                return Err(render_failed(
                    "internal Vello out-of-memory scope was already resolved",
                ));
            }
        };
        let internal = match self.internal.take() {
            Some(scope) => scope,
            None => {
                return Err(render_failed(
                    "internal Vello internal-error scope was already resolved",
                ));
            }
        };

        let validation_error = validation.pop().await;
        let memory_error = out_of_memory.pop().await;
        let internal_error = internal.pop().await;
        if let Some(source) = validation_error.or(memory_error).or(internal_error) {
            return Err(classify_checked_wgpu_error(failure_message, source));
        }
        Ok(())
    }
}

impl Drop for CheckedWgpuScope<'_> {
    fn drop(&mut self) {
        drop(self.validation.take());
        drop(self.out_of_memory.take());
        drop(self.internal.take());
    }
}

pub(super) struct CheckedComputePipeline {
    pipeline: OwnedGpu<wgpu::ComputePipeline>,
    bind_group_layout: OwnedGpu<wgpu::BindGroupLayout>,
    binding_indices: AccountedVec<u32>,
}

impl CheckedComputePipeline {
    fn create(
        device: &wgpu::Device,
        generation: &Generation,
        cpu: &CpuAccount,
        observer: Option<&Account>,
        shader: &ComputeShader<'_>,
        source_override: Option<&str>,
    ) -> Result<Self> {
        let (binding_indices, entries) = prepare_bindings(cpu, observer, shader)?;
        let shader_module = generation.opaque(GpuObjectKind::ShaderModule, observer, || {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader.name.as_ref()),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    source_override.unwrap_or(shader.wgsl.code.as_ref()),
                )),
            })
        })?;
        let bind_group_layout =
            generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Surgeist internal Vello bindings"),
                    entries: &entries,
                })
            })?;
        let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Surgeist internal Vello compute layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            })
        })?;
        let pipeline = generation.opaque(GpuObjectKind::ComputePipeline, observer, || {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(shader.name.as_ref()),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions {
                    zero_initialize_workgroup_memory: false,
                    ..Default::default()
                },
                cache: None,
            })
        })?;

        Ok(Self {
            pipeline,
            bind_group_layout,
            binding_indices,
        })
    }

    pub(super) fn pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    pub(super) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub(super) fn binding_indices(&self) -> &[u32] {
        &self.binding_indices
    }
}

fn prepare_bindings(
    cpu: &CpuAccount,
    observer: Option<&Account>,
    shader: &ComputeShader<'_>,
) -> Result<(AccountedVec<u32>, AccountedVec<wgpu::BindGroupLayoutEntry>)> {
    cpu.phase(|| {
        if shader.bindings.len() != shader.wgsl.binding_indices.len() {
            return Err(render_failed(
                "internal Vello shader metadata has inconsistent binding indices",
            ));
        }

        let mut binding_indices = AccountedVec::new_in(
            &StorageAccount::Renderer(cpu.clone()),
            shader.wgsl.binding_indices.len(),
            RenderResource::RetainedCpuBytes,
        )?;
        for &index in shader.wgsl.binding_indices.iter() {
            shader_work(cpu, observer, RenderWork::ShaderBindingVisits, 1)?;
            let index = u32::from(index);
            for previous in binding_indices.iter() {
                shader_work(cpu, observer, RenderWork::ShaderBindingVisits, 1)?;
                if *previous == index {
                    return Err(render_failed(
                        "internal Vello shader metadata repeats a binding index",
                    ));
                }
            }
            binding_indices.push(index)?;
        }
        let mut entries = AccountedVec::new_in(
            &StorageAccount::Renderer(cpu.clone()),
            shader.bindings.len(),
            RenderResource::ScratchCpuBytes,
        )?;
        for (binding, index) in shader
            .bindings
            .iter()
            .copied()
            .zip(binding_indices.iter().copied())
        {
            shader_work(cpu, observer, RenderWork::ShaderBindingVisits, 1)?;
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: index,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: binding_type(binding),
                count: None,
            })?;
        }
        Ok((binding_indices, entries))
    })
}

/// Keeps endpoint half-plane ties aligned with the MSAA mask LUT convention.
/// Upstream Vello 0.9 rounds an endpoint exactly on a sample into the extra
/// covered sample while the interior LUT selects the lower-coverage side;
/// translating the same edge across a 16px tile boundary can therefore change
/// one sample. The one-sided robust epsilon affects only that f32 tie.
const MSAA8_ENDPOINT_REPLACEMENTS: [ShaderSourceReplacement; 2] = [
    ShaderSourceReplacement {
        from: "round(8.0 * (xy0.y - f32(y)))",
        to: "round(8.0 * (xy0.y - f32(y)) - 8.0 * ROBUST_EPSILON)",
        matches: ShaderMatchCount::Twice,
        failure: "internal Vello MSAA shader no longer has both checked endpoint quantizers",
    },
    ShaderSourceReplacement {
        from: "round(8.0 * (xy1.y - f32(y)))",
        to: "round(8.0 * (xy1.y - f32(y)) - 8.0 * ROBUST_EPSILON)",
        matches: ShaderMatchCount::Twice,
        failure: "internal Vello MSAA shader no longer has both checked endpoint quantizers",
    },
];

const SET_LUM_REPLACEMENT: ShaderSourceReplacement = ShaderSourceReplacement {
    from: "fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {\n    return clip_color(c + (l - lum(c)));\n}",
    to: "fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {\n    let exact = vec3<bool>(surgeist_blend_luminosity_identity(c, l));\n    let shifted = select(c + (l - lum(c)), vec3<f32>(0.0), exact);\n    let clipped = surgeist_blend_clip_color(shifted, lum(shifted));\n    return select(clipped, vec3<f32>(l), exact);\n}",
    matches: ShaderMatchCount::Once,
    failure: "internal Vello fine shader no longer has the checked SetLum helper",
};

const BLEND_MATH_REPLACEMENTS: [ShaderSourceReplacement; 5] = [
    ShaderSourceReplacement {
        from: "fn blend_mix(",
        to: concat!(
            include_str!("../shaders/blend_math.wgsl"),
            "\nfn blend_mix("
        ),
        matches: ShaderMatchCount::Once,
        failure: "internal Vello fine shader no longer has the checked blend entry point",
    },
    ShaderSourceReplacement {
        from: "b = vec3(color_dodge(cb.x, cs.x), color_dodge(cb.y, cs.y), color_dodge(cb.z, cs.z));",
        to: "b = surgeist_blend_color_dodge(cb, cs);",
        matches: ShaderMatchCount::Once,
        failure: "internal Vello fine shader no longer has the checked color dodge call",
    },
    ShaderSourceReplacement {
        from: "b = vec3(color_burn(cb.x, cs.x), color_burn(cb.y, cs.y), color_burn(cb.z, cs.z));",
        to: "b = surgeist_blend_color_burn(cb, cs);",
        matches: ShaderMatchCount::Once,
        failure: "internal Vello fine shader no longer has the checked color burn call",
    },
    ShaderSourceReplacement {
        from: "b = soft_light(cb, cs);",
        to: "b = surgeist_blend_soft_light(cb, cs);",
        matches: ShaderMatchCount::Once,
        failure: "internal Vello fine shader no longer has the checked soft light call",
    },
    ShaderSourceReplacement {
        from: "set_lum(set_sat(",
        to: "set_lum(surgeist_blend_set_sat(",
        matches: ShaderMatchCount::Twice,
        failure: "internal Vello fine shader no longer has both checked SetSat calls",
    },
];

// A live local Plus accumulator can exceed normalized RGB. Bound an ordinary
// mixing result before backdrop-alpha modulation, preserving Plus as a separate
// complete operation rather than changing its existing additive accumulator.
const BLEND_RESULT_REPLACEMENT: ShaderSourceReplacement = ShaderSourceReplacement {
    from: "let mixed = blend_mix(cb, cs, mix_mode);",
    to: "var mixed = blend_mix(cb, cs, mix_mode);\n    if ((mode & 0xffu) != COMPOSE_PLUS) {\n        mixed = clamp(mixed, vec3<f32>(0.0), vec3<f32>(1.0));\n    }",
    matches: ShaderMatchCount::Once,
    failure: "internal Vello fine shader no longer has the checked blend-result boundary",
};

// Normal's fast path also consumes bounded B in the overlap. Keep extended
// local Plus source outside that overlap, without division or cancellation.
// The shared ClipOnly marker retains coverage arithmetic without a mixing step.
const NORMAL_BLEND_REPLACEMENT: ShaderSourceReplacement = ShaderSourceReplacement {
    from: "return backdrop * (1.0 - src.a) + src;",
    to: "if ((mode >> 8u) == MIX_CLIP) {\n            return backdrop * (1.0 - src.a) + src;\n        }\n        let bounded_source = clamp(src.rgb, vec3<f32>(0.0), vec3<f32>(src.a));\n        let rgb = (1.0 - backdrop.a) * src.rgb\n            + backdrop.a * bounded_source\n            + (1.0 - src.a) * backdrop.rgb;\n        return vec4<f32>(rgb, src.a + backdrop.a * (1.0 - src.a));",
    matches: ShaderMatchCount::Once,
    failure: "internal Vello fine shader no longer has the checked Normal blend boundary",
};

// WGSL sqrt inherits reciprocal/inverseSqrt accuracy, which supplies no finite
// zero-input certificate. Evaluate only a normal positive argument and select
// exact zero for squared lengths at most the smallest normal binary32 value.
// Repeating circular-field admission includes this format-derived lost norm.
const CIRCULAR_NORM_REPLACEMENT: ShaderSourceReplacement = ShaderSourceReplacement {
    from: "t = sqrt(xx + yy) - x * r1_recip;",
    to: "var radial_distance = 0.0;\n                        if (is_circular) {\n                            let squared_radius = xx + yy;\n                            radial_distance = select(sqrt(max(squared_radius, 0x1p-126f)), 0.0, squared_radius <= 0x1p-126f);\n                        } else {\n                            radial_distance = sqrt(xx + yy);\n                        }\n                        t = radial_distance - x * r1_recip;",
    matches: ShaderMatchCount::Once,
    failure: "internal Vello fine shader no longer has the checked circular norm boundary",
};

// Mode 3 is a private draw-word extension, intercepted before native extension
// or ramp lookup. Draw-leaf reads geometry after that word; coarse passes it
// through unchanged. Other extend modes retain their exact native operations.
const GRADIENT_ENDPOINT_REPLACEMENTS: [ShaderSourceReplacement; 4] = [
    ShaderSourceReplacement {
        from: "const EXTEND_PAD: u32 = 0u;",
        to: include_str!("shaders/gradient_endpoints.wgsl"),
        matches: ShaderMatchCount::Once,
        failure: "internal Vello fine shader no longer has the checked gradient helper boundary",
    },
    ShaderSourceReplacement {
        from: "let x = i32(round(extend_mode_normalized(my_d, lin.extend_mode) * f32(GRADIENT_WIDTH - 1)));\n                    let fg_rgba = textureLoad(gradients, vec2(x, i32(lin.index)), 0);",
        to: "var fg_rgba: vec4<f32>;\n                    if (lin.extend_mode == 3u) {\n                        fg_rgba = ordinary_gradient_color(my_d, lin.index);\n                    } else {\n                        let x = i32(round(extend_mode_normalized(my_d, lin.extend_mode) * f32(GRADIENT_WIDTH - 1)));\n                        fg_rgba = textureLoad(gradients, vec2(x, i32(lin.index)), 0);\n                    }",
        matches: ShaderMatchCount::Once,
        failure: "internal Vello fine shader no longer has the checked linear gradient lookup",
    },
    ShaderSourceReplacement {
        from: "t = extend_mode_normalized(focal_x + t_sign * t, rad.extend_mode);\n                        t = select(t, 1.0 - t, is_swapped);\n                        let x = i32(round(t * f32(GRADIENT_WIDTH - 1)));\n                        let fg_rgba = textureLoad(gradients, vec2(x, i32(rad.index)), 0);",
        to: "var fg_rgba: vec4<f32>;\n                        if (rad.extend_mode == 3u) {\n                            let raw_t = focal_x + t_sign * t;\n                            fg_rgba = ordinary_gradient_color(select(raw_t, 1.0 - raw_t, is_swapped), rad.index);\n                        } else {\n                            t = extend_mode_normalized(focal_x + t_sign * t, rad.extend_mode);\n                            t = select(t, 1.0 - t, is_swapped);\n                            let x = i32(round(t * f32(GRADIENT_WIDTH - 1)));\n                            fg_rgba = textureLoad(gradients, vec2(x, i32(rad.index)), 0);\n                        }",
        matches: ShaderMatchCount::Once,
        failure: "internal Vello fine shader no longer has the checked radial gradient lookup",
    },
    ShaderSourceReplacement {
        from: "let t = extend_mode_normalized(phi, sweep.extend_mode);\n                    let ramp_x = i32(round(t * f32(GRADIENT_WIDTH - 1)));\n                    let fg_rgba = textureLoad(gradients, vec2(ramp_x, i32(sweep.index)), 0);",
        to: "var fg_rgba: vec4<f32>;\n                    if (sweep.extend_mode == 3u) {\n                        fg_rgba = ordinary_gradient_color(phi, sweep.index);\n                    } else {\n                        let t = extend_mode_normalized(phi, sweep.extend_mode);\n                        let ramp_x = i32(round(t * f32(GRADIENT_WIDTH - 1)));\n                        fg_rgba = textureLoad(gradients, vec2(ramp_x, i32(sweep.index)), 0);\n                    }",
        matches: ShaderMatchCount::Once,
        failure: "internal Vello fine shader no longer has the checked sweep gradient lookup",
    },
];

#[derive(Clone, Copy)]
struct ShaderSourceReplacement {
    from: &'static str,
    to: &'static str,
    matches: ShaderMatchCount,
    failure: &'static str,
}

fn fine_shader_source(
    cpu: &CpuAccount,
    shader: &ComputeShader<'_>,
    adapt_msaa8_endpoints: bool,
) -> Result<AccountedVec<u8>> {
    let replacements = [
        SET_LUM_REPLACEMENT,
        BLEND_RESULT_REPLACEMENT,
        NORMAL_BLEND_REPLACEMENT,
        CIRCULAR_NORM_REPLACEMENT,
        GRADIENT_ENDPOINT_REPLACEMENTS[0],
        GRADIENT_ENDPOINT_REPLACEMENTS[1],
        GRADIENT_ENDPOINT_REPLACEMENTS[2],
        GRADIENT_ENDPOINT_REPLACEMENTS[3],
        BLEND_MATH_REPLACEMENTS[0],
        BLEND_MATH_REPLACEMENTS[1],
        BLEND_MATH_REPLACEMENTS[2],
        BLEND_MATH_REPLACEMENTS[3],
        BLEND_MATH_REPLACEMENTS[4],
        MSAA8_ENDPOINT_REPLACEMENTS[0],
        MSAA8_ENDPOINT_REPLACEMENTS[1],
    ];
    let [common @ .., _, _] = &replacements;
    let source = shader.wgsl.code.as_ref();
    if adapt_msaa8_endpoints {
        replace_shader_code(cpu, source, &replacements)
    } else {
        replace_shader_code(cpu, source, common)
    }
}

#[derive(Clone, Copy)]
enum ShaderMatchCount {
    Once,
    Twice,
}

#[derive(Clone, Copy)]
struct ShaderSourceEdit {
    start: usize,
    end: usize,
    output: &'static str,
}

impl ShaderSourceEdit {
    fn from_match((start, matched): (usize, &str), output: &'static str) -> Result<Self> {
        let end = start
            .checked_add(matched.len())
            .ok_or(crate::accounting::RenderAccountingError::Invariant)?;
        Ok(Self { start, end, output })
    }
}

fn replace_shader_code<const N: usize>(
    cpu: &CpuAccount,
    source: &str,
    replacements: &[ShaderSourceReplacement; N],
) -> Result<AccountedVec<u8>> {
    cpu.phase(|| {
        let mut edits = [[None; 2]; N];
        for (replacement, slots) in replacements.iter().zip(&mut edits) {
            if replacement.from.is_empty() {
                return Err(render_failed(replacement.failure));
            }
            cpu.work(RenderWork::ShaderSourceScanBytes, source.len())?;
            let mut matches = source.match_indices(replacement.from);
            let first = matches.next();
            let second = matches.next();
            *slots = match (replacement.matches, first, second, matches.count()) {
                (ShaderMatchCount::Once, Some(first), None, 0) => [
                    Some(ShaderSourceEdit::from_match(first, replacement.to)?),
                    None,
                ],
                (ShaderMatchCount::Twice, Some(first), Some(second), 0) => [
                    Some(ShaderSourceEdit::from_match(first, replacement.to)?),
                    Some(ShaderSourceEdit::from_match(second, replacement.to)?),
                ],
                _ => return Err(render_failed(replacement.failure)),
            };
        }
        let edits = edits.as_flattened_mut();
        edits.sort_unstable_by_key(|edit| edit.map_or(usize::MAX, |edit| edit.start));

        let mut previous_end = 0;
        let mut len = source.len();
        for edit in edits.iter().flatten() {
            if edit.start < previous_end {
                return Err(render_failed("internal Vello shader replacements overlap"));
            }
            let matched = source
                .get(edit.start..edit.end)
                .ok_or(crate::accounting::RenderAccountingError::Invariant)?;
            len = len
                .checked_sub(matched.len())
                .and_then(|len| len.checked_add(edit.output.len()))
                .ok_or(crate::accounting::RenderAccountingError::Invariant)?;
            previous_end = edit.end;
        }

        let mut code = AccountedVec::new_in(
            &StorageAccount::Renderer(cpu.clone()),
            len,
            RenderResource::ScratchCpuBytes,
        )?;
        let source = source.as_bytes();
        let mut start = 0;
        for edit in edits.iter().flatten() {
            let unchanged = source
                .get(start..edit.start)
                .ok_or(crate::accounting::RenderAccountingError::Invariant)?;
            append_shader_code(cpu, &mut code, unchanged)?;
            append_shader_code(cpu, &mut code, edit.output.as_bytes())?;
            start = edit.end;
        }
        let tail = source
            .get(start..)
            .ok_or(crate::accounting::RenderAccountingError::Invariant)?;
        append_shader_code(cpu, &mut code, tail)?;
        if code.len() != len {
            return Err(render_failed(
                "internal Vello shader replacements changed checked output length",
            ));
        }
        Ok(code)
    })
}

fn append_shader_code(cpu: &CpuAccount, code: &mut AccountedVec<u8>, bytes: &[u8]) -> Result<()> {
    shader_work(cpu, None, RenderWork::SerializedBytes, bytes.len())?;
    code.extend_from_slice(bytes)
}

fn create_fine_pipeline(
    device: &wgpu::Device,
    generation: &Generation,
    cpu: &CpuAccount,
    shader: &ComputeShader<'_>,
    adapt_msaa8_endpoints: bool,
) -> Result<CheckedComputePipeline> {
    let code = fine_shader_source(cpu, shader, adapt_msaa8_endpoints)?;
    let source = std::str::from_utf8(&code)
        .map_err(|_| render_failed("internal shader source is not UTF-8"))?;
    CheckedComputePipeline::create(device, generation, cpu, None, shader, Some(source))
}

fn shader_work(
    cpu: &CpuAccount,
    observer: Option<&Account>,
    work: RenderWork,
    amount: usize,
) -> Result<()> {
    if let Some(observer) = observer {
        observer.work(work, amount)
    } else {
        cpu.work(work, amount)
    }
}

pub(super) struct CheckedShaderSet {
    cpu: CpuAccount,
    expand_image: CheckedComputePipeline,
    pathtag_reduce: CheckedComputePipeline,
    pathtag_reduce2: CheckedComputePipeline,
    pathtag_scan1: CheckedComputePipeline,
    pathtag_scan: CheckedComputePipeline,
    pathtag_scan_large: CheckedComputePipeline,
    bbox_clear: CheckedComputePipeline,
    flatten: CheckedComputePipeline,
    draw_reduce: CheckedComputePipeline,
    draw_leaf: CheckedComputePipeline,
    clip_reduce: CheckedComputePipeline,
    clip_leaf: CheckedComputePipeline,
    binning: CheckedComputePipeline,
    tile_alloc: CheckedComputePipeline,
    path_count_setup: CheckedComputePipeline,
    path_count: CheckedComputePipeline,
    backdrop: CheckedComputePipeline,
    coarse: CheckedComputePipeline,
    path_tiling_setup: CheckedComputePipeline,
    path_tiling: CheckedComputePipeline,
    fine_area: CheckedComputePipeline,
    fine_msaa8: CheckedComputePipeline,
    fine_msaa16: CheckedComputePipeline,
}

impl CheckedShaderSet {
    pub(super) async fn create(
        device: &wgpu::Device,
        generation: &Generation,
        cpu: &CpuAccount,
    ) -> Result<Self> {
        checked_wgpu_build(device, || {
            // Borrow each rewritten source only for its pipeline creation, then
            // release its scratch before transforming the next fine program.
            // Initial scratch refusal still precedes all native allocations.
            let fine_area =
                create_fine_pipeline(device, generation, cpu, &SHADERS.fine_area, false)?;
            let fine_msaa8 =
                create_fine_pipeline(device, generation, cpu, &SHADERS.fine_msaa8, true)?;
            let fine_msaa16 =
                create_fine_pipeline(device, generation, cpu, &SHADERS.fine_msaa16, false)?;
            let expand_image = vello_shaders::ComputeShader {
                name: "Surgeist nearest image expansion".into(),
                workgroup_size: [8, 8, 1],
                bindings: std::borrow::Cow::Borrowed(&[BindType::BufReadOnly, BindType::Image]),
                workgroup_buffers: std::borrow::Cow::Borrowed(&[]),
                wgsl: vello_shaders::WgslSource {
                    code: include_str!("../shaders/pixelated_expand.wgsl").into(),
                    binding_indices: std::borrow::Cow::Borrowed(&[0, 1]),
                },
            };
            Ok(Self {
                cpu: cpu.clone(),
                expand_image: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &expand_image,
                    None,
                )?,
                pathtag_reduce: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.pathtag_reduce,
                    None,
                )?,
                pathtag_reduce2: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.pathtag_reduce2,
                    None,
                )?,
                pathtag_scan1: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.pathtag_scan1,
                    None,
                )?,
                pathtag_scan: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.pathtag_scan_small,
                    None,
                )?,
                pathtag_scan_large: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.pathtag_scan_large,
                    None,
                )?,
                bbox_clear: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.bbox_clear,
                    None,
                )?,
                flatten: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.flatten,
                    None,
                )?,
                draw_reduce: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.draw_reduce,
                    None,
                )?,
                draw_leaf: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.draw_leaf,
                    None,
                )?,
                clip_reduce: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.clip_reduce,
                    None,
                )?,
                clip_leaf: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.clip_leaf,
                    None,
                )?,
                binning: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.binning,
                    None,
                )?,
                tile_alloc: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.tile_alloc,
                    None,
                )?,
                path_count_setup: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.path_count_setup,
                    None,
                )?,
                path_count: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.path_count,
                    None,
                )?,
                backdrop: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.backdrop_dyn,
                    None,
                )?,
                coarse: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.coarse,
                    None,
                )?,
                path_tiling_setup: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.path_tiling_setup,
                    None,
                )?,
                path_tiling: CheckedComputePipeline::create(
                    device,
                    generation,
                    cpu,
                    None,
                    &SHADERS.path_tiling,
                    None,
                )?,
                fine_area,
                fine_msaa8,
                fine_msaa16,
            })
        })
        .await
    }

    pub(super) fn pipeline(
        &self,
        kernel: RasterKernel,
        observer: Option<&Account>,
    ) -> Result<&CheckedComputePipeline> {
        shader_work(&self.cpu, observer, RenderWork::ResourceProbes, 1)?;
        Ok(match kernel {
            RasterKernel::ExpandImage => &self.expand_image,
            RasterKernel::PathTagReduce => &self.pathtag_reduce,
            RasterKernel::PathTagReduce2 => &self.pathtag_reduce2,
            RasterKernel::PathTagScan1 => &self.pathtag_scan1,
            RasterKernel::PathTagScan => &self.pathtag_scan,
            RasterKernel::PathTagScanLarge => &self.pathtag_scan_large,
            RasterKernel::BboxClear => &self.bbox_clear,
            RasterKernel::Flatten => &self.flatten,
            RasterKernel::DrawReduce => &self.draw_reduce,
            RasterKernel::DrawLeaf => &self.draw_leaf,
            RasterKernel::ClipReduce => &self.clip_reduce,
            RasterKernel::ClipLeaf => &self.clip_leaf,
            RasterKernel::Binning => &self.binning,
            RasterKernel::TileAlloc => &self.tile_alloc,
            RasterKernel::PathCountSetup => &self.path_count_setup,
            RasterKernel::PathCount => &self.path_count,
            RasterKernel::Backdrop => &self.backdrop,
            RasterKernel::Coarse => &self.coarse,
            RasterKernel::PathTilingSetup => &self.path_tiling_setup,
            RasterKernel::PathTiling => &self.path_tiling,
            RasterKernel::FineArea => &self.fine_area,
            RasterKernel::FineMsaa8 => &self.fine_msaa8,
            RasterKernel::FineMsaa16 => &self.fine_msaa16,
        })
    }
}

async fn checked_wgpu_build<T>(
    device: &wgpu::Device,
    build: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let scope = CheckedWgpuScope::begin(device);
    let result = build();
    scope
        .finish("checked internal Vello shader creation failed")
        .await?;
    result
}

fn binding_type(binding: BindType) -> wgpu::BindingType {
    match binding {
        BindType::Buffer => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindType::BufReadOnly => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindType::Uniform => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindType::Image => wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba8Unorm,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        BindType::ImageRead => wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
    }
}

fn render_failed(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

fn classify_checked_wgpu_error(failure_message: &'static str, source: wgpu::Error) -> Error {
    let code = match &source {
        wgpu::Error::OutOfMemory { .. } => BackendErrorCode::SurfaceOutOfMemory,
        wgpu::Error::Validation { .. } | wgpu::Error::Internal { .. } => {
            BackendErrorCode::RenderFailed
        }
    };
    Error::new(code, failure_message).with_source(source)
}

#[cfg(test)]
pub(crate) async fn checked_shader_validation_for_test(device: &wgpu::Device) -> Result<()> {
    let generation = Generation::new(crate::RenderAccountingOptions::default())?;
    checked_wgpu_build(device, || {
        let _module = generation.opaque(GpuObjectKind::ShaderModule, None, || {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Surgeist deliberate invalid internal Vello WGSL"),
                source: wgpu::ShaderSource::Wgsl("@compute @workgroup_size(1) fn main( {".into()),
            })
        })?;
        Ok(())
    })
    .await
}

#[cfg(test)]
pub(crate) fn checked_scope_out_of_memory_for_test() -> Error {
    classify_checked_wgpu_error(
        "checked internal Vello resource or command encoding failed",
        wgpu::Error::OutOfMemory {
            source: Box::new(std::io::Error::other("injected internal Vello OOM")),
        },
    )
}

#[cfg(test)]
mod cpu_tests {
    use super::*;
    use crate::accounting::RendererCpu;
    use crate::{RenderAccountingOptions, RenderCpuScope, RenderLimits};

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn blend_programs_branch_only_on_the_selected_mode() {
        use wgpu::naga::{Expression, Statement, front::wgsl};

        let owner = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let area = fine_shader_source(&cpu, &SHADERS.fine_area, false).unwrap();
        let msaa8 = fine_shader_source(&cpu, &SHADERS.fine_msaa8, true).unwrap();
        let msaa16 = fine_shader_source(&cpu, &SHADERS.fine_msaa16, false).unwrap();
        let mut conditional = Vec::new();
        for (route, entry, source) in [
            ("graph", "blend_rgb", crate::shader::LAYER_COMPOSITE_WGSL),
            ("area", "blend_mix", std::str::from_utf8(&area).unwrap()),
            ("msaa8", "blend_mix", std::str::from_utf8(&msaa8).unwrap()),
            ("msaa16", "blend_mix", std::str::from_utf8(&msaa16).unwrap()),
        ] {
            let module = wgsl::parse_str(source).unwrap();
            let (root, _) = module
                .functions
                .iter()
                .find(|(_, function)| function.name.as_deref() == Some(entry))
                .unwrap();
            let mut pending = vec![root];
            let mut visited = Vec::new();
            while let Some(handle) = pending.pop() {
                if visited.contains(&handle) {
                    continue;
                }
                visited.push(handle);
                let function = &module.functions[handle];
                let mut blocks = vec![&function.body];
                while let Some(block) = blocks.pop() {
                    for statement in block {
                        match statement {
                            Statement::Block(block) => blocks.push(block),
                            Statement::Call { function, .. } => pending.push(*function),
                            Statement::Switch { selector, cases }
                                if handle == root
                                    && matches!(
                                        function.expressions[*selector],
                                        Expression::FunctionArgument(2)
                                    ) =>
                            {
                                blocks.extend(cases.iter().map(|case| &case.body));
                            }
                            Statement::If { .. }
                            | Statement::Loop { .. }
                            | Statement::Switch { .. } => {
                                conditional
                                    .push((route, function.name.as_deref().unwrap().to_owned()));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        assert!(
            conditional.is_empty(),
            "blend control flow outside mode selection: {conditional:?}"
        );
    }

    #[test]
    fn fine_replacements_preserve_the_exact_program_and_release_each_scratch_buffer() {
        let owner = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let mut total_emitted = 0;
        let mut total_scanned = 0;
        let mut largest_capacity = 0;
        for (shader, msaa8) in [
            (&SHADERS.fine_area, false),
            (&SHADERS.fine_msaa8, true),
            (&SHADERS.fine_msaa16, false),
        ] {
            // Exact source preservation is the adapter's artifact contract.
            // Independent GPU fixtures own the numeric blend/SetLum evidence.
            let source = fine_shader_source(&cpu, shader, msaa8).unwrap();
            let mut expected = shader
                .wgsl
                .code
                .replace(SET_LUM_REPLACEMENT.from, SET_LUM_REPLACEMENT.to)
                .replace(BLEND_RESULT_REPLACEMENT.from, BLEND_RESULT_REPLACEMENT.to)
                .replace(NORMAL_BLEND_REPLACEMENT.from, NORMAL_BLEND_REPLACEMENT.to)
                .replace(CIRCULAR_NORM_REPLACEMENT.from, CIRCULAR_NORM_REPLACEMENT.to);
            for replacement in GRADIENT_ENDPOINT_REPLACEMENTS {
                expected = expected.replace(replacement.from, replacement.to);
            }
            for replacement in BLEND_MATH_REPLACEMENTS {
                expected = expected.replace(replacement.from, replacement.to);
            }
            if msaa8 {
                expected = expected
                    .replace(
                        "round(8.0 * (xy0.y - f32(y)))",
                        "round(8.0 * (xy0.y - f32(y)) - 8.0 * ROBUST_EPSILON)",
                    )
                    .replace(
                        "round(8.0 * (xy1.y - f32(y)))",
                        "round(8.0 * (xy1.y - f32(y)) - 8.0 * ROBUST_EPSILON)",
                    );
            }
            assert_eq!(std::str::from_utf8(&source).unwrap(), expected);
            total_emitted += source.len() as u64;
            total_scanned += shader.wgsl.code.len() as u64 * if msaa8 { 15 } else { 13 };
            largest_capacity = largest_capacity.max(source.capacity() as u64);
            let usage = owner.snapshot().unwrap().usage();
            assert_eq!(usage.work(RenderWork::SerializedBytes), total_emitted);
            assert_eq!(usage.work(RenderWork::ShaderSourceScanBytes), total_scanned);
            assert_eq!(
                usage.live(RenderResource::ScratchCpuBytes),
                source.capacity() as u64
            );
            assert_eq!(
                usage.peak(RenderResource::ScratchCpuBytes),
                largest_capacity
            );
            drop(source);
            assert_eq!(
                owner
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::ScratchCpuBytes),
                0
            );
        }
        owner.ensure_healthy().unwrap();
    }

    #[test]
    fn fine_source_cardinality_mismatch_refuses_before_scratch_or_emission() {
        let owner = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let mut scanned = 0;
        for replacement in [
            SET_LUM_REPLACEMENT,
            BLEND_RESULT_REPLACEMENT,
            NORMAL_BLEND_REPLACEMENT,
            CIRCULAR_NORM_REPLACEMENT,
            GRADIENT_ENDPOINT_REPLACEMENTS[0],
            GRADIENT_ENDPOINT_REPLACEMENTS[1],
            GRADIENT_ENDPOINT_REPLACEMENTS[2],
            GRADIENT_ENDPOINT_REPLACEMENTS[3],
        ] {
            for changed_source in [
                "missing helper".to_owned(),
                format!("{}\n{}", replacement.from, replacement.from),
            ] {
                let error = replace_shader_code(&cpu, &changed_source, &[replacement]).unwrap_err();
                scanned += changed_source.len() as u64;
                assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
                assert_eq!(error.message(), replacement.failure);
            }
        }
        let error = replace_shader_code(
            &cpu,
            SET_LUM_REPLACEMENT.from,
            &[
                SET_LUM_REPLACEMENT,
                MSAA8_ENDPOINT_REPLACEMENTS[0],
                MSAA8_ENDPOINT_REPLACEMENTS[1],
            ],
        )
        .unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(error.message(), MSAA8_ENDPOINT_REPLACEMENTS[0].failure);
        scanned += 2 * SET_LUM_REPLACEMENT.from.len() as u64;
        let usage = owner.snapshot().unwrap().usage();
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::SerializedBytes), 0);
        assert_eq!(usage.work(RenderWork::ShaderSourceScanBytes), scanned);
        owner.ensure_healthy().unwrap();
    }

    #[test]
    fn shader_preparation_records_work_in_device_backing() {
        let owner = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_profile(crate::RenderProfile::Summary { timing: false }),
        )
        .unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let replacement = ShaderSourceReplacement {
            from: "old",
            to: "new!",
            matches: ShaderMatchCount::Once,
            failure: "missing source marker",
        };
        let code = replace_shader_code(&cpu, "old.", &[replacement]).unwrap();
        assert_eq!(code.as_slice(), b"new!.");
        drop(code);
        let (indices, entries) = prepare_bindings(&cpu, None, &SHADERS.bbox_clear).unwrap();
        assert_eq!(indices.as_slice(), &[0, 1]);
        assert_eq!(entries.len(), 2);
        drop((indices, entries));

        let snapshot = owner.snapshot().unwrap();
        let backing = snapshot.scope_usage(RenderCpuScope::DeviceBacking).unwrap();
        let cleanup = snapshot.scope_usage(RenderCpuScope::Cleanup).unwrap();
        assert_eq!(
            [
                backing.work(RenderWork::SerializedBytes),
                backing.work(RenderWork::ShaderBindingVisits),
                cleanup.work(RenderWork::SerializedBytes),
                cleanup.work(RenderWork::ShaderBindingVisits),
            ],
            [5, 5, 0, 0],
        );
        assert_eq!(backing.work(RenderWork::ShaderSourceScanBytes), 4);
        assert_eq!(cleanup.work(RenderWork::ShaderSourceScanBytes), 0);

        let frame = Account::for_test();
        drop(prepare_bindings(&cpu, Some(&frame), &SHADERS.bbox_clear).unwrap());
        assert_eq!(
            frame.work_count(RenderWork::ShaderBindingVisits).unwrap(),
            5
        );
        assert_eq!(
            owner
                .snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ShaderBindingVisits),
            5,
        );
    }

    #[test]
    fn overlapping_shader_edits_refuse_before_output_scratch() {
        let owner = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let replacements = [
            ShaderSourceReplacement {
                from: "aaa",
                to: "xxx",
                matches: ShaderMatchCount::Once,
                failure: "missing first marker",
            },
            ShaderSourceReplacement {
                from: "aab",
                to: "yyy",
                matches: ShaderMatchCount::Once,
                failure: "missing second marker",
            },
        ];
        let result = replace_shader_code(&cpu, "aaab", &replacements);
        assert!(
            matches!(result, Err(ref error) if error.code() == crate::ErrorCode::RenderFailed),
            "overlapping equal-width edits must refuse: {result:?}",
        );
        let snapshot = owner.snapshot().unwrap();
        assert_eq!(snapshot.usage().peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(snapshot.usage().work(RenderWork::SerializedBytes), 0);
        assert_eq!(snapshot.usage().work(RenderWork::ShaderSourceScanBytes), 8);
        owner.ensure_healthy().unwrap();
    }

    #[test]
    fn shader_edit_plan_uses_original_utf8_ranges_in_source_order() {
        let owner = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let replacements = [
            ShaderSourceReplacement {
                from: "ab",
                to: "éab",
                matches: ShaderMatchCount::Twice,
                failure: "missing ASCII markers",
            },
            ShaderSourceReplacement {
                from: "é",
                to: "X",
                matches: ShaderMatchCount::Twice,
                failure: "missing UTF-8 markers",
            },
        ];
        let code = replace_shader_code(&cpu, "éab!éab", &replacements).unwrap();
        assert_eq!(code.as_slice(), "Xéab!Xéab".as_bytes());
        let usage = owner.snapshot().unwrap().usage();
        assert_eq!(usage.work(RenderWork::ShaderSourceScanBytes), 18);
        assert_eq!(usage.work(RenderWork::SerializedBytes), 11);
    }

    #[test]
    fn fine_source_scratch_refusal_emits_nothing_and_preserves_healthy_custody() {
        let owner = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_limits(RenderLimits::new().with_scratch_bytes(0)),
        )
        .unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        for (shader, msaa8) in [
            (&SHADERS.fine_area, false),
            (&SHADERS.fine_msaa8, true),
            (&SHADERS.fine_msaa16, false),
        ] {
            let error = fine_shader_source(&cpu, shader, msaa8).unwrap_err();
            assert!(matches!(
                error.accounting_diagnostic(),
                Some(crate::RenderAccountingError::CpuRefused {
                    quantity: crate::RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
                    ..
                })
            ));
        }
        let usage = owner.snapshot().unwrap().usage();
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::SerializedBytes), 0);
        owner.ensure_healthy().unwrap();
    }

    #[test]
    fn borrowed_shader_metadata_owns_only_actual_indices_and_temporary_entries() {
        let owner = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let baseline = owner
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let (indices, entries) = prepare_bindings(&cpu, None, &SHADERS.bbox_clear).unwrap();
        assert_eq!(indices.as_slice(), [0, 1]);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.binding)
                .collect::<Vec<_>>(),
            [0, 1]
        );
        let usage = owner.snapshot().unwrap().usage();
        assert_eq!(
            usage.live(RenderResource::RetainedCpuBytes) - baseline,
            (indices.capacity() * std::mem::size_of::<u32>()) as u64
        );
        assert_eq!(
            usage.live(RenderResource::ScratchCpuBytes),
            (entries.capacity() * std::mem::size_of::<wgpu::BindGroupLayoutEntry>()) as u64
        );
        drop(entries);
        assert_eq!(
            owner
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        assert_eq!(
            owner
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes)
                - baseline,
            (indices.capacity() * std::mem::size_of::<u32>()) as u64
        );
        drop(indices);
        assert_eq!(
            owner
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            baseline
        );
    }

    #[test]
    fn binding_capacity_refusal_keeps_only_initial_cpu_custody_live() {
        let baseline = RendererCpu::new(RenderAccountingOptions::default())
            .unwrap()
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let owner = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_limits(RenderLimits::new().with_retained_bytes(baseline)),
        )
        .unwrap();
        let result = prepare_bindings(
            &owner.at(RenderCpuScope::DeviceBacking),
            None,
            &SHADERS.bbox_clear,
        );
        assert!(result.is_err());
        let usage = owner.snapshot().unwrap().usage();
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), baseline);
        assert_eq!(usage.work(RenderWork::ShaderBindingVisits), 0);
        owner.ensure_healthy().unwrap();
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn shader_initialization_cpu_refusal_makes_no_native_resource_request() {
        let mut renderer =
            pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
        let (device, _) = renderer
            .default_wgpu_device_queue()
            .expect("shader initialization needs a host adapter");
        let options = RenderAccountingOptions::default()
            .with_limits(RenderLimits::new().with_scratch_bytes(0));
        let cpu = RendererCpu::new(options).unwrap();
        let generation = Generation::new(options).unwrap();
        let result = pollster::block_on(CheckedShaderSet::create(
            device,
            &generation,
            &cpu.at(RenderCpuScope::DeviceBacking),
        ));
        assert!(result.is_err());
        assert_eq!(
            generation
                .snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ResourceAllocationRequests),
            0
        );
        assert_eq!(
            generation
                .snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ResourceAllocations),
            0
        );
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        cpu.ensure_healthy().unwrap();
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "shaders/gradient_endpoint_tests.rs"]
mod gradient_endpoint_tests;
