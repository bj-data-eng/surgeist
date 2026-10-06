//! Values4 Appendix C permission belongs to exact named property owners.
use crate::numeric::NumericInputContext;
use crate::{CssFeatureId, CssKnownProperty, CssParserContext, CssParserMode};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.quirky-length")];

pub(super) fn property_context<'a>(
    property: CssKnownProperty,
    context: CssParserContext,
    numeric: &'a NumericInputContext<'a>,
) -> NumericInputContext<'a> {
    use CssKnownProperty::*;
    let eligible = matches!(
        property,
        BackgroundPosition
            | BorderSpacing
            | BorderTopWidth
            | BorderRightWidth
            | BorderBottomWidth
            | BorderLeftWidth
            | BorderWidth
            | Bottom
            | Clip
            | FontSize
            | Height
            | Left
            | LetterSpacing
            | MarginRight
            | MarginLeft
            | MarginTop
            | MarginBottom
            | Margin
            | MaxHeight
            | MaxWidth
            | MinHeight
            | MinWidth
            | PaddingTop
            | PaddingRight
            | PaddingBottom
            | PaddingLeft
            | Padding
            | Right
            | TextIndent
            | Top
            | VerticalAlign
            | Width
            | WordSpacing
    );
    if context.mode() == CssParserMode::Quirks && eligible {
        NumericInputContext::QuirkyLengths(numeric.ordinary())
    } else {
        // Only the actual property grants permission; referenced owners do not.
        match numeric.ordinary() {
            NumericInputContext::Parsed(source) => NumericInputContext::parsed(source),
            NumericInputContext::Components(values, serialized) => {
                NumericInputContext::components(values, serialized)
            }
            NumericInputContext::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
        }
    }
}
