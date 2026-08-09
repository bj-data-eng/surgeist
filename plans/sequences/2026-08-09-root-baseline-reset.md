# Root Baseline Reset Implementation Sequence

## Authority

- Specification: `plans/specs/2026-08-09-root-baseline-reset.md`
- Reviewed specification revision:
  `14a8ead7cf6d2c70deb93be645c7372ae99aed1c3e626daaac65160624fe60d4`
- Review verdict: clean
- User objective: pin every configured submodule to the latest authoritative
  remote `main` tip and restore root to a compileable product-integration
  baseline.

## Ordered Cycles

### C01 — Promote Latest Leaves And Restore Root Baseline

- Owning repository: root `surgeist`.
- Specification sections: S1–S11.
- Bounded outcome: the root cleanup, all 14 exact latest-remote gitlinks,
  repaired root manifest and facade, reconciled README, refreshed root-owned API
  artifacts, and root-package compatibility evidence form one published root
  integration candidate.
- Prerequisites: all candidates in specification S4 are locally fetchable from
  their `.gitmodules` URLs; each equaled the authoritative remote-`main` tip when
  observed on 2026-08-09 and descends from the current pin; the user-authorized dirty
  deletion inventory remains exactly S4; root local `main` remains at the
  recorded cycle base and equals `origin/main`.
- Entry state: root cannot load Cargo metadata because surviving manifest wiring
  names deleted local targets; all submodules remain at the old pins; the exact
  candidate objects are retained under cycle-owned provenance refs; the
  specification is clean at the revision above.
- Exit evidence: root source, manifest, docs, and generated artifacts match the
  specified baseline; all selected gitlinks equal the exact S4 candidates and
  those commits remain available from the 14 authoritative repositories; the
  root-only workspace and retained feature matrix pass the configured verification
  gates; task, root-integration, and holistic reviews are clean; root `main` is
  published and remotely read back.
- Cross-repository handoff: none is created or consumed for this scoped reset,
  per specification S7 and the user's explicit waiver. The published root
  candidate and its exact gitlinks are the completion record.

No later cycle is planned for this initiative.
