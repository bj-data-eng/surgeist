//! Small native XML layered-sharing fixtures. Backing observations describe only
//! the required immutable reuse, never allocation totals or physical memory.
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::{
    DocumentId, Dom, DomBatch, DomChange, DomCommit, DomLimits, DomNameLimits, DomNodeKind,
    DomSnapshot, NodeId,
};
use surgeist_html::values::DocumentString;

mod selected;

const ROWS: usize = 2;
const COLUMNS: usize = 3;
const CELLS: usize = ROWS * COLUMNS;
const RAW: &[u16] = &[65, 0xd800, 0, 0xdc00];
const NEXT: &[u16] = &[110, 101, 120, 116];
const UNIQUE_ATTRIBUTE_NEXT: &[u16] = &[65, 110, 101, 119];
const UNIQUE_TEXT_NEXT: &[u16] = &[84, 110, 101, 119];
const ROW_IDS: [&str; ROWS] = ["r0", "r1"];
const CELL_IDS: [&str; CELLS] = ["c0", "c1", "c2", "c3", "c4", "c5"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Geometry {
    Tiny,
    Representative,
}
impl Geometry {
    fn rows(self) -> usize {
        match self {
            Self::Tiny => ROWS,
            Self::Representative => 8,
        }
    }
    fn columns(self) -> usize {
        match self {
            Self::Tiny => COLUMNS,
            Self::Representative => 32,
        }
    }
    fn cells(self) -> usize {
        self.rows() * self.columns()
    }
    fn nodes(self) -> usize {
        2 + self.rows() + 2 * self.cells()
    }
    fn managed_nodes(self, unique: Unique) -> usize {
        self.nodes() + self.attributes(unique)
    }
    fn attributes(self, unique: Unique) -> usize {
        2 * self.cells()
            + match unique {
                Unique::None => 0,
                Unique::Rows => self.rows(),
                Unique::Cells => self.cells(),
            }
    }
    fn command_limit(self) -> usize {
        match self {
            Self::Tiny => 256,
            // Document/table/insert: 3; row creation/insertion: 2 per row;
            // six cell/Text commands plus at most one cell ID: 7 per cell.
            // Cell IDs give the largest of the three authored patterns.
            Self::Representative => 3 + 2 * self.rows() + 7 * self.cells(),
        }
    }
    fn row_id(self, row: usize) -> DocumentString {
        match self {
            Self::Tiny => ROW_IDS[row].into(),
            Self::Representative => format!("r{row}").into(),
        }
    }
    fn cell_id(self, index: usize) -> DocumentString {
        match self {
            Self::Tiny => CELL_IDS[index].into(),
            Self::Representative => format!("c{index}").into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Independent,
    CallerShared,
}
impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::CallerShared => "caller-shared",
        }
    }
    fn caller_shared(self) -> bool {
        self == Self::CallerShared
    }
}
const MODES: [Mode; 2] = [Mode::Independent, Mode::CallerShared];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unique {
    None,
    Rows,
    Cells,
}
impl Unique {
    fn name(self) -> &'static str {
        match self {
            Self::None => "repeated",
            Self::Rows => "row-ids",
            Self::Cells => "cell-ids",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValuePattern {
    RepeatedShort,
    UniqueShort,
}
impl ValuePattern {
    fn name(self) -> &'static str {
        match self {
            Self::RepeatedShort => "repeated-short",
            Self::UniqueShort => "unique-short",
        }
    }
    fn validate(self, mode: Mode, unique: Unique) -> Result<(), &'static str> {
        if self == Self::UniqueShort {
            if mode == Mode::CallerShared {
                return Err("unique-short values cannot use one repeated caller-shared backing");
            }
            if unique != Unique::None {
                return Err("unique-short values require the repeated ID pattern");
            }
        }
        Ok(())
    }
    fn input(self, cell: usize, slot: usize) -> DocumentString {
        match self {
            Self::RepeatedShort => raw(),
            // The closed geometry has at most 256 cells: every data-a,
            // data-b and Text value has its own four-unit v000..v767 spelling.
            Self::UniqueShort => format!("v{:03}", 3 * cell + slot).into(),
        }
    }
}

