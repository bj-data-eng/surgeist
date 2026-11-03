//! Borrowed completed-source owner order, shared by analysis and fragment admission.
use super::{
    AnalysisDataSources, BidiClass, BidiMirroringGlyph, BidiOwner, CharInfo, anchor_kind,
    used_bidi_class, wrapping,
};
use crate::engine::scope_alignment::accounting as account;
use crate::protocol::paragraph::{ParagraphContext, ParagraphControlKind, SemanticAnchorItem};
use crate::{AnalysisTypography, InlineBox, InlineBoxKind, Result, projection::SourceProjection};

#[derive(Clone, Copy)]
pub(crate) struct Row {
    pub(crate) owner: BidiOwner,
    pub(crate) class: BidiClass,
    pub(crate) bracket: Option<(char, BidiMirroringGlyph)>,
}

/// This is the already-completed Source/Unicode facet, not another transcript.
/// The visitor allocates nothing and never resolves contextual bidi itself.
pub(crate) struct Input<'a> {
    pub(crate) typography: AnalysisTypography,
    pub(crate) projection: &'a SourceProjection,
    pub(crate) infos: &'a [(CharInfo, usize)],
    pub(crate) boxes: &'a [InlineBox],
    pub(crate) paragraph: &'a ParagraphContext,
    pub(crate) combination: Option<&'a crate::transform::combined::CombinedOutput>,
    pub(crate) events: Option<&'a [wrapping::LogicalEvent]>,
    pub(crate) anchor_positions: &'a [usize],
}

impl Input<'_> {
    pub(crate) fn visit(
        &self,
        ledger: &mut account::Ledger,
        mut emit_original: impl FnMut(Row, &mut account::Ledger) -> Result<()>,
    ) -> Result<()> {
        // Match the completed producer: only the effective stream is projected
        // to used LTR for whole-upright, before contextual UBA/direction facts.
        // Raw Unicode/control facts remain untouched in retained Analysis.
        let mut emit = |mut row: Row, ledger: &mut account::Ledger| {
            row.class = used_bidi_class(self.typography, row.class);
            emit_original(row, ledger)
        };
        if self.anchor_positions.len() != self.paragraph.anchor_order.len() {
            return Err(invalid());
        }
        for (position, pair) in self.anchor_positions.windows(2).enumerate() {
            account::work(ledger, 1)?;
            if pair[0] > pair[1] {
                let (kind, occurrence) =
                    anchor_kind(self.paragraph.anchor_order[position + 1].item);
                return Err(crate::protocol::protocol_error(
                    crate::ProtocolError::InvalidAnchorOrder {
                        position: position + 1,
                        kind,
                        occurrence,
                    },
                ));
            }
        }
        if let Some(output) = self.combination.filter(|o| !o.bindings.is_empty()) {
            let events = self.events.ok_or_else(invalid)?;
            let mut next_cell = output.bindings.iter().enumerate().peekable();
            let mut participant = 0;
            while participant <= events.len() {
                account::work(ledger, 1)?;
                if let Some(&(index, binding)) = next_cell.peek()
                    && binding.participants.start == participant
                {
                    if binding.participants.end < participant
                        || binding.participants.end > events.len()
                    {
                        return Err(invalid());
                    }
                    emit(
                        Row {
                            owner: BidiOwner::CombinedCell(index),
                            class: BidiClass::LeftToRight,
                            bracket: None,
                        },
                        ledger,
                    )?;
                    participant = binding.participants.end;
                    next_cell.next();
                    continue;
                }
                account::work(ledger, 1)?;
                let Some(event) = events.get(participant) else {
                    break;
                };
                match event.kind {
                    wrapping::Event::Text(atom) => emit(self.atom(atom, ledger)?, ledger)?,
                    wrapping::Event::Anchor(anchor) => {
                        if let Some(row) = self.anchor(anchor.item, ledger)? {
                            emit(row, ledger)?;
                        }
                    }
                    wrapping::Event::Deleted(..) => (),
                }
                participant += 1;
            }
            if next_cell.next().is_some() {
                return Err(invalid());
            }
        } else {
            let mut next_anchor = self
                .paragraph
                .anchor_order
                .iter()
                .zip(self.anchor_positions)
                .peekable();
            for index in 0..self.projection.atoms.len() {
                loop {
                    account::work(ledger, 1)?;
                    if !next_anchor.peek().is_some_and(|(_, at)| **at <= index) {
                        break;
                    }
                    let (anchor, _) = next_anchor.next().expect("peek established an anchor");
                    if let Some(row) = self.anchor(anchor.item, ledger)? {
                        emit(row, ledger)?;
                    }
                }
                emit(self.atom(index, ledger)?, ledger)?;
            }
            for (anchor, _) in next_anchor {
                if let Some(row) = self.anchor(anchor.item, ledger)? {
                    emit(row, ledger)?;
                }
            }
        }
        Ok(())
    }

    fn atom(&self, atom: usize, ledger: &mut account::Ledger) -> Result<Row> {
        account::work(ledger, 1)?;
        Ok(Row {
            owner: BidiOwner::Atom(atom),
            class: self.infos[atom].0.bidi_class,
            bracket: char::from_u32(self.projection.atoms[atom].code_point)
                .map(|ch| (ch, self.infos[atom].0.bracket)),
        })
    }

    fn anchor(
        &self,
        item: SemanticAnchorItem,
        ledger: &mut account::Ledger,
    ) -> Result<Option<Row>> {
        account::work(ledger, 1)?;
        let (cp, owner) = match item {
            SemanticAnchorItem::Atomic(index)
                if self.boxes[index].kind() == InlineBoxKind::InFlow =>
            {
                (0xfffc, BidiOwner::Atomic(index))
            }
            SemanticAnchorItem::ComputedLineBoundary(index) => {
                (0xa, BidiOwner::ComputedLineBoundary(index))
            }
            SemanticAnchorItem::Control(index) => {
                let kind = &self.paragraph.controls[index].kind;
                let cp = if *kind == ParagraphControlKind::ForcedBreak {
                    Some(0xa)
                } else {
                    kind.code_point()
                };
                let Some(cp) = cp else {
                    return Ok(None);
                };
                (cp, BidiOwner::Control(index))
            }
            _ => return Ok(None),
        };
        let sources = AnalysisDataSources;
        Ok(Some(Row {
            owner,
            class: icu_properties::CodePointMapData::<BidiClass>::new().get32(cp),
            bracket: char::from_u32(cp).map(|ch| (ch, sources.brackets().get32(cp))),
        }))
    }
}

fn invalid() -> crate::Error {
    crate::protocol::protocol_error(crate::ProtocolError::InvalidLineDirectionQuery)
}
