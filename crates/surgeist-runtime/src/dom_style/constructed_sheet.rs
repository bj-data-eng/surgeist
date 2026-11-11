//! Native construction joins the current Hosted, DOM and Style owners.
use super::*;
use crate::dom_lifecycle::{DomHostedRuntimeError, DomHostedTopLevel, HostedRuntime};
use std::rc::Rc;
use surgeist_cssom::{CssomLimits, CssomRevision, CssomSheetInit, CssomStore};
use surgeist_dom::{DocumentId, DomHostedSheetConstructionError};

/// Native Style policy used only when this Document has no producer yet.
pub struct DomConstructedSheetOwnerInit<'a> {
    /// Nonempty unique label used by the native Style integration.
    pub label: &'a str,
    /// Finite policy for the new actual CSSOM store.
    pub limits: CssomLimits,
}

/// Exact native publication identities and access to the original producer.
#[derive(Debug)]
pub struct DomConstructedSheet {
    owner: DomStyleCssomRoot,
    document: DocumentId,
    sheet: CssomSheetId,
    revision: CssomRevision,
}

impl DomConstructedSheet {
    /// The exact producer retained by DOM and Runtime at publication.
    pub fn owner(&self) -> &DomStyleCssomRoot {
        &self.owner
    }

    /// The authenticated constructor Document.
    pub fn document(&self) -> DocumentId {
        self.document
    }

    /// The actual constructed sheet identity.
    pub fn sheet(&self) -> &CssomSheetId {
        &self.sheet
    }

    /// The CSSOM revision which published this sheet.
    pub fn revision(&self) -> &CssomRevision {
        &self.revision
    }
}

/// Refusal from the actual reached native owner.
#[derive(Debug)]
pub enum DomConstructedSheetError {
    /// The Hosted runtime could not qualify the Window.
    Hosted(DomHostedRuntimeError),
    /// The original Style runtime could not admit or identify the owner.
    Style(DomStyleRuntimeError),
    /// The native Document or CSSOM producer refused construction.
    Construction(DomHostedSheetConstructionError),
}

impl std::fmt::Display for DomConstructedSheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DomConstructedSheetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Hosted(error) => Some(error),
            Self::Style(error) => Some(error),
            Self::Construction(error) => Some(error),
        }
    }
}

impl From<DomHostedRuntimeError> for DomConstructedSheetError {
    fn from(error: DomHostedRuntimeError) -> Self {
        Self::Hosted(error)
    }
}

impl From<DomStyleRuntimeError> for DomConstructedSheetError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Style(error)
    }
}

impl From<DomHostedSheetConstructionError> for DomConstructedSheetError {
    fn from(error: DomHostedSheetConstructionError) -> Self {
        Self::Construction(error)
    }
}

