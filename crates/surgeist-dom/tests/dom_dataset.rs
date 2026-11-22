//! Canonical dataset Attr order and Source's exact null-namespace local-name writes.

use std::cell::Cell;
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    Dom, DomBatch, DomError, DomSnapshot,
    edit::DomChange,
    limits::{DomLimits, DomNameLimits},
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress},
    tree::{DomAttr, DomDataset},
};
use surgeist_html::values::{DocumentString, Namespace};

fn units(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

fn fixture() -> (Dom, DomDataset) {
    let limits = DomLimits::new(
        64,
        65_536,
        512,
        0,
        0,
        64,
        1_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let html = setup.create_element_ns(&document, Namespace::html(), "div");
    let svg = setup.create_element_ns(&document, Namespace::svg(), "svg");
    let math = setup.create_element_ns(&document, Namespace::mathml(), "math");
    let other = setup.create_element_ns(&document, Namespace::from_uri("urn:other"), "div");
    setup.set_attribute_ns(
        &html,
        Namespace::from_uri("urn:first"),
        "p:data-key",
        "first",
    );
    setup.set_attribute_ns(
        &html,
        Namespace::from_uri("urn:second"),
        "data-key",
        "second",
    );
    setup.set_attribute_ns(&html, Namespace::none(), "data-key", "third");
    setup.set_attribute_ns(&html, Namespace::none(), "data-", "empty");
    setup.set_attribute_ns(&html, Namespace::none(), "data-Upper", "excluded");
    let done = dom.apply(setup, &AtomicBool::new(false)).unwrap();
    let snapshot = dom.snapshot();
    for ticket in [&svg, &math] {
        let node = snapshot
            .node(done.created(ticket).unwrap().unwrap())
            .unwrap();
        assert_eq!(DomDataset::new(node).unwrap().element(), node.id());
    }
    assert!(
        DomDataset::new(
            snapshot
                .node(done.created(&other).unwrap().unwrap())
                .unwrap()
        )
        .is_none()
    );
    assert!(
        DomDataset::new(
            snapshot
                .node(done.document(&document).unwrap().node())
                .unwrap()
        )
        .is_none()
    );
    let node = snapshot
        .node(done.created(&html).unwrap().unwrap())
        .unwrap();
    (dom, DomDataset::new(node).unwrap())
}

fn limits(work: u64) -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 100_000, work, 128, 128, 4096, 1)
}

fn execute(
    dom: &mut Dom,
    edit: DomSourceEdit,
    work: u64,
) -> Result<surgeist_dom::source::DomSourceCompletion, DomError> {
    let cancel = AtomicBool::new(false);
    let mut lease = dom.begin_source_edit(edit, limits(work), &cancel)?;
    match lease.advance(&cancel)? {
        DomSourceProgress::Complete(done) => Ok(done),
        DomSourceProgress::Failed(failure) => Err(failure.error().clone()),
        DomSourceProgress::Request(request) => panic!("unexpected service request: {request:?}"),
    }
}

fn get<'a>(data: DomDataset, snapshot: &'a DomSnapshot, name: &str) -> Option<DomAttr<'a>> {
    data.get_with(snapshot, &units(name), || Ok::<_, DomError>(()))
        .unwrap()
}

