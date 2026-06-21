# surgeist::retained Requirements

`surgeist::retained` is the retained semantic UI model for Surgeist. It owns durable UI identity, canonical element topology, projected traversal topology, semantic element data, classes, attributes, text payloads, behavior hooks, retained interaction state, event routes, structural patches, snapshots, projection caches, dirty projection slots, and change reports.

The public API should be designed for use as `surgeist::retained::*`. The module path supplies the layer name, so public type names should stay short and local: `Model`, `Element`, `NodeRef`, `Id`, `Key`, `KeyPath`, `Kind`, `Role`, `Tag`, `Class`, `Attribute`, `AttributeName`, `Value`, `Text`, `Hook`, `Trigger`, `Intent`, `Command`, `CommandName`, `Event`, `EventKind`, `EventName`, `Propagation`, `Phase`, `State`, `StatePatch`, `StateFlag`, `Presence`, `PointerCapture`, `PointerId`, `Patch`, `Mutation`, `MutationEdit`, `ProjectionEdit`, `ProjectionSource`, `ProjectionSlot`, `SlotKey`, `SlotName`, `VirtualProjection`, `VirtualRange`, `VirtualItem`, `SourceRevision`, `ReplaceMode`, `Snapshot`, `Route`, `RouteStep`, `ChangeFlags`, `ChangeSet`, `Report`, `Error`, `ErrorCode`, and `Result`.

This layer is the durable target for Rust-first immediate authoring, strict HTML ingestion, widget composition, markdown-derived rich text, app-state projection, and future visual editors.

## Scope

This module owns:

- Retained model creation and lifecycle.
- Opaque generated element identity.
- Stable author-provided keys and key-path matching.
- Canonical parent/child topology, child ordering, traversal, insertion, removal, replacement, and movement.
- Projected parent/child topology for derived visual traversal without reshaping canonical ownership.
- Projection slot caches, dirty projection slots, and explicit projected child resolution.
- Retained-owned virtual projection sources for large logical child sets.
- Semantic element data: kind, role, tags, labels, classes, attributes, text, hooks, and child structure.
- Retained interaction flags: hover, active, focus, focus-within, pointer capture, disabled, selected, pressed, checked, expanded, and presence.
- Explicit focus and pointer-capture state.
- Event routes over retained topology, including target, ancestor chain, capture/bubble order, and hook lookup.
- Behavior intent emission through command hooks.
- Structural and state patches from immediate projections, HTML ingestion, widgets, and app state.
- Change reports for retained-owned facts: structure, element metadata, text, hooks, state, presence, focus, pointer capture, and projection slots.
- Read-only snapshots and query APIs for tests, diagnostics, inspectors, and downstream crates.
- Strict string validation and canonicalization for non-text identifiers and hook data.
- Headless contract tests that prove identity, projection, event routing, change reporting, dirty projection slots, and string hygiene through host-independent setup.

## Boundary

`surgeist::retained` owns semantic durability and projected traversal facts. Authoring, markup ingestion, style resolution, layout, text shaping, input hit testing, rendering, accessibility projection, animation, platform integration, widgets, and application command execution are separate producers or consumers around it.

The hard dependency boundary is deliberate: retained stays usable in headless tests, code generation, inspectors, hot-reload validation, and future non-native hosts.

## Dependencies

Expected direct dependencies:

```text
surgeist-retained
  -> optional indexmap or slotmap/slab-like storage
  -> optional bitflags
  -> optional smol_str or compact string storage
  -> optional serde
```

`surgeist-retained` must not depend on `surgeist-window`, `surgeist-render`, `surgeist-text`, `surgeist-shape`, Vello, `wgpu`, Parley, Winit, AccessKit, HTML parsers, CSS parsers, Python, egui, widget crates, DSL crates, or app crates.

The dependency direction should be:

```text
surgeist-ui / surgeist-html / widgets / app projection
  -> surgeist-retained

style/layout/input/access/render bridges
  <- snapshots and reports from surgeist-retained
```

Higher layers emit retained elements, mutations, projection edits, and patches. Downstream layers consume snapshots, projected child lists, dirty projection slots, and change reports.

## Module Layout

The implementation should be split into small internal modules with one public front door. Module names are implementation structure, not public lineage; downstream users should import from `surgeist::retained::*` unless a future API explicitly promotes a submodule.

Planned modules:

- `lib.rs`: crate front door, curated re-exports, feature gates, and module declarations.
- `error.rs`: `Error`, `ErrorCode`, `Result`, and diagnostic context helpers.
- `string.rs`: validated string/token types: `Tag`, `Class`, `AttributeName`, `Value`, `Text`, `EventName`, `CommandName`, `SlotName`, and string hygiene constructors.
- `identity.rs`: `Id`, `Key`, `KeyPath`, key-path namespace components, and identity validation helpers.
- `element.rs`: `Element`, `Kind`, `Role`, `Attribute`, and element construction/validation.
- `state.rs`: `State`, `StatePatch`, `StateFlag`, `Presence`, `PointerCapture`, and `PointerId`.
- `event.rs`: `Hook`, `Trigger`, `Intent`, `Command`, `Event`, `EventKind`, `Propagation`, `Phase`, `Route`, and `RouteStep`.
- `projection.rs`: `ProjectionSlot`, `SlotKey`, `ProjectionEdit`, `ProjectionSource`, `VirtualProjection`, `VirtualRange`, `VirtualItem`, `SourceRevision`, `ReplaceMode`, projection-source validation, and slot resolution helpers.
- `mutation.rs`: `Patch`, `Mutation`, `MutationEdit`, mutation validation, and atomic mutation planning.
- `change.rs`: `ChangeFlags`, `ChangeSet`, `Report`, dirty-slot reporting, and change accumulation.
- `model.rs`: `Model`, private node storage, canonical topology, projected topology, mutation application, projection resolution, focus/capture updates, and event dispatch entry points.
- `snapshot.rs`: `Snapshot`, `NodeRef`, read-only traversal/query APIs, and debug inspection views.
- `tests.rs`: crate-level contract fixtures for identity, mutation, projection, virtualization, routing, change reports, and large-model behavior.

Rules:

- Keep public re-exports intentional in `lib.rs`; modules may stay private even when their types are public.
- Avoid broad `prelude` modules until repeated downstream use proves one is valuable.
- Keep storage helpers private to `model.rs` unless they become a real API boundary.
- Keep validation close to the type that owns the invariant; mutation-level validation can compose lower-level validators.
- Add new modules only when they express a durable boundary. Do not split purely to mirror every type.

