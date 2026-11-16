//! The document a published report answers for the tree rooted at a node.
//!
//! A tree has the document of its root. The expectations come from the pinned HTML edition: the
//! contents of a template belong to the appropriate template contents owner document of the
//! template's document, one inert Document that every template of a Document shares
//! (`references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md`
//! lines 3015 to 3037), and the fragment a fragment parse makes has the node document of the
//! target (`references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md` line 6854).

use crate::parser::support::{fragment_all_with_options, fragment_context, limits, offline_all};
use surgeist_html::{
    construction::{HtmlNodeId, HtmlNodeKind},
    parsing::HtmlParserScriptingMode,
};
use surgeist_html_parser::{
    HtmlParseEffect, HtmlParseMetadataDocument, HtmlParseMetadataStep, HtmlParseReport,
    HtmlParseTreeDocumentError, HtmlSource,
};
use surgeist_html_semantics::HtmlFragmentContext;

fn document_parse(source: &str) -> HtmlParseReport {
    offline_all(HtmlSource::utf8(source), limits())
        .parse()
        .unwrap()
}

fn fragment_parse(source: &str, context: HtmlFragmentContext) -> HtmlParseReport {
    fragment_all_with_options(
        HtmlSource::utf8(source),
        context,
        limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap()
    .parse()
    .unwrap()
}

/// The roots of the contents of every template of the report, in the order of its metadata.
fn template_contents(report: &HtmlParseReport) -> Vec<HtmlNodeId> {
    let mut metadata = report.metadata();
    let mut contents = Vec::new();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(template) = record.template() {
                    contents.push(template.contents().id());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => return contents,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
}

/// The owned Document a tree answers for.
fn owned_document(report: &HtmlParseReport, root: &HtmlNodeId) -> HtmlNodeId {
    let Some(HtmlParseMetadataDocument::Owned(document)) =
        report.tree_document(root).unwrap().document()
    else {
        panic!("the tree names an owned Document");
    };
    assert!(matches!(document.kind(), HtmlNodeKind::Document(_)));
    document.id()
}

#[test]
fn a_document_root_is_its_own_document_and_costs_no_search() {
    let report = document_parse("<!doctype html><p>a<template><b>b</b></template>");
    let root = report.document().unwrap().root().id();
    let answer = report.tree_document(&root).unwrap();
    assert_eq!(answer.probes(), 0);
    assert_eq!(owned_document(&report, &root), root);
}

#[test]
fn a_node_with_a_parent_is_not_a_root() {
    let report = document_parse("<!doctype html><p>a<template><b>b</b></template><!--c-->");
    let document = report.document().unwrap();
    let mut pending = vec![document.root().id()];
    let mut asked = 0;
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        pending.extend(node.children());
        if node.parent().is_some() {
            asked += 1;
            assert_eq!(
                report.tree_document(&id).err(),
                Some(HtmlParseTreeDocumentError::NotRoot),
                "{:?}",
                node.kind()
            );
        }
    }
    // The doctype, html, head, body, p, the text, the template and the comment.
    assert_eq!(asked, 8);
}

#[test]
fn a_node_of_another_result_is_foreign() {
    let report = document_parse("<!doctype html><p>a");
    let other = document_parse("<!doctype html><p>a<template></template>");
    let root = other.document().unwrap().root().id();
    assert_eq!(
        report.tree_document(&root).err(),
        Some(HtmlParseTreeDocumentError::Foreign)
    );
    for contents in template_contents(&other) {
        assert_eq!(
            report.tree_document(&contents).err(),
            Some(HtmlParseTreeDocumentError::Foreign)
        );
    }
}

#[test]
fn every_template_of_a_document_has_its_contents_in_one_inert_document() {
    let report = document_parse(
        "<!doctype html><template><p>a</p></template><div><template><template>b</template></template></div>",
    );
    let main = report.document().unwrap().root().id();
    let contents = template_contents(&report);
    assert_eq!(contents.len(), 3);
    let inert = owned_document(&report, &contents[0]);
    assert_ne!(inert, main);
    for root in &contents {
        assert_eq!(owned_document(&report, root), inert);
        assert!(report.tree_document(root).unwrap().probes() >= 1);
    }
    // The inert Document is its own document.
    assert_eq!(owned_document(&report, &inert), inert);
}

#[test]
fn the_search_for_a_root_does_not_depend_on_the_depth_of_its_tree() {
    let probes = |depth: usize| {
        let source = format!(
            "<!doctype html><template>{}</template>",
            "<div>".repeat(depth)
        );
        let report = document_parse(&source);
        let contents = template_contents(&report);
        assert_eq!(contents.len(), 1);
        report.tree_document(&contents[0]).unwrap().probes()
    };
    let shallow = probes(1);
    assert_eq!(shallow, 1);
    for depth in [8, 32, 64] {
        assert_eq!(probes(depth), shallow, "depth {depth}");
    }
}

#[test]
fn the_fragment_a_fragment_parse_makes_has_the_document_of_its_target() {
    let context = fragment_context();
    let target = context.target_document().subject().clone();
    let report = fragment_parse("<p>a</p>b<template><i>c</i></template>", context);
    let fragment = report.fragment().unwrap();
    let Some(HtmlParseMetadataDocument::Captured(document)) = report
        .tree_document(&fragment.root().id())
        .unwrap()
        .document()
    else {
        panic!("the fragment has a captured document");
    };
    assert_eq!(document.id(), target);
    // The contents of a template in the fragment belong to the appropriate template contents
    // owner document of the target's document, which the parse creates: an owned inert Document,
    // and not the captured one.
    let contents = template_contents(&report);
    assert_eq!(contents.len(), 1);
    owned_document(&report, &contents[0]);
}

#[test]
fn a_declarative_shadow_root_has_the_document_of_its_host() {
    let context = fragment_context();
    let target = context.target_document().subject().clone();
    let report = fragment_parse(
        "<div><template shadowrootmode=open><p>a</p></template></div>",
        context,
    );
    let effects = report.effects().unwrap().unwrap();
    let mut shadows = 0;
    for index in 0..effects.len() {
        if let Some(HtmlParseEffect::AttachShadow { contents, .. }) = effects.effect(index).unwrap()
        {
            shadows += 1;
            let Some(HtmlParseMetadataDocument::Captured(document)) =
                report.tree_document(&contents.id()).unwrap().document()
            else {
                panic!("the shadow root has a captured document");
            };
            assert_eq!(document.id(), target);
        }
    }
    assert_eq!(shadows, 1);
}
