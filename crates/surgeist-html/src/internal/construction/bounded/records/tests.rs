//! Tests of the record operations: their results, their refusals at every admission point, the
//! exactness of the depths they keep, and the work of an insertion.

use super::*;
use crate::internal::construction::{HtmlShadowRootMode, HtmlSlotAssignment};
use crate::internal::values::Namespace;

/// The refusal of a [`Meter`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Refused;

/// A consumer that records what it is asked and refuses at a chosen admission point: the
/// `refuse_at`th call of any of its methods, counting from zero. It chooses double the capacity of
/// a record or child bank, and the capacity a string needs.
#[derive(Default)]
struct Meter {
    refuse_at: Option<usize>,
    calls: Vec<&'static str>,
    work: [u64; 5],
    growths: Vec<HtmlConstructionBankGrowth>,
    records: Vec<HtmlConstructionGrowthRecord>,
    changes: Vec<HtmlConstructionChange>,
    largest: Vec<(HtmlResource, u64)>,
    /// Chooses the least capacity for every bank, so that a bank is full after each growth.
    exact: bool,
    /// Chooses a capacity of `isize::MAX` bytes for the banks of this kind, which the allocator
    /// refuses.
    huge: Option<Huge>,
}
/// The kinds of bank a [`Meter`] can choose a capacity for that the allocator refuses.
#[derive(Clone, Copy)]
enum Huge {
    Text,
    Children,
    Records,
}
const VISIT: usize = 0;
const WRITE: usize = 1;
const COPY: usize = 2;
const PROVIDER: usize = 3;
const INSPECT: usize = 4;

impl Meter {
    fn refusing_at(index: usize) -> Self {
        Self {
            refuse_at: Some(index),
            ..Self::default()
        }
    }
    fn call(&mut self, name: &'static str) -> Result<(), Refused> {
        let index = self.calls.len();
        self.calls.push(name);
        if self.refuse_at == Some(index) {
            Err(Refused)
        } else {
            Ok(())
        }
    }
}
impl Charge<HtmlWork, HtmlResource> for Meter {
    type Stop = Refused;
    fn charge(&mut self, batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
        self.call("charge")?;
        for change in batch {
            if let Change::Work(work, amount) = *change {
                let slot = match work {
                    HtmlWork::RecordVisit => VISIT,
                    HtmlWork::RecordWrite => WRITE,
                    HtmlWork::CopyByte => COPY,
                    HtmlWork::ProviderCall => PROVIDER,
                    HtmlWork::InspectByte => INSPECT,
                    _ => continue,
                };
                self.work[slot] += amount;
            }
        }
        Ok(())
    }
    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), Refused> {
        self.call("largest")?;
        self.largest.push((resource, value));
        Ok(())
    }
}
impl HtmlRecordAdmission for Meter {
    fn admit_change(
        &mut self,
        _: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), Refused> {
        self.call("admit_change")?;
        self.changes.push(change);
        Ok(())
    }
    fn admit_growth(&mut self, growth: HtmlConstructionBankGrowth) -> Result<usize, Refused> {
        self.call("admit_growth")?;
        self.growths.push(growth);
        let refused = matches!(
            (self.huge, growth.bank()),
            (Some(Huge::Text), HtmlConstructionBank::Text)
                | (Some(Huge::Children), HtmlConstructionBank::Children { .. })
                | (Some(Huge::Records), HtmlConstructionBank::Records)
        );
        if refused {
            return Ok(isize::MAX as usize / growth.element_size());
        }
        if self.exact {
            return Ok(growth.minimum_capacity());
        }
        Ok(match growth.bank() {
            HtmlConstructionBank::Records | HtmlConstructionBank::Children { .. } => growth
                .minimum_capacity()
                .max(growth.before().capacity() * 2),
            _ => growth.minimum_capacity(),
        })
    }
    fn record_growth(&mut self, record: HtmlConstructionGrowthRecord) -> Result<(), Refused> {
        self.call("record_growth")?;
        self.records.push(record);
        Ok(())
    }
    fn record_change(
        &mut self,
        _: &HtmlConstructionStorage,
        _: HtmlConstructionChange,
    ) -> Result<(), Refused> {
        self.call("record_change")
    }
}

type Outcome<T> = Result<T, HtmlConstructionRecordError<Refused>>;

fn ok<T>(outcome: Outcome<T>) -> T {
    match outcome {
        Ok(value) => value,
        Err(HtmlChargedError::Faulted(error)) => panic!("faulted: {error:?}"),
        Err(HtmlChargedError::Stopped(Refused)) => panic!("refused"),
    }
}
fn cause<T: std::fmt::Debug>(outcome: Outcome<T>) -> HtmlConstructionCause {
    match outcome {
        Err(HtmlChargedError::Faulted(error)) => error.cause(),
        other => panic!("expected a fault, got {other:?}"),
    }
}
fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn name(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}
fn storage() -> HtmlConstructionStorage {
    ok(HtmlConstructionStorage::new(&mut Meter::default()))
}
fn element(storage: &mut HtmlConstructionStorage, local: &str) -> HtmlBuilderNode {
    ok(storage.create_element(
        &name(local),
        OrderedAttributes::default(),
        None,
        &mut Meter::default(),
    ))
}
fn insert(storage: &mut HtmlConstructionStorage, node: &HtmlBuilderNode, parent: &HtmlBuilderNode) {
    ok(storage.insert(node, parent, None, &mut Meter::default()));
}
fn shadow_properties() -> HtmlShadowRootProperties {
    HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Closed,
        HtmlSlotAssignment::Manual,
        true,
        true,
        true,
    )
}
fn position(storage: &HtmlConstructionStorage, node: &HtmlBuilderNode) -> usize {
    storage.position(node).unwrap()
}
fn children(storage: &HtmlConstructionStorage, node: &HtmlBuilderNode) -> Vec<usize> {
    storage
        .node(node)
        .unwrap()
        .children()
        .map(|child| child.position())
        .collect()
}
/// The logical state of the storage: what a refused operation must leave as it was.
fn snapshot(storage: &HtmlConstructionStorage) -> Vec<String> {
    arena_snapshot(&storage.arena)
}
/// The logical state of an arena.
fn arena_snapshot(arena: &Arena) -> Vec<String> {
    let mut rows = vec![format!("{:?}", arena.usage)];
    for (index, record) in arena.records.iter().enumerate() {
        rows.push(format!(
            "{index} {:?} parent {:?} children {:?} depth {} population {} contents {:?} shadow {:?} properties {:?} owner {:?} designation {:?}",
            record.kind,
            record.parent.map(NodeIx::position),
            record.children.iter().map(|child| child.position()).collect::<Vec<_>>(),
            record.depth,
            record.population,
            record.template_contents,
            record.shadow,
            record.shadow_properties,
            record.owner,
            record.document_focused_area_designation,
        ));
    }
    rows
}

#[test]
fn a_new_storage_is_empty_and_two_storages_have_different_serials() {
    let (first, second) = (storage(), storage());
    assert!(first.is_empty());
    assert_eq!(first.usage(), HtmlConstructionUsage::default());
    assert_ne!(first.arena.serial, second.arena.serial);
}

