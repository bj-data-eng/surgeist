# Input state and processing

Input retains its current values and flags in the canonical owner. [Form associations and reflected targets](forms-and-select.md) and [Textarea values](text-controls.md) have related but distinct contracts.

## Values, radio groups and reflected targets

Native HTML Input has a checked `DomNode::as_input()` view over actual flags,
current value, selection/direction, selected-file state, Image history and
custom error. The view's `value()` applies HTML's current/default/default-on/file
getter mode; `state().current_value()` reads the separately retained internal
string. Every shared type state is admitted. Type/default/checked/name/multiple,
Range constraints and Color alpha/colorspace edits execute their real source
effects rather than reconstructing current state during reads. String/temporal,
Range and Color grammar and normalization remain with the shared producers.

`set_input_value`, `set_input_checked`, `set_input_indeterminate` and `reset_input`
are atomic native operations in `DomBatch`. Checkedness and dirty checkedness
are separate from checked content. Value setters compare the final sanitized
value, clamp real selection and apply the source's additional cursor effect.
Reset follows the shared per-control order; it does not dispatch a cancelable
whole-Form reset event or clear unspecified state. Custom error storage is shared
with Button/Fieldset and consumes its own normalized retained units.

Radio normalization uses the actual ordinary root, exact nonempty name and
current same/null Form owner. Owner reset applies radio effects immediately at
each clear/association, so A-to-null-to-A is observable in peers even when the
final owner is unchanged. Connected insertion, actual name/type changes and
true checkedness invoke their own source triggers. Shadow roots partition
ordinary groups; peer uncheck preserves peer dirty/default/indeterminate flags.
No second group cache or end-of-batch repair is maintained.

Input and Button share actual reflected-target writes and retirement. Input has
one popover target and no commandFor slot. `set_popover_target_element` accepts
either real family, including same-batch, self, detached and other-document
Elements. Its raw weak slot and HTML's scoped semantic getter remain distinct.
Generic popovertarget content changes clear the explicit slot even on redundant
sets. Clone initializes fresh weak/message/files/selection/history state, copies
attributes through actual hooks and applies only the Input cloning overlay for
current value and source-selected flags. Color cloning does not resolve its
default again before copying the actual current value. Adoption preserves live
state except actual owner/tree hooks; explicit retirement clears exact weak
references and releases all retained strings without following reused IDs.

## Native platform and resource policy

`Dom::with_native_input_platform` selects the two independent existing HTML
empty-selection and none-direction support enums. `Dom::new` chooses the named
`DomNativeInputPlatform::headless()` Supported/Supported profile. These immutable
facts are explicit native policy, not consequences of absent browsing context.
Native Documents and their Template inert owner Documents retain the establishing
environment. Adoption does not reset selection merely because it reads a Document.
Native image acquisition is explicitly disabled, with no enabled option. Image
entry/re-entry and src hooks retain exact absent/empty/nonempty source history and
selected zero coordinates; no URL validation, fetch/error/event or resource object
is fabricated.

Native File Upload Input exposes its exact `DomFileList`, including the
fresh empty selection. A FileList keeps its order and File identities.
`DomSourceEdit::assign_input_files` installs the supplied object in File
Upload state. A null assignment or a different Input state leaves the
selection unchanged. Assignment dispatches no input or change event.

A native host submits an actual choice through
`DomSourceEdit::queue_input_file_selection`. Source qualifies the Input's
observed DOM stamp, current mutability and permitted file count. The admitted
user-interaction task owns the exact list. Selection changes only when that
task executes, before its ordered input and change events. See the
[Runtime task contract](../../surgeist-runtime/docs/explanation.md#dom-orchestration).

`DomSourceLimits` counts each controlled File and FileList materialization once
per DOM owner. The retained-byte allowance counts objects, controls and backing
capacity, including the body and member vectors. The retained UTF-16 allowance
counts filename, media type and Filename-mode readback backing capacity.
`DomUsage::string_code_units()` reports the canonical DOM string budget for
attributes, current control values and other in-tree strings. See the
[storage and custody contract](storage.md#what-the-admission-budget-counts).

Filename-mode value readback uses the HTML prefix and the first selected
filename. An empty value write and native reset clear a nonempty selection
by installing a fresh empty list. Other references to the original list
still observe its original members. A nonempty value write returns
`InputFilenameInvalidState`. Adoption preserves the selection. Clone
and entry into File Upload establish a fresh empty list.

The [shared selection Source edits](text-controls.md#native-selection) apply
positional selection to Text, Search, Telephone, URL and Password. Other Input
types expose null through the positional getter and reject positional edits
with `InputSelectionInvalidState`. Select-all follows its own applicability
rule. The current native profile supplies selectable text for value text
controls. Picker and other rendered control text have no native producer.

## Native stepping

`DomSourceEdit::step_input` invokes the chosen direction with a signed count.
It reads the live Input type and reached attributes from the same Source
operation. Number, Range, Date, Month, Week, Time and Local Date and Time
support stepping. Other types and `step=any` return `InputStepInvalidState`.
The operation preserves the dirty-value flag and publishes no input or change
event. [Pinned stepping rules](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-input-stepup)

HTML computes the step grid exactly. Range compares finite lattice candidates
against the exact target and chooses the larger candidate at a tie. Temporal
coordinates stay exact through state-specific serialization. Number serializes
the finite binary64 result and reports a typed numeric overflow if that
conversion fails. Variable producer work uses both the Source operating
allowance and the separate Input processing allowance.

## Finite numerical and CSS processing

Reached native stepping and variable Range/CSS/Color processing require caller-selected
`DomLimits::with_input_processing(scratch_bytes, work)`. The seven-argument
constructor leaves both capacities zero, so fresh empty Text uses no numeric/CSS
service or invented numerical allocation. Other types remain real Input states;
insufficient processing allowance rejects the whole batch atomically. Scratch
uses `usize` bytes for the peak logical live adapter layouts, heterogeneous banks,
digits and output, including old/new allocation overlap and actual capacities.
It is not allocator bookkeeping/RSS or all Rust stack storage. Work is a separate
`u64` cumulative credit ceiling across every reached service in the private batch,
including nested hooks, with a typed `InputProcessingWorkLimit` stop. Logical work
can exceed address size without allocating that many bytes.

The private services charge reached protocol/fixed actions and described
observations/writes, register/action envelopes, actual retained-prefix movement
and output transfer. Fees are DOM's documented service policy, not parser credit
identity. Reservations are fallible, sizes checked and cancellation polled before
actions/copies; actual owner-borrowing banks are dropped before phase/owner reuse.
Standard-invalid Color parsing alone chooses HTML's black fallback. Missing CSS
facts, numerical failures, resource exhaustion and cancellation stay distinct
typed refusals. Current/default/message/Image-history strings independently
consume canonical retained UTF-16 admission. Lowering processing limits
does not erase admitted values. Picker and file/image acquisition remain
host services. JavaScript callers require a bindings adapter.

[Back to surgeist-dom](../README.md).
