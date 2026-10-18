//! Small representation experiments. These are private candidate subgraphs,
//! not a second production model or a whole-DOM memory/savings assertion.
use super::super::{bank, compact_tests::AllocationGraph, source};
use super::*;
use crate::{
    DocumentId, DocumentString, DomDocumentType, DomHostedGlobalId, DomOwnerId,
    DomProcessingInstruction, NodeId,
};

fn unlimited(_: Usage) -> Result<(), DomError> {
    Ok(())
}
struct Common<K> {
    document: DocumentId,
    global: Option<DomHostedGlobalId>,
    parent: Option<NodeId>,
    kind: K,
    own: Usage,
}
struct Slot<K> {
    generation: u64,
    retired: bool,
    node: Option<Box<Common<K>>>,
}
struct Pair<K> {
    value: bank::Bank<Slot<K>>,
    custody: bank::BankCustody<Slot<K>>,
}
struct Input<K> {
    kind: K,
    fields: Usage,
    child: Option<bank::ObjectCustody>,
}

fn register<K>(input: Input<K>, registry: &bank::CustodyRegistry) -> Pair<K> {
    let reserved = bank::Record::reserve().unwrap();
    let own = Usage::new(size_of::<Slot<K>>() + size_of::<Common<K>>(), 0)
        .checked_add(input.fields)
        .unwrap();
    let debit = registry.ledger().retain(own, unlimited).unwrap();
    let document = DocumentId(NodeId {
        owner: DomOwnerId::allocate().unwrap(),
        slot: 0,
        generation: 0,
    });
    let mut held = custody::Held::new(
        Slot {
            generation: 0,
            retired: false,
            node: Some(Box::new(Common {
                document,
                global: None,
                parent: None,
                kind: input.kind,
                own,
            })),
        },
        debit,
    );
    // Original child custody remains in input until the complete record owns
    // the prepared dependency, including a dependency-admission refusal.
    let dependency = input
        .child
        .as_ref()
        .map(|child| bank::Dependency::new(child.clone(), registry.ledger(), unlimited).unwrap());
    let shared = held.debit_mut().share(unlimited).unwrap();
    let (slot, zero) = held.into_parts();
    drop(zero);
    let mut record = bank::ControlledRecord::from_parts(reserved.materialize(slot), shared);
    if let Some(dependency) = dependency {
        record = record.with_dependency(dependency);
    }
    let (value, custody) = bank::Bank::default()
        .set_controlled(
            &bank::BankCustody::default(),
            0,
            record,
            false,
            bank::Totals {
                bytes: own.bytes(),
                units: own.units(),
                occurrences: 1,
            },
            registry,
            unlimited,
        )
        .unwrap();
    Pair { value, custody }
}
fn graph<K>(
    pair: &Pair<K>,
    registry: &bank::CustodyRegistry,
    inspect: impl FnOnce(&K, &mut AllocationGraph),
) -> AllocationGraph {
    let mut graph = AllocationGraph::default();
    pair.value.inspect_raw_allocations(&mut graph);
    let slot = &pair.value[0];
    assert_eq!(slot.generation, 0);
    assert!(!slot.retired);
    let common = slot.node.as_ref().unwrap();
    assert_eq!(common.document.0.slot, 0);
    assert!(common.global.is_none());
    assert!(common.parent.is_none());
    assert!(common.own.bytes() >= size_of::<Slot<K>>() + size_of::<Common<K>>());
    graph.allocation(
        &**common as *const Common<K> as usize,
        "common.box",
        size_of::<Common<K>>(),
    );
    inspect(&common.kind, &mut graph);
    registry.inspect_raw_allocations(&mut graph);
    let _borrowed_control = &pair.custody;
    graph
}
fn leaf<T>(value: T, fields: Usage, registry: &bank::CustodyRegistry) -> Sealed<T> {
    let own = Usage::new(size_of::<Payload<T>>(), 0)
        .checked_add(fields)
        .unwrap();
    Pending::new(
        custody::Held::new(
            value,
            registry.ledger().retain(Usage::ZERO, unlimited).unwrap(),
        ),
        own,
        unlimited,
    )
    .unwrap()
    .seal(registry, unlimited)
    .unwrap()
}
fn report(label: &str, graph: &AllocationGraph, registry: &bank::CustodyRegistry) {
    eprintln!(
        "compact_encoding {label}: raw={}B/{}alloc conservative={:?}; opaque Arc metadata excluded",
        graph.bytes(),
        graph.allocations(),
        registry.total()
    );
}