#[test]
fn creations_make_the_records_they_name_and_count_what_they_hold() {
    let mut storage = storage();
    let mut meter = Meter::default();
    let document = ok(storage.create_document(HtmlDocumentMode::Quirks, &mut meter));
    let text = ok(storage.create_text(&utf16("ab\u{1f642}"), &mut meter));
    let comment = ok(storage.create_comment(&[0xd800, 0, 0xdc00], &mut meter));
    let instruction =
        ok(storage.create_processing_instruction(&utf16("t"), &utf16("dd"), &mut meter));
    let doctype =
        ok(storage.create_doctype(&utf16("html"), Some(&utf16("pub")), Some(&[]), &mut meter));
    let bare = ok(storage.create_doctype(&utf16("x"), None, None, &mut meter));
    let fragment = ok(storage.create_fragment(&mut meter));
    let shadow = ok(storage.create_shadow_contents(shadow_properties(), &mut meter));
    let div =
        ok(storage.create_element(&name("div"), OrderedAttributes::default(), None, &mut meter));
    assert_eq!(storage.len(), 9);
    for (index, node) in [
        &document,
        &text,
        &comment,
        &instruction,
        &doctype,
        &bare,
        &fragment,
        &shadow,
        &div,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(position(&storage, node), index);
    }
    use crate::internal::construction::ConstructionNodeKind as Kind;
    let kind = |node: &HtmlBuilderNode| storage.node(node).unwrap().kind();
    assert!(matches!(
        kind(&document),
        Kind::Document(HtmlDocumentMode::Quirks)
    ));
    assert!(matches!(kind(&text), Kind::Text(value) if value.code_units() == utf16("ab\u{1f642}")));
    assert!(
        matches!(kind(&comment), Kind::Comment(value) if value.code_units() == [0xd800, 0, 0xdc00])
    );
    assert!(matches!(
        kind(&instruction),
        Kind::ProcessingInstruction(value) if value.target().code_units() == utf16("t")
            && value.data().code_units() == utf16("dd")
    ));
    assert!(matches!(
        kind(&doctype),
        Kind::Doctype(value) if value.name().code_units() == utf16("html")
            && value.public_id().is_some_and(|id| id.code_units() == utf16("pub"))
            && value.system_id().is_some_and(|id| id.is_empty())
    ));
    assert!(matches!(
        kind(&bare),
        Kind::Doctype(value) if value.public_id().is_none() && value.system_id().is_none()
    ));
    assert!(matches!(kind(&fragment), Kind::Fragment));
    assert!(storage.node(&shadow).unwrap().is_shadow_contents());
    assert!(!storage.node(&fragment).unwrap().is_shadow_contents());
    let usage = storage.usage();
    assert_eq!(usage.nodes(), 9);
    assert_eq!(usage.edges(), 0);
    assert_eq!(usage.attributes(), 0);
    // text 4, comment 3, instruction 3, doctype 4 + 3 + 0, bare 1, and the name of the div: its
    // local name 3 and the HTML namespace URI 28.
    assert_eq!(usage.utf16_units(), 4 + 3 + 3 + 7 + 1 + 3 + 28);
    assert_eq!(usage.depth(), 0);
    assert_eq!(
        storage.arena.records[0].document_focused_area_designation,
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
}

#[test]
fn a_template_creates_its_owned_contents_and_one_edge() {
    let mut storage = storage();
    let template = element(&mut storage, "template");
    assert_eq!(storage.len(), 2);
    let record = storage.node(&template).unwrap();
    let contents = record.template_contents().unwrap();
    assert_eq!(contents.position(), 1);
    assert_eq!(
        record.template_contents_kind(),
        Some(HtmlTemplateContentsKind::Owned)
    );
    assert!(matches!(
        contents.fragment_owner(),
        Some(HtmlFragmentOwner::Template(owner)) if *owner == storage.result_id(&template).unwrap()
    ));
    assert_eq!(storage.usage().nodes(), 2);
    assert_eq!(storage.usage().edges(), 1);
    assert_eq!(storage.usage().depth(), 1);
    assert_eq!(storage.arena.records[1].depth, 1);
    assert_eq!(storage.arena.population(0), 1);
    assert_eq!(storage.arena.population(1), 1);
    // A template in another namespace is an ordinary element.
    let foreign = ok(storage.create_element(
        &QualifiedName::new(Namespace::svg(), None, "template"),
        OrderedAttributes::default(),
        None,
        &mut Meter::default(),
    ));
    assert!(
        storage
            .node(&foreign)
            .unwrap()
            .template_contents()
            .is_none()
    );
}

#[test]
fn a_creation_charges_its_rows_its_units_and_its_growth() {
    let mut storage = storage();
    let mut meter = Meter::default();
    ok(storage.create_text(&utf16("abc"), &mut meter));
    // One record bank and one string grow; each charges one provider call.
    assert_eq!(meter.work[PROVIDER], 2);
    assert_eq!(meter.growths.len(), 2);
    assert_eq!(meter.growths[0].bank(), HtmlConstructionBank::Records);
    assert_eq!(meter.growths[0].minimum_capacity(), 1);
    assert_eq!(meter.growths[0].element_size(), size_of::<Record>());
    assert_eq!(meter.growths[1].bank(), HtmlConstructionBank::Text);
    assert_eq!(meter.growths[1].minimum_capacity(), 3);
    assert_eq!(meter.growths[1].element_size(), 2);
    assert_eq!(meter.growths[1].wrapper_bytes(), size_of::<Vec<u16>>());
    // The rows and the usage, then 3 units = 6 bytes in one run.
    assert_eq!(
        meter.work[COPY] as usize,
        size_of::<Record>() + size_of::<usize>() + size_of::<HtmlConstructionUsage>() + 6
    );
    // Two writes per record, one for the usage, one for the run of units.
    assert_eq!(meter.work[WRITE], 2 + 1 + 1);
    assert_eq!(meter.work[VISIT], 0);
    let largest = meter.largest.clone();
    assert!(largest.contains(&(HtmlResource::AtomicLayoutBytes, size_of::<Record>() as u64)));
    assert!(largest.contains(&(HtmlResource::TreeDepth, 0)));
    // The first record of each growth is the capacity chosen; the second is absent when the
    // reservation reached what was chosen.
    assert_eq!(meter.records.len(), 2);
    assert_eq!(meter.records[1].chosen(), 3);
    assert_eq!(meter.records[1].reached().capacity(), 3);
    // Room that is there needs no growth: reserving what the bank holds admits nothing.
    let mut again = Meter::default();
    ok(storage.reserve_records(0, &mut again));
    assert!(again.growths.is_empty());
    ok(storage.reserve_records(9, &mut again));
    assert_eq!(again.growths.len(), 1);
    assert_eq!(again.growths[0].before().initialized_len(), 1);
    assert!(storage.arena.records.capacity() >= 10);
    let mut roomy = Meter::default();
    ok(storage.create_comment(&[], &mut roomy));
    assert_eq!(
        roomy.growths.len(),
        1,
        "only the empty string grows: the records have room"
    );
    assert_eq!(roomy.growths[0].bank(), HtmlConstructionBank::Text);
    assert_eq!(roomy.growths[0].minimum_capacity(), 0);
}

#[test]
fn an_insertion_orders_children_and_counts_edges_and_depth() {
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    let a = element(&mut storage, "a");
    let b = element(&mut storage, "b");
    let c = element(&mut storage, "c");
    for node in [&a, &b, &c] {
        insert(&mut storage, node, &root);
    }
    assert_eq!(children(&storage, &root), [1, 2, 3]);
    assert_eq!(storage.usage().edges(), 3);
    assert_eq!(storage.usage().depth(), 1);
    // Before a sibling, and a move within the parent: the place counts without the node itself.
    ok(storage.insert(&c, &root, Some(&a), &mut Meter::default()));
    assert_eq!(children(&storage, &root), [3, 1, 2]);
    ok(storage.insert(&c, &root, Some(&b), &mut Meter::default()));
    assert_eq!(children(&storage, &root), [1, 3, 2]);
    ok(storage.insert(&a, &root, None, &mut Meter::default()));
    assert_eq!(children(&storage, &root), [3, 2, 1]);
    // Inserting before itself changes nothing and admits nothing.
    let mut meter = Meter::default();
    let before = snapshot(&storage);
    ok(storage.insert(&b, &root, Some(&b), &mut meter));
    assert_eq!(snapshot(&storage), before);
    assert!(meter.growths.is_empty() && meter.changes.is_empty());
    // A move into another parent releases the old place and adds no edge.
    ok(storage.insert(&b, &a, None, &mut Meter::default()));
    assert_eq!(children(&storage, &root), [3, 1]);
    assert_eq!(children(&storage, &a), [2]);
    assert_eq!(storage.usage().edges(), 3);
    assert_eq!(storage.usage().depth(), 2);
    assert_eq!(
        storage
            .node(&b)
            .unwrap()
            .parent()
            .map(|node| node.position()),
        Some(1)
    );
    // Detaching removes the edge and the depth follows.
    ok(storage.detach(&b, &mut Meter::default()));
    assert_eq!(storage.usage().edges(), 2);
    assert_eq!(storage.usage().depth(), 1);
    assert!(storage.node(&b).unwrap().parent().is_none());
    let mut meter = Meter::default();
    ok(storage.detach(&b, &mut meter));
    assert!(
        meter.changes.is_empty(),
        "a detached node is left unchanged"
    );
}

#[test]
fn an_insertion_names_the_cause_of_each_structural_refusal() {
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    let document = ok(storage.create_document(HtmlDocumentMode::NoQuirks, &mut Meter::default()));
    let a = element(&mut storage, "a");
    let b = element(&mut storage, "b");
    let text = ok(storage.create_text(&utf16("x"), &mut Meter::default()));
    let template = element(&mut storage, "template");
    insert(&mut storage, &a, &root);
    insert(&mut storage, &b, &a);
    let m = Meter::default;
    use HtmlConstructionCause as Cause;
    assert_eq!(cause(storage.insert(&a, &b, None, &mut m())), Cause::Cycle);
    assert_eq!(cause(storage.insert(&a, &a, None, &mut m())), Cause::Cycle);
    assert_eq!(
        cause(storage.insert(&b, &text, None, &mut m())),
        Cause::ParentCannotHaveChildren
    );
    assert_eq!(
        cause(storage.insert(&b, &template, None, &mut m())),
        Cause::TemplateContentsRequired
    );
    assert_eq!(
        cause(storage.insert(&document, &root, None, &mut m())),
        Cause::RootMove
    );
    assert_eq!(
        cause(storage.insert(&root, &a, None, &mut m())),
        Cause::RootMove
    );
    let contents = storage
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    assert_eq!(
        cause(storage.insert(&contents, &a, None, &mut m())),
        Cause::AssociationMove
    );
    assert_eq!(
        cause(storage.insert(&text, &root, Some(&b), &mut m())),
        Cause::WrongSibling
    );
    assert_eq!(cause(storage.detach(&root, &mut m())), Cause::RootMove);
    let foreign = ok(HtmlConstructionStorage::new(&mut m())).arena.serial;
    let handle = HtmlBuilderNode {
        tree: foreign,
        index: NodeIx::FIRST,
    };
    assert_eq!(
        cause(storage.insert(&handle, &root, None, &mut m())),
        Cause::ForeignBuilder
    );
    let past = HtmlBuilderNode {
        tree: storage.arena.serial,
        index: NodeIx::at(1_000),
    };
    assert_eq!(cause(storage.detach(&past, &mut m())), Cause::UnknownNode);
    // A node with children that goes below its own descendant is a cycle through the walk; a
    // template cannot go into its own contents.
    assert_eq!(
        cause(storage.insert(&template, &contents, None, &mut m())),
        Cause::Cycle
    );
    ok(storage.check_child_transfer(&a, &root, &mut m()));
    assert_eq!(
        cause(storage.check_child_transfer(&a, &b, &mut m())),
        Cause::Cycle
    );
    assert_eq!(
        cause(storage.check_child_transfer(&a, &a, &mut m())),
        Cause::Cycle
    );
    assert_eq!(
        cause(storage.check_child_transfer(&a, &text, &mut m())),
        Cause::ParentCannotHaveChildren
    );
}

#[test]
fn shadow_and_template_associations_set_owners_edges_and_depths() {
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    let host = element(&mut storage, "div");
    insert(&mut storage, &host, &root);
    let contents = ok(storage.create_shadow_contents(shadow_properties(), &mut Meter::default()));
    let inner = element(&mut storage, "p");
    insert(&mut storage, &inner, &contents);
    let edges = storage.usage().edges();
    ok(storage.attach_shadow(&host, &contents, &mut Meter::default()));
    assert_eq!(storage.usage().edges(), edges + 1);
    let host_id = storage.result_id(&host).unwrap();
    assert_eq!(
        storage
            .node(&host)
            .unwrap()
            .shadow_root()
            .map(|shadow| shadow.contents()),
        Some(storage.result_id(&contents).unwrap())
    );
    assert!(matches!(
        storage.node(&contents).unwrap().fragment_owner(),
        Some(HtmlFragmentOwner::ShadowHost(owner)) if *owner == host_id
    ));
    // The shadow contents sit one below the host, and their child one more.
    assert_eq!(
        storage.arena.records[position(&storage, &contents)].depth,
        2
    );
    assert_eq!(storage.arena.records[position(&storage, &inner)].depth, 3);
    assert_eq!(storage.usage().depth(), 3);
    let m = Meter::default;
    use HtmlConstructionCause as Cause;
    let second = ok(storage.create_shadow_contents(shadow_properties(), &mut m()));
    assert_eq!(
        cause(storage.attach_shadow(&host, &second, &mut m())),
        Cause::ShadowAlreadyAttached
    );
    assert_eq!(
        cause(storage.attach_shadow(&root, &second, &mut m())),
        Cause::ShadowHostMustBeElement
    );
    let plain = ok(storage.create_fragment(&mut m()));
    let other = element(&mut storage, "i");
    assert_eq!(
        cause(storage.attach_shadow(&other, &plain, &mut m())),
        Cause::InvalidContentsReference
    );
    assert_eq!(
        cause(storage.attach_shadow(&other, &contents, &mut m())),
        Cause::AssociationAlreadyOwned
    );
    // A host inside the contents it would own is a cycle.
    let inside = element(&mut storage, "b");
    let spare = ok(storage.create_shadow_contents(shadow_properties(), &mut m()));
    insert(&mut storage, &inside, &spare);
    assert_eq!(
        cause(storage.attach_shadow(&inside, &spare, &mut m())),
        Cause::Cycle
    );

    // A template gives up its contents for shadow contents.
    let template = element(&mut storage, "template");
    insert(&mut storage, &template, &root);
    let shadow = ok(storage.create_shadow_contents(shadow_properties(), &mut m()));
    let owned = storage
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let edges = storage.usage().edges();
    assert_eq!(
        cause(storage.set_template_shadow_contents(&template, &owned, &mut m())),
        Cause::InvalidContentsReference
    );
    ok(storage.set_template_shadow_contents(&template, &shadow, &mut m()));
    assert_eq!(storage.usage().edges(), edges - 1);
    let record = storage.node(&template).unwrap();
    assert_eq!(
        record.template_contents_kind(),
        Some(HtmlTemplateContentsKind::ShadowReference)
    );
    assert_eq!(
        record.template_contents().unwrap().position(),
        position(&storage, &shadow)
    );
    assert!(storage.node(&owned).unwrap().fragment_owner().is_none());
    assert_eq!(
        cause(storage.set_template_shadow_contents(&template, &shadow, &mut m())),
        Cause::InvalidTemplateAssociation
    );

    // The same with a host.
    let template = element(&mut storage, "template");
    let host = element(&mut storage, "section");
    let shadow = ok(storage.create_shadow_contents(shadow_properties(), &mut m()));
    let owned = storage
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    ok(storage.attach_template_shadow(&template, &host, &shadow, &mut m()));
    assert_eq!(
        storage.node(&template).unwrap().template_contents_kind(),
        Some(HtmlTemplateContentsKind::ShadowReference)
    );
    assert!(storage.node(&owned).unwrap().fragment_owner().is_none());
    assert!(storage.node(&host).unwrap().shadow_root().is_some());
    assert_exact(&storage.arena, "template shadow");
}

#[test]
fn a_document_mode_and_link_flags_are_set_on_their_own_kinds_only() {
    let mut storage = storage();
    let m = Meter::default;
    let document = ok(storage.create_document(HtmlDocumentMode::NoQuirks, &mut m()));
    ok(storage.set_document_mode(&document, HtmlDocumentMode::Quirks, &mut m()));
    assert!(matches!(
        storage.node(&document).unwrap().kind(),
        crate::internal::construction::ConstructionNodeKind::Document(HtmlDocumentMode::Quirks)
    ));
    let div = element(&mut storage, "div");
    use HtmlConstructionCause as Cause;
    assert_eq!(
        cause(storage.set_document_mode(&div, HtmlDocumentMode::Quirks, &mut m())),
        Cause::WrongNodeKind
    );
    let flags = crate::internal::elements::rel::HtmlLinkFlags::default();
    assert_eq!(
        cause(storage.set_link_flags(&div, flags, &mut m())),
        Cause::WrongNodeKind
    );
    assert_eq!(
        cause(storage.set_link_flags(&document, flags, &mut m())),
        Cause::WrongNodeKind
    );
    let link = element(&mut storage, "link");
    ok(storage.set_link_flags(&link, flags, &mut m()));
    assert!(matches!(
        storage.node(&link).unwrap().kind(),
        crate::internal::construction::ConstructionNodeKind::Element(view) if view.link_flags().is_some()
    ));
    // Flags given at creation are kept on a `link` element and refused on any other.
    let created = ok(storage.create_element(
        &name("link"),
        OrderedAttributes::default(),
        Some(flags),
        &mut m(),
    ));
    assert!(matches!(
        storage.node(&created).unwrap().kind(),
        crate::internal::construction::ConstructionNodeKind::Element(view) if view.link_flags().is_some()
    ));
    let before = snapshot(&storage);
    assert_eq!(
        cause(storage.create_element(
            &name("div"),
            OrderedAttributes::default(),
            Some(flags),
            &mut m()
        )),
        Cause::WrongNodeKind
    );
    assert_eq!(snapshot(&storage), before);
}

#[test]
fn a_publication_validates_the_tree_and_hands_the_storage_back_when_refused() {
    let mut storage = storage();
    let m = Meter::default;
    let root = ok(storage.create_fragment(&mut m()));
    let a = element(&mut storage, "a");
    let text = ok(storage.create_text(&utf16("x"), &mut m()));
    insert(&mut storage, &a, &root);
    insert(&mut storage, &text, &a);
    // An element is not an output root, and neither is a node with a parent.
    for bad in [&a, &text] {
        match storage.publish(bad, &mut m()) {
            Err(refusal) => {
                let (back, error) = refusal.into_parts();
                assert!(matches!(
                    error,
                    HtmlChargedError::Faulted(error) if error.cause() == HtmlConstructionCause::InvalidOutputRoot
                ));
                storage = back;
            }
            Ok(_) => panic!("published a bad root"),
        }
    }
    let before = snapshot(&storage);
    let mut meter = Meter::default();
    let Ok(result) = storage.publish(&root, &mut meter) else {
        panic!("publication refused");
    };
    assert_eq!(result.len(), 3);
    assert_eq!(result.usage().nodes(), 3);
    assert_eq!(meter.growths.len(), 1);
    assert_eq!(meter.growths[0].bank(), HtmlConstructionBank::Wrapper);
    assert!(meter.changes.is_empty());
    // The result holds the records exactly as the storage did.
    assert_eq!(arena_snapshot(result.arena()), before);
}

/// Runs `operation` with a consumer that refuses at each admission point in turn, and checks that
/// every refusal leaves the logical state of the storage as it was. Returns the calls of the run
/// that was not refused.
fn sweep<T>(
    setup: impl Fn() -> (HtmlConstructionStorage, [HtmlBuilderNode; 4]),
    operation: impl Fn(&mut HtmlConstructionStorage, &[HtmlBuilderNode; 4], &mut Meter) -> Outcome<T>,
) -> Vec<&'static str> {
    for index in 0..10_000 {
        let (mut storage, nodes) = setup();
        let before = snapshot(&storage);
        let mut meter = Meter::refusing_at(index);
        match operation(&mut storage, &nodes, &mut meter) {
            Err(HtmlChargedError::Stopped(Refused)) => {
                assert_eq!(
                    snapshot(&storage),
                    before,
                    "a refusal at the {index}th call ({:?}) changed the storage",
                    meter.calls.last()
                );
                assert_exact(&storage.arena, "after a refusal");
            }
            Err(HtmlChargedError::Faulted(error)) => panic!("faulted at {index}: {error:?}"),
            Ok(_) => {
                assert!(index > 0);
                assert_exact(&storage.arena, "after the operation");
                return meter.calls;
            }
        }
    }
    panic!("the operation was refused at every call");
}

