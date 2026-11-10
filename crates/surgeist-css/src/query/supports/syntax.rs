use crate::component::CssValueOrigin;
use crate::declaration::{CssDeclarationBody, CssImportance};
use crate::property::CssKnownDeclaration;
use crate::query::CssGeneralEnclosed;
use crate::rule::font_face::{CssFontFormat, CssFontTechHint};
use crate::source::CssSourcePosition;
use crate::source::Provenance;

/// A leaf of a supports condition: one test of the `<supports-condition>` grammar.
///
/// A condition is a [`CssCondition`](crate::query::CssCondition) over these leaves; its parenthesized layers,
/// negations and `and`/`or` lists are [`CssConditionKind`](crate::query::CssConditionKind) variants.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssSupportsLeaf {
    /// A declaration test such as `(display: grid)`, which is valid syntax even when the property or value is unknown.
    Declaration(Box<CssSupportsDeclaration>),
    /// A reference to a named supports condition, such as `(--my-test)`.
    Named(crate::query::supports::CssSupportsConditionName),
    /// A `selector()` test.
    Selector(CssSelector),
    /// A `font-tech()` test.
    FontTech(CssFontTechHint),
    /// A `font-format()` test.
    FontFormat(CssFontFormat),
    /// An `at-rule()` test, such as `at-rule(@media)`.
    AtRule(crate::query::supports::CssSupportsAtRule),
    /// A `<general-enclosed>` production that is not a recognized supports test.
    GeneralEnclosed(CssGeneralEnclosed),
}

/// One syntactically valid declaration test inside a supports condition.
///
/// Unknown properties and values outside the current property grammar remain
/// valid authored tests. `known()` is populated only when the existing schema
/// parser can provide its property-coupled typed declaration view.
#[derive(Clone, Debug)]
pub struct CssSupportsDeclaration {
    authored: Provenance<Option<String>>,
    property: String,
    importance: CssImportance,
    admitted: Option<SupportsAdmitted>,
    lexical: crate::query::supports::construction::SupportsLexical,
    property_index: usize,
    value_range: std::ops::Range<usize>,
}
// The property, importance and admitted typed declaration are the value. The lexical region
// retains the authored tokens and their indices; it is the value only when no typed
// declaration was admitted, and then its text is the value, compared as the declaration path
// compares a custom property's value.
impl PartialEq for CssSupportsDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.property == other.property
            && self.importance == other.importance
            && self.admitted == other.admitted
            && (self.admitted.is_some()
                || crate::component::values::verbatim_items_eq(
                    self.value_components(),
                    other.value_components(),
                ))
            && self.authored == other.authored
            && Provenance::new((&self.lexical, self.property_index, &self.value_range))
                == Provenance::new((&other.lexical, other.property_index, &other.value_range))
    }
}
#[derive(Clone, Debug, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "admitted supports views retain their checked property values inline"
)]
enum SupportsAdmitted {
    Known(CssKnownDeclaration),
    SvgGlyph(crate::property::CssSvgGlyphOrientationVerticalDeclaration),
}
impl CssSupportsDeclaration {
    pub(crate) fn new(
        authored: Option<String>,
        property: String,
        importance: CssImportance,
        admitted: Option<CssDeclarationBody>,
        lexical: crate::query::supports::construction::SupportsLexical,
        property_index: usize,
        value_range: std::ops::Range<usize>,
    ) -> Self {
        Self {
            authored: authored.into(),
            property,
            importance,
            admitted: admitted.and_then(|body| match body {
                CssDeclarationBody::Known(value) => Some(SupportsAdmitted::Known(value)),
                CssDeclarationBody::SvgGlyphOrientationVertical(value) => {
                    Some(SupportsAdmitted::SvgGlyph(value))
                }
                CssDeclarationBody::Custom(_) => None,
            }),
            lexical,
            property_index,
            value_range,
        }
    }
    /// Returns the original source text of the declaration test, or `None` when it was not parsed from source.
    #[must_use]
    pub fn authored(&self) -> Option<&str> {
        self.authored.inner().as_deref()
    }
    /// Returns the property name text as authored.
    #[must_use]
    pub fn property(&self) -> &str {
        &self.property
    }
    /// Returns whether the declaration carries `!important`.
    #[must_use]
    pub const fn importance(&self) -> CssImportance {
        self.importance
    }
    /// Returns the typed declaration when the property and value are known to the property schema, and otherwise `None`.
    #[must_use]
    pub const fn known(&self) -> Option<&CssKnownDeclaration> {
        match &self.admitted {
            Some(SupportsAdmitted::Known(value)) => Some(value),
            _ => None,
        }
    }
    /// Borrows the admitted independent SVG definition; `known()` returns None for it.
    #[must_use]
    pub const fn svg_glyph_orientation_vertical(
        &self,
    ) -> Option<&crate::property::CssSvgGlyphOrientationVerticalDeclaration> {
        match &self.admitted {
            Some(SupportsAdmitted::SvgGlyph(value)) => Some(value),
            _ => None,
        }
    }
    /// Returns the origin of the property-name component.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.property_component().origin()
    }
    /// Returns the source position of the property name, or `None` when its origin was not parsed from source.
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
    /// Returns all components of the declaration test.
    #[must_use]
    pub fn components(&self) -> &[crate::component::CssComponentValue] {
        self.lexical.items()
    }
    /// Returns the component holding the property name.
    #[must_use]
    pub fn property_component(&self) -> &crate::component::CssComponentValue {
        &self.components()[self.property_index]
    }
    /// Returns the components of the declaration value.
    #[must_use]
    pub fn value_components(&self) -> &[crate::component::CssComponentValue] {
        &self.components()[self.value_range.clone()]
    }
    pub(crate) fn lexical(&self) -> &crate::query::supports::construction::SupportsLexical {
        &self.lexical
    }
}
use crate::selector::CssSelector;
