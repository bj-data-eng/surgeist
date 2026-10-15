use std::sync::{Arc, atomic::AtomicBool};

use surgeist_html::bounded::{
    HtmlPiPseudoAttributeAction as Action, HtmlPiPseudoAttributeCursor as Cursor,
    HtmlPiPseudoAttributeError as CursorError, HtmlPiPseudoAttributeField as Field,
    HtmlPiPseudoAttributeName as Name, HtmlPiPseudoAttributeReply as Reply, is_xml_name_code_point,
    is_xml_name_start_code_point,
};

use crate::store::{check_cancel, reserve};
use crate::{DocumentString, DomError};

/// Canonical PI target, CharacterData and the map derived from that exact data.
/// Construction applies DOM's XML Name rule, independently of HTML authoring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomProcessingInstruction {
    target: DocumentString,
    data: DocumentString,
    attributes: Arc<Vec<(DocumentString, DocumentString)>>,
}

impl DomProcessingInstruction {
    pub(crate) fn new(
        target: DocumentString,
        data: DocumentString,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        let valid_target = {
            let mut points = target.code_points();
            points.next().is_some_and(is_xml_name_start_code_point)
                && points.all(is_xml_name_code_point)
        };
        if !valid_target
            || data
                .code_units()
                .windows(2)
                .any(|pair| pair == [0x3f, 0x3e])
        {
            return Err(DomError::InvalidCharacter);
        }
        Self::from_data(target, data, cancelled)
    }

    /// CharacterData replacement does not rerun PI initialization's syntax
    /// rejection; even data containing ?> is retained and its map recomputed.
    pub(crate) fn with_data(
        &self,
        data: DocumentString,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        Self::from_data(self.target.clone(), data, cancelled)
    }

    fn from_data(
        target: DocumentString,
        data: DocumentString,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        let attributes = parse_attributes(&data, cancelled)?;
        Ok(Self {
            target,
            data,
            attributes: Arc::new(attributes),
        })
    }

    pub fn target(&self) -> &DocumentString {
        &self.target
    }
    pub fn data(&self) -> &DocumentString {
        &self.data
    }
    /// Whole-string XML pseudo-attributes, in source order. Invalid data has an
    /// empty map; it is never replaced by partially parsed successful fields.
    pub fn attributes(&self) -> impl ExactSizeIterator<Item = (&DocumentString, &DocumentString)> {
        self.attributes.iter().map(|(name, value)| (name, value))
    }
    pub fn attribute(&self, name: &DocumentString) -> Option<&DocumentString> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.attributes.iter().try_fold(
            self.target.len().checked_add(self.data.len())?,
            |total, (name, value)| total.checked_add(name.len())?.checked_add(value.len()),
        )
    }
}

/// Source-defined DocumentType name and exact public/system identifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomDocumentType {
    name: DocumentString,
    public_id: DocumentString,
    system_id: DocumentString,
}
impl DomDocumentType {
    pub(crate) fn new(
        name: DocumentString,
        public_id: DocumentString,
        system_id: DocumentString,
    ) -> Result<Self, DomError> {
        // DOM §1.4 deliberately permits the empty name and non-XML spellings.
        if name
            .code_units()
            .iter()
            .any(|unit| matches!(*unit, 0 | 9 | 10 | 12 | 13 | 32 | 0x3e))
        {
            return Err(DomError::InvalidCharacter);
        }
        Ok(Self {
            name,
            public_id,
            system_id,
        })
    }
    pub fn name(&self) -> &DocumentString {
        &self.name
    }
    pub fn public_id(&self) -> &DocumentString {
        &self.public_id
    }
    pub fn system_id(&self) -> &DocumentString {
        &self.system_id
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.name
            .len()
            .checked_add(self.public_id.len())?
            .checked_add(self.system_id.len())
    }
}

fn parse_attributes(
    data: &DocumentString,
    cancelled: &AtomicBool,
) -> Result<Vec<(DocumentString, DocumentString)>, DomError> {
    let mut cursor = Cursor::new(data.len());
    let mut name = Vec::new();
    let mut value = Vec::new();
    let mut pairs: Vec<(DocumentString, DocumentString)> = Vec::new();
    while let Some(action) = cursor.action() {
        check_cancel(cancelled)?;
        let reply = match action {
            Action::Read(offset) => Reply::Unit(data.code_units().get(offset).copied()),
            Action::Step => Reply::Ack,
            Action::Begin(field) => {
                match field {
                    Field::Name => name.clear(),
                    Field::Value => value.clear(),
                }
                Reply::Ack
            }
            Action::Emit {
                field, units, len, ..
            } => {
                let target = match field {
                    Field::Name => &mut name,
                    Field::Value => &mut value,
                };
                reserve(target, len)?;
                target.extend_from_slice(&units[..len]);
                Reply::Ack
            }
            Action::NameLength(index) => Reply::Length(pairs[index].0.len()),
            Action::NameUnit(which, offset) => Reply::Unit(Some(match which {
                Name::Current => name[offset],
                Name::Pair(index) => pairs[index].0.code_units()[offset],
            })),
            Action::Commit => {
                reserve(&mut pairs, 1)?;
                pairs.push((
                    DocumentString::from_utf16(std::mem::take(&mut name)),
                    DocumentString::from_utf16(std::mem::take(&mut value)),
                ));
                Reply::Ack
            }
            Action::Discard => {
                name.clear();
                value.clear();
                Reply::Ack
            }
            Action::Pop => {
                pairs.pop();
                Reply::Ack
            }
            Action::Ready(valid) => {
                if !valid {
                    pairs.clear();
                }
                Reply::Ack
            }
        };
        match cursor.reply(reply) {
            Ok(()) => {}
            Err(CursorError::Representation) => return Err(DomError::RepresentationExhausted),
            Err(CursorError::Integrity) => {
                unreachable!("PI cursor replies reflect its actual input and field banks")
            }
        }
    }
    Ok(pairs)
}
