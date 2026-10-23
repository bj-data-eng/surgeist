# WebKit hyphen indication witness

This metadata-only witness identifies a bounded font fallback and logical
placement mechanism at WebKit revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`. It supports the
[Text-owned indication policy](../docs/text-layout-contract.md#hyphenation-policy-and-indication-ownership).
It is neither a language-convention table nor evidence of automatic hyphenation
resource availability. No source or license-header excerpt is redistributed,
compiled or ingested by this reference.

## Source identities and observations

- [`Source/WebCore/style/computed/StyleComputedStyle.cpp`, lines 405–420](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/computed/StyleComputedStyle.cpp#L405):
  complete-source SHA-256
  `755a281fc1832a6a5f89104528d5bba1f3031216fed64ab37c606089233070e2`.
  The indication resolver returns a specified string unchanged, including empty.
  For auto, it uses U+2010 when the primary font supplies a nonzero glyph,
  otherwise U+002D. Its explicit locale-related FIXME leaves language-specific
  resolution unfinished. With the same primary font, this operation gives the
  same answer for known, unknown and unsupported languages.
- [`Source/WebCore/platform/graphics/Font.cpp`, line 427](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/Font.cpp#L427):
  complete-source SHA-256
  `02af2240742e0e07b536adfd923e77507b7a1ff2588a31b08aa546b25c6e0003`.
  The glyph query uses that font's glyph page. This is not a search for U+2010
  throughout the text font cascade.
- [`Source/WebCore/layout/formattingContexts/inline/display/InlineDisplayContentBuilder.cpp`, lines 210–230](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/display/InlineDisplayContentBuilder.cpp#L210):
  complete-source SHA-256
  `ff658ee4c9940b468a797fe630c046d51dc906ce133fe1557790acfb8a8b4486`.
  The inspected path appends the style's indication to the selected substring.
  This establishes logical trailing insertion in that path, not physical-right
  placement in RTL or comprehensive language-specific placement support.
- [`Source/WebCore/platform/graphics/FontCascadeFonts.h`, lines 231–259](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontCascadeFonts.h#L231):
  complete-source SHA-256
  `ba58d0d1adc1be495efdee1024b664894c506de5094b9891baae80e9f35a6991`.
  Primary-font resolution explicitly cites CSS Fonts' first-available-font rule
  and distinguishes space eligibility from actual space-glyph availability.
- [`Source/WebCore/platform/graphics/FontCascadeFonts.cpp`, lines 200–233](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontCascadeFonts.cpp#L200):
  complete-source SHA-256
  `8d085714b786c3c80bd35c582062d340e11eaa14b76e7e9baf6c29da77b6f140`.
  Cascade realization begins with the style's family resolution, then has an
  explicit standard-family/last-resort path. This does not establish a native
  primary by taking arbitrary script-fallback candidates, or require failure for
  every missing family list.

These files were inspected on 2026-10-23. Their original file-level notices
identify LGPL version 2-or-later material for ComputedStyle and FontCascadeFonts.h,
and BSD notices for Font, InlineDisplayContentBuilder and FontCascadeFonts.cpp.
Those notices remain with the immutable
originals; this record copies no implementation text. See the existing
[WebKit attribution](../licenses/webkit/NOTICE.md) and
[LGPL version 2 text](../licenses/webkit/LICENSE-LGPL-2.txt).

Surgeist selects only the documented unknown-convention fallback. The pinned
CSS Text language and writing-system requirements retain authority; this browser
resolver's missing locale handling cannot establish their completion.
