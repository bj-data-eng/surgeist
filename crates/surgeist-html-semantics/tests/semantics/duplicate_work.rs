//! Charged work of the producer fact constructors that refuse duplicates.
//!
//! A constructor that compared each fact with every earlier one took work
//! quadratic in the facts. Each test builds `n` facts that are all distinct and
//! valid, finds the constructor's total work as the least admitting limit, and
//! requires that doubling `n` at most multiplies the total by 2.2 (a scan of the
//! earlier facts multiplies it by about 4 once it outweighs the work each fact
//! needs anyway). Results do not change; the work is the whole difference.

use super::constructor_work::{Run, attributed, big_builder, least_work_unreported, rational};
use crate::semantics::support::assert_doubling_at_most_2_2;
use surgeist_html::{
    construction::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment},
    elements::link::HtmlRelKeyword,
    microsyntax::tokens::HtmlSpaceToken,
};
use surgeist_html_semantics::{
    HtmlAboutBaseUrl, HtmlBaseDocumentFacts, HtmlBaseEncoding, HtmlBaseFactsError,
    HtmlBaseFactsLimits, HtmlBaseFactsRevision, HtmlBaseParseObservation, HtmlBaseParseOutcome,
    HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme, HtmlCapture, HtmlCaptureSubjectId,
    HtmlCapturedBaseDocument, HtmlCapturedBaseFacts, HtmlCapturedFrozenBase,
    HtmlCapturedShadowProperties, HtmlEncodingUrlFactsError, HtmlEncodingUrlParseFacts,
    HtmlEncodingUrlParseObservation, HtmlEncodingUrlParseOutcome, HtmlImageContentBoxObservation,
    HtmlImageDimensionCompleted, HtmlImageDimensionExpectation, HtmlImageDimensionFacts,
    HtmlImageDimensionFactsError, HtmlImageDimensionFactsLimits, HtmlImageDimensionMeasurement,
    HtmlImageDimensionOperationKey, HtmlImageDimensionResourceObservation,
    HtmlImageDimensionRevision, HtmlImageDimensionRole, HtmlImageDimensionSelection,
    HtmlImageSizingRevision, HtmlImgRequestState, HtmlLanguageDirectionError,
    HtmlLanguageDirectionFacts, HtmlLanguageDirectionLimits, HtmlListParticipation,
    HtmlListParticipationFact, HtmlListParticipationRevision, HtmlMediaAttributeInput,
    HtmlMediaContextRevision, HtmlMediaEnvironmentRevision, HtmlMediaFact, HtmlMediaFacts,
    HtmlMediaFactsError, HtmlMediaFactsLimits, HtmlMediaGrammarAttestation,
    HtmlMediaGrammarOutcome, HtmlMediaGrammarRevision, HtmlMediaMatchAttestation,
    HtmlMediaMatchOutcome, HtmlMediaProducerId, HtmlNaturalDimensionAxis, HtmlNaturalDimensionFact,
    HtmlNaturalDimensionFactsError, HtmlNaturalDimensionLimits, HtmlNaturalDimensionProducerId,
    HtmlNaturalDimensionRevision, HtmlNaturalDimensions, HtmlNaturalResourceId,
    HtmlNaturalResourceKey, HtmlParseBaseFacts, HtmlParticipationProducerId, HtmlRelBlobEntry,
    HtmlRelBookmarkSection, HtmlRelDefinition, HtmlRelDefinitionId, HtmlRelDefinitionKey,
    HtmlRelExtensionEffect, HtmlRelFacts, HtmlRelFactsError, HtmlRelFactsInput, HtmlRelFactsLimits,
    HtmlRelGrammarInput, HtmlRelOriginId, HtmlRelProducerId, HtmlRelRegistryOccurrence,
    HtmlRelRegistryStatus, HtmlRelRevision, HtmlRelSameSiteObservation, HtmlRelSectionId,
    HtmlRelSettingsId, HtmlRelSupportedTokens, HtmlRelUrlGrammar, HtmlRelUrlId, HtmlRelUrlKey,
    HtmlRelUrlObservation, HtmlSemanticContext, HtmlSemanticContextError, HtmlSemanticContextInput,
    HtmlSlotAssignedNodes,
};

const DOUBLINGS: [usize; 4] = [100, 200, 400, 800];

/// A document of `count` elements named `name` with the given attributes.
fn page(
    name: &str,
    attributes: &[(&str, &str)],
    count: usize,
) -> (HtmlCapture, Vec<HtmlCaptureSubjectId>) {
    let (mut b, d) = big_builder(count);
    let elements: Vec<_> = (0..count)
        .map(|_| attributed(&mut b, &d, name, attributes))
        .collect();
    for element in &elements {
        b.establish_children(element, &[]).unwrap();
    }
    b.establish_children(&d, &elements).unwrap();
    let capture = b.finish().unwrap();
    let ids = elements
        .iter()
        .map(|element| capture.subject_id(element).unwrap())
        .collect();
    (capture, ids)
}

