//! Fixed tables addressed by a closed enum, with no index that can be out of range.
//!
//! A [`Table`] stores one value per key variant. Its key type implements
//! [`Slots`] through [`slots!`], which expands to an exhaustive `match` from every
//! variant to a literal array index. The compiler then proves three facts:
//!
//! * a variant added to the key without a table slot is a non-exhaustive match;
//! * a literal index at or beyond the array length is rustc's deny-by-default
//!   `unconditional_panic` error; and
//! * [`slots!`] also evaluates, at compile time, that the slots are distinct, that
//!   there are exactly as many as the array holds, and that each equals the
//!   variant's discriminant (declaration order).
//!
//! Lookup is therefore total by type. Nothing here panics, and callers do not
//! index with `as usize`.

use std::{fmt, marker::PhantomData};

/// Maps every variant of a closed enum to its element of an `N`-element array.
pub(crate) trait Slots<const N: usize>: Copy {
    fn slot_ref<T>(table: &[T; N], key: Self) -> &T;
    fn slot_mut<T>(table: &mut [T; N], key: Self) -> &mut T;
}

/// One value per variant of `K`, stored in variant order.
pub(crate) struct Table<K, T, const N: usize> {
    values: [T; N],
    key: PhantomData<fn() -> K>,
}

impl<K: Slots<N>, T, const N: usize> Table<K, T, N> {
    /// Builds a table from values listed in variant order.
    pub(crate) const fn new(values: [T; N]) -> Self {
        Self {
            values,
            key: PhantomData,
        }
    }
    /// Builds a table holding the same value for every key.
    pub(crate) fn filled(value: T) -> Self
    where
        T: Copy,
    {
        Self::new([value; N])
    }
    pub(crate) fn get(&self, key: K) -> &T {
        K::slot_ref(&self.values, key)
    }
    pub(crate) fn get_mut(&mut self, key: K) -> &mut T {
        K::slot_mut(&mut self.values, key)
    }
    /// The value stored under `key`, copied out.
    pub(crate) fn copy(&self, key: K) -> T
    where
        T: Copy,
    {
        *self.get(key)
    }
    /// Transforms every value, keeping each under its own key.
    pub(crate) fn map<U>(self, f: impl FnMut(T) -> U) -> Table<K, U, N> {
        Table::new(self.values.map(f))
    }
    /// The values in variant order.
    pub(crate) fn as_array(&self) -> &[T; N] {
        &self.values
    }
    /// The values in variant order, mutably.
    pub(crate) fn as_mut_array(&mut self) -> &mut [T; N] {
        &mut self.values
    }
    /// The values in variant order, by value.
    pub(crate) fn into_array(self) -> [T; N] {
        self.values
    }
}

impl<K, T: Clone, const N: usize> Clone for Table<K, T, N> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            key: PhantomData,
        }
    }
}
impl<K, T: Copy, const N: usize> Copy for Table<K, T, N> {}
impl<K, T: PartialEq, const N: usize> PartialEq for Table<K, T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl<K, T: Eq, const N: usize> Eq for Table<K, T, N> {}
impl<K, T: fmt::Debug, const N: usize> fmt::Debug for Table<K, T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.values, f)
    }
}

/// Implements [`Slots`] for a fieldless enum and proves the mapping at compile time.
///
/// Every variant is listed with its slot. The slot must equal the variant's
/// discriminant, so a table is also readable in declaration order.
macro_rules! slots {
    ($key:ty, $count:expr, { $($variant:ident => $slot:literal),+ $(,)? }) => {
        impl $crate::keyed::Slots<{ $count }> for $key {
            fn slot_ref<T>(table: &[T; $count], key: Self) -> &T {
                match key {
                    $(Self::$variant => &table[$slot],)+
                }
            }
            fn slot_mut<T>(table: &mut [T; $count], key: Self) -> &mut T {
                match key {
                    $(Self::$variant => &mut table[$slot],)+
                }
            }
        }
        const _: () = {
            let slots: &[usize] = &[$($slot),+];
            let discriminants: &[usize] = &[$(<$key>::$variant as usize),+];
            assert!(slots.len() == $count, "every slot of the table is listed once");
            let mut i = 0;
            while i < slots.len() {
                assert!(slots[i] < $count, "slot lies inside the table");
                assert!(
                    slots[i] == discriminants[i],
                    "slot equals the variant's discriminant"
                );
                let mut j = i + 1;
                while j < slots.len() {
                    assert!(slots[i] != slots[j], "slots are distinct");
                    j += 1;
                }
                i += 1;
            }
        };
    };
}
pub(crate) use slots;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Probe {
        First,
        Second,
        Third,
    }
    slots!(Probe, 3, {
        First => 0,
        Second => 1,
        Third => 2,
    });
    type ProbeTable<T> = Table<Probe, T, 3>;

    #[test]
    fn each_key_addresses_exactly_its_own_variant_ordered_cell() {
        let mut table = ProbeTable::new([10, 20, 30]);
        assert_eq!(table.copy(Probe::First), 10);
        assert_eq!(table.copy(Probe::Second), 20);
        assert_eq!(table.copy(Probe::Third), 30);
        *table.get_mut(Probe::Second) = 21;
        assert_eq!(table.as_array(), &[10, 21, 30]);
        assert_eq!(*table.get(Probe::Second), 21);
        assert_eq!(table.map(|value| value + 1).into_array(), [11, 22, 31]);
    }
    #[test]
    fn filled_table_holds_one_value_per_key_and_prints_as_its_array() {
        let table = ProbeTable::filled(7_u8);
        assert_eq!(table.into_array(), [7, 7, 7]);
        assert_eq!(format!("{table:?}"), "[7, 7, 7]");
        assert_eq!(table, ProbeTable::new([7, 7, 7]));
    }
}
