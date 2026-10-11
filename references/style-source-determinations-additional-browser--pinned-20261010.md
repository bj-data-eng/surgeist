# Pinned browser witnesses for additional style determinations

Retrieved on 2026-10-10 America/New_York (UTC-04:00) from immutable local WebKit objects and official Blink Gitiles. Local WebKit extraction capture was 2026-10-11 02:59:01 UTC (rounded acquisition-file timestamps); Blink retrieval was 2026-10-11 02:58:18 UTC. Exact per-file UTC timestamps are retained in the capture manifest. This bounded source witness is implementation evidence, not a standard or whole-engine compatibility claim. Complete original file-level legal notices are retained in each represented source; unlisted source is omitted. No source was compiled or executed. [WebKit legal material](../licenses/webkit/NOTICE.md), [WebKit LGPL-2](../licenses/webkit/LICENSE-LGPL-2.txt) and [Chromium BSD](../licenses/chromium/LICENSE.txt) accompany the relevant files.

The two at-rule classifiers omit color-profile and return invalid for unknown names. These exact pinned classifiers supply no duplicate profile-channel binding precedent; they do not establish behavior of every other engine/version. WebKit groups keyframes by offset/range, easing and composite operation. Its implicit endpoint comments use an older neutral/add formulation, while the selected newer Web Animations clause uses underlying/replace; this witness supports interval/grouping mechanics only, not complete parity. WebKit has additional iteration-accumulation behavior outside the selected CSS replace-composition sampling envelope.

## <a id="webkit-cssatruleid-cpp"></a>Webkit: Source/WebCore/css/parser/CSSAtRuleID.cpp

Immutable source: [Source/WebCore/css/parser/CSSAtRuleID.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSAtRuleID.cpp). Full original SHA-256: `75b617c09371237044d852cb1371523bd7c57227d8958e5f0285cc267792235e` (3575 bytes). Retained scope: lines 1–72: complete file. All unlisted original source is omitted.

complete file:

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

## <a id="webkit-styleresolver-cpp"></a>Webkit: Source/WebCore/style/StyleResolver.cpp

Immutable source: [Source/WebCore/style/StyleResolver.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/StyleResolver.cpp). Full original SHA-256: `709c3a7091fd811f9db823df31a03bd7584b7c8dacbcb1afe8cf52e2a5134465` (40025 bytes). Retained scope: lines 1–29: complete legal header and original leading comments; lines 481–575: complete Vector<Ref<StyleRuleKeyframe>> Resolver::keyframeRulesForName. All unlisted original source is omitted.

complete legal header and original leading comments:

````cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004-2005 Allan Sandfeld Jensen (kde@carewolf.com)
 * Copyright (C) 2006, 2007 Nicholas Shanks (webkit@nickshanks.com)
 * Copyright (C) 2005-2019 Apple Inc. All rights reserved.
 * Copyright (C) 2007 Alexey Proskuryakov <ap@webkit.org>
 * Copyright (C) 2007, 2008 Eric Seidel <eric@webkit.org>
 * Copyright (C) 2008, 2009 Torch Mobile Inc. All rights reserved. (http://www.torchmobile.com/)
 * Copyright (c) 2011, Code Aurora Forum. All rights reserved.
 * Copyright (C) Research In Motion Limited 2011. All rights reserved.
 * Copyright (C) 2012, 2013 Google Inc. All rights reserved.
 * Copyright (C) 2014 Igalia S.L.
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

complete Vector<Ref<StyleRuleKeyframe>> Resolver::keyframeRulesForName:

````cpp
Vector<Ref<StyleRuleKeyframe>> Resolver::keyframeRulesForName(const AtomString& animationName, const TimingFunction* defaultTimingFunction) const
{
    if (animationName.isEmpty())
        return { };

    m_keyframesRuleMap.checkConsistency();

    // Check author map first then check user-agent map.
    auto it = m_keyframesRuleMap.find(animationName);
    if (it == m_keyframesRuleMap.end()) {
        it = userAgentKeyframes().find(animationName);
        if (it == userAgentKeyframes().end())
            return { };
    }

    auto compositeOperationForKeyframe = [](Ref<StyleRuleKeyframe> keyframe) -> CompositeOperation {
        if (auto compositeOperationCSSValue = keyframe->properties().getPropertyCSSValue(CSSPropertyAnimationComposition)) {
            if (auto compositeOperation = toCompositeOperation(*compositeOperationCSSValue))
                return *compositeOperation;
        }
        return Animation::initialCompositeOperation();
    };

    auto timingFunctionForKeyframe = [&](Ref<StyleRuleKeyframe> keyframe) -> Ref<const TimingFunction> {
        if (RefPtr timingFunctionCSSValue = keyframe->properties().getPropertyCSSValue(CSSPropertyAnimationTimingFunction)) {
            if (RefPtr timingFunction = createTimingFunctionDeprecated(timingFunctionCSSValue.releaseNonNull()))
                return timingFunction.releaseNonNull();
        }
        if (defaultTimingFunction)
            return *defaultTimingFunction;
        return CubicBezierTimingFunction::defaultTimingFunction();
    };

    HashSet<Ref<const TimingFunction>> timingFunctions;
    auto uniqueTimingFunctionForKeyframe = [&](Ref<StyleRuleKeyframe> keyframe) -> Ref<const TimingFunction> {
        Ref timingFunction = timingFunctionForKeyframe(keyframe);
        for (auto& existingTimingFunction : timingFunctions) {
            if (arePointingToEqualData(timingFunction.ptr(), existingTimingFunction.ptr()))
                return existingTimingFunction;
        }
        timingFunctions.add(timingFunction);
        return timingFunction;
    };

    Ref keyframesRule = it->value;
    auto* keyframes = &keyframesRule->keyframes();

    using KeyframeUniqueKey = std::tuple<SingleAnimationRangeName, double, Ref<const TimingFunction>, CompositeOperation>;
    auto hasDuplicateKeys = [&]() -> bool {
        HashSet<KeyframeUniqueKey> uniqueKeyframeKeys;
        for (auto& keyframe : *keyframes) {
            auto compositeOperation = compositeOperationForKeyframe(keyframe);
            auto timingFunction = uniqueTimingFunctionForKeyframe(keyframe);
            for (auto key : keyframe->keys()) {
                auto [offsetRangeName, offsetPercentage] = deprecatedStyleRuleKeyframeKeyToStyle(key);
                if (!uniqueKeyframeKeys.add({ offsetRangeName, offsetPercentage.value / 100, timingFunction, compositeOperation }))
                    return true;
            }
        }
        return false;
    }();

    if (!hasDuplicateKeys)
        return *keyframes;

    // Merge keyframes with a similar offset and timing function ensuring that merged keyframes
    // move to the end of the list if the offset is a timeline range.
    Vector<Ref<StyleRuleKeyframe>> deduplicatedKeyframes;
    HashMap<KeyframeUniqueKey, Ref<StyleRuleKeyframe>> keyframesMap;
    for (auto& originalKeyframe : *keyframes) {
        auto compositeOperation = compositeOperationForKeyframe(originalKeyframe);
        auto timingFunction = uniqueTimingFunctionForKeyframe(originalKeyframe);
        for (auto key : originalKeyframe->keys()) {
            auto [offsetRangeName, offsetPercentage] = deprecatedStyleRuleKeyframeKeyToStyle(key);
            KeyframeUniqueKey uniqueKey { offsetRangeName, offsetPercentage.value / 100, timingFunction, compositeOperation };
            if (RefPtr existingStyleRuleKeyframe = keyframesMap.get(uniqueKey)) {
                protect(existingStyleRuleKeyframe->mutableProperties())->mergeAndOverrideOnConflict(originalKeyframe->properties());
                if (existingStyleRuleKeyframe->keys()[0].rangeName == CSSValueNormal)
                    continue;
                deduplicatedKeyframes.removeFirstMatching([&](const auto& styleRuleKeyframe) {
                    return styleRuleKeyframe.ptr() == existingStyleRuleKeyframe;
                });
                deduplicatedKeyframes.append(*existingStyleRuleKeyframe);
            } else {
                auto styleRuleKeyframe = StyleRuleKeyframe::create(MutableStyleProperties::create());
                styleRuleKeyframe->setKey(key);
                protect(styleRuleKeyframe->mutableProperties())->mergeAndOverrideOnConflict(originalKeyframe->properties());
                keyframesMap.set(uniqueKey, styleRuleKeyframe);
                deduplicatedKeyframes.append(styleRuleKeyframe);
            }
        }
    }

    return deduplicatedKeyframes;
}
````

## <a id="webkit-keyframeinterpolation-cpp"></a>Webkit: Source/WebCore/animation/KeyframeInterpolation.cpp

Immutable source: [Source/WebCore/animation/KeyframeInterpolation.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/animation/KeyframeInterpolation.cpp). Full original SHA-256: `2317d23843ba4773296c530950b37d21c7fb930f43c5214c13bb0f8b980fd4ca` (13275 bytes). Retained scope: lines 1–218: complete file. All unlisted original source is omitted.

complete file:

````cpp
/*
 * Copyright (C) 2023-2025 Apple Inc. All rights reserved.
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
#include "KeyframeInterpolation.h"

namespace WebCore {

const KeyframeInterpolation::KeyframeInterval KeyframeInterpolation::interpolationKeyframes(Property property, double iterationProgress, const Keyframe& defaultStartKeyframe, const Keyframe& defaultEndKeyframe) const
{
    // 1. If iteration progress is unresolved abort this procedure.
    // 2. Let target property be the longhand property for which the effect value is to be calculated.
    // 3. If animation type of the target property is not animatable abort this procedure since the effect cannot be applied.
    // 4. Define the neutral value for composition as a value which, when combined with an underlying value using the add composite operation,
    //    produces the underlying value.

    // 5. Let property-specific keyframes be the result of getting the set of computed keyframes for this keyframe effect.
    // 6. Remove any keyframes from property-specific keyframes that do not have a property value for target property.
    unsigned numberOfKeyframesWithZeroOffset = 0;
    unsigned numberOfKeyframesWithOneOffset = 0;

    Vector<const Keyframe*> propertySpecificKeyframes;

    for (size_t i = 0; i < numberOfKeyframes(); ++i) {
        auto& keyframe = keyframeAtIndex(i);
        if (!keyframe.hasResolvedOffset() || !keyframe.animatesProperty(property))
            continue;
        auto offset = keyframe.offset();
        if (!offset)
            numberOfKeyframesWithZeroOffset++;
        if (offset == 1)
            numberOfKeyframesWithOneOffset++;
        propertySpecificKeyframes.append(&keyframe);
    }

    // 7. If property-specific keyframes is empty, return underlying value.
    if (propertySpecificKeyframes.isEmpty())
        return { };

    auto hasImplicitZeroKeyframe = !numberOfKeyframesWithZeroOffset;
    auto hasImplicitOneKeyframe = !numberOfKeyframesWithOneOffset;

    // 8. If there is no keyframe in property-specific keyframes with a computed keyframe offset of 0, create a new keyframe with a computed keyframe
    //    offset of 0, a property value set to the neutral value for composition, and a composite operation of add, and prepend it to the beginning of
    //    property-specific keyframes.
    if (hasImplicitZeroKeyframe) {
        propertySpecificKeyframes.insert(0, &defaultStartKeyframe);
        numberOfKeyframesWithZeroOffset = 1;
    }

    // 9. Similarly, if there is no keyframe in property-specific keyframes with a computed keyframe offset of 1, create a new keyframe with a computed
    //    keyframe offset of 1, a property value set to the neutral value for composition, and a composite operation of add, and append it to the end of
    //    property-specific keyframes.
    if (hasImplicitOneKeyframe) {
        propertySpecificKeyframes.append(&defaultEndKeyframe);
        numberOfKeyframesWithOneOffset = 1;
    }

    // 10. Let interval endpoints be an empty sequence of keyframes.
    Vector<const Keyframe*> intervalEndpoints;

    // 11. Populate interval endpoints by following the steps from the first matching condition from below:
    if (iterationProgress < 0 && numberOfKeyframesWithZeroOffset > 1) {
        // If iteration progress < 0 and there is more than one keyframe in property-specific keyframes with a computed keyframe offset of 0,
        // Add the first keyframe in property-specific keyframes to interval endpoints.
        intervalEndpoints.append(propertySpecificKeyframes.first());
    } else if (iterationProgress >= 1 && numberOfKeyframesWithOneOffset > 1) {
        // If iteration progress ≥ 1 and there is more than one keyframe in property-specific keyframes with a computed keyframe offset of 1,
        // Add the last keyframe in property-specific keyframes to interval endpoints.
        intervalEndpoints.append(propertySpecificKeyframes.last());
    } else {
        // Otherwise,
        // 1. Append to interval endpoints the last keyframe in property-specific keyframes whose computed keyframe offset is less than or equal
        //    to iteration progress and less than 1. If there is no such keyframe (because, for example, the iteration progress is negative),
        //    add the last keyframe whose computed keyframe offset is 0.
        // 2. Append to interval endpoints the next keyframe in property-specific keyframes after the one added in the previous step.
        size_t indexOfLastKeyframeWithZeroOffset = 0;
        int indexOfFirstKeyframeToAddToIntervalEndpoints = -1;
        for (size_t i = 0; i < propertySpecificKeyframes.size(); ++i) {
            auto offset = propertySpecificKeyframes[i]->offset();
            if (!offset)
                indexOfLastKeyframeWithZeroOffset = i;
            if (offset <= iterationProgress && offset < 1)
                indexOfFirstKeyframeToAddToIntervalEndpoints = i;
            else
                break;
        }

        if (indexOfFirstKeyframeToAddToIntervalEndpoints >= 0) {
            intervalEndpoints.append(propertySpecificKeyframes[indexOfFirstKeyframeToAddToIntervalEndpoints]);
            intervalEndpoints.append(propertySpecificKeyframes[indexOfFirstKeyframeToAddToIntervalEndpoints + 1]);
        } else {
            ASSERT(indexOfLastKeyframeWithZeroOffset < propertySpecificKeyframes.size() - 1);
            intervalEndpoints.append(propertySpecificKeyframes[indexOfLastKeyframeWithZeroOffset]);
            intervalEndpoints.append(propertySpecificKeyframes[indexOfLastKeyframeWithZeroOffset + 1]);
        }
    }

    return { intervalEndpoints, hasImplicitZeroKeyframe, hasImplicitOneKeyframe };
}

static double NODELETE transformProgressDuration(const WebAnimationTime& duration)
{
    if (auto time = duration.time())
        return time->seconds();
    return 1.0;
}

void KeyframeInterpolation::interpolateKeyframes(Property property, const KeyframeInterval& interval, double iterationProgress, double currentIteration, const WebAnimationTime& iterationDuration, TimingFunction::Before before, const CompositionCallback& compositionCallback, const AccumulationCallback& accumulationCallback, const InterpolationCallback& interpolationCallback, const RequiresInterpolationForAccumulativeIterationCallback& requiresInterpolationForAccumulativeIterationCallback) const
{
    auto& intervalEndpoints = interval.endpoints;
    if (intervalEndpoints.isEmpty())
        return;

    auto& startKeyframe = *intervalEndpoints.first();
    auto& endKeyframe = *intervalEndpoints.last();

    auto usedInterpolationForAccumulativeIteration = false;

    // 12. For each keyframe in interval endpoints:
    //     If keyframe has a composite operation that is not replace, or keyframe has no composite operation and the
    //     composite operation of this keyframe effect is not replace, then perform the following steps:
    //         1. Let composite operation to use be the composite operation of keyframe, or if it has none, the composite
    //            operation of this keyframe effect.
    //         2. Let value to combine be the property value of target property specified on keyframe.
    //         3. Replace the property value of target property on keyframe with the result of combining underlying value
    //            (Va) and value to combine (Vb) using the procedure for the composite operation to use corresponding to the
    //            target property’s animation type.
    if (isPropertyAdditiveOrCumulative(property)) {
        // Only do this for the 0 keyframe if it was provided explicitly, since otherwise we want to use the "neutral value
        // for composition" which really means we don't want to do anything but rather just use the underlying style which
        // is already set on startKeyframe.
        if (startKeyframe.offset() || !interval.hasImplicitZeroKeyframe) {
            auto startKeyframeCompositeOperation = startKeyframe.compositeOperation().value_or(compositeOperation());
            if (startKeyframeCompositeOperation != CompositeOperation::Replace)
                compositionCallback(startKeyframe, startKeyframeCompositeOperation);
        }

        // Only do this for the 1 keyframe if it was provided explicitly, since otherwise we want to use the "neutral value
        // for composition" which really means we don't want to do anything but rather just use the underlying style which
        // is already set on endKeyframe.
        if (endKeyframe.offset() != 1 || !interval.hasImplicitOneKeyframe) {
            auto endKeyframeCompositeOperation = endKeyframe.compositeOperation().value_or(compositeOperation());
            if (endKeyframeCompositeOperation != CompositeOperation::Replace)
                compositionCallback(endKeyframe, endKeyframeCompositeOperation);
        }

        // If this keyframe effect has an iteration composite operation of accumulate,
        if (iterationCompositeOperation() == IterationCompositeOperation::Accumulate && currentIteration && requiresInterpolationForAccumulativeIterationCallback()) {
            usedInterpolationForAccumulativeIteration = true;
            // apply the following step current iteration times:
            for (auto i = 0; i < currentIteration; ++i) {
                // replace the property value of target property on keyframe with the result of combining the
                // property value on the final keyframe in property-specific keyframes (Va) with the property
                // value on keyframe (Vb) using the accumulation procedure defined for target property.
                if (!startKeyframe.offset() && !interval.hasImplicitZeroKeyframe)
                    accumulationCallback(startKeyframe);
                if (endKeyframe.offset() == 1 && !interval.hasImplicitOneKeyframe)
                    accumulationCallback(endKeyframe);
            }
        }
    }

    // 13. If there is only one keyframe in interval endpoints return the property value of target property on that keyframe.
    if (intervalEndpoints.size() == 1) {
        interpolationCallback(0, 1, IterationCompositeOperation::Replace);
        return;
    }

    // 14. Let start offset be the computed keyframe offset of the first keyframe in interval endpoints.
    auto startOffset = startKeyframe.offset();

    // 15. Let end offset be the computed keyframe offset of last keyframe in interval endpoints.
    auto endOffset = endKeyframe.offset();

    // 16. Let interval distance be the result of evaluating (iteration progress - start offset) / (end offset - start offset).
    auto intervalDistance = (iterationProgress - startOffset) / (endOffset - startOffset);

    // 17. Let transformed distance be the result of evaluating the timing function associated with the first keyframe in interval endpoints
    //     passing interval distance as the input progress.
    auto transformedDistance = intervalDistance;
    if (!iterationDuration.isZero()) {
        auto rangeDuration = (endOffset - startOffset) * transformProgressDuration(iterationDuration);
        if (RefPtr timingFunction = timingFunctionForKeyframe(startKeyframe))
            transformedDistance = timingFunction->transformProgress(intervalDistance, rangeDuration, before);
    }

    // 18. Return the result of applying the interpolation procedure defined by the animation type of the target property, to the values of the target
    //     property specified on the two keyframes in interval endpoints taking the first such value as Vstart and the second as Vend and using transformed
    //     distance as the interpolation parameter p.
    auto iterationCompositeOperation = usedInterpolationForAccumulativeIteration ? IterationCompositeOperation::Replace : this->iterationCompositeOperation();
    currentIteration = usedInterpolationForAccumulativeIteration ? 0 : currentIteration;
    interpolationCallback(transformedDistance, currentIteration, iterationCompositeOperation);
}

} // namespace WebCore
````

## <a id="blink-css-at-rule-id-cc"></a>Blink: third_party/blink/renderer/core/css/parser/css_at_rule_id.cc

Immutable source: [third_party/blink/renderer/core/css/parser/css_at_rule_id.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_at_rule_id.cc). Full original SHA-256: `a829fe085e381f3120ef1e41ea25c27af5354d5eb3d328ac2fb6f9f0c4bdab88` (8146 bytes). Retained scope: lines 1–179: complete file. All unlisted original source is omitted.

complete file:

````cpp
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/css/parser/css_at_rule_id.h"

#include <algorithm>
#include <iterator>
#include <optional>
#include <string_view>

#include "third_party/blink/renderer/core/css/parser/css_parser_context.h"
#include "third_party/blink/renderer/core/frame/web_feature.h"
#include "third_party/blink/renderer/platform/instrumentation/use_counter.h"
#include "third_party/blink/renderer/platform/runtime_enabled_features.h"

namespace blink {

namespace {

// Metadata for at-rules. Sorted by name for binary search.
struct AtRuleEntry {
  const char* name;
  CSSAtRuleID id;
  WebFeature feature;
};

// clang-format off
constexpr AtRuleEntry kAtRuleEntries[] = {
    {"-webkit-keyframes", CSSAtRuleID::kCSSAtRuleWebkitKeyframes, WebFeature::kCSSAtRuleWebkitKeyframes},
    {"annotation", CSSAtRuleID::kCSSAtRuleAnnotation, WebFeature::kCSSAtRuleAnnotation},
    {"bottom-center", CSSAtRuleID::kCSSAtRuleBottomCenter, WebFeature::kCSSAtRulePageMargin},
    {"bottom-left", CSSAtRuleID::kCSSAtRuleBottomLeft, WebFeature::kCSSAtRulePageMargin},
    {"bottom-left-corner", CSSAtRuleID::kCSSAtRuleBottomLeftCorner, WebFeature::kCSSAtRulePageMargin},
    {"bottom-right", CSSAtRuleID::kCSSAtRuleBottomRight, WebFeature::kCSSAtRulePageMargin},
    {"bottom-right-corner", CSSAtRuleID::kCSSAtRuleBottomRightCorner, WebFeature::kCSSAtRulePageMargin},
    {"character-variant", CSSAtRuleID::kCSSAtRuleCharacterVariant, WebFeature::kCSSAtRuleCharacterVariant},
    {"charset", CSSAtRuleID::kCSSAtRuleCharset, WebFeature::kCSSAtRuleCharset},
    {"container", CSSAtRuleID::kCSSAtRuleContainer, WebFeature::kCSSAtRuleContainer},
    {"counter-style", CSSAtRuleID::kCSSAtRuleCounterStyle, WebFeature::kCSSAtRuleCounterStyle},
    {"font-face", CSSAtRuleID::kCSSAtRuleFontFace, WebFeature::kCSSAtRuleFontFace},
    {"font-feature-values", CSSAtRuleID::kCSSAtRuleFontFeatureValues, WebFeature::kCSSAtRuleFontFeatureValues},
    {"font-palette-values", CSSAtRuleID::kCSSAtRuleFontPaletteValues, WebFeature::kCSSAtRuleFontPaletteValues},
    {"import", CSSAtRuleID::kCSSAtRuleImport, WebFeature::kCSSAtRuleImport},
    {"keyframes", CSSAtRuleID::kCSSAtRuleKeyframes, WebFeature::kCSSAtRuleKeyframes},
    {"layer", CSSAtRuleID::kCSSAtRuleLayer, WebFeature::kCSSCascadeLayers},
    {"left-bottom", CSSAtRuleID::kCSSAtRuleLeftBottom, WebFeature::kCSSAtRulePageMargin},
    {"left-middle", CSSAtRuleID::kCSSAtRuleLeftMiddle, WebFeature::kCSSAtRulePageMargin},
    {"left-top", CSSAtRuleID::kCSSAtRuleLeftTop, WebFeature::kCSSAtRulePageMargin},
    {"media", CSSAtRuleID::kCSSAtRuleMedia, WebFeature::kCSSAtRuleMedia},
    {"namespace", CSSAtRuleID::kCSSAtRuleNamespace, WebFeature::kCSSAtRuleNamespace},
    {"ornaments", CSSAtRuleID::kCSSAtRuleOrnaments, WebFeature::kCSSAtRuleOrnaments},
    {"page", CSSAtRuleID::kCSSAtRulePage, WebFeature::kCSSAtRulePage},
    {"position-try", CSSAtRuleID::kCSSAtRulePositionTry, WebFeature::kCSSAnchorPositioning},
    {"property", CSSAtRuleID::kCSSAtRuleProperty, WebFeature::kCSSAtRuleProperty},
    {"right-bottom", CSSAtRuleID::kCSSAtRuleRightBottom, WebFeature::kCSSAtRulePageMargin},
    {"right-middle", CSSAtRuleID::kCSSAtRuleRightMiddle, WebFeature::kCSSAtRulePageMargin},
    {"right-top", CSSAtRuleID::kCSSAtRuleRightTop, WebFeature::kCSSAtRulePageMargin},
    {"scope", CSSAtRuleID::kCSSAtRuleScope, WebFeature::kCSSAtRuleScope},
    {"starting-style", CSSAtRuleID::kCSSAtRuleStartingStyle, WebFeature::kCSSAtRuleStartingStyle},
    {"styleset", CSSAtRuleID::kCSSAtRuleStyleset, WebFeature::kCSSAtRuleStylistic},
    {"stylistic", CSSAtRuleID::kCSSAtRuleStylistic, WebFeature::kCSSAtRuleStylistic},
    {"supports", CSSAtRuleID::kCSSAtRuleSupports, WebFeature::kCSSAtRuleSupports},
    {"swash", CSSAtRuleID::kCSSAtRuleSwash, WebFeature::kCSSAtRuleSwash},
    {"top-center", CSSAtRuleID::kCSSAtRuleTopCenter, WebFeature::kCSSAtRulePageMargin},
    {"top-left", CSSAtRuleID::kCSSAtRuleTopLeft, WebFeature::kCSSAtRulePageMargin},
    {"top-left-corner", CSSAtRuleID::kCSSAtRuleTopLeftCorner, WebFeature::kCSSAtRulePageMargin},
    {"top-right", CSSAtRuleID::kCSSAtRuleTopRight, WebFeature::kCSSAtRulePageMargin},
    {"top-right-corner", CSSAtRuleID::kCSSAtRuleTopRightCorner, WebFeature::kCSSAtRulePageMargin},
    {"view-transition", CSSAtRuleID::kCSSAtRuleViewTransition, WebFeature::kCSSAtRuleViewTransition},
};
// clang-format on

// At-rules gated behind runtime flags.
// Sorted by name for consistency with kAtRuleEntries.
struct FlaggedAtRuleEntry {
  const char* name;
  CSSAtRuleID id;
  WebFeature feature;
  bool (*is_enabled)();
};

// clang-format off
constexpr FlaggedAtRuleEntry kFlaggedAtRuleEntries[] = {
    {"apply", CSSAtRuleID::kCSSAtRuleApplyMixin, WebFeature::kCSSMixins,
     &RuntimeEnabledFeatures::CSSMixinsEnabled},
    {"contents", CSSAtRuleID::kCSSAtRuleContents, WebFeature::kCSSMixins,
     &RuntimeEnabledFeatures::CSSMixinsEnabled},
    {"custom-media", CSSAtRuleID::kCSSAtRuleCustomMedia, WebFeature::kCSSCustomMedia,
     &RuntimeEnabledFeatures::CSSCustomMediaEnabled},
    {"function", CSSAtRuleID::kCSSAtRuleFunction, WebFeature::kCSSFunctions,
     &RuntimeEnabledFeatures::CSSFunctionsEnabled},
    {"location", CSSAtRuleID::kCSSAtRuleLocation, WebFeature::kCSSAtRuleRoute,
     &RuntimeEnabledFeatures::RouteMatchingEnabled},
    {"mixin", CSSAtRuleID::kCSSAtRuleMixin, WebFeature::kCSSMixins,
     &RuntimeEnabledFeatures::CSSMixinsEnabled},
    {"navigation", CSSAtRuleID::kCSSAtRuleNavigation, WebFeature::kCSSAtRuleRoute,
     &RuntimeEnabledFeatures::RouteMatchingEnabled},
    {"private", CSSAtRuleID::kCSSAtRulePrivate, WebFeature::kCSSAtPrivate,
     &RuntimeEnabledFeatures::CSSPrivateEnabled},
    {"result", CSSAtRuleID::kCSSAtRuleResult, WebFeature::kCSSMixins,
     &RuntimeEnabledFeatures::CSSMixinsEnabled},
};
// clang-format on

// Compile-time validation that tables are sorted for binary search.
constexpr auto AtRuleNameProjection = [](const auto& entry) {
  return std::string_view(entry.name);
};

static_assert(std::ranges::is_sorted(kAtRuleEntries, {}, AtRuleNameProjection),
              "kAtRuleEntries must be sorted by name for binary search");
static_assert(std::ranges::is_sorted(kFlaggedAtRuleEntries,
                                     {},
                                     AtRuleNameProjection),
              "kFlaggedAtRuleEntries must be sorted by name");

}  // namespace

CSSAtRuleID CssAtRuleID(StringView name) {
  // Binary search the main table.
  const auto* it = std::lower_bound(
      std::begin(kAtRuleEntries), std::end(kAtRuleEntries), name,
      [](const AtRuleEntry& entry, const StringView& target) {
        return CodeUnitCompareIgnoringAsciiCase(StringView(entry.name),
                                                target) < 0;
      });
  if (it != std::end(kAtRuleEntries) &&
      EqualIgnoringAsciiCase(name, it->name)) {
    return it->id;
  }
  // Linear search the smaller flagged entries table.
  for (const auto& entry : kFlaggedAtRuleEntries) {
    if (entry.is_enabled() && EqualIgnoringAsciiCase(name, entry.name)) {
      return entry.id;
    }
  }
  return CSSAtRuleID::kCSSAtRuleInvalid;
}

StringView CssAtRuleIDToString(CSSAtRuleID id) {
  for (const auto& entry : kAtRuleEntries) {
    if (entry.id == id) {
      return entry.name;
    }
  }
  for (const auto& entry : kFlaggedAtRuleEntries) {
    if (entry.id == id) {
      return entry.name;
    }
  }
  NOTREACHED();
}

namespace {

std::optional<WebFeature> AtRuleFeature(CSSAtRuleID rule_id) {
  for (const auto& entry : kAtRuleEntries) {
    if (entry.id == rule_id) {
      return entry.feature;
    }
  }
  for (const auto& entry : kFlaggedAtRuleEntries) {
    if (entry.id == rule_id) {
      return entry.feature;
    }
  }
  NOTREACHED();
}

}  // namespace

void CountAtRule(const CSSParserContext* context, CSSAtRuleID rule_id) {
  if (std::optional<WebFeature> feature = AtRuleFeature(rule_id)) {
    context->Count(*feature);
  }
}

}  // namespace blink
````