impl DomStyleRuntime {
    /// Construct an unlisted stylesheet in the current native top-level
    /// Window. Only an idle Source and Hosted operation state is supported.
    /// The caller supplies native Hosted authority.
    pub fn construct_document_sheet(
        &self,
        dom: &mut DomEventDom<'_>,
        hosted: &HostedRuntime,
        target: DomHostedTopLevel,
        init: CssomSheetInit<'_>,
        fresh: DomConstructedSheetOwnerInit<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomConstructedSheet, DomConstructedSheetError> {
        hosted.with_current_top_level(dom, target, cancel, |source, dom| {
            if dom.pending_source_edit().is_some()
                || dom.snapshot().incomplete_hosted_operation().is_some()
            {
                return Err(DomStyleRuntimeError::from(DomError::SourceOperationPending).into());
            }
            let state = self.qualified(dom)?;
            let mut state = state
                .try_borrow_mut()
                .map_err(|_| DomStyleRuntimeError::Binding)?;
            state.step(cancel)?;
            if state.staged.is_some() || state.publication.is_some() {
                return Err(DomStyleRuntimeError::SheetSetPublicationPending.into());
            }
            let document = target.document();
            let root = document.node();
            // The ordinary canonical lookup also refuses an incomplete Source.
            let canonical = dom
                .style_cssom_root(root)
                .map_err(DomStyleRuntimeError::from)?;
            let mut owner = None;
            for bound in &state.roots {
                state.step(cancel)?;
                if bound.node == root {
                    owner = Some(bound.bound()?.clone());
                    break;
                }
            }
            let mut candidate = None;
            let owner = match owner {
                Some(owner) => owner,
                None => {
                    if canonical.is_some() {
                        return Err(DomStyleRuntimeError::Binding.into());
                    }
                    let snapshot = dom.snapshot();
                    state.prune_retired_roots(&snapshot, cancel)?;
                    let view = snapshot
                        .document(document)
                        .map_err(DomStyleRuntimeError::from)?;
                    let parser = if view.metadata().mode() == HtmlDocumentMode::Quirks {
                        CssParserMode::Quirks
                    } else {
                        CssParserMode::Standards
                    };
                    let (row, replacement) = state.prepare_constructed_root(
                        root,
                        fresh,
                        parser,
                        snapshot.stamp().revision().get(),
                        cancel,
                    )?;
                    let owner = row.bound()?.clone();
                    candidate = Some((row, replacement));
                    owner
                }
            };
            drop(canonical);
            let prepared = source.prepare_document_sheet(
                dom.raw_mut(),
                target.global(),
                document,
                &owner,
                init,
                cancel,
            )?;
            let published = prepared.publish()?;
            let mut retired = None;
            if let Some((row, replacement)) = candidate {
                if let Some(mut replacement) = replacement {
                    replacement.append(&mut state.roots);
                    retired = Some(std::mem::replace(&mut state.roots, replacement));
                }
                state.roots.push(row);
            }
            let (owner, sheet, revision) = published.finish();
            drop(state);
            drop(retired);
            Ok(DomConstructedSheet {
                owner,
                document,
                sheet,
                revision,
            })
        })
    }
}

impl StyleState {
    fn prepare_constructed_root(
        &self,
        root: NodeId,
        fresh: DomConstructedSheetOwnerInit<'_>,
        parser: CssParserMode,
        revision: u64,
        cancel: &AtomicBool,
    ) -> Result<(Root, Option<Vec<Root>>), DomStyleRuntimeError> {
        if fresh.label.is_empty() {
            return Err(DomStyleRuntimeError::Binding);
        }
        if self.roots.len() >= self.limits.roots {
            return Err(DomStyleRuntimeError::Admission);
        }
        for existing in &self.roots {
            self.step(cancel)?;
            if existing.label.len() == fresh.label.len() {
                let mut equal = true;
                for (left, right) in existing.label.bytes().zip(fresh.label.bytes()) {
                    self.step(cancel)?;
                    if left != right {
                        equal = false;
                        break;
                    }
                }
                if equal {
                    return Err(DomStyleRuntimeError::Binding);
                }
            }
        }
        let fixed = size_of::<Vec<u16>>() + size_of::<DomStyleCssomRoot>();
        let (_, requested) =
            persistent_bank_growth(self.roots.len(), self.roots.capacity(), size_of::<Root>())?;
        let admit = |label_bytes: usize, bank_bytes: usize| {
            let extra = fixed
                .checked_add(label_bytes)
                .and_then(|n| n.checked_add(bank_bytes))
                .ok_or(DomStyleRuntimeError::Admission)?;
            self.admit(extra, cancel)
        };
        admit(fresh.label.len(), requested)?;
        let mut label = String::new();
        label
            .try_reserve_exact(fresh.label.len())
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        let mut replacement = None;
        if requested != 0 {
            let mut bank = Vec::new();
            bank.try_reserve_exact(self.roots.len() + 1)
                .map_err(|_| DomStyleRuntimeError::Admission)?;
            for _ in &self.roots {
                self.step(cancel)?;
            }
            replacement = Some(bank);
        }
        let bank_bytes = replacement
            .as_ref()
            .map_or(0, Vec::capacity)
            .checked_mul(size_of::<Root>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        if label.capacity() != fresh.label.len() || bank_bytes != requested {
            admit(label.capacity(), bank_bytes)?;
        }
        for _ in fresh.label.bytes() {
            self.step(cancel)?;
        }
        label.push_str(fresh.label);
        let mut context = CssomContext::default();
        if parser == CssParserMode::Quirks {
            context
                .inputs
                .try_reserve_exact(1)
                .map_err(|_| DomStyleRuntimeError::Admission)?;
            let key = "hosted-document-parser";
            let mut identity = String::new();
            identity
                .try_reserve_exact(key.len())
                .map_err(|_| DomStyleRuntimeError::Admission)?;
            identity.push_str(key);
            context.inputs.push(CssomInput {
                version: CssomInputVersion {
                    role: CssomInputRole::ParserMode,
                    identity,
                    revision,
                },
                data: CssomInputData::ParserMode(CssParserContext::default().with_mode(parser)),
            });
        }
        let store = CssomStore::new(fresh.limits, context)?;
        let row = Root {
            node: root,
            label,
            resource: RootResource::Bound(Rc::new(DomStyleCssomRoot::new(root, store))),
            serial: 0,
            sheet_sets: sheet_sets::RootSets::new(true),
        };
        Ok((row, replacement))
    }
}
