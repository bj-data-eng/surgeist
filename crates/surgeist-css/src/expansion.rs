//! Intrinsic declaration expansion, before cascade or contextual resolution.
//!
//! The property schema selects the supported box, border, flow-tolerance, color
//! and inherited typography slice and
//! owns its longhand types, initial values, shorthand members and reset-only
//! members. Custom declarations retain their symbolic specified values.
//! Unselected known properties return an explicit capability error.

use std::fmt;
use std::sync::Arc;

use crate::parser::contains_substitution;
use crate::properties::{CssKnownDeclaration, CssKnownDeclaredValueRef, CssKnownPropertyValueRef};
use crate::syntax::*;
use crate::{
    CssBorderColors, CssComponentValues, CssContainer, CssContainerNames, CssContainerType,
    CssKnownProperty, CssPropertyValueParseError,
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
                formatter.write_str("replacement still contains var() or env() substitution")
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

macro_rules! intrinsic_initial {
    ($variant:ident, value, $initial:expr) => {
        CssLonghandInitialValue {
            value: InitialValue::Value(CssLonghandValue {
                value: Box::new(OwnedLonghandValue::$variant($initial)),
            }),
        }
    };
    ($variant:ident, user_agent, $initial:expr) => {
        CssLonghandInitialValue {
            value: InitialValue::UserAgent($initial),
        }
    };
}

// First filter the full property inventory to annotated rows. The bounded
// collector then emits complete enums and matches; macros never expand to
// partial enum variants or match arms, and no second property registry exists.
macro_rules! define_expansion_schema {
    ($input:ident, $numeric:ident; $(
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
            accessor: $accessor:ident, inherited: $inherited:literal, initial_kind: $initial_kind:ident, initial: $initial:expr
        }; $($rest:tt)*
    ) => {
        define_expansion_schema!(@collect
            [$($longhands)* ($variant, $value, $accessor, $inherited, $initial_kind, $initial)]
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
        [$(($longhand:ident, $value:ty, $accessor:ident, $inherited:literal, $initial_kind:ident, $initial:expr))*]
        [$(($shorthand:ident, $shorthand_accessor:ident,
            [$($member:ident => $projection:expr),+], [$($reset:ident),*]))*]
        [($universal:ident, $exclude_custom:literal, [$($excluded:ident),+])];
    ) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        enum Longhand {
            $($longhand,)*
        }

        #[derive(Clone, Debug, PartialEq)]
        enum OwnedLonghandValue {
            $($longhand($value),)*
        }

        /// A borrowed exact ordinary longhand value coupled to its property.
        ///
        /// The variants are generated only for schema-selected longhands.
        /// Symbolic values stay unresolved.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum CssLonghandValueRef<'a> {
            $(#[doc = concat!("The exact ordinary value for `", stringify!($longhand), "`.")]
            $longhand(&'a $value),)*
        }

        impl Longhand {
            fn initial(self) -> OwnedContributionValue {
                OwnedContributionValue::from_initial(self.initial_value())
            }

            fn initial_value(self) -> CssLonghandInitialValue {
                match self {
                    $(Self::$longhand => intrinsic_initial!($longhand, $initial_kind, $initial),)*
                }
            }

            const fn property(self) -> CssKnownProperty {
                match self { $(Self::$longhand => CssKnownProperty::$longhand,)* }
            }

            const fn inherited(self) -> bool {
                match self { $(Self::$longhand => $inherited,)* }
            }

            fn global(self, keyword: CssGlobalKeyword) -> OwnedContributionValue {
                OwnedContributionValue::Global(CssLonghandProperty(self), keyword)
            }
        }

        impl OwnedLonghandValue {
            const fn property(&self) -> CssLonghandProperty {
                match self {
                    $(Self::$longhand(_) => CssLonghandProperty(Longhand::$longhand),)*
                }
            }

            const fn view(&self) -> CssLonghandValueRef<'_> {
                match self { $(Self::$longhand(value) => CssLonghandValueRef::$longhand(value),)* }
            }
        }

        pub(crate) fn grammar_metadata(
            grammar: crate::CssPropertyGrammar,
        ) -> Result<&'static CssPropertyMetadata, CssPropertyMetadataError> {
            match grammar.resolved() {
                crate::properties::CssResolvedPropertyName::LegacyShorthand(
                    crate::properties::CssLegacyPropertyAlias::GlyphOrientationVertical,
                ) => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::TextOrientation.legacy_shorthands()[0],
                        kind: CssPropertyKindRef::Shorthand(&CssShorthandMetadata {
                            members: &[CssLonghandProperty(Longhand::TextOrientation)],
                            settable: &[CssLonghandProperty(Longhand::TextOrientation)],
                            reset: &[],
                            legacy: true,
                        }),
                    };
                    return Ok(&METADATA);
                }
                crate::properties::CssResolvedPropertyName::Canonical(_) => {}
            }
            match grammar.target_property() {
                $(CssKnownProperty::$longhand => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::$longhand.grammar(),
                        kind: CssPropertyKindRef::Longhand(&CssLonghandMetadata {
                            property: CssLonghandProperty(Longhand::$longhand),
                        }),
                    };
                    Ok(&METADATA)
                },)*
                $(CssKnownProperty::$shorthand => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::$shorthand.grammar(),
                        kind: CssPropertyKindRef::Shorthand(&CssShorthandMetadata {
                            members: &[
                                $(CssLonghandProperty(Longhand::$member),)+
                                $(CssLonghandProperty(Longhand::$reset),)*
                            ],
                            settable: &[$(CssLonghandProperty(Longhand::$member),)+],
                            reset: &[$(CssLonghandProperty(Longhand::$reset),)*],
                            legacy: false,
                        }),
                    };
                    Ok(&METADATA)
                },)*
                CssKnownProperty::$universal => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::$universal.grammar(),
                        kind: CssPropertyKindRef::UniversalReset(&CssUniversalResetMetadata { _private: () }),
                    };
                    Ok(&METADATA)
                },
                _ => Err(CssPropertyMetadataError::Unavailable(grammar)),
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
        ) -> Result<Vec<OwnedContributionValue>, CssExpansionError> {
            match value {
                $(CssKnownPropertyValueRef::$longhand(value) => Ok(vec![
                    OwnedContributionValue::Ordinary(CssLonghandValue { value: Box::new(OwnedLonghandValue::$longhand(value.$accessor().to_owned())) })
                ]),)*
                $(CssKnownPropertyValueRef::$shorthand(value) => {
                    let value = value.$shorthand_accessor();
                    Ok(vec![
                        $(match ($projection)(value) {
                            Some(projected) => OwnedContributionValue::Ordinary(CssLonghandValue { value: Box::new(OwnedLonghandValue::$member(projected)) }),
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

crate::properties::property_schema!(define_expansion_schema, expansion_input, numeric_input);

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
    /// An intrinsic initial requiring a user-agent environment.
    UserAgentInitial(CssUserAgentInitial),
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
    value: OwnedContributionValue,
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

    /// Borrows the coupled ordinary value, excluding CSS-wide and UA initial states.
    #[must_use]
    pub fn ordinary_value(&self) -> Option<&CssLonghandValue> {
        match &self.value {
            OwnedContributionValue::Ordinary(value) => Some(value),
            _ => None,
        }
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
    /// A decoded `var()` or `env()` at any component depth returns `ResidualSubstitution`
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
        let body = crate::property_value::checked_grammar_value_body(
            self.source
                .known()
                .expect("pending known declaration")
                .grammar(),
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

/// Expands custom declarations and the schema-selected intrinsic property slice.
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

#[cfg(test)]
#[path = "expansion/metadata_initial_tests.rs"]
mod metadata_initial_tests;

/// A checked terminal property identity. Construction stays with the schema owner.
/// ```compile_fail
/// use surgeist_css::{CssKnownProperty, CssLonghandProperty};
/// let _ = CssLonghandProperty(CssKnownProperty::Width);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssLonghandProperty(Longhand);
impl CssLonghandProperty {
    /// Returns the corresponding canonical property identity.
    #[must_use]
    pub const fn known_property(self) -> CssKnownProperty {
        self.0.property()
    }
}
/// An owned ordinary value whose active payload determines its terminal property.
/// ```compile_fail
/// use surgeist_css::CssLonghandValue;
/// let _ = CssLonghandValue { value: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssLonghandValue {
    value: Box<OwnedLonghandValue>,
}
impl CssLonghandValue {
    /// Returns the terminal property coupled to this value.
    #[must_use]
    pub fn property(&self) -> CssLonghandProperty {
        self.value.property()
    }
    /// Borrows its exact symbolic ordinary payload.
    #[must_use]
    pub fn view(&self) -> CssLonghandValueRef<'_> {
        self.value.view()
    }
}
/// Intrinsic initial requirements resolved only by the downstream user agent.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssUserAgentInitial {
    FontFamily,
}
impl CssUserAgentInitial {
    /// Returns the terminal property requiring context.
    #[must_use]
    pub const fn property(self) -> CssLonghandProperty {
        match self {
            Self::FontFamily => CssLonghandProperty(Longhand::FontFamily),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum InitialValue {
    Value(CssLonghandValue),
    UserAgent(CssUserAgentInitial),
}
/// An intrinsic initial, retaining either a symbolic ordinary value or a UA requirement.
/// ```compile_fail
/// use surgeist_css::CssLonghandInitialValue;
/// let _ = CssLonghandInitialValue { value: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssLonghandInitialValue {
    value: InitialValue,
}
/// A borrowed intrinsic initial without invented resolved values or provenance.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssInitialValueRef<'a> {
    Value(&'a CssLonghandValue),
    UserAgent(CssUserAgentInitial),
}
impl CssLonghandInitialValue {
    /// Returns the property determined by the active initial state.
    #[must_use]
    pub fn property(&self) -> CssLonghandProperty {
        match &self.value {
            InitialValue::Value(v) => v.property(),
            InitialValue::UserAgent(v) => v.property(),
        }
    }
    /// Borrows the initial without resolving external context.
    #[must_use]
    pub fn view(&self) -> CssInitialValueRef<'_> {
        match &self.value {
            InitialValue::Value(v) => CssInitialValueRef::Value(v),
            InitialValue::UserAgent(v) => CssInitialValueRef::UserAgent(*v),
        }
    }
}
#[derive(Clone, Debug)]
enum OwnedContributionValue {
    Ordinary(CssLonghandValue),
    Global(CssLonghandProperty, CssGlobalKeyword),
    UserAgent(CssUserAgentInitial),
}
impl OwnedContributionValue {
    fn from_initial(value: CssLonghandInitialValue) -> Self {
        match value.value {
            InitialValue::Value(v) => Self::Ordinary(v),
            InitialValue::UserAgent(v) => Self::UserAgent(v),
        }
    }
    fn property(&self) -> CssKnownProperty {
        match self {
            Self::Ordinary(v) => v.property(),
            Self::Global(p, _) => *p,
            Self::UserAgent(v) => v.property(),
        }
        .known_property()
    }
    fn view(&self) -> CssContributionValueRef<'_> {
        match self {
            Self::Ordinary(v) => CssContributionValueRef::Ordinary(v.view()),
            Self::Global(_, v) => CssContributionValueRef::Global(*v),
            Self::UserAgent(v) => CssContributionValueRef::UserAgentInitial(*v),
        }
    }
}
/// Intrinsic metadata availability is separate from recognition and parser support.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssPropertyMetadataError {
    Unavailable(crate::CssPropertyGrammar),
}
impl fmt::Display for CssPropertyMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(g) => write!(f, "intrinsic metadata unavailable for {}", g.name()),
        }
    }
}
impl std::error::Error for CssPropertyMetadataError {}
/// Schema-owned intrinsic grammar, initial and expansion metadata.
#[derive(Debug)]
pub struct CssPropertyMetadata {
    grammar: crate::CssPropertyGrammar,
    kind: CssPropertyKindRef<'static>,
}
impl CssPropertyMetadata {
    /// Returns the authored grammar described by this metadata.
    #[must_use]
    pub const fn grammar(&self) -> crate::CssPropertyGrammar {
        self.grammar
    }
    /// Returns the exact intrinsic property kind.
    #[must_use]
    pub const fn kind(&self) -> CssPropertyKindRef<'_> {
        self.kind
    }
}
/// Intrinsic metadata branches, never an unavailable placeholder.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum CssPropertyKindRef<'a> {
    Longhand(&'a CssLonghandMetadata),
    Shorthand(&'a CssShorthandMetadata),
    UniversalReset(&'a CssUniversalResetMetadata),
}
/// A terminal property's inheritance and intrinsic initial.
#[derive(Debug)]
pub struct CssLonghandMetadata {
    property: CssLonghandProperty,
}
impl CssLonghandMetadata {
    /// Returns the terminal property identity.
    #[must_use]
    pub const fn property(&self) -> CssLonghandProperty {
        self.property
    }
    /// Reports specified inheritance behavior, without performing cascade.
    #[must_use]
    pub const fn inherited_by_default(&self) -> bool {
        self.property.0.inherited()
    }
    /// Constructs its intrinsic initial without a fabricated authored occurrence.
    #[must_use]
    pub fn initial_value(&self) -> CssLonghandInitialValue {
        self.property.0.initial_value()
    }
}
/// Ordered terminal members of a canonical or legacy shorthand.
#[derive(Debug)]
pub struct CssShorthandMetadata {
    members: &'static [CssLonghandProperty],
    settable: &'static [CssLonghandProperty],
    reset: &'static [CssLonghandProperty],
    legacy: bool,
}
impl CssShorthandMetadata {
    /// Returns settable members followed by reset-only members.
    #[must_use]
    pub const fn members(&self) -> &'static [CssLonghandProperty] {
        self.members
    }
    /// Returns members directly settable by the shorthand grammar.
    #[must_use]
    pub const fn settable_members(&self) -> &'static [CssLonghandProperty] {
        self.settable
    }
    /// Returns reset-only terminal members.
    #[must_use]
    pub const fn reset_only_members(&self) -> &'static [CssLonghandProperty] {
        self.reset
    }
    /// Reports a distinct legacy grammar rather than a name-equivalent alias.
    #[must_use]
    pub const fn is_legacy(&self) -> bool {
        self.legacy
    }
}
/// Intrinsic exclusions for `all`, before contextual target selection.
/// ```compile_fail
/// use surgeist_css::CssUniversalResetMetadata;
/// let _ = CssUniversalResetMetadata { _private: () };
/// ```
#[derive(Debug)]
pub struct CssUniversalResetMetadata {
    _private: (),
}
impl CssUniversalResetMetadata {
    /// Reports explicit schema exclusions without selecting cascade winners.
    #[must_use]
    pub fn excludes(&self, property: CssPropertyNameRef<'_>) -> bool {
        universal_excludes(property)
    }
}
