use super::discovery_observation::Probe;
use super::*;
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};

fn policy() -> DomLimits {
    DomLimits::new(512, 131072, 2048, 512, 256, 16, 1_000_000)
        .unwrap()
        .with_select_processing(128, 8_000_000, 10_000_000)
        .unwrap()
        .with_select_value_queries(
            HtmlCaptureLimits::new(512, 2048, 2048, 131072, 512, 0, 0),
            HtmlSemanticLimits::new(10_000_000, 1, 2048, 128, 8_000_000, 2048),
        )
}
fn selected(dom: &Dom, option: NodeId) -> bool {
    dom.snapshot()
        .node(option)
        .unwrap()
        .as_option()
        .unwrap()
        .state()
        .flags()
        .selectedness()
}
fn options_case(unrelated: usize, has_option: bool) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(policy()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let select = setup.create_element_ns(&document, crate::Namespace::html(), "select");
    // Multiple makes selectedcontent refresh an immediate semantic no-op, so
    // its separate target-discovery path cannot account for these observations.
    setup.set_attribute(&select, "multiple", "");
    for _ in 0..unrelated {
        let node = setup.create_comment(&document, "unrelated option candidate");
        setup.insert_before(&node, &select, None);
    }
    let option = has_option.then(|| {
        let option = setup.create_element_ns(&document, crate::Namespace::html(), "option");
        setup.set_attribute(&option, "selected", "");
        setup.set_attribute(&option, "value", "literal");
        setup.insert_before(&option, &select, None);
        option
    });
    let commit = dom.apply(setup, &cancel).unwrap();
    let select = commit.created(&select).unwrap().unwrap();
    let option = option
        .as_ref()
        .map(|option| commit.created(option).unwrap().unwrap());
    let before = dom.snapshot();
    let probe = Probe::start();
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(select, -1);
    dom.apply(edit, &cancel).unwrap();
    let counts = probe.finish();
    if let Some(option) = option {
        assert!(
            before
                .node(option)
                .unwrap()
                .as_option()
                .unwrap()
                .state()
                .flags()
                .selectedness()
        );
        assert!(!selected(&dom, option));
    } else {
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    assert_eq!(
        counts.options, 1,
        "the public setter obtained its actual option list"
    );
    assert_eq!(counts.option_members, usize::from(has_option));
    assert_eq!(counts.targets, 0, "multiple suppresses target discovery");
    assert!(
        counts.option_candidates <= 8,
        "fixed options visited {} candidates with {unrelated} unrelated descendants",
        counts.option_candidates
    );
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; original-path discovery regression"]
fn no_options_does_not_discover_unrelated_select_descendants() {
    options_case(0, false);
    options_case(64, false);
}
#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; original-path discovery regression"]
fn one_option_does_not_discover_unrelated_select_descendants() {
    options_case(0, true);
    options_case(64, true);
}

fn targets_case(unrelated: usize, target_count: usize) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(policy()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let select = setup.create_element_ns(&document, crate::Namespace::html(), "select");
    let chosen = setup.create_element_ns(&document, crate::Namespace::html(), "option");
    setup.set_attribute(&chosen, "selected", "");
    setup.set_attribute(&chosen, "value", "chosen");
    let text = setup.create_text(&document, "chosen");
    setup.insert_before(&text, &chosen, None);
    setup.insert_before(&chosen, &select, None);
    let blocker = setup.create_element_ns(&document, crate::Namespace::html(), "option");
    setup.set_attribute(&blocker, "value", "unselected");
    setup.insert_before(&blocker, &select, None);
    let mut disabled = Vec::new();
    for _ in 0..unrelated {
        let comment = setup.create_comment(&document, "unrelated target candidate");
        setup.insert_before(&comment, &select, None);
        let target =
            setup.create_element_ns(&document, crate::Namespace::html(), "selectedcontent");
        // HTML's actual Option ancestor disables this target. It is outside
        // the chosen Option, so it does not enlarge the cloned payload.
        setup.insert_before(&target, &blocker, None);
        disabled.push(target);
    }
    let mut targets = Vec::new();
    for _ in 0..target_count {
        let target =
            setup.create_element_ns(&document, crate::Namespace::html(), "selectedcontent");
        setup.insert_before(&target, &select, None);
        targets.push(target);
    }
    let commit = dom.apply(setup, &cancel).unwrap();
    let select = commit.created(&select).unwrap().unwrap();
    let targets = targets
        .iter()
        .map(|target| commit.created(target).unwrap().unwrap())
        .collect::<Vec<_>>();
    for target in disabled {
        assert!(
            dom.snapshot()
                .node(commit.created(&target).unwrap().unwrap())
                .unwrap()
                .as_selectedcontent()
                .unwrap()
                .disabled()
        );
    }
    let before = dom.snapshot();
    let probe = Probe::start();
    // Exercise the original reached scheduler alone: option-list discovery is
    // a separate observed domain and is absent when no target exists.
    let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
    let key = draft.resolve(select.into()).unwrap();
    draft
        .schedule_selectedcontent(key, policy(), &cancel)
        .unwrap();
    let prepared = draft.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    let counts = probe.finish();
    for target in targets {
        let snapshot = dom.snapshot();
        let children = snapshot.node(target).unwrap().children();
        assert_eq!(children.len(), 1);
        assert_eq!(
            snapshot.node(children[0]).unwrap().kind(),
            &DomNodeKind::Text("chosen".into())
        );
    }
    if target_count == 0 {
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    assert_eq!(counts.targets, 1);
    assert_eq!(counts.target_members, target_count);
    assert_eq!(
        counts.options, target_count,
        "each enabled target obtains a fresh current option list"
    );
    assert_eq!(
        counts.selection_reads, target_count,
        "each target actually observes the first selected Option"
    );
    assert!(
        counts.target_candidates <= target_count + 4,
        "{target_count} enabled targets visited {} candidates with {unrelated} unrelated comments/disabled targets",
        counts.target_candidates
    );
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; original-path discovery regression"]
fn absent_enabled_selectedcontent_does_not_discover_unrelated_descendants() {
    targets_case(0, 0);
    targets_case(64, 0);
}
#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; original-path discovery regression"]
fn fixed_enabled_selectedcontent_targets_do_not_discover_unrelated_descendants() {
    targets_case(0, 2);
    targets_case(64, 2);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; original-path mapping regression"]
fn option_capture_child_identity_mapping_does_not_rescan_prior_rows() {
    let cancel = AtomicBool::new(false);
    for comments in [0, 64] {
        let mut dom = Dom::new(policy()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let select = setup.create_element_ns(&document, crate::Namespace::html(), "select");
        setup.set_attribute(&select, "multiple", "");
        let option = setup.create_element_ns(&document, crate::Namespace::html(), "option");
        let first = setup.create_text(&document, " \ta\n");
        setup.insert_before(&first, &option, None);
        for _ in 0..comments {
            let comment = setup.create_comment(&document, "excluded from option text");
            setup.insert_before(&comment, &option, None);
        }
        let last = setup.create_text(&document, " b ");
        setup.insert_before(&last, &option, None);
        setup.insert_before(&option, &select, None);
        let commit = dom.apply(setup, &cancel).unwrap();
        let select = commit.created(&select).unwrap().unwrap();
        let option = commit.created(&option).unwrap().unwrap();
        let before = dom.snapshot();
        let probe = Probe::start();
        let mut edit = DomBatch::new();
        edit.set_select_value(select, "a b");
        dom.apply(edit, &cancel).unwrap();
        let counts = probe.finish();
        assert!(
            selected(&dom, option),
            "actual fallback text ignores comments and collapses ASCII whitespace"
        );
        assert!(
            dom.snapshot()
                .node(option)
                .unwrap()
                .as_option()
                .unwrap()
                .state()
                .flags()
                .dirtiness()
        );
        assert!(
            !before
                .node(option)
                .unwrap()
                .as_option()
                .unwrap()
                .state()
                .flags()
                .dirtiness()
        );
        assert_eq!(counts.captures, 1);
        assert_eq!(
            counts.capture_nodes,
            comments + 3,
            "the genuine captured Option/Text/Comment graph is retained"
        );
        assert_eq!(
            counts.capture_edges,
            comments + 2,
            "the builder receives every original child in order"
        );
        assert!(
            counts.map_comparisons <= 2 * counts.capture_edges + 4,
            "{} captured edges performed {} exact-identity comparisons",
            counts.capture_edges,
            counts.map_comparisons
        );
    }
}
