//! Numeric and temporal input content limits and step, from HTML #the-min-and-max-attributes, #the-step-attribute and selected state definitions.

pub use crate::internal::attributes::input_bounds::{
    HtmlInputNumericAttributeReport, HtmlInputNumericAttributeValue, HtmlInputNumericAuthorState,
    HtmlInputNumericBoundOrigin, HtmlInputNumericCommonReport, HtmlInputNumericConstraints,
    HtmlInputNumericConstraintsError, HtmlInputNumericConstraintsOutcome,
    HtmlInputNumericConstraintsReport, HtmlInputNumericContent, HtmlInputNumericLimitAuthorState,
    HtmlInputNumericStep, HtmlInputNumericStepBaseOrigin, HtmlInputNumericStepOrigin,
    HtmlInputStep, HtmlInputTemporalConstraints, HtmlNonPositiveInputStepError,
    select_input_numeric_constraints,
};
