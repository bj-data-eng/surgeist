# Pinned browser witnesses for style source determinations

Retrieved on 2026-10-10 from local immutable WebKit objects and official Blink Gitiles. This bounded source witness is implementation evidence, not a standard or whole-engine compatibility claim. Source segments preserve their original bytes and complete file-level legal headers. No fetched source was compiled or executed. File-specific notices and exact hashes below accompany the retained material; [WebKit legal material](../licenses/webkit/NOTICE.md) and the [Chromium BSD license](../licenses/chromium/LICENSE.txt) remain separate.

WebKit provides style-time environment substitution and pending shorthand reparse, but the retained env function explicitly does not support indexed names. Blink provides indexed environment lookup through its style cascade and deferred shorthand reparse. Both inspected size-query paths use supported and effective size-containment axes rather than an independent layout-containment requirement. WebKit’s dark appearance functions are a limited setting-dependent interpretation; they do not implement the current standard’s entire user-preference hierarchy or settle the emulation getter signature. Contain’s model includes style and inline-size and expands strict/content internally; its switchOn may re-form strict/content, so this witness does not establish expanded computed serialization.

## <a id="webkit-document-cpp"></a>Webkit: Source/WebCore/dom/Document.cpp

Immutable source: [Source/WebCore/dom/Document.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Document.cpp). Full original SHA-256: `c3f64c3d98f23601ffa74b2e63b82f30aaeb4035bb537c657475a8c00095c55c` (464007 bytes). Retained scope: lines 1–27: complete legal header and original leading comments; lines 10118–10122: complete OptionSet<ColorScheme> Document::resolvedColorScheme; lines 10125–10149: complete bool Document::useDarkAppearance. All unlisted original source is omitted.

complete legal header and original leading comments:

````cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 *           (C) 2006 Alexey Proskuryakov (ap@webkit.org)
 * Copyright (C) 2004-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2008, 2009 Torch Mobile Inc. All rights reserved. (http://www.torchmobile.com/)
 * Copyright (C) 2008-2014 Google Inc. All rights reserved.
 * Copyright (C) 2010 Nokia Corporation and/or its subsidiary(-ies)
 * Copyright (C) Research In Motion Limited 2010-2011. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */

````

complete OptionSet<ColorScheme> Document::resolvedColorScheme:

````cpp
OptionSet<ColorScheme> Document::resolvedColorScheme(const Style::ComputedStyle* style) const
{
    bool isNormal = !style || style->colorScheme().isNormal();
    return isNormal ? m_colorScheme : style->colorScheme().colorScheme();
}
````

complete bool Document::useDarkAppearance:

````cpp
bool Document::useDarkAppearance([[maybe_unused]] const Style::ComputedStyle* style) const
{
#if ENABLE(DARK_MODE_CSS)
    auto colorScheme = resolvedColorScheme(style);

    if (colorScheme.contains(ColorScheme::Dark) && !colorScheme.contains(ColorScheme::Light))
        return true;
#else
    UNUSED_PARAM(style);
#endif

    bool pageUsesDarkAppearance = false;
    if (RefPtr documentPage = page())
        pageUsesDarkAppearance = documentPage->useDarkAppearance();

    if (settings().useSystemAppearance())
        return pageUsesDarkAppearance;

#if ENABLE(DARK_MODE_CSS)
    if (colorScheme.contains(ColorScheme::Dark))
        return pageUsesDarkAppearance;
#endif

    return false;
}
````

## <a id="webkit-renderelement-cpp"></a>Webkit: Source/WebCore/rendering/RenderElement.cpp

Immutable source: [Source/WebCore/rendering/RenderElement.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderElement.cpp). Full original SHA-256: `af95d006473143f8c9deaef39df301080f6fe33f25e515d7b8a870e617b44b41` (129139 bytes). Retained scope: lines 1–24: complete legal header and original leading comments; lines 2884–2896: complete bool RenderElement::hasEligibleContainmentForSizeQuery. All unlisted original source is omitted.

complete legal header and original leading comments:

````cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2005 Allan Sandfeld Jensen (kde@carewolf.com)
 * Copyright (C) 2005-2026 Samuel Weinig (sam@webkit.org)
 * Copyright (C) 2005-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2010-2018 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */

````

complete bool RenderElement::hasEligibleContainmentForSizeQuery:

````cpp
bool RenderElement::hasEligibleContainmentForSizeQuery() const
{
    auto& type = style().containerType();
    if (type.hasSize())
        return shouldApplySizeContainment();
    if (type.hasInlineSize())
        return shouldApplyInlineSizeContainment();
    // `scroll-state` (without a size axis) establishes no size containment, same as `normal`.
    if (type.isNormal() || type.hasScrollState())
        return true;
    ASSERT_NOT_REACHED();
    return false;
}
````

## <a id="webkit-containerqueryevaluator-cpp"></a>Webkit: Source/WebCore/style/ContainerQueryEvaluator.cpp

Immutable source: [Source/WebCore/style/ContainerQueryEvaluator.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/ContainerQueryEvaluator.cpp). Full original SHA-256: `b63ca26b4bf0d0f6d36af413b15069ebcdc729c44431279af6309c6aea14b901` (11146 bytes). Retained scope: lines 1–24: complete legal header and original leading comments; lines 142–243: complete RefPtr<const Element> ContainerQueryEvaluator::selectContainer. All unlisted original source is omitted.

complete legal header and original leading comments:

````cpp
/*
 * Copyright (C) 2022 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1.  Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 * 2.  Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in the
 *     documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
 * ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

````

complete RefPtr<const Element> ContainerQueryEvaluator::selectContainer:

````cpp
RefPtr<const Element> ContainerQueryEvaluator::selectContainer(CQ::ContainerRequirements requirements, const WTF::String& name, const Element& element, SelectionMode selectionMode, ScopeOrdinal scopeOrdinal, const ContainerQueryEvaluationState* evaluationState)
{
    // "For each element, the query container to be queried is selected from among the element’s
    // ancestor query containers that have a valid container-type for all the container features
    // in the <container-condition>. The optional <container-name> filters the set of query containers
    // considered to just those with a matching query container name."
    // https://drafts.csswg.org/css-contain-3/#container-rule

    auto isValidContainer = [&](const Style::ContainerType& containerType, const RenderElement* principalBox) {
        // A scroll-state query requires a scroll-state container.
        if (requirements.scrollState && !containerType.hasScrollState())
            return false;

        // No size container required: any container is valid (style query), or the
        // scroll-state requirement above has already been satisfied.
        if (!requirements.needsSizeContainer())
            return true;

        if (containerType.hasSize())
            return true;
        if (containerType.hasInlineSize()) {
            // Without a principal box the container matches but the query against it will evaluate to Unknown.
            if (!principalBox)
                return true;
            if (requirements.sizeAxes.contains(CQ::Axis::Block))
                return false;
            return !requirements.sizeAxes.contains(principalBox->isHorizontalWritingMode() ? CQ::Axis::Height : CQ::Axis::Width);
        }
        // A normal container, or a scroll-state-only container, provides no size
        // containment and so is not a valid container for a size query.
        if (containerType.isNormal() || containerType.hasScrollState())
            return false;
        RELEASE_ASSERT_NOT_REACHED();
    };

    auto isContainerForQuery = [&](const Element& candidateElement, const Element* originatingElement = nullptr) {
        CheckedPtr style = styleForContainer(candidateElement, requirements, evaluationState);
        if (!style)
            return false;
        if (!isValidContainer(style->containerType(), candidateElement.renderer()))
            return false;
        if (name.isEmpty())
            return true;

        return style->containerNames().containsIf([&](auto& scopedName) {
            auto isNameFromAllowedScope = [&](auto& scopedName) {
                // Names from :host rules are allowed when the candidate is the host element.
                RefPtr host = originatingElement ? originatingElement->shadowHost() : element.shadowHost();
                auto isHost = host == &candidateElement;
                if (scopedName.scopeOrdinal == ScopeOrdinal::Shadow && isHost)
                    return true;
                // Otherwise names from the inner scopes are ignored.
                return scopedName.scopeOrdinal <= ScopeOrdinal::Element;
            };
            return isNameFromAllowedScope(scopedName) && scopedName.name == name;
        });
    };

    auto findOriginatingElement = [&]() -> RefPtr<const Element> {
        if (selectionMode == SelectionMode::PseudoElement)
            return &element;

        // ::part() selectors query the composed tree
        if (selectionMode == SelectionMode::PartPseudoElement)
            return element;

        // ::slotted() selectors can query containers inside the shadow tree, including the slot itself.
        if (scopeOrdinal >= ScopeOrdinal::FirstSlot && scopeOrdinal <= ScopeOrdinal::SlotLimit)
            return assignedSlotForScopeOrdinal(element, scopeOrdinal);

        if (scopeOrdinal == ScopeOrdinal::Element && element.assignedSlot())
            return element.assignedSlot();

        return nullptr;
    };

    if (RefPtr originatingElement = findOriginatingElement()) {
        // For the ::part() and ::slotted() pseudo-element selectors, which represent real elements in the DOM tree,
        // query containers can be established by flat tree ancestors of those elements.
        // For other pseudo-elements, query containers can be established by inclusive flat tree ancestors of their originating element.
        // https://drafts.csswg.org/css-conditional-5/#container-queries
        for (Ref ancestor : composedTreeLineage(*originatingElement)) {
            if (isContainerForQuery(ancestor, originatingElement.get()))
                return ancestor.ptr();
        }
        return nullptr;
    }

    if (evaluationState && requirements.needsSizeContainer()) {
        for (auto& container : evaluationState->sizeQueryContainers | std::views::reverse) {
            if (isContainerForQuery(container))
                return container.ptr();
        }
        return { };
    }

    for (Ref ancestor : composedTreeAncestors(element)) {
        if (isContainerForQuery(ancestor))
            return ancestor.ptr();
    }
    return { };
}
````

## <a id="webkit-stylesubstitutionresolver-cpp"></a>Webkit: Source/WebCore/style/StyleSubstitutionResolver.cpp

Immutable source: [Source/WebCore/style/StyleSubstitutionResolver.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/StyleSubstitutionResolver.cpp). Full original SHA-256: `3e74556b756e1731a3d2fd02d05a70d718c752b45fbabe74bc2368a391c1233e` (65039 bytes). Retained scope: lines 1–25: complete legal header and original leading comments; lines 128–145: complete RefPtr<const CustomProperty> SubstitutionResolver::propertyValueForVariableName; lines 213–234: complete bool SubstitutionResolver::substituteEnvFunction; lines 196–211: complete bool SubstitutionResolver::substituteVarFunction; lines 236–277: complete bool SubstitutionResolver::substituteNamedValueOrFallback; lines 1366–1387: complete RefPtr<CSSValue> SubstitutionResolver::substituteAndParse(; lines 1389–1420: complete RefPtr<CSSValue> SubstitutionResolver::substituteAndParseShorthand. All unlisted original source is omitted.

complete legal header and original leading comments:

````cpp
/*
 * Copyright (C) 2026 Apple Inc. All rights reserved.
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

````

complete RefPtr<const CustomProperty> SubstitutionResolver::propertyValueForVariableName:

````cpp
RefPtr<const CustomProperty> SubstitutionResolver::propertyValueForVariableName(const AtomString& variableName, CSSValueID functionId)
{
    if (functionId == CSSValueEnv)
        return m_styleBuilder.state().document().styleScope().environmentVariables().valueForName(variableName);

    // Every parameter of the function whose arguments are being resolved shadows the calling element's
    // custom property of the same name, resolved or not, so a default can reference an earlier
    // parameter but never sees the calling element's value of a later one.
    if (!m_parameterValues.isEmpty() && functionId == CSSValueVar) {
        if (auto parameter = m_parameterValues.last().getOptional(variableName))
            return *parameter;
    }

    // Apply this variable first, in case it is still unresolved
    m_styleBuilder.applyCustomProperty(variableName);

    return protect(m_styleBuilder.state().style())->customPropertyValue(variableName);
}
````

complete bool SubstitutionResolver::substituteEnvFunction:

````cpp
bool SubstitutionResolver::substituteEnvFunction(CSSParserTokenRange range, Vector<CSSParserToken>& tokens, const CSSParserContext& context)
{
    // https://drafts.csswg.org/css-env-1/#env-function
    // FIXME: env()'s argument grammar is env( <declaration-value>, <declaration-value>? ) like var()'s,
    // so the name argument should be substituted and only then parsed as <custom-ident> <integer>*.
    // Only a literal <ident> name is supported here, the same limitation as the unsupported indices.
    range.consumeWhitespace();
    if (range.peek().type() != IdentToken)
        return false;
    auto name = range.consumeIncludingWhitespace().value().toAtomString();

    std::optional<CSSParserTokenRange> fallbackRange;
    if (!range.atEnd()) {
        if (range.peek().type() != CommaToken)
            return false;
        range.consumeIncludingWhitespace();
        range.trimTrailingWhitespace();
        fallbackRange = range;
    }

    return substituteNamedValueOrFallback(name, fallbackRange, CSSValueEnv, tokens, context);
}
````

complete bool SubstitutionResolver::substituteVarFunction:

````cpp
bool SubstitutionResolver::substituteVarFunction(CSSParserTokenRange range, Vector<CSSParserToken>& tokens, const CSSParserContext& context)
{
    // https://drafts.csswg.org/css-variables-2/#replace-a-var-function
    auto arguments = substituteVarArgumentGrammar(range, context);

    auto startIndex = tokens.size();
    if (!substituteNamedValueOrFallback(arguments.name, arguments.fallbackRange, CSSValueVar, tokens, context))
        return false;

    // https://drafts.csswg.org/css-values-5/#attr-security
    // A name argument resolved from attr()-tainted values taints the whole substitution value. The
    // URL to catch is in the substituted tokens, not in the name.
    propagateAttrTaint(arguments.isNameAttrTainted, std::span(tokens).subspan(startIndex));

    return true;
}
````

complete bool SubstitutionResolver::substituteNamedValueOrFallback:

````cpp
bool SubstitutionResolver::substituteNamedValueOrFallback(const std::optional<AtomString>& name, const std::optional<CSSParserTokenRange>& fallbackRange, CSSValueID functionId, Vector<CSSParserToken>& tokens, const CSSParserContext& context)
{
    ASSERT(functionId == CSSValueVar || functionId == CSSValueEnv);

    // A name that failed to parse leaves the reference guaranteed-invalid, which still permits the fallback.
    RefPtr property = name ? propertyValueForVariableName(*name, functionId) : nullptr;

    if (property && !property->isGuaranteedInvalid()) {
        if (property->tokens().size() > maxSubstitutionTokens)
            return false;

        // https://drafts.csswg.org/css-values-5/#attr-security
        // Propagate attr()-taint through var() references.
        propagateAttrTaint(property->isAttrTainted(), property->tokens());

        tokens.appendVector(property->tokens());
        return true;
    }

    if (!fallbackRange)
        return false;

    auto fallbackTokens = substituteTokenRange(*fallbackRange, context);
    if (!fallbackTokens || fallbackTokens->size() > maxSubstitutionTokens)
        return false;

    if (functionId == CSSValueVar && name) {
        auto* registered = m_styleBuilder.state().registeredProperty(*name);
        // A custom function's parameters and locals are registered to carry their type, but they are
        // not author registrations, so a fallback for one is not held to its syntax.
        auto* localRegistry = m_styleBuilder.state().localPropertyRegistry();
        bool isLocalRegistration = localRegistry && localRegistry->get(*name);
        // https://drafts.css-houdini.org/css-properties-values-api/#fallbacks-in-var-references
        // A used fallback must match the referenced registered property's syntax.
        if (registered && !isLocalRegistration && !registered->syntax.isUniversal()
            && !CSSPropertyParser::isValidCustomPropertyValueForSyntax(registered->syntax, *fallbackTokens, context))
            return false;
    }

    tokens.appendVector(*fallbackTokens);
    return true;
}
````

complete RefPtr<CSSValue> SubstitutionResolver::substituteAndParse(:

````cpp
RefPtr<CSSValue> SubstitutionResolver::substituteAndParse(const CSSSubstitutionValue& substitutionValue, CSSPropertyID propertyID)
{
    auto data = substitute(substitutionValue);
    if (!data)
        return nullptr;

    // https://drafts.csswg.org/css-values-5/#attr-security
    // Using an attr()-tainted value as or in a <url> makes a declaration invalid at computed-value time.
    if (propertyID != CSSPropertyCustom && m_hasTaintedURL)
        return nullptr;

    if (!arePointingToEqualData(substitutionValue.m_cache.dependencyData, data) || substitutionValue.m_cache.propertyID != propertyID) {
        substitutionValue.m_cache.value = CSSPropertyParser::parseStylePropertyLonghand(propertyID, data->tokens(), substitutionValue.context());
        substitutionValue.m_cache.propertyID = propertyID;
    }
    substitutionValue.m_cache.dependencyData = WTF::move(data);

    if (substitutionValue.m_simpleReference && substitutionValue.m_simpleReference->functionId == CSSValueInternalAutoBase)
        substitutionValue.m_cache.isBaseAppearance = isBaseAppearance();

    return substitutionValue.m_cache.value;
}
````

complete RefPtr<CSSValue> SubstitutionResolver::substituteAndParseShorthand:

````cpp
RefPtr<CSSValue> SubstitutionResolver::substituteAndParseShorthand(const CSSShorthandSubstitutionValue& substitution, CSSPropertyID propertyID)
{
    ASSERT(!CSSProperty::isDirectionAwareProperty(propertyID));

    auto& substitutionValue = substitution.shorthandValue();

    auto data = substitute(substitutionValue);
    if (!data)
        return nullptr;

    if (m_hasTaintedURL)
        return nullptr;

    if (!arePointingToEqualData(substitutionValue.m_cache.dependencyData, data)) {
        ParsedPropertyVector parsedProperties;
        if (!CSSPropertyParser::parseValue(substitution.m_shorthandPropertyId, IsImportant::No, data->tokens(), data->context(), parsedProperties, StyleRuleType::Style))
            substitution.m_cachedPropertyValues = { };
        else
            substitution.m_cachedPropertyValues = parsedProperties;
    }
    substitutionValue.m_cache.dependencyData = WTF::move(data);

    if (substitutionValue.m_simpleReference && substitutionValue.m_simpleReference->functionId == CSSValueInternalAutoBase)
        substitutionValue.m_cache.isBaseAppearance = isBaseAppearance();

    for (auto& property : substitution.m_cachedPropertyValues) {
        if (CSSProperty::resolveDirectionAwareProperty(property.id(), m_styleBuilder.state().style().writingMode()) == propertyID)
            return property.value();
    }

    return nullptr;
}
````

## <a id="webkit-stylecontain-h"></a>Webkit: Source/WebCore/style/values/contain/StyleContain.h

Immutable source: [Source/WebCore/style/values/contain/StyleContain.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/values/contain/StyleContain.h). Full original SHA-256: `cb897840277880b6cff31ea9686f324c69f66a8c880ccbb61c75657c33c89270` (4187 bytes). Retained scope: lines 1–104: complete file. All unlisted original source is omitted.

complete file:

````cpp
/*
 * Copyright (C) 2025 Samuel Weinig <sam@webkit.org>
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
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <WebCore/StyleValueTypes.h>

namespace WebCore {
namespace Style {

// <'contain'> = none | strict@(aliased-to=[size layout paint style]) | content@(aliased-to=[layout paint style]) | [ [size | inline-size] || layout || style || paint ]
// https://drafts.csswg.org/css-contain-2/#contain-property

enum class ContainValue : uint8_t {
    Size,
    InlineSize,
    Layout,
    Style,
    Paint,
};

using ContainValueEnumSet = SpaceSeparatedEnumSet<ContainValue>;

struct Contain {
    using EnumSet = ContainValueEnumSet;
    using value_type = ContainValueEnumSet::value_type;

    static constexpr ContainValueEnumSet strict { ContainValue::Size, ContainValue::Layout, ContainValue::Style, ContainValue::Paint };
    static constexpr ContainValueEnumSet content { ContainValue::Layout, ContainValue::Style, ContainValue::Paint };

    constexpr Contain(CSS::Keyword::None) : m_value { } { }
    constexpr Contain(CSS::Keyword::Strict) : m_value { strict } { }
    constexpr Contain(CSS::Keyword::Content) : m_value { content } { }
    constexpr Contain(EnumSet&& set) : m_value { WTF::move(set) } { }
    constexpr Contain(value_type value) : Contain { EnumSet { value } } { }
    constexpr Contain(std::initializer_list<value_type> initializerList) : Contain { EnumSet { initializerList } } { }

    static constexpr Contain fromRaw(EnumSet::StorageType rawValue)
    {
        if (!rawValue)
            return CSS::Keyword::None { };
        return EnumSet::fromRaw(rawValue);
    }
    constexpr EnumSet::StorageType toRaw() const { return m_value.toRaw(); }

    constexpr bool contains(value_type e) const { return m_value.contains(e); }
    constexpr bool containsAny(EnumSet other) const { return m_value.containsAny(other.value); }
    constexpr bool containsAll(EnumSet other) const { return m_value.containsAll(other.value); }
    constexpr bool containsOnly(EnumSet other) const { return m_value.containsOnly(other.value); }
    constexpr void add(EnumSet other) { m_value.value.add(other.value); }

    constexpr bool isNone() const { return m_value.isEmpty(); }

    template<typename... F> constexpr decltype(auto) switchOn(F&&... f) const
    {
        auto visitor = WTF::makeVisitor(std::forward<F>(f)...);

        if (isNone())
            return visitor(CSS::Keyword::None { });

        // Handle "strict" and "content" shorthands
        if (m_value == strict)
            return visitor(CSS::Keyword::Strict { });
        if (m_value == content)
            return visitor(CSS::Keyword::Content { });

        return visitor(m_value);
    }

    constexpr bool operator==(const Contain&) const = default;

private:
    EnumSet m_value;
};

// MARK: - Conversion

template<> struct CSSValueConversion<Contain> { auto operator()(BuilderState&, const CSSValue&) -> Contain; };

} // namespace Style
} // namespace WebCore

DEFINE_VARIANT_LIKE_CONFORMANCE(WebCore::Style::Contain)
````

## <a id="webkit-stylecontain-cpp"></a>Webkit: Source/WebCore/style/values/contain/StyleContain.cpp

Immutable source: [Source/WebCore/style/values/contain/StyleContain.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/values/contain/StyleContain.cpp). Full original SHA-256: `60dd625afd90d6af6d1ff5e5205e262a2954d8d16f3e9ececb3e507d4d89975f` (3683 bytes). Retained scope: lines 1–99: complete file. All unlisted original source is omitted.

complete file:

````cpp
/*
 * Copyright (C) 2025 Samuel Weinig <sam@webkit.org>
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
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "StyleContain.h"

#include "CSSKeywordValue.h"
#include "StyleBuilderChecking.h"

namespace WebCore {
namespace Style {

auto CSSValueConversion<Contain>::operator()(BuilderState& state, const CSSValue& value) -> Contain
{
    if (RefPtr keywordValue = dynamicDowncast<CSSKeywordValue>(value)) {
        switch (keywordValue->valueID()) {
        case CSSValueNone:
            return CSS::Keyword::None { };
        case CSSValueStrict:
            return CSS::Keyword::Strict { };
        case CSSValueContent:
            return CSS::Keyword::Content { };
        case CSSValueSize:
            return { ContainValue::Size };
        case CSSValueInlineSize:
            return { ContainValue::InlineSize };
        case CSSValueLayout:
            return { ContainValue::Layout };
        case CSSValueStyle:
            return { ContainValue::Style };
        case CSSValuePaint:
            return { ContainValue::Paint };
        default:
            state.setCurrentPropertyInvalidAtComputedValueTime();
            return CSS::Keyword::None { };
        }
    }

    auto list = requiredListDowncast<CSSValueList, CSSKeywordValue>(state, value);
    if (!list)
        return CSS::Keyword::None { };

    ContainValueEnumSet result;
    for (auto& item : *list) {
        switch (item.valueID()) {
        case CSSValueSize:
            if (result.contains(ContainValue::InlineSize)) {
                state.setCurrentPropertyInvalidAtComputedValueTime();
                return CSS::Keyword::None { };
            }
            result.value.add(ContainValue::Size);
            break;
        case CSSValueInlineSize:
            if (result.contains(ContainValue::Size)) {
                state.setCurrentPropertyInvalidAtComputedValueTime();
                return CSS::Keyword::None { };
            }
            result.value.add(ContainValue::InlineSize);
            break;
        case CSSValueLayout:
            result.value.add(ContainValue::Layout);
            break;
        case CSSValueStyle:
            result.value.add(ContainValue::Style);
            break;
        case CSSValuePaint:
            result.value.add(ContainValue::Paint);
            break;
        default:
            state.setCurrentPropertyInvalidAtComputedValueTime();
            return CSS::Keyword::None { };
        }
    }
    return result;
}

} // namespace Style
} // namespace WebCore
````

## <a id="webkit-stylecontainmentcheckerinlines-h"></a>Webkit: Source/WebCore/style/StyleContainmentCheckerInlines.h

Immutable source: [Source/WebCore/style/StyleContainmentCheckerInlines.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/StyleContainmentCheckerInlines.h). Full original SHA-256: `68afcfd975c6ea6b2538560fed7293ce85e04d1f33e9795f1eb7379144026d81` (7965 bytes). Retained scope: lines 1–187: complete file. All unlisted original source is omitted.

complete file:

````cpp
/*
 * Copyright (C) 2026 Samuel Weinig <sam@webkit.org>
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

#include <WebCore/Element.h>
#include <WebCore/NodeName.h>
#include <WebCore/StyleComputedStyle+GettersInlines.h>
#include <WebCore/StyleContainmentChecker.h>

namespace WebCore {
namespace Style {

inline ContainmentChecker::ContainmentChecker(const Style::ComputedStyle& style, const Element& element)
    : m_style { style }
    , m_element { element }
{
}

inline ContainmentChecker::~ContainmentChecker() = default;

inline bool ContainmentChecker::shouldApplyLayoutContainment() const
{
    // https://drafts.csswg.org/css-contain/#containment-layout

    // content-visibility hidden and auto turns on layout containment.
    auto hasContainment = m_style->usedContain().contains(ContainValue::Layout)
        || m_style->contentVisibility() == ContentVisibility::Hidden
        || m_style->contentVisibility() == ContentVisibility::Auto;
    if (!hasContainment)
        return false;

    // Giving an element layout containment has no effect if any of the following are true:
    //   if the element does not generate a principal box (as is the case with display: contents or display: none)
    //   if its principal box is an internal table box other than table-cell
    //   if its principal box is an internal ruby box or a non-atomic inline-level box

    //   if the element does not generate a principal box (as is the case with display: contents or display: none)
    if (!m_style->display().doesGenerateBox())
        return false;
    if (m_style->display().isInternalTableBox() && m_style->display() != DisplayType::TableCell)
        return false;
    if (m_style->display().isRubyContainerOrInternalRubyBox() || (m_style->display() == DisplayType::InlineFlow && !m_element->isReplaced(m_style.ptr())))
        return false;
    return true;
}

inline bool ContainmentChecker::shouldApplySizeContainment() const
{
    // https://drafts.csswg.org/css-contain/#containment-size

    auto hasContainment = m_style->usedContain().contains(ContainValue::Size)
        || m_style->contentVisibility() == ContentVisibility::Hidden
        || (m_style->contentVisibility() == ContentVisibility::Auto && !m_element->isRelevantToUser());
    if (!hasContainment)
        return false;

    // Giving an element size containment has no effect if any of the following are true:
    //   if the element does not generate a principal box (as is the case with display: contents or display: none)
    //   if its inner display type is table
    //   if its principal box is an internal table box
    //   if its principal box is an internal ruby box or a non-atomic inline-level box

    if (!m_style->display().doesGenerateBox())
        return false;
    if (m_style->display().isTableBox())
        return false;
    if (m_style->display().isInternalTableBox())
        return false;
    if (m_style->display().isRubyContainerOrInternalRubyBox() || (m_style->display() == DisplayType::InlineFlow && !m_element->isReplaced(m_style.ptr())))
        return false;

    // https://drafts.csswg.org/css-conditional-5/#size-container
    // SVG foreignObject elements are not eligible to be size query containers.
    if (m_element->elementName() == ElementNames::SVG::foreignObject)
        return false;

    return true;
}

inline bool ContainmentChecker::shouldApplyInlineSizeContainment() const
{
    // https://drafts.csswg.org/css-contain/#containment-inline-size

    if (!m_style->usedContain().contains(ContainValue::InlineSize))
        return false;

    // Giving an element inline-size containment has no effect if any of the following are true:
    //   if the element does not generate a principal box (as is the case with display: contents or display: none)
    //   if its inner display type is table
    //   if its principal box is an internal table box
    //   if its principal box is an internal ruby box or a non-atomic inline-level box

    if (!m_style->display().doesGenerateBox())
        return false;
    if (m_style->display().isTableBox())
        return false;
    if (m_style->display().isInternalTableBox())
        return false;
    if (m_style->display().isRubyContainerOrInternalRubyBox() || (m_style->display() == DisplayType::InlineFlow && !m_element->isReplaced(m_style.ptr())))
        return false;

    // https://drafts.csswg.org/css-conditional-5/#size-container
    // SVG foreignObject elements are not eligible to be size query containers.
    if (m_element->elementName() == ElementNames::SVG::foreignObject)
        return false;

    return true;
}

inline bool ContainmentChecker::shouldApplyStyleContainment() const
{
    // https://drafts.csswg.org/css-contain/#containment-style

    // content-visibility hidden and auto turns on style containment.
    return m_style->usedContain().contains(ContainValue::Style)
        || m_style->contentVisibility() == ContentVisibility::Hidden
        || m_style->contentVisibility() == ContentVisibility::Auto;
}

inline bool ContainmentChecker::shouldApplyPaintContainment() const
{
    // https://drafts.csswg.org/css-contain/#containment-paint

    // content-visibility hidden and auto turns on paint containment.
    auto hasContainment = m_style->usedContain().contains(ContainValue::Paint)
        || m_style->contentVisibility() == ContentVisibility::Hidden
        || m_style->contentVisibility() == ContentVisibility::Auto;
    if (!hasContainment)
        return false;

    // Giving an element paint containment has no effect if any of the following are true:
    //   if the element does not generate a principal box (as is the case with display: contents or display: none)
    //   if its principal box is an internal table box other than table-cell
    //   if its principal box is an internal ruby box or a non-atomic inline-level box

    if (!m_style->display().doesGenerateBox())
        return false;
    if (m_style->display().isInternalTableBox() && m_style->display() != DisplayType::TableCell)
        return false;
    if (m_style->display().isRubyContainerOrInternalRubyBox() || (m_style->display() == DisplayType::InlineFlow && !m_element->isReplaced(m_style.ptr())))
        return false;
    return true;
}

inline bool ContainmentChecker::isSkippedContentRoot() const
{
    if (!shouldApplySizeContainment())
        return false;

    switch (m_style->contentVisibility()) {
    case ContentVisibility::Visible:
        return false;
    case ContentVisibility::Hidden:
        return true;
    case ContentVisibility::Auto:
        return !m_element->isRelevantToUser();
    default:
        ASSERT_NOT_REACHED();
        return false;
    }
}

} // namespace Style
} // namespace WebCore
````

## <a id="blink-style-cascade-cc"></a>Blink: third_party/blink/renderer/core/css/resolver/style_cascade.cc

Immutable source: [third_party/blink/renderer/core/css/resolver/style_cascade.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/resolver/style_cascade.cc). Full original SHA-256: `ca4a98680f9dc2e02e40bc3e75a44bbc319a7426ddd2efc59fcc2a1c26868c23` (126836 bytes). Retained scope: lines 1–4: complete legal header and original leading comments; lines 1454–1532: complete const CSSValue* StyleCascade::ResolvePendingSubstitution; lines 1670–1740: complete bool StyleCascade::ResolveTokensInto; lines 2349–2388: complete bool StyleCascade::ResolveEnvInto; lines 1742–1824: complete bool StyleCascade::ResolveVarInto; lines 2095–2121: complete bool StyleCascade::AppendDataWithFallback; lines 2984–2996: complete CSSVariableData* StyleCascade::GetEnvironmentVariable. All unlisted original source is omitted.

complete legal header and original leading comments:

````cpp
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

````

complete const CSSValue* StyleCascade::ResolvePendingSubstitution:

````cpp
const CSSValue* StyleCascade::ResolvePendingSubstitution(
    const CSSProperty& property,
    const cssvalue::CSSPendingSubstitutionValue& value,
    const TreeScope* tree_scope,
    FunctionContext* function_context_from_mixins,
    CascadeResolver& resolver) {
  DCHECK(!property.IsSurrogate());
  DCHECK(!resolver.IsLocked(property));
  CascadeResolver::AutoLock lock(property, resolver);

  DCHECK_NE(property.PropertyID(), CSSPropertyID::kVariable);

  MarkHasVariableReference(property);

  // If the previous call to ResolvePendingSubstitution parsed 'value', then
  // we don't need to do it again.
  bool is_cached = resolver.shorthand_cache_.value == &value;

  if (!is_cached) {
    CSSUnparsedDeclarationValue* shorthand_value = value.ShorthandValue();
    const auto* shorthand_data = shorthand_value->VariableDataValue();
    CSSPropertyID shorthand_property_id = value.ShorthandPropertyId();

    TokenSequence sequence;

    CSSParserTokenStream stream(shorthand_data->OriginalText());
    if (!ResolveTokensInto(stream, tree_scope, resolver,
                           *GetParserContext(*shorthand_value),
                           function_context_from_mixins,
                           /* stop_type */ kEOFToken, sequence)) {
      return cssvalue::CSSUnsetValue::Create();
    }

    HeapVector<CSSPropertyValue, 64> parsed_properties;

    // NOTE: We don't actually need the original text to be comment-stripped,
    // since we're not storing it in a custom property anywhere.
    CSSParserTokenStream stream2(sequence.OriginalText(),
                                 sequence.GetAttrTaintedRanges());
    if (!CSSPropertyParser::ParseValue(
            shorthand_property_id, /*allow_important_annotation=*/false,
            stream2, shorthand_value->ParserContext(), parsed_properties,
            StyleRule::RuleType::kStyle)) {
      return cssvalue::CSSUnsetValue::Create();
    }

    resolver.shorthand_cache_.value = &value;
    resolver.shorthand_cache_.parsed_properties = std::move(parsed_properties);
  }

  const auto& parsed_properties = resolver.shorthand_cache_.parsed_properties;

  // For -internal-visited-properties with CSSPendingSubstitutionValues,
  // the inner 'shorthand_property_id' will expand to a set of longhands
  // containing the unvisited equivalent. Hence, when parsing the
  // CSSPendingSubstitutionValue, we look for the unvisited property in
  // parsed_properties.
  const CSSProperty* unvisited_property =
      property.IsVisited() ? property.GetUnvisitedProperty() : &property;

  unsigned parsed_properties_count = parsed_properties.size();
  for (unsigned i = 0; i < parsed_properties_count; ++i) {
    const CSSProperty& longhand =
        CSSProperty::Get(parsed_properties[i].PropertyID());

    // When using var() in a css-logical shorthand (e.g. margin-inline),
    // the longhands here will also be logical.
    if (unvisited_property == &ResolveSurrogate(longhand)) {
      return &parsed_properties[i].Value();
    }
  }

  // Reaching this point means that the shorthand parser did not produce
  // a value for `property` (one of the longhands), which should never happen.
  //
  // TODO(crbug.com/40527196): We can reach here due to our incorrect adjustment
  // of writing-mode and direction for table display types.
  return cssvalue::CSSUnsetValue::Create();
}
````

