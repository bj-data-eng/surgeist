//! Borrowed actual source/presentation ownership for one character operation.

use super::*;
use crate::engine::{AnalyzedData, PreparedData};
use crate::protocol::prepared::PresentationUnitKind;

#[derive(Clone, Copy)]
pub(crate) struct Input<'a> {
    pub(crate) projection: &'a SourceProjection,
    pub(crate) analysis: &'a Analysis,
    pub(crate) styles: &'a [ResolvedStyleRun],
    pub(crate) boxes: &'a [InlineBox],
    pub(crate) presentation: Option<&'a SelectedPresentation>,
    analyzed: Option<&'a AnalyzedData>,
    prepared: Option<&'a PreparedData>,
    scale: f32,
    active: bool,
}

impl<'a> Input<'a> {
    /// A native scalar/atomic operation carries no combined-cell authority.
    pub(crate) fn native(
        projection: &'a SourceProjection,
        analysis: &'a Analysis,
        styles: &'a [ResolvedStyleRun],
        boxes: &'a [InlineBox],
        presentation: Option<&'a SelectedPresentation>,
    ) -> Self {
        Self {
            projection,
            analysis,
            styles,
            boxes,
            presentation,
            analyzed: None,
            prepared: None,
            scale: 1.0,
            active: false,
        }
    }
    pub(crate) fn original(
        analyzed: &'a AnalyzedData,
        styles: &'a [ResolvedStyleRun],
        scale: f32,
    ) -> Self {
        let mut input = Self::native(
            &analyzed.projection,
            &analyzed.analysis,
            styles,
            analyzed.processed.source().boxes(),
            None,
        );
        input.analyzed = Some(analyzed);
        input.scale = scale;
        input.active = analyzed
            .combination
            .as_ref()
            .is_some_and(|o| !o.bindings.is_empty());
        input
    }

    pub(crate) fn selected(
        prepared: &'a PreparedData,
        presentation: &'a SelectedPresentation,
        ledger: &mut Ledger,
    ) -> Result<Self> {
        let mut active = false;
        if prepared
            .analyzed
            .combination
            .as_ref()
            .is_some_and(|o| !o.bindings.is_empty())
        {
            storage::require_ledger(ledger)?;
            for u in &presentation.units {
                account::work(ledger, 1)?;
                if matches!(u.kind, PresentationUnitKind::Cell { .. }) {
                    active = true;
                    break;
                }
            }
        }
        let mut input = Self::native(
            &presentation.projection,
            &presentation.analysis,
            &prepared.styles,
            prepared.analyzed.processed.source().boxes(),
            Some(presentation),
        );
        input.analyzed = Some(&prepared.analyzed);
        input.prepared = Some(prepared);
        input.scale = prepared.options.scale();
        input.active = active;
        Ok(input)
    }

    pub(crate) fn has_cells(self) -> bool {
        self.active
    }

