# WebKit SVG script clone state

Inspected 2026-10-12 at revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.

Source: https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/svg/SVGScriptElement.cpp

Original SHA-256: `b36a783f9ef0034c1f89b0989874356aa908f5df275bb264c633085148b49906`.

Original lines 117–120: the SVG script clone constructor receives false for parser insertion and the source already-started value. This is evidence for preserving the at-most-once processing bit on the clone, not adoption of WebKit shared HTML/SVG preparation machinery.

Metadata and behavioral summary only; no source excerpt. The original remains private research with GNU Library GPL notices. The [source-edition determination](SOURCE-EDITIONS.md#offline-disabled-svg-script-and-clone-sources) selects only the bounded clone correction.
