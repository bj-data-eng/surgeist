use surgeist_html::*;

fn limits() -> HtmlRelFactsLimits {
    HtmlRelFactsLimits::new(10000, 2_000_000, 2_000_000)
}
fn revision(n: u64) -> HtmlRelRevision {
    HtmlRelRevision::new(HtmlRelProducerId::fresh(), 4, n)
}
fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(200, 1000, 100, 10000, 200, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    tag: &str,
    values: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, tag),
        OrderedAttributes::new(
            values
                .iter()
                .map(|(k, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *k), *v)),
        )
        .unwrap(),
        HtmlCapturedIsValue::Absent,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn link(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    values: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(b, d, Namespace::html(), "link", values)
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}
fn context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
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
        HtmlSemanticLimits::new(2_000_000, 100, 10000, 10000, 2_000_000, 100000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
}
fn request(
    c: &HtmlCapture,
    d: &HtmlCaptureBuilderSubject,
    q: HtmlSemanticQuery,
) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(c.subject_id(d).unwrap(), q)
}
fn feeds(r: &HtmlSemanticReport) -> &[HtmlCaptureSubjectId] {
    let HtmlSemanticFact::DocumentFeeds(HtmlSemanticValue::Known(feeds)) = r.results()[0].fact()
    else {
        panic!("known complete feeds")
    };
    feeds
}
fn collection(r: &HtmlSemanticReport) -> &HtmlDocumentIconCollection {
    let HtmlSemanticFact::DocumentIcon(HtmlSemanticValue::Known(
        HtmlDocumentIconReadback::Declarations { collection, .. },
    )) = r.results()[0].fact()
    else {
        panic!("complete icon declaration collection")
    };
    collection
}
fn choice(r: &HtmlSemanticReport) -> Option<&HtmlDocumentIconChoice> {
    let value = match r.results()[0].fact() {
        HtmlSemanticFact::DocumentIcon(HtmlSemanticValue::Known(
            HtmlDocumentIconReadback::Selection(value),
        ))
        | HtmlSemanticFact::NextDocumentIcon(HtmlSemanticValue::Known(
            HtmlDocumentIconReadback::Selection(value),
        )) => value,
        _ => panic!("icon selection result"),
    };
    let HtmlSemanticValue::Known(choice) = value else {
        panic!("known selected icon")
    };
    choice.as_ref()
}
fn expected(media: Option<HtmlMediaContextRevision>) -> HtmlIconExpectedRevisions {
    HtmlIconExpectedRevisions::new(
        HtmlIconPresentationId::fresh(),
        revision(1),
        revision(2),
        media,
    )
}
fn observations(
    c: &HtmlDocumentIconCollection,
    expected: &HtmlIconExpectedRevisions,
    states: &[HtmlIconSuitability],
) -> Vec<HtmlIconSuitabilityObservation> {
    c.candidates()
        .iter()
        .zip(states)
        .map(|(candidate, state)| {
            if *state == HtmlIconSuitability::Unavailable {
                HtmlIconSuitabilityObservation::unavailable(
                    c,
                    candidate.subject().clone(),
                    expected.clone(),
                )
            } else {
                HtmlIconSuitabilityObservation::new(
                    c,
                    candidate.subject().clone(),
                    HtmlIconResourceKey::new(
                        HtmlIconResourceId::fresh(),
                        expected.resource().clone(),
                    ),
                    expected.clone(),
                    *state,
                )
            }
        })
        .collect()
}
fn three_icons() -> (
    HtmlCapture,
    HtmlSemanticContext,
    HtmlCaptureSubjectId,
    Vec<HtmlCaptureSubjectId>,
) {
    let (mut b, d) = builder();
    let nodes: Vec<_> = (0..3)
        .map(|_| link(&mut b, &d, &[("rel", "ICON icon"), ("sizes", "any 16X32")]))
        .collect();
    for n in &nodes {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &nodes);
    let c = b.finish().unwrap();
    let doc = c.subject_id(&d).unwrap();
    let ids = nodes.iter().map(|n| c.subject_id(n).unwrap()).collect();
    let cx = context(&c);
    (c, cx, doc, ids)
}

