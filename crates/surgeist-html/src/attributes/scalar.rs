//! Exact numeric content values and submitter keyword overrides.
//!
//! These clauses do not implement WebIDL getter conversion, focus navigation,
//! canvas allocation, textarea rendering, or effective form-owner selection.

use super::*;
use crate::{
    HtmlInteger, HtmlNonNegativeInteger, HtmlValueAuthorState, HtmlValueFailure,
    HtmlValueRecoveryState, HtmlValueReport,
};

/// Numeric-value author conformance, separate from prefix recovery/defaults.
/// This does not establish additional element or tree-context conformance.
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
    /// Textarea rows/cols, select size and positive span rules require authored positivity.
    ZeroMustBePositive,
    /// A whole valid integer exceeds this attribute's source-defined maximum.
    MaximumExceeded {
        maximum: u16,
    },
}

/// Why the selected attribute clause produced its intrinsic content value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNumericAttributeValueOrigin {
    Recovered,
    MissingDefault,
    ParseFailureDefault,
    /// Successful recovery produced zero, which rows/cols and positive span rules default.
    ZeroDefault,
    /// Successful recovery exceeded the source cap; common retains its exact value.
    Clamped,
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
    /// Returns the selected intrinsic processing clause, independently of author syntax.
    pub fn processing_source_clause(&self) -> &'static str {
        match self.attribute() {
            KnownHtmlAttribute::Tabindex => source!("tabindex-value"),
            KnownHtmlAttribute::Colspan
            | KnownHtmlAttribute::Rowspan
            | KnownHtmlAttribute::Span => {
                source!("forming-a-table")
            }
            _ => self.source_clause(),
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

const SELECT_SIZE: RuleMetadata = RuleMetadata {
    attribute: KnownHtmlAttribute::Size,
    rows: &[143],
    source: source!("attr-select-size"),
    context: Context::Elements(&[KnownHtmlElement::Select]),
    vocabulary_source: None,
};

// Bounded span author constraints and recovery caps share a source-owned rule.
#[derive(Clone, Copy)]
struct BoundedSpanRule {
    metadata: RuleMetadata,
    positive: bool,
    maximum: u16,
}

const CELL_COLSPAN: BoundedSpanRule = BoundedSpanRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Colspan,
        rows: &[27],
        source: source!("attr-tdth-colspan"),
        context: Context::Elements(&[KnownHtmlElement::Td, KnownHtmlElement::Th]),
        vocabulary_source: None,
    },
    positive: true,
    maximum: 1000,
};

const CELL_ROWSPAN: BoundedSpanRule = BoundedSpanRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Rowspan,
        rows: &[132],
        source: source!("attr-tdth-rowspan"),
        context: Context::Elements(&[KnownHtmlElement::Td, KnownHtmlElement::Th]),
        vocabulary_source: None,
    },
    positive: false,
    maximum: 65534,
};

const COL_SPAN: BoundedSpanRule = BoundedSpanRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Span,
        rows: &[147],
        source: source!("attr-col-span"),
        context: Context::Elements(&[KnownHtmlElement::Col]),
        vocabulary_source: None,
    },
    positive: true,
    maximum: 1000,
};

const COLGROUP_SPAN: BoundedSpanRule = BoundedSpanRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Span,
        rows: &[147],
        source: source!("attr-colgroup-span"),
        context: Context::Elements(&[KnownHtmlElement::Colgroup]),
        vocabulary_source: None,
    },
    positive: true,
    maximum: 1000,
};

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

// A failed u128 projection proves the exact nonnegative magnitude exceeds
// the selected span cap; it is neither a syntax failure nor a processing error.
fn capped_nonnegative(value: &HtmlNonNegativeInteger, maximum: u16) -> (u16, bool) {
    match value.to_u128() {
        Ok(value) if value <= u128::from(maximum) => (value as u16, false),
        _ => (maximum, true),
    }
}

fn nonnegative_constraint(
    value: &HtmlNonNegativeInteger,
    positive: bool,
    maximum: Option<u16>,
) -> Result<(), HtmlNumericAttributeConformanceError> {
    if positive && value.is_zero() {
        return Err(HtmlNumericAttributeConformanceError::ZeroMustBePositive);
    }
    if let Some(maximum) = maximum
        && capped_nonnegative(value, maximum).1
    {
        return Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum });
    }
    Ok(())
}

fn nonnegative_author(
    common: &HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
    positive: bool,
) -> HtmlNumericAttributeAuthorState {
    constrained_nonnegative_author(common, positive, None)
}

