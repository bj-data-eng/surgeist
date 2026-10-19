//! Public consumer regressions for root authority and bounded rollback.
//! The driver sees only exported actions, layouts, progress and final results.
use std::{cell::Cell, mem::size_of, rc::Rc, sync::Arc};
use surgeist_html::bounded::{
    HtmlAttributePool, HtmlAttributePoolOperation, HtmlAttributePoolResult, HtmlStringPool,
    HtmlStringPoolOperation, HtmlStringPoolResult, HtmlValuePoolActionKind, HtmlValuePoolError,
    HtmlValuePoolMark, HtmlValuePoolProgress, HtmlValuePoolRoot, HtmlValuePoolSeal,
    HtmlValuePoolUsage, HtmlValuePoolWork,
};
use surgeist_html::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Storage {
    bytes: usize,
    units: usize,
}
impl Storage {
    fn add(self, other: Self) -> Self {
        Self {
            bytes: self.bytes.checked_add(other.bytes).unwrap(),
            units: self.units.checked_add(other.units).unwrap(),
        }
    }
    fn sub(self, other: Self) -> Self {
        Self {
            bytes: self.bytes.checked_sub(other.bytes).unwrap(),
            units: self.units.checked_sub(other.units).unwrap(),
        }
    }
}
impl From<HtmlValuePoolUsage> for Storage {
    fn from(value: HtmlValuePoolUsage) -> Self {
        Self {
            bytes: value.bytes(),
            units: value.utf16_units(),
        }
    }
}
fn string_storage(value: &DocumentString) -> Storage {
    let capacity = value.shared_code_units().capacity();
    Storage {
        bytes: size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
        units: capacity,
    }
}
fn attribute_storage(value: &OrderedAttributes) -> Storage {
    let mut storage = Storage {
        bytes: size_of::<Vec<Attribute>>()
            + value.shared_attributes().capacity() * size_of::<Attribute>(),
        units: 0,
    };
    for attribute in value.iter() {
        for string in [
            attribute.name().namespace().uri(),
            attribute.name().prefix(),
            Some(attribute.name().local_name()),
            Some(attribute.value()),
        ]
        .into_iter()
        .flatten()
        {
            storage = storage.add(string_storage(string));
        }
    }
    storage
}

#[derive(Debug)]
struct Ledger {
    live: Cell<Storage>,
    spent: Cell<usize>,
    actions: Cell<usize>,
    reads: Cell<usize>,
}
#[derive(Clone, Debug)]
struct Meter(Rc<Ledger>);
impl Meter {
    fn new() -> Self {
        Self(Rc::new(Ledger {
            live: Cell::new(Storage::default()),
            spent: Cell::new(0),
            actions: Cell::new(0),
            reads: Cell::new(0),
        }))
    }
    fn pay(&self, work: HtmlValuePoolWork) {
        let amount = [
            work.record_visits(),
            work.inspected_bytes(),
            work.copied_bytes(),
            work.provider_calls(),
            work.reserve_bytes(),
        ]
        .into_iter()
        .try_fold(0_usize, usize::checked_add)
        .unwrap();
        let next = self.0.spent.get().checked_add(amount).unwrap();
        assert!(
            next <= 16_000_000,
            "public driver exhausted its finite work allowance"
        );
        self.0.spent.set(next);
    }
    fn reserve(&self, amount: Storage) -> Receipt {
        let mut receipt = Receipt {
            amount: Storage::default(),
            meter: self.clone(),
        };
        receipt.resize(amount);
        receipt
    }
    fn hold<T>(&self, value: T, amount: Storage) -> Held<T> {
        Held {
            value,
            _receipt: self.reserve(amount),
        }
    }
}
#[derive(Debug)]
struct Receipt {
    amount: Storage,
    meter: Meter,
}
impl Receipt {
    fn resize(&mut self, amount: Storage) {
        let next = self.meter.0.live.get().sub(self.amount).add(amount);
        assert!(
            next.bytes <= 16_000_000 && next.units <= 1_000_000,
            "public driver exhausted its finite storage allowance: {next:?}"
        );
        self.meter.0.live.set(next);
        self.amount = amount;
    }
    fn shrink_after_drop(&mut self, amount: Storage) {
        assert!(amount.bytes <= self.amount.bytes && amount.units <= self.amount.units);
        self.resize(amount);
    }
}
impl Drop for Receipt {
    fn drop(&mut self) {
        self.meter
            .0
            .live
            .set(self.meter.0.live.get().sub(self.amount));
    }
}
#[derive(Debug)]
struct Held<T> {
    value: T, // Data is destroyed before its conservative occurrence receipt.
    _receipt: Receipt,
}
enum Done<T> {
    Hit(Held<T>),
    Miss,
    Offered,
    Bypassed,
}
impl<T> Done<T> {
    fn label(&self) -> &'static str {
        match self {
            Self::Hit(_) => "Hit",
            Self::Miss => "Miss",
            Self::Offered => "Offered",
            Self::Bypassed => "Bypassed",
        }
    }
}

