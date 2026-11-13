//! Tests of the record operations: their results, their refusals at every admission point, the
//! exactness of the depths they keep, and the work of an insertion.

use super::*;
use crate::internal::construction::{HtmlShadowRootMode, HtmlSlotAssignment};
use crate::internal::values::{Attribute, Namespace};

mod names_tests;
mod publication_refusals;

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
    /// Chooses a capacity of `isize::MAX` bytes for this bank, which the allocator refuses.
    huge_bank: Option<HtmlConstructionBank>,
    /// The record of a growth this consumer refused.
    refused_record: Option<HtmlConstructionGrowthRecord>,
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
            (
                Some(Huge::Text),
                HtmlConstructionBank::Text | HtmlConstructionBank::TextArena
            ) | (Some(Huge::Children), HtmlConstructionBank::Children { .. })
                | (Some(Huge::Records), HtmlConstructionBank::Records)
        );
        if refused || self.huge_bank == Some(growth.bank()) {
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
        if let Err(refused) = self.call("record_growth") {
            self.refused_record = Some(record);
            return Err(refused);
        }
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
    let mut rows = arena_snapshot(&storage.arena);
    // The text arena: the units it holds and the units its records hold.
    rows.push(format!(
        "text {:?} used {}",
        storage.text,
        storage.text.used()
    ));
    // The attribute table: its rows and runs.
    rows.push(format!("attributes {:?}", storage.attrs));
    rows
}
/// The content of a record without the way it holds its text.
fn kind_shape(kind: &NodeContent) -> String {
    match kind {
        NodeContent::Text(text) => format!("Text of {} units", text.len()),
        NodeContent::Comment(text) => format!("Comment of {} units", text.len()),
        other => format!("{other:?}"),
    }
}
/// The logical state of an arena.
fn arena_snapshot(arena: &BuildArena) -> Vec<String> {
    let mut rows = vec![format!("{:?}", arena.usage)];
    for (index, record) in arena.records.iter().enumerate() {
        rows.push(format!(
            "{index} {} parent {:?} children {:?} depth {} population {} contents {:?} shadow {:?} properties {:?} owner {:?} designation {:?}",
            kind_shape(&record.kind),
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

/// The content of a record as a reader sees it, without the form of the storage it came from.
fn view_kind(kind: crate::internal::construction::ConstructionNodeKind<'_>) -> String {
    use crate::internal::construction::ConstructionNodeKind as Kind;
    match kind {
        Kind::Element(element) => {
            let attributes: Vec<_> = element
                .attributes()
                .iter()
                .expect("the attributes of an element")
                .map(|attribute| (attribute.name().clone(), attribute.value()))
                .collect();
            format!(
                "Element {:?} {:?} {attributes:?}",
                element.name(),
                element.link_flags()
            )
        }
        other => format!("{other:?}"),
    }
}

/// What a reader sees of the records of a storage that builds or of a published result: the
/// content, the place in the tree and the associations of every record, through the one view both
/// forms share.
fn view_snapshot<'a>(
    len: usize,
    node_at: impl Fn(usize) -> Option<HtmlConstructionNode<'a>>,
) -> Vec<String> {
    (0..len)
        .map(|index| {
            let node = node_at(index).expect("a record of the view");
            let positions = |nodes: Vec<HtmlConstructionNode<'_>>| {
                nodes.into_iter().map(|node| node.position()).collect::<Vec<_>>()
            };
            let shadow = node
                .shadow_root()
                .map(|root| (root.host(), root.contents(), root.properties()));
            format!(
                "{index} {} parent {:?} children {:?} contents {:?} {:?} shadow {shadow:?} properties {:?} owner {:?}",
                view_kind(node.kind()),
                node.parent().map(HtmlConstructionNode::position),
                positions(node.children().collect()),
                node.template_contents().map(HtmlConstructionNode::position),
                node.template_contents_kind(),
                node.shadow_properties(),
                node.fragment_owner(),
            )
        })
        .collect()
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
    assert!(
        matches!(kind(&text), Kind::Text(value) if value.code_units().unwrap() == utf16("ab\u{1f642}"))
    );
    assert!(
        matches!(kind(&comment), Kind::Comment(value) if value.code_units().unwrap() == [0xd800, 0, 0xdc00])
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
        Some(HtmlFragmentOwner::Template(owner)) if owner == storage.result_id(&template).unwrap()
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
    // One record bank and the text arena grow; each charges one provider call.
    assert_eq!(meter.work[PROVIDER], 2);
    assert_eq!(meter.growths.len(), 2);
    assert_eq!(meter.growths[0].bank(), HtmlConstructionBank::Records);
    assert_eq!(meter.growths[0].minimum_capacity(), 1);
    assert_eq!(meter.growths[0].element_size(), size_of::<Record>());
    assert_eq!(meter.growths[1].bank(), HtmlConstructionBank::TextArena);
    assert_eq!(meter.growths[1].minimum_capacity(), 3);
    assert_eq!(meter.growths[1].element_size(), 2);
    assert_eq!(meter.growths[1].wrapper_bytes(), 0);
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
    assert!(
        roomy.growths.is_empty(),
        "an empty string needs no unit of the arena, and the records have room"
    );
    // A comment whose units the arena has no room for grows the arena once, from its length.
    let mut grown = Meter::default();
    ok(storage.create_comment(&utf16("de"), &mut grown));
    assert_eq!(grown.growths.len(), 1);
    assert_eq!(grown.growths[0].bank(), HtmlConstructionBank::TextArena);
    assert_eq!(grown.growths[0].before().initialized_len(), 3);
    assert_eq!(grown.growths[0].before().capacity(), 3);
    assert_eq!(grown.growths[0].minimum_capacity(), 5);
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
        Some(HtmlFragmentOwner::ShadowHost(owner)) if owner == host_id
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
    let before = view_snapshot(storage.len(), |at| storage.node_at(at));
    let mut meter = Meter::default();
    let Ok(result) = storage.publish(&root, &mut meter) else {
        panic!("publication refused");
    };
    assert_eq!(result.len(), 3);
    assert_eq!(result.usage().nodes(), 3);
    // The result is a published vector of records and the contents made for them, and the shared
    // arena. The one text of three units has no spare units, so no block is packed.
    assert_eq!(
        meter
            .growths
            .iter()
            .map(|growth| growth.bank())
            .collect::<Vec<_>>(),
        [
            HtmlConstructionBank::Wrapper,
            HtmlConstructionBank::PublishedRows,
            HtmlConstructionBank::PublishedChildren,
        ]
    );
    assert!(meter.changes.is_empty());
    // The result holds the records exactly as the storage did.
    assert_eq!(view_snapshot(result.len(), |at| result.node_at(at)), before);
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
    kind: impl Fn(HtmlConstructionBank) -> bool,
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
    let records = growth_records(&meter, |bank| bank == HtmlConstructionBank::TextArena);
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
        crate::internal::construction::ConstructionNodeKind::Text(value) if value.code_units().unwrap() == utf16("abc")
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

fn histogram_is_exact(arena: &BuildArena) -> bool {
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
fn assert_exact(arena: &BuildArena, context: &str) {
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

/// The tree of [`tree`] whose text arena has room for many more units than its text holds, so
/// that a publication packs it.
fn spare_tree() -> (HtmlConstructionStorage, [HtmlBuilderNode; 4]) {
    let (mut storage, nodes) = tree();
    storage.text.reserve_exact(100).unwrap();
    (storage, nodes)
}

fn publish_tree(
    storage: HtmlConstructionStorage,
    root: &HtmlBuilderNode,
    meter: &mut Meter,
) -> Result<
    crate::internal::construction::bounded::HtmlConstructionResult,
    Box<(
        HtmlConstructionStorage,
        HtmlConstructionRecordError<Refused>,
    )>,
> {
    storage
        .publish(root, meter)
        .map_err(|refusal| Box::new(refusal.into_parts()))
}

/// The text node of the published `tree`, whose text record is its fourth.
fn published_text(
    result: &crate::internal::construction::bounded::HtmlConstructionResult,
) -> DocumentString {
    let node = result.node_at(3).unwrap().as_node().unwrap();
    let crate::internal::construction::HtmlNodeKind::Text(text) = node.kind() else {
        panic!("a text record");
    };
    text
}

#[test]
fn a_publication_packs_an_arena_with_more_than_a_quarter_of_its_units_spare() {
    let (storage, [root, ..]) = spare_tree();
    let spare = storage.text.capacity();
    assert!(spare > 100);
    let mut meter = Meter::default();
    let Ok(result) = publish_tree(storage, &root, &mut meter) else {
        panic!("publication refused");
    };
    let banks: Vec<_> = meter.growths.iter().map(|growth| growth.bank()).collect();
    assert_eq!(
        banks,
        [
            HtmlConstructionBank::Wrapper,
            HtmlConstructionBank::TextBlock,
            HtmlConstructionBank::PublishedRows,
            HtmlConstructionBank::PublishedChildren,
        ]
    );
    // The block is exactly the units the texts hold, and its growth names the arena it replaces.
    let block = meter.growths[1];
    assert_eq!(block.before().capacity(), spare);
    assert_eq!(block.minimum_capacity(), 1);
    assert_eq!(block.element_size(), 2);
    let text = published_text(&result);
    assert_eq!(text.code_units(), utf16("t"));
    assert_eq!(text.block().capacity(), 1);
    // The rows are a bank of their own, exact and of the row size, and every child but the
    // detached one is listed once.
    let rows = meter.growths[2];
    assert_eq!(rows.before(), HtmlConstructionBankCapacity::default());
    assert_eq!(rows.minimum_capacity(), result.len());
    assert_eq!(rows.element_size(), 20);
    let children = meter.growths[3];
    assert_eq!(children.minimum_capacity(), 3);
    assert_eq!(children.element_size(), 4);
}

#[test]
fn a_publication_of_an_arena_without_spare_units_makes_the_arena_its_block() {
    let (storage, [root, ..]) = tree();
    let capacity = storage.text.capacity();
    let mut meter = Meter::default();
    let Ok(result) = publish_tree(storage, &root, &mut meter) else {
        panic!("publication refused");
    };
    assert!(
        meter
            .growths
            .iter()
            .all(|growth| growth.bank() != HtmlConstructionBank::TextBlock)
    );
    let text = published_text(&result);
    assert_eq!(text.code_units(), utf16("t"));
    assert_eq!(text.block().capacity(), capacity);
    // The wrapper holds the vector header that shares the block.
    assert_eq!(meter.growths[0].bank(), HtmlConstructionBank::Wrapper);
    assert_eq!(meter.growths[0].wrapper_bytes(), size_of::<Vec<u16>>());
}

/// The banks the records of a consumer leave it holding something for: each with the dimensions
/// the bank had before its growth and the dimensions the consumer holds after the last record.
fn consumer_holdings(
    meter: &Meter,
) -> Vec<(
    HtmlConstructionBank,
    HtmlConstructionBankCapacity,
    HtmlConstructionBankCapacity,
)> {
    let mut banks: Vec<(
        HtmlConstructionBank,
        HtmlConstructionBankCapacity,
        HtmlConstructionBankCapacity,
    )> = Vec::new();
    for record in &meter.records {
        let growth = record.growth();
        match banks.iter_mut().find(|bank| bank.0 == growth.bank()) {
            Some(bank) => bank.2 = record.reached(),
            None => banks.push((growth.bank(), growth.before(), record.reached())),
        }
    }
    banks
}

/// Whether the consumer holds for every bank what it held before the publication.
fn holds_nothing_of_the_publication(meter: &Meter) -> bool {
    consumer_holdings(meter)
        .iter()
        .all(|(_, before, held)| before == held)
}

#[test]
fn a_publication_refused_at_any_step_leaves_the_consumer_holding_nothing_of_it() {
    type Make = fn() -> (HtmlConstructionStorage, [HtmlBuilderNode; 4]);
    // The banks whose allocator refuses the room chosen too make a second record of the growth,
    // which the sweep then refuses as well.
    for (make, huge_bank) in [
        (spare_tree as Make, None),
        (tree as Make, None),
        (spare_tree as Make, Some(HtmlConstructionBank::TextBlock)),
        (
            spare_tree as Make,
            Some(HtmlConstructionBank::PublishedRows),
        ),
        (tree as Make, Some(HtmlConstructionBank::PublishedChildren)),
    ] {
        let mut refusals_after_a_record = 0;
        for index in 0..10_000 {
            let (storage, [root, ..]) = make();
            let before = snapshot(&storage);
            let mut meter = Meter {
                huge_bank,
                ..Meter::refusing_at(index)
            };
            let (storage, error) = match publish_tree(storage, &root, &mut meter) {
                Err(refusal) => *refusal,
                Ok(_) => {
                    assert!(index > 0);
                    break;
                }
            };
            assert!(matches!(error, HtmlChargedError::Stopped(Refused)));
            assert_eq!(snapshot(&storage), before, "refused at the call {index}");
            // A consumer that refuses to take a record back holds it; every other refusal
            // leaves it holding nothing.
            let refused_take_back = meter.refused_record.is_some_and(|record| {
                record.reached() == record.growth().before()
                    && record.recorded() != record.growth().before()
            });
            if refused_take_back {
                continue;
            }
            if !meter.records.is_empty() {
                refusals_after_a_record += 1;
            }
            assert!(
                holds_nothing_of_the_publication(&meter),
                "refused at the call {index} ({:?}): the consumer holds {:?}",
                meter.calls.last(),
                consumer_holdings(&meter)
            );
        }
        assert!(
            refusals_after_a_record >= 4,
            "the refusals after a record are the later steps"
        );
    }
}

#[test]
fn a_publication_that_packs_is_refused_at_every_admission_point_with_the_storage_as_it_was() {
    let calls = sweep(spare_tree, |storage, [root, ..], meter| {
        let taken = std::mem::replace(
            storage,
            ok(HtmlConstructionStorage::new(&mut Meter::default())),
        );
        match publish_tree(taken, root, meter) {
            Ok(_) => Ok(()),
            Err(refusal) => {
                let (back, error) = *refusal;
                *storage = back;
                Err(error)
            }
        }
    });
    assert!(calls.iter().filter(|call| **call == "admit_growth").count() >= 4);
}

#[test]
fn an_allocator_that_refuses_the_spare_room_of_a_publication_bank_is_asked_for_the_room_needed() {
    for (bank, needed) in [
        (HtmlConstructionBank::TextBlock, 1),
        (HtmlConstructionBank::PublishedRows, 5),
        (HtmlConstructionBank::PublishedChildren, 3),
    ] {
        let (storage, [root, ..]) = spare_tree();
        let mut meter = Meter {
            huge_bank: Some(bank),
            ..Meter::default()
        };
        let Ok(result) = publish_tree(storage, &root, &mut meter) else {
            panic!("publication of {bank:?} refused");
        };
        let records = growth_records(&meter, |kind| kind == bank);
        assert_eq!(
            records.len(),
            2,
            "{bank:?}: the choice and the capacity reached"
        );
        assert_eq!(
            records[0].chosen(),
            isize::MAX as usize
                / meter
                    .growths
                    .iter()
                    .find(|g| g.bank() == bank)
                    .unwrap()
                    .element_size()
        );
        assert_eq!(records[1].reached().capacity(), needed, "{bank:?}");
        assert_eq!(result.len(), 5);
        assert_eq!(published_text(&result).code_units(), utf16("t"));
    }
}

#[test]
fn a_text_the_block_cannot_share_refuses_the_publication_and_returns_the_text_whole() {
    for (name, make) in [
        (
            "packed",
            spare_tree as fn() -> (HtmlConstructionStorage, [HtmlBuilderNode; 4]),
        ),
        (
            "not packed",
            tree as fn() -> (HtmlConstructionStorage, [HtmlBuilderNode; 4]),
        ),
    ] {
        let (storage, [root, ..]) = make();
        let before = snapshot(&storage);
        let units = storage.text.len();
        let refused = crate::internal::amortized::fault::refuse_text_share();
        let mut meter = Meter::default();
        let (mut storage, error) = match publish_tree(storage, &root, &mut meter) {
            Err(refusal) => *refusal,
            Ok(_) => panic!("{name}: published a text the block cannot share"),
        };
        assert!(
            holds_nothing_of_the_publication(&meter),
            "{name}: the consumer holds {:?}",
            consumer_holdings(&meter)
        );
        assert!(
            consumer_holdings(&meter).len() >= 3,
            "{name}: the banks were recorded before the refusal"
        );
        let HtmlChargedError::Faulted(error) = error else {
            panic!("{name}: the consumer refused");
        };
        assert_eq!(
            error.cause(),
            HtmlConstructionCause::InvalidTextReference,
            "{name}"
        );
        drop(refused);
        // The arena holds the units it held, and the text reads as before.
        assert_eq!(snapshot(&storage), before, "{name}");
        assert_eq!(storage.text.len(), units, "{name}");
        let kind = storage.node_at(3).unwrap().kind();
        assert!(
            matches!(kind, crate::internal::construction::ConstructionNodeKind::Text(text)
                if text.code_units().unwrap() == utf16("t")),
            "{name}"
        );
        // The storage publishes when the share is not refused.
        let Ok(result) = publish_tree(
            std::mem::replace(
                &mut storage,
                ok(HtmlConstructionStorage::new(&mut Meter::default())),
            ),
            &root,
            &mut Meter::default(),
        ) else {
            panic!("{name}: the storage did not publish after the refusal");
        };
        assert_eq!(published_text(&result).code_units(), utf16("t"), "{name}");
    }
}

#[test]
fn a_text_record_that_names_units_the_arena_lacks_refuses_the_publication() {
    let (mut storage, [root, ..]) = tree();
    storage.arena.records[3].kind = NodeContent::Text(TextRun::invalid());
    let before = snapshot(&storage);
    let (storage, error) = match publish_tree(storage, &root, &mut Meter::default()) {
        Err(refusal) => *refusal,
        Ok(_) => panic!("published a text with no units"),
    };
    let HtmlChargedError::Faulted(error) = error else {
        panic!("the consumer refused");
    };
    assert_eq!(error.cause(), HtmlConstructionCause::InvalidTextReference);
    assert_eq!(snapshot(&storage), before);
}

#[test]
fn a_node_of_a_storage_has_no_published_view_and_a_node_of_its_result_has() {
    let (storage, [root, ..]) = tree();
    let error = storage.node_at(3).unwrap().as_node().unwrap_err();
    assert_eq!(error.cause(), HtmlConstructionCause::NotPublished);
    let Ok(result) = publish_tree(storage, &root, &mut Meter::default()) else {
        panic!("publication refused");
    };
    assert!(result.node_at(3).unwrap().as_node().is_ok());
}

#[test]
fn rolling_back_a_native_append_takes_the_units_of_its_texts_out_of_the_count() {
    let mut storage = storage();
    ok(storage.create_fragment(&mut Meter::default()));
    let (len, usage) = (storage.len(), storage.usage());
    ok(storage.create_text(&utf16("abc"), &mut Meter::default()));
    ok(storage.create_comment(&utf16("de"), &mut Meter::default()));
    assert_eq!(storage.text.used(), 5);
    storage.rollback_native_append(len, usage).unwrap();
    assert_eq!(storage.len(), len);
    assert_eq!(storage.text.used(), 0);
    // The units stay written, so the arena has room for a later text only past them.
    assert_eq!(storage.text.len(), 5);
}

/// A fragment with one `div` that has the attributes `a`, `b` and `c`, a text and a comment, and an
/// `em` that follows it and has no attribute, whose
/// arena has more than a quarter of its units spare when `spare` and whose attribute `b` was
/// replaced by a longer value, which leaves its old units as dead units.
fn attributed_tree(spare: bool) -> (HtmlConstructionStorage, [HtmlBuilderNode; 4]) {
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    let parent = element(&mut storage, "div");
    for (local, value) in [("a", "1"), ("b", "22"), ("c", "333")] {
        ok(storage.append_attribute(
            &parent,
            &Attribute::new(QualifiedName::new(Namespace::none(), None, local), value),
            None,
            &mut Meter::default(),
        ));
    }
    ok(storage.replace_attribute(
        &parent,
        1,
        &Attribute::new(
            QualifiedName::new(Namespace::none(), None, "b"),
            "twenty-two",
        ),
        &mut Meter::default(),
    ));
    let text = ok(storage.create_text(&utf16("t"), &mut Meter::default()));
    let comment = ok(storage.create_comment(&utf16("cc"), &mut Meter::default()));
    let fresh = element(&mut storage, "em");
    insert(&mut storage, &parent, &root);
    insert(&mut storage, &fresh, &root);
    insert(&mut storage, &text, &parent);
    insert(&mut storage, &comment, &parent);
    if spare {
        storage.text.reserve_exact(100).unwrap();
    }
    (storage, [root, parent, fresh, text])
}

fn published_attributes(
    result: &crate::internal::construction::bounded::HtmlConstructionResult,
    position: usize,
) -> Vec<(String, String, usize)> {
    let node = result.node_at(position).unwrap().as_node().unwrap();
    let crate::internal::construction::HtmlNodeKind::Element(element) = node.kind() else {
        panic!("an element record");
    };
    element
        .attributes()
        .iter()
        .map(|attribute| {
            (
                attribute.name().local_name().to_utf8().unwrap(),
                attribute.value().to_utf8().unwrap(),
                attribute.value().borrowed_block().unwrap().address(),
            )
        })
        .collect()
}

#[test]
fn the_published_attributes_read_back_exactly_in_the_one_block_with_the_text() {
    for spare in [false, true] {
        let (storage, [root, parent, fresh, ..]) = attributed_tree(spare);
        let mut meter = Meter::default();
        let Ok(result) = publish_tree(storage, &root, &mut meter) else {
            panic!("publication refused");
        };
        let banks: Vec<_> = meter.growths.iter().map(|growth| growth.bank()).collect();
        assert_eq!(
            banks[..3],
            [
                HtmlConstructionBank::Wrapper,
                HtmlConstructionBank::PublishedAttributes,
                HtmlConstructionBank::PublishedRuns,
            ]
        );
        assert_eq!(
            banks.contains(&HtmlConstructionBank::TextBlock),
            spare,
            "the arena is packed when more than a quarter is spare"
        );
        // The table of the result is exact: three rows of 24 bytes, whatever the storage held.
        assert_eq!(meter.growths[1].minimum_capacity(), 3);
        assert_eq!(
            meter.growths[1].element_size(),
            size_of::<crate::internal::construction::attribute_table::PublishedAttr>()
        );
        let at = result.position(&parent).unwrap();
        let attributes = published_attributes(&result, at);
        let seen: Vec<(&str, &str)> = attributes
            .iter()
            .map(|(name, value, _)| (name.as_str(), value.as_str()))
            .collect();
        assert_eq!(seen, [("a", "1"), ("b", "twenty-two"), ("c", "333")]);
        // Every value and every text borrows the one block.
        let node = result.node_at(at).unwrap().as_node().unwrap();
        let text = node.children().next().unwrap();
        let crate::internal::construction::HtmlNodeKind::Text(text) =
            result.node(&text).unwrap().kind()
        else {
            panic!("a text record");
        };
        let block = text.borrowed_block().unwrap().address();
        assert!(attributes.iter().all(|(_, _, held)| *held == block));
        // The element with attributes reads them back, and an index past the end reads as none.
        let node = result.node_at(at).unwrap().as_node().unwrap();
        let crate::internal::construction::HtmlNodeKind::Element(element) = node.kind() else {
            panic!("an element record");
        };
        assert_eq!(element.attributes().len(), 3);
        assert!(element.attributes().get(3).is_none());
        assert!(element.attributes().get(usize::MAX).is_none());
        assert_eq!(element.attributes().to_ordered().len(), 3);
        // The element without attributes holds none.
        let empty = result.position(&fresh).unwrap();
        let node = result.node_at(empty).unwrap().as_node().unwrap();
        let crate::internal::construction::HtmlNodeKind::Element(element) = node.kind() else {
            panic!("an element record");
        };
        assert_eq!(element.attributes().len(), 0);
        assert!(element.attributes().is_empty());
        assert!(element.attributes().get(0).is_none());
        assert!(element.attributes().iter().next().is_none());
    }
}

/// A fragment of three elements: `p` with the attributes `x` and `y`, `q` with `a`, `b`, `c`,
/// `d` and `e`, and `r` with none. The rows of `q` do not start at the first row of the table.
fn three_elements() -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    for (local, attributes) in [
        ("p", &["x", "y"][..]),
        ("q", &["a", "b", "c", "d", "e"][..]),
        ("r", &[][..]),
    ] {
        let node = element(&mut storage, local);
        for attribute in attributes {
            ok(storage.append_attribute(
                &node,
                &Attribute::new(
                    QualifiedName::new(Namespace::none(), None, *attribute),
                    format!("v-{attribute}").as_str(),
                ),
                None,
                &mut Meter::default(),
            ));
        }
        insert(&mut storage, &node, &root);
    }
    (storage, root)
}

#[test]
fn an_attribute_is_read_at_every_index_of_a_build_row_and_a_published_row() {
    let expected: [&[&str]; 3] = [&["x", "y"], &["a", "b", "c", "d", "e"], &[]];
    // Records 1 to 3 are the elements, after the fragment root.
    let (storage, root) = three_elements();
    for (record, names) in expected.iter().enumerate() {
        let view = storage.node_at(record + 1).unwrap().kind();
        let crate::internal::construction::ConstructionNodeKind::Element(element) = view else {
            panic!("an element record");
        };
        let attributes = element.attributes();
        assert_eq!(attributes.len().unwrap(), names.len());
        for (index, local) in names.iter().enumerate() {
            let attribute = attributes
                .get(index)
                .unwrap()
                .expect("an attribute at the index");
            assert_eq!(attribute.name().local_name().to_utf8().unwrap(), *local);
            assert_eq!(
                attribute.value().code_units().unwrap(),
                format!("v-{local}").encode_utf16().collect::<Vec<_>>()
            );
        }
        assert!(attributes.get(names.len()).unwrap().is_none());
        assert!(attributes.get(usize::MAX).unwrap().is_none());
        // The iterator reaches an index by `nth` and `nth_back` too.
        for (index, local) in names.iter().enumerate() {
            let reached = attributes.iter().unwrap().nth(index).unwrap();
            assert_eq!(reached.name().local_name().to_utf8().unwrap(), *local);
            let back = names.len() - 1 - index;
            let reached = attributes.iter().unwrap().nth_back(index).unwrap();
            assert_eq!(reached.name().local_name().to_utf8().unwrap(), names[back]);
        }
        assert!(attributes.iter().unwrap().nth(names.len()).is_none());
        assert!(attributes.iter().unwrap().nth_back(names.len()).is_none());
    }
    let Ok(result) = publish_tree(storage, &root, &mut Meter::default()) else {
        panic!("publication refused");
    };
    for (record, names) in expected.iter().enumerate() {
        let node = result.node_at(record + 1).unwrap().as_node().unwrap();
        let crate::internal::construction::HtmlNodeKind::Element(element) = node.kind() else {
            panic!("an element record");
        };
        let attributes = element.attributes();
        assert_eq!(attributes.len(), names.len());
        for (index, local) in names.iter().enumerate() {
            let attribute = attributes.get(index).expect("an attribute at the index");
            assert_eq!(attribute.name().local_name().to_utf8().unwrap(), *local);
            assert_eq!(attribute.value().to_utf8().unwrap(), format!("v-{local}"));
        }
        assert!(attributes.get(names.len()).is_none());
        assert!(attributes.get(usize::MAX).is_none());
        for (index, local) in names.iter().enumerate() {
            let reached = attributes.iter().nth(index).unwrap();
            assert_eq!(reached.name().local_name().to_utf8().unwrap(), *local);
            let reached = attributes.iter().nth_back(index).unwrap();
            assert_eq!(
                reached.name().local_name().to_utf8().unwrap(),
                names[names.len() - 1 - index]
            );
        }
        assert!(attributes.iter().nth(names.len()).is_none());
        assert!(attributes.iter().nth_back(names.len()).is_none());
    }
}

/// The writes the reservation of room for `additional` attributes charges for the element `node`.
fn reservation_writes(
    storage: &mut HtmlConstructionStorage,
    node: &HtmlBuilderNode,
    additional: usize,
) -> u64 {
    let mut meter = Meter::default();
    if storage
        .reserve_attributes(node, additional, &mut meter)
        .is_err()
    {
        panic!("the reservation is refused");
    }
    meter.work[WRITE]
}

#[test]
fn a_reservation_charges_a_write_for_every_row_it_fills() {
    let attribute =
        |local: &str| Attribute::new(QualifiedName::new(Namespace::none(), None, local), "v");
    // An element without a run gets a new run of the rows reserved: the two writes of the run
    // and one write for each filler row.
    let mut storage = storage();
    let fresh = element(&mut storage, "p");
    assert_eq!(reservation_writes(&mut storage, &fresh, 5), 2 + 5);
    // A run that is not at the tail of the table moves to a region of twice its new size: one
    // write for each of its two live rows that moves, the two writes of the run, and one write
    // for each of the ten rows of the region, which the old rows' filler and the room fill.
    let mut storage = storage_with_two_runs(attribute);
    let first = storage.node_at(0).map(|record| record.handle()).unwrap();
    assert_eq!(reservation_writes(&mut storage, &first, 3), 2 + 2 + 10);
}

/// A storage with an element of two attributes whose run is followed by that of another.
fn storage_with_two_runs(attribute: impl Fn(&str) -> Attribute) -> HtmlConstructionStorage {
    let mut storage = storage();
    for (local, attributes) in [("p", &["a", "b"][..]), ("q", &["c"][..])] {
        let node = element(&mut storage, local);
        for name in attributes {
            ok(storage.append_attribute(&node, &attribute(name), None, &mut Meter::default()));
        }
    }
    storage
}

/// A consumer whose ledger holds the usage of every change it recorded, which refuses a change
/// that takes its attributes past `attribute_limit` and the call numbered `refuse_at`.
#[derive(Default)]
struct Ledger {
    attribute_limit: Option<usize>,
    /// The room a bank is given beyond the minimum it needs.
    spare: usize,
    refuse_at: Option<usize>,
    calls: usize,
    nodes: usize,
    attributes: usize,
    units: usize,
}
impl Ledger {
    fn call(&mut self) -> Result<(), Refused> {
        let index = self.calls;
        self.calls += 1;
        if self.refuse_at == Some(index) {
            Err(Refused)
        } else {
            Ok(())
        }
    }
    fn holds_nothing(&self) -> bool {
        (self.nodes, self.attributes, self.units) == (0, 0, 0)
    }
}
impl Charge<HtmlWork, HtmlResource> for Ledger {
    type Stop = Refused;
    fn charge(&mut self, _: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
        self.call()
    }
    fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), Refused> {
        self.call()
    }
}
impl HtmlRecordAdmission for Ledger {
    fn admit_change(
        &mut self,
        _: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), Refused> {
        self.call()?;
        let added = change.after().attributes() - change.before().attributes();
        match self.attribute_limit {
            Some(limit) if self.attributes + added > limit => Err(Refused),
            _ => Ok(()),
        }
    }
    fn admit_growth(&mut self, growth: HtmlConstructionBankGrowth) -> Result<usize, Refused> {
        self.call()?;
        Ok(growth.minimum_capacity() + self.spare)
    }
    fn record_growth(&mut self, _: HtmlConstructionGrowthRecord) -> Result<(), Refused> {
        self.call()
    }
    fn record_change(
        &mut self,
        _: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), Refused> {
        self.call()?;
        self.nodes += change.after().nodes() - change.before().nodes();
        self.attributes += change.after().attributes() - change.before().attributes();
        self.units += change.after().utf16_units() - change.before().utf16_units();
        Ok(())
    }
}

/// An element `p` with the attributes `a0` to `a3`, whose names the storage has interned.
fn authored_element(storage: &mut HtmlConstructionStorage) -> (QualifiedName, OrderedAttributes) {
    let attributes: Vec<Attribute> = (0..4)
        .map(|index| {
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, format!("a{index}").as_str()),
                format!("value {index}").as_str(),
            )
        })
        .collect();
    let element = name("p");
    for qualified in std::iter::once(&element).chain(attributes.iter().map(Attribute::name)) {
        ok(storage.intern_name(
            qualified.namespace(),
            qualified.prefix(),
            qualified.local_name().code_units(),
            &mut Meter::default(),
        ));
    }
    (element, OrderedAttributes::new(attributes).unwrap())
}

#[test]
fn a_consumer_limit_that_refuses_an_authored_attribute_leaves_its_ledger_unchanged() {
    let mut storage = storage();
    let (element, attributes) = authored_element(&mut storage);
    let before = snapshot(&storage);
    for limit in 0..4 {
        let mut ledger = Ledger {
            attribute_limit: Some(limit),
            ..Ledger::default()
        };
        let outcome = storage.create_element(&element, attributes.clone(), None, &mut ledger);
        assert!(
            matches!(outcome, Err(HtmlChargedError::Stopped(Refused))),
            "limit {limit}"
        );
        assert!(
            ledger.holds_nothing(),
            "limit {limit}: the ledger holds {} nodes, {} attributes and {} units",
            ledger.nodes,
            ledger.attributes,
            ledger.units
        );
        assert_eq!(snapshot(&storage), before, "limit {limit}");
    }
    // A limit that admits every attribute admits the element: its name, then each attribute.
    let mut ledger = Ledger {
        attribute_limit: Some(4),
        ..Ledger::default()
    };
    ok(storage.create_element(&element, attributes.clone(), None, &mut ledger));
    let units = |qualified: &QualifiedName, value: usize| {
        qualified.local_name().len() + qualified.prefix().map_or(0, DocumentString::len) + value
    };
    let expected_units = units(&element, 0)
        + element.namespace().uri().map_or(0, DocumentString::len)
        + attributes
            .iter()
            .map(|attribute| units(attribute.name(), attribute.value().len()))
            .sum::<usize>();
    assert_eq!(
        (ledger.nodes, ledger.attributes, ledger.units),
        (1, 4, expected_units)
    );
}

#[test]
fn a_refusal_at_any_step_of_creating_an_authored_element_leaves_storage_and_ledger_as_they_were() {
    // Each attempt has a fresh storage, so that every attempt makes the calls of a creation that
    // grows every bank: a bank that an earlier refused attempt grew keeps its room.
    let mut refused = 0;
    for index in 0.. {
        let mut storage = storage();
        let (element, attributes) = authored_element(&mut storage);
        let before = snapshot(&storage);
        let mut ledger = Ledger {
            refuse_at: Some(index),
            ..Ledger::default()
        };
        match storage.create_element(&element, attributes, None, &mut ledger) {
            Ok(_) => break,
            Err(HtmlChargedError::Stopped(Refused)) => {
                refused += 1;
                assert!(ledger.holds_nothing(), "refused at call {index}");
                assert_eq!(snapshot(&storage), before, "refused at call {index}");
            }
            Err(other) => panic!("call {index}: {other:?}"),
        }
    }
    // The calls the consumer receives, from the code of the creation of `p` with four attributes
    // in a storage that holds the five names and no record:
    // - the layout of the vector of interned names: 1;
    // - the lookup of each of the five names: a charge for the visit, a charge for the probe
    //   allowance and a charge for the one comparison of an equal hash: 3 x 5;
    // - the layouts of the record, the run, the rows and the depth, and the admission of the
    //   change: 5;
    // - the charge of the writes: 1, of the filled rows (one quantum): 1, and of the run: 1;
    // - for each attribute the charge of its value (one chunk) and of its row: 2 x 4;
    // - the admission of the four banks (records, text, runs, rows): 4, and the record of each
    //   at the capacity chosen, which the allocator reaches exactly: 4;
    // - the record of the change: 1.
    let expected = 1 + 3 * 5 + 5 + (1 + 1 + 1) + 2 * 4 + 4 + 4 + 1;
    assert_eq!(expected, 41);
    assert_eq!(refused, expected, "the creation made {refused} calls");
}

#[test]
fn a_refused_allocator_request_at_each_point_of_creating_an_authored_element_is_all_or_nothing() {
    use crate::internal::amortized::fault;
    // The creation makes five requests: the vector of the interned names of the attributes, then
    // the banks it grows (records, text, runs and rows), each with spare room.
    let mut counted = storage();
    let (element, attributes) = authored_element(&mut counted);
    let mut ledger = Ledger {
        spare: 4,
        ..Ledger::default()
    };
    let counting = fault::count_only();
    ok(counted.create_element(&element, attributes, None, &mut ledger));
    let requests = fault::requests();
    drop(counting);
    assert_eq!(requests, 1 + 4);
    let units = ledger.units;
    for refuse_from in 1..=requests {
        let mut storage = storage();
        let (element, attributes) = authored_element(&mut storage);
        let before = snapshot(&storage);
        let mut ledger = Ledger {
            spare: 4,
            ..Ledger::default()
        };
        let _armed = fault::arm(refuse_from);
        let outcome = storage.create_element(&element, attributes, None, &mut ledger);
        if refuse_from == 1 {
            // The vector of names has no fallback: its refusal is the allocation failure of the
            // creation, which leaves the storage and the ledger as they were.
            match outcome {
                Err(HtmlChargedError::Faulted(error)) => {
                    assert_eq!(error.cause(), HtmlConstructionCause::AllocationFailed);
                }
                other => panic!("request {refuse_from}: {other:?}"),
            }
            assert!(ledger.holds_nothing(), "request {refuse_from}");
            assert_eq!(snapshot(&storage), before, "request {refuse_from}");
        } else {
            // A refusal of spare room falls back to the room that is needed.
            let node = ok(outcome);
            assert_eq!(
                (ledger.nodes, ledger.attributes, ledger.units),
                (1, 4, units),
                "request {refuse_from}"
            );
            let at = storage.position(&node).unwrap();
            let crate::internal::construction::ConstructionNodeKind::Element(element) =
                storage.node_at(at).unwrap().kind()
            else {
                panic!("an element record");
            };
            assert_eq!(element.attributes().len().unwrap(), 4);
        }
    }
}

#[test]
fn an_authored_template_with_attributes_reads_them_back_built_and_published() {
    use crate::internal::construction::{ConstructionNodeKind, HtmlNodeKind};
    let mut storage = storage();
    let root = ok(storage.create_fragment(&mut Meter::default()));
    let attributes = OrderedAttributes::new([
        Attribute::new(QualifiedName::new(Namespace::none(), None, "id"), "shell"),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "class"), "a b"),
    ])
    .unwrap();
    let template =
        ok(storage.create_element(&name("template"), attributes, None, &mut Meter::default()));
    // The template record is followed by its owned contents, which carry no attributes.
    assert_eq!(storage.len(), 3);
    let at = position(&storage, &template);
    let expected = [("id", "shell"), ("class", "a b")];
    let ConstructionNodeKind::Element(element) = storage.node_at(at).unwrap().kind() else {
        panic!("an element record");
    };
    let attributes = element.attributes();
    assert_eq!(attributes.len().unwrap(), 2);
    for (index, (local, value)) in expected.iter().enumerate() {
        let attribute = attributes.get(index).unwrap().expect("an attribute");
        assert_eq!(attribute.name().local_name().to_utf8().unwrap(), *local);
        assert_eq!(attribute.value().code_units().unwrap(), utf16(value));
    }
    let contents = storage
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap();
    assert_eq!(contents.position(), at + 1);
    insert(&mut storage, &template, &root);
    let Ok(result) = publish_tree(storage, &root, &mut Meter::default()) else {
        panic!("publication refused");
    };
    let published = published_attributes(&result, 1);
    let seen: Vec<(&str, &str)> = published
        .iter()
        .map(|(name, value, _)| (name.as_str(), value.as_str()))
        .collect();
    assert_eq!(seen, expected);
    let node = result.node_at(1).unwrap().as_node().unwrap();
    let HtmlNodeKind::Element(element) = node.kind() else {
        panic!("an element record");
    };
    assert_eq!(element.attributes().len(), 2);
}

