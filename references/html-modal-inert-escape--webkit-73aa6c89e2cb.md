# Modal inertness escape: pinned WebKit witness

Source metadata and a bounded determination for Surgeist #1134, prepared 2026-10-11. HTML 877d6146487f §6.3.1–6.3.2 remains normative. Current HTML still states the topmost blocking subject rule and a general modal-dialog example; this witness resolves only whether a lower modal dialog independently resets inherited attribute inertness.

Selected determination: the blocking/topmost active modal subject is the escape boundary. A different dialog does not gain that boundary merely from its is-modal flag. For topmost A > inert container > lower-modal B > child, B and child remain inert. If B becomes the blocker, its own inert attribute still wins; otherwise inherited inertness from above B is cleared. No general host-provided escape bit is introduced.

WebKit is an implementation witness, not the HTML language pin. Its style storage, fullscreen behavior and active-dialog lifecycle are not ingested or assigned to HTML by this determination. This public reference retains source metadata and the independent determination only. Raw code excerpts and original headers remain private research: their documentation-excerpt redistribution basis has not been established from the supplied LGPL terms. No code or license-header excerpts are copied below.

Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.

The external files identify GNU Library General Public License version 2-or-later terms. The repository retains the [version 2 license](../licenses/webkit/LICENSE-LGPL-2.txt) and its existing [WebKit notice](../licenses/webkit/NOTICE.md). This metadata record copies neither the headers nor the selected code ranges; those remain at the linked immutable source URLs.

## Source metadata

- [StyleAdjuster.cpp, lines 775–802](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/StyleAdjuster.cpp#L775-L802): complete original SHA-256 `02fb43d141253144d2c57aa6a057b140c9309b4cc8d8786313e862c8267ae7a3`.
- [Document.cpp, lines 11250–11258](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Document.cpp#L11250-L11258): complete original SHA-256 `c3f64c3d98f23601ffa74b2e63b82f30aaeb4035bb537c657475a8c00095c55c`.

These are additional metadata for two existing catalog identities. Other retained representations remain bounded by their own original scope; this record does not expand their source-copy coverage. The full original files and their notices identify LGPL version 2-or-later material; this record makes no additional excerpt-permission claim. Source interpretation and public redistribution are separate determinations.
