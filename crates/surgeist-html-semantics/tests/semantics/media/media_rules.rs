use crate::semantics::support::{builder_with, capture_limits, semantic_limits as limits};
use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCaptureTemplateCacheInput,
    HtmlCapturedIsValue, HtmlCapturedShadowProperties, HtmlDocumentClass, HtmlMediaAttributeInput,
    HtmlMediaAttributeReadback, HtmlMediaCondition, HtmlMediaContextRevision,
    HtmlMediaExternalCapability, HtmlMediaFact, HtmlMediaFacts, HtmlMediaFactsError,
    HtmlMediaFactsLimits, HtmlMediaGrammarAttestation, HtmlMediaGrammarEvidence,
    HtmlMediaGrammarOutcome, HtmlMediaMatchAttestation, HtmlMediaMatchOutcome, HtmlMediaProcessing,
    HtmlMediaProducerId, HtmlMetaMediaProcessing, HtmlPictureSourceRequirement,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextError,
    HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticSession, HtmlSemanticUnsupported, HtmlSemanticValue, HtmlSourceMediaContext,
    HtmlSourceMediaDecision, HtmlTemplateDocumentRole, HtmlThemeColorMediaUniqueness,
};
use surgeist_html_semantics::{HtmlMediaEnvironmentRevision, HtmlMediaGrammarRevision};

