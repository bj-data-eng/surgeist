# WebKit: table sizing source witnesses

This reference preserves four complete source files at immutable WebKit revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`, including each original file-specific license header. These witnesses document fixed/automatic sizing and row/span/extra-height allocation for bounded comparison with CSS2 and the newer Tables draft. They are reference material, not a whole-engine compatibility claim or an adoption of every implementation detail.

Original copyright and GNU Library General Public License notices remain in each source block. Local license text: [LGPL version 2](../licenses/webkit/LICENSE-LGPL-2.txt). No implementation code is added to Surgeist by this reference.

## AutoTableLayout.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/AutoTableLayout.cpp). Path: `Source/WebCore/rendering/AutoTableLayout.cpp`. Source bytes: 44341; source lines: 871; SHA-256: `55361aef86046fa85c371400b7e4c3a7fdd8acde0678abc25894b99dd545240f`.

```cpp
/*
 * Copyright (C) 2002 Lars Knoll (knoll@kde.org)
 *           (C) 2002 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2003-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2014-2017 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License.
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

#include "config.h"
#include "AutoTableLayout.h"

#include "RenderBoxInlines.h"
#include "RenderChildIterator.h"
#include "RenderFlexibleBox.h"
#include "RenderGrid.h"
#include "RenderTableCellInlines.h"
#include "RenderTableCol.h"
#include "RenderTableInlines.h"
#include "RenderTableSection.h"
#include "RenderView.h"
#include "StylePreferredSize.h"
#include "StylePrimitiveNumericTypes+EvaluationMinimum.h"

namespace WebCore {

AutoTableLayout::AutoTableLayout(RenderTable* table)
    : TableLayout(table)
{
}

AutoTableLayout::~AutoTableLayout() = default;

bool AutoTableLayout::isColumnCollapsed(unsigned effCol) const
{
    CheckedPtr colElement = m_table->colElement(effCol);
    return colElement && colElement->style().visibility() == Visibility::Collapse;
}

void AutoTableLayout::recalcColumn(unsigned effCol)
{
    Layout& columnLayout = m_layoutStruct[effCol];

    // Check if this column is collapsed.
    if (isColumnCollapsed(effCol)) {
        columnLayout.effectiveLogicalWidth = CSS::Keyword::Auto { };
        columnLayout.effectiveMinLogicalWidth = 0;
        columnLayout.effectiveMaxLogicalWidth = 0;
        columnLayout.minLogicalWidth = 0;
        columnLayout.maxLogicalWidth = 0;
        columnLayout.logicalWidth = CSS::Keyword::Auto { };
        return;
    }

    RenderTableCell* fixedContributor = nullptr;
    RenderTableCell* maxContributor = nullptr;

    for (auto& child : childrenOfType<RenderObject>(*m_table)) {
        if (CheckedPtr column = dynamicDowncast<RenderTableCol>(child)) {
            // RenderTableCols don't have the concept of preferred logical width, but we need to clear their dirty bits
            // so that if we call setContentWidthsDirty(true) on a col or one of its descendants, we'll mark its
            // ancestors as dirty.
            column->clearContentLogicalWidthsInvalidation();
        } else if (CheckedPtr section = dynamicDowncast<RenderTableSection>(child)) {
            unsigned numRows = section->numRows();
            for (unsigned i = 0; i < numRows; ++i) {
                auto current = section->cellAt(i, effCol);
                auto* cell = current.primaryCell();
                
                if (current.inColSpan || !cell)
                    continue;

                bool cellHasContent = cell->firstChild()
                    || cell->style().border().hasBorder()
                    || !Style::isKnownZero(cell->style().paddingBox())
                    || cell->style().hasBackground();
                if (cellHasContent)
                    columnLayout.emptyCellsOnly = false;

                // A cell originates in this column. Ensure we have
                // a min/max width of at least 1px for this column now.
                columnLayout.minLogicalWidth = std::max(columnLayout.minLogicalWidth, 0.f);
                columnLayout.maxLogicalWidth = std::max(columnLayout.maxLogicalWidth, 0.f);

                if (cell->isOrthogonal())
                    cell->clearIntrinsicPadding();

                if (cell->colSpan() == 1) {
                    columnLayout.minLogicalWidth = std::max(cell->minLogicalWidthForColumnSizing().ceilToFloat(), columnLayout.minLogicalWidth);
                    float maxPreferredWidth = cell->maxLogicalWidthForColumnSizing().ceilToFloat();
                    if (maxPreferredWidth > columnLayout.maxLogicalWidth) {
                        columnLayout.maxLogicalWidth = maxPreferredWidth;
                        maxContributor = cell;
                    }

                    // All browsers implement a size limit on the cell's max width. 
                    // Our limit is based on KHTML's representation that used 16 bits widths.
                    // FIXME: Other browsers have a lower limit for the cell's max width. 
                    const float cCellMaxWidth = 32760;
                    auto [ cellLogicalWidth, cellUsedZoom ] = cell->styleOrColLogicalWidth();
                    if (auto fixedCellLogicalWidth = cellLogicalWidth.tryFixed()) {
                        if (fixedCellLogicalWidth->resolveZoom(cellUsedZoom) > cCellMaxWidth)
                            cellLogicalWidth = Style::PreferredSize::Fixed { cCellMaxWidth };
                    }
                    WTF::switchOn(cellLogicalWidth,
                        [&](const Style::PreferredSize::Fixed& fixedCellLogicalWidth) {
                            if (fixedCellLogicalWidth.isPositiveOrZero() && !columnLayout.logicalWidth.isPercentOrCalculated()) {
                                float logicalWidth = cell->adjustBorderBoxLogicalWidthForBoxSizing(fixedCellLogicalWidth);
                                // Honor the cell's CSS max-width constraint.
                                if (auto fixedMaxWidth = cell->style().logicalMaxWidth().tryFixed())
                                    logicalWidth = std::min(logicalWidth, cell->adjustBorderBoxLogicalWidthForBoxSizing(*fixedMaxWidth).toFloat());
                                if (auto fixedColumnLayoutLogicalWidth = columnLayout.logicalWidth.tryFixed()) {
                                    // Nav/IE weirdness
                                    if ((logicalWidth > fixedColumnLayoutLogicalWidth->resolveZoom(cellUsedZoom))
                                        || ((fixedColumnLayoutLogicalWidth->resolveZoom(cellUsedZoom) == logicalWidth) && (maxContributor == cell))) {
                                        columnLayout.logicalWidth = Style::PreferredSize::Fixed { logicalWidth };
                                        fixedContributor = cell;
                                    }
                                } else {
                                    columnLayout.logicalWidth = Style::PreferredSize::Fixed { logicalWidth };
                                    fixedContributor = cell;
                                }
                            }
                        },
                        [&](const Style::PreferredSize::Percentage& percentageCellLogicalWidth) {
                            m_hasPercent = true;
                            if (auto percentageColumnLayoutLogicalWidth = columnLayout.logicalWidth.tryPercentage(); percentageCellLogicalWidth.value > 0 && (!percentageColumnLayoutLogicalWidth || percentageCellLogicalWidth.value > percentageColumnLayoutLogicalWidth->value))
                                columnLayout.logicalWidth = cellLogicalWidth;
                        },
                        [&](const Style::PreferredSize::Calc&) {
                            columnLayout.logicalWidth = CSS::Keyword::Auto { };
                        },
                        [&](const auto&) { }
                    );
                } else if (!effCol || section->primaryCellAt(i, effCol - 1) != cell) {
                    // If a cell originates in this spanning column ensure we have a min/max width of at least 1px for it.
                    columnLayout.minLogicalWidth = std::max(columnLayout.minLogicalWidth, cell->maxLogicalWidthForColumnSizing() ? 1.f : 0.f);

                    // This spanning cell originates in this column. Insert the cell into spanning cells list.
                    insertSpanCell(cell);
                }
            }
        }
    }

    // Nav/IE weirdness
    if (auto fixedColumnLayoutLogicalWidth = columnLayout.logicalWidth.tryFixed()) {
        if (m_table->document().inQuirksMode() && columnLayout.maxLogicalWidth > fixedColumnLayoutLogicalWidth->resolveZoom(Style::ZoomFactor { columnLayout.usedZoom })
        && fixedContributor != maxContributor) {
            columnLayout.logicalWidth = CSS::Keyword::Auto { };
            fixedContributor = nullptr;
        }
    }

    columnLayout.maxLogicalWidth = std::max(columnLayout.maxLogicalWidth, columnLayout.minLogicalWidth);
}

void AutoTableLayout::fullRecalc()
{
    m_hasPercent = false;
    m_effectiveLogicalWidthDirty = true;

    unsigned nEffCols = m_table->numEffCols();
    m_layoutStruct.resizeToFit(nEffCols);
    m_layoutStruct.fill(Layout());
    m_spanCells.fill(0);

    Style::PreferredSize groupLogicalWidth = CSS::Keyword::Auto { };
    unsigned currentColumn = 0;
    for (RenderTableCol* column = m_table->firstColumn(); column; column = column->nextColumn()) {
        if (column->isTableColumnGroupWithColumnChildren())
            groupLogicalWidth = column->style().logicalWidth();
        else {
            auto colLogicalWidth = column->style().logicalWidth();
            // FIXME: calc() on tables should be handled consistently with other lengths.
            if (colLogicalWidth.isCalculated() || colLogicalWidth.isAuto())
                colLogicalWidth = groupLogicalWidth;
            if (colLogicalWidth.isSpecified() && colLogicalWidth.isKnownZero())
                colLogicalWidth = CSS::Keyword::Auto { };
            unsigned span = column->span();

            // Apply width to all columns covered by this col element.
            if (!colLogicalWidth.isAuto()) {
                for (unsigned spanOffset = 0; spanOffset < span; ++spanOffset) {
                    unsigned effCol = m_table->colToEffCol(currentColumn + spanOffset);
                    if (effCol < nEffCols && m_table->spanOfEffCol(effCol) == 1) {
                        m_layoutStruct[effCol].usedZoom = column->style().usedZoom();
                        m_layoutStruct[effCol].logicalWidth = colLogicalWidth;
                        if (auto fixedColLogicalWidth = colLogicalWidth.tryFixed(); fixedColLogicalWidth && m_layoutStruct[effCol].maxLogicalWidth < fixedColLogicalWidth->resolveZoom(column->style().usedZoomForLength()))
                            m_layoutStruct[effCol].maxLogicalWidth = fixedColLogicalWidth->resolveZoom(column->style().usedZoomForLength());
                    }
                }
            }
            currentColumn += span;
        }

        // For the last column in a column-group, we invalidate our group logical width.
        if (column->isTableColumn() && !column->nextSibling())
            groupLogicalWidth = CSS::Keyword::Auto { };
    }

    for (unsigned i = 0; i < nEffCols; i++)
        recalcColumn(i);

    for (auto& section : childrenOfType<RenderTableSection>(*m_table)) {
        section.clearContentLogicalWidthsInvalidation();
        for (auto* row = section.firstRow(); row; row = row->nextRow())
            row->clearContentLogicalWidthsInvalidation();
    }
}

static bool shouldScaleColumnsForParent(const RenderTable& table)
{
    RenderBlock* containingBlock = table.containingBlock();
    while (containingBlock && !is<RenderView>(containingBlock)) {
        // It doesn't matter if our table is auto or fixed: auto means we don't
        // scale. Fixed doesn't care if we do or not because it doesn't depend
        // on the cell contents' preferred widths.
        if (is<RenderTableCell>(containingBlock))
            return false;
        // The max logical width of a table may be "infinity" (or tableMaxWidth, to be more exact) if the sum if the
        // columns' percentages is 100% or more, AND there is at least one column that has a non-percentage-based positive
        // logical width. In such situations no table logical width will be large enough to satisfy the constraint
        // set by the contents. So the idea is to use ~infinity to make sure we use all available size in the containing
        // block. However, this just doesn't work if this is a flex or grid item, so disallow scaling in that case.
        if (isAnyOf<RenderFlexibleBox, RenderGrid>(containingBlock))
            return false;
        containingBlock = containingBlock->containingBlock();
    }
    return true;
}

std::pair<LayoutUnit, LayoutUnit> AutoTableLayout::computeIntrinsicLogicalWidths(TableIntrinsics intrinsics)
{
    fullRecalc();

    auto minWidth = LayoutUnit { };
    auto maxWidth = LayoutUnit { };

    float spanMaxLogicalWidth = calcEffectiveLogicalWidth();
    float maxPercent = 0;
    float maxNonPercent = 0;
    bool scaleColumnsForSelf = intrinsics == TableIntrinsics::ForLayout;

    float remainingPercent = 100;
    for (size_t i = 0; i < m_layoutStruct.size(); ++i) {
        minWidth += m_layoutStruct[i].effectiveMinLogicalWidth;
        maxWidth += m_layoutStruct[i].effectiveMaxLogicalWidth;
        if (scaleColumnsForSelf) {
            if (auto percentageEffectiveLogicalWidth = m_layoutStruct[i].effectiveLogicalWidth.tryPercentage()) {
                float percent = std::min(percentageEffectiveLogicalWidth->value, remainingPercent);
                // When percent columns meet or exceed 100% and there are remaining
                // columns, the other browsers (FF, Edge) use an artificially high max
                // width, so we do too. Instead of division by zero, logicalWidth and
                // maxNonPercent are set to tableMaxWidth.
                // Issue: https://github.com/w3c/csswg-drafts/issues/1501
                float logicalWidth = (percent > 0) ? m_layoutStruct[i].effectiveMaxLogicalWidth * 100 / percent : tableMaxWidth;
                maxPercent = std::max(logicalWidth,  maxPercent);
                remainingPercent -= percent;
            } else
                maxNonPercent += m_layoutStruct[i].effectiveMaxLogicalWidth;
        }
    }

    if (scaleColumnsForSelf) {
        if (maxNonPercent > 0)
            maxNonPercent = (remainingPercent > 0) ? maxNonPercent * 100 / remainingPercent : tableMaxWidth;
        m_scaledWidthFromPercentColumns = std::min(LayoutUnit(tableMaxWidth), LayoutUnit(std::max(maxPercent, maxNonPercent)));
        if (m_scaledWidthFromPercentColumns > maxWidth && shouldScaleColumnsForParent(*m_table))
            maxWidth = m_scaledWidthFromPercentColumns;
    }

    if (intrinsics == TableIntrinsics::ForKeyword && m_layoutStruct.isEmpty()) {
        ASSERT(!minWidth);
        ASSERT(!maxWidth);
        minWidth = m_table->bordersPaddingAndSpacingInRowDirection();
        maxWidth = minWidth;
    }

    maxWidth = std::max(maxWidth, LayoutUnit(spanMaxLogicalWidth));
    return { minWidth, maxWidth };
}

void AutoTableLayout::applyContentLogicalWidthQuirks(LayoutUnit& minWidth, LayoutUnit& maxWidth) const
{
    if (auto fixedTableLogicalWidth = m_table->style().logicalWidth().tryFixed(); fixedTableLogicalWidth && fixedTableLogicalWidth->isPositive()) {
        LayoutUnit minContentWidth = minWidth;
        LayoutUnit tableFixedWidth = m_table->overridingBorderBoxLogicalWidth().value_or(LayoutUnit { fixedTableLogicalWidth->resolveZoom(m_table->style().usedZoomForLength()) });
        LayoutUnit clampedWidth = std::max(minContentWidth, std::max(minWidth, tableFixedWidth));
        minWidth = clampedWidth;
        maxWidth = clampedWidth;

        if (auto fixedMaxWidth = m_table->style().logicalMaxWidth().tryFixed()) {
            LayoutUnit maxAllowedWidth { fixedMaxWidth->resolveZoom(m_table->style().usedZoomForLength()) };
            minWidth = std::min(minWidth, maxAllowedWidth);
            minWidth = std::max(minWidth, minContentWidth);
            maxWidth = minWidth;
        }
    }
}

/*
  This method takes care of colspans.
  effWidth is the same as width for cells without colspans. If we have colspans, they get modified.
 */
float AutoTableLayout::calcEffectiveLogicalWidth()
{
    float maxLogicalWidth = 0;

    size_t nEffCols = m_layoutStruct.size();
    float spacingInRowDirection = m_table->hBorderSpacing();

    for (size_t i = 0; i < nEffCols; ++i) {
        m_layoutStruct[i].effectiveLogicalWidth = m_layoutStruct[i].logicalWidth;
        m_layoutStruct[i].effectiveMinLogicalWidth = m_layoutStruct[i].minLogicalWidth;
        m_layoutStruct[i].effectiveMaxLogicalWidth = m_layoutStruct[i].maxLogicalWidth;
    }

    for (size_t i = 0; i < m_spanCells.size(); ++i) {
        RenderTableCell* cell = m_spanCells[i];
        if (!cell)
            break;

        unsigned span = cell->colSpan();

        auto [ cellLogicalWidth, cellUsedZoom ] = cell->styleOrColLogicalWidth();
        if (cellLogicalWidth.isKnownZero())
            cellLogicalWidth = CSS::Keyword::Auto { };

        unsigned effCol = m_table->colToEffCol(cell->col());
        size_t lastCol = effCol;
        if (cell->isOrthogonal())
            cell->clearIntrinsicPadding();
        float cellMinLogicalWidth = cell->minLogicalWidthForColumnSizing() + spacingInRowDirection;
        float cellMaxLogicalWidth = cell->maxLogicalWidthForColumnSizing() + spacingInRowDirection;
        float totalPercent = 0;
        float spanMinLogicalWidth = 0;
        float spanMaxLogicalWidth = 0;
        bool allColsArePercent = true;
        bool allColsAreFixed = true;
        bool haveAuto = false;
        bool spanHasEmptyCellsOnly = true;
        float fixedWidth = 0;
        while (lastCol < nEffCols && span > 0) {
            auto& columnLayout = m_layoutStruct[lastCol];

            auto fallbackCase = [&] {
                // If the column is a percentage width, do not let the spanning cell overwrite the
                // width value.  This caused a mis-rendering on amazon.com.
                // Sample snippet:
                // <table border=2 width=100%><
                //   <tr><td>1</td><td colspan=2>2-3</tr>
                //   <tr><td>1</td><td colspan=2 width=100%>2-3</td></tr>
                // </table>
                if (auto percentageEffectiveLogicalWidth = columnLayout.effectiveLogicalWidth.tryPercentage())
                    totalPercent += percentageEffectiveLogicalWidth->value;
                else {
                    columnLayout.effectiveLogicalWidth = CSS::Keyword::Auto { };
                    allColsArePercent = false;
                }
                allColsAreFixed = false;
            };

            WTF::switchOn(columnLayout.logicalWidth,
                [&](const Style::PreferredSize::Percentage& percentage) {
                    totalPercent += percentage.value;
                    allColsAreFixed = false;
                },
                [&](const Style::PreferredSize::Fixed& fixed) {
                    if (fixed.isPositive()) {
                        fixedWidth += fixed.resolveZoom(Style::ZoomFactor { columnLayout.usedZoom });
                        allColsArePercent = false;
                        // IE resets effWidth to Auto here, but this breaks the konqueror about page and seems to be some bad
                        // legacy behavior anyway. mozilla doesn't do this so I decided we don't neither.
                        return;
                    }
                    haveAuto = true;
                    fallbackCase();
                },
                [&](const CSS::Keyword::Auto&) {
                    haveAuto = true;
                    fallbackCase();
                },
                [&](const auto&) {
                    fallbackCase();
                }
            );
            if (!columnLayout.emptyCellsOnly)
                spanHasEmptyCellsOnly = false;
            span -= m_table->spanOfEffCol(lastCol);
            spanMinLogicalWidth += columnLayout.effectiveMinLogicalWidth;
            spanMaxLogicalWidth += columnLayout.effectiveMaxLogicalWidth;
            lastCol++;
            cellMinLogicalWidth -= spacingInRowDirection;
            cellMaxLogicalWidth -= spacingInRowDirection;
        }

        // adjust table max width if needed
        if (auto percentageCellLogicalWidth = cellLogicalWidth.tryPercentage()) {
            if (totalPercent > percentageCellLogicalWidth->value || allColsArePercent) {
                // can't satisfy this condition, treat as variable
                cellLogicalWidth = CSS::Keyword::Auto { };
            } else {
                maxLogicalWidth = std::max(maxLogicalWidth, std::max(spanMaxLogicalWidth, cellMaxLogicalWidth) * 100  / percentageCellLogicalWidth->value);

                // all non percent columns in the span get percent values to sum up correctly.
                float percentMissing = percentageCellLogicalWidth->value - totalPercent;
                float totalWidth = 0;
                for (unsigned pos = effCol; pos < lastCol; ++pos) {
                    if (!m_layoutStruct[pos].effectiveLogicalWidth.isPercentOrCalculated())
                        totalWidth += m_layoutStruct[pos].effectiveMaxLogicalWidth;
                }

                for (unsigned pos = effCol; pos < lastCol; ++pos) {
                    if (!m_layoutStruct[pos].effectiveLogicalWidth.isPercentOrCalculated()) {
                        // Handle the case when there's only one cell with 'width: percent' and it's empty.
                        auto percent = percentMissing * (totalWidth ? m_layoutStruct[pos].effectiveMaxLogicalWidth / totalWidth : 1);
                        totalWidth -= m_layoutStruct[pos].effectiveMaxLogicalWidth;
                        percentMissing -= percent;
                        if (percent > 0)
                            m_layoutStruct[pos].effectiveLogicalWidth = Style::PreferredSize::Percentage { percent };
                        else
                            m_layoutStruct[pos].effectiveLogicalWidth = CSS::Keyword::Auto { };
                    }
                    if (totalWidth <= 0)
                        break;
                }
            }
        }

        // Distribute the spanning cell's min/max widths across [effCol, lastCol) in proportion to
        // each column's percentage, using the given total as the denominator. percentForColumn
        // returns std::nullopt for columns that should be skipped.
        auto distributeByPercent = [&](float totalPercentForDistribution, auto&& percentForColumn) {
#if ASSERT_ENABLED
            float allocatedMinLogicalWidth = 0;
#endif
            float allocatedMaxLogicalWidth = 0;
            for (unsigned pos = effCol; pos < lastCol; ++pos) {
                auto percent = percentForColumn(pos);
                if (!percent)
                    continue;
                float columnMinLogicalWidth = *percent * cellMinLogicalWidth / totalPercentForDistribution;
                float columnMaxLogicalWidth = *percent * cellMaxLogicalWidth / totalPercentForDistribution;
                m_layoutStruct[pos].effectiveMinLogicalWidth = std::max(m_layoutStruct[pos].effectiveMinLogicalWidth, columnMinLogicalWidth);
                m_layoutStruct[pos].effectiveMaxLogicalWidth = columnMaxLogicalWidth;
#if ASSERT_ENABLED
                allocatedMinLogicalWidth += columnMinLogicalWidth;
#endif
                allocatedMaxLogicalWidth += columnMaxLogicalWidth;
            }
            ASSERT(allocatedMinLogicalWidth < cellMinLogicalWidth || WTF::areEssentiallyEqual(allocatedMinLogicalWidth, cellMinLogicalWidth));
            ASSERT(allocatedMaxLogicalWidth < cellMaxLogicalWidth || WTF::areEssentiallyEqual(allocatedMaxLogicalWidth, cellMaxLogicalWidth));
            cellMaxLogicalWidth -= allocatedMaxLogicalWidth;
        };

        // make sure minWidth and maxWidth of the spanning cell are honored
        if (cellMinLogicalWidth > spanMinLogicalWidth) {
            if (allColsAreFixed) {
                for (unsigned pos = effCol; fixedWidth > 0 && pos < lastCol; ++pos) {
                    // NOTE: The unchecked use of tryFixed() here is allowed because `allColsAreFixed` is true.
                    // FIXME: Find a more type safe way to enforce this invariant.
                    auto fixedLogicalWidth = m_layoutStruct[pos].logicalWidth.tryFixed()->resolveZoom(Style::ZoomFactor { m_layoutStruct[effCol].usedZoom });

                    float cellLogicalWidth = std::max(m_layoutStruct[pos].effectiveMinLogicalWidth, cellMinLogicalWidth * fixedLogicalWidth / fixedWidth);
                    fixedWidth -= fixedLogicalWidth;
                    cellMinLogicalWidth -= cellLogicalWidth;
                    m_layoutStruct[pos].effectiveMinLogicalWidth = cellLogicalWidth;
                }
            } else if (allColsArePercent) {
                // In this case, we just split the colspan's min and max widths following the percentage.
                // |allColsArePercent| means that either the logicalWidth *or* the effectiveLogicalWidth are percents, handle both of them here.
                distributeByPercent(totalPercent, [&](unsigned pos) -> std::optional<float> {
                    ASSERT(m_layoutStruct[pos].logicalWidth.isPercent() || m_layoutStruct[pos].effectiveLogicalWidth.isPercent());
                    auto percentageLogicalWidth = m_layoutStruct[pos].logicalWidth.tryPercentage();
                    auto percentageEffectiveLogicalWidth = m_layoutStruct[pos].effectiveLogicalWidth.tryPercentage();
                    ASSERT(percentageLogicalWidth || percentageEffectiveLogicalWidth);
                    return percentageLogicalWidth ? percentageLogicalWidth->value : percentageEffectiveLogicalWidth->value;
                });
            } else if (!allColsAreFixed && fixedWidth <= 0 && totalPercent > 0 && haveAuto) {
                // This branch handles the case where a percentage colspan cell has already
                // converted AUTO columns to effective percentages. We need to verify that:
                // 1. There are columns with original logicalWidth = auto.
                // 2. Those auto columns were converted to effective percentages.
                bool hasConvertedAutoColumns = false;
                for (unsigned pos = effCol; pos < lastCol; ++pos) {
                    // Check if this was originally an AUTO column that got converted to percentage.
                    if (m_layoutStruct[pos].logicalWidth.isAuto() && m_layoutStruct[pos].effectiveLogicalWidth.isPercentOrCalculated()) {
                        hasConvertedAutoColumns = true;
                        break;
                    }
                }

                // Additionally, verify the current cell is NOT a percentage cell (percentage cells are handled earlier).
                bool currentCellIsNotPercentage = !cellLogicalWidth.isPercentOrCalculated();
                // Only use percentage-based distribution if auto columns were actually converted AND we're processing a non-percentage colspan
                if (hasConvertedAutoColumns && currentCellIsNotPercentage) {
                    // By this point, the earlier code has converted auto columns to effectiveLogicalWidth percentages,
                    // so we can use the same percentage-based distribution as the allColsArePercent case.

                    // Calculate total effective percent (includes both original percent columns and converted auto columns)
                    float totalEffectivePercent = 0;
                    for (unsigned pos = effCol; pos < lastCol; ++pos) {
                        if (auto percentageEffectiveLogicalWidth = m_layoutStruct[pos].effectiveLogicalWidth.tryPercentage())
                            totalEffectivePercent += percentageEffectiveLogicalWidth->value;
                    }

                    // If all columns now have effective percentages, distribute accordingly
                    if (totalEffectivePercent > 0) {
                        distributeByPercent(totalEffectivePercent, [&](unsigned pos) -> std::optional<float> {
                            if (auto percentageEffectiveLogicalWidth = m_layoutStruct[pos].effectiveLogicalWidth.tryPercentage())
                                return percentageEffectiveLogicalWidth->value;
                            return std::nullopt;
                        });
                    }
                }
            } else {
                float remainingMaxLogicalWidth = spanMaxLogicalWidth;
                float remainingMinLogicalWidth = spanMinLogicalWidth;

                // Give min to variable first, to fixed second, and to others third.
                for (unsigned pos = effCol; remainingMaxLogicalWidth >= 0 && pos < lastCol; ++pos) {
                    if (auto fixedLogicalWidth = m_layoutStruct[pos].logicalWidth.tryFixed(); fixedLogicalWidth && haveAuto && fixedWidth <= cellMinLogicalWidth) {
                        float colMinLogicalWidth = std::max(m_layoutStruct[pos].effectiveMinLogicalWidth, fixedLogicalWidth->resolveZoom(Style::ZoomFactor { m_layoutStruct[pos].usedZoom }));
                        fixedWidth -= fixedLogicalWidth->resolveZoom(Style::ZoomFactor { m_layoutStruct[pos].usedZoom });
                        remainingMinLogicalWidth -= m_layoutStruct[pos].effectiveMinLogicalWidth;
                        remainingMaxLogicalWidth -= m_layoutStruct[pos].effectiveMaxLogicalWidth;
                        cellMinLogicalWidth -= colMinLogicalWidth;
                        m_layoutStruct[pos].effectiveMinLogicalWidth = colMinLogicalWidth;
                    }
                }

                for (unsigned pos = effCol; remainingMaxLogicalWidth >= 0 && pos < lastCol && remainingMinLogicalWidth < cellMinLogicalWidth; ++pos) {
                    if (!(m_layoutStruct[pos].logicalWidth.isFixed() && haveAuto && fixedWidth <= cellMinLogicalWidth)) {
                        float colMinLogicalWidth = std::max(m_layoutStruct[pos].effectiveMinLogicalWidth, remainingMaxLogicalWidth ? cellMinLogicalWidth * m_layoutStruct[pos].effectiveMaxLogicalWidth / remainingMaxLogicalWidth : cellMinLogicalWidth);
                        colMinLogicalWidth = std::min(m_layoutStruct[pos].effectiveMinLogicalWidth + (cellMinLogicalWidth - remainingMinLogicalWidth), colMinLogicalWidth);
                        remainingMaxLogicalWidth -= m_layoutStruct[pos].effectiveMaxLogicalWidth;
                        remainingMinLogicalWidth -= m_layoutStruct[pos].effectiveMinLogicalWidth;
                        cellMinLogicalWidth -= colMinLogicalWidth;
                        m_layoutStruct[pos].effectiveMinLogicalWidth = colMinLogicalWidth;
                    }
                }
            }
        }
        if (!cellLogicalWidth.isPercentOrCalculated()) {
            if (cellMaxLogicalWidth > spanMaxLogicalWidth) {
                for (unsigned pos = effCol; spanMaxLogicalWidth >= 0 && pos < lastCol; ++pos) {
                    float colMaxLogicalWidth = std::max(m_layoutStruct[pos].effectiveMaxLogicalWidth, spanMaxLogicalWidth ? cellMaxLogicalWidth * m_layoutStruct[pos].effectiveMaxLogicalWidth / spanMaxLogicalWidth : cellMaxLogicalWidth);
                    spanMaxLogicalWidth -= m_layoutStruct[pos].effectiveMaxLogicalWidth;
                    cellMaxLogicalWidth -= colMaxLogicalWidth;
                    m_layoutStruct[pos].effectiveMaxLogicalWidth = colMaxLogicalWidth;
                }
            }
        } else {
            for (unsigned pos = effCol; pos < lastCol; ++pos)
                m_layoutStruct[pos].maxLogicalWidth = std::max(m_layoutStruct[pos].maxLogicalWidth, m_layoutStruct[pos].minLogicalWidth);
        }
        // treat span ranges consisting of empty cells only as if they had content
        if (spanHasEmptyCellsOnly) {
            for (unsigned pos = effCol; pos < lastCol; ++pos)
                m_layoutStruct[pos].emptyCellsOnly = false;
        }
    }
    m_effectiveLogicalWidthDirty = false;

    return std::min<float>(maxLogicalWidth, tableMaxWidth);
}

