# Pinned browser leader support witnesses

This bounded reference retains the complete used content-parser functions and the explicit WebKit model support comment at immutable source revisions. It establishes that these inspected content front doors do not implement `leader()`. It supplies no leader layout algorithm and no resolution of CSS Generated Content draft questions. Complete original file-specific license headers precede every excerpt; no executable source is ingested.

WebKit sources carry their original BSD redistribution notices below. Blink source uses the [Chromium BSD license](../licenses/chromium/LICENSE.txt); its copyright header is also retained.

## webkit-content-model

Original: [immutable complete file](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/values/content/CSSContent.h). Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Path: `Source/WebCore/css/values/content/CSSContent.h`. Complete-file bytes: 9231; SHA-256: `0ac6d06f1971f77654fbf792c5f0720ab0ebb257be426681ef684c2aa7a6acb0`. Retained source line ranges: 35–52.

Excerpt 35–52, with complete original license header:

```cpp
/*
 * Copyright (C) 2026 Samuel Weinig <sam@webkit.org>
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

// <leader()>            = leader( <leader-type> )
// <target>              = <target-counter()> | <target-counters()> | <target-text()>
// <counter>             = <counter()> | <counters()>
// <quote>               = open-quote | close-quote | no-open-quote | no-close-quote
// <content-replacement> = <image>
// <content-list>        = [ <string> | contents | <image> | <counter> | <quote> | <target> | <leader()> ]+
// <alt-content>         = [ <string> | <counter> ]+

// <'content'>           = normal | none | [ <content-replacement> | <content-list> ] [/ <alt-content> ]?
// https://www.w3.org/TR/css-content-3/#propdef-content

// MISSING from <content-list>:
//    contents
//    <leader()>
//    <target>

// MISSING from <alt-content>:
//   <counter>
```

## webkit-content-parser

Original: [immutable complete file](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/parser/CSSPropertyParserConsumer+Content.cpp). Revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Path: `Source/WebCore/css/parser/CSSPropertyParserConsumer+Content.cpp`. Complete-file bytes: 11634; SHA-256: `f6eb551cb16df9379a7a37915ed07eb9d423dca5ea38a1b1b87125ad091afd30`. Retained source line ranges: 202–335.

Excerpt 202–335, with complete original license header:

```cpp
/*
 * Copyright (C) 2016-2023 Apple Inc. All rights reserved.
 * Copyright (C) 2024 Samuel Weinig <sam@webkit.org>
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

static std::optional<CSS::Content> consumeUnresolvedContent(CSSParserTokenRange& range, CSS::PropertyParserState& state)
{
    // Standard says this should be:
    //
    // <'content'> = normal | none | [ <content-replacement> | <content-list> ] [/ [ <string> | <counter> | <attr()> ]+ ]?
    // https://drafts.csswg.org/css-content-3/#propdef-content

    CSSParserTokenRangeGuard guard { range };

    switch (range.peek().id()) {
    case CSSValueNone:
        range.consumeIncludingWhitespace();
        guard.commit();
        return CSS::Content { CSS::Keyword::None { } };
    case CSSValueNormal:
        range.consumeIncludingWhitespace();
        guard.commit();
        return CSS::Content { CSS::Keyword::Normal { } };
    default:
        break;
    }

    auto consumeVisibleContentListItem = [&] -> std::optional<CSS::Content::VisibleContentListItem> {
        if (auto string = consumeUnresolvedString(range))
            return CSS::Content::Text { WTF::move(*string) };

        if (auto image = consumeImage(range, state))
            return CSS::Content::Image { CSS::ImageWrapper { image.releaseNonNull() } };

        if (auto quote = consumeSpecificUnresolvedIdent<CSS::Content::Quote::Type>(range))
            return CSS::Content::Quote { WTF::move(*quote) };

        // Restricted to the UA stylesheet until they get an official name.
        // See https://github.com/w3c/csswg-drafts/issues/14317
        if (isUASheetBehavior(state.context.mode)) {
            if (auto glyph = consumeSpecificUnresolvedIdent<CSS::Content::Glyph::Type>(range))
                return CSS::Content::Glyph { WTF::move(*glyph) };
        }

        switch (range.peek().functionId()) {
        case CSSValueAttr:
            return consumeUnresolvedContentLegacyAttrFunction(consumeFunction(range), state);
        case CSSValueCounter:
            return consumeUnresolvedContentCounterFunction(consumeFunction(range), state);
        case CSSValueCounters:
            return consumeUnresolvedContentCountersFunction(consumeFunction(range), state);
        default:
            break;
        }

        return std::nullopt;
    };

    auto consumeVisibleContentList = [&] -> std::optional<CSS::Content::VisibleContentList> {
        CSS::Content::VisibleContentList result;

        bool shouldEnd = false;
        do {
            auto item = consumeVisibleContentListItem();
            if (!item)
                return std::nullopt;

            result.value.append(WTF::move(*item));

            // Visible content parsing ends at '/' or end of range.
            if (!range.atEnd()) {
                auto value = range.peek();
                if (value.type() == DelimiterToken && value.delimiter() == '/')
                    shouldEnd = true;
            }
            shouldEnd = shouldEnd || range.atEnd();
        } while (!shouldEnd);

        return result;
    };

    auto consumeAltContentListItem = [&] -> std::optional<CSS::Content::AltContentListItem> {
        if (auto string = consumeUnresolvedString(range))
            return CSS::Content::Text { WTF::move(*string) };

        // FIXME: <alt-content> should support <counter> as well.
        switch (range.peek().functionId()) {
        case CSSValueAttr:
            return consumeUnresolvedContentLegacyAttrFunction(consumeFunction(range), state);
        default:
            break;
        }

        return std::nullopt;
    };

    auto consumeAltContentList = [&] -> std::optional<CSS::Content::AltContentList> {
        CSS::Content::AltContentList result;

        do {
            auto item = consumeAltContentListItem();
            if (!item)
                return std::nullopt;

            result.value.append(WTF::move(*item));
        } while (!range.atEnd());

        return result;
    };

    auto visibleContent = consumeVisibleContentList();
    if (!visibleContent)
        return std::nullopt;

    if (consumeSlashIncludingWhitespace(range)) {
        auto altContent = consumeAltContentList();
        if (!altContent)
            return std::nullopt;

        guard.commit();
        return CSS::Content::Data {
            WTF::move(*visibleContent),
            WTF::move(altContent)
        };
    }

    guard.commit();
    return CSS::Content::Data {
        WTF::move(*visibleContent),
        std::nullopt
    };
}

RefPtr<CSSValue> consumeContent(CSSParserTokenRange& range, CSS::PropertyParserState& state)
{
    if (auto unresolved = consumeUnresolvedContent(range, state))
        return CSSContentValue::create(WTF::move(*unresolved));
    return nullptr;
}
```

