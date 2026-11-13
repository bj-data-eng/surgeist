//! Admitted CSP violation records for deferred delivery.

use super::{Decision, Directive, Policy, PolicyDelivery, Violation};
use std::{error::Error, fmt, mem::size_of};
use surgeist_dom::DomSourceAttributeSink;

/// The original reached attribute sink and its stringified input.
#[derive(Clone, Copy)]
pub(super) struct AttributeReport<'a> {
    sink: DomSourceAttributeSink,
    input: &'a [u16],
}

impl<'a> AttributeReport<'a> {
    /// Associate a qualified sink with the original input.
    pub(super) const fn new(sink: DomSourceAttributeSink, input: &'a [u16]) -> Self {
        Self { sink, input }
    }
}

/// Why the reached violation could not be prepared for delivery.
#[derive(Debug, Eq, PartialEq)]
pub enum ReportError<E> {
    /// The caller's original work or storage controller refused.
    Stopped(E),
    /// The report text or its required backing cannot be represented.
    RepresentationExhausted,
    /// The allocator could not supply the exact admitted buffer.
    Allocation,
}

impl<E> fmt::Display for ReportError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Stopped(_) => "CSP report admission was refused",
            Self::RepresentationExhausted => "CSP report extent is not representable",
            Self::Allocation => "CSP report backing could not be reserved",
        })
    }
}

impl<E: Error + 'static> Error for ReportError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Stopped(error) => Some(error),
            _ => None,
        }
    }
}

/// One owned deferred violation payload. The queue owner retains the
/// reporting Global and the supplied backing through actual delivery.
#[derive(Debug)]
pub struct ViolationReport<Backing> {
    index: usize,
    directive: Directive,
    delivery: PolicyDelivery,
    original_policy: Vec<u16>,
    sample: Vec<u16>,
    _backing: Backing,
}

impl<Backing> ViolationReport<Backing> {
    /// Zero-based index in the original CSP list.
    pub const fn policy_index(&self) -> usize {
        self.index
    }

    /// The directive whose check reached this violation.
    pub const fn directive(&self) -> Directive {
        self.directive
    }

    /// The policy's disposition and delivery mechanism.
    pub const fn delivery(&self) -> PolicyDelivery {
        self.delivery
    }

    /// CSP resource label for policy creation or an attribute sink.
    pub const fn resource(&self) -> &'static str {
        match self.directive {
            Directive::RequireTrustedTypesFor => "trusted-types-sink",
            Directive::TrustedTypes => "trusted-types-policy",
            Directive::StyleSrcElem => "inline",
        }
    }

    /// Serialization of the effective directive set in source order.
    pub fn original_policy(&self) -> &[u16] {
        &self.original_policy
    }

    /// Exact bounded UTF-16 sample.
    pub fn sample(&self) -> &[u16] {
        &self.sample
    }
}

#[cfg(feature = "document-scripting")]
pub(crate) fn prepare_style_violation<E, Backing>(
    index: usize,
    policy: &Policy,
    sample: &[u16],
    step: &mut impl FnMut() -> Result<(), E>,
    admit: impl FnOnce(usize) -> Result<Backing, E>,
) -> Result<ViolationReport<Backing>, ReportError<E>> {
    prepare(
        Violation {
            index,
            policy,
            directive: Directive::StyleSrcElem,
        },
        Sample::Name(sample),
        step,
        admit,
    )
}

impl Decision<'_, '_> {
    /// Prepare the next reached violation using the caller's original sample.
    /// Failed formation leaves the current violation available to retry.
    /// Admission covers both text buffers. The deferred queue admits its
    /// own record storage separately before moving in the result.
    pub(super) fn next_report<E, Backing>(
        &mut self,
        sample: Sample<'_>,
        step: &mut impl FnMut() -> Result<(), E>,
        admit: impl FnOnce(usize) -> Result<Backing, E>,
    ) -> Result<Option<ViolationReport<Backing>>, ReportError<E>> {
        let Some(candidate) = self.pending_violation(step).map_err(ReportError::Stopped)? else {
            return Ok(None);
        };
        let report = prepare(candidate, sample, step, admit)?;
        self.accept_violation(candidate);
        Ok(Some(report))
    }
}