## Naming

Public names are authored for the `surgeist::retained` namespace:

- `Model` owns one retained UI model.
- `Element` is authored semantic UI data.
- `Node` is the retained topology entry that pairs an `Element` with identity, canonical links, projected links, state, and dirty flags.
- `Id` is an opaque generated retained identity.
- `Key` is an author-provided stable identity component.
- `KeyPath` is the resolved stable path used for projection matching.
- `Kind` describes the structural kind of an element, such as root, element, text, canvas, fragment, slot, or widget host.
- `Role` describes semantic role used by accessibility and interaction layers.
- `Tag` is a validated structural name for element, slot, and widget-host kinds.
- `Class`, `AttributeName`, `EventName`, `CommandName`, and `Value` are validated string-bearing semantic tokens.
- `Attribute` is a validated attribute name/value pair.
- `Text` is user-visible text payload data.
- `Hook` binds a `Trigger` to a command name.
- `Trigger` describes whether a hook responds to an event or retained behavior intent.
- `Intent` is retained behavior intent, such as command, select, focus, drag, menu, edit, or navigate.
- `Command` is a retained behavior request emitted from a hook.
- `CommandName` is a validated command identifier that may include namespace separators such as `project.run`.
- `Event` is a semantic retained event routed through the model.
- `EventKind` describes the semantic event being routed.
- `EventName` is a validated custom event identifier.
- `Propagation` describes target-only, bubble, or capture-then-bubble routing.
- `Phase` describes capture, target, or bubble route phase.
- `State` stores retained interaction state for one node.
- `StatePatch` describes a partial retained state change.
- `StateFlag` is a state selector fact, such as hovered, active, focused, disabled, or selected.
- `Presence` describes whether a node participates in layout/input/access/render while retaining state.
- `PointerCapture` describes pointer-capture state.
- `PointerId` is a stable identifier for one active pointer stream supplied by input routing.
- `Patch` describes one validated model mutation.
- `Mutation` is an atomic batch of retained edits produced by an authoring pass.
- `MutationEdit` describes either a precise patch or one projection-slot update.
- `ProjectionEdit` describes one immediate-style keyed projection-slot update.
- `ProjectionSource` describes the source used to resolve one projection slot.
- `ProjectionSlot` identifies a projected child outlet hosted by a retained node.
- `SlotKey` identifies the default outlet or a named outlet for one projection host.
- `SlotName` is a validated projection-slot name distinct from element tags.
- `VirtualProjection` describes a retained-owned virtual projection source.
- `VirtualRange` describes a half-open logical item range.
- `VirtualItem` pairs a stable virtual item key, logical index, and materialized element.
- `SourceRevision` is an optional opaque producer generation hint for a projection source.
- `ReplaceMode` describes how a projection preserves identity and retained state.
- `Snapshot` is a read-only view of a model at a point in time.
- `Route` is an ordered event route over retained topology.
- `RouteStep` is one routed node and event phase.
- `ChangeFlags` describes retained-owned facts changed on a node.
- `ChangeSet` lists inserted, removed, moved, and changed nodes.
- `Report` is the result of applying a mutation, projection edit, or patch.
- `Error` and `ErrorCode` are stable diagnostics.
- `Result<T>` is this module's result alias.

Use the module path for layer context. Prefer `surgeist::retained::Model` over `surgeist::retained::RetainedModel`.

## API Stability

The public names listed above are intended as first-pass front-door API contracts. Implementation may change freely behind these names, but downstream crates should not need to learn new vocabulary when retained storage, projection caching, or virtualization internals improve.

Rules:

- Public names should describe retained concepts, not storage details or current algorithms.
- Prefer adding narrow extension points over renaming established concepts.
- Public structs that may grow should use private fields plus constructors, accessors, and builders.
- Public plain-data structs with public fields should be reserved for shapes that are intentionally complete and unlikely to grow.
- `Model` should not implement public `Clone`; cloning large retained models conflicts with the transaction and performance contract.
- `Id` should remain opaque and must not expose storage-shaped accessors such as raw index or generation.
- Enums that are expected to grow, such as `EventKind`, `Intent`, `Patch`, `MutationEdit`, `ProjectionSource`, and `ErrorCode`, should be `#[non_exhaustive]`.
- Error codes are stable diagnostics; add a new code only when callers need to distinguish the failure programmatically.
- Virtualization extends projection through `ProjectionSource`, `VirtualProjection`, `VirtualRange`, and `VirtualItem`; it should not require a separate traversal model or a separate retained node type.
- Experimental helpers should stay private or behind explicit feature gates until their names and contracts are proven.

## Design Lessons

The retained layer should use the useful parts of Masonry's model while remaining a semantic retained model instead of a trait-object widget runtime.

Borrow these ideas:

- Store runtime state beside authored element data rather than inside every authoring API object.
- Separate event handling, state updates, projection resolution, dirty flags, snapshots, and render/access outputs into explicit concepts.
- Treat focus, pointer capture, disabled propagation, hovered/active flags, and presence as first-class retained state.
- Route events by target and ancestor chain, with browser-like bubbling semantics.
- Keep private dirty flags explicit and clear them deterministically so repeated no-op frames do not consume CPU.
- Keep canonical ownership stable while allowing projected traversal to change cheaply.
- Resolve projected child lists explicitly from dirty projection slots instead of eagerly reshaping canonical children.
- Provide inspection and test APIs from the beginning.
- Make generated internal IDs separate from stable external tags/keys.

Design constraints:

- Ordinary app structure is retained as semantic elements and patches.
- Styling and behavior use typed retained facts rather than arbitrary unverified property bags.
- Retained state has no direct layout, Vello scene, Winit event, AccessKit node, or platform lifecycle ownership.
- Patches mutate canonical ownership. Projection updates projected child lists, cached edge mappings, and projection-owned materialization without forcing the canonical parent to match the visual order.

The retained layer should also preserve lessons from `des-document`:

- App-facing authoring should project into retained state with minimal ceremony.
- Repeated input/state updates that do not change semantic state must be no-ops.
- Retained change facts must be precise enough for style, layout, text, input, paint, and accessibility consumers to decide their own work without retained naming those downstream concerns as its own state.
- Behavior hooks should emit typed, testable intent before any host adapter is involved.
- Strict authoring errors are preferable to forgiving repair.
- Tests should prove retained behavior through headless retained contracts.

## Identity Model

