use super::{Error, Result};
use crate::accounting::{
    Account, AccountedVec, RenderPhase, RenderResource, RenderWork, SharedValue,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum GaussianKernelSamplingForm {
    PairedLinear,
    FullNearest,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct GaussianKernelKey {
    standard_deviation_bits: u64,
    raster_scale_bits: u64,
    support_multiple_bits: u64,
    support_radius: u32,
    sampling_form: GaussianKernelSamplingForm,
}

impl GaussianKernelKey {
    pub(crate) const fn from_exact_plan(
        standard_deviation_bits: u64,
        raster_scale_bits: u64,
        support_multiple_bits: u64,
        support_radius: u32,
        sampling_form: GaussianKernelSamplingForm,
    ) -> Self {
        Self {
            standard_deviation_bits,
            raster_scale_bits,
            support_multiple_bits,
            support_radius,
            sampling_form,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GaussianKernelPlan {
    key: GaussianKernelKey,
    upload_bytes: SharedValue<Vec<u8>>,
    byte_len: u64,
}

/// Allocation-free geometry for the native exact-offset sampling route.
/// The coefficient owner retains the key and every derived size together so
/// consumers cannot substitute an independently calculated radius or count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GaussianKernelGeometry {
    key: GaussianKernelKey,
    device_standard_deviation: f64,
    sample_count: u32,
    byte_len: u64,
    weight_bytes: usize,
    serialized_bytes: usize,
}

impl GaussianKernelGeometry {
    pub(crate) fn try_new(
        standard_deviation: f64,
        raster_scale: f64,
        support_multiple: f64,
        sampling_form: GaussianKernelSamplingForm,
        account: &Account,
    ) -> Result<Self> {
        let account = account.at(RenderPhase::Kernel);
        account.phase(RenderPhase::Kernel, || {
            for (value, allow_zero) in [
                (standard_deviation, true),
                (raster_scale, false),
                (support_multiple, false),
            ] {
                account.work(RenderWork::CoordinateVisits, 1)?;
                if !value.is_finite() || value < 0.0 || (!allow_zero && value == 0.0) {
                    return Err(native_geometry_error(value));
                }
            }
            let device_standard_deviation =
                account.coordinate(|| standard_deviation * raster_scale)?;
            if !device_standard_deviation.is_finite()
                || (standard_deviation > 0.0 && device_standard_deviation == 0.0)
            {
                return Err(native_geometry_error(device_standard_deviation));
            }
            let support = account.coordinate(|| device_standard_deviation * support_multiple)?;
            if !support.is_finite() || support > f64::from(u32::MAX) {
                return Err(native_geometry_error(support));
            }
            // Every integer through 2^24 has an exact f32 representation. Merely
            // checking the last offset's round trip would admit larger even radii
            // whose intervening integer tap offsets are not representable.
            let support_radius = account.coordinate(|| support.ceil() as u32)?;
            if support_radius > 1 << 24 {
                return Err(native_geometry_error(f64::from(support_radius)));
            }
            let sample_count = gaussian_sample_count(support_radius, sampling_form)?;
            let weight_count = usize::try_from(support_radius)
                .ok()
                .and_then(|radius| radius.checked_add(1))
                .ok_or_else(|| native_geometry_error(f64::from(support_radius)))?;
            let weight_bytes = weight_count
                .checked_mul(size_of::<f64>())
                .filter(|&bytes| bytes <= isize::MAX as usize)
                .ok_or_else(|| native_geometry_error(f64::from(support_radius)))?;
            let byte_len = u64::from(sample_count) * 8;
            let serialized_bytes = usize::try_from(byte_len)
                .ok()
                .filter(|&bytes| bytes <= isize::MAX as usize)
                .ok_or_else(|| native_geometry_error(f64::from(sample_count)))?;
            Ok(Self {
                key: GaussianKernelKey::from_exact_plan(
                    standard_deviation.to_bits(),
                    raster_scale.to_bits(),
                    support_multiple.to_bits(),
                    support_radius,
                    sampling_form,
                ),
                device_standard_deviation,
                sample_count,
                byte_len,
                weight_bytes,
                serialized_bytes,
            })
        })
    }

    pub(crate) const fn key(self) -> GaussianKernelKey {
        self.key
    }
    pub(crate) const fn support_radius(self) -> u32 {
        self.key.support_radius
    }
    pub(crate) const fn sample_count(self) -> u32 {
        self.sample_count
    }
    pub(crate) const fn byte_len(self) -> u64 {
        self.byte_len
    }
}

fn native_geometry_error(value: f64) -> Error {
    crate::GaussianPreparationError::Numeric {
        phase: crate::GaussianNumericPhase::KernelExtent,
        value,
    }
    .into()
}

impl PartialEq for GaussianKernelPlan {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.upload_bytes() == other.upload_bytes()
            && self.byte_len == other.byte_len
    }
}
impl Eq for GaussianKernelPlan {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GaussianKernelBufferLimits {
    max_buffer_size: u64,
    max_storage_buffer_binding_size: u64,
}

impl GaussianKernelBufferLimits {
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

impl GaussianKernelPlan {
    pub(crate) fn try_new_with_limits(
        geometry: GaussianKernelGeometry,
        limits: GaussianKernelBufferLimits,
        account: &Account,
    ) -> Result<Self> {
        account.phase(RenderPhase::Kernel, || {
            let account = account.at(RenderPhase::Kernel);
            validate_buffer_byte_len(geometry.byte_len(), limits)?;
            // Check forward budgets without billing coefficient calculations or
            // serialization that have not happened. Actual loops below remain
            // the sole work producers and actual allocations own their holds.
            let weight_visits = 3 * u64::from(geometry.support_radius()) + 2;
            account.preview_work([
                (
                    RenderWork::KernelSampleVisits,
                    weight_visits + u64::from(geometry.sample_count()),
                ),
                (RenderWork::SerializedBytes, geometry.byte_len()),
            ])?;
            account.preview(RenderResource::ScratchCpuBytes, geometry.weight_bytes)?;
            let retained_bytes = geometry
                .serialized_bytes
                .checked_add(SharedValue::<Vec<u8>>::allocation_size())
                .ok_or_else(|| native_geometry_error(f64::from(geometry.sample_count)))?;
            account.preview(RenderResource::RetainedCpuBytes, retained_bytes)?;
            let positive_weights = NormalizedGaussianWeights::try_new(
                geometry.device_standard_deviation,
                geometry.support_radius(),
                &account,
            )?;
            let upload_bytes = positive_weights.serialize(geometry.key.sampling_form, &account)?;
            Ok(Self {
                key: geometry.key,
                upload_bytes: SharedValue::from_accounted(upload_bytes, &account.storage())?,
                byte_len: geometry.byte_len(),
            })
        })
    }

    pub(crate) fn matches_in(&self, other: &Self, account: &Account) -> Result<bool> {
        if self.key != other.key
            || self.byte_len != other.byte_len
            || self.upload_bytes.len() != other.upload_bytes.len()
        {
            return Ok(false);
        }
        if self.upload_bytes.same_allocation(&other.upload_bytes) {
            return Ok(true);
        }
        // The factory emits only complete 8-byte offset/weight samples. Keep
        // exact byte distinctions and bill each comparison independently from
        // the earlier weight calculation and sample serialization.
        for (actual, expected) in self
            .upload_bytes
            .as_chunks::<8>()
            .0
            .iter()
            .zip(other.upload_bytes.as_chunks::<8>().0)
        {
            account.work(RenderWork::KernelSampleVisits, 1)?;
            if actual != expected {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn try_new(
        standard_deviation: f64,
        raster_scale: f64,
        support_multiple: f64,
        sampling_form: GaussianKernelSamplingForm,
        account: &Account,
    ) -> Result<Self> {
        account.phase(RenderPhase::Kernel, || {
            let account = account.at(RenderPhase::Kernel);
            let (device_standard_deviation, support_radius) =
                validate_gaussian_geometry(standard_deviation, raster_scale, support_multiple)?;
            let positive_weights = NormalizedGaussianWeights::try_new(
                device_standard_deviation,
                support_radius,
                &account,
            )?;
            let upload_bytes = positive_weights.serialize(sampling_form, &account)?;
            let byte_len = u64::try_from(upload_bytes.len()).map_err(|_| {
                Error::invalid_value(
                    "Gaussian kernel byte length",
                    upload_bytes.len(),
                    "must fit in u64",
                )
            })?;
            Ok(Self {
                key: GaussianKernelKey::from_exact_plan(
                    standard_deviation.to_bits(),
                    raster_scale.to_bits(),
                    support_multiple.to_bits(),
                    support_radius,
                    sampling_form,
                ),
                upload_bytes: SharedValue::from_accounted(upload_bytes, &account.storage())?,
                byte_len,
            })
        })
    }

    pub(crate) const fn key(&self) -> GaussianKernelKey {
        self.key
    }

    pub(crate) fn upload_bytes(&self) -> &[u8] {
        &self.upload_bytes
    }

    pub(crate) const fn byte_len(&self) -> u64 {
        self.byte_len
    }

    pub(crate) fn validate_upload_byte_len(&self, actual_len: usize) -> Result<()> {
        if actual_len != self.upload_bytes.len() {
            return Err(Error::invalid_value(
                "Gaussian kernel upload byte length",
                actual_len,
                "must equal the exact serialized kernel plan length",
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_buffer_limits(&self, limits: GaussianKernelBufferLimits) -> Result<()> {
        validate_buffer_byte_len(self.byte_len, limits)
    }
}

fn validate_buffer_byte_len(byte_len: u64, limits: GaussianKernelBufferLimits) -> Result<()> {
    if byte_len == 0
        || byte_len > limits.max_buffer_size
        || byte_len > limits.max_storage_buffer_binding_size
    {
        return Err(Error::invalid_value(
            "Gaussian kernel buffer byte length",
            byte_len,
            "must be positive and no greater than the selected device buffer and storage-binding limits",
        ));
    }
    Ok(())
}

fn validate_gaussian_geometry(
    standard_deviation: f64,
    raster_scale: f64,
    support_multiple: f64,
) -> Result<(f64, u32)> {
    if !standard_deviation.is_finite() || standard_deviation < 0.0 {
        return Err(Error::invalid_value(
            "Gaussian standard deviation",
            standard_deviation,
            "must be finite and non-negative",
        ));
    }
    for (field, value) in [
        ("Gaussian raster scale", raster_scale),
        ("Gaussian support multiple", support_multiple),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(Error::invalid_value(
                field,
                value,
                "must be finite and greater than zero",
            ));
        }
    }
    let device_standard_deviation = standard_deviation * raster_scale;
    if !device_standard_deviation.is_finite() {
        return Err(Error::invalid_value(
            "Gaussian device standard deviation",
            device_standard_deviation,
            "must be finite",
        ));
    }
    let support = device_standard_deviation * support_multiple;
    if !support.is_finite() || support > f64::from(u32::MAX) {
        return Err(Error::invalid_value(
            "Gaussian support radius",
            support,
            "must be finite and fit in u32 device pixels",
        ));
    }
    Ok((device_standard_deviation, support.ceil() as u32))
}

// Construction retains the actual center-plus-positive allocation and its
// proved radius. Finished sampling never accepts a separate slice or count.
struct NormalizedGaussianWeights {
    support_radius: u32,
    positive_weights: AccountedVec<f64>,
}

impl NormalizedGaussianWeights {
    fn try_new(
        device_standard_deviation: f64,
        support_radius: u32,
        account: &Account,
    ) -> Result<Self> {
        let weight_count = usize::try_from(support_radius)
            .ok()
            .and_then(|radius| radius.checked_add(1))
            .ok_or_else(|| {
                Error::invalid_value(
                    "Gaussian kernel weight count",
                    support_radius,
                    "must fit addressable memory",
                )
            })?;
        let mut positive_weights = account.temporary(weight_count)?;
        for offset in 0..=support_radius {
            account.work(RenderWork::KernelSampleVisits, 1)?;
            // Zero device sigma has only the center sample.
            let weight = if offset == 0 {
                1.0
            } else {
                let ratio = f64::from(offset) / device_standard_deviation;
                (-0.5 * ratio * ratio).exp()
            };
            positive_weights.append_slot()?.write(weight);
        }
        let mut positive_sum = 0.0;
        for weight in positive_weights.iter().skip(1) {
            account.work(RenderWork::KernelSampleVisits, 1)?;
            positive_sum += weight;
        }
        let normalization = positive_weights[0] + 2.0 * positive_sum;
        if !normalization.is_finite() || normalization <= 0.0 {
            return Err(Error::invalid_value(
                "Gaussian kernel normalization",
                normalization,
                "must be finite and greater than zero",
            ));
        }
        for weight in positive_weights.iter_mut() {
            account.work(RenderWork::KernelSampleVisits, 1)?;
            *weight /= normalization;
        }
        Ok(Self {
            support_radius,
            positive_weights,
        })
    }

    fn sample_count(&self, sampling_form: GaussianKernelSamplingForm) -> Result<u32> {
        gaussian_sample_count(self.support_radius, sampling_form)
    }

    fn serialize(
        &self,
        sampling_form: GaussianKernelSamplingForm,
        account: &Account,
    ) -> Result<AccountedVec<u8>> {
        let sample_count = self.sample_count(sampling_form)?;
        let byte_capacity = usize::try_from(sample_count)
            .ok()
            .and_then(|count| count.checked_mul(8))
            .ok_or_else(|| {
                Error::invalid_value(
                    "Gaussian kernel byte length",
                    sample_count,
                    "must fit addressable memory",
                )
            })?;
        let mut upload_bytes = account.retained(byte_capacity)?;
        append_kernel_sample(&mut upload_bytes, 0.0, self.positive_weights[0], account)?;
        match sampling_form {
            GaussianKernelSamplingForm::FullNearest => {
                self.append_full(&mut upload_bytes, account)?
            }
            GaussianKernelSamplingForm::PairedLinear => {
                self.append_paired(&mut upload_bytes, account)?
            }
        }
        if upload_bytes.len() != byte_capacity {
            return Err(Error::invalid_value(
                "Gaussian kernel byte length",
                upload_bytes.len(),
                "must match the exact serialized sample count",
            ));
        }
        Ok(upload_bytes)
    }

    fn append_full(&self, upload_bytes: &mut AccountedVec<u8>, account: &Account) -> Result<()> {
        for (offset, &weight) in (1..=self.support_radius).zip(&self.positive_weights[1..]) {
            append_kernel_sample(upload_bytes, f64::from(offset), weight, account)?;
            append_kernel_sample(upload_bytes, -f64::from(offset), weight, account)?;
        }
        Ok(())
    }

    fn append_paired(&self, upload_bytes: &mut AccountedVec<u8>, account: &Account) -> Result<()> {
        let (pairs, tail) = self.positive_weights[1..].as_chunks::<2>();
        for (first, weights) in (1..=self.support_radius).step_by(2).zip(pairs) {
            let (offset, weight) = paired_gaussian_sample(first, weights)?;
            append_kernel_sample(upload_bytes, offset, weight, account)?;
            append_kernel_sample(upload_bytes, -offset, weight, account)?;
        }
        if let [weight] = tail {
            let offset = f64::from(self.support_radius);
            append_kernel_sample(upload_bytes, offset, *weight, account)?;
            append_kernel_sample(upload_bytes, -offset, *weight, account)?;
        }
        Ok(())
    }
}

fn gaussian_sample_count(
    support_radius: u32,
    sampling_form: GaussianKernelSamplingForm,
) -> Result<u32> {
    match sampling_form {
        GaussianKernelSamplingForm::FullNearest => support_radius
            .checked_mul(2)
            .and_then(|count| count.checked_add(1)),
        GaussianKernelSamplingForm::PairedLinear => support_radius
            .checked_add(1)
            .and_then(|count| count.checked_div(2))
            .and_then(|pairs| pairs.checked_mul(2))
            .and_then(|paired| paired.checked_add(1)),
    }
    .ok_or_else(|| {
        Error::invalid_value(
            "Gaussian kernel sample count",
            support_radius,
            "must fit in u32",
        )
    })
}

fn paired_gaussian_sample(
    first: u32,
    &[first_weight, second_weight]: &[f64; 2],
) -> Result<(f64, f64)> {
    let weight = first_weight + second_weight;
    if weight <= 0.0 {
        return Err(Error::invalid_value(
            "Gaussian paired sample weight",
            weight,
            "must remain greater than zero",
        ));
    }
    let offset =
        (f64::from(first) * first_weight + (f64::from(first) + 1.0) * second_weight) / weight;
    Ok((offset, weight))
}

fn append_kernel_sample(
    bytes: &mut AccountedVec<u8>,
    offset: f64,
    weight: f64,
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::KernelSampleVisits, 1)?;
    let offset = offset as f32;
    let weight = weight as f32;
    if !offset.is_finite() || !weight.is_finite() {
        return Err(Error::invalid_value(
            "Gaussian kernel sample",
            format!("offset {offset}, weight {weight}"),
            "must narrow to finite f32 values",
        ));
    }
    account.work(RenderWork::SerializedBytes, 8)?;
    bytes.extend_from_slice(&offset.to_le_bytes())?;
    bytes.extend_from_slice(&weight.to_le_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{FrameFixture, RenderAccountingOptions, RenderLimits, RendererCpu};

    fn native_plan(
        sigma: f64,
        scale: f64,
        support: f64,
        form: GaussianKernelSamplingForm,
        limits: GaussianKernelBufferLimits,
        account: &Account,
    ) -> Result<GaussianKernelPlan> {
        let geometry = GaussianKernelGeometry::try_new(sigma, scale, support, form, account)?;
        GaussianKernelPlan::try_new_with_limits(geometry, limits, account)
    }

    #[test]
    fn native_four_sigma_samples_match_independently_derived_rational_weights() {
        // For s=1/sqrt(2 ln 2), g(k)=2^(-k²); halving s yields
        // g(k)=2^(-4k²). These rational expectations do not evaluate exp or
        // normalize production coefficients to obtain their oracle.
        let sigma = 1.0 / (2.0 * std::f64::consts::LN_2).sqrt();
        for (scale, expected) in [
            (
                1.0,
                &[
                    32768.0 / 69761.0,
                    16384.0 / 69761.0,
                    2048.0 / 69761.0,
                    64.0 / 69761.0,
                    1.0 / 139522.0,
                ][..],
            ),
            (
                0.5,
                &[32768.0 / 36865.0, 2048.0 / 36865.0, 1.0 / 73730.0][..],
            ),
        ] {
            let account = Account::for_test();
            let plan = native_plan(
                sigma,
                scale,
                4.0,
                GaussianKernelSamplingForm::FullNearest,
                GaussianKernelBufferLimits::for_test(72, 72),
                &account,
            )
            .unwrap();
            let records = plan.upload_bytes().as_chunks::<8>().0;
            assert_eq!(records.len(), 2 * expected.len() - 1);
            for (index, record) in records.iter().enumerate() {
                let [a, b, c, d, w, x, y, z] = *record;
                let offset = f32::from_le_bytes([a, b, c, d]);
                let weight = f32::from_le_bytes([w, x, y, z]);
                let positive_index = index.div_ceil(2);
                let sign = if index == 0 || index % 2 == 1 {
                    1.0
                } else {
                    -1.0
                };
                assert_eq!(offset, sign * positive_index as f32);
                assert!((f64::from(weight) - expected[positive_index]).abs() < 3.0e-8);
            }
        }
    }

    #[test]
    fn native_exact_bytes_admit_and_either_one_under_device_limit_refuses_before_weights() {
        for (buffer, binding, succeeds) in [(72, 72, true), (71, 72, false), (72, 71, false)] {
            let options = RenderAccountingOptions::new();
            let cpu = RendererCpu::new(options).unwrap();
            let (result, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let plan = native_plan(
                        1.0,
                        1.0,
                        4.0,
                        GaussianKernelSamplingForm::FullNearest,
                        GaussianKernelBufferLimits::for_test(buffer, binding),
                        attempt.account(),
                    )?;
                    assert_eq!(plan.byte_len(), 72);
                    Ok(())
                })
                .unwrap()
                .into_parts();
            assert_eq!(result.is_ok(), succeeds);
            assert_eq!(fault, None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::CoordinateVisits), 6);
            assert_eq!(
                usage.work(RenderWork::KernelSampleVisits),
                if succeeds { 23 } else { 0 }
            );
            assert_eq!(
                usage.work(RenderWork::SerializedBytes),
                if succeeds { 72 } else { 0 }
            );
            assert_eq!(usage.work(RenderWork::ResourceAllocationRequests), 0);
            let cpu_usage = report.renderer_cpu().unwrap().usage();
            if !succeeds {
                assert_eq!(
                    cpu_usage.peak(RenderResource::ScratchCpuBytes),
                    cpu_usage.opening_live(RenderResource::ScratchCpuBytes)
                );
            }
            for resource in [
                RenderResource::RetainedCpuBytes,
                RenderResource::ScratchCpuBytes,
            ] {
                assert_eq!(cpu_usage.live(resource), cpu_usage.opening_live(resource));
            }
        }
    }

    #[test]
    fn native_known_work_refusal_performs_no_weight_visits_or_serialization() {
        for (work, limit) in [
            (RenderWork::KernelSampleVisits, 22),
            (RenderWork::SerializedBytes, 71),
        ] {
            let options = RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_work_limit(work, limit).unwrap());
            let cpu = RendererCpu::new(options).unwrap();
            let (result, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    native_plan(
                        1.0,
                        1.0,
                        4.0,
                        GaussianKernelSamplingForm::FullNearest,
                        GaussianKernelBufferLimits::for_test(72, 72),
                        attempt.account(),
                    )
                    .map(drop)
                })
                .unwrap()
                .into_parts();
            let error = result.unwrap_err();
            let Some(crate::RenderAccountingError::Refused(refusal)) =
                error.accounting_diagnostic()
            else {
                panic!("expected central work-budget refusal");
            };
            assert_eq!(refusal.quantity(), crate::RenderQuantity::Work(work));
            assert_eq!(refusal.current(), 0);
            assert_eq!(fault, None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::KernelSampleVisits), 0);
            assert_eq!(usage.work(RenderWork::SerializedBytes), 0);
            let usage = report.renderer_cpu().unwrap().usage();
            assert_eq!(
                usage.peak(RenderResource::ScratchCpuBytes),
                usage.opening_live(RenderResource::ScratchCpuBytes)
            );
        }
    }

    #[test]
    fn native_geometry_proves_all_integer_offsets_and_composed_density_before_allocation() {
        let account = Account::for_test();
        let exact = GaussianKernelGeometry::try_new(
            f64::from(1 << 22),
            1.0,
            4.0,
            GaussianKernelSamplingForm::FullNearest,
            &account,
        )
        .unwrap();
        assert_eq!(exact.support_radius(), 1 << 24);
        assert_eq!(exact.sample_count(), (1 << 25) + 1);
        assert_eq!(exact.byte_len(), u64::from((1_u32 << 25) + 1) * 8);
        for (sigma, scale) in [
            (f64::from((1 << 22) + 1), 1.0),
            (f64::MAX, 2.0),
            (1.0, f64::INFINITY),
            (f64::from_bits(1), 0.5),
        ] {
            let error = GaussianKernelGeometry::try_new(
                sigma,
                scale,
                4.0,
                GaussianKernelSamplingForm::FullNearest,
                &account,
            )
            .unwrap_err();
            assert!(matches!(
                error.gaussian_diagnostic(),
                Some(crate::GaussianPreparationError::Numeric {
                    phase: crate::GaussianNumericPhase::KernelExtent,
                    ..
                })
            ));
        }
        assert_eq!(
            account.work_count(RenderWork::KernelSampleVisits).unwrap(),
            0
        );
        assert_eq!(account.work_count(RenderWork::SerializedBytes).unwrap(), 0);
        let half = GaussianKernelGeometry::try_new(
            1.0,
            0.5,
            4.0,
            GaussianKernelSamplingForm::FullNearest,
            &account,
        )
        .unwrap();
        let full = GaussianKernelGeometry::try_new(
            1.0,
            1.0,
            4.0,
            GaussianKernelSamplingForm::FullNearest,
            &account,
        )
        .unwrap();
        assert_ne!(half.key(), full.key());
    }

    #[test]
    fn native_shared_plan_releases_only_after_the_last_upload_owner() {
        let account = Account::for_test();
        let plan = native_plan(
            1.0,
            1.0,
            4.0,
            GaussianKernelSamplingForm::FullNearest,
            GaussianKernelBufferLimits::for_test(72, 72),
            &account,
        )
        .unwrap();
        let alive = plan.upload_bytes.allocation_liveness_for_test();
        let clone = plan.clone();
        drop(plan);
        assert!(alive());
        assert_eq!(clone.upload_bytes().len(), 72);
        drop(clone);
        assert!(!alive());
    }

    // Independent 70-digit decimal exp(-x²/2), normalized by
    // 1 + 2Σ exp(-x²/2), rounded to serialized f32 values.
    fn literal_samples(radius: u32, form: GaussianKernelSamplingForm) -> Vec<(f32, f32)> {
        let (center, positives): (f32, &[(f32, f32)]) = match (radius, form) {
            (0, _) => (1.0, &[]),
            (1, _) => (0.45186275, &[(1.0, 0.27406862)]),
            (2, GaussianKernelSamplingForm::FullNearest) => {
                (0.40261996, &[(1.0, 0.24420135), (2.0, 0.054488685)])
            }
            (2, GaussianKernelSamplingForm::PairedLinear) => {
                (0.40261996, &[(1.1824255, 0.29869002)])
            }
            (3, GaussianKernelSamplingForm::FullNearest) => (
                0.39905027,
                &[(1.0, 0.24203622), (2.0, 0.054005582), (3.0, 0.004433048)],
            ),
            (3, GaussianKernelSamplingForm::PairedLinear) => {
                (0.39905027, &[(1.1824255, 0.29604182), (3.0, 0.004433048)])
            }
            _ => panic!("fixture covers radii zero through three"),
        };
        let mut samples = vec![(0.0, center)];
        for &(offset, weight) in positives {
            samples.extend([(offset, weight), (-offset, weight)]);
        }
        samples
    }

    #[test]
    fn zero_single_even_and_odd_support_preserve_literal_bytes_keys_and_work_limits() {
        use crate::accounting::RenderResource;
        for radius in 0..=3_u32 {
            for form in [
                GaussianKernelSamplingForm::FullNearest,
                GaussianKernelSamplingForm::PairedLinear,
            ] {
                // A positive valid sigma whose support product rounds to zero.
                let (sigma, support) = if radius == 0 {
                    (f64::from_bits(1), 0.25)
                } else {
                    (1.0, f64::from(radius))
                };
                let samples = literal_samples(radius, form);
                let expected_bytes: Vec<_> = samples
                    .iter()
                    .flat_map(|(offset, weight)| {
                        offset.to_le_bytes().into_iter().chain(weight.to_le_bytes())
                    })
                    .collect();
                // Generation r+1, summation r, normalization r+1, serialization s.
                let visits = 3 * u64::from(radius) + 2 + samples.len() as u64;
                let serialized = 8 * samples.len() as u64;
                for (work, exact) in [
                    (RenderWork::KernelSampleVisits, visits),
                    (RenderWork::SerializedBytes, serialized),
                ] {
                    let cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
                    // Retry the identical input on this renderer after refusal,
                    // changing only the next attempt's allowance to the exact limit.
                    for limit in [exact - 1, exact] {
                        let options = RenderAccountingOptions::new()
                            .with_limits(RenderLimits::new().with_work_limit(work, limit).unwrap());
                        let (_, _, report, fault) = FrameFixture::new(&cpu, options)
                            .run_sync(|attempt| {
                                let result = GaussianKernelPlan::try_new(
                                    sigma,
                                    1.0,
                                    support,
                                    form,
                                    attempt.account(),
                                );
                                match result {
                                    Ok(plan) => {
                                        assert_eq!(limit, exact);
                                        assert_eq!(plan.upload_bytes(), expected_bytes);
                                        assert_eq!(plan.byte_len(), serialized);
                                        assert_eq!(
                                            plan.key(),
                                            GaussianKernelKey::from_exact_plan(
                                                sigma.to_bits(),
                                                1.0_f64.to_bits(),
                                                support.to_bits(),
                                                radius,
                                                form
                                            )
                                        );
                                        let clone = plan.clone();
                                        assert!(
                                            plan.upload_bytes.same_allocation(&clone.upload_bytes)
                                        );
                                        drop(clone);
                                        drop(plan);
                                        Ok(())
                                    }
                                    Err(error) => {
                                        assert_eq!(limit, exact - 1);
                                        let diagnostic = error.accounting_diagnostic();
                                        let Some(crate::RenderAccountingError::Refused(refusal)) =
                                            diagnostic
                                        else {
                                            panic!("last sample must refuse");
                                        };
                                        assert_eq!(
                                            refusal.quantity(),
                                            crate::RenderQuantity::Work(work)
                                        );
                                        let amount = if work == RenderWork::SerializedBytes {
                                            8
                                        } else {
                                            1
                                        };
                                        assert_eq!(
                                            (refusal.current(), refusal.amount()),
                                            (exact - amount, amount)
                                        );
                                        Err(error)
                                    }
                                }
                            })
                            .unwrap()
                            .into_parts();
                        assert_eq!(fault, None);
                        let usage = report.attempt().unwrap().usage();
                        assert_eq!(
                            usage.work(RenderWork::KernelSampleVisits),
                            if work == RenderWork::KernelSampleVisits {
                                limit
                            } else {
                                visits
                            }
                        );
                        assert_eq!(
                            usage.work(RenderWork::SerializedBytes),
                            if limit == exact {
                                serialized
                            } else {
                                serialized - 8
                            }
                        );
                        assert_eq!(usage.work(RenderWork::UploadBytes), 0);
                        let usage = report.renderer_cpu().unwrap().usage();
                        for resource in [
                            RenderResource::RetainedCpuBytes,
                            RenderResource::ScratchCpuBytes,
                        ] {
                            assert_eq!(usage.live(resource), usage.opening_live(resource));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn underflowed_pair_refuses_after_normalization_and_center_serialization() {
        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let (_, _, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| {
                // ceil(0.01 * 300) = 3; exp(-5000) and later weights underflow.
                let error = GaussianKernelPlan::try_new(
                    0.01,
                    1.0,
                    300.0,
                    GaussianKernelSamplingForm::PairedLinear,
                    attempt.account(),
                )
                .unwrap_err();
                assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
                assert!(error.to_string().contains("Gaussian paired sample weight"));

                Err::<(), _>(error)
            })
            .unwrap()
            .into_parts();
        assert_eq!(fault, None);
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::KernelSampleVisits), 12);
        assert_eq!(usage.work(RenderWork::SerializedBytes), 8);
        let usage = report.renderer_cpu().unwrap().usage();
        for resource in [
            crate::accounting::RenderResource::RetainedCpuBytes,
            crate::accounting::RenderResource::ScratchCpuBytes,
        ] {
            assert_eq!(usage.live(resource), usage.opening_live(resource));
        }
    }

    #[test]
    fn exact_kernel_samples_preserve_conflicts_and_charge_only_reached_comparisons() {
        let setup = Account::for_test();
        let make = || {
            GaussianKernelPlan::try_new(
                1.0,
                1.0,
                1.0,
                GaussianKernelSamplingForm::FullNearest,
                &setup,
            )
            .unwrap()
        };
        let left = make();
        let equal = make();
        let clone = left.clone();
        assert_eq!(left.byte_len(), 24);
        assert_eq!(left, equal);
        assert!(!left.upload_bytes.same_allocation(&equal.upload_bytes));
        // Preserve the exact conflicting-byte rejection independently of key
        // equality. This private fixture changes only the second sample.
        let mut changed = make();
        let mut bytes = setup.retained(24).unwrap();
        bytes.extend_from_slice(changed.upload_bytes()).unwrap();
        bytes[8] ^= 1;
        changed.upload_bytes = SharedValue::from_accounted(bytes, &setup.storage()).unwrap();
        assert_ne!(left, changed);
        let different_key = GaussianKernelPlan::try_new(
            0.25,
            1.0,
            3.0,
            GaussianKernelSamplingForm::PairedLinear,
            &setup,
        )
        .unwrap();
        for (right, visits, expected) in [
            (&equal, 3u64, true),
            (&clone, 0, true),
            (&changed, 2, false),
            (&different_key, 0, false),
        ] {
            for limit in (visits.saturating_sub(1)..=visits).rev() {
                let options = RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::KernelSampleVisits, limit)
                        .unwrap(),
                );
                let mut context = crate::RenderAccountingContext::new(options);
                let result = context.run(
                    crate::RenderAttemptKind::Frame,
                    crate::RenderPhase::Kernel,
                    |account| left.matches_in(right, account),
                );
                let report = context.last_attempt().unwrap().attempt().unwrap();
                assert_eq!(report.usage().work(RenderWork::KernelSampleVisits), limit);
                assert_eq!(report.usage().work(RenderWork::SerializedBytes), 0);
                if limit == visits {
                    assert_eq!(result.unwrap(), expected);
                } else {
                    let Some(crate::RenderAccountingError::Refused(refusal)) =
                        result.unwrap_err().accounting_diagnostic()
                    else {
                        panic!("the next sample record must refuse");
                    };
                    assert_eq!(
                        refusal.quantity(),
                        crate::RenderQuantity::Work(RenderWork::KernelSampleVisits)
                    );
                    assert_eq!((refusal.current(), refusal.amount()), (limit, 1));
                }
            }
        }
    }

    #[test]
    fn zero_device_sigma_serializes_one_accounted_identity_sample() {
        for (sigma, scale) in [(0.0, 1.0), (f64::from_bits(1), 0.5)] {
            let options = RenderAccountingOptions::new();
            let cpu = RendererCpu::new(options).unwrap();
            let (result, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let plan = GaussianKernelPlan::try_new(
                        sigma,
                        scale,
                        2.5,
                        GaussianKernelSamplingForm::PairedLinear,
                        attempt.account(),
                    )?;
                    assert_eq!(plan.upload_bytes(), &[0, 0, 0, 0, 0, 0, 128, 63]);
                    Ok(())
                })
                .unwrap()
                .into_parts();
            result.unwrap();
            assert_eq!(fault, None);
            let attempt = report.attempt().unwrap();
            assert_eq!(attempt.status(), crate::RenderAttemptStatus::Succeeded);
            assert_eq!(attempt.usage().work(RenderWork::KernelSampleVisits), 3);
            assert_eq!(attempt.usage().work(RenderWork::SerializedBytes), 8);
        }
    }

    #[test]
    fn nearest_kernel_serializes_three_literal_samples_and_counts_actual_loops() {
        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let (_, _, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let plan = GaussianKernelPlan::try_new(
                    1.0,
                    1.0,
                    1.0,
                    GaussianKernelSamplingForm::FullNearest,
                    attempt.account(),
                )
                .unwrap();
                assert_eq!(plan.byte_len(), 24);
                let samples = plan.upload_bytes().as_chunks::<8>().0;
                let expected = [
                    (0.0_f32, 0.45186275_f32),
                    (1.0, 0.27406862),
                    (-1.0, 0.27406862),
                ];
                assert_eq!(samples.len(), expected.len());
                for (sample, (offset, weight)) in samples.iter().zip(expected) {
                    assert_eq!(&sample[..4], &offset.to_le_bytes());
                    assert_eq!(&sample[4..], &weight.to_le_bytes());
                }
                let clone = plan.clone();
                assert!(std::ptr::eq(
                    plan.upload_bytes().as_ptr(),
                    clone.upload_bytes().as_ptr()
                ));
                drop(clone);
                drop(plan);

                Ok(())
            })
            .unwrap()
            .into_parts();
        assert_eq!(fault, None);
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::KernelSampleVisits), 8);
        assert_eq!(usage.work(RenderWork::SerializedBytes), 24);
        assert_eq!(usage.work(RenderWork::UploadBytes), 0);
    }

    #[test]
    fn serialized_limit_refuses_before_copy_then_a_fresh_attempt_can_retry() {
        let options = RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::SerializedBytes, 24)
                .unwrap(),
        );
        let cpu = RendererCpu::new(options).unwrap();
        let (result, _, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let error = GaussianKernelPlan::try_new(
                    1.0,
                    1.0,
                    3.0,
                    GaussianKernelSamplingForm::PairedLinear,
                    attempt.account(),
                )
                .unwrap_err();

                Err::<(), _>(error)
            })
            .unwrap()
            .into_parts();
        assert!(matches!(
            result.unwrap_err().accounting_diagnostic(),
            Some(crate::RenderAccountingError::Refused(_))
        ));
        assert_eq!(report.status(), crate::RenderAttemptStatus::Refused);
        assert_eq!(
            report.attempt().unwrap().status(),
            crate::RenderAttemptStatus::Refused
        );
        assert_eq!(fault, None);
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::SerializedBytes),
            24
        );
        let refusal = report.attempt().unwrap().refusal().unwrap();
        assert_eq!(refusal.current, 24);
        assert_eq!(refusal.amount, 8);

        let (result, _, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|retry| {
                let plan = GaussianKernelPlan::try_new(
                    1.0,
                    1.0,
                    1.0,
                    GaussianKernelSamplingForm::PairedLinear,
                    retry.account(),
                )
                .unwrap();
                assert_eq!(plan.byte_len(), 24);
                drop(plan);

                Ok(())
            })
            .unwrap()
            .into_parts();
        result.unwrap();
        assert_eq!(report.status(), crate::RenderAttemptStatus::Succeeded);
        assert_eq!(
            report.attempt().unwrap().status(),
            crate::RenderAttemptStatus::Succeeded
        );
        assert_eq!(fault, None);
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::SerializedBytes),
            24
        );
    }

    #[test]
    fn untimed_profile_detail_does_not_change_kernel_work_or_cpu_peaks() {
        use crate::accounting::{RenderClock, RenderPhaseFilter, RenderProfile, RenderResource};
        use std::sync::atomic::{AtomicUsize, Ordering};
        static READS: AtomicUsize = AtomicUsize::new(0);
        fn clock() -> std::time::Duration {
            std::time::Duration::from_nanos(READS.fetch_add(1, Ordering::Relaxed) as u64)
        }
        READS.store(0, Ordering::Relaxed);
        let profiles = [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 0,
                filter: RenderPhaseFilter::All,
            },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 2,
                filter: RenderPhaseFilter::Phase(RenderPhase::Kernel),
            },
        ];
        let mut expected = None;
        for profile in profiles {
            let options = RenderAccountingOptions::new()
                .with_profile(profile)
                .with_clock(RenderClock::Host(clock));
            let cpu = RendererCpu::new(options).unwrap();
            let (_, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let plan = GaussianKernelPlan::try_new(
                        1.0,
                        1.0,
                        1.0,
                        GaussianKernelSamplingForm::FullNearest,
                        attempt.account(),
                    )
                    .unwrap();
                    assert_eq!(plan.byte_len(), 24);
                    drop(plan);

                    Ok(())
                })
                .unwrap()
                .into_parts();
            assert_eq!(fault, None);
            let attempt = report.attempt().unwrap();
            let cpu = report.renderer_cpu().unwrap();
            let usage = (
                attempt.usage().work(RenderWork::KernelSampleVisits),
                attempt.usage().work(RenderWork::SerializedBytes),
                cpu.usage().peak(RenderResource::RetainedCpuBytes),
                cpu.usage().peak(RenderResource::ScratchCpuBytes),
            );
            assert_eq!(attempt.elapsed(), None);
            assert_eq!(cpu.elapsed(), None);
            assert_eq!(
                cpu.usage().live(RenderResource::RetainedCpuBytes),
                cpu.usage().opening_live(RenderResource::RetainedCpuBytes)
            );
            assert_eq!(
                cpu.usage().live(RenderResource::ScratchCpuBytes),
                cpu.usage().opening_live(RenderResource::ScratchCpuBytes)
            );
            match expected {
                Some(expected) => assert_eq!(usage, expected),
                None => expected = Some(usage),
            }
        }
        assert_eq!(READS.load(Ordering::Relaxed), 0);
    }
}