/// A fragment with an element `parent` holding an element `middle` and a text, a detached
/// element `fresh`; the nodes are `[root, parent, middle, fresh]`.
fn tree() -> (HtmlConstructionStorage, [HtmlBuilderNode; 4]) {
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    let parent = element(&mut storage, "div");
    let middle = element(&mut storage, "span");
    let text = ok(storage.create_text(&utf16("t"), &mut Meter::default()));
    let fresh = element(&mut storage, "em");
    insert(&mut storage, &parent, &root);
    insert(&mut storage, &middle, &parent);
    insert(&mut storage, &text, &parent);
    // Enough spare room, so that only the sweeps that choose it exercise a growth.
    (storage, [root, parent, middle, fresh])
}

/// The same tree with every bank full: the record bank holds exactly its records and the child
/// order of each record exactly its children, so a creation grows the records and an insertion
/// grows the child order.
fn full_tree() -> (HtmlConstructionStorage, [HtmlBuilderNode; 4]) {
    let exact = || Meter {
        exact: true,
        ..Meter::default()
    };
    let mut storage = ok(HtmlConstructionStorage::new(&mut exact()));
    let root = ok(storage.create_fragment(&mut exact()));
    let parent = ok(storage.create_element(
        &name("div"),
        OrderedAttributes::default(),
        None,
        &mut exact(),
    ));
    let middle = ok(storage.create_element(
        &name("span"),
        OrderedAttributes::default(),
        None,
        &mut exact(),
    ));
    let text = ok(storage.create_text(&utf16("t"), &mut exact()));
    let fresh = ok(storage.create_element(
        &name("em"),
        OrderedAttributes::default(),
        None,
        &mut exact(),
    ));
    for (node, under) in [(&parent, &root), (&middle, &parent), (&text, &parent)] {
        ok(storage.insert(node, under, None, &mut exact()));
    }
    assert_eq!(
        storage.arena.records.capacity(),
        storage.arena.records.len()
    );
    (storage, [root, parent, middle, fresh])
}

