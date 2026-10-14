//! Exact expanded-name validation without a controller or scratch allocation.

use super::{Attribute, DocumentString, OrderedAttributes};
use std::{error::Error, fmt, sync::Arc};

/// The actual immutable attribute allocation awaiting expanded-name validation.
///
/// Retaining this owner requires no copy, reserve, normalization or validation.
/// Only a completed validation can publish it as [`OrderedAttributes`].
#[derive(Debug)]
pub struct HtmlAttributeStorage {
    attributes: Arc<Vec<Attribute>>,
}

impl HtmlAttributeStorage {
    /// Adopts the existing shared vector, including its actual spare capacity.
    pub fn from_shared_attributes(attributes: Arc<Vec<Attribute>>) -> Self {
        Self { attributes }
    }

    /// Borrows the retained immutable allocation.
    pub fn shared_attributes(&self) -> &Arc<Vec<Attribute>> {
        &self.attributes
    }

    /// Returns the retained backing without validating or publishing attributes.
    pub fn into_shared_attributes(self) -> Arc<Vec<Attribute>> {
        self.attributes
    }

    /// Begins duplicate-major, first-minor exact expanded-name validation.
    pub fn validate(self) -> HtmlAttributeValidation {
        let phase = if self.attributes.len() < 2 {
            Phase::Finished
        } else {
            Phase::Pair
        };
        HtmlAttributeValidation {
            storage: self,
            duplicate: 1,
            first: 0,
            phase,
            failure: None,
        }
    }
}

/// The first duplicate expanded name, or a refused ownership transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlAttributeValidationError {
    /// The original cursor still has comparisons to perform.
    NotReady,
    /// The supplied owner slot is empty.
    Consumed,
    /// Prefixes and values are excluded from this exact expanded-name identity.
    Duplicate { first: usize, duplicate: usize },
}

impl fmt::Display for HtmlAttributeValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotReady => formatter.write_str("attribute validation is incomplete"),
            Self::Consumed => formatter.write_str("attribute validation owner was consumed"),
            Self::Duplicate { first, duplicate } => write!(
                formatter,
                "attribute at index {duplicate} duplicates the expanded name at index {first}"
            ),
        }
    }
}

impl Error for HtmlAttributeValidationError {}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Phase {
    Pair,
    NamespaceLength,
    NamespaceUnit { offset: usize, len: usize },
    LocalLength,
    LocalUnit { offset: usize, len: usize },
    Finished,
}

/// The exact next comparison's fixed work, before its operand read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAttributeValidationLayout {
    record_visits: u64,
    inspected_bytes: u64,
}

impl HtmlAttributeValidationLayout {
    /// Number of attribute/string header visits.
    pub fn record_visits(self) -> u64 {
        self.record_visits
    }

    /// Bytes inspected in exact UTF-16 unit comparisons.
    pub fn inspected_bytes(self) -> u64 {
        self.inspected_bytes
    }
}

/// An exclusive cursor retaining its allocation on interruption or refusal.
#[derive(Debug)]
pub struct HtmlAttributeValidation {
    storage: HtmlAttributeStorage,
    duplicate: usize,
    first: usize,
    phase: Phase,
    failure: Option<HtmlAttributeValidationError>,
}

impl HtmlAttributeValidation {
    /// Describes the next comparison without reading the described operands.
    /// Dropping an action leaves the cursor at that same comparison.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlAttributeValidationAction<'_>>, HtmlAttributeValidationError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        Ok((!matches!(self.phase, Phase::Finished))
            .then_some(HtmlAttributeValidationAction { validation: self }))
    }

    /// Takes only a successfully completed cursor; refusals retain the owner.
    /// This transition performs no comparisons, allocation or backing clone.
    pub fn try_finish(
        owner: &mut Option<Self>,
    ) -> Result<HtmlValidatedAttributes, HtmlAttributeValidationError> {
        let validation = owner
            .as_ref()
            .ok_or(HtmlAttributeValidationError::Consumed)?;
        if let Some(error) = validation.failure {
            return Err(error);
        }
        if !matches!(validation.phase, Phase::Finished) {
            return Err(HtmlAttributeValidationError::NotReady);
        }
        Ok(HtmlValidatedAttributes {
            storage: owner
                .take()
                .expect("checked occupied validation owner")
                .storage,
        })
    }

    /// Recovers the original allocation without further reads or publication.
    pub fn abort(self) -> HtmlAttributeStorage {
        self.storage
    }

    fn next_pair(&mut self) {
        self.first += 1;
        if self.first == self.duplicate {
            self.first = 0;
            self.duplicate += 1;
        }
        self.phase = if self.duplicate == self.storage.attributes.len() {
            Phase::Finished
        } else {
            Phase::Pair
        };
    }

    fn duplicate(&mut self) -> Result<(), HtmlAttributeValidationError> {
        let error = HtmlAttributeValidationError::Duplicate {
            first: self.first,
            duplicate: self.duplicate,
        };
        self.failure = Some(error);
        Err(error)
    }

    fn names(&self) -> (&super::QualifiedName, &super::QualifiedName) {
        (
            self.storage.attributes[self.first].name(),
            self.storage.attributes[self.duplicate].name(),
        )
    }
}