#[test]
fn feed_discovery_includes_stylesheet_alternates_and_default_is_first_declaration() {
    for kind in ["application/rss+xml", "application/atom+xml"] {
        for rel in [
            "alternate stylesheet",
            "STYLESHEET Alternate alternate",
            "alternate",
        ] {
            let (mut b, d) = builder();
            let a = link(&mut b, &d, &[("rel", rel), ("type", kind)]);
            let z = link(
                &mut b,
                &d,
                &[
                    ("rel", "alternate"),
                    ("type", "application/atom+xml"),
                    ("href", "later"),
                ],
            );
            for n in [&a, &z] {
                children(&mut b, n, &[]);
            }
            children(&mut b, &d, &[a.clone(), z.clone()]);
            let c = b.finish().unwrap();
            let cx = context(&c);
            let ids = [c.subject_id(&a).unwrap(), c.subject_id(&z).unwrap()];
            let r = session(&c, &cx, 100)
                .query(&[request(&c, &d, HtmlSemanticQuery::DocumentFeeds)])
                .unwrap();
            assert_eq!(feeds(&r), ids);
            let input = HtmlDefaultFeedInput::new(
                &c,
                &cx,
                c.subject_id(&d).unwrap(),
                HtmlDefaultFeedApplicability::Applicable,
                limits(),
            )
            .unwrap();
            let r = session(&c, &cx, 100)
                .query(&[HtmlSemanticRequest::default_feed(input)])
                .unwrap();
            assert_eq!(
                r.results()[0].fact(),
                &HtmlSemanticFact::DefaultFeed(HtmlSemanticValue::Known(Some(ids[0].clone())))
            );
            let local = session(&c, &cx, 100)
                .query(&[request(&c, &a, HtmlSemanticQuery::LinkRelationships)])
                .unwrap();
            let HtmlSemanticFact::LinkRelationships(HtmlSemanticValue::Known(local)) =
                local.results()[0].fact()
            else {
                panic!()
            };
            let alternate = local
                .local()
                .occurrences()
                .iter()
                .find(|x| x.keyword() == Some(HtmlRelKeyword::Alternate))
                .unwrap();
            assert_eq!(
                alternate.effect(),
                Some(if rel == "alternate" {
                    HtmlRelEffect::Hyperlink
                } else {
                    HtmlRelEffect::StylesheetModifier
                })
            );
        }
    }
}
#[test]
fn feed_type_is_exact_and_anchor_area_are_never_document_candidates() {
    let (mut b, d) = builder();
    let mut nodes = Vec::new();
    for tag in ["a", "area", "link"] {
        for kind in [
            "Application/rss+xml",
            "application/rss+xml;param",
            " application/rss+xml",
            "application/atom+xml ",
            "application/rss+xml",
        ] {
            let n = element(
                &mut b,
                &d,
                Namespace::html(),
                tag,
                &[("rel", "alternate"), ("type", kind)],
            );
            children(&mut b, &n, &[]);
            nodes.push(n);
        }
    }
    let negative = link(&mut b, &d, &[("rel", "unknown"), ("type", "text/plain")]);
    children(&mut b, &negative, &[]);
    nodes.push(negative);
    children(&mut b, &d, &nodes);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx, 100)
        .query(&[request(&c, &d, HtmlSemanticQuery::DocumentFeeds)])
        .unwrap();
    assert_eq!(feeds(&r), [c.subject_id(&nodes[14]).unwrap()]);
}
#[test]
fn first_candidate_ignores_later_children_but_collections_require_complete_coverage() {
    for rel in ["alternate", "manifest"] {
        let (mut b, d) = builder();
        let a = link(&mut b, &d, &[("rel", rel), ("type", "application/rss+xml")]);
        let later = link(&mut b, &d, &[("rel", rel), ("href", "ready")]);
        children(&mut b, &d, &[a.clone(), later]); // Candidate's own children intentionally unavailable.
        let c = b.finish().unwrap();
        let cx = context(&c);
        let subject = c.subject_id(&a).unwrap();
        let req = if rel == "manifest" {
            request(&c, &d, HtmlSemanticQuery::FirstManifest)
        } else {
            HtmlSemanticRequest::default_feed(
                HtmlDefaultFeedInput::new(
                    &c,
                    &cx,
                    c.subject_id(&d).unwrap(),
                    HtmlDefaultFeedApplicability::Applicable,
                    limits(),
                )
                .unwrap(),
            )
        };
        let r = session(&c, &cx, 100).query(&[req]).unwrap();
        let expected = HtmlSemanticValue::Known(Some(subject.clone()));
        assert_eq!(
            r.results()[0].fact(),
            &if rel == "manifest" {
                HtmlSemanticFact::FirstManifest(expected)
            } else {
                HtmlSemanticFact::DefaultFeed(expected)
            }
        );
        if rel == "alternate" {
            let r = session(&c, &cx, 100)
                .query(&[request(&c, &d, HtmlSemanticQuery::DocumentFeeds)])
                .unwrap();
            assert_eq!(
                r.results()[0].fact(),
                &HtmlSemanticFact::DocumentFeeds(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::OrderedChildren(subject)
                ))
            );
        }
    }
}
#[test]
fn earlier_tree_or_registry_gap_blocks_first_manifest_and_known_membership_resolves_other_unknowns()
{
    for gap in ["tree", "registry", "known"] {
        let (mut b, d) = builder();
        let a = link(
            &mut b,
            &d,
            &[(
                "rel",
                if gap == "known" {
                    "unknown MANIFEST"
                } else if gap == "registry" {
                    "unknown"
                } else {
                    ""
                },
            )],
        );
        let z = link(&mut b, &d, &[("rel", "manifest"), ("href", "later")]);
        if gap != "tree" {
            children(&mut b, &a, &[]);
        }
        children(&mut b, &z, &[]);
        children(&mut b, &d, &[a.clone(), z]);
        let c = b.finish().unwrap();
        let cx = context(&c);
        let id = c.subject_id(&a).unwrap();
        let r = session(&c, &cx, 100)
            .query(&[request(&c, &d, HtmlSemanticQuery::FirstManifest)])
            .unwrap();
        let expected = match gap {
            "tree" => HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(id)),
            "registry" => HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
                subject: id,
                fact: HtmlRelMissingFact::Registry,
            }),
            _ => HtmlSemanticValue::Known(Some(id)),
        };
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::FirstManifest(expected)
        );
    }
}
#[test]
fn first_manifest_does_not_filter_missing_empty_invalid_href_or_format() {
    for href in [None, Some(""), Some("\t "), Some("not a URL")] {
        let (mut b, d) = builder();
        let mut fields = vec![("rel", "manifest"), ("type", "unsupported")];
        if let Some(href) = href {
            fields.push(("href", href));
        }
        let a = link(&mut b, &d, &fields);
        let z = link(
            &mut b,
            &d,
            &[("rel", "manifest"), ("href", "later.webmanifest")],
        );
        children(&mut b, &d, &[a.clone(), z]);
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = session(&c, &cx, 100)
            .query(&[request(&c, &d, HtmlSemanticQuery::FirstManifest)])
            .unwrap();
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::FirstManifest(HtmlSemanticValue::Known(Some(
                c.subject_id(&a).unwrap()
            )))
        );
    }
}
#[test]
fn foreign_wrappers_include_ordinary_descendants_and_detached_candidates_are_excluded() {
    let (mut b, d) = builder();
    let f = element(&mut b, &d, Namespace::svg(), "g", &[]);
    let a = link(&mut b, &d, &[("rel", "manifest")]);
    let detached = link(&mut b, &d, &[("rel", "manifest")]);
    children(&mut b, &detached, &[]);
    children(&mut b, &f, std::slice::from_ref(&a));
    children(&mut b, &d, std::slice::from_ref(&f));
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx, 100)
        .query(&[request(&c, &d, HtmlSemanticQuery::FirstManifest)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::FirstManifest(HtmlSemanticValue::Known(Some(c.subject_id(&a).unwrap())))
    );
}
#[test]
fn default_applicability_is_independent_and_empty_requires_complete_negative_proof() {
    let (mut b, d) = builder();
    children(&mut b, &d, &[]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let doc = c.subject_id(&d).unwrap();
    for applicability in [
        HtmlDefaultFeedApplicability::Applicable,
        HtmlDefaultFeedApplicability::Inapplicable,
        HtmlDefaultFeedApplicability::Unavailable,
    ] {
        let input =
            HtmlDefaultFeedInput::new(&c, &cx, doc.clone(), applicability, limits()).unwrap();
        let r = session(&c, &cx, 0)
            .query(&[HtmlSemanticRequest::default_feed(input)])
            .unwrap();
        let expected = match applicability {
            HtmlDefaultFeedApplicability::Applicable => HtmlSemanticValue::Known(None),
            HtmlDefaultFeedApplicability::Inapplicable => HtmlSemanticValue::Inapplicable,
            HtmlDefaultFeedApplicability::Unavailable => {
                HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
                    subject: doc.clone(),
                    fact: HtmlRelMissingFact::DefaultFeedApplicability,
                })
            }
        };
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::DefaultFeed(expected)
        );
    }
    assert_eq!(
        HtmlDefaultFeedInput::new(
            &c,
            &cx,
            doc,
            HtmlDefaultFeedApplicability::Inapplicable,
            HtmlRelFactsLimits::new(0, 0, 0)
        )
        .unwrap_err(),
        HtmlRelFactsError::RecordLimit
    );
}
#[test]
fn icon_collection_preserves_exact_arbitrary_dimensions_invalid_neighbors_and_absent_type() {
    let (mut b, d) = builder();
    let raw = "ANY 340282366920938463463374607431768211456X2 01x1 2x3";
    let a = link(&mut b, &d, &[("rel", "icon icon"), ("sizes", raw)]);
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx, 100)
        .query(&[request(&c, &d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    let items = collection(&r).candidates();
    assert_eq!(items.len(), 1);
    let a = &items[0];
    assert!(a.type_hint().is_none());
    assert!(a.href().is_none());
    assert_eq!(a.sizes().unwrap().ordinal(), 1);
    assert_eq!(a.sizes().unwrap().raw(), &DocumentString::from(raw));
    assert_eq!(a.size_occurrences().len(), 4);
    assert_eq!(
        a.size_occurrences()[0].declaration(),
        &HtmlIconSizeDeclaration::Scalable
    );
    let HtmlIconSizeDeclaration::Bitmap { width, height } = a.size_occurrences()[1].declaration()
    else {
        panic!()
    };
    assert_eq!(
        a.digits(width.clone()).unwrap(),
        "340282366920938463463374607431768211456"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(a.digits(height.clone()).unwrap(), [50]);
    assert_eq!(
        a.size_occurrences()[2].declaration(),
        &HtmlIconSizeDeclaration::Invalid(HtmlIconSizeError::LeadingZero)
    );
    assert!(matches!(
        a.media_rules().grammar(),
        HtmlMediaGrammarEvidence::NotSpecified
    ));
}
#[test]
fn equally_appropriate_icons_use_tree_order_and_rejection_tries_same_group_before_lower_group() {
    let (c, cx, d, ids) = three_icons();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::new(
            d.clone(),
            HtmlSemanticQuery::DocumentIcon,
        )])
        .unwrap();
    let expected = expected(None);
    let obs = observations(
        collection(&r),
        &expected,
        &[HtmlIconSuitability::Suitable; 3],
    );
    let input = HtmlDocumentIconInput::new(
        &c,
        &cx,
        collection(&r).clone(),
        expected,
        obs,
        HtmlIconOrdering::Known(vec![
            vec![ids[1].clone(), ids[0].clone()],
            vec![ids[2].clone()],
        ]),
        limits(),
    )
    .unwrap();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::document_icon(input)])
        .unwrap();
    let mut previous = choice(&r).unwrap().clone();
    assert_eq!(previous.declaration().subject(), &ids[1]);
    let examination = revision(7);
    for (wanted, count) in [(Some(&ids[0]), 1), (Some(&ids[2]), 2), (None, 3)] {
        let observation = HtmlIconExaminationObservation::new(
            previous.declaration().subject().clone(),
            previous.resource().clone(),
            HtmlIconExaminationId::fresh(),
            examination.clone(),
            HtmlIconCloserExamination::AttemptedThenInappropriate,
        );
        let input =
            HtmlNextDocumentIconInput::new(previous.clone(), &examination, observation, limits())
                .unwrap();
        let r = session(&c, &cx, 100)
            .query(&[HtmlSemanticRequest::next_document_icon(input)])
            .unwrap();
        assert_eq!(choice(&r).map(|x| x.declaration().subject()), wanted);
        assert_eq!(previous.rejected_count(), count - 1); // immutable earlier result
        if let Some(next) = choice(&r) {
            assert_eq!(next.rejected_count(), count);
            previous = next.clone();
        }
    }
}
#[test]
fn best_group_beats_later_declarations_and_reversed_observation_order_is_qualified() {
    let (c, cx, d, ids) = three_icons();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::new(d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    let expected = expected(None);
    let mut obs = observations(
        collection(&r),
        &expected,
        &[HtmlIconSuitability::Suitable; 3],
    );
    obs.reverse();
    let first_resource = obs[2].resource().unwrap().clone();
    let input = HtmlDocumentIconInput::new(
        &c,
        &cx,
        collection(&r).clone(),
        expected,
        obs,
        HtmlIconOrdering::Known(vec![
            vec![ids[0].clone()],
            vec![ids[1].clone(), ids[2].clone()],
        ]),
        limits(),
    )
    .unwrap();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::document_icon(input)])
        .unwrap();
    assert_eq!(choice(&r).unwrap().declaration().subject(), &ids[0]);
    assert_eq!(choice(&r).unwrap().resource(), &first_resource);
}
#[test]
fn unknown_suitability_and_order_remain_missing_while_complete_unsuitable_is_none() {
    let (c, cx, d, ids) = three_icons();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::new(
            d.clone(),
            HtmlSemanticQuery::DocumentIcon,
        )])
        .unwrap();
    for states in [
        [HtmlIconSuitability::Suitable; 3],
        [HtmlIconSuitability::Unavailable; 3],
        [HtmlIconSuitability::Unsuitable(HtmlIconUnsuitableReason::UnusableResource); 3],
    ] {
        let expected = expected(None);
        let obs = observations(collection(&r), &expected, &states);
        let input = HtmlDocumentIconInput::new(
            &c,
            &cx,
            collection(&r).clone(),
            expected,
            obs,
            HtmlIconOrdering::Unavailable,
            limits(),
        )
        .unwrap();
        let r = session(&c, &cx, 100)
            .query(&[HtmlSemanticRequest::document_icon(input)])
            .unwrap();
        let expected = if states[0] == HtmlIconSuitability::Suitable {
            HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
                subject: d.clone(),
                fact: HtmlRelMissingFact::IconOrdering,
            })
        } else if states[0] == HtmlIconSuitability::Unavailable {
            HtmlSemanticValue::Missing(HtmlSemanticMissing::LinkRelationship {
                subject: ids[0].clone(),
                fact: HtmlRelMissingFact::IconSuitability,
            })
        } else {
            HtmlSemanticValue::Known(None)
        };
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::DocumentIcon(HtmlSemanticValue::Known(
                HtmlDocumentIconReadback::Selection(expected)
            ))
        );
    }
}

