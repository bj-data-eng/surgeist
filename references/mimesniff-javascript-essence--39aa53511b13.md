# MIME Sniffing: JavaScript essence matching

Source: [immutable snapshot](https://mimesniff.spec.whatwg.org/commit-snapshots/39aa53511b13953d84fef8d4131d6f61d0ccbde6/), revision `39aa53511b13953d84fef8d4131d6f61d0ccbde6`, displayed date 17 July 2026; retrieved 2026-10-11. Complete HTML SHA-256 `11caac49505995bcef51153a827c6fe96464111c8914e2c399d9e8fe9a467cd7`. Bounded §4.6 definitions, all 16 strings and attached hook note.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). See [WHATWG license](../licenses/whatwg/LICENSE). Extraction and Markdown conversion by Surgeist, 2026-10-11, Pandoc 3.1.11.1; links made absolute, no scripts executed. No semantic changes or endorsement. Code-incorporation BSD permission does not replace the prose license.

<div>

A <span id="javascript-mime-type" class="dfn dfn-paneled" dfn-type="dfn" export="">JavaScript MIME type</span> is any <a href="https://mimesniff.spec.whatwg.org/commit-snapshots/39aa53511b13953d84fef8d4131d6f61d0ccbde6/#mime-type" id="ref-for-mime-type②①" data-link-type="dfn">MIME type</a> whose <a href="https://mimesniff.spec.whatwg.org/commit-snapshots/39aa53511b13953d84fef8d4131d6f61d0ccbde6/#mime-type-essence" id="ref-for-mime-type-essence⑨" data-link-type="dfn">essence</a> is one of the following:

- `application/ecmascript`
- `application/javascript`
- `application/x-ecmascript`
- `application/x-javascript`
- `text/ecmascript`
- `text/javascript`
- `text/javascript1.0`
- `text/javascript1.1`
- `text/javascript1.2`
- `text/javascript1.3`
- `text/javascript1.4`
- `text/javascript1.5`
- `text/jscript`
- `text/livescript`
- `text/x-ecmascript`
- `text/x-javascript`

A <a href="https://infra.spec.whatwg.org/#string" id="ref-for-string①" data-link-type="dfn">string</a> is a <span id="javascript-mime-type-essence-match" class="dfn dfn-paneled" dfn-type="dfn" export="">JavaScript MIME type essence match</span> if it is an <a href="https://infra.spec.whatwg.org/#ascii-case-insensitive" id="ref-for-ascii-case-insensitive" data-link-type="dfn">ASCII case-insensitive</a> match for one of the <a href="https://mimesniff.spec.whatwg.org/commit-snapshots/39aa53511b13953d84fef8d4131d6f61d0ccbde6/#javascript-mime-type" id="ref-for-javascript-mime-type①" data-link-type="dfn">JavaScript MIME type</a> essence strings.

This hook is used by the <a href="https://html.spec.whatwg.org/multipage/scripting.html#attr-script-type" id="ref-for-attr-script-type" data-link-type="element-sub"><code>type</code></a> attribute of <a href="https://html.spec.whatwg.org/multipage/scripting.html#script" id="ref-for-script" data-link-type="element"><code>script</code></a> elements. <a href="https://mimesniff.spec.whatwg.org/commit-snapshots/39aa53511b13953d84fef8d4131d6f61d0ccbde6/#biblio-html" data-link-type="biblio" title="HTML Standard">[HTML]</a>

</div>
