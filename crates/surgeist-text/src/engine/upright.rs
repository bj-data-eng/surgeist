//! Authenticated composition profile and actual upright invocation provenance.

use std::{collections::BTreeSet, ops::Range as AtomRange};

use super::{AnalyzedData, PreparedData, ShapingInput, analysis, resolve::AnalyzedStyleRun};
use crate::{
    DecorationKind, DecorationOffset, DecorationThickness, Error, ErrorCode, ErrorDetail,
    LineOrientation, Range, Result, TextStyleOwner, UprightContent, UprightContentPiece,
    UprightDecorationRole, UprightPresentation,
    projection::SourceProjection,
    protocol::prepared::{PresentationUnit, PresentationUnitKind, SelectedPresentation},
};

pub(crate) enum ShapingProfile<'a> {
    Horizontal,
    Sideways,
    Upright {
        units: &'a [analysis::upright::VerticalUnit],
        provenance: UprightProvenance<'a>,
    },
    Mixed {
        owners: &'a [AnalyzedStyleRun],
        units: &'a [analysis::upright::VerticalUnit],
        provenance: UprightProvenance<'a>,
    },
}

pub(crate) enum UprightProvenance<'a> {
    Original,
    Alternative {
        original: &'a SourceProjection,
        originals: &'a [Option<usize>],
        inserted: AtomRange<usize>,
        alternative: usize,
        insertion_anchor: usize,
    },
    Selected {
        original: &'a SourceProjection,
        originals: &'a [Option<usize>],
        units: &'a [PresentationUnit],
    },
}

impl UprightProvenance<'_> {
    pub(crate) fn content(
        &self,
        projection: &SourceProjection,
        atoms: AtomRange<usize>,
    ) -> UprightContent {
        let mut pieces = Vec::new();
        let mut previous_row = None;
        for atom in atoms {
            let original = match self {
                Self::Original => Some((projection, atom)),
                Self::Alternative {
                    original,
                    originals,
                    ..
                }
                | Self::Selected {
                    original,
                    originals,
                    ..
                } => originals[atom].map(|index| (*original, index)),
            };
            if let Some((projection, index)) = original {
                if let Some(map) = &projection.transform
                    && let Some(row) = map.original_row_for_atom(index)
                {
                    // Repeated transform emissions retain one correspondence
                    // within this contiguous original occurrence, not a hull.
                    if previous_row != Some(row) {
                        for piece in map.original_pieces(row) {
                            push_piece(
                                &mut pieces,
                                UprightContentPiece::Source {
                                    range: piece.aggregate,
                                },
                            );
                        }
                    }
                    previous_row = Some(row);
                    continue;
                }
                push_piece(
                    &mut pieces,
                    UprightContentPiece::Source {
                        range: projection.atoms[index].source,
                    },
                );
            } else {
                previous_row = None;
                let (alternative, insertion_anchor, range) = match self {
                    Self::Alternative {
                        inserted,
                        alternative,
                        insertion_anchor,
                        ..
                    } => {
                        let start = projection.atoms[inserted.start].presentation.start;
                        let range = projection.atoms[atom].presentation;
                        (
                            *alternative,
                            *insertion_anchor,
                            Range::new(range.start - start, range.end - start),
                        )
                    }
                    Self::Selected { units, .. } => {
                        let unit = units
                            .iter()
                            .find(|unit| unit.atoms.contains(&atom))
                            .expect("selected atom retains its semantic unit");
                        let PresentationUnitKind::Replacement { alternative, local } = unit.kind
                        else {
                            unreachable!("source-free selected atom retains its indication custody")
                        };
                        let range = projection.atoms[atom].presentation;
                        (
                            alternative,
                            unit.source.start,
                            Range::new(
                                local.start + range.start - unit.presentation.start,
                                local.start + range.end - unit.presentation.start,
                            ),
                        )
                    }
                    Self::Original => unreachable!("original projection has no indication atoms"),
                };
                push_piece(
                    &mut pieces,
                    UprightContentPiece::Indication {
                        alternative,
                        range,
                        insertion_anchor,
                    },
                );
            }
        }
        UprightContent::Presentation(UprightPresentation::new(pieces))
    }
}

