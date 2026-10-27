//! Tests for the `limits` module: the plain limit types and the charging
//! helpers `grow`, `copy_str` and `copy_boxed`.
//!
//! The helpers define one growth and copy model, documented on the helpers and
//! in the module: capacity doubling with a minimum of four, a new bank of
//! exactly the planned capacity, both banks live while elements move, one step
//! per element or scalar value moved, the old bank returned afterwards. The
//! expected call sequences below are derived by hand from that model for
//! concrete inputs, never by running the helpers.

use std::cell::{Cell, RefCell};
use std::mem::size_of;
use std::rc::Rc;

use surgeist_css_parser::limits::{
    DepthExcess, Failure, Limits, Meter, Refused, copy_boxed, copy_str, grow,
};

/// One call a helper made on its meter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Call {
    Step,
    Add(usize),
    Released(usize),
    Note(Failure),
}

use Call::{Add, Note, Released, Step};

/// A meter that records every call, refusing the fallible call (a `step` or an
/// `add`, counted from zero in the order they are made) at one chosen index.
#[derive(Default)]
struct Recording {
    calls: RefCell<Vec<Call>>,
    fallible: Cell<usize>,
    refuse_at: Option<usize>,
}

impl Recording {
    fn admitting() -> Self {
        Self::default()
    }

    fn refusing_call(index: usize) -> Self {
        Self {
            refuse_at: Some(index),
            ..Self::default()
        }
    }

    fn calls(&self) -> Vec<Call> {
        self.calls.borrow().clone()
    }

    fn admit(&self, call: Call) -> Result<(), Refused> {
        self.calls.borrow_mut().push(call);
        let index = self.fallible.get();
        self.fallible.set(index + 1);
        if self.refuse_at == Some(index) {
            Err(Refused)
        } else {
            Ok(())
        }
    }
}

impl Meter for Recording {
    fn step(&self) -> Result<(), Refused> {
        self.admit(Step)
    }

    fn add(&self, bytes: usize) -> Result<(), Refused> {
        self.admit(Add(bytes))
    }

    fn released(&self, bytes: usize) {
        self.calls.borrow_mut().push(Released(bytes));
    }

    fn note(&self, failure: Failure) -> Refused {
        self.calls.borrow_mut().push(Note(failure));
        Refused
    }
}

/// The charged size of the state that carries elements between banks: a vector
/// for the new bank and the iterator over the old one.
fn header<T>() -> usize {
    size_of::<Vec<T>>() + size_of::<std::vec::IntoIter<T>>()
}

/// A vector with exactly `capacity` capacity holding `len` elements `0..len`.
fn filled(len: usize, capacity: usize) -> Vec<u32> {
    let mut values = Vec::with_capacity(capacity);
    values.extend(0..u32::try_from(len).unwrap());
    assert_eq!((values.len(), values.capacity()), (len, capacity));
    values
}

fn steps(count: usize) -> Vec<Call> {
    vec![Step; count]
}

// ---- Limits, DepthExcess, Failure, Refused ---------------------------------

#[test]
fn limits_report_the_values_they_were_built_with() {
    let limits = Limits::new(32, 10_000, 1 << 20);
    assert_eq!(limits.max_depth(), 32);
    assert_eq!(limits.max_nodes(), 10_000);
    assert_eq!(limits.max_bytes(), 1 << 20);
}

#[test]
fn default_limits_have_depth_256_and_no_node_or_byte_limit() {
    let limits = Limits::default();
    assert_eq!(limits.max_depth(), 256);
    assert_eq!(limits.max_nodes(), usize::MAX);
    assert_eq!(limits.max_bytes(), usize::MAX);
}

#[test]
fn limits_are_copyable_comparable_values() {
    let limits = Limits::new(1, 2, 3);
    let copy = limits;
    assert_eq!(limits, copy);
    assert_ne!(limits, Limits::new(1, 2, 4));
    assert_ne!(limits, Limits::new(2, 2, 3));
    assert_ne!(limits, Limits::new(1, 3, 3));
}

#[test]
fn limits_are_constructible_in_a_constant() {
    const LIMITS: Limits = Limits::new(8, 16, 32);
    const DEPTH: u32 = LIMITS.max_depth();
    assert_eq!(DEPTH, 8);
}

#[test]
fn depth_excess_defaults_to_failing_the_build() {
    assert_eq!(DepthExcess::default(), DepthExcess::Fail);
    assert_ne!(DepthExcess::Fail, DepthExcess::DenyBlock);
}

#[test]
fn the_two_failures_are_distinct() {
    assert_ne!(Failure::Allocation, Failure::CapacityOverflow);
}

