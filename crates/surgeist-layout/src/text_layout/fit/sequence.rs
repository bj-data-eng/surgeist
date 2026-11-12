use super::super::{formatted, roles};
use super::*;

pub(super) struct AcceptedCandidate {
    pub(super) candidate: EvaluatedCandidate,
    pub(super) role_start: usize,
    pub(super) box_use_start: usize,
    pub(super) zone_basis: ZoneBasis,
    pub(super) phrase_basis: Vec<PhraseBasis>,
}

impl<B, C, A, R, X> CandidateEnvironment<B, C, A, R, X>
where
    B: FnMut(f64, f64) -> Result<Band, FitError>,
    C: FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    A: FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
    R: FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
    X: FnMut(EncounterReset) -> Result<(), FitError>,
{
    pub(super) fn accept_line(
        &mut self,
        input: &FitInput<'_>,
        progress: &mut SequenceEntry,
        accepted: AcceptedCandidate,
        accept_encounters: &mut impl FnMut() -> Result<(), FitError>,
        clear_source: &mut impl FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
        result: &mut Vec<FittedLine>,
    ) -> Result<LineEnd, FitError> {
        let AcceptedCandidate {
            candidate,
            role_start,
            box_use_start,
            zone_basis,
            phrase_basis,
        } = accepted;
        // Deferred replay above may have been a no-op. Restore the exact
        // candidate once more before any clear/accept operation observes it.
        self.restore(input, &candidate)?;
        let logical_end = finite(
            candidate.frame.logical_start + candidate.disposition.height(&candidate.finalized),
        )?;
        let cleared = if candidate.disposition.is_formatted() {
            clear_source(logical_end, &candidate.request)?
                .map(finite)
                .transpose()?
                .map_or(logical_end, |minimum| minimum.max(logical_end))
        } else {
            logical_end
        };
        progress.minimum_block = candidate.frame.minimum_block;
        progress.phase = candidate.frame.phase;
        if cleared > logical_end {
            progress.minimum_block = Some(
                progress
                    .minimum_block
                    .map_or(cleared, |minimum| minimum.max(cleared)),
            );
        }
        let ending = candidate.request.context.end;
        progress.boundary = candidate.request.end.clone();
        let first_formatted = progress.indent_state.first_formatted();
        if candidate.disposition.is_formatted() {
            progress.indent_state.commit(ending == LineEnd::Forced);
            progress
                .line_box_context
                .commit(candidate.request.context.line_box_direction);
        }
        accept_encounters()?;
        if crate::layout_math::normal_available_size(input.available).is_some()
            && candidate.disposition.is_formatted()
        {
            progress
                .hyphenation_state
                .commit(candidate.request.alternative.is_some());
        }
        // Trials and hypothetical manual portions do not mutate this
        // state; only the accepted outgoing ending replaces it.
        progress.incoming = candidate.finalized.hyphenation_continuation();
        let available = candidate.finalized.metric_bounds().size.width;
        let mut role_copies = formatted::EntryCopies::default();
        let mut role_uses =
            role_copies.rows_ref(input.entry_funding, self.roles.len() - role_start)?;
        roles::visit(input.entry_funding, self.roles.len() - role_start)?;
        role_uses.extend(self.roles.drain(role_start..));
        let role_uses = roles::retained_rows(role_uses, role_copies, input.entry_funding)?;
        result.push(FittedLine {
            disposition: candidate.disposition,
            boxes: candidate.boxes,
            box_uses: self.box_uses.split_off(box_use_start),
            first_formatted,
            role_uses,
            line_box_end: progress.line_box_context,
            zone_basis,
            phrase_basis,
            hyphenation_end: progress.hyphenation_state,
            encounters: candidate.encounters,
            finalized: candidate.finalized,
            amounts: candidate.amounts,
            scopes: candidate.scopes,
            natural_advance: candidate.natural_advance,
            program: None,
            inline_start: candidate.bands.combined().start,
            placed_start: candidate.placed_start,
            available,
            bands: candidate.bands,
            flow: LineFlow {
                frame: candidate.frame,
                end: logical_end,
                next: cleared,
            },
        });
        progress.logical_start = cleared;
        Ok(ending)
    }
}
