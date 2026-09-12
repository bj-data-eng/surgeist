//! Intrinsic declaration expansion, before cascade or contextual resolution.
//!
//! The property schema selects the supported box, border and flow-tolerance slice and
//! owns its longhand types, initial values, shorthand members and reset-only
//! members. Custom declarations retain their symbolic specified values.
//! Unselected known properties return an explicit capability error.

use std::fmt;
use std::sync::Arc;

use crate::properties::{CssKnownDeclaration, CssKnownDeclaredValueRef, CssKnownPropertyValueRef};
use crate::syntax::*;
use crate::{
    CssBorderColors, CssComponentValueRef, CssComponentValues, CssKnownProperty,
    CssPropertyValueParseError,
};

/// Why intrinsic expansion could not produce completed contributions.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssExpansionErrorKind {
    /// The known property is outside the currently selected expansion slice.
    UnsupportedProperty(CssKnownProperty),
    /// Strict reentry still contains a decoded `var()` function.
    ResidualSubstitution,
    /// Replacement components failed serialization or the original property grammar.
    InvalidReplacement(CssPropertyValueParseError),
}

/// A typed expansion capability or strict-reentry failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssExpansionError {
    kind: CssExpansionErrorKind,
}

impl CssExpansionError {
    /// Returns the precise failure without losing known or custom property identity.
    #[must_use]
    pub const fn kind(&self) -> &CssExpansionErrorKind {
        &self.kind
    }

