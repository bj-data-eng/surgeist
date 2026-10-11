# Pinned WebKit transition-length source witnesses

Complete immutable reference source files at `73aa6c89e2cb77c46184a81aec944e4ab99d114d` from [WebKit](https://github.com/WebKit/WebKit/tree/73aa6c89e2cb77c46184a81aec944e4ab99d114d), captured 2026-10-11. These are implementation witnesses, not normative CSS. Source copyright and complete license headers remain byte-for-byte inside each code block; [bundled WebKit legal material](../licenses/webkit/NOTICE.md) accompanies them. Added provenance is non-normative. No source code was incorporated into the product.

## Source/WebCore/dom/ViewTransition.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/ViewTransition.cpp). Source bytes: 46825; source lines: 1168; SHA-256 `5e947a60fa7fe56d4dd315607b689a5bc390e8a558f54af8d5f98020bd868d4c`.

```cpp
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
#include "ViewTransition.h"

#include "CSSCustomIdentValue.h"
#include "CSSFunctionValue.h"
#include "CSSKeyframeRule.h"
#include "CSSKeyframesRule.h"
#include "CSSTransformListValue.h"
#include "CSSValuePool.h"
#include "CheckVisibilityOptions.h"
#include "ContainerNodeInlines.h"
#include "ContextDestructionObserverInlines.h"
#include "DocumentEventLoop.h"
#include "DocumentTimeline.h"
#include "DocumentView.h"
#include "ElementInlines.h"
#include "FrameSnapshotting.h"
#include "HostWindow.h"
#include "JSDOMPromise.h"
#include "JSDOMPromiseDeferred.h"
#include "LayoutRect.h"
#include "Logging.h"
#include "PlatformScreen.h"
#include "PseudoElementRequest.h"
#include "RenderBoxInlines.h"
#include "RenderElementInlines.h"
#include "RenderFragmentedFlow.h"
#include "RenderInline.h"
#include "RenderLayer.h"
#include "RenderLayerModelObject.h"
#include "RenderObjectInlines.h"
#include "RenderView.h"
#include "RenderViewTransitionCapture.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "StyleDocumentScope.h"
#include "StyleExtractor.h"
#include "StyleResolver.h"
#include "StyleTransformFunction.h"
#include "StyleZoomPrimitivesInlines.h"
#include "Styleable.h"
#include "TransformState.h"
#include "ViewTransitionTypeSet.h"
#include "WebAnimation.h"
#include <wtf/OrderedHashSet.h>
#include <wtf/TZoneMallocInlines.h>
#include <wtf/text/MakeString.h>
#include <wtf/text/TextStream.h>

namespace WebCore {

WTF_MAKE_TZONE_ALLOCATED_IMPL(CapturedElement);
WTF_MAKE_TZONE_ALLOCATED_IMPL(ViewTransitionParams);
WTF_MAKE_TZONE_ALLOCATED_IMPL(ViewTransition);

ViewTransition::ViewTransition(Document& document, RefPtr<ViewTransitionUpdateCallback>&& updateCallback, Vector<AtomString>&& initialActiveTypes)
    : ActiveDOMObject(document)
    , m_updateCallback(WTF::move(updateCallback))
    , m_ready(createPromiseAndWrapper(document))
    , m_updateCallbackDone(createPromiseAndWrapper(document))
    , m_finished(createPromiseAndWrapper(document))
    , m_types(ViewTransitionTypeSet::create(document, WTF::move(initialActiveTypes)))
{
    document.registerForVisibilityStateChangedCallbacks(*this);
}

ViewTransition::ViewTransition(Document& document, Vector<AtomString>&& initialActiveTypes)
    : ActiveDOMObject(document)
    , m_isCrossDocument(true)
    , m_ready(createPromiseAndWrapper(document))
    , m_updateCallbackDone(createPromiseAndWrapper(document))
    , m_finished(createPromiseAndWrapper(document))
    , m_types(ViewTransitionTypeSet::create(document, WTF::move(initialActiveTypes)))
{
}


ViewTransition::~ViewTransition() = default;

Ref<ViewTransition> ViewTransition::createSamePage(Document& document, RefPtr<ViewTransitionUpdateCallback>&& updateCallback, Vector<AtomString>&& initialActiveTypes)
{
    Ref viewTransition = adoptRef(*new ViewTransition(document, WTF::move(updateCallback), WTF::move(initialActiveTypes)));

    LOG_WITH_STREAM(ViewTransitions, stream << "ViewTransition::createSamePage created transition " << viewTransition.ptr());

    viewTransition->suspendIfNeeded();
    return viewTransition;
}

// https://www.w3.org/TR/css-view-transitions-2/#resolve-inbound-cross-document-view-transition
RefPtr<ViewTransition> ViewTransition::resolveInboundCrossDocumentViewTransition(Document& document, std::unique_ptr<ViewTransitionParams> inboundViewTransitionParams)
{
    if (!inboundViewTransitionParams)
        return nullptr;

    if (MonotonicTime::now() - inboundViewTransitionParams->startTime > defaultTimeout)
        return nullptr;

    // Re-check against the new document's final origin, which may differ from the URL-derived
    // origin used by DocumentLoader::navigationCanTriggerCrossDocumentViewTransition.
    if (!inboundViewTransitionParams->oldDocumentOrigin
        || !inboundViewTransitionParams->oldDocumentOrigin->isSameOriginAs(document.securityOrigin()))
        return nullptr;

    if (document.activeViewTransition())
        return nullptr;

    auto types = document.resolveViewTransitionRule();
    if (std::holds_alternative<Document::SkipTransition>(types))
        return nullptr;

    RefPtr viewTransition = adoptRef(*new ViewTransition(document, WTF::move(std::get<Vector<AtomString>>(types))));
    viewTransition->suspendIfNeeded();

    viewTransition->m_namedElements.swap(inboundViewTransitionParams->namedElements);
    viewTransition->m_initialLargeViewportSize = inboundViewTransitionParams->initialLargeViewportSize;
    viewTransition->m_initialPageZoom = inboundViewTransitionParams->initialPageZoom;

    document.setActiveViewTransition(protect(viewTransition));

    protect(viewTransition->m_updateCallbackDone.second)->resolve();
    viewTransition->m_phase = ViewTransitionPhase::UpdateCallbackCalled;

    return viewTransition;
}

// https://drafts.csswg.org/css-view-transitions-2/#setup-cross-document-view-transition
Ref<ViewTransition> ViewTransition::setupCrossDocumentViewTransition(Document& document)
{
    auto types = document.resolveViewTransitionRule();
    ASSERT(!std::holds_alternative<Document::SkipTransition>(types));

    if (RefPtr activeViewTransition =  document.activeViewTransition())
        activeViewTransition->skipViewTransition(Exception { ExceptionCode::AbortError, "Old view transition aborted by new view transition."_s });

    Ref viewTransition = adoptRef(*new ViewTransition(document, WTF::move(std::get<Vector<AtomString>>(types))));
    viewTransition->suspendIfNeeded();

    document.setActiveViewTransition(protect(viewTransition.ptr()));

    return viewTransition;
}

DOMPromise& ViewTransition::ready()
{
    return m_ready.first.get();
}

DOMPromise& ViewTransition::updateCallbackDone()
{
    return m_updateCallbackDone.first.get();
}

DOMPromise& ViewTransition::finished()
{
    return m_finished.first.get();
}

// https://drafts.csswg.org/css-view-transitions/#skip-the-view-transition
void ViewTransition::skipViewTransition(ExceptionOr<JSC::JSValue>&& reason)
{
    if (!document())
        return;

    LOG_WITH_STREAM(ViewTransitions, stream << "ViewTransition " << this << " skipViewTransition - phase " << m_phase);

    ASSERT(m_phase != ViewTransitionPhase::Done);

    Ref document = *this->document();
    if (m_phase < ViewTransitionPhase::UpdateCallbackCalled) {
        protect(document->eventLoop())->queueTask(TaskSource::DOMManipulation, [weakThis = WeakPtr { *this }] {
            RefPtr protectedThis = weakThis.get();
            if (protectedThis && protect(protectedThis->document())->globalObject())
                protectedThis->callUpdateCallback();
        });

        if (m_isCrossDocument)
            protect(m_updateCallbackDone.second)->resolve();
    }

    document->clearRenderingIsSuppressedForViewTransition();

    if (document->activeViewTransition() == this)
        clearViewTransition();

    m_phase = ViewTransitionPhase::Done;

    if (reason.hasException())
        protect(m_ready.second)->reject(reason.releaseException());
    else {
        protect(m_ready.second)->rejectWithCallback([&] (auto&) {
            return reason.releaseReturnValue();
        }, RejectAsHandled::Yes);
    }

    protect(m_updateCallbackDone.first)->whenSettled([this, protectedThis = Ref { *this }] {
        if (isContextStopped())
            return;

        switch (protect(m_updateCallbackDone.first)->status()) {
        case DOMPromise::Status::Fulfilled:
            protect(m_finished.second)->resolve();
            break;
        case DOMPromise::Status::Rejected:
            protect(m_finished.second)->rejectWithCallback([this, protectedThis = Ref { *this }] (auto&) {
            return m_updateCallbackDone.first->result();
            }, RejectAsHandled::Yes);
            break;
        case DOMPromise::Status::Pending:
            ASSERT_NOT_REACHED();
            break;
        }
    });
}

bool ViewTransition::virtualHasPendingActivity() const
{
    return m_phase != ViewTransitionPhase::Done;
}

// https://drafts.csswg.org/css-view-transitions/#ViewTransition-skipTransition
void ViewTransition::skipTransition()
{
    if (m_phase != ViewTransitionPhase::Done)
        skipViewTransition(Exception { ExceptionCode::AbortError, "Skipping view transition because skipTransition() was called."_s });
}

// https://drafts.csswg.org/css-view-transitions/#call-dom-update-callback-algorithm
void ViewTransition::callUpdateCallback()
{
    if (!document())
        return;

    LOG_WITH_STREAM(ViewTransitions, stream << "ViewTransition " << this << " callUpdateCallback");

    ASSERT(m_phase < ViewTransitionPhase::UpdateCallbackCalled || m_phase == ViewTransitionPhase::Done);

    if (m_phase != ViewTransitionPhase::Done)
        m_phase = ViewTransitionPhase::UpdateCallbackCalled;

    if (m_isCrossDocument)
        return;

    Ref document = *this->document();
    Ref callbackPromise = [&] -> Ref<DOMPromise> {
        if (!m_updateCallback) {
            auto promiseAndWrapper = createPromiseAndWrapper(document);
            protect(promiseAndWrapper.second)->resolve();
            return WTF::move(promiseAndWrapper.first);
        }

        auto result = m_updateCallback->invoke();
        if (result.type() != CallbackResultType::Success || result.returnValue()->isSuspended()) {
            auto promiseAndWrapper = createPromiseAndWrapper(document);
            // FIXME: First case should reject with `ExceptionCode::ExistingExceptionError`.
            if (result.type() == CallbackResultType::ExceptionThrown)
                protect(promiseAndWrapper.second)->reject(ExceptionCode::TypeError);
            else
                protect(promiseAndWrapper.second)->reject();
            return WTF::move(promiseAndWrapper.first);
        }

        return result.releaseReturnValue();
    }();

    callbackPromise->whenSettledWithResult([weakThis = WeakPtr { *this }](auto*, bool isFulfilled, auto result) mutable {
        RefPtr protectedThis = weakThis.get();
        if (!protectedThis)
            return;
        protectedThis->m_updateCallbackTimeout = nullptr;
        if (isFulfilled) {
            protect(protectedThis->m_updateCallbackDone.second)->resolve();
            protectedThis->activateViewTransition();
            return;
        }

        protect(protectedThis->m_updateCallbackDone.second)->rejectWithCallback([&result] (auto&) {
            return result;
        }, RejectAsHandled::No);
        if (protectedThis->m_phase == ViewTransitionPhase::Done)
            return;
        protectedThis->m_ready.second->markAsHandled();
        protectedThis->skipViewTransition(WTF::move(result));
    });

    m_updateCallbackTimeout = protect(document->eventLoop())->scheduleTask(defaultTimeout, TaskSource::DOMManipulation, [weakThis = WeakPtr { *this }] {
        RefPtr protectedThis = weakThis.get();
        LOG_WITH_STREAM(ViewTransitions, stream << "ViewTransition " << protectedThis.get() << " update callback timed out");
        if (!protectedThis)
            return;
        if (protectedThis->m_phase == ViewTransitionPhase::Done)
            return;
        protectedThis->skipViewTransition(Exception { ExceptionCode::TimeoutError, "View transition update callback timed out."_s });
    });
}

// https://drafts.csswg.org/css-view-transitions/#setup-view-transition-algorithm
void ViewTransition::setupViewTransition()
{
    if (!document())
        return;

    ASSERT(m_phase == ViewTransitionPhase::PendingCapture);

    m_phase = ViewTransitionPhase::CapturingOldState;

    auto checkFailure = captureOldState();
    if (checkFailure.hasException()) {
        skipViewTransition(checkFailure.releaseException());
        return;
    }

    Ref document = *this->document();
    if (m_isCrossDocument)
        document->setRenderingIsSuppressedForViewTransitionImmediately();
    else
        document->setRenderingIsSuppressedForViewTransitionAfterUpdateRendering();

    protect(document->eventLoop())->queueTask(TaskSource::DOMManipulation, [weakThis = WeakPtr { *this }] {
        RefPtr protectedThis = weakThis.get();
        if (!protectedThis)
            return;
        if (protectedThis->m_phase == ViewTransitionPhase::Done)
            return;

        protectedThis->callUpdateCallback();
    });
}

static AtomString effectiveViewTransitionName(RenderLayerModelObject& renderer, Element& originatingElement, Style::Scope& documentScope, bool isCrossDocument)
{
    if (renderer.isSkippedContent())
        return nullAtom();

    auto& transitionName = renderer.style().viewTransitionName();

    auto computeScope = [&] -> Style::Scope* {
        SUPPRESS_UNCHECKED_LOCAL auto scope = Style::Scope::forOrdinal(originatingElement, transitionName.scopeOrdinal());
        if (!scope || scope != &documentScope)
            return nullptr;
        return scope;
    };

    return WTF::switchOn(transitionName,
        [&](const CSS::Keyword::None&) {
            return nullAtom();
        },
        [&](const CSS::Keyword::Auto&) {
            SUPPRESS_UNCHECKED_LOCAL auto scope = computeScope();
            if (!scope || !renderer.element())
                return nullAtom();

            Ref element = *renderer.element();
            if (scope == &Style::Scope::forNode(element) && element->hasID())
                return makeAtomString("-ua-id-"_s, renderer.element()->getIdAttribute());

            if (isCrossDocument)
                return nullAtom();

            return makeAtomString("-ua-auto-"_s, String::number(element->nodeIdentifier().toUInt64()));
        },
        [&](const CSS::Keyword::MatchElement&) {
            SUPPRESS_UNCHECKED_LOCAL auto scope = computeScope();
            if (!scope || isCrossDocument || !renderer.element())
                return nullAtom();

            Ref element = *renderer.element();
            return makeAtomString("-ua-auto-"_s, String::number(element->nodeIdentifier().toUInt64()));
        },
        [&](const Style::CustomIdent& customIdent) {
            SUPPRESS_UNCHECKED_LOCAL auto scope = computeScope();
            if (!scope)
                return nullAtom();

            return customIdent.value;
        }
    );
}

static ExceptionOr<void> checkDuplicateViewTransitionName(const AtomString& name, OrderedHashSet<AtomString>& usedTransitionNames)
{
    if (usedTransitionNames.contains(name))
        return Exception { ExceptionCode::InvalidStateError, makeString("Multiple elements found with view-transition-name: "_s, name) };
    usedTransitionNames.add(name);
    return { };
}

static Vector<AtomString> effectiveViewTransitionClassList(RenderLayerModelObject& renderer, Element& originatingElement, Style::Scope& documentScope)
{
    return WTF::switchOn(renderer.style().viewTransitionClasses(),
        [](const CSS::Keyword::None&) -> Vector<AtomString> {
            return { };
        },
        [&](const auto& list) -> Vector<AtomString> {
            auto scope = Style::Scope::forOrdinal(originatingElement, list[0].scopeOrdinal);
            if (!scope || scope != &documentScope)
                return { };

            return WTF::map(list, [&](auto& item) {
                return item.name;
            });
        }
    );
}

LayoutRect ViewTransition::captureOverflowRect(RenderLayerModelObject& renderer)
{
    if (!renderer.hasLayer())
        return { };

    if (renderer.isDocumentElementRenderer())
        return containingBlockRect();

    auto bounds = renderer.layer()->calculateLayerBounds(renderer.layer(), LayoutSize(), { RenderLayer::IncludeFilterOutsets, RenderLayer::ExcludeHiddenDescendants, RenderLayer::IncludeCompositedDescendants, RenderLayer::PreserveAncestorFlags, RenderLayer::ExcludeViewTransitionCapturedDescendants });
    return LayoutRect(encloseRectToDevicePixels(bounds, protect(renderer.document())->deviceScaleFactor()));
}

// The computed local-to-absolute transform, and layer bounds don't include the position
// of a RenderInline. Manually add an extra offset to adjust for it.
static LayoutPoint layerToLayoutOffset(const RenderLayerModelObject& renderer)
{
    if (const auto* renderInline = dynamicDowncast<RenderInline>(renderer)) {
        auto boundingBox = renderInline->borderBoxRectInContainer();
        return LayoutPoint { boundingBox.x(), boundingBox.y() };
    }
    return { };
}

static RefPtr<ImageBuffer> snapshotElementVisualOverflowClippedToViewport(LocalFrame& frame, RenderLayerModelObject& renderer, const LayoutRect& snapshotRect, const LayoutSize& subpixelOffset = { })
{
    ASSERT(renderer.hasLayer());
    CheckedRef layerRenderer = renderer;

    IntRect paintRect = enclosingIntRect(snapshotRect);

    if (layerRenderer->isDocumentElementRenderer()) {
        CheckedRef view = layerRenderer->view();
        layerRenderer = view.get();

        auto scrollPosition = protect(view->frameView())->scrollPosition();
        paintRect.moveBy(scrollPosition);
    }

    ASSERT(frame.page());
    float scaleFactor = frame.page()->deviceScaleFactor();

    ASSERT(frame.document());
    RefPtr frameView = frame.document()->view();
    if (!frameView)
        return nullptr;

    auto hostWindow = frameView->root() ? protect(frameView->root())->hostWindow() : nullptr;
    auto colorSpace = screenColorSpace(frameView);
#if PLATFORM(IOS_FAMILY)
    colorSpace = ColorSpace::SRGB(); // FIXME: We should use the screen colorspace on iOS too, but that has blending issues: webkit.org/b/318764.
#endif

    auto buffer = ImageBuffer::create(paintRect.size(), RenderingMode::Accelerated, RenderingPurpose::Snapshot, scaleFactor, colorSpace, PixelFormat::BGRA8, hostWindow);
    if (!buffer)
        return nullptr;

    buffer->context().translate(-paintRect.location());

    auto oldPaintBehavior = frameView->paintBehavior();
    frameView->setPaintBehavior(oldPaintBehavior | PaintBehavior::FlattenCompositingLayers | PaintBehavior::Snapshotting);

    auto paintFlags = RenderLayer::paintLayerPaintingCompositingAllPhasesFlags();
    paintFlags.add(RenderLayer::PaintLayerFlag::TemporaryClipRects);
    paintFlags.add(RenderLayer::PaintLayerFlag::AppliedTransform);
    paintFlags.add(RenderLayer::PaintLayerFlag::PaintingSkipDescendantViewTransition);
    layerRenderer->layer()->paint(buffer->context(), paintRect, subpixelOffset, frameView->paintBehavior(), nullptr, paintFlags);

    frameView->setPaintBehavior(oldPaintBehavior);
    return buffer;
}

// This only iterates through elements with a RenderLayer, which is sufficient for View Transitions which force their creation.
static ExceptionOr<void> forEachRendererInPaintOrder(NOESCAPE const std::function<ExceptionOr<void>(RenderLayerModelObject&)>& function, RenderLayer& layer)
{
    auto result = function(layer.renderer());
    if (result.hasException())
        return result.releaseException();

    if (auto* renderBox = dynamicDowncast<RenderBox>(layer.renderer()); renderBox && isSkippedContentRoot(*renderBox))
        return { };

    layer.updateLayerListsIfNeeded();

#if ASSERT_ENABLED
    LayerListMutationDetector mutationChecker(layer);
#endif

    for (CheckedPtr child : layer.negativeZOrderLayers()) {
        auto result = forEachRendererInPaintOrder(function, *child);
        if (result.hasException())
            return result.releaseException();
    }

    for (CheckedPtr child : layer.normalFlowLayers()) {
        auto result = forEachRendererInPaintOrder(function, *child);
        if (result.hasException())
            return result.releaseException();
    }

    for (CheckedPtr child : layer.positiveZOrderLayers()) {
        auto result = forEachRendererInPaintOrder(function, *child);
        if (result.hasException())
            return result.releaseException();
    }
    return { };
};

static bool rendererIsFragmented(const RenderLayerModelObject& renderer)
{
    // https://drafts.csswg.org/css-view-transitions-1/#capture-old-state-algorithm
    // View transitions explicitly excludes splitting of inline boxes across lines.

    CheckedPtr box = dynamicDowncast<RenderBox>(renderer);
    if (!box)
        return false;

    CheckedPtr enclosingFragmentedFlow = renderer.enclosingFragmentedFlow();
    if (!enclosingFragmentedFlow)
        return false;

    return enclosingFragmentedFlow->boxIsFragmented(*box);
}

// https://drafts.csswg.org/css-view-transitions/#capture-old-state-algorithm
ExceptionOr<void> ViewTransition::captureOldState()
{
    if (!document())
        return { };
    OrderedHashSet<AtomString> usedTransitionNames;
    Vector<CheckedRef<RenderLayerModelObject>> captureRenderers;

    // Ensure style & layout are up-to-date.
    protect(document())->updateLayoutIgnorePendingStylesheets();

    if (CheckedPtr view = document()->renderView()) {
        Ref frame = view->frameView().frame();
        m_initialLargeViewportSize = view->sizeForCSSLargeViewportUnits();
        m_initialPageZoom = frame->pageZoomFactor() * frame->frameScaleFactor();

        auto result = forEachRendererInPaintOrder([&](RenderLayerModelObject& renderer) -> ExceptionOr<void> {
            auto styleable = Styleable::fromRenderer(renderer);
            if (!styleable)
                return { };

            if (rendererIsFragmented(renderer))
                return { };

            if (auto name = effectiveViewTransitionName(renderer, protect(styleable->element), document()->styleScope(), isCrossDocument()); !name.isNull()) {
                if (auto check = checkDuplicateViewTransitionName(name, usedTransitionNames); check.hasException())
                    return check.releaseException();

                renderer.setCapturedInViewTransition(true);
                captureRenderers.append(renderer);
            }
            return { };
        }, *view->layer());
        if (result.hasException()) {
            for (auto& renderer : captureRenderers)
                renderer->setCapturedInViewTransition(false);
            return result.releaseException();
        }
    }

    for (auto& renderer : captureRenderers) {
        CapturedElement capture;

        copyElementBaseProperties(renderer.get(), capture.oldState);
        if (RefPtr frame = document()->frame())
            capture.oldImage = snapshotElementVisualOverflowClippedToViewport(*frame, renderer.get(), capture.oldState.overflowRect, capture.oldState.subpixelOffset);

        auto styleable = Styleable::fromRenderer(renderer);
        ASSERT(styleable);
        Ref element = styleable->element;
        capture.classList = effectiveViewTransitionClassList(renderer, element, document()->styleScope());

        auto transitionName = effectiveViewTransitionName(renderer, element, document()->styleScope(), isCrossDocument());
        m_namedElements.add(transitionName, capture);
    }

    for (auto& [name, capturedElement] : m_namedElements.map()) {
        if (capturedElement->oldState.intersectsViewport && capturedElement->oldImage) {
            if (RefPtr oldImage = *capturedElement->oldImage)
                oldImage->flushDrawingContextAsync();
        }
    }

    for (auto& renderer : captureRenderers)
        renderer->setCapturedInViewTransition(false);

    return { };
}

bool ViewTransition::updatePropertiesForGroupPseudo(CapturedElement& capturedElement, const AtomString& name)
{
    RefPtr properties = capturedElement.newState.properties ? capturedElement.newState.properties : capturedElement.oldState.properties;
    if (properties) {
        // group styles rule
        if (!capturedElement.groupStyleProperties) {
            capturedElement.groupStyleProperties = properties;
            protect(document()->styleScope().resolver())->setViewTransitionStyles(CSSSelector::PseudoElement::ViewTransitionGroup, name, *properties);
            return true;
        }
        return protect(*capturedElement.groupStyleProperties)->mergeAndOverrideOnConflict(*properties);
    }
    return false;
}

// https://drafts.csswg.org/css-view-transitions/#capture-new-state-algorithm
ExceptionOr<void> ViewTransition::captureNewState()
{
    if (!document())
        return { };
    OrderedHashSet<AtomString> usedTransitionNames;
    if (CheckedPtr view = document()->renderView()) {
        auto result = forEachRendererInPaintOrder([&](RenderLayerModelObject& renderer) -> ExceptionOr<void> {
            auto styleable = Styleable::fromRenderer(renderer);
            if (!styleable)
                return { };

            if (rendererIsFragmented(renderer))
                return { };

            Ref element = styleable->element;
            if (auto name = effectiveViewTransitionName(renderer, element, document()->styleScope(), isCrossDocument()); !name.isNull()) {
                if (auto check = checkDuplicateViewTransitionName(name, usedTransitionNames); check.hasException())
                    return check.releaseException();

                if (!m_namedElements.contains(name)) {
                    CapturedElement capturedElement;
                    m_namedElements.add(name, capturedElement);
                }
                auto namedElement = m_namedElements.find(name);
                namedElement->classList = effectiveViewTransitionClassList(renderer, element, document()->styleScope());
                namedElement->newElement = *styleable;

                // Do the work on updatePseudoElementStylesRead now
                // to avoid needing an extra iteration later on.
                if (CheckedPtr box = dynamicDowncast<RenderBoxModelObject>(renderer))
                    copyElementBaseProperties(*box, namedElement->newState);
            }
            return { };
        }, *view->layer());
        if (result.hasException())
            return result.releaseException();
    }
    return { };
}

void ViewTransition::setupDynamicStyleSheet(const AtomString& name, const CapturedElement& capturedElement)
{
    Ref resolver = document()->styleScope().resolver();

    // image animation name rule
    if (capturedElement.oldImage) {
        CSSValueListBuilder list;
        list.append(CSSCustomIdentValue::create(CSS::CustomIdent { "-ua-view-transition-fade-out"_s }));
        if (capturedElement.newElement)
            list.append(CSSCustomIdentValue::create(CSS::CustomIdent { "-ua-mix-blend-mode-plus-lighter"_s }));
        Ref valueList = CSSValueList::createCommaSeparated(WTF::move(list));
        Ref props = MutableStyleProperties::create();
        props->setProperty(CSSPropertyAnimationName, WTF::move(valueList));

        resolver->setViewTransitionStyles(CSSSelector::PseudoElement::ViewTransitionOld, name, props);
    }

    if (capturedElement.newElement) {
        CSSValueListBuilder list;
        list.append(CSSCustomIdentValue::create(CSS::CustomIdent { "-ua-view-transition-fade-in"_s }));
        if (capturedElement.oldImage)
            list.append(CSSCustomIdentValue::create(CSS::CustomIdent { "-ua-mix-blend-mode-plus-lighter"_s }));
        Ref valueList = CSSValueList::createCommaSeparated(WTF::move(list));
        Ref props = MutableStyleProperties::create();
        props->setProperty(CSSPropertyAnimationName, WTF::move(valueList));

        resolver->setViewTransitionStyles(CSSSelector::PseudoElement::ViewTransitionNew, name, props);
    }

    if (!capturedElement.oldImage || !capturedElement.newElement)
        return;

    // group animation name rule
    {
        Ref list = CSSValueList::createCommaSeparated(CSSCustomIdentValue::create(CSS::CustomIdent { makeAtomString("-ua-view-transition-group-anim-"_s, name) }));
        Ref props = MutableStyleProperties::create();
        props->setProperty(CSSPropertyAnimationName, WTF::move(list));

        resolver->setViewTransitionStyles(CSSSelector::PseudoElement::ViewTransitionGroup, name, props);
    }

    // image pair isolation rule
    {
        Ref props = MutableStyleProperties::create();
        props->setProperty(CSSPropertyIsolation, CSSKeywordValue::create(CSSValueID::CSSValueIsolate));

        resolver->setViewTransitionStyles(CSSSelector::PseudoElement::ViewTransitionImagePair, name, props);
    }

    if (!capturedElement.oldState.properties)
        return;

    // group keyframes
    static constexpr auto keyframeProperties = WTF::toArray<CSSPropertyID>({
        CSSPropertyWidth,
        CSSPropertyHeight,
        CSSPropertyTransform,
        CSSPropertyBackdropFilter,
    });
    Ref keyframe = StyleRuleKeyframe::create(protect(capturedElement.oldState.properties)->copyProperties(keyframeProperties));
    keyframe->setKeyText("from"_s);

    Ref keyframes = StyleRuleKeyframes::create(AtomString(makeString("-ua-view-transition-group-anim-"_s, name)));
    keyframes->wrapperAppendKeyframe(WTF::move(keyframe));

    // Register through the document scope so the keyframes are re-established if the resolver
    // is recreated (e.g. by a stylesheet mutation) during the transition. The keyframes are
    // discarded when the transition ends and the view transition styles are cleared.
    document()->styleScope().addViewTransitionKeyframes(WTF::move(keyframes));
}

// https://drafts.csswg.org/css-view-transitions/#setup-transition-pseudo-elements
void ViewTransition::setupTransitionPseudoElements()
{
    document()->setHasViewTransitionPseudoElementTree(true);

    for (auto& [name, capturedElement] : m_namedElements.map())
        setupDynamicStyleSheet(name, capturedElement);
}

ExceptionOr<void> ViewTransition::checkForViewportSizeChange()
{
    CheckedPtr view = document()->renderView();
    if (!view)
        return Exception { ExceptionCode::InvalidStateError, "Skipping view transition because viewport size changed."_s };

    Ref frame = view->frameView().frame();
    if (view->sizeForCSSLargeViewportUnits() != m_initialLargeViewportSize || m_initialPageZoom != (frame->pageZoomFactor() * frame->frameScaleFactor()))
        return Exception { ExceptionCode::InvalidStateError, "Skipping view transition because viewport size changed."_s };
    return { };
}

// https://drafts.csswg.org/css-view-transitions/#activate-view-transition
void ViewTransition::activateViewTransition()
{
    if (m_phase == ViewTransitionPhase::Done)
        return;

    protect(document())->clearRenderingIsSuppressedForViewTransition();

    // Ensure style & layout are up-to-date.
    protect(document())->updateLayoutIgnorePendingStylesheets();

    auto checkSize = checkForViewportSizeChange();
    if (checkSize.hasException()) {
        skipViewTransition(checkSize.releaseException());
        return;
    }

    auto checkFailure = captureNewState();
    if (checkFailure.hasException()) {
        skipViewTransition(checkFailure.releaseException());
        return;
    }

    setupTransitionPseudoElements();

    for (auto& [name, capturedElement] : m_namedElements.map()) {
        if (auto newStyleable = capturedElement->newElement.styleable())
            newStyleable->setCapturedInViewTransition(name);
    }

    if (RefPtr documentElement = document()->documentElement())
        documentElement->invalidateStyle();

    m_phase = ViewTransitionPhase::Animating;

    // Don't read pseudo-element styles, since that happened
    // as part of captureNewState.
    updatePseudoElementStylesWrite();
    updatePseudoElementRenderers();

    protect(m_ready.second)->resolve();
}

// https://drafts.csswg.org/css-view-transitions/#handle-transition-frame-algorithm
void ViewTransition::handleTransitionFrame()
{
    if (!document())
        return;

    RefPtr documentElement = document()->documentElement();
    if (!documentElement)
        return;

    auto checkForActiveAnimations = [&](const Style::PseudoElementIdentifier& pseudoElementIdentifier) {
        if (!documentElement->animations(pseudoElementIdentifier))
            return false;

        Ref timeline = protect(document())->timeline();
        for (auto& animation : *documentElement->animations(pseudoElementIdentifier)) {
            auto playState = animation->playState();
            if (playState == WebAnimation::PlayState::Paused || playState == WebAnimation::PlayState::Running)
                return true;
            if (timeline->hasPendingAnimationEventForAnimation(animation))
                return true;
        }
        return false;
    };

    bool hasActiveAnimations = checkForActiveAnimations({ PseudoElementType::ViewTransition });

    for (auto& name : namedElements().keys()) {
        if (hasActiveAnimations)
            break;
        hasActiveAnimations = checkForActiveAnimations({ PseudoElementType::ViewTransitionGroup, name })
            || checkForActiveAnimations({ PseudoElementType::ViewTransitionImagePair, name })
            || checkForActiveAnimations({ PseudoElementType::ViewTransitionNew, name })
            || checkForActiveAnimations({ PseudoElementType::ViewTransitionOld, name });
    }

    if (!hasActiveAnimations) {
        m_phase = ViewTransitionPhase::Done;
        clearViewTransition();
        protect(m_finished.second)->resolve();
        return;
    }

    auto checkSize = checkForViewportSizeChange();
    if (checkSize.hasException()) {
        skipViewTransition(checkSize.releaseException());
        return;
    }

    auto checkPseudoStyles = updatePseudoElementStylesRead();
    if (checkPseudoStyles.hasException()) {
        skipViewTransition(checkPseudoStyles.releaseException());
        return;
    }

    updatePseudoElementStylesWrite();
    updatePseudoElementRenderers();
}

// https://drafts.csswg.org/css-view-transitions/#clear-view-transition-algorithm
void ViewTransition::clearViewTransition()
{
    if (!document())
        return;

    Ref document = *this->document();
    ASSERT(document->activeViewTransition() == this);

    for (auto& [name, capturedElement] : m_namedElements.map()) {
        if (auto newStyleable = capturedElement->newElement.styleable())
            newStyleable->setCapturedInViewTransition(nullAtom());
    }

    document->setHasViewTransitionPseudoElementTree(false);
    document->styleScope().clearViewTransitionStyles();
    document->setActiveViewTransition(nullptr);

    if (RefPtr documentElement = document->documentElement())
        documentElement->invalidateStyle();
}

// https://drafts.csswg.org/css-view-transitions-1/#snapshot-containing-block
LayoutRect ViewTransition::containingBlockRect()
{
    RefPtr document = this->document();
    if (!document)
        return { };
    RefPtr frameView = document->view();
    if (!frameView)
        return { };
    // FIXME: Bug 285400 - Correctly account for insets.
    return { LayoutPoint { }, frameView->visibleContentRectIncludingScrollbars().size() };
}


// Rounds the x/y translation components to the nearest pixel (for non-perspective)
// transforms, and returns the subpixel offset that was removed.
static LayoutSize snapTransformationTranslationToDevicePixels(TransformationMatrix& matrix, float deviceScaleFactor)
{
    if (matrix.hasPerspective())
        return { };

    LayoutSize oldTranslation(matrix.m41(), matrix.m42());
    matrix.setM41(std::round(matrix.m41() * deviceScaleFactor) / deviceScaleFactor);
    matrix.setM42(std::round(matrix.m42() * deviceScaleFactor) / deviceScaleFactor);
    return oldTranslation - LayoutSize(matrix.m41(), matrix.m42());
}


void ViewTransition::copyElementBaseProperties(RenderLayerModelObject& renderer, CapturedElement::State& output)
{
    std::optional<const Styleable> styleable = Styleable::fromRenderer(renderer);
    ASSERT(styleable);
    Style::Extractor styleExtractor { &styleable->element, false, styleable->pseudoElementIdentifier };

    static constexpr auto transitionProperties = WTF::toArray<CSSPropertyID>({
        CSSPropertyWritingMode,
        CSSPropertyDirection,
        CSSPropertyTextOrientation,
        CSSPropertyMixBlendMode,
        CSSPropertyBackdropFilter,
#if ENABLE(DARK_MODE_CSS)
        CSSPropertyColorScheme,
#endif
    });

    output.overflowRect = captureOverflowRect(renderer);
    output.properties = styleExtractor.copyProperties(transitionProperties);

    CheckedRef frameView = renderer.view().frameView();

    RefPtr documentElement = renderer.document().documentElement();
    if (!documentElement)
        return;
    CheckedPtr documentElementRenderer = documentElement->renderer();
    if (!documentElementRenderer)
        return;

    if (renderer.isDocumentElementRenderer()) {
        output.size.setWidth(containingBlockRect().width());
        output.size.setHeight(containingBlockRect().height());
        output.isRootElement = true;
    } else if (CheckedPtr renderBox = dynamicDowncast<RenderBoxModelObject>(&renderer)) {
        output.isRootElement = false;
        output.size = renderBox->borderBoundingBox().size();

        auto transformState = renderer.viewTransitionTransform();
        TransformationMatrix transform;
        if (transformState.accumulatedTransform()) {
            transform = *transformState.accumulatedTransform();
            output.subpixelOffset = { };
        } else {
            transform.translate(transformState.accumulatedOffset().width(), transformState.accumulatedOffset().height());
            output.subpixelOffset = snapTransformationTranslationToDevicePixels(transform, protect(renderer.document())->deviceScaleFactor());
        }

        output.layerToLayoutOffset = layerToLayoutOffset(renderer);
        transform.translate(output.layerToLayoutOffset.x(), output.layerToLayoutOffset.y());

        auto offset = -toFloatSize(frameView->visibleContentRect().location());
        transform.translateRight(offset.width(), offset.height());

        auto mapped = transform.mapRect(output.overflowRect);
        output.intersectsViewport = mapped.intersects(frameView->boundsRect());

        // Apply the inverse of what will be added by the default value of 'transform-origin',
        // since the computed transform has already included it.
        transform.translate(output.size.width() / 2, output.size.height() / 2);
        transform.translateRight(-output.size.width() / 2, -output.size.height() / 2);

        // Factor out the zoom from the nearest common ancestor of the captured element and the view transition
        // pseudo tree (the document element), so that it doesn't get applied a second time when rendering the
        // snapshots.
        transform.unzoom(documentElementRenderer->style().usedZoom());

        Ref transformListValue = CSSTransformListValue::create(Style::createCSSValue(CSSValuePool::singleton(), documentElementRenderer->style(), transform));
        protect(output.properties)->setProperty(CSSPropertyTransform, WTF::move(transformListValue));
    }

    // Factor out the zoom from the nearest common ancestor of the captured element and the view transition
    // pseudo tree (the document element), so that it doesn't get applied a second time when rendering the
    // snapshots.
    auto cssSize = Style::unapplyingZoom<LayoutSize>(output.size, documentElementRenderer->style());
    protect(output.properties)->setProperty(CSSPropertyWidth, CSSPrimitiveValue::create(cssSize.width(), CSSUnitType::Px));
    protect(output.properties)->setProperty(CSSPropertyHeight, CSSPrimitiveValue::create(cssSize.height(), CSSUnitType::Px));
}

// https://drafts.csswg.org/css-view-transitions-1/#update-pseudo-element-styles
// Perform all reads required without making any mutations
ExceptionOr<void> ViewTransition::updatePseudoElementStylesRead()
{
    RefPtr document = this->document();
    if (!document)
        return { };

    document->updateLayoutIgnorePendingStylesheets();

    for (auto& [name, capturedElement] : m_namedElements.map()) {
        if (auto newStyleable = capturedElement->newElement.styleable()) {
            CheckedPtr renderer = dynamicDowncast<RenderBoxModelObject>(newStyleable->renderer());
            if (!renderer || rendererIsFragmented(*renderer))
                return Exception { ExceptionCode::InvalidStateError, "One of the transitioned elements is longer renderer or is fragmented."_s };

            copyElementBaseProperties(*renderer, capturedElement->newState);
        }
    }
    return { };
}

// https://drafts.csswg.org/css-view-transitions-1/#update-pseudo-element-styles
// Writes all the new styles using the data from the read pass.
void ViewTransition::updatePseudoElementStylesWrite()
{
    RefPtr document = this->document();
    if (!document)
        return;

    bool changed = false;
    for (auto& [name, capturedElement] : m_namedElements.map())
        changed |= updatePropertiesForGroupPseudo(capturedElement, name);

    if (changed) {
        if (RefPtr documentElement = document->documentElement())
            documentElement->invalidateStyle();
    }
}

ExceptionOr<void> ViewTransition::updatePseudoElementRenderers()
{
    RefPtr document = this->document();
    if (!document)
        return { };

    RefPtr documentElement = document->documentElement();
    if (!documentElement)
        return { };

    document->updateStyleIfNeededIgnoringPendingStylesheets();

    for (auto& [name, capturedElement] : m_namedElements.map()) {
        if (auto newStyleable = capturedElement->newElement.styleable()) {
            // FIXME: Also check fragmented content here.
            CheckedPtr renderer = dynamicDowncast<RenderBoxModelObject>(newStyleable->renderer());
            if (!renderer || renderer->isSkippedContent())
                return Exception { ExceptionCode::InvalidStateError, "One of the transitioned elements has become hidden."_s };

            Styleable styleable(*documentElement, Style::PseudoElementIdentifier { PseudoElementType::ViewTransitionNew, name });
            if (CheckedPtr viewTransitionCapture = dynamicDowncast<RenderViewTransitionCapture>(styleable.renderer())) {
                if (viewTransitionCapture->setCapturedSize(capturedElement->newState.size, capturedElement->newState.overflowRect, capturedElement->newState.layerToLayoutOffset))
                    viewTransitionCapture->setNeedsLayout();

                RefPtr<ImageBuffer> image;
                if (RefPtr frame = document->frame(); !viewTransitionCapture->canUseExistingLayers()) {
                    document->updateLayoutIgnorePendingStylesheets();
                    image = snapshotElementVisualOverflowClippedToViewport(*frame, *renderer, capturedElement->newState.overflowRect);
                } else if (auto* layer = renderer->isDocumentElementRenderer() ? renderer->view().layer() : renderer->layer())
                    layer->setNeedsCompositingGeometryUpdate();
                viewTransitionCapture->setImage(image);
            }
        }
    }

    return { };
}

void ViewTransition::setTypes(Ref<ViewTransitionTypeSet>&& newTypes)
{
    m_types = WTF::move(newTypes);
}

RenderViewTransitionCapture* ViewTransition::viewTransitionNewPseudoForCapturedElement(RenderLayerModelObject& renderer)
{
    auto styleable = Styleable::fromRenderer(renderer);
    if (!styleable)
        return nullptr;
    auto capturedName = styleable->element.viewTransitionCapturedName(styleable->pseudoElementIdentifier);
    if (capturedName.isNull())
        return nullptr;

    Styleable pseudoStyleable(*renderer.document().documentElement(), Style::PseudoElementIdentifier { PseudoElementType::ViewTransitionNew, capturedName });
    return dynamicDowncast<RenderViewTransitionCapture>(pseudoStyleable.renderer());
}

// https://drafts.csswg.org/css-view-transitions/#page-visibility-change-steps
void ViewTransition::visibilityStateChanged()
{
    if (!document())
        return;

    Ref document = *this->document();
    if (document->hidden()) {
        if (document->activeViewTransition() == this)
            skipViewTransition(Exception { ExceptionCode::InvalidStateError, "Skipping view transition because document visibility state has become hidden."_s });
    } else
        ASSERT(!document->activeViewTransition());
}

void ViewTransition::stop()
{
    if (!document())
        return;

    m_phase = ViewTransitionPhase::Done;

    Ref document = *this->document();
    document->unregisterForVisibilityStateChangedCallbacks(*this);

    if (document->activeViewTransition() == this)
        clearViewTransition();
}

Document* ViewTransition::document() const
{
    return downcast<Document>(scriptExecutionContext());
}

bool ViewTransition::documentElementIsCaptured() const
{
    auto* document = this->document();
    if (!document)
        return false;

    auto* documentElement = document->documentElement();
    if (!documentElement)
        return false;

    auto* renderer = documentElement->renderer();
    if (!renderer)
        return false;

    return renderer->capturedInViewTransition();
}

UniqueRef<ViewTransitionParams> ViewTransition::takeViewTransitionParams()
{
    auto params = makeUniqueRef<ViewTransitionParams>();
    params->namedElements.swap(m_namedElements);
    params->initialLargeViewportSize = m_initialLargeViewportSize;
    params->initialPageZoom = m_initialPageZoom;

    return params;
}

TextStream& operator<<(TextStream& ts, ViewTransitionPhase phase)
{
    switch (phase) {
    case ViewTransitionPhase::PendingCapture: ts << "PendingCapture"_s; break;
    case ViewTransitionPhase::CapturingOldState: ts << "CapturingOldState"_s; break;
    case ViewTransitionPhase::UpdateCallbackCalled: ts << "UpdateCallbackCalled"_s; break;
    case ViewTransitionPhase::Animating: ts << "Animating"_s; break;
    case ViewTransitionPhase::Done: ts << "Done"_s; break;
    }
    return ts;
}

}
```

## Source/WebCore/css/CSSToLengthConversionData.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSToLengthConversionData.cpp). Source bytes: 3201; source lines: 73; SHA-256 `3a9bf2b59d95f1f807d0fb117a691dee15f99ce0f53c193ed3bf2301ee8a4720`.

```cpp
/*
 * Copyright (C) 2013 Google Inc. All rights reserved.
 * Copyright (C) 2026 Samuel Weinig <sam@webkit.org>
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
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
#include "CSSToLengthConversionData.h"

#include "DocumentView.h"
#include "RenderView.h"
#include "StyleBuilderState.h"

namespace WebCore {

CSSToLengthConversionData::CSSToLengthConversionData(const CSSToLengthConversionData&) = default;
CSSToLengthConversionData::CSSToLengthConversionData(CSSToLengthConversionData&&) = default;

// FIXME: Only rely on the RenderView for style resolution if we have an active LocalFrameView.
static RenderView* NODELETE renderViewForDocument(const Document& document)
{
    if (document.view()) [[likely]]
        return document.renderView();
    return nullptr;
}

CSSToLengthConversionData::CSSToLengthConversionData(const Style::ComputedStyle& style, Style::BuilderState& builderState)
    : m_style(style)
    , m_rootStyle(builderState.rootElementStyle())
    , m_parentStyle(&builderState.parentStyle())
    , m_renderView(renderViewForDocument(builderState.document()))
    , m_elementForContainerUnitResolution(builderState.element())
    , m_styleBuilderState(&builderState)
{
}

CSSToLengthConversionData::CSSToLengthConversionData(const Style::ComputedStyle& style, const Style::ComputedStyle* rootStyle, const Style::ComputedStyle* parentStyle, const RenderView* renderView, const Element* elementForContainerUnitResolution)
    : m_style(style)
    , m_rootStyle(rootStyle)
    , m_parentStyle(parentStyle)
    , m_renderView(renderView)
    , m_elementForContainerUnitResolution(elementForContainerUnitResolution)
{
}

CSSToLengthConversionData::~CSSToLengthConversionData() = default;

} // namespace WebCore
```

## Source/WebCore/style/values/primitives/StyleLengthResolution.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/values/primitives/StyleLengthResolution.cpp). Source bytes: 25738; source lines: 739; SHA-256 `144ec800841b5cc4709d07355c8a726e9aab543c366f9cd07d19a225a672ef97`.

```cpp
/*
 * Copyright (C) 2025-2026 Samuel Weinig <sam@webkit.org>
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
#include "StyleLengthResolution.h"

#include "BoxSides.h"
#include "CSSPrimitiveNumericUnits.h"
#include "CSSToLengthConversionData.h"
#include "ContainerNodeInlines.h"
#include "ContainerQueryEvaluator.h"
#include "Document.h"
#include "Element.h"
#include "FontCascadeInlines.h"
#include "FontCascadeDescription.h"
#include "FontMetrics.h"
#include "NodeRenderStyle.h"
#include "RenderBox.h"
#include "RenderBoxInlines.h"
#include "RenderView.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "StyleLineHeight.h"
#include "StylePrimitiveNumericTypes+Conversions.h"
#include "StylePrimitiveNumericTypes+Evaluation.h"

namespace WebCore {
namespace Style {

// MARK: - Font Metric Zoom

// Raw font metrics include the usedZoomFactor and must be normalized.
static double unzoomFontMetric(double metric, const FontDescription& fontDescription)
{
    if (auto usedZoomFactor = fontDescription.usedZoomFactor(); usedZoomFactor > 0)
        return metric / usedZoomFactor;
    return metric;
}

// MARK: - "font dependent" and "root font dependent" resolution functions

// Resolve the "em", "rem" and "quirky-em" units.
// https://drafts.csswg.org/css-values-4/#em
// https://drafts.csswg.org/css-values-4/#rem
static double resolveEm(const FontCascade& fontCascadeForUnit)
{
    auto& fontDescription = fontCascadeForUnit.fontDescription();
    return fontDescription.computedSize();
}

// Resolve the "ex" and "rex" units.
// https://drafts.csswg.org/css-values-4/#ex
// https://drafts.csswg.org/css-values-4/#rex
static double resolveEx(const FontCascade& fontCascadeForUnit)
{
    auto& fontDescription = fontCascadeForUnit.fontDescription();
    auto& fontMetrics = fontCascadeForUnit.metricsOfPrimaryFont();
    if (fontMetrics.xHeight())
        return unzoomFontMetric(fontMetrics.xHeight().value(), fontDescription);

    return fontDescription.computedSize() / 2.0;
}

// Resolve the "cap" and "rcap" units.
// https://drafts.csswg.org/css-values-4/#cap
// https://drafts.csswg.org/css-values-4/#rcap
static double resolveCap(const FontCascade& fontCascadeForUnit)
{
    auto& fontDescription = fontCascadeForUnit.fontDescription();
    auto& fontMetrics = fontCascadeForUnit.metricsOfPrimaryFont();
    if (fontMetrics.capHeight())
        return unzoomFontMetric(fontMetrics.capHeight().value(), fontDescription);
    return unzoomFontMetric(fontMetrics.intAscent(), fontDescription);
}

// Resolve the "ch" and "rch" units.
// https://drafts.csswg.org/css-values-4/#ch
// https://drafts.csswg.org/css-values-4/#rch
static double resolveCh(const FontCascade& fontCascadeForUnit)
{
    return unzoomFontMetric(fontCascadeForUnit.zeroWidth(), fontCascadeForUnit.fontDescription());
}

// Resolve the "ic" and "ric" units.
// https://drafts.csswg.org/css-values-4/#ic
// https://drafts.csswg.org/css-values-4/#ric
static double resolveIc(const FontCascade& fontCascadeForUnit)
{
    auto& fontDescription = fontCascadeForUnit.fontDescription();
    auto ideogramWidth = fontCascadeForUnit.metricsOfPrimaryFont().ideogramWidth();
    if (!ideogramWidth)
        return fontDescription.unzoomedUsedSize();
    return unzoomFontMetric(ideogramWidth.value(), fontDescription);
}

// Resolve the "lh" and "rlh" units.
// https://drafts.csswg.org/css-values-4/#lh
// https://drafts.csswg.org/css-values-4/#rlh
static double resolveLh(const ComputedStyle* style, const FontCascade& fallbackFontCascadeForUnit)
{
    if (style) {
        auto& fontCascade = style->fontCascade();
        auto& fontDescription = fontCascade.fontDescription();

        return evaluate<float>(
            style->lineHeight(),
            LineHeightEvaluationContext {
                fontDescription.computedSize(),
                static_cast<float>(unzoomFontMetric(fontCascade.metricsOfPrimaryFont().lineSpacing(), fontDescription)),
            },
            ZoomFactor::none()
        );
    }

    return unzoomFontMetric(fallbackFontCascadeForUnit.metricsOfPrimaryFont().lineSpacing(), fallbackFontCascadeForUnit.fontDescription());
}

// MARK: - "viewport-percentage" resolution functions

enum class ViewportType : uint8_t {
    Default,
    Small,
    Large,
    Dynamic,
};

enum class ViewportDimension : uint8_t {
    Height,
    Width,
    Max,
    Min,
    Inline,
    Block,
};

template<LogicalBoxAxis axis>
static double NODELETE resolveViewportLogicalAxis(const FloatSize& size, const ComputedStyle& style)
{
    switch (mapAxisLogicalToPhysical(style.writingMode(), axis)) {
    case BoxAxis::Horizontal:
        return size.width();
    case BoxAxis::Vertical:
        return size.height();
    }

    RELEASE_ASSERT_NOT_REACHED();
}

template<LogicalBoxAxis axis>
static double resolveViewportLogicalAxis(const FloatSize& size, const RenderView& renderView, const auto& adaptor)
{
    if (auto* style = adaptor.styleForViewportUnits())
        return resolveViewportLogicalAxis<axis>(size, *style);

    auto* rootElement = renderView.document().documentElement();
    if (!rootElement)
        return 0;
    auto* rootStyle = rootElement->renderStyle();
    if (!rootStyle)
        return 0;
    return resolveViewportLogicalAxis<axis>(size, *rootStyle);
}

template<ViewportDimension axis>
static double resolveViewportAxis(const FloatSize& size, const RenderView& renderView, const auto& adaptor)
{
    if constexpr (axis == ViewportDimension::Height)
        return size.height();
    else if constexpr (axis == ViewportDimension::Width)
        return size.width();
    else if constexpr (axis == ViewportDimension::Max)
        return size.maxDimension();
    else if constexpr (axis == ViewportDimension::Min)
        return size.minDimension();
    else if constexpr (axis == ViewportDimension::Inline)
        return resolveViewportLogicalAxis<LogicalBoxAxis::Inline>(size, renderView, adaptor);
    else if constexpr (axis == ViewportDimension::Block)
        return resolveViewportLogicalAxis<LogicalBoxAxis::Block>(size, renderView, adaptor);
}

template<ViewportType type>
static FloatSize resolveViewportType(const RenderView& renderView)
{
    if constexpr (type == ViewportType::Default)
        return renderView.sizeForCSSDefaultViewportUnits();
    else if constexpr (type == ViewportType::Small)
        return renderView.sizeForCSSSmallViewportUnits();
    else if constexpr (type == ViewportType::Large)
        return renderView.sizeForCSSLargeViewportUnits();
    else if constexpr (type == ViewportType::Dynamic)
        return renderView.sizeForCSSDynamicViewportUnits();
}


template<ViewportType type, ViewportDimension axis>
static double resolveViewportUnit(const auto& adaptor)
{
    adaptor.setUsesViewportUnits();

    auto* renderView = adaptor.renderViewForViewportUnits();
    if (!renderView) {
        if constexpr (axis == ViewportDimension::Max || axis == ViewportDimension::Min)
            return 1;
        else
            return 0;
    }

    return resolveViewportAxis<axis>(resolveViewportType<type>(*renderView), *renderView, adaptor) / 100.0 / renderView->pageZoomFactor();
}

// MARK: - "container-percentage" resolution functions

static std::optional<double> resolveContainerUnit(CQ::Axis physicalAxis, const auto& adaptor)
{
    ASSERT(physicalAxis == CQ::Axis::Width || physicalAxis == CQ::Axis::Height);

    adaptor.setUsesContainerUnits();

    RefPtr element = adaptor.elementForContainerUnits();
    if (!element)
        return { };

    auto mode = !adaptor.styleForContainerUnits()->pseudoElementType()
        ? ContainerQueryEvaluator::SelectionMode::Element
        : ContainerQueryEvaluator::SelectionMode::PseudoElement;

    // "The query container for each axis is the nearest ancestor container that accepts container size queries on that axis."
    while ((element = ContainerQueryEvaluator::selectContainer(CQ::ContainerRequirements { physicalAxis }, nullString(), *element, mode))) {
        auto* containerRenderer = dynamicDowncast<RenderBox>(element->renderer());
        if (containerRenderer && containerRenderer->hasEligibleContainmentForSizeQuery()) {
            auto widthOrHeight = physicalAxis == CQ::Axis::Width ? containerRenderer->contentBoxWidth() : containerRenderer->contentBoxHeight();
            auto adjustedWidthOrHeight = widthOrHeight.toDouble();

            // FIXME: Document why `font-size` is excluded or remove this special case.
            if (adaptor.property() != CSSPropertyFontSize)
                adjustedWidthOrHeight = adjustedWidthOrHeight / adaptor.renderViewForViewportUnits()->pageZoomFactor();

            return adjustedWidthOrHeight / 100;
        }
        // For pseudo-elements the element itself can be the container. Avoid looping forever.
        mode = ContainerQueryEvaluator::SelectionMode::Element;
    }
    return { };
}

// MARK: - Length Resolution

struct CSSToLengthConversionDataAdaptor {
    const CSSToLengthConversionData& conversionData;

    void setUsesViewportUnits() const
    {
        if (CheckedPtr builderState = conversionData.styleBuilderState())
            builderState->setUsesViewportUnits();
    }

    void setUsesContainerUnits() const
    {
        if (CheckedPtr builderState = conversionData.styleBuilderState())
            builderState->setIsContainerDependent();
    }

    CSSPropertyID property() const
    {
        return conversionData.property();
    }

    bool computingFontProperty() const
    {
        // FIXME: This should be generated but first we need to determine if using `font-*` so literally is the right choice. It might make more sense to have this be all properties that effect font-relative units, so would need to include things like `math-depth`.
        // See https://github.com/w3c/csswg-drafts/issues/14306.
        switch (conversionData.property()) {
        case CSSPropertyFontFeatureSettings:
        case CSSPropertyFontKerning:
        case CSSPropertyFontPalette:
        case CSSPropertyFontSize:
        case CSSPropertyFontSizeAdjust:
        case CSSPropertyFontStyle:
        case CSSPropertyFontSynthesisSmallCaps:
        case CSSPropertyFontSynthesisStyle:
        case CSSPropertyFontSynthesisWeight:
        case CSSPropertyFontVariantAlternates:
        case CSSPropertyFontVariantCaps:
        case CSSPropertyFontVariantEastAsian:
        case CSSPropertyFontVariantEmoji:
        case CSSPropertyFontVariantLigatures:
        case CSSPropertyFontVariantNumeric:
        case CSSPropertyFontVariantPosition:
        case CSSPropertyFontWeight:
        case CSSPropertyFontWidth:
#if ENABLE(VARIATION_FONTS)
        case CSSPropertyFontOpticalSizing:
        case CSSPropertyFontVariationSettings:
#endif
            return true;
        default:
            return false;
        }
    }

    bool computingLineHeightProperty() const
    {
        return conversionData.property() == CSSPropertyLineHeight;
    }

    double applyTextZoom(double value) const
    {
        if (CheckedPtr builderState = conversionData.styleBuilderState())
            return value * builderState->zoomWithTextZoomFactor();
        return value;
    }

    const FontCascade& fontCascadeForFontUnits() const
    {
        if (computingFontProperty()) {
            ASSERT(conversionData.parentStyle());
            return conversionData.parentStyle()->fontCascade();
        }

        return conversionData.style().fontCascade();
    }

    const FontCascade& fontCascadeForRootFontUnits() const
    {
        if (conversionData.rootStyle())
            return conversionData.rootStyle()->fontCascade();

        // FIXME: This should fallback to the initial style, not the element style. To fix this, we will need to make the initial style (Document::initialStyle()) available to CSSToLengthConversionData.
        return fontCascadeForFontUnits();
    }

    const ComputedStyle* styleForLineHeightUnits() const
    {
        if (computingLineHeightProperty() || computingFontProperty())
            return conversionData.parentStyle();

        // FIXME: This should fallback to the initial style, not the element style. To fix this, we will need to make the initial style (Document::initialStyle()) available to CSSToLengthConversionData.
        return &conversionData.style();
    }

    const ComputedStyle* styleForRootLineHeightUnits() const
    {
        return conversionData.rootStyle();
    }

    const ComputedStyle* styleForViewportUnits() const
    {
        return &conversionData.style();
    }

    const RenderView* renderViewForViewportUnits() const
    {
        return conversionData.renderView();
    }

    const ComputedStyle* styleForContainerUnits() const
    {
        return &conversionData.style();
    }

    const Element* elementForContainerUnits() const
    {
        return conversionData.elementForContainerUnitResolution();
    }

    bool isHorizontalForContainerUnits() const
    {
        return conversionData.style().writingMode().isHorizontal();
    }
};

struct DirectDataAdaptor {
    CSSPropertyID propertyToCompute;
    const FontCascade& fontCascadeForUnit;
    const RenderView* renderViewForUnit;

    void setUsesViewportUnits() const
    {
    }

    void setUsesContainerUnits() const
    {
    }

    CSSPropertyID property() const
    {
        return propertyToCompute;
    }

    double applyTextZoom(double value) const
    {
        // FIXME: Add support for text zoom for direct data overload.
        return value;
    }

    const FontCascade& fontCascadeForFontUnits() const
    {
        return fontCascadeForUnit;
    }

    const FontCascade& fontCascadeForRootFontUnits() const
    {
        return fontCascadeForUnit;
    }

    const ComputedStyle* styleForLineHeightUnits() const
    {
        return nullptr;
    }

    const ComputedStyle* styleForRootLineHeightUnits() const
    {
        return nullptr;
    }

    const ComputedStyle* styleForViewportUnits() const
    {
        return nullptr;
    }

    const RenderView* renderViewForViewportUnits() const
    {
        return renderViewForUnit;
    }

    const ComputedStyle* styleForContainerUnits() const
    {
        return nullptr;
    }

    const Element* elementForContainerUnits() const
    {
        return nullptr;
    }

    bool isHorizontalForContainerUnits() const
    {
        return true;
    }
};

static double resolveLengthImpl(double value, CSS::LengthUnit lengthUnit, const auto& adaptor)
{
    using enum CSS::LengthUnit;

    auto applyTextZoomIfComputingLineHeight = [&](double value) {
        return adaptor.property() == CSSPropertyLineHeight ? adaptor.applyTextZoom(value) : value;
    };

    switch (lengthUnit) {
    case Px:
        return value * applyTextZoomIfComputingLineHeight(1.0);
    case Cm:
        return value * applyTextZoomIfComputingLineHeight(CSS::pixelsPerCm);
    case Mm:
        return value * applyTextZoomIfComputingLineHeight(CSS::pixelsPerMm);
    case Q:
        return value * applyTextZoomIfComputingLineHeight(CSS::pixelsPerQ);
    case In:
        return value * applyTextZoomIfComputingLineHeight(CSS::pixelsPerInch);
    case Pt:
        return value * applyTextZoomIfComputingLineHeight(CSS::pixelsPerPt);
    case Pc:
        return value * applyTextZoomIfComputingLineHeight(CSS::pixelsPerPc);

    // MARK: "font dependent" resolution

    case Em:
    case QuirkyEm:
        return value * adaptor.applyTextZoom(resolveEm(adaptor.fontCascadeForFontUnits()));
    case Ex:
        return value * adaptor.applyTextZoom(resolveEx(adaptor.fontCascadeForFontUnits()));
    case Cap:
        return value * adaptor.applyTextZoom(resolveCap(adaptor.fontCascadeForFontUnits()));

    case Ch:
        return value * adaptor.applyTextZoom(resolveCh(adaptor.fontCascadeForFontUnits()));
    case Ic:
        return value * adaptor.applyTextZoom(resolveIc(adaptor.fontCascadeForFontUnits()));

    case Lh:
        return value * applyTextZoomIfComputingLineHeight(resolveLh(adaptor.styleForLineHeightUnits(), adaptor.fontCascadeForFontUnits()));

    // MARK: "root font dependent" resolution

    case Rem:
        return value * adaptor.applyTextZoom(resolveEm(adaptor.fontCascadeForRootFontUnits()));
    case Rcap:
        return value * adaptor.applyTextZoom(resolveCap(adaptor.fontCascadeForRootFontUnits()));
    case Rch:
        return value * adaptor.applyTextZoom(resolveCh(adaptor.fontCascadeForRootFontUnits()));
    case Rex:
        return value * adaptor.applyTextZoom(resolveEx(adaptor.fontCascadeForRootFontUnits()));
    case Ric:
        return value * adaptor.applyTextZoom(resolveIc(adaptor.fontCascadeForRootFontUnits()));

    case Rlh:
        return value * applyTextZoomIfComputingLineHeight(resolveLh(adaptor.styleForRootLineHeightUnits(), adaptor.fontCascadeForRootFontUnits()));

    // MARK: "viewport-percentage" resolution

    case Vh:
        return value * resolveViewportUnit<ViewportType::Default, ViewportDimension::Height>(adaptor);
    case Vw:
        return value * resolveViewportUnit<ViewportType::Default, ViewportDimension::Width>(adaptor);
    case Vmax:
        return value * resolveViewportUnit<ViewportType::Default, ViewportDimension::Max>(adaptor);
    case Vmin:
        return value * resolveViewportUnit<ViewportType::Default, ViewportDimension::Min>(adaptor);
    case Vb:
        return value * resolveViewportUnit<ViewportType::Default, ViewportDimension::Block>(adaptor);
    case Vi:
        return value * resolveViewportUnit<ViewportType::Default, ViewportDimension::Inline>(adaptor);

    case Svh:
        return value * resolveViewportUnit<ViewportType::Small, ViewportDimension::Height>(adaptor);
    case Svw:
        return value * resolveViewportUnit<ViewportType::Small, ViewportDimension::Width>(adaptor);
    case Svmax:
        return value * resolveViewportUnit<ViewportType::Small, ViewportDimension::Max>(adaptor);
    case Svmin:
        return value * resolveViewportUnit<ViewportType::Small, ViewportDimension::Min>(adaptor);
    case Svb:
        return value * resolveViewportUnit<ViewportType::Small, ViewportDimension::Block>(adaptor);
    case Svi:
        return value * resolveViewportUnit<ViewportType::Small, ViewportDimension::Inline>(adaptor);

    case Lvh:
        return value * resolveViewportUnit<ViewportType::Large, ViewportDimension::Height>(adaptor);
    case Lvw:
        return value * resolveViewportUnit<ViewportType::Large, ViewportDimension::Width>(adaptor);
    case Lvmax:
        return value * resolveViewportUnit<ViewportType::Large, ViewportDimension::Max>(adaptor);
    case Lvmin:
        return value * resolveViewportUnit<ViewportType::Large, ViewportDimension::Min>(adaptor);
    case Lvb:
        return value * resolveViewportUnit<ViewportType::Large, ViewportDimension::Block>(adaptor);
    case Lvi:
        return value * resolveViewportUnit<ViewportType::Large, ViewportDimension::Inline>(adaptor);

    case Dvh:
        return value * resolveViewportUnit<ViewportType::Dynamic, ViewportDimension::Height>(adaptor);
    case Dvw:
        return value * resolveViewportUnit<ViewportType::Dynamic, ViewportDimension::Width>(adaptor);
    case Dvmax:
        return value * resolveViewportUnit<ViewportType::Dynamic, ViewportDimension::Max>(adaptor);
    case Dvmin:
        return value * resolveViewportUnit<ViewportType::Dynamic, ViewportDimension::Min>(adaptor);
    case Dvb:
        return value * resolveViewportUnit<ViewportType::Dynamic, ViewportDimension::Block>(adaptor);
    case Dvi:
        return value * resolveViewportUnit<ViewportType::Dynamic, ViewportDimension::Inline>(adaptor);

    // MARK: "container-percentage" resolution

    case Cqw:
        if (auto resolvedValue = resolveContainerUnit(CQ::Axis::Width, adaptor))
            return value * *resolvedValue;
        return resolveLengthImpl(value, Svw, adaptor);

    case Cqh:
        if (auto resolvedValue = resolveContainerUnit(CQ::Axis::Height, adaptor))
            return value * *resolvedValue;
        return resolveLengthImpl(value, Svh, adaptor);

    case Cqi:
        if (auto resolvedValue = resolveContainerUnit(adaptor.isHorizontalForContainerUnits() ? CQ::Axis::Width : CQ::Axis::Height, adaptor))
            return value * *resolvedValue;
        return resolveLengthImpl(value, Svi, adaptor);

    case Cqb:
        if (auto resolvedValue = resolveContainerUnit(adaptor.isHorizontalForContainerUnits() ? CQ::Axis::Height : CQ::Axis::Width, adaptor))
            return value * *resolvedValue;
        return resolveLengthImpl(value, Svb, adaptor);

    case Cqmax:
        if (value < 0)
            return std::min(resolveLengthImpl(value, Cqb, adaptor), resolveLengthImpl(value, Cqi, adaptor));
        return std::max(resolveLengthImpl(value, Cqb, adaptor), resolveLengthImpl(value, Cqi, adaptor));

    case Cqmin:
        if (value < 0)
            return std::max(resolveLengthImpl(value, Cqb, adaptor), resolveLengthImpl(value, Cqi, adaptor));
        return std::min(resolveLengthImpl(value, Cqb, adaptor), resolveLengthImpl(value, Cqi, adaptor));
    }

    RELEASE_ASSERT_NOT_REACHED();
}

double resolveLength(double value, CSS::LengthUnit lengthUnit, const BuilderState& state)
{
    return resolveLengthImpl(value, lengthUnit, CSSToLengthConversionDataAdaptor {
        .conversionData = state.cssToLengthConversionData(),
    });
}

double resolveLength(double value, CSS::LengthUnit lengthUnit, const CSSToLengthConversionData& conversionData)
{
    return resolveLengthImpl(value, lengthUnit, CSSToLengthConversionDataAdaptor {
        .conversionData = conversionData,
    });
}

double resolveLength(double value, CSS::LengthUnit lengthUnit, CSSPropertyID propertyToCompute, const FontCascade& fontCascadeForUnit, const RenderView* renderViewForUnit)
{
    return resolveLengthImpl(value, lengthUnit, DirectDataAdaptor {
        .propertyToCompute = propertyToCompute,
        .fontCascadeForUnit = fontCascadeForUnit,
        .renderViewForUnit = renderViewForUnit,
    });
}

double resolveLength(double value, CSS::LengthUnit lengthUnit, NoConversionDataRequiredToken)
{
    using enum CSS::LengthUnit;

    switch (lengthUnit) {
    case Px:
        return value;
    case Cm:
        return value * CSS::pixelsPerCm;
    case Mm:
        return value * CSS::pixelsPerMm;
    case Q:
        return value * CSS::pixelsPerQ;
    case In:
        return value * CSS::pixelsPerInch;
    case Pt:
        return value * CSS::pixelsPerPt;
    case Pc:
        return value * CSS::pixelsPerPc;

    case Em:
    case QuirkyEm:
    case Ex:
    case Cap:
    case Ch:
    case Ic:
    case Lh:
    case Rem:
    case Rcap:
    case Rch:
    case Rex:
    case Ric:
    case Rlh:
    case Vh:
    case Vw:
    case Vmax:
    case Vmin:
    case Vb:
    case Vi:
    case Svh:
    case Svw:
    case Svmax:
    case Svmin:
    case Svb:
    case Svi:
    case Lvh:
    case Lvw:
    case Lvmax:
    case Lvmin:
    case Lvb:
    case Lvi:
    case Dvh:
    case Dvw:
    case Dvmax:
    case Dvmin:
    case Dvb:
    case Dvi:
    case Cqw:
    case Cqh:
    case Cqi:
    case Cqb:
    case Cqmax:
    case Cqmin:
        ASSERT_NOT_REACHED();
        return -1;
    }

    RELEASE_ASSERT_NOT_REACHED();
}

bool equalForLengthResolution(const ComputedStyle& styleA, const ComputedStyle& styleB)
{
    // These properties affect results of `resolveLength` above.

    if (styleA.fontDescription().usedSize() != styleB.fontDescription().usedSize())
        return false;
    if (styleA.fontDescription().computedSize() != styleB.fontDescription().computedSize())
        return false;
    if (styleA.metricsOfPrimaryFont().xHeight() != styleB.metricsOfPrimaryFont().xHeight())
        return false;
    if (styleA.metricsOfPrimaryFont().zeroWidth() != styleB.metricsOfPrimaryFont().zeroWidth())
        return false;
    if (styleA.metricsOfPrimaryFont().lineSpacing() != styleB.metricsOfPrimaryFont().lineSpacing())
        return false;
    if (styleA.lineHeight() != styleB.lineHeight())
        return false;
    if (styleA.zoom() != styleB.zoom())
        return false;

    return true;
}

// MARK: - em-to-px utility functions

double emToPxDouble(double value, const ComputedStyle& style)
{
    return value * style.fontCascade().fontDescription().computedSize();
}

double emToPxDoubleZoomed(double value, const ComputedStyle& style)
{
    return value * style.fontCascade().fontDescription().usedSize();
}

} // namespace Style
} // namespace WebCore
```
