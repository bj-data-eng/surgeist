//! Exact numeric content values and submitter keyword overrides.
//!
//! These clauses do not implement WebIDL getter conversion, focus navigation,
//! canvas allocation, textarea rendering, or effective form-owner selection.

use super::*;
use crate::{
    HtmlInteger, HtmlNonNegativeInteger, HtmlValueAuthorState, HtmlValueFailure,
    HtmlValueRecoveryState, HtmlValueReport,
};

/// Attribute-level author conformance, separate from prefix recovery/defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNumericAttributeAuthorState {
    Missing,
    Conforming,
    Nonconforming(HtmlNumericAttributeConformanceError),
    Unapplied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNumericAttributeConformanceError {
    CommonValue(HtmlValueFailure),
    /// Textarea rows/cols require a strictly positive authored integer.
    ZeroMustBePositive,
}

/// Why the selected attribute clause produced its intrinsic content value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNumericAttributeValueOrigin {
    Recovered,
    MissingDefault,
    ParseFailureDefault,
    /// Successful nonnegative recovery produced zero, which rows/cols reject.
    ZeroDefault,
}

/// Exact tabindex content value; source-null is distinct from an unapplied rule.
/// This is not the signed-long, element-dependent tabIndex IDL getter result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTabindexValue {
    Integer(HtmlInteger),
    Null,
}

/// Numeric attribute processing with the shared exact name/raw/source envelope.
///
/// The common report retains whole-value authored syntax and independent prefix
/// recovery. The attribute author state adds its own constraint; intrinsic and
/// origin apply the selected default without rewriting that common evidence.
/// Unapplied names retain raw input but do not run the selected value grammar.
/// Raw-unit admission has the same standalone limits as HtmlValueInterpreter;
/// it is not a parser work/allocation or cancellation guarantee.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlNumericAttributeReport<A, T> {
    attribute: HtmlAttributeReport<T>,
    common: Option<HtmlValueReport<A, A>>,
    author: HtmlNumericAttributeAuthorState,
    origin: Option<HtmlNumericAttributeValueOrigin>,
}

impl<A, T> HtmlNumericAttributeReport<A, T> {
    pub fn element(&self) -> &QualifiedName {
        self.attribute.element()
    }
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.attribute.raw_attribute()
    }
    pub fn raw(&self) -> Option<&DocumentString> {
        self.attribute.raw()
    }
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.attribute.observation()
    }
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.attribute.applicability()
    }
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.attribute.attribute()
    }
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.attribute.inventory_rows()
    }
    pub fn source_clause(&self) -> &'static str {
        self.attribute.source_clause()
    }
    /// Applicable tabindex additionally applies this exact source-null clause.
    pub fn processing_source_clause(&self) -> &'static str {
        if self.attribute() == KnownHtmlAttribute::Tabindex {
            source!("tabindex-value")
        } else {
            self.source_clause()
        }
    }
    pub fn input_code_units(&self) -> usize {
        self.attribute.input_code_units()
    }
    /// None means the selected attribute grammar was unapplied, not missing raw.
    pub fn common(&self) -> Option<&HtmlValueReport<A, A>> {
        self.common.as_ref()
    }
    pub fn author(&self) -> HtmlNumericAttributeAuthorState {
        self.author
    }
    pub fn intrinsic(&self) -> Option<&T> {
        self.attribute.intrinsic()
    }
    pub fn origin(&self) -> Option<HtmlNumericAttributeValueOrigin> {
        self.origin
    }
}

#[derive(Clone, Copy)]
struct NonnegativeRule {
    metadata: RuleMetadata,
    default: u128,
    positive: bool,
}

macro_rules! nonnegative_rule {
    ($name:ident, $attribute:ident, $element:ident, $row:literal, $anchor:literal, $default:literal, $positive:literal) => {
        const $name: NonnegativeRule = NonnegativeRule {
            metadata: RuleMetadata {
                attribute: KnownHtmlAttribute::$attribute,
                rows: &[$row],
                source: source!($anchor),
                context: Context::Elements(&[KnownHtmlElement::$element]),
                vocabulary_source: None,
            },
            default: $default,
            positive: $positive,
        };
    };
}

