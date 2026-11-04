//! Charged validation work of the fact constructors.
//!
//! A constructor reports work only by refusing: it succeeds exactly when its
//! `validation_work` limit admits everything it charges, and otherwise stops at
//! the first charge that would pass the limit. The least admitting limit is
//! therefore the constructor's total charged work, and the refusal one below it
//! reports that same total as the work it required. These tests pin those
//! totals on fixed fixtures (characterization of the current charging, so a
//! later change to a total is visible and must be intended).
//!
//! Where a fixture is small enough the total is also derived by hand from the
//! charging rules of the constructor, as a check that does not read the
//! implementation's output.

use crate::semantics::support::builder_with;
use surgeist_html::{
    bounded::lexical::{HtmlSpaceTokenCursor, HtmlSpaceTokenStep},
    construction::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment},
    elements::link::HtmlRelKeyword,
    microsyntax::{
        numeric::{HtmlNonNegativeInteger, HtmlNonNegativeRational},
        tokens::HtmlSpaceToken,
    },
    values::{Attribute, Namespace, OrderedAttributes, QualifiedName},
    vocabulary::{KnownHtmlAttribute, KnownHtmlElement},
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlControlCurrentValue, HtmlControlFacts, HtmlControlFactsError,
    HtmlControlFactsLimits, HtmlDocumentFlatTreeRelation, HtmlDocumentLanguageFallback,
    HtmlDocumentModalBlocking, HtmlDocumentModalFact, HtmlElementReferenceFact,
    HtmlElementReferenceFacts, HtmlElementReferenceFactsError, HtmlElementReferenceFactsLimits,
    HtmlElementReferenceKey, HtmlExplicitElementReference, HtmlFlatTreeProducerId,
    HtmlFlatTreeRevision, HtmlFormAssociation, HtmlFormAssociationState,
    HtmlImageContentBoxObservation, HtmlImageDimensionCompleted, HtmlImageDimensionExpectation,
    HtmlImageDimensionFacts, HtmlImageDimensionFactsError, HtmlImageDimensionFactsLimits,
    HtmlImageDimensionMeasurement, HtmlImageDimensionOperationKey,
    HtmlImageDimensionResourceObservation, HtmlImageDimensionRevision, HtmlImageDimensionRole,
    HtmlImageDimensionSelection, HtmlImageSizingRevision, HtmlImgRequestState, HtmlInertFactError,
    HtmlInertFactLimits, HtmlInertFacts, HtmlInertNodeFact, HtmlInputControlState,
    HtmlLanguageDirectionError, HtmlLanguageDirectionFacts, HtmlLanguageDirectionLimits,
    HtmlListParticipationRevision, HtmlMediaAttributeInput, HtmlMediaContextRevision,
    HtmlMediaEnvironmentRevision, HtmlMediaFact, HtmlMediaFacts, HtmlMediaFactsError,
    HtmlMediaFactsLimits, HtmlMediaGrammarAttestation, HtmlMediaGrammarOutcome,
    HtmlMediaGrammarRevision, HtmlMediaMatchAttestation, HtmlMediaMatchOutcome,
    HtmlMediaProducerId, HtmlNaturalDimensionAxis, HtmlNaturalDimensionFact,
    HtmlNaturalDimensionFactsError, HtmlNaturalDimensionLimits, HtmlNaturalDimensionProducerId,
    HtmlNaturalDimensionRevision, HtmlNaturalDimensions, HtmlNaturalResourceId,
    HtmlNaturalResourceKey, HtmlOptionControlState, HtmlParticipationProducerId,
    HtmlPragmaLanguage, HtmlProtocolLanguage, HtmlRelDefinition, HtmlRelDefinitionId,
    HtmlRelDefinitionKey, HtmlRelExtensionEffect, HtmlRelFacts, HtmlRelFactsError,
    HtmlRelFactsInput, HtmlRelFactsLimits, HtmlRelProducerId, HtmlRelRegistryOccurrence,
    HtmlRelRegistryStatus, HtmlRelRevision, HtmlSlotAssignedNodes, HtmlTextareaControlState,
};