`Id` and `Key` serve different purposes.

```rust
pub struct Id(/* private */);

pub struct Key {
    // validated author-provided stable identity component
}

pub struct KeyPath {
    // opaque resolved retained identity namespace path
}
```

Rules:

- `Id` is generated by `Model` and is stable only for the lifetime of the retained node.
- `Id` must not be authored, serialized as app identity, or reused after removal.
- `Key` is authored by Rust DSL, HTML ingestion, widgets, generated UI, or app projection.
- `Key` is used to match immediate authoring output to existing retained nodes.
- `Key` must be unique among siblings that participate in projection matching.
- Duplicate sibling keys are errors.
- `KeyPath` is computed from explicit namespace components, not string concatenation.
- Key-path components include root, canonical keyed child, canonical positional fallback, projection slot, projected keyed child, projected positional fallback, and virtual item key.
- Canonical children, projection-owned children, named slots, and virtual item anchors occupy separate namespaces and cannot collide accidentally.
- Positional fallback components are ephemeral matching aids and should not be used as durable app identity.
- A keyed node preserves its `Id` and retained `State` across projections when its `KeyPath` still resolves to the same semantic slot.
- An unkeyed node may be treated as ephemeral and does not guarantee retained state preservation across projections.
- Reparenting a keyed node is allowed only through explicit move or projection rules that preserve consistency and produce a clear report.
- The root model has a stable implicit key path.

## Element Model

`Element` is authored semantic data. It should be plain, inspectable, cloneable, and independent from retained runtime metadata.

```rust
#[non_exhaustive]
pub struct Element {
    pub key: Option<Key>,
    pub kind: Kind,
    pub role: Role,
    pub label: Option<Text>,
    pub classes: Vec<Class>,
    pub attributes: Vec<Attribute>,
    pub text: Option<Text>,
    pub hooks: Vec<Hook>,
    pub children: Vec<Element>,
}

#[non_exhaustive]
pub enum Kind {
    Root,
    Element(Tag),
    Text,
    Canvas,
    Fragment,
    Slot(Tag),
    Widget(Tag),
}
```

Rules:

- `Element` carries semantic structure, not layout or rendering data.
- `Element` should provide constructors/builders for app-facing authoring; public field examples are conceptual and must not prevent adding semantic facts later.
- Browser-inspired element names such as `div`, `span`, `button`, `section`, and `canvas` may be represented as `Kind::Element(Tag)`. Markup crates perform parsing, repair policy, and source diagnostics before constructing retained elements.
- `Kind::Text` elements use `Text` as their content.
- `Kind::Canvas` marks a retained semantic host for canvas-like content; actual canvas rendering and hit testing belong above or below this crate.
- `Kind::Widget` marks a retained semantic host for reusable widget behavior; widget behavior lives outside this crate.
- `Role` must remain distinct from `Kind`. A `div`-like element can carry a button role, and a widget host can expose a precise role.
- `label` is a semantic label for command surfaces, form controls, and accessibility consumers; visible text remains ordinary retained text content.
- `classes` are ordered and may contain multiple values.
- Attributes are unique by `AttributeName`; setting an existing attribute replaces its value.
- Attribute order should be deterministic, but lookup should not depend on source order unless a higher layer explicitly asks for source preservation.
- Boolean attributes use an explicit empty `Value`; absent attributes remain distinct from present empty attributes.
- Empty text is valid when intentional.
- Element construction must reject malformed tags, classes, attributes, hooks, labels, text, and keys.

## Node Model

`Node` is retained model storage. It pairs authored element data with internal metadata, canonical ownership, projected traversal links, runtime state, and private dirty flags. Its storage fields are private; snapshots expose `NodeRef` for read-only inspection.

```rust
pub struct Node {
    // private retained storage
}

pub struct NodeRef<'a> {
    // borrowed retained node view
}
```

Internally, `Node` may store a normalized form of `Element` that removes recursive child storage after insertion. That normalized form is not part of the public API.

Rules:

- `Model` owns all `Node` values.
- `Node` is private storage; public read access goes through `NodeRef`.
- `NodeRef` exposes stable semantic accessors rather than the normalized internal `Element`.
- Each node has exactly one internal owner: root, canonical parent, or projection slot.
- Canonical parent/child links must always be internally consistent.
- Projected parent/child links must always resolve to live retained IDs or report stale cache entries as errors during validation.
- A live node may have at most one projected parent in the first-pass design.
- Cross-slot reuse, portals, and multi-parent projected edges are future extension points.
- A node cannot be its own ancestor.
- Removed canonical nodes must release their IDs, key-path mappings, projection slots, and projection-owned descendants.
- Removed projection-owned nodes must release their IDs and slot-scoped key-path mappings without mutating canonical children.
- Moving a node must update key paths for that node and its descendants.
- Canonical topology mutations must produce structure change facts for affected canonical ancestors and descendants.
- Projection topology changes must dirty the affected projection host/slot and projected ancestors without implying a canonical parent/child mutation.
- State-only changes must not require structure change facts unless retained state changes canonical or projected traversal eligibility.
- Internal storage should allow efficient lookup by `Id` and by `KeyPath`.

## Canonical And Projected Trees

Retained owns two related structures:

- The canonical tree is the durable source and ownership structure.
- The projected tree is the derived traversal structure used by layout, rendering, input, accessibility, and inspectors when they need visual order.

The canonical tree stores stable ownership:

```rust
enum Owner {
    Root,
    Canonical { parent: Id },
    Projection { slot: ProjectionSlot },
}
```

The projected tree stores derived edges. Its cache entries and node dirty flags are private implementation details.

```rust
pub struct ProjectionSlot {
    // private: host Id plus slot key
}

#[non_exhaustive]
pub enum SlotKey {
    Default,
    Named(SlotName),
}

pub struct SlotName(/* private */);
```

Rules:

- Canonical `parent` and `children` describe ownership, source structure, durable key paths, and mutation authority.
- Slot-scoped key paths include the host key path, slot identity, and projected child key or position.
- Projected parent links and projected child lists describe derived visual traversal.
- Retained exposes per-slot projected child lists as the low-level primitive.
- The default projection slot is the only implicit slot. When the default slot has no cache entry, it falls back to canonical children.
- Named slots have no implicit fallback and return only their resolved slot list.
- `ProjectionSlot` must be constructed through `ProjectionSlot::default(host)` or `ProjectionSlot::named(host, name)`.
- `SlotName` is a retained slot identity token. It may share validation rules with `Tag`, but it remains a separate type so element names and outlet names do not become the same public concept by accident.
- Retained does not compose default and named slots into one visual order in the first pass. DSL, widget, layout, or later composition layers decide how named slots are assembled.
- Projection changes dirty the projection host/slot, not necessarily the canonical parent.
- First-pass projected edges are single-parented. A node cannot appear in more than one resolved projected child list at a time.
- First-pass projection materializes projection-owned nodes inside `Model`; those nodes are owned by a projection slot rather than by the visual host's canonical `children`.
- Projected child lists are cached by `ProjectionSlot`.
- Re-resolving a projection slot updates the cached `projected_children` and projected-parent links only when the resolved list changed.
- Reapplying an equivalent projection should not change canonical topology, projected topology, private dirty flags, or reports.
- Projection resolution must preserve `Id` and `State` for matching keyed projected elements.
- Projection resolution rejects duplicate projected ownership, stale projected children, and projected cycles.
- Dirty flags bubble through the projected parent chain.
- Bubbling stops when an ancestor is already dirty.
- Retained dirty bubbling stops at the projected root or when an ancestor is already dirty; layout containment is handled by the layout crate, not retained.
- Retained must not perform layout, but it must expose precise projected traversal and dirty projection-slot facts so layout can decide where to recompute.

## Virtual Projection

Virtual projection is retained's stable primitive for a linear materialized child window whose logical item count is larger than the currently materialized retained nodes. A separate virtualization/controller layer may map lists, tables, trees, and grids into this primitive or into future projection-source forms.

Retained owns virtual identity, materialized virtual nodes, virtual state anchors, projection cache entries, and dirty-slot reporting. Retained does not own scroll offsets, measurement, item-size estimation, overscan policy, or layout math.

```rust
pub struct VirtualProjection {
    // private fields
}

impl VirtualProjection {
    pub fn dense(total_count: usize, range: VirtualRange, items: Vec<VirtualItem>) -> Result<Self>;
    pub fn with_source_revision(self, revision: SourceRevision) -> Self;
    pub fn total_count(&self) -> usize;
    pub fn range(&self) -> VirtualRange;
    pub fn items(&self) -> &[VirtualItem];
    pub fn source_revision(&self) -> Option<SourceRevision>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VirtualRange {
    // private fields
}

impl VirtualRange {
    pub fn new(start: usize, end: usize) -> Result<Self>;
    pub fn start(&self) -> usize;
    pub fn end(&self) -> usize;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

pub struct VirtualItem {
    // private fields
}

impl VirtualItem {
    pub fn new(index: usize, key: Key, element: Element) -> Result<Self>;
    pub fn index(&self) -> usize;
    pub fn key(&self) -> &Key;
    pub fn element(&self) -> &Element;
}

pub struct SourceRevision(/* private */);

impl SourceRevision {
    pub fn new(value: u64) -> Self;
    pub fn get(&self) -> u64;
}
```

Rules:

- `total_count` is the logical item count for the virtual source.
- `VirtualRange` is half-open: `start..end`.
- `VirtualRange::new` must reject `start > end`.
- `VirtualProjection::dense` must additionally reject ranges where `end > total_count`.
- `items` are the materialized virtual items supplied for the current window.
- First-pass virtual ranges are dense: `items.len()` must equal `range.len()`.
- Every `VirtualItem::index` must be within `range`.
- Every logical index in `range` must appear exactly once.
- `VirtualItem` values must be in deterministic ascending index order.
- `VirtualItem::key` is the stable identity for the logical item within the projection slot.
- Virtual item keys must be unique within one `VirtualProjection`.
- `VirtualItem::index` is a logical coordinate for ordering, diagnostics, and range coverage. It is never durable identity.
- A virtual item's retained key path is derived from the projection slot and virtual item key, not from its transient materialized position.
- `VirtualItem::key` supplies the root identity for the materialized item.
- The root `Element::key` inside `VirtualItem::element` must be absent; child element keys remain meaningful below the virtual item root.
- Retained materializes nodes only for supplied `items`.
- Retained must keep compact virtual state anchors for virtual item keys with durable retained state that would otherwise be lost when their nodes leave the materialized window.
- Virtual state anchors preserve durable retained selector/state facts such as selected, checked, and expanded.
- Ephemeral live state such as hovered, active, focused, focus-within, and pointer-captured requires a live materialized node and is released when that node leaves the resolved projection.
- A virtual item's `Id` is live only while the item is materialized. Apps should use `Key` for durable app identity across virtual windows.
- A virtual item that remains materialized across projection resolution should preserve `Id` under the same key and compatibility rules as ordinary projected elements.
- Re-materializing a virtual item with the same slot and key should restore preserved retained state where applicable.
- Reapplying a `VirtualProjection` with the same range, item keys, source revision, and equivalent elements should be a no-op.
- `SourceRevision` is an optional opaque producer generation hint. It is source-local, not global, and not a durable app version.
- Correctness is based on `total_count`, range, item keys, and element equivalence. A changed `SourceRevision` may help retained decide whether to compare or mark a slot pending, but resolved reports should still describe actual retained topology or state changes.
- Changing `range`, `total_count`, item keys, or item elements dirties the owning projection slot.
- A layout consumer or virtualization controller chooses the virtual range and constructs the `VirtualProjection`; retained validates and stores it.
- Future source forms may support provider callbacks, sparse pinned sections, or chunked ranges, but the stable retained contract remains slot source, item key, logical index, and explicit resolution.

## State Model

`State` is retained runtime interaction state. It is separate from authored `Element` data and from application model data.

```rust
#[non_exhaustive]
pub struct State {
    pub presence: Presence,
    pub disabled: bool,
    pub hovered: bool,
    pub active: bool,
    pub focused: bool,
    pub focus_within: bool,
    pub pointer_captured: bool,
    pub selected: bool,
    pub pressed: bool,
    pub checked: Option<bool>,
    pub expanded: Option<bool>,
}

#[non_exhaustive]
pub enum Presence {
    Visible,
    Hidden,
    RetainedOnly,
}

#[non_exhaustive]
pub enum StateFlag {
    Hovered,
    Active,
    Focused,
    FocusWithin,
    PointerCaptured,
    Disabled,
    Selected,
    Pressed,
    Checked,
    Expanded,
}

pub struct StatePatch {
    pub presence: Option<Presence>,
    pub disabled: Option<bool>,
    pub hovered: Option<bool>,
    pub active: Option<bool>,
    pub selected: Option<bool>,
    pub pressed: Option<bool>,
    pub checked: Option<Option<bool>>,
    pub expanded: Option<Option<bool>>,
}
```

