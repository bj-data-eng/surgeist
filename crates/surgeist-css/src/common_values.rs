//! Shared authored value shapes: `auto | <T>`, `logical? <T>{1,4}` and `<T>{1,2}`.
//!
//! Each shape has one model, one side-assignment rule, one shortest-form rule and one
//! charging rule. The element type supplies only its own equality and its own value text.

use crate::specified_rule_serialization::{SpecifiedProvider, SpecifiedRuleWriter};
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssSerializationError, CssSerializationLimits};

type SerializationResult<T> = Result<T, CssSerializationError>;
type EqualityResult = Result<bool, CssSerializationError>;

/// The `logical` switch of a box-side shorthand: physical or flow-relative side assignment.
///
/// Physical values are assigned in top, right, bottom, left order; logical values in
/// block-start, inline-start, block-end, inline-end order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBoxSideKind {
    /// Top, right, bottom, left.
    Physical,
    /// Block-start, inline-start, block-end, inline-end.
    Logical,
}

/// An `auto | <T>` value.
///
/// The `auto` keyword is deferred to the consumer; the value is whatever `T` retains,
/// including its origin and any symbolic math.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAutoOr<T> {
    /// The deferred `auto` keyword.
    Auto,
    /// The authored value.
    Value(T),
}

impl<T> CssAutoOr<T> {
    /// Returns whether this is the `auto` keyword.
    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Borrows the authored value, or `None` for `auto`.
    #[must_use]
    pub const fn value(&self) -> Option<&T> {
        match self {
            Self::Auto => None,
            Self::Value(value) => Some(value),
        }
    }
}

crate::specified_rule_serialization::specified_serialization!(
    CssAutoOr<crate::CssNumeric<crate::CssLength>>,
    CssAutoOr<crate::CssNumeric<crate::CssLengthPercentage>>,
    CssAutoOr<crate::CssNumeric<crate::CssLengthPercentage, crate::CssNonNegative>>,
);

impl<T: SpecifiedProvider> SpecifiedProvider for CssAutoOr<T> {
    fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Value(value) => value.append_to_rule_writer(writer),
        }
    }
}

/// An element of a [`CssFourSides`] or [`CssSidePair`].
///
/// The element writes its own value and says when two of its values are the same value; the
/// shape owns assignment, shortening and charging.
pub(crate) trait SideValue: SpecifiedProvider {
    /// Whether `other` is the same value, for the shortest-form comparison.
    fn same_side(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult;
}

/// A comparison context that never refuses: shortening compares values that serialization
/// already charges once, and a refusal there would only hide the shorter form.
pub(crate) fn comparison_context() -> SpecifiedSerializationContext {
    SpecifiedSerializationContext::new(CssSerializationLimits::new(
        usize::MAX,
        usize::MAX,
        usize::MAX,
    ))
}

/// `logical? <T>{1,4}` with one side-assignment, shortest-form and charging rule.
///
/// The authored one to four values are retained in source order; [`assigned`](Self::assigned)
/// applies the Box 3 one-to-four assignment.
///
/// # Specified serialization
///
/// Emits the shortest form (CSSOM "serialize a CSS value" step 2): a side that the Box 3
/// copy rules reproduce is omitted. The aggregate charges one input and one projection node,
/// which covers the `logical` keyword; every authored value is charged by its own provider
/// whether or not it is written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFourSides<T> {
    kind: CssBoxSideKind,
    authored: Box<[T]>,
}

impl<T> CssFourSides<T> {
    /// Accepts one to four authored values; any other count is rejected.
    #[must_use]
    pub fn try_new(kind: CssBoxSideKind, authored: Vec<T>) -> Option<Self> {
        (1..=4).contains(&authored.len()).then(|| Self {
            kind,
            authored: authored.into_boxed_slice(),
        })
    }

    /// Returns whether the sides are physical or flow-relative.
    #[must_use]
    pub const fn kind(&self) -> CssBoxSideKind {
        self.kind
    }

    /// Borrows the exact one to four authored values in source order.
    #[must_use]
    pub fn authored(&self) -> &[T] {
        &self.authored
    }