fn push_piece(pieces: &mut Vec<UprightContentPiece>, next: UprightContentPiece) {
    match (pieces.last_mut(), &next) {
        (
            Some(UprightContentPiece::Source { range: previous }),
            UprightContentPiece::Source { range },
        ) if previous.end == range.start && !previous.is_empty() && !range.is_empty() => {
            previous.end = range.end;
            return;
        }
        (
            Some(UprightContentPiece::Indication {
                alternative: a,
                range: previous,
                insertion_anchor: x,
            }),
            UprightContentPiece::Indication {
                alternative: b,
                range,
                insertion_anchor: y,
            },
        ) if a == b && x == y && previous.end == range.start => {
            previous.end = range.end;
            return;
        }
        _ => {}
    }
    pieces.push(next);
}

pub(crate) fn units(
    analyzed: &AnalyzedData,
    projection: &SourceProjection,
    analysis: &analysis::Analysis,
    provenance: &UprightProvenance<'_>,
    orientation: LineOrientation,
) -> Result<Vec<analysis::upright::VerticalUnit>> {
    if orientation != LineOrientation::Vertical {
        return Ok(Vec::new());
    }
    if analyzed.typography == crate::AnalysisTypography::Mixed {
        let barriers = projection.barrier_atoms.iter().copied().collect();
        return analysis::upright::mixed_units(
            projection,
            analysis,
            &analyzed.styles,
            &barriers,
            |atoms| provenance.content(projection, atoms),
        );
    }
    for index in analysis
        .infos
        .iter()
        .map(|(_, style)| *style)
        .collect::<BTreeSet<_>>()
    {
        admit_decorations(&analyzed.styles[index])?;
    }
    let barriers = projection.barrier_atoms.iter().copied().collect();
    let units =
        analysis::upright::units(projection, analysis, &analyzed.styles, &barriers, |atoms| {
            provenance.content(projection, atoms)
        })?;
    let mut result = Vec::new();
    result
        .try_reserve(units.len())
        .map_err(|_| analysis::capacity())?;
    result.extend(
        units
            .into_iter()
            .map(analysis::upright::VerticalUnit::Upright),
    );
    Ok(result)
}

pub(crate) fn original_profile<'a>(
    orientation: LineOrientation,
    units: &'a [analysis::upright::VerticalUnit],
    typography: crate::AnalysisTypography,
    owners: &'a [AnalyzedStyleRun],
) -> ShapingProfile<'a> {
    match orientation {
        LineOrientation::Horizontal => ShapingProfile::Horizontal,
        LineOrientation::SidewaysClockwise => ShapingProfile::Sideways,
        LineOrientation::Vertical if typography == crate::AnalysisTypography::Mixed => {
            ShapingProfile::Mixed {
                units,
                owners,
                provenance: UprightProvenance::Original,
            }
        }
        LineOrientation::Vertical => ShapingProfile::Upright {
            units,
            provenance: UprightProvenance::Original,
        },
    }
}