// Independent HTML877d #mq, #processing-the-media-attribute, #attr-style-media,
// #attr-source-media, #the-source-element and #meta-theme-color expectations.
fn attr(name: &str, value: &str) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
}
fn attrs(fields: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(fields.iter().map(|(n, v)| attr(n, v))).unwrap()
}
fn builder(class: HtmlDocumentClass) -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let (mut b, d) = builder_with(capture_limits());
    if class != HtmlDocumentClass::Unavailable {
        b.establish_document_template_facts(
            &d,
            class,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    }
    (b, d)
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
    fields: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name,
        fields,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn children(
    b: &mut HtmlCaptureBuilder,
    parent: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(parent, cs).unwrap();
}
fn single(kind: KnownHtmlElement, fields: &[(&str, &str)]) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder(HtmlDocumentClass::Html);
    let a = element(&mut b, &d, kind.qualified_name(), attrs(fields));
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    (c, id)
}
fn cx(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap()
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::MediaAttributeRules)
}
fn run(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    HtmlSemanticSession::new(
        c,
        &cx(c),
        HtmlValueInterpreter::new(100000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request(id)])
    .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlMediaAttributeReadback {
    match r.results()[0].fact() {
        HtmlSemanticFact::MediaAttributeRules(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}
fn condition(r: &HtmlMediaAttributeReadback) -> &HtmlMediaCondition {
    match r.processing() {
        HtmlMediaProcessing::Style(g) => g.condition(),
        HtmlMediaProcessing::Meta(HtmlMetaMediaProcessing::ThemeColor { condition, .. }) => {
            condition
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn cursor_literal_classes_keyword_trim_and_announced_one_unit_steps() {
    use HtmlMediaValueClass::{Absent as A, AsciiWhitespaceOnly as W, Empty as E, Nontrivial as N};
    for (raw, class, restrictive) in [
        (None, A, false),
        (Some(vec![]), E, false),
        (Some(vec![9, 10, 12, 13, 32]), W, false),
        (Some(vec![0xA0]), N, true),
        (Some(vec![0x85]), N, true),
        (Some(vec![0x2028]), N, true),
        (Some(vec![0]), N, true),
        (Some(vec![0xD800]), N, true),
        (Some(vec![97, 108, 108]), N, false),
        (Some(vec![9, 65, 76, 76, 32]), N, false),
        (Some(vec![97, 32, 108, 108]), N, true),
        (Some(vec![0xA0, 97, 108, 108]), N, true),
    ] {
        let mut cursor = HtmlMediaValueCursor::new(raw.as_deref());
        for index in 0..raw.as_ref().map_or(0, Vec::len) {
            assert_eq!(
                cursor.next_action(),
                HtmlMediaCursorAction::InspectUnit(index)
            );
            assert_eq!(cursor.result(), None);
            cursor.advance();
        }
        assert_eq!(cursor.next_action(), HtmlMediaCursorAction::Finish);
        cursor.advance();
        let result = cursor.result().unwrap();
        assert_eq!(result.class(), class);
        assert_eq!(result.restrictive_picture_value(), restrictive);
        assert_eq!(result.matches_unconditionally(), matches!(class, E | W));
        cursor.advance();
        assert_eq!(cursor.result(), Some(result));
    }
}

#[test]
fn ordinary_original_admission_expanded_roles_and_exact_meta_name() {
    let i = HtmlValueInterpreter::new(100);
    for text in ["", "\t\n\x0c\r ", "screen", "\u{A0}"] {
        let a = attr("media", text);
        let r = i
            .media(&KnownHtmlElement::Style.qualified_name(), Some(&a))
            .unwrap();
        assert_eq!(r.raw(), Some(&a));
        assert_eq!(r.role(), Some(HtmlMediaElementRole::Style));
        assert_eq!(r.applicability(), HtmlAttributeApplicability::Applicable);
    }
    let media = attr("media", "?");
    for (name, placement) in [
        ("THEME-color", HtmlMetaMediaPlacement::AllowedForThemeColor),
        (" theme-color", HtmlMetaMediaPlacement::Forbidden),
        ("theme-color ", HtmlMetaMediaPlacement::Forbidden),
        ("", HtmlMetaMediaPlacement::Forbidden),
    ] {
        let n = attr("name", name);
        let r = i
            .media_for_meta(
                &KnownHtmlElement::Meta.qualified_name(),
                Some(&media),
                Some(&n),
            )
            .unwrap();
        assert_eq!(r.name_attribute(), Some(&n));
        assert_eq!(
            r.meta_placement(),
            HtmlMediaPlacementObservation::Known(placement)
        );
    }
    assert_eq!(
        i.media(&KnownHtmlElement::Meta.qualified_name(), Some(&media))
            .unwrap()
            .meta_placement(),
        HtmlMediaPlacementObservation::Unchecked
    );
    assert_eq!(
        i.media_for_meta(
            &KnownHtmlElement::Meta.qualified_name(),
            Some(&media),
            None::<&surgeist_html::Attribute>
        )
        .unwrap()
        .meta_placement(),
        HtmlMediaPlacementObservation::Known(HtmlMetaMediaPlacement::Forbidden)
    );
    let wrong = attr("id", "x");
    assert_eq!(
        i.media(&KnownHtmlElement::Style.qualified_name(), Some(&wrong))
            .unwrap()
            .applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
    let ns = Attribute::new(QualifiedName::new(Namespace::html(), None, "media"), "x");
    assert_eq!(
        i.media(&KnownHtmlElement::Style.qualified_name(), Some(&ns))
            .unwrap()
            .applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
    assert!(
        HtmlValueInterpreter::new(0)
            .media(&KnownHtmlElement::Div.qualified_name(), Some(&media))
            .is_err()
    );
    let n = attr("name", "theme-color");
    assert!(
        HtmlValueInterpreter::new(1)
            .media_for_meta(
                &KnownHtmlElement::Div.qualified_name(),
                None::<&surgeist_html::Attribute>,
                Some(&n)
            )
            .is_err()
    );
}

#[test]
fn role_defaults_empty_match_and_forbidden_placement_remain_independent_of_css() {
    for kind in [KnownHtmlElement::Style, KnownHtmlElement::Link] {
        let (c, id) = single(kind, &[]);
        let r = run(&c, &id);
        assert_eq!(value(&r).grammar(), &HtmlMediaGrammarEvidence::NotSpecified);
        match value(&r).processing() {
            HtmlMediaProcessing::Style(g) => {
                assert_eq!(g.condition(), &HtmlMediaCondition::DefaultAll)
            }
            HtmlMediaProcessing::Link(g) => {
                assert_eq!(
                    g.hyperlink_advisory().condition(),
                    &HtmlMediaCondition::DefaultAll
                );
                assert_eq!(
                    g.external_resource_gate().condition(),
                    &HtmlMediaCondition::DefaultAll
                );
            }
            other => panic!("{other:?}"),
        }
    }
    for raw in ["", "\t", "\n", "\x0c", "\r", " ", "\t \r\n\x0c"] {
        let (c, id) = single(KnownHtmlElement::Style, &[("media", raw)]);
        let r = run(&c, &id);
        assert_eq!(condition(value(&r)), &HtmlMediaCondition::Unconditional);
        assert_eq!(
            value(&r).grammar(),
            &HtmlMediaGrammarEvidence::Missing(HtmlSemanticMissing::MediaGrammar(id))
        );
    }
    for raw in ["all", "ALL", "\u{A0}", "\u{85}", "\u{2028}", "\0"] {
        let (c, id) = single(KnownHtmlElement::Style, &[("media", raw)]);
        let r = run(&c, &id);
        assert_eq!(
            condition(value(&r)),
            &HtmlMediaCondition::Missing(HtmlSemanticMissing::MediaEnvironment(id))
        );
    }
    let (c, id) = single(KnownHtmlElement::Meta, &[("name", "other"), ("media", "?")]);
    let r = run(&c, &id);
    assert_eq!(
        value(&r).local().meta_placement(),
        HtmlMediaPlacementObservation::Known(HtmlMetaMediaPlacement::Forbidden)
    );
    assert_eq!(
        value(&r).processing(),
        &HtmlMediaProcessing::Meta(HtmlMetaMediaProcessing::NoProcessingEffect)
    );
    assert!(matches!(
        value(&r).grammar(),
        HtmlMediaGrammarEvidence::Missing(_)
    ));
}

fn revision() -> HtmlMediaContextRevision {
    let p = HtmlMediaProducerId::fresh();
    HtmlMediaContextRevision::new(
        HtmlMediaGrammarRevision::new(p.clone(), 1, 2),
        HtmlMediaEnvironmentRevision::new(p, 3, 4),
    )
}
fn fact(
    c: &HtmlCapture,
    id: &HtmlCaptureSubjectId,
    rev: &HtmlMediaContextRevision,
    grammar: Option<HtmlMediaGrammarOutcome>,
    matching: Option<HtmlMediaMatchOutcome>,
) -> HtmlMediaFact {
    HtmlMediaFact::new(
        HtmlMediaAttributeInput::new(c, id.clone(), 0).unwrap(),
        grammar.map(|o| HtmlMediaGrammarAttestation::new(rev.grammar().clone(), o)),
        matching.map(|o| HtmlMediaMatchAttestation::new(rev.clone(), o)),
    )
}
fn facts(
    c: &HtmlCapture,
    rev: HtmlMediaContextRevision,
    fs: Vec<HtmlMediaFact>,
) -> Result<HtmlMediaFacts, HtmlMediaFactsError> {
    HtmlMediaFacts::new(
        c,
        c.host_facts_revision().clone(),
        rev,
        fs,
        HtmlMediaFactsLimits::new(100, 100000, 100000),
    )
}
fn with_facts(
    c: &HtmlCapture,
    rev: HtmlMediaContextRevision,
    fs: HtmlMediaFacts,
) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_media(rev, Some(fs)),
        0,
    )
    .unwrap()
}

#[test]
fn external_grammar_matching_and_missing_are_independent_and_qualified() {
    for (g, m, expected) in [
        (
            HtmlMediaGrammarOutcome::Valid,
            HtmlMediaMatchOutcome::Matches,
            HtmlMediaCondition::CssMatches,
        ),
        (
            HtmlMediaGrammarOutcome::Invalid,
            HtmlMediaMatchOutcome::DoesNotMatch,
            HtmlMediaCondition::CssDoesNotMatch,
        ),
        (
            HtmlMediaGrammarOutcome::Invalid,
            HtmlMediaMatchOutcome::Matches,
            HtmlMediaCondition::CssMatches,
        ),
        (
            HtmlMediaGrammarOutcome::Unsupported(HtmlMediaExternalCapability::Edition),
            HtmlMediaMatchOutcome::Unsupported(HtmlMediaExternalCapability::EnvironmentFeature),
            HtmlMediaCondition::Unsupported(HtmlMediaExternalCapability::EnvironmentFeature),
        ),
    ] {
        let (c, id) = single(KnownHtmlElement::Style, &[("media", "screen")]);
        let rev = revision();
        let f = fact(&c, &id, &rev, Some(g), Some(m));
        let cx = with_facts(&c, rev.clone(), facts(&c, rev.clone(), vec![f]).unwrap());
        let r = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(100),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .query(&[request(&id)])
        .unwrap();
        assert_eq!(value(&r).grammar(), &HtmlMediaGrammarEvidence::External(g));
        assert_eq!(condition(value(&r)), &expected);
        assert_eq!(r.qualification().media_revision(), Some(&rev));
        assert_eq!(r.capture(), c.identity());
        assert_eq!(r.qualification().context(), cx.identity());
    }
    // An invalid grammar witness cannot fill absent environment evidence.
    let (c, id) = single(KnownHtmlElement::Style, &[("media", "?")]);
    let rev = revision();
    let f = fact(&c, &id, &rev, Some(HtmlMediaGrammarOutcome::Invalid), None);
    let cx = with_facts(&c, rev.clone(), facts(&c, rev, vec![f]).unwrap());
    let r = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request(&id)])
    .unwrap();
    assert_eq!(
        condition(value(&r)),
        &HtmlMediaCondition::Missing(HtmlSemanticMissing::MediaEnvironment(id))
    );
    // Even a supplied mismatch cannot overturn the HTML empty-string rule.
    let (c, id) = single(KnownHtmlElement::Style, &[("media", "")]);
    let rev = revision();
    let f = fact(
        &c,
        &id,
        &rev,
        Some(HtmlMediaGrammarOutcome::Invalid),
        Some(HtmlMediaMatchOutcome::DoesNotMatch),
    );
    let cx = with_facts(&c, rev.clone(), facts(&c, rev, vec![f]).unwrap());
    let r = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request(&id)])
    .unwrap();
    assert_eq!(condition(value(&r)), &HtmlMediaCondition::Unconditional);
    assert_eq!(
        value(&r).grammar(),
        &HtmlMediaGrammarEvidence::External(HtmlMediaGrammarOutcome::Invalid)
    );
}

#[test]
fn facts_reject_foreign_stale_duplicate_and_capacity_before_any_query() {
    let (c, id) = single(KnownHtmlElement::Style, &[("media", "screen")]);
    let (other, other_id) = single(KnownHtmlElement::Style, &[("media", "screen")]);
    assert!(matches!(
        HtmlMediaAttributeInput::new(&c, other_id, 0),
        Err(HtmlMediaFactsError::Capture(
            HtmlCaptureError::ForeignCapture
        ))
    ));
    assert_eq!(
        HtmlMediaAttributeInput::new(&c, id.clone(), 1),
        Err(HtmlMediaFactsError::InvalidInput)
    );
    let rev = revision();
    let f = fact(
        &c,
        &id,
        &rev,
        Some(HtmlMediaGrammarOutcome::Valid),
        Some(HtmlMediaMatchOutcome::Matches),
    );
    assert_eq!(
        f.input().raw(&c).unwrap().code_units(),
        &[115, 99, 114, 101, 101, 110]
    );
    assert!(matches!(
        facts(&other, rev.clone(), vec![f.clone()]),
        Err(HtmlMediaFactsError::Capture(
            HtmlCaptureError::ForeignCapture
        ))
    ));
    assert!(matches!(
        facts(&c, rev.clone(), vec![f.clone(), f.clone()]),
        Err(HtmlMediaFactsError::DuplicateInput(_))
    ));
    let changed_env = HtmlMediaContextRevision::new(
        rev.grammar().clone(),
        HtmlMediaEnvironmentRevision::new(rev.environment().producer().clone(), 3, 5),
    );
    assert!(matches!(
        facts(&c, changed_env.clone(), vec![f.clone()]),
        Err(HtmlMediaFactsError::MatchRevisionMismatch)
    ));
    let grammar_only = fact(&c, &id, &rev, Some(HtmlMediaGrammarOutcome::Valid), None);
    assert!(facts(&c, changed_env.clone(), vec![grammar_only]).is_ok());
    assert!(matches!(
        facts(&c, revision(), vec![f.clone()]),
        Err(HtmlMediaFactsError::GrammarRevisionMismatch)
    ));
    assert!(matches!(
        HtmlMediaFacts::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            rev.clone(),
            vec![],
            HtmlMediaFactsLimits::new(1, 1000, 1000)
        ),
        Err(HtmlMediaFactsError::HostRevisionMismatch)
    ));
    for l in [
        HtmlMediaFactsLimits::new(0, 100000, 100000),
        HtmlMediaFactsLimits::new(1, 0, 100000),
        HtmlMediaFactsLimits::new(1, 100000, 0),
    ] {
        assert!(
            HtmlMediaFacts::new(
                &c,
                c.host_facts_revision().clone(),
                rev.clone(),
                vec![f.clone()],
                l
            )
            .is_err()
        );
    }
    let fs = facts(&c, rev.clone(), vec![f]).unwrap();
    assert!(matches!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_media(changed_env, Some(fs)),
            0
        ),
        Err(HtmlSemanticContextError::MediaRevisionMismatch)
    ));
}