macro_rules! consumer {
    ($owner:ident, $pool:ty, $operation:ident, $result:ident, $value:ty, $storage:ident) => {
        struct $owner {
            pool: $pool,
            metadata: Receipt,
            meter: Meter,
        }
        impl $owner {
            fn new(meter: &Meter) -> Self {
                meter.pay(<$pool>::disposal_work()); // Before owner existence.
                let metadata = meter.reserve(Storage {
                    bytes: <$pool>::inline_bytes(),
                    units: 0,
                });
                Self {
                    pool: <$pool>::new(),
                    metadata,
                    meter: meter.clone(),
                }
            }
            fn storage(&self) -> Storage {
                self.pool.heap_usage().into()
            }
            fn settle(&mut self) {
                self.metadata.shrink_after_drop(
                    Storage {
                        bytes: <$pool>::inline_bytes(),
                        units: 0,
                    }
                    .add(self.storage()),
                );
            }
            fn begin(&mut self) -> HtmlValuePoolRoot {
                self.meter.pay(<$pool>::root_work());
                self.pool
                    .begin_root()
                    .expect("available root authority in this fixture")
            }
            fn mark(&mut self, root: &HtmlValuePoolRoot) -> Option<HtmlValuePoolMark> {
                self.meter.pay(<$pool>::mark_work());
                self.pool.mark(root).unwrap()
            }
            fn accept(&mut self, mark: HtmlValuePoolMark) {
                self.pool.accept_mark(mark).unwrap();
                self.settle();
            }
            fn restore(&mut self, mark: HtmlValuePoolMark) {
                let spent = self.meter.0.spent.get();
                let change = self.pool.restore_mark(mark).unwrap();
                assert_eq!(Storage::from(change.after()), self.storage());
                self.settle();
                assert_eq!(
                    spent,
                    self.meter.0.spent.get(),
                    "restore cannot refund or newly charge work"
                );
            }
            fn prepare(
                &mut self,
                root: HtmlValuePoolRoot,
            ) -> Result<HtmlValuePoolSeal, (HtmlValuePoolError, HtmlValuePoolRoot)> {
                self.meter.pay(<$pool>::seal_work());
                self.pool.prepare_seal(root)
            }
            fn seal(&mut self, seal: HtmlValuePoolSeal) {
                self.pool.validate_seal(&seal).unwrap();
                self.pool.seal(seal).unwrap();
                self.settle();
            }
            fn abort(&mut self, root: HtmlValuePoolRoot) {
                self.pool.abort_root(root).unwrap();
                self.settle();
            }
            fn abort_seal(&mut self, seal: HtmlValuePoolSeal) {
                self.pool.abort_seal(seal).unwrap();
                self.settle();
            }
            fn run(&mut self, input: &$value, offer: bool) -> Done<$value> {
                // Declare custody first so unwind drops the operation's actual
                // data before releasing its candidate reservation.
                let mut candidate = self.meter.reserve(Storage::default());
                let mut operation = Some(if offer {
                    self.pool.offer(input)
                } else {
                    self.pool.probe(input)
                });
                while let Some(action) = self.pool.next_action(operation.as_mut().unwrap()).unwrap()
                {
                    let layout = action.layout();
                    self.meter.pay(layout.work());
                    self.meter.pay(layout.closing_work());
                    self.metadata.resize(
                        Storage {
                            bytes: <$pool>::inline_bytes(),
                            units: 0,
                        }
                        .add(layout.pool().into()),
                    );
                    candidate.resize(layout.candidate().into()); // BEFORE Weak upgrade.
                    self.meter.0.actions.set(self.meter.0.actions.get() + 1);
                    let progress = match action.kind() {
                        HtmlValuePoolActionKind::Read => {
                            self.meter.0.reads.set(self.meter.0.reads.get() + 1);
                            operation.as_mut().unwrap().read(action).unwrap()
                        }
                        HtmlValuePoolActionKind::Pool => self
                            .pool
                            .perform(operation.as_mut().unwrap(), action)
                            .unwrap(),
                    };
                    if matches!(
                        progress,
                        HtmlValuePoolProgress::CandidateReleased | HtmlValuePoolProgress::Finished
                    ) {
                        candidate.shrink_after_drop(Storage::default());
                    }
                }
                let result = $operation::try_finish(&mut operation).unwrap();
                self.settle();
                match result {
                    $result::Hit(value) => {
                        assert_eq!(candidate.amount, $storage(value.value()));
                        Done::Hit(Held {
                            value: value.into_value(),
                            _receipt: candidate,
                        })
                    }
                    $result::Miss => Done::Miss,
                    $result::Offered(change) => {
                        assert_eq!(Storage::from(change.after()), self.storage());
                        Done::Offered
                    }
                    $result::Bypassed => Done::Bypassed,
                }
            }
        }
    };
}
consumer!(
    Strings,
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolResult,
    DocumentString,
    string_storage
);
consumer!(
    Attributes,
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolResult,
    OrderedAttributes,
    attribute_storage
);

