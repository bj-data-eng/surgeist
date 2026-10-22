// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Metadata and line-link projection adapted from Parley 0.9.0 layout/accessibility.rs.
// Surgeist owns source-unit grouping, checked identities, geometry and atomic publication.
// Exact origin and modifications: ../docs/accessibility-provenance.md.

use std::{collections::BTreeMap, sync::Arc};

use accesskit::{Node, NodeId, Role, TextDirection, TextPosition, TextSelection, TreeUpdate};
use skrifa::{
    FontRef,
    raw::{TableProvider, types::NameId},
};

use crate::{
    Affinity, Alignment, Cursor, Error, ErrorCode, ErrorDetail, Layout, Point, Range, Rect, Result,
    Selection, Text,
};

/// Accessibility bindings for one exact analyzed-source or placed projection.
#[derive(Clone)]
pub struct Accessibility {
    binding: Option<Binding>,
    spans: Vec<AccessSpan>,
    next_id: Option<u64>,
}

#[derive(Clone)]
struct AccessSpan {
    range: Range,
    empty_line: Option<usize>,
    id: NodeId,
    hard_break: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct NodeDescriptor {
    range: Range,
    empty_line: Option<usize>,
}

impl NodeDescriptor {
    fn key(self) -> (usize, usize, Option<usize>) {
        (self.range.start, self.range.end, self.empty_line)
    }
}

impl Accessibility {
    #[must_use]
    pub fn new(first_node_id: u64) -> Self {
        Self {
            binding: None,
            spans: Vec::new(),
            next_id: Some(first_node_id),
        }
    }
}

impl Default for Accessibility {
    fn default() -> Self {
        Self::new(1)
    }
}

#[derive(Clone)]
struct Metadata {
    family: Option<String>,
    size: f32,
    weight: f32,
    italic: bool,
    locale: Option<String>,
}

impl Metadata {
    fn from_run(run: &crate::engine::layout::data::PreparedRun, locale: Option<String>) -> Self {
        let font = &run.font;
        let family = FontRef::from_index(font.bytes(), font.index())
            .ok()
            .and_then(|font| font.name().ok())
            .and_then(|name| {
                name.name_record()
                    .iter()
                    .find(|record| record.name_id.get() == NameId::FAMILY_NAME)
                    .and_then(|record| record.string(name.string_data()).ok())
                    .map(|name| name.to_string())
            });
        Self {
            family,
            size: run.font_size,
            weight: run.font_attrs.weight.value(),
            italic: matches!(run.font_attrs.style, fontique::FontStyle::Italic),
            locale,
        }
    }

    fn apply(&self, node: &mut Node, pieces: &[&Fragment]) {
        if pieces
            .iter()
            .all(|piece| piece.metadata.family == self.family)
            && let Some(family) = &self.family
        {
            node.set_font_family(family.clone());
        }
        if pieces.iter().all(|piece| piece.metadata.size == self.size) {
            node.set_font_size(self.size);
        }
        if pieces
            .iter()
            .all(|piece| piece.metadata.weight == self.weight)
        {
            node.set_font_weight(self.weight);
        }
        if self.italic
            && pieces
                .iter()
                .all(|piece| piece.metadata.italic == self.italic)
        {
            node.set_italic();
        }
        if pieces
            .iter()
            .all(|piece| piece.metadata.locale == self.locale)
            && let Some(locale) = &self.locale
        {
            node.set_language(locale.clone());
        }
    }
}

struct Fragment {
    range: Range,
    bounds: Rect,
    line: usize,
    rtl: bool,
    word: bool,
    metadata: Metadata,
}

struct LocaleRun {
    range: Range,
    locale: Option<String>,
}

struct LocaleIndex(Vec<LocaleRun>);

impl LocaleIndex {
    fn new(styles: &[crate::engine::resolve::AnalyzedStyleRun]) -> Self {
        let mut runs = Vec::new();
        for run in styles {
            Self::push(
                &mut runs,
                run.range.start,
                run.range.end,
                run.style.authored().locale.clone(),
            );
        }
        Self(runs)
    }

