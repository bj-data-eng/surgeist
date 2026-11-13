//! One Global's created policy names and retained default policy.

use super::reports::Sample;
use super::{Decision, Policy, ReportError, Violation, ViolationReport};
use std::{error::Error, fmt, mem::size_of, rc::Rc};
use surgeist_html::values::DocumentString;

const DEFAULT: &[u16] = &[100, 101, 102, 97, 117, 108, 116];

/// The result of attempting to publish a policy after its CSP checks.
#[derive(Debug, Eq, PartialEq)]
pub enum PolicyCreationError<E> {
    /// The current CSP decision still has unchecked policies.
    PoliciesPending,
    /// An enforcing policy denied this name.
    BlockedByCsp,
    /// This factory already has a default policy.
    DefaultAlreadyExists,
    /// The requested name history cannot be represented.
    RepresentationExhausted,
    /// The original controller refused the retained name record.
    Admission(E),
}

impl<E> fmt::Display for PolicyCreationError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PoliciesPending => "policy creation still has unchecked CSP policies",
            Self::BlockedByCsp => "policy creation was blocked by CSP",
            Self::DefaultAlreadyExists => "this Global already has a default policy",
            Self::RepresentationExhausted => "policy name history is not representable",
            Self::Admission(_) => "policy name history admission was refused",
        })
    }
}

impl<E: Error + 'static> Error for PolicyCreationError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            _ => None,
        }
    }
}

struct NameRecord<Backing> {
    name: DocumentString,
    previous: Option<Box<Self>>,
    _backing: Backing,
}

/// Creation state owned by one Global. The embedding owner authenticates the
/// Global and its current CSP before beginning a creation. It separately owns
/// the backing of each policy object and the factory itself.
pub struct PolicyFactory<Payload, Backing> {
    names: Option<Box<NameRecord<Backing>>>,
    default: Option<Rc<Payload>>,
}

impl<Payload, Backing> Default for PolicyFactory<Payload, Backing> {
    fn default() -> Self {
        Self {
            names: None,
            default: None,
        }
    }
}

impl<Payload, Backing> PolicyFactory<Payload, Backing> {
    /// The exact default policy payload retained by this Global.
    pub fn default_policy(&self) -> Option<&Rc<Payload>> {
        self.default.as_ref()
    }

    /// Resolve prior name use and start an ordered decision over the Global's
    /// current qualified CSP list. Exact UTF-16 names are case-sensitive.
    /// The borrow keeps another creation from changing the history mid-check.
    pub fn begin<'factory, 'policy, 'name, E>(
        &'factory mut self,
        name: &'name DocumentString,
        policies: &'policy [Policy],
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<PolicyCreation<'factory, 'policy, 'name, Payload, Backing>, E> {
        step()?;
        let requested = name.code_units();
        let duplicate = self.has_name(requested, step)?;
        Ok(PolicyCreation {
            factory: self,
            decision: Decision::policy_creation(policies, requested, duplicate),
            name,
            is_default: requested == DEFAULT,
        })
    }

    fn has_name<E>(
        &self,
        requested: &[u16],
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let mut current = self.names.as_deref();
        while let Some(record) = current {
            step()?;
            let candidate = record.name.code_units();
            if candidate.len() == requested.len() {
                let mut same = true;
                for (candidate, requested) in candidate.iter().zip(requested) {
                    step()?;
                    if candidate != requested {
                        same = false;
                        break;
                    }
                }
                if same {
                    return Ok(true);
                }
            }
            current = record.previous.as_deref();
        }
        Ok(false)
    }
}

impl<Payload, Backing> Drop for PolicyFactory<Payload, Backing> {
    fn drop(&mut self) {
        while let Some(mut record) = self.names.take() {
            self.names = record.previous.take();
        }
    }
}

/// One in-flight policy creation. The owner stages yielded violations in order
/// before publishing. Dropping this attempt leaves the factory unchanged.
pub struct PolicyCreation<'factory, 'policy, 'name, Payload, Backing> {
    factory: &'factory mut PolicyFactory<Payload, Backing>,
    decision: Decision<'policy, 'name>,
    name: &'name DocumentString,
    is_default: bool,
}

impl<'factory, 'policy, Payload, Backing> PolicyCreation<'factory, 'policy, '_, Payload, Backing> {
    /// Read the next reached violation before accepting a creation result.
    pub fn next_violation<E>(
        &mut self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Violation<'policy>>, E> {
        self.decision.next_violation(step)
    }

    /// Prepare the next report using the original policy name.
    pub fn next_report<E, ReportBacking>(
        &mut self,
        step: &mut impl FnMut() -> Result<(), E>,
        admit: impl FnOnce(usize) -> Result<ReportBacking, E>,
    ) -> Result<Option<ViolationReport<ReportBacking>>, ReportError<E>> {
        self.decision
            .next_report(Sample::Name(self.name.code_units()), step, admit)
    }

    /// Admit the name record after successful CSP and name checks. The
    /// original factory stays unchanged while the policy object is formed.
    pub fn admit<E>(
        self,
        admit_name: impl FnOnce(usize) -> Result<Backing, E>,
    ) -> Result<AdmittedPolicyCreation<'factory, Payload, Backing>, PolicyCreationError<E>> {
        match self.decision.blocked() {
            None => return Err(PolicyCreationError::PoliciesPending),
            Some(true) => return Err(PolicyCreationError::BlockedByCsp),
            Some(false) => {}
        }
        if self.is_default && self.factory.default.is_some() {
            return Err(PolicyCreationError::DefaultAlreadyExists);
        }
        let bytes = self
            .name
            .retained_units()
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<u16>>()))
            .and_then(|bytes| bytes.checked_add(size_of::<NameRecord<Backing>>()))
            .ok_or(PolicyCreationError::RepresentationExhausted)?;
        let backing = admit_name(bytes).map_err(PolicyCreationError::Admission)?;
        let record = Box::new(NameRecord {
            name: self.name.clone(),
            previous: None,
            _backing: backing,
        });
        Ok(AdmittedPolicyCreation {
            factory: self.factory,
            record,
            is_default: self.is_default,
        })
    }
}

/// Admitted name custody for one creation. Dropping it before publication
/// releases the pending name and leaves the Global's history unchanged.
pub struct AdmittedPolicyCreation<'factory, Payload, Backing> {
    factory: &'factory mut PolicyFactory<Payload, Backing>,
    record: Box<NameRecord<Backing>>,
    is_default: bool,
}

impl<Payload, Backing> AdmittedPolicyCreation<'_, Payload, Backing> {
    /// Publish the caller's already formed policy. The default policy shares
    /// that exact payload identity, and other policy payloads stay caller-owned.
    pub fn publish(mut self, policy: &Rc<Payload>) {
        self.record.previous = self.factory.names.take();
        self.factory.names = Some(self.record);
        if self.is_default {
            self.factory.default = Some(policy.clone());
        }
    }
}

#[cfg(test)]
mod tests;
