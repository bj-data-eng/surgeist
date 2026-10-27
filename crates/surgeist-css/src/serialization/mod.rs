//! Serialization limits and errors, rule-graph input and lexical serializers.
//!
//! # Ordinary canonical number text
//!
//! Shared ordinary number, percentage and dimension serializers emit shortest base-ten text
//! with at most six fractional places, no exponent, and no negative rounded zero. Exact unit
//! conversion precedes rounding. The selected decimal halfway policy rounds away from zero;
//! CSSOM does not specify that direction. Original checked tokens, admission ranges, equality
//! and diagnostic origins stay exact. Each ordinary scalar keeps one input and one projection
//! visit; byte limits count its actual rounded text and unit within the cumulative writer.
//!
//! # Calculated canonical number text
//!
//! Finite non-color, declared relative-color and ordinary-origin calculation coefficients use
//! the same six-place fixed text policy, rounding the actual binary64 value after projection.
//! Symbols, units, arithmetic, signed-zero and exceptional-value behavior, and traversal costs
//! stay unchanged. Border-image
//! and scroll shorthands compare unrounded canonical components before selecting rounded text.
//! Relative calculations use this policy wherever embedded, including nested origins and mixes.
//! Ordinary origins preserve number/percentage dimensions, canonical angle units and explicit
//! unclamped alpha. Ordinary non-alpha component and hue calculations use this policy when
//! their projected result remains context-dependent, including inside mixes. The existing
//! projection result selects formatting after percentage scaling and angle conversion.
//! Retained calculated alpha and mix weights also use the shared coefficient policy.
//! Contextual alpha and calculated alpha outside ordinary RGB/HSL/HWB stay explicit and
//! unclamped; percentage alpha divides by 100 before formatting, while actual Origin alpha
//! preserves its dimension. Ordinary RGB/HSL/HWB scalar calculated alpha in standalone
//! colors and Mix children independently normalizes NaN to zero and clamps to 0..1,
//! omitting exact unity before rounding. This
//! follows the recorded WebKit interpretation of conflicting Color 4 phase clauses.
//! Calculated weights keep
//! percentages and their calculation wrapper, preventing omitted sibling shares from being
//! filled even when they emit `calc(50%)`. These rules apply to nested mixes and ordinary
//! colors inside a mix used as an origin. Resolved ordinary RGB/HSL/HWB non-alpha slots
//! emit scalars, including resolved siblings of contextual slots. Finite calculated scalars
//! round their original post-scale binary64 bits to six fractional places, nearest with ties
//! toward positive infinity; HSL/HWB conversion formats the final clipped RGB channels.
//! Direct literals keep exact decimal arithmetic. Captures must succeed before finalization
//! and keep their scratch costs even when final text is shorter. Retained alpha, weight and
//! contextual component captures count rounded scratch text within the cumulative budget.
//! Any directly missing ordinary RGB/HSL/HWB component, including alpha, selects a form
//! preserving `none`; omitted or calculated alpha alone does not. RGB uses normalized
//! `color(srgb ...)`, while HSL/HWB retain their named functions. Direct HSL/HWB channels
//! emit percentages and a bare degree hue. RGB clamps to its output domain; HSL saturation
//! has a zero minimum, including resolved negative/NaN calculations under the selected
//! frozen WebKit interpretation. Named lightness and HWB percentages remain unbounded,
//! with NaN becoming zero and dimensional infinities remaining valid calculations.
//! Numeric hue normalizes modulo 360 before conversion or named-form rounding.
//! Ordinary mix children follow the same rule, with cumulative limits on the actual text.
//! Direct Lab/LCH lightness in standalone colors and ordinary mix arguments clamps
//! to 0..100, while Oklab/Oklch lightness
//! clamps to 0..1 after exact percentage conversion. Direct LCH/Oklch chroma has a zero
//! minimum and no upper bound; the signed a/b axes remain unbounded. In-range literals
//! keep exact decimal text. A clipped direct endpoint is selected from borrowed lexical
//! metadata before rational allocation or unnecessary decimal expansion, including
//! finite authored exponents beyond i128. Each selected direct slot adds one logical
//! projection visit. Calculations retain their wrappers and capture costs; origin and
//! relative channels remain unclamped. These direct bounds apply to ordinary mix
//! arguments, including nested mixes and a mix used as an origin color.
//! Unbounded and in-range values still obey typed output and work limits.
//! Integer calculation text follows the finite number policy without
//! computed integer rounding; ordinary integer
//! literals retain exact digits. Arithmetic precision and range remain unfinished.

pub(crate) mod common_serialization;
pub(crate) mod cssom_rule_serialization;
pub(crate) mod edited_rule;
pub(crate) mod exact_decimal;
pub(crate) mod numeric_formatting;
pub(crate) mod pending_serialization;
pub(crate) mod serialization_escaping;
#[cfg(test)]
pub(crate) mod specified_provider_composition_tests;
pub(crate) mod specified_rule_serialization;
pub(crate) mod specified_serialization;

#[doc(inline)]
pub use common_serialization::{
    serialize_css_comma_separated_list, serialize_css_identifier, serialize_css_string,
    serialize_css_whitespace_separated_list,
};
#[doc(inline)]
pub use cssom_rule_serialization::{
    CssRuleCssomFormat, CssRuleCssomKind, CssRuleSerializationError,
};
#[doc(inline)]
pub use edited_rule::{CssGroupPreludeRef, CssRuleGraphInput, CssRuleSerializationLimits};
#[doc(inline)]
pub use specified_serialization::{CssSerializationError, CssSerializationLimits};
