//! Pinned DOM ordered set and DOMTokenList algorithms over canonical attributes.

use std::cell::Cell;
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    Dom, DomBatch, DomError, DomSnapshot,
    edit::DomChange,
    limits::{DomLimits, DomNameLimits, DomResource},
    source::{
        DomSourceEdit, DomSourceLimits, DomSourceOutput, DomSourceProgress, DomTokenListEdit,
    },
    tree::DomTokenList,
};
use surgeist_html::{
    elements::token_list::HtmlTokenListReflection,
    values::{DocumentString, Namespace},
};

fn fixture() -> (Dom, DomTokenList) {
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
    let element = setup.create_element_ns(&document, Namespace::html(), "div");
    setup.set_attribute_ns(
        &element,
        Namespace::from_uri("urn:other"),
        "class",
        "foreign",
    );
    let done = dom.apply(setup, &AtomicBool::new(false)).unwrap();
    (
        dom,
        DomTokenList::class(done.created(&element).unwrap().unwrap()),
    )
}

fn limits(outputs: usize) -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 128, outputs, 4096, 1)
}

fn raw(snapshot: &DomSnapshot, list: DomTokenList) -> Option<&[u16]> {
    list.raw_with(snapshot, || Ok::<_, DomError>(()))
        .unwrap()
        .map(DocumentString::code_units)
}

fn set_raw(dom: &mut Dom, list: DomTokenList, value: impl Into<DocumentString>) {
    let mut edit = DomBatch::new();
    edit.set_attribute_ns(list.element(), Namespace::none(), "class", value);
    dom.apply(edit, &AtomicBool::new(false)).unwrap();
}

fn edit(
    dom: &mut Dom,
    list: DomTokenList,
    edit: DomTokenListEdit,
    limits: DomSourceLimits,
) -> Result<Option<DomSourceOutput>, DomError> {
    let cancel = AtomicBool::new(false);
    let mut lease =
        dom.begin_source_edit(DomSourceEdit::edit_token_list(list, edit), limits, &cancel)?;
    match lease.advance(&cancel)? {
        DomSourceProgress::Complete(done) => Ok(done.output()),
        DomSourceProgress::Failed(failure) => Err(failure.error().clone()),
        DomSourceProgress::Request(request) => panic!("unexpected service request: {request:?}"),
    }
}

fn stored(dom: &Dom, list: DomTokenList, expected: &str) {
    assert_eq!(
        raw(&dom.snapshot(), list).unwrap(),
        DocumentString::from(expected).code_units()
    );
}

#[test]
fn capture_is_exact_namespace_case_and_utf16_and_observes_attribute_removal() {
    let (mut dom, list) = fixture();
    assert!(raw(&dom.snapshot(), list).is_none());
    let raw_units = [
        0x20, 0x41, 0x09, 0x61, 0x0a, 0x41, 0x0c, 0xd800, 0xa0, 0xb, 0x0d,
    ];
    set_raw(&mut dom, list, DocumentString::from_utf16(raw_units));
    let before = dom.snapshot();
    let captured = list
        .capture_with(&before, || Ok::<_, DomError>(()), |_| Ok(()))
        .unwrap();
    assert_eq!(raw(&before, list), Some(raw_units.as_slice()));
    assert_eq!(captured.len(), 3);
    assert_eq!(captured.item(0), Some(&[0x41][..]));
    assert_eq!(captured.item(1), Some(&[0x61][..]));
    assert_eq!(captured.item(2), Some(&[0xd800, 0xa0, 0xb][..]));
    assert_eq!(captured.item(3), None);
    assert!(
        !list
            .contains_with(&before, &[0x41, 0x09, 0x61], || Ok::<_, DomError>(()))
            .unwrap()
    );
    let mut remove = DomBatch::new();
    remove.remove_attribute_ns(list.element(), Namespace::none(), "class");
    dom.apply(remove, &AtomicBool::new(false)).unwrap();
    assert!(raw(&dom.snapshot(), list).is_none());
    assert_eq!(captured.item(2), Some(&[0xd800, 0xa0, 0xb][..]));
    let current = dom.snapshot();
    let empty = list
        .capture_with(
            &current,
            || Ok::<_, DomError>(()),
            |_| -> Result<(), DomError> { panic!("an empty capture requires no token bank") },
        )
        .unwrap();
    assert!(empty.is_empty());
}