fn media_total(count: usize) -> usize {
    let (capture, ids) = page("link", &[("rel", "icon"), ("media", "screen")], count);
    let media = HtmlMediaContextRevision::new(
        HtmlMediaGrammarRevision::new(HtmlMediaProducerId::fresh(), 1, 2),
        HtmlMediaEnvironmentRevision::new(HtmlMediaProducerId::fresh(), 3, 4),
    );
    least_work_unreported("media", |work| {
        let facts = ids
            .iter()
            .map(|id| {
                HtmlMediaFact::new(
                    HtmlMediaAttributeInput::new(&capture, id.clone(), 1).unwrap(),
                    Some(HtmlMediaGrammarAttestation::new(
                        media.grammar().clone(),
                        HtmlMediaGrammarOutcome::Valid,
                    )),
                    Some(HtmlMediaMatchAttestation::new(
                        media.clone(),
                        HtmlMediaMatchOutcome::Matches,
                    )),
                )
            })
            .collect();
        match HtmlMediaFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            media.clone(),
            facts,
            HtmlMediaFactsLimits::new(8 * count, 1 << 30, work),
        ) {
            Ok(_) => Run::Admitted,
            Err(HtmlMediaFactsError::WorkLimit) => Run::Refused { required: 0 },
            Err(other) => panic!("fixture refused for another reason: {other:?}"),
        }
    })
}

#[test]
fn media_facts_check_duplicates_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("media facts", &DOUBLINGS, media_total);
}

fn natural_total(count: usize) -> usize {
    let (capture, ids) = page("img", &[], count);
    let revision =
        HtmlNaturalDimensionRevision::new(HtmlNaturalDimensionProducerId::fresh(), 7, 17);
    least_work_unreported("natural dimensions", |work| {
        let keys: Vec<_> = ids
            .iter()
            .map(|_| HtmlNaturalResourceKey::new(HtmlNaturalResourceId::fresh(), 11, 13))
            .collect();
        let expected = ids
            .iter()
            .zip(&keys)
            .map(|(id, key)| HtmlNaturalDimensionFact::selection(id.clone(), Some(key.clone())))
            .collect();
        // Observed in the reverse order, so that finding an expected subject's
        // observation by scanning the observations is not accidentally cheap.
        let observed = ids
            .iter()
            .zip(&keys)
            .rev()
            .map(|(id, key)| {
                HtmlNaturalDimensionFact::observation(
                    id.clone(),
                    revision.clone(),
                    key.clone(),
                    HtmlNaturalDimensionAxis::KnownPresent(rational("3", "2")),
                    HtmlNaturalDimensionAxis::KnownPresent(rational("5", "1")),
                )
            })
            .collect();
        match HtmlNaturalDimensions::new(
            &capture,
            capture.host_facts_revision().clone(),
            revision.clone(),
            expected,
            observed,
            HtmlNaturalDimensionLimits::new(16 * count, 1 << 30, work),
        ) {
            Ok(_) => Run::Admitted,
            Err(HtmlNaturalDimensionFactsError::WorkLimit) => Run::Refused { required: 0 },
            Err(other) => panic!("fixture refused for another reason: {other:?}"),
        }
    })
}

#[test]
fn natural_dimensions_join_and_check_duplicates_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("natural dimensions", &DOUBLINGS, natural_total);
}

fn image_total(count: usize) -> usize {
    let (capture, ids) = page("img", &[], count);
    let resource = HtmlNaturalDimensionRevision::new(HtmlNaturalDimensionProducerId::fresh(), 5, 6);
    let participation = HtmlListParticipationRevision::new(
        HtmlParticipationProducerId::fresh(),
        1,
        2,
        HtmlParticipationProducerId::fresh(),
        3,
        4,
    );
    let known = |n: &str, d: &str| HtmlImageDimensionMeasurement::Known(rational(n, d));
    least_work_unreported("image dimensions", |work| {
        let mut expected = Vec::new();
        let mut resources = Vec::new();
        let mut boxes = Vec::new();
        for id in &ids {
            let operation = HtmlImageDimensionOperationKey::ImgPreferredDensityCorrected {
                operation: HtmlImageSizingRevision::new(HtmlParticipationProducerId::fresh(), 7, 8),
                request: 11,
                density: 12,
                preferred: 13,
            };
            let expectation = HtmlImageDimensionExpectation::new(
                id.clone(),
                HtmlImageDimensionRole::Img,
                HtmlImageDimensionSelection::Selected(HtmlNaturalResourceKey::new(
                    HtmlNaturalResourceId::fresh(),
                    14,
                    15,
                )),
                11,
                Some(operation.clone()),
            );
            resources.push(HtmlImageDimensionResourceObservation::img(
                resource.clone(),
                expectation.clone(),
                Some(HtmlImgRequestState::CompletelyAvailable),
                Some(HtmlImageDimensionCompleted::new(
                    operation,
                    known("3", "2"),
                    known("5", "1"),
                )),
            ));
            expected.push(expectation);
            boxes.push(HtmlImageContentBoxObservation::new(
                capture.identity().clone(),
                capture.host_facts_revision().clone(),
                id.clone(),
                participation.clone(),
                known("7", "1"),
                known("9", "2"),
            ));
        }
        // Observed in the reverse order, so that finding an expectation by
        // scanning the expectations is not accidentally cheap.
        resources.reverse();
        match HtmlImageDimensionFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            HtmlImageDimensionRevision::new(Some(resource.clone()), Some(participation.clone())),
            expected,
            resources,
            boxes,
            HtmlImageDimensionFactsLimits::new(16 * count, 1 << 30, work),
        ) {
            Ok(_) => Run::Admitted,
            Err(HtmlImageDimensionFactsError::WorkLimit) => Run::Refused { required: 0 },
            Err(other) => panic!("fixture refused for another reason: {other:?}"),
        }
    })
}

