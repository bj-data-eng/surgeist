# Focus editorial-gap implementation evidence

WebKit implementation pin `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Selected excerpts are corroboration, not a verbatim implementation of HTML focus-chain algorithms. The decisions belong to issue #1220.

## Source/WebCore/dom/Document.cpp

[Pinned source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Document.cpp), SHA-256 `c3f64c3d98f23601ffa74b2e63b82f30aaeb4035bb537c657475a8c00095c55c`.

Original copyright/license header:

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

Original lines 6630–6682, verbatim:

```cpp
    // Make sure newFocusedElement is actually in this document
    if (newFocusedElement && (&newFocusedElement->document() != this))
        return true;

    if (m_focusedElement == newFocusedElement)
        return true;

    if (backForwardCacheState() != NotInBackForwardCache)
        return false;

    RefPtr oldFocusedElement = std::exchange(m_focusedElement, nullptr);

    // Remove focus from the existing focus node (if any)
    if (oldFocusedElement) {
        bool focusChangeBlocked = false;

        oldFocusedElement->setFocus(false);
        setFocusNavigationStartingNode(nullptr);

        scheduleContentRelevancyUpdate(ContentRelevancy::Focused);

        if (options.removalEventsMode == FocusRemovalEventsMode::Dispatch) {
            // Dispatch a change event for form control elements that have been edited.
            if (RefPtr formControlElement = dynamicDowncast<HTMLFormControlElement>(*oldFocusedElement)) {
                if (formControlElement->wasChangedSinceLastFormControlChangeEvent())
                    formControlElement->dispatchFormControlChangeEvent();
            }

            // Dispatch the blur event and let the node do any other blur related activities (important for text fields)
            if (RefPtr page = this->page(); page && page->focusController().isFocused())
                oldFocusedElement->dispatchBlurEvent(newFocusedElement);

            if (m_focusedElement) {
                // handler shifted focus
                focusChangeBlocked = true;
                newFocusedElement = nullptr;
            }

            oldFocusedElement->dispatchFocusOutEventIfNeeded(newFocusedElement); // DOM level 3 bubbling blur event.

            if (m_focusedElement) {
                // handler shifted focus
                focusChangeBlocked = true;
                newFocusedElement = nullptr;
            }
        } else {
            // Match the order in HTMLTextFormControlElement::dispatchBlurEvent.
            if (RefPtr inputElement = dynamicDowncast<HTMLInputElement>(*oldFocusedElement))
                inputElement->endEditing();
            if (RefPtr page = this->page())
                page->chrome().client().elementDidBlur(*oldFocusedElement);
            ASSERT(!m_focusedElement);
        }
```

Original lines 6716–6748, verbatim:

```cpp
            // Bluring oldFocusedElement may have moved newFocusedElement across documents.
            return false;
        }
        if (newFocusedElement->isRootEditableElement() && !acceptsEditingFocus(*newFocusedElement)) {
            // delegate blocks focus change
            return false;
        }
        // Set focus on the new node
        m_focusedElement = newFocusedElement;
        RefPtr focusedElement = m_focusedElement;
        setFocusNavigationStartingNode(focusedElement.get());
        focusedElement->setFocus(true, options.visibility);
        if (options.trigger != FocusTrigger::Bindings)
            m_latestFocusTrigger = options.trigger;

        scheduleContentRelevancyUpdate(ContentRelevancy::Focused);

        // The setFocus call triggers a blur and a focus event. Event handlers could cause the focused element to be cleared.
        if (m_focusedElement != focusedElement) {
            // handler shifted focus
            return false;
        }

        // Dispatch the focus event and let the node do any other focus related activities (important for text fields)
        if (RefPtr page = this->page(); page && page->focusController().isFocused())
            focusedElement->dispatchFocusEvent(oldFocusedElement.copyRef(), options);

        if (m_focusedElement != focusedElement) {
            // handler shifted focus
            return false;
        }

        focusedElement->dispatchFocusInEventIfNeeded(oldFocusedElement.copyRef()); // DOM level 3 bubbling focus event.
```

## Source/WebCore/dom/Element.cpp

[Pinned source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Element.cpp), SHA-256 `d386258a8ce898fd42ece4365bad4842a30cc4c316c481ea665db8914777cb57`.

Original copyright/license header:

```text
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Peter Kelly (pmk@post.com)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 *           (C) 2007 David Smith (catfish.man@gmail.com)
 * Copyright (C) 2004-2026 Apple Inc. All rights reserved.
 *           (C) 2007 Eric Seidel (eric@webkit.org)
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

Original lines 4323–4335, verbatim:

