# Image counter symbols: pinned browser witnesses

Captured 2026-10-12 for Surgeist #707. These are bounded source excerpts, not executed browser tests or proof of complete image-counter support. Source is retained as reference only. Added analysis is Surgeist's, not upstream text.

WebKit's pinned grammar gates image symbols; its shown descriptor carrier/converter retains text and its formatter returns strings. The converter's unhandled default is not evidence for an intentional image-to-empty-string language rule. Blink likewise gates image syntax, while the shown SymbolToString handles strings and assumes custom identifiers otherwise. Neither inspected path defines a complete image-symbol conversion for a text-only consumer. Do not copy the unsupported conversion path or infer browser interoperability from syntax recognition. WebKit's fallback method corroborates same-value fallback and decimal termination, but applying it specifically to a text-only output requirement is the explicit Surgeist policy in #707.

Attribution: [WebKit](https://webkit.org/) and [Chromium/Blink](https://www.chromium.org/blink/). Original file headers are preserved below where present; CSSProperties.json has no initial copyright header. Chromium files use the retained [Chromium BSD license](../licenses/chromium/LICENSE.txt). WebKit C++ excerpts retain their complete Apple BSD notices. This reference changes only presentation and selects the stated line ranges.

## WebKit: Source/WebCore/css/CSSProperties.json

[Source at 73aa6c89e2cb77c46184a81aec944e4ab99d114d](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSProperties.json); full-file SHA-256 `848d6e24bbd4421ced9e8e840d718a5da012703f9c95851d60f7c1d1364ebaac`. Original line ranges: 16098–16106.

Original lines 16098–16106:

```cpp
        "<symbol>": {
            "grammar": "<string> | <image allowed-types=url,generated>@(settings-flag=counterStyleAtRuleImageSymbolsEnabled) | <custom-ident>",
            "grammar-comment": "There are inherent difficulties in supporting <image> symbols in @counter-styles, so gate them behind a flag for now - https://bugs.webkit.org/show_bug.cgi?id=167645.",
            "exported": true,
            "specification": {
                "category": "css-counter-styles",
                "url": "https://www.w3.org/TR/css-counter-styles-3/#counter-style-symbols"
            }
        },
```

## WebKit: Source/WebCore/css/CSSCounterStyleDescriptors.h

[Source at 73aa6c89e2cb77c46184a81aec944e4ab99d114d](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSCounterStyleDescriptors.h); full-file SHA-256 `f72ae6e38b6dc4212667b1aae95d08903045d6edfde11c4530dda5de1f831471`. Original line ranges: 40–54.

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
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
```

Original lines 40–54:

```cpp
    using Name = AtomString;
    using Ranges = Vector<std::pair<int, int>>;
    using SystemData = std::pair<CSSCounterStyleDescriptors::Name, int>;
    // The keywords that can be used as values for the counter-style `system` descriptor.
    // https://www.w3.org/TR/css-counter-styles-3/#counter-style-system
    struct Symbol {
        bool isCustomIdent { false };
        String text;
        friend bool operator==(const Symbol&, const Symbol&) = default;
        String cssText() const;
    };
    using AdditiveSymbols = Vector<std::pair<Symbol, unsigned>>;
    enum class System : uint8_t {
        Cyclic,
        Numeric,
```

## WebKit: Source/WebCore/css/CSSCounterStyleDescriptors.cpp

[Source at 73aa6c89e2cb77c46184a81aec944e4ab99d114d](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSCounterStyleDescriptors.cpp); full-file SHA-256 `fa2e6550f574f1ba171b89401cf98148b2f053c8a5399ecb1a344be9a75b8ec5`. Original line ranges: 81–91, 157–167.

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
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
```

Original lines 81–91:

```cpp
CSSCounterStyleDescriptors::Symbol symbolFromCSSValue(const CSSValue* value)
{
    if (RefPtr customIdentValue = dynamicDowncast<CSSCustomIdentValue>(value))
        return { .isCustomIdent = true, .text = customIdentValue->customIdent().value };

    if (RefPtr stringValue = dynamicDowncast<CSSStringValue>(value))
        return { .isCustomIdent = false, .text = stringValue->string().value };

    return { };
}

```

Original lines 157–167:

```cpp
Vector<CSSCounterStyleDescriptors::Symbol> symbolsFromCSSValue(const CSSValue& value)
{
    Vector<CSSCounterStyleDescriptors::Symbol> result;
    for (Ref symbolValue : downcast<CSSValueList>(value)) {
        auto symbol = symbolFromCSSValue(symbolValue.ptr());
        if (!symbol.text.isNull())
            result.append(symbol);
    }
    return result;
}

```

## WebKit: Source/WebCore/css/CSSRegisteredCounterStyle.cpp

[Source at 73aa6c89e2cb77c46184a81aec944e4ab99d114d](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSRegisteredCounterStyle.cpp); full-file SHA-256 `4f42da794aff94ddcccb6286f1bcb4fe1e5ad2f9d210a8b65c92159458b62ec2`. Original line ranges: 501–528.

```cpp
/*
 * Copyright (C) 2022-2024 Apple Inc. All rights reserved.
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

Original lines 501–528:

```cpp
String CSSRegisteredCounterStyle::fallbackText(int value, WritingMode writingMode)
{
    if (m_isFallingBack || !fallback().get()) {
        m_isFallingBack = false;
        return CSSCounterStyleRegistry::decimalCounter()->text(value, writingMode);
    }
    m_isFallingBack = true;
    auto fallbackText = protect(fallback())->text(value, writingMode);
    m_isFallingBack = false;
    return fallbackText;
}

String CSSRegisteredCounterStyle::text(int value, WritingMode writingMode)
{
    if (!isInRange(value))
        return fallbackText(value, writingMode);

    auto result = initialRepresentation(value, writingMode);
    if (result.isNull())
        return fallbackText(value, writingMode);
    if (!applyPadSymbols(result, value))
        return fallbackText(value, writingMode);
    if (shouldApplyNegativeSymbols(value))
        applyNegativeSymbols(result);

    return result;
}

```

## Blink: third_party/blink/renderer/core/css/parser/at_rule_counter_style_descriptor_parser.cc

[Source at 7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/at_rule_counter_style_descriptor_parser.cc); full-file SHA-256 `934e3bfc5900a15604ff56f3bd27ba8be6a7e1132ddf6b251e55a5f19c7b0045`. Original line ranges: 19–40.

```cpp
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
```

Original lines 19–40:

```cpp
CSSValue* ConsumeCounterStyleSymbol(CSSParserTokenStream& stream,
                                    const CSSParserContext& context,
                                    CSSParserLocalContext& local_context) {
  // <symbol> = <string> | <image> | <custom-ident>
  if (CSSValue* string = css_parsing_utils::ConsumeString(stream)) {
    return string;
  }
  if (RuntimeEnabledFeatures::CSSAtRuleCounterStyleImageSymbolsEnabled()) {
    // TODO(crbug.com/40747846): Image values may contain relative units and
    // tree counting functions. They need to handled somehow. <image> values are
    // marked at-risk in the spec.
    if (CSSValue* image =
            css_parsing_utils::ConsumeImage(stream, context, local_context)) {
      return image;
    }
  }
  if (CSSCustomIdentValue* custom_ident = css_parsing_utils::ConsumeCustomIdent(
          stream, context, local_context)) {
    return custom_ident;
  }
  return nullptr;
}
```

## Blink: third_party/blink/renderer/core/css/counter_style.cc

[Source at 7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/counter_style.cc); full-file SHA-256 `9e1b7e39512a51f9d49b0f43b8ed8d06274e1c28feae24fbd59f56a5c34b8ee4`. Original line ranges: 92–97, 783–823.

```cpp
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
```

Original lines 92–97:

```cpp
String SymbolToString(const CSSValue& value) {
  if (const CSSStringValue* string = DynamicTo<CSSStringValue>(value)) {
    return string->Value();
  }
  return To<CSSCustomIdentValue>(value).Value();
}
```

Original lines 783–823:

```cpp
  if (const CSSValue* fallback = style_rule_->GetFallback()) {
    fallback_name_ = To<CSSCustomIdentValue>(fallback)->Value();
  }

  if (HasSymbols(system_)) {
    if (system_ == CounterStyleSystem::kAdditive) {
      const CSSValueList* symbols =
          To<CSSValueList>(style_rule_->GetAdditiveSymbols());
      symbols_.ReserveInitialCapacity(symbols->length());
      additive_weights_.ReserveInitialCapacity(symbols->length());
      for (const auto& symbol : *symbols) {
        const auto& pair = To<CSSValuePair>(*symbol.Get());
        additive_weights_.push_back(
            To<CSSPrimitiveValue>(pair.First()).ComputeInteger(*media_values));
        symbols_.push_back(SymbolToString(pair.Second()));
      }
    } else {
      const CSSValueList* symbols = To<CSSValueList>(style_rule_->GetSymbols());
      symbols_.ReserveInitialCapacity(symbols->length());
      for (const auto& symbol : *symbols) {
        symbols_.push_back(SymbolToString(*symbol.Get()));
      }
    }
  }

  if (const CSSValue* negative = style_rule_->GetNegative()) {
    if (const CSSValuePair* pair = DynamicTo<CSSValuePair>(negative)) {
      negative_prefix_ = SymbolToString(pair->First());
      negative_suffix_ = SymbolToString(pair->Second());
    } else {
      negative_prefix_ = SymbolToString(*negative);
    }
  }

  if (const CSSValue* pad = style_rule_->GetPad()) {
    const CSSValuePair& pair = To<CSSValuePair>(*pad);
    pad_length_ =
        To<CSSPrimitiveValue>(pair.First()).ComputeInteger(*media_values);
    pad_symbol_ = SymbolToString(pair.Second());
  }

```

## Newer-standard recheck

The current Counter Styles Editor’s Draft still states 8 October 2026. Its complete selected sections 2, 3.7, 3.8 and 3.9 have identical whitespace-normalized visible text to the already retained [complete 8 October edition](css-counter-styles-3--editor-capture-20261009--ef29d0a06a13.md). Reuse that full Markdown reference instead of adding another overlapping edition. This comparison does not assert identical HTML bytes or an absence of changes outside the selected sections.

Current full HTML SHA-256: `3522e8fd7e1b659bd0586e086b9bf49fd063e66441584cc48e8c335ad1a467f2`. Compared section text hashes:

- `counter-styles`: `4f502fc5fb961a1ec4d8cb9aeae0548fca9ee8837619c03acf91e06344b5678b`.
- `counter-style-symbols`: `1ebc4f983b28de8c8cc79f668702cc5d7a75a8760c699544c18a2d60ca456b64`.
- `counter-style-speak-as`: `74016532826c215626962c2c6ad0d37a0d6c501d089c31b036dcaa0a871fe1d8`.
- `counter-style-fallback`: `36cd08729885cdd72d00f1e9373952f51842f9132c9d2d484f46d0be626b6bb9`.