#[test]
fn updates_canonicalize_reached_writes_and_skip_prescribed_noops() {
    use DomTokenListEdit::{Add, Remove, Replace, Toggle};
    let (mut dom, list) = fixture();
    assert_eq!(
        edit(&mut dom, list, Remove(Vec::new()), limits(128)),
        Ok(None)
    );
    assert!(raw(&dom.snapshot(), list).is_none());

    set_raw(&mut dom, list, " a \t b c a ");
    let before = dom.snapshot();
    assert_eq!(
        edit(
            &mut dom,
            list,
            Toggle {
                token: "a".into(),
                force: Some(true)
            },
            limits(128)
        ),
        Ok(Some(DomSourceOutput::TokenListResult(true)))
    );
    assert_eq!(
        edit(
            &mut dom,
            list,
            Replace {
                token: "missing".into(),
                replacement: "a".into()
            },
            limits(128)
        ),
        Ok(Some(DomSourceOutput::TokenListResult(false)))
    );
    assert_eq!(before.stamp(), dom.snapshot().stamp());
    stored(&dom, list, " a \t b c a ");

    assert_eq!(
        edit(&mut dom, list, Add(vec!["a".into()]), limits(128)),
        Ok(None)
    );
    stored(&dom, list, "a b c");
    assert_eq!(
        edit(
            &mut dom,
            list,
            Replace {
                token: "a".into(),
                replacement: "c".into()
            },
            limits(128)
        ),
        Ok(Some(DomSourceOutput::TokenListResult(true)))
    );
    stored(&dom, list, "c b");
    set_raw(&mut dom, list, "c b a");
    assert_eq!(
        edit(
            &mut dom,
            list,
            Replace {
                token: "a".into(),
                replacement: "c".into()
            },
            limits(128)
        ),
        Ok(Some(DomSourceOutput::TokenListResult(true)))
    );
    stored(&dom, list, "c b");
    assert_eq!(
        edit(
            &mut dom,
            list,
            Toggle {
                token: "b".into(),
                force: None
            },
            limits(128)
        ),
        Ok(Some(DomSourceOutput::TokenListResult(false)))
    );
    stored(&dom, list, "c");
    assert_eq!(
        edit(&mut dom, list, Remove(vec!["c".into()]), limits(128)),
        Ok(None)
    );
    stored(&dom, list, "");

    let snapshot = dom.snapshot();
    let namespace = Namespace::from_uri("urn:other");
    let attrs = snapshot.node(list.element()).unwrap().attributes().unwrap();
    assert_eq!(
        attrs
            .get_named_item_ns(&namespace, &DocumentString::from("class"))
            .unwrap()
            .value(),
        &DocumentString::from("foreign")
    );
}

#[test]
fn validation_output_and_cancellation_refusal_leave_no_partial_edit() {
    use DomTokenListEdit::{Add, Replace, Toggle};
    let (mut dom, list) = fixture();
    set_raw(&mut dom, list, " a a ");
    let before = dom.snapshot();
    let usage = dom.usage();
    for (input, outputs, expected) in [
        (
            Add(vec!["valid".into(), " ".into(), "".into()]),
            128,
            DomError::TokenContainsAsciiWhitespace,
        ),
        (
            Replace {
                token: " ".into(),
                replacement: "".into(),
            },
            128,
            DomError::EmptyToken,
        ),
        (
            Toggle {
                token: "b".into(),
                force: None,
            },
            0,
            DomError::SourceOutputLimit,
        ),
    ] {
        assert_eq!(edit(&mut dom, list, input, limits(outputs)), Err(expected));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(dom.pending_source_edit().is_none());
    }
    let cancel = AtomicBool::new(false);
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::edit_token_list(
                    list,
                    Toggle {
                        token: "b".into(),
                        force: None,
                    },
                ),
                limits(128),
                &cancel,
            )
            .unwrap();
        lease.request_cancel();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("cancelled unpublished edit completed without a service");
        };
        assert!(done.cancellation_requested());
        assert_eq!(done.output(), None);
    }
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(
        edit(
            &mut dom,
            list,
            Toggle {
                token: "b".into(),
                force: None
            },
            limits(128)
        ),
        Ok(Some(DomSourceOutput::TokenListResult(true)))
    );
    stored(&dom, list, "a b");
}