#[test]
fn creations_that_grow_the_records_are_refused_at_each_admission_point_leaving_the_storage_as_it_was()
 {
    let calls = sweep(full_tree, |storage, _, meter| {
        storage.create_text(&utf16("abc"), meter).map(|_| ())
    });
    // The records grow, then the string: two growths, each recorded before it is reserved.
    assert_eq!(
        calls.iter().filter(|call| **call == "admit_growth").count(),
        2
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| **call == "record_growth")
            .count(),
        2
    );
    sweep(full_tree, |storage, _, meter| {
        storage
            .create_processing_instruction(&utf16("a"), &utf16("bc"), meter)
            .map(|_| ())
    });
    sweep(full_tree, |storage, _, meter| {
        storage
            .create_doctype(&utf16("html"), Some(&utf16("p")), Some(&[]), meter)
            .map(|_| ())
    });
    sweep(full_tree, |storage, _, meter| {
        storage
            .create_element(&name("template"), OrderedAttributes::default(), None, meter)
            .map(|_| ())
    });
    sweep(full_tree, |storage, _, meter| {
        storage
            .create_document(HtmlDocumentMode::Quirks, meter)
            .map(|_| ())
    });
    sweep(full_tree, |storage, [_, parent, _, fresh], meter| {
        storage.insert(fresh, parent, None, meter)
    });
}