/// One non-replayable comparison borrowing its original cursor exclusively.
#[derive(Debug)]
pub struct HtmlAttributeValidationAction<'a> {
    validation: &'a mut HtmlAttributeValidation,
}

impl HtmlAttributeValidationAction<'_> {
    /// Returns work admitted before calling [`Self::advance`].
    pub fn layout(&self) -> HtmlAttributeValidationLayout {
        comparison_layout(self.validation.phase)
    }

    /// Performs only this action's admitted read and advances its cursor.
    /// A duplicate is latched before returning its first refusal.
    pub fn advance(self) -> Result<(), HtmlAttributeValidationError> {
        let validation = self.validation;
        let (left, right) = validation.names();
        match advance_comparison(
            validation.phase,
            left.namespace().uri(),
            left.local_name(),
            right.namespace().uri(),
            right.local_name(),
        ) {
            ComparisonStep::Continue(phase) => validation.phase = phase,
            ComparisonStep::Different => validation.next_pair(),
            ComparisonStep::Equal => return validation.duplicate(),
        }
        Ok(())
    }
}

/// A complete expanded-name proof attached to its actual immutable storage.
/// It cannot be constructed from a flag, caller assertion or another allocation.
#[derive(Debug)]
pub struct HtmlValidatedAttributes {
    storage: HtmlAttributeStorage,
}

impl HtmlValidatedAttributes {
    /// Borrows the exact validated order without publishing or copying storage.
    pub fn as_slice(&self) -> &[Attribute] {
        &self.storage.attributes
    }
    /// Publishes exact input order while moving the original shared backing.
    pub fn into_attributes(self) -> OrderedAttributes {
        OrderedAttributes {
            attributes: self.storage.attributes,
        }
    }
}

pub(crate) enum ComparisonStep {
    Continue(Phase),
    Different,
    Equal,
}
pub(crate) fn comparison_layout(phase: Phase) -> HtmlAttributeValidationLayout {
    match phase {
        Phase::Pair | Phase::NamespaceLength | Phase::LocalLength => {
            HtmlAttributeValidationLayout {
                record_visits: 2,
                inspected_bytes: 0,
            }
        }
        Phase::NamespaceUnit { .. } | Phase::LocalUnit { .. } => HtmlAttributeValidationLayout {
            record_visits: 0,
            inspected_bytes: 4,
        },
        Phase::Finished => unreachable!("finished comparison has no action"),
    }
}
pub(crate) fn advance_comparison(
    phase: Phase,
    left_namespace: Option<&DocumentString>,
    left_local: &DocumentString,
    right_namespace: Option<&DocumentString>,
    right_local: &DocumentString,
) -> ComparisonStep {
    use ComparisonStep::{Continue, Different, Equal};
    match phase {
        Phase::Pair => match (left_namespace, right_namespace) {
            (None, None) => Continue(Phase::LocalLength),
            (Some(_), Some(_)) => Continue(Phase::NamespaceLength),
            _ => Different,
        },
        Phase::NamespaceLength | Phase::LocalLength => {
            let namespace = matches!(phase, Phase::NamespaceLength);
            let (left, right) = if namespace {
                (
                    left_namespace.expect("checked namespace"),
                    right_namespace.expect("checked namespace"),
                )
            } else {
                (left_local, right_local)
            };
            if left.len() != right.len() {
                Different
            } else if left.is_empty() {
                if namespace {
                    Continue(Phase::LocalLength)
                } else {
                    Equal
                }
            } else if namespace {
                Continue(Phase::NamespaceUnit {
                    offset: 0,
                    len: left.len(),
                })
            } else {
                Continue(Phase::LocalUnit {
                    offset: 0,
                    len: left.len(),
                })
            }
        }
        Phase::NamespaceUnit { offset, len } | Phase::LocalUnit { offset, len } => {
            let namespace = matches!(phase, Phase::NamespaceUnit { .. });
            let (left, right) = if namespace {
                (
                    left_namespace.expect("checked namespace"),
                    right_namespace.expect("checked namespace"),
                )
            } else {
                (left_local, right_local)
            };
            if left.code_units()[offset] != right.code_units()[offset] {
                Different
            } else if offset + 1 == len {
                if namespace {
                    Continue(Phase::LocalLength)
                } else {
                    Equal
                }
            } else if namespace {
                Continue(Phase::NamespaceUnit {
                    offset: offset + 1,
                    len,
                })
            } else {
                Continue(Phase::LocalUnit {
                    offset: offset + 1,
                    len,
                })
            }
        }
        Phase::Finished => unreachable!("finished comparison has no action"),
    }
}