#[test]
fn toggle_needs_exact_source_work_and_releases_one_short_attempt() {
    let initial = " a \t b a ";
    let setup = || {
        let (mut dom, list) = fixture();
        set_raw(&mut dom, list, initial);
        (dom, list)
    };
    let toggle = || DomTokenListEdit::Toggle {
        token: "c".into(),
        force: None,
    };
    let allowance = |bytes, work| DomSourceLimits::new(bytes, 100_000, work, 128, 128, 4096, 1);
    let expected = Some(DomSourceOutput::TokenListResult(true));
    let ceiling = limits(128);
    let reaches = |bytes, work| {
        let (mut dom, list) = setup();
        edit(&mut dom, list, toggle(), allowance(bytes, work))
    };
    let first_admitted = |ceiling, admits: &dyn Fn(u64) -> bool| {
        assert!(
            admits(ceiling),
            "the calibration ceiling must admit the write"
        );
        let (mut low, mut high) = (0, ceiling);
        while low < high {
            let middle = low + (high - low) / 2;
            if admits(middle) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        low
    };

    let retained_ceiling = ceiling.retained_bytes();
    let work = first_admitted(
        ceiling.work(),
        &|work| match reaches(retained_ceiling, work) {
            Ok(output) => {
                assert_eq!(output, expected);
                true
            }
            Err(error) => {
                assert_eq!(error, DomError::SourceWorkLimit { limit: work });
                false
            }
        },
    );
    assert!(work > 0);
    let peak = first_admitted(u64::try_from(retained_ceiling).unwrap(), &|bytes| {
        let bytes = usize::try_from(bytes).unwrap();
        match reaches(bytes, work) {
            Ok(output) => {
                assert_eq!(output, expected);
                true
            }
            Err(error) => {
                assert_eq!(
                    error,
                    DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: bytes,
                    }
                );
                false
            }
        }
    });
    let exact = allowance(usize::try_from(peak).unwrap(), work);
    let cancel = AtomicBool::new(false);
    let completes = |dom: &mut Dom, list: DomTokenList| {
        let before = dom.snapshot().stamp();
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::edit_token_list(list, toggle()),
                exact,
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("the exact work and backing must publish the serialized attribute");
        };
        assert_eq!(done.output(), expected);
        assert!(!done.cancellation_requested());
        assert_eq!(done.commit().changes().from(), before);
        assert!(
            done.commit()
                .changes()
                .changes()
                .contains(&DomChange::Attributes(list.element()))
        );
        drop(lease);
        stored(dom, list, "a b c");
        assert!(dom.pending_source_edit().is_none());
    };

    let (mut admitted, list) = setup();
    completes(&mut admitted, list);

    let (mut refused, list) = setup();
    let before = refused.snapshot().stamp();
    let usage = refused.usage();
    {
        let short = allowance(retained_ceiling, work - 1);
        let mut lease = refused
            .begin_source_edit(
                DomSourceEdit::edit_token_list(list, toggle()),
                short,
                &cancel,
            )
            .unwrap();
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            DomError::SourceWorkLimit { limit: work - 1 }
        );
        assert!(!lease.status().has_published());
        assert_eq!(lease.status().published_phases(), 0);
        assert_eq!(lease.snapshot().stamp(), before);
    }
    assert_eq!(refused.snapshot().stamp(), before);
    assert_eq!(refused.usage(), usage);
    assert!(refused.pending_source_edit().is_none());
    stored(&refused, list, initial);
    // Retry with only the independently measured peak backing. Stranded
    // Source scratch would exhaust this allowance.
    completes(&mut refused, list);
}

struct Backing<'a> {
    bytes: usize,
    live: &'a Cell<usize>,
}

impl Drop for Backing<'_> {
    fn drop(&mut self) {
        self.live.set(self.live.get() - self.bytes);
    }
}

