//! Current-thread controls for measuring the optional DOM sharing layers.
//!
//! Every measured mode uses the same guard. It changes no production layout,
//! owns no pool authority, and never suppresses mandatory input admission. Keep
//! all formation/edit calls on this thread, then drop the guard before dropping
//! the DOM, reports or snapshots during reclamation measurements.
use super::{Owner, Session, State};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::store) enum Layers {
    /// Preserve caller-supplied backings and mandatory atoms, without pooling.
    CallerOnly,
    BanksOnly,
    Combined,
}

struct Control {
    owner: Weak<Mutex<State>>,
    layers: Layers,
}

thread_local! {
    static CONTROL: RefCell<Option<Control>> = const { RefCell::new(None) };
}

/// One non-nestable scope on the calling thread. Other owners remain combined.
#[must_use = "keep the fixture control alive through construction and edits"]
pub(in crate::store) struct Guard {
    // Moving this guard would leave the originating thread's control installed.
    _thread: PhantomData<Rc<()>>,
}

/// Select a layer configuration for this owner on the calling thread only.
///
/// Install before fixture construction and keep the choice unchanged through
/// its last edit/publication. A guard does not retain the owner strongly. Its
/// diagnostic Weak must be gone before owner-loss measurements begin.
pub(in crate::store) fn control(owner: &Owner, layers: Layers) -> Guard {
    CONTROL.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(slot.is_none(), "one sharing fixture control per thread");
        *slot = Some(Control {
            owner: Arc::downgrade(&owner.0),
            layers,
        });
    });
    Guard {
        _thread: PhantomData,
    }
}

pub(super) fn layers(session: &Session) -> Layers {
    CONTROL.with(|slot| {
        slot.borrow()
            .as_ref()
            .filter(|control| Weak::ptr_eq(&control.owner, &session.owner))
            .map_or(Layers::Combined, |control| control.layers)
    })
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Release the TLS borrow before dropping even the diagnostic Weak.
        let control = CONTROL.with(|slot| slot.borrow_mut().take());
        drop(control.expect("the scoped sharing fixture control is installed"));
    }
}
