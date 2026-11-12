use crate::semantics::support::{
    Charge, ExpectedUsage, assert_exact_and_one_short_caps,
    small_builder_with_short_text as builder,
};
use surgeist_html::{
    elements::link::{
        HtmlLinkFlags, HtmlLinkHistory, HtmlRelKeyword, HtmlRelLinks, HtmlRelSpelling,
    },
    microsyntax::tokens::HtmlSpaceToken,
    values::{
        Attribute, DocumentString, HtmlValueInterpreter, Namespace, OrderedAttributes,
        QualifiedName,
    },
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError,
    HtmlCaptureRegistryInput, HtmlCapturedIsValue, HtmlRelAssociation, HtmlRelBlobEntry,
    HtmlRelBookmarkSection, HtmlRelContextReadback, HtmlRelDefinition, HtmlRelDefinitionId,
    HtmlRelDefinitionKey, HtmlRelExpectedRevisions, HtmlRelExtensionEffect, HtmlRelFacts,
    HtmlRelFactsError, HtmlRelFactsInput, HtmlRelFactsLimits, HtmlRelGrammarInput,
    HtmlRelMissingFact, HtmlRelNoopenerInput, HtmlRelProducerId, HtmlRelRegistryOccurrence,
    HtmlRelRegistryStatus, HtmlRelRevision, HtmlRelSameSiteObservation, HtmlRelSectionId,
    HtmlRelSupportedTokens, HtmlRelUrlGrammar, HtmlRelUrlKey, HtmlRelUrlObservation,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticResult,
    HtmlSemanticSession, HtmlSemanticValue,
};
use surgeist_html_semantics::{HtmlRelOriginId, HtmlRelSettingsId, HtmlRelUrlId};

fn attrs(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(
        values
            .iter()
            .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    tag: &str,
    values: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, tag),
        attrs(values),
        HtmlCapturedIsValue::Absent,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn revision(n: u64) -> HtmlRelRevision {
    HtmlRelRevision::new(HtmlRelProducerId::fresh(), 3, n)
}
fn limits() -> HtmlRelFactsLimits {
    HtmlRelFactsLimits::new(1000, 100000, 1000000)
}
fn context(
    c: &HtmlCapture,
    expected: HtmlRelExpectedRevisions,
    facts: Option<HtmlRelFacts>,
) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_link_relationships(expected, facts),
        0,
    )
    .unwrap()
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    raw: usize,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(raw),
        HtmlSemanticLimits::new(1000000, 100, 10000, 10000, 1000000, 100000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
}
fn request(c: &HtmlCapture, e: &HtmlCaptureBuilderSubject) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(
        c.subject_id(e).unwrap(),
        HtmlSemanticQuery::LinkRelationships,
    )
}
fn read(report: &HtmlSemanticReport, index: usize) -> &HtmlRelContextReadback {
    let HtmlSemanticFact::LinkRelationships(HtmlSemanticValue::Known(value)) =
        report.results()[index].fact()
    else {
        panic!("expected known rel result")
    };
    value
}

