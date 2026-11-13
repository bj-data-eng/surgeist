//! CSP decisions for the Trusted Types attribute sink group and policy factory.
//!
//! Inputs are individual serialized CSPs after the delivery owner has split
//! header-list values. An ordered policy list supplies one violation candidate
//! per violated policy. The caller owns the Global and queues those candidates.

use std::ops::Range;
use surgeist_html::{
    security::csp::{
        HtmlCspDirectiveValues, HtmlCspDirectives, HtmlCspPolicySource, HtmlCspSerializedPolicy,
    },
    values::DocumentString,
};

mod attribute;
mod factory;
mod provider;
mod reports;
#[cfg(feature = "document-scripting")]
mod style_hash;
#[cfg(feature = "document-scripting")]
pub(crate) use attribute::AttributeMismatchProgress;
pub use attribute::{AttributeMismatch, AttributePolicies, AttributePolicyError};
pub use factory::{AdmittedPolicyCreation, PolicyCreation, PolicyCreationError, PolicyFactory};
pub use provider::{PolicyListCapacity, PolicyListError, PolicyProcessor};
#[cfg(feature = "document-scripting")]
pub(crate) use reports::attribute_sink_name;
#[cfg(feature = "document-scripting")]
pub(crate) use reports::prepare_style_violation;
pub use reports::{ReportError, ViolationReport};

/// The source and disposition of one delivered CSP policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyDelivery {
    /// An enforcing `Content-Security-Policy` header member.
    EnforcingHeader,
    /// A `Content-Security-Policy-Report-Only` header member.
    ReportOnlyHeader,
    /// An enforcing policy from a qualified `meta` delivery.
    Meta,
}

impl PolicyDelivery {
    pub(crate) fn enforces(self) -> bool {
        self != Self::ReportOnlyHeader
    }

    fn source(self) -> HtmlCspPolicySource {
        match self {
            Self::EnforcingHeader | Self::ReportOnlyHeader => HtmlCspPolicySource::Header,
            Self::Meta => HtmlCspPolicySource::Meta,
        }
    }
}

/// The two Trusted Types directives of one original serialized policy.
#[derive(Clone, Debug)]
pub struct Policy {
    serialized: DocumentString,
    delivery: PolicyDelivery,
    script_required: bool,
    names: Option<Range<usize>>,
}

impl Policy {
    /// Evaluate inline Style keywords, nonces and exact source hashes.
    #[cfg(feature = "document-scripting")]
    pub(crate) fn inline_style<E: From<surgeist_dom::DomError>>(
        &self,
        nonce: &surgeist_dom::DomStyleNonceContent,
        hashes: &mut style_hash::StyleHashes<'_>,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<(bool, bool)>, E> {
        let serialized =
            HtmlCspSerializedPolicy::new(self.serialized.code_units(), self.delivery.source());
        for name in [b"style-src-elem".as_slice(), b"style-src", b"default-src"] {
            let Some(directive) = serialized.directive_with(name, step)? else {
                continue;
            };
            let mut values = directive.values();
            let mut allows_inline = false;
            let mut has_credential = false;
            let mut unavailable = false;
            let mut report_sample = false;
            while let Some(token) = values.next_with(step)? {
                if let Some(value) = inline_credential(token, b"'nonce-", step)? {
                    has_credential = true;
                    match nonce {
                        surgeist_dom::DomStyleNonceContent::Present(nonce)
                            if matching_nonce(value, nonce.code_units(), step)? =>
                        {
                            return Ok(Some((true, false)));
                        }
                        surgeist_dom::DomStyleNonceContent::TokenizerHistoryUnavailable(nonce)
                            if matching_nonce(value, nonce.code_units(), step)? =>
                        {
                            unavailable = true;
                        }
                        _ => {}
                    }
                } else {
                    for algorithm in style_hash::Algorithm::ALL {
                        if let Some(value) = inline_credential(token, algorithm.prefix(), step)? {
                            has_credential = true;
                            if hashes.matches(algorithm, value, step)? {
                                return Ok(Some((true, false)));
                            }
                            break;
                        }
                    }
                }
                allows_inline |= keyword(token, b"'unsafe-inline'");
                report_sample |= token
                    .iter()
                    .copied()
                    .eq(b"'report-sample'".iter().copied().map(u16::from));
            }
            return Ok((!unavailable).then_some((allows_inline && !has_credential, report_sample)));
        }
        Ok(Some((true, false)))
    }

