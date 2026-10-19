//! Literal HTML877d CORS-settings and referrer-policy content-state oracles.
//! Applicability, parser ownership and resource execution belong to callers.
#![forbid(unsafe_code)]

use std::convert::Infallible;
use surgeist_html::bounded::{
    HtmlAttributeKeywordRead as Read, crossorigin_processing_with, referrerpolicy_processing_with,
};
use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeAuthorState as Author,
    HtmlAttributeRecognition as Recognition, HtmlCrossOrigin, HtmlCrossOriginState, HtmlOlType,
    HtmlReferrerPolicy, HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement,
};

fn cors(raw: Option<&DocumentString>) -> HtmlCrossOriginState {
    crossorigin_processing_with(raw, |_| Ok::<_, Infallible>(())).unwrap()
}
fn policy(raw: Option<&DocumentString>) -> HtmlReferrerPolicy {
    referrerpolicy_processing_with(raw, |_| Ok::<_, Infallible>(())).unwrap()
}

#[test]
fn cors_missing_empty_keywords_and_invalid_values_keep_their_distinct_defaults() {
    use HtmlCrossOriginState::{Keyword, NoCors};
    assert_eq!(cors(None), NoCors);
    for (literal, expected) in [
        ("", Keyword(HtmlCrossOrigin::Anonymous)),
        ("anonymous", Keyword(HtmlCrossOrigin::Anonymous)),
        ("AnOnYmOuS", Keyword(HtmlCrossOrigin::Anonymous)),
        ("use-credentials", Keyword(HtmlCrossOrigin::UseCredentials)),
        ("UsE-CrEdEnTiAlS", Keyword(HtmlCrossOrigin::UseCredentials)),
        ("credentials", Keyword(HtmlCrossOrigin::Anonymous)),
        (" use-credentials", Keyword(HtmlCrossOrigin::Anonymous)),
        ("use-credentials ", Keyword(HtmlCrossOrigin::Anonymous)),
        ("use-credentials\0", Keyword(HtmlCrossOrigin::Anonymous)),
        (
            "\u{ff55}se-credentials",
            Keyword(HtmlCrossOrigin::Anonymous),
        ),
    ] {
        let raw = DocumentString::from(literal);
        let before = raw.clone();
        assert_eq!(cors(Some(&raw)), expected, "{literal:?}");
        assert_eq!(raw, before);
    }
    for raw in [
        DocumentString::from_utf16([0xd800]),
        DocumentString::from_utf16([0xdc00]),
        DocumentString::from_utf16([0x75, 0xd800, 0x65]),
    ] {
        assert_eq!(cors(Some(&raw)), Keyword(HtmlCrossOrigin::Anonymous));
    }
}

#[test]
fn policy_uses_only_the_pinned_keywords_and_leaves_empty_default_unresolved() {
    use HtmlReferrerPolicy::*;
    assert_eq!(policy(None), Empty);
    assert_eq!(
        policy(Some(&DocumentString::from("StRiCt-OrIgIn"))),
        StrictOrigin
    );
    for (literal, expected) in [
        ("", Empty),
        ("no-referrer", NoReferrer),
        ("no-referrer-when-downgrade", NoReferrerWhenDowngrade),
        ("same-origin", SameOrigin),
        ("origin", Origin),
        ("strict-origin", StrictOrigin),
        ("origin-when-cross-origin", OriginWhenCrossOrigin),
        (
            "strict-origin-when-cross-origin",
            StrictOriginWhenCrossOrigin,
        ),
        ("unsafe-url", UnsafeUrl),
    ] {
        for spelling in [literal.to_owned(), literal.to_ascii_uppercase()] {
            let raw = DocumentString::from(spelling);
            assert_eq!(policy(Some(&raw)), expected, "{literal}");
        }
    }
    for literal in [
        "default",
        "never",
        "unknown",
        " origin",
        "origin ",
        "\tno-referrer",
        "no-referrer\n",
        "origin, no-referrer",
        "no-referrer unsafe-url",
        "\u{ff2f}rigin",
    ] {
        assert_eq!(
            policy(Some(&DocumentString::from(literal))),
            Empty,
            "{literal:?}"
        );
    }
    assert_eq!(policy(Some(&DocumentString::from_utf16([0xd800]))), Empty);
}

#[test]
fn missing_empty_and_matching_values_admit_the_actual_headers_and_pairs() {
    let mut observed = Vec::new();
    assert_eq!(
        crossorigin_processing_with(None, |read| {
            observed.push(read);
            Ok::<_, Infallible>(())
        })
        .unwrap(),
        HtmlCrossOriginState::NoCors
    );
    assert_eq!(observed, [Read::ValueHeader]);

    observed.clear();
    assert_eq!(
        cors_record(&DocumentString::default(), &mut observed),
        HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous)
    );
    assert_eq!(
        observed,
        [Read::ValueHeader, Read::KeywordHeader, Read::KeywordHeader]
    );

    observed.clear();
    assert_eq!(
        cors_record(&DocumentString::from("anonymous"), &mut observed),
        HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous)
    );
    let mut expected = vec![Read::ValueHeader, Read::KeywordHeader];
    expected.extend([Read::KeywordPair; 9]);
    assert_eq!(observed, expected);

    observed.clear();
    assert_eq!(
        referrerpolicy_processing_with(Some(&DocumentString::default()), |read| {
            observed.push(read);
            Ok::<_, Infallible>(())
        })
        .unwrap(),
        HtmlReferrerPolicy::Empty
    );
    assert_eq!(observed, [Read::ValueHeader, Read::KeywordHeader]);
}