#[test]
fn environment_revision_changes_match_without_reinterpreting_grammar() {
    let (c, id) = single(KnownHtmlElement::Style, &[("media", "screen")]);
    let rev = revision();
    for (environment, outcome, expected) in [
        (
            4,
            HtmlMediaMatchOutcome::Matches,
            HtmlMediaCondition::CssMatches,
        ),
        (
            5,
            HtmlMediaMatchOutcome::DoesNotMatch,
            HtmlMediaCondition::CssDoesNotMatch,
        ),
    ] {
        let current = HtmlMediaContextRevision::new(
            rev.grammar().clone(),
            HtmlMediaEnvironmentRevision::new(rev.environment().producer().clone(), 3, environment),
        );
        let f = fact(
            &c,
            &id,
            &current,
            Some(HtmlMediaGrammarOutcome::Valid),
            Some(outcome),
        );
        let cx = with_facts(
            &c,
            current.clone(),
            facts(&c, current.clone(), vec![f]).unwrap(),
        );
        let r = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(100),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .query(&[request(&id)])
        .unwrap();
        assert_eq!(condition(value(&r)), &expected);
        assert_eq!(
            value(&r).grammar(),
            &HtmlMediaGrammarEvidence::External(HtmlMediaGrammarOutcome::Valid)
        );
        assert_eq!(r.qualification().media_revision(), Some(&current));
    }
}