    fn push(runs: &mut Vec<LocaleRun>, start: usize, end: usize, locale: Option<String>) {
        if start == end {
            return;
        }
        if let Some(last) = runs.last_mut()
            && last.range.end == start
            && last.locale == locale
        {
            last.range.end = end;
        } else {
            runs.push(LocaleRun {
                range: Range::new(start, end),
                locale,
            });
        }
    }

    fn uniform(&self, range: Range) -> Option<String> {
        let first = self
            .0
            .get(self.0.partition_point(|run| run.range.end <= range.start))?;
        (range.end <= first.range.end)
            .then(|| first.locale.clone())
            .flatten()
    }
}

// Selectable nonempty ranges partition source units; empty line ranges may also occur.
// Distribute actual intersections, retaining fragment order even for overlapping or
// visually reordered fragments. No per-unit scan over unrelated fragments is needed.
fn fragment_memberships(ranges: &[Range], fragments: &[Fragment]) -> Vec<Vec<usize>> {
    let mut nonempty = Vec::new();
    let mut empty = Vec::new();
    for (index, range) in ranges.iter().copied().enumerate() {
        if range.is_empty() {
            empty.push((range.start, index));
        } else {
            nonempty.push((range, index));
        }
    }
    let mut memberships = vec![Vec::new(); ranges.len()];
    for (fragment_index, fragment) in fragments.iter().enumerate() {
        let first = nonempty.partition_point(|(range, _)| range.end <= fragment.range.start);
        for &(range, index) in &nonempty[first..] {
            if range.start >= fragment.range.end {
                break;
            }
            memberships[index].push(fragment_index);
        }
        let first = empty.partition_point(|(position, _)| *position <= fragment.range.start);
        for &(position, index) in &empty[first..] {
            if position >= fragment.range.end {
                break;
            }
            memberships[index].push(fragment_index);
        }
    }
    memberships
}

#[derive(Clone)]
struct Binding {
    analyzed: Arc<crate::engine::AnalyzedData>,
    placed: Option<Arc<()>>,
}

#[derive(Clone, Copy)]
enum Projection<'a> {
    Analyzed(&'a Arc<crate::engine::AnalyzedData>),
    Placed(&'a Layout, Point),
}

impl Projection<'_> {
    fn analyzed(&self) -> &Arc<crate::engine::AnalyzedData> {
        match self {
            Self::Analyzed(analyzed) => analyzed,
            Self::Placed(layout, _) => &layout.inner.prepared.analyzed,
        }
    }

    fn binding(self) -> Binding {
        Binding {
            analyzed: self.analyzed().clone(),
            placed: match self {
                Self::Analyzed(_) => None,
                Self::Placed(layout, _) => Some(layout.identity.clone()),
            },
        }
    }
}

pub(crate) fn analyzed_update(
    analyzed: &Arc<crate::engine::AnalyzedData>,
    state: &mut Accessibility,
    parent: NodeId,
    focus: NodeId,
) -> Result<TreeUpdate> {
    update(Projection::Analyzed(analyzed), state, parent, focus)
}

pub(crate) fn analyzed_access_position(
    analyzed: &Arc<crate::engine::AnalyzedData>,
    state: &Accessibility,
    cursor: Cursor,
) -> Result<Option<TextPosition>> {
    validate_binding(state, analyzed, None)?;
    crate::range::validate_index(
        analyzed.source.text(),
        cursor.index(),
        "accessibility cursor",
    )?;
    let unit = crate::engine::editing::caret_at(
        &analyzed.projection.graphemes,
        cursor.index(),
        cursor.affinity(),
    );
    Ok(access_stop(state, unit, cursor.affinity()))
}

pub(crate) fn analyzed_access_selection(
    analyzed: &Arc<crate::engine::AnalyzedData>,
    state: &Accessibility,
    selection: Selection,
) -> Result<Option<TextSelection>> {
    validate_binding(state, analyzed, None)?;
    crate::range::validate(analyzed.source.text(), selection.range())?;
    let (anchor, focus) = if selection.is_collapsed() {
        let point = analyzed_access_position(analyzed, state, selection.focus())?;
        (point, point)
    } else {
        let range = selection.range();
        let start = crate::engine::editing::caret_at(
            &analyzed.projection.graphemes,
            range.start,
            Affinity::Before,
        );
        let end = crate::engine::editing::caret_at(
            &analyzed.projection.graphemes,
            range.end,
            Affinity::After,
        );
        selection_positions(state, selection, Range::new(start, end))
    };
    Ok(anchor
        .zip(focus)
        .map(|(anchor, focus)| TextSelection { anchor, focus }))
}

fn selection_positions(
    state: &Accessibility,
    selection: Selection,
    range: Range,
) -> (Option<TextPosition>, Option<TextPosition>) {
    let start = access_stop(state, range.start, Affinity::After);
    let end = access_stop(state, range.end, Affinity::Before);
    if selection.anchor().index() <= selection.focus().index() {
        (start, end)
    } else {
        (end, start)
    }
}

fn validate_binding(
    state: &Accessibility,
    analyzed: &Arc<crate::engine::AnalyzedData>,
    placed: Option<&Arc<()>>,
) -> Result<()> {
    let valid = state.binding.as_ref().is_some_and(|binding| {
        Arc::ptr_eq(&binding.analyzed, analyzed)
            && match (&binding.placed, placed) {
                (None, None) => true,
                (Some(actual), Some(expected)) => Arc::ptr_eq(actual, expected),
                _ => false,
            }
    });
    if !valid {
        return Err(ax_error(ErrorDetail::AccessibilityBindingMismatch));
    }
    Ok(())
}

fn access_stop(state: &Accessibility, unit: usize, affinity: Affinity) -> Option<TextPosition> {
    let span = match affinity {
        Affinity::After => state.spans.iter().find(|span| span.range.start == unit),
        Affinity::Before => state.spans.iter().rev().find(|span| span.range.end == unit),
    }
    .or_else(|| state.spans.iter().rev().find(|span| span.range.end == unit))
    .or_else(|| state.spans.iter().find(|span| span.range.start == unit))?;
    let character_index =
        usize::from(unit == span.range.end && !span.range.is_empty() && !span.hard_break);
    Some(TextPosition {
        node: span.id,
        character_index,
    })
}

fn update(
    projection: Projection<'_>,
    state: &mut Accessibility,
    parent: NodeId,
    focus: NodeId,
) -> Result<TreeUpdate> {
    let analyzed = projection.analyzed();
    let host_text = original_host_text(analyzed.source.text())?;
    let locales = LocaleIndex::new(&analyzed.styles);
    let lines = match projection {
        Projection::Analyzed(_) => Vec::new(),
        Projection::Placed(layout, _) => layout.lines(),
    };
    let mut fragments = Vec::new();
    if let Projection::Placed(layout, _) = projection {
        for (line_index, line) in layout.inner.lines.iter().enumerate() {
            let data = &line.finalized.data;
            let metrics = &data.metrics;
            for cluster in &data.clusters {
                let run = &data.font_runs[cluster.run];
                fragments.push(Fragment {
                    range: cluster.source,
                    bounds: Rect::new(
                        line.origin.x + cluster.x,
                        line.origin.y + metrics.block_min,
                        cluster.advance,
                        metrics.block_max - metrics.block_min,
                    ),
                    line: line_index,
                    rtl: cluster.level & 1 != 0,
                    word: cluster.boundary >= crate::engine::analysis::Boundary::Word
                        && !cluster.whitespace.is_space_or_nbsp(),
                    metadata: Metadata::from_run(run, locales.uniform(cluster.source)),
                });
            }
        }
    }
    let mut descriptors: Vec<_> = match projection {
        Projection::Analyzed(_) => analyzed
            .projection
            .graphemes
            .windows(2)
            .map(|pair| NodeDescriptor {
                range: Range::new(pair[0], pair[1]),
                empty_line: None,
            })
            .collect(),
        Projection::Placed(layout, _) => layout
            .inner
            .interaction
            .selectable_ranges()
            .map(|range| NodeDescriptor {
                range,
                empty_line: None,
            })
            .collect(),
    };
    if matches!(projection, Projection::Analyzed(_)) && analyzed.source.text().is_empty() {
        descriptors.push(NodeDescriptor {
            range: Range::new(0, 0),
            empty_line: None,
        });
    }
    for line in &lines {
        if line.range().is_empty() {
            descriptors.push(NodeDescriptor {
                range: line.range(),
                empty_line: Some(line.index()),
            });
        }
    }
    descriptors.sort_by_key(|descriptor| descriptor.key());
    descriptors.dedup();
    let same = state
        .binding
        .as_ref()
        .is_some_and(|binding| Arc::ptr_eq(&binding.analyzed, analyzed));
    let retained: BTreeMap<_, _> = if same {
        state
            .spans
            .iter()
            .map(|span| ((span.range.start, span.range.end, span.empty_line), span.id))
            .collect()
    } else {
        BTreeMap::new()
    };
    let missing = descriptors
        .iter()
        .filter(|descriptor| !retained.contains_key(&descriptor.key()))
        .count();
    let available = state.next_id.map_or(0, |next| {
        let excluded =
            u128::from(parent.0 >= next) + u128::from(focus != parent && focus.0 >= next);
        u128::from(u64::MAX) - u128::from(next) + 1 - excluded
    });
    if missing as u128 > available {
        return Err(ax_error(ErrorDetail::AccessibilityIdentityExhausted));
    }
    let mut next_id = state.next_id;
    let mut spans = Vec::new();
    let mut nodes = Vec::new();
    let mut line_nodes = Vec::<(usize, usize)>::new();
    let ranges: Vec<_> = descriptors
        .iter()
        .map(|descriptor| descriptor.range)
        .collect();
    let memberships = fragment_memberships(&ranges, &fragments);
    for (range_index, descriptor) in descriptors.into_iter().enumerate() {
        let range = descriptor.range;
        let start = analyzed.projection.byte(range.start).0;
        let end = analyzed.projection.byte(range.end).0;
        let text = &host_text[start..end];
        let length = u8::try_from(text.len()).map_err(|_| {
            ax_error(ErrorDetail::UnsupportedAccessibilityUnit {
                range,
                bytes: text.len(),
            })
        })?;
        let id = retained.get(&descriptor.key()).copied();
        let id = match id {
            Some(id) => id,
            None => issue_id(&mut next_id, parent, focus)?,
        };
        if id == parent {
            return Err(ax_error(ErrorDetail::AccessibilityIdentityConflict {
                id: id.0,
            }));
        }
        let pieces: Vec<_> = memberships[range_index]
            .iter()
            .map(|index| &fragments[*index])
            .filter(|fragment| {
                descriptor
                    .empty_line
                    .is_none_or(|line| fragment.line == line)
            })
            .collect();
        let mut node = Node::new(Role::TextRun);
        if matches!(projection, Projection::Analyzed(_))
            && let Some(locale) = locales.uniform(range)
        {
            node.set_language(locale);
        }

        node.set_value(text.to_owned());
        node.set_character_lengths(if range.is_empty() {
            Vec::new()
        } else {
            vec![length]
        });
        if let Some(first) = pieces.first() {
            let Projection::Placed(_, origin) = projection else {
                unreachable!("only placed rows have fragments")
            };
            let bounds = pieces
                .iter()
                .skip(1)
                .fold(first.bounds, |bounds, fragment| {
                    union(bounds, fragment.bounds)
                });
            node.set_bounds(host_rect(bounds, origin)?);
            let direction = pieces.iter().all(|fragment| fragment.rtl == first.rtl);
            let one_line = pieces.iter().all(|fragment| fragment.line == first.line);
            if direction {
                node.set_text_direction(if first.rtl {
                    TextDirection::RightToLeft
                } else {
                    TextDirection::LeftToRight
                });
            }
            first.metadata.apply(&mut node, &pieces);
            if pieces
                .iter()
                .any(|fragment| fragment.range.start == range.start && fragment.word)
            {
                node.set_word_starts(vec![0]);
            }
            // Single fragments have an exact one-dimensional host projection. Multiple
            // pieces remain exact internally; omit optional fields rather than invent precision.
            if pieces.len() == 1 {
                node.set_character_positions(vec![0.0]);
                node.set_character_widths(vec![host_width(bounds.size.width)?]);
            }
            if one_line {
                line_nodes.push((first.line, nodes.len()));
            }
        } else if let Some(line) = descriptor.empty_line.and_then(|index| lines.get(index)) {
            let Projection::Placed(_, origin) = projection else {
                unreachable!("only placed rows have visual lines")
            };
            node.set_bounds(host_rect(line.bounds(), origin)?);
            line_nodes.push((line.index(), nodes.len()));
        }
        if let Projection::Placed(layout, _) = projection {
            node.set_text_align(match layout.options.alignment {
                Alignment::Start => {
                    if layout.direction() == crate::Direction::RightToLeft {
                        accesskit::TextAlign::Right
                    } else {
                        accesskit::TextAlign::Left
                    }
                }
                Alignment::End => {
                    if layout.direction() == crate::Direction::RightToLeft {
                        accesskit::TextAlign::Left
                    } else {
                        accesskit::TextAlign::Right
                    }
                }
                Alignment::Left => accesskit::TextAlign::Left,
                Alignment::Right => accesskit::TextAlign::Right,
                Alignment::Center => accesskit::TextAlign::Center,
                Alignment::Justify => accesskit::TextAlign::Justify,
            });
        }
        spans.push(AccessSpan {
            range,
            empty_line: descriptor.empty_line,
            id,
            hard_break: text.ends_with('\n') || text.ends_with('\r'),
        });
        nodes.push((id, node));
    }
    // This is off-side candidate state. No IDs or bindings are published on any refusal.
    for pair in line_nodes.windows(2) {
        if pair[0].0 == pair[1].0 {
            let previous = pair[0].1;
            let next = pair[1].1;
            let previous_id = nodes[previous].0;
            let next_id = nodes[next].0;
            nodes[previous].1.set_next_on_line(next_id);
            nodes[next].1.set_previous_on_line(previous_id);
        }
    }
    let mut parent_node = Node::new(Role::Paragraph);
    if let Projection::Placed(layout, origin) = projection {
        parent_node.set_bounds(host_rect(
            Rect::new(
                0.0,
                0.0,
                layout.metrics().size().width,
                layout.metrics().size().height,
            ),
            origin,
        )?);
    }
    parent_node.set_children(spans.iter().map(|span| span.id).collect::<Vec<_>>());
    nodes.push((parent, parent_node));
    *state = Accessibility {
        binding: Some(projection.binding()),
        spans,
        next_id,
    };
    Ok(TreeUpdate {
        nodes,
        tree: None,
        tree_id: accesskit::TreeId::ROOT,
        focus,
    })
}

impl Layout {
    /// Builds a complete host update and atomically replaces its source bindings.
    pub fn accessibility_update(
        &self,
        state: &mut Accessibility,
        parent: NodeId,
        focus: NodeId,
        origin: Point,
    ) -> Result<TreeUpdate> {
        update(Projection::Placed(self, origin), state, parent, focus)
    }