#[test]
fn dataset_reads_the_first_local_name_collision_in_canonical_order() {
    let (mut dom, data) = fixture();
    let initial = dom.snapshot();
    let selected = get(data, &initial, "key").unwrap();
    assert_eq!(selected.value().code_units(), units("first"));
    let mut observed = Vec::new();
    data.visit_with(
        &initial,
        || Ok::<_, DomError>(()),
        |property, attr| {
            let mut name = Vec::new();
            property.visit_with(
                || Ok::<_, DomError>(()),
                |unit| {
                    name.push(unit);
                    Ok(())
                },
            )?;
            observed.push((name, attr.value().code_units().to_vec()));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        observed,
        [
            (units("key"), units("first")),
            (units("key"), units("second")),
            (units("key"), units("third")),
            (vec![], units("empty")),
        ]
    );
    assert!(get(data, &initial, "Upper").is_none());

    execute(
        &mut dom,
        DomSourceEdit::set_attribute_local_name(data.element(), "data-key", "updated"),
        1_000_000,
    )
    .unwrap();
    let snapshot = dom.snapshot();
    let attributes = snapshot.node(data.element()).unwrap().attributes().unwrap();
    assert_eq!(
        attributes
            .get_named_item_ns(&Namespace::none(), &DocumentString::from("data-key"))
            .unwrap()
            .value(),
        &DocumentString::from("updated")
    );
    assert_eq!(get(data, &snapshot, "key").unwrap().id(), selected.id());
    execute(
        &mut dom,
        DomSourceEdit::remove_attribute(data.element(), "data-key"),
        1_000_000,
    )
    .unwrap();
    let snapshot = dom.snapshot();
    let attributes = snapshot.node(data.element()).unwrap().attributes().unwrap();
    assert!(
        attributes
            .get_named_item_ns(
                &Namespace::from_uri("urn:second"),
                &DocumentString::from("data-key")
            )
            .is_none()
    );
    assert!(
        attributes
            .get_named_item_ns(&Namespace::none(), &DocumentString::from("data-key"))
            .is_some()
    );
    assert_eq!(get(data, &snapshot, "key").unwrap().id(), selected.id());
}

#[test]
fn exact_local_source_write_preserves_colon_and_retains_validation() {
    let (mut dom, data) = fixture();
    let element = data.element();
    let before = dom.snapshot().stamp();
    let wanted = [0xd800, 0, 0x3a];
    let done = execute(
        &mut dom,
        DomSourceEdit::set_attribute_local_name(
            element,
            "data-a:b",
            DocumentString::from_utf16(wanted),
        ),
        1_000_000,
    )
    .unwrap();
    assert_eq!(done.commit().changes().from(), before);
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::Attributes(element))
    );
    let first = dom.snapshot();
    let attribute = get(data, &first, "a:b").unwrap();
    assert_eq!(
        attribute.name().local_name().code_units(),
        units("data-a:b")
    );
    assert!(attribute.name().prefix().is_none());
    assert!(attribute.name().namespace().is_none());
    assert_eq!(attribute.value().code_units(), wanted);
    let identity = attribute.id();
    execute(
        &mut dom,
        DomSourceEdit::set_attribute_local_name(element, "data-a:b", "replacement"),
        1_000_000,
    )
    .unwrap();
    assert_eq!(get(data, &dom.snapshot(), "a:b").unwrap().id(), identity);

    let stamp = dom.snapshot().stamp();
    for (name, expected) in [
        ("data-bad=name", DomError::InvalidCharacter),
        ("onclick", DomError::SourceAttributeGlobalUnavailable),
    ] {
        assert_eq!(
            execute(
                &mut dom,
                DomSourceEdit::set_attribute_local_name(element, name, "value"),
                1_000_000
            )
            .unwrap_err(),
            expected
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert!(dom.pending_source_edit().is_none());
    }
    execute(
        &mut dom,
        DomSourceEdit::remove_attribute(element, "data-a:b"),
        1_000_000,
    )
    .unwrap();
    assert!(get(data, &dom.snapshot(), "a:b").is_none());
}

#[test]
fn dataset_lookup_and_exact_local_write_respect_work_refusal() {
    let (dom, data) = fixture();
    let snapshot = dom.snapshot();
    let spent = Cell::new(0);
    let name = units("key");
    let lookup = |limit| {
        spent.set(0);
        data.get_with(&snapshot, &name, || {
            if spent.get() == limit {
                return Err(DomError::SourceWorkLimit { limit });
            }
            spent.set(spent.get() + 1);
            Ok(())
        })
    };
    assert!(lookup(u64::MAX).unwrap().is_some());
    let exact = spent.get();
    assert!(
        matches!(lookup(exact - 1), Err(DomError::SourceWorkLimit {limit}) if limit == exact - 1)
    );
    assert!(lookup(exact).unwrap().is_some());

    let (mut dom, data) = fixture();
    let stamp = dom.snapshot().stamp();
    let usage = dom.usage();
    // Probe adjacent budgets against the same unchanged native owner. Each
    // refusal drops its lease before the next attempt.
    for work in 0..=4096 {
        let result = execute(
            &mut dom,
            DomSourceEdit::set_attribute_local_name(data.element(), "data-b:c", "proof"),
            work,
        );
        match result {
            Ok(done) => {
                assert!(work > 0);
                assert_eq!(done.commit().changes().from(), stamp);
                assert!(
                    done.commit()
                        .changes()
                        .changes()
                        .contains(&DomChange::Attributes(data.element()))
                );
                assert_eq!(
                    get(data, &dom.snapshot(), "b:c").unwrap().value(),
                    &DocumentString::from("proof")
                );
                assert!(dom.pending_source_edit().is_none());
                return;
            }
            Err(error) => {
                assert_eq!(error, DomError::SourceWorkLimit { limit: work });
                assert_eq!(dom.snapshot().stamp(), stamp);
                assert_eq!(dom.usage(), usage);
                assert!(dom.pending_source_edit().is_none());
                assert!(get(data, &dom.snapshot(), "b:c").is_none());
            }
        }
    }
    panic!("the bounded fixture never admitted the exact local-name write");
}