#[test]
fn refused_displays_a_message_and_is_a_standard_error() {
    let refused = Refused;
    assert_eq!(refused.to_string(), "the meter refused the action");
    let boxed: Box<dyn std::error::Error> = Box::new(refused);
    assert_eq!(boxed.to_string(), "the meter refused the action");
    assert_eq!(refused, Refused);
}

// ---- grow ------------------------------------------------------------------

#[test]
fn growing_a_vector_with_room_takes_one_step_and_changes_nothing() {
    let meter = Recording::admitting();
    let mut values = filled(3, 8);
    grow(&meter, &mut values, 5).unwrap();
    assert_eq!(meter.calls(), [Step]);
    assert_eq!((values.len(), values.capacity()), (3, 8));
    assert_eq!(values, [0, 1, 2]);
}

#[test]
fn growing_by_nothing_a_full_vector_takes_one_step_and_changes_nothing() {
    let meter = Recording::admitting();
    let mut values = filled(4, 4);
    grow(&meter, &mut values, 0).unwrap();
    assert_eq!(meter.calls(), [Step]);
    assert_eq!(values.capacity(), 4);
}

#[test]
fn growing_an_empty_vector_plans_the_minimum_of_four_elements() {
    // required 1, doubled 0, minimum 4: target 4 elements of 4 bytes.
    let meter = Recording::admitting();
    let mut values: Vec<u32> = Vec::new();
    grow(&meter, &mut values, 1).unwrap();
    assert_eq!(
        meter.calls(),
        [Step, Add(16 + header::<u32>()), Released(header::<u32>())]
    );
    assert_eq!((values.len(), values.capacity()), (0, 4));
}

#[test]
fn growing_a_one_element_vector_applies_the_minimum_over_doubling() {
    // required 2, doubled 2, minimum 4: target 4. One element moves.
    let meter = Recording::admitting();
    let mut values = filled(1, 1);
    grow(&meter, &mut values, 1).unwrap();
    let h = header::<u32>();
    assert_eq!(meter.calls(), [Step, Add(16 + h), Step, Released(4 + h)]);
    assert_eq!(values.capacity(), 4);
    assert_eq!(values, [0]);
}

#[test]
fn growing_a_full_vector_doubles_its_capacity_and_steps_once_per_element() {
    // required 5, doubled 8, minimum 4: target 8 elements of 4 bytes, old bank
    // of 16 bytes, four elements moved.
    let meter = Recording::admitting();
    let mut values = filled(4, 4);
    grow(&meter, &mut values, 1).unwrap();
    let h = header::<u32>();
    let mut expected = vec![Step, Add(32 + h)];
    expected.extend(steps(4));
    expected.push(Released(16 + h));
    assert_eq!(meter.calls(), expected);
    assert_eq!(values.capacity(), 8);
    assert_eq!(values, [0, 1, 2, 3]);
}

#[test]
fn growing_by_more_than_doubling_plans_exactly_the_required_length() {
    // required 24 beats doubled 8: target 24 elements of 4 bytes.
    let meter = Recording::admitting();
    let mut values = filled(4, 4);
    grow(&meter, &mut values, 20).unwrap();
    let h = header::<u32>();
    let mut expected = vec![Step, Add(96 + h)];
    expected.extend(steps(4));
    expected.push(Released(16 + h));
    assert_eq!(meter.calls(), expected);
    assert_eq!(values.capacity(), 24);
}

#[test]
fn growing_a_vector_with_unused_capacity_doubles_that_capacity() {
    // len 0, capacity 8, required 9: doubled 16 beats required 9. No element
    // moves; the old bank is 32 bytes.
    let meter = Recording::admitting();
    let mut values = filled(0, 8);
    grow(&meter, &mut values, 9).unwrap();
    let h = header::<u32>();
    assert_eq!(meter.calls(), [Step, Add(64 + h), Released(32 + h)]);
    assert_eq!(values.capacity(), 16);
}

#[test]
fn growth_charges_by_the_size_of_the_element_type() {
    // Three-byte elements: required 5, doubled 8: target 8, so a planned bank
    // of 24 bytes replacing 12.
    let meter = Recording::admitting();
    let mut values: Vec<[u8; 3]> = Vec::with_capacity(4);
    values.extend([[1; 3], [2; 3], [3; 3], [4; 3]]);
    grow(&meter, &mut values, 1).unwrap();
    let h = header::<[u8; 3]>();
    let mut expected = vec![Step, Add(24 + h)];
    expected.extend(steps(4));
    expected.push(Released(12 + h));
    assert_eq!(meter.calls(), expected);
    assert_eq!(values.capacity(), 8);
    assert_eq!(values[3], [4; 3]);
}

#[test]
fn growing_a_vector_of_zero_sized_elements_only_takes_the_opening_step() {
    // The capacity of such a vector is unbounded, so there is always room.
    let meter = Recording::admitting();
    let mut values: Vec<()> = Vec::new();
    grow(&meter, &mut values, 5).unwrap();
    assert_eq!(meter.calls(), [Step]);
}

