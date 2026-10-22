// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Metadata and line-link projection adapted from Parley 0.9.0 layout/accessibility.rs.
// Surgeist owns source-unit grouping, checked identities, geometry and atomic publication.
// Exact origin and modifications: ../docs/accessibility-provenance.md.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use accesskit::{Node, NodeId, Role, TextDirection, TextPosition, TextSelection, TreeUpdate};
use skrifa::{
    FontRef,
    raw::{TableProvider, types::NameId},
};

use crate::{
    Affinity, Alignment, Brush, Cursor, Error, ErrorCode, ErrorDetail, Layout, Point, Range, Rect,
    Result, Selection, Text,
};

/// Accessibility bindings for one immutable layout and its exact source mapping.
#[derive(Clone)]
pub struct Accessibility {
    layout: Option<Arc<()>>,
    source: Option<Text>,
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
            layout: None,
            source: None,
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
    fn from_run(run: &parley::Run<'_, Brush>, locale: Option<String>) -> Self {
        let font = run.font();
        let family = FontRef::from_index(font.data.as_ref(), font.index)
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
            size: run.font_size(),
            weight: run.font_attrs().weight.value(),
            italic: matches!(run.font_attrs().style, fontique::FontStyle::Italic),
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
    fn new(source: &crate::Source, default: &crate::Style) -> Self {
        let mut edges = Vec::new();
        for (ordinal, span) in source.spans().iter().enumerate() {
            if !span.range().is_empty() {
                edges.push((span.range().start, ordinal, true));
                edges.push((span.range().end, ordinal, false));
            }
        }
        edges.sort_unstable();
        let mut active = BTreeSet::new();
        let mut runs = Vec::<LocaleRun>::new();
        let mut previous = 0;
        let mut locale = default.locale.clone();
        let mut edges = edges.into_iter().peekable();
        while let Some(&(position, _, _)) = edges.peek() {
            Self::push(&mut runs, previous, position, locale);
            while edges.peek().is_some_and(|edge| edge.0 == position) {
                let (_, ordinal, entering) = edges.next().expect("peeked locale edge");
                if entering {
                    active.insert(ordinal);
                } else {
                    active.remove(&ordinal);
                }
            }
            locale = active.last().map_or_else(
                || default.locale.clone(),
                |ordinal| source.spans()[*ordinal].style().locale.clone(),
            );
            previous = position;
        }
        Self::push(&mut runs, previous, source.text().len(), locale);
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

impl Layout {
    /// Builds a complete host update and atomically replaces its source bindings.
    pub fn accessibility_update(
        &self,
        state: &mut Accessibility,
        parent: NodeId,
        focus: NodeId,
        origin: Point,
    ) -> Result<TreeUpdate> {
        let locales = LocaleIndex::new(&self.source, &self.default_style);
        let lines = self.lines();
        let mut fragments = Vec::new();
        for (line_index, line) in self.inner.lines().enumerate() {
            for run in line.runs() {
                for cluster in run.clusters() {
                    let metrics = line.metrics();
                    fragments.push(Fragment {
                        range: self.projection.range(cluster.text_range()),
                        bounds: Rect::new(
                            cluster.visual_offset().unwrap_or(metrics.offset),
                            metrics.block_min_coord,
                            cluster.advance(),
                            metrics.block_max_coord - metrics.block_min_coord,
                        ),
                        line: line_index,
                        rtl: cluster.is_rtl(),
                        word: cluster.is_word_boundary() && !cluster.is_space_or_nbsp(),
                        metadata: Metadata::from_run(
                            &run,
                            locales.uniform(self.projection.range(cluster.text_range())),
                        ),
                    });
                }
            }
        }
        let mut descriptors: Vec<_> = self
            .projection
            .selectable_ranges()
            .map(|range| NodeDescriptor {
                range,
                empty_line: None,
            })
            .collect();
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
            .layout
            .as_ref()
            .is_some_and(|identity| Arc::ptr_eq(identity, &self.identity));
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
            let start = self.projection.byte(range.start).0;
            let end = self.projection.byte(range.end).0;
            let text = &self.projection.text[start..end];
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
            node.set_value(text.to_owned());
            node.set_character_lengths(if range.is_empty() {
                Vec::new()
            } else {
                vec![length]
            });
            if let Some(first) = pieces.first() {
                let bounds = pieces
                    .iter()
                    .skip(1)
                    .fold(first.bounds, |bounds, fragment| {
                        union(bounds, fragment.bounds)
                    });
                node.set_bounds(host_rect(bounds, origin));
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
                    node.set_character_widths(vec![bounds.size.width]);
                }
                if one_line {
                    line_nodes.push((first.line, nodes.len()));
                }
            } else if let Some(line) = descriptor.empty_line.and_then(|index| lines.get(index)) {
                node.set_bounds(host_rect(line.bounds(), origin));
                line_nodes.push((line.index(), nodes.len()));
            }
            node.set_text_align(match self.alignment {
                Alignment::Start => {
                    if self.inner.is_rtl() {
                        accesskit::TextAlign::Right
                    } else {
                        accesskit::TextAlign::Left
                    }
                }
                Alignment::End => {
                    if self.inner.is_rtl() {
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
        parent_node.set_bounds(host_rect(
            Rect::new(
                0.0,
                0.0,
                self.metrics().size().width,
                self.metrics().size().height,
            ),
            origin,
        ));
        parent_node.set_children(spans.iter().map(|span| span.id).collect::<Vec<_>>());
        nodes.push((parent, parent_node));
        *state = Accessibility {
            layout: Some(self.identity.clone()),
            source: Some(self.source.text().clone()),
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

    pub fn access_position(
        &self,
        state: &Accessibility,
        cursor: Cursor,
    ) -> Result<Option<TextPosition>> {
        self.validate_accessibility(state)?;
        super::range::validate_index(self.source.text(), cursor.index(), "accessibility cursor")?;
        Ok(self.access_stop(
            state,
            self.projection.caret(cursor.index(), cursor.affinity()),
            cursor.affinity(),
        ))
    }

    pub fn access_selection(
        &self,
        state: &Accessibility,
        selection: Selection,
    ) -> Result<Option<TextSelection>> {
        self.validate_accessibility(state)?;
        super::range::validate(self.source.text(), selection.range())?;
        let (anchor, focus) = if selection.is_collapsed() {
            let cursor = selection.focus();
            let point = self.access_position(state, cursor)?;
            (point, point)
        } else {
            let range = self.projection.selection_range(selection.range());
            let start = self.access_stop(state, range.start, Affinity::After);
            let end = self.access_stop(state, range.end, Affinity::Before);
            if selection.anchor().index() <= selection.focus().index() {
                (start, end)
            } else {
                (end, start)
            }
        };
        Ok(anchor
            .zip(focus)
            .map(|(anchor, focus)| TextSelection { anchor, focus }))
    }

    fn validate_accessibility(&self, state: &Accessibility) -> Result<()> {
        if !state
            .layout
            .as_ref()
            .is_some_and(|identity| Arc::ptr_eq(identity, &self.identity))
            || state.source.as_ref() != Some(self.source.text())
        {
            return Err(ax_error(ErrorDetail::AccessibilityLayoutMismatch));
        }
        Ok(())
    }

    fn access_stop(
        &self,
        state: &Accessibility,
        unit: usize,
        affinity: Affinity,
    ) -> Option<TextPosition> {
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

fn host_rect(rect: Rect, origin: Point) -> accesskit::Rect {
    accesskit::Rect {
        x0: f64::from(origin.x + rect.origin.x),
        y0: f64::from(origin.y + rect.origin.y),
        x1: f64::from(origin.x + rect.origin.x + rect.size.width),
        y1: f64::from(origin.y + rect.origin.y + rect.size.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FontCatalogueMode, Id, InlineBox, InlineBoxKind, Options, Size, Source, Style, System,
        SystemOptions,
    };

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
            Some(&ErrorDetail::AccessibilityLayoutMismatch)
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
        let index = LocaleIndex::new(
            &source,
            &Style {
                locale: Some("en".into()),
                ..Style::default()
            },
        );
        assert_eq!(index.uniform(Range::new(0, 2)).as_deref(), Some("fr"));
        assert_eq!(index.uniform(Range::new(2, 4)).as_deref(), Some("de"));
        assert_eq!(index.uniform(Range::new(4, 5)).as_deref(), Some("fr"));
        assert_eq!(index.uniform(Range::new(5, 6)).as_deref(), Some("en"));
        assert_eq!(index.uniform(Range::new(1, 5)), None);
    }
}