#[test]
fn qualified_form_effect_status_and_original_synonym_remain_independent() {
    let (mut b, d) = builder();
    let raw = "ext alias obsolete absent incomplete aonly";
    let e = element(&mut b, &d, "form", &[("rel", raw)]);
    let c = b.finish().unwrap();
    let subject = c.subject_id(&e).unwrap();
    let rev = revision(7);
    let key = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 1);
    let old = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 2);
    let aonly = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 3);
    let defs = vec![
        HtmlRelDefinition::extension(
            key.clone(),
            HtmlSpaceToken::new("ext").unwrap(),
            HtmlRelRegistryStatus::Proposed,
            HtmlRelExtensionEffect::NotAllowed,
            HtmlRelExtensionEffect::Hyperlink,
            HtmlRelExtensionEffect::Hyperlink,
        ),
        HtmlRelDefinition::extension(
            old.clone(),
            HtmlSpaceToken::new("obsolete").unwrap(),
            HtmlRelRegistryStatus::Discontinued,
            HtmlRelExtensionEffect::NotAllowed,
            HtmlRelExtensionEffect::Hyperlink,
            HtmlRelExtensionEffect::Hyperlink,
        ),
        HtmlRelDefinition::extension(
            aonly.clone(),
            HtmlSpaceToken::new("aonly").unwrap(),
            HtmlRelRegistryStatus::Ratified,
            HtmlRelExtensionEffect::NotAllowed,
            HtmlRelExtensionEffect::Hyperlink,
            HtmlRelExtensionEffect::NotAllowed,
        ),
    ];
    let occurrences = vec![
        HtmlRelRegistryOccurrence::definition(subject.clone(), raw.into(), 0..3, key.clone()),
        HtmlRelRegistryOccurrence::synonym(subject.clone(), raw.into(), 4..9, key.clone()),
        HtmlRelRegistryOccurrence::definition(subject.clone(), raw.into(), 10..18, old),
        HtmlRelRegistryOccurrence::proven_absent(subject.clone(), raw.into(), 19..25),
        HtmlRelRegistryOccurrence::unavailable(subject.clone(), raw.into(), 26..36),
        HtmlRelRegistryOccurrence::definition(subject.clone(), raw.into(), 37..42, aonly),
    ];
    let facts = HtmlRelFacts::new(
        &c,
        c.host_facts_revision(),
        HtmlRelFactsInput::new().with_registry(rev.clone(), defs, occurrences),
        limits(),
    )
    .unwrap();
    let cx = context(
        &c,
        HtmlRelExpectedRevisions::new().with_registry(rev),
        Some(facts),
    );
    let result = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
    let r = read(&result, 0);
    assert_eq!(r.links(), HtmlRelLinks::Declared);
    assert_eq!(
        r.extensions()[0].author_conformance(),
        &HtmlSemanticValue::Known(true)
    );
    assert_eq!(r.extensions()[1].definition(), Some(&key));
    assert_eq!(
        r.extensions()[1].canonical().unwrap().value(),
        &DocumentString::from("ext")
    );
    assert_eq!(
        r.local().occurrence_units(1),
        Some("alias".encode_utf16().collect::<Vec<_>>().as_slice())
    );
    assert_eq!(
        r.extensions()[1].author_conformance(),
        &HtmlSemanticValue::Known(false)
    );
    assert_eq!(
        r.extensions()[2].registry_status(),
        Some(HtmlRelRegistryStatus::Discontinued)
    );
    assert_eq!(
        r.extensions()[2].effect(),
        HtmlRelExtensionEffect::Hyperlink
    );
    for i in [2, 3, 5] {
        assert_eq!(
            r.extensions()[i].author_conformance(),
            &HtmlSemanticValue::Known(false)
        );
    }
    assert!(matches!(
        r.extensions()[4].author_conformance(),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
            fact: HtmlRelMissingFact::Registry,
            ..
        })
    ));
}

#[test]
fn synonyms_join_exact_canonical_revision_and_apply_standard_context() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "a", &[("rel", "writer"), ("href", "")]);
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let subject = c.subject_id(&e).unwrap();
    let rev = revision(1);
    let identity = HtmlRelDefinitionId::fresh();
    let key = HtmlRelDefinitionKey::new(identity.clone(), 5);
    let input = HtmlRelFactsInput::new().with_registry(
        rev.clone(),
        vec![HtmlRelDefinition::standard(
            key.clone(),
            HtmlRelKeyword::Author,
            HtmlRelRegistryStatus::Ratified,
        )],
        vec![HtmlRelRegistryOccurrence::synonym(
            subject.clone(),
            "writer".into(),
            0..6,
            key.clone(),
        )],
    );
    let facts = HtmlRelFacts::new(&c, c.host_facts_revision(), input, limits()).unwrap();
    let cx = context(
        &c,
        HtmlRelExpectedRevisions::new().with_registry(rev.clone()),
        Some(facts),
    );
    let result = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
    let r = read(&result, 0);
    assert_eq!(r.meanings()[0].keyword(), HtmlRelKeyword::Author);
    assert!(matches!(
        r.meanings()[0].association(),
        HtmlSemanticValue::Known(HtmlRelAssociation::Page(_))
    ));
    assert_eq!(
        r.local().occurrences()[0].spelling(),
        HtmlRelSpelling::Unknown
    );
    assert_eq!(
        r.local().occurrences()[0].keyword(),
        Some(HtmlRelKeyword::Author)
    );
    let stale = HtmlRelDefinitionKey::new(identity, 4);
    let input = HtmlRelFactsInput::new().with_registry(
        rev,
        vec![HtmlRelDefinition::standard(
            key,
            HtmlRelKeyword::Author,
            HtmlRelRegistryStatus::Ratified,
        )],
        vec![HtmlRelRegistryOccurrence::synonym(
            subject,
            "writer".into(),
            0..6,
            stale,
        )],
    );
    assert_eq!(
        HtmlRelFacts::new(&c, c.host_facts_revision(), input, limits()).unwrap_err(),
        HtmlRelFactsError::MissingDefinition
    );
}

