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
#[derive(Clone, Copy, Debug)]
pub struct HtmlMeterAttributeInput<'a> {
    pub value: Option<&'a Attribute>,
    pub minimum: Option<&'a Attribute>,
    pub maximum: Option<&'a Attribute>,
    pub low: Option<&'a Attribute>,
    pub high: Option<&'a Attribute>,
    pub optimum: Option<&'a Attribute>,
}
impl<'a> HtmlMeterAttributeInput<'a> {
    fn slots(self) -> [Option<&'a Attribute>; 6] {
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
    Value,
    Minimum,
    Maximum,
    Low,
    High,
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
    MissingRequired,
    MissingOptional,
    Conforming,
    Nonconforming(HtmlValueSyntaxError),
    /// Checked exact-key representation bounds, not binary64 overflow or syntax.
    RepresentationUnavailable,
    Unapplied,
}

/// Each variant is one source-defined mathematical <= comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMeterAuthorRelation {
    MinimumValue,
    ValueMaximum,
    MinimumLow,
    LowMaximum,
    MinimumHigh,
    HighMaximum,
    MinimumOptimum,
    OptimumMaximum,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMeterRelationUnavailable {
    MissingRequiredValue,
    Syntax(HtmlMeterNumberRole),
    Representation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMeterRelationState {
    NotRequired,
    Conforming,
    Violated,
    Unavailable(HtmlMeterRelationUnavailable),
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
    pub fn value(&self) -> &HtmlMeterNumberReport {
        &self.slots[0]
    }
    pub fn minimum(&self) -> &HtmlMeterNumberReport {
        &self.slots[1]
    }
    pub fn maximum(&self) -> &HtmlMeterNumberReport {
        &self.slots[2]
    }
    pub fn low(&self) -> &HtmlMeterNumberReport {
        &self.slots[3]
    }
    pub fn high(&self) -> &HtmlMeterNumberReport {
        &self.slots[4]
    }
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
        element: &QualifiedName,
        input: HtmlMeterAttributeInput<'_>,
    ) -> Result<HtmlMeterAttributeReport, HtmlValueAdmissionError> {
        let raw = input.slots();
        for slot in raw {
            self.admit_attribute_raw(slot.map(Attribute::value))?;
        }
        let (value, v) = self.meter_number(element, raw[0], HtmlMeterNumberRole::Value)?;
        let (minimum, min) = self.meter_number(element, raw[1], HtmlMeterNumberRole::Minimum)?;
        let (maximum, max) = self.meter_number(element, raw[2], HtmlMeterNumberRole::Maximum)?;
        let (low, l) = self.meter_number(element, raw[3], HtmlMeterNumberRole::Low)?;
        let (high, h) = self.meter_number(element, raw[4], HtmlMeterNumberRole::High)?;
        let (optimum, o) = self.meter_number(element, raw[5], HtmlMeterNumberRole::Optimum)?;
        let applicable = element.namespace() == &Namespace::html()
            && KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Meter);
        let relations = meter_relations(&[v, min, max, l, h, o], applicable);
        Ok(HtmlMeterAttributeReport {
            slots: [value, minimum, maximum, low, high, optimum],
            relations,
        })
    }
    fn meter_number(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
pub enum HtmlMeterNativeError {
    Syntax {
        role: HtmlMeterNumberRole,
        error: HtmlValueSyntaxError,
    },
    RelationViolation(HtmlMeterAuthorRelation),
    RepresentationUnavailable,
}
impl fmt::Display for HtmlMeterNativeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid authored meter attributes: {self:?}")
    }
}
impl Error for HtmlMeterNativeError {}

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
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    pub fn minimum(&self) -> Option<&DocumentString> {
        self.minimum.as_ref()
    }
    pub fn maximum(&self) -> Option<&DocumentString> {
        self.maximum.as_ref()
    }
    pub fn low(&self) -> Option<&DocumentString> {
        self.low.as_ref()
    }
    pub fn high(&self) -> Option<&DocumentString> {
        self.high.as_ref()
    }
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