#[test]
fn every_operation_refused_at_each_admission_point_leaves_the_storage_as_it_was() {
    let calls = sweep(tree, |storage, [_, parent, _, fresh], meter| {
        storage.insert(fresh, parent, None, meter)
    });
    // The insertion grows the parent's child order from two entries: it is admitted, recorded and
    // changed once each.
    for point in [
        "admit_change",
        "admit_growth",
        "record_growth",
        "record_change",
    ] {
        assert!(calls.contains(&point), "no {point} in {calls:?}");
    }
    // The limits are admitted before the charge of the writes, which comes before any growth.
    let admitted = calls
        .iter()
        .position(|call| *call == "admit_change")
        .unwrap();
    let grown = calls
        .iter()
        .position(|call| *call == "admit_growth")
        .unwrap();
    assert!(
        calls[admitted..grown].contains(&"charge"),
        "no charge between the admission of the change and the growth in {calls:?}"
    );
    assert!(!calls[..admitted].contains(&"admit_growth"));
    sweep(tree, |storage, [_, parent, middle, fresh], meter| {
        storage.insert(fresh, parent, Some(middle), meter)
    });
    sweep(tree, |storage, [root, parent, middle, _], meter| {
        storage.insert(middle, root, Some(parent), meter)
    });
    sweep(tree, |storage, [_, _, middle, _], meter| {
        storage.detach(middle, meter)
    });
    sweep(tree, |storage, [root, parent, _, _], meter| {
        storage.check_child_transfer(parent, root, meter)
    });
    // The creations.
    sweep(tree, |storage, _, meter| {
        storage.create_text(&utf16("abc"), meter).map(|_| ())
    });
    sweep(tree, |storage, _, meter| {
        storage
            .create_processing_instruction(&utf16("a"), &utf16("bc"), meter)
            .map(|_| ())
    });
    sweep(tree, |storage, _, meter| {
        storage
            .create_doctype(&utf16("html"), Some(&utf16("p")), None, meter)
            .map(|_| ())
    });
    sweep(tree, |storage, _, meter| {
        storage
            .create_element(&name("template"), OrderedAttributes::default(), None, meter)
            .map(|_| ())
    });
    sweep(tree, |storage, _, meter| {
        storage
            .create_document(HtmlDocumentMode::Quirks, meter)
            .map(|_| ())
    });
    sweep(tree, |storage, _, meter| storage.reserve_records(40, meter));
    // The associations.
    let shadow_tree = || {
        let (mut storage, [root, parent, middle, fresh]) = tree();
        let shadow = ok(storage.create_shadow_contents(shadow_properties(), &mut Meter::default()));
        insert(&mut storage, &fresh, &shadow);
        (storage, [root, parent, middle, shadow])
    };
    sweep(shadow_tree, |storage, [_, parent, _, shadow], meter| {
        storage.attach_shadow(parent, shadow, meter)
    });
    let template_tree = || {
        let (mut storage, [root, parent, middle, _]) = tree();
        let template = element(&mut storage, "template");
        insert(&mut storage, &template, &root);
        let owned = storage
            .node(&template)
            .unwrap()
            .template_contents()
            .unwrap()
            .handle();
        insert(&mut storage, &middle, &owned);
        let shadow = ok(storage.create_shadow_contents(shadow_properties(), &mut Meter::default()));
        (storage, [template, parent, owned, shadow])
    };
    sweep(template_tree, |storage, [template, _, _, shadow], meter| {
        storage.set_template_shadow_contents(template, shadow, meter)
    });
    sweep(
        template_tree,
        |storage, [template, parent, _, shadow], meter| {
            storage.attach_template_shadow(template, parent, shadow, meter)
        },
    );
    // A host inside the contents the template gives up has its depth measured by a walk.
    let inside_tree = || {
        let (mut storage, [template, parent, owned, shadow]) = template_tree();
        insert(&mut storage, &parent, &owned);
        (storage, [template, parent, owned, shadow])
    };
    sweep(
        inside_tree,
        |storage, [template, parent, _, shadow], meter| {
            storage.attach_template_shadow(template, parent, shadow, meter)
        },
    );
    // A refused publication hands the storage back as it was.
    sweep(tree, |storage, [root, ..], meter| {
        let taken = std::mem::replace(
            storage,
            ok(HtmlConstructionStorage::new(&mut Meter::default())),
        );
        match taken.publish(root, meter) {
            Ok(_) => Ok(()),
            Err(refusal) => {
                let (back, error) = refusal.into_parts();
                *storage = back;
                Err(error)
            }
        }
    });
}

/// The records the consumer saw for the bank of `kind`, in order.
fn growth_records(
    meter: &Meter,
    kind: fn(HtmlConstructionBank) -> bool,
) -> Vec<HtmlConstructionGrowthRecord> {
    meter
        .records
        .iter()
        .copied()
        .filter(|record| kind(record.growth().bank()))
        .collect()
}

