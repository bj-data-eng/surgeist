---
name: surgeist-admin
description: Manage Surgeist plans, progress records, verification evidence, temporary files, and pause/resume handoffs. Use for administrative work or the administrative part of ongoing implementation; leave implementation and review methods to PISCT.
---

# Surgeist administration

Keep enough durable information to resume and verify work without preserving
every intermediate file. This repository-owned skill supplies storage and
retention conventions for coordinators and workers. It does not change product
scope, PISCT's engineering checks, or the user's implementation, commit, and
publication authority. Explicit user instructions take precedence, including
requests to pause, retain an export, or delete particular records.

Scale administration to the work. A small completed edit may need only its Git
change and a concise result; create a ledger record only when something needs
durable coordination or tracking. Update records at meaningful decisions,
verification boundaries, or handoffs, not after every command.

## Give each fact one home

| Information | Authoritative home |
| --- | --- |
| Product behavior, fixtures, required provenance, public API | Tracked source and its existing owning documentation/artifact paths |
| Intended outcome, design, boundaries, acceptance criteria | One complete Markdown plan in the owning workspace's Plans ledger |
| Requirement status, findings, next action, verification and review summaries | The relevant requirement or work row in its existing ledger |
| Current checkout and committed changes | Git; inspect it again on resume |
| Drafts, captured output, machine-readable intermediates | Ignored `tmp/`; never the sole source of an accepted decision or completed verification claim |

Keep administrative records in Markdown fields the dashboard can render. JSON
is an exchange format or working input, not a parallel progress database. A plan
describes intended work; an execution record describes what actually happened.
Replace current summaries when facts change instead of appending a diary to the
plan. Link related rows rather than copying their bodies.

## Select the owner and retrieve narrowly

Use PISCT project **surgeist**. Use workspace **surgeist** for root integration
and one workspace named after each crate for its work, such as **surgeist-css**.
These are ledger namespaces, not Cargo workspaces or separate repositories.
Create missing workspaces only when actual work needs them. A cross-crate change
still has one primary owner; related rows point to that owner's plan or decision.
Do not create a complete set of ledgers for every crate in advance.

Discover the live registrations and ledger titles with `pisct workspace list`
and `pisct --workspace NAME ledger status`. Replace `NAME` with the selected
workspace. Use the CLI's configured registry; avoid hardcoded database paths.
Inspect a ledger's definition before selecting fields or writing. Use current
CLI help for guarded writes, pagination, and storage limits; this skill is not
a second CLI manual.

Start with title/status or requirement/state/next-action columns that actually
exist. Read the relevant plan body and work rows, then open supporting source
only when needed. Do not load whole ledgers, all JSON files, or raw logs merely
to orient yourself. Follow pagination when searching for a record or proving
that none exists. If durable storage is unavailable, preserve unsaved drafts
and report the limitation; do not silently create a substitute authority.

Name references with their workspace and ledger/row address. Use the current
row for ongoing work; include an exact revision only when a decision or review
depends on those exact bytes. Keep semantic requirement IDs intact across moves.
Row numbers alone are not global identities.

## Plans and current work

Reuse an existing plan's row for revisions. Store the complete current plan,
including adopted refinements that still apply; do not make the reader assemble
it from a base document and a chain of amendments. Split a plan only where work
has an independently useful scope and acceptance boundary. Original imported
plans need consolidation only when that work is revisited, not a bulk rewrite.

Follow the installed PISCT planning skill's storage procedure when saving plans:
validate Markdown, use guarded structured writes, read back the exact stored
revision, and compare the body with the draft before removing it. Invoke its
planning workflow when the user requests it. Preserve acceptance and completion
evidence when importing existing plans; storage or review alone does not grant
acceptance. An accepted plan is not permission to resume paused implementation.

Use the existing work row's status, evidence and next-action fields for the
current checkpoint. If there is no suitable row, create one for the actual
outcome in an appropriate existing ledger. Do not create a new plan, handoff
ledger, or status JSON just to record a pause. Fit the existing schema; a pause
can be stated in next-action text without inventing a new status choice.

Record the minimum needed for another coordinator to continue:

- Current phase and disposition, including user pause or unresolved blocker.
- Relevant plan/requirement references and the owning crate or root boundary.
- Git commit and, for uncommitted work, affected paths and their verification
  basis; include a diff or content digest when evidence depends on exact bytes.
- Completed checks and review conclusions, incomplete checks, and next action.
- Active writer/process ownership and any temporary artifact still needed.

