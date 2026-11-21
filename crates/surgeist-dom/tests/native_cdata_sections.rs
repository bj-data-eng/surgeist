//! Native CDATA contracts from the pinned DOM Document, Node and Text algorithms.

use std::sync::atomic::{AtomicBool, Ordering};

use surgeist_dom::{
    Dom, DomBatch, DomError,
    edit::DomChange,
    limits::{DomLimits, DomNameLimits, DomResource},
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress},
    tree::{
        DomNodeIteratorFilter, DomNodeIteratorProgress, DomNodeKind, DomTreeWalkerDirection,
        DomTreeWalkerFilter, DomTreeWalkerProgress,
    },
};
use surgeist_html::values::DocumentString;

fn limits() -> DomLimits {
    DomLimits::new(
        64,
        16_384,
        128,
        0,
        0,
        16,
        1_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_ranges(4)
    .unwrap()
    .with_node_iterators(4)
    .with_tree_walkers(4)
}

#[test]
fn source_factory_checks_document_class_before_metered_delimiter_search() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let xml = batch.create_document();
    let html = batch.create_html_document(&xml, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let xml = commit.document(&xml).unwrap();
    let html = commit.document(&html).unwrap();
    let stamp = dom.snapshot().stamp();
    let usage = dom.usage();

    let source_limits =
        |work, outputs| DomSourceLimits::new(16_000_000, 200_000, work, 64, outputs, 64, 3);
    let long_invalid = "x".repeat(8192) + "]]>";
    for (document, data, work, outputs, error) in [
        (
            html,
            long_invalid.as_str(),
            1024,
            64,
            DomError::NotSupported,
        ),
        (
            xml,
            long_invalid.as_str(),
            1024,
            64,
            DomError::SourceWorkLimit { limit: 1024 },
        ),
        (xml, "x]]]>tail", 100_000, 64, DomError::InvalidCharacter),
        (xml, "valid", 100_000, 0, DomError::SourceOutputLimit),
    ] {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::create_cdata_section(document, data),
                source_limits(work, outputs),
                &cancel,
            )
            .unwrap();
        assert!(matches!(lease.advance(&cancel), Err(actual) if actual == error));
        drop(lease);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(dom.usage(), usage);
    }

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_cdata_section(xml, "valid"),
            source_limits(100_000, 64),
            &cancel,
        )
        .unwrap();
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(lease.advance(&cancel), Err(DomError::Cancelled)));
    drop(lease);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
    cancel.store(false, Ordering::Relaxed);

    // In a plain batch, the authored string alone bounds validation work.
    let mut batch = DomBatch::new();
    batch.create_cdata_section(xml, "x".repeat(16_385));
    assert!(matches!(
        dom.apply(batch, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), stamp);

    let units = [0xd800, u16::from(b']'), u16::from(b'>'), 0xdc00];
    let data = DocumentString::from_utf16(units.to_vec());
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_cdata_section(xml, data),
            source_limits(100_000, 64),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("detached CDATA creation did not complete");
    };
    let id = done.published_node().unwrap();
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::Created(id))
    );
    drop(lease);
    let snapshot = dom.snapshot();
    let node = snapshot.node(id).unwrap();
    assert!(matches!(node.kind(), DomNodeKind::CdataSection(value) if value.code_units() == units));
    assert_eq!(node.data().unwrap().code_units(), units);
    assert_eq!(node.node_name(), "#cdata-section".into());
    assert_eq!(node.owner_document(), Some(xml));
    assert_eq!(node.parent(), None);
    assert_eq!(dom.usage().managed_nodes(), usage.managed_nodes() + 1);
    assert_eq!(
        dom.usage().string_code_units(),
        usage.string_code_units() + units.len()
    );
}