// One declaration owns applicability, defaults, constraints and source rows.
nonnegative_rule!(
    CANVAS_WIDTH,
    Width,
    Canvas,
    178,
    "attr-canvas-width",
    300,
    false
);
nonnegative_rule!(
    CANVAS_HEIGHT,
    Height,
    Canvas,
    64,
    "attr-canvas-height",
    150,
    false
);
nonnegative_rule!(
    TEXTAREA_COLS,
    Cols,
    Textarea,
    26,
    "attr-textarea-cols",
    20,
    true
);
nonnegative_rule!(
    TEXTAREA_ROWS,
    Rows,
    Textarea,
    131,
    "attr-textarea-rows",
    2,
    true
);

const TABINDEX: RuleMetadata = RuleMetadata {
    attribute: KnownHtmlAttribute::Tabindex,
    rows: &[156],
    source: source!("attr-tabindex"),
    context: Context::Global,
    vocabulary_source: None,
};

// These override records reuse the defining form keywords, but have no missing
// default. They never select the form owner's effective submission settings.
const FORMMETHOD: EnumRule<HtmlFormMethod> = EnumRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Formmethod,
        rows: &[58],
        source: source!("attr-fs-formmethod"),
        context: Context::SubmitterOverride,
        vocabulary_source: None,
    },
    keywords: FORM_METHOD.keywords,
    missing: None,
    invalid: FORM_METHOD.invalid,
    empty: None,
    exact_keywords: false,
};

const FORMENCTYPE: EnumRule<HtmlFormEnctype> = EnumRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Formenctype,
        rows: &[57],
        source: source!("attr-fs-formenctype"),
        context: Context::SubmitterOverride,
        vocabulary_source: None,
    },
    keywords: FORM_ENCTYPE.keywords,
    missing: None,
    invalid: FORM_ENCTYPE.invalid,
    empty: None,
    exact_keywords: false,
};

fn numeric_author<A>(common: &HtmlValueReport<A, A>) -> HtmlNumericAttributeAuthorState {
    match common.author() {
        HtmlValueAuthorState::Missing => HtmlNumericAttributeAuthorState::Missing,
        HtmlValueAuthorState::Valid(_) => HtmlNumericAttributeAuthorState::Conforming,
        HtmlValueAuthorState::Invalid(error) => HtmlNumericAttributeAuthorState::Nonconforming(
            HtmlNumericAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(*error)),
        ),
        HtmlValueAuthorState::Unrepresentable(error) => {
            HtmlNumericAttributeAuthorState::Nonconforming(
                HtmlNumericAttributeConformanceError::CommonValue(
                    HtmlValueFailure::Representation(*error),
                ),
            )
        }
        // Exact signed/nonnegative integer operations always define grammar.
        HtmlValueAuthorState::NotDefined => unreachable!("integer author grammar is defined"),
    }
}