#[test]
fn image_dimension_facts_check_duplicates_and_bindings_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("image dimension facts", &DOUBLINGS, image_total);
}

fn rel_revision() -> HtmlRelRevision {
    HtmlRelRevision::new(HtmlRelProducerId::fresh(), 3, 7)
}
/// The least `validation_work` of the link relationship facts that `build` makes
/// over `count` elements named `name` with the given attributes.
fn rel_total(
    label: &str,
    name: &str,
    attributes: &[(&str, &str)],
    count: usize,
    build: impl Fn(&[HtmlCaptureSubjectId]) -> HtmlRelFactsInput,
) -> usize {
    let (capture, ids) = page(name, attributes, count);
    least_work_unreported(label, |work| {
        match HtmlRelFacts::new(
            &capture,
            capture.host_facts_revision(),
            build(&ids),
            HtmlRelFactsLimits::new(1 << 20, 1 << 30, work),
        ) {
            Ok(_) => Run::Admitted,
            Err(HtmlRelFactsError::WorkLimit) => Run::Refused { required: 0 },
            Err(other) => panic!("fixture refused for another reason: {other:?}"),
        }
    })
}

#[test]
fn rel_registry_occurrences_check_duplicates_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("rel occurrences", &DOUBLINGS, |count| {
        rel_total("rel occurrences", "form", &[("rel", "ext")], count, |ids| {
            let key = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 1);
            let definition = HtmlRelDefinition::extension(
                key.clone(),
                HtmlSpaceToken::new("ext").unwrap(),
                HtmlRelRegistryStatus::Proposed,
                HtmlRelExtensionEffect::NotAllowed,
                HtmlRelExtensionEffect::Hyperlink,
                HtmlRelExtensionEffect::Hyperlink,
            );
            let occurrences = ids
                .iter()
                .map(|id| {
                    HtmlRelRegistryOccurrence::definition(
                        id.clone(),
                        "ext".into(),
                        0..3,
                        key.clone(),
                    )
                })
                .collect();
            HtmlRelFactsInput::new().with_registry(rel_revision(), vec![definition], occurrences)
        })
    });
}

#[test]
fn rel_supported_tokens_check_duplicates_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("rel supported tokens", &DOUBLINGS, |count| {
        rel_total("rel supported", "form", &[("rel", "ext")], count, |ids| {
            let records = ids
                .iter()
                .map(|id| HtmlRelSupportedTokens::new(id.clone(), vec![HtmlRelKeyword::NoOpener]))
                .collect();
            HtmlRelFactsInput::new().with_supported_tokens(rel_revision(), records)
        })
    });
}

#[test]
fn rel_url_grammar_checks_duplicates_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("rel url grammar", &DOUBLINGS, |count| {
        rel_total(
            "rel grammar",
            "a",
            &[("rel", "ext"), ("href", "x")],
            count,
            |ids| {
                let records = ids
                    .iter()
                    .map(|id| {
                        HtmlRelUrlGrammar::new(
                            id.clone(),
                            HtmlRelGrammarInput::Href,
                            "x".into(),
                            true,
                        )
                    })
                    .collect();
                HtmlRelFactsInput::new().with_url_grammar(rel_revision(), records)
            },
        )
    });
}

#[test]
fn rel_bookmark_sections_check_duplicates_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("rel bookmark sections", &DOUBLINGS, |count| {
        rel_total("rel sections", "a", &[("rel", "bookmark")], count, |ids| {
            let records = ids
                .iter()
                .map(|id| HtmlRelBookmarkSection::new(id.clone(), HtmlRelSectionId::fresh()))
                .collect();
            HtmlRelFactsInput::new().with_bookmark_sections(rel_revision(), records)
        })
    });
}

