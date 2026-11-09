use super::*;
use crate::accounting::{
    RenderAccountingOptions, RenderClock, RenderCpuScope, RenderProfile, RendererCpu,
    StorageAccount, StorageContractError,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
static TIME: AtomicU64 = AtomicU64::new(0);
fn clock() -> std::time::Duration {
    std::time::Duration::from_millis(TIME.load(Ordering::SeqCst))
}
struct Payload {
    bytes: Vec<u8>,
    cpu: RendererCpu,
    admitted: u64,
    drops: Arc<AtomicUsize>,
}
impl Drop for Payload {
    fn drop(&mut self) {
        assert_eq!(
            self.cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            self.admitted
        );
        assert_eq!(self.bytes, [1, 2, 3]);
        TIME.fetch_add(7, Ordering::SeqCst);
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn renderer_retention_refusal_drops_payload_before_credit_in_the_actual_owner_phase() {
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ] {
        TIME.store(0, Ordering::SeqCst);
        let cpu = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_profile(profile)
                .with_clock(RenderClock::Host(clock)),
        )
        .unwrap();
        let baseline = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let mut bytes =
            AccountedVec::<u8>::new_in(&storage, 3, RenderResource::RetainedCpuBytes).unwrap();
        bytes.copy_slot(&[1, 2, 3]).unwrap().write();
        let admitted = baseline + bytes.capacity() as u64;
        let (bytes, hold) = bytes.into_parts().unwrap();
        let mut custody = StyleCustody::new(&Account::for_test()).unwrap();
        custody.hold(hold).unwrap();
        let drops = Arc::new(AtomicUsize::new(0));
        TIME.store(5, Ordering::SeqCst);
        let error = match custody.transfer(Payload {
            bytes,
            cpu: cpu.clone(),
            admitted,
            drops: drops.clone(),
        }) {
            Err(error) => error,
            Ok(_) => panic!("renderer custody cannot become a standalone result"),
        };
        assert!(error.accounting_diagnostic().is_none());
        assert_eq!(
            std::error::Error::source(&error)
                .and_then(|s| s.downcast_ref::<StorageContractError>()),
            Some(&StorageContractError::WrongRetentionLifetime)
        );
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let closing = cpu.snapshot().unwrap();
        assert_eq!(
            closing.usage().live(RenderResource::RetainedCpuBytes),
            baseline
        );
        assert_eq!(
            closing.scope_elapsed(RenderCpuScope::FrameBacking),
            if profile == (RenderProfile::Summary { timing: true }) {
                Some(std::time::Duration::from_millis(7))
            } else {
                None
            }
        );
    }
}