#[test]
fn absolute_url_observation_is_required_only_for_actual_colon_token() {
    for grammar in [None, Some(false), Some(true)] {
        let (mut b, d) = builder();
        let e = element(&mut b, &d, "a", &[("rel", "urn:kind"), ("href", "x")]);
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        let rev = revision(1);
        let key = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 1);
        let mut expected = HtmlRelExpectedRevisions::new().with_registry(rev.clone());
        let mut input = HtmlRelFactsInput::new().with_registry(
            rev,
            vec![HtmlRelDefinition::extension(
                key.clone(),
                HtmlSpaceToken::new("urn:kind").unwrap(),
                HtmlRelRegistryStatus::Ratified,
                HtmlRelExtensionEffect::NotAllowed,
                HtmlRelExtensionEffect::Hyperlink,
                HtmlRelExtensionEffect::NotAllowed,
            )],
            vec![HtmlRelRegistryOccurrence::definition(
                id.clone(),
                "urn:kind".into(),
                0..8,
                key,
            )],
        );
        if let Some(valid) = grammar {
            let rev = revision(8);
            expected = expected.with_url_grammar(rev.clone());
            input = input.with_url_grammar(
                rev,
                vec![HtmlRelUrlGrammar::new(
                    id,
                    HtmlRelGrammarInput::ExtensionOccurrence { start: 0, end: 8 },
                    "urn:kind".into(),
                    valid,
                )],
            );
        }
        let facts = HtmlRelFacts::new(&c, c.host_facts_revision(), input, limits()).unwrap();
        let cx = context(&c, expected, Some(facts));
        let result = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
        let r = read(&result, 0);
        assert_eq!(r.extensions()[0].absolute_url_grammar(), grammar);
        match grammar {
            Some(v) => assert_eq!(
                r.extensions()[0].author_conformance(),
                &HtmlSemanticValue::Known(v)
            ),
            None => assert!(matches!(
                r.extensions()[0].author_conformance(),
                HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
                    fact: HtmlRelMissingFact::UrlGrammar,
                    ..
                })
            )),
        }
        assert_eq!(r.links(), HtmlRelLinks::Declared);
    }
}

#[test]
fn closed_supported_set_known_empty_and_missing_have_distinct_results() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "form", &[("rel", "help")]);
    let c = b.finish().unwrap();
    let rev = revision(1);
    let input = HtmlRelFactsInput::new().with_supported_tokens(
        rev.clone(),
        vec![HtmlRelSupportedTokens::new(
            c.subject_id(&e).unwrap(),
            vec![],
        )],
    );
    let facts = HtmlRelFacts::new(&c, c.host_facts_revision(), input, limits()).unwrap();
    let cx = context(
        &c,
        HtmlRelExpectedRevisions::new().with_supported_tokens(rev),
        Some(facts),
    );
    let result = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
    assert_eq!(
        read(&result, 0).supported_tokens(),
        &HtmlSemanticValue::Known(vec![])
    );
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let result = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
    // No supported-token fact was supplied for this link.
    assert_eq!(
        read(&result, 0).supported_tokens(),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
            subject: c.subject_id(&e).unwrap(),
            fact: HtmlRelMissingFact::SupportedTokens,
        })
    );
    let input = HtmlRelFactsInput::new().with_supported_tokens(
        revision(2),
        vec![HtmlRelSupportedTokens::new(
            c.subject_id(&e).unwrap(),
            vec![HtmlRelKeyword::Help],
        )],
    );
    assert_eq!(
        HtmlRelFacts::new(&c, c.host_facts_revision(), input, limits()).unwrap_err(),
        HtmlRelFactsError::InvalidSupportedToken
    );
}

#[test]
fn raw_admission_precedes_role_and_namespace_shortcuts_at_front_door() {
    let (mut b, d) = builder();
    let link = element(&mut b, &d, "link", &[("title", "x")]);
    let div = element(&mut b, &d, "div", &[("rel", "x")]);
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::svg(), None, "link"),
            attrs(&[("href", "x")]),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    for e in [link, div, foreign] {
        let mut s = session(&c, &cx, 0);
        let q = request(&c, &e);
        let failure = s.query(std::slice::from_ref(&q)).unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::Attribute(_)
        ));
        assert_eq!(failure.usage().scratch_records(), 0);
        assert_eq!(s.query(&[q]).unwrap_err(), failure);
    }
}

