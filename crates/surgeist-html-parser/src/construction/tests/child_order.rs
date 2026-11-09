use super::*;
use surgeist_html::values::Namespace;

fn start_test_insert(
    c: &mut ParseControl,
    owner: &mut ConstructionOwner,
    parent: &ParserNode,
    node: &ParserNode,
    before: Option<&ParserNode>,
) -> construction_owner::PendingConstruction {
    c.charge(
        HtmlParseWorkCategory::ProviderCall,
        2 + u64::from(before.is_some()),
    )
    .unwrap();
    owner
        .start(
            c,
            surgeist_html::bounded::construction::HtmlConstructionCommand::Insert {
                node: node.handle.clone(),
                parent: parent.handle.clone(),
                before: before.map(|value| value.handle.clone()),
            },
        )
        .unwrap()
}
/// Roomy limits for work comparisons across fanouts.
fn roomy_limits() -> HtmlParseLimits {
    limits()
        .into_builder()
        .auxiliary_entries(1_000_000)
        .active_formatting_entries(1_000_000)
        .managed_nodes(1_000_000)
        .managed_edges(1_000_000)
        .managed_capacity_bytes(1_000_000_000)
        .peak_transient_bytes(2_000_000_000)
        .atomic_layout_bytes(1_000_000_000)
        .work_credits(100_000_000_000)
        .finish()
        .unwrap()
}

/// Work a child-order change charged, by category; provider calls and
/// reservations are capacity growth, which depends on the order's spare room.
const ORDER_MEASURED: [HtmlParseWorkCategory; 6] = [
    HtmlParseWorkCategory::InputUnitRead,
    HtmlParseWorkCategory::InspectByte,
    HtmlParseWorkCategory::CopyByte,
    HtmlParseWorkCategory::StateStep,
    HtmlParseWorkCategory::RecordVisit,
    HtmlParseWorkCategory::RecordWrite,
];

