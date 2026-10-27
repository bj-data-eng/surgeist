//! Shared original/Runtime admission at actual public producer boundaries.
use super::*;
use std::cell::Cell;
use surgeist_dom::{
    DomParsedHtmlAssociationRuntimeUsage, DomParsedHtmlStyleRuntimeHold,
    DomParsedHtmlStyleSheetPreparation,
};

// References the existing Runtime counter and original Entry meter, never a
// replacement work owner. CSS only supplies preparation/pre/post grammar polls.
pub(super) struct Steps<'a> {
    pub(super) work: &'a Cell<u64>,
    pub(super) maximum: u64,
    pub(super) cancel: &'a AtomicBool,
}
impl Steps<'_> {
    pub(super) fn runtime(&self) -> Result<(), DomStyleRuntimeError> {
        StyleState::step_counter(self.work, self.cancel, self.maximum)
    }
    pub(super) fn prefix(
        &self,
        hold: &DomParsedHtmlStyleRuntimeHold<'_>,
    ) -> Result<(), DomStyleRuntimeError> {
        hold.step(self.cancel)?;
        self.runtime()
    }
    pub(super) fn joined(
        &self,
        prep: &DomParsedHtmlStyleSheetPreparation<'_, '_, '_>,
    ) -> Result<(), DomStyleRuntimeError> {
        prep.step(self.cancel)?;
        self.runtime()
    }
}
// Temporary/output representations beside the current Runtime bank. CSS/CSSOM
// own AST/grammar storage; these are actual holders/names/labels, not AST tariffs.
pub(super) struct Custody {
    pub(super) bytes: usize,
    pub(super) shared: usize,
    pub(super) units: usize,
    pub(super) current: usize,
    pub(super) maximum: usize,
    pub(super) outputs: usize,
}
impl Custody {
    pub(super) fn usage(
        &self,
    ) -> Result<DomParsedHtmlAssociationRuntimeUsage, DomStyleRuntimeError> {
        Ok(DomParsedHtmlAssociationRuntimeUsage::new(
            self.bytes - self.shared,
            self.units,
            self.outputs,
        )?)
    }
    pub(super) fn check(&self) -> Result<(), DomStyleRuntimeError> {
        if self
            .current
            .checked_add(self.bytes)
            .is_none_or(|n| n > self.maximum)
        {
            return Err(DomStyleRuntimeError::Admission);
        }
        Ok(())
    }
    pub(super) fn add(&mut self, bytes: usize) -> Result<(), DomStyleRuntimeError> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.check()
    }
    pub(super) fn admit(
        &self,
        prep: &mut DomParsedHtmlStyleSheetPreparation<'_, '_, '_>,
        steps: &Steps<'_>,
    ) -> Result<(), DomStyleRuntimeError> {
        self.check()?;
        steps.joined(prep)?;
        prep.admit_runtime(self.usage()?, steps.cancel)?;
        Ok(())
    }
    pub(super) fn string(
        &mut self,
        requested: usize,
        prep: &mut DomParsedHtmlStyleSheetPreparation<'_, '_, '_>,
        steps: &Steps<'_>,
    ) -> Result<String, DomStyleRuntimeError> {
        self.add(requested)?;
        self.admit(prep, steps)?;
        let mut value = String::new();
        value
            .try_reserve_exact(requested)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        self.add(
            value
                .capacity()
                .checked_sub(requested)
                .ok_or(DomStyleRuntimeError::Admission)?,
        )?;
        self.admit(prep, steps)?;
        Ok(value)
    }
}
pub(super) fn current_storage(
    state: &StyleState,
    hold: &mut DomParsedHtmlStyleRuntimeHold<'_>,
    custody: &Custody,
    steps: &Steps<'_>,
) -> Result<usize, DomStyleRuntimeError> {
    let headers = state.storage_headers(None, || steps.prefix(hold))?;
    let remaining = steps.maximum.saturating_sub(steps.work.get());
    let mut fault = None;
    let mut admit = |action| {
        if fault.is_some() {
            return false;
        }
        let result = match action {
            surgeist_css::source::CssSourceRetentionAction::Step => steps.prefix(hold),
            surgeist_css::source::CssSourceRetentionAction::Scratch { live_bytes } => (|| {
                steps.prefix(hold)?;
                let bytes = custody
                    .bytes
                    .checked_add(live_bytes)
                    .ok_or(DomStyleRuntimeError::Admission)?;
                hold.admit_runtime(
                    DomParsedHtmlAssociationRuntimeUsage::new(
                        bytes,
                        custody.units,
                        custody.outputs,
                    )?,
                    steps.cancel,
                )?;
                Ok(())
            })(
            ),
        };
        match result {
            Ok(()) => true,
            Err(e) => {
                fault = Some(e);
                false
            }
        }
    };
    let result = (|| {
        let mut sources = CssSourceRetention::with_admission(
            usize::MAX,
            usize::try_from(remaining).unwrap_or(usize::MAX),
            usize::try_from(remaining).unwrap_or(usize::MAX),
            &mut admit,
        )
        .map_err(|_| DomStyleRuntimeError::Admission)?;
        state.processing(None, |value| {
            sources
                .charge_step()
                .map_err(|_| DomStyleRuntimeError::Admission)?;
            for diagnostic in value.diagnostics() {
                sources
                    .visit(diagnostic)
                    .map_err(|_| DomStyleRuntimeError::Admission)?;
            }
            Ok(())
        })?;
        sources
            .complete_payload_bytes_admitted()
            .map_err(|_| DomStyleRuntimeError::Admission)
    })();
    if let Some(error) = fault {
        return Err(error);
    }
    headers
        .checked_add(result?)
        .ok_or(DomStyleRuntimeError::Admission)
}