#[test]
fn link_history_is_conditional_and_imported_flags_are_qualified_by_capture() {
    let (mut b, d) = builder();
    let missing = element(&mut b, &d, "link", &[("rel", "stylesheet")]);
    let plain = element(&mut b, &d, "link", &[("rel", "icon")]);
    let parser = element(&mut b, &d, "link", &[("rel", "stylesheet")]);
    let other = element(&mut b, &d, "link", &[("rel", "stylesheet")]);
    let mut flags = HtmlLinkFlags::fresh();
    flags.disabled_attribute_changed(
        &Namespace::none(),
        Some(DocumentString::from("").code_units()),
        None,
    );
    let history = HtmlLinkHistory::new(flags, true);
    b.establish_link_history(&parser, history).unwrap();
    b.establish_link_history(&other, HtmlLinkHistory::new(HtmlLinkFlags::fresh(), false))
        .unwrap();
    assert_eq!(
        b.establish_link_history(&parser, history).unwrap_err(),
        HtmlCaptureError::LinkHistoryAlreadyEstablished
    );
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let requests = [missing, plain, parser, other].map(|e| request(&c, &e));
    let report = session(&c, &cx, 100).query(&requests).unwrap();
    assert!(matches!(
        read(&report, 0).implicitly_potentially_render_blocking(),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
            fact: HtmlRelMissingFact::LinkHistory,
            ..
        })
    ));
    assert_eq!(
        read(&report, 1).implicitly_potentially_render_blocking(),
        &HtmlSemanticValue::Known(false)
    );
    assert_eq!(
        read(&report, 2).link_history(),
        &HtmlSemanticValue::Known(history)
    );
    assert_eq!(
        read(&report, 2).implicitly_potentially_render_blocking(),
        &HtmlSemanticValue::Known(true)
    );
    assert_eq!(
        read(&report, 3).implicitly_potentially_render_blocking(),
        &HtmlSemanticValue::Known(false)
    );
}

#[test]
fn noopener_ordered_dominance_and_actual_url_settings_facts_cover_all_roles() {
    // Last tuple member: None is missing URL coverage; null is Some(None);
    // non-null is Some(Some(same-site)). No URL strings are compared.
    let cases = [
        ("noopener opener", "named", None, Some(true)),
        ("noreferrer", "named", None, Some(true)),
        ("", "_BlAnK", None, Some(true)),
        ("opener", "_blank", Some(None), Some(false)),
        ("", "named", Some(None), Some(false)),
        ("opener", "named", Some(Some(true)), Some(false)),
        ("opener", "named", Some(Some(false)), Some(true)),
        ("", "named", None, None),
        ("alien", "_blank", Some(None), None),
        ("alien", "named", Some(Some(false)), Some(true)),
    ];
    for role in ["a", "area", "form"] {
        for (rel, target, blob, expected) in cases {
            let (mut b, d) = builder();
            let e = element(&mut b, &d, role, &[("rel", rel)]);
            let c = b.finish().unwrap();
            let id = c.subject_id(&e).unwrap();
            let url_rev = revision(1);
            let key = HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), url_rev.clone());
            let request = HtmlSemanticRequest::noopener(
                HtmlRelNoopenerInput::new(
                    &c,
                    c.host_facts_revision(),
                    id.clone(),
                    &url_rev,
                    key.clone(),
                    target.into(),
                    limits(),
                )
                .unwrap(),
            );
            let mut urls = vec![];
            let mut sites = vec![];
            if let Some(blob) = blob {
                let blob = match blob {
                    None => HtmlRelBlobEntry::Null,
                    Some(same_site) => {
                        let origin = HtmlRelOriginId::fresh();
                        sites.push(HtmlRelSameSiteObservation::new(
                            id.clone(),
                            key.clone(),
                            HtmlRelSettingsId::fresh(),
                            origin.clone(),
                            HtmlRelOriginId::fresh(),
                            same_site,
                        ));
                        HtmlRelBlobEntry::NonNull {
                            environment_origin: origin,
                        }
                    }
                };
                urls.push(HtmlRelUrlObservation::new(id, key, blob));
            }
            let settings = revision(2);
            let input = HtmlRelFactsInput::new()
                .with_url_records(url_rev.clone(), urls)
                .with_settings(settings.clone(), sites);
            let facts = HtmlRelFacts::new(&c, c.host_facts_revision(), input, limits()).unwrap();
            let cx = context(
                &c,
                HtmlRelExpectedRevisions::new()
                    .with_url_records(url_rev)
                    .with_settings(settings),
                Some(facts),
            );
            let result = session(&c, &cx, 100).query(&[request]).unwrap();
            match (result.results()[0].fact(), expected) {
                (HtmlSemanticFact::Noopener(HtmlSemanticValue::Known(actual)), Some(expected)) => {
                    assert_eq!(*actual, expected, "{role} {rel} {target}")
                }
                // Table rows with no expected value only require that the
                // outcome is missing evidence; the rows share no single payload.
                (HtmlSemanticFact::Noopener(HtmlSemanticValue::Missing(_)), None) => {}
                other => panic!("{role} {rel} {target}: {other:?}"),
            };
            assert_eq!(result.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn noopener_target_admission_and_url_identity_revision_cannot_be_bypassed() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "a", &[("rel", "noopener")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let revision = revision(1);
    let key = HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), revision.clone());
    let input = HtmlRelNoopenerInput::new(
        &c,
        c.host_facts_revision(),
        id.clone(),
        &revision,
        key,
        "target-is-long".into(),
        limits(),
    )
    .unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let failure = session(&c, &cx, 8)
        .query(&[HtmlSemanticRequest::noopener(input)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(failure.usage().scratch_records(), 0);
    let different = HtmlRelRevision::new(
        revision.producer().clone(),
        revision.configuration(),
        revision.revision() + 1,
    );
    let key = HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), different);
    assert_eq!(
        HtmlRelNoopenerInput::new(
            &c,
            c.host_facts_revision(),
            id,
            &revision,
            key,
            "".into(),
            limits()
        )
        .unwrap_err(),
        HtmlRelFactsError::UrlRevisionMismatch
    );
}

