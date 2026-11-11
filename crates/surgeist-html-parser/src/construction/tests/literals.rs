use super::*;
use crate::construction::storage_strings::Literal;

fn units(c: &ParseControl) -> u64 {
    c.usage()
        .resource(HtmlParseResource::ManagedUtf16Units)
        .current
}
fn provider_calls(c: &ParseControl) -> u64 {
    c.usage().work(HtmlParseWorkCategory::ProviderCall)
}

/// The first registration of a literal creates and charges its string. Every later one shares
/// it: the units of the occurrence count by its length, as for any stored string, and the
/// provider is asked for no string.
#[test]
fn a_later_registration_of_a_literal_shares_the_first_string_and_charges_its_units_only() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let storage = session.active_mut().unwrap();
    let before = (units(&c), provider_calls(&c));
    let (first, first_slot) = storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let created = (units(&c) - before.0, provider_calls(&c) - before.1);
    assert_eq!(first.to_utf8().unwrap(), "http://www.w3.org/1999/xhtml");
    // The 28 units of the URI, and a growth of the buffer for each capacity 1, 2, 4, 8, 16
    // and 32 that holds them, besides the wrapper and the row.
    assert_eq!(created.0, 28);
    assert!(created.1 >= 7, "{} provider requests", created.1);

    let before = (units(&c), provider_calls(&c));
    let (second, second_slot) = storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    assert_ne!(first_slot, second_slot);
    assert!(first.shares_storage_with(&second));
    assert_eq!(units(&c) - before.0, 28);
    // The ticket of the occurrence and its row.
    let shared = provider_calls(&c) - before.1;
    assert!(shared <= 4, "{shared} provider requests");

    // Another literal is another string.
    let (svg, _) = storage
        .register_literal(&mut c, Literal::SvgNamespace)
        .unwrap();
    assert!(!svg.shares_storage_with(&first));
    drop((first, second, svg));
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

/// The units and work credits a parse holds after each of two registrations of a literal.
fn measured() -> ((u64, u64), (u64, u64)) {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let storage = session.active_mut().unwrap();
    let credits = |c: &ParseControl| c.usage().resource(HtmlParseResource::WorkCredits).current;
    storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let first = (units(&c), credits(&c));
    storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let second = (units(&c), credits(&c));
    session.release(&mut c).unwrap();
    (first, second)
}

/// Registers a literal twice under `limits` and returns the result of the second registration.
/// The first literal stays intact whatever the second does, and the parse releases to zero.
fn second_registration(
    limits: crate::HtmlParseLimits,
    copy: bool,
) -> Result<(), crate::parser::control::HtmlParseStop> {
    let mut c = ParseControl::new(limits, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let storage = session.active_mut().unwrap();
    let (first, _) = storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let second = if copy {
        storage.register_literal_copy(&mut c, Literal::HtmlNamespace, None)
    } else {
        storage.register_literal(&mut c, Literal::HtmlNamespace)
    };
    let result = match second {
        Ok(_) => Ok(()),
        Err(SessionError::Stop(stop)) => {
            assert_eq!(c.stop(), Some(stop));
            Err(stop)
        }
        Err(other) => panic!("refused for another reason: {other:?}"),
    };
    assert_eq!(first.to_utf8().unwrap(), "http://www.w3.org/1999/xhtml");
    session.release(&mut c).unwrap();
    storage_zero(&c);
    result
}

/// A second registration needs exactly the 28 units of the literal: a limit that holds them
/// admits it, and one unit less refuses it with the unit stop.
#[test]
fn a_later_registration_of_a_literal_is_refused_one_unit_short_of_its_units() {
    let (first, second) = measured();
    assert_eq!(second.0 - first.0, 28);
    for copy in [false, true] {
        let limited = |units: u64| {
            limits()
                .into_builder()
                .managed_utf16_units(units)
                .finish()
                .unwrap()
        };
        assert_eq!(second_registration(limited(second.0), copy), Ok(()));
        assert!(matches!(
            second_registration(limited(second.0 - 1), copy),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedUtf16Units,
                ..
            })
        ));
    }
}