#[test]
fn child_order_changes_charge_the_same_work_at_every_fanout() {
    // Under `fanout` children, insert before the last child, move the last
    // child before its predecessor, move it to the end, detach it, and move
    // it into another parent.
    let measure = |fanout: usize| {
        let mut c = ParseControl::new(roomy_limits(), HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let other = session
            .allocate_element(&mut c, element(Namespace::none(), "other"))
            .unwrap();
        session.insert(&mut c, &root, &other, None, None).unwrap();
        let mut children = Vec::new();
        for _ in 0..fanout {
            let child = session
                .allocate_element(&mut c, element(Namespace::none(), "x"))
                .unwrap();
            session.insert(&mut c, &root, &child, None, None).unwrap();
            children.push(child);
        }
        let fresh = session
            .allocate_element(&mut c, element(Namespace::none(), "fresh"))
            .unwrap();
        let last = children.last().unwrap();
        let previous = &children[fanout - 2];
        let auxiliary = c.usage().resource(HtmlParseResource::AuxiliaryEntries);
        let before = ORDER_MEASURED.map(|category| c.usage().work(category));
        session
            .insert(&mut c, &root, &fresh, Some(last), None)
            .unwrap();
        session
            .insert(&mut c, &root, last, Some(previous), None)
            .unwrap();
        session.insert(&mut c, &root, last, None, None).unwrap();
        session.insert(&mut c, &other, last, None, None).unwrap();
        session.remove(&mut c, last, None).unwrap();
        assert_eq!(
            c.usage().resource(HtmlParseResource::AuxiliaryEntries),
            auxiliary,
            "no change acquires scratch entries"
        );
        let after = ORDER_MEASURED.map(|category| c.usage().work(category));
        session.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        std::array::from_fn::<u64, 6, _>(|i| after[i] - before[i])
    };
    let narrow = measure(8);
    for fanout in [64, 512] {
        assert_eq!(
            measure(fanout),
            narrow,
            "changes among {fanout} children charged different work than among 8"
        );
    }
}

#[test]
fn inserting_text_before_a_sibling_charges_the_same_work_at_every_fanout() {
    // Text inserted before the last child finds its preceding sibling
    // without walking the children in front of it.
    let measure = |fanout: usize| {
        let source = crate::HtmlSource::utf8(format!("<output>{}", "x".repeat(8)));
        let (mut c, mut session, output, mut lexer) = output_owner(&source, roomy_limits());
        let mut last = None;
        for _ in 0..fanout {
            let span = session.allocate_element(&mut c, html("span")).unwrap();
            session.insert(&mut c, &output, &span, None, None).unwrap();
            last = Some(span);
        }
        let last = last.unwrap();
        let mut text = next_text(&mut lexer, &mut c);
        let before = ORDER_MEASURED.map(|category| c.usage().work(category));
        session
            .insert_text(&mut c, &output, Some(&last), &mut text, None)
            .unwrap();
        let after = ORDER_MEASURED.map(|category| c.usage().work(category));
        let owner = &session.active().unwrap().construction;
        let children = owner.node_at(output.index).unwrap().child_count();
        assert_eq!(children, fanout + 1);
        session.release(&mut c).unwrap();
        std::array::from_fn::<u64, 6, _>(|i| after[i] - before[i])
    };
    let narrow = measure(5);
    for fanout in [37, 300] {
        assert_eq!(
            measure(fanout),
            narrow,
            "text before the last of {fanout} children charged different work than of 5"
        );
    }
}

#[test]
fn changes_in_an_order_charge_for_the_entries_between_them_and_the_nearer_end() {
    // Among `n` and `2n` children an order walks from both ends in turn, one
    // action for each entry, and its commit moves the entries between the
    // place and the nearer end, as a deque does. An action costs 3 record
    // visits for the controller's `action` call and 1 for the described
    // `Inspect`, so 4 visits per entry walked, and a shift costs one record
    // write and one copied word per entry moved:
    // - cases 0 to 2 change the front: the walk examines the last entry and
    //   then the first, which is the target (insert before the first child
    //   walks to the reference sibling; remove and move walk to the node
    //   itself), and no entry moves, so going from n to 2n adds nothing;
    // - case 3 removes the child at n / 2, which has n / 2 entries before it
    //   and n / 2 - 1 behind it. The walk examines the entries from the back
    //   first, the target being its (n / 2)th, which is action n - 1, and the
    //   nearer, back end moves n / 2 - 1 entries, so going from n = 20 to
    //   n = 40 adds 20 actions, 80 visits, and 10 writes and words.
    let word = mem::size_of::<usize>() as u64;
    // (visits, writes, words) added going from 20 to 40 children, per case.
    let expected: [(u64, u64, u64); 4] = [(0, 0, 0), (0, 0, 0), (0, 0, 0), (80, 10, 10)];
    for (case, (visits, writes, words)) in expected.into_iter().enumerate() {
        let measure = |fanout: usize| {
            let mut c = ParseControl::new(roomy_limits(), HtmlParseCancellation::new());
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let root = session.root(&mut c).unwrap();
            let mut children = Vec::new();
            for _ in 0..fanout {
                let child = session
                    .allocate_element(&mut c, element(Namespace::none(), "x"))
                    .unwrap();
                session.insert(&mut c, &root, &child, None, None).unwrap();
                children.push(child);
            }
            let fresh = session
                .allocate_element(&mut c, element(Namespace::none(), "fresh"))
                .unwrap();
            let before = ORDER_MEASURED.map(|category| c.usage().work(category));
            match case {
                0 => session
                    .insert(&mut c, &root, &fresh, Some(&children[0]), None)
                    .unwrap(),
                1 => session.remove(&mut c, &children[0], None).unwrap(),
                2 => session
                    .insert(&mut c, &root, &children[0], None, None)
                    .unwrap(),
                _ => session.remove(&mut c, &children[fanout / 2], None).unwrap(),
            }
            let after = ORDER_MEASURED.map(|category| c.usage().work(category));
            session.release(&mut c).unwrap();
            std::array::from_fn::<u64, 6, _>(|i| after[i] - before[i])
        };
        // Counts leave spare room in both orders, so no growth is charged.
        let (narrow, wide) = (measure(20), measure(40));
        let grown = |category: HtmlParseWorkCategory| {
            let index = ORDER_MEASURED
                .iter()
                .position(|candidate| *candidate == category)
                .unwrap();
            wide[index] - narrow[index]
        };
        assert_eq!(
            grown(HtmlParseWorkCategory::RecordVisit),
            visits,
            "case {case}"
        );
        assert_eq!(
            grown(HtmlParseWorkCategory::RecordWrite),
            writes,
            "case {case}"
        );
        assert_eq!(
            grown(HtmlParseWorkCategory::CopyByte),
            words * word,
            "case {case}"
        );
    }
}

#[test]
fn child_order_changes_leave_the_order_the_dom_steps_define() {
    let mut c = ParseControl::new(roomy_limits(), HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let other = session
        .allocate_element(&mut c, element(Namespace::none(), "other"))
        .unwrap();
    session.insert(&mut c, &root, &other, None, None).unwrap();
    let mut nodes = Vec::new();
    for name in ["a", "b", "c", "d"] {
        let node = session
            .allocate_element(&mut c, element(Namespace::none(), name))
            .unwrap();
        session.insert(&mut c, &root, &node, None, None).unwrap();
        nodes.push(node);
    }
    let [a, b, cc, d] = nodes.as_slice() else {
        panic!("four fixtures")
    };
    let order = |session: &ParserArenaSession, parent: &ParserNode| {
        session
            .active()
            .unwrap()
            .construction
            .node_at(parent.index)
            .unwrap()
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        order(&session, &root),
        [other.index, a.index, b.index, cc.index, d.index]
    );
    // Forward move within a parent: before the node after its own place.
    session.insert(&mut c, &root, a, Some(d), None).unwrap();
    assert_eq!(
        order(&session, &root),
        [other.index, b.index, cc.index, a.index, d.index]
    );
    // Backward move, then a move to the end.
    session.insert(&mut c, &root, d, Some(b), None).unwrap();
    session.insert(&mut c, &root, &other, None, None).unwrap();
    assert_eq!(
        order(&session, &root),
        [d.index, b.index, cc.index, a.index, other.index]
    );
    // Across parents, then detach from the middle.
    session.insert(&mut c, &other, b, None, None).unwrap();
    session.insert(&mut c, &other, cc, Some(b), None).unwrap();
    session.remove(&mut c, a, None).unwrap();
    assert_eq!(order(&session, &root), [d.index, other.index]);
    assert_eq!(order(&session, &other), [cc.index, b.index]);
    assert!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(a.index)
            .unwrap()
            .parent()
            .is_none()
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedEdges).current,
        4
    );
    session.release(&mut c).unwrap();
    assert_eq!(c.usage().actual_capacity_bytes, 0);
}

