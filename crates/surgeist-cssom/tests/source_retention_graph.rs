//! Actual SourceSnapshot identity and lifetime across retained CSSOM payloads.
//! Byte costs come from CSS's public storage authority, not a copied formula.

use std::sync::Arc;
use surgeist_css::{
    CssDeclarationList, CssMediaQueryList, CssMediaRule as AuthoredMediaRule, CssNamespaceContext,
    CssParseReport, CssParserContext, CssRule as AuthoredRule, CssSheet, CssStylesheetInputLimits,
    CssValueOrigin,
};
use surgeist_cssom::*;

fn parsed(original: Arc<Vec<u16>>) -> CssParseReport<CssSheet> {
    CssParserContext::default()
        .parse_sheet_utf16_with_limits(original, CssStylesheetInputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap()
}
fn backing(text: &str) -> Arc<Vec<u16>> {
    let mut units = Vec::with_capacity(4096);
    units.extend(text.encode_utf16());
    Arc::new(units)
}
fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "sheet".into(),
            revision: 1,
        },
        true,
    )
}
fn owner(bytes: usize) -> CssomStore {
    CssomStore::new(
        CssomLimits {
            max_retained_source_bytes: bytes,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap()
}
fn batch(owner: &CssomStore) -> CssomBatch {
    owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot()))
        .unwrap()
}
fn rootless_graph(report: &CssParseReport<CssSheet>) -> CssSheet {
    let child = report.syntax().rules()[0].clone();
    let group = AuthoredMediaRule::try_new(
        CssMediaQueryList::new(Vec::new()),
        vec![child.clone(), child],
        &CssNamespaceContext::default(),
    )
    .unwrap();
    let graph = CssSheet::try_from_rules(vec![AuthoredRule::Media(group)]).unwrap();
    assert!(graph.parse_source().is_none());
    graph
}
fn create(owner: &mut CssomStore, graph: CssSheet) -> CssomSheetId {
    let mut edit = batch(owner);
    let ticket = edit.create_typed_sheet(graph, inputs()).unwrap();
    owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn assert_source_limit(result: Result<CssomSheetTicket, CssomError>, maximum: usize) {
    assert!(
        matches!(result, Err(CssomError::Limit { resource: "retained parse source bytes", maximum: actual }) if actual == maximum)
    );
}

#[test]
fn composed_descendants_without_a_root_source_are_charged_once_per_actual_snapshot() {
    let report = parsed(backing("/* entire original retained */ .a { --x: token; }"));
    let source = report.syntax().parse_source().unwrap().source();
    let bytes = source.storage_usage().retained_bytes();
    let graph = rootless_graph(&report);
    let mut short = owner(bytes - 1);
    let before = short.snapshot();
    let mut edit = batch(&short);
    assert_source_limit(edit.create_typed_sheet(graph.clone(), inputs()), bytes - 1);
    assert!(matches!(short.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(short.snapshot().revision(), before.revision());

    // Repeated descendants and multiple canonical sheets retain the SAME source.
    let mut exact = owner(bytes);
    let first = create(&mut exact, graph.clone());
    let second = create(&mut exact, graph);
    assert_ne!(first, second);
    assert_eq!(exact.snapshot().sheets(), &[first, second]);

    let separate = parsed(backing("/* entire original retained */ .a { --x: token; }"));
    let other = separate.syntax().parse_source().unwrap().source();
    assert_eq!(source.as_str(), other.as_str());
    assert!(!source.same_snapshot(other));
    let before = exact.snapshot();
    let mut edit = batch(&exact);
    assert_source_limit(
        edit.create_typed_sheet(rootless_graph(&separate), inputs()),
        bytes,
    );
    assert!(matches!(exact.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(exact.snapshot().revision(), before.revision());
    assert_eq!(exact.snapshot().sheets(), before.sheets());
}

#[test]
fn constructed_media_with_only_a_parsed_prelude_still_retains_the_whole_source() {
    let report = parsed(backing(
        "/* prelude origin owns this backing */ @media screen and (width > 1px) {}",
    ));
    let source = report.syntax().parse_source().unwrap().source();
    let bytes = source.storage_usage().retained_bytes();
    let AuthoredRule::Media(media) = &report.syntax().rules()[0] else {
        panic!("literal media rule");
    };
    let CssValueOrigin::Parsed(origin) = media.query().queries()[0].origin() else {
        panic!("parsed prelude must expose its real source");
    };
    assert!(origin.source().same_snapshot(source));
    let wrapper = AuthoredMediaRule::try_new(
        media.query().clone(),
        Vec::new(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    let graph = CssSheet::try_from_rules(vec![AuthoredRule::Media(wrapper)]).unwrap();
    assert!(graph.parse_source().is_none());
    let mut short = owner(bytes - 1);
    let mut edit = batch(&short);
    assert_source_limit(edit.create_typed_sheet(graph.clone(), inputs()), bytes - 1);
    assert!(matches!(short.commit(edit), Err(CssomError::BatchAborted)));
    let mut exact = owner(bytes);
    let sheet = create(&mut exact, graph);
    let snapshot = exact.snapshot();
    assert_eq!(snapshot.sheet_rules(&sheet).unwrap().len(), 1);
}

#[test]
fn detached_objects_keep_their_source_charge_and_captures_keep_original_input_alive() {
    let original = backing("/* original shared units */ .a { --x: token; }");
    let weak = Arc::downgrade(&original);
    let report = parsed(original.clone());
    let bytes = report
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .storage_usage()
        .retained_bytes();
    let graph = rootless_graph(&report);
    let mut owner = owner(bytes);
    let sheet = create(&mut owner, graph);
    let snapshot = owner.snapshot();
    let root = snapshot.sheet_rules(&sheet).unwrap()[0].clone();
    let captured = owner.css_rule(&root).unwrap().capture(&snapshot).unwrap();
    drop(snapshot);
    drop(report);
    drop(original);
    assert!(weak.upgrade().is_some());

    let mut edit = batch(&owner);
    edit.delete_sheet_rule(&sheet, 0).unwrap();
    edit.remove_sheet(&sheet).unwrap();
    owner.commit(edit).unwrap();
    let detached = owner.snapshot();
    assert!(detached.sheets().is_empty());
    assert!(detached.sheet_rules(&sheet).unwrap().is_empty());
    assert_eq!(detached.parent_style_sheet(&root).unwrap(), None);
    assert!(detached.rule(&root).is_ok());
    let separate = parsed(backing("/* different retained parse */ .b { --x: value; }"));
    let mut edit = batch(&owner);
    assert_source_limit(
        edit.create_typed_sheet(rootless_graph(&separate), inputs()),
        bytes,
    );
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), detached.revision());
    drop(detached);
    drop(owner);
    // This exact caller-supplied backing now survives through the capture alone.
    assert!(weak.upgrade().is_some());
    assert!(captured.snapshot().rule(&root).is_ok());
    drop(captured);
    assert!(weak.upgrade().is_none());
}

fn empty_block(owner: &mut CssomStore) -> CssomBlockId {
    let mut edit = batch(owner);
    let ticket = edit
        .create_declarations(
            CssDeclarationList::try_new(Vec::new()).unwrap(),
            false,
            false,
            None,
        )
        .unwrap();
    owner.commit(edit).unwrap().block(&ticket).unwrap().clone()
}
#[test]
fn a_current_property_edit_cannot_hide_its_parsed_source_behind_an_empty_authored_block() {
    let mut control = owner(1_000_000);
    let block = empty_block(&mut control);
    let handle = control.css_style_declaration(&block).unwrap();
    let outcome = handle
        .set_property(
            &mut control,
            "--x",
            "token",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| Ok(CssomAdmission::new(Vec::new(), Vec::new())),
        )
        .unwrap();
    assert!(outcome.owner_effects().is_empty());
    drop(outcome);
    let snapshot = control.snapshot();
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(selected),
        ..
    } = snapshot.block(&block).unwrap().data()
    else {
        panic!("current custom property must be retained");
    };
    assert_eq!(selected.entries().len(), 1);
    // This is an actual retained SourceSnapshot, not merely a newly owned string.
    assert!(selected.entries()[0].source().parsed_value().is_some());
    assert_eq!(
        handle
            .get_property_value(&control, "--x", CssomDeclarationRequestLimits::default())
            .unwrap(),
        "token"
    );

    let mut restricted = owner(0);
    let block = empty_block(&mut restricted);
    let before = restricted.snapshot();
    let handle = restricted.css_style_declaration(&block).unwrap();
    assert!(matches!(
        handle.set_property(
            &mut restricted,
            "--x",
            "token",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| Ok(CssomAdmission::new(Vec::new(), Vec::new()))
        ),
        Err(CssomError::Limit {
            resource: "retained parse source bytes",
            maximum: 0
        })
    ));
    assert_eq!(restricted.snapshot().revision(), before.revision());
    assert_eq!(
        handle
            .get_property_value(&restricted, "--x", CssomDeclarationRequestLimits::default())
            .unwrap(),
        ""
    );
}