/// One URL observation per form, in the order of the forms or reversed.
fn url_observations(
    ids: &[HtmlCaptureSubjectId],
    revision: &HtmlRelRevision,
    origin: &HtmlRelOriginId,
) -> (Vec<HtmlRelUrlKey>, Vec<HtmlRelUrlObservation>) {
    let keys: Vec<_> = ids
        .iter()
        .map(|_| HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), revision.clone()))
        .collect();
    let urls = ids
        .iter()
        .zip(&keys)
        .map(|(id, key)| {
            HtmlRelUrlObservation::new(
                id.clone(),
                key.clone(),
                HtmlRelBlobEntry::NonNull {
                    environment_origin: origin.clone(),
                },
            )
        })
        .collect();
    (keys, urls)
}

#[test]
fn rel_url_observations_check_duplicates_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("rel url observations", &DOUBLINGS, |count| {
        rel_total("rel urls", "form", &[("rel", "ext")], count, |ids| {
            let revision = rel_revision();
            let (_, urls) = url_observations(ids, &revision, &HtmlRelOriginId::fresh());
            HtmlRelFactsInput::new().with_url_records(revision, urls)
        })
    });
}

#[test]
fn rel_same_site_observations_bind_to_urls_and_check_duplicates_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("rel same-site observations", &DOUBLINGS, |count| {
        rel_total("rel same site", "form", &[("rel", "ext")], count, |ids| {
            let revision = rel_revision();
            let origin = HtmlRelOriginId::fresh();
            let (keys, urls) = url_observations(ids, &revision, &origin);
            // Observed in the reverse order, so that finding an observation's URL
            // by scanning the URLs is not accidentally cheap.
            let settings = HtmlRelSettingsId::fresh();
            let same_site = ids
                .iter()
                .zip(&keys)
                .rev()
                .map(|(id, key)| {
                    HtmlRelSameSiteObservation::new(
                        id.clone(),
                        key.clone(),
                        settings.clone(),
                        origin.clone(),
                        HtmlRelOriginId::fresh(),
                        true,
                    )
                })
                .collect();
            HtmlRelFactsInput::new()
                .with_url_records(revision.clone(), urls)
                .with_settings(revision, same_site)
        })
    });
}

// ---------------------------------------------------------------------------
// The refusals themselves do not change when duplicates are found through an
// index: these hold for the previous pairwise checks as well.

#[test]
fn natural_dimension_facts_report_the_first_extra_observation_in_removal_order() {
    // Joining takes each expected selection's observation out of the list by
    // swapping the last one into its place, so the extras that remain are in
    // that order: A is taken from position 2, which moves E2 there; B from
    // position 0, which moves E2 again, leaving [E2, E1].
    let (capture, ids) = page("img", &[], 4);
    let [a, b, e1, e2] = [&ids[0], &ids[1], &ids[2], &ids[3]];
    let revision =
        HtmlNaturalDimensionRevision::new(HtmlNaturalDimensionProducerId::fresh(), 7, 17);
    let key = || HtmlNaturalResourceKey::new(HtmlNaturalResourceId::fresh(), 11, 13);
    let (key_a, key_b, key_1, key_2) = (key(), key(), key(), key());
    let observation = |id: &HtmlCaptureSubjectId, key: &HtmlNaturalResourceKey| {
        HtmlNaturalDimensionFact::observation(
            id.clone(),
            revision.clone(),
            key.clone(),
            HtmlNaturalDimensionAxis::KnownPresent(rational("3", "2")),
            HtmlNaturalDimensionAxis::KnownPresent(rational("5", "1")),
        )
    };
    let result = HtmlNaturalDimensions::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        vec![
            HtmlNaturalDimensionFact::selection(a.clone(), Some(key_a.clone())),
            HtmlNaturalDimensionFact::selection(b.clone(), Some(key_b.clone())),
        ],
        vec![
            observation(b, &key_b),
            observation(e1, &key_1),
            observation(a, &key_a),
            observation(e2, &key_2),
        ],
        HtmlNaturalDimensionLimits::new(100, 1 << 30, 1 << 20),
    );
    assert!(
        matches!(&result, Err(HtmlNaturalDimensionFactsError::UnexpectedObservation(id)) if id == e2),
        "{result:?}"
    );
}