/// The work of a second registration is admitted before it runs: a work limit that holds it
/// admits it, and one credit less refuses it with the work stop.
#[test]
fn a_later_registration_of_a_literal_is_refused_one_credit_short_of_its_work() {
    let (_, second) = measured();
    let limited = |credits: u64| {
        limits()
            .into_builder()
            .work_credits(credits)
            .finish()
            .unwrap()
    };
    assert_eq!(second_registration(limited(second.1), false), Ok(()));
    assert!(matches!(
        second_registration(limited(second.1 - 1), false),
        Err(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
}

/// The work charged so far in every category but the bytes reserved, which depend on where the
/// registry's own growth falls between two copies.
fn work_of(c: &ParseControl) -> Vec<u64> {
    crate::schema_support::CATEGORIES
        .iter()
        .filter(|&&(category, _)| category != HtmlParseWorkCategory::ReserveByte)
        .map(|&(category, _)| c.usage().work(category))
        .collect()
}

/// A copy of a field whose value is not a literal is the copy `copy_registered_field` makes, and
/// charges what that copy charges and nothing more.
#[test]
fn a_copy_of_a_value_that_is_not_a_literal_charges_what_the_plain_copy_charges() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let storage = session.active_mut().unwrap();
    let frozen = literal_string(&mut c, b"urn:other").unwrap();
    let (_, slot) = storage.register_string(&mut c, frozen, None).unwrap();
    let measure = |c: &ParseControl, before: &[u64]| -> Vec<u64> {
        work_of(c)
            .iter()
            .zip(before)
            .map(|(after, before)| after - before)
            .collect()
    };
    let before = work_of(&c);
    let (plain, _) = storage.copy_registered_field(&mut c, slot).unwrap();
    let plain_work = measure(&c, &before);
    let before = work_of(&c);
    let (routed, _) = storage
        .copy_registered_literal_field(&mut c, slot, false)
        .unwrap();
    assert_eq!(measure(&c, &before), plain_work);
    assert_eq!(routed.code_units(), plain.code_units());
    assert!(!routed.shares_storage_with(&plain));
    drop((plain, routed));
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

/// The registration of a literal copy, measured after the first registration of the literal:
/// the units, work credits and source map entries the parse holds after each.
fn copy_measure(group: bool) -> ((u64, u64, u64), (u64, u64, u64)) {
    let source = crate::HtmlSource::utf8("<html>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    if group {
        session.initialize_parse_evidence(&mut c, &source).unwrap();
    }
    let storage = session.active_mut().unwrap();
    let held = |c: &ParseControl| {
        (
            units(c),
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
        )
    };
    let (_, slot) = storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let first = held(&c);
    storage
        .copy_registered_literal_field(&mut c, slot, false)
        .unwrap();
    let second = held(&c);
    session.release(&mut c).unwrap();
    (first, second)
}

/// Copies a literal field under `limits`, with an origin group when `group`, and returns the
/// result of the copy. The first string stays intact and the parse releases to zero.
fn copy_under(limits: crate::HtmlParseLimits, group: bool) -> Result<(), HtmlParseStop> {
    let source = crate::HtmlSource::utf8("<html>");
    let mut c = ParseControl::new(limits, HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    if group {
        session.initialize_parse_evidence(&mut c, &source).unwrap();
    }
    let storage = session.active_mut().unwrap();
    let (first, slot) = storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let result = match storage.copy_registered_literal_field(&mut c, slot, false) {
        Ok((copy, _)) => {
            assert!(copy.shares_storage_with(&first));
            Ok(())
        }
        Err(SessionError::Stop(stop)) => {
            assert_eq!(c.stop(), Some(stop));
            Err(stop)
        }
        Err(other) => panic!("refused for another reason: {other:?}"),
    };
    assert_eq!(first.to_utf8().unwrap(), "http://www.w3.org/1999/xhtml");
    session.release(&mut c).unwrap();
    storage_zero(&c);
    result
}

/// A copy of a literal is refused one unit and one credit short of what it needs, with and
/// without the origin group it copies, and a limit that holds it admits it.
#[test]
fn a_copy_of_a_literal_is_refused_one_unit_and_one_credit_short() {
    for group in [false, true] {
        let (first, second) = copy_measure(group);
        assert_eq!(second.0 - first.0, 28);
        let units_limit = |units: u64| {
            limits()
                .into_builder()
                .managed_utf16_units(units)
                .finish()
                .unwrap()
        };
        assert_eq!(copy_under(units_limit(second.0), group), Ok(()));
        assert!(matches!(
            copy_under(units_limit(second.0 - 1), group),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedUtf16Units,
                ..
            })
        ));
        let credits_limit = |credits: u64| {
            limits()
                .into_builder()
                .work_credits(credits)
                .finish()
                .unwrap()
        };
        assert_eq!(copy_under(credits_limit(second.1), group), Ok(()));
        assert!(matches!(
            copy_under(credits_limit(second.1 - 1), group),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
    }
}

/// With an origin group the copy also needs source map entries for the group it copies: a limit
/// that holds them admits the copy and one entry less refuses it.
#[test]
fn a_copy_of_a_literal_with_an_origin_group_is_refused_one_source_map_entry_short() {
    let (first, second) = copy_measure(true);
    assert!(second.2 > first.2, "the copy adds an origin group");
    let maps_limit = |entries: u64| {
        limits()
            .into_builder()
            .source_map_entries(entries)
            .finish()
            .unwrap()
    };
    assert_eq!(copy_under(maps_limit(second.2), true), Ok(()));
    assert!(matches!(
        copy_under(maps_limit(second.2 - 1), true),
        Err(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::SourceMapEntries,
            ..
        })
    ));
}

/// An occurrence ticket admits units alone, so its block has other holders and the retirement
/// check skips the holder count. The ticket that charged the block keeps the check: its block is
/// held by the occurrence too, and the refusal stands.
#[test]
fn only_an_occurrence_ticket_skips_the_holder_check_of_a_retired_row() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let storage = session.active_mut().unwrap();
    let (first, charged) = storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let (second, occurrence) = storage
        .register_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    drop((first, second));
    assert_eq!(
        storage.releasable_string(&mut c, charged, false),
        Err(SessionError::AliasedPayload)
    );
    assert_eq!(
        storage.releasable_string(&mut c, occurrence, false),
        Ok(true)
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
