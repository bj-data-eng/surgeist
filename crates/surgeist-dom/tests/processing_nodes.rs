use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentString, Dom, DomBatch, DomChange, DomError, DomLimits, DomNodeKind, DomResource,
};

// Each native Document retains application/xml, UTF-8 and about:blank.
const DOCUMENT_UNITS: usize = 31;

fn limits() -> DomLimits {
    DomLimits::new(
        64,
        2048,
        128,
        0,
        0,
        16,
        65536,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

#[test]
fn node_names_follow_each_exposed_node_kind_without_reinterpreting_authored_names() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fragment = batch.create_fragment(&document);
    let text = batch.create_text(&document, "text");
    let comment = batch.create_comment(&document, "comment");
    let pi = batch.create_processing_instruction(&document, ":π.Test", "data > allowed");
    let doctype = batch.create_document_type(&document, "Mixed:Name/=", "public", "system");
    let commit = apply(&mut dom, batch);
    let capture = dom.snapshot();
    for (ticket, expected) in [
        (&document, "#document"),
        (&fragment, "#document-fragment"),
        (&text, "#text"),
        (&comment, "#comment"),
        (&pi, ":π.Test"),
        (&doctype, "Mixed:Name/="),
    ] {
        let id = commit.created(ticket).unwrap().unwrap();
        assert_eq!(
            capture.node(id).unwrap().node_name(),
            DocumentString::from(expected)
        );
    }
    let doctype = capture
        .node(commit.created(&doctype).unwrap().unwrap())
        .unwrap();
    assert_eq!(doctype.data(), None);
    let DomNodeKind::DocumentType(value) = doctype.kind() else {
        panic!("expected doctype")
    };
    assert_eq!(value.public_id(), &DocumentString::from("public"));
    assert_eq!(value.system_id(), &DocumentString::from("system"));
}

#[test]
fn pi_targets_use_xml_name_ranges_and_do_not_apply_html_reserved_target_rules() {
    for target in [
        ":",
        "_",
        "xml",
        "XML-stylesheet",
        "a:b",
        "π",
        "\u{c0}",
        "\u{10000}",
        "\u{effff}",
        "a\u{b7}\u{300}\u{203f}9.-",
    ] {
        let mut dom = Dom::new(limits()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let pi = batch.create_processing_instruction(&document, target, "");
        let commit = apply(&mut dom, batch);
        assert_eq!(
            dom.snapshot()
                .node(commit.created(&pi).unwrap().unwrap())
                .unwrap()
                .node_name(),
            DocumentString::from(target)
        );
    }
    for target in [
        "",
        "1name",
        ".name",
        "-name",
        "a b",
        "a/b",
        "a?",
        "\u{b7}",
        "\u{300}",
        "\u{37e}",
        "\u{f0000}",
    ] {
        let mut dom = Dom::new(limits()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        batch.create_processing_instruction(&document, target, "");
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::InvalidCharacter,
            "{target:?}"
        );
        assert_eq!(dom.snapshot().nodes().count(), 0);
        assert_eq!(dom.revision().get(), 0);
    }
    for surrogate in [0xd800, 0xdfff] {
        let mut dom = Dom::new(limits()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        batch.create_processing_instruction(
            &document,
            DocumentString::from_utf16(vec![surrogate]),
            "",
        );
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::InvalidCharacter
        );
    }
}

#[test]
fn pi_initial_terminator_rejection_rolls_back_all_earlier_batch_work() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let document = apply(&mut dom, setup).document(&document).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    for data in ["?>", "prefix ?> suffix", "a='?>'"] {
        let mut batch = DomBatch::new();
        let comment = batch.create_comment(document, "must roll back");
        batch.insert_before(&comment, document, None);
        batch.create_processing_instruction(document, "target", data);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::InvalidCharacter
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.snapshot()
                .node(document.node())
                .unwrap()
                .children()
                .is_empty()
        );
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}

#[test]
fn pi_map_uses_shared_whole_string_rules_and_keeps_raw_data_exact() {
    let mut dom = Dom::new(limits()).unwrap();
    let raw = "a='A&amp;&#x1F600;' b=\"\"";
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let pi = batch.create_processing_instruction(&document, "style", raw);
    let commit = apply(&mut dom, batch);
    let capture = dom.snapshot();
    let DomNodeKind::ProcessingInstruction(value) = capture
        .node(commit.created(&pi).unwrap().unwrap())
        .unwrap()
        .kind()
    else {
        panic!("expected PI")
    };
    assert_eq!(value.target(), &DocumentString::from("style"));
    assert_eq!(value.data(), &DocumentString::from(raw));
    assert_eq!(
        value.attribute(&"a".into()),
        Some(&DocumentString::from("A&😀"))
    );
    assert_eq!(
        value.attribute(&"b".into()),
        Some(&DocumentString::from(""))
    );
    assert_eq!(value.attribute(&"A".into()), None);
    let names = value
        .attributes()
        .map(|(name, _)| name.to_utf8().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, ["a", "b"]);
    let raw_units = raw.encode_utf16().count();
    assert_eq!(
        dom.usage().string_code_units(),
        DOCUMENT_UNITS + 5 + raw_units + 1 + 4 + 1
    );
}

#[test]
fn invalid_pi_pseudo_attributes_clear_the_whole_map_without_rejecting_the_node() {
    for data in [
        "a='one' a='two'",
        "a='one' trailing",
        "a='&unknown;'",
        "a='one' b='&#0;'",
    ] {
        let mut dom = Dom::new(limits()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let pi = batch.create_processing_instruction(&document, "target", data);
        let commit = apply(&mut dom, batch);
        let capture = dom.snapshot();
        let DomNodeKind::ProcessingInstruction(value) = capture
            .node(commit.created(&pi).unwrap().unwrap())
            .unwrap()
            .kind()
        else {
            panic!("expected PI")
        };
        assert_eq!(value.data(), &DocumentString::from(data));
        assert_eq!(value.attributes().len(), 0);
        assert_eq!(
            dom.usage().string_code_units(),
            DOCUMENT_UNITS + 6 + data.encode_utf16().count()
        );
    }
}

#[test]
fn pi_character_data_replacement_updates_maps_and_preserves_old_snapshots() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let pi = batch.create_processing_instruction(&document, "target", "a='old'");
    let commit = apply(&mut dom, batch);
    let pi = commit.created(&pi).unwrap().unwrap();
    let old = dom.snapshot();
    let mut edit = DomBatch::new();
    edit.replace_data(pi, 0, usize::MAX, "b='new'");
    let report = apply(&mut dom, edit);
    assert_eq!(report.changes().changes(), [DomChange::Data(pi)]);
    assert_eq!(dom.revision().get(), old.revision().get() + 1);
    let current = dom.snapshot();
    let DomNodeKind::ProcessingInstruction(value) = current.node(pi).unwrap().kind() else {
        panic!("expected PI")
    };
    assert_eq!(value.attribute(&"a".into()), None);
    assert_eq!(
        value.attribute(&"b".into()),
        Some(&DocumentString::from("new"))
    );
    let DomNodeKind::ProcessingInstruction(previous) = old.node(pi).unwrap().kind() else {
        panic!("expected PI")
    };
    assert_eq!(
        previous.attribute(&"a".into()),
        Some(&DocumentString::from("old"))
    );
    assert_eq!(previous.attribute(&"b".into()), None);
    let mut edit = DomBatch::new();
    edit.replace_data(pi, 0, usize::MAX, "?>");
    apply(&mut dom, edit);
    let latest = dom.snapshot();
    let DomNodeKind::ProcessingInstruction(value) = latest.node(pi).unwrap().kind() else {
        panic!("expected PI")
    };
    assert_eq!(value.data(), &DocumentString::from("?>"));
    assert_eq!(value.attributes().len(), 0);
    assert_eq!(
        latest.node(pi).unwrap().node_name(),
        DocumentString::from("target")
    );
}

#[test]
fn pi_replacement_can_remove_old_map_capacity_at_the_exact_live_string_limit() {
    let mut dom = Dom::new(
        DomLimits::new(
            4,
            DOCUMENT_UNITS + 8,
            8,
            0,
            0,
            8,
            8192,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let pi = batch.create_processing_instruction(&document, "t", "a='1'");
    let pi = apply(&mut dom, batch).created(&pi).unwrap().unwrap();
    assert_eq!(dom.usage().string_code_units(), DOCUMENT_UNITS + 8);
    let mut edit = DomBatch::new();
    edit.replace_data(pi, 0, usize::MAX, "invalid");
    apply(&mut dom, edit);
    assert_eq!(dom.usage().string_code_units(), DOCUMENT_UNITS + 8);
    assert_eq!(
        dom.snapshot().node(pi).unwrap().data(),
        Some(&DocumentString::from("invalid"))
    );
}

#[test]
fn doctype_names_use_relaxed_rule_including_empty_and_identifiers_are_lossless() {
    for name in [
        DocumentString::from(""),
        "0a/b=?".into(),
        DocumentString::from_utf16(vec![0xd800]),
    ] {
        let mut dom = Dom::new(limits()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let public = DocumentString::from_utf16(vec![0, 0xd800, 0x3e]);
        let system = DocumentString::from_utf16(vec![0xdfff, 0x20]);
        let doctype =
            batch.create_document_type(&document, name.clone(), public.clone(), system.clone());
        let commit = apply(&mut dom, batch);
        let snapshot = dom.snapshot();
        let node = snapshot
            .node(commit.created(&doctype).unwrap().unwrap())
            .unwrap();
        assert_eq!(node.node_name(), name);
        let DomNodeKind::DocumentType(value) = node.kind() else {
            panic!("expected doctype")
        };
        assert_eq!(value.public_id(), &public);
        assert_eq!(value.system_id(), &system);
        assert_eq!(
            dom.usage().string_code_units(),
            DOCUMENT_UNITS + name.len() + 5
        );
    }
    for name in [
        "bad name",
        "bad\tname",
        "bad\nname",
        "bad\u{c}name",
        "bad\rname",
        "bad\0name",
        "bad>name",
    ] {
        let mut dom = Dom::new(limits()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        batch.create_document_type(&document, name, "", "");
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::InvalidCharacter
        );
        assert_eq!(dom.snapshot().nodes().count(), 0);
    }
}

#[test]
fn document_allows_pi_comments_and_one_doctype_but_fragment_rejects_doctype() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let before = batch.create_processing_instruction(&document, "before", "");
    let doctype = batch.create_document_type(&document, "html", "", "");
    let after = batch.create_comment(&document, "after");
    batch.insert_before(&before, &document, None);
    batch.insert_before(&doctype, &document, None);
    batch.insert_before(&after, &document, None);
    let report = apply(&mut dom, batch);
    let document = report.document(&document).unwrap();
    let doctype = report.created(&doctype).unwrap().unwrap();
    assert_eq!(
        dom.snapshot().node(document.node()).unwrap().children(),
        [
            report.created(&before).unwrap().unwrap(),
            doctype,
            report.created(&after).unwrap().unwrap()
        ]
    );
    let baseline = dom.snapshot().stamp();
    let mut edit = DomBatch::new();
    let second = edit.create_document_type(document, "second", "", "");
    edit.insert_before(&second, document, None);
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), baseline);
    let mut edit = DomBatch::new();
    let fragment = edit.create_fragment(document);
    edit.insert_before(doctype, &fragment, None);
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    let mut edit = DomBatch::new();
    edit.insert_before(doctype, document, Some(doctype.into()));
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), baseline);
    let mut edit = DomBatch::new();
    edit.detach(doctype);
    edit.insert_before(doctype, document, None);
    apply(&mut dom, edit);
    assert_eq!(
        dom.snapshot()
            .node(document.node())
            .unwrap()
            .children()
            .last(),
        Some(&doctype)
    );
}