## blink-content-parser

Original: [immutable complete file](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/properties/longhands/longhands_custom.cc). Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Path: `third_party/blink/renderer/core/css/properties/longhands/longhands_custom.cc`. Complete-file bytes: 470541; SHA-256: `6da988becff26135302be3d8f9f90ebdf8f7d93cb768256192a99d42aae20b85`. Retained source line ranges: 3131–3211, 3215–3220.

Excerpt 3131–3211, with complete original license header:

```cpp
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

const CSSValue* ParseContentValue(CSSParserTokenStream& stream,
                                  const CSSParserContext& context,
                                  CSSParserLocalContext& local_context) {
  if (css_parsing_utils::IdentMatches<CSSValueID::kNone, CSSValueID::kNormal>(
          stream.Peek().Id())) {
    return css_parsing_utils::ConsumeIdent(stream);
  }

  CSSValueList* values = CSSValueList::CreateSpaceSeparated();
  CSSValueList* outer_list = CSSValueList::CreateSlashSeparated();
  bool alt_text_present = false;
  do {
    CSSParserSavePoint savepoint(stream);
    CSSValue* parsed_value =
        css_parsing_utils::ConsumeImage(stream, context, local_context);
    if (!parsed_value) {
      parsed_value = css_parsing_utils::ConsumeIdent<
          CSSValueID::kOpenQuote, CSSValueID::kCloseQuote,
          CSSValueID::kNoOpenQuote, CSSValueID::kNoCloseQuote>(stream);
    }
    if (!parsed_value) {
      parsed_value = css_parsing_utils::ConsumeString(stream);
    }
    if (!parsed_value) {
      if (stream.Peek().FunctionId() == CSSValueID::kCounter) {
        parsed_value =
            ConsumeCounterContent(stream, context, local_context, false);
      } else if (stream.Peek().FunctionId() == CSSValueID::kCounters) {
        parsed_value =
            ConsumeCounterContent(stream, context, local_context, true);
      }
    }
    if (!parsed_value) {
      if (css_parsing_utils::ConsumeSlashIncludingWhitespace(stream)) {
        // No values were parsed before the slash, so nothing to apply the
        // alternative text to.
        if (!values->length()) {
          return nullptr;
        }
        alt_text_present = true;
      } else {
        break;
      }
    } else {
      values->Append(*parsed_value);
    }
    savepoint.Release();
  } while (!stream.AtEnd() && !alt_text_present);
  if (!values->length()) {
    return nullptr;
  }
  outer_list->Append(*values);
  if (alt_text_present) {
    CSSValueList* alt_values = CSSValueList::CreateSpaceSeparated();
    do {
      CSSParserSavePoint savepoint(stream);
      CSSValue* alt_value = nullptr;
      if (RuntimeEnabledFeatures::CSSAltCounterEnabled() &&
          stream.Peek().FunctionId() == CSSValueID::kCounter) {
        alt_value =
            ConsumeCounterContent(stream, context, local_context, false);
      } else if (RuntimeEnabledFeatures::CSSAltCounterEnabled() &&
                 stream.Peek().FunctionId() == CSSValueID::kCounters) {
        alt_value = ConsumeCounterContent(stream, context, local_context, true);
      } else {
        alt_value = css_parsing_utils::ConsumeString(stream);
      }
      if (!alt_value) {
        break;
      }
      alt_values->Append(*alt_value);
      savepoint.Release();
    } while (!stream.AtEnd());
    if (!alt_values->length()) {
      return nullptr;
    }

    outer_list->Append(*alt_values);
  }
  return outer_list;
}
```

Excerpt 3215–3220, with complete original license header:

```cpp
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

const CSSValue* Content::ParseSingleValue(
    CSSParserTokenStream& stream,
    const CSSParserContext& context,
    CSSParserLocalContext& local_context) const {
  return ParseContentValue(stream, context, local_context);
}
```