struct Grid {
    geometry: Geometry,
    values: ValuePattern,
    document: DocumentId,
    table: NodeId,
    rows: Vec<NodeId>,
    cells: Vec<NodeId>,
    texts: Vec<NodeId>,
}
struct Constructed {
    grid: Grid,
    // Keep independently authored inputs observable, so an allocator cannot
    // recycle their addresses and imitate a sharing observation.
    inputs: Vec<[DocumentString; 3]>,
    creation: DomCommit,
}
fn owner(geometry: Geometry) -> Dom {
    Dom::new(
        DomLimits::new(
            (geometry.nodes() + geometry.attributes(Unique::Cells)).max(64),
            1_000_000,
            geometry.command_limit(),
            0,
            0,
            16,
            4_000_000,
            DomNameLimits::new(16_777_216, 4096, 4_000_000, 4_000_000).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn raw() -> DocumentString {
    DocumentString::from_utf16(RAW.to_vec())
}
fn construct(
    dom: &mut Dom,
    geometry: Geometry,
    mode: Mode,
    unique: Unique,
    values: ValuePattern,
) -> Constructed {
    let caller = raw();
    let inputs: Vec<_> = (0..geometry.cells())
        .map(|cell| {
            std::array::from_fn(|slot| {
                if mode.caller_shared() {
                    caller.clone()
                } else {
                    values.input(cell, slot)
                }
            })
        })
        .collect();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let table = batch.create_element(&document, "table");
    batch.insert_before(&table, &document, None);
    let mut rows = Vec::new();
    let mut cells = Vec::new();
    let mut texts = Vec::new();
    for row in 0..geometry.rows() {
        let parent = batch.create_element(&document, "row");
        if matches!(unique, Unique::Rows) {
            batch.set_attribute(&parent, "id", geometry.row_id(row));
        }
        batch.insert_before(&parent, &table, None);
        for column in 0..geometry.columns() {
            let index = row * geometry.columns() + column;
            let cell = batch.create_element(&document, "cell");
            batch.set_attribute(&cell, "data-a", inputs[index][0].clone());
            batch.set_attribute(&cell, "data-b", inputs[index][1].clone());
            if matches!(unique, Unique::Cells) {
                batch.set_attribute(&cell, "id", geometry.cell_id(index));
            }
            let text = batch.create_text(&document, inputs[index][2].clone());
            batch.insert_before(&text, &cell, None);
            batch.insert_before(&cell, &parent, None);
            cells.push(cell);
            texts.push(text);
        }
        rows.push(parent);
    }
    let creation = apply(dom, batch);
    let grid = Grid {
        geometry,
        values,
        document: creation.document(&document).unwrap(),
        table: creation.created(&table).unwrap().unwrap(),
        rows: rows
            .iter()
            .map(|id| creation.created(id).unwrap().unwrap())
            .collect(),
        cells: cells
            .iter()
            .map(|id| creation.created(id).unwrap().unwrap())
            .collect(),
        texts: texts
            .iter()
            .map(|id| creation.created(id).unwrap().unwrap())
            .collect(),
    };
    Constructed {
        grid,
        inputs,
        creation,
    }
}
fn attributes(snapshot: &DomSnapshot, id: NodeId) -> crate::DomNamedNodeMap<'_> {
    snapshot.node(id).unwrap().attributes().unwrap()
}
fn text(snapshot: &DomSnapshot, id: NodeId) -> &DocumentString {
    snapshot.node(id).unwrap().data().unwrap()
}

/// Check the declared ID grammar independently of construction's formatter.
/// No expected-string collection survives the check or a holder checkpoint.
fn representative_id_matches(value: &DocumentString, prefix: u16, ordinal: usize) -> bool {
    let Some((&first, digits)) = value.code_units().split_first() else {
        return false;
    };
    if first != prefix || digits.is_empty() || (digits.len() > 1 && digits[0] == u16::from(b'0')) {
        return false;
    }
    let actual = digits.iter().try_fold(0usize, |number, &digit| {
        if !(u16::from(b'0')..=u16::from(b'9')).contains(&digit) {
            return None;
        }
        number
            .checked_mul(10)?
            .checked_add(usize::from(digit - u16::from(b'0')))
    });
    actual == Some(ordinal)
}

/// Decode the actual four-unit spelling; never call construction's formatter
/// to manufacture an expected value or retain an expected-value inventory.
fn unique_value_matches(value: &DocumentString, ordinal: usize) -> bool {
    let [118, a, b, c] = value.code_units() else {
        return false;
    };
    let actual = [*a, *b, *c].into_iter().try_fold(0usize, |number, digit| {
        if !(48..=57).contains(&digit) {
            return None;
        }
        Some(number * 10 + usize::from(digit - 48))
    });
    actual == Some(ordinal)
}

fn check_grid(
    snapshot: &DomSnapshot,
    grid: &Grid,
    unique: Unique,
    attr_changed: bool,
    text_changed: bool,
) {
    let geometry = grid.geometry;
    assert_eq!(grid.rows.len(), geometry.rows());
    assert_eq!(grid.cells.len(), geometry.cells());
    assert_eq!(grid.texts.len(), geometry.cells());
    assert_eq!(
        snapshot.nodes().count(),
        geometry.nodes() + geometry.attributes(unique)
    );
    let (mut documents, mut elements, mut texts, mut attrs) = (0, 0, 0, 0);
    for id in snapshot.nodes() {
        match snapshot.node(id).unwrap().kind() {
            DomNodeKind::Document(_) => documents += 1,
            DomNodeKind::Element(_) => elements += 1,
            DomNodeKind::Text(_) => texts += 1,
            DomNodeKind::Attr(_) => attrs += 1,
            _ => panic!("unexpected node kind in the authored XML grid"),
        }
    }
    assert_eq!(attrs, geometry.attributes(unique));
    assert_eq!(
        (documents, elements, texts),
        (1, 1 + geometry.rows() + geometry.cells(), geometry.cells())
    );
    assert_eq!(
        snapshot.node(grid.document.node()).unwrap().children(),
        [grid.table]
    );
    assert_eq!(snapshot.node(grid.table).unwrap().children(), grid.rows);
    let mut attribute_count = attributes(snapshot, grid.table).len();
    assert_eq!(attribute_count, 0);
    assert_eq!(
        snapshot.node(grid.table).unwrap().parent(),
        Some(grid.document.node())
    );
    assert_eq!(
        snapshot.node(grid.table).unwrap().owner_document(),
        Some(grid.document)
    );
    assert_eq!(
        snapshot
            .node(grid.table)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name()
            .code_units(),
        [116, 97, 98, 108, 101]
    );
    for (row, id) in grid.rows.iter().copied().enumerate() {
        let node = snapshot.node(id).unwrap();
        assert_eq!(
            node.as_element().unwrap().name().local_name().code_units(),
            [114, 111, 119]
        );
        assert_eq!(node.parent(), Some(grid.table));
        assert_eq!(node.owner_document(), Some(grid.document));
        assert_eq!(
            node.children(),
            &grid.cells[row * geometry.columns()..(row + 1) * geometry.columns()]
        );
        let values = attributes(snapshot, id);
        attribute_count += values.len();
        if matches!(unique, Unique::Rows) {
            assert_eq!(values.len(), 1);
            assert_eq!(
                values.item(0).unwrap().name().local_name().code_units(),
                [105, 100]
            );
            match geometry {
                Geometry::Tiny => assert!(
                    values
                        .item(0)
                        .unwrap()
                        .value()
                        .code_units()
                        .iter()
                        .copied()
                        .eq(ROW_IDS[row].encode_utf16())
                ),
                Geometry::Representative => assert!(representative_id_matches(
                    values.item(0).unwrap().value(),
                    u16::from(b'r'),
                    row
                )),
            }
        } else {
            assert!(values.is_empty());
        }
    }
    for (index, id) in grid.cells.iter().copied().enumerate() {
        let cell = snapshot.node(id).unwrap();
        assert_eq!(cell.parent(), Some(grid.rows[index / geometry.columns()]));
        assert_eq!(cell.owner_document(), Some(grid.document));
        assert_eq!(cell.children(), [grid.texts[index]]);
        assert_eq!(
            cell.as_element().unwrap().name().local_name().code_units(),
            [99, 101, 108, 108]
        );
        let values = attributes(snapshot, id);
        attribute_count += values.len();
        assert_eq!(
            values.len(),
            if matches!(unique, Unique::Cells) {
                3
            } else {
                2
            }
        );
        for (offset, suffix) in [97, 98].into_iter().enumerate() {
            assert!(
                values
                    .item(offset)
                    .unwrap()
                    .name()
                    .namespace_uri()
                    .is_none()
            );
            assert!(values.item(offset).unwrap().name().prefix().is_none());
            assert_eq!(
                values
                    .item(offset)
                    .unwrap()
                    .name()
                    .local_name()
                    .code_units(),
                [100, 97, 116, 97, 45, suffix]
            );
            let changed = attr_changed && index == 0 && offset == 0;
            match grid.values {
                ValuePattern::RepeatedShort => assert_eq!(
                    values.item(offset).unwrap().value().code_units(),
                    if changed { NEXT } else { RAW }
                ),
                ValuePattern::UniqueShort if changed => {
                    assert_eq!(
                        values.item(offset).unwrap().value().code_units(),
                        UNIQUE_ATTRIBUTE_NEXT
                    );
                }
                ValuePattern::UniqueShort => {
                    assert!(unique_value_matches(
                        values.item(offset).unwrap().value(),
                        index * 3 + offset
                    ));
                }
            }
        }
        if matches!(unique, Unique::Cells) {
            assert_eq!(
                values.item(2).unwrap().name().local_name().code_units(),
                [105, 100]
            );
            match geometry {
                Geometry::Tiny => assert!(
                    values
                        .item(2)
                        .unwrap()
                        .value()
                        .code_units()
                        .iter()
                        .copied()
                        .eq(CELL_IDS[index].encode_utf16())
                ),
                Geometry::Representative => assert!(representative_id_matches(
                    values.item(2).unwrap().value(),
                    u16::from(b'c'),
                    index
                )),
            }
        }
        let node = snapshot.node(grid.texts[index]).unwrap();
        assert!(matches!(node.kind(), DomNodeKind::Text(_)));
        assert_eq!(node.parent(), Some(id));
        assert_eq!(node.owner_document(), Some(grid.document));
        let value = node.data().unwrap();
        match grid.values {
            ValuePattern::RepeatedShort => assert_eq!(
                value.code_units(),
                if text_changed && index == 0 {
                    NEXT
                } else {
                    RAW
                }
            ),
            ValuePattern::UniqueShort if text_changed && index == 0 => {
                assert_eq!(value.code_units(), UNIQUE_TEXT_NEXT);
            }
            ValuePattern::UniqueShort => assert!(unique_value_matches(value, index * 3 + 2)),
        }
    }
    assert_eq!(attribute_count, geometry.attributes(unique));
    if grid.values == ValuePattern::UniqueShort {
        let first = attributes(snapshot, grid.cells[0]);
        assert_eq!(
            first.item(0).unwrap().value().code_units(),
            if attr_changed {
                UNIQUE_ATTRIBUTE_NEXT
            } else {
                &[118, 48, 48, 48]
            }
        );
        assert_eq!(
            first.item(1).unwrap().value().code_units(),
            [118, 48, 48, 49]
        );
        assert_eq!(
            text(snapshot, grid.texts[0]).code_units(),
            if text_changed {
                UNIQUE_TEXT_NEXT
            } else {
                &[118, 48, 48, 50]
            }
        );
        let last = attributes(snapshot, *grid.cells.last().unwrap());
        let last_text = text(snapshot, *grid.texts.last().unwrap());
        match geometry {
            Geometry::Tiny => {
                assert_eq!(
                    last.item(0).unwrap().value().code_units(),
                    [118, 48, 49, 53]
                );
                assert_eq!(
                    last.item(1).unwrap().value().code_units(),
                    [118, 48, 49, 54]
                );
                assert_eq!(last_text.code_units(), [118, 48, 49, 55]);
            }
            Geometry::Representative => {
                assert_eq!(
                    last.item(0).unwrap().value().code_units(),
                    [118, 55, 54, 53]
                );
                assert_eq!(
                    last.item(1).unwrap().value().code_units(),
                    [118, 55, 54, 54]
                );
                assert_eq!(last_text.code_units(), [118, 55, 54, 55]);
            }
        }
    }
}
fn check_topology(before: &DomSnapshot, after: &DomSnapshot) {
    assert!(after.nodes().eq(before.nodes()));
    for id in before.nodes() {
        let old = before.node(id).unwrap();
        let new = after.node(id).unwrap();
        assert_eq!(new.parent(), old.parent());
        assert_eq!(new.children(), old.children());
        assert_eq!(new.owner_document(), old.owner_document());
    }
}
fn distinct_attributes(snapshot: &DomSnapshot, first: NodeId, second: NodeId) {
    let first = attributes(snapshot, first);
    let second = attributes(snapshot, second);
    assert_eq!(first.len(), second.len());
    assert!(
        first
            .iter()
            .zip(second.iter())
            .all(|(a, b)| a.id() != b.id())
    );
}

fn same_attribute_backing(
    first: crate::DomNamedNodeMap<'_>,
    second: crate::DomNamedNodeMap<'_>,
) -> bool {
    first.len() == second.len()
        && first.iter().zip(second.iter()).all(|(a, b)| {
            a.id() == b.id()
                && std::ptr::eq(a.name(), b.name())
                && std::ptr::eq(a.value(), b.value())
        })
}

fn check_inputs(inputs: &[[DocumentString; 3]], geometry: Geometry, mode: Mode) {
    assert_eq!(inputs.len(), geometry.cells());
    if !mode.caller_shared() {
        assert!(!inputs[0][0].shares_storage_with(&inputs[1][0]));
        assert!(!inputs[0][0].shares_storage_with(&inputs[0][2]));
    }
}
fn check_reuse(snapshot: &DomSnapshot, grid: &Grid, inputs: &[[DocumentString; 3]], mode: Mode) {
    check_inputs(inputs, grid.geometry, mode);
    let shared = grid.values == ValuePattern::RepeatedShort && mode.caller_shared();
    for index in 1..grid.geometry.cells() {
        distinct_attributes(snapshot, grid.cells[0], grid.cells[index]);
        assert_eq!(
            text(snapshot, grid.texts[0]).shares_storage_with(text(snapshot, grid.texts[index])),
            shared,
        );
    }
    let first = attributes(snapshot, grid.cells[0]);
    assert_eq!(
        first
            .item(0)
            .unwrap()
            .value()
            .shares_storage_with(first.item(1).unwrap().value()),
        shared
    );
    assert_eq!(
        first
            .item(0)
            .unwrap()
            .value()
            .shares_storage_with(text(snapshot, grid.texts[0])),
        shared
    );
    for (index, input) in inputs.iter().enumerate() {
        for (offset, value) in input[..2].iter().enumerate() {
            assert!(
                attributes(snapshot, grid.cells[index])
                    .item(offset)
                    .unwrap()
                    .value()
                    .shares_storage_with(value)
            );
        }
        assert!(text(snapshot, grid.texts[index]).shares_storage_with(&input[2]));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HolderStage {
    HarnessOnly,
    CallerInputsHeld,
    Populated,
    BeforeAttributeEdit,
    AttributeEdited,
    BeforeTextEdit,
    TextEdited,
    Edited,
    OwnerDropped,
    OldOnly,
    Released,
}
impl HolderStage {
    const ALL: [Self; 11] = [
        Self::HarnessOnly,
        Self::CallerInputsHeld,
        Self::Populated,
        Self::BeforeAttributeEdit,
        Self::AttributeEdited,
        Self::BeforeTextEdit,
        Self::TextEdited,
        Self::Edited,
        Self::OwnerDropped,
        Self::OldOnly,
        Self::Released,
    ];
    fn name(self) -> &'static str {
        match self {
            Self::HarnessOnly => "harness-only",
            Self::CallerInputsHeld => "caller-inputs-held",
            Self::Populated => "populated",
            Self::BeforeAttributeEdit => "before-attribute-edit",
            Self::AttributeEdited => "attribute-edited",
            Self::BeforeTextEdit => "before-text-edit",
            Self::TextEdited => "text-edited",
            Self::Edited => "edited",
            Self::OwnerDropped => "owner-dropped",
            Self::OldOnly => "old-only",
            Self::Released => "released",
        }
    }
}
fn observe(observer: &mut Option<&mut dyn FnMut(HolderStage)>, at: HolderStage) {
    if let Some(observer) = observer {
        observer(at);
    }
}

/// Operation intervals exclude callbacks, semantic/backing oracles, and commit
/// destruction. Setup includes authored inputs, batch construction, application
/// and qualified identity lookup. Grid/backing checks are separately timed;
/// commit-change assertions and commit destruction are outside both intervals.
struct SampleTimes {
    setup: Duration,
    old_snapshot: Duration,
    caller_inputs_drop: Duration,
    attribute_edit: Duration,
    text_edit: Duration,
    current_snapshot: Duration,
    oracle_checks: Duration,
    owner_drop: Duration,
    current_drop: Duration,
    last_snapshot_drop: Duration,
}
fn elapsed(started: Option<Instant>) -> Duration {
    started.map_or(Duration::ZERO, |started| started.elapsed())
}

/// The same real workload drives ordinary assertions and external checkpoints.
/// The callback receives no holders; all pointer observations borrow live data.
/// No counter-enabled timing is returned, and external inspection can perturb
/// even counter-disabled execution after a checkpoint wait has been excluded.
fn run_sample(
    geometry: Geometry,
    mode: Mode,
    unique: Unique,
    values: ValuePattern,
    mut observer: Option<&mut dyn FnMut(HolderStage)>,
) -> Option<SampleTimes> {
    values
        .validate(mode, unique)
        .expect("valid layered value selection");
    let timing = !super::scale_metrics::ENABLED;
    observe(&mut observer, HolderStage::HarnessOnly);
    let started = timing.then(Instant::now);
    let mut dom = owner(geometry);

    let Constructed {
        grid,
        inputs,
        creation,
    } = construct(&mut dom, geometry, mode, unique, values);
    let setup = elapsed(started);
    let started = timing.then(Instant::now);
    let old = dom.snapshot();
    let old_snapshot = elapsed(started);
    let started = timing.then(Instant::now);
    check_grid(&old, &grid, unique, false, false);
    check_reuse(&old, &grid, &inputs, mode);
    let mut oracle_checks = elapsed(started);
    // These checks complete before the creation receipt/snapshot goes.
    drop(creation);
    observe(&mut observer, HolderStage::CallerInputsHeld);
    let started = timing.then(Instant::now);
    drop(inputs);
    let caller_inputs_drop = elapsed(started);
    observe(&mut observer, HolderStage::Populated);

    observe(&mut observer, HolderStage::BeforeAttributeEdit);
    let started = timing.then(Instant::now);
    let mut batch = DomBatch::new();
    batch.set_attribute(
        grid.cells[0],
        "data-a",
        DocumentString::from_utf16(
            match values {
                ValuePattern::RepeatedShort => NEXT,
                ValuePattern::UniqueShort => UNIQUE_ATTRIBUTE_NEXT,
            }
            .to_vec(),
        ),
    );
    let attr_commit = apply(&mut dom, batch);
    let attribute_edit = elapsed(started);
    assert_eq!(
        attr_commit.changes().changes(),
        [
            DomChange::Attributes(grid.cells[0]),
            DomChange::AttrValue(attributes(&old, grid.cells[0]).item(0).unwrap().id())
        ]
    );
    assert_eq!(dom.revision().get(), old.revision().get() + 1);
    drop(attr_commit);
    observe(&mut observer, HolderStage::AttributeEdited);

    observe(&mut observer, HolderStage::BeforeTextEdit);
    let started = timing.then(Instant::now);
    let mut batch = DomBatch::new();
    batch.replace_data(
        grid.texts[0],
        0,
        usize::MAX,
        DocumentString::from_utf16(
            match values {
                ValuePattern::RepeatedShort => NEXT,
                ValuePattern::UniqueShort => UNIQUE_TEXT_NEXT,
            }
            .to_vec(),
        ),
    );
    let text_commit = apply(&mut dom, batch);
    let text_edit = elapsed(started);
    assert_eq!(
        text_commit.changes().changes(),
        [DomChange::Data(grid.texts[0])]
    );
    assert_eq!(dom.revision().get(), old.revision().get() + 2);
    drop(text_commit);
    observe(&mut observer, HolderStage::TextEdited);

    let started = timing.then(Instant::now);
    let current = dom.snapshot();
    let current_snapshot = elapsed(started);
    let started = timing.then(Instant::now);
    assert_eq!(current.revision().get(), old.revision().get() + 2);
    check_topology(&old, &current);
    check_grid(&current, &grid, unique, true, true);
    check_grid(&old, &grid, unique, false, false);
    for index in 1..geometry.cells() {
        assert!(same_attribute_backing(
            attributes(&current, grid.cells[index]),
            attributes(&old, grid.cells[index])
        ));
        assert!(
            text(&current, grid.texts[index]).shares_storage_with(text(&old, grid.texts[index]))
        );
    }
    assert!(!same_attribute_backing(
        attributes(&current, grid.cells[0]),
        attributes(&old, grid.cells[0])
    ));
    assert!(!text(&current, grid.texts[0]).shares_storage_with(text(&old, grid.texts[0])));
    oracle_checks += elapsed(started);
    observe(&mut observer, HolderStage::Edited);

    let started = timing.then(Instant::now);
    drop(dom);
    let owner_drop = elapsed(started);
    let started = timing.then(Instant::now);
    check_grid(&current, &grid, unique, true, true);
    check_grid(&old, &grid, unique, false, false);
    oracle_checks += elapsed(started);
    observe(&mut observer, HolderStage::OwnerDropped);
    let started = timing.then(Instant::now);
    drop(current);
    let current_drop = elapsed(started);
    let started = timing.then(Instant::now);
    check_grid(&old, &grid, unique, false, false);
    oracle_checks += elapsed(started);
    observe(&mut observer, HolderStage::OldOnly);
    // Grid contains scalar IDs and their Vec buffers, not a snapshot/commit.
    // Release this scratch after its last oracle, before the last snapshot.
    drop(grid);
    let started = timing.then(Instant::now);
    drop(old);
    let last_snapshot_drop = elapsed(started);
    observe(&mut observer, HolderStage::Released);
    timing.then_some(SampleTimes {
        setup,
        old_snapshot,
        caller_inputs_drop,
        attribute_edit,
        text_edit,
        current_snapshot,
        oracle_checks,
        owner_drop,
        current_drop,
        last_snapshot_drop,
    })
}

#[test]
fn ordinary_layers_preserve_attribute_identities_text_and_isolated_edits() {
    for mode in MODES {
        let _ = run_sample(
            Geometry::Tiny,
            mode,
            Unique::None,
            ValuePattern::RepeatedShort,
            None,
        );
    }
}

#[test]
fn unique_row_and_cell_ids_keep_attribute_identities_through_text_edits() {
    for mode in MODES {
        for unique in [Unique::Rows, Unique::Cells] {
            let mut dom = owner(Geometry::Tiny);

            let Constructed {
                grid,
                inputs,
                creation,
            } = construct(
                &mut dom,
                Geometry::Tiny,
                mode,
                unique,
                ValuePattern::RepeatedShort,
            );
            check_inputs(&inputs, Geometry::Tiny, mode);
            let old = dom.snapshot();
            check_grid(&old, &grid, unique, false, false);
            for index in 1..CELLS {
                distinct_attributes(&old, grid.cells[0], grid.cells[index]);
            }
            let mut batch = DomBatch::new();
            for (index, id) in grid.texts.iter().copied().enumerate() {
                batch.replace_data(id, 0, usize::MAX, CELL_IDS[index]);
            }
            let commit = apply(&mut dom, batch);
            assert_eq!(
                commit.changes().changes(),
                grid.texts
                    .iter()
                    .copied()
                    .map(DomChange::Data)
                    .collect::<Vec<_>>()
            );
            let current = dom.snapshot();
            assert_eq!(current.revision().get(), old.revision().get() + 1);
            check_topology(&old, &current);
            for (index, cell_id) in CELL_IDS.iter().enumerate() {
                assert!(
                    text(&current, grid.texts[index])
                        .code_units()
                        .iter()
                        .copied()
                        .eq(cell_id.encode_utf16())
                );
                assert_eq!(text(&old, grid.texts[index]).code_units(), RAW);
                assert_eq!(
                    current.node(grid.texts[index]).unwrap().parent(),
                    Some(grid.cells[index])
                );
                assert_eq!(
                    current.node(grid.cells[index]).unwrap().children(),
                    [grid.texts[index]]
                );
                assert!(same_attribute_backing(
                    attributes(&current, grid.cells[index]),
                    attributes(&old, grid.cells[index])
                ));
            }

            drop(commit);
            drop(dom);
            assert_eq!(
                current.nodes().count(),
                Geometry::Tiny.nodes() + Geometry::Tiny.attributes(unique)
            );
            check_grid(&old, &grid, unique, false, false);
            drop(inputs);
            drop(creation);
            drop(grid);
            drop(current);
            drop(old);
        }
    }
}

#[test]
fn caller_held_long_and_empty_values_keep_literals_and_noop_backings() {
    {
        let mut dom = owner(Geometry::Tiny);

        let reserved_units = Vec::with_capacity(73);
        assert!(reserved_units.capacity() >= 73);
        let reserved = DocumentString::from_utf16(reserved_units);
        let long = DocumentString::from_utf16([76, 0xd800, 0, 0xdc00].repeat(2048));
        assert_eq!(long.len(), 8192);
        let capacity = long.retained_units();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let element = batch.create_element(&document, "plain");
        batch.set_attribute(
            &element,
            "data-empty",
            DocumentString::from_utf16(Vec::new()),
        );
        batch.set_attribute(&element, "data-long", long.clone());
        let empty = batch.create_text(&document, reserved.clone());
        let text_node = batch.create_text(&document, long.clone());
        batch.insert_before(&element, &document, None);
        batch.insert_before(&empty, &element, None);
        batch.insert_before(&text_node, &element, None);
        let creation = apply(&mut dom, batch);
        let element = creation.created(&element).unwrap().unwrap();
        let empty = creation.created(&empty).unwrap().unwrap();
        let text_node = creation.created(&text_node).unwrap().unwrap();
        let old = dom.snapshot();
        assert_eq!(old.nodes().count(), 6);
        assert_eq!(old.node(element).unwrap().children(), [empty, text_node]);
        assert_eq!(attributes(&old, element).len(), 2);
        for (attribute, local) in attributes(&old, element)
            .iter()
            .zip(["data-empty", "data-long"])
        {
            assert!(attribute.name().namespace_uri().is_none());
            assert!(attribute.name().prefix().is_none());
            assert!(
                attribute
                    .name()
                    .local_name()
                    .code_units()
                    .iter()
                    .copied()
                    .eq(local.encode_utf16())
            );
        }
        assert_eq!(
            attributes(&old, element)
                .item(0)
                .unwrap()
                .value()
                .code_units(),
            []
        );
        assert_eq!(text(&old, empty).code_units(), []);
        assert_eq!(
            text(&old, empty).retained_units(),
            reserved.retained_units()
        );
        assert!(text(&old, empty).shares_storage_with(&reserved));
        for value in [
            attributes(&old, element).item(1).unwrap().value(),
            text(&old, text_node),
        ] {
            assert_eq!(value.len(), 8192);
            let (chunks, remainder) = value.code_units().as_chunks::<4>();
            assert!(remainder.is_empty());
            for chunk in chunks {
                assert_eq!(chunk, &[76, 0xd800, 0, 0xdc00]);
            }
            assert_eq!(value.retained_units(), capacity);
            assert!(value.shares_storage_with(&long));
        }
        let mut batch = DomBatch::new();
        batch.set_attribute(
            element,
            "data-empty",
            DocumentString::from_utf16(Vec::new()),
        );
        batch.set_attribute(element, "data-long", long.clone());
        batch.replace_data(empty, 0, usize::MAX, DocumentString::from_utf16(Vec::new()));
        let noop = apply(&mut dom, batch);
        assert!(noop.changes().changes().is_empty());
        let current = dom.snapshot();
        assert_eq!(current.stamp(), old.stamp());
        assert!(dom.changes_since(old.stamp()).unwrap().changes().is_empty());
        assert!(same_attribute_backing(
            attributes(&current, element),
            attributes(&old, element)
        ));
        assert!(text(&current, empty).shares_storage_with(text(&old, empty)));
        assert!(text(&current, text_node).shares_storage_with(&long));
        let mut batch = DomBatch::new();
        batch.replace_data(text_node, 0, usize::MAX, "cut");
        let edit = apply(&mut dom, batch);
        assert_eq!(edit.changes().changes(), [DomChange::Data(text_node)]);
        let changed = dom.snapshot();
        assert_eq!(changed.revision().get(), old.revision().get() + 1);
        check_topology(&old, &changed);
        assert_eq!(text(&changed, text_node).code_units(), [99, 117, 116]);
        assert_eq!(text(&old, text_node).len(), 8192);
        assert!(same_attribute_backing(
            attributes(&changed, element),
            attributes(&old, element)
        ));
        assert!(
            attributes(&changed, element)
                .item(1)
                .unwrap()
                .value()
                .shares_storage_with(&long)
        );

        drop(edit);
        drop(noop);
        drop(creation);
        drop(dom);
        assert_eq!(current.nodes().count(), 6);
        assert_eq!(text(&old, empty).code_units(), []);
        assert_eq!(text(&old, text_node).len(), 8192);
        assert_eq!(text(&changed, text_node).code_units(), [99, 117, 116]);
        drop(changed);
        drop(current);
        drop(old);
    }
}
