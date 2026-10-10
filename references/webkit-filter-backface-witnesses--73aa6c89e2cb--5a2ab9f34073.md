# WebKit filter and perspective-backface source witnesses

Source pin: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Retrieved from the existing local checkout on 2026-10-10; no WebKit build or source execution. These are implementation witnesses for Surgeist #356/#563, not normative CSS consensus or a claim about every WebKit platform/backend.

The two short kernel files are reproduced completely. The other files are explicitly bounded excerpts; each retains its complete original file-level legal header. Source paths, line ranges, hashes and immutable upstream links identify each excerpt. Added headings and omission boundaries are non-normative. No implementation code is incorporated into Surgeist production by this source capture.

Attribution: WebKit contributors, including the copyright owners named verbatim in the retained headers. The source headers state their applicable BSD 2-clause, LGPL 2-or-later, or LGPL 2.1-or-later / MPL 1.1 / GPL 2.0 alternatives; each declaration remains attached to its covered excerpt. Root maintains accompanying attribution and license inventories.

Manifest SHA-256: `5a2ab9f3407320e5de363a4fd6e16153c4c7842cf866d80293289a3e9f7a9c73`. Exact source bytes and segment checks are recorded in the ignored effects provenance.

## `Source/WebCore/platform/graphics/ColorMatrix.h`

Original source: [Source/WebCore/platform/graphics/ColorMatrix.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/ColorMatrix.h).

Scope: full source file. Full original file-level copyright and license header is retained; line labels and omission notes are added by Surgeist.

Original lines 1–227:

```cpp
/*
 * Copyright (C) 2020 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1.  Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 * 2.  Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in the
 *     documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE AND ITS CONTRIBUTORS "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL APPLE OR ITS CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <concepts>
#include <math.h>
#include <wtf/MathExtras.h>

namespace WebCore {

template<typename, size_t> struct ColorComponents;

template<size_t ColumnCount, size_t RowCount>
class ColorMatrix {
public:
    explicit constexpr ColorMatrix(std::span<const float, RowCount * ColumnCount> s)
    {
        std::ranges::copy(s, m_matrix.begin());
    }

    template<std::convertible_to<float> ...Ts>
    explicit constexpr ColorMatrix(Ts ...input)
        : m_matrix {{ static_cast<float>(input) ... }}
    {
        static_assert(sizeof...(Ts) == RowCount * ColumnCount);
    }

    template<size_t ToColumnCount, size_t ToRowCount>
    constexpr operator ColorMatrix<ToColumnCount, ToRowCount>() const;

    template<size_t NumberOfComponents>
    constexpr ColorComponents<float, NumberOfComponents> transformedColorComponents(const ColorComponents<float, NumberOfComponents>&) const;

    constexpr float at(size_t row, size_t column) const
    {
        return m_matrix[(row * ColumnCount) + column];
    }

    const std::array<float, RowCount * ColumnCount>& data() const LIFETIME_BOUND { return m_matrix; }

    friend bool operator==(const ColorMatrix&, const ColorMatrix&) = default;

private:
    std::array<float, RowCount * ColumnCount> m_matrix;
};

template <> template <> constexpr ColorMatrix<3, 3>::operator ColorMatrix<5, 4>() const
{
    return ColorMatrix<5, 4> {
        at(0, 0), at(0, 1), at(0, 2), 0, 0,
        at(1, 0), at(1, 1), at(1, 2), 0, 0,
        at(2, 0), at(2, 1), at(2, 2), 0, 0,
        0, 0, 0, 1, 0
    };
}

constexpr ColorMatrix<3, 3> brightnessColorMatrix(float amount)
{
    // Brightness is specified as a component transfer function: https://www.w3.org/TR/filter-effects-1/#brightnessEquivalent
    // which is equivalent to the following matrix.
    amount = std::max(amount, 0.0f);
    return ColorMatrix<3, 3> {
        amount, 0.0f, 0.0f,
        0.0f, amount, 0.0f,
        0.0f, 0.0f, amount,
    };
}

constexpr ColorMatrix<5, 4> contrastColorMatrix(float amount)
{
    // Contrast is specified as a component transfer function: https://www.w3.org/TR/filter-effects-1/#contrastEquivalent
    // which is equivalent to the following matrix.
    amount = std::max(amount, 0.0f);
    float intercept = -0.5f * amount + 0.5f;

    return ColorMatrix<5, 4> {
        amount, 0.0f, 0.0f, 0.0f, intercept,
        0.0f, amount, 0.0f, 0.0f, intercept,
        0.0f, 0.0f, amount, 0.0f, intercept,
        0.0f, 0.0f, 0.0f, 1.0f, 0.0f
    };
}

constexpr ColorMatrix<3, 3> grayscaleColorMatrix(float amount)
{
    // Values from https://www.w3.org/TR/filter-effects-1/#grayscaleEquivalent
    float oneMinusAmount = std::clamp(1.0f - amount, 0.0f, 1.0f);
    return ColorMatrix<3, 3> {
        0.2126f + 0.7874f * oneMinusAmount, 0.7152f - 0.7152f * oneMinusAmount, 0.0722f - 0.0722f * oneMinusAmount,
        0.2126f - 0.2126f * oneMinusAmount, 0.7152f + 0.2848f * oneMinusAmount, 0.0722f - 0.0722f * oneMinusAmount,
        0.2126f - 0.2126f * oneMinusAmount, 0.7152f - 0.7152f * oneMinusAmount, 0.0722f + 0.9278f * oneMinusAmount
    };
}

constexpr ColorMatrix<5, 4> invertColorMatrix(float amount)
{
    // Invert is specified as a component transfer function: https://www.w3.org/TR/filter-effects-1/#invertEquivalent
    // which is equivalent to the following matrix.
    amount = std::clamp(amount, 0.0f, 1.0f);
    float multiplier = 1.0f - amount * 2.0f;
    return ColorMatrix<5, 4> {
        multiplier, 0.0f, 0.0f, 0.0f, amount,
        0.0f, multiplier, 0.0f, 0.0f, amount,
        0.0f, 0.0f, multiplier, 0.0f, amount,
        0.0f, 0.0f, 0.0f, 1.0f, 0.0f
    };
}

constexpr ColorMatrix<5, 4> opacityColorMatrix(float amount)
{
    // Opacity is specified as a component transfer function: https://www.w3.org/TR/filter-effects-1/#opacityEquivalent
    // which is equivalent to the following matrix.
    amount = std::clamp(amount, 0.0f, 1.0f);
    return ColorMatrix<5, 4> {
        1.0f, 0.0f, 0.0f, 0.0f, 0.0f,
        0.0f, 1.0f, 0.0f, 0.0f, 0.0f,
        0.0f, 0.0f, 1.0f, 0.0f, 0.0f,
        0.0f, 0.0f, 0.0f, amount, 0.0f
    };
}

constexpr ColorMatrix<5, 4> luminanceToAlphaColorMatrix()
{
    // https://drafts.csswg.org/filter-effects/#attr-valuedef-type-luminancetoalpha
    return ColorMatrix<5, 4> {
        0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
        0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
        0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
        0.2126f, 0.7152f, 0.0722f, 0.0f, 0.0f
    };
}

constexpr ColorMatrix<3, 3> sepiaColorMatrix(float amount)
{
    // Values from https://www.w3.org/TR/filter-effects-1/#sepiaEquivalent
    float oneMinusAmount = std::clamp(1.0f - amount, 0.0f, 1.0f);
    return ColorMatrix<3, 3> {
        0.393f + 0.607f * oneMinusAmount, 0.769f - 0.769f * oneMinusAmount, 0.189f - 0.189f * oneMinusAmount,
        0.349f - 0.349f * oneMinusAmount, 0.686f + 0.314f * oneMinusAmount, 0.168f - 0.168f * oneMinusAmount,
        0.272f - 0.272f * oneMinusAmount, 0.534f - 0.534f * oneMinusAmount, 0.131f + 0.869f * oneMinusAmount
    };
}

constexpr ColorMatrix<3, 3> saturationColorMatrix(float amount)
{
    // Values from https://www.w3.org/TR/filter-effects-1/#feColorMatrixElement
    return ColorMatrix<3, 3> {
        0.213f + 0.787f * amount,  0.715f - 0.715f * amount, 0.072f - 0.072f * amount,
        0.213f - 0.213f * amount,  0.715f + 0.285f * amount, 0.072f - 0.072f * amount,
        0.213f - 0.213f * amount,  0.715f - 0.715f * amount, 0.072f + 0.928f * amount
    };
}

// NOTE: hueRotateColorMatrix is not constexpr due to use of cos/sin which are not constexpr yet.
inline ColorMatrix<3, 3> hueRotateColorMatrix(float angleInDegrees)
{
    float cosHue = cos(deg2rad(angleInDegrees));
    float sinHue = sin(deg2rad(angleInDegrees));

    // Values from https://www.w3.org/TR/filter-effects-1/#feColorMatrixElement
    return ColorMatrix<3, 3> {
        0.213f + cosHue * 0.787f - sinHue * 0.213f, 0.715f - cosHue * 0.715f - sinHue * 0.715f, 0.072f - cosHue * 0.072f + sinHue * 0.928f,
        0.213f - cosHue * 0.213f + sinHue * 0.143f, 0.715f + cosHue * 0.285f + sinHue * 0.140f, 0.072f - cosHue * 0.072f - sinHue * 0.283f,
        0.213f - cosHue * 0.213f - sinHue * 0.787f, 0.715f - cosHue * 0.715f + sinHue * 0.715f, 0.072f + cosHue * 0.928f + sinHue * 0.072f
    };
}

template<size_t ColumnCount, size_t RowCount>
template<size_t NumberOfComponents>
constexpr auto ColorMatrix<ColumnCount, RowCount>::transformedColorComponents(const ColorComponents<float, NumberOfComponents>& inputVector) const -> ColorComponents<float, NumberOfComponents>
{
    static_assert(ColorComponents<float, NumberOfComponents>::Size >= RowCount);
    
    ColorComponents<float, NumberOfComponents> result;
    for (size_t row = 0; row < RowCount; ++row) {
        if constexpr (ColumnCount <= ColorComponents<float, NumberOfComponents>::Size) {
            for (size_t column = 0; column < ColumnCount; ++column)
                result[row] += at(row, column) * inputVector[column];
        } else if constexpr (ColumnCount > ColorComponents<float, NumberOfComponents>::Size) {
            for (size_t column = 0; column < ColorComponents<float, NumberOfComponents>::Size; ++column)
                result[row] += at(row, column) * inputVector[column];
            for (size_t additionalColumn = ColorComponents<float, NumberOfComponents>::Size; additionalColumn < ColumnCount; ++additionalColumn)
                result[row] += at(row, additionalColumn);
        }
    }
    if constexpr (ColorComponents<float, NumberOfComponents>::Size > RowCount) {
        for (size_t additionalRow = RowCount; additionalRow < ColorComponents<float, NumberOfComponents>::Size; ++additionalRow)
            result[additionalRow] = inputVector[additionalRow];
    }

    return result;
}

template<typename T, typename M> inline constexpr auto applyMatricesToColorComponents(const ColorComponents<T, 4>& components, M matrix) -> ColorComponents<T, 4>
{
    return matrix.transformedColorComponents(components);
}

template<typename T, typename M, typename... Matrices> inline constexpr auto applyMatricesToColorComponents(const ColorComponents<T, 4>& components, M matrix, Matrices... matrices) -> ColorComponents<T, 4>
{
    return applyMatricesToColorComponents(matrix.transformedColorComponents(components), matrices...);
}

} // namespace WebCore
```

