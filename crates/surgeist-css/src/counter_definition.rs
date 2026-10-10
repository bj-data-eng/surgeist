//! Immutable prospective counter descriptors and intrinsic definition admission.
use crate::{
    CssCounterStyleDescriptorKind as Kind, CssCounterStyleDescriptorRef,
    CssCounterStyleDescriptorValue, CssCounterStyleDescriptorValueRef as Value,
    CssCounterStyleDescriptors, CssCounterStyleSystem, CssDescriptorOccurrence, CssIntegerValue,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

/// System identity excluding parameters such as a fixed system's first value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleAlgorithm {
    Cyclic,
    Numeric,
    Alphabetic,
    Symbolic,
    Additive,
    Fixed,
    Extends,
}

impl CssCounterStyleSystem {
    #[must_use]
    pub const fn algorithm(&self) -> CssCounterStyleAlgorithm {
        match self {
            Self::Cyclic => CssCounterStyleAlgorithm::Cyclic,
            Self::Numeric => CssCounterStyleAlgorithm::Numeric,
            Self::Alphabetic => CssCounterStyleAlgorithm::Alphabetic,
            Self::Symbolic => CssCounterStyleAlgorithm::Symbolic,
            Self::Additive => CssCounterStyleAlgorithm::Additive,
            Self::Fixed(_) => CssCounterStyleAlgorithm::Fixed,
            Self::Extends(_) => CssCounterStyleAlgorithm::Extends,
        }
    }
}

/// A checked value borrowed either from its genuine named occurrence or a raw edit.
#[derive(Clone, Copy, Debug)]
pub struct CssCounterStyleDescriptorEntry<'a> {
    value: &'a CssCounterStyleDescriptorValue,
    occurrence: Option<&'a CssDescriptorOccurrence<CssCounterStyleDescriptorValue>>,
}
impl<'a> CssCounterStyleDescriptorEntry<'a> {
    /// Raw edits retain their real value origin without inventing a descriptor name.
    #[must_use]
    pub const fn from_value(value: &'a CssCounterStyleDescriptorValue) -> Self {
        Self {
            value,
            occurrence: None,
        }
    }
    #[must_use]
    pub const fn value(self) -> &'a CssCounterStyleDescriptorValue {
        self.value
    }
    #[must_use]
    pub const fn occurrence(
        self,
    ) -> Option<&'a CssDescriptorOccurrence<CssCounterStyleDescriptorValue>> {
        self.occurrence
    }
}
impl<'a> From<CssCounterStyleDescriptorRef<'a>> for CssCounterStyleDescriptorEntry<'a> {
    fn from(value: CssCounterStyleDescriptorRef<'a>) -> Self {
        use CssCounterStyleDescriptorRef as D;
        let occurrence = match value {
            D::System(v)
            | D::Negative(v)
            | D::Symbols(v)
            | D::Prefix(v)
            | D::Suffix(v)
            | D::Range(v)
            | D::Pad(v)
            | D::Fallback(v)
            | D::AdditiveSymbols(v)
            | D::SpeakAs(v) => v,
        };
        Self {
            value: occurrence.value(),
            occurrence: Some(occurrence),
        }
    }
}

/// Why intrinsically admitted descriptor grammar does not define a counter style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleDefinitionIssue {
    InsufficientSymbols { required: usize, actual: usize },
    MissingAdditiveSymbols,
    ExtendsWithSymbols,
}

/// Intrinsic admission only; no lookup, substitution or representation execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleDefinitionStatus {
    Defined,
    Undefined(CssCounterStyleDefinitionIssue),
    /// A relevant descriptor is awaiting variable/environment substitution.
    SubstitutionDependent,
    /// Additive weight ordering must be checked after symbolic calculation resolution.
    RequiresResolution,
}

