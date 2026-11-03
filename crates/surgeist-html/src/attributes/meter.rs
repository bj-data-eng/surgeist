//! Meter content-number author constraints from #the-meter-element.
//! Captured meter_state remains the effective six-point processing owner.

use super::*;
use crate::numeric::{
    HtmlScientificDecimalKey, ScientificDecimalKeyError, strict_scientific_decimal_key,
};
use crate::{HtmlFloatingPointNumber, HtmlValueAuthorState, HtmlValueReport, HtmlValueSyntaxError};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

/// Actual unvalidated raw slots, not conformance or captured-context proof.
#[derive(Debug)]
pub struct HtmlMeterAttributeInput<'a, A: HtmlAttributeInput> {
    /// The raw `value` slot; `None` means the attribute is absent. The slot is not validated.
    pub value: Option<&'a A>,
    /// The raw `min` slot; `None` means the attribute is absent.
    pub minimum: Option<&'a A>,
    /// The raw `max` slot; `None` means the attribute is absent.
    pub maximum: Option<&'a A>,
    /// The raw `low` slot; `None` means the attribute is absent.
    pub low: Option<&'a A>,
    /// The raw `high` slot; `None` means the attribute is absent.
    pub high: Option<&'a A>,
    /// The raw `optimum` slot; `None` means the attribute is absent.
    pub optimum: Option<&'a A>,
}
impl<A: HtmlAttributeInput> Copy for HtmlMeterAttributeInput<'_, A> {}
impl<A: HtmlAttributeInput> Clone for HtmlMeterAttributeInput<'_, A> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, A: HtmlAttributeInput> HtmlMeterAttributeInput<'a, A> {
    fn slots(self) -> [Option<&'a A>; 6] {
        [
            self.value,
            self.minimum,
            self.maximum,
            self.low,
            self.high,
            self.optimum,
        ]
    }
}

/// The six authored content-number roles, distinct from processed quantities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMeterNumberRole {
    /// The `value` attribute, the only one the standard requires (HTML877d #the-meter-element).
    Value,
    /// The `min` attribute; absent, the authoring inequalities use 0.
    Minimum,
    /// The `max` attribute; absent, the authoring inequalities use 1.0.
    Maximum,
    /// The `low` attribute, the upper limit of the gauge's low segment.
    Low,
    /// The `high` attribute, the lower limit of the gauge's high segment.
    High,
    /// The `optimum` attribute, the position the gauge treats as best.
    Optimum,
}
const ROLES: [HtmlMeterNumberRole; 6] = [
    HtmlMeterNumberRole::Value,
    HtmlMeterNumberRole::Minimum,
    HtmlMeterNumberRole::Maximum,
    HtmlMeterNumberRole::Low,
    HtmlMeterNumberRole::High,
    HtmlMeterNumberRole::Optimum,
];
impl HtmlMeterNumberRole {
    fn index(self) -> usize {
        match self {
            Self::Value => 0,
            Self::Minimum => 1,
            Self::Maximum => 2,
            Self::Low => 3,
            Self::High => 4,
            Self::Optimum => 5,
        }
    }
    fn metadata(self) -> RuleMetadata {
        let (attribute, rows, source): (_, &'static [u16], _) = match self {
            Self::Value => (
                KnownHtmlAttribute::Value,
                &[177],
                source!("attr-meter-value"),
            ),
            Self::Minimum => (KnownHtmlAttribute::Min, &[98], source!("attr-meter-min")),
            Self::Maximum => (KnownHtmlAttribute::Max, &[93], source!("attr-meter-max")),
            Self::Low => (KnownHtmlAttribute::Low, &[91], source!("attr-meter-low")),
            Self::High => (KnownHtmlAttribute::High, &[66], source!("attr-meter-high")),
            Self::Optimum => (
                KnownHtmlAttribute::Optimum,
                &[114],
                source!("attr-meter-optimum"),
            ),
        };
        RuleMetadata {
            attribute,
            rows,
            source,
            context: Context::Elements(&[KnownHtmlElement::Meter]),
            vocabulary_source: None,
        }
    }
}