#[derive(Clone, Copy)]
pub(super) enum Sample<'a> {
    Name(&'a [u16]),
    Attribute(AttributeReport<'a>),
}

fn prepare<E, Backing>(
    violation: Violation<'_>,
    sample: Sample<'_>,
    step: &mut impl FnMut() -> Result<(), E>,
    admit: impl FnOnce(usize) -> Result<Backing, E>,
) -> Result<ViolationReport<Backing>, ReportError<E>> {
    let mut step = || step().map_err(ReportError::Stopped);
    let mut policy_units = 0;
    visit_policy(violation.policy(), &mut step, |_: u16| {
        add_unit(&mut policy_units)
    })?;
    let mut sample_units = 0;
    visit_sample(sample, &mut step, |_: u16| add_unit(&mut sample_units))?;
    let bytes = policy_units
        .checked_add(sample_units)
        .and_then(|units| units.checked_mul(size_of::<u16>()))
        .ok_or(ReportError::RepresentationExhausted)?;
    let backing = admit(bytes).map_err(ReportError::Stopped)?;
    let mut original_policy = buffer(policy_units)?;
    let mut source_sample = buffer(sample_units)?;
    visit_policy(violation.policy(), &mut step, |unit| {
        append(&mut original_policy, unit)
    })?;
    visit_sample(sample, &mut step, |unit| append(&mut source_sample, unit))?;
    Ok(ViolationReport {
        index: violation.policy_index(),
        directive: violation.directive(),
        delivery: violation.policy().delivery(),
        original_policy,
        sample: source_sample,
        _backing: backing,
    })
}

fn add_unit<E>(units: &mut usize) -> Result<(), ReportError<E>> {
    *units = units
        .checked_add(1)
        .ok_or(ReportError::RepresentationExhausted)?;
    Ok(())
}

fn buffer<E>(units: usize) -> Result<Vec<u16>, ReportError<E>> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(units)
        .map_err(|_| ReportError::Allocation)?;
    if output.capacity() != units {
        return Err(ReportError::Allocation);
    }
    Ok(output)
}

fn append<E>(output: &mut Vec<u16>, unit: u16) -> Result<(), ReportError<E>> {
    if output.len() == output.capacity() {
        return Err(ReportError::RepresentationExhausted);
    }
    output.push(unit);
    Ok(())
}

fn visit_policy<E>(
    policy: &Policy,
    step: &mut impl FnMut() -> Result<(), E>,
    mut emit: impl FnMut(u16) -> Result<(), E>,
) -> Result<(), E> {
    let mut directives = policy.directives();
    let mut first = true;
    while let Some(directive) = directives.next_with(step)? {
        if !first {
            visit_units([u16::from(b';'), u16::from(b' ')], step, &mut emit)?;
        }
        first = false;
        visit_units(directive.name().map(u16::from), step, &mut emit)?;
        let mut values = directive.values();
        while let Some(value) = values.next_with(step)? {
            visit_units(std::iter::once(u16::from(b' ')), step, &mut emit)?;
            visit_units(value.iter().copied(), step, &mut emit)?;
        }
    }
    Ok(())
}

fn visit_sample<E>(
    sample: Sample<'_>,
    step: &mut impl FnMut() -> Result<(), E>,
    mut emit: impl FnMut(u16) -> Result<(), E>,
) -> Result<(), E> {
    let source = match sample {
        Sample::Name(name) => name,
        Sample::Attribute(attribute) => {
            let (interface, name) = attribute_sink_name(attribute.sink);
            visit_units(
                interface
                    .encode_utf16()
                    .chain(std::iter::once(u16::from(b' ')))
                    .chain(name.encode_utf16())
                    .chain(std::iter::once(u16::from(b'|'))),
                step,
                &mut emit,
            )?;
            attribute.input
        }
    };
    visit_units(source.iter().copied().take(40), step, &mut emit)
}

pub(crate) fn attribute_sink_name(sink: DomSourceAttributeSink) -> (&'static str, &'static str) {
    match sink {
        DomSourceAttributeSink::EventHandler(name) => ("Element", name),
        DomSourceAttributeSink::HtmlIframeSrcdoc => ("HTMLIFrameElement", "srcdoc"),
        DomSourceAttributeSink::HtmlScriptSrc => ("HTMLScriptElement", "src"),
        DomSourceAttributeSink::SvgScriptHref => ("SVGScriptElement", "href"),
    }
}

fn visit_units<E>(
    units: impl IntoIterator<Item = u16>,
    step: &mut impl FnMut() -> Result<(), E>,
    emit: &mut impl FnMut(u16) -> Result<(), E>,
) -> Result<(), E> {
    for unit in units {
        step()?;
        emit(unit)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