/// Appends `pairs` elements, each followed by a new text node, under `parent`
/// and returns the work that charged, by category.
fn append_element_text_pairs(
    c: &mut ParseControl,
    session: &mut ParserArenaSession,
    parent: &ParserNode,
    lexer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    pairs: usize,
) -> [u64; 7] {
    const MEASURED: [HtmlParseWorkCategory; 7] = [
        HtmlParseWorkCategory::InputUnitRead,
        HtmlParseWorkCategory::InspectByte,
        HtmlParseWorkCategory::CopyByte,
        HtmlParseWorkCategory::StateStep,
        HtmlParseWorkCategory::RecordVisit,
        HtmlParseWorkCategory::RecordWrite,
        HtmlParseWorkCategory::ProviderCall,
    ];
    let before = MEASURED.map(|category| c.usage().work(category));
    for _ in 0..pairs {
        let span = session.allocate_element(c, html("span")).unwrap();
        session.insert(c, parent, &span, None, None).unwrap();
        let mut text = next_text(lexer, c);
        session
            .insert_text(c, parent, None, &mut text, None)
            .unwrap();
    }
    let after = MEASURED.map(|category| c.usage().work(category));
    std::array::from_fn(|i| after[i] - before[i])
}

/// Bytes the reservations of `count` appended children charged, counting
/// only the insertions.
fn append_reserve_bytes(count: usize) -> u64 {
    let caps = limits()
        .into_builder()
        .auxiliary_entries(1_000_000)
        .active_formatting_entries(1_000_000)
        .managed_nodes(1_000_000)
        .managed_edges(1_000_000)
        .managed_capacity_bytes(1_000_000_000)
        .peak_transient_bytes(2_000_000_000)
        .atomic_layout_bytes(1_000_000_000)
        .work_credits(100_000_000_000)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut reserved = 0;
    for _ in 0..count {
        let child = session
            .allocate_element(&mut c, element(Namespace::none(), "x"))
            .unwrap();
        let before = c.usage().work(HtmlParseWorkCategory::ReserveByte);
        session.insert(&mut c, &root, &child, None, None).unwrap();
        reserved += c.usage().work(HtmlParseWorkCategory::ReserveByte) - before;
    }
    session.release(&mut c).unwrap();
    reserved
}

#[test]
fn doubling_the_children_appended_to_one_parent_at_most_triples_their_reservation_work() {
    // Linear reservation work doubles with the count (a geometric series of
    // growth steps), exact per-append reservation quadruples it; three is
    // the midpoint, as in the whole-document linearity tests.
    let single = append_reserve_bytes(600);
    let double = append_reserve_bytes(1_200);
    assert!(
        double <= 3 * single,
        "appending 1200 children reserved {double} bytes, 600 reserved {single}"
    );
}

