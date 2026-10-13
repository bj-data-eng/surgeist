# Pinned WebKit contrast-color evidence

Implementation witness, not normative CSS. Full source notices are retained below. No source ingestion is selected. Pin: 73aa6c89e2cb77c46184a81aec944e4ab99d114d

## Source/WebCore/css/values/color/CSSContrastColorResolver.cpp

Source: https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/values/color/CSSContrastColorResolver.cpp

Original SHA256: 660886c44d4d6b1c6bc6e1dacedf733bf9eb596e0bb9cb123d34c2cffe3830c0

```cpp
/*
 * Copyright (C) 2024 Apple Inc. All rights reserved.
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
#include "CSSContrastColorResolver.h"

#include "ColorLuminance.h"

namespace WebCore {
namespace CSS {

// https://drafts.csswg.org/css-color-5/#contrast-color
WebCore::Color resolve(const ContrastColorResolver& resolver)
{
    // FIXME: Implement support for a non-naive resolution.

    auto luminance = relativeLuminance(resolver.color);

    auto contrastWithWhite = contrastRatio(1.0, luminance);
    auto contrastWithBlack = contrastRatio(0.0, luminance);

    return contrastWithWhite > contrastWithBlack ? WebCore::Color::white : WebCore::Color::black;
}

} // namespace CSS
} // namespace WebCore
```

## Source/WebCore/platform/graphics/ColorLuminance.h

Source: https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/ColorLuminance.h

Original SHA256: 794f5f1a3edc3241422cc73fcb9d523be1ba613a03a1fcbb2829f465cc00201c

```cpp
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

#include "ColorConversion.h"

namespace WebCore {

class Color;

template<typename ColorType> inline double relativeLuminance(const ColorType& color)
{
    // https://en.wikipedia.org/wiki/Relative_luminance

    // FIXME: This can be optimized a bit by observing that in some cases the conversion
    // to XYZA<float, WhitePoint::D65> in its entirety is unnecessary to get just the Y
    // component. For instance, for SRGBA<float>, this could be done as:
    //
    //     convertColor<LinearSRGBA<float>>(color) * LinearSRGBA<float>::linearToXYZ.row(1)
    //
    // (for a hypothetical row() function on ColorMatrix). We would probably want to implement
    // this in ColorConversion.h as a sibling function to convertColor which can get a channel
    // of a color in another space in this kind of optimal way.

    return convertColor<XYZA<float, WhitePoint::D65>>(color).resolved().y;
}

inline double contrastRatio(double relativeLuminanceA, double relativeLuminanceB)
{
    // Uses the WCAG 2.0 definition of contrast ratio.
    // https://www.w3.org/TR/WCAG20/#contrast-ratiodef
    auto lighterLuminance = relativeLuminanceA;
    auto darkerLuminance = relativeLuminanceB;

    if (lighterLuminance < darkerLuminance)
        std::swap(lighterLuminance, darkerLuminance);

    return (lighterLuminance + 0.05) / (darkerLuminance + 0.05);
}

template<typename ColorTypeA, typename ColorTypeB> inline double contrastRatio(const ColorTypeA& colorA, const ColorTypeB& colorB)
{
    return contrastRatio(relativeLuminance(colorA), relativeLuminance(colorB));
}

double relativeLuminance(const Color&);
double contrastRatio(const Color&, const Color&);

}
```

## Source/WebCore/platform/graphics/ColorLuminance.cpp

Source: https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/ColorLuminance.cpp

Original SHA256: 915b775c2184892eeb4f4d93c86ca601648083e8709a15ab8c1ab6ad76cee722

```cpp
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
#include "ColorLuminance.h"

#include "Color.h"

namespace WebCore {

double relativeLuminance(const Color& color)
{
    return color.callOnUnderlyingType([&] (const auto& underlyingColor) {
        return relativeLuminance(underlyingColor);
    });
}

double contrastRatio(const Color& colorA, const Color& colorB)
{
    return colorA.callOnUnderlyingType([&] (const auto& underlyingColorA) {
        return colorB.callOnUnderlyingType([&] (const auto& underlyingColorB) {
            return contrastRatio(underlyingColorA, underlyingColorB);
        });
    });
}

}
```