#[test]
fn rel_observations_of_one_subject_are_duplicates_only_when_their_keys_agree() {
    let (capture, ids) = page("a", &[("rel", "ext ext"), ("href", "x")], 2);
    let make = |input| {
        HtmlRelFacts::new(
            &capture,
            capture.host_facts_revision(),
            input,
            HtmlRelFactsLimits::new(1000, 1 << 20, 1 << 20),
        )
    };
    let key = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 1);
    let definition = || {
        HtmlRelDefinition::extension(
            key.clone(),
            HtmlSpaceToken::new("ext").unwrap(),
            HtmlRelRegistryStatus::Proposed,
            HtmlRelExtensionEffect::NotAllowed,
            HtmlRelExtensionEffect::Hyperlink,
            HtmlRelExtensionEffect::Hyperlink,
        )
    };
    let occurrence = |id: &HtmlCaptureSubjectId, range: std::ops::Range<usize>| {
        HtmlRelRegistryOccurrence::definition(id.clone(), "ext ext".into(), range, key.clone())
    };
    // Two tokens of one attribute, and the same token of two elements, differ.
    assert!(
        make(HtmlRelFactsInput::new().with_registry(
            rel_revision(),
            vec![definition()],
            vec![
                occurrence(&ids[0], 0..3),
                occurrence(&ids[0], 4..7),
                occurrence(&ids[1], 0..3),
            ],
        ))
        .is_ok()
    );
    assert_eq!(
        make(HtmlRelFactsInput::new().with_registry(
            rel_revision(),
            vec![definition()],
            vec![
                occurrence(&ids[0], 0..3),
                occurrence(&ids[1], 4..7),
                occurrence(&ids[0], 0..3),
            ],
        ))
        .unwrap_err(),
        HtmlRelFactsError::DuplicateObservation
    );
    let tokens = |id: &HtmlCaptureSubjectId| {
        HtmlRelSupportedTokens::new(id.clone(), vec![HtmlRelKeyword::NoOpener])
    };
    assert!(
        make(
            HtmlRelFactsInput::new()
                .with_supported_tokens(rel_revision(), vec![tokens(&ids[0]), tokens(&ids[1])])
        )
        .is_ok()
    );
    assert_eq!(
        make(HtmlRelFactsInput::new().with_supported_tokens(
            rel_revision(),
            vec![tokens(&ids[0]), tokens(&ids[1]), tokens(&ids[0])]
        ))
        .unwrap_err(),
        HtmlRelFactsError::DuplicateObservation
    );
    let grammar = |id: &HtmlCaptureSubjectId| {
        HtmlRelUrlGrammar::new(id.clone(), HtmlRelGrammarInput::Href, "x".into(), true)
    };
    assert!(
        make(
            HtmlRelFactsInput::new()
                .with_url_grammar(rel_revision(), vec![grammar(&ids[0]), grammar(&ids[1])])
        )
        .is_ok()
    );
    assert_eq!(
        make(
            HtmlRelFactsInput::new()
                .with_url_grammar(rel_revision(), vec![grammar(&ids[1]), grammar(&ids[1])])
        )
        .unwrap_err(),
        HtmlRelFactsError::DuplicateObservation
    );
    let section = |id: &HtmlCaptureSubjectId| {
        HtmlRelBookmarkSection::new(id.clone(), HtmlRelSectionId::fresh())
    };
    assert_eq!(
        make(HtmlRelFactsInput::new().with_bookmark_sections(
            rel_revision(),
            vec![section(&ids[0]), section(&ids[1]), section(&ids[0])]
        ))
        .unwrap_err(),
        HtmlRelFactsError::DuplicateObservation
    );
}

#[test]
fn rel_url_observations_of_one_subject_bind_by_url_and_refuse_equal_ones() {
    let (capture, ids) = page("form", &[("rel", "ext")], 2);
    let make = |input| {
        HtmlRelFacts::new(
            &capture,
            capture.host_facts_revision(),
            input,
            HtmlRelFactsLimits::new(1000, 1 << 20, 1 << 20),
        )
    };
    let revision = rel_revision();
    let origin = HtmlRelOriginId::fresh();
    let settings = HtmlRelSettingsId::fresh();
    let (first, second) = (
        HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), revision.clone()),
        HtmlRelUrlKey::new(HtmlRelUrlId::fresh(), revision.clone()),
    );
    let url = |id: &HtmlCaptureSubjectId, key: &HtmlRelUrlKey, origin: &HtmlRelOriginId| {
        HtmlRelUrlObservation::new(
            id.clone(),
            key.clone(),
            HtmlRelBlobEntry::NonNull {
                environment_origin: origin.clone(),
            },
        )
    };
    let same_site = |id: &HtmlCaptureSubjectId, key: &HtmlRelUrlKey| {
        HtmlRelSameSiteObservation::new(
            id.clone(),
            key.clone(),
            settings.clone(),
            origin.clone(),
            HtmlRelOriginId::fresh(),
            true,
        )
    };
    // One element with two URLs: each same-site observation finds its own.
    assert!(
        make(
            HtmlRelFactsInput::new()
                .with_url_records(
                    revision.clone(),
                    vec![
                        url(&ids[0], &first, &origin),
                        url(&ids[0], &second, &origin),
                        url(&ids[1], &first, &origin),
                    ]
                )
                .with_settings(
                    revision.clone(),
                    vec![
                        same_site(&ids[0], &second),
                        same_site(&ids[0], &first),
                        same_site(&ids[1], &first),
                    ]
                )
        )
        .is_ok()
    );
    assert_eq!(
        make(HtmlRelFactsInput::new().with_url_records(
            revision.clone(),
            vec![
                url(&ids[0], &first, &origin),
                url(&ids[1], &first, &origin),
                url(&ids[0], &first, &origin),
            ]
        ))
        .unwrap_err(),
        HtmlRelFactsError::DuplicateObservation
    );
    // A URL of another element does not bind the observation.
    assert_eq!(
        make(
            HtmlRelFactsInput::new()
                .with_url_records(revision.clone(), vec![url(&ids[1], &first, &origin)])
                .with_settings(revision.clone(), vec![same_site(&ids[0], &first)])
        )
        .unwrap_err(),
        HtmlRelFactsError::UrlBindingMismatch
    );
    assert_eq!(
        make(
            HtmlRelFactsInput::new()
                .with_url_records(
                    revision.clone(),
                    vec![
                        url(&ids[0], &first, &origin),
                        url(&ids[1], &second, &origin)
                    ]
                )
                .with_settings(
                    revision,
                    vec![same_site(&ids[0], &first), same_site(&ids[0], &first)]
                )
        )
        .unwrap_err(),
        HtmlRelFactsError::DuplicateObservation
    );
}