#[test]
fn new_node_strings_and_derived_pi_map_are_admitted_atomically() {
    let mut dom = Dom::new(
        DomLimits::new(
            8,
            DOCUMENT_UNITS + 7,
            16,
            0,
            0,
            8,
            8192,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let document = apply(&mut dom, setup).document(&document).unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.create_processing_instruction(document, "t", "a='1'");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: DOCUMENT_UNITS + 7
        }
    );
    let mut batch = DomBatch::new();
    batch.create_document_type(document, "ab", "cde", "fgh");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: DOCUMENT_UNITS + 7
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage().managed_nodes(), 1);
    assert_eq!(dom.usage().string_code_units(), DOCUMENT_UNITS);
}

#[test]
fn pi_offsets_count_utf16_units_and_noop_replacement_preserves_revision() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let pi = batch.create_processing_instruction(
        &document,
        "t",
        DocumentString::from_utf16(vec![0x41, 0xd83d, 0xde00, 0xd800]),
    );
    let pi = apply(&mut dom, batch).created(&pi).unwrap().unwrap();
    let old = dom.snapshot();
    let mut edit = DomBatch::new();
    edit.replace_data(pi, 2, 1, DocumentString::from_utf16(vec![0xdfff]));
    apply(&mut dom, edit);
    let current = dom.snapshot();
    assert_eq!(
        current.node(pi).unwrap().data().unwrap().code_units(),
        [0x41, 0xd83d, 0xdfff, 0xd800]
    );
    assert_eq!(
        old.node(pi).unwrap().data().unwrap().code_units(),
        [0x41, 0xd83d, 0xde00, 0xd800]
    );
    assert_eq!(dom.usage().string_code_units(), DOCUMENT_UNITS + 5);
    let stamp = current.stamp();
    let mut edit = DomBatch::new();
    edit.replace_data(pi, 2, 1, DocumentString::from_utf16(vec![0xdfff]));
    assert!(apply(&mut dom, edit).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
}