complete bool StyleCascade::ResolveTokensInto:

````cpp
bool StyleCascade::ResolveTokensInto(CSSParserTokenStream& stream,
                                     const TreeScope* tree_scope,
                                     CascadeResolver& resolver,
                                     const CSSParserContext& context,
                                     FunctionContext* function_context,
                                     CSSParserTokenType stop_type,
                                     TokenSequence& out) {
  bool success = true;
  int nesting_level = 0;
  while (true) {
    const CSSParserToken& token = stream.Peek();
    if (token.IsEOF()) {
      break;
    } else if (token.GetType() == stop_type && nesting_level == 0) {
      break;
    } else if (token.FunctionId() == CSSValueID::kVar) {
      CSSParserTokenStream::BlockGuard guard(stream);
      success &= ResolveVarInto(stream, tree_scope, resolver, context,
                                function_context, out);
    } else if (token.FunctionId() == CSSValueID::kInherit) {
      CSSParserTokenStream::BlockGuard guard(stream);
      success &= ResolveInheritInto(stream, tree_scope, resolver, context,
                                    function_context, out);
    } else if (token.FunctionId() == CSSValueID::kEnv) {
      CSSParserTokenStream::BlockGuard guard(stream);
      success &= ResolveEnvInto(stream, tree_scope, resolver, context, out);
    } else if (token.FunctionId() == CSSValueID::kAttr) {
      CSSParserTokenStream::BlockGuard guard(stream);
      state_.StyleBuilder().SetHasAttrFunction();
      success &= ResolveAttrInto(stream, tree_scope, resolver, context,
                                 function_context, out);
    } else if (token.FunctionId() == CSSValueID::kInternalAutoBase) {
      CSSParserTokenStream::BlockGuard guard(stream);
      success &=
          ResolveAutoBaseInto(stream, tree_scope, resolver, context, out);
    } else if (token.FunctionId() == CSSValueID::kIf) {
      CSSParserTokenStream::BlockGuard guard(stream);
      success &= ResolveIfInto(stream, tree_scope, resolver, context,
                               function_context, out);
    } else if (token.GetType() == kFunctionToken &&
               CSSVariableParser::IsValidVariableName(token.Value()) &&
               RuntimeEnabledFeatures::CSSFunctionsEnabled()) {
      // User-defined CSS function.
      CSSParserTokenStream::BlockGuard guard(stream);
      success &= ResolveFunctionInto(token.Value(), tree_scope, stream,
                                     resolver, context, function_context, out);
    } else {
      if (token.GetBlockType() == CSSParserToken::kBlockStart) {
        ++nesting_level;
      } else if (token.GetBlockType() == CSSParserToken::kBlockEnd) {
        if (nesting_level == 0) {
          // Attempting to go outside our block.
          break;
        }
        --nesting_level;
      }
      wtf_size_t start = stream.Offset();
      stream.ConsumeRaw();
      wtf_size_t end = stream.Offset();

      // NOTE: This will include any comment tokens that ConsumeRaw()
      // skipped over; i.e., any comment will be attributed to the
      // token after it and any trailing comments will be skipped.
      // This is fine, because trailing comments (sans whitespace)
      // should be skipped anyway.
      out.Append(token, stream.IsAttrTainted(start, end),
                 stream.StringRangeAt(start, end - start));
    }
  }
  return success;
}
````

