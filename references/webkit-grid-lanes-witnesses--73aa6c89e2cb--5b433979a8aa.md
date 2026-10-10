# WebKit Grid Lanes source witnesses

Source pin: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Retrieved from the existing local checkout on 2026-10-10; no WebKit build or source execution. These are implementation witnesses for Surgeist #997/#998, not normative CSS consensus or a claim about every WebKit platform/backend.

GridLanesLayout.cpp is reproduced completely. The other files are explicitly bounded excerpts; each retains its complete original file-level legal header. Source paths, line ranges, hashes and immutable upstream links identify each excerpt. Added headings and omission boundaries are non-normative. No implementation code is incorporated into Surgeist production by this source capture.

Attribution: WebKit contributors, including the copyright owners named verbatim in the retained headers. The source headers state their applicable BSD 2-clause or LGPL 2-or-later; each declaration remains attached to its covered excerpt. Root maintains accompanying attribution and license inventories.

Manifest SHA-256: `5b433979a8aa0fbcca9cdf9d2f9080e321b888ea39a511e8a6d89fa65da87b89`. Exact source bytes and segment checks are recorded in the ignored effects provenance.

## `Source/WebCore/rendering/GridLanesLayout.cpp`

Original source: [Source/WebCore/rendering/GridLanesLayout.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/GridLanesLayout.cpp).

Scope: full source file, including auto-only cursor movement. Full original file-level copyright and license header is retained; line labels and omission notes are added by Surgeist.

Original lines 1–272:

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
#include "config.h"
#include "GridLanesLayout.h"

#include "GridLayoutFunctions.h"
#include "RenderBoxInlines.h"
#include "RenderGrid.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "StyleGridPositionsResolver.h"
#include "WritingMode.h"

namespace WebCore {

GridLanesLayout::GridLanesLayout(RenderGrid& renderGrid, unsigned gridAxisTracksCount, Style::GridTrackSizingDirection stackingAxisDirection)
    : m_runningPositions(gridAxisTracksCount)
    , m_renderGrid(renderGrid)
    , m_stackingAxisGridGap(renderGrid.gridGap(stackingAxisDirection))
    , m_stackingAxisDirection(stackingAxisDirection)
{
    m_renderGrid->currentGrid().setupForGridLanesLayout();
    m_renderGrid->populateExplicitGridAndOrderIterator();
}

GridLanesResult GridLanesLayout::performGridLanesPlacement(const GridTrackSizingAlgorithm& algorithm, ResolvedFitTolerance fitTolerance, Phase layoutPhase)
{
    // 4.4 Grid Lanes Layout and Placement Algorithm
    // https://drafts.csswg.org/css-grid-3/#grid-lanes-layout-algorithm
    return placeGridLanesItems(algorithm, fitTolerance, layoutPhase);
}

GridLanesResult GridLanesLayout::placeGridLanesItems(const GridTrackSizingAlgorithm& algorithm, ResolvedFitTolerance fitTolerance, Phase layoutPhase)
{
    if (!gridAxisTracksCount())
        return { };

    HashMap<SingleThreadWeakRef<const RenderBox>, LayoutUnit> stackingAxisOffsets;
    LayoutUnit gridContentSize;

    auto& grid = m_renderGrid->currentGrid();
    for (CheckedRef gridItem : grid.orderIterator().gridItems()) {
        bool isAutoPlacedInGridAxis = !hasDefiniteGridAxisPosition(gridItem, gridAxisDirection());
        auto gridArea = isAutoPlacedInGridAxis ? gridAreaForIndefiniteGridAxisItem(gridItem, fitTolerance) : gridAreaForDefiniteGridAxisItem(gridItem);
        auto placement = insertIntoGridAndLayoutItem(algorithm, gridItem, gridArea, layoutPhase);

        if (isAutoPlacedInGridAxis)
            m_autoFlowNextCursor = gridAxisSpanFromArea(gridArea).endLine() % gridAxisTracksCount();

        stackingAxisOffsets.set(gridItem.get(), placement.marginBoxStart);
        gridContentSize = std::max(gridContentSize, placement.marginBoxEnd);
    }

    return { WTF::move(stackingAxisOffsets), gridContentSize };
}

GridArea GridLanesLayout::gridAreaForDefiniteGridAxisItem(const RenderBox& gridItem) const
{
    auto itemSpan = m_renderGrid->currentGrid().gridItemSpan(gridItem, gridAxisDirection());
    ASSERT(!itemSpan.isIndefinite());
    itemSpan.translate(m_renderGrid->currentGrid().explicitGridStart(gridAxisDirection()));
    return gridAreaFromGridAxisSpan(itemSpan);
}

LayoutUnit GridLanesLayout::calculateGridLanesIntrinsicLogicalWidth(RenderBox& gridItem, Phase layoutPhase)
{
    switch (layoutPhase) {
    case Phase::MinContent:
        return gridItem.computeSizingKeywordLogicalWidthUsing(CSS::Keyword::MinContent { }, { }, gridItem.borderAndPaddingLogicalWidth());
    case Phase::MaxContent:
        return gridItem.computeSizingKeywordLogicalWidthUsing(CSS::Keyword::MaxContent { }, { }, gridItem.borderAndPaddingLogicalWidth());
    case Phase::Layout:
        ASSERT_NOT_REACHED();
        return { };
    }

    return { };
}

void GridLanesLayout::setItemContainingBlockToGridArea(const GridTrackSizingAlgorithm& algorithm, RenderBox& gridItem)
{
    CheckedPtr<RenderGrid> containingBlock = dynamicDowncast<RenderGrid>(gridItem.containingBlock());
    if (!containingBlock) {
        ASSERT_NOT_REACHED();
        return;
    }

    // FIXME: We need to set both axes here because RenderGrid sets and expects them all over the place.
    // Ideally we untangle all that and only set the grid axis that we need. webkit.org/b/305136
    auto direction = gridAxisDirection();
    if (direction == Style::GridTrackSizingDirection::Columns) {
        gridItem.setGridAreaContentLogicalWidth(algorithm.gridAreaBreadthForGridItem(gridItem, direction));
        gridItem.setGridAreaContentLogicalHeight(containingBlock->availableLogicalHeightForContentBox());
    } else {
        gridItem.setGridAreaContentLogicalHeight(algorithm.gridAreaBreadthForGridItem(gridItem, direction));
        gridItem.setGridAreaContentLogicalWidth(containingBlock->contentBoxLogicalWidth());
    }

    // FIXME(249230): Try to cache grid lanes layout sizes
    gridItem.setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);
}

