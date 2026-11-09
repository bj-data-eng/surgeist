//! Closed dimension sets and the tables that hold one value per dimension.
//!
//! A consumer declares each set with [`dimensions!`](crate::dimensions). The
//! macro writes a fieldless enum and a table struct with one named field per
//! variant, and implements [`Dimension`] by matching every variant to its own
//! field. A lookup is therefore total by type: it uses no index, so it cannot
//! go out of range, and a table cannot be built with a dimension missing.

use std::{fmt, hash::Hash, marker::PhantomData};

/// A value a dimension table can hold.
pub trait TableValue: Copy + Eq + fmt::Debug + Send + Sync + 'static {}

impl<T: Copy + Eq + fmt::Debug + Send + Sync + 'static> TableValue for T {}

/// A closed set of work or resource dimensions.
///
/// Implement it only through [`dimensions!`](crate::dimensions); the
/// supertrait is hidden so that a hand-written implementation is a deliberate
/// step. The macro guarantees that [`Dimension::get`] and
/// [`Dimension::get_mut`] reach the field named by the variant.
pub trait Dimension:
    Copy + Eq + Hash + fmt::Debug + Send + Sync + 'static + crate::__private::Sealed
{
    /// Every dimension, in declaration order.
    const ALL: &'static [Self];

    /// One value per dimension.
    type Table<T: TableValue>: TableValue;

    /// The value stored for `key`.
    fn get<T: TableValue>(table: &Self::Table<T>, key: Self) -> &T;

    /// The value stored for `key`, for update.
    fn get_mut<T: TableValue>(table: &mut Self::Table<T>, key: Self) -> &mut T;

    /// Builds a table by calling `value` once per dimension, in declaration order.
    fn from_fn<T: TableValue>(value: impl FnMut(Self) -> T) -> Self::Table<T>;
}

/// The empty dimension set, for an operation with no work or no resource
/// dimensions. It has no values, so no change can name it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NoDimension {}

/// The table of [`NoDimension`]. It holds no values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NoTable<T>(PhantomData<T>);

impl<T> NoTable<T> {
    /// The only table of the empty dimension set.
    pub const EMPTY: Self = Self(PhantomData);
}

impl crate::__private::Sealed for NoDimension {}

impl Dimension for NoDimension {
    const ALL: &'static [Self] = &[];

    type Table<T: TableValue> = NoTable<T>;

    fn get<T: TableValue>(_: &NoTable<T>, key: Self) -> &T {
        match key {}
    }

    fn get_mut<T: TableValue>(_: &mut NoTable<T>, key: Self) -> &mut T {
        match key {}
    }

    fn from_fn<T: TableValue>(_: impl FnMut(Self) -> T) -> NoTable<T> {
        NoTable::EMPTY
    }
}

/// Declares a closed dimension set and its table.
///
/// ```
/// surgeist_ledger::dimensions! {
///     /// Work a parser charges.
///     pub enum ParseWork => ParseWorkTable {
///         /// One input byte read.
///         InputByte,
///         /// One tree node visited.
///         NodeVisit,
///     }
/// }
///
/// let limits = ParseWorkTable { InputByte: 4096, NodeVisit: 512 };
/// assert_eq!(limits.NodeVisit, 512);
///
/// const UNLIMITED: ParseWorkTable<u64> = ParseWorkTable::splat(u64::MAX);
/// assert_eq!(UNLIMITED.InputByte, u64::MAX);
/// ```
///
/// The enum derives `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `Ord`,
/// `PartialEq` and `PartialOrd`. The table has the enum's visibility, one
/// public field per variant with the variant's name and attributes, and
/// derives `Clone`, `Copy`, `Debug`, `Eq`, `Hash` and `PartialEq`.
///
/// A table built from its struct must name every dimension:
///
/// ```compile_fail,E0063
/// surgeist_ledger::dimensions! {
///     pub enum ParseWork => ParseWorkTable { InputByte, NodeVisit }
/// }
///
/// let limits = ParseWorkTable { InputByte: 4096 };
/// ```
#[macro_export]
macro_rules! dimensions {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident => $table:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        #[doc = concat!("One value per [`", stringify!($name), "`] dimension.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[allow(non_snake_case)]
        $vis struct $table<T> {
            $(
                $(#[$variant_meta])*
                pub $variant: T,
            )+
        }

        impl<T: Copy> $table<T> {
            /// A table holding `value` for every dimension.
            #[allow(dead_code)]
            $vis const fn splat(value: T) -> Self {
                Self {
                    $($variant: value,)+
                }
            }
        }

        impl $crate::__private::Sealed for $name {}

        impl $crate::Dimension for $name {
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            type Table<T: $crate::TableValue> = $table<T>;

            fn get<T: $crate::TableValue>(table: &$table<T>, key: Self) -> &T {
                match key {
                    $(Self::$variant => &table.$variant,)+
                }
            }

            fn get_mut<T: $crate::TableValue>(table: &mut $table<T>, key: Self) -> &mut T {
                match key {
                    $(Self::$variant => &mut table.$variant,)+
                }
            }

            fn from_fn<T: $crate::TableValue>(mut value: impl FnMut(Self) -> T) -> $table<T> {
                $table {
                    $($variant: value(Self::$variant),)+
                }
            }
        }
    };
}
