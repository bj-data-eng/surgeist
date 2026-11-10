//! Named restrictions from HTML877d #parse-a-sandboxing-directive,
//! complete retained source lines 65697–65746. Set order is not an oracle.
#![forbid(unsafe_code)]

use std::{
    collections::HashSet,
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use surgeist_html::{
    attributes::sandbox::{
        HtmlSandboxAuthorState, HtmlSandboxDirectiveState, HtmlSandboxFlag as F, HtmlSandboxFlags,
    },
    values::{Attribute, DocumentString, HtmlValueInterpreter},
    vocabulary::{KnownHtmlAttribute as A, KnownHtmlElement as E},
};

const RESTRICTIONS: [F; 16] = [
    F::Navigation,
    F::AuxiliaryNavigation,
    F::TopNavigationWithoutUserActivation,
    F::TopNavigationWithUserActivation,
    F::Origin,
    F::Forms,
    F::PointerLock,
    F::Scripts,
    F::AutomaticFeatures,
    F::DocumentDomain,
    F::PropagatesToAuxiliary,
    F::Modals,
    F::OrientationLock,
    F::Presentation,
    F::Downloads,
    F::CustomProtocolsNavigation,
];

fn parse(raw: &DocumentString) -> HtmlSandboxFlags {
    HtmlSandboxFlags::parse_with(raw, || Ok::<_, Infallible>(())).unwrap()
}

fn restrictions_except(actual: HtmlSandboxFlags, relaxed: &[F]) {
    let expected: HashSet<_> = RESTRICTIONS
        .into_iter()
        .filter(|f| !relaxed.contains(f))
        .collect();
    assert_eq!(actual.iter().collect::<HashSet<_>>(), expected);
    for flag in RESTRICTIONS {
        assert_eq!(actual.contains(flag), !relaxed.contains(&flag));
    }
    assert_eq!(actual.len(), expected.len());
    assert_eq!(actual.is_empty(), expected.is_empty());
}

#[test]
fn absent_contribution_is_empty_but_present_empty_and_ascii_space_restrict_all() {
    let absent = HtmlSandboxFlags::empty();
    assert!(absent.is_empty());
    assert_eq!(absent.len(), 0);
    for flag in RESTRICTIONS {
        assert!(!absent.contains(flag));
    }
    assert!(absent.iter().next().is_none());

    // Presence is the caller's observation, not fabricated input to parsing.
    let report = HtmlValueInterpreter::new(128)
        .iframe_sandbox(
            &E::Iframe.qualified_name(),
            None::<&surgeist_html::values::Attribute>,
        )
        .unwrap();
    assert_eq!(report.intrinsic(), Some(&HtmlSandboxDirectiveState::Absent));
    for raw in ["", " \t\n\r\u{c}"] {
        let present = parse(&raw.into());
        restrictions_except(present, &[]);
        assert_ne!(present, absent);
    }
}

#[test]
fn each_literal_source_keyword_relaxes_only_its_named_restrictions() {
    // Independent source relationships, not an enumeration of the producer's
    // recognition or projection tables used to derive expected results.
    let cases: &[(&str, &[F])] = &[
        ("allow-downloads", &[F::Downloads]),
        ("allow-forms", &[F::Forms]),
        ("allow-modals", &[F::Modals]),
        ("allow-orientation-lock", &[F::OrientationLock]),
        ("allow-pointer-lock", &[F::PointerLock]),
        (
            "allow-popups",
            &[F::AuxiliaryNavigation, F::CustomProtocolsNavigation],
        ),
        (
            "allow-popups-to-escape-sandbox",
            &[F::PropagatesToAuxiliary],
        ),
        ("allow-presentation", &[F::Presentation]),
        ("allow-same-origin", &[F::Origin]),
        ("allow-scripts", &[F::Scripts, F::AutomaticFeatures]),
        (
            "allow-top-navigation",
            &[
                F::TopNavigationWithoutUserActivation,
                F::TopNavigationWithUserActivation,
                F::CustomProtocolsNavigation,
            ],
        ),
        (
            "allow-top-navigation-by-user-activation",
            &[F::TopNavigationWithUserActivation],
        ),
        (
            "allow-top-navigation-to-custom-protocols",
            &[F::CustomProtocolsNavigation],
        ),
    ];
    for &(keyword, relaxed) in cases {
        for text in [keyword.to_owned(), keyword.to_ascii_uppercase()] {
            let raw = DocumentString::from(text);
            let flags = parse(&raw);
            restrictions_except(flags, relaxed);
            let attribute = Attribute::new(A::Sandbox.qualified_name(), raw);
            let report = HtmlValueInterpreter::new(128)
                .iframe_sandbox(&E::Iframe.qualified_name(), Some(&attribute))
                .unwrap();
            assert_eq!(
                report.intrinsic(),
                Some(&HtmlSandboxDirectiveState::Present(flags))
            );
            assert_eq!(report.author(), &HtmlSandboxAuthorState::Conforming);
        }
    }
}

#[test]
fn duplicates_unknown_tokens_and_non_ascii_preserve_only_recognized_relaxations() {
    // (value, duplicate as (first index, rejected index, rejected spelling),
    // unknown tokens as (token index, spelling)): the token indexes are read off
    // each value by hand, counting ASCII-whitespace-separated tokens from zero.
    for (text, duplicate, unknown) in [
        (
            "allow-forms allow-scripts allow-forms UNKNOWN",
            Some((0, 2, "allow-forms")),
            vec![(3, "UNKNOWN")],
        ),
        (
            "\tALLOW-forms\nallow-SCRIPTS\rallow-forms\u{c}unknown ",
            Some((0, 2, "allow-forms")),
            vec![(3, "unknown")],
        ),
        (
            "unknown allow-forms allow-scripts",
            None,
            vec![(0, "unknown")],
        ),
    ] {
        let raw = DocumentString::from(text);
        let flags = parse(&raw);
        restrictions_except(flags, &[F::Forms, F::Scripts, F::AutomaticFeatures]);
        let attribute = Attribute::new(A::Sandbox.qualified_name(), raw);
        let report = HtmlValueInterpreter::new(256)
            .iframe_sandbox(&E::Iframe.qualified_name(), Some(&attribute))
            .unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlSandboxDirectiveState::Present(flags))
        );
        let HtmlSandboxAuthorState::Nonconforming(diagnostics) = report.author() else {
            panic!("{text:?} is nonconforming");
        };
        assert_eq!(
            diagnostics.duplicate().map(|error| (
                error.first_index(),
                error.duplicate_index(),
                error.token().clone()
            )),
            duplicate.map(|(first, rejected, spelling)| (first, rejected, spelling.into())),
            "{text:?}"
        );
        assert_eq!(
            diagnostics
                .unknown_tokens()
                .iter()
                .map(|token| (token.token_index(), token.value().clone()))
                .collect::<Vec<_>>(),
            unknown
                .into_iter()
                .map(|(index, spelling)| (index, DocumentString::from(spelling)))
                .collect::<Vec<_>>(),
            "{text:?}"
        );
        assert_eq!(diagnostics.redundancies().count(), 0, "{text:?}");
    }
    // Only the five ASCII whitespace units delimit tokens; non-ASCII case
    // folding, trimming, NUL replacement and surrogate repair are not applied.
    for text in [
        "allow-forms\u{a0}allow-scripts",
        "allow-ſcripts",
        "allow-forms\u{b}allow-scripts",
        "allow-forms\0",
        "allow-scripts-suffix",
        "prefix-allow-forms",
    ] {
        restrictions_except(parse(&text.into()), &[]);
    }
    let mut units = "allow-scripts".encode_utf16().collect::<Vec<_>>();
    units.push(0xd800);
    restrictions_except(parse(&DocumentString::from_utf16(units)), &[]);
}

