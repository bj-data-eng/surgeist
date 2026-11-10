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
