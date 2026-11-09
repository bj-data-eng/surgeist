//! The fact constructors charging a scope a host supplies.
//!
//! Each fixture runs a constructor privately and in a host operation. The private run's least
//! admitting work limit is the oracle: the same fixture admitted under a host cap of exactly that
//! work charges the host that work, and one below it stops with the refusal the private run
//! reports at that limit, though the constructor's own limit is unbounded.

use crate::semantics::{
    constructor_work::{attributed, html, rational},
    support::{builder, builder_with, context},
};
use surgeist_html::{
    accounting::{
        HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapScope, HtmlHostScope, HtmlLimits,
        HtmlResource, HtmlWork,
    },
    microsyntax::tokens::HtmlSpaceToken,
};
use surgeist_html_semantics::{
    HtmlAboutBaseUrl, HtmlBaseDocumentFacts, HtmlBaseEncoding, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme, HtmlEncodingUrlParseFacts, HtmlParseBaseFacts,
};
use surgeist_html_semantics::{
    HtmlBaseFactsLimits, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlCapture, HtmlCaptureLimits,
    HtmlCaptureSubjectId, HtmlCapturedBaseFacts, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlDocumentFlatTreeRelation, HtmlElementReferenceFact, HtmlElementReferenceFacts,
    HtmlElementReferenceFactsLimits, HtmlElementReferenceKey, HtmlExplicitElementReference,
    HtmlFlatTreeProducerId, HtmlFlatTreeRevision, HtmlFormAssociation, HtmlFormAssociationState,
    HtmlGlobalInheritanceFacts, HtmlGlobalInheritanceLimits, HtmlGlobalInheritanceProducerId,
    HtmlGlobalInheritanceRevision, HtmlImageDimensionFacts, HtmlImageDimensionFactsLimits,
    HtmlImageDimensionRevision, HtmlInertFactLimits, HtmlInertFacts, HtmlInertNodeFact,
    HtmlLanguageDirectionFacts, HtmlLanguageDirectionLimits, HtmlMediaAttributeInput,
    HtmlMediaContextRevision, HtmlMediaEnvironmentRevision, HtmlMediaFact, HtmlMediaFacts,
    HtmlMediaFactsLimits, HtmlMediaGrammarAttestation, HtmlMediaGrammarOutcome,
    HtmlMediaGrammarRevision, HtmlMediaMatchAttestation, HtmlMediaMatchOutcome,
    HtmlMediaProducerId, HtmlNaturalDimensionAxis, HtmlNaturalDimensionFact,
    HtmlNaturalDimensionLimits, HtmlNaturalDimensionProducerId, HtmlNaturalDimensionRevision,
    HtmlNaturalDimensions, HtmlNaturalResourceId, HtmlNaturalResourceKey, HtmlOutputDefaultFacts,
    HtmlOutputDefaultLimits, HtmlRelDefinition, HtmlRelDefinitionId, HtmlRelDefinitionKey,
    HtmlRelExtensionEffect, HtmlRelFacts, HtmlRelFactsInput, HtmlRelFactsLimits, HtmlRelProducerId,
    HtmlRelRegistryOccurrence, HtmlRelRegistryStatus, HtmlRelRevision,
};
use surgeist_html_semantics::{HtmlDefaultFeedApplicability, HtmlDefaultFeedInput};
use surgeist_ledger::{Operation, ProfileMode, SharedOperation};

type Ledger = SharedOperation<HtmlWork, HtmlResource>;

/// How a constructor run ended: the same value for a private and a hosted run of one fixture.
#[derive(Debug, Eq, PartialEq)]
enum Outcome {
    Admitted,
    /// Refused at the work limit, with the figures the family reports, if it reports any.
    Refused(Option<(usize, usize)>),
}

/// A run of one fixture: below the host scope when one is given, and with `work` as the family's
/// own validation work limit.
type Run = fn(Option<&HtmlHostScope>, usize) -> Outcome;

/// A profiled host operation whose root caps the semantic steps at `work`.
fn host(work: u64) -> (Ledger, HtmlHostScope) {
    let caps = HtmlLimits::from_fn(
        |dimension| match dimension {
            HtmlWork::SemanticStep => work,
            _ => u64::MAX,
        },
        |_| u64::MAX,
    );
    let operation =
        Operation::with_profile("host", caps, 4, ProfileMode::Summary { timing: false }).unwrap();
    let root = operation.root();
    let ledger = operation.share();
    let scope = HtmlHostScope::new(ledger.clone(), root);
    (ledger, scope)
}

