//! Authenticated cut facts for a consumer's box-owned wrapping policy.

use std::sync::Arc;

use super::{
    PreparedBoundary, PreparedControl, PreparedText, ProtocolError, ProtocolHandleKind,
    protocol_error, selection::require_prepared,
};
use crate::engine::analysis::scope::ScopeCutIndex;
use crate::{CapturedScope, Result};

/// Why canonical scope incidence cannot be established for this request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WrappingScopeQueryError {
    /// This preparation has no contextual capture authority.
    MissingCapture,
    /// The scope belongs to a different immutable capture.
    ForeignCapture,
    /// The requested occurrence is a domain, rather than an inline box.
    NonInlineScope,
    /// The scope belongs to another lexical domain.
    WrongDomain,
    /// The executed correspondence does not represent this requested incidence.
    UnrepresentedIncidence,
}

fn refusal(reason: WrappingScopeQueryError) -> crate::Error {
    protocol_error(ProtocolError::WrappingScopeQuery { reason })
}

impl PreparedText {
    /// Whether this exact issued mixed prefix has consumed the actual control.
    /// All representable control kinds participate, including a Wbr transformed
    /// into real text. This observation grants no break or placement authority.
    pub fn boundary_consumes_control(
        &self,
        boundary: &PreparedBoundary,
        control: &PreparedControl,
    ) -> Result<bool> {
        require_prepared(
            self,
            &boundary.prepared,
            ProtocolHandleKind::Boundary,
            boundary.index,
        )?;
        require_prepared(
            self,
            &control.prepared,
            ProtocolHandleKind::Control,
            control.index,
        )?;
        let control = self.inner.tables.controls[control.index]
            .participant
            .ok_or_else(|| refusal(WrappingScopeQueryError::UnrepresentedIncidence))?;
        Ok(control < self.inner.tables.boundaries[boundary.index].participant)
    }

    /// Whether this original control cut survives at a genuine emitted
    /// typographic-unit boundary. Pure coordinate translation remains exact;
    /// actual snapping or a straddling replacement returns false. Missing
    /// represented control output is an error rather than assumed exactness.
    pub fn control_is_boundary_exact(&self, control: &PreparedControl) -> Result<bool> {
        require_prepared(
            self,
            &control.prepared,
            ProtocolHandleKind::Control,
            control.index,
        )?;
        let row = &self.inner.tables.controls[control.index];
        let atom = row
            .semantic_atom
            .ok_or_else(|| refusal(WrappingScopeQueryError::UnrepresentedIncidence))?;
        Ok(crate::engine::analysis::scope::anchor_exact(
            &self.inner.analyzed.core,
            row.anchor,
            atom,
        ))
    }

    /// Whether the actual flow cut lies strictly inside this captured inline
    /// box. Logical capture and executed correspondence remain authoritative
    /// even when no physical endpoint markers were requested. Foreign or
    /// unrepresented context is an error, never an assumed outside result.
    pub fn boundary_inside_captured_scope(
        &self,
        boundary: &PreparedBoundary,
        scope: &CapturedScope,
    ) -> Result<bool> {
        require_prepared(
            self,
            &boundary.prepared,
            ProtocolHandleKind::Boundary,
            boundary.index,
        )?;
        let analyzed = &self.inner.analyzed;
        let topology = analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .ok_or_else(|| refusal(WrappingScopeQueryError::MissingCapture))?;
        if !Arc::ptr_eq(&topology.capture, &scope.capture) {
            return Err(refusal(WrappingScopeQueryError::ForeignCapture));
        }
        let fact = &scope.capture.scopes[scope.index];
        if fact.definition.is_some() {
            return Err(refusal(WrappingScopeQueryError::NonInlineScope));
        }
        if fact.domain != topology.domain {
            return Err(refusal(WrappingScopeQueryError::WrongDomain));
        }
        let index = analyzed
            .scope_cuts
            .get_or_init(|| ScopeCutIndex::new(&analyzed.core))
            .as_ref()
            .map_err(|error| error.error())?;
        index
            .inside(
                self.inner.tables.boundaries[boundary.index].relation_cut,
                fact.enter,
                fact.exit,
            )
            .ok_or_else(|| refusal(WrappingScopeQueryError::UnrepresentedIncidence))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn prepare(text: &str, at: usize, kind: ParagraphControlKind) -> PreparedText {
        let source = Source::new(text);
        let input = ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                ..Style::default()
            },
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![ParagraphOrigin {
                source: source.clone(),
                local: Range::new(0, source.text().len()),
                aggregate: Range::new(0, source.text().len()),
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            }],
            connections: vec![],
            controls: vec![ParagraphControl { at, kind }],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![ParagraphAnchor {
                item: ParagraphAnchorItem::Control(0),
                affinity: Affinity::After,
            }],
        };
        let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap()
    }

    #[test]
    fn control_consumption_observes_real_mixed_slot_for_all_zero_output_kinds() {
        for kind in [
            ParagraphControlKind::InlineBoundaryStart,
            ParagraphControlKind::InlineBoundaryEnd,
            ParagraphControlKind::ForcedBreak,
            ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
        ] {
            let prepared = prepare("AB", 1, kind);
            let control = prepared.control_views().next().unwrap().handle();
            assert!(prepared.control_is_boundary_exact(&control).unwrap());
            for boundary in prepared.boundary_views() {
                assert_eq!(
                    prepared
                        .boundary_consumes_control(&boundary.handle(), &control)
                        .unwrap(),
                    boundary.participant_offset() >= 2
                );
            }
        }
    }

    #[test]
    fn projection_of_a_real_inside_grapheme_control_is_not_boundary_exact() {
        let prepared = prepare("A\u{301}B", 1, ParagraphControlKind::InlineBoundaryStart);
        let control = prepared.control_views().next().unwrap().handle();
        assert!(!prepared.control_is_boundary_exact(&control).unwrap());
        let foreign = prepare("AB", 1, ParagraphControlKind::InlineBoundaryStart);
        assert!(matches!(
            prepared
                .control_is_boundary_exact(&foreign.control_views().next().unwrap().handle())
                .unwrap_err()
                .detail(),
            Some(ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::ForeignPreparation {
                    kind: ProtocolHandleKind::Control,
                    ..
                }
            })
        ));
    }
}