#[test]
fn appending_children_to_a_wide_parent_charges_no_more_work_than_to_a_narrow_one() {
    const BLOCK: usize = 32;
    let source = crate::HtmlSource::utf8(format!("<output>{}", "x".repeat(2_000)));
    let caps = limits()
        .into_builder()
        .source_units(1_000_000)
        .buffered_utf16_units(1_000_000)
        .auxiliary_entries(1_000_000)
        .active_formatting_entries(1_000_000)
        .managed_nodes(1_000_000)
        .managed_edges(1_000_000)
        .managed_utf16_units(1_000_000)
        .source_map_entries(1_000_000)
        .source_map_payload_units(1_000_000)
        .diagnostics(1_000_000)
        .diagnostic_payload_units(1_000_000)
        .managed_capacity_bytes(1_000_000_000)
        .peak_transient_bytes(2_000_000_000)
        .atomic_layout_bytes(1_000_000_000)
        .work_credits(100_000_000_000)
        .finish()
        .unwrap();
    let (mut c, mut session, output, mut lexer) = output_owner(&source, caps);
    let narrow = append_element_text_pairs(&mut c, &mut session, &output, &mut lexer, BLOCK);
    for _ in 0..15 {
        append_element_text_pairs(&mut c, &mut session, &output, &mut lexer, BLOCK);
    }
    let wide = append_element_text_pairs(&mut c, &mut session, &output, &mut lexer, BLOCK);
    let owner = &session.active().unwrap().construction;
    assert!(owner.node_at(output.index).unwrap().child_count() > 1_000);
    // Capacity growth is rarer at a higher fanout, so a wide parent can
    // cost less, never more: no category depends on the existing children.
    assert!(
        wide.iter()
            .zip(narrow)
            .all(|(wide, narrow)| *wide <= narrow),
        "work added to a wide parent {wide:?} exceeds the narrow parent's {narrow:?}"
    );
    session.release(&mut c).unwrap();
}

#[test]
fn work_limit_stop_before_an_insert_before_commit_leaves_the_order_and_the_stop_intact() {
    let selected = limits();
    let work_limit = selected.work_credits();
    let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let b = session
        .allocate_element(&mut c, element(Namespace::none(), "b"))
        .unwrap();
    session.insert(&mut c, &root, &b, None, None).unwrap();
    let a = session
        .allocate_element(&mut c, element(Namespace::none(), "a"))
        .unwrap();
    let owner = &mut session.active_mut().unwrap().construction;
    let mut pending = start_test_insert(&mut c, owner, &root, &a, Some(&b));
    let copied = loop {
        let edges = pending.before_edges();
        let action = pending.action(&mut c).unwrap().unwrap();
        let description = action.description();
        if matches!(
            description,
            surgeist_html::bounded::construction::HtmlConstructionActionDescription::Commit { .. }
        ) {
            // The commit charges the bytes it copies as the amount the stop is then asked to refuse.
            let copied_bytes =
                crate::schema_support::through_charged(|charge| description.charge_commit(charge))
                    .categories[2] as usize;
            // Leave one credit too few for the commit's described writes.
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                HtmlParseWorkCategory::StateStep,
                work_limit - before - (copied_bytes as u64 - 1),
            )
            .unwrap();
            let copied = c.usage().work(HtmlParseWorkCategory::CopyByte);
            assert!(
                matches!(owner.perform_action(&mut c, action, &[], edges), Err(SessionError::Stop(HtmlParseStop::LimitExceeded { resource: HtmlParseResource::WorkCredits, requested_delta, .. })) if requested_delta == copied_bytes as u64)
            );
            break copied;
        }
        assert!(!owner.perform_action(&mut c, action, &[], edges).unwrap());
    };
    assert_eq!(c.usage().work(HtmlParseWorkCategory::CopyByte), copied);
    // The order was grown for the push before the stop; nothing was written.
    let capacity_at_stop = c.usage().actual_capacity_bytes;
    let first = c.stop();
    let work = c.usage().resource(HtmlParseResource::WorkCredits);
    owner.abort_pending(&mut c, pending).unwrap();
    assert_eq!(c.stop(), first);
    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    assert_eq!(c.usage().actual_capacity_bytes, capacity_at_stop);
    assert_eq!(
        owner
            .node_at(root.index)
            .unwrap()
            .children()
            .map(surgeist_html::bounded::construction::HtmlConstructionNode::position)
            .collect::<Vec<_>>(),
        [b.index]
    );
    assert!(owner.node_at(a.index).unwrap().parent().is_none());
    session.release(&mut c).unwrap();
    assert_eq!(c.usage().actual_capacity_bytes, 0);
}

