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
use std::{collections::BTreeMap, ops::Range};

use super::super::scope_alignment::accounting::{self as account, Ledger};
mod planning;
pub(crate) use planning::{PlanRequirements, plan};

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

#[derive(Clone, Debug, Default)]
pub(crate) struct Graph {
    pub(crate) units: Vec<Unit>,
    pub(crate) boundaries: Vec<Boundary>,
    pub(crate) potential: bool,
    regions: Vec<Region>,
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

pub(crate) fn needed(
    analysis: &Analysis,
    styles: &[ResolvedStyleRun],
    boxes: &[InlineBox],
) -> bool {
    analysis
        .infos
        .iter()
        .any(|(_, i)| styles[*i].style.letter_spacing != 0.0)
        || boxes.iter().any(|b| b.letter_spacing() != 0.0)
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
fn explicit_family(tag: &str) -> Option<Vec<Script>> {
    Some(match tag {
        "Latn" => vec![Script::Latin],
        "Grek" => vec![Script::Greek],
        "Cyrl" => vec![Script::Cyrillic],
        "Armn" => vec![Script::Armenian],
        "Hebr" => vec![Script::Hebrew],
        "Hant" | "Hans" | "Hani" | "Hanb" | "Bopo" => vec![Script::Han, Script::Bopomofo],
        "Jpan" | "Hrkt" | "Hira" | "Kana" => vec![Script::Han, Script::Hiragana, Script::Katakana],
        "Kore" | "Hang" | "Jamo" => vec![Script::Hangul, Script::Han],
        "Arab" => vec![Script::Arabic],
        "Rohg" => vec![Script::HanifiRohingya],
        "Mand" => vec![Script::Mandaic],
        "Mong" => vec![Script::Mongolian],
        "Nkoo" => vec![Script::Nko],
        "Phag" => vec![Script::PhagsPa],
        "Syrc" => vec![Script::Syriac],
        _ => return None,
    })
}
fn locale_family(locale: &fontique::Language) -> Option<Vec<Script>> {
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

fn operation_graphemes(
    projection: &SourceProjection,
    analysis: &Analysis,
    exclusions: Option<&BTreeMap<usize, Exclusion>>,
) -> Vec<Range<usize>> {
    let Some(exclusions) = exclusions.filter(|e| !e.is_empty()) else {
        return analysis.graphemes.clone();
    };
    // Only the completed invocation establishes transparency. Native shaping
    // already consumed these inputs, and their source EGC custody is retained.
    let mut components = projection.barrier_atoms.clone();
    components.extend([0, projection.atoms.len()]);
    components.sort_unstable();
    components.dedup();
    let mut scalars = projection.segmentation.chars().enumerate();
    let mut output: Vec<Range<usize>> = Vec::new();
    for component in components.windows(2) {
        let mut text = String::new();
        let mut kept = Vec::new();
        for (atom, ch) in scalars.by_ref().take(component[1] - component[0]) {
            if !exclusions.contains_key(&atom) {
                kept.push((text.len(), atom));
                text.push(ch);
            }
        }
        let stops: Vec<_> = icu_segmenter::GraphemeClusterSegmenter::new()
            .segment_str(&text)
            .collect();
        for pair in stops.windows(2) {
            let first = kept.partition_point(|(byte, _)| *byte < pair[0]);
            let end = kept.partition_point(|(byte, _)| *byte < pair[1]);
            if first == end {
                continue;
            }
            let a = kept[first].1;
            let b = kept[end - 1].1;
            let start = analysis.graphemes.partition_point(|g| g.end <= a);
            let last = analysis.graphemes.partition_point(|g| g.end <= b);
            let range = analysis.graphemes[start].start..analysis.graphemes[last].end;
            if exclusions.range(range.clone()).next().is_some() {
                if let Some(prior) = output.last_mut()
                    && prior.end > range.start
                {
                    prior.end = prior.end.max(range.end);
                } else {
                    output.push(range);
                }
            } else {
                for g in &analysis.graphemes[start..=last] {
                    if output.last() != Some(g) {
                        output.push(g.clone());
                    }
                }
            }
        }
    }
    output
}

pub(crate) fn build(
    projection: &SourceProjection,
    analysis: &Analysis,
    styles: &[ResolvedStyleRun],
    boxes: &[InlineBox],
    presentation: Option<&SelectedPresentation>,
    potential: bool,
    exclusions: Option<&BTreeMap<usize, Exclusion>>,
) -> Result<Graph> {
    if !potential && !needed(analysis, styles, boxes) {
        return Ok(Graph::default());
    }
    let mut units = Vec::new();
    for atoms in &operation_graphemes(projection, analysis, exclusions) {
        if atoms.is_empty() {
            continue;
        }
        let eligible: Vec<_> = atoms
            .clone()
            .filter(|atom| exclusions.is_none_or(|e| !e.contains_key(atom)))
            .collect();
        if eligible.is_empty() {
            continue;
        }
        let retained: Vec<_> = eligible
            .iter()
            .map(|&i| projection.atoms[i].clone())
            .collect();
        let slice = retained.as_slice();
        let base = unit_properties::base(slice).expect("nonempty presentation EGC");
        let base_index = eligible[slice
            .iter()
            .position(|a| std::ptr::eq(a, base))
            .expect("EGC base")];
        let style = &styles[analysis.infos[base_index].1].style;
        let properties = unit_properties::properties(slice)?;
        let scripts = properties.scripts.clone();
        let real: Vec<_> = scripts
            .iter()
            .copied()
            .filter(|s| !matches!(*s, Script::Common | Script::Inherited | Script::Unknown))
            .collect();
        let family = style.locale.as_ref().and_then(locale_family);
        let unspecified = style.locale.as_ref().is_none_or(|l| l.as_str() == "und");
        let numeric = GeneralCategoryGroup::Number.contains(properties.category);
        let letter = GeneralCategoryGroup::Letter.contains(properties.category) || numeric;
        let symbol = GeneralCategoryGroup::Symbol.contains(properties.category)
            || GeneralCategoryGroup::Punctuation.contains(properties.category);
        let source_supported = !real.is_empty() && real.iter().all(|s| supported(*s));
        let uniform_rule =
            !letter || real.iter().all(|s| cursive(*s)) || real.iter().all(|s| !cursive(*s));
        let mut resolved_scripts = real.clone();
        let mut qualified = if unspecified {
            source_supported && uniform_rule
                || (numeric || properties.category == GeneralCategory::SpaceSeparator)
                    && real.is_empty()
        } else {
            family.is_some() && (source_supported && uniform_rule || real.is_empty())
        };
        let ambiguous = (letter || symbol)
            && real.len() > 1
            && (!source_supported || real.iter().any(|s| cursive(*s)));
        let mut justify_qualified = qualified;
        if ambiguous {
            resolved_scripts = family
                .as_ref()
                .map(|f| real.iter().copied().filter(|s| f.contains(s)).collect())
                .unwrap_or_default();
            // The captured profile can exclude unrelated SCX alternatives.
            // Several admitted noncursive scripts share the ordinary operation;
            // cursive protection still needs one actual resolved script.
            let coherent = !resolved_scripts.is_empty()
                && resolved_scripts.iter().all(|s| supported(*s))
                && (resolved_scripts.iter().all(|s| !cursive(*s)) || resolved_scripts.len() == 1);
            justify_qualified = coherent;
            if letter {
                qualified = coherent;
            } else {
                qualified |= coherent;
            }
        }
        let is_cursive =
            letter && !resolved_scripts.is_empty() && resolved_scripts.iter().all(|s| cursive(*s));
        let justify_cursive = is_cursive
            || symbol
                && !resolved_scripts.is_empty()
                && resolved_scripts.iter().all(|s| cursive(*s));
        let domain = presentation.and_then(|p| {
            p.domains.iter().position(|d| {
                p.units[d.units.clone()]
                    .iter()
                    .any(|u| u.atoms.contains(&base_index))
            })
        });
        units.push(Unit {
            atoms: vec![atoms.clone()],
            atomics: Vec::new(),
            base: Some(base_index),
            left: style.letter_spacing,
            right: style.letter_spacing,
            qualified,
            cursive: is_cursive,
            justify_cursive,
            justify_qualified,
            symbol,
            resolved_scripts,
            context_scripts: family.unwrap_or_default(),
            content_context: false,
            scripts,
            domain,
            native_only: slice.iter().all(|a| {
                CodePointSetData::new::<DefaultIgnorableCodePoint>().contains32(a.code_point)
            }),
            barrier: slice
                .iter()
                .any(|a| matches!(a.code_point, 9 | 10 | 13 | 0x2028 | 0x2029)),
            level: analysis.levels[base_index],
        });
    }
    for (occurrence, b) in boxes.iter().enumerate() {
        if b.kind() == InlineBoxKind::OutOfFlow {
            continue;
        }
        if presentation.is_some_and(|p| !p.units.iter().any(|u| matches!(u.kind, crate::protocol::prepared::PresentationUnitKind::Atomic { occurrence: i } if i == occurrence))) { continue; }
        units.push(Unit {
            atoms: Vec::new(),
            atomics: vec![occurrence],
            base: None,
            left: b.letter_spacing(),
            right: b.letter_spacing(),
            qualified: true,
            cursive: false,
            justify_cursive: false,
            justify_qualified: true,
            symbol: false,
            resolved_scripts: Vec::new(),
            scripts: Vec::new(),
            context_scripts: Vec::new(),
            content_context: false,
            domain: None,
            barrier: false,
            native_only: false,
            level: analysis.box_levels[occurrence],
        });
    }
    // Resolve Common P/S only from actual source context; declared writing-system facts remain separate.
    for i in 0..units.len() {
        if !units[i].symbol
            || units[i].base.is_none()
            || units[i]
                .scripts
                .iter()
                .any(|s| !matches!(*s, Script::Common | Script::Inherited | Script::Unknown))
        {
            continue;
        }
        units[i].content_context = true;
        let mut counts = std::collections::BTreeMap::<Script, usize>::new();
        for other in &units {
            if other.domain != units[i].domain {
                continue;
            }
            let real: Vec<_> = other
                .scripts
                .iter()
                .copied()
                .filter(|s| !matches!(*s, Script::Common | Script::Inherited | Script::Unknown))
                .collect();
            if real.len() == 1 {
                *counts.entry(real[0]).or_default() += 1;
            }
        }
        let maximum = counts.values().copied().max().unwrap_or(0);
        let dominant: Vec<_> = counts
            .into_iter()
            .filter_map(|(script, count)| (count == maximum).then_some(script))
            .collect();
        let declared = &units[i].context_scripts;
        let mut actual = if dominant.is_empty() {
            declared.clone()
        } else {
            dominant
        };
        if actual.len() > 1 && actual.iter().any(|s| cursive(*s)) {
            let selected: Vec<_> = actual
                .iter()
                .copied()
                .filter(|s| declared.contains(s))
                .collect();
            if selected.len() == 1 {
                actual = selected;
            }
        }
        let uniform = !actual.is_empty()
            && actual.iter().all(|s| supported(*s))
            && (actual.iter().all(|s| cursive(*s)) || actual.iter().all(|s| !cursive(*s)));
        let actual_cursive = uniform && actual.iter().all(|s| cursive(*s));
        let declared_uniform = declared.is_empty()
            || declared.iter().all(|s| cursive(*s))
            || declared.iter().all(|s| !cursive(*s));
        let conflict = !declared.is_empty()
            && (!declared_uniform || declared.iter().all(|s| cursive(*s)) != actual_cursive);
        let base = units[i].base.expect("text symbol base");
        let unspecified = styles[analysis.infos[base].1]
            .style
            .locale
            .as_ref()
            .is_none_or(|l| l.as_str() == "und");
        units[i].qualified = uniform && (unspecified || !declared.is_empty());
        units[i].justify_qualified =
            units[i].qualified && !conflict && (!actual_cursive || actual.len() == 1);
        units[i].justify_cursive = actual_cursive;
        units[i].resolved_scripts = actual;
    }
    let mut selected_members = None;
    if let Some(p) = presentation {
        let order =
            crate::protocol::prepared::presentation::unit_order(&p.units, &p.domains, boxes)?;
        selected_members = Some(
            order
                .iter()
                .filter_map(|&i| {
                    let u = &p.units[i];
                    match u.kind {
                        crate::protocol::prepared::PresentationUnitKind::Atomic { occurrence } => {
                            (boxes[occurrence].kind() == InlineBoxKind::InFlow)
                                .then_some(Member::Atomic(occurrence))
                        }
                        _ => (!u.atoms.is_empty()).then(|| Member::Text(u.atoms.clone())),
                    }
                })
                .collect::<Vec<_>>(),
        );
        let mut rank = vec![0; p.units.len()];
        for (i, unit) in order.into_iter().enumerate() {
            rank[unit] = i;
        }
        units.sort_by_key(|u| {
            let index = p.units.iter().position(|p| if let Some(base) = u.base { p.atoms.contains(&base) } else { matches!(p.kind, crate::protocol::prepared::PresentationUnitKind::Atomic { occurrence } if u.atomics.contains(&occurrence)) }).expect("selected unit retains source custody");
            let sub = u.base.unwrap_or(0);
            (rank[index], if p.units[index].line_level & 1 == 0 { sub } else { usize::MAX - sub })
        });
    } else {
        units.sort_by_key(|u| {
            if let Some(base) = u.base {
                (projection.atoms[base].source.start, 1)
            } else {
                (boxes[u.atomics[0]].index(), 0)
            }
        });
        let mut reordered = Vec::with_capacity(units.len());
        let mut start = 0;
        for end in (0..units.len())
            .filter(|&i| units[i].barrier)
            .chain(std::iter::once(units.len()))
        {
            let participants = units[start..end]
                .iter()
                .map(|u| {
                    crate::BidiLevel::try_new(u.level)
                        .map(|level| crate::SuppliedInlineParticipant::Boundary { level })
                })
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|_| {
                    crate::Error::new(crate::ErrorCode::LayoutFailed, "character unit level")
                })?;
            let mut order = vec![0; participants.len()];
            crate::SuppliedLine::new(&participants)
                .reorder_into(&mut order)
                .map_err(|_| {
                    crate::Error::new(crate::ErrorCode::LayoutFailed, "character unit order")
                })?;
            reordered.extend(order.into_iter().map(|i| units[start + i].clone()));
            if end < units.len() {
                reordered.push(units[end].clone());
            }
            start = end.saturating_add(1);
        }
        units = reordered;
    }
    let mut grouped: Vec<Unit> = Vec::new();
    for unit in units {
        if !unit.atomics.is_empty()
            && let Some(prior) = grouped.last_mut()
            && !prior.atomics.is_empty()
        {
            prior.right = unit.right;
            prior.atomics.extend(unit.atomics);
            continue;
        }
        grouped.push(unit);
    }
    let mut boundaries = Vec::new();
    for i in 1..grouped.len() {
        if grouped[i - 1].barrier || grouped[i].barrier {
            continue;
        }
        let same_script = grouped[i - 1].resolved_scripts.len() == 1
            && grouped[i].resolved_scripts.len() == 1
            && grouped[i - 1].resolved_scripts == grouped[i].resolved_scripts;
        let protected = same_script && grouped[i - 1].cursive && grouped[i].cursive;
        let justify_protected =
            same_script && grouped[i - 1].justify_cursive && grouped[i].justify_cursive;
        let a = grouped[i - 1].right;
        let b = grouped[i].left;
        let authored = if protected || a == -b {
            0.0
        } else {
            a.midpoint(b)
        };
        let mean_lost = !protected && a != -b && authored == 0.0;
        boundaries.push(Boundary {
            before: i - 1,
            after: i,
            authored,
            mean_lost,
            geometry: if justify_protected {
                SpacingBoundaryGeometry::ProtectedCursive
            } else {
                SpacingBoundaryGeometry::Independent
            },
        });
    }
    let mut graph = Graph {
        units: grouped,
        boundaries,
        potential,
        regions: Vec::new(),
    };
    if let Some(members) = selected_members {
        graph.regions = graph.find_regions(&members);
    }
    Ok(graph)
}

/// The resolver owns which units depend on domain content. The caller proves
/// source contributions invariant; an absent proof is conditional, not a new
/// script winner. Counts and actual resolution remain solely in the rule above.
fn conditional_context_consumers(units: &[Unit], contributions_invariant: &[bool]) -> Vec<usize> {
    let affected: std::collections::BTreeSet<_> = units
        .iter()
        .enumerate()
        .filter(|(i, unit)| {
            unit.base.is_some() && !contributions_invariant.get(*i).copied().unwrap_or(false)
        })
        .map(|(_, unit)| unit.domain)
        .collect();
    units
        .iter()
        .enumerate()
        .filter_map(|(i, unit)| {
            (unit.content_context && affected.contains(&unit.domain)).then_some(i)
        })
        .collect()
}

impl Graph {
    pub(crate) fn demand(&self) -> bool {
        self.boundaries.iter().any(|b| {
            (b.authored != 0.0 || b.mean_lost)
                || self.potential && b.geometry != SpacingBoundaryGeometry::ProtectedCursive
        })
    }
    pub(crate) fn owner(&self, shaped: &ShapedData, unit: usize, order: &[usize]) -> Option<Owner> {
        order
            .iter()
            .rev()
            .find_map(|&ordinal| match shaped.items[ordinal] {
                ShapedItem::TextCluster(i)
                    if !shaped
                        .character_exclusions
                        .contains_key(&shaped.clusters[i].atoms.start)
                        && self.units[unit]
                            .atoms
                            .iter()
                            .any(|r| r.contains(&shaped.clusters[i].atoms.start)) =>
                {
                    Some(Owner::Cluster(i))
                }
                ShapedItem::InlineBox { occurrence, .. }
                    if self.units[unit].atomics.contains(&occurrence) =>
                {
                    Some(Owner::Atomic(occurrence))
                }
                _ => None,
            })
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
    ) -> SpacingBoundaryGeometry {
        let movement = self.movement_geometry(edge, invocations);
        if movement == SpacingBoundaryGeometry::ProtectedRequired {
            movement
        } else if self.boundaries[edge].geometry == SpacingBoundaryGeometry::ProtectedCursive {
            SpacingBoundaryGeometry::ProtectedCursive
        } else {
            movement
        }
    }
    pub(crate) fn movement_geometry(
        &self,
        edge: usize,
        invocations: &[Invocation],
    ) -> SpacingBoundaryGeometry {
        let mut constrained = false;
        for invocation in invocations {
            if !invocation.complete && self.touches(edge, invocation) {
                constrained = true;
            }
            for relation in &invocation.constraints {
                if self.crosses(edge, &relation.components) {
                    if relation.required && relation.kind == crate::SpacingConstraintKind::Ligature
                    {
                        return SpacingBoundaryGeometry::ProtectedRequired;
                    }
                    constrained = true;
                }
            }
            for components in invocation.components.iter().chain(&invocation.generated) {
                if self.crosses(edge, components) {
                    constrained = true;
                }
            }
        }
        if constrained {
            SpacingBoundaryGeometry::Constrained
        } else {
            SpacingBoundaryGeometry::Independent
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Owner {
    Cluster(usize),
    Atomic(usize),
}

pub(crate) fn failure(
    input: PresentationInput<'_>,
    graph: &Graph,
    edge: usize,
    operation: crate::SpacingOperation,
    invocations: &[Invocation],
) -> crate::Error {
    diagnostic(
        graph,
        edge,
        invocations,
        crate::SpacingStage::SelectedShaping,
        operation,
        |unit| {
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
            pieces
        },
        |base| {
            input.prepared.styles[input.presentation.analysis.infos[base].1]
                .style
                .locale
                .as_ref()
                .map(|l| l.as_str().to_owned())
        },
    )
}

fn diagnostic(
    graph: &Graph,
    edge: usize,
    invocations: &[Invocation],
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
    content: impl Fn(&Unit) -> Vec<crate::SpacingPiece>,
    locale: impl Fn(usize) -> Option<String>,
) -> crate::Error {
    let boundary = &graph.boundaries[edge];
    let mut pieces = Vec::new();
    let mut contexts = Vec::new();
    for index in [boundary.before, boundary.after] {
        let unit = &graph.units[index];
        let own = content(unit);
        pieces.extend(own.iter().copied());
        contexts.push(unit_context(unit, own, unit.base.and_then(&locale)));
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
) -> Result<()> {
    if graph.units.is_empty() {
        return Ok(());
    }
    let members = original_members(shaped)?;
    let regions = graph.find_regions(&members);
    for region in &regions {
        if graph.region_material(region) {
            return Err(original_region_error(analyzed, graph, region));
        }
    }
    if let Some(unit) = graph
        .units
        .iter()
        .enumerate()
        .find(|(i, u)| unqualified_unit(u) && unit_material(graph, *i))
        .map(|(_, u)| u)
    {
        return Err(original_unit_error(analyzed, unit));
    }
    if !graph.demand() {
        return Ok(());
    }
    let order: Vec<_> = graph
        .units
        .iter()
        .flat_map(|unit| {
            let mut members: Vec<_> = shaped
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| match *item {
                    ShapedItem::TextCluster(c)
                        if unit
                            .atoms
                            .iter()
                            .any(|a| a.contains(&shaped.clusters[c].atoms.start)) =>
                    {
                        Some(i)
                    }
                    ShapedItem::InlineBox { occurrence, .. }
                        if unit.atomics.contains(&occurrence) =>
                    {
                        Some(i)
                    }
                    _ => None,
                })
                .collect();
            if unit.level & 1 != 0 {
                members.reverse();
            }
            members
        })
        .collect();
    for (edge, boundary) in graph.boundaries.iter().enumerate() {
        let geometry = graph.movement_geometry(edge, &shaped.character_invocations);
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
                    let mut pieces = Vec::new();
                    for atoms in &unit.atoms {
                        for atom in atoms.clone() {
                            super::original_pieces(analyzed, atom, &mut pieces);
                        }
                    }
                    pieces.extend(
                        unit.atomics
                            .iter()
                            .map(|&occurrence| crate::SpacingPiece::Atomic { occurrence }),
                    );
                    pieces
                },
                |base| {
                    analyzed.styles[analyzed.analysis.infos[base].1]
                        .style
                        .locale_tag()
                        .map(str::to_owned)
                },
            ));
        }
        if let Some(owner) = graph.owner(shaped, boundary.before, &order) {
            if regions
                .iter()
                .any(|region| splits_region(&members, region, owner, shaped))
            {
                return Err(original_region_error(
                    analyzed,
                    graph,
                    regions
                        .iter()
                        .find(|region| splits_region(&members, region, owner, shaped))
                        .expect("actual split"),
                ));
            }
            apply_gap(
                shaped,
                owner,
                TextNumericPhase::Shaping,
                boundary.authored,
                0.0,
            )?;
        }
    }
    Ok(())
}

fn apply_gap(
    shaped: &mut ShapedData,
    owner: Owner,
    phase: TextNumericPhase,
    authored: f64,
    allocation: f64,
) -> Result<()> {
    apply_gap_accounted(shaped, owner, phase, authored, allocation, &mut None)
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
) -> bool {
    let boundary = &graph.boundaries[edge];
    [boundary.before, boundary.after].into_iter().any(|u| {
        shaped.clusters.iter().enumerate().any(|(i, c)| {
            graph.units[u]
                .atoms
                .iter()
                .any(|a| a.contains(&c.atoms.start))
                && dispositions[i] == crate::SpacingSlotDisposition::DiscardedAtLineEdge
        })
    })
}

pub(crate) fn apply_selected(
    selected: PresentationInput<'_>,
    graph: &Graph,
    shaped: &mut ShapedData,
    order: &[usize],
    dispositions: &[crate::SpacingSlotDisposition],
    allocations: Option<&[f64]>,
) -> Result<()> {
    apply_selected_accounted(
        selected,
        graph,
        shaped,
        order,
        dispositions,
        allocations,
        &mut None,
    )
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
    for region in &graph.regions {
        if graph.region_material(region) {
            return Err(region_error(
                selected,
                graph,
                region,
                crate::SpacingStage::SelectedShaping,
                crate::SpacingOperation::AuthoredTracking,
            ));
        }
    }
    if let Some(unit) = graph
        .units
        .iter()
        .enumerate()
        .find(|(i, u)| unqualified_unit(u) && unit_material(graph, *i))
        .map(|(_, u)| u)
    {
        return Err(unit_error(
            selected,
            unit,
            crate::SpacingStage::SelectedShaping,
            crate::SpacingOperation::AuthoredTracking,
        ));
    }
    if !graph.demand() && allocations.is_none() {
        return Ok(());
    }
    let members = account::collect(
        ledger,
        order.iter().filter_map(|&i| match shaped.items[i] {
            ShapedItem::TextCluster(c) => Some(Member::Text(shaped.clusters[c].atoms.clone())),
            ShapedItem::InlineBox { occurrence, .. } => Some(Member::Atomic(occurrence)),
            _ => None,
        }),
    )?;
    let mut regions = graph.find_regions_accounted(&members, ledger)?;
    for (edge, boundary) in graph.boundaries.iter().enumerate() {
        if discarded(graph, edge, shaped, dispositions) {
            continue;
        }
        let geometry = graph.geometry(edge, &shaped.character_invocations);
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
                || graph.movement_geometry(edge, &shaped.character_invocations)
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
            ));
        }
        if let Some(owner) = graph.owner(shaped, boundary.before, order) {
            if material != 0.0
                && let Some(region) = regions
                    .iter()
                    .find(|region| splits_region(&members, region, owner, shaped))
            {
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
                ));
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
}
#[derive(Clone, Debug)]
struct Region {
    first: usize,
    last: usize,
    units: Vec<usize>,
    members: Vec<Member>,
}

