//! New private primitive characterizations; no callable pre-implementation API
//! existed for a node owning independent links in several ordered groups.
use super::*;

fn fixture() -> (Dom, NodeId, [Key; 3]) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(64, 4096, 64, 64, 0, 0, 0).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element(&document, "parent");
    let children = [
        batch.create_element(&document, "a"),
        batch.create_element(&document, "b"),
        batch.create_element(&document, "c"),
    ];
    for child in &children {
        batch.insert_before(child, &parent, None);
    }
    let commit = dom.apply(batch, &cancel).unwrap();
    let id = |ticket: &CreatedNode| commit.created(ticket).unwrap().unwrap();
    (
        dom,
        id(&parent),
        children.each_ref().map(|child| Key::Existing(id(child))),
    )
}

struct Pair {
    value: Members,
    custody: MembersCustody,
}
impl Pair {
    fn change(
        &mut self,
        root: RootHeader,
        key: Key,
        present: bool,
        order: OrderView<'_>,
        registry: &bank::CustodyRegistry,
    ) -> RootHeader {
        let changed = self
            .value
            .change(
                &self.custody,
                root,
                if present {
                    Membership::Insert(key)
                } else {
                    Membership::Remove(key)
                },
                order,
                Mode::Ordinary,
                Edit {
                    registry,
                    admit: &mut |_| Ok(()),
                    step: &mut || Ok(()),
                },
            )
            .unwrap();
        self.value = changed.members;
        self.custody = changed.custody;
        changed.root
    }
}

#[test]
fn one_node_has_independent_links_in_two_groups_and_old_roots_survive_removal() {
    let (dom, _, keys) = fixture();
    let order = OrderView::state(&dom.state, dom.owner);
    let registry = &dom.custody_registry;
    registry.reconcile_releases();
    let before = registry.total();
    let mut pair = Pair {
        value: Members::default(),
        custody: MembersCustody::default(),
    };
    let mut first = RootHeader::default();
    for key in [keys[2], keys[0], keys[1]] {
        first = pair.change(first, key, true, order, registry);
    }
    let mut second = RootHeader::default();
    for key in [keys[1], keys[0]] {
        second = pair.change(second, key, true, order, registry);
    }
    assert_eq!(pair.value.inspect_group(first).unwrap(), keys);
    assert_eq!(pair.value.inspect_group(second).unwrap(), keys[..2]);
    let historical = Pair {
        value: pair.value.clone(),
        custody: pair.custody.clone(),
    };
    let next = pair.change(first, keys[0], false, order, registry);
    assert_eq!(pair.value.inspect_group(next).unwrap(), keys[1..]);
    assert_eq!(pair.value.inspect_group(second).unwrap(), keys[..2]);
    assert_eq!(historical.value.inspect_group(first).unwrap(), keys);
    drop(pair);
    assert_eq!(historical.value.inspect_group(second).unwrap(), keys[..2]);
    drop(historical);
    registry.reconcile_releases();
    assert_eq!(registry.total(), before);
}

#[test]
fn refused_or_cancelled_change_keeps_both_groups_and_their_custody() {
    let (dom, _, keys) = fixture();
    let order = OrderView::state(&dom.state, dom.owner);
    let registry = &dom.custody_registry;
    let mut pair = Pair {
        value: Members::default(),
        custody: MembersCustody::default(),
    };
    let first = pair.change(RootHeader::default(), keys[0], true, order, registry);
    let second = pair.change(RootHeader::default(), keys[0], true, order, registry);
    registry.reconcile_releases();
    let before = registry.total();
    for cancel in [false, true] {
        let result = pair.value.change(
            &pair.custody,
            first,
            Membership::Insert(keys[1]),
            order,
            Mode::Ordinary,
            Edit {
                registry,
                admit: &mut |_| Err(DomError::AllocationFailed),
                step: &mut || {
                    if cancel {
                        Err(DomError::Cancelled)
                    } else {
                        Ok(())
                    }
                },
            },
        );
        assert!(matches!(
            result,
            Err(DomError::AllocationFailed) | Err(DomError::Cancelled)
        ));
        assert_eq!(pair.value.inspect_group(first).unwrap(), [keys[0]]);
        assert_eq!(pair.value.inspect_group(second).unwrap(), [keys[0]]);
        registry.reconcile_releases();
        assert_eq!(registry.total(), before);
    }
}