## `Source/WebCore/platform/graphics/filters/software/FEDisplacementMapSoftwareApplier.cpp`

Original source: [Source/WebCore/platform/graphics/filters/software/FEDisplacementMapSoftwareApplier.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/filters/software/FEDisplacementMapSoftwareApplier.cpp).

Scope: full source file. Full original file-level copyright and license header is retained; line labels and omission notes are added by Surgeist.

Original lines 1–125:

```cpp
/*
 * Copyright (C) 2004, 2005, 2006, 2007 Nikolas Zimmermann <zimmermann@kde.org>
 * Copyright (C) 2004, 2005 Rob Buis <buis@kde.org>
 * Copyright (C) 2005 Eric Seidel <eric@webkit.org>
 * Copyright (C) 2009 Dirk Schulze <krit@webkit.org>
 * Copyright (C) Research In Motion Limited 2010. All rights reserved.
 * Copyright (C) 2021-2022 Apple Inc. All rights reserved.
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
#include "FEDisplacementMapSoftwareApplier.h"

#include "FEDisplacementMap.h"
#include "Filter.h"
#include "GraphicsContext.h"
#include "ImageBuffer.h"
#include "PixelBuffer.h"
#include <wtf/StdLibExtras.h>
#include <wtf/TZoneMallocInlines.h>

namespace WebCore {

WTF_MAKE_TZONE_ALLOCATED_IMPL(FEDisplacementMapSoftwareApplier);

FEDisplacementMapSoftwareApplier::FEDisplacementMapSoftwareApplier(const FEDisplacementMap& effect)
    : Base(effect)
{
    ASSERT(m_effect->xChannelSelector() != ChannelSelectorType::CHANNEL_UNKNOWN);
    ASSERT(m_effect->yChannelSelector() != ChannelSelectorType::CHANNEL_UNKNOWN);
}

int FEDisplacementMapSoftwareApplier::xChannelIndex() const
{
    return static_cast<int>(m_effect->xChannelSelector()) - 1;
}

int FEDisplacementMapSoftwareApplier::yChannelIndex() const
{
    return static_cast<int>(m_effect->yChannelSelector()) - 1;
}

bool FEDisplacementMapSoftwareApplier::apply(const Filter& filter, std::span<const Ref<FilterImage>> inputs, FilterImage& result) const
{
    Ref input = inputs[0].get();
    Ref input2 = inputs[1].get();

    // §16.4: when in2 is tainted, act as a pass-through of in.
    if (m_effect->in2IsTainted()) {
        RefPtr resultImage = result.imageBuffer();
        RefPtr inputImage = input->imageBuffer();
        if (!resultImage || !inputImage)
            return false;
        FloatRect inputImageRect = input->absoluteImageRectRelativeTo(result);
        resultImage->context().drawImageBuffer(*inputImage, inputImageRect);
        return true;
    }

    RefPtr destinationPixelBuffer = result.pixelBuffer(AlphaPremultiplication::Premultiplied);
    if (!destinationPixelBuffer)
        return false;

    auto effectADrawingRect = result.absoluteImageRectRelativeTo(input);
    auto inputPixelBuffer = input->getPixelBuffer(AlphaPremultiplication::Premultiplied, effectADrawingRect);

    auto effectBDrawingRect = result.absoluteImageRectRelativeTo(input2);
    // The calculations using the pixel values from ‘in2’ are performed using non-premultiplied color values.
    auto displacementPixelBuffer = input2->getPixelBuffer(AlphaPremultiplication::Unpremultiplied, effectBDrawingRect);
    
    if (!inputPixelBuffer || !displacementPixelBuffer)
        return false;

    ASSERT(inputPixelBuffer->bytes().size() == displacementPixelBuffer->bytes().size());

    auto paintSize = result.absoluteImageRect().size();
    auto scale = filter.resolvedSize({ m_effect->scale(), m_effect->scale() });
    auto absoluteScale = filter.scaledByFilterScale(scale);

    float scaleForColorX = absoluteScale.width() / 255.0;
    float scaleForColorY = absoluteScale.height() / 255.0;
    float scaledOffsetX = 0.5 - absoluteScale.width() * 0.5;
    float scaledOffsetY = 0.5 - absoluteScale.height() * 0.5;
    
    int displacementChannelX = xChannelIndex();
    int displacementChannelY = yChannelIndex();

    int rowBytes = paintSize.width() * 4;

    for (int y = 0; y < paintSize.height(); ++y) {
        int lineStartOffset = y * rowBytes;

        for (int x = 0; x < paintSize.width(); ++x) {
            int destinationIndex = lineStartOffset + x * 4;
            
            int srcX = x + static_cast<int>(scaleForColorX * displacementPixelBuffer->item(destinationIndex + displacementChannelX) + scaledOffsetX);
            int srcY = y + static_cast<int>(scaleForColorY * displacementPixelBuffer->item(destinationIndex + displacementChannelY) + scaledOffsetY);

            unsigned& destinationPixel = reinterpretCastSpanStartTo<unsigned>(destinationPixelBuffer->bytes().subspan(destinationIndex));
            if (srcX < 0 || srcX >= paintSize.width() || srcY < 0 || srcY >= paintSize.height()) {
                destinationPixel = 0;
                continue;
            }

            destinationPixel = reinterpretCastSpanStartTo<unsigned>(inputPixelBuffer->bytes().subspan(byteOffsetOfPixel(srcX, srcY, rowBytes)));
        }
    }

    return true;
}

} // namespace WebCore
```

