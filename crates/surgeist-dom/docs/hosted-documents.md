# Host observations and hosted documents

Host captures are immutable observations. Hosted creation and destruction instead run controlled retained operations through Runtime and the same canonical owner. Their current execution authority differs from historical snapshot state.

## External Host captures

`DomHostPublisher` captures complete fixed focus/hover or document-target Element
observations from an external producer against one exact `DomDocumentStamp`.
Supported absence, unsupported capability and inapplicability remain distinct.
Its immutable source identity includes the selected domain configuration; checked
`HostStateRevision` advances only when observations change. DOM-only revisions
require explicit whole-domain recomputation without advancing equal host facts.
Captures retain exact historical values independently of the publisher. Complete
adjacent coverage uses constant storage, with explicit fallback for older history
and whole-domain invalidation for availability changes. The independent
`DocumentTarget` variant retains the host's one selected target Element or known
absence, following HTML's target-elements contract. It does not navigate, decode
fragments or infer a target from IDs or URLs. An unknown current observation is
not a supported absence: a producer must obtain a qualified value or report its
unavailability without publishing. This boundary
does not implement HTML focus policy, pointer processing or canonical controls;
Runtime owns its concrete surface-selection association and capture adapter.
It rejects document-target bindings because it has no target-selection producer.

## Controlled hosted creation

Native hosted creation is a separate, retained source operation driven by
Runtime's `HostedRuntime` through the same `DomSession`. The DOM source issues
owner-qualified, generation-checked Host, browsing-context, Global, history-entry
and navigable identities. Its controlled phases establish the initial Document,
Permissions Policy, actual Global association, loaded time and initial entry.
Operation and phase checks reject foreign, stale and replayed publications while
preserving the valid continuation. Dropping a permission does not finish its
operation. Explicit record, byte and work limits admit the actual state.

The initial no-opener constructor establishes one immutable COOP policy and
URL/base/encoding payload before handing execution to Runtime. The published
Document borrows these actual values through `opener_policy()` and
`base_document_facts()`; other native creation profiles return `None`. Its
`document_base_record()` selects the actual current base through the same Base
owner as `document_base_url()`. Reads and retries preserve the constructor's URL,
encoding and provenance identities, including in historical snapshots after
destruction. These facts do not qualify an external URL parser or its responses.

The initial hosted Document has a real non-null, complete empty Global registry.
Its canonical registry view consumes the original HTML-semantics lookup over the
retained producer capture. A known definition miss is distinct from a null or
unavailable registry. This bounded factory supplies no definition mutation or
scoped registry implementation.

## Activity, children and lifecycle retirement

`DomDocument::is_fully_active()` reads the actual captured hosted association;
closed Documents return false. Snapshot `global_document` and `relevant_global`
lookups also retain historical identity. The corresponding methods on the current
`Dom` owner check the live issuer, identity and association before execution.
Native Frame/Iframe Source hooks create initial children and retain their actual
parent/container relationships and queued history work in this same owner.
Runtime computes the selected initial fullscreen and focus-without-user-activation
Permissions Policy; required
qualified URL observations remain explicit external inputs. Child entry target
names preserve absent, supplied-empty and exact supplied values. The fixed
top-level target name remains supplied-empty. Iframe creation consumes its
retained sandbox flags; declared-origin policy processing separately consumes
the source-selected raw sandbox attribute. Later attribute edits do not rewrite
an already completed Document policy.

Destruction retires hosted participation without deleting the managed Document
or its tree. Canonical creation, clone/import, adoption, shadow attachment and
native fragment installation can still consume its actual metadata, original
Global identity and retained registry. Input and Textarea keep using its fixed
platform policy. XML creation copies the associated origin and policy into a
fresh closed Document. None of these facts restores execution permission or
supplies unavailable family/resource services.

Idle mutable Style custody has a
[qualified withdrawal](source-and-style.md#mutable-cssom-custody-and-disabled-state)
at the original destruction cleanup. Ordinary detach or an empty producer list
alone does not release that custody.

Current staged activity is false
for actual retired participation; expired or unknown live authority remains an
error. Dialog hooks and standalone Img use that same activity decision.
An older snapshot retains its captured activity state. Live-child removal is
distinct from retained history and cleanup custody. Queued initial history
attachment remains obligatory after removal, and completed Document destruction
clears the actual entry's nullable
Document field without deleting the managed Document. These initial-entry
operations do not establish general navigation, replacement, resource loading, modal services or
JavaScript execution. The [Runtime explanation](../../surgeist-runtime/docs/explanation.md#dom-orchestration)
describes native task custody and checkpoint limitations.

[Back to surgeist-dom](../README.md).