complete bool StyleCascade::ResolveEnvInto:

````cpp
bool StyleCascade::ResolveEnvInto(CSSParserTokenStream& stream,
                                  const TreeScope* tree_scope,
                                  CascadeResolver& resolver,
                                  const CSSParserContext& context,
                                  TokenSequence& out) {
  state_.StyleBuilder().SetHasEnv();
  AtomicString variable_name = ConsumeVariableName(stream);

  if (variable_name == "safe-area-inset-bottom") {
    state_.StyleBuilder().SetHasEnvSafeAreaInsetBottom();
    state_.GetDocument()
        .GetStyleEngine()
        .SetNeedsToUpdateComplexSafeAreaConstraints();
  }

  // Normal env().
  DCHECK(stream.AtEnd() || (stream.Peek().GetType() == kCommaToken) ||
         (stream.Peek().GetType() == kNumberToken));

  Vector<unsigned> indices;
  if (!stream.AtEnd() && stream.Peek().GetType() != kCommaToken) {
    do {
      const CSSParserToken& token = stream.ConsumeIncludingWhitespaceRaw();
      DCHECK(token.GetNumericValueType() == kIntegerValueType);
      DCHECK(token.NumericValue() >= 0.);
      indices.push_back(static_cast<unsigned>(token.NumericValue()));
    } while (stream.Peek().GetType() == kNumberToken);
  }

  DCHECK(stream.AtEnd() || (stream.Peek().GetType() == kCommaToken));

  CSSVariableData* data =
      GetEnvironmentVariable(variable_name, std::move(indices));

  // Appending the fallback (if any) through the same path as var() gives it
  // the same treatment: the whitespace and comments surrounding it are not
  // part of the substitution value.
  return AppendDataWithFallback(data, stream, tree_scope, resolver, context,
                                /*function_context=*/nullptr, out);
}
````