#[test]
fn growth_moves_elements_without_cloning_or_dropping_them() {
    let meter = Recording::admitting();
    let marker = Rc::new(());
    let mut values = vec![Rc::clone(&marker), Rc::clone(&marker)];
    values.shrink_to_fit();
    assert_eq!(values.capacity(), 2);
    grow(&meter, &mut values, 1).unwrap();
    assert_eq!(values.len(), 2);
    assert_eq!(
        Rc::strong_count(&marker),
        3,
        "two moved elements and the marker"
    );
    assert!(Rc::ptr_eq(&values[0], &marker));
}

#[test]
fn growth_refused_at_the_opening_step_changes_nothing() {
    let meter = Recording::refusing_call(0);
    let mut values = filled(4, 4);
    assert_eq!(grow(&meter, &mut values, 1), Err(Refused));
    assert_eq!(meter.calls(), [Step]);
    assert_eq!((values.len(), values.capacity()), (4, 4));
    assert_eq!(values, [0, 1, 2, 3]);
}

#[test]
fn growth_refused_at_the_new_banks_charge_changes_nothing_and_releases_nothing() {
    let meter = Recording::refusing_call(1);
    let mut values = filled(4, 4);
    assert_eq!(grow(&meter, &mut values, 1), Err(Refused));
    assert_eq!(meter.calls(), [Step, Add(32 + header::<u32>())]);
    assert_eq!((values.len(), values.capacity()), (4, 4));
    assert_eq!(values, [0, 1, 2, 3]);
}

#[test]
fn growth_refused_while_moving_releases_both_banks_and_leaves_the_vector_empty() {
    // Calls 0 and 1 are the opening step and the charge; calls 2 to 5 are the
    // four element steps. Refuse the third element step (call 4).
    let meter = Recording::refusing_call(4);
    let mut values = filled(4, 4);
    assert_eq!(grow(&meter, &mut values, 1), Err(Refused));
    let h = header::<u32>();
    assert_eq!(
        meter.calls(),
        [
            Step,
            Add(32 + h),
            Step,
            Step,
            Step,
            // Old bank (16), new bank (32) and the bookkeeping state.
            Released(16 + 32 + h),
        ]
    );
    assert!(values.is_empty());
}

#[test]
fn growth_refused_at_the_first_element_step_also_releases_both_banks() {
    let meter = Recording::refusing_call(2);
    let mut values = filled(4, 4);
    assert_eq!(grow(&meter, &mut values, 1), Err(Refused));
    let h = header::<u32>();
    assert_eq!(
        meter.calls(),
        [Step, Add(32 + h), Step, Released(16 + 32 + h)]
    );
    assert!(values.is_empty());
}

#[test]
fn growth_refused_while_moving_drops_every_element_exactly_once() {
    let meter = Recording::refusing_call(3);
    let marker = Rc::new(());
    let mut values: Vec<Rc<()>> = Vec::with_capacity(4);
    values.extend((0..4).map(|_| Rc::clone(&marker)));
    assert_eq!(Rc::strong_count(&marker), 5);
    assert_eq!(grow(&meter, &mut values, 1), Err(Refused));
    assert_eq!(Rc::strong_count(&marker), 1, "no element leaked or doubled");
    assert!(values.is_empty());
}

#[test]
fn growth_that_cannot_reserve_its_bank_releases_the_charge_and_notes_allocation() {
    // A bank of isize::MAX bytes passes every size check and is far beyond
    // what any allocator provides.
    let target = usize::try_from(isize::MAX).unwrap();
    let meter = Recording::admitting();
    let mut values: Vec<u8> = Vec::new();
    assert_eq!(grow(&meter, &mut values, target), Err(Refused));
    let charged = target + header::<u8>();
    assert_eq!(
        meter.calls(),
        [
            Step,
            Add(charged),
            Released(charged),
            Note(Failure::Allocation)
        ]
    );
    assert!(values.is_empty());
    assert_eq!(values.capacity(), 0);
}

#[test]
fn growth_whose_required_length_overflows_notes_capacity_overflow() {
    let meter = Recording::admitting();
    let mut values: Vec<u8> = vec![1];
    assert_eq!(grow(&meter, &mut values, usize::MAX), Err(Refused));
    assert_eq!(meter.calls(), [Step, Note(Failure::CapacityOverflow)]);
    assert_eq!(values, [1]);
}

#[test]
fn growth_whose_planned_bytes_overflow_notes_capacity_overflow() {
    // 2^63 two-byte elements are 2^64 bytes.
    let meter = Recording::admitting();
    let mut values: Vec<u16> = Vec::new();
    assert_eq!(grow(&meter, &mut values, 1 << 63), Err(Refused));
    assert_eq!(meter.calls(), [Step, Note(Failure::CapacityOverflow)]);
}