#[test]
fn media_missing_and_unsupported_are_conditional_and_local_empty_ascii_whitespace_need_no_css_grammar()
 {
    for (raw, match_outcome) in [
        ("", None),
        (" \t\n\u{c}\r", None),
        ("all", None),
        ("all", Some(HtmlMediaMatchOutcome::Matches)),
        ("(width > 0px)", None),
        ("(width > 0px)", Some(HtmlMediaMatchOutcome::Matches)),
        (
            "(width > 0px)",
            Some(HtmlMediaMatchOutcome::Unsupported(
                HtmlMediaExternalCapability::EnvironmentFeature,
            )),
        ),
    ] {
        let (mut b, d) = builder();
        let a = link(&mut b, &d, &[("rel", "icon"), ("media", raw)]);
        children(&mut b, &a, &[]);
        children(&mut b, &d, std::slice::from_ref(&a));
        let c = b.finish().unwrap();
        let doc = c.subject_id(&d).unwrap();
        let subject = c.subject_id(&a).unwrap();
        let media = HtmlMediaContextRevision::new(
            HtmlMediaGrammarRevision::new(HtmlMediaProducerId::fresh(), 1, 2),
            HtmlMediaEnvironmentRevision::new(HtmlMediaProducerId::fresh(), 3, 4),
        );
        let facts = match_outcome.map(|outcome| {
            HtmlMediaFacts::new(
                &c,
                c.host_facts_revision().clone(),
                media.clone(),
                vec![HtmlMediaFact::new(
                    HtmlMediaAttributeInput::new(&c, subject.clone(), 1).unwrap(),
                    None,
                    Some(HtmlMediaMatchAttestation::new(media.clone(), outcome)),
                )],
                HtmlMediaFactsLimits::new(10, 10000, 100000),
            )
            .unwrap()
        });
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_media(media.clone(), facts),
            0,
        )
        .unwrap();
        let r = session(&c, &cx, 100)
            .query(&[HtmlSemanticRequest::new(
                doc.clone(),
                HtmlSemanticQuery::DocumentIcon,
            )])
            .unwrap();
        let declarations = collection(&r);
        assert!(matches!(
            declarations.candidates()[0].media_rules().grammar(),
            HtmlMediaGrammarEvidence::Missing(_)
        ));
        let expected = expected(Some(media));
        let obs = observations(declarations, &expected, &[HtmlIconSuitability::Suitable]);
        let input = HtmlDocumentIconInput::new(
            &c,
            &cx,
            declarations.clone(),
            expected,
            obs,
            HtmlIconOrdering::Known(vec![vec![subject.clone()]]),
            limits(),
        )
        .unwrap();
        let r = session(&c, &cx, 100)
            .query(&[HtmlSemanticRequest::document_icon(input)])
            .unwrap();
        if raw.is_empty()
            || raw == " \t\n\u{c}\r"
            || match_outcome == Some(HtmlMediaMatchOutcome::Matches)
        {
            assert_eq!(choice(&r).unwrap().declaration().subject(), &subject);
        } else {
            let expected = if match_outcome.is_none() {
                HtmlSemanticValue::Missing(HtmlSemanticMissing::MediaEnvironment(subject))
            } else {
                HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::IconMedia(
                    HtmlMediaExternalCapability::EnvironmentFeature,
                ))
            };
            assert_eq!(
                r.results()[0].fact(),
                &HtmlSemanticFact::DocumentIcon(HtmlSemanticValue::Known(
                    HtmlDocumentIconReadback::Selection(expected)
                ))
            );
        }
    }
}
#[test]
fn suitable_cannot_override_qualified_media_mismatch_and_decisive_format_rejection_can_ignore_missing_media()
 {
    for matches in [Some(HtmlMediaMatchOutcome::DoesNotMatch), None] {
        let (mut b, d) = builder();
        let a = link(&mut b, &d, &[("rel", "icon"), ("media", "screen")]);
        children(&mut b, &a, &[]);
        children(&mut b, &d, std::slice::from_ref(&a));
        let c = b.finish().unwrap();
        let subject = c.subject_id(&a).unwrap();
        let media = HtmlMediaContextRevision::new(
            HtmlMediaGrammarRevision::new(HtmlMediaProducerId::fresh(), 1, 2),
            HtmlMediaEnvironmentRevision::new(HtmlMediaProducerId::fresh(), 3, 4),
        );
        let facts = matches.map(|outcome| {
            HtmlMediaFacts::new(
                &c,
                c.host_facts_revision().clone(),
                media.clone(),
                vec![HtmlMediaFact::new(
                    HtmlMediaAttributeInput::new(&c, subject.clone(), 1).unwrap(),
                    None,
                    Some(HtmlMediaMatchAttestation::new(media.clone(), outcome)),
                )],
                HtmlMediaFactsLimits::new(10, 10000, 100000),
            )
            .unwrap()
        });
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_media(media.clone(), facts),
            0,
        )
        .unwrap();
        let r = session(&c, &cx, 100)
            .query(&[request(&c, &d, HtmlSemanticQuery::DocumentIcon)])
            .unwrap();
        let expected = expected(Some(media));
        if matches.is_some() {
            let obs = observations(collection(&r), &expected, &[HtmlIconSuitability::Suitable]);
            assert_eq!(
                HtmlDocumentIconInput::new(
                    &c,
                    &cx,
                    collection(&r).clone(),
                    expected.clone(),
                    obs,
                    HtmlIconOrdering::Known(vec![vec![subject.clone()]]),
                    limits()
                )
                .unwrap_err(),
                HtmlRelFactsError::IconMediaContradiction
            );
        }
        let obs = observations(
            collection(&r),
            &expected,
            &[HtmlIconSuitability::Unsuitable(
                HtmlIconUnsuitableReason::UnsupportedType,
            )],
        );
        let input = HtmlDocumentIconInput::new(
            &c,
            &cx,
            collection(&r).clone(),
            expected,
            obs,
            HtmlIconOrdering::Known(vec![]),
            limits(),
        )
        .unwrap();
        let r = session(&c, &cx, 100)
            .query(&[HtmlSemanticRequest::document_icon(input)])
            .unwrap();
        assert!(choice(&r).is_none());
    }
}
#[test]
fn observations_groups_and_retained_capacity_are_admitted_against_actual_collection_and_expected_revisions()
 {
    let (c, cx, d, ids) = three_icons();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::new(
            d.clone(),
            HtmlSemanticQuery::DocumentIcon,
        )])
        .unwrap();
    let declarations = collection(&r);
    let expected = expected(None);
    for groups in [
        vec![],
        vec![vec![]],
        vec![vec![
            ids[0].clone(),
            ids[0].clone(),
            ids[1].clone(),
            ids[2].clone(),
        ]],
        vec![vec![d.clone()]],
    ] {
        let obs = observations(declarations, &expected, &[HtmlIconSuitability::Suitable; 3]);
        assert_eq!(
            HtmlDocumentIconInput::new(
                &c,
                &cx,
                declarations.clone(),
                expected.clone(),
                obs,
                HtmlIconOrdering::Known(groups),
                limits()
            )
            .unwrap_err(),
            HtmlRelFactsError::IconOrderingInvalid
        );
    }
    let mut obs = Vec::with_capacity(1000);
    obs.extend(observations(
        declarations,
        &expected,
        &[HtmlIconSuitability::Suitable; 3],
    ));
    assert_eq!(
        HtmlDocumentIconInput::new(
            &c,
            &cx,
            declarations.clone(),
            expected.clone(),
            obs,
            HtmlIconOrdering::Unavailable,
            HtmlRelFactsLimits::new(100, 2_000_000, 2_000_000)
        )
        .unwrap_err(),
        HtmlRelFactsError::RecordLimit
    );
    assert_eq!(
        HtmlDocumentIconInput::new(
            &c,
            &cx,
            declarations.clone(),
            expected.clone(),
            vec![],
            HtmlIconOrdering::Unavailable,
            limits()
        )
        .unwrap_err(),
        HtmlRelFactsError::IconObservationCoverage
    );
    let wrong = super_expected_resource(&expected);
    let obs = observations(declarations, &wrong, &[HtmlIconSuitability::Suitable; 3]);
    assert_eq!(
        HtmlDocumentIconInput::new(
            &c,
            &cx,
            declarations.clone(),
            expected.clone(),
            obs,
            HtmlIconOrdering::Unavailable,
            limits()
        )
        .unwrap_err(),
        HtmlRelFactsError::SelectorBindingMismatch
    );
    let another = context(&c);
    let other = session(&c, &another, 100)
        .query(&[HtmlSemanticRequest::new(d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    let obs = observations(
        collection(&other),
        &expected,
        &[HtmlIconSuitability::Suitable; 3],
    );
    assert_eq!(
        HtmlDocumentIconInput::new(
            &c,
            &cx,
            declarations.clone(),
            expected,
            obs,
            HtmlIconOrdering::Unavailable,
            limits()
        )
        .unwrap_err(),
        HtmlRelFactsError::SelectorBindingMismatch
    );
}
fn super_expected_resource(e: &HtmlIconExpectedRevisions) -> HtmlIconExpectedRevisions {
    HtmlIconExpectedRevisions::new(
        e.presentation().clone(),
        e.policy().clone(),
        revision(9),
        e.media().cloned(),
    )
}
#[test]
fn rejection_requires_actual_previous_choice_resource_revision_and_attempted_use() {
    let (c, cx, d, ids) = three_icons();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::new(d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    let expected = expected(None);
    let obs = observations(
        collection(&r),
        &expected,
        &[HtmlIconSuitability::Suitable; 3],
    );
    let input = HtmlDocumentIconInput::new(
        &c,
        &cx,
        collection(&r).clone(),
        expected,
        obs,
        HtmlIconOrdering::Known(vec![ids.clone()]),
        limits(),
    )
    .unwrap();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::document_icon(input)])
        .unwrap();
    let previous = choice(&r).unwrap();
    let exam = revision(10);
    for (subject, resource, revision, outcome, error) in [
        (
            ids[0].clone(),
            previous.resource().clone(),
            exam.clone(),
            HtmlIconCloserExamination::AttemptedThenInappropriate,
            HtmlRelFactsError::SelectorBindingMismatch,
        ),
        (
            ids[2].clone(),
            HtmlIconResourceKey::new(
                HtmlIconResourceId::fresh(),
                previous.resource().revision().clone(),
            ),
            exam.clone(),
            HtmlIconCloserExamination::AttemptedThenInappropriate,
            HtmlRelFactsError::SelectorBindingMismatch,
        ),
        (
            ids[2].clone(),
            previous.resource().clone(),
            revision(11),
            HtmlIconCloserExamination::AttemptedThenInappropriate,
            HtmlRelFactsError::SelectorBindingMismatch,
        ),
        (
            ids[2].clone(),
            previous.resource().clone(),
            exam.clone(),
            HtmlIconCloserExamination::NotAttempted,
            HtmlRelFactsError::IconNotAttempted,
        ),
    ] {
        let observation = HtmlIconExaminationObservation::new(
            subject,
            resource,
            HtmlIconExaminationId::fresh(),
            revision,
            outcome,
        );
        assert_eq!(
            HtmlNextDocumentIconInput::new(previous.clone(), &exam, observation, limits())
                .unwrap_err(),
            error
        );
    }
    let observation = HtmlIconExaminationObservation::new(
        ids[2].clone(),
        previous.resource().clone(),
        HtmlIconExaminationId::fresh(),
        exam.clone(),
        HtmlIconCloserExamination::Unavailable,
    );
    let input =
        HtmlNextDocumentIconInput::new(previous.clone(), &exam, observation, limits()).unwrap();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::next_document_icon(input)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::NextDocumentIcon(HtmlSemanticValue::Known(
            HtmlDocumentIconReadback::Selection(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::LinkRelationship {
                    subject: ids[2].clone(),
                    fact: HtmlRelMissingFact::IconCloserExamination
                }
            ))
        ))
    );
    assert_eq!(previous.rejected_count(), 0);
    assert_eq!(previous.declaration().subject(), &ids[2]);
}
#[test]
fn selected_raw_slots_are_admitted_before_namespace_role_and_membership_shortcuts() {
    for ns in [Namespace::html(), Namespace::svg()] {
        for tag in ["link", "div"] {
            for (query, field) in [
                (HtmlSemanticQuery::DocumentFeeds, "type"),
                (HtmlSemanticQuery::FirstManifest, "rel"),
                (HtmlSemanticQuery::DocumentIcon, "sizes"),
            ] {
                let (mut b, d) = builder();
                let a = element(&mut b, &d, ns.clone(), tag, &[(field, "x")]);
                children(&mut b, &a, &[]);
                children(&mut b, &d, std::slice::from_ref(&a));
                let c = b.finish().unwrap();
                let cx = context(&c);
                let mut s = session(&c, &cx, 0);
                let error = s.query(&[request(&c, &d, query)]).unwrap_err();
                assert!(matches!(
                    error.kind(),
                    HtmlSemanticFailureKind::Attribute(_)
                ));
                assert_eq!(error.usage().scratch_records(), 0);
                assert_eq!(s.usage().scratch_records(), 0);
                let repeated = s.query(&[]).unwrap_err();
                assert_eq!(error, repeated);
                assert_eq!(error.qualification().unwrap().capture(), c.identity());
            }
        }
    }
}
#[test]
fn source_work_failure_discards_batch_and_preserves_capture_and_sticky_qualification() {
    let (c, cx, d, _) = three_icons();
    let request = HtmlSemanticRequest::new(d.clone(), HtmlSemanticQuery::DocumentIcon);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        HtmlSemanticLimits::new(75, 100, 10000, 10000, 2_000_000, 100000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let error = s.query(&[request]).unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(error.usage().scratch_records(), 0);
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), error);
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::new(d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    assert_eq!(collection(&r).candidates().len(), 3);
}

