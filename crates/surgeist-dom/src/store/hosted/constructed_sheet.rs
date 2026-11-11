//! Canonical CSSOM construction for one idle, authenticated hosted Document.
use super::*;
use crate::{DomStyleCssomRoot, DomStyleCssomRootError};
use std::{fmt::Write, sync::MutexGuard};
use surgeist_cssom::{
    CssomBatch, CssomCommitRetirement, CssomConstructorDocument, CssomEditGuard, CssomError,
    CssomInputRole, CssomInputVersion, CssomRevision, CssomSheetId, CssomSheetInit,
    CssomSheetTicket, CssomStore,
};

/// The reached owner preserves its original native or CSSOM refusal.
#[derive(Debug)]
pub enum DomHostedSheetConstructionError {
    /// The original Hosted issuer, Document or custody refused the operation.
    Hosted(DomHostedError),
    /// The actual CSSOM root could not be loaned.
    Root(DomStyleCssomRootError),
    /// The actual CSSOM staging or preparation refused the operation.
    Cssom(CssomError),
    /// The selected native URL serialization contains an isolated surrogate.
    InvalidBaseSerialization {
        /// Offset in the actual serialization.
        offset: usize,
        /// The original isolated UTF16 code unit.
        unit: u16,
    },
}

impl std::fmt::Display for DomHostedSheetConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DomHostedSheetConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Hosted(error) => Some(error),
            Self::Root(error) => Some(error),
            Self::Cssom(error) => Some(error),
            Self::InvalidBaseSerialization { .. } => None,
        }
    }
}

impl From<DomHostedError> for DomHostedSheetConstructionError {
    fn from(error: DomHostedError) -> Self {
        Self::Hosted(error)
    }
}

impl From<DomError> for DomHostedSheetConstructionError {
    fn from(error: DomError) -> Self {
        Self::Hosted(error.into())
    }
}

/// A private CSSOM batch and optional canonical attachment under the original
/// Store lock. Dropping this value publishes neither.
#[must_use]
pub struct DomHostedSheetConstruction<'d, 'r> {
    dom: &'d mut Dom,
    source: &'d DomHostedSource,
    cancel: &'d AtomicBool,
    guard: MutexGuard<'r, CssomStore>,
    candidate: Option<style_custody::Bank>,
    batch: CssomBatch,
    ticket: CssomSheetTicket,
    root: DomStyleCssomRoot,
    _frame: Retained,
}

/// A published native sheet while the original CSSOM lock and exclusive DOM
/// loan remain held for the composing owner's infallible suffix.
#[must_use]
pub struct DomHostedSheetConstructionPublished<'d, 'r> {
    _dom: &'d mut Dom,
    _source: &'d DomHostedSource,
    guard: MutexGuard<'r, CssomStore>,
    retired_cssom: CssomCommitRetirement,
    retired_bank: Option<style_custody::Bank>,
    root: DomStyleCssomRoot,
    sheet: CssomSheetId,
    revision: CssomRevision,
    _frame: Retained,
}

impl<'d, 'r> DomHostedSheetConstruction<'d, 'r> {
    /// Check the original cancellation signal at the prepared CSSOM fence.
    pub fn publish(
        self,
    ) -> Result<DomHostedSheetConstructionPublished<'d, 'r>, DomHostedSheetConstructionError> {
        let Self {
            dom,
            source,
            cancel,
            mut guard,
            candidate,
            batch,
            ticket,
            root,
            _frame,
        } = self;
        let prepared = guard
            .prepare_commit(batch)
            .map_err(DomHostedSheetConstructionError::Cssom)?;
        let sheet = prepared
            .sheet(&ticket)
            .map_err(DomHostedSheetConstructionError::Cssom)?;
        let id = sheet.id().clone();
        let revision = sheet.revision().clone();
        check_cancel(cancel)?;
        let (_, retired_cssom) = prepared.publish();
        let retired_bank =
            candidate.map(|bank| std::mem::replace(&mut dom.style_cssom_roots, bank));
        Ok(DomHostedSheetConstructionPublished {
            _dom: dom,
            _source: source,
            guard,
            retired_cssom,
            retired_bank,
            root,
            sheet: id,
            revision,
            _frame,
        })
    }
}

