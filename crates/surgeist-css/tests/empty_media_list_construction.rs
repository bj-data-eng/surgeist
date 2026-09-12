#![forbid(unsafe_code)]

//! Media Queries 4 permits an empty media-query list. Checked construction
//! must preserve the same authored domain as parsing; evaluation belongs to style.
//! https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/#mq-list

use surgeist_css::{CssMediaQueryList, CssRule, parse_sheet, validate_sheet};

#[test]
fn empty_media_list_construction_matches_valid_authored_rule() {
    let source = "@media {} .after { color: red; }";
    let parsed = parse_sheet(source);
    assert!(parsed.is_clean(), "{:?}", parsed.diagnostics());
    let [CssRule::Media(media), CssRule::Style(_)] = parsed.syntax().rules() else {
        panic!("empty media rule and following sibling must survive");
    };
    assert!(media.query().queries().is_empty());
    assert_eq!(validate_sheet(source).unwrap(), *parsed.syntax());
    let constructed = CssMediaQueryList::try_new(Vec::new())
        .expect("an empty list is valid authored media syntax");
    assert!(constructed.queries().is_empty());
    assert_eq!(&constructed, media.query());
}

#[test]
fn media_list_construction_preserves_nonempty_member_order_and_recovery() {
    let parsed = parse_sheet("@media print,???,screen {}");
    let [CssRule::Media(media)] = parsed.syntax().rules() else {
        panic!("one retained media rule");
    };
    let members = media.query().queries();
    assert_eq!(members.len(), 3);
    assert!(members[1].is_guaranteed_false());
    assert!(!members[0].is_guaranteed_false());
    assert!(!members[2].is_guaranteed_false());
    let constructed = CssMediaQueryList::try_new(members.to_vec())
        .expect("valid existing members remain constructible");
    assert_eq!(constructed.queries(), members);
}
