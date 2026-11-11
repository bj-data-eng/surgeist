//! Authenticated Window access to the original native CSSOM producer.

use super::*;
use crate::{DomConstructedSheet, DomConstructedSheetError, DomConstructedSheetOwnerInit};
use surgeist_bindings_js::cssom::{MediaEdit, StylesheetHost, stylesheet_exception};
use surgeist_cssom::{
    CssomBatch, CssomEditGuard, CssomError, CssomMediaListId, CssomSheetId, CssomSheetInit,
    CssomSheetText, CssomSnapshot,
};
use surgeist_dom::{DomHostedSheetConstructionError, DomStyleCssomRootError};

pub(super) type FailureSlot = HostFailureSlot<Failure, DomHostedRuntimeRetention>;

#[derive(Debug)]
pub(super) enum Failure {
    Native(DocumentScriptNativeError),
    Construction(DomConstructedSheetError),
    Owner(DomStyleCssomRootError),
    Cssom(CssomError),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Native(error) => error,
            Self::Construction(error) => error,
            Self::Owner(error) => error,
            Self::Cssom(error) => error,
        })
    }
}

pub(super) struct Host {
    pub(super) owner: Weak<NativeOwner>,
    pub(super) permit: DocumentScriptPermit,
    pub(super) failures: FailureSlot,
}

pub(super) struct Retention {
    owner: Weak<NativeOwner>,
    bytes: usize,
    _native: DomHostedRuntimeRetention,
}

impl Retention {
    pub(super) fn installation(owner: &Rc<NativeOwner>, native: DomHostedRuntimeRetention) -> Self {
        Self {
            owner: Rc::downgrade(owner),
            bytes: 0,
            _native: native,
        }
    }
}

impl Drop for Retention {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.upgrade() {
            owner.stylesheet_bytes.set(
                owner
                    .stylesheet_bytes
                    .get()
                    .checked_sub(self.bytes)
                    .unwrap_or(usize::MAX),
            );
        }
    }
}

impl Host {
    fn refusal(&self, scope: &mut Scope<'_>, error: Failure) -> Error {
        let source = match &error {
            Failure::Cssom(CssomError::Source(name))
            | Failure::Owner(DomStyleCssomRootError::Cssom(CssomError::Source(name)))
            | Failure::Construction(DomConstructedSheetError::Construction(
                DomHostedSheetConstructionError::Cssom(CssomError::Source(name)),
            )) => Some(*name),
            _ => None,
        };
        if let Some(name) = source {
            return stylesheet_exception(scope, name);
        }
        match self.failures.record(error) {
            Ok(failure) | Err(RecordFailure::Occupied { first: failure, .. }) => {
                scope.refuse_host(&failure)
            }
            Err(RecordFailure::Inactive { error, .. }) => error,
        }
    }

    fn current_owner(&self) -> Result<Rc<NativeOwner>, DocumentScriptNativeError> {
        self.owner
            .upgrade()
            .ok_or(DocumentScriptNativeError::Revoked)
    }

    fn attachment<'a>(
        &self,
        owner: &'a NativeOwner,
    ) -> Result<&'a Attachment, DocumentScriptNativeError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        if attachment.realm.identity() != self.permit.realm
            || attachment.target.document() != self.permit.document
        {
            return Err(DocumentScriptNativeError::ForeignPermit);
        }
        Ok(attachment)
    }

    fn native_result<T>(
        &self,
        scope: &mut Scope<'_>,
        value: Result<T, DocumentScriptNativeError>,
    ) -> Result<T, Error> {
        match value {
            Ok(value) => Ok(value),
            Err(error) => Err(self.refusal(scope, Failure::Native(error))),
        }
    }

    fn edit<T>(
        &self,
        scope: &mut Scope<'_>,
        sheet: &DomConstructedSheet,
        action: impl FnOnce(&mut CssomBatch, &AtomicBool) -> Result<T, CssomError>,
    ) -> Result<T, Error> {
        self.work(scope, 1)?;
        let owner = self.native_result(scope, self.current_owner())?;
        let result = (|| {
            let root = sheet.owner();
            let snapshot = root.snapshot().map_err(Failure::Owner)?;
            let mut batch = root
                .batch(CssomEditGuard::from_snapshot(&snapshot))
                .map_err(Failure::Owner)?;
            let result = action(&mut batch, &owner.cancellation).map_err(Failure::Cssom)?;
            if owner
                .cancellation
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                return Err(Failure::Cssom(CssomError::Cancelled));
            }
            root.commit(&mut Some(batch)).map_err(Failure::Owner)?;
            Ok(result)
        })();
        match result {
            Ok(value) => Ok(value),
            Err(error) => Err(self.refusal(scope, error)),
        }
    }
}

impl StylesheetHost for Host {
    type Sheet = DomConstructedSheet;
    type Storage = Retention;

    fn retain(&self, scope: &mut Scope<'_>, bytes: usize) -> Result<Retention, Error> {
        scope.check_active()?;
        let result = (|| {
            let owner = self.current_owner()?;
            self.attachment(&owner)?;
            let total = owner
                .stylesheet_bytes
                .get()
                .checked_add(bytes)
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
            if owner
                .stylesheet_base
                .get()
                .checked_add(total)
                .is_none_or(|all| all > owner.storage_limit)
            {
                return Err(DocumentScriptNativeError::StorageLimit);
            }
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let hosted = session.hosted_execution();
            let native =
                hosted.retain_script_payload(&mut session.dom(), bytes, &owner.cancellation)?;
            owner.stylesheet_bytes.set(total);
            Ok(Retention {
                owner: Rc::downgrade(&owner),
                bytes,
                _native: native,
            })
        })();
        self.native_result(scope, result)
    }

