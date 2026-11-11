//! Provenance carriers and the comparison that includes them.
//!
//! `==` on authored syntax, component values, specified numerics and diagnostics is
//! structural: it compares what a value says and ignores where it was read from. A field
//! that records where (an origin, span or position) is stored as a [`Provenance`],
//! whose equality is true outside [`same_provenance`], so a derived `PartialEq` over such
//! fields is structural without a hand-written impl. [`same_provenance`] is the named
//! operation that also compares them.

use super::CssSourceSnapshot;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

thread_local! {
    // Whether `Provenance` equality compares its payload on this thread.
    static COMPARING: Cell<bool> = const { Cell::new(false) };
    static SOURCE_COMPARISON: RefCell<Option<SourceComparison>> = const { RefCell::new(None) };
}

enum SourceComparison {
    Identity,
    ExactPair(CssSourceSnapshot, CssSourceSnapshot),
}

struct SourceScope(Option<SourceComparison>);

impl SourceScope {
    fn enter(comparison: SourceComparison) -> Self {
        Self(SOURCE_COMPARISON.replace(Some(comparison)))
    }
}

impl Drop for SourceScope {
    fn drop(&mut self) {
        SOURCE_COMPARISON.replace(self.0.take());
    }
}

// Called only after actual snapshot identity failed. An active scope refuses
// all unmatched pairs without reading their normalized text.
pub(super) fn compare_distinct_sources(
    left: &CssSourceSnapshot,
    right: &CssSourceSnapshot,
) -> Option<bool> {
    SOURCE_COMPARISON.with_borrow(|comparison| {
        comparison.as_ref().map(|comparison| match comparison {
            SourceComparison::Identity => false,
            SourceComparison::ExactPair(first, second) => {
                (left.same_snapshot(first) && right.same_snapshot(second))
                    || (left.same_snapshot(second) && right.same_snapshot(first))
            }
        })
    })
}

pub(crate) fn same_provenance_by_source_identity<T: PartialEq + ?Sized>(
    left: &T,
    right: &T,
) -> bool {
    let _sources = SourceScope::enter(SourceComparison::Identity);
    same_provenance(left, right)
}

pub(crate) fn same_provenance_in_exact_sources<T: PartialEq + ?Sized, E>(
    left: &T,
    left_source: &CssSourceSnapshot,
    right: &T,
    right_source: &CssSourceSnapshot,
    mut charge_unit: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    let (Some(left_units), Some(right_units)) =
        (left_source.original_utf16(), right_source.original_utf16())
    else {
        return Ok(false);
    };
    if !std::sync::Arc::ptr_eq(left_units, right_units) {
        if left_units.len() != right_units.len() {
            return Ok(false);
        }
        for (left, right) in left_units.iter().zip(right_units.iter()) {
            charge_unit()?;
            if left != right {
                return Ok(false);
            }
        }
    }
    let _sources = SourceScope::enter(SourceComparison::ExactPair(
        left_source.clone(),
        right_source.clone(),
    ));
    Ok(same_provenance(left, right))
}

/// Restores the comparison mode that was in force when a scope began, including on unwind.
struct Scope(bool);

impl Scope {
    fn comparing() -> Self {
        Self(COMPARING.replace(true))
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        COMPARING.set(self.0);
    }
}

/// Reports whether `left` and `right` are equal and carry equal provenance: every origin
/// (with the source text it addresses), span and position recorded in them compares as
/// well as the values themselves. A parse source retained beside a value is compared by
/// neither `==` nor this function.
///
/// `==` ignores that provenance, so this is the comparison for asking whether two values
/// came from the same place in the same source, for example to detect that a sheet or a
/// diagnostic list was replaced by structurally equal syntax parsed from different text.
///
/// ```
/// use surgeist_css::query::media::CssMediaQueryList;
/// use surgeist_css::source::same_provenance;
/// use surgeist_css::CssParser;
///
/// let compact = CssMediaQueryList::parse(&CssParser::default(), "a,b");
/// let spaced = CssMediaQueryList::parse(&CssParser::default(), "a , b");
/// assert_eq!(compact.syntax(), spaced.syntax());
/// assert!(!same_provenance(compact.syntax(), spaced.syntax()));
/// assert!(same_provenance(compact.syntax(), compact.syntax()));
/// ```
#[must_use]
pub fn same_provenance<T: PartialEq + ?Sized>(left: &T, right: &T) -> bool {
    let _scope = Scope::comparing();
    left == right
}

