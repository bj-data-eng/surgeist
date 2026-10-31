//! Captured output authority, separate from used paragraph facts.
use crate::processing::contextual::ScopeCapture;
use crate::{CapturedJointFlow, CapturedParagraph, CapturedScope};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) enum CapturedOwner {
    Raw(CapturedParagraph),
    Joint(CapturedJointFlow),
}
impl CapturedOwner {
    pub(crate) fn capture(&self) -> &Arc<ScopeCapture> {
        match self {
            Self::Raw(raw) => &raw.capture,
            Self::Joint(joint) => &joint.capture,
        }
    }
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Raw(_) => 1,
            Self::Joint(joint) => joint.capture.joint_flows[joint.index].members.len(),
        }
    }
    pub(crate) fn member(&self, ordinal: usize) -> CapturedParagraph {
        match self {
            Self::Raw(raw) => {
                assert_eq!(ordinal, 0, "admitted raw member");
                raw.clone()
            }
            Self::Joint(joint) => CapturedParagraph {
                capture: Arc::clone(&joint.capture),
                index: joint.capture.joint_flows[joint.index].members[ordinal].paragraph,
            },
        }
    }
    pub(crate) fn facts_owner(&self) -> CapturedParagraph {
        match self {
            Self::Raw(raw) => raw.clone(),
            Self::Joint(joint) => joint.facts_owner(),
        }
    }
    pub(crate) fn domain(&self) -> CapturedScope {
        match self {
            Self::Raw(raw) => raw.domain(),
            Self::Joint(joint) => joint.domain(),
        }
    }
    pub(crate) fn owns_cut(&self, cut: usize) -> bool {
        let capture = self.capture();
        match self {
            Self::Raw(raw) => capture
                .events
                .get(cut)
                .is_some_and(|event| event.paragraph == Some(raw.index)),
            Self::Joint(joint) => {
                let data = &capture.joint_flows[joint.index];
                let first = capture.paragraphs[data.members[0].paragraph].enter;
                let last = capture.paragraphs[data.members[data.members.len() - 1].paragraph].exit;
                first <= cut
                    && cut <= last
                    && capture.flow_orders[data.domain].binary_search(&cut).is_ok()
            }
        }
    }
    pub(crate) fn contains_paragraph(&self, paragraph: usize) -> bool {
        (0..self.len()).any(|ordinal| self.member(ordinal).index == paragraph)
    }
}

/// Borrowed observation of the genuine captured authority of one output.
/// Member and facts-owner handles retain their original immutable capture.
#[derive(Clone, Copy)]
pub struct CapturedAnalysisOwner<'a> {
    pub(crate) owner: &'a CapturedOwner,
}
impl CapturedAnalysisOwner<'_> {
    pub fn members(&self) -> impl ExactSizeIterator<Item = CapturedParagraph> + '_ {
        (0..self.owner.len()).map(|ordinal| self.owner.member(ordinal))
    }
    #[must_use]
    pub fn facts_owner(&self) -> CapturedParagraph {
        self.owner.facts_owner()
    }
    #[must_use]
    pub fn joint_flow(&self) -> Option<CapturedJointFlow> {
        match self.owner {
            CapturedOwner::Raw(_) => None,
            CapturedOwner::Joint(joint) => Some(joint.clone()),
        }
    }
    #[must_use]
    pub fn domain(&self) -> CapturedScope {
        self.owner.domain()
    }
}
impl std::fmt::Debug for CapturedAnalysisOwner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturedAnalysisOwner")
            .field("owner", self.owner)
            .finish()
    }
}
