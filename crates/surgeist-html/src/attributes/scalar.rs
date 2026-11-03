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
    /// The attribute is absent, which imposes no numeric requirement.
    Missing,
    /// The attribute is present and its whole value satisfies the grammar and every
    /// attribute-specific constraint.
    Conforming,
    /// The attribute is present but breaks a numeric requirement; the payload names which.
    Nonconforming(HtmlNumericAttributeConformanceError),
    /// The rule does not apply because the element or attribute name is not the one the rule
    /// selects.
    Unapplied,
}

/// Why a present numeric attribute value is nonconforming. The enum is `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlNumericAttributeConformanceError {
    /// The whole value is not a valid number of the expected kind; the payload is the common
    /// grammar failure.
    CommonValue(HtmlValueFailure),
    /// Textarea rows/cols, select size and positive span rules require authored positivity.
    ZeroMustBePositive,
    /// A whole valid integer exceeds this attribute's source-defined maximum.
    MaximumExceeded {
        /// The attribute's source-defined maximum.
        maximum: u16,
    },
}

impl std::fmt::Display for HtmlNumericAttributeConformanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommonValue(_) => f.write_str("numeric attribute value is not valid"),
            Self::ZeroMustBePositive => f.write_str("numeric attribute value must be positive"),
            Self::MaximumExceeded { maximum } => write!(
                f,
                "numeric attribute value exceeds its maximum of {maximum}"
            ),
        }
    }
}
impl std::error::Error for HtmlNumericAttributeConformanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CommonValue(error) => Some(error),
            _ => None,
        }
    }
}

/// Why the selected attribute clause produced its intrinsic content value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNumericAttributeValueOrigin {
    /// The value came from the standard's rules for parsing the attribute, possibly from a leading
    /// prefix of the text.
    Recovered,
    /// The attribute was absent, so the clause's missing-value default applies.
    MissingDefault,
    /// Parsing failed, so the clause's default for an unparsable value applies.
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
    /// The attribute is present and parses to this exact integer.
    Integer(HtmlInteger),
    /// The standard's null value: the attribute is absent or does not parse, which is distinct from
    /// an unapplied rule.
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
    /// The exact element name that was supplied, including any stored prefix.
    pub fn element(&self) -> &QualifiedName {
        self.attribute.element()
    }
    /// The exact supplied attribute (name, prefix and value), or `None` when it was absent.
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.attribute.raw_attribute()
    }
    /// The exact raw value in UTF-16 code units, distinguishing an absent attribute from an empty
    /// one.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.attribute.raw()
    }
    /// Whether the attribute was absent, present and empty, or present and nonempty.
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.attribute.observation()
    }
    /// Whether the attribute's rule applies to the supplied element.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.attribute.applicability()
    }
    /// The standard attribute this report is about.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.attribute.attribute()
    }
    /// The rows of the pinned specification's attribute index that this rule reproduces.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.attribute.inventory_rows()
    }
    /// The URL of the attribute's definition in the pinned HTML Standard snapshot.
    pub fn source_clause(&self) -> &'static str {
        self.attribute.source_clause()
    }
    /// Returns the selected intrinsic processing clause, independently of author syntax.
    pub fn processing_source_clause(&self) -> &'static str {
        match self.attribute() {
            KnownHtmlAttribute::Tabindex => source!("tabindex-value"),
            KnownHtmlAttribute::Headingoffset => {
                source!("get-an-element's-computed-heading-offset")
            }
            KnownHtmlAttribute::Colspan
            | KnownHtmlAttribute::Rowspan
            | KnownHtmlAttribute::Span => {
                source!("forming-a-table")
            }
            _ => self.source_clause(),
        }
    }
    /// The length of the raw value in UTF-16 code units, as admitted against the per-value cap.
    pub fn input_code_units(&self) -> usize {
        self.attribute.input_code_units()
    }
    /// None means the selected attribute grammar was unapplied, not missing raw.
    pub fn common(&self) -> Option<&HtmlValueReport<A, A>> {
        self.common.as_ref()
    }
    /// The author verdict, which adds the attribute's own constraint to the common grammar evidence
    /// and never rewrites it.
    pub fn author(&self) -> HtmlNumericAttributeAuthorState {
        self.author
    }
    /// The attribute's intrinsic content value after the selected default or clamp, or `None` when
    /// the rule does not apply.
    pub fn intrinsic(&self) -> Option<&T> {
        self.attribute.intrinsic()
    }
    /// How the intrinsic value was produced, or `None` when the rule does not apply.
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
    ($name:ident, $attribute:ident, $element:ident, $row:literal, $anchor:literal, $default:expr, $positive:literal) => {
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
    crate::canvas::CANVAS_DEFAULT_WIDTH,
    false
);
nonnegative_rule!(
    CANVAS_HEIGHT,
    Height,
    Canvas,
    64,
    "attr-canvas-height",
    crate::canvas::CANVAS_DEFAULT_HEIGHT,
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

// Author maximum only: the computed-offset algorithm accepts larger recovery.
const HEADING_OFFSET_MAXIMUM: u16 = 8;
const HEADING_OFFSET: NonnegativeRule = NonnegativeRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Headingoffset,
        rows: &[62],
        source: source!("attr-headingoffset"),
        context: Context::Global,
        vocabulary_source: None,
    },
    default: 0,
    positive: false,
};

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