fn string(meter: &Meter, id: u16) -> Held<DocumentString> {
    let value = DocumentString::from_utf16(vec![0xd800, 0, id]);
    let storage = string_storage(&value);
    meter.hold(value, storage)
}
fn attributes(meter: &Meter, id: u16) -> Held<OrderedAttributes> {
    let value = OrderedAttributes::new(vec![Attribute::new(
        QualifiedName::new(Namespace::none(), None, DocumentString::from("cell")),
        DocumentString::from_utf16(vec![0xd800, 0, id]),
    )])
    .unwrap();
    let storage = attribute_storage(&value);
    meter.hold(value, storage)
}

macro_rules! driver_sanity {
    ($test:ident, $owner:ident, $pool:ty, $input:ident, $storage:ident, $shared:ident) => {
        #[test]
        fn $test() {
            let meter = Meter::new();
            {
                let mut owner = $owner::new(&meter);
                let empty = owner.storage();
                let original = $input(&meter, 41);
                let independent = $input(&meter, 41);
                let distinct = $input(&meter, 42);
                assert!(!Arc::ptr_eq(
                    original.value.$shared(),
                    independent.value.$shared()
                ));
                let root = owner.begin();
                assert_eq!(owner.run(&original.value, true).label(), "Offered");
                let Done::Hit(output) = owner.run(&independent.value, false) else {
                    panic!("independently allocated equal nonempty value must hit");
                };
                assert!(Arc::ptr_eq(
                    output.value.$shared(),
                    original.value.$shared()
                ));
                assert_eq!(output.value, original.value);
                // Three caller occurrences and the separately preadmitted
                // candidate/output occurrence, plus the actual owner extent.
                let expected = Storage {
                    bytes: <$pool>::inline_bytes(),
                    units: 0,
                }
                .add(owner.storage())
                .add($storage(&original.value))
                .add($storage(&independent.value))
                .add($storage(&distinct.value))
                .add($storage(&output.value));
                assert_eq!(meter.0.live.get(), expected);
                assert_eq!(owner.run(&distinct.value, false).label(), "Miss");
                assert_eq!(
                    meter.0.live.get(),
                    expected,
                    "a miss releases provisional candidate custody"
                );
                owner.abort(root);
                assert_eq!(owner.storage(), empty);
                drop(owner);
                assert_eq!(
                    output.value, original.value,
                    "held output survives owner disposal"
                );
                assert!(Arc::ptr_eq(
                    output.value.$shared(),
                    original.value.$shared()
                ));
            }
            assert_eq!(meter.0.live.get(), Storage::default());
            assert!(meter.0.actions.get() > 0 && meter.0.reads.get() > 0);
        }
    };
}
driver_sanity!(
    public_string_driver_transfers_hit_custody_and_releases_all_receipts,
    Strings,
    HtmlStringPool,
    string,
    string_storage,
    shared_code_units
);
driver_sanity!(
    public_attribute_driver_transfers_hit_custody_and_releases_all_receipts,
    Attributes,
    HtmlAttributePool,
    attributes,
    attribute_storage,
    shared_attributes
);