    pub fn access_position(
        &self,
        state: &Accessibility,
        cursor: Cursor,
    ) -> Result<Option<TextPosition>> {
        self.validate_accessibility(state)?;
        super::range::validate_index(self.source().text(), cursor.index(), "accessibility cursor")?;
        Ok(access_stop(
            state,
            self.inner
                .interaction
                .caret(cursor.index(), cursor.affinity()),
            cursor.affinity(),
        ))
    }

    pub fn access_selection(
        &self,
        state: &Accessibility,
        selection: Selection,
    ) -> Result<Option<TextSelection>> {
        self.validate_accessibility(state)?;
        super::range::validate(self.source().text(), selection.range())?;
        let (anchor, focus) = if selection.is_collapsed() {
            let cursor = selection.focus();
            let point = self.access_position(state, cursor)?;
            (point, point)
        } else {
            let range = self.inner.interaction.selection_range(selection.range());
            selection_positions(state, selection, range)
        };
        Ok(anchor
            .zip(focus)
            .map(|(anchor, focus)| TextSelection { anchor, focus }))
    }

    fn validate_accessibility(&self, state: &Accessibility) -> Result<()> {
        validate_binding(state, &self.inner.prepared.analyzed, Some(&self.identity))
    }
}

fn issue_id(next: &mut Option<u64>, parent: NodeId, focus: NodeId) -> Result<NodeId> {
    loop {
        let value = next.ok_or_else(|| ax_error(ErrorDetail::AccessibilityIdentityExhausted))?;
        *next = value.checked_add(1);
        let id = NodeId(value);
        if id != parent && id != focus {
            return Ok(id);
        }
    }
}

fn ax_error(detail: ErrorDetail) -> Error {
    Error::new(
        ErrorCode::UnsupportedFeature,
        "accessibility projection is unavailable",
    )
    .with_detail(detail)
}

fn union(a: Rect, b: Rect) -> Rect {
    let x = a.origin.x.min(b.origin.x);
    let y = a.origin.y.min(b.origin.y);
    Rect::new(
        x,
        y,
        (a.origin.x + a.size.width).max(b.origin.x + b.size.width) - x,
        (a.origin.y + a.size.height).max(b.origin.y + b.size.height) - y,
    )
}

fn original_host_text(text: &Text) -> Result<String> {
    let mut result = String::new();
    let mut index = 0;
    for decoded in char::decode_utf16(text.code_units().iter().copied()) {
        let ch = decoded.map_err(|error| {
            ax_error(ErrorDetail::AccessibilityIllFormedSource {
                index,
                unit: error.unpaired_surrogate(),
            })
        })?;
        result.push(ch);
        index += ch.len_utf16();
    }
    Ok(result)
}

fn host_width(value: f64) -> Result<f32> {
    use crate::{TextNumericError as Numeric, TextNumericPhase as Phase};
    let converted = value as f32;
    let category = if !value.is_finite() {
        Some(Numeric::NonFinite)
    } else if !converted.is_finite() {
        Some(Numeric::OutOfRange)
    } else if value > 0.0 && converted <= 0.0 {
        Some(Numeric::PositiveLost)
    } else if value != 0.0 && converted == 0.0 {
        Some(Numeric::NonZeroLost)
    } else {
        None
    };
    if let Some(category) = category {
        return Err(crate::engine::text_numeric_error(
            "accessibility character width",
            Phase::Accessibility,
            category,
            value,
        ));
    }
    Ok(converted)
}

fn host_rect(rect: Rect, origin: Point) -> Result<accesskit::Rect> {
    let values = [
        origin.x + rect.origin.x,
        origin.y + rect.origin.y,
        origin.x + rect.origin.x + rect.size.width,
        origin.y + rect.origin.y + rect.size.height,
    ];
    for value in values {
        if !value.is_finite() {
            return Err(crate::engine::text_numeric_error(
                "accessibility bounds",
                crate::TextNumericPhase::Accessibility,
                crate::TextNumericError::NonFinite,
                value,
            ));
        }
    }
    Ok(accesskit::Rect {
        x0: values[0],
        y0: values[1],
        x1: values[2],
        y1: values[3],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FontCatalogueMode, Id, InlineBox, InlineBoxKind, Options, Size, Source, Style, System,
        SystemOptions,
    };
    use std::collections::BTreeSet;

    #[test]
    fn host_text_uses_exact_original_scalars_and_refuses_isolated_units() {
        let text = Text::from_code_units(vec![0xd83d, 0xde00, 0xfffd, 0x00a6]);
        assert_eq!(original_host_text(&text).unwrap(), "😀\u{fffd}\u{a6}");
        for (units, index, unit) in [
            (vec![0xd83d, 0xde00, 0xd800], 2, 0xd800),
            (vec![0x41, 0xdc00], 1, 0xdc00),
        ] {
            assert_eq!(
                original_host_text(&Text::from_code_units(units))
                    .unwrap_err()
                    .detail(),
                Some(&ErrorDetail::AccessibilityIllFormedSource { index, unit })
            );
        }
    }

    #[test]
    fn host_numeric_categories_keep_logical_precision_and_refuse_lost_width() {
        let rect = host_rect(
            Rect::new(0.25, 0.5, 1.0, 2.0),
            Point::new(1_099_511_627_776.0, 1_099_511_627_776.0),
        )
        .unwrap();
        assert_eq!(rect.x0, 1_099_511_627_776.25);
        assert_eq!(rect.x1, 1_099_511_627_777.25);
        assert_eq!(rect.y0, 1_099_511_627_776.5);
        assert_eq!(rect.y1, 1_099_511_627_778.5);
        assert_eq!(host_width(0.0).unwrap(), 0.0);
        assert!(matches!(
            host_width(f64::MIN_POSITIVE).unwrap_err().detail(),
            Some(ErrorDetail::InvalidTextNumeric {
                phase: crate::TextNumericPhase::Accessibility,
                category: crate::TextNumericError::PositiveLost,
                ..
            })
        ));
        assert!(host_rect(Rect::new(f64::MAX, 0.0, f64::MAX, 1.0), Point::default()).is_err());
    }

    #[test]
    fn distinct_box_only_lines_keep_distinct_ids_and_reuse_each_actual_focus() {
        let mut source = Source::new("");
        for id in 1..=3 {
            source.inline_box(InlineBox::new(
                Id::from_u64(id),
                InlineBoxKind::InFlow,
                0,
                Size::new(20.0, 20.0),
            ));
        }
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .expect("no glyph font needed for boxes");
        let layout = system
            .layout(
                source,
                Style::default(),
                Options {
                    width: Some(20.0),
                    ..Options::default()
                },
            )
            .expect("real box-only line layout");
        let lines = layout.lines();
        assert!(
            lines.len() > 1,
            "fixture needs distinct actual line occurrences"
        );
        assert!(lines.iter().all(|line| line.range() == Range::new(0, 0)));
        let mut state = Accessibility::new(10);
        let parent = NodeId(1000);
        let update = layout
            .accessibility_update(&mut state, parent, parent, Point::default())
            .expect("each actual empty line is represented");
        let children: Vec<_> = update
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::TextRun)
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(children.len(), lines.len());
        assert_eq!(
            children.iter().copied().collect::<BTreeSet<_>>().len(),
            lines.len()
        );
        for focus in &children {
            let repeat = layout
                .accessibility_update(&mut state, parent, *focus, Point::default())
                .expect("exact retained empty-line child focus");
            assert_eq!(repeat.focus, *focus);
            assert_eq!(
                repeat
                    .nodes
                    .iter()
                    .filter(|(_, node)| node.role() == Role::TextRun)
                    .map(|(id, _)| *id)
                    .collect::<Vec<_>>(),
                children
            );
        }
        let mut exhausted = Accessibility::new(u64::MAX);
        let error = layout
            .accessibility_update(&mut exhausted, parent, parent, Point::default())
            .expect_err("one remaining ID cannot cover multiple actual lines");
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::AccessibilityIdentityExhausted)
        );
        assert_eq!(
            layout
                .access_position(&exhausted, Cursor::new(0, Affinity::After))
                .expect_err("refusal published no mapping")
                .detail(),
            Some(&ErrorDetail::AccessibilityBindingMismatch)
        );
    }

    #[test]
    fn locale_index_preserves_later_declarations_and_exclusive_span_ends() {
        let mut source = Source::new("abcdef");
        source.span(
            Range::new(0, 5),
            Style {
                locale: Some("fr".into()),
                ..Style::default()
            },
        );
        source.span(
            Range::new(2, 4),
            Style {
                locale: Some("de".into()),
                ..Style::default()
            },
        );
        source.span(
            Range::new(3, 3),
            Style {
                locale: Some("it".into()),
                ..Style::default()
            },
        );
        let source = crate::ValidatedSource::try_from(source).unwrap();
        let default = crate::ValidatedStyle::try_from(Style {
            locale: Some("en".into()),
            ..Style::default()
        })
        .unwrap();
        let styles = crate::engine::resolve::analyze_styles(&source, &default);
        let index = LocaleIndex::new(&styles);
        assert_eq!(index.uniform(Range::new(0, 2)).as_deref(), Some("fr"));
        assert_eq!(index.uniform(Range::new(2, 4)).as_deref(), Some("de"));
        assert_eq!(index.uniform(Range::new(4, 5)).as_deref(), Some("fr"));
        assert_eq!(index.uniform(Range::new(5, 6)).as_deref(), Some("en"));
        assert_eq!(index.uniform(Range::new(1, 5)), None);
    }
}