#[test]
fn normalize_keeps_cdata_as_a_text_boundary_and_split_transfers_live_ranges() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let left = batch.create_text(&document, "L");
    let empty_cdata = batch.create_cdata_section(&document, "");
    let empty_text = batch.create_text(&document, "");
    let cdata = batch.create_cdata_section(
        &document,
        DocumentString::from_utf16(vec![0x0061, 0xd800, 0xdc00, 0x0062]),
    );
    let right = batch.create_text(&document, "R");
    let comment = batch.create_comment(&document, "stop");
    let separate = batch.create_text(&document, "!");
    batch.insert_before(&root, &document, None);
    for node in [
        &left,
        &empty_cdata,
        &empty_text,
        &cdata,
        &right,
        &comment,
        &separate,
    ] {
        batch.insert_before(node, &root, None);
    }
    let in_data = batch.create_range(&document);
    batch.set_range_start(&in_data, &cdata, 1);
    batch.set_range_end(&in_data, &cdata, 3);
    let after_data = batch.create_range(&document);
    batch.set_range_start(&after_data, &root, 4);
    batch.set_range_end(&after_data, &root, 4);
    let commit = dom.apply(batch, &cancel).unwrap();
    let in_data = commit.created_range(&in_data).unwrap().unwrap();
    let after_data = commit.created_range(&after_data).unwrap().unwrap();
    let node = |ticket| commit.created(ticket).unwrap().unwrap();
    let [
        root,
        left,
        empty_cdata,
        empty_text,
        cdata,
        right,
        comment,
        separate,
    ] = [
        &root,
        &left,
        &empty_cdata,
        &empty_text,
        &cdata,
        &right,
        &comment,
        &separate,
    ]
    .map(node);

    let mut batch = DomBatch::new();
    batch.normalize(root);
    dom.apply(batch, &cancel).unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot.node(root).unwrap().children(),
        [left, empty_cdata, cdata, right, comment, separate]
    );
    assert_eq!(snapshot.node(empty_text).unwrap().parent(), None);
    assert_eq!(snapshot.node(empty_cdata).unwrap().data().unwrap().len(), 0);
    let mut whole_text = Vec::new();
    snapshot
        .node(cdata)
        .unwrap()
        .visit_contiguous_text_with(
            || Ok::<_, DomError>(()),
            |value| {
                whole_text.extend_from_slice(value.code_units());
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(whole_text, [0x004c, 0x0061, 0xd800, 0xdc00, 0x0062, 0x0052]);
    let mut descendant_text = Vec::new();
    snapshot
        .node(root)
        .unwrap()
        .visit_descendant_text_with(
            || Ok::<_, DomError>(()),
            |_| Ok(()),
            |value| {
                descendant_text.extend_from_slice(value.code_units());
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        descendant_text,
        [0x004c, 0x0061, 0xd800, 0xdc00, 0x0062, 0x0052, 0x0021]
    );
    assert_eq!(snapshot.range(after_data).unwrap().end().offset(), 3);

    let mut batch = DomBatch::new();
    let tail = batch.split_text(cdata, 2);
    let commit = dom.apply(batch, &cancel).unwrap();
    let tail = commit.created(&tail).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot.node(root).unwrap().children(),
        [left, empty_cdata, cdata, tail, right, comment, separate]
    );
    assert!(
        matches!(snapshot.node(cdata).unwrap().kind(), DomNodeKind::CdataSection(value) if value.code_units() == [0x0061, 0xd800])
    );
    assert!(
        matches!(snapshot.node(tail).unwrap().kind(), DomNodeKind::Text(value) if value.code_units() == [0xdc00, 0x0062])
    );
    let range = snapshot.range(in_data).unwrap();
    assert_eq!(
        (range.start().container(), range.start().offset()),
        (cdata, 1)
    );
    assert_eq!((range.end().container(), range.end().offset()), (tail, 1));
    assert_eq!(snapshot.range(after_data).unwrap().end().offset(), 4);

    let mut batch = DomBatch::new();
    batch.replace_data(cdata, 1, usize::MAX, "]]>");
    let commit = dom.apply(batch, &cancel).unwrap();
    assert!(commit.changes().changes().contains(&DomChange::Data(cdata)));
    assert_eq!(
        dom.snapshot()
            .node(cdata)
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        [0x0061, 0x005d, 0x005d, 0x003e]
    );
}

#[test]
fn clone_captured_import_and_adoption_preserve_cdata_through_retirement() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let xml = batch.create_document();
    let html = batch.create_html_document(&xml, None);
    let cdata = batch.create_cdata_section(&xml, "first");
    // Mutation can produce a value the Document factory would reject.
    batch.replace_data(&cdata, 0, 5, "]]>");
    let commit = dom.apply(batch, &cancel).unwrap();
    let xml = commit.document(&xml).unwrap();
    let html = commit.document(&html).unwrap();
    let id = commit.created(&cdata).unwrap().unwrap();
    let historical = dom.snapshot();

    let mut batch = DomBatch::new();
    let copy = batch.clone_node(id, false);
    let import = batch.import_node(&historical, id, html, false);
    batch.adopt(id, html);
    let commit = dom.apply(batch, &cancel).unwrap();
    let copy = commit.created(&copy).unwrap().unwrap();
    let import = commit.created(&import).unwrap().unwrap();
    assert_ne!(id, copy);
    assert_ne!(id, import);
    assert_ne!(copy, import);
    let adopted = dom.snapshot();
    for (node, document) in [(id, html), (copy, xml), (import, html)] {
        let node = adopted.node(node).unwrap();
        assert_eq!(node.owner_document(), Some(document));
        assert!(
            matches!(node.kind(), DomNodeKind::CdataSection(data) if data == &DocumentString::from("]]>"))
        );
    }
    assert_eq!(historical.node(id).unwrap().owner_document(), Some(xml));

    let mut batch = DomBatch::new();
    batch.destroy(id);
    let commit = dom.apply(batch, &cancel).unwrap();
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::Destroyed(id))
    );
    assert!(matches!(dom.snapshot().node(id), Err(DomError::StaleNode)));
    assert_eq!(adopted.node(id).unwrap().owner_document(), Some(html));
    assert_eq!(historical.node(id).unwrap().owner_document(), Some(xml));
}

