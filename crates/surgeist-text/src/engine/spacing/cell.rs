//! Indexed, operation-local Cell consumers of the canonical Native spacing owner.
use super::*;

/// Classified once over actual completed used atoms; never retained with output.
pub(crate) struct CellWordPlan {
    candidates: Vec<SeparatorCandidate>,
    ranges: Vec<AtomRange<usize>>,
}
impl CellWordPlan {
    pub(crate) fn new(
        projection: &SourceProjection,
        analysis: &Analysis,
        styles: &[ResolvedStyleRun],
        atom_cells: &[Option<usize>],
        cell_count: usize,
        ledger: &mut Ledger,
    ) -> Result<Self> {
        if atom_cells.len() != projection.atoms.len() {
            return Err(invalid_word_execution());
        }
        let mut demand = false;
        for (atom, owner) in atom_cells.iter().enumerate() {
            account::work(ledger, 1)?;
            if owner.is_some() && styles[analysis.infos[atom].1].style.word_spacing != 0.0 {
                demand = true;
                break;
            }
        }
        let mut result = Self {
            candidates: Vec::new(),
            ranges: Vec::new(),
        };
        if !demand {
            return Ok(result);
        }
        let mut previous: Option<usize> = None;
        visit_candidates(
            projection,
            analysis,
            &BTreeSet::new(),
            ledger,
            |candidate, ledger| {
                account::work(ledger, 1)?;
                let Some(owner) = atom_cells[candidate.base] else {
                    return Ok(());
                };
                if styles[analysis.infos[candidate.base].1].style.word_spacing == 0.0 {
                    return Ok(());
                }
                if owner >= cell_count || previous.is_some_and(|old| owner < old) {
                    return Err(invalid_word_execution());
                }
                for atom in candidate.atoms.clone() {
                    account::work(ledger, 1)?;
                    if atom_cells[atom] != Some(owner) {
                        return Err(invalid_word_execution());
                    }
                }
                if result.ranges.is_empty() {
                    result.ranges = account::filled(ledger, cell_count, 0..0)?;
                }
                if previous != Some(owner) {
                    result.ranges[owner].start = result.candidates.len();
                }
                account::push(ledger, &mut result.candidates, candidate)?;
                result.ranges[owner].end = result.candidates.len();
                previous = Some(owner);
                Ok(())
            },
        )?;
        Ok(result)
    }
    pub(crate) fn for_cell(&self, cell: usize) -> &[SeparatorCandidate] {
        if self.ranges.is_empty() {
            &[]
        } else {
            &self.candidates[self.ranges[cell].clone()]
        }
    }
    pub(crate) fn drop(self, ledger: &mut Ledger) -> Result<()> {
        account::drop_vec(ledger, self.candidates)?;
        account::drop_vec(ledger, self.ranges)
    }
}

/// Actual per-packet receipts. They die after visual attribution; the chosen
/// positions retain only the executed half and unit-start operands.
pub(crate) struct CellWordExecution {
    units: Vec<WordSpacingUnit>,
}
impl CellWordExecution {
    pub(crate) fn new(
        analyzed: &AnalyzedData,
        styles: &[ResolvedStyleRun],
        shaped: &mut ShapedData,
        atoms: AtomRange<usize>,
        clusters: AtomRange<usize>,
        candidates: &[SeparatorCandidate],
        ledger: &mut Ledger,
    ) -> Result<Self> {
        let mut units = Vec::new();
        if candidates.is_empty() {
            return Ok(Self { units });
        }
        let mut incidence = account::filled(ledger, atoms.len(), None)?;
        for index in clusters {
            account::work(ledger, 1)?;
            let range = &shaped.clusters[index].atoms;
            if range.start < atoms.start || range.end > atoms.end {
                return Err(invalid_word_execution());
            }
            for atom in range.clone() {
                account::work(ledger, 1)?;
                let slot = &mut incidence[atom - atoms.start];
                if slot.replace(index).is_some() {
                    return Err(invalid_word_execution());
                }
            }
        }
        for candidate in candidates {
            account::work(ledger, 1)?;
            if candidate.atoms.start < atoms.start || candidate.atoms.end > atoms.end {
                return Err(invalid_word_execution());
            }
            let mut members = Vec::new();
            for atom in candidate.atoms.clone() {
                account::work(ledger, 1)?;
                if let Some(cluster) = incidence[atom - atoms.start] {
                    account::push(ledger, &mut members, cluster)?;
                }
            }
            if members.is_empty() {
                return Err(invalid_word_execution());
            }
            let authored = styles[analyzed.analysis.infos[candidate.base].1]
                .style
                .word_spacing;
            let unit = authored_unit_accounted(
                shaped,
                candidate,
                &members,
                authored,
                SpacingStage::Preparation,
                || pieces(analyzed, candidate.atoms.clone()),
                ledger,
            )?;
            account::drop_vec(ledger, members)?;
            account::push(ledger, &mut units, unit)?;
        }
        account::drop_vec(ledger, incidence)?;
        Ok(Self { units })
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub(crate) fn place(
        self,
        clusters: &mut [PreparedCluster],
        first_cluster: usize,
        ranks: &[usize],
        ledger: &mut Ledger,
    ) -> Result<Vec<WordSpacingPlacement>> {
        if self.units.is_empty() {
            return Ok(Vec::new());
        }
        // Conservative admission covers the canonical attribution's one unit
        // walk and one component walk before it starts; Native keeps its owner.
        account::work(ledger, self.units.len())?;
        let visits = self.units.iter().try_fold(self.units.len(), |n, unit| {
            n.checked_add(unit.components.len()).ok_or_else(|| {
                super::super::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
                    kind: crate::ScopeAlignmentLimitKind::StructuralWork,
                })
            })
        })?;
        account::work(ledger, visits)?;
        let dispositions = account::filled(
            ledger,
            clusters.len(),
            SpacingSlotDisposition::Retained { trailing: false },
        )?;
        let halves = visual_units_accounted(
            clusters,
            first_cluster,
            &self.units,
            ranks,
            &dispositions,
            TextNumericPhase::Shaping,
            ledger,
        )?;
        account::drop_vec(ledger, dispositions)?;
        let mut units = self.units;
        for unit in units.drain(..) {
            account::work(ledger, 1)?;
            account::drop_vec(ledger, unit.components)?;
        }
        account::drop_vec(ledger, units)?;
        Ok(halves)
    }
}

fn pieces(analyzed: &AnalyzedData, atoms: AtomRange<usize>) -> Result<Vec<SpacingPiece>> {
    let mut result = Vec::new();
    for atom in atoms {
        original_pieces(analyzed, atom, &mut result, &mut None)?;
    }
    Ok(result)
}