/// Strict grammar only. All applicable pairwise requirements are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMeterNumberAuthorState {
    /// The required `value` attribute is absent.
    MissingRequired,
    /// An optional attribute (`min`, `max`, `low`, `high` or `optimum`) is absent, which is
    /// conforming.
    MissingOptional,
    /// The attribute is present and its text is a valid floating-point number in the checked exact
    /// representation.
    Conforming,
    /// The attribute is present but its text is not a valid floating-point number. The payload
    /// names the first syntax failure.
    Nonconforming(HtmlValueSyntaxError),
    /// Checked exact-key representation bounds, not binary64 overflow or syntax.
    RepresentationUnavailable,
    /// The rule does not apply because the element is not an HTML `meter` or the supplied attribute
    /// is not one the rule covers, so no author verdict exists for this slot.
    Unapplied,
}

/// Each variant is one source-defined mathematical <= comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMeterAuthorRelation {
    /// The standard's `minimum ≤ value` inequality.
    MinimumValue,
    /// The standard's `value ≤ maximum` inequality.
    ValueMaximum,
    /// The standard's `minimum ≤ low` inequality, applicable only when `low` is present.
    MinimumLow,
    /// The standard's `low ≤ maximum` inequality, applicable only when `low` is present.
    LowMaximum,
    /// The standard's `minimum ≤ high` inequality, applicable only when `high` is present.
    MinimumHigh,
    /// The standard's `high ≤ maximum` inequality, applicable only when `high` is present.
    HighMaximum,
    /// The standard's `minimum ≤ optimum` inequality, applicable only when `optimum` is present.
    MinimumOptimum,
    /// The standard's `optimum ≤ maximum` inequality, applicable only when `optimum` is present.
    OptimumMaximum,
    /// The standard's `low ≤ high` inequality, applicable only when both are present.
    LowHigh,
}
const RELATIONS: [HtmlMeterAuthorRelation; 9] = [
    HtmlMeterAuthorRelation::MinimumValue,
    HtmlMeterAuthorRelation::ValueMaximum,
    HtmlMeterAuthorRelation::MinimumLow,
    HtmlMeterAuthorRelation::LowMaximum,
    HtmlMeterAuthorRelation::MinimumHigh,
    HtmlMeterAuthorRelation::HighMaximum,
    HtmlMeterAuthorRelation::MinimumOptimum,
    HtmlMeterAuthorRelation::OptimumMaximum,
    HtmlMeterAuthorRelation::LowHigh,
];
impl HtmlMeterAuthorRelation {
    fn index(self) -> usize {
        match self {
            Self::MinimumValue => 0,
            Self::ValueMaximum => 1,
            Self::MinimumLow => 2,
            Self::LowMaximum => 3,
            Self::MinimumHigh => 4,
            Self::HighMaximum => 5,
            Self::MinimumOptimum => 6,
            Self::OptimumMaximum => 7,
            Self::LowHigh => 8,
        }
    }
    fn operands(self) -> (HtmlMeterNumberRole, HtmlMeterNumberRole) {
        use HtmlMeterNumberRole::*;
        match self {
            Self::MinimumValue => (Minimum, Value),
            Self::ValueMaximum => (Value, Maximum),
            Self::MinimumLow => (Minimum, Low),
            Self::LowMaximum => (Low, Maximum),
            Self::MinimumHigh => (Minimum, High),
            Self::HighMaximum => (High, Maximum),
            Self::MinimumOptimum => (Minimum, Optimum),
            Self::OptimumMaximum => (Optimum, Maximum),
            Self::LowHigh => (Low, High),
        }
    }
}
/// Why an authoring inequality has no verdict. A missing required `value` is reported first;
/// otherwise the first operand that cannot be compared, left operand first, supplies the reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlMeterRelationUnavailable {
    /// The required `value` attribute is absent, so any inequality that involves it has no verdict.
    MissingRequiredValue,
    /// The named role's text is present but not a valid floating-point number.
    Syntax(HtmlMeterNumberRole),
    /// An operand is valid but its exact key exceeds the checked representation bounds, so the
    /// comparison cannot be made.
    Representation,
}
/// The verdict for one authoring inequality in a [`HtmlMeterAttributeReport`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMeterRelationState {
    /// The inequality is not required because an optional operand (`low`, `high` or `optimum`) is
    /// absent.
    NotRequired,
    /// Both operands are known and the inequality holds.
    Conforming,
    /// Both operands are known and the inequality does not hold, so the markup is nonconforming.
    Violated,
    /// The inequality cannot be decided; the payload gives the reason.
    Unavailable(HtmlMeterRelationUnavailable),
    /// The rule does not apply because the element is not an HTML `meter` or the supplied attribute
    /// is not one the rule covers.
    Unapplied,
}