impl Graph {
    fn find_regions(&self, members: &[Member]) -> Vec<Region> {
        self.find_regions_accounted(members, &mut None)
            .expect("ordinary regions have no scope ledger")
    }
    fn find_regions_accounted(
        &self,
        members: &[Member],
        ledger: &mut Ledger,
    ) -> Result<Vec<Region>> {
        let mut regions = Vec::new();
        for unit in self.units.iter().filter(|u| !u.atoms.is_empty()) {
            let positions = account::collect(
                ledger,
                members.iter().enumerate().filter_map(|(i, m)| match m {
                    Member::Text(atoms)
                        if unit
                            .atoms
                            .iter()
                            .any(|a| a.start < atoms.end && a.end > atoms.start) =>
                    {
                        Some(i)
                    }
                    _ => None,
                }),
            )?;
            let (Some(&first), Some(&last)) = (positions.first(), positions.last()) else {
                account::drop_vec(ledger, positions)?;
                continue;
            };
            account::drop_vec(ledger, positions)?;
            if !members[first..=last].iter().any(|m| matches!(m, Member::Atomic(i) if self.units.iter().any(|u| u.atomics.contains(i)))) { continue; }
            let units = account::collect(
                ledger,
                self.units.iter().enumerate().filter_map(|(i, u)| {
                    members[first..=last]
                        .iter()
                        .any(|m| match m {
                            Member::Text(atoms) => u
                                .atoms
                                .iter()
                                .any(|a| a.start < atoms.end && a.end > atoms.start),
                            Member::Atomic(occurrence) => u.atomics.contains(occurrence),
                        })
                        .then_some(i)
                }),
            )?;
            let owned_members = account::clone_vec(ledger, &members[first..=last])?;
            account::push(
                ledger,
                &mut regions,
                Region {
                    first,
                    last,
                    units,
                    members: owned_members,
                },
            )?;
        }
        Ok(regions)
    }
    fn region_material(&self, region: &Region) -> bool {
        region
            .units
            .iter()
            .any(|&i| self.units[i].left != 0.0 || self.units[i].right != 0.0)
    }
}

