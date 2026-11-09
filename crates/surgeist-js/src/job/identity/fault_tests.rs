use super::*;
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
};

#[derive(Default)]
struct TaskLog {
    marked: Cell<Option<JobFault>>,
    faults: RefCell<Vec<JobFault>>,
    retired: RefCell<Vec<JobRetirement>>,
    owner: RefCell<Option<Weak<ContextLifetime>>>,
    callback_unlocked: Cell<bool>,
    panic_on_fault: Cell<bool>,
}
impl TaskLog {
    fn for_owner(owner: &Rc<ContextLifetime>) -> Rc<Self> {
        Rc::new(Self {
            owner: RefCell::new(Some(Rc::downgrade(owner))),
            callback_unlocked: Cell::new(true),
            ..Self::default()
        })
    }
    fn check_callback(&self) {
        if let Some(owner) = self.owner.borrow().as_ref().and_then(Weak::upgrade) {
            self.callback_unlocked
                .set(self.callback_unlocked.get() && owner.registry.try_borrow_mut().is_ok());
            // The retained fault rejects this real registration before mutation.
            if owner.first_fault.get().is_some() {
                let attempt = owner.register(Rc::new(Self::default()));
                assert!(matches!(attempt, Err(JobRegistrationError::Fault(_))));
            }
        }
    }
}
#[derive(Debug)]
struct CallbackPanic;
impl OwnerTask for TaskLog {
    fn mark_fault(&self, fault: JobFault) {
        self.marked.set(self.marked.get().or(Some(fault)));
    }
    fn fault(&self, fault: JobFault) {
        assert_eq!(
            self.marked.get(),
            Some(fault),
            "fault delivery must follow its first-cause mark"
        );
        self.faults.borrow_mut().push(fault);
        self.check_callback();
        if self.panic_on_fault.get() {
            std::panic::panic_any(CallbackPanic);
        }
    }
    fn retire(&self, reason: JobRetirement) {
        self.retired.borrow_mut().push(reason);
        self.check_callback();
    }
}
fn fault(cause: JobFaultCause) -> JobFault {
    JobFault::new(JobFaultOwner::ContextLifetime, cause)
}
fn registration_error(
    result: Result<OwnerRegistration, JobRegistrationError>,
) -> JobRegistrationError {
    match result {
        Err(error) => error,
        Ok(_) => panic!("registration unexpectedly accepted"),
    }
}

#[test]
fn inaccessible_registry_fault_waits_for_actual_borrow_release_then_notifies_every_owner_task() {
    let owner = ContextLifetime::new();
    let tasks: Vec<_> = (0..3).map(|_| TaskLog::for_owner(&owner)).collect();
    let _registrations: Vec<_> = tasks
        .iter()
        .map(|task| owner.register(task.clone()).unwrap())
        .collect();
    let expected = fault(JobFaultCause::InaccessibleRegistry);
    owner
        .with_registry(|_| {
            assert_eq!(
                registration_error(owner.register(TaskLog::for_owner(&owner))),
                JobRegistrationError::Fault(expected)
            );
            assert!(tasks.iter().all(|task| task.faults.borrow().is_empty()));
            assert!(owner.cleanup_pending.get());
            Ok(())
        })
        .unwrap_err();
    for task in &tasks {
        assert_eq!(task.marked.get(), Some(expected));
        assert_eq!(&*task.faults.borrow(), &[expected]);
        assert!(task.callback_unlocked.get());
        assert!(task.retired.borrow().is_empty());
    }
    assert!(!owner.cleanup_pending.get());
    assert_eq!(
        registration_error(owner.register(TaskLog::for_owner(&owner))),
        JobRegistrationError::Fault(expected)
    );
    owner.retire();
    // Cleanup already detached the physical task bank; no second fault callback.
    assert!(tasks.iter().all(|task| task.faults.borrow().len() == 1));
}