macro_rules! seal_case {
    ($test:ident, $owner:ident, $input:ident, $shared:ident) => {
        #[test]
        fn $test() {
            let meter = Meter::new();
            let mut owner = $owner::new(&meter);
            let original = $input(&meter, 1);
            let initial = owner.begin();
            assert_eq!(owner.run(&original.value, true).label(), "Offered");
            let seal = owner.prepare(initial).unwrap();
            owner.seal(seal);
            let baseline = owner.storage();
            let root = owner.begin();
            let outer = owner.mark(&root).unwrap();
            let stale_restore = owner.mark(&root).unwrap();
            let stale_accept = owner.mark(&root).unwrap();
            let transient = $input(&meter, 2);
            assert_eq!(owner.run(&transient.value, true).label(), "Offered");
            let before_refusal = owner.storage();
            let root = match owner.prepare(root) {
                Err((error, root)) => {
                    assert_eq!(error, HtmlValuePoolError::NotReady);
                    root
                }
                Ok(seal) => {
                    owner.abort_seal(seal);
                    panic!("prepare_seal froze a root with unresolved nested marks");
                }
            };
            assert_eq!(owner.storage(), before_refusal, "refusal preserves state");
            let still_usable = owner
                .mark(&root)
                .expect("refusal preserves unfrozen root authority");
            owner.accept(still_usable);
            owner.restore(outer);
            assert_eq!(owner.storage(), baseline);
            let frozen = owner.prepare(root).unwrap();
            owner.restore(stale_restore);
            owner.accept(stale_accept);
            let other = $input(&meter, 3);
            assert_eq!(owner.run(&other.value, true).label(), "Bypassed");
            assert_eq!(owner.storage(), baseline);
            let Done::Hit(hit) = owner.run(&original.value, false) else {
                panic!("frozen read lost original candidate")
            };
            assert!(Arc::ptr_eq(hit.value.$shared(), original.value.$shared()));
            assert_eq!(owner.run(&transient.value, false).label(), "Miss");
            owner.pool.validate_seal(&frozen).unwrap();
            owner.abort_seal(frozen);
            assert_eq!(owner.storage(), baseline);
            let fresh = owner.begin();
            owner.abort(fresh);
            assert!(meter.0.actions.get() > 0 && meter.0.reads.get() > 0);
        }
    };
}
seal_case!(
    string_seal_refuses_marks_and_preserves_rollback_then_frozen_state,
    Strings,
    string,
    shared_code_units
);
seal_case!(
    attribute_seal_refuses_marks_and_preserves_rollback_then_frozen_state,
    Attributes,
    attributes,
    shared_attributes
);

#[test]
fn paired_seal_refusal_preserves_both_rollback_routes() {
    let meter = Meter::new();
    let mut strings = Strings::new(&meter);
    let mut banks = Attributes::new(&meter);
    let string_base = strings.storage();
    let bank_base = banks.storage();
    let string_root = strings.begin();
    let bank_root = banks.begin();
    let mark = banks.mark(&bank_root).unwrap();
    let text = string(&meter, 7);
    let bank = attributes(&meter, 7);
    assert_eq!(strings.run(&text.value, true).label(), "Offered");
    assert_eq!(banks.run(&bank.value, true).label(), "Offered");
    let first = strings.prepare(string_root).unwrap();
    let bank_root = match banks.prepare(bank_root) {
        Err((error, root)) => {
            assert_eq!(error, HtmlValuePoolError::NotReady);
            root
        }
        Ok(second) => {
            banks.abort_seal(second);
            strings.abort_seal(first);
            panic!("second pool prepared despite its outstanding mark");
        }
    };
    strings.abort_seal(first); // Never seal either half after paired refusal.
    assert_eq!(strings.storage(), string_base);
    banks.restore(mark);
    assert_eq!(banks.storage(), bank_base);
    let usable = banks.mark(&bank_root).unwrap();
    banks.accept(usable);
    banks.abort(bank_root);
    assert_eq!(strings.run(&text.value, false).label(), "Miss");
    assert_eq!(banks.run(&bank.value, false).label(), "Miss");
}

