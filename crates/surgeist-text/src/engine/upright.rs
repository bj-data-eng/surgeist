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
        units: &'a [analysis::upright::UprightUnit],
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
) -> Result<Vec<analysis::upright::UprightUnit>> {
    if orientation != LineOrientation::Vertical {
        return Ok(Vec::new());
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
    analysis::upright::units(projection, analysis, &analyzed.styles, &barriers, |atoms| {
        provenance.content(projection, atoms)
    })
}

pub(crate) fn original_profile(
    orientation: LineOrientation,
    units: &[analysis::upright::UprightUnit],
) -> ShapingProfile<'_> {
    match orientation {
        LineOrientation::Horizontal => ShapingProfile::Horizontal,
        LineOrientation::Sideways => ShapingProfile::Sideways,
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
        LineOrientation::Sideways => ShapingProfile::Sideways,
        LineOrientation::Vertical => ShapingProfile::Upright {
            units: &presentation.upright_units,
            provenance: UprightProvenance::Selected {
                original: &prepared.analyzed.projection,
                originals: &presentation.originals,
                units: &presentation.units,
            },
        },
    }
}

impl ShapingInput<'_> {
    pub(crate) fn is_upright(&self) -> bool {
        matches!(self.profile, ShapingProfile::Upright { .. })
    }
    pub(crate) fn upright_units(&self) -> &[analysis::upright::UprightUnit] {
        match &self.profile {
            ShapingProfile::Upright { units, .. } => units,
            ShapingProfile::Horizontal | ShapingProfile::Sideways => &[],
        }
    }
    pub(crate) fn upright_content(&self, atoms: AtomRange<usize>) -> UprightContent {
        let ShapingProfile::Upright { provenance, .. } = &self.profile else {
            unreachable!("only an upright invocation requests upright diagnostics")
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

fn admit_decorations(style: &AnalyzedStyleRun) -> Result<()> {
    let authored = style.style.authored();
    for (decoration, kind) in [
        (authored.underline, DecorationKind::Underline),
        (authored.strikethrough, DecorationKind::Strikethrough),
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
        .with_detail(ErrorDetail::UprightDecoration {
            owner: style_owner(style.owner),
            kind,
            role,
        }));
    }
    Ok(())
}
