//! Native SVG Tiny processing state, separate from HTML preparation.
use crate::{DomNode, NodeId};
use surgeist_html::SvgScriptFlags;

/// Actual native creation cause; neither variant claims parser provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSvgScriptOrigin {
    Native,
    Clone,
    /// Actual parser-created construction history.
    Parser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SvgScriptState {
    pub(crate) flags: SvgScriptFlags,
    pub(crate) origin: DomSvgScriptOrigin,
    qualification: ProcessingQualification,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessingQualification {
    Native(crate::document::ClosedSvgScriptProcessing),
    ParsedOffline(crate::store::parsed_installation::ParsedOfflineSvgQualification),
}
impl SvgScriptState {
    pub(crate) const fn fresh() -> Self {
        Self {
            flags: SvgScriptFlags::fresh(),
            origin: DomSvgScriptOrigin::Native,
            qualification: ProcessingQualification::Native(
                crate::document::ClosedSvgScriptProcessing::native_creation(),
            ),
        }
    }
    pub(crate) fn from_parsed(
        flags: SvgScriptFlags,
        origin: DomSvgScriptOrigin,
        qualification: &crate::store::parsed_installation::ParsedOfflineSvgQualification,
    ) -> Self {
        Self {
            flags,
            origin,
            qualification: ProcessingQualification::ParsedOffline(*qualification),
        }
    }
    pub(crate) fn mark_processed_in(
        &mut self,
        environment: crate::DomDocumentEnvironment,
    ) -> Result<(), crate::DomError> {
        // Current selected offline environment AND actual node admission cause
        // qualify this prefix. Neither absent BC nor Parser origin alone does.
        if environment.has_ever_had_navigable()
            || environment.browsing_context() != surgeist_html::HtmlBrowsingContextPresence::Absent
        {
            return Err(crate::DomError::SvgScriptProcessingUnavailable);
        }
        match self.qualification {
            ProcessingQualification::Native(proof) => proof.mark_processed(&mut self.flags),
            ProcessingQualification::ParsedOffline(_proof) => {
                // The completed immutable controller's actual ordinary/offline
                // qualification is retained, distinct from the Native witness.
                self.flags.mark_processed();
            }
        }
        Ok(())
    }
    pub(crate) const fn clone_for_element(self) -> Self {
        Self {
            flags: self.flags.clone_for_element(),
            origin: DomSvgScriptOrigin::Clone,
            qualification: self.qualification,
        }
    }
}

/// Checked SVG Script state in one canonical immutable revision.
/// This readback grants no acquisition, execution or load/error delivery.
#[derive(Clone, Copy, Debug)]
pub struct DomSvgScript<'a> {
    node: DomNode<'a>,
}
impl DomSvgScript<'_> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn flags(self) -> SvgScriptFlags {
        self.state().flags
    }
    pub fn origin(self) -> DomSvgScriptOrigin {
        self.state().origin
    }
    fn state(self) -> SvgScriptState {
        *self
            .node
            .as_element()
            .expect("checked SVG Script")
            .svg_script_state()
            .expect("checked SVG Script")
    }
}
impl<'a> DomNode<'a> {
    pub fn as_svg_script(self) -> Option<DomSvgScript<'a>> {
        self.as_element()?.svg_script_state()?;
        Some(DomSvgScript { node: self })
    }
}