#[test]
fn first_link_record_gate_is_exact_and_one_byte_short_does_not_allocate_it() {
    let (dom, _, keys) = fixture();
    let order = OrderView::state(&dom.state, dom.owner);
    for short in [false, true] {
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
        let baseline = ledger.total();
        let first = custody::Usage::new(size_of::<Link>(), 0)
            .checked_add(custody::SharedDebit::administration_usage())
            .unwrap();
        let limit = baseline.bytes() + first.bytes() - usize::from(short);
        let calls = std::cell::Cell::new(0);
        let accepted_first = std::cell::Cell::new(false);
        let result = Members::default().change(
            &MembersCustody::default(),
            RootHeader::default(),
            Membership::Insert(keys[0]),
            order,
            Mode::Ordinary,
            Edit {
                registry: &registry,
                admit: &mut |usage| {
                    let call = calls.get();
                    calls.set(call + 1);
                    if call == 0 {
                        assert_eq!(usage, baseline.checked_add(first).unwrap());
                        if usage.bytes() <= limit {
                            accepted_first.set(true);
                        } else {
                            return Err(DomError::AllocationFailed);
                        }
                    } else {
                        return Err(DomError::AllocationFailed);
                    }
                    Ok(())
                },
                step: &mut || Ok(()),
            },
        );
        assert!(
            result.is_err(),
            "later registry/path allocation is deliberately refused"
        );
        assert_eq!(accepted_first.get(), !short);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
    }
}

#[test]
fn shadow_order_connects_only_reciprocal_roots_before_light_children() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(64, 4096, 64, 64, 0, 0, 0).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    let light = batch.create_element(&document, "light");
    batch.insert_before(&light, &host, None);
    let shadow = batch.attach_shadow(
        &host,
        crate::HtmlShadowRootProperties::new(
            crate::HtmlShadowRootMode::Open,
            crate::HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let nested = batch.create_element(&document, "nested");
    batch.insert_before(&nested, &shadow, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let key = |ticket: &CreatedNode| Key::Existing(commit.created(ticket).unwrap().unwrap());
    let (host, shadow, nested, light) = (key(&host), key(&shadow), key(&nested), key(&light));
    let empty = DomBatch::new();
    let mut draft = Draft::new(&dom, empty.owner, &cancel).unwrap();
    assert_eq!(
        draft
            .order_view()
            .root(nested, Mode::Ordinary, &mut || Ok(()))
            .unwrap(),
        shadow
    );
    assert_eq!(
        draft
            .order_view()
            .root(nested, Mode::ShadowIncluding, &mut || Ok(()))
            .unwrap(),
        host
    );
    assert_eq!(
        draft
            .order_view()
            .compare(nested, light, Mode::ShadowIncluding, &mut || Ok(()))
            .unwrap(),
        Ordering::Less
    );
    // Preparing host retirement disconnects the reciprocal edge before the
    // semantic holder is removed. The current view remains unchanged.
    assert_eq!(
        draft
            .order_view()
            .removed(host)
            .root(nested, Mode::ShadowIncluding, &mut || Ok(()))
            .unwrap(),
        shadow
    );
    assert_eq!(
        draft
            .order_view()
            .root(nested, Mode::ShadowIncluding, &mut || Ok(()))
            .unwrap(),
        host
    );
    draft
        .with_node_fields(shadow, &cancel, |fields| fields.host = None)
        .unwrap();
    assert_eq!(
        draft
            .order_view()
            .root(nested, Mode::ShadowIncluding, &mut || Ok(()))
            .unwrap(),
        shadow
    );
    assert_eq!(
        draft
            .order_view()
            .root(light, Mode::ShadowIncluding, &mut || Ok(()))
            .unwrap(),
        host
    );
    assert!(matches!(
        draft
            .order_view()
            .compare(nested, light, Mode::ShadowIncluding, &mut || Ok(())),
        Err(DomError::HierarchyRequest)
    ));
    draft
        .with_node_fields(shadow, &cancel, |fields| fields.host = Some(host))
        .unwrap();
    assert_eq!(
        draft
            .order_view()
            .compare(nested, light, Mode::ShadowIncluding, &mut || Ok(()))
            .unwrap(),
        Ordering::Less
    );
}