#[test]
fn growth_whose_bookkeeping_state_overflows_the_charge_notes_capacity_overflow() {
    // The planned bank alone fits in a usize; adding the bookkeeping state does
    // not.
    let meter = Recording::admitting();
    let mut values: Vec<u8> = Vec::new();
    assert_eq!(grow(&meter, &mut values, usize::MAX - 10), Err(Refused));
    assert_eq!(meter.calls(), [Step, Note(Failure::CapacityOverflow)]);
}

// ---- copy_str --------------------------------------------------------------

#[test]
fn copying_a_string_takes_an_opening_step_charges_its_bytes_and_steps_per_scalar() {
    let meter = Recording::admitting();
    let copy = copy_str(&meter, "abc").unwrap();
    assert_eq!(meter.calls(), [Step, Add(3), Step, Step, Step]);
    assert_eq!(copy, "abc");
    assert_eq!(copy.capacity(), 3, "the capacity is exactly the length");
}

#[test]
fn copying_an_empty_string_takes_the_opening_step_and_charges_nothing_more() {
    let meter = Recording::admitting();
    let copy = copy_str(&meter, "").unwrap();
    assert_eq!(meter.calls(), [Step, Add(0)]);
    assert!(copy.is_empty());
    assert_eq!(copy.capacity(), 0);
}

#[test]
fn copying_a_string_charges_bytes_but_steps_per_scalar_value_not_per_byte() {
    // One, two and four bytes: seven bytes, three scalar values.
    let text = "a\u{e9}\u{1f600}";
    assert_eq!(text.len(), 7);
    let meter = Recording::admitting();
    let copy = copy_str(&meter, text).unwrap();
    assert_eq!(meter.calls(), [Step, Add(7), Step, Step, Step]);
    assert_eq!(copy, text);
    assert_eq!(copy.capacity(), 7);
}

#[test]
fn copying_a_string_refused_at_the_opening_step_charges_nothing() {
    let meter = Recording::refusing_call(0);
    assert_eq!(copy_str(&meter, "abc"), Err(Refused));
    assert_eq!(meter.calls(), [Step]);
}

#[test]
fn copying_a_string_refused_at_its_charge_releases_nothing() {
    let meter = Recording::refusing_call(1);
    assert_eq!(copy_str(&meter, "abc"), Err(Refused));
    assert_eq!(meter.calls(), [Step, Add(3)]);
}

#[test]
fn copying_a_string_refused_at_a_scalar_step_releases_the_partial_copys_capacity() {
    // Calls 0 and 1 are the opening step and the charge; call 2 is the first
    // scalar, call 3 the second.
    let meter = Recording::refusing_call(3);
    assert_eq!(copy_str(&meter, "abc"), Err(Refused));
    assert_eq!(meter.calls(), [Step, Add(3), Step, Step, Released(3)]);
}

#[test]
fn copying_a_string_refused_at_its_first_scalar_step_releases_the_charge() {
    let meter = Recording::refusing_call(2);
    assert_eq!(copy_str(&meter, "xy"), Err(Refused));
    assert_eq!(meter.calls(), [Step, Add(2), Step, Released(2)]);
}

// ---- copy_boxed ------------------------------------------------------------

#[test]
fn copying_to_a_boxed_string_charges_as_copying_a_string_when_no_shrink_is_needed() {
    let meter = Recording::admitting();
    let copy = copy_boxed(&meter, "abc").unwrap();
    assert_eq!(meter.calls(), [Step, Add(3), Step, Step, Step]);
    assert_eq!(&*copy, "abc");
}

#[test]
fn copying_an_empty_string_to_a_boxed_string_charges_as_copying_a_string() {
    let meter = Recording::admitting();
    let copy = copy_boxed(&meter, "").unwrap();
    assert_eq!(meter.calls(), [Step, Add(0)]);
    assert_eq!(&*copy, "");
}

#[test]
fn copying_to_a_boxed_string_charges_per_scalar_value() {
    let meter = Recording::admitting();
    let copy = copy_boxed(&meter, "\u{e9}\u{1f600}").unwrap();
    assert_eq!(meter.calls(), [Step, Add(6), Step, Step]);
    assert_eq!(&*copy, "\u{e9}\u{1f600}");
}

#[test]
fn copying_to_a_boxed_string_passes_refusals_through_as_copying_a_string_does() {
    for refused in 0..5 {
        let boxed = Recording::refusing_call(refused);
        let plain = Recording::refusing_call(refused);
        assert_eq!(copy_boxed(&boxed, "abc"), Err(Refused));
        assert_eq!(copy_str(&plain, "abc"), Err(Refused));
        assert_eq!(boxed.calls(), plain.calls(), "refused call {refused}");
    }
}