enum Kind<P, D> {
    Text(DocumentString),
    Comment(DocumentString),
    Pi(P),
    Doctype(D),
}
type Inline = Kind<DomProcessingInstruction, DomDocumentType>;
type Typed = Kind<Arc<Payload<DomProcessingInstruction>>, Arc<Payload<DomDocumentType>>>;
fn inline_graph(pair: &Pair<Inline>, registry: &bank::CustodyRegistry) -> AllocationGraph {
    graph(pair, registry, |kind, graph| match kind {
        Kind::Text(value) | Kind::Comment(value) => graph.string(value),
        Kind::Pi(value) => value.inspect_compact_baseline(graph),
        Kind::Doctype(value) => {
            for text in [value.name(), value.public_id(), value.system_id()] {
                graph.string(text);
            }
        }
    })
}
fn typed_graph(pair: &Pair<Typed>, registry: &bank::CustodyRegistry) -> AllocationGraph {
    graph(pair, registry, |kind, graph| match kind {
        Kind::Text(value) | Kind::Comment(value) => graph.string(value),
        Kind::Pi(value) => {
            graph.target(value, "leaf.payload");
            value.value.inspect_compact_baseline(graph);
        }
        Kind::Doctype(value) => {
            graph.target(value, "leaf.payload");
            for text in [
                value.value.name(),
                value.value.public_id(),
                value.value.system_id(),
            ] {
                graph.string(text);
            }
        }
    })
}
#[test]
fn pi_doctype_choices_measure_plain_and_actual_leaf_body_control_and_overlap() {
    for case in ["Text", "Comment", "PI", "Doctype"] {
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
        let inline = match case {
            "Text" | "Comment" => {
                let text = DocumentString::from("data");
                let mut fields = source::storage::Payload::default();
                fields.string(&text).unwrap();
                Input {
                    kind: if case == "Text" {
                        Kind::Text(text)
                    } else {
                        Kind::Comment(text)
                    },
                    fields: Usage::new(fields.bytes, fields.units),
                    child: None,
                }
            }
            "PI" => {
                let value =
                    DomProcessingInstruction::from_parsed("target".into(), "data".into(), vec![]);
                let fields = value.retained_payload(|| Ok(())).unwrap();
                Input {
                    kind: Kind::Pi(value),
                    fields: Usage::new(fields.bytes, fields.units),
                    child: None,
                }
            }
            _ => {
                let value =
                    DomDocumentType::from_parsed("html".into(), "public".into(), "system".into());
                let mut fields = source::storage::Payload::default();
                for text in [value.name(), value.public_id(), value.system_id()] {
                    fields.string(text).unwrap();
                }
                Input {
                    kind: Kind::Doctype(value),
                    fields: Usage::new(fields.bytes, fields.units),
                    child: None,
                }
            }
        };
        let inline = register(inline, &registry);
        report(
            &format!("{case} inline only"),
            &inline_graph(&inline, &registry),
            &registry,
        );
        let typed = match &inline.value[0].node.as_ref().unwrap().kind {
            Kind::Text(value) | Kind::Comment(value) => {
                let mut fields = source::storage::Payload::default();
                fields.string(value).unwrap();
                Input {
                    kind: if case == "Text" {
                        Kind::Text(value.clone())
                    } else {
                        Kind::Comment(value.clone())
                    },
                    fields: Usage::new(fields.bytes, fields.units),
                    child: None,
                }
            }
            Kind::Pi(value) => {
                let fields = value.retained_payload(|| Ok(())).unwrap();
                let retained = leaf(
                    value.clone(),
                    Usage::new(fields.bytes, fields.units),
                    &registry,
                );
                Input {
                    kind: Kind::Pi(retained.value),
                    fields: Usage::ZERO,
                    child: Some(retained.custody),
                }
            }
            Kind::Doctype(value) => {
                let mut fields = source::storage::Payload::default();
                for text in [value.name(), value.public_id(), value.system_id()] {
                    fields.string(text).unwrap();
                }
                let retained = leaf(
                    value.clone(),
                    Usage::new(fields.bytes, fields.units),
                    &registry,
                );
                Input {
                    kind: Kind::Doctype(retained.value),
                    fields: Usage::ZERO,
                    child: Some(retained.custody),
                }
            }
        };
        let typed = register(typed, &registry);
        let mut overlap = inline_graph(&inline, &registry);
        overlap.merge(&typed_graph(&typed, &registry));
        report(&format!("{case} alternatives overlap"), &overlap, &registry);
        drop(inline);
        registry.reconcile_releases();
        report(
            &format!("{case} typed only"),
            &typed_graph(&typed, &registry),
            &registry,
        );
        drop(typed);
        registry.reconcile_releases();
        assert_eq!(
            registry.total(),
            bank::CustodyRegistry::administration_usage()
        );
    }
}

