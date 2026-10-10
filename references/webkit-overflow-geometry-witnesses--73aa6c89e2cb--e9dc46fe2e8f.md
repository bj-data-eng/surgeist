# WebKit overflow geometry source witnesses

Source pin: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Read from the existing local checkout on 2026-10-10, without building or executing WebKit. These bounded implementation witnesses support Surgeist #206 source resolution; they do not establish normative CSS consensus or current behavior on every WebKit backend.

All excerpts retain their complete original file-level copyright, license conditions and disclaimers. RenderBox.cpp declares LGPL 2-or-later; TransformationMatrix.cpp and .h declare BSD 2-clause. Added titles, paths, line labels and omission notices are non-normative. No implementation code is incorporated into Surgeist production by this capture. Accompanying attribution/license inventories are maintained by root.

The actual overflow path shown here is layoutOverflowRectForPropagation → applyPaintGeometryTransformToRect → mapRect → internalMapPoint → multVecMatrix. It maps four corners for the general transform and does not divide at w=0. The separate ray-tracing projectPoint/projected-quad helper is not this call path and is not reproduced or claimed as a scroll-overflow algorithm. Margin behavior is context dependent. These witnesses support selecting an explicit Surgeist policy rather than claiming that WebKit supplies one complete horizon/margin contract.

Manifest SHA-256: `e9dc46fe2e8fc9b4b1af41a8121624ff73411c6986699a2668db4f39669c0b67`. Raw and segment hashes are recorded in the owned overflow provenance. Each segment is checked byte-for-byte against the pinned local source.

## `Source/WebCore/rendering/RenderBox.cpp`

Original source: [Source/WebCore/rendering/RenderBox.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderBox.cpp).

Scope: complete margin eligibility, child overflow union, layout overflow union, margin union, transform, and descendant propagation functions. Original complete file-level legal header is retained. Line labels and explicit omissions are added by Surgeist.

Original lines 1–24; surrounding source omitted:

```cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2005 Allan Sandfeld Jensen (kde@carewolf.com)
 * Copyright (C) 2005-2025 Samuel Weinig (sam@webkit.org)
 * Copyright (C) 2005-2025 Apple Inc. All rights reserved.
 * Copyright (C) 2015-2019 Google Inc. All rights reserved.
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

Original lines 2697–2708; surrounding source omitted:

```cpp
static bool NODELETE shouldMarginInlineEndContributeToScrollableOverflow(auto& renderer)
{
    auto isSupportedContent = renderer.isGridItem() || renderer.isFlexItemIncludingDeprecated() || (renderer.isInFlow() && renderer.parent()->isBlockContainer());
    if (!isSupportedContent)
        return false;

    auto& parentStyle = renderer.parent()->style();
    if (parentStyle.overflowX() != Overflow::Visible && parentStyle.overflowX() != Overflow::Clip)
        return true;
    return parentStyle.overflowY() != Overflow::Visible && parentStyle.overflowY() != Overflow::Clip;
}
```

Original lines 4711–4770; surrounding source omitted:

```cpp
void RenderBox::addOverflowWithRendererOffset(const RenderBox& renderer, LayoutSize offsetFromThis, OptionSet<ComputeOverflowOptions> options)
{
    UNUSED_PARAM(options);

    // Never allow flow threads to propagate overflow up to a parent.
    if (renderer.isRenderFragmentedFlow())
        return;

    CheckedPtr fragmentedFlow = enclosingFragmentedFlow();
    if (fragmentedFlow)
        fragmentedFlow->addFragmentsOverflowFromChild(*this, renderer, offsetFromThis);

    // Only propagate layout overflow from the child if the child isn't clipping its overflow.  If it is, then
    // its overflow is internal to it, and we don't care about it. layoutOverflowRectForPropagation takes care of this
    // and just propagates the border box rect instead.
    auto childLayoutOverflowRect = renderer.layoutOverflowRectForPropagation(writingMode());
    childLayoutOverflowRect.move(offsetFromThis);
    addLayoutOverflow(childLayoutOverflowRect);

    if ((hasPotentiallyScrollableOverflow() || isRenderView())
        && options.containsAny({ ComputeOverflowOptions::MarginsExtendLayoutOverflow, ComputeOverflowOptions::MarginsExtendContentAreaX, ComputeOverflowOptions::MarginsExtendContentAreaY })) {
        addMarginBoxOverflow(renderer, offsetFromThis, options);
    }

    if (paintContainmentApplies())
        return;

    // Add in visual overflow from the child. Even if the child clips its overflow, it may still
    // have visual overflow of its own set from box shadows or reflections. It is unnecessary to propagate this
    // overflow if we are clipping our own overflow.
    if (hasPotentiallyScrollableOverflow())
        return;

    auto childVisualOverflowRect = std::optional<LayoutRect> { };
    auto computeChildVisualOverflowRect = [&] () {
        childVisualOverflowRect = renderer.visualOverflowRectForPropagation(writingMode());
        childVisualOverflowRect->move(offsetFromThis);
    };
    // If this block is flowed inside a flow thread, make sure its overflow is propagated to the containing fragments.
    if (fragmentedFlow) {
        computeChildVisualOverflowRect();
        fragmentedFlow->addFragmentsVisualOverflow(*this, *childVisualOverflowRect);
    } else {
        // Update our visual overflow in case the child spills out the block, but only if we were going to paint
        // the child block ourselves.
        if (renderer.hasSelfPaintingLayer() && !hasFilter())
            return;
    }
    if (!childVisualOverflowRect)
        computeChildVisualOverflowRect();
    addVisualOverflow(*childVisualOverflowRect);
}

