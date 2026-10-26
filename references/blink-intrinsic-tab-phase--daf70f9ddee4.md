# Blink intrinsic tab-phase source capture

This bounded static capture records the decisive implementation path behind the
[#64 intrinsic-phase determination](https://github.com/bj-data-eng/surgeist/issues/64#issuecomment-6389474658).
The [Text/Layout tab contract](../docs/text-layout-contract.md#preserved-css-tab-stops)
owns Surgeist's selected operation. This reference supplies evidence and its
limits; it does not certify implementation, browser execution or parity.

## Source identity and attribution

Upstream: [Chromium Blink](https://www.chromium.org/blink/), immutable commit
`daf70f9ddee4b45d4f8870f574fbde61c0db4ca7`. Retrieved on 24 October 2026 through
the GitHub repository-file API at that exact revision. The table hashes each
complete original UTF-8 file, including its header and final newline. Git blob
identities were independently recomputed using Git's blob header and original
bytes. Only the explicitly bounded excerpts below are retained in this document;
they are not compiled, executed, linked or imported as a product dependency.

| Original file | Git blob | Original bytes | Raw SHA-256 |
| --- | --- | ---: | --- |
| [`inline_node.cc`](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node.cc) | `7cb5320e979d4ca3f9e3d69d603d9cedd5b91a0d` | 96597 | `fc57e21de1c0c2c29bd2c2d92ca8b8d8b7aa7bab69a524d7c185c7859aa793f1` |
| [`line_breaker.cc`](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/line_breaker.cc) | `b973ea4f1c04fc16ea021edf3c67d08bd87ea8ba` | 189392 | `84d6fe431463f7d28423f810824b06251957b7e5ddad25a79ce8533fd3864380` |
| [`shape_result.cc`](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/shaping/shape_result.cc) | `e80c6c3fa919b9965af854d48e6bd4400436327f` | 92478 | `fc66a4c3f0cb927b9a24d2c73e5780eea2e87de53eb373c11de145e84227caa5` |
| [`font.cc`](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/font.cc) | `8c11f957239946c2bb2d6502de1fe070088882d8` | 14433 | `3a4544b07fcd3e8e675a649842f015bb9e9401f7916b44c4603bd5dd1860a077` |
| [`inline_node_test.cc`](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node_test.cc) | `541f351f45f00f54cca0fc86bc2e7c8a02133a62` | 63796 | `aac3d35023adba7faee33294c72256117d643326042c6ff9d9df9ac07b51045e` |

Original paths are the linked repository paths above. Headings, line locators,
omission boundaries and explanatory prose are additions; retained source
segments are unchanged.

`inline_node.cc`, `line_breaker.cc` and `inline_node_test.cc` each have this
complete original notice:

```cpp
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
```

Their pinned root LICENSE is byte-identical to the retained
[Chromium BSD-style license](../licenses/chromium/LICENSE.txt), SHA-256
`368cca1106be99d39ecd32a38d8305585d802a475effb66380b91ffc9bcf709b`;
[immutable original](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/LICENSE),
Git blob `2249a28657f86c645f8978b75babc1bd52aca318`.

`shape_result.cc` retains copyright (c) 2012 Google Inc. and copyright (C) 2013
BlackBerry Limited. Its complete original file-specific three-clause BSD notice
is byte-identical to the existing
[shape-result notice](../licenses/chromium/shape-result-cc-range-license-header.txt),
SHA-256 `ca25e400635fa86af97842949c0c73ca4f5541c4d25d72a2cf8639d232e4c6d3`.
That file-specific notice, including conditions and disclaimer, accompanies the
excerpt; the generic Chromium license does not replace it.

`font.cc` has the following complete original notice. Its GNU Library General
Public License version 2-or-later terms remain distinct from those BSD notices;
the complete [GNU Library GPL v2 text](../licenses/webkit/LICENSE-LGPL-2.txt)
is already retained locally. The linked immutable original supplies the complete
corresponding source.

```cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2000 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2003, 2006, 2010, 2011 Apple Inc. All rights reserved.
 * Copyright (c) 2007, 2008, 2010 Google Inc. All rights reserved.
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

## Intrinsic sizing enters the real line breaker

`ComputeMinMaxSizes` selects min-content and, when needed, max-content
`ComputeContentSize`. The latter creates a real `LineBreaker` with the selected
mode. Zero or maximum available inline size establishes fitting constraints; it
does not alone identify the tab phase. The later position initialization and tab
resolver complete that trace.

[`inline_node.cc`, original lines 2326–2355](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node.cc#L2326-L2355):

```cpp
MinMaxSizesResult InlineNode::ComputeMinMaxSizes(
    WritingMode container_writing_mode,
    const ConstraintSpace& space,
    const MinMaxSizesFloatInput& float_input) const {
  PrepareLayoutIfNeeded();

  // Compute the max of inline sizes of all line boxes with 0 available inline
  // size. This gives the min-content, the width where lines wrap at every
  // break opportunity.
  LineBreaker::MaxSizeCache max_size_cache;
  MinMaxSizes sizes;
  std::optional<LayoutUnit> max_size;
  bool depends_on_block_constraints = false;
  sizes.min_size =
      ComputeContentSize(*this, container_writing_mode, space, float_input,
                         LineBreakerMode::kMinContent, &max_size_cache,
                         &max_size, &depends_on_block_constraints);
  if (max_size) {
    sizes.max_size = *max_size;
  } else {
    sizes.max_size = ComputeContentSize(
        *this, container_writing_mode, space, float_input,
        LineBreakerMode::kMaxContent, &max_size_cache, nullptr, nullptr);
  }

  // Negative text-indent can make min > max. Ensure max encompasses the min.
  sizes.max_size = std::max(sizes.min_size, sizes.max_size);

  return MinMaxSizesResult(sizes, depends_on_block_constraints);
}
```

[`inline_node.cc`, original lines 1983–2005](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node.cc#L1983-L2005):

```cpp
static LayoutUnit ComputeContentSize(InlineNode node,
                                     WritingMode container_writing_mode,
                                     const ConstraintSpace& space,
                                     const MinMaxSizesFloatInput& float_input,
                                     LineBreakerMode mode,
                                     LineBreaker::MaxSizeCache* max_size_cache,
                                     std::optional<LayoutUnit>* max_size_out,
                                     bool* depends_on_block_constraints_out) {
  const ComputedStyle& style = node.Style();
  LayoutUnit available_inline_size =
      mode == LineBreakerMode::kMaxContent ? LayoutUnit::Max() : LayoutUnit();

  ExclusionSpace empty_exclusion_space;
  LeadingFloats empty_leading_floats;
  LineLayoutOpportunity line_opportunity(available_inline_size);
  LayoutUnit result;
  LineBreaker line_breaker(
      node, mode, space, line_opportunity, empty_leading_floats,
      /* break_token */ nullptr,
      /* column_spanner_path */ nullptr, &empty_exclusion_space);
  line_breaker.SetIntrinsicSizeOutputs(max_size_cache,
                                       depends_on_block_constraints_out);
  const InlineItemsData& items_data = line_breaker.ItemsData();
```

The ordinary sizing loop consumes the breaker's actual line widths:

[`inline_node.cc`, original lines 2236–2242](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node.cc#L2236-L2242):

```cpp
  LineInfo line_info;
  do {
    line_breaker.NextLine(&line_info);
    if (line_info.Results().empty())
      break;

    LayoutUnit inline_size = line_info.Width();
```

Tabs explicitly prevent assuming that max-content advances equal those obtained
during the min-content pass:

[`inline_node.cc`, original lines 2200–2207](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node.cc#L2200-L2207):

```cpp
#endif
          // Tabulation characters change the widths by their positions, so
          // their widths for the max size may be different from the widths for
          // the min size. Fall back to 2 pass for now.
          if (c == uchar::kTab) {
            AddTabulationCharacters(item, result.Length());
            continue;
          }
```

## Line initialization supplies the logical pen

For intrinsic modes percentage indent resolves to zero. Applicable indent is
recorded before the line's initial position is assigned. After computing base
direction, the line breaker sets `position_` to that indent without adding an
opposite-block-edge width. These are source observations about this path, not a
claim that the real block edge is numerically zero.

[`line_breaker.cc`, original lines 811–824](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/line_breaker.cc#L811-L824):

```cpp
  DCHECK(!line_info->TextIndent());
  const ComputedStyle& style = line_info->LineStyle();
  if (ShouldApplyTextIndent(style, is_first_formatted_line_,
                            previous_line_had_forced_break_) &&
      !IsSubLineBreaker()) [[unlikely]] {
    const Length& length = style.TextIndent();
    LayoutUnit maximum_value;
    // Ignore percentages (resolve to 0) when calculating min/max intrinsic
    // sizes.
    if (length.HasPercent() && mode_ == LineBreakerMode::kContent) {
      maximum_value = constraint_space_.AvailableSize().inline_size;
    }
    line_info->SetTextIndent(MinimumValueForLength(length, maximum_value));
  }
```

[`line_breaker.cc`, original lines 837–846](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/line_breaker.cc#L837-L846):

```cpp
  ComputeBaseDirection();
  line_info->SetBaseDirection(base_direction_);
  hyphen_index_.reset();
  has_any_hyphens_ = false;
  resume_block_in_inline_in_same_flow_ = false;

  // Use 'text-indent' as the initial position. This lets tab positions to align
  // regardless of 'text-indent'.
  applied_text_indent_ = line_info->TextIndent();
  position_ = applied_text_indent_;
```

## Preserved TAB uses that pen directly

The TAB branch passes `position_` to tabulation shaping. The feature-controlled
font selection shown here is retained as context; it is not adopted as a Surgeist
feature switch.

[`line_breaker.cc`, original lines 2910–2938](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/line_breaker.cc#L2910-L2938):

```cpp
void LineBreaker::HandleControlItem(const InlineItem& item,
                                    LineInfo* line_info) {
  DCHECK_GE(item.Length(), 1u);
  if (item.TextType() == TextItemType::kForcedLineBreak) {
    HandleForcedLineBreak(&item, line_info);
    return;
  }

  DCHECK_EQ(item.TextType(), TextItemType::kFlowControl);
  UChar character = Text()[item.StartOffset()];
  switch (character) {
    case uchar::kTab: {
      DCHECK(item.Style());
      const ComputedStyle& style = *item.Style();
      if (!style.GetFont()->PrimaryFont()) {
        // TODO(crbug.com/561873): PrimaryFont should not be nullptr.
        HandleEmptyText(item, line_info);
        return;
      }
      const Font* font = RuntimeEnabledFeatures::TabSizeAncestorEnabled()
                             ? &node_.FontForTab()
                             : style.GetFont();
      const ShapeResult* shape_result =
          ShapeResult::CreateForTabulationCharacters(
              font, item.Direction(), style.GetTabSize(), position_,
              item.StartOffset(), item.Length());
      HandleText(item, *shape_result, line_info);
      return;
    }
```

Tab shaping sends that position directly to `Font::TabWidth`. Direction selects
the run direction and glyph indices. The second and later tabs receive the base
width. No opposite-edge phase correction occurs in this function.

[`shape_result.cc`, original lines 1860–1906](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/shaping/shape_result.cc#L1860-L1906):

```cpp
const ShapeResult* ShapeResult::CreateForTabulationCharacters(
    const Font* font,
    TextDirection direction,
    const TabSize& tab_size,
    float position,
    unsigned start_index,
    unsigned length) {
  DCHECK_GT(length, 0u);
  const SimpleFontData* font_data = font->PrimaryFontForTabSize();
  DCHECK(font_data);
  ShapeResult* result =
      MakeGarbageCollected<ShapeResult>(start_index, length, direction);
  result->has_vertical_offsets_ =
      font_data->PlatformData().IsVerticalAnyUpright();
  // Tab characters are always LTR or RTL, not TTB, even when
  // isVerticalAnyUpright().
  hb_direction_t hb_direction =
      blink::IsLtr(direction) ? HB_DIRECTION_LTR : HB_DIRECTION_RTL;
  // Only the advance of the first tab is affected by |position|.
  TextRunLayoutUnit advance = TextRunLayoutUnit::FromFloatRound(
      font->TabWidth(font_data, tab_size, position));
  do {
    unsigned run_length = std::min(length, HarfBuzzRunGlyphData::kMaxGlyphs);
    ShapeResultRun* run = MakeGarbageCollected<ShapeResultRun>(
        font_data, hb_direction, CanvasRotationInVertical::kRegular,
        HB_SCRIPT_COMMON, start_index, run_length, run_length);
    InlineLayoutUnit run_width;
    for (unsigned i = 0; i < run_length; i++) {
      // 2nd and following tabs have the base width, without using |position|.
      if (i == 1) {
        advance = TextRunLayoutUnit::FromFloatRound(
            font->TabWidth(font_data, tab_size));
      }
      const unsigned index = blink::IsLtr(direction) ? i : length - 1 - i;
      run->glyph_data_[i] = {font_data->SpaceGlyph(), index, SafeToBreak::kSafe,
                             advance};
      run_width += advance;
    }
    run->width_ = run_width;
    result->width_ += run->width_;
    result->runs_.push_back(run);
    DCHECK_GE(length, run_length);
    length -= run_length;
    start_index += run_length;
  } while (length);
  return result;
}
```

The selected float-position resolver computes a forward remainder against the
base tab width and applies its own minimum-gap threshold. Its threshold is
**half SPACE width**, which differs from Surgeist's adopted **half ch**. This
excerpt does not select Blink's signed-period, rounding or zero-tab behavior for
Surgeist.

[`font.cc`, original lines 313–334](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/font.cc#L313-L334):

```cpp
float Font::TabWidth(const SimpleFontData* font_data,
                     const TabSize& tab_size,
                     float position) const {
  float base_tab_width = TabWidth(font_data, tab_size);
  if (!base_tab_width)
    return GetFontDescription().LetterSpacing();

  float modulized_position = fmodf(position, base_tab_width);
  if (modulized_position < 0) [[unlikely]] {
    modulized_position += base_tab_width;
  }

  float distance_to_tab_stop = base_tab_width - modulized_position;

  // Let the minimum width be the half of the space width so that it's always
  // recognizable.  if the distance to the next tab stop is less than that,
  // advance an additional tab stop.
  if (distance_to_tab_stop < font_data->SpaceWidth() / 2)
    distance_to_tab_stop += base_tab_width;

  return distance_to_tab_stop;
}
```

## Static intrinsic test witness

The table declares min/max expectations and includes two preserved TABs followed
by X:

[`inline_node_test.cc`, original lines 491–497](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node_test.cc#L491-L497):

```cpp
struct MinMaxData {
  const char* content;
  std::array<int, 2> min_max;
  const char* target_style = "";
  const char* style = "";
  const char* lang = nullptr;
} min_max_data[] = {
```

[`inline_node_test.cc`, original lines 510–513](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node_test.cc#L510-L513):

```cpp
    // Tabulation characters with `break-word`.
    {"&#9;&#9;<span>X</span>",
     {10, 170},
     "white-space: pre-wrap; word-break: break-word;"},
```

The parameterized test loads Ahem, sets a 10px font and compares the computed
min/max sizes to that table:

[`inline_node_test.cc`, original lines 553–583](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_node_test.cc#L553-L583):

```cpp
class MinMaxTest : public InlineNodeTest,
                   public testing::WithParamInterface<MinMaxData> {};

INSTANTIATE_TEST_SUITE_P(InlineNodeTest,
                         MinMaxTest,
                         testing::ValuesIn(min_max_data));

TEST_P(MinMaxTest, Data) {
  const MinMaxData& data = GetParam();
  LoadAhem();
  StringBuilder html;
  html << R"HTML(
    <!DOCTYPE html>
    <style>
    #target { font: 10px Ahem;)HTML"
       << data.target_style << " }\n    " << data.style << R"HTML(
    </style>
    <div id="target")HTML";
  if (data.lang) {
    html << " lang='" << data.lang << "'";
    LayoutLocale::SetHyphenationForTesting(AtomicString(data.lang),
                                           MockHyphenation::Create());
  }
  html << ">" << data.content << "</div>";
  SetupHtml("target", html.ReleaseString());
  InlineNodeForTest node = CreateInlineNode();
  const MinMaxSizes actual_sizes = ComputeMinMaxSizes(node);
  const MinMaxSizes expected_sizezs{LayoutUnit(data.min_max[0]),
                                    LayoutUnit(data.min_max[1])};
  EXPECT_EQ(actual_sizes, expected_sizezs);
}
```

With the fixture's 10px Ahem advance and default eight-space tab period, the
expected maximum 170 is consistent with two 80px tabs followed by a 10px X. It
supports logical tab accumulation. This row does not independently exercise
opposite paragraph/block starts, signed periods, half-ch thresholds or the
unconstrained opposite-edge case. The source test was inspected, not executed.

## Native determination and limits of inference

For unconstrained min-/max-content measurement, the adopted #64 determination
selects a logical grid at the unindented paragraph line-start, with the pen
initialized by applicable resolved indent and advanced in paragraph direction.
This applies independently of whether paragraph and nearest-block starts coincide.
Used layout still takes
its origin from the real nearest-block content-start edge, including authentic
displacement and direction. The two operations require distinct request/cache
identity and recomputation; an intrinsic result is not final placement.

That is a native measurement convention supported by the complete source path,
not a located CSSWG resolution of the indefinite opposite-edge phase. The
precedent supplies an implementable convention; it does not establish complete
browser equivalence. Surgeist retains its already selected half-ch,
signed-derived-period, Phase II, source-custody, line-selection and numeric rules.
No Blink implementation is integrated by retaining these excerpts.

The earlier WebKit inspection remains narrower: its metric helper and intrinsic
measurement modes did not settle this opposite-edge phase. This Blink capture
adds line initialization, actual TAB dispatch, shaping, resolver and static test
context; the zero/infinite constraint alone was not the decisive evidence.
No browser, Blink test, build or downloaded source code was executed for this
capture. Surgeist implementation and composed verification remain with #64.