/// Exact slot evidence; grammar, binary64 representation/recovery and authored
/// relationships are independent phases.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlMeterNumberReport {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>>,
    author: HtmlMeterNumberAuthorState,
}
impl HtmlMeterNumberReport {
    /// Exact metadata and presence. Generic keyword recognition/author fields
    /// are Unapplied; common() and author() own this number's grammar evidence.
    pub fn attribute(&self) -> &HtmlAttributeReport<()> {
        &self.attribute
    }
    /// None means the rule is unapplied; applicable absence has Missing states.
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>> {
        self.common.as_ref()
    }
    /// The strict-grammar verdict for this slot, including the checked exact-representation limit.
    /// It is independent of the pairwise relations, which only the attribute report states.
    pub fn author(&self) -> HtmlMeterNumberAuthorState {
        self.author
    }
}

/// All six exact inputs and nine independently established author relations.
///
/// Every original raw slot obeys the per-value UTF-16 admission cap. Common
/// reports and exact keys use ordinary allocation, with no physical-memory
/// ledger, work/cancellation bound or captured-session guarantee. This does not
/// certify gauge purpose, children, rendering, microdata or effective state.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlMeterAttributeReport {
    slots: [HtmlMeterNumberReport; 6],
    relations: [HtmlMeterRelationState; 9],
}
impl HtmlMeterAttributeReport {
    /// The report for the required `value` attribute.
    pub fn value(&self) -> &HtmlMeterNumberReport {
        &self.slots[0]
    }
    /// The report for the `min` attribute; absence is `MissingOptional`, not a default of 0.
    pub fn minimum(&self) -> &HtmlMeterNumberReport {
        &self.slots[1]
    }
    /// The report for the `max` attribute; absence is `MissingOptional`, not a default of 1.0.
    pub fn maximum(&self) -> &HtmlMeterNumberReport {
        &self.slots[2]
    }
    /// The report for the `low` attribute.
    pub fn low(&self) -> &HtmlMeterNumberReport {
        &self.slots[3]
    }
    /// The report for the `high` attribute.
    pub fn high(&self) -> &HtmlMeterNumberReport {
        &self.slots[4]
    }
    /// The report for the `optimum` attribute.
    pub fn optimum(&self) -> &HtmlMeterNumberReport {
        &self.slots[5]
    }
    /// Optional known absence suppresses irrelevant comparisons; missing
    /// required value is unavailable. Otherwise wrong roles are Unapplied,
    /// and unavailable operands have left-then-right diagnostic precedence.
    /// Every slot's original evidence survives this diagnostic choice.
    pub fn relation(&self, relation: HtmlMeterAuthorRelation) -> HtmlMeterRelationState {
        self.relations[relation.index()]
    }
}

