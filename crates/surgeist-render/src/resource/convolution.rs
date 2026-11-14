use crate::{
    Error, Result,
    accounting::{Account, CpuAccount, RenderCpuScope, RenderWork, SharedValue},
};
use std::hash::{Hash, Hasher};

/// Exact admitted coefficient bytes, shared with the immutable upload plan.
/// The fingerprint only rejects candidates; equality always proves actual bytes.
#[derive(Clone, Debug)]
pub(crate) struct ConvolutionWeightsKey {
    fingerprint: u64,
    logical_tap_count: usize,
    bytes: SharedValue<Vec<u8>>,
    cpu: CpuAccount,
}

impl PartialEq for ConvolutionWeightsKey {
    fn eq(&self, other: &Self) -> bool {
        self.logical_tap_count == other.logical_tap_count
            && self.fingerprint == other.fingerprint
            && self.bytes == other.bytes
    }
}
impl Eq for ConvolutionWeightsKey {}
impl Hash for ConvolutionWeightsKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.logical_tap_count.hash(state);
        self.fingerprint.hash(state);
        self.bytes.len().hash(state);
    }
}

impl ConvolutionWeightsKey {
    pub(super) fn matches_in(&self, other: &Self, observer: Option<&Account>) -> Result<bool> {
        if self.logical_tap_count != other.logical_tap_count
            || self.fingerprint != other.fingerprint
            || self.bytes.len() != other.bytes.len()
        {
            return Ok(false);
        }
        // Zero logical taps always serialize the constructor-proved four zero
        // bytes required for an unused binding. Comparing that binding is no
        // filter-tap visit, even when its allocations differ.
        if self.logical_tap_count == 0 || self.bytes.same_allocation(&other.bytes) {
            return Ok(true);
        }
        match observer {
            Some(account) => {
                self.compare_bytes(other, || account.work(RenderWork::KernelSampleVisits, 1))
            }
            None => {
                let cpu = self.cpu.at(RenderCpuScope::Cleanup);
                let mut result = Ok(false);
                cpu.maintenance_phase(|| {
                    result = self.compare_bytes(other, || {
                        cpu.maintenance(RenderWork::KernelSampleVisits, 1);
                        Ok(())
                    });
                });
                result
            }
        }
    }

