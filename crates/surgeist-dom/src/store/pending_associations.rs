//! Original parsed obligations belong to the canonical owner, not its output packet.
use super::*;
use crate::parsed_installation::{
    DomParsedHtmlAssociationDocument, DomParsedHtmlAssociationId, DomParsedHtmlBinding,
    DomParsedHtmlCapturedBinding, DomParsedHtmlPendingAssociations,
};
use surgeist_html_parser::HtmlParseReport;

struct Report {
    value: Arc<HtmlParseReport>,
    usage: custody::Usage,
    first: DomParsedHtmlAssociationId,
}
pub(super) struct Entry {
    context_basis: Option<crate::DomDocumentStamp>,
    id: DomParsedHtmlAssociationId,
    report: Arc<custody::Held<Report>>,
    documents: Vec<DomParsedHtmlAssociationDocument>,
    bindings: Arc<Vec<DomParsedHtmlBinding>>,
    captured: Arc<Vec<DomParsedHtmlCapturedBinding>>,
    payload: custody::Usage,
}
impl Entry {
    fn view(&self) -> DomParsedHtmlPendingAssociations<'_> {
        DomParsedHtmlPendingAssociations {
            context_basis: self.context_basis,
            id: self.id,
            report: &self.report.value().value,
            documents: &self.documents,
            bindings: &self.bindings,
            captured: &self.captured,
        }
    }
}
/// The one prequalified structural-installation handoff, not separately
/// constructible execution inputs.
pub(super) struct Transfer {
    pub context_basis: Option<crate::DomDocumentStamp>,
    pub id: DomParsedHtmlAssociationId,
    pub report: Arc<HtmlParseReport>,
    pub report_usage: custody::Usage,
    pub documents: Vec<DomParsedHtmlAssociationDocument>,
    pub bindings: Arc<Vec<DomParsedHtmlBinding>>,
    pub captured: Arc<Vec<DomParsedHtmlCapturedBinding>>,
}
struct Document {
    id: DocumentId,
    pending: DomParsedHtmlAssociationId,
}
/// Copies are persistent immutable membership views. They carry the same original
/// debits and do not grant consumption or native-hook execution authority.
pub(super) struct Bank {
    entries: bank::Bank<Entry>,
    documents: bank::Bank<Document>,
    entry_custody: bank::BankCustody<Entry>,
    document_custody: bank::BankCustody<Document>,
    registry: bank::CustodyRegistry,
    usage: custody::Usage,
}
impl Clone for Bank {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            documents: self.documents.clone(),
            entry_custody: self.entry_custody.clone(),
            document_custody: self.document_custody.clone(),
            registry: self.registry.clone(),
            usage: self.usage,
        }
    }
}
impl Drop for Bank {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.entries));
        drop(std::mem::take(&mut self.documents));
        drop(std::mem::take(&mut self.entry_custody));
        drop(std::mem::take(&mut self.document_custody));
        self.registry.reconcile_releases();
    }
}
impl Bank {
    pub fn new(registry: &bank::CustodyRegistry) -> Self {
        Self {
            entries: bank::Bank::default(),
            documents: bank::Bank::default(),
            entry_custody: bank::BankCustody::default(),
            document_custody: bank::BankCustody::default(),
            registry: registry.clone(),
            usage: custody::Usage::ZERO,
        }
    }
    pub fn usage(&self) -> custody::Usage {
        self.usage
    }
    pub fn iter(&self) -> impl Iterator<Item = DomParsedHtmlPendingAssociations<'_>> {
        self.entries.values().map(Entry::view)
    }
    pub fn document(&self, id: DocumentId) -> Option<DomParsedHtmlAssociationId> {
        self.documents
            .get(id.node().slot)
            .filter(|row| row.id == id)
            .map(|row| row.pending)
    }
    pub fn admit(&self, limits: DomLimits) -> Result<(), DomError> {
        Self::admit_usage(self.usage, limits)
    }
    fn admit_usage(usage: custody::Usage, limits: DomLimits) -> Result<(), DomError> {
        admit(
            usage.bytes(),
            limits.parsed_association_bytes,
            DomResource::ParsedAssociationBytes,
        )?;
        admit(
            usage.units(),
            limits.parsed_association_units,
            DomResource::ParsedAssociationUnits,
        )
    }
    pub fn prepare(
        &self,
        input: Transfer,
        limits: DomLimits,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let Transfer {
            context_basis,
            id,
            report,
            report_usage,
            documents,
            bindings,
            captured,
        } = input;
        let mut shared = None;
        for entry in self.entries.values() {
            step()?;
            if Arc::ptr_eq(&entry.report.value().value, &report) {
                shared = Some(entry.report.clone());
                break;
            }
        }
        let map_bytes = size_of::<Entry>()
            .checked_add(
                documents
                    .capacity()
                    .checked_mul(size_of::<DomParsedHtmlAssociationDocument>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .and_then(|v| {
                v.checked_add(
                    bindings
                        .capacity()
                        .checked_mul(size_of::<DomParsedHtmlBinding>())?,
                )
            })
            .and_then(|v| {
                v.checked_add(
                    captured
                        .capacity()
                        .checked_mul(size_of::<DomParsedHtmlCapturedBinding>())?,
                )
            })
            .and_then(|v| {
                v.checked_add(
                    size_of::<Vec<DomParsedHtmlBinding>>()
                        + size_of::<Vec<DomParsedHtmlCapturedBinding>>(),
                )
            })
            .ok_or(DomError::RepresentationExhausted)?;
        let payload = custody::Usage::new(map_bytes, 0);
        let new_report = if shared.is_none() {
            report_usage.checked_add(custody::Usage::new(size_of::<custody::Held<Report>>(), 0))?
        } else {
            custody::Usage::ZERO
        };
        Self::admit_usage(
            self.usage.checked_add(payload)?.checked_add(new_report)?,
            limits,
        )?;
        let registry = &self.registry;
        registry.reconcile_releases();
        let baseline = registry.total();
        // Canonical Source accounting sees the same actual record/path/report
        // union, including old and replacement bank paths during preparation.
        let mut admit_storage = |amount: custody::Usage| {
            step()?;
            // The old canonical owner stays live. Every new actual control,
            // record and replacement path is admitted before allocation.
            let extra = custody::Usage::new(
                amount.bytes().saturating_sub(baseline.bytes()),
                amount.units().saturating_sub(baseline.units()),
            );
            Self::admit_usage(self.usage.checked_add(extra)?, limits)
        };
        let report = match shared {
            Some(shared) => shared,
            None => {
                let debit = registry.ledger().retain(new_report, &mut admit_storage)?;
                Arc::new(custody::Held::new(
                    Report {
                        value: report,
                        usage: new_report,
                        first: id,
                    },
                    debit,
                ))
            }
        };
        let mut candidate = self.clone();
        for document in &documents {
            admit_storage(registry.total())?;
            let slot = document.document.node().slot;
            if let Some(old) = candidate.documents.get(slot) {
                if old.id != document.document {
                    return Err(DomError::StaleNode);
                }
                continue;
            }
            let bytes = size_of::<Document>();
            let row = bank::ControlledRecord::new(
                Document {
                    id: document.document,
                    pending: id,
                },
                custody::Usage::new(bytes, 0),
                registry,
                &mut admit_storage,
            )?;
            let (bank, custody) = candidate.documents.set_controlled(
                &candidate.document_custody,
                slot,
                row,
                false,
                bank::Totals {
                    bytes,
                    units: 0,
                    occurrences: 1,
                },
                registry,
                &mut admit_storage,
            )?;
            candidate.documents = bank;
            candidate.document_custody = custody;
        }
        let key = usize::try_from(id.stamp.revision().get())
            .map_err(|_| DomError::RepresentationExhausted)?;
        if candidate.entries.get(key).is_some() {
            return Err(DomError::RepresentationExhausted);
        }
        let record = bank::ControlledRecord::new(
            Entry {
                context_basis,
                id,
                report,
                documents,
                bindings,
                captured,
                payload,
            },
            payload,
            registry,
            &mut admit_storage,
        )?;
        let (bank, custody) = candidate.entries.set_controlled(
            &candidate.entry_custody,
            key,
            record,
            false,
            bank::Totals {
                bytes: payload.bytes(),
                units: 0,
                occurrences: 1,
            },
            registry,
            &mut admit_storage,
        )?;
        candidate.entries = bank;
        candidate.entry_custody = custody;
        // Price retained immutable backing once by actual report owner, plus
        // each destination's maps and actual sparse index backing. Shared report
        // lookup above is custody deduplication, never execution identity.
        let mut usage = custody::Usage::new(
            candidate
                .entries
                .index_charge()
                .bytes
                .checked_add(candidate.documents.index_charge().bytes)
                .ok_or(DomError::RepresentationExhausted)?
                .checked_add(candidate.documents.totals().bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            0,
        );
        for entry in candidate.entries.values() {
            admit_storage(custody::Usage::ZERO)?;
            usage = usage.checked_add(entry.payload)?;
            if entry.report.value().first == entry.id {
                usage = usage.checked_add(entry.report.value().usage)?;
            }
        }
        Self::admit_usage(usage, limits)?;
        candidate.usage = usage;
        Ok(candidate)
    }
}
impl Dom {
    /// Original parsed obligations retained independently of installation outputs.
    /// A returned immutable view grants no execution/admission authority. Each
    /// iterator step visits the finite sparse canonical bank; callers choose
    /// their own read traversal budget rather than receiving an eager scan.
    pub fn pending_parsed_html_associations(
        &self,
    ) -> impl Iterator<Item = DomParsedHtmlPendingAssociations<'_>> {
        self.pending_associations.iter()
    }
}
impl Draft {
    pub(super) fn guard_pending_associations(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        // Existing Documents alone can carry installed parsed contributions.
        let Key::Existing(id) = document else {
            return Ok(());
        };
        if let Some(pending) = self.pending_associations.document(DocumentId(id)) {
            Err(DomError::ParsedAssociationPending {
                document: DocumentId(id),
                pending,
            })
        } else {
            Ok(())
        }
    }
}