fn picture(
    a_fields: &[(&str, &str)],
    siblings: &[(KnownHtmlElement, &[(&str, &str)])],
    before: bool,
) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder(HtmlDocumentClass::Html);
    let p = element(
        &mut b,
        &d,
        KnownHtmlElement::Picture.qualified_name(),
        attrs(&[]),
    );
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Source.qualified_name(),
        attrs(a_fields),
    );
    let mut cs = Vec::new();
    if !before {
        cs.push(a.clone());
    }
    for (kind, fields) in siblings {
        let z = element(&mut b, &d, kind.qualified_name(), attrs(fields));
        children(&mut b, &z, &[]);
        cs.push(z);
    }
    if before {
        cs.push(a.clone());
    }
    children(&mut b, &a, &[]);
    children(&mut b, &p, &cs);
    children(&mut b, &d, &[p]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    (c, id)
}
fn requirement(r: &HtmlSemanticReport) -> &HtmlPictureSourceRequirement {
    match value(r).processing() {
        HtmlMediaProcessing::Source {
            picture_requirement,
            ..
        } => picture_requirement,
        other => panic!("{other:?}"),
    }
}

#[test]
fn picture_requirement_uses_only_actual_following_siblings_and_presence() {
    for (fields, expected) in [
        (
            vec![("type", "")],
            HtmlPictureSourceRequirement::SatisfiedByTypePresence,
        ),
        (
            vec![("media", "screen")],
            HtmlPictureSourceRequirement::SatisfiedByRestrictiveMedia,
        ),
        (
            vec![("media", "\u{A0}all")],
            HtmlPictureSourceRequirement::SatisfiedByRestrictiveMedia,
        ),
    ] {
        let (c, id) = picture(
            &fields,
            &[(KnownHtmlElement::Img, &[("srcset", "")])],
            false,
        );
        assert_eq!(requirement(&run(&c, &id)), &expected);
    }
    for media in [None, Some(""), Some(" \t"), Some("all"), Some(" ALL \r")] {
        let fields = media.map_or_else(Vec::new, |v| vec![("media", v)]);
        for (kind, sibling_fields) in [
            (KnownHtmlElement::Source, &[][..]),
            (KnownHtmlElement::Img, &[("srcset", "")][..]),
        ] {
            let (c, id) = picture(&fields, &[(kind, sibling_fields)], false);
            assert!(matches!(
                requirement(&run(&c, &id)),
                HtmlPictureSourceRequirement::Violated { .. }
            ));
        }
        let (c, id) = picture(&fields, &[(KnownHtmlElement::Img, &[])], false);
        assert_eq!(
            requirement(&run(&c, &id)),
            &HtmlPictureSourceRequirement::NotTriggered
        );
        let (c, id) = picture(&fields, &[(KnownHtmlElement::Source, &[])], true);
        assert_eq!(
            requirement(&run(&c, &id)),
            &HtmlPictureSourceRequirement::NotTriggered
        );
    }
}
#[test]
fn picture_missing_trigger_is_not_a_violation_and_own_witness_can_resolve_it() {
    for (fields, witness) in [
        (&[][..], false),
        (&[("type", "")][..], true),
        (&[("media", "screen")][..], true),
    ] {
        let (mut b, d) = builder(HtmlDocumentClass::Html);
        let p = element(
            &mut b,
            &d,
            KnownHtmlElement::Picture.qualified_name(),
            attrs(&[]),
        );
        let a = element(
            &mut b,
            &d,
            KnownHtmlElement::Source.qualified_name(),
            attrs(fields),
        );
        b.establish_parent(&a, &p).unwrap();
        children(&mut b, &d, &[p]);
        let c = b.finish().unwrap();
        let id = c.subject_id(&a).unwrap();
        let r = run(&c, &id);
        assert_eq!(
            matches!(
                requirement(&r),
                HtmlPictureSourceRequirement::Missing(HtmlSemanticMissing::OrderedChildren(_))
            ),
            !witness
        );
    }
}