```cpp
    if (RefPtr root = shadowRootWithDelegatesFocus(*this)) {
        RefPtr currentlyFocusedElement = document->focusedElement();
        if (root->isShadowIncludingInclusiveAncestorOf(currentlyFocusedElement.get())) {
            if (RefPtr page = document->page())
                page->chrome().client().elementDidRefocus(*currentlyFocusedElement, options);
            return;
        }

        newTarget = findFocusDelegateForTarget(*root, options.trigger);
        if (!newTarget)
            return;
    } else if (!isProgramaticallyFocusable(*newTarget)) {
        if (this == document->documentElement())
```

## Source/WebCore/page/FocusController.cpp

[Pinned source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/page/FocusController.cpp), SHA-256 `da8704209addbcea2bbdf58daeae6c1a3bf0aa1d20f640b84b830aa64d56a849`.

Original copyright/license header:

```text
/*
 * Copyright (C) 2006, 2007, 2013 Apple Inc. All rights reserved.
 * Copyright (C) 2008 Nuanti Ltd.
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
```

Original lines 409–441, verbatim:

```cpp
static inline void dispatchEventsOnWindowAndFocusedElement(Document* document, bool focused)
{
    // If we have a focused node we should dispatch blur on it before we blur the window.
    // If we have a focused node we should dispatch focus on it after we focus the window.
    // https://bugs.webkit.org/show_bug.cgi?id=27105

    // Do not fire events while modal dialogs are up.  See https://bugs.webkit.org/show_bug.cgi?id=33962
    if (Page* page = document->page()) {
        if (page->defersLoading())
            return;
    }

    if (RefPtr focusedElement = document->focusedElement(); !focused && focusedElement) {
        if (focusedElement->transferredFocusToPicker()) {
            // The webpage lost focus because the focused element transferred focus to
            // a non-web-content picker when it was activated. We don't want to post any
            // web-exposed events (e.g. blur) in these cases, so return.
            focusedElement->didSuppressBlurDueToPickerFocusTransfer();
            return;
        }

        if (RefPtr formControlElement = dynamicDowncast<HTMLFormControlElement>(*focusedElement)) {
            if (formControlElement->wasChangedSinceLastFormControlChangeEvent())
                formControlElement->dispatchFormControlChangeEvent();
        }

        focusedElement->dispatchBlurEvent(nullptr);
    }

    document->dispatchWindowEvent(Event::create(focused ? eventNames().focusEvent : eventNames().blurEvent, Event::CanBubble::No, Event::IsCancelable::No));
    if (RefPtr focusedElement = document->focusedElement(); focused && focusedElement)
        focusedElement->dispatchFocusEvent(nullptr, { });
}
```

## Evidence limits

Document updates its own focused Element and dispatches Element focus/blur; FocusController separately dispatches Window events. The shadow delegation code accepts a nullable stored focused Element before delegate search. That stored pointer is not the standard system-focus-qualified current-area query, so it does not establish behavior when system focus is absent. The null handling selected in #1220 is an explicit local interpretation of the pinned source domains.

## Current standard comparison

The [current HTML focus processing model](https://html.spec.whatwg.org/multipage/interaction.html#focus-update-steps), checked 2026-10-17, retains the nullable anchor projection, the traversable-only current-chain condition, the unbound document in designation, and the ambiguous event-target conditionals. No upstream correction is claimed. The existing retained HTML877d host-facts reference remains the normative source; its literal wording is unchanged.

## Child Document to parent focus scope

Additional original FocusController.cpp lines349–365 at the same pin and hash:

```cpp
Variant<RefPtr<Element>, RefPtr<Frame>> FocusNavigationScope::owner() const
{
    if (m_slotElement)
        return m_slotElement.get();

    ASSERT(m_treeScopeRootNode);
    if (RefPtr shadowRoot = dynamicDowncast<ShadowRoot>(*m_treeScopeRootNode))
        return shadowRoot->host();
    if (invokerForOpenPopover(m_treeScopeRootNode.get()))
        return downcast<Element>(m_treeScopeRootNode.get());
    if (RefPtr frame = m_treeScopeRootNode->document().frame()) {
        if (RefPtr ownerElement = frame->ownerElement())
            return RefPtr<Element> { ownerElement };
        return RefPtr<Frame> { frame->parent() };
    }
    return RefPtr<Element> { nullptr };
}
```

This scope-navigation helper prefers the actual frame owner Element when present. It corroborates the container-Element boundary; it is not a verbatim implementation of HTML focus-chain. The selected correction in #1220 uses the child navigable’s container and never appends a raw Navigable. Literal pinned HTML chain and navigable/container definitions remain in the existing retained host-facts reference.
