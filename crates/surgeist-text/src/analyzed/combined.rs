//! Borrowed completed source-cell observations. These confer neither font nor
//! placement authority and remain available when cell preparation is unsupported.
use crate::engine::AnalyzedCore;
use crate::protocol::paragraph::FlowOriginKey;
use crate::transform::{combined::CombinedOutput, relation::OriginalPiece};
use crate::{
    CapturedEventPosition, CapturedParagraph, CapturedScope, CompleteTextNodeContribution,
    ParagraphBaseDirection, ParagraphOriginKind, Range, Source, TextCombineUpright,
};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct AnalyzedCombinedCell<'a> {
    pub(crate) core: &'a AnalyzedCore,
    pub(crate) output: &'a CombinedOutput,
    pub(crate) index: usize,
}
impl<'a> AnalyzedCombinedCell<'a> {
    fn group(&self) -> usize {
        self.output.bindings[self.index].group
    }
    #[must_use]
    pub fn scope(&self) -> CapturedScope {
        let topology = self
            .core
            .paragraph
            .scope_topology
            .as_ref()
            .expect("captured cell");
        CapturedScope {
            capture: Arc::clone(&topology.capture),
            index: self.output.plan.groups[self.group()].scope,
        }
    }
    #[must_use]
    pub fn parent_scope(&self) -> Option<CapturedScope> {
        let topology = self
            .core
            .paragraph
            .scope_topology
            .as_ref()
            .expect("captured cell");
        self.output.plan.groups[self.group()]
            .parent
            .map(|index| CapturedScope {
                capture: Arc::clone(&topology.capture),
                index,
            })
    }
    #[must_use]
    pub fn mode(&self) -> TextCombineUpright {
        self.output.plan.groups[self.group()].mode
    }
    #[must_use]
    pub fn inner_direction(&self) -> ParagraphBaseDirection {
        self.output.plan.groups[self.group()].inner_direction
    }
    /// Actual outer source-cell level, independent of painted inner atoms.
    #[must_use]
    pub fn outer_bidi_level(&self) -> crate::BidiLevel {
        crate::BidiLevel::try_new(self.core.analysis.cell_levels[self.index])
            .expect("resolved source-cell level")
    }
    /// Canonical non-width EGCs before fixed-partition replay and inner edges.
    #[must_use]
    pub fn selection_unit_count(&self) -> usize {
        self.output.plan.groups[self.group()].units.len()
    }
    /// Canonical EGCs of the completed used payload, after inner exclusions.
    #[must_use]
    pub fn used_unit_count(&self) -> usize {
        self.output.plan.used[self.group()].units.len()
    }
    /// Actual semantic used code points after the source width rule. Interior
    /// zero SPACE is included; edge/control exclusions are not included.
    pub fn used_code_points(&self) -> impl ExactSizeIterator<Item = u32> + 'a {
        let plan = &self.output.plan;
        plan.used[self.group()]
            .parts
            .iter()
            .map(|&part| plan.parts[part].normal)
    }
    /// Actual completed presentation emissions. Unlike used_code_points,
    /// this excludes zero SPACE; it makes no font/glyph/provider claim.
    pub fn presentation_code_points(&self) -> impl ExactSizeIterator<Item = u32> + 'a {
        let plan = &self.output.plan;
        plan.used[self.group()]
            .presentation_parts
            .iter()
            .map(|&part| plan.parts[part].normal)
    }
    /// Resolved ordinary inner levels, including selected-line L1, for exactly
    /// the used code-point sequence. These are distinct from outer cell bidi.
    pub fn inner_bidi_levels(&self) -> impl ExactSizeIterator<Item = crate::BidiLevel> + 'a {
        self.output.plan.used[self.group()]
            .levels
            .iter()
            .map(|&level| crate::BidiLevel::try_new(level).expect("resolved source level"))
    }
    /// Query-local original custody, including selected parts removed by inner
    /// edges or emitted as semantic zero SPACE. Neither hulls nor paint prove it.
    pub(crate) fn intersects_original_range(&self, origin: FlowOriginKey, local: Range) -> bool {
        self.source_pieces().any(|part| {
            part.piece.origin == origin
                && part.piece.local.start.max(local.start) < part.piece.local.end.min(local.end)
        })
    }

    /// Exact original pieces, retaining repeated expansion ancestry. Generated
    /// separators have no invented authored piece. Suppressed source parts are
    /// retained and identify their final used disposition independently.
    pub fn source_pieces(
        &self,
    ) -> impl Iterator<Item = AnalyzedCombinedSourcePiece<'a>> + 'a + use<'a> {
        let plan = &self.output.plan;
        let core = self.core;
        plan.groups[self.group()]
            .parts
            .clone()
            .flat_map(move |part| {
                plan.parts[part]
                    .pieces
                    .iter()
                    .map(move |piece| AnalyzedCombinedSourcePiece {
                        core,
                        piece,
                        used: plan.retained[part],
                        presentation: plan.visible[part],
                        emission: plan.parts[part].emission,
                    })
            })
    }
}
impl std::fmt::Debug for AnalyzedCombinedCell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyzedCombinedCell")
            .field("scope", &self.scope())
            .field("selection_units", &self.selection_unit_count())
            .field("used_units", &self.used_unit_count())
            .finish()
    }
}