fn metas(
    class: HtmlDocumentClass,
    a_media: Option<&str>,
    z_media: Option<&str>,
    unknown: bool,
) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder(class);
    let mut fields = vec![("name", "theme-color")];
    if let Some(v) = a_media {
        fields.push(("media", v));
    }
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        attrs(&fields),
    );
    let mut fields = vec![("name", "THEME-COLOR")];
    if let Some(v) = z_media {
        fields.push(("media", v));
    }
    let z = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        attrs(&fields),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    let mut cs = vec![a.clone()];
    if unknown {
        cs.push(element(
            &mut b,
            &d,
            KnownHtmlElement::Div.qualified_name(),
            attrs(&[]),
        ));
    }
    cs.push(z);
    children(&mut b, &d, &cs);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    (c, id)
}
fn uniqueness(r: &HtmlSemanticReport) -> &HtmlThemeColorMediaUniqueness {
    match value(r).processing() {
        HtmlMediaProcessing::Meta(HtmlMetaMediaProcessing::ThemeColor { uniqueness, .. }) => {
            uniqueness
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn theme_uniqueness_exact_raw_presence_document_class_and_incomplete_coverage() {
    for (a, z, dup) in [
        (Some(""), Some(""), true),
        (Some("screen"), Some("screen"), true),
        (Some("screen"), Some("SCREEN"), false),
        (Some(" screen"), Some("screen"), false),
        (Some(""), None, false),
    ] {
        let (c, id) = metas(HtmlDocumentClass::Html, a, z, false);
        let r = run(&c, &id);
        assert_eq!(
            matches!(
                uniqueness(&r),
                HtmlThemeColorMediaUniqueness::Duplicate { .. }
            ),
            dup
        );
        if !dup {
            assert!(matches!(
                uniqueness(&r),
                HtmlThemeColorMediaUniqueness::Unique { .. }
            ));
        }
    }
    let (c, id) = metas(HtmlDocumentClass::Html, None, None, true);
    let r = run(&c, &id);
    assert_eq!(uniqueness(&r), &HtmlThemeColorMediaUniqueness::NotRequired);
    assert_eq!(condition(value(&r)), &HtmlMediaCondition::NoFilter);
    for (z, dup) in [("screen", true), ("print", false)] {
        let (c, id) = metas(HtmlDocumentClass::Html, Some("screen"), Some(z), true);
        let r = run(&c, &id);
        if dup {
            assert!(matches!(
                uniqueness(&r),
                HtmlThemeColorMediaUniqueness::Duplicate { .. }
            ));
        } else {
            assert!(matches!(
                uniqueness(&r),
                HtmlThemeColorMediaUniqueness::Missing(HtmlSemanticMissing::OrderedChildren(_))
            ));
        }
    }
    for class in [HtmlDocumentClass::NonHtml, HtmlDocumentClass::Unavailable] {
        let (c, id) = metas(class, Some(""), Some(""), false);
        let r = run(&c, &id);
        match class {
            HtmlDocumentClass::NonHtml => assert_eq!(
                uniqueness(&r),
                &HtmlThemeColorMediaUniqueness::NoHtmlDocumentScope
            ),
            _ => assert!(matches!(
                uniqueness(&r),
                HtmlThemeColorMediaUniqueness::Missing(HtmlSemanticMissing::DocumentClass(_))
            )),
        }
    }
}

#[test]
fn template_host_edges_and_detached_roots_do_not_expand_document_uniqueness() {
    let (mut b, d) = builder(HtmlDocumentClass::Html);
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        attrs(&[("name", "theme-color"), ("media", "screen")]),
    );
    let template = element(
        &mut b,
        &d,
        KnownHtmlElement::Template.qualified_name(),
        attrs(&[]),
    );
    let fragment = b.add_fragment(&d).unwrap();
    let z = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        attrs(&[("name", "theme-color"), ("media", "screen")]),
    );
    children(&mut b, &z, &[]);
    children(&mut b, &fragment, std::slice::from_ref(&z));
    b.establish_template_contents(&template, &fragment).unwrap();
    children(&mut b, &a, &[]);
    children(&mut b, &template, &[]);
    children(&mut b, &d, &[a.clone(), template]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let zid = c.subject_id(&z).unwrap();
    assert!(matches!(
        uniqueness(&run(&c, &id)),
        HtmlThemeColorMediaUniqueness::Unique { .. }
    ));
    assert_eq!(
        uniqueness(&run(&c, &zid)),
        &HtmlThemeColorMediaUniqueness::NoDocumentMetadataScope
    );
}

#[test]
fn semantic_atomic_limits_cancellation_scratch_release_and_sticky_stop() {
    let (c, id) = metas(
        HtmlDocumentClass::Html,
        Some("screen"),
        Some("print"),
        false,
    );
    let context = cx(&c);
    let mut s = HtmlSemanticSession::new(
        &c,
        &context,
        HtmlValueInterpreter::new(100),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let r = s.query(&[request(&id)]).unwrap();
    assert!(matches!(
        uniqueness(&r),
        HtmlThemeColorMediaUniqueness::Unique { .. }
    ));
    let used = r.usage();
    assert_eq!(used.scratch_records(), 0);
    assert!(used.peak_scratch_records() > 0);
    let exact = HtmlSemanticLimits::new(
        used.work(),
        used.output_records(),
        used.peak_scratch_records(),
        0,
        used.allocated_bytes(),
        used.nested_output_records(),
    );
    assert!(
        HtmlSemanticSession::new(
            &c,
            &context,
            HtmlValueInterpreter::new(100),
            exact,
            HtmlSemanticCancellation::new()
        )
        .unwrap()
        .query(&[request(&id)])
        .is_ok()
    );
    for l in [
        HtmlSemanticLimits::new(used.work() - 1, 100, 1000, 0, 1000000, 100000),
        HtmlSemanticLimits::new(20000000, 0, 1000, 0, 1000000, 100000),
        HtmlSemanticLimits::new(20000000, 100, 0, 0, 1000000, 100000),
        HtmlSemanticLimits::new(20000000, 100, 1000, 0, used.allocated_bytes() - 1, 100000),
        HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 0),
    ] {
        let mut s = HtmlSemanticSession::new(
            &c,
            &context,
            HtmlValueInterpreter::new(100),
            l,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let failed = s.query(&[request(&id)]).unwrap_err();
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.failure(), Some(&failed));
        assert_eq!(s.query(&[]).unwrap_err(), failed);
    }
    // A completed first request cannot be returned when the second raw value fails.
    let (mut b, d) = builder(HtmlDocumentClass::Html);
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Style.qualified_name(),
        attrs(&[("media", "")]),
    );
    let z = element(
        &mut b,
        &d,
        KnownHtmlElement::Style.qualified_name(),
        attrs(&[("media", "screen")]),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &d, &[a.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let context = cx(&c);
    let ids = [c.subject_id(&a).unwrap(), c.subject_id(&z).unwrap()];
    let mut s = HtmlSemanticSession::new(
        &c,
        &context,
        HtmlValueInterpreter::new(0),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let e = s.query(&[request(&ids[0]), request(&ids[1])]).unwrap_err();
    assert!(matches!(e.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), e);
    let cancellation = HtmlSemanticCancellation::new();
    cancellation.cancel();
    let mut s = HtmlSemanticSession::new(
        &c,
        &context,
        HtmlValueInterpreter::new(100),
        limits(),
        cancellation,
    )
    .unwrap();
    let e = s.query(&[request(&ids[0])]).unwrap_err();
    assert_eq!(e.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[]).unwrap_err(), e);
}

#[test]
fn expanded_field_identity_foreign_roles_and_isolated_utf16_are_preserved() {
    let (mut b, d) = builder(HtmlDocumentClass::Html);
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some(DocumentString::from("p")), "media"),
        DocumentString::from_utf16(vec![0xD800, 0xA0]),
    );
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Style.qualified_name(),
        OrderedAttributes::new([raw.clone()]).unwrap(),
    );
    let foreign = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "style"),
        attrs(&[("media", "")]),
    );
    let unrelated = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        attrs(&[("media", "")]),
    );
    let wrong_field = element(
        &mut b,
        &d,
        KnownHtmlElement::Style.qualified_name(),
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "media"),
            "screen",
        )])
        .unwrap(),
    );
    for a in [&a, &foreign, &unrelated, &wrong_field] {
        children(&mut b, a, &[]);
    }
    children(
        &mut b,
        &d,
        &[
            a.clone(),
            foreign.clone(),
            unrelated.clone(),
            wrong_field.clone(),
        ],
    );
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let r = run(&c, &id);
    assert_eq!(value(&r).local().raw(), Some(&raw));
    assert_eq!(
        value(&r).local().value().class(),
        HtmlMediaValueClass::Nontrivial
    );
    assert_eq!(
        HtmlMediaAttributeInput::new(&c, id.clone(), 0)
            .unwrap()
            .raw(&c)
            .unwrap()
            .code_units(),
        &[0xD800, 0xA0]
    );
    let r = run(&c, &c.subject_id(&wrong_field).unwrap());
    assert_eq!(condition(value(&r)), &HtmlMediaCondition::DefaultAll);
    assert!(value(&r).local().raw().is_none());
    for (a, expected) in [(foreign, true), (unrelated, false)] {
        let id = c.subject_id(&a).unwrap();
        assert_eq!(
            HtmlMediaAttributeInput::new(&c, id.clone(), 0),
            Err(HtmlMediaFactsError::InvalidInput)
        );
        let r = run(&c, &id);
        match r.results()[0].fact() {
            HtmlSemanticFact::MediaAttributeRules(v) if expected => assert_eq!(
                v,
                &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
            ),
            HtmlSemanticFact::MediaAttributeRules(v) => {
                assert_eq!(v, &HtmlSemanticValue::Inapplicable)
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn source_media_filter_distinguishes_absence_unavailable_and_actual_mismatch() {
    for parent in [KnownHtmlElement::Audio, KnownHtmlElement::Video] {
        for raw in [None, Some(""), Some("screen")] {
            let (mut b, d) = builder(HtmlDocumentClass::Html);
            let p = element(&mut b, &d, parent.qualified_name(), attrs(&[]));
            let fields = raw.map_or_else(Vec::new, |v| vec![("media", v)]);
            let a = element(
                &mut b,
                &d,
                KnownHtmlElement::Source.qualified_name(),
                attrs(&fields),
            );
            children(&mut b, &a, &[]);
            children(&mut b, &p, std::slice::from_ref(&a));
            children(&mut b, &d, &[p]);
            let c = b.finish().unwrap();
            let id = c.subject_id(&a).unwrap();
            let r = run(&c, &id);
            let HtmlMediaProcessing::Source {
                context, decision, ..
            } = value(&r).processing()
            else {
                panic!()
            };
            assert_eq!(context, &HtmlSourceMediaContext::MediaElement);
            match raw {
                None => assert_eq!(decision, &HtmlSourceMediaDecision::NoFilter),
                Some("") => assert_eq!(
                    decision,
                    &HtmlSourceMediaDecision::NoMediaExclusion(HtmlMediaCondition::Unconditional)
                ),
                _ => {
                    assert_eq!(
                        decision,
                        &HtmlSourceMediaDecision::Missing(HtmlSemanticMissing::MediaEnvironment(
                            id.clone()
                        ))
                    );
                    let rev = revision();
                    let f = fact(
                        &c,
                        &id,
                        &rev,
                        None,
                        Some(HtmlMediaMatchOutcome::DoesNotMatch),
                    );
                    let context = with_facts(&c, rev.clone(), facts(&c, rev, vec![f]).unwrap());
                    let r = HtmlSemanticSession::new(
                        &c,
                        &context,
                        HtmlValueInterpreter::new(100),
                        limits(),
                        HtmlSemanticCancellation::new(),
                    )
                    .unwrap()
                    .query(&[request(&id)])
                    .unwrap();
                    assert!(matches!(
                        value(&r).processing(),
                        HtmlMediaProcessing::Source {
                            decision: HtmlSourceMediaDecision::Skip,
                            ..
                        }
                    ));
                }
            }
        }
    }
}

#[test]
fn shadow_tree_uniqueness_has_no_host_document_scope_or_cross_tree_duplicate() {
    let (mut b, d) = builder(HtmlDocumentClass::Html);
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        attrs(&[("name", "theme-color"), ("media", "")]),
    );
    let host = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        attrs(&[]),
    );
    let shadow = b
        .add_shadow_root(
            &d,
            &host,
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Open,
                    HtmlSlotAssignment::Named,
                    false,
                    false,
                    false,
                ),
                false,
                false,
                false,
            ),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let z = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        attrs(&[("name", "theme-color"), ("media", "")]),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &host, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &shadow, std::slice::from_ref(&z));
    children(&mut b, &d, &[a.clone(), host]);
    let c = b.finish().unwrap();
    assert!(matches!(
        uniqueness(&run(&c, &c.subject_id(&a).unwrap())),
        HtmlThemeColorMediaUniqueness::Unique { .. }
    ));
    assert_eq!(
        uniqueness(&run(&c, &c.subject_id(&z).unwrap())),
        &HtmlThemeColorMediaUniqueness::NoDocumentMetadataScope
    );
}

