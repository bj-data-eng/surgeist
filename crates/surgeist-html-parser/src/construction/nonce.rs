//! Independent owned nonce slot, with the original parser control and registry.
use super::*;
#[cfg(test)]
use surgeist_html::values::DocumentString;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlFragmentOwner},
    values::QualifiedName,
};

impl SessionStorage {
    pub(super) fn prepare_nonce(
        &mut self,
        c: &mut ParseControl,
        name: Option<&QualifiedName>,
    ) -> Result<Option<usize>, SessionError> {
        let Some(name) = name else {
            return Ok(None);
        };
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND as u64,
        )?;
        if !surgeist_html::bounded::security::is_nonce_element(name) {
            return Ok(None);
        }
        let empty = literal_string(c, b"")?;
        let (value, raw) = self.register_string_owned(c, empty, None)?;
        drop(value);
        let prepared = self.prepare_payload_slot_owned(c)?;
        let slot = self
            .payloads
            .push_row(prepared, RetainedPayload::Nonce { node: None, raw })
            .map_err(|_| latch_registry_row(c))?;
        Ok(Some(slot))
    }
    pub(super) fn nonce_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<(usize, usize)>, SessionError> {
        c.require_identity(&self.control)?;
        if let Some(slot) = self.payloads.joined(c, node, Join::Nonce)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::Nonce {
                node: Some(owner),
                raw,
            }) = self.payloads.values.get(slot)
                && *owner == node
            {
                c.charge(
                    HtmlParseWorkCategory::CopyByte,
                    const { (2 * mem::size_of::<usize>()) as u64 },
                )?;
                return Ok(Some((slot, *raw)));
            }
        }
        Ok(None)
    }
    pub(super) fn replace_nonce(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        raw: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(&RetainedPayload::Nonce { raw: old, .. }) = self.payloads.values.get(slot) else {
            return Err(SessionError::NonceMetadataUnavailable);
        };
        payload_write::<usize>(c, 1)?;
        let Some(RetainedPayload::Nonce { raw: stored, .. }) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(c));
        };
        *stored = raw;
        self.retire_raw_field(c, old)
    }
    pub(super) fn initial_nonce_attributes(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Ok(());
        };
        let count = element.attributes().count(c)?;
        for position in 0..count {
            self.initial_nonce_attribute(c, node, position)?;
        }
        Ok(())
    }
    pub(super) fn initial_nonce_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        self.install_nonce_attribute(c, node, position)
    }
    /// Copies the value of the nonce attribute at `position` into the nonce slot of the element.
    fn install_nonce_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND as u64,
        )?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let attribute = element
            .attributes()
            .at(c, position)?
            .ok_or(SessionError::WrongToken)?;
        let value = attribute.value().units(c)?;
        let ignored = matches!(
            surgeist_html::security::nonce::nonce_attribute_change(
                element.name(),
                attribute.name(),
                Some(value)
            ),
            surgeist_html::security::nonce::HtmlNonceAttributeEffect::Ignore
        );
        if ignored {
            return Ok(());
        }
        let raw = self.copy_attribute_value(c, node, position)?;
        let (slot, _) = self
            .nonce_slot(c, node)?
            .ok_or(SessionError::NonceMetadataUnavailable)?;
        self.replace_nonce(c, slot, raw)
    }
    pub(super) fn clone_nonce(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        copy: usize,
    ) -> Result<(), SessionError> {
        let Some((_, original)) = self.nonce_slot(c, source)? else {
            return Ok(());
        };
        let (slot, _) = self
            .nonce_slot(c, copy)?
            .ok_or(SessionError::NonceMetadataUnavailable)?;
        let raw = self.copy_raw_field(c, original)?;
        self.replace_nonce(c, slot, raw)
    }
    pub(super) fn nonce_environment(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<
        (
            surgeist_html::security::nonce::HtmlNonceConnection,
            surgeist_html::security::nonce::HtmlHeaderDeliveredCsp,
        ),
        SessionError,
    > {
        use surgeist_html::security::nonce::{
            HtmlHeaderDeliveredCsp as P, HtmlNonceConnection as C,
        };
        let mut root = self.ordinary_root(c, node, None)?;
        for _ in 0..self.construction.len() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            if matches!(
                self.construction
                    .node_at(root)
                    .ok_or_else(|| latch_registry_row(c))?
                    .kind(),
                ConstructionNodeKind::Document(_)
            ) {
                // Every owned construction Document is standalone/inert, with no BC.
                return Ok((C::NotBrowsingContextConnected, P::Unavailable));
            }
            if let Some(HtmlFragmentOwner::ShadowHost(host)) = &self
                .construction
                .node_at(root)
                .ok_or_else(|| latch_registry_row(c))?
                .fragment_owner()
            {
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                let host_position = self
                    .construction
                    .node_by_result_id(host)
                    .map_err(|_| SessionError::ForeignNode)?
                    .position();
                root = self.ordinary_root(c, host_position, None)?;
                continue;
            }
            if let Some(fragment) = &self.fragment {
                for (index, effect) in fragment.effects.as_slice().iter().enumerate() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if let context::ContextEffect::AttachShadow {
                        contents: Some(contents),
                        host,
                        ..
                    } = effect
                        && *contents == root
                    {
                        match host {
                            context::ShadowHostKey::Owned(host) => {
                                root = self.ordinary_root(c, *host, None)?;
                                break;
                            }
                            context::ShadowHostKey::Captured(host) => {
                                return self.captured_nonce_environment(c, host);
                            }
                        }
                    }
                    if index.saturating_add(1) == fragment.effects.len() {
                        return Ok((C::NotBrowsingContextConnected, P::Unavailable));
                    }
                }
                if fragment.effects.len() != 0 {
                    continue;
                }
            }
            return Ok((C::NotBrowsingContextConnected, P::Unavailable));
        }
        Err(SessionError::Cycle)
    }
    fn captured_nonce_environment(
        &self,
        c: &mut ParseControl,
        host: &crate::HtmlCaptureElementId,
    ) -> Result<
        (
            surgeist_html::security::nonce::HtmlNonceConnection,
            surgeist_html::security::nonce::HtmlHeaderDeliveredCsp,
        ),
        SessionError,
    > {
        use surgeist_html::security::nonce::{
            HtmlHeaderDeliveredCsp as P, HtmlNonceConnection as C,
        };
        let document = match self.captured_shadow_including_document(c, host) {
            Ok(Some(document)) => document,
            Ok(None) => return Ok((C::NotBrowsingContextConnected, P::Unavailable)),
            Err(SessionError::ContextFactUnavailable(
                crate::HtmlCaptureFact::Parent | crate::HtmlCaptureFact::FragmentHost,
            )) => return Ok((C::Unavailable, P::Unavailable)),
            Err(error) => return Err(error),
        };
        let environment = document.nonce_environment();
        Ok((
            match environment.browsing_context() {
                surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent => {
                    C::NotBrowsingContextConnected
                }
                surgeist_html::security::nonce::HtmlBrowsingContextPresence::Present => {
                    C::BrowsingContextConnected
                }
                surgeist_html::security::nonce::HtmlBrowsingContextPresence::Unavailable => {
                    C::Unavailable
                }
            },
            environment.header_delivered_csp(),
        ))
    }
    /// The original immutable captured ancestry, independent of browsing-context
    /// presence. Unknown parents/hosts never prove disconnection.
    pub(super) fn captured_shadow_including_document<'a>(
        &'a self,
        c: &mut ParseControl,
        host: &crate::HtmlCaptureElementId,
    ) -> Result<Option<&'a crate::HtmlCapturedDocument>, SessionError> {
        let capture = self
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?
            .context
            .capture();
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let mut node = capture
            .node(host.subject())
            .map_err(|_| SessionError::ForeignNode)?;
        for _ in 0..capture.usage().subjects() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            if let crate::HtmlCapturedNodeKind::Document(document) = node.kind() {
                return Ok(Some(document));
            }
            payload_write::<crate::HtmlCapturedParent>(c, 1)?;
            match node.parent() {
                crate::HtmlCapturedParent::Subject(parent) => {
                    c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                    node = capture
                        .node(&parent)
                        .map_err(|_| SessionError::ForeignNode)?;
                }
                crate::HtmlCapturedParent::Unavailable => {
                    return Err(SessionError::ContextFactUnavailable(
                        crate::HtmlCaptureFact::Parent,
                    ));
                }
                crate::HtmlCapturedParent::Root => {
                    // A plain fragment's generic host may be its Template
                    // owner. Only a ShadowRoot host extends shadow ancestry.
                    if !matches!(node.kind(), crate::HtmlCapturedNodeKind::Fragment(fragment)
                        if matches!(fragment.kind(), crate::HtmlCapturedFragmentKind::ShadowRoot(_)))
                    {
                        return Ok(None);
                    }
                    payload_write::<Option<crate::HtmlCapturedHost>>(c, 1)?;
                    match node.host() {
                        Some(crate::HtmlCapturedHost::Element(host)) => {
                            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                            node = capture
                                .node(host.subject())
                                .map_err(|_| SessionError::ForeignNode)?;
                        }
                        Some(crate::HtmlCapturedHost::Unavailable) => {
                            return Err(SessionError::ContextFactUnavailable(
                                crate::HtmlCaptureFact::FragmentHost,
                            ));
                        }
                        _ => return Ok(None),
                    }
                }
            }
        }
        Err(SessionError::Cycle)
    }
    pub(super) fn nonce_inserted(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Ok(());
        };
        let mut selected = None;
        for (position, attribute) in element.attributes().each(c)?.enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND as u64,
            )?;
            let value = attribute.value().units(c)?;
            let ignored = matches!(
                surgeist_html::security::nonce::nonce_attribute_change(
                    element.name(),
                    attribute.name(),
                    Some(value)
                ),
                surgeist_html::security::nonce::HtmlNonceAttributeEffect::Ignore
            );
            if !ignored {
                if value.is_empty() {
                    return Ok(());
                }
                selected = Some(position);
                break;
            }
        }
        let Some(position) = selected else {
            return Ok(());
        };
        let (connection, policy) = self.nonce_environment(c, node)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND as u64,
        )?;
        let element = element_record(&self.construction, c, node)?;
        let attribute = element_attribute(&self.construction, c, node, position)?;
        let value = attribute.value().units(c)?;
        let effect = surgeist_html::security::nonce::nonce_insertion_effect(
            element.name(),
            Some(value),
            connection,
            policy,
        );
        match effect {
            surgeist_html::security::nonce::HtmlNonceInsertionEffect::KeepContent => Ok(()),
            surgeist_html::security::nonce::HtmlNonceInsertionEffect::Unavailable(fact) => {
                Err(SessionError::ContextFactUnavailable(match fact {
                    surgeist_html::security::nonce::HtmlNonceMissing::Connection => {
                        crate::HtmlCaptureFact::BrowsingContextConnection
                    }
                    surgeist_html::security::nonce::HtmlNonceMissing::HeaderDeliveredCsp => {
                        crate::HtmlCaptureFact::HeaderDeliveredCsp
                    }
                }))
            }
            surgeist_html::security::nonce::HtmlNonceInsertionEffect::HideContentPreservingSlot => {
                let (slot, prior) = self
                    .nonce_slot(c, node)?
                    .ok_or(SessionError::NonceMetadataUnavailable)?;
                let saved = self.copy_raw_field(c, prior)?;
                #[cfg(test)]
                tests::interrupt_at(tests::NonceCancelPoint::Saved);
                let empty = literal_string(c, b"")?;
                self.set_nonce_content_owned(c, node, Some(empty))?;
                #[cfg(test)]
                tests::interrupt_at(tests::NonceCancelPoint::ContentEmptied);
                self.replace_nonce(c, slot, saved)
            }
        }
    }
    pub(super) fn nonce_inserted_tree(
        &mut self,
        c: &mut ParseControl,
        inserted: usize,
    ) -> Result<(), SessionError> {
        let mut next = Some(inserted);
        while let Some(node) = next {
            self.nonce_inserted(c, node)?;
            next = self.next_nonce_inserted(c, inserted, node)?;
        }
        Ok(())
    }
    fn next_nonce_inserted(
        &self,
        c: &mut ParseControl,
        root: usize,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(shadow) = self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .shadow_root()
        {
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            return Ok(Some(
                self.construction
                    .node_by_result_id(&shadow.contents())
                    .map_err(|_| SessionError::ForeignNode)?
                    .position(),
            ));
        }
        if let Some(child) = self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .child_at(0)
        {
            c.note_sibling(node, 0);
            return Ok(Some(child.position()));
        }
        let mut cursor = node;
        for _ in 0..self.construction.len() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if cursor == root {
                return Ok(None);
            }
            if let Some(parent) = self
                .construction
                .node_at(cursor)
                .ok_or_else(|| latch_registry_row(c))?
                .parent()
                .map(|node| node.position())
            {
                let count = self
                    .construction
                    .node_at(parent)
                    .ok_or_else(|| latch_registry_row(c))?
                    .child_count();
                // The step that reached `cursor` left its ordinal behind, so the
                // search for it starts there and costs the siblings after it.
                let mut ordinal = None;
                if let Some(hinted) = c.sibling_hint(parent)
                    && hinted < count
                {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if self
                        .construction
                        .node_at(parent)
                        .ok_or_else(|| latch_registry_row(c))?
                        .child_at(hinted)
                        .map(|child| child.position())
                        == Some(cursor)
                    {
                        ordinal = Some(hinted);
                    }
                }
                if ordinal.is_none() {
                    for index in 0..count {
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        let child = self
                            .construction
                            .node_at(parent)
                            .ok_or_else(|| latch_registry_row(c))?
                            .child_at(index)
                            .ok_or_else(|| latch_registry_row(c))?
                            .position();
                        if child == cursor {
                            ordinal = Some(index);
                            break;
                        }
                    }
                }
                let ordinal = ordinal.ok_or(SessionError::WrongSibling)?;
                let following = ordinal.saturating_add(1);
                if following < count {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let child = self
                        .construction
                        .node_at(parent)
                        .ok_or_else(|| latch_registry_row(c))?
                        .child_at(following)
                        .ok_or_else(|| latch_registry_row(c))?
                        .position();
                    c.note_sibling(parent, following);
                    return Ok(Some(child));
                }
                cursor = parent;
            } else if let Some(HtmlFragmentOwner::ShadowHost(host)) = &self
                .construction
                .node_at(cursor)
                .ok_or_else(|| latch_registry_row(c))?
                .fragment_owner()
            {
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                let host_position = self
                    .construction
                    .node_by_result_id(host)
                    .map_err(|_| SessionError::ForeignNode)?
                    .position();
                cursor = host_position;
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if let Some(child) = self
                    .construction
                    .node_at(cursor)
                    .ok_or_else(|| latch_registry_row(c))?
                    .child_at(0)
                {
                    return Ok(Some(child.position()));
                }
            } else {
                return Err(SessionError::ForeignNode);
            }
        }
        Err(SessionError::Cycle)
    }
    #[cfg(test)]
    pub(super) fn set_nonce_content(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        value: Option<FrozenUtf16>,
    ) -> Result<(), SessionError> {
        if let Err(error) = self.require(c) {
            if let Some(value) = value {
                value.release(c)?;
            }
            return Err(error);
        }
        self.set_nonce_content_owned(c, node, value)
    }
    // Insertion already owns the active connection-effect phase. Changing the
    // nonce must progress under that ownership rather than reenter an idle API.
    fn set_nonce_content_owned(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        mut value: Option<FrozenUtf16>,
    ) -> Result<(), SessionError> {
        let mut previous = None;
        let result = (|| {
            c.require_identity(&self.control)?;
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            self.nonce_slot(c, node)?
                .ok_or(SessionError::NonceMetadataUnavailable)?;
            previous = Some(self.begin_attribute_change(c, node, None)?);
            self.set_nonce_content_inner(c, node, &mut value)
        })();
        if let Some(value) = value {
            value.release(c)?;
        }
        if c.stop().is_none()
            && let Some(previous) = previous
        {
            self.end_node_copy(c, previous.as_ref())?;
        }
        result
    }
    fn set_nonce_content_inner(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        value: &mut Option<FrozenUtf16>,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND as u64,
        )?;
        if !surgeist_html::bounded::security::is_nonce_element(element.name()) {
            return Err(SessionError::NonceMetadataUnavailable);
        }
        let mut position = None;
        for (index, attribute) in element.attributes().each(c)?.enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                surgeist_html::bounded::security::HTML_NONCE_INSPECTION_BOUND as u64,
            )?;
            let units = attribute.value().units(c)?;
            let ignored = matches!(
                surgeist_html::security::nonce::nonce_attribute_change(
                    element.name(),
                    attribute.name(),
                    Some(units)
                ),
                surgeist_html::security::nonce::HtmlNonceAttributeEffect::Ignore
            );
            if !ignored {
                position = Some(index);
                break;
            }
        }
        if let Some(value) = value.take() {
            let value = self.field_string_owned(c, value, None)?;
            let name = match literal_string(c, b"nonce")
                .and_then(|name| self.field_string_owned(c, name, None))
            {
                Ok(name) => name,
                Err(error) => {
                    value.release(c)?;
                    return Err(error);
                }
            };
            if let Err(error) = payload_write::<[FieldString; 2]>(c, 1) {
                name.release(c)?;
                value.release(c)?;
                return Err(error);
            }
            if let Some(position) = position {
                self.replace_registered_attribute(c, node, position, name, value)?;
                self.install_nonce_attribute(c, node, position)
            } else {
                // The actual merge owner invokes the same installation hook.
                self.append_registered_attribute(c, node, name, value)
            }
        } else {
            self.remove_managed_field(c, node, "nonce")?;
            let (slot, _) = self
                .nonce_slot(c, node)?
                .ok_or(SessionError::NonceMetadataUnavailable)?;
            let empty = literal_string(c, b"")?;
            let (value, raw) = self.register_string_owned(c, empty, None)?;
            drop(value);
            self.replace_nonce(c, slot, raw)
        }
    }
}

impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn cryptographic_nonce<'a>(
        &'a self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<&'a DocumentString, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        let (_, raw) = storage
            .nonce_slot(c, node)?
            .ok_or(SessionError::NonceMetadataUnavailable)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::String {
            value, allocation, ..
        }) = storage.payloads.values.get(raw)
        else {
            return Err(latch_registry_row(c));
        };
        c.require_document_string(allocation, value)?;
        Ok(value)
    }
    #[cfg(test)]
    fn set_cryptographic_nonce(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        mut value: Option<FrozenUtf16>,
    ) -> Result<(), SessionError> {
        let result = (|| {
            let storage = self.active_mut()?;
            storage.require(c)?;
            let node = storage.index(c, node)?;
            let (slot, _) = storage
                .nonce_slot(c, node)?
                .ok_or(SessionError::NonceMetadataUnavailable)?;
            let previous = storage.begin_attribute_change(c, node, None)?;
            let supplied = value.take().ok_or(SessionError::Consumed)?;
            let group = if let Some(evidence) = storage.graph() {
                match evidence.source_independent_group(c, supplied.text().len()) {
                    Ok(group) => Some(group),
                    Err(error) => {
                        supplied.release(c)?;
                        return Err(error);
                    }
                }
            } else {
                None
            };
            let (native, raw) = storage.register_group_string_owned(c, supplied, group)?;
            drop(native);
            let result = storage.replace_nonce(c, slot, raw);
            if c.stop().is_none() {
                storage.end_node_copy(c, previous.as_ref())?;
            }
            result
        })();
        if let Some(value) = value {
            value.release(c)?;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html::{
        construction::ConstructionNodeKind,
        values::{DocumentString, Namespace, QualifiedName},
    };
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) enum NonceCancelPoint {
        Saved,
        ContentEmptied,
    }
    struct TestCancellation {
        point: NonceCancelPoint,
        cancellation: crate::HtmlParseCancellation,
        reached: bool,
    }
    thread_local! { static CANCELLATION: std::cell::RefCell<Option<TestCancellation>> = const { std::cell::RefCell::new(None) }; }
    // Observe only completed real actions. No fee, controller or producer field changes.
    pub(super) fn interrupt_at(point: NonceCancelPoint) {
        CANCELLATION.with_borrow_mut(|state| {
            if let Some(state) = state
                && state.point == point
                && !state.reached
            {
                state.reached = true;
                state.cancellation.cancel();
            }
        });
    }
    struct CancellationObservation;
    impl CancellationObservation {
        fn install(point: NonceCancelPoint, cancellation: crate::HtmlParseCancellation) -> Self {
            CANCELLATION.with_borrow_mut(|state| {
                assert!(state.is_none());
                *state = Some(TestCancellation {
                    point,
                    cancellation,
                    reached: false,
                });
            });
            Self
        }
        fn reached(&self) -> bool {
            CANCELLATION.with_borrow(|state| state.as_ref().is_some_and(|state| state.reached))
        }
    }
    impl Drop for CancellationObservation {
        fn drop(&mut self) {
            CANCELLATION.with_borrow_mut(|state| *state = None);
        }
    }

    fn fixture() -> (
        ParseControl,
        ParserArenaSession,
        ParserNode,
        crate::HtmlParseCancellation,
    ) {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=hidden nonce=A>"),
            super::super::tests::limits(),
        );
        let cancellation = request.cancellation().clone();
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            super::super::tests::input_owner(&request);
        let node = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            )
            .unwrap();
        lexer.release(&mut c).unwrap();
        (c, session, node, cancellation)
    }
    fn supplied(c: &mut ParseControl, units: &[u16]) -> FrozenUtf16 {
        let mut buffer = Utf16Buffer::new(c).unwrap();
        buffer.extend_units(units, c).unwrap();
        buffer.freeze(c).unwrap()
    }
    #[test]
    fn nonce_owned_slot_setter_content_remove_and_clone_are_independent() {
        let (mut c, mut session, node, _) = fixture();
        assert_eq!(
            session
                .cryptographic_nonce(&mut c, &node)
                .unwrap()
                .code_units(),
            &[65]
        );
        let value = supplied(&mut c, &[0, 32, 0xd800, 0xffff]);
        session
            .set_cryptographic_nonce(&mut c, &node, Some(value))
            .unwrap();
        assert_eq!(
            session
                .active()
                .unwrap()
                .form_attribute(&mut c, node.index, "nonce")
                .unwrap()
                .unwrap(),
            &[65]
        );
        let copy = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .unwrap();
        let copy = session.active().unwrap().node(&mut c, copy).unwrap();
        assert_eq!(
            session
                .cryptographic_nonce(&mut c, &copy)
                .unwrap()
                .code_units(),
            &[0, 32, 0xd800, 0xffff]
        );
        assert_eq!(
            session
                .active()
                .unwrap()
                .form_attribute(&mut c, copy.index, "nonce")
                .unwrap()
                .unwrap(),
            &[65]
        );
        let value = supplied(&mut c, &[66]);
        session
            .active_mut()
            .unwrap()
            .set_nonce_content(&mut c, node.index, Some(value))
            .unwrap();
        assert_eq!(
            session
                .cryptographic_nonce(&mut c, &node)
                .unwrap()
                .code_units(),
            &[66]
        );
        session
            .active_mut()
            .unwrap()
            .set_nonce_content(&mut c, node.index, None)
            .unwrap();
        assert_eq!(
            session
                .cryptographic_nonce(&mut c, &node)
                .unwrap()
                .code_units(),
            &[]
        );
        assert!(
            session
                .active()
                .unwrap()
                .form_attribute(&mut c, node.index, "nonce")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            session
                .cryptographic_nonce(&mut c, &copy)
                .unwrap()
                .code_units(),
            &[0, 32, 0xd800, 0xffff]
        );
        session.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }
    #[test]
    fn nonce_standalone_insert_and_reinsert_do_not_hide_or_recopy_slot() {
        let (mut c, mut session, node, _) = fixture();
        let value = supplied(&mut c, &[83]);
        session
            .set_cryptographic_nonce(&mut c, &node, Some(value))
            .unwrap();
        let root = session.root(&mut c).unwrap();
        for _ in 0..2 {
            session.insert(&mut c, &root, &node, None, None).unwrap();
        }
        assert_eq!(
            session
                .cryptographic_nonce(&mut c, &node)
                .unwrap()
                .code_units(),
            &[83]
        );
        assert_eq!(
            session
                .active()
                .unwrap()
                .form_attribute(&mut c, node.index, "nonce")
                .unwrap()
                .unwrap(),
            &[65]
        );
        session.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }
    #[test]
    fn nonce_cancelled_and_foreign_setter_disposes_original_incoming_owner() {
        let (mut c, mut session, node, cancellation) = fixture();
        let (mut other, mut foreign, foreign_node, _) = fixture();
        let value = supplied(&mut c, &[66]);
        assert_eq!(
            session.set_cryptographic_nonce(&mut c, &foreign_node, Some(value)),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(
            session
                .cryptographic_nonce(&mut c, &node)
                .unwrap()
                .code_units(),
            &[65]
        );
        let value = supplied(&mut c, &[67]);
        cancellation.cancel();
        let result = session.set_cryptographic_nonce(&mut c, &node, Some(value));
        assert!(matches!(
            result,
            Err(SessionError::Stop(HtmlParseStop::Cancelled))
        ));
        session.release(&mut c).unwrap();
        foreign.release(&mut other).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        crate::parser::tree_builder::tests::assert_released(&other);
    }
    #[test]
    fn nonce_lookup_exact_work_and_one_short_preserve_first_stop_and_cleanup() {
        // The lookup's whole cost is measured from an unconstrained twin; the
        // exact budget then completes it and one credit less stops it.
        let cost = {
            let (mut c, mut session, node, _) = fixture();
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            session
                .active()
                .unwrap()
                .nonce_slot(&mut c, node.index)
                .unwrap()
                .unwrap();
            let cost = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
            cost
        };
        for short in [false, true] {
            let (mut c, mut session, node, cancellation) = fixture();
            let storage = session.active().unwrap();
            let slot = storage.payloads.values.iter().position(|entry| {
                matches!(entry, RetainedPayload::Nonce { node: Some(owner), .. } if *owner == node.index)
            }).unwrap();
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let maximum = super::super::tests::limits().work_credits();
            c.charge(
                HtmlParseWorkCategory::StateStep,
                maximum - used - cost + u64::from(short),
            )
            .unwrap();
            let before = c.usage().clone();
            let result = session.active().unwrap().nonce_slot(&mut c, node.index);
            if short {
                let first = c.stop().unwrap();
                assert!(matches!(first, HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    requested_delta,
                    ..
                } if requested_delta == const { (2 * mem::size_of::<usize>()) as u64 }));
                assert_eq!(result, Err(SessionError::Stop(first)));
                let snapshot = c.first_stop_usage().cloned();
                cancellation.cancel();
                assert_eq!(
                    session.active().unwrap().nonce_slot(&mut c, node.index),
                    Err(SessionError::Stop(first))
                );
                assert_eq!(c.first_stop_usage(), snapshot.as_ref());
            } else {
                assert_eq!(result.unwrap().unwrap().0, slot);
                assert_eq!(
                    c.usage().work(HtmlParseWorkCategory::CopyByte)
                        - before.work(HtmlParseWorkCategory::CopyByte),
                    const { (2 * mem::size_of::<usize>()) as u64 }
                );
                // Exact useful-work budget exhausted; stop before disposing admitted owners.
                c.charge(HtmlParseWorkCategory::StateStep, 1).unwrap_err();
            }
            let first = c.stop();
            let at_stop = c.first_stop_usage().cloned();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
            assert_eq!(c.stop(), first);
            assert_eq!(c.first_stop_usage(), at_stop.as_ref());
        }
    }
    #[test]
    fn nonce_empty_backing_and_slot_have_two_actual_auxiliary_admissions() {
        for cap in [1, 2, 3] {
            let limits = super::super::tests::limits()
                .into_builder()
                .auxiliary_entries(cap)
                .active_formatting_entries(cap)
                .finish()
                .unwrap();
            let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            // The actual fragment root owns one node-join cell, with no Document metadata. The
            // nonce producer adds exactly its empty backing and metadata slot.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                1
            );
            let name = QualifiedName::new(Namespace::html(), None, "p");
            let result = session
                .active_mut()
                .unwrap()
                .prepare_nonce(&mut c, Some(&name));
            if cap == 3 {
                let slot = result.unwrap().unwrap();
                let storage = session.active().unwrap();
                let RetainedPayload::Nonce { node, raw } = storage.payloads.values[slot] else {
                    panic!("actual nonce record");
                };
                assert_eq!(node, None); // Native binding belongs to the later element commit.
                let RetainedPayload::String { ref value, .. } = storage.payloads.values[raw] else {
                    panic!("actual empty backing");
                };
                assert_eq!(value.code_units(), &[]);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    3
                );
            } else {
                let first = c.stop().unwrap();
                assert_eq!(result, Err(SessionError::Stop(first)));
                assert!(
                    matches!(first, HtmlParseStop::LimitExceeded { resource: HtmlParseResource::AuxiliaryEntries, limit, current, requested_delta: 1 } if limit == cap && current == cap)
                );
                let storage = session.active().unwrap();
                assert_eq!(storage.payloads.values.len(), (cap - 1) as usize);
                assert!(
                    !storage
                        .payloads
                        .values
                        .iter()
                        .any(|entry| matches!(entry, RetainedPayload::Nonce { .. }))
                );
                if cap == 2 {
                    assert!(
                        matches!(&storage.payloads.values[0], RetainedPayload::String { value, .. } if value.is_empty())
                    );
                }
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
            }
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                0
            );
            let first = c.stop();
            let at_stop = c.first_stop_usage().cloned();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
            assert_eq!(c.stop(), first);
            assert_eq!(c.first_stop_usage(), at_stop.as_ref());
        }
    }
    fn connected_request() -> crate::HtmlFragmentParseRequest {
        use crate::*;
        use surgeist_html::{
            construction::HtmlDocumentMode,
            parsing::HtmlParserScriptingMode,
            security::nonce::{
                HtmlBrowsingContextPresence, HtmlHeaderDeliveredCsp, HtmlNonceDocumentEnvironment,
            },
            values::{Namespace, OrderedAttributes, QualifiedName},
        };
        let mut b = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = b
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        b.establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
        b.establish_document_nonce_environment(
            &document,
            HtmlNonceDocumentEnvironment::new(
                HtmlBrowsingContextPresence::Present,
                HtmlHeaderDeliveredCsp::Present,
            ),
        )
        .unwrap();
        let host = b
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        b.establish_children(&document, std::slice::from_ref(&host))
            .unwrap();
        b.establish_no_shadow_root(&host).unwrap();
        let capture = b.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&host).unwrap()).unwrap();
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8("<template shadowrootmode=open><p nonce=A></p></template>"),
            context,
            super::super::tests::limits(),
            HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
    }
    fn complete_connected(driver: &mut crate::parser::tree_builder::ContextFragmentDriver<'_>) {
        for _ in 0..256 {
            match driver.step().unwrap() {
                crate::parser::tree_builder::head::HeadStep::Complete => return,
                crate::parser::tree_builder::head::HeadStep::Consumed => {}
                _ => panic!("literal complete fragment"),
            }
        }
        panic!("finite literal did not complete")
    }
    fn p_node(c: &mut ParseControl, session: &ParserArenaSession) -> ParserNode {
        let storage = session.active().unwrap();
        let index = (0..storage.construction.len()).find(|&position|matches!(storage.construction.node_at(position).unwrap().kind(),ConstructionNodeKind::Element(e) if e.name().local_name()==&DocumentString::from("p"))).unwrap();
        storage.node(c, index).unwrap()
    }
    #[test]
    fn nonce_connected_reinsertion_preserves_set_slot_not_current_content_and_clone() {
        let request = connected_request();
        let mut driver = crate::parser::tree_builder::initialized_fragment_fixture(&request);
        complete_connected(&mut driver);
        let (c, session, _) = driver.input_fixture_parts().unwrap();
        let node = p_node(c, session);
        assert_eq!(
            session.cryptographic_nonce(c, &node).unwrap().code_units(),
            &[65]
        );
        assert_eq!(
            session
                .active()
                .unwrap()
                .form_attribute(c, node.index, "nonce")
                .unwrap()
                .unwrap(),
            &[]
        );
        let value = supplied(c, &[83]);
        session
            .set_cryptographic_nonce(c, &node, Some(value))
            .unwrap();
        let value = supplied(c, &[66]);
        session
            .active_mut()
            .unwrap()
            .set_nonce_content(c, node.index, Some(value))
            .unwrap();
        assert_eq!(
            session.cryptographic_nonce(c, &node).unwrap().code_units(),
            &[66]
        );
        assert_eq!(
            session
                .active()
                .unwrap()
                .form_attribute(c, node.index, "nonce")
                .unwrap()
                .unwrap(),
            &[66]
        );
        let value = supplied(c, &[83]);
        session
            .set_cryptographic_nonce(c, &node, Some(value))
            .unwrap();
        let storage = session.active().unwrap();
        let parent = storage
            .node(
                c,
                storage
                    .construction
                    .node_at(node.index)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position())
                    .unwrap(),
            )
            .unwrap();
        session.insert(c, &parent, &node, None, None).unwrap();
        assert_eq!(
            session.cryptographic_nonce(c, &node).unwrap().code_units(),
            &[83]
        );
        assert_eq!(
            session
                .active()
                .unwrap()
                .form_attribute(c, node.index, "nonce")
                .unwrap()
                .unwrap(),
            &[]
        );
        let document = session
            .active()
            .unwrap()
            .node_document_ref(c, node.index)
            .unwrap();
        let copy = session
            .active_mut()
            .unwrap()
            .clone_node(c, node.index, document)
            .unwrap();
        let copy = session.active().unwrap().node(c, copy).unwrap();
        assert_eq!(
            session.cryptographic_nonce(c, &copy).unwrap().code_units(),
            &[83]
        );
        assert_eq!(
            session
                .active()
                .unwrap()
                .form_attribute(c, copy.index, "nonce")
                .unwrap()
                .unwrap(),
            &[]
        );
        driver.release().unwrap();
    }
    #[test]
    fn nonce_cancel_after_real_save_or_empty_update_keeps_first_stop_and_all_cleanup_owners() {
        for point in [NonceCancelPoint::Saved, NonceCancelPoint::ContentEmptied] {
            let request = connected_request();
            let mut driver = crate::parser::tree_builder::initialized_fragment_fixture(&request);
            complete_connected(&mut driver);
            let (c, session, _) = driver.input_fixture_parts().unwrap();
            let node = p_node(c, session);
            let value = supplied(c, &[66]);
            session
                .active_mut()
                .unwrap()
                .set_nonce_content(c, node.index, Some(value))
                .unwrap();
            let value = supplied(c, &[83]);
            session
                .set_cryptographic_nonce(c, &node, Some(value))
                .unwrap();
            let observation =
                CancellationObservation::install(point, request.cancellation().clone());
            let error = session
                .active_mut()
                .unwrap()
                .nonce_inserted(c, node.index)
                .unwrap_err();
            assert!(observation.reached());
            assert_eq!(error, SessionError::Stop(HtmlParseStop::Cancelled));
            let first = c.stop();
            let at_stop = c.first_stop_usage().cloned();
            assert!(!session.active().unwrap().payloads.values.is_empty());
            assert_eq!(
                session.active_mut().unwrap().nonce_inserted(c, node.index),
                Err(SessionError::Stop(HtmlParseStop::Cancelled))
            );
            assert_eq!(c.stop(), first);
            assert_eq!(c.first_stop_usage(), at_stop.as_ref());
            let mut input = Some(driver);
            let mut failed =
                crate::parser::tree_builder::ContextFragmentDriver::take_failed(&mut input)
                    .unwrap();
            assert!(input.is_none());
            assert_eq!(failed.stop(), HtmlParseStop::Cancelled);
            assert_eq!(failed.at_stop(), at_stop.as_ref().unwrap());
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::ManagedEdges,
                HtmlParseResource::ManagedAttributes,
                HtmlParseResource::ManagedUtf16Units,
            ] {
                assert_eq!(
                    failed.after_cleanup().resource(resource).current,
                    0,
                    "{resource:?}"
                );
            }
            failed.release().unwrap();
        }
    }
}
