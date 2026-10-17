# WebKit fresh Document focus initialization

Implementation pin: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. These source excerpts support a bounded native initialization choice; they do not equate WebKit's Element pointer with HTML's full focused-area model.

The selected Surgeist contract is recorded in #1220. HTML877d permits a null designation in its resolver but does not explicitly initialize it. Fresh known-null is distinct from unavailable imported history and from system-focus absence.

## Source/WebCore/dom/Document.h

[Pinned source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Document.h); SHA-256 `d834dcea6916301c50febc380c74c28f0162c72c75b510b34d4b3feb409fa542`.

Source license/copyright header, retained verbatim:

```text
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 *           (C) 2006 Alexey Proskuryakov (ap@webkit.org)
 * Copyright (C) 2004-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2008, 2009 Torch Mobile Inc. All rights reserved. (http://www.torchmobile.com/)
 * Copyright (C) 2010 Nokia Corporation and/or its subsidiary(-ies)
 * Copyright (C) 2011 Google Inc. All rights reserved.
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
 *
 */
```

Original lines 1009–1009; verbatim C++:

```cpp
    Element* focusedElement() const { return m_focusedElement.get(); }
```

Original lines 2367–2367; verbatim C++:

```cpp
    RefPtr<Element> m_focusedElement;
```

## Source/WebCore/dom/Document.cpp

[Pinned source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Document.cpp); SHA-256 `c3f64c3d98f23601ffa74b2e63b82f30aaeb4035bb537c657475a8c00095c55c`.

Source license/copyright header, retained verbatim:

```text
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
```

Original lines 677–766; verbatim C++:

