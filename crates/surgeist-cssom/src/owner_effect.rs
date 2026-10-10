//! Bounded owner writes are reserved with publication and applied by a binding.
use crate::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Default)]
pub(crate) struct EffectQuota {
    count: AtomicUsize,
    bytes: AtomicUsize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssomOwnerEffectId {
    owner: CssomOwnerId,
    serial: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomOwnerEffectStatus {
    Pending,
    Applying,
    Applied,
    HostFailed,
    Abandoned,
    Canceled,
}
/// Non-clone single-use write reservation. Dropping it releases its pending quota.
#[derive(Debug)]
pub struct CssomOwnerEffect {
    id: CssomOwnerEffectId,
    block: CssomBlockId,
    owner: CssomInputVersion,
    revision: CssomRevision,
    inputs: Arc<CssomInputManifest>,
    value: String,
    status: CssomOwnerEffectStatus,
    quota: Arc<EffectQuota>,
    reserved: bool,
}
impl CssomOwnerEffect {
    pub fn id(&self) -> &CssomOwnerEffectId {
        &self.id
    }
    pub fn block(&self) -> &CssomBlockId {
        &self.block
    }
    pub fn owner(&self) -> &CssomInputVersion {
        &self.owner
    }
    pub fn revision(&self) -> &CssomRevision {
        &self.revision
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn status(&self) -> CssomOwnerEffectStatus {
        self.status
    }
    pub fn cancel(&mut self) -> Result<(), CssomError> {
        if self.status != CssomOwnerEffectStatus::Pending {
            return Err(CssomError::EffectAlreadyConsumed);
        }
        self.status = CssomOwnerEffectStatus::Canceled;
        self.release();
        Ok(())
    }
    fn release(&mut self) {
        if self.reserved {
            self.reserved = false;
            self.quota.count.fetch_sub(1, Ordering::Relaxed);
            self.quota
                .bytes
                .fetch_sub(self.value.len(), Ordering::Relaxed);
        }
    }
}
impl Drop for CssomOwnerEffect {
    fn drop(&mut self) {
        self.release();
    }
}
/// Exact host facts. Unrelated notifications remain owned/queued by the caller.
#[derive(Clone, Debug)]
pub struct CssomStyleAttributeChange {
    pub owner: CssomInputVersion,
    pub local_name: String,
    pub namespace: Option<String>,
    pub value: Option<String>,
    pub echo: Option<CssomOwnerEffectId>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomOwnerNotification {
    MatchingEcho,
    Unrelated,
}
/// The binding holds this before its synchronous write. No host callback runs in CSSOM.
pub struct CssomOwnerApplication<'store, 'effect> {
    store: &'store mut CssomStore,
    effect: &'effect mut CssomOwnerEffect,
}
impl CssomOwnerApplication<'_, '_> {
    pub fn effect(&self) -> &CssomOwnerEffect {
        self.effect
    }
    pub fn flags(&self) -> CssomBlockFlags {
        self.store.state.blocks[&self.effect.block]
            .flags
            .with_updating()
    }
    pub fn classify(&self, change: &CssomStyleAttributeChange) -> CssomOwnerNotification {
        if change.echo.as_ref() == Some(&self.effect.id)
            && change.owner == self.effect.owner
            && change.local_name == "style"
            && change.namespace.is_none()
            && change.value.as_deref() == Some(self.effect.value.as_str())
        {
            CssomOwnerNotification::MatchingEcho
        } else {
            CssomOwnerNotification::Unrelated
        }
    }
    pub fn finish_success(mut self) {
        self.finish(CssomOwnerEffectStatus::Applied);
    }
    /// Host failure follows publication; it does not roll back CSS or its revision.
    pub fn finish_host_failure(mut self) {
        self.finish(CssomOwnerEffectStatus::HostFailed);
    }
    fn finish(&mut self, status: CssomOwnerEffectStatus) {
        self.effect.status = status;
    }
}
impl Drop for CssomOwnerApplication<'_, '_> {
    fn drop(&mut self) {
        self.store.updating = None;
        if self.effect.status == CssomOwnerEffectStatus::Applying {
            self.effect.status = CssomOwnerEffectStatus::Abandoned;
        }
        self.effect.release();
    }
}
impl CssomBlockFlags {
    fn with_updating(mut self) -> Self {
        self.updating = true;
        self
    }
}
impl CssomStore {
    pub fn declaration_flags(&self, id: &CssomBlockId) -> Result<CssomBlockFlags, CssomError> {
        let mut flags = self.state.block(id)?.flags;
        flags.updating = self.updating.as_ref() == Some(id);
        Ok(flags)
    }
    pub fn begin_owner_application<'store, 'effect>(
        &'store mut self,
        effect: &'effect mut CssomOwnerEffect,
    ) -> Result<CssomOwnerApplication<'store, 'effect>, CssomError> {
        if effect.id.owner != self.state.owner {
            return Err(CssomError::ForeignOwner);
        }
        if effect.status != CssomOwnerEffectStatus::Pending {
            return Err(CssomError::EffectAlreadyConsumed);
        }
        if self.state.revision != effect.revision
            || self.snapshot().inputs() != *effect.inputs
            || self.state.block(&effect.block)?.owner.as_ref() != Some(&effect.owner)
        {
            return Err(CssomError::StaleOwnerEffect);
        }
        effect.status = CssomOwnerEffectStatus::Applying;
        self.updating = Some(effect.block.clone());
        Ok(CssomOwnerApplication {
            store: self,
            effect,
        })
    }
    pub(crate) fn reserve_owner_effects(
        &self,
        batch: &CssomBatch,
        revision: &CssomRevision,
    ) -> Result<(Vec<CssomOwnerEffect>, u64), CssomError> {
        let mut effects = Vec::new();
        let mut next = self.next_effect;
        if batch.owner_updates.is_empty() {
            return Ok((effects, next));
        }
        let inputs = Arc::new(batch.staged.input_manifest());
        for (id, budget) in &batch.owner_updates {
            let block = batch.staged.block(id)?;
            let Some(owner) = &block.owner else {
                continue;
            };
            if next == 0 || next > self.limits.max_owner_effect_identity {
                return Err(CssomError::EffectIdentityExhausted);
            }
            let mut budget = budget.clone();
            let value = crate::declaration::serialize_block(block, &mut budget)?;
            effects.try_reserve(1).map_err(|_| CssomError::Limit {
                resource: "owner effect capacity",
                maximum: self.limits.max_pending_owner_effects,
            })?;
            self.effect_quota
                .count
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                    count
                        .checked_add(1)
                        .filter(|n| *n <= self.limits.max_pending_owner_effects)
                })
                .map_err(|_| CssomError::Limit {
                    resource: "pending owner effects",
                    maximum: self.limits.max_pending_owner_effects,
                })?;
            if self
                .effect_quota
                .bytes
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |bytes| {
                    bytes
                        .checked_add(value.len())
                        .filter(|n| *n <= self.limits.max_pending_owner_effect_bytes)
                })
                .is_err()
            {
                self.effect_quota.count.fetch_sub(1, Ordering::Relaxed);
                return Err(CssomError::Limit {
                    resource: "pending owner effect bytes",
                    maximum: self.limits.max_pending_owner_effect_bytes,
                });
            }
            effects.push(CssomOwnerEffect {
                id: CssomOwnerEffectId {
                    owner: self.state.owner.clone(),
                    serial: next,
                },
                block: id.clone(),
                owner: owner.clone(),
                revision: revision.clone(),
                inputs: inputs.clone(),
                value,
                status: CssomOwnerEffectStatus::Pending,
                quota: self.effect_quota.clone(),
                reserved: true,
            });
            next = next.checked_add(1).unwrap_or(0);
        }
        Ok((effects, next))
    }
}
