//! Controlled probes of ordinary pinned HTML character-reference data.
//!
//! Names exclude the leading ampersand and retain exact ASCII case and the
//! optional semicolon recorded in the normative table. This module does not
//! implement tokenization, attribute-context exceptions, numeric-reference
//! diagnostics, or missing-semicolon rules. Callers borrow an already admitted
//! query; source admission and buffering remain with their input owner.
//!
//! Every search interval step and record visit is admitted before access. Each
//! repeated key comparison charges five inspected bytes (one static ASCII byte
//! and one query u32 slot). Payload inspection charges its fixed Rust layout;
//! result construction charges that result layout as copy credits plus one
//! record write. These layout credits include enum slots/padding, independently
//! of compiler ABI optimization. There are no allocations or provider calls.
//! Longest probing stops when a prefix cannot continue, so its input inspections
//! are bounded by the longest static name plus one, rather than query length.

use super::control::{Invariant, ParseControl, ParseStop, WorkCategory};
#[cfg(test)]
use crate::{HtmlParseCancellation, HtmlParseLimits, HtmlParseResource};
use std::{cmp::Ordering, mem::size_of};

#[path = "character_references.rs"]
mod data;

/// A nonempty, at-most-two-scalar replacement from the normative table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReferenceValue {
    One(char),
    Two(char, char),
}

/// Exact membership and strict continuation are separate observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NameLookup {
    Unknown,
    Prefix,
    Complete {
        value: ReferenceValue,
        can_continue: bool,
    },
}

/// A longest complete name found before the query ceased to be a table prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(test)]
pub(super) struct ReferenceMatch {
    pub(super) consumed: usize,
    pub(super) value: ReferenceValue,
    pub(super) semicolon: bool,
}

fn admit_result<T>(control: &mut ParseControl) -> Result<(), ParseStop> {
    control.charge(WorkCategory::CopyByte, size_of::<T>() as u64)?;
    control.charge(WorkCategory::RecordWrite, 1)
}

fn record(
    index: usize,
    control: &mut ParseControl,
) -> Result<&'static (&'static str, ReferenceValue), ParseStop> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    data::NAMED_REFERENCES
        .get(index)
        .ok_or_else(|| control.fail_invariant(Invariant::InputCursorState))
}

fn compare_name(
    name: &str,
    query: &[u32],
    control: &mut ParseControl,
) -> Result<Ordering, ParseStop> {
    for (&point, &byte) in query.iter().zip(name.as_bytes()) {
        control.charge(WorkCategory::InspectByte, 5)?;
        let order = u32::from(byte).cmp(&point);
        if order != Ordering::Equal {
            return Ok(order);
        }
    }
    control.charge(WorkCategory::StateStep, 1)?;
    Ok(name.len().cmp(&query.len()))
}

