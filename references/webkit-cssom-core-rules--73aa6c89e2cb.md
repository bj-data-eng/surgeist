# WebKit core CSSOM source witness

This reference retains 5 complete source files from [Webkit](https://webkit.org/) at immutable revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. It supplies bounded implementation evidence for the stated CSSOM planning contribution. It is not a CSS specification, a whole-engine compatibility promise, or evidence about other revisions.

Retrieved on 2026-10-09 from the existing local checkout using immutable `git show` bytes. Only the exact revision’s file bytes are included; working-tree changes are excluded. Each full file has its original path, byte hash and license header below. Added headings, provenance and the evidence map are documentation formatting; the source bytes are unchanged. The captured code is reference material and was not compiled or executed.

Original file-specific copyright notices, license conditions and disclaimers remain in the source blocks. The [local legal entry](../licenses/webkit/NOTICE.md) identifies applicable full license texts and their provenance. File-specific LGPL notices govern the named LGPL files; a project’s BSD license is not applied to them by inference.

The source boundaries below identify the relied-on behavior. Retaining complete files supplies surrounding context without selecting unrelated engine behavior.

| Source | Evidence boundaries | Original file SHA-256 |
| --- | --- | --- |
| [CSSImportRule.cpp](#cssimportrule-cpp) | [lines 60–64](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSImportRule.cpp#L60), [lines 125–152](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSImportRule.cpp#L125) | `f03cd5006f1c7ebaa2eb24d91eecba8a96da66d109d1a6364f6b71e1163e61ba` |
| [MediaList.cpp](#medialist-cpp) | [lines 61–84](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/MediaList.cpp#L61) | `252b1c23d9082a5966661a1bb0132a300599bd6da8dd5f61f58e3ca047fa8233` |
| [CSSRule.h](#cssrule-h) | [lines 93–109](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSRule.h#L93) | `48a242cca9b1f8b9803bed92adfad95b6e4f9b6c63c73591b7abde1b3736cdfa` |
| [CSSStyleSheet.cpp](#cssstylesheet-cpp) | [lines 583–588](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSStyleSheet.cpp#L583) | `e2f98fed847f3b407c7aa21756917bf3867aaf6b004de6a1c0e3de769f2614e8` |
| [CSSGroupingRule.cpp](#cssgroupingrule-cpp) | [lines 124–128](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSGroupingRule.cpp#L124) | `960e383b8ba17bf09f21ae42fca77cb21bb44594786f304b8f1d4c7a2561970c` |

## <a id="cssimportrule-cpp"></a>CSSImportRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSImportRule.cpp). Path: `Source/WebCore/css/CSSImportRule.cpp`. Source bytes: 4965; source lines: 165; SHA-256: `f03cd5006f1c7ebaa2eb24d91eecba8a96da66d109d1a6364f6b71e1163e61ba`.

```cpp
/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * (C) 2002-2003 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2002, 2005, 2006, 2008, 2009, 2010, 2012, 2013 Apple Inc. All rights reserved.
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

#include "config.h"
#include "CSSImportRule.h"

#include "CSSLayerBlockRule.h"
#include "CSSSerializationContext.h"
#include "CSSStyleSheet.h"
#include "CSSURL.h"
#include "CSSValueTypes.h"
#include "CachedCSSStyleSheet.h"
#include "MediaList.h"
#include "MediaQueryParser.h"
#include "StyleRuleImport.h"
#include "StyleSheetContents.h"
#include <wtf/text/StringBuilder.h>

namespace WebCore {

WTF_MAKE_TZONE_ALLOCATED_IMPL(CSSImportRule);

CSSImportRule::CSSImportRule(StyleRuleImport& importRule, CSSStyleSheet* parent)
    : CSSRule(parent)
    , m_importRule(importRule)
{
}

CSSImportRule::~CSSImportRule()
{
    if (m_styleSheetCSSOMWrapper)
        protect(m_styleSheetCSSOMWrapper)->clearOwnerRule();
    if (m_mediaCSSOMWrapper)
        protect(m_mediaCSSOMWrapper)->detachFromParent();
}

String CSSImportRule::href() const
{
    return m_importRule.get().href();
}

MediaList& CSSImportRule::media() const
{
    if (!m_mediaCSSOMWrapper)
        m_mediaCSSOMWrapper = MediaList::create(const_cast<CSSImportRule*>(this));
    return *m_mediaCSSOMWrapper;
}

String CSSImportRule::layerName() const
{
    auto name = m_importRule->cascadeLayerName();
    if (!name)
        return { };

    return stringFromCascadeLayerName(*name);
}

String CSSImportRule::supportsText() const
{
    return m_importRule->supportsText();
}

String CSSImportRule::cssTextInternal(const String& urlString, const CSS::SerializationContext& context) const
{
    StringBuilder builder;
    builder.append("@import "_s);
    CSS::serializationForCSS(builder, context, CSS::URL { .specified = urlString, .resolved = { }, .modifiers = { } });

    if (auto layerName = this->layerName(); !layerName.isNull()) {
        if (layerName.isEmpty())
            builder.append(" layer"_s);
        else
            builder.append(" layer("_s, layerName, ')');
    }

    auto supports = supportsText();
    if (!supports.isNull())
        builder.append(" supports("_s, WTF::move(supports), ')');

    if (!mediaQueries().isEmpty()) {
        builder.append(' ');
        MQ::serialize(builder, mediaQueries());
    }

    builder.append(';');
    return builder.toString();
}

String CSSImportRule::cssText() const
{
    return cssTextInternal(m_importRule->href(), CSS::defaultSerializationContext());
}

String CSSImportRule::cssText(const CSS::SerializationContext& context) const
{
    if (RefPtr sheet = styleSheet()) {
        auto urlString = context.replacementURLStringsForCSSStyleSheet.get(*sheet);
        if (!urlString.isEmpty())
            return cssTextInternal(urlString, context);
    }

    auto urlString = m_importRule->href();
    auto replacementURLString = context.replacementURLStrings.get(urlString);
    return cssTextInternal(replacementURLString.isEmpty() ? urlString : replacementURLString, context);
}

CSSStyleSheet* CSSImportRule::styleSheet() const
{ 
    if (!m_importRule.get().styleSheet())
        return nullptr;

    std::optional<bool> isOriginClean;
    if (auto* cachedSheet = m_importRule->cachedCSSStyleSheet())
        isOriginClean = cachedSheet->isCORSSameOrigin();

    if (!m_styleSheetCSSOMWrapper)
        m_styleSheetCSSOMWrapper = CSSStyleSheet::create(*m_importRule.get().styleSheet(), const_cast<CSSImportRule*>(this), isOriginClean);
    return m_styleSheetCSSOMWrapper.get(); 
}

void CSSImportRule::reattach(StyleRuleBase&)
{
    // FIXME: Implement when enabling caching for stylesheets with import rules.
    ASSERT_NOT_REACHED();
}

const MQ::MediaQueryList& CSSImportRule::mediaQueries() const
{
    return m_importRule->mediaQueries();
}

void CSSImportRule::setMediaQueries(MQ::MediaQueryList&& queries)
{
    m_importRule->setMediaQueries(WTF::move(queries));
}

void CSSImportRule::getChildStyleSheets(HashSet<Ref<CSSStyleSheet>>& childStyleSheets)
{
    RefPtr sheet = styleSheet();
    if (!sheet)
        return;

    if (childStyleSheets.add(*sheet).isNewEntry)
        sheet->getChildStyleSheets(childStyleSheets);
}

} // namespace WebCore

```

## <a id="medialist-cpp"></a>MediaList.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/MediaList.cpp). Path: `Source/WebCore/css/MediaList.cpp`. Source bytes: 5268; source lines: 180; SHA-256: `252b1c23d9082a5966661a1bb0132a300599bd6da8dd5f61f58e3ca047fa8233`.

```cpp
/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004-2020 Apple Inc. All rights reserved.
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

#include "config.h"
#include "MediaList.h"

#include "CSSImportRule.h"
#include "CSSMediaRule.h"
#include "CSSStyleSheet.h"
#include "Document.h"
#include "LocalDOMWindow.h"
#include "MediaQuery.h"
#include "MediaQueryParser.h"
#include <wtf/NeverDestroyed.h>
#include <wtf/text/StringBuilder.h>
#include <wtf/text/TextStream.h>

namespace WebCore {

MediaList::MediaList(CSSStyleSheet* parentSheet)
    : m_parentStyleSheet(parentSheet)
{
}

MediaList::MediaList(CSSRule* parentRule)
    : m_parentRule(parentRule)
{
}

MediaList::~MediaList() = default;

void MediaList::detachFromParent()
{
    m_detachedMediaQueries = mediaQueries();
    m_parentStyleSheet = nullptr;
    m_parentRule = nullptr;
}

unsigned MediaList::length() const
{
    return mediaQueries().size();
}

const MQ::MediaQueryList& MediaList::mediaQueries() const
{
    if (m_detachedMediaQueries)
        return *m_detachedMediaQueries;
    if (auto* rule = dynamicDowncast<CSSImportRule>(m_parentRule.get()))
        return rule->mediaQueries();
    if (auto* rule = dynamicDowncast<CSSMediaRule>(m_parentRule.get()))
        return rule->mediaQueries();
    return m_parentStyleSheet->mediaQueries();
}

void MediaList::setMediaQueries(MQ::MediaQueryList&& queries)
{
    if (RefPtr parentStyleSheet = m_parentStyleSheet.get()) {
        parentStyleSheet->setMediaQueries(WTF::move(queries));
        parentStyleSheet->didMutate();
        return;
    }

    CSSStyleSheet::RuleMutationScope mutationScope(m_parentRule.get());
    if (RefPtr rule = dynamicDowncast<CSSImportRule>(m_parentRule.get()))
        rule->setMediaQueries(WTF::move(queries));
    if (RefPtr rule = dynamicDowncast<CSSMediaRule>(m_parentRule.get()))
        rule->setMediaQueries(WTF::move(queries));
}

String MediaList::mediaText() const
{
    StringBuilder builder;
    MQ::serialize(builder, mediaQueries());
    return builder.toString();
}

void MediaList::setMediaText(const String& value)
{
    setMediaQueries(MQ::MediaQueryParser::parse(value, strictCSSParserContext()));
}

CSSRule* MediaList::parentRule() const
{
    return m_parentRule.get();
}

CSSStyleSheet* MediaList::parentStyleSheet() const
{
    return m_parentStyleSheet.get();
}

String MediaList::item(unsigned index) const
{
    auto& queries = mediaQueries();
    if (index < queries.size()) {
        StringBuilder builder;
        MQ::serialize(builder, queries[index]);
        return builder.toString();
    }
    return { };
}

ExceptionOr<void> MediaList::deleteMedium(const String& value)
{
    // https://drafts.csswg.org/cssom/#dom-medialist-deletemedium
    // The value is parsed as a single media query; bail unless parsing
    // produces exactly one query.
    auto parsedQueries = MQ::MediaQueryParser::parse(value, strictCSSParserContext());
    if (parsedQueries.size() != 1)
        return Exception { ExceptionCode::NotFoundError };

    auto serializeQuery = [](const MQ::MediaQuery& query) {
        StringBuilder builder;
        MQ::serialize(builder, query);
        return builder.toString();
    };
    auto queryToRemove = serializeQuery(parsedQueries[0]);
    auto queries = mediaQueries();
    auto removedCount = queries.removeAllMatching([&](auto& query) {
        return serializeQuery(query) == queryToRemove;
    });
    if (!removedCount)
        return Exception { ExceptionCode::NotFoundError };

    setMediaQueries(WTF::move(queries));
    return { };
}

void MediaList::appendMedium(const String& value)
{
    if (value.isEmpty())
        return;

    // https://drafts.csswg.org/cssom/#dom-medialist-appendmedium
    auto parsedQueries = MQ::MediaQueryParser::parse(value, strictCSSParserContext());
    if (parsedQueries.size() != 1)
        return;

    StringBuilder builder;
    MQ::serialize(builder, parsedQueries[0]);
    auto newQueryText = builder.toString();

    auto queries = mediaQueries();
    bool alreadyPresent = queries.containsIf([&](auto& query) {
        StringBuilder existingQueryBuilder;
        MQ::serialize(existingQueryBuilder, query);
        return existingQueryBuilder.toString() == newQueryText;
    });
    if (alreadyPresent)
        return;

    queries.append(WTF::move(parsedQueries[0]));
    setMediaQueries(WTF::move(queries));
}

TextStream& operator<<(TextStream& ts, const MediaList& mediaList)
{
    ts << mediaList.mediaText();
    return ts;
}

} // namespace WebCore


```

## <a id="cssrule-h"></a>CSSRule.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSRule.h). Path: `Source/WebCore/css/CSSRule.h`. Source bytes: 3750; source lines: 117; SHA-256: `48a242cca9b1f8b9803bed92adfad95b6e4f9b6c63c73591b7abde1b3736cdfa`.

```cpp
/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * (C) 2002-2003 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2002-2021 Apple Inc. All rights reserved.
 * Copyright (C) 2011 Andreas Kling (kling@webkit.org)
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

#pragma once

#include <WebCore/CSSParserEnum.h>
#include <WebCore/StyleRuleType.h>
#include <wtf/RefCountedAndCanMakeWeakPtr.h>
#include <wtf/TypeCasts.h>
#include <wtf/text/WTFString.h>

namespace WebCore {

class CSSStyleSheet;
class StyleRuleBase;
class StyleRule;
class StyleRuleWithNesting;

struct CSSParserContext;

template<typename> class ExceptionOr;

namespace CSS {
struct SerializationContext;
}

class CSSRule : public RefCountedAndCanMakeWeakPtr<CSSRule> {
public:
    virtual ~CSSRule() = default;

    WEBCORE_EXPORT unsigned short typeForCSSOM() const;

    virtual StyleRuleType styleRuleType() const = 0;
    virtual bool isGroupingRule() const { return false; }
    virtual String cssText() const = 0;
    virtual String cssText(const CSS::SerializationContext&) const { return cssText(); }
    virtual void reattach(StyleRuleBase&) = 0;

    void setParentStyleSheet(CSSStyleSheet*);
    void setParentRule(CSSRule*);
    CSSStyleSheet* parentStyleSheet() const;
    CSSRule* parentRule() const { return m_parentIsRule ? m_parentRule : nullptr; }
    bool hasStyleRuleAncestor() const;
    CSSParserEnum::NestedContext nestedContext() const;
    virtual RefPtr<StyleRuleWithNesting> prepareChildStyleRuleForNesting(StyleRule&);
    virtual void getChildStyleSheets(HashSet<Ref<CSSStyleSheet>>&) { }

    WEBCORE_EXPORT ExceptionOr<void> setCssText(const String&);

protected:
    explicit CSSRule(CSSStyleSheet*);

    bool hasCachedSelectorText() const { return m_hasCachedSelectorText; }
    void setHasCachedSelectorText(bool hasCachedSelectorText) const { m_hasCachedSelectorText = hasCachedSelectorText; }

    const CSSParserContext& parserContext() const;

private:
    mutable unsigned char m_hasCachedSelectorText : 1;
    unsigned char m_parentIsRule : 1;
    union {
        CSSRule* m_parentRule;
        CSSStyleSheet* m_parentStyleSheet;
    };
};

inline CSSRule::CSSRule(CSSStyleSheet* parent)
    : m_hasCachedSelectorText(false)
    , m_parentIsRule(false)
    , m_parentStyleSheet(parent)
{
}

inline void CSSRule::setParentStyleSheet(CSSStyleSheet* styleSheet)
{
    m_parentIsRule = false;
    m_parentStyleSheet = styleSheet;
}

inline void CSSRule::setParentRule(CSSRule* rule)
{
    m_parentIsRule = true;
    m_parentRule = rule;
}

inline CSSStyleSheet* CSSRule::parentStyleSheet() const
{
    if (m_parentIsRule)
        return m_parentRule ? m_parentRule->parentStyleSheet() : nullptr;
    return m_parentStyleSheet;
}

} // namespace WebCore

#define SPECIALIZE_TYPE_TRAITS_CSS_RULE(ToValueTypeName, predicate) \
SPECIALIZE_TYPE_TRAITS_BEGIN(WebCore::ToValueTypeName) \
    static bool isType(const WebCore::CSSRule& rule) { return rule.styleRuleType() == WebCore::predicate; } \
SPECIALIZE_TYPE_TRAITS_END()

```

## <a id="cssstylesheet-cpp"></a>CSSStyleSheet.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSStyleSheet.cpp). Path: `Source/WebCore/css/CSSStyleSheet.cpp`. Source bytes: 22619; source lines: 664; SHA-256: `e2f98fed847f3b407c7aa21756917bf3867aaf6b004de6a1c0e3de769f2614e8`.

```cpp
/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004-2025 Apple Inc. All rights reserved.
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

#include "config.h"
#include "CSSStyleSheet.h"

#include "CSSImportRule.h"
#include "CSSKeyframesRule.h"
#include "CSSParser.h"
#include "CSSRuleList.h"
#include "DocumentSecurityOrigin.h"
#include "HTMLLinkElement.h"
#include "HTMLStyleElement.h"
#include "JSCSSStyleSheet.h"
#include "JSDOMConvertInterface.h"
#include "JSDOMPromiseDeferred.h"
#include "JSNodeCustomInlines.h"
#include "Logging.h"
#include "MediaList.h"
#include "MediaQueryParser.h"
#include "Node.h"
#include "OriginAccessPatterns.h"
#include "SVGElementTypeHelpers.h"
#include "SVGStyleElement.h"
#include "SecurityOrigin.h"
#include "ShadowRoot.h"
#include "StyleDocumentScope.h"
#include "StyleResolver.h"
#include "StyleRule.h"
#include "StyleSheetContents.h"
#include "StyleSheetContentsCache.h"
#include <wtf/HexNumber.h>
#include <wtf/text/MakeString.h>
#include <wtf/text/StringBuilder.h>

namespace WebCore {

static Style::Scope& NODELETE styleScopeFor(ContainerNode& treeScope)
{
    ASSERT(is<Document>(treeScope) || is<ShadowRoot>(treeScope));
    if (auto* shadowRoot = dynamicDowncast<ShadowRoot>(treeScope))
        return shadowRoot->styleScope();
    return downcast<Document>(treeScope).styleScope();
}

DECLARE_ALLOCATOR_WITH_HEAP_IDENTIFIER(StyleSheetCSSRuleList);
class StyleSheetCSSRuleList final : public CSSRuleList {
    WTF_DEPRECATED_MAKE_FAST_ALLOCATED_WITH_HEAP_IDENTIFIER(StyleSheetCSSRuleList, StyleSheetCSSRuleList);
public:
    StyleSheetCSSRuleList(CSSStyleSheet* sheet) : m_styleSheet(sheet) { }
    
private:
    void NODELETE ref() const final { m_styleSheet->ref(); }
    void deref() const final { m_styleSheet->deref(); }

    unsigned length() const final { return m_styleSheet->length(); }
    CSSRule* item(unsigned index) const final { return protect(m_styleSheet)->item(index); }

    CSSStyleSheet* NODELETE styleSheet() const final { return m_styleSheet.get(); }

    SingleThreadWeakPtr<CSSStyleSheet> m_styleSheet;
};
DEFINE_ALLOCATOR_WITH_HEAP_IDENTIFIER(StyleSheetCSSRuleList);

#if ASSERT_ENABLED
static bool isAcceptableCSSStyleSheetParent(Node* parentNode)
{
    // Only these nodes can be parents of StyleSheets, and they need to call clearOwnerNode() when moved out of document.
    return !parentNode
        || parentNode->isDocumentNode()
        || isAnyOf<HTMLLinkElement, HTMLStyleElement, SVGStyleElement>(*parentNode)
        || parentNode->nodeType() == NodeType::ProcessingInstruction;
}
#endif // ASSERT_ENABLED

Ref<CSSStyleSheet> CSSStyleSheet::create(Ref<StyleSheetContents>&& sheet, CSSImportRule* ownerRule, std::optional<bool> isOriginClean)
{
    return adoptRef(*new CSSStyleSheet(WTF::move(sheet), ownerRule, isOriginClean));
}

Ref<CSSStyleSheet> CSSStyleSheet::create(Ref<StyleSheetContents>&& sheet, Node& ownerNode, const std::optional<bool>& isCleanOrigin)
{
    return adoptRef(*new CSSStyleSheet(WTF::move(sheet), ownerNode, TextPosition(), false, isCleanOrigin));
}

Ref<CSSStyleSheet> CSSStyleSheet::createInline(Ref<StyleSheetContents>&& sheet, Element& owner, const TextPosition& startPosition)
{
    return adoptRef(*new CSSStyleSheet(WTF::move(sheet), owner, startPosition, true, true));
}

ExceptionOr<Ref<CSSStyleSheet>> CSSStyleSheet::create(Document& document, Init&& init)
{
    URL baseURL;
    if (init.baseURL.isNull())
        baseURL = document.baseURL();
    else {
        baseURL = document.encodingParseURL(init.baseURL);
        if (!baseURL.isValid())
            return Exception { ExceptionCode::NotAllowedError, "Base URL is invalid"_s };
    }

    CSSParserContext parserContext(document, baseURL);
    parserContext.shouldIgnoreImportRules = true;
    return adoptRef(*new CSSStyleSheet(StyleSheetContents::create(parserContext), document, WTF::move(init)));
}

CSSStyleSheet::CSSStyleSheet(Ref<StyleSheetContents>&& contents, CSSImportRule* ownerRule, std::optional<bool> isOriginClean)
    : m_contents(WTF::move(contents))
    , m_isOriginClean(isOriginClean)
    , m_ownerRule(ownerRule)
{
    if (RefPtr parent = parentStyleSheet())
        m_styleScope = parent->styleScope();

    protect(m_contents)->registerClient(this);
}

CSSStyleSheet::CSSStyleSheet(Ref<StyleSheetContents>&& contents, Node& ownerNode, const TextPosition& startPosition, bool isInlineStylesheet, const std::optional<bool>& isOriginClean)
    : m_contents(WTF::move(contents))
    , m_isInlineStylesheet(isInlineStylesheet)
    , m_isOriginClean(isOriginClean)
    , m_styleScope(Style::Scope::forNode(ownerNode))
    , m_ownerNode(ownerNode)
    , m_startPosition(startPosition)
{
    ASSERT(isAcceptableCSSStyleSheetParent(&ownerNode));
    protect(m_contents)->registerClient(this);
}

// https://w3c.github.io/csswg-drafts/cssom-1/#dom-cssstylesheet-cssstylesheet
CSSStyleSheet::CSSStyleSheet(Ref<StyleSheetContents>&& contents, Document& document, Init&& options)
    : m_contents(WTF::move(contents))
    , m_isDisabled(options.disabled)
    , m_wasConstructedByJS(true)
    , m_isOriginClean(true)
    , m_constructorDocument(document)
{
    protect(m_contents)->registerClient(this);
    protect(m_contents)->checkLoaded();

    WTF::switchOn(WTF::move(options.media),
        [this](Ref<MediaList>&& mediaList) {
            if (auto queries = mediaList->mediaQueries(); !queries.isEmpty())
                setMediaQueries(WTF::move(queries));
        },
        [this](String&& mediaString) {
            setMediaQueries(MQ::MediaQueryParser::parse(mediaString, strictCSSParserContext()));
        }
    );
}

CSSStyleSheet::~CSSStyleSheet()
{
    // For style rules outside the document, .parentStyleSheet can become null even if the style rule
    // is still observable from JavaScript. This matches the behavior of .parentNode for nodes, but
    // it's not ideal because it makes the CSSOM's behavior depend on the timing of garbage collection.
    for (unsigned i = 0; i < m_childRuleCSSOMWrappers.size(); ++i) {
        if (m_childRuleCSSOMWrappers[i])
            m_childRuleCSSOMWrappers[i]->setParentStyleSheet(nullptr);
    }
    if (m_mediaCSSOMWrapper)
        protect(m_mediaCSSOMWrapper)->detachFromParent();

    protect(m_contents)->unregisterClient(this);
}

Node* CSSStyleSheet::ownerNode() const
{
    assertIsOwnerThread();
    return m_ownerNode.get();
}

RefPtr<StyleRuleWithNesting> CSSStyleSheet::prepareChildStyleRuleForNesting(StyleRule& styleRule)
{
    RuleMutationScope scope(this);
    auto& rules = m_contents->m_childRules;
    for (size_t i = 0 ; i < rules.size() ; i++) {
        if (rules[i].ptr() == &styleRule) {
            auto styleRuleWithNesting = StyleRuleWithNesting::create(WTF::move(styleRule));
            rules[i] = styleRuleWithNesting;
            return styleRuleWithNesting;
        }
    }
    return { };
}

auto CSSStyleSheet::willMutateRules() -> ContentsClonedForMutation {
    // If we are the only client it is safe to mutate.
    if (m_contents->hasOneClient() && !m_contents->isInMemoryCache()) {
        m_contents->setMutable();
        return ContentsClonedForMutation::No;
    }
    // Only cacheable stylesheets should have multiple clients.
    ASSERT(m_contents->isCacheable());

    // Copy-on-write.
    protect(m_contents)->unregisterClient(this);
    m_contents = protect(m_contents)->copy();
    protect(m_contents)->registerClient(this);

    m_contents->setMutable();

    // Any existing CSSOM wrappers need to be connected to the copied child rules.
    reattachChildRuleCSSOMWrappers();

    return ContentsClonedForMutation::Yes;
}

void CSSStyleSheet::didMutateRuleFromCSSStyleDeclaration()
{
    ASSERT(m_contents->isMutable());
    ASSERT(m_contents->hasOneClient());
    didMutate();
}

// FIXME: counter-style: we might need something similar for counter-style (rdar://103018993).
void CSSStyleSheet::didMutateRules(RuleMutationType mutationType, ContentsClonedForMutation contentsClonedForMutation, StyleRuleKeyframes* insertedKeyframesRule, const String& modifiedKeyframesRuleName)
{
    ASSERT(m_contents->isMutable());
    ASSERT(m_contents->hasOneClient());
    m_contents->setHasResolvedNesting(false);

    forEachStyleScope([&](Style::Scope& scope) {
        if ((mutationType == RuleInsertion || mutationType == RuleReplace) && contentsClonedForMutation == ContentsClonedForMutation::No && !scope.activeStyleSheetsContains(*this)) {
            if (insertedKeyframesRule) {
                if (RefPtr resolver = scope.resolverIfExists())
                    resolver->addKeyframeStyle(*insertedKeyframesRule);
                return;
            }
            scope.didChangeActiveStyleSheetCandidates();
            return;
        }

        if (mutationType == KeyframesRuleMutation) {
            if (RefPtr ownerDocument = this->ownerDocument())
                ownerDocument->keyframesRuleDidChange(modifiedKeyframesRuleName);
        }

        scope.didChangeStyleSheetContents();

        m_wasMutated = true;
    });
}

void CSSStyleSheet::didMutate()
{
    forEachStyleScope([&](auto& scope) {
        scope.didChangeStyleSheetContents();

        m_wasMutated = true;
    });
}

void CSSStyleSheet::forEachStyleScope(NOESCAPE const Function<void(Style::Scope&)>& apply)
{
    if (CheckedPtr scope = styleScope()) {
        apply(*scope);
        return;
    }
    for (Ref treeScope : m_adoptingTreeScopes)
        apply(styleScopeFor(treeScope));
}

void CSSStyleSheet::clearOwnerNode()
{
    Locker locker { m_opaqueRootLockForGC };
    m_ownerNode = nullptr;
}

WebCoreOpaqueRoot CSSStyleSheet::opaqueRootForGCThread()
{
    Locker locker { m_opaqueRootLockForGC };
    if (m_ownerNode)
        return root(m_ownerNode.get());
    if (SUPPRESS_UNCOUNTED_LOCAL SUPPRESS_UNCHECKED_LOCAL CSSImportRule* ownerRule = m_ownerRule.get()) {
        // Cannot ref on the GC thread, same as ownerRule above.
        if (SUPPRESS_UNCOUNTED_LOCAL auto* parentSheet = ownerRule->parentStyleSheet())
            return parentSheet->opaqueRootForGCThread();
    }
    return WebCoreOpaqueRoot { this };
}

CSSImportRule* CSSStyleSheet::ownerRule() const
{
    return m_ownerRule.get();
}

void CSSStyleSheet::clearOwnerRule()
{
    Locker locker { m_opaqueRootLockForGC };
    m_ownerRule = nullptr;
}

void CSSStyleSheet::reattachChildRuleCSSOMWrappers()
{
    for (unsigned i = 0; i < m_childRuleCSSOMWrappers.size(); ++i) {
        if (!m_childRuleCSSOMWrappers[i])
            continue;
        protect(m_childRuleCSSOMWrappers[i])->reattach(protect(*m_contents->ruleAt(i)));
    }
}

void CSSStyleSheet::setDisabled(bool disabled)
{ 
    if (disabled == m_isDisabled)
        return;
    m_isDisabled = disabled;

    forEachStyleScope([](auto& scope) {
        scope.didChangeActiveStyleSheetCandidates();
    });
}

void CSSStyleSheet::setMediaQueries(MQ::MediaQueryList&& mediaQueries)
{
    m_mediaQueries = WTF::move(mediaQueries);
}

unsigned CSSStyleSheet::length() const
{
    return m_contents->ruleCount();
}

CSSRule* CSSStyleSheet::item(unsigned index)
{
    unsigned ruleCount = length();
    if (index >= ruleCount)
        return nullptr;

    ASSERT(m_childRuleCSSOMWrappers.isEmpty() || m_childRuleCSSOMWrappers.size() == ruleCount);
    if (m_childRuleCSSOMWrappers.size() < ruleCount)
        m_childRuleCSSOMWrappers.grow(ruleCount);

    RefPtr<CSSRule>& cssRule = m_childRuleCSSOMWrappers[index];
    if (!cssRule)
        cssRule = protect(m_contents)->ruleAt(index)->createCSSOMWrapper(*this);
    return cssRule.get();
}

bool CSSStyleSheet::canAccessRules() const
{
    if (m_isOriginClean)
        return m_isOriginClean.value();

    URL baseURL = m_contents->baseURL();
    if (baseURL.isEmpty())
        return true;

    RefPtr document = ownerDocument();
    if (!document)
        return false;

    return protect(document->securityOrigin())->canRequest(baseURL, OriginAccessPatternsForWebProcess::singleton());
}

ExceptionOr<unsigned> CSSStyleSheet::insertRule(const String& ruleString, unsigned index)
{
    LOG_WITH_STREAM(StyleSheets, stream << "CSSStyleSheet " << this << " insertRule() " << ruleString << " at " << index);

    ASSERT(m_childRuleCSSOMWrappers.isEmpty() || m_childRuleCSSOMWrappers.size() == m_contents->ruleCount());

    if (!canAccessRules())
        return Exception { ExceptionCode::SecurityError, "Not allowed to insert rule into cross-origin stylesheet"_s };

    if (index > length())
        return Exception { ExceptionCode::IndexSizeError };
    RefPtr rule = CSSParser::parseRule(ruleString, m_contents.get().parserContext(), m_contents.ptr(), CSSParser::AllowedRules::ImportRules);

    if (!rule)
        return Exception { ExceptionCode::SyntaxError };

    if (m_wasConstructedByJS && rule->isImportRule())
        return Exception { ExceptionCode::SyntaxError, "Cannot inserted an @import rule in a constructed CSSStyleSheet object"_s };

    RuleMutationScope mutationScope(this, RuleInsertion, dynamicDowncast<StyleRuleKeyframes>(*rule));

    bool isNamespace = rule->isNamespaceRule();
    bool success = m_contents.get().wrapperInsertRule(rule.releaseNonNull(), index);
    if (!success) {
        if (isNamespace)
            return Exception { ExceptionCode::InvalidStateError };
        return Exception { ExceptionCode::HierarchyRequestError };
    }

    if (!m_childRuleCSSOMWrappers.isEmpty())
        m_childRuleCSSOMWrappers.insert(index, RefPtr<CSSRule>());

    return index;
}

ExceptionOr<void> CSSStyleSheet::deleteRule(unsigned index)
{
    LOG_WITH_STREAM(StyleSheets, stream << "CSSStyleSheet " << this << " deleteRule(" << index << ")");

    ASSERT(m_childRuleCSSOMWrappers.isEmpty() || m_childRuleCSSOMWrappers.size() == m_contents->ruleCount());

    if (!canAccessRules())
        return Exception { ExceptionCode::SecurityError, "Not allowed to delete rule from cross-origin stylesheet"_s };

    if (index >= length())
        return Exception { ExceptionCode::IndexSizeError };
    RuleMutationScope mutationScope(this);

    bool success = protect(m_contents)->wrapperDeleteRule(index);
    if (!success)
        return Exception { ExceptionCode::InvalidStateError };
    if (!m_childRuleCSSOMWrappers.isEmpty()) {
        if (m_childRuleCSSOMWrappers[index])
            m_childRuleCSSOMWrappers[index]->setParentStyleSheet(nullptr);
        m_childRuleCSSOMWrappers.removeAt(index);
    }

    return { };
}

ExceptionOr<int> CSSStyleSheet::addRule(const String& selector, const String& style, std::optional<unsigned> index)
{
    LOG_WITH_STREAM(StyleSheets, stream << "CSSStyleSheet " << this << " addRule() selector " << selector << " style " << style << " at " << index);

    auto text = makeString(selector, " { "_s, style, !style.isEmpty() ? " "_s : ""_s, '}');
    auto insertRuleResult = insertRule(text, index.value_or(length()));
    if (insertRuleResult.hasException())
        return insertRuleResult.releaseException();
    // As per Microsoft documentation, always return -1.
    return -1;
}

ExceptionOr<Ref<CSSRuleList>> CSSStyleSheet::cssRulesForBindings()
{
    auto cssRules = this->cssRules();
    if (!cssRules)
        return Exception { ExceptionCode::SecurityError, "Not allowed to access cross-origin stylesheet"_s };
    return cssRules.releaseNonNull();
}

RefPtr<CSSRuleList> CSSStyleSheet::cssRules()
{
    if (!canAccessRules())
        return nullptr;

    return cssRulesSkippingAccessCheck();
}

RefPtr<CSSRuleList> CSSStyleSheet::cssRulesSkippingAccessCheck()
{
    if (!m_ruleListCSSOMWrapper)
        m_ruleListCSSOMWrapper = makeUnique<StyleSheetCSSRuleList>(this);

    return m_ruleListCSSOMWrapper.get();
}

String CSSStyleSheet::href() const
{
    return m_contents->originalURL();
}

URL CSSStyleSheet::baseURL() const
{
    return m_contents->baseURL();
}

bool CSSStyleSheet::isLoading() const
{
    return m_contents->isLoading();
}

MediaList* CSSStyleSheet::media() const 
{
    if (!m_mediaCSSOMWrapper)
        m_mediaCSSOMWrapper = MediaList::create(const_cast<CSSStyleSheet*>(this));
    return m_mediaCSSOMWrapper.get();
}

CSSStyleSheet* CSSStyleSheet::parentStyleSheet() const 
{ 
    auto* ownerRule = m_ownerRule.get();
    return ownerRule ? ownerRule->parentStyleSheet() : nullptr;
}

Ref<CSSStyleSheet> CSSStyleSheet::rootStyleSheet()
{
    RefPtr root = this;
    while (root->parentStyleSheet())
        root = root->parentStyleSheet();
    return root.releaseNonNull();
}

Ref<const CSSStyleSheet> CSSStyleSheet::rootStyleSheet() const
{
    return const_cast<CSSStyleSheet&>(*this).rootStyleSheet();
}

Document* CSSStyleSheet::ownerDocument() const
{
    Ref root = rootStyleSheet();
    return root->ownerNode() ? &root->ownerNode()->document() : nullptr;
}

Style::Scope* CSSStyleSheet::styleScope()
{
    return m_styleScope.get();
}

void CSSStyleSheet::clearChildRuleCSSOMWrappers()
{
    m_childRuleCSSOMWrappers.clear();
}

String CSSStyleSheet::debugDescription() const
{
    return makeString("CSSStyleSheet "_s, "0x"_s, hex(reinterpret_cast<uintptr_t>(this), Lowercase), ' ', href());
}

String CSSStyleSheet::cssText(const CSS::SerializationContext& context)
{
    auto ruleList = cssRulesSkippingAccessCheck();
    if (!ruleList)
        return { };

    StringBuilder result;
    for (unsigned index = 0; index < ruleList->length(); ++index) {
        RefPtr rule = ruleList->item(index);
        if (!rule)
            continue;

        auto ruleText = rule->cssText(context);
        if (!result.isEmpty() && !ruleText.isEmpty())
            result.append(' ');

        result.append(ruleText);
    }
    return result.toString();
}

// https://w3c.github.io/csswg-drafts/cssom-1/#dom-cssstylesheet-replace
void CSSStyleSheet::replace(String&& text, Ref<DeferredPromise>&& promise)
{
    auto result = replaceSync(WTF::move(text));
    if (result.hasException())
        promise->reject(result.releaseException());
    promise->resolve<IDLInterface<CSSStyleSheet>>(*this);
}

// https://w3c.github.io/csswg-drafts/cssom-1/#dom-cssstylesheet-replacesync
ExceptionOr<void> CSSStyleSheet::replaceSync(String&& text)
{
    if (!m_wasConstructedByJS)
        return Exception { ExceptionCode::NotAllowedError, "This CSSStyleSheet object was not constructed by JavaScript"_s };

    // Try to use the cache in the case where contents is replaced before the stylesheet is attached to the document.
    if (isDetached() && m_childRuleCSSOMWrappers.isEmpty()) {
        auto key = Style::StyleSheetContentsCache::Key { text, m_contents->parserContext() };
        auto cachedContents = Style::StyleSheetContentsCache::singleton().get(key);
        if (cachedContents) {
            protect(m_contents)->unregisterClient(this);
            m_contents = *cachedContents;
            protect(m_contents)->registerClient(this);
        } else {
            protect(m_contents)->parseString(WTF::move(text));
            if (protect(m_contents)->isCacheable())
                Style::StyleSheetContentsCache::singleton().add(WTF::move(key), m_contents);
        }
        return { };
    }

    RuleMutationScope mutationScope(this, RuleReplace);
    protect(m_contents)->clearRules();
    for (auto& childRuleWrapper : m_childRuleCSSOMWrappers)
        if (childRuleWrapper)
            childRuleWrapper->setParentStyleSheet(nullptr);
    m_childRuleCSSOMWrappers.clear();

    protect(m_contents)->parseString(WTF::move(text));
    return { };
}

bool CSSStyleSheet::isDetached() const
{
    assertIsOwnerThread();
    return !m_ownerNode
        && !m_ownerRule
        && m_adoptingTreeScopes.isEmptyIgnoringNullReferences();
}

Document* CSSStyleSheet::constructorDocument() const
{
    return m_constructorDocument.get();
}

void CSSStyleSheet::addAdoptingTreeScope(ContainerNode& treeScope)
{
    ASSERT(is<Document>(treeScope) || is<ShadowRoot>(treeScope));
    m_adoptingTreeScopes.add(treeScope);
    styleScopeFor(treeScope).didChangeActiveStyleSheetCandidates();
}

void CSSStyleSheet::removeAdoptingTreeScope(ContainerNode& treeScope)
{
    ASSERT(is<Document>(treeScope) || is<ShadowRoot>(treeScope));
    m_adoptingTreeScopes.remove(treeScope);
    styleScopeFor(treeScope).didChangeStyleSheetContents();
}

void CSSStyleSheet::getChildStyleSheets(HashSet<Ref<CSSStyleSheet>>& childStyleSheets)
{
    RefPtr ruleList = cssRules();
    if (!ruleList)
        return;

    for (unsigned index = 0; index < ruleList->length(); ++index) {
        if (RefPtr rule = ruleList->item(index))
            rule->getChildStyleSheets(childStyleSheets);
    }
}

CSSStyleSheet::RuleMutationScope::RuleMutationScope(CSSStyleSheet* sheet, RuleMutationType mutationType, StyleRuleKeyframes* insertedKeyframesRule)
    : m_styleSheet(sheet)
    , m_mutationType(mutationType)
    , m_insertedKeyframesRule(insertedKeyframesRule)
{
    ASSERT(m_styleSheet);
    m_contentsClonedForMutation = m_styleSheet->willMutateRules();
}

CSSStyleSheet::RuleMutationScope::RuleMutationScope(CSSRule* rule)
    : m_styleSheet(rule ? rule->parentStyleSheet() : nullptr)
    , m_mutationType(is<CSSKeyframesRule>(rule) ? KeyframesRuleMutation : OtherMutation)
    , m_contentsClonedForMutation(ContentsClonedForMutation::No)
    , m_insertedKeyframesRule(nullptr)
    , m_modifiedKeyframesRuleName([rule] {
        auto* cssKeyframeRule = dynamicDowncast<CSSKeyframesRule>(rule);
        return cssKeyframeRule ? cssKeyframeRule->name() : emptyAtom();
    }())
{
    if (RefPtr styleSheet = m_styleSheet.get())
        m_contentsClonedForMutation = styleSheet->willMutateRules();
}

CSSStyleSheet::RuleMutationScope::~RuleMutationScope()
{
    if (RefPtr styleSheet = m_styleSheet.get()) {
        styleSheet->didMutateRules(m_mutationType, m_contentsClonedForMutation, m_insertedKeyframesRule.get(), m_modifiedKeyframesRuleName);
        styleSheet->contents().clearHasNestingRulesCache();
    }
}

}

```

## <a id="cssgroupingrule-cpp"></a>CSSGroupingRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSGroupingRule.cpp). Path: `Source/WebCore/css/CSSGroupingRule.cpp`. Source bytes: 8291; source lines: 223; SHA-256: `960e383b8ba17bf09f21ae42fca77cb21bb44594786f304b8f1d4c7a2561970c`.

```cpp
/*
 * Copyright (C) 2011 Adobe Systems Incorporated. All rights reserved.
 * Copyright (C) 2012-2024 Apple Inc. All rights reserved.
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
#include "CSSGroupingRule.h"

#include "CSSParser.h"
#include "CSSRuleList.h"
#include "CSSStyleSheet.h"
#include "StylePropertiesInlines.h"
#include "StyleRule.h"
#include "StyleSheetContents.h"
#include <wtf/text/StringBuilder.h>

namespace WebCore {

CSSGroupingRule::CSSGroupingRule(StyleRuleGroup& groupRule, CSSStyleSheet* parent)
    : CSSRule(parent)
    , m_groupRule(groupRule)
    , m_childRuleCSSOMWrappers(groupRule.childRules().size())
{
}

CSSGroupingRule::~CSSGroupingRule()
{
    ASSERT(m_childRuleCSSOMWrappers.size() == m_groupRule->childRules().size());
    for (auto& wrapper : m_childRuleCSSOMWrappers) {
        if (wrapper)
            wrapper->setParentRule(nullptr);
    }
}

ExceptionOr<unsigned> CSSGroupingRule::insertRule(const String& ruleString, unsigned index)
{
    ASSERT(m_childRuleCSSOMWrappers.size() == m_groupRule->childRules().size());

    if (index > m_groupRule->childRules().size()) {
        // IndexSizeError: Raised if the specified index is not a valid insertion point.
        return Exception { ExceptionCode::IndexSizeError };
    }

    RefPtr styleSheet = parentStyleSheet();
    auto nestedContextWithCurrentRule = [&] -> CSSParserEnum::NestedContext {
        if (m_groupRule->isStyleRule()) {
            ASSERT_NOT_REACHED(); // This is handled in CSSStyleRule.
            return CSSParserEnum::NestedContextType::Style;
        }
        if (m_groupRule->isScopeRule())
            return CSSParserEnum::NestedContextType::Scope;
        // Find the context in the ancestor chain.
        return nestedContext();
    }();
    RefPtr newRule = CSSParser::parseRule(ruleString, parserContext(), styleSheet ? &styleSheet->contents() : nullptr, CSSParser::AllowedRules::ImportRules, nestedContextWithCurrentRule);
    if (!newRule) {
        // CSSNestedDeclarations parsing is allowed if there is an ancestor style rule or an ancestor scope rule.
        if (!nestedContextWithCurrentRule)
            return Exception { ExceptionCode::SyntaxError };
        newRule = CSSParser::parseNestedDeclarations(ruleString, parserContext());
        if (!newRule)
            return Exception { ExceptionCode::SyntaxError };
    }

    if (newRule->isImportRule() || newRule->isNamespaceRule()) {
        // FIXME: an HierarchyRequestError should also be thrown for a @charset.
        // They are currently not getting parsed, resulting in a SyntaxError
        // to get raised above.

        // HierarchyRequestError: Raised if the rule cannot be inserted at the specified
        // index, e.g., if an @import rule is inserted after a standard rule set or other
        // at-rule.
        return Exception { ExceptionCode::HierarchyRequestError };
    }

    if (hasStyleRuleAncestor() && !newRule->isStyleRule() && !newRule->isGroupRule() && !newRule->isNestedDeclarationsRule())
        return Exception { ExceptionCode::HierarchyRequestError };

    CSSStyleSheet::RuleMutationScope mutationScope(this);

    protect(m_groupRule)->wrapperInsertRule(index, newRule.releaseNonNull());

    m_childRuleCSSOMWrappers.insert(index, RefPtr<CSSRule>());
    return index;
}

ExceptionOr<void> CSSGroupingRule::deleteRule(unsigned index)
{
    ASSERT(m_childRuleCSSOMWrappers.size() == m_groupRule->childRules().size());

    if (index >= m_groupRule->childRules().size()) {
        // IndexSizeError: Raised if the specified index does not correspond to a
        // rule in the media rule list.
        return Exception { ExceptionCode::IndexSizeError };
    }

    CSSStyleSheet::RuleMutationScope mutationScope(this);

    protect(m_groupRule)->wrapperRemoveRule(index);

    if (m_childRuleCSSOMWrappers[index])
        m_childRuleCSSOMWrappers[index]->setParentRule(nullptr);
    m_childRuleCSSOMWrappers.removeAt(index);

    return { };
}

void CSSGroupingRule::appendCSSTextForItemsInternal(StringBuilder& builder, StringBuilder& rules) const
{
    builder.append(" {"_s);
    if (rules.isEmpty()) {
        builder.append("\n}"_s);
        return;
    }

    builder.append(static_cast<StringView>(rules), "\n}"_s);
}

void CSSGroupingRule::appendCSSTextForItems(StringBuilder& builder) const
{
    StringBuilder rules;
    cssTextForRules(rules);
    appendCSSTextForItemsInternal(builder, rules);
}

void CSSGroupingRule::cssTextForRules(StringBuilder& rules) const
{
    auto& childRules = m_groupRule->childRules();
    for (unsigned index = 0; index < childRules.size(); ++index) {
        auto ruleText = protect(item(index))->cssText();
        if (!ruleText.isEmpty())
            rules.append("\n  "_s, WTF::move(ruleText));
    }
}

void CSSGroupingRule::appendCSSTextWithReplacementURLsForItems(StringBuilder& builder, const CSS::SerializationContext& context) const
{
    StringBuilder rules;
    cssTextForRulesWithReplacementURLs(rules, context);
    appendCSSTextForItemsInternal(builder, rules);
}

void CSSGroupingRule::cssTextForRulesWithReplacementURLs(StringBuilder& rules, const CSS::SerializationContext& context) const
{
    auto& childRules = m_groupRule->childRules();
    for (unsigned index = 0; index < childRules.size(); index++) {
        RefPtr wrappedRule = item(index);
        rules.append("\n  "_s, wrappedRule->cssText(context));
    }
}

RefPtr<StyleRuleWithNesting> CSSGroupingRule::prepareChildStyleRuleForNesting(StyleRule& styleRule)
{
    CSSStyleSheet::RuleMutationScope scope(this);
    auto& rules = m_groupRule->m_childRules;
    for (size_t i = 0 ; i < rules.size() ; i++) {
        if (rules[i].ptr() == &styleRule) {
            auto styleRuleWithNesting = StyleRuleWithNesting::create(WTF::move(styleRule));
            rules[i] = styleRuleWithNesting;
            return styleRuleWithNesting;
        }        
    }
    return { };
}

unsigned CSSGroupingRule::length() const
{ 
    return m_groupRule->childRules().size(); 
}

CSSRule* CSSGroupingRule::item(unsigned index) const
{ 
    if (index >= length())
        return nullptr;
    ASSERT(m_childRuleCSSOMWrappers.size() == m_groupRule->childRules().size());
    auto& rule = m_childRuleCSSOMWrappers[index];
    if (!rule)
        rule = protect(m_groupRule->childRules()[index])->createCSSOMWrapper(const_cast<CSSGroupingRule&>(*this));
    return rule.get();
}

CSSRuleList& CSSGroupingRule::cssRules() const
{
    if (!m_ruleListCSSOMWrapper)
        lazyInitialize(m_ruleListCSSOMWrapper, makeUniqueWithoutRefCountedCheck<LiveCSSRuleList<CSSGroupingRule>>(const_cast<CSSGroupingRule&>(*this)));
    return *m_ruleListCSSOMWrapper;
}

void CSSGroupingRule::reattach(StyleRuleBase& rule)
{
    m_groupRule = downcast<StyleRuleGroup>(rule);
    for (unsigned i = 0; i < m_childRuleCSSOMWrappers.size(); ++i) {
        if (m_childRuleCSSOMWrappers[i])
            protect(m_childRuleCSSOMWrappers[i])->reattach(m_groupRule->childRules()[i]);
    }
}

} // namespace WebCore

```