struct CommonInline {
    scope: surgeist_html::HtmlCommonScope,
    nonce: Option<DocumentString>,
    focus: Option<Arc<Payload<NodeId>>>,
    flags: u8,
}
struct CommonData {
    scope: surgeist_html::HtmlCommonScope,
    nonce: Option<DocumentString>,
    focus: Option<Arc<Payload<NodeId>>>,
}
struct CommonTyped {
    data: Option<Arc<Payload<CommonData>>>,
    flags: u8,
}
#[test]
fn common_binding_choices_measure_actual_nonce_and_null_focus_graphs() {
    for scope in [
        surgeist_html::HtmlCommonScope::Element,
        surgeist_html::HtmlCommonScope::Html,
    ] {
        for applicable_nonce in [false, true] {
            if scope == surgeist_html::HtmlCommonScope::Html && !applicable_nonce {
                continue;
            }
            let ledger = custody::Ledger::default();
            let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
            let nonce = applicable_nonce.then(|| DocumentString::from(""));
            let mut fields = source::storage::Payload::default();
            if let Some(nonce) = &nonce {
                fields.string(nonce).unwrap();
            }
            let fields = Usage::new(fields.bytes, fields.units);
            let inline = register(
                Input {
                    kind: CommonInline {
                        scope,
                        nonce,
                        focus: None,
                        flags: 0,
                    },
                    fields,
                    child: None,
                },
                &registry,
            );
            report(
                "common inline held",
                &graph(&inline, &registry, |value, graph| {
                    assert_eq!(value.scope, scope);
                    assert!(value.focus.is_none());
                    assert_eq!(value.flags, 0);
                    if let Some(nonce) = &value.nonce {
                        graph.string(nonce);
                    }
                }),
                &registry,
            );
            let data = if scope == surgeist_html::HtmlCommonScope::Element && !applicable_nonce {
                None
            } else {
                Some(leaf(
                    CommonData {
                        scope,
                        nonce: inline.value[0].node.as_ref().unwrap().kind.nonce.clone(),
                        focus: None,
                    },
                    fields,
                    &registry,
                ))
            };
            let (data, child) = match data {
                Some(data) => (Some(data.value), Some(data.custody)),
                None => (None, None),
            };
            let typed = register(
                Input {
                    kind: CommonTyped { data, flags: 0 },
                    fields: Usage::ZERO,
                    child,
                },
                &registry,
            );
            let inspect_typed = |value: &CommonTyped, graph: &mut AllocationGraph| {
                assert_eq!(value.flags, 0);
                if let Some(data) = &value.data {
                    graph.target(data, "common.payload");
                    assert_eq!(data.value.scope, scope);
                    assert!(data.value.focus.is_none());
                    if let Some(nonce) = &data.value.nonce {
                        graph.string(nonce);
                    }
                } else {
                    assert_eq!(scope, surgeist_html::HtmlCommonScope::Element);
                    assert!(!applicable_nonce);
                }
            };
            let mut overlap = graph(&inline, &registry, |value, graph| {
                if let Some(nonce) = &value.nonce {
                    graph.string(nonce);
                }
            });
            overlap.merge(&graph(&typed, &registry, inspect_typed));
            report("common alternatives overlap", &overlap, &registry);
            drop(inline);
            registry.reconcile_releases();
            report(
                "common typed only",
                &graph(&typed, &registry, inspect_typed),
                &registry,
            );
            drop(typed);
            registry.reconcile_releases();
            assert_eq!(
                registry.total(),
                bank::CustodyRegistry::administration_usage()
            );
        }
    }
}

