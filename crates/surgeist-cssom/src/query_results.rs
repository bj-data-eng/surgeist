//! Supplied query-result capture and correlation. CSSOM never evaluates queries.
use crate::*;

/// The independently revised facts used by a supplied evaluator result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomQueryFacts {
    Media {
        document: CssomInputVersion,
        window: CssomInputVersion,
    },
    Supports {
        support: CssomInputVersion,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomQueryResultRole {
    Media,
    Supports,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomQueryMatchError {
    MissingResult,
    ForeignOwner,
    WrongRule,
    WrongRole,
    StaleRevision,
    StaleContext,
    MissingFact(CssomInputRole),
    InvalidFactRole {
        expected: CssomInputRole,
        actual: CssomInputRole,
    },
    FactBindingMismatch(CssomInputRole),
}
impl std::fmt::Display for CssomQueryMatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "query result correlation failed: {self:?}")
    }
}
impl std::error::Error for CssomQueryMatchError {}
/// An immutable supplied result bound to one actual owning rule capture. CSSOM
/// validates correlation and returns the value; it never evaluates the query.
#[derive(Clone, Debug)]
pub struct CssomQueryResult {
    capture: CssomSnapshot,
    rule: CssomRuleId,
    role: CssomQueryResultRole,
    facts: CssomQueryFacts,
    matched: bool,
}
impl CssomQueryResult {
    pub fn owner(&self) -> &CssomOwnerId {
        self.capture.owner()
    }
    pub fn revision(&self) -> &CssomRevision {
        self.capture.revision()
    }
    pub fn rule(&self) -> &CssomRuleId {
        &self.rule
    }
    pub const fn role(&self) -> CssomQueryResultRole {
        self.role
    }
    pub fn context(&self) -> &CssomContext {
        self.capture.context()
    }
    pub fn facts(&self) -> &CssomQueryFacts {
        &self.facts
    }
    pub const fn supplied_match(&self) -> bool {
        self.matched
    }
}
impl CssomSnapshot {
    fn query_role(&self, id: &CssomRuleId) -> Result<CssomQueryResultRole, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(_),
                ..
            } => Ok(CssomQueryResultRole::Media),
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(_),
                ..
            } => Ok(CssomQueryResultRole::Supports),
            _ => Err(CssomError::WrongKind),
        }
    }
    fn query_fact(
        &self,
        version: &CssomInputVersion,
        role: CssomInputRole,
    ) -> Result<&CssomInputData, CssomError> {
        if version.role != role {
            return Err(CssomError::QueryMatch(
                CssomQueryMatchError::InvalidFactRole {
                    expected: role,
                    actual: version.role.clone(),
                },
            ));
        }
        self.state
            .context
            .inputs
            .iter()
            .find(|input| &input.version == version && input.data.role() == role)
            .map(|input| &input.data)
            .ok_or(CssomError::QueryMatch(CssomQueryMatchError::MissingFact(
                role,
            )))
    }
    /// Records an evaluator's boolean with explicit actual input endpoints. Media
    /// requires an associated sheet, Document and that Document's Window; Supports
    /// requires a support profile. No callback, host lookup or default is used.
    pub fn capture_query_result(
        &self,
        id: &CssomRuleId,
        facts: CssomQueryFacts,
        matched: bool,
    ) -> Result<CssomQueryResult, CssomError> {
        let role = self.query_role(id)?;
        match (&facts, role) {
            (CssomQueryFacts::Media { document, window }, CssomQueryResultRole::Media) => {
                let CssomInputData::Document { identity } =
                    self.query_fact(document, CssomInputRole::Document)?
                else {
                    unreachable!("checked role");
                };
                let CssomInputData::Window {
                    document: window_document,
                    ..
                } = self.query_fact(window, CssomInputRole::Window)?
                else {
                    unreachable!("checked role");
                };
                let sheet = self.parent_style_sheet(id)?.ok_or(CssomError::QueryMatch(
                    CssomQueryMatchError::MissingFact(CssomInputRole::Document),
                ))?;
                if window_document != identity
                    || self
                        .sheet(sheet)?
                        .inputs
                        .constructor_document
                        .as_ref()
                        .is_some_and(|sheet_document| sheet_document != identity)
                {
                    return Err(CssomError::QueryMatch(
                        CssomQueryMatchError::FactBindingMismatch(CssomInputRole::Document),
                    ));
                }
            }
            (CssomQueryFacts::Supports { support }, CssomQueryResultRole::Supports) => {
                self.query_fact(support, CssomInputRole::Support)?;
            }
            _ => return Err(CssomError::QueryMatch(CssomQueryMatchError::WrongRole)),
        }
        Ok(CssomQueryResult {
            capture: self.clone(),
            rule: id.clone(),
            role,
            facts,
            matched,
        })
    }
    /// Returns only a matching result from this exact immutable capture. Absence,
    /// stale input/state, a foreign owner or another rule are typed failures.
    pub(crate) fn condition_matches(
        &self,
        id: &CssomRuleId,
        result: Option<&CssomQueryResult>,
    ) -> Result<bool, CssomError> {
        let role = self.query_role(id)?;
        let result = result.ok_or(CssomError::QueryMatch(CssomQueryMatchError::MissingResult))?;
        let error = if result.owner() != self.owner() {
            Some(CssomQueryMatchError::ForeignOwner)
        } else if result.role != role {
            Some(CssomQueryMatchError::WrongRole)
        } else if &result.rule != id {
            Some(CssomQueryMatchError::WrongRule)
        } else if result.context() != self.context() {
            Some(CssomQueryMatchError::StaleContext)
        } else if result.revision() != self.revision() {
            Some(CssomQueryMatchError::StaleRevision)
        } else {
            None
        };
        if let Some(error) = error {
            return Err(CssomError::QueryMatch(error));
        }
        Ok(result.matched)
    }
}