/// The least validation work limit the fixture admits privately.
fn least_work(run: Run) -> usize {
    let mut high = 1;
    while run(None, high) != Outcome::Admitted {
        high *= 2;
    }
    let mut low = 0;
    assert_ne!(run(None, 0), Outcome::Admitted, "the fixture charges work");
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if run(None, middle) == Outcome::Admitted {
            high = middle;
        } else {
            low = middle;
        }
    }
    high
}

/// Checks the fixture against the private run, and that the host operation holds the family's
/// scope and its growth scope below the host's.
fn charges_the_host(family: HtmlCapScope, run: Run) {
    let total = least_work(run);
    let (ledger, scope) = host(total as u64);
    assert_eq!(run(Some(&scope), usize::MAX), Outcome::Admitted);
    let report = ledger.snapshot().unwrap().unwrap();
    let names: Vec<_> = report.scopes.iter().map(|scope| scope.name).collect();
    assert_eq!(
        names,
        ["host", family.name(), HtmlAuxiliaryScope::FactGrown.name()]
    );
    assert_eq!(report.scopes[1].parent, Some(scope.scope()));
    assert_eq!(report.scopes[2].parent, Some(report.scopes[1].scope));
    assert_eq!(
        report.scopes[0].usage.work(HtmlWork::SemanticStep),
        total as u64
    );
    assert_eq!(
        report.scopes[1].usage.work(HtmlWork::SemanticStep),
        total as u64
    );
    assert_eq!(
        report.scopes[2].usage.work(HtmlWork::SemanticStep),
        0,
        "the growth scope projects nothing"
    );
    // One below the total, the host's cap refuses as the family's own limit does.
    let (_, tight) = host(total as u64 - 1);
    assert_eq!(run(Some(&tight), usize::MAX), run(None, total - 1));
    // A host operation with no room for the two scopes stops the constructor with the scope
    // fault, whatever the family calls its other errors.
    let operation =
        Operation::<HtmlWork, HtmlResource>::new("host", HtmlLimits::unlimited(), 2).unwrap();
    let root = operation.root();
    let full = HtmlHostScope::new(operation.share(), root);
    assert_ne!(run(Some(&full), usize::MAX), Outcome::Admitted);
}

fn element_page(names: &[(&str, &[(&str, &str)])]) -> (HtmlCapture, Vec<HtmlCaptureSubjectId>) {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        100, 10000, 100, 100000, 100, 10, 10, 16_777_216,
    ));
    let elements: Vec<_> = names
        .iter()
        .map(|(name, attributes)| attributed(&mut b, &d, name, attributes))
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

