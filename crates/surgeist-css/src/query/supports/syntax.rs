use crate::component::CssValueOrigin;
use crate::declaration::{CssDeclarationBody, CssImportance};
use crate::property::CssKnownDeclaration;
use crate::query::CssGeneralEnclosed;
use crate::rule::font_face::{CssFontFormat, CssFontTechHint};
use crate::source::CssSourcePosition;

/// A leaf of a supports condition: one test of the `<supports-condition>` grammar.
///
/// A condition is a [`CssCondition`](crate::query::CssCondition) over these leaves; its parenthesized layers,
/// negations and `and`/`or` lists are [`CssConditionKind`](crate::query::CssConditionKind) variants.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssSupportsLeaf {
    Declaration(Box<CssSupportsDeclaration>),
    Named(crate::query::supports::CssSupportsConditionName),
    Selector(CssSelector),
    FontTech(CssFontTechHint),
    FontFormat(CssFontFormat),
    AtRule(crate::query::supports::CssSupportsAtRule),
    GeneralEnclosed(CssGeneralEnclosed),
}

/// One syntactically valid declaration test inside a supports condition.
///
/// Unknown properties and values outside the current property grammar remain
/// valid authored tests. `known()` is populated only when the existing schema
/// parser can provide its property-coupled typed declaration view.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsDeclaration {
    authored: Option<String>,
    property: String,
    importance: CssImportance,
    admitted: Option<SupportsAdmitted>,
    lexical: crate::query::supports::construction::SupportsLexical,
    property_index: usize,
    value_range: std::ops::Range<usize>,
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
            authored,
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
    #[must_use]
    pub fn authored(&self) -> Option<&str> {
        self.authored.as_deref()
    }
    #[must_use]
    pub fn property(&self) -> &str {
        &self.property
    }
    #[must_use]
    pub const fn importance(&self) -> CssImportance {
        self.importance
    }
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
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.property_component().origin()
    }
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
    #[must_use]
    pub fn components(&self) -> &[crate::component::CssComponentValue] {
        self.lexical.items()
    }
    #[must_use]
    pub fn property_component(&self) -> &crate::component::CssComponentValue {
        &self.components()[self.property_index]
    }
    #[must_use]
    pub fn value_components(&self) -> &[crate::component::CssComponentValue] {
        &self.components()[self.value_range.clone()]
    }
    pub(crate) fn lexical(&self) -> &crate::query::supports::construction::SupportsLexical {
        &self.lexical
    }
}
use crate::selector::CssSelector;
