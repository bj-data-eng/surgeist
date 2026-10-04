---
name: surgeist-admin
description: Coordinate Surgeist issues and Project state, local working material, and pause/resume handoffs. Use for repository administration during authorized work; PISCT owns implementation and review methods.
---

# Surgeist Administration

Use [Surgeist Coordination](https://github.com/users/bj-data-eng/projects/1) as
the single progress record for completed work and actionable future work in
`bj-data-eng/surgeist`. The coordinator owns issue and Project updates within the
user's authority. This convention grants no implementation, issue-creation,
closure, commit, publication, or cleanup authority. Preserve user pauses.
Use PISCT's focused skills for engineering and review.

## Work And Ownership

An issue states the outcome, acceptance criteria, ownership boundaries, and
relevant sources. Use module → crate contributions → bounded properties or tasks
where that grouping fits the work. Own shared capabilities once and link
dependent work to that owner. Native parent-child relationships describe scope;
blocking dependencies describe execution order. Keep both acyclic. Completing a
leaf does not certify the enclosing contribution or module.

Issue decomposition and relationships describe the current working organization.
Within existing authority, the coordinator may combine, split, move, or reorder
work and remove obsolete blocking edges when implementation reveals a simpler
route. Preserve underlying requirements, coverage, ownership, genuine
prerequisites, and explicit human constraints; reconcile active assignments and
update affected issues and relationships. Reuse Review results only for the
requirements and source they actually cover; regrouping alone requires no new
review, while uncovered work still needs its applicable evidence.

Read the relevant issue, its parent, and actual prerequisites before expanding
the search. Include closed issues and archived Project items when reconstructing
progress; an empty active view does not establish absence. Reference material
supports acceptance criteria without automatically becoming another issue.

Use the existing fields when useful: Crates, Workstream, Boundary, Priority,
Plan, and Needs. Assign Workstream explicitly, not from GitHub author identity.
Boundary distinguishes Root-owned, Crate-owned, Cross-crate, and
Upstream-blocked work. Keep requirement coverage and consequential decisions
with their owning issue or tracked source. Do not build a parallel progress
database in local files, field values, or comments.

## Project State

Surgeist uses issues and direct commits; no PR workflow is required. Planned
issues are managed through the Project. Authorized API creation can attach an
issue in one `createIssue` call using the explicit Surgeist `repositoryId` and
Project node ID in `projectV2Ids`; the UI's default repository does not supply
the API argument. Read the existing identities instead of guessing them.

Status choices are Backlog, Ready, In Progress, In Review, Blocked, and Done.
The enabled built-in workflows own these effects:

| Event | Effect |
| --- | --- |
| Matching issue created or updated, or sub-issue of a Project issue | Automatic Project intake |
| Issue added | Status Backlog |
| Status Done | Issue closed |
| Issue closed | Status Done |
| Issue reopened | Status Ready |
| Closed issue unchanged for more than two weeks | Archived at the next 12-hour check |

The coordinator records planned completion through Status. Close unplanned work
directly as **not planned** and let the built-in closure rule set Done. Done
means resolved or unplanned; it does not assert verified implementation.
Before changing Status or issue state, account for these side effects and inspect
the resulting state. Do not duplicate built-in transitions with synchronization
scripts or assume an event order. Archive timing follows the last update, not
simply the closure date. Workflow settings are user-owned; inspect saved
configuration when changing or troubleshooting it within authorized scope.

## Review And Working Material

Review is separate from Status: Not Started, CLEAN, or NOT CLEAN. The coordinator
alone updates Review; reviewers remain read-only against GitHub. Keep detailed
verdicts and supporting evidence local while they serve an active review,
diagnosis, or handoff. No linked review record or compulsory report file is
required in GitHub. PISCT's review skills own the actual judgment and scope.

An `accepted` PISCT implementation-review verdict maps to CLEAN for its reviewed
scope, including when advisory suggestions remain. Record CLEAN when the issue's
required reviews are satisfied and retain it at completion. Use NOT CLEAN when
blocking findings or insufficient review evidence must accompany a handoff;
routine review corrections can remain local until resolved. Preserve unresolved
findings and limitations with enough context for the next action. Review applies
only to the issue's scope, independently of its resolved or unplanned outcome.

Use `$pisct:planning` only when explicitly requested. A local Markdown plan is
useful when the issue needs design detail; it is not a prerequisite for every
issue. Use one plan home and link it when needed. Accepted product decisions
must remain discoverable in the owning issue or tracked source, not solely in a
disposable draft. Do not create a separate plan/status lifecycle.

Keep owned drafts, captured output, and API inputs in ignored `tmp/`, using a
topic subdirectory when needed. Honor a tool's required temporary location
without making a second copy. Stream routine successful output. Keep logs,
reports, or exact inputs only while a concrete consumer needs them; retain an
uncertain write's input until its outcome is reconciled. Preserve required
standards provenance and raw data that belongs in a source fixture, not command
transcripts. Remove only owned consumed files; never sweep shared files by age,
prefix, or extension, or purge history as incidental cleanup.

## Pause And Resume

On pause, stop or account for owned workers/processes through PISCT process
handling. Leave enough current information to continue: the relevant issue,
blocker or next action, outstanding checks, consequential decisions, and active
ownership or needed working material. Preserve unfinished edits and inputs;
administration does not authorize stashing, discarding, or resuming code.

On authorized resume, inspect Git, the owning issue and relevant relationships,
local working material, and active ownership. Reconcile changed source before
reusing evidence. Restore only material the next action needs. At completion,
record the scoped outcome and applicable Review state, verify the Project result,
and remove owned consumed temporary material. Report the result and remaining
work briefly. Publication and unrelated cleanup retain their own authority.