#[test]
fn an_allocator_that_refuses_the_spare_room_is_asked_again_for_the_room_needed() {
    // A text of three units: the string asks for `isize::MAX` bytes, which the allocator refuses,
    // and the retry for the three units that are needed succeeds.
    let mut storage = storage();
    let mut meter = Meter {
        huge: Some(Huge::Text),
        ..Meter::default()
    };
    let text = ok(storage.create_text(&utf16("abc"), &mut meter));
    let records = growth_records(&meter, |bank| bank == HtmlConstructionBank::Text);
    assert_eq!(records.len(), 2, "the choice and the capacity reached");
    assert_eq!(records[0].chosen(), isize::MAX as usize / 2);
    assert_eq!(records[0].recorded().capacity(), 0);
    assert_eq!(records[0].reached().capacity(), isize::MAX as usize / 2);
    assert_eq!(records[1].recorded().capacity(), isize::MAX as usize / 2);
    assert_eq!(records[1].reached().capacity(), 3);
    // Two growths and the retry: three requests to the allocator, three provider calls.
    assert_eq!(meter.work[PROVIDER], 3);
    assert!(matches!(
        storage.node(&text).unwrap().kind(),
        crate::internal::construction::ConstructionNodeKind::Text(value) if value.code_units() == utf16("abc")
    ));

    // A child order of two entries that is full: the push asks for room for three.
    let (mut storage, [_, parent, _, fresh]) = full_tree();
    let mut meter = Meter {
        huge: Some(Huge::Children),
        ..Meter::default()
    };
    ok(storage.insert(&fresh, &parent, None, &mut meter));
    let records = growth_records(&meter, |bank| {
        matches!(bank, HtmlConstructionBank::Children { .. })
    });
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].recorded().capacity(), 2);
    assert_eq!(
        records[0].reached().capacity(),
        isize::MAX as usize / size_of::<NodeIx>()
    );
    assert_eq!(records[1].reached().capacity(), 3);
    assert_eq!(meter.work[PROVIDER], 2, "one growth and its retry");
    assert_eq!(children(&storage, &parent).len(), 3);

    // The record bank, full: a reservation of one more record.
    let (mut storage, _) = full_tree();
    let before = storage.arena.records.len();
    let mut meter = Meter {
        huge: Some(Huge::Records),
        ..Meter::default()
    };
    ok(storage.reserve_records(1, &mut meter));
    let records = growth_records(&meter, |bank| bank == HtmlConstructionBank::Records);
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].recorded().capacity(), before);
    assert_eq!(records[1].reached().capacity(), before + 1);
    assert_eq!(storage.arena.records.capacity(), before + 1);
    assert_eq!(meter.work[PROVIDER], 2);
}

#[test]
fn an_allocator_that_refuses_the_room_needed_is_an_allocation_failure_that_changes_nothing() {
    // No operation can need more room than an allocator gives, so the refusal of the room needed
    // is injected into the reservation of `grow`, which every operation reserves through.
    for kind in [Huge::Text, Huge::Children, Huge::Records] {
        let (mut storage, [_, parent, _, _]) = full_tree();
        let before = snapshot(&storage);
        let growth = match kind {
            Huge::Text => HtmlConstructionBankGrowth {
                bank: HtmlConstructionBank::Text,
                before: HtmlConstructionBankCapacity::default(),
                minimum_capacity: 3,
                element_size: size_of::<u16>(),
                wrapper_bytes: size_of::<Vec<u16>>(),
            },
            Huge::Children => storage.children_growth(parent.index, 3).unwrap(),
            Huge::Records => storage
                .records_growth(storage.arena.records.len() + 1)
                .unwrap(),
        };
        let chosen = growth.minimum_capacity() + 4;
        let mut meter = Meter::default();
        let mut asked = Vec::new();
        let outcome = storage.grow(growth, chosen, &mut meter, |_, additional| {
            asked.push(additional);
            Reserved {
                ok: false,
                reached: growth.before(),
            }
        });
        assert!(matches!(
            outcome,
            Err(HtmlChargedError::Faulted(error)) if error.cause() == HtmlConstructionCause::AllocationFailed
        ));
        // The spare room, then the room that is needed.
        let len = growth.before().initialized_len();
        assert_eq!(asked, [chosen - len, growth.minimum_capacity() - len]);
        assert_eq!(meter.work[PROVIDER], 1, "the retry");
        assert_eq!(meter.records.len(), 2);
        assert_eq!(meter.records[0].reached().capacity(), chosen);
        assert_eq!(meter.records[1].recorded().capacity(), chosen);
        assert_eq!(
            meter.records[1].reached().capacity(),
            growth.before().capacity()
        );
        assert_eq!(snapshot(&storage), before);
    }
}

#[test]
fn a_growth_chosen_too_small_or_unrepresentable_is_a_typed_fault() {
    struct Small;
    impl Charge<HtmlWork, HtmlResource> for Small {
        type Stop = Refused;
        fn charge(&mut self, _: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
            Ok(())
        }
        fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), Refused> {
            Ok(())
        }
    }
    impl HtmlRecordAdmission for Small {
        fn admit_change(
            &mut self,
            _: &HtmlConstructionStorage,
            _: HtmlConstructionChange,
        ) -> Result<(), Refused> {
            Ok(())
        }
        fn admit_growth(&mut self, growth: HtmlConstructionBankGrowth) -> Result<usize, Refused> {
            Ok(growth.minimum_capacity().saturating_sub(1))
        }
        fn record_growth(&mut self, _: HtmlConstructionGrowthRecord) -> Result<(), Refused> {
            Ok(())
        }
        fn record_change(
            &mut self,
            _: &HtmlConstructionStorage,
            _: HtmlConstructionChange,
        ) -> Result<(), Refused> {
            Ok(())
        }
    }
    let mut storage = storage();
    let before = snapshot(&storage);
    assert!(matches!(
        storage.create_fragment(&mut Small),
        Err(HtmlChargedError::Faulted(error)) if error.cause() == HtmlConstructionCause::CapacityMismatch
    ));
    assert_eq!(snapshot(&storage), before);
    // A layout past what an allocation can hold is not representable.
    struct Huge;
    impl Charge<HtmlWork, HtmlResource> for Huge {
        type Stop = Refused;
        fn charge(&mut self, _: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
            Ok(())
        }
        fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), Refused> {
            Ok(())
        }
    }
    impl HtmlRecordAdmission for Huge {
        fn admit_change(
            &mut self,
            _: &HtmlConstructionStorage,
            _: HtmlConstructionChange,
        ) -> Result<(), Refused> {
            Ok(())
        }
        fn admit_growth(&mut self, _: HtmlConstructionBankGrowth) -> Result<usize, Refused> {
            Ok(usize::MAX)
        }
        fn record_growth(&mut self, _: HtmlConstructionGrowthRecord) -> Result<(), Refused> {
            Ok(())
        }
        fn record_change(
            &mut self,
            _: &HtmlConstructionStorage,
            _: HtmlConstructionChange,
        ) -> Result<(), Refused> {
            Ok(())
        }
    }
    assert!(matches!(
        storage.create_fragment(&mut Huge),
        Err(HtmlChargedError::Faulted(error)) if error.cause() == HtmlConstructionCause::RepresentationExhausted
    ));
    assert_eq!(snapshot(&storage), before);
}

// ---- exactness of depths ----

fn histogram_is_exact(arena: &Arena) -> bool {
    (0..arena.records.len()).all(|depth| {
        arena.population(depth)
            == arena
                .records
                .iter()
                .filter(|record| record.depth == depth)
                .count()
    })
}

/// Checks every cached depth, the usage depth and the histogram against depths recounted from
/// each record's own parent or owner.
fn assert_exact(arena: &Arena, context: &str) {
    use super::shape::owning_parent;
    let recount = |start: NodeIx| {
        let (mut index, mut depth) = (start, 0usize);
        while let Some(parent) = arena.records.get(index.position()).and_then(owning_parent) {
            index = parent;
            depth += 1;
            assert!(depth <= arena.records.len(), "{context}: owner cycle");
        }
        depth
    };
    let mut deepest = 0;
    for (index, record) in arena.records.iter().enumerate() {
        let expected = recount(NodeIx::at(index));
        assert_eq!(record.depth, expected, "{context}: depth of record {index}");
        deepest = deepest.max(expected);
    }
    assert_eq!(arena.usage.depth, deepest, "{context}: usage depth");
    assert!(histogram_is_exact(arena), "{context}: histogram");
}

