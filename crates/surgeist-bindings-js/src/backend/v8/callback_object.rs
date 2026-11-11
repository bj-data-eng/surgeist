use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use crate::callback_object::CallbackObjectIdentity;
use crate::{Error, RealmKey, Root};

use super::{Entered, Operations, Roots};

struct Entry {
    identity: Weak<CallbackObjectIdentity>,
    object: v8::Global<v8::Object>,
    _context: v8::Global<v8::Context>,
    // Both engine handles are destroyed before the native backing is released.
    _backing: Box<dyn Any>,
}

#[derive(Default)]
pub(super) struct CallbackObjects {
    next: Cell<u64>,
    entries: RefCell<BTreeMap<u64, Rc<Entry>>>,
}

impl CallbackObjects {
    pub(super) const fn storage_bytes() -> usize {
        size_of::<CallbackObjectIdentity>() + size_of::<Entry>() + size_of::<(u64, Rc<Entry>)>()
    }

    pub(super) fn release(
        &self,
        roots: &Roots,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<(), Error> {
        if holder.owner != roots.owner.id() {
            return Err(Error::ForeignOwner);
        }
        let entry = {
            let mut entries = self
                .entries
                .try_borrow_mut()
                .map_err(|_| roots.conflict())?;
            if !entries
                .get(&holder.index)
                .is_some_and(|entry| entry.identity.ptr_eq(&Rc::downgrade(holder)))
            {
                return Err(Error::ExpiredIdentity);
            }
            entries.remove(&holder.index)
        };
        drop(entry);
        Ok(())
    }

    pub(super) fn reap(&self, roots: &Roots) {
        let retired = match self.entries.try_borrow_mut() {
            Ok(mut entries) => {
                let dead: Vec<_> = entries
                    .iter()
                    .filter_map(|(index, entry)| {
                        (entry.identity.strong_count() == 0).then_some(*index)
                    })
                    .collect();
                dead.into_iter()
                    .filter_map(|index| entries.remove(&index))
                    .collect::<Vec<_>>()
            }
            Err(_) => {
                roots.conflict();
                return;
            }
        };
        drop(retired);
    }
}

fn creation_context<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    roots: &Roots,
    object: v8::Local<'s, v8::Object>,
) -> Result<(v8::Local<'s, v8::Context>, RealmKey), Error> {
    if object.is_proxy() {
        return Err(Error::CallbackRealmUnavailable);
    }
    let context = object
        .get_creation_context(scope)
        .ok_or(Error::CallbackRealmUnavailable)?;
    let key = context
        .get_slot::<RealmKey>()
        .ok_or(Error::CallbackRealmUnavailable)?;
    if key.owner != roots.owner.id() {
        return Err(Error::ForeignOwner);
    }
    Ok((context, *key))
}

impl Entered<'_, '_, '_> {
    pub(super) fn observe_callback_object(&mut self, root: &Root) -> Result<RealmKey, Error> {
        self.check_active()?;
        let object = v8::Local::<v8::Object>::try_from(self.roots.local(self.scope, root)?)
            .map_err(|_| Error::ExpectedObject)?;
        creation_context(self.scope, &self.roots, object).map(|(_, realm)| realm)
    }

    pub(super) fn pin_callback_object(
        &mut self,
        root: &Root,
        expected_realm: RealmKey,
        backing: Box<dyn Any>,
    ) -> Result<Rc<CallbackObjectIdentity>, Error> {
        self.check_active()?;
        let roots = &self.roots;
        let object = v8::Local::<v8::Object>::try_from(roots.local(self.scope, root)?)
            .map_err(|_| Error::ExpectedObject)?;
        let (context, realm) = creation_context(self.scope, roots, object)?;
        if realm != expected_realm {
            return Err(Error::CallbackRealmUnavailable);
        }
        let bank = &roots.callback_objects;
        let index = bank
            .next
            .get()
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        bank.next.set(index);
        let identity = Rc::new(CallbackObjectIdentity {
            owner: realm.owner,
            index,
        });
        let entry = Rc::new(Entry {
            identity: Rc::downgrade(&identity),
            object: v8::Global::new(self.scope, object),
            _context: v8::Global::new(self.scope, context),
            _backing: backing,
        });
        bank.entries
            .try_borrow_mut()
            .map_err(|_| roots.conflict())?
            .insert(index, entry);
        Ok(identity)
    }

    pub(super) fn open_pinned_callback_object(
        &mut self,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<Root, Error> {
        self.check_active()?;
        let roots = &self.roots;
        if holder.owner != roots.owner.id() {
            return Err(Error::ForeignOwner);
        }
        let entry = roots
            .callback_objects
            .entries
            .try_borrow()
            .map_err(|_| roots.conflict())?
            .get(&holder.index)
            .filter(|entry| entry.identity.ptr_eq(&Rc::downgrade(holder)))
            .cloned()
            .ok_or(Error::ExpiredIdentity)?;
        let object = v8::Local::new(self.scope, &entry.object);
        roots.retain(self.scope, self.realm, object.into())
    }
}