fn natural(scope: Option<&HtmlHostScope>, work: usize) -> Outcome {
    use surgeist_html_semantics::HtmlNaturalDimensionFactsError as Error;
    let (capture, ids) = element_page(&[("img", &[])]);
    let revision =
        HtmlNaturalDimensionRevision::new(HtmlNaturalDimensionProducerId::fresh(), 7, 17);
    let key = HtmlNaturalResourceKey::new(HtmlNaturalResourceId::fresh(), 11, 13);
    let expected = vec![HtmlNaturalDimensionFact::selection(
        ids[0].clone(),
        Some(key.clone()),
    )];
    let observed = vec![HtmlNaturalDimensionFact::observation(
        ids[0].clone(),
        revision.clone(),
        key,
        HtmlNaturalDimensionAxis::KnownPresent(rational("3", "2")),
        HtmlNaturalDimensionAxis::KnownPresent(rational("5", "1")),
    )];
    let limits = HtmlNaturalDimensionLimits::new(100, 1_000_000, work);
    let host = capture.host_facts_revision().clone();
    let result = match scope {
        Some(scope) => HtmlNaturalDimensions::with_accounting(
            &capture, host, revision, expected, observed, limits, scope,
        ),
        None => HtmlNaturalDimensions::new(&capture, host, revision, expected, observed, limits),
    };
    match result {
        Ok(_) => Outcome::Admitted,
        Err(Error::WorkLimit) => Outcome::Refused(None),
        Err(Error::Accounting(HtmlAccountingFault::ScopeLimit { .. })) => Outcome::Refused(None),
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

fn media(scope: Option<&HtmlHostScope>, work: usize) -> Outcome {
    use surgeist_html_semantics::HtmlMediaFactsError as Error;
    let (capture, ids) = element_page(&[
        ("link", &[("rel", "icon"), ("media", "screen")]),
        ("style", &[("media", "all")]),
    ]);
    let context = HtmlMediaContextRevision::new(
        HtmlMediaGrammarRevision::new(HtmlMediaProducerId::fresh(), 1, 2),
        HtmlMediaEnvironmentRevision::new(HtmlMediaProducerId::fresh(), 3, 4),
    );
    let facts = ids
        .iter()
        .zip([1, 0])
        .map(|(id, ordinal)| {
            HtmlMediaFact::new(
                HtmlMediaAttributeInput::new(&capture, id.clone(), ordinal).unwrap(),
                Some(HtmlMediaGrammarAttestation::new(
                    context.grammar().clone(),
                    HtmlMediaGrammarOutcome::Valid,
                )),
                Some(HtmlMediaMatchAttestation::new(
                    context.clone(),
                    HtmlMediaMatchOutcome::Matches,
                )),
            )
        })
        .collect();
    let limits = HtmlMediaFactsLimits::new(100, 1_000_000, work);
    let host = capture.host_facts_revision().clone();
    let result = match scope {
        Some(scope) => {
            HtmlMediaFacts::with_accounting(&capture, host, context, facts, limits, scope)
        }
        None => HtmlMediaFacts::new(&capture, host, context, facts, limits),
    };
    match result {
        Ok(_) => Outcome::Admitted,
        Err(Error::WorkLimit) => Outcome::Refused(None),
        Err(Error::Accounting(HtmlAccountingFault::ScopeLimit { .. })) => Outcome::Refused(None),
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

fn rel(scope: Option<&HtmlHostScope>, work: usize) -> Outcome {
    use surgeist_html_semantics::HtmlRelFactsError as Error;
    let raw = "ext other zzz";
    let (capture, ids) = element_page(&[("form", &[("rel", raw)])]);
    let revision = HtmlRelRevision::new(HtmlRelProducerId::fresh(), 3, 7);
    let first = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 1);
    let second = HtmlRelDefinitionKey::new(HtmlRelDefinitionId::fresh(), 2);
    let definition = |key: &HtmlRelDefinitionKey, name: &str| {
        HtmlRelDefinition::extension(
            key.clone(),
            HtmlSpaceToken::new(name).unwrap(),
            HtmlRelRegistryStatus::Proposed,
            HtmlRelExtensionEffect::NotAllowed,
            HtmlRelExtensionEffect::Hyperlink,
            HtmlRelExtensionEffect::Hyperlink,
        )
    };
    let input = HtmlRelFactsInput::new().with_registry(
        revision,
        vec![definition(&first, "ext"), definition(&second, "other")],
        vec![
            HtmlRelRegistryOccurrence::definition(ids[0].clone(), raw.into(), 0..3, first),
            HtmlRelRegistryOccurrence::definition(ids[0].clone(), raw.into(), 4..9, second),
            HtmlRelRegistryOccurrence::unavailable(ids[0].clone(), raw.into(), 10..13),
        ],
    );
    let limits = HtmlRelFactsLimits::new(100, 1_000_000, work);
    let result = match scope {
        Some(scope) => HtmlRelFacts::with_accounting(
            &capture,
            capture.host_facts_revision(),
            input,
            limits,
            scope,
        ),
        None => HtmlRelFacts::new(&capture, capture.host_facts_revision(), input, limits),
    };
    match result {
        Ok(_) => Outcome::Admitted,
        Err(Error::WorkLimit) => Outcome::Refused(None),
        Err(Error::Accounting(HtmlAccountingFault::ScopeLimit { .. })) => Outcome::Refused(None),
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

/// A form with a `button` and a `div`, and an `input` that the form owns.
fn controls() -> (
    HtmlCapture,
    [HtmlCaptureSubjectId; 4], // form, input, button, target
) {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        100, 1000, 100, 100000, 100, 0, 0, 16_777_216,
    ));
    let form = html(&mut b, &d, "form");
    let input = html(&mut b, &d, "input");
    let button = html(&mut b, &d, "button");
    let target = html(&mut b, &d, "div");
    for leaf in [&input, &button, &target] {
        b.establish_children(leaf, &[]).unwrap();
    }
    b.establish_children(&form, &[input.clone(), button.clone(), target.clone()])
        .unwrap();
    b.establish_children(&d, std::slice::from_ref(&form))
        .unwrap();
    let capture = b.finish().unwrap();
    let ids = [&form, &input, &button, &target].map(|e| capture.subject_id(e).unwrap());
    (capture, ids)
}

fn control(scope: Option<&HtmlHostScope>, work: usize) -> Outcome {
    use surgeist_html_semantics::HtmlControlFactsError as Error;
    let (capture, [form, input, button, _]) = controls();
    let associations = vec![
        HtmlFormAssociation::new(
            input,
            HtmlFormAssociationState::EstablishedCurrent(Some(form)),
        ),
        HtmlFormAssociation::new(button, HtmlFormAssociationState::ResetAgainstCapture),
    ];
    let limits = HtmlControlFactsLimits::new(associations.capacity(), work);
    let host = capture.host_facts_revision().clone();
    let result = match scope {
        Some(scope) => HtmlControlFacts::with_accounting(
            &capture,
            host,
            associations,
            vec![],
            vec![],
            vec![],
            limits,
            scope,
        ),
        None => HtmlControlFacts::new(&capture, host, associations, vec![], vec![], vec![], limits),
    };
    match result {
        Ok(_) => Outcome::Admitted,
        Err(Error::ValidationWork { limit, required }) => Outcome::Refused(Some((limit, required))),
        Err(Error::Accounting(HtmlAccountingFault::ScopeLimit { .. })) => Outcome::Refused(None),
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

fn reference(scope: Option<&HtmlHostScope>, work: usize) -> Outcome {
    use surgeist_html_semantics::HtmlElementReferenceFactsError as Error;
    let (capture, [_, input, button, target]) = controls();
    let records = vec![
        HtmlElementReferenceFact::new(
            button.clone(),
            HtmlElementReferenceKey::CommandFor,
            HtmlExplicitElementReference::Element(target),
        ),
        HtmlElementReferenceFact::new(
            input,
            HtmlElementReferenceKey::PopoverTarget,
            HtmlExplicitElementReference::Unavailable,
        ),
    ];
    let limits = HtmlElementReferenceFactsLimits::new(records.capacity(), work);
    let host = capture.host_facts_revision().clone();
    let result = match scope {
        Some(scope) => {
            HtmlElementReferenceFacts::with_accounting(&capture, host, records, limits, scope)
        }
        None => HtmlElementReferenceFacts::new(&capture, host, records, limits),
    };
    match result {
        Ok(_) => Outcome::Admitted,
        Err(Error::ValidationWork { limit, required }) => Outcome::Refused(Some((limit, required))),
        Err(Error::Accounting(HtmlAccountingFault::ScopeLimit { .. })) => Outcome::Refused(None),
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

fn inert(scope: Option<&HtmlHostScope>, work: usize) -> Outcome {
    use surgeist_html_semantics::HtmlInertFactError as Error;
    let (capture, [form, ..]) = controls();
    let nodes = vec![HtmlInertNodeFact::new(
        form,
        None,
        HtmlDocumentFlatTreeRelation::Unavailable,
    )];
    let records = nodes.capacity();
    let revision = HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 7, 12);
    let limits = HtmlInertFactLimits::new(records, work);
    let host = capture.host_facts_revision().clone();
    let result = match scope {
        Some(scope) => {
            HtmlInertFacts::with_accounting(&capture, host, revision, nodes, vec![], limits, scope)
        }
        None => HtmlInertFacts::new(&capture, host, revision, nodes, vec![], limits),
    };
    match result {
        Ok(_) => Outcome::Admitted,
        Err(Error::ValidationWork { limit, required }) => Outcome::Refused(Some((limit, required))),
        Err(Error::Accounting(HtmlAccountingFault::ScopeLimit { .. })) => Outcome::Refused(None),
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

#[test]
fn natural_dimension_facts_charge_the_host_scope() {
    charges_the_host(HtmlCapScope::NaturalDimensionFacts, natural);
}

#[test]
fn media_facts_charge_the_host_scope() {
    charges_the_host(HtmlCapScope::MediaFacts, media);
}

#[test]
fn link_relationship_facts_charge_the_host_scope() {
    charges_the_host(HtmlCapScope::LinkRelationshipFacts, rel);
}

#[test]
fn control_facts_charge_the_host_scope() {
    charges_the_host(HtmlCapScope::ControlFacts, control);
}

#[test]
fn element_reference_facts_charge_the_host_scope() {
    charges_the_host(HtmlCapScope::ElementReferenceFacts, reference);
}

#[test]
fn inert_facts_charge_the_host_scope() {
    charges_the_host(HtmlCapScope::InertFacts, inert);
}

/// Runs a constructor of empty inputs below a host scope and returns the names of the scopes the
/// host operation then holds. Empty inputs charge no work, so only the scopes show the charge.
fn scopes_of_an_empty_run(run: impl FnOnce(&HtmlCapture, &HtmlHostScope)) -> Vec<&'static str> {
    let (capture, _) = controls();
    let (ledger, scope) = host(u64::MAX);
    run(&capture, &scope);
    let report = ledger.snapshot().unwrap().unwrap();
    report.scopes.iter().map(|scope| scope.name).collect()
}

fn below_the_host(family: HtmlCapScope) -> Vec<&'static str> {
    vec!["host", family.name(), HtmlAuxiliaryScope::FactGrown.name()]
}

#[test]
fn output_default_facts_charge_the_host_scope() {
    let names = scopes_of_an_empty_run(|capture, scope| {
        HtmlOutputDefaultFacts::with_accounting(
            capture,
            capture.host_facts_revision().clone(),
            vec![],
            HtmlOutputDefaultLimits::new(10, 10, 1_000),
            scope,
        )
        .unwrap();
    });
    assert_eq!(names, below_the_host(HtmlCapScope::OutputDefault));
}

#[test]
fn global_inheritance_facts_charge_the_host_scope() {
    let names = scopes_of_an_empty_run(|capture, scope| {
        HtmlGlobalInheritanceFacts::with_accounting(
            capture,
            capture.host_facts_revision().clone(),
            HtmlGlobalInheritanceRevision::new(HtmlGlobalInheritanceProducerId::fresh(), 1, 2),
            vec![],
            vec![],
            HtmlGlobalInheritanceLimits::new(10, 1_000),
            scope,
        )
        .unwrap();
    });
    assert_eq!(names, below_the_host(HtmlCapScope::GlobalInheritance));
}

#[test]
fn language_direction_facts_charge_the_host_scope() {
    let names = scopes_of_an_empty_run(|capture, scope| {
        HtmlLanguageDirectionFacts::with_accounting(
            capture,
            capture.host_facts_revision().clone(),
            vec![],
            vec![],
            vec![],
            HtmlLanguageDirectionLimits::new(10, 10, 10, 1_000),
            scope,
        )
        .unwrap();
    });
    assert_eq!(names, below_the_host(HtmlCapScope::LanguageDirection));
}

#[test]
fn image_dimension_facts_charge_the_host_scope() {
    let names = scopes_of_an_empty_run(|capture, scope| {
        HtmlImageDimensionFacts::with_accounting(
            capture,
            capture.host_facts_revision().clone(),
            HtmlImageDimensionRevision::new(None, None),
            vec![],
            vec![],
            vec![],
            HtmlImageDimensionFactsLimits::new(10, 1_000, 1_000),
            scope,
        )
        .unwrap();
    });
    assert_eq!(names, below_the_host(HtmlCapScope::ImageDimensionFacts));
}

#[test]
fn captured_base_facts_charge_the_host_scope() {
    let names = scopes_of_an_empty_run(|capture, scope| {
        HtmlCapturedBaseFacts::with_accounting(
            capture,
            capture.host_facts_revision().clone(),
            HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1),
            vec![],
            vec![],
            HtmlBaseFactsLimits::new(10, 10, 1_000),
            scope,
        )
        .unwrap();
    });
    assert_eq!(names, below_the_host(HtmlCapScope::BaseFacts));
}

fn base_url(revision: &HtmlBaseFactsRevision, text: &str) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        text.into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap()
}

#[test]
fn parse_base_facts_charge_the_host_scope() {
    let names = scopes_of_an_empty_run(|_, scope| {
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1);
        let document = HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(base_url(&revision, "https://example.test/")),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(HtmlBaseEncoding::fresh()),
        )
        .unwrap();
        HtmlParseBaseFacts::with_accounting(
            document,
            vec![],
            HtmlBaseFactsLimits::new(10, 1_000, 1_000),
            scope,
        )
        .unwrap();
    });
    assert_eq!(names, below_the_host(HtmlCapScope::BaseFacts));
}

#[test]
fn encoding_url_parse_facts_charge_the_host_scope() {
    let names = scopes_of_an_empty_run(|_, scope| {
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        HtmlEncodingUrlParseFacts::with_accounting(
            base_url(&revision, "about:blank"),
            HtmlBaseEncoding::fresh(),
            revision,
            vec![],
            HtmlBaseFactsLimits::new(16, 4096, 10_000),
            scope,
        )
        .unwrap();
    });
    assert_eq!(names, below_the_host(HtmlCapScope::BaseFacts));
}

#[test]
fn default_feed_inputs_charge_the_host_scope() {
    let (mut b, d) = builder();
    b.establish_children(&d, &[]).unwrap();
    let capture = b.finish().unwrap();
    let cx = context(&capture);
    let (ledger, scope) = host(u64::MAX);
    HtmlDefaultFeedInput::with_accounting(
        &capture,
        &cx,
        capture.subject_id(&d).unwrap(),
        HtmlDefaultFeedApplicability::Applicable,
        HtmlRelFactsLimits::new(100, 1_000_000, 1_000_000),
        &scope,
    )
    .unwrap();
    let report = ledger.snapshot().unwrap().unwrap();
    let names: Vec<_> = report.scopes.iter().map(|scope| scope.name).collect();
    assert_eq!(names, below_the_host(HtmlCapScope::LinkRelationshipFacts));
    assert!(report.scopes[1].usage.work(HtmlWork::SemanticStep) > 0);
}

#[test]
fn a_hosted_registry_formation_charges_its_own_scope_and_its_builders_capture_scope() {
    let (ledger, scope) = host(u64::MAX);
    let revision = surgeist_html_semantics::HtmlHostFactsRevision::fresh();
    let private =
        HtmlCapture::complete_empty_global_registry_with(revision.clone(), usize::MAX, |_| {
            Ok::<_, ()>(())
        })
        .unwrap();
    let hosted = HtmlCapture::complete_empty_global_registry_with_accounting(
        revision,
        usize::MAX,
        |_| Ok::<_, ()>(()),
        &scope,
    )
    .unwrap();
    assert_eq!(
        hosted.formation_peak_bytes(),
        private.formation_peak_bytes()
    );
    assert_eq!(
        hosted.retained_storage_usage(),
        private.retained_storage_usage()
    );
    let report = ledger.snapshot().unwrap().unwrap();
    let names: Vec<_> = report.scopes.iter().map(|scope| scope.name).collect();
    assert_eq!(
        names,
        [
            "host",
            HtmlCapScope::RegistryFormation.name(),
            HtmlCapScope::Capture.name()
        ]
    );
    assert_eq!(
        report.scopes[0]
            .usage
            .live(HtmlResource::CapturedRegistries),
        1
    );
    // The backing the capture holds stays charged; the formation peak is a gauge, which stays
    // below the host scope.
    assert!(report.scopes[0].usage.live(HtmlResource::FormationBacking) > 0);
    assert_eq!(report.scopes[0].usage.peak(HtmlResource::FormationBytes), 0);
}

#[test]
fn a_host_scope_bound_without_room_for_the_formation_scopes_stops_it_with_the_scope_fault() {
    let operation =
        Operation::<HtmlWork, HtmlResource>::new("host", HtmlLimits::unlimited(), 2).unwrap();
    let root = operation.root();
    let scope = HtmlHostScope::new(operation.share(), root);
    let error = HtmlCapture::complete_empty_global_registry_with_accounting(
        surgeist_html_semantics::HtmlHostFactsRevision::fresh(),
        usize::MAX,
        |_| Ok::<_, ()>(()),
        &scope,
    )
    .unwrap_err();
    assert_eq!(
        error.accounting_fault(),
        Some(HtmlAccountingFault::ScopeLimit { limit: 2 })
    );
}
