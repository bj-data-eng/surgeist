//! HTML Table-part creation and replacement under one retained Source edit.

use super::*;
use crate::internal::{
    compound::TablePartEdit,
    hosted::DomCreationRealmFallback,
    source::{DomElementCreationName, DomElementCreationOptions},
    table_part::DomHtmlTablePart,
};
use surgeist_html::{values::Namespace, vocabulary::KnownHtmlElement};

/// The exact insertion still owed after canonical removal and its native effects.
#[derive(Clone, Copy)]
pub(in crate::internal::store) struct Insertion {
    table: Key,
    candidate: Key,
    part: DomHtmlTablePart,
}

impl Draft {
    pub(super) fn edit_html_table_part(
        &mut self,
        table: Key,
        part: DomHtmlTablePart,
        action: TablePartEdit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.table_part_kind(table, cancel)? != Some(KnownHtmlElement::Table) {
            return Err(DomError::WrongNodeKind);
        }

        match action {
            TablePartEdit::Create { fallback } => {
                if let Some(part) = self.first_html_table_part(table, part, cancel)? {
                    return self.register_ticket(part, cancel);
                }
                let document = self.node(table)?.document();
                let candidate =
                    self.form_html_table_part(document, part, fallback, limits, cancel)?;
                self.register_ticket(candidate, cancel)?;
                self.insert_html_table_part(
                    Insertion {
                        table,
                        candidate,
                        part,
                    },
                    limits,
                    cancel,
                )
            }
            TablePartEdit::Set(value) => {
                let candidate = if let Some(value) = value {
                    self.source_step(cancel)?;
                    let node = self.resolve(value.into())?;
                    if self.table_part_kind(node, cancel)? != Some(part.element()) {
                        self.prepare_compound_failure_slot(cancel)?;
                        return self.compound_error(DomError::HierarchyRequest);
                    }
                    Some(node)
                } else {
                    None
                };
                let insertion = candidate.map(|candidate| Insertion {
                    table,
                    candidate,
                    part,
                });
                let Some(current) = self.first_html_table_part(table, part, cancel)? else {
                    return match insertion {
                        Some(frame) => self.insert_html_table_part(frame, limits, cancel),
                        None => Ok(()),
                    };
                };
                if let Some(frame) = insertion {
                    // The removal pushes its actual effects above this tail.
                    self.push_effect(
                        select::Effect::TablePartInsert(frame),
                        self.effect_scoped,
                        cancel,
                    )?;
                }
                self.remove(current, limits, cancel)
            }
        }
    }

    fn table_part_kind(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<KnownHtmlElement>, DomError> {
        self.source_step(cancel)?;
        let transaction::KindRead::Element(element) = self.node(key)?.kind() else {
            return Ok(None);
        };
        Ok(KnownHtmlElement::from_name(element.name()))
    }

    fn first_html_table_part(
        &self,
        table: Key,
        part: DomHtmlTablePart,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        part.first_with(self.node(table)?.children().iter(), |child| {
            self.table_part_kind(child, cancel)
        })
    }

    fn form_html_table_part(
        &mut self,
        document: Key,
        part: DomHtmlTablePart,
        fallback: Option<DomCreationRealmFallback>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        let operands = source::formation::Formation::new(self, cancel)?;
        let uri =
            source::formation::string("http://www.w3.org/1999/xhtml", operands.as_ref(), cancel)?;
        let local =
            source::formation::string(part.element().local_name(), operands.as_ref(), cancel)?;
        let name = DomElementCreationName::Qualified(Namespace::from_uri(uri), local);
        let formation = source::formation::Formation::new(self, cancel)?;
        let element = self.native_element(
            &name,
            &DomElementCreationOptions::default(),
            self.document_metadata(document)?,
            formation.as_ref(),
            cancel,
        )?;
        drop(name);
        drop(operands);
        let key = self.allocate_element_formed(element, document, limits, cancel, formation)?;
        if self.node(key)?.relevant_global().is_none() && fallback.is_some() {
            let global = self.resolve_creation_global(document, fallback, cancel)?;
            self.establish_creation_global(key, global, cancel)?;
        }
        self.initialize_native_element(key, None, limits, cancel)?;
        Ok(key)
    }

    fn html_table_part_reference(
        &self,
        table: Key,
        part: DomHtmlTablePart,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        let children = self.node(table)?.children();
        match part {
            DomHtmlTablePart::Caption => Ok(children.first()),
            DomHtmlTablePart::Head => {
                for child in children.iter() {
                    self.source_step(cancel)?;
                    if let transaction::KindRead::Element(element) = self.node(child)?.kind()
                        && !matches!(
                            KnownHtmlElement::from_name(element.name()),
                            Some(KnownHtmlElement::Caption | KnownHtmlElement::Colgroup)
                        )
                    {
                        return Ok(Some(child));
                    }
                }
                Ok(None)
            }
            DomHtmlTablePart::Foot => Ok(None),
        }
    }

    pub(in crate::internal::store) fn insert_html_table_part(
        &mut self,
        frame: Insertion,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Insertion {
            table,
            candidate,
            part,
        } = frame;
        self.prepare_compound_failure_slot(cancel)?;
        let before = (|| {
            let before = self.html_table_part_reference(table, part, cancel)?;
            self.ensure_pre_insert(candidate, table, before, None, false, cancel)?;
            if before == Some(candidate) {
                self.compound_next(candidate, cancel)
            } else {
                Ok(before)
            }
        })();
        match before {
            Ok(before) => self.insert_validated(candidate, table, before, limits, cancel),
            Err(error) => self.compound_error(error),
        }
    }
}