/// How a constructor run ended at one work limit.
pub enum Run {
    /// Every charge was admitted.
    Admitted,
    /// The work refusal, with the total it required.
    Refused { required: usize },
}

/// The least work limit a constructor admits, found by search over `run`.
///
/// `run` builds the constructor's input afresh and reports whether the limit
/// admitted it; any other failure is a broken fixture and must panic inside
/// `run`. The total is checked to be the exact boundary: one less is refused.
pub fn least_work(label: &str, run: impl Fn(usize) -> Run) -> usize {
    let mut high = 1;
    while matches!(run(high), Run::Refused { .. }) {
        high *= 2;
    }
    let mut low = 0;
    // `low` is refused (0 admits nothing the fixture charges), `high` admits.
    assert!(
        matches!(run(0), Run::Refused { .. }),
        "{label}: a fixture must charge work"
    );
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        match run(middle) {
            Run::Admitted => high = middle,
            Run::Refused { .. } => low = middle,
        }
    }
    match run(high - 1) {
        Run::Refused { required } => assert_eq!(
            required, high,
            "{label}: the refusal one below the total must require the total"
        ),
        Run::Admitted => panic!("{label}: {high} is not the least admitting limit"),
    }
    high
}

/// As [`least_work`] for a family whose work refusal carries no totals: the
/// least admitting limit, with the limit below it refused.
pub fn least_work_unreported(label: &str, run: impl Fn(usize) -> Run) -> usize {
    let mut high = 1;
    while matches!(run(high), Run::Refused { .. }) {
        high *= 2;
    }
    assert!(
        matches!(run(0), Run::Refused { .. }),
        "{label}: a fixture must charge work"
    );
    let mut low = 0;
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        match run(middle) {
            Run::Admitted => high = middle,
            Run::Refused { .. } => low = middle,
        }
    }
    assert!(
        matches!(run(high - 1), Run::Refused { .. }),
        "{label}: {high} is not the least admitting limit"
    );
    high
}

// ---------------------------------------------------------------------------
// Inert facts

fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, name),
        OrderedAttributes::default(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn inert_revision() -> HtmlFlatTreeRevision {
    HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 7, 12)
}
fn inert_run(
    c: &HtmlCapture,
    nodes: Vec<HtmlInertNodeFact>,
    documents: Vec<HtmlDocumentModalFact>,
    work: usize,
) -> Run {
    let records = nodes.capacity() + documents.capacity();
    match HtmlInertFacts::new(
        c,
        c.host_facts_revision().clone(),
        inert_revision(),
        nodes,
        documents,
        HtmlInertFactLimits::new(records, work),
    ) {
        Ok(_) => Run::Admitted,
        Err(HtmlInertFactError::ValidationWork { limit, required }) => {
            assert_eq!(limit, work);
            Run::Refused { required }
        }
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

/// One document, an `html` root, a `body`, and under the body a `dialog`, a
/// `div` and a `span`, each connected through its parent.
struct Page {
    capture: HtmlCapture,
    document: HtmlCaptureSubjectId,
    html: HtmlCaptureSubjectId,
    body: HtmlCaptureSubjectId,
    dialog: HtmlCaptureSubjectId,
    div: HtmlCaptureSubjectId,
    span: HtmlCaptureSubjectId,
}
fn page() -> Page {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        100, 1000, 100, 100000, 100, 0, 0, 16_777_216,
    ));
    let root = html(&mut b, &d, "html");
    let body = html(&mut b, &d, "body");
    let dialog = html(&mut b, &d, "dialog");
    let div = html(&mut b, &d, "div");
    let span = html(&mut b, &d, "span");
    b.establish_children(&dialog, &[]).unwrap();
    b.establish_children(&div, &[]).unwrap();
    b.establish_children(&span, &[]).unwrap();
    b.establish_children(&body, &[dialog.clone(), div.clone(), span.clone()])
        .unwrap();
    b.establish_children(&root, std::slice::from_ref(&body))
        .unwrap();
    b.establish_children(&d, std::slice::from_ref(&root))
        .unwrap();
    let capture = b.finish().unwrap();
    Page {
        document: capture.subject_id(&d).unwrap(),
        html: capture.subject_id(&root).unwrap(),
        body: capture.subject_id(&body).unwrap(),
        dialog: capture.subject_id(&dialog).unwrap(),
        div: capture.subject_id(&div).unwrap(),
        span: capture.subject_id(&span).unwrap(),
        capture,
    }
}
fn inert_fact(
    subject: &HtmlCaptureSubjectId,
    connected: Option<bool>,
    relation: HtmlDocumentFlatTreeRelation,
) -> HtmlInertNodeFact {
    HtmlInertNodeFact::new(subject.clone(), connected, relation)
}