#[test]
fn incoming_facts_actual_storage_and_validation_work_have_exact_boundaries() {
    let (c, id) = single(KnownHtmlElement::Style, &[("media", "screen")]);
    let rev = revision();
    let f = fact(&c, &id, &rev, None, None);
    let bytes = std::mem::size_of::<HtmlMediaFact>();
    assert!(
        HtmlMediaFacts::new(
            &c,
            c.host_facts_revision().clone(),
            rev.clone(),
            vec![f.clone()],
            HtmlMediaFactsLimits::new(1, bytes, 66)
        )
        .is_ok()
    );
    assert!(matches!(
        HtmlMediaFacts::new(
            &c,
            c.host_facts_revision().clone(),
            rev.clone(),
            vec![f.clone()],
            HtmlMediaFactsLimits::new(1, bytes - 1, 66)
        ),
        Err(HtmlMediaFactsError::ByteLimit)
    ));
    assert!(matches!(
        HtmlMediaFacts::new(
            &c,
            c.host_facts_revision().clone(),
            rev.clone(),
            vec![f.clone()],
            HtmlMediaFactsLimits::new(1, bytes, 65)
        ),
        Err(HtmlMediaFactsError::WorkLimit)
    ));
    let mut overreserved = Vec::with_capacity(8);
    overreserved.push(f);
    assert!(matches!(
        HtmlMediaFacts::new(
            &c,
            c.host_facts_revision().clone(),
            rev,
            overreserved,
            HtmlMediaFactsLimits::new(1, bytes, 100)
        ),
        Err(HtmlMediaFactsError::RecordLimit)
    ));
}