fn constrained_nonnegative_author(
    common: &HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
    positive: bool,
    maximum: Option<u16>,
) -> HtmlNumericAttributeAuthorState {
    if let HtmlValueAuthorState::Valid(value) = common.author()
        && let Err(error) = nonnegative_constraint(value, positive, maximum)
    {
        return HtmlNumericAttributeAuthorState::Nonconforming(error);
    }
    numeric_author(common)
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
        let author = nonnegative_author(&common, rule.positive);
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

    fn bounded_span(
        &self,
        rule: BoundedSpanRule,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>, HtmlValueAdmissionError>
    {
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
        let author = constrained_nonnegative_author(&common, rule.positive, Some(rule.maximum));
        let (value, origin) = match common.recovery() {
            HtmlValueRecoveryState::Value(value) if rule.positive && value.is_zero() => {
                (1, HtmlNumericAttributeValueOrigin::ZeroDefault)
            }
            HtmlValueRecoveryState::Value(value) => {
                let (value, clamped) = capped_nonnegative(value, rule.maximum);
                (
                    value,
                    if clamped {
                        HtmlNumericAttributeValueOrigin::Clamped
                    } else {
                        HtmlNumericAttributeValueOrigin::Recovered
                    },
                )
            }
            HtmlValueRecoveryState::Missing => (1, HtmlNumericAttributeValueOrigin::MissingDefault),
            HtmlValueRecoveryState::Failed(_) => {
                (1, HtmlNumericAttributeValueOrigin::ParseFailureDefault)
            }
        };
        attribute.intrinsic = Some(value);
        Ok(HtmlNumericAttributeReport {
            attribute,
            common: Some(common),
            author,
            origin: Some(origin),
        })
    }

    /// Interprets exact HTML td/th colspan content: absent, failed or zero
    /// recovery selects 1; larger recovered quantities are capped at 1000.
    /// Author syntax independently requires an integer in 1..=1000. This does
    /// not establish overlap conformance or reflect a WebIDL getter.
    pub fn cell_colspan(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>, HtmlValueAdmissionError>
    {
        self.bounded_span(CELL_COLSPAN, element, raw)
    }

    /// Interprets exact HTML td/th rowspan content: absent or failed recovery
    /// selects 1; successful recovery is capped at 65534 and preserves zero.
    /// Zero symbolically means all remaining rows in the row group, not a
    /// resolved height or the table algorithm's initial working height 1.
    /// Author syntax requires an integer in 0..=65534. Overlap conformance,
    /// actual table formation and WebIDL reflection are separate operations.
    pub fn cell_rowspan(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>, HtmlValueAdmissionError>
    {
        self.bounded_span(CELL_ROWSPAN, element, raw)
    }

    /// Interprets exact HTML col span numeric content. Author syntax requires
    /// an integer in 1..=1000; absent, failed or zero recovery selects 1 and
    /// larger recovered quantities are capped at 1000. This content quantity
    /// does not establish the col element's parent or actual table use and is
    /// separate from WebIDL reflection and CSS table layout.
    pub fn col_span(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>, HtmlValueAdmissionError>
    {
        self.bounded_span(COL_SPAN, element, raw)
    }

    /// Interprets only exact HTML colgroup span's numeric content value.
    /// Numeric-value author syntax requires an integer in 1..=1000; absent,
    /// failed or zero recovery selects 1 and larger recovery is capped at 1000.
    /// Conforming here certifies only that numeric value: it does not establish
    /// the source restriction requiring no col elements or the group's content
    /// model. The content quantity is retained independently of whether table
    /// formation uses it; actual col children make that algorithm ignore the
    /// group's own span. No child evidence, group width or IDL getter is inferred.
    pub fn colgroup_span(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>, HtmlValueAdmissionError>
    {
        self.bounded_span(COLGROUP_SPAN, element, raw)
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
    metadata: RuleMetadata,
    positive: bool,
) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
    nonnegative_constraint(&value, positive, None)?;
    Ok(Attribute::new(
        metadata.attribute.qualified_name(),
        value.decimal_digits(),
    ))
}

fn bounded_span_author_attribute(
    value: HtmlNonNegativeInteger,
    rule: BoundedSpanRule,
) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
    nonnegative_constraint(&value, rule.positive, Some(rule.maximum))?;
    Ok(Attribute::new(
        rule.metadata.attribute.qualified_name(),
        value.decimal_digits(),
    ))
}

impl HtmlNonNegativeInteger {
    /// Constructs canonical span numeric syntax in 1..=1000 for col/colgroup.
    /// Zero and exact larger magnitudes are rejected rather than clamped.
    /// This establishes intrinsic numeric conformance only, not the colgroup
    /// child restriction/content model or the element's actual table use.
    pub fn into_column_span_attribute(
        self,
    ) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
        bounded_span_author_attribute(self, COL_SPAN)
    }

    /// Constructs canonical conforming cell colspan in 1..=1000.
    /// Exact larger magnitudes are rejected, never clamped or called malformed.
    pub fn into_cell_colspan_attribute(
        self,
    ) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
        bounded_span_author_attribute(self, CELL_COLSPAN)
    }

    /// Constructs canonical conforming cell rowspan in 0..=65534.
    /// Zero retains the source meaning of all remaining rows in the row group.
    pub fn into_cell_rowspan_attribute(
        self,
    ) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
        bounded_span_author_attribute(self, CELL_ROWSPAN)
    }

    /// Constructs conforming select size syntax. Authored zero is rejected,
    /// independently of the source processing algorithm's recovered zero.
    pub fn into_select_size_attribute(
        self,
    ) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
        nonnegative_author_attribute(self, SELECT_SIZE, true)
    }

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
        nonnegative_author_attribute(self, TEXTAREA_COLS.metadata, TEXTAREA_COLS.positive)
    }
    /// Rejects authored zero; larger exact values have no imposed machine bound.
    pub fn into_textarea_rows_attribute(
        self,
    ) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
        nonnegative_author_attribute(self, TEXTAREA_ROWS.metadata, TEXTAREA_ROWS.positive)
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