/// Image dimension facts joined with participation and natural dimension facts,
/// for `count` images, each with a content box, listed in the reverse order in
/// the facts they are joined with so that finding one by scanning is not cheap.
fn join_total(count: usize) -> usize {
    let (capture, ids) = page("img", &[], count);
    let natural_revision =
        HtmlNaturalDimensionRevision::new(HtmlNaturalDimensionProducerId::fresh(), 5, 6);
    let participation_revision = HtmlListParticipationRevision::new(
        HtmlParticipationProducerId::fresh(),
        1,
        2,
        HtmlParticipationProducerId::fresh(),
        3,
        4,
    );
    let keys: Vec<_> = ids
        .iter()
        .map(|_| HtmlNaturalResourceKey::new(HtmlNaturalResourceId::fresh(), 14, 15))
        .collect();
    let known = |n: &str, d: &str| HtmlImageDimensionMeasurement::Known(rational(n, d));
    least_work_unreported("image dimension joins", |work| {
        let expected = ids
            .iter()
            .zip(&keys)
            .map(|(id, key)| {
                HtmlImageDimensionExpectation::new(
                    id.clone(),
                    HtmlImageDimensionRole::Img,
                    HtmlImageDimensionSelection::Selected(key.clone()),
                    11,
                    None,
                )
            })
            .collect();
        let boxes = ids
            .iter()
            .map(|id| {
                HtmlImageContentBoxObservation::new(
                    capture.identity().clone(),
                    capture.host_facts_revision().clone(),
                    id.clone(),
                    participation_revision.clone(),
                    known("7", "1"),
                    known("9", "2"),
                )
            })
            .collect();
        let image_revision = HtmlImageDimensionRevision::new(
            Some(natural_revision.clone()),
            Some(participation_revision.clone()),
        );
        let image = match HtmlImageDimensionFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            image_revision.clone(),
            expected,
            vec![],
            boxes,
            HtmlImageDimensionFactsLimits::new(16 * count, 1 << 30, work),
        ) {
            Ok(image) => image,
            Err(HtmlImageDimensionFactsError::WorkLimit) => return Run::Refused { required: 0 },
            Err(other) => panic!("fixture refused for another reason: {other:?}"),
        };
        let participation = HtmlListParticipation::new(
            &capture,
            participation_revision.clone(),
            ids.iter()
                .rev()
                .map(|id| HtmlListParticipationFact::new(id.clone(), None, Some(true), Some(true)))
                .collect(),
            16 * count,
        )
        .unwrap();
        let natural = HtmlNaturalDimensions::new(
            &capture,
            capture.host_facts_revision().clone(),
            natural_revision.clone(),
            ids.iter()
                .zip(&keys)
                .rev()
                .map(|(id, key)| HtmlNaturalDimensionFact::selection(id.clone(), Some(key.clone())))
                .collect(),
            vec![],
            HtmlNaturalDimensionLimits::new(16 * count, 1 << 30, 1 << 40),
        )
        .unwrap();
        match HtmlSemanticContext::new(
            &capture,
            capture.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![])
                .with_list_participation(participation_revision.clone(), Some(participation))
                .with_natural_dimensions(natural_revision.clone(), Some(natural))
                .with_image_dimensions(image_revision, Some(image)),
            0,
        ) {
            Ok(_) => Run::Admitted,
            Err(HtmlSemanticContextError::ImageDimensionsFacts(
                HtmlImageDimensionFactsError::WorkLimit,
            )) => Run::Refused { required: 0 },
            Err(other) => panic!("fixture refused for another reason: {other:?}"),
        }
    })
}

#[test]
fn image_dimension_joins_find_each_participation_and_natural_fact_in_work_linear_in_the_facts() {
    assert_doubling_at_most_2_2("image dimension joins", &DOUBLINGS, join_total);
}