    /// Select the first occurrence of each directive in an individual CSP.
    /// The meter runs before inspecting source code units.
    pub fn parse<E>(
        serialized: DocumentString,
        delivery: PolicyDelivery,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Self, E> {
        let policy = HtmlCspSerializedPolicy::new(serialized.code_units(), delivery.source());
        let script_required = match policy.directive_with(b"require-trusted-types-for", step)? {
            Some(directive) => has_script(directive.values(), step)?,
            None => false,
        };
        let names = policy
            .directive_with(b"trusted-types", step)?
            .map(|directive| directive.value_range());
        Ok(Self {
            serialized,
            delivery,
            script_required,
            names,
        })
    }

    /// The source and disposition of this policy.
    pub const fn delivery(&self) -> PolicyDelivery {
        self.delivery
    }

    /// The original delivered text. A reporting owner serializes its parsed
    /// CSP directive set when forming the violation record.
    pub fn serialized(&self) -> &DocumentString {
        &self.serialized
    }

    /// Visits the effective directive set, in order, for violation reporting.
    pub fn directives(&self) -> HtmlCspDirectives<'_> {
        HtmlCspSerializedPolicy::new(self.serialized.code_units(), self.delivery.source())
            .directives()
    }

    fn denies_name<E>(
        &self,
        name: &[u16],
        duplicate: bool,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let Some(values) = self
            .names
            .as_ref()
            .and_then(|range| self.serialized.code_units().get(range.clone()))
        else {
            return Ok(false);
        };
        let mut permitted = false;
        let mut allows_duplicate = !duplicate;
        let mut values = HtmlCspDirectiveValues::new(values);
        while !(permitted && allows_duplicate) {
            let Some(value) = values.next_with(step)? else {
                break;
            };
            if keyword(value, b"'allow-duplicates'") {
                allows_duplicate = true;
            } else if value == [u16::from(b'*')] || matching_name(value, name, step)? {
                permitted = true;
            }
        }
        Ok(!(permitted && allows_duplicate))
    }
}

#[cfg(feature = "document-scripting")]
pub(crate) use style_hash::StyleHashes;

#[cfg(feature = "document-scripting")]
fn inline_credential<'a, E>(
    token: &'a [u16],
    prefix: &[u8],
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<Option<&'a [u16]>, E> {
    let Some((head, tail)) = token.split_at_checked(prefix.len()) else {
        return Ok(None);
    };
    if !keyword(head, prefix) {
        return Ok(None);
    }
    let Some((&0x27, value)) = tail.split_last() else {
        return Ok(None);
    };
    let unpadded = value
        .strip_suffix(&[0x3d, 0x3d])
        .or_else(|| value.strip_suffix(&[0x3d]))
        .unwrap_or(value);
    if unpadded.is_empty() {
        return Ok(None);
    }
    for &unit in unpadded {
        step()?;
        if !matches!(unit, 0x41..=0x5a | 0x61..=0x7a | 0x30..=0x39 | 0x2b | 0x2f | 0x2d | 0x5f) {
            return Ok(None);
        }
    }
    Ok(Some(value))
}

#[cfg(feature = "document-scripting")]
fn matching_nonce<E>(
    expected: &[u16],
    actual: &[u16],
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    if expected.len() != actual.len() {
        return Ok(false);
    }
    for (expected, actual) in expected.iter().zip(actual) {
        step()?;
        if expected != actual {
            return Ok(false);
        }
    }
    Ok(true)
}

fn keyword(units: &[u16], literal: &[u8]) -> bool {
    units.len() == literal.len()
        && units.iter().zip(literal).all(|(&unit, byte)| {
            u8::try_from(unit).is_ok_and(|unit| unit.eq_ignore_ascii_case(byte))
        })
}

