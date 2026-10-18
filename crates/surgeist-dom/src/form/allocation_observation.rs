//! Thread-local observation immediately after the actual fresh history target
//! is allocated. No address or record is retained by this observer.
use super::FormPastName;
use std::{cell::Cell, marker::PhantomData, rc::Rc, sync::Arc};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Counts {
    pub targets: usize,
    pub target_bytes: usize,
    pub capacity_bytes: usize,
}
thread_local! {
    static ACTIVE: Cell<Option<Counts>> = const { Cell::new(None) };
}

pub(super) fn constructed<K>(value: &Arc<Vec<FormPastName<K>>>) {
    ACTIVE.with(|active| {
        if let Some(mut counts) = active.get() {
            counts.targets = counts.targets.checked_add(1).expect("actual Form targets");
            counts.target_bytes = counts
                .target_bytes
                .checked_add(size_of::<Vec<FormPastName<K>>>())
                .expect("actual Form target bytes");
            counts.capacity_bytes = value
                .capacity()
                .checked_mul(size_of::<FormPastName<K>>())
                .and_then(|bytes| counts.capacity_bytes.checked_add(bytes))
                .expect("actual Form capacity bytes");
            active.set(Some(counts));
        }
    });
}

pub(crate) struct Observation {
    live: bool,
    _thread: PhantomData<Rc<()>>,
}
impl Observation {
    pub(crate) fn begin() -> Self {
        crate::store::scale_metrics::require_enabled()
            .expect("select the counter-enabled test build");
        ACTIVE.with(|active| {
            assert!(
                active.get().is_none(),
                "one Form allocation observer per thread"
            );
            active.set(Some(Counts::default()));
        });
        Self {
            live: true,
            _thread: PhantomData,
        }
    }
    pub(crate) fn counts(&self) -> Counts {
        ACTIVE.with(|active| active.get().expect("active Form observer"))
    }
    pub(crate) fn finish(mut self) -> Counts {
        self.live = false;
        ACTIVE.with(|active| active.take().expect("active Form observer"))
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        if self.live {
            ACTIVE.with(|active| active.set(None));
        }
    }
}
