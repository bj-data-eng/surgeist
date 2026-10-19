use crate::semantics::support::builder;
use crate::semantics::support::context;
use crate::semantics::support::session;
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlMetaContentSlice,
    HtmlMetaEncodingCharsetConflict, HtmlMetaPragmaContent, HtmlMetaPragmaContentAuthor,
    HtmlMetaPragmaContentReport, HtmlMetaPragmaUniqueness, HtmlMetaRefreshLexical,
    HtmlMetaRefreshTarget, HtmlMetaRefreshValue, HtmlScriptingStatus, HtmlSemanticCancellation,
    HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing,
    HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource,
    HtmlSemanticUnsupported, HtmlSemanticValue,
};
// Literal expectations come from HTML 877d #pragma-directives, not HTTP headers.

fn attrs(fields: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(
        fields
            .iter()
            .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
    a: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name,
        a,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn meta(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    fields: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(b, d, KnownHtmlElement::Meta.qualified_name(), attrs(fields))
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 10000, 1000000, 100000)
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::MetaPragmaContent)
}

fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId, raw: usize) -> HtmlSemanticReport {
    session(
        c,
        &context(c),
        limits(),
        raw,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(id)])
    .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlMetaPragmaContentReport {
    match r.results()[0].fact() {
        HtmlSemanticFact::MetaPragmaContent(HtmlSemanticValue::Known(v)) => v,
        x => panic!("{x:?}"),
    }
}
fn single(fields: &[(&str, &str)]) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, fields);
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    (c, id)
}
fn language(r: &HtmlSemanticReport) -> Option<&[u16]> {
    match value(r).interpretation() {
        HtmlMetaPragmaContent::ContentLanguage { candidate } => {
            candidate.as_ref().map(HtmlMetaContentSlice::code_units)
        }
        x => panic!("{x:?}"),
    }
}
fn refresh(r: &HtmlSemanticReport) -> &HtmlMetaRefreshLexical {
    match value(r).interpretation() {
        HtmlMetaPragmaContent::Refresh(v) => v,
        x => panic!("{x:?}"),
    }
}
fn parsed(r: &HtmlSemanticReport) -> &HtmlMetaRefreshValue {
    match refresh(r) {
        HtmlMetaRefreshLexical::Parsed(v) => v,
        x => panic!("{x:?}"),
    }
}
fn suffix(v: &HtmlMetaRefreshValue) -> Option<&[u16]> {
    match v.target() {
        HtmlMetaRefreshTarget::DocumentUrl => None,
        HtmlMetaRefreshTarget::UrlString(v) => Some(v.code_units()),
    }
}