#[test]
fn a_publication_with_attributes_is_refused_at_every_admission_point_and_takes_every_bank_back() {
    for spare in [false, true] {
        let setup = move || attributed_tree(spare);
        let calls = sweep(setup, |storage, [root, ..], meter| {
            let taken = std::mem::replace(
                storage,
                ok(HtmlConstructionStorage::new(&mut Meter::default())),
            );
            match publish_tree(taken, root, meter) {
                Ok(_) => Ok(()),
                Err(refusal) => {
                    let (back, error) = *refusal;
                    *storage = back;
                    Err(error)
                }
            }
        });
        let growths = calls.iter().filter(|call| **call == "admit_growth").count();
        assert_eq!(growths, 5 + usize::from(spare), "spare: {spare}");
    }
    // The consumer's own ledger: a refusal after every bank was recorded leaves it holding
    // what it held before the publication, the published attribute table included.
    for spare in [false, true] {
        let (storage, [root, ..]) = attributed_tree(spare);
        let refused = crate::internal::amortized::fault::refuse_text_share();
        let mut meter = Meter::default();
        let Err(refusal) = publish_tree(storage, &root, &mut meter) else {
            panic!("published a value the block cannot share");
        };
        drop(refused);
        let (_, error) = *refusal;
        let HtmlChargedError::Faulted(error) = error else {
            panic!("the consumer refused");
        };
        assert_eq!(error.cause(), HtmlConstructionCause::InvalidTextReference);
        assert!(holds_nothing_of_the_publication(&meter), "spare: {spare}");
        assert!(
            consumer_holdings(&meter)
                .iter()
                .any(|(bank, ..)| *bank == HtmlConstructionBank::PublishedAttributes),
            "the table was recorded before the refusal"
        );
    }
}