fn original_members(shaped: &ShapedData) -> Result<Vec<Member>> {
    let participants = shaped
        .items
        .iter()
        .map(|item| {
            let level = match *item {
                ShapedItem::TextCluster(i) => shaped.clusters[i].level,
                ShapedItem::Cell(i) => shaped.cells[i].level,
                ShapedItem::InlineBox { level, .. } => level,
                ShapedItem::Anchor { .. } => 0,
            };
            crate::BidiLevel::try_new(level)
                .map(|level| crate::SuppliedInlineParticipant::Boundary { level })
                .map_err(|_| {
                    crate::Error::new(
                        crate::ErrorCode::LayoutFailed,
                        "original mixed spacing level",
                    )
                })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut order = vec![0; participants.len()];
    crate::SuppliedLine::new(&participants)
        .reorder_into(&mut order)
        .map_err(|_| {
            crate::Error::new(
                crate::ErrorCode::LayoutFailed,
                "original mixed spacing order",
            )
        })?;
    Ok(order
        .into_iter()
        .filter_map(|i| match shaped.items[i] {
            ShapedItem::TextCluster(c) => Some(Member::Text(shaped.clusters[c].atoms.clone())),
            ShapedItem::InlineBox { occurrence, .. } => Some(Member::Atomic(occurrence)),
            _ => None,
        })
        .collect())
}

fn splits_region(members: &[Member], region: &Region, owner: Owner, shaped: &ShapedData) -> bool {
    let position = members.iter().position(|m| match (m, owner) {
        (Member::Text(atoms), Owner::Cluster(i)) => atoms.contains(&shaped.clusters[i].atoms.start),
        (Member::Atomic(a), Owner::Atomic(b)) => *a == b,
        _ => false,
    });
    position.is_some_and(|p| region.first <= p && p < region.last)
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

fn region_error(
    input: PresentationInput<'_>,
    graph: &Graph,
    region: &Region,
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
) -> crate::Error {
    let content = |member: &Member| match member {
        Member::Text(atoms) => {
            super::slot_pieces(input.prepared, input.presentation, atoms.clone())
                .into_iter()
                .map(|p| p.owner)
                .collect::<Vec<_>>()
        }
        Member::Atomic(occurrence) => vec![crate::SpacingPiece::Atomic {
            occurrence: *occurrence,
        }],
    };
    let mut contexts = Vec::new();
    for &i in &region.units {
        let unit = &graph.units[i];
        let pieces = unit
            .atoms
            .iter()
            .flat_map(|a| content(&Member::Text(a.clone())))
            .chain(
                unit.atomics
                    .iter()
                    .flat_map(|&a| content(&Member::Atomic(a))),
            )
            .collect();
        let locale = unit.base.and_then(|b| {
            input.prepared.styles[input.presentation.analysis.infos[b].1]
                .style
                .locale
                .as_ref()
                .map(|l| l.as_str().to_owned())
        });
        contexts.push(unit_context(unit, pieces, locale));
    }
    crate::Error::new(
        crate::ErrorCode::LayoutFailed,
        "interleaved text and atomic topology is not qualified for material spacing",
    )
    .with_detail(crate::ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        reason: SpacingFailureReason::InterleavedUnit,
        pieces: region.members.iter().flat_map(content).collect(),
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

fn original_region_error(
    analyzed: &super::super::AnalyzedData,
    graph: &Graph,
    region: &Region,
) -> crate::Error {
    let content = |member: &Member| match member {
        Member::Text(atoms) => {
            let mut pieces = Vec::new();
            for a in atoms.clone() {
                super::original_pieces(analyzed, a, &mut pieces);
            }
            pieces
        }
        Member::Atomic(occurrence) => vec![crate::SpacingPiece::Atomic {
            occurrence: *occurrence,
        }],
    };
    let contexts = region
        .units
        .iter()
        .map(|&i| {
            let unit = &graph.units[i];
            let pieces = unit
                .atoms
                .iter()
                .flat_map(|a| content(&Member::Text(a.clone())))
                .chain(
                    unit.atomics
                        .iter()
                        .flat_map(|&a| content(&Member::Atomic(a))),
                )
                .collect();
            let locale = unit.base.and_then(|b| {
                analyzed.styles[analyzed.analysis.infos[b].1]
                    .style
                    .locale_tag()
                    .map(str::to_owned)
            });
            unit_context(unit, pieces, locale)
        })
        .collect();
    crate::Error::new(
        crate::ErrorCode::LayoutFailed,
        "interleaved text and atomic topology is not qualified for material spacing",
    )
    .with_detail(crate::ErrorDetail::UnsupportedSpacing {
        stage: crate::SpacingStage::Preparation,
        operation: crate::SpacingOperation::AuthoredTracking,
        reason: SpacingFailureReason::InterleavedUnit,
        pieces: region.members.iter().flat_map(content).collect(),
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
        contexts,
        constraints: Box::new([]),
    })
}

pub(crate) fn admit(input: PresentationInput<'_>, graph: &Graph, basis: bool) -> Result<()> {
    for region in &graph.regions {
        if basis || graph.region_material(region) {
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
            ));
        }
    }
    if let Some(unit) = graph
        .units
        .iter()
        .enumerate()
        .find(|(i, u)| {
            let boundary = graph
                .boundaries
                .iter()
                .any(|b| b.before == *i || b.after == *i);
            (unqualified_unit(u) || basis && !u.justify_qualified && !u.barrier)
                && if basis {
                    !u.native_only || boundary
                } else {
                    unit_material(graph, *i)
                }
        })
        .map(|(_, u)| u)
    {
        return Err(unit_error(
            input,
            unit,
            crate::SpacingStage::Selection,
            if basis {
                crate::SpacingOperation::InterCharacterBasis
            } else {
                crate::SpacingOperation::AuthoredTracking
            },
        ));
    }
    Ok(())
}

fn unit_material(graph: &Graph, index: usize) -> bool {
    let unit = &graph.units[index];
    if !unit.native_only {
        return unit.left != 0.0 || unit.right != 0.0;
    }
    graph
        .boundaries
        .iter()
        .any(|b| (b.before == index || b.after == index) && (b.authored != 0.0 || b.mean_lost))
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
            super::original_pieces(analyzed, a, &mut pieces);
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
