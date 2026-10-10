# WebKit container-query CSSOM source witness

This reference retains four complete source files from [WebKit](https://webkit.org/) at immutable commit [`73aa6c89e2cb77c46184a81aec944e4ab99d114d`](https://github.com/WebKit/WebKit/tree/73aa6c89e2cb77c46184a81aec944e4ab99d114d). It supplies bounded implementation evidence for the CSSOM container-query planning contribution. It is not a CSS specification, a whole-engine compatibility promise, or evidence about other WebKit revisions.

Retrieved locally on 2026-10-09 from the existing pinned checkout using `git show 73aa6c89e2cb77c46184a81aec944e4ab99d114d:<path>`. The checkout is read-only for this capture; unrelated working-tree modifications are excluded. Each complete file’s exact bytes are identified below. Added headings, provenance and the evidence map are non-normative; source text is unchanged.

Every file carries its complete Apple copyright notice, two-clause redistribution conditions and disclaimer in the retained source below. Redistribution is permitted by those file-specific notices, which remain authoritative; no license from a different WebKit file is inferred. The capture is documentation material, not newly compiled or linked software.

The [local attribution entry](../licenses/webkit/NOTICE.md) links both exact original license-notice variants.

The relevant source boundaries are listed below; the full files preserve surrounding context and licensing. These observations are source-backed implementation evidence, not normative CSSWG resolutions.

| Source | Evidence boundary | Original file SHA-256 |
| --- | --- | --- |
| [CSSContainerRule.cpp](#csscontainerrule-cpp) | [Getters and CSS rule serialization, lines 62–110](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSContainerRule.cpp#L62-L110) | `3ad862beb77c30a5639d94e25aa612cd2cb00920a49285cb117191f55905b50f` |
| [ContainerQueryParser.cpp](#containerqueryparser-cpp) | [Named query with omitted condition, lines 211–218](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/ContainerQueryParser.cpp#L211-L218) | `1ff6c3d31b4bf81450193837c10cc8a5c014f8ca3bfb713908da17450aba855e` |
| [GenericMediaQuerySerialization.cpp](#genericmediaqueryserialization-cpp) | [Empty-condition serialization, lines 51–64](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/GenericMediaQuerySerialization.cpp#L51-L64) | `7906a6e01157bd14ba9d77f1a5367e322777361b25848aaa116706b5eb09f1d8` |
| [ContainerQuery.cpp](#containerquery-cpp) | [Named-query and query-list serialization, lines 102–122](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/ContainerQuery.cpp#L102-L122) | `b5e5fdabff4dd45105c8fea2761bedfb8ac9e51061fc27dcddd21164589e2477` |

## <a id="csscontainerrule-cpp"></a>CSSContainerRule.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSContainerRule.cpp). Path: `Source/WebCore/css/CSSContainerRule.cpp`. Source bytes: 3630; source lines: 113; SHA-256: `3ad862beb77c30a5639d94e25aa612cd2cb00920a49285cb117191f55905b50f`.

```cpp
/*
 * Copyright (C) 2022 Apple Inc. All rights reserved.
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
#include "CSSContainerRule.h"

#include "CSSMarkup.h"
#include "CSSStyleSheet.h"
#include "GenericMediaQuerySerialization.h"
#include "StyleRule.h"
#include <wtf/text/StringBuilder.h>

namespace WebCore {

CSSContainerRule::CSSContainerRule(StyleRuleContainer& rule, CSSStyleSheet* parent)
    : CSSConditionRule(rule, parent)
{
}

Ref<CSSContainerRule> CSSContainerRule::create(StyleRuleContainer& rule, CSSStyleSheet* parent)
{
    return adoptRef(*new CSSContainerRule(rule, parent));
}

const StyleRuleContainer& CSSContainerRule::styleRuleContainer() const
{
    return downcast<StyleRuleContainer>(groupRule());
}

String CSSContainerRule::cssText() const
{
    StringBuilder builder;
    builder.append("@container "_s);
    builder.append(conditionText());

    appendCSSTextForItems(builder);
    return builder.toString();
}

String CSSContainerRule::conditionText() const
{
    StringBuilder builder;
    CQ::serialize(builder, styleRuleContainer().containerQuery());
    return builder.toString();
}

String CSSContainerRule::containerName() const
{
    const auto& query = styleRuleContainer().containerQuery();
    if (query.size() > 1)
        return ""_s;

    ASSERT(query.size() == 1);
    const auto& onlyCondition = query.first();

    StringBuilder builder;
    if (!onlyCondition.name.isEmpty())
        serializeIdentifier(builder, onlyCondition.name);
    return builder.toString();
}

String CSSContainerRule::containerQuery() const
{
    const auto& query = styleRuleContainer().containerQuery();
    if (query.size() > 1)
        return ""_s;

    ASSERT(query.size() == 1);
    const auto& onlyCondition = query.first();

    StringBuilder builder;
    MQ::serialize(builder, onlyCondition.condition);
    return builder.toString();
}

Vector<CSSContainerRule::Condition> CSSContainerRule::conditions() const
{
    return WTF::map(styleRuleContainer().containerQuery(), [](const auto& condition) {
        StringBuilder nameBuilder, queryBuilder;
        serializeIdentifier(nameBuilder, condition.name);
        MQ::serialize(queryBuilder, condition.condition);

        return Condition {
            .name = nameBuilder.toString(),
            .query = queryBuilder.toString()
        };
    });
}

} // namespace WebCore


```

## <a id="containerqueryparser-cpp"></a>ContainerQueryParser.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/ContainerQueryParser.cpp). Path: `Source/WebCore/css/query/ContainerQueryParser.cpp`. Source bytes: 10564; source lines: 269; SHA-256: `1ff6c3d31b4bf81450193837c10cc8a5c014f8ca3bfb713908da17450aba855e`.

```cpp
/*
 * Copyright (C) 2022 Apple Inc. All rights reserved.
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
#include "ContainerQueryParser.h"

#include "CSSCustomPropertyValue.h"
#include "CSSPropertyParser.h"
#include "CSSPropertyParserConsumer+Ident.h"
#include "CSSPropertyParserConsumer+Primitives.h"
#include "CSSSubstitutionParser.h"
#include "ContainerQueryFeatures.h"
#include <wtf/NeverDestroyed.h>

namespace WebCore {
namespace CQ {

using namespace MQ;

// Parses a <style-range> per css-conditional-5 §6.2:
//   <style-range> = <style-range-value> <mf-comparison> <style-range-value>
//                 | <style-range-value> <mf-lt> <style-range-value> <mf-lt> <style-range-value>
//                 | <style-range-value> <mf-gt> <style-range-value> <mf-gt> <style-range-value>
//   <style-range-value> = <custom-property-name> | <style-feature-value>
// Each operand is captured as an unresolved declaration value and resolved against the query
// container during evaluation; a bare <custom-property-name> is detected there and treated as var().
static std::optional<Feature> consumeStyleRangeFeature(CSSParserTokenRange& range, const CSSParserContext& context)
{
    auto isComparisonDelimiter = [](const CSSParserToken& token) {
        return token.type() == DelimiterToken
            && (token.delimiter() == '<' || token.delimiter() == '>' || token.delimiter() == '=');
    };

    // A <style-feature-value> can't contain comparison tokens, so any comparison delimiter at the
    // top level of the block separates operands. consumeComponentValue() skips over whole blocks
    // (calc(), var(), attr(), ...), keeping their inner tokens out of the split.
    auto consumeOperandRange = [&]() -> CSSParserTokenRange {
        auto start = range;
        while (!range.atEnd() && !isComparisonDelimiter(range.peek()))
            range.consumeComponentValue();
        auto operand = start.rangeUntil(range);
        operand.consumeWhitespace();
        operand.trimTrailingWhitespace();
        return operand;
    };

    struct Operand {
        AtomString customPropertyName;
        Ref<CSSCustomPropertyValue> value;
    };

    auto consumeOperand = [&]() -> std::optional<Operand> {
        auto operandRange = consumeOperandRange();
        if (operandRange.atEnd())
            return { };

        auto customPropertyName = MQ::bareCustomPropertyName(operandRange.span());

        // A non-name operand (literal, calc(), var(), attr(), ...) still needs a valid custom-property
        // name for parsing/resolution; it is never looked up as a real property.
        static MainThreadNeverDestroyed<const AtomString> operandName("--style-range-operand"_s);
        auto name = customPropertyName.isNull() ? operandName.get() : customPropertyName;
        auto value = CSSSubstitutionParser::parseDeclarationValue(name, operandRange, context);
        if (!value)
            return { };

        return Operand { customPropertyName, value.releaseNonNull() };
    };

    auto firstOperand = consumeOperand();
    if (!firstOperand)
        return { };

    auto firstOperator = FeatureParser::consumeRangeComparisonOperator(range);
    if (!firstOperator)
        return { };

    auto secondOperand = consumeOperand();
    if (!secondOperand)
        return { };

    std::optional<ComparisonOperator> secondOperator;
    std::optional<Operand> thirdOperand;
    if (!range.atEnd()) {
        secondOperator = FeatureParser::consumeRangeComparisonOperator(range);
        if (!secondOperator)
            return { };
        thirdOperand = consumeOperand();
        if (!thirdOperand)
            return { };
    }

    if (!range.atEnd())
        return { };

    if (secondOperator && !isConsistentThreeWayComparison(*firstOperator, *secondOperator))
        return { };

    // The center operand is the second one for three-operand ranges, the first otherwise. A bare
    // <custom-property-name> center is stored as the feature name (matching the plain/size model, so
    // serialization and custom-property invalidation keep working); other centers go in `subject`.
    auto setCenter = [](Feature& feature, Operand&& center) {
        if (!center.customPropertyName.isNull())
            feature.name = center.customPropertyName;
        else
            feature.subject = Value { WTF::move(center.value) };
    };

    Feature feature;
    feature.syntax = Syntax::Range;

    if (!thirdOperand) {
        setCenter(feature, WTF::move(*firstOperand));
        feature.rightComparison = Comparison { *firstOperator, Value { WTF::move(secondOperand->value) } };
        return feature;
    }

    feature.leftComparison = Comparison { *firstOperator, Value { WTF::move(firstOperand->value) } };
    setCenter(feature, WTF::move(*secondOperand));
    feature.rightComparison = Comparison { *secondOperator, Value { WTF::move(thirdOperand->value) } };
    return feature;
}

// A style() feature queries a single custom property, in boolean (style(--foo)), plain
// (style(--foo: value)) or <style-range> form. Unlike size features, style features are always
// custom properties, so this is simpler than the generic boolean/plain/range path.
static std::optional<Feature> consumeStyleFeature(CSSParserTokenRange& range, const CSSParserContext& context)
{
    auto rangeForStyleRange = range;

    auto name = FeatureParser::consumeFeatureName(range);
    if (isCustomPropertyName(name)) {
        range.consumeWhitespace();
        if (range.atEnd())
            return Feature { .name = name, .syntax = Syntax::Boolean };

        if (range.peek().type() == ColonToken) {
            range.consumeIncludingWhitespace();
            auto value = FeatureParser::consumeCustomPropertyValue(name, range, context);
            if (!value || !range.atEnd())
                return { };
            return Feature {
                .name = name,
                .syntax = Syntax::Plain,
                .rightComparison = Comparison { ComparisonOperator::Equal, WTF::move(value) }
            };
        }
    }

    range = rangeForStyleRange;
    return consumeStyleRangeFeature(range, context);
}

Vector<const MQ::FeatureSchema*> ContainerQueryParser::featureSchemas()
{
    return Features::allSchemas();
}

std::optional<ContainerQuery> ContainerQueryParser::consumeContainerQuery(CSSParserTokenRange& range, const CSSParserContext& context)
{
    Vector<ContainerCondition> queries;

    do {
        auto query = CQ::ContainerQueryParser::consumeContainerCondition(range, context);
        if (!query)
            return std::nullopt;

        queries.append(WTF::move(*query));
    } while (CSSPropertyParserHelpers::consumeCommaIncludingWhitespace(range));

    if (!range.atEnd())
        return std::nullopt;

    ASSERT(!queries.isEmpty());

    return queries;
}

std::optional<ContainerCondition> ContainerQueryParser::consumeContainerCondition(CSSParserTokenRange& range, const CSSParserContext& context)
{
    auto consumeName = [&] {
        if (range.peek().type() == LeftParenthesisToken || range.peek().type() == FunctionToken)
            return nullAtom();

        return CSSPropertyParserHelpers::consumeEagerlyResolvableCustomIdentRawExcluding(range, { CSSValueNone, CSSValueAnd, CSSValueOr, CSSValueNot }).toAtomString();
    };

    auto name = consumeName();

    auto condition = consumeCondition(range, context);

    if (!condition) {
        if (name.isEmpty())
            return { };
        // it's valid to have a named container query without a condition, like "@container --name {}"
        condition = MQ::Condition { };
    }

    ContainerRequirements requirements;
    auto containsUnknownFeature = ContainsUnknownFeature::No;

    traverseFeatures(*condition, [&](auto& feature) {
        requirements.sizeAxes.add(requiredAxesForFeature(feature));
        if (Features::isScrollStateFeature(feature.schema))
            requirements.scrollState = true;
        if (!feature.schema)
            containsUnknownFeature = ContainsUnknownFeature::Yes;
    });

    return ContainerCondition { name, *condition, requirements, containsUnknownFeature };
}

bool ContainerQueryParser::isValidFunctionId(CSSValueID functionId)
{
    return functionId == CSSValueStyle || functionId == CSSValueScrollState;
}

const MQ::FeatureSchema* ContainerQueryParser::schemaForFeatureName(const AtomString& name, const CSSParserContext& context, State& state)
{
    if (state.inFunctionId == CSSValueStyle)
        return &Features::style();

    if (state.inFunctionId == CSSValueScrollState) {
        if (!context.cssScrollStateContainerQueriesEnabled)
            return nullptr;
        return Features::scrollState(name);
    }

    return GenericMediaQueryParser<ContainerQueryParser>::schemaForFeatureName(name, context, state);
}

std::optional<MQ::Feature> ContainerQueryParser::consumeAndValidateFeature(CSSParserTokenRange& range, const CSSParserContext& context, State& state)
{
    // style() features (boolean, plain and <style-range>) are all custom-property queries.
    if (state.inFunctionId == CSSValueStyle) {
        auto feature = consumeStyleFeature(range, context);
        if (!feature)
            return { };

        feature->schema = &Features::style();
        return feature;
    }

    return GenericMediaQueryParser<ContainerQueryParser>::consumeAndValidateFeature(range, context, state);
}

}
}

```

## <a id="genericmediaqueryserialization-cpp"></a>GenericMediaQuerySerialization.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/GenericMediaQuerySerialization.cpp). Path: `Source/WebCore/css/query/GenericMediaQuerySerialization.cpp`. Source bytes: 5027; source lines: 148; SHA-256: `7906a6e01157bd14ba9d77f1a5367e322777361b25848aaa116706b5eb09f1d8`.

```cpp
/*
 * Copyright (C) 2022 Apple Inc. All rights reserved.
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
#include "GenericMediaQuerySerialization.h"

#include "CSSMarkup.h"
#include "CSSPrimitiveNumericTypes+Serialization.h"
#include "CSSSerializationContext.h"

namespace WebCore {
namespace MQ {

static void serialize(StringBuilder& builder, const QueryInParens& queryInParens)
{
    WTF::switchOn(queryInParens, [&](auto& node) {
        if (node.functionId)
            builder.append(nameString(*node.functionId));
        builder.append('(');
        serialize(builder, node);
        builder.append(')');
    }, [&](const GeneralEnclosed& generalEnclosed) {
        builder.append(generalEnclosed.name);
        builder.append('(');
        builder.append(generalEnclosed.text);
        builder.append(')');
    });
}

void serialize(StringBuilder& builder, const Condition& condition)
{
    if (condition.queries.size() == 1 && condition.logicalOperator == LogicalOperator::Not) {
        builder.append("not "_s);
        serialize(builder, condition.queries.first());
        return;
    }

    for (auto& query : condition.queries) {
        if (&query != &condition.queries.first())
            builder.append(condition.logicalOperator == LogicalOperator::And ? " and "_s : " or "_s);
        serialize(builder, query);
    }
}

static void serialize(StringBuilder& builder, const Value& value)
{
    WTF::switchOn(value,
        [&](const auto& value) {
            CSS::serializationForCSS(builder, CSS::defaultSerializationContext(), value);
        },
        [&](const Ref<CSSCustomPropertyValue>& value) {
            builder.append(protect(value)->cssText(CSS::defaultSerializationContext()));
        }
    );
}

void serialize(StringBuilder& builder, const Feature& feature)
{
    auto serializeRangeComparisonOperator = [&](ComparisonOperator op) {
        builder.append(' ');
        switch (op) {
        case ComparisonOperator::LessThan:
            builder.append('<');
            break;
        case ComparisonOperator::LessThanOrEqual:
            builder.append("<="_s);
            break;
        case ComparisonOperator::Equal:
            builder.append('=');
            break;
        case ComparisonOperator::GreaterThan:
            builder.append('>');
            break;
        case ComparisonOperator::GreaterThanOrEqual:
            builder.append(">="_s);
            break;
        }
        builder.append(' ');
    };

    switch (feature.syntax) {
    case Syntax::Boolean:
        serializeIdentifier(builder, feature.name);
        break;

    case Syntax::Plain:
        switch (feature.rightComparison->op) {
        case MQ::ComparisonOperator::LessThanOrEqual:
            builder.append("max-"_s);
            break;
        case MQ::ComparisonOperator::Equal:
            break;
        case MQ::ComparisonOperator::GreaterThanOrEqual:
            builder.append("min-"_s);
            break;
        case MQ::ComparisonOperator::LessThan:
        case MQ::ComparisonOperator::GreaterThan:
            ASSERT_NOT_REACHED();
            break;
        }
        serializeIdentifier(builder, feature.name);

        builder.append(": "_s);
        serialize(builder, *feature.rightComparison->value);
        break;

    case Syntax::Range:
        if (feature.leftComparison) {
            serialize(builder, *feature.leftComparison->value);
            serializeRangeComparisonOperator(feature.leftComparison->op);
        }

        if (feature.subject)
            serialize(builder, *feature.subject);
        else
            serializeIdentifier(builder, feature.name);

        if (feature.rightComparison) {
            serializeRangeComparisonOperator(feature.rightComparison->op);
            serialize(builder, *feature.rightComparison->value);
        }
        break;
    }
}

}
}

```

## <a id="containerquery-cpp"></a>ContainerQuery.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/query/ContainerQuery.cpp). Path: `Source/WebCore/css/query/ContainerQuery.cpp`. Source bytes: 4854; source lines: 125; SHA-256: `b5e5fdabff4dd45105c8fea2761bedfb8ac9e51061fc27dcddd21164589e2477`.

```cpp
/*
 * Copyright (C) 2022 Apple Inc. All rights reserved.
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
#include "ContainerQuery.h"

#include "CSSCustomPropertyValue.h"
#include "CSSMarkup.h"
#include "CSSPropertyParser.h"
#include "CSSTokenizer.h"
#include "CSSValue.h"
#include "CSSValueKeywords.h"
#include "ContainerQueryFeatures.h"
#include "GenericMediaQueryParser.h"
#include "GenericMediaQuerySerialization.h"
#include <wtf/NeverDestroyed.h>
#include <wtf/text/StringBuilder.h>

namespace WebCore {
namespace CQ {

OptionSet<Axis> requiredAxesForFeature(const MQ::Feature& feature)
{
    if (feature.schema == &Features::width())
        return { Axis::Width };
    if (feature.schema == &Features::height())
        return { Axis::Height };
    if (feature.schema == &Features::inlineSize())
        return { Axis::Inline };
    if (feature.schema == &Features::blockSize())
        return { Axis::Block };
    if (feature.schema == &Features::aspectRatio() || feature.schema == &Features::orientation())
        return { Axis::Inline, Axis::Block };
    return { };
}

void collectCustomPropertyNames(const MQ::Feature& feature, HashSet<AtomString>& names)
{
    auto collectFromCustomPropertyValue = [&](const CSSCustomPropertyValue& value) {
        auto& tokens = value.tokens();

        // A bare <custom-property-name> operand is evaluated as var(--name).
        if (auto name = MQ::bareCustomPropertyName(tokens.span()); !name.isNull())
            names.add(name);

        // var() references, at any nesting depth.
        // FIXME: This only sees literal names. A name that comes from substitution, e.g.
        // var(var(--name)), leaves the indirectly named property uncollected and so unwatched.
        for (size_t i = 0; i < tokens.size(); ++i) {
            if (tokens[i].type() != FunctionToken || tokens[i].functionId() != CSSValueVar)
                continue;
            for (size_t j = i + 1; j < tokens.size(); ++j) {
                if (CSSTokenizer::isWhitespace(tokens[j].type()))
                    continue;
                if (tokens[j].type() == IdentToken && isCustomPropertyName(tokens[j].value()))
                    names.add(tokens[j].value().toAtomString());
                break;
            }
        }
    };

    auto collectFromValue = [&](const std::optional<MQ::Value>& value) {
        if (!value)
            return;
        if (auto* customProperty = std::get_if<Ref<CSSCustomPropertyValue>>(&*value))
            collectFromCustomPropertyValue(customProperty->get());
    };

    // The queried property of a plain/boolean feature, or the bare-name center of a range.
    if (isCustomPropertyName(feature.name))
        names.add(feature.name);

    // Range operands: a non-name center and the comparison bounds may reference further properties.
    collectFromValue(feature.subject);
    if (feature.leftComparison)
        collectFromValue(feature.leftComparison->value);
    if (feature.rightComparison)
        collectFromValue(feature.rightComparison->value);
}

void serialize(StringBuilder& builder, const ContainerCondition& condition)
{
    auto name = condition.name;
    // No-op if empty.
    serializeIdentifier(builder, name);

    StringBuilder conditionString;
    serialize(conditionString, condition.condition);

    // If the name and condition are both non-empty, put a space in-between to separate them.
    if (!name.isEmpty() && !conditionString.isEmpty())
        builder.append(' ');

    // No-op if empty.
    builder.append(conditionString);
}

void serialize(StringBuilder& builder, const ContainerQuery& query)
{
    builder.append(interleave(query, CQ::serialize, ", "_s));
}

}
}

```
