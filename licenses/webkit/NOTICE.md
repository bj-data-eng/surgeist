# WebKit reference attribution

Upstream project: [WebKit](https://webkit.org/).

The [container-query source witness](../../references/webkit-cssom-container-query--73aa6c89e2cb.md) retains four complete source files from immutable commit `73aa6c89e2cb77c46184a81aec944e4ab99d114d`, acquired from the existing local checkout on 2026-10-09. Each file’s exact path/hash and complete original license notice remain in that reference. The added headings and evidence map are documentation formatting only; source bytes remain unchanged.

Copyright (C) 2022 Apple Inc. All rights reserved.

| Source | File-specific license notice |
| --- | --- |
| [CSSContainerRule.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSContainerRule.cpp) | [apple-bsd-2-clause-with-contributors.txt](apple-bsd-2-clause-with-contributors.txt) |
| [ContainerQueryParser.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/ContainerQueryParser.cpp) | [apple-bsd-2-clause.txt](apple-bsd-2-clause.txt) |
| [GenericMediaQuerySerialization.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/GenericMediaQuerySerialization.cpp) | [apple-bsd-2-clause.txt](apple-bsd-2-clause.txt) |
| [ContainerQuery.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/ContainerQuery.cpp) | [apple-bsd-2-clause.txt](apple-bsd-2-clause.txt) |

These are the exact two-clause redistribution conditions and disclaimers carried by the selected files, with both header variants retained. They do not determine licensing for other WebKit files. No WebKit binary or compiled implementation is bundled by this reference addition.

## Core CSSOM source files

The [core CSSOM witness](../../references/webkit-cssom-core-rules--73aa6c89e2cb.md) retains five complete files at the same pinned revision. Four carry GNU Library General Public License version 2 or later notices: CSSImportRule.cpp, MediaList.cpp, CSSRule.h and CSSStyleSheet.cpp. Their complete original copyright notices remain in the source blocks, including Lars Knoll, Dirk Mueller, Apple Inc. and Andreas Kling. The full [GNU Library General Public License version 2](LICENSE-LGPL-2.txt) is copied byte-for-byte from `Source/WebCore/LICENSE-LGPL-2` at that revision.

CSSGroupingRule.cpp instead carries the [original two-clause notice](cssgroupingrule-bsd-2-clause.txt), copyright 2011 Adobe Systems Incorporated and 2012–2024 Apple Inc. All rights reserved. Its complete notice remains in the source block. No LGPL/BSD terms are substituted across files, and none of this source is compiled or linked into Surgeist by these references.

## Wrapper and at-rule dispatch source files

The [wrapper and dispatch witness](../../references/webkit-cssom-wrapper-rules--73aa6c89e2cb.md) retains twelve complete files from the preferred revision and one complete counter-style header from newer revision `437139e30888c5e1b4c35f889f32e1d0c39d018e`, acquired from immutable local git objects on 2026-10-10. Four newer identities are verified byte-identical to retained preferred bodies and are recorded without duplicate code blocks. Complete copyright, redistribution conditions, and disclaimers remain in every source block. The following exact header files are copied byte-for-byte; identical headers share a file.

| Retained source | Exact original header material |
| --- | --- |
| [CSSCounterStyleRule.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSCounterStyleRule.cpp) | [counterstyle-bsd-2-clause.txt](counterstyle-bsd-2-clause.txt) |
| [CSSCounterStyleRule.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSCounterStyleRule.h) | [counterstyle-bsd-2-clause.txt](counterstyle-bsd-2-clause.txt) |
| [CSSFontFeatureValuesRule.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSFontFeatureValuesRule.cpp) | [fontfeaturevaluesrule-bsd-2-clause.txt](fontfeaturevaluesrule-bsd-2-clause.txt) |
| [CSSFontPaletteValuesRule.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSFontPaletteValuesRule.cpp) | [fontpalettevaluesrule-bsd-2-clause.txt](fontpalettevaluesrule-bsd-2-clause.txt) |
| [CSSLayerBlockRule.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSLayerBlockRule.cpp) | [layer-rule-bsd-2-clause.txt](layer-rule-bsd-2-clause.txt) |
| [CSSLayerStatementRule.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSLayerStatementRule.cpp) | [layer-rule-bsd-2-clause.txt](layer-rule-bsd-2-clause.txt) |
| [CSSSupportsRule.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSSupportsRule.cpp) | [supports-rule-bsd-3-clause.txt](supports-rule-bsd-3-clause.txt) |
| [CSSScopeRule.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSScopeRule.cpp) | [scope-rule-bsd-2-clause.txt](scope-rule-bsd-2-clause.txt) |
| [FontFeatureValues.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontFeatureValues.h) | [apple-bsd-2-clause-with-contributors.txt](apple-bsd-2-clause-with-contributors.txt) |
| [FontFeatureValues.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontFeatureValues.cpp) | [apple-bsd-2-clause-with-contributors.txt](apple-bsd-2-clause-with-contributors.txt) |
| [CSSAtRuleID.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSAtRuleID.h) | [at-rule-id-h-bsd-3-clause.txt](at-rule-id-h-bsd-3-clause.txt) |
| [CSSAtRuleID.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSAtRuleID.cpp) | [at-rule-id-cpp-bsd-3-clause.txt](at-rule-id-cpp-bsd-3-clause.txt) |
| [CSSCounterStyleRule.h (newer revision)](https://github.com/WebKit/WebKit/blob/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/css/CSSCounterStyleRule.h) | [counterstyle-bsd-2-clause.txt](counterstyle-bsd-2-clause.txt) |

These notices include the source-declared Tyler Wilcock, Apple Inc., Motorola Mobility Inc., and Chromium Authors attribution. Each two-clause or three-clause notice applies only to its listed files. The existing LGPL files remain governed by their own notices. No copied reference source is compiled or linked into Surgeist.