/// A borrowed ordered prospective descriptor snapshot, with last occurrence reads.
#[derive(Clone, Debug)]
pub struct CssCounterStyleDescriptorCollection<'a> {
    entries: Vec<CssCounterStyleDescriptorEntry<'a>>,
}
impl<'a> CssCounterStyleDescriptorCollection<'a> {
    pub fn try_new(
        entries: Vec<CssCounterStyleDescriptorEntry<'a>>,
    ) -> Result<Self, CssSpecifiedValueSerializationError> {
        Self::try_new_with_limits(entries, CssSpecifiedValueSerializationLimits::default())
    }
    /// One cumulative budget traverses all values before publishing the collection.
    pub fn try_new_with_limits(
        entries: Vec<CssCounterStyleDescriptorEntry<'a>>,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.node()?;
        for entry in &entries {
            writer.node()?;
            entry.value.append_to_rule_writer(&mut writer)?;
        }
        Ok(Self { entries })
    }
    #[must_use]
    pub fn entries(&self) -> &[CssCounterStyleDescriptorEntry<'a>] {
        &self.entries
    }
    #[must_use]
    pub fn effective(&self, kind: Kind) -> Option<CssCounterStyleDescriptorEntry<'a>> {
        self.entries
            .iter()
            .rev()
            .copied()
            .find(|entry| entry.value.kind() == kind)
    }
    /// Missing system selects symbolic; unresolved system has no guessed identity.
    #[must_use]
    pub fn algorithm(&self) -> Option<CssCounterStyleAlgorithm> {
        match self.effective(Kind::System).map(|entry| entry.value.view()) {
            None => Some(CssCounterStyleAlgorithm::Symbolic),
            Some(Value::System(system)) => Some(system.algorithm()),
            _ => None,
        }
    }
    #[must_use]
    pub fn definition_status(&self) -> CssCounterStyleDefinitionStatus {
        use CssCounterStyleAlgorithm as A;
        use CssCounterStyleDefinitionIssue as I;
        use CssCounterStyleDefinitionStatus as S;
        let Some(algorithm) = self.algorithm() else {
            return S::SubstitutionDependent;
        };
        if algorithm == A::Extends {
            return if self.effective(Kind::Symbols).is_some()
                || self.effective(Kind::AdditiveSymbols).is_some()
            {
                S::Undefined(I::ExtendsWithSymbols)
            } else {
                S::Defined
            };
        }
        if algorithm == A::Additive {
            return match self
                .effective(Kind::AdditiveSymbols)
                .map(|entry| entry.value.view())
            {
                Some(Value::Pending(_)) => S::SubstitutionDependent,
                Some(Value::AdditiveSymbols(symbols))
                    if symbols
                        .tuples()
                        .iter()
                        .any(|tuple| !matches!(tuple.weight(), CssIntegerValue::Literal(_))) =>
                {
                    S::RequiresResolution
                }
                Some(Value::AdditiveSymbols(_)) => S::Defined,
                _ => S::Undefined(I::MissingAdditiveSymbols),
            };
        }
        let required = if matches!(algorithm, A::Numeric | A::Alphabetic) {
            2
        } else {
            1
        };
        let actual = match self
            .effective(Kind::Symbols)
            .map(|entry| entry.value.view())
        {
            Some(Value::Pending(_)) => return S::SubstitutionDependent,
            Some(Value::Symbols(symbols)) => symbols.symbols().len(),
            _ => 0,
        };
        if actual < required {
            S::Undefined(I::InsufficientSymbols { required, actual })
        } else {
            S::Defined
        }
    }
}
impl CssCounterStyleDescriptors {
    /// Borrows actual authored occurrences for prospective reconstruction.
    pub fn entries(&self) -> impl ExactSizeIterator<Item = CssCounterStyleDescriptorEntry<'_>> {
        self.occurrences().map(CssCounterStyleDescriptorEntry::from)
    }
    pub fn prospective(
        &self,
    ) -> Result<CssCounterStyleDescriptorCollection<'_>, CssSpecifiedValueSerializationError> {
        CssCounterStyleDescriptorCollection::try_new(self.entries().collect())
    }
}