    fn analyzed(self) -> &'a AnalyzedData {
        self.analyzed
            .expect("cell operations retain their authentic analyzed source")
    }

    pub(super) fn cell_binding(self, prepared_cell: usize) -> usize {
        self.prepared
            .expect("selected cell has its actual preparation")
            .shaped
            .cells[prepared_cell]
            .binding
    }

    pub(super) fn visit_cells(
        self,
        ledger: &mut Ledger,
        mut visit: impl FnMut(usize, u8, Range<usize>, &mut Ledger) -> Result<()>,
    ) -> Result<()> {
        if !self.has_cells() {
            return Ok(());
        }
        if let Some(p) = self.presentation {
            for unit in &p.units {
                account::work(ledger, 1)?;
                if let PresentationUnitKind::Cell { cell } = unit.kind {
                    visit(
                        self.cell_binding(cell),
                        unit.line_level,
                        unit.atoms.clone(),
                        ledger,
                    )?;
                }
            }
        } else {
            let output = self.analyzed().combination.as_ref().expect("actual cells");
            for binding in 0..output.bindings.len() {
                account::work(ledger, 1)?;
                visit(
                    binding,
                    self.analyzed().analysis.cell_levels[binding],
                    output.bindings[binding].atoms.clone(),
                    ledger,
                )?;
            }
        }
        Ok(())
    }

    fn with_cell_style<T>(
        self,
        binding: usize,
        f: impl FnOnce(&crate::Style) -> Result<T>,
    ) -> Result<T> {
        let output = self
            .analyzed()
            .combination
            .as_ref()
            .expect("actual cell source");
        let scope = output.plan.groups[output.bindings[binding].group].scope;
        let topology = self
            .analyzed()
            .paragraph
            .scope_topology
            .as_ref()
            .expect("prepared cell has authenticated capture");
        let captured = crate::CapturedScope {
            capture: std::sync::Arc::clone(&topology.capture),
            index: scope,
        };
        let style = captured.style().ok_or_else(|| {
            crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::MissingStyle {
                scope,
            })
        })?;
        f(style)
    }

    pub(super) fn tracking(self, binding: usize, ledger: &mut Ledger) -> Result<f64> {
        account::work(ledger, 1)?;
        self.with_cell_style(binding, |style| {
            crate::engine::resolve::scaled_logical(
                style.letter_spacing,
                self.scale,
                false,
                "letter spacing",
            )
        })
    }

    pub(super) fn cell_pieces(
        self,
        binding: usize,
        ledger: &mut Ledger,
    ) -> Result<Vec<crate::SpacingPiece>> {
        let output = self
            .analyzed()
            .combination
            .as_ref()
            .expect("actual cell source");
        let group = &output.plan.groups[output.bindings[binding].group];
        let mut pieces = Vec::new();
        for part in &output.plan.parts[group.parts.clone()] {
            account::work(ledger, 1)?;
            for piece in &part.pieces {
                account::work(ledger, 1)?;
                let value = crate::SpacingPiece::Source {
                    origin: self
                        .analyzed()
                        .paragraph
                        .origin_map
                        .visit_view_origin(piece.origin, || account::work(ledger, 1))?
                        .expect("cell piece retains its actual view origin")
                        .get(),
                    local: piece.local,
                    aggregate: piece.aggregate,
                };
                let mut duplicate = false;
                for existing in &pieces {
                    account::work(ledger, 1)?;
                    if *existing == value {
                        duplicate = true;
                        break;
                    }
                }
                if !duplicate {
                    account::push(ledger, &mut pieces, value)?;
                }
            }
        }
        Ok(pieces)
    }

    pub(super) fn cell_locale(self, binding: usize, ledger: &mut Ledger) -> Result<Option<String>> {
        account::work(ledger, 1)?;
        self.with_cell_style(binding, |style| {
            style
                .locale
                .as_ref()
                .map(|locale| {
                    let bytes = account::clone_vec(ledger, locale.as_bytes())?;
                    Ok(String::from_utf8(bytes).expect("captured locale is UTF-8"))
                })
                .transpose()
        })
    }

    pub(super) fn set_original_order(self, units: &mut [Unit], ledger: &mut Ledger) -> Result<()> {
        if !self.has_cells() {
            for unit in units {
                unit.order = if let Some(base) = unit.base {
                    (self.projection.atoms[base].source.start, 1)
                } else {
                    (self.boxes[unit.atomics[0]].index(), 0)
                };
            }
            return Ok(());
        }
        let mut atoms = account::filled(ledger, self.projection.atoms.len(), None)?;
        let mut atomics = account::filled(ledger, self.boxes.len(), None)?;
        for (order, event) in self.analyzed().transform.events.iter().enumerate() {
            account::work(ledger, 1)?;
            match event {
                crate::transform::relation::RelationEvent::Text { atom } => {
                    atoms[*atom] = Some(order)
                }
                crate::transform::relation::RelationEvent::Anchor { order: anchor } => {
                    if let crate::protocol::paragraph::SemanticAnchorItem::Atomic(i) =
                        self.analyzed().paragraph.anchor_order[*anchor].item
                    {
                        atomics[i] = Some(order);
                    }
                }
                _ => (),
            }
        }
        for unit in units {
            account::work(ledger, 1)?;
            let order = match unit.subject {
                Subject::Text => atoms[unit.base.expect("text base")].expect("actual text event"),
                Subject::Atomics => atomics[unit.atomics[0]].expect("actual atomic event"),
                Subject::Cell(binding) => {
                    self.analyzed()
                        .combination
                        .as_ref()
                        .expect("actual cells")
                        .bindings[binding]
                        .events
                        .start
                }
            };
            unit.order = (order, 0);
        }
        account::drop_vec(ledger, atoms)?;
        account::drop_vec(ledger, atomics)
    }

    pub(super) fn binding_count(self) -> usize {
        self.analyzed
            .and_then(|a| a.combination.as_ref())
            .map_or(0, |o| o.bindings.len())
    }
}
