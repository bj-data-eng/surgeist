use crate::declaration::{CssCustomPropertyName, CssGlobalKeyword, CssSubstitutionDependentValue};
use crate::property::CssKnownProperty;

/// A borrowed authored property-name view derived from a declaration body.
///
/// The known branch uses canonical generated identity, SVG has its independent terminal,
/// and the custom branch preserves its
/// case-sensitive name. The view stores no parallel identity and performs no cascade lookup.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssPropertyNameRef<'a> {
    /// The independent SVG terminal, selected explicitly without a canonical schema target.
    SvgGlyphOrientationVertical,
    /// A canonical schema-generated known property identity.
    Known(CssKnownProperty),
    /// A borrowed case-sensitive custom property name.
    Custom(&'a CssCustomPropertyName),
}

/// The shared intrinsic definition of the open custom-property family.
///
/// Every validated [`CssCustomPropertyName`] returns the same record. Its facts follow
/// [CSS Variables 1
/// §2](https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#defining-variables).
/// This metadata describes the family without performing substitution, inheritance,
/// selector or media matching, or animation sampling.
#[derive(Debug)]
pub struct CssCustomPropertyMetadata {
    pub(crate) source: crate::conformance::CssSpecificationSource,
}

/// The guaranteed-invalid intrinsic initial value of a custom property.
///
/// Obtain this marker from [`CssCustomPropertyMetadata::initial_value`]. It describes
/// the family definition, not an authored occurrence or a computed-value instance.
/// An authored empty token value remains distinct from an authored `initial` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGuaranteedInvalidInitial {
    _private: (),
}

impl CssCustomPropertyMetadata {
    /// Returns the selected specification source defining these intrinsic facts.
    #[must_use]
    pub const fn source(&self) -> crate::conformance::CssSpecificationSource {
        self.source
    }

    /// Returns whether the family accepts the declaration-value grammar.
    /// Actual token validation belongs to the declaration and property-value fronts.
    #[must_use]
    pub const fn accepts_declaration_values(&self) -> bool {
        true
    }

    /// Returns whether an empty authored token value is valid.
    #[must_use]
    pub const fn allows_empty_value(&self) -> bool {
        true
    }

    /// Returns the typed intrinsic initial marker defined by
    /// [CSS Variables 1
    /// §2.2](https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#guaranteed-invalid).
    #[must_use]
    pub const fn initial_value(&self) -> CssGuaranteedInvalidInitial {
        CssGuaranteedInvalidInitial { _private: () }
    }

    /// Returns whether the family applies to every element and pseudo-element,
    /// including pseudo-elements with restricted property lists.
    #[must_use]
    pub const fn applies_to_all_elements_and_pseudo_elements(&self) -> bool {
        true
    }

    /// Returns whether the family is inherited by default.
    #[must_use]
    pub const fn inherited_by_default(&self) -> bool {
        true
    }

    /// Returns whether the family defines a percentage basis.
    /// The definition's n/a does not prohibit percentage tokens in authored values.
    #[must_use]
    pub const fn has_percentage_basis(&self) -> bool {
        false
    }

    /// Returns whether the computed contract is the specified tokens after variable
    /// substitution, or the guaranteed-invalid value. This does not substitute tokens
    /// or construct a computed-value payload.
    #[must_use]
    pub const fn computed_value_is_tokens_or_guaranteed_invalid(&self) -> bool {
        true
    }

    /// Returns whether canonical order follows the authored value grammar.
    #[must_use]
    pub const fn canonical_order_is_grammar_order(&self) -> bool {
        true
    }

    /// Returns whether the family's intrinsic animation classification is discrete.
    #[must_use]
    pub const fn animation_is_discrete(&self) -> bool {
        true
    }

    /// Returns whether the family supports all media, including nonvisual media.
    #[must_use]
    pub const fn supports_all_media(&self) -> bool {
        true
    }
}

/// The authored declared-value domain for the `all` property.
///
/// Unlike ordinary known properties, `all` has no typed-value branch: it can retain only a
/// CSS-wide keyword or syntax whose grammar depends on later substitution. It does not apply
/// cascade, perform substitution, or resolve the resulting values.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssAllDeclaredValue {
    /// A whole-value CSS-wide keyword.
    Global(CssGlobalKeyword),
    /// Authored syntax whose validity depends on later substitution.
    SubstitutionDependent(CssSubstitutionDependentValue),
}

impl CssAllDeclaredValue {
    /// Returns the symbolic CSS-wide keyword when present.
    #[must_use]
    pub const fn global(&self) -> Option<CssGlobalKeyword> {
        match self {
            Self::Global(keyword) => Some(*keyword),
            Self::SubstitutionDependent(_) => None,
        }
    }

    /// Returns the substitution-dependent authored value when present.
    #[must_use]
    pub const fn substitution_dependent(&self) -> Option<&CssSubstitutionDependentValue> {
        match self {
            Self::Global(_) => None,
            Self::SubstitutionDependent(value) => Some(value),
        }
    }
}
