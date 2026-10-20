//! Lossless owner-attribute ingress. CSS owns preprocessing and declaration grammar.
use crate::{declaration_budget::DeclarationBudget, *};
use std::{fmt, sync::Arc};
use surgeist_css::*;

/// Actual original owner text. UTF8 convenience is encoded without CSS interpretation
/// and follows the same CSS producer boundary as native UTF16.
#[derive(Clone, Debug)]
pub enum CssomStyleAttributeInput<'a> {
    Utf8(&'a str),
    Utf16(Arc<Vec<u16>>),
}
impl<'a> From<&'a str> for CssomStyleAttributeInput<'a> {
    fn from(text: &'a str) -> Self {
        Self::Utf8(text)
    }
}
impl CssomStyleAttributeInput<'_> {
    pub(crate) fn matches_utf8(&self, expected: &str) -> bool {
        match self {
            Self::Utf8(text) => *text == expected,
            Self::Utf16(units) => expected.encode_utf16().eq(units.iter().copied()),
        }
    }
}

/// CSSOM refusal and CSS input preparation/interruption remain distinct.
#[derive(Debug)]
pub enum CssomStyleAttributeError<E> {
    Cssom(CssomError),
    Input(CssUtf16InputError<E>),
}
impl<E> From<CssomError> for CssomStyleAttributeError<E> {
    fn from(error: CssomError) -> Self {
        Self::Cssom(error)
    }
}
impl<E> From<CssUtf16InputError<E>> for CssomStyleAttributeError<E> {
    fn from(error: CssUtf16InputError<E>) -> Self {
        Self::Input(error)
    }
}
impl<E: fmt::Display> fmt::Display for CssomStyleAttributeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cssom(error) => error.fmt(f),
            Self::Input(error) => error.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for CssomStyleAttributeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cssom(error) => Some(error),
            Self::Input(error) => Some(error),
        }
    }
}

impl CssomBatch {
    /// Prepares a qualifying external style change without requesting an echo owner write.
    /// Unrelated names/namespaces, computed and updating blocks return before input work.
    /// Absence retains the existing empty-replacement semantics, including parsing.
    ///
    /// UTF8 encoding is bounded by original-unit/source-byte limits and polls per scalar.
    /// CSS preparation has its own work/backing bounds and polls; general grammar stays
    /// opaque between CSS boundary polls under the cumulative component/depth/byte limits.
    /// Existing declaration/provider and staged-state/source quotas remain independent.
    /// Any input or CSSOM refusal poisons this batch and publishes nothing.
    pub fn prepare_style_attribute_change<E>(
        &mut self,
        id: &CssomBlockId,
        change: &CssomStyleAttributeChange<'_>,
        input_limits: CssUtf16InputLimits,
        limits: CssomDeclarationRequestLimits,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<CssomPreparedDeclaration>, CssomStyleAttributeError<E>> {
        self.apply(|this| {
            let block = this.staged.block(id)?;
            if change.local_name != "style"
                || change.namespace.is_some()
                || block.flags.computed
                || block.flags.updating
            {
                return Ok(None);
            }
            if block.owner() != Some(&change.owner) {
                return Err(CssomError::StaleOwnerNotification.into());
            }
            if !matches!(
                block.data(),
                CssomBlockData::Properties {
                    domain: CssomPropertyDomain::Ordinary,
                    ..
                }
            ) {
                return Err(CssomError::WrongKind.into());
            }
            let context = this.staged.declaration_context(id)?.parser_context();
            let mut budget = DeclarationBudget::new(limits, 12);
            let component_limits = budget.input_components(this.limits.max_input_bytes)?;
            let original = match change.value.as_ref() {
                Some(CssomStyleAttributeInput::Utf16(units)) => units.clone(),
                Some(CssomStyleAttributeInput::Utf8(text)) => {
                    encode_input(text, input_limits, &mut poll)?
                }
                None => encode_input("", input_limits, &mut poll)?,
            };
            let report = context.parse_declaration_block_contents_utf16_with_limits(
                original,
                input_limits,
                component_limits,
                poll,
            )?;
            this.prepare_attribute_report(id, report, budget)
                .map(Some)
                .map_err(Into::into)
        })
    }

    /// Initializes a staged creation in this same batch without exposing its tentative ID.
    pub fn prepare_created_style_attribute_change<E>(
        &mut self,
        ticket: &CssomBlockTicket,
        change: &CssomStyleAttributeChange<'_>,
        input_limits: CssUtf16InputLimits,
        limits: CssomDeclarationRequestLimits,
        poll: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<CssomPreparedDeclaration>, CssomStyleAttributeError<E>> {
        self.apply(|this| {
            let id = this.created_block_id(ticket)?.clone();
            this.prepare_style_attribute_change(&id, change, input_limits, limits, poll)
        })
    }
}

fn encode_input<E>(
    text: &str,
    limits: CssUtf16InputLimits,
    poll: &mut impl FnMut() -> Result<(), E>,
) -> Result<Arc<Vec<u16>>, CssUtf16InputError<E>> {
    use CssUtf16InputError::{Allocation, CapacityOverflow, Interrupted, Limit};
    let mut count = 0_usize;
    for scalar in text.chars() {
        poll().map_err(Interrupted)?;
        count = count
            .checked_add(scalar.len_utf16())
            .ok_or(CapacityOverflow)?;
        if count > limits.max_input_units {
            return Err(Limit("input units"));
        }
    }
    let bytes = count.checked_mul(2).ok_or(CapacityOverflow)?;
    if bytes > limits.max_retained_bytes {
        return Err(Limit("retained source bytes"));
    }
    let mut units = Vec::new();
    units.try_reserve_exact(count).map_err(|_| Allocation)?;
    for scalar in text.chars() {
        poll().map_err(Interrupted)?;
        let mut encoded = [0; 2];
        units.extend_from_slice(scalar.encode_utf16(&mut encoded));
    }
    Ok(Arc::new(units))
}