The coordinator writes shared checkpoint and disposition records. Delegated
workers return concise results and their owned temporary paths unless ledger
ownership was explicitly delegated. Preserve other agents' work and records.

## Evidence that survives cleanup

For a check, retain the source basis, working directory, exact command and
relevant environment/feature/toolchain selection, exit or interruption result,
and a short outcome. Include pass/fail/ignored counts where useful and the
specific failure that matters. Distinguish successful checks, expected RED,
unexpected failures, interruption, and process-cleanup failure. Partial success
does not establish a complete suite pass.

For a review, retain the reviewed source basis, review scope, independent
reviewer identity when available, verdict, and actionable findings/dispositions.
For committed RED/GREEN requirements, retain both exact commits and reproduction
commands. Reuse the owning row for this evidence; use a separate linked record
only when the evidence is genuinely shared or too substantial for that row.
This does not replace required checks or independent reproduction.

A log path, JSON filename, or checksum alone is not evidence of success. Before
discarding a uniquely useful report, put its decisive observations and source
references in the owning row. If raw data is essential to reproduce an outcome,
preserve it as a proper fixture or source artifact in its existing repository
owner, with required provenance and attribution, within authorized scope. If
that cannot yet be done, keep the owned temporary input and record why and when
it can be removed. Never turn `tmp/` into a required build or test input.

Identify specifications by dated URL/version, relevant anchors and hash where
required; identify source by commit and repository-relative path. Local download
locations are conveniences. Required pinned bytes and immutable review material
must remain available for their actual consumers, not merely have a remembered
filename.

## Temporary file lifecycle

All paths below are relative to the repository root and are created on demand.
Use a short topic or task subdirectory when ownership or collisions require it;
there is no mandatory file manifest, historical-path index, or per-turn report.

| Area | Use | End of lifetime |
| --- | --- | --- |
| `tmp/plans/` | Active plan and review drafts | After required review and verified ledger storage; keep unfinished or unsaved drafts |
| `tmp/logs/` | Output needed for an active diagnosis or review | After recording the useful result and finishing that consumer |
| `tmp/json/` | Ledger write inputs, selected query results, generation intermediates | After successful write/readback or the consuming operation completes |
| `tmp/references/` | Downloaded reference material needed by active work | When no active consumer needs it and the durable source identity is recorded |
| `tmp/work/` | Other owned experiments and temporary helpers | When the experiment or handoff is complete |

Honor a tool's required temporary location without making a second copy here.
Keep an uncertain write's exact input until its outcome is reconciled. Existing
temporary material outside these areas can stay until its owning work is touched;
do not perform an unrelated mass migration to satisfy the naming convention.

Stream routine command output; do not save successful output by default. Save a
log only when its contents have a named debugging or review consumer. Read the
failure excerpt or relevant fields, not the full capture. Hash raw material when
identity or provenance matters, not every disposable scratch file.

When moving a cache, retain useful filenames and update a current consumer only
if it actually needs the new location. Do not rewrite historical reports,
maintain redirect copies, or create old-to-new path inventories. A missing cache
is not a missing requirement: consult the durable record and recreate it only
when needed and authorized. Do not claim a deleted artifact was inspected or
that unavailable evidence is independently verified.

## Pause, resume, and finish

On pause, stop or account for owned workers/processes using PISCT process
handling, save the current checkpoint, and preserve unfinished source edits and
inputs still needed. A clean temporary directory is not more important than
recoverable work. Do not stash, commit, discard, or resume code just to satisfy
an administrative request.

On an authorized resume, read the owning checkpoint and relevant plan, inspect
Git and active ownership, and reconcile changed source before reusing evidence.
If a prior check was interrupted, the missing check remains outstanding. Restore
only the working material the next action needs.

At a completed boundary, store the durable result and confirm it is retrievable,
then remove the current task's consumed drafts, logs, JSON inputs and helpers.
For shared or pre-existing files, establish ownership and retention needs before
deleting; never sweep by age, filename prefix, or extension alone. Ledger purge,
unrelated cleanup, history changes, and publication require their own applicable
authorization. A user's explicit deletion request takes precedence; report any
resulting loss of needed evidence without inventing a new retention obligation.

Report the substantive result, its durable location, and remaining work briefly.
Administrative completion means the work can be resumed or assessed from its
owning records and source, with temporary exceptions accounted for. It does not
mean that the product milestone is complete.
