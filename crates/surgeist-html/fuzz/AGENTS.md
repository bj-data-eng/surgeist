# HTML fuzz package guide

Apply the repository, HTML and shared fuzz guides. This verification package owns
Unicode cases, codecs, captured fixtures, expectations and reductions. Use public
HTML APIs; no private parser copy or fuzz-only production export. Framework owns
engines, worker supervision, artifacts and same-failure reduction.

Read README.md and fuzz.toml for the exact revisioned scope and limits. Preserve
exact UTF16 and chronological typed diagnostics; source offsets need not increase.
New native behavior is not authorized by a fuzz counterexample: retain exact case,
context/limits/failure and return it to the coordinator before a production fix.

Run focused tests in default/property/coverage/all-feature modes, strict all-target
all-feature Clippy and formatting through bounded pisct run with offline/locked
Cargo and one build job. No live campaign against dirty source or tool acquisition.