/// A field that records where a value came from instead of what it is.
///
/// Equality is true unless the comparison runs under [`same_provenance`], and hashing
/// ignores the payload, so a derived `PartialEq`, `Eq` and `Hash` over this field are
/// structural and agree with each other. `Debug` shows the payload alone.
#[derive(Clone, Copy, Default)]
pub(crate) struct Provenance<T>(T);

impl<T> Provenance<T> {
    pub(crate) const fn new(value: T) -> Self {
        Self(value)
    }

    pub(crate) const fn inner(&self) -> &T {
        &self.0
    }

    pub(crate) fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Provenance<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Provenance<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Provenance<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<T: PartialEq> PartialEq for Provenance<T> {
    fn eq(&self, other: &Self) -> bool {
        !COMPARING.get() || self.0 == other.0
    }
}

impl<T: Eq> Eq for Provenance<T> {}

impl<T> Hash for Provenance<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

/// A retained field that no equality compares: the source a value was parsed from, kept for
/// its consumer and its accounting, and bookkeeping that is not part of the value.
///
/// Equality is true under `==` and under [`same_provenance`] alike, and hashing ignores the
/// payload. `Debug` shows the payload alone.
#[derive(Clone, Copy, Default)]
pub(crate) struct Uncompared<T>(T);

impl<T> Uncompared<T> {
    pub(crate) const fn new(value: T) -> Self {
        Self(value)
    }

    pub(crate) const fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Uncompared<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: fmt::Debug> fmt::Debug for Uncompared<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<T> PartialEq for Uncompared<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Eq for Uncompared<T> {}

impl<T> Hash for Uncompared<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

/// A value whose layout the grammar gives no meaning, such as the whitespace and comments
/// around the delimiters of a raw operand list that is the only data a type retains.
///
/// `==` compares through [`LayoutEq`], which ignores that layout; under [`same_provenance`]
/// the payload compares exactly, so the layout is compared too. Hashing ignores the
/// payload. `Debug` shows the payload alone.
#[derive(Clone, Default)]
pub(crate) struct Layout<T>(T);

/// Equality that ignores the layout (whitespace and comments) its grammar gives no meaning.
pub(crate) trait LayoutEq: PartialEq {
    fn layout_eq(&self, other: &Self) -> bool;
}

impl<T> Layout<T> {
    pub(crate) const fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Layout<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Layout<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Layout<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<T: LayoutEq> PartialEq for Layout<T> {
    fn eq(&self, other: &Self) -> bool {
        if COMPARING.get() {
            self.0 == other.0
        } else {
            self.0.layout_eq(&other.0)
        }
    }
}

impl<T: LayoutEq + Eq> Eq for Layout<T> {}

impl<T> Hash for Layout<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

/// A raw operand list whose text is the value, as a custom property's value and text that
/// awaits substitution are: the declaration path compares the authored text between the
/// first and last significant token.
///
/// `==` compares through [`VerbatimEq`], which trims the edge whitespace and comments and
/// compares the text between them, interior whitespace and comments included; under
/// [`same_provenance`] the payload compares exactly. Hashing ignores the payload. `Debug`
/// shows the payload alone.
#[derive(Clone, Default)]
pub(crate) struct Verbatim<T>(T);

/// Equality of the authored text between the first and last significant token.
pub(crate) trait VerbatimEq: PartialEq {
    fn verbatim_eq(&self, other: &Self) -> bool;
}

impl<T> Verbatim<T> {
    pub(crate) const fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Verbatim<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Verbatim<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Verbatim<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<T: VerbatimEq> PartialEq for Verbatim<T> {
    fn eq(&self, other: &Self) -> bool {
        if COMPARING.get() {
            self.0 == other.0
        } else {
            self.0.verbatim_eq(&other.0)
        }
    }
}

impl<T: VerbatimEq + Eq> Eq for Verbatim<T> {}

impl<T> Hash for Verbatim<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