/// A deterministic pseudo-random sequence; the seed is the only input.
struct Sequence(u64);
impl Sequence {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % bound
    }
}

#[test]
fn every_topology_operation_keeps_depths_usage_and_histogram_exact_under_random_use() {
    const KINDS: usize = 6;
    let mut successes = [0usize; KINDS];
    for seed in [1, 2, 3, 4, 5, 6] {
        let mut sequence = Sequence(seed);
        let mut storage = storage();
        let m = Meter::default;
        let (mut elements, mut templates, mut shadows, mut fragments) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for _ in 0..32 {
            elements.push(element(&mut storage, "e"));
        }
        for _ in 0..40 {
            templates.push(element(&mut storage, "template"));
        }
        // No operation detaches a shadow root, so each host and each shadow contents fragment is
        // spent by one attachment; the pools are sized so that every operation keeps succeeding
        // often enough to matter.
        for _ in 0..64 {
            shadows.push(ok(
                storage.create_shadow_contents(shadow_properties(), &mut m())
            ));
        }
        for _ in 0..24 {
            fragments.push(ok(storage.create_fragment(&mut m())));
        }
        let contents: Vec<HtmlBuilderNode> = templates
            .iter()
            .map(|template| {
                storage
                    .node(template)
                    .unwrap()
                    .template_contents()
                    .unwrap()
                    .handle()
            })
            .collect();
        let movable: Vec<HtmlBuilderNode> = elements.iter().chain(&templates).cloned().collect();
        let parents: Vec<HtmlBuilderNode> = elements
            .iter()
            .chain(&fragments)
            .chain(&shadows)
            .chain(&contents)
            .cloned()
            .collect();
        assert_exact(&storage.arena, "initial");
        for step in 0..3000 {
            let pick =
                |sequence: &mut Sequence, from: &[HtmlBuilderNode]| from[sequence.next(from.len())];
            let kind = sequence.next(KINDS);
            let outcome = match kind {
                0 | 1 => {
                    let (node, parent) =
                        (pick(&mut sequence, &movable), pick(&mut sequence, &parents));
                    storage.insert(&node, &parent, None, &mut m())
                }
                2 => {
                    let node = pick(&mut sequence, &movable);
                    storage.detach(&node, &mut m())
                }
                3 => {
                    let (host, shadow) =
                        (pick(&mut sequence, &movable), pick(&mut sequence, &shadows));
                    storage.attach_shadow(&host, &shadow, &mut m())
                }
                4 => {
                    let (template, host, shadow) = (
                        pick(&mut sequence, &templates),
                        pick(&mut sequence, &movable),
                        pick(&mut sequence, &shadows),
                    );
                    storage.attach_template_shadow(&template, &host, &shadow, &mut m())
                }
                _ => {
                    let (template, shadow) = (
                        pick(&mut sequence, &templates),
                        pick(&mut sequence, &shadows),
                    );
                    storage.set_template_shadow_contents(&template, &shadow, &mut m())
                }
            };
            successes[kind] += usize::from(outcome.is_ok());
            assert_exact(
                &storage.arena,
                &format!("seed {seed}, step {step}, kind {kind}"),
            );
        }
    }
    assert!(
        successes.iter().all(|count| *count > 50),
        "accepted operations per kind: {successes:?}"
    );
}

#[test]
fn rolling_back_nothing_a_refused_creation_leaves_the_histogram_exact() {
    let mut storage = storage();
    ok(storage.create_fragment(&mut Meter::default()));
    let before = snapshot(&storage);
    for index in 0..40 {
        let mut meter = Meter::refusing_at(index);
        if storage
            .create_element(
                &name("template"),
                OrderedAttributes::default(),
                None,
                &mut meter,
            )
            .is_ok()
        {
            break;
        }
        assert_eq!(snapshot(&storage), before);
        assert!(histogram_is_exact(&storage.arena));
    }
}

// ---- the work of an insertion ----

/// The work of inserting a fresh element at the end of a parent, by category.
fn append_work(storage: &mut HtmlConstructionStorage, parent: &HtmlBuilderNode) -> [u64; 5] {
    let fresh = element(storage, "fresh");
    // The child order has room, so the insertion grows nothing.
    let mut meter = Meter::default();
    ok(storage.insert(&fresh, parent, None, &mut meter));
    assert!(
        meter.growths.is_empty(),
        "the measured insertion grew a bank"
    );
    meter.work
}

#[test]
fn appending_a_fresh_node_visits_the_same_records_at_every_fanout_and_depth() {
    let wide = |fanout: usize| {
        let mut storage = storage();
        let root = ok(storage.create_fragment(&mut Meter::default()));
        ok(storage.reserve_records(fanout + 2, &mut Meter::default()));
        for _ in 0..fanout {
            let child = element(&mut storage, "x");
            insert(&mut storage, &child, &root);
        }
        // Room for one more entry in the order.
        let mut meter = Meter::default();
        let spare = element(&mut storage, "spare");
        ok(storage.insert(&spare, &root, None, &mut meter));
        ok(storage.detach(&spare, &mut meter));
        append_work(&mut storage, &root)
    };
    let deep = |depth: usize| {
        let mut storage = storage();
        let mut parent = ok(storage.create_fragment(&mut Meter::default()));
        ok(storage.reserve_records(depth + 8, &mut Meter::default()));
        for _ in 0..depth {
            let child = element(&mut storage, "x");
            insert(&mut storage, &child, &parent);
            parent = child;
        }
        for _ in 0..3 {
            let sibling = element(&mut storage, "y");
            insert(&mut storage, &sibling, &parent);
        }
        append_work(&mut storage, &parent)
    };
    let base_wide = wide(1_000);
    let base_deep = deep(1_000);
    for size in [2_000, 4_000, 8_000] {
        assert_eq!(wide(size), base_wide, "fanout {size}");
        assert_eq!(deep(size), base_deep, "depth {size}");
    }
    // The visits are the operands, the shallow check and the placement, and no more.
    assert_eq!(base_wide[VISIT], 2 + 2 + 3);
    assert_eq!(base_deep, base_wide);
}

/// The records written by inserting a fresh element before the child at `place` of `fanout`
/// children, or last when `place == fanout`, with room in the child order.
fn insert_writes(fanout: usize, place: usize) -> u64 {
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    ok(storage.reserve_records(fanout + 2, &mut Meter::default()));
    let mut children = Vec::new();
    for _ in 0..fanout {
        let child = element(&mut storage, "x");
        insert(&mut storage, &child, &root);
        children.push(child);
    }
    assert!(
        storage.node(&root).unwrap().children().len() < 64,
        "the child order has room for one more entry"
    );
    let fresh = element(&mut storage, "fresh");
    let mut meter = Meter::default();
    ok(storage.insert(&fresh, &root, children.get(place), &mut meter));
    assert!(meter.growths.is_empty(), "the insertion grew a bank");
    meter.work[WRITE]
}

#[test]
fn inserting_at_a_place_writes_the_entries_shifted_towards_the_nearer_end() {
    // A deque opens a place by moving the entries between it and the nearer end, so inserting
    // before the child at place `p` of `n` entries moves the lesser of `p` and `n - p` of them,
    // one write each, and every other write of the insertion is the same at every place.
    let fanout = 60;
    let base = insert_writes(fanout, 0);
    for place in [1, 7, 29, 30, 31, 53, 59, 60] {
        let moved = place.min(fanout - place) as u64;
        assert_eq!(
            insert_writes(fanout, place),
            base + moved,
            "inserting before the child at place {place} of {fanout}"
        );
    }
}