/// The smallest facts: one unavailable record. By hand, from the constructor:
/// the duplicate set for one record costs 1; the record's node lookup 1; its
/// connection walk visits the element (lookup 1, kind test 1, parent step 1) and
/// then the document (lookup 1, kind test 1) for 5; the relation test 1; the
/// flat-chain walk finds the record (1 fact examined) and reads its relation
/// (1) before stopping at a non-parent; the duplicate set for no documents
/// costs 0. No connectedness was supplied, so there is no comparison to charge:
/// 1 + 1 + 5 + 1 + 1 + 1 = 10.
#[test]
fn one_unavailable_record_charges_ten() {
    let p = page();
    let total = least_work("one unavailable record", |work| {
        inert_run(
            &p.capture,
            vec![inert_fact(
                &p.html,
                None,
                HtmlDocumentFlatTreeRelation::Unavailable,
            )],
            vec![],
            work,
        )
    });
    assert_eq!(total, 10);
}

fn page_nodes(p: &Page) -> Vec<HtmlInertNodeFact> {
    vec![
        inert_fact(&p.html, Some(true), HtmlDocumentFlatTreeRelation::Root),
        inert_fact(
            &p.body,
            Some(true),
            HtmlDocumentFlatTreeRelation::Parent(p.html.clone()),
        ),
        inert_fact(
            &p.dialog,
            Some(true),
            HtmlDocumentFlatTreeRelation::Parent(p.body.clone()),
        ),
        inert_fact(
            &p.div,
            Some(true),
            HtmlDocumentFlatTreeRelation::Parent(p.body.clone()),
        ),
        inert_fact(&p.span, None, HtmlDocumentFlatTreeRelation::Excluded),
    ]
}

#[test]
fn a_page_of_roots_parents_and_a_modal_blocker_charges_its_pinned_total() {
    let p = page();
    let documents = || {
        vec![HtmlDocumentModalFact::new(
            p.document.clone(),
            HtmlDocumentModalBlocking::BlockedBy(p.dialog.clone()),
        )]
    };
    let total = least_work("page", |work| {
        inert_run(&p.capture, page_nodes(&p), documents(), work)
    });
    // The same records without the modal fact charge the record scan alone.
    let nodes_only = least_work("page nodes", |work| {
        inert_run(&p.capture, page_nodes(&p), vec![], work)
    });
    // By hand, with `A` the size of the element vocabulary (a recognition of an
    // element name costs (name units + 1) * A + 28 for the HTML namespace).
    // Records, in order, each charge a node lookup (1), the connection walk
    // (3 per element up to the document, then 2), the supplied-connectedness
    // check (1 when both sides are known), the relation test (1) and the
    // relation's own checks, then the flat chain walk (a lookup of the record
    // by position in the list, plus 1, per hop):
    //   html  (root):          1 + 5 + 1 + 1 + 2                       = 10
    //   body  (child of html): 1 + 8 + 1 + 1 + (1 + 1 + 5 + 1)         = 19
    //   dialog (child of body):1 + 11 + 1 + 1 + (1 + 1 + 8 + 2)        = 26
    //   div   (child of body): the same                                = 26
    //   span  (excluded):      1 + 11 + 1                              = 13
    // chains: html 2, body 3 + 2, dialog 4 + 3 + 2, div 5 + 3 + 2, span 6 = 32
    // and 5 for the duplicate set: 94 + 32 + 5 = 131.
    assert_eq!(nodes_only, 131);
    // The modal fact adds 1 for its duplicate set and 1 for its record, a node
    // lookup (1), the dialog recognition (7A + 28), the document test (1), the
    // dialog's connection walk (11) and its record lookup (3).
    assert_eq!(
        total,
        131 + 1 + 1 + 1 + 7 * KnownHtmlElement::ALL.len() + 28 + 1 + 11 + 3
    );
}

