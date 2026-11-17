//! Closed dimension sets and the tables that hold one value per dimension.
//!
//! A consumer declares each set with [`dimensions!`](crate::dimensions). The
//! macro writes a fieldless enum, a table that stores one value per variant in
//! an array in declaration order, and a fields struct with one named field per
//! variant, from which a table is built. [`Dimension`] looks a value up at the
//! position of its variant, so in an optimized build a lookup is arithmetic on
//! the dimension and has no branch on which dimension it names. The array has
//! one element per variant, so a lookup cannot go out of range, and the fields
//! struct names every variant, so a table cannot be built with a dimension
//! missing.

use std::{fmt, hash::Hash, marker::PhantomData};

/// A value a dimension table can hold.
pub trait TableValue: Copy + Eq + fmt::Debug + Send + Sync + 'static {}

impl<T: Copy + Eq + fmt::Debug + Send + Sync + 'static> TableValue for T {}

/// A closed set of work or resource dimensions.
///
/// Implement it only through [`dimensions!`](crate::dimensions); the
/// supertrait is hidden so that a hand-written implementation is a deliberate
/// step. The macro guarantees that [`Dimension::get`] and
/// [`Dimension::get_mut`] reach the value stored for the variant.
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

/// Declares a closed dimension set, its table and the fields a table is built from.
///
/// ```
/// surgeist_ledger::dimensions! {
///     /// Work a parser charges.
///     pub enum ParseWork => ParseWorkTable, ParseWorkFields {
///         /// One input byte read.
///         InputByte,
///         /// One tree node visited.
///         NodeVisit,
///     }
/// }
///
/// let limits = ParseWorkTable::new(ParseWorkFields { InputByte: 4096, NodeVisit: 512 });
/// assert_eq!(limits[ParseWork::NodeVisit], 512);
///
/// const UNLIMITED: ParseWorkTable<u64> = ParseWorkTable::splat(u64::MAX);
/// assert_eq!(UNLIMITED[ParseWork::InputByte], u64::MAX);
/// ```
///
/// The enum derives `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `Ord`,
/// `PartialEq` and `PartialOrd`. The table and the fields struct have the
/// enum's visibility.
///
/// The table stores its values in declaration order, implements `Clone`,
/// `Copy`, `Debug`, `Eq`, `Hash` and `PartialEq`, and is indexed by the enum.
/// Its `Debug` output names each value by its variant. `new` and `splat` are
/// `const`, so a constant table stays a constant.
///
/// The fields struct has one public field per variant with the variant's name
/// and documentation, and derives `Clone`, `Copy`, `Debug`, `Eq` and
/// `PartialEq`. A variant accepts doc comments only, because each one is
/// copied to both the variant and its field; the enum itself accepts any
/// attributes.
///
/// A table must be built from a value for every dimension:
///
/// ```compile_fail,E0063
/// surgeist_ledger::dimensions! {
///     pub enum ParseWork => ParseWorkTable, ParseWorkFields { InputByte, NodeVisit }
/// }
///
/// let limits = ParseWorkTable::new(ParseWorkFields { InputByte: 4096 });
/// ```
///
/// The values of a table are private to the module that declares it, so other
/// code cannot build one by position:
///
/// ```compile_fail,E0451
/// mod parser {
///     surgeist_ledger::dimensions! {
///         pub enum ParseWork => ParseWorkTable, ParseWorkFields { InputByte, NodeVisit }
///     }
/// }
///
/// let limits = parser::ParseWorkTable { values: [4096, 512] };
/// ```
#[macro_export]
macro_rules! dimensions {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident => $table:ident, $fields:ident {
            $(
                $(#[doc = $variant_doc:expr])*
                $variant:ident
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        $vis enum $name {
            $(
                $(#[doc = $variant_doc])*
                $variant,
            )+
        }

        #[doc = concat!(
            "One value per [`", stringify!($name), "`] dimension, by name. ",
            "A [`", stringify!($table), "`] is built from it."
        )]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[allow(non_snake_case)]
        $vis struct $fields<T> {
            $(
                $(#[doc = $variant_doc])*
                pub $variant: T,
            )+
        }

        impl<T: Copy> $fields<T> {
            /// Fields holding `value` for every dimension, as the base of a
            /// struct update that names the others.
            #[allow(dead_code)]
            $vis const fn splat(value: T) -> Self {
                Self {
                    $($variant: value,)+
                }
            }
        }

        #[doc = concat!("One value per [`", stringify!($name), "`] dimension.")]
        #[derive(Clone, Copy, Eq, Hash, PartialEq)]
        $vis struct $table<T> {
            values: [T; [$($name::$variant),+].len()],
        }

        impl<T: Copy> $table<T> {
            /// The table holding each value of `fields` for the dimension
            /// that names it.
            #[allow(dead_code)]
            $vis const fn new(fields: $fields<T>) -> Self {
                Self {
                    values: [$(fields.$variant),+],
                }
            }

            /// A table holding `value` for every dimension.
            #[allow(dead_code)]
            $vis const fn splat(value: T) -> Self {
                Self {
                    values: [value; [$($name::$variant),+].len()],
                }
            }
        }

        impl<T> ::core::ops::Index<$name> for $table<T> {
            type Output = T;

            #[inline]
            fn index(&self, key: $name) -> &T {
                // The array holds one value per variant, so the position of a
                // variant is always present. The first value stands in for a
                // position the enum cannot produce, which leaves the lookup
                // without a panic path.
                let [first, rest @ ..] = &self.values;
                match (key as usize).checked_sub(1) {
                    ::core::option::Option::None => first,
                    ::core::option::Option::Some(position) => {
                        rest.get(position).unwrap_or(first)
                    }
                }
            }
        }

        impl<T> ::core::ops::IndexMut<$name> for $table<T> {
            #[inline]
            fn index_mut(&mut self, key: $name) -> &mut T {
                let [first, rest @ ..] = &mut self.values;
                match (key as usize).checked_sub(1) {
                    ::core::option::Option::None => first,
                    ::core::option::Option::Some(position) => {
                        rest.get_mut(position).unwrap_or(first)
                    }
                }
            }
        }

        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for $table<T> {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter
                    .debug_struct(stringify!($table))
                    $(.field(stringify!($variant), &self[$name::$variant]))+
                    .finish()
            }
        }

        impl $crate::__private::Sealed for $name {}

        impl $crate::Dimension for $name {
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            type Table<T: $crate::TableValue> = $table<T>;

            #[inline]
            fn get<T: $crate::TableValue>(table: &$table<T>, key: Self) -> &T {
                &table[key]
            }

            #[inline]
            fn get_mut<T: $crate::TableValue>(table: &mut $table<T>, key: Self) -> &mut T {
                &mut table[key]
            }

            fn from_fn<T: $crate::TableValue>(mut value: impl FnMut(Self) -> T) -> $table<T> {
                $table {
                    values: [$(value(Self::$variant)),+],
                }
            }
        }
    };
}