macro_rules! long_root_case {
    ($test:ident, $owner:ident, $input:ident, $value:ty, $shared:ident) => {
        #[test]
        fn $test() {
            fn membership(owner: &mut $owner, values: &[Held<$value>]) -> Vec<bool> {
                values.iter().map(|input| match owner.run(&input.value, false) {
                    Done::Hit(output) => {
                        assert!(Arc::ptr_eq(output.value.$shared(), input.value.$shared()));
                        true
                    }
                    Done::Miss => false,
                    other => panic!("eligible read returned {}", other.label()),
                }).collect()
            }
            fn replay(owner: &mut $owner, baseline: &[Held<$value>], incoming: &[Held<$value>]) -> Vec<Vec<bool>> {
                let root = owner.begin();
                let mut trace = Vec::new();
                for (n, input) in incoming.iter().enumerate() {
                    assert_eq!(owner.run(&input.value, true).label(), "Offered");
                    let mut step = membership(owner, baseline);
                    step.extend(membership(owner, &incoming[..=n]));
                    trace.push(step);
                }
                owner.abort(root);
                trace
            }
            let meter = Meter::new();
            {
                let mut owner = $owner::new(&meter);
                // Ordinary caller workload sizes, not a private capacity query.
                // The one seed publication establishes an observable baseline;
                // no row/offer in the tested root is independently sealed.
                let baseline: Vec<_> = (0..100).map(|n| $input(&meter, n)).collect();
                let seed = owner.begin();
                for input in &baseline {
                    assert_eq!(owner.run(&input.value, true).label(), "Offered");
                }
                let seed = owner.prepare(seed).unwrap();
                owner.seal(seed);
                let initial_usage = owner.storage();
                let initial_members = membership(&mut owner, &baseline);
                assert!(initial_members.iter().any(|live| *live));

                let root = owner.begin();
                let accepted = owner.mark(&root).unwrap();
                let first: Vec<_> = (1000..1257).map(|n| $input(&meter, n)).collect();
                for (n, input) in first.iter().enumerate() {
                    let outcome = owner.run(&input.value, true);
                    assert_eq!(outcome.label(), "Offered", "single root refused fresh eligible insertion {n}");
                }
                owner.accept(accepted);
                let nested_usage = owner.storage();
                let nested_base = membership(&mut owner, &baseline);
                let nested_first = membership(&mut owner, &first);
                let restored = owner.mark(&root).unwrap();
                let second: Vec<_> = (2000..2256).map(|n| $input(&meter, n)).collect();
                for (n, input) in second.iter().enumerate() {
                    let outcome = owner.run(&input.value, true);
                    assert_eq!(outcome.label(), "Offered", "single root refused later insertion {}", n + first.len());
                }
                // New useful patterns arrive only after 513 successful changes
                // in this same root. Held outputs keep their actual backing live.
                let late: Vec<_> = (60000..60003).map(|n| $input(&meter, n)).collect();
                let mut held_outputs = Vec::new();
                for (late_n, input) in late.iter().enumerate() {
                    assert_eq!(owner.run(&input.value, true).label(), "Offered");
                    for _ in 0..3 {
                        let independent = $input(&meter, 60000 + u16::try_from(late_n).unwrap());
                        let Done::Hit(output) = owner.run(&independent.value, false) else {
                            panic!("late admitted live pattern was not reusable");
                        };
                        assert!(Arc::ptr_eq(output.value.$shared(), input.value.$shared()));
                        held_outputs.push(output);
                    }
                }
                owner.restore(restored);
                assert_eq!(owner.storage(), nested_usage);
                assert_eq!(membership(&mut owner, &baseline), nested_base);
                assert_eq!(membership(&mut owner, &first), nested_first);
                assert!(membership(&mut owner, &second).iter().all(|live| !*live));
                assert!(membership(&mut owner, &late).iter().all(|live| !*live));
                owner.abort(root);
                assert_eq!(owner.storage(), initial_usage);
                assert_eq!(membership(&mut owner, &baseline), initial_members);
                assert!(membership(&mut owner, &first).iter().all(|live| !*live));

                // Restoring a cursor is observable through a deterministic
                // eviction replay from the same retained membership/backings.
                let replay_inputs: Vec<_> = (5000..5024).map(|n| $input(&meter, n)).collect();
                let first_trace = replay(&mut owner, &baseline, &replay_inputs);
                assert_eq!(owner.storage(), initial_usage);
                assert_eq!(membership(&mut owner, &baseline), initial_members);
                let second_trace = replay(&mut owner, &baseline, &replay_inputs);
                assert_eq!(first_trace, second_trace, "abort must restore eviction behavior as well as membership");
                assert_eq!(owner.storage(), initial_usage);
                assert_eq!(membership(&mut owner, &baseline), initial_members);
                assert_eq!(held_outputs.len(), 9);
                for (n, output) in held_outputs.iter().enumerate() {
                    assert!(Arc::ptr_eq(output.value.$shared(), late[n / 3].value.$shared()));
                }
                eprintln!("{}: successful_changes=516, retained_hits={}, actions={}, reads={}, spent_work={}, pool_bytes={}",
                    stringify!($test), held_outputs.len(), meter.0.actions.get(), meter.0.reads.get(), meter.0.spent.get(), owner.storage().bytes);
            }
            assert_eq!(meter.0.live.get(), Storage::default(), "all actual inputs/output/pool receipts released");
        }
    };
}
long_root_case!(
    string_single_root_keeps_admitting_late_patterns_and_restores_accepted_and_nested_changes,
    Strings,
    string,
    DocumentString,
    shared_code_units
);
long_root_case!(
    attribute_single_root_keeps_admitting_late_patterns_and_restores_accepted_and_nested_changes,
    Attributes,
    attributes,
    OrderedAttributes,
    shared_attributes
);

