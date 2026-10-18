//! Exact-generation mutable CSSOM custody remains with the exclusive DOM owner.
//! Immutable DOM captures contain neither these resources nor their debit graph.

use std::mem::{self, size_of};

use super::{
    DocumentId, DomError, NodeId,
    bank::{self, BankCustody, ControlledRecord, CustodyRegistry, Totals},
    custody::Usage,
};
use crate::style::DomStyleCssomRoot;

struct Root {
    owner: DomStyleCssomRoot,
    document: DocumentId,
}

struct DocumentRoots {
    document: DocumentId,
    count: usize,
}

/// Semantic holders drop before their corresponding custody holders. Distinct
/// prepared roots share one ledger, admitting their actual simultaneous union.
pub(super) struct Bank {
    roots: bank::Bank<Root>,
    documents: bank::Bank<DocumentRoots>,
    root_custody: BankCustody<Root>,
    document_custody: BankCustody<DocumentRoots>,
    registry: CustodyRegistry,
}

impl Drop for Bank {
    fn drop(&mut self) {
        let roots = mem::take(&mut self.roots);
        let documents = mem::take(&mut self.documents);
        let root_custody = mem::take(&mut self.root_custody);
        let document_custody = mem::take(&mut self.document_custody);
        drop(roots);
        drop(documents);
        drop(root_custody);
        drop(document_custody);
        self.registry.reconcile_releases();
    }
}

impl Clone for Bank {
    fn clone(&self) -> Self {
        Self {
            roots: self.roots.clone(),
            documents: self.documents.clone(),
            root_custody: self.root_custody.clone(),
            document_custody: self.document_custody.clone(),
            registry: self.registry.clone(),
        }
    }
}

impl Bank {
    pub fn new(registry: &CustodyRegistry) -> Self {
        Self {
            roots: bank::Bank::default(),
            documents: bank::Bank::default(),
            root_custody: BankCustody::default(),
            document_custody: BankCustody::default(),
            registry: registry.clone(),
        }
    }

    pub const fn inline_bytes() -> usize {
        size_of::<Self>()
    }

    pub fn is_empty(&self) -> bool {
        self.roots.record_count() == 0
    }

    pub fn get(&self, root: NodeId) -> Option<&DomStyleCssomRoot> {
        self.roots
            .get(root.slot)
            .filter(|record| record.owner.root() == root)
            .map(|record| &record.owner)
    }

    pub fn has_document(&self, document: DocumentId) -> bool {
        self.documents
            .get(document.node().slot)
            .is_some_and(|record| record.document == document && record.count != 0)
    }

    /// Root payloads, index paths and control records share the canonical
    /// owner's registry. Admission receives its absolute live union; CSSOM's
    /// deep mutable internals remain excluded under their producer's contract.
    pub fn prepare_insert(
        &self,
        owner: &DomStyleCssomRoot,
        document: DocumentId,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        step()?;
        // A prior generation must have relinquished its canonical custody;
        // insertion never silently replaces any actual live mutable owner.
        if self.roots.get(owner.root().slot).is_some() {
            return Err(DomError::StyleReceiptMismatch);
        }
        self.registry.reconcile_releases();
        admit(self.registry.total())?;
        let mut candidate = self.clone();
        candidate.install_root(owner, document, &mut admit)?;
        candidate.change_document_count(document, true, &mut admit)?;
        Ok(candidate)
    }

    /// Inputs come only from actual touched retirement/NodeDocument evidence.
    /// The candidate is formed lazily after an exact bound root is selected.
    pub fn prepare_publication(
        &self,
        retired: impl IntoIterator<Item = NodeId>,
        document_changes: impl IntoIterator<Item = (NodeId, DocumentId)>,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Self>, DomError> {
        self.registry.reconcile_releases();
        let mut candidate: Option<Self> = None;
        for id in retired {
            step()?;
            let current = candidate.as_ref().unwrap_or(self);
            let Some(document) = current
                .roots
                .get(id.slot)
                .filter(|record| record.owner.root() == id)
                .map(|record| record.document)
            else {
                continue;
            };
            if candidate.is_none() {
                admit(self.registry.total())?;
                candidate = Some(self.clone());
            }
            let current = candidate.as_mut().expect("selected root candidate");
            #[cfg(test)]
            super::style_custody_tests::observe(super::style_custody_tests::Site::Publication);
            let (roots, custody, removed) = current.roots.remove_controlled(
                &current.root_custody,
                id.slot,
                &current.registry,
                &mut admit,
            )?;
            current.roots = roots;
            current.root_custody = custody;
            #[cfg(test)]
            super::style_custody_tests::observe(super::style_custody_tests::Site::Retention);
            drop(removed);
            current.change_document_count(document, false, &mut admit)?;
        }
        for (id, document) in document_changes {
            step()?;
            let current = candidate.as_ref().unwrap_or(self);
            let Some(record) = current
                .roots
                .get(id.slot)
                .filter(|record| record.owner.root() == id && record.document != document)
            else {
                continue;
            };
            let former = record.document;
            let owner = self.get(id).expect("selected canonical Style resource");
            if candidate.is_none() {
                admit(self.registry.total())?;
                candidate = Some(self.clone());
            }
            let current = candidate.as_mut().expect("changed root Document candidate");
            current.install_root(owner, document, &mut admit)?;
            current.change_document_count(former, false, &mut admit)?;
            current.change_document_count(document, true, &mut admit)?;
        }
        Ok(candidate)
    }

    fn install_root(
        &mut self,
        owner: &DomStyleCssomRoot,
        document: DocumentId,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let bytes = size_of::<Root>()
            .checked_add(DomStyleCssomRoot::retained_owner_bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        #[cfg(test)]
        if self.get(owner.root()).is_some() {
            super::style_custody_tests::observe(super::style_custody_tests::Site::BindingCopy);
        }
        let record = ControlledRecord::new(
            Root {
                owner: owner.duplicate(),
                document,
            },
            Usage::new(bytes, 0),
            &self.registry,
            &mut *admit,
        )?;
        let (roots, custody) = self.roots.set_controlled(
            &self.root_custody,
            owner.root().slot,
            record,
            false,
            Totals {
                bytes,
                units: 0,
                occurrences: 1,
            },
            &self.registry,
            admit,
        )?;
        self.roots = roots;
        self.root_custody = custody;
        Ok(())
    }

    fn change_document_count(
        &mut self,
        document: DocumentId,
        add: bool,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let previous = self.documents.get(document.node().slot);
        if previous.is_some_and(|previous| previous.document != document) {
            return Err(DomError::StyleReceiptMismatch);
        }
        let count = previous.map_or(0, |previous| previous.count);
        let count = if add {
            count.checked_add(1)
        } else {
            count.checked_sub(1)
        }
        .ok_or(DomError::RepresentationExhausted)?;
        let (documents, custody) = if count == 0 {
            let (documents, custody, removed) = self.documents.remove_controlled(
                &self.document_custody,
                document.node().slot,
                &self.registry,
                admit,
            )?;
            drop(removed);
            (documents, custody)
        } else {
            let bytes = size_of::<DocumentRoots>();
            let record = ControlledRecord::new(
                DocumentRoots { document, count },
                Usage::new(bytes, 0),
                &self.registry,
                &mut *admit,
            )?;
            self.documents.set_controlled(
                &self.document_custody,
                document.node().slot,
                record,
                false,
                Totals {
                    bytes,
                    units: 0,
                    occurrences: 1,
                },
                &self.registry,
                admit,
            )?
        };
        self.documents = documents;
        self.document_custody = custody;
        Ok(())
    }
}