#[test]
fn absent_empty_invalid_http_equiv_delivers_no_state_not_a_default() {
    for fields in [
        vec![],
        vec![("http-equiv", "")],
        vec![("http-equiv", " refresh")],
        vec![("http-equiv", "invalid")],
    ] {
        let (c, id) = single(&fields);
        let r = report(&c, &id, 30);
        let v = value(&r);
        assert_eq!(v.http_equiv().intrinsic(), None);
        assert_eq!(v.interpretation(), &HtmlMetaPragmaContent::NoState);
        assert_eq!(v.content_author(), None);
        assert_eq!(v.uniqueness(), &HtmlMetaPragmaUniqueness::NotRequired);
        assert_eq!(
            v.charset_conflict(),
            &HtmlMetaEncodingCharsetConflict::NotRequired
        );
    }
}
#[test]
fn content_language_rejects_any_comma_and_selects_only_first_ascii_space_token() {
    for (input, expected) in [
        ("en-US", Some("en-US")),
        (" \t\n\r\u{c}en-US fr", Some("en-US")),
        ("en-US,fr", None),
        ("en-US rest,ignored", None),
        ("", None),
        (" \t\n", None),
        ("\u{a0}en rest", Some("\u{a0}en")),
    ] {
        let (c, id) = single(&[("http-equiv", "CONTENT-LANGUAGE"), ("content", input)]);
        let r = report(&c, &id, 100);
        assert_eq!(
            language(&r),
            expected
                .map(DocumentString::from)
                .as_ref()
                .map(DocumentString::code_units)
        );
        assert_eq!(
            value(&r).http_equiv().author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::NonconformingKeyword
            )
        );
    }
}
#[test]
fn language_token_and_raw_content_preserve_exact_surrogates_null_and_ranges() {
    let (mut b, d) = builder();
    let raw = DocumentString::from_utf16(vec![32, 0xd800, 0, 0xdc00, 0xa0, 9, 88]);
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        OrderedAttributes::new([
            Attribute::new(
                KnownHtmlAttribute::HttpEquiv.qualified_name(),
                "content-language",
            ),
            Attribute::new(KnownHtmlAttribute::Content.qualified_name(), raw.clone()),
        ])
        .unwrap(),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 16);
    let HtmlMetaPragmaContent::ContentLanguage { candidate: Some(s) } = value(&r).interpretation()
    else {
        panic!()
    };
    assert_eq!(s.code_units(), &[0xd800, 0, 0xdc00, 0xa0]);
    assert_eq!(s.code_unit_range(), 1..5);
    assert_eq!(s.raw_content(), &raw);
    assert_eq!(value(&r).content_attribute().unwrap().value(), &raw);
}
#[test]
fn default_style_is_exact_nonempty_name_including_all_whitespace() {
    for input in [None, Some(""), Some(" \t"), Some(" Preferred ")] {
        let mut fields = vec![("http-equiv", "default-style")];
        if let Some(input) = input {
            fields.push(("content", input));
        }
        let (c, id) = single(&fields);
        let r = report(&c, &id, 20);
        let HtmlMetaPragmaContent::DefaultStyle { candidate } = value(&r).interpretation() else {
            panic!()
        };
        assert_eq!(
            candidate.as_ref().map(HtmlMetaContentSlice::code_units),
            input
                .filter(|s| !s.is_empty())
                .map(DocumentString::from)
                .as_ref()
                .map(DocumentString::code_units)
        );
        assert_eq!(
            value(&r).content_author(),
            Some(if input.is_some() {
                HtmlMetaPragmaContentAuthor::Satisfied
            } else {
                HtmlMetaPragmaContentAuthor::MissingContent
            })
        );
    }
}
#[test]
fn encoding_content_has_exact_html_pattern_not_generic_mime_grammar() {
    for (input, valid) in [
        ("text/html;charset=utf-8", true),
        ("TEXT/HTML; \t\r\n\u{c}CHARSET=UTF-8", true),
        ("text/html; charset=utf8", false),
        (" text/html;charset=utf-8", false),
        ("text/html ;charset=utf-8", false),
        ("text/html;charset=utf-8 ", false),
        ("text/html;charset=utf-8;x=y", false),
        ("text/html;\u{a0}charset=utf-8", false),
        ("", false),
    ] {
        let (c, id) = single(&[("http-equiv", "content-type"), ("content", input)]);
        let r = report(&c, &id, 100);
        assert_eq!(
            value(&r).content_author(),
            Some(if valid {
                HtmlMetaPragmaContentAuthor::Satisfied
            } else {
                HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax
            })
        );
        assert_eq!(
            value(&r).interpretation(),
            &HtmlMetaPragmaContent::EncodingDeclaration
        );
    }
}
#[test]
fn ignored_states_keep_independent_keyword_and_content_author_observations() {
    for (input, valid) in [
        ("IE=edge", true),
        ("ie=EDGE", true),
        (" IE=edge", false),
        ("IE=edge ", false),
        ("IE=11", false),
        ("", false),
    ] {
        let (c, id) = single(&[("http-equiv", "x-ua-compatible"), ("content", input)]);
        let r = report(&c, &id, 20);
        assert_eq!(
            value(&r).interpretation(),
            &HtmlMetaPragmaContent::IgnoredXUaCompatible
        );
        assert_eq!(
            value(&r).content_author(),
            Some(if valid {
                HtmlMetaPragmaContentAuthor::Satisfied
            } else {
                HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax
            })
        );
    }
    let (c, id) = single(&[
        ("http-equiv", "set-cookie"),
        ("content", "not parsed cookie data"),
    ]);
    let r = report(&c, &id, 30);
    assert_eq!(
        value(&r).interpretation(),
        &HtmlMetaPragmaContent::IgnoredSetCookie
    );
    assert_eq!(
        value(&r).http_equiv().author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::NonconformingKeyword
        )
    );
}
#[test]
fn every_selected_state_preserves_missing_content_without_defaulting_presence() {
    for state in [
        "content-language",
        "content-type",
        "default-style",
        "refresh",
        "set-cookie",
        "x-ua-compatible",
        "content-security-policy",
    ] {
        let (c, id) = single(&[("http-equiv", state)]);
        let r = report(&c, &id, 30);
        assert_eq!(value(&r).content_attribute(), None);
        assert_eq!(
            value(&r).content_author(),
            Some(HtmlMetaPragmaContentAuthor::MissingContent)
        );
    }
}
#[test]
fn refresh_recovery_delay_and_default_document_url_are_exact() {
    for (input, seconds, author) in [
        ("0", "0", HtmlMetaPragmaContentAuthor::Satisfied),
        ("00042", "42", HtmlMetaPragmaContentAuthor::Satisfied),
        (
            " \t42",
            "42",
            HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax,
        ),
        (".5", "0", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
        (".", "0", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
        ("1.5", "1", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
        ("1..2", "1", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
        ("10;", "10", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
        (
            "10, \t",
            "10",
            HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax,
        ),
        ("10  ", "10", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
    ] {
        let (c, id) = single(&[("http-equiv", "refresh"), ("content", input)]);
        let r = report(&c, &id, 30);
        let v = parsed(&r);
        assert_eq!(v.delay_seconds().decimal_digits(), seconds);
        assert_eq!(suffix(v), None);
        assert_eq!(value(&r).content_author(), Some(author));
    }
}
#[test]
fn refresh_recovery_preserves_source_url_prefix_jumps_and_quote_truncation() {
    for (input, expected) in [
        ("2; URL=x", "x"),
        ("2; uRl = \t'x'ignored", "x"),
        ("2; 'x'ignored", "x"),
        ("2; \"x\"ignored", "x"),
        ("2; 'x", "x"),
        ("2; URx", "URx"),
        ("2; Ux", "Ux"),
        ("2; URLx", "URLx"),
        ("2; URL  x", "URL  x"),
        ("2; x  ", "x  "),
        ("2; URL=", ""),
        ("2; URL=''ignored", ""),
        ("2, URL=x", "x"),
        ("2 URL=x", "x"),
        ("2;URL=x", "x"),
    ] {
        let (c, id) = single(&[("http-equiv", "refresh"), ("content", input)]);
        let r = report(&c, &id, 100);
        assert_eq!(parsed(&r).delay_seconds().decimal_digits(), "2");
        assert_eq!(
            suffix(parsed(&r)),
            Some(DocumentString::from(expected).code_units())
        );
    }
}
#[test]
fn refresh_strict_author_frame_is_independent_of_recovery_and_url_validity() {
    for (input, expected) in [
        (
            "10; URL=x",
            HtmlMetaPragmaContentAuthor::ExternalGrammarUnchecked,
        ),
        (
            "10; \turl=",
            HtmlMetaPragmaContentAuthor::ExternalGrammarUnchecked,
        ),
        (
            "10; URL=not a validated URL",
            HtmlMetaPragmaContentAuthor::ExternalGrammarUnchecked,
        ),
        ("10;URL=x", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
        ("10; URL =x", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
        (
            "10; URL='x'",
            HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax,
        ),
        ("10, URL=x", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
        ("1.2; URL=x", HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax),
    ] {
        let (c, id) = single(&[("http-equiv", "refresh"), ("content", input)]);
        let r = report(&c, &id, 100);
        assert_eq!(value(&r).content_author(), Some(expected));
        assert!(matches!(refresh(&r), HtmlMetaRefreshLexical::Parsed(_)));
    }
}
#[test]
fn refresh_lexical_rejections_do_not_become_a_zero_or_document_url_default() {
    for input in [
        "",
        " \t",
        "+1",
        "-0",
        "-1",
        "a",
        "1x",
        "1/URL=x",
        "1\u{a0}URL=x",
    ] {
        let (c, id) = single(&[("http-equiv", "refresh"), ("content", input)]);
        let r = report(&c, &id, 30);
        assert_eq!(refresh(&r), &HtmlMetaRefreshLexical::Rejected);
        assert_eq!(
            value(&r).content_author(),
            Some(HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax)
        );
    }
}
#[test]
fn refresh_arbitrary_delay_and_exact_utf16_url_are_not_flattened() {
    let quantity = "123456789012345678901234567890123456789012345678901234567890";
    let (c, id) = single(&[("http-equiv", "refresh"), ("content", quantity)]);
    let r = report(&c, &id, 100);
    assert_eq!(parsed(&r).delay_seconds().decimal_digits(), quantity);
    let (mut b, d) = builder();
    let raw = DocumentString::from_utf16(vec![
        48, 59, 32, 85, 82, 76, 61, 39, 0xd800, 0, 0xdc00, 39, 88,
    ]);
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        OrderedAttributes::new([
            Attribute::new(KnownHtmlAttribute::HttpEquiv.qualified_name(), "refresh"),
            Attribute::new(KnownHtmlAttribute::Content.qualified_name(), raw.clone()),
        ])
        .unwrap(),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 13);
    assert_eq!(suffix(parsed(&r)), Some(&[0xd800, 0, 0xdc00][..]));
}
#[test]
fn csp_head_relation_and_content_do_not_fake_delegated_policy_parsing() {
    for head in [true, false] {
        let (mut b, d) = builder();
        let p = element(
            &mut b,
            &d,
            if head {
                KnownHtmlElement::Head.qualified_name()
            } else {
                KnownHtmlElement::Div.qualified_name()
            },
            attrs(&[]),
        );
        let a = meta(
            &mut b,
            &d,
            &[
                ("http-equiv", "content-security-policy"),
                ("content", "report-uri bad; sandbox"),
            ],
        );
        children(&mut b, &a, &[]);
        children(&mut b, &p, std::slice::from_ref(&a));
        children(&mut b, &d, std::slice::from_ref(&p));
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&a).unwrap(), 40);
        assert_eq!(
            value(&r).interpretation(),
            &HtmlMetaPragmaContent::ContentSecurityPolicy {
                head_parent: HtmlSemanticValue::Known(head),
                nonempty_content: true
            }
        );
        assert_eq!(
            value(&r).content_author(),
            Some(HtmlMetaPragmaContentAuthor::ExternalGrammarUnchecked)
        );
    }
}
#[test]
fn csp_unknown_parent_is_missing_even_with_empty_present_content() {
    let (mut b, d) = builder();
    let a = meta(
        &mut b,
        &d,
        &[("http-equiv", "content-security-policy"), ("content", "")],
    );
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let r = report(&c, &id, 30);
    assert_eq!(
        value(&r).interpretation(),
        &HtmlMetaPragmaContent::ContentSecurityPolicy {
            head_parent: HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id.clone())),
            nonempty_content: false
        }
    );
    assert_eq!(
        value(&r).uniqueness(),
        &HtmlMetaPragmaUniqueness::Missing(HtmlSemanticMissing::Parent(id))
    );
}
#[test]
fn same_state_competitor_without_content_counts_but_different_state_does_not() {
    for (competitor, duplicate) in [("REFRESH", true), ("default-style", false)] {
        let (mut b, d) = builder();
        let a = meta(&mut b, &d, &[("http-equiv", "refresh"), ("content", "0")]);
        let z = meta(&mut b, &d, &[("http-equiv", competitor)]);
        children(&mut b, &a, &[]);
        children(&mut b, &z, &[]);
        children(&mut b, &d, &[a.clone(), z.clone()]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&a).unwrap(), 30);
        assert_eq!(
            value(&r).uniqueness(),
            &if duplicate {
                HtmlMetaPragmaUniqueness::Duplicate(c.subject_id(&z).unwrap())
            } else {
                HtmlMetaPragmaUniqueness::Unique
            }
        );
    }
}
#[test]
fn duplicate_pragma_and_charset_conflict_are_independent_even_on_same_subject() {
    for (duplicate, charset) in [(false, false), (true, false), (false, true), (true, true)] {
        let (mut b, d) = builder();
        let a = meta(
            &mut b,
            &d,
            &[
                ("http-equiv", "content-type"),
                ("content", "text/html;charset=utf-8"),
            ],
        );
        let z = meta(
            &mut b,
            &d,
            if duplicate {
                &[("http-equiv", "content-type")]
            } else {
                &[]
            },
        );
        let k = meta(
            &mut b,
            &d,
            if charset {
                &[("charset", "invalid encoding and deliberately over cap")]
            } else {
                &[]
            },
        );
        for n in [&a, &z, &k] {
            children(&mut b, n, &[]);
        }
        children(&mut b, &d, &[a.clone(), z.clone(), k.clone()]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&a).unwrap(), 23);
        assert_eq!(
            value(&r).uniqueness(),
            &if duplicate {
                HtmlMetaPragmaUniqueness::Duplicate(c.subject_id(&z).unwrap())
            } else {
                HtmlMetaPragmaUniqueness::Unique
            }
        );
        assert_eq!(
            value(&r).charset_conflict(),
            &if charset {
                HtmlMetaEncodingCharsetConflict::Present(c.subject_id(&k).unwrap())
            } else {
                HtmlMetaEncodingCharsetConflict::Absent
            }
        );
    }
    let (c, id) = single(&[
        ("http-equiv", "content-type"),
        ("charset", ""),
        ("content", ""),
    ]);
    let r = report(&c, &id, 12);
    assert_eq!(
        value(&r).charset_conflict(),
        &HtmlMetaEncodingCharsetConflict::Present(id)
    );
    assert_eq!(
        value(&r).content_author(),
        Some(HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax)
    );
}
#[test]
fn known_document_positive_witnesses_can_resolve_gaps_independently() {
    let (mut b, d) = builder();
    let a = meta(
        &mut b,
        &d,
        &[("http-equiv", "content-type"), ("content", "")],
    );
    let gap = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        attrs(&[]),
    );
    let z = meta(&mut b, &d, &[("http-equiv", "CONTENT-TYPE")]);
    for n in [&a, &z] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), gap.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 12);
    assert_eq!(
        value(&r).uniqueness(),
        &HtmlMetaPragmaUniqueness::Duplicate(c.subject_id(&z).unwrap())
    );
    assert_eq!(
        value(&r).charset_conflict(),
        &HtmlMetaEncodingCharsetConflict::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&gap).unwrap()
        ))
    );
}
#[test]
fn detached_and_unknown_ancestry_do_not_infer_document_from_node_document() {
    for established in [true, false] {
        let (mut b, d) = builder();
        let root = element(
            &mut b,
            &d,
            KnownHtmlElement::Div.qualified_name(),
            attrs(&[]),
        );
        let a = meta(&mut b, &d, &[("http-equiv", "refresh"), ("content", "0")]);
        let z = meta(&mut b, &d, &[("http-equiv", "refresh")]);
        for n in [&a, &z] {
            children(&mut b, n, &[]);
        }
        children(&mut b, &root, &[a.clone(), z]);
        if established {
            b.establish_root(&root).unwrap();
        }
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&a).unwrap(), 7);
        assert_eq!(
            value(&r).uniqueness(),
            &if established {
                HtmlMetaPragmaUniqueness::NoDocumentMetadataScope
            } else {
                HtmlMetaPragmaUniqueness::Missing(HtmlSemanticMissing::Parent(
                    c.subject_id(&root).unwrap(),
                ))
            }
        );
    }
}
#[test]
fn other_document_and_template_contents_do_not_join_pragma_population() {
    let (mut b, d) = builder();
    let e = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let a = meta(&mut b, &d, &[("http-equiv", "refresh"), ("content", "0")]);
    let z = meta(&mut b, &e, &[("http-equiv", "refresh")]);
    let t = element(
        &mut b,
        &d,
        KnownHtmlElement::Template.qualified_name(),
        attrs(&[]),
    );
    let f = b.add_fragment(&d).unwrap();
    b.establish_template_contents(&t, &f).unwrap();
    let inner = meta(&mut b, &d, &[("http-equiv", "refresh")]);
    for n in [&a, &z, &t, &inner] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &f, std::slice::from_ref(&inner));
    children(&mut b, &d, &[a.clone(), t]);
    children(&mut b, &e, std::slice::from_ref(&z));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap(), 7)).uniqueness(),
        &HtmlMetaPragmaUniqueness::Unique
    );
    assert_eq!(
        value(&report(&c, &c.subject_id(&inner).unwrap(), 7)).uniqueness(),
        &HtmlMetaPragmaUniqueness::NoDocumentMetadataScope
    );
}
#[test]
fn irrelevant_competitor_content_and_foreign_meta_do_not_influence() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, &[("http-equiv", "set-cookie")]);
    let z = meta(
        &mut b,
        &d,
        &[
            ("http-equiv", "SET-COOKIE"),
            ("content", "far too long to fit raw limit"),
        ],
    );
    let foreign = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "meta"),
        attrs(&[("http-equiv", "far too long to fit raw limit")]),
    );
    for n in [&a, &z, &foreign] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), foreign, z.clone()]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 10);
    assert_eq!(
        value(&r).uniqueness(),
        &HtmlMetaPragmaUniqueness::Duplicate(c.subject_id(&z).unwrap())
    );
}
fn capped(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(
        if resource == HtmlSemanticResource::Work {
            cap
        } else {
            20000000
        },
        if resource == HtmlSemanticResource::OutputRecords {
            cap
        } else {
            100
        },
        if resource == HtmlSemanticResource::ScratchRecords {
            cap
        } else {
            1000
        },
        if resource == HtmlSemanticResource::IntegerDigits {
            cap
        } else {
            10000
        },
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            1000000
        },
        if resource == HtmlSemanticResource::NestedOutputRecords {
            cap
        } else {
            100000
        },
    )
}
#[test]
fn refresh_exact_resources_and_one_short_caps_are_sticky_and_cleanup_scratch() {
    let (c, id) = single(&[("http-equiv", "refresh"), ("content", "12345; URL=x")]);
    let cx = context(&c);
    let usage = report(&c, &id, 30).usage();
    for (resource, exact) in [
        (HtmlSemanticResource::Work, usage.work()),
        (HtmlSemanticResource::OutputRecords, usage.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            usage.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            usage.allocated_bytes(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            usage.nested_output_records(),
        ),
        (HtmlSemanticResource::IntegerDigits, 5),
    ] {
        for cap in [exact, exact - 1] {
            let mut s = session(
                &c,
                &cx,
                capped(resource, cap),
                30,
                HtmlSemanticCancellation::new(),
            );
            let r = s.query(&[request(&id)]);
            if cap == exact {
                assert_eq!(
                    parsed(&r.unwrap()).delay_seconds().decimal_digits(),
                    "12345"
                );
            } else {
                let fail = r.unwrap_err();
                assert!(
                    matches!(fail.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if *r==resource),
                    "{fail:?}"
                );
                assert_eq!(s.query(&[]).unwrap_err(), fail);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn late_raw_failure_is_atomic_and_never_rescued_by_later_state_witness() {
    let (mut b, d) = builder();
    let a = meta(
        &mut b,
        &d,
        &[("http-equiv", "default-style"), ("content", "x")],
    );
    let z = meta(
        &mut b,
        &d,
        &[
            ("http-equiv", "default-style"),
            ("content", "01234567890123456789"),
        ],
    );
    for n in [&a, &z] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 13, HtmlSemanticCancellation::new());
    let fail = s
        .query(&[
            request(&c.subject_id(&a).unwrap()),
            request(&c.subject_id(&z).unwrap()),
        ])
        .unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(s.usage().scratch_records(), 0);
}
#[test]
fn cancellation_after_a_retained_report_stops_new_work_without_mutating_old_value() {
    let (c, id) = single(&[("http-equiv", "refresh"), ("content", "7; URL=x")]);
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 30, cancel.clone());
    let r = s.query(&[request(&id)]).unwrap();
    cancel.cancel();
    let fail = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(fail.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(fail.qualification().unwrap().capture(), c.identity());
    assert_eq!(fail.qualification().unwrap().context(), cx.identity());
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(parsed(&r).delay_seconds().decimal_digits(), "7");
    assert_eq!(suffix(parsed(&r)), Some(&[120][..]));
}
#[test]
fn complete_raw_admission_is_utf16_based_even_when_no_intrinsic_state() {
    let (c, id) = single(&[("http-equiv", "other"), ("content", "123456")]);
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new());
    assert!(matches!(
        s.query(&[request(&id)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
}
#[test]
fn namespaces_and_late_foreign_capture_requests_are_not_silently_applied() {
    let (mut b, d) = builder();
    let a = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "meta"),
        attrs(&[("http-equiv", "refresh")]),
    );
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let r = report(&c, &id, 7);
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::MetaPragmaContent(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    ));
    let (c, id) = single(&[("http-equiv", "default-style")]);
    let (other, foreign) = single(&[]);
    assert_ne!(c.identity(), other.identity());
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 13, HtmlSemanticCancellation::new());
    assert!(s.query(&[request(&id), request(&foreign)]).is_err());
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn both_population_witnesses_after_unknown_subtree_resolve_independent_missing() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, &[("http-equiv", "content-type")]);
    let gap = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        attrs(&[]),
    );
    let witness = meta(
        &mut b,
        &d,
        &[("http-equiv", "CONTENT-TYPE"), ("charset", "")],
    );
    for n in [&a, &witness] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), gap, witness.clone()]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 12);
    let expected = c.subject_id(&witness).unwrap();
    assert_eq!(
        value(&r).uniqueness(),
        &HtmlMetaPragmaUniqueness::Duplicate(expected.clone())
    );
    assert_eq!(
        value(&r).charset_conflict(),
        &HtmlMetaEncodingCharsetConflict::Present(expected)
    );
}