/* gets all cells that originate in a column and have a cellspan > 1
   Sorts them by increasing cellspan
*/
void AutoTableLayout::insertSpanCell(RenderTableCell* cell)
{
    ASSERT_ARG(cell, cell && cell->colSpan() != 1);
    if (!cell || cell->colSpan() == 1)
        return;

    unsigned size = m_spanCells.size();
    if (!size || m_spanCells[size-1] != 0) {
        m_spanCells.grow(size + 10);
        for (unsigned i = 0; i < 10; i++)
            m_spanCells[size + i] = 0;
        size += 10;
    }

    // add them in sort. This is a slow algorithm, and a binary search or a fast sorting after collection would be better
    unsigned pos = 0;
    unsigned span = cell->colSpan();
    while (pos < m_spanCells.size() && m_spanCells[pos] && span > m_spanCells[pos]->colSpan())
        ++pos;
    memmoveSpan(m_spanCells.mutableSpan().subspan(pos + 1), m_spanCells.subspan(pos, size - (pos + 1)));
    m_spanCells[pos] = cell;
}

void AutoTableLayout::layout()
{
    // table layout based on the values collected in the layout structure.
    float tableLogicalWidth = m_table->logicalWidth() - m_table->bordersPaddingAndSpacingInRowDirection();
    float available = tableLogicalWidth;
    size_t nEffCols = m_table->numEffCols();

    // FIXME: It is possible to be called without having properly updated our internal representation.
    // This means that our preferred logical widths were not recomputed as expected.
    if (nEffCols != m_layoutStruct.size()) {
        fullRecalc();
        // FIXME: Table layout shouldn't modify our table structure (but does due to columns and column-groups).
        nEffCols = m_table->numEffCols();
    }

    if (m_effectiveLogicalWidthDirty)
        calcEffectiveLogicalWidth();

    bool havePercent = false;
    int numFixed = 0;
    size_t numberOfNonEmptyAuto = 0;
    std::optional<float> totalAuto;
    float totalFixed = 0;
    float totalPercent = 0;
    float allocAuto = 0;
    unsigned numAutoEmptyCellsOnly = 0;

    // fill up every cell with its minWidth
    for (size_t i = 0; i < nEffCols; ++i) {
        // Check if this column is collapsed
        if (isColumnCollapsed(i)) {
            m_layoutStruct[i].computedLogicalWidth = 0;
            continue;
        }

        float cellLogicalWidth = m_layoutStruct[i].effectiveMinLogicalWidth;
        m_layoutStruct[i].computedLogicalWidth = cellLogicalWidth;
        available -= cellLogicalWidth;
        WTF::switchOn(m_layoutStruct[i].effectiveLogicalWidth,
            [&](const Style::PreferredSize::Fixed&) {
                numFixed++;
                totalFixed += m_layoutStruct[i].effectiveMaxLogicalWidth;
            },
            [&](const Style::PreferredSize::Percentage& percentageLogicalWidth) {
                havePercent = true;
                totalPercent += percentageLogicalWidth.value;
            },
            [&](const CSS::Keyword::Auto&) {
                if (m_layoutStruct[i].emptyCellsOnly)
                    numAutoEmptyCellsOnly++;
                else {
                    ++numberOfNonEmptyAuto;
                    totalAuto = totalAuto.value_or(0.f) + m_layoutStruct[i].effectiveMaxLogicalWidth;
                    allocAuto += cellLogicalWidth;
                }
            },
            [&](const auto&) { }
        );
    }

    // allocate width to percent cols
    if (available > 0 && havePercent) {
        for (size_t i = 0; i < nEffCols; ++i) {
            if (isColumnCollapsed(i))
                continue;

            auto& logicalWidth = m_layoutStruct[i].effectiveLogicalWidth;
            if (logicalWidth.isPercentOrCalculated()) {
                float cellLogicalWidth = std::max<float>(m_layoutStruct[i].effectiveMinLogicalWidth, Style::evaluateMinimum<float>(logicalWidth, tableLogicalWidth, Style::ZoomFactor { m_layoutStruct[i].usedZoom }));
                available += m_layoutStruct[i].computedLogicalWidth - cellLogicalWidth;
                m_layoutStruct[i].computedLogicalWidth = cellLogicalWidth;
            }
        }
        if (totalPercent > 100) {
            // remove overallocated space from the last columns
            float excess = tableLogicalWidth * (totalPercent - 100) / 100;
            for (unsigned i = nEffCols; i; ) {
                --i;
                if (isColumnCollapsed(i))
                    continue;

                if (m_layoutStruct[i].effectiveLogicalWidth.isPercentOrCalculated()) {
                    float cellLogicalWidth = m_layoutStruct[i].computedLogicalWidth;
                    float reduction = std::min(cellLogicalWidth,  excess);
                    // the lines below might look inconsistent, but that's the way it's handled in mozilla
                    excess -= reduction;
                    float newLogicalWidth = std::max(m_layoutStruct[i].effectiveMinLogicalWidth, cellLogicalWidth - reduction);
                    available += cellLogicalWidth - newLogicalWidth;
                    m_layoutStruct[i].computedLogicalWidth = newLogicalWidth;
                }
            }
        }
    }
    
    // then allocate width to fixed cols
    if (available > 0) {
        for (size_t i = 0; i < nEffCols; ++i) {
            if (isColumnCollapsed(i))
                continue;

            auto& logicalWidth = m_layoutStruct[i].effectiveLogicalWidth;
            auto usedZoom = m_layoutStruct[i].usedZoom;
            if (auto fixedLogicalWidth = logicalWidth.tryFixed(); fixedLogicalWidth && fixedLogicalWidth->resolveZoom(Style::ZoomFactor { usedZoom }) > m_layoutStruct[i].computedLogicalWidth) {
                available += m_layoutStruct[i].computedLogicalWidth - fixedLogicalWidth->resolveZoom(Style::ZoomFactor { usedZoom });
                m_layoutStruct[i].computedLogicalWidth = fixedLogicalWidth->resolveZoom(Style::ZoomFactor { usedZoom });
            }
        }
    }

    // now satisfy variable
    if (available > 0 && numberOfNonEmptyAuto) {
        ASSERT(totalAuto);
        available += allocAuto; // this gets redistributed.
        auto equalWidthForZeroLengthColumns = std::optional<float> { };
        if (!*totalAuto) {
            // All columns in this table are (non-empty)zero length with 'width: auto'.
            equalWidthForZeroLengthColumns = available / numberOfNonEmptyAuto;
        }
        for (size_t i = 0; i < nEffCols; ++i) {
            if (isColumnCollapsed(i))
                continue;

            auto& column = m_layoutStruct[i];
            if (!column.effectiveLogicalWidth.isAuto() || column.emptyCellsOnly)
                continue;
            auto columnWidthCandidate = equalWidthForZeroLengthColumns ? *equalWidthForZeroLengthColumns : available * column.effectiveMaxLogicalWidth / *totalAuto;
            column.computedLogicalWidth = std::max(column.computedLogicalWidth, columnWidthCandidate);
            available -= column.computedLogicalWidth;
            if (!equalWidthForZeroLengthColumns) {
                *totalAuto -= column.effectiveMaxLogicalWidth;
                if (*totalAuto <= 0)
                    break;
            }
        }
    }

    // spread over fixed columns
    if (available > 0 && numFixed) {
        for (size_t i = 0; i < nEffCols; ++i) {
            if (isColumnCollapsed(i))
                continue;

            auto& logicalWidth = m_layoutStruct[i].effectiveLogicalWidth;
            if (logicalWidth.isFixed()) {
                float cellLogicalWidth = available * m_layoutStruct[i].effectiveMaxLogicalWidth / totalFixed;
                available -= cellLogicalWidth;
                totalFixed -= m_layoutStruct[i].effectiveMaxLogicalWidth;
                m_layoutStruct[i].computedLogicalWidth += cellLogicalWidth;
            }
        }
    }

    // spread over percent columns
    if (available > 0 && m_hasPercent && totalPercent < 100) {
        for (size_t i = 0; i < nEffCols; ++i) {
            if (isColumnCollapsed(i))
                continue;

            auto& logicalWidth = m_layoutStruct[i].effectiveLogicalWidth;
            if (auto percentageLogicalWidth = logicalWidth.tryPercentage()) {
                float cellLogicalWidth = available * percentageLogicalWidth->value / totalPercent;
                available -= cellLogicalWidth;
                totalPercent -= percentageLogicalWidth->value;
                m_layoutStruct[i].computedLogicalWidth += cellLogicalWidth;
                if (!available || !totalPercent)
                    break;
            }
        }
    }

    // spread over the rest
    if (available > 0 && nEffCols > numAutoEmptyCellsOnly) {
        unsigned total = nEffCols - numAutoEmptyCellsOnly;
        // Count collapsed columns to subtract from total
        unsigned numCollapsed = 0;
        for (size_t i = 0; i < nEffCols; ++i) {
            if (isColumnCollapsed(i))
                numCollapsed++;
        }
        total -= numCollapsed;

        // still have some width to spread
        for (unsigned i = nEffCols; i && total > 0; ) {
            --i;
            if (isColumnCollapsed(i))
                continue;

            // variable columns with empty cells only don't get any width
            if (m_layoutStruct[i].effectiveLogicalWidth.isAuto() && m_layoutStruct[i].emptyCellsOnly)
                continue;
            float cellLogicalWidth = available / total;
            available -= cellLogicalWidth;
            total--;
            m_layoutStruct[i].computedLogicalWidth += cellLogicalWidth;
        }
    }

    if (available > 0 && numAutoEmptyCellsOnly && nEffCols == numAutoEmptyCellsOnly) {
        // All columns in this table are empty with 'width: auto'.
        auto equalWidthForColumns = available / numAutoEmptyCellsOnly;
        for (size_t i = 0; i < nEffCols; ++i) {
            if (isColumnCollapsed(i))
                continue;

            auto& column = m_layoutStruct[i];
            column.computedLogicalWidth = equalWidthForColumns;
            available -= column.computedLogicalWidth;
        }
    }

    // If we have over-allocated, reduce every cell according to the difference between desired width and min-width
    // this seems to produce to the pixel exact results with IE. Wonder if some of this also holds for width distributing.
    // Need to reduce cells with the following prioritization:
    // This is basically the reverse of how we grew the cells.
    if (available < 0)
        available = shrinkCellWidthForType<CSS::Keyword::Auto>(available);
    if (available < 0)
        available = shrinkCellWidthForType<Style::PreferredSize::Fixed>(available);
    if (available < 0)
        available = shrinkCellWidthForType<Style::PreferredSize::Percentage>(available);

    LayoutUnit pos;
    for (size_t i = 0; i < nEffCols; ++i) {
        m_table->setColumnPosition(i, pos);

        if (isColumnCollapsed(i)) {
            // Don't add width or spacing for collapsed columns.
            continue;
        }

        pos += LayoutUnit::fromFloatCeil(m_layoutStruct[i].computedLogicalWidth) + m_table->hBorderSpacing();
    }
    m_table->setColumnPosition(m_table->columnPositions().size() - 1, pos);
}

template<typename T> float AutoTableLayout::shrinkCellWidthForType(float available)
{
    unsigned nEffCols = m_table->numEffCols();
    float logicalWidthBeyondMin = 0;
    for (unsigned i = nEffCols; i; ) {
        --i;
        auto& logicalWidth = m_layoutStruct[i].effectiveLogicalWidth;
        if (WTF::holdsAlternative<T>(logicalWidth))
            logicalWidthBeyondMin += m_layoutStruct[i].computedLogicalWidth - m_layoutStruct[i].effectiveMinLogicalWidth;
    }

    for (unsigned i = nEffCols; i && logicalWidthBeyondMin > 0; ) {
        --i;
        auto& logicalWidth = m_layoutStruct[i].effectiveLogicalWidth;
        if (WTF::holdsAlternative<T>(logicalWidth)) {
            float minMaxDiff = m_layoutStruct[i].computedLogicalWidth - m_layoutStruct[i].effectiveMinLogicalWidth;
            float reduce = available * minMaxDiff / logicalWidthBeyondMin;
            m_layoutStruct[i].computedLogicalWidth += reduce;
            available -= reduce;
            logicalWidthBeyondMin -= minMaxDiff;
            if (available >= 0)
                break;
        }
    }

    return available;
}

}
```

## FixedTableLayout.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/FixedTableLayout.cpp). Path: `Source/WebCore/rendering/FixedTableLayout.cpp`. Source bytes: 14791; source lines: 328; SHA-256: `3ffd9e20bfb0791799ca3503edf1a23f6249f2535382b9c08cb9df6bf2e1ffbf`.

```cpp
/*
 * Copyright (C) 2002 Lars Knoll (knoll@kde.org)
 *           (C) 2002 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2003-2026 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License.
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

#include "config.h"
#include "FixedTableLayout.h"

#include "RenderBoxInlines.h"
#include "RenderTableCellInlines.h"
#include "RenderTableCol.h"
#include "RenderTableInlines.h"
#include "RenderTableSection.h"
#include "StylePreferredSize.h"
#include "StylePrimitiveNumericTypes+Evaluation.h"

/*
  The text below is from the CSS 2.1 specs.

  Fixed table layout

  With this (fast) algorithm, the horizontal layout of the table does
  not depend on the contents of the cells; it only depends on the
  table's width, the width of the columns, and borders or cell
  spacing.

  The table's width may be specified explicitly with the 'width'
  property. A value of 'auto' (for both 'display: table' and 'display:
  inline-table') means use the automatic table layout algorithm.

  In the fixed table layout algorithm, the width of each column is
  determined as follows:

    1. A column element with a value other than 'auto' for the 'width'
    property sets the width for that column.

    2. Otherwise, a cell in the first row with a value other than
    'auto' for the 'width' property sets the width for that column. If
    the cell spans more than one column, the width is divided over the
    columns.

    3. Any remaining columns equally divide the remaining horizontal
    table space (minus borders or cell spacing).

  The width of the table is then the greater of the value of the
  'width' property for the table element and the sum of the column
  widths (plus cell spacing or borders). If the table is wider than
  the columns, the extra space should be distributed over the columns.


  In this manner, the user agent can begin to lay out the table once
  the entire first row has been received. Cells in subsequent rows do
  not affect column widths. Any cell that has content that overflows
  uses the 'overflow' property to determine whether to clip the
  overflow content.
*/

