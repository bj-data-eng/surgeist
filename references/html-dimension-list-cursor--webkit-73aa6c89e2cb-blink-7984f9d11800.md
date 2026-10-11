# HTML dimension-list decimal cursor correction

Selected for Surgeist HTML #1133 on 11 October 2026. This is a bounded source-deficiency determination, not an upstream erratum or a replacement HTML edition.

## Determination

In the pinned HTML algorithm, inner step 7 tests for U+002E but fails to advance the cursor before collecting whitespace/digits. The collection therefore consumes nothing, making its fraction branch unreachable and hiding the subsequent unit. Insert exactly one cursor advance after recognizing the dot, before collecting that sequence. Keep the remaining pinned steps unchanged.

Expected results: `1.5%` → (1.5, percentage), `1.5*` → (1.5, relative), `1.5` → (1.5, absolute), `.5%` → (0.5, percentage), `1.%` → (1, percentage), `1. 5 0%` → (1.5, percentage). The last two follow the existing pinned empty-fraction and whitespace-removal steps after this correction. Test these independently, plus integer-only and comma-list controls.

Pinned normative source: [HTML §2.3.4.7](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/common-microsyntaxes.html#rules-for-parsing-a-list-of-dimensions), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`; complete source SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. The retained HTML foundations capture preserves the original wording and must not be silently repaired.

WebKit explicitly increments its cursor in this exact list algorithm. Blink likewise consumes the dot before its fraction collection, corroborating `1.5%`; its leading-digit guard differs for `.5%` and is not adopted. Browser overflow, trimming, empty-list and other differences are not selected by this correction. The surrounding pinned fraction computation plus the exact WebKit cursor operation establish the bounded interpretation. No claim of upstream ratification or browser-wide agreement on the whole algorithm is made.


## WebKit/WebKit witness

[Immutable complete source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/parser/HTMLParserIdioms.cpp). Complete source bytes: 24645; SHA-256: `99107062a7eaec7499838aaa5fab77045d6d92348c62b260cff4daf9cbef226d`. Selected original lines 543–684, plus original file license header. These are source excerpts, not an HTML standard or an adopted implementation.

```cpp
/*
 * Copyright (C) 2010-2023 Apple Inc. All rights reserved.
 * Copyright (C) 2014 Google Inc. All rights reserved.
 * Copyright (C) 2025 Samuel Weinig <sam@webkit.org>
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
```

```cpp
template<typename CharacterType>
static FixedVector<HTMLDimensionsListValue> parseHTMLDimensionsList(StringParsingBuffer<CharacterType>& rawInput)
{
    // https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#lists-of-dimensions

    // 1. Let `raw input` be the string being parsed.
    // 2. If the last character in `raw input` is a U+002C COMMA character (,), then remove that character from `raw input`.
    if (rawInput[rawInput.lengthRemaining() - 1] == ',')
        rawInput.dropLast();

    // 3. Split the string raw input on commas. Let `raw tokens` be the resulting list of tokens.
    auto numberOfCommas = countCommas(rawInput);
    auto numberOfTokens = numberOfCommas + 1;

    // 4. Let `result` be an empty list of number/unit pairs.
    FixedVector<HTMLDimensionsListValue> result(numberOfTokens);

    // 5. For each token in raw tokens, run the following substeps:
    for (size_t i = 0; i < numberOfTokens; ++i) {
        // NOTE: The "Split the string raw input on commas" step above is being done lazily
        // and includes stripping leading and trailing whitespace from each token.
        skipWhile<isASCIIWhitespace>(rawInput);

        // NOTE: Step 5.5 is done first as an optimization.
        // 5.5. If position is past the end of input, set unit to relative and jump to the last substep.
        if (!rawInput.hasCharactersRemaining()) {
            result[i] = HTMLDimensionsListValue { .number = 0, .unit = HTMLDimensionsListValue::Unit::Relative };
            continue;
        }
        if (*rawInput == ',') {
            // Move past the comma.
            ++rawInput;
            result[i] = HTMLDimensionsListValue { .number = 0, .unit = HTMLDimensionsListValue::Unit::Relative };
            continue;
        }

        // 5.1. Let `input` be the token.
        // 5.2. Let `position` be a pointer into input, initially pointing at the start of the string.
        // NOTE: As our implementation finds the tokens lazily, the pointer is just `raw input` itself, not `position`.

        // 5.3. Let `value` be the number 0.
        double value = 0;

        // 5.4. Let `unit` be absolute.
        HTMLDimensionsListValue::Unit unit = HTMLDimensionsListValue::Unit::Absolute;

        // 5.6. If the character at position is an ASCII digit, collect a sequence of code points that are ASCII digits from input given position, interpret the resulting sequence as an integer in base ten, and increment value by that integer.
        Checked<unsigned, RecordOverflow> integer = 0;
        while (rawInput.hasCharactersRemaining() && isASCIIDigit(*rawInput)) {
            integer *= 10;
            integer += (*rawInput - '0');
            ++rawInput;
        }

        // The spec does not specify how to deal with arbitrarily large numbers, so we bail on overflow, falling back on "1*", matching the previous implementation.
        // Filed https://github.com/whatwg/html/issues/11539 to track a standard solution.

        if (integer.hasOverflowed()) [[unlikely]] {
            result[i] = HTMLDimensionsListValue { .number = 1, .unit = HTMLDimensionsListValue::Unit::Relative };

            skipUntil(rawInput, ',');
            if (rawInput.hasCharactersRemaining())
                ++rawInput;
            continue;
        }

        value = integer.value();

        // 5.7. If the character at position is U+002E (.), then:
        if (rawInput.hasCharactersRemaining() && *rawInput == '.') {
            ++rawInput;

            // 5.7.1. Collect a sequence of code points consisting of ASCII whitespace and ASCII digits from input given position. Let `s` be the resulting sequence.
            // 5.7.2. Remove all ASCII whitespace in `s`.

            unsigned length = 0;
            Checked<unsigned, RecordOverflow> fraction = 0;
            while (rawInput.hasCharactersRemaining() && (isASCIIWhitespace(*rawInput) || isASCIIDigit(*rawInput))) {
                if (isASCIIDigit(*rawInput)) {
                    ++length;
                    fraction *= 10;
                    fraction += (*rawInput - '0');
                }
                ++rawInput;
            }

            // The spec does not specify how to deal with arbitrarily large numbers, so we bail on overflow, falling back on "1*", matching the previous implementation.
            // Filed https://github.com/whatwg/html/issues/11539 to track a standard solution.

            if (fraction.hasOverflowed()) [[unlikely]] {
                result[i] = HTMLDimensionsListValue { .number = 1, .unit = HTMLDimensionsListValue::Unit::Relative };

                skipUntil(rawInput, ',');
                if (rawInput.hasCharactersRemaining())
                    ++rawInput;
                continue;
            }

            // 5.7.3. If `s` is not the empty string, then:
            // 5.7.3.1. Let `length` be the number of characters in s (after the spaces were removed).
            // 5.7.3.2. Let `fraction` be the result of interpreting s as a base-ten integer, and then dividing that number by 10^length.
            // 5.7.3.3. Increment value by fraction.

            if (length > 0)
                value += fraction.value() / std::pow(10.0, length);
        }

        // 5.8. Skip ASCII whitespace within input given position.
        skipWhile<isASCIIWhitespace>(rawInput);

        // 5.9. If the character at position is a U+0025 PERCENT SIGN character (%), then set unit to percentage.
        //      Otherwise, if the character at position is a U+002A ASTERISK character (*), then set unit to relative.
        if (rawInput.hasCharactersRemaining()) {
            if (*rawInput == '%') {
                ++rawInput;
                unit = HTMLDimensionsListValue::Unit::Percentage;
            } else if (*rawInput == '*') {
                ++rawInput;
                unit = HTMLDimensionsListValue::Unit::Relative;
            }
        }

        // 5.10. Add an entry to result consisting of the number given by `value` and the unit given by `unit`.
        result[i] = HTMLDimensionsListValue { .number = value, .unit = unit };

        // NOTE: This means trailing junk is allowed.
        skipUntil(rawInput, ',');
        if (rawInput.hasCharactersRemaining())
            ++rawInput;
    }

    return result;
}

FixedVector<HTMLDimensionsListValue> parseHTMLDimensionsList(StringView listOfDimensionsString)
{
    if (listOfDimensionsString.isEmpty())
        return { };

    return readCharactersForParsing(listOfDimensionsString, [](auto buffer) {
        return parseHTMLDimensionsList(buffer);
    });
```


## chromium/chromium witness

[Immutable complete source](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/html/html_dimension.cc). Complete source bytes: 7699; SHA-256: `9e5b54fa9647a5e3cc4ee6317e6cea5ff1312078d445ddf586fae04f7f866090`. Selected original lines 43–143, plus original file license header. These are source excerpts, not an HTML standard or an adopted implementation.

```cpp
/*
 * Copyright (C) 2013 Google Inc. All rights reserved.
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
```

```cpp
template <typename CharacterType>
static HTMLDimension ParseDimension(
    base::span<const CharacterType> characters) {
  // HTML5's split removes leading and trailing spaces so we need to skip the
  // leading spaces here.
  const size_t digits_start =
      SkipWhile<CharacterType, IsAsciiSpace>(characters, 0);

  // This is Step 5.5. in the algorithm. Going to the last step would make the
  // code less readable.
  if (digits_start >= characters.size()) {
    return HTMLDimension(0., HTMLDimension::kRelative);
  }

  size_t position =
      SkipWhile<CharacterType, IsAsciiDigit>(characters, digits_start);

  double value = 0.;
  if (position > digits_start) {
    std::optional<unsigned> integer_value = CharactersToUInt(
        characters.subspan(digits_start, position - digits_start),
        NumberParsingOptions());
    if (!integer_value) {
      return HTMLDimension(0., HTMLDimension::kRelative);
    }
    value += *integer_value;

    if (SkipExactly<CharacterType>(characters, '.', position)) {
      Vector<CharacterType> fraction_numbers;
      while (position < characters.size() &&
             (IsAsciiDigit(characters[position]) ||
              IsAsciiSpace(characters[position]))) {
        if (IsAsciiDigit(characters[position])) {
          fraction_numbers.push_back(characters[position]);
        }
        ++position;
      }

      if (fraction_numbers.size()) {
        std::optional<unsigned> fraction_value = CharactersToUInt(
            base::span(fraction_numbers), NumberParsingOptions());
        if (!fraction_value) {
          return HTMLDimension(0., HTMLDimension::kRelative);
        }
        value += static_cast<double>(*fraction_value) /
                 pow(10., static_cast<double>(fraction_numbers.size()));
      }
    }
  }

  position = SkipWhile<CharacterType, IsAsciiSpace>(characters, position);

  HTMLDimension::HTMLDimensionType type = HTMLDimension::kAbsolute;
  if (position < characters.size()) {
    if (characters[position] == '*')
      type = HTMLDimension::kRelative;
    else if (characters[position] == '%')
      type = HTMLDimension::kPercentage;
  }

  return HTMLDimension(value, type);
}

static HTMLDimension ParseDimension(const StringView& token) {
  return VisitCharacters(token,
                         [](auto chars) { return ParseDimension(chars); });
}

// This implements the "rules for parsing a list of dimensions" per HTML5.
// http://www.whatwg.org/specs/web-apps/current-work/multipage/common-microsyntaxes.html#rules-for-parsing-a-list-of-dimensions
Vector<HTMLDimension> ParseListOfDimensions(const StringView& input) {
  static const char kComma = ',';

  // Step 2. Remove the last character if it's a comma.
  StringView trimmed_input = input;
  if (trimmed_input.ends_with(kComma)) {
    trimmed_input.remove_suffix(1);
  }

  // HTML5's split doesn't return a token for an empty string so
  // we need to match them here.
  if (trimmed_input.empty()) {
    return Vector<HTMLDimension>();
  }

  // Step 3. To avoid String copies, we just look for commas instead of
  // splitting.
  Vector<HTMLDimension> parsed_dimensions;
  while (true) {
    const wtf_size_t next_comma = trimmed_input.find(kComma);
    if (next_comma == kNotFound)
      break;

    parsed_dimensions.push_back(
        ParseDimension(StringView(trimmed_input, 0, next_comma)));
    trimmed_input.remove_prefix(next_comma + 1);
  }

  parsed_dimensions.push_back(ParseDimension(trimmed_input));
  return parsed_dimensions;
}
```