bool RenderBox::hasLayoutOverflow() const
{
    if (!m_overflow)
        return false;

    return !flippedPaddingBoxRect().contains(m_overflow->layoutOverflowRect());
}
```

Original lines 4813–4831; surrounding source omitted:

```cpp
void RenderBox::addLayoutOverflow(const LayoutRect& rect)
{
    auto clientBox = flippedPaddingBoxRect();
    if (clientBox.contains(rect) || rect.isEmpty())
        return;

    // For overflow clip objects, we don't want to propagate overflow into unreachable areas.
    if (hasPotentiallyScrollableOverflow() || isRenderView()) {
        LayoutRect clippedRect = clampToAllowedLayoutOverflow(rect, clientBox);

        // Now re-test with the adjusted rectangle and see if it has become unreachable or fully
        // contained.
        if (clientBox.contains(clippedRect) || clippedRect.isEmpty())
            return;

        ensureOverflow().addLayoutOverflow(clippedRect);
    } else
        ensureOverflow().addLayoutOverflow(rect);
}
```

Original lines 4842–4865; surrounding source omitted:

```cpp
void RenderBox::addMarginBoxOverflow(const RenderBox& renderer, LayoutSize offsetFromThis, OptionSet<ComputeOverflowOptions> options)
{
    auto childMarginRect = renderer.marginBoxRect();
    childMarginRect.move(offsetFromThis);

    auto contentArea = scrollableContentAreaOverflowRect();
    if (contentArea.contains(childMarginRect))
        return; // Quick-check short circuit.

    // Some in-flow boxes (e.g. grid items) only extend the layout overflow edge.
    if (options.contains(ComputeOverflowOptions::MarginsExtendLayoutOverflow)) {
        auto clippedChildMarginRect = clampToAllowedLayoutOverflow(childMarginRect, flippedPaddingBoxRect());
        if (!layoutOverflowRect().contains(clippedChildMarginRect))
            ensureOverflow().addLayoutOverflow(clippedChildMarginRect);
        ASSERT(!options.containsAny({ ComputeOverflowOptions::MarginsExtendContentAreaX, ComputeOverflowOptions::MarginsExtendContentAreaY })); // If you need this to work, remove the return statement.
        return;
    }

    // Some in-flow boxes (e.g. flex items) extend the content overflow edge as well.
    if (options.contains(ComputeOverflowOptions::MarginsExtendContentAreaX) && !contentArea.containsX(childMarginRect))
        ensureOverflow().addContentOverflowX(childMarginRect);
    if (options.contains(ComputeOverflowOptions::MarginsExtendContentAreaY) && !contentArea.containsY(childMarginRect))
        ensureOverflow().addContentOverflowY(childMarginRect);
}
```

Original lines 4939–4972; surrounding source omitted:

```cpp
LayoutRect RenderBox::applyPaintGeometryTransformToRect(LayoutRect rect) const
{
    // If we are relatively positioned or if we have a transform, then we have to convert
    // this rectangle into physical coordinates, apply relative positioning and transforms
    // to it, and then convert it back.
    // It ensures that the overflow rect tracks the paint geometry and not the inflow layout position.

    bool isTransformed = this->isTransformed();
    // While a stickily positioned renderer is also inflow positioned, they stretch the overflow rect with their inflow geometry
    // (as opposed to the paint geometry) because they are not stationary.
    bool paintGeometryAffectsOverflow = isTransformed || (isInFlowPositioned() && !isStickilyPositioned());

    if (!paintGeometryAffectsOverflow)
        return rect;

    flipForWritingMode(rect);

    LayoutSize containerOffset;
    if (isInFlowPositioned())
        containerOffset = offsetForInFlowPosition();

    auto container = this->container();
    if (shouldUseTransformFromContainer(container)) {
        TransformationMatrix transform;
        getTransformFromContainer(containerOffset, transform);
        rect = transform.mapRect(rect);
    } else
        rect.move(offsetForInFlowPosition());

    // Now we need to flip back.
    flipForWritingMode(rect);

    return rect;
}
```

Original lines 5013–5042; surrounding source omitted:

```cpp
LayoutRect RenderBox::layoutOverflowRectForPropagation(const WritingMode parentWritingMode) const
{
    // Only propagate interior layout overflow if we don't completely clip it.
    auto rect = borderBoxRect();
    // As per https://drafts.csswg.org/css-overflow-3/#scrollable, both flex and grid items margins' should contribute to the scrollable overflow area.
    if (shouldMarginInlineEndContributeToScrollableOverflow(*this)) {
        auto marginEnd = std::max(0_lu, this->marginEnd(parentWritingMode));
        parentWritingMode.isHorizontal() ? rect.setWidth(rect.width() + marginEnd) : rect.setHeight(rect.height() + marginEnd);
    }

    if (!shouldApplyLayoutContainment()) {
        if (hasNonVisibleOverflow()) {
            if (style().overflowX() == Overflow::Clip && style().overflowY() == Overflow::Visible) {
                LayoutRect clippedOverflowRect = layoutOverflowRect();
                clippedOverflowRect.setX(rect.x());
                clippedOverflowRect.setWidth(rect.width());
                rect.unite(clippedOverflowRect);
            } else if (style().overflowY() == Overflow::Clip && style().overflowX() == Overflow::Visible) {
                LayoutRect clippedOverflowRect = layoutOverflowRect();
                clippedOverflowRect.setY(rect.y());
                clippedOverflowRect.setHeight(rect.height());
                rect.unite(clippedOverflowRect);
            }
        } else
            rect.unite(layoutOverflowRect());
    }

    rect = applyPaintGeometryTransformToRect(rect);
    return convertRectToParentWritingMode(rect, parentWritingMode);
}
```

## `Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp`

Original source: [Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp).

Scope: complete forward rectangle mapping and its two-dimensional homogeneous mapping helper. Original complete file-level legal header is retained. Line labels and explicit omissions are added by Surgeist.

Original lines 1–26; surrounding source omitted:

```cpp
/*
 * Copyright (C) 2005-2025 Apple Inc. All rights reserved.
 * Copyright (C) 2016-2020 Google Inc. All rights reserved.
 * Copyright (C) 2009 Torch Mobile, Inc.
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

Original lines 940–1008; surrounding source omitted:

```cpp
FloatRect TransformationMatrix::mapRect(const FloatRect& r) const
{
    auto type = this->type();
    if (type == Type::IdentityOrTranslation) {
        FloatRect mappedRect(r);
        mappedRect.move(static_cast<float>(m_matrix[3][0]), static_cast<float>(m_matrix[3][1]));
        return mappedRect;
    }

    float minX = r.x();
    float minY = r.y();
    float maxX = r.maxX();
    float maxY = r.maxY();

    if (type == Type::Affine) {
        double a = m11();
        double b = m12();
        double c = m21();
        double d = m22();

        double minResultX;
        double minResultY;
        double maxResultX;
        double maxResultY;

        if (a > 0) {
            maxResultX = a * maxX;
            minResultX = a * minX;
        } else {
            maxResultX = a * minX;
            minResultX = a * maxX;
        }

        if (b > 0) {
            maxResultY = b * maxX;
            minResultY = b * minX;
        } else {
            maxResultY = b * minX;
            minResultY = b * maxX;
        }

        if (c > 0) {
            maxResultX += c * maxY;
            minResultX += c * minY;
        } else {
            maxResultX += c * minY;
            minResultX += c * maxY;
        }

        if (d > 0) {
            maxResultY += d * maxY;
            minResultY += d * minY;
        } else {
            maxResultY += d * minY;
            minResultY += d * maxY;
        }

        return FloatRect(minResultX + m41(), minResultY + m42(), maxResultX - minResultX, maxResultY - minResultY);
    }

    FloatQuad result;

    result.setP1(internalMapPoint(FloatPoint(minX, minY)));
    result.setP2(internalMapPoint(FloatPoint(maxX, minY)));
    result.setP3(internalMapPoint(FloatPoint(maxX, maxY)));
    result.setP4(internalMapPoint(FloatPoint(minX, maxY)));

    return result.boundingBox();
}
```

Original lines 1723–1732; surrounding source omitted:

```cpp
void TransformationMatrix::multVecMatrix(double x, double y, double& resultX, double& resultY) const
{
    resultX = m_matrix[3][0] + x * m_matrix[0][0] + y * m_matrix[1][0];
    resultY = m_matrix[3][1] + x * m_matrix[0][1] + y * m_matrix[1][1];
    double w = m_matrix[3][3] + x * m_matrix[0][3] + y * m_matrix[1][3];
    if (w != 1 && w != 0) {
        resultX /= w;
        resultY /= w;
    }
}
```

## `Source/WebCore/platform/graphics/transforms/TransformationMatrix.h`

Original source: [Source/WebCore/platform/graphics/transforms/TransformationMatrix.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/TransformationMatrix.h).

Scope: complete internalMapPoint helper connecting forward mapping to multVecMatrix. Original complete file-level legal header is retained. Line labels and explicit omissions are added by Surgeist.

Original lines 1–24; surrounding source omitted:

```cpp
/*
 * Copyright (C) 2005-2016 Apple Inc. All rights reserved.
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

Original lines 456–463; surrounding source omitted:

```cpp
    FloatPoint internalMapPoint(const FloatPoint& sourcePoint) const
    {
        double resultX;
        double resultY;
        multVecMatrix(sourcePoint.x(), sourcePoint.y(), resultX, resultY);
        return FloatPoint(static_cast<float>(resultX), static_cast<float>(resultY));
    }
```