#[test]
fn a_value_the_arena_lacks_and_a_run_the_table_lacks_refuse_the_publication_and_read_as_invalid() {
    use crate::internal::construction::attribute_table::AttrRow;
    // A value whose range the text arena does not hold.
    let (mut storage, [root, ..]) = attributed_tree(false);
    let target = storage.attrs.live(match &storage.arena.records[1].kind {
        NodeContent::Element(element) => element.attrs,
        _ => panic!("the first child record is the div"),
    });
    let first = target.unwrap()[0].entry().clone();
    let run = match &storage.arena.records[1].kind {
        NodeContent::Element(element) => element.attrs.unwrap(),
        _ => unreachable!(),
    };
    storage
        .attrs
        .replace(run, 0, AttrRow::new(first, TextRun::invalid().text()))
        .unwrap();
    let before = snapshot(&storage);
    let view = storage.node_at(1).unwrap().kind();
    let crate::internal::construction::ConstructionNodeKind::Element(element) = view else {
        panic!("an element record");
    };
    let value = element.attributes().get(0).unwrap().unwrap().value();
    assert_eq!(
        value.code_units().unwrap_err().cause(),
        HtmlConstructionCause::InvalidTextReference
    );
    let (storage, error) = match publish_tree(storage, &root, &mut Meter::default()) {
        Err(refusal) => *refusal,
        Ok(_) => panic!("published a value with no units"),
    };
    let HtmlChargedError::Faulted(error) = error else {
        panic!("the consumer refused");
    };
    assert_eq!(error.cause(), HtmlConstructionCause::InvalidTextReference);
    assert_eq!(snapshot(&storage), before);
    // An element that names a run the table does not hold.
    let (mut storage, [root, ..]) = attributed_tree(false);
    if let NodeContent::Element(element) = &mut storage.arena.records[1].kind {
        element.attrs = Some(crate::internal::construction::attribute_table::RunIx::beyond());
    }
    let view = storage.node_at(1).unwrap().kind();
    let crate::internal::construction::ConstructionNodeKind::Element(element) = view else {
        panic!("an element record");
    };
    assert_eq!(
        element.attributes().len().unwrap_err().cause(),
        HtmlConstructionCause::InvalidAttributeReference
    );
    assert!(element.attributes().iter().is_err());
    let before = snapshot(&storage);
    let (storage, error) = match publish_tree(storage, &root, &mut Meter::default()) {
        Err(refusal) => *refusal,
        Ok(_) => panic!("published an element with no run"),
    };
    let HtmlChargedError::Faulted(error) = error else {
        panic!("the consumer refused");
    };
    assert_eq!(
        error.cause(),
        HtmlConstructionCause::InvalidAttributeReference
    );
    assert_eq!(snapshot(&storage), before);
}