fn starts_with(name: &str, query: &[u32], control: &mut ParseControl) -> Result<bool, ParseStop> {
    control.charge(WorkCategory::StateStep, 1)?;
    if query.len() > name.len() {
        return Ok(false);
    }
    // The query is no longer than the name, so the pairing visits every query
    // point.
    for (&point, &byte) in query.iter().zip(name.as_bytes()) {
        control.charge(WorkCategory::InspectByte, 5)?;
        if u32::from(byte) != point {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Looks up an exact name or a still-viable prefix, retaining case.
pub(super) fn lookup_name(
    query: &[u32],
    control: &mut ParseControl,
) -> Result<NameLookup, ParseStop> {
    control.checkpoint()?;
    let mut lower = 0;
    let mut upper = data::NAMED_REFERENCES.len();
    while lower < upper {
        control.charge(WorkCategory::StateStep, 1)?;
        let middle = lower.midpoint(upper);
        let candidate = record(middle, control)?;
        if compare_name(candidate.0, query, control)? == Ordering::Less {
            lower = middle.saturating_add(1);
        } else {
            upper = middle;
        }
    }
    if lower == data::NAMED_REFERENCES.len() {
        admit_result::<NameLookup>(control)?;
        return Ok(NameLookup::Unknown);
    }
    let candidate = record(lower, control)?;
    if compare_name(candidate.0, query, control)? == Ordering::Equal {
        let following = lower.saturating_add(1);
        let can_continue = if following < data::NAMED_REFERENCES.len() {
            let next = record(following, control)?;
            starts_with(next.0, query, control)?
        } else {
            false
        };
        control.charge(
            WorkCategory::InspectByte,
            size_of::<ReferenceValue>() as u64,
        )?;
        admit_result::<NameLookup>(control)?;
        Ok(NameLookup::Complete {
            value: candidate.1,
            can_continue,
        })
    } else {
        let prefix = starts_with(candidate.0, query, control)?;
        admit_result::<NameLookup>(control)?;
        Ok(if prefix {
            NameLookup::Prefix
        } else {
            NameLookup::Unknown
        })
    }
}

/// Returns exact membership; a proper prefix alone supplies no replacement.
#[cfg(test)]
pub(super) fn exact_name(
    query: &[u32],
    control: &mut ParseControl,
) -> Result<Option<ReferenceValue>, ParseStop> {
    let result = lookup_name(query, control)?;
    admit_result::<Option<ReferenceValue>>(control)?;
    Ok(match result {
        NameLookup::Complete { value, .. } => Some(value),
        NameLookup::Unknown | NameLookup::Prefix => None,
    })
}

/// Probes the longest complete table spelling, without tokenizer context rules.
///
/// A later terminal failure returns that failure rather than a shorter saved
/// match. A successful fallback preserves whether its own spelling had a
/// semicolon; it does not decide whether an attribute may consume that spelling.
#[cfg(test)]
pub(super) fn longest_name(
    query: &[u32],
    control: &mut ParseControl,
) -> Result<Option<ReferenceMatch>, ParseStop> {
    control.checkpoint()?;
    let mut longest = None;
    for length in 1..=query.len() {
        control.charge(WorkCategory::StateStep, 1)?;
        match lookup_name(&query[..length], control)? {
            NameLookup::Complete {
                value,
                can_continue,
            } => {
                control.charge(WorkCategory::InspectByte, size_of::<u32>() as u64)?;
                let semicolon = query[length - 1] == u32::from(b';');
                admit_result::<Option<ReferenceMatch>>(control)?;
                longest = Some(ReferenceMatch {
                    consumed: length,
                    value,
                    semicolon,
                });
                if !can_continue {
                    break;
                }
            }
            NameLookup::Prefix => {}
            NameLookup::Unknown => break,
        }
    }
    admit_result::<Option<ReferenceMatch>>(control)?;
    Ok(longest)
}

/// Returns only a listed C1 replacement, independently of numeric parsing.
///
/// Unlisted or outside-range code points return no override; their acceptance,
/// error reporting and final numeric-reference value belong to the tokenizer.
pub(super) fn c1_replacement(
    code_point: u32,
    control: &mut ParseControl,
) -> Result<Option<char>, ParseStop> {
    control.checkpoint()?;
    control.charge(WorkCategory::StateStep, 1)?;
    control.charge(WorkCategory::InspectByte, size_of::<u32>() as u64)?;
    let index = code_point.checked_sub(0x80).filter(|&index| index < 32);
    if let Some(index) = index {
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.charge(WorkCategory::InspectByte, size_of::<Option<char>>() as u64)?;
        admit_result::<Option<char>>(control)?;
        data::C1_REPLACEMENTS
            .get(index as usize)
            .copied()
            .ok_or_else(|| control.fail_invariant(Invariant::InputCursorState))
    } else {
        admit_result::<Option<char>>(control)?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controller(work_credits: u64, cancellation: HtmlParseCancellation) -> ParseControl {
        ParseControl::new(
            HtmlParseLimits::builder()
                .source_units(0)
                .buffered_utf16_units(0)
                .token_attributes(0)
                .auxiliary_entries(0)
                .managed_nodes(0)
                .managed_edges(0)
                .managed_attributes(0)
                .managed_utf16_units(0)
                .source_map_entries(0)
                .source_map_payload_units(0)
                .diagnostics(0)
                .diagnostic_payload_units(0)
                .context_effects(0)
                .open_elements(0)
                .tree_depth(0)
                .managed_capacity_bytes(0)
                .peak_transient_bytes(0)
                .atomic_layout_bytes(0)
                .work_credits(work_credits)
                .finish()
                .unwrap(),
            cancellation,
        )
    }

    fn control() -> ParseControl {
        controller(100_000, HtmlParseCancellation::new())
    }

    fn points(name: &str) -> Vec<u32> {
        name.chars().map(u32::from).collect()
    }

    #[test]
    fn a_table_index_beyond_the_table_stops_instead_of_indexing() {
        let mut control = control();
        let stop = record(data::NAMED_REFERENCES.len(), &mut control)
            .err()
            .unwrap();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::TokenizerState)
        );
        assert_eq!(control.stop(), Some(stop));
    }

    #[test]
    fn exact_values_match_independent_normative_rows() {
        // HTML named-character-reference table: literal expectations, not
        // values read back from the generated implementation.
        for (name, expected) in [
            ("Aacute", ReferenceValue::One('\u{c1}')),
            ("Aacute;", ReferenceValue::One('\u{c1}')),
            ("aacute;", ReferenceValue::One('\u{e1}')),
            ("AMP", ReferenceValue::One('&')),
            ("amp;", ReferenceValue::One('&')),
            (
                "CounterClockwiseContourIntegral;",
                ReferenceValue::One('\u{2233}'),
            ),
            ("NotEqualTilde;", ReferenceValue::Two('\u{2242}', '\u{338}')),
            ("acE;", ReferenceValue::Two('\u{223e}', '\u{333}')),
            ("fjlig;", ReferenceValue::Two('f', 'j')),
            ("Afr;", ReferenceValue::One('\u{1d504}')),
        ] {
            assert_eq!(
                exact_name(&points(name), &mut control()).unwrap(),
                Some(expected),
                "{name}"
            );
        }
    }

    #[test]
    fn prefix_complete_and_case_mismatch_remain_distinct() {
        assert_eq!(
            lookup_name(&[], &mut control()).unwrap(),
            NameLookup::Prefix
        );
        assert_eq!(
            lookup_name(&points("am"), &mut control()).unwrap(),
            NameLookup::Prefix
        );
        assert_eq!(
            lookup_name(&points("amp"), &mut control()).unwrap(),
            NameLookup::Complete {
                value: ReferenceValue::One('&'),
                can_continue: true
            }
        );
        assert_eq!(
            lookup_name(&points("amp;"), &mut control()).unwrap(),
            NameLookup::Complete {
                value: ReferenceValue::One('&'),
                can_continue: false
            }
        );
        assert_eq!(
            lookup_name(&points("Amp"), &mut control()).unwrap(),
            NameLookup::Unknown
        );
        assert_eq!(exact_name(&points("am"), &mut control()).unwrap(), None);
        assert_eq!(exact_name(&points("&amp;"), &mut control()).unwrap(), None);
        assert_eq!(exact_name(&[0xd800], &mut control()).unwrap(), None);
        assert_eq!(exact_name(&[0x110000], &mut control()).unwrap(), None);
    }

    #[test]
    fn longest_match_retains_shorter_complete_fallback() {
        for (query, consumed, value, semicolon) in [
            ("notin;tail", 6, ReferenceValue::One('\u{2209}'), true),
            ("notit;", 3, ReferenceValue::One('\u{ac}'), false),
            ("amp=tail", 3, ReferenceValue::One('&'), false),
            ("amp;tail", 4, ReferenceValue::One('&'), true),
            (
                "NotEqualTilde;tail",
                14,
                ReferenceValue::Two('\u{2242}', '\u{338}'),
                true,
            ),
        ] {
            assert_eq!(
                longest_name(&points(query), &mut control()).unwrap(),
                Some(ReferenceMatch {
                    consumed,
                    value,
                    semicolon
                }),
                "{query}"
            );
        }
        assert_eq!(longest_name(&points("am"), &mut control()).unwrap(), None);
        assert_eq!(
            longest_name(&points("unknown;"), &mut control()).unwrap(),
            None
        );
        assert_eq!(longest_name(&[], &mut control()).unwrap(), None);
    }

    #[test]
    fn longest_lookup_does_not_inspect_unrelated_query_tail() {
        let mut short = control();
        let mut long = control();
        let mut query = points("amp;");
        query.extend(std::iter::repeat_n(0xd800, 100_000));
        assert_eq!(
            longest_name(&query, &mut long).unwrap(),
            longest_name(&points("amp;"), &mut short).unwrap()
        );
        assert_eq!(long.usage(), short.usage());
        assert_eq!(
            long.usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            0
        );
        assert_eq!(
            long.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    #[test]
    fn c1_overrides_match_every_independent_normative_row() {
        let expected = [
            (0x80, '\u{20ac}'),
            (0x82, '\u{201a}'),
            (0x83, '\u{192}'),
            (0x84, '\u{201e}'),
            (0x85, '\u{2026}'),
            (0x86, '\u{2020}'),
            (0x87, '\u{2021}'),
            (0x88, '\u{2c6}'),
            (0x89, '\u{2030}'),
            (0x8a, '\u{160}'),
            (0x8b, '\u{2039}'),
            (0x8c, '\u{152}'),
            (0x8e, '\u{17d}'),
            (0x91, '\u{2018}'),
            (0x92, '\u{2019}'),
            (0x93, '\u{201c}'),
            (0x94, '\u{201d}'),
            (0x95, '\u{2022}'),
            (0x96, '\u{2013}'),
            (0x97, '\u{2014}'),
            (0x98, '\u{2dc}'),
            (0x99, '\u{2122}'),
            (0x9a, '\u{161}'),
            (0x9b, '\u{203a}'),
            (0x9c, '\u{153}'),
            (0x9e, '\u{17e}'),
            (0x9f, '\u{178}'),
        ];
        for (code_point, replacement) in expected {
            assert_eq!(
                c1_replacement(code_point, &mut control()).unwrap(),
                Some(replacement)
            );
        }
        for code_point in [
            0, 0x7f, 0x81, 0x8d, 0x8f, 0x90, 0x9d, 0xa0, 0xd800, 0x110000,
        ] {
            assert_eq!(c1_replacement(code_point, &mut control()).unwrap(), None);
        }
    }

    #[test]
    fn c1_lookup_charges_inspection_visit_and_complete_result_layout() {
        let mut control = control();
        assert_eq!(
            c1_replacement(0x80, &mut control).unwrap(),
            Some('\u{20ac}')
        );
        assert_eq!(control.usage().work(WorkCategory::InspectByte), 8);
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 1);
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte),
            size_of::<Option<char>>() as u64
        );
        assert_eq!(control.usage().work(WorkCategory::RecordWrite), 1);
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), 0);
    }

    #[test]
    fn exact_capacity_and_one_less_cover_a_real_complete_lookup() {
        let query = points("NotEqualTilde;");
        let mut measured = control();
        let expected = exact_name(&query, &mut measured).unwrap();
        let credits = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let mut exact = controller(credits, HtmlParseCancellation::new());
        assert_eq!(exact_name(&query, &mut exact).unwrap(), expected);
        assert_eq!(exact.usage(), measured.usage());

        let mut short = controller(credits - 1, HtmlParseCancellation::new());
        assert!(matches!(
            exact_name(&query, &mut short),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
        assert!(
            short
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current
                < credits
        );
        assert!(short.stop().is_some());
    }

    #[test]
    fn terminal_failure_returns_no_saved_longest_match() {
        let mut measured = control();
        assert!(
            longest_name(&points("not"), &mut measured)
                .unwrap()
                .is_some()
        );
        let credits = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let mut limited = controller(credits, HtmlParseCancellation::new());
        assert!(matches!(
            longest_name(&points("notin;"), &mut limited),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
    }

    #[test]
    fn cancellation_is_observed_before_empty_and_nonempty_probes() {
        let cancellation = HtmlParseCancellation::new();
        let mut control = controller(100_000, cancellation.clone());
        cancellation.cancel();
        assert_eq!(lookup_name(&[], &mut control), Err(ParseStop::Cancelled));
        assert_eq!(
            exact_name(&points("amp;"), &mut control),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(longest_name(&[], &mut control), Err(ParseStop::Cancelled));
        assert_eq!(
            c1_replacement(0x80, &mut control),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
    }

    #[test]
    fn first_terminal_remains_sticky_across_cancelled_followup() {
        let cancellation = HtmlParseCancellation::new();
        let mut control = controller(0, cancellation.clone());
        let first = lookup_name(&points("amp;"), &mut control).unwrap_err();
        assert!(matches!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        cancellation.cancel();
        assert_eq!(c1_replacement(0x80, &mut control), Err(first));
        assert_eq!(longest_name(&points("amp;"), &mut control), Err(first));
    }
}