complete bool StyleCascade::ResolveVarInto:

````cpp
bool StyleCascade::ResolveVarInto(CSSParserTokenStream& stream,
                                  const TreeScope* tree_scope,
                                  CascadeResolver& resolver,
                                  const CSSParserContext& context,
                                  FunctionContext* function_context,
                                  TokenSequence& out) {
  AtomicString var_name =
      ConsumeAndComputeVariableName(stream, context, state_);
  // Note that `var_name` may be "unknown" when an ident() function
  // didn't produce a valid variable name. Looking up this custom property
  // name (which is unreachable for authors, and therefore never set)
  // automatically gives the correct IACVT/fallback behavior without
  // any explicit handling.
  DCHECK(stream.AtEnd() || (stream.Peek().GetType() == kCommaToken));

  // Within a function context (i.e. when resolving values within the body of
  // an @function rule), var() must first look for local variables
  // and arguments.
  //
  // https://drafts.csswg.org/css-mixins-1/#locally-substitute-a-var
  if (function_context) {
    // Locals shadow arguments, which shadow custom properties
    // from the element.

    for (FunctionContext* frame = function_context; frame;
         frame = frame->parent) {
      // Ensure that any local variable with a matching name is applied
      // (i.e. exists on frame->locals).
      LookupAndApplyLocalVariable(var_name, resolver, context, *frame);
      if (std::optional<CSSVariableData*> local_variable =
              FindOrNullopt(frame->locals, var_name)) {
        // Note that we should indeed pass `function_context` here,
        // and not `frame`. This is because the `function_context
        // is only used to resolve the fallback, which must be interpreted
        // in the function context holding the var() function.
        return AppendDataWithFallback(local_variable.value(), stream,
                                      tree_scope, resolver, context,
                                      function_context, out);
      }
      // Note that there is no "lookup and apply" step for arguments; one
      // argument cannot reference another using var() or similar.
      if (std::optional<CSSVariableData*> argument =
              FindOrNullopt(frame->arguments, var_name)) {
        return AppendDataWithFallback(argument.value(), stream, tree_scope,
                                      resolver, context, function_context, out);
      }
    }
  }

  CustomProperty property(&var_name, state_.GetDocument());

  // Any custom property referenced (by anything, even just once) in the
  // document can currently not be animated on the compositor. Hence we mark
  // properties that have been referenced.
  MarkIsReferenced(property);

  if (!resolver.DetectCycle(property)) {
    // We are about to substitute var(property). In order to do that, we must
    // know the computed value of 'property', hence we Apply it.
    //
    // We can however not do this if we're in a cycle. If a cycle is detected
    // here, it means we are already resolving 'property', and have discovered
    // a reference to 'property' during that resolution.
    LookupAndApply(property, resolver);
  }

  CSSVariableData* data = GetVariableData(property);

  // If substitution is not allowed, treat the value as
  // invalid-at-computed-value-time.
  //
  // https://drafts.csswg.org/css-variables/#animation-tainted
  if (!resolver.AllowSubstitution(data)) {
    data = nullptr;
  }

  if (resolver.InCycle()) {
    // Either DetectCycle() or LookupAndApply() caused a cycle.
    return false;
  }
  return AppendDataWithFallback(data, stream, tree_scope, resolver, context,
                                function_context, out);
}
````