/// The banks a publication recorded with its consumer, in the order it recorded them.
fn recorded_banks(meter: &Meter) -> Vec<HtmlConstructionBank> {
    meter
        .records
        .iter()
        .map(|record| record.growth().bank())
        .collect()
}

#[test]
fn a_published_tree_retains_twenty_bytes_a_node_four_a_child_entry_and_eight_an_attribute_run() {
    let (storage, [root, ..]) = attributed_tree(false);
    let mut meter = Meter::default();
    let Ok(result) = publish_tree(storage, &root, &mut meter) else {
        panic!("publication refused");
    };
    // The nodes, the child entries and the runs are read from the published tree: the entries are
    // the children its rows list, and a run is held by each element that has attributes.
    let nodes = result.len();
    let mut children = 0usize;
    let mut runs = 0usize;
    for position in 0..nodes {
        let node = result.node_at(position).unwrap().as_node().unwrap();
        children += node.children().len();
        if let crate::internal::construction::HtmlNodeKind::Element(element) = node.kind() {
            runs += usize::from(!element.attributes().is_empty());
        }
    }
    // The fixture is one fragment root with two elements, a text and a comment below it, and one
    // element with attributes. Every node but the one root is listed once.
    assert_eq!((nodes, children, runs), (5, 4, 1));
    assert_eq!(children, nodes - 1);
    // The banks that describe the shape of the tree are the rows, the child entries and the
    // runs; the tree has no associations or leaves, so no bank is made for them. The text block
    // is not packed, and the published attribute rows and the wrapper of the result are not part
    // of the shape.
    assert_eq!(
        recorded_banks(&meter),
        [
            HtmlConstructionBank::Wrapper,
            HtmlConstructionBank::PublishedAttributes,
            HtmlConstructionBank::PublishedRuns,
            HtmlConstructionBank::PublishedRows,
            HtmlConstructionBank::PublishedChildren,
        ]
    );
    let held = |bank| -> usize {
        meter
            .records
            .iter()
            .filter(|record| record.growth().bank() == bank)
            .map(|record| record.reached().capacity() * record.growth().element_size())
            .sum()
    };
    assert_eq!(held(HtmlConstructionBank::PublishedRows), 20 * nodes);
    assert_eq!(held(HtmlConstructionBank::PublishedChildren), 4 * children);
    assert_eq!(held(HtmlConstructionBank::PublishedRuns), 8 * runs);
}