impl DomHostedSheetConstructionPublished<'_, '_> {
    /// Release the native loans and move out the exact publication identities.
    pub fn finish(self) -> (DomStyleCssomRoot, CssomSheetId, CssomRevision) {
        let Self {
            guard,
            retired_cssom,
            retired_bank,
            root,
            sheet,
            revision,
            ..
        } = self;
        drop(guard);
        drop(retired_cssom);
        drop(retired_bank);
        (root, sheet, revision)
    }
}

impl DomHostedSource {
    /// Stage an unlisted constructed sheet for this Global's associated
    /// Document. An existing canonical row must match the supplied producer.
    /// The current route requires idle Source and Hosted operation state.
    pub fn prepare_document_sheet<'d, 'r>(
        &'d self,
        dom: &'d mut Dom,
        global: DomHostedGlobalId,
        document: DocumentId,
        root: &'r DomStyleCssomRoot,
        init: CssomSheetInit<'_>,
        cancel: &'d AtomicBool,
    ) -> Result<DomHostedSheetConstruction<'d, 'r>, DomHostedSheetConstructionError> {
        check_cancel(cancel)?;
        dom.validate_hosted_source(self)?;
        dom.require_source_idle()?;
        // Hosted creation owns this slot before publishing a snapshot marker.
        if dom.hosted_operations.synchronous().is_some() {
            return Err(DomHostedError::IncompleteOperation.into());
        }
        if global.source() != self.id() {
            return Err(DomHostedError::ForeignSource.into());
        }
        if dom.global_document(global)? != document {
            return Err(DomHostedError::StaleIdentity.into());
        }
        if root.root() != document.node() {
            return Err(DomError::StyleReceiptMismatch.into());
        }
        let work = self.work(cancel)?;
        if !dom.hosted_document_fully_active(document, &work, cancel)? {
            return Err(DomHostedError::StaleIdentity.into());
        }
        work.step(cancel)?;
        let bind = match dom.style_cssom_roots.get(document.node()) {
            Some(existing) if !existing.same_owner(root) => {
                return Err(DomError::StyleReceiptMismatch.into());
            }
            Some(_) => false,
            None => true,
        };
        let frame_bytes = size_of::<DomHostedSheetConstruction<'_, '_>>()
            .max(size_of::<DomHostedSheetConstructionPublished<'_, '_>>());
        crate::hosted::admit(
            frame_bytes,
            self.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let frame = self.retain(frame_bytes, cancel)?;
        let snapshot = dom.snapshot();
        let view = snapshot.document(document)?;
        let base = view.document_base_record()?.serialization().code_units();
        let facts = constructor_facts(
            self,
            &work,
            document,
            snapshot.stamp(),
            base,
            frame_bytes,
            cancel,
        )?;
        drop(snapshot);
        let candidate = if bind {
            let step = || match work.step(cancel) {
                Ok(()) => Ok(()),
                Err(DomHostedError::Dom(error)) => Err(error),
                Err(DomHostedError::WorkLimit { limit }) => {
                    Err(DomError::SourceWorkLimit { limit })
                }
                Err(_) => Err(DomError::RepresentationExhausted),
            };
            let outside = dom
                .source_owner_custody_bytes()?
                .checked_add(dom.source_retained_stack_bytes()?)
                .ok_or(DomError::RepresentationExhausted)?;
            let outside = outside
                .checked_add(dom.source_dispatch_custody_bytes(step)?)
                .ok_or(DomError::RepresentationExhausted)?;
            let scratch = self
                .limits()
                .scratch_bytes()
                .checked_sub(frame_bytes)
                .and_then(|bytes| bytes.checked_sub(facts.bytes))
                .ok_or(DomError::RepresentationExhausted)?;
            let limit = self.remaining_bytes()?.min(scratch);
            let units = dom.fullscreen_custody_units()?;
            Some(dom.style_cssom_roots.prepare_insert(
                root,
                document,
                |usage| {
                    step()?;
                    let bytes = usage
                        .bytes()
                        .checked_add(outside)
                        .ok_or(DomError::RepresentationExhausted)?;
                    super::super::admit(bytes, limit, DomResource::SourceRetainedBytes)?;
                    let units = usage
                        .units()
                        .checked_add(units)
                        .ok_or(DomError::RepresentationExhausted)?;
                    super::super::admit(units, dom.limits.units, DomResource::SourceRetainedUnits)
                },
                step,
            )?)
        } else {
            None
        };
        let guard = root.lock().map_err(DomHostedSheetConstructionError::Root)?;
        let edit_guard = CssomEditGuard::from_snapshot(&guard.snapshot());
        let mut batch = guard
            .batch(edit_guard)
            .map_err(DomHostedSheetConstructionError::Cssom)?;
        let construction = batch
            .construct_sheet(facts.document, init, cancel)
            .map_err(DomHostedSheetConstructionError::Cssom)?;
        let ticket = construction.ticket().clone();
        drop(construction);
        drop(facts.retained);
        Ok(DomHostedSheetConstruction {
            dom,
            source: self,
            cancel,
            guard,
            candidate,
            batch,
            ticket,
            root: root.duplicate(),
            _frame: frame,
        })
    }
}

struct ConstructorFacts {
    document: CssomConstructorDocument,
    retained: Retained,
    bytes: usize,
}

fn constructor_facts(
    source: &DomHostedSource,
    work: &Work,
    document: DocumentId,
    stamp: DomStamp,
    base: &[u16],
    frame_bytes: usize,
    cancel: &AtomicBool,
) -> Result<ConstructorFacts, DomHostedSheetConstructionError> {
    let node = document.node();
    let identity_bytes = "stylesheet:document:".len()
        + 2
        + node.owner.get().checked_ilog10().unwrap_or(0) as usize
        + 1
        + node.slot.checked_ilog10().unwrap_or(0) as usize
        + 1
        + node.generation.checked_ilog10().unwrap_or(0) as usize
        + 1;
    let mut location_bytes = 0usize;
    visit_serialization(base, work, cancel, |scalar| {
        location_bytes = location_bytes
            .checked_add(scalar.len_utf8())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        Ok(())
    })?;
    let bytes = (2 * identity_bytes)
        .checked_add(location_bytes)
        .ok_or(DomHostedError::RepresentationExhausted)?;
    let check_scratch = |bytes: usize| {
        let bytes = frame_bytes
            .checked_add(bytes)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            bytes,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )
    };
    check_scratch(bytes)?;
    let mut retained = source.retain(bytes, cancel)?;
    let mut identity = String::new();
    let mut version_identity = String::new();
    let mut base_location = String::new();
    for (output, capacity) in [
        (&mut identity, identity_bytes),
        (&mut version_identity, identity_bytes),
        (&mut base_location, location_bytes),
    ] {
        output
            .try_reserve_exact(capacity)
            .map_err(|_| DomHostedError::AllocationFailed)?;
    }
    let actual = identity
        .capacity()
        .checked_add(version_identity.capacity())
        .and_then(|bytes| bytes.checked_add(base_location.capacity()))
        .ok_or(DomHostedError::RepresentationExhausted)?;
    check_scratch(actual)?;
    if actual > bytes {
        retained.append(source.retain(actual - bytes, cancel)?)?;
    }
    work.step(cancel)?;
    write!(
        &mut identity,
        "stylesheet:document:{}:{}:{}",
        node.owner.get(),
        node.slot,
        node.generation
    )
    .map_err(|_| DomHostedError::RepresentationExhausted)?;
    work.spend(identity.len(), cancel)?;
    version_identity.push_str(&identity);
    visit_serialization(base, work, cancel, |scalar| {
        base_location.push(scalar);
        Ok(())
    })?;
    Ok(ConstructorFacts {
        document: CssomConstructorDocument {
            version: CssomInputVersion {
                role: CssomInputRole::Document,
                identity: version_identity,
                revision: stamp.revision().get(),
            },
            identity,
            base_location,
        },
        retained,
        bytes: actual,
    })
}

fn visit_serialization(
    base: &[u16],
    work: &Work,
    cancel: &AtomicBool,
    mut visit: impl FnMut(char) -> Result<(), DomHostedError>,
) -> Result<(), DomHostedSheetConstructionError> {
    let mut offset = 0;
    let mut units = char::decode_utf16(base.iter().copied());
    while offset < base.len() {
        work.step(cancel)?;
        let decoded = units
            .next()
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let scalar =
            decoded.map_err(
                |error| DomHostedSheetConstructionError::InvalidBaseSerialization {
                    offset,
                    unit: error.unpaired_surrogate(),
                },
            )?;
        visit(scalar)?;
        offset += scalar.len_utf16();
    }
    Ok(())
}