/// A fragment `root` with the ordinary children `a` (position 1) and `b` (position 2).
fn membership_tree() -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    for local in ["a", "b"] {
        let child = element(&mut storage, local);
        insert(&mut storage, &child, &root);
    }
    (storage, root)
}

/// The cause with which publishing `root` is refused, with the storage the refusal hands back.
fn publish_refusal(
    storage: HtmlConstructionStorage,
    root: &HtmlBuilderNode,
) -> (HtmlConstructionStorage, HtmlConstructionCause) {
    let Err(refusal) = storage.publish(root, &mut Meter::default()) else {
        panic!("the publication was not refused");
    };
    let (storage, error) = refusal.into_parts();
    match error {
        HtmlChargedError::Faulted(error) => (storage, error.cause()),
        HtmlChargedError::Stopped(Refused) => panic!("the meter refused nothing"),
    }
}

#[test]
fn a_publication_accepts_each_child_listed_exactly_once_whatever_stale_counts_say() {
    let (mut storage, root) = membership_tree();
    // A count left by an earlier publication is never trusted.
    for record in &mut storage.arena.records {
        record.publish_found = 2;
    }
    let result = ok(storage
        .publish(&root, &mut Meter::default())
        .map_err(|refusal| refusal.into_parts().1));
    assert_eq!(result.len(), 3);
}

#[test]
fn a_publication_refuses_a_child_listed_twice_and_a_later_one_never_trusts_its_counts() {
    let (mut storage, root) = membership_tree();
    let listed = storage.arena.records[0].children.clone();
    storage.arena.records[0].children =
        VecDeque::from([NodeIx::at(1), NodeIx::at(1), NodeIx::at(2)]);
    let (mut storage, cause) = publish_refusal(storage, &root);
    assert_eq!(cause, HtmlConstructionCause::ParentChildConflict);
    // The refusal left its counts written; mending the list lets the next publication succeed.
    storage.arena.records[0].children = listed;
    let published = storage.publish(&root, &mut Meter::default());
    assert!(published.is_ok());
}

#[test]
fn a_publication_refuses_a_child_missing_from_its_parents_list_despite_a_stale_count() {
    let (mut storage, root) = membership_tree();
    storage.arena.records[0].children = VecDeque::from([NodeIx::at(1)]);
    storage.arena.records[2].publish_found = 1;
    let (_, cause) = publish_refusal(storage, &root);
    assert_eq!(cause, HtmlConstructionCause::ParentChildConflict);
}

#[test]
fn a_publication_refuses_a_child_listed_only_by_another_parent() {
    // `b` names the root as its parent but only `a` lists it.
    let (mut storage, root) = membership_tree();
    storage.arena.records[0].children = VecDeque::from([NodeIx::at(1)]);
    storage.arena.records[1].children = VecDeque::from([NodeIx::at(2)]);
    let (_, cause) = publish_refusal(storage, &root);
    assert_eq!(cause, HtmlConstructionCause::ParentChildConflict);
}

/// The planned writes of an order edit that no longer find what planning found are refused before
/// the writes, with the records as they were.
#[test]
fn an_order_edit_that_no_longer_finds_its_planned_places_is_refused_before_any_write() {
    use super::order::Edit;
    let (mut storage, _root) = membership_tree();
    let (a, b) = (NodeIx::at(1), NodeIx::at(2));
    // `b` leaves the root's order for its end: the place it is planned to leave must hold it.
    let planned = Edit {
        old_parent: Some(NodeIx::FIRST),
        old_position: 1,
        insertion_position: 1,
    };
    // The root's order has room for the entry it gets back.
    storage.arena.records[0].children.reserve(4);
    assert_eq!(
        storage.verify_order(b, Some(NodeIx::FIRST), &planned),
        Ok(())
    );
    let before = snapshot(&storage);
    for (edit, cause) in [
        (
            Edit {
                old_position: 0,
                ..planned
            },
            HtmlConstructionCause::ParentChildConflict,
        ),
        (
            Edit {
                insertion_position: 2,
                ..planned
            },
            HtmlConstructionCause::ParentChildConflict,
        ),
    ] {
        assert_eq!(
            storage.verify_order(b, Some(NodeIx::FIRST), &edit),
            Err(cause.error())
        );
    }
    // A parent past every record is the unknown-node defect, and a full order is a capacity
    // mismatch.
    assert_eq!(
        storage.verify_order(
            a,
            Some(NodeIx::at(u32::MAX as usize - 1)),
            &Edit {
                old_parent: None,
                ..Edit::default()
            }
        ),
        Err(HtmlConstructionCause::UnknownNode.error())
    );
    let children = &mut storage.arena.records[0].children;
    while children.len() < children.capacity() {
        children.push_back(NodeIx::FIRST);
    }
    let full = snapshot(&storage);
    assert_eq!(
        storage.verify_order(
            a,
            Some(NodeIx::FIRST),
            &Edit {
                old_parent: None,
                insertion_position: 0,
                ..Edit::default()
            }
        ),
        Err(HtmlConstructionCause::CapacityMismatch.error())
    );
    assert_eq!(snapshot(&storage), full);
    assert_ne!(full, before, "the order was filled to make it full");
}

#[test]
fn rolling_back_a_native_append_removes_its_records_from_the_histogram() {
    let mut storage = storage();
    ok(storage.create_fragment(&mut Meter::default()));
    let (len, usage) = (storage.len(), storage.usage());
    assert!(histogram_is_exact(&storage.arena));
    // A template creates two records, at depths 0 and 1.
    element(&mut storage, "template");
    assert_eq!(storage.len(), len + 2);
    assert!(histogram_is_exact(&storage.arena));
    storage.rollback_native_append(len, usage).unwrap();
    assert_eq!(storage.len(), len);
    assert_eq!(storage.usage(), usage);
    assert!(histogram_is_exact(&storage.arena));
    assert_eq!(storage.arena.population(0), 1);
}

#[test]
fn a_shared_string_is_held_as_it_is_without_a_bank_or_a_copy() {
    let text = DocumentString::from_utf16(utf16("shared text"));
    let mut storage = storage();
    let mut shared = Meter::default();
    let node = ok(storage.create_shared_text(text.clone(), &mut shared));
    let mut copied = Meter::default();
    ok(storage.create_text(text.code_units(), &mut copied));
    // The record holds the caller's string itself.
    let Some(crate::internal::construction::ConstructionNodeKind::Text(held)) =
        storage.node(&node).ok().map(|node| node.kind())
    else {
        panic!("a text record");
    };
    assert_eq!(held.code_units(), text.code_units());
    let NodeContent::Text(stored) = &storage.arena.records[0].kind else {
        panic!("a text record");
    };
    assert!(stored.shares_storage_with(&text));
    assert_eq!(storage.usage().utf16_units(), 2 * text.len());
    // Only the copy asks for a bank for its units and copies them, in runs of at most 256 bytes.
    let banks = |meter: &Meter| {
        meter
            .growths
            .iter()
            .filter(|growth| growth.bank() == HtmlConstructionBank::Text)
            .count()
    };
    assert_eq!((banks(&shared), banks(&copied)), (0, 1));
    let units_bytes = (text.len() * size_of::<u16>()) as u64;
    assert_eq!(copied.work[COPY] - shared.work[COPY], units_bytes);
    assert_eq!(copied.work[PROVIDER] - shared.work[PROVIDER], 1);
}