#[test]
fn a_publication_makes_no_bank_of_contents_for_its_records() {
    // Neither tree has an association or a leaf, and the banks recorded are those of the shared
    // tree, the attributes, the rows and the children: no bank holds a content for each record.
    for (storage, [root, ..]) in [attributed_tree(false), attributed_tree(true)] {
        let mut meter = Meter::default();
        let Ok(_) = publish_tree(storage, &root, &mut meter) else {
            panic!("publication refused");
        };
        let banks = recorded_banks(&meter);
        let allowed = [
            HtmlConstructionBank::Wrapper,
            HtmlConstructionBank::PublishedAttributes,
            HtmlConstructionBank::PublishedRuns,
            HtmlConstructionBank::PublishedRows,
            HtmlConstructionBank::PublishedChildren,
            HtmlConstructionBank::TextBlock,
        ];
        assert!(banks.iter().all(|bank| allowed.contains(bank)), "{banks:?}");
    }
}

/// A document with a doctype, a processing instruction, two templates (one owning its contents,
/// one referring to shadow contents), a shadow host, text, a comment, attributes and an element
/// that nothing holds; the builder handle of the document is the second value.
fn rich_tree() -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let m = Meter::default;
    let mut storage = storage();
    let document = ok(storage.create_document(HtmlDocumentMode::Quirks, &mut m()));
    let doctype = ok(storage.create_doctype(&utf16("html"), Some(&utf16("p")), None, &mut m()));
    let instruction = ok(storage.create_processing_instruction(&utf16("t"), &utf16("d"), &mut m()));
    let html = element(&mut storage, "html");
    for node in [&doctype, &instruction, &html] {
        insert(&mut storage, node, &document);
    }
    // A template that owns its contents, which hold a text.
    let template = element(&mut storage, "template");
    insert(&mut storage, &template, &html);
    let contents = storage
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let inside = ok(storage.create_text(&utf16("in"), &mut m()));
    insert(&mut storage, &inside, &contents);
    // A host with a shadow root.
    let host = element(&mut storage, "div");
    insert(&mut storage, &host, &html);
    let shadow = ok(storage.create_shadow_contents(shadow_properties(), &mut m()));
    let slot = element(&mut storage, "slot");
    insert(&mut storage, &slot, &shadow);
    ok(storage.attach_shadow(&host, &shadow, &mut m()));
    // A template that refers to the shadow contents of another host.
    let referring = element(&mut storage, "template");
    let other_host = element(&mut storage, "section");
    let other_shadow = ok(storage.create_shadow_contents(shadow_properties(), &mut m()));
    insert(&mut storage, &referring, &html);
    insert(&mut storage, &other_host, &html);
    ok(storage.attach_template_shadow(&referring, &other_host, &other_shadow, &mut m()));
    // Attributes, text and a comment on the element of the first host, and a detached element.
    ok(storage.append_attribute(
        &host,
        &Attribute::new(QualifiedName::new(Namespace::none(), None, "id"), "h"),
        None,
        &mut m(),
    ));
    let text = ok(storage.create_text(&utf16("t"), &mut m()));
    let comment = ok(storage.create_comment(&utf16("c"), &mut m()));
    insert(&mut storage, &text, &host);
    insert(&mut storage, &comment, &host);
    let loose = element(&mut storage, "em");
    ok(storage.append_attribute(
        &loose,
        &Attribute::new(QualifiedName::new(Namespace::none(), None, "k"), "v"),
        None,
        &mut m(),
    ));
    (storage, document)
}

