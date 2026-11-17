//! Scoped observations at known DOM-owned Arc<Vec<T>> allocation sites.
//! Opaque Arc control blocks and allocator metadata are not measured.
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    PiAttributes,
    StylePolicies,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Counts {
    pub targets: usize,
    pub target_bytes: usize,
    pub capacity_bytes: usize,
}
thread_local! { static ACTIVE: RefCell<Option<(Kind, Counts)>> = const { RefCell::new(None) }; }

pub(crate) fn allocated<T>(kind: Kind, capacity: usize) {
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let Some((selected, counts)) = active.as_mut() else {
            return;
        };
        if *selected != kind {
            return;
        }
        counts.targets = counts.targets.checked_add(1).expect("known target count");
        counts.target_bytes = counts
            .target_bytes
            .checked_add(size_of::<Vec<T>>())
            .expect("known target body");
        counts.capacity_bytes = counts
            .capacity_bytes
            .checked_add(
                capacity
                    .checked_mul(size_of::<T>())
                    .expect("actual buffer extent"),
            )
            .expect("actual buffer extents");
    });
}
pub(crate) struct Observation {
    live: bool,
    _thread: PhantomData<Rc<()>>,
}
impl Observation {
    pub fn begin(kind: Kind) -> Self {
        super::scale_metrics::require_enabled().expect("select the counter-enabled build");
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            assert!(active.is_none(), "one known-target observation per thread");
            *active = Some((kind, Counts::default()));
        });
        Self {
            live: true,
            _thread: PhantomData,
        }
    }
    pub fn finish(mut self) -> Counts {
        self.live = false;
        ACTIVE.with(|active| active.borrow_mut().take().expect("active observation").1)
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        if self.live {
            ACTIVE.with(|active| {
                active.borrow_mut().take();
            });
        }
    }
}