#[test]
fn append_succeeds_at_the_auxiliary_limit_because_it_uses_no_scratch_slot() {
    let selected = limits()
        .into_builder()
        .auxiliary_entries(4)
        .active_formatting_entries(4)
        .finish()
        .unwrap();
    let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let child = session
        .allocate_element(&mut control, element(Namespace::none(), "x"))
        .unwrap();
    session
        .insert(&mut control, &root, &child, None, None)
        .unwrap();
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [child.index]
    );
    let usage = control.usage();
    assert_eq!(usage.resource(HtmlParseResource::ManagedEdges).current, 1);
    assert_eq!(
        usage
            .resource(HtmlParseResource::AuxiliaryEntries)
            .high_water,
        4
    );
    session.release(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

/// High-water marks before and after the changes of `order_scenario`.
type OrderMarks = ([u64; 2], [u64; 2]);
/// The first stop of `order_scenario` and the order `root` held then.
type OrderStop = (SessionError, Vec<usize>);

/// Builds `root` holding `a`, `b`, `e` and `other` holding `d`, then
/// moves `c` before `d`, moves `b` before `a`, detaches `e`, and returns
/// the usage mark before the changes and after them, or the first stop.
fn order_scenario(selected: HtmlParseLimits) -> Result<OrderMarks, OrderStop> {
    let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let other = session
        .allocate_element(&mut control, element(Namespace::none(), "other"))
        .unwrap();
    session
        .insert(&mut control, &root, &other, None, None)
        .unwrap();
    let mut nodes = Vec::new();
    for name in ["a", "b", "c", "d", "e"] {
        nodes.push(
            session
                .allocate_element(&mut control, element(Namespace::none(), name))
                .unwrap(),
        );
    }
    let [a, b, c, d, e] = nodes.as_slice() else {
        panic!("expected five fixture handles")
    };
    for node in [a, b, e, c] {
        session
            .insert(&mut control, &root, node, None, None)
            .unwrap();
    }
    session.insert(&mut control, &other, d, None, None).unwrap();
    let mark = |control: &ParseControl| {
        [
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .high_water,
            control.usage().actual_capacity_high_water,
        ]
    };
    let before = mark(&control);
    let steps: [(&ParserNode, &ParserNode, Option<&ParserNode>); 2] =
        [(&other, c, Some(d)), (&root, b, Some(a))];
    for (parent, node, reference) in steps {
        if let Err(error) = session.insert(&mut control, parent, node, reference, None) {
            let orders = session
                .active()
                .unwrap()
                .construction
                .node_at(root.index)
                .unwrap()
                .children()
                .map(|node| node.position())
                .collect();
            return Err((error, orders));
        }
    }
    session.remove(&mut control, e, None).unwrap();
    let after = mark(&control);
    let order = |parent: &ParserNode| {
        session
            .active()
            .unwrap()
            .construction
            .node_at(parent.index)
            .unwrap()
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>()
    };
    assert_eq!(order(&root), [other.index, b.index, a.index]);
    assert_eq!(order(&other), [c.index, d.index]);
    assert!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(e.index)
            .unwrap()
            .parent()
            .is_none()
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedEdges)
            .current,
        5
    );
    session.release(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        0
    );
    Ok((before, after))
}

#[test]
fn child_order_changes_acquire_no_auxiliary_entries() {
    let (before, after) = order_scenario(roomy_limits()).unwrap_or_else(|_| panic!());
    assert_eq!(after[0], before[0]);
    // The exact observed high water is enough for the whole scenario.
    let exact = roomy_limits()
        .into_builder()
        .auxiliary_entries(after[0])
        .active_formatting_entries(after[0])
        .finish()
        .unwrap();
    order_scenario(exact).unwrap_or_else(|_| panic!());
}

#[test]
fn growing_a_parents_order_for_a_move_stops_at_the_capacity_limit_with_the_order_intact() {
    let (before, after) = order_scenario(roomy_limits()).unwrap_or_else(|_| panic!());
    assert!(after[1] > before[1], "the moves grow an order");
    let exact = roomy_limits()
        .into_builder()
        .managed_capacity_bytes(after[1])
        .finish()
        .unwrap();
    order_scenario(exact).unwrap_or_else(|_| panic!());
    let short = roomy_limits()
        .into_builder()
        .managed_capacity_bytes(after[1] - 1)
        .finish()
        .unwrap();
    let (error, root_order) = order_scenario(short).expect_err("one byte short");
    assert!(matches!(
        error,
        SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        })
    ));
    // The first move needs `other`'s order to grow, and the stop leaves
    // `root` holding [other, a, b, e, c] as it did.
    assert_eq!(root_order, [1, 2, 3, 6, 4]);
}