#[test]
fn a_published_tree_reads_every_node_as_the_storage_does_and_has_no_unreadable_node() {
    for (storage, root) in [rich_tree(), {
        let (storage, [root, ..]) = attributed_tree(true);
        (storage, root)
    }] {
        let before = view_snapshot(storage.len(), |at| storage.node_at(at));
        let Ok(result) = publish_tree(storage, &root, &mut Meter::default()) else {
            panic!("publication refused");
        };
        assert_eq!(view_snapshot(result.len(), |at| result.node_at(at)), before);
        for at in 0..result.len() {
            let node = result.node_at(at).unwrap();
            assert!(!matches!(
                node.kind(),
                crate::internal::construction::ConstructionNodeKind::Unreadable(_)
            ));
            assert!(!matches!(
                node.as_node().unwrap().kind(),
                HtmlNodeKind::Unreadable(_)
            ));
        }
    }
}

#[test]
fn a_published_tree_lists_every_node_with_a_parent_once_and_keeps_rows_for_detached_nodes() {
    let (storage, root) = rich_tree();
    let nodes = storage.len();
    let Ok(result) = publish_tree(storage, &root, &mut Meter::default()) else {
        panic!("publication refused");
    };
    let mut listed = vec![0usize; result.len()];
    for at in 0..result.len() {
        let node = result.node_at(at).unwrap();
        for child in node.children() {
            listed[child.position()] += 1;
            assert_eq!(child.parent().map(HtmlConstructionNode::position), Some(at));
        }
    }
    for (at, times) in listed.iter().enumerate() {
        let node = result.node_at(at).unwrap();
        assert_eq!(
            *times,
            usize::from(node.parent().is_some()),
            "node {at} is listed once when it has a parent and never otherwise"
        );
    }
    // The document, the contents of the first template, the contents the second template
    // released, both shadow contents and the detached element have no parent and keep a row.
    let parentless = (0..result.len())
        .filter(|at| result.node_at(*at).unwrap().parent().is_none())
        .count();
    assert_eq!(parentless, 6);
    assert_eq!(result.len(), nodes);
}