#[test]
fn damaged_free_head_is_not_followed_during_physical_owner_cleanup() {
    let owner = ContextLifetime::new();
    let tasks: Vec<_> = (0..3).map(|_| TaskLog::for_owner(&owner)).collect();
    let _registrations: Vec<_> = tasks
        .iter()
        .map(|task| owner.register(task.clone()).unwrap())
        .collect();
    owner.registry.borrow_mut().free = Some(usize::MAX);
    let expected = fault(JobFaultCause::InvalidFreeLink { index: usize::MAX });
    assert_eq!(
        registration_error(owner.register(TaskLog::for_owner(&owner))),
        JobRegistrationError::Fault(expected)
    );
    for task in &tasks {
        assert_eq!(task.marked.get(), Some(expected));
        assert_eq!(&*task.faults.borrow(), &[expected]);
        assert!(task.callback_unlocked.get());
    }
    assert_eq!(owner.first_fault.get(), Some(expected));
    assert_eq!(
        registration_error(owner.register(TaskLog::for_owner(&owner))),
        JobRegistrationError::Fault(expected)
    );
}

#[test]
fn stale_owner_key_reports_identity_fault_without_removing_the_replacement_task() {
    let owner = ContextLifetime::new();
    let old_task = TaskLog::for_owner(&owner);
    let original = owner.register(old_task.clone()).unwrap();
    let stale = OwnerRegistration {
        owner: original.owner.clone(),
        index: original.index,
        expected: original.expected.clone(),
    };
    let index = original.index;
    drop(original);
    let replacement = TaskLog::for_owner(&owner);
    let _current = owner.register(replacement.clone()).unwrap();
    drop(stale);
    let expected = fault(JobFaultCause::InvalidSlotIdentity { index });
    assert_eq!(replacement.marked.get(), Some(expected));
    assert_eq!(&*replacement.faults.borrow(), &[expected]);
    assert!(replacement.callback_unlocked.get());
    assert!(old_task.marked.get().is_none());
    assert!(old_task.faults.borrow().is_empty());
    assert_eq!(
        registration_error(owner.register(TaskLog::for_owner(&owner))),
        JobRegistrationError::Fault(expected)
    );
}

#[test]
fn drop_only_owner_index_error_is_retained_and_cleans_the_remaining_bank() {
    let owner = ContextLifetime::new();
    let task = TaskLog::for_owner(&owner);
    let mut registration = owner.register(task.clone()).unwrap();
    registration.index = usize::MAX;
    drop(registration);
    let expected = fault(JobFaultCause::InvalidIndex { index: usize::MAX });
    assert_eq!(task.marked.get(), Some(expected));
    assert_eq!(&*task.faults.borrow(), &[expected]);
    assert!(task.callback_unlocked.get());
    assert_eq!(
        registration_error(owner.register(TaskLog::for_owner(&owner))),
        JobRegistrationError::Fault(expected)
    );
}

#[test]
fn a_panicking_owner_callback_cannot_skip_other_tasks_or_hold_the_registry_borrow() {
    let owner = ContextLifetime::new();
    let tasks: Vec<_> = (0..3).map(|_| TaskLog::for_owner(&owner)).collect();
    tasks[0].panic_on_fault.set(true);
    let _registrations: Vec<_> = tasks
        .iter()
        .map(|task| owner.register(task.clone()).unwrap())
        .collect();
    owner.registry.borrow_mut().free = Some(usize::MAX);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = owner.register(TaskLog::for_owner(&owner));
    }))
    .unwrap_err();
    assert!(panic.is::<CallbackPanic>());
    let expected = fault(JobFaultCause::InvalidFreeLink { index: usize::MAX });
    for task in &tasks {
        assert_eq!(task.marked.get(), Some(expected));
        assert_eq!(&*task.faults.borrow(), &[expected]);
        assert!(task.callback_unlocked.get());
    }
    assert_eq!(
        registration_error(owner.register(TaskLog::for_owner(&owner))),
        JobRegistrationError::Fault(expected)
    );
}

#[test]
fn normal_owner_retirement_is_exact_and_does_not_create_a_fault() {
    let owner = ContextLifetime::new();
    let tasks: Vec<_> = (0..2).map(|_| TaskLog::for_owner(&owner)).collect();
    let registrations: Vec<_> = tasks
        .iter()
        .map(|task| owner.register(task.clone()).unwrap())
        .collect();
    owner.retire();
    owner.retire();
    drop(registrations);
    assert!(owner.first_fault.get().is_none());
    for task in &tasks {
        assert!(task.marked.get().is_none());
        assert!(task.faults.borrow().is_empty());
        assert_eq!(&*task.retired.borrow(), &[JobRetirement::OwnerDropped]);
        assert!(task.callback_unlocked.get());
    }
}
