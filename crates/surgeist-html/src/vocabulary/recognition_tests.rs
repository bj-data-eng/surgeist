//! The spelling comparison of the generated recognition, over every near miss of every member.
//!
//! A lookup reaches a member's slot for the member and for the inputs that hash to that slot. The
//! comparison of the spelling is what rejects the latter, so it is exercised here directly and
//! through the lookup, for every member of each set and every single edit of its spelling over
//! the ASCII alphabet, not only for the inputs that a sample happens to send to the slot.

use super::{
    KnownHtmlElement, KnownHtmlEventHandler, KnownMathmlElement, KnownSvgElement,
    name_hash::units_spell,
};

/// Every input one edit from `name` over the ASCII alphabet: each substitution that changes a
/// unit, each removal and each insertion.
fn near_misses(name: &str) -> Vec<Vec<u16>> {
    let units: Vec<u16> = name.encode_utf16().collect();
    let mut found = Vec::new();
    for at in 0..units.len() {
        let mut removed = units.clone();
        removed.remove(at);
        found.push(removed);
        for unit in 0..=127u16 {
            if units.get(at) != Some(&unit) {
                let mut substituted = units.clone();
                substituted[at] = unit;
                found.push(substituted);
            }
        }
    }
    for at in 0..=units.len() {
        for unit in 0..=127u16 {
            let mut inserted = units.clone();
            inserted.insert(at, unit);
            found.push(inserted);
        }
    }
    found
}

macro_rules! near_miss_tests {
    ($test:ident, $set:ident) => {
        #[test]
        fn $test() {
            for (position, member) in $set::ALL.iter().copied().enumerate() {
                let name = member.local_name();
                let spelling: Vec<u16> = name.encode_utf16().collect();
                assert!(units_spell(&spelling, name), "{name}");
                assert_eq!($set::recognize(&spelling), Some(member), "{name}");
                assert_eq!($set::selected_position(&spelling), Some(position), "{name}");
                let mut in_slot = 0;
                for near in near_misses(name) {
                    assert!(!units_spell(&near, name), "{name} {near:?}");
                    if $set::selected_position(&near) == Some(position) {
                        in_slot += 1;
                        assert_eq!($set::recognize(&near), None, "{name} {near:?}");
                    }
                }
                assert!(in_slot > 0, "no near miss of {name} reaches its slot");
            }
        }
    };
}

near_miss_tests!(html_element_near_misses_are_rejected, KnownHtmlElement);
near_miss_tests!(
    event_handler_near_misses_are_rejected,
    KnownHtmlEventHandler
);
near_miss_tests!(mathml_element_near_misses_are_rejected, KnownMathmlElement);
near_miss_tests!(svg_element_near_misses_are_rejected, KnownSvgElement);