pub(crate) fn selected_profile<'a>(
    prepared: &'a PreparedData,
    presentation: &'a SelectedPresentation,
) -> ShapingProfile<'a> {
    match prepared.options.orientation() {
        LineOrientation::Horizontal => ShapingProfile::Horizontal,
        LineOrientation::SidewaysClockwise => ShapingProfile::Sideways,
        LineOrientation::Vertical
            if prepared.analyzed.typography == crate::AnalysisTypography::Mixed =>
        {
            ShapingProfile::Mixed {
                units: &presentation.vertical_units,
                owners: &prepared.analyzed.styles,
                provenance: UprightProvenance::Selected {
                    original: &prepared.analyzed.projection,
                    originals: &presentation.originals,
                    units: &presentation.units,
                },
            }
        }
        LineOrientation::Vertical => ShapingProfile::Upright {
            units: &presentation.vertical_units,
            provenance: UprightProvenance::Selected {
                original: &prepared.analyzed.projection,
                originals: &presentation.originals,
                units: &presentation.units,
            },
        },
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum RunRole {
    Horizontal,
    Sideways,
    Upright,
}

impl ShapingInput<'_> {
    pub(crate) fn is_mixed(&self) -> bool {
        matches!(self.profile, ShapingProfile::Mixed { .. })
    }
    pub(crate) fn role(&self, atom: usize) -> RunRole {
        match &self.profile {
            ShapingProfile::Horizontal => RunRole::Horizontal,
            ShapingProfile::Sideways => RunRole::Sideways,
            ShapingProfile::Upright { .. } => RunRole::Upright,
            ShapingProfile::Mixed { units, .. } => units
                .get(units.partition_point(|unit| unit.atoms().end <= atom))
                .filter(|unit| unit.atoms().contains(&atom))
                .map_or(RunRole::Upright, |unit| {
                    if unit.upright().is_some() {
                        RunRole::Upright
                    } else {
                        RunRole::Sideways
                    }
                }),
        }
    }
    pub(crate) fn vertical_units(&self) -> &[analysis::upright::VerticalUnit] {
        match &self.profile {
            ShapingProfile::Upright { units, .. } | ShapingProfile::Mixed { units, .. } => units,
            ShapingProfile::Horizontal | ShapingProfile::Sideways => &[],
        }
    }
    pub(crate) fn upright_content(&self, atoms: AtomRange<usize>) -> UprightContent {
        let provenance = match &self.profile {
            ShapingProfile::Upright { provenance, .. }
            | ShapingProfile::Mixed { provenance, .. } => provenance,
            _ => unreachable!("only an upright invocation requests upright diagnostics"),
        };
        provenance.content(self.projection, atoms)
    }
}

pub(crate) fn style_owner(owner: super::resolve::AnalyzedStyleOwner) -> TextStyleOwner {
    match owner {
        super::resolve::AnalyzedStyleOwner::Source(range) => TextStyleOwner::Source { range },
        super::resolve::AnalyzedStyleOwner::Control(occurrence) => {
            TextStyleOwner::Control { occurrence }
        }
        super::resolve::AnalyzedStyleOwner::Scope(scope) => TextStyleOwner::Scope { scope },
    }
}

pub(crate) fn admit_mixed_decoration(input: &ShapingInput<'_>, index: usize) -> Result<()> {
    let ShapingProfile::Mixed { owners, .. } = &input.profile else {
        return Ok(());
    };
    let owner = style_owner(owners[index].owner);
    let style = &input.styles[index].style;
    admit_resolved_decoration(style, owner)
}

pub(crate) fn admit_resolved_decoration(
    style: &super::resolve::ResolvedStyle,
    owner: TextStyleOwner,
) -> Result<()> {
    admit_decoration_values(style.underline, style.strikethrough, owner)
}

fn admit_decorations(style: &AnalyzedStyleRun) -> Result<()> {
    let authored = style.style.authored();
    admit_decoration_values(
        authored.underline,
        authored.strikethrough,
        style_owner(style.owner),
    )
}

fn admit_decoration_values(
    underline: crate::Decoration,
    strikethrough: crate::Decoration,
    owner: TextStyleOwner,
) -> Result<()> {
    for (decoration, kind) in [
        (underline, DecorationKind::Underline),
        (strikethrough, DecorationKind::Strikethrough),
    ] {
        if !decoration.enabled() {
            continue;
        }
        let offset = matches!(decoration.offset(), DecorationOffset::Auto);
        let thickness = matches!(decoration.thickness(), DecorationThickness::Auto);
        let role = match (offset, thickness) {
            (true, true) => UprightDecorationRole::OffsetAndThickness,
            (true, false) => UprightDecorationRole::Offset,
            (false, true) => UprightDecorationRole::Thickness,
            (false, false) => continue,
        };
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "automatic upright decoration metrics are unsupported",
        )
        .with_detail(ErrorDetail::UprightDecoration { owner, kind, role }));
    }
    Ok(())
}