#[test]
fn context_rejects_other_capture_and_missing_css_attestations_stay_missing() {
    let (c, id) = single(KnownHtmlElement::Style, &[("media", "screen")]);
    let (other, _) = single(KnownHtmlElement::Style, &[("media", "screen")]);
    let rev = revision();
    let f = fact(
        &c,
        &id,
        &rev,
        Some(HtmlMediaGrammarOutcome::Unavailable),
        Some(HtmlMediaMatchOutcome::Unavailable),
    );
    let fs = facts(&c, rev.clone(), vec![f.clone()]).unwrap();
    assert!(matches!(
        HtmlSemanticContext::new(
            &other,
            other.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_media(rev.clone(), Some(fs)),
            0
        ),
        Err(HtmlSemanticContextError::MediaCaptureMismatch)
    ));
    let cx = with_facts(&c, rev.clone(), facts(&c, rev, vec![f]).unwrap());
    let r = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request(&id)])
    .unwrap();
    assert_eq!(
        value(&r).grammar(),
        &HtmlMediaGrammarEvidence::Missing(HtmlSemanticMissing::MediaGrammar(id.clone()))
    );
    assert_eq!(
        condition(value(&r)),
        &HtmlMediaCondition::Missing(HtmlSemanticMissing::MediaEnvironment(id))
    );
}