// ---------------------------------------------------------------------------
// Language and direction facts

fn attributed(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
    attributes: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, name),
        OrderedAttributes::new(attributes.iter().map(|(name, value)| {
            Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
        }))
        .unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn shadow_root(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    host: &HtmlCaptureBuilderSubject,
) -> HtmlCaptureBuilderSubject {
    b.add_shadow_root(
        d,
        host,
        HtmlCapturedShadowProperties::new(
            HtmlShadowRootProperties::new(
                HtmlShadowRootMode::Closed,
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
}

/// A document with a `textarea`, a text `input`, and a `div` host whose closed
/// named shadow root holds two slots. The host has two text children.
struct Directed {
    capture: HtmlCapture,
    document: HtmlCaptureSubjectId,
    textarea: HtmlCaptureSubjectId,
    input: HtmlCaptureSubjectId,
    first_slot: HtmlCaptureSubjectId,
    second_slot: HtmlCaptureSubjectId,
    texts: [HtmlCaptureSubjectId; 2],
}
fn directed() -> Directed {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        100, 1000, 100, 100000, 100, 0, 0, 16_777_216,
    ));
    let textarea = attributed(&mut b, &d, "textarea", &[]);
    let input = attributed(&mut b, &d, "input", &[("type", "text")]);
    let host = attributed(&mut b, &d, "div", &[]);
    let one = b.add_text(&d, "a".into()).unwrap();
    let two = b.add_text(&d, "b".into()).unwrap();
    b.establish_children(&host, &[one.clone(), two.clone()])
        .unwrap();
    let root = shadow_root(&mut b, &d, &host);
    let first = attributed(&mut b, &d, "slot", &[]);
    let second = attributed(&mut b, &d, "slot", &[]);
    b.establish_children(&root, &[first.clone(), second.clone()])
        .unwrap();
    b.establish_children(&textarea, &[]).unwrap();
    b.establish_children(&input, &[]).unwrap();
    b.establish_children(&first, &[]).unwrap();
    b.establish_children(&second, &[]).unwrap();
    b.establish_children(&d, &[textarea.clone(), input.clone(), host])
        .unwrap();
    let capture = b.finish().unwrap();
    Directed {
        document: capture.subject_id(&d).unwrap(),
        textarea: capture.subject_id(&textarea).unwrap(),
        input: capture.subject_id(&input).unwrap(),
        first_slot: capture.subject_id(&first).unwrap(),
        second_slot: capture.subject_id(&second).unwrap(),
        texts: [
            capture.subject_id(&one).unwrap(),
            capture.subject_id(&two).unwrap(),
        ],
        capture,
    }
}
fn language_run(
    f: &Directed,
    fallbacks: Vec<HtmlDocumentLanguageFallback>,
    values: Vec<HtmlControlCurrentValue>,
    assignments: Vec<HtmlSlotAssignedNodes>,
    work: usize,
) -> Run {
    match HtmlLanguageDirectionFacts::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        fallbacks,
        values,
        assignments,
        HtmlLanguageDirectionLimits::new(100, 100, 1000, work),
    ) {
        Ok(_) => Run::Admitted,
        Err(HtmlLanguageDirectionError::ValidationWork { limit, required }) => {
            assert_eq!(limit, work);
            Run::Refused { required }
        }
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}
/// The total for the chosen vectors of the language fixture: fallbacks, current
/// values, slot assignments.
fn language_total(f: &Directed, label: &str, use_vectors: [bool; 3]) -> usize {
    least_work(label, |work| {
        let [fallbacks, values, assignments] = use_vectors;
        language_run(
            f,
            if fallbacks {
                vec![HtmlDocumentLanguageFallback::new(
                    f.document.clone(),
                    HtmlPragmaLanguage::Tag("en".into()),
                    HtmlProtocolLanguage::Single("fr".into()),
                )]
            } else {
                vec![]
            },
            if values {
                vec![
                    HtmlControlCurrentValue::new(f.textarea.clone(), Some("abc".into())),
                    HtmlControlCurrentValue::new(f.input.clone(), None),
                ]
            } else {
                vec![]
            },
            if assignments {
                vec![
                    HtmlSlotAssignedNodes::new(f.first_slot.clone(), Some(f.texts.to_vec())),
                    HtmlSlotAssignedNodes::new(f.second_slot.clone(), Some(vec![])),
                ]
            } else {
                vec![]
            },
            work,
        )
    })
}

#[test]
fn language_facts_charge_their_pinned_totals() {
    let f = directed();
    let totals = [
        language_total(&f, "fallbacks", [true, false, false]),
        language_total(&f, "values", [false, true, false]),
        language_total(&f, "assignments", [false, false, true]),
        language_total(&f, "all", [true, true, true]),
    ];
    // By hand, with `A` and `K` the sizes of the element and attribute
    // vocabularies. Fallback: duplicate set 1, record 1, document lookup 1,
    // pragma tag 1, protocol 1. Values: duplicate set 2, then per record 1 and
    // a lookup 1 and the recognition of the element name; the textarea
    // (9 units: 9A + 28) and its value 1, the input (6A + 28) with its `type`
    // attribute scan (1 + 4), its type recognition envelope (18A + 5K + 92 + 87).
    // Assignments: duplicate set 2; the first slot charges 1, a lookup 1, the
    // recognition (5A + 28), the host search (4), the host children (2) and its
    // two nodes (4 and 5); the second slot, empty, charges 1 + 1 + 5A + 28.
    let a = KnownHtmlElement::ALL.len();
    let k = KnownHtmlAttribute::ALL.len();
    assert_eq!(
        totals,
        [
            5,
            247 + 33 * a + 5 * k,
            77 + 10 * a,
            5 + 247 + 33 * a + 5 * k + 77 + 10 * a
        ]
    );
}

// ---------------------------------------------------------------------------
// Control facts and element-reference facts

/// A document holding a `form` that contains an `input`, a `textarea`, a
/// `button`, a `select` with an `option`, and a `div` target.
struct Controls {
    capture: HtmlCapture,
    form: HtmlCaptureSubjectId,
    input: HtmlCaptureSubjectId,
    textarea: HtmlCaptureSubjectId,
    button: HtmlCaptureSubjectId,
    option: HtmlCaptureSubjectId,
    target: HtmlCaptureSubjectId,
}
fn controls() -> Controls {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        100, 1000, 100, 100000, 100, 0, 0, 16_777_216,
    ));
    let form = html(&mut b, &d, "form");
    let input = html(&mut b, &d, "input");
    let textarea = html(&mut b, &d, "textarea");
    let button = html(&mut b, &d, "button");
    let select = html(&mut b, &d, "select");
    let option = html(&mut b, &d, "option");
    let target = html(&mut b, &d, "div");
    for leaf in [&input, &textarea, &button, &option, &target] {
        b.establish_children(leaf, &[]).unwrap();
    }
    b.establish_children(&select, std::slice::from_ref(&option))
        .unwrap();
    b.establish_children(
        &form,
        &[
            input.clone(),
            textarea.clone(),
            button.clone(),
            select,
            target.clone(),
        ],
    )
    .unwrap();
    b.establish_children(&d, std::slice::from_ref(&form))
        .unwrap();
    let capture = b.finish().unwrap();
    Controls {
        form: capture.subject_id(&form).unwrap(),
        input: capture.subject_id(&input).unwrap(),
        textarea: capture.subject_id(&textarea).unwrap(),
        button: capture.subject_id(&button).unwrap(),
        option: capture.subject_id(&option).unwrap(),
        target: capture.subject_id(&target).unwrap(),
        capture,
    }
}
fn control_run(f: &Controls, associations: bool, states: bool, work: usize) -> Run {
    let (associations, inputs, options, textareas) = if associations {
        (
            vec![
                HtmlFormAssociation::new(
                    f.input.clone(),
                    HtmlFormAssociationState::EstablishedCurrent(Some(f.form.clone())),
                ),
                HtmlFormAssociation::new(
                    f.textarea.clone(),
                    HtmlFormAssociationState::EstablishedCurrent(None),
                ),
                HtmlFormAssociation::new(
                    f.button.clone(),
                    HtmlFormAssociationState::ResetAgainstCapture,
                ),
            ],
            vec![],
            vec![],
            vec![],
        )
    } else {
        (vec![], vec![], vec![], vec![])
    };
    let (inputs, options, textareas) = if states {
        (
            vec![HtmlInputControlState::new(
                f.input.clone(),
                Some(false),
                None,
                Some(true),
                None,
            )],
            vec![HtmlOptionControlState::new(
                f.option.clone(),
                Some(true),
                None,
            )],
            vec![HtmlTextareaControlState::new(
                f.textarea.clone(),
                Some(true),
            )],
        )
    } else {
        (inputs, options, textareas)
    };
    let records =
        associations.capacity() + inputs.capacity() + options.capacity() + textareas.capacity();
    match HtmlControlFacts::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        associations,
        inputs,
        options,
        textareas,
        HtmlControlFactsLimits::new(records, work),
    ) {
        Ok(_) => Run::Admitted,
        Err(HtmlControlFactsError::ValidationWork { limit, required }) => {
            assert_eq!(limit, work);
            Run::Refused { required }
        }
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

#[test]
fn control_facts_charge_their_pinned_totals() {
    let f = controls();
    let totals = [
        least_work("associations", |work| control_run(&f, true, false, work)),
        least_work("states", |work| control_run(&f, false, true, work)),
    ];
    // By hand. Associations: duplicate set 3; each record charges 1, a lookup 1,
    // the HTML namespace test 29 and the recognition of its element name
    // (input 6A + 28, textarea 9A + 28, button 7A + 28); the input's owner adds
    // a lookup 1, the form recognition (5A + 28), 2 for the documents, and the
    // two root walks (6 and 4). States: three duplicate sets of 1, then per
    // record 1, a lookup 1 and the recognition (input 6A + 28, option 7A + 28,
    // textarea 9A + 28).
    let a = KnownHtmlElement::ALL.len();
    assert_eq!(totals, [221 + 27 * a, 93 + 22 * a]);
}

fn reference_run(f: &Controls, work: usize) -> Run {
    let records = vec![
        HtmlElementReferenceFact::new(
            f.button.clone(),
            HtmlElementReferenceKey::CommandFor,
            HtmlExplicitElementReference::Element(f.target.clone()),
        ),
        HtmlElementReferenceFact::new(
            f.button.clone(),
            HtmlElementReferenceKey::PopoverTarget,
            HtmlExplicitElementReference::Null,
        ),
        HtmlElementReferenceFact::new(
            f.input.clone(),
            HtmlElementReferenceKey::PopoverTarget,
            HtmlExplicitElementReference::Unavailable,
        ),
    ];
    let capacity = records.capacity();
    match HtmlElementReferenceFacts::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        records,
        HtmlElementReferenceFactsLimits::new(capacity, work),
    ) {
        Ok(_) => Run::Admitted,
        Err(HtmlElementReferenceFactsError::ValidationWork { limit, required }) => {
            assert_eq!(limit, work);
            Run::Refused { required }
        }
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

#[test]
fn element_reference_facts_charge_their_pinned_total() {
    let f = controls();
    // By hand: duplicate set 3; each record charges 2 and the recognition of its
    // element name (button 7A + 28, input 6A + 28); the first record's target
    // lookup adds 1.
    let a = KnownHtmlElement::ALL.len();
    assert_eq!(
        least_work("element references", |work| reference_run(&f, work)),
        94 + 20 * a
    );
}

// ---------------------------------------------------------------------------
// The four fact families: link relationships, media, image dimensions and
// natural dimensions. Their refusals do not carry the totals, so these pin the
// least admitting work limit only.

fn integer(digits: &str) -> HtmlNonNegativeInteger {
    HtmlNonNegativeInteger::from_decimal_digits(&digits.into()).unwrap()
}
fn rational(numerator: &str, denominator: &str) -> HtmlNonNegativeRational {
    HtmlNonNegativeRational::new(integer(numerator), integer(denominator)).unwrap()
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

fn natural_run(work: usize) -> Run {
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
    match HtmlNaturalDimensions::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision,
        expected,
        observed,
        HtmlNaturalDimensionLimits::new(100, 1_000_000, work),
    ) {
        Ok(_) => Run::Admitted,
        Err(HtmlNaturalDimensionFactsError::WorkLimit) => Run::Refused { required: 0 },
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

fn media_run(work: usize) -> Run {
    let (capture, ids) = element_page(&[
        ("link", &[("rel", "icon"), ("media", "screen")]),
        ("link", &[("rel", "icon"), ("media", "print")]),
        ("style", &[("media", "all")]),
    ]);
    let media = HtmlMediaContextRevision::new(
        HtmlMediaGrammarRevision::new(HtmlMediaProducerId::fresh(), 1, 2),
        HtmlMediaEnvironmentRevision::new(HtmlMediaProducerId::fresh(), 3, 4),
    );
    let ordinals = [1, 1, 0];
    let facts = ids
        .iter()
        .zip(ordinals)
        .map(|(id, ordinal)| {
            HtmlMediaFact::new(
                HtmlMediaAttributeInput::new(&capture, id.clone(), ordinal).unwrap(),
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
        media,
        facts,
        HtmlMediaFactsLimits::new(100, 1_000_000, work),
    ) {
        Ok(_) => Run::Admitted,
        Err(HtmlMediaFactsError::WorkLimit) => Run::Refused { required: 0 },
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

fn rel_run(work: usize) -> Run {
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
    match HtmlRelFacts::new(
        &capture,
        capture.host_facts_revision(),
        input,
        HtmlRelFactsLimits::new(100, 1_000_000, work),
    ) {
        Ok(_) => Run::Admitted,
        Err(HtmlRelFactsError::WorkLimit) => Run::Refused { required: 0 },
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

fn image_run(work: usize) -> Run {
    let (capture, ids) = element_page(&[("img", &[])]);
    let resource = HtmlNaturalDimensionRevision::new(HtmlNaturalDimensionProducerId::fresh(), 5, 6);
    let participation = HtmlListParticipationRevision::new(
        HtmlParticipationProducerId::fresh(),
        1,
        2,
        HtmlParticipationProducerId::fresh(),
        3,
        4,
    );
    let operation = HtmlImageDimensionOperationKey::ImgPreferredDensityCorrected {
        operation: HtmlImageSizingRevision::new(HtmlParticipationProducerId::fresh(), 7, 8),
        request: 11,
        density: 12,
        preferred: 13,
    };
    let expectation = HtmlImageDimensionExpectation::new(
        ids[0].clone(),
        HtmlImageDimensionRole::Img,
        HtmlImageDimensionSelection::Selected(HtmlNaturalResourceKey::new(
            HtmlNaturalResourceId::fresh(),
            14,
            15,
        )),
        11,
        Some(operation.clone()),
    );
    let known = |n: &str, d: &str| HtmlImageDimensionMeasurement::Known(rational(n, d));
    let observation = HtmlImageDimensionResourceObservation::img(
        resource.clone(),
        expectation.clone(),
        Some(HtmlImgRequestState::CompletelyAvailable),
        Some(HtmlImageDimensionCompleted::new(
            operation,
            known("3", "2"),
            known("5", "1"),
        )),
    );
    let content_box = HtmlImageContentBoxObservation::new(
        capture.identity().clone(),
        capture.host_facts_revision().clone(),
        ids[0].clone(),
        participation.clone(),
        known("7", "1"),
        known("9", "2"),
    );
    match HtmlImageDimensionFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlImageDimensionRevision::new(Some(resource), Some(participation)),
        vec![expectation],
        vec![observation],
        vec![content_box],
        HtmlImageDimensionFactsLimits::new(100, 1_000_000, work),
    ) {
        Ok(_) => Run::Admitted,
        Err(HtmlImageDimensionFactsError::WorkLimit) => Run::Refused { required: 0 },
        Err(other) => panic!("fixture refused for another reason: {other:?}"),
    }
}

#[test]
fn fact_families_charge_their_pinned_totals() {
    let totals = [
        least_work_unreported("natural dimensions", natural_run),
        least_work_unreported("media", media_run),
        least_work_unreported("link relationships", rel_run),
        least_work_unreported("image dimensions", image_run),
    ];
    let a = KnownHtmlElement::ALL.len();
    let r = HtmlRelKeyword::ALL.len();
    // By hand. Natural dimensions: the expected selection 1; the observation 1,
    // its revision test 1 and two axes 2; the join of the selection charges 1,
    // looks up its observation 1 and checks it 1: 8.
    // Media: per fact 64 + 1 + 1 and one comparison per earlier fact: 3 * 66 + 0 + 1 + 2.
    // Image dimensions: the expectation resolves its element (1 + 4A + 28) and
    // charges 1; the resource charges 1, finds its binding 1, compares it 1,
    // completes 1 and measures two axes 2; the content box resolves its element
    // (1 + 4A + 28), charges 3 and measures two axes 2: 70 + 8A.
    // Link relationships: five vectors and two more for URL observations 7; the
    // definitions "ext" (1 + 1 + 5 + 4R) and "other" (1 + 1 + 1 + 1 + 5 + 6R),
    // each keyword costing one more than the definition's units; then each of
    // three occurrences of the `form`: 1, a node lookup 1, its recognition
    // (5A + 28), 1, the `rel` attribute lookup 8, the raw comparison 14 and the
    // token walk (1 + 4 per step of the HTML tokenizer to the occurrence's
    // token); the second and third compare with the earlier occurrences at 3
    // each (3 + 6); the two defined occurrences look up their definition (1,
    // 2) and compare spelling (7, 11).
    let raw: Vec<u16> = "ext other zzz".encode_utf16().collect();
    let walk = |start: usize, end: usize| {
        let mut work = HtmlSpaceTokenCursor::BEGIN_WORK;
        let mut cursor = HtmlSpaceTokenCursor::new(&raw);
        loop {
            work += HtmlSpaceTokenCursor::STEP_WORK;
            match cursor.step() {
                HtmlSpaceTokenStep::Token { start: s, end: e } if s == start && e == end => {
                    return work;
                }
                HtmlSpaceTokenStep::Done => panic!("no token {start}..{end}"),
                _ => {}
            }
        }
    };
    let walks = walk(0, 3) + walk(4, 9) + walk(10, 13);
    assert_eq!(
        totals,
        [
            8,
            3 * 66 + 1 + 2,
            7 + (7 + 4 * r) + (9 + 6 * r) + 3 * (53 + 5 * a) + walks + (3 + 6) + (1 + 7 + 2 + 11),
            70 + 8 * a,
        ]
    );
}