#[test]
fn hosted_template_and_shadow_candidates_are_outside_ordinary_document_discovery() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let host = element(
            &mut b,
            &d,
            Namespace::html(),
            if shadow { "div" } else { "template" },
            &[],
        );
        let subtree = if shadow {
            b.add_shadow_root(
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
            .unwrap()
        } else {
            let f = b.add_fragment(&d).unwrap();
            b.establish_template_contents(&host, &f).unwrap();
            f
        };
        let inner = link(
            &mut b,
            &d,
            &[
                ("rel", "alternate manifest icon"),
                ("type", "application/rss+xml"),
            ],
        );
        children(&mut b, &inner, &[]);
        children(&mut b, &subtree, std::slice::from_ref(&inner));
        children(&mut b, &host, &[]);
        children(&mut b, &d, std::slice::from_ref(&host));
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = session(&c, &cx, 100)
            .query(&[
                request(&c, &d, HtmlSemanticQuery::DocumentFeeds),
                request(&c, &d, HtmlSemanticQuery::FirstManifest),
                request(&c, &d, HtmlSemanticQuery::DocumentIcon),
            ])
            .unwrap();
        assert_eq!(feeds(&r), []);
        assert_eq!(
            r.results()[1].fact(),
            &HtmlSemanticFact::FirstManifest(HtmlSemanticValue::Known(None))
        );
        let HtmlSemanticFact::DocumentIcon(HtmlSemanticValue::Known(
            HtmlDocumentIconReadback::Declarations {
                collection,
                selection,
            },
        )) = r.results()[2].fact()
        else {
            panic!()
        };
        assert!(collection.candidates().is_empty());
        assert_eq!(selection, &HtmlSemanticValue::Known(None));
    }
}
#[test]
fn registered_canonical_aliases_join_selector_membership_and_unavailable_aliases_block_complete_collection()
 {
    for qualified in [true, false] {
        let (mut b, d) = builder();
        let a = link(
            &mut b,
            &d,
            &[("rel", "discovered"), ("type", "application/rss+xml")],
        );
        children(&mut b, &a, &[]);
        children(&mut b, &d, std::slice::from_ref(&a));
        let c = b.finish().unwrap();
        let subject = c.subject_id(&a).unwrap();
        let rev = revision(12);
        let key = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 1);
        let definition = HtmlRelDefinition::standard(
            key.clone(),
            HtmlRelKeyword::Alternate,
            HtmlRelRegistryStatus::Ratified,
        );
        let record = if qualified {
            HtmlRelRegistryOccurrence::synonym(subject.clone(), "discovered".into(), 0..10, key)
        } else {
            HtmlRelRegistryOccurrence::unavailable(subject.clone(), "discovered".into(), 0..10)
        };
        let facts = HtmlRelFacts::new(
            &c,
            c.host_facts_revision(),
            HtmlRelFactsInput::new().with_registry(rev.clone(), vec![definition], vec![record]),
            limits(),
        )
        .unwrap();
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_link_relationships(
                HtmlRelExpectedRevisions::new().with_registry(rev),
                Some(facts),
            ),
            0,
        )
        .unwrap();
        let r = session(&c, &cx, 100)
            .query(&[request(&c, &d, HtmlSemanticQuery::DocumentFeeds)])
            .unwrap();
        if qualified {
            assert_eq!(feeds(&r), [subject]);
        } else {
            assert_eq!(
                r.results()[0].fact(),
                &HtmlSemanticFact::DocumentFeeds(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::LinkRelationship {
                        subject,
                        fact: HtmlRelMissingFact::Registry
                    }
                ))
            );
        }
    }
}
#[test]
fn collection_requires_all_icon_membership_and_child_coverage_and_ignores_author_invalid_sizes() {
    let (mut b, d) = builder();
    let a = link(&mut b, &d, &[("rel", "icon"), ("sizes", "01x1")]);
    let z = link(&mut b, &d, &[("rel", "unknown")]);
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &d, &[a, z.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx, 100)
        .query(&[request(&c, &d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::DocumentIcon(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::LinkRelationship {
                subject: c.subject_id(&z).unwrap(),
                fact: HtmlRelMissingFact::Registry
            }
        ))
    );
    let (mut b, d) = builder();
    let a = link(&mut b, &d, &[("rel", "icon"), ("sizes", "01x1")]);
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx, 100)
        .query(&[request(&c, &d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    let expected = expected(None);
    let obs = observations(collection(&r), &expected, &[HtmlIconSuitability::Suitable]);
    let input = HtmlDocumentIconInput::new(
        &c,
        &cx,
        collection(&r).clone(),
        expected,
        obs,
        HtmlIconOrdering::Known(vec![vec![c.subject_id(&a).unwrap()]]),
        limits(),
    )
    .unwrap();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::document_icon(input)])
        .unwrap();
    assert_eq!(
        choice(&r).unwrap().declaration().subject(),
        &c.subject_id(&a).unwrap()
    );
}
#[test]
fn changed_context_refuses_formed_choice_and_raw_session_limit_precedes_selection() {
    let (c, cx, d, ids) = three_icons();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::new(d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    let expected = expected(None);
    let obs = observations(
        collection(&r),
        &expected,
        &[HtmlIconSuitability::Suitable; 3],
    );
    let input = HtmlDocumentIconInput::new(
        &c,
        &cx,
        collection(&r).clone(),
        expected,
        obs,
        HtmlIconOrdering::Known(vec![ids]),
        limits(),
    )
    .unwrap();
    let other = context(&c);
    let error = session(&c, &other, 100)
        .query(&[HtmlSemanticRequest::document_icon(input.clone())])
        .unwrap_err();
    assert_eq!(
        error.kind(),
        &HtmlSemanticFailureKind::RelationshipSelectorRequestMismatch
    );
    let error = session(&c, &cx, 0)
        .query(&[HtmlSemanticRequest::document_icon(input.clone())])
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::document_icon(input)])
        .unwrap();
    assert!(choice(&r).is_some());
}