    fn work(&self, scope: &mut Scope<'_>, units: usize) -> Result<(), Error> {
        scope.check_active()?;
        let result = (|| {
            let owner = self.current_owner()?;
            self.attachment(&owner)?;
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let hosted = session.hosted_execution();
            hosted.admit_script_work(&session.dom(), units, &owner.cancellation)?;
            Ok(())
        })();
        self.native_result(scope, result)
    }

    fn construct(
        &self,
        scope: &mut Scope<'_>,
        init: CssomSheetInit<'_>,
    ) -> Result<Self::Sheet, Error> {
        if scope.realm_authority()?.identity() != self.permit.realm {
            return Err(self.refusal(
                scope,
                Failure::Native(DocumentScriptNativeError::ForeignPermit),
            ));
        }
        self.work(scope, 1)?;
        let owner = self.native_result(scope, self.current_owner())?;
        let result = (|| {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| Failure::Native(DocumentScriptNativeError::Busy))?;
            let attachment = self.attachment(&owner).map_err(Failure::Native)?;
            let observation = owner
                .observe(attachment, &mut session)
                .map_err(Failure::Native)?;
            if observation.status() != HtmlScriptingStatus::Enabled {
                return Err(Failure::Native(DocumentScriptNativeError::Hosted(
                    DomHostedRuntimeError::Dom(DomHostedError::ScriptingDisabled),
                )));
            }
            let mut label_storage = [0; 160];
            let label = sheet_owner_label(attachment.target.document(), &mut label_storage)
                .map_err(Failure::Native)?;
            let hosted = session.hosted_execution();
            let style = session.style_execution();
            style
                .construct_document_sheet(
                    &mut session.dom(),
                    &hosted,
                    attachment.target,
                    init,
                    DomConstructedSheetOwnerInit {
                        label,
                        limits: Default::default(),
                    },
                    &owner.cancellation,
                )
                .map_err(Failure::Construction)
        })();
        match result {
            Ok(receipt) => Ok(receipt),
            Err(error) => Err(self.refusal(scope, error)),
        }
    }

    fn sheet_id(sheet: &Self::Sheet) -> &CssomSheetId {
        sheet.sheet()
    }

    fn snapshot(&self, scope: &mut Scope<'_>, sheet: &Self::Sheet) -> Result<CssomSnapshot, Error> {
        self.work(scope, 1)?;
        match sheet.owner().snapshot() {
            Ok(snapshot) => Ok(snapshot),
            Err(error) => Err(self.refusal(scope, Failure::Owner(error))),
        }
    }

    fn insert(
        &self,
        scope: &mut Scope<'_>,
        sheet: &Self::Sheet,
        text: Arc<Vec<u16>>,
        index: u32,
    ) -> Result<u32, Error> {
        self.edit(scope, sheet, |batch, cancel| {
            batch.insert_sheet_rule(
                sheet.sheet(),
                CssomSheetText::Utf16(text),
                index as usize,
                cancel,
            )?;
            Ok(index)
        })
    }

    fn delete(&self, scope: &mut Scope<'_>, sheet: &Self::Sheet, index: u32) -> Result<(), Error> {
        self.edit(scope, sheet, |batch, _| {
            batch.delete_sheet_rule(sheet.sheet(), index as usize)?;
            Ok(())
        })
    }

    fn disable(
        &self,
        scope: &mut Scope<'_>,
        sheet: &Self::Sheet,
        disabled: bool,
    ) -> Result<(), Error> {
        self.edit(scope, sheet, |batch, _| {
            batch.set_disabled(sheet.sheet(), disabled)?;
            Ok(())
        })
    }

    fn edit_media(
        &self,
        scope: &mut Scope<'_>,
        sheet: &Self::Sheet,
        media: &CssomMediaListId,
        edit: MediaEdit,
        text: Arc<Vec<u16>>,
    ) -> Result<(), Error> {
        self.edit(scope, sheet, |batch, cancel| {
            let text = CssomSheetText::Utf16(text);
            match edit {
                MediaEdit::SetText => batch.set_media_text(media, text, cancel)?,
                MediaEdit::Append => batch.append_medium(media, text, cancel)?,
                MediaEdit::Delete => batch.delete_medium(media, text, cancel)?,
            };
            Ok(())
        })
    }

    fn refuse(&self, scope: &mut Scope<'_>, error: CssomError) -> Error {
        self.refusal(scope, Failure::Cssom(error))
    }
}

fn sheet_owner_label(
    document: DocumentId,
    buffer: &mut [u8; 160],
) -> Result<&str, DocumentScriptNativeError> {
    use std::io::Write;
    let mut cursor = &mut buffer[..];
    write!(cursor, "window-constructed-sheets:{:?}", document.node())
        .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
    let extent = 160 - cursor.len();
    let label = buffer
        .get(..extent)
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    std::str::from_utf8(label).map_err(|_| DocumentScriptNativeError::RepresentationExhausted)
}
