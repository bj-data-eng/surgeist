# CSSWG marker, native pseudo identity and named-support witnesses

This bounded evidence record accompanies Surgeist planning issues #378, #379 and #648. It is not a specification. Retrieval: 2026-10-11. The original comments belong to the official `w3c/csswg-drafts` repository. Comment IDs are stable locators, not immutable content; the exact captured JSON and UTF-8 body hashes identify the inspected text. Exact response bytes remain in local working evidence under `tmp/work/dom-style-contracts/additional-selectors/`.

Four short accepted-resolution excerpts are retained below. The fifth and sixth sources contribute only metadata and attributed findings; their full prose is not reproduced. These are bounded discussion witnesses, not full document conversions. Original proceedings remain upstream; no license beyond the source's applicable terms is asserted for GitHub discussion material.

| Exact source | Created / last updated | Captured JSON SHA-256 | Comment body SHA-256 | Retained material |
| --- | --- | --- | --- | --- |
| [CSSWG #4506 comment 1403998621](https://github.com/w3c/csswg-drafts/issues/4506#issuecomment-1403998621) | `2023-01-25T17:43:43Z` / `2023-01-25T17:43:43Z` | `8e6936c3adab0f54fff513fc52f3aaa569d4783b44c24299ecb35a0b433e5b25` | `bc5d66c720508f99090130bdb2871eee82972ed76463dc6f24e2c440a43fde4d` | short resolution excerpt |
| [CSSWG #3607 comment 467136978](https://github.com/w3c/csswg-drafts/issues/3607#issuecomment-467136978) | `2019-02-25T19:01:08Z` / `2019-02-25T19:01:08Z` | `3661f2f86f10271034529e6eec04fb27ad43672022703b30b862c230ca2e5212` | `0ce5b1aa5fb433afed26d4af2973e8bc7cfe7ffc8bce22d6482d2001d5783125` | short resolution excerpt |
| [CSSWG #3603 comment 484156093](https://github.com/w3c/csswg-drafts/issues/3603#issuecomment-484156093) | `2019-04-17T16:09:16Z` / `2019-04-17T16:09:16Z` | `7107b7921c36e882925ad4de5669708033274b6942a462e05b21cb3335cbe6a5` | `ae8d7153bd0d37fd1229bc42cb307d2196095bed35f411efd3152b0a55bf1cfb` | short resolution excerpt |
| [CSSWG #12159 comment 3005238286](https://github.com/w3c/csswg-drafts/issues/12159#issuecomment-3005238286) | `2025-06-25T15:39:01Z` / `2025-06-25T15:39:01Z` | `c10f63cf6a5239168b1bd2e95668178f33190597a89a97e81091962b592c7e43` | `23a49cab1bbe1b16daef26ae937e90943806a61815ca60c1f4a3cb7ac0a7cd93` | short resolution excerpt |
| [CSSWG #12622 comment 3497547041](https://github.com/w3c/csswg-drafts/issues/12622#issuecomment-3497547041) | `2025-11-06T14:35:11Z` / `2025-11-06T14:35:11Z` | `22682eba23196b71d15073f56f7cc1222b55b3905de2dae63dfab58ca21025c5` | `752f5d8a1f0a1d14a810edd235e3870eaecade850ce25727311a961e47a291c1` | metadata only |
| [CSSWG #12973 comment 3418874961](https://github.com/w3c/csswg-drafts/issues/12973#issuecomment-3418874961) | `2025-10-18T22:15:06Z` / `2025-10-18T22:15:06Z` | `ca41c46b0be3536f387fe68f62e0319626e75462cb2ef92dbc4f7748c8dabdfb` | `fa8adb1d480a3592027961106e9211793d6b422330d5aec540e27324f067d122` | metadata only |

Combined identity: SHA-256 `b321c7ac6475e74792718fa93915de18b3d0b4bc201860a60f5e6ceca8538420`, computed over the six exact response byte streams in table order.

## <a id="marker-first-line"></a>Marker and first-line

CSSWG #4506 remains open for specification editing, but its 25 January 2023 meeting comment contains an accepted resolution:

> RESOLVED: ::first-line applies to markers when `list-style-position` is `inside` but excludes them when it’s `outside`

Select this decision over the stale discussion warning in [current CSS Pseudo4 §4.2](selectors-execution-css-pseudo-4--editor-capture-20261010--46f88407070a.md#marker-pseudo). The first-line formatting and fragment inheritance rules continue to apply; first-letter continues to ignore markers. This resolves whether marker content participates, without changing the list/marker applicability rules.

## <a id="pseudo-identity-nullness"></a>Pseudo lookup identity and nullness

CSSWG #3607 and #3603 are closed. Their accepted resolutions are:

> RESOLVE: pseudo() must always return the same object. Note that object can be GC'd if this is unobservable.

> RESOLVED: Do a css parse on parameter for element.pseudo()

The more recent #12159 resolution is:

> RESOLVED: proposed: for a valid selector we will always return the CSSPseudoElement object

Read these together with the explicit [current Pseudo4 §7.2 algorithm](selectors-execution-css-pseudo-4--editor-capture-20261010--46f88407070a.md#window-interface): parse failure, element-backed pseudo types and non-tree-abiding pseudo types yield null; accepted tree-abiding non-element-backed types yield an object even if the originating element cannot currently generate it. Observable identity for the same origin and full pseudo selector remains stable. The old warning referring to #3607/#3603 is stale. The separate prospective existence API in #12158 does not change this lookup contract and is outside the planned native lookup surface.

## <a id="named-support-gap"></a>Named-support composition and activation gap

The [current Conditional5 §8 capture](additional-selectors-css-conditional-5--editor-capture-20261011--d1b76ceb44e1.md#supports-condition-rule) supplies non-rendering feature-test semantics, conjunction for two declarations in an example, subsequent references, and last-definition-in-document-order wins. It shows qualified-rule and at-rule candidates. It does not supply a full algorithm for recursive heterogeneous composition, recovered candidate evaluation, nested definition activation, registry scope, or invalidation. The inspected #12622 comment separately identifies context and named-query evaluation questions; it is discussion evidence rather than an adopted algorithm. The #12973 follow-up comment records the edit aligning duplicate precedence with other naming rules. Nothing in that inspected duplicate-rule discussion introduces a last-preceding restriction or a reference-position exception; the complete last-definition rule is preserved.

The required browser fallback was checked at existing pins, in order. [WebKit's complete recognition enum/table](webkit-cssom-wrapper-rules--73aa6c89e2cb.md#cssatruleid-h) at `73aa6c89e2cb77c46184a81aec944e4ab99d114d` has no supports-condition; the recorded newer `437139e30888c5e1b4c35f889f32e1d0c39d018e` tables are byte-identical. [Blink's complete enum/dispatch tables](blink-cssom-wrapper-gaps--7984f9d11800.md#css-at-rule-id-h) at `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2` likewise lack recognition. These inspected revisions cannot provide the missing implemented evaluator. Full exact source, hashes and file-specific legal notices already reside in those references and their linked licenses; no browser source is copied here.

The remaining semantics are therefore an explicitly labelled Surgeist policy under the user's authorized deterministic gap resolution, recorded in #648 and implemented/tested through #643. This finding does not reopen the adopted authored candidate model or CSSOM projection decisions. Reject rendering the body as styles and reject dropping unknown/recovered candidates to produce a spuriously true query. Reject claiming the policy is an implemented browser behavior or a CSSWG algorithm.
