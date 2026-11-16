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

/// The first occurrence of a literal creates and charges its string, and says it did, so that the
/// element that holds the occurrence does not count its units again. Every later occurrence
/// shares the string and charges nothing: the record that holds it counts its units, as for any
/// stored string, and the provider is asked for no string.
#[test]
fn a_later_occurrence_of_a_literal_shares_the_first_string_and_charges_nothing() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let storage = session.active_mut().unwrap();
    let before = (units(&c), provider_calls(&c));
    let (first, counted) = storage
        .shared_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let created = (units(&c) - before.0, provider_calls(&c) - before.1);
    assert_eq!(first.to_utf8().unwrap(), "http://www.w3.org/1999/xhtml");
    assert!(counted);
    // The 28 units of the URI, and a growth of the buffer for each capacity 1, 2, 4, 8, 16
    // and 32 that holds them, besides the wrapper and the row.
    assert_eq!(created.0, 28);
    assert!(created.1 >= 7, "{} provider requests", created.1);

    let before = (units(&c), provider_calls(&c));
    let (second, counted) = storage
        .shared_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    assert!(!counted);
    assert!(first.shares_storage_with(&second));
    assert_eq!((units(&c), provider_calls(&c)), before);

    // Another literal is another string.
    let (svg, counted) = storage
        .shared_literal(&mut c, Literal::SvgNamespace)
        .unwrap();
    assert!(counted);
    assert!(!svg.shares_storage_with(&first));
    drop((first, second, svg));
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

/// The units and work credits a parse holds after the first occurrence of a literal.
fn measured() -> (u64, u64) {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let storage = session.active_mut().unwrap();
    let before = (
        units(&c),
        c.usage().resource(HtmlParseResource::WorkCredits).current,
    );
    storage
        .shared_literal(&mut c, Literal::HtmlNamespace)
        .unwrap();
    let after = (
        units(&c),
        c.usage().resource(HtmlParseResource::WorkCredits).current,
    );
    session.release(&mut c).unwrap();
    (after.0 - before.0, after.1 - before.1)
}

/// The first occurrence of a literal under `limits`. The parse releases to zero.
fn first_occurrence(limits: crate::HtmlParseLimits) -> Result<(), HtmlParseStop> {
    let mut c = ParseControl::new(limits, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let storage = session.active_mut().unwrap();
    let result = match storage.shared_literal(&mut c, Literal::HtmlNamespace) {
        Ok(_) => Ok(()),
        Err(SessionError::Stop(stop)) => {
            assert_eq!(c.stop(), Some(stop));
            Err(stop)
        }
        Err(other) => panic!("refused for another reason: {other:?}"),
    };
    session.release(&mut c).unwrap();
    storage_zero(&c);
    result
}

/// The first occurrence needs exactly the 28 units of the literal: a limit that holds them admits
/// it, and one unit less refuses it with the unit stop. Its work is admitted before it runs: a
/// work limit that holds it admits it, and one credit less refuses it with the work stop.
#[test]
fn the_first_occurrence_of_a_literal_is_refused_one_unit_or_one_credit_short() {
    let (needed_units, _) = measured();
    assert_eq!(needed_units, 28);
    let base = limits();
    let with_units = |units: u64| {
        base.into_builder()
            .managed_utf16_units(units)
            .finish()
            .unwrap()
    };
    // The units the new session already holds count against the limit, so the limit is the
    // units it holds beside the literal's.
    let held = {
        let mut c = control();
        let session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let held = units(&c);
        let mut session = session;
        session.release(&mut c).unwrap();
        held
    };
    assert_eq!(first_occurrence(with_units(held + 28)), Ok(()));
    assert!(matches!(
        first_occurrence(with_units(held + 27)),
        Err(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedUtf16Units,
            ..
        })
    ));
}