## `Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp`

Original source: [Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp).

Scope: complete isBackFaceVisible function. Full original file-level copyright and license header is retained; line labels and omission notes are added by Surgeist.

Original lines 1–26:

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

Original lines 2129–2155:

```cpp
bool TransformationMatrix::isBackFaceVisible() const
{
    // Back-face visibility is determined by transforming the normal vector (0, 0, 1) and
    // checking the sign of the resulting z component. However, normals cannot be
    // transformed by the original matrix, they require being transformed by the
    // inverse-transpose.
    //
    // Since we know we will be using (0, 0, 1), and we only care about the z-component of
    // the transformed normal, then we only need the m33() element of the
    // inverse-transpose. Therefore we do not need the transpose.
    //
    // Additionally, if we only need the m33() element, we do not need to compute a full
    // inverse. Instead, knowing the inverse of a matrix is adjoint(matrix) / determinant,
    // we can simply compute the m33() of the adjoint (adjugate) matrix, without computing
    // the full adjoint.

    double determinant = WebCore::determinant4x4(m_matrix);

    // If the matrix is not invertible, then we assume its backface is not visible.
    if (!std::isnormal(determinant))
        return false;

    double cofactor33 = determinant3x3(m11(), m12(), m14(), m21(), m22(), m24(), m41(), m42(), m44());
    double zComponentOfTransformedNormal = cofactor33 / determinant;

    return zComponentOfTransformedNormal < 0;
}
```

