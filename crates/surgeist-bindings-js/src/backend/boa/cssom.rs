use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

use boa_engine::{JsData, JsObject, JsValue, object::WeakJsObject};
use boa_gc::{Finalize, GcRefCell, Trace};

use crate::{Error, RealmKey};

use super::{super::Projection, State};

#[derive(Default, Trace, Finalize, JsData)]
pub(super) struct ProjectableObject {
    // The links follow guest reachability and must be traced rather than
    // independently rooted by the host registry.
    pub(super) links: GcRefCell<Vec<JsValue>>,
    pub(super) id: GcRefCell<Option<u32>>,
}

#[derive(Trace, Finalize, JsData)]
pub(super) struct CssomPrototypes(pub(super) Vec<JsObject>);

#[derive(Clone)]
pub(super) struct Projected {
    pub(super) realm: RealmKey,
    pub(super) object: WeakJsObject,
    pub(super) payload: Rc<dyn Any>,
}

#[derive(Default)]
pub(super) struct Projections {
    pub(super) next: Cell<u32>,
    pub(super) entries: RefCell<BTreeMap<u32, Projected>>,
}

impl Projections {
    pub(super) fn storage_bytes() -> usize {
        size_of::<(u32, Projected)>() + size_of::<[JsValue; 2]>()
    }

    pub(super) fn live(
        &self,
        state: &State,
        identity: Projection,
    ) -> Result<Option<JsObject>, Error> {
        if identity.realm.owner != state.owner.id() {
            return Err(Error::ForeignOwner);
        }
        let entries = self.entries.try_borrow().map_err(|_| state.conflict())?;
        let Some(entry) = entries.get(&identity.id) else {
            return Ok(None);
        };
        if identity.realm != entry.realm {
            return Err(state.conflict());
        }
        Ok(entry.object.upgrade())
    }

    pub(super) fn reap(&self, state: &State) {
        loop {
            let removed = {
                let Ok(mut entries) = self.entries.try_borrow_mut() else {
                    state.conflict();
                    return;
                };
                let expired = entries
                    .iter()
                    .find_map(|(&id, entry)| entry.object.upgrade().is_none().then_some(id));
                expired.and_then(|id| entries.remove(&id))
            };
            if removed.is_none() {
                break;
            }
            drop(removed);
        }
    }
}