impl HtmlValueInterpreter {
    fn nonnegative_attribute(
        &self,
        rule: NonnegativeRule,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<
        HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
        HtmlValueAdmissionError,
    > {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut attribute = report(rule.metadata, element, raw);
        if attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlNumericAttributeReport {
                attribute,
                common: None,
                author: HtmlNumericAttributeAuthorState::Unapplied,
                origin: None,
            });
        }
        let common = self.nonnegative_integer(raw.map(Attribute::value))?;
        let mut author = numeric_author(&common);
        if rule.positive
            && let HtmlValueAuthorState::Valid(value) = common.author()
            && value.is_zero()
        {
            author = HtmlNumericAttributeAuthorState::Nonconforming(
                HtmlNumericAttributeConformanceError::ZeroMustBePositive,
            );
        }
        let (value, origin) = match common.recovery() {
            HtmlValueRecoveryState::Value(value) if !rule.positive || !value.is_zero() => {
                (value.clone(), HtmlNumericAttributeValueOrigin::Recovered)
            }
            HtmlValueRecoveryState::Value(_) => (
                rule.default.into(),
                HtmlNumericAttributeValueOrigin::ZeroDefault,
            ),
            HtmlValueRecoveryState::Missing => (
                rule.default.into(),
                HtmlNumericAttributeValueOrigin::MissingDefault,
            ),
            HtmlValueRecoveryState::Failed(_) => (
                rule.default.into(),
                HtmlNumericAttributeValueOrigin::ParseFailureDefault,
            ),
        };
        attribute.intrinsic = Some(value);
        Ok(HtmlNumericAttributeReport {
            attribute,
            common: Some(common),
            author,
            origin: Some(origin),
        })
    }

    /// Interprets exact canvas bitmap width content, including valid zero.
    /// Missing or failed nonnegative recovery selects 300, without bitmap effects.
    pub fn canvas_width(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<
        HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
        HtmlValueAdmissionError,
    > {
        self.nonnegative_attribute(CANVAS_WIDTH, element, raw)
    }

    /// Interprets exact canvas bitmap height content; missing/failed recovery is 150.
    /// This does not apply WebIDL conversion or allocate a bitmap.
    pub fn canvas_height(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<
        HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
        HtmlValueAdmissionError,
    > {
        self.nonnegative_attribute(CANVAS_HEIGHT, element, raw)
    }

    /// Interprets textarea character width; missing, failure or zero selects 20.
    /// The exact positive content result is not an unsigned-long IDL projection.
    pub fn textarea_cols(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<
        HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
        HtmlValueAdmissionError,
    > {
        self.nonnegative_attribute(TEXTAREA_COLS, element, raw)
    }

    /// Interprets textarea character height; missing, failure or zero selects 2.
    /// This does not compute wrapping, rendering or unsigned-long IDL projection.
    pub fn textarea_rows(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<
        HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
        HtmlValueAdmissionError,
    > {
        self.nonnegative_attribute(TEXTAREA_ROWS, element, raw)
    }

    /// Interprets exact signed tabindex content; missing/failed recovery is Null.
    /// Does not implement tabIndex's signed-long range and element defaults,
    /// summary-parent qualification, focusability or focus navigation order.
    pub fn tabindex(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlNumericAttributeReport<HtmlInteger, HtmlTabindexValue>, HtmlValueAdmissionError>
    {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut attribute = report(TABINDEX, element, raw);
        if attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlNumericAttributeReport {
                attribute,
                common: None,
                author: HtmlNumericAttributeAuthorState::Unapplied,
                origin: None,
            });
        }
        let common = self.integer(raw.map(Attribute::value))?;
        let author = numeric_author(&common);
        let (value, origin) = match common.recovery() {
            HtmlValueRecoveryState::Value(value) => (
                HtmlTabindexValue::Integer(value.clone()),
                HtmlNumericAttributeValueOrigin::Recovered,
            ),
            HtmlValueRecoveryState::Missing => (
                HtmlTabindexValue::Null,
                HtmlNumericAttributeValueOrigin::MissingDefault,
            ),
            HtmlValueRecoveryState::Failed(_) => (
                HtmlTabindexValue::Null,
                HtmlNumericAttributeValueOrigin::ParseFailureDefault,
            ),
        };
        attribute.intrinsic = Some(value);
        Ok(HtmlNumericAttributeReport {
            attribute,
            common: Some(common),
            author,
            origin: Some(origin),
        })
    }
}

macro_rules! override_methods {
    ($ordinary:ident, $input:ident, $button:ident, $rule:ident, $type:ty) => {
        impl HtmlValueInterpreter {
            /// Interprets a submitter content override with no guessed submitter facts.
            /// Missing has no intrinsic state: effective form-owner fallback remains
            /// unresolved. A present value selects a keyword/default independently
            /// of unchecked author context, with no form submission effects.
            pub fn $ordinary(
                &self,
                element: &QualifiedName,
                raw: Option<&Attribute>,
            ) -> Result<HtmlAttributeReport<$type>, HtmlValueAdmissionError> {
                self.enumerated(&$rule, element, raw)
            }

            /// Interprets the override plus supplied actual input-state authorship.
            /// Forbidden state does not erase content; no form owner is selected.
            pub fn $input(
                &self,
                element: &QualifiedName,
                raw: Option<&Attribute>,
                state: HtmlInputType,
            ) -> Result<HtmlAttributeReport<$type>, HtmlValueAdmissionError> {
                self.enumerated_with_context(&$rule, element, raw, SuppliedContext::Input(state))
            }

            /// Interprets the override plus actual button submitter authorship.
            /// Auto eligibility is supplied, never inferred from type alone.
            pub fn $button(
                &self,
                element: &QualifiedName,
                raw: Option<&Attribute>,
                context: HtmlButtonSubmitterContext,
            ) -> Result<HtmlAttributeReport<$type>, HtmlValueAdmissionError> {
                self.enumerated_with_context(
                    &$rule,
                    element,
                    raw,
                    SuppliedContext::ButtonSubmitter(context),
                )
            }
        }
    };
}

override_methods!(
    formmethod,
    formmethod_for_input,
    formmethod_for_button,
    FORMMETHOD,
    HtmlFormMethod
);
override_methods!(
    formenctype,
    formenctype_for_input,
    formenctype_for_button,
    FORMENCTYPE,
    HtmlFormEnctype
);

fn nonnegative_author_attribute(
    value: HtmlNonNegativeInteger,
    rule: NonnegativeRule,
) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
    if rule.positive && value.is_zero() {
        return Err(HtmlNumericAttributeConformanceError::ZeroMustBePositive);
    }
    Ok(Attribute::new(
        rule.metadata.attribute.qualified_name(),
        value.decimal_digits(),
    ))
}

impl HtmlNonNegativeInteger {
    /// Constructs exact conforming canvas width syntax, including zero.
    pub fn into_canvas_width_attribute(self) -> Attribute {
        Attribute::new(
            CANVAS_WIDTH.metadata.attribute.qualified_name(),
            self.decimal_digits(),
        )
    }
    /// Constructs exact conforming canvas height syntax, including zero.
    pub fn into_canvas_height_attribute(self) -> Attribute {
        Attribute::new(
            CANVAS_HEIGHT.metadata.attribute.qualified_name(),
            self.decimal_digits(),
        )
    }
    /// Rejects authored zero; larger exact values have no imposed machine bound.
    pub fn into_textarea_cols_attribute(
        self,
    ) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
        nonnegative_author_attribute(self, TEXTAREA_COLS)
    }
    /// Rejects authored zero; larger exact values have no imposed machine bound.
    pub fn into_textarea_rows_attribute(
        self,
    ) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
        nonnegative_author_attribute(self, TEXTAREA_ROWS)
    }
}

impl HtmlInteger {
    /// Constructs canonical exact signed tabindex content syntax, not an IDL value.
    pub fn into_tabindex_attribute(self) -> Attribute {
        let mut value = String::new();
        if self.is_negative() {
            value.push('-');
        }
        value.push_str(self.magnitude().decimal_digits());
        Attribute::new(KnownHtmlAttribute::Tabindex.qualified_name(), value)
    }
}

impl HtmlFormMethod {
    /// Constructs a real override keyword; omission/fallback has no constructor.
    pub fn into_formmethod_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Formmethod.qualified_name(),
            self.canonical_keyword(),
        )
    }
}

impl HtmlFormEnctype {
    /// Constructs a real override keyword; omission/fallback has no constructor.
    pub fn into_formenctype_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Formenctype.qualified_name(),
            self.canonical_keyword(),
        )
    }
}