#[test]
fn a_publication_refuses_an_element_that_names_an_entry_its_names_do_not_hold() {
    // An entry of another table at the position of an entry of this one, and an entry at a
    // position this table does not have.
    for number in [0u32, 99] {
        let (mut storage, [root, ..]) = attributed_tree(false);
        if let NodeContent::Element(element) = &mut storage.arena.records[1].kind {
            element.name = Arc::new(crate::internal::construction::names::NameEntry::new(
                name("other"),
                0,
                number,
            ));
        }
        let before = snapshot(&storage);
        let mut meter = Meter::default();
        let (storage, error) = match publish_tree(storage, &root, &mut meter) {
            Err(refusal) => *refusal,
            Ok(_) => panic!("published an element with a foreign name"),
        };
        let HtmlChargedError::Faulted(error) = error else {
            panic!("the consumer refused");
        };
        assert_eq!(error.cause(), HtmlConstructionCause::InvalidNameReference);
        assert_eq!(snapshot(&storage), before);
        assert!(
            meter.growths.is_empty(),
            "the references are checked before a bank is admitted"
        );
    }
}

#[test]
fn a_row_that_names_a_reference_its_tree_does_not_hold_reads_as_the_typed_defect() {
    use crate::internal::construction::tree::{Leaf, PublishedRow, RowContent, Span, Tree};
    let row = |content| PublishedRow {
        parent: None,
        child_end: 0,
        content,
    };
    let mut tree = Tree::forged(vec![
        row(RowContent::Element {
            link_flags: None,
            name: 9,
            run: None,
        }),
        row(RowContent::Element {
            link_flags: None,
            name: 0,
            run: std::num::NonZeroU32::new(4),
        }),
        row(RowContent::Text(Span::new(0..3).unwrap())),
        row(RowContent::Comment(Span::new(0..3).unwrap())),
        row(RowContent::ProcessingInstruction(0)),
        row(RowContent::Doctype(1)),
        PublishedRow {
            parent: None,
            child_end: 7,
            content: RowContent::Fragment {
                shadow: None,
                owner: None,
            },
        },
    ]);
    tree.names.entries.push(Arc::new(
        crate::internal::construction::names::NameEntry::new(name("a"), 0, 0),
    ));
    tree.leaves.push(Leaf::Doctype(
        crate::internal::construction::HtmlDoctype::new("d", None, None),
    ));
    let cause = |at: usize| match tree.node(NodeIx::at(at)).kind() {
        HtmlNodeKind::Unreadable(cause) => Some(cause),
        _ => None,
    };
    assert_eq!(cause(0), Some(HtmlConstructionCause::InvalidNameReference));
    assert_eq!(
        cause(1),
        Some(HtmlConstructionCause::InvalidAttributeReference)
    );
    assert_eq!(cause(2), Some(HtmlConstructionCause::InvalidTextReference));
    assert_eq!(cause(3), Some(HtmlConstructionCause::InvalidTextReference));
    // A processing instruction row at a doctype leaf, and a doctype row past the leaves.
    assert_eq!(cause(4), Some(HtmlConstructionCause::InvalidLeafReference));
    assert_eq!(cause(5), Some(HtmlConstructionCause::InvalidLeafReference));
    // A fragment whose children end past the child array, and a row past the rows, are the
    // unknown node.
    assert_eq!(cause(6), Some(HtmlConstructionCause::UnknownNode));
    assert_eq!(cause(40), Some(HtmlConstructionCause::UnknownNode));
    assert_eq!(tree.node(NodeIx::at(6)).children().len(), 0);
}

