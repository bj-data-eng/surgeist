//! Exact callback object custody and scoped Boa Realm entry.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, btree_map::Entry as MapEntry};
use std::ops::Bound::{Excluded, Included};
use std::rc::{Rc, Weak};

use boa_engine::realm::Realm as BoaRealm;
use boa_engine::{Context, JsObject};

use crate::callback_object::{CallbackObjectIdentity, HolderBacking};
use crate::{Error, RealmKey, Root};

use super::{Entered, Operations, State};

struct Entry {
    identity: Weak<CallbackObjectIdentity>,
    object: JsObject,
    _realm: BoaRealm,
    // Retire the guest handles before releasing the backend's share of funding.
    _backing: Rc<HolderBacking>,
}

#[derive(Default)]
pub(super) struct CallbackObjects {
    next: Cell<u64>,
    entries: RefCell<BTreeMap<u64, Rc<Entry>>>,
}

impl CallbackObjects {
    pub(super) const fn storage_bytes() -> usize {
        size_of::<CallbackObjectIdentity>()
            + size_of::<Entry>()
            + size_of::<(u64, Rc<Entry>)>()
            + size_of::<HolderBacking>()
    }

    fn matches(entry: &Entry, holder: &Rc<CallbackObjectIdentity>) -> bool {
        entry.identity.ptr_eq(&Rc::downgrade(holder))
    }

    fn check_owner(state: &State, holder: &CallbackObjectIdentity) -> Result<(), Error> {
        if holder.owner == state.owner.id() {
            Ok(())
        } else {
            Err(Error::ForeignOwner)
        }
    }

    fn entry(
        &self,
        state: &State,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<Rc<Entry>, Error> {
        Self::check_owner(state, holder)?;
        self.entries
            .try_borrow()
            .map_err(|_| state.owner.fail(Error::RegistryConflict))?
            .get(&holder.index)
            .filter(|entry| Self::matches(entry, holder))
            .cloned()
            .ok_or(Error::ExpiredIdentity)
    }

    pub(super) fn validate(
        &self,
        state: &State,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<(), Error> {
        self.entry(state, holder).map(|_| ())
    }

    pub(super) fn open(
        &self,
        state: &State,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<JsObject, Error> {
        Ok(self.entry(state, holder)?.object.clone())
    }

    pub(super) fn open_with_realm(
        &self,
        state: &State,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<(JsObject, BoaRealm), Error> {
        let entry = self.entry(state, holder)?;
        Ok((entry.object.clone(), entry._realm.clone()))
    }

    fn retain(
        &self,
        state: &State,
        object: JsObject,
        realm: BoaRealm,
        backing: Box<dyn Any>,
    ) -> Result<Rc<CallbackObjectIdentity>, Error> {
        let index = self
            .next
            .get()
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let backing = HolderBacking::new(backing);
        let identity = Rc::new(CallbackObjectIdentity {
            owner: state.owner.id(),
            index,
            _backing: backing.clone(),
        });
        let entry = Rc::new(Entry {
            identity: Rc::downgrade(&identity),
            object,
            _realm: realm,
            _backing: backing,
        });
        let mut entries = self
            .entries
            .try_borrow_mut()
            .map_err(|_| state.owner.fail(Error::RegistryConflict))?;
        let MapEntry::Vacant(slot) = entries.entry(index) else {
            return Err(state.owner.fail(Error::RegistryConflict));
        };
        slot.insert(entry);
        self.next.set(index);
        Ok(identity)
    }

    pub(super) fn release(
        &self,
        state: &State,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<(), Error> {
        Self::check_owner(state, holder)?;
        let entry = {
            let mut entries = self
                .entries
                .try_borrow_mut()
                .map_err(|_| state.owner.fail(Error::RegistryConflict))?;
            let MapEntry::Occupied(slot) = entries.entry(holder.index) else {
                return Err(Error::ExpiredIdentity);
            };
            if !Self::matches(slot.get(), holder) {
                return Err(Error::ExpiredIdentity);
            }
            slot.remove()
        };
        drop(entry);
        Ok(())
    }

    pub(super) fn reap(&self, state: &State) {
        let end = self.next.get();
        let mut last = 0;
        loop {
            let retired =
                {
                    let Ok(mut entries) = self.entries.try_borrow_mut() else {
                        state.owner.fail(Error::RegistryConflict);
                        return;
                    };
                    let Some(index) = entries.range((Excluded(last), Included(end))).find_map(
                        |(index, entry)| (entry.identity.strong_count() == 0).then_some(*index),
                    ) else {
                        return;
                    };
                    last = index;
                    entries.remove(&index)
                };
            // Backing can reenter the host. Release it outside the registry loan.
            drop(retired);
        }
    }
}

struct RealmEntry<'context> {
    context: &'context mut Context,
    previous: BoaRealm,
}

impl Drop for RealmEntry<'_> {
    fn drop(&mut self) {
        self.context.enter_realm(self.previous.clone());
    }
}

impl Entered<'_> {
    fn callback_creation_realm(
        &mut self,
        object: &JsObject,
    ) -> Result<(RealmKey, BoaRealm), Error> {
        let realm = object
            .creation_realm()
            .ok_or(Error::CallbackRealmUnavailable)?;
        Ok((self.state.realm_key(&realm)?, realm))
    }

    pub(super) fn observe_callback_object(&mut self, root: &Root) -> Result<RealmKey, Error> {
        self.check_active()?;
        let object = self.object_value(root)?;
        self.callback_creation_realm(&object).map(|(key, _)| key)
    }

    pub(super) fn pin_callback_object(
        &mut self,
        root: &Root,
        expected: RealmKey,
        backing: Box<dyn Any>,
    ) -> Result<Rc<CallbackObjectIdentity>, Error> {
        self.check_active()?;
        let object = self.object_value(root)?;
        let (key, realm) = self.callback_creation_realm(&object)?;
        if key != expected {
            return Err(Error::CallbackRealmUnavailable);
        }
        self.state
            .callbacks
            .retain(&self.state, object, realm, backing)
    }

    pub(super) fn enter_callback_object<T>(
        &mut self,
        root: &Root,
        expected: RealmKey,
        action: impl FnOnce(&mut Entered<'_>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.check_active()?;
        let object = self.object_value(root)?;
        let (key, realm) = self.callback_creation_realm(&object)?;
        if key != expected {
            return Err(Error::CallbackRealmUnavailable);
        }
        self.in_boa_realm(key, realm, action)
    }

    pub(super) fn in_boa_realm<T>(
        &mut self,
        key: RealmKey,
        realm: BoaRealm,
        action: impl FnOnce(&mut Entered<'_>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let previous = self.context.enter_realm(realm);
        let entry = RealmEntry {
            context: &mut *self.context,
            previous,
        };
        let mut entered = Entered {
            context: &mut *entry.context,
            state: self.state.clone(),
            realm: key,
        };
        action(&mut entered)
    }
}
