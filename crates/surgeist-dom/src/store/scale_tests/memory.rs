//! Small native memory-fixture pilot. These observations are not an allocator
//! census: physical measurements and parser/layer controls are separate work.
//! Every population and edit uses the ordinary public DOM entry points.

use super::{
    Context, Duration, Fixture, Instant, SAMPLES, WARMUPS, apply, attribute, black_box, fixture,
    fixture_with_capacity, report, require_release, scale_metrics,
};
use crate::{DocumentString, DomBatch, DomChange, DomNodeKind, DomSnapshot, NodeId};

const MANAGED: usize = 1000;
const SEED_NODES: usize = 5;

#[derive(Clone, Copy, Debug)]
enum Population {
    DetachedElements,
    DetachedText,
    AttachedElements,
    AttachedText,
    VacantComments,
}

impl Population {
    fn attached(self) -> bool {
        matches!(self, Self::AttachedElements | Self::AttachedText)
    }

    fn text(self) -> bool {
        matches!(self, Self::DetachedText | Self::AttachedText)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Counts {
    documents: usize,
    elements: usize,
    text: usize,
    comments: usize,
    other: usize,
    parent_edges: usize,
    data_units: usize,
}

/// No per-node output allocation. This deliberately walks the public snapshot;
/// its elapsed time is reported separately from setup and isolated mutation.
fn traverse(snapshot: &DomSnapshot) -> Counts {
    let mut counts = Counts::default();
    for id in snapshot.nodes() {
        let node = snapshot.node(id).unwrap();
        match node.kind() {
            DomNodeKind::Document(_) => counts.documents += 1,
            DomNodeKind::Element(_) => counts.elements += 1,
            DomNodeKind::Text(_) => counts.text += 1,
            DomNodeKind::Comment(_) => counts.comments += 1,
            _ => counts.other += 1,
        }
        counts.parent_edges += usize::from(node.parent().is_some());
        counts.data_units += node.data().map_or(0, |value| value.len());
    }
    counts
}

fn populate(population: Population) -> Fixture {
    if matches!(population, Population::VacantComments) {
        // Existing fixture publishes all historical identities before retiring
        // them through public destroy; the vacancies are not private seeding.
        return fixture(MANAGED, Context::Vacant);
    }
    // Reuse the genuine two-Document/Element/Text seed and finite admission.
    // Its extra capacity is configuration, not an observed-storage oracle.
    let started = Instant::now();
    let mut result = fixture_with_capacity(SEED_NODES, Context::Detached, MANAGED - SEED_NODES);
    let parent = result
        .dom
        .snapshot()
        .node(result.other_document.node())
        .unwrap()
        .children()[0];
    let mut batch = DomBatch::new();
    for _ in SEED_NODES..MANAGED {
        let node = if population.text() {
            batch.create_text(result.other_document, "plain")
        } else {
            batch.create_element(result.other_document, "plain")
        };
        if population.attached() {
            batch.insert_before(&node, parent, None);
        }
    }
    drop(apply(&mut result.dom, batch));
    result.managed = MANAGED;
    result.peak_managed = MANAGED;
    assert_eq!(result.dom.usage().managed_nodes(), MANAGED);
    result.setup = started.elapsed();
    result
}

fn check_population(snapshot: &DomSnapshot, fixture: &Fixture, population: Population) -> Counts {
    let counts = traverse(snapshot);
    let additional = MANAGED - SEED_NODES;
    assert_eq!(counts.documents, 2);
    assert_eq!(counts.other, 0);
    assert_eq!(
        counts.elements,
        2 + usize::from(!population.text() && !matches!(population, Population::VacantComments))
            * additional
    );
    assert_eq!(counts.text, 1 + usize::from(population.text()) * additional);
    assert_eq!(
        counts.comments,
        usize::from(matches!(population, Population::VacantComments)) * additional
    );
    assert_eq!(
        counts.parent_edges,
        3 + usize::from(population.attached()) * additional
    );
    let population_units = match population {
        Population::DetachedText | Population::AttachedText => 5 * additional,
        Population::VacantComments => additional,
        Population::DetachedElements | Population::AttachedElements => 0,
    };
    assert_eq!(counts.data_units, 4 + population_units);
    assert_eq!(snapshot.nodes().count(), MANAGED);
    assert_eq!(
        snapshot.node(fixture.target).unwrap().children(),
        [fixture.text]
    );
    let parent = snapshot
        .node(fixture.other_document.node())
        .unwrap()
        .children()[0];
    let expected_children = if population.attached() { additional } else { 0 };
    assert_eq!(
        snapshot.node(parent).unwrap().children().len(),
        expected_children
    );
    for id in snapshot.nodes() {
        // These five real seed IDs are independently identifiable via the
        // public two-Document topology and the selected target/Text IDs.
        if id == fixture.target
            || id == fixture.text
            || id == fixture.other_document.node()
            || id == parent
            || snapshot.node(id).unwrap().as_document().is_some()
        {
            continue;
        }
        let node = snapshot.node(id).unwrap();
        assert_eq!(node.owner_document(), Some(fixture.other_document));
        assert_eq!(node.parent(), population.attached().then_some(parent));
        match node.kind() {
            DomNodeKind::Element(value) => {
                assert_eq!(value.name().local_name(), &DocumentString::from("plain"))
            }
            DomNodeKind::Text(value) => assert_eq!(value, &DocumentString::from("plain")),
            DomNodeKind::Comment(value) => assert_eq!(value, &DocumentString::from("u")),
            _ => panic!("only the declared public population is present"),
        }
    }
    counts
}

fn check_selected(
    snapshot: &DomSnapshot,
    target: NodeId,
    text: NodeId,
    changed_text: bool,
    changed: bool,
) {
    assert_eq!(
        attribute(snapshot, target),
        DocumentString::from(if changed && !changed_text {
            "next"
        } else {
            "base"
        })
    );
    assert_eq!(
        snapshot.node(text).unwrap().data(),
        Some(&DocumentString::from(if changed && changed_text {
            "next"
        } else {
            "base"
        }))
    );
    assert_eq!(snapshot.node(target).unwrap().children(), [text]);
}

#[derive(Default)]
struct Timings {
    setup: Vec<Duration>,
    traversal: Vec<Duration>,
    snapshot: Vec<Duration>,
    edit: Vec<Duration>,
    owner_drop: Vec<Duration>,
    current_drop: Vec<Duration>,
    last_snapshot_drop: Vec<Duration>,
}

fn run_population(population: Population) {
    let mut times = Timings::default();
    for sample in 0..WARMUPS + SAMPLES {
        let started = Instant::now();
        let mut fixture = populate(population);
        let setup = started.elapsed();
        let started = Instant::now();
        let old = black_box(fixture.dom.snapshot());
        let snapshot = started.elapsed();
        let started = Instant::now();
        let counts = black_box(traverse(&old));
        let traversal = started.elapsed();
        assert_eq!(counts, check_population(&old, &fixture, population));
        check_selected(&old, fixture.target, fixture.text, population.text(), false);
        let baseline = fixture.dom.custody_registry.total();

        let mut batch = DomBatch::new();
        let expected = if population.text() {
            batch.replace_data(fixture.text, 0, usize::MAX, "next");
            DomChange::Data(fixture.text)
        } else {
            batch.set_attribute(fixture.target, "data-local", "next");
            DomChange::Attributes(fixture.target)
        };
        let started = Instant::now();
        let commit = apply(&mut fixture.dom, batch);
        let edit = started.elapsed();
        assert_eq!(commit.changes().changes(), [expected]);
        // Release the commit's current snapshot before explicitly holding the
        // two snapshots whose destruction order this pilot characterizes.
        drop(commit);
        let current = fixture.dom.snapshot();
        assert_eq!(current.revision().get(), old.revision().get() + 1);
        assert!(current.nodes().eq(old.nodes()));
        for id in old.nodes() {
            let previous = old.node(id).unwrap();
            let next = current.node(id).unwrap();
            assert_eq!(next.parent(), previous.parent());
            assert_eq!(next.owner_document(), previous.owner_document());
            assert_eq!(next.children(), previous.children());
        }
        assert_eq!(check_population(&current, &fixture, population), counts);
        check_selected(
            &current,
            fixture.target,
            fixture.text,
            population.text(),
            true,
        );
        check_selected(&old, fixture.target, fixture.text, population.text(), false);
        let after = fixture.dom.custody_registry.total();
        eprintln!(
            "memory_pilot population={population:?} sample={sample} warmup={} managed={} rows=0 cells=0 counts={counts:?} retired={} peak_managed={} phase=published_with_old_snapshot controlled_before={baseline:?} controlled_after={after:?} controlled_quantity_is_not_physical=true setup_ns={} traversal_ns={} snapshot_acquire_ns={} edit_ns={} limits={:?}",
            sample < WARMUPS,
            fixture.managed,
            fixture.retired,
            fixture.peak_managed,
            setup.as_nanos(),
            traversal.as_nanos(),
            snapshot.as_nanos(),
            edit.as_nanos(),
            fixture.dom.limits()
        );
        let target = fixture.target;
        let text = fixture.text;
        let started = Instant::now();
        drop(fixture);
        let owner_drop = started.elapsed();
        check_selected(&current, target, text, population.text(), true);
        check_selected(&old, target, text, population.text(), false);
        assert_eq!(current.nodes().count(), MANAGED);
        assert_eq!(old.nodes().count(), MANAGED);
        eprintln!(
            "memory_pilot population={population:?} sample={sample} phase=owner_dropped current_and_old_snapshots_live=true owner_drop_ns={}",
            owner_drop.as_nanos()
        );
        let started = Instant::now();
        drop(current);
        let current_drop = started.elapsed();
        check_selected(&old, target, text, population.text(), false);
        assert_eq!(old.nodes().count(), MANAGED);
        let started = Instant::now();
        drop(old);
        let last_snapshot_drop = started.elapsed();
        eprintln!(
            "memory_pilot population={population:?} sample={sample} phase=all_semantic_holders_dropped current_snapshot_drop_ns={} last_snapshot_drop_ns={} allocator_reclamation_not_observed=true",
            current_drop.as_nanos(),
            last_snapshot_drop.as_nanos()
        );
        if sample >= WARMUPS {
            times.setup.push(setup);
            times.traversal.push(traversal);
            times.snapshot.push(snapshot);
            times.edit.push(edit);
            times.owner_drop.push(owner_drop);
            times.current_drop.push(current_drop);
            times.last_snapshot_drop.push(last_snapshot_drop);
        }
    }
    eprintln!("memory_pilot summary population={population:?} managed={MANAGED} rows=0 cells=0");
    for (label, values) in [
        ("memory_setup", &times.setup),
        ("memory_traversal", &times.traversal),
        ("memory_snapshot_acquire", &times.snapshot),
        ("memory_isolated_edit", &times.edit),
        ("memory_owner_drop_two_snapshots_held", &times.owner_drop),
        ("memory_current_snapshot_drop_old_held", &times.current_drop),
        ("memory_last_snapshot_drop", &times.last_snapshot_drop),
    ] {
        report(label, values);
    }
}

#[test]
#[ignore = "explicit release-only 1000-node memory fixture pilot; no physical-census claim"]
fn native_small_populations_preserve_snapshots_through_edit_and_owner_release() {
    require_release();
    if let Some(selected) = std::env::var_os("SURGEIST_SCALE_SIZES") {
        assert_eq!(
            selected, "1000",
            "this initial memory pilot accepts only 1000 nodes"
        );
    }
    eprintln!(
        "memory_pilot profile=release managed_selection={MANAGED} seed_managed={SEED_NODES} warmups={WARMUPS} samples={SAMPLES} counters_enabled={} physical_backend=none revision={} toolchain={} machine={} os={} arch={} setup_ingress=ordinary_native setup_includes_fixture_checks=true traversal_scope=node_kind_parent_data_length",
        scale_metrics::ENABLED,
        std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into()),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    for population in [
        Population::DetachedElements,
        Population::DetachedText,
        Population::AttachedElements,
        Population::AttachedText,
        Population::VacantComments,
    ] {
        run_population(population);
    }
}
