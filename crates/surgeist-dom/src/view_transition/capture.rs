//! Compact historical CSSOM qualifications, without retaining the stylesheet graph.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use surgeist_cssom::{CssomError, CssomParent, CssomRuleData, CssomSnapshot};

/// Finite preparation of one fixed-size structural View Transition input.
/// Work admits each producer read and each rule-list identity comparison before
/// it happens. This does not replace the CSSOM owner's graph limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomViewTransitionCaptureLimits {
    retained_bytes: usize,
    work: u64,
}
impl DomViewTransitionCaptureLimits {
    pub const fn new(retained_bytes: usize, work: u64) -> Self {
        Self {
            retained_bytes,
            work,
        }
    }
}
/// Existence of this exact sheet in the captured owner/revision. No whole CSSOM
/// snapshot, UA placement, rendering resource or current readiness is retained.
#[derive(Clone, Debug)]
pub struct DomPreparedViewTransitionSheet {
    pub(crate) sheet: CssomSheetId,
    pub(crate) revision: CssomRevision,
}
impl DomPreparedViewTransitionSheet {
    pub fn capture(
        snapshot: &CssomSnapshot,
        sheet: &CssomSheetId,
        limits: DomViewTransitionCaptureLimits,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let mut capture = Capture::new::<Self>(limits, cancel)?;
        capture.step()?;
        snapshot.sheet(sheet).map_err(cssom_error)?;
        capture.step()?;
        Ok(Self {
            sheet: sheet.clone(),
            revision: snapshot.revision().clone(),
        })
    }
    pub fn sheet(&self) -> &CssomSheetId {
        &self.sheet
    }
    pub fn revision(&self) -> &CssomRevision {
        &self.revision
    }
}
/// Exact selected rule qualifications at one historical revision. Private
/// formation checks sheet membership, direct parents, kinds, uniqueness and
/// old-before-new ordering. Apply separately checks the actual DOM branch shape.
#[derive(Clone, Debug)]
pub struct DomPreparedViewTransitionRules {
    pub(crate) sheet: CssomSheetId,
    pub(crate) revision: CssomRevision,
    pub(crate) associations: DomViewTransitionRuleAssociations,
}
impl DomPreparedViewTransitionRules {
    pub fn capture(
        snapshot: &CssomSnapshot,
        sheet: &CssomSheetId,
        associations: DomViewTransitionRuleAssociations,
        limits: DomViewTransitionCaptureLimits,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let mut capture = Capture::new::<Self>(limits, cancel)?;
        capture.step()?;
        let sheet_value = snapshot.sheet(sheet).map_err(cssom_error)?;
        capture.step()?;
        let list = snapshot.rules(sheet_value.rules()).map_err(cssom_error)?;
        let (old, new) = match &associations.image_animation {
            DomViewTransitionImageRules::Empty => (None, None),
            DomViewTransitionImageRules::Old(id) => (Some(id), None),
            DomViewTransitionImageRules::New(id) => (None, Some(id)),
            DomViewTransitionImageRules::OldThenNew { old, new } => (Some(old), Some(new)),
        };
        let ids = [
            associations.group_keyframes.as_ref(),
            associations.group_animation_name.as_ref(),
            associations.group_styles.as_ref(),
            associations.image_pair_isolation.as_ref(),
            old,
            new,
        ];
        let mut positions = [None; 6];
        for (index, id) in list
            .iter()
            .enumerate()
            .take(if ids.iter().all(Option::is_none) {
                0
            } else {
                list.len()
            })
        {
            capture.step()?;
            for (slot, selected) in ids.iter().enumerate() {
                capture.step()?;
                if selected.is_some_and(|selected| selected == id) {
                    positions[slot] = Some(index);
                }
            }
        }
        for (index, selected) in ids.iter().enumerate() {
            capture.step()?;
            let Some(id) = selected else { continue };
            capture.step()?;
            let rule = snapshot.rule(id).map_err(cssom_error)?;
            if !matches!(
                (index, rule.data()),
                (0, CssomRuleData::Keyframes { .. }) | (1..=5, CssomRuleData::Style { .. })
            ) {
                return Err(DomError::ViewTransitionCssom(
                    DomViewTransitionCssomError::WrongKind,
                ));
            }
            if !matches!(rule.parent(), CssomParent::Sheet(parent) if parent == sheet)
                || positions[index].is_none()
            {
                return Err(DomError::ViewTransitionCssom(
                    DomViewTransitionCssomError::WrongSheet,
                ));
            }
            for previous in ids[..index].iter().flatten() {
                capture.step()?;
                if *previous == *id {
                    return Err(DomError::ViewTransitionCssom(
                        DomViewTransitionCssomError::DuplicateRule,
                    ));
                }
            }
        }
        if old.is_some() && new.is_some() && positions[4] >= positions[5] {
            return Err(DomError::ViewTransitionCssom(
                DomViewTransitionCssomError::RuleOrder,
            ));
        }
        capture.step()?;
        Ok(Self {
            sheet: sheet.clone(),
            revision: snapshot.revision().clone(),
            associations,
        })
    }
    pub fn sheet(&self) -> &CssomSheetId {
        &self.sheet
    }
    pub fn revision(&self) -> &CssomRevision {
        &self.revision
    }
    pub fn associations(&self) -> &DomViewTransitionRuleAssociations {
        &self.associations
    }
}
struct Capture<'a> {
    limits: DomViewTransitionCaptureLimits,
    work: u64,
    cancel: &'a AtomicBool,
}
impl<'a> Capture<'a> {
    fn new<T>(
        limits: DomViewTransitionCaptureLimits,
        cancel: &'a AtomicBool,
    ) -> Result<Self, DomError> {
        if cancel.load(Ordering::Acquire) {
            return Err(DomError::Cancelled);
        }
        crate::store::admit(
            std::mem::size_of::<T>(),
            limits.retained_bytes,
            crate::DomResource::ViewTransitionCaptureBytes,
        )?;
        Ok(Self {
            limits,
            work: 0,
            cancel,
        })
    }
    fn step(&mut self) -> Result<(), DomError> {
        if self.cancel.load(Ordering::Acquire) {
            return Err(DomError::Cancelled);
        }
        let work = self
            .work
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if work > self.limits.work {
            return Err(DomError::ViewTransitionCaptureWorkLimit {
                limit: self.limits.work,
            });
        }
        self.work = work;
        Ok(())
    }
}
fn cssom_error(error: CssomError) -> DomError {
    DomError::ViewTransitionCssom(match error {
        CssomError::ForeignOwner => DomViewTransitionCssomError::ForeignOwner,
        _ => DomViewTransitionCssomError::MissingObject,
    })
}
