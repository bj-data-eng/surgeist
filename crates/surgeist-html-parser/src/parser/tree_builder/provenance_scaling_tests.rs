//! Retention oracle for shared text history, independent of physical layouts.
use super::{StandaloneDocumentDriver, entry::OrdinaryDocumentEntry, head::HeadStep};
use crate::{
    HtmlDocumentMode, HtmlDocumentParseRequest, HtmlParseResource, HtmlSource,
    construction::{
        ParserNode,
        native_view::{NativeKind, NativeString, NativeView},
    },
};

fn ascii(view: &mut NativeView<'_>, actual: NativeString<'_>, expected: &str) {
    assert!(expected.is_ascii());
    assert_eq!(actual.len(), expected.len());
    for (index, unit) in expected.bytes().enumerate() {
        assert_eq!(view.unit(&actual, index).unwrap(), Some(u16::from(unit)));
    }
    assert_eq!(view.unit(&actual, expected.len()).unwrap(), None);
}

fn child(view: &mut NativeView<'_>, parent: &ParserNode, index: usize) -> ParserNode {
    let node = view.child_at(parent, index).unwrap().unwrap();
    let actual_parent = view.parent(&node).unwrap().unwrap();
    assert!(view.same_node(&actual_parent, parent).unwrap());
    node
}

fn element(view: &mut NativeView<'_>, node: &ParserNode, local: &str, children: usize) {
    let NativeKind::Element(name) = view.kind(node).unwrap() else {
        panic!("expected HTML element {local}");
    };
    assert!(name.prefix.is_none());
    ascii(
        view,
        name.namespace.unwrap(),
        "http://www.w3.org/1999/xhtml",
    );
    ascii(view, name.local, local);
    assert_eq!(view.attribute_count(node).unwrap(), 0);
    assert_eq!(view.child_count(node).unwrap(), children);
    assert!(view.child_at(node, children).unwrap().is_none());
}

fn retained_text_units(input: &'static str, expected_text: &'static str) -> u64 {
    let request =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(input), super::tests::limits(1_000));
    let mut entry = OrdinaryDocumentEntry::new(&request);
    let mut driver = Some(entry.take_driver().unwrap());
    loop {
        match driver.as_mut().unwrap().step().unwrap() {
            HeadStep::Consumed => {}
            HeadStep::Complete => break,
            other => panic!("unexpected ordinary continuation {other:?}"),
        }
    }
    let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
    assert!(driver.is_none());
    assert_eq!(completed.diagnostics().unwrap().len(), 0);
    // Snapshot before observer/projection work: admitted retained payload
    // length, not capacity, allocator headers, RSS or cumulative work.
    let retained = completed
        .usage()
        .unwrap()
        .resource(HtmlParseResource::ManagedUtf16Units)
        .current;
    {
        let mut view = completed.native_view().unwrap();
        let root = view.root().unwrap();
        assert!(matches!(
            view.kind(&root).unwrap(),
            NativeKind::Document(HtmlDocumentMode::NoQuirks)
        ));
        assert!(view.parent(&root).unwrap().is_none());
        assert_eq!(view.child_count(&root).unwrap(), 2);
        assert!(view.child_at(&root, 2).unwrap().is_none());
        let doctype = child(&mut view, &root, 0);
        let NativeKind::Doctype {
            name,
            public_id,
            system_id,
        } = view.kind(&doctype).unwrap()
        else {
            panic!("expected actual doctype");
        };
        ascii(&mut view, name, "html");
        ascii(&mut view, public_id.unwrap(), "");
        ascii(&mut view, system_id.unwrap(), "");
        assert_eq!(view.child_count(&doctype).unwrap(), 0);
        let html = child(&mut view, &root, 1);
        element(&mut view, &html, "html", 2);
        let head = child(&mut view, &html, 0);
        element(&mut view, &head, "head", 0);
        let body = child(&mut view, &html, 1);
        element(&mut view, &body, "body", 1);
        let paragraph = child(&mut view, &body, 0);
        element(
            &mut view,
            &paragraph,
            "p",
            usize::from(!expected_text.is_empty()),
        );
        if !expected_text.is_empty() {
            let text = child(&mut view, &paragraph, 0);
            let NativeKind::Text(value) = view.kind(&text).unwrap() else {
                panic!("expected one actual text node");
            };
            ascii(&mut view, value, expected_text);
            assert_eq!(view.child_count(&text).unwrap(), 0);
        }
    }
    // Real completed-owner transfer/release; no alias, fake snapshot or
    // private counter mutation makes cleanup succeed.
    let mut frozen = completed.freeze_parts().unwrap();
    assert_eq!(frozen.diagnostics().unwrap().len(), 0);
    frozen.release().unwrap();
    assert_eq!(frozen.retained_usage().actual_capacity_bytes, 0);
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::ManagedEdges,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::SourceMapPayloadUnits,
        HtmlParseResource::ManagedCapacityBytes,
    ] {
        assert_eq!(frozen.retained_usage().resource(resource).current, 0);
    }
    retained
}

#[test]
fn adjacent_text_history_retains_only_unique_leaf_and_final_output_units() {
    let fixed = retained_text_units("<!doctype html><body><p></p>", "");
    let with_text = retained_text_units("<!doctype html><body><p>abcdefgh</p>", "abcdefgh");
    let added = with_text.checked_sub(fixed).unwrap();
    // Eight unique one-unit leaves plus one eight-unit current native value.
    // Historical merged values are reconstructible lineage, not another
    // materialized payload per prefix. Metadata belongs to other domains.
    let unique_leaf_units = 8;
    let final_output_units = 8;
    assert!(
        added <= unique_leaf_units + final_output_units,
        "retained text payload units {added} exceed unique leaves + final output (16); fixed={fixed}, with_text={with_text}"
    );
}
