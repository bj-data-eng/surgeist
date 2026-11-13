//! The directive set of one serialized CSP under CSP3 §2.2.1.

use super::csp_policy::transition::META_REMOVED_DIRECTIVES;
use std::ops::Range;

/// Where the serialized policy was delivered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCspPolicySource {
    /// A member extracted from a response CSP header list.
    Header,
    /// An enforcing meta policy, after applying the HTML directive exclusions.
    Meta,
}

/// One surviving directive and the exact source span of its value.
#[derive(Clone, Debug)]
pub struct HtmlCspDirective<'a> {
    name: &'a [u16],
    value: &'a [u16],
    value_range: Range<usize>,
}

impl<'a> HtmlCspDirective<'a> {
    /// The canonical ASCII-lowercase name.
    pub fn name(&self) -> impl ExactSizeIterator<Item = u8> + 'a {
        self.name
            .iter()
            .map(|&unit| (unit as u8).to_ascii_lowercase())
    }

    /// The exact UTF-16 range following the name, in the original input.
    pub fn value_range(&self) -> Range<usize> {
        self.value_range.clone()
    }

    /// Cursor over the ASCII-whitespace-delimited value tokens.
    pub fn values(&self) -> HtmlCspDirectiveValues<'a> {
        HtmlCspDirectiveValues::new(self.value)
    }
}

/// A metered cursor over the values of a directive.
pub struct HtmlCspDirectiveValues<'a> {
    remaining: &'a [u16],
}

impl<'a> HtmlCspDirectiveValues<'a> {
    /// Starts at the exact value span of a directive.
    pub const fn new(value: &'a [u16]) -> Self {
        Self { remaining: value }
    }

    /// Reads the next nonempty token. A stopped read leaves the cursor in place.
    pub fn next_with<E>(
        &mut self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<&'a [u16]>, E> {
        let mut remaining = self.remaining;
        take_while(&mut remaining, step, whitespace)?;
        let token = take_while(&mut remaining, step, |unit| !whitespace(unit))?;
        self.remaining = remaining;
        Ok((!token.is_empty()).then_some(token))
    }
}

/// A borrow of one individual policy. Header list extraction belongs to the
/// response adapter. Constructing the view performs no reads or allocations.
#[derive(Clone, Copy, Debug)]
pub struct HtmlCspSerializedPolicy<'a> {
    units: &'a [u16],
    source: HtmlCspPolicySource,
}

impl<'a> HtmlCspSerializedPolicy<'a> {
    /// Borrows the exact delivered units and their source.
    pub const fn new(units: &'a [u16], source: HtmlCspPolicySource) -> Self {
        Self { units, source }
    }

    /// Finds the first occurrence of the ASCII name, case-insensitively.
    pub fn directive_with<E>(
        self,
        name: &[u8],
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<HtmlCspDirective<'a>>, E> {
        let mut scan = Scan::new(self.units);
        while let Some(ScannedDirective { directive, .. }) = scan.next_with(step)? {
            if matches_name(&directive, name, step)? && !self.excludes(&directive, step)? {
                return Ok(Some(directive));
            }
        }
        Ok(None)
    }

    /// Visits the distinct directives in first-occurrence order.
    pub const fn directives(self) -> HtmlCspDirectives<'a> {
        HtmlCspDirectives {
            policy: self,
            scan: Scan::new(self.units),
        }
    }

    fn excludes<E>(
        self,
        directive: &HtmlCspDirective<'_>,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        if self.source == HtmlCspPolicySource::Meta {
            for removed in META_REMOVED_DIRECTIVES {
                if matches_name(directive, removed.ascii_name(), step)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

/// A metered, allocation-free cursor through one policy's distinct directives.
/// Full enumeration revisits earlier source to check for duplicate names.
pub struct HtmlCspDirectives<'a> {
    policy: HtmlCspSerializedPolicy<'a>,
    scan: Scan<'a>,
}

impl<'a> HtmlCspDirectives<'a> {
    /// Returns the next accepted directive. Refusal leaves the current directive unconsumed.
    pub fn next_with<E>(
        &mut self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<HtmlCspDirective<'a>>, E> {
        let mut scan = self.scan;
        while let Some(ScannedDirective { prefix, directive }) = scan.next_with(step)? {
            if !self.policy.excludes(&directive, step)? && !appeared_in(prefix, &directive, step)? {
                self.scan = scan;
                return Ok(Some(directive));
            }
            self.scan = scan;
        }
        self.scan = scan;
        Ok(None)
    }
}

fn appeared_in<E>(
    prefix: &[u16],
    candidate: &HtmlCspDirective<'_>,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    let mut scan = Scan::new(prefix);
    while let Some(ScannedDirective {
        directive: earlier, ..
    }) = scan.next_with(step)?
    {
        if equal_names(
            earlier.name,
            candidate.name,
            |unit| (unit as u8).to_ascii_lowercase(),
            step,
        )? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn matches_name<E>(
    directive: &HtmlCspDirective<'_>,
    name: &[u8],
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    equal_names(directive.name, name, |unit| unit.to_ascii_lowercase(), step)
}

fn equal_names<T: Copy, E>(
    first: &[u16],
    second: &[T],
    lower: impl Fn(T) -> u8,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    if first.len() != second.len() {
        return Ok(false);
    }
    for (a, b) in first.iter().zip(second) {
        step()?;
        if (*a as u8).to_ascii_lowercase() != lower(*b) {
            return Ok(false);
        }
    }
    Ok(true)
}

struct ScannedDirective<'a> {
    prefix: &'a [u16],
    directive: HtmlCspDirective<'a>,
}

#[derive(Clone, Copy)]
struct Scan<'a> {
    source: &'a [u16],
    remaining: &'a [u16],
}

impl<'a> Scan<'a> {
    const fn new(source: &'a [u16]) -> Self {
        Self {
            source,
            remaining: source,
        }
    }

    fn next_with<E>(
        &mut self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<ScannedDirective<'a>>, E> {
        while !self.remaining.is_empty() {
            let prefix_end = self.source.len() - self.remaining.len();
            let mut ascii = true;
            let mut token = take_while(&mut self.remaining, step, |unit| {
                if unit == u16::from(b';') {
                    return false;
                }
                ascii &= unit <= 0x7f;
                true
            })?;
            let end = prefix_end + token.len();
            if let Some((_, tail)) = self.remaining.split_first() {
                self.remaining = tail;
            }
            if !ascii {
                continue;
            }
            take_while(&mut token, step, whitespace)?;
            let name = take_while(&mut token, step, |unit| !whitespace(unit))?;
            if !name.is_empty() {
                let directive = HtmlCspDirective {
                    name,
                    value: token,
                    value_range: (end - token.len())..end,
                };
                return Ok(Some(ScannedDirective {
                    prefix: &self.source[..prefix_end],
                    directive,
                }));
            }
        }
        Ok(None)
    }
}

fn whitespace(unit: u16) -> bool {
    matches!(unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
}

fn take_while<'a, E>(
    remaining: &mut &'a [u16],
    step: &mut impl FnMut() -> Result<(), E>,
    mut keep: impl FnMut(u16) -> bool,
) -> Result<&'a [u16], E> {
    let mut len = 0;
    for unit in *remaining {
        step()?;
        if !keep(*unit) {
            break;
        }
        len += 1;
    }
    let (prefix, tail) = remaining.split_at(len);
    *remaining = tail;
    Ok(prefix)
}

#[cfg(test)]
mod tests;