Rules:

- `Visible` nodes participate in layout, input, render, and accessibility snapshots.
- `Hidden` nodes participate in layout snapshots while being excluded from input, render, and accessibility snapshots.
- `RetainedOnly` nodes retain state while being excluded from layout, input, render, and accessibility snapshots.
- Disabled state propagates through effective projected descendants for input-routing and state-query purposes.
- Focus cannot remain on a node that becomes disabled, hidden from input, retained-only, or removed.
- Pointer capture cannot remain on a node that becomes disabled, hidden from input, retained-only, or removed.
- `focus_within` is derived from the focused node and resolved projected ancestor chain.
- `hovered`, `active`, and `pointer_captured` are state facts supplied by input routing.
- `pointer_captured` is derived from the model's pointer-capture table.
- `StateFlag` excludes `Presence`; presence is queried separately.
- State flag changes must produce retained change facts for the changed node. Downstream style, input, paint, and accessibility consumers decide their own work from those facts.
- Reapplying the same state must be a no-op and must not request work.
- `StatePatch` mutates own retained state only.
- `StatePatch` must not expose model-derived fields such as `focused`, `focus_within`, or `pointer_captured`; those are changed through retained APIs.
- Effective disabled and presence eligibility are derived by walking resolved projected ancestors for UI routing.
- Canonical ownership queries may inspect own state without applying projected ancestry.
- "Hidden from input" means a node or one of its effective projected ancestors has `Presence::Hidden` or `Presence::RetainedOnly`, or is disabled.

## String Hygiene

This crate is the first durable semantic boundary, so non-text strings must be validated before they enter retained storage.

```rust
pub struct Tag(/* private */);
pub struct Class(/* private */);
pub struct AttributeName(/* private */);
pub struct Attribute {
    pub name: AttributeName,
    pub value: Value,
}
pub struct Value(/* private */);
pub struct EventName(/* private */);
pub struct CommandName(/* private */);
pub struct Text(/* private */);
```

Rules:

- `Tag`, `Class`, `AttributeName`, `Key`, `EventName`, and `CommandName` must be constructed through strict fallible constructors.
- Identifier-like strings must be non-empty after required canonicalization.
- Identifier-like strings must reject NUL, control characters, malformed whitespace, and unsupported separators.
- `CommandName` may allow namespace separators such as `.`, `:`, `/`, or `-` only when the constructor documents the allowed grammar.
- Constructors must define whether they trim. Tags, classes, keys, event names, and command names should trim boundary whitespace; text should preserve authored whitespace.
- Empty command names are errors.
- `Text` preserves user-authored visible content, including spaces and line breaks.
- `Text` must reject NUL and unsupported control characters while preserving useful Unicode text.
- Sanitization is explicit and reportable.
- Any lossy helper is named explicitly, such as `Tag::sanitize_lossy`, and reports whether it changed input.
- Malformed string content produces stable diagnostics before storage.

## Hooks And Commands

`Hook` binds retained semantic triggers to command names. It is data, not executable behavior.

```rust
pub struct Hook {
    pub trigger: Trigger,
    pub command: CommandName,
}

#[non_exhaustive]
pub enum Trigger {
    Event(EventKind),
    Intent(Intent),
}

#[non_exhaustive]
pub enum Intent {
    Command,
    Select,
    Focus,
    Drag,
    Menu,
    Edit,
    Navigate,
    Custom(EventName),
}

pub struct Command {
    // private fields
}

#[non_exhaustive]
pub enum EventKind {
    PointerEnter,
    PointerLeave,
    PointerDown,
    PointerUp,
    Click,
    ContextMenu,
    KeyDown,
    KeyUp,
    Input,
    Change,
    Focus,
    Blur,
    Select,
    DragStart,
    Drag,
    DragEnd,
    Custom(EventName),
}
```

Rules:

- Hooks store command names and triggers only.
- Application code maps emitted commands to typed actions outside this crate.
- A routed event may emit zero or more `Command` values.
- Commands must include target identity, trigger, route phase, and route context so app code can inspect where they came from.
- `Command` and `Event` use constructors and accessors rather than public fields so routing metadata can grow without source breaks.
- Typed action mapping may be provided by a small registry helper, but retained command emission must remain useful without a concrete app action type.
- Unknown custom trigger names are valid only through strict `EventName` constructors.
- Hook matching must be deterministic when several hooks match the same trigger.

## Event Routing

The retained layer owns event routes over known topology. Physical hit testing and native event conversion supply retained event targets from outside this crate.

```rust
pub struct Event {
    // private fields
}

impl Event {
    pub fn new(target: Id, kind: EventKind) -> Self;
    pub fn intent(target: Id, intent: Intent) -> Self;
    pub fn with_propagation(self, propagation: Propagation) -> Self;
    pub fn with_pointer(self, pointer: PointerId) -> Self;
}

pub struct Route {
    pub steps: Vec<RouteStep>,
}

pub struct RouteStep {
    pub id: Id,
    pub phase: Phase,
}

pub enum Propagation {
    TargetOnly,
    Bubble,
    CaptureThenBubble,
}

pub enum Phase {
    Capture,
    Target,
    Bubble,
}
```

Rules:

- A caller supplies a target `Id` after hit testing, focus routing, accessibility routing, or app logic.
- Ordinary semantic events are constructed with `Event::new(target, EventKind)`.
- Retained behavior-intent events are constructed with `Event::intent(target, Intent)`.
- `Trigger` remains hook-matching vocabulary; callers do not pass a raw `Trigger` to `Event::new`.
- `Model` resolves the target into a route.
- UI event routes use the resolved projected parent chain when projected topology exists, falling back to canonical parents.
- Canonical ancestor queries remain available for ownership, diagnostics, and source inspection.
- Bubble order runs from target to ancestors.
- Capture order runs from root to target when requested.
- The target appears once with `Phase::Target`; capture and bubble steps contain ancestors only.
- Disabled nodes and nodes hidden from input cannot be event targets.
- Events targeting a removed or stale `Id` fail with a stable diagnostic.
- Pointer events that participate in pointer capture must include `PointerId`.
- Event routing should support pointer, keyboard, focus, command, selection, drag, retained intent, and custom semantic triggers.
- Event routing is headless and uses retained topology only.
- Route construction must be testable using only retained topology.

## Focus And Pointer Capture