    /// Returns the Box 3 one-to-four assignment: top, right, bottom, left for physical sides
    /// and block-start, inline-start, block-end, inline-end for logical sides.
    ///
    /// This does not determine the shorthand's full reset membership.
    #[must_use]
    pub fn assigned(&self) -> [&T; 4] {
        std::array::from_fn(|role| &self.authored[self.authored_index(role)])
    }

    /// The authored position that supplies one assigned side.
    fn authored_index(&self, role: usize) -> usize {
        match (self.authored.len(), role) {
            (1, _) | (2, 0 | 2) => 0,
            (2, _) | (3, 3) => 1,
            (_, role) => role,
        }
    }
}

impl<T> CssFourSides<T> {
    /// The number of leading authored values that the shortest form writes.
    pub(crate) fn shortest_count(&self) -> usize
    where
        T: SideValue,
    {
        let assigned = self.assigned();
        let mut context = comparison_context();
        crate::common_serialization::shortest_four_side_count(|left, right| {
            assigned[left].same_side(assigned[right], &mut context)
        })
        .unwrap_or(self.authored.len())
    }

    /// Writes the shortest form. `before` runs before each authored value is visited, for
    /// callers that attribute work to the longhand that supplied it.
    pub(crate) fn append_with(
        &self,
        writer: &mut SpecifiedRuleWriter,
        before: &mut impl FnMut(usize),
    ) -> SerializationResult<()>
    where
        T: SideValue,
    {
        // The aggregate belongs to the declaration that supplied the first value.
        writer.source_member(0, |writer| {
            before(0);
            writer.node()
        })?;
        if self.kind == CssBoxSideKind::Logical {
            writer.source_member(0, |writer| writer.append("logical "))?;
        }
        self.append_values(writer, self.shortest_count(), "", before)
    }

    /// Visits every authored value, charging each one: the first `count` are written, separated
    /// by one space and introduced by `lead`, and the rest are visited without output.
    pub(crate) fn append_values(
        &self,
        writer: &mut SpecifiedRuleWriter,
        count: usize,
        lead: &str,
        before: &mut impl FnMut(usize),
    ) -> SerializationResult<()>
    where
        T: SideValue,
    {
        for (index, value) in self.authored.iter().enumerate() {
            writer.source_member(index, |writer| {
                before(index);
                if index >= count {
                    return writer.without_output(|writer| value.append_to_rule_writer(writer));
                }
                writer.append(if index == 0 { lead } else { " " })?;
                value.append_to_rule_writer(writer)
            })?;
        }
        if 0 < count && count < self.authored.len() {
            // The responsible source is the last value whose bytes were written.
            writer.source_member(count - 1, |_| Ok(()))?;
        }
        Ok(())
    }
}

impl<T: SideValue> SpecifiedProvider for CssFourSides<T> {
    fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        self.append_with(writer, &mut |_| {})
    }
}

/// `<T>{1,2}` where an omitted second value copies the first.
///
/// The model keeps whether the second value was authored, so no second origin is invented.
///
/// # Specified serialization
///
/// Emits the shortest form (CSSOM "serialize a CSS value" step 2): a second value that equals
/// the first, as the element type decides, is omitted. Like [`CssFourSides`], the aggregate
/// charges one input and one projection node, and every authored value is charged by its own
/// provider whether or not it is written.
///
/// # Construction
///
/// Pairs of plain checked values are built with `new`. The `cue`, `pause` and `rest` pairs,
/// `CssSidePair<CssCue>` and `CssSidePair<CssSpeechBreak>`, have no `new`: their elements can
/// retain syntax that an implicit close recovered, which a programmatic constructor must reject,
/// so they are built with a checked `try_new`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSidePair<T> {
    first: T,
    authored_second: Option<T>,
}

/// Elements that carry no invariant beyond their own checked type.
///
/// A pair of such elements is constructed with [`CssSidePair::new`]; a pair whose elements
/// retain recoverable syntax (`cue`, `pause` and `rest`) has a checked `try_new` instead.
pub(crate) mod unchecked {
    /// Sealed: only this crate decides which elements are unchecked.
    pub trait Sealed {}
}