#[test]
fn formation_checks_work_bytes_group_spare_capacity_and_next_transitive_records_before_retention() {
    let (c, cx, d, ids) = three_icons();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::new(d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    let expected = expected(None);
    for (limits, error) in [
        (
            HtmlRelFactsLimits::new(10000, 0, 2_000_000),
            HtmlRelFactsError::ByteLimit,
        ),
        (
            HtmlRelFactsLimits::new(10000, 2_000_000, 0),
            HtmlRelFactsError::WorkLimit,
        ),
    ] {
        let obs = observations(
            collection(&r),
            &expected,
            &[HtmlIconSuitability::Suitable; 3],
        );
        assert_eq!(
            HtmlDocumentIconInput::new(
                &c,
                &cx,
                collection(&r).clone(),
                expected.clone(),
                obs,
                HtmlIconOrdering::Known(vec![ids.clone()]),
                limits
            )
            .unwrap_err(),
            error
        );
    }
    let mut group = Vec::with_capacity(1000);
    group.extend(ids.clone());
    let obs = observations(
        collection(&r),
        &expected,
        &[HtmlIconSuitability::Suitable; 3],
    );
    assert_eq!(
        HtmlDocumentIconInput::new(
            &c,
            &cx,
            collection(&r).clone(),
            expected.clone(),
            obs,
            HtmlIconOrdering::Known(vec![group]),
            HtmlRelFactsLimits::new(100, 2_000_000, 2_000_000)
        )
        .unwrap_err(),
        HtmlRelFactsError::RecordLimit
    );
    let obs = observations(
        collection(&r),
        &expected,
        &[HtmlIconSuitability::Suitable; 3],
    );
    let input = HtmlDocumentIconInput::new(
        &c,
        &cx,
        collection(&r).clone(),
        expected,
        obs,
        HtmlIconOrdering::Known(vec![ids]),
        limits(),
    )
    .unwrap();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::document_icon(input)])
        .unwrap();
    let prior = choice(&r).unwrap();
    let exam = revision(10);
    let observation = HtmlIconExaminationObservation::new(
        prior.declaration().subject().clone(),
        prior.resource().clone(),
        HtmlIconExaminationId::fresh(),
        exam.clone(),
        HtmlIconCloserExamination::AttemptedThenInappropriate,
    );
    assert_eq!(
        HtmlNextDocumentIconInput::new(
            prior.clone(),
            &exam,
            observation,
            HtmlRelFactsLimits::new(1, 2_000_000, 2_000_000)
        )
        .unwrap_err(),
        HtmlRelFactsError::RecordLimit
    );
}
#[test]
fn sizes_author_preserves_invalid_before_duplicate_and_does_not_remove_original_occurrences() {
    for (raw, expected) in [
        (
            "ANY any",
            HtmlIconSizesAuthor::Duplicate {
                first_index: 0,
                duplicate_index: 1,
            },
        ),
        (
            "ANY any 01x1",
            HtmlIconSizesAuthor::Invalid {
                token_index: 2,
                reason: HtmlIconSizeError::LeadingZero,
            },
        ),
        (
            "16X16 16x16",
            HtmlIconSizesAuthor::Duplicate {
                first_index: 0,
                duplicate_index: 1,
            },
        ),
    ] {
        let (mut b, d) = builder();
        let a = link(&mut b, &d, &[("rel", "icon"), ("sizes", raw)]);
        children(&mut b, &a, &[]);
        children(&mut b, &d, std::slice::from_ref(&a));
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = session(&c, &cx, 100)
            .query(&[request(&c, &d, HtmlSemanticQuery::DocumentIcon)])
            .unwrap();
        let a = &collection(&r).candidates()[0];
        assert_eq!(a.sizes_author(), &expected);
        assert_eq!(a.size_occurrences()[1].duplicate_of(), Some(0));
        assert_eq!(
            a.size_occurrences().len(),
            if raw == "ANY any 01x1" { 3 } else { 2 }
        );
    }
}
#[test]
fn cancellation_and_nested_output_failure_are_atomic_and_retire_all_traversal_storage() {
    let (c, cx, d, _) = three_icons();
    let cancellation = HtmlSemanticCancellation::new();
    cancellation.cancel();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        HtmlSemanticLimits::new(2_000_000, 100, 10000, 10000, 2_000_000, 100000),
        cancellation,
    )
    .unwrap();
    let error = s
        .query(&[HtmlSemanticRequest::new(
            d.clone(),
            HtmlSemanticQuery::DocumentIcon,
        )])
        .unwrap_err();
    assert_eq!(error.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(error.usage().scratch_records(), 0);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        HtmlSemanticLimits::new(2_000_000, 100, 10000, 10000, 2_000_000, 0),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let error = s
        .query(&[
            HtmlSemanticRequest::new(d.clone(), HtmlSemanticQuery::FirstManifest),
            HtmlSemanticRequest::new(d, HtmlSemanticQuery::DocumentIcon),
        ])
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(error.usage().scratch_records(), 0);
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), error);
}