    fn compare_bytes(&self, other: &Self, mut visit: impl FnMut() -> Result<()>) -> Result<bool> {
        for (left, right) in self
            .bytes
            .as_chunks::<4>()
            .0
            .iter()
            .zip(other.bytes.as_chunks::<4>().0)
        {
            visit()?;
            if left != right {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Renderer-owned serialization of already normalized, zero-or-normal f32 weights.
/// No public preparation backing is retained without its separate renderer import.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConvolutionWeightsPlan {
    key: ConvolutionWeightsKey,
    byte_len: u64,
}

impl ConvolutionWeightsPlan {
    pub(crate) fn try_new(
        weights: &[f32],
        account: &Account,
        limits: &wgpu::Limits,
    ) -> Result<Self> {
        // This execution plan's retained serialization belongs to the renderer,
        // including when an idle cache entry outlives the frame attempt.
        let cpu = account.cpu()?.clone();
        let byte_capacity = weights.len().max(1).checked_mul(4).ok_or_else(|| {
            Error::invalid_value(
                "convolution weights byte length",
                weights.len(),
                "must fit addressable memory",
            )
        })?;
        let byte_len = u64::try_from(byte_capacity).map_err(|_| {
            Error::invalid_value(
                "convolution weights byte length",
                byte_capacity,
                "must fit in u64",
            )
        })?;
        validate_limits(byte_len, limits)?;
        let mut bytes = account.retained(byte_capacity)?;
        let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
        let zero = [0.0_f32];
        let logical_tap_count = weights.len();
        let empty = weights.is_empty();
        let weights = if empty { &zero } else { weights };
        for &weight in weights {
            if !empty {
                account.work(RenderWork::KernelSampleVisits, 1)?;
            }
            if weight != 0.0 && !weight.is_normal() {
                return Err(Error::invalid_value(
                    "convolution normalized weight",
                    weight,
                    "must be zero or a finite normal f32",
                ));
            }
            let encoded = weight.to_le_bytes();
            account.work(RenderWork::SerializedBytes, encoded.len())?;
            for byte in encoded {
                fingerprint = (fingerprint ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
            }
            bytes.extend_from_slice(&encoded)?;
        }
        Ok(Self {
            key: ConvolutionWeightsKey {
                fingerprint,
                logical_tap_count,
                bytes: SharedValue::from_accounted(bytes, &account.storage())?,
                cpu,
            },
            byte_len,
        })
    }

    pub(crate) fn key(&self) -> ConvolutionWeightsKey {
        self.key.clone()
    }
    pub(crate) const fn byte_len(&self) -> u64 {
        self.byte_len
    }
    pub(crate) fn upload_bytes(&self) -> &[u8] {
        &self.key.bytes
    }
    #[cfg(test)]
    pub(crate) fn matches_in(&self, other: &Self, account: &Account) -> Result<bool> {
        self.key.matches_in(&other.key, Some(account))
    }
    pub(crate) fn validate_upload_byte_len(&self, actual: usize) -> Result<()> {
        if actual != self.key.bytes.len() {
            return Err(Error::invalid_value(
                "convolution weights upload byte length",
                actual,
                "must equal the admitted serialized byte length",
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_buffer_limits(&self, limits: &wgpu::Limits) -> Result<()> {
        validate_limits(self.byte_len, limits)
    }
}

fn validate_limits(byte_len: u64, limits: &wgpu::Limits) -> Result<()> {
    if byte_len < 4
        || byte_len > limits.max_buffer_size
        || byte_len > limits.max_storage_buffer_binding_size
    {
        return Err(Error::invalid_value(
            "convolution weights buffer byte length",
            byte_len,
            "must contain at least one f32 and fit the device buffer and storage-binding limits",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weights_preserve_exact_signed_f32_bytes_and_empty_binding_is_one_zero() {
        let account = Account::for_test();
        let limits = wgpu::Limits::default();
        let plan = ConvolutionWeightsPlan::try_new(&[1.0, -2.5, -0.0], &account, &limits).unwrap();
        assert_eq!(
            plan.upload_bytes(),
            &[0, 0, 128, 63, 0, 0, 32, 192, 0, 0, 0, 128]
        );
        assert_eq!(plan.byte_len(), 12);
        let empty = ConvolutionWeightsPlan::try_new(&[], &account, &limits).unwrap();
        assert_eq!(empty.upload_bytes(), &[0, 0, 0, 0]);
        assert_eq!(empty.byte_len(), 4);
    }
    #[test]
    fn serialization_bills_real_coefficients_and_dummy_bytes_independently() {
        let account = Account::for_test();
        let limits = wgpu::Limits::default();
        let dummy = ConvolutionWeightsPlan::try_new(&[], &account, &limits).unwrap();
        assert_eq!(
            account.work_count(RenderWork::KernelSampleVisits).unwrap(),
            0
        );
        assert_eq!(account.work_count(RenderWork::SerializedBytes).unwrap(), 4);
        let real = ConvolutionWeightsPlan::try_new(&[1.0, -2.0, 0.0], &account, &limits).unwrap();
        assert_eq!(
            account.work_count(RenderWork::KernelSampleVisits).unwrap(),
            3
        );
        assert_eq!(account.work_count(RenderWork::SerializedBytes).unwrap(), 16);
        drop((dummy, real));
    }

    #[test]
    fn weights_refuse_non_normal_values_and_each_device_buffer_limit() {
        let account = Account::for_test();
        let mut limits = wgpu::Limits::default();
        for value in [f32::NAN, f32::INFINITY, f32::from_bits(1)] {
            assert!(ConvolutionWeightsPlan::try_new(&[value], &account, &limits).is_err());
        }
        limits.max_buffer_size = 4;
        assert!(ConvolutionWeightsPlan::try_new(&[1.0, 2.0], &account, &limits).is_err());
        limits.max_buffer_size = 8;
        limits.max_storage_buffer_binding_size = 4;
        assert!(ConvolutionWeightsPlan::try_new(&[1.0, 2.0], &account, &limits).is_err());
        limits.max_storage_buffer_binding_size = 8;
        assert!(ConvolutionWeightsPlan::try_new(&[1.0, 2.0], &account, &limits).is_ok());
    }
    #[test]
    fn dummy_bindings_compare_without_filter_tap_visits_and_differ_from_real_zero() {
        let preparation = Account::for_test();
        let limits = wgpu::Limits::default();
        let left = ConvolutionWeightsPlan::try_new(&[], &preparation, &limits).unwrap();
        let right = ConvolutionWeightsPlan::try_new(&[], &preparation, &limits).unwrap();
        let real_zero = ConvolutionWeightsPlan::try_new(&[0.0], &preparation, &limits).unwrap();
        let comparison = Account::for_test();
        assert!(left.matches_in(&right, &comparison).unwrap());
        assert!(!left.matches_in(&real_zero, &comparison).unwrap());
        assert_ne!(left.key(), real_zero.key());
        assert_eq!(
            comparison
                .work_count(RenderWork::KernelSampleVisits)
                .unwrap(),
            0
        );
    }

    #[test]
    fn equal_real_weight_allocations_bill_one_exact_comparison_per_tap() {
        let preparation = Account::for_test();
        let limits = wgpu::Limits::default();
        let left =
            ConvolutionWeightsPlan::try_new(&[1.0, -2.0, 0.0], &preparation, &limits).unwrap();
        let right =
            ConvolutionWeightsPlan::try_new(&[1.0, -2.0, 0.0], &preparation, &limits).unwrap();
        let comparison = Account::for_test();
        assert!(left.matches_in(&right, &comparison).unwrap());
        assert_eq!(
            comparison
                .work_count(RenderWork::KernelSampleVisits)
                .unwrap(),
            3
        );
    }

    #[test]
    fn weight_keys_compare_content_without_conflating_signed_zero() {
        let account = Account::for_test();
        let limits = wgpu::Limits::default();
        let a = ConvolutionWeightsPlan::try_new(&[1.0, 0.0], &account, &limits).unwrap();
        let b = ConvolutionWeightsPlan::try_new(&[1.0, 0.0], &account, &limits).unwrap();
        assert!(a.matches_in(&b, &account).unwrap());
        assert!(a.matches_in(&a.clone(), &account).unwrap());
        let mut collision =
            ConvolutionWeightsPlan::try_new(&[1.0, -0.0], &account, &limits).unwrap();
        collision.key.fingerprint = a.key.fingerprint;
        assert!(!a.matches_in(&collision, &account).unwrap());
    }
}