fn cors_record(raw: &DocumentString, observed: &mut Vec<Read>) -> HtmlCrossOriginState {
    crossorigin_processing_with(Some(raw), |read| {
        observed.push(read);
        Ok::<_, Infallible>(())
    })
    .unwrap()
}

#[test]
fn every_refusal_stops_at_the_exact_admitted_operand_and_preserves_error() {
    for raw in [
        None,
        Some(DocumentString::default()),
        Some("UsE-CrEdEnTiAlS".into()),
        Some("strict-origin-when-cross-origin".into()),
        Some(DocumentString::from_utf16([0xd800; 9])),
    ] {
        let mut cors_reads = Vec::new();
        crossorigin_processing_with(raw.as_ref(), |read| {
            cors_reads.push(read);
            Ok::<_, Infallible>(())
        })
        .unwrap();
        let mut policy_reads = Vec::new();
        referrerpolicy_processing_with(raw.as_ref(), |read| {
            policy_reads.push(read);
            Ok::<_, Infallible>(())
        })
        .unwrap();
        for (is_cors, reads) in [(true, cors_reads), (false, policy_reads)] {
            for (cut, &denied) in reads.iter().enumerate() {
                let mut observed = Vec::new();
                let mut admit = |read| {
                    observed.push(read);
                    if observed.len() == cut + 1 {
                        Err((cut, read, "cancelled"))
                    } else {
                        Ok(())
                    }
                };
                let result = if is_cors {
                    crossorigin_processing_with(raw.as_ref(), &mut admit).map(|_| ())
                } else {
                    referrerpolicy_processing_with(raw.as_ref(), &mut admit).map(|_| ())
                };
                assert_eq!(result, Err((cut, denied, "cancelled")));
                assert_eq!(observed, reads[..=cut]);
            }
        }
    }
}

#[test]
fn exact_budget_succeeds_and_one_short_refuses_before_returning_a_state() {
    let raw = DocumentString::from("anonymous");
    // Presence header, first keyword header and nine UTF-16/ASCII pairs.
    for (budget, succeeds) in [(29usize, true), (28, false), (0, false)] {
        let mut remaining = budget;
        let result = crossorigin_processing_with(Some(&raw), |read| {
            let cost = if read == Read::KeywordPair { 3 } else { 1 };
            remaining = remaining.checked_sub(cost).ok_or("admission exhausted")?;
            Ok(())
        });
        if succeeds {
            assert_eq!(
                result,
                Ok(HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous))
            );
            assert_eq!(remaining, 0);
        } else {
            assert_eq!(result, Err("admission exhausted"));
        }
    }
    assert_eq!(
        referrerpolicy_processing_with(None, |_| Err("cancelled")),
        Err("cancelled")
    );
    let raw = DocumentString::from("origin");
    // Presence, five keyword headers and six actual UTF-16/ASCII pairs.
    for (budget, succeeds) in [(24usize, true), (23, false)] {
        let mut remaining = budget;
        let result = referrerpolicy_processing_with(Some(&raw), |read| {
            let cost = if read == Read::KeywordPair { 3 } else { 1 };
            remaining = remaining.checked_sub(cost).ok_or("admission exhausted")?;
            Ok(())
        });
        if succeeds {
            assert_eq!(result, Ok(HtmlReferrerPolicy::Origin));
            assert_eq!(remaining, 0);
        } else {
            assert_eq!(result, Err("admission exhausted"));
        }
    }
}

#[test]
fn long_length_mismatches_read_only_fixed_keyword_headers() {
    let raw = DocumentString::from_utf16(vec![0xd800; 100_000]);
    let mut observed = Vec::new();
    assert_eq!(
        cors_record(&raw, &mut observed),
        HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous)
    );
    assert_eq!(
        observed,
        [Read::ValueHeader, Read::KeywordHeader, Read::KeywordHeader]
    );
    observed.clear();
    assert_eq!(
        referrerpolicy_processing_with(Some(&raw), |read| {
            observed.push(read);
            Ok::<_, Infallible>(())
        })
        .unwrap(),
        HtmlReferrerPolicy::Empty
    );
    assert_eq!(observed.first(), Some(&Read::ValueHeader));
    assert_eq!(observed[1..], [Read::KeywordHeader; 9]);
}

#[test]
fn shared_selector_preserves_ordinary_exact_case_and_author_reports() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (literal, expected) in [
        ("1", HtmlOlType::Decimal),
        ("a", HtmlOlType::LowerAlpha),
        ("A", HtmlOlType::UpperAlpha),
        ("i", HtmlOlType::LowerRoman),
        ("I", HtmlOlType::UpperRoman),
    ] {
        let raw = Attribute::new(KnownHtmlAttribute::Type.qualified_name(), literal);
        let report = interpreter
            .ol_type(&KnownHtmlElement::Ol.qualified_name(), Some(&raw))
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&expected));
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(
            report.recognition(),
            Recognition::Keyword {
                matched: literal,
                canonical: literal
            }
        );
    }
    let img = KnownHtmlElement::Img.qualified_name();
    for literal in ["", "AnOnYmOuS", "use-credentials", "bad"] {
        let raw = Attribute::new(KnownHtmlAttribute::Crossorigin.qualified_name(), literal);
        assert_eq!(
            interpreter
                .crossorigin(&img, Some(&raw))
                .unwrap()
                .intrinsic(),
            Some(&cors(Some(raw.value())))
        );
    }
}
