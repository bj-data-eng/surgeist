//! The selected relaxed DOM name rules, distinct from XML PI target grammar.

use crate::{DocumentString, DomError, Namespace, QualifiedName};
use std::sync::atomic::AtomicBool;

fn forbidden_common(unit: u16) -> bool {
    matches!(unit, 0 | 0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x2f | 0x3e)
}

pub(crate) fn valid_element_local_name(
    name: &DocumentString,
    cancelled: &AtomicBool,
) -> Result<bool, DomError> {
    valid_element_units(name.code_units(), cancelled)
}

fn valid_element_units(name: &[u16], cancelled: &AtomicBool) -> Result<bool, DomError> {
    let Some((&first, rest)) = name.split_first() else {
        return Ok(false);
    };
    if matches!(first, 0x41..=0x5a | 0x61..=0x7a) {
        for &unit in rest {
            crate::store::check_cancel(cancelled)?;
            if forbidden_common(unit) {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    // Every non-ASCII code point is admitted, including isolated surrogates;
    // scanning UTF-16 units is equivalent for these source predicates.
    if !matches!(first, 0x3a | 0x5f | 0x80..=0xffff) {
        return Ok(false);
    }
    for unit in rest {
        crate::store::check_cancel(cancelled)?;
        if !matches!(unit, 0x41..=0x5a | 0x61..=0x7a | 0x30..=0x39 | 0x2d | 0x2e | 0x3a | 0x5f | 0x80..=0xffff)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn valid_attribute_local_name(
    name: &DocumentString,
    cancelled: &AtomicBool,
) -> Result<bool, DomError> {
    if name.is_empty() {
        return Ok(false);
    }
    for &unit in name.code_units() {
        crate::store::check_cancel(cancelled)?;
        if forbidden_common(unit) || unit == 0x3d {
            return Ok(false);
        }
    }
    Ok(true)
}

fn exact(value: &DocumentString, text: &str) -> bool {
    value.code_units().iter().copied().eq(text.encode_utf16())
}

#[derive(Clone, Copy)]
enum NameContext {
    Element,
    Attribute,
}

/// Validation precedes allocation/splitting, preserving the source error order.
pub(crate) fn attribute_name(
    namespace: Namespace,
    qualified: DocumentString,
    cancelled: &AtomicBool,
) -> Result<QualifiedName, DomError> {
    extracted_name(namespace, qualified, NameContext::Attribute, cancelled)
}

pub(crate) fn element_name(
    namespace: Namespace,
    qualified: DocumentString,
    cancelled: &AtomicBool,
) -> Result<QualifiedName, DomError> {
    extracted_name(namespace, qualified, NameContext::Element, cancelled)
}

fn extracted_name(
    namespace: Namespace,
    qualified: DocumentString,
    context: NameContext,
    cancelled: &AtomicBool,
) -> Result<QualifiedName, DomError> {
    let mut split = None;
    for (position, &unit) in qualified.code_units().iter().enumerate() {
        crate::store::check_cancel(cancelled)?;
        if unit == 0x3a {
            split = Some(position);
            break;
        }
    }
    if let Some(at) = split {
        let prefix = &qualified.code_units()[..at];
        if prefix.is_empty() {
            return Err(DomError::InvalidCharacter);
        }
        for &unit in prefix {
            crate::store::check_cancel(cancelled)?;
            if forbidden_common(unit) {
                return Err(DomError::InvalidCharacter);
            }
        }
    }
    let local = &qualified.code_units()[split.map_or(0, |at| at + 1)..];
    if matches!(context, NameContext::Element) {
        if !valid_element_units(local, cancelled)? {
            return Err(DomError::InvalidCharacter);
        }
    } else {
        if local.is_empty() {
            return Err(DomError::InvalidCharacter);
        }
        for &unit in local {
            crate::store::check_cancel(cancelled)?;
            if forbidden_common(unit) || unit == 0x3d {
                return Err(DomError::InvalidCharacter);
            }
        }
    }
    let prefix_is = |text: &str| {
        split.is_some_and(|at| {
            qualified.code_units()[..at]
                .iter()
                .copied()
                .eq(text.encode_utf16())
        })
    };
    let namespace_is = |text| namespace.uri().is_some_and(|uri| exact(uri, text));
    if split.is_some() && namespace.uri().is_none()
        || prefix_is("xml") && !namespace_is("http://www.w3.org/XML/1998/namespace")
        || (exact(&qualified, "xmlns") || prefix_is("xmlns"))
            && !namespace_is("http://www.w3.org/2000/xmlns/")
        || namespace_is("http://www.w3.org/2000/xmlns/")
            && !exact(&qualified, "xmlns")
            && !prefix_is("xmlns")
    {
        return Err(DomError::Namespace);
    }
    let prefix = split
        .map(|at| copy_string(&qualified.code_units()[..at]))
        .transpose()?;
    let local = if let Some(at) = split {
        copy_string(&qualified.code_units()[at + 1..])?
    } else {
        qualified
    };
    Ok(QualifiedName::new(namespace, prefix, local))
}

fn copy_string(units: &[u16]) -> Result<DocumentString, DomError> {
    let mut copy = Vec::new();
    crate::store::reserve(&mut copy, units.len())?;
    copy.extend_from_slice(units);
    Ok(DocumentString::from_utf16(copy))
}

pub(crate) fn name_units(name: &QualifiedName) -> Option<usize> {
    name.local_name()
        .len()
        .checked_add(name.prefix().map_or(0, DocumentString::len))?
        .checked_add(name.namespace().uri().map_or(0, DocumentString::len))
}

pub(crate) fn qualified_spelling(name: &QualifiedName) -> DocumentString {
    let Some(prefix) = name.prefix() else {
        return name.local_name().clone();
    };
    let mut units = prefix.code_units().to_vec();
    units.push(0x3a);
    units.extend_from_slice(name.local_name().code_units());
    DocumentString::from_utf16(units)
}

pub(crate) fn qualified_equals(name: &QualifiedName, value: &DocumentString) -> bool {
    let prefix = name
        .prefix()
        .into_iter()
        .flat_map(|prefix| prefix.code_units().iter().copied().chain([0x3a]));
    prefix
        .chain(name.local_name().code_units().iter().copied())
        .eq(value.code_units().iter().copied())
}

/// HTML API casing is ASCII-only; exact UTF-16 non-ASCII units are untouched.
pub(crate) const fn ascii_lower(unit: u16) -> u16 {
    if unit >= 0x41 && unit <= 0x5a {
        unit + 0x20
    } else {
        unit
    }
}
pub(crate) fn ascii_lowercase(
    value: DocumentString,
    cancelled: &AtomicBool,
) -> Result<DocumentString, DomError> {
    let mut changed = false;
    for &unit in value.code_units() {
        crate::store::check_cancel(cancelled)?;
        changed |= unit != ascii_lower(unit);
    }
    if !changed {
        return Ok(value);
    }
    let mut units = Vec::new();
    crate::store::reserve(&mut units, value.len())?;
    for &unit in value.code_units() {
        crate::store::check_cancel(cancelled)?;
        units.push(ascii_lower(unit));
    }
    Ok(DocumentString::from_utf16(units))
}
pub(crate) fn in_html_namespace(name: &QualifiedName) -> bool {
    name.namespace().uri().is_some_and(|uri| {
        uri.code_units()
            .iter()
            .copied()
            .eq("http://www.w3.org/1999/xhtml".encode_utf16())
    })
}
/// Lowercase only the requested qualified spelling, never stored attributes.
pub(crate) fn qualified_equals_lowercase_input(
    name: &QualifiedName,
    value: &DocumentString,
) -> bool {
    let prefix = name
        .prefix()
        .into_iter()
        .flat_map(|prefix| prefix.code_units().iter().copied().chain([0x3a]));
    prefix
        .chain(name.local_name().code_units().iter().copied())
        .eq(value.code_units().iter().copied().map(ascii_lower))
}
