//! Canonical Document permission, top-Document and intrinsic focus observations.
use super::*;
use crate::source::FocusQualification;
use surgeist_html::KnownHtmlAttribute;
pub(in crate::store) enum AllowFocusPermission {
    Allowed,
    NeedsWindow(FocusQualification),
    Unavailable,
}
impl Draft {
    pub(in crate::store) fn document_allow_focus_permission(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<AllowFocusPermission, DomError> {
        self.source_step(cancel)?;
        let environment = self.document_metadata(document)?.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(AllowFocusPermission::Unavailable);
        };
        let retired = matches!(
            association.phase,
            crate::hosted::DocumentPhase::Retired { .. }
        );
        if environment.browsing_context() == surgeist_html::HtmlBrowsingContextPresence::Present {
            if !matches!(
                association.phase,
                crate::hosted::DocumentPhase::InitialEntry { .. }
            ) {
                return Ok(AllowFocusPermission::Unavailable);
            }
            if self.document_fully_active(document, cancel)? {
                self.source_step(cancel)?;
                if association.policy.inherited_focus_without_user_activation()
                    == crate::DomPermissionsPolicyValue::Enabled
                {
                    return Ok(AllowFocusPermission::Allowed);
                }
            }
        } else if !retired {
            return Ok(AllowFocusPermission::Unavailable);
        }
        let document = DocumentId(
            self.published_id(document)
                .ok_or(DomError::UnavailableCreation)?,
        );
        Ok(AllowFocusPermission::NeedsWindow(if retired {
            FocusQualification::RetiredDocument {
                document,
                global: association.global,
                host: association.host,
            }
        } else {
            FocusQualification::DeniedDocument {
                document,
                global: association.global,
                host: association.host,
            }
        }))
    }
    pub(in crate::store) fn focus_boolean(
        &self,
        subject: Key,
        selected: surgeist_html::HtmlBooleanAttribute,
        attribute: KnownHtmlAttribute,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let element = self.element(subject)?;
        let mut raw = None;
        for item in element.attributes().iter() {
            self.source_step(cancel)?;
            if item.name().namespace_uri().is_none()
                && self.source_ascii(item.name().local_name(), attribute.local_name(), cancel)?
            {
                raw = Some(item);
                break;
            }
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        if let Some(formation) = &formation {
            let mut payload = source::storage::Payload::default();
            payload.name(element.name())?;
            if let Some(raw) = raw {
                payload.name(raw.name())?;
                payload.string(raw.value())?;
            }
            formation.retain_payload(payload, cancel)?;
        }
        // The existing interpreter clones only its exact report inputs. Bound
        // its fixed vocabulary/namespace applicability and Boolean spelling
        // comparisons before invocation, independently of report retention.
        for _ in 0..4 {
            self.source_step(cancel)?;
            for _ in 0..=element
                .name()
                .namespace_uri()
                .map_or(0, DocumentString::len)
            {
                self.source_step(cancel)?;
            }
            for known in surgeist_html::KnownHtmlElement::ALL {
                self.source_step(cancel)?;
                for _ in 0..=known.local_name().len() {
                    self.source_step(cancel)?;
                }
            }
        }
        if let Some(raw) = raw {
            for _ in 0..=raw.name().local_name().len() {
                self.source_step(cancel)?;
            }
            for _ in 0..=raw.value().len() {
                self.source_step(cancel)?;
            }
        }
        let report = surgeist_html::HtmlValueInterpreter::new(raw.map_or(0, |a| a.value().len()))
            .boolean_attribute(selected, element.name(), raw)
            .expect("actual admitted Boolean raw extent");
        let value = report
            .intrinsic()
            .copied()
            .ok_or(DomError::DialogInapplicable)?;
        drop(report);
        drop(formation);
        Ok(value)
    }
    pub(in crate::store) fn focus_top_document(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        let environment = self.document_metadata(document)?.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(None);
        };
        let crate::hosted::DocumentPhase::InitialEntry { navigable, .. } = association.phase else {
            return Ok(None);
        };
        association
            .validate_live()
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let graph = self
            .hosted_history
            .as_deref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let mut step = || {
            self.source_step(cancel)
                .map_err(crate::DomHostedError::from)
        };
        let node = graph
            .navigable_with(navigable, &mut step)
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => DomError::HostedSourceUnavailable,
            })?;
        let top = graph
            .navigable_with(node.traversable, &mut step)
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => DomError::HostedSourceUnavailable,
            })?;
        let active = graph
            .entry_with(top.active, &mut step)
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => DomError::HostedSourceUnavailable,
            })?
            .document;
        let Some(active) = active else {
            return Ok(None);
        };
        let key = self.resolve(active.node().into())?;
        self.source_step(cancel)?;
        let environment = self.document_metadata(key)?.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(None);
        };
        association
            .validate_live()
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        if !matches!(association.phase, crate::hosted::DocumentPhase::InitialEntry { navigable,.. } if navigable==top.id)
            || association.global != top.global
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        Ok(Some(key))
    }
}