GridLanesLayout::StackingAxisPlacement GridLanesLayout::insertIntoGridAndLayoutItem(const GridTrackSizingAlgorithm& algorithm, RenderBox& gridItem, const GridArea& area, Phase layoutPhase)
{
    auto shouldOverrideLogicalWidth = [&](RenderBox& gridItem, Phase layoutPhase) {
        if (layoutPhase == Phase::Layout)
            return false;

        if (!(gridItem.style().logicalWidth().isAuto() || gridItem.style().logicalWidth().isPercent()))
            return false;

        ASSERT(m_renderGrid->isStackingAxis(Style::GridTrackSizingDirection::Columns));

        if (gridItem.style().writingMode().isOrthogonal(m_renderGrid->style().writingMode()))
            return false;

        if (auto* renderGrid = dynamicDowncast<RenderGrid>(gridItem); renderGrid && renderGrid->isSubgridRows())
            return false;

        return true;
    };

    if (shouldOverrideLogicalWidth(gridItem, layoutPhase))
        gridItem.setOverridingBorderBoxLogicalWidth(calculateGridLanesIntrinsicLogicalWidth(gridItem, layoutPhase));

    m_renderGrid->currentGrid().insert(gridItem, area);
    setItemContainingBlockToGridArea(algorithm, gridItem);
    gridItem.layoutIfNeeded();
    return updateRunningPositions(gridItem, area);
}

LayoutUnit GridLanesLayout::stackingAxisMarginBoxForItem(const RenderBox& gridItem)
{
    LayoutUnit marginBoxSize;
    if (m_stackingAxisDirection == Style::GridTrackSizingDirection::Rows) {
        if (GridLayoutFunctions::isOrthogonalGridItem(m_renderGrid, gridItem))
            marginBoxSize = gridItem.isHorizontalWritingMode() ? gridItem.borderBoxWidth() + gridItem.horizontalMarginExtent() : gridItem.borderBoxHeight() + gridItem.verticalMarginExtent();
        else
            marginBoxSize = gridItem.logicalHeight() + gridItem.marginLogicalHeight();

    } else {
        if (GridLayoutFunctions::isOrthogonalGridItem(m_renderGrid, gridItem))
            marginBoxSize = gridItem.isHorizontalWritingMode() ? gridItem.borderBoxHeight() + gridItem.verticalMarginExtent() : gridItem.borderBoxWidth() + gridItem.horizontalMarginExtent();
        else
            marginBoxSize = gridItem.logicalWidth() + gridItem.marginLogicalWidth();
    }
    return marginBoxSize;
}

