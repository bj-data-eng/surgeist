//! Source MediaList operations over the shared live collection owner.
use crate::parser_resources::check_resource_diagnostics;
use crate::*;
use surgeist_css::*;

/// A source mutation's disposition and original recovered parser diagnostics.
/// Publication is reported separately by the enclosing batch commit.
#[derive(Clone, Debug)]
pub struct CssomMediaEdit {
    changed: bool,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}
impl CssomMediaEdit {
    pub(crate) fn into_diagnostics(self) -> Vec<CssRecoveryDiagnostic> {
        self.diagnostics
    }
    pub const fn changed(&self) -> bool {
        self.changed
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
}

impl CssomSnapshot {
    /// Canonical CSSOM text of the associated current collection.
    pub(crate) fn media_text(
        &self,
        id: &CssomMediaListId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        self.media(id)?
            .serialize_cssom_with_limits(limits)
            .map(|text| text.as_css().to_owned())
            .map_err(CssomError::Media)
    }
    pub(crate) fn media_length(&self, id: &CssomMediaListId) -> Result<usize, CssomError> {
        Ok(self.media(id)?.queries().len())
    }
    /// Returns null out of range, independently of the output allowance.
    pub(crate) fn media_item(
        &self,
        id: &CssomMediaListId,
        index: usize,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Option<String>, CssomError> {
        self.media(id)?
            .queries()
            .get(index)
            .map(|query| {
                query
                    .serialize_cssom_with_limits(limits)
                    .map(|text| text.as_css().to_owned())
                    .map_err(CssomError::Media)
            })
            .transpose()
    }
}

impl CssomBatch {
    /// Replaces the contents through CSS-owned list recovery; empty input clears.
    /// No sheet security/modification guard or owner-attribute write is implied.
    pub fn set_media_text(
        &mut self,
        id: &CssomMediaListId,
        source: &str,
    ) -> Result<CssomMediaEdit, CssomError> {
        self.apply(|this| {
            this.staged.media(id)?;
            if source.is_empty() {
                return Ok(CssomMediaEdit {
                    changed: this.set_media(id, CssMediaQueryList::new(Vec::new()))?,
                    diagnostics: Vec::new(),
                });
            }
            check_input(source, &this.limits)?;
            let (value, diagnostics) = recovered(parse_media_query_list(source))?;
            value
                .serialize_cssom_with_limits(this.limits.css)
                .map_err(CssomError::Media)?;
            Ok(CssomMediaEdit {
                changed: this.set_media(id, value)?,
                diagnostics,
            })
        })
    }
    /// Appends a single recovered query unless its canonical bytes already exist.
    pub fn append_medium(
        &mut self,
        id: &CssomMediaListId,
        source: &str,
    ) -> Result<CssomMediaEdit, CssomError> {
        self.edit_medium(id, source, false)
    }
    /// Removes every canonical match; a valid absent candidate is NotFound.
    /// Empty/multiple-query input is a successful no-op.
    pub fn delete_medium(
        &mut self,
        id: &CssomMediaListId,
        source: &str,
    ) -> Result<CssomMediaEdit, CssomError> {
        self.edit_medium(id, source, true)
    }
    fn edit_medium(
        &mut self,
        id: &CssomMediaListId,
        source: &str,
        remove: bool,
    ) -> Result<CssomMediaEdit, CssomError> {
        self.apply(|this| {
            this.staged.media(id)?;
            check_input(source, &this.limits)?;
            let (candidate, diagnostics) = recovered(parse_cssom_media_query(source))?;
            let Some(candidate) = candidate else {
                return Ok(CssomMediaEdit {
                    changed: false,
                    diagnostics,
                });
            };
            // CSS owns cumulative source/projection/output accounting for the
            // candidate and every scanned member. No token spelling/evaluation
            // or separately reset pair allowance supplies equality.
            let current = this.staged.media(id)?;
            let scan_len = current
                .queries()
                .len()
                .checked_add(1)
                .ok_or(CssomError::Limit {
                    resource: "media scan capacity",
                    maximum: this.limits.max_entries,
                })?;
            let mut scan = Vec::new();
            scan.try_reserve(scan_len).map_err(|_| CssomError::Limit {
                resource: "media scan capacity",
                maximum: this.limits.max_entries,
            })?;
            scan.push(candidate.clone());
            scan.extend_from_slice(current.queries());
            let texts = CssMediaQueryList::new(scan)
                .serialize_cssom_members_with_limits(this.limits.css)
                .map_err(CssomError::Media)?;
            let candidate_text = texts[0].as_css();
            let found = texts[1..]
                .iter()
                .any(|text| text.as_css() == candidate_text);
            if !remove && found {
                return Ok(CssomMediaEdit {
                    changed: false,
                    diagnostics,
                });
            }
            if remove && !found {
                return Err(CssomError::Source(CssomException::NotFound));
            }
            let mut selected = current.queries().to_vec();
            if remove {
                let mut index = 0;
                selected.retain(|_| {
                    let keep = texts[index + 1].as_css() != candidate_text;
                    index += 1;
                    keep
                });
            } else {
                selected.push(candidate);
            }
            Ok(CssomMediaEdit {
                changed: this.set_media(id, CssMediaQueryList::new(selected))?,
                diagnostics,
            })
        })
    }
}

fn check_input(source: &str, limits: &CssomLimits) -> Result<(), CssomError> {
    if source.len() > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    Ok(())
}
fn recovered<T>(report: CssParseReport<T>) -> Result<(T, Vec<CssRecoveryDiagnostic>), CssomError> {
    check_resource_diagnostics(report.diagnostics())?;
    Ok(report.into_parts())
}

impl CssomBatch {
    /// Product typed media replacement uses the existing checked Media Queries carrier.
    pub fn set_media(
        &mut self,
        id: &CssomMediaListId,
        value: CssMediaQueryList,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.media(id)?;
            if this.staged.media[id] == value {
                return Ok(false);
            }
            let sheets = this
                .staged
                .sheets
                .iter()
                .filter(|(_, sheet)| &sheet.media == id)
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>();
            for sheet in sheets {
                this.staged
                    .sheets
                    .get_mut(&sheet)
                    .expect("selected sheet")
                    .inputs
                    .media = value.clone();
            }
            this.staged.media.insert(id.clone(), value);
            this.affect(CssomObjectId::MediaList(id.clone()), &[CssomChange::Media]);
            Ok(true)
        })
    }
}