    fn new(kind: CssExpansionErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for CssExpansionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            CssExpansionErrorKind::UnsupportedProperty(property) => {
                write!(
                    formatter,
                    "intrinsic expansion does not support {}",
                    property.canonical_name()
                )
            }
            CssExpansionErrorKind::ResidualSubstitution => {
                formatter.write_str("replacement still contains var() substitution")
            }
            CssExpansionErrorKind::InvalidReplacement(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for CssExpansionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.kind() {
            CssExpansionErrorKind::InvalidReplacement(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
enum ContributionValue<T> {
    Ordinary(T),
    Global(CssGlobalKeyword),
}

// First filter the full property inventory to annotated rows. The bounded
// collector then emits complete enums and matches; macros never expand to
// partial enum variants or match arms, and no second property registry exists.
macro_rules! define_expansion_schema {
    ($input:ident; $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $wrapper:ident, $representation:ident, $parser:ident, $dispatch:block
        $(, expansion = $kind:ident { $($metadata:tt)* })?;
    )*) => {
        define_expansion_schema!(@collect [] [] [];
            $($( $variant, $kind { $($metadata)* }; )?)*
        );
    };
    (@collect [$($longhands:tt)*] [$($shorthands:tt)*] [$($universal:tt)*];
        $variant:ident, longhand {
            wrapper: $wrapper_kind:ident, value: $value:ty,
            accessor: $accessor:ident, initial: $initial:expr
        }; $($rest:tt)*
    ) => {
        define_expansion_schema!(@collect
            [$($longhands)* ($variant, $value, $accessor, $initial)]
            [$($shorthands)*] [$($universal)*]; $($rest)*
        );
    };
    (@collect [$($longhands:tt)*] [$($shorthands:tt)*] [$($universal:tt)*];
        $variant:ident, shorthand {
            wrapper: $wrapper_kind:ident, accessor: $accessor:ident,
            members: [$($member:ident => $projection:expr),+],
            reset_only: [$($reset:ident),*]
        }; $($rest:tt)*
    ) => {
        define_expansion_schema!(@collect
            [$($longhands)*]
            [$($shorthands)* ($variant, $accessor, [$($member => $projection),+], [$($reset),*])]
            [$($universal)*]; $($rest)*
        );
    };
    (@collect [$($longhands:tt)*] [$($shorthands:tt)*] [$($universal:tt)*];
        $variant:ident, universal { exclude_custom: $exclude_custom:literal,
            excluded: [$($excluded:ident),+]
        }; $($rest:tt)*
    ) => {
        define_expansion_schema!(@collect
            [$($longhands)*] [$($shorthands)*]
            [$($universal)* ($variant, $exclude_custom, [$($excluded),+])]; $($rest)*
        );
    };
    (@collect
        [$(($longhand:ident, $value:ty, $accessor:ident, $initial:expr))*]
        [$(($shorthand:ident, $shorthand_accessor:ident,
            [$($member:ident => $projection:expr),+], [$($reset:ident),*]))*]
        [($universal:ident, $exclude_custom:literal, [$($excluded:ident),+])];
    ) => {
        #[derive(Clone, Copy, Debug)]
        enum Longhand {
            $($longhand,)*
        }

        #[derive(Clone, Debug)]
        enum OwnedLonghandValue {
            $($longhand(ContributionValue<$value>),)*
        }

        /// A borrowed exact ordinary longhand value coupled to its property.
        ///
        /// The variants are generated only for the selected box, border and
        /// flow-tolerance longhands. Symbolic values stay unresolved.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum CssLonghandValueRef<'a> {
            $(#[doc = concat!("The exact ordinary value for `", stringify!($longhand), "`.")]
            $longhand(&'a $value),)*
        }

        impl Longhand {
            fn initial(self) -> OwnedLonghandValue {
                match self {
                    $(Self::$longhand => OwnedLonghandValue::$longhand(
                        ContributionValue::Ordinary($initial)
                    ),)*
                }
            }

            fn global(self, keyword: CssGlobalKeyword) -> OwnedLonghandValue {
                match self {
                    $(Self::$longhand => OwnedLonghandValue::$longhand(
                        ContributionValue::Global(keyword)
                    ),)*
                }
            }
        }

        impl OwnedLonghandValue {
            const fn property(&self) -> CssKnownProperty {
                match self {
                    $(Self::$longhand(_) => CssKnownProperty::$longhand,)*
                }
            }

            const fn view(&self) -> CssContributionValueRef<'_> {
                match self {
                    $(Self::$longhand(ContributionValue::Ordinary(value)) => {
                        CssContributionValueRef::Ordinary(CssLonghandValueRef::$longhand(value))
                    }
                    Self::$longhand(ContributionValue::Global(keyword)) => {
                        CssContributionValueRef::Global(*keyword)
                    })*
                }
            }
        }

        fn expansion_shape(property: CssKnownProperty) -> Result<ExpansionShape, CssExpansionError> {
            match property {
                CssKnownProperty::$universal => Ok(ExpansionShape::UniversalReset),
                $(CssKnownProperty::$longhand => {
                    Ok(ExpansionShape::Longhands(&[Longhand::$longhand]))
                })*
                $(CssKnownProperty::$shorthand => {
                    Ok(ExpansionShape::Longhands(&[$(Longhand::$member,)+ $(Longhand::$reset,)*]))
                })*
                _ => Err(CssExpansionError::new(CssExpansionErrorKind::UnsupportedProperty(property))),
            }
        }

        fn universal_excludes(property: CssPropertyNameRef<'_>) -> bool {
            match property {
                CssPropertyNameRef::Custom(_) => $exclude_custom,
                CssPropertyNameRef::Known(property) => matches!(property, $(CssKnownProperty::$excluded)|+),
            }
        }

        fn ordinary_values(
            property: CssKnownProperty,
            value: CssKnownPropertyValueRef<'_>,
        ) -> Result<Vec<OwnedLonghandValue>, CssExpansionError> {
            match value {
                $(CssKnownPropertyValueRef::$longhand(value) => Ok(vec![
                    OwnedLonghandValue::$longhand(ContributionValue::Ordinary(value.$accessor().to_owned()))
                ]),)*
                $(CssKnownPropertyValueRef::$shorthand(value) => {
                    let value = value.$shorthand_accessor();
                    Ok(vec![
                        $(match ($projection)(value) {
                            Some(projected) => OwnedLonghandValue::$member(ContributionValue::Ordinary(projected)),
                            None => Longhand::$member.initial(),
                        },)+
                        $(Longhand::$reset.initial(),)*
                    ])
                })*
                _ => Err(CssExpansionError::new(CssExpansionErrorKind::UnsupportedProperty(property))),
            }
        }
    };
}

crate::properties::property_schema!(define_expansion_schema, expansion_input);

#[derive(Clone, Copy, Debug)]
enum ExpansionShape {
    Longhands(&'static [Longhand]),
    UniversalReset,
}

/// A borrowed longhand contribution's ordinary value or whole-value CSS-wide keyword.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssContributionValueRef<'a> {
    Ordinary(CssLonghandValueRef<'a>),
    Global(CssGlobalKeyword),
}

#[derive(Debug)]
struct ContributionContext {
    source: CssDeclaration,
    replacement: Option<CssComponentValues>,
}

/// One property-coupled contribution retaining its authored source occurrence.
///
/// Fields and construction are private: the ordinary payload always belongs to
/// [`Self::property`]. Contributions share one source/replacement context, so
/// expansion does not duplicate a complete component tree for each longhand.
#[derive(Clone, Debug)]
pub struct CssLonghandContribution {
    value: OwnedLonghandValue,
    context: Arc<ContributionContext>,
}

impl CssLonghandContribution {
    /// Returns the property identity derived from the active value variant.
    #[must_use]
    pub fn property(&self) -> CssKnownProperty {
        self.value.property()
    }

    /// Borrows the exact ordinary value or symbolic global keyword.
    #[must_use]
    pub fn value(&self) -> CssContributionValueRef<'_> {
        self.value.view()
    }

    /// Returns the original declaration, including importance and occurrence identity.
    #[must_use]
    pub fn source(&self) -> &CssDeclaration {
        &self.context.source
    }

    /// Returns the caller's replacement input after strict reentry, if any.
    #[must_use]
    pub fn replacement_components(&self) -> Option<&CssComponentValues> {
        self.context.replacement.as_ref()
    }
}

/// Completed selected longhand contributions from one declaration occurrence.
#[derive(Clone, Debug)]
pub struct CssLonghandContributions {
    items: Vec<CssLonghandContribution>,
}

impl CssLonghandContributions {
    /// Returns the unique contributed longhands in property-schema member order.
    #[must_use]
    pub fn items(&self) -> &[CssLonghandContribution] {
        &self.items
    }
}

/// The symbolic effect of `all`, before selecting applicable cascade targets.
#[derive(Clone, Debug)]
pub struct CssUniversalReset {
    keyword: CssGlobalKeyword,
    context: Arc<ContributionContext>,
}

impl CssUniversalReset {
    /// Returns the authored or replacement CSS-wide keyword without resolving it.
    #[must_use]
    pub const fn keyword(&self) -> CssGlobalKeyword {
        self.keyword
    }

    /// Returns the original `all` declaration occurrence and importance.
    #[must_use]
    pub fn source(&self) -> &CssDeclaration {
        &self.context.source
    }

    /// Returns the caller's replacement components after strict reentry, if any.
    #[must_use]
    pub fn replacement_components(&self) -> Option<&CssComponentValues> {
        self.context.replacement.as_ref()
    }

    /// Reports only the explicit `all` exclusions: custom properties, `direction`
    /// and `unicode-bidi`. A false result is not full target applicability;
    /// selection, inheritance and cascade remain downstream responsibilities.
    #[must_use]
    pub fn excludes(&self, property: CssPropertyNameRef<'_>) -> bool {
        universal_excludes(property)
    }
}

/// One symbolic custom-property contribution before cascade and substitution.
///
/// Private construction guarantees a custom declaration. The original occurrence
/// supplies its case-sensitive name, token value or CSS-wide keyword, importance,
/// and provenance without copying a second representation of those semantics.
#[derive(Clone, Debug)]
pub struct CssCustomPropertyContribution {
    source: CssDeclaration,
}

impl CssCustomPropertyContribution {
    /// Returns the unchanged authored declaration and its occurrence identity.
    #[must_use]
    pub const fn source(&self) -> &CssDeclaration {
        &self.source
    }

    /// Borrows the validated custom name and symbolic specified value.
    #[must_use]
    pub fn declaration(&self) -> &CssCustomDeclaration {
        self.source
            .custom()
            .expect("custom contributions are constructed only from custom declarations")
    }
}

/// Completed intrinsic contributions; custom values and universal resets stay symbolic.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssContributions {
    Longhands(CssLonghandContributions),
    UniversalReset(CssUniversalReset),
    Custom(CssCustomPropertyContribution),
}

/// An intrinsic expansion result, possibly awaiting external substitution.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssExpansion {
    Contributions(CssContributions),
    Pending(CssPendingSubstitution),
}

/// A supported authored declaration whose grammar awaits substitution.
///
/// This object retains the original occurrence. Reentry is immutable and
/// retryable; failures never publish a partially expanded contribution set.
#[derive(Clone, Debug)]
pub struct CssPendingSubstitution {
    source: CssDeclaration,
}

impl CssPendingSubstitution {
    /// Returns the unchanged authored declaration awaiting replacement.
    #[must_use]
    pub const fn source(&self) -> &CssDeclaration {
        &self.source
    }

    /// Checks replacement components against the original property and expands
    /// them atomically. This strict transition returns completed contributions
    /// or an error, never another pending state. It preserves the caller's token
    /// origins and the original declaration's identity and importance.
    ///
    /// A decoded `var()` at any component depth returns `ResidualSubstitution`
    /// before grammar checking. Other invalid values retain the same mapped
    /// error as [`crate::parse_property_value`], including source resource limits.
    pub fn reenter(
        &self,
        replacement: CssComponentValues,
    ) -> Result<CssContributions, CssExpansionError> {
        if contains_substitution(&replacement) {
            return Err(CssExpansionError::new(
                CssExpansionErrorKind::ResidualSubstitution,
            ));
        }
        let body = crate::property_value::checked_property_value_body(
            self.source.property_name(),
            &replacement,
        )
        .map_err(|error| {
            CssExpansionError::new(CssExpansionErrorKind::InvalidReplacement(error))
        })?;
        let CssDeclarationBody::Known(known) = body else {
            unreachable!("pending expansion is created only for supported known properties");
        };
        let shape = expansion_shape(known.property())?;
        complete_contributions(
            &known,
            shape,
            Arc::new(ContributionContext {
                source: self.source.clone(),
                replacement: Some(replacement),
            }),
        )
    }
}

/// Expands custom declarations and selected box, border and flow-tolerance declarations.
///
/// Ordinary shorthands contribute every member, applying intrinsic initial
/// values to omissions. `border` also resets the five border-image longhands;
/// CSS-wide keywords propagate to ordinary and reset-only members alike. `all`
/// remains a symbolic reset. Substitution-dependent supported declarations
/// return a pending handle, and unselected known properties return typed errors.
/// Custom declarations return one completed symbolic contribution even when their
/// tokens contain `var()`; their values are not substituted or computed here.
/// This operation does not choose cascade winners, substitute variables,
/// resolve writing modes, evaluate lengths or colors, or load images.
pub fn expand_declaration(source: &CssDeclaration) -> Result<CssExpansion, CssExpansionError> {
    let known = match source.body() {
        CssDeclarationBody::Known(known) => known,
        CssDeclarationBody::Custom(_) => {
            return Ok(CssExpansion::Contributions(CssContributions::Custom(
                CssCustomPropertyContribution {
                    source: source.clone(),
                },
            )));
        }
    };
    let shape = expansion_shape(known.property())?;
    if known.substitution_dependent().is_some() {
        return Ok(CssExpansion::Pending(CssPendingSubstitution {
            source: source.clone(),
        }));
    }
    complete_contributions(
        known,
        shape,
        Arc::new(ContributionContext {
            source: source.clone(),
            replacement: None,
        }),
    )
    .map(CssExpansion::Contributions)
}

/// Counts the schema-selected output before normalization allocates expansion members.
/// Pending, custom, and universal-reset groups each occupy one symbolic output unit.
pub(crate) fn expansion_member_count(source: &CssDeclaration) -> Result<usize, CssExpansionError> {
    let Some(known) = source.known() else {
        return Ok(1);
    };
    let shape = expansion_shape(known.property())?;
    if known.substitution_dependent().is_some() {
        return Ok(1);
    }
    Ok(match shape {
        ExpansionShape::Longhands(members) => members.len(),
        ExpansionShape::UniversalReset => 1,
    })
}

fn complete_contributions(
    known: &CssKnownDeclaration,
    shape: ExpansionShape,
    context: Arc<ContributionContext>,
) -> Result<CssContributions, CssExpansionError> {
    let values = match known.declared_value() {
        CssKnownDeclaredValueRef::Global(keyword) => match shape {
            ExpansionShape::UniversalReset => {
                return Ok(CssContributions::UniversalReset(CssUniversalReset {
                    keyword,
                    context,
                }));
            }
            ExpansionShape::Longhands(members) => members
                .iter()
                .map(|member| member.global(keyword))
                .collect(),
        },
        CssKnownDeclaredValueRef::Property(value) => ordinary_values(known.property(), value)?,
        CssKnownDeclaredValueRef::SubstitutionDependent(_) => {
            return Err(CssExpansionError::new(
                CssExpansionErrorKind::ResidualSubstitution,
            ));
        }
    };
    Ok(CssContributions::Longhands(CssLonghandContributions {
        items: values
            .into_iter()
            .map(|value| CssLonghandContribution {
                value,
                context: Arc::clone(&context),
            })
            .collect(),
    }))
}

fn contains_substitution(values: &CssComponentValues) -> bool {
    values.items().iter().any(|value| match value.view() {
        CssComponentValueRef::Function(function) => {
            function.name().eq_ignore_ascii_case("var") || contains_substitution(function.values())
        }
        CssComponentValueRef::Block(block) => contains_substitution(block.values()),
        CssComponentValueRef::Token(_) | CssComponentValueRef::Comment(_) => false,
    })
}

#[cfg(test)]
#[path = "expansion/metadata_initial_tests.rs"]
mod metadata_initial_tests;
