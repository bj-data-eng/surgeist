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