namespace WebCore {

FixedTableLayout::FixedTableLayout(RenderTable* table)
    : TableLayout(table)
{
}

float FixedTableLayout::calcWidthArray()
{
    // FIXME: We might want to wait until we have all of the first row before computing for the first time.
    float usedWidth = 0;

    // iterate over all <col> elements
    unsigned nEffCols = m_table->numEffCols();
    m_width.fill(Style::PreferredSize { CSS::Keyword::Auto { } }, nEffCols);

    unsigned currentEffectiveColumn = 0;
    for (RenderTableCol* col = m_table->firstColumn(); col; col = col->nextColumn()) {
        // RenderTableCols don't have the concept of preferred logical width, but we need to clear their dirty bits
        // so that if we call setContentWidthsDirty(true) on a col or one of its descendants, we'll mark it's
        // ancestors as dirty.
        col->clearContentLogicalWidthsInvalidation();

        // Width specified by column-groups that have column child does not affect column width in fixed layout tables
        if (col->isTableColumnGroupWithColumnChildren())
            continue;

        auto colStyleLogicalWidth = col->style().logicalWidth();
        auto colUsedZoom = col->style().usedZoomForLength();
        float effectiveColWidth = 0;
        if (auto fixedColStyleLogicalWidth = colStyleLogicalWidth.tryFixed(); fixedColStyleLogicalWidth && fixedColStyleLogicalWidth->isPositive())
            effectiveColWidth = fixedColStyleLogicalWidth->resolveZoom(colUsedZoom);
        else if (colStyleLogicalWidth.isCalculated())
            colStyleLogicalWidth = CSS::Keyword::Auto { };

        unsigned span = col->span();
        while (span) {
            unsigned spanInCurrentEffectiveColumn;
            if (currentEffectiveColumn >= nEffCols) {
                m_table->appendColumn(span);
                nEffCols++;
                m_width.append(CSS::Keyword::Auto { });
                spanInCurrentEffectiveColumn = span;
            } else {
                if (span < m_table->spanOfEffCol(currentEffectiveColumn)) {
                    m_table->splitColumn(currentEffectiveColumn, span);
                    nEffCols++;
                    m_width.append(CSS::Keyword::Auto { });
                }
                spanInCurrentEffectiveColumn = m_table->spanOfEffCol(currentEffectiveColumn);
            }
            if (auto fixedColStyleLogicalWidth = colStyleLogicalWidth.tryFixed(); fixedColStyleLogicalWidth && fixedColStyleLogicalWidth->isPositive()) {
                m_width[currentEffectiveColumn] = Style::PreferredSize::Fixed { effectiveColWidth * spanInCurrentEffectiveColumn };
                usedWidth += effectiveColWidth * spanInCurrentEffectiveColumn;
            } else if (auto percentageColStyleLogicalWidth = colStyleLogicalWidth.tryPercentage(); percentageColStyleLogicalWidth && percentageColStyleLogicalWidth->value > 0) {
                m_width[currentEffectiveColumn] = Style::PreferredSize::Percentage { percentageColStyleLogicalWidth->value * spanInCurrentEffectiveColumn };
                usedWidth += effectiveColWidth * spanInCurrentEffectiveColumn;
            }
            span -= spanInCurrentEffectiveColumn;
            currentEffectiveColumn++;
        }
    }

    // Iterate over the first row in case some are unspecified.
    RenderTableSection* section = m_table->topNonEmptySection();
    if (!section)
        return usedWidth;

    unsigned currentColumn = 0;

    RenderTableRow* firstRow = section->firstRow();
    for (RenderTableCell* cell = firstRow->firstCell(); cell; cell = cell->nextCell()) {
        auto [ logicalWidth, usedZoom ] = cell->styleOrColLogicalWidth();
        unsigned span = cell->colSpan();
        float fixedBorderBoxLogicalWidth = 0;
        // FIXME: Support other length types. If the width is non-auto, it should probably just use
        // RenderBox::computeLogicalWidthInFragmentUsing to compute the width.
        if (auto fixedLogicalWidth = logicalWidth.tryFixed(); fixedLogicalWidth && fixedLogicalWidth->isPositive()) {
            fixedBorderBoxLogicalWidth = cell->adjustBorderBoxLogicalWidthForBoxSizing(*fixedLogicalWidth);
            logicalWidth = Style::PreferredSize::Fixed { fixedBorderBoxLogicalWidth };
        } else if (logicalWidth.isCalculated())
            logicalWidth = CSS::Keyword::Auto { };

        unsigned usedSpan = 0;
        while (usedSpan < span && currentColumn < nEffCols) {
            float eSpan = m_table->spanOfEffCol(currentColumn);
            // Only set if no col element has already set it.
            if (m_width[currentColumn].isAuto() && !logicalWidth.isAuto()) {
                if (auto fixedLogicalWidth = logicalWidth.tryFixed()) {
                    // Zoom was applied when we called adjustBorderBoxLogicalWidthForBoxSizing
                    m_width[currentColumn] = Style::PreferredSize::Fixed { fixedLogicalWidth->resolveZoom(Style::ZoomFactor { 1.0f }) * eSpan / span };
                }
                else if (auto percentageLogicalWidth = logicalWidth.tryPercentage())
                    m_width[currentColumn] = Style::PreferredSize::Percentage { percentageLogicalWidth->value * eSpan / span };
                usedWidth += fixedBorderBoxLogicalWidth * eSpan / span;
            }
            usedSpan += eSpan;
            ++currentColumn;
        }

        // FixedTableLayout doesn't use min/maxContentLogicalWidths, but we need to clear the
        // dirty bit on the cell so that we'll correctly mark its ancestors dirty
        // in case we later call invalidateContentLogicalWidths() on it later.
        if (cell->hasInvalidContentLogicalWidths())
            cell->clearContentLogicalWidthsInvalidation();
    }

    return usedWidth;
}

std::pair<LayoutUnit, LayoutUnit> FixedTableLayout::computeIntrinsicLogicalWidths(TableIntrinsics)
{
    auto logicalWidth = LayoutUnit { calcWidthArray() };
    return { logicalWidth, logicalWidth };
}

void FixedTableLayout::applyContentLogicalWidthQuirks(LayoutUnit& minWidth, LayoutUnit& maxWidth) const
{
    auto& tableLogicalWidth = m_table->style().logicalWidth();
    if (auto fixedTableLogicalWidth = tableLogicalWidth.tryFixed(); fixedTableLogicalWidth && fixedTableLogicalWidth->isPositive())
        minWidth = maxWidth = std::max(minWidth, LayoutUnit(fixedTableLogicalWidth->resolveZoom(m_table->style().usedZoomForLength())) - m_table->bordersPaddingAndSpacingInRowDirection());

    /*
        <table style="width:100%; background-color:red"><tr><td>
            <table style="background-color:blue"><tr><td>
                <table style="width:100%; background-color:green; table-layout:fixed"><tr><td>
                    Content
                </td></tr></table>
            </td></tr></table>
        </td></tr></table>
    */ 
    // In this example, the two inner tables should be as large as the outer table. 
    // We can achieve this effect by making the max-width of fixed tables with percentage
    // widths be infinite.
    if (tableLogicalWidth.isPercentOrCalculated() && maxWidth < tableMaxWidth)
        maxWidth = tableMaxWidth;
}

void FixedTableLayout::layout()
{
    float tableLogicalWidth = m_table->logicalWidth() - m_table->bordersPaddingAndSpacingInRowDirection();
    unsigned nEffCols = m_table->numEffCols();

    // FIXME: It is possible to be called without having properly updated our internal representation.
    // This means that our preferred logical widths were not recomputed as expected.
    if (nEffCols != m_width.size()) {
        calcWidthArray();
        // FIXME: Table layout shouldn't modify our table structure (but does due to columns and column-groups).
        nEffCols = m_table->numEffCols();
    }

    Vector<float> calcWidth(FillWith { }, nEffCols, 0);

    unsigned numAuto = 0;
    unsigned autoSpan = 0;
    float totalFixedWidth = 0;
    float totalPercentWidth = 0;
    float totalPercent = 0;

    // Compute requirements and try to satisfy fixed and percent widths.
    // Percentages are of the table's width, so for example
    // for a table width of 100px with columns (40px, 10%), the 10% compute
    // to 10px here, and will scale up to 20px in the final (80px, 20px).
    for (unsigned i = 0; i < nEffCols; i++) {
        if (auto fixedWidth = m_width[i].tryFixed()) {
            // We zoomed the widths inside of calcWidthArray so here we pass in a zoom
            // factor of 1.0f to avoid double zooming.
            calcWidth[i] = Style::evaluate<float>(*fixedWidth, Style::ZoomFactor { 1.0f });
            totalFixedWidth += calcWidth[i];
        } else if (auto percentageWidth = m_width[i].tryPercentage()) {
            calcWidth[i] = Style::evaluate<float>(*percentageWidth, tableLogicalWidth);
            totalPercentWidth += calcWidth[i];
            totalPercent += percentageWidth->value;
        } else if (m_width[i].isAuto()) {
            numAuto++;
            autoSpan += m_table->spanOfEffCol(i);
        }
    }

    float hspacing = m_table->hBorderSpacing();
    float totalWidth = totalFixedWidth + totalPercentWidth;
    if (!numAuto || totalWidth > tableLogicalWidth) {
        // If there are no auto columns, or if the total is too wide, take
        // what we have and scale it to fit as necessary.
        if (totalWidth != tableLogicalWidth) {
            // Fixed widths only scale up
            if (totalFixedWidth && totalWidth < tableLogicalWidth) {
                totalFixedWidth = 0;
                for (unsigned i = 0; i < nEffCols; i++) {
                    if (m_width[i].isFixed()) {
                        calcWidth[i] = calcWidth[i] * tableLogicalWidth / totalWidth;
                        totalFixedWidth += calcWidth[i];
                    }
                }
            }
            if (totalPercent) {
                totalPercentWidth = 0;
                for (unsigned i = 0; i < nEffCols; i++) {
                    if (auto percentageWidth = m_width[i].tryPercentage()) {
                        calcWidth[i] = percentageWidth->value * (tableLogicalWidth - totalFixedWidth) / totalPercent;
                        totalPercentWidth += calcWidth[i];
                    }
                }
            }
            totalWidth = totalFixedWidth + totalPercentWidth;
        }
    } else {
        // Divide the remaining width among the auto columns.
        ASSERT(autoSpan >= numAuto);
        float remainingWidth = tableLogicalWidth - totalFixedWidth - totalPercentWidth - hspacing * (autoSpan - numAuto);
        int lastAuto = 0;
        for (unsigned i = 0; i < nEffCols; i++) {
            if (m_width[i].isAuto()) {
                unsigned span = m_table->spanOfEffCol(i);
                float w = remainingWidth * span / autoSpan;
                calcWidth[i] = w + hspacing * (span - 1);
                remainingWidth -= w;
                if (!remainingWidth)
                    break;
                lastAuto = i;
                numAuto--;
                ASSERT(autoSpan >= span);
                autoSpan -= span;
            }
        }
        // Last one gets the remainder.
        if (remainingWidth)
            calcWidth[lastAuto] += remainingWidth;
        totalWidth = tableLogicalWidth;
    }

    if (totalWidth < tableLogicalWidth) {
        // Spread extra space over columns.
        float remainingWidth = tableLogicalWidth - totalWidth;
        int total = nEffCols;
        while (total) {
            float w = remainingWidth / total;
            remainingWidth -= w;
            calcWidth[--total] += w;
        }
        if (nEffCols > 0)
            calcWidth[nEffCols - 1] += remainingWidth;
    }
    
    float pos = 0;
    for (unsigned i = 0; i < nEffCols; i++) {
        m_table->setColumnPosition(i, LayoutUnit(pos));
        pos += calcWidth[i] + hspacing;
    }
    auto colPositionsSize = m_table->columnPositions().size();
    if (colPositionsSize > 0)
        m_table->setColumnPosition(colPositionsSize - 1, LayoutUnit(pos));
}

} // namespace WebCore
```

## RenderTable.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderTable.cpp). Path: `Source/WebCore/rendering/RenderTable.cpp`. Source bytes: 77376; source lines: 1825; SHA-256: `6ec74ca9dfb2d292494a40c819b8cd527c47176430f73ef3e1322a59cdb979cd`.

```cpp
/*
 * Copyright (C) 1997 Martin Jones (mjones@kde.org)
 *           (C) 1997 Torben Weis (weis@kde.org)
 *           (C) 1998 Waldo Bastian (bastian@kde.org)
 *           (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 * Copyright (C) 2003-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2014-2019 Google Inc. All rights reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@nypop.com)
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

#include "config.h"
#include "RenderTable.h"

#include "AutoTableLayout.h"
#include "BackgroundPainter.h"
#include "BorderPainter.h"
#include "BorderShape.h"
#include "CollapsedBorderValue.h"
#include "Document.h"
#include "FixedTableLayout.h"
#include "HitTestResult.h"
#include "HTMLNames.h"
#include "HTMLTableElement.h"
#include "InlineIteratorInlineBox.h"
#include "LayoutRepainter.h"
#include "LocalFrameView.h"
#include "PaintInfoInlines.h"
#include "RenderBlockFlow.h"
#include "RenderBlockInlines.h"
#include "RenderBoxInlines.h"
#include "RenderChildIterator.h"
#include "RenderDescendantIterator.h"
#include "RenderElementInlines.h"
#include "RenderElementStyleInlines.h"
#include "RenderFlexibleBox.h"
#include "RenderIterator.h"
#include "RenderLayer.h"
#include "RenderLayoutState.h"
#include "RenderObjectInlines.h"
#include "RenderTableCaption.h"
#include "RenderTableCellInlines.h"
#include "RenderTableCol.h"
#include "RenderTableInlines.h"
#include "RenderTableRowInlines.h"
#include "RenderTableSectionInlines.h"
#include "RenderTreeBuilder.h"
#include "RenderView.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "StylePrimitiveNumericTypes+Evaluation.h"
#include "StylePrimitiveNumericTypes+EvaluationMinimum.h"
#include "StyleSizing.h"
#include <wtf/SetForScope.h>
#include <wtf/StackStats.h>
#include <wtf/TZoneMallocInlines.h>

namespace WebCore {

using namespace HTMLNames;

WTF_MAKE_TZONE_ALLOCATED_IMPL(RenderTable);

RenderTable::RenderTable(Type type, Element& element, Style::ComputedStyle&& style)
    : RenderBlock(type, element, WTF::move(style), { })
    , m_columnPos(FillWith { }, 1, 0)
{
    setChildrenInline(false);
    ASSERT(isRenderTable());
}

RenderTable::RenderTable(Type type, Document& document, Style::ComputedStyle&& style)
    : RenderBlock(type, document, WTF::move(style), { })
    , m_columnPos(FillWith { }, 1, 0)
{
    setChildrenInline(false);
    ASSERT(isRenderTable());
}

RenderTable::~RenderTable() = default;

RenderTableSection* NODELETE RenderTable::topSection() const
{
    ASSERT(!needsSectionRecalc());
    if (m_head)
        return m_head.get();
    if (m_firstBody)
        return m_firstBody.get();
    return m_foot.get();
}

RenderTableSection* RenderTable::bottomSection() const
{
    recalcSectionsIfNeeded();
    if (m_foot)
        return m_foot.get();
    for (CheckedPtr child = lastChild(); child; child = child->previousSibling()) {
        if (child.get() == m_head.get())
            continue;
        if (auto* tableSection = dynamicDowncast<RenderTableSection>(*child))
            return tableSection;
    }
    return m_head.get();
}

bool RenderTable::collapseBorders() const
{
    return style().borderCollapse() == BorderCollapse::Collapse;
}

void RenderTable::styleDidChange(Style::Difference diff, const Style::ComputedStyle* oldStyle)
{
    RenderBlock::styleDidChange(diff, oldStyle);
    propagateStyleToAnonymousChildren(StylePropagationType::AllChildren);

    bool oldFixedTableLayout = oldStyle && oldStyle->isFixedTableLayout();

    auto zoom = style().usedZoomForLength();

    // In the collapsed border model, there is no cell spacing.
    m_hSpacing = collapseBorders() ? 0 : Style::evaluate<float>(style().borderHorizontalSpacing(), zoom);
    m_vSpacing = collapseBorders() ? 0 : Style::evaluate<float>(style().borderVerticalSpacing(), zoom);
    ASSERT(m_hSpacing >= 0);
    ASSERT(m_vSpacing >= 0);

    if (!m_tableLayout || style().isFixedTableLayout() != oldFixedTableLayout) {
        // According to the CSS2 spec, you only use fixed table layout if an explicit width is specified on the table. Auto width implies auto table layout.
        if (style().isFixedTableLayout())
            m_tableLayout = makeUnique<FixedTableLayout>(this);
        else {
            auto invalidateAllTableCellsContentLogicalWidths = [&] {
                if (!m_tableLayout)
                    return;
                // Fixed table layout sets min/max preferred widths to clean without actually computing them (see FixedTableLayout::calcWidthArray).
                for (auto& section : childrenOfType<RenderTableSection>(*this)) {
                    for (CheckedPtr row = section.firstRow(); row; row = row->nextRow()) {
                        for (CheckedPtr cell = row->firstCell(); cell; cell = cell->nextCell())
                            cell->invalidateContentLogicalWidths();
                    }
                }
            };
            invalidateAllTableCellsContentLogicalWidths();
            m_tableLayout = makeUnique<AutoTableLayout>(this);
        }
    }

    if (oldStyle)
        invalidateCollapsedBordersAfterStyleChangeIfNeeded(*oldStyle, style());
}

static inline void resetSectionPointerIfNotBefore(SingleThreadWeakPtr<RenderTableSection>& section, RenderObject* before)
{
    if (!before || !section)
        return;
    auto* previousSibling = before->previousSibling();
    while (previousSibling && previousSibling != section)
        previousSibling = previousSibling->previousSibling();
    if (!previousSibling)
        section.clear();
}

void RenderTable::willInsertTableColumn(RenderTableCol&, RenderObject*)
{
    m_hasColElements = true;
}

void RenderTable::willInsertTableSection(RenderTableSection& child, RenderObject* beforeChild)
{
    switch (child.style().display().value) {
    case Style::DisplayType::TableHeaderGroup:
        resetSectionPointerIfNotBefore(m_head, beforeChild);
        if (!m_head)
            m_head = child;
        else {
            resetSectionPointerIfNotBefore(m_firstBody, beforeChild);
            if (!m_firstBody)
                m_firstBody = child;
        }
        break;
    case Style::DisplayType::TableFooterGroup:
        resetSectionPointerIfNotBefore(m_foot, beforeChild);
        if (!m_foot) {
            m_foot = child;
            break;
        }
        [[fallthrough]];
    case Style::DisplayType::TableRowGroup:
        resetSectionPointerIfNotBefore(m_firstBody, beforeChild);
        if (!m_firstBody)
            m_firstBody = child;
        break;
    default:
        ASSERT_NOT_REACHED();
    }

    setNeedsSectionRecalc();
}

void RenderTable::addCaption(RenderTableCaption& caption)
{
    ASSERT(m_captions.find(&caption) == notFound);
    m_captions.append(caption);
}

void RenderTable::removeCaption(RenderTableCaption& oldCaption)
{
    bool removed = m_captions.removeFirst(&oldCaption);
    ASSERT_UNUSED(removed, removed);
}

void RenderTable::invalidateCachedColumns()
{
    m_columnRenderersValid = false;
    m_columnRenderers.shrink(0);
    m_effectiveColumnIndexMap.clear();
}

void RenderTable::invalidateCachedColumnOffsets()
{
    m_columnOffsetTop = -1;
    m_columnOffsetHeight = -1;
}

void RenderTable::addColumn(const RenderTableCol*)
{
    invalidateCachedColumns();
}

void RenderTable::invalidateColumns()
{
    invalidateCachedColumns();
    // We don't really need to recompute our sections, but we need to update our
    // column count and whether we have a column. Currently, we only have one
    // size-fit-all flag but we may have to consider splitting it.
    setNeedsSectionRecalc();
}

void RenderTable::updateLogicalWidth()
{
    recalcSectionsIfNeeded();

    if (isGridItem()) {
        // FIXME: Investigate whether the grid layout algorithm provides all the logic
        // needed and that we're not skipping anything essential due to the early return here.
        RenderBlock::updateLogicalWidth();
        return;
    }

    if (isOutOfFlowPositioned()) {
        LogicalExtentComputedValues computedValues;
        computeOutOfFlowPositionedLogicalWidth(computedValues);
        setLogicalWidth(computedValues.extent);
        setLogicalLeft(computedValues.position);
        setMarginStart(computedValues.margins.start);
        setMarginEnd(computedValues.margins.end);
    }

    auto& containingBlock = *this->containingBlock();

    LayoutUnit availableLogicalWidth = containingBlockLogicalWidthForContent();
    bool hasPerpendicularContainingBlock = writingMode().isOrthogonal(containingBlock.writingMode());
    LayoutUnit containerWidthInInlineDirection = hasPerpendicularContainingBlock ? perpendicularContainingBlockLogicalHeight() : availableLogicalWidth;

    auto& styleLogicalWidth = style().logicalWidth();
    if (auto overridingLogicalWidth = this->overridingBorderBoxLogicalWidth())
        setLogicalWidth(*overridingLogicalWidth);
    else if ((styleLogicalWidth.isSpecified() && (styleLogicalWidth.isPossiblyPositive() || styleLogicalWidth.isKnownZero())) || styleLogicalWidth.isIntrinsicOrStretch())
        setLogicalWidth(convertStyleLogicalWidthToComputedWidth(styleLogicalWidth, containerWidthInInlineDirection));
    else {
        // Subtract out any fixed margins from our available width for auto width tables.
        auto zoom = style().usedZoomForLength();
        auto marginStart = Style::evaluateMinimum<LayoutUnit>(style().marginStart(), availableLogicalWidth, zoom);
        auto marginEnd = Style::evaluateMinimum<LayoutUnit>(style().marginEnd(), availableLogicalWidth, zoom);
        auto marginTotal = marginStart + marginEnd;

        // Subtract out our margins to get the available content width.
        LayoutUnit availableContentLogicalWidth = std::max<LayoutUnit>(0, containerWidthInInlineDirection - marginTotal);
        if (shrinkToAvoidFloats() && containingBlock.containsFloats() && !hasPerpendicularContainingBlock) {
            // FIXME: Work with regions someday.
            availableContentLogicalWidth = shrinkLogicalWidthToAvoidFloats(marginStart, marginEnd, containingBlock);
        }

        // Ensure we aren't bigger than our available width.
        LayoutUnit maxWidth = maxContentLogicalWidthContribution();
        // scaledWidthFromPercentColumns depends on m_layoutStruct in TableLayoutAlgorithmAuto, which
        // maxContentLogicalWidth fills in. So scaledWidthFromPercentColumns has to be called after
        // maxContentLogicalWidth.
        LayoutUnit scaledWidth = m_tableLayout->scaledWidthFromPercentColumns() + bordersPaddingAndSpacingInRowDirection();
        maxWidth = std::max(scaledWidth, maxWidth);
        setLogicalWidth(std::min(availableContentLogicalWidth, maxWidth));
    }

    // Ensure we aren't bigger than our max-width style.
    auto& styleMaxLogicalWidth = style().logicalMaxWidth();
    if (styleMaxLogicalWidth.isSpecified() || styleMaxLogicalWidth.isIntrinsicOrStretch()) {
        LayoutUnit computedMaxLogicalWidth = convertStyleLogicalWidthToComputedWidth(styleMaxLogicalWidth, availableLogicalWidth);
        setLogicalWidth(std::min(logicalWidth(), computedMaxLogicalWidth));
    }

    // Ensure we aren't smaller than our min preferred width.
    setLogicalWidth(std::max(logicalWidth(), minContentLogicalWidthContribution()));

    // Ensure we aren't smaller than our min-width style.
    auto& styleMinLogicalWidth = style().logicalMinWidth();
    if (styleMinLogicalWidth.isSpecified() || styleMinLogicalWidth.isIntrinsicOrStretch()) {
        LayoutUnit computedMinLogicalWidth = convertStyleLogicalWidthToComputedWidth(styleMinLogicalWidth, availableLogicalWidth);
        setLogicalWidth(std::max(logicalWidth(), computedMinLogicalWidth));
    }

    // Finally, with our true width determined, compute our margins for real.
    setMarginStart(0);
    setMarginEnd(0);
    if (!hasPerpendicularContainingBlock) {
        LayoutUnit containerLogicalWidthForAutoMargins = availableLogicalWidth;
        if (avoidsFloats() && containingBlock.containsFloats())
            containerLogicalWidthForAutoMargins = containingBlockAvailableLineWidth();
        ComputedMarginValues marginValues;
        bool hasSameDirection = !containingBlock.writingMode().isInlineOpposing(writingMode());
        computeInlineDirectionMargins(containingBlock, availableLogicalWidth, containerLogicalWidthForAutoMargins, logicalWidth(),
            hasSameDirection ? marginValues.start : marginValues.end,
            hasSameDirection ? marginValues.end : marginValues.start);
        setMarginStart(marginValues.start);
        setMarginEnd(marginValues.end);
    } else {
        auto zoom = style().usedZoomForLength();
        setMarginStart(Style::evaluateMinimum<LayoutUnit>(style().marginStart(), availableLogicalWidth, zoom));
        setMarginEnd(Style::evaluateMinimum<LayoutUnit>(style().marginEnd(), availableLogicalWidth, zoom));
    }
}

// This method takes a Style::ComputedStyle's logical width, min-width, or max-width length and computes its actual value.

template<typename SizeType> LayoutUnit RenderTable::convertStyleLogicalWidthToComputedWidth(const SizeType& styleLogicalWidth, LayoutUnit availableWidth)
{
    if (styleLogicalWidth.isIntrinsicOrStretch())
        return computeSizingKeywordLogicalWidthUsing(styleLogicalWidth, availableWidth, bordersPaddingAndSpacingInRowDirection());

    // HTML tables' width styles already include borders and padding, but CSS tables' width styles do not.
    LayoutUnit borders;
    bool isCSSTable = !is<HTMLTableElement>(element());
    if (isCSSTable && styleLogicalWidth.isSpecified() && styleLogicalWidth.isPossiblyPositive() && style().boxSizing() == BoxSizing::ContentBox)
        borders = borderStart() + borderEnd() + (collapseBorders() ? 0_lu : paddingStart() + paddingEnd());

    return Style::evaluateMinimum<LayoutUnit>(styleLogicalWidth, availableWidth, style().usedZoomForLength()) + borders;
}

template<typename SizeType> LayoutUnit RenderTable::convertStyleLogicalHeightToComputedHeight(const SizeType& styleLogicalHeight)
{
    CheckedRef checkedThis { *this };
    LayoutUnit borderAndPaddingBefore = borderBefore() + (collapseBorders() ? 0_lu : paddingBefore());
    LayoutUnit borderAndPaddingAfter = borderAfter() + (collapseBorders() ? 0_lu : paddingAfter());
    LayoutUnit borderAndPadding = borderAndPaddingBefore + borderAndPaddingAfter;
    return WTF::switchOn(styleLogicalHeight,
        [&](const typename SizeType::Fixed& fixedStyleLogicalHeight) {
            // HTML tables size as though CSS height includes border/padding, CSS tables do not.
            LayoutUnit borders;
            // FIXME: We cannot apply box-sizing: content-box on <table> which other browsers allow.
            if (is<HTMLTableElement>(checkedThis->element()) || checkedThis->style().boxSizing() == BoxSizing::BorderBox)
                borders = borderAndPadding;
            return Style::evaluate<LayoutUnit>(fixedStyleLogicalHeight, checkedThis->style().usedZoomForLength()) - borders;
        },
        [&](const typename SizeType::Percentage&) {
            return checkedThis->computePercentageLogicalHeight(styleLogicalHeight).value_or(0_lu);
        },
        [&](const typename SizeType::Calc& calc) {
            return checkedThis->computePercentageLogicalHeight(calc).value_or(0_lu);
        },
        [&](const typename SizeType::CalcSize& calcSize) {
            return checkedThis->computePercentageLogicalHeight(calcSize).value_or(0_lu);
        },
        [&](Style::IsIntrinsicOrStretchSizeKeyword auto const&) {
            return checkedThis->computeSizingKeywordLogicalContentHeightUsing(styleLogicalHeight, checkedThis->logicalHeight() - borderAndPadding, borderAndPadding).value_or(0_lu);
        },
        // The remaining keywords are not valid computed values for a table's logical height. They are
        // enumerated explicitly (rather than caught by a generic handler) so that adding a new keyword
        // to any of the sizing types fails to compile here and forces this switch to be revisited.
        [&](const CSS::Keyword::Auto&) {
            ASSERT_NOT_REACHED();
            return 0_lu;
        },
        [&](const CSS::Keyword::None&) {
            ASSERT_NOT_REACHED();
            return 0_lu;
        },
        [&](const CSS::Keyword::Intrinsic&) {
            ASSERT_NOT_REACHED();
            return 0_lu;
        },
        [&](const CSS::Keyword::MinIntrinsic&) {
            ASSERT_NOT_REACHED();
            return 0_lu;
        }
    );
}

static LayoutUnit captionLogicalHeight(const RenderTableCaption& caption, WritingMode tableWritingMode)
{
    auto captionLogicalHeight = caption.writingMode().isOrthogonal(tableWritingMode) ? caption.logicalWidth() : caption.logicalHeight();
    return captionLogicalHeight + caption.marginBefore(tableWritingMode) + caption.marginAfter(tableWritingMode);
}

void RenderTable::layoutCaption(RenderTableCaption& caption)
{
    LayoutRect captionRect(caption.borderBoxRectInContainer());

    auto setCaptionLocation = [&] {
        auto logicalLocation = LayoutPoint { caption.marginStart(writingMode()), caption.marginBefore(writingMode()) + logicalHeight() };
        caption.setLocation(writingMode().isHorizontal() ? logicalLocation : logicalLocation.transposedPoint());
    };

    if (caption.needsLayout()) {
        // The margins may not be available but ensure the caption is at least located beneath any previous sibling caption
        // so that it does not mistakenly think any floats in the previous caption intrude into it.
        setCaptionLocation();
        // If RenderTableCaption ever gets a layout() function, use it here.
        caption.layoutIfNeeded();
    }
    // Apply the margins to the location now that they are definitely available from layout
    setCaptionLocation();

    if (!selfNeedsLayout() && caption.checkForRepaintDuringLayout())
        caption.repaintDuringLayoutIfMoved(captionRect);

    setLogicalHeight(logicalHeight() + captionLogicalHeight(caption, writingMode()));
}

void RenderTable::layoutCaptions(BottomCaptionLayoutPhase bottomCaptionLayoutPhase)
{
    if (m_captions.isEmpty())
        return;
    // FIXME: Collapse caption margin.
    for (auto& caption : m_captions) {
        if ((bottomCaptionLayoutPhase == BottomCaptionLayoutPhase::Yes && caption->style().captionSide() != CaptionSide::Bottom)
            || (bottomCaptionLayoutPhase == BottomCaptionLayoutPhase::No && caption->style().captionSide() == CaptionSide::Bottom))
            continue;
        layoutCaption(*caption);
    }
}

void RenderTable::distributeExtraLogicalHeight(LayoutUnit extraLogicalHeight)
{
    if (extraLogicalHeight <= 0)
        return;

    // Collect tbody sections separately from thead/tfoot
    Vector<RenderTableSection*> tbodySections;
    for (CheckedPtr section = topSection(); section; section = sectionBelow(section)) {
        // A section is a tbody if it's not the header or footer.
        if (section != m_head.get() && section != m_foot.get())
            tbodySections.append(section);
    }

    // If we have tbody sections, distribute all extra height to them only.
    // This matches the expected behavior where thead/tfoot keep their intrinsic size.
    if (!tbodySections.isEmpty()) {
        LayoutUnit totalTBodyHeight = 0;
        for (auto& section : tbodySections)
            totalTBodyHeight += section->logicalHeight();

        // Distribute proportionally to each tbody's intrinsic height; if all tbodies
        // are empty, distribute equally. The last section absorbs any rounding remainder.
        auto shareForSection = [&](size_t sectionIndex) -> LayoutUnit {
            if (totalTBodyHeight > 0)
                return (extraLogicalHeight * tbodySections[sectionIndex]->logicalHeight()) / totalTBodyHeight;
            return extraLogicalHeight / tbodySections.size();
        };

        LayoutUnit remainingHeight = extraLogicalHeight;
        for (size_t sectionIndex = 0; sectionIndex < tbodySections.size(); ++sectionIndex) {
            LayoutUnit extraHeightForSection;
            if (sectionIndex == tbodySections.size() - 1)
                extraHeightForSection = remainingHeight;
            else {
                extraHeightForSection = shareForSection(sectionIndex);
                remainingHeight -= extraHeightForSection;
            }

            tbodySections[sectionIndex]->distributeExtraLogicalHeightToRows(extraHeightForSection);
        }
        return;
    }

    // No tbody sections - fall back to distributing to thead or tfoot.
    if (CheckedPtr section = topSection())
        section->distributeExtraLogicalHeightToRows(extraLogicalHeight);
}

void RenderTable::simplifiedNormalFlowLayout()
{
    for (auto& caption : m_captions)
        caption->layoutIfNeeded();
    for (RenderTableSection* section = topSection(); section; section = sectionBelow(section)) {
        section->layoutIfNeeded();
        section->layoutRows();
        section->computeOverflowFromCells();
        section->addVisualEffectOverflow();
    }
}

LayoutUnit RenderTable::sumCaptionsLogicalHeight() const
{
    LayoutUnit height;
    for (auto& caption : m_captions)
        height += captionLogicalHeight(*caption, writingMode());
    return height;
}

void RenderTable::setNeedsSectionRecalc()
{
    if (renderTreeBeingDestroyed())
        return;
    m_needsSectionRecalc = true;
    setNeedsLayout();
}

void RenderTable::layout()
{
    StackStats::LayoutCheckPoint layoutCheckPoint;
    ASSERT(needsLayout());

    if (simplifiedLayout())
        return;

    recalcSectionsIfNeeded();
    // FIXME: We should do this recalc lazily in borderStart/borderEnd so that we don't have to make sure
    // to call this before we call borderStart/borderEnd to avoid getting a stale value.
    recalcBordersInRowDirection();
    bool sectionMoved = false;
    LayoutUnit movedSectionLogicalTop;
    unsigned sectionCount = 0;
    bool shouldCacheContentLogicalHeightForFlexItem = true;

    LayoutRepainter repainter(*this);
    {
        LayoutStateMaintainer statePusher(*this, locationOffset(), isTransformed() || hasReflection() || writingMode().isBlockFlipped());

        LayoutUnit oldLogicalWidth = logicalWidth();
        LayoutUnit oldLogicalHeight = logicalHeight();
        updateLogicalWidth();
        if (logicalWidth() != oldLogicalWidth) {
            for (auto& caption : m_captions)
                caption->setNeedsLayout(MarkingBehavior::MarkOnlyThis);
        }
        resetLogicalHeightBeforeLayoutIfNeeded();
        // FIXME: The optimisation below doesn't work since the internal table
        // layout could have changed. We need to add a flag to the table
        // layout that tells us if something has changed in the min max
        // calculations to do it correctly.
        //     if ( oldWidth != width() || columns.size() + 1 != columnPos.size() )
        m_tableLayout->layout();

        LayoutUnit totalSectionLogicalHeight;
        LayoutUnit oldTableLogicalTop;
        for (auto& caption : m_captions) {
            if (caption->style().captionSide() == CaptionSide::Bottom)
                continue;
            oldTableLogicalTop += captionLogicalHeight(*caption, writingMode());
        }

        bool collapsing = collapseBorders();

        for (auto& child : childrenOfType<RenderElement>(*this)) {
            if (CheckedPtr section = dynamicDowncast<RenderTableSection>(child)) {
                if (m_columnLogicalWidthChanged)
                    section->setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);
                section->layoutIfNeeded();
                totalSectionLogicalHeight += section->calcRowLogicalHeight();
                if (collapsing)
                    section->recalcOuterBorder();
                ASSERT(!section->needsLayout());
            } else if (CheckedPtr column = dynamicDowncast<RenderTableCol>(child)) {
                column->layoutIfNeeded();
                ASSERT(!column->needsLayout());
            }
        }

        // If any table section moved vertically, we will just repaint everything from that
        // section down (it is quite unlikely that any of the following sections
        // did not shift).
        layoutCaptions();
        if (!m_captions.isEmpty() && logicalHeight() != oldTableLogicalTop) {
            sectionMoved = true;
            movedSectionLogicalTop = std::min(logicalHeight(), oldTableLogicalTop);
        }

        LayoutUnit borderAndPaddingBefore = borderBefore() + (collapsing ? 0_lu : paddingBefore());
        LayoutUnit borderAndPaddingAfter = borderAfter() + (collapsing ? 0_lu : paddingAfter());

        setLogicalHeight(logicalHeight() + borderAndPaddingBefore);

        LayoutUnit computedLogicalHeight;

        if (!isOutOfFlowPositioned())
            updateLogicalHeight();
        else {
            // Can't call updateLogicalHeight here - it would set logicalHeight breaking section positioning below (sections accumulate from borderAndPaddingBefore, not from the final table height).
            auto computedValues = computeLogicalHeight(logicalHeight(), 0_lu);
            computedLogicalHeight = computedValues.extent - borderAndPaddingBefore - borderAndPaddingAfter - sumCaptionsLogicalHeight();
        }

        auto& logicalHeightLength = style().logicalHeight();
        if (!isOutOfFlowPositioned() && (logicalHeightLength.isIntrinsicOrStretch() || (logicalHeightLength.isSpecified() && logicalHeightLength.isPossiblyPositive())))
            computedLogicalHeight = convertStyleLogicalHeightToComputedHeight(logicalHeightLength);

        if (auto overridingLogicalHeight = this->overridingBorderBoxLogicalHeight())
            computedLogicalHeight = std::max(computedLogicalHeight, *overridingLogicalHeight - (borderAndPaddingBefore + borderAndPaddingAfter) - sumCaptionsLogicalHeight());

        if (!shouldIgnoreLogicalMinMaxHeightSizes()) {
            auto& logicalMaxHeightLength = style().logicalMaxHeight();
            if (logicalMaxHeightLength.isStretch() || logicalMaxHeightLength.isSpecified()) {
                LayoutUnit computedMaxLogicalHeight = convertStyleLogicalHeightToComputedHeight(logicalMaxHeightLength);
                computedLogicalHeight = std::min(computedLogicalHeight, computedMaxLogicalHeight);
            }

            auto logicalMinHeightLength = style().logicalMinHeight();
            if (logicalMinHeightLength.isMinContent() || logicalMinHeightLength.isMaxContent() || logicalMinHeightLength.isFitContent())
                logicalMinHeightLength = CSS::Keyword::Auto { };
            if (logicalMinHeightLength.isIntrinsicOrStretch() || logicalMinHeightLength.isSpecified()) {
                LayoutUnit computedMinLogicalHeight = convertStyleLogicalHeightToComputedHeight(logicalMinHeightLength);
                computedLogicalHeight = std::max(computedLogicalHeight, computedMinLogicalHeight);
            }
        }

        distributeExtraLogicalHeight(computedLogicalHeight - totalSectionLogicalHeight);

        for (RenderTableSection* section = topSection(); section; section = sectionBelow(section))
            section->layoutRows();

        if (!topSection() && computedLogicalHeight > totalSectionLogicalHeight && !document().inQuirksMode()) {
            // Completely empty tables (with no sections or anything) should at least honor their
            // overriding or specified height in strict mode, but this value will not be cached.
            shouldCacheContentLogicalHeightForFlexItem = false;
            auto tableLogicalHeight = [&] {
                if (auto overridingLogicalHeight = this->overridingBorderBoxLogicalHeight())
                    return *overridingLogicalHeight - borderAndPaddingAfter;
                return logicalHeight() + computedLogicalHeight;
            };
            setLogicalHeight(tableLogicalHeight());
        }

        LayoutUnit sectionLogicalLeft = writingMode().isLogicalLeftInlineStart() ? borderStart() : borderEnd();
        if (!collapsing)
            sectionLogicalLeft += writingMode().isLogicalLeftInlineStart() ? paddingStart() : paddingEnd();

        // position the table sections
        RenderTableSection* section = topSection();
        while (section) {
            sectionCount++;
            if (!sectionMoved && section->logicalTop() != logicalHeight()) {
                sectionMoved = true;
                movedSectionLogicalTop = std::min(logicalHeight(), section->logicalTop()) + (writingMode().isHorizontal() ? section->visualOverflowRect().y() : section->visualOverflowRect().x());
            }
            section->setLogicalLocation(LayoutPoint(sectionLogicalLeft, logicalHeight()));

            setLogicalHeight(logicalHeight() + section->logicalHeight());
            section->addVisualEffectOverflow();
            
            section = sectionBelow(section);
        }

        setLogicalHeight(logicalHeight() + borderAndPaddingAfter);

        layoutCaptions(BottomCaptionLayoutPhase::Yes);

        if (isOutOfFlowPositioned())
            updateLogicalHeight();

        // Layout was changed, so probably borders too.
        invalidateCollapsedBorders();

        // The location or height of one or more sections may have changed.
        invalidateCachedColumnOffsets();

        computeInFlowOverflow(flippedContentBoxRect());

        // table can be containing block of positioned elements.
        bool dimensionChanged = oldLogicalWidth != logicalWidth() || oldLogicalHeight != logicalHeight();
        layoutOutOfFlowBoxes(dimensionChanged ? RelayoutChildren::Yes : RelayoutChildren::No);
        addOverflowFromOutOfFlowBoxes();

        updateLayerTransform();
    }

    auto* layoutState = view().frameView().layoutContext().layoutState();
    if (layoutState && layoutState->pageLogicalHeight())
        setPageLogicalOffset(layoutState->pageLogicalOffset(this, logicalTop()));

    bool didFullRepaint = repainter.repaintAfterLayout();
    // Repaint with our new bounds if they are different from our old bounds.
    if (!didFullRepaint && sectionMoved) {
        if (writingMode().isHorizontal())
            repaintRectangle(LayoutRect(visualOverflowRect().x(), movedSectionLogicalTop, visualOverflowRect().width(), visualOverflowRect().maxY() - movedSectionLogicalTop));
        else
            repaintRectangle(LayoutRect(movedSectionLogicalTop, visualOverflowRect().y(), visualOverflowRect().maxX() - movedSectionLogicalTop, visualOverflowRect().height()));
    }

    bool paginated = layoutState && layoutState->isPaginated();
    if (sectionCount && sectionMoved && paginated) {
        // FIXME: Table layout should always stabilize even when section moves (see webkit.org/b/174412).
        if (m_recursiveSectionMovedWithPaginationLevel < sectionCount) {
            SetForScope recursiveSectionMovedWithPaginationLevel(m_recursiveSectionMovedWithPaginationLevel, m_recursiveSectionMovedWithPaginationLevel + 1);
            markForPaginationRelayoutIfNeeded();
            layoutIfNeeded();
        } else
            ASSERT_NOT_REACHED();
    }
    
    // FIXME: This value isn't the intrinsic content logical height, but we need
    // to update the value as its used by flexbox layout. crbug.com/367324
    if (shouldCacheContentLogicalHeightForFlexItem) {
        if (CheckedPtr flexContainer = dynamicDowncast<RenderFlexibleBox>(parent()))
            flexContainer->setFlexItemContentLogicalHeightFromLayout(*this, contentBoxLogicalHeight());
    }

    m_columnLogicalWidthChanged = false;
    clearNeedsLayout();
}

void RenderTable::invalidateCollapsedBordersAfterStyleChangeIfNeeded(const Style::ComputedStyle& oldStyle, const Style::ComputedStyle& newStyle, RenderTableCell* cellWithStyleChange)
{
    auto shouldInvalidate = [&] {
        if (oldStyle.writingMode() != newStyle.writingMode())
            return true;
        return !Style::borderIsEquivalentForPainting(oldStyle, newStyle);
    };

    if (shouldInvalidate())
        invalidateCollapsedBorders(cellWithStyleChange);
}

void RenderTable::invalidateCollapsedBorders(RenderTableCell* cellWithStyleChange)
{
    m_collapsedBordersValid = false;
    m_collapsedBorders.clear();

    for (auto& section : childrenOfType<RenderTableSection>(*this))
        section.clearCachedCollapsedBorders();

    if (!m_collapsedEmptyBorderIsPresent)
        return;

    if (cellWithStyleChange) {
        // It is enough to invalidate just the surrounding cells when cell border style changes.
        cellWithStyleChange->invalidateHasEmptyCollapsedBorders();
        if (auto* below = cellBelow(cellWithStyleChange))
            below->invalidateHasEmptyCollapsedBorders();
        if (auto* above = cellAbove(cellWithStyleChange))
            above->invalidateHasEmptyCollapsedBorders();
        if (auto* before = cellBefore(cellWithStyleChange))
            before->invalidateHasEmptyCollapsedBorders();
        if (auto* after = cellAfter(cellWithStyleChange))
            after->invalidateHasEmptyCollapsedBorders();
        return;
    }

    for (auto& section : childrenOfType<RenderTableSection>(*this)) {
        for (auto* row = section.firstRow(); row; row = row->nextRow()) {
            for (auto* cell = row->firstCell(); cell; cell = cell->nextCell()) {
                ASSERT(cell->table() == this);
                cell->invalidateHasEmptyCollapsedBorders();
            }
        }
    }
    m_collapsedEmptyBorderIsPresent = false;
}

// Collect all the unique border values that we want to paint in a sorted list.
void RenderTable::recalcCollapsedBorders()
{
    if (m_collapsedBordersValid)
        return;
    m_collapsedBorders.clear();
    for (auto& section : childrenOfType<RenderTableSection>(*this)) {
        for (RenderTableRow* row = section.firstRow(); row; row = row->nextRow()) {
            for (RenderTableCell* cell = row->firstCell(); cell; cell = cell->nextCell()) {
                ASSERT(cell->table() == this);
                cell->collectBorderValues(m_collapsedBorders);
            }
        }
    }
    RenderTableCell::sortBorderValues(m_collapsedBorders);
    m_collapsedBordersValid = true;
}

void RenderTable::addOverflowFromInFlowChildren(OptionSet<ComputeOverflowOptions> options)
{
    UNUSED_PARAM(options);

    // Add overflow from borders.
    // Technically it's odd that we are incorporating the borders into layout overflow, which is only supposed to be about overflow from our
    // descendant objects, but since tables don't support overflow:auto, this works out fine.
    if (collapseBorders()) {
        LayoutUnit rightBorderOverflow = borderBoxWidth() + outerBorderRight() - borderRight();
        LayoutUnit leftBorderOverflow = borderLeft() - outerBorderLeft();
        LayoutUnit bottomBorderOverflow = borderBoxHeight() + outerBorderBottom() - borderBottom();
        LayoutUnit topBorderOverflow = borderTop() - outerBorderTop();
        LayoutRect borderOverflowRect(leftBorderOverflow, topBorderOverflow, rightBorderOverflow - leftBorderOverflow, bottomBorderOverflow - topBorderOverflow);
        if (borderOverflowRect != borderBoxRect()) {
            // Do NOT add layout overflow for collapsed borders — they are a
            // painting artifact and must not inflate the scroll container's
            // scrollWidth/scrollHeight.
            // See https://github.com/w3c/csswg-drafts/issues/6230
            addVisualOverflow(borderOverflowRect);
        }
    }

    // Add overflow from our caption.
    for (auto& caption : m_captions) {
        if (caption)
            addOverflowFromContainedBox(*caption);
    }

    // Add overflow from our sections.
    for (auto* section = topSection(); section; section = sectionBelow(section))
        addOverflowFromContainedBox(*section);
}

void RenderTable::paint(PaintInfo& paintInfo, const LayoutPoint& paintOffset)
{
    auto isSkippedContent = [&] {
        if (style().usedContentVisibility() == ContentVisibility::Visible)
            return false;
        // FIXME: Tables can never be skipped content roots. If a table is _inside_ a skipped subtree, we should have bailed out at the skipped root ancestor.
        if (auto* containingBlock = this->containingBlock(); containingBlock && containingBlock->isAnonymousBlock() && !containingBlock->style().isSkippedRootOrSkippedContent())
            return true;
        return false;
    };
    if (isSkippedContent())
        return;

    LayoutPoint adjustedPaintOffset = paintOffset + location();

    PaintPhase paintPhase = paintInfo.phase;

    if (!isDocumentElementRenderer()) {
        LayoutRect overflowBox = visualOverflowRect();
        flipForWritingMode(overflowBox);
        overflowBox.moveBy(adjustedPaintOffset);
        if (!overflowBox.intersects(paintInfo.rect))
            return;
    }

    bool pushedClip = pushContentsClip(paintInfo, adjustedPaintOffset);
    paintObject(paintInfo, adjustedPaintOffset);
    if (pushedClip)
        popContentsClip(paintInfo, paintPhase, adjustedPaintOffset);
}

void RenderTable::paintObject(PaintInfo& paintInfo, const LayoutPoint& paintOffset)
{
    PaintPhase paintPhase = paintInfo.phase;
    if ((paintPhase == PaintPhase::BlockBackground || paintPhase == PaintPhase::ChildBlockBackground) && hasVisibleBoxDecorations() && style().usedVisibility() == Visibility::Visible)
        paintBoxDecorations(paintInfo, paintOffset);

    if (paintPhase == PaintPhase::Mask) {
        paintMask(paintInfo, paintOffset);
        return;
    }

    if (paintPhase == PaintPhase::Accessibility)
        paintInfo.accessibilityRegionContext()->takeBounds(*this, paintOffset);

    // We're done.  We don't bother painting any children.
    if (paintPhase == PaintPhase::BlockBackground)
        return;
    
    // We don't paint our own background, but we do let the kids paint their backgrounds.
    if (paintPhase == PaintPhase::ChildBlockBackgrounds)
        paintPhase = PaintPhase::ChildBlockBackground;

    PaintInfo info(paintInfo);
    info.phase = paintPhase;
    info.updateSubtreePaintRootForChildren(this);

    for (auto& box : childrenOfType<RenderBox>(*this)) {
        if (!box.hasSelfPaintingLayer() && (box.isRenderTableSection() || box.isRenderTableCaption())) {
            LayoutPoint childPoint = flipForWritingModeForChild(box, paintOffset);
            box.paint(info, childPoint);
        }
    }
    
    if (collapseBorders() && paintPhase == PaintPhase::ChildBlockBackground && style().usedVisibility() == Visibility::Visible) {
        paintCollapsedBorderPasses(info, [&](PaintInfo& borderPaintInfo) {
            for (RenderTableSection* section = bottomSection(); section; section = sectionAbove(section)) {
                LayoutPoint childPoint = flipForWritingModeForChild(*section, paintOffset);
                section->paint(borderPaintInfo, childPoint);
            }
        });
    }

    // Paint outline.
    if ((paintPhase == PaintPhase::Outline || paintPhase == PaintPhase::SelfOutline) && hasOutline() && style().usedVisibility() == Visibility::Visible)
        paintOutline(paintInfo, LayoutRect(paintOffset, borderBoxSize()));
}

template<typename Function>
void RenderTable::paintCollapsedBorderPasses(const PaintInfo& paintInfo, NOESCAPE const Function& paintPass)
{
    ASSERT(collapseBorders());
    recalcCollapsedBorders();

    PaintInfo borderPaintInfo(paintInfo);
    borderPaintInfo.phase = PaintPhase::CollapsedTableBorders;

    for (auto& border : m_collapsedBorders) {
        m_currentBorder = &border;
        paintPass(borderPaintInfo);
    }
    m_currentBorder = nullptr;
}

void RenderTable::paintCollapsedBordersForRow(PaintInfo& paintInfo, RenderTableRow& row, const LayoutPoint& paintOffset)
{
    paintCollapsedBorderPasses(paintInfo, [&](PaintInfo& borderPaintInfo) {
        for (CheckedPtr cell = row.firstCell(); cell; cell = cell->nextCell()) {
            if (!cell->hasSelfPaintingLayer()) {
                auto cellPoint = row.flipForWritingModeForChild(*cell, paintOffset);
                cell->paintCollapsedBorders(borderPaintInfo, cellPoint);
            }
        }
    });
}

void RenderTable::adjustBorderBoxRectForPainting(LayoutRect& rect)
{
    for (auto& caption : m_captions) {
        auto captionLogicalHeightInTableWritingMode = captionLogicalHeight(*caption, writingMode());
        bool captionIsBefore = (caption->style().captionSide() != CaptionSide::Bottom) ^ writingMode().isBlockFlipped();
        if (writingMode().isHorizontal()) {
            rect.setHeight(rect.height() - captionLogicalHeightInTableWritingMode);
            if (captionIsBefore)
                rect.move(0_lu, captionLogicalHeightInTableWritingMode);
        } else {
            rect.setWidth(rect.width() - captionLogicalHeightInTableWritingMode);
            if (captionIsBefore)
                rect.move(captionLogicalHeightInTableWritingMode, 0_lu);
        }
    }
    
    RenderBlock::adjustBorderBoxRectForPainting(rect);
}

void RenderTable::paintBoxDecorations(PaintInfo& paintInfo, const LayoutPoint& paintOffset)
{
    if (!paintInfo.shouldPaintWithinRoot(*this))
        return;

    LayoutRect rect(paintOffset, borderBoxSize());
    adjustBorderBoxRectForPainting(rect);

    BackgroundPainter backgroundPainter { *this, paintInfo };

    auto bleedAvoidance = determineBleedAvoidance(paintInfo.context());
    if (!BackgroundPainter::boxShadowShouldBeAppliedToBackground(*this, rect.location(), bleedAvoidance, { }))
        backgroundPainter.paintBoxShadow(rect, style(), Style::ShadowStyle::Normal);

    GraphicsContextStateSaver stateSaver(paintInfo.context(), false);
    if (bleedAvoidance == BleedAvoidance::UseTransparencyLayer) {
        // To avoid the background color bleeding out behind the border, we'll render background and border
        // into a transparency layer, and then clip that in one go (which requires setting up the clip before
        // beginning the layer).
        stateSaver.save();
        auto borderShape = BorderShape::shapeForBorderRect(style(), rect);
        borderShape.clipToOuterShape(paintInfo.context(), style().deviceScaleFactor());
        paintInfo.context().beginTransparencyLayer(1);
    }

    backgroundPainter.paintBackground(rect, bleedAvoidance);
    backgroundPainter.paintBoxShadow(rect, style(), Style::ShadowStyle::Inset);

    if (style().border().hasVisibleBorderDecoration() && !collapseBorders())
        BorderPainter { *this, paintInfo }.paintBorder(rect, style());

    if (bleedAvoidance == BleedAvoidance::UseTransparencyLayer)
        paintInfo.context().endTransparencyLayer();
}

void RenderTable::paintMask(PaintInfo& paintInfo, const LayoutPoint& paintOffset)
{
    if (style().usedVisibility() != Visibility::Visible || paintInfo.phase != PaintPhase::Mask)
        return;

    LayoutRect rect(paintOffset, borderBoxSize());
    adjustBorderBoxRectForPainting(rect);

    paintMaskImages(paintInfo, rect);
}

std::pair<LayoutUnit, LayoutUnit> RenderTable::computeIntrinsicLogicalWidths(TableIntrinsics intrinsics) const
{
    recalcSectionsIfNeeded();
    // FIXME: Do the recalc in borderStart/borderEnd and make those const_cast this call.
    // Then m_borderStart/m_borderEnd will be transparent a cache and it removes the possibility
    // of reading out stale values.
    // FIXME: Restructure the table layout code so that we can make this method const.
    // FIXME: We should include captions widths here like we do in computeIntrinsicLogicalWidthContributions.
    const_cast<RenderTable*>(this)->recalcBordersInRowDirection();
    return const_cast<RenderTable*>(this)->m_tableLayout->computeIntrinsicLogicalWidths(intrinsics);
}

std::pair<LayoutUnit, LayoutUnit> RenderTable::computeIntrinsicLogicalWidths() const
{
    return computeIntrinsicLogicalWidths(TableIntrinsics::ForLayout);
}

std::pair<LayoutUnit, LayoutUnit> RenderTable::computeIntrinsicKeywordLogicalWidths() const
{
    return computeIntrinsicLogicalWidths(TableIntrinsics::ForKeyword);
}

void RenderTable::computeIntrinsicLogicalWidthContributions()
{
    ASSERT(hasInvalidContentLogicalWidths());

    std::tie(m_minContentLogicalWidthContribution, m_maxContentLogicalWidthContribution) = computeIntrinsicLogicalWidths();

    LayoutUnit bordersPaddingAndSpacing = bordersPaddingAndSpacingInRowDirection();
    m_minContentLogicalWidthContribution += bordersPaddingAndSpacing;
    m_maxContentLogicalWidthContribution += bordersPaddingAndSpacing;

    m_tableLayout->applyContentLogicalWidthQuirks(m_minContentLogicalWidthContribution, m_maxContentLogicalWidthContribution);

    for (auto& caption : m_captions) {
        LayoutUnit captionMinWidth = caption->minContentLogicalWidthContribution();
        captionMinWidth += marginIntrinsicLogicalWidthForChild(*caption);

        m_minContentLogicalWidthContribution = std::max(m_minContentLogicalWidthContribution, captionMinWidth);
    }
    m_maxContentLogicalWidthContribution = std::max(m_maxContentLogicalWidthContribution, m_minContentLogicalWidthContribution);

    auto& styleToUse = style();
    // FIXME: This should probably be checking for isSpecified since you should be able to use percentage or calc values for min-width.
    if (auto fixedLogicalMinWidth = styleToUse.logicalMinWidth().tryFixed(); fixedLogicalMinWidth && fixedLogicalMinWidth->isPositive()) {
        m_maxContentLogicalWidthContribution = std::max(m_maxContentLogicalWidthContribution, adjustContentBoxLogicalWidthForBoxSizing(*fixedLogicalMinWidth));
        m_minContentLogicalWidthContribution = std::max(m_minContentLogicalWidthContribution, adjustContentBoxLogicalWidthForBoxSizing(*fixedLogicalMinWidth));
    }

    // FIXME: This should probably be checking for isSpecified since you should be able to use percentage or calc values for maxWidth.
    if (auto fixedLogicalMaxWidth = styleToUse.logicalMaxWidth().tryFixed()) {
        m_maxContentLogicalWidthContribution = std::min(m_maxContentLogicalWidthContribution, adjustContentBoxLogicalWidthForBoxSizing(*fixedLogicalMaxWidth));
        m_maxContentLogicalWidthContribution = std::max(m_maxContentLogicalWidthContribution, m_minContentLogicalWidthContribution);
    }

    // FIXME: We should be adding borderAndPaddingLogicalWidth here, but m_tableLayout->computeIntrinsicLogicalWidthContributions already does,
    // so a bunch of tests break doing this naively.
    clearContentLogicalWidthsInvalidation();

    // Row widths are set by the section, not computed from preferred widths,
    // so their dirty bit is never cleared by the normal preferred width
    // computation. Clear it here so it doesn't block subsequent invalidation
    // from propagating through the row to the table.
    for (CheckedPtr section = topSection(); section; section = sectionBelow(section)) {
        section->clearContentLogicalWidthsInvalidation();
        for (CheckedPtr row = section->firstRow(); row; row = row->nextRow())
            row->clearContentLogicalWidthsInvalidation();
    }
}

RenderTableSection* RenderTable::topNonEmptySection() const
{
    RenderTableSection* section = topSection();
    if (section && !section->numRows())
        section = sectionBelow(section, SkipEmptySections::Yes);
    return section;
}

RenderTableSection* RenderTable::bottomNonEmptySection() const
{
    auto* section = bottomSection();
    if (section && !section->numRows())
        section = sectionAbove(section, SkipEmptySections::Yes);
    return section;
}

void RenderTable::splitColumn(unsigned position, unsigned firstSpan)
{
    // We split the column at "position", taking "firstSpan" cells from the span.
    ASSERT(m_columns[position].span > firstSpan);
    m_columns.insert(position, { firstSpan });
    m_columns[position + 1].span -= firstSpan;

    // Propagate the change in our columns representation to the sections that don't need
    // cell recalc. If they do, they will be synced up directly with m_columns later.
    for (auto& section : childrenOfType<RenderTableSection>(*this)) {
        if (section.needsCellRecalc())
            continue;

        section.splitColumn(position, firstSpan);
    }

    m_columnPos.grow(numEffCols() + 1);
}

void RenderTable::appendColumn(unsigned span)
{
    unsigned newColumnIndex = m_columns.size();
    m_columns.append({ span });

    // Unless the table has cell(s) with colspan that exceed the number of columns afforded
    // by the other rows in the table we can use the fast path when mapping columns to effective columns.
    m_hasCellColspanThatDeterminesTableWidth = m_hasCellColspanThatDeterminesTableWidth || span > 1;

    // Propagate the change in our columns representation to the sections that don't need
    // cell recalc. If they do, they will be synced up directly with m_columns later.
    for (auto& section : childrenOfType<RenderTableSection>(*this)) {
        if (section.needsCellRecalc())
            continue;

        section.appendColumn(newColumnIndex);
    }

    m_columnPos.grow(numEffCols() + 1);
}

LayoutUnit RenderTable::borderSpacingInRowDirection() const
{
    unsigned effectiveColumnCount = numEffCols();
    if (!effectiveColumnCount)
        return { };

    unsigned collapsedColumnCount = 0;
    for (unsigned i = 0; i < effectiveColumnCount; ++i) {
        if (auto* col = colElement(i); col && col->style().visibility() == Visibility::Collapse)
            ++collapsedColumnCount;
    }

    unsigned visibleColumnCount = effectiveColumnCount - collapsedColumnCount;
    return (visibleColumnCount + 1) * hBorderSpacing();
}

RenderTableCol* RenderTable::firstColumn() const
{
    for (auto& child : childrenOfType<RenderObject>(*this)) {
        if (auto* column = dynamicDowncast<RenderTableCol>(child))
            return const_cast<RenderTableCol*>(column);
    }
    return nullptr;
}

void RenderTable::updateColumnCache() const
{
    ASSERT(m_hasColElements);
    ASSERT(m_columnRenderers.isEmpty());
    ASSERT(m_effectiveColumnIndexMap.isEmpty());
    ASSERT(!m_columnRenderersValid);

    unsigned columnIndex = 0;
    for (RenderTableCol* columnRenderer = firstColumn(); columnRenderer; columnRenderer = columnRenderer->nextColumn()) {
        if (columnRenderer->isTableColumnGroupWithColumnChildren())
            continue;
        m_columnRenderers.append(columnRenderer);
        // FIXME: We should look to compute the effective column index successively from previous values instead of
        // calling colToEffCol(), which is in O(numEffCols()). Although it's unlikely that this is a hot function.
        m_effectiveColumnIndexMap.add(*columnRenderer, colToEffCol(columnIndex));
        columnIndex += columnRenderer->span();
    }
    m_columnRenderersValid = true;
}

unsigned RenderTable::effectiveIndexOfColumn(const RenderTableCol& column) const
{
    if (!m_columnRenderersValid)
        updateColumnCache();
    const RenderTableCol* columnToUse = &column;
    if (columnToUse->isTableColumnGroupWithColumnChildren())
        columnToUse = columnToUse->nextColumn(); // First column in column-group
    auto it = m_effectiveColumnIndexMap.find(columnToUse);
    ASSERT(it != m_effectiveColumnIndexMap.end());
    if (it == m_effectiveColumnIndexMap.end())
        return std::numeric_limits<unsigned>::max();
    return it->value;
}

LayoutUnit RenderTable::offsetTopForColumn(const RenderTableCol& column) const
{
    if (effectiveIndexOfColumn(column) >= numEffCols())
        return 0;
    if (m_columnOffsetTop >= 0) {
        ASSERT(!needsLayout());
        return m_columnOffsetTop;
    }
    RenderTableSection* section = topNonEmptySection();
    return m_columnOffsetTop = section ? section->offsetTop() : 0_lu;
}

LayoutUnit RenderTable::offsetLeftForColumn(const RenderTableCol& column) const
{
    unsigned columnIndex = effectiveIndexOfColumn(column);
    if (columnIndex >= numEffCols())
        return 0;
    return m_columnPos[columnIndex] + m_hSpacing + borderLeft();
}

LayoutUnit RenderTable::offsetWidthForColumn(const RenderTableCol& column) const
{
    const RenderTableCol* currentColumn = &column;
    bool hasColumnChildren = currentColumn->isTableColumnGroupWithColumnChildren();
    if (hasColumnChildren)
        currentColumn = currentColumn->nextColumn(); // First column in column-group
    unsigned numberOfEffectiveColumns = numEffCols();
    ASSERT_WITH_SECURITY_IMPLICATION(m_columnPos.size() >= numberOfEffectiveColumns + 1);
    LayoutUnit width;
    LayoutUnit spacing = m_hSpacing;
    while (currentColumn) {
        unsigned columnIndex = effectiveIndexOfColumn(*currentColumn);
        unsigned span = currentColumn->span();
        while (span && columnIndex < numberOfEffectiveColumns) {
            width += m_columnPos[columnIndex + 1] - m_columnPos[columnIndex] - spacing;
            span -= m_columns[columnIndex].span;
            ++columnIndex;
            if (span)
                width += spacing;
        }
        if (!hasColumnChildren)
            break;
        currentColumn = currentColumn->nextColumn();
        if (!currentColumn || currentColumn->isTableColumnGroup())
            break;
        width += spacing;
    }
    return width;
}

LayoutUnit RenderTable::offsetHeightForColumn(const RenderTableCol& column) const
{
    if (effectiveIndexOfColumn(column) >= numEffCols())
        return 0;
    if (m_columnOffsetHeight >= 0) {
        ASSERT(!needsLayout());
        return m_columnOffsetHeight;
    }
    LayoutUnit height;
    for (RenderTableSection* section = topSection(); section; section = sectionBelow(section))
        height += section->offsetHeight();
    m_columnOffsetHeight = height;
    return m_columnOffsetHeight;
}

RenderTableCol* RenderTable::slowColElement(unsigned col, bool* startEdge, bool* endEdge) const
{
    ASSERT(m_hasColElements);

    if (!m_columnRenderersValid)
        updateColumnCache();

    unsigned columnCount = 0;
    for (auto& columnRenderer : m_columnRenderers) {
        if (!columnRenderer)
            continue;
        unsigned span = columnRenderer->span();
        unsigned startCol = columnCount;
        ASSERT(span >= 1);
        unsigned endCol = columnCount + span - 1;
        columnCount += span;
        if (columnCount > col) {
            if (startEdge)
                *startEdge = startCol == col;
            if (endEdge)
                *endEdge = endCol == col;
            return columnRenderer.get();
        }
    }
    return nullptr;
}

void RenderTable::recalcSections() const
{
    ASSERT(m_needsSectionRecalc);

    m_head.clear();
    m_foot.clear();
    m_firstBody.clear();
    m_hasColElements = false;
    m_hasCellColspanThatDeterminesTableWidth = hasCellColspanThatDeterminesTableWidth();

    // We need to get valid pointers to caption, head, foot and first body again
    for (auto* child = firstChildBox(); child; child = child->nextSiblingBox()) {
        switch (child->style().display().value) {
        case Style::DisplayType::TableColumn:
        case Style::DisplayType::TableColumnGroup:
            m_hasColElements = true;
            break;
        case Style::DisplayType::TableHeaderGroup:
            if (CheckedPtr section = dynamicDowncast<RenderTableSection>(*child)) {
                if (!m_head)
                    m_head = *section;
                else if (!m_firstBody)
                    m_firstBody = *section;
                section->recalcCellsIfNeeded();
            }
            break;
        case Style::DisplayType::TableFooterGroup:
            if (CheckedPtr section = dynamicDowncast<RenderTableSection>(*child)) {
                if (!m_foot)
                    m_foot = *section;
                else if (!m_firstBody)
                    m_firstBody = *section;
                section->recalcCellsIfNeeded();
            }
            break;
        case Style::DisplayType::TableRowGroup:
            if (CheckedPtr section = dynamicDowncast<RenderTableSection>(*child)) {
                if (!m_firstBody)
                    m_firstBody = *section;
                section->recalcCellsIfNeeded();
            }
            break;
        default:
            break;
        }
    }

    // repair column count (addChild can grow it too much, because it always adds elements to the last row of a section)
    unsigned maxCols = 0;
    for (auto& section : childrenOfType<RenderTableSection>(*this)) {
        unsigned sectionCols = section.numColumns();
        if (sectionCols > maxCols)
            maxCols = sectionCols;
    }
    
    m_columns.resize(maxCols);
    m_columnPos.resize(maxCols + 1);

    // Now that we know the number of maximum number of columns, let's shrink the sections grids if needed.
    for (auto& section : childrenOfType<RenderTableSection>(const_cast<RenderTable&>(*this)))
        section.removeRedundantColumns();

    ASSERT(selfNeedsLayout() || !wasSkippedDuringLastLayoutDueToContentVisibility() || *wasSkippedDuringLastLayoutDueToContentVisibility());

    m_needsSectionRecalc = false;
}

LayoutUnit RenderTable::calcBorderStart() const
{
    if (!collapseBorders())
        return RenderBlock::borderStart();

    // Determined by the first cell of the first row. See the CSS 2.1 spec, section 17.6.2.
    if (!numEffCols())
        return 0;

    float borderWidth = 0;

    const BorderValue& tableStartBorder = style().borderStart();
    if (tableStartBorder.hasHiddenStyle())
        return 0;
    auto deviceScaleFactor = style().deviceScaleFactor();
    if (tableStartBorder.hasVisibleStyle())
        borderWidth = Style::evaluate<float>(tableStartBorder.width, style().usedZoomForLength(), deviceScaleFactor);

    if (RenderTableCol* column = colElement(0)) {
        // FIXME: We don't account for direction on columns and column groups.
        const BorderValue& columnAdjoiningBorder = column->style().borderStart();
        if (columnAdjoiningBorder.hasHiddenStyle())
            return 0;
        if (columnAdjoiningBorder.hasVisibleStyle())
            borderWidth = std::max(borderWidth, Style::evaluate<float>(columnAdjoiningBorder.width, column->style().usedZoomForLength(), deviceScaleFactor));
        // FIXME: This logic doesn't properly account for the first column in the first column-group case.
    }

    if (const RenderTableSection* topNonEmptySection = this->topNonEmptySection()) {
        const BorderValue& sectionAdjoiningBorder = topNonEmptySection->borderAdjoiningTableStart();
        if (sectionAdjoiningBorder.hasHiddenStyle())
            return 0;

        if (sectionAdjoiningBorder.hasVisibleStyle())
            borderWidth = std::max(borderWidth, Style::evaluate<float>(sectionAdjoiningBorder.width, topNonEmptySection->style().usedZoomForLength(), deviceScaleFactor));

        if (const RenderTableCell* adjoiningStartCell = topNonEmptySection->cellAt(0, 0).primaryCell()) {
            // FIXME: Make this work with perpendicular and flipped cells.
            const BorderValue& startCellAdjoiningBorder = adjoiningStartCell->borderAdjoiningTableStart();
            if (startCellAdjoiningBorder.hasHiddenStyle())
                return 0;

            const BorderValue& firstRowAdjoiningBorder = adjoiningStartCell->row()->borderAdjoiningTableStart();
            if (firstRowAdjoiningBorder.hasHiddenStyle())
                return 0;

            if (startCellAdjoiningBorder.hasVisibleStyle())
                borderWidth = std::max(borderWidth, Style::evaluate<float>(startCellAdjoiningBorder.width, adjoiningStartCell->style().usedZoomForLength(), deviceScaleFactor));
            if (firstRowAdjoiningBorder.hasVisibleStyle())
                borderWidth = std::max(borderWidth, Style::evaluate<float>(firstRowAdjoiningBorder.width, adjoiningStartCell->row()->style().usedZoomForLength(), deviceScaleFactor));
        }
    }
    return CollapsedBorderValue::adjustedCollapsedBorderWidth(borderWidth, deviceScaleFactor, writingMode().isInlineFlipped());
}

LayoutUnit RenderTable::calcBorderEnd() const
{
    if (!collapseBorders())
        return RenderBlock::borderEnd();

    // Determined by the last cell of the first row. See the CSS 2.1 spec, section 17.6.2.
    if (!numEffCols())
        return 0;

    float borderWidth = 0;

    const BorderValue& tableEndBorder = style().borderEnd();
    if (tableEndBorder.hasHiddenStyle())
        return 0;
    auto deviceScaleFactor = style().deviceScaleFactor();
    if (tableEndBorder.hasVisibleStyle())
        borderWidth = Style::evaluate<float>(tableEndBorder.width, style().usedZoomForLength(), deviceScaleFactor);

    unsigned endColumn = numEffCols() - 1;
    if (RenderTableCol* column = colElement(endColumn)) {
        // FIXME: We don't account for direction on columns and column groups.
        auto& columnAdjoiningStyle = column->style();
        auto& columnAdjoiningBorder = columnAdjoiningStyle.borderEnd();
        if (columnAdjoiningBorder.hasHiddenStyle())
            return 0;
        if (columnAdjoiningBorder.hasVisibleStyle())
            borderWidth = std::max(borderWidth, Style::evaluate<float>(columnAdjoiningBorder.width, columnAdjoiningStyle.usedZoomForLength(), deviceScaleFactor));
        // FIXME: This logic doesn't properly account for the last column in the last column-group case.
    }

    if (const RenderTableSection* topNonEmptySection = this->topNonEmptySection()) {
        auto& sectionAdjoiningStyle = topNonEmptySection->style();
        auto& sectionAdjoiningBorder = topNonEmptySection->borderAdjoiningTableEnd();
        if (sectionAdjoiningBorder.hasHiddenStyle())
            return 0;

        if (sectionAdjoiningBorder.hasVisibleStyle())
            borderWidth = std::max(borderWidth, Style::evaluate<float>(sectionAdjoiningBorder.width, sectionAdjoiningStyle.usedZoomForLength(), deviceScaleFactor));

        if (const RenderTableCell* adjoiningEndCell = topNonEmptySection->cellAt(0, lastColumnIndex()).primaryCell()) {
            // FIXME: Make this work with perpendicular and flipped cells.
            auto& endCellAdjoiningStyle = adjoiningEndCell->style();
            auto& endCellAdjoiningBorder = adjoiningEndCell->borderAdjoiningTableEnd();
            if (endCellAdjoiningBorder.hasHiddenStyle())
                return 0;

            auto& firstRowAdjoiningStyle = adjoiningEndCell->row()->style();
            auto& firstRowAdjoiningBorder = adjoiningEndCell->row()->borderAdjoiningTableEnd();
            if (firstRowAdjoiningBorder.hasHiddenStyle())
                return 0;

            if (endCellAdjoiningBorder.hasVisibleStyle())
                borderWidth = std::max(borderWidth, Style::evaluate<float>(endCellAdjoiningBorder.width, endCellAdjoiningStyle.usedZoomForLength(), deviceScaleFactor));
            if (firstRowAdjoiningBorder.hasVisibleStyle())
                borderWidth = std::max(borderWidth, Style::evaluate<float>(firstRowAdjoiningBorder.width, firstRowAdjoiningStyle.usedZoomForLength(), deviceScaleFactor));
        }
    }
    return CollapsedBorderValue::adjustedCollapsedBorderWidth(borderWidth, deviceScaleFactor, !writingMode().isInlineFlipped());
}

void RenderTable::recalcBordersInRowDirection()
{
    // FIXME: We need to compute the collapsed before / after borders in the same fashion.
    m_borderStart = calcBorderStart();
    m_borderEnd = calcBorderEnd();
}

LayoutUnit RenderTable::borderBefore() const
{
    if (collapseBorders()) {
        recalcSectionsIfNeeded();
        return outerBorderBefore();
    }
    return RenderBlock::borderBefore();
}

LayoutUnit RenderTable::borderAfter() const
{
    if (collapseBorders()) {
        recalcSectionsIfNeeded();
        return outerBorderAfter();
    }
    return RenderBlock::borderAfter();
}

LayoutUnit RenderTable::outerBorderBefore() const
{
    if (!collapseBorders())
        return 0;

    LayoutUnit borderWidth;

    if (RenderTableSection* topSection = this->topSection()) {
        borderWidth = topSection->outerBorderBefore();
        if (borderWidth < 0)
            return 0;   // Overridden by hidden
    }

    auto& tb = style().borderBefore();
    if (tb.hasHiddenStyle())
        return 0;
    if (tb.hasVisibleStyle()) {
        auto deviceScaleFactor = style().deviceScaleFactor();
        LayoutUnit collapsedBorderWidth = std::max(borderWidth, LayoutUnit(Style::evaluate<float>(tb.width, style().usedZoomForLength(), deviceScaleFactor) / 2));
        borderWidth = floorToDevicePixel(collapsedBorderWidth, deviceScaleFactor);
    }
    return borderWidth;
}

LayoutUnit RenderTable::outerBorderAfter() const
{
    if (!collapseBorders())
        return 0;

    LayoutUnit borderWidth;

    if (RenderTableSection* section = bottomSection()) {
        borderWidth = section->outerBorderAfter();
        if (borderWidth < 0)
            return 0; // Overridden by hidden
    }
    auto& tb = style().borderAfter();
    if (tb.hasHiddenStyle())
        return 0;
    if (tb.hasVisibleStyle()) {
        auto deviceScaleFactor = style().deviceScaleFactor();
        LayoutUnit collapsedBorderWidth = std::max(borderWidth, LayoutUnit((Style::evaluate<float>(tb.width, style().usedZoomForLength(), deviceScaleFactor) + (1 / deviceScaleFactor)) / 2));
        borderWidth = floorToDevicePixel(collapsedBorderWidth, deviceScaleFactor);
    }
    return borderWidth;
}

LayoutUnit RenderTable::outerBorderStart() const
{
    if (!collapseBorders())
        return 0;

    auto& tb = style().borderStart();
    if (tb.hasHiddenStyle())
        return 0;
    if (tb.hasVisibleStyle()) {
        auto deviceScaleFactor = style().deviceScaleFactor();
        return CollapsedBorderValue::adjustedCollapsedBorderWidth(Style::evaluate<float>(tb.width, style().usedZoomForLength(), deviceScaleFactor), deviceScaleFactor, writingMode().isInlineFlipped());
    }

    bool allHidden = true;
    LayoutUnit borderWidth;

    for (RenderTableSection* section = topSection(); section; section = sectionBelow(section)) {
        LayoutUnit sw = section->outerBorderStart();
        if (sw < 0)
            continue;
        allHidden = false;
        borderWidth = std::max(borderWidth, sw);
    }
    if (allHidden)
        return 0;

    return borderWidth;
}

LayoutUnit RenderTable::outerBorderEnd() const
{
    if (!collapseBorders())
        return 0;

    auto& tb = style().borderEnd();
    if (tb.hasHiddenStyle())
        return 0;
    if (tb.hasVisibleStyle()) {
        auto deviceScaleFactor = style().deviceScaleFactor();
        return CollapsedBorderValue::adjustedCollapsedBorderWidth(Style::evaluate<float>(tb.width, style().usedZoomForLength(), deviceScaleFactor), deviceScaleFactor, !writingMode().isInlineFlipped());
    }

    bool allHidden = true;
    LayoutUnit borderWidth;

    for (RenderTableSection* section = topSection(); section; section = sectionBelow(section)) {
        LayoutUnit sw = section->outerBorderEnd();
        if (sw < 0)
            continue;
        allHidden = false;
        borderWidth = std::max(borderWidth, sw);
    }
    if (allHidden)
        return 0;

    return borderWidth;
}

RenderTableSection* RenderTable::sectionAbove(const RenderTableSection* section, SkipEmptySections skipEmptySections) const
{
    recalcSectionsIfNeeded();

    if (section == m_head)
        return nullptr;

    RenderObject* prevSection = section == m_foot ? lastChild() : section->previousSibling();
    while (prevSection) {
        auto* tableSection = dynamicDowncast<RenderTableSection>(*prevSection);
        if (tableSection && prevSection != m_head && prevSection != m_foot && (skipEmptySections == SkipEmptySections::No || tableSection->numRows()))
            return tableSection;
        prevSection = prevSection->previousSibling();
    }
    if (!prevSection && m_head && (skipEmptySections == SkipEmptySections::No || m_head->numRows()))
        return m_head.get();
    return nullptr;
}

RenderTableSection* RenderTable::sectionBelow(const RenderTableSection* section, SkipEmptySections skipEmptySections) const
{
    recalcSectionsIfNeeded();

    if (section == m_foot)
        return nullptr;

    RenderObject* nextSection = section == m_head ? firstChild() : section->nextSibling();
    while (nextSection) {
        auto* tableSection = dynamicDowncast<RenderTableSection>(*nextSection);
        if (tableSection && nextSection != m_head && nextSection != m_foot && (skipEmptySections == SkipEmptySections::No || tableSection->numRows()))
            return tableSection;
        nextSection = nextSection->nextSibling();
    }
    if (!nextSection && m_foot && (skipEmptySections == SkipEmptySections::No || m_foot->numRows()))
        return m_foot.get();
    return nullptr;
}

RenderTableCell* RenderTable::cellAbove(const RenderTableCell* cell) const
{
    recalcSectionsIfNeeded();

    // Find the section and row to look in
    unsigned r = cell->rowIndex();
    RenderTableSection* section = nullptr;
    unsigned rAbove = 0;
    if (r > 0) {
        // cell is not in the first row, so use the above row in its own section
        section = cell->section();
        rAbove = r - 1;
    } else {
        section = sectionAbove(cell->section(), SkipEmptySections::Yes);
        if (section) {
            ASSERT(section->numRows());
            rAbove = section->numRows() - 1;
        }
    }

    // Look up the cell in the section's grid, which requires effective col index
    if (section) {
        unsigned effCol = colToEffCol(cell->col());
        RenderTableSection::CellStruct& aboveCell = section->cellAt(rAbove, effCol);
        return aboveCell.primaryCell();
    }
    return nullptr;
}

RenderTableCell* RenderTable::cellBelow(const RenderTableCell* cell) const
{
    recalcSectionsIfNeeded();

    // Find the section and row to look in
    unsigned r = cell->rowIndex() + cell->rowSpan() - 1;
    RenderTableSection* section = nullptr;
    unsigned rBelow = 0;
    if (r < cell->section()->numRows() - 1) {
        // The cell is not in the last row, so use the next row in the section.
        section = cell->section();
        rBelow = r + 1;
    } else {
        section = sectionBelow(cell->section(), SkipEmptySections::Yes);
        if (section)
            rBelow = 0;
    }

    // Look up the cell in the section's grid, which requires effective col index
    if (section) {
        unsigned effCol = colToEffCol(cell->col());
        RenderTableSection::CellStruct& belowCell = section->cellAt(rBelow, effCol);
        return belowCell.primaryCell();
    }
    return nullptr;
}

RenderTableCell* RenderTable::cellBefore(const RenderTableCell* cell) const
{
    recalcSectionsIfNeeded();

    RenderTableSection* section = cell->section();
    unsigned effCol = colToEffCol(cell->col());
    if (!effCol)
        return nullptr;
    
    // If we hit a colspan back up to a real cell.
    RenderTableSection::CellStruct& prevCell = section->cellAt(cell->rowIndex(), effCol - 1);
    return prevCell.primaryCell();
}

RenderTableCell* RenderTable::cellAfter(const RenderTableCell* cell) const
{
    recalcSectionsIfNeeded();

    unsigned effCol = colToEffCol(cell->col() + cell->colSpan());
    if (effCol >= numEffCols())
        return nullptr;
    return cell->section()->primaryCellAt(cell->rowIndex(), effCol);
}

std::optional<LayoutUnit> RenderTable::firstLineBaseline() const
{
    // The baseline of a 'table' is the same as the 'inline-table' baseline per CSS 3 Flexbox (CSS 2.1
    // doesn't define the baseline of a 'table' only an 'inline-table').
    // This is also needed to properly determine the baseline of a cell if it has a table child.

    if ((isWritingModeRoot() && !isFlexItem()) || shouldApplyLayoutContainment())
        return { };

    recalcSectionsIfNeeded();

    CheckedPtr topNonEmptySection = this->topNonEmptySection();
    if (!topNonEmptySection)
        return { };

    auto baseline = std::optional<LayoutUnit> { };
    if (auto firstLineBaseline = topNonEmptySection->firstLineBaseline())
        baseline = firstLineBaseline;
    else if (topNonEmptySection->firstRow() && !topNonEmptySection->firstRow()->firstCell()) {
        // Other browsers use the top of the section as the baseline if its first row is empty of cells or content.
        // The baseline of an empty row isn't specified by CSS 2.1.
        baseline = 0_lu;
    }
    return baseline ? std::optional(topNonEmptySection->logicalTop() + *baseline) : std::nullopt;
}

std::optional<LayoutUnit> RenderTable::lastLineBaseline() const
{
    if (isWritingModeRoot() || shouldApplyLayoutContainment())
        return { };

    recalcSectionsIfNeeded();

    CheckedPtr tableSection = bottomNonEmptySection();
    if (!tableSection)
        return { };

    if (auto lastLineBaseline = tableSection->lastLineBaseline())
        return tableSection->logicalTop() + *lastLineBaseline;
    return { };
}

LayoutRect RenderTable::overflowClipRect(const LayoutPoint& location, OverlayScrollbarSizeRelevancy relevancy, PaintPhase phase) const
{
    LayoutRect rect;
    // Don't clip out the table's side of the collapsed borders if we're in the paint phase that will ask the sections to paint them.
    // Likewise, if we're self-painting we avoid clipping them out as the clip rect that will be passed down to child layers from RenderLayer will do that instead.
    if (phase == PaintPhase::ChildBlockBackgrounds || layer()->isSelfPaintingLayer()) {
        rect = borderBoxRect();
        rect.setLocation(location + rect.location());
    } else
        rect = RenderBox::overflowClipRect(location, relevancy);

    // If we have a caption, expand the clip to include the caption.
    // FIXME: Technically this is wrong, but it's virtually impossible to fix this
    // for real until captions have been re-written.
    // FIXME: This code assumes (like all our other caption code) that only top/bottom are
    // supported.  When we actually support left/right and stop mapping them to top/bottom,
    // we might have to hack this code first (depending on what order we do these bug fixes in).
    if (!m_captions.isEmpty()) {
        if (writingMode().isHorizontal()) {
            rect.setHeight(borderBoxHeight());
            rect.setY(location.y());
        } else {
            rect.setWidth(borderBoxWidth());
            rect.setX(location.x());
        }
    }

    return rect;
}

bool RenderTable::nodeAtPoint(const HitTestRequest& request, HitTestResult& result, const HitTestLocation& locationInContainer, const LayoutPoint& accumulatedOffset, HitTestAction action)
{
    LayoutPoint adjustedLocation = accumulatedOffset + location();

    // Check kids first.
    if (!hasNonVisibleOverflow() || locationInContainer.intersects(overflowClipRect(adjustedLocation))) {
        for (RenderObject* child = lastChild(); child; child = child->previousSibling()) {
            CheckedPtr box = dynamicDowncast<RenderBox>(*child);
            if (box && !box->hasSelfPaintingLayer() && (box->isRenderTableSection() || box->isRenderTableCaption())) {
                LayoutPoint childPoint = flipForWritingModeForChild(*box, adjustedLocation);
                if (box->nodeAtPoint(request, result, locationInContainer, childPoint, action)) {
                    updateHitTestResult(result, toLayoutPoint(locationInContainer.point() - childPoint));
                    return true;
                }
            }
        }
    }

    // Check our bounds next.
    LayoutRect boundsRect(adjustedLocation, borderBoxSize());
    if (visibleToHitTesting(request) && (action == HitTestAction::BlockBackground || action == HitTestAction::ChildBlockBackground) && locationInContainer.intersects(boundsRect)) {
        updateHitTestResult(result, flipForWritingMode(locationInContainer.point() - toLayoutSize(adjustedLocation)));
        if (result.addNodeToListBasedTestResult(protect(nodeForHitTest()).get(), request, locationInContainer, boundsRect) == HitTestProgress::Stop)
            return true;
    }

    return false;
}

void RenderTable::markForPaginationRelayoutIfNeeded()
{
    auto* layoutState = view().frameView().layoutContext().layoutState();
    if (!layoutState || !layoutState->isPaginated() || (!layoutState->pageLogicalHeightChanged() && (!layoutState->pageLogicalHeight() || layoutState->pageLogicalOffset(this, logicalTop()) == pageLogicalOffset())))
        return;
    
    // When a table moves, we have to dirty all of the sections too.
    setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);
    for (auto& child : childrenOfType<RenderTableSection>(*this)) {
        if (!child.needsLayout())
            child.setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);
    }
}

}
```

## RenderTableSection.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderTableSection.cpp). Path: `Source/WebCore/rendering/RenderTableSection.cpp`. Source bytes: 77513; source lines: 1776; SHA-256: `8f40aa60cad29c0f0b1690d3fd6faaa43ad50f05e8a816045db353527391022a`.

```cpp
/*
 * Copyright (C) 1997 Martin Jones (mjones@kde.org)
 *           (C) 1997 Torben Weis (weis@kde.org)
 *           (C) 1998 Waldo Bastian (bastian@kde.org)
 *           (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 * Copyright (C) 2003-2025 Apple Inc. All rights reserved.
 * Copyright (C) 2014-2018 Google Inc. All rights reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@nypop.com)
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

#include "config.h"
#include "RenderTableSection.h"

#include "BorderPainter.h"
#include "Document.h"
#include "HTMLFieldSetElement.h"
#include "HTMLFormControlElement.h"
#include "HTMLNames.h"
#include "HitTestResult.h"
#include "PaintInfo.h"
#include "PaintInfoInlines.h"
#include "RenderBoxInlines.h"
#include "RenderBoxModelObjectInlines.h"
#include "RenderChildIterator.h"
#include "RenderElementStyleInlines.h"
#include "RenderLayoutState.h"
#include "RenderObjectInlines.h"
#include "RenderTableCellInlines.h"
#include "RenderTableCol.h"
#include "RenderTableRow.h"
#include "RenderTableSectionInlines.h"
#include "RenderTextControl.h"
#include "RenderTreeBuilder.h"
#include "RenderView.h"
#include "StylePrimitiveNumericTypes+Evaluation.h"
#include <limits>
#include <ranges>
#include <wtf/HashSet.h>
#include <wtf/StackStats.h>
#include <wtf/TZoneMallocInlines.h>

namespace WebCore {

using namespace HTMLNames;

WTF_MAKE_TZONE_ALLOCATED_IMPL(RenderTableSection);

// Those 2 variables are used to balance the memory consumption vs the repaint time on big tables.
static const unsigned gMinTableSizeToUseFastPaintPathWithOverflowingCell = 75 * 75;
static const float gMaxAllowedOverflowingCellRatioForFastPaintPath = 0.1f;

static inline void setRowLogicalHeightToRowStyleLogicalHeight(RenderTableSection::RowStruct& row)
{
    ASSERT(row.rowRenderer);
    row.logicalHeight = row.rowRenderer->style().logicalHeight();
}

static inline void updateLogicalHeightForCell(RenderTableSection::RowStruct& row, const RenderTableCell* cell)
{
    // We ignore height settings on rowspan cells.
    if (cell->rowSpan() != 1)
        return;

    auto& logicalHeight = !cell->isOrthogonal() ? cell->style().logicalHeight() : cell->style().logicalWidth();
    if (logicalHeight.isPossiblyPositive()) {
        if (auto percentageLogicalHeight = logicalHeight.tryPercentage()) {
            if (auto percentageRowLogicalHeight = row.logicalHeight.tryPercentage(); !percentageRowLogicalHeight || percentageRowLogicalHeight->value < percentageLogicalHeight->value)
                row.logicalHeight = logicalHeight;
        } else if (auto fixedLogicalHeight = logicalHeight.tryFixed()) {
            if (auto fixedRowLogicalHeight = row.logicalHeight.tryFixed(); row.logicalHeight.isAuto() || (fixedRowLogicalHeight && fixedRowLogicalHeight->resolveZoom(cell->style().usedZoomForLength()) < fixedLogicalHeight->resolveZoom(cell->style().usedZoomForLength())))
                row.logicalHeight = logicalHeight;
        }
    }
}

RenderTableSection::RenderTableSection(Element& element, Style::ComputedStyle&& style)
    : RenderBox(Type::TableSection, element, WTF::move(style))
{
    setInline(false);
    ASSERT(isRenderTableSection());
}

RenderTableSection::RenderTableSection(Document& document, Style::ComputedStyle&& style)
    : RenderBox(Type::TableSection, document, WTF::move(style))
{
    setInline(false);
    ASSERT(isRenderTableSection());
}

RenderTableSection::~RenderTableSection() = default;

ASCIILiteral RenderTableSection::renderName() const
{
    return (isAnonymous() || isPseudoElement()) ? "RenderTableSection (anonymous)"_s : "RenderTableSection"_s;
}

void RenderTableSection::styleDidChange(Style::Difference diff, const Style::ComputedStyle* oldStyle)
{
    RenderBox::styleDidChange(diff, oldStyle);
    propagateStyleToAnonymousChildren(StylePropagationType::AllChildren);

    if (CheckedPtr table = this->table(); table && oldStyle)
        table->invalidateCollapsedBordersAfterStyleChangeIfNeeded(*oldStyle, style());
}

void RenderTableSection::willBeRemovedFromTree()
{
    RenderBox::willBeRemovedFromTree();

    // Preventively invalidate our cells as we may be re-inserted into
    // a new table which would require us to rebuild our structure.
    setNeedsCellRecalc();
}

void RenderTableSection::willInsertTableRow(RenderTableRow& child, RenderObject* beforeChild)
{
    if (beforeChild)
        setNeedsCellRecalc();

    unsigned insertionRow = m_cRow;
    ++m_cRow;
    m_cCol = 0;

    ensureRows(m_cRow);

    m_grid[insertionRow].rowRenderer = &child;
    child.setRowIndex(insertionRow);

    if (!beforeChild)
        setRowLogicalHeightToRowStyleLogicalHeight(m_grid[insertionRow]);
}

void RenderTableSection::ensureRows(unsigned numRows)
{
    if (numRows <= m_grid.size())
        return;

    unsigned oldSize = m_grid.size();
    m_grid.grow(numRows);

    unsigned effectiveColumnCount = std::max(1u, table()->numEffCols());
    for (unsigned row = oldSize; row < m_grid.size(); ++row)
        m_grid[row].row.resizeToFit(effectiveColumnCount);
}

void RenderTableSection::addCell(RenderTableCell* cell, RenderTableRow* row)
{
    // We don't insert the cell if we need cell recalc as our internal columns' representation
    // will have drifted from the table's representation. Also recalcCells will call addCell
    // at a later time after sync'ing our columns' with the table's.
    if (needsCellRecalc())
        return;

    unsigned rSpan = cell->rowSpan();
    unsigned cSpan = cell->colSpan();
    const Vector<RenderTable::ColumnStruct>& columns = table()->columns();
    unsigned nCols = columns.size();
    unsigned insertionRow = row->rowIndex();

    // ### mozilla still seems to do the old HTML way, even for strict DTD
    // (see the annotation on table cell layouting in the CSS specs and the testcase below:
    // <TABLE border>
    // <TR><TD>1 <TD rowspan="2">2 <TD>3 <TD>4
    // <TR><TD colspan="2">5
    // </TABLE>
    while (m_cCol < nCols && (cellAt(insertionRow, m_cCol).hasCells() || cellAt(insertionRow, m_cCol).inColSpan))
        m_cCol++;

    updateLogicalHeightForCell(m_grid[insertionRow], cell);

    ensureRows(insertionRow + rSpan);

    m_grid[insertionRow].rowRenderer = row;

    unsigned col = m_cCol;
    // tell the cell where it is
    bool inColSpan = false;
    while (cSpan) {
        unsigned currentSpan;
        if (m_cCol >= nCols) {
            table()->appendColumn(cSpan);
            currentSpan = cSpan;
        } else {
            if (cSpan < columns[m_cCol].span)
                table()->splitColumn(m_cCol, cSpan);
            currentSpan = columns[m_cCol].span;
        }
        for (unsigned r = 0; r < rSpan; r++) {
            CellStruct& c = cellAt(insertionRow + r, m_cCol);
            ASSERT(cell);
            c.cells.append(cell);
            // If cells overlap then we take the slow path for painting.
            if (c.cells.size() > 1)
                m_hasMultipleCellLevels = true;
            if (inColSpan)
                c.inColSpan = true;
        }
        m_cCol++;
        cSpan -= currentSpan;
        inColSpan = true;
    }
    cell->setCol(table()->effColToCol(col));
}

static LayoutUnit resolveLogicalHeightForRow(const Style::PreferredSize& rowLogicalHeight, Style::ZoomFactor usedZoom)
{
    if (auto fixedRowLogicalHeight = rowLogicalHeight.tryFixed())
        return Style::evaluate<LayoutUnit>(*fixedRowLogicalHeight, usedZoom);
    if (rowLogicalHeight.isCalculated())
        return Style::evaluate<LayoutUnit>(rowLogicalHeight, 0, usedZoom);
    return 0;
}

LayoutUnit RenderTableSection::calcRowLogicalHeight()
{
    SetLayoutNeededForbiddenScope layoutForbiddenScope(*this);

    ASSERT(!needsLayout());

    RenderTableCell* cell;

    // We ignore the border-spacing on any non-top section as it is already included in the previous section's last row position.
    LayoutUnit spacing;
    if (this == table()->topSection())
        spacing = table()->vBorderSpacing();

    LayoutStateMaintainer statePusher(*this, locationOffset(), isTransformed() || hasReflection() || writingMode().isBlockFlipped());

    m_rowPos.resize(m_grid.size() + 1);
    m_rowPos[0] = spacing;

    unsigned totalRows = m_grid.size();

    for (unsigned r = 0; r < totalRows; r++) {
        m_grid[r].baseline = 0;
        LayoutUnit baselineDescent;

        if (m_grid[r].logicalHeight.isSpecified()) {
        // Our base size is the biggest logical height from our cells' styles (excluding row spanning cells).
            m_rowPos[r + 1] = std::max(m_rowPos[r] + resolveLogicalHeightForRow(m_grid[r].logicalHeight, m_grid[r].rowRenderer->style().usedZoomForLength()), 0_lu);
        } else {
        // Non-specified lengths are ignored because the row already accounts for the cells intrinsic logical height.
            m_rowPos[r + 1] = std::max(m_rowPos[r], 0_lu);
        }

        Row& row = m_grid[r].row;
        unsigned totalCols = row.size();

        for (unsigned c = 0; c < totalCols; c++) {
            CellStruct& current = cellAt(r, c);
            for (unsigned i = 0; i < current.cells.size(); i++) {
                cell = current.cells[i];
                if (current.inColSpan && cell->rowSpan() == 1)
                    continue;

                // FIXME: We are always adding the height of a rowspan to the last rows which doesn't match
                // other browsers. See webkit.org/b/52185 for example.
                if ((cell->rowIndex() + cell->rowSpan() - 1) != r) {
                    // We will apply the height of the rowspan to the current row if next row is not valid.
                    if ((r + 1) < totalRows) {
                        unsigned col = 0;
                        CellStruct nextRowCell = cellAt(r + 1, col);

                        // We are trying to find that next row is valid or not.
                        while (nextRowCell.cells.size() && nextRowCell.cells[0]->rowSpan() > 1 && nextRowCell.cells[0]->rowIndex() < (r + 1)) {
                            col++;
                            if (col < totalCols)
                                nextRowCell = cellAt(r + 1, col);
                            else
                                break;
                        }

                        // We are adding the height of the rowspan to the current row if next row is not valid.
                        if (col < totalCols && nextRowCell.cells.size())
                            continue;
                    }
                }

                // For row spanning cells, |r| is the last row in the span.
                unsigned cellStartRow = cell->rowIndex();

                if (cell->overridingBorderBoxLogicalHeight() && !cell->isOrthogonal()) {
                    cell->clearIntrinsicPadding();
                    cell->clearOverridingSize();
                    cell->setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);
                    cell->layoutIfNeeded();
                }

                LayoutUnit cellLogicalHeight = cell->logicalHeightForRowSizing();
                m_rowPos[r + 1] = std::max(m_rowPos[r + 1], m_rowPos[cellStartRow] + cellLogicalHeight);

                // Find out the baseline. The baseline is set on the first row in a rowspan.
                if (cell->isBaselineAligned()) {
                    LayoutUnit baselinePosition = cell->cellBaselinePosition() - cell->intrinsicPaddingBefore();
                    LayoutUnit borderAndComputedPaddingBefore = cell->borderAndPaddingBefore() - cell->intrinsicPaddingBefore();
                    if (baselinePosition > borderAndComputedPaddingBefore) {
                        m_grid[cellStartRow].baseline = std::max(m_grid[cellStartRow].baseline, baselinePosition);
                        // The descent of a cell that spans multiple rows does not affect the height of the first row it spans, so don't let it
                        // become the baseline descent applied to the rest of the row. Also we don't account for the baseline descent of
                        // non-spanning cells when computing a spanning cell's extent.
                        LayoutUnit cellStartRowBaselineDescent;
                        if (cell->rowSpan() == 1) {
                            baselineDescent = std::max(baselineDescent, cellLogicalHeight - baselinePosition);
                            cellStartRowBaselineDescent = baselineDescent;
                        }
                        m_rowPos[cellStartRow + 1] = std::max(m_rowPos[cellStartRow + 1], m_rowPos[cellStartRow] + m_grid[cellStartRow].baseline + cellStartRowBaselineDescent);
                    }
                }
            }
        }

        // Add the border-spacing to our final position.
        // Use table border-spacing even in non-top sections
        spacing = table()->vBorderSpacing();
        m_rowPos[r + 1] += m_grid[r].rowRenderer ? spacing : 0_lu;
        m_rowPos[r + 1] = std::max(m_rowPos[r + 1], m_rowPos[r]);
    }

    for (size_t rowIndex = 0; rowIndex < totalRows; ++rowIndex) {
        if (m_grid[rowIndex].rowRenderer && m_grid[rowIndex].rowRenderer->style().visibility() == Visibility::Collapse) {
            auto delta = m_rowPos[rowIndex + 1] - m_rowPos[rowIndex];
            if (delta > 0_lu) {
                // Reduce height of collapsed row to 0 without affecting other rows
                for (size_t adjustedRowIndex = rowIndex + 1; adjustedRowIndex <= totalRows; ++adjustedRowIndex)
                    m_rowPos[adjustedRowIndex] -= delta;
            }
        }
    }

    ASSERT(!needsLayout());
    return m_rowPos[m_grid.size()];
}

LayoutUnit RenderTableSection::cellLogicalWidthInTableDirectionIncludingColumnSpan(const RenderTableCell& cell, size_t startColumn, size_t numberOfColumns) const
{
    ASSERT(startColumn < numberOfColumns);

    auto endColumn = startColumn;
    auto columnSpan = cell.colSpan();
    auto& columns = table()->columns();
    while (columnSpan && endColumn < numberOfColumns) {
        ASSERT(endColumn < columns.size());
        columnSpan -= columns[endColumn].span;
        endColumn++;
    }
    auto& columnPositions = table()->columnPositions();
    if (startColumn >= columnPositions.size() || endColumn >= columnPositions.size()) {
        ASSERT_NOT_REACHED();
        return { };
    }
    return columnPositions[endColumn] - columnPositions[startColumn] - table()->hBorderSpacing();
}

void RenderTableSection::layout()
{
    StackStats::LayoutCheckPoint layoutCheckPoint;
    ASSERT(needsLayout());
    ASSERT(!needsCellRecalc());
    ASSERT(!table()->needsSectionRecalc());

    m_forceSlowPaintPathWithOverflowingCell = false;
    // addChild may over-grow m_grid but we don't want to throw away the memory too early as addChild
    // can be called in a loop (e.g during parsing). Doing it now ensures we have a stable-enough structure.
    m_grid.shrinkToFit();

    LayoutStateMaintainer statePusher(*this, locationOffset(), isTransformed() || hasReflection() || writingMode().isBlockFlipped());
    bool paginated = view().frameView().layoutContext().layoutState()->isPaginated();

    for (size_t rowIndex = 0; rowIndex < m_grid.size(); ++rowIndex) {
        auto& columnList = m_grid[rowIndex].row;
        auto numberOfColumns = columnList.size();
        CheckedPtr rowRenderer = m_grid[rowIndex].rowRenderer;
        // First, propagate our table layout's information to the cells. This will mark the row as needing layout
        // if there was a column logical width change.
        for (size_t startColumn = 0; startColumn < numberOfColumns; ++startColumn) {
            auto& currentColumn = columnList[startColumn];
            auto* cell = currentColumn.primaryCell();
            if (!cell || currentColumn.inColSpan)
                continue;

            auto cellHadSelfNeedsLayout = cell->selfNeedsLayout();
            cell->setCellLogicalWidth(cellLogicalWidthInTableDirectionIncludingColumnSpan(*cell, startColumn, numberOfColumns));
            if (!cellHadSelfNeedsLayout && cell->selfNeedsLayout() && rowRenderer)
                rowRenderer->setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);
        }

        if (rowRenderer) {
            if (!rowRenderer->needsLayout() && paginated && view().frameView().layoutContext().layoutState()->pageLogicalHeightChanged())
                rowRenderer->setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);

            rowRenderer->layoutIfNeeded();
        }
    }
    clearNeedsLayout();
}

void RenderTableSection::distributeExtraLogicalHeightToPercentRows(LayoutUnit& extraLogicalHeight, int totalPercent)
{
    if (!totalPercent)
        return;

    unsigned totalRows = m_grid.size();
    LayoutUnit totalHeight = m_rowPos[totalRows] + extraLogicalHeight;
    LayoutUnit totalLogicalHeightAdded;
    totalPercent = std::min(totalPercent, 100);
    auto rowHeight = rowLogicalHeight(0);
    for (unsigned r = 0; r < totalRows; ++r) {
        if (auto percentageLogicalHeight = m_grid[r].logicalHeight.tryPercentage(); totalPercent > 0 && percentageLogicalHeight) {
            LayoutUnit toAdd = std::min(extraLogicalHeight, LayoutUnit((totalHeight * percentageLogicalHeight->value / 100) - rowHeight));
            // If toAdd is negative, then we don't want to shrink the row (this bug
            // affected Outlook Web Access).
            toAdd = std::max(0_lu, toAdd);
            totalLogicalHeightAdded += toAdd;
            extraLogicalHeight -= toAdd;
            totalPercent -= percentageLogicalHeight->value;
        }
        ASSERT(totalRows >= 1);
        if (r < totalRows - 1)
            rowHeight = rowLogicalHeight(r + 1);
        m_rowPos[r + 1] += totalLogicalHeightAdded;
    }
}

void RenderTableSection::distributeExtraLogicalHeightToAutoRows(LayoutUnit& extraLogicalHeight, unsigned autoRowsCount)
{
    if (!autoRowsCount)
        return;

    LayoutUnit totalLogicalHeightAdded;
    for (unsigned r = 0; r < m_grid.size(); ++r) {
        if (autoRowsCount > 0 && m_grid[r].logicalHeight.isAuto() && m_grid[r].rowRenderer) {
            // Recomputing |extraLogicalHeightForRow| guarantees that we properly ditribute round |extraLogicalHeight|.
            LayoutUnit extraLogicalHeightForRow = extraLogicalHeight / autoRowsCount;
            totalLogicalHeightAdded += extraLogicalHeightForRow;
            extraLogicalHeight -= extraLogicalHeightForRow;
            --autoRowsCount;
        }
        m_rowPos[r + 1] += totalLogicalHeightAdded;
    }
}

LayoutUnit RenderTableSection::rowLogicalHeight(unsigned row) const
{
    ASSERT(!needsLayout());
    if (row >= m_grid.size()) {
        ASSERT_NOT_REACHED();
        return { };
    }
    return m_rowPos[row + 1] - m_rowPos[row] - table()->vBorderSpacing();
}

void RenderTableSection::distributeRemainingExtraLogicalHeight(LayoutUnit& extraLogicalHeight)
{
    unsigned totalRows = m_grid.size();

    if (extraLogicalHeight <= 0 || !m_rowPos[totalRows])
        return;

    // Pre-compute row heights before distribution since the loop modifies m_rowPos.
    Vector<LayoutUnit, 8> rowHeights(totalRows);
    LayoutUnit totalRowSize;
    for (unsigned r = 0; r < totalRows; ++r) {
        rowHeights[r] = rowLogicalHeight(r);
        totalRowSize += rowHeights[r];
    }

    if (totalRowSize <= 0) {
        // All rows are zero-height — distribute equally.
        LayoutUnit totalLogicalHeightAdded;
        for (unsigned r = 0; r < totalRows; ++r) {
            totalLogicalHeightAdded += extraLogicalHeight / totalRows;
            m_rowPos[r + 1] += totalLogicalHeightAdded;
        }
        extraLogicalHeight -= totalLogicalHeightAdded;
        return;
    }

    LayoutUnit totalLogicalHeightAdded;
    for (unsigned r = 0; r < totalRows; ++r) {
        totalLogicalHeightAdded += extraLogicalHeight * rowHeights[r] / totalRowSize;
        m_rowPos[r + 1] += totalLogicalHeightAdded;
    }

    extraLogicalHeight -= totalLogicalHeightAdded;
}

LayoutUnit RenderTableSection::distributeExtraLogicalHeightToRows(LayoutUnit extraLogicalHeight)
{
    if (!extraLogicalHeight)
        return extraLogicalHeight;

    unsigned totalRows = m_grid.size();
    if (!totalRows)
        return extraLogicalHeight;

    unsigned autoRowsCount = 0;
    int totalPercent = 0;
    for (unsigned r = 0; r < totalRows; r++) {
        if (m_grid[r].logicalHeight.isAuto() && m_grid[r].rowRenderer)
            ++autoRowsCount;
        else if (auto percentageLogicalHeight = m_grid[r].logicalHeight.tryPercentage())
            totalPercent += percentageLogicalHeight->value;
    }

    // If this section has no intrinsic height and there are other sections,
    // distribute based on whether we have percentage/auto rows that can grow.
    if (!m_rowPos[totalRows] && nextSibling()) {
        if (!autoRowsCount && !totalPercent)
            return extraLogicalHeight;
    }

    LayoutUnit remainingExtraLogicalHeight = extraLogicalHeight;
    distributeExtraLogicalHeightToPercentRows(remainingExtraLogicalHeight, totalPercent);
    distributeExtraLogicalHeightToAutoRows(remainingExtraLogicalHeight, autoRowsCount);
    distributeRemainingExtraLogicalHeight(remainingExtraLogicalHeight);
    return extraLogicalHeight - remainingExtraLogicalHeight;
}

static bool NODELETE shouldFlexCellChild(const RenderTableCell& cell, const RenderBox& cellDescendant)
{
    if (!cell.style().logicalHeight().isSpecified())
        return false;
    if (cellDescendant.scrollsOverflowY())
        return true;
    if (cellDescendant.isBlockLevelReplacedOrAtomicInline())
        return true;
    return is<HTMLFormControlElement>(cellDescendant.element()) && !is<HTMLFieldSetElement>(cellDescendant.element());
}

void RenderTableSection::relayoutCellIfFlexed(RenderTableCell& cell, int rowIndex, int rowHeight)
{
    // Force percent height children to lay themselves out again.
    // This will cause these children to grow to fill the cell.
    // FIXME: There is still more work to do here to fully match WinIE (should
    // it become necessary to do so). In quirks mode, WinIE behaves like we
    // do, but it will clip the cells that spill out of the table section. In
    // strict mode, Mozilla and WinIE both regrow the table to accommodate the
    // new height of the cell (thus letting the percentages cause growth one
    // time only). We may also not be handling row-spanning cells correctly.
    //
    // Note also the oddity where replaced elements always flex, and yet blocks/tables do
    // not necessarily flex. WinIE is crazy and inconsistent, and we can't hope to
    // match the behavior perfectly, but we'll continue to refine it as we discover new
    // bugs. :)
    bool cellChildrenFlex = false;
    bool flexAllChildren = cell.style().logicalHeight().isFixed() || (!table()->style().logicalHeight().isAuto() && rowHeight != cell.logicalHeight());
    
    for (auto& renderer : childrenOfType<RenderBox>(cell)) {
        if (renderer.style().logicalHeight().isPercentOrCalculated() && (flexAllChildren || shouldFlexCellChild(cell, renderer))) {
            auto* renderTable = dynamicDowncast<RenderTable>(renderer);
            if (!renderTable || renderTable->hasSections()) {
                cellChildrenFlex = true;
                break;
            }
        }
    }

    if (!cellChildrenFlex) {
        if (TrackedRendererListHashSet* percentHeightDescendants = cell.percentHeightDescendants()) {
            for (auto& descendant : *percentHeightDescendants) {
                if (flexAllChildren || shouldFlexCellChild(cell, descendant)) {
                    cellChildrenFlex = true;
                    break;
                }
            }
        }
    }
    
    if (!cellChildrenFlex)
        return;

    cell.setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);
    // Alignment within a cell is based off the calculated
    // height, which becomes irrelevant once the cell has
    // been resized based off its percentage.
    cell.setOverridingLogicalHeightFromRowHeight(rowHeight);
    cell.layoutIfNeeded();
    
    if (!cell.isBaselineAligned())
        return;
    
    // If the baseline moved, we may have to update the data for our row. Find out the new baseline.
    LayoutUnit baseline = cell.cellBaselinePosition();
    if (baseline > cell.borderAndPaddingBefore())
        m_grid[rowIndex].baseline = std::max(m_grid[rowIndex].baseline, baseline);
}

void RenderTableSection::layoutRows()
{
    SetLayoutNeededForbiddenScope layoutForbiddenScope(*this);

    ASSERT(!needsLayout());

    auto numberOfRows = m_grid.size();

    // Set the width of our section now.  The rows will also be this width.
    setLogicalWidth(table()->contentBoxLogicalWidth());
    m_forceSlowPaintPathWithOverflowingCell = false;

    LayoutUnit vspacing = table()->vBorderSpacing();
    size_t numberOfEffectiveColumns = table()->numEffCols();

    LayoutStateMaintainer statePusher(*this, locationOffset(), isTransformed() || writingMode().isBlockFlipped());

    for (size_t rowIndex = 0; rowIndex < numberOfRows; rowIndex++) {

        auto layoutCells = [&] {
            LayoutUnit rowHeightIncreaseForPagination;

            for (size_t columnIndex = 0; columnIndex < numberOfEffectiveColumns; columnIndex++) {
                CellStruct& cs = cellAt(rowIndex, columnIndex);
                RenderTableCell* cell = cs.primaryCell();

                if (!cell || cs.inColSpan)
                    continue;

                int rowIndex = cell->rowIndex();
                auto rowHeight = m_rowPos[rowIndex + cell->rowSpan()] - m_rowPos[rowIndex] - vspacing;

                relayoutCellIfFlexed(*cell, rowIndex, rowHeight);

                auto logicalHeightForIntrinsicPadding = !cell->isOrthogonal() ? rowHeight : cellLogicalWidthInTableDirectionIncludingColumnSpan(*cell, columnIndex, numberOfEffectiveColumns);
                if (cell->computeIntrinsicPadding(logicalHeightForIntrinsicPadding)) {
                    // FIXME: Changing an intrinsic padding shouldn't trigger a relayout as it only shifts the cell inside the row but doesn't change the logical height.
                    cell->setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);
                }

                LayoutRect oldCellRect = cell->borderBoxRectInContainer();

                setLogicalPositionForCell(cell, columnIndex);

                auto* layoutState = view().frameView().layoutContext().layoutState();
                if (!cell->needsLayout() && layoutState->pageLogicalHeight() && layoutState->pageLogicalOffset(cell, cell->logicalTop()) != cell->pageLogicalOffset())
                    cell->setChildNeedsLayout(MarkingBehavior::MarkOnlyThis);

                if (cell->isOrthogonal()) {
                    cell->setNeedsLayout(MarkingBehavior::MarkOnlyThis);
                    cell->setOverridingBorderBoxLogicalWidth(rowHeight);
                }
                cell->layoutIfNeeded();

                // FIXME: Make pagination work with vertical tables.
                if (layoutState->pageLogicalHeight() && cell->logicalHeight() != rowHeight) {
                    // FIXME: Pagination might have made us change size. For now just shrink or grow the cell to fit without doing a relayout.
                    // We'll also do a basic increase of the row height to accommodate the cell if it's bigger, but this isn't quite right
                    // either. It's at least stable though and won't result in an infinite # of relayouts that may never stabilize.
                    if (cell->logicalHeight() > rowHeight)
                        rowHeightIncreaseForPagination = std::max(rowHeightIncreaseForPagination, cell->logicalHeight() - rowHeight);
                    cell->setLogicalHeight(rowHeight);
                }

                LayoutSize childOffset(cell->location() - oldCellRect.location());
                if (childOffset.width() || childOffset.height()) {
                    view().frameView().layoutContext().addLayoutDelta(childOffset);

                    // If the child moved, we have to repaint it as well as any floating/positioned
                    // descendants. An exception is if we need a layout. In this case, we know we're going to
                    // repaint ourselves (and the child) anyway.
                    if (!table()->selfNeedsLayout() && cell->checkForRepaintDuringLayout())
                        cell->repaintDuringLayoutIfMoved(oldCellRect);
                }
            }
            if (rowHeightIncreaseForPagination) {
                for (size_t index = rowIndex + 1; index <= numberOfRows; ++index)
                    m_rowPos[index] += rowHeightIncreaseForPagination;
                for (size_t index = 0; index < numberOfEffectiveColumns; ++index) {
                    Vector<RenderTableCell*, 1>& cells = cellAt(rowIndex, index).cells;
                    for (size_t cellIndex = 0; cellIndex < cells.size(); ++cellIndex)
                        cells[cellIndex]->setLogicalHeight(cells[cellIndex]->logicalHeight() + rowHeightIncreaseForPagination);
                }
            }
        };

        if (CheckedPtr rowRenderer = m_grid[rowIndex].rowRenderer) {
            // FIXME: the x() position of the row should be table()->hBorderSpacing() so that it can
            // report the correct offsetLeft. However, that will require a lot of rebaselining of test results.
            auto oldRowRect = rowRenderer->borderBoxRectInContainer();
            rowRenderer->setLogicalLocation({ 0_lu, m_rowPos[rowIndex] });
            rowRenderer->setLogicalWidth(logicalWidth());

            LayoutUnit rowLogicalHeight;
            auto rowHasVisibilityCollapse = (m_grid[rowIndex].rowRenderer && m_grid[rowIndex].rowRenderer->style().visibility() == Visibility::Collapse) || style().visibility() == Visibility::Collapse;
            if (!rowHasVisibilityCollapse)
                rowLogicalHeight = m_rowPos[rowIndex + 1] - m_rowPos[rowIndex] - vspacing;

            ASSERT(rowLogicalHeight >= 0);
            rowRenderer->setLogicalHeight(rowLogicalHeight);
            rowRenderer->updateLayerTransform();

            if (rowRenderer->borderBoxRectInContainer() != oldRowRect && !table()->selfNeedsLayout() && rowRenderer->checkForRepaintDuringLayout())
                rowRenderer->repaintDuringLayoutIfMoved(oldRowRect);

            // Push the row's offset onto the layout state so that pagination offsets
            // are consistent with what RenderTableRow::layout() pushes.
            LayoutStateMaintainer rowStatePusher(*rowRenderer, rowRenderer->locationOffset());
            layoutCells();

            rowRenderer->clearOverflow();
            rowRenderer->addVisualEffectOverflow();
            rowRenderer->addOverflowFromInFlowChildren();
            rowRenderer->layoutOutOfFlowBoxes(RelayoutChildren::Yes);
            rowRenderer->clearNeedsLayout();
        } else
            layoutCells();
    }

    ASSERT(!needsLayout());

    // Distribute any extra height from an explicit section height to the rows before
    // committing the final logical height.
    auto distributeExplicitSectionHeightToRows = [&] {
        auto fixedHeight = style().logicalHeight().tryFixed();
        if (!fixedHeight)
            return;
        LayoutUnit specifiedHeight = Style::evaluate<LayoutUnit>(*fixedHeight, style().usedZoomForLength());
        if (specifiedHeight <= m_rowPos[numberOfRows])
            return;
        distributeExtraLogicalHeightToRows(specifiedHeight - m_rowPos[numberOfRows]);
    };
    distributeExplicitSectionHeightToRows();

    setLogicalHeight(m_rowPos[numberOfRows]);

    updateLayerTransform();

    computeOverflowFromCells(numberOfRows, numberOfEffectiveColumns);
}

bool RenderTableSection::hasOverflowingCell() const
{
    return m_overflowingCells.computeSize() || m_forceSlowPaintPathWithOverflowingCell;
}

void RenderTableSection::computeOverflowFromCells()
{
    unsigned totalRows = m_grid.size();
    unsigned nEffCols = table()->numEffCols();
    computeOverflowFromCells(totalRows, nEffCols);
}

void RenderTableSection::computeOverflowFromCells(unsigned totalRows, unsigned nEffCols)
{
    clearOverflow();
    m_overflowingCells.clear();
    unsigned totalCellsCount = nEffCols * totalRows;
    unsigned maxAllowedOverflowingCellsCount = totalCellsCount < gMinTableSizeToUseFastPaintPathWithOverflowingCell ? 0 : gMaxAllowedOverflowingCellRatioForFastPaintPath * totalCellsCount;

#if ASSERT_ENABLED
    bool hasOverflowingCell = false;
#endif
    // Now that our height has been determined, add in overflow from rows (which
    // already include cell overflow from addOverflowFromInFlowChildren in layoutRows).
    for (unsigned r = 0; r < totalRows; r++) {
        if (CheckedPtr row = m_grid[r].rowRenderer)
            addOverflowFromContainedBox(*row);
        for (unsigned c = 0; c < nEffCols; c++) {
            CellStruct& cs = cellAt(r, c);
            RenderTableCell* cell = cs.primaryCell();
            if (!cell || cs.inColSpan)
                continue;
            if (r < totalRows - 1 && cell == primaryCellAt(r + 1, c))
                continue;
#if ASSERT_ENABLED
            hasOverflowingCell |= cell->hasVisualOverflow();
#endif
            if (cell->hasVisualOverflow() && !m_forceSlowPaintPathWithOverflowingCell) {
                m_overflowingCells.add(*cell);
                if (m_overflowingCells.computeSize() > maxAllowedOverflowingCellsCount) {
                    // We need to set m_forcesSlowPaintPath only if there is a least one overflowing cells as the hit testing code rely on this information.
                    m_forceSlowPaintPathWithOverflowingCell = true;
                    // The slow path does not make any use of the overflowing cells info, don't hold on to the memory.
                    m_overflowingCells.clear();
                }
            }
        }
    }
    ASSERT(hasOverflowingCell == this->hasOverflowingCell());
}

LayoutUnit RenderTableSection::calcBlockDirectionOuterBorder(BlockBorderSide side) const
{
    unsigned totalCols = table()->numEffCols();
    if (!m_grid.size() || !totalCols)
        return 0;

    float borderWidth = 0;

    auto writingMode = table()->writingMode();
    const BorderValue& sectionBorder = (side == BlockBorderSide::BorderBefore) ? style().borderBefore(writingMode) : style().borderAfter(writingMode);
    if (sectionBorder.hasHiddenStyle())
        return -1;

    auto deviceScaleFactor = style().deviceScaleFactor();
    if (sectionBorder.hasVisibleStyle())
        borderWidth = Style::evaluate<float>(sectionBorder.width, style().usedZoomForLength(), deviceScaleFactor);

    const RenderTableRow* row = (side == BlockBorderSide::BorderBefore) ? firstRow() : lastRow();
    auto& rowStyle = row->style();
    const BorderValue& rowBorder = (side == BlockBorderSide::BorderBefore) ? rowStyle.borderBefore(writingMode) : rowStyle.borderAfter(writingMode);

    if (rowBorder.hasHiddenStyle())
        return -1;

    if (rowBorder.hasVisibleStyle()) {
        float rowBorderWidth = Style::evaluate<float>(rowBorder.width, rowStyle.usedZoomForLength(), deviceScaleFactor);
        if (rowBorderWidth > borderWidth)
            borderWidth = rowBorderWidth;
    }

    bool allHidden = true;
    unsigned rowIndex = (side == BlockBorderSide::BorderBefore) ? 0 : (m_grid.size() - 1);
    for (unsigned c = 0; c < totalCols; c++) {
        const CellStruct& current = cellAt(rowIndex, c);
        if (current.inColSpan || !current.hasCells())
            continue;

        // FIXME: Make this work with perpendicular and flipped cells.
        auto& cellBorderStyle = current.primaryCell()->style();
        const BorderValue& cellBorder = (side == BlockBorderSide::BorderBefore) ? cellBorderStyle.borderBefore(writingMode) : cellBorderStyle.borderAfter(writingMode);

        // FIXME: Don't repeat for the same col group
        RenderTableCol* colGroup = table()->colElement(c);
        if (colGroup) {
            auto& colGroupStyle = colGroup->style();
            const BorderValue& colBorder = (side == BlockBorderSide::BorderBefore) ? colGroupStyle.borderBefore(writingMode) : colGroupStyle.borderAfter(writingMode);
            if (colBorder.hasHiddenStyle() || cellBorder.hasHiddenStyle())
                continue;

            allHidden = false;

            if (colBorder.hasVisibleStyle()) {
                float colBorderWidth = Style::evaluate<float>(colBorder.width, colGroupStyle.usedZoomForLength(), deviceScaleFactor);
                if (colBorderWidth > borderWidth)
                    borderWidth = colBorderWidth;
            }
            if (cellBorder.hasVisibleStyle()) {
                float cellBorderWidth = Style::evaluate<float>(cellBorder.width, cellBorderStyle.usedZoomForLength(), deviceScaleFactor);
                if (cellBorderWidth > borderWidth)
                    borderWidth = cellBorderWidth;
            }
        } else {
            if (cellBorder.hasHiddenStyle())
                continue;
            allHidden = false;

            if (cellBorder.hasVisibleStyle()) {
                float cellBorderWidth = Style::evaluate<float>(cellBorder.width, cellBorderStyle.usedZoomForLength(), deviceScaleFactor);
                if (cellBorderWidth > borderWidth)
                    borderWidth = cellBorderWidth;
            }
        }
    }

    if (allHidden)
        return -1;
    return CollapsedBorderValue::adjustedCollapsedBorderWidth(borderWidth, deviceScaleFactor, (side == BlockBorderSide::BorderAfter));
}

LayoutUnit RenderTableSection::calcInlineDirectionOuterBorder(InlineBorderSide side) const
{
    unsigned totalCols = table()->numEffCols();
    if (!m_grid.size() || !totalCols)
        return 0;

    float borderWidth = 0;
    auto writingMode = table()->writingMode();

    const BorderValue& sectionBorder = (side == InlineBorderSide::BorderStart) ? style().borderStart(writingMode) : style().borderEnd(writingMode);
    if (sectionBorder.hasHiddenStyle())
        return -1;

    auto deviceScaleFactor = style().deviceScaleFactor();
    if (sectionBorder.hasVisibleStyle())
        borderWidth = Style::evaluate<float>(sectionBorder.width, style().usedZoomForLength(), deviceScaleFactor);

    unsigned colIndex = (side == InlineBorderSide::BorderStart) ? 0 : (totalCols - 1);
    if (RenderTableCol* colGroup = table()->colElement(colIndex)) {
        auto& colGroupStyle = colGroup->style();
        const BorderValue& colBorder = (side == InlineBorderSide::BorderStart) ? colGroupStyle.borderStart(writingMode) : colGroupStyle.borderEnd(writingMode);

        if (colBorder.hasHiddenStyle())
            return -1;

        if (colBorder.hasVisibleStyle()) {
            float colBorderWidth = Style::evaluate<float>(colBorder.width, colGroupStyle.usedZoomForLength(), deviceScaleFactor);
            if (colBorderWidth > borderWidth)
                borderWidth = colBorderWidth;
        }
    }

    bool allHidden = true;
    for (unsigned r = 0; r < m_grid.size(); r++) {
        const CellStruct& current = cellAt(r, colIndex);
        if (!current.hasCells())
            continue;

        // FIXME: Make this work with perpendicular and flipped cells.
        auto& cellBorderStyle = current.primaryCell()->style();
        const BorderValue& cellBorder = (side == InlineBorderSide::BorderStart) ? cellBorderStyle.borderStart(writingMode) : cellBorderStyle.borderEnd(writingMode);
        auto& rowBorderStyle = current.primaryCell()->parent()->style();
        const BorderValue& rowBorder = (side == InlineBorderSide::BorderStart) ? rowBorderStyle.borderStart(writingMode) : rowBorderStyle.borderEnd(writingMode);
        // FIXME: Don't repeat for the same cell
        if (cellBorder.hasHiddenStyle() || rowBorder.hasHiddenStyle())
            continue;

        allHidden = false;

        if (cellBorder.hasVisibleStyle()) {
            float cellBorderWidth = Style::evaluate<float>(cellBorder.width, cellBorderStyle.usedZoomForLength(), deviceScaleFactor);
            if (cellBorderWidth > borderWidth)
                borderWidth = cellBorderWidth;
        }

        if (rowBorder.hasVisibleStyle()) {
            float rowBorderWidth = Style::evaluate<float>(rowBorder.width, rowBorderStyle.usedZoomForLength(), deviceScaleFactor);
            if (rowBorderWidth > borderWidth)
                borderWidth = rowBorderWidth;
        }
    }

    if (allHidden)
        return -1;
    return CollapsedBorderValue::adjustedCollapsedBorderWidth(borderWidth, deviceScaleFactor, (side == InlineBorderSide::BorderStart) ? writingMode.isInlineFlipped() : !writingMode.isInlineFlipped());
}

void RenderTableSection::recalcOuterBorder()
{
    m_outerBorderBefore = calcBlockDirectionOuterBorder(BlockBorderSide::BorderBefore);
    m_outerBorderAfter = calcBlockDirectionOuterBorder(BlockBorderSide::BorderAfter);
    m_outerBorderStart = calcInlineDirectionOuterBorder(InlineBorderSide::BorderStart);
    m_outerBorderEnd  = calcInlineDirectionOuterBorder(InlineBorderSide::BorderEnd);
}

std::optional<LayoutUnit> RenderTableSection::firstLineBaseline() const
{
    if (!m_grid.size() || m_rowPos.size() != m_grid.size() + 1)
        return { };

    if (auto firstLineBaseline = m_grid.first().baseline)
        return m_rowPos.first() + firstLineBaseline;

    return baselineFromCellContentEdges(ItemPosition::Baseline);
}

std::optional<LayoutUnit> RenderTableSection::lastLineBaseline() const
{
    if (!m_grid.size() || m_rowPos.size() != m_grid.size() + 1)
        return  { };

    if (auto lastLineBaseline = m_grid.last().baseline)
        return m_rowPos[m_grid.size() - 1] + lastLineBaseline;

    return baselineFromCellContentEdges(ItemPosition::LastBaseline);
}

std::optional<LayoutUnit> RenderTableSection::baselineFromCellContentEdges(ItemPosition alignment) const
{
    ASSERT(alignment == ItemPosition::Baseline || alignment == ItemPosition::LastBaseline);
    auto row = alignment == ItemPosition::Baseline ? m_grid[0].row : m_grid[m_grid.size() - 1].row;
    
    std::optional<LayoutUnit> result;
    for (size_t i = 0; i < row.size(); ++i) {
        const CellStruct& cs = row.at(i);
        const RenderTableCell* cell = cs.primaryCell();
        // Only cells with content have a baseline
        if (cell && cell->contentBoxLogicalHeight()) {
            LayoutUnit candidate = m_rowPos[cell->rowIndex()] + cell->borderAndPaddingBefore() + cell->contentBoxLogicalHeight();
            result = std::max(result.value_or(candidate), candidate);
        }
    }
    return result;
}

void RenderTableSection::paint(PaintInfo& paintInfo, const LayoutPoint& paintOffset)
{
    ASSERT(!needsLayout());
    // avoid crashing on bugs that cause us to paint with dirty layout
    if (needsLayout())
        return;
    
    unsigned totalRows = m_grid.size();
    unsigned totalCols = table()->columns().size();

    if (!totalRows || !totalCols)
        return;

    LayoutPoint adjustedPaintOffset = paintOffset + location();

    PaintPhase phase = paintInfo.phase;
    bool pushedClip = pushContentsClip(paintInfo, adjustedPaintOffset);
    paintObject(paintInfo, adjustedPaintOffset);
    if (pushedClip)
        popContentsClip(paintInfo, phase, adjustedPaintOffset);

    if ((phase == PaintPhase::Outline || phase == PaintPhase::SelfOutline) && style().usedVisibility() == Visibility::Visible)
        paintOutline(paintInfo, LayoutRect(adjustedPaintOffset, borderBoxSize()));
}

static inline bool NODELETE compareCellPositions(const SingleThreadWeakPtr<RenderTableCell>& elem1, const SingleThreadWeakPtr<RenderTableCell>& elem2)
{
    return elem1->rowIndex() < elem2->rowIndex();
}

// This comparison is used only when we have overflowing cells as we have an unsorted array to sort. We thus need
// to sort both on rows and columns to properly repaint.
static inline bool NODELETE compareCellPositionsWithOverflowingCells(const SingleThreadWeakPtr<RenderTableCell>& elem1, const SingleThreadWeakPtr<RenderTableCell>& elem2)
{
    if (elem1->rowIndex() != elem2->rowIndex())
        return elem1->rowIndex() < elem2->rowIndex();

    return elem1->col() < elem2->col();
}

void RenderTableSection::paintCell(RenderTableCell* cell, PaintInfo& paintInfo, const LayoutPoint& paintOffset)
{
    auto& row = downcast<RenderTableRow>(*cell->parent());
    auto rowBackgroundPaintOffset = flipForWritingModeForChild(row, paintOffset);
    auto rowPaintOffset = rowBackgroundPaintOffset + row.location();
    auto cellPoint = row.flipForWritingModeForChild(*cell, rowPaintOffset);
    PaintPhase paintPhase = paintInfo.phase;

    if (paintPhase == PaintPhase::BlockBackground || paintPhase == PaintPhase::ChildBlockBackground) {
        // We need to handle painting a stack of backgrounds.  This stack (from bottom to top) consists of
        // the column group, column, row group, row, and then the cell.

        // Column groups and columns first.
        // FIXME: Columns and column groups do not currently support opacity, and they are being painted "too late" in
        // the stack, since we have already opened a transparency layer (potentially) for the table row group.
        // Note that we deliberately ignore whether or not the cell has a layer, since these backgrounds paint "behind" the
        // cell.
        if (RenderTableCol* column = table()->colElement(cell->col())) {
            if (RenderTableCol* columnGroup = column->enclosingColumnGroup())
                cell->paintBackgroundsBehindCell(paintInfo, cellPoint, columnGroup, cellPoint);
            cell->paintBackgroundsBehindCell(paintInfo, cellPoint, column, cellPoint);
        }

        // Paint the row group next.
        cell->paintBackgroundsBehindCell(paintInfo, cellPoint, this, paintOffset);

        // Paint the row next, but only if it doesn't have a layer.  If a row has a layer, it will be responsible for
        // painting the row background for the cell.
        if (!row.hasSelfPaintingLayer())
            cell->paintBackgroundsBehindCell(paintInfo, cellPoint, &row, rowBackgroundPaintOffset);
    }
    if ((!cell->hasSelfPaintingLayer() && !row.hasSelfPaintingLayer()))
        cell->paint(paintInfo, cellPoint);
}

LayoutRect RenderTableSection::logicalRectForWritingModeAndDirection(const LayoutRect& rect) const
{
    LayoutRect tableAlignedRect(rect);

    flipForWritingMode(tableAlignedRect);

    if (!writingMode().isHorizontal())
        tableAlignedRect = tableAlignedRect.transposedRect();

    const Vector<LayoutUnit>& columnPos = table()->columnPositions();
    // The table's writing mode determines in which direction the rows flow.
    if (table()->writingMode().isInlineFlipped())
        tableAlignedRect.setX(columnPos[columnPos.size() - 1] - tableAlignedRect.maxX());

    return tableAlignedRect;
}

CellSpan RenderTableSection::dirtiedRows(const LayoutRect& damageRect) const
{
    if (m_forceSlowPaintPathWithOverflowingCell) 
        return fullTableRowSpan();

    CellSpan coveredRows = spannedRows(damageRect, IncludeAllIntersectingCells);

    // To repaint the border we might need to repaint first or last row even if they are not spanned themselves.
    if (coveredRows.start >= m_rowPos.size() - 1 && m_rowPos.last() + table()->outerBorderAfter() >= damageRect.y())
        --coveredRows.start;

    if (!coveredRows.end && m_rowPos[0] - table()->outerBorderBefore() <= damageRect.maxY())
        ++coveredRows.end;

    return coveredRows;
}

CellSpan RenderTableSection::dirtiedColumns(const LayoutRect& damageRect) const
{
    if (m_forceSlowPaintPathWithOverflowingCell) 
        return fullTableColumnSpan();

    CellSpan coveredColumns = spannedColumns(damageRect, IncludeAllIntersectingCells);

    const Vector<LayoutUnit>& columnPos = table()->columnPositions();
    // To repaint the border we might need to repaint first or last column even if they are not spanned themselves.
    if (coveredColumns.start >= columnPos.size() - 1 && columnPos.last() + table()->outerBorderEnd() >= damageRect.x())
        --coveredColumns.start;

    if (!coveredColumns.end && columnPos[0] - table()->outerBorderStart() <= damageRect.maxX())
        ++coveredColumns.end;

    return coveredColumns;
}

CellSpan RenderTableSection::spannedRows(const LayoutRect& flippedRect, ShouldIncludeAllIntersectingCells shouldIncludeAllIntersectionCells) const
{
    // Find the first row that starts after rect top.
    unsigned nextRow = std::ranges::upper_bound(m_rowPos, flippedRect.y()) - m_rowPos.begin();
    if (shouldIncludeAllIntersectionCells == IncludeAllIntersectingCells && nextRow && m_rowPos[nextRow - 1] == flippedRect.y())
        --nextRow;

    if (nextRow == m_rowPos.size())
        return CellSpan(m_rowPos.size() - 1, m_rowPos.size() - 1); // After all rows.

    unsigned startRow = nextRow > 0 ? nextRow - 1 : 0;

    // Find the first row that starts after rect bottom.
    unsigned endRow;
    if (m_rowPos[nextRow] >= flippedRect.maxY())
        endRow = nextRow;
    else {
        endRow = std::upper_bound(m_rowPos.subspan(static_cast<int32_t>(nextRow)).data(), m_rowPos.end(), flippedRect.maxY()) - m_rowPos.begin();
        if (endRow == m_rowPos.size())
            endRow = m_rowPos.size() - 1;
    }

    return CellSpan(startRow, endRow);
}

CellSpan RenderTableSection::spannedColumns(const LayoutRect& flippedRect, ShouldIncludeAllIntersectingCells shouldIncludeAllIntersectionCells) const
{
    const Vector<LayoutUnit>& columnPos = table()->columnPositions();

    // Find the first column that starts after rect left.
    // lower_bound doesn't handle the edge between two cells properly as it would wrongly return the
    // cell on the logical top/left.
    // upper_bound on the other hand properly returns the cell on the logical bottom/right, which also
    // matches the behavior of other browsers.
    unsigned nextColumn = std::ranges::upper_bound(columnPos, flippedRect.x()) - columnPos.begin();
    if (shouldIncludeAllIntersectionCells == IncludeAllIntersectingCells && nextColumn && columnPos[nextColumn - 1] == flippedRect.x())
        --nextColumn;

    if (nextColumn == columnPos.size())
        return CellSpan(columnPos.size() - 1, columnPos.size() - 1); // After all columns.

    unsigned startColumn = nextColumn > 0 ? nextColumn - 1 : 0;

    // Find the first column that starts after rect right.
    unsigned endColumn;
    if (columnPos[nextColumn] >= flippedRect.maxX())
        endColumn = nextColumn;
    else {
        endColumn = std::upper_bound(columnPos.subspan(static_cast<int32_t>(nextColumn)).data(), columnPos.end(), flippedRect.maxX()) - columnPos.begin();
        if (endColumn == columnPos.size())
            endColumn = columnPos.size() - 1;
    }

    return CellSpan(startColumn, endColumn);
}

Color RenderTableSection::rowGroupBorderColor(CSSPropertyID borderColor) const
{
    switch (borderColor) {
    case CSSPropertyBorderTopColor:
        return style().visitedDependentBorderTopColorApplyingColorFilter();
    case CSSPropertyBorderRightColor:
        return style().visitedDependentBorderRightColorApplyingColorFilter();
    case CSSPropertyBorderBottomColor:
        return style().visitedDependentBorderBottomColorApplyingColorFilter();
    case CSSPropertyBorderLeftColor:
        return style().visitedDependentBorderLeftColorApplyingColorFilter();
    default:
        ASSERT_NOT_REACHED();
        return Color::black;
    }
}

void RenderTableSection::paintRowGroupBorder(const PaintInfo& paintInfo, bool antialias, LayoutRect rect, BoxSide side, CSSPropertyID borderColor, BorderStyle borderStyle, BorderStyle tableBorderStyle)
{
    if (tableBorderStyle == BorderStyle::Hidden)
        return;
    rect.intersect(paintInfo.rect);
    if (rect.isEmpty())
        return;
    BorderPainter::drawLineForBoxSide(paintInfo.context(), protect(document()), rect, side, rowGroupBorderColor(borderColor), borderStyle, 0, 0, antialias);
}

LayoutUnit RenderTableSection::offsetLeftForRowGroupBorder(RenderTableCell* cell, const LayoutRect& rowGroupRect, unsigned row)
{

    if (table()->writingMode().isHorizontal()) {
        if (table()->writingMode().isInlineLeftToRight())
            return cell ? cell->x() + cell->borderBoxWidth() : 0_lu;
        return -outerBorderLeft(table()->writingMode());
    }
    bool isLastRow = row + 1 == m_grid.size();
    return rowGroupRect.width() - m_rowPos[row + 1] + (isLastRow ? -outerBorderLeft(table()->writingMode()) : 0_lu);
}

LayoutUnit RenderTableSection::offsetTopForRowGroupBorder(RenderTableCell* cell, BoxSide borderSide, unsigned row)
{
    bool isLastRow = row + 1 == m_grid.size();

    if (table()->writingMode().isHorizontal())
        return m_rowPos[row] + (!row && borderSide == BoxSide::Right ? -outerBorderTop(table()->writingMode()) : isLastRow && borderSide == BoxSide::Left ? outerBorderTop(table()->writingMode()) : 0_lu);
    if (table()->writingMode().isInlineTopToBottom())
        return (cell ? cell->y() + cell->borderBoxHeight() : 0_lu) + (borderSide == BoxSide::Left ? outerBorderTop(table()->writingMode()) : 0_lu);
    return borderSide == BoxSide::Right ? -outerBorderTop(table()->writingMode()) : 0_lu;
}

LayoutUnit RenderTableSection::verticalRowGroupBorderHeight(RenderTableCell* cell, const LayoutRect& rowGroupRect, unsigned row)
{
    bool isLastRow = row + 1 == m_grid.size();

    if (table()->writingMode().isHorizontal())
        return m_rowPos[row + 1] - m_rowPos[row] + (!row ? outerBorderTop(table()->writingMode()) : isLastRow ? outerBorderBottom(table()->writingMode()) : 0_lu);
    if (table()->writingMode().isInlineTopToBottom())
        return rowGroupRect.height() - (cell ? cell->y() + cell->borderBoxHeight() : 0_lu) + outerBorderBottom(table()->writingMode());
    return cell ? rowGroupRect.height() - (cell->y() - cell->borderBoxHeight()) : 0_lu;
}

LayoutUnit RenderTableSection::horizontalRowGroupBorderWidth(RenderTableCell* cell, const LayoutRect& rowGroupRect, unsigned row, unsigned column)
{
    if (table()->writingMode().isHorizontal()) {
        if (table()->writingMode().isInlineLeftToRight())
            return rowGroupRect.width() - (cell ? cell->x() + cell->borderBoxWidth() : 0_lu) + (!column ? outerBorderLeft(table()->writingMode()) : column == table()->numEffCols() ? outerBorderRight(table()->writingMode()) : 0_lu);
        return cell ? rowGroupRect.width() - (cell->x() - cell->borderBoxWidth()) : 0_lu;
    }
    bool isLastRow = row + 1 == m_grid.size();
    return m_rowPos[row + 1] - m_rowPos[row] + (isLastRow ? outerBorderLeft(table()->writingMode()) : !row ? outerBorderRight(table()->writingMode()) : 0_lu);
}

void RenderTableSection::paintRowGroupBorderIfRequired(const PaintInfo& paintInfo, const LayoutPoint& paintOffset, unsigned row, unsigned column, BoxSide borderSide, RenderTableCell* cell)
{
    if (table()->currentBorderValue()->precedence() > BorderPrecedence::RowGroup)
        return;
    if (paintInfo.context().paintingDisabled())
        return;

    const Style::ComputedStyle& style = this->style();
    bool antialias = BorderPainter::shouldAntialiasLines(paintInfo.context());
    LayoutRect rowGroupRect = LayoutRect(paintOffset, borderBoxSize());
    rowGroupRect.moveBy(-LayoutPoint(outerBorderLeft(table()->writingMode()), (borderSide == BoxSide::Right) ? 0_lu : outerBorderTop(table()->writingMode())));

    switch (borderSide) {
    case BoxSide::Top:
        paintRowGroupBorder(
            paintInfo,
            antialias,
            LayoutRect {
                paintOffset.x() + offsetLeftForRowGroupBorder(cell, rowGroupRect, row),
                rowGroupRect.y(),
                horizontalRowGroupBorderWidth(cell, rowGroupRect, row, column),
                Style::evaluate<LayoutUnit>(style.borderTop().width, style.usedZoomForLength(), style.deviceScaleFactor()),
            },
            BoxSide::Top,
            CSSPropertyBorderTopColor,
            style.borderTopStyle(),
            table()->style().borderTopStyle()
        );
        break;
    case BoxSide::Bottom:
        paintRowGroupBorder(
            paintInfo,
            antialias,
            LayoutRect {
                paintOffset.x() + offsetLeftForRowGroupBorder(cell, rowGroupRect, row),
                rowGroupRect.y() + rowGroupRect.height(),
                horizontalRowGroupBorderWidth(cell, rowGroupRect, row, column),
                Style::evaluate<LayoutUnit>(style.borderBottom().width, style.usedZoomForLength(), style.deviceScaleFactor()),
            },
            BoxSide::Bottom,
            CSSPropertyBorderBottomColor,
            style.borderBottomStyle(),
            table()->style().borderBottomStyle()
        );
        break;
    case BoxSide::Left:
        paintRowGroupBorder(
            paintInfo,
            antialias,
            LayoutRect {
                rowGroupRect.x(),
                rowGroupRect.y() + offsetTopForRowGroupBorder(cell, borderSide, row),
                Style::evaluate<LayoutUnit>(style.borderLeft().width, style.usedZoomForLength(), style.deviceScaleFactor()),
                verticalRowGroupBorderHeight(cell, rowGroupRect, row),
            },
            BoxSide::Left,
            CSSPropertyBorderLeftColor,
            style.borderLeftStyle(),
            table()->style().borderLeftStyle()
        );
        break;
    case BoxSide::Right:
        paintRowGroupBorder(
            paintInfo,
            antialias,
            LayoutRect {
                rowGroupRect.x() + rowGroupRect.width(),
                rowGroupRect.y() + offsetTopForRowGroupBorder(cell, borderSide, row),
                Style::evaluate<LayoutUnit>(style.borderRight().width, style.usedZoomForLength(), style.deviceScaleFactor()),
                verticalRowGroupBorderHeight(cell, rowGroupRect, row),
            },
            BoxSide::Right,
            CSSPropertyBorderRightColor,
            style.borderRightStyle(),
            table()->style().borderRightStyle()
        );
        break;
    default:
        break;
    }

}

static BoxSide NODELETE physicalBorderForDirection(const WritingMode writingMode, CollapsedBorderSide side)
{
    // FIXME: Replace this with types/methods from BoxSides.h
    switch (side) {
    case CollapsedBorderSide::Start:
        if (writingMode.isHorizontal())
            return writingMode.isInlineLeftToRight() ? BoxSide::Left : BoxSide::Right;
        return writingMode.isInlineTopToBottom() ? BoxSide::Top : BoxSide::Bottom;
    case CollapsedBorderSide::End:
        if (writingMode.isHorizontal())
            return writingMode.isInlineLeftToRight() ? BoxSide::Right : BoxSide::Left;
        return writingMode.isInlineTopToBottom() ? BoxSide::Bottom : BoxSide::Top;
    case CollapsedBorderSide::Before:
        if (writingMode.isHorizontal())
            return writingMode.isBlockTopToBottom() ? BoxSide::Top : BoxSide::Bottom;
        return writingMode.isBlockLeftToRight() ? BoxSide::Left : BoxSide::Right;
    case CollapsedBorderSide::After:
        if (writingMode.isHorizontal())
            return writingMode.isBlockTopToBottom() ? BoxSide::Bottom : BoxSide::Top;
        return writingMode.isBlockLeftToRight() ? BoxSide::Right : BoxSide::Left;
    }
    ASSERT_NOT_REACHED();
    return BoxSide::Left;
}

void RenderTableSection::paintObject(PaintInfo& paintInfo, const LayoutPoint& paintOffset)
{
    if (paintInfo.phase == PaintPhase::Accessibility) {
        if (auto* context = paintInfo.accessibilityRegionContext()) {
            context->takeBounds(*this, paintOffset);
            for (auto& rowStruct : m_grid) {
                if (auto* row = rowStruct.rowRenderer; row && !row->hasSelfPaintingLayer())
                    context->takeBounds(*row, paintOffset + row->location());
            }
        }
    }

    auto localRepaintRect = paintInfo.rect;
    localRepaintRect.moveBy(-paintOffset);

    CellSpan dirtiedRows { 0, 0 };
    CellSpan dirtiedColumns { 0, 0 };

    if (localRepaintRect.contains(borderBoxRectInContainer())) {
        dirtiedRows = fullTableRowSpan();
        dirtiedColumns = fullTableColumnSpan();
    } else {
        auto tableAlignedRect = logicalRectForWritingModeAndDirection(localRepaintRect);
        dirtiedRows = this->dirtiedRows(tableAlignedRect);
        dirtiedColumns = this->dirtiedColumns(tableAlignedRect);
    }

    if (dirtiedColumns.start == dirtiedColumns.end)
        return;

    auto paintRowOutline = [&](unsigned rowIndex, PaintPhase phase) {
        if (phase != PaintPhase::Outline && phase != PaintPhase::SelfOutline)
            return;

        auto* row = m_grid[rowIndex].rowRenderer;
        if (row && !row->hasSelfPaintingLayer() && row->hasOutline())
            row->paintOutlineForRowIfNeeded(paintInfo, paintOffset);
    };

    auto paintRowShadow = [&](unsigned rowIndex, PaintPhase phase) {
        if (phase != PaintPhase::BlockBackground && phase != PaintPhase::ChildBlockBackground)
            return;

        auto* row = m_grid[rowIndex].rowRenderer;
        if (row && !row->hasSelfPaintingLayer() && !row->style().boxShadow().isNone())
            row->paintShadowForRowIfNeeded(paintInfo, paintOffset);
    };

    auto paintContiguousCells = [&]() {
        // Draw the dirty cells in the order that they appear.
        for (unsigned r = dirtiedRows.start; r < dirtiedRows.end; r++) {
            paintRowShadow(r , paintInfo.phase);
            paintRowOutline(r, paintInfo.phase);

            for (unsigned c = dirtiedColumns.start; c < dirtiedColumns.end; c++) {
                CellStruct& current = cellAt(r, c);
                RenderTableCell* cell = current.primaryCell();
                if (!cell || (r > dirtiedRows.start && primaryCellAt(r - 1, c) == cell) || (c > dirtiedColumns.start && primaryCellAt(r, c - 1) == cell))
                    continue;
                paintCell(cell, paintInfo, paintOffset);
            }
        }
    };

    auto paintContiguousCellsWithCollapsedBorders = [&]() {
        // Collapsed borders are painted from the bottom right to the top left so that precedence
        // due to cell position is respected. We need to paint one row beyond the topmost dirtied
        // row to calculate its collapsed border value.
        unsigned startRow = dirtiedRows.start ? dirtiedRows.start - 1 : 0;
        for (unsigned r = dirtiedRows.end; r > startRow; r--) {
            unsigned row = r - 1;
            bool shouldPaintRowGroupBorder = false;
            for (unsigned c = dirtiedColumns.end; c > dirtiedColumns.start; c--) {
                unsigned col = c - 1;
                CellStruct& current = cellAt(row, col);
                RenderTableCell* cell = current.primaryCell();
                if (!cell) {
                    if (!c)
                        paintRowGroupBorderIfRequired(paintInfo, paintOffset, row, col, physicalBorderForDirection(table()->writingMode(), CollapsedBorderSide::Start));
                    else if (c == table()->numEffCols())
                        paintRowGroupBorderIfRequired(paintInfo, paintOffset, row, col, physicalBorderForDirection(table()->writingMode(), CollapsedBorderSide::End));

                    shouldPaintRowGroupBorder = true;
                    continue;
                }

                if ((row > dirtiedRows.start && primaryCellAt(row - 1, col) == cell) || (col > dirtiedColumns.start && primaryCellAt(row, col - 1) == cell))
                    continue;

                // If we had a run of null cells paint their corresponding section of the row group's border if necessary. Note that
                // this will only happen once within a row as the null cells will always be clustered together on one end of the row.
                if (shouldPaintRowGroupBorder) {
                    if (r == m_grid.size())
                        paintRowGroupBorderIfRequired(paintInfo, paintOffset, row, col, physicalBorderForDirection(table()->writingMode(), CollapsedBorderSide::After), cell);
                    else if (!row && !table()->sectionAbove(this))
                        paintRowGroupBorderIfRequired(paintInfo, paintOffset, row, col, physicalBorderForDirection(table()->writingMode(), CollapsedBorderSide::Before), cell);

                    shouldPaintRowGroupBorder = false;
                }

                auto& row = downcast<RenderTableRow>(*cell->parent());
                auto rowPaintOffset = flipForWritingModeForChild(row, paintOffset) + row.location();
                auto cellPoint = row.flipForWritingModeForChild(*cell, rowPaintOffset);
                cell->paintCollapsedBorders(paintInfo, cellPoint);
            }
        }
    };

    auto paintDirtyCells = [&]() {
        // The overflowing cells should be scarce to avoid adding a lot of cells to the HashSet.
#if ASSERT_ENABLED
        unsigned totalRows = m_grid.size();
        unsigned totalCols = table()->columns().size();
        ASSERT(m_overflowingCells.computeSize() < totalRows * totalCols * gMaxAllowedOverflowingCellRatioForFastPaintPath);
#endif

        // To make sure we properly repaint the section, we repaint all the overflowing cells that we collected.
        auto cells = copyToVector(m_overflowingCells);

        HashSet<CheckedPtr<RenderTableCell>> spanningCells;

        for (unsigned r = dirtiedRows.start; r < dirtiedRows.end; r++) {
            paintRowOutline(r, paintInfo.phase);

            for (unsigned c = dirtiedColumns.start; c < dirtiedColumns.end; c++) {
                CellStruct& current = cellAt(r, c);
                if (!current.hasCells())
                    continue;

                for (unsigned i = 0; i < current.cells.size(); ++i) {
                    if (m_overflowingCells.contains(*current.cells[i]))
                        continue;

                    if (current.cells[i]->rowSpan() > 1 || current.cells[i]->colSpan() > 1) {
                        if (!spanningCells.add(current.cells[i]).isNewEntry)
                            continue;
                    }

                    cells.append(current.cells[i]);
                }
            }
        }

        // Sort the dirty cells by paint order.
        if (m_overflowingCells.isEmptyIgnoringNullReferences())
            std::ranges::stable_sort(cells, compareCellPositions);
        else
            std::ranges::sort(cells, compareCellPositionsWithOverflowingCells);

        if (paintInfo.phase == PaintPhase::CollapsedTableBorders) {
            for (unsigned i = cells.size(); i > 0; --i) {
                auto& row = downcast<RenderTableRow>(*cells[i - 1]->parent());
                auto rowPaintOffset = flipForWritingModeForChild(row, paintOffset) + row.location();
                auto cellPoint = row.flipForWritingModeForChild(*cells[i - 1], rowPaintOffset);
                cells[i - 1]->paintCollapsedBorders(paintInfo, cellPoint);
            }
        } else {
            for (unsigned i = 0; i < cells.size(); ++i)
                paintCell(cells[i].get(), paintInfo, paintOffset);
        }
    };

    if (!m_hasMultipleCellLevels && m_overflowingCells.isEmptyIgnoringNullReferences()) {
        if (paintInfo.phase == PaintPhase::CollapsedTableBorders)
            paintContiguousCellsWithCollapsedBorders();
        else
            paintContiguousCells();
    } else
        paintDirtyCells();
}

void RenderTableSection::imageChanged(WrappedImagePtr, const IntRect*)
{
    // FIXME: Examine cells and repaint only the rect the image paints in.
    if (!parent())
        return;
    repaint();
}

void RenderTableSection::recalcCells()
{
    ASSERT(m_needsCellRecalc);
    // We reset the flag here to ensure that addCell() works. This is safe to do because we clear the grid
    // and update its dimensions to be consistent with the table's column representation before we rebuild
    // the grid using addCell().
    m_needsCellRecalc = false;

    m_cCol = 0;
    m_cRow = 0;
    m_grid.clear();

    for (RenderTableRow* row = firstRow(); row; row = row->nextRow()) {
        unsigned insertionRow = m_cRow;
        m_cRow++;
        m_cCol = 0;
        ensureRows(m_cRow);

        m_grid[insertionRow].rowRenderer = row;
        row->setRowIndex(insertionRow);
        setRowLogicalHeightToRowStyleLogicalHeight(m_grid[insertionRow]);

        for (RenderTableCell* cell = row->firstCell(); cell; cell = cell->nextCell())
            addCell(cell, row);
    }

    m_grid.shrinkToFit();
    setNeedsLayout();
}

void RenderTableSection::removeRedundantColumns()
{
    auto maximumNumberOfColumns = table()->numEffCols();
    for (auto& rowItem : m_grid) {
        if (rowItem.row.size() <= maximumNumberOfColumns)
            continue;
        rowItem.row.shrink(maximumNumberOfColumns);
    }
}

// FIXME: This function could be made O(1) in certain cases (like for the non-most-constrainive cells' case).
void RenderTableSection::rowLogicalHeightChanged(unsigned rowIndex)
{
    if (needsCellRecalc())
        return;

    setRowLogicalHeightToRowStyleLogicalHeight(m_grid[rowIndex]);

    for (RenderTableCell* cell = m_grid[rowIndex].rowRenderer->firstCell(); cell; cell = cell->nextCell())
        updateLogicalHeightForCell(m_grid[rowIndex], cell);
}

void RenderTableSection::setNeedsCellRecalc()
{
    m_needsCellRecalc = true;

    // Clear the grid now to ensure that we don't hold onto any stale pointers (e.g. a cell renderer that is being removed).
    m_grid.clear();

    if (RenderTable* t = table())
        t->setNeedsSectionRecalc();
}

unsigned RenderTableSection::numColumns() const
{
    ASSERT(!m_needsCellRecalc);
    unsigned result = 0;
    
    for (unsigned r = 0; r < m_grid.size(); ++r) {
        for (unsigned c = result; c < table()->numEffCols(); ++c) {
            const CellStruct& cell = cellAt(r, c);
            if (cell.hasCells() || cell.inColSpan)
                result = c;
        }
    }
    
    return result + 1;
}

const BorderValue& RenderTableSection::borderAdjoiningStartCell(const RenderTableCell& cell) const
{
    ASSERT_UNUSED(cell, cell.isFirstOrLastCellInRow());
    return style().borderStart(table()->writingMode());
}

const BorderValue& RenderTableSection::borderAdjoiningEndCell(const RenderTableCell& cell) const
{
    ASSERT_UNUSED(cell, cell.isFirstOrLastCellInRow());
    return style().borderEnd(table()->writingMode());
}

void RenderTableSection::appendColumn(unsigned pos)
{
    ASSERT(!m_needsCellRecalc);

    for (unsigned row = 0; row < m_grid.size(); ++row)
        m_grid[row].row.resize(pos + 1);
}

void RenderTableSection::splitColumn(unsigned pos, unsigned first)
{
    ASSERT(!m_needsCellRecalc);

    if (m_cCol > pos)
        m_cCol++;
    for (unsigned row = 0; row < m_grid.size(); ++row) {
        Row& r = m_grid[row].row;
        r.insert(pos + 1, CellStruct());
        if (r[pos].hasCells()) {
            r[pos + 1].cells.appendVector(r[pos].cells);
            RenderTableCell* cell = r[pos].primaryCell();
            ASSERT(cell);
            ASSERT(cell->colSpan() >= (r[pos].inColSpan ? 1u : 0));
            unsigned colleft = cell->colSpan() - r[pos].inColSpan;
            if (first > colleft)
              r[pos + 1].inColSpan = 0;
            else
              r[pos + 1].inColSpan = first + r[pos].inColSpan;
        } else {
            r[pos + 1].inColSpan = 0;
        }
    }
}

// Hit Testing
bool RenderTableSection::nodeAtPoint(const HitTestRequest& request, HitTestResult& result, const HitTestLocation& locationInContainer, const LayoutPoint& accumulatedOffset, HitTestAction action)
{
    // If we have no children then we have nothing to do.
    if (!firstRow())
        return false;

    // Table sections cannot ever be hit tested.  Effectively they do not exist.
    // Just forward to our children always.
    LayoutPoint adjustedLocation = accumulatedOffset + location();

    if (hasNonVisibleOverflow() && !locationInContainer.intersects(overflowClipRect(adjustedLocation)))
        return false;

    if (hasOverflowingCell()) {
        for (RenderTableRow* row = lastRow(); row; row = row->previousRow()) {
            // FIXME: We have to skip over inline flows, since they can show up inside table rows
            // at the moment (a demoted inline <form> for example). If we ever implement a
            // table-specific hit-test method (which we should do for performance reasons anyway),
            // then we can remove this check.
            if (!row->hasSelfPaintingLayer()) {
                auto rowOffset = flipForWritingModeForChild(*row, adjustedLocation);
                if (row->nodeAtPoint(request, result, locationInContainer, rowOffset, action))
                    return true;
            }
        }
        return false;
    }

    recalcCellsIfNeeded();

    LayoutRect hitTestRect = locationInContainer.boundingBox();
    hitTestRect.moveBy(-adjustedLocation);

    LayoutRect tableAlignedRect = logicalRectForWritingModeAndDirection(hitTestRect);
    CellSpan rowSpan = spannedRows(tableAlignedRect, DoNotIncludeAllIntersectingCells);
    CellSpan columnSpan = spannedColumns(tableAlignedRect, DoNotIncludeAllIntersectingCells);

    // Now iterate over the spanned rows and columns.
    for (unsigned hitRow = rowSpan.start; hitRow < rowSpan.end; ++hitRow) {
        for (unsigned hitColumn = columnSpan.start; hitColumn < columnSpan.end; ++hitColumn) {
            CellStruct& current = cellAt(hitRow, hitColumn);

            // If the cell is empty, there's nothing to do
            if (!current.hasCells())
                continue;

            for (unsigned i = current.cells.size() ; i; ) {
                --i;
                RenderTableCell* cell = current.cells[i];
                auto& row = downcast<RenderTableRow>(*cell->parent());
                auto rowPoint = flipForWritingModeForChild(row, adjustedLocation) + row.location();
                auto cellPoint = row.flipForWritingModeForChild(*cell, rowPoint);
                if (static_cast<RenderObject*>(cell)->nodeAtPoint(request, result, locationInContainer, cellPoint, action)) {
                    updateHitTestResult(result, locationInContainer.point() - toLayoutSize(cellPoint));
                    return true;
                }
            }
            if (!request.resultIsElementList())
                break;
        }
        if (!request.resultIsElementList())
            break;
    }

    return false;
}

void RenderTableSection::clearCachedCollapsedBorders()
{
    if (!table()->collapseBorders())
        return;
    m_cellsCollapsedBorders.clear();
}

void RenderTableSection::removeCachedCollapsedBorders(const RenderTableCell& cell)
{
    if (!table()->collapseBorders())
        return;
    
    for (auto side = std::to_underlying(CollapsedBorderSide::Before); side <= std::to_underlying(CollapsedBorderSide::End); ++side)
        m_cellsCollapsedBorders.remove(std::make_pair(&cell, side));
}

void RenderTableSection::setCachedCollapsedBorder(const RenderTableCell& cell, CollapsedBorderSide side, CollapsedBorderValue border)
{
    ASSERT(table()->collapseBorders());
    ASSERT(border.width());
    m_cellsCollapsedBorders.set(std::make_pair(&cell, std::to_underlying(side)), border);
}

CollapsedBorderValue RenderTableSection::cachedCollapsedBorder(const RenderTableCell& cell, CollapsedBorderSide side)
{
    ASSERT(table()->collapseBorders() && table()->collapsedBordersAreValid());
    auto it = m_cellsCollapsedBorders.find(std::make_pair(&cell, std::to_underlying(side)));
    // Only non-empty collapsed borders are in the hashmap.
    if (it == m_cellsCollapsedBorders.end())
        return CollapsedBorderValue(BorderValue(), Color(), BorderPrecedence::Cell, cell.style().usedZoomForLength(), cell.style().deviceScaleFactor());
    return it->value;
}

void RenderTableSection::setLogicalPositionForCell(RenderTableCell* cell, unsigned effectiveColumn) const
{
    LayoutPoint oldCellLocation = cell->location();

    LayoutPoint cellLocation(0_lu, 0_lu);
    LayoutUnit horizontalBorderSpacing = table()->hBorderSpacing();

    // The table's writing mode determines in which direction the rows flow.
    if (table()->writingMode().isInlineFlipped())
        cellLocation.setX(table()->columnPositions()[table()->numEffCols()] - table()->columnPositions()[table()->colToEffCol(cell->col() + cell->colSpan())] + horizontalBorderSpacing);
    else
        cellLocation.setX(table()->columnPositions()[effectiveColumn] + horizontalBorderSpacing);

    if (cell->isOrthogonal())
        cellLocation = cellLocation.transposedPoint();
    cell->setLogicalLocation(cellLocation);
    view().frameView().layoutContext().addLayoutDelta(oldCellLocation - cell->location());
}

} // namespace WebCore
```