GridLanesLayout::StackingAxisPlacement GridLanesLayout::updateRunningPositions(const RenderBox& gridItem, const GridArea& area)
{
    auto gridAxisSpan = gridAxisSpanFromArea(area);
    ASSERT(gridAxisSpan.startLine() < m_runningPositions.size() && gridAxisSpan.endLine() <= m_runningPositions.size());
    gridAxisSpan.clamp(m_runningPositions.size());

    LayoutUnit previousRunningPosition;
    for (auto line : gridAxisSpan)
        previousRunningPosition = std::max(previousRunningPosition, m_runningPositions[line]);

    auto marginBoxEnd = previousRunningPosition + stackingAxisMarginBoxForItem(gridItem);

    for (auto span : gridAxisSpan)
        m_runningPositions[span] = marginBoxEnd + m_stackingAxisGridGap;

    return { previousRunningPosition, marginBoxEnd };
}

LayoutUnit GridLanesLayout::maxRunningPositionForSpan(unsigned startLine, unsigned spanLength) const
{
    LayoutUnit maxPosition;
    for (unsigned lineOffset = 0; lineOffset < spanLength; lineOffset++)
        maxPosition = std::max(maxPosition, m_runningPositions[startLine + lineOffset]);
    return maxPosition;
}

GridArea GridLanesLayout::gridAreaForIndefiniteGridAxisItem(const RenderBox& item, ResolvedFitTolerance fitTolerance)
{
    auto itemSpanLength = std::min<unsigned>(Style::GridPositionsResolver::spanSizeForAutoPlacedItem(item, gridAxisDirection()), gridAxisTracksCount());
    auto gridAxisLines = gridAxisTracksCount() + 1;

    if (WTF::holdsAlternative<CSS::Keyword::Infinite>(fitTolerance)) {
        // Infinite tolerance: place items strictly in order without considering track lengths
        // Use round-robin placement starting from the cursor position
        auto startingLine = m_autoFlowNextCursor;

        // If the item doesn't fit at the cursor position, wrap to the beginning
        if (startingLine + itemSpanLength > gridAxisTracksCount())
            startingLine = 0;

        auto gridAxisPosition = GridSpan::translatedDefiniteGridSpan(startingLine, startingLine + itemSpanLength);
        return gridAreaFromGridAxisSpan(gridAxisPosition);
    }

    // For normal and length-percentage tolerances, find positions within tolerance of the shortest track
    auto toleranceValue = std::get<LayoutUnit>(fitTolerance);

    // Step 1: Find the absolute shortest position across all tracks
    auto maxStartingLine = gridAxisLines - itemSpanLength;
    LayoutUnit absoluteShortest = LayoutUnit::max();
    for (unsigned i = 0; i < maxStartingLine; i++)
        absoluteShortest = std::min(absoluteShortest, maxRunningPositionForSpan(i, itemSpanLength));

    // Step 2: Find first position within tolerance of shortest, starting from the cursor position.
    unsigned smallestMaxPosLine = 0;
    auto autoFlowNextCursorShift = (m_autoFlowNextCursor > maxStartingLine) ? 0 : m_autoFlowNextCursor;
    for (unsigned i = 0; i < maxStartingLine; i++) {
        auto startingLine = (autoFlowNextCursorShift + i) % maxStartingLine;

        auto maxPosForCurrentStartingLine = maxRunningPositionForSpan(startingLine, itemSpanLength);

        // Accept first position within tolerance of the absolute shortest
        if (maxPosForCurrentStartingLine <= absoluteShortest + toleranceValue) {
            smallestMaxPosLine = startingLine;
            break;
        }
    }

    auto gridAxisPosition = GridSpan::translatedDefiniteGridSpan(smallestMaxPosLine, smallestMaxPosLine + itemSpanLength);
    return gridAreaFromGridAxisSpan(gridAxisPosition);
}

LayoutUnit GridLanesResult::stackingAxisOffsetForGridItem(const RenderBox& gridItem) const
{
    return m_stackingAxisOffsets.getOptional(gridItem).value_or(0_lu);
}

inline Style::GridTrackSizingDirection GridLanesLayout::gridAxisDirection() const
{
    // The stacking axis and grid axis can never be the same.
    // They are always perpendicular to each other.
    return orthogonalDirection(m_stackingAxisDirection);
}

bool GridLanesLayout::hasDefiniteGridAxisPosition(const RenderBox& gridItem, Style::GridTrackSizingDirection gridAxisDirection) const
{
    return !Style::GridPositionsResolver::resolveGridPositionsFromStyle(m_renderGrid, gridItem, gridAxisDirection).isIndefinite();
}

GridSpan GridLanesLayout::gridAxisSpanFromArea(const GridArea& gridArea) const
{
    return gridArea.span(gridAxisDirection());
}