#[test]
fn unknown_parent_remains_explicit_and_does_not_erase_sufficient_own_witness() {
    for fields in [vec![], vec![("type", "")], vec![("media", "screen")]] {
        let (mut b, d) = builder(HtmlDocumentClass::Html);
        let a = element(
            &mut b,
            &d,
            KnownHtmlElement::Source.qualified_name(),
            attrs(&fields),
        );
        let c = b.finish().unwrap();
        let id = c.subject_id(&a).unwrap();
        let r = run(&c, &id);
        let HtmlMediaProcessing::Source {
            context,
            picture_requirement,
            ..
        } = value(&r).processing()
        else {
            panic!()
        };
        assert_eq!(
            context,
            &HtmlSourceMediaContext::Missing(HtmlSemanticMissing::Parent(id.clone()))
        );
        let expected = match fields.first().map(|f| f.0) {
            Some("type") => HtmlPictureSourceRequirement::SatisfiedByTypePresence,
            Some(_) => HtmlPictureSourceRequirement::SatisfiedByRestrictiveMedia,
            None => HtmlPictureSourceRequirement::Missing(HtmlSemanticMissing::Parent(id)),
        };
        assert_eq!(picture_requirement, &expected);
    }
}

#[test]
fn foreign_and_descendant_source_lookalikes_cannot_trigger_picture_requirement() {
    let (mut b, d) = builder(HtmlDocumentClass::Html);
    let p = element(
        &mut b,
        &d,
        KnownHtmlElement::Picture.qualified_name(),
        attrs(&[]),
    );
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Source.qualified_name(),
        attrs(&[]),
    );
    let foreign = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "source"),
        attrs(&[]),
    );
    let wrapper = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        attrs(&[]),
    );
    let inner = element(
        &mut b,
        &d,
        KnownHtmlElement::Source.qualified_name(),
        attrs(&[]),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &foreign, &[]);
    children(&mut b, &inner, &[]);
    children(&mut b, &wrapper, &[inner]);
    children(&mut b, &p, &[a.clone(), foreign, wrapper]);
    children(&mut b, &d, &[p]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    assert_eq!(
        requirement(&run(&c, &id)),
        &HtmlPictureSourceRequirement::NotTriggered
    );
}

#[test]
fn captured_media_raw_admission_precedes_namespace_and_role_shortcuts() {
    let (mut b, d) = builder(HtmlDocumentClass::Html);
    let mut handles = Vec::new();
    for name in [
        KnownHtmlElement::Style.qualified_name(),
        KnownHtmlElement::Div.qualified_name(),
        QualifiedName::new(Namespace::svg(), None, "style"),
    ] {
        let a = element(&mut b, &d, name, attrs(&[("media", "screen")]));
        children(&mut b, &a, &[]);
        handles.push(a);
    }
    children(&mut b, &d, &handles);
    let c = b.finish().unwrap();
    let context = cx(&c);
    let mut observed = Vec::new();
    for a in &handles {
        let id = c.subject_id(a).unwrap();
        let mut s = HtmlSemanticSession::new(
            &c,
            &context,
            HtmlValueInterpreter::new(0),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let result = s.query(&[request(&id)]);
        assert_eq!(s.usage().scratch_records(), 0);
        if let Err(e) = &result {
            assert_eq!(s.query(&[]).unwrap_err(), *e);
        }
        observed.push(
            matches!(result, Err(e) if matches!(e.kind(), HtmlSemanticFailureKind::Attribute(_))),
        );
    }
    // All three actual requests execute before the admission-order discriminator.
    assert_eq!(observed, [true, true, true]);
}
