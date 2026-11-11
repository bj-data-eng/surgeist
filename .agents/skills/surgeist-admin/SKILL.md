---
name: surgeist-admin
description: Coordinate Surgeist issues and labels, pull requests, local working material, and pause/resume handoffs. Use for repository administration during authorized work; PISCT owns implementation and review methods.
---

# Surgeist Administration

Use [GitHub issues](https://github.com/bj-data-eng/surgeist/issues), their labels,
state and native relationships as the progress record for completed work and
actionable future work in `bj-data-eng/surgeist`. The coordinator owns issue and
label updates within the
user's authority. Honor execution pauses under
[Pause And Resume](#pause-and-resume).
Use PISCT's focused skills for engineering and review.
Use the latest stable Rust release and keep the workspace's declared Rust version aligned with it.

## Work And Ownership

An issue states the outcome, acceptance criteria, ownership boundaries, and
relevant sources. State engineering decisions and requirements directly in issue
bodies and comments; do not attribute them to "the user" or narrate the chat
instructions that established them. Use module → crate contributions → bounded
properties or tasks where that grouping fits the work. Own shared capabilities
once and link dependent work to that owner. Native parent-child relationships
describe scope; blocking dependencies describe execution order. Keep both acyclic. Completing a
leaf does not certify the enclosing contribution or module.

Issue decomposition and relationships describe the current working organization.
Within existing authority, the coordinator may combine, split, move, or reorder
work and remove obsolete blocking edges when implementation reveals a simpler
route. Preserve underlying requirements, coverage, ownership, prerequisites,
and explicit human constraints; reconcile active assignments and
update affected issues and relationships. Reuse review results only for the
requirements and source they actually cover; regrouping alone requires no new
review, while uncovered work still needs its applicable evidence.

Read the relevant issue, its parent, and actual prerequisites before expanding
the search. Include closed issues when reconstructing progress; an empty
filtered result does not establish absence. Reference material
supports acceptance criteria without automatically becoming another issue.

Keep requirement coverage and consequential decisions with their owning issue
or tracked source. Do not build a parallel progress database in local files or
comments.

## Issue Labels And State

Surgeist tracks work in issues and lands changes on `main` through
[pull requests](#pull-requests). Labels are authoritative for crate names,
ownership, accepted review and priority. Read current labels before creating or
changing them; preserve unrelated labels and resolve conflicting assignments
from the issue's actual scope rather than guessing.

| Dimension | Labels and assignment |
| --- | --- |
| Crates | Exact crate-name labels, including `surgeist` for the facade. Tag crates with actual work in scope, including removal, rather than every dependency mentioned. Preserve legitimate historical crate labels on closed issues. |
| Ownership | One of `crate-owned`, `cross-crate`, or `root-owned`. A consumer's direct adapter can be crate-owned; work requiring changes across owners is cross-crate. Workspace wiring and shared root tooling are root-owned unless the scope also spans other owners. |
| Review | `review-clean` records accepted required review for the issue's scope; follow [Review And Working Material](#review-and-working-material). |
| Priority | At most one of `P0`, `P1`, `P2`, `P3`, highest to lowest. Assign only an established priority; leave unknown priority unset. Replace the old priority label when an authorized priority decision changes. |

Within authorized issue work, creating a missing label for a new implemented or
planned crate is authorized. Establish its exact name from the package manifest
or the accepted crate scope, use red `#b60205`, and give it a concise one-line
description of its responsibility. This authorizes the label, not implementation
of the crate or creation of unrelated label categories.

The coordinator closes completed work as **completed** once the authorized scope
and required reviews are satisfied. Close explicitly excluded work as **not
planned** and apply `not-planned`; closure alone does not assert implementation
or accepted review. Existing API creation must name the Surgeist repository
explicitly; read identifiers rather than guessing them.

Trust successful issue and label updates without routine post-update readback.
Inspect state when an update fails, its outcome is uncertain, or work resumes.
The Architecture Coordinator thread is the current contact for architecture
questions. Workflow settings remain user-owned; changing them requires explicit
authorization.

## Pull Requests

A pull request publishes one cohesive outcome for integration into `main`. It
is not a progress record, review record, or decision home. Opening, updating,
and merging one require publication authority for that branch and target. Pull
requests leave PISCT's commit, publication, and review requirements unchanged.

Unless the assignment names a branch, branch from current `origin/main` as
`branch/<topic>`, where the topic is a short kebab-case name for the outcome. A
pull request may carry several logical commits that share one outcome and
verification; do not batch unrelated outcomes. By default one pull request
completes one issue. Several issues share one only when they are small and
change the same code for the same reason. PISCT's testing and commit skills own
the commits inside it.

Create a pull request only when it is ready to be merged: its outcome is
complete, implemented, independently reviewed with the findings fixed on the
branch, and verified against current `origin/main`. Complete implementation,
required independent reviews, and local verification before creation. Keep
implementation testing and independent engineering reviews local. GitHub's
required hygiene checks supplement that evidence. Only checks that require an
existing pull request may remain pending. This rule also applies to draft pull
requests. With auto-merge enabled, assume the pull request can merge as soon as
required platform checks pass; do not rely on time
after creation to finish work, fix findings, or decide whether it should land.
Do not open one per commit or step. If an issue is too large for one reviewable
pull request, split it into parts that are each a complete, independently
verified outcome. Earlier parts use `Refs #N`, the last part uses `Closes #N`,
and the split is recorded on the issue.

The title supplies the merge-commit subject by default, with GitHub appending
the pull request number. Validate the title against the repository's
commit-message requirements before creating the pull request.
The body states the change, public impact, and verification evidence. Keep
decisions and judgement calls on the owning issue; the body may link them.

Pull request text, commit messages and tracked files name no development
tools and carry no "Generated with" lines or co-author trailers. Tracked files
cite no branch commit SHAs, because a rebase replaces them; describe a
measurement by its method and inputs, or cite a commit already on `main`.

A closing keyword in the body or a `Closes:` commit trailer closes its issue
when the change reaches `main`. Use one
only where closure authority covers that issue and its required review is
satisfied, with `review-clean` applied by the coordinator before creating the
pull request. Give each such issue its own `Closes #N` line in the body, because
GitHub links only the first issue of a list. Use `Refs #N` for every other
issue the pull request concerns.

`main` is published history: never rebase, amend, or force-push it. A
repository ruleset rejects non-fast-forward pushes to the default branch.
Until a branch lands, its history may be rewritten, including after it is
pushed, when that makes the landed history truthful, for example to split a
commit whose message misdescribes it. This repository rule takes precedence
over PISCT's general rule against rewriting published history, for branches
only. Bring a branch up to date by rebasing it onto `origin/main` and
force-pushing it; never merge `origin/main` into a branch. Each landing then
forms one merge bubble on the current tip of `main`, with no crossing lines.
After a rebase, re-run the verification the rebase invalidates.

The default-branch ruleset requires a pull request, allows merge commits only,
and requires the branch to be up to date with `main` with its required status
checks passing. The required checks are "Branch follows branch/<topic>",
"One closing keyword per issue", and "No tool names in text, commits or files".
The ruleset rejects direct pushes to `main`. Creating a ready pull request
starts the required checks. Enable auto-merge for that pull request with the
merge-commit method only after
confirming its current branch head is ready to land, with all required local
verification and independent reviews complete. Repository-level availability
alone does not enable auto-merge for each pull request.
Once required checks pass, GitHub merges using the pull request title and
deletes the remote head branch. No custom merge commit or manual merge command
is required.
Never squash or rebase-merge. Monitor the checks and automatic landing through
completion. Disable auto-merge before changing the branch head or when a failure
invalidates readiness. Correct the branch, repeat affected local verification
and independent reviews, and enable auto-merge again only when the new head is
ready to land.
If `main` moves first and the branch becomes out of date, rebase it onto current
`origin/main`, re-run invalidated verification, and update the pull request.
After the merge, read back `main`: its first parent is the previous `main`, its
second parent is the verified branch head, its tree equals the branch tree, and
its subject uses the pull request title with the pull request number suffix.
Confirm the remote head branch was deleted; it stays restorable from the pull
request. Remove local branches, worktrees, and Cargo targets only with cleanup
authority.

## Review And Working Material

Review approval is separate from issue closure. The coordinator alone maintains
`review-clean`; reviewers remain read-only against GitHub. Keep detailed
verdicts and supporting evidence local while they serve an active review,
diagnosis, or handoff. No linked review record or compulsory report file is
required in GitHub. PISCT's review skills own the actual judgment and scope.

An `accepted` PISCT implementation-review verdict supports `review-clean` for its
reviewed scope, including when advisory suggestions remain. Apply the label only
when all required reviews are satisfied and retain it at completion. Remove it
when blocking findings or material scope or implementation changes invalidate
that acceptance; unrelated changes do not invalidate it. Absence of the label
conveys no accepted review. Routine corrections can remain local until resolved;
preserve unresolved findings and limitations with enough context for the next
action. Review applies only to the issue's scope, independently of its resolved
or unplanned outcome.

Use `$pisct:planning` only when explicitly requested. A local Markdown plan is
useful when the issue needs design detail; it is not a prerequisite for every
issue. Use one plan home and link it when needed. Accepted product decisions
must remain discoverable in the owning issue or tracked source, not solely in a
disposable draft. Do not create a separate plan/status lifecycle.

Keep owned drafts, captured output, and API inputs in ignored `tmp/`, using a
topic or issue subdirectory for related work. Separate current authoring inputs
from historical review evidence, reference captures, and build output. Honor a
tool's required temporary location without making a second copy. Stream routine
successful output. Retain logs, reports, and exact inputs only while a concrete
consumer needs them; retain an uncertain write's input until its outcome is
reconciled. Preserve required standards provenance and raw source-fixture data.

Prefer the existing reusable Cargo target when the verification context permits.
Use a separate target for a concrete isolation or tool requirement, and identify
the retained cache in the current handoff. Build output is reconstructible cache,
not review evidence. With cleanup authority, remove obsolete owned targets after
their verification or packaging consumers finish. A target outside a worktree
survives archiving that worktree; account for it explicitly. Preserve useful
shared caches and respect active process ownership.

At verification completion and pause points, review which owned caches still
have a concrete upcoming consumer, including worktree and external targets.
Promptly remove unused caches within cleanup authority; retain useful caches
regardless of size. Never delete active or other-owned caches.

After publication or a settled operation, retire consumed owned scratch inputs,
duplicate source copies, and intermediate captures within cleanup authority.
Before relying on Git recovery, establish the source commit/blob and account for
local differences. A historical preservation hash records a checkpoint; it does
not make every guarded file a permanent retention requirement. Keep final scoped
review evidence needed for later composition, unresolved findings, and unfinished
drafts. Do not rewrite a historical verdict to describe current state.

When reorganizing retained evidence, preserve immutable bytes and verify moved or
losslessly compressed content. Update active links and consumers; record a small
old-to-new path mapping when historical records must retain their original paths.
Do not introduce a second progress registry or routine per-file manifests merely
to organize storage. Shared monitoring, reporting, configuration, and reference
material may have other consumers. Remove only owned consumed files; never sweep
shared files by age, prefix, or extension, or purge history as incidental cleanup.

## Pause And Resume

Execution pauses and resume instructions are local session state, separate from
the issue's product progress record. Keep them only in local handoffs under
ignored `tmp/`. Never publish execution pause notices or paused-work status in
GitHub issues, comments, labels, or other GitHub records. Do not change
issue state to represent an execution pause. These rules do not
affect technical requirements for paused animations or other modeled states.

On pause, stop or account for owned workers/processes through PISCT process
handling. Keep one current local handoff at a stable entry point. Replace
superseded checkpoint wording rather than appending a running transcript. Move a
completed work narrative into clearly historical material only while a concrete
consumer still needs it; otherwise retire it within cleanup authority. Leave enough
current information to continue: the relevant issue,
blocker or next action, outstanding checks, consequential decisions, and active
ownership or needed working material. Preserve unfinished edits and inputs;
administration does not authorize stashing, discarding, or resuming code.

On authorized resume, inspect Git, the owning issue and relevant relationships,
local working material, and active ownership. Reconcile changed source before
reusing evidence. Restore only material the next action needs. At completion,
record the scoped outcome and applicable review label, and remove owned consumed
temporary material. Report the result and remaining work briefly. Publication
and unrelated cleanup retain their own authority.
