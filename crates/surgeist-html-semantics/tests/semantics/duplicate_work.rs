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
use surgeist_html::{elements::link::HtmlRelKeyword, microsyntax::tokens::HtmlSpaceToken};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureSubjectId, HtmlImageContentBoxObservation, HtmlImageDimensionCompleted,
    HtmlImageDimensionExpectation, HtmlImageDimensionFacts, HtmlImageDimensionFactsError,
    HtmlImageDimensionFactsLimits, HtmlImageDimensionMeasurement, HtmlImageDimensionOperationKey,
    HtmlImageDimensionResourceObservation, HtmlImageDimensionRevision, HtmlImageDimensionRole,
    HtmlImageDimensionSelection, HtmlImageSizingRevision, HtmlImgRequestState,
    HtmlListParticipationRevision, HtmlMediaAttributeInput, HtmlMediaContextRevision,
    HtmlMediaEnvironmentRevision, HtmlMediaFact, HtmlMediaFacts, HtmlMediaFactsError,
    HtmlMediaFactsLimits, HtmlMediaGrammarAttestation, HtmlMediaGrammarOutcome,
    HtmlMediaGrammarRevision, HtmlMediaMatchAttestation, HtmlMediaMatchOutcome,
    HtmlMediaProducerId, HtmlNaturalDimensionAxis, HtmlNaturalDimensionFact,
    HtmlNaturalDimensionFactsError, HtmlNaturalDimensionLimits, HtmlNaturalDimensionProducerId,
    HtmlNaturalDimensionRevision, HtmlNaturalDimensions, HtmlNaturalResourceId,
    HtmlNaturalResourceKey, HtmlParticipationProducerId, HtmlRelBlobEntry, HtmlRelBookmarkSection,
    HtmlRelDefinition, HtmlRelDefinitionId, HtmlRelDefinitionKey, HtmlRelExtensionEffect,
    HtmlRelFacts, HtmlRelFactsError, HtmlRelFactsInput, HtmlRelFactsLimits, HtmlRelGrammarInput,
    HtmlRelOriginId, HtmlRelProducerId, HtmlRelRegistryOccurrence, HtmlRelRegistryStatus,
    HtmlRelRevision, HtmlRelSameSiteObservation, HtmlRelSectionId, HtmlRelSettingsId,
    HtmlRelSupportedTokens, HtmlRelUrlGrammar, HtmlRelUrlId, HtmlRelUrlKey, HtmlRelUrlObservation,
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