#[test]
fn capture_growth_admits_overlap_and_releases_both_banks_on_failure() {
    let (mut dom, list) = fixture();
    set_raw(&mut dom, list, "a b a");
    let snapshot = dom.snapshot();
    let slot = size_of::<&[u16]>();
    let live = Cell::new(0);
    let try_capture = |limit| {
        list.capture_with(
            &snapshot,
            || Ok::<_, DomError>(()),
            |bytes| {
                let total = live.get() + bytes;
                if total > limit {
                    return Err(DomError::AllocationFailed);
                }
                live.set(total);
                Ok(Backing { bytes, live: &live })
            },
        )
    };
    assert!(matches!(
        try_capture(3 * slot - 1),
        Err(DomError::AllocationFailed)
    ));
    assert_eq!(live.get(), 0);
    let capture = try_capture(3 * slot).unwrap();
    assert_eq!(capture.len(), 2);
    assert_eq!(live.get(), 2 * slot);
    drop(capture);
    assert_eq!(live.get(), 0);
}

#[test]
fn html_associations_use_the_actual_family_and_exact_content_attribute() {
    use HtmlTokenListReflection::*;
    let (mut dom, _) = fixture();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let script = setup.create_element_ns(&document, Namespace::html(), "script");
    let link = setup.create_element_ns(&document, Namespace::html(), "link");
    let style = setup.create_element_ns(&document, Namespace::html(), "style");
    let anchor = setup.create_element_ns(&document, Namespace::html(), "a");
    let area = setup.create_element_ns(&document, Namespace::html(), "area");
    let form = setup.create_element_ns(&document, Namespace::html(), "form");
    let iframe = setup.create_element_ns(&document, Namespace::html(), "iframe");
    let output = setup.create_element_ns(&document, Namespace::html(), "output");
    let svg_link = setup.create_element_ns(&document, Namespace::svg(), "link");
    let foreign_namespace = Namespace::from_uri("urn:attribute");
    setup.set_attribute_ns(&link, foreign_namespace.clone(), "p:rel", "untouched");
    let created = dom.apply(setup, &AtomicBool::new(false)).unwrap();
    let snapshot = dom.snapshot();
    let node = |ticket| {
        snapshot
            .node(created.created(ticket).unwrap().unwrap())
            .unwrap()
    };
    assert!(DomTokenList::reflect_html(node(&svg_link), LinkRel).is_none());
    assert!(DomTokenList::reflect_html(node(&script), StyleBlocking).is_none());
    let link_classes = DomTokenList::class(node(&link).id());
    assert_eq!(link_classes.html_reflection(), None);

    let cases = [
        (&script, ScriptBlocking, "blocking"),
        (&link, LinkBlocking, "blocking"),
        (&style, StyleBlocking, "blocking"),
        (&anchor, AnchorRel, "rel"),
        (&area, AreaRel, "rel"),
        (&form, FormRel, "rel"),
        (&link, LinkRel, "rel"),
        (&link, LinkSizes, "sizes"),
        (&iframe, IframeSandbox, "sandbox"),
        (&output, OutputHtmlFor, "for"),
    ];
    for (ticket, reflection, attribute) in cases {
        let list = DomTokenList::reflect_html(node(ticket), reflection).unwrap();
        assert_eq!(list.html_reflection(), Some(reflection));
        assert_eq!(list.element(), node(ticket).id());
        assert!(
            list.attribute_name()
                .iter()
                .copied()
                .eq(attribute.encode_utf16())
        );
        assert_eq!(raw(&dom.snapshot(), list), None);
        assert_eq!(
            edit(
                &mut dom,
                list,
                DomTokenListEdit::Add(vec![attribute.into()]),
                limits(128)
            ),
            Ok(None)
        );
        stored(&dom, list, attribute);
        assert_eq!(
            dom.snapshot()
                .node(list.element())
                .unwrap()
                .attributes()
                .unwrap()
                .get_named_item_ns(&Namespace::none(), &DocumentString::from(attribute))
                .unwrap()
                .value(),
            &DocumentString::from(attribute)
        );
    }
    let snapshot = dom.snapshot();
    let attrs = snapshot
        .node(link_classes.element())
        .unwrap()
        .attributes()
        .unwrap();
    assert_eq!(
        attrs
            .get_named_item_ns(&foreign_namespace, &DocumentString::from("rel"))
            .unwrap()
            .value(),
        &DocumentString::from("untouched")
    );
    assert_eq!(raw(&snapshot, link_classes), None);
}
