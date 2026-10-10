# WebKit CSSOM wrapper and dispatch source witness

This reference retains 13 complete source files from [WebKit](https://webkit.org/). The preferred immutable revision is `73aa6c89e2cb77c46184a81aec944e4ab99d114d`; one complete counter-style header from newer revision `437139e30888c5e1b4c35f889f32e1d0c39d018e` records a persisting getter deficiency. It provides bounded CSSOM implementation evidence, independently of the normative specifications and any Surgeist implementation decision.

Retrieved on 2026-10-10 from the existing local checkout using immutable git object bytes. Each complete file below retains its original path, exact byte hash, and copyright/license header. Added provenance, headings, and evidence labels are documentation formatting; the source bytes are unchanged and were not compiled or executed.

File-specific legal material is recorded in the [local attribution](../licenses/webkit/NOTICE.md). The original notices govern each source file.

The LayerBlock, Supports, and Scope wrappers explicitly call the grouping formatter retained in the [core CSSOM witness](webkit-cssom-core-rules--73aa6c89e2cb.md#cssgroupingrule-cpp). Container wrapper and query evidence remain in the [container-query witness](webkit-cssom-container-query--73aa6c89e2cb.md#csscontainerrule-cpp). They are not duplicated here.

The six font-feature categories do not establish complete output for the selected current [Fonts 4 `historicalForms` surface](css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md#cssfontfeaturevaluesrule) or its top-level [font-display descriptor](css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md#descdef-font-feature-values-font-display). Neither retained browser font-feature wrapper emits that descriptor. Their native integer storage also does not define conversion from authored exact integers outside its representable range. The pinned counter-style getter returns an empty `speakAs` string; the [Blink witness](blink-cssom-wrapper-gaps--7984f9d11800.md#css-counter-style-rule-cc) retains the stored-value getter that this WebKit revision lacks. Supports condition, selector, and query serialization remain distinct component contracts; the wrapper source alone does not define them.

The at-rule dispatch evidence has no named-support rule. The selected Conditional 5 [section 8](css-conditional-5--editor-capture-20261009--75e15be8c8b4.md#supports-condition-rule) and [section 9.2](css-conditional-5--editor-capture-20261009--75e15be8c8b4.md#the-csssupportscondition-interface) define a non-rendering test body and `CSSGroupingRule` inheritance but no connecting child-rule projection or whole-rule serialization algorithm. Inheritance alone does not settle the mixed test-body representation. The absence finding applies to these inspected revisions.

| Source | Evidence boundary | Original file SHA-256 |
| --- | --- | --- |
| [CSSCounterStyleRule.cpp](#csscounterstylerule-cpp) | cssText at line 132 supplies the ten-descriptor wrapper order and spacing. | `b1da5281de80c0aa8a2b4d0a7741d4b9eaeabd256aad64e18050c4c8b126c291` |
| [CSSCounterStyleRule.h](#csscounterstylerule-h) | StyleRuleCounterStyle::speakAs at line 55 returns an empty string. | `7acd520219c1130a324b42eb62fbb030f13294d16b7662593a972ef1b2f13d06` |
| [CSSFontFeatureValuesRule.cpp](#cssfontfeaturevaluesrule-cpp) | cssText at line 42 supplies six-category order, category blocks, and alias formatting; it emits no font-display descriptor. | `bacb7b63b433e6f2e3b31e944f9bd10373d81e8867c6ef3052e22771c38134f3` |
| [CSSFontPaletteValuesRule.cpp](#cssfontpalettevaluesrule-cpp) | cssText at line 89 orders present descriptors; getter methods supply their component text. | `6e5440ee8058614157daab4b339d6e3af1b8d986ba125322352a1e1c8f18b7bd` |
| [CSSLayerBlockRule.cpp](#csslayerblockrule-cpp) | cssText at line 51 delegates its named or anonymous prelude to the grouping helper; stringFromCascadeLayerName serializes dotted names. | `28e158518363f9bbb9efcaec563503d9ae131b97793c54c97849ebe0ba4c4376` |
| [CSSLayerStatementRule.cpp](#csslayerstatementrule-cpp) | cssText at line 54 joins nameList with comma-space and appends a semicolon. | `7b2039cea01dd5409135a007adaec6e66e199bcf3bb791888d08442c017fd878` |
| [CSSSupportsRule.cpp](#csssupportsrule-cpp) | cssText at line 51 composes conditionText with the grouping helper; it does not define a separate condition-token policy. | `adcc1015637073ada0ad431c15a09e5b3e305f7e57df79b7a543ce6f55b87fc0` |
| [CSSScopeRule.cpp](#cssscoperule-cpp) | cssText at line 49 composes optional start/end selectors and the grouping helper. | `2824c9f7a6298b5a963e85f3d68db09cace221ba5c3d0957bc078b9530f51d24` |
| [FontFeatureValues.h](#fontfeaturevalues-h) | Tags at line 50 uses OrderedHashMap; the category enum and storage contain six categories. | `c8f1257cabc1812cf890b985199f519e88718708d5e67fe449d7186631dca137` |
| [FontFeatureValues.cpp](#fontfeaturevalues-cpp) | updateOrInsertForType at line 65 inserts input tags into the selected ordered map. | `0a1e7b9c879248685d6a475441a719aecb8f1d55c0c59946372bd8bc775a5f73` |
| [CSSAtRuleID.h](#cssatruleid-h) | The complete at-rule enum contains supports and no supports-condition. | `513fd46453ad215eebf3a43451865a5baf1a7d0d3a7f76e9c41bbeb1dc23eb95` |
| [CSSAtRuleID.cpp](#cssatruleid-cpp) | The complete name-dispatch table contains supports and no supports-condition, custom-media, when, else, or color-profile. | `75b617c09371237044d852cb1371523bd7c57227d8958e5f0285cc267792235e` |
| [CSSCounterStyleRule.h (newer revision)](#newer-csscounterstylerule-h) | The newer header retains the empty speakAs getter at line 55. Its other bytes differ from the preferred header. | `66a2906a7799f7874fd6d9feed27de4a6aaf700affee2bf6b099603d91048b75` |

## Newer-revision checks without duplicate source blocks

The four sources below were compared at newer fetched WebKit revision `437139e30888c5e1b4c35f889f32e1d0c39d018e` using git objects without changing the checkout. Each is byte-identical to its complete preferred-revision body linked here; the newer immutable identity and the same exact hash are recorded separately. The differing newer counter-style header has its own complete block below.

| Newer source | Equal retained body | Source SHA-256 |
| --- | --- | --- |
| [CSSFontFeatureValuesRule.cpp](https://github.com/WebKit/WebKit/blob/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/css/CSSFontFeatureValuesRule.cpp) | [CSSFontFeatureValuesRule.cpp](#cssfontfeaturevaluesrule-cpp) | `bacb7b63b433e6f2e3b31e944f9bd10373d81e8867c6ef3052e22771c38134f3` |
| [FontFeatureValues.h](https://github.com/WebKit/WebKit/blob/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/platform/graphics/FontFeatureValues.h) | [FontFeatureValues.h](#fontfeaturevalues-h) | `c8f1257cabc1812cf890b985199f519e88718708d5e67fe449d7186631dca137` |
| [CSSAtRuleID.h](https://github.com/WebKit/WebKit/blob/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/css/parser/CSSAtRuleID.h) | [CSSAtRuleID.h](#cssatruleid-h) | `513fd46453ad215eebf3a43451865a5baf1a7d0d3a7f76e9c41bbeb1dc23eb95` |
| [CSSAtRuleID.cpp](https://github.com/WebKit/WebKit/blob/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/css/parser/CSSAtRuleID.cpp) | [CSSAtRuleID.cpp](#cssatruleid-cpp) | `75b617c09371237044d852cb1371523bd7c57227d8958e5f0285cc267792235e` |

## <a id="csscounterstylerule-cpp"></a>CSSCounterStyleRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSCounterStyleRule.cpp). Path: `Source/WebCore/css/CSSCounterStyleRule.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 12357; source lines: 309; SHA-256: `b1da5281de80c0aa8a2b4d0a7741d4b9eaeabd256aad64e18050c4c8b126c291`.

````cpp
/*
 * Copyright (C) 2021 Tyler Wilcock <twilco.o@protonmail.com>.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "CSSCounterStyleRule.h"

#include "CSSCounterStyleDescriptors.h"
#include "CSSKeywordValue.h"
#include "CSSMarkup.h"
#include "CSSPropertyParser.h"
#include "CSSPropertyParserConsumer+CounterStyles.h"
#include "CSSStyleSheet.h"
#include "CSSTokenizer.h"
#include "CSSValuePair.h"
#include "MutableStyleProperties.h"
#include "StyleProperties.h"
#include "StylePropertiesInlines.h"
#include <wtf/text/MakeString.h>
#include <wtf/text/StringBuilder.h>

namespace WebCore {

StyleRuleCounterStyle::StyleRuleCounterStyle(const AtomString& name, CSSCounterStyleDescriptors&& descriptors)
    : StyleRuleBase(StyleRuleType::CounterStyle)
    , m_name(name)
    , m_descriptors(WTF::move(descriptors))
{
}

Ref<StyleRuleCounterStyle> StyleRuleCounterStyle::create(const AtomString& name, CSSCounterStyleDescriptors&& descriptors)
{
    return adoptRef(*new StyleRuleCounterStyle(name, WTF::move(descriptors)));
}

CSSCounterStyleDescriptors::System toCounterStyleSystemEnum(const CSSValue* system)
{
    if (!system)
        return CSSCounterStyleDescriptors::System::Symbolic;

    ASSERT(system->isKeywordValue() || system->isPair());
    CSSValueID systemKeyword = CSSValueInvalid;
    if (RefPtr systemIdent = dynamicDowncast<CSSKeywordValue>(system))
        systemKeyword = systemIdent->valueID();
    else if (system->isPair()) {
        if (RefPtr systemIdent = dynamicDowncast<CSSKeywordValue>(system->first())) {
            // This value must be `fixed` or `extends`, both of which can or must have an additional component.
            systemKeyword = systemIdent->valueID();
        }
    }
    switch (systemKeyword) {
    case CSSValueCyclic:
        return CSSCounterStyleDescriptors::System::Cyclic;
    case CSSValueFixed:
        return CSSCounterStyleDescriptors::System::Fixed;
    case CSSValueSymbolic:
        return CSSCounterStyleDescriptors::System::Symbolic;
    case CSSValueAlphabetic:
        return CSSCounterStyleDescriptors::System::Alphabetic;
    case CSSValueNumeric:
        return CSSCounterStyleDescriptors::System::Numeric;
    case CSSValueAdditive:
        return CSSCounterStyleDescriptors::System::Additive;
    case CSSValueInternalDisclosureClosed:
        return CSSCounterStyleDescriptors::System::DisclosureClosed;
    case CSSValueInternalDisclosureOpen:
        return CSSCounterStyleDescriptors::System::DisclosureOpen;
    case CSSValueInternalSimplifiedChineseInformal:
        return CSSCounterStyleDescriptors::System::SimplifiedChineseInformal;
    case CSSValueInternalSimplifiedChineseFormal:
        return CSSCounterStyleDescriptors::System::SimplifiedChineseFormal;
    case CSSValueInternalTraditionalChineseInformal:
        return CSSCounterStyleDescriptors::System::TraditionalChineseInformal;
    case CSSValueInternalTraditionalChineseFormal:
        return CSSCounterStyleDescriptors::System::TraditionalChineseFormal;
    case CSSValueInternalJapaneseInformal:
        return CSSCounterStyleDescriptors::System::JapaneseInformal;
    case CSSValueInternalJapaneseFormal:
        return CSSCounterStyleDescriptors::System::JapaneseFormal;
    case CSSValueInternalKoreanHangulFormal:
        return CSSCounterStyleDescriptors::System::KoreanHangulFormal;
    case CSSValueInternalKoreanHanjaInformal:
        return CSSCounterStyleDescriptors::System::KoreanHanjaInformal;
    case CSSValueInternalKoreanHanjaFormal:
        return CSSCounterStyleDescriptors::System::KoreanHanjaFormal;
    case CSSValueInternalEthiopicNumeric:
        return CSSCounterStyleDescriptors::System::EthiopicNumeric;
    case CSSValueExtends:
        return CSSCounterStyleDescriptors::System::Extends;
    default:
        ASSERT_NOT_REACHED();
        return CSSCounterStyleDescriptors::System::Symbolic;
    }
}

StyleRuleCounterStyle::~StyleRuleCounterStyle() = default;

Ref<CSSCounterStyleRule> CSSCounterStyleRule::create(StyleRuleCounterStyle& rule, CSSStyleSheet* sheet)
{
    return adoptRef(*new CSSCounterStyleRule(rule, sheet));
}

CSSCounterStyleRule::CSSCounterStyleRule(StyleRuleCounterStyle& counterStyleRule, CSSStyleSheet* parent)
    : CSSRule(parent)
    , m_counterStyleRule(counterStyleRule)
{
}

CSSCounterStyleRule::~CSSCounterStyleRule() = default;

String CSSCounterStyleRule::cssText() const
{
    String systemText = system();
    const auto systemPrefix = systemText.isEmpty() ? ""_s : " system: "_s;
    const auto systemSuffix = systemText.isEmpty() ? ""_s : ";"_s;

    String symbolsText = symbols();
    const auto symbolsPrefix = symbolsText.isEmpty() ? ""_s : " symbols: "_s;
    const auto symbolsSuffix = symbolsText.isEmpty() ? ""_s : ";"_s;

    String additiveSymbolsText = additiveSymbols();
    const auto additiveSymbolsPrefix = additiveSymbolsText.isEmpty() ? ""_s : " additive-symbols: "_s;
    const auto additiveSymbolsSuffix = additiveSymbolsText.isEmpty() ? ""_s : ";"_s;

    String negativeText = negative();
    const auto negativePrefix = negativeText.isEmpty() ? ""_s : " negative: "_s;
    const auto negativeSuffix = negativeText.isEmpty() ? ""_s : ";"_s;

    String prefixText = prefix();
    const auto prefixTextPrefix = prefixText.isEmpty() ? ""_s : " prefix: "_s;
    const auto prefixTextSuffix = prefixText.isEmpty() ? ""_s : ";"_s;

    String suffixText = suffix();
    const auto suffixTextPrefix = suffixText.isEmpty() ? ""_s : " suffix: "_s;
    const auto suffixTextSuffix = suffixText.isEmpty() ? ""_s : ";"_s;

    String padText = pad();
    const auto padPrefix = padText.isEmpty() ? ""_s : " pad: "_s;
    const auto padSuffix = padText.isEmpty() ? ""_s : ";"_s;

    String rangeText = range();
    const auto rangePrefix = rangeText.isEmpty() ? ""_s : " range: "_s;
    const auto rangeSuffix = rangeText.isEmpty() ? ""_s : ";"_s;

    String fallbackText = fallback();
    const auto fallbackPrefix = fallbackText.isEmpty() ? ""_s : " fallback: "_s;
    const auto fallbackSuffix = fallbackText.isEmpty() ? ""_s : ";"_s;

    String speakAsText = speakAs();
    const auto speakAsPrefix = speakAsText.isEmpty() ? ""_s : " speak-as: "_s;
    const auto speakAsSuffix = speakAsText.isEmpty() ? ""_s : ";"_s;

    StringBuilder builder;

    builder.append("@counter-style "_s);
    serializeIdentifier(builder, name());

    builder.append(" {"_s,
        systemPrefix, systemText, systemSuffix,
        symbolsPrefix, symbolsText, symbolsSuffix,
        additiveSymbolsPrefix, additiveSymbolsText, additiveSymbolsSuffix,
        negativePrefix, negativeText, negativeSuffix,
        prefixTextPrefix, prefixText, prefixTextSuffix,
        suffixTextPrefix, suffixText, suffixTextSuffix,
        padPrefix, padText, padSuffix,
        rangePrefix, rangeText, rangeSuffix,
        fallbackPrefix, fallbackText, fallbackSuffix,
        speakAsPrefix, speakAsText, speakAsSuffix,
    " }"_s);

    return builder.toString();
}

void CSSCounterStyleRule::reattach(StyleRuleBase& rule)
{
    m_counterStyleRule = downcast<StyleRuleCounterStyle>(rule);
}

RefPtr<CSSValue> CSSCounterStyleRule::cssValueFromText(CSSPropertyID propertyID, const String& string)
{
    return CSSPropertyParser::parseCounterStyleDescriptor(propertyID, string, parserContext());
}

// https://drafts.csswg.org/css-counter-styles-3/#dom-csscounterstylerule-name
void CSSCounterStyleRule::setName(const String& text)
{
    auto tokenizer = CSSTokenizer(text);
    auto tokenRange = tokenizer.tokenRange();
    auto name = CSSPropertyParserHelpers::consumeCounterStyleNameInPrelude(tokenRange);
    if (!name)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setName(WTF::move(name));
}

void CSSCounterStyleRule::setSystem(const String& text)
{
    auto systemValue = cssValueFromText(CSSPropertySystem, text);
    if (!systemValue)
        return;
    auto system = toCounterStyleSystemEnum(systemValue.get());
    // If the attribute being set is `system`, and the new value would change the algorithm used, do nothing
    // and abort these steps.
    // (It's okay to change an aspect of the algorithm, like the first symbol value of a `fixed` system.)
    // https://www.w3.org/TR/css-counter-styles-3/#the-csscounterstylerule-interface
    auto systemData = extractSystemDataFromCSSValue(WTF::move(systemValue), system);
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setSystemData(WTF::move(systemData));
}

void CSSCounterStyleRule::setNegative(const String& text)
{
    auto newValue = cssValueFromText(CSSPropertyNegative, text);
    if (!newValue)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setNegative(negativeSymbolsFromCSSValue(newValue.releaseNonNull()));
}

void CSSCounterStyleRule::setPrefix(const String& text)
{
    auto newValue = cssValueFromText(CSSPropertyPrefix, text);
    if (!newValue)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setPrefix(symbolFromCSSValue(WTF::move(newValue)));
}

void CSSCounterStyleRule::setSuffix(const String& text)
{
    auto newValue = cssValueFromText(CSSPropertySuffix, text);
    if (!newValue)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setSuffix(symbolFromCSSValue(WTF::move(newValue)));
}

void CSSCounterStyleRule::setRange(const String& text)
{
    auto newValue = cssValueFromText(CSSPropertyRange, text);
    if (!newValue)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setRanges(rangeFromCSSValue(newValue.releaseNonNull()));
}

void CSSCounterStyleRule::setPad(const String& text)
{
    auto newValue = cssValueFromText(CSSPropertyPad, text);
    if (!newValue)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setPad(padFromCSSValue(newValue.releaseNonNull()));
}

void CSSCounterStyleRule::setFallback(const String& text)
{
    auto newValue = cssValueFromText(CSSPropertyFallback, text);
    if (!newValue)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setFallbackName(fallbackNameFromCSSValue(newValue.releaseNonNull()));
}

void CSSCounterStyleRule::setSymbols(const String& text)
{
    auto newValue = cssValueFromText(CSSPropertySymbols, text);
    if (!newValue)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setSymbols(symbolsFromCSSValue(newValue.releaseNonNull()));
}

void CSSCounterStyleRule::setAdditiveSymbols(const String& text)
{
    auto newValue = cssValueFromText(CSSPropertyAdditiveSymbols, text);
    if (!newValue)
        return;
    CSSStyleSheet::RuleMutationScope mutationScope(this);
    mutableDescriptors().setAdditiveSymbols(additiveSymbolsFromCSSValue(newValue.releaseNonNull()));
}

void CSSCounterStyleRule::setSpeakAs(const String&)
{
    // FIXME: @counter-style speak-as not supported (rdar://103019111).
}

} // namespace WebCore
````

## <a id="csscounterstylerule-h"></a>CSSCounterStyleRule.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSCounterStyleRule.h). Path: `Source/WebCore/css/CSSCounterStyleRule.h`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 5536; source lines: 120; SHA-256: `7acd520219c1130a324b42eb62fbb030f13294d16b7662593a972ef1b2f13d06`.

````cpp
/*
 * Copyright (C) 2021 Tyler Wilcock <twilco.o@protonmail.com>.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include "CSSCounterStyleDescriptors.h"
#include "CSSRule.h"
#include "StyleProperties.h"
#include "StyleRule.h"
#include <wtf/text/AtomString.h>

namespace WebCore {
class StyleRuleCounterStyle final : public StyleRuleBase {
public:
    static Ref<StyleRuleCounterStyle> create(const AtomString&, CSSCounterStyleDescriptors&&);
    ~StyleRuleCounterStyle();

    Ref<StyleRuleCounterStyle> copy() const { return adoptRef(*new StyleRuleCounterStyle(*this)); }

    const CSSCounterStyleDescriptors& descriptors() const LIFETIME_BOUND { return m_descriptors; };
    CSSCounterStyleDescriptors& mutableDescriptors() LIFETIME_BOUND { return m_descriptors; };

    const AtomString& name() const LIFETIME_BOUND { return m_name; }
    String system() const { return m_descriptors.systemCSSText(); }
    String negative() const { return m_descriptors.negativeCSSText(); }
    String prefix() const { return m_descriptors.prefixCSSText(); }
    String suffix() const { return m_descriptors.suffixCSSText(); }
    String range() const { return { m_descriptors.rangesCSSText() }; }
    String pad() const { return m_descriptors.padCSSText(); }
    String fallback() const { return m_descriptors.fallbackCSSText(); }
    String symbols() const { return m_descriptors.symbolsCSSText(); }
    String additiveSymbols() const { return m_descriptors.additiveSymbolsCSSText(); }
    String speakAs() const { return { }; }
    bool newValueInvalidOrEqual(CSSPropertyID, const RefPtr<CSSValue> newValue) const;

    void setName(const AtomString& name) { m_name = name; }

private:
    explicit StyleRuleCounterStyle(const AtomString&, CSSCounterStyleDescriptors&&);
    StyleRuleCounterStyle(const StyleRuleCounterStyle&) = default;

    AtomString m_name;
    CSSCounterStyleDescriptors m_descriptors;
};

class CSSCounterStyleRule final : public CSSRule {
public:
    static Ref<CSSCounterStyleRule> create(StyleRuleCounterStyle&, CSSStyleSheet*);
    virtual ~CSSCounterStyleRule();

    String cssText() const final;
    void NODELETE reattach(StyleRuleBase&) final;
    StyleRuleType styleRuleType() const final { return StyleRuleType::CounterStyle; }

    String name() const { return m_counterStyleRule->name(); }
    String system() const { return m_counterStyleRule->system(); }
    String negative() const { return m_counterStyleRule->negative(); }
    String prefix() const { return m_counterStyleRule->prefix(); }
    String suffix() const { return m_counterStyleRule->suffix(); }
    String range() const { return m_counterStyleRule->range(); }
    String pad() const { return m_counterStyleRule->pad(); }
    String fallback() const { return m_counterStyleRule->fallback(); }
    String symbols() const { return m_counterStyleRule->symbols(); }
    String additiveSymbols() const { return m_counterStyleRule->additiveSymbols(); }
    String speakAs() const { return m_counterStyleRule->speakAs(); }

    void setName(const String&);
    void setSystem(const String&);
    void setNegative(const String&);
    void setPrefix(const String&);
    void setSuffix(const String&);
    void setRange(const String&);
    void setPad(const String&);
    void setFallback(const String&);
    void setSymbols(const String&);
    void setAdditiveSymbols(const String&);
    void NODELETE setSpeakAs(const String&);

private:
    CSSCounterStyleRule(StyleRuleCounterStyle&, CSSStyleSheet* parent);

    bool setterInternal(CSSPropertyID, const String&);
    RefPtr<CSSValue> cssValueFromText(CSSPropertyID, const String&);
    const CSSCounterStyleDescriptors& descriptors() const LIFETIME_BOUND { return m_counterStyleRule->descriptors(); }
    CSSCounterStyleDescriptors& mutableDescriptors() LIFETIME_BOUND { return m_counterStyleRule->mutableDescriptors(); }

    Ref<StyleRuleCounterStyle> m_counterStyleRule;
};

CSSCounterStyleDescriptors::System NODELETE toCounterStyleSystemEnum(const CSSValue*);

} // namespace WebCore

SPECIALIZE_TYPE_TRAITS_CSS_RULE(CSSCounterStyleRule, StyleRuleType::CounterStyle)

SPECIALIZE_TYPE_TRAITS_BEGIN(WebCore::StyleRuleCounterStyle)
static bool isType(const WebCore::StyleRuleBase& rule) { return rule.isCounterStyleRule(); }
SPECIALIZE_TYPE_TRAITS_END()
````

## <a id="cssfontfeaturevaluesrule-cpp"></a>CSSFontFeatureValuesRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSFontFeatureValuesRule.cpp). Path: `Source/WebCore/css/CSSFontFeatureValuesRule.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 4795; source lines: 124; SHA-256: `bacb7b63b433e6f2e3b31e944f9bd10373d81e8867c6ef3052e22771c38134f3`.

````cpp
/*
 * Copyright (C) 2021-2026 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "CSSFontFeatureValuesRule.h"

#include "CSSMarkup.h"
#include "CSSPropertyParserConsumer+Font.h"
#include "CSSStyleSheet.h"
#include "CSSTokenizer.h"

namespace WebCore {

CSSFontFeatureValuesRule::CSSFontFeatureValuesRule(StyleRuleFontFeatureValues& fontFeatureValuesRule, CSSStyleSheet* parent)
    : CSSRule(parent)
    , m_fontFeatureValuesRule(fontFeatureValuesRule)
{
}

String CSSFontFeatureValuesRule::cssText() const
{
    StringBuilder builder;
    builder.append("@font-feature-values "_s);
    auto joinFontFamiliesWithSeparator = [&builder] (const auto& elements, ASCIILiteral separator) {
        bool first = true;
        for (auto element : elements) {
            if (!first)
                builder.append(separator);
            serializeFontFamily(builder, element);
            first = false;
        }
    };
    joinFontFamiliesWithSeparator(m_fontFeatureValuesRule->fontFamilies(), ", "_s);
    builder.append(" { "_s);
    const auto& value = m_fontFeatureValuesRule->value();
    
    auto addVariant = [&builder] (const String& variantName, const auto& tags) {
        if (!tags.isEmpty()) {
            builder.append('@', variantName, " { "_s);
            for (auto tag : tags) {
                serializeIdentifier(builder, tag.key);
                builder.append(':');
                for (auto integer : tag.value)
                    builder.append(' ', integer);
                builder.append("; "_s);
            }
            builder.append("} "_s);
        }
    };
    
    // WPT expects the order used in Servo.
    // https://searchfox.org/mozilla-central/source/servo/components/style/stylesheets/font_feature_values_rule.rs#430
    addVariant("swash"_s, value->swash());
    addVariant("stylistic"_s, value->stylistic());
    addVariant("ornaments"_s, value->ornaments());
    addVariant("annotation"_s, value->annotation());
    addVariant("character-variant"_s, value->characterVariant());
    addVariant("styleset"_s, value->styleset());
    
    builder.append('}');
    return builder.toString();
}

void CSSFontFeatureValuesRule::reattach(StyleRuleBase& rule)
{
    m_fontFeatureValuesRule = downcast<StyleRuleFontFeatureValues>(rule);
}

// https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-fontfamily
void CSSFontFeatureValuesRule::setFontFamily(const String& fontFamily)
{
    CSSTokenizer tokenizer(fontFamily);
    auto tokenRange = tokenizer.tokenRange();
    auto fontFamilies = CSSPropertyParserHelpers::consumeFontFeatureValuesPreludeFamilyNameList(tokenRange, parserContext());
    if (fontFamilies.isEmpty() || !tokenRange.atEnd())
        return;

    CSSStyleSheet::RuleMutationScope mutationScope(this);
    protect(m_fontFeatureValuesRule)->setFontFamilies(WTF::move(fontFamilies));
}

CSSFontFeatureValuesBlockRule::CSSFontFeatureValuesBlockRule(StyleRuleFontFeatureValuesBlock& block , CSSStyleSheet* parent)
    : CSSRule(parent)
    , m_fontFeatureValuesBlockRule(block)
{
}

String CSSFontFeatureValuesBlockRule::cssText() const
{
    // This rule is always contained inside a FontFeatureValuesRule,
    // which is the only one we are expected to serialize to CSS.
    // We should never serialize a Block by itself.
    ASSERT_NOT_REACHED();
    return { };
}

void CSSFontFeatureValuesBlockRule::reattach(StyleRuleBase& rule)
{
    m_fontFeatureValuesBlockRule = downcast<StyleRuleFontFeatureValuesBlock>(rule);
}

} // namespace WebCore
````

## <a id="cssfontpalettevaluesrule-cpp"></a>CSSFontPaletteValuesRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSFontPaletteValuesRule.cpp). Path: `Source/WebCore/css/CSSFontPaletteValuesRule.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 4722; source lines: 132; SHA-256: `6e5440ee8058614157daab4b339d6e3af1b8d986ba125322352a1e1c8f18b7bd`.

````cpp
/*
 * Copyright (C) 2021 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "CSSFontPaletteValuesRule.h"

#include "CSSMarkup.h"
#include "ColorSerialization.h"
#include "JSDOMMapLike.h"
#include "StyleProperties.h"
#include "StyleRule.h"
#include <wtf/text/MakeString.h>
#include <wtf/text/StringBuilder.h>
#include <wtf/text/WTFString.h>

namespace WebCore {

CSSFontPaletteValuesRule::CSSFontPaletteValuesRule(StyleRuleFontPaletteValues& fontPaletteValuesRule, CSSStyleSheet* parent)
    : CSSRule(parent)
    , m_fontPaletteValuesRule(fontPaletteValuesRule)
{
}

CSSFontPaletteValuesRule::~CSSFontPaletteValuesRule() = default;

String CSSFontPaletteValuesRule::name() const
{
    return m_fontPaletteValuesRule->name();
}

String CSSFontPaletteValuesRule::fontFamily() const
{
    auto serialize = [] (auto& family) {
        return serializeFontFamily(family.string());
    };
    return makeStringByJoining(m_fontPaletteValuesRule->fontFamilies().map(serialize).span(), ", "_s);
}

String CSSFontPaletteValuesRule::basePalette() const
{
    if (!m_fontPaletteValuesRule->basePalette())
        return StringImpl::empty();

    switch (m_fontPaletteValuesRule->basePalette()->type) {
    case FontPaletteIndex::Type::Light:
        return "light"_s;
    case FontPaletteIndex::Type::Dark:
        return "dark"_s;
    case FontPaletteIndex::Type::Integer:
        return makeString(m_fontPaletteValuesRule->basePalette()->integer);
    }
    RELEASE_ASSERT_NOT_REACHED();
}

String CSSFontPaletteValuesRule::overrideColors() const
{     
    StringBuilder result;
    for (size_t i = 0; i < m_fontPaletteValuesRule->overrideColors().size(); ++i) {
        if (i)
            result.append(", "_s);
        const auto& item = m_fontPaletteValuesRule->overrideColors()[i];
        result.append(item.first, ' ', serializationForCSS(item.second));
    }
    return result.toString();
}

String CSSFontPaletteValuesRule::cssText() const
{
    StringBuilder builder;

    builder.append("@font-palette-values "_s);
    serializeIdentifier(builder, name());
    builder.append(" { "_s);

    if (!m_fontPaletteValuesRule->fontFamilies().isEmpty())
        builder.append("font-family: "_s, fontFamily(), "; "_s);

    if (m_fontPaletteValuesRule->basePalette()) {
        switch (m_fontPaletteValuesRule->basePalette()->type) {
        case FontPaletteIndex::Type::Light:
            builder.append("base-palette: light; "_s);
            break;
        case FontPaletteIndex::Type::Dark:
            builder.append("base-palette: dark; "_s);
            break;
        case FontPaletteIndex::Type::Integer:
            builder.append("base-palette: "_s, m_fontPaletteValuesRule->basePalette()->integer, "; "_s);
            break;
        }
    }

    if (!m_fontPaletteValuesRule->overrideColors().isEmpty()) {
        builder.append("override-colors:"_s);
        for (size_t i = 0; i < m_fontPaletteValuesRule->overrideColors().size(); ++i) {
            if (i)
                builder.append(',');
            builder.append(' ', m_fontPaletteValuesRule->overrideColors()[i].first, ' ', serializationForCSS(m_fontPaletteValuesRule->overrideColors()[i].second));
        }
        builder.append("; "_s);
    }
    builder.append('}');
    return builder.toString();
}

void CSSFontPaletteValuesRule::reattach(StyleRuleBase& rule)
{
    m_fontPaletteValuesRule = downcast<StyleRuleFontPaletteValues>(rule);
}

} // namespace WebCore
````

## <a id="csslayerblockrule-cpp"></a>CSSLayerBlockRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSLayerBlockRule.cpp). Path: `Source/WebCore/css/CSSLayerBlockRule.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 2630; source lines: 84; SHA-256: `28e158518363f9bbb9efcaec563503d9ae131b97793c54c97849ebe0ba4c4376`.

````cpp
/*
 * Copyright (C) 2021 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1. Redistributions of source code must retain the above
 *    copyright notice, this list of conditions and the following
 *    disclaimer.
 * 2. Redistributions in binary form must reproduce the above
 *    copyright notice, this list of conditions and the following
 *    disclaimer in the documentation and/or other materials
 *    provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDER "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY,
 * OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR
 * TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF
 * THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */

#include "config.h"
#include "CSSLayerBlockRule.h"

#include "CSSMarkup.h"
#include "CSSStyleSheet.h"
#include "StyleRule.h"
#include <wtf/text/StringBuilder.h>

namespace WebCore {

CSSLayerBlockRule::CSSLayerBlockRule(StyleRuleLayer& rule, CSSStyleSheet* parent)
    : CSSGroupingRule(rule, parent)
{
    ASSERT(!rule.isStatement());
}

Ref<CSSLayerBlockRule> CSSLayerBlockRule::create(StyleRuleLayer& rule, CSSStyleSheet* parent)
{
    return adoptRef(*new CSSLayerBlockRule(rule, parent));
}

String CSSLayerBlockRule::cssText() const
{
    StringBuilder builder;

    builder.append("@layer"_s);
    if (auto name = this->name(); !name.isEmpty())
        builder.append(' ', name);
    appendCSSTextForItems(builder);
    return builder.toString();
}

String CSSLayerBlockRule::name() const
{
    Ref layer = downcast<StyleRuleLayer>(groupRule());

    if (layer->name().isEmpty())
        return emptyString();

    return stringFromCascadeLayerName(layer->name());
}

String stringFromCascadeLayerName(const CascadeLayerName& name)
{
    StringBuilder result;
    for (auto& segment : name) {
        serializeIdentifier(result, segment);
        if (&segment != &name.last())
            result.append('.');
    }
    return result.toString();
}

} // namespace WebCore

````

## <a id="csslayerstatementrule-cpp"></a>CSSLayerStatementRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSLayerStatementRule.cpp). Path: `Source/WebCore/css/CSSLayerStatementRule.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 2650; source lines: 87; SHA-256: `7b2039cea01dd5409135a007adaec6e66e199bcf3bb791888d08442c017fd878`.

````cpp
/*
 * Copyright (C) 2021 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1. Redistributions of source code must retain the above
 *    copyright notice, this list of conditions and the following
 *    disclaimer.
 * 2. Redistributions in binary form must reproduce the above
 *    copyright notice, this list of conditions and the following
 *    disclaimer in the documentation and/or other materials
 *    provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDER "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY,
 * OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR
 * TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF
 * THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */

#include "config.h"
#include "CSSLayerStatementRule.h"

#include "CSSLayerBlockRule.h"
#include "CSSStyleSheet.h"
#include "StyleRule.h"
#include <wtf/text/StringBuilder.h>

namespace WebCore {

CSSLayerStatementRule::CSSLayerStatementRule(StyleRuleLayer& rule, CSSStyleSheet* parent)
    : CSSRule(parent)
    , m_layerRule(rule)
{
    ASSERT(rule.isStatement());
}

Ref<CSSLayerStatementRule> CSSLayerStatementRule::create(StyleRuleLayer& rule, CSSStyleSheet* parent)
{
    return adoptRef(*new CSSLayerStatementRule(rule, parent));
}

CSSLayerStatementRule::~CSSLayerStatementRule() = default;

String CSSLayerStatementRule::cssText() const
{
    StringBuilder result;

    result.append("@layer "_s);

    auto nameList = this->nameList();
    for (auto& name : nameList) {
        result.append(name);
        if (&name != &nameList.last())
            result.append(", "_s);
    }
    result.append(';');

    return result.toString();
}

Vector<String> CSSLayerStatementRule::nameList() const
{
    Vector<String> result;

    for (auto& name : m_layerRule.get().nameList())
        result.append(stringFromCascadeLayerName(name));

    return result;
}

void CSSLayerStatementRule::reattach(StyleRuleBase& rule)
{
    m_layerRule = downcast<StyleRuleLayer>(rule);
}

} // namespace WebCore

````

## <a id="csssupportsrule-cpp"></a>CSSSupportsRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSSupportsRule.cpp). Path: `Source/WebCore/css/CSSSupportsRule.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 2822; source lines: 77; SHA-256: `adcc1015637073ada0ad431c15a09e5b3e305f7e57df79b7a543ce6f55b87fc0`.

````cpp
/*
 * Copyright (C) 2012 Motorola Mobility Inc. All rights reserved.
 * Copyright (C) 2020 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above
 *    copyright notice, this list of conditions and the following disclaimer in
 *    the documentation and/or other materials provided with the distribution.
 * 3. Neither the name of Motorola Mobility Inc. nor the names of its
 *    contributors may be used to endorse or promote products derived from this
 *    software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "CSSSupportsRule.h"

#include "CSSStyleSheet.h"
#include "StyleProperties.h"
#include "StyleRule.h"
#include <wtf/text/StringBuilder.h>

namespace WebCore {

CSSSupportsRule::CSSSupportsRule(StyleRuleSupports& rule, CSSStyleSheet* parent)
    : CSSConditionRule(rule, parent)
{
}

Ref<CSSSupportsRule> CSSSupportsRule::create(StyleRuleSupports& rule, CSSStyleSheet* parent)
{
    return adoptRef(*new CSSSupportsRule(rule, parent));
}

String CSSSupportsRule::cssText() const
{
    StringBuilder builder;
    builder.append("@supports "_s, conditionText());
    appendCSSTextForItems(builder);
    return builder.toString();
}

String CSSSupportsRule::cssText(const CSS::SerializationContext& context) const
{
    StringBuilder builder;
    builder.append("@supports "_s, conditionText());
    appendCSSTextWithReplacementURLsForItems(builder, context);
    return builder.toString();
}

String CSSSupportsRule::conditionText() const
{
    return downcast<StyleRuleSupports>(groupRule()).conditionText();
}

bool CSSSupportsRule::matches() const
{
    return downcast<StyleRuleSupports>(groupRule()).conditionIsSupported();
}

} // namespace WebCore
````

## <a id="cssscoperule-cpp"></a>CSSScopeRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSScopeRule.cpp). Path: `Source/WebCore/css/CSSScopeRule.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 2598; source lines: 81; SHA-256: `2824c9f7a6298b5a963e85f3d68db09cace221ba5c3d0957bc078b9530f51d24`.

````cpp
/*
 * Copyright (C) 2023 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "CSSScopeRule.h"

#include "StyleRule.h"
#include <wtf/text/StringBuilder.h>

namespace WebCore {

CSSScopeRule::CSSScopeRule(StyleRuleScope& rule, CSSStyleSheet* parent)
    : CSSGroupingRule(rule, parent)
{
}

Ref<CSSScopeRule> CSSScopeRule::create(StyleRuleScope& rule, CSSStyleSheet* parent)
{
    return adoptRef(*new CSSScopeRule(rule, parent));
}

const StyleRuleScope& CSSScopeRule::styleRuleScope() const
{
    return downcast<StyleRuleScope>(groupRule());
}

String CSSScopeRule::cssText() const
{
    StringBuilder builder;
    builder.append("@scope"_s);
    auto start = this->start();
    if (!start.isEmpty())
        builder.append(" ("_s, start, ')');
    auto end = this->end();
    if (!end.isEmpty())
        builder.append(" to "_s, '(', end, ')');
    appendCSSTextForItems(builder);
    return builder.toString();
}

String CSSScopeRule::start() const
{
    auto& scope = styleRuleScope().originalScopeStart();
    if (scope.isEmpty())
        return { };

    return scope.selectorsText();
}

String CSSScopeRule::end() const
{
    auto& scope = styleRuleScope().originalScopeEnd();
    if (scope.isEmpty())
        return { };

    return scope.selectorsText();
}

} // namespace WebCore
````

## <a id="fontfeaturevalues-h"></a>FontFeatureValues.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontFeatureValues.h). Path: `Source/WebCore/platform/graphics/FontFeatureValues.h`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 4206; source lines: 150; SHA-256: `c8f1257cabc1812cf890b985199f519e88718708d5e67fe449d7186631dca137`.

````cpp
/*
 * Copyright (C) 2022 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <wtf/Hasher.h>
#include <wtf/OrderedHashMap.h>
#include <wtf/text/StringHash.h>
#include <wtf/text/WTFString.h>

namespace WebCore {

using FontFeatureValuesTag = std::pair<String, Vector<unsigned>>;

enum class FontFeatureValuesType {
    Styleset,
    Stylistic,
    CharacterVariant,
    Swash,
    Ornaments,
    Annotation
};

class FontFeatureValues : public RefCounted<FontFeatureValues> {
public:
    // Insertion-ordered per CSS OM spec: serialization must reproduce the
    // order in which `@<variant>` tags were declared in the stylesheet.
    using Tags = OrderedHashMap<String, Vector<unsigned>>;
    static Ref<FontFeatureValues> create() { return adoptRef(*new FontFeatureValues()); }
    virtual ~FontFeatureValues() = default;

    bool isEmpty() const
    {
        return m_styleset.isEmpty() 
            && m_stylistic.isEmpty() 
            && m_characterVariant.isEmpty() 
            && m_swash.isEmpty()
            && m_ornaments.isEmpty()
            && m_annotation.isEmpty();
    }
    
    bool operator==(const FontFeatureValues& other) const
    {
        return equalIgnoringOrder(m_styleset, other.styleset())
            && equalIgnoringOrder(m_stylistic, other.stylistic())
            && equalIgnoringOrder(m_characterVariant, other.characterVariant())
            && equalIgnoringOrder(m_swash, other.swash())
            && equalIgnoringOrder(m_ornaments, other.ornaments())
            && equalIgnoringOrder(m_annotation, other.annotation());
    }
    
    const Tags& styleset() const
    {
        return m_styleset;
    }

    Tags& styleset()
    {
        return m_styleset;
    }

    const Tags& stylistic() const
    {
        return m_stylistic;
    }

    Tags& stylistic()
    {
        return m_stylistic;
    }

    const Tags& characterVariant() const
    {
        return m_characterVariant;
    }

    Tags& characterVariant()
    {
        return m_characterVariant;
    }

    const Tags& swash() const
    {
        return m_swash;
    }

    Tags& swash()
    {
        return m_swash;
    }

    const Tags& ornaments() const
    {
        return m_ornaments;
    }

    Tags& ornaments()
    {
        return m_ornaments;
    }

    const Tags& annotation() const
    {
        return m_annotation;
    }

    Tags& annotation()
    {
        return m_annotation;
    }

    friend void add(Hasher&, const FontFeatureValues&);
    friend WTF::TextStream& operator<<(WTF::TextStream&, const FontFeatureValues&);
    void updateOrInsert(const FontFeatureValues&);
    void updateOrInsertForType(FontFeatureValuesType, const Vector<FontFeatureValuesTag>&);

private:
    FontFeatureValues() = default;

    Tags m_styleset;
    Tags m_stylistic;
    Tags m_characterVariant;
    Tags m_swash;
    Tags m_ornaments;
    Tags m_annotation;
};

} // namespace WebCore
````

## <a id="fontfeaturevalues-cpp"></a>FontFeatureValues.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontFeatureValues.cpp). Path: `Source/WebCore/platform/graphics/FontFeatureValues.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 4358; source lines: 114; SHA-256: `0a1e7b9c879248685d6a475441a719aecb8f1d55c0c59946372bd8bc775a5f73`.

````cpp
/*
 * Copyright (C) 2022 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "FontFeatureValues.h"

#include <wtf/text/TextStream.h>

namespace WebCore {

void add(Hasher& hasher, const FontFeatureValues& fontFeatureValues)
{
    auto hashTags = [&hasher](const auto& tags) {
        add(hasher, tags.isEmpty());
        for (const auto& tag : tags)
            add(hasher, tag.key, tag.value);
    };
    hashTags(fontFeatureValues.m_styleset);
    hashTags(fontFeatureValues.m_stylistic);
    hashTags(fontFeatureValues.m_characterVariant);
    hashTags(fontFeatureValues.m_swash);
    hashTags(fontFeatureValues.m_ornaments);
    hashTags(fontFeatureValues.m_annotation);
}

void FontFeatureValues::updateOrInsert(const FontFeatureValues& other)
{
    if (this == &other)
        return;
    
    auto updateOrInsertTags = [](auto& into, const auto& tags) {
        for (const auto& tag : tags)
            into.set(tag.key, tag.value);
    };
    updateOrInsertTags(m_styleset, other.styleset());
    updateOrInsertTags(m_stylistic, other.stylistic());
    updateOrInsertTags(m_characterVariant, other.characterVariant());
    updateOrInsertTags(m_swash, other.swash());
    updateOrInsertTags(m_ornaments, other.ornaments());
    updateOrInsertTags(m_annotation, other.annotation());
}

void FontFeatureValues::updateOrInsertForType(FontFeatureValuesType type, const Vector<FontFeatureValuesTag>& tags)
{
    auto updateOrInsertTags = [](auto& into, const Vector<FontFeatureValuesTag>& tags) {
        for (const FontFeatureValuesTag& tag : tags)
            into.set(tag.first, tag.second);
    };
    switch (type) {
    case FontFeatureValuesType::Styleset:
        updateOrInsertTags(m_styleset, tags);
        break;
    case FontFeatureValuesType::Stylistic:
        updateOrInsertTags(m_stylistic, tags);
        break;
    case FontFeatureValuesType::CharacterVariant:
        updateOrInsertTags(m_characterVariant, tags);
        break;
    case FontFeatureValuesType::Swash:
        updateOrInsertTags(m_swash, tags);
        break;
    case FontFeatureValuesType::Ornaments:
        updateOrInsertTags(m_ornaments, tags);
        break;
    case FontFeatureValuesType::Annotation:
        updateOrInsertTags(m_annotation, tags);
        break;
    }
    
}

WTF::TextStream& operator<<(WTF::TextStream& ts, const FontFeatureValues& fontFeatureValues)
{
    auto printTags = [&ts](const auto& name, const auto& tags) {
        if (tags.isEmpty())
            return;

        ts << '{' << name << ": "_s;
        for (const auto& tag : tags)
            ts << '{' << tag.key << ':' << tag.value << "} "_s;
        ts << '}';
    };
    printTags("styleset", fontFeatureValues.m_styleset);
    printTags("stylistic", fontFeatureValues.m_stylistic);
    printTags("characterVariant", fontFeatureValues.m_characterVariant);
    printTags("swash", fontFeatureValues.m_swash);
    printTags("ornaments", fontFeatureValues.m_ornaments);
    printTags("annotation", fontFeatureValues.m_annotation);
    return ts;  
}

} // namespace WebCore
````

## <a id="cssatruleid-h"></a>CSSAtRuleID.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSAtRuleID.h). Path: `Source/WebCore/css/parser/CSSAtRuleID.h`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 2483; source lines: 73; SHA-256: `513fd46453ad215eebf3a43451865a5baf1a7d0d3a7f76e9c41bbeb1dc23eb95`.

````cpp
// Copyright 2015 The Chromium Authors. All rights reserved.
// Copyright (C) 2016-2021 Apple Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//    * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#pragma once

#include <wtf/text/StringView.h>

namespace WebCore {

enum CSSAtRuleID : uint8_t {
    CSSAtRuleInvalid = 0,

    CSSAtRuleCharset,
    CSSAtRuleFontFace,
    CSSAtRuleImport,
    CSSAtRuleKeyframes,
    CSSAtRuleMedia,
    CSSAtRuleNamespace,
    CSSAtRulePage,
    CSSAtRulePositionTry,
    CSSAtRuleSupports,
    CSSAtRuleViewTransition,

    CSSAtRuleWebkitKeyframes,
    CSSAtRuleCounterStyle,
    CSSAtRuleLayer,
    CSSAtRuleContainer,
    CSSAtRuleProperty,

    CSSAtRuleFontFeatureValues,
    CSSAtRuleStylistic,
    CSSAtRuleStyleset,
    CSSAtRuleCharacterVariant,
    CSSAtRuleSwash,
    CSSAtRuleOrnaments,
    CSSAtRuleAnnotation,

    CSSAtRuleFontPaletteValues,
    CSSAtRuleScope,
    CSSAtRuleStartingStyle,
    CSSAtRuleFunction,
    CSSAtRuleEnvironmentMap,
};

CSSAtRuleID cssAtRuleID(StringView name);

} // namespace WebCore
````

## <a id="cssatruleid-cpp"></a>CSSAtRuleID.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSAtRuleID.cpp). Path: `Source/WebCore/css/parser/CSSAtRuleID.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 3575; source lines: 72; SHA-256: `75b617c09371237044d852cb1371523bd7c57227d8958e5f0285cc267792235e`.

````cpp
// Copyright 2015 The Chromium Authors. All rights reserved.
// Copyright (C) 2016-2022 Apple Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//    * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#include "config.h"
#include "CSSAtRuleID.h"

#include <wtf/SortedArrayMap.h>

namespace WebCore {

CSSAtRuleID cssAtRuleID(StringView name)
{
    static constexpr SortedArrayMap cssAtRules { WTF::toArray<std::pair<ComparableLettersLiteral, CSSAtRuleID>>({
        { "-webkit-keyframes"_s,     CSSAtRuleWebkitKeyframes },
        { "annotation"_s,            CSSAtRuleAnnotation },
        { "character-variant"_s,     CSSAtRuleCharacterVariant },
        { "charset"_s,               CSSAtRuleCharset },
        { "container"_s,             CSSAtRuleContainer },
        { "counter-style"_s,         CSSAtRuleCounterStyle },
        { "environment-map"_s,       CSSAtRuleEnvironmentMap },
        { "font-face"_s,             CSSAtRuleFontFace },
        { "font-feature-values"_s,   CSSAtRuleFontFeatureValues },
        { "font-palette-values"_s,   CSSAtRuleFontPaletteValues },
        { "function"_s,              CSSAtRuleFunction },
        { "import"_s,                CSSAtRuleImport },
        { "keyframes"_s,             CSSAtRuleKeyframes },
        { "layer"_s,                 CSSAtRuleLayer },
        { "media"_s,                 CSSAtRuleMedia },
        { "namespace"_s,             CSSAtRuleNamespace },
        { "ornaments"_s,             CSSAtRuleOrnaments },
        { "page"_s,                  CSSAtRulePage },
        { "position-try"_s,          CSSAtRulePositionTry },
        { "property"_s,              CSSAtRuleProperty },
        { "scope"_s,                 CSSAtRuleScope },
        { "starting-style"_s,        CSSAtRuleStartingStyle },
        { "styleset"_s,              CSSAtRuleStyleset },
        { "stylistic"_s,             CSSAtRuleStylistic },
        { "supports"_s,              CSSAtRuleSupports },
        { "swash"_s,                 CSSAtRuleSwash },
        { "view-transition"_s,       CSSAtRuleViewTransition },
    }) };
    return cssAtRules.get(name, CSSAtRuleInvalid);
}

} // namespace WebCore

````

## <a id="newer-csscounterstylerule-h"></a>CSSCounterStyleRule.h (newer revision)

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/css/CSSCounterStyleRule.h). Path: `Source/WebCore/css/CSSCounterStyleRule.h`. Revision: `437139e30888c5e1b4c35f889f32e1d0c39d018e`. Source bytes: 5617; source lines: 120; SHA-256: `66a2906a7799f7874fd6d9feed27de4a6aaf700affee2bf6b099603d91048b75`.

````cpp
/*
 * Copyright (C) 2021 Tyler Wilcock <twilco.o@protonmail.com>.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include "CSSCounterStyleDescriptors.h"
#include "CSSRule.h"
#include "StyleProperties.h"
#include "StyleRule.h"
#include <wtf/text/AtomString.h>

namespace WebCore {
class StyleRuleCounterStyle final : public StyleRuleBase {
public:
    static Ref<StyleRuleCounterStyle> create(const AtomString&, CSSCounterStyleDescriptors&&);
    ~StyleRuleCounterStyle();

    Ref<StyleRuleCounterStyle> copy() const { return adoptRef(*new StyleRuleCounterStyle(*this)); }

    const CSSCounterStyleDescriptors& descriptors() const LIFETIME_BOUND { return m_descriptors; };
    CSSCounterStyleDescriptors& mutableDescriptors() LIFETIME_BOUND { return m_descriptors; };

    const AtomString& name() const LIFETIME_BOUND { return m_name; }
    String system() const { return m_descriptors.systemCSSText(); }
    String negative() const { return m_descriptors.negativeCSSText(); }
    String prefix() const { return m_descriptors.prefixCSSText(); }
    String suffix() const { return m_descriptors.suffixCSSText(); }
    String range() const { return { m_descriptors.rangesCSSText() }; }
    String pad() const { return m_descriptors.padCSSText(); }
    String fallback() const { return m_descriptors.fallbackCSSText(); }
    String symbols() const { return m_descriptors.symbolsCSSText(); }
    String additiveSymbols() const { return m_descriptors.additiveSymbolsCSSText(); }
    String speakAs() const { return { }; }
    bool newValueInvalidOrEqual(CSSPropertyID, const RefPtr<CSSValue> newValue) const;

    void setName(const AtomString& name) { m_name = name; }

private:
    explicit StyleRuleCounterStyle(const AtomString&, CSSCounterStyleDescriptors&&);
    StyleRuleCounterStyle(const StyleRuleCounterStyle&) = default;

    AtomString m_name;
    CSSCounterStyleDescriptors m_descriptors;
};

class CSSCounterStyleRule final : public CSSRule {
public:
    static Ref<CSSCounterStyleRule> create(StyleRuleCounterStyle&, CSSStyleSheet*);
    virtual ~CSSCounterStyleRule();

    String cssText() const final;
    void NODELETE reattach(StyleRuleBase&) final;
    StyleRuleType styleRuleType() const final { return StyleRuleType::CounterStyle; }

    String name() const { return m_counterStyleRule->name(); }
    String system() const { return protect(m_counterStyleRule)->system(); }
    String negative() const { return protect(m_counterStyleRule)->negative(); }
    String prefix() const { return protect(m_counterStyleRule)->prefix(); }
    String suffix() const { return protect(m_counterStyleRule)->suffix(); }
    String range() const { return protect(m_counterStyleRule)->range(); }
    String pad() const { return protect(m_counterStyleRule)->pad(); }
    String fallback() const { return protect(m_counterStyleRule)->fallback(); }
    String symbols() const { return protect(m_counterStyleRule)->symbols(); }
    String additiveSymbols() const { return protect(m_counterStyleRule)->additiveSymbols(); }
    String speakAs() const { return m_counterStyleRule->speakAs(); }

    void setName(const String&);
    void setSystem(const String&);
    void setNegative(const String&);
    void setPrefix(const String&);
    void setSuffix(const String&);
    void setRange(const String&);
    void setPad(const String&);
    void setFallback(const String&);
    void setSymbols(const String&);
    void setAdditiveSymbols(const String&);
    void NODELETE setSpeakAs(const String&);

private:
    CSSCounterStyleRule(StyleRuleCounterStyle&, CSSStyleSheet* parent);

    bool setterInternal(CSSPropertyID, const String&);
    RefPtr<CSSValue> cssValueFromText(CSSPropertyID, const String&);
    const CSSCounterStyleDescriptors& descriptors() const LIFETIME_BOUND { return m_counterStyleRule->descriptors(); }
    CSSCounterStyleDescriptors& mutableDescriptors() LIFETIME_BOUND { return m_counterStyleRule->mutableDescriptors(); }

    Ref<StyleRuleCounterStyle> m_counterStyleRule;
};

CSSCounterStyleDescriptors::System NODELETE toCounterStyleSystemEnum(const CSSValue*);

} // namespace WebCore

SPECIALIZE_TYPE_TRAITS_CSS_RULE(CSSCounterStyleRule, StyleRuleType::CounterStyle)

SPECIALIZE_TYPE_TRAITS_BEGIN(WebCore::StyleRuleCounterStyle)
static bool isType(const WebCore::StyleRuleBase& rule) { return rule.isCounterStyleRule(); }
SPECIALIZE_TYPE_TRAITS_END()
````