#[test]
fn every_document_selector_is_inapplicable_to_element_and_fragment_subjects() {
    let (mut b, d) = builder();
    let e = link(&mut b, &d, &[("rel", "icon manifest alternate")]);
    let f = b.add_fragment(&d).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    for subject in [&e, &f] {
        let queries = [
            HtmlSemanticQuery::DocumentFeeds,
            HtmlSemanticQuery::DefaultFeed,
            HtmlSemanticQuery::FirstManifest,
            HtmlSemanticQuery::DocumentIcon,
            HtmlSemanticQuery::NextDocumentIcon,
        ];
        for query in queries {
            let r = session(&c, &cx, 100)
                .query(&[request(&c, subject, query)])
                .unwrap();
            assert!(
                matches!(
                    r.results()[0].fact(),
                    HtmlSemanticFact::DocumentFeeds(HtmlSemanticValue::Inapplicable)
                        | HtmlSemanticFact::DefaultFeed(HtmlSemanticValue::Inapplicable)
                        | HtmlSemanticFact::FirstManifest(HtmlSemanticValue::Inapplicable)
                        | HtmlSemanticFact::DocumentIcon(HtmlSemanticValue::Inapplicable)
                        | HtmlSemanticFact::NextDocumentIcon(HtmlSemanticValue::Inapplicable)
                ),
                "{query:?}: {:?}",
                r.results()[0].fact()
            );
        }
    }
}

