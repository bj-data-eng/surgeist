# Overflow-only relative inset browser witnesses

These excerpts identify the sizing inputs supplied to relative-position offsets
at pinned browser revisions. The [capability determination](https://github.com/bj-data-eng/surgeist/issues/1010#issuecomment-6716890125)
separates browser indefinite-axis handling from Block's accepted visual-inset basis.
The governing clauses are retained in [Position 3](css-position-3--WD-css-position-3-20251007--3c8a8120af2c.md#insets)
and [Break 3](css-break-3-layout-decisions--editor-capture-20261010--debb8c3e11e2.md#transforms).

Each entry retains its complete file-level notice, complete original source hash
and exact excerpt bounds. Coverage is limited to the named source lines; browser
execution results are not included. Attribution and full license texts are linked
from the [Chromium notice](../licenses/chromium/NOTICE.md#relative-inset-witnesses)
and [WebKit notice](../licenses/webkit/NOTICE.md#relative-inset-witness).

## Blink relative offset

[Pinned relative_utils.cc](https://raw.githubusercontent.com/chromium/chromium/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/relative_utils.cc)

Complete source SHA-256: `d0cc2b7beb53ad0842e9c8fc0da539fc22bd87a1dd5517e829640032276ef4f6`.

### File-level notice

```cpp
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
```

### Source lines 18–89

```cpp
LogicalOffset ComputeRelativeOffset(
    const ComputedStyle& child_style,
    WritingDirectionMode container_writing_direction,
    const LogicalSize& available_size) {
  if (child_style.GetPosition() != EPosition::kRelative)
    return LogicalOffset();

  const PhysicalSize physical_size = ToPhysicalSize(
      available_size, container_writing_direction.GetWritingMode());

  // Helper function to correctly resolve insets.
  auto ResolveInset = [](const Length& length,
                         LayoutUnit size) -> std::optional<LayoutUnit> {
    if (length.IsAuto())
      return std::nullopt;
    if (length.HasPercent() && size == kIndefiniteSize) {
      return std::nullopt;
    }
    return MinimumValueForLength(length, size);
  };

  std::optional<LayoutUnit> left =
      ResolveInset(child_style.Left(), physical_size.width);
  std::optional<LayoutUnit> right =
      ResolveInset(child_style.Right(), physical_size.width);
  std::optional<LayoutUnit> top =
      ResolveInset(child_style.Top(), physical_size.height);
  std::optional<LayoutUnit> bottom =
      ResolveInset(child_style.Bottom(), physical_size.height);

  // Common case optimization.
  if (!left && !right && !top && !bottom)
    return LogicalOffset();

  // Conflict resolution rules: https://www.w3.org/TR/css-position-3/#rel-pos
  if (!left && !right) {
    left = LayoutUnit();
    right = LayoutUnit();
  } else if (!left) {
    left = -*right;
  } else if (!right) {
    right = -*left;
  }

  if (!top && !bottom) {
    top = LayoutUnit();
    bottom = LayoutUnit();
  } else if (!top) {
    top = -*bottom;
  } else if (!bottom) {
    bottom = -*top;
  }

  switch (container_writing_direction.GetWritingMode()) {
    case WritingMode::kHorizontalTb:
      return container_writing_direction.IsLtr() ? LogicalOffset(*left, *top)
                                                 : LogicalOffset(*right, *top);
    case WritingMode::kVerticalRl:
    case WritingMode::kSidewaysRl:
      return container_writing_direction.IsLtr()
                 ? LogicalOffset(*top, *right)
                 : LogicalOffset(*bottom, *right);
    case WritingMode::kVerticalLr:
      return container_writing_direction.IsLtr()
                 ? LogicalOffset(*top, *left)
                 : LogicalOffset(*bottom, *left);
    case WritingMode::kSidewaysLr:
      return container_writing_direction.IsLtr() ? LogicalOffset(*bottom, *left)
                                                 : LogicalOffset(*top, *left);
    default:
      NOTREACHED();
  }
```

## Blink fragment child offset

[Pinned box_fragment_builder.cc](https://raw.githubusercontent.com/chromium/chromium/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/box_fragment_builder.cc)

Complete source SHA-256: `1245c7815e763be28c1f99f73eaf719cbea10bec072eb0ef84786bb46c0b88b5`.

### File-level notice

```cpp
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
```

### Source lines 255–268

```cpp
  if (!relative_offset) {
    relative_offset = LogicalOffset();
    if (box_type_ != PhysicalFragment::BoxType::kInlineBox) {
      if (child.IsLineBox()) {
        if (child.MayHaveDescendantAboveBlockStart()) [[unlikely]] {
          may_have_descendant_above_block_start_ = true;
        }
      } else if (child.IsCSSBox()) {
        // Apply the relative position offset.
        const auto& box_child = To<PhysicalBoxFragment>(child);
        if (box_child.Style().GetPosition() == EPosition::kRelative) {
          relative_offset = ComputeRelativeOffsetForBoxFragment(
              box_child, GetWritingDirection(), child_available_size_);
        }
```

## Blink child available-size initialization

[Pinned box_fragment_builder.h](https://raw.githubusercontent.com/chromium/chromium/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/box_fragment_builder.h)

Complete source SHA-256: `613403225e206cc371d898654f179f15a5cf0008f2266b51770d775888d13399`.

### File-level notice

```cpp
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
```

### Source lines 85–100

```cpp
    if (!node_ || (!node_.IsTableSection() && !node_.IsTableRow())) {
      border_scrollbar_padding_ = initial_fragment_geometry.border +
                                  initial_fragment_geometry.scrollbar;
      // Padding doesn't take up layout space in fieldset containers (that's
      // done inside the anonymous child wrapper).
      if (!node_ || !node_.IsFieldsetContainer()) {
        border_scrollbar_padding_ += initial_fragment_geometry.padding;
      }
    }
    original_border_scrollbar_padding_block_start_ =
        border_scrollbar_padding_.block_start;
    if (node_) {
      child_available_size_ = CalculateChildAvailableSize(
          space_, To<BlockNode>(node_), size_,
          border_padding_ + initial_fragment_geometry.scrollbar);
    }
```

## Blink child available-size calculation

[Pinned length_utils.cc](https://raw.githubusercontent.com/chromium/chromium/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/length_utils.cc)

Complete source SHA-256: `11a125b26485b7e76272b7dcf56076968f2f5a5cb6edb9e5e3fac45834cab526`.

### File-level notice

```cpp
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
```

### Source lines 1779–1794

```cpp
LogicalSize CalculateChildAvailableSize(
    const ConstraintSpace& space,
    const BlockNode& node,
    const LogicalSize border_box_size,
    const BoxStrut& border_scrollbar_padding) {
  LogicalSize child_available_size =
      ShrinkLogicalSize(border_box_size, border_scrollbar_padding);

  if (space.IsAnonymous() ||
      (node.IsAnonymousBlockFlow() &&
       child_available_size.block_size == kIndefiniteSize)) {
    child_available_size.block_size = space.AvailableSize().block_size;
  }

  return child_available_size;
}
```

## WebKit relative offset

[Pinned RenderBoxModelObject.cpp](https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderBoxModelObject.cpp)

Complete source SHA-256: `77f3cacd5668ec1b423ecf5d0bbf198e7c90952d566e55a36cd0eca542ff909c`.

### File-level notice

```cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2005 Allan Sandfeld Jensen (kde@carewolf.com)
 *           (C) 2005, 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2005-2025 Apple Inc. All rights reserved.
 * Copyright (C) 2010-2018 Google Inc. All rights reserved.
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

### Source lines 382–471

```cpp
LayoutSize RenderBoxModelObject::relativePositionOffset() const
{
    auto containingBlockSkippingAnonymous = [&] {
        // This is a workaround for anonymous blocks interfering with percentage
        // offset resolution. The proper fix is to not construct anonymous blocks in the
        // first place (see webkit.org/b/307004).
        auto* containingBlock = this->containingBlock();
        while (containingBlock && containingBlock->isAnonymousBlock())
            containingBlock = containingBlock->containingBlock();
        return containingBlock;
    };
    auto* containingBlock = containingBlockSkippingAnonymous();

    auto& style = this->style();
    auto& left = style.left();
    auto& right = style.right();
    auto& top = style.top();
    auto& bottom = style.bottom();

    LayoutSize offset;
    auto topFixed = top.tryFixed();
    auto leftFixed = left.tryFixed();
    if (topFixed && leftFixed && bottom.isAuto() && right.isAuto() && containingBlock->writingMode().isAnyLeftToRight()) {
        offset.expand(leftFixed->resolveZoom(style.usedZoomForLength()), topFixed->resolveZoom(style.usedZoomForLength()));
        return offset;
    }

    // Objects that shrink to avoid floats normally use available line width when computing containing block width. However
    // in the case of relative positioning using percentages, we can't do this. The offset should always be resolved using the
    // available width of the containing block. Therefore we don't use containingBlockLogicalWidthForContent() here, but instead explicitly
    // call availableWidth on our containing block.
    if (!left.isAuto() || !right.isAuto()) {
        auto availableWidth = [&] {
            auto* renderBox = dynamicDowncast<RenderBox>(*this);
            if (!renderBox || !renderBox->isGridItem())
                return containingBlock->contentBoxWidth();
            // For grid items the containing block is the grid area, so offsets should be resolved against that.
            auto containingBlockContentWidth = renderBox->gridAreaContentWidth(containingBlock->writingMode());
            if (!containingBlockContentWidth || !*containingBlockContentWidth) {
                ASSERT_NOT_REACHED();
                return containingBlock->contentBoxWidth();
            }
            return **containingBlockContentWidth;
        };
        if (!left.isAuto()) {
            if (!right.isAuto() && !containingBlock->writingMode().isAnyLeftToRight())
                offset.setWidth(-Style::evaluate<LayoutUnit>(right, !right.isFixed() ? availableWidth() : 0_lu, style.usedZoomForLength()));
            else
                offset.expand(Style::evaluate<LayoutUnit>(left, !left.isFixed() ? availableWidth() : 0_lu, style.usedZoomForLength()), 0_lu);
        } else if (!right.isAuto())
            offset.expand(-Style::evaluate<LayoutUnit>(right, !right.isFixed() ? availableWidth() : 0_lu, style.usedZoomForLength()), 0_lu);
    }

    // If the containing block of a relatively positioned element does not
    // specify a height, a percentage top or bottom offset should be resolved as
    // auto. An exception to this is if the containing block has the WinIE quirk
    // where <html> and <body> assume the size of the viewport. In this case,
    // calculate the percent offset based on this height.
    // See <https://bugs.webkit.org/show_bug.cgi?id=26396>.
    // Another exception is a grid item, as the containing block is the grid area:
    // https://drafts.csswg.org/css-grid/#grid-item-sizing
    if (top.isAuto() && bottom.isAuto())
        return offset;

    auto containingBlockHasDefiniteHeight = containingBlock->hasDefiniteLogicalHeightForPercentageResolutionFromStyle();
#if ASSERT_ENABLED
    verifyDefiniteHeightConsistencyBetweenStyleAndContainingBlockChain(*containingBlock, containingBlockHasDefiniteHeight);
#endif
    auto availableHeight = [&] {
        auto* renderBox = dynamicDowncast<RenderBox>(*this);
        if (!renderBox || !renderBox->isGridItem())
            return containingBlock->contentBoxHeight();
        // For grid items the containing block is the grid area, so offsets should be resolved against that.
        auto containingBlockContentHeight = renderBox->gridAreaContentHeight(containingBlock->style().writingMode());
        if (!containingBlockContentHeight || !*containingBlockContentHeight) {
            ASSERT_NOT_REACHED();
            return containingBlock->contentBoxHeight();
        }
        return **containingBlockContentHeight;
    };
    if (!top.isAuto() && (!top.isPercentOrCalculated() || containingBlockHasDefiniteHeight)) {
        // FIXME: The computation of the available height is repeated later for "bottom".
        // We could refactor this and move it to some common code for both ifs, however moving it outside of the ifs
        // is not possible as it'd cause performance regressions.
        offset.expand(0_lu, Style::evaluate<LayoutUnit>(top, !top.isFixed() ? availableHeight() : 0_lu, style.usedZoomForLength()));
    } else if (!bottom.isAuto() && (!bottom.isPercentOrCalculated() || containingBlockHasDefiniteHeight)) {
        // FIXME: Check comment above for "top", it applies here too.
        offset.expand(0_lu, -Style::evaluate<LayoutUnit>(bottom, !bottom.isFixed() ? availableHeight() : 0_lu, style.usedZoomForLength()));
    }
    return offset;
```