enum ExactFact {
    Absent,
    Number(HtmlScientificDecimalKey),
    Syntax(HtmlValueSyntaxError),
    Representation,
    Unapplied,
}
fn exact_fact(raw: Option<&DocumentString>) -> ExactFact {
    match raw {
        None => ExactFact::Absent,
        Some(raw) => match strict_scientific_decimal_key(raw) {
            Ok(key) => ExactFact::Number(key),
            Err(ScientificDecimalKeyError::Syntax(_)) => {
                ExactFact::Syntax(HtmlValueSyntaxError::InvalidNumericSyntax)
            }
            Err(ScientificDecimalKeyError::RepresentationExhausted) => ExactFact::Representation,
        },
    }
}

impl HtmlValueInterpreter {
    /// Admits all six original values before exact expanded-name applicability.
    /// This checks authored numbers and relationships, not processing defaults
    /// or clamps. Malformed present min/max never become author defaults 0/1.
    pub fn meter_attributes(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        input: HtmlMeterAttributeInput<'_, impl HtmlAttributeInput>,
    ) -> Result<HtmlMeterAttributeReport, HtmlValueAdmissionError> {
        let raw = input.slots();
        for slot in raw {
            self.admit_attribute_raw(slot.map(HtmlAttributeInput::value))?;
        }
        let (value, v) = self.meter_number(element, raw[0], HtmlMeterNumberRole::Value)?;
        let (minimum, min) = self.meter_number(element, raw[1], HtmlMeterNumberRole::Minimum)?;
        let (maximum, max) = self.meter_number(element, raw[2], HtmlMeterNumberRole::Maximum)?;
        let (low, l) = self.meter_number(element, raw[3], HtmlMeterNumberRole::Low)?;
        let (high, h) = self.meter_number(element, raw[4], HtmlMeterNumberRole::High)?;
        let (optimum, o) = self.meter_number(element, raw[5], HtmlMeterNumberRole::Optimum)?;
        let applicable = crate::values::is_html_namespace_uri(element.namespace_uri())
            && KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Meter);
        let relations = meter_relations(&[v, min, max, l, h, o], applicable);
        Ok(HtmlMeterAttributeReport {
            slots: [value, minimum, maximum, low, high, optimum],
            relations,
        })
    }
    fn meter_number(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        role: HtmlMeterNumberRole,
    ) -> Result<(HtmlMeterNumberReport, ExactFact), HtmlValueAdmissionError> {
        let attribute = report(role.metadata(), element, raw);
        let mut output = HtmlMeterNumberReport {
            attribute,
            common: None,
            author: HtmlMeterNumberAuthorState::Unapplied,
        };
        if output.attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok((output, ExactFact::Unapplied));
        }
        let common = self.floating_point_number(output.attribute.raw())?;
        let fact = match common.author() {
            HtmlValueAuthorState::Invalid(error) => ExactFact::Syntax(*error),
            _ => exact_fact(output.attribute.raw()),
        };
        output.author = match &fact {
            ExactFact::Absent if role == HtmlMeterNumberRole::Value => {
                HtmlMeterNumberAuthorState::MissingRequired
            }
            ExactFact::Absent => HtmlMeterNumberAuthorState::MissingOptional,
            ExactFact::Number(_) => HtmlMeterNumberAuthorState::Conforming,
            ExactFact::Syntax(error) => HtmlMeterNumberAuthorState::Nonconforming(*error),
            ExactFact::Representation => HtmlMeterNumberAuthorState::RepresentationUnavailable,
            ExactFact::Unapplied => HtmlMeterNumberAuthorState::Unapplied,
        };
        output.common = Some(common);
        Ok((output, fact))
    }
}