// Borrowed span processing has the bounded front door, not the report readers' one.
pub(crate) mod table_span {
    use super::*;

    /// The td/th colspan, td/th rowspan, col span and colgroup span content rules.
    ///
    /// Each shares one recovery: the span attribute's value is parsed with the
    /// rules for parsing non-negative integers (HTML 877d §2.3.4.2); an absent
    /// attribute, a parse error, or (for the positive rules) zero selects 1; a
    /// larger value is capped at the rule's maximum. The maximum and zero handling
    /// come from the same declarations the `HtmlValueInterpreter` readers use, so
    /// the report-bearing and borrowed entry points cannot diverge. This enum
    /// establishes no element, parent or table-model evidence: callers apply the
    /// rule only to the element it names.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum HtmlTableSpanRule {
        /// td/th `colspan`: 1..=1000, zero selects 1 (forming a table, step 8).
        CellColspan,
        /// td/th `rowspan`: 0..=65534, zero preserved as "remaining rows" (step 9).
        CellRowspan,
        /// col `span`: 1..=1000, zero selects 1 (forming a table, column groups).
        ColSpan,
        /// colgroup `span`: 1..=1000, zero selects 1 (forming a table, column groups).
        ColgroupSpan,
    }

    impl HtmlTableSpanRule {
        fn declaration(self) -> BoundedSpanRule {
            match self {
                Self::CellColspan => CELL_COLSPAN,
                Self::CellRowspan => CELL_ROWSPAN,
                Self::ColSpan => COL_SPAN,
                Self::ColgroupSpan => COLGROUP_SPAN,
            }
        }

        /// Returns the content value selected for `raw` without allocating.
        ///
        /// This equals the `intrinsic` value of the matching `HtmlValueInterpreter`
        /// reader (`cell_colspan`, `cell_rowspan`, `col_span`, `colgroup_span`) for
        /// the same raw value, but builds no report and clones neither the raw value
        /// nor its digits. The caller admits the raw value's length against its own
        /// input cap and charges [`Self::work_bound`] before the call.
        pub fn content_value(self, raw: Option<&DocumentString>) -> u16 {
            self.content_value_steps(raw, &mut || {})
        }

        /// Returns an upper bound on the work units [`Self::content_value`] performs
        /// for a raw value of `input_units` UTF-16 code units, or `None` when the
        /// bound is not representable.
        ///
        /// One unit is one raw-unit inspection or digit projection step, plus one
        /// for reading the attribute's presence and length and one for selecting the
        /// result. For n units the prefix scan feeds at most n units, a negative
        /// prefix is checked for a nonzero digit in at most n - 1 steps, and the
        /// projection takes at most n - 1 steps, so the total is at most 3n + 2. An
        /// absent attribute costs one unit. Early stops (a non-digit, a capped
        /// value, a nonzero negative digit) only reduce the work.
        pub fn work_bound(self, input_units: usize) -> Option<usize> {
            match self {
                Self::CellColspan | Self::CellRowspan | Self::ColSpan | Self::ColgroupSpan => {
                    input_units.checked_mul(3)?.checked_add(2)
                }
            }
        }

        fn content_value_steps(self, raw: Option<&DocumentString>, step: &mut impl FnMut()) -> u16 {
            let rule = self.declaration();
            step();
            let Some(raw) = raw else {
                return 1;
            };
            let parsed = crate::numeric::parse_nonnegative_integer_capped_steps(
                raw.code_units(),
                usize::from(rule.maximum),
                step,
            );
            step();
            match parsed {
                Ok(0) if rule.positive => 1,
                Ok(value) => u16::try_from(value).unwrap_or(rule.maximum),
                Err(_) => 1,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const RULES: [HtmlTableSpanRule; 4] = [
            HtmlTableSpanRule::CellColspan,
            HtmlTableSpanRule::CellRowspan,
            HtmlTableSpanRule::ColSpan,
            HtmlTableSpanRule::ColgroupSpan,
        ];

        /// The pinned algorithms, written from the Standard's steps rather than
        /// from the implementation: HTML 877d §2.3.4.1 (rules for parsing
        /// integers: skip ASCII whitespace, optional sign, digits, stop at the
        /// first non-digit, error without a digit), §2.3.4.2 (negative is an
        /// error, `-0` is zero) and §4.9.12 forming a table (the cap, and zero
        /// selecting 1 except for rowspan).
        fn oracle(rule: HtmlTableSpanRule, raw: Option<&[u16]>) -> u16 {
            let (maximum, zero_is_one) = match rule {
                HtmlTableSpanRule::CellColspan => (1000_u64, true),
                HtmlTableSpanRule::CellRowspan => (65534, false),
                HtmlTableSpanRule::ColSpan | HtmlTableSpanRule::ColgroupSpan => (1000, true),
            };
            let Some(raw) = raw else { return 1 };
            let mut chars = raw.iter().copied().peekable();
            while chars
                .peek()
                .is_some_and(|c| [9, 10, 12, 13, 32].contains(c))
            {
                chars.next();
            }
            let mut negative = false;
            if let Some(&c) = chars.peek()
                && (c == u16::from(b'-') || c == u16::from(b'+'))
            {
                negative = c == u16::from(b'-');
                chars.next();
            }
            let digits: Vec<u64> = chars
                .take_while(|c| (u16::from(b'0')..=u16::from(b'9')).contains(c))
                .map(|c| u64::from(c - u16::from(b'0')))
                .collect();
            if digits.is_empty() {
                return 1;
            }
            let mut value = 0_u64;
            for digit in &digits {
                value = (value * 10 + digit).min(maximum + 1);
            }
            let any_nonzero = digits.iter().any(|d| *d != 0);
            if negative && any_nonzero {
                return 1;
            }
            if value == 0 && zero_is_one {
                return 1;
            }
            u16::try_from(value.min(maximum)).expect("the cap fits u16")
        }

        fn steps(rule: HtmlTableSpanRule, raw: Option<&DocumentString>) -> (u16, usize) {
            let mut count = 0;
            let value = rule.content_value_steps(raw, &mut || count += 1);
            (value, count)
        }

        /// Every string up to `length` over the alphabet, plus structured long
        /// values that stress each pass: whitespace, signs, zeros and nines.
        fn corpus() -> Vec<Vec<u16>> {
            let alphabet: Vec<u16> = " -+019x\u{a0}".encode_utf16().collect();
            let mut all = vec![Vec::new()];
            let mut frontier = vec![Vec::new()];
            for _ in 0..6 {
                let mut next = Vec::new();
                for prefix in &frontier {
                    for &unit in &alphabet {
                        let mut value: Vec<u16> = prefix.clone();
                        value.push(unit);
                        next.push(value);
                    }
                }
                all.extend(next.iter().cloned());
                frontier = next;
            }
            for length in [1_usize, 2, 3, 999, 1000, 1001, 4096, 100_000] {
                let unit = |c: char| c.to_string().encode_utf16().collect::<Vec<u16>>();
                let repeat = |c: char, n: usize| unit(c).repeat(n);
                all.push(repeat('0', length));
                all.push(repeat('9', length));
                all.push(repeat(' ', length));
                all.push([unit('-'), repeat('0', length - 1)].concat());
                all.push([unit('+'), repeat('9', length - 1)].concat());
                all.push([unit('1'), repeat('0', length - 1)].concat());
                all.push([unit('-'), repeat('0', length - 1), unit('1')].concat());
                all.push([repeat(' ', length / 2), repeat('7', length - length / 2)].concat());
            }
            all
        }

        #[test]
        fn values_match_the_standard_for_every_rule_and_input() {
            for rule in RULES {
                assert_eq!(rule.content_value(None), 1, "{rule:?} absent");
                for units in corpus() {
                    let raw = DocumentString::from_utf16(units.clone());
                    assert_eq!(
                        rule.content_value(Some(&raw)),
                        oracle(rule, Some(&units)),
                        "{rule:?} {:?}",
                        String::from_utf16_lossy(
                            &units.iter().copied().take(24).collect::<Vec<_>>()
                        )
                    );
                }
            }
        }

        #[test]
        fn borrowed_values_equal_the_report_readers_intrinsic_values() {
            let interpreter = HtmlValueInterpreter::new(1_000_000);
            let td = KnownHtmlElement::Td.qualified_name();
            let col = KnownHtmlElement::Col.qualified_name();
            let colgroup = KnownHtmlElement::Colgroup.qualified_name();
            for units in corpus() {
                let raw = DocumentString::from_utf16(units);
                let attribute =
                    |name: KnownHtmlAttribute| Attribute::new(name.qualified_name(), raw.clone());
                let colspan = attribute(KnownHtmlAttribute::Colspan);
                let rowspan = attribute(KnownHtmlAttribute::Rowspan);
                let span = attribute(KnownHtmlAttribute::Span);
                for (rule, report) in [
                    (
                        HtmlTableSpanRule::CellColspan,
                        interpreter.cell_colspan(&td, Some(&colspan)),
                    ),
                    (
                        HtmlTableSpanRule::CellRowspan,
                        interpreter.cell_rowspan(&td, Some(&rowspan)),
                    ),
                    (
                        HtmlTableSpanRule::ColSpan,
                        interpreter.col_span(&col, Some(&span)),
                    ),
                    (
                        HtmlTableSpanRule::ColgroupSpan,
                        interpreter.colgroup_span(&colgroup, Some(&span)),
                    ),
                ] {
                    assert_eq!(
                        report.unwrap().intrinsic().copied(),
                        Some(rule.content_value(Some(&raw))),
                        "{rule:?}"
                    );
                }
            }
        }

        #[test]
        fn work_never_exceeds_the_published_bound_and_the_bound_is_reached() {
            for rule in RULES {
                assert_eq!(steps(rule, None), (1, 1));
                assert_eq!(rule.work_bound(0), Some(2));
                for units in corpus() {
                    let raw = DocumentString::from_utf16(units.clone());
                    let (_, work) = steps(rule, Some(&raw));
                    let bound = rule.work_bound(units.len()).unwrap();
                    assert!(
                        work <= bound,
                        "{rule:?} did {work} units for {} input units, bound {bound}",
                        units.len()
                    );
                }
                // A negative zero run takes every pass at full length: the header
                // read, n scans, n - 1 sign-prefix checks, n - 1 projection steps
                // and the result selection, which is 3n.
                for length in [1_usize, 2, 1000, 100_000] {
                    let mut units = vec![u16::from(b'-')];
                    units.extend(std::iter::repeat_n(u16::from(b'0'), length - 1));
                    let raw = DocumentString::from_utf16(units);
                    let (_, work) = steps(rule, Some(&raw));
                    assert_eq!(work, 3 * length, "{rule:?} at {length}");
                    assert_eq!(rule.work_bound(length), Some(3 * length + 2));
                }
            }
        }

        #[test]
        fn unrepresentable_bounds_are_none() {
            for rule in RULES {
                assert_eq!(rule.work_bound(usize::MAX), None);
                assert_eq!(rule.work_bound(usize::MAX / 3 + 1), None);
            }
        }
    }
}

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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<
        HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
        HtmlValueAdmissionError,
    > {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut attribute = report(rule.metadata, element, raw);
        if attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlNumericAttributeReport {
                attribute,
                common: None,
                author: HtmlNumericAttributeAuthorState::Unapplied,
                origin: None,
            });
        }
        let common = self.nonnegative_integer(raw.map(HtmlAttributeInput::value))?;
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>, HtmlValueAdmissionError>
    {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut attribute = report(rule.metadata, element, raw);
        if attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlNumericAttributeReport {
                attribute,
                common: None,
                author: HtmlNumericAttributeAuthorState::Unapplied,
                origin: None,
            });
        }
        let common = self.nonnegative_integer(raw.map(HtmlAttributeInput::value))?;
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>, HtmlValueAdmissionError>
    {
        self.bounded_span(COLGROUP_SPAN, element, raw)
    }

    /// Interprets global headingoffset's local author range and offset contribution.
    /// Whole valid nonnegative syntax must be in 0..=8 for author conformance.
    /// Successful prefix recovery retains its exact magnitude, including values
    /// above eight; missing/failed recovery contributes zero with distinct origins.
    /// This is not an ancestor-summed offset, final heading level or live IDL getter.
    ///
    /// Whole original UTF-16 admission precedes expanded-name applicability. The
    /// result uses ordinary shared report/value allocation and the per-value cap,
    /// without aggregate work, physical-memory or cancellation accounting. Actual
    /// captured ancestor/reset/shadow-host computation remains with its own owner.
    pub fn heading_offset(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<
        HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
        HtmlValueAdmissionError,
    > {
        let mut result = self.nonnegative_attribute(HEADING_OFFSET, element, raw)?;
        if let Some(common) = result.common.as_ref() {
            result.author = constrained_nonnegative_author(
                common,
                HEADING_OFFSET.positive,
                Some(HEADING_OFFSET_MAXIMUM),
            );
        }
        Ok(result)
    }

    /// Interprets exact canvas bitmap width content, including valid zero.
    /// Missing or failed nonnegative recovery selects 300, without bitmap effects.
    pub fn canvas_width(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlNumericAttributeReport<HtmlInteger, HtmlTabindexValue>, HtmlValueAdmissionError>
    {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut attribute = report(TABINDEX, element, raw);
        if attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlNumericAttributeReport {
                attribute,
                common: None,
                author: HtmlNumericAttributeAuthorState::Unapplied,
                origin: None,
            });
        }
        let common = self.integer(raw.map(HtmlAttributeInput::value))?;
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
                element: &(impl crate::HtmlNameInput + ?Sized),
                raw: Option<&impl crate::HtmlAttributeInput>,
            ) -> Result<HtmlAttributeReport<$type>, HtmlValueAdmissionError> {
                self.enumerated(&$rule, element, raw)
            }

            /// Interprets the override plus supplied actual input-state authorship.
            /// Forbidden state does not erase content; no form owner is selected.
            pub fn $input(
                &self,
                element: &(impl crate::HtmlNameInput + ?Sized),
                raw: Option<&impl crate::HtmlAttributeInput>,
                state: HtmlInputType,
            ) -> Result<HtmlAttributeReport<$type>, HtmlValueAdmissionError> {
                self.enumerated_with_context(&$rule, element, raw, SuppliedContext::Input(state))
            }

            /// Interprets the override plus actual button submitter authorship.
            /// Auto eligibility is supplied, never inferred from type alone.
            pub fn $button(
                &self,
                element: &(impl crate::HtmlNameInput + ?Sized),
                raw: Option<&impl crate::HtmlAttributeInput>,
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
    /// Emits canonical global headingoffset author syntax in 0..=8.
    /// Larger exact magnitudes are rejected rather than clamped or called invalid
    /// integer syntax. This proves no computed ancestor offset or final level.
    /// Attribute construction uses ordinary allocation without a captured budget.
    pub fn into_heading_offset_attribute(
        self,
    ) -> Result<Attribute, HtmlNumericAttributeConformanceError> {
        nonnegative_constraint(&self, HEADING_OFFSET.positive, Some(HEADING_OFFSET_MAXIMUM))?;
        Ok(Attribute::new(
            HEADING_OFFSET.metadata.attribute.qualified_name(),
            self.decimal_digits(),
        ))
    }

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
    /// The display size is known: `value` is the integer and `origin` says whether parsing, a
    /// default or a clamp produced it.
    Value {
        /// The display-size integer.
        value: &'a HtmlNonNegativeInteger,
        /// How the integer was produced.
        origin: HtmlNumericAttributeValueOrigin,
    },
    /// The `size` rule does not apply; the payload is the applicability that was observed.
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
    /// The exact element name that was supplied, including any stored prefix.
    pub fn element(&self) -> &QualifiedName {
        self.attribute.element()
    }
    /// The exact supplied `size` attribute (name, prefix and value), or `None` when it was absent.
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.attribute.raw_attribute()
    }
    /// The exact raw `size` value in UTF-16 code units, distinguishing an absent attribute from an
    /// empty one.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.attribute.raw()
    }
    /// Whether `size` was absent, present and empty, or present and nonempty.
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.attribute.observation()
    }
    /// Whether the `size` rule applies to the supplied element.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.attribute.applicability()
    }
    /// The standard attribute this report is about, `size`.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.attribute.attribute()
    }
    /// The rows of the pinned specification's attribute index that this rule reproduces.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.attribute.inventory_rows()
    }
    /// The URL of the `size` attribute's definition in the pinned HTML Standard snapshot.
    pub fn source_clause(&self) -> &'static str {
        self.attribute.source_clause()
    }
    /// The URL of the pinned standard's definition of the select display size.
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
    /// The author verdict for `size`. The `multiple` attribute is judged separately by
    /// [`HtmlSelectSizeReport::multiple`].
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        size: Option<&impl crate::HtmlAttributeInput>,
        multiple: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlSelectSizeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(size.map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(multiple.map(HtmlAttributeInput::value))?;
        let attribute = report(SELECT_SIZE, element, size);
        let multiple = self.boolean_attribute(HtmlBooleanAttribute::Multiple, element, multiple)?;
        let common = if attribute.applicability() == HtmlAttributeApplicability::Applicable {
            Some(self.nonnegative_integer(size.map(HtmlAttributeInput::value))?)
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
            multiple.intrinsic().map(|present| {
                (
                    (u128::from(crate::select::select_display_size_default(*present))).into(),
                    origin,
                )
            })
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

/// Source-qualified parsed content for `ol[start]` or `li[value]`, with exact
/// signed author syntax independently retained from prefix recovery.
///
/// This local declaration is not an ordered-list starting value, list-item
/// ordinal or WebIDL signed-long getter. Missing or failed contents have no
/// invented default. The captured semantic owner supplies actual reversed
/// counts, list ownership and ordinal sequencing separately.
///
/// Reports and exact integers use ordinary allocation plus the interpreter's
/// per-original-value UTF-16 cap, without physical-memory, work or cancellation
/// guarantees. Numeric conformance does not establish element/tree conformance.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlListIntegerAttributeReport {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlInteger, HtmlInteger>>,
    author: HtmlNumericAttributeAuthorState,
}

impl HtmlListIntegerAttributeReport {
    /// Exact names, raw observation and defining metadata. Generic keyword
    /// recognition/author fields are Unapplied; numeric evidence is in common()
    /// and author(). A specified li value leaves parent-author context Unchecked.
    pub fn attribute(&self) -> &HtmlAttributeReport<()> {
        &self.attribute
    }

    /// None means the rule was unapplied and no signed grammar ran. Applicable
    /// absence retains a common report with independent Missing states.
    pub fn common(&self) -> Option<&HtmlValueReport<HtmlInteger, HtmlInteger>> {
        self.common.as_ref()
    }

    /// Checks only the whole numeric authored spelling, not list relationships.
    pub fn author(&self) -> HtmlNumericAttributeAuthorState {
        self.author
    }

    /// Borrows a successful parsed content integer, including nonconforming
    /// recovered prefixes. Missing, failed and unapplied contents return None;
    /// their distinct evidence remains in common() and attribute().
    pub fn content_integer(&self) -> Option<&HtmlInteger> {
        match self.common.as_ref()?.recovery() {
            HtmlValueRecoveryState::Value(value) => Some(value),
            HtmlValueRecoveryState::Missing | HtmlValueRecoveryState::Failed(_) => None,
        }
    }
}

const OL_START: RuleMetadata = RuleMetadata {
    attribute: KnownHtmlAttribute::Start,
    rows: &[153],
    source: source!("attr-ol-start"),
    context: Context::Elements(&[KnownHtmlElement::Ol]),
    vocabulary_source: None,
};

const LI_VALUE: RuleMetadata = RuleMetadata {
    attribute: KnownHtmlAttribute::Value,
    rows: &[176],
    source: source!("attr-li-value"),
    context: Context::Elements(&[KnownHtmlElement::Li]),
    vocabulary_source: None,
};

impl HtmlValueInterpreter {
    fn list_integer_attribute(
        &self,
        metadata: RuleMetadata,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlListIntegerAttributeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut attribute = report(metadata, element, raw);
        let common = if attribute.applicability() == HtmlAttributeApplicability::Applicable {
            if metadata.attribute == KnownHtmlAttribute::Value && raw.is_some() {
                // The li table excludes authored value on a child of ul/menu;
                // this standalone declaration has no actual parent observation.
                attribute.context_author = HtmlAttributeContextAuthorState::Unchecked;
            }
            Some(self.integer(raw.map(HtmlAttributeInput::value))?)
        } else {
            None
        };
        let author = common
            .as_ref()
            .map_or(HtmlNumericAttributeAuthorState::Unapplied, numeric_author);
        Ok(HtmlListIntegerAttributeReport {
            attribute,
            common,
            author,
        })
    }

    /// Interprets only exact HTML `ol[start]` local signed content. The full raw
    /// length is admitted before applicability; wrong roles run no grammar.
    /// This supplies no missing/failed starting-value fallback, reversed count,
    /// rendering/owner facts or IDL projection. Native exact syntax can already
    /// be authored with HtmlInteger::as_document_string and Attribute::new.
    pub fn ol_start(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlListIntegerAttributeReport, HtmlValueAdmissionError> {
        self.list_integer_attribute(OL_START, element, raw)
    }

    /// Interprets only exact HTML `li[value]` local signed content. Every present
    /// applicable value leaves the separate actual-parent author restriction
    /// Unchecked: the source prohibits authored value when li is a child of ul
    /// or menu. Absence needs no such presence check. Numeric conformance and
    /// prefix recovery are independent; neither establishes a list owner,
    /// ordinal, parent eligibility or WebIDL value. Full raw admission precedes
    /// applicability. Existing exact integer/Attribute factories author syntax.
    pub fn li_value(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlListIntegerAttributeReport, HtmlValueAdmissionError> {
        self.list_integer_attribute(LI_VALUE, element, raw)
    }
}