#[test]
fn competitor_raw_failure_before_later_duplicate_is_terminal() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, &[("http-equiv", "refresh"), ("content", "0")]);
    let too_long = meta(&mut b, &d, &[("http-equiv", "01234567890123456789")]);
    let witness = meta(&mut b, &d, &[("http-equiv", "refresh")]);
    for n in [&a, &too_long, &witness] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), too_long, witness]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 7, HtmlSemanticCancellation::new());
    let fail = s.query(&[request(&c.subject_id(&a).unwrap())]).unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn every_refresh_work_cut_stops_atomically_without_retaining_scratch() {
    let (c, id) = single(&[("http-equiv", "refresh"), ("content", "12; URL='x'")]);
    let cx = context(&c);
    let work = report(&c, &id, 20).usage().work();
    for cap in 0..work {
        let mut s = session(
            &c,
            &cx,
            capped(HtmlSemanticResource::Work, cap),
            20,
            HtmlSemanticCancellation::new(),
        );
        let fail = s.query(&[request(&id)]).unwrap_err();
        assert!(matches!(
            fail.kind(),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                ..
            }
        ));
        assert_eq!(s.query(&[]).unwrap_err(), fail);
        assert_eq!(s.usage().scratch_records(), 0);
        assert!(s.usage().work() <= cap);
    }
}
