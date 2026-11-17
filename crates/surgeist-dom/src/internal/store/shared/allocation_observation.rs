//! Scope-local observations of actual Arc<Vec<T>> target construction. The
//! incoming Vec capacity is retained storage, not another allocation event.
use std::{cell::RefCell, marker::PhantomData, mem::size_of, rc::Rc};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Counts {
    pub headers: usize,
    pub zero_capacity_headers: usize,
    pub header_bytes: usize,
    pub retained_capacity_bytes: usize,
}
thread_local! {
    static ACTIVE: RefCell<Option<Counts>> = const { RefCell::new(None) };
}
pub(super) fn allocated<T>(capacity: usize) {
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let Some(counts) = active.as_mut() else {
            return;
        };
        counts.headers = counts
            .headers
            .checked_add(1)
            .expect("actual sequence target count");
        counts.zero_capacity_headers = counts
            .zero_capacity_headers
            .checked_add(usize::from(capacity == 0))
            .expect("actual empty target count");
        counts.header_bytes = counts
            .header_bytes
            .checked_add(size_of::<Vec<T>>())
            .expect("actual sequence header extent");
        counts.retained_capacity_bytes = counts
            .retained_capacity_bytes
            .checked_add(
                capacity
                    .checked_mul(size_of::<T>())
                    .expect("actual retained sequence capacity"),
            )
            .expect("actual retained sequence capacities");
    });
}
pub(super) struct Observation {
    live: bool,
    _thread: PhantomData<Rc<()>>,
}
impl Observation {
    pub fn begin() -> Self {
        super::super::scale_metrics::require_enabled()
            .expect("select the counter-enabled test build");
        ACTIVE.with(|active| {
            assert!(
                active.borrow().is_none(),
                "one sequence allocation observation per thread"
            );
            *active.borrow_mut() = Some(Counts::default());
        });
        Self {
            live: true,
            _thread: PhantomData,
        }
    }
    pub fn counts(&self) -> Counts {
        ACTIVE.with(|active| {
            active
                .borrow()
                .expect("active sequence allocation observation")
        })
    }
    pub fn finish(mut self) -> Counts {
        self.live = false;
        ACTIVE.with(|active| {
            active
                .borrow_mut()
                .take()
                .expect("active sequence allocation observation")
        })
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
