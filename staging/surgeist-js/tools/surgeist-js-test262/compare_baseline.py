#!/usr/bin/env python3
"""Compare complete Test262 case inventories and guard prior passes."""

import argparse
from collections import Counter
import gzip
import json
from pathlib import Path


def load_report(path):
    with (gzip.open(path, "rt", encoding="utf-8") if path.suffix == ".gz"
          else path.open(encoding="utf-8")) as source:
        report = json.load(source)

    cases = {}

    def collect(suite, parent):
        directory = parent / suite["n"]
        for test in suite.get("t", []):
            case = (directory / test["n"]).as_posix()
            outcome = test["r"]
            if case in cases or outcome not in {"O", "I", "F", "P"}:
                raise ValueError(f"duplicate case or unrecognized outcome for {case}")
            cases[case] = (outcome, test["v"])
        for child in suite.get("s", []):
            collect(child, directory)

    collect(report["r"], Path())
    counts = Counter(outcome for outcome, _ in cases.values())
    expected = {"t": len(cases), "o": counts["O"], "i": counts["I"], "p": counts["P"]}
    if report["r"]["a"] != expected or not cases:
        raise ValueError(f"inconsistent Test262 report totals in {path}")
    return report["u"], cases, counts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--exact", action="store_true", help="Require identical case outcomes.")
    args = parser.parse_args()

    pinned, baseline, _ = load_report(args.baseline)
    revision, candidate, counts = load_report(args.candidate)
    if revision != pinned:
        parser.error(f"candidate Test262 revision {revision} differs from {pinned}")

    missing = baseline.keys() - candidate.keys()
    unexpected = candidate.keys() - baseline.keys()
    regressions = []
    for case in sorted(baseline.keys() & candidate.keys()):
        old_outcome, old_edition = baseline[case]
        outcome, edition = candidate[case]
        if edition != old_edition or (
            outcome != old_outcome
            and (args.exact or old_outcome == "O" or outcome in {"I", "P"})
        ):
            regressions.append((case, old_outcome, outcome, old_edition, edition))

    for case in sorted(missing):
        print(f"Missing {case}")
    for case in sorted(unexpected):
        print(f"Unexpected {case}")
    for case, old_outcome, outcome, old_edition, edition in regressions:
        print(f"Regressed {case} ({old_outcome}/{old_edition} → {outcome}/{edition})")
    total = len(candidate)
    passed = counts["O"]
    print(f"{passed}/{total} passed ({passed / total:.2%}), "
          f"{counts['I']} ignored, {counts['F']} failed, {counts['P']} panicked")
    if passed * 100 <= total * 90:
        print("Conformance must exceed 90% of the pinned suite, including ignored cases.")
        return 1
    return int(bool(missing or unexpected or regressions))


if __name__ == "__main__":
    raise SystemExit(main())
