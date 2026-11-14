use std::cell::Cell;
use std::rc::Rc;

use surgeist_bindings_js::{EngineKind, Error, ObjectPrototype, Process};

struct Funding(Rc<Cell<usize>>);

impl Drop for Funding {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn identity_clones_share_one_admission_through_process_teardown() -> Result<(), Error> {
    let drops = Rc::new(Cell::new(0));
    let admissions = Cell::new(0);
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let storage = Funding(drops.clone());
    let holder = process.execution().in_realm(&realm, |scope| {
        let value = scope.object(ObjectPrototype::Null)?;
        let observed = scope.callback_object(&value)?;
        scope.retain_callback_object(observed, |bytes| {
            assert!(bytes > size_of::<Funding>());
            admissions.set(admissions.get() + 1);
            Ok::<_, Error>(storage)
        })
    })?;
    let alias = holder.clone();
    process.execution().in_realm(&realm, |scope| {
        let original = scope.open_callback_object(&holder)?;
        let cloned = scope.open_callback_object(&alias)?;
        assert!(scope.strict_equals(&original, &cloned)?);
        Ok(())
    })?;
    assert_eq!(admissions.get(), 1);
    assert_eq!(drops.get(), 0);
    drop(process);
    assert!(alias.associated_realm().check_owner_active().is_err());
    assert_eq!(drops.get(), 0);

    let mut next = Process::bootstrap(EngineKind::V8)?;
    let receiving = next.execution().create_realm()?;
    next.execution().in_realm(&receiving, |scope| {
        assert!(matches!(
            scope.open_callback_object(&alias),
            Err(Error::ForeignOwner)
        ));
        Ok(())
    })?;
    drop(holder);
    assert_eq!(drops.get(), 0);
    drop(alias);
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn explicit_release_drops_handles_but_preserves_funding_for_surviving_aliases() -> Result<(), Error>
{
    let drops = Rc::new(Cell::new(0));
    let admissions = Cell::new(0);
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let storage = Funding(drops.clone());
    let holder = process.execution().in_realm(&realm, |scope| {
        let value = scope.object(ObjectPrototype::Default)?;
        let observed = scope.callback_object(&value)?;
        scope.retain_callback_object(observed, |_| {
            admissions.set(admissions.get() + 1);
            Ok::<_, Error>(storage)
        })
    })?;
    let alias = holder.clone();
    process.release_callback_object(holder)?;
    assert_eq!(drops.get(), 0);
    process.execution().in_realm(&realm, |scope| {
        assert!(matches!(
            scope.open_callback_object(&alias),
            Err(Error::ExpiredIdentity)
        ));
        Ok(())
    })?;
    assert_eq!(admissions.get(), 1);
    drop(alias);
    assert_eq!(drops.get(), 1);
    drop(process);
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn native_holder_metadata_does_not_keep_its_sdk_entry_or_payload_alive() -> Result<(), Error> {
    let native_drops = Rc::new(Cell::new(0));
    let payload_funding_drops = Rc::new(Cell::new(0));
    let holder_funding_drops = Rc::new(Cell::new(0));
    let admissions = Cell::new(0);
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let payload = Funding(native_drops.clone());
    let mut funding = [
        Funding(payload_funding_drops.clone()),
        Funding(holder_funding_drops.clone()),
    ]
    .into_iter();
    let holder = process.execution().in_realm(&realm, |scope| {
        scope.project_retained_native_object(ObjectPrototype::Null, payload, |_| {
            admissions.set(admissions.get() + 1);
            funding.next().ok_or(Error::BackendUnavailable)
        })
    })?;
    let alias = holder.clone();
    assert_eq!(admissions.get(), 2);
    assert_eq!(native_drops.get(), 0);
    assert_eq!(payload_funding_drops.get(), 0);
    assert_eq!(holder_funding_drops.get(), 0);
    drop(process);
    assert_eq!(native_drops.get(), 1);
    assert_eq!(payload_funding_drops.get(), 1);
    assert_eq!(holder_funding_drops.get(), 0);
    drop(holder);
    assert_eq!(holder_funding_drops.get(), 0);
    drop(alias);
    assert_eq!(holder_funding_drops.get(), 1);
    Ok(())
}

pub(super) fn holder_custody_contracts() -> Result<(), Error> {
    identity_clones_share_one_admission_through_process_teardown()?;
    explicit_release_drops_handles_but_preserves_funding_for_surviving_aliases()?;
    native_holder_metadata_does_not_keep_its_sdk_entry_or_payload_alive()
}
