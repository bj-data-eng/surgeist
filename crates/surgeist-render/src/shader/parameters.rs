use super::key::{ShaderMaskQualityKey, ShaderMaskSamplingKey};
use crate::accounting::{
    Account, AccountedVec, RenderResource, RenderWork, SharedValue, StorageAccount,
};
use crate::{
    BackendErrorCode, Color, Error, Point, Rect, Result,
    frame::CompositeSourceSpace,
    layer::{BlendMode, LayerComposition, PorterDuffOperator},
    pass::{
        RuntimeColorClampBoundary, RuntimeColorOperation, RuntimeColorOperationKind,
        RuntimeLayerCompositeParameters, RuntimeSpatialDescriptor,
    },
};

#[cfg(all(test, not(target_arch = "wasm32")))]
use crate::image::{Extend, ImageQuality};

/// Qualified rebased facts shared by Gaussian planning and the exact shader wire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GaussianParameterFacts {
    pub(crate) stage: u32,
    pub(crate) source_size: [u32; 2],
    pub(crate) target_size: [u32; 2],
    pub(crate) source_base: [i32; 2],
    pub(crate) source_clip: [f64; 4],
    pub(crate) output_clip: [f64; 4],
    pub(crate) periodic_base: [f64; 2],
    pub(crate) periodic_step: [f64; 2],
    pub(crate) edge_mode: u32,
    pub(crate) periodic: bool,
    pub(crate) direct: bool,
    pub(crate) axis: u32,
    pub(crate) first_tap: u32,
    pub(crate) tap_count: u32,
    pub(crate) first: bool,
    pub(crate) linear: bool,
    pub(crate) converted: bool,
    pub(crate) color: [f32; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GaussianParameterBytes([u8; 256]);

impl GaussianParameterBytes {
    pub(crate) fn try_new(facts: GaussianParameterFacts, account: &Account) -> Result<Self> {
        let invalid = |value| {
            Error::from(crate::GaussianPreparationError::Numeric {
                phase: crate::GaussianNumericPhase::GridMapping,
                value,
            })
        };
        if facts.stage > 5
            || facts.edge_mode > 3
            || facts.axis > 1
            || (facts.stage == 3
                && (facts.tap_count == 0
                    || facts.tap_count > 32
                    || facts.first_tap.checked_add(facts.tap_count).is_none()))
        {
            return Err(invalid(f64::from(facts.stage)));
        }
        for size in facts.target_size.into_iter().chain(if facts.stage == 5 {
            [0; 2]
        } else {
            facts.source_size
        }) {
            // Every reached fragment center and its unit-cell boundary must be
            // represented exactly; frame qualification proves map/tap endpoints.
            if size > 8_388_608 {
                return Err(invalid(f64::from(size)));
            }
        }
        let mut words = [[0_u32; 4]; 16];
        words[0] = [
            facts.source_size[0],
            facts.source_size[1],
            facts.target_size[0],
            facts.target_size[1],
        ];
        words[1] = [
            u32::from_ne_bytes(facts.source_base[0].to_ne_bytes()),
            u32::from_ne_bytes(facts.source_base[1].to_ne_bytes()),
            0,
            0,
        ];
        let mut floats = |row: usize, values: [f64; 4]| -> Result<()> {
            for (slot, value) in words[row].iter_mut().zip(values) {
                let uploaded = account.coordinate(|| value as f32)?;
                if !value.is_finite()
                    || !(uploaded == 0.0 || uploaded.is_normal())
                    || (value != 0.0 && uploaded == 0.0)
                {
                    return Err(invalid(value));
                }
                *slot = uploaded.to_bits();
            }
            Ok(())
        };
        let needs_source_clip = !facts.periodic && matches!(facts.stage, 1 | 2);
        let needs_output_clip = matches!(facts.stage, 1 | 4 | 5);
        for (needed, row, clip, size) in [
            (needs_source_clip, 2, facts.source_clip, facts.source_size),
            (needs_output_clip, 3, facts.output_clip, facts.target_size),
        ] {
            if needed {
                if clip[0] < 0.0
                    || clip[1] < 0.0
                    || clip[2] < clip[0]
                    || clip[3] < clip[1]
                    || clip[2] > f64::from(size[0])
                    || clip[3] > f64::from(size[1])
                {
                    return Err(invalid(clip[0]));
                }
                floats(row, clip)?;
            }
        }
        if facts.periodic && matches!(facts.stage, 1 | 4) && !facts.source_size.contains(&0) {
            if facts.periodic_step.into_iter().any(|value| value <= 0.0) {
                return Err(invalid(facts.periodic_step[0]));
            }
            floats(
                4,
                [
                    facts.periodic_base[0],
                    facts.periodic_base[1],
                    facts.periodic_step[0],
                    facts.periodic_step[1],
                ],
            )?;
        }
        words[5] = [facts.stage, facts.first_tap, facts.tap_count, facts.axis];
        words[6] = [
            facts.edge_mode,
            u32::from(facts.periodic),
            u32::from(facts.direct),
            u32::from(facts.first),
        ];
        words[7] = [u32::from(facts.converted), u32::from(facts.linear), 0, 0];
        if facts.stage == 5 {
            for value in facts.color {
                if !value.is_finite() || value < 0.0 || value > 1.0 {
                    return Err(invalid(f64::from(value)));
                }
            }
            if facts.color[..3].iter().any(|value| *value > facts.color[3]) {
                return Err(invalid(f64::from(facts.color[3])));
            }
            words[8] = facts.color.map(f32::to_bits);
        }
        account.work(RenderWork::SerializedBytes, 256)?;
        let mut bytes = [0; 256];
        for (index, word) in words.into_iter().flatten().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        Ok(Self(bytes))
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 256] {
        &self.0
    }
}

/// Qualified rebased texel coordinates for the exact offset uniform block.
pub(crate) struct OffsetParameterFacts {
    pub(crate) translation: [f64; 2],
    pub(crate) ratio: [f64; 2],
    pub(crate) source_clip: [f64; 4],
    pub(crate) result_clip: [f64; 4],
    pub(crate) source_extent: [u32; 2],
    pub(crate) result_extent: [u32; 2],
    pub(crate) sampling: u32,
    pub(crate) rebase: bool,
}

/// Fixed inline bytes: map, source clip, result clip, extents, modes, reserved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OffsetParameterBytes([u8; 256]);

impl OffsetParameterBytes {
    pub(crate) fn try_new(facts: OffsetParameterFacts, account: &Account) -> Result<Self> {
        let invalid = || {
            Error::invalid_value(
                "offset parameters",
                "runtime offset",
                "must fit the qualified shader coordinate and sampling domains",
            )
        };
        if facts.sampling > 3
            || facts.ratio.into_iter().any(|value| value <= 0.0)
            || facts
                .source_extent
                .into_iter()
                .chain(facts.result_extent)
                .any(|value| value > 8_388_608)
            || (facts.rebase && facts.sampling != 0)
        {
            return Err(invalid());
        }
        let mut words = [[0_u32; 4]; 16];
        for (row, values) in [
            [
                facts.translation[0],
                facts.translation[1],
                facts.ratio[0],
                facts.ratio[1],
            ],
            facts.source_clip,
            facts.result_clip,
        ]
        .into_iter()
        .enumerate()
        {
            for (slot, value) in words[row].iter_mut().zip(values) {
                let uploaded = account.coordinate(|| value as f32)?;
                if !value.is_finite()
                    || !(uploaded == 0.0 || uploaded.is_normal())
                    || (value != 0.0 && uploaded == 0.0)
                {
                    return Err(invalid());
                }
                *slot = uploaded.to_bits();
            }
        }
        // Copy modes are admitted only after the frame owner proves an aligned
        // lattice. Check that the actual uploaded map retains integer copying.
        if facts.sampling == 0
            && (words[0][2..] != [1.0_f32.to_bits(); 2]
                || words[0][..2]
                    .iter()
                    .any(|value| f32::from_bits(*value).fract() != 0.0))
        {
            return Err(invalid());
        }
        for (clip, extent, row) in [
            (facts.source_clip, facts.source_extent, 1),
            (facts.result_clip, facts.result_extent, 2),
        ] {
            if clip[0] < 0.0
                || clip[1] < 0.0
                || clip[2] < clip[0]
                || clip[3] < clip[1]
                || clip[2] > f64::from(extent[0])
                || clip[3] > f64::from(extent[1])
                || f32::from_bits(words[row][2]) < f32::from_bits(words[row][0])
                || f32::from_bits(words[row][3]) < f32::from_bits(words[row][1])
            {
                return Err(invalid());
            }
        }
        words[3] = [
            facts.source_extent[0],
            facts.source_extent[1],
            facts.result_extent[0],
            facts.result_extent[1],
        ];
        words[4] = [facts.sampling, u32::from(facts.rebase), 0, 0];
        account.work(RenderWork::SerializedBytes, 256)?;
        let mut bytes = [0; 256];
        for (index, word) in words.into_iter().flatten().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        Ok(Self(bytes))
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 256] {
        &self.0
    }
}

/// Validated runtime facts for the exact 256-byte convolution uniform block.
pub(crate) struct ConvolutionParameterFacts {
    pub(crate) capture_origin: [f64; 2],
    pub(crate) capture_scale: f64,
    pub(crate) kernel_origin: [f64; 2],
    pub(crate) unit_length: [f64; 2],
    pub(crate) source_grid_origin: [i32; 2],
    pub(crate) source_grid_extent: [u32; 2],
    pub(crate) result_grid_origin: [i32; 2],
    pub(crate) result_grid_extent: [u32; 2],
    pub(crate) source_domain: Rect,
    pub(crate) result_domain: Rect,
    pub(crate) result_capture_origin: [f64; 2],
    pub(crate) result_capture_scale: f64,
    pub(crate) order: [u32; 2],
    pub(crate) target: [u32; 2],
    pub(crate) stage: u32,
    pub(crate) first_tap: u32,
    pub(crate) tap_count: u32,
    pub(crate) edge: u32,
    pub(crate) preserve_alpha: bool,
    pub(crate) linear_srgb: bool,
    pub(crate) first_batch: bool,
    pub(crate) bias: f64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConvolutionParameterBytes([u8; 256]);

impl ConvolutionParameterBytes {
    pub(crate) fn try_new(facts: ConvolutionParameterFacts, account: &Account) -> Result<Self> {
        let invalid = || {
            Error::invalid_value(
                "convolution parameters",
                "runtime convolution",
                "must fit the validated shader coordinate and stage domains",
            )
        };
        if facts.stage > 5
            || facts.edge > 3
            || facts.tap_count > 32
            || facts.order.contains(&0)
            || facts.source_domain.width() < 0.0
            || facts.source_domain.height() < 0.0
            || facts.result_domain.width() < 0.0
            || facts.result_domain.height() < 0.0
            || (facts.stage != 4
                && (facts.target[0] >= facts.order[0] || facts.target[1] >= facts.order[1]))
            || facts.first_tap.checked_add(facts.tap_count).is_none()
            || facts
                .source_grid_extent
                .into_iter()
                .chain(facts.result_grid_extent)
                .any(|v| v > i32::MAX as u32)
            || facts.capture_scale <= 0.0
            || facts.result_capture_scale <= 0.0
            || facts.unit_length.into_iter().any(|v| v <= 0.0)
        {
            return Err(invalid());
        }
        let mut words = [[0_u32; 4]; 16];
        let mut floats = |row: usize, values: [f64; 4]| -> Result<()> {
            for (slot, value) in words[row].iter_mut().zip(values) {
                let value = account.coordinate(|| value as f32)?;
                if !value.is_finite() {
                    return Err(invalid());
                }
                *slot = value.to_bits();
            }
            Ok(())
        };
        floats(
            0,
            [
                facts.capture_origin[0],
                facts.capture_origin[1],
                facts.capture_scale,
                0.0,
            ],
        )?;
        floats(
            1,
            [
                facts.kernel_origin[0],
                facts.kernel_origin[1],
                facts.unit_length[0],
                facts.unit_length[1],
            ],
        )?;
        floats(
            4,
            [
                facts.source_domain.x(),
                facts.source_domain.y(),
                account.coordinate(|| facts.source_domain.x() + facts.source_domain.width())?,
                account.coordinate(|| facts.source_domain.y() + facts.source_domain.height())?,
            ],
        )?;
        floats(
            5,
            [
                facts.result_domain.x(),
                facts.result_domain.y(),
                account.coordinate(|| facts.result_domain.x() + facts.result_domain.width())?,
                account.coordinate(|| facts.result_domain.y() + facts.result_domain.height())?,
            ],
        )?;
        floats(
            6,
            [
                facts.result_capture_origin[0],
                facts.result_capture_origin[1],
                facts.result_capture_scale,
                0.0,
            ],
        )?;
        floats(10, [facts.bias, 0.0, 0.0, 0.0])?;
        // Positive f64 spatial values must remain positive after wire narrowing.
        if f32::from_bits(words[0][2]) <= 0.0
            || f32::from_bits(words[6][2]) <= 0.0
            || f32::from_bits(words[1][2]) <= 0.0
            || f32::from_bits(words[1][3]) <= 0.0
        {
            return Err(invalid());
        }
        for (row, origin, extent) in [
            (2, facts.source_grid_origin, facts.source_grid_extent),
            (3, facts.result_grid_origin, facts.result_grid_extent),
        ] {
            words[row] = [
                u32::from_ne_bytes(origin[0].to_ne_bytes()),
                u32::from_ne_bytes(origin[1].to_ne_bytes()),
                extent[0],
                extent[1],
            ];
        }
        words[7] = [
            facts.order[0],
            facts.order[1],
            facts.target[0],
            facts.target[1],
        ];
        words[8] = [facts.stage, facts.first_tap, facts.tap_count, facts.edge];
        words[9] = [
            u32::from(facts.preserve_alpha),
            u32::from(facts.linear_srgb),
            u32::from(facts.first_batch),
            0,
        ];
        account.work(RenderWork::SerializedBytes, 256)?;
        let mut bytes = [0; 256];
        for (index, word) in words.into_iter().flatten().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        Ok(Self(bytes))
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 256] {
        &self.0
    }
}

const COLOR_FILTER_OPERATION_HEADER_BYTE_LEN: usize = 16;
const COLOR_FILTER_OPERATION_RECORD_BYTE_LEN: usize = 32;
type ColorFilterOperationHeader = [u8; COLOR_FILTER_OPERATION_HEADER_BYTE_LEN];
type ColorFilterOperationRecord = [u8; COLOR_FILTER_OPERATION_RECORD_BYTE_LEN];

/// Exact 32-byte WGSL drop-shadow parameter block.
///
/// Bytes `0..8` retain the continuous logical offset, bytes `8..16` are zero
/// alignment, and bytes `16..32` contain the finite solid premultiplied color.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DropShadowParameterBytes([u8; 32]);

impl DropShadowParameterBytes {
    pub(crate) fn try_new(offset: Point, color: Color, observer: Option<&Account>) -> Result<Self> {
        let offset_x = narrow_drop_shadow_scalar("drop shadow offset x", offset.x())?;
        let offset_y = narrow_drop_shadow_scalar("drop shadow offset y", offset.y())?;
        let channels = [color.r(), color.g(), color.b(), color.a()];
        if channels
            .into_iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err(drop_shadow_parameter_error(
                "drop shadow solid color",
                "must contain finite unit-interval channels",
            ));
        }
        let premultiplied = [
            color.r() * color.a(),
            color.g() * color.a(),
            color.b() * color.a(),
            color.a(),
        ];
        if let Some(observer) = observer {
            observer.work(RenderWork::SerializedBytes, 32)?;
        }
        let mut bytes = [0_u8; 32];
        bytes[0..4].copy_from_slice(&offset_x.to_le_bytes());
        bytes[4..8].copy_from_slice(&offset_y.to_le_bytes());
        for (index, value) in premultiplied.into_iter().enumerate() {
            bytes[16 + index * 4..20 + index * 4].copy_from_slice(&value.to_le_bytes());
        }
        Ok(Self(bytes))
    }

    #[must_use]
    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
pub(crate) fn drop_shadow_parameter_bytes_for_test(
    offset: Point,
    color: Color,
) -> Result<[u8; 32]> {
    DropShadowParameterBytes::try_new(offset, color, None).map(|bytes| bytes.0)
}

fn narrow_drop_shadow_scalar(field: &'static str, value: f64) -> Result<f32> {
    let narrowed = value as f32;
    if !narrowed.is_finite() {
        return Err(drop_shadow_parameter_error(
            field,
            "must remain finite after f64-to-f32 narrowing",
        ));
    }
    Ok(narrowed)
}

fn drop_shadow_parameter_error(field: &'static str, invariant: &'static str) -> Error {
    Error::invalid_value(field, "runtime drop shadow", invariant)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ColorFilterOperationBufferLimits {
    max_buffer_size: u64,
    max_storage_buffer_binding_size: u64,
}

impl ColorFilterOperationBufferLimits {
    #[must_use]
    pub(crate) fn from_device_limits(limits: &wgpu::Limits) -> Self {
        Self {
            max_buffer_size: limits.max_buffer_size,
            max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn for_test(
        max_buffer_size: u64,
        max_storage_buffer_binding_size: u64,
    ) -> Self {
        Self {
            max_buffer_size,
            max_storage_buffer_binding_size,
        }
    }
}

/// Exact checked storage-buffer bytes for one ordered color-filter run.
///
/// Bytes `0..16` are the operation count followed by three zero `u32` pads.
/// Every following 32-byte record stores tag, zero flag, exponent, one zero
/// alignment word, a four-scalar finite payload, and no other state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ColorFilterOperationBytes {
    bytes: SharedValue<Vec<u8>>,
}

impl ColorFilterOperationBytes {
    pub(crate) fn matches_in(&self, other: &Self, account: &Account) -> Result<bool> {
        if self.bytes.len() != other.bytes.len() {
            return Ok(false);
        }
        if self.bytes.same_allocation(&other.bytes) {
            return Ok(true);
        }
        // The sole serializer owns a 16-byte header followed by complete
        // 32-byte operation records. Compare their exact bytes, including pads,
        // while admitting each reached operation separately from serialization.
        let (header, records) = self.wire_parts()?;
        let (other_header, other_records) = other.wire_parts()?;
        if header != other_header {
            return Ok(false);
        }
        for (actual, expected) in records.iter().zip(other_records) {
            account.work(RenderWork::FilterOperationVisits, 1)?;
            if actual != expected {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn wire_parts(&self) -> Result<(&ColorFilterOperationHeader, &[ColorFilterOperationRecord])> {
        let (header, body) = self
            .bytes
            .split_first_chunk::<COLOR_FILTER_OPERATION_HEADER_BYTE_LEN>()
            .ok_or_else(color_filter_wire_partition_error)?;
        let (records, []) = body.as_chunks::<COLOR_FILTER_OPERATION_RECORD_BYTE_LEN>() else {
            return Err(color_filter_wire_partition_error());
        };
        Ok((header, records))
    }

    pub(crate) fn try_from_runtime_operations_with_limits(
        account: &Account,
        operations: &[RuntimeColorOperation],
        limits: ColorFilterOperationBufferLimits,
    ) -> Result<Self> {
        Self::serialize(&account.storage(), Some(account), operations, limits)
    }
    fn serialize(
        storage: &StorageAccount,
        observer: Option<&Account>,
        operations: &[RuntimeColorOperation],
        limits: ColorFilterOperationBufferLimits,
    ) -> Result<Self> {
        let operation_count = u32::try_from(operations.len())
            .map_err(|_| color_filter_operation_count_error(operations.len()))?;
        let byte_len = checked_color_filter_operation_byte_len(operation_count, limits)?;
        let byte_len = usize::try_from(byte_len).map_err(|_| {
            color_filter_operation_byte_len_error(
                byte_len,
                "must fit the host address space after device-limit validation",
            )
        })?;

        let mut bytes = AccountedVec::new_in(storage, byte_len, RenderResource::RetainedCpuBytes)?;
        bytes.resize(byte_len, 0)?;
        let (header, body) = bytes
            .split_first_chunk_mut::<COLOR_FILTER_OPERATION_HEADER_BYTE_LEN>()
            .ok_or_else(color_filter_wire_partition_error)?;
        let (records, []) = body.as_chunks_mut::<COLOR_FILTER_OPERATION_RECORD_BYTE_LEN>() else {
            return Err(color_filter_wire_partition_error());
        };
        if records.len() != operations.len() {
            return Err(color_filter_wire_partition_error());
        }
        serialization_work(
            storage,
            observer,
            RenderWork::SerializedBytes,
            COLOR_FILTER_OPERATION_HEADER_BYTE_LEN,
        )?;
        header[..4].copy_from_slice(&operation_count.to_le_bytes());

        for (slot, operation) in records.iter_mut().zip(operations.iter().copied()) {
            serialization_work(storage, observer, RenderWork::FilterOperationVisits, 1)?;
            let record = serialize_color_filter_operation(operation)?;
            serialization_work(
                storage,
                observer,
                RenderWork::SerializedBytes,
                COLOR_FILTER_OPERATION_RECORD_BYTE_LEN,
            )?;
            *slot = record;
        }
        Ok(Self {
            bytes: SharedValue::from_accounted(bytes, storage)?,
        })
    }

    #[must_use]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[cfg(test)]
    pub(crate) fn try_from_runtime_operations_for_test(
        operations: &[RuntimeColorOperation],
        limits: ColorFilterOperationBufferLimits,
    ) -> Result<Self> {
        let owner = crate::accounting::RendererCpu::new(crate::RenderAccountingOptions::default())?;
        let storage = StorageAccount::Renderer(owner.at(crate::RenderCpuScope::DeviceBacking));
        Self::serialize(&storage, None, operations, limits)
    }
}

fn color_filter_wire_partition_error() -> Error {
    Error::new(
        BackendErrorCode::RenderFailed,
        "color filter storage must contain one header and complete operation records",
    )
}

fn serialization_work(
    storage: &StorageAccount,
    observer: Option<&Account>,
    work: RenderWork,
    amount: usize,
) -> Result<()> {
    if let Some(observer) = observer {
        return observer.work(work, amount);
    }
    match storage {
        StorageAccount::Renderer(cpu) => {
            cpu.maintenance(
                work,
                u64::try_from(amount)
                    .map_err(|_| crate::accounting::RenderAccountingError::Invariant)?,
            );
            Ok(())
        }
        StorageAccount::Preparation(account) => account.work(work, amount),
    }
}

fn checked_color_filter_operation_byte_len(
    operation_count: u32,
    limits: ColorFilterOperationBufferLimits,
) -> Result<u64> {
    let records = (COLOR_FILTER_OPERATION_RECORD_BYTE_LEN as u64)
        .checked_mul(u64::from(operation_count))
        .ok_or_else(|| {
            color_filter_operation_byte_len_error(
                operation_count,
                "must produce a checked u64 record byte length",
            )
        })?;
    let byte_len = (COLOR_FILTER_OPERATION_HEADER_BYTE_LEN as u64)
        .checked_add(records)
        .ok_or_else(|| {
            color_filter_operation_byte_len_error(
                operation_count,
                "must produce a checked u64 header-plus-record byte length",
            )
        })?;
    if byte_len > limits.max_buffer_size || byte_len > limits.max_storage_buffer_binding_size {
        return Err(color_filter_operation_byte_len_error(
            byte_len,
            "must not exceed max_buffer_size or max_storage_buffer_binding_size",
        ));
    }
    Ok(byte_len)
}

fn serialize_color_filter_operation(
    operation: RuntimeColorOperation,
) -> Result<ColorFilterOperationRecord> {
    if operation.clamp_boundary()
        != RuntimeColorClampBoundary::ClampStraightRgbaToUnitThenPremultiply
    {
        return Err(Error::invalid_value(
            "color filter operation clamp boundary",
            "unsupported runtime boundary",
            "must clamp straight RGBA and premultiply after every operation",
        ));
    }

    let (tag, zero, exponent, payload) = match operation.operation() {
        RuntimeColorOperationKind::Brightness(amount) => {
            let (zero, exponent, mantissa) = checked_runtime_amount_parts(amount)?;
            (0_u32, zero, exponent, [mantissa, 0.0, 0.0, 0.0])
        }
        RuntimeColorOperationKind::Contrast(amount) => {
            let (zero, exponent, mantissa) = checked_runtime_amount_parts(amount)?;
            (1_u32, zero, exponent, [mantissa, 0.0, 0.0, 0.0])
        }
        RuntimeColorOperationKind::Grayscale(amount) => (
            2_u32,
            0,
            0,
            [checked_runtime_unit(amount.value())?, 0.0, 0.0, 0.0],
        ),
        RuntimeColorOperationKind::HueRotate(angle) => {
            let sine = checked_color_filter_payload(angle.sine())?;
            let cosine = checked_color_filter_payload(angle.cosine())?;
            (3_u32, 0, 0, [sine, cosine, 0.0, 0.0])
        }
        RuntimeColorOperationKind::Invert(amount) => (
            4_u32,
            0,
            0,
            [checked_runtime_unit(amount.value())?, 0.0, 0.0, 0.0],
        ),
        RuntimeColorOperationKind::Opacity(amount) => (
            5_u32,
            0,
            0,
            [checked_runtime_unit(amount.value())?, 0.0, 0.0, 0.0],
        ),
        RuntimeColorOperationKind::Saturate(amount) => {
            let (zero, exponent, mantissa) = checked_runtime_amount_parts(amount)?;
            (6_u32, zero, exponent, [mantissa, 0.0, 0.0, 0.0])
        }
        RuntimeColorOperationKind::Sepia(amount) => (
            7_u32,
            0,
            0,
            [checked_runtime_unit(amount.value())?, 0.0, 0.0, 0.0],
        ),
    };

    let mut bytes = [0_u8; COLOR_FILTER_OPERATION_RECORD_BYTE_LEN];
    bytes[0..4].copy_from_slice(&tag.to_le_bytes());
    bytes[4..8].copy_from_slice(&zero.to_le_bytes());
    bytes[8..12].copy_from_slice(&exponent.to_le_bytes());
    for (index, value) in payload.into_iter().enumerate() {
        let offset = 16 + index * 4;
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}

fn checked_runtime_amount_parts(
    amount: crate::filter::RuntimeFilterAmount,
) -> Result<(u32, i32, f32)> {
    if amount.zero() {
        if amount.mantissa() != 0.0 || amount.exponent() != 0 {
            return Err(color_filter_operation_scalar_error(
                "zero amount must carry zero mantissa and exponent",
            ));
        }
        return Ok((1, 0, 0.0));
    }
    let mantissa = checked_color_filter_payload(amount.mantissa())?;
    if !(0.5..1.0).contains(&mantissa) {
        return Err(color_filter_operation_scalar_error(
            "nonzero amount mantissa must be normalized to [0.5, 1)",
        ));
    }
    Ok((0, amount.exponent(), mantissa))
}

fn checked_runtime_unit(value: f32) -> Result<f32> {
    let value = checked_color_filter_payload(value)?;
    if !(0.0..=1.0).contains(&value) {
        return Err(color_filter_operation_scalar_error(
            "unit amount must remain in the inclusive unit interval",
        ));
    }
    Ok(value)
}

fn checked_color_filter_payload(value: f32) -> Result<f32> {
    if !value.is_finite() {
        return Err(color_filter_operation_scalar_error(
            "every scalar payload must be finite",
        ));
    }
    Ok(value)
}

fn color_filter_operation_count_error(value: impl std::fmt::Display) -> Error {
    Error::invalid_value(
        "color filter operation count",
        value,
        "must fit in u32 before operation-buffer byte calculation",
    )
}

fn color_filter_operation_byte_len_error(
    value: impl std::fmt::Display,
    invariant: &'static str,
) -> Error {
    Error::invalid_value(
        "color filter operation buffer byte length",
        value,
        invariant,
    )
}

fn color_filter_operation_scalar_error(invariant: &'static str) -> Error {
    Error::invalid_value(
        "color filter operation scalar payload",
        "runtime operation",
        invariant,
    )
}

#[cfg(test)]
pub(crate) fn color_filter_operation_byte_len_for_test(
    operation_count: u64,
    limits: ColorFilterOperationBufferLimits,
) -> Result<u64> {
    let operation_count = u32::try_from(operation_count)
        .map_err(|_| color_filter_operation_count_error(operation_count))?;
    checked_color_filter_operation_byte_len(operation_count, limits)
}

/// Exact 48-byte WGSL spatial uniform with explicit little-endian encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PassSpatialUniformBytes([u8; 48]);

impl PassSpatialUniformBytes {
    pub(crate) fn try_from_runtime_spatial_descriptors(
        source: RuntimeSpatialDescriptor,
        destination: RuntimeSpatialDescriptor,
        observer: Option<&Account>,
    ) -> Result<Self> {
        let source_origin = source.texel_origin();
        let source_origin_x =
            narrow_spatial_scalar("pass spatial source origin x", source_origin.x())?;
        let source_origin_y =
            narrow_spatial_scalar("pass spatial source origin y", source_origin.y())?;
        let source_raster_scale =
            narrow_raster_scale("pass spatial source raster scale", source.raster_scale())?;

        let destination_origin = destination.texel_origin();
        let destination_origin_x =
            narrow_spatial_scalar("pass spatial destination origin x", destination_origin.x())?;
        let destination_origin_y =
            narrow_spatial_scalar("pass spatial destination origin y", destination_origin.y())?;
        let destination_raster_scale = narrow_raster_scale(
            "pass spatial destination raster scale",
            destination.raster_scale(),
        )?;

        let source_extent = source.device_extent();
        let destination_extent = destination.device_extent();
        if let Some(observer) = observer {
            observer.work(RenderWork::SerializedBytes, 48)?;
        }
        let mut bytes = [0_u8; 48];
        bytes[0..4].copy_from_slice(&source_origin_x.to_le_bytes());
        bytes[4..8].copy_from_slice(&source_origin_y.to_le_bytes());
        bytes[8..12].copy_from_slice(&source_raster_scale.to_le_bytes());
        bytes[16..20].copy_from_slice(&destination_origin_x.to_le_bytes());
        bytes[20..24].copy_from_slice(&destination_origin_y.to_le_bytes());
        bytes[24..28].copy_from_slice(&destination_raster_scale.to_le_bytes());
        bytes[32..36].copy_from_slice(&source_extent.width().to_le_bytes());
        bytes[36..40].copy_from_slice(&source_extent.height().to_le_bytes());
        bytes[40..44].copy_from_slice(&destination_extent.width().to_le_bytes());
        bytes[44..48].copy_from_slice(&destination_extent.height().to_le_bytes());
        Ok(Self(bytes))
    }

    #[must_use]
    pub(crate) const fn as_bytes(&self) -> &[u8; 48] {
        &self.0
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn into_bytes_for_test(self) -> [u8; 48] {
        self.0
    }
}

/// Exact semantic backdrop mirror rectangle in logical coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BlurEdgeParameterBytes([u8; 16]);

impl BlurEdgeParameterBytes {
    pub(crate) fn try_from_semantic_bounds(
        bounds: Rect,
        observer: Option<&Account>,
    ) -> Result<Self> {
        let minimum_x = narrow_spatial_scalar("blur semantic mirror minimum x", bounds.x())?;
        let minimum_y = narrow_spatial_scalar("blur semantic mirror minimum y", bounds.y())?;
        let maximum_x = narrow_spatial_scalar(
            "blur semantic mirror maximum x",
            bounds.x() + bounds.width(),
        )?;
        let maximum_y = narrow_spatial_scalar(
            "blur semantic mirror maximum y",
            bounds.y() + bounds.height(),
        )?;
        if maximum_x <= minimum_x || maximum_y <= minimum_y {
            return Err(Error::invalid_value(
                "blur semantic mirror bounds",
                format!("{bounds:?}"),
                "must narrow to a finite positive logical rectangle",
            ));
        }
        if let Some(observer) = observer {
            observer.work(RenderWork::SerializedBytes, 16)?;
        }
        let mut bytes = [0_u8; 16];
        for (index, value) in [minimum_x, minimum_y, maximum_x, maximum_y]
            .into_iter()
            .enumerate()
        {
            let offset = index * 4;
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        Ok(Self(bytes))
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// Exact 128-byte WGSL composite parameter block.
///
/// The byte ranges are fixed as follows: source-texel linear coefficients `0..16`,
/// source-texel and mask-normalized translations `16..32`, mask-normalized
/// linear coefficients `32..48`, image dimensions and pixelated lattice factors `48..64`, normalized
/// texel-center facts `64..80`, opacity/blend/quality/extend `80..96`, and
/// clip presence, mask value and application bits, source coordinate space and basic operator code `96..112`.
/// Gaussian source-cell operation coverage occupies `112..128`; its presence is bit 1
/// of the source-space word. Other sources retain a zero row.
/// SourceOver is operator code zero; the separate local Plus mix code ignores it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompositeParameterBytes([u8; 128]);

#[derive(Clone, Copy, Debug, PartialEq)]
struct CompositeMaskParameterFacts {
    bounds: [f64; 4],
    dimensions: [u32; 2],
    texel_center_facts: [f64; 4],
    sampling: ShaderMaskSamplingKey,
    value: crate::layer::MaskValue,
    application: crate::layer::MaskApplication,
}

#[derive(Clone, Copy)]
struct CompositeGridFacts {
    origin: [f64; 2],
    scale: f64,
    extent: [u32; 2],
}

impl CompositeGridFacts {
    fn from_runtime(spatial: RuntimeSpatialDescriptor) -> Self {
        let origin = spatial.texel_origin();
        let extent = spatial.device_extent();
        Self {
            origin: [origin.x(), origin.y()],
            scale: spatial.raster_scale(),
            extent: [extent.width(), extent.height()],
        }
    }

    #[cfg(test)]
    const fn unit() -> Self {
        Self {
            origin: [0.0; 2],
            scale: 1.0,
            extent: [1; 2],
        }
    }
}

#[derive(Clone, Copy)]
struct CompositeMap {
    linear: [f32; 4],
    translation: [f32; 2],
}

struct CompositeParameterFacts {
    affine: [f64; 6],
    source_space: CompositeSourceSpace,
    source_operation_clip: Option<[f64; 4]>,
    opacity: f32,
    composition: LayerComposition,
    has_clip: bool,
    mask: Option<CompositeMaskParameterFacts>,
    source: CompositeGridFacts,
    destination: CompositeGridFacts,
}

struct NarrowedCompositeMask {
    mapping: CompositeMap,
    dimensions: [u32; 2],
    pixelated: Option<[u32; 2]>,
    texel_centers: [f32; 4],
    sampling: ShaderMaskSamplingKey,
    value: crate::layer::MaskValue,
    application: crate::layer::MaskApplication,
}

fn pixelated_mask_multipliers(
    destination_to_local: [f64; 6],
    mask: &CompositeMaskParameterFacts,
    raster_scale: f64,
    account: &Account,
) -> Result<[u32; 2]> {
    let a = account.coordinate(|| destination_to_local[0])?;
    let b = account.coordinate(|| destination_to_local[1])?;
    let c = account.coordinate(|| destination_to_local[2])?;
    let d = account.coordinate(|| destination_to_local[3])?;
    let ad = account.coordinate(|| a * d)?;
    let bc = account.coordinate(|| b * c)?;
    let determinant = account.coordinate(|| ad - bc)?;
    let inverse_determinant = account.coordinate(|| determinant.recip())?;
    let negative_inverse = account.coordinate(|| -inverse_determinant)?;
    let local_to_destination = [
        account.coordinate(|| inverse_determinant * d)?,
        account.coordinate(|| negative_inverse * b)?,
        account.coordinate(|| negative_inverse * c)?,
        account.coordinate(|| inverse_determinant * a)?,
    ];

    // The factor producer only uses the linear part of each transform. Keep
    // the Kurbo coefficient expressions, including their zero terms.
    let raster_scale = account.coordinate(|| raster_scale)?;
    let local_to_device = linear_product(
        [raster_scale, 0.0, 0.0, raster_scale],
        local_to_destination,
        account,
    )?;
    let width = account.coordinate(|| f64::from(mask.dimensions[0]))?;
    let bounds_width = account.coordinate(|| mask.bounds[2])?;
    let sx = account.coordinate(|| bounds_width / width)?;
    let height = account.coordinate(|| f64::from(mask.dimensions[1]))?;
    let bounds_height = account.coordinate(|| mask.bounds[3])?;
    let sy = account.coordinate(|| bounds_height / height)?;
    let [a, b, c, d] = linear_product(local_to_device, [sx, 0.0, 0.0, sy], account)?;
    crate::image::pixelated_multipliers(kurbo::Affine::new([a, b, c, d, 0.0, 0.0]), account)
}

fn linear_product(left: [f64; 4], right: [f64; 4], account: &Account) -> Result<[f64; 4]> {
    account.work(RenderWork::CoordinateVisits, 8)?;
    let [a, b, c, d] = left;
    let [g, h, i, j] = right;
    let pair = |x, y, z, w| -> Result<f64> {
        let left = account.coordinate(|| x * y)?;
        let right = account.coordinate(|| z * w)?;
        account.coordinate(|| left + right)
    };
    Ok([
        pair(a, g, c, h)?,
        pair(b, g, d, h)?,
        pair(a, i, c, j)?,
        pair(b, i, d, j)?,
    ])
}

#[derive(Clone, Copy)]
struct CompositeLocalMap {
    linear: [f64; 4],
    translation: [f64; 2],
}

fn composite_mapping_error(invariant: &'static str) -> Error {
    Error::invalid_value("composite mapping", "resolved composite", invariant)
}

fn validate_composite_grid(
    grid: CompositeGridFacts,
    destination: bool,
    account: &Account,
) -> Result<()> {
    for value in grid.origin {
        if !account.coordinate(|| value)?.is_finite() {
            return Err(composite_mapping_error("grid origins must be finite"));
        }
    }
    let scale = account.coordinate(|| grid.scale as f32)?;
    let reciprocal = account.coordinate(|| scale.recip())?;
    if !grid.scale.is_finite()
        || grid.scale <= 0.0
        || !scale.is_normal()
        || scale <= 0.0
        || !reciprocal.is_normal()
        || reciprocal <= 0.0
    {
        return Err(composite_mapping_error(
            "grid scales and their uploaded reciprocals must be positive normal values",
        ));
    }
    for extent in grid.extent {
        if extent == 0 || extent > i32::MAX as u32 {
            return Err(composite_mapping_error(
                "grid extents must fit positive signed texture coordinates",
            ));
        }
        if destination {
            let maximum = account.coordinate(|| f64::from(extent) - 0.5)?;
            let uploaded = account.coordinate(|| maximum as f32)?;
            if f64::from(uploaded) > f64::from(i32::MAX) {
                return Err(composite_mapping_error(
                    "destination pixel centers must fit signed texture coordinates",
                ));
            }
        }
    }
    Ok(())
}

fn composite_local_map(
    affine: [f64; 6],
    destination: CompositeGridFacts,
    account: &Account,
) -> Result<CompositeLocalMap> {
    let [a, b, c, d, e, f] = affine;
    let mut linear = [0.0; 4];
    for (output, value) in linear.iter_mut().zip([a, b, c, d]) {
        *output = account.coordinate(|| value / destination.scale)?;
    }
    let mut translation = [0.0; 2];
    for (axis, (x, y, offset)) in [(a, c, e), (b, d, f)].into_iter().enumerate() {
        let x = account.coordinate(|| x * destination.origin[0])?;
        let y = account.coordinate(|| y * destination.origin[1])?;
        let sum = account.coordinate(|| x + y)?;
        translation[axis] = account.coordinate(|| sum + offset)?;
    }
    Ok(CompositeLocalMap {
        linear,
        translation,
    })
}

fn narrow_composite_map(mapping: CompositeLocalMap, account: &Account) -> Result<CompositeMap> {
    let narrow = |value: f64| -> Result<f32> {
        let uploaded = account.coordinate(|| value as f32)?;
        if !value.is_finite()
            || !(uploaded == 0.0 || uploaded.is_normal())
            || (value != 0.0 && uploaded == 0.0)
        {
            return Err(composite_mapping_error(
                "rebased mapping scalars must narrow to zero or normal finite values without nonzero underflow",
            ));
        }
        Ok(uploaded)
    };
    let mut linear = [0.0; 4];
    let mut translation = [0.0; 2];
    for (output, value) in linear.iter_mut().zip(mapping.linear) {
        *output = narrow(value)?;
    }
    for (output, value) in translation.iter_mut().zip(mapping.translation) {
        *output = narrow(value)?;
    }
    Ok(CompositeMap {
        linear,
        translation,
    })
}

fn composite_source_map(
    space: CompositeSourceSpace,
    local: Option<CompositeLocalMap>,
    source: CompositeGridFacts,
    destination: CompositeGridFacts,
    account: &Account,
) -> Result<CompositeMap> {
    let mut mapping = match space {
        CompositeSourceSpace::LayerLocal => {
            local.ok_or_else(|| composite_mapping_error("source mapping has no local transform"))?
        }
        CompositeSourceSpace::SurfaceGrid => {
            let ratio = account.coordinate(|| 1.0 / destination.scale)?;
            CompositeLocalMap {
                linear: [ratio, 0.0, 0.0, ratio],
                translation: destination.origin,
            }
        }
    };
    for value in &mut mapping.linear {
        *value = account.coordinate(|| *value * source.scale)?;
    }
    for (value, origin) in mapping.translation.iter_mut().zip(source.origin) {
        let difference = account.coordinate(|| *value - origin)?;
        *value = account.coordinate(|| difference * source.scale)?;
    }
    narrow_composite_map(mapping, account)
}

fn composite_mask_map(
    local: CompositeLocalMap,
    mask: &CompositeMaskParameterFacts,
    account: &Account,
) -> Result<CompositeMap> {
    let mut mapping = local;
    for (index, value) in mapping.linear.iter_mut().enumerate() {
        let dimension = mask.bounds[2 + index % 2];
        *value = account.coordinate(|| *value / dimension)?;
    }
    for (axis, value) in mapping.translation.iter_mut().enumerate() {
        let relative = account.coordinate(|| *value - mask.bounds[axis])?;
        *value = account.coordinate(|| relative / mask.bounds[2 + axis])?;
    }
    narrow_composite_map(mapping, account)
}

fn qualify_composite_map(mapping: CompositeMap, extent: [u32; 2], account: &Account) -> Result<()> {
    let [a, b, c, d] = mapping.linear;
    // Scaling the determinant avoids introducing overflow solely in the proof.
    let scale = account.coordinate(|| a.abs().max(b.abs()).max(c.abs()).max(d.abs()))?;
    if scale == 0.0 {
        return Err(composite_mapping_error(
            "uploaded mapping must remain nonsingular",
        ));
    }
    let a = account.coordinate(|| a / scale)?;
    let b = account.coordinate(|| b / scale)?;
    let c = account.coordinate(|| c / scale)?;
    let d = account.coordinate(|| d / scale)?;
    let ad = account.coordinate(|| a * d)?;
    let bc = account.coordinate(|| b * c)?;
    let determinant = account.coordinate(|| ad - bc)?;
    if !determinant.is_normal() {
        return Err(composite_mapping_error(
            "uploaded mapping must retain a finite nonzero normalized determinant",
        ));
    }
    let maximum_x = account.coordinate(|| f64::from(extent[0]) - 0.5)?;
    let maximum_y = account.coordinate(|| f64::from(extent[1]) - 0.5)?;
    let maximum_x = account.coordinate(|| maximum_x as f32)?;
    let maximum_y = account.coordinate(|| maximum_y as f32)?;
    // Each scalar product and partial addition in WGSL is monotone on this
    // rectangle. Its four corners bound every reached pixel-center operation.
    for x in [0.5, maximum_x] {
        for y in [0.5, maximum_y] {
            for axis in 0..2 {
                let product_x = account.coordinate(|| mapping.linear[axis] * x)?;
                let product_y = account.coordinate(|| mapping.linear[axis + 2] * y)?;
                let sum = account.coordinate(|| product_x + product_y)?;
                let result = account.coordinate(|| sum + mapping.translation[axis])?;
                if !product_x.is_finite()
                    || !product_y.is_finite()
                    || !sum.is_finite()
                    || !result.is_finite()
                {
                    return Err(composite_mapping_error(
                        "every uploaded mapping product and partial addition must remain finite over destination pixels",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn qualify_mask_sampling(
    texel_facts: [f32; 4],
    quality: ShaderMaskQualityKey,
    pixelated: Option<[u32; 2]>,
    account: &Account,
) -> Result<()> {
    if texel_facts
        .into_iter()
        .any(|value| !value.is_normal() || value <= 0.0)
    {
        return Err(composite_mapping_error(
            "mask texel-center facts must be positive normal values",
        ));
    }
    let finite = |value: f32| -> Result<f32> {
        if !value.is_finite() {
            return Err(composite_mapping_error(
                "reachable mask sampling arithmetic must remain finite",
            ));
        }
        Ok(value)
    };
    let index = |value: f32| -> Result<()> {
        let floored = account.coordinate(|| value.floor())?;
        // The cubic neighbors and reflected indexing period are signed values.
        if !floored.is_finite()
            || f64::from(floored) < f64::from(i32::MIN) + 4.0
            || f64::from(floored) > f64::from(i32::MAX) - 4.0
        {
            return Err(composite_mapping_error(
                "reachable mask sampling neighbors must fit signed texture coordinates",
            ));
        }
        Ok(())
    };
    for axis in 0..2 {
        for normalized in [0.0_f32, 1.0] {
            // The shader guard restricts normalized coordinates to [0,1].
            let relative = finite(account.coordinate(|| normalized - texel_facts[axis])?)?;
            let sample = finite(account.coordinate(|| relative / texel_facts[axis + 2])?)?;
            match quality {
                ShaderMaskQualityKey::Low => index(finite(account.coordinate(|| sample + 0.5)?)?)?,
                ShaderMaskQualityKey::Medium | ShaderMaskQualityKey::High => index(sample)?,
                ShaderMaskQualityKey::Pixelated => {
                    let factors = pixelated.ok_or_else(|| {
                        composite_mapping_error(
                            "pixelated mask sampling requires its measured lattice factors",
                        )
                    })?;
                    let factor = account.coordinate(|| factors[axis] as f32)?;
                    if !factor.is_normal() || factor <= 0.0 {
                        return Err(composite_mapping_error(
                            "pixelated mask factors must be positive normal values",
                        ));
                    }
                    let centered = finite(account.coordinate(|| sample + 0.5)?)?;
                    let scaled = finite(account.coordinate(|| centered * factor)?)?;
                    let enlarged = finite(account.coordinate(|| scaled - 0.5)?)?;
                    let base = account.coordinate(|| enlarged.floor())?;
                    let first = finite(account.coordinate(|| base / factor)?)?;
                    index(first)?;
                    let next = finite(account.coordinate(|| base + 1.0)?)?;
                    let second = finite(account.coordinate(|| next / factor)?)?;
                    index(second)?;
                }
            }
        }
    }
    Ok(())
}

impl CompositeParameterBytes {
    pub(crate) fn try_from_runtime_layer(
        parameters: &RuntimeLayerCompositeParameters,
        source: RuntimeSpatialDescriptor,
        destination: RuntimeSpatialDescriptor,
        account: &Account,
    ) -> Result<Self> {
        let mask = parameters.alpha_mask().map(|mask| {
            let bounds = mask.bounds();
            let dimensions = mask.image_dimensions();
            let texel_centers = mask.texel_center_facts();
            let [half_x, half_y] = texel_centers.half_texel_normalized();
            let [texel_x, texel_y] = texel_centers.texel_size_normalized();
            CompositeMaskParameterFacts {
                bounds: [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
                dimensions: [dimensions.width(), dimensions.height()],
                texel_center_facts: [half_x, half_y, texel_x, texel_y],
                sampling: mask.sampling(),
                value: mask.value(),
                application: mask.application(),
            }
        });
        Self::try_from_facts(
            CompositeParameterFacts {
                affine: parameters.destination_to_layer_local().affine().as_array(),
                source_space: parameters.source_space(),
                source_operation_clip: parameters
                    .filter_footprint()
                    .map(|footprint| footprint.output_clip()),
                opacity: parameters.opacity(),
                composition: parameters.composition(),
                has_clip: parameters.has_clip(),
                mask,
                source: CompositeGridFacts::from_runtime(source),
                destination: CompositeGridFacts::from_runtime(destination),
            },
            account,
        )
    }

    fn try_from_facts(facts: CompositeParameterFacts, account: &Account) -> Result<Self> {
        let CompositeParameterFacts {
            affine: destination_to_local,
            source_space,
            source_operation_clip,
            opacity,
            composition,
            has_clip,
            mask,
            source,
            destination,
        } = facts;
        validate_composite_grid(source, false, account)?;
        validate_composite_grid(destination, true, account)?;
        let source_operation_clip = source_operation_clip.map(|clip| -> Result<[f32; 4]> {
            if clip[0] < 0.0 || clip[1] < 0.0 || clip[2] < clip[0] || clip[3] < clip[1]
                || clip[2] > f64::from(source.extent[0]) || clip[3] > f64::from(source.extent[1]) {
                return Err(composite_mapping_error("Gaussian operation clip must fit its actual source grid"));
            }
            let mut narrowed = [0.0; 4];
            for (destination, value) in narrowed.iter_mut().zip(clip) {
                *destination = account.coordinate(|| value as f32)?;
                if !value.is_finite() || !((*destination == 0.0) || destination.is_normal())
                    || (value != 0.0 && *destination == 0.0) {
                    return Err(composite_mapping_error("Gaussian operation clip must retain finite normal source texel coordinates"));
                }
            }
            Ok(narrowed)
        }).transpose()?;
        let local_mapping = if source_space == CompositeSourceSpace::LayerLocal || mask.is_some() {
            Some(composite_local_map(
                destination_to_local,
                destination,
                account,
            )?)
        } else {
            None
        };
        let source_mapping =
            composite_source_map(source_space, local_mapping, source, destination, account)?;
        qualify_composite_map(source_mapping, destination.extent, account)?;

        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(Error::invalid_value(
                "composite opacity",
                opacity,
                "must be finite and clamped to the inclusive unit interval",
            ));
        }

        let mask = mask
            .map(|mask| -> Result<NarrowedCompositeMask> {
                if mask.bounds.into_iter().any(|value| !value.is_finite())
                    || mask.bounds[2] <= 0.0
                    || mask.bounds[3] <= 0.0
                {
                    return Err(composite_mapping_error(
                        "mask bounds must be finite with positive dimensions",
                    ));
                }
                let mapping = composite_mask_map(
                    local_mapping.ok_or_else(|| {
                        composite_mapping_error("mask mapping has no local transform")
                    })?,
                    &mask,
                    account,
                )?;
                qualify_composite_map(mapping, destination.extent, account)?;
                let [width, height] = mask.dimensions;
                if width == 0
                    || height == 0
                    || width > (i32::MAX / 2) as u32
                    || height > (i32::MAX / 2) as u32
                {
                    return Err(Error::invalid_value(
                        "composite mask image dimensions",
                        format!("{width}x{height}"),
                        "must be positive and retain signed sampling neighbors and reflection periods",
                    ));
                }
                let pixelated = if mask.sampling.quality() == ShaderMaskQualityKey::Pixelated {
                    Some(pixelated_mask_multipliers(
                        destination_to_local,
                        &mask,
                        destination.scale,
                        account,
                    )?)
                } else {
                    None
                };
                let [a, b, c, d] = mask.texel_center_facts;
                let texel_centers = [
                    narrow_positive_composite_scalar("composite mask texel fact", a)?,
                    narrow_positive_composite_scalar("composite mask texel fact", b)?,
                    narrow_positive_composite_scalar("composite mask texel fact", c)?,
                    narrow_positive_composite_scalar("composite mask texel fact", d)?,
                ];
                qualify_mask_sampling(texel_centers, mask.sampling.quality(), pixelated, account)?;
                Ok(NarrowedCompositeMask {
                    mapping,
                    dimensions: [width, height],
                    pixelated,
                    texel_centers,
                    sampling: mask.sampling,
                    value: mask.value,
                    application: mask.application,
                })
            })
            .transpose()?;
        account.work(RenderWork::SerializedBytes, 128)?;
        let mut bytes = [0_u8; 128];
        for (index, value) in source_mapping.linear.into_iter().enumerate() {
            write_f32(&mut bytes, index * 4, value);
        }
        for (index, value) in source_mapping.translation.into_iter().enumerate() {
            write_f32(&mut bytes, 16 + index * 4, value);
        }
        if let Some(mask) = mask {
            for (index, value) in mask.mapping.linear.into_iter().enumerate() {
                write_f32(&mut bytes, 32 + index * 4, value);
            }
            for (index, value) in mask.mapping.translation.into_iter().enumerate() {
                write_f32(&mut bytes, 24 + index * 4, value);
            }
            write_u32(&mut bytes, 48, mask.dimensions[0]);
            write_u32(&mut bytes, 52, mask.dimensions[1]);
            if let Some([x, y]) = mask.pixelated {
                write_u32(&mut bytes, 56, x);
                write_u32(&mut bytes, 60, y);
            }
            for (index, value) in mask.texel_centers.into_iter().enumerate() {
                write_f32(&mut bytes, 64 + index * 4, value);
            }
            write_u32(&mut bytes, 88, mask.sampling.quality().parameter_code());
            write_u32(&mut bytes, 92, mask.sampling.extend().parameter_code());
            use crate::{
                MaskLuminanceSpace,
                layer::{MaskApplication, MaskValue},
            };
            let mask_value_code = match mask.value {
                MaskValue::Alpha => 1,
                MaskValue::Luminance(MaskLuminanceSpace::Srgb) => 2,
                MaskValue::Luminance(MaskLuminanceSpace::LinearSrgb) => 3,
            };
            let mask_application = match mask.application {
                MaskApplication::Attenuate => 0,
                MaskApplication::CoverageSource => 4,
            };
            write_u32(&mut bytes, 100, mask_value_code | mask_application);
        }

        write_f32(&mut bytes, 80, opacity);
        write_u32(
            &mut bytes,
            84,
            blend_parameter_code(composition.blend_mode()),
        );
        write_u32(&mut bytes, 96, u32::from(has_clip));
        write_u32(
            &mut bytes,
            104,
            (match source_space {
                CompositeSourceSpace::LayerLocal => 0,
                CompositeSourceSpace::SurfaceGrid => 1,
            }) | if source_operation_clip.is_some() {
                2
            } else {
                0
            },
        );
        write_u32(
            &mut bytes,
            108,
            operator_parameter_code(composition.operator()),
        );
        if let Some(clip) = source_operation_clip {
            for (index, value) in clip.into_iter().enumerate() {
                write_f32(&mut bytes, 112 + index * 4, value);
            }
        }
        Ok(Self(bytes))
    }

    #[must_use]
    pub(crate) const fn as_bytes(&self) -> &[u8; 128] {
        &self.0
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn into_bytes_for_test(self) -> [u8; 128] {
        self.0
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) struct CompositeParameterGpuVectorFactsForTest {
    pub(crate) layer_point: [f64; 2],
    pub(crate) mask_bounds: [f64; 4],
    pub(crate) mask_dimensions: [u32; 2],
    pub(crate) quality: ImageQuality,
    pub(crate) extend: Extend,
    pub(crate) opacity: f32,
    pub(crate) composition: LayerComposition,
    pub(crate) has_clip: bool,
    pub(crate) has_mask: bool,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) fn composite_parameter_bytes_for_gpu_vector_for_test(
    facts: CompositeParameterGpuVectorFactsForTest,
) -> Result<[u8; 128]> {
    if !facts.opacity.is_finite() {
        return Err(Error::invalid_value(
            "composite opacity",
            facts.opacity,
            "must be finite before clamping",
        ));
    }
    let mask = facts.has_mask.then(|| {
        let [width, height] = facts.mask_dimensions;
        let texel_x = 1.0 / f64::from(width);
        let texel_y = 1.0 / f64::from(height);
        CompositeMaskParameterFacts {
            bounds: facts.mask_bounds,
            dimensions: facts.mask_dimensions,
            value: crate::layer::MaskValue::Alpha,
            application: crate::layer::MaskApplication::Attenuate,
            texel_center_facts: [texel_x * 0.5, texel_y * 0.5, texel_x, texel_y],
            sampling: ShaderMaskSamplingKey::with_rendering(facts.quality, None, facts.extend),
        }
    });
    CompositeParameterBytes::try_from_facts(
        CompositeParameterFacts {
            affine: [
                1.0,
                0.0,
                0.0,
                1.0,
                facts.layer_point[0] - 0.5,
                facts.layer_point[1] - 0.5,
            ],
            source_space: CompositeSourceSpace::LayerLocal,
            source_operation_clip: None,
            opacity: facts.opacity.clamp(0.0, 1.0),
            composition: facts.composition,
            has_clip: facts.has_clip,
            mask,
            source: CompositeGridFacts {
                origin: [-1.0; 2],
                scale: 1.0,
                extent: [7, 4],
            },
            destination: CompositeGridFacts::unit(),
        },
        &Account::for_test(),
    )
    .map(CompositeParameterBytes::into_bytes_for_test)
}

const fn blend_parameter_code(blend: BlendMode) -> u32 {
    match blend {
        BlendMode::Normal => 0,
        BlendMode::Multiply => 1,
        BlendMode::Screen => 2,
        BlendMode::Overlay => 3,
        BlendMode::Darken => 4,
        BlendMode::Lighten => 5,
        BlendMode::Plus => 6,
        BlendMode::ColorDodge => 7,
        BlendMode::ColorBurn => 8,
        BlendMode::HardLight => 9,
        BlendMode::SoftLight => 10,
        BlendMode::Difference => 11,
        BlendMode::Exclusion => 12,
        BlendMode::Hue => 13,
        BlendMode::Saturation => 14,
        BlendMode::Color => 15,
        BlendMode::Luminosity => 16,
    }
}

const fn operator_parameter_code(operator: Option<PorterDuffOperator>) -> u32 {
    match operator {
        None | Some(PorterDuffOperator::SourceOver) => 0,
        Some(PorterDuffOperator::Clear) => 1,
        Some(PorterDuffOperator::Copy) => 2,
        Some(PorterDuffOperator::Destination) => 3,
        Some(PorterDuffOperator::DestinationOver) => 4,
        Some(PorterDuffOperator::SourceIn) => 5,
        Some(PorterDuffOperator::DestinationIn) => 6,
        Some(PorterDuffOperator::SourceOut) => 7,
        Some(PorterDuffOperator::DestinationOut) => 8,
        Some(PorterDuffOperator::SourceAtop) => 9,
        Some(PorterDuffOperator::DestinationAtop) => 10,
        Some(PorterDuffOperator::Xor) => 11,
        Some(PorterDuffOperator::PlusLighter) => 12,
    }
}

fn write_f32(bytes: &mut [u8; 128], offset: usize, value: f32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8; 128], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn narrow_composite_scalar(field: &'static str, value: f64) -> Result<f32> {
    let narrowed = value as f32;
    if !narrowed.is_finite() {
        return Err(Error::invalid_value(
            field,
            value,
            "must remain finite after f64-to-f32 narrowing",
        ));
    }
    Ok(narrowed)
}

fn narrow_positive_composite_scalar(field: &'static str, value: f64) -> Result<f32> {
    let narrowed = narrow_composite_scalar(field, value)?;
    if narrowed <= 0.0 {
        return Err(Error::invalid_value(
            field,
            value,
            "must remain strictly positive after f64-to-f32 narrowing",
        ));
    }
    Ok(narrowed)
}

fn narrow_spatial_scalar(field: &'static str, value: f64) -> Result<f32> {
    let narrowed = value as f32;
    if !narrowed.is_finite() {
        return Err(Error::invalid_value(
            field,
            value,
            "must remain finite after f64-to-f32 narrowing",
        ));
    }
    Ok(narrowed)
}

fn narrow_raster_scale(field: &'static str, value: f64) -> Result<f32> {
    let narrowed = narrow_spatial_scalar(field, value)?;
    if narrowed <= 0.0 {
        return Err(Error::invalid_value(
            field,
            value,
            "must remain strictly positive after f64-to-f32 narrowing",
        ));
    }
    Ok(narrowed)
}

#[cfg(test)]
mod surface_grid_parameter_tests {
    use super::*;

    #[test]
    fn composite_parameters_preserve_plus_and_encode_shader_mix_modes() {
        // These are Render's shader dispatch codes, independent of Peniko's codes.
        for (blend, expected_code) in [
            (BlendMode::Normal, 0),
            (BlendMode::Multiply, 1),
            (BlendMode::Screen, 2),
            (BlendMode::Overlay, 3),
            (BlendMode::Darken, 4),
            (BlendMode::Lighten, 5),
            (BlendMode::Plus, 6),
            (BlendMode::ColorDodge, 7),
            (BlendMode::ColorBurn, 8),
            (BlendMode::HardLight, 9),
            (BlendMode::SoftLight, 10),
            (BlendMode::Difference, 11),
            (BlendMode::Exclusion, 12),
            (BlendMode::Hue, 13),
            (BlendMode::Saturation, 14),
            (BlendMode::Color, 15),
            (BlendMode::Luminosity, 16),
        ] {
            let bytes = CompositeParameterBytes::try_from_facts(
                CompositeParameterFacts {
                    affine: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    source_space: CompositeSourceSpace::LayerLocal,
                    source_operation_clip: None,
                    opacity: 1.0,
                    composition: LayerComposition::from_blend(blend),
                    has_clip: false,
                    mask: None,
                    source: CompositeGridFacts::unit(),
                    destination: CompositeGridFacts::unit(),
                },
                &Account::for_test(),
            )
            .unwrap()
            .into_bytes_for_test();
            let encoded_code = u32::from_le_bytes(bytes[84..88].try_into().unwrap());
            assert_eq!(encoded_code, expected_code, "{blend:?}");
            assert_eq!(u32::from_le_bytes(bytes[108..112].try_into().unwrap()), 0);
        }
    }

    #[test]
    fn composite_parameters_encode_basic_operators_independently_of_the_mix_code() {
        for (operator, expected_code) in [
            (PorterDuffOperator::SourceOver, 0),
            (PorterDuffOperator::Clear, 1),
            (PorterDuffOperator::Copy, 2),
            (PorterDuffOperator::Destination, 3),
            (PorterDuffOperator::DestinationOver, 4),
            (PorterDuffOperator::SourceIn, 5),
            (PorterDuffOperator::DestinationIn, 6),
            (PorterDuffOperator::SourceOut, 7),
            (PorterDuffOperator::DestinationOut, 8),
            (PorterDuffOperator::SourceAtop, 9),
            (PorterDuffOperator::DestinationAtop, 10),
            (PorterDuffOperator::Xor, 11),
        ] {
            let bytes = CompositeParameterBytes::try_from_facts(
                CompositeParameterFacts {
                    affine: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    source_space: CompositeSourceSpace::SurfaceGrid,
                    source_operation_clip: None,
                    opacity: 0.625,
                    composition: LayerComposition::try_porter_duff(operator, BlendMode::Color)
                        .unwrap(),
                    has_clip: true,
                    mask: None,
                    source: CompositeGridFacts::unit(),
                    destination: CompositeGridFacts::unit(),
                },
                &Account::for_test(),
            )
            .unwrap()
            .into_bytes_for_test();
            let word =
                |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            assert_eq!(word(84), 15, "{operator:?} keeps the Color mixing code");
            assert_eq!(word(108), expected_code, "{operator:?}");
            assert_eq!((word(96), word(100), word(104)), (1, 0, 1));
            assert_eq!(word(80), 0.625_f32.to_bits());
        }
    }

    #[test]
    fn surface_source_mapping_keeps_mask_inverse_and_independent_pixelated_axes() {
        let mask = CompositeMaskParameterFacts {
            bounds: [0.0, 0.0, 2.0, 3.0],
            value: crate::layer::MaskValue::Alpha,
            application: crate::layer::MaskApplication::Attenuate,
            dimensions: [2, 3],
            texel_center_facts: [0.25, 1.0 / 6.0, 0.5, 1.0 / 3.0],
            sampling: ShaderMaskSamplingKey::with_rendering(
                crate::ImageQuality::Medium,
                Some(crate::ImageRendering::Pixelated),
                crate::Extend::Pad,
            ),
        };
        let bytes = CompositeParameterBytes::try_from_facts(
            CompositeParameterFacts {
                affine: [0.4, 0.0, 0.0, 0.5, 0.0, 0.0],
                source_space: CompositeSourceSpace::SurfaceGrid,
                source_operation_clip: None,
                opacity: 1.0,
                composition: LayerComposition::from_blend(BlendMode::Normal),
                has_clip: false,
                mask: Some(mask),
                source: CompositeGridFacts::unit(),
                destination: CompositeGridFacts::unit(),
            },
            &Account::for_test(),
        )
        .unwrap()
        .into_bytes_for_test();
        let word =
            |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        assert_eq!(word(0), 1.0_f32.to_bits());
        assert_eq!(word(12), 1.0_f32.to_bits());
        assert_eq!(word(32), 0.2_f32.to_bits());
        assert_eq!(word(44), (1.0_f32 / 6.0).to_bits());
        assert_eq!((word(56), word(60)), (3, 2));
        assert_eq!((word(96), word(100), word(104), word(108)), (0, 1, 1, 0));
    }
}

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::accounting::{FrameFixture, RendererCpu};
    use crate::filter::RuntimeUnitFilterAmount;
    use crate::{
        GlyphRunLimits, RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
        RenderAttemptKind, RenderAttemptStatus, RenderChangeKind, RenderCpuEvent, RenderCpuScope,
        RenderEvent, RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile, RenderQuantity,
        RenderRefusal, RenderRefusalReason, UnitFilterAmount,
    };

    fn ordered_color_operations() -> [RuntimeColorOperation; 2] {
        let unit = |value| {
            RuntimeUnitFilterAmount::try_from_algorithm(UnitFilterAmount::try_new(value).unwrap())
                .unwrap()
        };
        [
            RuntimeColorOperation::for_test(RuntimeColorOperationKind::Invert(unit(0.25))),
            RuntimeColorOperation::for_test(RuntimeColorOperationKind::Opacity(unit(0.75))),
        ]
    }

    #[test]
    fn color_filter_serialization_writes_exact_header_and_ordered_records() {
        // WGSL wire words in little-endian order: count and header padding,
        // then tag, zero flag, exponent, alignment and four payload words.
        const EMPTY: [u32; 4] = [0, 0, 0, 0];
        const ONE: [u32; 12] = [1, 0, 0, 0, 4, 0, 0, 0, 0x3e80_0000, 0, 0, 0];
        const TWO: [u32; 20] = [
            2,
            0,
            0,
            0,
            4,
            0,
            0,
            0,
            0x3e80_0000,
            0,
            0,
            0,
            5,
            0,
            0,
            0,
            0x3f40_0000,
            0,
            0,
            0,
        ];
        let operations = ordered_color_operations();
        let account = Account::for_test();
        for (source, expected) in [
            (&operations[..0], EMPTY.as_slice()),
            (&operations[..1], ONE.as_slice()),
            (operations.as_slice(), TWO.as_slice()),
        ] {
            let bytes = ColorFilterOperationBytes::try_from_runtime_operations_with_limits(
                &account,
                source,
                ColorFilterOperationBufferLimits::for_test(80, 80),
            )
            .unwrap();
            let expected = expected
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>();
            assert_eq!(bytes.as_bytes(), expected, "{} records", source.len());
        }
    }

    #[test]
    fn color_filter_serialization_respects_exact_work_prefix_and_backing_custody() {
        use RenderWork::{FilterOperationVisits, SerializedBytes};

        let operations = ordered_color_operations();
        let phase = RenderPhase::GpuPrepare;
        let retained = RenderResource::RetainedCpuBytes;
        let owner_scope = RenderCpuScope::FrameBacking;
        // The header admits 16 bytes. Each operation admits one visit before
        // its 32 bytes. The refusal tuple is (current usage, requested work).
        for (work, limit, bytes, visits, refusal) in [
            (SerializedBytes, 15, 0, 0, Some((0, 16))),
            (SerializedBytes, 16, 16, 1, Some((16, 32))),
            (SerializedBytes, 48, 48, 2, Some((48, 32))),
            (SerializedBytes, 80, 80, 2, None),
            (FilterOperationVisits, 0, 16, 0, Some((0, 1))),
            (FilterOperationVisits, 1, 48, 1, Some((1, 1))),
            (FilterOperationVisits, 2, 80, 2, None),
        ] {
            let limits = RenderLimits::new().with_work_limit(work, limit).unwrap();
            let options = RenderAccountingOptions::new()
                .with_limits(limits)
                .with_profile(RenderProfile::Detailed {
                    timing: false,
                    event_limit: 64,
                    filter: RenderPhaseFilter::All,
                });
            let cpu = RendererCpu::new(options).unwrap();
            let (result, started, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let account = attempt.account().at(phase);
                    account.phase(phase, || {
                        ColorFilterOperationBytes::try_from_runtime_operations_with_limits(
                            &account,
                            &operations,
                            ColorFilterOperationBufferLimits::for_test(80, 80),
                        )
                    })
                })
                .unwrap()
                .into_parts();
            assert!(started);
            assert_eq!(fault, None);

            let attempt = report.attempt().unwrap();
            let usage = attempt.phase_usage(phase);
            assert_eq!(usage.work(SerializedBytes), bytes, "{work:?} cap {limit}");
            assert_eq!(
                usage.work(FilterOperationVisits),
                visits,
                "{work:?} cap {limit}"
            );

            let cpu_interval = report.renderer_cpu().unwrap();
            let backing = cpu_interval.scope_usage(owner_scope);
            let opening = backing.opening_live(retained);
            let changes = cpu_interval.events().filter_map(|event| match event {
                RenderCpuEvent::Change {
                    scope,
                    quantity: RenderQuantity::Resource(resource),
                    amount,
                    kind,
                    ..
                } if scope == owner_scope && resource == retained => Some((kind, amount)),
                _ => None,
            });
            let changes = changes.collect::<Vec<_>>();
            if let Some((current, amount)) = refusal {
                let refusal = RenderRefusal {
                    phase: RenderPhase::Attempt,
                    quantity: RenderQuantity::Work(work),
                    current,
                    amount,
                    reason: RenderRefusalReason::Limit { limit },
                };
                let diagnostic = Some(RenderAccountingError::Refused(refusal));
                assert_eq!(attempt.status(), RenderAttemptStatus::Refused);
                assert_eq!(attempt.refusal(), Some(refusal));
                assert_eq!(report.accounting_diagnostic(), diagnostic);
                assert_eq!(result.unwrap_err().accounting_diagnostic(), diagnostic);
                assert!(attempt.events().any(|event| matches!(
                    event,
                    RenderEvent::Refused { phase: origin, .. } if origin == phase
                )));
                let allocated = backing.peak(retained) - opening;
                assert!(allocated >= 80);
                assert_eq!(backing.live(retained), opening);
                assert_eq!(
                    changes,
                    [
                        (RenderChangeKind::Acquire, allocated),
                        (RenderChangeKind::Release, allocated),
                    ]
                );
            } else {
                assert_eq!(attempt.status(), RenderAttemptStatus::Succeeded);
                assert_eq!(attempt.refusal(), None);
                let bytes = result.unwrap();
                assert_eq!(bytes.as_bytes().len(), 80);
                let capacity = bytes.bytes.capacity() as u64;
                let owner = SharedValue::<Vec<u8>>::allocation_size_for_test() as u64;
                assert_eq!(backing.live(retained), opening + capacity + owner);
                assert_eq!(backing.peak(retained), backing.live(retained));
                assert_eq!(
                    changes,
                    [
                        (RenderChangeKind::Acquire, capacity),
                        (RenderChangeKind::Acquire, owner),
                    ]
                );
                drop(bytes);
            }

            let after = cpu.snapshot().unwrap();
            assert_eq!(
                after.scope_usage(owner_scope).unwrap().live(retained),
                opening
            );
            assert_eq!(after.fault(), None);
        }
    }

    #[test]
    fn exact_color_records_preserve_content_and_charge_only_reached_comparisons() {
        let setup = Account::for_test();
        // Literal valid invert records from this owner's documented wire
        // format: count/pads, tag4, zero flags/exponent/pads, finite amount.
        let make = |amounts: &[f32]| {
            let mut bytes = setup.retained(16 + amounts.len() * 32).unwrap();
            bytes.resize(16 + amounts.len() * 32, 0u8).unwrap();
            bytes[..4].copy_from_slice(&u32::try_from(amounts.len()).unwrap().to_le_bytes());
            for (index, amount) in amounts.iter().enumerate() {
                let offset = 16 + index * 32;
                bytes[offset..offset + 4].copy_from_slice(&4u32.to_le_bytes());
                bytes[offset + 16..offset + 20].copy_from_slice(&amount.to_le_bytes());
            }
            ColorFilterOperationBytes {
                bytes: SharedValue::from_accounted(bytes, &setup.storage()).unwrap(),
            }
        };
        let left = make(&[0.25, 0.5, 0.75]);
        let equal = make(&[0.25, 0.5, 0.75]);
        let clone = left.clone();
        let changed = make(&[0.25, 0.125, 0.75]);
        let shorter = make(&[0.25, 0.5]);
        assert_eq!(left, equal);
        assert!(!left.bytes.same_allocation(&equal.bytes));
        for (right, visits, expected) in [
            (&equal, 3u64, true),
            (&clone, 0, true),
            (&changed, 2, false),
            (&shorter, 0, false),
        ] {
            for limit in (visits.saturating_sub(1)..=visits).rev() {
                let options = RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::FilterOperationVisits, limit)
                        .unwrap(),
                );
                let mut context = RenderAccountingContext::new(options);
                let result = context.run(
                    RenderAttemptKind::Frame,
                    crate::RenderPhase::Encode,
                    |account| left.matches_in(right, account),
                );
                let report = context.last_attempt().unwrap().attempt().unwrap();
                assert_eq!(
                    report.usage().work(RenderWork::FilterOperationVisits),
                    limit
                );
                assert_eq!(report.usage().work(RenderWork::SerializedBytes), 0);
                if limit == visits {
                    assert_eq!(result.unwrap(), expected);
                } else {
                    let Some(crate::RenderAccountingError::Refused(refusal)) =
                        result.unwrap_err().accounting_diagnostic()
                    else {
                        panic!("the next operation record must refuse");
                    };
                    assert_eq!(
                        refusal.quantity(),
                        crate::RenderQuantity::Work(RenderWork::FilterOperationVisits)
                    );
                    assert_eq!((refusal.current(), refusal.amount()), (limit, 1));
                }
            }
        }
    }

    #[test]
    fn fixed_parameter_serialization_admits_the_exact_block_after_validation() {
        for (limit, succeeds) in [(31, false), (32, true)] {
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::SerializedBytes, limit)
                .unwrap();
            let mut context = RenderAccountingContext::new(
                RenderAccountingOptions::default().with_limits(limits),
            );
            let result = context.prepare(
                RenderAttemptKind::GlyphRun,
                GlyphRunLimits::new(0, 0, 0, 0),
                #[cfg(feature = "render-text")]
                None,
                |account| {
                    DropShadowParameterBytes::try_new(
                        Point::new(0.0, 0.0),
                        Color::TRANSPARENT,
                        Some(account),
                    )
                },
            );
            assert_eq!(result.is_ok(), succeeds);
            if let Ok(bytes) = result {
                assert_eq!(bytes.as_bytes(), &[0; 32]);
            }
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                report.usage().work(RenderWork::SerializedBytes),
                if succeeds { 32 } else { 0 }
            );
            assert_eq!(
                report.status(),
                if succeeds {
                    RenderAttemptStatus::Succeeded
                } else {
                    RenderAttemptStatus::Refused
                }
            );
        }
    }

    #[test]
    fn invalid_composite_mask_is_rejected_before_any_output_byte_is_admitted() {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::SerializedBytes, 0)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::default().with_limits(limits));
        let result = context.prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(0, 0, 0, 0),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                CompositeParameterBytes::try_from_facts(
                    CompositeParameterFacts {
                        affine: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                        source_space: CompositeSourceSpace::LayerLocal,
                        source_operation_clip: None,
                        opacity: 1.0,
                        composition: LayerComposition::from_blend(BlendMode::Normal),
                        has_clip: false,
                        mask: Some(CompositeMaskParameterFacts {
                            bounds: [0.0, 0.0, 1.0, 1.0],
                            value: crate::layer::MaskValue::Alpha,
                            application: crate::layer::MaskApplication::Attenuate,
                            dimensions: [0, 1],
                            texel_center_facts: [0.5, 0.5, 1.0, 1.0],
                            sampling: ShaderMaskSamplingKey::with_rendering(
                                crate::image::ImageQuality::Medium,
                                None,
                                crate::image::Extend::Pad,
                            ),
                        }),
                        source: CompositeGridFacts::unit(),
                        destination: CompositeGridFacts::unit(),
                    },
                    account,
                )
            },
        );
        let error = result.unwrap_err();
        assert!(error.accounting_diagnostic().is_none());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::SerializedBytes),
            0
        );
    }
}

#[cfg(test)]
mod convolution_parameter_tests {
    use super::*;

    fn facts() -> ConvolutionParameterFacts {
        ConvolutionParameterFacts {
            capture_origin: [-2.0, 3.0],
            capture_scale: 2.0,
            kernel_origin: [-2.0, 3.0],
            unit_length: [0.5, 1.5],
            source_grid_origin: [-3, 4],
            source_grid_extent: [7, 8],
            result_grid_origin: [-5, 2],
            result_grid_extent: [11, 12],
            source_domain: Rect::new(-1.0, 4.0, 3.0, 5.0),
            result_domain: Rect::new(-2.0, 3.0, 8.0, 9.0),
            result_capture_origin: [-2.0, 3.0],
            result_capture_scale: 2.0,
            order: [7, 5],
            target: [3, 2],
            stage: 1,
            first_tap: 32,
            tap_count: 3,
            edge: 2,
            preserve_alpha: true,
            linear_srgb: true,
            first_batch: false,
            bias: -0.25,
        }
    }

    #[test]
    fn convolution_wire_retains_signed_origins_runtime_flags_and_zero_reserved_words() {
        let bytes = ConvolutionParameterBytes::try_new(facts(), &Account::for_test()).unwrap();
        let words: Vec<u32> = bytes
            .as_bytes()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|word| u32::from_le_bytes(*word))
            .collect();
        assert_eq!(words.len(), 64);
        assert_eq!(&words[8..12], &[(-3_i32) as u32, 4, 7, 8]);
        assert_eq!(&words[12..16], &[(-5_i32) as u32, 2, 11, 12]);
        assert_eq!(&words[28..32], &[7, 5, 3, 2]);
        assert_eq!(&words[32..36], &[1, 32, 3, 2]);
        assert_eq!(&words[36..40], &[1, 1, 0, 0]);
        assert_eq!(words[40], (-0.25_f32).to_bits());
        assert!(words[41..].iter().all(|word| *word == 0));
    }

    #[test]
    fn convolution_wire_refuses_nonfinite_and_underflowed_coordinates_and_unbounded_batches() {
        let account = Account::for_test();
        let mut input = facts();
        input.capture_origin[0] = f64::INFINITY;
        assert!(ConvolutionParameterBytes::try_new(input, &account).is_err());
        let mut input = facts();
        input.unit_length[0] = f64::MIN_POSITIVE;
        assert!(ConvolutionParameterBytes::try_new(input, &account).is_err());
        let mut input = facts();
        input.tap_count = 33;
        assert!(ConvolutionParameterBytes::try_new(input, &account).is_err());
    }
}

#[cfg(test)]
mod offset_parameter_tests {
    use super::*;

    fn facts() -> OffsetParameterFacts {
        OffsetParameterFacts {
            translation: [-0.5, 1.25],
            ratio: [1.0, 0.5],
            source_clip: [0.25, 0.5, 6.75, 7.5],
            result_clip: [0.0, 0.25, 10.5, 11.75],
            source_extent: [7, 8],
            result_extent: [11, 12],
            sampling: 3,
            rebase: false,
        }
    }

    #[test]
    fn offset_wire_preserves_rebased_map_clips_modes_and_zero_reserved_words() {
        let bytes = OffsetParameterBytes::try_new(facts(), &Account::for_test()).unwrap();
        let words: Vec<_> = bytes
            .as_bytes()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|word| u32::from_le_bytes(*word))
            .collect();
        let floats = |values: &[f32]| values.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
        assert_eq!(&words[0..4], floats(&[-0.5, 1.25, 1.0, 0.5]));
        assert_eq!(&words[4..8], floats(&[0.25, 0.5, 6.75, 7.5]));
        assert_eq!(&words[8..12], floats(&[0.0, 0.25, 10.5, 11.75]));
        assert_eq!(&words[12..16], &[7, 8, 11, 12]);
        assert_eq!(&words[16..20], &[3, 0, 0, 0]);
        assert!(words[20..].iter().all(|value| *value == 0));
    }

    #[test]
    fn offset_wire_accepts_each_sampling_policy_and_only_integral_raw_rebases() {
        for sampling in 0..=3 {
            let mut input = facts();
            input.sampling = sampling;
            input.translation = [-1.0, 2.0];
            input.ratio = [1.0; 2];
            assert!(OffsetParameterBytes::try_new(input, &Account::for_test()).is_ok());
        }
        let mut input = facts();
        input.sampling = 0;
        input.translation = [-1.0, 2.0];
        input.ratio = [1.0; 2];
        input.rebase = true;
        let bytes = OffsetParameterBytes::try_new(input, &Account::for_test()).unwrap();
        assert_eq!(&bytes.as_bytes()[64..72], &[0, 0, 0, 0, 1, 0, 0, 0]);
        for (translation, ratio, sampling) in [
            ([0.5, 1.0], [1.0; 2], 0),
            ([0.0; 2], [0.5, 1.0], 0),
            ([0.0; 2], [1.0; 2], 1),
        ] {
            let mut input = facts();
            input.translation = translation;
            input.ratio = ratio;
            input.sampling = sampling;
            input.rebase = true;
            assert!(OffsetParameterBytes::try_new(input, &Account::for_test()).is_err());
        }
    }

    #[test]
    fn offset_wire_refuses_invalid_narrowing_and_domains_before_serialization() {
        for invalid in 0..9 {
            let mut input = facts();
            match invalid {
                0 => input.translation[0] = f64::INFINITY,
                1 => input.translation[0] = f64::MIN_POSITIVE,
                2 => input.translation[0] = f64::from(f32::MIN_POSITIVE) / 2.0,
                3 => input.ratio[1] = 0.0,
                4 => input.source_clip[2] = 8.0,
                5 => input.result_clip[0] = -0.25,
                6 => input.result_clip[3] = input.result_clip[1] - 0.125,
                7 => input.source_extent[0] = 8_388_609,
                _ => input.sampling = 4,
            }
            let mut context = crate::RenderAccountingContext::default();
            let result = context.run(
                crate::RenderAttemptKind::Frame,
                crate::RenderPhase::GpuPrepare,
                |account| OffsetParameterBytes::try_new(input, account),
            );
            assert!(result.is_err(), "case {invalid}");
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::SerializedBytes),
                0,
                "case {invalid}"
            );
        }
    }

    #[test]
    fn offset_wire_admits_exactly_one_fixed_block_with_no_coefficient_work() {
        for (limit, succeeds) in [(255, false), (256, true)] {
            let limits = crate::RenderLimits::new()
                .with_work_limit(RenderWork::SerializedBytes, limit)
                .unwrap();
            let mut context = crate::RenderAccountingContext::new(
                crate::RenderAccountingOptions::default().with_limits(limits),
            );
            let result = context.run(
                crate::RenderAttemptKind::Frame,
                crate::RenderPhase::GpuPrepare,
                |account| OffsetParameterBytes::try_new(facts(), account),
            );
            assert_eq!(result.is_ok(), succeeds);
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                report.usage().work(RenderWork::SerializedBytes),
                if succeeds { 256 } else { 0 }
            );
            assert_eq!(report.usage().work(RenderWork::KernelSampleVisits), 0);
        }
    }
}

#[cfg(test)]
mod rebased_composite_parameter_tests {
    use super::*;

    fn facts(origin: f64) -> CompositeParameterFacts {
        CompositeParameterFacts {
            affine: [1.0, 0.0, 0.0, 1.0, origin, 0.0],
            source_space: CompositeSourceSpace::LayerLocal,
            source_operation_clip: None,
            opacity: 1.0,
            composition: LayerComposition::from_blend(BlendMode::Normal),
            has_clip: false,
            mask: Some(CompositeMaskParameterFacts {
                bounds: [origin, 0.0, 2.0, 1.0],
                dimensions: [2, 1],
                texel_center_facts: [0.25, 0.5, 0.5, 1.0],
                sampling: ShaderMaskSamplingKey::with_rendering(
                    crate::ImageQuality::Medium,
                    None,
                    crate::Extend::Reflect,
                ),
                value: crate::layer::MaskValue::Alpha,
                application: crate::layer::MaskApplication::Attenuate,
            }),
            source: CompositeGridFacts {
                origin: [origin, 0.0],
                scale: 1.0,
                extent: [2, 1],
            },
            destination: CompositeGridFacts {
                origin: [0.0; 2],
                scale: 1.0,
                extent: [2, 1],
            },
        }
    }

    #[test]
    fn composite_wire_rebases_both_source_and_mask_before_large_origins_narrow() {
        for origin in [0.0, 100_000_000.0, -100_000_000.0, 1.0e100] {
            let bytes =
                CompositeParameterBytes::try_from_facts(facts(origin), &Account::for_test())
                    .unwrap();
            let words = bytes
                .as_bytes()
                .as_chunks::<4>()
                .0
                .iter()
                .map(|word| u32::from_le_bytes(*word))
                .collect::<Vec<_>>();
            let expected = [
                1.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 1.0,
            ]
            .map(f32::to_bits);
            assert_eq!(&words[..12], expected, "origin {origin}");
            assert_eq!(&words[12..16], &[2, 1, 0, 0]);
            assert_eq!(&words[16..20], [0.25_f32, 0.5, 0.5, 1.0].map(f32::to_bits));
        }
    }

    #[test]
    fn composite_wire_composes_destination_origin_scale_and_source_grid_independently() {
        let mut input = facts(100_000_000.0);
        input.affine = [2.0, 0.0, 0.0, 3.0, 100_000_000.0, 0.0];
        input.source.scale = 4.0;
        input.destination.origin = [1.5, 2.0];
        input.destination.scale = 2.0;
        let bytes = CompositeParameterBytes::try_from_facts(input, &Account::for_test()).unwrap();
        let float = |offset: usize| {
            f32::from_le_bytes(bytes.as_bytes()[offset..offset + 4].try_into().unwrap())
        };
        // Source: ((2*x+origin,3*y)-source_origin)*4 at destination O+p/2.
        assert_eq!(
            [float(0), float(12), float(16), float(20)],
            [4.0, 6.0, 12.0, 24.0]
        );
        // Mask normalization uses width2 and height1 rather than source scale4.
        assert_eq!(
            [float(32), float(44), float(24), float(28)],
            [0.5, 1.5, 1.5, 6.0]
        );
    }

    #[test]
    fn composite_wire_refuses_reachable_overflow_singular_narrowing_and_invalid_grid_before_bytes()
    {
        for invalid in 0..7 {
            let mut input = facts(0.0);
            match invalid {
                0 => input.source.scale = f64::from(f32::MIN_POSITIVE) / 2.0,
                1 => input.destination.scale = f64::from(f32::MAX),
                2 => {
                    input.affine = [
                        f64::from(f32::MAX) / 4.0,
                        0.0,
                        -f64::from(f32::MAX) / 4.0,
                        f64::from(f32::MAX) / 8.0,
                        0.0,
                        0.0,
                    ];
                    input.destination.extent = [16; 2];
                }
                3 => input.affine = [1.0, 1.0, 1.0, 1.0 + 1.0e-10, 0.0, 0.0],
                4 => input.mask.as_mut().unwrap().dimensions[0] = (i32::MAX / 2) as u32 + 1,
                5 => {
                    input.mask.as_mut().unwrap().texel_center_facts[2] =
                        f64::from(f32::MIN_POSITIVE)
                }
                _ => input.destination.extent[0] = i32::MAX as u32,
            }
            let mut context = crate::RenderAccountingContext::default();
            let result = context.run(
                crate::RenderAttemptKind::Frame,
                crate::RenderPhase::GpuPrepare,
                |account| CompositeParameterBytes::try_from_facts(input, account),
            );
            assert!(result.is_err(), "case {invalid}");
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::SerializedBytes),
                0,
                "case {invalid}"
            );
        }
    }
}

#[cfg(test)]
mod gaussian_parameter_tests {
    use super::*;

    fn facts() -> GaussianParameterFacts {
        GaussianParameterFacts {
            stage: 4,
            source_size: [3, 2],
            target_size: [8, 4],
            source_base: [-7, 9],
            source_clip: [0.0, 0.0, 3.0, 2.0],
            output_clip: [0.25, 0.5, 7.75, 3.5],
            periodic_base: [2.5, 1.25],
            periodic_step: [1.2, 1.6],
            edge_mode: 1,
            periodic: true,
            direct: false,
            axis: 1,
            first_tap: 32,
            tap_count: 17,
            first: false,
            linear: true,
            converted: true,
            color: [0.0; 4],
        }
    }

    #[test]
    fn gaussian_wire_preserves_declared_rebased_maps_and_zero_reserved_words() {
        let bytes = GaussianParameterBytes::try_new(facts(), &Account::for_test()).unwrap();
        let word =
            |offset: usize| u32::from_le_bytes(bytes.0[offset..offset + 4].try_into().unwrap());
        assert_eq!(word(0), 3);
        assert_eq!(word(8), 8);
        assert_eq!(i32::from_le_bytes(bytes.0[16..20].try_into().unwrap()), -7);
        assert_eq!(f32::from_bits(word(48)), 0.25);
        assert_eq!(f32::from_bits(word(64)), 2.5);
        assert_eq!(f32::from_bits(word(72)), 1.2);
        assert_eq!([word(80), word(84), word(88), word(92)], [4, 32, 17, 1]);
        assert_eq!([word(96), word(100), word(104), word(108)], [1, 1, 0, 0]);
        assert_eq!([word(112), word(116)], [1, 1]);
        assert!(bytes.0[120..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn gaussian_wire_refuses_reached_coordinate_narrowing_and_oversized_batches() {
        for bad in [f64::INFINITY, f64::NAN, f64::MAX, f64::MIN_POSITIVE] {
            let mut input = facts();
            input.periodic_step[0] = bad;
            assert!(GaussianParameterBytes::try_new(input, &Account::for_test()).is_err());
        }
        let mut input = facts();
        input.stage = 3;
        input.tap_count = 33;
        assert!(GaussianParameterBytes::try_new(input, &Account::for_test()).is_err());
        input.tap_count = 1;
        input.first_tap = u32::MAX;
        assert!(GaussianParameterBytes::try_new(input, &Account::for_test()).is_err());
        input.first_tap = 0;
        input.target_size[0] = 8_388_609;
        assert!(GaussianParameterBytes::try_new(input, &Account::for_test()).is_err());
    }

    #[test]
    fn gaussian_constant_and_raw_copy_skip_unreachable_coordinate_payloads() {
        let mut input = facts();
        input.stage = 5;
        input.source_size = [u32::MAX; 2];
        input.source_clip = [f64::NAN; 4];
        input.periodic_step = [f64::NAN; 2];
        input.color = [0.25, 0.125, 0.0, 0.5];
        let bytes = GaussianParameterBytes::try_new(input, &Account::for_test()).unwrap();
        assert_eq!(&bytes.0[128..132], &0.25_f32.to_le_bytes());
        input.stage = 0;
        input.source_size = [3, 2];
        input.output_clip = [f64::NAN; 4];
        assert!(GaussianParameterBytes::try_new(input, &Account::for_test()).is_ok());
    }
}

#[cfg(test)]
mod gaussian_composite_parameter_tests {
    use super::*;

    fn facts(
        clip: Option<[f64; 4]>,
        source_space: CompositeSourceSpace,
    ) -> CompositeParameterFacts {
        CompositeParameterFacts {
            affine: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            source_space,
            source_operation_clip: clip,
            opacity: 1.0,
            composition: LayerComposition::from_blend(BlendMode::Normal),
            has_clip: false,
            mask: None,
            source: CompositeGridFacts {
                origin: [0.0; 2],
                scale: 1.0,
                extent: [3, 2],
            },
            destination: CompositeGridFacts {
                origin: [0.0; 2],
                scale: 1.0,
                extent: [3, 2],
            },
        }
    }

    #[test]
    fn gaussian_operation_wire_encodes_rebased_fractional_clip_without_replacing_source_space() {
        for (space, low_bit) in [
            (CompositeSourceSpace::LayerLocal, 0),
            (CompositeSourceSpace::SurfaceGrid, 1),
        ] {
            let bytes = CompositeParameterBytes::try_from_facts(
                facts(Some([0.25, 0.5, 2.75, 1.5]), space),
                &Account::for_test(),
            )
            .unwrap();
            assert_eq!(
                u32::from_le_bytes(bytes.0[104..108].try_into().unwrap()),
                low_bit | 2
            );
            for (actual, expected) in bytes.0[112..128]
                .as_chunks::<4>()
                .0
                .iter()
                .zip([0.25_f32, 0.5, 2.75, 1.5])
            {
                assert_eq!(f32::from_le_bytes(*actual), expected);
            }
            let ordinary =
                CompositeParameterBytes::try_from_facts(facts(None, space), &Account::for_test())
                    .unwrap();
            assert_eq!(
                u32::from_le_bytes(ordinary.0[104..108].try_into().unwrap()),
                low_bit
            );
            assert!(ordinary.0[112..].iter().all(|byte| *byte == 0));
        }
    }

    #[test]
    fn gaussian_operation_wire_rejects_clips_outside_actual_source_or_unrepresentable_narrowing() {
        for clip in [
            [-0.25, 0.0, 2.0, 1.0],
            [0.0, 0.0, 3.25, 1.0],
            [0.0, 0.0, 2.0, 2.25],
            [0.0, 0.0, f64::NAN, 1.0],
            [f64::MIN_POSITIVE, 0.0, 2.0, 1.0],
        ] {
            assert!(
                CompositeParameterBytes::try_from_facts(
                    facts(Some(clip), CompositeSourceSpace::LayerLocal),
                    &Account::for_test()
                )
                .is_err()
            );
        }
    }
}