#[test]
fn union_retains_each_actual_parent_and_element_restriction() {
    let parent = parse(&"allow-forms allow-scripts".into());
    let element = parse(&"allow-scripts allow-same-origin".into());
    // Parent forbids Origin; element forbids Forms. Only the two restrictions
    // relaxed by BOTH contributions remain absent from their union.
    let combined = parent.union(element);
    restrictions_except(combined, &[F::Scripts, F::AutomaticFeatures]);
    assert_eq!(combined, element.union(parent));
    assert_eq!(parent.union(parent), parent);
    assert_eq!(HtmlSandboxFlags::empty().union(parent), parent);
    assert_eq!(parent.union(HtmlSandboxFlags::empty()), parent);
    restrictions_except(parent.union(parse(&"".into())), &[]);
    assert_eq!(combined.iter().count(), 14);
}

#[derive(Debug, Eq, PartialEq)]
enum Stop {
    WorkLimit { limit: usize },
    Cancelled,
}

#[test]
fn exact_admission_succeeds_and_each_one_short_refusal_returns_the_original_error() {
    let raw = DocumentString::from(" \tallow-FORMS allow-scripts unknown ");
    let mut required = 0;
    let flags = HtmlSandboxFlags::parse_with(&raw, || {
        required += 1;
        Ok::<_, Infallible>(())
    })
    .unwrap();
    restrictions_except(flags, &[F::Forms, F::Scripts, F::AutomaticFeatures]);
    assert!(required > 0, "admission precedes the raw header");
    // The generous run only locates reached fault-injection boundaries. Literal
    // source relationships above independently establish the successful value.
    // `parse_with` documents that a refusal is returned unchanged before the
    // reached action, so the refusing call is the last one: none may follow it.
    for limit in 0..required {
        let mut spent = 0;
        let mut refused = false;
        assert_eq!(
            HtmlSandboxFlags::parse_with(&raw, || {
                assert!(!refused, "no admission follows a refusal");
                if spent == limit {
                    refused = true;
                    return Err(Stop::WorkLimit { limit });
                }
                spent += 1;
                Ok(())
            }),
            Err(Stop::WorkLimit { limit })
        );
        assert!(refused);
    }
    let mut spent = 0;
    assert_eq!(
        HtmlSandboxFlags::parse_with(&raw, || {
            if spent == required {
                return Err(Stop::WorkLimit { limit: required });
            }
            spent += 1;
            Ok(())
        }),
        Ok(flags)
    );
    // Empty present input still consults the caller (the declared header,
    // terminal token step and final projection admissions), so it never
    // bypasses it. Inject a refusal at each admission it reaches, advancing the
    // refusal point until a run completes: every refusal comes back unchanged
    // with nothing after it, and the completed run is the all-restrictions value.
    let empty = DocumentString::from("");
    let mut refusals = 0;
    let completed = loop {
        let limit = refusals;
        let mut spent = 0;
        let mut refused = false;
        let result = HtmlSandboxFlags::parse_with(&empty, || {
            assert!(!refused, "no admission follows a refusal");
            if spent == limit {
                refused = true;
                return Err(Stop::WorkLimit { limit });
            }
            spent += 1;
            Ok(())
        });
        match result {
            Ok(flags) => {
                assert!(!refused);
                break flags;
            }
            Err(error) => {
                assert_eq!(error, Stop::WorkLimit { limit });
                assert!(refused);
                refusals += 1;
                assert!(
                    refusals < 1000,
                    "empty input admits only finitely many actions"
                );
            }
        }
    };
    assert!(refusals > 0, "the caller is consulted for empty input");
    restrictions_except(completed, &[]);
}

