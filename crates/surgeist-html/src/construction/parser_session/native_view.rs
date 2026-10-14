//! Checked borrowed observation of the actual canonical construction records.
//! This reader never mutates the session or copies its tree. Payload strings
//! expose individual admitted UTF-16 reads, not an unmetered backing slice.

use super::*;

#[derive(Clone, Copy)]
pub(crate) struct NativeString<'a> {
    value: &'a DocumentString,
    owner: &'a ControlIdentity,
    len: usize,
}
impl NativeString<'_> {
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.len
    }
}
#[derive(Clone, Copy)]
pub(crate) struct NativeName<'a> {
    pub(crate) namespace: Option<NativeString<'a>>,
    pub(crate) prefix: Option<NativeString<'a>>,
    pub(crate) local: NativeString<'a>,
}
#[derive(Clone, Copy)]
pub(crate) struct NativeAttribute<'a> {
    pub(crate) name: NativeName<'a>,
    pub(crate) value: NativeString<'a>,
}
pub(crate) enum NativeKind<'a> {
    Document(HtmlDocumentMode),
    Fragment,
    Element(NativeName<'a>),
    Text(NativeString<'a>),
    Comment(NativeString<'a>),
    ProcessingInstruction {
        target: NativeString<'a>,
        data: NativeString<'a>,
    },
    Doctype {
        name: NativeString<'a>,
        public_id: Option<NativeString<'a>>,
        system_id: Option<NativeString<'a>>,
    },
}
pub(crate) enum NativeFragmentOwner {
    Template(ParserNode),
    ShadowHost(ParserNode),
    CapturedShadowHost(crate::HtmlCaptureElementId),
}
pub(crate) struct NativeShadow {
    pub(crate) host: ParserNode,
    pub(crate) contents: ParserNode,
    pub(crate) properties: HtmlShadowRootProperties,
    pub(crate) declarative: Option<bool>,
    pub(crate) keep_registry_null: Option<bool>,
    pub(crate) available_to_internals: Option<bool>,
}

/// A checked read-only record borrow plus the sole mutable resource controller.
/// Lending control supports actual admitted consumer output/scratch; it never
/// lends the session, records, payload registry or an evidence mutation path.
pub(crate) struct NativeView<'a> {
    storage: &'a SessionStorage,
    control: &'a mut ParseControl,
}
impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn native_view<'a>(
        &'a self,
        control: &'a mut ParseControl,
    ) -> Result<NativeView<'a>, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<NativeView>() as u64)?;
        Ok(NativeView { storage, control })
    }
}
impl<'a> NativeView<'a> {
    #[cfg(test)]
    pub(crate) fn control(&mut self) -> &mut ParseControl {
        self.control
    }
    #[cfg(test)]
    pub(crate) fn usage(&self) -> &crate::parser::control::ParseUsage {
        self.control.usage()
    }
    #[cfg(test)]
    pub(crate) fn root(&mut self) -> Result<ParserNode, SessionError> {
        self.storage.root(self.control)
    }
    #[cfg(test)]
    pub(crate) fn same_node(
        &mut self,
        left: &ParserNode,
        right: &ParserNode,
    ) -> Result<bool, SessionError> {
        Ok(self.storage.index(self.control, left)? == self.storage.index(self.control, right)?)
    }
    #[cfg(test)]
    fn string(&self, value: &'a DocumentString) -> NativeString<'a> {
        NativeString {
            value,
            owner: &self.storage.control,
            len: value.len(),
        }
    }
    #[cfg(test)]
    fn name(&self, name: &'a QualifiedName) -> NativeName<'a> {
        NativeName {
            namespace: name.namespace().uri().map(|s| self.string(s)),
            prefix: name.prefix().map(|s| self.string(s)),
            local: self.string(name.local_name()),
        }
    }
    /// The kind and fixed native field headers are inspected only after their
    /// admission. Variable contents remain behind individually charged reads.
    #[cfg(test)]
    pub(crate) fn kind(&mut self, node: &ParserNode) -> Result<NativeKind<'a>, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.control.charge(
            WorkCategory::InspectByte,
            mem::size_of::<HtmlNodeKind>() as u64,
        )?;
        self.control
            .charge(WorkCategory::CopyByte, mem::size_of::<NativeKind>() as u64)?;
        Ok(
            match &self
                .storage
                .construction
                .node_at(index)
                .expect("retained construction position")
                .kind()
            {
                HtmlNodeKind::Document(mode) => NativeKind::Document(*mode),
                HtmlNodeKind::Fragment => NativeKind::Fragment,
                HtmlNodeKind::Element(element) => NativeKind::Element(self.name(element.name())),
                HtmlNodeKind::Text(text) => NativeKind::Text(self.string(text)),
                HtmlNodeKind::Comment(text) => NativeKind::Comment(self.string(text)),
                HtmlNodeKind::ProcessingInstruction(pi) => NativeKind::ProcessingInstruction {
                    target: self.string(pi.target()),
                    data: self.string(pi.data()),
                },
                HtmlNodeKind::Doctype(doctype) => NativeKind::Doctype {
                    name: self.string(doctype.name()),
                    public_id: doctype.public_id().map(|s| self.string(s)),
                    system_id: doctype.system_id().map(|s| self.string(s)),
                },
            },
        )
    }
    #[cfg(test)]
    pub(crate) fn unit(
        &mut self,
        value: &NativeString<'_>,
        index: usize,
    ) -> Result<Option<u16>, SessionError> {
        self.storage.require(self.control)?;
        self.control.require_identity(value.owner)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        if index >= value.len {
            return Ok(None);
        }
        self.control
            .charge(WorkCategory::InspectByte, mem::size_of::<u16>() as u64)?;
        Ok(Some(value.value.code_units()[index]))
    }
    #[cfg(test)]
    pub(crate) fn attribute_count(&mut self, node: &ParserNode) -> Result<usize, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.control.charge(
            WorkCategory::InspectByte,
            mem::size_of::<HtmlNodeKind>() as u64,
        )?;
        match &self
            .storage
            .construction
            .node_at(index)
            .expect("retained construction position")
            .kind()
        {
            HtmlNodeKind::Element(element) => Ok(element.attributes().len()),
            _ => Err(SessionError::WrongToken),
        }
    }
    #[cfg(test)]
    pub(crate) fn attribute_at(
        &mut self,
        node: &ParserNode,
        position: usize,
    ) -> Result<Option<NativeAttribute<'a>>, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .storage
            .construction
            .node_at(index)
            .expect("retained construction position")
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        if position >= element.attributes().len() {
            return Ok(None);
        }
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.control.charge(
            WorkCategory::InspectByte,
            mem::size_of::<Attribute>() as u64,
        )?;
        self.control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<NativeAttribute>() as u64,
        )?;
        let attribute = &element.attributes().as_slice()[position];
        Ok(Some(NativeAttribute {
            name: self.name(attribute.name()),
            value: self.string(attribute.value()),
        }))
    }
    #[cfg(test)]
    pub(crate) fn child_count(&mut self, node: &ParserNode) -> Result<usize, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self
            .storage
            .construction
            .node_at(index)
            .expect("retained construction position")
            .child_count())
    }
    #[cfg(test)]
    pub(crate) fn child_at(
        &mut self,
        node: &ParserNode,
        position: usize,
    ) -> Result<Option<ParserNode>, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        if position
            >= self
                .storage
                .construction
                .node_at(index)
                .expect("retained construction position")
                .child_count()
        {
            return Ok(None);
        }
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        let child = self
            .storage
            .construction
            .node_at(index)
            .expect("retained construction position")
            .child_at(position)
            .expect("retained child ordinal")
            .position();
        self.storage.node(self.control, child).map(Some)
    }
    #[cfg(test)]
    pub(crate) fn parent(&mut self, node: &ParserNode) -> Result<Option<ParserNode>, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.storage
            .construction
            .node_at(index)
            .expect("retained construction position")
            .parent()
            .map(|parent| self.storage.node(self.control, parent.position()))
            .transpose()
    }
    #[cfg(test)]
    pub(crate) fn template_contents(
        &mut self,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.storage
            .construction
            .node_at(index)
            .expect("retained construction position")
            .template_contents()
            .map(|contents| self.storage.node(self.control, contents.position()))
            .transpose()
    }
    #[cfg(test)]
    fn association_node(&mut self, node: &HtmlNodeId) -> Result<ParserNode, SessionError> {
        self.storage.require(self.control)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        let position = self
            .storage
            .construction
            .storage()
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        self.storage.node(self.control, position)
    }
    #[cfg(test)]
    pub(crate) fn fragment_owner(
        &mut self,
        node: &ParserNode,
    ) -> Result<Option<NativeFragmentOwner>, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        match &self
            .storage
            .construction
            .node_at(index)
            .expect("retained construction position")
            .fragment_owner()
        {
            Some(HtmlFragmentOwner::Template(owner)) => self
                .association_node(owner)
                .map(|node| Some(NativeFragmentOwner::Template(node))),
            Some(HtmlFragmentOwner::ShadowHost(owner)) => self
                .association_node(owner)
                .map(|node| Some(NativeFragmentOwner::ShadowHost(node))),
            None => {
                if let Some(fragment) = &self.storage.fragment {
                    for effect in fragment.effects.as_slice() {
                        self.control.charge(WorkCategory::RecordVisit, 1)?;
                        if let context::ContextEffect::AttachShadow {
                            host: context::ShadowHostKey::Captured(host),
                            contents: Some(root),
                            ..
                        } = effect
                            && *root == index
                        {
                            self.control.charge(WorkCategory::ProviderCall, 1)?;
                            payload_write::<NativeFragmentOwner>(self.control, 1)?;
                            return Ok(Some(NativeFragmentOwner::CapturedShadowHost(host.clone())));
                        }
                    }
                }
                Ok(None)
            }
        }
    }
    #[cfg(test)]
    pub(crate) fn shadow(
        &mut self,
        node: &ParserNode,
    ) -> Result<Option<NativeShadow>, SessionError> {
        let index = self.storage.index(self.control, node)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        let Some(shadow) = self
            .storage
            .construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?
            .shadow_root()
        else {
            return Ok(None);
        };
        self.control.charge(
            WorkCategory::InspectByte,
            mem::size_of::<HtmlShadowRootProperties>() as u64,
        )?;
        self.control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<NativeShadow>() as u64,
        )?;
        let state = if self.storage.fragment.is_some() {
            Some(
                self.storage
                    .shadow_state(self.control, shadow.contents().index)?,
            )
        } else {
            None
        };
        Ok(Some(NativeShadow {
            declarative: state.map(|s| s.declarative),
            keep_registry_null: state.map(|s| s.keep_registry_null),
            available_to_internals: state.map(|s| s.available_to_internals),
            host: self.association_node(shadow.host())?,
            contents: self.association_node(shadow.contents())?,
            properties: shadow.properties(),
        }))
    }
}
