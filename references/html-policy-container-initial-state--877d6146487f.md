# HTML Standard: initial Document policy container

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-containers), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`. Extracted on 16 October 2026 from the retained complete HTML, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Source ranges: raw lines 7076–7078 (Document initial state) and 65813–65829 (policy-container items). This is a bounded excerpt, not the complete policy-container algorithms.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is [Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/), with the [exact WHATWG license](../licenses/whatwg/LICENSE). Source prose below is manually converted to Markdown with normalized whitespace, list boundaries and explicit source links. No semantic change or endorsement is implied.

## Document initial state

Each [Document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document) has a [policy container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-policy-container) (a policy container), initially a new policy container, which contains policies which apply to the Document.

## Policy-container items

A [policy container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container) is a struct containing policies that apply to a Document, a WorkerGlobalScope, or a WorkletGlobalScope. It has the following items:

- A [CSP list](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-csp-list), which is a CSP list. It is initially empty.
- An [embedder policy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-embedder-policy), which is an embedder policy. It is initially a new embedder policy.
- A [referrer policy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-referrer-policy), which is a referrer policy. It is initially the default referrer policy.
- An [integrity policy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-integrity-policy), which is an integrity policy, initially a new integrity policy.
- A [report-only integrity policy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-report-only-integrity-policy), which is an integrity policy, initially a new integrity policy.

## Selection and limits

The existing [pinned Referrer Policy vocabulary](referrer-policy-vocabulary--cc435b05ca4a.md) supplies the default policy value. Native Meta processing and its bounded capability decision are owned by issue [#1124](https://github.com/bj-data-eng/surgeist/issues/1124). This excerpt establishes initial state only; it does not import policy cloning, response processing, CSP parsing or enforcement, embedder/integrity processing, or fetched/parsed Document history. Empty CSP is known only for an actual new container whose admitted operations preserve it.
