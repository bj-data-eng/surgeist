# Fetch: script-like destinations

Source: [immutable Fetch Standard snapshot](https://fetch.spec.whatwg.org/commit-snapshots/e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d/#request-destination-script-like), revision `e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d`, retrieved 2026-10-12. Complete rendered snapshot: 1,932,351 bytes; SHA-256 `639d437c000a870bf161804a92f9e0ce1b22dae61f39d8975a7b8429302534eb`. Complete original retained in the development source captures as `fetch--e9460d1b22ea.html`. Selected definition and adjacent warning: original lines 2820–2828, section 2.2.5 Requests.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose: [CC BY 4.0](../licenses/whatwg/LICENSE); [WHATWG license](https://github.com/whatwg/fetch/blob/e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d/LICENSE). Bounded extraction and readable Markdown conversion by Surgeist, 2026-10-12. Formatting and links changed; no semantic changes or endorsement. The selected clauses contain no tables or diagrams.

## Selected definition

<a id="request-destination-script-like"></a>

A request's destination is **script-like** if it is `audioworklet`, `paintworklet`, `script`, `serviceworker`, `sharedworker`, or `worker`.

Warning: Algorithms that use script-like should also consider `xslt` as that too can cause script execution. It is not included in the list as it is not always relevant and might require different behavior.

## Surgeist selection

This definition supplies only the external finite vocabulary required by HTML revision `877d6146487f15b15bc8dd0e3c625063a37b456a`, `#module-preload-destination` (original line 19192). HTML itself adds `json`, `style`, and `text`: nine module-preload destinations in total. `xslt` remains excluded; the warning does not amend the finite definition and HTML does not add it.

HTML `#attr-link-as` (original lines 10665–10689) owns enumeration, context validity and absent/invalid handling. Its keyword set is the union of six preload destinations (`fetch`, `font`, `image`, `script`, `style`, `track`) and the nine module-preload destinations: thirteen distinct states. Match the complete value ASCII case-insensitively without trimming. Missing, empty and unrecognized values have no state, with raw presence and author diagnostics retained separately. A recognized preload-only state such as `image` remains that state under modulepreload and fails its destination membership check; do not turn it into no state or `script`.

HTML's actual modulepreload algorithm (original lines 19218–19222) selects `script` when the attribute is in no state, including invalid raw values; this processing recovery does not make invalid author syntax conforming. Do not change the intrinsic enumeration's no-state result into a default state. Preload context and modulepreload context retain their separate author requirements.

This is a bounded external-definition selection for #1133 A010, not a new HTML edition or adoption of Fetch networking, resource loading, service-worker execution, module evaluation, or JavaScript bindings. The intrinsic owner stores and interprets the vocabulary even when execution is unsupported.