Focus and pointer capture are retained facts because they affect event routing, state selectors, accessibility, and downstream change decisions.

```rust
pub struct PointerCapture {
    pub pointer: PointerId,
    pub target: Id,
}
```

Rules:

- At most one text focus target exists per retained model unless a future multi-focus design is explicitly added.
- Pointer capture is keyed by pointer identity, allowing simultaneous captures for separate pointer streams.
- Captured pointer events route to the capture target before ordinary hit-test targets; when an event includes a captured `PointerId`, `Model` resolves the capture target even if the caller supplied a different hit-test target.
- Losing focus or capture must emit state changes and change facts.
- Removing, disabling, hiding from input, or making a focused or captured node retained-only must release focus/capture deterministically.
- Focus traversal policy is not first-pass scope. The retained layer should store focusable facts and support explicit focus changes; a later input/navigation layer can own traversal algorithms.

## Mutations, Projection, And Patches

`Patch` is the precise ID-based mutation format for the canonical ownership tree. `ProjectionEdit` is the immediate-style update format for one projected slot. `Mutation` is the atomic batch format that may mix patches and projection edits from Rust DSL, HTML ingestion, widget behavior, or app state.

Projection edits should avoid physically reshaping the canonical tree. They update projection-slot inputs and mark slots dirty; explicit mutable resolution APIs refresh projected traversal when a consumer needs it.

```rust
pub struct Mutation {
    // private fields
}

impl Mutation {
    pub fn new() -> Self;
    pub fn push(&mut self, edit: MutationEdit);
    pub fn with(edit: MutationEdit) -> Self;
}

#[non_exhaustive]
pub enum MutationEdit {
    Projection(ProjectionEdit),
    Patch(Patch),
}

pub struct ProjectionEdit {
    // private fields
}

impl ProjectionEdit {
    pub fn new(slot: ProjectionSlot, source: ProjectionSource, mode: ReplaceMode) -> Self;
    pub fn slot(&self) -> ProjectionSlot;
    pub fn source(&self) -> &ProjectionSource;
    pub fn mode(&self) -> ReplaceMode;
}

#[non_exhaustive]
pub enum ProjectionSource {
    Elements(Vec<Element>),
    Virtual(VirtualProjection),
}

pub enum ReplaceMode {
    PreserveCompatible,
    PreserveIdentity,
    ResetIdentity,
}

#[non_exhaustive]
pub enum Patch {
    Insert { parent: Id, index: usize, element: Element },
    Replace { id: Id, element: Element, mode: ReplaceMode },
    Remove { id: Id },
    Move { id: Id, parent: Id, index: usize },
    ReorderChildren { parent: Id, children: Vec<Id> },
    SetKind { id: Id, kind: Kind },
    SetRole { id: Id, role: Role },
    SetLabel { id: Id, label: Option<Text> },
    SetClasses { id: Id, classes: Vec<Class> },
    SetAttribute { id: Id, name: AttributeName, value: Value },
    RemoveAttribute { id: Id, name: AttributeName },
    SetText { id: Id, text: Option<Text> },
    SetHooks { id: Id, hooks: Vec<Hook> },
    SetState { id: Id, state: StatePatch },
}
```

Rules:

- Patches must be validated before mutation.
- Failed projection or mutation application must leave the model unchanged.
- Patch application must be deterministic.
- `Mutation` batches must be atomic on error.
- `Patch` mutates canonical ownership or live retained state according to owner-kind rules.
- Canonical structural patches are `Insert`, `Replace`, `Remove`, `Move`, and `ReorderChildren`.
- Canonical structural patches may target only canonical ownership.
- Projection-owned nodes are inserted, replaced, removed, and reordered by projection resolution, not by canonical structural patches.
- `SetState` may target any live node.
- Element-data patches such as `SetKind`, `SetRole`, `SetLabel`, `SetClasses`, `SetAttribute`, `RemoveAttribute`, `SetText`, and `SetHooks` may target canonical nodes. Projection-owned element data is updated through its projection source.
- `ProjectionEdit` updates the projection source for a host/slot and marks that projection slot dirty.
- `ProjectionEdit` does not rewrite the host node's canonical `children`.
- A projection slot can represent the default visual children for a host or a named outlet such as a widget slot.
- `ProjectionSlot::default(host)` is the standard slot for ordinary projected children.
- `ProjectionSlot::named(host, name)` is the standard slot constructor for named outlets.
- `ProjectionSource::Elements` is the ordinary concrete child-list source.
- `ProjectionSource::Virtual` is the retained-owned source for virtualized child sets.
- `ProjectionSource` is intentionally non-exhaustive so new source forms can be added without renaming projection APIs.
- Projection matching uses the projection slot, source kind, authored keys, virtual item keys, and sibling key rules before falling back to positional matching for unkeyed concrete children.
- `ReplaceMode::PreserveCompatible` preserves `Id` and eligible state when key path and kind remain compatible.
- `ReplaceMode::PreserveIdentity` preserves `Id` across an incompatible kind only when explicitly requested and reported.
- `ReplaceMode::ResetIdentity` replaces identity and retained state for the projected slot.
- Projection matching allows an immediate authoring pass to rebuild projected child lists while preserving retained state for matching keyed elements or virtual item keys.
- Equivalent projected children must leave the projection cache unchanged and produce an empty change report.
- Projection cache entries store resolved projected child IDs and enough source metadata to skip re-resolution when inputs are unchanged.
- Dirty projection slots are re-resolved through explicit mutable APIs, not eagerly during every projection call and not implicitly from immutable snapshots.
- Re-resolution updates projected-parent edges, projected child lists, and projection change flags only for affected slots.
- Dirty flags bubble upward through projected parents.
- Bubbling stops when an ancestor is already dirty or at the projected root.
- Removing a node removes descendants.
- Moving a node moves descendants with it.
- Removing or moving a canonical node must clear or update any projection cache entries that reference it.
- Applying patches, projection edits, or mutations must produce a `Report`.
- Projection reports duplicate keys, duplicate virtual item keys, invalid virtual ranges, invalid virtual items, invalid parents, invalid indices, cycles, stale IDs, invalid strings, and malformed hooks as stable diagnostics.

## Change Reports And Dirty Flags

Change reports are retained facts supplied to downstream systems. They are not a hidden renderer, layout scheduler, style resolver, or accessibility builder.

```rust
pub struct ChangeFlags {
    // private flags
}

pub struct ChangeSet {
    // private fields
}

pub struct Report {
    // private fields
}
```