#[test]
fn noopener_registered_synonym_uses_its_exact_canonical_definition() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "form", &[("rel", "isolate")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let registry = revision(1);
    let key = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 9);
    let input = HtmlRelFactsInput::new().with_registry(
        registry.clone(),
        vec![HtmlRelDefinition::standard(
            key.clone(),
            HtmlRelKeyword::NoOpener,
            HtmlRelRegistryStatus::Discontinued,
        )],
        vec![HtmlRelRegistryOccurrence::synonym(
            id.clone(),
            "isolate".into(),
            0..7,
            key,
        )],
    );
    let facts = HtmlRelFacts::new(&c, c.host_facts_revision(), input, limits()).unwrap();
    let cx = context(
        &c,
        HtmlRelExpectedRevisions::new().with_registry(registry),
        Some(facts),
    );
    let url = revision(3);
    let input = HtmlRelNoopenerInput::new(
        &c,
        c.host_facts_revision(),
        id,
        &url,
        HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), url.clone()),
        "named".into(),
        limits(),
    )
    .unwrap();
    let result = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::noopener(input)])
        .unwrap();
    assert_eq!(
        result.results()[0].fact(),
        &HtmlSemanticFact::Noopener(HtmlSemanticValue::Known(true))
    );
    assert_eq!(result.usage().scratch_records(), 0);
}

#[test]
fn noopener_exact_work_scratch_and_bytes_limits_retire_temporary_capacity_on_failure() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "a", &[("rel", "noopener opener")]);
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let rev = revision(1);
    let q = HtmlSemanticRequest::noopener(
        HtmlRelNoopenerInput::new(
            &c,
            c.host_facts_revision(),
            c.subject_id(&e).unwrap(),
            &rev,
            HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), rev.clone()),
            "named".into(),
            limits(),
        )
        .unwrap(),
    );
    let success = session(&c, &cx, 100)
        .query(std::slice::from_ref(&q))
        .unwrap();
    let usage = success.usage();
    assert_eq!(usage.scratch_records(), 0);
    // The independent expectation for the usage capped below: one Boolean
    // result record, a relationship-token scan whose private layout makes
    // work, scratch and token-buffer bytes required-to-be-charged rather than
    // pinned, and no nested storage; the exact cap on every resource must
    // reproduce the report and one short must fail with exactly that resource.
    assert_exact_and_one_short_caps(
        "noopener",
        &c,
        &cx,
        100,
        HtmlSemanticLimits::new(20000000, 1, 1000, 0, 1000000, 0),
        std::slice::from_ref(&q),
        ExpectedUsage {
            output_records: Charge::Exactly(1),
            peak_scratch_records: Charge::Charged,
            allocated_bytes: Charge::AtLeast(std::mem::size_of::<HtmlSemanticResult>()),
            ..ExpectedUsage::none()
        },
    );
    let exact = HtmlSemanticLimits::new(
        usage.work(),
        1,
        usage.peak_scratch_records(),
        0,
        usage.allocated_bytes(),
        0,
    );
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        exact,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert_eq!(
        s.query(std::slice::from_ref(&q)).unwrap().results()[0].fact(),
        &HtmlSemanticFact::Noopener(HtmlSemanticValue::Known(true))
    );
    for (limit, resource) in [
        (
            HtmlSemanticLimits::new(
                usage.work() - 1,
                1,
                usage.peak_scratch_records(),
                0,
                usage.allocated_bytes(),
                0,
            ),
            HtmlSemanticResource::Work,
        ),
        (
            HtmlSemanticLimits::new(
                usage.work(),
                1,
                usage.peak_scratch_records() - 1,
                0,
                usage.allocated_bytes(),
                0,
            ),
            HtmlSemanticResource::ScratchRecords,
        ),
        (
            HtmlSemanticLimits::new(
                usage.work(),
                1,
                usage.peak_scratch_records(),
                0,
                usage.allocated_bytes() - 1,
                0,
            ),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(100),
            limit,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let failure = s.query(std::slice::from_ref(&q)).unwrap_err();
        assert!(
            matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if *r==resource)
        );
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(failure.usage().scratch_records(), 0);
        assert_eq!(s.query(std::slice::from_ref(&q)).unwrap_err(), failure);
    }
    let cancellation = HtmlSemanticCancellation::new();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        exact,
        cancellation.clone(),
    )
    .unwrap();
    cancellation.cancel();
    let failure = s.query(&[q]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(failure.usage().scratch_records(), 0);
}