macro_rules! public_guard_edges {
    ($test:ident, $owner:ident, $pool:ty, $operation:ident, $result:ident, $value:ty, $input:ident, $shared:ident) => {
        #[test]
        fn $test() {
            struct Session<'a> {
                owner: &'a mut $owner,
                root: Option<HtmlValuePoolRoot>,
            }
            impl Drop for Session<'_> {
                fn drop(&mut self) {
                    if let Some(root) = self.root.take() {
                        self.owner.abort(root);
                    }
                }
            }
            struct Checkpoint<'a, 'b> {
                session: &'a mut Session<'b>,
                mark: Option<HtmlValuePoolMark>,
            }
            impl Drop for Checkpoint<'_, '_> {
                fn drop(&mut self) {
                    if let Some(mark) = self.mark.take() {
                        self.session.owner.accept(mark);
                    }
                }
            }
            struct Prepared<'a> {
                owner: &'a mut $owner,
                seal: Option<HtmlValuePoolSeal>,
            }
            impl Drop for Prepared<'_> {
                fn drop(&mut self) {
                    if let Some(seal) = self.seal.take() {
                        self.owner.abort_seal(seal);
                    }
                }
            }
            let meter = Meter::new();
            {
                let mut owner = $owner::new(&meter);
                let original = $input(&meter, 70);
                let root = owner.begin();
                assert!(matches!(owner.run(&original.value, true), Done::Offered));
                let seal = owner.prepare(root).unwrap();
                owner.seal(seal);
                let baseline = owner.storage();

                // A consumer cancellation has no failure-time admission. The
                // nested guard accepts; the enclosing root guard still aborts.
                let transient = $input(&meter, 71);
                let cancelled: Result<(), &'static str> = {
                    let root = owner.begin();
                    let mut session = Session {
                        owner: &mut owner,
                        root: Some(root),
                    };
                    let mark = session.owner.mark(session.root.as_ref().unwrap()).unwrap();
                    let checkpoint = Checkpoint {
                        session: &mut session,
                        mark: Some(mark),
                    };
                    assert!(matches!(
                        checkpoint.session.owner.run(&transient.value, true),
                        Done::Offered
                    ));
                    Err("cancelled")
                };
                assert_eq!(cancelled, Err("cancelled"));
                assert_eq!(owner.storage(), baseline);
                assert!(matches!(owner.run(&transient.value, false), Done::Miss));
                assert!(matches!(owner.run(&original.value, false), Done::Hit(_)));

                // Panic after a real strong upgrade drops the operation before
                // its candidate receipt, then the session aborts. No private
                // phase or slot determines where the panic is injected.
                let equal = $input(&meter, 70);
                let before = meter.0.live.get();
                let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let root = owner.begin();
                    let session = Session {
                        owner: &mut owner,
                        root: Some(root),
                    };
                    let mut candidate = meter.reserve(Storage::default());
                    let mut operation = Some(session.owner.pool.probe(&equal.value));
                    while let Some(action) = session
                        .owner
                        .pool
                        .next_action(operation.as_mut().unwrap())
                        .unwrap()
                    {
                        let layout = action.layout();
                        meter.pay(layout.work());
                        meter.pay(layout.closing_work());
                        session.owner.metadata.resize(
                            Storage {
                                bytes: <$pool>::inline_bytes(),
                                units: 0,
                            }
                            .add(layout.pool().into()),
                        );
                        candidate.resize(layout.candidate().into());
                        let progress = match action.kind() {
                            HtmlValuePoolActionKind::Read => {
                                operation.as_mut().unwrap().read(action).unwrap()
                            }
                            HtmlValuePoolActionKind::Pool => session
                                .owner
                                .pool
                                .perform(operation.as_mut().unwrap(), action)
                                .unwrap(),
                        };
                        if progress == HtmlValuePoolProgress::CandidateAcquired {
                            assert!(candidate.amount.bytes > 0);
                            panic!("consumer panic after admitted strong candidate");
                        }
                    }
                    panic!("fixture failed to acquire a live candidate");
                }));
                let panic = panic.unwrap_err();
                assert_eq!(
                    panic.downcast_ref::<&str>(),
                    Some(&"consumer panic after admitted strong candidate")
                );
                assert_eq!(meter.0.live.get(), before);
                assert_eq!(owner.storage(), baseline);
                let root = owner.begin();
                let seal = owner.prepare(root).unwrap();
                drop(Prepared {
                    owner: &mut owner,
                    seal: Some(seal),
                });
                let root = owner.begin();
                owner.abort(root);

                // An expired candidate releases the pre-upgrade reservation;
                // its Weak target-space allowance remains until pool retirement.
                let expired = $input(&meter, 72);
                let root = owner.begin();
                assert!(matches!(owner.run(&expired.value, true), Done::Offered));
                let seal = owner.prepare(root).unwrap();
                owner.seal(seal);
                let weak = Arc::downgrade(expired.value.$shared());
                drop(expired);
                assert!(weak.upgrade().is_none());
                let query = $input(&meter, 72);
                let before = meter.0.live.get();
                assert!(matches!(owner.run(&query.value, false), Done::Miss));
                assert_eq!(meter.0.live.get(), before);

                // A second operation supersedes an unfinished pool action.
                // The old action cannot mutate state or retain a candidate.
                let mut first = Some(owner.pool.probe(&query.value));
                let stale = loop {
                    let action = owner
                        .pool
                        .next_action(first.as_mut().unwrap())
                        .unwrap()
                        .unwrap();
                    meter.pay(action.layout().work());
                    meter.pay(action.layout().closing_work());
                    if action.kind() == HtmlValuePoolActionKind::Pool {
                        break action;
                    }
                    first.as_mut().unwrap().read(action).unwrap();
                };
                let mut second = Some(owner.pool.probe(&equal.value));
                let other = owner
                    .pool
                    .next_action(second.as_mut().unwrap())
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    first.as_mut().unwrap().read(other).unwrap_err(),
                    HtmlValuePoolError::WrongAction
                );
                let unchanged = owner.storage();
                assert_eq!(
                    owner.pool.perform(first.as_mut().unwrap(), stale).unwrap(),
                    HtmlValuePoolProgress::Finished
                );
                assert!(matches!(
                    $operation::try_finish(&mut first).unwrap(),
                    $result::Bypassed
                ));
                drop(second); // Early abandonment of read-only borrowed input.
                assert_eq!(owner.storage(), unchanged);
                owner.settle();
            }
            assert_eq!(meter.0.live.get(), Storage::default());
        }
    };
}
public_guard_edges!(
    public_string_guards_cleanup_cancel_panic_dead_upgrade_and_supersession,
    Strings,
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolResult,
    DocumentString,
    string,
    shared_code_units
);
public_guard_edges!(
    public_attribute_guards_cleanup_cancel_panic_dead_upgrade_and_supersession,
    Attributes,
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolResult,
    OrderedAttributes,
    attributes,
    shared_attributes
);