#[test]
fn cdata_has_a_distinct_filter_bit_and_cannot_become_a_document_child() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fragment = batch.create_fragment(&document);
    let cdata = batch.create_cdata_section(&document, "same");
    let text = batch.create_text(&document, "same");
    batch.insert_before(&text, &fragment, None);
    batch.insert_before(&cdata, &fragment, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let cdata = commit.created(&cdata).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert!(
        !snapshot
            .node(cdata)
            .unwrap()
            .is_equal_node_with(text, || Ok::<_, DomError>(()), |_| Ok(()),)
            .unwrap()
    );

    for invalid_child in [cdata, fragment] {
        let mut batch = DomBatch::new();
        batch.insert_before(invalid_child, document, None);
        assert!(matches!(
            dom.apply(batch, &cancel),
            Err(DomError::HierarchyRequest)
        ));
        assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
        assert_eq!(
            dom.snapshot().node(fragment).unwrap().children(),
            [text, cdata]
        );
    }

    for (mask, expected) in [(0x4, text), (0x8, cdata)] {
        let iterator = dom
            .create_node_iterator(fragment, mask, DomNodeIteratorFilter::None, &cancel)
            .unwrap();
        assert!(matches!(
            dom.node_iterator_next(iterator, 4096, &cancel).unwrap(),
            DomNodeIteratorProgress::Complete(Some(id)) if id == expected
        ));
        assert!(matches!(
            dom.node_iterator_next(iterator, 4096, &cancel).unwrap(),
            DomNodeIteratorProgress::Complete(None)
        ));

        let walker = dom
            .create_tree_walker(fragment, mask, DomTreeWalkerFilter::None, &cancel)
            .unwrap();
        let progress = dom
            .navigate_tree_walker(
                walker,
                DomTreeWalkerDirection::NextNode,
                || Ok::<_, DomError>(()),
                &cancel,
            )
            .unwrap();
        let DomTreeWalkerProgress::Match(request) = progress else {
            panic!("expected one Text-family traversal match");
        };
        assert_eq!(request.accept(&mut dom), Ok(expected));
        let progress = dom
            .navigate_tree_walker(
                walker,
                DomTreeWalkerDirection::NextNode,
                || Ok::<_, DomError>(()),
                &cancel,
            )
            .unwrap();
        assert!(matches!(progress, DomTreeWalkerProgress::NoMatch));
    }
}