/// The intrinsic HTML display-size concept, not the size IDL getter (whose
/// ReflectDefault is zero), rendered geometry, or current selection state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectDisplaySize<'a> {
    Value {
        value: &'a HtmlNonNegativeInteger,
        origin: HtmlNumericAttributeValueOrigin,
    },
    Unapplied(HtmlAttributeApplicability),
    /// A missing/failed size needs actual multiple presence. A supplied wrong
    /// attribute role cannot establish that presence or count as absence.
    MultipleRoleUnavailable(HtmlAttributeApplicability),
}

/// Exact select size and multiple content records with separate author checks
/// and source-defined display size (HTML #attr-select-size/#concept-select-size).
///
/// Each raw slot is admitted through the interpreter's finite UTF-16 unit cap
/// before name/grammar interpretation. Derived integers and reports use ordinary
/// Rust allocation. This is not parser/captured-session resource accounting,
/// cancellation, WebIDL projection, rendering or selection execution.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlSelectSizeReport {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>>,
    author: HtmlNumericAttributeAuthorState,
    multiple: HtmlAttributeReport<bool>,
    fallback: Option<(HtmlNonNegativeInteger, HtmlNumericAttributeValueOrigin)>,
}
impl HtmlSelectSizeReport {
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
    pub fn processing_source_clause(&self) -> &'static str {
        source!("concept-select-size")
    }
    /// Returns size's raw units; multiple retains its own count in multiple().
    pub fn input_code_units(&self) -> usize {
        self.attribute.input_code_units()
    }
    /// None means size was unapplied. Applicable absence has a Missing report.
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>> {
        self.common.as_ref()
    }
    pub fn author(&self) -> HtmlNumericAttributeAuthorState {
        self.author
    }
    /// Actual content presence and independent Boolean spelling conformance.
    pub fn multiple(&self) -> &HtmlAttributeReport<bool> {
        &self.multiple
    }
    /// Successful parsing, including zero, does not require multiple evidence.
    /// Only missing/failed parsing consults that evidence for the 1/4 fallback.
    pub fn display_size(&self) -> HtmlSelectDisplaySize<'_> {
        let Some(common) = &self.common else {
            return HtmlSelectDisplaySize::Unapplied(self.applicability());
        };
        if let HtmlValueRecoveryState::Value(value) = common.recovery() {
            return HtmlSelectDisplaySize::Value {
                value,
                origin: HtmlNumericAttributeValueOrigin::Recovered,
            };
        }
        match &self.fallback {
            Some((value, origin)) => HtmlSelectDisplaySize::Value {
                value,
                origin: *origin,
            },
            None => HtmlSelectDisplaySize::MultipleRoleUnavailable(self.multiple.applicability()),
        }
    }
}
impl HtmlValueInterpreter {
    /// Interprets only exact HTML `select[size]`, with actual multiple content
    /// evidence. None establishes absence; a wrong supplied role is not absence.
    /// Both raw slots are admitted before applicability, even if recovery makes
    /// multiple unnecessary for the display-size result.
    pub fn select_size(
        &self,
        element: &QualifiedName,
        size: Option<&Attribute>,
        multiple: Option<&Attribute>,
    ) -> Result<HtmlSelectSizeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(size.map(Attribute::value))?;
        self.admit_attribute_raw(multiple.map(Attribute::value))?;
        let attribute = report(SELECT_SIZE, element, size);
        let multiple = self.boolean_attribute(HtmlBooleanAttribute::Multiple, element, multiple)?;
        let common = if attribute.applicability() == HtmlAttributeApplicability::Applicable {
            Some(self.nonnegative_integer(size.map(Attribute::value))?)
        } else {
            None
        };
        let author = common
            .as_ref()
            .map_or(HtmlNumericAttributeAuthorState::Unapplied, |common| {
                nonnegative_author(common, true)
            });
        let fallback = common.as_ref().and_then(|common| {
            let origin = match common.recovery() {
                HtmlValueRecoveryState::Missing => HtmlNumericAttributeValueOrigin::MissingDefault,
                HtmlValueRecoveryState::Failed(_) => {
                    HtmlNumericAttributeValueOrigin::ParseFailureDefault
                }
                HtmlValueRecoveryState::Value(_) => return None,
            };
            multiple
                .intrinsic()
                .map(|present| ((if *present { 4_u128 } else { 1_u128 }).into(), origin))
        });
        Ok(HtmlSelectSizeReport {
            attribute,
            common,
            author,
            multiple,
            fallback,
        })
    }
}