macro_rules! miss_promotion {
    ($test:ident, $owner:ident, $pool:ty, $operation:ident, $result:ident, $value:ty, $make:expr, $storage:ident, $shared:ident, $headers:expr, $units:expr) => {
        #[test]
        fn $test() {
            #[derive(Debug, Default, Eq, PartialEq)]
            struct ReadTrace {
                descriptors: usize,
                reads: usize,
                bytes: usize,
                pool_performs: usize,
            }
            fn drive(owner: &mut $owner, operation: &mut Option<$operation<'_>>, candidate: &mut Receipt) -> ReadTrace {
                let mut trace = ReadTrace::default();
                loop {
                    trace.descriptors += 1;
                    let Some(action) = owner.pool.next_action(operation.as_mut().unwrap()).unwrap() else { break; };
                    let layout = action.layout();
                    owner.meter.pay(layout.work());
                    owner.meter.pay(layout.closing_work());
                    owner.metadata.resize(Storage { bytes: <$pool>::inline_bytes(), units: 0 }.add(layout.pool().into()));
                    candidate.resize(layout.candidate().into());
                    let progress = match action.kind() {
                        HtmlValuePoolActionKind::Read => {
                            trace.reads += 1;
                            trace.bytes += layout.work().inspected_bytes();
                            operation.as_mut().unwrap().read(action).unwrap()
                        }
                        HtmlValuePoolActionKind::Pool => {
                            trace.pool_performs += 1;
                            owner.pool.perform(operation.as_mut().unwrap(), action).unwrap()
                        },
                    };
                    if matches!(progress, HtmlValuePoolProgress::CandidateReleased | HtmlValuePoolProgress::Finished) { candidate.shrink_after_drop(Storage::default()); }
                }
                trace
            }
            let meter = Meter::new();
            {
                let mut owner = $owner::new(&meter);
                let value: $value = $make;
                let storage = $storage(&value);
                let input = meter.hold(value, storage);
                let root = owner.begin();
                let mut candidate = meter.reserve(Storage::default());
                let mut operation = Some(owner.pool.probe(&input.value));
                assert_eq!(
                    owner
                        .pool
                        .offer_miss(operation.as_mut().unwrap())
                        .unwrap_err(),
                    HtmlValuePoolError::NotReady
                );
                let trace = drive(&mut owner, &mut operation, &mut candidate);
                assert_eq!(trace, ReadTrace { descriptors: $headers + 8, reads: $headers + 3, bytes: $units * size_of::<u16>(), pool_performs: 4 }, "65-unit value uses three runs plus explicit headers/short components, four candidate probes and final descriptor");
                assert!(owner.pool.offer_miss(operation.as_mut().unwrap()).unwrap());
                assert_eq!(drive(&mut owner, &mut operation, &mut candidate), ReadTrace { descriptors: 3, reads: 0, bytes: 0, pool_performs: 2 }, "same qualified miss retains its complete fingerprint and probe result");
                assert!(matches!(
                    $operation::try_finish(&mut operation).unwrap(),
                    $result::Offered(_)
                ));
                owner.settle();
                // Ordinary caller flow after next_action(None): attempt
                // promotion, then finish either the retained Hit or the offer.
                let mut candidate = meter.reserve(Storage::default());
                let mut hit = Some(owner.pool.probe(&input.value));
                drive(&mut owner, &mut hit, &mut candidate);
                let held = candidate.amount;
                assert!(held.bytes > 0);
                assert!(!owner.pool.offer_miss(hit.as_mut().unwrap()).unwrap());
                assert_eq!(candidate.amount, held);
                let superseding = owner.pool.probe(&input.value);
                assert!(!owner.pool.offer_miss(hit.as_mut().unwrap()).unwrap());
                assert_eq!(candidate.amount, held);
                drop(superseding);
                let $result::Hit(selected) = $operation::try_finish(&mut hit).unwrap() else {
                    panic!("completed Hit survives maybe-promotion")
                };
                let selected = Held {
                    value: selected.into_value(),
                    _receipt: candidate,
                };
                assert!(Arc::ptr_eq(selected.value.$shared(), input.value.$shared()));

                // The same borrowed operand cannot promote after an actual
                // competing operation or after a checkpoint invalidates it.
                let other: $value = $make;
                let storage = $storage(&other);
                let other = meter.hold(other, storage);
                let mut empty = $owner::new(&meter);
                let empty_root = empty.begin();
                let mut candidate = meter.reserve(Storage::default());
                let mut miss = Some(empty.pool.probe(&input.value));
                drive(&mut empty, &mut miss, &mut candidate);
                let superseding = empty.pool.probe(&other.value);
                assert!(!empty.pool.offer_miss(miss.as_mut().unwrap()).unwrap());
                assert!(matches!(
                    $operation::try_finish(&mut miss).unwrap(),
                    $result::Bypassed
                ));
                drop(superseding);
                let mut miss = Some(empty.pool.probe(&input.value));
                drive(&mut empty, &mut miss, &mut candidate);
                let mark = empty.mark(&empty_root).unwrap();
                assert!(!empty.pool.offer_miss(miss.as_mut().unwrap()).unwrap());
                assert!(matches!(
                    $operation::try_finish(&mut miss).unwrap(),
                    $result::Bypassed
                ));
                assert_eq!(empty.storage(), Storage::default());
                empty.restore(mark);
                empty.abort(empty_root);
                owner.abort(root);
                assert!(Arc::ptr_eq(selected.value.$shared(), input.value.$shared()));
            }
            assert_eq!(meter.0.live.get(), Storage::default());
        }
    };
}
miss_promotion!(
    public_string_miss_promotion_reuses_hash_and_length_qualified_reads,
    Strings,
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolResult,
    DocumentString,
    DocumentString::from_utf16(vec![0xd800; 65]),
    string_storage,
    shared_code_units,
    1,
    65
);
miss_promotion!(
    public_attribute_miss_promotion_reuses_hash_and_length_qualified_reads,
    Attributes,
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolResult,
    OrderedAttributes,
    OrderedAttributes::new(vec![Attribute::new(
        QualifiedName::new(Namespace::none(), None, DocumentString::from("cell")),
        DocumentString::from_utf16(vec![0xd800; 65])
    )])
    .unwrap(),
    attribute_storage,
    shared_attributes,
    6,
    69
);