fn operand<'a>(
    fact: &'a ExactFact,
    role: HtmlMeterNumberRole,
    zero: &'a ExactFact,
    one: &'a ExactFact,
) -> Result<&'a HtmlScientificDecimalKey, HtmlMeterRelationState> {
    use HtmlMeterRelationState as State;
    use HtmlMeterRelationUnavailable as Unavailable;
    match fact {
        ExactFact::Number(key) => Ok(key),
        ExactFact::Absent => match role {
            HtmlMeterNumberRole::Minimum => operand(zero, role, zero, one),
            HtmlMeterNumberRole::Maximum => operand(one, role, zero, one),
            HtmlMeterNumberRole::Value => {
                Err(State::Unavailable(Unavailable::MissingRequiredValue))
            }
            _ => Err(State::NotRequired),
        },
        ExactFact::Syntax(_) => Err(State::Unavailable(Unavailable::Syntax(role))),
        ExactFact::Representation => Err(State::Unavailable(Unavailable::Representation)),
        ExactFact::Unapplied => Err(State::Unapplied),
    }
}
fn meter_relations(facts: &[ExactFact; 6], applicable: bool) -> [HtmlMeterRelationState; 9] {
    use HtmlMeterNumberRole as Role;
    use HtmlMeterRelationState as State;
    if !applicable {
        return [State::Unapplied; 9];
    }
    let zero = exact_fact(Some(&DocumentString::from("0")));
    let one = exact_fact(Some(&DocumentString::from("1")));
    std::array::from_fn(|index| {
        let (left, right) = RELATIONS[index].operands();
        let a = &facts[left.index()];
        let b = &facts[right.index()];
        if [left, right].into_iter().any(|role| {
            matches!(role, Role::Low | Role::High | Role::Optimum)
                && matches!(facts[role.index()], ExactFact::Absent)
        }) {
            return State::NotRequired;
        }
        if (left == Role::Value || right == Role::Value)
            && matches!(facts[Role::Value.index()], ExactFact::Absent)
        {
            return State::Unavailable(HtmlMeterRelationUnavailable::MissingRequiredValue);
        }
        if matches!(a, ExactFact::Unapplied) || matches!(b, ExactFact::Unapplied) {
            return State::Unapplied;
        }
        let a = match operand(a, left, &zero, &one) {
            Ok(key) => key,
            Err(state) => return state,
        };
        let b = match operand(b, right, &zero, &one) {
            Ok(key) => key,
            Err(state) => return state,
        };
        match a.compare(b) {
            Ok(Ordering::Greater) => State::Violated,
            Ok(_) => State::Conforming,
            Err(_) => State::Unavailable(HtmlMeterRelationUnavailable::Representation),
        }
    })
}

/// A checked native bundle could not satisfy its authored-number requirements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlMeterNativeError {
    /// A slot's text is not a valid floating-point number. The first slot in value, min, max, low,
    /// high, optimum order that fails is reported.
    Syntax {
        /// The slot whose text failed.
        role: HtmlMeterNumberRole,
        /// The first syntax failure in that slot's text.
        error: HtmlValueSyntaxError,
    },
    /// All slots are valid but an authoring inequality fails. The payload is the first failing
    /// relation in declaration order.
    RelationViolation(HtmlMeterAuthorRelation),
    /// An exact key or comparison exceeds the checked representation bounds, so the
    /// inequalities cannot be decided. Slots are examined in slot order and the first syntax or
    /// key failure is returned, so a later slot may also have a syntax failure.
    RepresentationUnavailable,
}
impl fmt::Display for HtmlMeterNumberRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Value => "value",
            Self::Minimum => "min",
            Self::Maximum => "max",
            Self::Low => "low",
            Self::High => "high",
            Self::Optimum => "optimum",
        })
    }
}
impl fmt::Display for HtmlMeterAuthorRelation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lesser, greater) = self.operands();
        write!(formatter, "meter {lesser} must not exceed {greater}")
    }
}
impl fmt::Display for HtmlMeterNativeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { role, .. } => {
                write!(
                    formatter,
                    "meter {role} attribute is not a valid floating-point number"
                )
            }
            Self::RelationViolation(relation) => fmt::Display::fmt(relation, formatter),
            Self::RepresentationUnavailable => formatter
                .write_str("meter attributes cannot be compared in the selected numeric model"),
        }
    }
}
impl Error for HtmlMeterNativeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Syntax { error, .. } => Some(error),
            Self::RelationViolation(_) | Self::RepresentationUnavailable => None,
        }
    }
}