fn has_script<E>(
    mut values: HtmlCspDirectiveValues<'_>,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    while let Some(value) = values.next_with(step)? {
        if keyword(value, b"'script'") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn matching_name<E>(
    token: &[u16],
    name: &[u16],
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    if token.is_empty() || token.len() != name.len() {
        return Ok(false);
    }
    for (&unit, &requested) in token.iter().zip(name) {
        step()?;
        if unit != requested
            || !matches!(unit, 0x30..=0x39 | 0x41..=0x5a | 0x61..=0x7a | 0x2d | 0x23 | 0x3d | 0x5f | 0x2f | 0x40 | 0x2e | 0x25)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether the current CSP list invokes the default policy for a script sink.
/// Report-only entries participate when `include_report_only` is true.
pub fn requires_script_values<E>(
    policies: &[Policy],
    include_report_only: bool,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    for policy in policies {
        step()?;
        if policy.script_required && (include_report_only || policy.delivery.enforces()) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The actual directive whose violation is reached.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Directive {
    /// `require-trusted-types-for` on a script-group sink.
    RequireTrustedTypesFor,
    /// `trusted-types` on a policy factory invocation.
    TrustedTypes,
    /// `style-src-elem` is the effective directive for an inline Style element.
    StyleSrcElem,
}

/// One ordered CSP violation candidate. The Global's report owner provides
/// the sink or policy-name sample and performs queued delivery.
#[derive(Clone, Copy, Debug)]
pub struct Violation<'a> {
    index: usize,
    policy: &'a Policy,
    directive: Directive,
}

impl<'a> Violation<'a> {
    /// Zero-based index in the original CSP list.
    pub const fn policy_index(self) -> usize {
        self.index
    }
    /// The actual policy whose directive was violated.
    pub const fn policy(self) -> &'a Policy {
        self.policy
    }
    /// The actual effective directive.
    pub const fn directive(self) -> Directive {
        self.directive
    }
}

enum Check<'a> {
    Mismatch,
    Creation { name: &'a [u16], duplicate: bool },
}

/// Progress through the actual ordered CSP list for one decision.
/// An interrupted meter leaves the current policy unconsumed. A delivered
/// candidate is returned at most once by this cursor.
pub struct Decision<'policy, 'name> {
    policies: &'policy [Policy],
    check: Check<'name>,
    index: usize,
    blocked: bool,
}

impl<'policy, 'name> Decision<'policy, 'name> {
    /// Inspect a mismatched input after the default policy yielded no value.
    pub const fn script_mismatch(policies: &'policy [Policy]) -> Self {
        Self {
            policies,
            check: Check::Mismatch,
            index: 0,
            blocked: false,
        }
    }

    /// Inspect the requested name and the factory's actual prior-use fact.
    pub const fn policy_creation(
        policies: &'policy [Policy],
        name: &'name [u16],
        duplicate: bool,
    ) -> Self {
        Self {
            policies,
            check: Check::Creation { name, duplicate },
            index: 0,
            blocked: false,
        }
    }

    /// Yield the next reached violation, including after enforcement blocks.
    pub fn next_violation<E>(
        &mut self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Violation<'policy>>, E> {
        let candidate = self.pending_violation(step)?;
        if let Some(violation) = candidate {
            self.accept_violation(violation);
        }
        Ok(candidate)
    }

    fn pending_violation<E>(
        &mut self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Violation<'policy>>, E> {
        while let Some(policy) = self.policies.get(self.index) {
            step()?;
            let directive = match self.check {
                Check::Mismatch => policy
                    .script_required
                    .then_some(Directive::RequireTrustedTypesFor),
                Check::Creation { name, duplicate } => policy
                    .denies_name(name, duplicate, step)?
                    .then_some(Directive::TrustedTypes),
            };
            if let Some(directive) = directive {
                return Ok(Some(Violation {
                    index: self.index,
                    policy,
                    directive,
                }));
            }
            self.index += 1;
        }
        Ok(None)
    }

    fn accept_violation(&mut self, violation: Violation<'policy>) {
        self.index += 1;
        self.blocked |= violation.policy.delivery.enforces();
    }

    /// The enforcement decision once every policy has been examined.
    pub fn blocked(&self) -> Option<bool> {
        (self.index == self.policies.len()).then_some(self.blocked)
    }
}

#[cfg(test)]
mod tests;