```cpp
Document::Document(LocalFrame* frame, const Settings& settings, const URL& url, DocumentClasses documentClasses, OptionSet<ConstructionFlag> constructionFlags, std::optional<ScriptExecutionContextIdentifier> identifier)
    : ContainerNode(*this, NodeType::Document)
    , TreeScope(*this)
    , ScriptExecutionContext(Type::Document, identifier)
    , FrameDestructionObserver(frame)
    , m_settings(settings)
    , m_speculationRules(SpeculationRules::create())
    , m_parserContentPolicy(DefaultParserContentPolicy)
    , m_creationURL(url)
    , m_domTreeVersion(++s_globalTreeVersion)
    , m_styleScope(makeUniqueRef<Style::DocumentScope>(*this))
    , m_styleRecalcTimer([this] { Ref { *this }->updateStyleIfNeeded(); })
#if !LOG_DISABLED
    , m_documentCreationTime(MonotonicTime::now())
#endif
#if ENABLE(XSLT)
    , m_applyPendingXSLTransformsTimer(*this, &Document::applyPendingXSLTransformsTimerFired)
#endif
    , m_xmlVersion("1.0"_s)
    , m_intersectionObserversInitialUpdateTimer(*this, &Document::intersectionObserversInitialUpdateTimerFired)
    , m_loadEventDelayTimer(*this, &Document::loadEventDelayTimerFired)
#if PLATFORM(IOS_FAMILY) && ENABLE(DEVICE_ORIENTATION)
    , m_deviceMotionClient(makeUnique<DeviceMotionClientIOS>(page() ? page()->deviceOrientationUpdateProvider() : nullptr))
    , m_deviceMotionController(makeUnique<DeviceMotionController>(*m_deviceMotionClient))
    , m_deviceOrientationClient(makeUnique<DeviceOrientationClientIOS>(page() ? page()->deviceOrientationUpdateProvider() : nullptr))
    , m_deviceOrientationController(makeUnique<DeviceOrientationController>(*m_deviceOrientationClient))
#endif
    , m_pendingTasksTimer(*this, &Document::pendingTasksTimerFired)
    , m_visualUpdatesSuppressionTimer(*this, &Document::visualUpdatesSuppressionTimerFired)
    , m_sharedObjectPoolClearTimer(*this, &Document::clearSharedObjectPool)
    , m_didAssociateFormControlsTimer(*this, &Document::didAssociateFormControlsTimerFired)
    , m_cookieCacheExpiryTimer(*this, &Document::invalidateDOMCookieCache)
    , m_socketProvider(page() ? &page()->socketProvider() : nullptr)
    , m_selection(makeUniqueRef<FrameSelection>(this))
    , m_fragmentDirectiveForBindings(FragmentDirective::create())
    , m_documentClasses(documentClasses)
    , m_latestFocusTrigger { FocusTrigger::Other }
    , m_isSynthesized(constructionFlags.contains(ConstructionFlag::Synthesized))
    , m_isNonRenderedPlaceholder(constructionFlags.contains(ConstructionFlag::NonRenderedPlaceholder))
    , m_frameIdentifier(frame ? std::optional(frame->frameID()) : std::nullopt)
    , m_syncData(DocumentSyncData::create())
{
    setEventTargetFlag(EventTargetFlag::IsConnected);
    addToDocumentsMap();

    // We depend on the url getting immediately set in subframes, but we
    // also depend on the url NOT getting immediately set in opened windows.
    // See fast/dom/early-frame-url.html
    // and fast/dom/location-new-window-no-crash.html, respectively.
    // FIXME: Can/should we unify this behavior?
    if ((frame && frame->ownerElement()) || !url.isEmpty())
        setURL(URL { url });

    if (!frame)
        setUsesNullCustomElementRegistry();

    initSecurityContext();

    InspectorInstrumentation::addEventListenersToNode(*this);
    setStorageBlockingPolicy(m_settings->storageBlockingPolicy());

    ASSERT(!m_quirks);
    ASSERT(!m_cachedResourceLoader);
    ASSERT(!m_extensionStyleSheets);
    ASSERT(!m_visitedLinkState);
    ASSERT(!m_markers);
    ASSERT(!m_scriptRunner);
    ASSERT(!m_moduleLoader);
#if ENABLE(FULLSCREEN_API)
    ASSERT(!m_fullscreen);
#endif
#if ENABLE(MODEL_ELEMENT_IMMERSIVE)
    ASSERT(!m_immersive);
#endif
    ASSERT(!m_fontSelector);
    ASSERT(!m_fontLoader);
    ASSERT(!m_undoManager);
    ASSERT(!m_editor);
    ASSERT(!m_reportingScope);
    ASSERT(!hasEmptySecurityOriginPolicyAndContentSecurityPolicy() || !hasInitializedSecurityOriginPolicyOrContentSecurityPolicy());
    m_constructionDidFinish = true;

    // We walk all of the relevant enums to popular one at a time in a switch statement to make sure
    // that an engineer writes the relevant manual code whenever a new generated type is added.
    for (const DocumentSyncDataType dataType : allDocumentSyncDataTypes)
        populateDocumentSyncDataForNewlyConstructedDocument(dataType);

    if (!settings.mutationEventsEnabled())
        m_shouldNotFireMutationEvents = true;
}
```

## Source/WTF/wtf/RefPtr.h

[Pinned source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WTF/wtf/RefPtr.h); SHA-256 `fd733fbdf971011e46551aa49906d9b8c726ce990936cd5d176cdeba10085ee9`.

Source license/copyright header, retained verbatim:

```text
/*
 *  Copyright (C) 2005-2025 Apple Inc. All rights reserved.
 *
 *  This library is free software; you can redistribute it and/or
 *  modify it under the terms of the GNU Library General Public
 *  License as published by the Free Software Foundation; either
 *  version 2 of the License, or (at your option) any later version.
 *
 *  This library is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 *  Library General Public License for more details.
 *
 *  You should have received a copy of the GNU Library General Public License
 *  along with this library; see the file COPYING.LIB.  If not, write to
 *  the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 *  Boston, MA 02110-1301, USA.
 *
 */
```

Original lines 94–95; verbatim C++:

```cpp
    ALWAYS_INLINE constexpr RefPtr() : m_ptr(nullptr) { }
    ALWAYS_INLINE constexpr RefPtr(std::nullptr_t) : m_ptr(nullptr) { }
```

## Current standard comparison

On 2026-10-17, the [current HTML focus data model](https://html.spec.whatwg.org/multipage/interaction.html#focused-area-of-the-document) still declares the designation and tests for null in the top-level resolver, without specifying a fresh initialization in those clauses. This comparison supplies no new normative default; the pinned HTML model and the explicit native choice remain authoritative.
