//! Immutable, exactly compared displacement storage-buffer uploads.
use crate::{
    Result,
    accounting::{Account, CpuAccount, RenderCpuScope, RenderWork, SharedValue},
    displacement::DisplacementNumericPhase,
    displacement::DisplacementPreparationError,
    frame::{DisplacementAddressEntry, DisplacementMapRecord},
};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DisplacementBufferKind {
    AddressTables,
    MapRecords,
}

/// Actual selected-device facts used by early qualification and native acquisition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DisplacementBufferLimits {
    max_buffer_size: u64,
    max_storage_binding: u64,
    max_uniform_binding: u64,
    max_storage_buffers: u32,
    max_sampled_textures: u32,
    max_bindings: u32,
    max_uniform_buffers: u32,
    max_groups: u32,
}
impl DisplacementBufferLimits {
    pub(crate) const fn from_device_limits(limits: &wgpu::Limits) -> Self {
        Self {
            max_buffer_size: limits.max_buffer_size,
            max_storage_binding: limits.max_storage_buffer_binding_size,
            max_uniform_binding: limits.max_uniform_buffer_binding_size,
            max_storage_buffers: limits.max_storage_buffers_per_shader_stage,
            max_sampled_textures: limits.max_sampled_textures_per_shader_stage,
            max_bindings: limits.max_bindings_per_bind_group,
            max_uniform_buffers: limits.max_uniform_buffers_per_shader_stage,
            max_groups: limits.max_bind_groups,
        }
    }
    pub(crate) fn validate_bindings(
        self,
        storage_buffers: u32,
        sampled_textures: u32,
        bindings: u32,
    ) -> Result<()> {
        let error = |value| invalid(DisplacementNumericPhase::TableLayout, value);
        if self.max_buffer_size < 256 || self.max_uniform_binding < 256 {
            return Err(error(256.0));
        }
        if self.max_uniform_buffers < 1 || self.max_groups < 1 {
            return Err(error(1.0));
        }
        if storage_buffers > self.max_storage_buffers {
            return Err(error(f64::from(storage_buffers)));
        }
        if sampled_textures > self.max_sampled_textures {
            return Err(error(f64::from(sampled_textures)));
        }
        if bindings > self.max_bindings {
            return Err(error(f64::from(bindings)));
        }
        Ok(())
    }
    fn validate_storage(self, length: u64) -> Result<()> {
        if length == 0 || length > self.max_buffer_size || length > self.max_storage_binding {
            return Err(invalid(
                DisplacementNumericPhase::TableLayout,
                length as f64,
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DisplacementBufferKey {
    kind: DisplacementBufferKind,
    fingerprint: u64,
    bytes: SharedValue<Vec<u8>>,
    cpu: CpuAccount,
}

impl PartialEq for DisplacementBufferKey {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.fingerprint == other.fingerprint
            && self.bytes == other.bytes
    }
}
impl Eq for DisplacementBufferKey {}
impl Hash for DisplacementBufferKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.fingerprint.hash(state);
        self.bytes.len().hash(state);
    }
}

impl DisplacementBufferKey {
    #[cfg(test)]
    pub(super) const fn kind(&self) -> DisplacementBufferKind {
        self.kind
    }
    pub(super) fn matches_in(&self, other: &Self, observer: Option<&Account>) -> Result<bool> {
        if self.kind != other.kind
            || self.fingerprint != other.fingerprint
            || self.bytes.len() != other.bytes.len()
        {
            return Ok(false);
        }
        if self.bytes.same_allocation(&other.bytes) {
            return Ok(true);
        }
        let compare = |visit: &mut dyn FnMut() -> Result<()>| -> Result<bool> {
            for (left, right) in self
                .bytes
                .as_chunks::<4>()
                .0
                .iter()
                .zip(other.bytes.as_chunks::<4>().0.iter())
            {
                visit()?;
                if left != right {
                    return Ok(false);
                }
            }
            Ok(true)
        };
        match observer {
            Some(account) => compare(&mut || account.work(RenderWork::ResourceProbes, 1)),
            None => {
                let cpu = self.cpu.at(RenderCpuScope::Cleanup);
                let mut result = Ok(false);
                cpu.maintenance_phase(|| {
                    result = compare(&mut || {
                        cpu.maintenance(RenderWork::ResourceProbes, 1);
                        Ok(())
                    });
                });
                result
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DisplacementBufferPlan {
    key: DisplacementBufferKey,
}

impl DisplacementBufferPlan {
    pub(crate) fn validate_layout(
        kind: DisplacementBufferKind,
        count: usize,
        limits: DisplacementBufferLimits,
    ) -> Result<usize> {
        if (kind == DisplacementBufferKind::AddressTables && count != 512) || count == 0 {
            return Err(invalid(
                if kind == DisplacementBufferKind::MapRecords {
                    DisplacementNumericPhase::MapMapping
                } else {
                    DisplacementNumericPhase::TableLayout
                },
                count as f64,
            ));
        }
        let stride = if kind == DisplacementBufferKind::AddressTables {
            8
        } else {
            16
        };
        let length = count
            .checked_mul(stride)
            .ok_or_else(|| invalid(DisplacementNumericPhase::TableLayout, count as f64))?;
        limits.validate_storage(length as u64)?;
        Ok(length)
    }
    pub(crate) fn address_tables(
        entries: &[DisplacementAddressEntry],
        account: &Account,
        limits: &wgpu::Limits,
    ) -> Result<Self> {
        Self::serialize(
            DisplacementBufferKind::AddressTables,
            entries.len(),
            entries.iter().flat_map(|entry| {
                [
                    u32::from_ne_bytes(entry.q0().to_ne_bytes()),
                    u32::from(entry.outside()),
                ]
            }),
            account,
            limits,
        )
    }

    pub(crate) fn map_records(
        records: &[DisplacementMapRecord],
        account: &Account,
        limits: &wgpu::Limits,
    ) -> Result<Self> {
        Self::serialize(
            DisplacementBufferKind::MapRecords,
            records.len(),
            records.iter().flat_map(|record| {
                [
                    record.pixel(),
                    record.coverage().to_bits(),
                    u32::from(record.outside()),
                    0,
                ]
            }),
            account,
            limits,
        )
    }

    fn serialize(
        kind: DisplacementBufferKind,
        count: usize,
        words: impl Iterator<Item = u32>,
        account: &Account,
        limits: &wgpu::Limits,
    ) -> Result<Self> {
        let length = Self::validate_layout(
            kind,
            count,
            DisplacementBufferLimits::from_device_limits(limits),
        )?;
        account.work(RenderWork::SerializedBytes, length)?;
        let mut bytes = account.retained(length)?;
        let mut fingerprint = std::collections::hash_map::DefaultHasher::new();
        kind.hash(&mut fingerprint);
        length.hash(&mut fingerprint);
        for word in words {
            let word = word.to_le_bytes();
            word.hash(&mut fingerprint);
            bytes.extend_from_slice(&word)?;
        }
        if bytes.len() != length {
            return Err(crate::accounting::RenderAccountingError::Invariant.into());
        }
        Ok(Self {
            key: DisplacementBufferKey {
                kind,
                fingerprint: fingerprint.finish(),
                bytes: SharedValue::from_accounted(bytes, &account.storage())?,
                cpu: account.cpu()?.clone(),
            },
        })
    }

    pub(crate) fn kind(&self) -> DisplacementBufferKind {
        self.key.kind
    }
    pub(crate) fn key(&self) -> DisplacementBufferKey {
        self.key.clone()
    }
    pub(crate) fn byte_len(&self) -> u64 {
        self.key.bytes.len() as u64
    }
    pub(crate) fn upload_bytes(&self) -> &[u8] {
        &self.key.bytes
    }
    pub(crate) fn validate_upload_byte_len(&self, actual: usize) -> Result<()> {
        if actual != self.key.bytes.len() {
            return Err(crate::accounting::RenderAccountingError::Invariant.into());
        }
        Ok(())
    }
    pub(crate) fn validate_buffer_limits(&self, limits: &wgpu::Limits) -> Result<()> {
        DisplacementBufferLimits::from_device_limits(limits).validate_storage(self.byte_len())
    }
}

fn invalid(phase: DisplacementNumericPhase, value: f64) -> crate::Error {
    DisplacementPreparationError::Numeric { phase, value }.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_storage_limit_accepts_tables_and_one_under_refuses_before_serialization() {
        let account = Account::for_test();
        let mut limits = wgpu::Limits {
            max_buffer_size: 4096,
            max_storage_buffer_binding_size: 4096,
            ..wgpu::Limits::default()
        };
        let table = DisplacementBufferPlan::serialize(
            DisplacementBufferKind::AddressTables,
            512,
            std::iter::repeat_n(0, 1024),
            &account,
            &limits,
        )
        .unwrap();
        assert_eq!(table.byte_len(), 4096);
        assert_eq!(table.upload_bytes(), &[0; 4096]);
        let refused = Account::for_test();
        limits.max_storage_buffer_binding_size = 4095;
        assert!(
            DisplacementBufferPlan::serialize(
                DisplacementBufferKind::AddressTables,
                512,
                std::iter::repeat_n(0, 1024),
                &refused,
                &limits
            )
            .is_err()
        );
        assert_eq!(refused.work_count(RenderWork::SerializedBytes).unwrap(), 0);
    }
    #[test]
    fn storage_cache_fingerprint_collisions_compare_the_exact_word_prefix() {
        let account = Account::for_test();
        let limits = wgpu::Limits::default();
        let make = |pixel: u32| {
            let mut value = DisplacementBufferPlan::serialize(
                DisplacementBufferKind::MapRecords,
                1,
                [0, 0.5f32.to_bits(), pixel, 0].into_iter(),
                &account,
                &limits,
            )
            .unwrap();
            value.key.fingerprint = 7;
            value.key
        };
        let left = make(0);
        let equal = make(0);
        let collision = make(1);
        assert!(left.matches_in(&left.clone(), Some(&account)).unwrap());
        assert_eq!(account.work_count(RenderWork::ResourceProbes).unwrap(), 0);
        assert!(left.matches_in(&equal, Some(&account)).unwrap());
        assert!(!left.matches_in(&collision, Some(&account)).unwrap());
        assert_eq!(account.work_count(RenderWork::ResourceProbes).unwrap(), 7);
        assert_ne!(left, collision);
    }
}