#[test]
fn changing_url_record_identity_does_not_reuse_a_null_blob_observation() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "a", &[("rel", "opener")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let rev = revision(1);
    let observed = HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), rev.clone());
    let requested = HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), rev.clone());
    let facts = HtmlRelFacts::new(
        &c,
        c.host_facts_revision(),
        HtmlRelFactsInput::new().with_url_records(
            rev.clone(),
            vec![HtmlRelUrlObservation::new(
                id.clone(),
                observed,
                HtmlRelBlobEntry::Null,
            )],
        ),
        limits(),
    )
    .unwrap();
    let cx = context(
        &c,
        HtmlRelExpectedRevisions::new().with_url_records(rev.clone()),
        Some(facts),
    );
    let q = HtmlSemanticRequest::noopener(
        HtmlRelNoopenerInput::new(
            &c,
            c.host_facts_revision(),
            id,
            &rev,
            requested,
            "named".into(),
            limits(),
        )
        .unwrap(),
    );
    let report = session(&c, &cx, 100).query(&[q]).unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::Noopener(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::LinkRelationship {
                fact: HtmlRelMissingFact::UrlRecord,
                ..
            }
        ))
    ));
}

#[test]
fn captured_link_href_author_uses_exact_grammar_and_image_context_uses_as_owner() {
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        "link",
        &[
            ("rel", "preload"),
            ("as", "IMAGE"),
            ("imagesrcset", "candidate 1x"),
            ("href", "url"),
        ],
    );
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let r = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
    assert!(read(&r, 0).local().image_source_preload_context());
    assert!(matches!(
        read(&r, 0).href_url_author(),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
            fact: HtmlRelMissingFact::UrlGrammar,
            ..
        })
    ));
    for valid in [false, true] {
        let rev = revision(1);
        let facts = HtmlRelFacts::new(
            &c,
            c.host_facts_revision(),
            HtmlRelFactsInput::new().with_url_grammar(
                rev.clone(),
                vec![HtmlRelUrlGrammar::new(
                    c.subject_id(&e).unwrap(),
                    HtmlRelGrammarInput::Href,
                    "url".into(),
                    valid,
                )],
            ),
            limits(),
        )
        .unwrap();
        let cx = context(
            &c,
            HtmlRelExpectedRevisions::new().with_url_grammar(rev),
            Some(facts),
        );
        let r = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
        assert_eq!(
            read(&r, 0).href_url_author(),
            &HtmlSemanticValue::Known(valid)
        );
    }
    for href in ["", " \t\r\n\u{c}"] {
        let (mut b, d) = builder();
        let e = element(&mut b, &d, "link", &[("rel", "icon"), ("href", href)]);
        b.establish_root(&e).unwrap();
        let c = b.finish().unwrap();
        let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
        let r = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
        assert_eq!(
            read(&r, 0).href_url_author(),
            &HtmlSemanticValue::Known(false)
        );
        let facts = HtmlRelFactsInput::new().with_url_grammar(
            revision(1),
            vec![HtmlRelUrlGrammar::new(
                c.subject_id(&e).unwrap(),
                HtmlRelGrammarInput::Href,
                href.into(),
                true,
            )],
        );
        assert_eq!(
            HtmlRelFacts::new(&c, c.host_facts_revision(), facts, limits()).unwrap_err(),
            HtmlRelFactsError::GrammarContradiction
        );
    }
}