#[test]
fn canonical_media_mismatch_independently_excludes_unobserved_icon_suitability() {
    let (mut b, d) = builder();
    let a = link(&mut b, &d, &[("rel", "icon"), ("media", "screen")]);
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let subject = c.subject_id(&a).unwrap();
    let media = HtmlMediaContextRevision::new(
        HtmlMediaGrammarRevision::new(HtmlMediaProducerId::fresh(), 1, 2),
        HtmlMediaEnvironmentRevision::new(HtmlMediaProducerId::fresh(), 3, 4),
    );
    let facts = HtmlMediaFacts::new(
        &c,
        c.host_facts_revision().clone(),
        media.clone(),
        vec![HtmlMediaFact::new(
            HtmlMediaAttributeInput::new(&c, subject.clone(), 1).unwrap(),
            None,
            Some(HtmlMediaMatchAttestation::new(
                media.clone(),
                HtmlMediaMatchOutcome::DoesNotMatch,
            )),
        )],
        HtmlMediaFactsLimits::new(10, 10000, 100000),
    )
    .unwrap();
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_media(media.clone(), Some(facts)),
        0,
    )
    .unwrap();
    let r = session(&c, &cx, 100)
        .query(&[request(&c, &d, HtmlSemanticQuery::DocumentIcon)])
        .unwrap();
    let expected = expected(Some(media));
    let obs = observations(
        collection(&r),
        &expected,
        &[HtmlIconSuitability::Unavailable],
    );
    let input = HtmlDocumentIconInput::new(
        &c,
        &cx,
        collection(&r).clone(),
        expected,
        obs,
        HtmlIconOrdering::Unavailable,
        limits(),
    )
    .unwrap();
    let r = session(&c, &cx, 100)
        .query(&[HtmlSemanticRequest::document_icon(input)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::DocumentIcon(HtmlSemanticValue::Known(
            HtmlDocumentIconReadback::Selection(HtmlSemanticValue::Known(None))
        ))
    );
}