enum Presence<T> {
    Unavailable,
    Empty,
    Data(Arc<Payload<T>>),
}
struct DocumentTagged {
    groups: [Presence<Vec<NodeId>>; 5],
}
struct DocumentBits {
    known: u8,
    groups: [Option<Arc<Payload<Vec<NodeId>>>>; 5],
}
#[test]
fn presence_choices_measure_unavailable_empty_and_reserved_storage() {
    for known in [false, true] {
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
        let tagged = register(
            Input {
                kind: DocumentTagged {
                    groups: std::array::from_fn(|_| {
                        if known {
                            Presence::Empty
                        } else {
                            Presence::Unavailable
                        }
                    }),
                },
                fields: Usage::ZERO,
                child: None,
            },
            &registry,
        );
        report(
            "document tagged default",
            &graph(&tagged, &registry, |value, _| {
                assert!(value.groups.iter().all(|value| matches!(
                    (known, value),
                    (true, Presence::Empty) | (false, Presence::Unavailable)
                )));
            }),
            &registry,
        );
        let bits = register(
            Input {
                kind: DocumentBits {
                    known: if known { 0b11111 } else { 0 },
                    groups: std::array::from_fn(|_| None),
                },
                fields: Usage::ZERO,
                child: None,
            },
            &registry,
        );
        let mut overlap = graph(&tagged, &registry, |_, _| {});
        overlap.merge(&graph(&bits, &registry, |_, _| {}));
        report(
            "document presence alternatives overlap",
            &overlap,
            &registry,
        );
        drop(tagged);
        registry.reconcile_releases();
        report(
            "document explicit known bits default",
            &graph(&bits, &registry, |value, _| {
                assert_eq!(value.known, if known { 0b11111 } else { 0 });
                assert!(value.groups.iter().all(Option::is_none));
            }),
            &registry,
        );
        drop(bits);
        registry.reconcile_releases();
        let raw = Vec::<NodeId>::with_capacity(7);
        let capacity = raw.capacity();
        let retained = leaf(
            raw,
            Usage::new(capacity * size_of::<NodeId>(), 0),
            &registry,
        );
        let tagged = register(
            Input {
                kind: Presence::Data(retained.value),
                fields: Usage::ZERO,
                child: Some(retained.custody),
            },
            &registry,
        );
        report(
            "reserved-empty actual data",
            &graph(&tagged, &registry, |value, graph| {
                let Presence::Data(value) = value else {
                    panic!("reserved storage is allocated state")
                };
                assert!(value.value.is_empty());
                assert_eq!(value.value.capacity(), capacity);
                graph.target(value, "presence.payload");
                graph.buffer(&value.value, "presence.buffer");
            }),
            &registry,
        );
        drop(tagged);
        registry.reconcile_releases();
        assert_eq!(
            registry.total(),
            bank::CustodyRegistry::administration_usage()
        );
    }
}
