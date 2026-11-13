use super::*;
use crate::trusted_types_csp::{Directive, PolicyDelivery};
use std::{cell::Cell, convert::Infallible};

fn step() -> Result<(), Infallible> {
    Ok(())
}

fn policy(text: &str, delivery: PolicyDelivery) -> Policy {
    Policy::parse(text.into(), delivery, &mut step).unwrap()
}

#[test]
fn names_are_case_sensitive_and_nondefault_policy_payloads_are_not_kept_alive() {
    let mut factory = PolicyFactory::<(), ()>::default();
    let first = Rc::new(());
    let probe = Rc::downgrade(&first);
    let uppercase = "Name".into();
    factory
        .begin(&uppercase, &[], &mut step)
        .unwrap()
        .admit(|_| step())
        .unwrap()
        .publish(&first);
    drop(first);
    assert!(probe.upgrade().is_none());

    let list = [policy("trusted-types *", PolicyDelivery::EnforcingHeader)];
    let lower = "name".into();
    let mut distinct = factory.begin(&lower, &list, &mut step).unwrap();
    assert!(distinct.next_violation(&mut step).unwrap().is_none());
    distinct.admit(|_| step()).unwrap().publish(&Rc::new(()));
    let mut duplicate = factory.begin(&uppercase, &list, &mut step).unwrap();
    let violation = duplicate.next_violation(&mut step).unwrap().unwrap();
    assert_eq!(violation.directive(), Directive::TrustedTypes);
    assert_eq!(
        duplicate
            .admit(|_| -> Result<(), Infallible> { panic!("blocked names cannot reserve") })
            .err(),
        Some(PolicyCreationError::BlockedByCsp)
    );
}

#[test]
fn default_policy_identity_is_retained_and_csp_is_reached_before_duplicate_default() {
    let mut factory = PolicyFactory::<u32, ()>::default();
    let default = "default".into();
    let first = Rc::new(17);
    factory
        .begin(&default, &[], &mut step)
        .unwrap()
        .admit(|_| step())
        .unwrap()
        .publish(&first);
    assert!(Rc::ptr_eq(factory.default_policy().unwrap(), &first));

    let policies = [
        policy("trusted-types 'none'", PolicyDelivery::ReportOnlyHeader),
        policy(
            "trusted-types * 'allow-duplicates'",
            PolicyDelivery::EnforcingHeader,
        ),
    ];
    let pending = factory.begin(&default, &policies, &mut step).unwrap();
    assert_eq!(
        pending.admit(|_| step()).err(),
        Some(PolicyCreationError::PoliciesPending)
    );
    let mut duplicate = factory.begin(&default, &policies, &mut step).unwrap();
    let report = duplicate.next_violation(&mut step).unwrap().unwrap();
    assert_eq!(report.policy_index(), 0);
    assert!(duplicate.next_violation(&mut step).unwrap().is_none());
    assert_eq!(
        duplicate.admit(|_| step()).err(),
        Some(PolicyCreationError::DefaultAlreadyExists)
    );
    assert!(Rc::ptr_eq(factory.default_policy().unwrap(), &first));
}

#[test]
fn refused_or_abandoned_admission_leaves_the_exact_name_available() {
    struct Debit(Rc<Cell<usize>>);
    impl Drop for Debit {
        fn drop(&mut self) {
            self.0.set(self.0.get() - 1);
        }
    }
    let debits = Rc::new(Cell::new(0));
    let mut factory = PolicyFactory::<(), Debit>::default();
    let name = DocumentString::from_utf16([0xd800, 0, 0x41]);
    let refused = factory
        .begin(&name, &[], &mut step)
        .unwrap()
        .admit(|_| Err("cap"));
    assert_eq!(refused.err(), Some(PolicyCreationError::Admission("cap")));
    let debit = |bytes| -> Result<Debit, Infallible> {
        assert!(bytes >= 3 * size_of::<u16>() + size_of::<Vec<u16>>());
        debits.set(debits.get() + 1);
        Ok(Debit(debits.clone()))
    };
    let list = [policy("trusted-types *", PolicyDelivery::EnforcingHeader)];
    let mut pending = factory.begin(&name, &list, &mut step).unwrap();
    assert!(pending.next_violation(&mut step).unwrap().is_none());
    let admitted = pending.admit(debit).unwrap();
    assert_eq!(debits.get(), 1);
    drop(admitted);
    assert_eq!(debits.get(), 0);

    let mut retry = factory.begin(&name, &list, &mut step).unwrap();
    assert!(retry.next_violation(&mut step).unwrap().is_none());
    retry.admit(debit).unwrap().publish(&Rc::new(()));
    assert_eq!(debits.get(), 1);
    drop(factory);
    assert_eq!(debits.get(), 0);
}
