//! Original U+200B expansion over the actual post-Phase-I mixed stream.
use super::relation::TransformMap;
use crate::engine::{
    analysis::wrapping::{Event, LogicalEvent},
    resolve::AnalyzedStyleRun,
};
use crate::projection::SourceProjection;
use crate::protocol::paragraph::{ParagraphAnchorItem, ParagraphControlKind, ParagraphData};
use crate::{
    Error, ErrorCode, ErrorDetail, InlineBoxKind, Result, TextTransformError, WordSpaceTransform,
};

/// None is genuinely transparent; Some records the next actual item's forced role.
fn forced(
    event: &LogicalEvent,
    input: &SourceProjection,
    source: &crate::Source,
    paragraph: &ParagraphData,
) -> Option<bool> {
    match event.kind {
        Event::Text(atom) => Some(crate::engine::analysis::is_hard_separator(
            input.atoms[atom].code_point,
        )),
        Event::Deleted(..) => None,
        Event::Anchor(anchor) => match anchor.item {
            ParagraphAnchorItem::EmptyOrigin(_) => None,
            ParagraphAnchorItem::Atomic(i) => {
                (source.boxes()[i].kind() == InlineBoxKind::InFlow).then_some(false)
            }
            ParagraphAnchorItem::Control(i) => match paragraph.controls[i].kind {
                ParagraphControlKind::InlineBoundaryStart
                | ParagraphControlKind::InlineBoundaryEnd => None,
                ParagraphControlKind::ForcedBreak => Some(true),
                _ => Some(false),
            },
        },
    }
}

pub(super) fn apply(
    mut input: SourceProjection,
    map: &mut TransformMap,
    events: &[LogicalEvent],
    styles: &[AnalyzedStyleRun],
    processed: &crate::ProcessedSource,
    paragraph: &ParagraphData,
) -> Result<SourceProjection> {
    if !map.rows.iter().any(|row| {
        row.code_point == 0x200b
            && styles[row.style].style.authored().word_space_transform != WordSpaceTransform::None
    }) {
        return Ok(input);
    }
    let mut adjacent = Vec::new();
    adjacent.try_reserve_exact(input.atoms.len()).map_err(|_| {
        Error::new(
            ErrorCode::SourceProcessingCapacity,
            "separator adjacency storage unavailable",
        )
        .with_detail(ErrorDetail::TextTransform {
            reason: TextTransformError::Capacity,
        })
    })?;
    adjacent.resize(input.atoms.len(), false);
    let mut before = false;
    for event in events {
        if let Event::Text(atom) = event.kind {
            adjacent[atom] |= before;
        }
        if let Some(is_forced) = forced(event, &input, processed.source(), paragraph) {
            before = is_forced;
        }
    }
    let mut after = false;
    for event in events.iter().rev() {
        if let Event::Text(atom) = event.kind {
            adjacent[atom] |= after;
        }
        if let Some(is_forced) = forced(event, &input, processed.source(), paragraph) {
            after = is_forced;
        }
    }
    let mut changed = false;
    for (row, atom) in input.atoms.iter_mut().enumerate() {
        if atom.code_point != 0x200b || adjacent[row] {
            continue;
        }
        let cp = match styles[map.rows[row].style]
            .style
            .authored()
            .word_space_transform
        {
            WordSpaceTransform::None => continue,
            WordSpaceTransform::Space => {
                map.rows[row].separator_space = true;
                ' '
            }
            WordSpaceTransform::IdeographicSpace => '\u{3000}',
        };
        atom.code_point = u32::from(cp);
        atom.glyph = cp;
        changed = true;
    }
    if !changed {
        return Ok(input);
    }
    let end = input.presentation_end();
    // Both replacements retain one UTF-16 unit and one genuine input row.
    // Rebuild semantic strings/graphemes before any casing decision is made.
    let mut result = SourceProjection::from_atoms_at_barriers(
        input.atoms,
        end,
        input.barriers,
        input.barrier_atoms,
    )?;
    result.anchor_atoms = input.anchor_atoms;
    result.style_rows = input.style_rows;
    Ok(result)
}
