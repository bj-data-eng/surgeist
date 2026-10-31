//! Text-owned visual character units and retained movement constraints.

use super::super::{
    analysis::{Analysis, unit_properties},
    layout::data::{ShapedData, ShapedItem},
    resolve::ResolvedStyleRun,
};
use crate::projection::SourceProjection;
use crate::protocol::{input::PresentationInput, prepared::SelectedPresentation};
use crate::{
    InlineBox, InlineBoxKind, Result, SpacingBoundaryGeometry, SpacingFailureReason,
    TextNumericPhase,
};
use icu_properties::{
    CodePointSetData,
    props::{DefaultIgnorableCodePoint, GeneralCategory, GeneralCategoryGroup, Script},
};
use std::ops::Range;

use super::super::scope_alignment::accounting::{self as account, Ledger};
mod construction;
mod index;
mod input;
mod planning;
mod storage;
pub(crate) use input::Input;
pub(crate) use planning::{PlanRequirements, plan};
pub(crate) use storage::{GapArenaOwnership, cell_gap_indices, visit_shaped_allocations};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum GraphOwnership {
    #[default]
    Native,
    CellScratch,
    CellRetained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Subject {
    Text,
    Atomics,
    Cell(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlanReason {
    ExactBoundary,
    ConditionalEnvelope,
}
#[derive(Clone, Debug)]
pub(crate) struct PlanCoverage {
    pub(crate) atoms: Range<usize>,
    pub(crate) reason: PlanReason,
}

/// Final native DI outcomes that admit no independent input contribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Exclusion {
    Residual,
    Deleted,
}

#[derive(Clone, Debug)]
pub(crate) struct Unit {
    subject: Subject,
    order: (usize, usize),
    pub(crate) atoms: Vec<Range<usize>>,
    pub(crate) atomics: Vec<usize>,
    pub(crate) base: Option<usize>,
    pub(crate) left: f64,
    pub(crate) right: f64,
    qualified: bool,
    cursive: bool,
    justify_cursive: bool,
    justify_qualified: bool,
    symbol: bool,
    resolved_scripts: Vec<Script>,
    scripts: Vec<Script>,
    context_scripts: Vec<Script>,
    /// The canonical Common P/S rule consumes this unit's domain content.
    content_context: bool,
    domain: Option<usize>,
    barrier: bool,
    native_only: bool,
    level: u8,
}

#[derive(Clone, Debug)]
pub(crate) struct Boundary {
    pub(crate) before: usize,
    pub(crate) after: usize,
    pub(crate) authored: f64,
    mean_lost: bool,
    pub(crate) geometry: SpacingBoundaryGeometry,
}

#[derive(Debug, Default)]
pub(crate) struct Graph {
    pub(crate) units: Vec<Unit>,
    pub(crate) boundaries: Vec<Boundary>,
    pub(crate) potential: bool,
    regions: Vec<Region>,
    ownership: GraphOwnership,
    index: index::Index,
}

#[derive(Clone, Debug)]
pub(crate) struct Constraint {
    pub(crate) run: usize,
    pub(crate) components: Vec<usize>,
    pub(crate) required: bool,
    pub(crate) kind: crate::SpacingConstraintKind,
    pub(crate) provenance: Vec<surgeist_shaper::FeatureProvenance>,
}

#[derive(Clone, Debug)]
pub(crate) struct Invocation {
    pub(crate) atoms: Range<usize>,
    pub(crate) run: usize,
    pub(crate) complete: bool,
    pub(crate) path: Option<surgeist_shaper::ComponentPath>,
    pub(crate) components: Vec<Vec<usize>>,
    pub(crate) generated: Vec<Vec<usize>>,
    pub(crate) constraints: Vec<Constraint>,
}

fn supported(script: Script) -> bool {
    matches!(
        script,
        Script::Latin
            | Script::Greek
            | Script::Cyrillic
            | Script::Armenian
            | Script::Hebrew
            | Script::Han
            | Script::Hiragana
            | Script::Katakana
            | Script::Bopomofo
            | Script::Hangul
    ) || cursive(script)
}
fn cursive(script: Script) -> bool {
    matches!(
        script,
        Script::Arabic
            | Script::HanifiRohingya
            | Script::Mandaic
            | Script::Mongolian
            | Script::Nko
            | Script::PhagsPa
            | Script::Syriac
    )
}
fn explicit_family(tag: &str) -> Option<&'static [Script]> {
    Some(match tag {
        "Latn" => &[Script::Latin],
        "Grek" => &[Script::Greek],
        "Cyrl" => &[Script::Cyrillic],
        "Armn" => &[Script::Armenian],
        "Hebr" => &[Script::Hebrew],
        "Hant" | "Hans" | "Hani" | "Hanb" | "Bopo" => &[Script::Han, Script::Bopomofo],
        "Jpan" | "Hrkt" | "Hira" | "Kana" => &[Script::Han, Script::Hiragana, Script::Katakana],
        "Kore" | "Hang" | "Jamo" => &[Script::Hangul, Script::Han],
        "Arab" => &[Script::Arabic],
        "Rohg" => &[Script::HanifiRohingya],
        "Mand" => &[Script::Mandaic],
        "Mong" => &[Script::Mongolian],
        "Nkoo" => &[Script::Nko],
        "Phag" => &[Script::PhagsPa],
        "Syrc" => &[Script::Syriac],
        _ => return None,
    })
}
fn locale_family(locale: &fontique::Language) -> Option<&'static [Script]> {
    let fields =
        1 + usize::from(locale.script().is_some()) + usize::from(locale.region().is_some());
    if locale.as_str().split('-').count() != fields {
        return None;
    }
    if let Some(script) = locale.script() {
        return explicit_family(script);
    }
    explicit_family(match locale.language() {
        "en" => "Latn",
        "el" => "Grek",
        "ru" => "Cyrl",
        "hy" => "Armn",
        "he" => "Hebr",
        "zh" => "Hani",
        "ja" => "Jpan",
        "ko" => "Kore",
        "ar" => "Arab",
        "syr" => "Syrc",
        "nqo" => "Nkoo",
        _ => return None,
    })
}