/// Checked coupled authored syntax, not a mathematical quantity or processed
/// meter state. Accepted code units are preserved exactly, including numbers
/// beyond binary64 range/precision. Construction uses ordinary allocation;
/// checked key arithmetic is fallible, allocator exhaustion is not recoverable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMeterAttributes {
    value: DocumentString,
    minimum: Option<DocumentString>,
    maximum: Option<DocumentString>,
    low: Option<DocumentString>,
    high: Option<DocumentString>,
    optimum: Option<DocumentString>,
}
impl HtmlMeterAttributes {
    /// Requires value, complete strict grammar and all nine applicable exact
    /// constraints. Rejection precedence is syntax/key failures in emitted slot
    /// order, then relations in HtmlMeterAuthorRelation declaration order.
    /// Optional absence is retained; optimum need not lie between low and high.
    pub fn from_syntax(
        value: DocumentString,
        minimum: Option<DocumentString>,
        maximum: Option<DocumentString>,
        low: Option<DocumentString>,
        high: Option<DocumentString>,
        optimum: Option<DocumentString>,
    ) -> Result<Self, HtmlMeterNativeError> {
        let facts = [
            Some(&value),
            minimum.as_ref(),
            maximum.as_ref(),
            low.as_ref(),
            high.as_ref(),
            optimum.as_ref(),
        ]
        .map(exact_fact);
        for (role, fact) in ROLES.into_iter().zip(&facts) {
            match fact {
                ExactFact::Syntax(error) => {
                    return Err(HtmlMeterNativeError::Syntax {
                        role,
                        error: *error,
                    });
                }
                ExactFact::Representation => {
                    return Err(HtmlMeterNativeError::RepresentationUnavailable);
                }
                _ => {}
            }
        }
        for (relation, state) in RELATIONS.into_iter().zip(meter_relations(&facts, true)) {
            match state {
                HtmlMeterRelationState::Violated => {
                    return Err(HtmlMeterNativeError::RelationViolation(relation));
                }
                HtmlMeterRelationState::Unavailable(_) | HtmlMeterRelationState::Unapplied => {
                    return Err(HtmlMeterNativeError::RepresentationUnavailable);
                }
                _ => {}
            }
        }
        Ok(Self {
            value,
            minimum,
            maximum,
            low,
            high,
            optimum,
        })
    }
    /// The exact `value` text as authored, in UTF-16 code units, without trimming or numeric
    /// conversion.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    /// The exact `min` text, or `None` when the attribute was absent.
    pub fn minimum(&self) -> Option<&DocumentString> {
        self.minimum.as_ref()
    }
    /// The exact `max` text, or `None` when the attribute was absent.
    pub fn maximum(&self) -> Option<&DocumentString> {
        self.maximum.as_ref()
    }
    /// The exact `low` text, or `None` when the attribute was absent.
    pub fn low(&self) -> Option<&DocumentString> {
        self.low.as_ref()
    }
    /// The exact `high` text, or `None` when the attribute was absent.
    pub fn high(&self) -> Option<&DocumentString> {
        self.high.as_ref()
    }
    /// The exact `optimum` text, or `None` when the attribute was absent.
    pub fn optimum(&self) -> Option<&DocumentString> {
        self.optimum.as_ref()
    }
    /// Moves only present syntax into value/min/max/low/high/optimum order.
    /// No canonicalization, projection, revalidation or tree reparse occurs.
    pub fn into_attributes(self) -> Vec<Attribute> {
        let syntax = [
            Some(self.value),
            self.minimum,
            self.maximum,
            self.low,
            self.high,
            self.optimum,
        ];
        let mut output = Vec::with_capacity(syntax.iter().flatten().count());
        for (role, syntax) in ROLES.into_iter().zip(syntax) {
            if let Some(syntax) = syntax {
                output.push(Attribute::new(
                    role.metadata().attribute.qualified_name(),
                    syntax,
                ));
            }
        }
        output
    }
}