## `Source/WebCore/rendering/BackgroundPainter.cpp`

Original source: [Source/WebCore/rendering/BackgroundPainter.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/BackgroundPainter.cpp).

Scope: complete calculateFillLayerImageGeometryImpl function. Full original file-level copyright and license header is retained; line labels and omission notes are added by Surgeist.

Original lines 1–24:

```cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2005 Allan Sandfeld Jensen (kde@carewolf.com)
 *           (C) 2005, 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2005-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2010-2013 Google Inc. All rights reserved.
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

Original lines 628–816:

```cpp
template<typename Layer> BackgroundImageGeometry BackgroundPainter::calculateFillLayerImageGeometryImpl(const RenderBoxModelObject& renderer, const RenderLayerModelObject* paintContainer, const Layer& fillLayer, Style::ZoomFactor zoom, const LayoutPoint& paintOffset, const LayoutRect& borderBoxRect, std::optional<FillBox> overrideOrigin)
{
    auto& view = renderer.view();

    LayoutUnit left;
    LayoutUnit top;
    LayoutSize positioningAreaSize;
    // Determine the background positioning area and set destination rect to the background painting area.
    // Destination rect will be adjusted later if the background is non-repeating.
    CheckedPtr enclosingLayer = renderer.enclosingLayer();
    bool isTransformed = renderer.isTransformed() || (enclosingLayer && enclosingLayer->hasTransformedAncestor());
    bool fixedAttachment = fillLayer.attachment() == FillAttachment::FixedBackground && !isTransformed;

    LayoutRect destinationRect(borderBoxRect);
    float deviceScaleFactor = protect(renderer)->document().deviceScaleFactor();
    if (!fixedAttachment) {
        LayoutUnit right;
        LayoutUnit bottom;
        // Scroll and Local.
        auto fillLayerOrigin = overrideOrigin.value_or(fillLayer.origin());
        if (fillLayerOrigin != FillBox::BorderBox) {
            left = renderer.borderLeft();
            right = renderer.borderRight();
            top = renderer.borderTop();
            bottom = renderer.borderBottom();
            if (fillLayerOrigin == FillBox::ContentBox) {
                left += renderer.paddingLeft();
                right += renderer.paddingRight();
                top += renderer.paddingTop();
                bottom += renderer.paddingBottom();
            }
        }

        // The background of the box generated by the root element covers the entire canvas including
        // its margins. Since those were added in already, we have to factor them out when computing
        // the background positioning area.
        if (renderer.isDocumentElementRenderer()) {
            positioningAreaSize = downcast<RenderBox>(renderer).borderBoxSize() - LayoutSize(left + right, top + bottom);
            positioningAreaSize = LayoutSize(snapSizeToDevicePixel(positioningAreaSize, LayoutPoint(), deviceScaleFactor));
            if (renderer.writingMode().isBlockFlipped()) {
                LayoutRect flippedRootBorderBox = downcast<RenderBox>(renderer).borderBoxRectInContainer();
                view.flipForWritingMode(flippedRootBorderBox);
                left += flippedRootBorderBox.x() - borderBoxRect.x();
                top += flippedRootBorderBox.y() - borderBoxRect.y();
            }
            if (protect(view)->frameView().hasExtendedBackgroundRectForPainting()) {
                LayoutRect extendedBackgroundRect = protect(view)->frameView().extendedBackgroundRectForPainting();
                left += (renderer.marginLeft() - extendedBackgroundRect.x());
                top += (renderer.marginTop() - extendedBackgroundRect.y());
            }
        } else {
            positioningAreaSize = borderBoxRect.size() - LayoutSize(left + right, top + bottom);
            positioningAreaSize = LayoutSize(snapRectToDevicePixels(LayoutRect(paintOffset, positioningAreaSize), deviceScaleFactor).size());
        }
    } else {
        LayoutRect viewportRect;
        FloatBoxExtent obscuredContentInsets;
        if (renderer.settings().fixedBackgroundsPaintRelativeToDocument())
            viewportRect = view.unscaledDocumentRect();
        else {
            CheckedRef frameView = view.frameView();
            bool useFixedLayout = frameView->useFixedLayout() && !frameView->fixedLayoutSize().isEmpty();

            if (useFixedLayout) {
                // Use the fixedLayoutSize() when useFixedLayout() because the rendering will scale
                // down the frameView to to fit in the current viewport.
                viewportRect.setSize(frameView->fixedLayoutSize());
            } else
                viewportRect.setSize(frameView->sizeForVisibleContent());

            if (renderer.fixedBackgroundPaintsInLocalCoordinates()) {
                if (!useFixedLayout) {
                    // Shifting location by the content insets is needed for layout tests which expect
                    // layout to be shifted when calling window.internals.setObscuredContentInsets().
                    obscuredContentInsets = frameView->obscuredContentInsets(ScrollView::InsetType::WebCoreOrPlatformInset);
                    viewportRect.setLocation({ -obscuredContentInsets.left(), -obscuredContentInsets.top() });
                }
            } else if (useFixedLayout || frameView->frameScaleFactor() != 1) {
                // scrollPositionForFixedPosition() is adjusted for page scale and it does not include
                // insets so do not add it to the calculation below.
                viewportRect.setLocation(frameView->scrollPositionForFixedPosition());
            } else {
                // documentScrollPositionRelativeToViewOrigin() is already adjusted for content insets
                // so we need to account for that in calculating the phase size
                obscuredContentInsets = frameView->obscuredContentInsets(ScrollView::InsetType::WebCoreOrPlatformInset);
                viewportRect.setLocation(frameView->documentScrollPositionRelativeToViewOrigin());
            }

            left += obscuredContentInsets.left();
            top += obscuredContentInsets.top();
        }

        if (paintContainer)
            viewportRect.moveBy(LayoutPoint(-paintContainer->localToAbsolute(FloatPoint())));

        destinationRect = viewportRect;
        positioningAreaSize = destinationRect.size();
        positioningAreaSize.setWidth(positioningAreaSize.width() - obscuredContentInsets.left());
        positioningAreaSize.setHeight(positioningAreaSize.height() - obscuredContentInsets.top());
        positioningAreaSize = LayoutSize(snapRectToDevicePixels(LayoutRect(destinationRect.location(), positioningAreaSize), deviceScaleFactor).size());
    }

    LayoutSize tileSize = calculateFillTileSize(renderer, fillLayer, zoom, positioningAreaSize);

    auto backgroundRepeatX = fillLayer.repeat().x();
    auto backgroundRepeatY = fillLayer.repeat().y();
    LayoutUnit availableWidth = positioningAreaSize.width() - tileSize.width();
    LayoutUnit availableHeight = positioningAreaSize.height() - tileSize.height();

    LayoutSize spaceSize;
    LayoutSize phase;
    auto computedXPosition = Style::evaluate<LayoutUnit>(fillLayer.positionX(), availableWidth, zoom);
    if (backgroundRepeatX == FillRepeat::Round && positioningAreaSize.width() > 0 && tileSize.width() > 0) {
        int numTiles = std::max(1, roundToInt(positioningAreaSize.width() / tileSize.width()));
        if (!fillLayer.size().specifiedHeight() && backgroundRepeatY != FillRepeat::Round)
            tileSize.setHeight(tileSize.height() * positioningAreaSize.width() / (numTiles * tileSize.width()));

        tileSize.setWidth(positioningAreaSize.width() / numTiles);
        phase.setWidth(tileSize.width() ? tileSize.width() - fmodf((computedXPosition + left), tileSize.width()) : 0);
    }

    auto computedYPosition = Style::evaluate<LayoutUnit>(fillLayer.positionY(), availableHeight, zoom);
    if (backgroundRepeatY == FillRepeat::Round && positioningAreaSize.height() > 0 && tileSize.height() > 0) {
        int numTiles = std::max(1, roundToInt(positioningAreaSize.height() / tileSize.height()));
        if (!fillLayer.size().specifiedWidth() && backgroundRepeatX != FillRepeat::Round)
            tileSize.setWidth(tileSize.width() * positioningAreaSize.height() / (numTiles * tileSize.height()));

        tileSize.setHeight(positioningAreaSize.height() / numTiles);
        phase.setHeight(tileSize.height() ? tileSize.height() - fmodf((computedYPosition + top), tileSize.height()) : 0);
    }

    if (backgroundRepeatX == FillRepeat::Repeat) {
        phase.setWidth(tileSize.width() ? tileSize.width() - fmodf(computedXPosition + left, tileSize.width()) : 0);
        spaceSize.setWidth(0);
    } else if (backgroundRepeatX == FillRepeat::Space && tileSize.width() > 0) {
        if (auto space = getSpace(positioningAreaSize.width(), tileSize.width())) {
            LayoutUnit actualWidth = tileSize.width() + *space;
            computedXPosition = 0;
            spaceSize.setWidth(*space);
            spaceSize.setHeight(0);
            phase.setWidth(actualWidth ? actualWidth - fmodf((computedXPosition + left), actualWidth) : 0);
        } else
            backgroundRepeatX = FillRepeat::NoRepeat;
    }

    if (backgroundRepeatX == FillRepeat::NoRepeat) {
        LayoutUnit xOffset = left + computedXPosition;
        if (xOffset > 0)
            destinationRect.move(xOffset, 0_lu);
        xOffset = std::min<LayoutUnit>(xOffset, 0);
        phase.setWidth(-xOffset);
        destinationRect.setWidth(tileSize.width() + xOffset);
        spaceSize.setWidth(0);
    }

    if (backgroundRepeatY == FillRepeat::Repeat) {
        phase.setHeight(tileSize.height() ? tileSize.height() - fmodf(computedYPosition + top, tileSize.height()) : 0);
        spaceSize.setHeight(0);
    } else if (backgroundRepeatY == FillRepeat::Space && tileSize.height() > 0) {
        if (auto space = getSpace(positioningAreaSize.height(), tileSize.height())) {
            LayoutUnit actualHeight = tileSize.height() + *space;
            computedYPosition = 0;
            spaceSize.setHeight(*space);
            phase.setHeight(actualHeight ? actualHeight - fmodf((computedYPosition + top), actualHeight) : 0);
        } else
            backgroundRepeatY = FillRepeat::NoRepeat;
    }
    if (backgroundRepeatY == FillRepeat::NoRepeat) {
        LayoutUnit yOffset = top + computedYPosition;
        if (yOffset > 0)
            destinationRect.move(0_lu, yOffset);
        yOffset = std::min<LayoutUnit>(yOffset, 0);
        phase.setHeight(-yOffset);
        destinationRect.setHeight(tileSize.height() + yOffset);
        spaceSize.setHeight(0);
    }

    if (fixedAttachment) {
        LayoutPoint attachmentPoint = borderBoxRect.location();
        phase.expand(std::max<LayoutUnit>(attachmentPoint.x() - destinationRect.x(), 0), std::max<LayoutUnit>(attachmentPoint.y() - destinationRect.y(), 0));
    }

    destinationRect.intersect(borderBoxRect);

    auto tileSizeWithoutPixelSnapping = tileSize;
    pixelSnapBackgroundImageGeometryForPainting(destinationRect, tileSize, phase, spaceSize, deviceScaleFactor);

    return BackgroundImageGeometry(destinationRect, tileSizeWithoutPixelSnapping, tileSize, phase, spaceSize, fixedAttachment);
}
```

## `Source/WebCore/rendering/RenderLayer.cpp`

Original source: [Source/WebCore/rendering/RenderLayer.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderLayer.cpp).

Scope: bounded paintLayerContents extracts showing filter context selection, background painting and filter application; intervening content is omitted. Full original file-level copyright and license header is retained; line labels and omission notes are added by Surgeist.

Original lines 1–45:

```cpp
/*
 * Copyright (C) 2006-2024 Apple Inc. All rights reserved.
 * Copyright (C) 2013-2014 Google Inc. All rights reserved.
 * Copyright (C) 2019 Adobe. All rights reserved.
 * Copyright (c) 2020, 2021, 2022, 2026 Igalia S.L.
 *
 * Portions are Copyright (C) 1998 Netscape Communications Corporation.
 *
 * Other contributors:
 *   Robert O'Callahan <roc+@cs.cmu.edu>
 *   David Baron <dbaron@fas.harvard.edu>
 *   Christian Biesinger <cbiesinger@web.de>
 *   Randall Jesup <rjesup@wgate.com>
 *   Roland Mainz <roland.mainz@informatik.med.uni-giessen.de>
 *   Josh Soref <timeless@mac.com>
 *   Boris Zbarsky <bzbarsky@mit.edu>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 *
 * Alternatively, the contents of this file may be used under the terms
 * of either the Mozilla Public License Version 1.1, found at
 * http://www.mozilla.org/MPL/ (the "MPL") or the GNU General Public
 * License Version 2.0, found at http://www.fsf.org/copyleft/gpl.html
 * (the "GPL"), in which case the provisions of the MPL or the GPL are
 * applicable instead of those above.  If you wish to allow use of your
 * version of this file only under the terms of one of those two
 * licenses (the MPL or the GPL) and not to allow others to use your
 * version of this file under the LGPL, indicate your decision by
 * deletingthe provisions above and replace them with the notice and
 * other provisions required by the MPL or the GPL, as the case may be.
 * If you do not delete the provisions above, a recipient may use your
 * version of this file under any of the LGPL, the MPL or the GPL.
 */