#[test]
fn author_bookmark_help_and_translation_follow_actual_tree_context() {
    let (mut b, d) = builder();
    let root = element(&mut b, &d, "html", &[("lang", "EN")]);
    let article = element(&mut b, &d, "article", &[]);
    let a = element(
        &mut b,
        &d,
        "a",
        &[("rel", "author bookmark help"), ("href", "x")],
    );
    let area = element(
        &mut b,
        &d,
        "area",
        &[("rel", "author bookmark"), ("href", "x")],
    );
    let form = element(&mut b, &d, "form", &[("rel", "help")]);
    let alt = element(
        &mut b,
        &d,
        "link",
        &[("rel", "alternate"), ("href", "x"), ("hreflang", "en")],
    );
    b.establish_children(&d, std::slice::from_ref(&root))
        .unwrap();
    b.establish_children(
        &root,
        &[article.clone(), area.clone(), form.clone(), alt.clone()],
    )
    .unwrap();
    b.establish_children(&article, std::slice::from_ref(&a))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let r = session(&c, &cx, 100)
        .query(&[
            request(&c, &a),
            request(&c, &area),
            request(&c, &form),
            request(&c, &alt),
        ])
        .unwrap();
    let meanings = read(&r, 0).meanings();
    for m in meanings.iter().filter(|m| {
        matches!(
            m.keyword(),
            HtmlRelKeyword::Author | HtmlRelKeyword::Bookmark
        )
    }) {
        assert_eq!(
            m.association(),
            &HtmlSemanticValue::Known(HtmlRelAssociation::Article(c.subject_id(&article).unwrap()))
        );
    }
    let help = meanings
        .iter()
        .find(|m| m.keyword() == HtmlRelKeyword::Help)
        .unwrap();
    assert_eq!(
        help.association(),
        &HtmlSemanticValue::Known(HtmlRelAssociation::ParentAndChildren(Some(
            c.subject_id(&article).unwrap()
        )))
    );
    assert!(matches!(
        read(&r, 1).meanings()[0].association(),
        HtmlSemanticValue::Known(HtmlRelAssociation::Page(_))
    ));
    assert!(matches!(
        read(&r, 1).meanings()[1].association(),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
            fact: HtmlRelMissingFact::BookmarkSection,
            ..
        })
    ));
    assert_eq!(
        read(&r, 2).meanings()[0].association(),
        &HtmlSemanticValue::Known(HtmlRelAssociation::ParentAndChildren(Some(
            c.subject_id(&root).unwrap()
        )))
    );
    // The defining clause says different strings; no invented language-tag case folding.
    assert_eq!(
        read(&r, 3).alternate_translation(),
        &HtmlSemanticValue::Known(true)
    );
    let section = HtmlRelSectionId::fresh();
    let rev = revision(2);
    let facts = HtmlRelFacts::new(
        &c,
        c.host_facts_revision(),
        HtmlRelFactsInput::new().with_bookmark_sections(
            rev.clone(),
            vec![HtmlRelBookmarkSection::new(
                c.subject_id(&area).unwrap(),
                section.clone(),
            )],
        ),
        limits(),
    )
    .unwrap();
    let cx = context(
        &c,
        HtmlRelExpectedRevisions::new().with_bookmark_sections(rev),
        Some(facts),
    );
    let r = session(&c, &cx, 100).query(&[request(&c, &area)]).unwrap();
    assert_eq!(
        read(&r, 0).meanings()[1].association(),
        &HtmlSemanticValue::Known(HtmlRelAssociation::Section(section))
    );
    let (mut b, d) = builder();
    let a = element(
        &mut b,
        &d,
        "a",
        &[("href", "x"), ("rel", "author bookmark help")],
    );
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let r = session(&c, &cx, 100).query(&[request(&c, &a)]).unwrap();
    // Each meaning's association depends on the link's tree context, and the
    // anchor's parent was never established, so that parent is the missing
    // evidence of every meaning.
    let missing =
        HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(c.subject_id(&a).unwrap()));
    assert!(
        read(&r, 0)
            .meanings()
            .iter()
            .all(|m| m.association() == &missing)
    );
}

#[test]
fn image_preload_context_preserves_unknown_extension_and_qualified_alias() {
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        "link",
        &[
            ("rel", "prime-image"),
            ("as", "image"),
            ("imagesrcset", "x 1x"),
        ],
    );
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let r = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
    assert!(matches!(
        read(&r, 0).image_source_preload_author(),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
            fact: HtmlRelMissingFact::Registry,
            ..
        })
    ));
    let rev = revision(1);
    let key = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 3);
    let facts = HtmlRelFacts::new(
        &c,
        c.host_facts_revision(),
        HtmlRelFactsInput::new().with_registry(
            rev.clone(),
            vec![HtmlRelDefinition::standard(
                key.clone(),
                HtmlRelKeyword::Preload,
                HtmlRelRegistryStatus::Ratified,
            )],
            vec![HtmlRelRegistryOccurrence::synonym(
                c.subject_id(&e).unwrap(),
                "prime-image".into(),
                0..11,
                key,
            )],
        ),
        limits(),
    )
    .unwrap();
    let cx = context(
        &c,
        HtmlRelExpectedRevisions::new().with_registry(rev),
        Some(facts),
    );
    let r = session(&c, &cx, 100).query(&[request(&c, &e)]).unwrap();
    assert!(read(&r, 0).local().image_source_preload_context());
    assert_eq!(
        read(&r, 0).image_source_preload_author(),
        &HtmlSemanticValue::Known(true)
    );
    assert_eq!(
        read(&r, 0).local().occurrence_units(0),
        Some("prime-image".encode_utf16().collect::<Vec<_>>().as_slice())
    );
}

