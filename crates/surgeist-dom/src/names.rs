//! The selected relaxed DOM name rules, distinct from XML PI target grammar.

use crate::{DocumentString, DomError, Namespace, QualifiedName};

fn forbidden_common(unit: u16) -> bool {
    matches!(unit, 0 | 0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x2f | 0x3e)
}

pub(crate) fn valid_element_local_name(
    name: &DocumentString,
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    valid_element_units(name.code_units(), read)
}

fn valid_element_units(
    name: &[u16],
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    if name.is_empty() {
        return Ok(false);
    }
    read()?;
    let first = name[0];
    if matches!(first, 0x41..=0x5a | 0x61..=0x7a) {
        for unit in &name[1..] {
            read()?;
            if forbidden_common(*unit) {
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
    for unit in &name[1..] {
        read()?;
        let unit = *unit;
        if !matches!(unit, 0x41..=0x5a | 0x61..=0x7a | 0x30..=0x39 | 0x2d | 0x2e | 0x3a | 0x5f | 0x80..=0xffff)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn valid_attribute_local_name(
    name: &DocumentString,
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    valid_attribute_units(name.code_units(), read)
}

fn valid_attribute_units(
    name: &[u16],
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    if name.is_empty() {
        return Ok(false);
    }
    for unit in name {
        read()?;
        let unit = *unit;
        if forbidden_common(unit) || unit == 0x3d {
            return Ok(false);
        }
    }
    Ok(true)
}

fn valid_prefix_units(
    prefix: &[u16],
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    if prefix.is_empty() {
        return Ok(false);
    }
    for unit in prefix {
        read()?;
        if forbidden_common(*unit) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Direct native installation admits actual components, not a reconstructed
/// createElementNS operand. Invocation-only xml/xmlns/namespace extraction
/// conditions do not rewrite or constrain this distinct component entry.
pub(crate) fn validate_element_components(
    name: &QualifiedName,
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<(), DomError> {
    validate_components(name, NameContext::Element, read)
}
pub(crate) fn validate_attribute_components(
    name: &QualifiedName,
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<(), DomError> {
    validate_components(name, NameContext::Attribute, read)
}
fn validate_components(
    name: &QualifiedName,
    context: NameContext,
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<(), DomError> {
    if let Some(prefix) = name.prefix()
        && !valid_prefix_units(prefix.code_units(), read)?
    {
        return Err(DomError::InvalidCharacter);
    }
    let valid = match context {
        NameContext::Element => valid_element_local_name(name.local_name(), read)?,
        NameContext::Attribute => valid_attribute_local_name(name.local_name(), read)?,
    };
    if !valid {
        return Err(DomError::InvalidCharacter);
    }
    Ok(())
}

fn exact(
    value: &[u16],
    text: &str,
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    let mut expected = text.encode_utf16();
    for unit in value {
        read()?;
        if Some(*unit) != expected.next() {
            return Ok(false);
        }
    }
    Ok(expected.next().is_none())
}
fn prefix_is(
    qualified: &DocumentString,
    split: Option<usize>,
    text: &str,
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    match split {
        Some(at) => exact(&qualified.code_units()[..at], text, read),
        None => Ok(false),
    }
}
fn namespace_is(
    namespace: &Namespace,
    text: &str,
    read: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    match namespace.uri() {
        Some(uri) => exact(uri.code_units(), text, read),
        None => Ok(false),
    }
}

#[derive(Clone, Copy)]
enum NameContext {
    Element,
    Attribute,
}

/// Validation precedes materialization/splitting, preserving source error order.
/// The materializer copies exact UTF-16, lowering ASCII only when requested.
pub(crate) fn attribute_name(
    namespace: &Namespace,
    qualified: &DocumentString,
    read: &mut impl FnMut() -> Result<(), DomError>,
    materialize: &mut impl FnMut(&[u16], bool) -> Result<DocumentString, DomError>,
    share: &mut impl FnMut(&DocumentString) -> Result<DocumentString, DomError>,
) -> Result<QualifiedName, DomError> {
    extracted_name(
        namespace,
        qualified,
        NameContext::Attribute,
        read,
        materialize,
        share,
    )
}

pub(crate) fn element_name(
    namespace: &Namespace,
    qualified: &DocumentString,
    read: &mut impl FnMut() -> Result<(), DomError>,
    materialize: &mut impl FnMut(&[u16], bool) -> Result<DocumentString, DomError>,
    share: &mut impl FnMut(&DocumentString) -> Result<DocumentString, DomError>,
) -> Result<QualifiedName, DomError> {
    extracted_name(
        namespace,
        qualified,
        NameContext::Element,
        read,
        materialize,
        share,
    )
}

fn extracted_name(
    namespace: &Namespace,
    qualified: &DocumentString,
    context: NameContext,
    read: &mut impl FnMut() -> Result<(), DomError>,
    materialize: &mut impl FnMut(&[u16], bool) -> Result<DocumentString, DomError>,
    share: &mut impl FnMut(&DocumentString) -> Result<DocumentString, DomError>,
) -> Result<QualifiedName, DomError> {
    let mut split = None;
    for (position, unit) in qualified.code_units().iter().enumerate() {
        read()?;
        if *unit == 0x3a {
            split = Some(position);
            break;
        }
    }
    if let Some(at) = split {
        let prefix = &qualified.code_units()[..at];
        if !valid_prefix_units(prefix, read)? {
            return Err(DomError::InvalidCharacter);
        }
    }
    let local = &qualified.code_units()[split.map_or(0, |at| at + 1)..];
    if matches!(context, NameContext::Element) {
        if !valid_element_units(local, read)? {
            return Err(DomError::InvalidCharacter);
        }
    } else if !valid_attribute_units(local, read)? {
        return Err(DomError::InvalidCharacter);
    }
    if split.is_some() && namespace.uri().is_none()
        || prefix_is(qualified, split, "xml", read)?
            && !namespace_is(namespace, "http://www.w3.org/XML/1998/namespace", read)?
        || (exact(qualified.code_units(), "xmlns", read)?
            || prefix_is(qualified, split, "xmlns", read)?)
            && !namespace_is(namespace, "http://www.w3.org/2000/xmlns/", read)?
        || namespace_is(namespace, "http://www.w3.org/2000/xmlns/", read)?
            && !exact(qualified.code_units(), "xmlns", read)?
            && !prefix_is(qualified, split, "xmlns", read)?
    {
        return Err(DomError::Namespace);
    }
    let prefix = split
        .map(|at| materialize(&qualified.code_units()[..at], false))
        .transpose()?;
    let local = if let Some(at) = split {
        materialize(&qualified.code_units()[at + 1..], false)?
    } else {
        share(qualified)?
    };
    let namespace = namespace
        .uri()
        .map(|uri| share(uri).map(Namespace::from_uri))
        .transpose()?
        .unwrap_or_else(Namespace::none);
    Ok(QualifiedName::new(namespace, prefix, local))
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
    value: &DocumentString,
    read: &mut impl FnMut() -> Result<(), DomError>,
    materialize: &mut impl FnMut(&[u16], bool) -> Result<DocumentString, DomError>,
    share: &mut impl FnMut(&DocumentString) -> Result<DocumentString, DomError>,
) -> Result<DocumentString, DomError> {
    let mut changed = false;
    for unit in value.code_units() {
        read()?;
        let unit = *unit;
        changed |= unit != ascii_lower(unit);
    }
    if !changed {
        return share(value);
    }
    materialize(value.code_units(), true)
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

#[cfg(test)]
mod materialization_tests {
    use super::*;
    use std::sync::Arc;

    fn materialize(units: &[u16], lower: bool) -> Result<DocumentString, DomError> {
        Ok(DocumentString::from_utf16(
            units
                .iter()
                .map(|unit| {
                    if lower && (0x41..=0x5a).contains(unit) {
                        unit + 0x20
                    } else {
                        *unit
                    }
                })
                .collect::<Vec<_>>(),
        ))
    }
    #[test]
    fn relaxed_names_and_exact_utf16_namespace_parts_keep_their_existing_meaning() {
        let mut read = || Ok(());
        assert!(valid_element_local_name(&"A!=".into(), &mut read).unwrap());
        assert!(!valid_element_local_name(&"9a".into(), &mut read).unwrap());
        assert!(valid_attribute_local_name(&"9a".into(), &mut read).unwrap());
        assert!(!valid_attribute_local_name(&"a=b".into(), &mut read).unwrap());
        let raw = DocumentString::from_utf16([0xd800, 0x3a, 0x58, 0xdc00, 0x3a, 0x59]);
        let mut calls = Vec::new();
        let name = element_name(
            &(Namespace::from_uri("urn:example")),
            &(raw),
            &mut read,
            &mut |units, lower| {
                calls.push((units.to_vec(), lower));
                materialize(units, lower)
            },
            &mut |value| Ok(value.clone()),
        )
        .unwrap();
        assert_eq!(name.prefix().unwrap().code_units(), [0xd800]);
        assert_eq!(name.local_name().code_units(), [0x58, 0xdc00, 0x3a, 0x59]);
        assert_eq!(name.namespace(), &Namespace::from_uri("urn:example"));
        assert_eq!(
            calls,
            [
                (vec![0xd800], false),
                (vec![0x58, 0xdc00, 0x3a, 0x59], false)
            ]
        );
        validate_element_components(&name, &mut read).unwrap();
        validate_attribute_components(&name, &mut read).unwrap();
    }
    #[test]
    fn invalid_character_precedes_namespace_and_both_precede_materialization() {
        for (uri, raw, expected) in [
            (None, "p:9", DomError::InvalidCharacter),
            (None, ":a", DomError::InvalidCharacter),
            (None, "p:a", DomError::Namespace),
            (Some("urn:wrong"), "xml:a", DomError::Namespace),
            (Some("urn:wrong"), "xmlns:a", DomError::Namespace),
            (
                Some("http://www.w3.org/2000/xmlns/"),
                "p:a",
                DomError::Namespace,
            ),
            (None, "xmlns", DomError::Namespace),
        ] {
            let mut materialized = 0;
            let error = element_name(
                &(uri.map_or_else(Namespace::none, Namespace::from_uri)),
                &(raw.into()),
                &mut || Ok(()),
                &mut |units, lower| {
                    materialized += 1;
                    materialize(units, lower)
                },
                &mut |value| Ok(value.clone()),
            )
            .unwrap_err();
            assert_eq!(error, expected, "{raw}");
            assert_eq!(materialized, 0, "{raw}");
        }
        let xml = element_name(
            &(Namespace::from_uri("http://www.w3.org/XML/1998/namespace")),
            &("xml:a".into()),
            &mut || Ok(()),
            &mut materialize,
            &mut |value| Ok(value.clone()),
        )
        .unwrap();
        assert_eq!(xml.prefix().unwrap().code_units(), [120, 109, 108]);
        let xmlns = attribute_name(
            &(Namespace::from_uri("http://www.w3.org/2000/xmlns/")),
            &("xmlns:a".into()),
            &mut || Ok(()),
            &mut materialize,
            &mut |value| Ok(value.clone()),
        )
        .unwrap();
        assert_eq!(xmlns.local_name().code_units(), [97]);
    }
    #[test]
    fn unchanged_no_split_and_ascii_casing_preserve_identity_or_materialize_once() {
        let raw = DocumentString::from_utf16([0xd800, 0x61, 0xdc00]);
        let original = raw.shared_code_units().clone();
        let mut count = 0;
        let name = attribute_name(
            &(Namespace::none()),
            &(raw),
            &mut || Ok(()),
            &mut |units, lower| {
                count += 1;
                materialize(units, lower)
            },
            &mut |value| Ok(value.clone()),
        )
        .unwrap();
        assert!(Arc::ptr_eq(
            &original,
            name.local_name().shared_code_units()
        ));
        let same = ascii_lowercase(
            &(name.local_name().clone()),
            &mut || Ok(()),
            &mut |units, lower| {
                count += 1;
                materialize(units, lower)
            },
            &mut |value| Ok(value.clone()),
        )
        .unwrap();
        assert!(Arc::ptr_eq(&original, same.shared_code_units()));
        assert_eq!(count, 0);
        let mixed = DocumentString::from_utf16([0x41, 0xd800, 0x3a, 0x5a, 0xdc00]);
        let mut calls = Vec::new();
        let lower = ascii_lowercase(
            &(mixed),
            &mut || Ok(()),
            &mut |units, flag| {
                calls.push((units.to_vec(), flag));
                materialize(units, flag)
            },
            &mut |value| Ok(value.clone()),
        )
        .unwrap();
        assert_eq!(lower.code_units(), [0x61, 0xd800, 0x3a, 0x7a, 0xdc00]);
        assert_eq!(calls, [(vec![0x41, 0xd800, 0x3a, 0x5a, 0xdc00], true)]);
    }
    #[test]
    fn every_read_refusal_stops_before_materialization_and_preserves_the_exact_error() {
        let raw: DocumentString = "xml:a".into();
        let namespace = Namespace::from_uri("http://www.w3.org/XML/1998/namespace");
        let mut total = 0;
        element_name(
            &(namespace.clone()),
            &(raw.clone()),
            &mut || {
                total += 1;
                Ok(())
            },
            &mut materialize,
            &mut |value| Ok(value.clone()),
        )
        .unwrap();
        assert!(
            total > raw.len(),
            "validation and exact namespace comparisons are reached"
        );
        for stop in 1..=total {
            let mut seen = 0;
            let mut copies = 0;
            let error = element_name(
                &(namespace.clone()),
                &(raw.clone()),
                &mut || {
                    seen += 1;
                    if seen == stop {
                        Err(DomError::Cancelled)
                    } else {
                        Ok(())
                    }
                },
                &mut |units, lower| {
                    copies += 1;
                    materialize(units, lower)
                },
                &mut |value| Ok(value.clone()),
            )
            .unwrap_err();
            assert_eq!(error, DomError::Cancelled);
            assert_eq!(seen, stop);
            assert_eq!(copies, 0);
        }
    }
    #[test]
    fn materialization_refusal_does_not_publish_a_partial_name_or_reach_later_parts() {
        for stop in 1..=2 {
            let mut calls = 0;
            let error = attribute_name(
                &(Namespace::from_uri("urn:example")),
                &("p:a".into()),
                &mut || Ok(()),
                &mut |units, lower| {
                    calls += 1;
                    if calls == stop {
                        Err(DomError::NotSupported)
                    } else {
                        materialize(units, lower)
                    }
                },
                &mut |value| Ok(value.clone()),
            )
            .unwrap_err();
            assert_eq!(error, DomError::NotSupported);
            assert_eq!(calls, stop);
        }
        assert_eq!(
            ascii_lowercase(
                &("A".into()),
                &mut || Err(DomError::Cancelled),
                &mut materialize,
                &mut |value| Ok(value.clone()),
            )
            .unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(
            ascii_lowercase(
                &("A".into()),
                &mut || Ok(()),
                &mut |_, _| Err(DomError::NotSupported),
                &mut |value| Ok(value.clone()),
            )
            .unwrap_err(),
            DomError::NotSupported
        );
    }
}