/// A host with `children` text children and a shadow root of `slots` slots, in
/// the given assignment mode: the slots and the text children.
fn slotted(
    mode: HtmlSlotAssignment,
    slots: usize,
    children: usize,
) -> (
    HtmlCapture,
    Vec<HtmlCaptureSubjectId>,
    Vec<HtmlCaptureSubjectId>,
) {
    let (mut b, d) = big_builder(slots + children + 4);
    let host = attributed(&mut b, &d, "div", &[]);
    let texts: Vec<_> = (0..children)
        .map(|n| b.add_text(&d, format!("t{n}").as_str().into()).unwrap())
        .collect();
    b.establish_children(&host, &texts).unwrap();
    let root = b
        .add_shadow_root(
            &d,
            &host,
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Closed,
                    mode,
                    false,
                    false,
                    false,
                ),
                false,
                false,
                false,
            ),
            surgeist_html_semantics::HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let slot_elements: Vec<_> = (0..slots)
        .map(|_| attributed(&mut b, &d, "slot", &[]))
        .collect();
    for slot in &slot_elements {
        b.establish_children(slot, &[]).unwrap();
    }
    b.establish_children(&root, &slot_elements).unwrap();
    b.establish_children(&d, std::slice::from_ref(&host))
        .unwrap();
    let capture = b.finish().unwrap();
    let id = |node| capture.subject_id(node).unwrap();
    let slot_ids = slot_elements.iter().map(id).collect();
    let text_ids = texts.iter().map(id).collect();
    (capture, slot_ids, text_ids)
}
fn language_total(
    capture: &HtmlCapture,
    assignments: impl Fn() -> Vec<HtmlSlotAssignedNodes>,
    assigned: usize,
) -> usize {
    least_work_unreported(
        "language assignments",
        |work| match HtmlLanguageDirectionFacts::new(
            capture,
            capture.host_facts_revision().clone(),
            vec![],
            vec![],
            assignments(),
            HtmlLanguageDirectionLimits::new(1 << 20, 2 * assigned, 1 << 20, work),
        ) {
            Ok(_) => Run::Admitted,
            Err(HtmlLanguageDirectionError::ValidationWork { .. }) => Run::Refused { required: 0 },
            Err(other) => panic!("fixture refused for another reason: {other:?}"),
        },
    )
}

#[test]
fn nodes_assigned_to_one_slot_check_duplicates_in_work_linear_in_the_nodes() {
    assert_doubling_at_most_2_2("one slot's nodes", &DOUBLINGS, |count| {
        let (capture, slots, texts) = slotted(HtmlSlotAssignment::Named, 1, count);
        language_total(
            &capture,
            || {
                vec![HtmlSlotAssignedNodes::new(
                    slots[0].clone(),
                    Some(texts.clone()),
                )]
            },
            count,
        )
    });
}

#[test]
fn nodes_assigned_to_many_slots_check_duplicates_in_work_linear_in_the_nodes() {
    assert_doubling_at_most_2_2("many slots' nodes", &DOUBLINGS, |count| {
        let (capture, slots, texts) = slotted(HtmlSlotAssignment::Manual, count, count);
        language_total(
            &capture,
            || {
                slots
                    .iter()
                    .zip(&texts)
                    .map(|(slot, text)| {
                        HtmlSlotAssignedNodes::new(slot.clone(), Some(vec![text.clone()]))
                    })
                    .collect()
            },
            count,
        )
    });
}

#[test]
fn a_node_assigned_twice_is_refused_wherever_the_other_assignment_is() {
    let (capture, slots, texts) = slotted(HtmlSlotAssignment::Manual, 3, 3);
    let make = |assignments| {
        HtmlLanguageDirectionFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            vec![],
            vec![],
            assignments,
            HtmlLanguageDirectionLimits::new(100, 100, 100, 1 << 20),
        )
    };
    let assign = |slot: usize, nodes: &[usize]| {
        HtmlSlotAssignedNodes::new(
            slots[slot].clone(),
            Some(nodes.iter().map(|n| texts[*n].clone()).collect()),
        )
    };
    assert!(make(vec![assign(0, &[0, 1]), assign(1, &[2]), assign(2, &[])]).is_ok());
    // Twice within a slot, and once in an earlier slot.
    for assignments in [
        vec![assign(0, &[0, 1, 0])],
        vec![assign(0, &[1]), assign(1, &[2]), assign(2, &[0, 1])],
    ] {
        assert!(matches!(
            make(assignments),
            Err(HtmlLanguageDirectionError::DuplicateAssignment(_))
        ));
    }
    // A later slot's repeat of an earlier node is reported even though the
    // earlier slot is the one with the node first.
    match make(vec![assign(0, &[0]), assign(1, &[0])]) {
        Err(HtmlLanguageDirectionError::DuplicateAssignment(id)) => assert_eq!(id, texts[0]),
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Base and hyperlink observations

fn base_revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1)
}
fn base_url(revision: &HtmlBaseFactsRevision) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://base.example/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap()
}

