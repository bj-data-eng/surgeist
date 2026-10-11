# HTML autocomplete WebAuthn edge disposition

Selected 2026-10-11 for Surgeist #1133. This is independent analysis and source metadata, not redistributed browser source or issue prose.

## Sources

- Normative HTML revision: `877d6146487f15b15bc8dd0e3c625063a37b456a`, [autofill processing model](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autofill-processing-model). Already retained in `html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md`, processing at line 11827 onward; full original HTML SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
- [Current HTML processing model](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#autofill-processing-model), checked 2026-10-11, displayed last update 2026-10-09: same relevant branches, no repair.
- [WHATWG HTML #13028](https://github.com/whatwg/html/issues/13028), opened 2026-10-03, open as checked 2026-10-11: reports the retained-field defect and proposes changing the preceding detail field and casing. A proposal is not an adopted correction.
- [WHATWG HTML #8231](https://github.com/whatwg/html/issues/8231) and merged [PR #8232](https://github.com/whatwg/html/pull/8232), merge `469b353b255959757695d6233ee76c6faf195304`: historical category/IDL-value correction, not a correction of field identity.
- Merged [PR #13029](https://github.com/whatwg/html/pull/13029), merge `47c10fe92e437cb44af7a3daf876bc64bea7f8e0`: redundant token-limit removal, not resolution of #13028.
- [Pinned WebKit Autofill.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/Autofill.cpp#L211), complete file SHA-256 `a9718e995bd0ea90a46fe68120453a3d9bf3eb3ebcd8572009864390ed733c15`. Lines 211, 237–267 retain the final field while processing the preceding token and accept bare webauthn. This corroborates only the two lowercase edge results; WebKit folds tokens and omits scope/hints, so its whole parser is not selected. Browser file stays private research material; this reference includes no source excerpts.

## Selected disposition

Implement literal pinned processing for both edges. No source correction is selected. Keep a documented known upstream defect rather than silently changing the pinned language edition.

| Authored value | Author grammar | Processed field | Credential type | IDL value | Hint/scope |
| --- | --- | --- | --- | --- | --- |
| `webauthn` | invalid: missing detail field | `webauthn` | `webauthn` | `webauthn` | empty |
| `username webauthn` | valid on an appropriate input/textarea | `webauthn` | `webauthn` | `username webauthn` | empty |

Author conformance does not gate processing recovery. Native detail construction still requires a real detail field and does not construct bare webauthn as valid detail syntax.

The final credential token remains the field variable. Do not silently reassign it after stepping back. The report must represent processed credential field identity separately from the authored detail-field vocabulary. Preserve exact source token text for field/IDL output; category recognition is ASCII-insensitive. For `USERNAME WEBAUTHN`, field is `WEBAUTHN`, credential type is canonical `webauthn`, IDL value is `USERNAME WEBAUTHN`. Section and contact/mode formatting follows their own source steps; no blanket lowercase transform.

Test these outcomes and the category reset with `home tel webauthn`: field webauthn; credential webauthn; hint/scope home; IDL home tel webauthn. Invalid `on webauthn`, `webauthn webauthn` and `username webauthn extra` use the existing default path. Do not force form-owner lookup on the successful credential branches.

These choices resolve the two implementation questions. They do not certify this as desirable autofill classification or implement credential mediation. Revisit the bounded rule when a new source correction is explicitly selected; do not add a second compatibility mode now.