#[test]
fn cancellation_at_every_reached_action_keeps_exact_raw_backing_and_retry_value() {
    let mut units = Vec::with_capacity(4096);
    units.extend("allow-forms \tallow-scripts unknown".encode_utf16());
    let backing = Arc::new(units);
    let raw = DocumentString::from_shared_utf16(backing.clone());
    let holders = Arc::strong_count(&backing);
    let mut required = 0;
    let expected = HtmlSandboxFlags::parse_with(&raw, || {
        required += 1;
        Ok::<_, Infallible>(())
    })
    .unwrap();
    restrictions_except(expected, &[F::Forms, F::Scripts, F::AutomaticFeatures]);
    for cancel_before in 0..required {
        let cancel = AtomicBool::new(false);
        let mut admitted = 0;
        assert_eq!(
            HtmlSandboxFlags::parse_with(&raw, || {
                if admitted == cancel_before {
                    cancel.store(true, Ordering::Relaxed);
                }
                if cancel.load(Ordering::Relaxed) {
                    return Err(Stop::Cancelled);
                }
                admitted += 1;
                Ok(())
            }),
            Err(Stop::Cancelled)
        );
        assert_eq!(admitted, cancel_before);
        assert_eq!(Arc::strong_count(&backing), holders);
        assert!(Arc::ptr_eq(&raw.to_shared_utf16(), &backing));
    }
    assert_eq!(parse(&raw), expected);
    assert_eq!(raw.code_units(), backing.as_slice());
}
