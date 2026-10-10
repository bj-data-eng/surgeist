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

## Font width and decoration metric source files

The [font width and decoration witness](../../references/browser-font-width-decoration--pinned-20261010.md) retains two complete font matching files and a complete TextBoxPainter.cpp from the preferred revision. The same font matching files are verified byte-identical at newer revision `437139e30888c5e1b4c35f889f32e1d0c39d018e` and linked to the retained bodies without duplication. Original headers remain attached to each complete source block; the accompanying exact header copies are listed here. The separate Blink file in that witness is covered by the Chromium notice.

| Retained WebKit source | Exact original header material |
| --- | --- |
| [FontSelectionAlgorithm.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontSelectionAlgorithm.cpp) | [font-selection-algorithm-bsd-2-clause.txt](font-selection-algorithm-bsd-2-clause.txt) |
| [FontSelectionAlgorithm.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontSelectionAlgorithm.h) | [font-selection-algorithm-bsd-2-clause.txt](font-selection-algorithm-bsd-2-clause.txt) |
| [TextBoxPainter.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/TextBoxPainter.cpp) | [text-box-painter-bsd-2-clause.txt](text-box-painter-bsd-2-clause.txt) |

These notices retain Copyright (C) 2017–2026 Apple Inc. for the matching files and Copyright (C) 2021–2023 Apple Inc. for TextBoxPainter.cpp. All three use their exact two-clause BSD notices; no source is compiled or linked by these reference copies.

## Whitespace, character alignment and hyphenation source files

The [text source witness](../../references/webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md) retains twelve complete files at the preferred revision, acquired from local immutable git objects on 2026-10-10. Every file retains its complete copyright and license notice. RenderStyleConstants.h carries GNU Library General Public License version 2 or later; the full [version 2 license](LICENSE-LGPL-2.txt) is already retained from this same revision. Its source-declared copyright notices include Lars Knoll, Antti Koivisto, Dirk Mueller, Apple Inc., Graham Dennis, Torch Mobile Inc., and Samuel Weinig. The other eleven files carry two-clause BSD notices; only byte-identical headers share accompanying copies.

| Retained source | Applicable local license or exact header material |
| --- | --- |
| [RenderStyleConstants.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/style/RenderStyleConstants.h) | [LICENSE-LGPL-2.txt](LICENSE-LGPL-2.txt) |
| [StyleTextAlign.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/values/text/StyleTextAlign.h) | [styletextalign-h-bsd-2-clause.txt](styletextalign-h-bsd-2-clause.txt) |
| [TextBreakingPositionContext.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/text/TextBreakingPositionContext.h) | [textbreakingpositioncontext-h-bsd-2-clause.txt](textbreakingpositioncontext-h-bsd-2-clause.txt) |
| [InlineLayoutState.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineLayoutState.h) | [scope-rule-bsd-2-clause.txt](scope-rule-bsd-2-clause.txt) |
| [InlineFormattingContext.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineFormattingContext.cpp) | [inlineformattingcontext-cpp-bsd-2-clause.txt](inlineformattingcontext-cpp-bsd-2-clause.txt) |
| [InlineLineBuilder.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineLineBuilder.cpp) | [inlinelinebuilder-cpp-bsd-2-clause.txt](inlinelinebuilder-cpp-bsd-2-clause.txt) |
| [LineLayoutResult.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/LineLayoutResult.h) | [scope-rule-bsd-2-clause.txt](scope-rule-bsd-2-clause.txt) |
| [InlineFormattingUtils.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineFormattingUtils.cpp) | [inlineformattingutils-cpp-bsd-2-clause.txt](inlineformattingutils-cpp-bsd-2-clause.txt) |
| [InlineContentBreaker.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineContentBreaker.cpp) | [inlinecontentbreaker-cpp-bsd-2-clause.txt](inlinecontentbreaker-cpp-bsd-2-clause.txt) |
| [InlineContentBreaker.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineContentBreaker.h) | [inlinecontentbreaker-h-bsd-2-clause.txt](inlinecontentbreaker-h-bsd-2-clause.txt) |
| [TextUtil.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/text/TextUtil.cpp) | [textutil-cpp-bsd-2-clause.txt](textutil-cpp-bsd-2-clause.txt) |
| [TextUtil.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/text/TextUtil.h) | [inlinecontentbreaker-cpp-bsd-2-clause.txt](inlinecontentbreaker-cpp-bsd-2-clause.txt) |

The original per-file copyright years and any Google Inc. attribution remain in the attached headers. Added reference labels do not change code bytes. These documentation copies are not compiled or linked product code.

## Filter and perspective-backface source files

The [filter and perspective-backface witness](../../references/webkit-filter-backface-witnesses--73aa6c89e2cb--5a2ab9f34073.md) retains two complete source files and three explicitly bounded excerpts at the preferred revision, acquired from local immutable git objects on 2026-10-10. Each excerpt includes its complete original file-level header, including every named copyright holder and contributor. Added line labels and omission boundaries are documentation formatting; the retained source segments are unchanged.

| Retained source and scope | Applicable local license or exact header material |
| --- | --- |
| [ColorMatrix.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/ColorMatrix.h): full source file | [colormatrix-h-bsd-2-clause.txt](colormatrix-h-bsd-2-clause.txt) |
| [FEDisplacementMapSoftwareApplier.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/filters/software/FEDisplacementMapSoftwareApplier.cpp): full source file | [GNU Library GPL version 2 or later](LICENSE-LGPL-2.txt) |
| [TransformationMatrix.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp): complete isBackFaceVisible function | [transformationmatrix-cpp-bsd-2-clause.txt](transformationmatrix-cpp-bsd-2-clause.txt) |
| [BackgroundPainter.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/BackgroundPainter.cpp): complete calculateFillLayerImageGeometryImpl function | [GNU Library GPL version 2 or later](LICENSE-LGPL-2.txt) |
| [RenderLayer.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderLayer.cpp): bounded paintLayerContents extracts showing filter context selection, background painting and filter application; intervening content is omitted | [render-layer-tri-license-header.txt](render-layer-tri-license-header.txt); [LGPL 2.1](LICENSE-LGPL-2.1.txt) or [MPL 1.1](LICENSE-MPL-1.1.txt) or [GPL 2.0](LICENSE-GPL-2.txt), preserving the original LGPL 2.1-or-later option |

The full LGPL 2.1 license is copied byte-for-byte from `Source/WebCore/LICENSE-LGPL-2.1` at the preferred revision. RenderLayer.cpp retains its original alternatives: LGPL version 2.1 or later, or MPL version 1.1, or GPL version 2.0. Its original header is unchanged; no alternative is deleted or selected by these documentation copies. The full MPL 1.1 text is retained verbatim from [Mozilla’s plain-text license](https://www.mozilla.org/media/MPL/1.1/index.txt), and GPL 2.0 from [GNU’s version 2 plain-text license](https://www.gnu.org/licenses/old-licenses/gpl-2.0.txt), retrieved on 2026-10-10.

FEDisplacementMapSoftwareApplier.cpp and BackgroundPainter.cpp retain their own LGPL version 2 or later declarations and all original copyright names. ColorMatrix.h and TransformationMatrix.cpp retain their own exact BSD notices. No copied reference source is compiled or linked into Surgeist, and this notice does not replace file-specific licensing with a project-wide WebKit license.