complete bool StyleCascade::AppendDataWithFallback:

````cpp
bool StyleCascade::AppendDataWithFallback(CSSVariableData* data,
                                          CSSParserTokenStream& stream,
                                          const TreeScope* tree_scope,
                                          CascadeResolver& resolver,
                                          const CSSParserContext& context,
                                          FunctionContext* function_context,
                                          TokenSequence& out) {
  if (data) {
    DCHECK(!data->NeedsVariableResolution());
    return out.Append(data, data->IsAttrTainted(),
                      CSSVariableData::kMaxVariableBytes);
  }
  // Empty/invalid data; try fallback:
  if (ConsumeComma(stream)) {
    stream.ConsumeWhitespace();
    TokenSequence fallback;
    if (ResolveTokensInto(stream, tree_scope, resolver, context,
                          function_context,
                          /*stop_type=*/kEOFToken, fallback)) {
      return out.AppendFallback(
          fallback,
          /*is_attr_tainted=*/!fallback.GetAttrTaintedRanges()->empty(),
          CSSVariableData::kMaxVariableBytes);
    }
  }
  return false;
}
````

complete CSSVariableData* StyleCascade::GetEnvironmentVariable:

````cpp
CSSVariableData* StyleCascade::GetEnvironmentVariable(
    const AtomicString& name,
    Vector<unsigned> indices) const {
  // If we are in a User Agent Shadow DOM then we should not record metrics.
  ContainerNode& scope_root = state_.GetElement().GetTreeScope().RootNode();
  auto* shadow_root = DynamicTo<ShadowRoot>(&scope_root);
  bool is_ua_scope = shadow_root && shadow_root->IsUserAgent();

  return state_.GetDocument()
      .GetStyleEngine()
      .EnsureEnvironmentVariables()
      .ResolveVariable(name, std::move(indices), !is_ua_scope);
}
````

