# Browser font width and decoration metric source witness

This reference retains four complete files from immutable [WebKit](https://webkit.org/) and [Chromium/Blink](https://www.chromium.org/) revisions. Source bytes, paths, hashes, and original legal headers are retained unchanged. Sources were acquired on 2026-10-10 from local WebKit git objects and official Chromium Gitiles; they were not compiled or executed. Added provenance and evidence descriptions are non-normative. File-specific attribution is recorded in the local [WebKit notice](../licenses/webkit/NOTICE.md) and [Chromium notice](../licenses/chromium/NOTICE.md); the retained original notices govern each file.

The [current Fonts 4 excerpt](css-fonts-4-width-matching--editor-capture-20261010--72f4c49ea2ec.md) still distinguishes the property summary's `>=100%` wider-first rule from the detailed matching algorithm's `<=100%` narrower-first rule. `FontSelectionAlgorithm::widthDistance` uses a strict `>normalWidthValue()` test for the wider-first branch; the header defines normal width as 100. Thus this preferred WebKit source independently selects the narrower-first branch at exactly 100% and accepts a range containing the requested width with zero distance.

The [current Text Decoration 4 excerpt](css-text-decor-4-position-controls--editor-capture-20261010--8820547faa34.md#line-position) still marks the no-considered-text, different-font-size line-through case as unresolved. WebKit `TextBoxPainter` selects a decorating box's style and font metrics for line-through foreground painting; Blink `TextDecorationInfo` selects the decorating box's used font when that box is available, with a target-font fallback for its separately described cases. Neither file establishes a general painting rule for an empty considered-text fragment or the draft's averaging policy. These are metric-selection observations, not an inferred resolution of the empty-text standards question. The browser thickness formulas and vertical-writing limitations are also not adopted as a replacement for the specific current normative thickness and offset clauses.

| File | Decisive evidence | Exact source SHA-256 |
| --- | --- | --- |
| [FontSelectionAlgorithm.cpp](#fontselectionalgorithm-cpp) | widthDistance uses the wider-first branch only above normal width; a containing width range has zero distance. | `685054c9852cca15446ebd59258f816bd3d57d0bf72f9a662d54ef268524b189` |
| [FontSelectionAlgorithm.h](#fontselectionalgorithm-h) | normalWidthValue is exactly 100. | `da3fbdcb9fceb00dec75091c8ace8278fbb68562744baa9b29ddefd07d1d8829` |
| [TextBoxPainter.cpp](#textboxpainter-cpp) | computedLinethroughCenter and decoratingBoxStyleForInlineBox, then collectDecoratingBoxesForForegroundPainting / paintForegroundDecorations, use decorating-box metrics in a text-box painting path. | `a99ab3ec47f623434671dd91c4d719c6482e51cde4573feeb1c5cd3d17058265` |
| [text_decoration_info.cc](#text-decoration-info-cc) | ResolveDecorationAt obtains decorating-box used font; ComputeLineThroughLineData positions around its ascent. Neither path specifies an empty considered-text fragment. | `85ac6220ca637b806fc81d46fd1b3dbcaecd0b59570adbc3a4f91cdb7cf152d8` |

## Newer WebKit comparison without duplicate bodies

The two font matching files at newer fetched revision `437139e30888c5e1b4c35f889f32e1d0c39d018e` are byte-identical to their preferred-revision bodies below. Each immutable path identity is recorded separately; no duplicate body is retained.

| Newer source identity | Equal retained body | SHA-256 |
| --- | --- | --- |
| [Source/WebCore/platform/graphics/FontSelectionAlgorithm.cpp](https://github.com/WebKit/WebKit/blob/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/platform/graphics/FontSelectionAlgorithm.cpp) | [FontSelectionAlgorithm.cpp](#fontselectionalgorithm-cpp) | `685054c9852cca15446ebd59258f816bd3d57d0bf72f9a662d54ef268524b189` |
| [Source/WebCore/platform/graphics/FontSelectionAlgorithm.h](https://github.com/WebKit/WebKit/blob/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/platform/graphics/FontSelectionAlgorithm.h) | [FontSelectionAlgorithm.h](#fontselectionalgorithm-h) | `da3fbdcb9fceb00dec75091c8ace8278fbb68562744baa9b29ddefd07d1d8829` |

## <a id="fontselectionalgorithm-cpp"></a>FontSelectionAlgorithm.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontSelectionAlgorithm.cpp). Path: `Source/WebCore/platform/graphics/FontSelectionAlgorithm.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 9451; source lines: 203; SHA-256: `685054c9852cca15446ebd59258f816bd3d57d0bf72f9a662d54ef268524b189`.

````cpp
/*
 * Copyright (C) 2017-2026 Apple Inc. All rights reserved.
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
#include "FontSelectionAlgorithm.h"

namespace WebCore {

FontSelectionAlgorithm::FontSelectionAlgorithm(FontSelectionRequest request, Vector<Capabilities>&& capabilities, std::optional<Capabilities> bounds)
    : m_request(request)
    , m_capabilities(WTF::move(capabilities))
{
    ASSERT(!m_capabilities.isEmpty());
    if (bounds)
        m_capabilitiesBounds = bounds.value();
    else {
        for (auto& capabilities : m_capabilities)
            m_capabilitiesBounds.expand(capabilities);
    }
}

auto FontSelectionAlgorithm::widthDistance(Capabilities capabilities) const -> DistanceResult
{
    auto width = capabilities.width;
    ASSERT(width.isValid());
    if (width.includes(m_request.width))
        return { FontSelectionValue(), m_request.width };

    if (m_request.width > normalWidthValue()) {
        if (width.minimum > m_request.width)
            return { width.minimum - m_request.width, width.minimum };
        ASSERT(width.maximum < m_request.width);
        auto threshold = std::max(m_request.width, m_capabilitiesBounds.width.maximum);
        return { threshold - width.maximum, width.maximum };
    }

    if (width.maximum < m_request.width)
        return { m_request.width - width.maximum, width.maximum };
    ASSERT(width.minimum > m_request.width);
    auto threshold = std::min(m_request.width, m_capabilitiesBounds.width.minimum);
    return { width.minimum - threshold, width.minimum };
}

auto FontSelectionAlgorithm::styleDistance(Capabilities capabilities) const -> DistanceResult
{
    auto slope = capabilities.slope;
    auto requestSlope = m_request.slope.value_or(normalItalicValue());
    ASSERT(slope.isValid());

    // Per CSS Fonts 4 §5.2, italic and oblique are not interchangeable when choosing
    // faces. When requesting oblique (slnt axis), italic-labeled faces are only a last
    // resort after both oblique and normal faces. When requesting italic (ital axis),
    // italic-labeled faces are preferred, but oblique faces are acceptable before normal.
    // https://drafts.csswg.org/css-fonts-4/#font-style-matching
    //
    // We implement this by giving mismatched faces a distance penalty large enough to
    // always lose to any matching-category face, while still allowing last-resort use
    // when no better face exists.
    if (m_request.slopeAxis == FontStyleAxis::slnt && capabilities.faceAxis == FontStyleAxis::ital) {
        auto clampedSlope = std::clamp(requestSlope, slope.minimum, slope.maximum);
        return { FontSelectionValue::maximumValue(), clampedSlope };
    }

    if (m_request.penalizeObliqueFontSelection && capabilities.faceAxis == FontStyleAxis::slnt) {
        auto clampedSlope = std::clamp(requestSlope, slope.minimum, slope.maximum);
        return { FontSelectionValue::maximumValue(), clampedSlope };
    }

    if (slope.includes(requestSlope))
        return { FontSelectionValue(), requestSlope };

    if (requestSlope >= italicThreshold()) {
        if (slope.minimum > requestSlope)
            return { slope.minimum - requestSlope, slope.minimum };
        ASSERT(requestSlope > slope.maximum);
        auto threshold = std::max(requestSlope, m_capabilitiesBounds.slope.maximum);
        return { threshold - slope.maximum, slope.maximum };
    }

    if (requestSlope >= FontSelectionValue()) {
        if (slope.maximum >= FontSelectionValue() && slope.maximum < requestSlope)
            return { requestSlope - slope.maximum, slope.maximum };
        if (slope.minimum > requestSlope)
            return { slope.minimum, slope.minimum };
        ASSERT(slope.maximum < FontSelectionValue());
        auto threshold = std::max(requestSlope, m_capabilitiesBounds.slope.maximum);
        return { threshold - slope.maximum, slope.maximum };
    }

    if (requestSlope > -italicThreshold()) {
        if (slope.minimum > requestSlope && slope.minimum <= FontSelectionValue())
            return { slope.minimum - requestSlope, slope.minimum };
        if (slope.maximum < requestSlope)
            return { -slope.maximum, slope.maximum };
        ASSERT(slope.minimum > FontSelectionValue());
        auto threshold = std::min(requestSlope, m_capabilitiesBounds.slope.minimum);
        return { slope.minimum - threshold, slope.minimum };
    }

    if (slope.maximum < requestSlope)
        return { requestSlope - slope.maximum, slope.maximum };
    ASSERT(slope.minimum > requestSlope);
    auto threshold = std::min(requestSlope, m_capabilitiesBounds.slope.minimum);
    return { slope.minimum - threshold, slope.minimum };
}

auto FontSelectionAlgorithm::weightDistance(Capabilities capabilities) const -> DistanceResult
{
    auto weight = capabilities.weight;
    ASSERT(weight.isValid());
    if (weight.includes(m_request.weight))
        return { FontSelectionValue(), m_request.weight };

    if (m_request.weight >= lowerWeightSearchThreshold() && m_request.weight <= upperWeightSearchThreshold()) {
        if (weight.minimum > m_request.weight && weight.minimum <= upperWeightSearchThreshold())
            return { weight.minimum - m_request.weight, weight.minimum };
        if (weight.maximum < m_request.weight)
            return { upperWeightSearchThreshold() - weight.maximum, weight.maximum };
        ASSERT(weight.minimum > upperWeightSearchThreshold());
        auto threshold = std::min(m_request.weight, m_capabilitiesBounds.weight.minimum);
        return { weight.minimum - threshold, weight.minimum };
    }
    if (m_request.weight < lowerWeightSearchThreshold()) {
        if (weight.maximum < m_request.weight)
            return { m_request.weight - weight.maximum, weight.maximum };
        ASSERT(weight.minimum > m_request.weight);
        auto threshold = std::min(m_request.weight, m_capabilitiesBounds.weight.minimum);
        return { weight.minimum - threshold, weight.minimum };
    }
    ASSERT(m_request.weight >= upperWeightSearchThreshold());
    if (weight.minimum > m_request.weight)
        return { weight.minimum - m_request.weight, weight.minimum };
    ASSERT(weight.maximum < m_request.weight);
    auto threshold = std::max(m_request.weight, m_capabilitiesBounds.weight.maximum);
    return { threshold - weight.maximum, weight.maximum };
}

FontSelectionValue FontSelectionAlgorithm::bestValue(std::span<const bool> eliminated, DistanceFunction computeDistance) const
{
    std::optional<DistanceResult> smallestDistance;
    for (size_t i = 0, size = m_capabilities.size(); i < size; ++i) {
        if (eliminated[i])
            continue;
        auto distanceResult = (this->*computeDistance)(m_capabilities[i]);
        if (!smallestDistance || distanceResult.distance < smallestDistance.value().distance)
            smallestDistance = distanceResult;
    }
    ASSERT(smallestDistance);
    return smallestDistance.value().value;
}

void FontSelectionAlgorithm::filterCapability(std::span<bool> eliminated, DistanceFunction computeDistance, CapabilitiesRange inclusionRange)
{
    auto value = bestValue(eliminated, computeDistance);
    for (size_t i = 0, size = m_capabilities.size(); i < size; ++i)
        eliminated[i] = eliminated[i] || !(m_capabilities[i].*inclusionRange).includes(value);
}

const Vector<bool>& FontSelectionAlgorithm::ensureEliminatedCapabilities()
{
    if (!m_eliminatedCapabilities) {
        Vector<bool, 256> eliminated(FillWith { }, m_capabilities.size(), false);
        filterCapability(eliminated.mutableSpan(), &FontSelectionAlgorithm::widthDistance, &Capabilities::width);
        filterCapability(eliminated.mutableSpan(), &FontSelectionAlgorithm::styleDistance, &Capabilities::slope);
        filterCapability(eliminated.mutableSpan(), &FontSelectionAlgorithm::weightDistance, &Capabilities::weight);
        m_eliminatedCapabilities = WTF::move(eliminated);
    }
    return *m_eliminatedCapabilities;
}

const Vector<bool>& FontSelectionAlgorithm::eliminatedCapabilities()
{
    return ensureEliminatedCapabilities();
}

size_t FontSelectionAlgorithm::indexOfBestCapabilities()
{
    return ensureEliminatedCapabilities().find(false);
}

}
````

## <a id="fontselectionalgorithm-h"></a>FontSelectionAlgorithm.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontSelectionAlgorithm.h). Path: `Source/WebCore/platform/graphics/FontSelectionAlgorithm.h`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 12757; source lines: 427; SHA-256: `da3fbdcb9fceb00dec75091c8ace8278fbb68562744baa9b29ddefd07d1d8829`.

````cpp
/*
 * Copyright (C) 2017-2026 Apple Inc. All rights reserved.
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

#include <WebCore/TextFlags.h>
#include <algorithm>
#include <tuple>
#include <wtf/Hasher.h>
#include <wtf/text/TextStream.h>

namespace WebCore {

// Unclamped, unchecked, signed fixed-point number representing a value used for font variations.
// Sixteen bits in total, one sign bit, two fractional bits, smallest positive value is 0.25,
// maximum value is 8191.75, and minimum value is -8192.
class FontSelectionValue {
public:
    using BackingType = int16_t;

    FontSelectionValue() = default;

    // Exposed over IPC
    explicit constexpr FontSelectionValue(BackingType);

    // Explicit because it won't work correctly for values outside the representable range.
    explicit constexpr FontSelectionValue(int);

    // Explicit because it won't work correctly for values outside the representable range and because precision can be lost.
    explicit constexpr FontSelectionValue(float);

    // Precision can be lost, but value will be clamped to the representable range.
    static constexpr FontSelectionValue clampFloat(float);

    // Since floats have 23 mantissa bits, every value can be represented losslessly.
    constexpr operator float() const;

    static constexpr FontSelectionValue maximumValue();
    static constexpr FontSelectionValue minimumValue();

    friend constexpr FontSelectionValue operator+(FontSelectionValue, FontSelectionValue);
    friend constexpr FontSelectionValue operator-(FontSelectionValue, FontSelectionValue);
    friend constexpr FontSelectionValue operator*(FontSelectionValue, FontSelectionValue);
    friend constexpr FontSelectionValue operator/(FontSelectionValue, FontSelectionValue);
    friend constexpr FontSelectionValue operator-(FontSelectionValue);

    friend auto operator<=>(FontSelectionValue, FontSelectionValue) = default;

    constexpr BackingType rawValue() const { return m_backing; }

private:
    enum class RawTag { RawTag };
    constexpr FontSelectionValue(int, RawTag);

    static constexpr int fractionalEntropy = 4;
    BackingType m_backing { 0 };
};

constexpr FontSelectionValue::FontSelectionValue(BackingType x)
    : m_backing(x)
{
}

constexpr FontSelectionValue::FontSelectionValue(int x)
    : m_backing(x * fractionalEntropy)
{
    // FIXME: Should we assert the passed in value was in range?
}

constexpr FontSelectionValue::FontSelectionValue(float x)
    : m_backing(x * fractionalEntropy)
{
    // FIXME: Should we assert the passed in value was in range?
}

constexpr FontSelectionValue::operator float() const
{
    return m_backing / static_cast<float>(fractionalEntropy);
}

constexpr FontSelectionValue FontSelectionValue::maximumValue()
{
    return { std::numeric_limits<BackingType>::max(), RawTag::RawTag };
}

constexpr FontSelectionValue FontSelectionValue::minimumValue()
{
    return { std::numeric_limits<BackingType>::min(), RawTag::RawTag };
}

constexpr FontSelectionValue FontSelectionValue::clampFloat(float value)
{
    return FontSelectionValue { std::max<float>(minimumValue(), std::min<float>(value, maximumValue())) };
}

constexpr FontSelectionValue::FontSelectionValue(int rawValue, RawTag)
    : m_backing(rawValue)
{
}

constexpr FontSelectionValue operator+(FontSelectionValue a, FontSelectionValue b)
{
    return { a.m_backing + b.m_backing, FontSelectionValue::RawTag::RawTag };
}

constexpr FontSelectionValue operator-(FontSelectionValue a, FontSelectionValue b)
{
    return { a.m_backing - b.m_backing, FontSelectionValue::RawTag::RawTag };
}

constexpr FontSelectionValue operator*(FontSelectionValue a, FontSelectionValue b)
{
    return { a.m_backing * b.m_backing / FontSelectionValue::fractionalEntropy, FontSelectionValue::RawTag::RawTag };
}

constexpr FontSelectionValue operator/(FontSelectionValue a, FontSelectionValue b)
{
    return { a.m_backing * FontSelectionValue::fractionalEntropy / b.m_backing, FontSelectionValue::RawTag::RawTag };
}

constexpr FontSelectionValue operator-(FontSelectionValue value)
{
    return { -value.m_backing, FontSelectionValue::RawTag::RawTag };
}

constexpr FontSelectionValue italicThreshold()
{
    return FontSelectionValue { 14 };
}

constexpr bool isItalic(std::optional<FontSelectionValue> slope)
{
    return slope && slope.value() >= italicThreshold();
}

constexpr FontSelectionValue normalItalicValue()
{
    return FontSelectionValue { 0 };
}

constexpr FontSelectionValue italicValue()
{
    return FontSelectionValue { 14 };
}

constexpr FontSelectionValue boldThreshold()
{
    return FontSelectionValue { 600 };
}

constexpr FontSelectionValue boldWeightValue()
{
    return FontSelectionValue { 700 };
}

constexpr FontSelectionValue normalWeightValue()
{
    return FontSelectionValue { 400 };
}

constexpr FontSelectionValue lightWeightValue()
{
    return FontSelectionValue { 200 };
}

constexpr bool isFontWeightBold(FontSelectionValue fontWeight)
{
    return fontWeight >= boldThreshold();
}

constexpr FontSelectionValue lowerWeightSearchThreshold()
{
    return FontSelectionValue { 400 };
}

constexpr FontSelectionValue upperWeightSearchThreshold()
{
    return FontSelectionValue { 500 };
}

constexpr FontSelectionValue ultraCondensedWidthValue()
{
    return FontSelectionValue { 50 };
}

constexpr FontSelectionValue extraCondensedWidthValue()
{
    return FontSelectionValue { 62.5f };
}

constexpr FontSelectionValue condensedWidthValue()
{
    return FontSelectionValue { 75 };
}

constexpr FontSelectionValue semiCondensedWidthValue()
{
    return FontSelectionValue { 87.5f };
}

constexpr FontSelectionValue normalWidthValue()
{
    return FontSelectionValue { 100 };
}

constexpr FontSelectionValue semiExpandedWidthValue()
{
    return FontSelectionValue { 112.5f };
}

constexpr FontSelectionValue expandedWidthValue()
{
    return FontSelectionValue { 125 };
}

constexpr FontSelectionValue extraExpandedWidthValue()
{
    return FontSelectionValue { 150 };
}

constexpr FontSelectionValue ultraExpandedWidthValue()
{
    return FontSelectionValue { 200 };
}

inline void add(Hasher& hasher, const FontSelectionValue& value)
{
    add(hasher, value.rawValue());
}

// [Inclusive, Inclusive]
struct FontSelectionRange {
    using Value = FontSelectionValue;

    constexpr FontSelectionRange(Value a, Value b)
        : minimum(std::min(a, b))
        , maximum(std::max(a, b))
    {
    }

    explicit constexpr FontSelectionRange(Value value)
        : minimum(value)
        , maximum(value)
    {
    }

    friend constexpr bool operator==(const FontSelectionRange&, const FontSelectionRange&) = default;

    constexpr bool isValid() const
    {
        return minimum <= maximum;
    }

    void expand(const FontSelectionRange& other)
    {
        ASSERT(other.isValid());
        if (!isValid())
            *this = other;
        else {
            minimum = std::min(minimum, other.minimum);
            maximum = std::max(maximum, other.maximum);
        }
        ASSERT(isValid());
    }

    constexpr bool includes(Value target) const
    {
        return target >= minimum && target <= maximum;
    }

    Value minimum { 1 };
    Value maximum { 0 };
};

inline void add(Hasher& hasher, const FontSelectionRange& range)
{
    add(hasher, range.minimum, range.maximum);
}

struct FontSelectionRequest {
    using Value = FontSelectionValue;

    Value weight;
    Value width;
    std::optional<Value> slope;
    FontStyleAxis slopeAxis { FontStyleAxis::normal };
    bool penalizeObliqueFontSelection { false };

    friend bool operator==(const FontSelectionRequest&, const FontSelectionRequest&) = default;
};

inline TextStream& operator<<(TextStream& ts, const FontSelectionValue& fontSelectionValue)
{
    ts << TextStream::FormatNumberRespectingIntegers(fontSelectionValue.rawValue());
    return ts;
}

inline TextStream& operator<<(TextStream& ts, const std::optional<FontSelectionValue>& optionalFontSelectionValue)
{
    ts << optionalFontSelectionValue.value_or(normalItalicValue());
    return ts;
}

inline void add(Hasher& hasher, const FontSelectionRequest& request)
{
    add(hasher, request.weight, request.width, request.slope, std::to_underlying(request.slopeAxis), request.penalizeObliqueFontSelection);
}

struct FontSelectionCapabilities {
    using Range = FontSelectionRange;

    friend constexpr bool operator==(const FontSelectionCapabilities&, const FontSelectionCapabilities&) = default;

    void expand(const FontSelectionCapabilities& capabilities)
    {
        weight.expand(capabilities.weight);
        width.expand(capabilities.width);
        slope.expand(capabilities.slope);
    }

    Range weight { normalWeightValue() };
    Range width { normalWidthValue() };
    Range slope { normalItalicValue() };
    FontStyleAxis faceAxis { FontStyleAxis::normal };
};

struct FontSelectionSpecifiedCapabilities {
    using Capabilities = FontSelectionCapabilities;
    using Range = FontSelectionRange;
    using OptionalRange = std::optional<Range>;

    constexpr Capabilities computeFontSelectionCapabilities() const
    {
        return { computeWeight(), computeWidth(), computeSlope(), faceAxis };
    }

    friend constexpr bool operator==(const FontSelectionSpecifiedCapabilities&, const FontSelectionSpecifiedCapabilities&) = default;

    FontSelectionSpecifiedCapabilities& operator=(const Capabilities& other)
    {
        weight = other.weight;
        width = other.width;
        slope = other.slope;
        faceAxis = other.faceAxis;
        return *this;
    }

    constexpr Range computeWeight() const
    {
        return weight.value_or(Range { normalWeightValue() });
    }

    constexpr Range computeWidth() const
    {
        return width.value_or(Range { normalWidthValue() });
    }

    constexpr Range computeSlope() const
    {
        return slope.value_or(Range { normalItalicValue() });
    }

    OptionalRange weight;
    OptionalRange width;
    OptionalRange slope;
    FontStyleAxis faceAxis { FontStyleAxis::normal };
};

inline void add(Hasher& hasher, const FontSelectionSpecifiedCapabilities& capabilities)
{
    add(hasher, capabilities.weight, capabilities.width, capabilities.slope, std::to_underlying(capabilities.faceAxis));
}

class FontSelectionAlgorithm {
public:
    using Capabilities = FontSelectionCapabilities;

    FontSelectionAlgorithm() = delete;
    FontSelectionAlgorithm(FontSelectionRequest, Vector<Capabilities>&&, std::optional<Capabilities> capabilitiesBounds = std::nullopt);

    struct DistanceResult {
        FontSelectionValue distance;
        FontSelectionValue value;
    };
    DistanceResult widthDistance(Capabilities) const;
    DistanceResult styleDistance(Capabilities) const;
    DistanceResult weightDistance(Capabilities) const;

    size_t indexOfBestCapabilities();
    const Vector<bool>& eliminatedCapabilities();

private:
    using DistanceFunction = DistanceResult (FontSelectionAlgorithm::*)(Capabilities) const;
    using CapabilitiesRange = FontSelectionRange Capabilities::*;
    FontSelectionValue bestValue(std::span<const bool> eliminated, DistanceFunction) const;
    void filterCapability(std::span<bool> eliminated, DistanceFunction, CapabilitiesRange);
    const Vector<bool>& ensureEliminatedCapabilities();

    FontSelectionRequest m_request;
    Capabilities m_capabilitiesBounds;
    const Vector<Capabilities> m_capabilities;
    std::optional<Vector<bool>> m_eliminatedCapabilities;
};

}
````

## <a id="textboxpainter-cpp"></a>TextBoxPainter.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/TextBoxPainter.cpp). Path: `Source/WebCore/rendering/TextBoxPainter.cpp`. Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Source bytes: 88136; source lines: 1790; SHA-256: `a99ab3ec47f623434671dd91c4d719c6482e51cde4573feeb1c5cd3d17058265`.

````cpp
/*
 * Copyright (C) 2021-2023 Apple Inc. All rights reserved.
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

#include "config.h"
#include "TextBoxPainter.h"

#include "CaretRectComputation.h"
#include "CompositionHighlight.h"
#include "DocumentMarkerController.h"
#include "Editor.h"
#include "EventRegion.h"
#include "FontCascadeInlines.h"
#include "GraphicsContext.h"
#include "HTMLAnchorElement.h"
#include "InlineIteratorBoxInlines.h"
#include "InlineIteratorInlineBox.h"
#include "InlineIteratorLineBox.h"
#include "InlineIteratorTextBoxInlines.h"
#include "InlineTextBoxStyle.h"
#include "LayoutInlineTextBox.h"
#include "LineSelection.h"
#include "PaintInfo.h"
#include "PaintInfoInlines.h"
#include "PlatformRenderTheme.h"
#include "RenderBlock.h"
#include "RenderBoxModelObjectInlines.h"
#include "RenderCombineText.h"
#include "RenderElementStyleInlines.h"
#include "RenderElementInlines.h"
#include "RenderGlyph.h"
#include "RenderObjectInlines.h"
#include "RenderText.h"
#include "RenderTheme.h"
#include "RenderView.h"
#include "RenderedDocumentMarker.h"
#include "Settings.h"
#include "StyleTextDecorationInset.h"
#include "StyleTextDecorationLine.h"
#include "StyleTextDecorationThickness.h"
#include "StyledMarkedText.h"
#include "TextPaintStyle.h"
#include "TextPainter.h"
#include <numeric>
#include <ranges>

#if ENABLE(WRITING_TOOLS)
#include "GraphicsContextCG.h"
#endif

namespace WebCore {

static FloatRect calculateDocumentMarkerBounds(const InlineIterator::TextBoxIterator&, const MarkedText&);

static std::optional<bool> emphasisMarkExistsAndIsAbove(const RenderText& renderer, const Style::ComputedStyle& style)
{
    // This function returns true if there are text emphasis marks and they are suppressed by ruby text.
    if (style.textEmphasisStyle().isNone())
        return { };

    auto emphasisPosition = style.textEmphasisPosition();
    bool isAbove = !emphasisPosition.contains(Style::TextEmphasisPositionValue::Under);
    if (style.writingMode().isVerticalTypographic())
        isAbove = !emphasisPosition.contains(Style::TextEmphasisPositionValue::Left);

    auto findRubyAnnotation = [&]() -> RenderBlockFlow* {
        for (auto* baseCandidate = renderer.parent(); baseCandidate; baseCandidate = baseCandidate->parent()) {
            if (!baseCandidate->isInline())
                return { };
            if (baseCandidate->style().display() == Style::DisplayType::RubyBase) {
                if (auto* annotationCandidate = dynamicDowncast<RenderBlockFlow>(baseCandidate->nextSibling()); annotationCandidate && annotationCandidate->style().display() == Style::DisplayType::RubyText)
                    return annotationCandidate;
                return { };
            }
        }
        return { };
    };

    if (auto* annotation = findRubyAnnotation()) {
        // The emphasis marks are suppressed only if there is a ruby annotation box on the same side and it is not empty.
        if (annotation->hasContentfulInlineLine() && isAbove == (annotation->style().rubyPosition() == RubyPosition::Over))
            return { };
    }

    return isAbove;
}

struct ShapedContent {
    StringBuilder text;
    float visualLeft { 0.f }; // visual left of the shaped content.
    size_t textBoxStartOffset { 0 }; // text box's position relative to the shaped content.
    float textBoxVisualLeft { 0.f }; // text box's left relative to the visual left of the shaped content.
};
static void buildTextForShaping(ShapedContent& shapedContent, InlineIterator::BoxModernPath textBox, bool needsTextBoxVisualLeft = false)
{
    ASSERT(textBox.direction() == TextDirection::RTL);

    auto shapingBoundaryIterator = textBox;
    // 1. Find shaping boundary start when we are at the end or inside a shape range (note that we deal with
    // rtl content hence the opposite direction walk)
    // 2. Walk from the start to the end and build the text content.

    auto moveToShapingBoundaryStart = [&] {
        if (shapingBoundaryIterator.box().text().isAtShapingBoundaryStart())
            return;

        shapingBoundaryIterator.traverseNextLeafOnLine();
        for (; !shapingBoundaryIterator.atEnd(); shapingBoundaryIterator.traverseNextLeafOnLine()) {
            auto& displayBox = shapingBoundaryIterator.box();
            if (displayBox.isText()) {
                shapedContent.textBoxStartOffset += displayBox.text().length();
                if (displayBox.text().isAtShapingBoundaryStart())
                    break;
            }
        }
    };
    moveToShapingBoundaryStart();

    if (shapingBoundaryIterator.atEnd() || !shapingBoundaryIterator.isText()) {
        ASSERT_NOT_REACHED();
        return;
    }

    auto buildTextContent = [&] {
        for (; !shapingBoundaryIterator.atEnd(); shapingBoundaryIterator.traversePreviousLeafOnLine()) {
            auto& displayBox = shapingBoundaryIterator.box();
            if (!displayBox.isText())
                continue;
            auto& text = displayBox.text();
            if (shapingBoundaryIterator.direction() == TextDirection::LTR) {
                shapedContent.text.clear();
                return;
            }

            shapedContent.text.append(text.renderedContent());
            if (text.isAtShapingBoundaryEnd()) {
                shapedContent.visualLeft = displayBox.visualRectIgnoringBlockDirection().x();
                return;
            }
        }
        // We should always find the boundary end.
        ASSERT_NOT_REACHED();
        shapedContent.text.clear();
    };
    buildTextContent();

    if (shapedContent.text.isEmpty()) {
        ASSERT_NOT_REACHED();
        return;
    }

    auto computeVisualLeftForTextBox = [&] {
        if (!needsTextBoxVisualLeft)
            return;
        // Starting from visual left, walk all the way to the current text box.
        for (; !shapingBoundaryIterator.atEnd(); shapingBoundaryIterator.traverseNextLeafOnLine()) {
            if (shapingBoundaryIterator == textBox)
                return;
            auto& displayBox = shapingBoundaryIterator.box();
            if (displayBox.isText())
                shapedContent.textBoxVisualLeft += displayBox.visualRectIgnoringBlockDirection().width();
        }
    };
    computeVisualLeftForTextBox();
}

TextBoxPainter::TextBoxPainter(const LayoutIntegration::InlineContent& inlineContent, const InlineDisplay::Box& box, const Style::ComputedStyle& style, PaintInfo& paintInfo, const LayoutPoint& paintOffset)
    : m_textBox(InlineIterator::BoxModernPath { inlineContent, inlineContent.indexForBox(box) })
    , m_renderer(downcast<RenderText>(m_textBox.renderer()))
    , m_document(m_renderer->document())
    , m_style(style)
    , m_logicalRect(m_textBox.isHorizontal() ? m_textBox.visualRectIgnoringBlockDirection() : m_textBox.visualRectIgnoringBlockDirection().transposedRect())
    , m_paintTextRun(m_textBox.textRun())
    , m_paintInfo(paintInfo)
    , m_selectableRange(m_textBox.selectableRange())
    , m_paintOffset(paintOffset)
    , m_paintRect(computePaintRect(paintOffset))
    , m_isFirstLine(m_textBox.isFirstFormattedLine())
    , m_isCombinedText([&] {
        auto* combineTextRenderer = dynamicDowncast<RenderCombineText>(m_renderer.get());
        return combineTextRenderer && combineTextRenderer->isCombined();
    }())
    , m_isPrinting(m_document->printing())
    , m_haveSelection(computeHaveSelection())
{
    ASSERT(paintInfo.phase == PaintPhase::Foreground || paintInfo.phase == PaintPhase::Selection || paintInfo.phase == PaintPhase::TextClip || paintInfo.phase == PaintPhase::EventRegion || paintInfo.phase == PaintPhase::Accessibility
#if ENABLE(AX_CUSTOM_COLOR_MODE)
        || paintInfo.phase == PaintPhase::AXCustomColorComputeBackdrops
        || paintInfo.phase == PaintPhase::AXCustomColorCollectBackgrounds
#endif
    );

    SUPPRESS_UNCOUNTED_LOCAL auto& editor = m_renderer->frame().editor();
    m_containsComposition = m_renderer->textNode() && editor.compositionNode() == m_renderer->textNode();
    m_compositionWithCustomUnderlines = m_containsComposition && editor.compositionUsesCustomUnderlines();
}

TextBoxPainter::~TextBoxPainter() = default;

InlineIterator::TextBoxIterator TextBoxPainter::makeIterator() const
{
    auto pathCopy = m_textBox;
    return InlineIterator::TextBoxIterator { WTF::move(pathCopy) };
}

void TextBoxPainter::paint()
{
    if (m_paintInfo.paintBehavior.contains(PaintBehavior::ExcludeText))
        return;

    if (m_paintInfo.phase == PaintPhase::Selection && !m_haveSelection && !m_paintInfo.paintBehavior.contains(PaintBehavior::IncludeDocumentMarkers))
        return;

    if (m_paintInfo.phase == PaintPhase::EventRegion) {
        constexpr OptionSet<HitTestRequest::Type> hitType { HitTestRequest::Type::IgnoreCSSPointerEventsProperty };
        if (m_renderer->parent()->visibleToHitTesting(hitType))
            m_paintInfo.eventRegionContext()->unite(FloatRoundedRect(m_paintRect), m_renderer, m_style);
        return;
    }

#if ENABLE(AX_CUSTOM_COLOR_MODE)
    if (m_paintInfo.phase == PaintPhase::AXCustomColorCollectBackgrounds)
        return;

    if (m_paintInfo.phase == PaintPhase::AXCustomColorComputeBackdrops) {
        m_paintInfo.axCustomColorBackdropContext()->updateTextBackdrop(m_renderer, m_paintRect);
        return;
    }
#endif

    std::optional<RotationDirection> glyphRotation;
    if (!textBox().isHorizontal() && !m_isCombinedText) {
        glyphRotation = textBox().writingMode().isLineOverLeft()
            ? RotationDirection::Counterclockwise
            : RotationDirection::Clockwise;
        m_paintInfo.context().concatCTM(rotation(m_paintRect, *glyphRotation));
    }

    if (m_paintInfo.phase == PaintPhase::Accessibility) {
        if (glyphRotation) {
            auto transform = rotation(m_paintRect, *glyphRotation);
            m_paintInfo.accessibilityRegionContext()->takeBounds(m_renderer, transform.mapRect(m_paintRect), textBox().lineIndex());
        } else
            m_paintInfo.accessibilityRegionContext()->takeBounds(m_renderer, m_paintRect, textBox().lineIndex());

        return;
    }

    if (m_paintInfo.phase == PaintPhase::Selection && m_paintInfo.paintBehavior.contains(PaintBehavior::IncludeDocumentMarkers))
        paintPlatformDocumentMarkers();

    if (hasSynthesizedGlyph()) {
        if (m_paintInfo.phase == PaintPhase::Foreground)
            paintSynthesizedGlyph();
        if (glyphRotation) {
            auto backRotation = *glyphRotation == RotationDirection::Clockwise ? RotationDirection::Counterclockwise : RotationDirection::Clockwise;
            m_paintInfo.context().concatCTM(rotation(m_paintRect, backRotation));
        }
        return;
    }

    if (m_paintInfo.phase == PaintPhase::Foreground) {
        auto shouldPaintBackgroundFill = [&] {
            if (m_isPrinting)
                return false;
#if ENABLE(TEXT_SELECTION)
            if (m_haveSelection && !m_compositionWithCustomUnderlines)
                return true;
#endif
            if (m_containsComposition && !m_compositionWithCustomUnderlines)
                return true;
            if (auto* markers = m_document->markersIfExists(); markers && markers->hasMarkers())
                return true;
            if (m_document->hasHighlight())
                return true;
            return false;
        };
        if (shouldPaintBackgroundFill())
            paintBackgroundFill();

        paintPlatformDocumentMarkers();
    }

    paintForegroundAndDecorations();

    if (m_paintInfo.phase == PaintPhase::Foreground) {
        if (m_compositionWithCustomUnderlines)
            paintCompositionUnderlines();

        protect(m_renderer->page())->addRelevantRepaintedObject(m_renderer, enclosingLayoutRect(m_paintRect));

        bool isOnlyTextBoxForElement = [&]() {
            if (m_textBox.boxIndex() != 1)
                return false;
            auto& content = m_textBox.inlineContent().displayContent();
            return content.lines.size() == 1 && content.boxes.size() == 2;
        }();

        m_document->didPaintText(textBox().formattingContextRoot(), textBox().visualRectIgnoringBlockDirection(), isOnlyTextBoxForElement);
    }

    if (glyphRotation) {
        auto backRotation = *glyphRotation == RotationDirection::Clockwise
            ? RotationDirection::Counterclockwise
            : RotationDirection::Clockwise;
        m_paintInfo.context().concatCTM(rotation(m_paintRect, backRotation));
    }
}

std::pair<unsigned, unsigned> TextBoxPainter::selectionStartEnd() const
{
    return m_renderer->view().selection().rangeForTextBox(m_renderer, m_selectableRange);
}

MarkedText TextBoxPainter::createMarkedTextFromSelectionInBox()
{
    auto [selectionStart, selectionEnd] = selectionStartEnd();
    if (selectionStart < selectionEnd)
        return { selectionStart, selectionEnd, MarkedText::Type::Selection };
    return { };
}

void TextBoxPainter::paintCompositionForeground(const StyledMarkedText& markedText)
{
    auto hasCompositionCustomHighlights = [&]() {
        if (!m_containsComposition)
            return false;

        Ref editor = protect(m_renderer->frame())->editor();
        return editor->compositionUsesCustomHighlights();
    };

    if (!hasCompositionCustomHighlights()) {
        paintForeground(markedText);
        return;
    }

    // The highlight ranges must be "packed" so that there is no non-empty interval between
    // any two adjacent highlight ranges. This is needed since otherwise, `paintForeground`
    // will not be called in those would-be non-empty intervals.
    Ref editor = protect(m_renderer->frame())->editor();
    auto highlights = editor->customCompositionHighlights();

    Vector<CompositionHighlight> highlightsWithForeground;
    highlightsWithForeground.append({ textBox().start(), highlights[0].startOffset, { }, { } });

    for (size_t i = 0; i < highlights.size(); ++i) {
        highlightsWithForeground.append(highlights[i]);
        if (i != highlights.size() - 1)
            highlightsWithForeground.append({ highlights[i].endOffset, highlights[i + 1].startOffset, { }, { } });
    }

    highlightsWithForeground.append({ highlights.last().endOffset, textBox().end(), { }, { } });

    for (auto& highlight : highlightsWithForeground) {
        auto style = StyledMarkedText::computeStyleForUnmarkedMarkedText(m_renderer, m_style, m_isFirstLine, m_paintInfo);

        if (highlight.endOffset <= textBox().start())
            continue;

        if (highlight.startOffset >= textBox().end())
            break;

        auto [clampedStart, clampedEnd] = m_selectableRange.clamp(highlight.startOffset, highlight.endOffset);

        if (highlight.foregroundColor)
            style.textStyles.fillColor = *highlight.foregroundColor;

        paintForeground({ MarkedText { clampedStart, clampedEnd, MarkedText::Type::Unmarked }, style });

        if (highlight.endOffset > textBox().end())
            break;
    }
}

void TextBoxPainter::paintForegroundAndDecorations()
{
    auto shouldPaintSelectionForeground = m_haveSelection && !m_compositionWithCustomUnderlines;
    auto hasTextDecoration = !m_style->textDecorationLineInEffect().isNone();
    auto hasHighlightDecoration = m_document->hasHighlight() && !MarkedText::collectForHighlights(m_renderer, m_selectableRange, MarkedText::PaintPhase::Decoration).isEmpty();

    auto hasSpellingOrGrammarDecoration = [&] {
        auto markedTexts = MarkedText::collectForDocumentMarkers(m_renderer, m_selectableRange, MarkedText::PaintPhase::Decoration);

        auto hasSpellingError = markedTexts.containsIf([](auto&& markedText) {
            return markedText.type == MarkedText::Type::SpellingError;
        });

        if (hasSpellingError) {
            auto spellingErrorStyle = m_renderer->spellingErrorPseudoStyle();
            if (spellingErrorStyle)
                return !spellingErrorStyle->textDecorationLineInEffect().isNone();
        }

        auto hasGrammarError = markedTexts.containsIf([](auto&& markedText) {
            return markedText.type == MarkedText::Type::GrammarError;
        });

        if (hasGrammarError) {
            auto grammarErrorStyle = m_renderer->grammarErrorPseudoStyle();
            if (grammarErrorStyle)
                return !grammarErrorStyle->textDecorationLineInEffect().isNone();
        }

        return false;
    };

    auto hasSelectionDecoration = [&] {
        if (!shouldPaintSelectionForeground)
            return false;
        CheckedPtr selectionStyle = m_renderer->selectionPseudoStyle();
        return selectionStyle && !selectionStyle->textDecorationLineInEffect().isNone();
    };

    auto hasDecoration = hasTextDecoration || hasHighlightDecoration || hasSpellingOrGrammarDecoration() || hasSelectionDecoration();

    auto contentMayNeedStyledMarkedText = [&] {
        if (hasDecoration)
            return true;
        if (shouldPaintSelectionForeground)
            return true;
        if (auto* markers = m_document->markersIfExists(); markers && markers->hasMarkers())
            return true;
        if (m_document->hasHighlight())
            return true;
        return false;
    };
    auto startPosition = m_selectableRange.clamp(textBox().start());
    auto endPosition = m_selectableRange.clamp(textBox().end());

    if (!contentMayNeedStyledMarkedText()) {
        auto markedText = MarkedText { startPosition, endPosition, MarkedText::Type::Unmarked };
        auto styledMarkedText = StyledMarkedText { markedText, StyledMarkedText::computeStyleForUnmarkedMarkedText(m_renderer, m_style, m_isFirstLine, m_paintInfo) };
        paintCompositionForeground(styledMarkedText);
        return;
    }

    Vector<MarkedText> markedTexts;
    if (m_paintInfo.phase != PaintPhase::Selection) {
        // The marked texts for the gaps between document markers and selection are implicitly created by subdividing the entire line.
        markedTexts.append({ startPosition, endPosition, MarkedText::Type::Unmarked });

        if (!m_isPrinting) {
            markedTexts.appendVector(MarkedText::collectForDocumentMarkers(m_renderer, m_selectableRange, MarkedText::PaintPhase::Foreground));
            markedTexts.appendVector(MarkedText::collectForHighlights(m_renderer, m_selectableRange, MarkedText::PaintPhase::Foreground));

            bool shouldPaintDraggedContent = !(m_paintInfo.paintBehavior.contains(PaintBehavior::ExcludeSelection));
            if (shouldPaintDraggedContent) {
                auto markedTextsForDraggedContent = MarkedText::collectForDraggedAndTransparentContent(DocumentMarkerType::DraggedContent, m_renderer, m_selectableRange);
                if (!markedTextsForDraggedContent.isEmpty()) {
                    shouldPaintSelectionForeground = false;
                    markedTexts.appendVector(WTF::move(markedTextsForDraggedContent));
                }
            }
            auto markedTextsForTransparentContent = MarkedText::collectForDraggedAndTransparentContent(DocumentMarkerType::TransparentContent, m_renderer, m_selectableRange);
            if (!markedTextsForTransparentContent.isEmpty())
                markedTexts.appendVector(WTF::move(markedTextsForTransparentContent));

            markedTexts.appendVector(MarkedText::collectForDictationStreamingOpacity(m_renderer, m_selectableRange));
        }
    }
    // The selection marked text acts as a placeholder when computing the marked texts for the gaps...
    if (shouldPaintSelectionForeground) {
        ASSERT(!m_isPrinting);
        auto selectionMarkedText = createMarkedTextFromSelectionInBox();
        if (!selectionMarkedText.isEmpty())
            markedTexts.append(WTF::move(selectionMarkedText));
    }

    auto styledMarkedTexts = StyledMarkedText::subdivideAndResolve(markedTexts, m_renderer, m_isFirstLine, m_paintInfo);

    // ... now remove the selection marked text if we are excluding selection.
    if (!m_isPrinting && m_paintInfo.paintBehavior.contains(PaintBehavior::ExcludeSelection)) {
        styledMarkedTexts.removeAllMatching([] (const StyledMarkedText& markedText) {
            return markedText.type == MarkedText::Type::Selection;
        });
    }

    if (hasDecoration && m_paintInfo.phase != PaintPhase::Selection) {
        unsigned length = m_selectableRange.truncation.value_or(m_paintTextRun.length());
        unsigned selectionStart = 0;
        unsigned selectionEnd = 0;
        if (m_haveSelection)
            std::tie(selectionStart, selectionEnd) = selectionStartEnd();

        FloatRect textDecorationSelectionClipOutRect;
        if ((m_paintInfo.paintBehavior.contains(PaintBehavior::ExcludeSelection)) && selectionStart < selectionEnd && selectionEnd <= length) {
            textDecorationSelectionClipOutRect = m_paintRect;
            float logicalWidthBeforeRange;
            float logicalWidthAfterRange;
            float logicalSelectionWidth = fontCascade().widthOfTextRange(m_paintTextRun, selectionStart, selectionEnd, logicalWidthBeforeRange, logicalWidthAfterRange);
            // FIXME: Do we need to handle vertical bottom to top text?
            if (!textBox().isHorizontal()) {
                textDecorationSelectionClipOutRect.move(0, logicalWidthBeforeRange);
                textDecorationSelectionClipOutRect.setHeight(logicalSelectionWidth);
            } else if (textBox().direction() == TextDirection::RTL) {
                textDecorationSelectionClipOutRect.move(logicalWidthAfterRange, 0);
                textDecorationSelectionClipOutRect.setWidth(logicalSelectionWidth);
            } else {
                textDecorationSelectionClipOutRect.move(logicalWidthBeforeRange, 0);
                textDecorationSelectionClipOutRect.setWidth(logicalSelectionWidth);
            }
        }

        // Coalesce styles of adjacent marked texts to minimize the number of drawing commands.
        auto coalescedStyledMarkedTexts = StyledMarkedText::coalesceAdjacentWithEqualDecorations(styledMarkedTexts);

        for (auto& markedText : coalescedStyledMarkedTexts) {
            unsigned startOffset = markedText.startOffset;
            unsigned endOffset = markedText.endOffset;
            if (startOffset < endOffset) {
                // Avoid measuring the text when the entire line box is selected as an optimization.
                auto snappedPaintRect = snapRectToDevicePixelsWithWritingDirection(LayoutRect { m_paintRect }, m_document->deviceScaleFactor(), m_paintTextRun.ltr());
                if (startOffset || endOffset != m_paintTextRun.length()) {
                    LayoutRect selectionRect = { m_paintRect.x(), m_paintRect.y(), m_paintRect.width(), m_paintRect.height() };
                    fontCascade().adjustSelectionRectForText(m_renderer->canUseSimplifiedTextMeasuring().value_or(false), m_paintTextRun, selectionRect, startOffset, endOffset);
                    snappedPaintRect = snapRectToDevicePixelsWithWritingDirection(selectionRect, m_document->deviceScaleFactor(), m_paintTextRun.ltr());
                }
                auto decorationPaintRect = writingMode().isHorizontal() ? FloatRect { snappedPaintRect.x(), m_paintRect.y(), snappedPaintRect.width(), snappedPaintRect.height() } : snappedPaintRect;
                auto decorationPainter = createDecorationPainter(markedText, textDecorationSelectionClipOutRect);
                paintBackgroundDecorations(decorationPainter, markedText, decorationPaintRect);
                paintCompositionForeground(markedText);
                paintForegroundDecorations(decorationPainter, markedText, decorationPaintRect);
            }
        }
    } else {
        // Coalesce styles of adjacent marked texts to minimize the number of drawing commands.
        auto coalescedStyledMarkedTexts = StyledMarkedText::coalesceAdjacentWithEqualForeground(styledMarkedTexts);

        if (coalescedStyledMarkedTexts.isEmpty())
            return;

        for (auto& markedText : coalescedStyledMarkedTexts)
            paintCompositionForeground(markedText);
    }
}

void TextBoxPainter::paintBackgroundFill()
{
    if (m_containsComposition && !m_compositionWithCustomUnderlines) {
        Ref editor = protect(m_renderer->frame())->editor();

        if (editor->compositionUsesCustomHighlights()) {
            for (auto& highlight : editor->customCompositionHighlights()) {
                if (!highlight.backgroundColor)
                    continue;

                if (highlight.endOffset <= textBox().start())
                    continue;

                if (highlight.startOffset >= textBox().end())
                    break;

                auto [clampedStart, clampedEnd] = m_selectableRange.clamp(highlight.startOffset, highlight.endOffset);
                paintBackgroundFillForRange(clampedStart, clampedEnd, *highlight.backgroundColor, BackgroundStyle::Rounded);

                if (highlight.endOffset > textBox().end())
                    break;
            }
        } else {
            auto [clampedStart, clampedEnd] = m_selectableRange.clamp(editor->compositionStart(), editor->compositionEnd());
            paintBackgroundFillForRange(clampedStart, clampedEnd, CompositionHighlight::defaultCompositionFillColor, BackgroundStyle::Normal);
        }
    }

    Vector<MarkedText> markedTexts;
    markedTexts.appendVector(MarkedText::collectForDocumentMarkers(m_renderer, m_selectableRange, MarkedText::PaintPhase::Background));
    markedTexts.appendVector(MarkedText::collectForHighlights(m_renderer, m_selectableRange, MarkedText::PaintPhase::Background));

    auto styledMarkedTexts = StyledMarkedText::subdivideAndResolve(markedTexts, m_renderer, m_isFirstLine, m_paintInfo);

    // Coalesce styles of adjacent marked texts to minimize the number of drawing commands.
    auto coalescedStyledMarkedTexts = StyledMarkedText::coalesceAdjacentWithEqualBackground(styledMarkedTexts);
    for (auto& markedText : coalescedStyledMarkedTexts)
        paintBackgroundFillForRange(markedText.startOffset, markedText.endOffset, markedText.style.backgroundColor, BackgroundStyle::Normal);

#if ENABLE(TEXT_SELECTION)
    auto hasSelectionWithNonCustomUnderline = m_haveSelection && !m_compositionWithCustomUnderlines;
    if (hasSelectionWithNonCustomUnderline && !m_paintInfo.context().paintingDisabled()) {
        auto selectionMarkedText = createMarkedTextFromSelectionInBox();
        if (!selectionMarkedText.isEmpty()) {
            auto selectionMarkedTexts = Vector<MarkedText>::from(WTF::move(selectionMarkedText));
            for (auto& markedText : StyledMarkedText::subdivideAndResolve(selectionMarkedTexts, m_renderer, m_isFirstLine, m_paintInfo))
                paintBackgroundFillForRange(markedText.startOffset, markedText.endOffset, markedText.style.backgroundColor, BackgroundStyle::Normal);
        }
    }
#endif
}

LayoutRect TextBoxPainter::selectionRectForRange(unsigned startOffset, unsigned endOffset) const
{
    // Note that if the text is truncated, we let the thing being painted in the truncation
    // draw its own highlight.
    auto lineBox = makeIterator()->lineBox();
    auto selectionBottom = LineSelection::logicalBottom(*lineBox);
    auto selectionTop = LineSelection::logicalTopAdjustedForPrecedingBlock(*lineBox);
    // Use same y positioning and height as for selection, so that when the selection and this subrange are on
    // the same word there are no pieces sticking out.
    auto deltaY = LayoutUnit { writingMode().isLineInverted() ? selectionBottom - m_logicalRect.maxY() : m_logicalRect.y() - selectionTop };
    auto selectionHeight = LayoutUnit { std::max(0.f, selectionBottom - selectionTop) };
    auto selectionRect = LayoutRect { LayoutUnit(m_paintRect.x()), LayoutUnit(m_paintRect.y() - deltaY), LayoutUnit(m_logicalRect.width()), selectionHeight };

    if (isInsideShapedContent()) {
        auto shapedContent = ShapedContent { };

        buildTextForShaping(shapedContent, m_textBox, true);
        selectionRect.setX(selectionRect.x() - shapedContent.textBoxVisualLeft);
        auto selectionLength = endOffset - startOffset;
        auto adjustedStartOffset = shapedContent.textBoxStartOffset + startOffset;

        auto characterScanForCodePath = true;
        auto expansion = m_textBox.box().expansion();
        auto paintRect = m_paintRect;
        paintRect.shiftXEdgeTo(shapedContent.visualLeft);
        auto run = TextRun { shapedContent.text, paintRect.x(), expansion.horizontalExpansion, expansion.behavior, m_textBox.direction(), m_style->rtlOrdering() == Order::Visual, characterScanForCodePath };

        fontCascade().adjustSelectionRectForText(false, run, selectionRect, adjustedStartOffset, adjustedStartOffset + selectionLength);
        return selectionRect;
    }

    fontCascade().adjustSelectionRectForText(m_renderer->canUseSimplifiedTextMeasuring().value_or(false), m_paintTextRun, selectionRect, startOffset, endOffset);
    return selectionRect;
}

void TextBoxPainter::paintBackgroundFillForRange(unsigned startOffset, unsigned endOffset, const Color& color, BackgroundStyle backgroundStyle)
{
    if (startOffset >= endOffset)
        return;

    GraphicsContext& context = m_paintInfo.context();
    GraphicsContextStateSaver stateSaver { context };
    updateGraphicsContext(context, TextPaintStyle { color }); // Don't draw text at all!

    auto selectionRect = selectionRectForRange(startOffset, endOffset);

    if (m_paintTextRun.length() == endOffset - startOffset) {
        // FIXME: We should reconsider re-measuring the content when non-whitespace runs are joined together (see webkit.org/b/251318).
        auto unAdjustedSelectionRectMaxX = LayoutUnit { m_paintRect.x() + m_logicalRect.width() };
        auto visualRight = std::max(selectionRect.maxX(), unAdjustedSelectionRectMaxX);
        selectionRect.shiftMaxXEdgeTo(visualRight);
    }

    // FIXME: Support painting combined text. See <https://bugs.webkit.org/show_bug.cgi?id=180993>.
    auto backgroundRect = snapRectToDevicePixels(selectionRect, m_document->deviceScaleFactor());
    if (!writingMode().isHorizontal()) {
        auto ctm = context.getCTM();
        if (auto inverseCTM = ctm.inverse())
            backgroundRect = inverseCTM->mapRect(snapRectToDevicePixels(LayoutRect { ctm.mapRect(FloatRect { selectionRect }) }, 1));
    }
    if (backgroundStyle == BackgroundStyle::Rounded) {
        backgroundRect.expand(-1, -1);
        backgroundRect.move(0.5, 0.5);
        context.fillRoundedRect(FloatRoundedRect { backgroundRect, CornerRadii { 2 } }, color);
        return;
    }

    context.fillRect(backgroundRect, color);
}

static bool NODELETE isTransparent(const StyledMarkedText& markedText)
{
    switch (markedText.type) {
    case MarkedText::Type::DraggedContent:
    case MarkedText::Type::TransparentContent:
    case MarkedText::Type::DictationStreamingOpacity:
        return true;

    default:
        return false;
    }
}

bool TextBoxPainter::hasSynthesizedGlyph() const
{
    CheckedPtr inlineTextBox = dynamicDowncast<Layout::InlineTextBox>(m_textBox.box().layoutBox());
    return inlineTextBox && inlineTextBox->hasSynthesizedGlyph();
}

void TextBoxPainter::paintSynthesizedGlyph()
{
    // Drawn from the font metrics rather than from the character the counter style produced, matching the width TextUtil::width reserved for it.
    auto& fontMetrics = m_style->metricsOfPrimaryFont();
    auto ascent = fontMetrics.ascent();
    auto bulletWidth = (ascent * 2 / 3 + 1) / 2;

    auto& context = m_paintInfo.context();
    auto color = m_style->visitedDependentTextFillColorApplyingColorFilter();
    context.setFillColor(color);
    context.setStrokeColor(color);
    context.setStrokeStyle(StrokeStyle::SolidStroke);

    // Check 'content' before 'list-style-type'.
    if (CheckedPtr glyphRenderer = dynamicDowncast<RenderGlyph>(m_renderer.get())) {
        auto fontSize = m_style->fontCascade().size();
        auto size = fontSize / 4;
        FloatPoint center { glyphRenderer->advanceRatio() * fontSize / 2, fontMetrics.ascent(FontBaseline::Central) };
        center.moveBy(m_paintRect.location());

        bool shouldFlip = (glyphRenderer->glyph() == SynthesizedGlyph::PickerDown) == textBox().writingMode().isLineInverted();
        GraphicsContextStateSaver stateSaver(context);
        if (shouldFlip) {
            // A flipped chevron reads optically low, so lift it by a quarter of the glyph size.
            context.concatCTM(AffineTransform { }.translate(center.x(), center.y() - size / 4).rotate(180).translate(-center.x(), -center.y()));
        }

        context.setLineCap(LineCap::Butt);
        context.setLineJoin(LineJoin::Miter);
        context.setStrokeThickness(std::max(1.0f, fontSize * 0.05f));

        // Draw chevron pointing down.
        Path chevron;
        chevron.moveTo({ center.x() - size, center.y() - size / 2 });
        chevron.addLineTo({ center.x(), center.y() + size / 2 });
        chevron.addLineTo({ center.x() + size, center.y() - size / 2 });
        context.strokePath(chevron);
        return;
    }

    auto markerRect = FloatRect { 1, 3 * (ascent - ascent * 2 / 3) / 2, bulletWidth, bulletWidth };
    markerRect.moveBy(m_paintRect.location());

    auto listStyleType = m_style->listStyleType();
    if (listStyleType.isDisc())
        context.fillEllipse(markerRect);
    else if (listStyleType.isSquare())
        context.fillRect(markerRect);
    else if (listStyleType.isCircle()) {
        context.setStrokeThickness(1.0f);
        context.strokeEllipse(markerRect);
    } else
        ASSERT_NOT_REACHED();
}

void TextBoxPainter::paintForeground(const StyledMarkedText& markedText)
{
    if (markedText.startOffset >= markedText.endOffset)
        return;

    auto& context = m_paintInfo.context();
    const FontCascade& font = fontCascade();

    float emphasisMarkOffset = 0;
    auto emphasisExistsAndIsAbove = emphasisMarkExistsAndIsAbove(m_renderer, m_style);
    auto& emphasisMark = emphasisExistsAndIsAbove ? m_style->textEmphasisStyle().markString() : nullAtom();
    if (!emphasisMark.isEmpty())
        emphasisMarkOffset = *emphasisExistsAndIsAbove ? -font.metricsOfPrimaryFont().ascent()  - font.emphasisMarkDescent(emphasisMark) : font.metricsOfPrimaryFont().descent() + font.emphasisMarkAscent(emphasisMark);

    TextPainter textPainter {
        context,
        font,
        m_style,
        markedText.style.textStyles,
        markedText.style.textShadow,
        (!markedText.style.textShadow.isNone() && !m_style->appleColorFilter().isNone()) ? m_style->appleColorFilter() : Style::AppleColorFilter::none(),
        emphasisMark,
        emphasisMarkOffset,
        m_isCombinedText ? &downcast<RenderCombineText>(m_renderer.get()) : nullptr
    };

    bool isTransparentMarkedText = isTransparent(markedText);
    GraphicsContextStateSaver stateSaver(context, markedText.style.textStyles.strokeWidth > 0 || isTransparentMarkedText);
    if (isTransparentMarkedText)
        context.setAlpha(markedText.style.alpha);
    updateGraphicsContext(context, markedText.style.textStyles);

    if (isInsideShapedContent() && paintForegroundForShapeRange(textPainter))
        return;

    // Backgrounds paint before all text, so clip a stroked partial segment to its forward edge to keep its stroke overflow off a following highlight's background (adjacent inline boxes composite this way); the slack leaves the other edges effectively unclipped.
    GraphicsContextStateSaver clipStateSaver(context, false);
    if (markedText.style.textStyles.strokeWidth > 0 && markedText.endOffset < m_paintTextRun.length()) {
        LayoutRect segmentRect { m_paintRect };
        fontCascade().adjustSelectionRectForText(m_renderer->canUseSimplifiedTextMeasuring().value_or(false), m_paintTextRun, segmentRect, markedText.startOffset, markedText.endOffset);
        auto snapped = snapRectToDevicePixelsWithWritingDirection(segmentRect, m_document->deviceScaleFactor(), m_paintTextRun.ltr());
        static constexpr float overflowSlack = 4096;
        bool ltr = m_paintTextRun.ltr();
        FloatRect clipRect;
        if (writingMode().isHorizontal()) {
            float minX = ltr ? m_paintRect.x() - overflowSlack : snapped.x();
            float maxX = ltr ? snapped.maxX() : m_paintRect.maxX() + overflowSlack;
            clipRect = { minX, m_paintRect.y() - overflowSlack, maxX - minX, m_paintRect.height() + 2 * overflowSlack };
        } else {
            float minY = ltr ? m_paintRect.y() - overflowSlack : snapped.y();
            float maxY = ltr ? snapped.maxY() : m_paintRect.maxY() + overflowSlack;
            clipRect = { m_paintRect.x() - overflowSlack, minY, m_paintRect.width() + 2 * overflowSlack, maxY - minY };
        }
        clipStateSaver.save();
        context.clip(clipRect);
    }

    textPainter.setGlyphDisplayListIfNeeded(textBox().box(), m_paintInfo, m_style, m_paintTextRun);
    // TextPainter wants the box rectangle and text origin of the entire line box.
    textPainter.paintRange(m_paintTextRun, m_paintRect, textOriginFromPaintRect(m_paintRect), markedText.startOffset, markedText.endOffset);
}

bool TextBoxPainter::paintForegroundForShapeRange(TextPainter& textPainter)
{
    ASSERT(m_document->settings().textShapingAcrossInlineBoxes());
    ASSERT(m_textBox.direction() == TextDirection::RTL);

    auto& context = m_paintInfo.context();

    auto clipRect = [&] {
        // We could also just use ink overflow here but since non-range painting
        // sets up no clipping, we should not do that either here.
        auto rect = FloatRect::infiniteRect();
        rect.setX(m_paintRect.x());
        // Note that this is RTL direction.
        auto& textContent = m_textBox.box().text();
        if (!textContent.isAtShapingBoundaryStart())
            rect.setWidth(m_paintRect.width());
        // FIXME: Setup a semi-inifite rect for the (visually) first box where x is -inifite with fixed maxX.
        return rect;
    };
    context.save();
    context.clip(clipRect());
    auto shapedContent = ShapedContent { };
    buildTextForShaping(shapedContent, m_textBox);

    if (shapedContent.text.isEmpty())
        return false;

    auto paintRect = m_paintRect;
    paintRect.shiftXEdgeTo(m_paintOffset.x() + shapedContent.visualLeft);

    auto characterScanForCodePath = true;
    auto expansion = m_textBox.box().expansion();
    auto run = TextRun { shapedContent.text, paintRect.x(), expansion.horizontalExpansion, expansion.behavior, m_textBox.direction(), m_style->rtlOrdering() == Order::Visual, characterScanForCodePath };
    run.disableSpacing();

    textPainter.paintRange(run, paintRect, textOriginFromPaintRect(paintRect), 0, shapedContent.text.length());
    context.restore();
    return true;
}

TextDecorationPainter TextBoxPainter::createDecorationPainter(const StyledMarkedText& markedText, const FloatRect& clipOutRect)
{
    auto& context = m_paintInfo.context();

    updateGraphicsContext(context, markedText.style.textStyles);

    // Note that if the text is truncated, we let the thing being painted in the truncation
    // draw its own decoration.
    GraphicsContextStateSaver stateSaver { context, false };
    bool isTransparentContent = isTransparent(markedText);
    if (isTransparentContent || !clipOutRect.isEmpty()) {
        stateSaver.save();
        if (isTransparentContent)
            context.setAlpha(markedText.style.alpha);
        if (!clipOutRect.isEmpty())
            context.clipOut(clipOutRect);
    }

    // Create painter
    return {
        context,
        fontCascade(),
        markedText.style.textShadow,
        (!markedText.style.textShadow.isNone() && !m_style->appleColorFilter().isNone()) ? m_style->appleColorFilter() : Style::AppleColorFilter::none(),
        m_document->printing(),
        writingMode()
    };
}

static inline float computedAutoTextDecorationThickness(const Style::ComputedStyle& styleToUse, float deviceScaleFactor)
{
    return ceilToDevicePixel(Style::TextDecorationThickness { CSS::Keyword::Auto { } }.resolve(styleToUse), deviceScaleFactor);
}

static inline float resolveTextDecorationThicknessForPaintingBox(const std::optional<Style::TextDecorationThickness>& originatorThickness, const Style::ComputedStyle& paintingBoxStyle, float deviceScaleFactor)
{
    // The originator's text-decoration-thickness propagates to descendants and is resolved against the painting
    // box so its font size and zoom apply. When the originator did not specify a thickness, fall back to auto.
    auto thickness = originatorThickness.value_or(Style::TextDecorationThickness { CSS::Keyword::Auto { } });
    return ceilToDevicePixel(thickness.resolve(paintingBoxStyle), deviceScaleFactor);
}

static inline float computedLinethroughCenter(const Style::ComputedStyle& styleToUse, float textDecorationThickness, float autoTextDecorationThickness)
{
    auto center = 2 * styleToUse.metricsOfPrimaryFont().ascent() / 3 + autoTextDecorationThickness / 2;
    return center - textDecorationThickness / 2;
}

static inline Style::TextDecorationLine computedTextDecorationType(const Style::ComputedStyle& style, const TextDecorationPainter::Styles& textDecorationStyles)
{
    auto textDecorations = style.textDecorationLineInEffect();
    textDecorations.addOrReplaceIfNotNone(TextDecorationPainter::textDecorationsInEffectForStyle(textDecorationStyles));
    return textDecorations;
}

static inline CheckedRef<const Style::ComputedStyle> decoratingBoxStyleForInlineBox(const InlineIterator::InlineBox& inlineBox, bool isFirstLine)
{
    if (!inlineBox.isRootInlineBox())
        return inlineBox.style();
    // "When specified on or propagated to a block container that establishes an inline formatting context, the decorations are propagated to an anonymous
    // inline box that wraps all the in-flow inline-level children of the block container"
    // https://drafts.csswg.org/css-text-decor-4/#line-decoration
    // Sadly we don't have the concept of anonymous inline box for all inline-level chidren when content forces us to generate anonymous block containers.
    for (const RenderElement* ancestor = &inlineBox.renderer(); ancestor; ancestor = ancestor->parent()) {
        if (!ancestor->isAnonymous())
            return isFirstLine ? ancestor->firstLineStyle() : ancestor->style();
    }
    ASSERT_NOT_REACHED();
    return inlineBox.style();
}

static inline bool isAlwaysDecoratingBoxForBackground(const InlineIterator::InlineBox& inlineBox)
{
    // <font> and <a> are always considered decorating boxes, so a propagated under/overline is drawn at their
    // position, not only at the root inline box.
    RefPtr element = inlineBox.renderer().element();
    return element && (is<HTMLAnchorElement>(*element) || element->hasTagName(HTMLNames::fontTag));
}

static inline bool isDecoratingBoxForBackground(const InlineIterator::InlineBox& inlineBox, const Style::ComputedStyle& styleToUse)
{
    if (isAlwaysDecoratingBoxForBackground(inlineBox))
        return true;
    return styleToUse.textDecorationLine().containsAny({ Style::TextDecorationLine::Flag::Underline, Style::TextDecorationLine::Flag::Overline })
        || (inlineBox.isRootInlineBox() && styleToUse.textDecorationLineInEffect().containsAny({ Style::TextDecorationLine::Flag::Underline, Style::TextDecorationLine::Flag::Overline }));
}

static inline bool isDecoratingBoxForForeground(const InlineIterator::InlineBox& inlineBox, const Style::ComputedStyle& styleToUse)
{
    // Line-through has no <a>/<font> always-decorating quirk (that is an under/overline behavior). A box
    // establishes line-through when its own style sets it, or - for the root inline box - when it is in effect.
    return styleToUse.textDecorationLine().containsAny({ Style::TextDecorationLine::Flag::LineThrough })
        || (inlineBox.isRootInlineBox() && styleToUse.textDecorationLineInEffect().containsAny({ Style::TextDecorationLine::Flag::LineThrough }));
}

void TextBoxPainter::collectDecoratingBoxesForBackgroundPainting(DecoratingBoxList& decoratingBoxList, const InlineIterator::TextBoxIterator& textBox, const FloatRect& textBoxRect, const TextDecorationPainter::Styles& overrideDecorationStyle)
{
    auto parentInlineBox = textBox->parentInlineBox();
    if (!parentInlineBox) {
        ASSERT_NOT_REACHED();
        return;
    }

    auto textBoxLocation = textBoxRect.location();
    auto decorationWidth = textBoxRect.width();
    if (parentInlineBox->isRootInlineBox()) {
        CheckedRef rootStyle = decoratingBoxStyleForInlineBox(*parentInlineBox, m_isFirstLine);
        decoratingBoxList.append({ parentInlineBox, rootStyle, overrideDecorationStyle, textBoxLocation, decorationWidth });
        // The highlight overlay's decoration layers over the originating box's own decoration rather than replacing it.
        auto rootDecorationStyle = TextDecorationPainter::stylesForRenderer(parentInlineBox->renderer(), rootStyle->textDecorationLineInEffect(), m_isFirstLine);
        if (!rootStyle->textDecorationLineInEffect().isNone() && overrideDecorationStyle != rootDecorationStyle)
            decoratingBoxList.append({ parentInlineBox, rootStyle, rootDecorationStyle, textBoxLocation, decorationWidth });
        return;
    }

    if (writingMode().isLineInverted()) {
        // FIXME: underlineOffsetForTextBoxPainting returns incorrect value for vertical-lr.
        decoratingBoxList.append({ parentInlineBox, m_style.get(), overrideDecorationStyle, textBoxLocation, decorationWidth });
        return;
    }

    auto appendIfIsDecoratingBoxForBackground = [&] (auto& inlineBox) {
        CheckedRef style = decoratingBoxStyleForInlineBox(*inlineBox, m_isFirstLine);
        auto computedDecorationStyle = TextDecorationPainter::stylesForRenderer(inlineBox->renderer(), style->textDecorationLineInEffect(), m_isFirstLine);
        auto isParentInlineBox = &inlineBox == &parentInlineBox;

        if (inlineBox->isRubyBase()) {
            decorationWidth = inlineBox->logicalWidth();
            writingMode().isHorizontal() ? textBoxLocation.setX(m_paintOffset.x() + inlineBox->logicalLeft()) : textBoxLocation.setX(textBoxLocation.x() - (textBox->logicalLeft() - inlineBox->logicalLeft()));
        }

        // Some cases even non-decoration boxes may have some decoration pieces coming from the marked text (e.g. highlight).
        if (!isDecoratingBoxForBackground(*inlineBox, style) && (!isParentInlineBox || overrideDecorationStyle == computedDecorationStyle))
            return;

        auto decoratingBoxLocation = textBoxLocation;
        // Normally text box top is aligned with the parent inline box top (i.e. textBox->logicalTop() == parentInlineBox->logicalTop()) but when inline box sides are trimmed (see: text-box property)
        // inline box gets an offset while text box does not.
        if (&inlineBox->renderer() != &parentInlineBox->renderer()) {
            auto decoratingBoxContentBoxTop = inlineBox->logicalTop() + (!inlineBox->isRootInlineBox() ? inlineBox->renderer().borderAndPaddingBefore() : LayoutUnit(0_lu));
            auto parentInlineBoxContentBoxTop = parentInlineBox->logicalTop() + (!parentInlineBox->isRootInlineBox() ? parentInlineBox->renderer().borderAndPaddingBefore() : LayoutUnit(0_lu));
            decoratingBoxLocation.moveBy(FloatPoint { 0.f, decoratingBoxContentBoxTop - parentInlineBoxContentBoxTop + textBoxEdgeAdjustmentForUnderline(parentInlineBox->style()) });
        } else
            decoratingBoxLocation.moveBy(FloatPoint { 0.f, textBoxEdgeAdjustmentForUnderline(inlineBox->style()) });

        auto decorationStyles = [&] {
            auto styles = isParentInlineBox ? overrideDecorationStyle : computedDecorationStyle;
            if (inlineBox->isRootInlineBox() || isAlwaysDecoratingBoxForBackground(*inlineBox))
                return styles;

            // A non-root inline box only originates the decorations set on it directly. An underline or overline that
            // propagates through it is painted by the box that introduced it (an ancestor, ultimately the root inline
            // box; <a>/<font> are decorating boxes too), so this box must not paint it again at its own position.
            auto ownDecorations = style->textDecorationLine();
            if (!ownDecorations.hasUnderline() && (!isParentInlineBox || styles.underline == computedDecorationStyle.underline))
                styles.underline = { };
            if (!ownDecorations.hasOverline() && (!isParentInlineBox || styles.overline == computedDecorationStyle.overline))
                styles.overline = { };
            return styles;
        };

        decoratingBoxList.append({
            inlineBox,
            style,
            decorationStyles(),
            decoratingBoxLocation,
            decorationWidth
        });
    };

    // FIXME: Figure out if the decoration styles coming from the styled marked text should be used only on the closest inline box (direct parent).
    appendIfIsDecoratingBoxForBackground(parentInlineBox);
    for (auto ancestorInlineBox = parentInlineBox->parentInlineBox(); ancestorInlineBox; ancestorInlineBox = ancestorInlineBox->parentInlineBox()) {
        appendIfIsDecoratingBoxForBackground(ancestorInlineBox);
        if (ancestorInlineBox->isRootInlineBox())
            break;
    }
}

static float autoTextDecorationInset(const Style::ComputedStyle& style)
{
    // A small UA-chosen inset (relative to font size) so that two adjacent identical underlined
    // elements do not appear to share a single continuous underline (important for e.g. Chinese,
    // where underlining is a form of punctuation).
    return style.usedFontSize() / 8;
}

struct DecoratingBoxFragmentInlineSizes {
    float preceding { 0.f };
    float current { 0.f };
    float following { 0.f };

    float total() const { return preceding + current + following; }
};
static DecoratingBoxFragmentInlineSizes decoratingBoxFragmentInlineSizes(const InlineIterator::InlineBox& decoratingInlineBox)
{
    auto inlineSizes = DecoratingBoxFragmentInlineSizes { .current = decoratingInlineBox.logicalWidth() };
    for (auto fragment = decoratingInlineBox.nextInlineBoxLineLeftward(); fragment; fragment.traverseInlineBoxLineLeftward())
        inlineSizes.preceding += fragment->logicalWidth();
    for (auto fragment = decoratingInlineBox.nextInlineBoxLineRightward(); fragment; fragment.traverseInlineBoxLineRightward())
        inlineSizes.following += fragment->logicalWidth();
    return inlineSizes;
}

std::pair<FloatPoint, float> TextBoxPainter::insetAdjustedDecorationLocationAndWidth(const DecoratingBox& decoratingBox, const StyledMarkedText& markedText) const
{
    auto boxOrigin = decoratingBox.location;
    auto width = decoratingBox.contentWidth;

    // text-decoration-inset and box-decoration-break are not inherited, but a decoration propagates from
    // the box that introduces it to the (possibly descendant) box that paints it, so they are resolved
    // from the originating box and carried on the decoration Styles (see collectStylesForRenderer()),
    // alongside the decoration color/thickness.
    auto& insetStyles = decoratingBox.textDecorationStyles;
    if (!insetStyles.inset)
        return { boxOrigin, width };
    auto& inset = *insetStyles.inset;

    auto& style = decoratingBox.style.get();
    auto writingMode = style.writingMode();
    auto decoratingInlineBox = decoratingBox.inlineBox;
    auto isSliced = insetStyles.boxDecorationBreak != BoxDecorationBreak::Clone;

    auto fragmentInlineSizes = isSliced ? decoratingBoxFragmentInlineSizes(*decoratingInlineBox) : DecoratingBoxFragmentInlineSizes { .current = decoratingInlineBox->logicalWidth() };
    auto autoValue = inset.isAuto() ? autoTextDecorationInset(style) : 0.f;
    auto percentageBasis = fragmentInlineSizes.total();
    auto startInset = inset.resolvedStart(style, autoValue, percentageBasis);
    auto endInset = inset.resolvedEnd(style, autoValue, percentageBasis);
    if (!startInset && !endInset)
        return { boxOrigin, width };

    // box-decoration-break: the start inset applies only to the first fragment's start edge and the
    // end inset only to the last fragment's end edge; for box-decoration-break: clone every fragment is
    // a complete box, so both endpoints are inset on every line.
    auto closedEdges = isSliced ? decoratingInlineBox->closedEdges() : RectEdges<bool>(true);
    auto hasLogicalStartEdge = closedEdges.start(writingMode);
    auto hasLogicalEndEdge = closedEdges.end(writingMode);

    auto insetForFragment = [](float inset, float inlineSizeToBoxEdge, bool ownsEdge) {
        if (inset > 0)
            return std::max(0.f, inset - inlineSizeToBoxEdge);
        return ownsEdge ? inset : 0.f;
    };
    startInset = insetForFragment(startInset, fragmentInlineSizes.preceding, hasLogicalStartEdge);
    endInset = insetForFragment(endInset, fragmentInlineSizes.following, hasLogicalEndEdge);
    auto startEdgeOnFragment = hasLogicalStartEdge || startInset > 0;
    auto endEdgeOnFragment = hasLogicalEndEdge || endInset > 0;

    bool isLTR = writingMode.isBidiLTR();

    auto textBox = makeIterator();
    bool ownsLineLeftEdge = !decoratingInlineBox || textBox == decoratingInlineBox->firstLeafBox();
    bool ownsLineRightEdge = !decoratingInlineBox || textBox == decoratingInlineBox->lastLeafBox();
    bool ownsLogicalStart = !markedText.startOffset;
    bool ownsLogicalEnd = markedText.endOffset == m_paintTextRun.length();

    auto visualLeftInset = isLTR ? startInset : endInset;
    auto visualRightInset = isLTR ? endInset : startInset;
    auto leftEdgeOnFragment = isLTR ? startEdgeOnFragment : endEdgeOnFragment;
    auto rightEdgeOnFragment = isLTR ? endEdgeOnFragment : startEdgeOnFragment;
    float leftEdgeMove = leftEdgeOnFragment ? visualLeftInset : 0.f;
    float rightEdgeMove = rightEdgeOnFragment ? -visualRightInset : 0.f;

    // The part of the inset that moves both visual edges the same way is a shift of the whole decoration,
    // so every painted piece gets it and a symmetric inset stays rigid across bidi runs.
    // The rest is the extend/trim overhang, which only the piece reaching that visual edge gets, leaving
    // interior pieces (e.g. a superscript at another baseline) where they are for a pure extend/trim.
    float decorationInlineShift = (leftEdgeOnFragment && rightEdgeOnFragment) ? (leftEdgeMove + rightEdgeMove) / 2.f : 0.f;
    bool reachesVisualLeft = leftEdgeOnFragment && ownsLineLeftEdge && (isLTR ? ownsLogicalStart : ownsLogicalEnd);
    bool reachesVisualRight = rightEdgeOnFragment && ownsLineRightEdge && (isLTR ? ownsLogicalEnd : ownsLogicalStart);
    float leftOverhang = reachesVisualLeft ? leftEdgeMove - decorationInlineShift : 0.f;
    float rightOverhang = reachesVisualRight ? rightEdgeMove - decorationInlineShift : 0.f;

    float adjustedLeft = boxOrigin.x() + decorationInlineShift + leftOverhang;
    float adjustedRight = boxOrigin.x() + width + decorationInlineShift + rightOverhang;
    boxOrigin.setX(adjustedLeft);
    width = std::max(0.f, adjustedRight - adjustedLeft);
    return { boxOrigin, width };
}

void TextBoxPainter::paintBackgroundDecorations(TextDecorationPainter& decorationPainter, const StyledMarkedText& markedText, const FloatRect& textBoxPaintRect)
{
    if (m_isCombinedText)
        m_paintInfo.context().concatCTM(rotation(m_paintRect, RotationDirection::Clockwise));

    auto textRun = m_paintTextRun.subRun(markedText.startOffset, markedText.endOffset - markedText.startOffset);

    auto textBox = makeIterator();
    auto decoratingBoxList = DecoratingBoxList { };
    collectDecoratingBoxesForBackgroundPainting(decoratingBoxList, textBox, textBoxPaintRect, markedText.style.textDecorationStyles);

    for (auto& decoratingBox : decoratingBoxList | std::views::reverse) {
        auto computedTextDecorationType = WebCore::computedTextDecorationType(decoratingBox.style.get(), decoratingBox.textDecorationStyles);
        auto computedBackgroundDecorationGeometry = [&] {
            // text-decoration-thickness is set on the originating box and applies to every line that box paints,
            // so the per-line slots in textDecorationStyles all carry the same value. We pick whichever line
            // we're about to draw just so we have a populated slot to read; the others would do.
            auto& thicknessDecoration = computedTextDecorationType.hasUnderline() ? decoratingBox.textDecorationStyles.underline : computedTextDecorationType.hasOverline() ? decoratingBox.textDecorationStyles.overline : decoratingBox.textDecorationStyles.linethrough;
            auto textDecorationThickness = resolveTextDecorationThicknessForPaintingBox(thicknessDecoration.thickness, decoratingBox.style.get(), m_document->deviceScaleFactor());
            auto underlineOffset = [&] {
                if (!computedTextDecorationType.hasUnderline())
                    return 0.f;
                auto baseOffset = underlineOffsetForTextBoxPainting(*decoratingBox.inlineBox, decoratingBox.style.get(), decoratingBox.textDecorationStyles.underlineOffset);
                auto wavyOffset = decoratingBox.textDecorationStyles.underline.decorationStyle == TextDecorationStyle::Wavy ? wavyOffsetFromDecoration() : 0.f;
                return baseOffset + wavyOffset;
            };
            auto autoTextDecorationThickness = computedAutoTextDecorationThickness(decoratingBox.style.get(), m_document->deviceScaleFactor());
            auto overlineOffset = [&] {
                if (!computedTextDecorationType.hasOverline())
                    return 0.f;
                auto baseOffset = overlineOffsetForTextBoxPainting(*decoratingBox.inlineBox, decoratingBox.style.get());
                baseOffset += (autoTextDecorationThickness - textDecorationThickness);
                auto wavyOffset = decoratingBox.textDecorationStyles.overline.decorationStyle == TextDecorationStyle::Wavy ? wavyOffsetFromDecoration() : 0.f;
                return baseOffset - wavyOffset;
            };

            auto [insetBoxOrigin, insetWidth] = insetAdjustedDecorationLocationAndWidth(decoratingBox, markedText);
            return TextDecorationPainter::BackgroundDecorationGeometry {
                textOriginFromPaintRect(textBoxPaintRect),
                insetBoxOrigin,
                insetWidth,
                textDecorationThickness,
                underlineOffset(),
                overlineOffset(),
                computedLinethroughCenter(decoratingBox.style.get(), textDecorationThickness, autoTextDecorationThickness),
                decoratingBox.style->metricsOfPrimaryFont().ascent() + 2.f,
                wavyStrokeParameters(decoratingBox.style->usedFontSize())
            };
        };

        decorationPainter.paintBackgroundDecorations(m_style, textRun, computedBackgroundDecorationGeometry(), computedTextDecorationType, decoratingBox.textDecorationStyles, m_document->deviceScaleFactor());
    }

    if (m_isCombinedText)
        m_paintInfo.context().concatCTM(rotation(m_paintRect, RotationDirection::Counterclockwise));
}

void TextBoxPainter::collectDecoratingBoxesForForegroundPainting(DecoratingBoxList& decoratingBoxList, const InlineIterator::TextBoxIterator& textBox, const FloatRect& textBoxRect, const TextDecorationPainter::Styles& overrideDecorationStyle)
{
    auto parentInlineBox = textBox->parentInlineBox();
    if (!parentInlineBox) {
        ASSERT_NOT_REACHED();
        return;
    }

    auto textBoxLocation = textBoxRect.location();
    auto decorationWidth = textBoxRect.width();
    if (parentInlineBox->isRootInlineBox()) {
        CheckedRef rootStyle = decoratingBoxStyleForInlineBox(*parentInlineBox, m_isFirstLine);
        decoratingBoxList.append({ parentInlineBox, rootStyle, overrideDecorationStyle, textBoxLocation, decorationWidth });
        // The highlight overlay's decoration layers over the originating box's own decoration rather than replacing it.
        auto rootDecorationStyle = TextDecorationPainter::stylesForRenderer(parentInlineBox->renderer(), rootStyle->textDecorationLineInEffect(), m_isFirstLine);
        if (!rootStyle->textDecorationLineInEffect().isNone() && overrideDecorationStyle != rootDecorationStyle)
            decoratingBoxList.append({ parentInlineBox, rootStyle, rootDecorationStyle, textBoxLocation, decorationWidth });
        return;
    }

    auto appendIfIsDecoratingBoxForForeground = [&] (auto& inlineBox) {
        CheckedRef style = decoratingBoxStyleForInlineBox(*inlineBox, m_isFirstLine);
        auto computedDecorationStyle = TextDecorationPainter::stylesForRenderer(inlineBox->renderer(), style->textDecorationLineInEffect(), m_isFirstLine);
        auto isParentInlineBox = &inlineBox == &parentInlineBox;

        if (!isDecoratingBoxForForeground(*inlineBox, style) && (!isParentInlineBox || overrideDecorationStyle == computedDecorationStyle))
            return;

        // Unlike under/overline, line-through is positioned by linethroughCenter (from the decorating box's
        // font metrics), so we only carry the inter-box vertical offset, not textBoxEdgeAdjustmentForUnderline.
        auto decoratingBoxLocation = textBoxLocation;
        if (&inlineBox->renderer() != &parentInlineBox->renderer()) {
            auto decoratingBoxContentBoxTop = inlineBox->logicalTop() + (!inlineBox->isRootInlineBox() ? inlineBox->renderer().borderAndPaddingBefore() : 0_lu);
            auto parentInlineBoxContentBoxTop = parentInlineBox->logicalTop() + (!parentInlineBox->isRootInlineBox() ? parentInlineBox->renderer().borderAndPaddingBefore() : 0_lu);
            decoratingBoxLocation.moveBy(FloatPoint { 0.f, decoratingBoxContentBoxTop - parentInlineBoxContentBoxTop });
        }

        decoratingBoxList.append({
            inlineBox,
            style,
            isParentInlineBox ? overrideDecorationStyle : computedDecorationStyle,
            decoratingBoxLocation,
            decorationWidth
        });
    };

    appendIfIsDecoratingBoxForForeground(parentInlineBox);
    for (auto ancestorInlineBox = parentInlineBox->parentInlineBox(); ancestorInlineBox; ancestorInlineBox = ancestorInlineBox->parentInlineBox()) {
        appendIfIsDecoratingBoxForForeground(ancestorInlineBox);
        if (ancestorInlineBox->isRootInlineBox())
            break;
    }
}

void TextBoxPainter::paintForegroundDecorations(TextDecorationPainter& decorationPainter, const StyledMarkedText& markedText, const FloatRect& textBoxPaintRect)
{
    auto textBox = makeIterator();
    auto decoratingBoxList = DecoratingBoxList { };
    collectDecoratingBoxesForForegroundPainting(decoratingBoxList, textBox, textBoxPaintRect, markedText.style.textDecorationStyles);

    if (m_isCombinedText)
        m_paintInfo.context().concatCTM(rotation(m_paintRect, RotationDirection::Clockwise));

    auto deviceScaleFactor = m_document->deviceScaleFactor();
    for (auto& decoratingBox : decoratingBoxList | std::views::reverse) {
        if (!WebCore::computedTextDecorationType(decoratingBox.style.get(), decoratingBox.textDecorationStyles).hasLineThrough())
            continue;

        auto textDecorationThickness = resolveTextDecorationThicknessForPaintingBox(decoratingBox.textDecorationStyles.linethrough.thickness, decoratingBox.style.get(), deviceScaleFactor);
        auto linethroughCenter = computedLinethroughCenter(decoratingBox.style.get(), textDecorationThickness, computedAutoTextDecorationThickness(decoratingBox.style.get(), deviceScaleFactor));
        auto [insetBoxOrigin, insetWidth] = insetAdjustedDecorationLocationAndWidth(decoratingBox, markedText);
        decorationPainter.paintForegroundDecorations({ insetBoxOrigin
            , insetWidth
            , textDecorationThickness
            , linethroughCenter
            , wavyStrokeParameters(decoratingBox.style->usedFontSize()) }, decoratingBox.textDecorationStyles);
    }

    if (m_isCombinedText)
        m_paintInfo.context().concatCTM(rotation(m_paintRect, RotationDirection::Counterclockwise));
}

static CornerRadii radiiForUnderline(const CompositionUnderline& underline, unsigned markedTextStartOffset, unsigned markedTextEndOffset)
{
    auto radii = CornerRadii { 0 };

#if HAVE(REDESIGNED_TEXT_CURSOR)
    if (!redesignedTextCursorEnabled())
        return radii;

    if (underline.startOffset >= markedTextStartOffset) {
        radii.setTopLeft({ 1, 1 });
        radii.setBottomLeft({ 1, 1 });
    }

    if (underline.endOffset <= markedTextEndOffset) {
        radii.setTopRight({ 1, 1 });
        radii.setBottomRight({ 1, 1 });
    }
#else
    UNUSED_PARAM(underline);
    UNUSED_PARAM(markedTextStartOffset);
    UNUSED_PARAM(markedTextEndOffset);
#endif

    return radii;
}

#if HAVE(REDESIGNED_TEXT_CURSOR)
enum class TrimSide : bool {
    Left,
    Right,
};

static CornerRadii NODELETE trimRadii(const CornerRadii& radii, TrimSide trimSide)
{
    switch (trimSide) {
    case TrimSide::Left:
        return { { }, radii.topRight(), { }, radii.bottomRight() };
    case TrimSide::Right:
        return { radii.topLeft(), { }, radii.bottomLeft(), { } };
    }
}

enum class TextBoxSnapDirection : uint8_t {
    Left,
    Right,
    Both,
};

static FloatRect snapRectToDevicePixelsInDirection(const FloatRect& rect, float deviceScaleFactor, TextBoxSnapDirection snapDirection)
{
    const auto layoutRect = LayoutRect { rect };
    switch (snapDirection) {
    case TextBoxSnapDirection::Left:
        return snapRectToDevicePixelsWithWritingDirection(layoutRect, deviceScaleFactor, true);
    case TextBoxSnapDirection::Right:
        return snapRectToDevicePixelsWithWritingDirection(layoutRect, deviceScaleFactor, false);
    case TextBoxSnapDirection::Both:
        auto snappedRectLeft = snapRectToDevicePixelsWithWritingDirection(layoutRect, deviceScaleFactor, true);
        return snapRectToDevicePixelsWithWritingDirection(LayoutRect { snappedRectLeft }, deviceScaleFactor, false);
    }
}

enum class TextBoxFragmentLocationWithinLayoutBox : uint8_t { First = 1 << 0, Last = 1 << 1 };
static OptionSet<TextBoxFragmentLocationWithinLayoutBox> NODELETE textBoxFragmentLocationWithinLayoutBox(const InlineIterator::BoxModernPath& textBox)
{
    OptionSet<TextBoxFragmentLocationWithinLayoutBox> location;
    if (textBox.box().isFirstForLayoutBox())
        location.add(TextBoxFragmentLocationWithinLayoutBox::First);
    if (textBox.box().isLastForLayoutBox())
        location.add(TextBoxFragmentLocationWithinLayoutBox::Last);
    return location;
}
#endif

void TextBoxPainter::fillCompositionUnderline(float start, float width, const CompositionUnderline& underline, const CornerRadii& radii, bool hasLiveConversion) const
{
#if HAVE(REDESIGNED_TEXT_CURSOR)
    if (!redesignedTextCursorEnabled())
#endif
    {
        // Thick marked text underlines are 2px thick as long as there is room for the 2px line under the baseline.
        // All other marked text underlines are 1px thick.
        // If there's not enough space the underline will touch or overlap characters.
        int lineThickness = 1;
        int baseline = m_style->metricsOfPrimaryFont().ascent();
        if (underline.thick && m_logicalRect.height() - baseline >= 2)
            lineThickness = 2;

        // We need to have some space between underlines of subsequent clauses, because some input methods do not use different underline styles for those.
        // We make each line shorter, which has a harmless side effect of shortening the first and last clauses, too.
        start += 1;
        width -= 2;

        auto underlineColor = underline.compositionUnderlineColor == CompositionUnderlineColor::TextColor
            ? m_style->visitedDependentTextFillColorApplyingColorFilter()
            : Style::ColorResolver { m_style }.colorResolvingCurrentColorApplyingColorFilter(underline.color);

        auto& context = m_paintInfo.context();
        context.setStrokeColor(underlineColor);
        context.setStrokeThickness(lineThickness);
        context.drawLineForText(FloatRect(m_paintRect.x() + start, m_paintRect.y() + m_logicalRect.height() - lineThickness, width, lineThickness), m_isPrinting);
        return;
    }

#if HAVE(REDESIGNED_TEXT_CURSOR)
    if (!underline.color.isVisible())
        return;

    // Thick marked text underlines are 2px thick as long as there is room for the 2px line under the baseline.
    // All other marked text underlines are 1px thick.
    // If there's not enough space the underline will touch or overlap characters.
    int lineThickness = 1;
    int baseline = m_style->metricsOfPrimaryFont().ascent();
    if (m_logicalRect.height() - baseline >= 2)
        lineThickness = 2;

    auto underlineColor = [this] {
#if PLATFORM(MAC)
        auto cssColorValue = CSSValueAppleSystemControlAccent;
#else
        auto cssColorValue = CSSValueAppleSystemBlue;
#endif
        auto styleColorOptions = m_renderer->styleColorOptions();
        return RenderTheme::singleton().systemColor(cssColorValue, styleColorOptions | StyleColorOptions::UseSystemAppearance);
    }();

    if (!underline.thick && hasLiveConversion)
        underlineColor = underlineColor.colorWithAlpha(0.35);

    auto& context = m_paintInfo.context();
    context.setStrokeColor(underlineColor);
    context.setStrokeThickness(lineThickness);

    auto rect = FloatRect(m_paintRect.x() + start, m_paintRect.y() + m_logicalRect.height() - lineThickness, width, lineThickness);

    if (radii.isZero()) {
        context.drawLineForText(rect, m_isPrinting);
        return;
    }

    // We cannot directly draw rounded edges for every rect, since a single textbox path may be split up over multiple rects.
    // Drawing rounded edges unconditionally could then produce broken underlines between continuous rects.
    // As a mitigation, we consult the textbox path to understand the current rect's position in the textbox path.
    // If we're the only box in the path, then we fallback to unconditionally drawing rounded edges.
    // If not, we flatten out the right, left, or both edges depending on whether we're at the start, end, or middle of a path, respectively.
    auto fragmentLocation = textBoxFragmentLocationWithinLayoutBox(m_textBox);
    auto deviceScaleFactor = m_document->deviceScaleFactor();
    if (fragmentLocation.containsAll({ TextBoxFragmentLocationWithinLayoutBox::First, TextBoxFragmentLocationWithinLayoutBox::Last }))
        context.fillRoundedRect(FloatRoundedRect { rect, radii }, underlineColor);
    else if (fragmentLocation == TextBoxFragmentLocationWithinLayoutBox::First)
        context.fillRoundedRect(FloatRoundedRect { snapRectToDevicePixelsInDirection(rect, deviceScaleFactor, TextBoxSnapDirection::Right), trimRadii(radii, TrimSide::Right) }, underlineColor);
    else if (fragmentLocation == TextBoxFragmentLocationWithinLayoutBox::Last)
        context.fillRoundedRect(FloatRoundedRect { snapRectToDevicePixelsInDirection(rect, deviceScaleFactor, TextBoxSnapDirection::Left), trimRadii(radii, TrimSide::Left) }, underlineColor);
    else {
        ASSERT(fragmentLocation.isEmpty());
        // This text fragment is right in the middle of the box content.
        context.fillRect(snapRectToDevicePixelsInDirection(rect, deviceScaleFactor, TextBoxSnapDirection::Both), underlineColor);
    }
#else
    UNUSED_PARAM(radii);
    UNUSED_PARAM(hasLiveConversion);
#endif
}

void TextBoxPainter::paintCompositionUnderlines()
{        
    Ref protectedFrame = m_renderer->frame();
    auto& underlines = protectedFrame->editor().customCompositionUnderlines();
    auto underlineCount = underlines.size();

    if (!underlineCount)
        return;

    auto hasLiveConversion = false;

    auto markedTextStartOffset = underlines[0].startOffset;
    auto markedTextEndOffset = underlines[0].endOffset;

    for (const auto& underline : underlines) {
        if (underline.thick)
            hasLiveConversion = true;

        if (underline.startOffset < markedTextStartOffset)
            markedTextStartOffset = underline.startOffset;

        if (underline.endOffset > markedTextEndOffset)
            markedTextEndOffset = underline.endOffset;
    }

    for (size_t i = 0; i < underlineCount; ++i) {
        auto& underline = underlines[i];
        if (underline.endOffset <= textBox().start()) {
            // Underline is completely before this run. This might be an underline that sits
            // before the first run we draw, or underlines that were within runs we skipped
            // due to truncation.
            continue;
        }

        if (underline.startOffset >= textBox().end())
            break; // Underline is completely after this run, bail. A later run will paint it.

        auto underlineRadii = radiiForUnderline(underline, markedTextStartOffset, markedTextEndOffset);

        // Underline intersects this run. Paint it.
        paintCompositionUnderline(underline, underlineRadii, hasLiveConversion);

        if (underline.endOffset > textBox().end())
            break; // Underline also runs into the next run. Bail now, no more marker advancement.
    }
}

static inline void NODELETE mirrorRTLSegment(float logicalWidth, TextDirection direction, float& start, float width)
{
    if (direction == TextDirection::LTR)
        return;
    start = logicalWidth - width - start;
}

float TextBoxPainter::textPosition()
{
    // When computing the width of a text run, RenderBlock::computeInlineDirectionPositionsForLine() doesn't include the actual offset
    // from the containing block edge in its measurement. textPosition() should be consistent so the text are rendered in the same width.
    if (!m_logicalRect.x())
        return 0;
    return m_logicalRect.x() - makeIterator()->lineBox()->contentLogicalLeft();
}

void TextBoxPainter::paintCompositionUnderline(const CompositionUnderline& underline, const CornerRadii& radii, bool hasLiveConversion)
{
    float start = 0; // start of line to draw, relative to tx
    float width = m_logicalRect.width(); // how much line to draw
    bool useWholeWidth = true;
    unsigned paintStart = textBox().start();
    unsigned paintEnd = textBox().end();
    if (paintStart <= underline.startOffset) {
        paintStart = underline.startOffset;
        useWholeWidth = false;
        start = m_renderer->width(textBox().start(), paintStart - textBox().start(), textPosition(), m_isFirstLine);
    }
    if (paintEnd != underline.endOffset) {
        paintEnd = std::min(paintEnd, (unsigned)underline.endOffset);
        useWholeWidth = false;
    }
    if (m_selectableRange.truncation) {
        paintEnd = std::min(paintEnd, textBox().start() + *m_selectableRange.truncation);
        useWholeWidth = false;
    }
    if (!useWholeWidth) {
        width = m_renderer->width(paintStart, paintEnd - paintStart, textPosition() + start, m_isFirstLine);
        mirrorRTLSegment(m_logicalRect.width(), textBox().direction(), start, width);
    }

    fillCompositionUnderline(start, width, underline, radii, hasLiveConversion);
}

static void removeMarkersPaintedByTextDecorationPainter(const RenderText& renderer, Vector<MarkedText>& markedTexts)
{
    // SpellingError marked text that is styled via ::spelling-error is removed from being painted here and it is painted as regular text-decoration at TextDecorationPainter,
    // unless its text-decoration-line is spelling-error itself. In the latter case we should paint decoration with our native spelling error markers.
    auto spellingErrorPseudoStyle = renderer.spellingErrorPseudoStyle();
    if (spellingErrorPseudoStyle && !spellingErrorPseudoStyle->textDecorationLineInEffect().isSpellingError()) {
        markedTexts.removeAllMatching([] (auto&& markedText) {
            return markedText.type == MarkedText::Type::SpellingError;
        });
    }

    // GrammarError marked text that is styled via ::grammar-error is removed from being painted here and it is painted as regular text-decoration at TextDecorationPainter
    auto grammarErrorPseudoStyle = renderer.grammarErrorPseudoStyle();
    if (grammarErrorPseudoStyle && !grammarErrorPseudoStyle->textDecorationLineInEffect().isNone()) {
        markedTexts.removeAllMatching([] (auto&& markedText) {
            return markedText.type == MarkedText::Type::GrammarError;
        });
    }
}

static std::optional<MarkedText> NODELETE markedTextForTextDecorationLineSpellingError(const RenderText& renderer)
{
    if (!renderer.style().textDecorationLineInEffect().isSpellingError())
        return std::nullopt;
    return std::make_optional<MarkedText>(0, static_cast<unsigned>(renderer.length()), MarkedText::Type::SpellingError);
}

static std::optional<MarkedText> NODELETE markedTextForTextDecorationLineGrammarError(const RenderText& renderer)
{
    if (!renderer.style().textDecorationLineInEffect().isGrammarError())
        return std::nullopt;
    return std::make_optional<MarkedText>(0, static_cast<unsigned>(renderer.length()), MarkedText::Type::GrammarError);
}

void TextBoxPainter::paintPlatformDocumentMarkers()
{
    if (m_paintInfo.paintBehavior.contains(PaintBehavior::Snapshotting) && !m_paintInfo.paintBehavior.contains(PaintBehavior::IncludeDocumentMarkers))
        return;

    auto markedTexts = MarkedText::collectForDocumentMarkers(m_renderer, m_selectableRange, MarkedText::PaintPhase::Decoration);
    // We want to paint text-decoration-line: spelling-error and grammar-error the same way we natively paint text marked with spelling errors
    auto textDecorationLineSpellingErrorAsMarkedText = markedTextForTextDecorationLineSpellingError(m_renderer);
    auto textDecorationLineGrammarErrorAsMarkedText = markedTextForTextDecorationLineGrammarError(m_renderer);

    if (markedTexts.isEmpty() && !textDecorationLineSpellingErrorAsMarkedText && !textDecorationLineGrammarErrorAsMarkedText)
        return;

    // Defer painting to TextDecorationPainter if needed
    removeMarkersPaintedByTextDecorationPainter(m_renderer, markedTexts);

    auto transparentContentMarkedTexts = MarkedText::collectForDraggedAndTransparentContent(DocumentMarkerType::TransparentContent, m_renderer, m_selectableRange);

    // Ensure the transparent content marked texts go first in the vector, so that they take precedence over
    // the other marked texts when being subdivided so that they do not get painted.
    Vector<MarkedText> allMarkedTexts;
    allMarkedTexts.appendVector(transparentContentMarkedTexts);
    allMarkedTexts.appendVector(MarkedText::collectForDictationStreamingOpacity(m_renderer, m_selectableRange));
    allMarkedTexts.appendVector(markedTexts);
    if (textDecorationLineSpellingErrorAsMarkedText)
        allMarkedTexts.append(*textDecorationLineSpellingErrorAsMarkedText);
    if (textDecorationLineGrammarErrorAsMarkedText)
        allMarkedTexts.append(*textDecorationLineGrammarErrorAsMarkedText);

    for (auto& markedText : MarkedText::subdivide(allMarkedTexts, MarkedText::OverlapStrategy::Frontmost)) {
        switch (markedText.type) {
        case MarkedText::Type::DraggedContent:
        case MarkedText::Type::TransparentContent:
        case MarkedText::Type::DictationStreamingOpacity:
            continue;

        default:
            paintPlatformDocumentMarker(markedText);
            break;
        }
    }
}

#if ENABLE(WRITING_TOOLS)

constexpr Seconds writingToolsAnimationLoop = 10000_ms;
static void drawWritingToolsUnderline(GraphicsContext& context, const FloatRect& rect, IntSize frameSize)
{
    auto radius = rect.height() / 2.0;
    auto minX = rect.x();
    auto maxX = rect.maxX();
    auto minY = rect.y();
    auto maxY = rect.maxY();
    auto midY = std::midpoint(minY, maxY);

    auto frameX = frameSize.width();
    auto frameY = frameSize.height();

    constexpr auto redColor = SRGBA<uint8_t> { 227, 100, 136 };
    constexpr auto yellowColor = SRGBA<uint8_t> { 242, 225, 162 };
    constexpr auto purpleColor = SRGBA<uint8_t> { 154, 109, 209 };

    auto animationProgress = (MonotonicTime::now() % writingToolsAnimationLoop).value() / 10;

    auto xOffset = frameX * fmod(animationProgress + midY / frameY, 1.0);
    constexpr std::array colorList { purpleColor, redColor, yellowColor, redColor, purpleColor, purpleColor, redColor, yellowColor, redColor, purpleColor };

    Ref gradient = Gradient::create(Gradient::LinearData { FloatPoint(0 - xOffset, 0), FloatPoint(frameX * 2 - xOffset, frameY) }, { ColorInterpolationMethod::SRGB { }, AlphaPremultiplication::Unpremultiplied });

    auto colorStop = 0.f;
    auto colorIncrement = 1.0 / colorList.size();
    for (auto color : colorList) {
        gradient->addColorStop({ colorStop, color });
        colorStop += colorIncrement;
    }

    context.save();
    context.setFillGradient(WTF::move(gradient));

    Path path;
    path.moveTo(FloatPoint(minX + radius, maxY));
    path.addArc(FloatPoint(minX + radius, midY), radius, piOverTwoDouble, 3 * piOverTwoDouble, RotationDirection::Clockwise);
    path.addLineTo(FloatPoint(maxX - radius, minY));
    path.addArc(FloatPoint(maxX - radius, midY), radius, 3 * piOverTwoDouble, piOverTwoDouble, RotationDirection::Clockwise);

    context.fillPath(path);
    context.restore();
}

#endif // ENABLE(WRITING_TOOLS)

void TextBoxPainter::paintPlatformDocumentMarker(const MarkedText& markedText)
{
    // Never print document markers (rdar://5327887)
    if (m_document->printing())
        return;

    auto bounds = calculateDocumentMarkerBounds(makeIterator(), markedText);
    bounds.moveBy(m_paintRect.location());

#if ENABLE(WRITING_TOOLS)
#if ENABLE(SIMPLIFIED_SUGGESTION_UNDERLINE)
    constexpr static bool useSimplifiedSuggestionUnderline = true;
#else
    constexpr static bool useSimplifiedSuggestionUnderline = false;
#endif
    if (!useSimplifiedSuggestionUnderline && markedText.type == MarkedText::Type::WritingToolsTextSuggestion) {
        drawWritingToolsUnderline(m_paintInfo.context(), bounds,  protect(m_renderer->frame().view())->size());
        return;
    }
#endif

    auto lineStyleMode = [&] {
        switch (markedText.type) {
        case MarkedText::Type::SpellingError:
            return DocumentMarkerLineStyleMode::Spelling;
        case MarkedText::Type::GrammarError:
            return DocumentMarkerLineStyleMode::Grammar;
        case MarkedText::Type::Correction:
#if ENABLE(WRITING_TOOLS)
        case MarkedText::Type::WritingToolsTextSuggestion:
#endif
            return DocumentMarkerLineStyleMode::AutocorrectionReplacement;
        case MarkedText::Type::DictationAlternatives:
            return DocumentMarkerLineStyleMode::DictationAlternatives;
#if PLATFORM(IOS_FAMILY)
        case MarkedText::Type::DictationPhraseWithAlternatives:
            // FIXME: Rename DocumentMarkerLineStyle::TextCheckingDictationPhraseWithAlternatives and remove the PLATFORM(IOS_FAMILY)-guard.
            return DocumentMarkerLineStyleMode::TextCheckingDictationPhraseWithAlternatives;
#endif
        default:
            ASSERT_NOT_REACHED();
            return DocumentMarkerLineStyleMode::Spelling;
        }
    }();

    auto lineStyleColor = RenderTheme::singleton().documentMarkerLineColor(m_renderer, lineStyleMode);
    if (auto* marker = markedText.marker)
        lineStyleColor = lineStyleColor.colorWithAlphaMultipliedBy(marker->opacity());

    m_paintInfo.context().drawDotsForDocumentMarker(bounds, { lineStyleMode, lineStyleColor });
}

FloatRect TextBoxPainter::computePaintRect(const LayoutPoint& paintOffset)
{
    FloatPoint localPaintOffset(paintOffset);
    if (writingMode().isVertical()) {
        localPaintOffset.move(0, -m_logicalRect.height());
        if (writingMode().isLineOverLeft())
            localPaintOffset.move(m_logicalRect.height(), m_logicalRect.width());
    }

    auto visualRect = textBox().visualRectIgnoringBlockDirection();
    textBox().formattingContextRoot().flipForWritingMode(visualRect);

    auto boxOrigin = visualRect.location();

    boxOrigin.moveBy(localPaintOffset);
    if (writingMode().isVertical()) {
        // This is required by the CTM rotation we do for vertical content.
        boxOrigin.setX(roundToDevicePixel(LayoutUnit { boxOrigin.x() }, m_document->deviceScaleFactor()));
    }
    return { boxOrigin, FloatSize(m_logicalRect.width(), m_logicalRect.height()) };
}

FloatRect calculateDocumentMarkerBounds(const InlineIterator::TextBoxIterator& textBox, const MarkedText& markedText)
{
    auto& font = textBox->fontCascade();
    auto [y, height] = DocumentMarkerController::markerYPositionAndHeightForFont(font);

    // Avoid measuring the text when the entire line box is selected as an optimization.
    if (markedText.startOffset || markedText.endOffset != textBox->selectableRange().clamp(textBox->end())) {
        auto run = textBox->textRun();
        auto selectionRect = LayoutRect { 0_lu, y, 0_lu, height };
        font.adjustSelectionRectForText(textBox->renderer().canUseSimplifiedTextMeasuring().value_or(false), run, selectionRect, markedText.startOffset, markedText.endOffset);
        return selectionRect;
    }

    return FloatRect(0, y, textBox->logicalWidth(), height);
}

bool TextBoxPainter::computeHaveSelection() const
{
    if (m_isPrinting || m_paintInfo.phase == PaintPhase::TextClip)
        return false;

    return m_renderer->view().selection().highlightStateForTextBox(m_renderer, m_selectableRange) != RenderObject::HighlightState::None;
}

const FontCascade& TextBoxPainter::fontCascade() const
{
    if (m_isCombinedText)
        return downcast<RenderCombineText>(m_renderer.get()).textCombineFont();

    return m_style->fontCascade();
}

FloatPoint TextBoxPainter::textOriginFromPaintRect(const FloatRect& paintRect) const
{
    auto ascent = m_style->metricsOfPrimaryFont().ascent();
    if (writingMode().isVertical()) {
        // FIXME: This is required by the CTM rotation logic. We should eventually (re)move it though.
        ascent = roundToDevicePixel(LayoutUnit { ascent }, m_document->deviceScaleFactor());
    }

    auto textOrigin = FloatPoint { paintRect.x(), paintRect.y() + ascent };

    if (m_isCombinedText) {
        if (auto newOrigin = downcast<RenderCombineText>(m_renderer.get()).computeTextOrigin(paintRect))
            textOrigin = newOrigin.value();
    }

    if (writingMode().isHorizontal())
        textOrigin.setY(roundToDevicePixel(LayoutUnit { textOrigin.y() }, m_document->deviceScaleFactor()));
    else
        textOrigin.setX(roundToDevicePixel(LayoutUnit { textOrigin.x() }, m_document->deviceScaleFactor()));

    return textOrigin;
}

bool TextBoxPainter::isInsideShapedContent() const
{
    auto& textContent = textBox().box().text();
    return textContent.isAtShapingBoundaryStart() || textContent.isAtShapingBoundaryEnd() || textContent.isInsideShapingBoundary();
}

}
````

## <a id="text-decoration-info-cc"></a>text_decoration_info.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/paint/text_decoration_info.cc). Path: `third_party/blink/renderer/core/paint/text_decoration_info.cc`. Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Source bytes: 28358; source lines: 726; SHA-256: `85ac6220ca637b806fc81d46fd1b3dbcaecd0b59570adbc3a4f91cdb7cf152d8`.

````cpp
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/paint/text_decoration_info.h"

#include <math.h>

#include <algorithm>

#include "base/feature_list.h"
#include "base/types/optional_util.h"
#include "build/build_config.h"
#include "third_party/blink/public/common/features.h"
#include "third_party/blink/renderer/core/layout/inline/fragment_item.h"
#include "third_party/blink/renderer/core/layout/text_decoration_offset.h"
#include "third_party/blink/renderer/core/paint/inline_paint_context.h"
#include "third_party/blink/renderer/core/paint/text_paint_style.h"
#include "third_party/blink/renderer/core/style/computed_style.h"
#include "third_party/blink/renderer/platform/geometry/length_functions.h"
#include "third_party/blink/renderer/platform/runtime_enabled_features.h"

namespace blink {

namespace {

static ResolvedUnderlinePosition ResolveUnderlinePosition(
    const ComputedStyle& style) {
  const TextUnderlinePosition position = style.GetTextUnderlinePosition();

  // |auto| should resolve to |under| to avoid drawing through glyphs in
  // scripts where it would not be appropriate (e.g., ideographs.)
  // However, this has performance implications. For now, we only work with
  // vertical text.
  if (style.GetFontBaseline() != kCentralBaseline) {
    if (EnumHasFlags(position, TextUnderlinePosition::kUnder)) {
      return ResolvedUnderlinePosition::kUnder;
    }
    if (EnumHasFlags(position, TextUnderlinePosition::kFromFont)) {
      return ResolvedUnderlinePosition::kNearAlphabeticBaselineFromFont;
    }
    return ResolvedUnderlinePosition::kNearAlphabeticBaselineAuto;
  }
  // Compute language-appropriate default underline position.
  // https://drafts.csswg.org/css-text-decor-3/#default-stylesheet
  UScriptCode script = style.GetFontDescription().GetScript();
  if (script == USCRIPT_KATAKANA_OR_HIRAGANA || script == USCRIPT_HANGUL) {
    if (EnumHasFlags(position, TextUnderlinePosition::kLeft)) {
      return ResolvedUnderlinePosition::kUnder;
    }
    return ResolvedUnderlinePosition::kOver;
  }
  if (EnumHasFlags(position, TextUnderlinePosition::kRight)) {
    return ResolvedUnderlinePosition::kOver;
  }
  return ResolvedUnderlinePosition::kUnder;
}

inline bool ShouldUseDecoratingBox(const ComputedStyle& style) {
  // Disable the decorating box for styles not in the tree, because they can't
  // find the decorating box. For example, |HighlightPainter| creates a
  // |kPseudoIdHighlight| pseudo style on the fly.
  const PseudoId pseudo_id = style.StyleType();
  if (IsHighlightPseudoElement(pseudo_id))
    return false;
  return true;
}

LayoutUnit InlineSizeOfItem(const FragmentItem& item) {
  return item.IsHorizontal() ? item.Size().width : item.Size().height;
}

// Returns true if |item| continues the run of fragments decorated by
// |decoration|. Zero-sized fragments (e.g. collapsed spaces and forced line
// breaks inside the decorated run) do not end the run, but non-text leaves
// and fragments with different decorations do.
bool ContinuesDecoratedRun(const FragmentItem* item,
                           wtf_size_t decoration_index,
                           const AppliedTextDecoration& decoration) {
  if (!item || !item->IsText()) {
    return false;
  }
  const AppliedTextDecorationVector& decorations =
      item->Style().AppliedTextDecorations();
  if (decoration_index >= decorations.size()) {
    return false;
  }
  return decorations[decoration_index] == decoration;
}

bool HasSameDecorationAtIndexOnLine(const FragmentItem* item,
                                    wtf_size_t decoration_index,
                                    const AppliedTextDecoration& decoration) {
  if (!item) {
    return false;
  }
  DCHECK(item->IsText());
  DCHECK(!item->IsLineBreak());
  return ContinuesDecoratedRun(item, decoration_index, decoration) &&
         InlineSizeOfItem(*item) > LayoutUnit();
}

float ResolveInsetForFragment(float inset, float preceding_size) {
  if (inset <= 0.0f) {
    return preceding_size == 0.0f ? inset : 0.0f;
  }
  return std::max(inset - preceding_size, 0.0f);
}

bool NeedsFragmentContextForDecoration(
    const AppliedTextDecoration& decoration) {
  const TextDecorationInset& inset = decoration.DecorationInset();
  if (inset.GetStart().IsAuto()) {
    return false;
  }
  return !inset.GetStart().IsZero() || !inset.GetEnd().IsZero();
}

static enum StrokeStyle TextDecorationStyleToStrokeStyle(
    ETextDecorationStyle decoration_style) {
  switch (decoration_style) {
    case ETextDecorationStyle::kSolid:
      return kSolidStroke;
    case ETextDecorationStyle::kDouble:
      return kDoubleStroke;
    case ETextDecorationStyle::kDotted:
      return kDottedStroke;
    case ETextDecorationStyle::kDashed:
      return kDashedStroke;
    case ETextDecorationStyle::kWavy:
      return kWavyStroke;
  }
}

#if !BUILDFLAG(IS_APPLE)
WaveDefinition MakeSpellingGrammarWave(float effective_zoom) {
  const float wavelength = 6 * effective_zoom;
  return {
      .wavelength = wavelength,
      .control_point_distance = 5 * effective_zoom,
      // Offset by a quarter of a wavelength, to get a result closer to
      // Microsoft Word circa 2021.
      .phase = -0.75f * wavelength,
  };
}
#endif

}  // namespace

TextDecorationFragmentContext ComputeTextDecorationFragmentContext(
    const InlineCursor& cursor) {
  CHECK(RuntimeEnabledFeatures::CSSTextDecorationInsetEnabled());
  TextDecorationFragmentContext fragment_context;
  InlineCursor line_cursor = cursor;
  line_cursor.ExpandRootToContainingBlock();
  line_cursor.MoveTo(*cursor.CurrentItem());
  fragment_context.fragment_cursor = &cursor;

  InlineCursor previous_cursor = line_cursor;
  previous_cursor.MoveToPreviousInlineLeafOnLine();
  if (previous_cursor.CurrentItem() &&
      previous_cursor.CurrentItem()->IsText() &&
      !previous_cursor.CurrentItem()->IsLineBreak()) {
    fragment_context.previous_fragment_on_line = previous_cursor.CurrentItem();
  }
  InlineCursor next_cursor = line_cursor;
  next_cursor.MoveToNextInlineLeafOnLine();
  if (next_cursor.CurrentItem() && next_cursor.CurrentItem()->IsText() &&
      !next_cursor.CurrentItem()->IsLineBreak()) {
    fragment_context.next_fragment_on_line = next_cursor.CurrentItem();
  }
  return fragment_context;
}

std::optional<gfx::RectF> ComputeUnderOverDecorationBounds(
    const ComputedStyle& style,
    const UsedFont& font,
    LayoutUnit inline_size) {
  DCHECK(style.HasAppliedTextDecorations());
  if (!font.PrimaryFont()) {
    return std::nullopt;
  }

  std::optional<gfx::RectF> bounds;
  auto unite = [&bounds](const gfx::RectF& rect) {
    if (!bounds) {
      bounds = rect;
      return;
    }
    bounds->UnionEvenIfEmpty(rect);
  };

  TextDecorationInfo decoration_info(
      LineRelativeOffset(LayoutUnit(), LayoutUnit()), inline_size, style, font,
      /*inline_context=*/nullptr, TextDecorationLine::kNone, Color());
  TextDecorationOffset decoration_offset(style);
  for (wtf_size_t i = 0; i < decoration_info.AppliedDecorationCount(); ++i) {
    const ResolvedDecoration decoration =
        decoration_info.ResolveDecorationAt(i);
    if (!decoration.HasFontData()) {
      continue;
    }

    if (decoration.HasUnderline()) {
      unite(DecorationLinePainter::Bounds(
          decoration_info.ComputeUnderlineLineData(decoration,
                                                   decoration_offset)));
    }
    if (decoration.HasOverline()) {
      unite(
          DecorationLinePainter::Bounds(decoration_info.ComputeOverlineLineData(
              decoration, decoration_offset)));
    }
  }
  return bounds;
}

TextDecorationInfo::TextDecorationInfo(
    LineRelativeOffset local_origin,
    LayoutUnit width,
    const ComputedStyle& target_style,
    const UsedFont& target_font,
    const InlinePaintContext* inline_context,
    const TextDecorationLine selection_decoration_line,
    const Color selection_decoration_color,
    const AppliedTextDecoration* decoration_override,
    IsSvgText is_svg_text,
    float svg_resource_scaling_factor,
    TextDecorationFragmentContext fragment_context,
    bool conservative_inset_bounds)
    : target_style_(target_style),
      inline_context_(inline_context),
      target_used_font_(target_font),
      selection_decoration_line_(selection_decoration_line),
      selection_decoration_color_(selection_decoration_color),
      decoration_override_(decoration_override),
      local_origin_(local_origin),
      width_(width),
      target_ascent_(target_font.FloatAscent()),
      svg_resource_scaling_factor_(svg_resource_scaling_factor),
      fragment_context_(fragment_context),
      conservative_inset_bounds_(conservative_inset_bounds),
      // NOTE: The use of is_svg_text here is probably problematic.
      // See LayoutSVGInlineText::ComputeNewScaledFontForStyle() for
      // a workaround that is needed due to that.
      use_decorating_box_(inline_context && !decoration_override_ &&
                          !is_svg_text && ShouldUseDecoratingBox(target_style)),
      is_svg_text_(is_svg_text) {
  for (wtf_size_t i = 0; i < AppliedDecorationCount(); ++i) {
    const auto& decoration = AppliedDecoration(i);
    union_all_lines_ |= decoration.Lines();
    if (!antialias_ && (decoration.Style() == ETextDecorationStyle::kDotted ||
                        decoration.Style() == ETextDecorationStyle::kDashed)) {
      antialias_ = true;
    }
  }
}

// static
bool TextDecorationInfo::NeedsFragmentContextForInset(
    const ComputedStyle& style) {
  CHECK(style.HasAppliedTextDecorations());
  for (const AppliedTextDecoration& decoration :
       style.AppliedTextDecorations()) {
    if (NeedsFragmentContextForDecoration(decoration)) {
      return true;
    }
  }
  return false;
}

TextDecorationInfo::DecoratedRunMetrics
TextDecorationInfo::ComputeDecoratedRunMetrics(
    const ResolvedDecoration& decoration,
    wtf_size_t decoration_index) const {
  LayoutUnit size_before;
  LayoutUnit size_after;
  if (fragment_context_.fragment_cursor &&
      fragment_context_.fragment_cursor->Current()) {
    // Build the line cursor on demand; this only runs for decorated
    // fragments whose insets need run metrics, keeping the per-fragment
    // paint path free of `InlineCursor` copies.
    InlineCursor line_cursor = *fragment_context_.fragment_cursor;
    line_cursor.ExpandRootToContainingBlock();
    line_cursor.MoveTo(*fragment_context_.fragment_cursor->CurrentItem());

    const AppliedTextDecoration& applied = *decoration.applied_text_decoration;
    for (InlineCursor cursor = line_cursor;;) {
      cursor.MoveToPreviousInlineLeaf();
      const FragmentItem* item = cursor.CurrentItem();
      if (!ContinuesDecoratedRun(item, decoration_index, applied)) {
        break;
      }
      size_before += InlineSizeOfItem(*item);
    }
    for (InlineCursor cursor = line_cursor;;) {
      cursor.MoveToNextInlineLeaf();
      const FragmentItem* item = cursor.CurrentItem();
      if (!ContinuesDecoratedRun(item, decoration_index, applied)) {
        break;
      }
      size_after += InlineSizeOfItem(*item);
    }

    // Visual-order traversal can encounter unrelated bidi content between
    // fragments of the same text node. Account for all fragments of the node
    // so wrapped insets are still accumulated in logical order.
    const FragmentItem* current_item = line_cursor.CurrentItem();
    DCHECK(current_item);
    DCHECK(current_item->GetLayoutObject());
    LayoutUnit size_before_in_node;
    LayoutUnit size_after_in_node;
    bool found_current = false;
    InlineCursor cursor = line_cursor;
    cursor.MoveTo(*current_item->GetLayoutObject());
    for (; cursor.Current(); cursor.MoveToNextForSameLayoutObject()) {
      const FragmentItem* item = cursor.CurrentItem();
      if (item == current_item) {
        found_current = true;
      } else if (found_current) {
        size_after_in_node += InlineSizeOfItem(*item);
      } else {
        size_before_in_node += InlineSizeOfItem(*item);
      }
    }
    DCHECK(found_current);
    size_before = std::max(size_before, size_before_in_node);
    size_after = std::max(size_after, size_after_in_node);
  }
  return {.size_before = size_before.ToFloat(),
          .size_after = size_after.ToFloat(),
          .total_size = (size_before + width_ + size_after).ToFloat()};
}

wtf_size_t TextDecorationInfo::AppliedDecorationCount() const {
  if (HasDecorationOverride())
    return 1;
  return target_style_.AppliedTextDecorations().size();
}

const AppliedTextDecoration& TextDecorationInfo::AppliedDecoration(
    wtf_size_t index) const {
  if (HasDecorationOverride())
    return *decoration_override_;
  return target_style_.AppliedTextDecorations()[index];
}

const ResolvedDecoration TextDecorationInfo::ResolveDecorationAt(
    wtf_size_t decoration_index) {
  DCHECK_LT(decoration_index, AppliedDecorationCount());

  ResolvedDecoration decoration(target_used_font_,
                                AppliedDecoration(decoration_index));
  decoration.lines = decoration.applied_text_decoration->Lines();
  decoration.has_underline =
      EnumHasFlags(decoration.lines, TextDecorationLine::kUnderline);
  decoration.has_overline =
      EnumHasFlags(decoration.lines, TextDecorationLine::kOverline);

  // Compute the |ComputedStyle| of the decorating box.
  const ComputedStyle* decorating_box_style;
  const DecoratingBox* decorating_box = nullptr;
  if (use_decorating_box_) {
    DCHECK(inline_context_);
    DCHECK_EQ(inline_context_->DecoratingBoxes().size(),
              AppliedDecorationCount());
    bool disable_decorating_box;
    if (decoration_index >= inline_context_->DecoratingBoxes().size())
        [[unlikely]] {
      disable_decorating_box = true;
    } else {
      decorating_box = &inline_context_->DecoratingBoxes()[decoration_index];
      decorating_box_style = &decorating_box->Style();

      // Disable the decorating box when the baseline is central, because the
      // decorating box doesn't produce the ideal position.
      // https://drafts.csswg.org/css-text-decor-3/#:~:text=text%20is%20not%20aligned%20to%20the%20alphabetic%20baseline
      // TODO(crbug.com/563435074): The vertical flow in alphabetic baseline
      // should use the decorating box. It needs supporting the rotated
      // coordinate system text painters use when painting vertical text.
      disable_decorating_box = !decorating_box_style->IsHorizontalWritingMode();
    }

    if (disable_decorating_box) [[unlikely]] {
      use_decorating_box_ = false;
      decorating_box = nullptr;
      decorating_box_style = &target_style_;
    }
  } else {
    DCHECK(!decorating_box);
    decorating_box_style = &target_style_;
  }
  DCHECK(decorating_box_style);
  if (decorating_box_style != decorating_box_style_) {
    decorating_box_style_ = decorating_box_style;
    original_underline_position_ =
        ResolveUnderlinePosition(*decorating_box_style);

    // text-underline-position may flip underline and overline.
    flip_underline_and_overline_ =
        original_underline_position_ == ResolvedUnderlinePosition::kOver;
  }

  if (flip_underline_and_overline_) [[unlikely]] {
    decoration.underline_position = ResolvedUnderlinePosition::kUnder;
    std::swap(decoration.has_underline, decoration.has_overline);
  } else {
    decoration.underline_position = original_underline_position_;
  }
  decoration.is_flipped_underline_and_overline = flip_underline_and_overline_;

  if (!is_svg_text_ && decorating_box) {
    decoration.used_font = decorating_box->GetUsedFont();
  } else {
    // `target_used_font_` was already copied to decoration.used_font.
  }

  decoration.effective_zoom = decorating_box_style_->EffectiveZoom();
  decoration.offset_from_decorating_box =
      decoration.HasUnderline() && decorating_box
          ? OffsetFromDecoratingBox(*decorating_box)
          : LayoutUnit();
  if (decorating_box) {
    decoration.inline_offset_from_decorating_box =
        InlineOffsetFromDecoratingBox(*decorating_box);
  }

  decoration.resolved_thickness = ComputeThickness(decoration);
  ResolveDecorationInsets(decoration_index, decoration);
  return decoration;
}

DecorationGeometry TextDecorationInfo::ComputeLineData(
    const ResolvedDecoration& decoration,
    TextDecorationLine line,
    float line_offset) const {
  const float double_offset_from_thickness =
      decoration.resolved_thickness + 1.0f;
  float double_offset;
  float wavy_offset;
  switch (line) {
    case TextDecorationLine::kUnderline:
    case TextDecorationLine::kSpellingError:
    case TextDecorationLine::kGrammarError:
      double_offset = double_offset_from_thickness;
      wavy_offset = double_offset_from_thickness;
      break;
    case TextDecorationLine::kOverline:
      double_offset = -double_offset_from_thickness;
      wavy_offset = -double_offset_from_thickness;
      break;
    case TextDecorationLine::kLineThrough:
      // Floor double_offset in order to avoid double-line gap to appear of
      // different size depending on position where the double line is drawn
      // because of rounding downstream in DecorationLinePainter.
      double_offset = floorf(double_offset_from_thickness);
      wavy_offset = 0;
      break;
    case TextDecorationLine::kNone:
    case TextDecorationLine::kBlink:
      NOTREACHED();
  }

  StrokeStyle style;
  std::optional<WaveDefinition> spelling_wave;
  bool antialias = antialias_;
  if (line == TextDecorationLine::kSpellingError ||
      line == TextDecorationLine::kGrammarError) {
#if BUILDFLAG(IS_ANDROID)
    if (base::FeatureList::IsEnabled(features::kAndroidSpellcheckNativeUi)) {
      style = kSolidStroke;
      antialias = true;
      spelling_wave = std::nullopt;
    } else {
      style = kWavyStroke;
      spelling_wave = MakeSpellingGrammarWave(decoration.effective_zoom);
    }
#elif BUILDFLAG(IS_APPLE)
    style = kDottedStroke;
    antialias = true;
#else
    style = kWavyStroke;
    spelling_wave = MakeSpellingGrammarWave(decoration.effective_zoom);
#endif
  } else {
    style = TextDecorationStyleToStrokeStyle(
        decoration.applied_text_decoration->Style());
  }

  gfx::RectF decoration_rect(
      gfx::PointF(local_origin_) +
          gfx::Vector2dF(decoration.line_left_inset, line_offset),
      gfx::SizeF(std::max(0.0f, Width().ToFloat() - decoration.line_left_inset -
                                    decoration.line_right_inset),
                 decoration.resolved_thickness));

  DecorationGeometry geometry =
      DecorationGeometry::Make(style, decoration_rect, double_offset,
                               wavy_offset, base::OptionalToPtr(spelling_wave));
  geometry.antialias = antialias;
  if (geometry.style == kWavyStroke &&
      RuntimeEnabledFeatures::WavyDecorationContinuousPhaseEnabled()) {
    geometry.wavy_pattern_shift =
        decoration.inline_offset_from_decorating_box.ToFloat() +
        decoration.line_left_inset;
  }
  return geometry;
}

void TextDecorationInfo::ResolveDecorationInsets(
    wtf_size_t decoration_index,
    ResolvedDecoration& decoration) const {
  const AppliedTextDecoration& applied_text_decoration =
      *decoration.applied_text_decoration;
  const TextDecorationInset& inset = applied_text_decoration.DecorationInset();
  const bool is_auto = inset.GetStart().IsAuto();
  if (!is_auto && inset.GetStart().IsZero() && inset.GetEnd().IsZero()) {
    return;
  }

  float start_inset = 0.0f;
  float end_inset = 0.0f;
  if (is_auto) {
    // Keep adjacent automatic underlines visually distinct, but avoid
    // disproportionately large trims for very thick decorations.
    const float auto_inset =
        TextDecorationInset::ResolveAutoInset(decoration.resolved_thickness);
    start_inset = auto_inset;
    end_inset = auto_inset;
  } else if (applied_text_decoration.BoxDecorationBreak() ==
             EBoxDecorationBreak::kSlice) {
    const DecoratedRunMetrics run_metrics =
        ComputeDecoratedRunMetrics(decoration, decoration_index);
    const float resolved_start =
        FloatValueForLength(inset.GetStart(), run_metrics.total_size);
    const float resolved_end =
        FloatValueForLength(inset.GetEnd(), run_metrics.total_size);
    start_inset =
        ResolveInsetForFragment(resolved_start, run_metrics.size_before);
    end_inset = ResolveInsetForFragment(resolved_end, run_metrics.size_after);
  } else {
    const bool has_previous_same_decoration = HasSameDecorationAtIndexOnLine(
        fragment_context_.previous_fragment_on_line, decoration_index,
        applied_text_decoration);
    const bool has_next_same_decoration = HasSameDecorationAtIndexOnLine(
        fragment_context_.next_fragment_on_line, decoration_index,
        applied_text_decoration);
    if (has_previous_same_decoration && has_next_same_decoration) {
      return;
    }

    const float percentage_reference = Width().ToFloat();
    if (!has_previous_same_decoration) {
      start_inset = FloatValueForLength(inset.GetStart(), percentage_reference);
    }
    if (!has_next_same_decoration) {
      end_inset = FloatValueForLength(inset.GetEnd(), percentage_reference);
    }
  }

  if (conservative_inset_bounds_ && !is_auto) {
    // For overflow computations, positive inset values could under-estimate
    // bounds on interior fragments where those trims are not applied.
    // Keep only negative (expanding) contributions.
    start_inset = std::min(start_inset, 0.0f);
    end_inset = std::min(end_inset, 0.0f);
  }

  if (TargetStyle().Direction() == TextDirection::kRtl) {
    std::swap(start_inset, end_inset);
  }
  decoration.line_left_inset = start_inset;
  decoration.line_right_inset = end_inset;
}

// Returns the offset of the target text/box (|local_origin_|) from the
// decorating box.
LayoutUnit TextDecorationInfo::OffsetFromDecoratingBox(
    const DecoratingBox& decorating_box) const {
  DCHECK(use_decorating_box_);
  DCHECK(inline_context_);
  // Compute the paint offset of the decorating box. The |local_origin_| is
  // already adjusted to the paint offset.
  const LayoutUnit decorating_box_paint_offset =
      decorating_box.ContentOffsetInContainer().top +
      inline_context_->PaintOffset().top;
  return decorating_box_paint_offset - local_origin_.line_over;
}

// The decorating box is only used in horizontal writing modes (see
// |ResolveDecorationAt|), where line-left equals the physical left.
LayoutUnit TextDecorationInfo::InlineOffsetFromDecoratingBox(
    const DecoratingBox& decorating_box) const {
  DCHECK(use_decorating_box_);
  DCHECK(inline_context_);
  const LayoutUnit decorating_box_paint_offset =
      decorating_box.ContentOffsetInContainer().left +
      inline_context_->PaintOffset().left;
  return local_origin_.line_left - decorating_box_paint_offset;
}

DecorationGeometry TextDecorationInfo::ComputeUnderlineLineData(
    const ResolvedDecoration& decoration,
    const TextDecorationOffset& decoration_offset) const {
  DCHECK(decoration.HasUnderline());
  // Don't apply text-underline-offset to overlines. |line_offset| is zero.
  Length line_offset;
  if (decoration.is_flipped_underline_and_overline) [[unlikely]] {
    line_offset = Length();
  } else {
    line_offset = decoration.applied_text_decoration->UnderlineOffset();
  }
  float paint_underline_offset = decoration_offset.ComputeUnderlineOffset(
      decoration.underline_position, decoration.used_font.ComputedSize(),
      decoration.used_font, line_offset, decoration.resolved_thickness);
  // The offset is for the decorating box. Convert it for the target text/box.
  paint_underline_offset += decoration.offset_from_decorating_box;
  return ComputeLineData(decoration, TextDecorationLine::kUnderline,
                         paint_underline_offset);
}

DecorationGeometry TextDecorationInfo::ComputeOverlineLineData(
    const ResolvedDecoration& decoration,
    const TextDecorationOffset& decoration_offset) const {
  DCHECK(decoration.HasOverline());
  // Don't apply text-underline-offset to overline.
  Length line_offset;
  FontVerticalPositionType position;
  if (decoration.is_flipped_underline_and_overline) [[unlikely]] {
    line_offset = decoration.applied_text_decoration->UnderlineOffset();
    position = FontVerticalPositionType::TopOfEmHeight;
  } else {
    line_offset = Length();
    position = FontVerticalPositionType::TextTop;
  }
  const int paint_overline_offset =
      decoration_offset.ComputeUnderlineOffsetForUnder(
          line_offset, TargetStyle().ComputedFontSize(), decoration.used_font,
          decoration.resolved_thickness, position);
  return ComputeLineData(decoration, TextDecorationLine::kOverline,
                         paint_overline_offset);
}

DecorationGeometry TextDecorationInfo::ComputeLineThroughLineData(
    const ResolvedDecoration& decoration) const {
  DCHECK(decoration.HasLineThrough());
  // For increased line thickness, the line-through decoration needs to grow
  // in both directions from its origin, subtract half the thickness to keep
  // it centered at the same origin.
  const float line_through_offset = 2 * decoration.used_font.FloatAscent() / 3 -
                                    decoration.resolved_thickness / 2;
  return ComputeLineData(decoration, TextDecorationLine::kLineThrough,
                         line_through_offset);
}

DecorationGeometry TextDecorationInfo::ComputeSpellingOrGrammarErrorLineData(
    const ResolvedDecoration& decoration,
    const TextDecorationOffset& decoration_offset) const {
  DCHECK(decoration.HasSpellingOrGrammarError());
  DCHECK(!decoration.HasUnderline());
  DCHECK(!decoration.HasOverline());
  DCHECK(!decoration.HasLineThrough());
  const int paint_underline_offset = decoration_offset.ComputeUnderlineOffset(
      decoration.underline_position, TargetStyle().ComputedFontSize(),
      decoration.used_font, Length(), decoration.resolved_thickness);
  return ComputeLineData(decoration,
                         decoration.HasSpellingError()
                             ? TextDecorationLine::kSpellingError
                             : TextDecorationLine::kGrammarError,
                         paint_underline_offset);
}

Color TextDecorationInfo::LineColor(
    const ResolvedDecoration& decoration) const {
  if (decoration.HasSpellingError()) {
    return LayoutTheme::GetTheme().PlatformSpellingMarkerUnderlineColor();
  }
  if (decoration.HasGrammarError()) {
    return LayoutTheme::GetTheme().PlatformGrammarMarkerUnderlineColor();
  }

  if (highlight_override_)
    return *highlight_override_;

  // Find the matched normal and selection |AppliedTextDecoration|
  // and use the text-decoration-color from selection when it is.
  if (decoration.lines == selection_decoration_line_) {
    return selection_decoration_color_;
  }

  return decoration.applied_text_decoration->GetColor();
}

float TextDecorationInfo::ComputeThickness(
    const ResolvedDecoration& decoration) const {
  if (decoration.HasSpellingOrGrammarError()) {
    // Spelling and grammar error thickness doesn't depend on the font size.
#if BUILDFLAG(IS_ANDROID)
    // TODO(crbug.com/434081396): Verify with UX that this is accurate.
    // This number was derived based on visual inspection of the rendered
    // lines on device.
    // Android uses 2 "display-independent-pixels". See
    // "TextAppearance.Suggestion"
    // https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/res/res/values/styles.xml;l=309
    return (base::FeatureList::IsEnabled(features::kAndroidSpellcheckNativeUi))
               ? 2.5f * decoration.effective_zoom
               : 1.f * decoration.effective_zoom;
#elif BUILDFLAG(IS_APPLE)
    return 2.f * decoration.effective_zoom;
#else
    return 1.f * decoration.effective_zoom;
#endif
  }
  const float thickness =
      decoration.applied_text_decoration->Thickness().Resolve(
          decoration.used_font.UsedSize(), decoration.used_font.PrimaryFont(),
          decoration.used_font.ScalingFactor());
  return std::max(is_svg_text_ ? 0.0f : 1.0f, thickness);
}

void TextDecorationInfo::SetHighlightOverrideColor(
    const std::optional<Color>& color) {
  highlight_override_ = color;
}

}  // namespace blink
````