/// Borrowed original occurrence/resource correspondence of a completed cell.
/// Equal ranges or resource identities never merge separate captured originals.
#[derive(Clone, Copy)]
pub struct AnalyzedCombinedSourcePiece<'a> {
    core: &'a AnalyzedCore,
    piece: &'a OriginalPiece,
    used: bool,
    presentation: bool,
    emission: Option<crate::transform::relation::Emission>,
}
impl AnalyzedCombinedSourcePiece<'_> {
    fn original(&self) -> &crate::protocol::paragraph::OriginData {
        let local = self
            .core
            .paragraph
            .origin_map
            .view_origin(self.piece.origin)
            .expect("admitted complete cell member");
        &self.core.paragraph.origins[local.get()]
    }
    #[must_use]
    pub fn source(&self) -> &Source {
        self.original().original(&self.core.processed)
    }
    #[must_use]
    pub fn local(&self) -> Range {
        self.piece.local
    }
    #[must_use]
    pub fn aggregate(&self) -> Range {
        self.piece.aggregate
    }
    #[must_use]
    pub fn kind(&self) -> ParagraphOriginKind {
        self.original().kind
    }
    #[must_use]
    pub fn complete_text_node(&self) -> Option<&CompleteTextNodeContribution> {
        self.original().text_node.as_ref()
    }
    #[must_use]
    pub fn is_used(&self) -> bool {
        self.used
    }
    /// True only when this source part owns a completed presentation scalar.
    /// Zero SPACE can be semantically used while this remains false.
    #[must_use]
    pub fn has_presentation_emission(&self) -> bool {
        self.presentation
    }
    /// None denotes a retained lexical SPACE, not an invented transform input.
    #[must_use]
    pub fn emission_subposition(&self) -> Option<usize> {
        self.emission.map(|emission| emission.subposition)
    }
    #[must_use]
    pub fn origin_position(&self) -> CapturedEventPosition {
        let topology = self
            .core
            .paragraph
            .scope_topology
            .as_ref()
            .expect("captured source cell");
        CapturedEventPosition {
            capture: Arc::clone(&topology.capture),
            index: self
                .piece
                .origin
                .captured_event()
                .expect("captured original"),
        }
    }
    #[must_use]
    pub fn paragraph(&self) -> CapturedParagraph {
        let position = self.origin_position();
        CapturedParagraph {
            index: position.capture.events[position.index]
                .paragraph
                .expect("actual original owner"),
            capture: position.capture,
        }
    }
}
impl std::fmt::Debug for AnalyzedCombinedSourcePiece<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyzedCombinedSourcePiece")
            .field("origin", &self.origin_position())
            .field("local", &self.local())
            .field("aggregate", &self.aggregate())
            .field("used", &self.used)
            .finish()
    }
}