GridArea GridLanesLayout::gridAreaFromGridAxisSpan(const GridSpan& gridAxisSpan) const
{
    auto stackingAxisSpan = GridSpan::stackingAxisTranslatedDefiniteGridSpan();
    return m_stackingAxisDirection == Style::GridTrackSizingDirection::Rows
        ? GridArea { stackingAxisSpan, gridAxisSpan }
        : GridArea { gridAxisSpan, stackingAxisSpan };
}
} // end namespace WebCore
```

## `Source/WebCore/style/StyleAdjuster.cpp`

Original source: [Source/WebCore/style/StyleAdjuster.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/StyleAdjuster.cpp).

Scope: bounded normal grid-auto-flow adjustment, not the whole adjuster function. Full original file-level copyright and license header is retained; line labels and omission notes are added by Surgeist.

Original lines 1–28:

```cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004-2005 Allan Sandfeld Jensen (kde@carewolf.com)
 * Copyright (C) 2006, 2007 Nicholas Shanks (webkit@nickshanks.com)
 * Copyright (C) 2005-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2007 Alexey Proskuryakov <ap@webkit.org>
 * Copyright (C) 2007, 2008 Eric Seidel <eric@webkit.org>
 * Copyright (C) 2008, 2009 Torch Mobile Inc. All rights reserved. (http://www.torchmobile.com/)
 * Copyright (c) 2011, Code Aurora Forum. All rights reserved.
 * Copyright (C) Research In Motion Limited 2011. All rights reserved.
 * Copyright (C) 2012-2020 Google Inc. All rights reserved.
 * Copyright (C) 2014, 2020, 2022 Igalia S.L.
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
```

Original lines 508–520:

```cpp
        auto adjustGridAutoFlow = [&](GridAutoFlow::Direction direction) {
            if (auto gridAutoFlow = style.gridAutoFlow(); gridAutoFlow.direction() == GridAutoFlow::Direction::Normal) {
                gridAutoFlow.setDirection(direction);
                style.setGridAutoFlow(gridAutoFlow);
            }
        };
        if (display.isGridBox())
            adjustGridAutoFlow(GridAutoFlow::Direction::Row);
        else if (display.isGridLanesBox()) {
            auto direction = (!style.gridTemplateRows().isNone() && style.gridTemplateColumns().isNone())
                ? GridAutoFlow::Direction::Column : GridAutoFlow::Direction::Row;
            adjustGridAutoFlow(direction);
        }
```

## `Source/WebCore/rendering/RenderGrid.cpp`

Original source: [Source/WebCore/rendering/RenderGrid.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderGrid.cpp).

Scope: complete isStackingAxis and prepareGridItemForPositionedLayout functions with adjacent orientation context. Full original file-level copyright and license header is retained; line labels and omission notes are added by Surgeist.

Original lines 1–25:

```cpp
/*
 * Copyright (C) 2011, 2022 Apple Inc. All rights reserved.
 * Copyright (C) 2013-2017 Igalia S.L.
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

Original lines 1182–1200:

```cpp
// Grid Lanes Spec Section 2 Grid Lanes Layout Model
// https://drafts.csswg.org/css-grid-3/#grid-lanes-model
// The stacking axis is the axis that items are stacked along; the orthogonal axis is
// the grid axis, which establishes the tracks. Which axis is which is determined per
// Section 2.3 Orienting Grid Lanes Layout, which we key off grid-auto-flow.
// https://drafts.csswg.org/css-grid-3/#grid-lanes-orientation
// Note that a subgrid defers to its parent before we check display, so a display:grid
// subgrid of a grid lanes container can answer true here.
bool RenderGrid::isStackingAxis(Style::GridTrackSizingDirection direction) const
{
    // isSubgrid will return false if the stacking axis matches. Need to check style if we are a subgrid
    auto& tracks = style().gridTemplateList(direction);
    if (auto* parentGrid = dynamicDowncast<RenderGrid>(parent()); parentGrid && tracks.subgrid)
        return parentGrid->isStackingAxis(direction);
    if (style().display() != Style::DisplayType::BlockGridLanes && style().display() != Style::DisplayType::InlineGridLanes)
        return false;
    return (direction == Style::GridTrackSizingDirection::Columns) == style().gridAutoFlow().isColumn();
}
```

Original lines 1723–1733:

```cpp

void RenderGrid::prepareGridItemForPositionedLayout(RenderBox& gridItem)
{
    ASSERT(gridItem.isOutOfFlowPositioned());
    gridItem.containingBlock()->addOutOfFlowBox(gridItem);

    CheckedPtr gridItemLayer = gridItem.layer();
    // Static position of a positioned grid item should use the content-box (https://drafts.csswg.org/css-grid/#static-position).
    gridItemLayer->setStaticInlinePosition(borderAndPaddingStart());
    gridItemLayer->setStaticBlockPosition(borderAndPaddingBefore());
}
```