Rules:

- Canonical structural changes report `structure`.
- Projected child-list or projected-edge changes report `projection` and the affected `ProjectionSlot`.
- Kind, role, label, class, attribute, text, hook, presence, state, focus, and pointer-capture changes each report their own retained fact.
- Change flags must be explicit, inspectable, and testable.
- `ChangeFlags`, `ChangeSet`, and `Report` expose accessors instead of public fields so new retained facts can be added without source breaks.
- Consumers may use change facts and changed projection slots to skip work, but retained must remain correct if consumers rebuild everything.
- `Report` contains the changes produced by one mutation transaction.
- `Model` accumulates the same change facts until `take_changes` is called.
- `take_changes` clears reported changes only. It must not clear unresolved projection slots.
- Unresolved projection slots are cleared only by successful projection resolution or by removing the owning host/slot.
- `ChangeSet::changed_projection_slots` reports projection slots affected by one committed operation.
- `Snapshot::dirty_slots` reports currently unresolved projection slots.
- A slot is reported in `changed_projection_slots` when it becomes dirty or when resolution changes its projected child list.
- Unresolved dirty slots are queryable without being repeatedly re-reported on no-op frames.
- If a slot is already dirty and receives a different pending projection source, that transaction reports the changed projection slot again.
- If a slot is already dirty and receives an equivalent pending projection source, the transaction reports no new projection-slot change.
- `self_dirty` means a node's own retained inputs changed.
- `child_dirty` means a projected descendant has changed.
- `projection_dirty` means a projected child list or projected edge mapping changed.
- Dirty flags bubble through projected parents, not canonical parents, because visual/layout traversal follows the projected tree.
- Retained records dirty projected positions; layout decides how far layout invalidation must propagate.
- Reapplying equivalent element data or state must produce an empty `ChangeSet`.

## Snapshots And Queries

Snapshots are the public read side of the retained model.

```rust
pub struct Snapshot<'a> {
    // read-only model view
}

impl Model {
    pub fn snapshot(&self) -> Snapshot<'_>;
}

impl Snapshot<'_> {
    pub fn root(&self) -> Id;
    pub fn get(&self, id: Id) -> Option<NodeRef<'_>>;
    pub fn find_key(&self, key_path: &KeyPath) -> Option<Id>;
    pub fn children(&self, id: Id) -> Result<impl Iterator<Item = Id> + '_>;
    pub fn projected_children(&self, slot: ProjectionSlot) -> Result<impl Iterator<Item = Id> + '_>;
    pub fn ancestors(&self, id: Id) -> Result<impl Iterator<Item = Id> + '_>;
    pub fn projected_ancestors(&self, id: Id) -> Result<impl Iterator<Item = Id> + '_>;
    pub fn descendants(&self, id: Id) -> Result<impl Iterator<Item = Id> + '_>;
    pub fn virtual_projection(&self, slot: ProjectionSlot) -> Result<Option<&VirtualProjection>>;
    pub fn effective_presence(&self, id: Id) -> Result<Presence>;
    pub fn is_input_eligible(&self, id: Id) -> Result<bool>;
    pub fn by_class(&self, class: &Class) -> impl Iterator<Item = Id> + '_;
    pub fn by_role(&self, role: Role) -> impl Iterator<Item = Id> + '_;
    pub fn hooks(&self, id: Id) -> Result<&[Hook]>;
    pub fn dirty_slots(&self) -> impl Iterator<Item = ProjectionSlot> + '_;
}
```

Rules:

- Snapshots are read-only.
- Snapshots expose read-only retained references.
- Snapshot traversal order must be deterministic.
- `children` traverses canonical ownership children.
- `projected_children` traverses resolved projected children for one slot, falling back to canonical children only for an unresolved default slot with no projection cache entry.
- `projected_children` returns `ErrorCode::UnresolvedProjection` when the requested slot has unresolved dirty projection state.
- `virtual_projection` returns cached virtual-source metadata only for resolved slots and returns `ErrorCode::UnresolvedProjection` for dirty slots.
- `virtual_projection` returns resolved virtual source metadata for a slot when the resolved source uses `ProjectionSource::Virtual`.
- Pending virtual sources are observed through dirty-slot facts until explicitly resolved.
- The snapshot API does not expose projected descendants in the first pass; callers compose repeated per-slot child queries explicitly.
- Immutable snapshots never resolve projections.
- Projection resolution is explicit through mutable `Model` APIs.
- Raw traversal methods return all live retained nodes, including hidden and retained-only nodes.
- Eligibility helpers such as `effective_presence` and `is_input_eligible` expose downstream participation decisions explicitly.
- Snapshot APIs should make test assertions concise.
- Query APIs must be explicit about whether they include hidden or retained-only nodes.
- Debug reports should include key path, kind, role, classes, state, projected parent, projected children, dirty flags, and change facts.
- A snapshot should be enough for downstream style, layout, input, accessibility, and diagnostics consumers to make their own decisions after required projection slots have been resolved.

## Model API

```rust
pub struct Model {
    // private retained storage
}

impl Model {
    pub fn new(root: Element) -> Result<Self>;
    pub fn empty() -> Self;
    pub fn root(&self) -> Id;
    pub fn snapshot(&self) -> Snapshot<'_>;
    pub fn apply(&mut self, patch: Patch) -> Result<Report>;
    pub fn apply_projection(&mut self, projection: ProjectionEdit) -> Result<Report>;
    pub fn mutate(&mut self, mutation: Mutation) -> Result<Report>;
    pub fn resolve_projection(&mut self, slot: ProjectionSlot) -> Result<Report>;
    pub fn resolve_dirty_projections(&mut self) -> Result<Report>;
    pub fn route(&self, event: Event) -> Result<Route>;
    pub fn dispatch(&mut self, event: Event) -> Result<Report>;
    pub fn focus(&mut self, id: Option<Id>) -> Result<Report>;
    pub fn capture_pointer(&mut self, capture: PointerCapture) -> Result<Report>;
    pub fn release_pointer(&mut self, pointer: PointerId) -> Result<Report>;
    pub fn take_changes(&mut self) -> ChangeSet;
}
```

Rules:

- `Model::new` validates the entire element tree before retaining it.
- `Model::empty` creates a valid root model.
- `apply` is for one patch.
- `apply_projection` records one projection-slot source and dirties that slot.
- `mutate` is for a validated atomic batch of patches and projection edits.
- Mutation application records projection-slot inputs and dirty slots; it should not eagerly re-resolve every dirty projection unless required for validation.
- `resolve_projection` re-resolves one dirty slot and reports projected child-list changes.
- `resolve_dirty_projections` re-resolves currently dirty slots in deterministic order for consumers that want eager consistency before traversal.
- `Patch::Replace` uses `ReplaceMode` to make identity and state preservation explicit.
- `dispatch` may update retained state and emit commands.
- `route` does not mutate and routes only over resolved projected topology.
- `route` returns `ErrorCode::UnresolvedProjection` when the target route depends on dirty or unresolved projected topology.
- Callers that need projected routing after projection edits must call `resolve_projection` or `resolve_dirty_projections` first.
- Pointer capture is set and released per `PointerId`.
- Mutation methods return reports rather than hiding side effects.
- The public API should favor fluent, clear calls but keep the retained layer lower-level than the authoring DSL.

## Scale And Performance

The retained model must be designed for large real applications. A 10,000-node retained model is a baseline contract, not an edge case.

Rules:

- Creating, validating, and snapshotting a 10,000-node model must be practical in unit tests.
- Lookup by `Id` and `KeyPath` must be indexed and should not require full-tree scans in normal paths.
- Applying a localized patch to a 10,000-node model should touch the changed node, required ancestors or descendants, and relevant indexes rather than rebuilding unrelated subtrees.
- Applying a localized projection to one slot in a 10,000-node model should dirty only that slot, changed projected nodes, and affected projected ancestors.
- Reapplying an equivalent 10,000-node projection should produce an empty `ChangeSet` and avoid avoidable per-node churn.
- Projection cache lookup by `ProjectionSlot` should be indexed.
- Projection resolution should scale with the affected slot/subtree, not the whole canonical model.
- Virtual projection resolution should scale with the materialized item count plus preserved state anchors touched by the window change, not `total_count`.
- A virtual projection with 200,000 logical items and a 100-item materialized range should not allocate 200,000 nodes.
- Event routing in a deep or broad 10,000-node model should scale with projected route depth, not total node count.
- `take_changes` should report accumulated change facts and dirty slots without scanning clean nodes.
- Snapshot traversal may be linear when the caller explicitly traverses a subtree or whole model.
- Projected traversal reports unresolved slots; explicit projection resolution is the measurable mutation point.
- Contract tests should use deterministic fixture generation so large-model failures are reproducible.
- Performance assertions should avoid brittle wall-clock thresholds by default; prefer structural counters, operation counts, changed-node counts, and debug instrumentation. Optional ignored benchmarks may add wall-clock budgets later.

## Error Model

Errors must be stable enough for tests and tooling.

```rust
#[non_exhaustive]
pub enum ErrorCode {
    InvalidString,
    EmptyCommand,
    DuplicateKey,
    MissingNode,
    StaleId,
    InvalidParent,
    InvalidIndex,
    InvalidVirtualRange,
    InvalidVirtualItem,
    Cycle,
    InvalidMove,
    InvalidPatch,
    InvalidProjection,
    InvalidRoute,
    UnresolvedProjection,
    DisabledTarget,
    IneligibleTarget,
    UnsupportedFeature,
}
```

Rules:

- Every error includes an `ErrorCode`.
- Errors should include the relevant `Id`, `KeyPath`, string field, patch index, or trigger when available.
- Errors should explain what failed and which invariant would have been violated.
- Strict errors are preferable to implicit repair.
- Error text should be useful for authoring and hot-reload workflows even though hot reload lives above this crate.

## Testing Requirements

Contract tests must prove:

- New models validate identity, topology, keys, strings, and parent/child consistency.
- Duplicate sibling keys fail.
- Invalid strings fail before entering storage.
- Immediate projection preserves `Id` and `State` for matching keyed elements.
- Projection updates projected child lists without rewriting canonical `children`.
- Projection cache stores and reuses equivalent resolved child lists.
- `ProjectionSource::Elements` and `ProjectionSource::Virtual` both resolve through the same slot contract.
- Virtual projection validates range bounds, dense item coverage, item order, item indexes, root element key conflicts, and duplicate virtual item keys.
- Virtual projection materializes only supplied items and preserves eligible state by slot plus virtual item key.
- Virtual projection preserves `Id` while a keyed virtual item remains materialized across resolution.
- Virtual projection can update a small materialized range inside a very large `total_count` without full-model churn.
- Key paths distinguish canonical, projected, slot, positional fallback, and virtual item namespaces without collisions.
- Dirty projection slots are reported precisely.
- Re-editing an already dirty slot reports the slot only when the pending projection source changes.
- Dirty flags bubble through projected parent links and stop when an ancestor is already dirty.
- A node cannot have more than one projected parent in the first-pass design.
- `route` and snapshot projected traversal fail clearly when relevant projection state is unresolved.
- Projection removes stale nodes and reports removed descendants.
- Mutation batches are atomic on error.
- Moving nodes updates descendants and key paths.
- Focus and pointer capture are released when targets are removed, disabled, hidden from input, or made retained-only.
- Disabled state propagates through resolved projected ancestry for routing and state-flag queries.
- Reapplying equivalent state is a no-op.
- Event routing produces target, capture, and bubble order deterministically.
- Hooks emit commands with target and route context.
- Change reports distinguish retained facts: structure, kind, role, label, classes, attributes, text, hooks, presence, state, focus, pointer capture, and projection.
- `take_changes` clears accumulated change facts without clearing unresolved projection slots.
- Snapshots support deterministic traversal and query APIs through retained-only dependencies.
- Snapshot traversal returns all live nodes unless an explicit eligibility helper is used.
- 10,000-node models can be created, projected, patched, queried, routed, snapshotted, and changed without full-model churn for localized changes.
- Equivalent 10,000-node projections preserve retained identity and produce empty changes after the initial projection.
- A localized edit in a 10,000-node model reports only the edited node, required ancestors or descendants, affected projection slots, and the relevant change facts.

## Future Extension Points

These are intentional extension points, not first-pass requirements:

- Focus traversal policy.
- Multi-window retained model ownership.
- Cross-model portals.
- Shadow-root-like composition boundaries.
- Rich editor state.
- Full accessibility node projection.
- Style selector indexing.
- Animation state.
- Dev inspector metadata.
- Serialization for persisted UI templates.
- Incremental diff import from external markup or generated UI descriptions.
- Provider-backed projection sources.
- Sparse pinned virtual sections.
- Chunked virtual ranges.