```

Original lines 3794–3836:

```cpp
    { // Scope for filter-related state changes.
        ClipRect backgroundRect;

        if (shouldHaveFiltersForPainting(context, paintFlags, paintBehavior)) {
            // When we called collectFragments() last time, paintDirtyRect was reset to represent the filter bounds.
            // Now we need to compute the backgroundRect uncontaminated by filters, in order to clip the filtered result.
            // Note that we also use paintingInfo here, not localPaintingInfo which filters also contaminated.
            LayerFragments layerFragments;
            auto clipRectOptions = isPaintingOverflowContents ? clipRectOptionsForPaintingOverflowContents : clipRectDefaultOptions;
            clipRectOptions.add(ClipRectsOption::OutsideFilter);
            if (localPaintFlags & PaintLayerFlag::TemporaryClipRects)
                clipRectOptions.add(ClipRectsOption::Temporary);
            collectFragments(layerFragments, paintingInfo.rootLayer, paintingInfo.paintDirtyRect, ExcludeCompositedPaginatedLayers, PaintingClipRects, clipRectOptions, offsetFromRoot);
            updatePaintingInfoForFragments(layerFragments, paintingInfo, localPaintFlags, shouldPaintContent, offsetFromRoot);

            // FIXME: Handle more than one fragment.
            backgroundRect = layerFragments.isEmpty() ? ClipRect() : layerFragments[0].backgroundRect();

            if (haveTransparency) {
                // If we have a filter and transparency, we have to eagerly start a transparency layer here, rather than risk a child layer lazily starts one with the wrong context.
                beginTransparencyLayers(context, paintingInfo, paintingInfo.paintDirtyRect);
            }
        }

        LayerPaintingInfo localPaintingInfo(paintingInfo);

        // The outermost <svg> is a replaced element in the CSS box tree, so its filter is set up in
        // CSS box coordinates (see RenderLayerFilters::beginFilterEffect) and none of the SVG user
        // space corrections in this scope apply to it.
        bool filtersInSVGUserSpace = renderer().isSVGLayerAwareRenderer() && !renderer().isRenderSVGRoot();

        // Position the filter buffer and composite its result at the element's
        // nominalSVGLayoutLocation, independent of intermediate force-layers that perturb offsetFromRoot.
        auto svgFilterOffset = columnAwareOffsetFromRoot;
        if (renderer().isSVGLayerAwareRenderer() && shouldHaveFiltersForPainting(context, paintFlags, paintBehavior)) {
            if (auto* svgModel = dynamicDowncast<RenderSVGModelObject>(renderer()))
                svgFilterOffset = toLayoutSize(svgModel->nominalSVGLayoutLocation());
            else if (auto* svgBlock = dynamicDowncast<RenderSVGBlock>(renderer()))
                svgFilterOffset = toLayoutSize(svgBlock->nominalSVGLayoutLocation());
        }

        auto* filterContext = setupFilters(context, localPaintingInfo, localPaintFlags, svgFilterOffset, backgroundRect);
```

Original lines 3865–3882:

```cpp
        else if (hasFailedFilterForSVG) {
            shouldPaintContent = false;
            isPaintingCompositedForeground = false;
        }

        bool shouldPaintOutline = [&]() {
            if (!isSelfPaintingLayer)
                return false;

            if (!shouldPaintContent)
                return false;

            if (isPaintingOverlayScrollbars || isCollectingEventRegion || isCollectingAccessibilityRegion)
                return false;

            // For the current layer, the outline has been painted by the primary GraphicsLayer.
            if (localPaintFlags.contains(PaintLayerFlag::PaintingOverflowContentsRoot))
                return false;
```

Original lines 3928–3950:

```cpp
                paintBackgroundForFragments(layerFragments, currentContext, context, paintingInfo.paintDirtyRect, haveTransparency,
                    localPaintingInfo, paintBehavior, subtreePaintRootForRenderer);
            }
        }

        if (shouldPaintNegativeZIndexChildren) {
            if (m_svgData)
                paintNegativeZOrderChildrenForSVG(currentContext, paintingInfo, localPaintFlags);
            else {
                // Now walk the sorted list of children with negative z-indices.
                paintList(negativeZOrderLayers(), currentContext, paintingInfo, localPaintFlags);
            }
        }

        if (isPaintingCompositedForeground && shouldPaintContent && !isPaintingOverlaySVGSegment)
            paintForegroundForFragments(layerFragments, currentContext, context, paintingInfo.paintDirtyRect, haveTransparency, localPaintingInfo, paintBehavior, subtreePaintRootForRenderer);

        if (isCollectingEventRegion && !isInsideSkippedSubtree)
            collectEventRegionForFragments(layerFragments, currentContext, localPaintingInfo, paintBehavior);

        if (isCollectingAccessibilityRegion)
            collectAccessibilityRegionsForFragments(layerFragments, currentContext, localPaintingInfo, paintBehavior);
```

Original lines 3989–3994:

```cpp
                    context.translate(svgFilterCompensation);
                }
            }
            applyFilters(context, paintingInfo, paintBehavior, backgroundRect);
            // Painting a snapshot might have temporarily overriden the filter painting strategy,
            // make sure it gets reset.
```