#[test]
fn a_child_of_a_published_node_is_reached_by_its_index_with_one_step_from_either_end() {
    // Characterization of the readers, not of a defect: the child at an index is made without a
    // step over the children before it, whatever the fanout and the index.
    for fanout in [1000usize, 2000, 4000, 8000] {
        let mut storage = storage();
        let root = ok(storage.create_fragment(&mut Meter::default()));
        for _ in 0..fanout {
            let comment = ok(storage.create_comment(&utf16("c"), &mut Meter::default()));
            insert(&mut storage, &comment, &root);
        }
        let Ok(result) = publish_tree(storage, &root, &mut Meter::default()) else {
            panic!("publication refused");
        };
        let node = result.node_at(0).unwrap().as_node().unwrap();
        for index in [0, fanout / 2, fanout - 1] {
            let made = crate::internal::construction::tree::mapped::made;
            let before = made();
            let front = node.children().nth(index).unwrap();
            let back = node.children().nth_back(fanout - 1 - index).unwrap();
            assert_eq!(made() - before, 2, "fanout {fanout}, index {index}");
            assert_eq!(front, back);
        }
        assert_eq!(node.children().len(), fanout);
    }
}

#[test]
fn a_publication_with_associations_and_leaves_is_refused_at_every_admission_point_and_takes_every_bank_back()
 {
    let setup = || {
        let (storage, root) = rich_tree();
        (storage, [root; 4])
    };
    let calls = sweep(setup, |storage, [root, ..], meter| {
        let taken = std::mem::replace(
            storage,
            ok(HtmlConstructionStorage::new(&mut Meter::default())),
        );
        match publish_tree(taken, root, meter) {
            Ok(_) => Ok(()),
            Err(refusal) => {
                let (back, error) = *refusal;
                *storage = back;
                Err(error)
            }
        }
    });
    // The shared tree, the attribute rows and runs, the rows, the children, the associations and
    // the leaves; the text is not packed.
    assert_eq!(
        calls.iter().filter(|call| **call == "admit_growth").count(),
        7
    );
    // The consumer holds nothing of a publication refused after each record.
    for index in 0..calls.len() {
        let (storage, root) = rich_tree();
        let mut meter = Meter::refusing_at(index);
        let Err(refusal) = publish_tree(storage, &root, &mut meter) else {
            continue;
        };
        let (_, error) = *refusal;
        assert!(matches!(error, HtmlChargedError::Stopped(Refused)));
        assert!(
            holds_nothing_of_the_publication(&meter),
            "refused at the call {index}: the consumer holds {:?}",
            consumer_holdings(&meter)
        );
    }
}
