use super::*;
use crate::parser::control::{Invariant, ParseStop};
fn control() -> ParseControl {
    ParseControl::new(
        crate::construction::tests::limits(),
        crate::HtmlParseCancellation::new(),
    )
}
fn attribute(name: &str) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, name), "v")
}
struct Fixture {
    owner: ConstructionOwner,
    shared: SharedAllocation<Vec<Attribute>>,
    vector: VectorCapacity<Attribute>,
}
impl Fixture {
    fn new(c: &mut ParseControl, names: &[&str], capacity: usize) -> Self {
        let (mut values, mut vector) = c.new_vector().unwrap();
        c.reserve_vector(&mut values, &mut vector, capacity)
            .unwrap();
        for name in names {
            values.push(attribute(name));
        }
        let (backing, shared) = c.share_owned(values).unwrap();
        let attributes = validate_parser_attributes(backing, c)
            .unwrap_or_else(|_| panic!("fixture unique names"));
        let mut owner = ConstructionOwner::new(c).unwrap();
        owner
            .apply(
                c,
                HtmlConstructionCommand::Create {
                    kind: HtmlNodeKind::Element(HtmlElement::new(
                        QualifiedName::new(Namespace::none(), None, "root"),
                        attributes,
                    )),
                },
                &[(HtmlParseResource::ManagedNodes, 1)],
            )
            .unwrap();
        Self {
            owner,
            shared,
            vector,
        }
    }
    fn backing(&self) -> &Arc<Vec<Attribute>> {
        let HtmlNodeKind::Element(v) = self.owner.node_at(0).unwrap().kind() else {
            unreachable!()
        };
        v.attributes().shared_attributes()
    }
    fn slot(
        &mut self,
        c: &mut ParseControl,
        kind: usize,
        incoming: &mut Option<Attribute>,
    ) -> Result<Option<Attribute>, SessionError> {
        self.owner
            .begin_attributes(c, 0, HtmlConstructionAttributeEditMode::Slots)?;
        let result = (|| {
            let group = self.owner.attribute_edit(c).unwrap();
            let slot = match kind {
                0 => group.reserve(100),
                1 => group.replace(0, incoming),
                _ => group.pop(),
            }
            .map_err(|e| construction_error(c, e))?;
            Ok(prepare_attribute_slot(c, slot, &mut self.shared, &mut self.vector)?.commit())
        })();
        let HtmlNodeKind::Element(v) = self.owner.node_at(0).unwrap().kind() else {
            unreachable!()
        };
        c.restore_attribute_weak(v.attributes().shared_attributes(), &mut self.shared);
        if result.is_ok() {
            self.owner.finish_attributes(c)?;
        } else {
            self.owner.abort_attributes(c)?;
        }
        result
    }
    fn append(
        &mut self,
        c: &mut ParseControl,
        incoming: &mut Vec<Attribute>,
        ticket: &VectorCapacity<Attribute>,
    ) -> Result<(), SessionError> {
        c.require_incoming_attribute_vector(incoming, ticket)?;
        self.owner
            .begin_attributes(c, 0, HtmlConstructionAttributeEditMode::Slots)?;
        let result = (|| {
            let slot = self
                .owner
                .attribute_edit(c)
                .unwrap()
                .append(incoming)
                .map_err(|e| construction_error(c, e))?;
            prepare_attribute_slot(c, slot, &mut self.shared, &mut self.vector)?.commit();
            Ok(())
        })();
        let HtmlNodeKind::Element(v) = self.owner.node_at(0).unwrap().kind() else {
            unreachable!()
        };
        c.restore_attribute_weak(v.attributes().shared_attributes(), &mut self.shared);
        if result.is_ok() {
            self.owner.finish_attributes(c)?;
        } else {
            self.owner.abort_attributes(c)?;
        }
        result
    }
    fn release(self, c: &mut ParseControl) {
        let backing = self.backing().clone();
        self.owner.retire(c).unwrap();
        let values = c.release_shared(backing, self.shared).unwrap();
        c.release_vector(values, self.vector).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }
}
#[test]
fn reserve_refusal_retains_contents_actual_capacity_and_original_weak_ticket() {
    let mut c = control();
    let mut fixture = Fixture::new(&mut c, &["a"], 1);
    let before = c.usage().actual_capacity_bytes;
    c.test_limit(HtmlParseResource::ManagedCapacityBytes, before);
    assert!(matches!(
        fixture.slot(&mut c, 0, &mut None),
        Err(SessionError::Stop(ParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }))
    ));
    assert_eq!(
        fixture.backing()[0].name().local_name().to_utf8().unwrap(),
        "a"
    );
    assert_eq!(Arc::weak_count(fixture.backing()), 1);
    assert_eq!(c.usage().actual_capacity_bytes, before);
    let first = c.stop();
    fixture.release(&mut c);
    assert_eq!(c.stop(), first);
}
#[test]
fn external_strong_or_weak_alias_refuses_before_reserve_and_preserves_tickets() {
    for weak in [false, true] {
        let mut c = control();
        let mut fixture = Fixture::new(&mut c, &[], 0);
        let strong = (!weak).then(|| fixture.backing().clone());
        let alias = weak.then(|| Arc::downgrade(fixture.backing()));
        assert!(matches!(
            fixture.slot(&mut c, 0, &mut None),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                Invariant::SharedAllocationAliased
            )))
        ));
        drop(strong);
        drop(alias);
        assert_eq!(Arc::weak_count(fixture.backing()), 1);
        fixture.release(&mut c);
    }
}
#[test]
fn replacement_qualifies_foreign_controller_and_cancellation_before_incoming_take() {
    for foreign in [false, true] {
        let cancellation = crate::HtmlParseCancellation::new();
        let mut c = ParseControl::new(crate::construction::tests::limits(), cancellation.clone());
        let mut fixture = Fixture::new(&mut c, &["old"], 1);
        let mut other = control();
        if !foreign {
            cancellation.cancel();
        }
        let active = if foreign { &mut other } else { &mut c };
        let mut incoming = Some(attribute("new"));
        assert!(fixture.slot(active, 1, &mut incoming).is_err());
        assert!(incoming.is_some());
        assert_eq!(
            fixture.backing()[0].name().local_name().to_utf8().unwrap(),
            "old"
        );
        fixture.release(&mut c);
    }
}
#[test]
fn checked_replacement_moves_real_slot_without_reserve_and_counts_new_comparison() {
    let mut c = control();
    let mut fixture = Fixture::new(&mut c, &["old", "other"], 2);
    let mut incoming = Some(attribute("new"));
    let before = c.usage().clone();
    let old = fixture.slot(&mut c, 1, &mut incoming).unwrap().unwrap();
    assert!(incoming.is_none());
    assert_eq!(old.name().local_name().to_utf8().unwrap(), "old");
    assert_eq!(
        fixture.backing()[0].name().local_name().to_utf8().unwrap(),
        "new"
    );
    assert_eq!(
        c.usage().actual_capacity_bytes,
        before.actual_capacity_bytes
    );
    // Different local lengths reject the only other pair at its length header;
    // no UTF16 unit is inspected, no allocator is called.
    assert_eq!(
        c.usage().work(WorkCategory::InspectByte) - before.work(WorkCategory::InspectByte),
        0
    );
    assert_eq!(
        c.usage().work(WorkCategory::ReserveByte) - before.work(WorkCategory::ReserveByte),
        0
    );
    assert_eq!(Arc::weak_count(fixture.backing()), 1);
    drop(old);
    fixture.release(&mut c);
}
#[test]
fn one_short_actual_replacement_layout_preserves_input_and_installed_backing() {
    for short in [false, true] {
        let mut c = control();
        let mut fixture = Fixture::new(&mut c, &["old"], 1);
        let mut incoming = Some(attribute("new"));
        c.test_limit(
            HtmlParseResource::AtomicLayoutBytes,
            (3 * mem::size_of::<Attribute>()) as u64 - u64::from(short),
        );
        let result = fixture.slot(&mut c, 1, &mut incoming);
        if short {
            assert!(matches!(
                result,
                Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AtomicLayoutBytes,
                    ..
                }))
            ));
            assert!(incoming.is_some());
            assert_eq!(
                fixture.backing()[0].name().local_name().to_utf8().unwrap(),
                "old"
            );
        } else {
            drop(result.unwrap());
            assert!(incoming.is_none());
        }
        assert_eq!(Arc::weak_count(fixture.backing()), 1);
        fixture.release(&mut c);
    }
}
#[test]
fn source_order_append_uses_same_bank_without_new_capacity_or_logical_admission() {
    let mut c = control();
    let mut fixture = Fixture::new(&mut c, &["old"], 3);
    let (mut incoming, mut ticket) = c.new_vector().unwrap();
    c.reserve_vector(&mut incoming, &mut ticket, 2).unwrap();
    incoming.push(attribute("second"));
    incoming.push(attribute("first"));
    c.acquire(HtmlParseResource::ManagedAttributes, 3).unwrap();
    let before = c.usage().actual_capacity_bytes;
    fixture.append(&mut c, &mut incoming, &ticket).unwrap();
    fixture.append(&mut c, &mut incoming, &ticket).unwrap();
    assert_eq!(
        fixture
            .backing()
            .iter()
            .map(|v| v.name().local_name().to_utf8().unwrap())
            .collect::<Vec<_>>(),
        ["old", "first", "second"]
    );
    assert!(incoming.is_empty());
    assert_eq!(c.usage().actual_capacity_bytes, before);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        3
    );
    c.release_vector(incoming, ticket).unwrap();
    fixture.release(&mut c);
}
#[test]
fn foreign_stage_and_one_short_work_refuse_before_actual_slot_move() {
    for foreign in [false, true] {
        let mut c = control();
        let mut other = control();
        let mut fixture = Fixture::new(&mut c, &["old"], 2);
        let producer = if foreign { &mut other } else { &mut c };
        let (mut incoming, mut ticket) = producer.new_vector().unwrap();
        producer
            .reserve_vector(&mut incoming, &mut ticket, 1)
            .unwrap();
        incoming.push(attribute("new"));
        if !foreign {
            c.test_limit(
                HtmlParseResource::WorkCredits,
                c.usage().resource(HtmlParseResource::WorkCredits).current + 1,
            );
        }
        assert!(fixture.append(&mut c, &mut incoming, &ticket).is_err());
        assert_eq!(incoming.len(), 1);
        assert_eq!(fixture.backing().len(), 1);
        assert_eq!(Arc::weak_count(fixture.backing()), 1);
        if foreign {
            other.release_vector(incoming, ticket).unwrap();
        } else {
            c.release_vector(incoming, ticket).unwrap();
        }
        fixture.release(&mut c);
    }
}

#[test]
fn checked_initial_append_inspects_exact_quadratic_current_prefix_pairs() {
    for count in [1usize, 2, 4, 8] {
        let mut c = control();
        let mut fixture = Fixture::new(&mut c, &[], count);
        let before = c.usage().work(WorkCategory::InspectByte);
        for index in 0..count {
            let (mut incoming, mut ticket) = c.new_vector().unwrap();
            c.reserve_vector(&mut incoming, &mut ticket, 1).unwrap();
            incoming.push(attribute(&format!("a{index}")));
            fixture.append(&mut c, &mut incoming, &ticket).unwrap();
            c.release_vector(incoming, ticket).unwrap();
        }
        // Each distinct two-unit name compares the shared 'a' and then its
        // differing digit: two UTF16 pairs (8 bytes), once per prior slot.
        let pairs = count * (count - 1) / 2;
        assert_eq!(
            c.usage().work(WorkCategory::InspectByte) - before,
            8 * pairs as u64
        );
        assert_eq!(fixture.owner.test_storage().usage().attributes(), count);
        // Four literal root-name units plus three units per actual aN=v slot.
        assert_eq!(
            fixture.owner.test_storage().usage().utf16_units(),
            4 + 3 * count
        );
        fixture.release(&mut c);
    }
}
