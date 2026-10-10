# Chromium and Blink reference attribution

Upstream projects: [Chromium](https://www.chromium.org/) and [Blink](https://www.chromium.org/blink/).

The [Page and margin CSSOM source witness](../../references/blink-cssom-page-rules--7984f9d11800.md) retains six complete files at immutable Chromium revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`, acquired from official Gitiles TEXT responses on 2026-10-09. Every path and exact source hash is retained in that reference, together with all original file headers. Added headings and evidence-map formatting do not change source bytes. The source is documentation material, not compiled or linked product code.

The four Chromium-authored files css_margin_rule.cc, css_margin_rule.idl, css_parser.cc and css_parser_impl.cc carry BSD-style project-license declarations. Their copyright notices (2024 or 2014 The Chromium Authors) remain in full. The exact root [Chromium LICENSE](LICENSE.txt), copyright 2015 The Chromium Authors, is retained from that same immutable revision and contains the BSD redistribution conditions and disclaimer.

The other two files, css_page_rule.cc and css_selector.cc, explicitly carry GNU Library General Public License version 2 or later notices. Their complete copyright notices remain in the source blocks, including Lars Knoll, Waldo Bastian, Andreas Schlapbach, Dirk Mueller, Apple Inc., David Smith and Google Inc. The exact [Blink LICENSE_FOR_ABOUT_CREDITS](blink-LICENSE_FOR_ABOUT_CREDITS.txt) is retained from `third_party/blink/LICENSE_FOR_ABOUT_CREDITS` at this same revision. It includes the full GNU Library General Public License version 2 and GNU Lesser General Public License version 2.1, together with the source-derived upstream credit statement. The Chromium BSD license is not substituted for the file-specific LGPL notices.

## Wrapper deficiency and dispatch source files

The [wrapper deficiency witness](../../references/blink-cssom-wrapper-gaps--7984f9d11800.md) retains seven additional complete files at the same immutable Chromium revision, acquired from official Gitiles TEXT responses on 2026-10-10. Every original header, path, and decoded-byte hash is retained. All seven carry the Chromium BSD-style project-license declaration and are covered by the same exact [root LICENSE](LICENSE.txt) already retained at that revision. The original file copyright years remain in the complete source blocks.

| Retained source | Original copyright declaration |
| --- | --- |
| [css_counter_style_rule.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_counter_style_rule.cc) | Copyright 2020 The Chromium Authors |
| [css_custom_media_rule.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_custom_media_rule.cc) | Copyright 2025 The Chromium Authors |
| [css_font_feature_values_rule.idl](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_font_feature_values_rule.idl) | Copyright 2022 The Chromium Authors |
| [css_font_feature_values_rule.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_font_feature_values_rule.cc) | Copyright 2022 The Chromium Authors |
| [style_rule_font_feature_values.h](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/style_rule_font_feature_values.h) | Copyright 2022 The Chromium Authors |
| [css_at_rule_id.h](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_at_rule_id.h) | Copyright 2015 The Chromium Authors |
| [css_at_rule_id.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_at_rule_id.cc) | Copyright 2015 The Chromium Authors |

No new LGPL files are included in this witness, and the earlier Page/margin LGPL notices are unchanged. Source blocks are documentation copies and were not compiled or linked into Surgeist.

## Text properties, line breaking and decoration metric source files

The [text source witness](../../references/blink-text-source-gaps--7984f9d11800.md) retains three additional complete source files, and the [font width and decoration witness](../../references/browser-font-width-decoration--pinned-20261010.md) retains one complete Blink decoration source file. All four were acquired from official Gitiles TEXT responses on 2026-10-10 at the same immutable Chromium revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. The original headers, exact decoded-byte hashes, paths and immutable links are retained in their source blocks.

| Retained source | Original copyright declaration |
| --- | --- |
| [css_properties.json5](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_properties.json5) | Copyright 2017 The Chromium Authors |
| [line_breaker.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/inline/line_breaker.cc) | Copyright 2016 The Chromium Authors |
| [line_breaker.h](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/inline/line_breaker.h) | Copyright 2017 The Chromium Authors |
| [text_decoration_info.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/paint/text_decoration_info.cc) | Copyright 2020 The Chromium Authors |

All four carry the Chromium BSD-style project-license declaration and are covered by the exact retained [Chromium root LICENSE](LICENSE.txt) from that revision. No LGPL terms are substituted across files; the earlier Page/margin LGPL source notices remain unchanged. These reference additions do not compile or link copied source into Surgeist.