impl<T: unchecked::Sealed> CssSidePair<T> {
    /// Retains the first value and whether the second value was authored.
    #[must_use]
    pub const fn new(first: T, authored_second: Option<T>) -> Self {
        Self::from_parts(first, authored_second)
    }
}

impl<T> CssSidePair<T> {
    pub(crate) const fn from_parts(first: T, authored_second: Option<T>) -> Self {
        Self {
            first,
            authored_second,
        }
    }

    /// Borrows the first value.
    #[must_use]
    pub const fn first(&self) -> &T {
        &self.first
    }

    /// Borrows the explicitly authored second value, if any.
    #[must_use]
    pub const fn authored_second(&self) -> Option<&T> {
        self.authored_second.as_ref()
    }

    /// Borrows the effective second value, repeating the first when it was omitted.
    #[must_use]
    pub const fn second(&self) -> &T {
        match &self.authored_second {
            Some(second) => second,
            None => &self.first,
        }
    }
}

impl<T> CssSidePair<T> {
    /// Writes the pair under its family's rule. `equal` decides whether an equal second value
    /// is omitted, and `before` runs before each value is visited, for callers that attribute
    /// work to the longhand that supplied it.
    pub(crate) fn append_with(
        &self,
        writer: &mut SpecifiedRuleWriter,
        equal: &mut impl FnMut(&T, &T, &mut SpecifiedSerializationContext) -> EqualityResult,
        before: &mut impl FnMut(usize),
    ) -> SerializationResult<()>
    where
        T: SideValue,
    {
        // The aggregate belongs to the declaration that supplied the first value.
        writer.source_member(0, |writer| {
            before(0);
            writer.node()
        })?;
        writer.source_member(0, |writer| {
            before(0);
            self.first.append_to_rule_writer(writer)
        })?;
        let Some(second) = &self.authored_second else {
            return Ok(());
        };
        let mut omitted = false;
        writer.source_member(1, |writer| {
            before(1);
            if equal(second, &self.first, &mut writer.context)? {
                writer.without_output(|writer| second.append_to_rule_writer(writer))?;
                omitted = true;
                return Ok(());
            }
            writer.append(" ")?;
            second.append_to_rule_writer(writer)
        })?;
        if omitted {
            // The responsible source is the first value, whose bytes were written.
            writer.source_member(0, |_| Ok(()))?;
        }
        Ok(())
    }
}

impl<T: SideValue> SpecifiedProvider for CssSidePair<T> {
    fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        let mut context = comparison_context();
        self.append_with(
            writer,
            &mut |second, first, _| second.same_side(first, &mut context),
            &mut |_| {},
        )
    }
}

/// Declares `$ty` an element of the shapes: its pair rule and its equality.
macro_rules! side_element {
    (
        $ty:ty,
        same = |$this:ident, $other:ident, $context:ident| $same:expr
    ) => {
        impl $crate::common_values::SideValue for $ty {
            fn same_side(
                &self,
                other: &Self,
                context: &mut $crate::specified_serialization::SpecifiedSerializationContext,
            ) -> ::std::result::Result<bool, $crate::CssSerializationError> {
                let ($this, $other, $context) = (self, other, context);
                $same
            }
        }
    };
}
pub(crate) use side_element;

/// Gives each `CssFourSides<$ty>` its public `serialize_specified`.
macro_rules! four_sides_entry {
    ($($ty:ty),+ $(,)?) => {$(
        $crate::specified_rule_serialization::specified_serialization!(
            $crate::CssFourSides<$ty>
        );
    )+};
}
pub(crate) use four_sides_entry;

/// Gives each `CssSidePair<$ty>` its public `serialize_specified`.
macro_rules! side_pair_serialization {
    ($($ty:ty),+ $(,)?) => {$(
        $crate::specified_rule_serialization::specified_serialization!(
            $crate::CssSidePair<$ty>
        );
    )+};
}
pub(crate) use side_pair_serialization;

/// Gives each `CssSidePair<$ty>` its public constructor and `serialize_specified`.
macro_rules! side_pair_entry {
    ($($ty:ty),+ $(,)?) => {$(
        $crate::common_values::side_pair_serialization!($ty);
        impl $crate::common_values::unchecked::Sealed for $ty {}
    )+};
}
pub(crate) use side_pair_entry;