#[test]
fn pi_map_growth_over_budget_preserves_prior_data_map_revision_and_history() {
    let mut dom = Dom::new(
        DomLimits::new(
            8,
            DOCUMENT_UNITS + 7,
            16,
            0,
            0,
            8,
            8192,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let pi = batch.create_processing_instruction(&document, "t", "x");
    let pi = apply(&mut dom, batch).created(&pi).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut edit = DomBatch::new();
    edit.replace_data(pi, 0, usize::MAX, "a='1'");
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: DOCUMENT_UNITS + 7
        }
    );
    assert_eq!(dom.usage(), usage);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot().node(pi).unwrap().data(),
        Some(&DocumentString::from("x"))
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn destroying_pi_and_doctype_reclaims_all_stored_strings_without_breaking_old_capture() {
    let mut dom = Dom::new(
        DomLimits::new(
            8,
            DOCUMENT_UNITS + 11,
            16,
            0,
            0,
            8,
            8192,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let pi = batch.create_processing_instruction(&document, "t", "a='1'");
    let doctype = batch.create_document_type(&document, "d", "p", "s");
    let commit = apply(&mut dom, batch);
    let pi = commit.created(&pi).unwrap().unwrap();
    let doctype = commit.created(&doctype).unwrap().unwrap();
    let old = dom.snapshot();
    assert_eq!(dom.usage().string_code_units(), DOCUMENT_UNITS + 11);
    let mut edit = DomBatch::new();
    edit.destroy(pi);
    edit.destroy(doctype);
    let report = apply(&mut dom, edit);
    assert_eq!(
        report.changes().changes(),
        [DomChange::Destroyed(pi), DomChange::Destroyed(doctype)]
    );
    assert_eq!(dom.usage().string_code_units(), DOCUMENT_UNITS);
    assert_eq!(dom.usage().managed_nodes(), 1);
    assert_eq!(dom.snapshot().node(pi).unwrap_err(), DomError::StaleNode);
    assert_eq!(
        old.node(doctype).unwrap().node_name(),
        DocumentString::from("d")
    );
    let DomNodeKind::ProcessingInstruction(value) = old.node(pi).unwrap().kind() else {
        panic!("expected PI")
    };
    assert_eq!(
        value.attribute(&"a".into()),
        Some(&DocumentString::from("1"))
    );
}