/// Checks the contract of one limit over a growing list. The allocation path of a list does not
/// depend on any limit, so the query's peak usage `U` of the limited resource is fixed by its
/// input and the query succeeds under a limit exactly when the limit is at least `U`.
///
/// `limits_at` sets the limit under test and leaves every other resource unbinding. `U` is the
/// least limit under which the query succeeds, found by bisection, and must equal what `used`
/// reads from the unlimited run: the record counters and the allocated bytes are only ever
/// raised, so their final value is their peak. Then, for every limit in `candidates` and for the
/// window `U - 64..=U + 64`: usage never exceeds the limit, a refusal is a limit refusal, and the
/// query succeeds if and only if the limit is at least `U`.
fn assert_limit_threshold(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    request: &HtmlSemanticRequest,
    candidates: impl Iterator<Item = usize>,
    limits_at: impl Fn(usize) -> HtmlSemanticLimits,
    used: impl Fn(&HtmlSemanticUsageView) -> usize,
) {
    let run = |limit: usize| {
        let mut s = HtmlSemanticSession::new(
            c,
            cx,
            HtmlValueInterpreter::new(1_000_000),
            limits_at(limit),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let result = s.query(std::slice::from_ref(request));
        (result, s.usage())
    };
    let ample = 1_000_000_000;
    let (unlimited, usage) = run(ample);
    unlimited.unwrap();
    let (mut refused, mut accepted) = (0, ample);
    while refused + 1 < accepted {
        let middle = refused + (accepted - refused) / 2;
        if run(middle).0.is_ok() {
            accepted = middle;
        } else {
            refused = middle;
        }
    }
    let threshold = accepted;
    assert!(threshold > 0, "the query must retain something");
    assert_eq!(
        threshold,
        used(&usage),
        "the threshold is the unlimited run's usage"
    );
    for limit in candidates.chain(threshold.saturating_sub(64)..=threshold + 64) {
        let (result, usage) = run(limit);
        assert!(
            used(&usage) <= limit,
            "limit {limit}: usage {} exceeds it",
            used(&usage)
        );
        match result {
            Ok(_) => assert!(
                limit >= threshold,
                "limit {limit} succeeded below {threshold}"
            ),
            Err(error) => {
                assert!(
                    matches!(error.kind(), HtmlSemanticFailureKind::LimitExceeded { .. }),
                    "limit {limit}: {error:?}"
                );
                assert!(
                    limit < threshold,
                    "limit {limit} refused at or above {threshold}"
                );
            }
        }
    }
}
type HtmlSemanticUsageView = surgeist_html_semantics::HtmlSemanticUsage;

fn forty_tokens() -> String {
    (0..40)
        .map(|i| format!("t{i}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A relationship list of 40 distinct occurrences grows geometrically in nested output records;
/// the capacity is admitted under the nested-record limit and under the allocated-byte limit, and
/// never exceeds either.
#[test]
fn geometric_occurrence_growth_succeeds_exactly_at_its_peak_usage_under_the_nested_record_and_byte_limits()
 {
    let (mut b, d) = builder();
    let raw = forty_tokens();
    let e = element(&mut b, &d, "a", &[("rel", &raw)]);
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let q = request(&c, &e);
    assert_limit_threshold(
        &c,
        &cx,
        &q,
        0..=400,
        |nested| HtmlSemanticLimits::new(100_000_000, 1000, 1000, 0, 100_000_000, nested),
        |usage| usage.nested_output_records(),
    );
    assert_limit_threshold(
        &c,
        &cx,
        &q,
        (0..=40_000).step_by(37),
        |bytes| HtmlSemanticLimits::new(100_000_000, 1000, 1000, 0, bytes, 100_000),
        |usage| usage.allocated_bytes(),
    );
}

/// The same for the transient occurrence list of a noopener evaluation, which counts as scratch
/// records and is retired when the evaluation finishes: the peak never exceeds the scratch limit
/// or the byte limit.
#[test]
fn geometric_scratch_occurrence_growth_succeeds_exactly_at_its_peak_usage_under_the_scratch_record_and_byte_limits()
 {
    let (mut b, d) = builder();
    let raw = format!("noopener {}", forty_tokens());
    let e = element(&mut b, &d, "a", &[("rel", &raw)]);
    let c = b.finish().unwrap();
    let cx = context(&c, HtmlRelExpectedRevisions::new(), None);
    let rev = revision(1);
    let q = HtmlSemanticRequest::noopener(
        HtmlRelNoopenerInput::new(
            &c,
            c.host_facts_revision(),
            c.subject_id(&e).unwrap(),
            &rev,
            HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), rev.clone()),
            "named".into(),
            limits(),
        )
        .unwrap(),
    );
    assert_limit_threshold(
        &c,
        &cx,
        &q,
        0..=400,
        |scratch| HtmlSemanticLimits::new(100_000_000, 1000, scratch, 0, 100_000_000, 100_000),
        |usage| usage.peak_scratch_records(),
    );
    assert_limit_threshold(
        &c,
        &cx,
        &q,
        (0..=40_000).step_by(37),
        |bytes| HtmlSemanticLimits::new(100_000_000, 1000, 100_000, 0, bytes, 100_000),
        |usage| usage.allocated_bytes(),
    );
}
