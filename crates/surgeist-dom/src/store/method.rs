//! The one original named-method continuation of a canonical segment.
use super::*;

#[expect(
    clippy::large_enum_variant,
    reason = "Keep the single continuation inline in the admitted Draft and checkpoint; boxing adds separately fallible allocation"
)]
pub(super) enum Frame {
    Dialog(dialog::MethodFrame),
    DesignMode(design_mode::Frame),
    FullscreenExit(Option<crate::DomFullscreenExitReturn>),
}
impl Frame {
    pub(super) fn checkpoint(
        &self,
        admission: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        match self {
            Self::FullscreenExit(result) => {
                check_cancel(cancel)?;
                Ok(Self::FullscreenExit(*result))
            }
            Self::Dialog(frame) => Ok(Self::Dialog(frame.checkpoint(admission, cancel)?)),
            Self::DesignMode(frame) => {
                check_cancel(cancel)?;
                Ok(Self::DesignMode(*frame))
            }
        }
    }
    pub(super) fn is_running(&self) -> bool {
        match self {
            Self::FullscreenExit(result) => result.is_none(),
            Self::Dialog(_) => true,
            Self::DesignMode(frame) => {
                frame.cursor.action() != surgeist_html::HtmlDesignModeAction::Complete
            }
        }
    }
}