#[test]
fn hyperlink_observations_check_duplicates_in_work_linear_in_the_observations() {
    assert_doubling_at_most_2_2("hyperlink observations", &DOUBLINGS, |count| {
        let revision = base_revision();
        let base = base_url(&revision);
        let encoding = HtmlBaseEncoding::fresh();
        least_work_unreported("hyperlink observations", |work| {
            let observations = (0..count)
                .map(|n| {
                    HtmlEncodingUrlParseObservation::new(
                        format!("link-{n}").as_str().into(),
                        HtmlEncodingUrlParseOutcome::Failure,
                    )
                })
                .collect();
            match HtmlEncodingUrlParseFacts::new(
                base.clone(),
                encoding.clone(),
                revision.clone(),
                observations,
                HtmlBaseFactsLimits::new(8 * count, 1 << 30, work),
            ) {
                Ok(_) => Run::Admitted,
                Err(HtmlEncodingUrlFactsError::Formation(HtmlBaseFactsError::WorkLimit)) => {
                    Run::Refused { required: 0 }
                }
                Err(other) => panic!("fixture refused for another reason: {other:?}"),
            }
        })
    });
}

#[test]
fn parse_base_observations_check_duplicates_in_work_linear_in_the_observations() {
    assert_doubling_at_most_2_2("parse base observations", &DOUBLINGS, |count| {
        let revision = base_revision();
        let fallback = base_url(&revision);
        let encoding = HtmlBaseEncoding::fresh();
        least_work_unreported("parse base observations", |work| {
            let document = HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(fallback.clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                None,
            )
            .unwrap();
            let observations = (0..count)
                .map(|n| {
                    HtmlBaseParseObservation::new(
                        format!("base-{n}").as_str().into(),
                        fallback.clone(),
                        encoding.clone(),
                        revision.clone(),
                        HtmlBaseParseOutcome::Failure,
                    )
                    .unwrap()
                })
                .collect();
            match HtmlParseBaseFacts::new(
                document,
                observations,
                HtmlBaseFactsLimits::new(8 * count, 1 << 30, work),
            ) {
                Ok(_) => Run::Admitted,
                Err(HtmlBaseFactsError::WorkLimit) => Run::Refused { required: 0 },
                Err(other) => panic!("fixture refused for another reason: {other:?}"),
            }
        })
    });
}

#[test]
fn captured_base_documents_and_frozen_rows_check_duplicates_in_work_linear_in_the_rows() {
    assert_doubling_at_most_2_2("captured base rows", &DOUBLINGS, |count| {
        let (mut b, first) = big_builder(2 * count);
        let mut documents = vec![first];
        for _ in 1..count {
            documents.push(
                b.add_document(
                    surgeist_html::construction::HtmlDocumentMode::NoQuirks,
                    surgeist_html_semantics::HtmlScriptingStatus::Disabled,
                    surgeist_html_semantics::HtmlCaptureRegistryInput::Null,
                )
                .unwrap(),
            );
        }
        let bases: Vec<_> = documents
            .iter()
            .map(|d| {
                let base = attributed(&mut b, d, "base", &[("href", "x")]);
                b.establish_children(&base, &[]).unwrap();
                b.establish_children(d, std::slice::from_ref(&base))
                    .unwrap();
                base
            })
            .collect();
        let capture = b.finish().unwrap();
        let document_ids: Vec<_> = documents
            .iter()
            .map(|d| {
                capture
                    .document_id(&capture.subject_id(d).unwrap())
                    .unwrap()
            })
            .collect();
        let base_ids: Vec<_> = bases
            .iter()
            .map(|base| capture.subject_id(base).unwrap())
            .collect();
        let revision = base_revision();
        let fallback = base_url(&revision);
        least_work_unreported("captured base rows", |work| {
            let rows = document_ids
                .iter()
                .map(|document| {
                    HtmlCapturedBaseDocument::new(
                        document.clone(),
                        HtmlParseBaseFacts::new(
                            HtmlBaseDocumentFacts::new(
                                revision.clone(),
                                Some(fallback.clone()),
                                HtmlAboutBaseUrl::KnownNull,
                                false,
                                None,
                            )
                            .unwrap(),
                            vec![],
                            HtmlBaseFactsLimits::new(8, 1 << 30, 1 << 30),
                        )
                        .unwrap(),
                    )
                })
                .collect();
            let frozen = base_ids
                .iter()
                .zip(&document_ids)
                .map(|(subject, document)| {
                    HtmlCapturedFrozenBase::new(
                        subject.clone(),
                        document.clone(),
                        "x".into(),
                        None,
                        revision.clone(),
                    )
                })
                .collect();
            match HtmlCapturedBaseFacts::new(
                &capture,
                capture.host_facts_revision().clone(),
                revision.clone(),
                rows,
                frozen,
                HtmlBaseFactsLimits::new(16 * count, 1 << 30, work),
            ) {
                Ok(_) => Run::Admitted,
                Err(HtmlBaseFactsError::WorkLimit) => Run::Refused { required: 0 },
                Err(other) => panic!("fixture refused for another reason: {other:?}"),
            }
        })
    });
}