## <a id="blink-container-query-evaluator-cc"></a>Blink: third_party/blink/renderer/core/css/container_query_evaluator.cc

Immutable source: [third_party/blink/renderer/core/css/container_query_evaluator.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/container_query_evaluator.cc). Full original SHA-256: `3a0d84ab42d49bc7974dbdc331d55378c5df60d5b260b6c5f09b5269497ca25b` (38001 bytes). Retained scope: lines 1–4: complete legal header and original leading comments; lines 32–41: complete PhysicalAxes ContainerTypeAxes; lines 577–615: complete void ContainerQueryEvaluator::UpdateContainerSize. All unlisted original source is omitted.

complete legal header and original leading comments:

````cpp
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

````

complete PhysicalAxes ContainerTypeAxes:

````cpp
PhysicalAxes ContainerTypeAxes(const ComputedStyle& style) {
  LogicalAxes axes = kLogicalAxesNone;
  if (style.ContainerType() & kContainerTypeInlineSize) {
    axes |= kLogicalAxesInline;
  }
  if (style.ContainerType() & kContainerTypeBlockSize) {
    axes |= kLogicalAxesBlock;
  }
  return ToPhysicalAxes(axes, style.GetWritingMode());
}
````

complete void ContainerQueryEvaluator::UpdateContainerSize:

````cpp
void ContainerQueryEvaluator::UpdateContainerSize(PhysicalSize size,
                                                  PhysicalAxes contained_axes) {
  size_ = size;
  contained_axes_ = contained_axes;

  std::optional<double> width;
  std::optional<double> height;

  const MediaValues& existing_values = media_query_evaluator_->GetMediaValues();
  Element* container = existing_values.ContainerElement();

  // An axis is "supported" only when it appears in the computed value of
  // 'container-type', and when containment is actually applied for that axis.
  //
  // See IsEligibleForSizeContainment (and similar).
  PhysicalAxes supported_axes =
      ContainerTypeAxes(container->ComputedStyleRef()) & contained_axes;

  if ((supported_axes & PhysicalAxes(kPhysicalAxesHorizontal)) !=
      PhysicalAxes(kPhysicalAxesNone)) {
    width = size.width.ToDouble();
  }

  if ((supported_axes & PhysicalAxes(kPhysicalAxesVertical)) !=
      PhysicalAxes(kPhysicalAxesNone)) {
    height = size.height.ToDouble();
  }

  auto* query_values = MakeGarbageCollected<CSSContainerValues>(
      container->GetDocument(), *container, width, height,
      existing_values.StuckHorizontal(), existing_values.StuckVertical(),
      existing_values.SnappedFlags(), existing_values.ScrollableHorizontal(),
      existing_values.ScrollableVertical(),
      existing_values.ScrolledHorizontal(), existing_values.ScrolledVertical(),
      existing_values.AbsContainerWritingDirection(),
      existing_values.AnchoredFallback());
  media_query_evaluator_ =
      MakeGarbageCollected<MediaQueryEvaluator>(query_values);
}
````

## <a id="blink-legacy-speak"></a>Blink legacy speak entry

[third_party/blink/renderer/core/css/css_properties.json5](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_properties.json5); full original SHA-256 `d8e8f3205ca241d76ab736ebc2b21a89aa5275c0fd8b7be32c09bea6317b8aba` (390281 bytes). Retained lines 1–3 and 6008–6019; all other source omitted. This entry lists normal/none and punctuation hints, not the modern auto/never/always grammar, so it cannot adjudicate Speech 1’s computed auto conflict.

````cpp
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
````

````json5
    {
      name: "speak",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: [
        "none", "normal", "spell-out", "digits", "literal-punctuation",
        "no-punctuation"
      ],
      default_value: "normal",
    },
````