pub(crate) use construction::build;

/// The resolver owns which units depend on domain content. The caller proves
/// source contributions invariant; an absent proof is conditional, not a new
/// script winner. Counts and actual resolution remain solely in the rule above.
fn conditional_context_consumers(
    units: &[Unit],
    contributions_invariant: &[bool],
    ledger: &mut Ledger,
) -> Result<Vec<usize>> {
    let mut affected = Vec::new();
    for (i, unit) in units.iter().enumerate() {
        account::work(ledger, 1)?;
        if unit.base.is_some() && !contributions_invariant.get(i).copied().unwrap_or(false) {
            let mut found = false;
            for domain in &affected {
                account::work(ledger, 1)?;
                if *domain == unit.domain {
                    found = true;
                    break;
                }
            }
            if !found {
                account::push(ledger, &mut affected, unit.domain)?;
            }
        }
    }
    let mut output = Vec::new();
    for (i, unit) in units.iter().enumerate() {
        account::work(ledger, 1)?;
        if unit.content_context {
            for domain in &affected {
                account::work(ledger, 1)?;
                if *domain == unit.domain {
                    account::push(ledger, &mut output, i)?;
                    break;
                }
            }
        }
    }
    account::drop_vec(ledger, affected)?;
    Ok(output)
}

impl Graph {
    pub(crate) fn demand(&self, ledger: &mut Ledger) -> Result<bool> {
        for b in &self.boundaries {
            account::work(ledger, 1)?;
            if b.authored != 0.0
                || b.mean_lost
                || self.potential && b.geometry != SpacingBoundaryGeometry::ProtectedCursive
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub(crate) fn owners(
        &self,
        shaped: &ShapedData,
        order: &[usize],
        ledger: &mut Ledger,
    ) -> Result<Vec<Option<Owner>>> {
        self.index.owners(shaped, order, self.units.len(), ledger)
    }
    fn crosses(&self, edge: usize, members: &[usize]) -> bool {
        let side = |component| {
            self.units
                .iter()
                .position(|u| u.atoms.iter().any(|a| a.contains(&component)))
                .map(|i| i <= self.boundaries[edge].before)
        };
        members.iter().any(|&i| side(i) == Some(true))
            && members.iter().any(|&i| side(i) == Some(false))
    }
    fn touches(&self, edge: usize, invocation: &Invocation) -> bool {
        let boundary = &self.boundaries[edge];
        [boundary.before, boundary.after].iter().any(|&u| {
            self.units[u]
                .atoms
                .iter()
                .any(|a| a.start < invocation.atoms.end && a.end > invocation.atoms.start)
        })
    }
    pub(crate) fn geometry(
        &self,
        edge: usize,
        invocations: &[Invocation],
        ledger: &mut Ledger,
    ) -> Result<SpacingBoundaryGeometry> {
        let movement = self.movement_geometry(edge, invocations, ledger)?;
        Ok(if movement == SpacingBoundaryGeometry::ProtectedRequired {
            movement
        } else if self.boundaries[edge].geometry == SpacingBoundaryGeometry::ProtectedCursive {
            SpacingBoundaryGeometry::ProtectedCursive
        } else {
            movement
        })
    }
    fn crosses_accounted(
        &self,
        edge: usize,
        members: &[usize],
        ledger: &mut Ledger,
    ) -> Result<bool> {
        let mut before = false;
        let mut after = false;
        for &component in members {
            account::work(ledger, 1)?;
            if let Some(unit) = self.index.atom(component, ledger)? {
                before |= unit <= self.boundaries[edge].before;
                after |= unit > self.boundaries[edge].before;
            }
        }
        Ok(before && after)
    }
    fn touches_accounted(
        &self,
        edge: usize,
        invocation: &Invocation,
        ledger: &mut Ledger,
    ) -> Result<bool> {
        let b = &self.boundaries[edge];
        for u in [b.before, b.after] {
            for a in &self.units[u].atoms {
                account::work(ledger, 1)?;
                if a.start < invocation.atoms.end && a.end > invocation.atoms.start {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    pub(crate) fn movement_geometry(
        &self,
        edge: usize,
        invocations: &[Invocation],
        ledger: &mut Ledger,
    ) -> Result<SpacingBoundaryGeometry> {
        let mut constrained = false;
        for invocation in invocations {
            account::work(ledger, 1)?;
            if !invocation.complete && self.touches_accounted(edge, invocation, ledger)? {
                constrained = true;
            }
            for relation in &invocation.constraints {
                account::work(ledger, 1)?;
                if self.crosses_accounted(edge, &relation.components, ledger)? {
                    if relation.required && relation.kind == crate::SpacingConstraintKind::Ligature
                    {
                        return Ok(SpacingBoundaryGeometry::ProtectedRequired);
                    }
                    constrained = true;
                }
            }
            for components in invocation.components.iter().chain(&invocation.generated) {
                account::work(ledger, 1)?;
                if self.crosses_accounted(edge, components, ledger)? {
                    constrained = true;
                }
            }
        }
        Ok(if constrained {
            SpacingBoundaryGeometry::Constrained
        } else {
            SpacingBoundaryGeometry::Independent
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Owner {
    Cluster(usize),
    Atomic(usize),
    Cell(usize),
}

struct DiagnosticContext {
    pieces: Vec<crate::SpacingPiece>,
    locale: Option<String>,
}
fn cell_context(
    analyzed: &crate::engine::AnalyzedData,
    binding: usize,
    ledger: &mut Ledger,
) -> Result<DiagnosticContext> {
    let input = Input::original(analyzed, &[], 1.0);
    let pieces = input.cell_pieces(binding, ledger)?;
    let locale = input.cell_locale(binding, ledger)?;
    Ok(DiagnosticContext { pieces, locale })
}

pub(crate) fn failure(
    input: PresentationInput<'_>,
    graph: &Graph,
    edge: usize,
    operation: crate::SpacingOperation,
    invocations: &[Invocation],
    ledger: &mut Ledger,
) -> crate::Error {
    diagnostic(
        graph,
        edge,
        invocations,
        crate::SpacingStage::SelectedShaping,
        operation,
        |unit| {
            if let Subject::Cell(binding) = unit.subject {
                return cell_context(&input.prepared.analyzed, binding, ledger);
            }
            let mut pieces = Vec::new();
            for atoms in &unit.atoms {
                pieces.extend(
                    super::slot_pieces(input.prepared, input.presentation, atoms.clone())
                        .into_iter()
                        .map(|p| p.owner),
                );
            }
            pieces.extend(
                unit.atomics
                    .iter()
                    .map(|&occurrence| crate::SpacingPiece::Atomic { occurrence }),
            );
            let locale = unit.base.and_then(|base| {
                input.prepared.styles[input.presentation.analysis.infos[base].1]
                    .style
                    .locale
                    .as_ref()
                    .map(|l| l.as_str().to_owned())
            });
            Ok(DiagnosticContext { pieces, locale })
        },
    )
}

fn diagnostic(
    graph: &Graph,
    edge: usize,
    invocations: &[Invocation],
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
    mut context: impl FnMut(&Unit) -> Result<DiagnosticContext>,
) -> crate::Error {
    let boundary = &graph.boundaries[edge];
    let mut pieces = Vec::new();
    let mut contexts = Vec::new();
    for index in [boundary.before, boundary.after] {
        let unit = &graph.units[index];
        let own = match context(unit) {
            Ok(value) => value,
            Err(error) => return error,
        };
        pieces.extend(own.pieces.iter().copied());
        contexts.push(unit_context(unit, own.pieces, own.locale));
    }
    let mut constraints: Vec<_> = invocations
        .iter()
        .flat_map(|i| &i.constraints)
        .filter(|c| graph.crosses(edge, &c.components))
        .map(|c| crate::SpacingConstraint {
            kind: c.kind.clone(),
            run: c.run,
            components: c.components.iter().map(|&a| a..a + 1).collect(),
            provenance: c.provenance.clone().into_boxed_slice(),
        })
        .collect();
    for invocation in invocations.iter().filter(|i| graph.touches(edge, i)) {
        if let Some(path) = invocation.path {
            constraints.push(crate::SpacingConstraint {
                kind: crate::SpacingConstraintKind::UnqualifiedProviderPath(path),
                run: invocation.run,
                components: vec![invocation.atoms.clone()].into_boxed_slice(),
                provenance: Box::new([]),
            });
        }
    }
    let reason = if !qualified(graph, edge) {
        SpacingFailureReason::UnqualifiedWritingSystem
    } else if invocations
        .iter()
        .any(|i| !i.complete && graph.touches(edge, i))
        || constraints.iter().any(|c| {
            matches!(
                c.kind,
                crate::SpacingConstraintKind::ExternalJoiningContext { .. }
            )
        })
    {
        SpacingFailureReason::UnqualifiedProviderPath
    } else if invocations.iter().flat_map(|i| &i.constraints).any(|c| {
        graph.crosses(edge, &c.components)
            && !c.required
            && c.kind == crate::SpacingConstraintKind::Ligature
    }) {
        SpacingFailureReason::PreservedOptionalLigature
    } else {
        SpacingFailureReason::CrossUnitConstraint
    };
    crate::Error::new(
        crate::ErrorCode::LayoutFailed,
        "material character spacing cannot preserve this boundary",
    )
    .with_detail(crate::ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        pieces: pieces.into_boxed_slice(),
        provider_atoms: [boundary.before, boundary.after]
            .into_iter()
            .flat_map(|u| graph.units[u].atoms.clone())
            .collect(),
        reason,
        contexts: contexts.into_boxed_slice(),
        constraints: constraints.into_boxed_slice(),
    })
}

pub(crate) fn qualified(graph: &Graph, edge: usize) -> bool {
    let edge = &graph.boundaries[edge];
    graph.units[edge.before].qualified && graph.units[edge.after].qualified
}

pub(crate) fn original(
    analyzed: &super::super::AnalyzedData,
    graph: &Graph,
    shaped: &mut ShapedData,
    ledger: &mut Ledger,
) -> Result<()> {
    let mut native = None;
    let ledger = if graph.ownership != GraphOwnership::Native {
        storage::require_ledger(ledger)?;
        ledger
    } else {
        &mut native
    };
    if graph.units.is_empty() {
        return Ok(());
    }
    let members = original_members(shaped, ledger)?;
    let mut regions = graph.find_regions_accounted(&members, ledger)?;
    for region in &regions {
        if graph.region_material(region, ledger)? {
            return Err(original_region_error(analyzed, graph, region, ledger));
        }
    }
    for (i, unit) in graph.units.iter().enumerate() {
        account::work(ledger, 1)?;
        if unqualified_unit(unit) && unit_material(graph, i, ledger)? {
            return Err(original_unit_error(analyzed, unit));
        }
    }
    if !graph.demand(ledger)? {
        drop_regions(regions, ledger)?;
        account::drop_vec(ledger, members)?;
        return Ok(());
    }
    if graph.ownership != GraphOwnership::Native {
        storage::fund_gaps(shaped, ledger)?;
    }
    let order = graph.index.original_order(shaped, &graph.units, ledger)?;
    let owners = graph.owners(shaped, &order, ledger)?;
    for (edge, boundary) in graph.boundaries.iter().enumerate() {
        let geometry = graph.movement_geometry(edge, &shaped.character_invocations, ledger)?;
        if matches!(geometry, SpacingBoundaryGeometry::ProtectedRequired) {
            continue;
        }
        if boundary.mean_lost {
            return Err(super::super::text_numeric_error(
                "character boundary mean",
                TextNumericPhase::Shaping,
                crate::TextNumericError::NonZeroLost,
                boundary.authored,
            ));
        }
        if boundary.authored == 0.0 {
            continue;
        }
        if !qualified(graph, edge) || geometry == SpacingBoundaryGeometry::Constrained {
            return Err(diagnostic(
                graph,
                edge,
                &shaped.character_invocations,
                crate::SpacingStage::Preparation,
                crate::SpacingOperation::AuthoredTracking,
                |unit| {
                    if let Subject::Cell(binding) = unit.subject {
                        return cell_context(analyzed, binding, ledger);
                    }
                    let mut pieces = Vec::new();
                    for atoms in &unit.atoms {
                        for atom in atoms.clone() {
                            super::original_pieces(analyzed, atom, &mut pieces, &mut None)
                                .expect("Native source-piece collection has no scope admission");
                        }
                    }
                    pieces.extend(
                        unit.atomics
                            .iter()
                            .map(|&occurrence| crate::SpacingPiece::Atomic { occurrence }),
                    );
                    let locale = unit.base.and_then(|base| {
                        analyzed.styles[analyzed.analysis.infos[base].1]
                            .style
                            .locale_tag()
                            .map(str::to_owned)
                    });
                    Ok(DiagnosticContext { pieces, locale })
                },
            ));
        }
        if let Some(owner) = owners[boundary.before] {
            for region in &regions {
                account::work(ledger, 1)?;
                if splits_region(&members, region, owner, shaped, ledger)? {
                    return Err(original_region_error(analyzed, graph, region, ledger));
                }
            }
            apply_gap_accounted(
                shaped,
                owner,
                TextNumericPhase::Shaping,
                boundary.authored,
                0.0,
                ledger,
            )?;
        }
    }
    for region in &mut regions {
        account::drop_vec(ledger, std::mem::take(&mut region.units))?;
        account::drop_vec(ledger, std::mem::take(&mut region.members))?;
    }
    account::drop_vec(ledger, regions)?;
    account::drop_vec(ledger, members)?;
    account::drop_vec(ledger, order)?;
    account::drop_vec(ledger, owners)?;
    Ok(())
}

fn apply_gap_accounted(
    shaped: &mut ShapedData,
    owner: Owner,
    phase: TextNumericPhase,
    authored: f64,
    allocation: f64,
    ledger: &mut Ledger,
) -> Result<()> {
    let gap = super::super::layout::add("character spacing total", phase, authored, allocation)?;
    let base = match owner {
        Owner::Cluster(i) => shaped.clusters[i].advance,
        Owner::Atomic(_) => 0.0,
        Owner::Cell(i) => shaped.cells[i].advance,
    };
    account::push(
        ledger,
        &mut shaped.character_gaps,
        Gap {
            owner,
            base,
            amount: gap,
            allocation,
        },
    )?;
    match owner {
        Owner::Cluster(i) => {
            let cluster = &mut shaped.clusters[i];
            cluster.advance =
                super::super::layout::add("character cell advance", phase, cluster.advance, gap)?;
            cluster.logical_adjustments[0] = authored;
            if allocation != 0.0 {
                cluster.logical_adjustments[2] = allocation;
            }
        }
        Owner::Atomic(i) => {
            account::insert(ledger, &mut shaped.atomic_gaps, i, gap)?;
        }
        Owner::Cell(i) => {
            shaped.cells[i].advance = super::super::layout::add(
                "combined character cell advance",
                phase,
                shaped.cells[i].advance,
                gap,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn atomic_advance(
    shaped: &ShapedData,
    occurrence: usize,
    allocation: f64,
) -> Result<f64> {
    super::super::layout::add(
        "atomic character cell advance",
        TextNumericPhase::Finalization,
        allocation,
        shaped.atomic_gaps.get(&occurrence).copied().unwrap_or(0.0),
    )
}

pub(crate) fn discarded(
    graph: &Graph,
    edge: usize,
    shaped: &ShapedData,
    dispositions: &[crate::SpacingSlotDisposition],
    ledger: &mut Ledger,
) -> Result<bool> {
    let boundary = &graph.boundaries[edge];
    for u in [boundary.before, boundary.after] {
        for (i, c) in shaped.clusters.iter().enumerate() {
            account::work(ledger, 1)?;
            for a in &graph.units[u].atoms {
                account::work(ledger, 1)?;
                if a.contains(&c.atoms.start)
                    && dispositions[i] == crate::SpacingSlotDisposition::DiscardedAtLineEdge
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub(crate) fn apply_selected_accounted(
    selected: PresentationInput<'_>,
    graph: &Graph,
    shaped: &mut ShapedData,
    order: &[usize],
    dispositions: &[crate::SpacingSlotDisposition],
    allocations: Option<&[f64]>,
    ledger: &mut Ledger,
) -> Result<()> {
    if graph.units.is_empty() {
        return Ok(());
    }
    if graph.ownership != GraphOwnership::Native {
        storage::fund_gaps(shaped, ledger)?;
    }
    for region in &graph.regions {
        if graph.region_material(region, ledger)? {
            return Err(region_error(
                selected,
                graph,
                region,
                crate::SpacingStage::SelectedShaping,
                crate::SpacingOperation::AuthoredTracking,
                ledger,
            ));
        }
    }
    for (i, unit) in graph.units.iter().enumerate() {
        account::work(ledger, 1)?;
        if unqualified_unit(unit) && unit_material(graph, i, ledger)? {
            return Err(unit_error(
                selected,
                unit,
                crate::SpacingStage::SelectedShaping,
                crate::SpacingOperation::AuthoredTracking,
            ));
        }
    }
    if !graph.demand(ledger)? && allocations.is_none() {
        return Ok(());
    }
    let owners = graph.owners(shaped, order, ledger)?;
    let mut members = Vec::new();
    for &i in order {
        account::work(ledger, 1)?;
        let member = match shaped.items[i] {
            ShapedItem::TextCluster(c) => Some(Member::Text(shaped.clusters[c].atoms.clone())),
            ShapedItem::InlineBox { occurrence, .. } => Some(Member::Atomic(occurrence)),
            ShapedItem::Cell(cell) => Some(Member::Cell(shaped.cells[cell].binding)),
            _ => None,
        };
        if let Some(member) = member {
            account::push(ledger, &mut members, member)?;
        }
    }
    let mut regions = graph.find_regions_accounted(&members, ledger)?;
    for (edge, boundary) in graph.boundaries.iter().enumerate() {
        if discarded(graph, edge, shaped, dispositions, ledger)? {
            continue;
        }
        let geometry = graph.geometry(edge, &shaped.character_invocations, ledger)?;
        let allocation = allocations.map_or(0.0, |a| a[edge]);
        if matches!(
            geometry,
            SpacingBoundaryGeometry::ProtectedRequired | SpacingBoundaryGeometry::ProtectedCursive
        ) {
            if allocation != 0.0 {
                return Err(crate::protocol::protocol_error(
                    crate::ProtocolError::IneligibleSpacingSlot { slot: edge },
                ));
            }
            if geometry == SpacingBoundaryGeometry::ProtectedRequired {
                continue;
            }
        }
        if boundary.mean_lost {
            return Err(super::super::text_numeric_error(
                "character boundary mean",
                TextNumericPhase::Finalization,
                crate::TextNumericError::NonZeroLost,
                boundary.authored,
            ));
        }
        let material = super::super::layout::add(
            "character spacing effect",
            TextNumericPhase::Finalization,
            boundary.authored,
            allocation,
        )?;
        if material != 0.0
            && (!qualified(graph, edge)
                || graph.movement_geometry(edge, &shaped.character_invocations, ledger)?
                    == SpacingBoundaryGeometry::Constrained)
        {
            return Err(failure(
                selected,
                graph,
                edge,
                if allocation != 0.0 {
                    crate::SpacingOperation::InterCharacterAllocation
                } else {
                    crate::SpacingOperation::AuthoredTracking
                },
                &shaped.character_invocations,
                ledger,
            ));
        }
        if let Some(owner) = owners[boundary.before] {
            if material != 0.0 {
                for region in &regions {
                    account::work(ledger, 1)?;
                    if splits_region(&members, region, owner, shaped, ledger)? {
                        return Err(region_error(
                            selected,
                            graph,
                            region,
                            crate::SpacingStage::SelectedShaping,
                            if allocation == 0.0 {
                                crate::SpacingOperation::AuthoredTracking
                            } else {
                                crate::SpacingOperation::InterCharacterAllocation
                            },
                            ledger,
                        ));
                    }
                }
            }
            apply_gap_accounted(
                shaped,
                owner,
                TextNumericPhase::Finalization,
                boundary.authored,
                allocation,
                ledger,
            )?;
        }
    }
    for region in &mut regions {
        account::drop_vec(ledger, std::mem::take(&mut region.units))?;
        account::drop_vec(ledger, std::mem::take(&mut region.members))?;
    }
    account::drop_vec(ledger, regions)?;
    account::drop_vec(ledger, members)?;
    account::drop_vec(ledger, owners)?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Gap {
    pub(crate) owner: Owner,
    pub(crate) base: f64,
    pub(crate) amount: f64,
    pub(crate) allocation: f64,
}

pub(crate) fn restore(shaped: &mut ShapedData) {
    for gap in &shaped.character_gaps {
        if let Owner::Cell(i) = gap.owner {
            shaped.cells[i].advance = gap.base;
        }
        if let Owner::Cluster(i) = gap.owner {
            shaped.clusters[i].advance = gap.base;
            shaped.clusters[i].logical_adjustments[0] = 0.0;
            if gap.allocation != 0.0 {
                shaped.clusters[i].logical_adjustments[2] = 0.0;
            }
        }
    }
    shaped.character_gaps.clear();
    shaped.atomic_gaps.clear();
}

#[derive(Clone, Debug)]
enum Member {
    Text(Range<usize>),
    Atomic(usize),
    Cell(usize),
}

fn member_matches(unit: &Unit, member: &Member, ledger: &mut Ledger) -> Result<bool> {
    account::work(ledger, 1)?;
    match member {
        Member::Text(atoms) => {
            for range in &unit.atoms {
                account::work(ledger, 1)?;
                if range.start < atoms.end && range.end > atoms.start {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Member::Atomic(i) => {
            for occurrence in &unit.atomics {
                account::work(ledger, 1)?;
                if occurrence == i {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Member::Cell(binding) => Ok(unit.subject == Subject::Cell(*binding)),
    }
}
#[derive(Clone, Debug)]
struct Region {
    first: usize,
    last: usize,
    units: Vec<usize>,
    members: Vec<Member>,
}

fn drop_regions(mut regions: Vec<Region>, ledger: &mut Ledger) -> Result<()> {
    for region in &mut regions {
        account::drop_vec(ledger, std::mem::take(&mut region.units))?;
        account::drop_vec(ledger, std::mem::take(&mut region.members))?;
    }
    account::drop_vec(ledger, regions)
}

impl Graph {
    fn find_regions_accounted(
        &self,
        members: &[Member],
        ledger: &mut Ledger,
    ) -> Result<Vec<Region>> {
        let mut regions = Vec::new();
        for unit in &self.units {
            account::work(ledger, 1)?;
            if unit.subject != Subject::Text {
                continue;
            }
            let mut first = None;
            let mut last = None;
            for (i, member) in members.iter().enumerate() {
                if member_matches(unit, member, ledger)? {
                    first.get_or_insert(i);
                    last = Some(i);
                }
            }
            let (Some(first), Some(last)) = (first, last) else {
                continue;
            };
            let mut interleaved = false;
            for member in &members[first..=last] {
                account::work(ledger, 1)?;
                if !matches!(member, Member::Text(_)) {
                    for unit in &self.units {
                        if member_matches(unit, member, ledger)? {
                            interleaved = true;
                            break;
                        }
                    }
                }
                if interleaved {
                    break;
                }
            }
            if !interleaved {
                continue;
            }
            let mut owned = Vec::new();
            for (i, unit) in self.units.iter().enumerate() {
                for member in &members[first..=last] {
                    if member_matches(unit, member, ledger)? {
                        account::push(ledger, &mut owned, i)?;
                        break;
                    }
                }
            }
            let members = account::clone_vec(ledger, &members[first..=last])?;
            account::push(
                ledger,
                &mut regions,
                Region {
                    first,
                    last,
                    units: owned,
                    members,
                },
            )?;
        }
        Ok(regions)
    }
    fn region_material(&self, region: &Region, ledger: &mut Ledger) -> Result<bool> {
        for &i in &region.units {
            account::work(ledger, 1)?;
            if self.units[i].left != 0.0 || self.units[i].right != 0.0 {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn original_members(shaped: &ShapedData, ledger: &mut Ledger) -> Result<Vec<Member>> {
    let mut levels = Vec::new();
    for item in &shaped.items {
        account::work(ledger, 1)?;
        let level = match *item {
            ShapedItem::TextCluster(i) => shaped.clusters[i].level,
            ShapedItem::Cell(i) => shaped.cells[i].level,
            ShapedItem::InlineBox { level, .. } => level,
            ShapedItem::Anchor { .. } => 0,
        };
        account::push(ledger, &mut levels, level)?;
    }
    let order = construction::order_levels(&levels, ledger)?;
    let mut members = Vec::new();
    for i in &order {
        account::work(ledger, 1)?;
        let member = match shaped.items[*i] {
            ShapedItem::TextCluster(c) => Some(Member::Text(shaped.clusters[c].atoms.clone())),
            ShapedItem::InlineBox { occurrence, .. } => Some(Member::Atomic(occurrence)),
            ShapedItem::Cell(i) => Some(Member::Cell(shaped.cells[i].binding)),
            _ => None,
        };
        if let Some(member) = member {
            account::push(ledger, &mut members, member)?;
        }
    }
    account::drop_vec(ledger, order)?;
    account::drop_vec(ledger, levels)?;
    Ok(members)
}

fn splits_region(
    members: &[Member],
    region: &Region,
    owner: Owner,
    shaped: &ShapedData,
    ledger: &mut Ledger,
) -> Result<bool> {
    for (position, m) in members.iter().enumerate() {
        account::work(ledger, 1)?;
        let matched = match (m, owner) {
            (Member::Text(a), Owner::Cluster(i)) => a.contains(&shaped.clusters[i].atoms.start),
            (Member::Atomic(a), Owner::Atomic(b)) => *a == b,
            (Member::Cell(a), Owner::Cell(i)) => *a == shaped.cells[i].binding,
            _ => false,
        };
        if matched {
            return Ok(region.first <= position && position < region.last);
        }
    }
    Ok(false)
}

fn unit_context(
    unit: &Unit,
    pieces: Vec<crate::SpacingPiece>,
    locale: Option<String>,
) -> crate::SpacingContext {
    crate::SpacingContext {
        pieces: pieces.into_boxed_slice(),
        locale,
        candidates: unit
            .scripts
            .iter()
            .filter_map(|&s| crate::TextScript::from_script(s))
            .collect(),
        resolved_scripts: unit
            .resolved_scripts
            .iter()
            .filter_map(|&s| crate::TextScript::from_script(s))
            .collect(),
        writing_systems: unit
            .context_scripts
            .iter()
            .filter_map(|&s| crate::TextScript::from_script(s))
            .collect(),
        letter_spacing_before: unit.left,
        letter_spacing_after: unit.right,
    }
}

#[derive(Clone, Copy)]
enum DiagnosticSource<'a> {
    Original(&'a crate::engine::AnalyzedData),
    Selected(PresentationInput<'a>),
}
impl<'a> DiagnosticSource<'a> {
    fn analyzed(self) -> &'a crate::engine::AnalyzedData {
        match self {
            Self::Original(a) => a,
            Self::Selected(i) => &i.prepared.analyzed,
        }
    }
    fn pieces(self, member: &Member, ledger: &mut Ledger) -> Result<Vec<crate::SpacingPiece>> {
        match member {
            Member::Cell(binding) => {
                Input::original(self.analyzed(), &[], 1.0).cell_pieces(*binding, ledger)
            }
            Member::Atomic(occurrence) => Ok(vec![crate::SpacingPiece::Atomic {
                occurrence: *occurrence,
            }]),
            Member::Text(atoms) => Ok(match self {
                Self::Original(a) => {
                    let mut pieces = Vec::new();
                    for atom in atoms.clone() {
                        super::original_pieces(a, atom, &mut pieces, &mut None)
                            .expect("Native source-piece collection has no scope admission");
                    }
                    pieces
                }
                Self::Selected(i) => super::slot_pieces(i.prepared, i.presentation, atoms.clone())
                    .into_iter()
                    .map(|p| p.owner)
                    .collect(),
            }),
        }
    }
    fn context(self, unit: &Unit, ledger: &mut Ledger) -> Result<DiagnosticContext> {
        if let Subject::Cell(binding) = unit.subject {
            return cell_context(self.analyzed(), binding, ledger);
        }
        let mut pieces = Vec::new();
        for atoms in &unit.atoms {
            pieces.extend(self.pieces(&Member::Text(atoms.clone()), ledger)?);
        }
        for &atomic in &unit.atomics {
            pieces.extend(self.pieces(&Member::Atomic(atomic), ledger)?);
        }
        let locale = unit.base.and_then(|base| match self {
            Self::Original(a) => a.styles[a.analysis.infos[base].1]
                .style
                .locale_tag()
                .map(str::to_owned),
            Self::Selected(i) => i.prepared.styles[i.presentation.analysis.infos[base].1]
                .style
                .locale
                .as_ref()
                .map(|l| l.as_str().to_owned()),
        });
        Ok(DiagnosticContext { pieces, locale })
    }
}
fn region_diagnostic(
    source: DiagnosticSource<'_>,
    graph: &Graph,
    region: &Region,
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
    ledger: &mut Ledger,
) -> crate::Error {
    let mut contexts = Vec::new();
    for &i in &region.units {
        let unit = &graph.units[i];
        let c = match source.context(unit, ledger) {
            Ok(c) => c,
            Err(e) => return e,
        };
        contexts.push(unit_context(unit, c.pieces, c.locale));
    }
    let mut pieces = Vec::new();
    for member in &region.members {
        match source.pieces(member, ledger) {
            Ok(own) => pieces.extend(own),
            Err(e) => return e,
        }
    }
    crate::Error::new(
        crate::ErrorCode::LayoutFailed,
        "interleaved text and atomic topology is not qualified for material spacing",
    )
    .with_detail(crate::ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        reason: SpacingFailureReason::InterleavedUnit,
        pieces: pieces.into_boxed_slice(),
        provider_atoms: region
            .members
            .iter()
            .filter_map(|m| {
                if let Member::Text(a) = m {
                    Some(a.clone())
                } else {
                    None
                }
            })
            .collect(),
        contexts: contexts.into_boxed_slice(),
        constraints: Box::new([]),
    })
}
fn region_error(
    input: PresentationInput<'_>,
    graph: &Graph,
    region: &Region,
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
    ledger: &mut Ledger,
) -> crate::Error {
    region_diagnostic(
        DiagnosticSource::Selected(input),
        graph,
        region,
        stage,
        operation,
        ledger,
    )
}
fn original_region_error(
    analyzed: &crate::engine::AnalyzedData,
    graph: &Graph,
    region: &Region,
    ledger: &mut Ledger,
) -> crate::Error {
    region_diagnostic(
        DiagnosticSource::Original(analyzed),
        graph,
        region,
        crate::SpacingStage::Preparation,
        crate::SpacingOperation::AuthoredTracking,
        ledger,
    )
}

pub(crate) fn admit(
    input: PresentationInput<'_>,
    graph: &Graph,
    basis: bool,
    ledger: &mut Ledger,
) -> Result<()> {
    let mut native = None;
    let ledger = if graph.ownership == GraphOwnership::Native {
        &mut native
    } else {
        storage::require_ledger(ledger)?;
        ledger
    };
    for region in &graph.regions {
        account::work(ledger, 1)?;
        if basis || graph.region_material(region, ledger)? {
            return Err(region_error(
                input,
                graph,
                region,
                crate::SpacingStage::Selection,
                if basis {
                    crate::SpacingOperation::InterCharacterBasis
                } else {
                    crate::SpacingOperation::AuthoredTracking
                },
                ledger,
            ));
        }
    }
    for (i, u) in graph.units.iter().enumerate() {
        account::work(ledger, 1)?;
        let mut boundary = false;
        for b in &graph.boundaries {
            account::work(ledger, 1)?;
            if b.before == i || b.after == i {
                boundary = true;
                break;
            }
        }
        if (unqualified_unit(u) || basis && !u.justify_qualified && !u.barrier)
            && if basis {
                !u.native_only || boundary
            } else {
                unit_material(graph, i, ledger)?
            }
        {
            return Err(unit_error(
                input,
                u,
                crate::SpacingStage::Selection,
                if basis {
                    crate::SpacingOperation::InterCharacterBasis
                } else {
                    crate::SpacingOperation::AuthoredTracking
                },
            ));
        }
    }
    Ok(())
}

fn unit_material(graph: &Graph, index: usize, ledger: &mut Ledger) -> Result<bool> {
    let unit = &graph.units[index];
    if !unit.native_only {
        return Ok(unit.left != 0.0 || unit.right != 0.0);
    }
    for b in &graph.boundaries {
        account::work(ledger, 1)?;
        if (b.before == index || b.after == index) && (b.authored != 0.0 || b.mean_lost) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn unqualified_unit(unit: &Unit) -> bool {
    !unit.qualified && !unit.barrier
}
fn unit_error(
    input: PresentationInput<'_>,
    unit: &Unit,
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
) -> crate::Error {
    let pieces: Vec<_> = unit
        .atoms
        .iter()
        .flat_map(|a| {
            super::slot_pieces(input.prepared, input.presentation, a.clone())
                .into_iter()
                .map(|p| p.owner)
        })
        .chain(
            unit.atomics
                .iter()
                .map(|&occurrence| crate::SpacingPiece::Atomic { occurrence }),
        )
        .collect();
    let locale = unit.base.and_then(|b| {
        input.prepared.styles[input.presentation.analysis.infos[b].1]
            .style
            .locale
            .as_ref()
            .map(|l| l.as_str().to_owned())
    });
    crate::Error::new(
        crate::ErrorCode::LayoutFailed,
        "character-unit topology is not qualified for this spacing operation",
    )
    .with_detail(crate::ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        reason: SpacingFailureReason::UnqualifiedWritingSystem,
        contexts: vec![unit_context(unit, pieces.clone(), locale)].into_boxed_slice(),
        pieces: pieces.into_boxed_slice(),
        provider_atoms: unit.atoms.clone().into_boxed_slice(),
        constraints: Box::new([]),
    })
}
fn original_unit_error(analyzed: &super::super::AnalyzedData, unit: &Unit) -> crate::Error {
    let mut pieces = Vec::new();
    for atoms in &unit.atoms {
        for a in atoms.clone() {
            super::original_pieces(analyzed, a, &mut pieces, &mut None)
                .expect("Native source-piece collection has no scope admission");
        }
    }
    pieces.extend(
        unit.atomics
            .iter()
            .map(|&occurrence| crate::SpacingPiece::Atomic { occurrence }),
    );
    let locale = unit.base.and_then(|b| {
        analyzed.styles[analyzed.analysis.infos[b].1]
            .style
            .locale_tag()
            .map(str::to_owned)
    });
    crate::Error::new(
        crate::ErrorCode::LayoutFailed,
        "character-unit topology is not qualified for authored tracking",
    )
    .with_detail(crate::ErrorDetail::UnsupportedSpacing {
        stage: crate::SpacingStage::Preparation,
        operation: crate::SpacingOperation::AuthoredTracking,
        reason: SpacingFailureReason::UnqualifiedWritingSystem,
        contexts: vec![unit_context(unit, pieces.clone(), locale)].into_boxed_slice(),
        pieces: pieces.into_boxed_slice(),
        provider_atoms: unit.atoms.clone().into_boxed_slice(),
        constraints: Box::new([]),
    })
}
