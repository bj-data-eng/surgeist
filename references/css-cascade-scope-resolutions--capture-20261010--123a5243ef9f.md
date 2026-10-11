# CSSWG scope-cascade resolution witnesses

This reference records the precise resolution evidence used to adjudicate Surgeist planning issue #317. It is a bounded source witness, not a new specification. Retrieval: 2026-10-10. Original comments are from the CSSWG meeting bot in the official `w3c/csswg-drafts` repository. Comment URLs can retain their ID after an edit; the captured API response and body hashes identify the inspected text. The exact original response bytes and dated meeting HTML remain in local working evidence under `tmp/work/dom-style-contracts/selectors/`.

| Exact source | Source identity and inspected edit time | Captured JSON SHA-256 | UTF-8 comment-body SHA-256 |
| --- | --- | --- | --- |
| [resolution-6790.json](https://github.com/w3c/csswg-drafts/issues/6790#issuecomment-1479754855) | GitHub comment ID `1479754855`; created and last updated `2023-03-22T15:13:02Z` | `25a99309c07d50367c75be689b466737a2959e77702c9a1b3024fd074e7307c6` | `429d5f7c7747229de0516da1991943f01c88a9a7af840699e03c1ceb016caee8` |
| [resolution-10795.json](https://github.com/w3c/csswg-drafts/issues/10795#issuecomment-3200836319) | GitHub comment ID `3200836319`; created and last updated `2025-08-19T13:45:45Z` | `cab599c5e2f772570d5a86bf3635a58d50e5867140818af8d4870a0874796966` | `06c57a0d4312877c4d5084bfef45dd6e8582d01b86cb6051e3f21bf211bbe97a` |

The official [22 March 2023 dated minutes](https://www.w3.org/2023/03/22-css-minutes.html) corroborate the #6790 resolution. Captured HTML SHA-256: `17bc60ec006d63bfc624c669e914c7a7dd787a4183bebbe1cc3e879ada5ded7c`. This is a bounded evidence record; the full minutes are not copied here. Original W3C copyright and [permissive Software and Document License](../licenses/w3c/software-license-2023.txt) apply to the attributed proceedings. The combined capture identity is SHA-256 `123a5243ef9f6a1e38104e9e543a792c28006ab672f97aab66b9f47e89a0d608`, computed over the three retained files in the table/minutes order.

## <a id="specificity-priority"></a>Specificity priority

CSSWG #6790 is closed and records the 22 March 2023 accepted resolution:

> RESOLVED: cascade proximity is weaker than specificity

This selects the explicit order in [current Cascade 6 §3.1](css-cascade-scope-6--editor-capture-20261010--9f61dd29dd2a.md#cascade-sort): specificity, then scope proximity, then order of appearance, after the preceding cascade criteria tie. The sentence in [§3.5.1](css-cascade-scope-6--editor-capture-20261010--9f61dd29dd2a.md#scope-effects) saying proximity applies regardless of specificity conflicts with that sorting order and the accepted resolution. Retain the conflict as source history; use the accepted weak-proximity resolution for implementation. This is standards adjudication, not a browser compatibility exception.

## <a id="nested-proximity"></a>Nested proximity

CSSWG #10795 is closed and records the 19 August 2025 resolution:

> RESOLVED: Close no change for now, unless someone comes back with clear use cases + proposal

The retained outcome is the single proximity value from the innermost scope root specified in [Cascade 6 §3.5.5](css-cascade-scope-6--editor-capture-20261010--9f61dd29dd2a.md#scope-scope). Enclosing scopes still constrain which inner scope roots and subjects participate, but no outer-distance vector or scope-count tie-breaker is added. The draft's still-rendered issue link is not evidence that the proposal remains unresolved.

## Source selection boundary

Apply these captures to unstarted selector/cascade execution work. Preserve the prior 2024 Cascade 6 edition and all closed authored-CSS grammar history. No WebKit/Blink fallback is needed: the current standards plus explicit CSSWG resolutions establish both decisions.
