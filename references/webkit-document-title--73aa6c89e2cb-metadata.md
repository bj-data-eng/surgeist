# WebKit evidence: absent SVG document title

Inspected 2026-10-12 at revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.
This is a metadata and behavioral-summary reference, not a source-code excerpt
or a complete converted source representation.

- [Document.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Document.cpp): original SHA-256 `c3f64c3d98f23601ffa74b2e63b82f30aaeb4035bb537c657475a8c00095c55c`.
- [Document.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Document.h): original SHA-256 `d834dcea6916301c50febc380c74c28f0162c72c75b510b34d4b3feb409fa542`.

Original-file line numbers: Document.cpp 2462–2475 clears the title state when
there is no title element; 2524–2527 selects direct SVG title children;
2550–2561 chooses the SVG selection path for an SVG document element and
updates the title state. Document.h 1203 exposes the stored title string.
This supports empty readback when no direct SVG title exists. It does not
select WebKit's other text extraction, canonicalization, lifecycle or scheduling
behavior for Surgeist.

Selected narrow correction to HTML877d document.title getter step 1: when a
complete captured SVG svg document element has no direct SVG title child,
use empty value with no source element. Unknown required children remain
missing information, not known absence. An existing empty title retains its
source identity. Do not search deeper descendants or fall back to HTML title.

The complete originals remain in ignored private research material. They have
GNU Library GPL notices. This public reference retains no original source
excerpts or implementation bodies and grants no permission to redistribute
those originals. The [edition determination](SOURCE-EDITIONS.md#absent-svg-document-title)
and [source catalog](SOURCE-CATALOG.md) record this bounded selection.
